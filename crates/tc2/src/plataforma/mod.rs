//! Puerto `Plataforma` (ADR F1-4 decision 1, F1-13 D1a) y sus adaptadores
//! de GitHub (F2-6) y GitLab (F2-9).
//!
//! El puerto es la unica parte del CLI que habla con la plataforma, por su
//! API HTTP y sin `gh` (F1-14). Sus operaciones forman una lista cerrada y
//! devuelven el contrato de salida de [`crate::contrato`]; cada una corre con
//! la identidad que le da F1-6 y falla cerrada sin ella. Los dos adaptadores
//! comparten el transporte ([`conexion`]) y la evaluacion ([`puertas`]), y
//! dan la misma linea byte a byte (RR-15).
//!
//! Se usa desde la biblioteca: el binario no gana subcomandos hasta que los
//! comandos de fase (F6-2 `review`, F7 `finish`, F3-8 `doctor`) lo llamen.

pub mod conexion;
pub mod credenciales;
pub mod deteccion;
pub mod errores;
pub mod github;
pub mod gitlab;
pub mod http;
pub mod puertas;

use std::collections::BTreeSet;

pub use credenciales::{Credenciales, CredencialesEntorno, CredencialesFijas, Rol, Secreto};
pub use deteccion::{Destino, detectar};
pub use github::GitHub;
pub use gitlab::GitLab;
pub use puertas::{EstadoCheck, Lecturas, evaluar};

use crate::contrato::Informe;

/// Resultado de una lectura tipada: el `Err` ya es el informe que se
/// devuelve.
pub type Paso<T> = Result<T, Informe>;

/// Plataforma soportada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Proveedor {
    /// GitHub.
    GitHub,
    /// GitLab.
    GitLab,
}

impl Proveedor {
    /// Nombre en la configuracion y en la linea.
    pub fn nombre(self) -> &'static str {
        match self {
            Self::GitHub => "github",
            Self::GitLab => "gitlab",
        }
    }

    /// Nombre para la persona, en los diagnosticos.
    pub fn titulo(self) -> &'static str {
        match self {
            Self::GitHub => "GitHub",
            Self::GitLab => "GitLab",
        }
    }
}

/// Lista de personas por id numerico (F1-6, decision 3) y, para GitLab, los
/// ids de sus claves de firma (F1-13, decision 6: la firma de GitLab no trae
/// el usuario). La lee F2-13 del fichero de `.taskcode/`; aqui llega ya
/// leida.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Personas {
    ids: BTreeSet<u64>,
    claves: BTreeSet<u64>,
}

impl Personas {
    /// Lista con estos ids.
    pub fn nuevas(ids: impl IntoIterator<Item = u64>) -> Self {
        Self {
            ids: ids.into_iter().collect(),
            claves: BTreeSet::new(),
        }
    }

    /// La misma lista con estas claves de firma (ids de clave GPG o SSH de
    /// la plataforma).
    #[must_use]
    pub fn con_claves(mut self, claves: impl IntoIterator<Item = u64>) -> Self {
        self.claves = claves.into_iter().collect();
        self
    }

    /// Si el id es de una persona de la lista.
    pub fn contiene(&self, id: u64) -> bool {
        self.ids.contains(&id)
    }

    /// Si la clave de firma es de una persona de la lista.
    pub fn contiene_clave(&self, clave: u64) -> bool {
        self.claves.contains(&clave)
    }
}

/// Cuenta de la plataforma.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Usuario {
    /// Nombre de usuario.
    pub login: String,
    /// Id numerico (el que se compara, no el login).
    pub id: u64,
    /// `User` o `Bot` (en GitLab, `Bot` si la cuenta es `bot: true`).
    pub tipo: String,
}

impl Usuario {
    /// Persona que cuenta: `User`, en la lista y no `web-flow` (F1-6,
    /// decision 3, reglas 2, 3, 4 y 6).
    pub fn es_persona(&self, personas: &Personas) -> bool {
        self.tipo == "User" && self.login != "web-flow" && personas.contiene(self.id)
    }
}

/// Checks obligatorios con su origen y garantias que la politica exige
/// impuestas por la plataforma (F1-13, decision 5).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModeloPuertas {
    /// `(nombre, origen)`: un check solo cuenta si lo publico su origen
    /// (G09). En GitHub el origen es el `integration_id` de la App; en
    /// GitLab, el `author.id` del estado del commit (F1-13, decision 6).
    pub checks: Vec<(String, u64)>,
    /// Ids de garantia (`G03`...) que tienen que estar impuestas.
    pub exigir_impuestas: Vec<String>,
}

/// Firma de un commit (contrato del firmante, F1-6 decision 3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Firma {
    /// Firma verificada por la plataforma.
    pub verificada: bool,
    /// Motivo o estado que da la plataforma.
    pub motivo: String,
    /// La cuenta `committer`, si la plataforma la asocia (GitHub).
    pub committer: Option<Usuario>,
    /// Id de la clave que firmo, si la plataforma lo da (GitLab).
    pub clave: Option<u64>,
}

impl Firma {
    /// Si el commit cuenta como aprobacion de una persona: firma verificada
    /// y committer persona de la lista o clave de una persona de la lista
    /// (reglas 1 a 4; la 5, `Plan-Blob`, es de quien llama).
    pub fn cuenta_como_aprobacion(&self, personas: &Personas) -> bool {
        self.verificada
            && (self
                .committer
                .as_ref()
                .is_some_and(|c| c.es_persona(personas))
                || self.clave.is_some_and(|k| personas.contiene_clave(k)))
    }
}

/// Reglas leidas de la plataforma (F1-5, `leer-puertas`). La comparacion con
/// el modelo es de F3-8.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReglasLeidas {
    /// `false` si el plan no da las reglas (GitHub: 403 con aviso de plan
    /// en los rulesets; GitLab: 403 en las reglas de aprobacion). En GitLab
    /// Free el endpoint da 200 con la lista vacia (observado en el E2E de
    /// F2-9), asi que `true` no prueba que el plan sea Premium: lo afina
    /// F3-8.
    pub plan_da_reglas: bool,
    /// Nombres de los rulesets (GitHub) o de las reglas de aprobacion
    /// (GitLab).
    pub rulesets: Vec<String>,
    /// Reglas que aplican a la rama: tipos de regla (GitHub) o la
    /// proteccion de la rama (GitLab: `push=N`, `merge=N`, `force-push`,
    /// `code-owner`).
    pub reglas_rama: Vec<String>,
}

/// Puerto `Plataforma`: lista cerrada de operaciones (F1-4, F1-5, F1-6,
/// F1-13). Las escrituras de puertas (`aplicar-puertas`) son de F3-1.
pub trait Plataforma {
    /// `abrir-pr` (agente): abre el PR de `rama` contra `base`, o devuelve
    /// el que ya haya abierto (`sin-cambios`).
    fn abrir_pr(&self, rama: &str, base: &str, titulo: &str, cuerpo: &str) -> Informe;

    /// `estado-puertas` (agente o persona): checks con su origen,
    /// aprobaciones de personas y rama al dia.
    fn estado_puertas(&self, rol: Rol, pr: u64, modelo: &ModeloPuertas) -> Informe;

    /// `merge-pr` (persona): merge commit del PR si su head sigue siendo
    /// `sha` (el evaluado).
    fn merge_pr(&self, pr: u64, sha: &str) -> Informe;

    /// `leer-puertas` (agente o persona).
    fn leer_puertas(&self, rol: Rol, rama: &str) -> Paso<ReglasLeidas>;

    /// `leer-permisos-ci` (persona): que el token de la CI no pueda aprobar.
    fn leer_permisos_ci(&self) -> Informe;

    /// Firma del commit de `approve` (agente).
    fn firma_commit(&self, sha: &str) -> Paso<Firma>;

    /// Usuario autenticado con el token de ese rol.
    fn identidad(&self, rol: Rol) -> Paso<Usuario>;
}
