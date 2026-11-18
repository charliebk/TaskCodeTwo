//! Progreso en vivo de una tarea (F3-12; ADR F1-17, puntos 5, 7 y 8, con la
//! adenda de TASK-084).
//!
//! - [`fichero`]: `progreso/TASK-NNN.json` bajo `.git/taskcodetwo/`, con el
//!   anterior en `TASK-NNN.anterior.json`.
//! - [`calculo`]: `calcular`, puro, sobre la estimacion de TASK-083.

pub mod calculo;
pub mod fichero;

pub use calculo::calcular;
pub use fichero::{Lectura, Progreso, Punto};
