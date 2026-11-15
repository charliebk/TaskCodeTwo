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
use super::http::{FalloHttp, Respuesta};
use crate::contrato::{Informe, Resultado};

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

/// Traduce una respuesta que no es 2xx. `op` es la operacion, `rol` con que
/// identidad corria y `permiso` el que necesita (para el 403 sin plan).
pub fn traducir(p: Proveedor, op: &str, rol: Rol, permiso: &str, r: &Respuesta) -> Informe {
    let api = mensaje_api(r);
    let nombre = p.titulo();
    let cita = if api.is_empty() {
        String::new()
    } else {
        format!(" ({nombre}: «{api}»)")
    };
    let objeto = match p {
        Proveedor::GitHub => "el repo, el PR o el commit",
        Proveedor::GitLab => "el proyecto, el MR o el commit",
    };
    match r.estado {
        401 => Informe::nuevo(op, Resultado::Denegada)
            .con("motivo", "credenciales")
            .con("rol", rol.nombre())
            .diagnostico_sin_traducir(format!(
                "el token de {} no vale o ha caducado: define o renueva {}{cita}",
                rol.nombre(),
                rol.variable()
            )),
        _ if es_limite(r) => {
            let s = espera(r);
            Informe::nuevo(op, Resultado::Agotada)
                .con("motivo", "limite-peticiones")
                .con("reintenta-en", s.clone())
                .diagnostico_sin_traducir(format!(
                    "limite de peticiones de la API agotado: espera {s} s y relanza{cita}"
                ))
        }
        403 if p == Proveedor::GitHub && api.contains(AVISO_PLAN) => {
            Informe::nuevo(op, Resultado::Rechazada)
                .con("motivo", "plan")
                .diagnostico_sin_traducir(format!(
                    "el plan de GitHub no da esta funcion en un repo privado: haz el repo publico o usa GitHub Pro, Team o Enterprise{cita}"
                ))
        }
        403 => Informe::nuevo(op, Resultado::Denegada)
            .con("motivo", "permisos")
            .con("rol", rol.nombre())
            .diagnostico_sin_traducir(format!(
                "el token de {} no tiene permiso: necesita {permiso}{cita}",
                rol.nombre()
            )),
        404 => Informe::nuevo(op, Resultado::Denegada)
            .con("motivo", "no-existe-o-sin-acceso")
            .con("rol", rol.nombre())
            .diagnostico_sin_traducir(format!(
                "no existe o el token de {} no lo ve: revisa {objeto} y el alcance del token ({nombre} da 404 en un repo privado sin acceso){cita}",
                rol.nombre()
            )),
        // GitLab da 405 o 406 cuando el MR no se puede mergear.
        405 => no_mergeable(op, nombre, &cita),
        406 if p == Proveedor::GitLab => no_mergeable(op, nombre, &cita),
        409 => Informe::nuevo(op, Resultado::Rechazada)
            .con("motivo", "head-cambiado")
            .diagnostico_sin_traducir(format!(
                "la rama ha cambiado desde que se evaluaron las puertas: vuelve a lanzar estado-puertas{cita}"
            )),
        // El detalle del 422 de GitHub viene en `errors[].message`, no en
        // `message`.
        422 if r.cuerpo.contains("No commits between") => Informe::nuevo(op, Resultado::Rechazada)
            .con("motivo", "sin-commits")
            .diagnostico_sin_traducir(format!(
                "la rama no tiene commits nuevos respecto a la base: no hay nada que revisar{cita}"
            )),
        400 if p == Proveedor::GitLab => validacion(op, nombre, &cita),
        422 => validacion(op, nombre, &cita),
        e => Informe::nuevo(op, Resultado::Error)
            .con("motivo", "http")
            .con("estado", e.to_string())
            .diagnostico_sin_traducir(format!(
                "respuesta inesperada de {nombre} ({e}): reintenta mas tarde{cita}"
            )),
    }
}

fn no_mergeable(op: &str, nombre: &str, cita: &str) -> Informe {
    Informe::nuevo(op, Resultado::Rechazada)
        .con("motivo", "no-mergeable")
        .diagnostico_sin_traducir(format!(
            "{nombre} no deja mergear: faltan puertas o hay conflictos; lanza estado-puertas{cita}"
        ))
}

fn validacion(op: &str, nombre: &str, cita: &str) -> Informe {
    Informe::nuevo(op, Resultado::Error)
        .con("motivo", "validacion")
        .diagnostico_sin_traducir(format!("{nombre} rechazo los datos de la peticion{cita}"))
}

/// Traduce un fallo sin respuesta.
pub fn fallo(op: &str, f: &FalloHttp) -> Informe {
    match f {
        FalloHttp::Rechazada(m) => Informe::nuevo(op, Resultado::Rechazada)
            .con("motivo", "destino")
            .diagnostico_sin_traducir(format!(
                "llamada rechazada antes de salir a la red: {}",
                sanear(m)
            )),
        FalloHttp::Sinred(m) => Informe::nuevo(op, Resultado::Sinred).diagnostico_sin_traducir(format!(
            "no se pudo hablar con la API: comprueba la red y reintenta ({})",
            sanear(m)
        )),
        FalloHttp::Tls(m) => Informe::nuevo(op, Resultado::Error)
            .con("motivo", "tls")
            .diagnostico_sin_traducir(format!(
                "fallo de TLS: si la red usa un proxy con su propia CA, instalala en el almacen de certificados del sistema ({})",
                sanear(m)
            )),
        FalloHttp::RespuestaInvalida(m) => Informe::nuevo(op, Resultado::Error)
            .con("motivo", "respuesta-invalida")
            .diagnostico_sin_traducir(format!(
                "respuesta de la API ilegible o demasiado grande ({})",
                sanear(m)
            )),
        FalloHttp::Redireccion(e) => Informe::nuevo(op, Resultado::Error)
            .con("motivo", "redireccion")
            .diagnostico_sin_traducir(format!(
                "la API respondio con una redireccion ({e}) y no se siguen: revisa la URL de la API o el nombre del repo (renombrado o movido)"
            )),
        FalloHttp::Otro(m) => Informe::nuevo(op, Resultado::Error)
            .con("motivo", "cliente")
            .diagnostico_sin_traducir(format!("fallo del cliente HTTP ({})", sanear(m))),
    }
}

/// Respuesta 2xx con un cuerpo que no tiene la forma esperada.
pub fn respuesta_invalida(p: Proveedor, op: &str, que: &str) -> Informe {
    Informe::nuevo(op, Resultado::Error)
        .con("motivo", "respuesta-invalida")
        .diagnostico_sin_traducir(format!(
            "la respuesta de {} no tiene la forma esperada ({que})",
            p.titulo()
        ))
}
