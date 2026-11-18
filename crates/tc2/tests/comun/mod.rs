//! Fixtures de los tests de la capa Git: repos reales en una carpeta temporal
//! (bajo `std::env::temp_dir()`, nunca en la raiz de `C:\`), un remoto bare
//! local y la configuracion del usuario aislada. Cada fixture borra su
//! carpeta al terminar.

#![allow(dead_code)] // cada fichero de tests usa una parte
#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

pub mod http_mock;
pub mod ops;

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use tc2::contrato::{Informe, Resultado};
use tc2::git::Git;

static CONTADOR: AtomicUsize = AtomicUsize::new(0);

const CONFIG: &str = "[user]\n\tname = Prueba\n\temail = prueba@example.com\n[core]\n\tautocrlf = false\n[init]\n\tdefaultBranch = develop\n[commit]\n\tgpgsign = false\n[tag]\n\tgpgsign = false\n";

/// Una carpeta temporal con su configuracion de git aislada.
pub struct Fixture {
    pub raiz: PathBuf,
    config: PathBuf,
}

impl Fixture {
    pub fn nueva(nombre: &str) -> Self {
        let n = CONTADOR.fetch_add(1, Ordering::SeqCst);
        let raiz = std::env::temp_dir()
            .join("tc2-tests")
            .join(format!("{nombre}-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&raiz);
        std::fs::create_dir_all(&raiz).expect("crear la carpeta de la fixture");
        let config = raiz.join("gitconfig");
        std::fs::write(&config, CONFIG).expect("escribir gitconfig");
        Self { raiz, config }
    }

    /// Variables que aislan a git de la configuracion del usuario.
    pub fn entorno(&self) -> Vec<(OsString, OsString)> {
        vec![
            ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
            (
                "GIT_CONFIG_GLOBAL".into(),
                self.config.clone().into_os_string(),
            ),
        ]
    }

    /// Puerto Git de la capa con la configuracion aislada.
    pub fn git(&self) -> Git {
        Git::nuevo().con_entorno(self.entorno())
    }

    /// `git` directo para preparar la fixture (no pasa por la capa).
    pub fn crudo(&self, dir: &Path, args: &[&str]) -> String {
        let o = Command::new("git")
            .current_dir(dir)
            .args(args)
            .envs(self.entorno())
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

    /// Remoto bare (`develop` y `main` en el mismo commit, sin force) y un
    /// clon que hace de worktree principal.
    pub fn remoto_y_clon(&self) -> (PathBuf, PathBuf) {
        let bare = self.raiz.join("remoto.git");
        let clon = self.raiz.join("clon");
        self.crudo(
            &self.raiz,
            &["init", "-q", "--bare", "-b", "develop", "remoto.git"],
        );
        self.crudo(&bare, &["config", "receive.denyNonFastForwards", "true"]);
        self.crudo(&self.raiz, &["init", "-q", "-b", "develop", "clon"]);
        escribir(&clon, "README.md", "linea 1\n");
        self.crudo(&clon, &["add", "README.md"]);
        self.crudo(&clon, &["commit", "-q", "-m", "inicial"]);
        self.crudo(&clon, &["branch", "main"]);
        self.crudo(&clon, &["remote", "add", "origin", bare.to_str().unwrap()]);
        self.crudo(&clon, &["push", "-q", "origin", "develop", "main"]);
        self.crudo(&clon, &["fetch", "-q", "origin"]);
        (bare, clon)
    }

    /// Otro clon del mismo remoto (otra persona).
    pub fn otro_clon(&self, bare: &Path, nombre: &str) -> PathBuf {
        self.crudo(&self.raiz, &["clone", "-q", bare.to_str().unwrap(), nombre]);
        self.raiz.join(nombre)
    }

    /// Huella del estado: refs del remoto y del clon, worktrees y status de
    /// cada worktree. Si no cambia, la operacion no toco nada.
    pub fn huella(&self, bare: &Path, clon: &Path) -> String {
        let mut h = self.crudo(bare, &["for-each-ref", "--format=%(refname) %(objectname)"]);
        h += &self.crudo(clon, &["for-each-ref", "--format=%(refname) %(objectname)"]);
        let wts = self.crudo(clon, &["worktree", "list", "--porcelain"]);
        h += &wts;
        for l in wts.lines() {
            if let Some(r) = l.strip_prefix("worktree ") {
                h += &self.crudo(Path::new(r), &["status", "--porcelain"]);
            }
        }
        h
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.raiz);
    }
}

pub fn escribir(dir: &Path, rel: &str, contenido: &str) {
    let p = dir.join(rel);
    if let Some(padre) = p.parent() {
        std::fs::create_dir_all(padre).unwrap();
    }
    std::fs::write(p, contenido).unwrap();
}

/// Comprueba el contrato de salida de F1-4 (decision 4) en una salida.
pub fn contrato(i: &Informe) {
    for s in [i.salida(false), i.salida(true)] {
        assert!(
            s.stdout.lines().count() <= 5,
            "mas de 5 lineas: {}",
            s.stdout
        );
        assert!(!s.stdout.contains('\u{1b}') && !s.stderr.contains('\u{1b}'));
        assert!(
            s.stderr.lines().count() <= 1,
            "stderr de mas de una linea: {}",
            s.stderr
        );
        if i.resultado.es_exito() {
            assert!(s.stderr.is_empty(), "stderr en un exito: {}", s.stderr);
        }
    }
    let linea = i.linea();
    let mut t = linea.split(' ');
    assert_eq!(t.next(), Some(i.comando.as_str()));
    let r = t.next().unwrap();
    assert!(
        [
            "ok",
            "sin-cambios",
            "rechazada",
            "sinred",
            "agotada",
            "conflicto",
            "esperando",
            "denegada",
            "uso",
            "error"
        ]
        .contains(&r),
        "resultado fuera del contrato: {linea}"
    );
    for kv in t {
        let (k, v) = kv
            .split_once('=')
            .unwrap_or_else(|| panic!("sin '=': {linea}"));
        assert!(
            !k.is_empty() && k.bytes().all(|b| b.is_ascii_lowercase() || b == b'-'),
            "clave fuera del contrato: {linea}"
        );
        if ["commit", "desde"].contains(&k) {
            assert!(
                v.len() == 12 && v.bytes().all(|b| b.is_ascii_hexdigit()),
                "hash no de 12: {linea}"
            );
        }
    }
}

/// Lanza la operacion dos veces: la segunda no cambia el estado y da
/// `sin-cambios` o la misma linea (idempotencia de F1-4, PoC B1).
pub fn dos_veces(f: &Fixture, bare: &Path, clon: &Path, op: impl Fn() -> Informe) -> Informe {
    let primera = op();
    contrato(&primera);
    let antes = f.huella(bare, clon);
    let segunda = op();
    contrato(&segunda);
    assert_eq!(
        antes,
        f.huella(bare, clon),
        "la 2.a vez cambio el estado: {}",
        segunda.linea()
    );
    assert!(
        segunda.resultado == Resultado::SinCambios || segunda.linea() == primera.linea(),
        "2.a vez: {} (1.a: {})",
        segunda.linea(),
        primera.linea()
    );
    primera
}

/// Detector independiente por lista permitida (F1-4 B2b): cada linea del
/// registro es un subcomando permitido sin argumentos prohibidos.
pub fn registro_permitido(registro: &[String]) {
    const PERMITIDOS: [&str; 26] = [
        "add",
        "branch",
        "cat-file",
        "commit",
        "commit-tree",
        "diff",
        "fetch",
        "for-each-ref",
        "hash-object",
        "log",
        "ls-remote",
        "ls-tree",
        "merge",
        "merge-base",
        "pull",
        "push",
        "read-tree",
        "rev-parse",
        "show",
        "show-ref",
        "status",
        "tag",
        "update-index",
        "update-ref",
        "worktree",
        "write-tree",
    ];
    assert!(!registro.is_empty());
    for l in registro {
        let mut t = l.split(' ');
        assert_eq!(t.next(), Some("git"), "{l}");
        let sub = t.next().unwrap_or("");
        assert!(
            PERMITIDOS.contains(&sub),
            "subcomando fuera de la lista: {l}"
        );
        let args: Vec<&str> = t.collect();
        for a in &args {
            assert!(!a.starts_with("--force"), "force: {l}");
            let malo = match sub {
                "push" => {
                    a.starts_with('+')
                        || a.starts_with(':')
                        || ["-f", "--mirror", "--delete", "-d"].contains(a)
                }
                "branch" => ["-f", "-d", "-D", "-m", "-M"].contains(a),
                "tag" => ["-d", "-f"].contains(a),
                "merge" => *a == "--abort",
                "commit" => *a == "--amend",
                "add" => ["-A", "--all", "."].contains(a),
                "read-tree" => ["-u", "-m", "--reset"].contains(a),
                "update-ref" => {
                    *a == "--stdin"
                        || (a.starts_with("refs/") && !a.starts_with("refs/taskcodetwo/"))
                }
                _ => false,
            };
            assert!(!malo, "argumento prohibido: {l}");
        }
        if sub == "pull" {
            assert!(args.contains(&"--ff-only"), "pull sin --ff-only: {l}");
        }
    }
}
