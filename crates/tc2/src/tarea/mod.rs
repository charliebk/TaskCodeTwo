//! Modelo de tarea (F2-5; ADR F1-9): parser y serializador de `tarea.md`.
//!
//! - [`leer`] distingue el formato por la clave `formato`: con ella, formato 2
//!   ([`Tarea`], el que escribe la 2.0); sin ella, formato 1 de la 0.16
//!   ([`TareaV1`]), que se lee para migrarlo pero no se escribe (F1-9 punto 3).
//! - [`Tarea::escribir`] emite el frontmatter en la forma fija de F1-9: YAML
//!   valido para un parser estandar (H11), cadenas siempre entre comillas.
//! - El cuerpo se lee por secciones reconocidas por titulo ([`Cuerpo`]) y se
//!   conserva byte a byte.
//!
//! Hoja pura: no depende de git, de la red ni de la rama de estado.

mod cuerpo;
mod modelo;
mod yaml;

use std::fmt;

pub use cuerpo::{Cuerpo, Seccion};
pub use modelo::{
    COMPLEJIDADES, ESTADOS, REGLAS, Rol, RolRevision, Roles, TIPOS, Tarea, TareaV1, id_valido,
    titulo_valido,
};

/// Un `tarea.md` leido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Documento {
    /// Formato 1 (0.16), sin `formato`.
    Formato1(TareaV1, Cuerpo),
    /// Formato 2 (`formato: 2`).
    Formato2(Tarea, Cuerpo),
}

/// Por que no se pudo leer un `tarea.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorTarea {
    /// Clave del frontmatter a la que se refiere, si es de una.
    pub clave: Option<String>,
    /// Que falla.
    pub mensaje: String,
}

impl ErrorTarea {
    pub(crate) fn general(mensaje: impl Into<String>) -> Self {
        Self {
            clave: None,
            mensaje: mensaje.into(),
        }
    }

    pub(crate) fn en(clave: &str, mensaje: impl Into<String>) -> Self {
        Self {
            clave: Some(clave.to_owned()),
            mensaje: mensaje.into(),
        }
    }
}

impl fmt::Display for ErrorTarea {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.clave {
            Some(c) => write!(f, "{c}: {}", self.mensaje),
            None => f.write_str(&self.mensaje),
        }
    }
}

impl std::error::Error for ErrorTarea {}

/// Lee un `tarea.md` (todo es dato no confiable, F1-8: esquema estricto,
/// claves duplicadas rechazadas, desconocidas conservadas).
pub fn leer(texto: &str) -> Result<Documento, ErrorTarea> {
    let partes = yaml::partir(texto)?;
    let cuerpo = Cuerpo::leer(partes.cuerpo);
    if partes.tiene("formato") {
        Ok(Documento::Formato2(Tarea::leer(&partes)?, cuerpo))
    } else {
        Ok(Documento::Formato1(TareaV1::leer(&partes)?, cuerpo))
    }
}
