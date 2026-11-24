//! Soporte comun de los tests unitarios de las operaciones sobre la rama de
//! estado (A8-18): el doble [`EnMemoria`] y los ayudantes para montarlo y
//! leer lo que devuelve una operacion.

use std::collections::BTreeMap;

use super::{Cambio, Estado, Fallo};

/// Estado en memoria: ruta -> contenido.
pub(crate) struct EnMemoria(pub(crate) BTreeMap<String, String>);

impl EnMemoria {
    /// Estado con los ficheros dados por su ruta.
    pub(crate) fn de_rutas(ficheros: &[(&str, String)]) -> Self {
        EnMemoria(
            ficheros
                .iter()
                .map(|(r, t)| ((*r).to_owned(), t.clone()))
                .collect(),
        )
    }
}

impl Estado for EnMemoria {
    fn rutas(&self) -> Vec<String> {
        self.0.keys().cloned().collect()
    }
    fn leer(&self, ruta: &str) -> Result<Option<String>, Fallo> {
        Ok(self.0.get(ruta).cloned())
    }
}

/// Estado con cada tarea guardada en `{id}/tarea.md`.
pub(crate) fn estado(tareas: &[(&str, String)]) -> EnMemoria {
    EnMemoria(
        tareas
            .iter()
            .map(|(id, t)| (format!("{id}/tarea.md"), t.clone()))
            .collect(),
    )
}

/// Motivo del rechazo, si la operacion se rechazo.
pub(crate) fn motivo(r: &Result<Vec<Cambio>, Fallo>) -> Option<&str> {
    match r {
        Err(Fallo::Rechazada { motivo, .. }) => Some(motivo),
        _ => None,
    }
}

/// Contenido del unico cambio, o el resultado en `Debug` si no es uno.
pub(crate) fn texto(r: &Result<Vec<Cambio>, Fallo>) -> String {
    match r.as_deref() {
        Ok([c]) => c.contenido.clone(),
        otro => format!("{otro:?}"),
    }
}
