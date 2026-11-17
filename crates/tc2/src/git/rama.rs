//! Operaciones de rama y worktree de tarea: `localizar-principal`,
//! `validar-base`, `comprobar-id`, `crear-rama-y-worktree` y `reanudar`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::estado::hay_remoto;
use super::remoto::{FalloRemoto, clasificar_fallo_remoto};
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
    comprobar_id_salvo(git, dir, id, None)
}

/// [`comprobar_id`] sin contar `propia`, la rama que la tarea ya tiene en su
/// `rama:` (F3-10, paso 7 del plan de TASK-081): tras `liberar` o
/// `reasignar`, la siguiente persona sigue en esa rama. Cualquier otra rama
/// `*/task-nnn-*` sigue siendo `ocupada`.
pub fn comprobar_id_salvo(git: &Git, dir: &Path, id: &str, propia: Option<&str>) -> Informe {
    const OP: &str = "comprobar-id";
    if !es_id(id) {
        return Informe::nuevo(OP, Resultado::Uso).con("motivo", "id-invalido");
    }
    match ramas_del_id(git, dir, OP, id).map(|r| ramas_ajenas(r, propia)) {
        Ok(r) if r.is_empty() => Informe::nuevo(OP, Resultado::Ok).con("id", id),
        Ok(r) => Informe::nuevo(OP, Resultado::Rechazada)
            .con("id", id)
            .con("motivo", "ocupada")
            .con_lista("ramas", r)
            .diagnostico(Mensaje::nuevo(&txt::ID_CON_RAMA).con("id", id)),
        Err(i) => i,
    }
}

/// `ramas` sin `propia` (comparacion exacta del nombre).
fn ramas_ajenas(mut ramas: Vec<String>, propia: Option<&str>) -> Vec<String> {
    if let Some(p) = propia {
        ramas.retain(|r| r != p);
    }
    ramas
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
    Ok(nombres(&remoto.stdout, &local))
}

/// Nombres de rama de la salida de `ls-remote --heads` y de `branch --list
/// --format=%(refname:short)`, ordenados y sin repetir.
fn nombres(remoto: &str, local: &str) -> Vec<String> {
    let mut ramas: Vec<String> = remoto
        .lines()
        .filter_map(|l| l.split('\t').nth(1))
        .filter_map(|r| r.strip_prefix("refs/heads/"))
        .chain(local.lines().map(str::trim).filter(|l| !l.is_empty()))
        .map(str::to_owned)
        .collect();
    ramas.sort();
    ramas.dedup();
    ramas
}

/// Patron de las ramas de tarea (ADR F1-3: `<tipo>/task-nnn-<slug>`).
/// `ls-remote` lo casa por sufijo (tambien `refs/heads/task-1-x`, sin tipo) y
/// `branch --list` contra el nombre entero: lo que cuenta lo decide
/// [`id_de_rama`], igual para los dos.
const PATRON_TAREAS: &str = "*/task-*";

/// Cifras como mucho de un numero de tarea en una rama: mas que eso no es un
/// ID (una fecha `task-20261116-x`, un `task-4294967295-y`) y no debe
/// disparar el siguiente ID ni agotarlo.
const CIFRAS_MAX: usize = 6;

/// Numero de tarea de una rama: el primer componente, salvo el primero (el
/// tipo), de la forma `task-<cifras>` o `task-<cifras>-<resto>`, con 1 a
/// [`CIFRAS_MAX`] cifras.
fn id_de_rama(rama: &str) -> Option<u32> {
    rama.split('/').skip(1).find_map(|c| {
        let resto = c.strip_prefix("task-")?;
        let fin = resto
            .find(|ch: char| !ch.is_ascii_digit())
            .unwrap_or(resto.len());
        let (cifras, tras) = resto.split_at(fin);
        (!cifras.is_empty()
            && cifras.len() <= CIFRAS_MAX
            && (tras.is_empty() || tras.starts_with('-')))
        .then(|| cifras.parse().ok())
        .flatten()
    })
}

/// IDs que ya tienen rama de tarea (`*/task-nnn-*`) en local o, si hay
/// `origin`, en el remoto (F2-4b): el alta no nombra una rama que ya existe.
/// Un fallo del remoto se clasifica como en `fetch`: `sinred` (4),
/// `denegada` (8) o `error` local.
pub fn ids_con_rama(git: &Git, dir: &Path, op: &str) -> Paso<BTreeSet<u32>> {
    let principal = localizar_principal(git, dir)?;
    let remoto = if hay_remoto(git, &principal, op)? {
        let s = correr(
            git,
            &principal,
            op,
            Subcomando::LsRemote,
            &["--heads", REMOTO, PATRON_TAREAS],
        )?;
        if !s.exito() {
            return Err(match clasificar_fallo_remoto(&s.stderr) {
                FalloRemoto::SinRed | FalloRemoto::RamaInexistente => {
                    Informe::nuevo(op, Resultado::Sinred)
                        .con("remoto", REMOTO)
                        .diagnostico(super::remoto::sin_red_remoto(op))
                }
                FalloRemoto::Local => super::remoto::error_local(op, &s.primera_linea_error()),
                otro => super::remoto::denegada(op, otro, &s.primera_linea_error()),
            });
        }
        s.stdout
    } else {
        String::new()
    };
    let local = exigir(
        git,
        &principal,
        op,
        Subcomando::Branch,
        &["--list", "--format=%(refname:short)", PATRON_TAREAS],
    )?;
    Ok(nombres(&remoto, &local)
        .iter()
        .filter_map(|r| id_de_rama(r))
        .collect())
}

/// Fecha del ultimo commit (committer, segundos desde 1970) de cada rama de
/// `origin` tal como la conoce el clon (`refs/remotes/origin/*`), por nombre
/// de rama sin `origin/` (F3-10, D2a). Un solo `for-each-ref` y sin red: la
/// frescura es la del ultimo `fetch`.
pub fn fechas_de_ramas(git: &Git, dir: &Path, op: &str) -> Paso<BTreeMap<String, u64>> {
    let prefijo = format!("refs/remotes/{REMOTO}/");
    let s = exigir(
        git,
        dir,
        op,
        Subcomando::ForEachRef,
        &["--format=%(committerdate:unix) %(refname)", &prefijo],
    )?;
    Ok(fechas(&s, &prefijo))
}

/// Las lineas `<segundos> <ref>` de [`fechas_de_ramas`]; las que no se
/// entienden (o `HEAD`, que no es una rama) se saltan.
fn fechas(salida: &str, prefijo: &str) -> BTreeMap<String, u64> {
    salida
        .lines()
        .filter_map(|l| {
            let (fecha, r) = l.trim().split_once(' ')?;
            let rama = r.strip_prefix(prefijo).filter(|r| *r != "HEAD")?;
            Some((rama.to_owned(), fecha.parse().ok()?))
        })
        .collect()
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

/// Inversa de [`nombre_de_rama`]: el `tipo` y el `slug` de `rama` si es la
/// rama de la tarea `id` (`<tipo>/<task-nnn>-<slug>`) y pasa la validacion
/// de las operaciones de rama. `start` (F3-6) crea asi la rama que dejo el
/// alta en `rama:`.
pub fn partes_de_rama(id: &str, rama: &str) -> Option<(String, String)> {
    let (tipo, resto) = rama.split_once('/')?;
    let slug = resto.strip_prefix(&format!("{}-", id.to_lowercase()))?;
    let valida = rama_valida("partes-de-rama", id, tipo, slug).ok()?;
    (valida == rama).then(|| (tipo.to_owned(), slug.to_owned()))
}

fn ruta_texto(op: &str, p: &Path) -> Paso<String> {
    p.to_str().map(str::to_owned).ok_or_else(|| {
        Informe::nuevo(op, Resultado::Error).diagnostico(&txt::RUTA_WORKTREE_NO_UTF8)
    })
}

/// Que `git status` del principal no muestre los worktrees de tarea:
/// `/.taskcodetwo/worktrees/` en `info/exclude` del repo comun (no versionado).
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

#[cfg(test)]
mod tests {
    use super::{fechas, id_de_rama, nombre_de_rama, nombres, partes_de_rama, ramas_ajenas};

    /// TASK-081: las fechas por rama sin `origin/`, sin `HEAD` ni lineas
    /// que no se entienden.
    #[test]
    fn fechas_de_las_ramas_remotas() {
        let f = fechas(
            "1794910830 refs/remotes/origin/feature/task-001-uno\n\
             1 refs/remotes/origin/HEAD\n\
             x refs/remotes/origin/feature/mala\n\
             1700000000 refs/remotes/otro/feature/task-002-x\n\
             1700000001 refs/remotes/origin/develop\n",
            "refs/remotes/origin/",
        );
        let v: Vec<(&str, u64)> = f.iter().map(|(k, v)| (k.as_str(), *v)).collect();
        assert_eq!(
            v,
            [
                ("develop", 1_700_000_001),
                ("feature/task-001-uno", 1_794_910_830)
            ]
        );
    }

    /// TASK-081 (paso 7): la rama propia de la tarea no cuenta como ocupada;
    /// cualquier otra `*/task-nnn-*`, si.
    #[test]
    fn la_rama_propia_no_ocupa_el_id() {
        let ramas = vec![
            "feature/task-001-uno".to_owned(),
            "fix/task-001-a-mano".to_owned(),
        ];
        assert_eq!(
            ramas_ajenas(ramas.clone(), Some("feature/task-001-uno")),
            ["fix/task-001-a-mano"]
        );
        assert_eq!(ramas_ajenas(ramas.clone(), None), ramas);
        assert!(ramas_ajenas(vec![ramas[0].clone()], Some(&ramas[0])).is_empty());
    }

    /// F3-6: ida y vuelta con `nombre_de_rama`; lo que no es la rama de esa
    /// tarea o no pasa la validacion, `None`.
    #[test]
    fn partes_de_rama_es_la_inversa_de_nombre_de_rama() {
        for (id, tipo, slug) in [
            ("TASK-033", "feature", "f3-6-start"),
            ("TASK-1000", "hotfix", "x"),
            ("TASK-007", "fix", "a-b-c-1"),
        ] {
            let rama = nombre_de_rama(id, tipo, slug);
            assert_eq!(
                partes_de_rama(id, &rama),
                Some((tipo.to_owned(), slug.to_owned())),
                "{rama}"
            );
        }
        for (id, rama) in [
            ("TASK-033", "feature/task-034-x"),
            ("TASK-033", "feature/task-033"),
            ("TASK-033", "feature/task-033-"),
            ("TASK-033", "task-033-x"),
            ("TASK-033", "Feature/task-033-x"),
            ("TASK-033", "feature/task-033-con espacio"),
            ("TASK-033", "a/b/task-033-x"),
            ("TASK-33", "feature/task-33-x"),
        ] {
            assert_eq!(partes_de_rama(id, rama), None, "{rama}");
        }
    }

    /// Lo que devuelve `ls-remote` por sufijo (comprobado con git 2.56):
    /// cuenta un componente `task-<cifras>` que no sea el primero.
    #[test]
    fn id_de_las_ramas_de_tarea() {
        for (rama, id) in [
            ("feature/task-005-y", Some(5)),
            ("a/b/task-1-x", Some(1)),
            ("feature/task-10", Some(10)),
            ("feature/task-12-s/sub", Some(12)),
            ("fix/task-0042-x", Some(42)),
            ("task-7-raiz", None),
            ("feature/xtask-9-z", None),
            ("feature/task-x", None),
            ("feature/task-9x", None),
            ("feature/task--1", None),
            ("fix/TASK-11-m", None),
            ("feature/task-99999999999-x", None),
            // Mas de 6 cifras no es un ID: ni rompe ni dispara el siguiente.
            ("feature/task-4294967295-y", None),
            ("feature/task-20261116-x", None),
            ("feature/task-1234567", None),
            ("feature/task-0000001-x", None),
            ("feature/task-999999-x", Some(999_999)),
            ("feature/task-000042", Some(42)),
            // Un componente largo no tapa uno valido mas adelante.
            ("a/task-20261116/task-7-x", Some(7)),
        ] {
            assert_eq!(id_de_rama(rama), id, "{rama}");
        }
        let n = nombres(
            "abc\trefs/heads/feature/task-005-y\nabc\trefs/heads/task-7-raiz\n",
            "feature/task-005-y\n  \nfix/task-3-z\n",
        );
        assert_eq!(n, ["feature/task-005-y", "fix/task-3-z", "task-7-raiz"]);
    }
}
