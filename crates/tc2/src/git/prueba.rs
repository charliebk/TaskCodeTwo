//! Soporte comun de los tests unitarios que lanzan git de sistema (A8-20):
//! la configuracion aislada de la del usuario y el git con el que se
//! preparan los escenarios. Los tests de integracion tienen el suyo en
//! `tests/comun` (otro crate).

use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

pub(crate) type R<T> = Result<T, Box<dyn std::error::Error>>;

/// Configuracion de git aislada de la del usuario.
pub(crate) const CONFIG: &str = "[user]\n\tname = Prueba\n\temail = prueba@example.com\n[core]\n\tautocrlf = false\n[init]\n\tdefaultBranch = develop\n[commit]\n\tgpgsign = false\n[tag]\n\tgpgsign = false\n";

/// Entorno que hace que git lea solo `raiz/gitconfig`.
pub(crate) fn entorno(raiz: &Path) -> Vec<(OsString, OsString)> {
    vec![
        ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
        (
            "GIT_CONFIG_GLOBAL".into(),
            raiz.join("gitconfig").into_os_string(),
        ),
    ]
}

/// Escribe [`CONFIG`] en `raiz/gitconfig`, la que lee [`entorno`].
pub(crate) fn aislar(raiz: &Path) -> R<()> {
    std::fs::write(raiz.join("gitconfig"), CONFIG)?;
    Ok(())
}

/// git de sistema para preparar el escenario; devuelve stdout recortado.
pub(crate) fn sh(raiz: &Path, dir: &Path, args: &[&str]) -> R<String> {
    let s = Command::new("git")
        .current_dir(dir)
        .args(args)
        .envs(entorno(raiz))
        .env("LC_ALL", "C")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()?;
    if !s.status.success() {
        return Err(format!("git {args:?}: {}", String::from_utf8_lossy(&s.stderr)).into());
    }
    Ok(String::from_utf8_lossy(&s.stdout).trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `sh` lee la configuracion que deja `aislar`, no la del usuario.
    #[test]
    fn sh_lee_la_configuracion_aislada() -> R<()> {
        let raiz = std::env::temp_dir()
            .join("tc2-tests")
            .join(format!("git-prueba-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&raiz);
        std::fs::create_dir_all(&raiz)?;
        aislar(&raiz)?;
        let email = sh(&raiz, &raiz, &["config", "user.email"]);
        let _ = std::fs::remove_dir_all(&raiz);
        assert_eq!(email?, "prueba@example.com");
        Ok(())
    }
}
