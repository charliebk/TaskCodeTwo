//! Puerto `Git` (ADR F1-4, decision 1): lanza el binario `git` con un entorno
//! fijo en cada llamada y solo con los subcomandos de la lista permitida.
//!
//! La lista permitida es un `enum`: `reset`, `rebase`, `stash` o `clean` no se
//! pueden escribir (RN-05 por construccion, H9). Dentro de los permitidos, los
//! argumentos prohibidos (`--force`, refspec `+`, borrar ramas o tags...) se
//! rechazan antes de lanzar nada.

use std::cell::RefCell;
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

/// Subcomandos de `git` que la capa puede lanzar (lista permitida de F1-4 B2b,
/// reducida a los que usan las operaciones de F2-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Subcomando {
    /// `git add`
    Add,
    /// `git branch`
    Branch,
    /// `git commit`
    Commit,
    /// `git diff`
    Diff,
    /// `git fetch`
    Fetch,
    /// `git for-each-ref`
    ForEachRef,
    /// `git ls-remote`
    LsRemote,
    /// `git merge`
    Merge,
    /// `git merge-base`
    MergeBase,
    /// `git pull`
    Pull,
    /// `git push`
    Push,
    /// `git rev-parse`
    RevParse,
    /// `git show-ref`
    ShowRef,
    /// `git status`
    Status,
    /// `git tag`
    Tag,
    /// `git worktree`
    Worktree,
}

impl Subcomando {
    /// Nombre del subcomando en la linea de `git`.
    pub fn nombre(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Branch => "branch",
            Self::Commit => "commit",
            Self::Diff => "diff",
            Self::Fetch => "fetch",
            Self::ForEachRef => "for-each-ref",
            Self::LsRemote => "ls-remote",
            Self::Merge => "merge",
            Self::MergeBase => "merge-base",
            Self::Pull => "pull",
            Self::Push => "push",
            Self::RevParse => "rev-parse",
            Self::ShowRef => "show-ref",
            Self::Status => "status",
            Self::Tag => "tag",
            Self::Worktree => "worktree",
        }
    }
}

/// Lo que devolvio una llamada a `git` que llego a ejecutarse.
#[derive(Debug, Clone)]
pub struct SalidaGit {
    /// Codigo de salida (`-1` si el proceso murio por una senal).
    pub codigo: i32,
    /// Salida estandar, tal cual.
    pub stdout: String,
    /// Salida de error, tal cual.
    pub stderr: String,
}

impl SalidaGit {
    /// Si `git` termino con 0.
    pub fn exito(&self) -> bool {
        self.codigo == 0
    }

    /// Primera linea no vacia de stderr, para un diagnostico.
    pub fn primera_linea_error(&self) -> String {
        self.stderr
            .lines()
            .map(str::trim)
            .find(|l| !l.is_empty())
            .unwrap_or("sin mensaje")
            .to_owned()
    }
}

/// La llamada no llego a ejecutarse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorGit {
    /// Argumento prohibido dentro de un subcomando permitido.
    Prohibido(String),
    /// No se pudo lanzar el binario.
    Lanzar(String),
}

impl std::fmt::Display for ErrorGit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Prohibido(a) => write!(f, "argumento prohibido para la capa: {a}"),
            Self::Lanzar(e) => write!(f, "no se pudo lanzar git: {e}"),
        }
    }
}

/// Variables que, heredadas, harian que `git` mirase otro repo u otro indice.
const ENTORNO_QUITADO: [&str; 6] = [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_COMMON_DIR",
    "GIT_OBJECT_DIRECTORY",
    "GIT_NAMESPACE",
];

/// Entorno fijo de cada llamada (F1-4, decision 1).
const ENTORNO_FIJO: [(&str, &str); 5] = [
    ("LC_ALL", "C"),
    ("LANG", "C"),
    ("GIT_TERMINAL_PROMPT", "0"),
    ("GIT_PAGER", "cat"),
    ("GIT_EDITOR", "true"),
];

/// Puerto `Git`: un binario `git` y un registro de cada llamada.
#[derive(Debug)]
pub struct Git {
    binario: OsString,
    entorno_extra: Vec<(OsString, OsString)>,
    registro: RefCell<Vec<String>>,
}

impl Default for Git {
    fn default() -> Self {
        Self::nuevo()
    }
}

impl Git {
    /// El `git` del `PATH`, con el entorno del proceso mas el entorno fijo.
    pub fn nuevo() -> Self {
        Self {
            binario: OsString::from("git"),
            entorno_extra: Vec::new(),
            registro: RefCell::new(Vec::new()),
        }
    }

    /// Anade variables de entorno a cada llamada (los tests aislan asi la
    /// configuracion del usuario sin tocar el entorno del proceso).
    #[must_use]
    pub fn con_entorno(mut self, vars: Vec<(OsString, OsString)>) -> Self {
        self.entorno_extra = vars;
        self
    }

    /// Copia del registro: una linea `git <subcomando> <args>` por llamada
    /// ejecutada.
    pub fn registro(&self) -> Vec<String> {
        self.registro.borrow().clone()
    }

    /// Lanza `git <sub> <args>` en `dir`. Devuelve `Ok` aunque `git` termine
    /// con error (lo interpreta la operacion); `Err` si no llega a lanzarse.
    pub fn ejecutar(
        &self,
        dir: &Path,
        sub: Subcomando,
        args: &[&str],
    ) -> Result<SalidaGit, ErrorGit> {
        comprobar_argumentos(sub, args)?;
        let mut cmd = Command::new(&self.binario);
        cmd.current_dir(dir)
            .args(["-c", "color.ui=never", "-c", "core.quotepath=false"])
            .arg(sub.nombre())
            .args(args);
        for v in ENTORNO_QUITADO {
            cmd.env_remove(v);
        }
        for (k, v) in ENTORNO_FIJO {
            cmd.env(k, v);
        }
        for (k, v) in &self.entorno_extra {
            cmd.env(k, v);
        }
        self.registro
            .borrow_mut()
            .push(format!("git {} {}", sub.nombre(), args.join(" ")));
        let salida = cmd.output().map_err(|e| ErrorGit::Lanzar(e.to_string()))?;
        Ok(SalidaGit {
            codigo: salida.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&salida.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&salida.stderr).into_owned(),
        })
    }
}

/// Argumentos prohibidos dentro de los subcomandos permitidos (F1-4,
/// decision 3). Se mira cada argumento entero, no subcadenas (H9).
fn comprobar_argumentos(sub: Subcomando, args: &[&str]) -> Result<(), ErrorGit> {
    let prohibido = |a: &str| Err(ErrorGit::Prohibido(format!("{} {a}", sub.nombre())));
    for &a in args {
        if a.starts_with("--force") {
            return prohibido(a);
        }
        let malo = match sub {
            Subcomando::Push => {
                a == "-f"
                    || a == "-d"
                    || a == "--delete"
                    || a == "--mirror"
                    || a == "--prune"
                    || a == "--all"
                    || a.starts_with('+')
                    || a.starts_with(':')
            }
            Subcomando::Branch => matches!(
                a,
                "-f" | "-d" | "-D" | "-m" | "-M" | "-c" | "-C" | "--delete" | "--move" | "--copy"
            ),
            Subcomando::Worktree => a == "-f",
            Subcomando::Tag => matches!(a, "-d" | "-f" | "--delete"),
            Subcomando::Merge => a == "--abort",
            Subcomando::Commit => a == "--amend",
            _ => false,
        };
        if malo {
            return prohibido(a);
        }
    }
    if sub == Subcomando::Pull && !args.contains(&"--ff-only") {
        return prohibido("sin --ff-only");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rechaza_argumentos_prohibidos_sin_lanzar_git() {
        // Binario inexistente: si llegara a lanzarse, el error seria `Lanzar`.
        let git = Git {
            binario: OsString::from("no-existe-este-git"),
            entorno_extra: Vec::new(),
            registro: RefCell::new(Vec::new()),
        };
        let casos: [(Subcomando, &[&str]); 12] = [
            (Subcomando::Push, &["origin", "+r:r"]),
            (Subcomando::Push, &["--force-with-lease", "origin", "r"]),
            (Subcomando::Push, &["origin", ":r"]),
            (Subcomando::Push, &["--mirror", "origin"]),
            (Subcomando::Push, &["-f", "origin", "r"]),
            (Subcomando::Branch, &["-D", "r"]),
            (Subcomando::Branch, &["-f", "r", "HEAD"]),
            (Subcomando::Worktree, &["remove", "--force", "w"]),
            (Subcomando::Tag, &["-d", "v1"]),
            (Subcomando::Merge, &["--abort"]),
            (Subcomando::Pull, &["origin", "develop"]),
            (Subcomando::Commit, &["--amend", "-m", "x"]),
        ];
        for (sub, args) in casos {
            let r = git.ejecutar(Path::new("."), sub, args);
            assert!(
                matches!(r, Err(ErrorGit::Prohibido(_))),
                "{sub:?} {args:?}: {r:?}"
            );
        }
        assert!(git.registro().is_empty());
        // Un argumento permitido si intenta lanzar (y falla por el binario).
        let r = git.ejecutar(
            Path::new("."),
            Subcomando::Push,
            &["--porcelain", "origin", "a:b"],
        );
        assert!(matches!(r, Err(ErrorGit::Lanzar(_))));
    }
}
