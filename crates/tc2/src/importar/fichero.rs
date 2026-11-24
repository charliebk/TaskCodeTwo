//! Fichero de import (F3-2, RF-02): texto → [`Lote`]. Puro: no toca disco,
//! git ni la red; todo lo que se puede rechazar sin el estado (limites,
//! formato, claves, referencias locales y ciclos) se rechaza aqui, antes de
//! hablar con el remoto.
//!
//! Formato, el del import de la 0.16 (`src/core/import-parser.ts`) con dos
//! anadidos opcionales:
//!
//! ```text
//! ---                      <- cabecera opcional: lote y sprint
//! lote: fase-3
//! sprint: 3
//! ---
//! ### F3-2 Titulo          <- una entrada; la clave es el primer token
//! > objetivo               <- lineas `>` antes del primer criterio
//! - criterio               <- uno o mas; una linea sangrada lo continua
//! Dependencias: F2-4, TASK-019
//! ```
//!
//! Como en la 0.16: lo anterior al primer `###` se ignora, un encabezado de
//! otro nivel cierra la entrada y sin cabecera el sprint llega por
//! `--sprint`. A diferencia de la 0.16, una entrada mala no se salta: el
//! lote entero se rechaza (todo o nada, F1-2).

use std::collections::{BTreeMap, BTreeSet};

use crate::alta::{Entrada, NoListo};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::importar as txt;
use crate::tarea::{id_valido, titulo_valido};

/// Tamano maximo del fichero (D4 de TASK-029): el tope unico de la lectura
/// acotada (modulo `lectura`), el mismo de `plan guardar`.
pub use crate::lectura::MAX_BYTES;
/// Entradas maximas por lote (D4 de TASK-029).
pub const MAX_ENTRADAS: usize = 200;
/// Longitud maxima de una clave de entrada o de un nombre de lote.
const MAX_NOMBRE: usize = 64;

/// Topes del fichero (parametro, D4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limites {
    /// Bytes.
    pub bytes: usize,
    /// Entradas.
    pub entradas: usize,
}

impl Default for Limites {
    fn default() -> Self {
        Self {
            bytes: MAX_BYTES,
            entradas: MAX_ENTRADAS,
        }
    }
}

/// Un fichero de import leido.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Lote {
    /// `lote:` de la cabecera, si la hay.
    pub nombre: Option<String>,
    /// `sprint:` de la cabecera, si lo hay.
    pub sprint: Option<u32>,
    /// Entradas en el orden del fichero.
    pub entradas: Vec<EntradaLote>,
}

/// Una entrada del fichero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntradaLote {
    /// Linea (desde 1) de su `###`.
    pub linea: usize,
    /// Clave dentro del lote: el primer token del titulo si parece una
    /// (`F3-2`, `T1`, `A.4`: letras, cifras, `.`, `_`, `-` y al menos una
    /// cifra) o, si no, el slug del titulo.
    pub clave: String,
    /// Titulo completo, como en la 0.16 (la clave incluida).
    pub titulo: String,
    /// Objetivo (lineas `>`).
    pub objetivo: String,
    /// Criterios.
    pub criterios: Vec<String>,
    /// Dependencias tal como se escribieron: claves de este lote o
    /// `TASK-NNN`.
    pub dependencias: Vec<String>,
}

impl EntradaLote {
    /// La [`Entrada`] del alta, sin dependencias (se resuelven contra el
    /// estado).
    pub fn entrada(&self, sprint: u32) -> Entrada {
        let mut e = Entrada::nueva(&self.titulo);
        e.sprint = sprint;
        e.objetivo.clone_from(&self.objetivo);
        e.criterios.clone_from(&self.criterios);
        e
    }
}

/// Por que no se acepta un fichero: `uso` con este `motivo=`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorLote {
    /// Valor de `motivo=`.
    pub motivo: &'static str,
    /// Claves de mas para la linea (`linea=`, `causa=`...).
    pub claves: Vec<(&'static str, String)>,
    /// Diagnostico.
    pub mensaje: Mensaje,
}

impl ErrorLote {
    fn nuevo(motivo: &'static str, mensaje: Mensaje) -> Self {
        Self {
            motivo,
            claves: Vec::new(),
            mensaje,
        }
    }

    fn en(motivo: &'static str, linea: usize, mensaje: Mensaje) -> Self {
        Self::nuevo(motivo, mensaje.con("linea", linea.to_string())).con("linea", linea.to_string())
    }

    fn con(mut self, clave: &'static str, valor: String) -> Self {
        self.claves.push((clave, valor));
        self
    }
}

/// Nombre de lote valido: ASCII alfanumerico, `.`, `_` y `-`, hasta 64.
pub fn nombre_valido(n: &str) -> bool {
    !n.is_empty()
        && n.len() <= MAX_NOMBRE
        && n.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

/// Si `token` sirve de clave explicita: [`nombre_valido`] con alguna cifra
/// y sin ser un `TASK-NNN` (que en `Dependencias:` es una tarea existente).
fn parece_clave(token: &str) -> bool {
    nombre_valido(token) && token.bytes().any(|b| b.is_ascii_digit()) && !id_valido(token)
}

/// Lee un fichero de import.
pub fn leer(texto: &str, limites: Limites) -> Result<Lote, ErrorLote> {
    if texto.len() > limites.bytes {
        return Err(grande(limites));
    }
    let texto = texto.strip_prefix('\u{feff}').unwrap_or(texto);
    let lineas: Vec<&str> = texto
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .collect();
    let (nombre, sprint, desde) = cabecera(&lineas)?;
    let entradas = entradas(&lineas, desde, limites)?;
    if entradas.is_empty() {
        return Err(ErrorLote::nuevo(
            "sin-entradas",
            Mensaje::nuevo(&txt::SIN_ENTRADAS),
        ));
    }
    referencias(&entradas)?;
    Ok(Lote {
        nombre,
        sprint,
        entradas,
    })
}

/// El error de fichero demasiado grande (tambien lo da el lector de disco
/// antes de leerlo).
pub fn grande(limites: Limites) -> ErrorLote {
    ErrorLote::nuevo(
        "fichero-grande",
        Mensaje::nuevo(&txt::FICHERO_GRANDE).con("limite", limites.bytes.to_string()),
    )
    .con("limite", limites.bytes.to_string())
}

/// Cabecera `---` opcional: (lote, sprint, primera linea tras ella).
fn cabecera(lineas: &[&str]) -> Result<(Option<String>, Option<u32>, usize), ErrorLote> {
    if lineas.first() != Some(&"---") {
        return Ok((None, None, 0));
    }
    let Some(fin) = lineas
        .iter()
        .skip(1)
        .position(|l| *l == "---")
        .map(|p| p + 1)
    else {
        return Err(ErrorLote::en(
            "cabecera-sin-cerrar",
            1,
            Mensaje::nuevo(&txt::CABECERA_SIN_CERRAR),
        ));
    };
    let mut nombre = None;
    let mut sprint = None;
    for (i, l) in lineas.iter().enumerate().take(fin).skip(1) {
        let l = l.trim();
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        let malo = || {
            ErrorLote::en(
                "cabecera-invalida",
                i + 1,
                Mensaje::nuevo(&txt::CABECERA_INVALIDA),
            )
        };
        let (k, v) = l.split_once(':').ok_or_else(malo)?;
        let v = sin_comillas(v.trim());
        match k.trim() {
            "lote" if nombre.is_none() => {
                if !nombre_valido(v) {
                    return Err(ErrorLote::nuevo(
                        "lote-invalido",
                        Mensaje::nuevo(&txt::LOTE_INVALIDO).con("lote", v),
                    ));
                }
                nombre = Some(v.to_owned());
            }
            "sprint" if sprint.is_none() => {
                sprint = Some(crate::alta::sprint_de(v).ok_or_else(malo)?);
            }
            _ => return Err(malo()),
        }
    }
    Ok((nombre, sprint, fin + 1))
}

/// `"x"` o `'x'` → `x`.
fn sin_comillas(v: &str) -> &str {
    for q in ['"', '\''] {
        if let Some(s) = v.strip_prefix(q).and_then(|s| s.strip_suffix(q)) {
            return s;
        }
    }
    v
}

/// Entrada a medio leer.
struct EnCurso {
    linea: usize,
    titulo: String,
    objetivo: Vec<String>,
    criterios: Vec<String>,
    dependencias: Vec<String>,
}

/// Las entradas desde la linea `desde` (indice), con las reglas de la 0.16.
fn entradas(
    lineas: &[&str],
    desde: usize,
    limites: Limites,
) -> Result<Vec<EntradaLote>, ErrorLote> {
    let mut v: Vec<EntradaLote> = Vec::new();
    let mut claves: BTreeSet<String> = BTreeSet::new();
    let mut actual: Option<EnCurso> = None;
    let mut pegada = false;
    let mut cerrar = |e: Option<EnCurso>, v: &mut Vec<EntradaLote>| -> Result<(), ErrorLote> {
        let Some(e) = e else { return Ok(()) };
        let lote = cerrada(e)?;
        if !claves.insert(lote.clave.clone()) {
            return Err(ErrorLote::en(
                "clave-duplicada",
                lote.linea,
                Mensaje::nuevo(&txt::CLAVE_DUPLICADA).con("clave", lote.clave.as_str()),
            )
            .con("clave", lote.clave.clone()));
        }
        if v.len() >= limites.entradas {
            return Err(ErrorLote::nuevo(
                "demasiadas-entradas",
                Mensaje::nuevo(&txt::DEMASIADAS_ENTRADAS)
                    .con("limite", limites.entradas.to_string()),
            )
            .con("limite", limites.entradas.to_string()));
        }
        v.push(lote);
        Ok(())
    };
    for (i, linea) in lineas.iter().enumerate().skip(desde) {
        let n = i + 1;
        if let Some(t) = linea
            .strip_prefix("###")
            .filter(|t| t.is_empty() || t.starts_with([' ', '\t']))
        {
            cerrar(actual.take(), &mut v)?;
            actual = Some(EnCurso {
                linea: n,
                titulo: t.trim().to_owned(),
                objetivo: Vec::new(),
                criterios: Vec::new(),
                dependencias: Vec::new(),
            });
            pegada = false;
            continue;
        }
        let Some(e) = actual.as_mut() else {
            continue; // preambulo: se ignora, como en la 0.16
        };
        if es_encabezado(linea) {
            cerrar(actual.take(), &mut v)?;
            continue;
        }
        if linea.trim().is_empty() {
            pegada = false;
            continue;
        }
        if e.criterios.is_empty()
            && let Some(o) = linea.strip_prefix('>')
        {
            e.objetivo.push(o.strip_prefix(' ').unwrap_or(o).to_owned());
            continue;
        }
        if let Some(d) = dependencias(linea) {
            e.dependencias.extend(d);
            pegada = false;
            continue;
        }
        if let Some(c) = criterio(linea) {
            e.criterios.push(c.trim().to_owned());
            pegada = true;
            continue;
        }
        if pegada
            && linea.starts_with([' ', '\t'])
            && let Some(ultimo) = e.criterios.last_mut()
        {
            ultimo.push(' ');
            ultimo.push_str(linea.trim());
            continue;
        }
        return Err(ErrorLote::en(
            "linea-suelta",
            n,
            Mensaje::nuevo(&txt::LINEA_SUELTA),
        ));
    }
    cerrar(actual.take(), &mut v)?;
    Ok(v)
}

/// `#`..`######` seguido de espacio (otro nivel que `###` cierra la
/// entrada).
fn es_encabezado(l: &str) -> bool {
    let n = l.bytes().take_while(|b| *b == b'#').count();
    (1..=6).contains(&n) && l[n..].starts_with([' ', '\t'])
}

/// `- x` o `* x`, con sangria.
fn criterio(l: &str) -> Option<&str> {
    let t = l.trim_start();
    let resto = t.strip_prefix(['-', '*'])?;
    resto
        .starts_with([' ', '\t'])
        .then(|| resto.trim())
        .filter(|r| !r.is_empty())
}

/// `Dependencias: a, b` (sin distinguir mayusculas en la palabra).
fn dependencias(l: &str) -> Option<Vec<String>> {
    const PALABRA: &str = "dependencias:";
    let cabeza = l.get(..PALABRA.len())?;
    if !cabeza.eq_ignore_ascii_case(PALABRA) {
        return None;
    }
    Some(
        l[PALABRA.len()..]
            .split(',')
            .map(str::trim)
            .filter(|d| !d.is_empty())
            .map(str::to_owned)
            .collect(),
    )
}

/// Comprueba una entrada terminada y le pone la clave.
fn cerrada(e: EnCurso) -> Result<EntradaLote, ErrorLote> {
    if !titulo_valido(&e.titulo) {
        return Err(ErrorLote::en(
            "titulo-invalido",
            e.linea,
            Mensaje::nuevo(&txt::TITULO_INVALIDO),
        ));
    }
    let primero = e.titulo.split_whitespace().next().unwrap_or_default();
    let clave = if parece_clave(primero) {
        primero.to_owned()
    } else {
        crate::tarea::texto::slug(&e.titulo)
    };
    let mut dependencias = Vec::new();
    for d in e.dependencias {
        if !dependencias.contains(&d) {
            dependencias.push(d);
        }
    }
    let lote = EntradaLote {
        linea: e.linea,
        clave,
        titulo: e.titulo,
        objetivo: e.objetivo.join("\n").trim().to_owned(),
        criterios: e.criterios,
        dependencias,
    };
    lote.entrada(0).listo(true).map_err(|c: NoListo| {
        ErrorLote::en(
            "enunciado-no-listo",
            lote.linea,
            Mensaje::nuevo(&txt::NO_LISTO_EN_LINEA)
                .con("titulo", lote.titulo.as_str())
                .con_mensaje("causa", c.mensaje()),
        )
        .con("causa", c.causa().to_owned())
    })?;
    Ok(lote)
}

/// Cada dependencia es una clave del lote o un `TASK-NNN`, y las locales
/// no forman ciclos.
fn referencias(entradas: &[EntradaLote]) -> Result<(), ErrorLote> {
    let indice: BTreeMap<&str, usize> = entradas
        .iter()
        .enumerate()
        .map(|(i, e)| (e.clave.as_str(), i))
        .collect();
    let mut aristas: Vec<Vec<usize>> = Vec::with_capacity(entradas.len());
    for e in entradas {
        let mut a = Vec::new();
        for d in &e.dependencias {
            match indice.get(d.as_str()) {
                Some(j) => a.push(*j),
                None if id_valido(d) => {}
                None => {
                    return Err(ErrorLote::en(
                        crate::alta::REFERENCIA_INEXISTENTE,
                        e.linea,
                        Mensaje::nuevo(&txt::REFERENCIA_INEXISTENTE).con("referencia", d.as_str()),
                    )
                    .con("referencia", d.clone()));
                }
            }
        }
        aristas.push(a);
    }
    if let Some(ciclo) = ciclo(&aristas) {
        let nombres: Vec<&str> = ciclo
            .iter()
            .filter_map(|i| entradas.get(*i).map(|e| e.clave.as_str()))
            .collect();
        return Err(ErrorLote::nuevo(
            "ciclo",
            Mensaje::nuevo(&txt::CICLO).con("ciclo", nombres.join(" -> ")),
        )
        .con("ciclo", nombres.join(",")));
    }
    Ok(())
}

/// Un ciclo del grafo (`aristas[i]`: de quien depende `i`), cerrado (el
/// primero se repite al final), o `None`. Iterativo: 200 entradas en
/// cadena no gastan pila.
fn ciclo(aristas: &[Vec<usize>]) -> Option<Vec<usize>> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Color {
        Blanco,
        Gris,
        Negro,
    }
    let mut color = vec![Color::Blanco; aristas.len()];
    for raiz in 0..aristas.len() {
        if color[raiz] != Color::Blanco {
            continue;
        }
        // Pila de (nodo, siguiente arista por mirar).
        let mut pila: Vec<(usize, usize)> = vec![(raiz, 0)];
        color[raiz] = Color::Gris;
        while let Some((nodo, k)) = pila.last_mut() {
            let nodo = *nodo;
            let Some(&sig) = aristas.get(nodo).and_then(|a| a.get(*k)) else {
                color[nodo] = Color::Negro;
                pila.pop();
                continue;
            };
            *k += 1;
            match color.get(sig) {
                Some(Color::Blanco) => {
                    color[sig] = Color::Gris;
                    pila.push((sig, 0));
                }
                Some(Color::Gris) => {
                    let desde = pila.iter().position(|(n, _)| *n == sig).unwrap_or(0);
                    let mut c: Vec<usize> = pila[desde..].iter().map(|(n, _)| *n).collect();
                    c.push(sig);
                    return Some(c);
                }
                _ => {}
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(t: &str) -> Lote {
        let r = leer(t, Limites::default());
        assert!(r.is_ok(), "{r:?}");
        r.unwrap_or_default()
    }

    fn motivo(t: &str) -> &'static str {
        leer(t, Limites::default()).map_or_else(|e| e.motivo, |_| "ok")
    }

    #[test]
    fn formato_016_sin_cabecera() {
        let l = ok(
            "# Plan\n\nprosa ignorada\n\n### F1-19 ADR memoria\n> Objetivo uno\n>\n> dos\n- Criterio a\n  sigue a\n* Criterio b\n\n### Sin clave aqui\n- c\n## Otra cosa\ntexto suelto tras cerrar\n",
        );
        assert_eq!((l.nombre, l.sprint), (None, None));
        let e = &l.entradas;
        assert_eq!(e.len(), 2);
        assert_eq!((e[0].linea, e[0].clave.as_str()), (5, "F1-19"));
        assert_eq!(e[0].titulo, "F1-19 ADR memoria");
        assert_eq!(e[0].objetivo, "Objetivo uno\n\ndos");
        assert_eq!(e[0].criterios, ["Criterio a sigue a", "Criterio b"]);
        assert_eq!(e[1].clave, "sin-clave-aqui");
        assert_eq!(e[1].objetivo, "");
    }

    #[test]
    fn cabecera_y_dependencias() {
        let l = ok(
            "\u{feff}---\r\nlote: fase-3\r\nsprint: \"3\"\r\n---\r\n### A1 Uno\r\n- c\r\n### A2 Dos\r\nDependencias: A1, TASK-019, A1\r\n- c\r\n",
        );
        assert_eq!((l.nombre.as_deref(), l.sprint), (Some("fase-3"), Some(3)));
        assert_eq!(l.entradas[1].dependencias, ["A1", "TASK-019"]);
        assert_eq!(l.entradas[1].linea, 7);
        for (t, m) in [
            ("---\nlote: x\n", "cabecera-sin-cerrar"),
            ("---\nhito: 3\n---\n### A1 x\n- c\n", "cabecera-invalida"),
            ("---\nsprint: -1\n---\n### A1 x\n- c\n", "cabecera-invalida"),
            (
                "---\nsprint: 1\nsprint: 1\n---\n### A1 x\n- c\n",
                "cabecera-invalida",
            ),
            (
                "---\nlote: con espacio\n---\n### A1 x\n- c\n",
                "lote-invalido",
            ),
            ("---\nlote: a/b\n---\n### A1 x\n- c\n", "lote-invalido"),
        ] {
            assert_eq!(motivo(t), m, "{t}");
        }
    }

    #[test]
    fn entradas_malas_rechazan_el_lote_entero() {
        for (t, m) in [
            ("", "sin-entradas"),
            ("solo prosa\n", "sin-entradas"),
            ("### A1 x\n", "enunciado-no-listo"),
            ("### A1 x\n> solo objetivo\n", "enunciado-no-listo"),
            ("### A1 x\n- <criterio comprobable>\n", "enunciado-no-listo"),
            ("### A1 x\n- a\n- A\n", "enunciado-no-listo"),
            ("### \n- a\n", "titulo-invalido"),
            ("###\n- a\n", "titulo-invalido"),
            ("### A1 x\nprosa\n- a\n", "linea-suelta"),
            ("### A1 x\n- a\n> objetivo tarde\n", "linea-suelta"),
            ("### A1 x\n- a\n\n  sangrada tras blanco\n", "linea-suelta"),
            ("### A1 x\n- a\n### A1 y\n- b\n", "clave-duplicada"),
            (
                "### A1 x\n- a\nDependencias: B9\n",
                "referencia-inexistente",
            ),
            ("### A1 x\n- a\nDependencias: A1\n", "ciclo"),
            (
                "### A1 x\n- a\nDependencias: A3\n### A2 y\n- b\nDependencias: A1\n### A3 z\n- c\nDependencias: A2\n",
                "ciclo",
            ),
        ] {
            assert_eq!(motivo(t), m, "{t:?}");
        }
        // La linea del error es la del `###` de la entrada mala.
        let mut t = String::new();
        for i in 1..=40 {
            t.push_str(&format!("### T{i} tarea {i}\n- criterio {i}\n"));
        }
        let malo = t.replacen("### T30 tarea 30\n- criterio 30\n", "### T30 tarea 30\n", 1);
        let e = leer(&malo, Limites::default()).err();
        assert_eq!(
            e.map(|e| (e.motivo, e.claves)),
            Some((
                "enunciado-no-listo",
                vec![
                    ("linea", "59".to_owned()),
                    ("causa", "sin-criterios".to_owned())
                ]
            ))
        );
        let e = leer(&t.replace("criterio 3\n", "x\nprosa\n"), Limites::default()).err();
        assert_eq!(e.map(|e| e.motivo), Some("linea-suelta"));
    }

    #[test]
    fn limites_de_bytes_y_de_entradas() {
        let l = Limites {
            bytes: 64,
            entradas: 2,
        };
        assert_eq!(
            leer(&"x".repeat(65), l).err().map(|e| e.motivo),
            Some("fichero-grande")
        );
        let tres = "### A1 a\n- c\n### A2 b\n- c\n### A3 c\n- c\n";
        assert_eq!(tres.len(), 39);
        assert_eq!(
            leer(tres, l).err().map(|e| e.motivo),
            Some("demasiadas-entradas")
        );
        assert_eq!(leer(&tres[..26], l).map(|x| x.entradas.len()), Ok(2));
    }

    #[test]
    fn claves_y_ciclos() {
        assert!(parece_clave("F3-2") && parece_clave("T1") && parece_clave("A.4"));
        assert!(!parece_clave("Arreglar") && !parece_clave("TASK-019") && !parece_clave("F3/2"));
        assert_eq!(ciclo(&[vec![1], vec![2], vec![]]), None);
        assert_eq!(ciclo(&[vec![1], vec![0]]), Some(vec![0, 1, 0]));
        assert_eq!(ciclo(&[vec![], vec![2], vec![1]]), Some(vec![1, 2, 1]));
        // Una cadena larga no desborda la pila.
        let cadena: Vec<Vec<usize>> = (0..10_000)
            .map(|i| if i == 0 { vec![] } else { vec![i - 1] })
            .collect();
        assert_eq!(ciclo(&cadena), None);
    }
}
