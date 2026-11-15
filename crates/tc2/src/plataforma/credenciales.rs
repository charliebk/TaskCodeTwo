//! Credenciales por rol (ADR F1-6, decisiones 1 y 5).
//!
//! Cada operacion del puerto declara con que [`Rol`] corre y pide su token a
//! un proveedor de [`Credenciales`]. Si falta, la operacion sale `rechazada
//! motivo=identidad` y nunca prueba con otro rol.
//!
//! Hasta que F6-1 cree las Apps, el proveedor de produccion lee variables de
//! entorno; el token de persona solo se usa con un terminal interactivo.

use std::collections::BTreeMap;
use std::io::IsTerminal;

/// Identidad con la que corre una operacion (F1-6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rol {
    /// El agente implementador.
    Agente,
    /// El revisor IA.
    Revisor,
    /// La persona.
    Persona,
}

impl Rol {
    /// Nombre en la linea de contrato.
    pub fn nombre(self) -> &'static str {
        match self {
            Self::Agente => "agente",
            Self::Revisor => "revisor",
            Self::Persona => "persona",
        }
    }

    /// Variable de entorno de su token.
    pub fn variable(self) -> &'static str {
        match self {
            Self::Agente => "TC2_TOKEN_AGENTE",
            Self::Revisor => "TC2_TOKEN_REVISOR",
            Self::Persona => "TC2_TOKEN_PERSONA",
        }
    }
}

/// Un token. No tiene `Display` y su `Debug` no lo ensena.
#[derive(Clone, PartialEq, Eq)]
pub struct Secreto(String);

impl Secreto {
    /// Envuelve un token.
    pub fn nuevo(token: impl Into<String>) -> Self {
        Self(token.into())
    }

    /// El valor, solo para la cabecera `Authorization`.
    pub(crate) fn exponer(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for Secreto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secreto(***)")
    }
}

/// Proveedor de tokens por rol.
pub trait Credenciales {
    /// Token de ese rol, si lo hay.
    fn token(&self, rol: Rol) -> Option<Secreto>;
}

/// Tokens de las variables `TC2_TOKEN_<ROL>`. El de persona solo si la
/// entrada estandar es un terminal (F1-6 decision 5: token de persona solo
/// con TTY).
#[derive(Debug, Clone, Copy, Default)]
pub struct CredencialesEntorno;

impl Credenciales for CredencialesEntorno {
    fn token(&self, rol: Rol) -> Option<Secreto> {
        if rol == Rol::Persona && !std::io::stdin().is_terminal() {
            return None;
        }
        std::env::var(rol.variable())
            .ok()
            .filter(|t| !t.trim().is_empty())
            .map(Secreto::nuevo)
    }
}

/// Tokens fijos (tests y arneses E2E).
#[derive(Debug, Clone, Default)]
pub struct CredencialesFijas(BTreeMap<Rol, Secreto>);

impl CredencialesFijas {
    /// Sin ningun token.
    pub fn nuevas() -> Self {
        Self::default()
    }

    /// Anade el token de un rol.
    #[must_use]
    pub fn con(mut self, rol: Rol, token: Secreto) -> Self {
        self.0.insert(rol, token);
        self
    }
}

impl Credenciales for CredencialesFijas {
    fn token(&self, rol: Rol) -> Option<Secreto> {
        self.0.get(&rol).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_debug_no_ensena_el_token() {
        let s = Secreto::nuevo("ghp_supersecreto");
        assert_eq!(format!("{s:?}"), "Secreto(***)");
        let c = CredencialesFijas::nuevas().con(Rol::Agente, s);
        assert!(!format!("{c:?}").contains("supersecreto"));
        assert!(c.token(Rol::Persona).is_none());
        assert!(c.token(Rol::Agente).is_some());
    }
}
