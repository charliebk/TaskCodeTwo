//! Escenario «remoto bare servido por un ssh falso» de los tests de
//! integracion (A8-01, A8-16): personas, identidad y operaciones de la
//! biblioteca lanzadas por una persona.

//!
//! `origin` apunta a `ssh://git@github.com/o/r.git` (GitHub por el host) y
//! el transporte ssh se sustituye con `GIT_SSH_COMMAND`, que sirve el remoto
//! bare de la fixture; sin red, un «ssh» que responde «Connection refused».

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use tc2::contrato::Informe;
use tc2::git::Git;
use tc2::identidad::{Cache, Cuenta, Quien};
use tc2::plataforma::Proveedor;
use tc2::toma::Entorno;

use super::Fixture;
use super::git::git_en;
use super::ops::sin_catalogo;

/// `origin` de los clones: GitHub por el host, servido por el «ssh» falso.
pub const ORIGIN: &str = "ssh://git@github.com/o/r.git";
/// `personas.yml` con ana (1001) y beto (1002).
pub const PERSONAS: &str =
    "---\npersonas:\n  - {id: 1001, login: \"ana\"}\n  - {id: 1002, login: \"beto\"}\n";
/// `config.yml` de la base con el modo que `plan` congela.
pub const CONFIG_AUTOMATICO: &str = "---\nformato: 2\nmodo_flujo: automatico\n";

impl Fixture {
    /// Entorno de git con el «ssh» que sirve el remoto bare de la fixture
    /// (`upload-pack` para leer, `receive-pack` para publicar).
    pub fn entorno_ssh(&self) -> Vec<(OsString, OsString)> {
        let bare = self
            .raiz
            .join("remoto.git")
            .to_string_lossy()
            .replace('\\', "/");
        let mut e = self.entorno();
        e.push((
            "GIT_SSH_COMMAND".into(),
            format!(
                "sh -c 'case \"$2\" in *receive-pack*) exec git receive-pack \"$0\";; *) exec git upload-pack \"$0\";; esac' '{bare}'"
            )
            .into(),
        ));
        e.push(("GIT_SSH_VARIANT".into(), "simple".into()));
        e
    }

    /// Entorno de git cuyo «ssh» no llega al host, como sin red.
    pub fn entorno_sin_red(&self) -> Vec<(OsString, OsString)> {
        let mut e = self.entorno();
        e.push((
            "GIT_SSH_COMMAND".into(),
            "sh -c 'echo \"ssh: connect to host github.com port 22: Connection refused\" >&2; exit 255'"
                .into(),
        ));
        e.push(("GIT_SSH_VARIANT".into(), "simple".into()));
        e
    }

    /// `git <args>` en `dir` con el remoto de la fixture.
    pub fn git_ssh(&self, dir: &Path, args: &[&str]) -> String {
        self.git_ssh_con(dir, args, &[])
    }

    /// Como [`Fixture::git_ssh`], con variables de mas (fechas de un commit).
    pub fn git_ssh_con(&self, dir: &Path, args: &[&str], vars: &[(&str, &str)]) -> String {
        let mut e = self.entorno_ssh();
        e.extend(vars.iter().map(|(k, v)| ((*k).into(), (*v).into())));
        git_en(dir, &e, args)
    }

    /// Otro clon del remoto con `origin` en GitHub.
    pub fn clon_github(&self, bare: &Path, nombre: &str) -> PathBuf {
        let d = self.otro_clon(bare, nombre);
        self.crudo(&d, &["remote", "set-url", "origin", ORIGIN]);
        d
    }
}

/// La cuenta de GitHub `id`/`login`.
pub fn cuenta(id: u64, login: &str) -> Cuenta {
    Cuenta {
        proveedor: Proveedor::GitHub,
        id,
        login: login.to_owned(),
    }
}

/// Identifica el clon `dir` como la persona `id`/`login` (cache vigente).
pub fn identificar(dir: &Path, id: u64, login: &str) {
    Cache::en(&dir.join(".git"))
        .guardar(&cuenta(id, login), tc2::identidad::ahora())
        .unwrap();
}

/// `op` con la biblioteca, lanzada por la persona `cuenta` en un terminal
/// (`Quien::Persona`, que el binario solo da con un token real): la linea de
/// contrato.
pub fn como_persona(
    dir: &Path,
    entorno: Vec<(OsString, OsString)>,
    cuenta: Cuenta,
    op: impl FnOnce(&Entorno<'_>) -> Informe,
) -> String {
    let g = Git::nuevo().con_entorno(entorno);
    let q = move || -> Result<Quien, Informe> { Ok(Quien::Persona(cuenta.clone())) };
    let e = Entorno {
        dir,
        git: &g,
        quien: &q,
        catalogo: &sin_catalogo,
        ahora: tc2::identidad::ahora(),
    };
    op(&e).linea()
}
