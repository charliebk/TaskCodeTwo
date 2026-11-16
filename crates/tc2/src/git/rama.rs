//! Operaciones de rama y worktree de tarea: `localizar-principal`,
//! `validar-base`, `comprobar-id`, `crear-rama-y-worktree` y `reanudar`.

use std::path::{Path, PathBuf};

use super::{
    CARPETA_WORKTREES, Git, Paso, REMOTO, Subcomando, commit_de, correr, es_id, exigir, misma_ruta,
    nombre_de_rama, worktree_relativo, worktrees,
};
use crate::contrato::{Informe, Resultado, corto};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::git as txt;

/// Worktree principal: la primera entrada de `git worktree list` (ADR F1-3).
/// Funciona lanzado desde el principal o desde cualquier worktree de tarea.
pub fn localizar_principal(git: &Git, dir: &Path) -> Result<PathBuf, Informe> {
    const OP: &str = "localizar-principal";
    let s = correr(git, dir, OP, Subcomando::Worktree, &["list", "--porcelain"])?;
    let primera = s
        .stdout
        .lines()
        .next()
        .and_then(|l| l.strip_prefix("worktree "))
        .filter(|_| s.exito());
    match primera {
        Some(r) => Ok(PathBuf::from(r)),
        None => Err(Informe::nuevo(OP, Resultado::Rechazada)
            .con("motivo", "no-es-repo")
            .diagnostico(&txt::NO_ES_REPO)),
    }
}

/// Comprueba que `base` existe y apunta a un commit.
pub fn validar_base(git: &Git, dir: &Path, base: &str) -> Informe {
    const OP: &str = "validar-base";
    if base.is_empty() || base.starts_with('-') {
        return Informe::nuevo(OP, Resultado::Uso).con("motivo", "base-invalida");
    }
    match commit_de(git, dir, OP, base) {
        Ok(Some(c)) => Informe::nuevo(OP, Resultado::Ok)
            .con("base", base)
            .con("commit", corto(&c)),
        Ok(None) => Informe::nuevo(OP, Resultado::Rechazada)
            .con("base", base)
            .con("motivo", "no-existe")
            .diagnostico(Mensaje::nuevo(&txt::BASE_NO_EXISTE).con("base", base)),
        Err(i) => i,
    }
}

/// Mira si el ID ya tiene rama, en local o en el remoto (patron
/// `*/task-nnn-*`). Ocupado → `rechazada motivo=ocupada`.
pub fn comprobar_id(git: &Git, dir: &Path, id: &str) -> Informe {
    const OP: &str = "comprobar-id";
    if !es_id(id) {
        return Informe::nuevo(OP, Resultado::Uso).con("motivo", "id-invalido");
    }
    match ramas_del_id(git, dir, OP, id) {
        Ok(r) if r.is_empty() => Informe::nuevo(OP, Resultado::Ok).con("id", id),
        Ok(r) => Informe::nuevo(OP, Resultado::Rechazada)
            .con("id", id)
            .con("motivo", "ocupada")
            .con_lista("ramas", r)
            .diagnostico(Mensaje::nuevo(&txt::ID_CON_RAMA).con("id", id)),
        Err(i) => i,
    }
}

fn ramas_del_id(git: &Git, dir: &Path, op: &str, id: &str) -> Paso<Vec<String>> {
    let patron = format!("*/{}-*", id.to_lowercase());
    let remoto = correr(
        git,
        dir,
        op,
        Subcomando::LsRemote,
        &["--heads", REMOTO, &patron],
    )?;
    if !remoto.exito() {
        return Err(Informe::nuevo(op, Resultado::Sinred)
            .con("remoto", REMOTO)
            .diagnostico(super::remoto::sin_red_remoto(op)));
    }
    let local = exigir(
        git,
        dir,
        op,
        Subcomando::Branch,
        &["--list", "--format=%(refname:short)", &patron],
    )?;
    let mut ramas: Vec<String> = remoto
        .stdout
        .lines()
        .filter_map(|l| l.split('\t').nth(1))
        .filter_map(|r| r.strip_prefix("refs/heads/"))
        .chain(local.lines().map(str::trim).filter(|l| !l.is_empty()))
        .map(str::to_owned)
        .collect();
    ramas.sort();
    ramas.dedup();
    Ok(ramas)
}

/// Valida `id`, `tipo` y `slug` y devuelve el nombre de rama.
pub(crate) fn rama_valida(op: &str, id: &str, tipo: &str, slug: &str) -> Paso<String> {
    let tipo_ok = !tipo.is_empty() && tipo.bytes().all(|b| b.is_ascii_lowercase());
    let slug_ok = !slug.is_empty()
        && !slug.starts_with('-')
        && slug
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    if !es_id(id) {
        Err(Informe::nuevo(op, Resultado::Uso).con("motivo", "id-invalido"))
    } else if !tipo_ok {
        Err(Informe::nuevo(op, Resultado::Uso).con("motivo", "tipo-invalido"))
    } else if !slug_ok {
        Err(Informe::nuevo(op, Resultado::Uso).con("motivo", "slug-invalido"))
    } else {
        Ok(nombre_de_rama(id, tipo, slug))
    }
}

fn ruta_texto(op: &str, p: &Path) -> Paso<String> {
    p.to_str().map(str::to_owned).ok_or_else(|| {
        Informe::nuevo(op, Resultado::Error).diagnostico(&txt::RUTA_WORKTREE_NO_UTF8)
    })
}

/// Que `git status` del principal no muestre los worktrees de tarea:
/// `/.taskcode/worktrees/` en `info/exclude` del repo comun (no versionado).
fn excluir_worktrees(git: &Git, principal: &Path, op: &str) -> Paso<()> {
    excluir(git, principal, op, &format!("/{CARPETA_WORKTREES}/"))
}

/// Anade `linea` a `info/exclude` del repo comun si no esta (no versionado).
pub(crate) fn excluir(git: &Git, principal: &Path, op: &str, linea: &str) -> Paso<()> {
    let comun = exigir(
        git,
        principal,
        op,
        Subcomando::RevParse,
        &["--git-common-dir"],
    )?;
    let comun = principal.join(comun.trim());
    let info = comun.join("info");
    let fichero = info.join("exclude");
    let error = |e: std::io::Error| {
        Informe::nuevo(op, Resultado::Error).diagnostico(error_de_fichero(&fichero, &e))
    };
    // Solo un fichero ausente cuenta como vacio: si no se puede leer, no se
    // reescribe (se perderia lo que la persona tenga en el).
    let actual = match std::fs::read_to_string(&fichero) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(error(e)),
    };
    if actual.lines().any(|l| l.trim() == linea) {
        return Ok(());
    }
    std::fs::create_dir_all(&info).map_err(error)?;
    let mut nuevo = actual;
    if !nuevo.is_empty() && !nuevo.ends_with('\n') {
        nuevo.push('\n');
    }
    nuevo.push_str(linea);
    nuevo.push('\n');
    std::fs::write(&fichero, nuevo).map_err(error)
}

/// Rama y worktree nuevos para una tarea (caso b de F1-3). Repetirlo da
/// `sin-cambios`; si la rama ya existe sin su worktree, la recupera
/// [`reanudar`].
pub fn crear_rama_y_worktree(
    git: &Git,
    dir: &Path,
    id: &str,
    tipo: &str,
    slug: &str,
    base: &str,
) -> Informe {
    crear(git, dir, id, tipo, slug, base).unwrap_or_else(|i| i)
}

fn crear(git: &Git, dir: &Path, id: &str, tipo: &str, slug: &str, base: &str) -> Paso<Informe> {
    const OP: &str = "crear-rama-y-worktree";
    let rama = rama_valida(OP, id, tipo, slug)?;
    if base.is_empty() || base.starts_with('-') {
        return Err(Informe::nuevo(OP, Resultado::Uso).con("motivo", "base-invalida"));
    }
    let principal = localizar_principal(git, dir).map_err(|i| renombrar(i, OP))?;
    let wt = principal.join(CARPETA_WORKTREES).join(id);
    let rel = worktree_relativo(id);
    let rechazo = |motivo: &str| {
        Informe::nuevo(OP, Resultado::Rechazada)
            .con("rama", rama.as_str())
            .con("motivo", motivo)
    };
    if let Some(w) = worktrees(git, &principal, OP)?
        .into_iter()
        .find(|w| w.rama.as_deref() == Some(rama.as_str()))
    {
        return Ok(if misma_ruta(&w.ruta, &wt) {
            Informe::nuevo(OP, Resultado::SinCambios)
                .con("rama", rama.as_str())
                .con("worktree", rel)
        } else {
            rechazo("rama-en-otro-worktree").diagnostico(&txt::RAMA_EN_OTRO_WORKTREE)
        });
    }
    if commit_de(git, &principal, OP, &format!("refs/heads/{rama}"))?.is_some() {
        return Ok(rechazo("rama-existe").diagnostico(&txt::RAMA_YA_EXISTE));
    }
    if commit_de(
        git,
        &principal,
        OP,
        &format!("refs/remotes/{REMOTO}/{rama}"),
    )?
    .is_some()
    {
        return Ok(rechazo("rama-en-remoto").diagnostico(&txt::RAMA_YA_PUBLICADA));
    }
    if wt.exists() {
        return Ok(rechazo("carpeta-ocupada")
            .diagnostico(Mensaje::nuevo(&txt::CARPETA_OCUPADA).con("ruta", rel.as_str())));
    }
    let Some(commit) = commit_de(git, &principal, OP, base)? else {
        return Ok(Informe::nuevo(OP, Resultado::Rechazada)
            .con("base", base)
            .con("motivo", "base-no-existe")
            .diagnostico(Mensaje::nuevo(&txt::BASE_NO_EXISTE).con("base", base)));
    };
    excluir_worktrees(git, &principal, OP)?;
    let wt_txt = ruta_texto(OP, &wt)?;
    exigir(
        git,
        &principal,
        OP,
        Subcomando::Worktree,
        &["add", "-q", "--no-track", "-b", &rama, &wt_txt, &commit],
    )?;
    Ok(Informe::nuevo(OP, Resultado::Ok)
        .con("rama", rama.as_str())
        .con("worktree", rel)
        .con("base", corto(&commit)))
}

/// Recupera la tarea tras perder el worktree o la rama local (caso c de
/// F1-3): crea solo lo que falte. Nada que crear → `sin-cambios`.
pub fn reanudar(git: &Git, dir: &Path, id: &str, tipo: &str, slug: &str) -> Informe {
    reanudar_paso(git, dir, id, tipo, slug).unwrap_or_else(|i| i)
}

fn reanudar_paso(git: &Git, dir: &Path, id: &str, tipo: &str, slug: &str) -> Paso<Informe> {
    const OP: &str = "reanudar";
    let rama = rama_valida(OP, id, tipo, slug)?;
    let principal = localizar_principal(git, dir).map_err(|i| renombrar(i, OP))?;
    let wt = principal.join(CARPETA_WORKTREES).join(id);
    let rel = worktree_relativo(id);
    let rechazo = |motivo: &str| {
        Informe::nuevo(OP, Resultado::Rechazada)
            .con("rama", rama.as_str())
            .con("motivo", motivo)
    };
    // Un worktree borrado a mano sigue registrado hasta que se poda.
    exigir(git, &principal, OP, Subcomando::Worktree, &["prune"])?;
    let mut creado = Vec::new();
    if commit_de(git, &principal, OP, &format!("refs/heads/{rama}"))?.is_none() {
        let remota = format!("refs/remotes/{REMOTO}/{rama}");
        if commit_de(git, &principal, OP, &remota)?.is_none() {
            return Ok(rechazo("rama-no-existe").diagnostico(&txt::TAREA_SIN_RAMA));
        }
        exigir(
            git,
            &principal,
            OP,
            Subcomando::Branch,
            &["-q", "--no-track", &rama, &remota],
        )?;
        creado.push("rama".to_owned());
    }
    match worktrees(git, &principal, OP)?
        .into_iter()
        .find(|w| w.rama.as_deref() == Some(rama.as_str()))
    {
        Some(w) if misma_ruta(&w.ruta, &wt) => {}
        Some(_) => {
            return Ok(rechazo("rama-en-otro-worktree").diagnostico(&txt::RAMA_EN_OTRO_WORKTREE));
        }
        None => {
            if wt.exists() {
                return Ok(rechazo("carpeta-ocupada")
                    .diagnostico(Mensaje::nuevo(&txt::CARPETA_OCUPADA).con("ruta", rel.as_str())));
            }
            excluir_worktrees(git, &principal, OP)?;
            let wt_txt = ruta_texto(OP, &wt)?;
            exigir(
                git,
                &principal,
                OP,
                Subcomando::Worktree,
                &["add", "-q", &wt_txt, &rama],
            )?;
            creado.push("worktree".to_owned());
        }
    }
    let i = Informe::nuevo(
        OP,
        if creado.is_empty() {
            Resultado::SinCambios
        } else {
            Resultado::Ok
        },
    )
    .con("rama", rama.as_str())
    .con("worktree", rel);
    Ok(if creado.is_empty() {
        i
    } else {
        i.con_lista("creado", creado)
    })
}

/// «`ruta`: error» de un fallo de E/S sobre un fichero.
pub(crate) fn error_de_fichero(ruta: &Path, e: &std::io::Error) -> Mensaje {
    Mensaje::nuevo(&txt::ERROR_DE_FICHERO)
        .con("ruta", ruta.display().to_string())
        .con("error", e.to_string())
}

/// Un fallo de `localizar-principal` se informa con el nombre de la operacion
/// que lo pidio.
pub(crate) fn renombrar(mut i: Informe, op: &str) -> Informe {
    op.clone_into(&mut i.comando);
    i
}
