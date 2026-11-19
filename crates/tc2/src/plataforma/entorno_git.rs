//! Entorno de `git` para que el agente empuje y firme como autor con su
//! propia identidad (F6-1; ADR F1-6, decision 1: `push` y `commit-estado`
//! corren con el agente).
//!
//! El token va a `git` **por el entorno, nunca en la linea de ordenes**: con
//! `GIT_CONFIG_COUNT`, `GIT_CONFIG_KEY_<n>` y `GIT_CONFIG_VALUE_<n>` (config
//! de ambito «command», Git >= 2.31) se define
//! `http.<url>.extraheader = Authorization: Basic <base64(usuario:token)>`,
//! limitado a la URL del remoto, y, si se da, el autor y committer con
//! `GIT_AUTHOR_*`/`GIT_COMMITTER_*`. Ni el registro del puerto `Git` ni los
//! argumentos de los hijos de git (`git-remote-https`) lo ven.
//!
//! Quien llama compone el puerto con [`crate::git::Git::con_entorno_extra`]
//! o [`crate::git::Git::derivado`]: la capa `git` no depende de
//! `plataforma`. `review` (F6-2) lo pide al adaptador con
//! [`super::Plataforma::entorno_push`], que lo arma con [`entorno_push`].

use std::ffi::OsString;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;

use super::Paso;
use super::Proveedor;
use super::conexion::Conexion;
use super::credenciales::{Rol, Secreto};
use crate::contrato::{Informe, Resultado};
use crate::i18n::catalogo::plataforma as t;

/// Nombre de la operacion del puerto (F6-2).
pub const ENTORNO_PUSH: &str = "entorno-push";

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
/// con `token` y, si se da, firme como `autor` (`review` solo empuja: sin
/// autor, los commits siguen con la identidad local). La cabecera es un
/// lote de una entrada de `GIT_CONFIG_*` numerado desde 0:
/// [`crate::git::Git::con_entorno_extra`] lo renumera tras las entradas que
/// ya tengan el puerto y el proceso, sin pisarlas. Pasadas a otro `Command`
/// a mano, ocupan la entrada 0.
pub fn entorno_git(
    proveedor: Proveedor,
    url_remoto: &str,
    token: &Secreto,
    autor: Option<&AutorGit>,
) -> Vec<(OsString, OsString)> {
    let basica = STANDARD.encode(format!("{}:{}", usuario(proveedor), token.exponer()));
    let par = |k: &str, v: String| (OsString::from(k), OsString::from(v));
    let mut v = vec![
        par("GIT_CONFIG_COUNT", "1".to_owned()),
        par("GIT_CONFIG_KEY_0", format!("http.{url_remoto}.extraheader")),
        par(
            "GIT_CONFIG_VALUE_0",
            format!("Authorization: Basic {basica}"),
        ),
    ];
    if let Some(a) = autor {
        v.extend([
            par("GIT_AUTHOR_NAME", a.nombre.clone()),
            par("GIT_AUTHOR_EMAIL", a.correo.clone()),
            par("GIT_COMMITTER_NAME", a.nombre.clone()),
            par("GIT_COMMITTER_EMAIL", a.correo.clone()),
        ]);
    }
    v
}

/// `entorno-push` de los adaptadores (F6-2): el entorno de [`entorno_git`]
/// con el token del agente de `con`, sin autor, limitado a `url_remoto`.
/// Solo `https://`: por ssh la cabecera no aplica (empujaria la identidad
/// local) y por `http://` el token viajaria en claro. Un fallo al obtener el
/// token sale como el de cualquier operacion del agente.
pub(crate) fn entorno_push(con: &Conexion, url_remoto: &str) -> Paso<Vec<(OsString, OsString)>> {
    let valida = url_remoto
        .strip_prefix("https://")
        .is_some_and(|r| !r.is_empty() && !r.chars().any(|c| c.is_control() || c == ' '));
    if !valida {
        return Err(Informe::nuevo(ENTORNO_PUSH, Resultado::Rechazada)
            .con("motivo", "remoto-no-https")
            .diagnostico(&t::REMOTO_NO_HTTPS));
    }
    let token = con.token(ENTORNO_PUSH, Rol::Agente)?;
    Ok(entorno_git(con.proveedor(), url_remoto, &token, None))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plataforma::credenciales::CredencialesFijas;
    use crate::plataforma::http::Base;
    use crate::plataforma::{GitHub, GitLab, Personas, Plataforma};

    fn valor<'a>(v: &'a [(OsString, OsString)], k: &str) -> Option<&'a str> {
        v.iter().find(|(x, _)| x == k).and_then(|(_, y)| y.to_str())
    }

    /// F6-2: los dos adaptadores dan la cabecera del agente limitada a la
    /// URL del remoto, sin autor; un remoto que no es `https://` se rechaza
    /// sin pedir el token, y sin token del agente sale `motivo=identidad`.
    #[test]
    fn entorno_push_de_los_adaptadores() {
        let fijas = || {
            Box::new(CredencialesFijas::nuevas().con(Rol::Agente, Secreto::nuevo("tok")))
                as Box<dyn crate::plataforma::Credenciales>
        };
        let gh = GitHub::nuevo(Base::github(), "o/r", fijas(), Personas::default());
        let base_gl = Base::nueva("https://gitlab.example/api/v4").map_err(|_| ());
        let gl = base_gl
            .and_then(|b| GitLab::nuevo(b, "g/p", fijas(), Personas::default()).map_err(|_| ()));
        assert!(gh.is_ok() && gl.is_ok());
        let (Ok(gh), Ok(gl)) = (gh, gl) else {
            return;
        };
        let casos: [(&dyn Plataforma, &str); 2] =
            [(&gh, "x-access-token:tok"), (&gl, "oauth2:tok")];
        for (p, basica) in casos {
            let url = "https://forja.example/o/r.git";
            let v = p.entorno_push(url).unwrap_or_default();
            assert_eq!(
                valor(&v, "GIT_CONFIG_KEY_0"),
                Some("http.https://forja.example/o/r.git.extraheader")
            );
            let esperado = format!("Authorization: Basic {}", STANDARD.encode(basica));
            assert_eq!(valor(&v, "GIT_CONFIG_VALUE_0"), Some(esperado.as_str()));
            assert_eq!(valor(&v, "GIT_AUTHOR_NAME"), None);
            for mala in [
                "git@forja.example:o/r.git",
                "http://forja.example/o/r.git",
                "/tmp/r.git",
                "https://",
            ] {
                let linea = p.entorno_push(mala).err().map(|i| i.linea());
                assert_eq!(
                    linea.as_deref(),
                    Some("entorno-push rechazada motivo=remoto-no-https"),
                    "{mala}"
                );
            }
        }
        let sin = GitHub::nuevo(
            Base::github(),
            "o/r",
            Box::new(CredencialesFijas::nuevas()),
            Personas::default(),
        );
        let i = sin
            .ok()
            .and_then(|p| p.entorno_push("https://github.com/o/r.git").err());
        assert_eq!(
            i.and_then(|i| i.valor("motivo").map(str::to_owned))
                .as_deref(),
            Some("identidad")
        );
    }
}
