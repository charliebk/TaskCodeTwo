//! Carriles de la politica (ADR F1-10, decision 1): el sitio unico de la
//! tabla de suelo. F2-7 solo necesita las columnas que valida la
//! configuracion (roles de diseno minimos y perfil); F4 y F6 amplian este
//! modulo en vez de copiar la tabla.

use serde::Deserialize;

/// Carril de una tarea, de menor a mayor exigencia (F1-10, decision 1).
/// `release` y `hotfix` son tipos, no carriles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Carril {
    /// Solo documentacion.
    Errata,
    /// Correccion.
    Fix,
    /// Funcionalidad nueva.
    Funcionalidad,
    /// Toca `.github/`, `.taskcode/`, `CODEOWNERS` o rutas declaradas.
    RiesgoAlto,
}

impl Carril {
    /// Los cuatro carriles, en orden.
    pub const TODOS: [Self; 4] = [
        Self::Errata,
        Self::Fix,
        Self::Funcionalidad,
        Self::RiesgoAlto,
    ];

    /// Nombre en la configuracion y en la linea.
    pub fn nombre(self) -> &'static str {
        match self {
            Self::Errata => "errata",
            Self::Fix => "fix",
            Self::Funcionalidad => "funcionalidad",
            Self::RiesgoAlto => "riesgo-alto",
        }
    }

    /// Carril por su nombre.
    pub fn desde(nombre: &str) -> Option<Self> {
        Self::TODOS.into_iter().find(|c| c.nombre() == nombre)
    }

    /// Roles de diseno minimos del suelo (columna «Roles de diseno»; «todos»
    /// en riesgo alto son los cuatro roles base, como en la PoC de F1-10).
    pub fn roles_min(self) -> u64 {
        match self {
            Self::Errata => 0,
            Self::Fix | Self::Funcionalidad => 1,
            Self::RiesgoAlto => 4,
        }
    }

    /// Perfil de `verificacion` del suelo (columna «Perfil»).
    pub fn perfil(self) -> Perfil {
        match self {
            Self::Errata | Self::Fix | Self::Funcionalidad | Self::RiesgoAlto => Perfil::Completa,
        }
    }
}

/// Perfil de verificacion exigido por la politica (F1-10, columna «Perfil»;
/// F1-7), de menor a mayor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub enum Perfil {
    /// Atajo local, no es puerta.
    #[serde(rename = "rapida")]
    Rapida,
    /// Tests afectados.
    #[serde(rename = "impacto")]
    Impacto,
    /// La suite completa.
    #[serde(rename = "completa")]
    Completa,
    /// La completa y los tests pesados.
    #[serde(rename = "completa y pesada")]
    CompletaYPesada,
}

/// Nombres de los perfiles, en orden.
pub(crate) const NOMBRES_PERFIL: [&str; 4] = ["rapida", "impacto", "completa", "completa y pesada"];

impl Perfil {
    /// Los cuatro perfiles, en orden.
    pub const TODOS: [Self; 4] = [
        Self::Rapida,
        Self::Impacto,
        Self::Completa,
        Self::CompletaYPesada,
    ];

    /// Nombre en la configuracion.
    pub fn nombre(self) -> &'static str {
        match self {
            Self::Rapida => NOMBRES_PERFIL[0],
            Self::Impacto => NOMBRES_PERFIL[1],
            Self::Completa => NOMBRES_PERFIL[2],
            Self::CompletaYPesada => NOMBRES_PERFIL[3],
        }
    }

    /// Los perfiles que no bajan de este (este incluido), por nombre.
    pub(crate) fn y_mayores(self) -> &'static [&'static str] {
        let i = Self::TODOS.iter().position(|p| *p == self).unwrap_or(0);
        NOMBRES_PERFIL.get(i..).unwrap_or(&NOMBRES_PERFIL)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nombres_y_orden() {
        for c in Carril::TODOS {
            assert_eq!(Carril::desde(c.nombre()), Some(c));
        }
        assert_eq!(Carril::desde("release"), None);
        assert!(Carril::Errata < Carril::RiesgoAlto);
        assert_eq!(
            Perfil::Completa.y_mayores(),
            ["completa", "completa y pesada"]
        );
        for p in Perfil::TODOS {
            assert_eq!(p.y_mayores().first(), Some(&p.nombre()));
        }
    }
}
