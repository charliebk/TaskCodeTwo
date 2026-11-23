//! Git de sistema para preparar escenarios (A8-20): una sola forma de lanzar
//! `git` fuera de la capa, siempre aislado de la configuracion del usuario.

use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

/// `git <args>` en `dir` con `entorno`, que tiene que traer su
/// `GIT_CONFIG_GLOBAL` (si no, la configuracion global de quien corre los
/// tests entraria en el escenario). Fija `GIT_CONFIG_NOSYSTEM`, `LC_ALL=C` y
/// `GIT_TERMINAL_PROMPT=0`; exige exito y devuelve stdout.
pub fn git_en(dir: &Path, entorno: &[(OsString, OsString)], args: &[&str]) -> String {
    assert!(
        entorno.iter().any(|(k, _)| k == "GIT_CONFIG_GLOBAL"),
        "git_en sin GIT_CONFIG_GLOBAL: git {args:?} en {}",
        dir.display()
    );
    let o = Command::new("git")
        .current_dir(dir)
        .args(args)
        .envs(entorno.iter().cloned())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("LC_ALL", "C")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("lanzar git");
    assert!(
        o.status.success(),
        "git {args:?} en {}: {}",
        dir.display(),
        String::from_utf8_lossy(&o.stderr)
    );
    String::from_utf8_lossy(&o.stdout).into_owned()
}
