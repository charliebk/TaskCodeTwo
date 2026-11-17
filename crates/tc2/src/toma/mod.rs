//! `tc2 start` (F3-6): toma atomica de la tarea y su rama y worktree.

pub mod operacion;

pub use operacion::{Tomar, desde_anotacion, instante};

/// Nombre de la orden y de la operacion de la rama de estado.
pub const NOMBRE: &str = "start";
