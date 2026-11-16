//! Cerrojo local del clon (`.git/taskcodetwo/estado.lock`, ADR F1-2 con el
//! nombre de [`crate::huella`]): la
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

/// Nombre del fichero del cerrojo en la carpeta propia de la comun.
pub const FICHERO: &str = crate::huella::FICHERO_CERROJO;

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
    /// `espera`. El fichero vive en la carpeta propia de la 2.0, que se
    /// crea si no existe.
    pub fn tomar(comun: &Path, espera: Duration) -> Result<Self, ErrorCerrojo> {
        let dir = crate::huella::dir_propio(comun).map_err(ErrorCerrojo::Io)?;
        let fichero = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(dir.join(FICHERO))
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

#[cfg(test)]
mod tests {
    use super::*;

    /// C2: el cerrojo se escribe en `<comun>/taskcodetwo/estado.lock`, nunca
    /// suelto en la carpeta comun.
    #[test]
    fn el_cerrojo_vive_en_la_carpeta_propia() -> std::io::Result<()> {
        let comun = std::env::temp_dir()
            .join("tc2-tests")
            .join(format!("cerrojo-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&comun);
        std::fs::create_dir_all(&comun)?;
        let tomado = Cerrojo::tomar(&comun, Duration::from_millis(100));
        assert!(tomado.is_ok(), "{tomado:?}");
        assert!(comun.join("taskcodetwo").join("estado.lock").is_file());
        let mut sueltos = Vec::new();
        for e in std::fs::read_dir(&comun)? {
            sueltos.push(e?.file_name());
        }
        assert_eq!(sueltos, ["taskcodetwo"]);
        drop(tomado);
        let _ = std::fs::remove_dir_all(&comun);
        Ok(())
    }
}
