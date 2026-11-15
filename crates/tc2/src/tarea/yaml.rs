//! Costura YAML del modelo de tarea (ADR F1-14, punto 1: el YAML vive detras
//! de un modulo propio para poder cambiar de crate sin tocar el resto).
//!
//! - Lectura: [`partir`] separa el frontmatter en entradas de primer nivel con
//!   su texto crudo (para conservar las claves desconocidas byte a byte) y
//!   [`decodificar`] valida el documento entero con `serde-saphyr` (claves
//!   duplicadas, anclas, alias y claves de fusion como error) y decodifica
//!   las claves conocidas.
//! - Escritura: emisor propio ([`cadena`], [`lista`]...), porque F1-9 fija la
//!   forma byte a byte (comillas dobles siempre, orden de claves, `roles` en
//!   flujo) y un serializador generico no la da.

use serde::de::DeserializeOwned;
use serde_saphyr::{DuplicateKeyPolicy, MergeKeyPolicy, Options};

use super::ErrorTarea;

/// Delimitador del frontmatter.
const DELIMITADOR: &str = "---";

/// Una entrada de primer nivel del frontmatter: su clave y sus lineas tal
/// cual (con `\n` al final de cada una).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Entrada {
    pub clave: String,
    pub crudo: String,
}

/// Un `tarea.md` partido: entradas del frontmatter y el cuerpo intacto.
#[derive(Debug)]
pub(crate) struct Partes<'a> {
    pub entradas: Vec<Entrada>,
    pub cuerpo: &'a str,
}

impl Partes<'_> {
    /// Si el frontmatter tiene la clave `clave`.
    pub fn tiene(&self, clave: &str) -> bool {
        self.entradas.iter().any(|e| e.clave == clave)
    }

    /// El texto YAML del frontmatter (sin delimitadores), normalizado a `\n`.
    pub fn yaml(&self) -> String {
        self.entradas.iter().map(|e| e.crudo.as_str()).collect()
    }
}

/// Si `clave` es una clave de primer nivel admitida: minusculas ASCII,
/// cifras, `_` y `-`. Las claves entre comillas o complejas se rechazan para
/// que la clave que ve el separador sea la misma que ve el parser.
fn clave_admitida(clave: &str) -> bool {
    !clave.is_empty()
        && clave
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

/// Parte `texto` en frontmatter y cuerpo. Admite BOM UTF-8 y finales `\r\n`
/// (el frontmatter se normaliza a `\n`; el cuerpo queda intacto).
pub(crate) fn partir(texto: &str) -> Result<Partes<'_>, ErrorTarea> {
    let texto = texto.strip_prefix('\u{feff}').unwrap_or(texto);
    let mut lineas = texto.split_inclusive('\n');
    let sin_fin = |l: &str| l.trim_end_matches('\n').trim_end_matches('\r').to_owned();
    match lineas.next() {
        Some(l) if sin_fin(l) == DELIMITADOR => {}
        _ => return Err(ErrorTarea::general("no empieza con un frontmatter `---`")),
    }
    let mut consumido = texto.split_inclusive('\n').next().map_or(0, str::len);
    let mut entradas: Vec<Entrada> = Vec::new();
    let mut cerrado = false;
    for l in lineas {
        consumido += l.len();
        let linea = sin_fin(l);
        if linea == DELIMITADOR {
            cerrado = true;
            break;
        }
        let continua = linea.is_empty() || linea.starts_with([' ', '\t', '#']);
        if continua {
            match entradas.last_mut() {
                Some(e) => {
                    e.crudo.push_str(&linea);
                    e.crudo.push('\n');
                }
                None if linea.trim().is_empty() => {}
                None => {
                    return Err(ErrorTarea::general(
                        "texto antes de la primera clave del frontmatter",
                    ));
                }
            }
            continue;
        }
        let Some((clave, _)) = linea.split_once(':') else {
            return Err(ErrorTarea::general(format!(
                "linea del frontmatter sin clave: «{linea}»"
            )));
        };
        if !clave_admitida(clave) {
            return Err(ErrorTarea::general(format!(
                "clave no admitida en el frontmatter: «{clave}»"
            )));
        }
        if entradas.iter().any(|e| e.clave == clave) {
            return Err(ErrorTarea::en(clave, "clave duplicada"));
        }
        entradas.push(Entrada {
            clave: clave.to_owned(),
            crudo: format!("{linea}\n"),
        });
    }
    if !cerrado {
        return Err(ErrorTarea::general("el frontmatter no se cierra con `---`"));
    }
    Ok(Partes {
        entradas,
        cuerpo: texto.get(consumido..).unwrap_or(""),
    })
}

/// Opciones de lectura: duplicadas, claves de fusion, anclas y alias como
/// error (ninguna tarea real los usa; un alias que crece es un vector de
/// denegacion de servicio, F1-8).
fn opciones() -> Options {
    let mut o = Options::default();
    o.duplicate_keys = DuplicateKeyPolicy::Error;
    o.merge_keys = MergeKeyPolicy::Error;
    let mut presupuesto = o.budget.unwrap_or_default();
    presupuesto.max_aliases = 0;
    presupuesto.max_anchors = 0;
    o.budget = Some(presupuesto);
    o
}

/// Valida el YAML entero y decodifica las claves conocidas en `T` (las
/// desconocidas se validan como YAML y se ignoran: las conserva [`partir`]).
pub(crate) fn decodificar<T: DeserializeOwned>(yaml: &str) -> Result<T, ErrorTarea> {
    serde_saphyr::from_str_with_options::<T>(yaml, opciones())
        .map_err(|e| ErrorTarea::general(format!("frontmatter YAML no valido: {e}")))
}

/// Cadena YAML entre comillas dobles (F1-9 punto 5): `\` y `"` escapados y
/// los caracteres de control y separadores de linea como `\uXXXX`, para que
/// cualquier texto salga como YAML valido de una sola linea.
pub(crate) fn cadena(texto: &str) -> String {
    let mut s = String::with_capacity(texto.len() + 2);
    s.push('"');
    for c in texto.chars() {
        match c {
            '\\' => s.push_str("\\\\"),
            '"' => s.push_str("\\\""),
            c if c.is_control() || matches!(c, '\u{2028}' | '\u{2029}' | '\u{feff}') => {
                s.push_str(&format!("\\u{:04X}", u32::from(c)));
            }
            c => s.push(c),
        }
    }
    s.push('"');
    s
}

/// Cadena opcional: `null` o [`cadena`].
pub(crate) fn opcional(texto: Option<&str>) -> String {
    texto.map_or_else(|| "null".to_owned(), cadena)
}

/// Lista en flujo de cadenas: `[]` o `["a", "b"]`.
pub(crate) fn lista(elementos: &[String]) -> String {
    let dentro: Vec<String> = elementos.iter().map(|e| cadena(e)).collect();
    format!("[{}]", dentro.join(", "))
}

/// Entero opcional: `null` o el numero.
pub(crate) fn numero(n: Option<u64>) -> String {
    n.map_or_else(|| "null".to_owned(), |n| n.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partir_separa_entradas_y_cuerpo_intacto() -> Result<(), ErrorTarea> {
        let t = "\u{feff}---\r\nid: \"TASK-001\"\r\nroles:\r\n  a: {b: 1}\r\n# nota\r\nx: 1\r\n---\r\n## Objetivo\r\n";
        let p = partir(t)?;
        let claves: Vec<&str> = p.entradas.iter().map(|e| e.clave.as_str()).collect();
        assert_eq!(claves, ["id", "roles", "x"]);
        assert_eq!(p.entradas[1].crudo, "roles:\n  a: {b: 1}\n# nota\n");
        assert_eq!(p.cuerpo, "## Objetivo\r\n");
        assert!(p.tiene("x") && !p.tiene("formato"));
        Ok(())
    }

    #[test]
    fn partir_rechaza_lo_que_el_separador_no_entiende() {
        for (t, error) in [
            ("sin frontmatter\n", "no empieza"),
            ("---\nid: 1\n", "no se cierra"),
            ("---\n  suelto\nid: 1\n---\n", "antes de la primera"),
            ("---\n\"id\": 1\n---\n", "no admitida"),
            ("---\nId: 1\n---\n", "no admitida"),
            ("---\n? id\n: 1\n---\n", "sin clave"),
            ("---\nsolo texto\n---\n", "sin clave"),
            ("---\nid: 1\nid: 2\n---\n", "duplicada"),
        ] {
            let e = partir(t).err().map(|e| e.to_string());
            assert!(
                e.as_deref().is_some_and(|e| e.contains(error)),
                "{t:?}: {e:?}"
            );
        }
    }

    #[derive(serde::Deserialize, Debug)]
    struct Prueba {
        a: Option<String>,
    }

    #[test]
    fn decodificar_rechaza_duplicadas_anidadas_anclas_y_fusion() -> Result<(), ErrorTarea> {
        for (yaml, motivo) in [
            ("a: \"x\"\nb: {c: 1, c: 2}\n", "uplicate"),
            ("a: &x \"v\"\nb: *x\n", "nchor"),
            ("b: {c: 1}\nd:\n  <<: {c: 2}\n", "erge"),
        ] {
            // Control positivo: con las opciones por defecto, anclas y fusion
            // se aceptarian; el rechazo es de las opciones de `opciones()`.
            let e = decodificar::<Prueba>(yaml).err().map(|e| e.to_string());
            assert!(
                e.as_deref().is_some_and(|e| e.contains(motivo)),
                "{yaml:?}: {e:?}"
            );
        }
        assert!(serde_saphyr::from_str::<Prueba>("a: &x \"v\"\nb: *x\n").is_ok());
        let p = decodificar::<Prueba>("a: \"C:\\\\ruta\"\nb: [1, 2]\n")?;
        assert_eq!(p.a.as_deref(), Some("C:\\ruta"));
        Ok(())
    }

    #[test]
    fn cadena_escapa_lo_necesario() {
        assert_eq!(cadena("C:\\r \"x\" ñ"), "\"C:\\\\r \\\"x\\\" ñ\"");
        assert_eq!(cadena("a\nb\u{2028}"), "\"a\\u000Ab\\u2028\"");
        assert_eq!(opcional(None), "null");
        assert_eq!(lista(&[]), "[]");
        assert_eq!(lista(&["a".into(), "b c".into()]), "[\"a\", \"b c\"]");
        assert_eq!(numero(Some(7)), "7");
    }
}
