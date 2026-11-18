//! Evaluacion neutral de las puertas (F1-5 y F1-13, PoC M): de lo leido en
//! la plataforma a la linea `estado-puertas ...`. Es pura y comun a GitHub y
//! GitLab, para que las dos den la misma linea byte a byte.

use crate::contrato::{Informe, Resultado};
use crate::i18n::catalogo::plataforma as t;
use crate::i18n::{Mensaje, Texto};

/// Nombre de la operacion.
pub const ESTADO_PUERTAS: &str = "estado-puertas";

/// Nombre de la operacion de escritura de puertas (TASK-118).
pub const APLICAR_PUERTAS: &str = "aplicar-puertas";

/// `aplicar-puertas` rechazada porque los checks `faltan` no se han visto
/// publicados por su origen en la cabeza de `develop` (TASK-118 D2):
/// exigirlos bloquearia `develop` para todos.
pub fn sin_origen_visto(faltan: Vec<String>) -> Informe {
    let diag = Mensaje::nuevo(&t::CHECKS_SIN_ORIGEN_VISTO).con("faltan", faltan.join(", "));
    Informe::nuevo(APLICAR_PUERTAS, Resultado::Rechazada)
        .con("motivo", "puertas-sin-origen")
        .con_lista("faltan", faltan)
        .diagnostico(diag)
}

/// Los checks de `o` que no estan en `vistos` (`(nombre, origen)` de los
/// publicados en la cabeza de `develop`): los de origen fijo tienen que
/// estar publicados por ese origen; los del pipeline, por cualquiera. En el
/// orden del modelo.
pub fn no_vistos(o: &Origenes, vistos: &[(String, u64)]) -> Vec<String> {
    checks_del_modelo()
        .into_iter()
        .filter(|c| match o.de(c) {
            Some(id) => !vistos.iter().any(|(n, x)| n == c && *x == id),
            None => !vistos.iter().any(|(n, _)| n == c),
        })
        .map(str::to_owned)
        .collect()
}

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
            .diagnostico(&t::CHECKS_EN_ROJO);
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
        let diag = Mensaje::nuevo(&t::FALTAN_PUERTAS).con("falta", falta.join(", "));
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
        .diagnostico(&t::PR_CERRADO_REABRE)
}

/// La politica exige impuesta una garantia que el plan no da.
pub fn rechazada_por_plan(pr: u64, garantia: &str) -> Informe {
    Informe::nuevo(ESTADO_PUERTAS, Resultado::Rechazada)
        .con("pr", pr.to_string())
        .con("motivo", "plan")
        .con("garantia", garantia)
        .diagnostico(Mensaje::nuevo(&t::RECHAZADA_POR_PLAN).con("garantia", garantia))
}

/// La plataforma no tiene configurada una garantia que se lee de su
/// configuracion (GitLab: G01, rama destino protegida; G03 y demas de
/// Premium, una regla de aprobacion). `arreglo` dice que hacer.
pub fn rechazada_por_config(pr: u64, garantia: &str, arreglo: &'static Texto) -> Informe {
    Informe::nuevo(ESTADO_PUERTAS, Resultado::Rechazada)
        .con("pr", pr.to_string())
        .con("motivo", "config")
        .con("garantia", garantia)
        .diagnostico(
            Mensaje::nuevo(&t::NO_SE_CUMPLE)
                .con("garantia", garantia)
                .con_mensaje("arreglo", arreglo),
        )
}

/// A que se aplica una puerta del modelo neutral (F1-5, decision 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Objetivo {
    /// Ramas, como refs completas o patrones (`refs/heads/feature/**`).
    Ramas(Vec<String>),
    /// Tags, como patrones (`refs/tags/v*`).
    Tags(Vec<String>),
}

/// Solo por PR (F1-5, decision 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoloPr {
    /// Aprobaciones de personas.
    pub aprobaciones: u8,
}

/// Una puerta del modelo neutral, sin nombres de GitHub ni de GitLab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Puerta {
    /// Nombre sin el prefijo (`develop`, `tags-version`...).
    pub nombre: &'static str,
    /// Ramas o tags.
    pub objetivo: Objetivo,
    /// Solo por PR, con sus aprobaciones; `None` si se empuja directo.
    pub solo_pr: Option<SoloPr>,
    /// Rama al dia con la base antes del merge (`strict`).
    pub al_dia: bool,
    /// Checks obligatorios. Su origen, en [`Origenes`].
    pub checks: Vec<&'static str>,
}

impl Puerta {
    /// Nombre completo, con el prefijo de la huella (`taskcodetwo-develop`).
    pub fn nombre_completo(&self) -> String {
        format!("{}{}", crate::huella::PREFIJO_RULESETS, self.nombre)
    }
}

/// El modelo de puertas de F1-5 (decision 1), con los nombres de la huella
/// (F1-12 D1a): `develop`, `main`, ramas de tarea, rama de estado y tags.
pub fn modelo() -> Vec<Puerta> {
    let rama = |r: &str| format!("refs/heads/{r}");
    vec![
        Puerta {
            nombre: "develop",
            objetivo: Objetivo::Ramas(vec![rama("develop")]),
            solo_pr: Some(SoloPr { aprobaciones: 1 }),
            al_dia: true,
            checks: vec!["verificacion", "revision-ia", "carril"],
        },
        Puerta {
            nombre: "main",
            objetivo: Objetivo::Ramas(vec![rama("main")]),
            solo_pr: Some(SoloPr { aprobaciones: 1 }),
            // F1-5 decision 2: `main` sin `strict` (PoC E).
            al_dia: false,
            checks: vec!["verificacion", "carril"],
        },
        Puerta {
            nombre: "ramas-de-tarea",
            objetivo: Objetivo::Ramas(vec![rama("feature/**"), rama("fix/**"), rama("hotfix/**")]),
            solo_pr: None,
            al_dia: false,
            checks: Vec::new(),
        },
        Puerta {
            nombre: "tareas",
            objetivo: Objetivo::Ramas(vec![rama(crate::huella::RAMA_ESTADO)]),
            solo_pr: None,
            al_dia: false,
            checks: Vec::new(),
        },
        Puerta {
            nombre: "tags-version",
            objetivo: Objetivo::Tags(vec!["refs/tags/v*".to_owned()]),
            solo_pr: None,
            al_dia: false,
            checks: Vec::new(),
        },
    ]
}

/// `integration_id` de GitHub Actions (F1-5, PoC A): el origen de
/// `verificacion` y `carril` en GitHub.
pub const ID_ACTIONS: u64 = 15368;

/// Checks del modelo que publica el revisor IA (su origen es
/// `plataforma.revisor_id`).
const CHECKS_DEL_REVISOR: [&str; 1] = ["revision-ia"];

/// De donde sale cada check obligatorio del modelo (TASK-118, D-id del
/// plan): sin el, una puerta con checks no se aplica.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Origenes {
    /// `(nombre, origen)`: en GitHub, el `integration_id` de la App que lo
    /// publica (`app.id` del check-run); en GitLab, el `author.id` del
    /// estado del commit (F1-13, decision 6).
    pub checks: Vec<(String, u64)>,
    /// Checks sin origen fijo (GitLab: los jobs del pipeline, que impone
    /// «Pipelines must succeed» entero; su autor es quien lanzo el
    /// pipeline, F1-13 G09). Cuentan con que se hayan visto publicados.
    pub del_pipeline: Vec<String>,
}

impl Origenes {
    /// El origen de `check`, si tiene uno fijo.
    pub fn de(&self, check: &str) -> Option<u64> {
        self.checks
            .iter()
            .find(|(n, _)| n == check)
            .map(|(_, id)| *id)
    }
}

/// Los checks obligatorios del modelo, sin repetir, en orden de aparicion.
pub fn checks_del_modelo() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = Vec::new();
    for p in modelo() {
        for c in p.checks {
            if !v.contains(&c) {
                v.push(c);
            }
        }
    }
    v
}

/// Los origenes de los checks del modelo en `proveedor`, con `revisor_id`
/// de la configuracion (`plataforma.revisor_id`; 0 cuenta como ausente).
/// Pura. `Err` con los checks que se quedan sin origen.
///
/// GitHub: `verificacion` y `carril` los publica Actions ([`ID_ACTIONS`]) y
/// `revision-ia` la App del revisor. GitLab: `revision-ia` es un estado del
/// revisor (`author.id`) y `verificacion` y `carril` son del pipeline.
pub fn origenes(
    proveedor: super::Proveedor,
    revisor_id: Option<u64>,
) -> Result<Origenes, Vec<&'static str>> {
    let revisor = revisor_id.filter(|id| *id != 0);
    let mut o = Origenes::default();
    let mut faltan = Vec::new();
    for c in checks_del_modelo() {
        let del_revisor = CHECKS_DEL_REVISOR.contains(&c);
        match (del_revisor, proveedor, revisor) {
            (true, _, Some(id)) => o.checks.push((c.to_owned(), id)),
            (true, _, None) => faltan.push(c),
            (false, super::Proveedor::GitHub, _) => o.checks.push((c.to_owned(), ID_ACTIONS)),
            (false, super::Proveedor::GitLab, _) => o.del_pipeline.push(c.to_owned()),
        }
    }
    if faltan.is_empty() {
        Ok(o)
    } else {
        Err(faltan)
    }
}

/// La puerta traducida a un ruleset de GitHub (la forma de los JSON de la
/// PoC C de F1-5). Sin `origenes` es la propuesta de `init` (checks sin
/// `integration_id` y la clave `_nota`, que GitHub no admite); con ellos,
/// el cuerpo que `aplicar-puertas` envia (TASK-118), con el
/// `integration_id` de cada check.
pub fn ruleset_github(p: &Puerta, origenes: Option<&Origenes>) -> serde_json::Value {
    use serde_json::json;
    let (target, include) = match &p.objetivo {
        Objetivo::Ramas(r) => ("branch", r),
        Objetivo::Tags(t) => ("tag", t),
    };
    let mut reglas = vec![json!({"type": "deletion"})];
    match p.objetivo {
        Objetivo::Ramas(_) => reglas.push(json!({"type": "non_fast_forward"})),
        Objetivo::Tags(_) => reglas.push(json!({
            "type": "update",
            "parameters": {"update_allows_fetch_and_merge": false}
        })),
    }
    if let Some(pr) = p.solo_pr {
        reglas.push(json!({
            "type": "pull_request",
            "parameters": {
                "allowed_merge_methods": ["merge"],
                "dismiss_stale_reviews_on_push": true,
                "require_code_owner_review": true,
                "require_last_push_approval": true,
                "required_approving_review_count": pr.aprobaciones,
                "required_review_thread_resolution": false
            }
        }));
    }
    // Checks sin origen conocido: con origenes no deberia quedar ninguno.
    let mut sin_origen = false;
    if !p.checks.is_empty() {
        let checks: Vec<serde_json::Value> = p
            .checks
            .iter()
            .map(|c| match origenes.and_then(|o| o.de(c)) {
                Some(id) => json!({"context": c, "integration_id": id}),
                None => {
                    sin_origen = true;
                    json!({"context": c})
                }
            })
            .collect();
        reglas.push(json!({
            "type": "required_status_checks",
            "parameters": {
                "strict_required_status_checks_policy": p.al_dia,
                "required_status_checks": checks
            }
        }));
    }
    // Sin `integration_id` cualquier identidad con permiso de escribir checks
    // los daria por buenos (F1-5 lo descarta): la propuesta lleva una clave
    // que GitHub no admite, asi que no se puede aplicar tal cual.
    let nota = sin_origen.then_some(NOTA_SIN_ORIGEN);
    let mut v = json!({
        "name": p.nombre_completo(),
        "target": target,
        "enforcement": "active",
        "bypass_actors": [],
        "conditions": {"ref_name": {"include": include, "exclude": []}},
        "rules": reglas
    });
    if let (Some(n), Some(o)) = (nota, v.as_object_mut()) {
        o.insert("_nota".to_owned(), n.into());
    }
    v
}

/// Nota de una propuesta con checks sin origen: no es aplicable.
pub const NOTA_SIN_ORIGEN: &str = "no aplicable: checks sin integration_id; la aplica tc2 init --aplicar-puertas con plataforma.revisor_id y los checks ya publicados en develop";

#[cfg(test)]
mod tests {
    use super::*;

    /// Los rulesets propuestos llevan el prefijo de la huella y la forma de
    /// la PoC C de F1-5 (`main` sin `strict`, tags sin mover).
    #[test]
    fn modelo_y_rulesets_de_github() {
        let m = modelo();
        let nombres: Vec<String> = m.iter().map(Puerta::nombre_completo).collect();
        assert_eq!(
            nombres,
            [
                "taskcodetwo-develop",
                "taskcodetwo-main",
                "taskcodetwo-ramas-de-tarea",
                "taskcodetwo-tareas",
                "taskcodetwo-tags-version"
            ]
        );
        let tipos = |v: &serde_json::Value| -> Vec<String> {
            v["rules"]
                .as_array()
                .map(|r| {
                    r.iter()
                        .filter_map(|x| x["type"].as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default()
        };
        let develop = ruleset_github(&m[0], None);
        assert_eq!(develop["name"], "taskcodetwo-develop");
        assert_eq!(develop["target"], "branch");
        assert_eq!(
            tipos(&develop),
            [
                "deletion",
                "non_fast_forward",
                "pull_request",
                "required_status_checks"
            ]
        );
        assert_eq!(
            develop["rules"][3]["parameters"]["strict_required_status_checks_policy"],
            true
        );
        let main = ruleset_github(&m[1], None);
        assert_eq!(
            main["rules"][3]["parameters"]["strict_required_status_checks_policy"],
            false
        );
        // Con checks sin origen, la propuesta no es aplicable tal cual.
        assert_eq!(develop["_nota"], NOTA_SIN_ORIGEN);
        assert!(
            develop["rules"][3]["parameters"]["required_status_checks"][0]["integration_id"]
                .is_null()
        );
        let tareas = ruleset_github(&m[3], None);
        assert!(tareas.get("_nota").is_none());
        assert_eq!(
            tareas["conditions"]["ref_name"]["include"][0],
            "refs/heads/taskcodetwo/tareas"
        );
        assert_eq!(tipos(&tareas), ["deletion", "non_fast_forward"]);
        let tags = ruleset_github(&m[4], None);
        assert_eq!(tags["target"], "tag");
        assert_eq!(tipos(&tags), ["deletion", "update"]);
    }

    /// TASK-118, criterio 2 (unitario): sin `revisor_id` (o con 0),
    /// `revision-ia` se queda sin origen en las dos plataformas; con el, en
    /// GitHub `verificacion` y `carril` son de Actions y en GitLab del
    /// pipeline.
    #[test]
    fn origenes_de_los_checks() {
        use super::super::Proveedor;
        for p in [Proveedor::GitHub, Proveedor::GitLab] {
            assert_eq!(origenes(p, None), Err(vec!["revision-ia"]), "{p:?}");
            assert_eq!(origenes(p, Some(0)), Err(vec!["revision-ia"]), "{p:?}");
        }
        let gh = origenes(Proveedor::GitHub, Some(99001)).unwrap_or_default();
        assert_eq!(
            gh.checks,
            [
                ("verificacion".to_owned(), ID_ACTIONS),
                ("revision-ia".to_owned(), 99001),
                ("carril".to_owned(), ID_ACTIONS)
            ]
        );
        assert!(gh.del_pipeline.is_empty());
        let gl = origenes(Proveedor::GitLab, Some(2002)).unwrap_or_default();
        assert_eq!(gl.checks, [("revision-ia".to_owned(), 2002)]);
        assert_eq!(gl.del_pipeline, ["verificacion", "carril"]);
    }

    /// Con origenes, el ruleset lleva el `integration_id` de cada check y no
    /// la `_nota`: es el cuerpo que se envia.
    #[test]
    fn ruleset_con_origenes_es_aplicable() {
        let o = origenes(super::super::Proveedor::GitHub, Some(99001)).unwrap_or_default();
        let m = modelo();
        let develop = ruleset_github(&m[0], Some(&o));
        assert!(develop.get("_nota").is_none(), "{develop}");
        let checks = &develop["rules"][3]["parameters"]["required_status_checks"];
        assert_eq!(
            checks,
            &serde_json::json!([
                {"context": "verificacion", "integration_id": ID_ACTIONS},
                {"context": "revision-ia", "integration_id": 99001},
                {"context": "carril", "integration_id": ID_ACTIONS}
            ])
        );
        // Un origen que falta deja la nota: nunca se envia sin el.
        let parcial = Origenes {
            checks: vec![("verificacion".into(), ID_ACTIONS)],
            del_pipeline: Vec::new(),
        };
        assert_eq!(
            ruleset_github(&m[1], Some(&parcial))["_nota"],
            NOTA_SIN_ORIGEN
        );
    }

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
