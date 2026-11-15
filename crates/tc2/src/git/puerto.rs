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
/// reducida a los que usan las operaciones de F2-2 y la rama de estado de
/// F2-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Subcomando {
    /// `git add`
    Add,
    /// `git branch`
    Branch,
    /// `git commit`
    Commit,
    /// `git commit-tree` (crea un objeto commit; no mueve ninguna rama)
    CommitTree,
    /// `git diff`
    Diff,
    /// `git fetch`
    Fetch,
    /// `git for-each-ref`
    ForEachRef,
    /// `git hash-object`
    HashObject,
    /// `git ls-remote`
    LsRemote,
    /// `git ls-tree`
    LsTree,
    /// `git merge`
    Merge,
    /// `git merge-base`
    MergeBase,
    /// `git pull`
    Pull,
    /// `git push`
    Push,
    /// `git read-tree` (solo con indice temporal)
    ReadTree,
    /// `git rev-parse`
    RevParse,
    /// `git show`
    Show,
    /// `git show-ref`
    ShowRef,
    /// `git status`
    Status,
    /// `git tag`
    Tag,
    /// `git update-index` (solo con indice temporal)
    UpdateIndex,
    /// `git update-ref` (solo bajo `refs/taskcode/`)
    UpdateRef,
    /// `git worktree`
    Worktree,
    /// `git write-tree` (solo con indice temporal)
    WriteTree,
}

impl Subcomando {
    /// Nombre del subcomando en la linea de `git`.
    pub fn nombre(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Branch => "branch",
            Self::Commit => "commit",
            Self::CommitTree => "commit-tree",
            Self::Diff => "diff",
            Self::Fetch => "fetch",
            Self::ForEachRef => "for-each-ref",
            Self::HashObject => "hash-object",
            Self::LsRemote => "ls-remote",
            Self::LsTree => "ls-tree",
            Self::Merge => "merge",
            Self::MergeBase => "merge-base",
            Self::Pull => "pull",
            Self::Push => "push",
            Self::ReadTree => "read-tree",
            Self::RevParse => "rev-parse",
            Self::Show => "show",
            Self::ShowRef => "show-ref",
            Self::Status => "status",
            Self::Tag => "tag",
            Self::UpdateIndex => "update-index",
            Self::UpdateRef => "update-ref",
            Self::Worktree => "worktree",
            Self::WriteTree => "write-tree",
        }
    }

    /// Los que trabajan sobre un indice: solo se lanzan con un indice
    /// temporal ([`Git::ejecutar_con_indice`]), nunca sobre el del repo.
    pub fn exige_indice_temporal(self) -> bool {
        matches!(self, Self::ReadTree | Self::UpdateIndex | Self::WriteTree)
    }
}

/// Prefijo de las unicas refs que puede escribir `update-ref` (pendientes y
/// rechazadas de la rama de estado, F2-3). Nunca `refs/heads/*` (F1-4 D3).
pub const PREFIJO_REFS_PROPIAS: &str = "refs/taskcode/";

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
        if sub.exige_indice_temporal() {
            return Err(ErrorGit::Prohibido(format!(
                "{} sin indice temporal",
                sub.nombre()
            )));
        }
        self.lanzar(dir, sub, args, None)
    }

    /// Como [`Git::ejecutar`], pero con `GIT_INDEX_FILE=indice`: solo para
    /// `read-tree`, `update-index` y `write-tree`, que asi nunca tocan el
    /// indice del repo. `indice` tiene que ser una ruta absoluta.
    pub fn ejecutar_con_indice(
        &self,
        dir: &Path,
        sub: Subcomando,
        args: &[&str],
        indice: &Path,
    ) -> Result<SalidaGit, ErrorGit> {
        if !sub.exige_indice_temporal() {
            return Err(ErrorGit::Prohibido(format!(
                "{} con indice temporal",
                sub.nombre()
            )));
        }
        if !indice.is_absolute() {
            return Err(ErrorGit::Prohibido(format!(
                "{} con indice relativo",
                sub.nombre()
            )));
        }
        self.lanzar(dir, sub, args, Some(indice))
    }

    fn lanzar(
        &self,
        dir: &Path,
        sub: Subcomando,
        args: &[&str],
        indice: Option<&Path>,
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
        if let Some(i) = indice {
            cmd.env("GIT_INDEX_FILE", i);
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
/// decision 3). Se mira cada argumento entero, no subcadenas (H9). No se
/// miran el texto de un mensaje (`-m <texto>` en `commit` y `tag`) ni las
/// rutas tras `--` en los subcomandos que las aceptan (`add`, `commit`,
/// `status`, `diff`): ahi una palabra como `--force` es dato, no opcion.
fn comprobar_argumentos(sub: Subcomando, args: &[&str]) -> Result<(), ErrorGit> {
    let prohibido = |a: &str| Err(ErrorGit::Prohibido(format!("{} {a}", sub.nombre())));
    let con_rutas = matches!(
        sub,
        Subcomando::Add | Subcomando::Commit | Subcomando::Status | Subcomando::Diff
    );
    let con_mensaje = matches!(sub, Subcomando::Commit | Subcomando::Tag);
    let mut iter = args.iter().copied();
    while let Some(a) = iter.next() {
        if a == "--" && con_rutas {
            break;
        }
        if a == "-m" && con_mensaje {
            iter.next();
            continue;
        }
        if es_prohibido(sub, a) {
            return prohibido(a);
        }
    }
    if sub == Subcomando::Pull && !args.contains(&"--ff-only") {
        return prohibido("sin --ff-only");
    }
    if sub == Subcomando::UpdateRef && !forma_de_update_ref(args) {
        return prohibido(&args.join(" "));
    }
    Ok(())
}

/// `update-ref` solo con `<ref> <nuevo> [<viejo>]` o `-d <ref> <viejo>`, y la
/// ref bajo [`PREFIJO_REFS_PROPIAS`]: nunca una rama, nunca `--stdin`.
fn forma_de_update_ref(args: &[&str]) -> bool {
    let propia = |r: &str| {
        r.strip_prefix(PREFIJO_REFS_PROPIAS)
            .is_some_and(|resto| !resto.is_empty() && !resto.contains(".."))
    };
    let valor = |v: &str| !v.is_empty() && !v.starts_with('-');
    match args {
        ["-d", r, viejo] => propia(r) && valor(viejo),
        [r, nuevo] => propia(r) && valor(nuevo),
        [r, nuevo, viejo] => propia(r) && valor(nuevo) && valor(viejo),
        _ => false,
    }
}

fn es_prohibido(sub: Subcomando, a: &str) -> bool {
    if a.starts_with("--force") {
        return true;
    }
    match sub {
        Subcomando::Push => {
            matches!(
                a,
                "-f" | "-d" | "--delete" | "--mirror" | "--prune" | "--all"
            ) || a.starts_with('+')
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
        Subcomando::ReadTree => matches!(a, "-u" | "-m" | "--reset") || a.starts_with("--prefix"),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Puerto con un binario inexistente: si una llamada llegara a lanzarse,
    /// el error seria `Lanzar`, no `Prohibido`.
    fn git_sin_binario() -> Git {
        Git {
            binario: OsString::from("no-existe-este-git"),
            entorno_extra: Vec::new(),
            registro: RefCell::new(Vec::new()),
        }
    }

    #[test]
    fn rechaza_cada_argumento_prohibido_sin_lanzar_git() {
        let git = git_sin_binario();
        // Una fila por regla de `es_prohibido` y de la decision 3 de F1-4,
        // formas cortas y largas.
        let casos: &[(Subcomando, &[&str])] = &[
            (Subcomando::Push, &["--force", "origin", "r"]),
            (Subcomando::Push, &["--force-with-lease", "origin", "r"]),
            (Subcomando::Push, &["-f", "origin", "r"]),
            (Subcomando::Push, &["-d", "origin", "r"]),
            (Subcomando::Push, &["--delete", "origin", "r"]),
            (Subcomando::Push, &["--mirror", "origin"]),
            (Subcomando::Push, &["--prune", "origin"]),
            (Subcomando::Push, &["--all", "origin"]),
            (Subcomando::Push, &["origin", "+r:r"]),
            (Subcomando::Push, &["origin", ":r"]),
            (Subcomando::Branch, &["-f", "r", "HEAD"]),
            (Subcomando::Branch, &["--force", "r", "HEAD"]),
            (Subcomando::Branch, &["-d", "r"]),
            (Subcomando::Branch, &["-D", "r"]),
            (Subcomando::Branch, &["--delete", "r"]),
            (Subcomando::Branch, &["-m", "r", "s"]),
            (Subcomando::Branch, &["-M", "r", "s"]),
            (Subcomando::Branch, &["--move", "r", "s"]),
            (Subcomando::Branch, &["-c", "r", "s"]),
            (Subcomando::Branch, &["-C", "r", "s"]),
            (Subcomando::Branch, &["--copy", "r", "s"]),
            (Subcomando::Worktree, &["remove", "-f", "w"]),
            (Subcomando::Worktree, &["remove", "--force", "w"]),
            (Subcomando::Tag, &["-d", "v1"]),
            (Subcomando::Tag, &["--delete", "v1"]),
            (Subcomando::Tag, &["-f", "v1"]),
            (Subcomando::Tag, &["--force", "v1"]),
            (Subcomando::Merge, &["--abort"]),
            (Subcomando::Commit, &["--amend", "-m", "x"]),
            (Subcomando::Pull, &["origin", "develop"]),
            (Subcomando::UpdateRef, &["refs/heads/develop", "abc"]),
            (Subcomando::UpdateRef, &["refs/remotes/origin/x", "abc"]),
            (Subcomando::UpdateRef, &["-d", "refs/heads/develop", "abc"]),
            (Subcomando::UpdateRef, &["--stdin"]),
            (Subcomando::UpdateRef, &["-d", "refs/taskcode/p/1"]),
            (Subcomando::UpdateRef, &["refs/taskcode/", "abc"]),
            (Subcomando::UpdateRef, &["refs/taskcode/../heads/x", "abc"]),
            (
                Subcomando::UpdateRef,
                &["--no-deref", "refs/taskcode/p/1", "abc"],
            ),
            (
                Subcomando::UpdateRef,
                &["refs/taskcode/p/1", "abc", "def", "x"],
            ),
            (Subcomando::ReadTree, &["abc"]),
            (Subcomando::UpdateIndex, &["--add", "a"]),
            (Subcomando::WriteTree, &[]),
        ];
        for (sub, args) in casos {
            let r = git.ejecutar(Path::new("."), *sub, args);
            assert!(
                matches!(r, Err(ErrorGit::Prohibido(_))),
                "{sub:?} {args:?}: {r:?}"
            );
        }
        assert!(git.registro().is_empty());
    }

    #[test]
    fn mensajes_y_rutas_no_son_opciones() {
        let git = git_sin_binario();
        let permitidos: &[(Subcomando, &[&str])] = &[
            (Subcomando::Push, &["--porcelain", "origin", "a:b"]),
            (
                Subcomando::Commit,
                &["-q", "-m", "--force no se usa", "--only", "--", "a.txt"],
            ),
            (Subcomando::Add, &["--", "--force-x.txt"]),
            (Subcomando::Status, &["--porcelain", "--", "--amend"]),
            (Subcomando::Tag, &["-a", "-m", "-d", "--", "v1", "abc"]),
            (Subcomando::Pull, &["-q", "--ff-only", "origin", "develop"]),
        ];
        for (sub, args) in permitidos {
            let r = git.ejecutar(Path::new("."), *sub, args);
            assert!(
                matches!(r, Err(ErrorGit::Lanzar(_))),
                "{sub:?} {args:?}: {r:?}"
            );
        }
        // Con indice temporal solo los tres de indice, con ruta absoluta y
        // sin las formas que tocarian el worktree.
        let indice = std::env::temp_dir().join("indice-de-prueba");
        let r = git.ejecutar_con_indice(Path::new("."), Subcomando::Commit, &[], &indice);
        assert!(matches!(r, Err(ErrorGit::Prohibido(_))), "{r:?}");
        let r = git.ejecutar_con_indice(
            Path::new("."),
            Subcomando::WriteTree,
            &[],
            Path::new("relativo"),
        );
        assert!(matches!(r, Err(ErrorGit::Prohibido(_))), "{r:?}");
        for malo in ["-u", "-m", "--reset", "--prefix=x/"] {
            let r = git.ejecutar_con_indice(
                Path::new("."),
                Subcomando::ReadTree,
                &[malo, "abc"],
                &indice,
            );
            assert!(matches!(r, Err(ErrorGit::Prohibido(_))), "{malo}: {r:?}");
        }
        let r = git.ejecutar_con_indice(
            Path::new("."),
            Subcomando::UpdateIndex,
            &["--force-remove", "a"],
            &indice,
        );
        assert!(matches!(r, Err(ErrorGit::Prohibido(_))), "{r:?}");
        let r = git.ejecutar_con_indice(Path::new("."), Subcomando::ReadTree, &["abc"], &indice);
        assert!(matches!(r, Err(ErrorGit::Lanzar(_))), "{r:?}");
        for ok in [
            &["refs/taskcode/pendientes/000001", "abc"][..],
            &["refs/taskcode/pendientes/000001", "abc", "0000"][..],
            &["-d", "refs/taskcode/pendientes/000001", "abc"][..],
        ] {
            let r = git.ejecutar(Path::new("."), Subcomando::UpdateRef, ok);
            assert!(matches!(r, Err(ErrorGit::Lanzar(_))), "{ok:?}: {r:?}");
        }
        // Tras `--` en push no hay rutas: se sigue mirando.
        let r = git.ejecutar(Path::new("."), Subcomando::Push, &["origin", "--", "+r:r"]);
        assert!(matches!(r, Err(ErrorGit::Prohibido(_))));
    }
}
