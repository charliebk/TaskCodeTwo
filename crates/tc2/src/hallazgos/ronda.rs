//! Rondas incrementales del revisor IA (F6-6; RF-33).
//!
//! Funciones puras: la ronda siguiente se decide con el check `revision-ia`
//! anterior, el que publico la App del revisor (no la rama de estado, que
//! escribe el agente: F1-8). Cada check lleva su ronda en el titulo y todos
//! los hallazgos que siguen abiertos en sus anotaciones.

use serde::{Deserialize, Serialize};

use super::AceptadoDe;

/// Como se revisa esta ronda.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Modo {
    /// Ronda 1 (o sin ronda anterior valida): todo el diff contra la base.
    Completa,
    /// El diff desde el commit de la ronda anterior y sus abiertos.
    Incremental,
    /// Anterior en verde y los cambios solo tocan ficheros con MENOR
    /// abiertos: verificacion determinista, sin modelo; la ronda no sube.
    Ligera,
    /// Pasado el tope de rondas con un CRITICO o IMPORTANTE abierto: sin
    /// modelo, los abiertos se arrastran y el check sale `tope`.
    Tope,
}

impl Modo {
    /// Como se escribe en la linea y en `ronda.json`.
    pub fn texto(self) -> &'static str {
        match self {
            Modo::Completa => "completa",
            Modo::Incremental => "incremental",
            Modo::Ligera => "ligera",
            Modo::Tope => "tope",
        }
    }
}

/// La decision de la ronda: `ronda.json` (`$defs/ronda` de
/// `hallazgos.schema.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    pub version: u32,
    pub ronda: u32,
    pub modo: Modo,
    /// Commit del check anterior (base del diff incremental).
    pub sha_anterior: Option<String>,
    /// Tope efectivo de rondas.
    pub tope: u32,
    /// Abiertos heredados que siguen abiertos.
    pub abiertos: Vec<AceptadoDe>,
    /// MENOR heredados que se cierran porque su fichero cambio.
    pub cerrados: Vec<AceptadoDe>,
}
