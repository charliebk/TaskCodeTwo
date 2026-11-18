//! `diff-tarea`: resumen en una linea de lo que cambia la tarea frente a su
//! base (ADR F1-4). El diff completo, a peticion, es de `review` (F6-2).
//! [`ficheros_de_rama`]: los ficheros que toca una rama, para el aviso de
//! solape de `start` (F3-11).

use std::collections::BTreeSet;
use std::path::Path;

use super::{Git, Paso, Subcomando, base_de_rama, commit_de, exigir, rama_actual};
use crate::contrato::{Informe, Resultado, corto};
use crate::i18n::catalogo::git as txt;

/// Ficheros y lineas que cambian desde el `merge-base` con la base de la rama
/// (incluye lo no guardado del worktree; no los ficheros sin seguimiento).
pub fn diff_tarea(git: &Git, dir: &Path) -> Informe {
    diff_paso(git, dir).unwrap_or_else(|i| i)
}

/// Los ficheros que cambia `rama` desde su `merge-base` con `base` (F3-11):
/// un solo `git diff --name-only -z --no-renames <base>...<rama> --`, sin
/// red. Con `--no-renames` un renombrado da los dos nombres (el viejo y el
/// nuevo), como S08 de F1-15; con `-z` y `core.quotepath=false`, las rutas
/// tal cual, sin comillas. Un `diff` que falla (sin `merge-base`, la base o
/// la rama no existen) es el `Err`.
pub fn ficheros_de_rama(
    git: &Git,
    dir: &Path,
    op: &str,
    base: &str,
    rama: &str,
) -> Paso<BTreeSet<String>> {
    let rango = format!("{base}...{rama}");
    let s = exigir(
        git,
        dir,
        op,
        Subcomando::Diff,
        &["--name-only", "-z", "--no-renames", &rango, "--"],
    )?;
    Ok(s.split('\0')
        .filter(|r| !r.is_empty())
        .map(str::to_owned)
        .collect())
}

fn diff_paso(git: &Git, dir: &Path) -> Paso<Informe> {
    const OP: &str = "diff-tarea";
    let Some(rama) = rama_actual(git, dir, OP)? else {
        return Ok(Informe::nuevo(OP, Resultado::Rechazada)
            .con("motivo", "sin-rama")
            .diagnostico(&txt::LANZALO_EN_EL_WORKTREE));
    };
    let base = base_de_rama(&rama);
    if commit_de(git, dir, OP, base)?.is_none() {
        return Ok(Informe::nuevo(OP, Resultado::Rechazada)
            .con("base", base)
            .con("motivo", "base-no-existe"));
    }
    let desde = exigir(git, dir, OP, Subcomando::MergeBase, &["HEAD", base])?;
    let desde = desde.trim();
    let numstat = exigir(git, dir, OP, Subcomando::Diff, &["--numstat", "-z", desde])?;
    let (mut ficheros, mut mas, mut menos) = (0usize, 0u64, 0u64);
    // Con -z: `<mas>\t<menos>\t<ruta>\0`; en binarios, `-\t-\t`; en un
    // renombrado la ruta va vacia y siguen dos campos con las rutas.
    let mut campos = numstat.split('\0');
    while let Some(c) = campos.next() {
        if c.is_empty() {
            continue;
        }
        let mut partes = c.splitn(3, '\t');
        let (a, b, ruta) = (partes.next(), partes.next(), partes.next());
        if ruta == Some("") {
            campos.next();
            campos.next();
        }
        ficheros += 1;
        mas += a.and_then(|x| x.parse().ok()).unwrap_or(0);
        menos += b.and_then(|x| x.parse().ok()).unwrap_or(0);
    }
    Ok(Informe::nuevo(OP, Resultado::Ok)
        .con("base", base)
        .con("desde", corto(desde))
        .con("ficheros", ficheros.to_string())
        .con("mas", mas.to_string())
        .con("menos", menos.to_string()))
}
