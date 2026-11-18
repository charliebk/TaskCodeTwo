//! `quitar-worktree`: quita el worktree de una tarea cerrada; la rama se
//! conserva (ADR F1-3: las ramas no se borran).

use std::path::Path;

use super::rama::renombrar;
use super::{
    CARPETA_WORKTREES, Git, Paso, REMOTO, Subcomando, commit_de, es_ancestro, es_id, exigir,
    localizar_principal, misma_ruta, worktree_relativo, worktrees,
};
use crate::contrato::{Informe, Resultado};
use crate::i18n::Texto;
use crate::i18n::catalogo::git as txt;

/// Como esta el worktree de una tarea.
enum Revision {
    /// No hay worktree registrado en `.taskcodetwo/worktrees/<id>`.
    Ninguno,
    /// Limpio y con todo publicado en `origin/<rama>`: se puede quitar.
    Publicado {
        /// Rama del worktree.
        rama: String,
    },
    /// No se puede quitar sin perder algo: el motivo y que hacer.
    Pendiente(&'static str, &'static Texto),
}

/// Revisa el worktree de `id` del repo cuyo principal es `principal`: sin
/// cambios sin guardar, con rama (no HEAD suelto) y con HEAD contenido en
/// `origin/<rama>` tal como la conoce el clon.
fn revisar(git: &Git, principal: &Path, op: &str, id: &str) -> Paso<Revision> {
    let wt = principal.join(CARPETA_WORKTREES).join(id);
    let Some(w) = worktrees(git, principal, op)?
        .into_iter()
        .find(|w| misma_ruta(&w.ruta, &wt))
    else {
        return Ok(Revision::Ninguno);
    };
    let status = exigir(git, &w.ruta, op, Subcomando::Status, &["--porcelain", "-z"])?;
    if !status.is_empty() {
        return Ok(Revision::Pendiente(
            "cambios-sin-guardar",
            &txt::WORKTREE_CON_CAMBIOS,
        ));
    }
    // Con HEAD suelto no hay rama remota con la que comparar: sus commits se
    // perderian al quitar el worktree.
    let Some(rama) = w.rama else {
        return Ok(Revision::Pendiente("sin-rama", &txt::WORKTREE_HEAD_SUELTO));
    };
    let publicada = match commit_de(git, principal, op, &format!("refs/remotes/{REMOTO}/{rama}"))? {
        Some(remota) => es_ancestro(git, &w.ruta, op, "HEAD", &remota)?,
        None => false,
    };
    if !publicada {
        return Ok(Revision::Pendiente(
            "sin-publicar",
            &txt::WORKTREE_SIN_PUBLICAR,
        ));
    }
    Ok(Revision::Publicado { rama })
}

/// Lo que se perderia al dejar la tarea de `id` (F3-10, D4): `None` si no
/// hay worktree o esta limpio y publicado; si no, el motivo
/// (`cambios-sin-guardar`, `sin-rama` o `sin-publicar`). Es la misma
/// comprobacion con la que [`quitar_worktree`] decide si lo quita. `op`
/// nombra los fallos de git.
pub fn worktree_publicado(git: &Git, dir: &Path, op: &str, id: &str) -> Paso<Option<&'static str>> {
    let principal = localizar_principal(git, dir).map_err(|i| renombrar(i, op))?;
    Ok(match revisar(git, &principal, op, id)? {
        Revision::Ninguno | Revision::Publicado { .. } => None,
        Revision::Pendiente(motivo, _) => Some(motivo),
    })
}

/// Quita `.taskcodetwo/worktrees/<id>`. No existe → `sin-cambios`. Con cambios
/// sin guardar o commits sin publicar o HEAD suelto → `rechazada`, sin tocar nada.
pub fn quitar_worktree(git: &Git, dir: &Path, id: &str) -> Informe {
    quitar(git, dir, id).unwrap_or_else(|i| i)
}

fn quitar(git: &Git, dir: &Path, id: &str) -> Paso<Informe> {
    const OP: &str = "quitar-worktree";
    if !es_id(id) {
        return Err(Informe::nuevo(OP, Resultado::Uso).con("motivo", "id-invalido"));
    }
    let principal = localizar_principal(git, dir).map_err(|i| renombrar(i, OP))?;
    let rama = match revisar(git, &principal, OP, id)? {
        Revision::Ninguno => return Ok(Informe::nuevo(OP, Resultado::SinCambios).con("id", id)),
        Revision::Pendiente(motivo, que_hacer) => {
            return Ok(Informe::nuevo(OP, Resultado::Rechazada)
                .con("id", id)
                .con("motivo", motivo)
                .diagnostico(que_hacer));
        }
        Revision::Publicado { rama } => rama,
    };
    let rel = worktree_relativo(id);
    exigir(git, &principal, OP, Subcomando::Worktree, &["remove", &rel])?;
    Ok(Informe::nuevo(OP, Resultado::Ok)
        .con("id", id)
        .con("rama", rama.as_str()))
}
