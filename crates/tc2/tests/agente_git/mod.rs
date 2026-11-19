//! Para los E2E de F6-1 (TASK-047): el agente crea una rama con un commit y
//! la empuja con su propia identidad, por git y con el entorno de
//! `tc2::plataforma::entorno_git` (cabecera por `GIT_CONFIG_*`, autor del
//! agente). La configuracion de git del usuario queda fuera (ni su ayudante
//! de credenciales ni su firma): si el push sale, es por el token del
//! agente. Carpeta temporal bajo `std::env::temp_dir()`, borrada al soltar.

#![allow(dead_code)]
#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use tc2::contrato::Resultado;
use tc2::git::{self, Git};
use tc2::plataforma::{AutorGit, Proveedor, Secreto, entorno_git};

/// Clon temporal del agente.
pub struct Clon {
    raiz: PathBuf,
    pub dir: PathBuf,
    env: Vec<(OsString, OsString)>,
}

impl Drop for Clon {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.raiz);
    }
}

impl Clon {
    fn git(&self, dir: &Path, args: &[&str]) -> String {
        let o = Command::new("git")
            .current_dir(dir)
            .args(args)
            .envs(self.env.iter().cloned())
            .env("LC_ALL", "C")
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()
            .expect("lanzar git");
        assert!(
            o.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&o.stderr)
        );
        String::from_utf8_lossy(&o.stdout).trim().to_owned()
    }
}

/// Clona `base` de `url_repo` (con el token del agente: tambien prueba que
/// el agente lee por git), crea `rama` con un commit de `autor` y la empuja
/// con `tc2::git::push`. Devuelve el clon y el sha del commit.
pub fn publicar_rama(
    proveedor: Proveedor,
    url_repo: &str,
    prefijo: &str,
    token: &Secreto,
    autor: &AutorGit,
    (base, rama): (&str, &str),
) -> (Clon, String) {
    let raiz = std::env::temp_dir().join("tc2-tests").join(format!(
        "e2e-agente-{}-{}",
        std::process::id(),
        rama.replace('/', "-")
    ));
    let _ = std::fs::remove_dir_all(&raiz);
    std::fs::create_dir_all(&raiz).unwrap();
    let config = raiz.join("gitconfig");
    std::fs::write(&config, "[core]\n\tautocrlf = false\n").unwrap();
    let mut env: Vec<(OsString, OsString)> = vec![
        ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
        ("GIT_CONFIG_GLOBAL".into(), config.into_os_string()),
    ];
    env.extend(entorno_git(proveedor, prefijo, token, autor));
    let clon = Clon {
        dir: raiz.join("clon"),
        raiz,
        env,
    };
    clon.git(&clon.raiz, &["init", "-q", "-b", base, "clon"]);
    clon.git(&clon.dir, &["remote", "add", "origin", url_repo]);
    clon.git(&clon.dir, &["fetch", "-q", "origin", base]);
    clon.git(
        &clon.dir,
        &["checkout", "-q", "-b", rama, &format!("origin/{base}")],
    );
    let fichero = format!("e2e/{}.txt", rama.replace('/', "-"));
    std::fs::create_dir_all(clon.dir.join("e2e")).unwrap();
    std::fs::write(clon.dir.join(&fichero), "commit del agente (TASK-047)\n").unwrap();
    clon.git(&clon.dir, &["add", &fichero]);
    clon.git(
        &clon.dir,
        &["commit", "-q", "-m", "e2e: commit del agente (TASK-047)"],
    );
    let sha = clon.git(&clon.dir, &["rev-parse", "HEAD"]);
    let g = Git::nuevo().con_entorno(clon.env.clone());
    let i = git::push(&g, &clon.dir, &git::ESPERAS_PUSH);
    eprintln!("  {}", i.linea());
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    (clon, sha)
}
