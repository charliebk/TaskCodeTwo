//! `capacidades` (F1-13 decision 3): que garantias impone la plataforma con
//! el plan observado, cuales sustituye el CLI y cuales no hay.
//!
//! ```text
//! capacidades(plataforma, plan_observado, puertas) -> Vec<Garantia>
//! ```
//!
//! Funcion pura sobre [`TABLA`], la tabla de equivalencias G01..G20 de la
//! decision 2 compilada en el binario (no se lee del repo: no es
//! configuracion). El plan se observa (`leer-plan` y el 403 de plan de
//! `leer-puertas`); sin plan, «plan no comprobado» y no se supone nada.
//! Una garantia que el plan da pero que las puertas leidas no tienen sale
//! `no-disponible motivo=config arreglo=tc2-init`; solo se mira la
//! configuracion de las que `leer-puertas` lee (G01, G02, G03, G06 y G09);
//! las demas siguen al plan.

use super::{ARREGLO_INIT, Estado, Hallazgo};
use crate::plataforma::{PlanObservado, Proveedor, ReglasLeidas, TierGitLab};

/// Tier de una garantia en GitHub.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TierGitHub {
    /// La dan los rulesets: Free en un repo publico; Pro, Team o
    /// Enterprise en uno privado.
    Rulesets,
    /// Cola de merge: organizacion publica o Enterprise Cloud.
    OrgOEnterprise,
    /// No depende del plan.
    Todos,
}

/// Tier de una garantia en GitLab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TierGitLabFila {
    /// Desde Free.
    Free,
    /// Desde Premium.
    Premium,
    /// Ningun tier la da tal cual.
    Ninguno,
}

/// Una fila de la tabla de equivalencias.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fila {
    /// `G01`...
    pub id: &'static str,
    /// Tier en GitHub.
    pub github: TierGitHub,
    /// Sustituto en GitHub si el plan no la da (nombre corto).
    pub sustituto_github: Option<&'static str>,
    /// Tier en GitLab.
    pub gitlab: TierGitLabFila,
    /// Sustituto en GitLab (nombre corto).
    pub sustituto_gitlab: Option<&'static str>,
}

const fn fila(
    id: &'static str,
    github: TierGitHub,
    sustituto_github: Option<&'static str>,
    gitlab: TierGitLabFila,
    sustituto_gitlab: Option<&'static str>,
) -> Fila {
    Fila {
        id,
        github,
        sustituto_github,
        gitlab,
        sustituto_gitlab,
    }
}

use TierGitHub::{OrgOEnterprise, Rulesets, Todos};
use TierGitLabFila::{Free, Ninguno, Premium};

const AL_DIA: Option<&str> = Some("al-dia-por-cli");
const MERGE_PERSONA: Option<&str> = Some("merge-solo-persona");

/// La tabla de la decision 2 de F1-13 (tiers y sustitutos).
pub const TABLA: [Fila; 20] = [
    fila(
        "G01",
        Rulesets,
        None,
        Ninguno,
        Some("maintainer-solo-persona"),
    ),
    fila("G02", Rulesets, None, Free, None),
    fila("G03", Rulesets, None, Premium, MERGE_PERSONA),
    fila("G04", Rulesets, None, Premium, MERGE_PERSONA),
    fila("G05", Rulesets, None, Premium, MERGE_PERSONA),
    fila("G06", Rulesets, None, Premium, Some("carril-propietarios")),
    fila("G07", Rulesets, None, Free, None),
    fila("G08", Rulesets, None, Ninguno, AL_DIA),
    fila("G09", Rulesets, None, Ninguno, Some("autor-del-estado")),
    fila("G10", OrgOEnterprise, AL_DIA, Premium, AL_DIA),
    fila("G11", Rulesets, None, Free, None),
    fila("G12", Todos, None, Premium, Some("cuenta-de-servicio")),
    fila("G13", Todos, None, Free, None),
    fila("G14", Todos, None, Free, None),
    fila("G15", Todos, None, Free, None),
    fila("G16", Todos, None, Free, None),
    fila("G17", Todos, None, Free, None),
    fila("G18", Todos, None, Free, None),
    fila("G19", Todos, None, Free, None),
    fila("G20", Todos, None, Free, None),
];

/// Estado de una garantia.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoGarantia {
    /// La impone la plataforma.
    Impuesta,
    /// La sustituye el CLI o un job (degradada declarada).
    Sustituida(&'static str),
    /// No la hay.
    NoDisponible,
}

/// Por que una garantia no esta impuesta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motivo {
    /// El plan no la da.
    Plan,
    /// Ningun tier de la plataforma la da.
    Plataforma,
    /// El plan la da y el proyecto no la tiene configurada.
    Config,
}

impl Motivo {
    /// Valor en la linea.
    pub fn texto(self) -> &'static str {
        match self {
            Self::Plan => "plan",
            Self::Plataforma => "plataforma",
            Self::Config => "config",
        }
    }
}

/// Una garantia evaluada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Garantia {
    /// `G01`...
    pub id: &'static str,
    /// Estado.
    pub estado: EstadoGarantia,
    /// Motivo, si no esta impuesta.
    pub motivo: Option<Motivo>,
    /// Arreglo (sin espacios), si no esta impuesta.
    pub arreglo: Option<&'static str>,
}

/// Plan nombrado a partir de lo observado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
    /// GitHub: nombre, si los rulesets estan y si la cola esta.
    GitHub {
        /// `github-free-publico-usuario`...
        nombre: &'static str,
        /// Los rulesets estan en el plan.
        rulesets: bool,
        /// La cola de merge esta en el plan.
        cola: bool,
    },
    /// GitLab con su tier.
    GitLab(TierGitLab),
}

impl Plan {
    /// Valor de `plan=`.
    pub fn nombre(self) -> String {
        match self {
            Self::GitHub { nombre, .. } => nombre.to_owned(),
            Self::GitLab(t) => format!("gitlab-{}", t.nombre()),
        }
    }
}

/// El plan de lo observado; `None`: «plan no comprobado». En GitHub, un
/// privado sin puertas leidas no se sabe si es Pro/Team o Free; Enterprise
/// Cloud no se distingue de Team (la cola sale sustituida: lo conservador).
pub fn plan_de(
    p: Proveedor,
    observado: Option<&PlanObservado>,
    puertas: Option<&ReglasLeidas>,
) -> Option<Plan> {
    match (p, observado) {
        (
            Proveedor::GitHub,
            Some(PlanObservado::GitHub {
                publico,
                organizacion,
            }),
        ) => {
            let (publico, org) = (*publico, *organizacion);
            if publico {
                return Some(Plan::GitHub {
                    nombre: if org {
                        "github-free-publico-org"
                    } else {
                        "github-free-publico-usuario"
                    },
                    rulesets: true,
                    cola: org,
                });
            }
            let rulesets = puertas?.plan_da_reglas;
            let nombre = match (org, rulesets) {
                (false, true) => "github-pro-privado-usuario",
                (false, false) => "github-free-privado-usuario",
                (true, true) => "github-team-privado-org",
                (true, false) => "github-free-privado-org",
            };
            Some(Plan::GitHub {
                nombre,
                rulesets,
                cola: false,
            })
        }
        (Proveedor::GitLab, Some(PlanObservado::GitLab(Some(t)))) => Some(Plan::GitLab(*t)),
        // Sin plan legible, el 403 de las reglas de aprobacion (Premium)
        // con el token del proyecto dice Free (F1-13 decision 3).
        (Proveedor::GitLab, _) if puertas.is_some_and(|r| !r.plan_da_reglas) => {
            Some(Plan::GitLab(TierGitLab::Free))
        }
        _ => None,
    }
}

/// Si las puertas leidas no tienen configurada una garantia que se lee de
/// ellas (`Some(true)`: falta); `None` si no se sabe.
fn falta_config(p: Proveedor, id: &str, r: &ReglasLeidas) -> Option<bool> {
    let tiene = |x: &str| r.reglas_rama.iter().any(|y| y == x);
    match (p, id) {
        (Proveedor::GitHub, "G01") => Some(!(tiene("deletion") && tiene("non_fast_forward"))),
        (_, "G02") => Some(!super::plataforma::solo_pr(p, r)),
        (_, "G03") => Some(r.aprobaciones < 1),
        (Proveedor::GitLab, "G06") => Some(!tiene("code-owner")),
        (Proveedor::GitHub, "G09") => r.checks_requeridos.as_ref().map(Vec::is_empty),
        _ => None,
    }
}

/// `capacidades` (F1-13 decision 3, funcion pura).
pub fn capacidades(p: Proveedor, plan: Plan, puertas: Option<&ReglasLeidas>) -> Vec<Garantia> {
    TABLA
        .iter()
        .map(|f| {
            let g = |estado, motivo, arreglo| Garantia {
                id: f.id,
                estado,
                motivo,
                arreglo,
            };
            let base = match (plan, p) {
                (Plan::GitHub { rulesets, cola, .. }, Proveedor::GitHub) => {
                    let da = match f.github {
                        Rulesets => rulesets,
                        OrgOEnterprise => cola,
                        Todos => true,
                    };
                    let arreglo = match f.github {
                        OrgOEnterprise => "org-publica-o-enterprise-cloud",
                        _ => "repo-publico-o-github-pro",
                    };
                    match (da, f.sustituto_github) {
                        (true, _) => g(EstadoGarantia::Impuesta, None, None),
                        (false, Some(s)) => g(
                            EstadoGarantia::Sustituida(s),
                            Some(Motivo::Plan),
                            Some(arreglo),
                        ),
                        (false, None) => g(
                            EstadoGarantia::NoDisponible,
                            Some(Motivo::Plan),
                            Some(arreglo),
                        ),
                    }
                }
                (Plan::GitLab(t), Proveedor::GitLab) => {
                    let (da, motivo, arreglo) = match f.gitlab {
                        Free => (true, Motivo::Plan, ""),
                        Premium => (t >= TierGitLab::Premium, Motivo::Plan, "gitlab-premium"),
                        Ninguno => (false, Motivo::Plataforma, "ninguno-limite-de-la-plataforma"),
                    };
                    match (da, f.sustituto_gitlab) {
                        (true, _) => g(EstadoGarantia::Impuesta, None, None),
                        (false, Some(s)) => {
                            g(EstadoGarantia::Sustituida(s), Some(motivo), Some(arreglo))
                        }
                        (false, None) => {
                            g(EstadoGarantia::NoDisponible, Some(motivo), Some(arreglo))
                        }
                    }
                }
                // Plan de una plataforma y adaptador de otra: nada impuesto.
                _ => g(EstadoGarantia::NoDisponible, Some(Motivo::Plataforma), None),
            };
            match (base.estado, puertas.and_then(|r| falta_config(p, f.id, r))) {
                (EstadoGarantia::Impuesta, Some(true)) => g(
                    EstadoGarantia::NoDisponible,
                    Some(Motivo::Config),
                    Some(ARREGLO_INIT),
                ),
                _ => base,
            }
        })
        .collect()
}

/// Lo que va en la primera linea de `doctor`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cuentas {
    /// Plan observado (`github-free-publico-usuario`, `gitlab-free`...) o
    /// `no-comprobado`.
    pub plan: String,
    /// `(impuestas, sustituidas, no-disponibles)`; `None` sin plan.
    pub cuentas: Option<(usize, usize, usize)>,
}

/// `(impuestas, sustituidas, no-disponibles)`.
pub fn contar(gs: &[Garantia]) -> (usize, usize, usize) {
    let n = |f: fn(&EstadoGarantia) -> bool| gs.iter().filter(|g| f(&g.estado)).count();
    (
        n(|e| *e == EstadoGarantia::Impuesta),
        n(|e| matches!(e, EstadoGarantia::Sustituida(_))),
        n(|e| *e == EstadoGarantia::NoDisponible),
    )
}

/// El hallazgo de una garantia: `ok` impuesta; si no, `aviso` con su
/// motivo, el sustituto y el arreglo (F1-13 seccion C).
pub fn hallazgo(g: &Garantia) -> Hallazgo {
    let h = Hallazgo::nuevo("garantia", g.id, Estado::Aviso);
    let h = match g.motivo {
        Some(m) => h.motivo(m.texto()),
        None => h,
    };
    let h = match g.estado {
        EstadoGarantia::Impuesta => {
            return Hallazgo::nuevo("garantia", g.id, Estado::Ok).con("capacidad", "impuesta");
        }
        EstadoGarantia::Sustituida(s) => h.con("capacidad", "sustituida").con("por", s),
        EstadoGarantia::NoDisponible => h.con("capacidad", "no-disponible"),
    };
    match g.arreglo {
        Some(a) => h.arreglo(a),
        None => h,
    }
}

/// Hallazgos y cuentas de `capacidades` con lo leido. Sin plan, un solo
/// hallazgo `garantia plan no-comprobado` y `plan=no-comprobado`.
pub fn de_lo_leido(
    p: Option<Proveedor>,
    observado: Option<&PlanObservado>,
    puertas: Option<&ReglasLeidas>,
) -> (Vec<Hallazgo>, Cuentas) {
    let plan = p.and_then(|p| plan_de(p, observado, puertas).map(|x| (p, x)));
    let Some((p, plan)) = plan else {
        return (
            vec![
                Hallazgo::nuevo("garantia", "plan", Estado::NoComprobado)
                    .motivo("plan-no-comprobado"),
            ],
            Cuentas {
                plan: "no-comprobado".to_owned(),
                cuentas: None,
            },
        );
    };
    let gs = capacidades(p, plan, puertas);
    (
        gs.iter().map(hallazgo).collect(),
        Cuentas {
            plan: plan.nombre(),
            cuentas: Some(contar(&gs)),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Puertas leidas con todo lo que `init` configura (para que solo
    /// cuente el plan).
    fn completas(p: Proveedor, plan_da_reglas: bool) -> ReglasLeidas {
        let reglas = match p {
            Proveedor::GitHub => vec!["deletion", "non_fast_forward", "pull_request"],
            Proveedor::GitLab => vec!["push=0", "merge=40", "code-owner"],
        };
        ReglasLeidas {
            plan_da_reglas,
            rulesets: Vec::new(),
            reglas_rama: reglas.into_iter().map(str::to_owned).collect(),
            aprobaciones: 1,
            checks_requeridos: Some(vec!["verificacion".to_owned()]),
            todos_los_checks: p == Proveedor::GitLab,
            checks_publicados: None,
        }
    }

    fn cuentas(
        p: Proveedor,
        o: PlanObservado,
        plan_da_reglas: bool,
    ) -> (String, (usize, usize, usize)) {
        let r = completas(p, plan_da_reglas);
        let plan = plan_de(p, Some(&o), Some(&r));
        assert!(plan.is_some(), "{o:?}");
        let Some(plan) = plan else {
            return (String::new(), (0, 0, 0));
        };
        (plan.nombre(), contar(&capacidades(p, plan, Some(&r))))
    }

    /// C11 (parte pura): el bloque de capacidades de F1-13 sale de la tabla
    /// con el plan observado (impuestas / sustituidas / no disponibles).
    /// `github enterprise-privado-org` no se distingue de Team: sale como
    /// Team (cola sustituida), lo conservador; `free-privado-org` es nuevo.
    #[test]
    fn el_bloque_de_capacidades_de_f1_13() {
        let gh = |publico, organizacion| PlanObservado::GitHub {
            publico,
            organizacion,
        };
        let g = Proveedor::GitHub;
        for (o, reglas, nombre, esperado) in [
            (
                gh(true, false),
                true,
                "github-free-publico-usuario",
                (19, 1, 0),
            ),
            (
                gh(false, false),
                false,
                "github-free-privado-usuario",
                (9, 1, 10),
            ),
            (
                gh(false, false),
                true,
                "github-pro-privado-usuario",
                (19, 1, 0),
            ),
            (gh(true, true), true, "github-free-publico-org", (20, 0, 0)),
            (gh(false, true), true, "github-team-privado-org", (19, 1, 0)),
            (
                gh(false, true),
                false,
                "github-free-privado-org",
                (9, 1, 10),
            ),
        ] {
            assert_eq!(cuentas(g, o, reglas), (nombre.to_owned(), esperado));
        }
        for (t, esperado) in [
            (TierGitLab::Free, (11, 9, 0)),
            (TierGitLab::Premium, (17, 3, 0)),
            (TierGitLab::Ultimate, (17, 3, 0)),
        ] {
            let (_, c) = cuentas(Proveedor::GitLab, PlanObservado::GitLab(Some(t)), true);
            assert_eq!(c, esperado, "{t:?}");
        }
    }

    /// El plan no se supone: GitHub privado sin puertas leidas, GitLab sin
    /// plan legible y con las reglas de aprobacion en 200, o sin nada.
    #[test]
    fn sin_plan_no_se_supone_nada() {
        let privado = PlanObservado::GitHub {
            publico: false,
            organizacion: false,
        };
        assert_eq!(plan_de(Proveedor::GitHub, Some(&privado), None), None);
        assert_eq!(plan_de(Proveedor::GitHub, None, None), None);
        let r = completas(Proveedor::GitLab, true);
        assert_eq!(
            plan_de(
                Proveedor::GitLab,
                Some(&PlanObservado::GitLab(None)),
                Some(&r)
            ),
            None
        );
        // El 403 de las reglas de aprobacion: Free.
        let r = completas(Proveedor::GitLab, false);
        assert_eq!(
            plan_de(Proveedor::GitLab, None, Some(&r)),
            Some(Plan::GitLab(TierGitLab::Free))
        );
        let (hs, c) = de_lo_leido(Some(Proveedor::GitHub), None, None);
        assert_eq!(
            hs.iter().map(Hallazgo::linea).collect::<Vec<_>>(),
            ["garantia plan no-comprobado motivo=plan-no-comprobado"]
        );
        assert_eq!(c.plan, "no-comprobado");
    }

    /// Una garantia que el plan da y las puertas no tienen: `no-disponible
    /// motivo=config arreglo=tc2-init`; sustituida y no disponible por plan,
    /// con su sustituto y su arreglo (la linea de la seccion C de F1-13).
    #[test]
    fn config_sustituidas_y_no_disponibles() {
        let mut r = completas(Proveedor::GitLab, true);
        r.reglas_rama = vec!["push=30".to_owned()];
        let gs = capacidades(Proveedor::GitLab, Plan::GitLab(TierGitLab::Free), Some(&r));
        let l = |id: &str| {
            gs.iter()
                .find(|g| g.id == id)
                .map(|g| hallazgo(g).linea())
                .unwrap_or_default()
        };
        assert_eq!(
            l("G02"),
            "garantia G02 aviso motivo=config capacidad=no-disponible arreglo=tc2-init"
        );
        assert_eq!(
            l("G01"),
            "garantia G01 aviso motivo=plataforma capacidad=sustituida por=maintainer-solo-persona arreglo=ninguno-limite-de-la-plataforma"
        );
        assert_eq!(
            l("G03"),
            "garantia G03 aviso motivo=plan capacidad=sustituida por=merge-solo-persona arreglo=gitlab-premium"
        );
        assert_eq!(l("G13"), "garantia G13 ok capacidad=impuesta");
        let gs = capacidades(
            Proveedor::GitHub,
            Plan::GitHub {
                nombre: "github-free-privado-usuario",
                rulesets: false,
                cola: false,
            },
            None,
        );
        assert_eq!(
            hallazgo(&gs[2]).linea(),
            "garantia G03 aviso motivo=plan capacidad=no-disponible arreglo=repo-publico-o-github-pro"
        );
    }
}
