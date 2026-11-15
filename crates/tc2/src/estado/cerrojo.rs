//! Cerrojo local del clon (`.git/taskcode-estado.lock`, ADR F1-2): la
//! persona y sus agentes del mismo clon no escriben la rama de estado a la
//! vez.
//!
//! Es un cerrojo del sistema operativo sobre el fichero (`File::try_lock`):
//! si el proceso muere, el sistema lo suelta, asi que no puede quedar
//! huerfano (pregunta abierta de F1-2). El fichero se queda; lo que cuenta es
//! el cerrojo, no su existencia.

use std::fs::{File, OpenOptions, TryLockError};
use std::path::Path;
use std::time::{Duration, Instant};

/// Nombre del fichero del cerrojo en la carpeta comun del repo.
pub const FICHERO: &str = "taskcode-estado.lock";

/// Cerrojo tomado; se suelta al soltar el valor.
#[derive(Debug)]
pub struct Cerrojo {
    _fichero: File,
}

/// Por que no se tomo el cerrojo.
#[derive(Debug)]
pub enum ErrorCerrojo {
    /// Otro proceso lo tuvo durante toda la espera.
    Ocupado,
    /// No se pudo abrir el fichero o pedir el cerrojo.
    Io(std::io::Error),
}

impl Cerrojo {
    /// Toma el cerrojo de la carpeta comun `comun`, esperando como mucho
    /// `espera`.
    pub fn tomar(comun: &Path, espera: Duration) -> Result<Self, ErrorCerrojo> {
        let fichero = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(comun.join(FICHERO))
            .map_err(ErrorCerrojo::Io)?;
        let limite = Instant::now() + espera;
        loop {
            match fichero.try_lock() {
                Ok(()) => return Ok(Self { _fichero: fichero }),
                Err(TryLockError::WouldBlock) => {
                    if Instant::now() >= limite {
                        return Err(ErrorCerrojo::Ocupado);
                    }
                    std::thread::sleep(Duration::from_millis(20));
                }
                Err(TryLockError::Error(e)) => return Err(ErrorCerrojo::Io(e)),
            }
        }
    }
}
