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
//! `no-disponible motivo=config arreglo=tc2-init`. Solo se mira la
//! configuracion de las que `leer-puertas` lee ([`lee_config`]: en GitHub
//! G01, G02, G03, G04, G05 y G09; en GitLab G02, G03 y G06); las demas
//! siguen al plan. Si la configuracion de una de esas no se pudo leer (sin
//! puertas, o el 403 de las reglas de aprobacion de GitLab con un plan que
//! las da), sale `no-comprobado`: nunca `impuesta` sin leerla (H1 y H4 de
//! TASK-035).

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
    /// El plan la da, pero su configuracion no se pudo leer.
    NoComprobada,
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
    /// No se leyeron las puertas de la rama.
    PuertasNoLeidas,
    /// Un 403 al leer la configuracion con un plan que la da.
    PermisoInsuficiente,
}

impl Motivo {
    /// Valor en la linea.
    pub fn texto(self) -> &'static str {
        match self {
            Self::Plan => "plan",
            Self::Plataforma => "plataforma",
            Self::Config => "config",
            Self::PuertasNoLeidas => "puertas-no-leidas",
            Self::PermisoInsuficiente => "permiso-insuficiente",
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

/// Si `doctor` lee de `leer-puertas` la configuracion de la garantia `id`
/// en `p` (GitHub: los tipos de regla y los `parameters` de `pull_request`
/// y `required_status_checks`; GitLab: la rama protegida y las reglas de
/// aprobacion).
pub fn lee_config(p: Proveedor, id: &str) -> bool {
    matches!(
        (p, id),
        (Proveedor::GitHub, "G01" | "G04" | "G05" | "G09")
            | (_, "G02" | "G03")
            | (Proveedor::GitLab, "G06")
    )
}

/// Lo que dicen las puertas de una garantia que se lee de ellas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Config {
    /// Configurada.
    Tiene,
    /// No configurada.
    Falta,
    /// No se pudo leer, con el motivo.
    NoLeida(Motivo),
}

/// La configuracion de `id` en las puertas leidas; `None` si `doctor` no la
/// lee ([`lee_config`]) y la garantia sigue al plan.
fn config(p: Proveedor, id: &str, puertas: Option<&ReglasLeidas>) -> Option<Config> {
    if !lee_config(p, id) {
        return None;
    }
    let Some(r) = puertas else {
        return Some(Config::NoLeida(Motivo::PuertasNoLeidas));
    };
    let tiene = |x: &str| r.reglas_rama.iter().any(|y| y == x);
    let falta = match (p, id) {
        (Proveedor::GitHub, "G01") => Some(!(tiene("deletion") && tiene("non_fast_forward"))),
        (_, "G02") => Some(!super::plataforma::solo_pr(p, r)),
        // GitLab: el 403 de las reglas de aprobacion. Si el plan las da
        // (si no, G03 ya no esta impuesta), es un permiso, no la config.
        (Proveedor::GitLab, "G03") if !r.plan_da_reglas => {
            return Some(Config::NoLeida(Motivo::PermisoInsuficiente));
        }
        (_, "G03") => Some(r.aprobaciones < 1),
        (Proveedor::GitHub, "G04") => Some(!r.descartar_viejas),
        (Proveedor::GitHub, "G05") => Some(!r.ultimo_push),
        (Proveedor::GitLab, "G06") => Some(!tiene("code-owner")),
        (Proveedor::GitHub, "G09") => r.checks_requeridos.as_ref().map(Vec::is_empty),
        _ => None,
    };
    Some(match falta {
        Some(true) => Config::Falta,
        Some(false) => Config::Tiene,
        None => Config::NoLeida(Motivo::PuertasNoLeidas),
    })
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
            if base.estado != EstadoGarantia::Impuesta {
                return base;
            }
            match config(p, f.id, puertas) {
                Some(Config::Falta) => g(
                    EstadoGarantia::NoDisponible,
                    Some(Motivo::Config),
                    Some(ARREGLO_INIT),
                ),
                Some(Config::NoLeida(m)) => g(EstadoGarantia::NoComprobada, Some(m), None),
                Some(Config::Tiene) | None => base,
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
    /// Garantias que el plan da y cuya configuracion no se pudo leer (no
    /// cuentan como impuestas; `no-comprobadas=N` en la primera linea si
    /// hay alguna).
    pub no_comprobadas: usize,
}

/// `(impuestas, sustituidas, no-disponibles)`; las no comprobadas no
/// cuentan en ninguna ([`no_comprobadas`]).
pub fn contar(gs: &[Garantia]) -> (usize, usize, usize) {
    let n = |f: fn(&EstadoGarantia) -> bool| gs.iter().filter(|g| f(&g.estado)).count();
    (
        n(|e| *e == EstadoGarantia::Impuesta),
        n(|e| matches!(e, EstadoGarantia::Sustituida(_))),
        n(|e| *e == EstadoGarantia::NoDisponible),
    )
}

/// Cuantas garantias salen `no-comprobado`.
pub fn no_comprobadas(gs: &[Garantia]) -> usize {
    gs.iter()
        .filter(|g| g.estado == EstadoGarantia::NoComprobada)
        .count()
}

/// El hallazgo de una garantia: `ok` impuesta; `no-comprobado` con su
/// motivo si su configuracion no se leyo; si no, `aviso` con su motivo, el
/// sustituto y el arreglo (F1-13 seccion C).
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
        EstadoGarantia::NoComprobada => {
            let x = Hallazgo::nuevo("garantia", g.id, Estado::NoComprobado);
            return match g.motivo {
                Some(m) => x.motivo(m.texto()),
                None => x,
            };
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
                no_comprobadas: 0,
            },
        );
    };
    let gs = capacidades(p, plan, puertas);
    (
        gs.iter().map(hallazgo).collect(),
        Cuentas {
            plan: plan.nombre(),
            cuentas: Some(contar(&gs)),
            no_comprobadas: no_comprobadas(&gs),
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
            fallo_checks: None,
            descartar_viejas: p == Proveedor::GitHub,
            ultimo_push: p == Proveedor::GitHub,
        }
    }

    const GH_PUBLICO: Plan = Plan::GitHub {
        nombre: "github-free-publico-usuario",
        rulesets: true,
        cola: false,
    };

    /// Las lineas de las garantias que no son `ok`.
    fn no_ok(gs: &[Garantia]) -> Vec<String> {
        gs.iter()
            .map(hallazgo)
            .filter(|h| h.estado != Estado::Ok)
            .map(|h| h.linea())
            .collect()
    }

    /// H2 de TASK-035: cada garantia cuya configuracion se lee de las
    /// puertas de GitHub sale `no-disponible motivo=config` cuando falta
    /// (y solo ella); con todo configurado, ninguna. Mata las mutaciones
    /// `Some(false)` de G01, G03 y G09 (y las de G04 y G05).
    #[test]
    fn config_de_github_garantia_a_garantia() {
        let p = Proveedor::GitHub;
        let r = completas(p, true);
        let base = capacidades(p, GH_PUBLICO, Some(&r));
        assert_eq!(
            no_ok(&base),
            [
                "garantia G10 aviso motivo=plan capacidad=sustituida por=al-dia-por-cli arreglo=org-publica-o-enterprise-cloud"
            ]
        );
        let config = |id: &str| {
            format!("garantia {id} aviso motivo=config capacidad=no-disponible arreglo=tc2-init")
        };
        type Cambio = fn(&mut ReglasLeidas);
        let casos: [(&str, Cambio); 6] = [
            ("G01", |r| r.reglas_rama.retain(|x| x != "deletion")),
            ("G01", |r| r.reglas_rama.retain(|x| x != "non_fast_forward")),
            ("G03", |r| r.aprobaciones = 0),
            ("G04", |r| r.descartar_viejas = false),
            ("G05", |r| r.ultimo_push = false),
            ("G09", |r| r.checks_requeridos = Some(Vec::new())),
        ];
        for (id, cambiar) in casos {
            let mut r = completas(p, true);
            cambiar(&mut r);
            let gs = capacidades(p, GH_PUBLICO, Some(&r));
            let l = no_ok(&gs);
            assert!(l.contains(&config(id)), "{id}: {l:?}");
            assert_eq!(l.len(), 2, "{id}: {l:?}");
            assert_eq!(contar(&gs), (18, 1, 1), "{id}");
        }
    }

    /// H1 de TASK-035: sin puertas leidas, las garantias cuya configuracion
    /// `doctor` lee salen `no-comprobado motivo=puertas-no-leidas` y no
    /// cuentan como impuestas; las demas siguen al plan.
    #[test]
    fn sin_puertas_lo_que_se_lee_no_esta_comprobado() {
        let gs = capacidades(Proveedor::GitHub, GH_PUBLICO, None);
        let nc: Vec<&str> = gs
            .iter()
            .filter(|g| g.estado == EstadoGarantia::NoComprobada)
            .map(|g| g.id)
            .collect();
        assert_eq!(nc, ["G01", "G02", "G03", "G04", "G05", "G09"]);
        assert_eq!(contar(&gs), (13, 1, 0));
        assert_eq!(no_comprobadas(&gs), 6);
        assert_eq!(
            hallazgo(&gs[0]).linea(),
            "garantia G01 no-comprobado motivo=puertas-no-leidas"
        );
        let gl = Plan::GitLab(TierGitLab::Premium);
        let gs = capacidades(Proveedor::GitLab, gl, None);
        let nc: Vec<&str> = gs
            .iter()
            .filter(|g| g.estado == EstadoGarantia::NoComprobada)
            .map(|g| g.id)
            .collect();
        assert_eq!(nc, ["G02", "G03", "G06"]);
        assert_eq!((contar(&gs), no_comprobadas(&gs)), ((14, 3, 0), 3));
        // GitHub sin `checks_requeridos` leidos: G09 tampoco se da.
        let mut r = completas(Proveedor::GitHub, true);
        r.checks_requeridos = None;
        let gs = capacidades(Proveedor::GitHub, GH_PUBLICO, Some(&r));
        assert_eq!(
            hallazgo(&gs[8]).linea(),
            "garantia G09 no-comprobado motivo=puertas-no-leidas"
        );
        // La primera linea las cuenta aparte.
        let (_, c) = de_lo_leido(
            Some(Proveedor::GitLab),
            Some(&PlanObservado::GitLab(Some(TierGitLab::Premium))),
            None,
        );
        assert_eq!((c.cuentas, c.no_comprobadas), (Some((14, 3, 0)), 3));
    }

    /// H4 de TASK-035: el 403 de las reglas de aprobacion de GitLab con un
    /// plan legible que las da no es config ausente: `no-comprobado
    /// motivo=permiso-insuficiente`. En Free, G03 sigue sustituida.
    #[test]
    fn el_403_de_aprobaciones_con_premium_es_un_permiso() {
        let r = completas(Proveedor::GitLab, false);
        let gs = capacidades(
            Proveedor::GitLab,
            Plan::GitLab(TierGitLab::Premium),
            Some(&r),
        );
        assert_eq!(
            hallazgo(&gs[2]).linea(),
            "garantia G03 no-comprobado motivo=permiso-insuficiente"
        );
        assert_eq!((contar(&gs), no_comprobadas(&gs)), ((16, 3, 0), 1));
        let gs = capacidades(Proveedor::GitLab, Plan::GitLab(TierGitLab::Free), Some(&r));
        assert_eq!(
            gs[2].estado,
            EstadoGarantia::Sustituida("merge-solo-persona")
        );
    }

    /// Tier de GitHub de una celda de la tabla de F1-13.
    fn tier_github(c: &str) -> Option<TierGitHub> {
        match c {
            "Free público; Pro/Team/Enterprise" => Some(TierGitHub::Rulesets),
            "org pública; Enterprise Cloud" => Some(TierGitHub::OrgOEnterprise),
            "todos" => Some(TierGitHub::Todos),
            _ => None,
        }
    }

    /// Tier de GitLab de una celda.
    fn tier_gitlab(c: &str) -> Option<TierGitLabFila> {
        match c {
            "Free" => Some(TierGitLabFila::Free),
            "Premium" => Some(TierGitLabFila::Premium),
            "ninguno" => Some(TierGitLabFila::Ninguno),
            _ => None,
        }
    }

    /// El nombre corto de un sustituto (`—`: ninguno; si no, el primer
    /// texto entre comillas invertidas).
    fn sustituto(c: &str) -> Option<String> {
        let c = c.trim();
        if c == "—" {
            return None;
        }
        c.strip_prefix('`')
            .and_then(|r| r.split('`').next())
            .map(str::to_owned)
            .or_else(|| Some(format!("ilegible:{c}")))
    }

    /// R1-01 de TASK-035: la tabla compilada es la del bloque
    /// `tabla-equivalencias` de F1-13, fila a fila (tier GitHub, tier GitLab
    /// y los dos sustitutos).
    #[test]
    fn la_tabla_compilada_es_la_de_f1_13() {
        let ruta = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/adr/F1-13-adaptador-de-plataforma.md");
        let texto = std::fs::read_to_string(&ruta).unwrap_or_default();
        let bloque = texto
            .split("<!-- tabla-equivalencias:inicio -->")
            .nth(1)
            .and_then(|r| r.split("<!-- tabla-equivalencias:fin -->").next())
            .unwrap_or_default();
        type FilaAdr = (
            String,
            Option<TierGitHub>,
            Option<String>,
            Option<TierGitLabFila>,
            Option<String>,
        );
        let adr: Vec<FilaAdr> = bloque
            .lines()
            .map(str::trim)
            .filter(|l| l.starts_with("| G"))
            .map(|l| {
                let c: Vec<&str> = l.split('|').map(str::trim).collect();
                assert_eq!(c.len(), 11, "{l}");
                let (gh, gl) = c[8].split_once(" / GitLab: ").unwrap_or(("", ""));
                (
                    c[1].to_owned(),
                    tier_github(c[5]),
                    sustituto(gh.strip_prefix("GitHub: ").unwrap_or("?")),
                    tier_gitlab(c[7]),
                    sustituto(gl),
                )
            })
            .collect();
        let compilada: Vec<FilaAdr> = TABLA
            .iter()
            .map(|f| {
                (
                    f.id.to_owned(),
                    Some(f.github),
                    f.sustituto_github.map(str::to_owned),
                    Some(f.gitlab),
                    f.sustituto_gitlab.map(str::to_owned),
                )
            })
            .collect();
        assert_eq!(adr.len(), 20, "{}", ruta.display());
        for (a, c) in adr.iter().zip(&compilada) {
            assert_eq!(a, c);
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
