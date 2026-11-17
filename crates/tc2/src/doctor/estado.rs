//! Rama de estado e instalacion.
//!
//! - La rama de estado se lee sobre la misma ref que `board`, pero sin
//!   `fetch` (lo ultimo traido; D4a de F3-5b: `doctor` solo lee):
//!   ausente, `falla motivo=sin-rama-de-estado`; una ref que existe y no es
//!   un commit, `falla motivo=ref-ilegible ref=...`; un `tarea.md` que no se
//!   entiende, `falla motivo=tarea-ilegible ids=...`.
//! - La instalacion es el inventario de `tc2 init` en simulacion, sin
//!   credenciales (no llama a la API: los rulesets salen `pendiente
//!   motivo=sin-token` y los mira [`super::plataforma`]). Cada accion que
//!   no sale `existe` es un hallazgo, asi `doctor` e `init` no discrepan.

use std::path::{Path, PathBuf};

use super::{ARREGLO_INIT, Estado, Hallazgo, NOMBRE};
use crate::contrato::corto;
use crate::estado;
use crate::git::{Git, Subcomando, correr};
use crate::init::{self, Accion, Decision};
use crate::plataforma::{Credenciales, CredencialesFijas};

/// La ref de la rama de estado que lee `board` (la misma que
/// [`estado::leer`]) y la carpeta principal.
fn referencia(g: &Git, dir: &Path) -> Option<(PathBuf, String)> {
    estado::referencia_de_lectura(g, dir).ok()
}

/// Si `r` existe (apunte o no a un commit).
fn existe_ref(g: &Git, principal: &Path, r: &str) -> bool {
    correr(
        g,
        principal,
        NOMBRE,
        Subcomando::RevParse,
        &["-q", "--verify", "--end-of-options", r],
    )
    .is_ok_and(|s| s.exito())
}

/// Hallazgos de la rama de estado y el commit leido (12).
pub(super) fn rama(g: &Git, dir: &Path) -> (Vec<Hallazgo>, Option<String>) {
    let h = |e: Estado| Hallazgo::nuevo("estado", "rama", e);
    let (principal, r) = referencia(g, dir).unwrap_or_else(|| (dir.to_path_buf(), String::new()));
    let vista = match estado::leer(g, dir) {
        Ok(v) => v,
        Err(i) if i.valor("motivo") == Some("sin-rama-de-estado") => {
            let x = if existe_ref(g, &principal, &r) {
                h(Estado::Falla).motivo("ref-ilegible").con("ref", r)
            } else {
                h(Estado::Falla)
                    .motivo("sin-rama-de-estado")
                    .arreglo(ARREGLO_INIT)
            };
            return (vec![x], None);
        }
        Err(i) => {
            let m = i
                .valor("motivo")
                .map_or_else(|| "ref-ilegible".to_owned(), str::to_owned);
            return (vec![h(Estado::Falla).motivo(m).con("ref", r)], None);
        }
    };
    let commit = vista.commit().map(corto);
    let mut v = vec![h(Estado::Ok).con("ref", r.clone())];
    let t = |e: Estado| Hallazgo::nuevo("estado", "tareas", e);
    v.push(match crate::situacion::calcular(&vista) {
        Ok(s) if s.ilegibles.is_empty() => t(Estado::Ok),
        Ok(s) => t(Estado::Falla)
            .motivo("tarea-ilegible")
            .con("ids", s.ilegibles.join(",")),
        Err(_) => t(Estado::Falla).motivo("estado-ilegible").con("ref", r),
    });
    (v, commit)
}

/// Un hallazgo por accion del plan de `init` (sin los rulesets).
pub(super) fn instalacion(g: &Git, dir: &Path) -> Vec<Hallazgo> {
    let sin_token = || -> Box<dyn Credenciales> { Box::new(CredencialesFijas::nuevas()) };
    let c = init::Contexto {
        dir,
        git: g,
        credenciales: &sin_token,
    };
    match init::plan(&c) {
        Ok(acciones) => acciones
            .iter()
            .filter(|a| a.clase != "ruleset")
            .map(de_accion)
            .collect(),
        // Sin red (o denegada) el inventario no se puede hacer entero: no se
        // dice nada de la instalacion que no se haya leido.
        Err(i) => vec![
            Hallazgo::nuevo("instalacion", "plan", Estado::NoComprobado).motivo(
                match i.valor("motivo") {
                    Some(m) => m.to_owned(),
                    None => i.resultado.texto().to_owned(),
                },
            ),
        ],
    }
}

/// El hallazgo de una accion del plan de `init` (funcion pura).
pub fn de_accion(a: &Accion) -> Hallazgo {
    let h = |e: Estado| Hallazgo::nuevo("instalacion", a.clase, e).con("ruta", a.ruta.clone());
    let motivo = a.motivo.clone().unwrap_or_default();
    match a.decision {
        Decision::Existe => h(Estado::Ok),
        Decision::Crea => h(Estado::Aviso).motivo("falta").arreglo(ARREGLO_INIT),
        // Sin token `init` no lee la persona: lo que falta lo crea `init`.
        Decision::Pendiente if motivo == "sin-token" => {
            h(Estado::Aviso).motivo("falta").arreglo(ARREGLO_INIT)
        }
        Decision::Pendiente => h(Estado::Aviso).motivo(motivo),
        Decision::Persona => {
            let x = h(Estado::Aviso).motivo(motivo);
            match &a.orden {
                Some(o) => x.arreglo(o.clone()),
                None => x,
            }
        }
        Decision::Rechaza => {
            let x = h(Estado::Falla).motivo(motivo);
            match &a.orden {
                Some(o) => x.arreglo(o.clone()),
                None => x,
            }
        }
    }
}
