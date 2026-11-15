//! De HTTP a la linea de contrato, con el arreglo para la persona (F2-6 y
//! F2-9: «errores dirigidos a la persona»). Es el unico sitio que conoce los
//! codigos HTTP, y lo comparten los adaptadores de GitHub y GitLab: el
//! [`Proveedor`] pone el nombre de la plataforma en el diagnostico.
//!
//! El diagnostico dice que hacer. El mensaje de la API es dato no confiable
//! (F1-8, Z2): se recorta y se quitan los caracteres de control antes de
//! citarlo. Nunca lleva el token ni cabeceras.

use super::Proveedor;
use super::credenciales::Rol;
use super::http::{FalloHttp, LIMITE_CUERPO, Respuesta};
use crate::contrato::{Informe, Resultado};
use crate::i18n::catalogo::plataforma as t;
use crate::i18n::{Mensaje, Texto};

/// Aviso de GitHub cuando el plan no da rulesets ni proteccion de rama.
pub const AVISO_PLAN: &str = "Upgrade to GitHub Pro";

/// Mensaje de la API, saneado: sin controles y como mucho 200 caracteres.
/// GitHub lo da en `message`; GitLab, en `message` (texto, objeto o lista)
/// o en `error`.
pub fn mensaje_api(r: &Respuesta) -> String {
    let v = serde_json::from_str::<serde_json::Value>(&r.cuerpo).ok();
    let m = v
        .as_ref()
        .and_then(|v| v.get("message").or_else(|| v.get("error")))
        .map(|m| match m.as_str() {
            Some(s) => s.to_owned(),
            None => m.to_string(),
        })
        .unwrap_or_default();
    sanear(&m)
}

/// Texto externo listo para un diagnostico.
pub fn sanear(t: &str) -> String {
    t.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .take(200)
        .collect::<String>()
        .trim()
        .to_owned()
}

/// Si la respuesta es el 403 de plan de GitHub.
pub fn es_aviso_de_plan(r: &Respuesta) -> bool {
    r.estado == 403 && mensaje_api(r).contains(AVISO_PLAN)
}

/// Si la respuesta es un limite de peticiones agotado.
fn es_limite(r: &Respuesta) -> bool {
    // El limite secundario de GitHub puede llegar sin cabeceras: entonces lo
    // dice el mensaje. GitLab da 429.
    (r.estado == 403 || r.estado == 429)
        && (r.restantes.as_deref() == Some("0")
            || r.reintentar.is_some()
            || r.estado == 429
            || mensaje_api(r).to_ascii_lowercase().contains("rate limit"))
}

/// Segundos de espera que da la API, si los da.
fn espera(r: &Respuesta) -> String {
    if let Some(s) = r.reintentar.as_deref().filter(|s| es_numero(s)) {
        return s.to_owned();
    }
    let ahora = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    r.reinicio
        .as_deref()
        .and_then(|s| s.parse::<u64>().ok())
        .map_or_else(|| "60".to_owned(), |t| t.saturating_sub(ahora).to_string())
}

fn es_numero(s: &str) -> bool {
    !s.is_empty() && s.len() <= 10 && s.bytes().all(|b| b.is_ascii_digit())
}

/// Permiso que necesita una operacion, para el diagnostico del 403: un
/// alcance de la API tal cual (`contents:write`) o una descripcion del
/// catalogo (GitLab: scope y rol).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permiso {
    /// Nombre del alcance, igual en todos los idiomas.
    Alcance(&'static str),
    /// Descripcion que se traduce.
    Descrito(&'static Texto),
}

impl From<&'static str> for Permiso {
    fn from(a: &'static str) -> Self {
        Self::Alcance(a)
    }
}

impl From<&'static Texto> for Permiso {
    fn from(t: &'static Texto) -> Self {
        Self::Descrito(t)
    }
}

impl Permiso {
    /// Rellena el hueco `{permiso}` de `m`.
    pub fn en(self, m: Mensaje) -> Mensaje {
        match self {
            Self::Alcance(a) => m.con("permiso", a),
            Self::Descrito(t) => m.con_mensaje("permiso", t),
        }
    }
}

/// Traduce una respuesta que no es 2xx. `op` es la operacion, `rol` con que
/// identidad corria y `permiso` el que necesita (para el 403 sin plan).
pub fn traducir(p: Proveedor, op: &str, rol: Rol, permiso: Permiso, r: &Respuesta) -> Informe {
    let api = mensaje_api(r);
    let nombre = p.titulo();
    // La cita solo lleva el nombre de la plataforma y el texto de la API:
    // nada que traducir.
    let cita = if api.is_empty() {
        String::new()
    } else {
        format!(" ({nombre}: «{api}»)")
    };
    let objeto = match p {
        Proveedor::GitHub => &t::OBJETO_GITHUB,
        Proveedor::GitLab => &t::OBJETO_GITLAB,
    };
    let con_cita = |texto: &'static Texto| Mensaje::nuevo(texto).con("cita", cita.clone());
    match r.estado {
        401 => Informe::nuevo(op, Resultado::Denegada)
            .con("motivo", "credenciales")
            .con("rol", rol.nombre())
            .diagnostico(
                con_cita(&t::TOKEN_NO_VALE)
                    .con("rol", rol.nombre())
                    .con("variable", rol.variable()),
            ),
        _ if es_limite(r) => {
            let s = espera(r);
            Informe::nuevo(op, Resultado::Agotada)
                .con("motivo", "limite-peticiones")
                .con("reintenta-en", s.clone())
                .diagnostico(con_cita(&t::LIMITE_AGOTADO).con("segundos", s))
        }
        403 if p == Proveedor::GitHub && api.contains(AVISO_PLAN) => {
            Informe::nuevo(op, Resultado::Rechazada)
                .con("motivo", "plan")
                .diagnostico(con_cita(&t::PLAN_GITHUB))
        }
        403 => Informe::nuevo(op, Resultado::Denegada)
            .con("motivo", "permisos")
            .con("rol", rol.nombre())
            .diagnostico(permiso.en(con_cita(&t::SIN_PERMISO).con("rol", rol.nombre()))),
        404 => Informe::nuevo(op, Resultado::Denegada)
            .con("motivo", "no-existe-o-sin-acceso")
            .con("rol", rol.nombre())
            .diagnostico(
                con_cita(&t::NO_EXISTE)
                    .con("rol", rol.nombre())
                    .con_mensaje("objeto", objeto)
                    .con("plataforma", nombre),
            ),
        // GitLab da 405 o 406 cuando el MR no se puede mergear.
        405 => no_mergeable(op, nombre, &cita),
        406 if p == Proveedor::GitLab => no_mergeable(op, nombre, &cita),
        409 => Informe::nuevo(op, Resultado::Rechazada)
            .con("motivo", "head-cambiado")
            .diagnostico(con_cita(&t::HEAD_CAMBIADO)),
        // El detalle del 422 de GitHub viene en `errors[].message`, no en
        // `message`.
        422 if r.cuerpo.contains("No commits between") => Informe::nuevo(op, Resultado::Rechazada)
            .con("motivo", "sin-commits")
            .diagnostico(con_cita(&t::SIN_COMMITS)),
        400 if p == Proveedor::GitLab => validacion(op, nombre, &cita),
        422 => validacion(op, nombre, &cita),
        e => Informe::nuevo(op, Resultado::Error)
            .con("motivo", "http")
            .con("estado", e.to_string())
            .diagnostico(
                con_cita(&t::RESPUESTA_INESPERADA)
                    .con("plataforma", nombre)
                    .con("estado", e.to_string()),
            ),
    }
}

fn no_mergeable(op: &str, nombre: &str, cita: &str) -> Informe {
    Informe::nuevo(op, Resultado::Rechazada)
        .con("motivo", "no-mergeable")
        .diagnostico(
            Mensaje::nuevo(&t::NO_MERGEABLE)
                .con("plataforma", nombre)
                .con("cita", cita),
        )
}

fn validacion(op: &str, nombre: &str, cita: &str) -> Informe {
    Informe::nuevo(op, Resultado::Error)
        .con("motivo", "validacion")
        .diagnostico(
            Mensaje::nuevo(&t::VALIDACION)
                .con("plataforma", nombre)
                .con("cita", cita),
        )
}

/// Traduce un fallo sin respuesta.
pub fn fallo(op: &str, f: &FalloHttp) -> Informe {
    match f {
        FalloHttp::Rechazada(m) => Informe::nuevo(op, Resultado::Rechazada)
            .con("motivo", "destino")
            .diagnostico(Mensaje::nuevo(&t::LLAMADA_RECHAZADA).con_mensaje("motivo", *m)),
        FalloHttp::Sinred(m) => Informe::nuevo(op, Resultado::Sinred)
            .diagnostico(Mensaje::nuevo(&t::SIN_RED).con("detalle", sanear(m))),
        FalloHttp::Tls(m) => Informe::nuevo(op, Resultado::Error)
            .con("motivo", "tls")
            .diagnostico(Mensaje::nuevo(&t::TLS).con("detalle", sanear(m))),
        FalloHttp::RespuestaInvalida(m) => Informe::nuevo(op, Resultado::Error)
            .con("motivo", "respuesta-invalida")
            .diagnostico(Mensaje::nuevo(&t::RESPUESTA_ILEGIBLE).con("detalle", sanear(m))),
        FalloHttp::CuerpoGrande => Informe::nuevo(op, Resultado::Error)
            .con("motivo", "respuesta-invalida")
            .diagnostico(Mensaje::nuevo(&t::RESPUESTA_ILEGIBLE).con_mensaje(
                "detalle",
                Mensaje::nuevo(&t::DECLARA_MAS_DE).con("limite", LIMITE_CUERPO.to_string()),
            )),
        FalloHttp::Redireccion(e) => Informe::nuevo(op, Resultado::Error)
            .con("motivo", "redireccion")
            .diagnostico(Mensaje::nuevo(&t::REDIRECCION).con("estado", e.to_string())),
        FalloHttp::Otro(m) => Informe::nuevo(op, Resultado::Error)
            .con("motivo", "cliente")
            .diagnostico(Mensaje::nuevo(&t::CLIENTE_HTTP).con("detalle", sanear(m))),
    }
}

/// Respuesta 2xx con un cuerpo que no tiene la forma esperada. `que` dice
/// que falta, tal cual: el error del lector JSON o el nombre de un campo.
pub fn respuesta_invalida(p: Proveedor, op: &str, que: &str) -> Informe {
    forma_inesperada(op, p, |m| m.con("que", que))
}

/// Como [`respuesta_invalida`], con `que` descrito en el catalogo.
pub fn respuesta_invalida_por(p: Proveedor, op: &str, que: &'static Texto) -> Informe {
    forma_inesperada(op, p, |m| m.con_mensaje("que", que))
}

fn forma_inesperada(op: &str, p: Proveedor, que: impl FnOnce(Mensaje) -> Mensaje) -> Informe {
    Informe::nuevo(op, Resultado::Error)
        .con("motivo", "respuesta-invalida")
        .diagnostico(que(
            Mensaje::nuevo(&t::FORMA_INESPERADA).con("plataforma", p.titulo())
        ))
}
