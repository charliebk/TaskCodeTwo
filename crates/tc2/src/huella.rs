//! Huella de la 2.0 en un repo: los unicos nombres que `tc2` deja en el
//! (ADR F1-12 D1a y su adenda de TASK-116).
//!
//! Todo sale de un solo nombre, [`NOMBRE`], para que la 2.0 y la 0.16 no
//! compartan nada y `tc2 desinstalar` (F2-14) reconozca lo suyo por nombre:
//!
//! - la rama de estado, [`RAMA_ESTADO`];
//! - las refs propias, bajo [`PREFIJO_REFS`];
//! - la carpeta de los worktrees de tarea, [`CARPETA_WORKTREES`].
//!
//! La configuracion del proyecto (`.taskcode/config.yml`, `personas.yml` y
//! el vocabulario) no esta aqui: F1-12 D4a la deja compartida con la 0.16.

/// El nombre del que salen todos los demas. Es un macro para poder
/// componer constantes con `concat!`.
macro_rules! nombre {
    () => {
        "taskcode"
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

/// Texto del `LEEME.md` del commit raiz de la rama de estado.
pub(crate) const TEXTO_LEEME: &str = concat!(
    "# Rama de estado de TaskCodeTwo\n\nEsta rama (`",
    nombre!(),
    "/tareas`) guarda las tareas y el roadmap (ADR F1-1).\nNo tiene historia comun con las ramas de codigo: no la mezcles con ellas.\nLa escribe el CLI; se monta como worktree en `tareas/`.\n"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todo_sale_del_mismo_nombre() {
        assert_eq!(RAMA_ESTADO, format!("{NOMBRE}/tareas"));
        assert_eq!(PREFIJO_REFS, format!("refs/{NOMBRE}/"));
        assert!(REFS_PENDIENTES.starts_with(PREFIJO_REFS));
        assert!(REFS_RECHAZADAS.starts_with(PREFIJO_REFS));
        assert_eq!(CARPETA_WORKTREES, format!(".{NOMBRE}/worktrees"));
        assert!(TEXTO_LEEME.contains(&format!("(`{RAMA_ESTADO}`)")));
    }
}
