//! Nucleo puro de la estimacion (ADR F1-17, decisiones 3, 4 y 5):
//! [`estimar`] (cascada de tareas parecidas con mediana y p80 por fase y en
//! total), [`restante`] (lo que falta desde la fase en curso) y [`error`]
//! (estimado frente a real).
//!
//! - Cascada con al menos [`N_MIN`] muestras por nivel: carril, complejidad
//!   y tamano; carril y complejidad; carril; y repo. Sin semillas de otros
//!   proyectos (D3a de TASK-083): si ningun nivel llega, `sin-datos`, nunca
//!   un numero inventado.
//! - Carril declarado (D5a): una tarea sin carril solo se compara en `repo`.
//! - La tarea estimada nunca es muestra de si misma (C7).
//! - Mediana y p80 como `mediana` y `cuantil` de `poc-f1-17.py` (rango mas
//!   cercano, el de F1-18).

use super::historico::Muestra;
use super::real::{FASES, Real};
use crate::carril::Carril;

/// Muestras minimas de un nivel de la cascada (fijado antes de medir).
pub const N_MIN: usize = 5;

/// Cuantil que se ensena ademas de la mediana (D2a).
pub const P80: f64 = 0.8;

/// Nivel de una estimacion sin ningun nivel con [`N_MIN`] muestras.
pub const SIN_DATOS: &str = "sin-datos";

/// Niveles de la cascada, en orden.
pub const NIVELES: [&str; 4] = [
    "carril+complejidad+tamano",
    "carril+complejidad",
    "carril",
    "repo",
];

/// Lo que se sabe de la tarea al estimarla.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Objetivo {
    /// Su ID (nunca es muestra de si misma).
    pub id: String,
    /// Carril declarado.
    pub carril: Option<Carril>,
    /// Complejidad declarada.
    pub complejidad: Option<String>,
    /// Casillas de criterios de aceptacion.
    pub criterios: usize,
}

/// Mediana y p80, en minutos.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rango {
    /// Mediana.
    pub mediana: f64,
    /// Percentil 80 (rango mas cercano).
    pub p80: f64,
}

/// Rango por fase; una fase que no se conoce, `None`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PorFase {
    /// Implementacion.
    pub implementacion: Option<Rango>,
    /// Rondas.
    pub rondas: Option<Rango>,
    /// Cierre.
    pub cierre: Option<Rango>,
}

impl PorFase {
    /// La fase por su nombre.
    pub fn fase(&self, f: &str) -> Option<Rango> {
        match f {
            "implementacion" => self.implementacion,
            "rondas" => self.rondas,
            "cierre" => self.cierre,
            _ => None,
        }
    }
}

/// La estimacion de una tarea (forma del punto 7 de F1-17).
#[derive(Debug, Clone, PartialEq)]
pub struct Estimacion {
    /// Instante del calculo, `AAAA-MM-DDTHH:MM:SSZ`.
    pub calculada: String,
    /// Nivel de la cascada usado, o [`SIN_DATOS`].
    pub nivel: String,
    /// Muestras del nivel (con `sin-datos`, las disponibles).
    pub n: usize,
    /// Rango por fase; `None` con `sin-datos`.
    pub por_fase: Option<PorFase>,
    /// Rango del total; `None` con `sin-datos`.
    pub total: Option<Rango>,
    /// Estimacion del agente en el plan (D1a); `None` hasta que exista
    /// `plan` (F4-1).
    pub agente: Option<u64>,
}

impl Estimacion {
    /// Si no hubo tareas parecidas suficientes.
    pub fn sin_datos(&self) -> bool {
        self.nivel == SIN_DATOS
    }
}

/// Mediana de `xs` (media de las dos centrales si son pares).
pub fn mediana(xs: &[f64]) -> Option<f64> {
    let mut v = xs.to_vec();
    v.sort_by(f64::total_cmp);
    let n = v.len();
    match n {
        0 => None,
        _ if n % 2 == 1 => v.get(n / 2).copied(),
        _ => Some((v[n / 2 - 1] + v[n / 2]) / 2.0),
    }
}

/// Cuantil `q` por rango mas cercano: el valor en la posicion
/// `ceil(q * n)` de la lista ordenada (con la misma tolerancia que la PoC).
pub fn cuantil(xs: &[f64], q: f64) -> Option<f64> {
    let mut v = xs.to_vec();
    v.sort_by(f64::total_cmp);
    if v.is_empty() {
        return None;
    }
    let k = ((q * v.len() as f64) - 1e-9).ceil().max(1.0) as usize;
    v.get(k.min(v.len()) - 1).copied()
}

/// Mediana y p80 de una lista no vacia.
fn rango(xs: &[f64]) -> Option<Rango> {
    Some(Rango {
        mediana: mediana(xs)?,
        p80: cuantil(xs, P80)?,
    })
}

/// Tamano por criterios (1-3, 4-6, 7 o mas), el de la PoC.
fn tamano(criterios: usize) -> u8 {
    match criterios {
        0..=3 => 1,
        4..=6 => 2,
        _ => 3,
    }
}

/// Si `m` es parecida a `obj` en `nivel`. Sin carril declarado solo vale
/// `repo` (D5a); la complejidad se compara tal cual (`null` con `null`,
/// como la PoC).
fn parecida(obj: &Objetivo, m: &Muestra, nivel: &str) -> bool {
    let carril = obj.carril.is_some() && obj.carril == m.carril;
    match nivel {
        "carril+complejidad+tamano" => {
            carril
                && obj.complejidad == m.complejidad
                && tamano(obj.criterios) == tamano(m.criterios)
        }
        "carril+complejidad" => carril && obj.complejidad == m.complejidad,
        "carril" => carril,
        _ => true,
    }
}

/// Las muestras que puede usar `obj`: todas menos las que la contienen
/// (C7: nunca es muestra de si misma).
fn disponibles<'a>(obj: &Objetivo, muestras: &'a [Muestra]) -> Vec<&'a Muestra> {
    muestras
        .iter()
        .filter(|m| !m.miembros.contains(&obj.id) && m.id != obj.id)
        .collect()
}

/// El primer nivel de la cascada con [`N_MIN`] muestras parecidas y esas
/// muestras; `None` si ninguno llega.
pub fn parecidas<'a>(
    obj: &Objetivo,
    muestras: &'a [Muestra],
) -> Option<(&'static str, Vec<&'a Muestra>)> {
    let d = disponibles(obj, muestras);
    NIVELES.into_iter().find_map(|nivel| {
        let ms: Vec<&Muestra> = d
            .iter()
            .copied()
            .filter(|m| parecida(obj, m, nivel))
            .collect();
        (ms.len() >= N_MIN).then_some((nivel, ms))
    })
}

/// `estimar(tarea, muestras) -> Estimacion` (funcion pura): mediana y p80
/// por fase y en total del primer nivel con [`N_MIN`] muestras; si ninguno
/// llega, `sin-datos` con las muestras disponibles en `n`.
pub fn estimar(obj: &Objetivo, muestras: &[Muestra], calculada: &str) -> Estimacion {
    let Some((nivel, ms)) = parecidas(obj, muestras) else {
        return Estimacion {
            calculada: calculada.to_owned(),
            nivel: SIN_DATOS.to_owned(),
            n: disponibles(obj, muestras).len(),
            por_fase: None,
            total: None,
            agente: None,
        };
    };
    let de = |f: fn(&Real) -> f64| rango(&ms.iter().map(|m| f(&m.real)).collect::<Vec<_>>());
    Estimacion {
        calculada: calculada.to_owned(),
        nivel: nivel.to_owned(),
        n: ms.len(),
        por_fase: Some(PorFase {
            implementacion: de(|r| r.implementacion),
            rondas: de(|r| r.rondas),
            cierre: de(|r| r.cierre),
        }),
        total: de(Real::total),
        agente: None,
    }
}

/// Fase en curso de una tarea abierta (para [`restante`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fase {
    /// Antes de `start`: faltan las tres fases medibles.
    Diseno,
    /// De `start` a la primera `review`.
    Implementacion,
    /// De la primera `review` a `finish`.
    Rondas,
}

/// Lo que falta de una tarea (punto 5 de F1-17).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Restante {
    /// Mediana de lo que falta (con `desbordada`, cota inferior).
    pub mediana: f64,
    /// p80 de lo que falta (sin sentido con `desbordada`: no se ensena).
    pub p80: f64,
    /// Muestras usadas.
    pub n: usize,
    /// Si se condiciono a las parecidas que ya pasaron de lo transcurrido.
    pub condicionada: bool,
    /// Si la fase en curso va por encima de sus parecidas: lo que se da es
    /// solo una cota inferior (las fases siguientes).
    pub desbordada: bool,
}

/// `restante(parecidas, fase, transcurrido)`: la distribucion de lo que
/// falta entre las parecidas. Si al menos [`N_MIN`] ya pasaron de los
/// minutos `transcurrido` en la fase en curso, se condiciona a esas; si no,
/// la fase va desbordada y la cifra es una cota inferior. Sin parecidas,
/// `None`.
pub fn restante(parecidas: &[&Muestra], fase: Fase, transcurrido: f64) -> Option<Restante> {
    if parecidas.is_empty() {
        return None;
    }
    let (siguientes, actual): (&[&str], Option<&str>) = match fase {
        Fase::Diseno => (&FASES, None),
        Fase::Implementacion => (&FASES, Some("implementacion")),
        Fase::Rondas => (&FASES[1..], Some("rondas")),
    };
    let real = |m: &Muestra, f: &str| m.real.fase(f).unwrap_or_default();
    let vivas: Vec<&Muestra> = parecidas
        .iter()
        .copied()
        .filter(|m| actual.is_none_or(|a| real(m, a) > transcurrido))
        .collect();
    let condicionada = vivas.len() >= N_MIN && actual.is_some();
    let base: Vec<&Muestra> = if vivas.len() >= N_MIN {
        vivas.clone()
    } else {
        parecidas.to_vec()
    };
    let xs: Vec<f64> = base
        .iter()
        .map(|m| {
            let mut r: f64 = siguientes.iter().map(|f| real(m, f)).sum();
            if let Some(a) = actual {
                r -= transcurrido.min(real(m, a));
            }
            r.max(0.0)
        })
        .collect();
    Some(Restante {
        mediana: mediana(&xs)?,
        p80: cuantil(&xs, P80)?,
        n: base.len(),
        condicionada,
        desbordada: actual.is_some() && vivas.len() < N_MIN,
    })
}

/// Error de una fase (o del total): estimado (la mediana guardada) frente a
/// real.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ErrorFase {
    /// `implementacion`, `rondas`, `cierre` o `total`.
    pub fase: &'static str,
    /// Mediana estimada.
    pub estimado: f64,
    /// Real.
    pub real: f64,
    /// |estimado - real|, en minutos.
    pub absoluto: f64,
    /// |estimado - real| / real (real de al menos 1 min, como la PoC).
    pub relativo: f64,
    /// estimado / real (real de al menos 1 min).
    pub razon: f64,
}

/// El error de `e` frente a `r` por fase y en total, de las que `e` tiene
/// cifra (D4a de TASK-083: funcion pura; la escritura al cerrar es de F8-7).
pub fn error(e: &Estimacion, r: &Real) -> Vec<ErrorFase> {
    let mut v = Vec::new();
    let mut uno = |fase: &'static str, est: Option<Rango>, real: f64| {
        if let Some(est) = est {
            let base = real.max(1.0);
            let absoluto = (est.mediana - real).abs();
            v.push(ErrorFase {
                fase,
                estimado: est.mediana,
                real,
                absoluto,
                relativo: absoluto / base,
                razon: est.mediana / base,
            });
        }
    };
    if let Some(p) = &e.por_fase {
        for f in FASES {
            uno(f, p.fase(f), r.fase(f).unwrap_or_default());
        }
    }
    uno("total", e.total, r.total());
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn muestra(
        id: &str,
        carril: Option<Carril>,
        compl: Option<&str>,
        crit: usize,
        imp: f64,
    ) -> Muestra {
        Muestra {
            id: id.to_owned(),
            miembros: vec![id.to_owned()],
            carril,
            complejidad: compl.map(str::to_owned),
            criterios: crit,
            real: Real {
                implementacion: imp,
                rondas: 10.0,
                cierre: 5.0,
            },
            start: 0,
            finish: 0,
        }
    }

    fn obj(carril: Option<Carril>) -> Objetivo {
        Objetivo {
            id: "TASK-100".to_owned(),
            carril,
            complejidad: Some("media".to_owned()),
            criterios: 2,
        }
    }

    const F: Option<Carril> = Some(Carril::Funcionalidad);

    #[test]
    fn mediana_y_p80_por_rango_mas_cercano() {
        assert_eq!(mediana(&[3.0, 1.0, 2.0]), Some(2.0));
        assert_eq!(mediana(&[4.0, 1.0, 2.0, 3.0]), Some(2.5));
        assert_eq!(mediana(&[]), None);
        let seis = [10.0, 20.0, 30.0, 40.0, 50.0, 60.0];
        assert_eq!(cuantil(&seis, P80), Some(50.0));
        assert_eq!(cuantil(&seis[..5], P80), Some(40.0));
        assert_eq!(cuantil(&[7.0], P80), Some(7.0));
        assert_eq!(cuantil(&[], P80), None);
    }

    /// C2 y C3: la cascada baja de nivel con menos de 5 y, sin ninguno,
    /// `sin-datos` sin cifras.
    #[test]
    fn cascada_y_sin_datos() {
        let mut ms: Vec<Muestra> = (0..4)
            .map(|i| {
                muestra(
                    &format!("TASK-{i:03}"),
                    F,
                    Some("media"),
                    2,
                    10.0 * f64::from(i + 1),
                )
            })
            .collect();
        let e = estimar(&obj(F), &ms, "t");
        assert_eq!((e.nivel.as_str(), e.n, e.total), (SIN_DATOS, 4, None));
        assert!(e.por_fase.is_none() && e.sin_datos());
        ms.push(muestra("TASK-010", F, Some("alta"), 2, 50.0));
        let e = estimar(&obj(F), &ms, "t");
        assert_eq!((e.nivel.as_str(), e.n), ("carril", 5));
        assert_eq!(
            e.por_fase.and_then(|p| p.implementacion),
            Some(Rango {
                mediana: 30.0,
                p80: 40.0
            })
        );
        ms.push(muestra("TASK-011", F, Some("media"), 9, 60.0));
        let e = estimar(&obj(F), &ms, "t");
        assert_eq!((e.nivel.as_str(), e.n), ("carril+complejidad", 5));
        ms.push(muestra("TASK-012", F, Some("media"), 1, 60.0));
        let e = estimar(&obj(F), &ms, "t");
        assert_eq!((e.nivel.as_str(), e.n), ("carril+complejidad+tamano", 5));
        // Sin carril declarado (D5a), solo `repo`.
        let e = estimar(&obj(None), &ms, "t");
        assert_eq!((e.nivel.as_str(), e.n), ("repo", 7));
        // Las muestras sin carril no cuentan en los niveles de carril.
        let sin: Vec<Muestra> = (0..6)
            .map(|i| muestra(&format!("TASK-{i:03}"), None, Some("media"), 2, 5.0))
            .collect();
        assert_eq!(estimar(&obj(None), &sin, "t").nivel, "repo");
    }

    /// C7: la tarea estimada nunca es muestra de si misma, ni sola ni dentro
    /// de un grupo de troceo.
    #[test]
    fn c7_nunca_es_muestra_de_si_misma() {
        let mut ms: Vec<Muestra> = (0..4)
            .map(|i| muestra(&format!("TASK-{i:03}"), F, Some("media"), 2, 10.0))
            .collect();
        ms.push(muestra("TASK-100", F, Some("media"), 2, 99.0));
        let e = estimar(&obj(F), &ms, "t");
        assert_eq!((e.nivel.as_str(), e.n), (SIN_DATOS, 4));
        let mut grupo = muestra("TASK-050+TASK-100", F, Some("media"), 2, 99.0);
        grupo.miembros = vec!["TASK-050".into(), "TASK-100".into()];
        ms.pop();
        ms.push(grupo);
        assert_eq!(estimar(&obj(F), &ms, "t").n, 4);
    }

    /// Punto 5 de F1-17: condicionada a las que pasaron de lo transcurrido;
    /// con menos de 5, desbordada y cota inferior.
    #[test]
    fn restante_condicionado_y_desbordado() {
        let ms: Vec<Muestra> = [10.0, 20.0, 30.0, 40.0, 50.0, 60.0]
            .iter()
            .enumerate()
            .map(|(i, x)| muestra(&format!("TASK-{i:03}"), F, None, 2, *x))
            .collect();
        let p: Vec<&Muestra> = ms.iter().collect();
        // Diseno: todo lo que falta (imp + 15).
        let r = restante(&p, Fase::Diseno, 0.0);
        assert_eq!(
            r.map(|r| (r.mediana, r.p80, r.condicionada, r.desbordada)),
            Some((50.0, 65.0, false, false))
        );
        // 5 minutos en implementacion: pasan las 6, condicionada.
        let r = restante(&p, Fase::Implementacion, 5.0);
        assert_eq!(
            r.map(|r| (r.n, r.condicionada, r.desbordada)),
            Some((6, true, false))
        );
        // 45 minutos: solo 2 por encima, desbordada; la cota sale de todas.
        let r = restante(&p, Fase::Implementacion, 45.0);
        assert_eq!(r.map(|r| (r.n, r.desbordada)), Some((6, true)));
        // En rondas, lo que falta son rondas y cierre.
        let r = restante(&p, Fase::Rondas, 0.0);
        assert_eq!(r.map(|r| r.mediana), Some(15.0));
        assert_eq!(restante(&[], Fase::Rondas, 0.0), None);
    }

    /// C6: [30, 52] guardado frente a 75 de real: 45 min, 0,60 y 0,40.
    #[test]
    fn c6_error_absoluto_relativo_y_razon() {
        let e = Estimacion {
            calculada: "t".into(),
            nivel: "carril".into(),
            n: 5,
            por_fase: Some(PorFase {
                implementacion: Some(Rango {
                    mediana: 30.0,
                    p80: 52.0,
                }),
                rondas: None,
                cierre: None,
            }),
            total: Some(Rango {
                mediana: 69.0,
                p80: 90.0,
            }),
            agente: None,
        };
        let r = Real {
            implementacion: 75.0,
            rondas: 30.0,
            cierre: 10.0,
        };
        let v = error(&e, &r);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].fase, "implementacion");
        assert_eq!(v[0].absoluto, 45.0);
        assert!((v[0].relativo - 0.6).abs() < 1e-12);
        assert!((v[0].razon - 0.4).abs() < 1e-12);
        assert_eq!(
            (v[1].fase, v[1].absoluto, v[1].real),
            ("total", 46.0, 115.0)
        );
        // Real de menos de 1 min cuenta 1.
        let r0 = Real {
            implementacion: 0.0,
            rondas: 0.0,
            cierre: 0.0,
        };
        assert_eq!(error(&e, &r0)[0].relativo, 30.0);
    }
}
