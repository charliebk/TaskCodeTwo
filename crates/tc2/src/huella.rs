//! Huella de la 2.0 en un repo: los unicos nombres que `tc2` deja en el
//! (ADR F1-12 D1a y su adenda de TASK-116).
//!
//! Todo sale de un solo nombre, [`NOMBRE`], para que la 2.0 y la 0.16 no
//! compartan nada y `tc2 desinstalar` (F2-14) reconozca lo suyo por nombre:
//!
//! - la rama de estado, [`RAMA_ESTADO`];
//! - las refs propias, bajo [`PREFIJO_REFS`];
//! - la carpeta de los worktrees de tarea, [`CARPETA_WORKTREES`];
//! - lo que se guarda en la carpeta comun de git (`.git` del principal),
//!   bajo [`CARPETA_COMUN`]: el cerrojo, la cache de identidad, la marca de
//!   la ultima sincronizacion, los indices temporales de la transaccion y el
//!   progreso en vivo ([`CARPETA_PROGRESO`]). `tc2 desinstalar` borra la
//!   carpeta entera, asi que un fichero nuevo aqui queda reconocido sin
//!   tocarlo. Sus ficheros se escriben de una vez con `escritura::atomica`.
//!
//! La configuracion del proyecto (`.taskcode/config.yml`, `personas.yml` y
//! el vocabulario) no esta aqui: F1-12 D4a la deja compartida con la 0.16.

use std::io;
use std::path::{Path, PathBuf};

/// El nombre del que salen todos los demas. Es un macro para poder
/// componer constantes con `concat!`.
macro_rules! nombre {
    () => {
        "taskcodetwo"
    };
}

/// Nombre propio de la 2.0 en un repo.
pub const NOMBRE: &str = nombre!();

/// Rama huerfana del estado (ADR F1-1 con el nombre de F1-12 D1a).
pub const RAMA_ESTADO: &str = concat!(nombre!(), "/tareas");

/// Prefijo de las unicas refs que escribe la 2.0 (y `update-ref` acepta).
pub const PREFIJO_REFS: &str = concat!("refs/", nombre!(), "/");

/// Refs de las operaciones pendientes de publicar (sin red, F1-2 D2b).
pub const REFS_PENDIENTES: &str = concat!("refs/", nombre!(), "/pendientes/");

/// Refs de las pendientes que al re-ejecutarse ya no procedian.
pub const REFS_RECHAZADAS: &str = concat!("refs/", nombre!(), "/rechazadas/");

/// Carpeta de los worktrees de tarea, relativa al worktree principal
/// (F1-3 D2a: dentro del proyecto, excluida por `info/exclude`).
pub const CARPETA_WORKTREES: &str = concat!(".", nombre!(), "/worktrees");

/// Carpeta propia dentro de la carpeta comun de git (`.git/taskcodetwo/`).
pub const CARPETA_COMUN: &str = nombre!();

/// Cerrojo local del clon, en [`CARPETA_COMUN`] (F1-2).
pub const FICHERO_CERROJO: &str = "estado.lock";

/// Cache de la identidad, en [`CARPETA_COMUN`] (F2-13).
pub const FICHERO_IDENTIDAD: &str = "identidad.yml";

/// Hora del ultimo `fetch` con exito de la rama de estado, en
/// [`CARPETA_COMUN`] (F3-5b, D3a).
pub const FICHERO_SINCRONIZACION: &str = "sincronizacion";

/// Indices temporales de la transaccion, en [`CARPETA_COMUN`].
pub const CARPETA_TEMPORALES: &str = "estado-tmp";

/// Progreso en vivo de cada tarea en curso del clon, en [`CARPETA_COMUN`]
/// (F3-12): `TASK-NNN.json` y el anterior, `TASK-NNN.anterior.json`. Es
/// derivado y sin versionar; lo escriben las transiciones y lo leen la
/// status line y `tc2 status --seguir` (adenda de TASK-084 en F1-17).
pub const CARPETA_PROGRESO: &str = "progreso";

/// Workflow de CI que deja `tc2 init` (F3-1). Hasta que F5-1 entregue el
/// reutilizable, `init` solo deja la ranura (`pendiente motivo=sin-plantilla`).
pub const RUTA_WORKFLOW: &str = concat!(".github/workflows/", nombre!(), ".yml");

/// Lineas que `tc2 init` anade a `.gitignore` (F1-3 encargaba a F3-1 ignorar
/// la carpeta montada y la propia): versionadas, se anaden y nunca se quitan
/// solas.
pub const LINEAS_GITIGNORE: [&str; 2] = ["/tareas/", concat!("/.", nombre!(), "/")];

/// Lineas que la 2.0 escribe en `info/exclude` (`git::rama::excluir`): los
/// worktrees de tarea y la rama de estado montada.
pub const LINEAS_EXCLUDE: [&str; 2] = [concat!("/.", nombre!(), "/worktrees/"), "/tareas/"];

/// Prefijo de los rulesets que propone `tc2 init` (F1-12 D1a; F1-5 los
/// llamaba `taskcode-*`).
pub const PREFIJO_RULESETS: &str = concat!(nombre!(), "-");

/// Texto del `LEEME.md` del commit raiz de la rama de estado.
pub(crate) const TEXTO_LEEME: &str = concat!(
    "# Rama de estado de TaskCodeTwo\n\nEsta rama (`",
    nombre!(),
    "/tareas`) guarda las tareas y el roadmap (ADR F1-1).\nNo tiene historia comun con las ramas de codigo: no la mezcles con ellas.\nLa escribe el CLI; se monta como worktree en `tareas/`.\n"
);

/// Carpeta propia bajo la carpeta comun `comun`, sin crearla.
pub fn en_comun(comun: &Path) -> PathBuf {
    comun.join(CARPETA_COMUN)
}

/// Carpeta propia bajo la carpeta comun `comun`, creada si no existe.
pub fn dir_propio(comun: &Path) -> io::Result<PathBuf> {
    let d = en_comun(comun);
    std::fs::create_dir_all(&d)?;
    Ok(d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todo_sale_del_mismo_nombre() {
        assert_eq!(NOMBRE, "taskcodetwo");
        assert_eq!(RAMA_ESTADO, format!("{NOMBRE}/tareas"));
        assert_eq!(PREFIJO_REFS, format!("refs/{NOMBRE}/"));
        assert_eq!(CARPETA_COMUN, NOMBRE);
        assert_eq!(CARPETA_WORKTREES, format!(".{NOMBRE}/worktrees"));
        assert!(TEXTO_LEEME.contains(&format!("(`{RAMA_ESTADO}`)")));
        assert_eq!(RUTA_WORKFLOW, format!(".github/workflows/{NOMBRE}.yml"));
        assert_eq!(PREFIJO_RULESETS, format!("{NOMBRE}-"));
        // Los ficheros de la carpeta comun no se pisan entre si.
        let ficheros = [
            FICHERO_CERROJO,
            FICHERO_IDENTIDAD,
            FICHERO_SINCRONIZACION,
            CARPETA_TEMPORALES,
            CARPETA_PROGRESO,
        ];
        for (i, a) in ficheros.iter().enumerate() {
            assert!(ficheros[i + 1..].iter().all(|b| a != b), "{a} repetido");
        }
    }

    /// Las lineas de `exclude` y `.gitignore` nombran las carpetas reales.
    #[test]
    fn lineas_de_exclude_y_gitignore() {
        let tareas = format!("/{}/", crate::estado::CARPETA);
        assert_eq!(
            LINEAS_EXCLUDE,
            [format!("/{CARPETA_WORKTREES}/"), tareas.clone()]
        );
        assert_eq!(LINEAS_GITIGNORE, [tareas, format!("/.{NOMBRE}/")]);
    }

    /// C5: las refs propias viven bajo `refs/taskcodetwo/`.
    #[test]
    fn refs_propias_bajo_su_prefijo() {
        assert_eq!(PREFIJO_REFS, "refs/taskcodetwo/");
        assert_eq!(REFS_PENDIENTES, "refs/taskcodetwo/pendientes/");
        assert_eq!(REFS_RECHAZADAS, "refs/taskcodetwo/rechazadas/");
        assert_eq!(crate::git::PREFIJO_REFS_PROPIAS, PREFIJO_REFS);
        assert_eq!(crate::estado::REFS_PENDIENTES, REFS_PENDIENTES);
        assert_eq!(crate::estado::REFS_RECHAZADAS, REFS_RECHAZADAS);
    }

    /// C3: la carpeta de worktrees no coincide con ninguna ruta de la 0.16
    /// (`.taskcode/...` en el arbol y `.git/taskcode/...` en la comun).
    #[test]
    fn worktrees_fuera_de_las_rutas_de_la_016() {
        let w = crate::git::CARPETA_WORKTREES;
        assert_eq!(w, ".taskcodetwo/worktrees");
        for ruta_016 in [".taskcode/", ".git/taskcode/", ".git/", "tareas/"] {
            assert!(!w.starts_with(ruta_016), "{w} empieza por {ruta_016}");
        }
        assert_ne!(w.split('/').next(), Some(".taskcode"));
    }

    #[test]
    fn dir_propio_crea_la_carpeta() -> io::Result<()> {
        let comun = std::env::temp_dir()
            .join("tc2-tests")
            .join(format!("huella-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&comun);
        let d = dir_propio(&comun)?;
        assert_eq!(d, comun.join("taskcodetwo"));
        assert!(d.is_dir());
        let _ = std::fs::remove_dir_all(&comun);
        Ok(())
    }
}
