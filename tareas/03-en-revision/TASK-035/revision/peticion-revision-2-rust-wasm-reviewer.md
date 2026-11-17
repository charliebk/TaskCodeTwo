# Peticion de revision — TASK-035 (ronda 2)

- Tarea: TASK-035 — F3-8 doctor: entorno, rama de estado, permisos, rulesets y checks
- Rama revisada: feature/task-035-f3-8-doctor-entorno-rama-de-estado-permi
- Rama base: develop
- Commit revisado (HEAD): f5b75c0e07d17e946b10a6c7254bdb06b463e1cf
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-035 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 1aa0a12edd779a7211e471d9132d566be737d3da (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (11 hallazgo(s)))

## Instrucciones para el agente revisor (verificacion acotada)

Eres un revisor INDEPENDIENTE del agente que implemento. La rama ya paso una
revision completa: esta ronda verifica las correcciones de la ronda anterior,
no vuelve a revisar la rama entera ni los MENORES de la ronda anterior (tu
tabla solo trae CRITICOS e IMPORTANTES). Haz esto, y nada mas:

1. Comprueba, uno por uno, que cada hallazgo pendiente de tu tabla de abajo
   queda resuelto por el diff de esta ronda (los marcados `corregido` tambien:
   verificalo, no lo des por bueno). Para un CRITICO o IMPORTANTE, reproduce el
   fallo original y comprueba que ya no ocurre; puedes leer el repo entero si la
   causa no esta en el diff.
2. Ejecuta la suite (o el comando de verificacion del proyecto) sobre el commit
   revisado y di el resultado: verificar sin ejecutar no vale. Si esta peticion
   ya trae la verificacion de la ronda en verde para ese mismo commit, no la
   repitas: cita ese resultado y corre solo los tests de lo que verificas.
3. Busca en el diff de esta ronda el fallo que la correccion abre: es justo
   donde se cuelan los fallos nuevos. Si encuentras uno, construye el caso que
   lo rompe antes de reportarlo.

Solo abre un hallazgo NUEVO si es CRITICO o si lo introduce la propia correccion
(algo que el diff de esta ronda rompe). Lo demas que veas fuera de eso va como
nota al final del informe, no en la tabla: no abre otra ronda.

Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. En la tabla, cada hallazgo pendiente que ya este
resuelto va como `corregido`; el que no, `abierto`. Vuelca tu salida en el
informe de esta ronda (informe-revision-2-rust-wasm-reviewer.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-035/tarea.md
- tareas/03-en-revision/TASK-035/planificacion/
- crates/tc2/src/doctor/capacidades.rs
- crates/tc2/src/doctor/estado.rs
- crates/tc2/src/doctor/mod.rs
- crates/tc2/src/doctor/plataforma.rs
- crates/tc2/src/estado/lectura.rs
- crates/tc2/src/estado/mod.rs
- crates/tc2/src/plataforma/github.rs
- crates/tc2/src/plataforma/gitlab.rs
- crates/tc2/src/plataforma/mod.rs
- crates/tc2/tests/doctor.rs
- docs/adr/F1-13-adaptador-de-plataforma.md
- docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 3 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| H1 | IMPORTANTE | corregido | crates/tc2/src/doctor/plataforma.rs:287-295; crates/tc2/src/doctor/capacidades.rs:376-383 | informe-revision-1-rust-wasm-reviewer.md |
| H2 | IMPORTANTE | corregido | crates/tc2/src/doctor/capacidades.rs:309-319 (tests :459-620) | informe-revision-1-rust-wasm-reviewer.md |

## Commits a revisar (git log 1aa0a12edd779a7211e471d9132d566be737d3da..HEAD)

````
f5b75c0 docs(TASK-035): adendas de F1-13 y F1-4 tras la ronda 1
0158805 test(TASK-035): reproducciones de la ronda 1 contra el mock
af26182 fix(TASK-035): doctor no da por impuesta una garantia sin leer su config
82d7239 refactor(TASK-035): doctor decide sobre la ref de estado que expone estado
fe169a0 chore(TASK-035): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 1aa0a12edd779a7211e471d9132d566be737d3da..HEAD)

````diff
diff --git a/crates/tc2/src/doctor/capacidades.rs b/crates/tc2/src/doctor/capacidades.rs
index 7d10b45..60a2668 100644
--- a/crates/tc2/src/doctor/capacidades.rs
+++ b/crates/tc2/src/doctor/capacidades.rs
@@ -10,9 +10,13 @@
 //! configuracion). El plan se observa (`leer-plan` y el 403 de plan de
 //! `leer-puertas`); sin plan, «plan no comprobado» y no se supone nada.
 //! Una garantia que el plan da pero que las puertas leidas no tienen sale
-//! `no-disponible motivo=config arreglo=tc2-init`; solo se mira la
-//! configuracion de las que `leer-puertas` lee (G01, G02, G03, G06 y G09);
-//! las demas siguen al plan.
+//! `no-disponible motivo=config arreglo=tc2-init`. Solo se mira la
+//! configuracion de las que `leer-puertas` lee ([`lee_config`]: en GitHub
+//! G01, G02, G03, G04, G05 y G09; en GitLab G02, G03 y G06); las demas
+//! siguen al plan. Si la configuracion de una de esas no se pudo leer (sin
+//! puertas, o el 403 de las reglas de aprobacion de GitLab con un plan que
+//! las da), sale `no-comprobado`: nunca `impuesta` sin leerla (H1 y H4 de
+//! TASK-035).
 
 use super::{ARREGLO_INIT, Estado, Hallazgo};
 use crate::plataforma::{PlanObservado, Proveedor, ReglasLeidas, TierGitLab};
@@ -116,6 +120,8 @@ pub enum EstadoGarantia {
     Sustituida(&'static str),
     /// No la hay.
     NoDisponible,
+    /// El plan la da, pero su configuracion no se pudo leer.
+    NoComprobada,
 }
 
 /// Por que una garantia no esta impuesta.
@@ -127,6 +133,10 @@ pub enum Motivo {
     Plataforma,
     /// El plan la da y el proyecto no la tiene configurada.
     Config,
+    /// No se leyeron las puertas de la rama.
+    PuertasNoLeidas,
+    /// Un 403 al leer la configuracion con un plan que la da.
+    PermisoInsuficiente,
 }
 
 impl Motivo {
@@ -136,6 +146,8 @@ impl Motivo {
             Self::Plan => "plan",
             Self::Plataforma => "plataforma",
             Self::Config => "config",
+            Self::PuertasNoLeidas => "puertas-no-leidas",
+            Self::PermisoInsuficiente => "permiso-insuficiente",
         }
     }
 }
@@ -230,18 +242,60 @@ pub fn plan_de(
     }
 }
 
-/// Si las puertas leidas no tienen configurada una garantia que se lee de
-/// ellas (`Some(true)`: falta); `None` si no se sabe.
-fn falta_config(p: Proveedor, id: &str, r: &ReglasLeidas) -> Option<bool> {
+/// Si `doctor` lee de `leer-puertas` la configuracion de la garantia `id`
+/// en `p` (GitHub: los tipos de regla y los `parameters` de `pull_request`
+/// y `required_status_checks`; GitLab: la rama protegida y las reglas de
+/// aprobacion).
+pub fn lee_config(p: Proveedor, id: &str) -> bool {
+    matches!(
+        (p, id),
+        (Proveedor::GitHub, "G01" | "G04" | "G05" | "G09")
+            | (_, "G02" | "G03")
+            | (Proveedor::GitLab, "G06")
+    )
+}
+
+/// Lo que dicen las puertas de una garantia que se lee de ellas.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+enum Config {
+    /// Configurada.
+    Tiene,
+    /// No configurada.
+    Falta,
+    /// No se pudo leer, con el motivo.
+    NoLeida(Motivo),
+}
+
+/// La configuracion de `id` en las puertas leidas; `None` si `doctor` no la
+/// lee ([`lee_config`]) y la garantia sigue al plan.
+fn config(p: Proveedor, id: &str, puertas: Option<&ReglasLeidas>) -> Option<Config> {
+    if !lee_config(p, id) {
+        return None;
+    }
+    let Some(r) = puertas else {
+        return Some(Config::NoLeida(Motivo::PuertasNoLeidas));
+    };
     let tiene = |x: &str| r.reglas_rama.iter().any(|y| y == x);
-    match (p, id) {
+    let falta = match (p, id) {
         (Proveedor::GitHub, "G01") => Some(!(tiene("deletion") && tiene("non_fast_forward"))),
         (_, "G02") => Some(!super::plataforma::solo_pr(p, r)),
+        // GitLab: el 403 de las reglas de aprobacion. Si el plan las da
+        // (si no, G03 ya no esta impuesta), es un permiso, no la config.
+        (Proveedor::GitLab, "G03") if !r.plan_da_reglas => {
+            return Some(Config::NoLeida(Motivo::PermisoInsuficiente));
+        }
         (_, "G03") => Some(r.aprobaciones < 1),
+        (Proveedor::GitHub, "G04") => Some(!r.descartar_viejas),
+        (Proveedor::GitHub, "G05") => Some(!r.ultimo_push),
         (Proveedor::GitLab, "G06") => Some(!tiene("code-owner")),
         (Proveedor::GitHub, "G09") => r.checks_requeridos.as_ref().map(Vec::is_empty),
         _ => None,
-    }
+    };
+    Some(match falta {
+        Some(true) => Config::Falta,
+        Some(false) => Config::Tiene,
+        None => Config::NoLeida(Motivo::PuertasNoLeidas),
+    })
 }
 
 /// `capacidades` (F1-13 decision 3, funcion pura).
@@ -299,13 +353,17 @@ pub fn capacidades(p: Proveedor, plan: Plan, puertas: Option<&ReglasLeidas>) ->
                 // Plan de una plataforma y adaptador de otra: nada impuesto.
                 _ => g(EstadoGarantia::NoDisponible, Some(Motivo::Plataforma), None),
             };
-            match (base.estado, puertas.and_then(|r| falta_config(p, f.id, r))) {
-                (EstadoGarantia::Impuesta, Some(true)) => g(
+            if base.estado != EstadoGarantia::Impuesta {
+                return base;
+            }
+            match config(p, f.id, puertas) {
+                Some(Config::Falta) => g(
                     EstadoGarantia::NoDisponible,
                     Some(Motivo::Config),
                     Some(ARREGLO_INIT),
                 ),
-                _ => base,
+                Some(Config::NoLeida(m)) => g(EstadoGarantia::NoComprobada, Some(m), None),
+                Some(Config::Tiene) | None => base,
             }
         })
         .collect()
@@ -319,9 +377,14 @@ pub struct Cuentas {
     pub plan: String,
     /// `(impuestas, sustituidas, no-disponibles)`; `None` sin plan.
     pub cuentas: Option<(usize, usize, usize)>,
+    /// Garantias que el plan da y cuya configuracion no se pudo leer (no
+    /// cuentan como impuestas; `no-comprobadas=N` en la primera linea si
+    /// hay alguna).
+    pub no_comprobadas: usize,
 }
 
-/// `(impuestas, sustituidas, no-disponibles)`.
+/// `(impuestas, sustituidas, no-disponibles)`; las no comprobadas no
+/// cuentan en ninguna ([`no_comprobadas`]).
 pub fn contar(gs: &[Garantia]) -> (usize, usize, usize) {
     let n = |f: fn(&EstadoGarantia) -> bool| gs.iter().filter(|g| f(&g.estado)).count();
     (
@@ -331,8 +394,16 @@ pub fn contar(gs: &[Garantia]) -> (usize, usize, usize) {
     )
 }
 
-/// El hallazgo de una garantia: `ok` impuesta; si no, `aviso` con su
-/// motivo, el sustituto y el arreglo (F1-13 seccion C).
+/// Cuantas garantias salen `no-comprobado`.
+pub fn no_comprobadas(gs: &[Garantia]) -> usize {
+    gs.iter()
+        .filter(|g| g.estado == EstadoGarantia::NoComprobada)
+        .count()
+}
+
+/// El hallazgo de una garantia: `ok` impuesta; `no-comprobado` con su
+/// motivo si su configuracion no se leyo; si no, `aviso` con su motivo, el
+/// sustituto y el arreglo (F1-13 seccion C).
 pub fn hallazgo(g: &Garantia) -> Hallazgo {
     let h = Hallazgo::nuevo("garantia", g.id, Estado::Aviso);
     let h = match g.motivo {
@@ -343,6 +414,13 @@ pub fn hallazgo(g: &Garantia) -> Hallazgo {
         EstadoGarantia::Impuesta => {
             return Hallazgo::nuevo("garantia", g.id, Estado::Ok).con("capacidad", "impuesta");
         }
+        EstadoGarantia::NoComprobada => {
+            let x = Hallazgo::nuevo("garantia", g.id, Estado::NoComprobado);
+            return match g.motivo {
+                Some(m) => x.motivo(m.texto()),
+                None => x,
+            };
+        }
         EstadoGarantia::Sustituida(s) => h.con("capacidad", "sustituida").con("por", s),
         EstadoGarantia::NoDisponible => h.con("capacidad", "no-disponible"),
     };
@@ -369,6 +447,7 @@ pub fn de_lo_leido(
             Cuentas {
                 plan: "no-comprobado".to_owned(),
                 cuentas: None,
+                no_comprobadas: 0,
             },
         );
     };
@@ -378,6 +457,7 @@ pub fn de_lo_leido(
         Cuentas {
             plan: plan.nombre(),
             cuentas: Some(contar(&gs)),
+            no_comprobadas: no_comprobadas(&gs),
         },
     )
 }
@@ -401,6 +481,217 @@ mod tests {
             checks_requeridos: Some(vec!["verificacion".to_owned()]),
             todos_los_checks: p == Proveedor::GitLab,
             checks_publicados: None,
+            fallo_checks: None,
+            descartar_viejas: p == Proveedor::GitHub,
+            ultimo_push: p == Proveedor::GitHub,
+        }
+    }
+
+    const GH_PUBLICO: Plan = Plan::GitHub {
+        nombre: "github-free-publico-usuario",
+        rulesets: true,
+        cola: false,
+    };
+
+    /// Las lineas de las garantias que no son `ok`.
+    fn no_ok(gs: &[Garantia]) -> Vec<String> {
+        gs.iter()
+            .map(hallazgo)
+            .filter(|h| h.estado != Estado::Ok)
+            .map(|h| h.linea())
+            .collect()
+    }
+
+    /// H2 de TASK-035: cada garantia cuya configuracion se lee de las
+    /// puertas de GitHub sale `no-disponible motivo=config` cuando falta
+    /// (y solo ella); con todo configurado, ninguna. Mata las mutaciones
+    /// `Some(false)` de G01, G03 y G09 (y las de G04 y G05).
+    #[test]
+    fn config_de_github_garantia_a_garantia() {
+        let p = Proveedor::GitHub;
+        let r = completas(p, true);
+        let base = capacidades(p, GH_PUBLICO, Some(&r));
+        assert_eq!(
+            no_ok(&base),
+            [
+                "garantia G10 aviso motivo=plan capacidad=sustituida por=al-dia-por-cli arreglo=org-publica-o-enterprise-cloud"
+            ]
+        );
+        let config = |id: &str| {
+            format!("garantia {id} aviso motivo=config capacidad=no-disponible arreglo=tc2-init")
+        };
+        type Cambio = fn(&mut ReglasLeidas);
+        let casos: [(&str, Cambio); 6] = [
+            ("G01", |r| r.reglas_rama.retain(|x| x != "deletion")),
+            ("G01", |r| r.reglas_rama.retain(|x| x != "non_fast_forward")),
+            ("G03", |r| r.aprobaciones = 0),
+            ("G04", |r| r.descartar_viejas = false),
+            ("G05", |r| r.ultimo_push = false),
+            ("G09", |r| r.checks_requeridos = Some(Vec::new())),
+        ];
+        for (id, cambiar) in casos {
+            let mut r = completas(p, true);
+            cambiar(&mut r);
+            let gs = capacidades(p, GH_PUBLICO, Some(&r));
+            let l = no_ok(&gs);
+            assert!(l.contains(&config(id)), "{id}: {l:?}");
+            assert_eq!(l.len(), 2, "{id}: {l:?}");
+            assert_eq!(contar(&gs), (18, 1, 1), "{id}");
+        }
+    }
+
+    /// H1 de TASK-035: sin puertas leidas, las garantias cuya configuracion
+    /// `doctor` lee salen `no-comprobado motivo=puertas-no-leidas` y no
+    /// cuentan como impuestas; las demas siguen al plan.
+    #[test]
+    fn sin_puertas_lo_que_se_lee_no_esta_comprobado() {
+        let gs = capacidades(Proveedor::GitHub, GH_PUBLICO, None);
+        let nc: Vec<&str> = gs
+            .iter()
+            .filter(|g| g.estado == EstadoGarantia::NoComprobada)
+            .map(|g| g.id)
+            .collect();
+        assert_eq!(nc, ["G01", "G02", "G03", "G04", "G05", "G09"]);
+        assert_eq!(contar(&gs), (13, 1, 0));
+        assert_eq!(no_comprobadas(&gs), 6);
+        assert_eq!(
+            hallazgo(&gs[0]).linea(),
+            "garantia G01 no-comprobado motivo=puertas-no-leidas"
+        );
+        let gl = Plan::GitLab(TierGitLab::Premium);
+        let gs = capacidades(Proveedor::GitLab, gl, None);
+        let nc: Vec<&str> = gs
+            .iter()
+            .filter(|g| g.estado == EstadoGarantia::NoComprobada)
+            .map(|g| g.id)
+            .collect();
+        assert_eq!(nc, ["G02", "G03", "G06"]);
+        assert_eq!((contar(&gs), no_comprobadas(&gs)), ((14, 3, 0), 3));
+        // GitHub sin `checks_requeridos` leidos: G09 tampoco se da.
+        let mut r = completas(Proveedor::GitHub, true);
+        r.checks_requeridos = None;
+        let gs = capacidades(Proveedor::GitHub, GH_PUBLICO, Some(&r));
+        assert_eq!(
+            hallazgo(&gs[8]).linea(),
+            "garantia G09 no-comprobado motivo=puertas-no-leidas"
+        );
+        // La primera linea las cuenta aparte.
+        let (_, c) = de_lo_leido(
+            Some(Proveedor::GitLab),
+            Some(&PlanObservado::GitLab(Some(TierGitLab::Premium))),
+            None,
+        );
+        assert_eq!((c.cuentas, c.no_comprobadas), (Some((14, 3, 0)), 3));
+    }
+
+    /// H4 de TASK-035: el 403 de las reglas de aprobacion de GitLab con un
+    /// plan legible que las da no es config ausente: `no-comprobado
+    /// motivo=permiso-insuficiente`. En Free, G03 sigue sustituida.
+    #[test]
+    fn el_403_de_aprobaciones_con_premium_es_un_permiso() {
+        let r = completas(Proveedor::GitLab, false);
+        let gs = capacidades(
+            Proveedor::GitLab,
+            Plan::GitLab(TierGitLab::Premium),
+            Some(&r),
+        );
+        assert_eq!(
+            hallazgo(&gs[2]).linea(),
+            "garantia G03 no-comprobado motivo=permiso-insuficiente"
+        );
+        assert_eq!((contar(&gs), no_comprobadas(&gs)), ((16, 3, 0), 1));
+        let gs = capacidades(Proveedor::GitLab, Plan::GitLab(TierGitLab::Free), Some(&r));
+        assert_eq!(
+            gs[2].estado,
+            EstadoGarantia::Sustituida("merge-solo-persona")
+        );
+    }
+
+    /// Tier de GitHub de una celda de la tabla de F1-13.
+    fn tier_github(c: &str) -> Option<TierGitHub> {
+        match c {
+            "Free público; Pro/Team/Enterprise" => Some(TierGitHub::Rulesets),
+            "org pública; Enterprise Cloud" => Some(TierGitHub::OrgOEnterprise),
+            "todos" => Some(TierGitHub::Todos),
+            _ => None,
+        }
+    }
+
+    /// Tier de GitLab de una celda.
+    fn tier_gitlab(c: &str) -> Option<TierGitLabFila> {
+        match c {
+            "Free" => Some(TierGitLabFila::Free),
+            "Premium" => Some(TierGitLabFila::Premium),
+            "ninguno" => Some(TierGitLabFila::Ninguno),
+            _ => None,
+        }
+    }
+
+    /// El nombre corto de un sustituto (`—`: ninguno; si no, el primer
+    /// texto entre comillas invertidas).
+    fn sustituto(c: &str) -> Option<String> {
+        let c = c.trim();
+        if c == "—" {
+            return None;
+        }
+        c.strip_prefix('`')
+            .and_then(|r| r.split('`').next())
+            .map(str::to_owned)
+            .or_else(|| Some(format!("ilegible:{c}")))
+    }
+
+    /// R1-01 de TASK-035: la tabla compilada es la del bloque
+    /// `tabla-equivalencias` de F1-13, fila a fila (tier GitHub, tier GitLab
+    /// y los dos sustitutos).
+    #[test]
+    fn la_tabla_compilada_es_la_de_f1_13() {
+        let ruta = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
+            .join("../../docs/adr/F1-13-adaptador-de-plataforma.md");
+        let texto = std::fs::read_to_string(&ruta).unwrap_or_default();
+        let bloque = texto
+            .split("<!-- tabla-equivalencias:inicio -->")
+            .nth(1)
+            .and_then(|r| r.split("<!-- tabla-equivalencias:fin -->").next())
+            .unwrap_or_default();
+        type FilaAdr = (
+            String,
+            Option<TierGitHub>,
+            Option<String>,
+            Option<TierGitLabFila>,
+            Option<String>,
+        );
+        let adr: Vec<FilaAdr> = bloque
+            .lines()
+            .map(str::trim)
+            .filter(|l| l.starts_with("| G"))
+            .map(|l| {
+                let c: Vec<&str> = l.split('|').map(str::trim).collect();
+                assert_eq!(c.len(), 11, "{l}");
+                let (gh, gl) = c[8].split_once(" / GitLab: ").unwrap_or(("", ""));
+                (
+                    c[1].to_owned(),
+                    tier_github(c[5]),
+                    sustituto(gh.strip_prefix("GitHub: ").unwrap_or("?")),
+                    tier_gitlab(c[7]),
+                    sustituto(gl),
+                )
+            })
+            .collect();
+        let compilada: Vec<FilaAdr> = TABLA
+            .iter()
+            .map(|f| {
+                (
+                    f.id.to_owned(),
+                    Some(f.github),
+                    f.sustituto_github.map(str::to_owned),
+                    Some(f.gitlab),
+                    f.sustituto_gitlab.map(str::to_owned),
+                )
+            })
+            .collect();
+        assert_eq!(adr.len(), 20, "{}", ruta.display());
+        for (a, c) in adr.iter().zip(&compilada) {
+            assert_eq!(a, c);
         }
     }
 
diff --git a/crates/tc2/src/doctor/estado.rs b/crates/tc2/src/doctor/estado.rs
index cd90aad..fd3f0f4 100644
--- a/crates/tc2/src/doctor/estado.rs
+++ b/crates/tc2/src/doctor/estado.rs
@@ -9,32 +9,26 @@
 //!   motivo=sin-token` y los mira [`super::plataforma`]). Cada accion que
 //!   no sale `existe` es un hallazgo, asi `doctor` e `init` no discrepan.
 
-use std::path::Path;
+use std::path::{Path, PathBuf};
 
 use super::{ARREGLO_INIT, Estado, Hallazgo, NOMBRE};
 use crate::contrato::corto;
-use crate::estado::{self, RAMA};
-use crate::git::estado::hay_remoto;
-use crate::git::{Git, REMOTO, Subcomando, correr, localizar_principal};
+use crate::estado;
+use crate::git::{Git, Subcomando, correr};
 use crate::init::{self, Accion, Decision};
 use crate::plataforma::{Credenciales, CredencialesFijas};
 
-/// La ref de la rama de estado que lee `board`: la remota si hay `origin`.
-fn referencia(g: &Git, dir: &Path) -> Option<String> {
-    let principal = localizar_principal(g, dir).ok()?;
-    let remoto = hay_remoto(g, &principal, NOMBRE).ok()?;
-    Some(if remoto {
-        format!("refs/remotes/{REMOTO}/{RAMA}")
-    } else {
-        format!("refs/heads/{RAMA}")
-    })
+/// La ref de la rama de estado que lee `board` (la misma que
+/// [`estado::leer`]) y la carpeta principal.
+fn referencia(g: &Git, dir: &Path) -> Option<(PathBuf, String)> {
+    estado::referencia_de_lectura(g, dir).ok()
 }
 
 /// Si `r` existe (apunte o no a un commit).
-fn existe_ref(g: &Git, dir: &Path, r: &str) -> bool {
+fn existe_ref(g: &Git, principal: &Path, r: &str) -> bool {
     correr(
         g,
-        dir,
+        principal,
         NOMBRE,
         Subcomando::RevParse,
         &["-q", "--verify", "--end-of-options", r],
@@ -45,11 +39,11 @@ fn existe_ref(g: &Git, dir: &Path, r: &str) -> bool {
 /// Hallazgos de la rama de estado y el commit leido (12).
 pub(super) fn rama(g: &Git, dir: &Path) -> (Vec<Hallazgo>, Option<String>) {
     let h = |e: Estado| Hallazgo::nuevo("estado", "rama", e);
-    let r = referencia(g, dir).unwrap_or_default();
+    let (principal, r) = referencia(g, dir).unwrap_or_else(|| (dir.to_path_buf(), String::new()));
     let vista = match estado::leer(g, dir) {
         Ok(v) => v,
         Err(i) if i.valor("motivo") == Some("sin-rama-de-estado") => {
-            let x = if existe_ref(g, dir, &r) {
+            let x = if existe_ref(g, &principal, &r) {
                 h(Estado::Falla).motivo("ref-ilegible").con("ref", r)
             } else {
                 h(Estado::Falla)
diff --git a/crates/tc2/src/doctor/mod.rs b/crates/tc2/src/doctor/mod.rs
index 10155db..fe0f6a7 100644
--- a/crates/tc2/src/doctor/mod.rs
+++ b/crates/tc2/src/doctor/mod.rs
@@ -10,7 +10,8 @@
 //! Salida (F1-4 decision 4, F1-13 seccion C): como mucho 5 lineas en
 //! stdout. La primera, `doctor <ok|rechazada> fallos=N avisos=N
 //! no-comprobado=N plataforma=... [plan=... impuestas=N sustituidas=N
-//! no-disponibles=N] [commit=...]`; detras, los hallazgos que no son `ok`,
+//! no-disponibles=N [no-comprobadas=N]] [commit=...]`; detras, los
+//! hallazgos que no son `ok`,
 //! los peores primero, y si no caben, `y N mas: tc2 doctor --json`. Con
 //! `--json`, una linea con todos los hallazgos (tambien los `ok`). Codigo 3
 //! con alguna `falla` (D5a); 0 con avisos o no-comprobados.
@@ -210,6 +211,9 @@ pub fn informe(hs: &[Hallazgo], r: &Resumen) -> Informe {
                 .con("impuestas", imp.to_string())
                 .con("sustituidas", sus.to_string())
                 .con("no-disponibles", nod.to_string());
+            if c.no_comprobadas > 0 {
+                i = i.con("no-comprobadas", c.no_comprobadas.to_string());
+            }
         }
     }
     if let Some(c) = &r.commit {
diff --git a/crates/tc2/src/doctor/plataforma.rs b/crates/tc2/src/doctor/plataforma.rs
index f4fbc01..1203e70 100644
--- a/crates/tc2/src/doctor/plataforma.rs
+++ b/crates/tc2/src/doctor/plataforma.rs
@@ -90,8 +90,11 @@ pub fn checks_del_modelo() -> Vec<&'static str> {
 /// del modelo que la rama no exige, `falla motivo=check-no-obligatorio`;
 /// uno exigido que no esta entre los publicados en su cabeza, `falla
 /// motivo=check-sin-publicar`, o `aviso` si no hay ninguno publicado (repo
-/// recien creado, sin el workflow de F5-1). Solo se mira el nombre: el
-/// origen (`integration_id`) espera a F6-1.
+/// recien creado, sin el workflow de F5-1). Un check exigido que no es del
+/// modelo y no esta publicado es `aviso` (D4a habla de los del modelo). Si
+/// los publicados no se leyeron, `no-comprobado` con el motivo (y
+/// `permiso=` en un 403). Solo se mira el nombre: el origen
+/// (`integration_id`) espera a F6-1.
 pub fn checks(p: Option<Proveedor>, leidas: &Paso<ReglasLeidas>) -> Vec<Hallazgo> {
     let modelo = checks_del_modelo();
     let h = |c: &str, e: Estado| Hallazgo::nuevo("check", c, e).con("rama", RAMA_CHECKS);
@@ -141,11 +144,23 @@ pub fn checks(p: Option<Proveedor>, leidas: &Paso<ReglasLeidas>) -> Vec<Hallazgo
                     .motivo("check-no-obligatorio")
                     .arreglo(ARREGLO_INIT);
             }
-            match &r.checks_publicados {
-                None => h(c, Estado::NoComprobado).motivo("checks-no-leidos"),
-                Some(v) if v.iter().any(|x| x == c) => h(c, Estado::Ok),
-                Some(v) if v.is_empty() => h(c, Estado::Aviso).motivo("check-sin-publicar"),
-                Some(_) => h(c, Estado::Falla).motivo("check-sin-publicar"),
+            match (&r.checks_publicados, &r.fallo_checks) {
+                (None, Some(f)) => {
+                    let x = h(c, Estado::NoComprobado).motivo(f.motivo.clone());
+                    match f.permiso {
+                        Some(p) => x.con("permiso", p),
+                        None => x,
+                    }
+                }
+                (None, None) => h(c, Estado::NoComprobado).motivo("checks-no-leidos"),
+                (Some(v), _) if v.iter().any(|x| x == c) => h(c, Estado::Ok),
+                (Some(v), _) if v.is_empty() => h(c, Estado::Aviso).motivo("check-sin-publicar"),
+                // D4a habla de los checks del modelo: uno exigido que el
+                // modelo no conoce y no esta publicado es `aviso` (R1-04).
+                (Some(_), _) if !modelo.contains(&c) => {
+                    h(c, Estado::Aviso).motivo("check-sin-publicar")
+                }
+                (Some(_), _) => h(c, Estado::Falla).motivo("check-sin-publicar"),
             }
         })
         .collect()
@@ -239,22 +254,20 @@ pub(super) fn comprobar(
     let p: &dyn Plataforma = c.plataforma.as_ref();
     let prov = p.proveedor();
     let ramas = ramas_solo_pr();
-    let mut leidas: Vec<Paso<ReglasLeidas>> = Vec::new();
-    for (_, rama) in &ramas {
-        let r = p.leer_puertas(c.rol, rama);
-        // Sin red o sin credenciales en la primera lectura no se sigue: lo
-        // de la plataforma queda `no-comprobado`.
-        if let Err(i) = &r
-            && leidas.is_empty()
-            && matches!(
-                Consulta::de_fallo(i),
-                Consulta::SinRed | Consulta::SinCredenciales
-            )
-        {
-            let n = Consulta::de_fallo(i).nombre();
-            return (sin_plataforma(n), n.to_owned(), None);
-        }
-        leidas.push(r);
+    // Lo comun al repo se lee una vez y los checks publicados solo en la
+    // rama de los checks (R1-06 de TASK-035).
+    let nombres: Vec<&str> = ramas.iter().map(|(_, r)| r.as_str()).collect();
+    let leidas: Vec<Paso<ReglasLeidas>> = p.leer_puertas_de(c.rol, &nombres, RAMA_CHECKS);
+    // Sin red o sin credenciales en la primera lectura no se sigue: lo de
+    // la plataforma queda `no-comprobado`.
+    if let Some(Err(i)) = leidas.first()
+        && matches!(
+            Consulta::de_fallo(i),
+            Consulta::SinRed | Consulta::SinCredenciales
+        )
+    {
+        let n = Consulta::de_fallo(i).nombre();
+        return (sin_plataforma(n), n.to_owned(), None);
     }
     let mut hs = Vec::new();
     let mut plataforma = "ok";
@@ -294,3 +307,74 @@ pub(super) fn comprobar(
     hs.extend(h);
     (hs, plataforma.to_owned(), Some(cuentas))
 }
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::plataforma::FalloLectura;
+
+    fn leidas(requeridos: &[&str], publicados: Option<&[&str]>) -> ReglasLeidas {
+        let v = |x: &[&str]| x.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
+        ReglasLeidas {
+            plan_da_reglas: true,
+            checks_requeridos: Some(v(requeridos)),
+            checks_publicados: publicados.map(v),
+            ..ReglasLeidas::default()
+        }
+    }
+
+    fn lineas(r: ReglasLeidas) -> Vec<String> {
+        checks(Some(Proveedor::GitHub), &Ok(r))
+            .iter()
+            .map(Hallazgo::linea)
+            .collect()
+    }
+
+    /// R1-04 de TASK-035: un check exigido que el modelo no conoce y no
+    /// esta publicado es `aviso` (D4a habla de los del modelo); uno del
+    /// modelo en el mismo caso sigue siendo `falla`.
+    #[test]
+    fn check_ajeno_al_modelo_sin_publicar_es_aviso() {
+        let modelo = checks_del_modelo();
+        let mut req = modelo.clone();
+        req.push("ajeno");
+        let l = lineas(leidas(&req, Some(&modelo[..2])));
+        assert_eq!(
+            l,
+            [
+                "check verificacion ok rama=develop",
+                "check revision-ia ok rama=develop",
+                "check carril falla motivo=check-sin-publicar rama=develop",
+                "check ajeno aviso motivo=check-sin-publicar rama=develop",
+            ]
+        );
+    }
+
+    /// H3 de TASK-035: si los checks publicados no se leyeron, se dice por
+    /// que: un 403 con su permiso; otro fallo, su motivo.
+    #[test]
+    fn checks_no_leidos_dicen_por_que() {
+        let modelo = checks_del_modelo();
+        let mut r = leidas(&modelo, None);
+        r.fallo_checks = Some(FalloLectura {
+            motivo: "permiso-insuficiente".to_owned(),
+            permiso: Some("checks:read"),
+        });
+        assert_eq!(
+            lineas(r.clone())[0],
+            "check verificacion no-comprobado motivo=permiso-insuficiente rama=develop permiso=checks:read"
+        );
+        r.fallo_checks = Some(FalloLectura::motivo("respuesta-invalida"));
+        assert_eq!(
+            lineas(r)[0],
+            "check verificacion no-comprobado motivo=respuesta-invalida rama=develop"
+        );
+        let i = Informe::nuevo("leer-puertas", Resultado::Error).con("motivo", "http");
+        assert_eq!(
+            FalloLectura::de(&i, "checks:read"),
+            FalloLectura::motivo("http")
+        );
+        let i = Informe::nuevo("leer-puertas", Resultado::Denegada).con("motivo", "permisos");
+        assert_eq!(FalloLectura::de(&i, "read_api").permiso, Some("read_api"));
+    }
+}
diff --git a/crates/tc2/src/estado/lectura.rs b/crates/tc2/src/estado/lectura.rs
index 6ec3d1e..b366395 100644
--- a/crates/tc2/src/estado/lectura.rs
+++ b/crates/tc2/src/estado/lectura.rs
@@ -139,16 +139,24 @@ impl Estado for Vista<'_> {
     }
 }
 
-/// Vista del estado tal como lo conoce este clon: lo ultimo traido de
-/// `origin/taskcodetwo/tareas` (sin `fetch`: la lectura fresca es de F3-5b) o,
-/// en un repo sin remoto, la rama local. Sin rama de estado → `rechazada`.
-pub fn leer<'a>(git: &'a Git, dir: &Path) -> Result<Vista<'a>, Informe> {
+/// La carpeta principal y la ref de la rama de estado que lee [`leer`]: la
+/// remota si hay `origin`, la local si no. `doctor` (F3-8) decide sobre la
+/// misma ref.
+pub(crate) fn referencia_de_lectura(git: &Git, dir: &Path) -> Result<(PathBuf, String), Informe> {
     let principal = localizar_principal(git, dir)?;
     let referencia = if hay_remoto(git, &principal, OP)? {
         format!("refs/remotes/{REMOTO}/{RAMA}")
     } else {
         format!("refs/heads/{RAMA}")
     };
+    Ok((principal, referencia))
+}
+
+/// Vista del estado tal como lo conoce este clon: lo ultimo traido de
+/// `origin/taskcodetwo/tareas` (sin `fetch`: la lectura fresca es de F3-5b) o,
+/// en un repo sin remoto, la rama local. Sin rama de estado → `rechazada`.
+pub fn leer<'a>(git: &'a Git, dir: &Path) -> Result<Vista<'a>, Informe> {
+    let (principal, referencia) = referencia_de_lectura(git, dir)?;
     match crate::git::commit_de(git, &principal, OP, &referencia)? {
         Some(c) => Vista::nueva(git, &principal, Some(c)),
         None => Err(Informe::nuevo(OP, Resultado::Rechazada)
diff --git a/crates/tc2/src/estado/mod.rs b/crates/tc2/src/estado/mod.rs
index 73b2ad6..0ac0470 100644
--- a/crates/tc2/src/estado/mod.rs
+++ b/crates/tc2/src/estado/mod.rs
@@ -29,6 +29,7 @@ mod transaccion;
 
 pub use crate::git::Cambio;
 pub use cerrojo::Cerrojo;
+pub(crate) use lectura::referencia_de_lectura;
 pub use lectura::{Estado, Vista, leer};
 pub use montaje::{Previsto, montar, montar_con, previsto};
 pub use tareas::{Leida, de_otra_persona, es_tarea_md, por_estado, tarea_de, tareas};
diff --git a/crates/tc2/src/plataforma/github.rs b/crates/tc2/src/plataforma/github.rs
index 91ac519..fbc12ce 100644
--- a/crates/tc2/src/plataforma/github.rs
+++ b/crates/tc2/src/plataforma/github.rs
@@ -17,8 +17,8 @@ use super::errores::{self, es_aviso_de_plan};
 use super::http::{Base, Metodo, codificar};
 use super::puertas::{self, EstadoCheck, Lecturas};
 use super::{
-    Firma, ModeloPuertas, Paso, Personas, PlanObservado, Plataforma, PrAbierto, Proveedor,
-    ReglasLeidas, Usuario,
+    FalloLectura, Firma, ModeloPuertas, Paso, Personas, PlanObservado, Plataforma, PrAbierto,
+    Proveedor, ReglasLeidas, Usuario,
 };
 use crate::contrato::{Informe, Resultado, Valor, corto};
 use crate::i18n::catalogo::plataforma as t;
@@ -35,6 +35,12 @@ const LEER_PLAN: &str = "leer-plan";
 /// Permiso de `leer-puertas` (lo nombra `doctor` en `permiso=`).
 pub const PERMISO_LEER_PUERTAS: &str = "metadata:read";
 
+/// Permiso de los check-runs que lee `leer-puertas` (GitHub App:
+/// «Repository permissions for "Checks"» read,
+/// <https://docs.github.com/en/rest/authentication/permissions-required-for-github-apps>,
+/// 2026-11-17).
+pub const PERMISO_CHECKS: &str = "checks:read";
+
 /// Permiso de `leer-permisos-ci`.
 pub const PERMISO_PERMISOS_CI: &str = "administration:read";
 
@@ -254,12 +260,21 @@ struct ReglaApi {
     parameters: Option<ParametrosApi>,
 }
 
+/// Los `parameters` de las reglas `pull_request` y
+/// `required_status_checks` (esquema de rulesets,
+/// <https://docs.github.com/en/rest/repos/rules>, 2026-11-17: los cinco
+/// booleanos y el contador de `pull_request` son obligatorios, pero un
+/// `null` no tumba la lectura: H5 de TASK-035).
 #[derive(Deserialize, Default)]
 struct ParametrosApi {
     #[serde(default)]
     required_approving_review_count: Option<u64>,
-    #[serde(default)]
+    #[serde(default, deserialize_with = "super::nulo_por_defecto")]
     required_status_checks: Vec<CheckRequeridoApi>,
+    #[serde(default)]
+    dismiss_stale_reviews_on_push: Option<bool>,
+    #[serde(default)]
+    require_last_push_approval: Option<bool>,
 }
 
 #[derive(Deserialize)]
@@ -614,66 +629,40 @@ impl Plataforma for GitHub {
     }
 
     fn leer_puertas(&self, rol: Rol, rama: &str) -> Paso<ReglasLeidas> {
-        if !rama_valida(rama) {
-            return Err(uso(LEER_PUERTAS, "rama-invalida", &t::RAMA_NO_VALIDA));
-        }
-        if rol == Rol::Revisor {
-            return Err(rol_no_valido(LEER_PUERTAS, rol));
-        }
-        let permiso = PERMISO_LEER_PUERTAS;
-        let r = self
-            .con
-            .pedir(LEER_PUERTAS, rol, self.ep(&Endpoint::Rulesets), None)?;
-        if es_aviso_de_plan(&r) {
-            return Ok(ReglasLeidas {
-                plan_da_reglas: false,
-                ..ReglasLeidas::default()
-            });
-        }
-        if !r.exito() {
-            return Err(self.con.traducir(LEER_PUERTAS, rol, permiso, &r));
+        validar_puertas(rol, rama)?;
+        match self.rulesets(rol)? {
+            Some(rulesets) => self.puertas_de_rama(rol, rama, &rulesets, true),
+            None => Ok(sin_reglas_por_plan()),
         }
-        let rulesets: Vec<RulesetApi> = self.con.parsear(LEER_PUERTAS, &r.cuerpo)?;
-        let reglas: Vec<ReglaApi> = self.con.leer(
-            LEER_PUERTAS,
-            rol,
-            permiso,
-            self.ep(&Endpoint::ReglasRama(rama)),
-        )?;
-        let parametros = |tipo: &'static str| {
-            reglas
-                .iter()
-                .filter(move |r| r.tipo == tipo)
-                .filter_map(|r| r.parameters.as_ref())
+    }
+
+    fn leer_puertas_de(
+        &self,
+        rol: Rol,
+        ramas: &[&str],
+        checks_de: &str,
+    ) -> Vec<Paso<ReglasLeidas>> {
+        // Los rulesets son del repo: se leen una vez (solo si alguna rama
+        // es valida) y valen para todas.
+        let validas: Vec<Paso<()>> = ramas.iter().map(|r| validar_puertas(rol, r)).collect();
+        let comun = if validas.iter().any(Result::is_ok) {
+            Some(self.rulesets(rol))
+        } else {
+            None
         };
-        let aprobaciones = parametros("pull_request")
-            .filter_map(|p| p.required_approving_review_count)
-            .max()
-            .unwrap_or(0);
-        let requeridos: Vec<String> = parametros("required_status_checks")
-            .flat_map(|p| p.required_status_checks.iter().map(|c| c.context.clone()))
-            .collect();
-        // Los check-runs de la cabeza de la rama (`heads/<rama>`, la forma
-        // que documenta la API). Si no se pueden leer, no se supone nada.
-        let (_, ruta) = self.ep(&Endpoint::CheckRunsDeRama(rama));
-        let publicados = self
-            .con
-            .leer_paginas(LEER_PUERTAS, rol, permiso, ruta, |c| {
-                self.con
-                    .parsear::<CheckRunsApi>(LEER_PUERTAS, c)
-                    .map(|x| x.check_runs)
+        ramas
+            .iter()
+            .zip(validas)
+            .map(|(rama, v)| {
+                v?;
+                match comun.as_ref() {
+                    Some(Ok(Some(rs))) => self.puertas_de_rama(rol, rama, rs, *rama == checks_de),
+                    Some(Ok(None)) => Ok(sin_reglas_por_plan()),
+                    Some(Err(i)) => Err(i.clone()),
+                    None => Err(rol_no_valido(LEER_PUERTAS, rol)),
+                }
             })
-            .ok()
-            .map(|v| v.into_iter().map(|r| r.name).collect());
-        Ok(ReglasLeidas {
-            plan_da_reglas: true,
-            rulesets: rulesets.into_iter().map(|r| r.name).collect(),
-            reglas_rama: reglas.into_iter().map(|r| r.tipo).collect(),
-            aprobaciones,
-            checks_requeridos: Some(requeridos),
-            todos_los_checks: false,
-            checks_publicados: publicados,
-        })
+            .collect()
     }
 
     fn leer_permisos_ci(&self) -> Informe {
@@ -755,6 +744,108 @@ impl Plataforma for GitHub {
     }
 }
 
+/// Rama y rol de `leer-puertas`.
+fn validar_puertas(rol: Rol, rama: &str) -> Paso<()> {
+    if !rama_valida(rama) {
+        return Err(uso(LEER_PUERTAS, "rama-invalida", &t::RAMA_NO_VALIDA));
+    }
+    if rol == Rol::Revisor {
+        return Err(rol_no_valido(LEER_PUERTAS, rol));
+    }
+    Ok(())
+}
+
+/// Lo leido cuando el plan no da los rulesets (403 con aviso de plan).
+fn sin_reglas_por_plan() -> ReglasLeidas {
+    ReglasLeidas {
+        plan_da_reglas: false,
+        ..ReglasLeidas::default()
+    }
+}
+
+impl GitHub {
+    /// Los nombres de los rulesets del repo; `None` si el plan no los da.
+    fn rulesets(&self, rol: Rol) -> Paso<Option<Vec<String>>> {
+        let r = self
+            .con
+            .pedir(LEER_PUERTAS, rol, self.ep(&Endpoint::Rulesets), None)?;
+        if es_aviso_de_plan(&r) {
+            return Ok(None);
+        }
+        if !r.exito() {
+            return Err(self
+                .con
+                .traducir(LEER_PUERTAS, rol, PERMISO_LEER_PUERTAS, &r));
+        }
+        let rulesets: Vec<RulesetApi> = self.con.parsear(LEER_PUERTAS, &r.cuerpo)?;
+        Ok(Some(rulesets.into_iter().map(|r| r.name).collect()))
+    }
+
+    /// Las reglas de `rama` con los `rulesets` ya leidos; los check-runs de
+    /// su cabeza solo `con_checks`.
+    fn puertas_de_rama(
+        &self,
+        rol: Rol,
+        rama: &str,
+        rulesets: &[String],
+        con_checks: bool,
+    ) -> Paso<ReglasLeidas> {
+        let reglas: Vec<ReglaApi> = self.con.leer(
+            LEER_PUERTAS,
+            rol,
+            PERMISO_LEER_PUERTAS,
+            self.ep(&Endpoint::ReglasRama(rama)),
+        )?;
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
+        let descartar_viejas =
+            parametros("pull_request").any(|p| p.dismiss_stale_reviews_on_push == Some(true));
+        let ultimo_push =
+            parametros("pull_request").any(|p| p.require_last_push_approval == Some(true));
+        let requeridos: Vec<String> = parametros("required_status_checks")
+            .flat_map(|p| p.required_status_checks.iter().map(|c| c.context.clone()))
+            .collect();
+        // Los check-runs de la cabeza de la rama (`heads/<rama>`, la forma
+        // que documenta la API). Si no se pueden leer, no se supone nada y
+        // se dice por que (H3 de TASK-035).
+        let (publicados, fallo_checks) = if con_checks {
+            let (_, ruta) = self.ep(&Endpoint::CheckRunsDeRama(rama));
+            match self
+                .con
+                .leer_paginas(LEER_PUERTAS, rol, PERMISO_CHECKS, ruta, |c| {
+                    self.con
+                        .parsear::<CheckRunsApi>(LEER_PUERTAS, c)
+                        .map(|x| x.check_runs)
+                }) {
+                Ok(v) => (Some(v.into_iter().map(|r| r.name).collect()), None),
+                Err(i) => (None, Some(FalloLectura::de(&i, PERMISO_CHECKS))),
+            }
+        } else {
+            (None, None)
+        };
+        Ok(ReglasLeidas {
+            plan_da_reglas: true,
+            rulesets: rulesets.to_vec(),
+            reglas_rama: reglas.into_iter().map(|r| r.tipo).collect(),
+            aprobaciones,
+            checks_requeridos: Some(requeridos),
+            todos_los_checks: false,
+            checks_publicados: publicados,
+            fallo_checks,
+            descartar_viejas,
+            ultimo_push,
+        })
+    }
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
diff --git a/crates/tc2/src/plataforma/gitlab.rs b/crates/tc2/src/plataforma/gitlab.rs
index 02b8a77..f74bf22 100644
--- a/crates/tc2/src/plataforma/gitlab.rs
+++ b/crates/tc2/src/plataforma/gitlab.rs
@@ -18,8 +18,8 @@ use super::errores;
 use super::http::{Base, Metodo, codificar};
 use super::puertas::{self, EstadoCheck, Lecturas};
 use super::{
-    Clave, Firma, ModeloPuertas, Paso, Personas, PlanObservado, Plataforma, PrAbierto, Proveedor,
-    ReglasLeidas, TierGitLab, Usuario,
+    Clave, FalloLectura, Firma, ModeloPuertas, Paso, Personas, PlanObservado, Plataforma,
+    PrAbierto, Proveedor, ReglasLeidas, TierGitLab, Usuario,
 };
 use crate::contrato::{Informe, Resultado, Valor, corto};
 use crate::i18n::catalogo::plataforma as t;
@@ -206,10 +206,10 @@ struct NivelApi {
 struct RamaProtegidaApi {
     name: String,
     merge_access_levels: Vec<NivelApi>,
-    #[serde(default)]
+    #[serde(default, deserialize_with = "super::nulo_por_defecto")]
     push_access_levels: Vec<NivelApi>,
     allow_force_push: bool,
-    #[serde(default)]
+    #[serde(default, deserialize_with = "super::nulo_por_defecto")]
     code_owner_approval_required: bool,
 }
 
@@ -239,7 +239,9 @@ struct AprobadorApi {
 #[derive(Deserialize)]
 struct ReglaAprobacionApi {
     name: String,
-    #[serde(default)]
+    /// `null` cuenta como 0 (no pide ninguna): lo conservador, `doctor` la
+    /// da por no configurada (H5 de TASK-035).
+    #[serde(default, deserialize_with = "super::nulo_por_defecto")]
     approvals_required: u64,
 }
 
@@ -377,8 +379,10 @@ impl GitLab {
     }
 
     /// Nombres de los estados de la cabeza de `rama` (la API de estados
-    /// pide el sha: se lee antes la rama). `None` si algo no se lee.
-    fn checks_publicados(&self, rol: Rol, rama: &str) -> Option<Vec<String>> {
+    /// pide el sha: se lee antes la rama). Si algo no se lee, el motivo
+    /// (un 403, con su permiso: H3 de TASK-035).
+    fn checks_publicados(&self, rol: Rol, rama: &str) -> Result<Vec<String>, FalloLectura> {
+        let fallo = |i: Informe| FalloLectura::de(&i, PERMISO_LECTURA_CORTO);
         let r: RamaApi = self
             .con
             .leer(
@@ -387,9 +391,9 @@ impl GitLab {
                 PERMISO_LECTURA,
                 self.ep(&Endpoint::Rama(rama)),
             )
-            .ok()?;
+            .map_err(fallo)?;
         if !sha_valido(&r.commit.id) {
-            return None;
+            return Err(FalloLectura::motivo("respuesta-invalida"));
         }
         let (_, ruta) = self.ep(&Endpoint::Estados(&r.commit.id));
         let estados: Vec<EstadoApi> = self
@@ -397,8 +401,8 @@ impl GitLab {
             .leer_paginas(LEER_PUERTAS, rol, PERMISO_LECTURA, ruta, |c| {
                 self.con.parsear(LEER_PUERTAS, c)
             })
-            .ok()?;
-        Some(estados.into_iter().map(|e| e.name).collect())
+            .map_err(fallo)?;
+        Ok(estados.into_iter().map(|e| e.name).collect())
     }
 
     /// Si `rama` tiene commits que no estan en `base`.
@@ -777,65 +781,37 @@ impl Plataforma for GitLab {
     }
 
     fn leer_puertas(&self, rol: Rol, rama: &str) -> Paso<ReglasLeidas> {
-        if !rama_valida(rama) {
-            return Err(uso(LEER_PUERTAS, "rama-invalida", &t::RAMA_NO_VALIDA));
-        }
-        if rol == Rol::Revisor {
-            return Err(rol_no_valido(LEER_PUERTAS, rol));
-        }
-        let (_, ruta) = self.ep(&Endpoint::RamasProtegidas);
-        let ramas: Vec<RamaProtegidaApi> =
-            self.con
-                .leer_paginas(LEER_PUERTAS, rol, PERMISO_LECTURA, ruta, |c| {
-                    self.con.parsear(LEER_PUERTAS, c)
-                })?;
-        let mut reglas_rama = Vec::new();
-        if let Some(p) = ramas.iter().find(|p| p.name == rama) {
-            reglas_rama.extend(
-                p.push_access_levels
-                    .iter()
-                    .map(|n| format!("push={}", n.access_level)),
-            );
-            reglas_rama.extend(
-                p.merge_access_levels
-                    .iter()
-                    .map(|n| format!("merge={}", n.access_level)),
-            );
-            if p.allow_force_push {
-                reglas_rama.push("force-push".to_owned());
-            }
-            if p.code_owner_approval_required {
-                reglas_rama.push("code-owner".to_owned());
-            }
-        }
-        let r = self.con.pedir(
-            LEER_PUERTAS,
-            rol,
-            self.ep(&Endpoint::ReglasAprobacion),
-            None,
-        )?;
-        let (plan_da_reglas, reglas) = if r.estado == 403 {
-            (false, Vec::new())
-        } else if r.exito() {
-            let v: Vec<ReglaAprobacionApi> = self.con.parsear(LEER_PUERTAS, &r.cuerpo)?;
-            (true, v)
+        validar_puertas(rol, rama)?;
+        let comun = self.puertas_comunes(rol)?;
+        Ok(self.puertas_de_rama(rol, rama, &comun, true))
+    }
+
+    fn leer_puertas_de(
+        &self,
+        rol: Rol,
+        ramas: &[&str],
+        checks_de: &str,
+    ) -> Vec<Paso<ReglasLeidas>> {
+        // Ramas protegidas, reglas de aprobacion y proyecto son del
+        // proyecto: se leen una vez (R1-06 de TASK-035).
+        let validas: Vec<Paso<()>> = ramas.iter().map(|r| validar_puertas(rol, r)).collect();
+        let comun = if validas.iter().any(Result::is_ok) {
+            Some(self.puertas_comunes(rol))
         } else {
-            return Err(self.con.traducir(LEER_PUERTAS, rol, PERMISO_LECTURA, &r));
+            None
         };
-        let (requeridos, todos) = self.checks_requeridos(rol);
-        Ok(ReglasLeidas {
-            plan_da_reglas,
-            aprobaciones: reglas
-                .iter()
-                .map(|r| r.approvals_required)
-                .max()
-                .unwrap_or(0),
-            rulesets: reglas.into_iter().map(|r| r.name).collect(),
-            reglas_rama,
-            checks_requeridos: requeridos,
-            todos_los_checks: todos,
-            checks_publicados: self.checks_publicados(rol, rama),
-        })
+        ramas
+            .iter()
+            .zip(validas)
+            .map(|(rama, v)| {
+                v?;
+                match comun.as_ref() {
+                    Some(Ok(c)) => Ok(self.puertas_de_rama(rol, rama, c, *rama == checks_de)),
+                    Some(Err(i)) => Err(i.clone()),
+                    None => Err(rol_no_valido(LEER_PUERTAS, rol)),
+                }
+            })
+            .collect()
     }
 
     fn leer_permisos_ci(&self) -> Informe {
@@ -934,6 +910,116 @@ impl Plataforma for GitLab {
     }
 }
 
+/// Rama y rol de `leer-puertas`.
+fn validar_puertas(rol: Rol, rama: &str) -> Paso<()> {
+    if !rama_valida(rama) {
+        return Err(uso(LEER_PUERTAS, "rama-invalida", &t::RAMA_NO_VALIDA));
+    }
+    if rol == Rol::Revisor {
+        return Err(rol_no_valido(LEER_PUERTAS, rol));
+    }
+    Ok(())
+}
+
+/// Lo que `leer-puertas` lee del proyecto, comun a todas las ramas.
+struct PuertasComunes {
+    ramas: Vec<RamaProtegidaApi>,
+    plan_da_reglas: bool,
+    reglas: Vec<ReglaAprobacionApi>,
+    requeridos: Option<Vec<String>>,
+    todos: bool,
+}
+
+impl GitLab {
+    /// Ramas protegidas, reglas de aprobacion (un 403: `plan_da_reglas`
+    /// falso) y checks requeridos del proyecto.
+    fn puertas_comunes(&self, rol: Rol) -> Paso<PuertasComunes> {
+        let (_, ruta) = self.ep(&Endpoint::RamasProtegidas);
+        let ramas: Vec<RamaProtegidaApi> =
+            self.con
+                .leer_paginas(LEER_PUERTAS, rol, PERMISO_LECTURA, ruta, |c| {
+                    self.con.parsear(LEER_PUERTAS, c)
+                })?;
+        let r = self.con.pedir(
+            LEER_PUERTAS,
+            rol,
+            self.ep(&Endpoint::ReglasAprobacion),
+            None,
+        )?;
+        let (plan_da_reglas, reglas) = if r.estado == 403 {
+            (false, Vec::new())
+        } else if r.exito() {
+            let v: Vec<ReglaAprobacionApi> = self.con.parsear(LEER_PUERTAS, &r.cuerpo)?;
+            (true, v)
+        } else {
+            return Err(self.con.traducir(LEER_PUERTAS, rol, PERMISO_LECTURA, &r));
+        };
+        let (requeridos, todos) = self.checks_requeridos(rol);
+        Ok(PuertasComunes {
+            ramas,
+            plan_da_reglas,
+            reglas,
+            requeridos,
+            todos,
+        })
+    }
+
+    /// Las puertas de `rama` con lo comun ya leido; los estados de su
+    /// cabeza solo `con_checks`.
+    fn puertas_de_rama(
+        &self,
+        rol: Rol,
+        rama: &str,
+        c: &PuertasComunes,
+        con_checks: bool,
+    ) -> ReglasLeidas {
+        let mut reglas_rama = Vec::new();
+        if let Some(p) = c.ramas.iter().find(|p| p.name == rama) {
+            reglas_rama.extend(
+                p.push_access_levels
+                    .iter()
+                    .map(|n| format!("push={}", n.access_level)),
+            );
+            reglas_rama.extend(
+                p.merge_access_levels
+                    .iter()
+                    .map(|n| format!("merge={}", n.access_level)),
+            );
+            if p.allow_force_push {
+                reglas_rama.push("force-push".to_owned());
+            }
+            if p.code_owner_approval_required {
+                reglas_rama.push("code-owner".to_owned());
+            }
+        }
+        let (checks_publicados, fallo_checks) = if con_checks {
+            match self.checks_publicados(rol, rama) {
+                Ok(v) => (Some(v), None),
+                Err(f) => (None, Some(f)),
+            }
+        } else {
+            (None, None)
+        };
+        ReglasLeidas {
+            plan_da_reglas: c.plan_da_reglas,
+            aprobaciones: c
+                .reglas
+                .iter()
+                .map(|r| r.approvals_required)
+                .max()
+                .unwrap_or(0),
+            rulesets: c.reglas.iter().map(|r| r.name.clone()).collect(),
+            reglas_rama,
+            checks_requeridos: c.requeridos.clone(),
+            todos_los_checks: c.todos,
+            checks_publicados,
+            fallo_checks,
+            descartar_viejas: false,
+            ultimo_push: false,
+        }
+    }
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
diff --git a/crates/tc2/src/plataforma/mod.rs b/crates/tc2/src/plataforma/mod.rs
index 3576c55..549748b 100644
--- a/crates/tc2/src/plataforma/mod.rs
+++ b/crates/tc2/src/plataforma/mod.rs
@@ -150,8 +150,71 @@ pub struct ReglasLeidas {
     pub todos_los_checks: bool,
     /// Nombres de los checks publicados en la cabeza de la rama (GitHub,
     /// check-runs; GitLab, estados del commit). `None` si no se pudieron
-    /// leer (rama inexistente, permisos...).
+    /// leer (rama inexistente, permisos...: el motivo, en
+    /// [`ReglasLeidas::fallo_checks`]) o no se pidieron
+    /// ([`Plataforma::leer_puertas_de`]).
     pub checks_publicados: Option<Vec<String>>,
+    /// Por que no se leyeron los checks publicados, si se pidieron y fallo
+    /// la lectura (F3-8, H3 de TASK-035): un 403 da su permiso.
+    pub fallo_checks: Option<FalloLectura>,
+    /// GitHub: alguna regla `pull_request` de la rama tiene
+    /// `dismiss_stale_reviews_on_push` (G04). En GitLab no se lee.
+    pub descartar_viejas: bool,
+    /// GitHub: alguna regla `pull_request` de la rama tiene
+    /// `require_last_push_approval` (G05). En GitLab no se lee.
+    pub ultimo_push: bool,
+}
+
+/// Por que fallo una lectura secundaria que no tumba la operacion (los
+/// checks publicados de `leer-puertas`).
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct FalloLectura {
+    /// `permiso-insuficiente` en un 403; si no, el `motivo` del informe (o
+    /// su resultado).
+    pub motivo: String,
+    /// El permiso que falta, en un 403.
+    pub permiso: Option<&'static str>,
+}
+
+impl FalloLectura {
+    /// El fallo de `i`, con `permiso` si es un 403 de permisos.
+    pub fn de(i: &Informe, permiso: &'static str) -> Self {
+        let denegada = i.resultado == crate::contrato::Resultado::Denegada
+            && i.valor("motivo") == Some("permisos");
+        if denegada {
+            Self {
+                motivo: "permiso-insuficiente".to_owned(),
+                permiso: Some(permiso),
+            }
+        } else {
+            Self {
+                motivo: i
+                    .valor("motivo")
+                    .unwrap_or_else(|| i.resultado.texto())
+                    .to_owned(),
+                permiso: None,
+            }
+        }
+    }
+
+    /// Un fallo sin informe (respuesta que no se entiende).
+    pub fn motivo(m: &str) -> Self {
+        Self {
+            motivo: m.to_owned(),
+            permiso: None,
+        }
+    }
+}
+
+/// `null` como el valor por defecto (H5 de TASK-035): `#[serde(default)]`
+/// solo cubre el campo ausente.
+pub(crate) fn nulo_por_defecto<'de, D, T>(d: D) -> Result<T, D::Error>
+where
+    D: serde::Deserializer<'de>,
+    T: Default + serde::Deserialize<'de>,
+{
+    use serde::Deserialize;
+    Ok(Option::<T>::deserialize(d)?.unwrap_or_default())
 }
 
 /// Tier de GitLab (F1-13, decision 2).
@@ -242,6 +305,20 @@ pub trait Plataforma {
     /// `leer-puertas` (agente o persona).
     fn leer_puertas(&self, rol: Rol, rama: &str) -> Paso<ReglasLeidas>;
 
+    /// `leer-puertas` de varias `ramas` a la vez (F3-8, R1-06 de
+    /// TASK-035): lo comun al repo se lee una vez y los checks publicados
+    /// solo en `checks_de` (en las demas, `checks_publicados: None`). Un
+    /// resultado por rama, en el mismo orden. Por defecto, una a una.
+    fn leer_puertas_de(
+        &self,
+        rol: Rol,
+        ramas: &[&str],
+        checks_de: &str,
+    ) -> Vec<Paso<ReglasLeidas>> {
+        let _ = checks_de;
+        ramas.iter().map(|r| self.leer_puertas(rol, r)).collect()
+    }
+
     /// `leer-permisos-ci` (persona): que el token de la CI no pueda aprobar.
     fn leer_permisos_ci(&self) -> Informe;
 
diff --git a/docs/adr/F1-13-adaptador-de-plataforma.md b/docs/adr/F1-13-adaptador-de-plataforma.md
index a0a2d50..4d28e58 100644
--- a/docs/adr/F1-13-adaptador-de-plataforma.md
+++ b/docs/adr/F1-13-adaptador-de-plataforma.md
@@ -5,7 +5,7 @@
 - Requisitos: RR-15 (GitHub y GitLab desde la 2.0, con la diferencia documentada y dicha por `doctor`), RF-54 (siempre por PR o MR, GitLab detrás del adaptador), RF-64 (`doctor` con el arreglo de cada fallo), RN-01 (identidad propia del agente) y RR-08 (las garantías las impone la plataforma) de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); RR-12 (equipo) para la interferencia en un MR compartido
 - Depende de: [ADR F1-4](F1-4-operaciones-git-y-contrato-de-salida.md) (puerto `Plataforma`, lista cerrada, contrato de salida y códigos 0–7), [ADR F1-5](F1-5-puertas-en-github.md) (modelo neutral de puertas, `aplicar-puertas`, `leer-puertas`, D2a y D3a), [ADR F1-6](F1-6-identidades-agente-revisor-persona.md) (operación → identidad, contrato del firmante, decisión 6), [ADR F1-7](F1-7-verificacion-ci-perfiles-cache-runners.md) (caché en el check `verificacion`), [ADR F1-8](F1-8-seguridad-modelo-de-amenazas.md) (dos clases de jobs, `carril` con la definición de la base, runner propio), [ADR F1-10](F1-10-carriles-y-modo-automatico.md) (carril mínimo con los dos nombres de cada renombrado), [ADR F1-11](F1-11-extensiones-declaradas.md) (comandos sí, hooks solo como atajo opcional), [ADR F1-12](F1-12-actualizacion-entre-versiones.md) (nombres `tc2` y `taskcodetwo`) y [ADR F1-14](F1-14-distribucion-e-instalacion.md) (HTTP con TLS del sistema, sin `gh` ni `glab`)
 - Adenda (2026-11-17, TASK-032, F3-5): operación nueva de lectura `prs-abiertos(rol)` → lista de `{número, rama, sha, borrador}` de los PR/MR abiertos **del propio repo**, por número; campos mínimos (D2 y D3 de TASK-032: sin checks ni aprobaciones). Rol `agente` o `persona` (el `revisor` se rechaza); permiso `pull_requests:read` en GitHub (el mismo que ya usa `abrir-pr` para buscar el PR existente) y el permiso de lectura del adaptador en GitLab (`read_api` o `api` y rol Developer, el mismo de `abrir-pr`). Los PR de forks no cuentan: en GitHub se descarta el que tiene `head.repo.id` distinto de `base.repo.id` (o nulo, fork borrado); en GitLab, el que tiene `source_project_id` distinto de `target_project_id`. Borrador: `draft` en GitHub; `draft` o `work_in_progress` (obsoleto) en GitLab. Campos de GitLab comprobados contra la documentación de la API de MR (2026-11-17), no contra la API real. Una rama que no pasa `rama_valida` se salta (no puede casar con una tarea); un sha que no es de 40 hex es `respuesta-invalida`. Paginación con `Link rel=next` hasta `MAX_PAGINAS` (10 × 100); con más, `error motivo=demasiadas-paginas` y `board` sale con `plataforma=error`. El tope HTTP de 30 s es **por llamada**, no por operación: en el peor caso una consulta paginada podría esperar 10 × 30 s; sin red cae en la primera llamada (tope de conexión de 10 s). Probado contra el mock (`tests/prs_abiertos.rs`), no contra GitHub ni GitLab reales
-- Adenda (2026-11-17, TASK-035, F3-8): **`leer-puertas` gana lecturas** (decisión 4) y devuelve además `aprobaciones`, `checks_requeridos`, `todos_los_checks` y `checks_publicados`. GitHub: los `parameters` de las reglas que ya trae `GET /rules/branches/{rama}` (`required_status_checks[].context`, `required_approving_review_count`) y una llamada más, `GET /commits/heads/{rama}/check-runs` (la página da la rama como `heads/BRANCH_NAME`: [doc](https://docs.github.com/en/rest/checks/runs), 2026-11-17; esquema de reglas: [doc](https://docs.github.com/en/rest/repos/rules), 2026-11-17, que no trae ejemplo de respuesta: los nombres salen del esquema de rulesets). GitLab: `GET /projects/:id` (`only_allow_merge_if_pipeline_succeeds`: GitLab no exige checks por nombre; con «Pipelines must succeed» cuenta el pipeline entero, `todos_los_checks`), `GET /repository/branches/:rama` (`commit.id`: [doc](https://docs.gitlab.com/api/branches/), 2026-11-17) y `GET /repository/commits/:sha/statuses` (la página documenta `:sha` como «hash of the commit» y no dice que acepte una rama: [doc](https://docs.gitlab.com/api/commits/), 2026-11-17; por eso se lee antes la rama). El campo `only_allow_merge_if_pipeline_succeeds` sale en los ejemplos de respuesta de la [API de proyectos](https://docs.gitlab.com/api/projects/) (2026-11-17), no en su tabla de atributos: si no viene, `checks_requeridos` es `None` y `doctor` dice `no-comprobado`. Lo que no se lee nunca se da por bueno. **Operación nueva de lectura `leer-plan(rol)`** (agente o persona): GitHub, `GET /repos/{o}/{r}` (`private`, `visibility`, `owner.type`; [doc](https://docs.github.com/en/rest/repos/repos), 2026-11-17; `internal` cuenta como privado); GitLab, `GET /projects/:id` (`namespace.id`) y `GET /namespaces/:id` (`plan`: la [API de namespaces](https://docs.gitlab.com/api/namespaces/), 2026-11-17, lo da con los ejemplos `free`, `ultimate` y `default` y no dice quién lo ve). Solo `free`, `premium` y `ultimate` se reconocen; cualquier otro valor, o el namespace ilegible, es «plan no comprobado», salvo el 403 de `approval_rules`, que sigue diciendo Free (decisión 3). **Corrige la decisión 3 en GitLab**: el E2E de F2-9 observó que en Free `approval_rules` da 200 con la lista vacía, no 403; por eso el plan sale del namespace. **`capacidades` en F3-8**: la tabla de la decisión 2 compilada (`doctor/capacidades.rs`); nombres de plan `github-free-publico-usuario`, `github-free-privado-usuario`, `github-pro-privado-usuario`, `github-free-publico-org`, `github-team-privado-org`, `github-free-privado-org` (nuevo: 9/1/10) y `gitlab-free|premium|ultimate`; `enterprise-privado-org` no se distingue de Team con lo observable y sale como Team (la cola sustituida: lo conservador). La configuración (`motivo=config arreglo=tc2-init`) solo se comprueba en las garantías que `leer-puertas` lee (G01, G02, G03, G06 en GitLab y G09 en GitHub); las demás siguen al plan. Arreglos: `repo-publico-o-github-pro`, `org-publica-o-enterprise-cloud`, `gitlab-premium`, `ninguno-limite-de-la-plataforma` y `tc2-init`. Línea de `doctor`: las cuentas van en su primera línea (`plan=… impuestas=N sustituidas=M no-disponibles=K`, sin la línea aparte `doctor puertas …` de la sección C, que no cabría en las 5) y cada garantía no impuesta es un hallazgo `garantia Gnn aviso motivo=… capacidad=sustituida por=…|no-disponible arreglo=…`. Probado contra el mock (`tests/doctor.rs`), no contra GitHub ni GitLab reales
+- Adenda (2026-11-17, TASK-035, F3-8): **`leer-puertas` gana lecturas** (decisión 4) y devuelve además `aprobaciones`, `checks_requeridos`, `todos_los_checks` y `checks_publicados`. GitHub: los `parameters` de las reglas que ya trae `GET /rules/branches/{rama}` (`required_status_checks[].context`, `required_approving_review_count`) y una llamada más, `GET /commits/heads/{rama}/check-runs` (la página da la rama como `heads/BRANCH_NAME`: [doc](https://docs.github.com/en/rest/checks/runs), 2026-11-17; esquema de reglas: [doc](https://docs.github.com/en/rest/repos/rules), 2026-11-17, que no trae ejemplo de respuesta: los nombres salen del esquema de rulesets). GitLab: `GET /projects/:id` (`only_allow_merge_if_pipeline_succeeds`: GitLab no exige checks por nombre; con «Pipelines must succeed» cuenta el pipeline entero, `todos_los_checks`), `GET /repository/branches/:rama` (`commit.id`: [doc](https://docs.gitlab.com/api/branches/), 2026-11-17) y `GET /repository/commits/:sha/statuses` (la página documenta `:sha` como «hash of the commit» y no dice que acepte una rama: [doc](https://docs.gitlab.com/api/commits/), 2026-11-17; por eso se lee antes la rama). El campo `only_allow_merge_if_pipeline_succeeds` sale en los ejemplos de respuesta de la [API de proyectos](https://docs.gitlab.com/api/projects/) (2026-11-17), no en su tabla de atributos: si no viene, `checks_requeridos` es `None` y `doctor` dice `no-comprobado`. Lo que no se lee nunca se da por bueno. **Operación nueva de lectura `leer-plan(rol)`** (agente o persona): GitHub, `GET /repos/{o}/{r}` (`private`, `visibility`, `owner.type`; [doc](https://docs.github.com/en/rest/repos/repos), 2026-11-17; `internal` cuenta como privado); GitLab, `GET /projects/:id` (`namespace.id`) y `GET /namespaces/:id` (`plan`: la [API de namespaces](https://docs.gitlab.com/api/namespaces/), 2026-11-17, lo da con los ejemplos `free`, `ultimate` y `default` y no dice quién lo ve). Solo `free`, `premium` y `ultimate` se reconocen; cualquier otro valor, o el namespace ilegible, es «plan no comprobado», salvo el 403 de `approval_rules`, que sigue diciendo Free (decisión 3). **Corrige la decisión 3 en GitLab**: el E2E de F2-9 observó que en Free `approval_rules` da 200 con la lista vacía, no 403; por eso el plan sale del namespace. **`capacidades` en F3-8**: la tabla de la decisión 2 compilada (`doctor/capacidades.rs`); nombres de plan `github-free-publico-usuario`, `github-free-privado-usuario`, `github-pro-privado-usuario`, `github-free-publico-org`, `github-team-privado-org`, `github-free-privado-org` (nuevo: 9/1/10) y `gitlab-free|premium|ultimate`; `enterprise-privado-org` no se distingue de Team con lo observable y sale como Team (la cola sustituida: lo conservador). La configuración (`motivo=config arreglo=tc2-init`) solo se comprueba en las garantías que `leer-puertas` lee: en GitHub, G01 (`deletion` y `non_fast_forward`), G02 (`pull_request`), G03 (`required_approving_review_count`), G04 (`dismiss_stale_reviews_on_push`), G05 (`require_last_push_approval`) y G09 (`required_status_checks`), todos de `GET /rules/branches/{rama}` (los cuatro parámetros de `pull_request` son obligatorios en el esquema de [rulesets](https://docs.github.com/en/rest/repos/rules), 2026-11-17); en GitLab, G02 (`push_access_levels`), G03 (`approval_rules`) y G06 (`code_owner_approval_required`); las demás siguen al plan. Si la configuración de una de esas garantías no se pudo leer (las puertas de `develop` fallaron, o en GitLab el 403 de `approval_rules` con un plan legible que las da), sale `garantia Gnn no-comprobado motivo=puertas-no-leidas|permiso-insuficiente`, no cuenta como impuesta y la primera línea añade `no-comprobadas=N` (ronda 1 de TASK-035, H1 y H4). Un `null` donde la API da una lista, un booleano o un número (`required_status_checks`, `push_access_levels`, `code_owner_approval_required`, `approvals_required`) se lee como vacío, `false` o 0: lo conservador. `doctor` lee las puertas de `develop` y `main` con una sola lectura de lo común al repo (`rulesets`; en GitLab, ramas protegidas, `approval_rules` y proyecto) y los checks publicados solo en `develop`; si esos checks no se leen, `check … no-comprobado` da el motivo y, en un 403, `permiso=checks:read` en GitHub (permiso «Checks» de lectura de una GitHub App: [doc](https://docs.github.com/en/rest/authentication/permissions-required-for-github-apps), 2026-11-17) o `permiso=read_api` en GitLab. Arreglos: `repo-publico-o-github-pro`, `org-publica-o-enterprise-cloud`, `gitlab-premium`, `ninguno-limite-de-la-plataforma` y `tc2-init`. Línea de `doctor`: las cuentas van en su primera línea (`plan=… impuestas=N sustituidas=M no-disponibles=K`, sin la línea aparte `doctor puertas …` de la sección C, que no cabría en las 5) y cada garantía no impuesta es un hallazgo `garantia Gnn aviso motivo=… capacidad=sustituida por=…|no-disponible arreglo=…`. Probado contra el mock (`tests/doctor.rs`), no contra GitHub ni GitLab reales
 - Decide: Carlos, con sus decisiones del 2026-11-14 (nombres `tc2`, `taskcodetwo` y rama `taskcodetwo/tareas`; comandos deterministas antes que hooks; seguridad y trabajo en grupo imprescindibles; sin trocear y fiel a los datos; GitHub y GitLab solo lectura) y la consigna «la opción recomendada» (D1a a D8a del plan de TASK-015)
 
 ## Contexto
diff --git a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
index 7aaa269..b522b7c 100644
--- a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
+++ b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
@@ -8,7 +8,7 @@
 - Adenda (2026-11-17, TASK-032, F3-5): entra `prs-abiertos` en la lista cerrada (decisión 2) para `tc2 board` (D2 de TASK-032, aceptada por Carlos el 2026-11-16); detalle de llamadas en la [adenda de F1-13](F1-13-adaptador-de-plataforma.md). `board` y `status`, como `roadmap`, escriben avisos de una línea en stderr con código 0; el canal fiable son las claves de la línea y del JSON
 - Adenda (2026-11-17, TASK-033, F3-6): `start [--json] TASK-NNN` compone `fetch`, `validar-base`, `comprobar-id`, `commit-estado`, `crear-rama-y-worktree` y `reanudar` sin operaciones nuevas (la base de un `hotfix` es el último tag por versión de `refs/tags/` contenido en `origin/main`, leído con `for-each-ref --merged`, o su punta). Línea de éxito: `start ok id=… rama=… worktree=… base=… estado=<commit de la toma> asignado=<login>`; dueño con la tarea empezada: `sin-cambios` o `ok creado=rama,worktree` sin escribir estado. Motivos de `rechazada` (3): `no-existe`, `formato-1` (con `arreglo=tc2 actualizar`), `ilegible`, `tomada` (con `asignado=<login>`), `asignado-heredado`, `terminada`, `descartada`, `estado-no-valido`, `dependencias-pendientes` (con `dependencias=`), `rama-invalida`, `tipo-release`, `ocupada` (con `ramas=`), `carpeta-ocupada` (la carpeta del worktree existe y no es el worktree registrado de la rama; se comprueba antes de publicar la toma, sin `estado=`) e `identidad`; `uso` (2): `sin-id`, `id-invalido`, `argumentos-de-mas`; sin red, `sinred pendiente=no` (4)
 - Adenda (2026-11-17, TASK-034, F3-7): `descartar [--json] TASK-NNN "<motivo>"`, `pausa [--json] TASK-NNN` y `reanudar [--json] TASK-NNN` componen `fetch` (solo `descartar`), `commit-estado` y `quitar-worktree` (solo `descartar`, tras publicar) sin operaciones nuevas. Líneas: `descartar ok id=… commit=… intentos=… tareas=… worktree=quitado|ninguno|conservado [worktree-motivo=cambios-sin-guardar|sin-publicar|sin-rama|en-uso|error]` (el worktree conservado no hace fallar la orden: el descarte ya está publicado y la rama no se toca nunca); `pausa ok id=… commit=… intentos=… tareas=…` y `reanudar ok id=… commit=… intentos=… tareas=…` (las claves de la transacción, como `descartar`, sin `worktree`). `sin-cambios` (0): descartar una descartada (vale el motivo de entonces y se intenta quitar el worktree), pausar una en pausa o reanudar una que no lo está. Motivos de `rechazada` (3): `no-existe`, `formato-1` (con `arreglo=tc2 actualizar`), `ilegible`, `de-otra-persona` (con `asignado=<login>`; D2: puede quien la lleva o cualquiera si no la lleva nadie), `asignado-heredado`, `terminada`, `descartada` (pausa y reanudar), `estado-no-valido` e `identidad`; `start` gana `pausada` (con `arreglo=tc2 reanudar`). `uso` (2): `sin-id`, `id-invalido`, `argumentos-de-mas` y, en `descartar`, `sin-motivo` y `motivo-invalido` (vacío, solo espacios, saltos de línea o caracteres de control, o más de 500 caracteres). Sin red: `descartar sinred pendiente=no` (4); `pausa`/`reanudar sinred pendiente=si` (4), que la siguiente transacción con red publica (adenda de TASK-034 en F1-2)
-- Adenda (2026-11-17, TASK-035, F3-8): `doctor [--json]` compone lecturas sin escribir nada (ni `fetch`): `git version` entra en la lista permitida (decisión 3; solo lectura), y la lista cerrada de la plataforma gana `leer-plan` (detalle en la [adenda de F1-13](F1-13-adaptador-de-plataforma.md)). Salida: primera línea `doctor <ok|rechazada> fallos=N avisos=N no-comprobado=N plataforma=ok|no-configurada|sin-credenciales|sin-red|error [plan=… impuestas=N sustituidas=N no-disponibles=N] [commit=…]`; detrás, los hallazgos que no son `ok` (`<zona> <id> <falla|aviso|no-comprobado> [motivo=…] [claves] [arreglo=…]`), los peores primero, y si no caben en 5 líneas, 3 y `y N mas: tc2 doctor --json`. `--json`: una línea con las claves y `hallazgos` (todos, también los `ok`). Código 3 (`rechazada`) con alguna `falla` (D5a de TASK-035) y 0 con avisos o no-comprobados; fuera de un repo, `doctor rechazada motivo=no-es-repo` (3); argumentos de más, `uso motivo=argumentos` (2; `--recursos` llega con F3-9). Git por debajo de 2.56: `aviso motivo=git-no-verificado` (D3a de TASK-035); sin git o con una versión que no se entiende, `falla`. Diagnóstico de una línea en stderr solo con alguna `falla`
+- Adenda (2026-11-17, TASK-035, F3-8): `doctor [--json]` compone lecturas sin escribir nada (ni `fetch`): `git version` entra en la lista permitida (decisión 3; solo lectura), y la lista cerrada de la plataforma gana `leer-plan` (detalle en la [adenda de F1-13](F1-13-adaptador-de-plataforma.md)). Salida: primera línea `doctor <ok|rechazada> fallos=N avisos=N no-comprobado=N plataforma=ok|no-configurada|sin-credenciales|sin-red|error [plan=… impuestas=N sustituidas=N no-disponibles=N [no-comprobadas=N]] [commit=…]` (`no-comprobadas` solo si alguna garantía cuya configuración se lee no se pudo leer: no cuenta como impuesta; ronda 1 de TASK-035); detrás, los hallazgos que no son `ok` (`<zona> <id> <falla|aviso|no-comprobado> [motivo=…] [claves] [arreglo=…]`), los peores primero, y si no caben en 5 líneas, 3 y `y N mas: tc2 doctor --json`. `--json`: una línea con las claves y `hallazgos` (todos, también los `ok`). Código 3 (`rechazada`) con alguna `falla` (D5a de TASK-035) y 0 con avisos o no-comprobados; fuera de un repo, `doctor rechazada motivo=no-es-repo` (3); argumentos de más, `uso motivo=argumentos` (2; `--recursos` llega con F3-9). Git por debajo de 2.56: `aviso motivo=git-no-verificado` (D3a de TASK-035); sin git o con una versión que no se entiende, `falla`. Diagnóstico de una línea en stderr solo con alguna `falla`
 - Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
 
 ## Contexto
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 1aa0a12edd779a7211e471d9132d566be737d3da..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-035/planificacion/brainstorm/peticion-plan-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-035/planificacion/plan-final.md                 |    0
 tareas/03-en-revision/TASK-035/revision/informe-revision-1-code-quality-reviewer.md         |   21 +
 tareas/03-en-revision/TASK-035/revision/informe-revision-1-rust-wasm-reviewer.md            |   21 +
 tareas/03-en-revision/TASK-035/revision/peticion-revision-1-code-quality-reviewer.md        |   96 ++++
 tareas/03-en-revision/TASK-035/revision/peticion-revision-1-rust-wasm-reviewer.md           | 2575 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-035/tarea.md                                    |    6 +-
 7 files changed, 2717 insertions(+), 2 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff 1aa0a12edd779a7211e471d9132d566be737d3da..HEAD -- ":(literal)crates/tc2/tests/doctor.rs"
````

````
crates/tc2/tests/doctor.rs | 180 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++---------
 1 file changed, 171 insertions(+), 9 deletions(-)
````
