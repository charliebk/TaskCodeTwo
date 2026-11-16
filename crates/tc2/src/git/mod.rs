//! Capa de operaciones Git deterministas (F2-2, ADR F1-4).
//!
//! Cada operacion es una funcion que recibe el puerto [`Git`] y la carpeta
//! desde la que se lanza, y devuelve un [`Informe`] con el contrato de
//! salida. Son publicas en la biblioteca para que las compongan los comandos
//! de fase (F2-3, F3-6, F7-3); en el binario solo se ven `guardar` y
//! `actualizar-rama` (D1a).
//!
//! El remoto es siempre `origin` y las bases, `origin/develop` u
//! `origin/main` (ADR F1-3).

mod diff;
pub(crate) mod estado;
mod guardar;
mod puerto;
pub(crate) mod rama;
pub(crate) mod remoto;
mod tag;
mod worktree;

pub use diff::diff_tarea;
pub use estado::Cambio;
pub use guardar::guardar;
pub use puerto::{ErrorGit, Git, PREFIJO_REFS_PROPIAS, SalidaCruda, SalidaGit, Subcomando};
pub use rama::{
    comprobar_id, crear_rama_y_worktree, ids_con_rama, localizar_principal, reanudar, validar_base,
};
pub use remoto::{
    ESPERAS_PUSH, FalloRemoto, actualizar_principal, actualizar_rama, clasificar_fallo_remoto,
    fetch, push,
};
pub use tag::tag_anotado;
pub use worktree::quitar_worktree;

use std::path::{Component, Path, PathBuf};

use crate::contrato::{Informe, Resultado};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::git as txt;

/// Nombre del remoto (ADR F1-3).
pub const REMOTO: &str = "origin";

/// Carpeta de los worktrees de tarea, relativa al worktree principal.
pub const CARPETA_WORKTREES: &str = crate::huella::CARPETA_WORKTREES;

/// Resultado interno: el `Err` ya es el informe que se devuelve.
pub(crate) type Paso<T> = Result<T, Informe>;

/// Lanza `git` y convierte el fallo al lanzar en un informe `error`.
pub(crate) fn correr(
    git: &Git,
    dir: &Path,
    op: &str,
    sub: Subcomando,
    args: &[&str],
) -> Paso<SalidaGit> {
    git.ejecutar(dir, sub, args).map_err(|e| {
        Informe::nuevo(op, Resultado::Error)
            .con("git", sub.nombre())
            .diagnostico(e.mensaje())
    })
}

/// Diagnostico de un subcomando de git que termino con error.
pub(crate) fn git_fallo(sub: Subcomando, s: &SalidaGit) -> Mensaje {
    Mensaje::nuevo(&txt::GIT_FALLO)
        .con("subcomando", sub.nombre())
        .con("error", s.primera_linea_error())
}

/// Como [`correr`], pero un codigo distinto de 0 tambien es `error`.
pub(crate) fn exigir(
    git: &Git,
    dir: &Path,
    op: &str,
    sub: Subcomando,
    args: &[&str],
) -> Paso<String> {
    let s = correr(git, dir, op, sub, args)?;
    if s.exito() {
        Ok(s.stdout)
    } else {
        Err(Informe::nuevo(op, Resultado::Error)
            .con("git", sub.nombre())
            .diagnostico(git_fallo(sub, &s)))
    }
}

/// Commit al que apunta `referencia`, o `None` si no existe.
pub(crate) fn commit_de(git: &Git, dir: &Path, op: &str, referencia: &str) -> Paso<Option<String>> {
    let r = format!("{referencia}^{{commit}}");
    let s = correr(
        git,
        dir,
        op,
        Subcomando::RevParse,
        &["-q", "--verify", "--end-of-options", &r],
    )?;
    Ok(s.exito().then(|| s.stdout.trim().to_owned()))
}

/// Si `a` esta contenido en `b`.
pub(crate) fn es_ancestro(git: &Git, dir: &Path, op: &str, a: &str, b: &str) -> Paso<bool> {
    let s = correr(
        git,
        dir,
        op,
        Subcomando::MergeBase,
        &["--is-ancestor", a, b],
    )?;
    match s.codigo {
        0 => Ok(true),
        1 => Ok(false),
        _ => {
            Err(Informe::nuevo(op, Resultado::Error)
                .diagnostico(git_fallo(Subcomando::MergeBase, &s)))
        }
    }
}

/// Rama local en la que esta `dir`, o `None` si HEAD esta suelto.
pub(crate) fn rama_actual(git: &Git, dir: &Path, op: &str) -> Paso<Option<String>> {
    let s = exigir(
        git,
        dir,
        op,
        Subcomando::RevParse,
        &["--symbolic-full-name", "HEAD"],
    )?;
    Ok(s.trim().strip_prefix("refs/heads/").map(str::to_owned))
}

/// `develop` y `main` no se tocan desde una operacion de tarea.
pub(crate) fn es_protegida(rama: &str) -> bool {
    rama == "develop" || rama == "main"
}

/// Base de una rama de tarea segun su prefijo (ADR F1-3): `hotfix/` sale de
/// `main`; el resto, de `develop`.
pub fn base_de_rama(rama: &str) -> &'static str {
    if rama.starts_with("hotfix/") {
        "origin/main"
    } else {
        "origin/develop"
    }
}

/// Un worktree de `git worktree list --porcelain`.
#[derive(Debug, Clone)]
pub(crate) struct Worktree {
    pub ruta: PathBuf,
    pub rama: Option<String>,
}

pub(crate) fn worktrees(git: &Git, dir: &Path, op: &str) -> Paso<Vec<Worktree>> {
    let s = exigir(git, dir, op, Subcomando::Worktree, &["list", "--porcelain"])?;
    let mut v: Vec<Worktree> = Vec::new();
    for l in s.lines() {
        if let Some(r) = l.strip_prefix("worktree ") {
            v.push(Worktree {
                ruta: PathBuf::from(r),
                rama: None,
            });
        } else if let (Some(r), Some(w)) = (l.strip_prefix("branch refs/heads/"), v.last_mut()) {
            w.rama = Some(r.to_owned());
        }
    }
    Ok(v)
}

/// Si dos rutas son la misma carpeta (en Windows `git` escribe `C:/...` y el
/// sistema puede dar nombres cortos 8.3: se comparan canonicas si existen).
pub(crate) fn misma_ruta(a: &Path, b: &Path) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(x), Ok(y)) => x == y,
        _ => normalizar(a) == normalizar(b),
    }
}

fn normalizar(p: &Path) -> Vec<String> {
    p.components()
        .filter(|c| !matches!(c, Component::CurDir))
        .map(|c| {
            let t = c.as_os_str().to_string_lossy().replace('\\', "/");
            if cfg!(windows) { t.to_lowercase() } else { t }
        })
        .collect()
}

/// Entradas de `git status --porcelain -z`: (estado XY, ruta). Las de
/// renombrado traen un segundo campo (la ruta de origen) que se salta.
pub(crate) fn entradas_status(salida: &str) -> Vec<(String, String)> {
    let mut v = Vec::new();
    let mut campos = salida.split('\0').filter(|c| !c.is_empty());
    while let Some(c) = campos.next() {
        let xy = c.get(..2).unwrap_or(c).to_owned();
        let ruta = c.get(3..).unwrap_or("").to_owned();
        if xy.starts_with('R') || xy.starts_with('C') {
            campos.next();
        }
        v.push((xy, ruta));
    }
    v
}

/// `TASK-NNN` con al menos tres cifras.
pub(crate) fn es_id(id: &str) -> bool {
    id.strip_prefix("TASK-")
        .is_some_and(|n| n.len() >= 3 && n.bytes().all(|b| b.is_ascii_digit()))
}

/// Nombre de la rama de una tarea (ADR F1-3): `<tipo>/<task-nnn>-<slug>`.
pub fn nombre_de_rama(id: &str, tipo: &str, slug: &str) -> String {
    format!("{tipo}/{}-{slug}", id.to_lowercase())
}

/// Ruta relativa al principal, con `/`, para la linea de contrato.
pub fn worktree_relativo(id: &str) -> String {
    format!("{CARPETA_WORKTREES}/{id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_z_con_renombrado_y_espacios() {
        let s = " M a b.txt\0R  nuevo.txt\0viejo.txt\0?? d/\0";
        let e = entradas_status(s);
        assert_eq!(
            e,
            vec![
                (" M".to_owned(), "a b.txt".to_owned()),
                ("R ".to_owned(), "nuevo.txt".to_owned()),
                ("??".to_owned(), "d/".to_owned()),
            ]
        );
    }

    #[test]
    fn ids_ramas_y_bases() {
        assert!(es_id("TASK-007") && es_id("TASK-1234"));
        assert!(!es_id("TASK-7") && !es_id("task-007") && !es_id("TASK-00a"));
        assert_eq!(
            nombre_de_rama("TASK-007", "feature", "login"),
            "feature/task-007-login"
        );
        assert_eq!(base_de_rama("hotfix/task-009-x"), "origin/main");
        assert_eq!(base_de_rama("feature/task-007-login"), "origin/develop");
    }
}
