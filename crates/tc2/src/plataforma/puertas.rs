//! Evaluacion neutral de las puertas (F1-5 y F1-13, PoC M): de lo leido en
//! la plataforma a la linea `estado-puertas ...`. Es pura y comun a GitHub y
//! GitLab, para que las dos den la misma linea byte a byte.

use crate::contrato::{Informe, Resultado};

/// Nombre de la operacion.
pub const ESTADO_PUERTAS: &str = "estado-puertas";

/// Estado de un check obligatorio, contado solo si lo publico su origen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoCheck {
    /// Terminado en verde.
    Ok,
    /// Sin terminar o sin publicar por su origen.
    Pendiente,
    /// Terminado en rojo.
    Fallo,
}

/// Lo leido de un PR abierto, ya traducido al modelo neutral.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lecturas {
    /// Numero del PR (o iid del MR).
    pub pr: u64,
    /// Checks obligatorios con su estado, en el orden del modelo.
    pub checks: Vec<(String, EstadoCheck)>,
    /// Aprobaciones de personas de la lista.
    pub aprobaciones: usize,
    /// Aprobaciones que no cuentan (bots, cuentas fuera de la lista).
    pub ignoradas: usize,
    /// Si la rama esta al dia con la base.
    pub al_dia: bool,
    /// Si la rama se puede mergear sin conflictos.
    pub sin_conflictos: bool,
}

/// `EstadoPuertas` neutral: rojo si un check fallo; esperando si falta algo;
/// ok si no falta nada.
pub fn evaluar(l: &Lecturas) -> Informe {
    let pr = l.pr.to_string();
    let mut rojo: Vec<String> = l
        .checks
        .iter()
        .filter(|(_, e)| *e == EstadoCheck::Fallo)
        .map(|(n, _)| n.clone())
        .collect();
    if !rojo.is_empty() {
        rojo.sort();
        return Informe::nuevo(ESTADO_PUERTAS, Resultado::Rechazada)
            .con("pr", pr)
            .con("motivo", "check-rojo")
            .con_lista("checks", rojo)
            .diagnostico("hay checks obligatorios en rojo: corrige y vuelve a empujar");
    }
    let mut falta: Vec<String> = l
        .checks
        .iter()
        .filter(|(_, e)| *e != EstadoCheck::Ok)
        .map(|(n, _)| n.clone())
        .collect();
    falta.sort();
    if l.aprobaciones < 1 {
        falta.push("aprobacion".into());
    }
    if !l.al_dia {
        falta.push("al-dia".into());
    }
    if !l.sin_conflictos {
        falta.push("sin-conflictos".into());
    }
    let resultado = if falta.is_empty() {
        Resultado::Ok
    } else {
        Resultado::Esperando
    };
    let i = Informe::nuevo(ESTADO_PUERTAS, resultado)
        .con("pr", pr)
        .con("aprobaciones", l.aprobaciones.to_string())
        .con("ignoradas", l.ignoradas.to_string());
    if falta.is_empty() {
        i
    } else {
        let diag = format!(
            "faltan puertas: {}; espera o atiende lo que falta (al-dia o sin-conflictos: lanza actualizar-rama)",
            falta.join(", ")
        );
        i.con_lista("falta", falta).diagnostico(diag)
    }
}

/// PR ya mergeado: nada que evaluar.
pub fn mergeado(pr: u64) -> Informe {
    Informe::nuevo(ESTADO_PUERTAS, Resultado::SinCambios)
        .con("pr", pr.to_string())
        .con("estado", "mergeado")
}

/// PR cerrado sin mergear.
pub fn pr_cerrado(pr: u64) -> Informe {
    Informe::nuevo(ESTADO_PUERTAS, Resultado::Rechazada)
        .con("pr", pr.to_string())
        .con("motivo", "pr-cerrado")
        .diagnostico("el PR esta cerrado sin mergear: reabrelo o abre otro")
}

/// La politica exige impuesta una garantia que el plan no da.
pub fn rechazada_por_plan(pr: u64, garantia: &str) -> Informe {
    Informe::nuevo(ESTADO_PUERTAS, Resultado::Rechazada)
        .con("pr", pr.to_string())
        .con("motivo", "plan")
        .con("garantia", garantia)
        .diagnostico(format!(
            "la politica del proyecto exige {garantia} impuesta y el plan de la plataforma no la da: cambia de plan o quita {garantia} de la politica"
        ))
}

/// La plataforma no tiene configurada una garantia que se lee de su
/// configuracion (GitLab, G01: rama destino protegida, merge solo de
/// Maintainers y sin force-push).
pub fn rechazada_por_config(pr: u64, garantia: &str) -> Informe {
    Informe::nuevo(ESTADO_PUERTAS, Resultado::Rechazada)
        .con("pr", pr.to_string())
        .con("motivo", "config")
        .con("garantia", garantia)
        .diagnostico(format!(
            "la rama destino no cumple {garantia}: protegela con merge solo para Maintainers y sin force-push (taskctl setup lo propone)"
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn l(checks: &[(&str, EstadoCheck)], aprob: usize, al_dia: bool) -> Lecturas {
        Lecturas {
            pr: 7,
            checks: checks.iter().map(|(n, e)| ((*n).to_owned(), *e)).collect(),
            aprobaciones: aprob,
            ignoradas: 0,
            al_dia,
            sin_conflictos: al_dia,
        }
    }

    #[test]
    fn rojo_gana_a_lo_que_falta() {
        let i = evaluar(&l(
            &[("b", EstadoCheck::Fallo), ("a", EstadoCheck::Fallo)],
            0,
            false,
        ));
        assert_eq!(
            i.linea(),
            "estado-puertas rechazada pr=7 motivo=check-rojo checks=a,b"
        );
        let i = evaluar(&l(&[("c", EstadoCheck::Pendiente)], 0, false));
        assert_eq!(
            i.linea(),
            "estado-puertas esperando pr=7 aprobaciones=0 ignoradas=0 falta=c,aprobacion,al-dia,sin-conflictos"
        );
        assert_eq!(i.resultado.codigo(), 7);
        let i = evaluar(&l(&[("c", EstadoCheck::Ok)], 1, true));
        assert_eq!(
            i.linea(),
            "estado-puertas ok pr=7 aprobaciones=1 ignoradas=0"
        );
    }
}
