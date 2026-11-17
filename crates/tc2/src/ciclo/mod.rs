//! `tc2 descartar`, `tc2 pausa` y `tc2 reanudar` (F3-7, RF-07 y RF-08).
//!
//! Las operaciones de la rama de estado estan en [`operacion`].

pub mod operacion;

pub use operacion::{Descartar, Pausar, desde_anotacion};
