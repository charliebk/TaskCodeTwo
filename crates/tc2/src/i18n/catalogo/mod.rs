//! Catalogo de mensajes, un modulo por zona del codigo: cada carpeta de
//! `src/` escribe sus entradas en el suyo y no se pisan.

pub mod actualizar;
pub mod ciclo;
pub mod cli;
pub mod comun;
pub mod config;
pub mod desinstalar;
pub mod doctor;
pub mod estado;
pub mod estimacion;
pub mod git;
pub mod hallazgos;
pub mod identidad;
pub mod importar;
pub mod init;
pub mod plan;
pub mod plataforma;
pub mod progreso;
pub mod revision;
pub mod roadmap;
pub mod seleccion;
pub mod siguiente;
pub mod tablero;
pub mod toma;

use super::Texto;

/// Todas las entradas del catalogo.
pub fn todos() -> impl Iterator<Item = &'static Texto> {
    [
        actualizar::TODOS,
        ciclo::TODOS,
        cli::TODOS,
        comun::TODOS,
        config::TODOS,
        desinstalar::TODOS,
        doctor::TODOS,
        estado::TODOS,
        estimacion::TODOS,
        git::TODOS,
        hallazgos::TODOS,
        identidad::TODOS,
        importar::TODOS,
        init::TODOS,
        plan::TODOS,
        plataforma::TODOS,
        progreso::TODOS,
        revision::TODOS,
        roadmap::TODOS,
        seleccion::TODOS,
        siguiente::TODOS,
        tablero::TODOS,
        toma::TODOS,
    ]
    .into_iter()
    .flatten()
    .copied()
}
