//! Rondas incrementales del revisor IA (F6-6; RF-33).
//!
//! Funciones puras: la ronda siguiente se decide con el check `revision-ia`
//! anterior, el que publico la App del revisor (no la rama de estado, que
//! escribe el agente: F1-8). Cada check lleva su ronda en el titulo y todos
//! los hallazgos que siguen abiertos en sus anotaciones.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{AceptadoDe, Dictamen, VERSION, dictamen_publicado, plano, ronda_de_titulo};

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

/// Un check-run tal como lo da la API (`GET .../commits/<sha>/check-runs`):
/// solo lo que se mira; lo demas se ignora.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct CheckRun {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub conclusion: Option<String>,
    #[serde(default)]
    pub head_sha: String,
    #[serde(default)]
    pub app: Option<AppCheck>,
    #[serde(default)]
    pub output: Option<SalidaCheck>,
}

/// La App que publico el check.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct AppCheck {
    #[serde(default)]
    pub id: u64,
}

/// `output` de un check-run.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct SalidaCheck {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
}

impl CheckRun {
    /// `output.title`, o vacio.
    pub fn titulo(&self) -> &str {
        self.output
            .as_ref()
            .and_then(|o| o.title.as_deref())
            .unwrap_or_default()
    }
}

/// El check `revision-ia` de un commit que cuenta: el mas nuevo de ese
/// nombre que publico la App `origen` (la API los da del mas nuevo al mas
/// viejo), y solo si ya termino. Lo comparten `leer-revision` (F6-5) y la
/// ronda siguiente (F6-6).
pub fn revision_de_origen(runs: Vec<CheckRun>, origen: u64) -> Option<CheckRun> {
    runs.into_iter()
        .find(|r| {
            r.name == crate::revision::REVISION_IA && r.app.as_ref().is_some_and(|a| a.id == origen)
        })
        .filter(|r| r.status == "completed" && r.id != 0)
}

/// El check que sirve de ronda anterior: recorriendo los commits del PR del
/// mas nuevo al mas viejo (sin el head), el primero cuyo `revision-ia` de la
/// App `origen` dice `rojo` o `verde` en su titulo. Una ronda `incompleta`
/// no cuenta ni sirve de base, y una `tope` tampoco (no reviso nada; lleva
/// los mismos abiertos que la ronda que la origino).
pub fn elegir(commits: Vec<Vec<CheckRun>>, origen: u64) -> Option<CheckRun> {
    commits.into_iter().find_map(|runs| {
        revision_de_origen(runs, origen).filter(|r| {
            matches!(
                Dictamen::de_titulo(r.titulo()),
                Some(Dictamen::Rojo | Dictamen::Verde)
            )
        })
    })
}

/// La ronda anterior, leida de su check y de sus anotaciones.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Anterior {
    pub ronda: u32,
    pub sha: String,
    pub dictamen: Dictamen,
    /// Solo los que siguen abiertos.
    pub abiertos: Vec<AceptadoDe>,
}

/// La ronda anterior de `check` con los aceptados leidos de sus anotaciones,
/// solo si el check casa ([`dictamen_publicado`], la regla de
/// `leer-revision`), dice `rojo` o `verde` y su commit es un SHA. `None` si
/// no: el workflow lo trata como un check alterado (falla en cerrado).
pub fn anterior(check: &CheckRun, aceptados: Vec<AceptadoDe>) -> Option<Anterior> {
    let titulo = check.titulo();
    let conclusion = check.conclusion.as_deref().unwrap_or_default();
    let dictamen = dictamen_publicado(titulo, conclusion, &aceptados)
        .filter(|d| matches!(d, Dictamen::Rojo | Dictamen::Verde))?;
    let sha_valido =
        check.head_sha.len() == 40 && check.head_sha.chars().all(|c| c.is_ascii_hexdigit());
    if !sha_valido {
        return None;
    }
    Some(Anterior {
        ronda: ronda_de_titulo(titulo)?,
        sha: check.head_sha.to_ascii_lowercase(),
        dictamen,
        abiertos: aceptados.into_iter().filter(AceptadoDe::abierto).collect(),
    })
}

/// La ronda siguiente a `anterior` (F6-6, decisiones 4 a 6 del plan de
/// TASK-052), con `cambiados` los ficheros del diff desde el commit
/// anterior y `tope` el tope efectivo de rondas:
///
/// - sin anterior: ronda 1, `completa`;
/// - anterior `verde` y los cambios solo tocan ficheros con MENOR abiertos:
///   `ligera` (sin modelo; la ronda no sube; los MENOR de ficheros
///   cambiados se cierran);
/// - anterior `rojo` y la ronda siguiente pasa de `tope`: `tope` (sin
///   modelo; los abiertos se arrastran todos);
/// - si no, `incremental`: la ronda sube, los MENOR de ficheros cambiados
///   se cierran (no pasan por el cierre del modelo) y los CRITICO e
///   IMPORTANTE siguen abiertos hasta un cierre valido.
pub fn decidir(anterior: Option<&Anterior>, cambiados: &[String], tope: u32) -> Decision {
    let tope = tope.max(1);
    let Some(a) = anterior else {
        return Decision {
            version: VERSION,
            ronda: 1,
            modo: Modo::Completa,
            sha_anterior: None,
            tope,
            abiertos: Vec::new(),
            cerrados: Vec::new(),
        };
    };
    let cambiado = |f: &str| cambiados.iter().any(|c| c == f);
    let menores: BTreeSet<&str> = a
        .abiertos
        .iter()
        .filter(|x| !x.grave())
        .map(|x| x.hallazgo.fichero.as_str())
        .collect();
    let (modo, ronda) = if a.dictamen == Dictamen::Verde
        && cambiados.iter().all(|c| menores.contains(c.as_str()))
    {
        (Modo::Ligera, a.ronda)
    } else if a.dictamen == Dictamen::Rojo && a.ronda.saturating_add(1) > tope {
        (Modo::Tope, a.ronda.saturating_add(1))
    } else {
        (Modo::Incremental, a.ronda.saturating_add(1))
    };
    let (cerrados, abiertos): (Vec<AceptadoDe>, Vec<AceptadoDe>) = a
        .abiertos
        .iter()
        .cloned()
        .partition(|x| modo != Modo::Tope && !x.grave() && cambiado(&x.hallazgo.fichero));
    Decision {
        version: VERSION,
        ronda,
        modo,
        sha_anterior: Some(a.sha.clone()),
        tope,
        abiertos,
        cerrados,
    }
}

/// Una revisora del detalle de `tc2 revisoras` (`--detalle`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct RevisoraDetalle {
    pub id: String,
    #[serde(default)]
    pub rutas: Vec<String>,
}

/// Reparte los abiertos CRITICO e IMPORTANTE entre las revisoras elegidas:
/// cada uno a su `revisora` si esta elegida; si no, a la primera que tiene
/// su fichero entre sus rutas. Uno que no casa con ninguna no se da a nadie
/// (nadie puede cerrarlo: sigue abierto). Los MENOR no pasan por el cierre
/// del modelo.
pub fn repartir(abiertos: &[AceptadoDe], detalle: &[RevisoraDetalle]) -> Vec<Vec<AceptadoDe>> {
    let mut r: Vec<Vec<AceptadoDe>> = vec![Vec::new(); detalle.len()];
    for a in abiertos.iter().filter(|a| a.abierto() && a.grave()) {
        let k = detalle.iter().position(|d| d.id == a.revisora).or_else(|| {
            detalle
                .iter()
                .position(|d| d.rutas.contains(&a.hallazgo.fichero))
        });
        if let Some(lista) = k.and_then(|k| r.get_mut(k)) {
            lista.push(a.clone());
        }
    }
    r
}

/// Los abiertos de una revisora para su prompt, en Markdown, con todo el
/// texto del modelo por [`plano`] (dato, no instrucciones).
pub fn abiertos_markdown(abiertos: &[AceptadoDe]) -> String {
    if abiertos.is_empty() {
        return "No tienes hallazgos abiertos que cerrar.\n".to_owned();
    }
    let mut s = String::from(
        "| id | ronda | severidad | clase | fichero | linea | descripcion |\n|---|---|---|---|---|---|---|\n",
    );
    for a in abiertos {
        let h = &a.hallazgo;
        s.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            plano(&h.id, 64),
            a.ronda,
            h.severidad.texto(),
            h.clase.texto(),
            plano(&h.fichero, 200),
            h.linea,
            plano(&h.descripcion, 400)
        ));
    }
    s
}

/// Cuantas rondas extra pide la tarea: los elementos de lista de su seccion
/// `## Rondas extra` (F1-16; uno por ronda).
pub fn rondas_extra(cuerpo: &crate::tarea::Cuerpo) -> u32 {
    cuerpo.seccion("Rondas extra").map_or(0, |s| {
        let n = s
            .texto
            .lines()
            .filter(|l| l.starts_with("- ") || l.starts_with("* "))
            .count();
        u32::try_from(n).unwrap_or(u32::MAX)
    })
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use crate::hallazgos::Estado;

    fn a(id: &str, severidad: &str, fichero: &str) -> AceptadoDe {
        serde_json::from_str(&format!(
            r#"{{"revisora":"rev:a","hallazgo":{{"id":"{id}","severidad":"{severidad}","clase":"correccion","fichero":"{fichero}","linea":1,"descripcion":"d","evidencia":{{"tipo":"borde","entrada":"x","esperado":"1","obtenido":"2"}}}}}}"#
        ))
        .unwrap_or_else(|e| panic!("{e}"))
    }

    fn previa(dictamen: Dictamen, ronda: u32, abiertos: Vec<AceptadoDe>) -> Anterior {
        Anterior {
            ronda,
            sha: "a".repeat(40),
            dictamen,
            abiertos,
        }
    }

    fn rutas(r: &[&str]) -> Vec<String> {
        r.iter().map(|x| (*x).to_owned()).collect()
    }

    fn ids(v: &[AceptadoDe]) -> Vec<&str> {
        v.iter().map(|x| x.hallazgo.id.as_str()).collect()
    }

    /// F6-6, criterio 2 (unitario): las reglas de la decision de la ronda
    /// (RF-33; decisiones 4 a 6 del plan).
    #[test]
    fn decidir_aplica_tope_ligera_e_incremental() {
        // Sin anterior (o con una incompleta, que `elegir` salta): ronda 1.
        let d = decidir(None, &rutas(&["x"]), 2);
        assert_eq!((d.ronda, d.modo, d.sha_anterior), (1, Modo::Completa, None));
        // Rojo en la ronda 2 con tope 2: tope, sin modelo, con todo arrastrado
        // (tambien el MENOR de un fichero cambiado).
        let rojo = previa(
            Dictamen::Rojo,
            2,
            vec![a("I1", "IMPORTANTE", "a.rs"), a("M1", "MENOR", "b.rs")],
        );
        let d = decidir(Some(&rojo), &rutas(&["a.rs", "b.rs"]), 2);
        assert_eq!((d.ronda, d.modo, d.tope), (3, Modo::Tope, 2));
        assert_eq!(ids(&d.abiertos), ["I1", "M1"]);
        assert!(d.cerrados.is_empty());
        // Una ronda extra en la tarea sube el tope: incremental.
        let d = decidir(Some(&rojo), &rutas(&["a.rs", "b.rs"]), 3);
        assert_eq!((d.ronda, d.modo), (3, Modo::Incremental));
        assert_eq!(d.sha_anterior.as_deref(), Some("a".repeat(40).as_str()));
        // El MENOR de un fichero cambiado se cierra sin modelo; el grave sigue.
        assert_eq!(ids(&d.abiertos), ["I1"]);
        assert_eq!(ids(&d.cerrados), ["M1"]);
        // Verde con cambios solo en ficheros con MENOR abiertos: ligera, la
        // ronda no sube.
        let verde = previa(
            Dictamen::Verde,
            2,
            vec![a("M1", "MENOR", "b.rs"), a("M2", "MENOR", "c.rs")],
        );
        let d = decidir(Some(&verde), &rutas(&["b.rs"]), 2);
        assert_eq!((d.ronda, d.modo), (2, Modo::Ligera));
        assert_eq!(ids(&d.cerrados), ["M1"]);
        assert_eq!(ids(&d.abiertos), ["M2"]);
        // Un cambio fuera de ellos: incremental (y en verde no hay tope).
        let d = decidir(Some(&verde), &rutas(&["b.rs", "otro.rs"]), 2);
        assert_eq!((d.ronda, d.modo), (3, Modo::Incremental));
    }

    fn check(id: u64, app: u64, titulo: &str, estado: &str) -> CheckRun {
        CheckRun {
            id,
            name: "revision-ia".to_owned(),
            status: estado.to_owned(),
            conclusion: Some("failure".to_owned()),
            head_sha: "b".repeat(40),
            app: Some(AppCheck { id: app }),
            output: Some(SalidaCheck {
                title: Some(titulo.to_owned()),
                summary: None,
            }),
        }
    }

    /// Solo cuenta el check de la App del revisor, terminado; una ronda
    /// `incompleta` o `tope` no sirve de base y se sigue al commit anterior.
    #[test]
    fn elegir_salta_incompletas_y_otras_apps() {
        let rojo = "revision-ia: rojo (ronda=1 criticos=0 importantes=1 menores=0)";
        let commits = vec![
            vec![check(5, 7, "revision-ia: incompleta (x)", "completed")],
            vec![check(4, 99, rojo, "completed")],
            vec![check(3, 7, rojo, "in_progress")],
            vec![check(
                2,
                7,
                "revision-ia: tope (ronda=3 criticos=0 importantes=1 menores=0)",
                "completed",
            )],
            vec![check(1, 7, rojo, "completed")],
        ];
        assert_eq!(elegir(commits.clone(), 7).map(|c| c.id), Some(1));
        assert_eq!(elegir(commits, 8), None);
        // `anterior` exige que el check case con sus anotaciones.
        let c = check(1, 7, rojo, "completed");
        let i1 = a("I1", "IMPORTANTE", "a.rs");
        let p = anterior(&c, vec![i1.clone()]).unwrap_or_else(|| panic!("no casa"));
        assert_eq!((p.ronda, p.dictamen), (1, Dictamen::Rojo));
        assert_eq!(anterior(&c, Vec::new()), None);
        let mut cerrado = i1;
        cerrado.estado = Estado::Cerrado;
        assert_eq!(anterior(&c, vec![cerrado]), None);
    }

    /// Cada abierto grave va a su revisora si esta elegida, o a la que tiene
    /// su fichero; los MENOR no se reparten. `## Rondas extra` cuenta los
    /// elementos de primer nivel.
    #[test]
    fn repartir_y_rondas_extra() {
        let detalle = vec![
            RevisoraDetalle {
                id: "rev:b".to_owned(),
                rutas: rutas(&["a.rs"]),
            },
            RevisoraDetalle {
                id: "rev:a".to_owned(),
                rutas: rutas(&["z.rs"]),
            },
        ];
        let mut otra = a("I2", "CRITICO", "a.rs");
        otra.revisora = "rev:c".to_owned();
        let mut suelta = a("I3", "IMPORTANTE", "nada.rs");
        suelta.revisora = "rev:d".to_owned();
        let r = repartir(
            &[
                a("I1", "IMPORTANTE", "a.rs"),
                otra,
                a("M1", "MENOR", "a.rs"),
                suelta,
            ],
            &detalle,
        );
        assert_eq!(ids(&r[0]), ["I2"]);
        assert_eq!(ids(&r[1]), ["I1"]);
        let cuerpo = crate::tarea::Cuerpo::leer(
            "## Rondas extra\n\n- Ronda 3: la decide Carlos.\n  - detalle\n- Ronda 4\n\n## Otra\n\n- no cuenta\n",
        );
        assert_eq!(rondas_extra(&cuerpo), 2);
        assert_eq!(rondas_extra(&crate::tarea::Cuerpo::leer("## Otra\n")), 0);
    }
}
