//! Carriles de la politica (ADR F1-10, decision 1): el sitio unico de la
//! tabla de suelo. F2-7 solo necesita las columnas que valida la
//! configuracion (roles de diseno minimos y perfil); F4 y F6 amplian este
//! modulo en vez de copiar la tabla. F4-1 (`plan`) anade el carril
//! declarado por el tipo y cuantos roles de diseno se lanzan.

use serde::Deserialize;

use crate::config::Config;

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

    /// Carril declarado por el tipo de la tarea (F1-10, PoC C): `fix` para
    /// el tipo `fix` y `funcionalidad` para el resto. `errata` no sale del
    /// tipo (la crea su orden, D5a) y riesgo alto solo se pide o se deriva
    /// del diff (F4-1 D4).
    pub fn declarado(tipo: &str) -> Self {
        if tipo == "fix" {
            Self::Fix
        } else {
            Self::Funcionalidad
        }
    }

    /// Cuantos roles de diseno lanza `plan` (F4-1, columna «Roles de
    /// diseno»): errata 0, fix 1 (D2), funcionalidad los de la complejidad
    /// (`por_complejidad`, de la heuristica de la 0.16, D5) y como poco
    /// [`Carril::roles_min`], riesgo alto los cuatro de base (D1). Un
    /// `hotfix` fuera de riesgo alto se queda en 1 como mucho (el tope de la
    /// 0.16). Despues, `politica_carriles.<carril>.roles_diseno` solo sube
    /// (F1-10 decision 2: la configuracion endurece, nunca baja); el
    /// esquema lo deja en 5 como mucho.
    pub fn roles_diseno(self, por_complejidad: u8, tipo: &str, config: &Config) -> u8 {
        let suelo = u8::try_from(self.roles_min()).unwrap_or(u8::MAX);
        let mut n = match self {
            Self::Errata => 0,
            Self::Fix => 1,
            Self::Funcionalidad => por_complejidad.max(suelo),
            Self::RiesgoAlto => suelo,
        };
        if tipo == "hotfix" && self != Self::RiesgoAlto {
            n = n.min(TOPE_HOTFIX);
        }
        let minimo = config
            .politica_carriles
            .get(&self)
            .and_then(|p| p.roles_diseno)
            .unwrap_or(0);
        n.max(minimo)
    }
}

/// Roles de diseno de un `hotfix` como mucho (`agentes_brainstorm_hotfix` de
/// la 0.16): abrevia el brainstorm previo, nunca la revision.
const TOPE_HOTFIX: u8 = 1;

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

    /// Criterio 2 de TASK-036 (tabla): `roles_diseno` y la seleccion por
    /// carril x roles por complejidad x tipo (hotfix) x etiqueta de
    /// rendimiento x `politica_carriles`. Uno en fix, todos en riesgo alto.
    #[test]
    fn roles_de_diseno_por_carril() {
        use crate::config::PoliticaCarril;
        use crate::plan::roles::seleccionar;
        let suelo = Config::suelo();
        let con = |c: Carril, n: u8| {
            let mut x = Config::suelo();
            x.politica_carriles.insert(
                c,
                PoliticaCarril {
                    roles_diseno: Some(n),
                    plan_aprobado: None,
                    paradas_extra: Vec::new(),
                    revisoras_extra: Vec::new(),
                    lote: None,
                    perfil: None,
                },
            );
            x
        };
        let rend = vec!["rendimiento".to_owned()];
        let nada: Vec<String> = Vec::new();
        let fix3 = con(Carril::Fix, 3);
        let func5 = con(Carril::Funcionalidad, 5);
        let otro = con(Carril::RiesgoAlto, 5);
        // (carril, por complejidad, tipo, config, etiquetas, roles, ids)
        type Caso<'a> = (Carril, u8, &'a str, &'a Config, &'a Vec<String>, u8, usize);
        let casos: [Caso<'_>; 16] = [
            (Carril::Errata, 4, "feature", &suelo, &nada, 0, 0),
            (Carril::Fix, 0, "fix", &suelo, &nada, 1, 1),
            (Carril::Fix, 4, "fix", &suelo, &rend, 1, 1),
            (Carril::Fix, 0, "fix", &fix3, &nada, 3, 3),
            (Carril::Fix, 0, "fix", &fix3, &rend, 3, 4),
            (Carril::Fix, 0, "fix", &otro, &nada, 1, 1),
            (Carril::Funcionalidad, 0, "feature", &suelo, &nada, 1, 1),
            (Carril::Funcionalidad, 1, "feature", &suelo, &rend, 1, 1),
            (Carril::Funcionalidad, 2, "feature", &suelo, &rend, 2, 3),
            (Carril::Funcionalidad, 4, "feature", &suelo, &nada, 4, 4),
            (Carril::Funcionalidad, 4, "hotfix", &suelo, &nada, 1, 1),
            (Carril::Funcionalidad, 0, "feature", &func5, &nada, 5, 5),
            (Carril::RiesgoAlto, 0, "feature", &suelo, &nada, 4, 4),
            (Carril::RiesgoAlto, 0, "hotfix", &suelo, &nada, 4, 4),
            (Carril::RiesgoAlto, 0, "fix", &suelo, &rend, 4, 5),
            (Carril::RiesgoAlto, 0, "feature", &otro, &nada, 5, 5),
        ];
        for (c, complejidad, tipo, cfg, et, roles, ids) in casos {
            let n = c.roles_diseno(complejidad, tipo, cfg);
            assert_eq!(n, roles, "{c:?} {complejidad} {tipo}");
            assert_eq!(
                seleccionar(n, et).len(),
                ids,
                "{c:?} {complejidad} {tipo} {et:?}"
            );
        }
        assert_eq!(Carril::declarado("fix"), Carril::Fix);
        for t in ["feature", "hotfix", "release", "otro"] {
            assert_eq!(Carril::declarado(t), Carril::Funcionalidad, "{t}");
        }
    }
}
