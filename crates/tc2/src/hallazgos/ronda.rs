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
/// viejo), y solo si ya termino. Es la regla de `leer-revision` (F6-5); la
/// ronda siguiente (F6-6) mira todos los del commit ([`elegir`]).
pub fn revision_de_origen(runs: Vec<CheckRun>, origen: u64) -> Option<CheckRun> {
    runs.into_iter()
        .find(|r| {
            r.name == crate::revision::REVISION_IA && r.app.as_ref().is_some_and(|a| a.id == origen)
        })
        .filter(|r| r.status == "completed" && r.id != 0)
}

/// El modo de una ronda leido del resumen de su check: la linea `Ronda N
/// (<modo>): ...` que escribe [`super::resumen_veredicto`]. El texto del
/// modelo no puede fingirla: va por [`super::plano`], sin saltos de linea, y
/// siempre dentro de una fila de tabla o de un elemento de lista.
pub fn modo_de_resumen(resumen: &str) -> Option<Modo> {
    resumen.lines().find_map(|l| {
        let resto = l.strip_prefix("Ronda ")?;
        let (numero, resto) = resto.split_once(" (")?;
        if numero.is_empty() || !numero.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let (modo, _) = resto.split_once("): ")?;
        [Modo::Completa, Modo::Incremental, Modo::Ligera, Modo::Tope]
            .into_iter()
            .find(|m| m.texto() == modo)
    })
}

/// Lo que da [`elegir`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Eleccion {
    /// El check que sirve de ronda anterior.
    pub check: Option<CheckRun>,
    /// Si entre el head y ese check hubo una ronda `ligera`: su codigo no lo
    /// vio ningun modelo, asi que la ronda siguiente no puede ser otra
    /// `ligera`.
    pub ligera_pendiente: bool,
}

/// El check que sirve de ronda anterior: recorriendo los commits del PR del
/// mas nuevo al mas viejo (sin el head) y, en cada commit, sus check-runs
/// `revision-ia` terminados de la App `origen` del mas nuevo al mas viejo
/// (por `id`), el primero cuyo titulo dice `rojo` o `verde` y que no fue una
/// ronda `ligera`. Una ronda `incompleta` no cuenta ni sirve de base (aunque
/// sea la ultima de su commit: se mira la anterior del mismo commit), una
/// `tope` tampoco (no reviso nada; lleva los mismos abiertos que la ronda que
/// la origino) y una `ligera` tampoco: no la vio ningun modelo, y la base
/// del diff incremental sigue siendo el ultimo commit revisado por modelo.
pub fn elegir(commits: Vec<Vec<CheckRun>>, origen: u64) -> Eleccion {
    let mut ligera_pendiente = false;
    for mut runs in commits {
        runs.retain(|r| {
            r.name == crate::revision::REVISION_IA
                && r.app.as_ref().is_some_and(|a| a.id == origen)
                && r.status == "completed"
                && r.id != 0
        });
        runs.sort_by_key(|r| std::cmp::Reverse(r.id));
        for r in runs {
            if !matches!(
                Dictamen::de_titulo(r.titulo()),
                Some(Dictamen::Rojo | Dictamen::Verde)
            ) {
                continue;
            }
            let resumen = r.output.as_ref().and_then(|o| o.summary.as_deref());
            if resumen.and_then(modo_de_resumen) == Some(Modo::Ligera) {
                ligera_pendiente = true;
                continue;
            }
            return Eleccion {
                check: Some(r),
                ligera_pendiente,
            };
        }
    }
    Eleccion {
        check: None,
        ligera_pendiente,
    }
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
/// anterior, `tope` el tope efectivo de rondas y `ligera_pendiente` si desde
/// ese commit ya hubo una ronda `ligera` ([`Eleccion`]):
///
/// - sin anterior: ronda 1, `completa`;
/// - anterior `verde`, sin otra `ligera` desde el y los cambios solo tocan
///   ficheros con MENOR abiertos: `ligera` (sin modelo; la ronda no sube;
///   los MENOR de ficheros cambiados se cierran). No sirve de base: lo que
///   entro en ella lo ve la ronda siguiente, que ya no puede ser `ligera`;
/// - anterior `rojo` y la ronda siguiente pasa de `tope`: `tope` (sin
///   modelo; los abiertos se arrastran todos);
/// - si no, `incremental`: la ronda sube, los MENOR de ficheros cambiados
///   se cierran (no pasan por el cierre del modelo) y los CRITICO e
///   IMPORTANTE siguen abiertos hasta un cierre valido.
pub fn decidir(
    anterior: Option<&Anterior>,
    cambiados: &[String],
    tope: u32,
    ligera_pendiente: bool,
) -> Decision {
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
        && !ligera_pendiente
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

/// Cuantas rondas extra pide la tarea: los elementos de lista de primer
/// nivel (`- ...` o `* ...`, con texto) de sus secciones `## Rondas extra` y
/// `## Ronda extra` (F1-16; uno por ronda). Lo que va dentro de un bloque de
/// codigo (``` o ~~~) no cuenta.
pub fn rondas_extra(cuerpo: &crate::tarea::Cuerpo) -> u32 {
    let n: usize = ["Rondas extra", "Ronda extra"]
        .into_iter()
        .filter_map(|t| cuerpo.seccion(t))
        .map(|s| elementos_de_lista(&s.texto))
        .sum();
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// Elementos de lista de primer nivel con texto, fuera de los bloques de
/// codigo.
fn elementos_de_lista(texto: &str) -> usize {
    let mut valla: Option<&str> = None;
    let mut n = 0;
    for l in texto.lines() {
        let t = l.trim_start();
        let abre = ["```", "~~~"].into_iter().find(|v| t.starts_with(v));
        match (valla, abre) {
            (None, Some(v)) => {
                valla = Some(v);
                continue;
            }
            (Some(v), Some(w)) if v == w => {
                valla = None;
                continue;
            }
            (Some(_), _) => continue,
            (None, None) => {}
        }
        let item = l.strip_prefix("- ").or_else(|| l.strip_prefix("* "));
        if item.is_some_and(|x| !x.trim().is_empty()) {
            n += 1;
        }
    }
    n
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
        let d = decidir(None, &rutas(&["x"]), 2, false);
        assert_eq!((d.ronda, d.modo, d.sha_anterior), (1, Modo::Completa, None));
        // Rojo en la ronda 2 con tope 2: tope, sin modelo, con todo arrastrado
        // (tambien el MENOR de un fichero cambiado).
        let rojo = previa(
            Dictamen::Rojo,
            2,
            vec![a("I1", "IMPORTANTE", "a.rs"), a("M1", "MENOR", "b.rs")],
        );
        let d = decidir(Some(&rojo), &rutas(&["a.rs", "b.rs"]), 2, false);
        assert_eq!((d.ronda, d.modo, d.tope), (3, Modo::Tope, 2));
        assert_eq!(ids(&d.abiertos), ["I1", "M1"]);
        assert!(d.cerrados.is_empty());
        // Una ronda extra en la tarea sube el tope: incremental.
        let d = decidir(Some(&rojo), &rutas(&["a.rs", "b.rs"]), 3, false);
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
        let d = decidir(Some(&verde), &rutas(&["b.rs"]), 2, false);
        assert_eq!((d.ronda, d.modo), (2, Modo::Ligera));
        assert_eq!(ids(&d.cerrados), ["M1"]);
        assert_eq!(ids(&d.abiertos), ["M2"]);
        // Un cambio fuera de ellos: incremental (y en verde no hay tope).
        let d = decidir(Some(&verde), &rutas(&["b.rs", "otro.rs"]), 2, false);
        assert_eq!((d.ronda, d.modo), (3, Modo::Incremental));
    }

    /// IMP-1 (rust) y MEN-3 (calidad) de la ronda 1: si desde la ronda con
    /// modelo ya hubo una `ligera`, la siguiente no puede ser otra `ligera`
    /// (lo que entro en ella no lo vio ningun modelo): es `incremental` desde
    /// el commit revisado por modelo.
    #[test]
    fn tras_una_ligera_la_ronda_siguiente_va_con_modelo() {
        let verde = previa(
            Dictamen::Verde,
            2,
            vec![a("M1", "MENOR", "b.rs"), a("M2", "MENOR", "c.rs")],
        );
        let d = decidir(Some(&verde), &rutas(&["b.rs"]), 2, true);
        assert_eq!((d.ronda, d.modo), (3, Modo::Incremental));
        assert_eq!(d.sha_anterior.as_deref(), Some("a".repeat(40).as_str()));
        assert_eq!(ids(&d.cerrados), ["M1"]);
    }

    fn check_con(id: u64, titulo: &str, resumen: &str) -> CheckRun {
        let mut c = check(id, 7, titulo, "completed");
        if let Some(o) = c.output.as_mut() {
            o.summary = Some(resumen.to_owned());
        }
        c
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
        assert_eq!(elegir(commits.clone(), 7).check.map(|c| c.id), Some(1));
        assert_eq!(elegir(commits, 8), Eleccion::default());
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

    /// MEN-1 (calidad) de la ronda 1: con varios checks en el mismo commit,
    /// cuenta el mas nuevo (por `id`, venga en el orden que venga) que no sea
    /// `incompleta`: un rojo seguido de un incompleta en el mismo commit sigue
    /// siendo la ronda anterior.
    #[test]
    fn en_un_commit_cuenta_el_mas_nuevo_que_no_es_incompleta() {
        let rojo = "revision-ia: rojo (ronda=1 criticos=0 importantes=1 menores=0)";
        let verde = "revision-ia: verde (ronda=1 criticos=0 importantes=0 menores=0)";
        let incompleta = "revision-ia: incompleta (sin saldo)";
        for runs in [
            vec![
                check(9, 7, incompleta, "completed"),
                check(8, 7, rojo, "completed"),
            ],
            vec![
                check(8, 7, rojo, "completed"),
                check(9, 7, incompleta, "completed"),
            ],
        ] {
            assert_eq!(elegir(vec![runs], 7).check.map(|c| c.id), Some(8));
        }
        // Dos completos en el mismo commit: el mas nuevo.
        let runs = vec![
            check(8, 7, rojo, "completed"),
            check(10, 7, verde, "completed"),
        ];
        assert_eq!(elegir(vec![runs], 7).check.map(|c| c.id), Some(10));
    }

    /// IMP-1 (rust) de la ronda 1: una ronda `ligera` (lo dice la linea
    /// `Ronda N (ligera)` de su resumen) no sirve de ronda anterior: se elige
    /// el check con modelo de antes y se avisa de que hubo una ligera.
    #[test]
    fn una_ligera_no_sirve_de_ronda_anterior() {
        let verde = "revision-ia: verde (ronda=2 criticos=0 importantes=0 menores=1)";
        let ligera = check_con(
            6,
            verde,
            "**Veredicto: verde** (conclusion success).\n\nRonda 2 (ligera): 0 nuevos, 1 persisten, 1 cerrados.\n",
        );
        let con_modelo = check_con(
            5,
            verde,
            "**Veredicto: verde**.\n\nRonda 2 (incremental): 0 nuevos, 2 persisten, 1 cerrados.\n",
        );
        let e = elegir(vec![vec![ligera.clone()], vec![con_modelo.clone()]], 7);
        assert_eq!(e.check.map(|c| c.id), Some(5));
        assert!(e.ligera_pendiente);
        let e = elegir(vec![vec![con_modelo]], 7);
        assert_eq!(e.check.map(|c| c.id), Some(5));
        assert!(!e.ligera_pendiente);
        // El texto del modelo no finge la linea: va en un elemento de lista.
        let fingida = check_con(
            4,
            verde,
            "Ronda 2 (incremental): 0 nuevos.\n\n- Ronda 2 (ligera): 0 nuevos\n",
        );
        assert_eq!(elegir(vec![vec![fingida]], 7).check.map(|c| c.id), Some(4));
        assert_eq!(modo_de_resumen("Ronda x (ligera): 0"), None);
        assert_eq!(modo_de_resumen("Ronda 3 (otra): 0"), None);
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

    /// MEN-4 (calidad) y MEN-2 (rust) de la ronda 1: `## Rondas extra` cuenta
    /// los elementos `- ` y `* ` con texto, fuera de los bloques de codigo, y
    /// `## Ronda extra` tambien vale (F1-16 nombra las dos).
    #[test]
    fn rondas_extra_cuenta_solo_elementos_con_texto() {
        let n = |t: &str| rondas_extra(&crate::tarea::Cuerpo::leer(t));
        assert_eq!(n("## Rondas extra\n\n* Ronda 3\n* Ronda 4\n"), 2);
        assert_eq!(n("## Rondas extra\n\n- \n-\n*   \n- Ronda 3\n"), 1);
        assert_eq!(
            n("## Rondas extra\n\n```\n- ejemplo\n- otro\n```\n~~~\n* x\n~~~\n- Ronda 3\n"),
            1
        );
        assert_eq!(n("## Ronda extra\n\n- a\n"), 1);
        assert_eq!(n("## Ronda extra\n\n- a\n\n## Rondas extra\n\n- b\n"), 2);
    }
}
