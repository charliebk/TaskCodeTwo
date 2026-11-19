//! Entorno de `git` para que el agente empuje y firme como autor con su
//! propia identidad (F6-1; ADR F1-6, decision 1: `push` y `commit-estado`
//! corren con el agente).
//!
//! El token va a `git` **por el entorno, nunca en la linea de ordenes**: con
//! `GIT_CONFIG_COUNT`, `GIT_CONFIG_KEY_<n>` y `GIT_CONFIG_VALUE_<n>` (config
//! de ambito «command», Git >= 2.31) se define
//! `http.<url>.extraheader = Authorization: Basic <base64(usuario:token)>`,
//! limitado a la URL del remoto, y el autor y committer con
//! `GIT_AUTHOR_*`/`GIT_COMMITTER_*`. Ni el registro del puerto `Git` ni los
//! argumentos de los hijos de git (`git-remote-https`) lo ven.
//!
//! Quien llama compone el puerto con [`crate::git::Git::con_entorno_extra`]:
//! la capa `git` no depende de `plataforma`. Conectarlo a `review` es de
//! F6-2.

use std::ffi::OsString;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;

use super::Proveedor;
use super::credenciales::Secreto;

/// Autor (y committer) de los commits del agente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutorGit {
    /// Nombre.
    pub nombre: String,
    /// Correo.
    pub correo: String,
}

impl AutorGit {
    /// La cuenta `<slug>[bot]` de una GitHub App. Con el id de esa cuenta,
    /// el correo `<id>+<slug>[bot]@users.noreply.github.com`, que GitHub
    /// asocia a la cuenta; sin el, `<slug>[bot]@users.noreply.github.com`.
    pub fn bot_github(slug: &str, id_cuenta: Option<u64>) -> Self {
        let login = format!("{slug}[bot]");
        let correo = match id_cuenta {
            Some(id) => format!("{id}+{login}@users.noreply.github.com"),
            None => format!("{login}@users.noreply.github.com"),
        };
        Self {
            nombre: login,
            correo,
        }
    }
}

/// Usuario de la autenticacion basica de git con un token: el de los tokens
/// de instalacion de GitHub y el de OAuth de GitLab (GitLab admite cualquier
/// usuario no vacio con un token de proyecto o de cuenta de servicio).
fn usuario(p: Proveedor) -> &'static str {
    match p {
        Proveedor::GitHub => "x-access-token",
        Proveedor::GitLab => "oauth2",
    }
}

/// Variables para que `git` hable con `url_remoto` (p. ej.
/// `https://github.com/`; la cabecera solo va a las URL que empiezan asi)
/// con `token` y firme como `autor`. Ocupan las entradas 0 de
/// `GIT_CONFIG_*`: no se combinan con otras definidas igual.
pub fn entorno_git(
    proveedor: Proveedor,
    url_remoto: &str,
    token: &Secreto,
    autor: &AutorGit,
) -> Vec<(OsString, OsString)> {
    let basica = STANDARD.encode(format!("{}:{}", usuario(proveedor), token.exponer()));
    let par = |k: &str, v: String| (OsString::from(k), OsString::from(v));
    vec![
        par("GIT_CONFIG_COUNT", "1".to_owned()),
        par("GIT_CONFIG_KEY_0", format!("http.{url_remoto}.extraheader")),
        par(
            "GIT_CONFIG_VALUE_0",
            format!("Authorization: Basic {basica}"),
        ),
        par("GIT_AUTHOR_NAME", autor.nombre.clone()),
        par("GIT_AUTHOR_EMAIL", autor.correo.clone()),
        par("GIT_COMMITTER_NAME", autor.nombre.clone()),
        par("GIT_COMMITTER_EMAIL", autor.correo.clone()),
    ]
}
