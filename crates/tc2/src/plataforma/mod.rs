//! Puerto `Plataforma` (ADR F1-4 decision 1, F1-13 D1a) y su adaptador de
//! GitHub (F2-6).
//!
//! El puerto es la unica parte del CLI que habla con la plataforma, por su
//! API HTTP y sin `gh` (F1-14). Sus operaciones forman una lista cerrada y
//! devuelven el contrato de salida de [`crate::contrato`]; cada una corre con
//! la identidad que le da F1-6 y falla cerrada sin ella. GitLab (F2-9)
//! implementa el mismo trait y pasa la misma bateria.
//!
//! Se usa desde la biblioteca: el binario no gana subcomandos hasta que los
//! comandos de fase (F6-2 `review`, F7 `finish`, F3-8 `doctor`) lo llamen.

pub mod credenciales;
pub mod deteccion;
pub mod errores;
pub mod github;
pub mod http;
pub mod puertas;

use std::collections::BTreeSet;

pub use credenciales::{Credenciales, CredencialesEntorno, CredencialesFijas, Rol, Secreto};
pub use deteccion::{Destino, Proveedor, detectar};
pub use github::GitHub;
pub use puertas::{EstadoCheck, Lecturas, evaluar};

use crate::contrato::Informe;

/// Resultado de una lectura tipada: el `Err` ya es el informe que se
/// devuelve.
pub type Paso<T> = Result<T, Informe>;

/// Lista de personas por id numerico (F1-6, decision 3). La lee F2-13 del
/// fichero de `.taskcode/`; aqui llega ya leida.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Personas(BTreeSet<u64>);

impl Personas {
    /// Lista con estos ids.
    pub fn nuevas(ids: impl IntoIterator<Item = u64>) -> Self {
        Self(ids.into_iter().collect())
    }

    /// Si el id es de una persona de la lista.
    pub fn contiene(&self, id: u64) -> bool {
        self.0.contains(&id)
    }
}

/// Cuenta de la plataforma.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Usuario {
    /// Nombre de usuario.
    pub login: String,
    /// Id numerico (el que se compara, no el login).
    pub id: u64,
    /// `User` o `Bot` (GitHub).
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
    /// `(nombre, integration_id)`: un check solo cuenta si lo publico esa
    /// App (G09).
    pub checks: Vec<(String, u64)>,
    /// Ids de garantia (`G03`...) que tienen que estar impuestas.
    pub exigir_impuestas: Vec<String>,
}

/// Firma de un commit (contrato del firmante, F1-6 decision 3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Firma {
    /// `commit.verification.verified`.
    pub verificada: bool,
    /// `commit.verification.reason`.
    pub motivo: String,
    /// La cuenta `committer`, si GitHub la asocia.
    pub committer: Option<Usuario>,
}

impl Firma {
    /// Si el commit cuenta como aprobacion de una persona: firma verificada
    /// y committer persona de la lista (reglas 1 a 4; la 5, `Plan-Blob`, es
    /// de quien llama).
    pub fn cuenta_como_aprobacion(&self, personas: &Personas) -> bool {
        self.verificada
            && self
                .committer
                .as_ref()
                .is_some_and(|c| c.es_persona(personas))
    }
}

/// Reglas leidas de la plataforma (F1-5, `leer-puertas`). La comparacion con
/// el modelo es de F3-8.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReglasLeidas {
    /// `false` si el plan no da rulesets (403 con aviso de plan).
    pub plan_da_reglas: bool,
    /// Nombres de los rulesets del repo.
    pub rulesets: Vec<String>,
    /// Tipos de regla que aplican a la rama.
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
