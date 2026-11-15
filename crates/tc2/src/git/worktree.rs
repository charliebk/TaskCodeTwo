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

/// Quita `.taskcode/worktrees/<id>`. No existe → `sin-cambios`. Con cambios
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
    let wt = principal.join(CARPETA_WORKTREES).join(id);
    let Some(w) = worktrees(git, &principal, OP)?
        .into_iter()
        .find(|w| misma_ruta(&w.ruta, &wt))
    else {
        return Ok(Informe::nuevo(OP, Resultado::SinCambios).con("id", id));
    };
    let rechazo = |motivo: &str, que_hacer: &'static Texto| {
        Informe::nuevo(OP, Resultado::Rechazada)
            .con("id", id)
            .con("motivo", motivo)
            .diagnostico(que_hacer)
    };
    let status = exigir(git, &w.ruta, OP, Subcomando::Status, &["--porcelain", "-z"])?;
    if !status.is_empty() {
        return Ok(rechazo("cambios-sin-guardar", &txt::WORKTREE_CON_CAMBIOS));
    }
    // Con HEAD suelto no hay rama remota con la que comparar: sus commits se
    // perderian al quitar el worktree.
    let Some(rama) = &w.rama else {
        return Ok(rechazo("sin-rama", &txt::WORKTREE_HEAD_SUELTO));
    };
    let publicada = match commit_de(
        git,
        &principal,
        OP,
        &format!("refs/remotes/{REMOTO}/{rama}"),
    )? {
        Some(remota) => es_ancestro(git, &w.ruta, OP, "HEAD", &remota)?,
        None => false,
    };
    if !publicada {
        return Ok(rechazo("sin-publicar", &txt::WORKTREE_SIN_PUBLICAR));
    }
    let rel = worktree_relativo(id);
    exigir(git, &principal, OP, Subcomando::Worktree, &["remove", &rel])?;
    Ok(Informe::nuevo(OP, Resultado::Ok)
        .con("id", id)
        .con("rama", rama.as_str()))
}
