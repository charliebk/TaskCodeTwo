//! Catalogo de mensajes, un modulo por zona del codigo: cada carpeta de
//! `src/` escribe sus entradas en el suyo y no se pisan.

pub mod actualizar;
pub mod cli;
pub mod comun;
pub mod config;
pub mod desinstalar;
pub mod estado;
pub mod git;
pub mod identidad;
pub mod importar;
pub mod init;
pub mod plataforma;
pub mod roadmap;
pub mod siguiente;
pub mod tablero;

use super::Texto;

/// Todas las entradas del catalogo.
pub fn todos() -> impl Iterator<Item = &'static Texto> {
    [
        actualizar::TODOS,
        cli::TODOS,
        comun::TODOS,
        config::TODOS,
        desinstalar::TODOS,
        estado::TODOS,
        git::TODOS,
        identidad::TODOS,
        importar::TODOS,
        init::TODOS,
        plataforma::TODOS,
        roadmap::TODOS,
        siguiente::TODOS,
        tablero::TODOS,
    ]
    .into_iter()
    .flatten()
    .copied()
}
