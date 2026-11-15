//! De HTTP a la linea de contrato, con el arreglo para la persona (F2-6:
//! «errores dirigidos a la persona»). Es el unico sitio que conoce los
//! codigos HTTP; el adaptador de GitLab (F2-9) lo reutiliza.
//!
//! El diagnostico dice que hacer. El mensaje de la API es dato no confiable
//! (F1-8, Z2): se recorta y se quitan los caracteres de control antes de
//! citarlo. Nunca lleva el token ni cabeceras.

use super::credenciales::Rol;
use super::http::{FalloHttp, Respuesta};
use crate::contrato::{Informe, Resultado};

/// Aviso de GitHub cuando el plan no da rulesets ni proteccion de rama.
pub const AVISO_PLAN: &str = "Upgrade to GitHub Pro";

/// Mensaje de la API (`message`), saneado: sin controles y como mucho 200
/// caracteres.
pub fn mensaje_api(r: &Respuesta) -> String {
    let m = serde_json::from_str::<serde_json::Value>(&r.cuerpo)
        .ok()
        .and_then(|v| v.get("message").and_then(|m| m.as_str()).map(str::to_owned))
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
    // dice el mensaje.
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
pub fn traducir(op: &str, rol: Rol, permiso: &str, r: &Respuesta) -> Informe {
    let api = mensaje_api(r);
    let cita = if api.is_empty() {
        String::new()
    } else {
        format!(" (GitHub: «{api}»)")
    };
    match r.estado {
        401 => Informe::nuevo(op, Resultado::Denegada)
            .con("motivo", "credenciales")
            .con("rol", rol.nombre())
            .diagnostico(format!(
                "el token de {} no vale o ha caducado: define o renueva {}{cita}",
                rol.nombre(),
                rol.variable()
            )),
        _ if es_limite(r) => {
            let s = espera(r);
            Informe::nuevo(op, Resultado::Agotada)
                .con("motivo", "limite-peticiones")
                .con("reintenta-en", s.clone())
                .diagnostico(format!(
                    "limite de peticiones de la API agotado: espera {s} s y relanza{cita}"
                ))
        }
        403 if api.contains(AVISO_PLAN) => Informe::nuevo(op, Resultado::Rechazada)
            .con("motivo", "plan")
            .diagnostico(format!(
                "el plan de GitHub no da esta funcion en un repo privado: haz el repo publico o usa GitHub Pro, Team o Enterprise{cita}"
            )),
        403 => Informe::nuevo(op, Resultado::Denegada)
            .con("motivo", "permisos")
            .con("rol", rol.nombre())
            .diagnostico(format!(
                "el token de {} no tiene permiso: necesita {permiso}{cita}",
                rol.nombre()
            )),
        404 => Informe::nuevo(op, Resultado::Denegada)
            .con("motivo", "no-existe-o-sin-acceso")
            .con("rol", rol.nombre())
            .diagnostico(format!(
                "no existe o el token de {} no lo ve: revisa el repo, el PR o el commit y el alcance del token (GitHub da 404 en un repo privado sin acceso){cita}",
                rol.nombre()
            )),
        405 => Informe::nuevo(op, Resultado::Rechazada)
            .con("motivo", "no-mergeable")
            .diagnostico(format!(
                "GitHub no deja mergear: faltan puertas o hay conflictos; lanza estado-puertas{cita}"
            )),
        409 => Informe::nuevo(op, Resultado::Rechazada)
            .con("motivo", "head-cambiado")
            .diagnostico(format!(
                "la rama ha cambiado desde que se evaluaron las puertas: vuelve a lanzar estado-puertas{cita}"
            )),
        // El detalle del 422 viene en `errors[].message`, no en `message`.
        422 if r.cuerpo.contains("No commits between") => Informe::nuevo(op, Resultado::Rechazada)
            .con("motivo", "sin-commits")
            .diagnostico(format!(
                "la rama no tiene commits nuevos respecto a la base: no hay nada que revisar{cita}"
            )),
        422 => Informe::nuevo(op, Resultado::Error)
            .con("motivo", "validacion")
            .diagnostico(format!("GitHub rechazo los datos de la peticion{cita}")),
        e => Informe::nuevo(op, Resultado::Error)
            .con("motivo", "http")
            .con("estado", e.to_string())
            .diagnostico(format!(
                "respuesta inesperada de GitHub ({e}): reintenta mas tarde{cita}"
            )),
    }
}

/// Traduce un fallo sin respuesta.
pub fn fallo(op: &str, f: &FalloHttp) -> Informe {
    match f {
        FalloHttp::Rechazada(m) => Informe::nuevo(op, Resultado::Rechazada)
            .con("motivo", "destino")
            .diagnostico(format!(
                "llamada rechazada antes de salir a la red: {}",
                sanear(m)
            )),
        FalloHttp::Sinred(m) => Informe::nuevo(op, Resultado::Sinred).diagnostico(format!(
            "no se pudo hablar con la API: comprueba la red y reintenta ({})",
            sanear(m)
        )),
        FalloHttp::Tls(m) => Informe::nuevo(op, Resultado::Error)
            .con("motivo", "tls")
            .diagnostico(format!(
                "fallo de TLS: si la red usa un proxy con su propia CA, instalala en el almacen de certificados del sistema ({})",
                sanear(m)
            )),
        FalloHttp::RespuestaInvalida(m) => Informe::nuevo(op, Resultado::Error)
            .con("motivo", "respuesta-invalida")
            .diagnostico(format!(
                "respuesta de la API ilegible o demasiado grande ({})",
                sanear(m)
            )),
        FalloHttp::Redireccion(e) => Informe::nuevo(op, Resultado::Error)
            .con("motivo", "redireccion")
            .diagnostico(format!(
                "la API respondio con una redireccion ({e}) y no se siguen: revisa la URL de la API o el nombre del repo (renombrado o movido)"
            )),
        FalloHttp::Otro(m) => Informe::nuevo(op, Resultado::Error)
            .con("motivo", "cliente")
            .diagnostico(format!("fallo del cliente HTTP ({})", sanear(m))),
    }
}

/// Respuesta 2xx con un cuerpo que no tiene la forma esperada.
pub fn respuesta_invalida(op: &str, que: &str) -> Informe {
    Informe::nuevo(op, Resultado::Error)
        .con("motivo", "respuesta-invalida")
        .diagnostico(format!(
            "la respuesta de GitHub no tiene la forma esperada ({que})"
        ))
}
