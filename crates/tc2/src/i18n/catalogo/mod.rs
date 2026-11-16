//! Catalogo de mensajes, un modulo por zona del codigo: cada carpeta de
//! `src/` escribe sus entradas en el suyo y no se pisan.

pub mod actualizar;
pub mod cli;
pub mod comun;
pub mod config;
pub mod estado;
pub mod git;
pub mod identidad;
pub mod plataforma;

use super::Texto;

/// Todas las entradas del catalogo.
pub fn todos() -> impl Iterator<Item = &'static Texto> {
    [
        actualizar::TODOS,
        cli::TODOS,
        comun::TODOS,
        config::TODOS,
        estado::TODOS,
        git::TODOS,
        identidad::TODOS,
        plataforma::TODOS,
    ]
    .into_iter()
    .flatten()
    .copied()
}
