//! Busqueda de errores con evidencia (F6-4; RR-03; ADR F1-8 decision 3 y
//! adenda F6-4).
//!
//! [`validar`] es una funcion pura: recibe la respuesta de una revisora (el
//! texto que devolvio el modelo), el diff que vio esa revisora ([`Diff`]) y,
//! si lo hay, el informe de verificacion de F5-4, y devuelve los hallazgos
//! aceptados y los rechazados con su motivo ([`Validados`]). No lee ficheros,
//! no llama a la red ni a un modelo: el modelo no decide, lo que devuelve es
//! un dato que se valida.
//!
//! Contrato (`crates/tc2/esquemas/hallazgos.schema.json`, version 2): la
//! respuesta trae exactamente un bloque `json` cercado con acentos graves con
//! una lista de hallazgos (`$defs/respuesta` del esquema); cada hallazgo
//! apunta a una linea del lado nuevo del diff, lleva su clase de la lista
//! cerrada de F1-15 ([`Clase`], F6-5) y trae evidencia de su tipo
//! (`reproduccion`, `mutante` o `borde`), tambien los MENOR. La evidencia se
//! declara y se ancla al diff, pero no se ejecuta: eso es de F6-6 (adenda
//! F6-4 de F1-8). [`instrucciones`] es el texto que va al prompt, hecho con
//! las mismas constantes que el validador para que no se separen.
//!
//! F6-5: [`veredicto`] junta los [`Validados`] de todas las revisoras y saca
//! la conclusion del check `revision-ia` (`failure` si queda abierto un
//! CRITICO o IMPORTANTE aceptado o si alguna revisora no dejo respuesta
//! valida; `success` si no), su resumen ([`resumen_veredicto`]) y las
//! anotaciones por linea ([`anotaciones`]). Tambien es una funcion pura.

pub mod orden;

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::verificacion::informe::{Informe as InformeVerificacion, Resultado as ResultadoPaso};

/// Version del contrato de hallazgos.
pub const VERSION: u32 = 2;

/// Hallazgos que se validan como mucho por respuesta; los que sobran se
/// rechazan juntos con [`motivo::FUERA_DE_TOPE`].
pub const TOPE_HALLAZGOS: usize = 100;

/// Ids estables de los motivos de rechazo.
pub mod motivo {
    /// La respuesta no trae ningun bloque `json`.
    pub const SIN_BLOQUE: &str = "sin-bloque";
    /// Trae mas de un bloque `json`.
    pub const BLOQUE_REPETIDO: &str = "bloque-repetido";
    /// El bloque no se cierra, no es JSON o no es una lista.
    pub const JSON_INVALIDO: &str = "json-invalido";
    /// La lista pasa de [`super::TOPE_HALLAZGOS`].
    pub const FUERA_DE_TOPE: &str = "fuera-de-tope";
    /// El hallazgo no tiene la forma del contrato.
    pub const HALLAZGO_INVALIDO: &str = "hallazgo-invalido";
    /// Otro hallazgo anterior usa el mismo id.
    pub const ID_REPETIDO: &str = "id-repetido";
    /// Sin evidencia.
    pub const SIN_EVIDENCIA: &str = "sin-evidencia";
    /// La evidencia no tiene los campos de su tipo.
    pub const EVIDENCIA_INCOMPLETA: &str = "evidencia-incompleta";
    /// El fichero o la linea no estan en el diff.
    pub const LINEA_FUERA_DEL_DIFF: &str = "linea-fuera-del-diff";
    /// `linea_original` no es el texto de la linea citada.
    pub const MUTANTE_NO_COINCIDE: &str = "mutante-no-coincide";
    /// La mutacion no cambia la linea.
    pub const MUTANTE_SIN_CAMBIO: &str = "mutante-sin-cambio";
    /// `esperado` y `obtenido` son iguales.
    pub const BORDE_SIN_DIFERENCIA: &str = "borde-sin-diferencia";
    /// `paso_rojo` no es un paso en rojo del informe de verificacion.
    pub const PASO_ROJO_NO_ESTA: &str = "paso-rojo-no-esta";
}

/// Los motivos de rechazo con su explicacion, en el orden en que se
/// comprueban. Van tal cual al prompt ([`instrucciones`]) y al esquema.
pub const MOTIVOS: [(&str, &str); 13] = [
    (
        motivo::SIN_BLOQUE,
        "la respuesta no trae ningun bloque json",
    ),
    (
        motivo::BLOQUE_REPETIDO,
        "la respuesta trae mas de un bloque json",
    ),
    (
        motivo::JSON_INVALIDO,
        "el bloque no se cierra, no es JSON o no es una lista",
    ),
    (
        motivo::FUERA_DE_TOPE,
        "la lista pasa del tope de hallazgos: se rechazan los que sobran",
    ),
    (
        motivo::HALLAZGO_INVALIDO,
        "no es un objeto, le falta una clave, trae una desconocida o un valor fuera de forma (id, severidad, linea, texto vacio)",
    ),
    (motivo::ID_REPETIDO, "un hallazgo anterior ya usa ese id"),
    (motivo::SIN_EVIDENCIA, "no trae evidencia"),
    (
        motivo::EVIDENCIA_INCOMPLETA,
        "tipo desconocido, faltan campos de su tipo, alguno esta vacio o trae campos de otro tipo",
    ),
    (
        motivo::LINEA_FUERA_DEL_DIFF,
        "el fichero o la linea no estan en el lado nuevo del diff que vio la revisora",
    ),
    (
        motivo::MUTANTE_NO_COINCIDE,
        "linea_original no es el texto de esa linea en el diff",
    ),
    (
        motivo::MUTANTE_SIN_CAMBIO,
        "la mutacion es igual a la linea original",
    ),
    (
        motivo::BORDE_SIN_DIFERENCIA,
        "esperado y obtenido son iguales",
    ),
    (
        motivo::PASO_ROJO_NO_ESTA,
        "paso_rojo no es un paso en rojo del informe de verificacion, o no hay informe",
    ),
];

/// Severidad de un hallazgo (las de la revision por pares de TaskCode).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severidad {
    Critico,
    Importante,
    Menor,
}

impl Severidad {
    /// Como se escribe en el contrato.
    pub fn texto(self) -> &'static str {
        match self {
            Severidad::Critico => "CRITICO",
            Severidad::Importante => "IMPORTANTE",
            Severidad::Menor => "MENOR",
        }
    }
}

/// Clase de un hallazgo: la lista cerrada de F1-15 (decision 12 y D7a), con
/// precedencia en este orden si encaja en varias (F6-5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Clase {
    PuertaSeguridad,
    Contrato,
    Correccion,
    FidelidadEvidencia,
    Reproducibilidad,
}

impl Clase {
    /// Todas, en el orden de precedencia.
    pub const TODAS: [Clase; 5] = [
        Clase::PuertaSeguridad,
        Clase::Contrato,
        Clase::Correccion,
        Clase::FidelidadEvidencia,
        Clase::Reproducibilidad,
    ];

    /// Como se escribe en el contrato.
    pub fn texto(self) -> &'static str {
        match self {
            Clase::PuertaSeguridad => "puerta-seguridad",
            Clase::Contrato => "contrato",
            Clase::Correccion => "correccion",
            Clase::FidelidadEvidencia => "fidelidad-evidencia",
            Clase::Reproducibilidad => "reproducibilidad",
        }
    }

    /// Cuando se aplica, para el prompt.
    fn cuando(self) -> &'static str {
        match self {
            Clase::PuertaSeguridad => {
                "la seguridad, o una puerta que deja pasar lo que deberia parar"
            }
            Clase::Contrato => "una interfaz, formato o linea de contrato no cumple lo acordado",
            Clase::Correccion => "el codigo hace otra cosa de la que deberia",
            Clase::FidelidadEvidencia => {
                "lo que se afirma (docs, cifras, tests) no se sostiene con la evidencia"
            }
            Clase::Reproducibilidad => "el resultado depende del entorno, del orden o del azar",
        }
    }
}

/// Tipo de evidencia (RR-03: reproduccion, mutantes y bordes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TipoEvidencia {
    Reproduccion,
    Mutante,
    Borde,
}

impl TipoEvidencia {
    /// Como se escribe en el contrato.
    pub fn texto(self) -> &'static str {
        match self {
            TipoEvidencia::Reproduccion => "reproduccion",
            TipoEvidencia::Mutante => "mutante",
            TipoEvidencia::Borde => "borde",
        }
    }

    /// Los campos que admite este tipo (ademas de `tipo`).
    fn campos(self) -> &'static [&'static str] {
        match self {
            TipoEvidencia::Reproduccion => &["pasos", "fallo_esperado", "paso_rojo"],
            TipoEvidencia::Mutante => &["linea_original", "mutacion", "test"],
            TipoEvidencia::Borde => &["entrada", "esperado", "obtenido"],
        }
    }

    /// Lo que pide, para el detalle del rechazo.
    fn pide(self) -> &'static str {
        match self {
            TipoEvidencia::Reproduccion => "pasos y fallo_esperado, o paso_rojo",
            TipoEvidencia::Mutante => "linea_original, mutacion y test",
            TipoEvidencia::Borde => "entrada, esperado y obtenido",
        }
    }
}

/// Evidencia de un hallazgo: `tipo` y los campos de ese tipo, ninguno de
/// otro (lo comprueba [`validar`]; aqui solo se rechaza lo desconocido).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidencia {
    pub tipo: TipoEvidencia,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pasos: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallo_esperado: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paso_rojo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linea_original: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mutacion: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entrada: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub esperado: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub obtenido: Option<String>,
}

/// Las claves de un hallazgo sin la evidencia.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cabecera {
    id: String,
    severidad: Severidad,
    clase: Clase,
    fichero: String,
    linea: u32,
    descripcion: String,
}

/// Un hallazgo que cumple el contrato. Se lee de vuelta (F6-5) del
/// `raw_details` de su anotacion, sin claves de mas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Aceptado {
    pub id: String,
    pub severidad: Severidad,
    pub clase: Clase,
    pub fichero: String,
    pub linea: u32,
    pub descripcion: String,
    pub evidencia: Evidencia,
}

/// Un rechazo: de un hallazgo (`indice` en la lista) o de toda la respuesta
/// (`indice` nulo).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Rechazo {
    pub indice: Option<usize>,
    /// El id del hallazgo, si tiene uno con forma valida.
    pub id: Option<String>,
    /// Uno de [`MOTIVOS`]; al leerlo de vuelta, uno desconocido es error.
    pub motivo: &'static str,
    /// Que fallo, para la persona.
    pub detalle: String,
}

/// Resultado de validar la respuesta de una revisora.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Validados {
    pub version: u32,
    pub revisora: String,
    pub aceptados: Vec<Aceptado>,
    pub rechazados: Vec<Rechazo>,
}

/// Un [`Rechazo`] leido de vuelta, con el motivo aun sin contrastar.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RechazoLeido {
    indice: Option<usize>,
    id: Option<String>,
    motivo: String,
    detalle: String,
}

impl<'de> Deserialize<'de> for Rechazo {
    /// El motivo tiene que ser uno de [`MOTIVOS`].
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let r = RechazoLeido::deserialize(d)?;
        let motivo = MOTIVOS
            .iter()
            .map(|(id, _)| *id)
            .find(|id| *id == r.motivo)
            .ok_or_else(|| serde::de::Error::custom(format!("motivo desconocido: {}", r.motivo)))?;
        Ok(Self {
            indice: r.indice,
            id: r.id,
            motivo,
            detalle: r.detalle,
        })
    }
}

impl Rechazo {
    /// A quien se refiere: el id del hallazgo, `#<indice>` si no tiene uno
    /// valido, o `respuesta`.
    pub fn quien(&self) -> String {
        match (&self.id, self.indice) {
            (Some(id), _) => id.clone(),
            (None, Some(i)) => format!("#{i}"),
            (None, None) => "respuesta".to_owned(),
        }
    }
}

impl Validados {
    /// `quien:motivo` por rechazo, para la linea de contrato.
    pub fn motivos(&self) -> Vec<String> {
        self.rechazados
            .iter()
            .map(|r| format!("{}:{}", r.quien(), r.motivo))
            .collect()
    }
}

/// Las lineas del lado nuevo de un diff unificado de git: por fichero, el
/// numero de linea y su texto (las de contexto y las anadidas).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Diff {
    lineas: BTreeMap<String, BTreeMap<u32, String>>,
}

impl Diff {
    /// Lee un diff de `git diff` (con o sin prefijos `a/` y `b/`). Lo que no
    /// entiende lo salta; un diff cortado deja fuera lo que no llego.
    pub fn leer(texto: &str) -> Self {
        let mut d = Diff::default();
        let mut fichero: Option<String> = None;
        let mut nuevo: u32 = 0;
        let mut quedan_nuevo: u32 = 0;
        let mut quedan_viejo: u32 = 0;
        for l in texto.split('\n') {
            let l = l.strip_suffix('\r').unwrap_or(l);
            if quedan_nuevo > 0 || quedan_viejo > 0 {
                let mut registrar = |resto: &str| {
                    if let Some(f) = &fichero {
                        d.lineas
                            .entry(f.clone())
                            .or_default()
                            .insert(nuevo, resto.to_owned());
                    }
                };
                match l.as_bytes().first() {
                    Some(b'+') if quedan_nuevo > 0 => {
                        registrar(&l[1..]);
                        nuevo = nuevo.saturating_add(1);
                        quedan_nuevo -= 1;
                        continue;
                    }
                    Some(b'-') if quedan_viejo > 0 => {
                        quedan_viejo -= 1;
                        continue;
                    }
                    Some(b' ') if quedan_nuevo > 0 => {
                        registrar(&l[1..]);
                        nuevo = nuevo.saturating_add(1);
                        quedan_nuevo -= 1;
                        quedan_viejo = quedan_viejo.saturating_sub(1);
                        continue;
                    }
                    Some(b'\\') => continue,
                    // Fin inesperado del trozo (diff mal formado): se sigue
                    // buscando cabeceras.
                    _ => {
                        quedan_nuevo = 0;
                        quedan_viejo = 0;
                    }
                }
            }
            if l.starts_with("diff --git ") {
                fichero = None;
            } else if let Some(r) = l.strip_prefix("+++ ") {
                fichero = ruta_nueva(r);
            } else if let Some(r) = l.strip_prefix("@@ ")
                && let Some((viejo, (inicio, largo))) = cabecera_de_trozo(r)
            {
                nuevo = inicio;
                quedan_nuevo = largo;
                quedan_viejo = viejo;
            }
        }
        d
    }

    /// El texto de la linea `n` del lado nuevo de `fichero`, sin el `+` ni
    /// el espacio inicial.
    pub fn linea(&self, fichero: &str, n: u32) -> Option<&str> {
        self.lineas
            .get(fichero)
            .and_then(|m| m.get(&n))
            .map(String::as_str)
    }
}

/// La ruta de una linea `+++ `: sin el tabulador final que git pone a las
/// rutas con espacios, sin comillas (decodificando los escapes de git) y sin
/// `b/`; `None` si es `/dev/null`.
fn ruta_nueva(r: &str) -> Option<String> {
    let r = r.trim_end_matches('\t');
    if r == "/dev/null" {
        return None;
    }
    let r = match r.strip_prefix('"').and_then(|x| x.strip_suffix('"')) {
        Some(x) => sin_escapes_de_git(x),
        None => r.to_owned(),
    };
    Some(r.strip_prefix("b/").unwrap_or(&r).to_owned())
}

/// El interior de una ruta entrecomillada por git (`core.quotePath`): los
/// escapes C (`\"`, `\\`, `\t`, `\n`...) y los octales `\ooo`, que son bytes
/// de UTF-8 (`\303\261` es `ñ`). Un escape que no entiende se deja tal cual.
fn sin_escapes_de_git(x: &str) -> String {
    let b = x.as_bytes();
    let mut s: Vec<u8> = Vec::with_capacity(b.len());
    let mut i = 0;
    while let Some(&c) = b.get(i) {
        i += 1;
        if c != b'\\' {
            s.push(c);
            continue;
        }
        let octal = b
            .get(i..i + 3)
            .filter(|d| d.iter().all(|x| (b'0'..=b'7').contains(x)))
            .map(|d| d.iter().fold(0u32, |n, x| n * 8 + u32::from(x - b'0')))
            .and_then(|n| u8::try_from(n).ok());
        if let Some(n) = octal {
            s.push(n);
            i += 3;
            continue;
        }
        let simple = match b.get(i) {
            Some(b'"') => Some(b'"'),
            Some(b'\\') => Some(b'\\'),
            Some(b't') => Some(b'\t'),
            Some(b'n') => Some(b'\n'),
            Some(b'r') => Some(b'\r'),
            Some(b'a') => Some(0x07),
            Some(b'b') => Some(0x08),
            Some(b'f') => Some(0x0C),
            Some(b'v') => Some(0x0B),
            _ => None,
        };
        match simple {
            Some(e) => {
                s.push(e);
                i += 1;
            }
            None => s.push(b'\\'),
        }
    }
    String::from_utf8_lossy(&s).into_owned()
}

/// `-a,b +c,d @@ ...` da `(b, (c, d))`; sin coma, el largo es 1.
fn cabecera_de_trozo(r: &str) -> Option<(u32, (u32, u32))> {
    let mut partes = r.split_whitespace();
    let viejo = rango(partes.next()?.strip_prefix('-')?)?;
    let nuevo = rango(partes.next()?.strip_prefix('+')?)?;
    Some((viejo.1, nuevo))
}

fn rango(t: &str) -> Option<(u32, u32)> {
    match t.split_once(',') {
        Some((a, b)) => Some((a.parse().ok()?, b.parse().ok()?)),
        None => Some((t.parse().ok()?, 1)),
    }
}

/// El contenido del unico bloque `json` de la respuesta, o el motivo y el
/// detalle del rechazo de toda la respuesta.
fn bloque_json(respuesta: &str) -> Result<String, (&'static str, String)> {
    let mut bloques: Vec<String> = Vec::new();
    let mut abierto: Option<(usize, String)> = None;
    for l in respuesta.split('\n') {
        let l = l.strip_suffix('\r').unwrap_or(l);
        let t = l.trim();
        let graves = t.chars().take_while(|c| *c == '`').count();
        match &mut abierto {
            Some((n, contenido)) => {
                if graves >= *n && graves == t.len() {
                    bloques.push(std::mem::take(contenido));
                    abierto = None;
                } else {
                    contenido.push_str(l);
                    contenido.push('\n');
                }
            }
            None => {
                if graves >= 3 && t[graves..].trim().eq_ignore_ascii_case("json") {
                    abierto = Some((graves, String::new()));
                }
            }
        }
    }
    if abierto.is_some() {
        return Err((
            motivo::JSON_INVALIDO,
            "el bloque json no se cierra".to_owned(),
        ));
    }
    match bloques.len() {
        0 => Err((
            motivo::SIN_BLOQUE,
            "la respuesta no trae un bloque json con la lista de hallazgos".to_owned(),
        )),
        1 => Ok(bloques.remove(0)),
        n => Err((
            motivo::BLOQUE_REPETIDO,
            format!("la respuesta trae {n} bloques json; tiene que traer uno"),
        )),
    }
}

/// Un id: letra o digito y luego letras, digitos, `.`, `_` o `-`; hasta 64.
fn es_id(t: &str) -> bool {
    let mut c = t.chars();
    t.len() <= 64
        && c.next().is_some_and(|x| x.is_ascii_alphanumeric())
        && c.all(|x| x.is_ascii_alphanumeric() || matches!(x, '.' | '_' | '-'))
}

fn no_vacio(t: &str) -> bool {
    !t.trim().is_empty()
}

struct Fallo {
    id: Option<String>,
    motivo: &'static str,
    detalle: String,
}

/// Que la evidencia traiga los campos de su tipo, llenos, y ninguno ajeno.
fn evidencia_completa(e: &Evidencia) -> Result<(), String> {
    let texto = |o: &Option<String>| o.as_deref().map(no_vacio);
    let campos: [(&str, Option<bool>); 9] = [
        (
            "pasos",
            e.pasos
                .as_ref()
                .map(|p| !p.is_empty() && p.iter().all(|s| no_vacio(s))),
        ),
        ("fallo_esperado", texto(&e.fallo_esperado)),
        ("paso_rojo", texto(&e.paso_rojo)),
        ("linea_original", texto(&e.linea_original)),
        ("mutacion", texto(&e.mutacion)),
        ("test", texto(&e.test)),
        ("entrada", texto(&e.entrada)),
        ("esperado", texto(&e.esperado)),
        ("obtenido", texto(&e.obtenido)),
    ];
    let propios = e.tipo.campos();
    for (nombre, estado) in campos {
        match estado {
            Some(_) if !propios.contains(&nombre) => {
                return Err(format!(
                    "{nombre} no es de una evidencia {}",
                    e.tipo.texto()
                ));
            }
            Some(false) => return Err(format!("{nombre} esta vacio")),
            _ => {}
        }
    }
    let tiene = |n: &str| campos.iter().any(|(m, x)| *m == n && *x == Some(true));
    let completa = match e.tipo {
        TipoEvidencia::Reproduccion => {
            (tiene("pasos") && tiene("fallo_esperado")) || tiene("paso_rojo")
        }
        _ => propios.iter().all(|n| tiene(n)),
    };
    if completa {
        Ok(())
    } else {
        Err(format!(
            "una evidencia {} pide {}",
            e.tipo.texto(),
            e.tipo.pide()
        ))
    }
}

fn validar_uno(
    h: &Value,
    diff: &Diff,
    informe: Option<&InformeVerificacion>,
    vistos: &mut BTreeSet<String>,
) -> Result<Aceptado, Fallo> {
    let Some(obj) = h.as_object() else {
        return Err(Fallo {
            id: None,
            motivo: motivo::HALLAZGO_INVALIDO,
            detalle: "el hallazgo no es un objeto".to_owned(),
        });
    };
    let id = obj
        .get("id")
        .and_then(Value::as_str)
        .filter(|s| es_id(s))
        .map(str::to_owned);
    let fallo = |motivo: &'static str, detalle: String| Fallo {
        id: id.clone(),
        motivo,
        detalle,
    };
    let evidencia = match obj.get("evidencia") {
        None | Some(Value::Null) => {
            return Err(fallo(
                motivo::SIN_EVIDENCIA,
                "el hallazgo no trae evidencia".to_owned(),
            ));
        }
        Some(e) => e,
    };
    let mut resto = obj.clone();
    resto.remove("evidencia");
    let c: Cabecera = serde_json::from_value(Value::Object(resto))
        .map_err(|e| fallo(motivo::HALLAZGO_INVALIDO, e.to_string()))?;
    if !es_id(&c.id) {
        return Err(fallo(
            motivo::HALLAZGO_INVALIDO,
            "id no valido: letras, digitos, punto, guion y guion bajo, hasta 64".to_owned(),
        ));
    }
    if c.linea == 0 || !no_vacio(&c.fichero) || !no_vacio(&c.descripcion) {
        return Err(fallo(
            motivo::HALLAZGO_INVALIDO,
            "linea desde 1, y fichero y descripcion no vacios".to_owned(),
        ));
    }
    if !vistos.insert(c.id.clone()) {
        return Err(fallo(
            motivo::ID_REPETIDO,
            format!("el id {} ya lo usa un hallazgo anterior", c.id),
        ));
    }
    let e: Evidencia = serde_json::from_value(evidencia.clone())
        .map_err(|x| fallo(motivo::EVIDENCIA_INCOMPLETA, x.to_string()))?;
    evidencia_completa(&e).map_err(|x| fallo(motivo::EVIDENCIA_INCOMPLETA, x))?;
    let Some(texto) = diff.linea(&c.fichero, c.linea) else {
        return Err(fallo(
            motivo::LINEA_FUERA_DEL_DIFF,
            format!(
                "{}:{} no esta en el lado nuevo del diff que vio la revisora",
                c.fichero, c.linea
            ),
        ));
    };
    if let Some(original) = &e.linea_original {
        if original.trim() != texto.trim() {
            return Err(fallo(
                motivo::MUTANTE_NO_COINCIDE,
                format!(
                    "linea_original no es el texto de {}:{} en el diff",
                    c.fichero, c.linea
                ),
            ));
        }
        if e.mutacion.as_deref().map(str::trim) == Some(original.trim()) {
            return Err(fallo(
                motivo::MUTANTE_SIN_CAMBIO,
                "la mutacion es igual a la linea original".to_owned(),
            ));
        }
    }
    if let (Some(esperado), Some(obtenido)) = (&e.esperado, &e.obtenido)
        && esperado.trim() == obtenido.trim()
    {
        return Err(fallo(
            motivo::BORDE_SIN_DIFERENCIA,
            "esperado y obtenido son iguales: no hay borde".to_owned(),
        ));
    }
    if let Some(paso) = &e.paso_rojo {
        let en_rojo = informe.is_some_and(|i| {
            i.pasos
                .iter()
                .any(|p| p.nombre == paso.trim() && p.resultado == ResultadoPaso::Rojo)
        });
        if !en_rojo {
            return Err(fallo(
                motivo::PASO_ROJO_NO_ESTA,
                format!(
                    "el paso {} no figura en rojo en el informe de verificacion",
                    paso.trim()
                ),
            ));
        }
    }
    Ok(Aceptado {
        id: c.id,
        severidad: c.severidad,
        clase: c.clase,
        fichero: c.fichero,
        linea: c.linea,
        descripcion: c.descripcion,
        evidencia: e,
    })
}

/// Valida la respuesta de la revisora `revisora` contra el diff que vio y,
/// si lo hay, el informe de verificacion (para `paso_rojo`).
pub fn validar(
    revisora: &str,
    respuesta: &str,
    diff: &Diff,
    informe: Option<&InformeVerificacion>,
) -> Validados {
    let mut v = Validados {
        version: VERSION,
        revisora: revisora.to_owned(),
        aceptados: Vec::new(),
        rechazados: Vec::new(),
    };
    let de_toda = |motivo: &'static str, detalle: String| Rechazo {
        indice: None,
        id: None,
        motivo,
        detalle,
    };
    let lista = match bloque_json(respuesta).map(|b| serde_json::from_str::<Value>(&b)) {
        Err((m, d)) => {
            v.rechazados.push(de_toda(m, d));
            return v;
        }
        Ok(Ok(Value::Array(l))) => l,
        Ok(Ok(_)) => {
            v.rechazados.push(de_toda(
                motivo::JSON_INVALIDO,
                "el bloque json no es una lista".to_owned(),
            ));
            return v;
        }
        Ok(Err(e)) => {
            v.rechazados.push(de_toda(
                motivo::JSON_INVALIDO,
                format!("el bloque no es JSON: {e}"),
            ));
            return v;
        }
    };
    let mut vistos = BTreeSet::new();
    for (i, h) in lista.iter().enumerate() {
        if i >= TOPE_HALLAZGOS {
            v.rechazados.push(Rechazo {
                indice: Some(i),
                id: None,
                motivo: motivo::FUERA_DE_TOPE,
                detalle: format!(
                    "{} hallazgos pasan del tope de {TOPE_HALLAZGOS}",
                    lista.len() - TOPE_HALLAZGOS
                ),
            });
            break;
        }
        match validar_uno(h, diff, informe, &mut vistos) {
            Ok(a) => v.aceptados.push(a),
            Err(f) => v.rechazados.push(Rechazo {
                indice: Some(i),
                id: f.id,
                motivo: f.motivo,
                detalle: f.detalle,
            }),
        }
    }
    v
}

/// Ejemplo de respuesta valida de las instrucciones (lo validan los tests
/// contra `tests/fixtures/hallazgos/diff.txt`).
const EJEMPLO: &str = r#"[
  {
    "id": "H1",
    "severidad": "CRITICO",
    "clase": "correccion",
    "fichero": "src/calculo.rs",
    "linea": 5,
    "descripcion": "La condicion esta al reves: con dos o mas valores devuelve None.",
    "evidencia": {
      "tipo": "borde",
      "entrada": "media(&[2.0, 4.0])",
      "esperado": "Some(3.0)",
      "obtenido": "None"
    }
  },
  {
    "id": "H2",
    "severidad": "MENOR",
    "clase": "fidelidad-evidencia",
    "fichero": "src/calculo.rs",
    "linea": 12,
    "descripcion": "Ningun test fija el tope de 100.",
    "evidencia": {
      "tipo": "mutante",
      "linea_original": "x.min(100)",
      "mutacion": "x.min(101)",
      "test": "tope_corta_en_100"
    }
  }
]"#;

/// El texto de «Lo que devuelves» del prompt de cada revisora, hecho con las
/// mismas constantes que [`validar`].
pub fn instrucciones() -> String {
    let mut s = String::new();
    s.push_str(
        "Devuelve un informe en Markdown en espanol y, al final, exactamente un bloque ```json con la lista de tus hallazgos (`[]` si no hay ninguno). \
Solo cuenta lo que hay en ese bloque: `tc2 hallazgos` lo valida con reglas fijas y rechaza todo hallazgo que no las cumpla, tambien los MENOR. \
El check `revision-ia` lista los aceptados y los rechazados con su motivo.\n\n",
    );
    s.push_str("Cada hallazgo es un objeto con estas claves y ninguna mas:\n\n");
    s.push_str("- `id`: corto y unico en la lista (letras, digitos, `.`, `_` o `-`; hasta 64).\n");
    let severidades: Vec<String> = [Severidad::Critico, Severidad::Importante, Severidad::Menor]
        .iter()
        .map(|x| format!("`{}`", x.texto()))
        .collect();
    s.push_str(&format!("- `severidad`: {}.\n", severidades.join(", ")));
    s.push_str("- `clase`: una de esta lista; si encaja en varias, la primera (F1-15):\n");
    for c in Clase::TODAS {
        s.push_str(&format!("  - `{}`: {}.\n", c.texto(), c.cuando()));
    }
    s.push_str(
        "- `fichero`: la ruta tal como sale en la cabecera `+++ b/<ruta>` del diff, sin `b/`.\n",
    );
    s.push_str(
        "- `linea`: numero de linea del lado nuevo del diff. Se cuenta desde el `+c` de la cabecera `@@ -a,b +c,d @@`, sumando las lineas que empiezan por espacio o por `+` y saltando las que empiezan por `-`. Tiene que ser una linea del diff que ves.\n",
    );
    s.push_str("- `descripcion`: que falla y por que, en una o dos frases.\n");
    s.push_str(
        "- `evidencia`: obligatoria en todos. Un objeto con `tipo` y los campos de ese tipo, llenos, y ninguno de otro tipo:\n",
    );
    s.push_str(
        "  - `reproduccion`: `pasos` (lista no vacia) y `fallo_esperado`; o `paso_rojo`, el nombre de un paso en rojo del informe de verificacion (F5-4).\n",
    );
    s.push_str(
        "  - `mutante`: `linea_original` (el texto de la linea `linea` copiado tal cual del diff, sin el `+` ni el espacio inicial), `mutacion` (esa linea cambiada, distinta) y `test` (el test que deberia detectar la mutacion y hoy no lo hace).\n",
    );
    s.push_str(
        "  - `borde`: `entrada`, `esperado` (lo que deberia salir) y `obtenido` (lo que sale segun el codigo), distintos.\n\n",
    );
    s.push_str(
        "En esta ronda nadie ejecuta la evidencia: se comprueba que este completa y anclada al diff (la ejecucion es de F6-6). \
No inventes lineas que no ves: si el diff esta cortado, el hallazgo de lo que no llego se rechaza.\n\n",
    );
    s.push_str(&format!(
        "Como mucho {TOPE_HALLAZGOS} hallazgos. Motivos de rechazo:\n\n"
    ));
    for (id, explicacion) in MOTIVOS {
        s.push_str(&format!("- `{id}`: {explicacion}.\n"));
    }
    s.push_str("\nEjemplo del bloque:\n\n```json\n");
    s.push_str(EJEMPLO);
    s.push_str("\n```\n");
    s
}

/// Texto de la respuesta del modelo como texto plano de Markdown en una
/// linea: sin saltos ni controles, con la puntuacion que da formato escapada
/// y cortado a `tope` caracteres. Es dato del modelo (F6-7): no puede abrir
/// tablas, enlaces, imagenes ni HTML en el check. Tampoco autoenlaces (URL
/// desnudas, `www.`, correos), menciones `@usuario`, matematicas `$` ni
/// emojis `:codigo:`; y los caracteres Unicode de formato (categoria Cf:
/// ancho cero, cambios de direccion...) se quitan.
pub fn plano(t: &str, tope: usize) -> String {
    let cs: Vec<char> = t.chars().filter(|c| !es_de_formato(*c)).collect();
    let mut s = String::new();
    let mut n = 0;
    let mut espacio = false;
    for (i, &c) in cs.iter().enumerate() {
        if n >= tope {
            s.push('…');
            break;
        }
        if c.is_whitespace() || c.is_control() {
            if !espacio && !s.is_empty() {
                s.push(' ');
                n += 1;
            }
            espacio = true;
            continue;
        }
        espacio = false;
        if matches!(
            c,
            '\\' | '`'
                | '*'
                | '_'
                | '['
                | ']'
                | '('
                | ')'
                | '<'
                | '>'
                | '|'
                | '!'
                | '~'
                | '#'
                | '&'
                | '@'
                | '$'
        ) || (c == ':' && abre_enlace_o_emoji(cs.get(i + 1..).unwrap_or_default()))
            || (c == '.'
                && s.get(s.len().saturating_sub(3)..)
                    .is_some_and(|x| x.eq_ignore_ascii_case("www")))
        {
            s.push('\\');
        }
        s.push(c);
        n += 1;
    }
    s.trim_end().to_owned()
}

/// Lo que sigue a un `:` lo convierte en autoenlace (`://`) o en la apertura
/// de un emoji (`:codigo:`). Un `:` suelto, como el de
/// `taskcode-plugin:revisor`, se deja tal cual.
fn abre_enlace_o_emoji(resto: &[char]) -> bool {
    if resto.first() == Some(&'/') {
        return true;
    }
    let nombre = resto
        .iter()
        .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '+' | '-'))
        .count();
    nombre > 0 && resto.get(nombre) == Some(&':')
}

/// Caracteres Unicode de formato (categoria Cf): invisibles, pero cambian la
/// direccion o el corte del texto.
fn es_de_formato(c: char) -> bool {
    matches!(
        c,
        '\u{AD}'
            | '\u{600}'..='\u{605}'
            | '\u{61C}'
            | '\u{6DD}'
            | '\u{70F}'
            | '\u{890}'..='\u{891}'
            | '\u{8E2}'
            | '\u{180E}'
            | '\u{200B}'..='\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{206F}'
            | '\u{FEFF}'
            | '\u{FFF9}'..='\u{FFFB}'
            | '\u{110BD}'
            | '\u{110CD}'
            | '\u{13430}'..='\u{1343F}'
            | '\u{1BCA0}'..='\u{1BCA3}'
            | '\u{1D173}'..='\u{1D17A}'
            | '\u{E0001}'
            | '\u{E0020}'..='\u{E007F}'
    )
}

/// La seccion del check `revision-ia` para una revisora: las tablas de
/// aceptados y rechazados, con todo el texto del modelo pasado por
/// [`plano`].
pub fn resumen_markdown(v: &Validados) -> String {
    let mut s = format!(
        "### Hallazgos de {}: {} aceptados, {} rechazados\n\n",
        plano(&v.revisora, 120),
        v.aceptados.len(),
        v.rechazados.len()
    );
    if !v.aceptados.is_empty() {
        s.push_str("| id | severidad | clase | fichero | linea | evidencia | descripcion |\n|---|---|---|---|---|---|---|\n");
        for a in &v.aceptados {
            s.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} |\n",
                plano(&a.id, 64),
                a.severidad.texto(),
                a.clase.texto(),
                plano(&a.fichero, 200),
                a.linea,
                a.evidencia.tipo.texto(),
                plano(&a.descripcion, 400)
            ));
        }
        s.push('\n');
    }
    if !v.rechazados.is_empty() {
        s.push_str("| rechazado | motivo | detalle |\n|---|---|---|\n");
        for r in &v.rechazados {
            s.push_str(&format!(
                "| {} | {} | {} |\n",
                plano(&r.quien(), 64),
                r.motivo,
                plano(&r.detalle, 300)
            ));
        }
        s.push('\n');
    }
    s
}

// --- Veredicto del check (F6-5) ---------------------------------------------

/// Anotaciones como mucho por peticion a la API de check-runs (tope de
/// GitHub).
pub const ANOTACIONES_POR_LOTE: usize = 50;

/// Prefijo del titulo del check; tras el viene el veredicto.
pub const PREFIJO_TITULO: &str = "revision-ia: ";

/// Lo que dice el check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Dictamen {
    /// Queda abierto un CRITICO o IMPORTANTE aceptado.
    Rojo,
    /// Sin CRITICOS ni IMPORTANTES abiertos.
    Verde,
    /// Alguna revisora no dejo respuesta valida, o no hubo ninguna: la
    /// revision no ocurrio (falla en cerrado, decision 1 de TASK-051).
    Incompleta,
}

impl Dictamen {
    /// Como se escribe en la linea y en el titulo.
    pub fn texto(self) -> &'static str {
        match self {
            Dictamen::Rojo => "rojo",
            Dictamen::Verde => "verde",
            Dictamen::Incompleta => "incompleta",
        }
    }

    /// Conclusion del check-run: solo `success` o `failure` (nunca
    /// `neutral`, que GitHub da por cumplido).
    pub fn conclusion(self) -> &'static str {
        match self {
            Dictamen::Verde => "success",
            Dictamen::Rojo | Dictamen::Incompleta => "failure",
        }
    }

    /// El dictamen de un titulo del check (`revision-ia: <dictamen> ...`).
    pub fn de_titulo(titulo: &str) -> Option<Self> {
        let resto = titulo.strip_prefix(PREFIJO_TITULO)?;
        match resto.split(' ').next()? {
            "rojo" => Some(Dictamen::Rojo),
            "verde" => Some(Dictamen::Verde),
            "incompleta" => Some(Dictamen::Incompleta),
            _ => None,
        }
    }
}

/// Un aceptado con la revisora que lo encontro: el `raw_details` de su
/// anotacion, que `tc2 review` lee de vuelta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AceptadoDe {
    pub revisora: String,
    pub hallazgo: Aceptado,
}

/// Una revisora en el resumen del veredicto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fila {
    /// `None` si no dejo un JSON de `validar` legible.
    pub revisora: Option<String>,
    pub aceptados: usize,
    pub rechazados: usize,
    /// El motivo del rechazo de toda la respuesta, si lo hubo.
    pub incompleta: Option<&'static str>,
}

/// El veredicto de todas las revisoras.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Veredicto {
    pub dictamen: Dictamen,
    pub criticos: usize,
    pub importantes: usize,
    pub menores: usize,
    pub rechazados: usize,
    pub filas: Vec<Fila>,
    pub aceptados: Vec<AceptadoDe>,
}

impl Veredicto {
    /// Revisoras sin respuesta valida.
    pub fn incompletas(&self) -> usize {
        self.filas
            .iter()
            .filter(|f| f.revisora.is_none() || f.incompleta.is_some())
            .count()
    }

    /// Titulo del check.
    pub fn titulo(&self) -> String {
        titulo_de(self.dictamen, self.criticos, self.importantes, self.menores)
    }
}

fn titulo_de(d: Dictamen, criticos: usize, importantes: usize, menores: usize) -> String {
    format!(
        "{PREFIJO_TITULO}{} (criticos={criticos} importantes={importantes} menores={menores})",
        d.texto()
    )
}

/// El dictamen de un check `revision-ia` publicado, solo si su titulo casa
/// con su conclusion y con los aceptados leidos de sus anotaciones: el
/// titulo es el que daria [`Veredicto::titulo`] con esos recuentos, la
/// conclusion la de ese dictamen, y un `rojo` tiene algun CRITICO o
/// IMPORTANTE y un `verde` ninguno. Un `incompleta` sin anotaciones vale
/// con cualquier titulo (el de `publicar` cuando no hay veredicto de
/// `validar` o la API lo rechaza). `None` si no casan: un check con menos
/// anotaciones de las que dice su titulo (un lote que no llego) o alterado.
pub fn dictamen_publicado(
    titulo: &str,
    conclusion: &str,
    aceptados: &[AceptadoDe],
) -> Option<Dictamen> {
    let d = Dictamen::de_titulo(titulo)?;
    if conclusion != d.conclusion() {
        return None;
    }
    let n = |s: Severidad| {
        aceptados
            .iter()
            .filter(|a| a.hallazgo.severidad == s)
            .count()
    };
    let (c, i, m) = (
        n(Severidad::Critico),
        n(Severidad::Importante),
        n(Severidad::Menor),
    );
    let coherente = match d {
        Dictamen::Rojo => c + i > 0,
        Dictamen::Verde => c + i == 0,
        Dictamen::Incompleta => true,
    };
    let casa =
        titulo == titulo_de(d, c, i, m) || (d == Dictamen::Incompleta && aceptados.is_empty());
    (coherente && casa).then_some(d)
}

/// Junta los resultados de `validar` de cada revisora (`None` si no dejo
/// uno legible). En F6-5 todo aceptado esta abierto (el cierre es de F6-6).
pub fn veredicto(revisoras: &[Option<Validados>]) -> Veredicto {
    let mut v = Veredicto {
        dictamen: Dictamen::Verde,
        criticos: 0,
        importantes: 0,
        menores: 0,
        rechazados: 0,
        filas: Vec::new(),
        aceptados: Vec::new(),
    };
    for r in revisoras {
        let Some(r) = r else {
            v.filas.push(Fila {
                revisora: None,
                aceptados: 0,
                rechazados: 0,
                incompleta: None,
            });
            continue;
        };
        for a in &r.aceptados {
            match a.severidad {
                Severidad::Critico => v.criticos += 1,
                Severidad::Importante => v.importantes += 1,
                Severidad::Menor => v.menores += 1,
            }
            v.aceptados.push(AceptadoDe {
                revisora: r.revisora.clone(),
                hallazgo: a.clone(),
            });
        }
        v.rechazados += r.rechazados.len();
        v.filas.push(Fila {
            revisora: Some(r.revisora.clone()),
            aceptados: r.aceptados.len(),
            rechazados: r.rechazados.len(),
            incompleta: r
                .rechazados
                .iter()
                .find(|x| x.indice.is_none())
                .map(|x| x.motivo),
        });
    }
    v.dictamen = if v.filas.is_empty() || v.incompletas() > 0 {
        Dictamen::Incompleta
    } else if v.criticos + v.importantes > 0 {
        Dictamen::Rojo
    } else {
        Dictamen::Verde
    };
    v
}

/// Una anotacion del check-run (API de check-runs de GitHub).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Anotacion {
    pub path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub annotation_level: String,
    pub title: String,
    pub message: String,
    pub raw_details: String,
}

/// Tope de bytes de `raw_details` en la API de check-runs (64 KB).
pub const TOPE_RAW_DETAILS: usize = 64 * 1024;

/// Mensaje de una anotacion cuya descripcion se queda vacia al pasarla por
/// [`plano`] (solo controles o caracteres de formato): la API no admite un
/// `message` vacio.
pub const MENSAJE_SIN_TEXTO: &str = "descripcion sin texto legible";

/// Por debajo de estos bytes un texto no se recorta (asi cada recorte quita
/// bytes aunque anada `…`).
const RECORTE_MINIMO: usize = 16;

/// Corta `s` por la mitad de sus bytes (en limite de caracter) y anade `…`.
fn recortar_mitad(s: &mut String) {
    let mut corte = s.len() / 2;
    while !s.is_char_boundary(corte) {
        corte -= 1;
    }
    s.truncate(corte);
    s.push('…');
}

/// El `raw_details` de `a`: su JSON, con los textos libres del hallazgo
/// (descripcion y evidencia) recortados por la mitad, el mas largo cada vez
/// (o la lista de `pasos`, si pesa mas), hasta caber en
/// [`TOPE_RAW_DETAILS`]. El id, el fichero, la linea y la revisora no se
/// tocan: sigue siendo un [`AceptadoDe`] que `leer-revision` lee de vuelta.
fn raw_details(a: &AceptadoDe) -> String {
    let mut a = a.clone();
    loop {
        let json = serde_json::to_string(&a).unwrap_or_default();
        if json.len() <= TOPE_RAW_DETAILS {
            return json;
        }
        let h = &mut a.hallazgo;
        let e = &mut h.evidencia;
        // Una lista de varios pasos se recorta entera; un paso solo, como
        // texto.
        let (pasos, unico) = match e.pasos.as_mut() {
            Some(p) if p.len() > 1 => (Some(p), None),
            Some(p) => (None, p.first_mut()),
            None => (None, None),
        };
        let peso_pasos: usize = pasos
            .as_ref()
            .map_or(0, |p| p.iter().map(|s| s.len() + 3).sum());
        let mut textos: Vec<&mut String> = vec![&mut h.descripcion];
        textos.extend(unico);
        for t in [
            &mut e.fallo_esperado,
            &mut e.paso_rojo,
            &mut e.linea_original,
            &mut e.mutacion,
            &mut e.test,
            &mut e.entrada,
            &mut e.esperado,
            &mut e.obtenido,
        ] {
            textos.extend(t.as_mut());
        }
        let largo = textos
            .into_iter()
            .filter(|t| t.len() > RECORTE_MINIMO)
            .max_by_key(|t| t.len());
        match (largo, pasos) {
            (Some(t), _) if t.len() >= peso_pasos => recortar_mitad(t),
            (_, Some(p)) if peso_pasos > 0 => p.truncate(p.len() / 2),
            (Some(t), _) => recortar_mitad(t),
            // Nada mas que recortar: no cabe y la API lo rechazara
            // (`publicar` cae entonces a `incompleta`).
            (None, _) => return json,
        }
    }
}

/// Una anotacion por aceptado, en lotes de [`ANOTACIONES_POR_LOTE`]:
/// `failure` para CRITICO e IMPORTANTE, `warning` para MENOR; titulo y
/// mensaje pasados por [`plano`] (un mensaje vacio pasa a
/// [`MENSAJE_SIN_TEXTO`]); `raw_details`, el [`AceptadoDe`] en JSON acotado a
/// [`TOPE_RAW_DETAILS`].
pub fn anotaciones(v: &Veredicto) -> Vec<Vec<Anotacion>> {
    let todas: Vec<Anotacion> = v
        .aceptados
        .iter()
        .map(|a| {
            let h = &a.hallazgo;
            let titulo = format!(
                "{} {} {} ({})",
                plano(&h.id, 64),
                h.severidad.texto(),
                h.clase.texto(),
                plano(&a.revisora, 120)
            );
            Anotacion {
                path: h.fichero.clone(),
                start_line: h.linea,
                end_line: h.linea,
                annotation_level: match h.severidad {
                    Severidad::Critico | Severidad::Importante => "failure",
                    Severidad::Menor => "warning",
                }
                .to_owned(),
                // GitHub corta el titulo en 255 caracteres.
                title: titulo.chars().take(255).collect(),
                message: Some(plano(&h.descripcion, 4000))
                    .filter(|m| !m.is_empty())
                    .unwrap_or_else(|| MENSAJE_SIN_TEXTO.to_owned()),
                raw_details: raw_details(a),
            }
        })
        .collect();
    todas
        .chunks(ANOTACIONES_POR_LOTE)
        .map(<[Anotacion]>::to_vec)
        .collect()
}

/// El resumen del check: el veredicto, una fila por revisora (todo lo del
/// modelo por [`plano`]) y, si se dan, las lineas de `extra` (la seleccion y
/// el estado de cada revisora que deja el workflow), tambien por [`plano`].
pub fn resumen_veredicto(v: &Veredicto, extra: &str) -> String {
    let explicacion = match v.dictamen {
        Dictamen::Rojo => {
            "Queda abierto al menos un hallazgo CRITICO o IMPORTANTE aceptado: el check sale en rojo (F6-5)."
        }
        Dictamen::Verde => "Sin hallazgos CRITICOS ni IMPORTANTES abiertos.",
        Dictamen::Incompleta => {
            "Alguna revisora no dejo una respuesta valida (o no hubo ninguna): la revision no se completo y el check sale en rojo (F6-5)."
        }
    };
    let mut s = format!(
        "**Veredicto: {}** (conclusion {}). {explicacion}\n\n\
         Aceptados: {} CRITICO, {} IMPORTANTE, {} MENOR; rechazados: {}; revisoras: {}, incompletas: {}.\n\n",
        v.dictamen.texto(),
        v.dictamen.conclusion(),
        v.criticos,
        v.importantes,
        v.menores,
        v.rechazados,
        v.filas.len(),
        v.incompletas()
    );
    if !v.filas.is_empty() {
        s.push_str("| revisora | aceptados | rechazados | respuesta |\n|---|---|---|---|\n");
        for (k, f) in v.filas.iter().enumerate() {
            let (quien, respuesta) = match (&f.revisora, f.incompleta) {
                (None, _) => (format!("#{k}"), "sin-resultado".to_owned()),
                (Some(r), None) => (plano(r, 120), "valida".to_owned()),
                (Some(r), Some(m)) => (plano(r, 120), format!("incompleta ({m})")),
            };
            s.push_str(&format!(
                "| {quien} | {} | {} | {respuesta} |\n",
                f.aceptados, f.rechazados
            ));
        }
        s.push('\n');
    }
    let lineas: Vec<String> = extra
        .lines()
        .filter(|l| no_vacio(l))
        .map(|l| plano(l, 2000))
        .collect();
    if !lineas.is_empty() {
        s.push_str("Seleccion y estado de cada revisora:\n\n");
        for l in lineas {
            s.push_str(&format!("- {l}\n"));
        }
    }
    s
}

/// El informe de la revision IA en la carpeta de la tarea (F6-5,
/// `$defs/informe-tarea`): leido del check publicado, sin la prosa del
/// modelo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InformeTarea {
    pub version: u32,
    pub tarea: String,
    pub pr: u64,
    pub sha: String,
    pub conclusion: String,
    pub veredicto: Dictamen,
    pub criticos: usize,
    pub importantes: usize,
    pub menores: usize,
    /// El resumen del check (ya pasado por [`plano`]).
    pub resumen: String,
    pub aceptados: Vec<AceptadoDe>,
}

impl InformeTarea {
    /// El informe de la tarea `tarea` con lo leido del check.
    pub fn nuevo(
        tarea: &str,
        pr: u64,
        sha: &str,
        (conclusion, veredicto): (&str, Dictamen),
        resumen: &str,
        aceptados: Vec<AceptadoDe>,
    ) -> Self {
        let n = |s: Severidad| {
            aceptados
                .iter()
                .filter(|a| a.hallazgo.severidad == s)
                .count()
        };
        Self {
            version: VERSION,
            tarea: tarea.to_owned(),
            pr,
            sha: sha.to_owned(),
            conclusion: conclusion.to_owned(),
            veredicto,
            criticos: n(Severidad::Critico),
            importantes: n(Severidad::Importante),
            menores: n(Severidad::Menor),
            resumen: resumen.to_owned(),
            aceptados,
        }
    }

    /// El mismo informe en Markdown: veredicto y tabla de aceptados por
    /// [`plano`].
    pub fn markdown(&self) -> String {
        let mut s = format!(
            "# Revision IA de {} (PR {}, commit {})\n\n\
             Veredicto: **{}** (conclusion {}). Aceptados: {} CRITICO, {} IMPORTANTE, {} MENOR.\n\n",
            plano(&self.tarea, 64),
            self.pr,
            plano(&self.sha, 64),
            self.veredicto.texto(),
            plano(&self.conclusion, 32),
            self.criticos,
            self.importantes,
            self.menores
        );
        if self.aceptados.is_empty() {
            s.push_str("Sin hallazgos aceptados.\n");
        } else {
            s.push_str(
                "| revisora | id | severidad | clase | fichero | linea | evidencia | descripcion |\n|---|---|---|---|---|---|---|---|\n",
            );
            for a in &self.aceptados {
                let h = &a.hallazgo;
                s.push_str(&format!(
                    "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
                    plano(&a.revisora, 120),
                    plano(&h.id, 64),
                    h.severidad.texto(),
                    h.clase.texto(),
                    plano(&h.fichero, 200),
                    h.linea,
                    h.evidencia.tipo.texto(),
                    plano(&h.descripcion, 400)
                ));
            }
        }
        s
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;

    const DIFF: &str = include_str!("../../tests/fixtures/hallazgos/diff.txt");
    const MIXTA: &str = include_str!("../../tests/fixtures/hallazgos/respuesta-mixta.md");
    const ROJO: &str = include_str!("../../tests/fixtures/informe-verificacion/xtask-rojo.json");
    const ESQUEMA: &str = include_str!("../../esquemas/hallazgos.schema.json");

    fn diff() -> Diff {
        Diff::leer(DIFF)
    }

    fn una(json: &str) -> String {
        format!("Informe.\n\n```json\n[{json}]\n```\n")
    }

    const BORDE: &str = r#"{"id":"B","severidad":"MENOR","clase":"correccion","fichero":"src/calculo.rs","linea":5,"descripcion":"d","evidencia":{"tipo":"borde","entrada":"x","esperado":"1","obtenido":"2"}}"#;

    #[test]
    fn el_diff_da_las_lineas_del_lado_nuevo() {
        let d = diff();
        assert_eq!(
            d.linea("src/calculo.rs", 1),
            Some("//! Calculos de prueba.")
        );
        assert_eq!(d.linea("src/calculo.rs", 5), Some("    if v.len() > 1 {"));
        assert_eq!(d.linea("src/calculo.rs", 12), Some("    x.min(100)"));
        assert_eq!(d.linea("src/calculo.rs", 13), Some("}"));
        assert_eq!(d.linea("src/calculo.rs", 14), None);
        assert_eq!(d.linea("src/calculo.rs", 0), None);
        // Las lineas borradas no estan en el lado nuevo: tras la 2 viene la 3
        // anadida, no la vieja.
        assert_eq!(
            d.linea("src/calculo.rs", 3),
            Some("/// Media de los valores; `None` si no hay ninguno.")
        );
        // Segundo fichero, un segundo trozo y una linea anadida que empieza
        // por `++`.
        assert_eq!(d.linea("docs/nota.md", 10), Some("++ no es una cabecera"));
        assert_eq!(d.linea("docs/nota.md", 11), Some("fin"));
        assert_eq!(d.linea("docs/nota.md", 1), Some("uno bis"));
        assert_eq!(d.linea("docs/nota.md", 5), None);
        // Un fichero borrado no deja lineas.
        assert_eq!(d.linea("viejo.rs", 1), None);
        // Cortado a mitad, lo que no llego no esta.
        let corte = DIFF.find("pub fn tope").unwrap_or(DIFF.len());
        let cortado = Diff::leer(&DIFF[..corte]);
        assert_eq!(cortado.linea("src/calculo.rs", 10), Some(""));
        assert_eq!(cortado.linea("src/calculo.rs", 12), None);
    }

    /// Git entrecomilla las rutas no ASCII con escapes octales
    /// (`core.quotePath`): el hallazgo da la ruta real.
    #[test]
    fn el_diff_decodifica_las_rutas_entrecomilladas() {
        let texto = concat!(
            "diff --git \"a/src/\\303\\261.rs\" \"b/src/\\303\\261.rs\"\n",
            "--- \"a/src/\\303\\261.rs\"\n",
            "+++ \"b/src/\\303\\261.rs\"\n",
            "@@ -1,1 +1,2 @@\n",
            " uno\n",
            "+dos\n",
            "diff --git a/x b/x\n",
            "+++ \"b/con \\\"comillas\\\"\\\\y\\ttab\"\n",
            "@@ -0,0 +1 @@\n",
            "+z\n",
        );
        let d = Diff::leer(texto);
        assert_eq!(d.linea("src/ñ.rs", 2), Some("dos"));
        assert_eq!(d.linea("src/\\303\\261.rs", 2), None);
        assert_eq!(d.linea("con \"comillas\"\\y\ttab", 1), Some("z"));
        let v = validar(
            "r",
            &una(
                r#"{"id":"Q","severidad":"MENOR","clase":"correccion","fichero":"src/ñ.rs","linea":2,"descripcion":"d","evidencia":{"tipo":"borde","entrada":"x","esperado":"1","obtenido":"2"}}"#,
            ),
            &d,
            None,
        );
        assert_eq!((v.aceptados.len(), v.rechazados.len()), (1, 0), "{v:?}");
        // Un escape que no es de git se deja como esta.
        assert_eq!(sin_escapes_de_git("a\\qb\\"), "a\\qb\\");
    }

    #[test]
    fn la_respuesta_mixta_da_tres_aceptados_y_dos_rechazados() {
        let v = validar("r", MIXTA, &diff(), None);
        let ids: Vec<&str> = v.aceptados.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids, ["H1", "H2", "H3"]);
        assert_eq!(v.motivos(), ["H4:sin-evidencia", "H5:linea-fuera-del-diff"]);
    }

    /// Cada regla, con un caso que la rompe.
    #[test]
    fn cada_regla_rechaza_con_su_motivo() {
        let d = diff();
        let casos: Vec<(String, &str)> = vec![
            ("Sin bloque, solo prosa.".to_owned(), "respuesta:sin-bloque"),
            (format!("{}\n{}", una(BORDE), una(BORDE)), "respuesta:bloque-repetido"),
            ("```json\n[\n".to_owned(), "respuesta:json-invalido"),
            ("```json\n[{]\n```".to_owned(), "respuesta:json-invalido"),
            ("```json\n{\"id\":\"B\"}\n```".to_owned(), "respuesta:json-invalido"),
            (una("3"), "#0:hallazgo-invalido"),
            (una(&BORDE.replace("\"MENOR\"", "\"GRAVE\"")), "B:hallazgo-invalido"),
            (una(&BORDE.replace("\"clase\":\"correccion\",", "")), "B:hallazgo-invalido"),
            (una(&BORDE.replace("\"correccion\"", "\"estilo\"")), "B:hallazgo-invalido"),
            (una(&BORDE.replace("\"linea\":5", "\"linea\":0")), "B:hallazgo-invalido"),
            (una(&BORDE.replace("\"linea\":5", "\"linea\":5,\"x\":1")), "B:hallazgo-invalido"),
            (una(&BORDE.replace("\"descripcion\":\"d\"", "\"descripcion\":\" \"")), "B:hallazgo-invalido"),
            (una(&BORDE.replace("\"id\":\"B\"", "\"id\":\"no vale\"")), "#0:hallazgo-invalido"),
            (una(&format!("{BORDE},{BORDE}")), "B:id-repetido"),
            (una(&BORDE.replace(",\"evidencia\":{\"tipo\":\"borde\",\"entrada\":\"x\",\"esperado\":\"1\",\"obtenido\":\"2\"}", "")), "B:sin-evidencia"),
            (una(&BORDE.replace("\"tipo\":\"borde\"", "\"tipo\":\"opinion\"")), "B:evidencia-incompleta"),
            (una(&BORDE.replace(",\"obtenido\":\"2\"", "")), "B:evidencia-incompleta"),
            (una(&BORDE.replace("\"obtenido\":\"2\"", "\"obtenido\":\"\"")), "B:evidencia-incompleta"),
            (una(&BORDE.replace("\"obtenido\":\"2\"", "\"obtenido\":\"2\",\"test\":\"t\"")), "B:evidencia-incompleta"),
            (una(&BORDE.replace("\"obtenido\":\"2\"", "\"obtenido\":\"2\",\"x\":1")), "B:evidencia-incompleta"),
            (una(&BORDE.replace("\"linea\":5", "\"linea\":99")), "B:linea-fuera-del-diff"),
            (una(&BORDE.replace("src/calculo.rs", "src/otro.rs")), "B:linea-fuera-del-diff"),
            (una(&BORDE.replace("\"obtenido\":\"2\"", "\"obtenido\":\" 1 \"")), "B:borde-sin-diferencia"),
            (
                una(r#"{"id":"M","severidad":"IMPORTANTE","clase":"contrato","fichero":"src/calculo.rs","linea":12,"descripcion":"d","evidencia":{"tipo":"mutante","linea_original":"x.min(10)","mutacion":"x.min(11)","test":"t"}}"#),
                "M:mutante-no-coincide",
            ),
            (
                una(r#"{"id":"M","severidad":"IMPORTANTE","clase":"contrato","fichero":"src/calculo.rs","linea":12,"descripcion":"d","evidencia":{"tipo":"mutante","linea_original":"x.min(100)","mutacion":" x.min(100)","test":"t"}}"#),
                "M:mutante-sin-cambio",
            ),
            (
                una(r#"{"id":"R","severidad":"MENOR","clase":"correccion","fichero":"src/calculo.rs","linea":8,"descripcion":"d","evidencia":{"tipo":"reproduccion","pasos":[]}}"#),
                "R:evidencia-incompleta",
            ),
            (
                una(r#"{"id":"R","severidad":"MENOR","clase":"correccion","fichero":"src/calculo.rs","linea":8,"descripcion":"d","evidencia":{"tipo":"reproduccion","pasos":["a"]}}"#),
                "R:evidencia-incompleta",
            ),
            (
                una(r#"{"id":"R","severidad":"MENOR","clase":"correccion","fichero":"src/calculo.rs","linea":8,"descripcion":"d","evidencia":{"tipo":"reproduccion","paso_rojo":"clippy"}}"#),
                "R:paso-rojo-no-esta",
            ),
        ];
        for (respuesta, esperado) in casos {
            let v = validar("r", &respuesta, &d, None);
            assert_eq!(v.motivos(), [esperado], "{respuesta}");
        }
        // El caso base se acepta.
        let v = validar("r", &una(BORDE), &d, None);
        assert_eq!((v.aceptados.len(), v.rechazados.len()), (1, 0));
        // Mas de TOPE_HALLAZGOS: los que sobran, un rechazo.
        let muchos: Vec<String> = (0..=TOPE_HALLAZGOS)
            .map(|i| BORDE.replace("\"id\":\"B\"", &format!("\"id\":\"B{i}\"")))
            .collect();
        let v = validar("r", &una(&muchos.join(",")), &d, None);
        assert_eq!(v.aceptados.len(), TOPE_HALLAZGOS);
        assert_eq!(v.motivos(), [format!("#{TOPE_HALLAZGOS}:fuera-de-tope")]);
    }

    #[test]
    fn paso_rojo_se_contrasta_con_el_informe() {
        let informe = crate::verificacion::informe::leer(ROJO).unwrap_or_else(|e| panic!("{e}"));
        let r = |paso: &str| {
            una(&format!(
                r#"{{"id":"R","severidad":"MENOR","clase":"correccion","fichero":"src/calculo.rs","linea":8,"descripcion":"d","evidencia":{{"tipo":"reproduccion","paso_rojo":"{paso}"}}}}"#
            ))
        };
        let v = validar("r", &r("clippy"), &diff(), Some(&informe));
        assert_eq!((v.aceptados.len(), v.rechazados.len()), (1, 0), "{v:?}");
        // Con espacios alrededor tambien: el nombre se compara sin ellos.
        let v = validar("r", &r(" clippy "), &diff(), Some(&informe));
        assert_eq!((v.aceptados.len(), v.rechazados.len()), (1, 0), "{v:?}");
        // `fmt` esta en verde en ese informe.
        let v = validar("r", &r("fmt"), &diff(), Some(&informe));
        assert_eq!(v.motivos(), ["R:paso-rojo-no-esta"]);
    }

    /// Las instrucciones nombran cada motivo y su ejemplo pasa el validador.
    #[test]
    fn las_instrucciones_salen_del_validador() {
        let t = instrucciones();
        for (id, _) in MOTIVOS {
            assert!(t.contains(&format!("`{id}`")), "falta {id}");
        }
        for x in [
            "`CRITICO`",
            "`IMPORTANTE`",
            "`MENOR`",
            "`reproduccion`",
            "`mutante`",
            "`borde`",
        ] {
            assert!(t.contains(x), "falta {x}");
        }
        for c in Clase::TODAS {
            assert!(t.contains(&format!("`{}`", c.texto())), "falta {c:?}");
        }
        let v = validar("r", &t, &diff(), None);
        assert_eq!((v.aceptados.len(), v.rechazados.len()), (2, 0), "{v:?}");
    }

    /// El esquema declara los mismos motivos, severidades y tipos que el
    /// codigo, y las mismas claves que la salida.
    #[test]
    fn el_esquema_y_el_codigo_dicen_lo_mismo() {
        let e: Value = serde_json::from_str(ESQUEMA).unwrap_or_else(|x| panic!("{x}"));
        let textos = |v: &Value| -> Vec<String> {
            v.as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default()
        };
        let motivos: Vec<String> = MOTIVOS.iter().map(|(m, _)| (*m).to_owned()).collect();
        assert_eq!(
            textos(&e["$defs"]["rechazo"]["properties"]["motivo"]["enum"]),
            motivos
        );
        assert_eq!(
            textos(&e["$defs"]["hallazgo"]["properties"]["severidad"]["enum"]),
            ["CRITICO", "IMPORTANTE", "MENOR"]
        );
        let clases: Vec<&str> = Clase::TODAS.iter().map(|c| c.texto()).collect();
        assert_eq!(
            textos(&e["$defs"]["hallazgo"]["properties"]["clase"]["enum"]),
            clases
        );
        assert_eq!(e["properties"]["version"]["const"], VERSION);
        let v = validar("r", MIXTA, &diff(), None);
        let salida = serde_json::to_value(&v).unwrap_or_else(|x| panic!("{x}"));
        let claves = |v: &Value| -> Vec<String> {
            let mut c: Vec<String> = v
                .as_object()
                .map(|m| m.keys().cloned().collect())
                .unwrap_or_default();
            c.sort();
            c
        };
        assert_eq!(claves(&salida), claves(&e["properties"]));
        assert_eq!(
            claves(&salida["rechazados"][0]),
            claves(&e["$defs"]["rechazo"]["properties"])
        );
        assert_eq!(
            claves(&salida["aceptados"][0]),
            claves(&e["$defs"]["hallazgo"]["properties"])
        );
        for (tipo, campos) in [
            (TipoEvidencia::Reproduccion, "reproduccion"),
            (TipoEvidencia::Mutante, "mutante"),
            (TipoEvidencia::Borde, "borde"),
        ] {
            let mut esperadas: Vec<String> =
                tipo.campos().iter().map(|c| (*c).to_owned()).collect();
            esperadas.push("tipo".to_owned());
            esperadas.sort();
            assert_eq!(claves(&e["$defs"][campos]["properties"]), esperadas);
        }
    }

    #[test]
    fn el_resumen_no_deja_pasar_markdown_del_modelo() {
        let malo = BORDE.replace(
            "\"descripcion\":\"d\"",
            "\"descripcion\":\"| x | ![i](http://e/p.png) <b>\\n# t `c`\"",
        );
        let v = validar("r", &una(&malo), &diff(), None);
        let md = resumen_markdown(&v);
        assert!(
            md.contains(r"\| x \| \!\[i\]\(http\://e/p.png\) \<b\> \# t \`c\`"),
            "{md}"
        );
        // Autoenlaces, menciones, matematicas y emojis salen escapados; los
        // caracteres de formato (Cf) se quitan; el resto del texto, intacto.
        let raro = "\u{202E}evil \u{200B}zero @user http://x.y www.e.com $x$ :smile: ñ \u{2066}a\u{2069}\u{FEFF}b \u{AD}c";
        assert_eq!(
            plano(raro, 400),
            r"evil zero \@user http\://x.y www\.e.com \$x\$ \:smile: ñ ab c"
        );
        for c in plano(raro, 400).chars() {
            assert!(!es_de_formato(c), "queda U+{:04X}", u32::from(c));
        }
        // Un `:` suelto no es un emoji ni un enlace: no se escapa.
        assert_eq!(
            plano("taskcode-plugin:revisor a: b 10:30", 100),
            "taskcode-plugin:revisor a: b 10:30"
        );
        assert_eq!(plano(&"a".repeat(10), 4), "aaaa…");
        let v = validar("r", "nada", &diff(), None);
        assert!(resumen_markdown(&v).contains("| respuesta | sin-bloque |"));
    }

    /// Las claves obligatorias del esquema son todas las que escribe el
    /// codigo: quitar una de `required` (p. ej. `raw_details`, que lee
    /// `leer-revision`, o `aceptados` del informe) da rojo (MEN-2 de la
    /// ronda 1 de TASK-051).
    #[test]
    fn el_esquema_exige_todas_las_claves_de_anotacion_e_informe() {
        let e: Value = serde_json::from_str(ESQUEMA).unwrap_or_else(|x| panic!("{x}"));
        let ordenadas = |mut c: Vec<String>| {
            c.sort();
            c
        };
        let requeridas = |def: &str| -> Vec<String> {
            ordenadas(
                e["$defs"][def]["required"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(Value::as_str)
                            .map(str::to_owned)
                            .collect()
                    })
                    .unwrap_or_default(),
            )
        };
        let claves = |v: &Value| -> Vec<String> {
            ordenadas(
                v.as_object()
                    .map(|m| m.keys().cloned().collect())
                    .unwrap_or_default(),
            )
        };
        let v = veredicto(&[Some(validar("r", MIXTA, &diff(), None))]);
        let anotacion = anotaciones(&v)
            .first()
            .and_then(|l| l.first())
            .cloned()
            .unwrap_or_else(|| panic!("sin anotaciones"));
        let anotacion = serde_json::to_value(&anotacion).unwrap_or_else(|x| panic!("{x}"));
        assert!(requeridas("anotacion").contains(&"raw_details".to_owned()));
        assert_eq!(requeridas("anotacion"), claves(&anotacion));
        // El esquema declara el mismo tope de raw_details que el codigo.
        assert_eq!(
            e["$defs"]["anotacion"]["properties"]["raw_details"]["maxLength"],
            TOPE_RAW_DETAILS
        );
        let informe = InformeTarea::nuevo(
            "TASK-051",
            12,
            &"a".repeat(40),
            ("failure", Dictamen::Rojo),
            "r",
            Vec::new(),
        );
        let informe = serde_json::to_value(&informe).unwrap_or_else(|x| panic!("{x}"));
        assert!(requeridas("informe-tarea").contains(&"aceptados".to_owned()));
        assert_eq!(requeridas("informe-tarea"), claves(&informe));
        let de = serde_json::to_value(&v.aceptados).unwrap_or_else(|x| panic!("{x}"));
        assert_eq!(requeridas("aceptado-de"), claves(&de[0]));
    }

    /// Un mensaje que `plano` deja vacio no sale vacio, y `raw_details` cabe
    /// en el tope de la API y se lee de vuelta (MEN-2 y MEN-3 de la ronda 1
    /// de TASK-051).
    #[test]
    fn las_anotaciones_respetan_los_limites_de_la_api() {
        let con = |descripcion: &str| -> Veredicto {
            let h = BORDE.replace(
                "\"descripcion\":\"d\"",
                &format!(
                    "\"descripcion\":{}",
                    serde_json::to_string(descripcion).unwrap_or_default()
                ),
            );
            let v = validar("r", &una(&h), &diff(), None);
            assert_eq!(v.aceptados.len(), 1, "{v:?}");
            veredicto(&[Some(v)])
        };
        let primera = |v: &Veredicto| -> Anotacion {
            anotaciones(v)
                .first()
                .and_then(|l| l.first())
                .cloned()
                .unwrap_or_else(|| panic!("sin anotaciones"))
        };
        // Solo controles y caracteres de ancho cero: `validar` lo acepta.
        let a = primera(&con("\u{7}\u{200B}\u{FEFF}"));
        assert_eq!(a.message, MENSAJE_SIN_TEXTO);
        // Una descripcion normal sale tal cual.
        assert_eq!(primera(&con("d")).message, "d");
        // 70 000 caracteres de descripcion: recortada hasta caber.
        let largo = format!("ñ{}", "x".repeat(70_000));
        let v = con(&largo);
        let a = primera(&v);
        assert!(
            a.raw_details.len() <= TOPE_RAW_DETAILS,
            "{}",
            a.raw_details.len()
        );
        let leido: AceptadoDe =
            serde_json::from_str(&a.raw_details).unwrap_or_else(|x| panic!("{x}"));
        assert_eq!(leido.hallazgo.id, "B");
        assert_eq!(leido.revisora, "r");
        assert!(leido.hallazgo.descripcion.starts_with("ñxx"));
        assert!(leido.hallazgo.descripcion.ends_with('…'));
        assert!(leido.hallazgo.descripcion.len() > TOPE_RAW_DETAILS / 4);
        // Lo que cabe no se toca.
        let corto = con("d");
        assert_eq!(
            primera(&corto).raw_details,
            serde_json::to_string(&corto.aceptados[0]).unwrap_or_default()
        );
        // Muchos pasos cortos: se recorta la lista.
        let mut v = con("d");
        v.aceptados[0].hallazgo.evidencia = Evidencia {
            tipo: TipoEvidencia::Reproduccion,
            pasos: Some(vec!["paso".to_owned(); 30_000]),
            fallo_esperado: Some("f".to_owned()),
            paso_rojo: None,
            linea_original: None,
            mutacion: None,
            test: None,
            entrada: None,
            esperado: None,
            obtenido: None,
        };
        let a = primera(&v);
        assert!(
            a.raw_details.len() <= TOPE_RAW_DETAILS,
            "{}",
            a.raw_details.len()
        );
        let leido: AceptadoDe =
            serde_json::from_str(&a.raw_details).unwrap_or_else(|x| panic!("{x}"));
        let pasos = leido.hallazgo.evidencia.pasos.unwrap_or_default();
        assert!(!pasos.is_empty() && pasos.len() < 30_000, "{}", pasos.len());
    }

    /// El dictamen publicado solo se da si el titulo casa con la conclusion
    /// y los aceptados (MEN-1 de la ronda 1 de TASK-051).
    #[test]
    fn el_dictamen_publicado_casa_titulo_conclusion_y_aceptados() {
        let v = veredicto(&[Some(validar("r", MIXTA, &diff(), None))]);
        let t = v.titulo();
        let c = v.dictamen.conclusion();
        assert_eq!(dictamen_publicado(&t, c, &v.aceptados), Some(v.dictamen));
        // Un aceptado de menos (un lote perdido).
        let menos = v.aceptados.get(1..).unwrap_or_default();
        assert_eq!(dictamen_publicado(&t, c, menos), None);
        assert_eq!(dictamen_publicado(&t, "success", &v.aceptados), None);
        assert_eq!(
            dictamen_publicado("revision-ia: neutral", c, &v.aceptados),
            None
        );
        // Incompleta: sin anotaciones, cualquier titulo; con anotaciones, el
        // suyo exacto.
        let inc = "revision-ia: incompleta (sin veredicto de validar)";
        assert_eq!(
            dictamen_publicado(inc, "failure", &[]),
            Some(Dictamen::Incompleta)
        );
        assert_eq!(dictamen_publicado(inc, "failure", &v.aceptados), None);
        let mut i = v.clone();
        i.dictamen = Dictamen::Incompleta;
        assert_eq!(
            dictamen_publicado(&i.titulo(), "failure", &v.aceptados),
            Some(Dictamen::Incompleta)
        );
    }
}
