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
pub mod personas;
pub mod puertas;

pub use credenciales::{Credenciales, CredencialesEntorno, CredencialesFijas, Rol, Secreto};
pub use deteccion::{Destino, detectar};
pub use github::GitHub;
pub use gitlab::GitLab;
pub use personas::{Clave, Persona, Personas};
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
    pub clave: Option<Clave>,
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
    /// Aprobaciones que pide la plataforma (F3-8): GitHub, el
    /// `required_approving_review_count` de la regla `pull_request` de la
    /// rama; GitLab, el mayor `approvals_required` de las reglas de
    /// aprobacion. 0 si no pide ninguna.
    pub aprobaciones: u64,
    /// Checks que la rama exige por nombre (F3-8; GitHub: los `context` de
    /// `required_status_checks`). `None` si no se pudo leer.
    pub checks_requeridos: Option<Vec<String>>,
    /// GitLab con «Pipelines must succeed»: cuenta todo el pipeline, asi
    /// que cualquier check publicado en el es obligatorio (sin origen: G09
    /// sustituido por `autor-del-estado`).
    pub todos_los_checks: bool,
    /// Nombres de los checks publicados en la cabeza de la rama (GitHub,
    /// check-runs; GitLab, estados del commit). `None` si no se pudieron
    /// leer (rama inexistente, permisos...: el motivo, en
    /// [`ReglasLeidas::fallo_checks`]) o no se pidieron
    /// ([`Plataforma::leer_puertas_de`]).
    pub checks_publicados: Option<Vec<String>>,
    /// Por que no se leyeron los checks publicados, si se pidieron y fallo
    /// la lectura (F3-8, H3 de TASK-035): un 403 da su permiso.
    pub fallo_checks: Option<FalloLectura>,
    /// GitHub: alguna regla `pull_request` de la rama tiene
    /// `dismiss_stale_reviews_on_push` (G04). En GitLab no se lee.
    pub descartar_viejas: bool,
    /// GitHub: alguna regla `pull_request` de la rama tiene
    /// `require_last_push_approval` (G05). En GitLab no se lee.
    pub ultimo_push: bool,
}

/// Por que fallo una lectura secundaria que no tumba la operacion (los
/// checks publicados de `leer-puertas`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FalloLectura {
    /// `permiso-insuficiente` en un 403; si no, el `motivo` del informe (o
    /// su resultado).
    pub motivo: String,
    /// El permiso que falta, en un 403.
    pub permiso: Option<&'static str>,
}

impl FalloLectura {
    /// El fallo de `i`, con `permiso` si es un 403 de permisos.
    pub fn de(i: &Informe, permiso: &'static str) -> Self {
        let denegada = i.resultado == crate::contrato::Resultado::Denegada
            && i.valor("motivo") == Some("permisos");
        if denegada {
            Self {
                motivo: "permiso-insuficiente".to_owned(),
                permiso: Some(permiso),
            }
        } else {
            Self {
                motivo: i
                    .valor("motivo")
                    .unwrap_or_else(|| i.resultado.texto())
                    .to_owned(),
                permiso: None,
            }
        }
    }

    /// Un fallo sin informe (respuesta que no se entiende).
    pub fn motivo(m: &str) -> Self {
        Self {
            motivo: m.to_owned(),
            permiso: None,
        }
    }
}

/// `null` como el valor por defecto (H5 de TASK-035): `#[serde(default)]`
/// solo cubre el campo ausente.
pub(crate) fn nulo_por_defecto<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Default + serde::Deserialize<'de>,
{
    use serde::Deserialize;
    Ok(Option::<T>::deserialize(d)?.unwrap_or_default())
}

/// Tier de GitLab (F1-13, decision 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TierGitLab {
    /// Free.
    Free,
    /// Premium.
    Premium,
    /// Ultimate.
    Ultimate,
}

impl TierGitLab {
    /// El tier del `plan` de un namespace (`free`, `premium`, `ultimate`);
    /// cualquier otro valor (`default`, nombres antiguos) no se reconoce.
    pub fn de_plan(plan: &str) -> Option<Self> {
        match plan {
            "free" => Some(Self::Free),
            "premium" => Some(Self::Premium),
            "ultimate" => Some(Self::Ultimate),
            _ => None,
        }
    }

    /// Nombre en la linea.
    pub fn nombre(self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Premium => "premium",
            Self::Ultimate => "ultimate",
        }
    }
}

/// Lo que se observa del plan (F1-13 decision 3: «el plan se observa, no se
/// declara»). En GitHub, la visibilidad y el tipo de cuenta del repo (el
/// 403 de plan de los rulesets lo aporta `leer-puertas`); en GitLab, el
/// `plan` del namespace, si se lee y se reconoce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanObservado {
    /// Repo de GitHub.
    GitHub {
        /// Publico (ni privado ni interno).
        publico: bool,
        /// De una organizacion.
        organizacion: bool,
    },
    /// Proyecto de GitLab, con el tier si se reconoce.
    GitLab(Option<TierGitLab>),
}

/// Un PR (o MR) abierto del propio repo, con lo minimo que necesita `board`
/// (F3-5, D3 de TASK-032: sin checks ni aprobaciones).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrAbierto {
    /// Numero del PR (en GitLab, el `iid` del MR).
    pub numero: u64,
    /// Rama de origen.
    pub rama: String,
    /// Sha de la cabeza (40 hex).
    pub sha: String,
    /// Si es borrador.
    pub borrador: bool,
}

/// Puerto `Plataforma`: lista cerrada de operaciones (F1-4, F1-5, F1-6,
/// F1-13). Las escrituras de puertas (`aplicar-puertas`) son de F3-1.
pub trait Plataforma {
    /// `prs-abiertos` (agente o persona; adenda de TASK-032 a F1-4 y F1-13):
    /// los PR abiertos cuya rama de origen es del propio repo (los de forks
    /// no cuentan), ordenados por numero. Pagina hasta
    /// [`conexion::MAX_PAGINAS`]; con mas, falla cerrado.
    fn prs_abiertos(&self, rol: Rol) -> Paso<Vec<PrAbierto>>;

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

    /// `leer-puertas` de varias `ramas` a la vez (F3-8, R1-06 de
    /// TASK-035): lo comun al repo se lee una vez y los checks publicados
    /// solo en `checks_de` (en las demas, `checks_publicados: None`). Un
    /// resultado por rama, en el mismo orden. Por defecto, una a una.
    fn leer_puertas_de(
        &self,
        rol: Rol,
        ramas: &[&str],
        checks_de: &str,
    ) -> Vec<Paso<ReglasLeidas>> {
        let _ = checks_de;
        ramas.iter().map(|r| self.leer_puertas(rol, r)).collect()
    }

    /// `leer-permisos-ci` (persona): que el token de la CI no pueda aprobar.
    fn leer_permisos_ci(&self) -> Informe;

    /// Firma del commit de `approve` (agente).
    fn firma_commit(&self, sha: &str) -> Paso<Firma>;

    /// Usuario autenticado con el token de ese rol.
    fn identidad(&self, rol: Rol) -> Paso<Usuario>;

    /// La plataforma del adaptador (F3-8: `doctor` traduce lo leido con la
    /// tabla de su plataforma). `None` en los adaptadores de prueba.
    fn proveedor(&self) -> Option<Proveedor> {
        None
    }

    /// `leer-plan` (agente o persona; F3-8): lo que se observa del plan
    /// para `capacidades`. Por defecto no se sabe (adaptadores de prueba).
    fn leer_plan(&self, _rol: Rol) -> Paso<PlanObservado> {
        Err(
            Informe::nuevo("leer-plan", crate::contrato::Resultado::Error)
                .con("motivo", "no-soportada"),
        )
    }
}
