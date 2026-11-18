//! La forma del campo `estimacion` de `tarea.md` (ADR F1-17, punto 7), su
//! lectura, la linea para la persona y las claves de la linea de contrato.
//!
//! ```yaml
//! estimacion:
//!   calculada: "2026-11-15T12:00:00Z"
//!   nivel: "carril+complejidad+tamano"
//!   n: 17
//!   por_fase: {implementacion: [21, 52], rondas: [16, 34], cierre: [6, 12]}
//!   total: [50, 96]
//!   agente: null
//! ```
//!
//! Los pares son [mediana, p80] en minutos enteros (redondeo al mas
//! cercano). Con `nivel: "sin-datos"`, `por_fase: null` y `total: null`.

use serde::Deserialize;

use super::estimar::{Estimacion, N_MIN, PorFase, Rango};
use super::real::FASES;
use crate::contrato::Valor;
use crate::tarea::yaml;

/// Minutos enteros de una cifra (redondeo al mas cercano).
pub fn minutos(x: f64) -> u64 {
    let r = x.round();
    if r.is_finite() && r > 0.0 {
        r as u64
    } else {
        0
    }
}

/// `[mediana, p80]` en minutos.
fn par(r: Rango) -> String {
    format!("[{}, {}]", minutos(r.mediana), minutos(r.p80))
}

impl Estimacion {
    /// La entrada `estimacion:` del frontmatter, con su salto final: lo que
    /// guarda [`crate::tarea::Tarea::poner_estimacion`].
    pub fn yaml(&self) -> String {
        let por_fase = match &self.por_fase {
            None => "null".to_owned(),
            Some(p) => {
                let v: Vec<String> = FASES
                    .iter()
                    .filter_map(|f| p.fase(f).map(|r| format!("{f}: {}", par(r))))
                    .collect();
                format!("{{{}}}", v.join(", "))
            }
        };
        format!(
            "estimacion:\n  calculada: {}\n  nivel: {}\n  n: {}\n  por_fase: {por_fase}\n  total: {}\n  agente: {}\n",
            yaml::cadena(&self.calculada),
            yaml::cadena(&self.nivel),
            self.n,
            self.total.map_or_else(|| "null".to_owned(), par),
            yaml::numero(self.agente),
        )
    }

    /// Lee la entrada cruda `estimacion: ...` (la de
    /// [`crate::tarea::Tarea::estimacion`]). Las claves que faltan valen
    /// `null` (o `0`/vacio); una clave desconocida o un tipo que no es el
    /// suyo es error.
    pub fn leer(crudo: &str) -> Result<Self, String> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Fases {
            implementacion: Option<(f64, f64)>,
            rondas: Option<(f64, f64)>,
            cierre: Option<(f64, f64)>,
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Crudo {
            calculada: Option<String>,
            nivel: Option<String>,
            n: Option<u64>,
            por_fase: Option<Fases>,
            total: Option<(f64, f64)>,
            agente: Option<u64>,
        }
        #[derive(Deserialize)]
        struct Entrada {
            estimacion: Option<Crudo>,
        }
        let r = |p: Option<(f64, f64)>| p.map(|(mediana, p80)| Rango { mediana, p80 });
        let e: Entrada = yaml::decodificar(crudo).map_err(|e| e.to_string())?;
        let c = e.estimacion.ok_or_else(|| "estimacion: null".to_owned())?;
        Ok(Self {
            calculada: c.calculada.unwrap_or_default(),
            nivel: c.nivel.unwrap_or_default(),
            n: usize::try_from(c.n.unwrap_or_default()).unwrap_or(usize::MAX),
            por_fase: c.por_fase.map(|f| PorFase {
                implementacion: r(f.implementacion),
                rondas: r(f.rondas),
                cierre: r(f.cierre),
            }),
            total: r(c.total),
            agente: c.agente,
        })
    }

    /// La linea para la persona: `estimacion: ≈M min (p80 P), n=K,
    /// nivel=X` o, sin datos, `estimacion: sin tareas parecidas (n=k<5)`.
    /// Es un rango de tareas parecidas, no una estimacion calibrada (D8a).
    pub fn linea(&self) -> String {
        match self.total {
            Some(t) if !self.sin_datos() => format!(
                "estimacion: ≈{} min (p80 {}), n={}, nivel={}",
                minutos(t.mediana),
                minutos(t.p80),
                self.n,
                self.nivel
            ),
            _ => format!("estimacion: sin tareas parecidas (n={}<{N_MIN})", self.n),
        }
    }

    /// Las claves de la linea de contrato: `estimacion=<mediana>|sin-datos`,
    /// `estimacion-p80=<p80>` (con datos), `estimacion-n=<n>` y
    /// `estimacion-nivel=<nivel>`.
    pub fn claves(&self) -> Vec<(String, Valor)> {
        let t = |s: String| Valor::Texto(s);
        let mut v = Vec::new();
        match self.total {
            Some(r) if !self.sin_datos() => {
                v.push(("estimacion".to_owned(), t(minutos(r.mediana).to_string())));
                v.push(("estimacion-p80".to_owned(), t(minutos(r.p80).to_string())));
            }
            _ => v.push((
                "estimacion".to_owned(),
                t(super::estimar::SIN_DATOS.to_owned()),
            )),
        }
        v.push(("estimacion-n".to_owned(), t(self.n.to_string())));
        v.push(("estimacion-nivel".to_owned(), t(self.nivel.clone())));
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn con_datos() -> Estimacion {
        Estimacion {
            calculada: "2026-11-15T12:00:00Z".into(),
            nivel: "carril+complejidad+tamano".into(),
            n: 17,
            por_fase: Some(PorFase {
                implementacion: Some(Rango {
                    mediana: 21.4,
                    p80: 52.0,
                }),
                rondas: Some(Rango {
                    mediana: 15.5,
                    p80: 34.0,
                }),
                cierre: Some(Rango {
                    mediana: 6.0,
                    p80: 12.0,
                }),
            }),
            total: Some(Rango {
                mediana: 49.5,
                p80: 96.0,
            }),
            agente: None,
        }
    }

    /// La forma del punto 7 de F1-17, byte a byte, y su lectura (enteros).
    #[test]
    fn forma_de_f1_17_ida_y_vuelta() {
        let e = con_datos();
        let y = e.yaml();
        assert_eq!(
            y,
            "estimacion:\n  calculada: \"2026-11-15T12:00:00Z\"\n  nivel: \"carril+complejidad+tamano\"\n  n: 17\n  por_fase: {implementacion: [21, 52], rondas: [16, 34], cierre: [6, 12]}\n  total: [50, 96]\n  agente: null\n"
        );
        let l = Estimacion::leer(&y);
        let redondeada = |x: f64| x.round();
        assert_eq!(
            l.as_ref().map(|l| l.total.map(|t| t.mediana)),
            Ok(Some(redondeada(49.5)))
        );
        assert_eq!(l.as_ref().map(|l| l.n), Ok(17));
        assert_eq!(l.map(|l| l.yaml()), Ok(y));
    }

    /// C3: sin datos, `por_fase: null`, `total: null` y la linea lo dice
    /// sin cifras.
    #[test]
    fn c3_sin_datos_sin_cifras() {
        let e = Estimacion {
            calculada: "2026-11-15T12:00:00Z".into(),
            nivel: "sin-datos".into(),
            n: 3,
            por_fase: None,
            total: None,
            agente: None,
        };
        assert_eq!(
            e.yaml(),
            "estimacion:\n  calculada: \"2026-11-15T12:00:00Z\"\n  nivel: \"sin-datos\"\n  n: 3\n  por_fase: null\n  total: null\n  agente: null\n"
        );
        assert_eq!(e.linea(), "estimacion: sin tareas parecidas (n=3<5)");
        let claves: Vec<String> = e
            .claves()
            .iter()
            .map(|(k, v)| match v {
                Valor::Texto(t) => format!("{k}={t}"),
                Valor::Lista(_) => String::new(),
            })
            .collect();
        assert_eq!(
            claves,
            [
                "estimacion=sin-datos",
                "estimacion-n=3",
                "estimacion-nivel=sin-datos"
            ]
        );
        assert_eq!(Estimacion::leer(&e.yaml()), Ok(e));
        assert_eq!(
            con_datos().linea(),
            "estimacion: ≈50 min (p80 96), n=17, nivel=carril+complejidad+tamano"
        );
    }

    #[test]
    fn lectura_tolerante_pero_estricta_en_tipos() {
        let parcial = "estimacion:\n  calculada: \"x\"\n  por_fase: {implementacion: [21, 52]}\n  total: [50, 96]\n";
        let e = Estimacion::leer(parcial);
        assert_eq!(
            e.as_ref()
                .map(|e| (e.nivel.as_str(), e.por_fase.and_then(|p| p.rondas))),
            Ok(("", None))
        );
        for malo in [
            "estimacion: null\n",
            "estimacion:\n  otra: 1\n",
            "estimacion:\n  total: [1]\n",
            "estimacion:\n  n: \"x\"\n",
            "estimacion: [\n",
        ] {
            assert!(Estimacion::leer(malo).is_err(), "{malo}");
        }
        assert_eq!(minutos(-3.0), 0);
        assert_eq!(minutos(f64::NAN), 0);
        assert_eq!(minutos(29.5), 30);
    }
}
