//! Deteccion de la plataforma por el host de `origin` (F1-13, decision 7,
//! D8a): `github.com` es GitHub y `gitlab.com` es GitLab; otro host exige la
//! clave `plataforma` con la URL de la API, y si falta o contradice el host
//! se falla cerrado.
//!
//! Quien llama obtiene la URL con `git ls-remote --get-url origin` y la clave
//! de `.taskcode/config.yml` (F3-1); aqui llegan como parametros.

use super::http::Base;
use crate::contrato::{Informe, Resultado};
use crate::i18n::catalogo::plataforma as t;
use crate::i18n::{Mensaje, Texto};

/// Nombre de la operacion en los errores.
const OP: &str = "detectar-plataforma";

pub use super::Proveedor;

/// A donde habla el puerto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Destino {
    /// Plataforma.
    pub proveedor: Proveedor,
    /// `dueno/repo` (GitHub) o `grupo/.../proyecto` (GitLab).
    pub proyecto: String,
    /// Base de la API.
    pub api: Base,
}

/// Detecta la plataforma. `clave` es `(plataforma, url_api)` de la
/// configuracion, si esta.
pub fn detectar(url_origin: &str, clave: Option<(&str, &str)>) -> Result<Destino, Informe> {
    let (host, proyecto) = partir_url(url_origin).ok_or_else(|| {
        Informe::nuevo(OP, Resultado::Rechazada)
            .con("motivo", "origin")
            .diagnostico(&t::ORIGIN_NO_ENTENDIDO)
    })?;
    let por_host = match host.as_str() {
        "github.com" => Some(Proveedor::GitHub),
        "gitlab.com" => Some(Proveedor::GitLab),
        _ => None,
    };
    let declarado = match clave {
        None => None,
        Some((p, url)) => {
            let prov = match p {
                "github" => Proveedor::GitHub,
                "gitlab" => Proveedor::GitLab,
                _ => {
                    return Err(rechazo(&t::CLAVE_PLATAFORMA));
                }
            };
            let base = Base::nueva(url)
                .map_err(|m| rechazo(Mensaje::nuevo(&t::URL_API).con_mensaje("motivo", m)))?;
            Some((prov, base))
        }
    };
    let (proveedor, api) = match (por_host, declarado) {
        (Some(h), Some((d, _))) if h != d => {
            return Err(rechazo(
                Mensaje::nuevo(&t::PLATAFORMA_CONTRADICE)
                    .con("plataforma", d.nombre())
                    .con("host", host),
            ));
        }
        // Con un host conocido, la API es la oficial: una clave que apunte a
        // otra mandaria los tokens fuera (falla cerrado, decision 7).
        (Some(h), Some((_, base))) => {
            let oficial = api_oficial(h).map_err(rechazo)?;
            if base != oficial {
                return Err(rechazo(
                    Mensaje::nuevo(&t::API_NO_CAMBIABLE)
                        .con("host", host)
                        .con("api", oficial.texto()),
                ));
            }
            (h, base)
        }
        (Some(h), None) => (h, api_oficial(h).map_err(rechazo)?),
        (None, Some((d, base))) => (d, base),
        (None, None) => {
            return Err(rechazo(
                Mensaje::nuevo(&t::DECLARA_PLATAFORMA).con("host", host),
            ));
        }
    };
    let segmentos = proyecto.split('/').count();
    if proveedor == Proveedor::GitHub && segmentos != 2 {
        return Err(rechazo(&t::ORIGIN_GITHUB));
    }
    Ok(Destino {
        proveedor,
        proyecto,
        api,
    })
}

/// API publica de un host conocido.
fn api_oficial(p: Proveedor) -> Result<Base, &'static Texto> {
    match p {
        Proveedor::GitHub => Ok(Base::github()),
        Proveedor::GitLab => Base::nueva("https://gitlab.com/api/v4"),
    }
}

fn rechazo(m: impl Into<Mensaje>) -> Informe {
    Informe::nuevo(OP, Resultado::Rechazada)
        .con("motivo", "plataforma")
        .diagnostico(m)
}

/// `(host, proyecto)` de una URL https, ssh o scp, sin `.git` ni barras.
fn partir_url(url: &str) -> Option<(String, String)> {
    let url = url.trim();
    let (host, ruta) = if let Some(r) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("ssh://"))
    {
        let (autoridad, ruta) = r.split_once('/')?;
        let host = autoridad.rsplit('@').next()?;
        let host = host.split(':').next()?;
        (host, ruta)
    } else if !url.contains("://") {
        let (izq, ruta) = url.split_once(':')?;
        let host = izq.rsplit('@').next()?;
        (host, ruta)
    } else {
        return None;
    };
    let ruta = ruta.trim_matches('/');
    let ruta = ruta.strip_suffix(".git").unwrap_or(ruta);
    let valido = |s: &str| {
        !s.is_empty()
            && s != "."
            && s != ".."
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-._".contains(&b))
    };
    let host_ok = !host.is_empty()
        && host
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-');
    if !host_ok || !ruta.split('/').all(valido) || !ruta.contains('/') {
        return None;
    }
    Some((host.to_ascii_lowercase(), ruta.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn por_host_y_por_clave() {
        for url in [
            "https://github.com/charliebk/TaskCodeTwo.git",
            "git@github.com:charliebk/TaskCodeTwo.git",
            "ssh://git@github.com/charliebk/TaskCodeTwo",
        ] {
            let d = detectar(url, None).ok();
            assert_eq!(
                d.map(|d| (d.proveedor, d.proyecto, d.api.texto().to_owned())),
                Some((
                    Proveedor::GitHub,
                    "charliebk/TaskCodeTwo".to_owned(),
                    "https://api.github.com".to_owned()
                )),
                "{url}"
            );
        }
        let d = detectar("git@gitlab.com:g/sub/p.git", None).ok();
        assert_eq!(d.map(|d| d.proveedor), Some(Proveedor::GitLab));
        let d = detectar(
            "https://git.empresa.com/o/r.git",
            Some(("github", "https://git.empresa.com/api/v3")),
        )
        .ok();
        assert_eq!(
            d.map(|d| d.api.texto().to_owned()),
            Some("https://git.empresa.com/api/v3".to_owned())
        );
    }

    #[test]
    fn falla_cerrado() {
        for (url, clave) in [
            ("https://git.empresa.com/o/r.git", None),
            (
                "https://github.com/o/r",
                Some(("gitlab", "https://gitlab.com/api/v4")),
            ),
            (
                "https://git.empresa.com/o/r",
                Some(("bitbucket", "https://x")),
            ),
            ("https://git.empresa.com/o/r", Some(("github", "http://x"))),
            (
                "https://github.com/o/r",
                Some(("github", "https://api.evil.example")),
            ),
            (
                "git@gitlab.com:g/p.git",
                Some(("gitlab", "https://evil.example/api/v4")),
            ),
            ("https://github.com/o/r/s", None),
            ("https://github.com/o", None),
            ("file:///c/repo", None),
            ("https://github.com/o/..", None),
        ] {
            let r = detectar(url, clave);
            assert!(
                r.as_ref()
                    .is_err_and(|i| i.resultado == Resultado::Rechazada),
                "{url}: {r:?}"
            );
        }
    }
}
