//! Progreso en vivo de una tarea (F3-12; ADR F1-17, puntos 5, 7 y 8, con la
//! adenda de TASK-084).
//!
//! - [`fichero`]: `progreso/TASK-NNN.json` bajo `.git/taskcodetwo/`, con el
//!   anterior en `TASK-NNN.anterior.json`.
//! - [`calculo`]: `calcular`, puro, sobre la estimacion de TASK-083.
//! - [`linea`]: la linea de la status line y de `status --seguir`, pura.
//! - [`orden_estado`]: `tc2 estado --stdin`, la status line de Claude Code.
//!   No recibe [`crate::git::Git`]: sin red ni subprocesos por
//!   construccion. Busca el repo subiendo desde `workspace.current_dir` del
//!   JSON (o `cwd`, o la carpeta del proceso) hasta un `.git` (carpeta, o
//!   fichero `gitdir:` con `commondir`), y lee uno o pocos ficheros. Dentro
//!   de `.taskcodetwo/worktrees/<ID>` la tarea es esa; en el principal, la
//!   de calculo mas reciente y `+N en curso`. Siempre una linea y codigo 0.

pub mod calculo;
pub mod fichero;
pub mod linea;

use std::io::Read;
use std::path::{Path, PathBuf};

pub use calculo::calcular;
pub use fichero::{Lectura, Progreso, Punto};

use crate::Salida;
use crate::git::Git;

/// Orden de la status line (`tc2 estado --stdin`, la de
/// [`distribucion::STATUS_LINE`]).
pub const ESTADO: &str = "estado";

/// Bytes como mucho del JSON de la status line que se leen de stdin.
pub const LIMITE_STDIN: u64 = 1024 * 1024;

/// Bytes como mucho de un fichero `.git` o `commondir`.
const LIMITE_GIT: u64 = 4096;

/// Un repo encontrado sin lanzar git.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repo {
    /// Carpeta git del worktree (`.git`, o la de `gitdir:`).
    pub gitdir: PathBuf,
    /// Carpeta comun de git (la `.git` del principal).
    pub comun: PathBuf,
    /// La tarea si el worktree es `.taskcodetwo/worktrees/<ID>`.
    pub id: Option<String>,
}

/// Texto de un fichero pequeno, sin el salto final.
fn texto_corto(p: &Path) -> Option<String> {
    let mut s = String::new();
    std::fs::File::open(p)
        .ok()?
        .take(LIMITE_GIT + 1)
        .read_to_string(&mut s)
        .ok()?;
    (s.len() as u64 <= LIMITE_GIT).then(|| s.trim_end_matches(['\r', '\n']).to_owned())
}

/// `rel` relativo a `base` si no es absoluto.
fn junto(base: &Path, rel: &str) -> PathBuf {
    let r = Path::new(rel);
    if r.is_absolute() {
        r.to_path_buf()
    } else {
        base.join(r)
    }
}

/// El ID si `raiz` es `<principal>/.taskcodetwo/worktrees/<ID>`.
fn id_de_worktree(raiz: &Path) -> Option<String> {
    let id = raiz.file_name()?.to_str()?;
    let padre = raiz.parent()?;
    let abuelo = padre.parent()?;
    let carpeta = format!(
        "{}/{}",
        abuelo.file_name()?.to_str()?,
        padre.file_name()?.to_str()?
    );
    (carpeta == crate::huella::CARPETA_WORKTREES && crate::git::es_id(id)).then(|| id.to_owned())
}

/// Busca el repo de `desde` subiendo por sus carpetas, sin lanzar git.
pub fn buscar(desde: &Path) -> Option<Repo> {
    for d in desde.ancestors() {
        let punto_git = d.join(".git");
        let Ok(m) = std::fs::metadata(&punto_git) else {
            continue;
        };
        if m.is_dir() {
            return Some(Repo {
                gitdir: punto_git.clone(),
                comun: punto_git,
                id: None,
            });
        }
        let t = texto_corto(&punto_git)?;
        let gitdir = junto(d, t.strip_prefix("gitdir:")?.trim());
        let comun = texto_corto(&gitdir.join("commondir"))
            .map_or_else(|| gitdir.clone(), |c| junto(&gitdir, c.trim()));
        return Some(Repo {
            gitdir,
            comun,
            id: id_de_worktree(d),
        });
    }
    None
}

/// Segundos desde 1970 de la ultima modificacion de `p`.
fn mtime(p: &Path) -> Option<u64> {
    std::fs::metadata(p)
        .ok()?
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
}

/// La ultima actividad que se ve sin lanzar git: el `logs/HEAD` del
/// worktree (con `core.logAllRefUpdates`, por defecto en un repo con
/// worktree, cada commit lo toca).
fn actividad(gitdir: &Path) -> Option<u64> {
    mtime(&gitdir.join("logs").join("HEAD"))
}

/// La linea de una tarea leida.
fn linea_de(l: &Lectura, id: &str, gitdir: &Path, ahora: u64) -> Option<(u64, String)> {
    match l {
        Lectura::Ninguno => None,
        Lectura::Bueno(p) => Some((p.calculada, linea::linea(p, ahora, actividad(gitdir)))),
        Lectura::Anterior(p) => Some((
            p.calculada,
            linea::con_antiguedad(&linea::linea(p, ahora, actividad(gitdir)), p, ahora),
        )),
        Lectura::Ilegible => Some((0, linea::ilegible(id))),
    }
}

/// La linea de progreso para la carpeta `dir` a las `ahora`.
pub fn linea_en(dir: &Path, ahora: u64) -> String {
    let Some(repo) = buscar(dir) else {
        return linea::SIN_TAREA.to_owned();
    };
    let carpeta = fichero::carpeta(&repo.comun);
    if let Some(id) = &repo.id {
        return linea_de(&fichero::leer(&carpeta, id), id, &repo.gitdir, ahora)
            .map_or_else(|| linea::SIN_TAREA.to_owned(), |(_, l)| l);
    }
    // En el principal (o en `tareas/`): la de calculo mas reciente.
    let ids = fichero::ids(&carpeta);
    let mejor = ids
        .iter()
        .filter_map(|id| {
            let gitdir = repo.comun.join("worktrees").join(id);
            linea_de(&fichero::leer(&carpeta, id), id, &gitdir, ahora)
        })
        .max_by_key(|(calculada, _)| *calculada);
    match mejor {
        Some((_, l)) => linea::con_mas(&l, ids.len().saturating_sub(1)),
        None => linea::SIN_TAREA.to_owned(),
    }
}

/// La carpeta que dice el JSON de la status line: `workspace.current_dir`
/// o, si no viene, `cwd` (documentacion de Claude Code: los dos llevan lo
/// mismo).
pub fn carpeta_del_json(json: &str) -> Option<PathBuf> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    v.pointer("/workspace/current_dir")
        .and_then(serde_json::Value::as_str)
        .or_else(|| v.get("cwd").and_then(serde_json::Value::as_str))
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
}

/// `tc2 estado --stdin` con lo leido de stdin, la carpeta del proceso como
/// respaldo y la hora. Sin `Git` en la firma: sin red ni subprocesos.
pub fn orden_estado(stdin: &str, cwd: Option<&Path>, ahora: u64) -> String {
    match carpeta_del_json(stdin).or_else(|| cwd.map(Path::to_path_buf)) {
        Some(d) => linea_en(&d, ahora),
        None => linea::SIN_TAREA.to_owned(),
    }
}

/// Que hizo un refresco.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refresco {
    /// Escribio el progreso de la tarea.
    Escrito,
    /// La tarea no esta en curso ni en revision (o la deja quien la lanza):
    /// borro su progreso.
    Borrado,
}

/// Nombre de la operacion en los informes de error.
const OP: &str = "progreso";

/// La carpeta de progreso del repo de `dir` (localiza el principal: lanza
/// git).
fn carpeta_de(g: &Git, dir: &Path) -> Result<(PathBuf, PathBuf), String> {
    let principal = crate::git::localizar_principal(g, dir).map_err(|i| i.linea())?;
    let comun = crate::git::estado::dir_comun(g, &principal, OP).map_err(|i| i.linea())?;
    Ok((principal, fichero::carpeta(&comun)))
}

/// `refrescar(g, dir, id, ahora)`: recalcula y escribe el progreso de `id`
/// tras una transicion, con el estado que conoce el clon (sin `fetch`):
/// todos los `tarea.md` y un `git log` de `develop` (O(N): solo en las
/// transiciones). Si la tarea no esta en curso ni en revision, borra su
/// progreso. Lo llaman las transiciones (`lib.rs`, `de_una_tarea`); `review`
/// y `finish` (F6-2, F7-1) lo llamaran igual cuando existan.
pub fn refrescar(g: &Git, dir: &Path, id: &str, ahora: u64) -> Result<Refresco, String> {
    let (principal, carpeta) = carpeta_de(g, dir)?;
    let vista = crate::estado::leer(g, dir).map_err(|i| i.linea())?;
    let texto = |f: crate::estado::Fallo| match f {
        crate::estado::Fallo::Rechazada { motivo, .. } => motivo,
        crate::estado::Fallo::Error(d) => d.texto(crate::i18n::Idioma::Es),
    };
    let (t, c) = match crate::estado::tarea_de(&vista, id) {
        Ok(x) => x,
        // No existe, es de la 0.16 o no se entiende: no hay progreso.
        Err(crate::estado::Fallo::Rechazada { .. }) => return quitar_en(&carpeta, id),
        Err(f) => return Err(texto(f)),
    };
    if calculo::fase_de(&t.estado).is_none() {
        return quitar_en(&carpeta, id);
    }
    let leidas =
        crate::estado::tareas(&vista, &crate::estado::Estado::rutas(&vista)).map_err(texto)?;
    let commits = crate::estimacion::commits_de(g, &principal);
    let h = crate::estimacion::historico(&leidas, &commits);
    match calcular(&t, &c, &h, &commits, ahora) {
        Some(p) => fichero::escribir(&carpeta, &p)
            .map(|()| Refresco::Escrito)
            .map_err(|e| e.to_string()),
        None => quitar_en(&carpeta, id),
    }
}

fn quitar_en(carpeta: &Path, id: &str) -> Result<Refresco, String> {
    fichero::borrar(carpeta, id)
        .map(|()| Refresco::Borrado)
        .map_err(|e| e.to_string())
}

/// Clave de la linea de una transicion cuyo progreso no se pudo escribir.
pub const CLAVE: &str = "progreso";

/// Valor de [`CLAVE`].
pub const NO_ESCRITO: &str = "no-escrito";

/// Escribe `p`, ya calculado, en el clon de `dir` (lo usa `start`, que lo
/// calcula dentro de la toma con el historico que ya leyo).
pub fn escribir_en(g: &Git, dir: &Path, p: &Progreso) -> Result<(), String> {
    let (_, carpeta) = carpeta_de(g, dir)?;
    fichero::escribir(&carpeta, p).map_err(|e| e.to_string())
}

/// Borra el progreso de `id` en el clon de `dir` (tras `descartar`,
/// `liberar` o `reasignar`: la tarea deja de estar en curso aqui).
pub fn quitar(g: &Git, dir: &Path, id: &str) -> Result<Refresco, String> {
    let (_, carpeta) = carpeta_de(g, dir)?;
    quitar_en(&carpeta, id)
}

/// `tc2 estado --stdin` en el proceso: lee stdin (con tope), la carpeta
/// actual y el reloj. Siempre codigo 0 y stderr vacio.
pub fn estado_del_proceso() -> Salida {
    let mut entrada = String::new();
    let _ = std::io::stdin()
        .lock()
        .take(LIMITE_STDIN)
        .read_to_string(&mut entrada);
    let cwd = std::env::current_dir().ok();
    let l = orden_estado(&entrada, cwd.as_deref(), crate::identidad::ahora());
    Salida {
        codigo: 0,
        stdout: format!("{l}\n"),
        stderr: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(nombre: &str) -> PathBuf {
        let d = std::env::temp_dir()
            .join("tc2-tests")
            .join(format!("progreso-busca-{nombre}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    /// La busqueda sin git: `.git` carpeta o fichero (`gitdir:` relativo o
    /// absoluto, con `commondir`), desde una subcarpeta; en un worktree de
    /// tarea, su ID.
    #[test]
    fn busca_sin_git() -> std::io::Result<()> {
        let raiz = dir("busca");
        let principal = raiz.join("p");
        let comun = principal.join(".git");
        std::fs::create_dir_all(principal.join("src").join("a"))?;
        std::fs::create_dir_all(&comun)?;
        let r = buscar(&principal.join("src").join("a"));
        assert_eq!(
            r,
            Some(Repo {
                gitdir: comun.clone(),
                comun: comun.clone(),
                id: None
            })
        );
        // Worktree de tarea con `gitdir:` absoluto y `commondir` relativo.
        let wt = principal
            .join(".taskcodetwo")
            .join("worktrees")
            .join("TASK-012");
        let admin = comun.join("worktrees").join("TASK-012");
        std::fs::create_dir_all(wt.join("sub"))?;
        std::fs::create_dir_all(&admin)?;
        std::fs::write(wt.join(".git"), format!("gitdir: {}\n", admin.display()))?;
        std::fs::write(admin.join("commondir"), "../..\n")?;
        let r = buscar(&wt.join("sub")).unwrap_or_else(|| unreachable!());
        assert_eq!(r.gitdir, admin);
        assert_eq!(r.comun, admin.join("../.."));
        assert_eq!(r.id.as_deref(), Some("TASK-012"));
        // `tareas/` (gitdir relativo, sin commondir): no es de una tarea.
        let tareas = principal.join("tareas");
        std::fs::create_dir_all(&tareas)?;
        std::fs::write(tareas.join(".git"), "gitdir: ../.git/worktrees/tareas")?;
        let r = buscar(&tareas).unwrap_or_else(|| unreachable!());
        assert_eq!(r.id, None);
        assert_eq!(r.gitdir, tareas.join("../.git/worktrees/tareas"));
        // Un worktree con nombre que no es un ID, o fuera de la carpeta.
        let otro = principal.join(".taskcodetwo").join("worktrees").join("x");
        std::fs::create_dir_all(&otro)?;
        std::fs::write(otro.join(".git"), format!("gitdir: {}", admin.display()))?;
        assert_eq!(buscar(&otro).and_then(|r| r.id), None);
        // Un `.git` fichero que no es `gitdir:`: nada.
        let raro = raiz.join("raro");
        std::fs::create_dir_all(&raro)?;
        std::fs::write(raro.join(".git"), "basura")?;
        assert_eq!(buscar(&raro), None);
        let _ = std::fs::remove_dir_all(&raiz);
        Ok(())
    }

    /// La orden es la que reconoce `tc2 desinstalar` (F1-12 D1a).
    #[test]
    fn la_orden_es_la_de_la_distribucion() {
        assert_eq!(format!("tc2 {ESTADO} --stdin"), distribucion::STATUS_LINE);
    }

    #[test]
    fn carpeta_del_json_con_respaldo() {
        assert_eq!(
            carpeta_del_json(r#"{"workspace":{"current_dir":"/a/b"},"cwd":"/c"}"#),
            Some(PathBuf::from("/a/b"))
        );
        assert_eq!(
            carpeta_del_json(r#"{"cwd":"/c"}"#),
            Some(PathBuf::from("/c"))
        );
        for malo in [
            "",
            "x",
            "{}",
            r#"{"workspace":{"current_dir":7}}"#,
            r#"{"cwd":""}"#,
        ] {
            assert_eq!(carpeta_del_json(malo), None, "{malo}");
        }
    }

    /// C7: fuera de un repo, sin tarea; el JSON que no vale usa la carpeta
    /// del proceso.
    #[test]
    fn c7_fuera_de_un_repo() {
        let d = dir("fuera");
        let _ = std::fs::create_dir_all(&d);
        // La carpeta temporal no esta dentro de un repo (si lo estuviera, el
        // test no probaria nada: se comprueba).
        if buscar(&d).is_none() {
            let json = format!(
                "{{\"workspace\":{{\"current_dir\":{}}}}}",
                serde_json::Value::from(d.display().to_string())
            );
            assert_eq!(orden_estado(&json, None, 0), linea::SIN_TAREA);
            assert_eq!(orden_estado("no es json", Some(&d), 0), linea::SIN_TAREA);
        }
        assert_eq!(orden_estado("", None, 0), linea::SIN_TAREA);
        let _ = std::fs::remove_dir_all(&d);
    }

    /// La linea del principal: la de calculo mas reciente y `+N en curso`;
    /// en un worktree de tarea, la suya.
    #[test]
    fn principal_y_worktree() -> std::io::Result<()> {
        let raiz = dir("principal");
        let principal = raiz.join("p");
        let comun = principal.join(".git");
        std::fs::create_dir_all(&comun)?;
        let carpeta = fichero::carpeta(&comun);
        assert_eq!(linea_en(&principal, 2_000), linea::SIN_TAREA);
        let mut a = fichero::tests::ejemplo("TASK-001");
        a.calculada = 1_000;
        let mut b = fichero::tests::ejemplo("TASK-002");
        b.calculada = 1_500;
        fichero::escribir(&carpeta, &a)?;
        fichero::escribir(&carpeta, &b)?;
        let l = linea_en(&principal, 1_500);
        assert!(
            l.starts_with("TASK-002 · ") && l.ends_with(" · +1 en curso"),
            "{l}"
        );
        let wt = principal
            .join(".taskcodetwo")
            .join("worktrees")
            .join("TASK-001");
        std::fs::create_dir_all(&wt)?;
        std::fs::write(wt.join(".git"), "gitdir: ../../../.git/worktrees/TASK-001")?;
        std::fs::create_dir_all(comun.join("worktrees").join("TASK-001"))?;
        std::fs::write(
            comun.join("worktrees").join("TASK-001").join("commondir"),
            "../..",
        )?;
        let l = linea_en(&wt, 1_000);
        assert!(
            l.starts_with("TASK-001 · ") && !l.contains("en curso ·"),
            "{l}"
        );
        // La tarea de otro worktree sin fichero: sin tarea.
        let wt3 = principal
            .join(".taskcodetwo")
            .join("worktrees")
            .join("TASK-003");
        std::fs::create_dir_all(&wt3)?;
        std::fs::write(wt3.join(".git"), "gitdir: ../../../.git/worktrees/TASK-003")?;
        assert_eq!(linea_en(&wt3, 1_000), linea::SIN_TAREA);
        let _ = std::fs::remove_dir_all(&raiz);
        Ok(())
    }
}
