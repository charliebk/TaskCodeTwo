# Peticion de revision — TASK-035 (ronda 1)

- Tarea: TASK-035 — F3-8 doctor: entorno, rama de estado, permisos, rulesets y checks
- Rama revisada: feature/task-035-f3-8-doctor-entorno-rama-de-estado-permi
- Rama base: develop
- Commit revisado (HEAD): 1aa0a12edd779a7211e471d9132d566be737d3da
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-035 (criterios de aceptacion y plan)
- Alcance: completa (motivo: primera ronda de revision)

## Instrucciones para el agente revisor

Eres un revisor INDEPENDIENTE del agente que implemento. Tu trabajo es
reproducir empiricamente, no leer el diff y opinar: clona el repo a un
directorio temporal, corre la suite tu mismo (salvo que esta
peticion ya traiga la verificacion de la ronda en verde para el commit revisado;
entonces vale ese resultado) y construye el caso que
rompe el codigo antes de reportarlo. Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. Vuelca tu salida en el
informe de esta ronda (informe-revision-1-rust-wasm-reviewer.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-035/tarea.md
- tareas/03-en-revision/TASK-035/planificacion/
- crates/tc2/src/doctor/capacidades.rs
- crates/tc2/src/doctor/entorno.rs
- crates/tc2/src/doctor/estado.rs
- crates/tc2/src/doctor/mod.rs
- crates/tc2/src/doctor/plataforma.rs
- crates/tc2/src/git/puerto.rs
- crates/tc2/src/i18n/catalogo/cli.rs
- crates/tc2/src/i18n/catalogo/doctor.rs
- crates/tc2/src/i18n/catalogo/mod.rs
- crates/tc2/src/identidad.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/plataforma/github.rs
- crates/tc2/src/plataforma/gitlab.rs
- crates/tc2/src/plataforma/mod.rs
- crates/tc2/src/tablero.rs
- crates/tc2/tests/doctor.rs

## Commits a revisar (git log develop..HEAD)

````
1aa0a12 docs(TASK-035): autorrevision con rust-wasm-reviewer y code-quality-reviewer
d505e92 docs(TASK-035): adendas de F1-13 y F1-4 por doctor
7fb0bf0 feat(TASK-035): capacidades G01..G20 con el plan observado en doctor
7c143e9 feat(TASK-035): checks requeridos y publicados en leer-puertas y doctor
dfc9098 refactor(TASK-035): conexion con la plataforma en identidad y proteccion en doctor
a3363c8 feat(TASK-035): doctor lee la rama de estado y el plan de init
529d54a feat(TASK-035): esqueleto de doctor, agregador, salida y entorno
c2c9e8a chore(TASK-035): tarea en curso
````

## Diff de tu dominio (16 fichero(s) de 18; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/doctor/capacidades.rs b/crates/tc2/src/doctor/capacidades.rs
new file mode 100644
index 0000000..7d10b45
--- /dev/null
+++ b/crates/tc2/src/doctor/capacidades.rs
@@ -0,0 +1,546 @@
+//! `capacidades` (F1-13 decision 3): que garantias impone la plataforma con
+//! el plan observado, cuales sustituye el CLI y cuales no hay.
+//!
+//! ```text
+//! capacidades(plataforma, plan_observado, puertas) -> Vec<Garantia>
+//! ```
+//!
+//! Funcion pura sobre [`TABLA`], la tabla de equivalencias G01..G20 de la
+//! decision 2 compilada en el binario (no se lee del repo: no es
+//! configuracion). El plan se observa (`leer-plan` y el 403 de plan de
+//! `leer-puertas`); sin plan, «plan no comprobado» y no se supone nada.
+//! Una garantia que el plan da pero que las puertas leidas no tienen sale
+//! `no-disponible motivo=config arreglo=tc2-init`; solo se mira la
+//! configuracion de las que `leer-puertas` lee (G01, G02, G03, G06 y G09);
+//! las demas siguen al plan.
+
+use super::{ARREGLO_INIT, Estado, Hallazgo};
+use crate::plataforma::{PlanObservado, Proveedor, ReglasLeidas, TierGitLab};
+
+/// Tier de una garantia en GitHub.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum TierGitHub {
+    /// La dan los rulesets: Free en un repo publico; Pro, Team o
+    /// Enterprise en uno privado.
+    Rulesets,
+    /// Cola de merge: organizacion publica o Enterprise Cloud.
+    OrgOEnterprise,
+    /// No depende del plan.
+    Todos,
+}
+
+/// Tier de una garantia en GitLab.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum TierGitLabFila {
+    /// Desde Free.
+    Free,
+    /// Desde Premium.
+    Premium,
+    /// Ningun tier la da tal cual.
+    Ninguno,
+}
+
+/// Una fila de la tabla de equivalencias.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub struct Fila {
+    /// `G01`...
+    pub id: &'static str,
+    /// Tier en GitHub.
+    pub github: TierGitHub,
+    /// Sustituto en GitHub si el plan no la da (nombre corto).
+    pub sustituto_github: Option<&'static str>,
+    /// Tier en GitLab.
+    pub gitlab: TierGitLabFila,
+    /// Sustituto en GitLab (nombre corto).
+    pub sustituto_gitlab: Option<&'static str>,
+}
+
+const fn fila(
+    id: &'static str,
+    github: TierGitHub,
+    sustituto_github: Option<&'static str>,
+    gitlab: TierGitLabFila,
+    sustituto_gitlab: Option<&'static str>,
+) -> Fila {
+    Fila {
+        id,
+        github,
+        sustituto_github,
+        gitlab,
+        sustituto_gitlab,
+    }
+}
+
+use TierGitHub::{OrgOEnterprise, Rulesets, Todos};
+use TierGitLabFila::{Free, Ninguno, Premium};
+
+const AL_DIA: Option<&str> = Some("al-dia-por-cli");
+const MERGE_PERSONA: Option<&str> = Some("merge-solo-persona");
+
+/// La tabla de la decision 2 de F1-13 (tiers y sustitutos).
+pub const TABLA: [Fila; 20] = [
+    fila(
+        "G01",
+        Rulesets,
+        None,
+        Ninguno,
+        Some("maintainer-solo-persona"),
+    ),
+    fila("G02", Rulesets, None, Free, None),
+    fila("G03", Rulesets, None, Premium, MERGE_PERSONA),
+    fila("G04", Rulesets, None, Premium, MERGE_PERSONA),
+    fila("G05", Rulesets, None, Premium, MERGE_PERSONA),
+    fila("G06", Rulesets, None, Premium, Some("carril-propietarios")),
+    fila("G07", Rulesets, None, Free, None),
+    fila("G08", Rulesets, None, Ninguno, AL_DIA),
+    fila("G09", Rulesets, None, Ninguno, Some("autor-del-estado")),
+    fila("G10", OrgOEnterprise, AL_DIA, Premium, AL_DIA),
+    fila("G11", Rulesets, None, Free, None),
+    fila("G12", Todos, None, Premium, Some("cuenta-de-servicio")),
+    fila("G13", Todos, None, Free, None),
+    fila("G14", Todos, None, Free, None),
+    fila("G15", Todos, None, Free, None),
+    fila("G16", Todos, None, Free, None),
+    fila("G17", Todos, None, Free, None),
+    fila("G18", Todos, None, Free, None),
+    fila("G19", Todos, None, Free, None),
+    fila("G20", Todos, None, Free, None),
+];
+
+/// Estado de una garantia.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum EstadoGarantia {
+    /// La impone la plataforma.
+    Impuesta,
+    /// La sustituye el CLI o un job (degradada declarada).
+    Sustituida(&'static str),
+    /// No la hay.
+    NoDisponible,
+}
+
+/// Por que una garantia no esta impuesta.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Motivo {
+    /// El plan no la da.
+    Plan,
+    /// Ningun tier de la plataforma la da.
+    Plataforma,
+    /// El plan la da y el proyecto no la tiene configurada.
+    Config,
+}
+
+impl Motivo {
+    /// Valor en la linea.
+    pub fn texto(self) -> &'static str {
+        match self {
+            Self::Plan => "plan",
+            Self::Plataforma => "plataforma",
+            Self::Config => "config",
+        }
+    }
+}
+
+/// Una garantia evaluada.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub struct Garantia {
+    /// `G01`...
+    pub id: &'static str,
+    /// Estado.
+    pub estado: EstadoGarantia,
+    /// Motivo, si no esta impuesta.
+    pub motivo: Option<Motivo>,
+    /// Arreglo (sin espacios), si no esta impuesta.
+    pub arreglo: Option<&'static str>,
+}
+
+/// Plan nombrado a partir de lo observado.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Plan {
+    /// GitHub: nombre, si los rulesets estan y si la cola esta.
+    GitHub {
+        /// `github-free-publico-usuario`...
+        nombre: &'static str,
+        /// Los rulesets estan en el plan.
+        rulesets: bool,
+        /// La cola de merge esta en el plan.
+        cola: bool,
+    },
+    /// GitLab con su tier.
+    GitLab(TierGitLab),
+}
+
+impl Plan {
+    /// Valor de `plan=`.
+    pub fn nombre(self) -> String {
+        match self {
+            Self::GitHub { nombre, .. } => nombre.to_owned(),
+            Self::GitLab(t) => format!("gitlab-{}", t.nombre()),
+        }
+    }
+}
+
+/// El plan de lo observado; `None`: «plan no comprobado». En GitHub, un
+/// privado sin puertas leidas no se sabe si es Pro/Team o Free; Enterprise
+/// Cloud no se distingue de Team (la cola sale sustituida: lo conservador).
+pub fn plan_de(
+    p: Proveedor,
+    observado: Option<&PlanObservado>,
+    puertas: Option<&ReglasLeidas>,
+) -> Option<Plan> {
+    match (p, observado) {
+        (
+            Proveedor::GitHub,
+            Some(PlanObservado::GitHub {
+                publico,
+                organizacion,
+            }),
+        ) => {
+            let (publico, org) = (*publico, *organizacion);
+            if publico {
+                return Some(Plan::GitHub {
+                    nombre: if org {
+                        "github-free-publico-org"
+                    } else {
+                        "github-free-publico-usuario"
+                    },
+                    rulesets: true,
+                    cola: org,
+                });
+            }
+            let rulesets = puertas?.plan_da_reglas;
+            let nombre = match (org, rulesets) {
+                (false, true) => "github-pro-privado-usuario",
+                (false, false) => "github-free-privado-usuario",
+                (true, true) => "github-team-privado-org",
+                (true, false) => "github-free-privado-org",
+            };
+            Some(Plan::GitHub {
+                nombre,
+                rulesets,
+                cola: false,
+            })
+        }
+        (Proveedor::GitLab, Some(PlanObservado::GitLab(Some(t)))) => Some(Plan::GitLab(*t)),
+        // Sin plan legible, el 403 de las reglas de aprobacion (Premium)
+        // con el token del proyecto dice Free (F1-13 decision 3).
+        (Proveedor::GitLab, _) if puertas.is_some_and(|r| !r.plan_da_reglas) => {
+            Some(Plan::GitLab(TierGitLab::Free))
+        }
+        _ => None,
+    }
+}
+
+/// Si las puertas leidas no tienen configurada una garantia que se lee de
+/// ellas (`Some(true)`: falta); `None` si no se sabe.
+fn falta_config(p: Proveedor, id: &str, r: &ReglasLeidas) -> Option<bool> {
+    let tiene = |x: &str| r.reglas_rama.iter().any(|y| y == x);
+    match (p, id) {
+        (Proveedor::GitHub, "G01") => Some(!(tiene("deletion") && tiene("non_fast_forward"))),
+        (_, "G02") => Some(!super::plataforma::solo_pr(p, r)),
+        (_, "G03") => Some(r.aprobaciones < 1),
+        (Proveedor::GitLab, "G06") => Some(!tiene("code-owner")),
+        (Proveedor::GitHub, "G09") => r.checks_requeridos.as_ref().map(Vec::is_empty),
+        _ => None,
+    }
+}
+
+/// `capacidades` (F1-13 decision 3, funcion pura).
+pub fn capacidades(p: Proveedor, plan: Plan, puertas: Option<&ReglasLeidas>) -> Vec<Garantia> {
+    TABLA
+        .iter()
+        .map(|f| {
+            let g = |estado, motivo, arreglo| Garantia {
+                id: f.id,
+                estado,
+                motivo,
+                arreglo,
+            };
+            let base = match (plan, p) {
+                (Plan::GitHub { rulesets, cola, .. }, Proveedor::GitHub) => {
+                    let da = match f.github {
+                        Rulesets => rulesets,
+                        OrgOEnterprise => cola,
+                        Todos => true,
+                    };
+                    let arreglo = match f.github {
+                        OrgOEnterprise => "org-publica-o-enterprise-cloud",
+                        _ => "repo-publico-o-github-pro",
+                    };
+                    match (da, f.sustituto_github) {
+                        (true, _) => g(EstadoGarantia::Impuesta, None, None),
+                        (false, Some(s)) => g(
+                            EstadoGarantia::Sustituida(s),
+                            Some(Motivo::Plan),
+                            Some(arreglo),
+                        ),
+                        (false, None) => g(
+                            EstadoGarantia::NoDisponible,
+                            Some(Motivo::Plan),
+                            Some(arreglo),
+                        ),
+                    }
+                }
+                (Plan::GitLab(t), Proveedor::GitLab) => {
+                    let (da, motivo, arreglo) = match f.gitlab {
+                        Free => (true, Motivo::Plan, ""),
+                        Premium => (t >= TierGitLab::Premium, Motivo::Plan, "gitlab-premium"),
+                        Ninguno => (false, Motivo::Plataforma, "ninguno-limite-de-la-plataforma"),
+                    };
+                    match (da, f.sustituto_gitlab) {
+                        (true, _) => g(EstadoGarantia::Impuesta, None, None),
+                        (false, Some(s)) => {
+                            g(EstadoGarantia::Sustituida(s), Some(motivo), Some(arreglo))
+                        }
+                        (false, None) => {
+                            g(EstadoGarantia::NoDisponible, Some(motivo), Some(arreglo))
+                        }
+                    }
+                }
+                // Plan de una plataforma y adaptador de otra: nada impuesto.
+                _ => g(EstadoGarantia::NoDisponible, Some(Motivo::Plataforma), None),
+            };
+            match (base.estado, puertas.and_then(|r| falta_config(p, f.id, r))) {
+                (EstadoGarantia::Impuesta, Some(true)) => g(
+                    EstadoGarantia::NoDisponible,
+                    Some(Motivo::Config),
+                    Some(ARREGLO_INIT),
+                ),
+                _ => base,
+            }
+        })
+        .collect()
+}
+
+/// Lo que va en la primera linea de `doctor`.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Cuentas {
+    /// Plan observado (`github-free-publico-usuario`, `gitlab-free`...) o
+    /// `no-comprobado`.
+    pub plan: String,
+    /// `(impuestas, sustituidas, no-disponibles)`; `None` sin plan.
+    pub cuentas: Option<(usize, usize, usize)>,
+}
+
+/// `(impuestas, sustituidas, no-disponibles)`.
+pub fn contar(gs: &[Garantia]) -> (usize, usize, usize) {
+    let n = |f: fn(&EstadoGarantia) -> bool| gs.iter().filter(|g| f(&g.estado)).count();
+    (
+        n(|e| *e == EstadoGarantia::Impuesta),
+        n(|e| matches!(e, EstadoGarantia::Sustituida(_))),
+        n(|e| *e == EstadoGarantia::NoDisponible),
+    )
+}
+
+/// El hallazgo de una garantia: `ok` impuesta; si no, `aviso` con su
+/// motivo, el sustituto y el arreglo (F1-13 seccion C).
+pub fn hallazgo(g: &Garantia) -> Hallazgo {
+    let h = Hallazgo::nuevo("garantia", g.id, Estado::Aviso);
+    let h = match g.motivo {
+        Some(m) => h.motivo(m.texto()),
+        None => h,
+    };
+    let h = match g.estado {
+        EstadoGarantia::Impuesta => {
+            return Hallazgo::nuevo("garantia", g.id, Estado::Ok).con("capacidad", "impuesta");
+        }
+        EstadoGarantia::Sustituida(s) => h.con("capacidad", "sustituida").con("por", s),
+        EstadoGarantia::NoDisponible => h.con("capacidad", "no-disponible"),
+    };
+    match g.arreglo {
+        Some(a) => h.arreglo(a),
+        None => h,
+    }
+}
+
+/// Hallazgos y cuentas de `capacidades` con lo leido. Sin plan, un solo
+/// hallazgo `garantia plan no-comprobado` y `plan=no-comprobado`.
+pub fn de_lo_leido(
+    p: Option<Proveedor>,
+    observado: Option<&PlanObservado>,
+    puertas: Option<&ReglasLeidas>,
+) -> (Vec<Hallazgo>, Cuentas) {
+    let plan = p.and_then(|p| plan_de(p, observado, puertas).map(|x| (p, x)));
+    let Some((p, plan)) = plan else {
+        return (
+            vec![
+                Hallazgo::nuevo("garantia", "plan", Estado::NoComprobado)
+                    .motivo("plan-no-comprobado"),
+            ],
+            Cuentas {
+                plan: "no-comprobado".to_owned(),
+                cuentas: None,
+            },
+        );
+    };
+    let gs = capacidades(p, plan, puertas);
+    (
+        gs.iter().map(hallazgo).collect(),
+        Cuentas {
+            plan: plan.nombre(),
+            cuentas: Some(contar(&gs)),
+        },
+    )
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    /// Puertas leidas con todo lo que `init` configura (para que solo
+    /// cuente el plan).
+    fn completas(p: Proveedor, plan_da_reglas: bool) -> ReglasLeidas {
+        let reglas = match p {
+            Proveedor::GitHub => vec!["deletion", "non_fast_forward", "pull_request"],
+            Proveedor::GitLab => vec!["push=0", "merge=40", "code-owner"],
+        };
+        ReglasLeidas {
+            plan_da_reglas,
+            rulesets: Vec::new(),
+            reglas_rama: reglas.into_iter().map(str::to_owned).collect(),
+            aprobaciones: 1,
+            checks_requeridos: Some(vec!["verificacion".to_owned()]),
+            todos_los_checks: p == Proveedor::GitLab,
+            checks_publicados: None,
+        }
+    }
+
+    fn cuentas(
+        p: Proveedor,
+        o: PlanObservado,
+        plan_da_reglas: bool,
+    ) -> (String, (usize, usize, usize)) {
+        let r = completas(p, plan_da_reglas);
+        let plan = plan_de(p, Some(&o), Some(&r));
+        assert!(plan.is_some(), "{o:?}");
+        let Some(plan) = plan else {
+            return (String::new(), (0, 0, 0));
+        };
+        (plan.nombre(), contar(&capacidades(p, plan, Some(&r))))
+    }
+
+    /// C11 (parte pura): el bloque de capacidades de F1-13 sale de la tabla
+    /// con el plan observado (impuestas / sustituidas / no disponibles).
+    /// `github enterprise-privado-org` no se distingue de Team: sale como
+    /// Team (cola sustituida), lo conservador; `free-privado-org` es nuevo.
+    #[test]
+    fn el_bloque_de_capacidades_de_f1_13() {
+        let gh = |publico, organizacion| PlanObservado::GitHub {
+            publico,
+            organizacion,
+        };
+        let g = Proveedor::GitHub;
+        for (o, reglas, nombre, esperado) in [
+            (
+                gh(true, false),
+                true,
+                "github-free-publico-usuario",
+                (19, 1, 0),
+            ),
+            (
+                gh(false, false),
+                false,
+                "github-free-privado-usuario",
+                (9, 1, 10),
+            ),
+            (
+                gh(false, false),
+                true,
+                "github-pro-privado-usuario",
+                (19, 1, 0),
+            ),
+            (gh(true, true), true, "github-free-publico-org", (20, 0, 0)),
+            (gh(false, true), true, "github-team-privado-org", (19, 1, 0)),
+            (
+                gh(false, true),
+                false,
+                "github-free-privado-org",
+                (9, 1, 10),
+            ),
+        ] {
+            assert_eq!(cuentas(g, o, reglas), (nombre.to_owned(), esperado));
+        }
+        for (t, esperado) in [
+            (TierGitLab::Free, (11, 9, 0)),
+            (TierGitLab::Premium, (17, 3, 0)),
+            (TierGitLab::Ultimate, (17, 3, 0)),
+        ] {
+            let (_, c) = cuentas(Proveedor::GitLab, PlanObservado::GitLab(Some(t)), true);
+            assert_eq!(c, esperado, "{t:?}");
+        }
+    }
+
+    /// El plan no se supone: GitHub privado sin puertas leidas, GitLab sin
+    /// plan legible y con las reglas de aprobacion en 200, o sin nada.
+    #[test]
+    fn sin_plan_no_se_supone_nada() {
+        let privado = PlanObservado::GitHub {
+            publico: false,
+            organizacion: false,
+        };
+        assert_eq!(plan_de(Proveedor::GitHub, Some(&privado), None), None);
+        assert_eq!(plan_de(Proveedor::GitHub, None, None), None);
+        let r = completas(Proveedor::GitLab, true);
+        assert_eq!(
+            plan_de(
+                Proveedor::GitLab,
+                Some(&PlanObservado::GitLab(None)),
+                Some(&r)
+            ),
+            None
+        );
+        // El 403 de las reglas de aprobacion: Free.
+        let r = completas(Proveedor::GitLab, false);
+        assert_eq!(
+            plan_de(Proveedor::GitLab, None, Some(&r)),
+            Some(Plan::GitLab(TierGitLab::Free))
+        );
+        let (hs, c) = de_lo_leido(Some(Proveedor::GitHub), None, None);
+        assert_eq!(
+            hs.iter().map(Hallazgo::linea).collect::<Vec<_>>(),
+            ["garantia plan no-comprobado motivo=plan-no-comprobado"]
+        );
+        assert_eq!(c.plan, "no-comprobado");
+    }
+
+    /// Una garantia que el plan da y las puertas no tienen: `no-disponible
+    /// motivo=config arreglo=tc2-init`; sustituida y no disponible por plan,
+    /// con su sustituto y su arreglo (la linea de la seccion C de F1-13).
+    #[test]
+    fn config_sustituidas_y_no_disponibles() {
+        let mut r = completas(Proveedor::GitLab, true);
+        r.reglas_rama = vec!["push=30".to_owned()];
+        let gs = capacidades(Proveedor::GitLab, Plan::GitLab(TierGitLab::Free), Some(&r));
+        let l = |id: &str| {
+            gs.iter()
+                .find(|g| g.id == id)
+                .map(|g| hallazgo(g).linea())
+                .unwrap_or_default()
+        };
+        assert_eq!(
+            l("G02"),
+            "garantia G02 aviso motivo=config capacidad=no-disponible arreglo=tc2-init"
+        );
+        assert_eq!(
+            l("G01"),
+            "garantia G01 aviso motivo=plataforma capacidad=sustituida por=maintainer-solo-persona arreglo=ninguno-limite-de-la-plataforma"
+        );
+        assert_eq!(
+            l("G03"),
+            "garantia G03 aviso motivo=plan capacidad=sustituida por=merge-solo-persona arreglo=gitlab-premium"
+        );
+        assert_eq!(l("G13"), "garantia G13 ok capacidad=impuesta");
+        let gs = capacidades(
+            Proveedor::GitHub,
+            Plan::GitHub {
+                nombre: "github-free-privado-usuario",
+                rulesets: false,
+                cola: false,
+            },
+            None,
+        );
+        assert_eq!(
+            hallazgo(&gs[2]).linea(),
+            "garantia G03 aviso motivo=plan capacidad=no-disponible arreglo=repo-publico-o-github-pro"
+        );
+    }
+}
diff --git a/crates/tc2/src/doctor/entorno.rs b/crates/tc2/src/doctor/entorno.rs
new file mode 100644
index 0000000..8b250ed
--- /dev/null
+++ b/crates/tc2/src/doctor/entorno.rs
@@ -0,0 +1,160 @@
+//! Comprobaciones del entorno: `git` y su version (D3a: `aviso` por debajo de
+//! 2.56, `falla` solo sin git o con una version que no se entiende), el
+//! `tc2` que corre, estar en un repo y la configuracion de la base.
+
+use std::path::Path;
+
+use super::{ARREGLO_INIT, Estado, Hallazgo, NOMBRE};
+use crate::config;
+use crate::contrato::Informe;
+use crate::git::{ErrorGit, Git, SalidaGit, Subcomando, localizar_principal};
+
+/// Motivo sin `git`.
+pub(super) const SIN_GIT: &str = "sin-git";
+
+/// La version de git con la que se ha verificado todo (F1-1, adenda de
+/// F2-3; F1-4 «salida de git entre versiones»).
+pub const GIT_VERIFICADO: (u32, u32, u32) = (2, 56, 0);
+
+/// `(mayor, menor, parche)` de la salida de `git version` (`git version
+/// 2.56.0.windows.4`); `None` si no se entiende.
+pub fn version_de_git(salida: &str) -> Option<(u32, u32, u32)> {
+    let resto = salida.trim().strip_prefix("git version ")?;
+    let mut partes = resto.split(['.', ' ', '-']);
+    let mut num = || partes.next().and_then(|p| p.parse::<u32>().ok());
+    let mayor = num()?;
+    let menor = num()?;
+    Some((mayor, menor, num().unwrap_or(0)))
+}
+
+/// El hallazgo de `git` con lo que dio `git version` (funcion pura).
+pub fn hallazgo_git(r: &Result<SalidaGit, ErrorGit>) -> Hallazgo {
+    let h = Hallazgo::nuevo("entorno", "git", Estado::Ok);
+    let s = match r {
+        Err(_) => {
+            return Hallazgo {
+                estado: Estado::Falla,
+                ..h
+            }
+            .motivo(SIN_GIT)
+            .arreglo("instalar-git");
+        }
+        Ok(s) => s,
+    };
+    let version = s.exito().then(|| version_de_git(&s.stdout)).flatten();
+    match version {
+        None => Hallazgo {
+            estado: Estado::Falla,
+            ..h
+        }
+        .motivo("git-irreconocible")
+        .arreglo("instalar-git"),
+        Some(v) => {
+            let texto = format!("{}.{}.{}", v.0, v.1, v.2);
+            if v < GIT_VERIFICADO {
+                Hallazgo {
+                    estado: Estado::Aviso,
+                    ..h
+                }
+                .motivo("git-no-verificado")
+                .con("version", texto)
+                .arreglo("git-2.56-o-posterior")
+            } else {
+                h.con("version", texto)
+            }
+        }
+    }
+}
+
+/// `git version` con el puerto.
+pub(super) fn git(g: &Git, dir: &Path) -> Hallazgo {
+    hallazgo_git(&g.ejecutar(dir, Subcomando::Version, &[]))
+}
+
+/// El `tc2` que corre y su protocolo (siempre `ok`: es informativo).
+pub(super) fn tc2() -> Hallazgo {
+    Hallazgo::nuevo("entorno", "tc2", Estado::Ok)
+        .con("version", crate::VERSION)
+        .con("protocolo", crate::PROTOCOLO.to_string())
+}
+
+/// Fuera de un repo, `doctor rechazada motivo=no-es-repo` con el
+/// diagnostico de la capa git.
+pub(super) fn repo(g: &Git, dir: &Path) -> Result<(), Informe> {
+    localizar_principal(g, dir).map(|_| ()).map_err(|mut i| {
+        NOMBRE.clone_into(&mut i.comando);
+        i
+    })
+}
+
+/// La configuracion de la base (`origin/develop`, F1-10 decision 3).
+pub(super) fn config(g: &Git, dir: &Path) -> Hallazgo {
+    let h = |e: Estado| Hallazgo::nuevo("entorno", "config", e);
+    match config::de_base(g, dir) {
+        Ok(_) => h(Estado::Ok),
+        Err(i) => match i.valor("motivo") {
+            Some("sin-base") => h(Estado::Aviso)
+                .motivo("sin-base")
+                .arreglo("git fetch origin develop"),
+            Some("sin-config") => h(Estado::Aviso).motivo("sin-config").arreglo(ARREGLO_INIT),
+            Some("formato") => h(Estado::Falla)
+                .motivo("proyecto-016")
+                .arreglo(crate::init::ARREGLO_016),
+            Some(m) => h(Estado::Falla).motivo(m.to_owned()),
+            None => h(Estado::NoComprobado).motivo(i.resultado.texto()),
+        },
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn salida(codigo: i32, stdout: &str) -> Result<SalidaGit, ErrorGit> {
+        Ok(SalidaGit {
+            codigo,
+            stdout: stdout.to_owned(),
+            stderr: String::new(),
+        })
+    }
+
+    #[test]
+    fn version_de_git_se_lee() {
+        assert_eq!(
+            version_de_git("git version 2.56.0.windows.4\n"),
+            Some((2, 56, 0))
+        );
+        assert_eq!(version_de_git("git version 2.43.0"), Some((2, 43, 0)));
+        assert_eq!(version_de_git("git version 3.0"), Some((3, 0, 0)));
+        for malo in ["", "git version", "git version x.y", "hg version 2.56.0"] {
+            assert_eq!(version_de_git(malo), None, "{malo}");
+        }
+    }
+
+    /// C6 (parte pura): sin git o irreconocible, `falla`; por debajo de
+    /// 2.56, `aviso motivo=git-no-verificado`; 2.56 o mas, `ok`.
+    #[test]
+    fn git_falla_sin_binario_y_avisa_por_debajo_de_2_56() {
+        let l = |r: Result<SalidaGit, ErrorGit>| hallazgo_git(&r).linea();
+        assert_eq!(
+            l(Err(ErrorGit::Lanzar("no existe".into()))),
+            "entorno git falla motivo=sin-git arreglo=instalar-git"
+        );
+        assert_eq!(
+            l(salida(0, "otra cosa\n")),
+            "entorno git falla motivo=git-irreconocible arreglo=instalar-git"
+        );
+        assert_eq!(
+            l(salida(1, "git version 2.56.0\n")),
+            "entorno git falla motivo=git-irreconocible arreglo=instalar-git"
+        );
+        assert_eq!(
+            l(salida(0, "git version 2.55.9\n")),
+            "entorno git aviso motivo=git-no-verificado version=2.55.9 arreglo=git-2.56-o-posterior"
+        );
+        assert_eq!(
+            l(salida(0, "git version 2.56.0.windows.4\n")),
+            "entorno git ok version=2.56.0"
+        );
+    }
+}
diff --git a/crates/tc2/src/doctor/estado.rs b/crates/tc2/src/doctor/estado.rs
new file mode 100644
index 0000000..cd90aad
--- /dev/null
+++ b/crates/tc2/src/doctor/estado.rs
@@ -0,0 +1,135 @@
+//! Rama de estado e instalacion.
+//!
+//! - La rama de estado se lee como `board` (lo ultimo traido, sin `fetch`):
+//!   ausente, `falla motivo=sin-rama-de-estado`; una ref que existe y no es
+//!   un commit, `falla motivo=ref-ilegible ref=...`; un `tarea.md` que no se
+//!   entiende, `falla motivo=tarea-ilegible ids=...`.
+//! - La instalacion es el inventario de `tc2 init` en simulacion, sin
+//!   credenciales (no llama a la API: los rulesets salen `pendiente
+//!   motivo=sin-token` y los mira [`super::plataforma`]). Cada accion que
+//!   no sale `existe` es un hallazgo, asi `doctor` e `init` no discrepan.
+
+use std::path::Path;
+
+use super::{ARREGLO_INIT, Estado, Hallazgo, NOMBRE};
+use crate::contrato::corto;
+use crate::estado::{self, RAMA};
+use crate::git::estado::hay_remoto;
+use crate::git::{Git, REMOTO, Subcomando, correr, localizar_principal};
+use crate::init::{self, Accion, Decision};
+use crate::plataforma::{Credenciales, CredencialesFijas};
+
+/// La ref de la rama de estado que lee `board`: la remota si hay `origin`.
+fn referencia(g: &Git, dir: &Path) -> Option<String> {
+    let principal = localizar_principal(g, dir).ok()?;
+    let remoto = hay_remoto(g, &principal, NOMBRE).ok()?;
+    Some(if remoto {
+        format!("refs/remotes/{REMOTO}/{RAMA}")
+    } else {
+        format!("refs/heads/{RAMA}")
+    })
+}
+
+/// Si `r` existe (apunte o no a un commit).
+fn existe_ref(g: &Git, dir: &Path, r: &str) -> bool {
+    correr(
+        g,
+        dir,
+        NOMBRE,
+        Subcomando::RevParse,
+        &["-q", "--verify", "--end-of-options", r],
+    )
+    .is_ok_and(|s| s.exito())
+}
+
+/// Hallazgos de la rama de estado y el commit leido (12).
+pub(super) fn rama(g: &Git, dir: &Path) -> (Vec<Hallazgo>, Option<String>) {
+    let h = |e: Estado| Hallazgo::nuevo("estado", "rama", e);
+    let r = referencia(g, dir).unwrap_or_default();
+    let vista = match estado::leer(g, dir) {
+        Ok(v) => v,
+        Err(i) if i.valor("motivo") == Some("sin-rama-de-estado") => {
+            let x = if existe_ref(g, dir, &r) {
+                h(Estado::Falla).motivo("ref-ilegible").con("ref", r)
+            } else {
+                h(Estado::Falla)
+                    .motivo("sin-rama-de-estado")
+                    .arreglo(ARREGLO_INIT)
+            };
+            return (vec![x], None);
+        }
+        Err(i) => {
+            let m = i
+                .valor("motivo")
+                .map_or_else(|| "ref-ilegible".to_owned(), str::to_owned);
+            return (vec![h(Estado::Falla).motivo(m).con("ref", r)], None);
+        }
+    };
+    let commit = vista.commit().map(corto);
+    let mut v = vec![h(Estado::Ok).con("ref", r.clone())];
+    let t = |e: Estado| Hallazgo::nuevo("estado", "tareas", e);
+    v.push(match crate::situacion::calcular(&vista) {
+        Ok(s) if s.ilegibles.is_empty() => t(Estado::Ok),
+        Ok(s) => t(Estado::Falla)
+            .motivo("tarea-ilegible")
+            .con("ids", s.ilegibles.join(",")),
+        Err(_) => t(Estado::Falla).motivo("estado-ilegible").con("ref", r),
+    });
+    (v, commit)
+}
+
+/// Un hallazgo por accion del plan de `init` (sin los rulesets).
+pub(super) fn instalacion(g: &Git, dir: &Path) -> Vec<Hallazgo> {
+    let sin_token = || -> Box<dyn Credenciales> { Box::new(CredencialesFijas::nuevas()) };
+    let c = init::Contexto {
+        dir,
+        git: g,
+        credenciales: &sin_token,
+    };
+    match init::plan(&c) {
+        Ok(acciones) => acciones
+            .iter()
+            .filter(|a| a.clase != "ruleset")
+            .map(de_accion)
+            .collect(),
+        // Sin red (o denegada) el inventario no se puede hacer entero: no se
+        // dice nada de la instalacion que no se haya leido.
+        Err(i) => vec![
+            Hallazgo::nuevo("instalacion", "plan", Estado::NoComprobado).motivo(
+                match i.valor("motivo") {
+                    Some(m) => m.to_owned(),
+                    None => i.resultado.texto().to_owned(),
+                },
+            ),
+        ],
+    }
+}
+
+/// El hallazgo de una accion del plan de `init` (funcion pura).
+pub fn de_accion(a: &Accion) -> Hallazgo {
+    let h = |e: Estado| Hallazgo::nuevo("instalacion", a.clase, e).con("ruta", a.ruta.clone());
+    let motivo = a.motivo.clone().unwrap_or_default();
+    match a.decision {
+        Decision::Existe => h(Estado::Ok),
+        Decision::Crea => h(Estado::Aviso).motivo("falta").arreglo(ARREGLO_INIT),
+        // Sin token `init` no lee la persona: lo que falta lo crea `init`.
+        Decision::Pendiente if motivo == "sin-token" => {
+            h(Estado::Aviso).motivo("falta").arreglo(ARREGLO_INIT)
+        }
+        Decision::Pendiente => h(Estado::Aviso).motivo(motivo),
+        Decision::Persona => {
+            let x = h(Estado::Aviso).motivo(motivo);
+            match &a.orden {
+                Some(o) => x.arreglo(o.clone()),
+                None => x,
+            }
+        }
+        Decision::Rechaza => {
+            let x = h(Estado::Falla).motivo(motivo);
+            match &a.orden {
+                Some(o) => x.arreglo(o.clone()),
+                None => x,
+            }
+        }
+    }
+}
diff --git a/crates/tc2/src/doctor/mod.rs b/crates/tc2/src/doctor/mod.rs
new file mode 100644
index 0000000..10155db
--- /dev/null
+++ b/crates/tc2/src/doctor/mod.rs
@@ -0,0 +1,453 @@
+//! `tc2 doctor` (F3-8): que le falta al entorno, a la rama de estado, a la
+//! instalacion y a la plataforma para que TaskCodeTwo funcione, con el
+//! arreglo de cada fallo (RF-64).
+//!
+//! Solo lee, como `board`: ni `fetch` ni escrituras en el repo, en
+//! `<git-common-dir>/taskcodetwo/` ni en la plataforma (solo `GET`). Cada
+//! comprobacion deja un [`Hallazgo`] (`ok`, `aviso`, `falla` o
+//! `no-comprobado`); el agregador ([`informe`]) es puro.
+//!
+//! Salida (F1-4 decision 4, F1-13 seccion C): como mucho 5 lineas en
+//! stdout. La primera, `doctor <ok|rechazada> fallos=N avisos=N
+//! no-comprobado=N plataforma=... [plan=... impuestas=N sustituidas=N
+//! no-disponibles=N] [commit=...]`; detras, los hallazgos que no son `ok`,
+//! los peores primero, y si no caben, `y N mas: tc2 doctor --json`. Con
+//! `--json`, una linea con todos los hallazgos (tambien los `ok`). Codigo 3
+//! con alguna `falla` (D5a); 0 con avisos o no-comprobados.
+
+pub mod capacidades;
+mod entorno;
+mod estado;
+pub mod plataforma;
+
+use std::path::Path;
+
+use crate::Salida;
+use crate::contrato::{Informe, Resultado, cadena_json, escapar_valor};
+use crate::git::Git;
+use crate::i18n::catalogo::doctor as txt;
+use crate::i18n::{Idioma, Mensaje};
+use crate::identidad::Conectar;
+
+pub use entorno::{hallazgo_git, version_de_git};
+
+/// Nombre de la orden.
+pub const NOMBRE: &str = "doctor";
+
+/// Lineas como mucho en stdout (F1-4, decision 4).
+pub const MAX_LINEAS: usize = 5;
+
+/// Arreglo de lo que deja `tc2 init`.
+pub const ARREGLO_INIT: &str = "tc2-init";
+
+/// Estado de una comprobacion.
+#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
+pub enum Estado {
+    /// Falla: el codigo es 3.
+    Falla,
+    /// Funciona, pero algo esta degradado o falta.
+    Aviso,
+    /// No se pudo comprobar (sin red, sin token...): nunca se da por buena.
+    NoComprobado,
+    /// Bien.
+    Ok,
+}
+
+impl Estado {
+    /// Valor en la linea.
+    pub fn texto(self) -> &'static str {
+        match self {
+            Self::Falla => "falla",
+            Self::Aviso => "aviso",
+            Self::NoComprobado => "no-comprobado",
+            Self::Ok => "ok",
+        }
+    }
+}
+
+/// Lo que se encontro en una comprobacion.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Hallazgo {
+    /// Zona: `entorno`, `estado`, `instalacion`, `proteccion`, `permisos`,
+    /// `check`, `garantia`...
+    pub area: &'static str,
+    /// Que se comprobo dentro de la zona (`git`, `develop`, `G03`...).
+    pub id: String,
+    /// Como salio.
+    pub estado: Estado,
+    /// Por que (sin espacios).
+    pub motivo: Option<String>,
+    /// Claves de mas (`permiso=`, `ids=`...), en orden.
+    pub extra: Vec<(&'static str, String)>,
+    /// Que hacer (sin espacios).
+    pub arreglo: Option<String>,
+}
+
+impl Hallazgo {
+    /// Hallazgo sin motivo ni arreglo.
+    pub fn nuevo(area: &'static str, id: impl Into<String>, estado: Estado) -> Self {
+        Self {
+            area,
+            id: id.into(),
+            estado,
+            motivo: None,
+            extra: Vec::new(),
+            arreglo: None,
+        }
+    }
+
+    /// Con motivo.
+    #[must_use]
+    pub fn motivo(mut self, m: impl Into<String>) -> Self {
+        self.motivo = Some(m.into());
+        self
+    }
+
+    /// Con una clave de mas.
+    #[must_use]
+    pub fn con(mut self, k: &'static str, v: impl Into<String>) -> Self {
+        self.extra.push((k, v.into()));
+        self
+    }
+
+    /// Con arreglo.
+    #[must_use]
+    pub fn arreglo(mut self, a: impl Into<String>) -> Self {
+        self.arreglo = Some(a.into());
+        self
+    }
+
+    /// Valor de una clave de mas.
+    pub fn valor(&self, k: &str) -> Option<&str> {
+        self.extra
+            .iter()
+            .find(|(c, _)| *c == k)
+            .map(|(_, v)| v.as_str())
+    }
+
+    /// `<area> <id> <estado> [motivo=...] [claves...] [arreglo=...]`, con los
+    /// valores escapados como el contrato (sin espacios).
+    pub fn linea(&self) -> String {
+        let mut s = format!(
+            "{} {} {}",
+            self.area,
+            escapar_valor(&self.id, false),
+            self.estado.texto()
+        );
+        let claves = self
+            .motivo
+            .iter()
+            .map(|m| ("motivo", m.as_str()))
+            .chain(self.extra.iter().map(|(k, v)| (*k, v.as_str())))
+            .chain(self.arreglo.iter().map(|a| ("arreglo", a.as_str())));
+        for (k, v) in claves {
+            s.push_str(&format!(" {k}={}", escapar_valor(v, false)));
+        }
+        s
+    }
+
+    /// El hallazgo como objeto JSON.
+    pub fn json(&self) -> String {
+        let mut s = format!(
+            "{{\"area\":{},\"id\":{},\"estado\":{}",
+            cadena_json(self.area),
+            cadena_json(&self.id),
+            cadena_json(self.estado.texto())
+        );
+        if let Some(m) = &self.motivo {
+            s.push_str(&format!(",\"motivo\":{}", cadena_json(m)));
+        }
+        for (k, v) in &self.extra {
+            s.push_str(&format!(",{}:{}", cadena_json(k), cadena_json(v)));
+        }
+        if let Some(a) = &self.arreglo {
+            s.push_str(&format!(",\"arreglo\":{}", cadena_json(a)));
+        }
+        s.push('}');
+        s
+    }
+}
+
+/// Lo que va en la primera linea ademas de las cuentas.
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct Resumen {
+    /// Valor de `plataforma=`: `ok`, `no-configurada`, `sin-credenciales`,
+    /// `sin-red` o `error`.
+    pub plataforma: String,
+    /// Plan observado y cuentas de `capacidades` (F1-13 decision 3): `None`
+    /// si no se llego a la plataforma.
+    pub capacidades: Option<capacidades::Cuentas>,
+    /// Commit de la rama de estado (12), si se leyo.
+    pub commit: Option<String>,
+}
+
+/// Cuantos hallazgos hay en `e`.
+fn cuantos(hs: &[Hallazgo], e: Estado) -> usize {
+    hs.iter().filter(|h| h.estado == e).count()
+}
+
+/// La linea de contrato de `doctor` (funcion pura): `rechazada` con alguna
+/// `falla` (D5a), `ok` si no. El diagnostico nombra el primer fallo.
+pub fn informe(hs: &[Hallazgo], r: &Resumen) -> Informe {
+    let fallos = cuantos(hs, Estado::Falla);
+    let resultado = if fallos > 0 {
+        Resultado::Rechazada
+    } else {
+        Resultado::Ok
+    };
+    let mut i = Informe::nuevo(NOMBRE, resultado)
+        .con("fallos", fallos.to_string())
+        .con("avisos", cuantos(hs, Estado::Aviso).to_string())
+        .con(
+            "no-comprobado",
+            cuantos(hs, Estado::NoComprobado).to_string(),
+        )
+        .con("plataforma", r.plataforma.clone());
+    if let Some(c) = &r.capacidades {
+        i = i.con("plan", c.plan.clone());
+        if let Some((imp, sus, nod)) = c.cuentas {
+            i = i
+                .con("impuestas", imp.to_string())
+                .con("sustituidas", sus.to_string())
+                .con("no-disponibles", nod.to_string());
+        }
+    }
+    if let Some(c) = &r.commit {
+        i = i.con("commit", c.clone());
+    }
+    if let Some(primero) = ordenados(hs)
+        .into_iter()
+        .find(|h| h.estado == Estado::Falla)
+    {
+        i = i.diagnostico(
+            Mensaje::nuevo(&txt::CON_FALLOS)
+                .con("n", fallos.to_string())
+                .con("primero", primero.linea()),
+        );
+    }
+    i
+}
+
+/// Los hallazgos que no son `ok`, los peores primero (estable: dentro de
+/// cada estado, en el orden en que se comprobaron).
+pub fn ordenados(hs: &[Hallazgo]) -> Vec<&Hallazgo> {
+    let mut v: Vec<&Hallazgo> = hs.iter().filter(|h| h.estado != Estado::Ok).collect();
+    v.sort_by_key(|h| h.estado);
+    v
+}
+
+/// Las lineas de stdout sin `--json` (funcion pura): la primera y, detras,
+/// los hallazgos que caben en [`MAX_LINEAS`]; si no caben todos, los
+/// primeros y `y N mas: tc2 doctor --json`.
+pub fn lineas(hs: &[Hallazgo], r: &Resumen) -> Vec<String> {
+    let mut v = vec![informe(hs, r).linea()];
+    let malos = ordenados(hs);
+    let hueco = MAX_LINEAS - 1;
+    if malos.len() <= hueco {
+        v.extend(malos.iter().map(|h| h.linea()));
+    } else {
+        v.extend(malos.iter().take(hueco - 1).map(|h| h.linea()));
+        v.push(format!(
+            "y {} mas: tc2 doctor --json",
+            malos.len() - (hueco - 1)
+        ));
+    }
+    v
+}
+
+/// La salida del proceso: texto o JSON (una linea con todos los
+/// hallazgos), diagnostico en stderr solo con alguna `falla`.
+pub fn salida(hs: &[Hallazgo], r: &Resumen, json: bool, idioma: Idioma) -> Salida {
+    let i = informe(hs, r);
+    let mut s = i.salida_en(json, idioma);
+    if json {
+        let todos: Vec<String> = hs.iter().map(Hallazgo::json).collect();
+        let linea = s.stdout.trim_end();
+        if let Some(sin_cierre) = linea.strip_suffix('}') {
+            s.stdout = format!("{sin_cierre},\"hallazgos\":[{}]}}\n", todos.join(","));
+        }
+    } else {
+        s.stdout = lineas(hs, r).join("\n") + "\n";
+    }
+    s
+}
+
+/// Lo que necesita `doctor` del entorno.
+pub struct Entorno<'a> {
+    /// Carpeta desde la que se lanza.
+    pub dir: &'a Path,
+    /// Puerto Git.
+    pub git: &'a Git,
+    /// Construye el adaptador de la plataforma.
+    pub conectar: &'a Conectar<'a>,
+}
+
+/// Todas las comprobaciones. `Err` solo si no se puede ni empezar (fuera de
+/// un repo): `rechazada` con el diagnostico.
+pub fn comprobar(e: &Entorno<'_>) -> Result<(Vec<Hallazgo>, Resumen), Informe> {
+    let mut hs = vec![entorno::tc2()];
+    let git = entorno::git(e.git, e.dir);
+    let hay_git = !matches!(git.motivo.as_deref(), Some(entorno::SIN_GIT));
+    hs.push(git);
+    if !hay_git {
+        // Sin git no se puede comprobar nada mas: se dice, sin suponer.
+        for (area, id) in [
+            ("entorno", "config"),
+            ("estado", "rama"),
+            ("instalacion", "plan"),
+            ("plataforma", "conexion"),
+        ] {
+            hs.push(Hallazgo::nuevo(area, id, Estado::NoComprobado).motivo(entorno::SIN_GIT));
+        }
+        return Ok((
+            hs,
+            Resumen {
+                plataforma: "no-configurada".to_owned(),
+                ..Resumen::default()
+            },
+        ));
+    }
+    entorno::repo(e.git, e.dir)?;
+    hs.push(entorno::config(e.git, e.dir));
+    let (h, commit) = estado::rama(e.git, e.dir);
+    hs.extend(h);
+    hs.extend(estado::instalacion(e.git, e.dir));
+    let (h, plataforma, capacidades) = plataforma::comprobar(e);
+    hs.extend(h);
+    Ok((
+        hs,
+        Resumen {
+            plataforma,
+            capacidades,
+            commit,
+        },
+    ))
+}
+
+/// Lee las opciones (sin `--json`): ninguna.
+fn leer_opciones(args: &[String]) -> Result<(), Informe> {
+    if args.is_empty() {
+        Ok(())
+    } else {
+        Err(Informe::nuevo(NOMBRE, Resultado::Uso)
+            .con("motivo", "argumentos")
+            .diagnostico(&txt::USO_DOCTOR))
+    }
+}
+
+/// `doctor [--json]` (sin `--json`, que quita `lib.rs`).
+pub fn orden(args: &[String], json: bool, idioma: Idioma, e: &Entorno<'_>) -> Salida {
+    if let Err(i) = leer_opciones(args) {
+        return i.salida_en(json, idioma);
+    }
+    match comprobar(e) {
+        Ok((hs, r)) => salida(&hs, &r, json, idioma),
+        Err(i) => i.salida_en(json, idioma),
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn h(area: &'static str, id: &str, e: Estado) -> Hallazgo {
+        Hallazgo::nuevo(area, id, e).motivo("m")
+    }
+
+    fn resumen() -> Resumen {
+        Resumen {
+            plataforma: "ok".to_owned(),
+            capacidades: None,
+            commit: Some("0123456789ab".to_owned()),
+        }
+    }
+
+    /// C7: la primera linea fija; detras, los peores primero; con mas de 4,
+    /// 3 y `y N mas`; nunca mas de 5 lineas; JSON con todos y sin ANSI.
+    #[test]
+    fn cinco_lineas_como_mucho_y_los_peores_primero() {
+        let hs = vec![
+            h("entorno", "tc2", Estado::Ok),
+            h("instalacion", "workflow", Estado::Aviso),
+            h("plataforma", "conexion", Estado::NoComprobado),
+            h("proteccion", "develop", Estado::Falla),
+            h("check", "carril", Estado::Aviso),
+            h("check", "verificacion", Estado::Falla),
+        ];
+        let l = lineas(&hs, &resumen());
+        assert_eq!(
+            l,
+            [
+                "doctor rechazada fallos=2 avisos=2 no-comprobado=1 plataforma=ok commit=0123456789ab",
+                "proteccion develop falla motivo=m",
+                "check verificacion falla motivo=m",
+                "instalacion workflow aviso motivo=m",
+                "y 2 mas: tc2 doctor --json",
+            ]
+        );
+        // Con 4 que no son ok, caben todos.
+        let l = lineas(&hs[..5], &resumen());
+        assert_eq!(l.len(), 5);
+        assert_eq!(l[4], "plataforma conexion no-comprobado motivo=m");
+        // Sin nada malo, solo la primera.
+        assert_eq!(lineas(&hs[..1], &resumen()).len(), 1);
+        let s = salida(&hs, &resumen(), false, Idioma::Es);
+        assert_eq!(s.codigo, 3);
+        assert_eq!(s.stdout.lines().count(), MAX_LINEAS);
+        assert_eq!(s.stderr.lines().count(), 1);
+        assert!(
+            s.stderr.contains("proteccion develop falla"),
+            "{}",
+            s.stderr
+        );
+        let j = salida(&hs, &resumen(), true, Idioma::En);
+        assert_eq!(j.codigo, 3);
+        assert_eq!(j.stdout.lines().count(), 1);
+        assert!(!j.stdout.contains('\u{1b}'));
+        let v: serde_json::Value = serde_json::from_str(&j.stdout).unwrap_or_default();
+        assert_eq!(v["hallazgos"].as_array().map(Vec::len), Some(6));
+        assert_eq!(v["claves"]["fallos"], "2");
+        assert_eq!(v["hallazgos"][0]["estado"], "ok");
+    }
+
+    /// C8: 0 sin fallos aunque haya avisos y no-comprobados; 3 con uno.
+    #[test]
+    fn codigo_cero_sin_fallos_y_tres_con_uno() {
+        let mut hs = vec![
+            h("instalacion", "workflow", Estado::Aviso),
+            h("plataforma", "conexion", Estado::NoComprobado),
+        ];
+        let s = salida(&hs, &resumen(), false, Idioma::Es);
+        assert_eq!((s.codigo, s.stderr.as_str()), (0, ""));
+        assert!(
+            s.stdout
+                .starts_with("doctor ok fallos=0 avisos=1 no-comprobado=1")
+        );
+        hs.push(h("estado", "rama", Estado::Falla));
+        assert_eq!(salida(&hs, &resumen(), true, Idioma::Es).codigo, 3);
+    }
+
+    #[test]
+    fn la_linea_escapa_los_valores() {
+        let x = Hallazgo::nuevo("instalacion", "rama", Estado::Aviso)
+            .motivo("sin-publicar")
+            .con("ruta", "refs/heads/main")
+            .arreglo("git push origin main");
+        assert_eq!(
+            x.linea(),
+            "instalacion rama aviso motivo=sin-publicar ruta=refs/heads/main arreglo=git%20push%20origin%20main"
+        );
+    }
+
+    #[test]
+    fn argumentos_de_mas_son_uso() {
+        assert!(leer_opciones(&[]).is_ok());
+        for a in ["--recursos", "x"] {
+            let i = leer_opciones(&[a.to_owned()]).err();
+            assert_eq!(
+                i.map(|i| i.linea()).as_deref(),
+                Some("doctor uso motivo=argumentos")
+            );
+        }
+    }
+}
diff --git a/crates/tc2/src/doctor/plataforma.rs b/crates/tc2/src/doctor/plataforma.rs
new file mode 100644
index 0000000..f4fbc01
--- /dev/null
+++ b/crates/tc2/src/doctor/plataforma.rs
@@ -0,0 +1,296 @@
+//! Lo que se lee de la plataforma: la proteccion de las ramas del modelo con
+//! `solo_pr` (`develop` y `main`) comparada con
+//! [`crate::plataforma::puertas::modelo`], y los permisos (la lectura de las
+//! puertas y los de la CI). Solo lecturas del puerto (`GET`).
+//!
+//! Sin plataforma, sin credenciales o sin red, todos los hallazgos de aqui
+//! salen `no-comprobado`: nunca se da por buena una proteccion sin leerla.
+
+use super::{ARREGLO_INIT, Entorno, Estado, Hallazgo};
+use crate::contrato::{Informe, Resultado};
+use crate::identidad::Consulta;
+use crate::plataforma::puertas::{Objetivo, modelo};
+use crate::plataforma::{Paso, Plataforma, Proveedor, ReglasLeidas, github, gitlab};
+
+/// Lo que se comprueba aqui, para decir `no-comprobado` de todo a la vez.
+const COMPROBACIONES: [(&str, &str); 4] = [
+    ("proteccion", "develop"),
+    ("proteccion", "main"),
+    ("permisos", "ci"),
+    ("garantia", "plan"),
+];
+
+/// Las ramas del modelo que solo admiten PR (F1-5 decision 2).
+pub fn ramas_solo_pr() -> Vec<(&'static str, String)> {
+    modelo()
+        .into_iter()
+        .filter(|p| p.solo_pr.is_some())
+        .filter_map(|p| match &p.objetivo {
+            Objetivo::Ramas(r) if r.len() == 1 => r[0]
+                .strip_prefix("refs/heads/")
+                .map(|x| (p.nombre, x.to_owned())),
+            _ => None,
+        })
+        .collect()
+}
+
+/// Si la rama solo admite cambios por PR o MR, con lo leido (G02): en
+/// GitHub, la regla `pull_request`; en GitLab, protegida y con «Allowed to
+/// push» a nadie (`push=0` y ningun otro nivel).
+pub fn solo_pr(p: Proveedor, r: &ReglasLeidas) -> bool {
+    match p {
+        Proveedor::GitHub => r.reglas_rama.iter().any(|x| x == "pull_request"),
+        Proveedor::GitLab => {
+            let push: Vec<&String> = r
+                .reglas_rama
+                .iter()
+                .filter(|x| x.starts_with("push="))
+                .collect();
+            !push.is_empty() && push.iter().all(|x| *x == "push=0")
+        }
+    }
+}
+
+/// El permiso de una lectura, para `permiso=`.
+fn permiso(p: Option<Proveedor>, op: &str) -> &'static str {
+    match (p, op) {
+        (Some(Proveedor::GitHub), "leer-permisos-ci") => github::PERMISO_PERMISOS_CI,
+        (Some(Proveedor::GitHub), _) => github::PERMISO_LEER_PUERTAS,
+        _ => gitlab::PERMISO_LECTURA_CORTO,
+    }
+}
+
+/// `no-comprobado` de todo lo de la plataforma, con `motivo`.
+fn sin_plataforma(motivo: &str) -> Vec<Hallazgo> {
+    let mut v: Vec<Hallazgo> = COMPROBACIONES
+        .iter()
+        .map(|(a, id)| Hallazgo::nuevo(a, *id, Estado::NoComprobado).motivo(motivo))
+        .collect();
+    v.extend(checks_del_modelo().into_iter().map(|c| {
+        Hallazgo::nuevo("check", c, Estado::NoComprobado)
+            .motivo(motivo)
+            .con("rama", RAMA_CHECKS)
+    }));
+    v
+}
+
+/// La rama cuyos checks se comprueban (C5): `develop`, la de integracion.
+pub const RAMA_CHECKS: &str = "develop";
+
+/// Los checks que el modelo exige en [`RAMA_CHECKS`].
+pub fn checks_del_modelo() -> Vec<&'static str> {
+    modelo()
+        .into_iter()
+        .find(|p| p.nombre == RAMA_CHECKS)
+        .map(|p| p.checks)
+        .unwrap_or_default()
+}
+
+/// Hallazgos de los checks de `develop` (funcion pura, C5 y D4a): un check
+/// del modelo que la rama no exige, `falla motivo=check-no-obligatorio`;
+/// uno exigido que no esta entre los publicados en su cabeza, `falla
+/// motivo=check-sin-publicar`, o `aviso` si no hay ninguno publicado (repo
+/// recien creado, sin el workflow de F5-1). Solo se mira el nombre: el
+/// origen (`integration_id`) espera a F6-1.
+pub fn checks(p: Option<Proveedor>, leidas: &Paso<ReglasLeidas>) -> Vec<Hallazgo> {
+    let modelo = checks_del_modelo();
+    let h = |c: &str, e: Estado| Hallazgo::nuevo("check", c, e).con("rama", RAMA_CHECKS);
+    let todos_igual = |e: Estado, motivo: &str| -> Vec<Hallazgo> {
+        modelo
+            .iter()
+            .map(|c| {
+                let mut x = h(c, e).motivo(motivo);
+                if motivo == "plan" {
+                    x = x.arreglo("repo-publico-o-github-pro");
+                }
+                x
+            })
+            .collect()
+    };
+    let r = match leidas {
+        Ok(r) => r,
+        Err(i) if es_de_permisos(i) => {
+            return todos_igual(Estado::NoComprobado, "permiso-insuficiente");
+        }
+        Err(i) => {
+            return todos_igual(
+                Estado::NoComprobado,
+                i.valor("motivo").unwrap_or_else(|| i.resultado.texto()),
+            );
+        }
+    };
+    if p == Some(Proveedor::GitHub) && !r.plan_da_reglas {
+        return todos_igual(Estado::Aviso, "plan");
+    }
+    let Some(requeridos) = &r.checks_requeridos else {
+        return todos_igual(Estado::NoComprobado, "checks-no-leidos");
+    };
+    let mut nombres: Vec<&str> = modelo.clone();
+    nombres.extend(
+        requeridos
+            .iter()
+            .map(String::as_str)
+            .filter(|c| !modelo.contains(c)),
+    );
+    nombres
+        .into_iter()
+        .map(|c| {
+            let exigido = r.todos_los_checks || requeridos.iter().any(|x| x == c);
+            if !exigido {
+                return h(c, Estado::Falla)
+                    .motivo("check-no-obligatorio")
+                    .arreglo(ARREGLO_INIT);
+            }
+            match &r.checks_publicados {
+                None => h(c, Estado::NoComprobado).motivo("checks-no-leidos"),
+                Some(v) if v.iter().any(|x| x == c) => h(c, Estado::Ok),
+                Some(v) if v.is_empty() => h(c, Estado::Aviso).motivo("check-sin-publicar"),
+                Some(_) => h(c, Estado::Falla).motivo("check-sin-publicar"),
+            }
+        })
+        .collect()
+}
+
+/// Si un fallo es un 403 de permisos (`denegada motivo=permisos`).
+fn es_de_permisos(i: &Informe) -> bool {
+    i.resultado == Resultado::Denegada && i.valor("motivo") == Some("permisos")
+}
+
+/// El hallazgo de la proteccion de `rama` (funcion pura).
+pub fn proteccion(
+    p: Option<Proveedor>,
+    nombre: &str,
+    rama: &str,
+    leidas: &Paso<ReglasLeidas>,
+) -> Hallazgo {
+    let h = |e: Estado| Hallazgo::nuevo("proteccion", rama, e);
+    let r = match leidas {
+        Ok(r) => r,
+        Err(i) if es_de_permisos(i) => {
+            return h(Estado::NoComprobado).motivo("permiso-insuficiente");
+        }
+        Err(i) => {
+            return h(Estado::NoComprobado).motivo(
+                i.valor("motivo")
+                    .unwrap_or_else(|| i.resultado.texto())
+                    .to_owned(),
+            );
+        }
+    };
+    let Some(p) = p else {
+        return h(Estado::NoComprobado).motivo("plataforma-desconocida");
+    };
+    if p == Proveedor::GitHub && !r.plan_da_reglas {
+        // D2a de F1-5: el plan no da rulesets; lo dice `capacidades`.
+        return h(Estado::Aviso)
+            .motivo("plan")
+            .arreglo("repo-publico-o-github-pro");
+    }
+    let ruleset = format!("{}{nombre}", crate::huella::PREFIJO_RULESETS);
+    let falta_ruleset = p == Proveedor::GitHub && !r.rulesets.contains(&ruleset);
+    if falta_ruleset || !solo_pr(p, r) {
+        return h(Estado::Falla)
+            .motivo("proteccion-ausente")
+            .arreglo(ARREGLO_INIT);
+    }
+    h(Estado::Ok)
+}
+
+/// El hallazgo de los permisos de la CI (funcion pura).
+pub fn permisos_ci(p: Option<Proveedor>, i: &Informe) -> Hallazgo {
+    let h = |e: Estado| Hallazgo::nuevo("permisos", "ci", e);
+    let motivo = i.valor("motivo");
+    match (i.resultado, motivo) {
+        (Resultado::Ok, _) => h(Estado::Ok),
+        (Resultado::Rechazada, Some("ci-puede-aprobar")) => h(Estado::Falla)
+            .motivo("ci-puede-aprobar")
+            .arreglo("desactivar-aprobacion-por-actions"),
+        (Resultado::Rechazada, Some("identidad")) if i.valor("falta").is_some() => {
+            h(Estado::NoComprobado).motivo("sin-token-persona")
+        }
+        (Resultado::Rechazada, Some("identidad")) => h(Estado::Falla)
+            .motivo("persona-fuera-de-lista")
+            .arreglo(crate::plataforma::personas::RUTA),
+        _ if es_de_permisos(i) => h(Estado::Falla)
+            .motivo("permiso-insuficiente")
+            .con("permiso", permiso(p, "leer-permisos-ci"))
+            .con("rol", "persona"),
+        (Resultado::Sinred, _) => h(Estado::NoComprobado).motivo("sin-red"),
+        (Resultado::Denegada, Some("credenciales")) => h(Estado::Falla)
+            .motivo("credenciales")
+            .con("rol", "persona"),
+        (r, m) => h(Estado::Falla).motivo(m.unwrap_or_else(|| r.texto()).to_owned()),
+    }
+}
+
+/// Todo lo de la plataforma: hallazgos, `plataforma=` y las cuentas de
+/// `capacidades`.
+pub(super) fn comprobar(
+    e: &Entorno<'_>,
+) -> (Vec<Hallazgo>, String, Option<super::capacidades::Cuentas>) {
+    let (personas, _) = crate::roadmap::personas_y_aviso(e.git, e.dir);
+    let c = match (e.conectar)(&personas) {
+        Ok(c) => c,
+        Err(consulta) => {
+            let n = consulta.nombre();
+            return (sin_plataforma(n), n.to_owned(), None);
+        }
+    };
+    let p: &dyn Plataforma = c.plataforma.as_ref();
+    let prov = p.proveedor();
+    let ramas = ramas_solo_pr();
+    let mut leidas: Vec<Paso<ReglasLeidas>> = Vec::new();
+    for (_, rama) in &ramas {
+        let r = p.leer_puertas(c.rol, rama);
+        // Sin red o sin credenciales en la primera lectura no se sigue: lo
+        // de la plataforma queda `no-comprobado`.
+        if let Err(i) = &r
+            && leidas.is_empty()
+            && matches!(
+                Consulta::de_fallo(i),
+                Consulta::SinRed | Consulta::SinCredenciales
+            )
+        {
+            let n = Consulta::de_fallo(i).nombre();
+            return (sin_plataforma(n), n.to_owned(), None);
+        }
+        leidas.push(r);
+    }
+    let mut hs = Vec::new();
+    let mut plataforma = "ok";
+    for ((nombre, rama), r) in ramas.iter().zip(&leidas) {
+        hs.push(proteccion(prov, nombre, rama, r));
+    }
+    if let Some(r) = ramas
+        .iter()
+        .zip(&leidas)
+        .find(|((_, rama), _)| rama == RAMA_CHECKS)
+        .map(|(_, r)| r)
+    {
+        hs.extend(checks(prov, r));
+    }
+    // Un 403 al leer las puertas: una sola falla con su permiso.
+    if let Some(i) = leidas.iter().filter_map(|r| r.as_ref().err()).next() {
+        if es_de_permisos(i) {
+            hs.push(
+                Hallazgo::nuevo("permisos", "leer-puertas", Estado::Falla)
+                    .motivo("permiso-insuficiente")
+                    .con("permiso", permiso(prov, "leer-puertas"))
+                    .con("rol", c.rol.nombre()),
+            );
+        } else {
+            plataforma = Consulta::de_fallo(i).nombre();
+        }
+    }
+    hs.push(permisos_ci(prov, &p.leer_permisos_ci()));
+    // `capacidades` con el plan observado y las puertas de `develop`.
+    let observado = p.leer_plan(c.rol).ok();
+    let develop = ramas
+        .iter()
+        .zip(&leidas)
+        .find(|((_, rama), _)| rama == RAMA_CHECKS)
+        .and_then(|(_, r)| r.as_ref().ok());
+    let (h, cuentas) = super::capacidades::de_lo_leido(prov, observado.as_ref(), develop);
+    hs.extend(h);
+    (hs, plataforma.to_owned(), Some(cuentas))
+}
diff --git a/crates/tc2/src/git/puerto.rs b/crates/tc2/src/git/puerto.rs
index 3cdf400..69a93c8 100644
--- a/crates/tc2/src/git/puerto.rs
+++ b/crates/tc2/src/git/puerto.rs
@@ -66,6 +66,8 @@ pub enum Subcomando {
     UpdateIndex,
     /// `git update-ref` (solo bajo `refs/taskcodetwo/`)
     UpdateRef,
+    /// `git version` (lectura; `tc2 doctor`, F3-8)
+    Version,
     /// `git worktree`
     Worktree,
     /// `git write-tree` (solo con indice temporal)
@@ -99,6 +101,7 @@ impl Subcomando {
             Self::Tag => "tag",
             Self::UpdateIndex => "update-index",
             Self::UpdateRef => "update-ref",
+            Self::Version => "version",
             Self::Worktree => "worktree",
             Self::WriteTree => "write-tree",
         }
diff --git a/crates/tc2/src/i18n/catalogo/cli.rs b/crates/tc2/src/i18n/catalogo/cli.rs
index 7e83773..009b406 100644
--- a/crates/tc2/src/i18n/catalogo/cli.rs
+++ b/crates/tc2/src/i18n/catalogo/cli.rs
@@ -10,8 +10,8 @@ crate::i18n::textos! {
         en: "tc2: could not write the output: {error}",
     }
     USO {
-        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<motivo>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
-        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<reason>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
+        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<motivo>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
+        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<reason>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
     }
     FALTA_LA_ORDEN {
         es: "tc2: falta la orden. {uso}",
diff --git a/crates/tc2/src/i18n/catalogo/doctor.rs b/crates/tc2/src/i18n/catalogo/doctor.rs
new file mode 100644
index 0000000..ea75626
--- /dev/null
+++ b/crates/tc2/src/i18n/catalogo/doctor.rs
@@ -0,0 +1,12 @@
+//! Mensajes de `tc2 doctor` (F3-8).
+
+crate::i18n::textos! {
+    USO_DOCTOR {
+        es: "uso: doctor [--json] (doctor --recursos llega con F3-9)",
+        en: "usage: doctor [--json] (doctor --recursos comes with F3-9)",
+    }
+    CON_FALLOS {
+        es: "doctor encontro {n} fallo(s); el primero: {primero}. Detalle: tc2 doctor --json",
+        en: "doctor found {n} failure(s); the first one: {primero}. Details: tc2 doctor --json",
+    }
+}
diff --git a/crates/tc2/src/i18n/catalogo/mod.rs b/crates/tc2/src/i18n/catalogo/mod.rs
index a98a1bd..c716f93 100644
--- a/crates/tc2/src/i18n/catalogo/mod.rs
+++ b/crates/tc2/src/i18n/catalogo/mod.rs
@@ -7,6 +7,7 @@ pub mod cli;
 pub mod comun;
 pub mod config;
 pub mod desinstalar;
+pub mod doctor;
 pub mod estado;
 pub mod git;
 pub mod identidad;
@@ -29,6 +30,7 @@ pub fn todos() -> impl Iterator<Item = &'static Texto> {
         comun::TODOS,
         config::TODOS,
         desinstalar::TODOS,
+        doctor::TODOS,
         estado::TODOS,
         git::TODOS,
         identidad::TODOS,
diff --git a/crates/tc2/src/identidad.rs b/crates/tc2/src/identidad.rs
index 56c8c97..27f8877 100644
--- a/crates/tc2/src/identidad.rs
+++ b/crates/tc2/src/identidad.rs
@@ -30,7 +30,8 @@ use crate::i18n::catalogo::identidad as t;
 use crate::i18n::{Mensaje, Texto};
 use crate::plataforma::personas::RUTA as RUTA_PERSONAS;
 use crate::plataforma::{
-    Credenciales, Destino, GitHub, GitLab, Personas, Plataforma, Proveedor, Rol, Usuario, detectar,
+    Credenciales, CredencialesEntorno, Destino, GitHub, GitLab, Personas, Plataforma, PrAbierto,
+    Proveedor, Rol, Usuario, detectar,
 };
 use crate::tarea::Asignado;
 
@@ -512,6 +513,88 @@ pub fn adaptador(
     })
 }
 
+/// Lo que se supo de la plataforma: la clave `plataforma=` de `board` y de
+/// `doctor`.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum Consulta {
+    /// `--sin-plataforma`.
+    NoConsultada,
+    /// Sin `origin` o con una plataforma que no se reconoce.
+    NoConfigurada,
+    /// Sin token del rol, o con uno que la plataforma no acepta.
+    SinCredenciales,
+    /// No se pudo hablar con la plataforma.
+    SinRed,
+    /// Cualquier otro fallo (permisos, respuesta invalida, mas paginas de
+    /// las que se leen...): falla cerrado.
+    Error,
+    /// Los PR abiertos del repo.
+    Hecha(Vec<PrAbierto>),
+}
+
+impl Consulta {
+    /// Valor de `plataforma=`.
+    pub fn nombre(&self) -> &'static str {
+        match self {
+            Self::NoConsultada => "no-consultada",
+            Self::NoConfigurada => "no-configurada",
+            Self::SinCredenciales => "sin-credenciales",
+            Self::SinRed => "sin-red",
+            Self::Error => "error",
+            Self::Hecha(_) => "ok",
+        }
+    }
+
+    /// La de un fallo de una lectura de la plataforma.
+    pub fn de_fallo(i: &Informe) -> Self {
+        match (i.resultado, i.valor("motivo")) {
+            (Resultado::Sinred, _) => Self::SinRed,
+            (Resultado::Rechazada, Some("identidad"))
+            | (Resultado::Denegada, Some("credenciales")) => Self::SinCredenciales,
+            _ => Self::Error,
+        }
+    }
+}
+
+/// Un adaptador listo y el rol con el que se le pregunta.
+pub struct Conectada {
+    /// El adaptador.
+    pub plataforma: Box<dyn Plataforma>,
+    /// Rol de la consulta.
+    pub rol: Rol,
+}
+
+/// Construye el adaptador con la lista de personas de la base, o dice por
+/// que no ([`Consulta::NoConfigurada`] o [`Consulta::SinCredenciales`]).
+pub type Conectar<'a> = dyn Fn(&Personas) -> Result<Conectada, Consulta> + 'a;
+
+/// El adaptador de produccion: plataforma por el host de `origin` y la
+/// clave `plataforma` de la configuracion de la base (F1-13, decision 7),
+/// tokens de las variables `TC2_TOKEN_*`. Pregunta con el token de persona
+/// si lo hay (solo con terminal, F1-6 decision 5) y si no con el de agente;
+/// es una eleccion previa, no un reintento con otro rol. La usan `board` y
+/// `doctor`.
+pub fn conectar_produccion<'a>(
+    g: &'a Git,
+    dir: &'a Path,
+) -> impl Fn(&Personas) -> Result<Conectada, Consulta> + 'a {
+    move |personas: &Personas| {
+        // La misma deteccion que `identidad` y `start` (D6 de TASK-033).
+        let destino = destino_de(g, dir).map_err(|_| Consulta::NoConfigurada)?;
+        let credenciales = CredencialesEntorno;
+        let rol = if credenciales.token(Rol::Persona).is_some() {
+            Rol::Persona
+        } else if credenciales.token(Rol::Agente).is_some() {
+            Rol::Agente
+        } else {
+            return Err(Consulta::SinCredenciales);
+        };
+        let plataforma = adaptador(destino, Box::new(credenciales), personas.clone())
+            .map_err(|_| Consulta::NoConfigurada)?;
+        Ok(Conectada { plataforma, rol })
+    }
+}
+
 /// Quien ejecuta en el repo de `e` (D6 de TASK-033): la plataforma de
 /// `origin`, la lista de personas de la base y [`quien_ejecuta`] con el
 /// terminal de `e`. Con terminal trae antes la rama base (hace falta red
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 766cf08..5d9a746 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -17,7 +17,7 @@
 //! [`siguiente`] (orden de trabajo y `next`, F3-3) y [`situacion`] y
 //! [`roadmap`] (el calculo comun y `roadmap`, F3-4) y [`tablero`]
 //! (`board` y `status`, F3-5) y [`toma`] (`start`, F3-6) y [`ciclo`]
-//! (`descartar`, `pausa` y `reanudar`, F3-7). Las capas
+//! (`descartar`, `pausa` y `reanudar`, F3-7) y [`doctor`] (F3-8). Las capas
 //! que vienen entran como modulos de esta biblioteca.
 
 pub mod actualizar_cli;
@@ -28,6 +28,7 @@ pub mod config;
 pub mod contrato;
 pub mod descarga;
 pub mod desinstalar;
+pub mod doctor;
 pub mod estado;
 pub mod git;
 pub mod grafo;
@@ -149,6 +150,10 @@ where
             let (json, resto) = con_json(resto);
             init::orden(&resto, json, idioma)
         }
+        [orden, resto @ ..] if orden == doctor::NOMBRE => {
+            let (json, resto) = con_json(resto);
+            orden_doctor(&resto, json, idioma)
+        }
         [orden, resto @ ..] if orden == toma::NOMBRE => {
             let (json, resto) = con_json(resto);
             de_una_tarea(toma::NOMBRE, |e| toma::orden(&resto, e)).salida_en(json, idioma)
@@ -347,6 +352,23 @@ fn orden_tablero(orden: &str, args: &[String], json: bool, idioma: Idioma) -> Sa
     }
 }
 
+/// `doctor [--json]` (F3-8) en la carpeta actual, con la plataforma de
+/// produccion (la misma deteccion y los mismos tokens que `board`).
+fn orden_doctor(args: &[String], json: bool, idioma: Idioma) -> Salida {
+    let dir = match tablero::carpeta_actual(doctor::NOMBRE) {
+        Ok(d) => d,
+        Err(i) => return i.salida_en(json, idioma),
+    };
+    let g = git::Git::nuevo();
+    let conectar = tablero::conectar_produccion(&g, &dir);
+    let e = doctor::Entorno {
+        dir: &dir,
+        git: &g,
+        conectar: &conectar,
+    };
+    doctor::orden(args, json, idioma, &e)
+}
+
 /// Catalogo de produccion: reconstruye las operaciones de la rama de estado
 /// desde su anotacion. Las operaciones nuevas se anaden aqui.
 fn catalogo(anotacion: &str) -> Option<Box<dyn estado::Operacion>> {
diff --git a/crates/tc2/src/plataforma/github.rs b/crates/tc2/src/plataforma/github.rs
index ca32cde..91ac519 100644
--- a/crates/tc2/src/plataforma/github.rs
+++ b/crates/tc2/src/plataforma/github.rs
@@ -17,7 +17,8 @@ use super::errores::{self, es_aviso_de_plan};
 use super::http::{Base, Metodo, codificar};
 use super::puertas::{self, EstadoCheck, Lecturas};
 use super::{
-    Firma, ModeloPuertas, Paso, Personas, Plataforma, PrAbierto, Proveedor, ReglasLeidas, Usuario,
+    Firma, ModeloPuertas, Paso, Personas, PlanObservado, Plataforma, PrAbierto, Proveedor,
+    ReglasLeidas, Usuario,
 };
 use crate::contrato::{Informe, Resultado, Valor, corto};
 use crate::i18n::catalogo::plataforma as t;
@@ -29,6 +30,13 @@ const LEER_PERMISOS_CI: &str = "leer-permisos-ci";
 const FIRMA_COMMIT: &str = "firma-commit";
 const IDENTIDAD: &str = "identidad";
 const PRS_ABIERTOS: &str = "prs-abiertos";
+const LEER_PLAN: &str = "leer-plan";
+
+/// Permiso de `leer-puertas` (lo nombra `doctor` en `permiso=`).
+pub const PERMISO_LEER_PUERTAS: &str = "metadata:read";
+
+/// Permiso de `leer-permisos-ci`.
+pub const PERMISO_PERMISOS_CI: &str = "administration:read";
 
 /// Garantias que en GitHub dan los rulesets (tier «Free publico;
 /// Pro/Team/Enterprise» de la tabla de F1-13): sin ellos, el 403 de plan.
@@ -39,6 +47,7 @@ const GARANTIAS_DE_RULESETS: [&str; 10] = [
 /// Endpoints de la API que usa el adaptador (lista cerrada).
 enum Endpoint<'a> {
     Usuario,
+    Repo,
     Pr(u64),
     PrsAbiertos { cabeza: &'a str, base: &'a str },
     TodosLosPrsAbiertos,
@@ -47,6 +56,7 @@ enum Endpoint<'a> {
     ReglasRama(&'a str),
     Rulesets,
     CheckRuns(&'a str),
+    CheckRunsDeRama(&'a str),
     Reviews(u64),
     PermisosWorkflow,
     Commit(&'a str),
@@ -57,6 +67,7 @@ impl Endpoint<'_> {
         let r = format!("/repos/{repo}");
         match self {
             Self::Usuario => (Metodo::Get, "/user".into()),
+            Self::Repo => (Metodo::Get, r),
             Self::Pr(n) => (Metodo::Get, format!("{r}/pulls/{n}")),
             Self::PrsAbiertos { cabeza, base } => (
                 Metodo::Get,
@@ -77,6 +88,10 @@ impl Endpoint<'_> {
                 Metodo::Get,
                 format!("{r}/commits/{sha}/check-runs?per_page=100"),
             ),
+            Self::CheckRunsDeRama(rama) => (
+                Metodo::Get,
+                format!("{r}/commits/heads/{rama}/check-runs?per_page=100"),
+            ),
             Self::Reviews(n) => (Metodo::Get, format!("{r}/pulls/{n}/reviews?per_page=100")),
             Self::PermisosWorkflow => (Metodo::Get, format!("{r}/actions/permissions/workflow")),
             Self::Commit(sha) => (Metodo::Get, format!("{r}/commits/{sha}")),
@@ -209,6 +224,21 @@ struct VerificacionApi {
     reason: String,
 }
 
+/// Lo que se lee del repo para el plan (GH-01 de F1-13).
+#[derive(Deserialize)]
+struct RepoPlanApi {
+    private: bool,
+    #[serde(default)]
+    visibility: Option<String>,
+    owner: DuenoApi,
+}
+
+#[derive(Deserialize)]
+struct DuenoApi {
+    #[serde(rename = "type")]
+    tipo: String,
+}
+
 #[derive(Deserialize)]
 struct RulesetApi {
     name: String,
@@ -218,6 +248,23 @@ struct RulesetApi {
 struct ReglaApi {
     #[serde(rename = "type")]
     tipo: String,
+    /// Los de `pull_request` y `required_status_checks`; los demas tipos
+    /// traen otros campos (o ninguno) y se ignoran.
+    #[serde(default)]
+    parameters: Option<ParametrosApi>,
+}
+
+#[derive(Deserialize, Default)]
+struct ParametrosApi {
+    #[serde(default)]
+    required_approving_review_count: Option<u64>,
+    #[serde(default)]
+    required_status_checks: Vec<CheckRequeridoApi>,
+}
+
+#[derive(Deserialize)]
+struct CheckRequeridoApi {
+    context: String,
 }
 
 /// Adaptador de GitHub para un repo.
@@ -573,7 +620,7 @@ impl Plataforma for GitHub {
         if rol == Rol::Revisor {
             return Err(rol_no_valido(LEER_PUERTAS, rol));
         }
-        let permiso = "metadata:read";
+        let permiso = PERMISO_LEER_PUERTAS;
         let r = self
             .con
             .pedir(LEER_PUERTAS, rol, self.ep(&Endpoint::Rulesets), None)?;
@@ -593,10 +640,39 @@ impl Plataforma for GitHub {
             permiso,
             self.ep(&Endpoint::ReglasRama(rama)),
         )?;
+        let parametros = |tipo: &'static str| {
+            reglas
+                .iter()
+                .filter(move |r| r.tipo == tipo)
+                .filter_map(|r| r.parameters.as_ref())
+        };
+        let aprobaciones = parametros("pull_request")
+            .filter_map(|p| p.required_approving_review_count)
+            .max()
+            .unwrap_or(0);
+        let requeridos: Vec<String> = parametros("required_status_checks")
+            .flat_map(|p| p.required_status_checks.iter().map(|c| c.context.clone()))
+            .collect();
+        // Los check-runs de la cabeza de la rama (`heads/<rama>`, la forma
+        // que documenta la API). Si no se pueden leer, no se supone nada.
+        let (_, ruta) = self.ep(&Endpoint::CheckRunsDeRama(rama));
+        let publicados = self
+            .con
+            .leer_paginas(LEER_PUERTAS, rol, permiso, ruta, |c| {
+                self.con
+                    .parsear::<CheckRunsApi>(LEER_PUERTAS, c)
+                    .map(|x| x.check_runs)
+            })
+            .ok()
+            .map(|v| v.into_iter().map(|r| r.name).collect());
         Ok(ReglasLeidas {
             plan_da_reglas: true,
             rulesets: rulesets.into_iter().map(|r| r.name).collect(),
             reglas_rama: reglas.into_iter().map(|r| r.tipo).collect(),
+            aprobaciones,
+            checks_requeridos: Some(requeridos),
+            todos_los_checks: false,
+            checks_publicados: publicados,
         })
     }
 
@@ -607,7 +683,7 @@ impl Plataforma for GitHub {
         let p: PermisosApi = match self.con.leer(
             LEER_PERMISOS_CI,
             Rol::Persona,
-            "administration:read",
+            PERMISO_PERMISOS_CI,
             self.ep(&Endpoint::PermisosWorkflow),
         ) {
             Ok(p) => p,
@@ -656,6 +732,27 @@ impl Plataforma for GitHub {
                 .map(Usuario::from)
         })
     }
+
+    fn proveedor(&self) -> Option<Proveedor> {
+        Some(Proveedor::GitHub)
+    }
+
+    fn leer_plan(&self, rol: Rol) -> Paso<PlanObservado> {
+        if rol == Rol::Revisor {
+            return Err(rol_no_valido(LEER_PLAN, rol));
+        }
+        let r: RepoPlanApi = self.con.leer(
+            LEER_PLAN,
+            rol,
+            PERMISO_LEER_PUERTAS,
+            self.ep(&Endpoint::Repo),
+        )?;
+        Ok(PlanObservado::GitHub {
+            // `internal` (Enterprise) cuenta como privado.
+            publico: !r.private && r.visibility.as_deref().is_none_or(|v| v == "public"),
+            organizacion: r.owner.tipo == "Organization",
+        })
+    }
 }
 
 #[cfg(test)]
diff --git a/crates/tc2/src/plataforma/gitlab.rs b/crates/tc2/src/plataforma/gitlab.rs
index e34cb33..02b8a77 100644
--- a/crates/tc2/src/plataforma/gitlab.rs
+++ b/crates/tc2/src/plataforma/gitlab.rs
@@ -18,8 +18,8 @@ use super::errores;
 use super::http::{Base, Metodo, codificar};
 use super::puertas::{self, EstadoCheck, Lecturas};
 use super::{
-    Clave, Firma, ModeloPuertas, Paso, Personas, Plataforma, PrAbierto, Proveedor, ReglasLeidas,
-    Usuario,
+    Clave, Firma, ModeloPuertas, Paso, Personas, PlanObservado, Plataforma, PrAbierto, Proveedor,
+    ReglasLeidas, TierGitLab, Usuario,
 };
 use crate::contrato::{Informe, Resultado, Valor, corto};
 use crate::i18n::catalogo::plataforma as t;
@@ -32,6 +32,11 @@ const LEER_PERMISOS_CI: &str = "leer-permisos-ci";
 const FIRMA_COMMIT: &str = "firma-commit";
 const IDENTIDAD: &str = "identidad";
 const PRS_ABIERTOS: &str = "prs-abiertos";
+const LEER_PLAN: &str = "leer-plan";
+
+/// El scope de lectura en una clave (`doctor`, `permiso=`); el texto
+/// completo, con el rol, es [`PERMISO_LECTURA`].
+pub const PERMISO_LECTURA_CORTO: &str = "read_api";
 
 /// Permiso de lectura que necesita un token de GitLab.
 static PERMISO_LECTURA: &Texto = &t::PERMISO_LECTURA_GITLAB;
@@ -67,6 +72,9 @@ enum Endpoint<'a> {
     Aprobaciones(u64),
     Firma(&'a str),
     Commit(&'a str),
+    Proyecto,
+    Rama(&'a str),
+    Namespace(u64),
 }
 
 impl Endpoint<'_> {
@@ -116,6 +124,14 @@ impl Endpoint<'_> {
                 format!("{p}/repository/commits/{sha}/signature"),
             ),
             Self::Commit(sha) => (Metodo::Get, format!("{p}/repository/commits/{sha}")),
+            Self::Proyecto => (Metodo::Get, p),
+            // Fuera del proyecto: la ruta la construye el adaptador con el
+            // id numerico que da el propio proyecto.
+            Self::Namespace(id) => (Metodo::Get, format!("/namespaces/{id}")),
+            Self::Rama(rama) => (
+                Metodo::Get,
+                format!("{p}/repository/branches/{}", codificar(rama)),
+            ),
         }
     }
 }
@@ -255,6 +271,32 @@ struct CommitApi {
     id: String,
 }
 
+/// Lo que se lee del proyecto. Los dos campos solo salen con sesion
+/// (GL-01 de F1-13): ausentes, `None`.
+#[derive(Deserialize)]
+struct ProyectoApi {
+    #[serde(default)]
+    only_allow_merge_if_pipeline_succeeds: Option<bool>,
+    #[serde(default)]
+    namespace: Option<NamespaceRefApi>,
+}
+
+#[derive(Deserialize)]
+struct NamespaceRefApi {
+    id: u64,
+}
+
+#[derive(Deserialize)]
+struct NamespaceApi {
+    #[serde(default)]
+    plan: Option<String>,
+}
+
+#[derive(Deserialize)]
+struct RamaApi {
+    commit: CommitApi,
+}
+
 /// Adaptador de GitLab para un proyecto.
 pub struct GitLab {
     con: Conexion,
@@ -317,6 +359,48 @@ impl GitLab {
             .map(|m| m.iid))
     }
 
+    /// Checks requeridos del proyecto (F3-8): GitLab no exige checks por
+    /// nombre; con «Pipelines must succeed» cuenta el pipeline entero
+    /// (`todos`). `None` si el proyecto no se pudo leer o no trae el campo
+    /// (sin sesion no sale, GL-01 de F1-13).
+    fn checks_requeridos(&self, rol: Rol) -> (Option<Vec<String>>, bool) {
+        let p = self.con.leer::<ProyectoApi>(
+            LEER_PUERTAS,
+            rol,
+            PERMISO_LECTURA,
+            self.ep(&Endpoint::Proyecto),
+        );
+        match p.ok().and_then(|p| p.only_allow_merge_if_pipeline_succeeds) {
+            Some(todos) => (Some(Vec::new()), todos),
+            None => (None, false),
+        }
+    }
+
+    /// Nombres de los estados de la cabeza de `rama` (la API de estados
+    /// pide el sha: se lee antes la rama). `None` si algo no se lee.
+    fn checks_publicados(&self, rol: Rol, rama: &str) -> Option<Vec<String>> {
+        let r: RamaApi = self
+            .con
+            .leer(
+                LEER_PUERTAS,
+                rol,
+                PERMISO_LECTURA,
+                self.ep(&Endpoint::Rama(rama)),
+            )
+            .ok()?;
+        if !sha_valido(&r.commit.id) {
+            return None;
+        }
+        let (_, ruta) = self.ep(&Endpoint::Estados(&r.commit.id));
+        let estados: Vec<EstadoApi> = self
+            .con
+            .leer_paginas(LEER_PUERTAS, rol, PERMISO_LECTURA, ruta, |c| {
+                self.con.parsear(LEER_PUERTAS, c)
+            })
+            .ok()?;
+        Some(estados.into_iter().map(|e| e.name).collect())
+    }
+
     /// Si `rama` tiene commits que no estan en `base`.
     fn tiene_commits(&self, rama: &str, base: &str) -> Paso<bool> {
         let commits: Vec<CommitApi> = self.con.leer(
@@ -730,21 +814,27 @@ impl Plataforma for GitLab {
             self.ep(&Endpoint::ReglasAprobacion),
             None,
         )?;
-        if r.estado == 403 {
-            return Ok(ReglasLeidas {
-                plan_da_reglas: false,
-                rulesets: Vec::new(),
-                reglas_rama,
-            });
-        }
-        if !r.exito() {
+        let (plan_da_reglas, reglas) = if r.estado == 403 {
+            (false, Vec::new())
+        } else if r.exito() {
+            let v: Vec<ReglaAprobacionApi> = self.con.parsear(LEER_PUERTAS, &r.cuerpo)?;
+            (true, v)
+        } else {
             return Err(self.con.traducir(LEER_PUERTAS, rol, PERMISO_LECTURA, &r));
-        }
-        let reglas: Vec<ReglaAprobacionApi> = self.con.parsear(LEER_PUERTAS, &r.cuerpo)?;
+        };
+        let (requeridos, todos) = self.checks_requeridos(rol);
         Ok(ReglasLeidas {
-            plan_da_reglas: true,
+            plan_da_reglas,
+            aprobaciones: reglas
+                .iter()
+                .map(|r| r.approvals_required)
+                .max()
+                .unwrap_or(0),
             rulesets: reglas.into_iter().map(|r| r.name).collect(),
             reglas_rama,
+            checks_requeridos: requeridos,
+            todos_los_checks: todos,
+            checks_publicados: self.checks_publicados(rol, rama),
         })
     }
 
@@ -811,6 +901,37 @@ impl Plataforma for GitLab {
                 .map(Usuario::from)
         })
     }
+
+    fn proveedor(&self) -> Option<Proveedor> {
+        Some(Proveedor::GitLab)
+    }
+
+    fn leer_plan(&self, rol: Rol) -> Paso<PlanObservado> {
+        if rol == Rol::Revisor {
+            return Err(rol_no_valido(LEER_PLAN, rol));
+        }
+        let p: ProyectoApi = self.con.leer(
+            LEER_PLAN,
+            rol,
+            PERMISO_LECTURA,
+            self.ep(&Endpoint::Proyecto),
+        )?;
+        let Some(ns) = p.namespace else {
+            return Ok(PlanObservado::GitLab(None));
+        };
+        // El `plan` del namespace (API de namespaces; la pagina no dice
+        // quien lo ve): si no se lee o no se reconoce, no se supone ninguno.
+        let r = self
+            .con
+            .pedir(LEER_PLAN, rol, self.ep(&Endpoint::Namespace(ns.id)), None)?;
+        if !r.exito() {
+            return Ok(PlanObservado::GitLab(None));
+        }
+        let n: NamespaceApi = self.con.parsear(LEER_PLAN, &r.cuerpo)?;
+        Ok(PlanObservado::GitLab(
+            n.plan.as_deref().and_then(TierGitLab::de_plan),
+        ))
+    }
 }
 
 #[cfg(test)]
diff --git a/crates/tc2/src/plataforma/mod.rs b/crates/tc2/src/plataforma/mod.rs
index dd6b340..3576c55 100644
--- a/crates/tc2/src/plataforma/mod.rs
+++ b/crates/tc2/src/plataforma/mod.rs
@@ -136,6 +136,72 @@ pub struct ReglasLeidas {
     /// proteccion de la rama (GitLab: `push=N`, `merge=N`, `force-push`,
     /// `code-owner`).
     pub reglas_rama: Vec<String>,
+    /// Aprobaciones que pide la plataforma (F3-8): GitHub, el
+    /// `required_approving_review_count` de la regla `pull_request` de la
+    /// rama; GitLab, el mayor `approvals_required` de las reglas de
+    /// aprobacion. 0 si no pide ninguna.
+    pub aprobaciones: u64,
+    /// Checks que la rama exige por nombre (F3-8; GitHub: los `context` de
+    /// `required_status_checks`). `None` si no se pudo leer.
+    pub checks_requeridos: Option<Vec<String>>,
+    /// GitLab con «Pipelines must succeed»: cuenta todo el pipeline, asi
+    /// que cualquier check publicado en el es obligatorio (sin origen: G09
+    /// sustituido por `autor-del-estado`).
+    pub todos_los_checks: bool,
+    /// Nombres de los checks publicados en la cabeza de la rama (GitHub,
+    /// check-runs; GitLab, estados del commit). `None` si no se pudieron
+    /// leer (rama inexistente, permisos...).
+    pub checks_publicados: Option<Vec<String>>,
+}
+
+/// Tier de GitLab (F1-13, decision 2).
+#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
+pub enum TierGitLab {
+    /// Free.
+    Free,
+    /// Premium.
+    Premium,
+    /// Ultimate.
+    Ultimate,
+}
+
+impl TierGitLab {
+    /// El tier del `plan` de un namespace (`free`, `premium`, `ultimate`);
+    /// cualquier otro valor (`default`, nombres antiguos) no se reconoce.
+    pub fn de_plan(plan: &str) -> Option<Self> {
+        match plan {
+            "free" => Some(Self::Free),
+            "premium" => Some(Self::Premium),
+            "ultimate" => Some(Self::Ultimate),
+            _ => None,
+        }
+    }
+
+    /// Nombre en la linea.
+    pub fn nombre(self) -> &'static str {
+        match self {
+            Self::Free => "free",
+            Self::Premium => "premium",
+            Self::Ultimate => "ultimate",
+        }
+    }
+}
+
+/// Lo que se observa del plan (F1-13 decision 3: «el plan se observa, no se
+/// declara»). En GitHub, la visibilidad y el tipo de cuenta del repo (el
+/// 403 de plan de los rulesets lo aporta `leer-puertas`); en GitLab, el
+/// `plan` del namespace, si se lee y se reconoce.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum PlanObservado {
+    /// Repo de GitHub.
+    GitHub {
+        /// Publico (ni privado ni interno).
+        publico: bool,
+        /// De una organizacion.
+        organizacion: bool,
+    },
+    /// Proyecto de GitLab, con el tier si se reconoce.
+    GitLab(Option<TierGitLab>),
 }
 
 /// Un PR (o MR) abierto del propio repo, con lo minimo que necesita `board`
@@ -184,4 +250,19 @@ pub trait Plataforma {
 
     /// Usuario autenticado con el token de ese rol.
     fn identidad(&self, rol: Rol) -> Paso<Usuario>;
+
+    /// La plataforma del adaptador (F3-8: `doctor` traduce lo leido con la
+    /// tabla de su plataforma). `None` en los adaptadores de prueba.
+    fn proveedor(&self) -> Option<Proveedor> {
+        None
+    }
+
+    /// `leer-plan` (agente o persona; F3-8): lo que se observa del plan
+    /// para `capacidades`. Por defecto no se sabe (adaptadores de prueba).
+    fn leer_plan(&self, _rol: Rol) -> Paso<PlanObservado> {
+        Err(
+            Informe::nuevo("leer-plan", crate::contrato::Resultado::Error)
+                .con("motivo", "no-soportada"),
+        )
+    }
 }
diff --git a/crates/tc2/src/tablero.rs b/crates/tc2/src/tablero.rs
index 357b4f7..1692e22 100644
--- a/crates/tc2/src/tablero.rs
+++ b/crates/tc2/src/tablero.rs
@@ -28,7 +28,7 @@ use crate::i18n::catalogo::cli as txt_cli;
 use crate::i18n::catalogo::roadmap as txt_roadmap;
 use crate::i18n::catalogo::tablero as txt;
 use crate::i18n::{Idioma, Mensaje};
-use crate::plataforma::{Credenciales, CredencialesEntorno, Personas, Plataforma, PrAbierto, Rol};
+use crate::plataforma::{Personas, PrAbierto};
 use crate::situacion::{self, Ficha, Quien, Situacion};
 
 /// Nombre de `board`.
@@ -292,59 +292,9 @@ fn tablero_en(
     Ok((tablero(&s, &personas, &commit), avisos, personas))
 }
 
-/// Lo que se supo de la plataforma: la clave `plataforma=` de `board`.
-#[derive(Debug, Clone, PartialEq, Eq)]
-pub enum Consulta {
-    /// `--sin-plataforma`.
-    NoConsultada,
-    /// Sin `origin` o con una plataforma que no se reconoce.
-    NoConfigurada,
-    /// Sin token del rol, o con uno que la plataforma no acepta.
-    SinCredenciales,
-    /// No se pudo hablar con la plataforma.
-    SinRed,
-    /// Cualquier otro fallo (permisos, respuesta invalida, mas paginas de
-    /// las que se leen...): falla cerrado.
-    Error,
-    /// Los PR abiertos del repo.
-    Hecha(Vec<PrAbierto>),
-}
-
-impl Consulta {
-    /// Valor de `plataforma=`.
-    pub fn nombre(&self) -> &'static str {
-        match self {
-            Self::NoConsultada => "no-consultada",
-            Self::NoConfigurada => "no-configurada",
-            Self::SinCredenciales => "sin-credenciales",
-            Self::SinRed => "sin-red",
-            Self::Error => "error",
-            Self::Hecha(_) => "ok",
-        }
-    }
-
-    /// La de un fallo de `prs-abiertos`.
-    pub fn de_fallo(i: &Informe) -> Self {
-        match (i.resultado, i.valor("motivo")) {
-            (Resultado::Sinred, _) => Self::SinRed,
-            (Resultado::Rechazada, Some("identidad"))
-            | (Resultado::Denegada, Some("credenciales")) => Self::SinCredenciales,
-            _ => Self::Error,
-        }
-    }
-}
-
-/// Un adaptador listo y el rol con el que se le pregunta.
-pub struct Conectada {
-    /// El adaptador.
-    pub plataforma: Box<dyn Plataforma>,
-    /// Rol de la consulta.
-    pub rol: Rol,
-}
-
-/// Construye el adaptador con la lista de personas de la base, o dice por
-/// que no ([`Consulta::NoConfigurada`] o [`Consulta::SinCredenciales`]).
-pub type Conectar<'a> = dyn Fn(&Personas) -> Result<Conectada, Consulta> + 'a;
+// La conexion con la plataforma vive en `identidad` desde F3-8 (la usan
+// `board` y `doctor`); se reexporta para no cambiar la API de `board`.
+pub use crate::identidad::{Conectada, Conectar, Consulta, conectar_produccion};
 
 /// Lo que necesitan las dos ordenes del entorno.
 pub struct Entorno<'a> {
@@ -356,33 +306,6 @@ pub struct Entorno<'a> {
     pub conectar: &'a Conectar<'a>,
 }
 
-/// El adaptador de produccion: plataforma por el host de `origin` y la
-/// clave `plataforma` de la configuracion de la base (F1-13, decision 7),
-/// tokens de las variables `TC2_TOKEN_*`. Pregunta con el token de persona
-/// si lo hay (solo con terminal, F1-6 decision 5) y si no con el de agente;
-/// es una eleccion previa, no un reintento con otro rol.
-pub fn conectar_produccion<'a>(
-    g: &'a Git,
-    dir: &'a Path,
-) -> impl Fn(&Personas) -> Result<Conectada, Consulta> + 'a {
-    move |personas: &Personas| {
-        // La misma deteccion que `identidad` y `start` (D6 de TASK-033).
-        let destino = crate::identidad::destino_de(g, dir).map_err(|_| Consulta::NoConfigurada)?;
-        let credenciales = CredencialesEntorno;
-        let rol = if credenciales.token(Rol::Persona).is_some() {
-            Rol::Persona
-        } else if credenciales.token(Rol::Agente).is_some() {
-            Rol::Agente
-        } else {
-            return Err(Consulta::SinCredenciales);
-        };
-        let plataforma =
-            crate::identidad::adaptador(destino, Box::new(credenciales), personas.clone())
-                .map_err(|_| Consulta::NoConfigurada)?;
-        Ok(Conectada { plataforma, rol })
-    }
-}
-
 /// El PR de cada tarea y los avisos (C3b).
 #[derive(Debug, Clone, Default, PartialEq, Eq)]
 pub struct Cruce {
@@ -643,7 +566,7 @@ pub fn carpeta_actual(op: &str) -> Result<std::path::PathBuf, Informe> {
 #[cfg(test)]
 mod tests {
     use super::*;
-    use crate::plataforma::{Firma, ModeloPuertas, Paso, ReglasLeidas, Usuario};
+    use crate::plataforma::{Firma, ModeloPuertas, Paso, Plataforma, ReglasLeidas, Rol, Usuario};
     use crate::roadmap::tests::{estado_de_referencia, personas};
 
     const COMMIT: &str = "0123456789ab";
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-035/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-035/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-035/tarea.md                                    | 30 ++++++++++++++++++++++++++++--
 3 files changed, 28 insertions(+), 2 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/doctor.rs"
````

````
crates/tc2/tests/doctor.rs | 965 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 965 insertions(+)
````
