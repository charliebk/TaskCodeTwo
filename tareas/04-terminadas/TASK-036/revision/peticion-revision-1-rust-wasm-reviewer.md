# Peticion de revision — TASK-036 (ronda 1)

- Tarea: TASK-036 — F4-1 plan con brainstorm de roles
- Rama revisada: feature/task-036-f4-1-plan-con-brainstorm-de-roles
- Rama base: develop
- Commit revisado (HEAD): 6ecf89825e4a41e13abdd7f70a2fa08bcbf65ff6
- Fecha: 2026-11-19
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-036 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-036/tarea.md
- tareas/03-en-revision/TASK-036/planificacion/
- crates/tc2/src/carril.rs
- crates/tc2/src/config/mod.rs
- crates/tc2/src/i18n/catalogo/cli.rs
- crates/tc2/src/i18n/catalogo/mod.rs
- crates/tc2/src/i18n/catalogo/plan.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/plan/guardar.rs
- crates/tc2/src/plan/heuristica.rs
- crates/tc2/src/plan/mod.rs
- crates/tc2/src/plan/operacion.rs
- crates/tc2/src/plan/plantillas.rs
- crates/tc2/src/plan/roles.rs
- crates/tc2/tests/plan.rs

## Commits a revisar (git log develop..HEAD)

````
6ecf898 chore(TASK-036): autorrevision
dbe7b5f feat(TASK-036): tc2 plan con brainstorm de roles y tc2 plan guardar
a963125 chore(TASK-036): tarea en curso
````

## Diff de tu dominio (13 fichero(s) de 14; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/carril.rs b/crates/tc2/src/carril.rs
index 837e361..087217f 100644
--- a/crates/tc2/src/carril.rs
+++ b/crates/tc2/src/carril.rs
@@ -1,10 +1,13 @@
 //! Carriles de la politica (ADR F1-10, decision 1): el sitio unico de la
 //! tabla de suelo. F2-7 solo necesita las columnas que valida la
 //! configuracion (roles de diseno minimos y perfil); F4 y F6 amplian este
-//! modulo en vez de copiar la tabla.
+//! modulo en vez de copiar la tabla. F4-1 (`plan`) anade el carril
+//! declarado por el tipo y cuantos roles de diseno se lanzan.
 
 use serde::Deserialize;
 
+use crate::config::Config;
+
 /// Carril de una tarea, de menor a mayor exigencia (F1-10, decision 1).
 /// `release` y `hotfix` son tipos, no carriles.
 #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
@@ -60,8 +63,51 @@ impl Carril {
             Self::Errata | Self::Fix | Self::Funcionalidad | Self::RiesgoAlto => Perfil::Completa,
         }
     }
+
+    /// Carril declarado por el tipo de la tarea (F1-10, PoC C): `fix` para
+    /// el tipo `fix` y `funcionalidad` para el resto. `errata` no sale del
+    /// tipo (la crea su orden, D5a) y riesgo alto solo se pide o se deriva
+    /// del diff (F4-1 D4).
+    pub fn declarado(tipo: &str) -> Self {
+        if tipo == "fix" {
+            Self::Fix
+        } else {
+            Self::Funcionalidad
+        }
+    }
+
+    /// Cuantos roles de diseno lanza `plan` (F4-1, columna «Roles de
+    /// diseno»): errata 0, fix 1 (D2), funcionalidad los de la complejidad
+    /// (`por_complejidad`, de la heuristica de la 0.16, D5) y como poco
+    /// [`Carril::roles_min`], riesgo alto los cuatro de base (D1). Un
+    /// `hotfix` fuera de riesgo alto se queda en 1 como mucho (el tope de la
+    /// 0.16). Despues, `politica_carriles.<carril>.roles_diseno` solo sube
+    /// (F1-10 decision 2: la configuracion endurece, nunca baja); el
+    /// esquema lo deja en 5 como mucho.
+    pub fn roles_diseno(self, por_complejidad: u8, tipo: &str, config: &Config) -> u8 {
+        let suelo = u8::try_from(self.roles_min()).unwrap_or(u8::MAX);
+        let mut n = match self {
+            Self::Errata => 0,
+            Self::Fix => 1,
+            Self::Funcionalidad => por_complejidad.max(suelo),
+            Self::RiesgoAlto => suelo,
+        };
+        if tipo == "hotfix" && self != Self::RiesgoAlto {
+            n = n.min(TOPE_HOTFIX);
+        }
+        let minimo = config
+            .politica_carriles
+            .get(&self)
+            .and_then(|p| p.roles_diseno)
+            .unwrap_or(0);
+        n.max(minimo)
+    }
 }
 
+/// Roles de diseno de un `hotfix` como mucho (`agentes_brainstorm_hotfix` de
+/// la 0.16): abrevia el brainstorm previo, nunca la revision.
+const TOPE_HOTFIX: u8 = 1;
+
 /// Perfil de verificacion exigido por la politica (F1-10, columna «Perfil»;
 /// F1-7), de menor a mayor.
 #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
@@ -128,4 +174,67 @@ mod tests {
             assert_eq!(p.y_mayores().first(), Some(&p.nombre()));
         }
     }
+
+    /// Criterio 2 de TASK-036 (tabla): `roles_diseno` y la seleccion por
+    /// carril x roles por complejidad x tipo (hotfix) x etiqueta de
+    /// rendimiento x `politica_carriles`. Uno en fix, todos en riesgo alto.
+    #[test]
+    fn roles_de_diseno_por_carril() {
+        use crate::config::PoliticaCarril;
+        use crate::plan::roles::seleccionar;
+        let suelo = Config::suelo();
+        let con = |c: Carril, n: u8| {
+            let mut x = Config::suelo();
+            x.politica_carriles.insert(
+                c,
+                PoliticaCarril {
+                    roles_diseno: Some(n),
+                    plan_aprobado: None,
+                    paradas_extra: Vec::new(),
+                    revisoras_extra: Vec::new(),
+                    lote: None,
+                    perfil: None,
+                },
+            );
+            x
+        };
+        let rend = vec!["rendimiento".to_owned()];
+        let nada: Vec<String> = Vec::new();
+        let fix3 = con(Carril::Fix, 3);
+        let func5 = con(Carril::Funcionalidad, 5);
+        let otro = con(Carril::RiesgoAlto, 5);
+        // (carril, por complejidad, tipo, config, etiquetas, roles, ids)
+        type Caso<'a> = (Carril, u8, &'a str, &'a Config, &'a Vec<String>, u8, usize);
+        let casos: [Caso<'_>; 16] = [
+            (Carril::Errata, 4, "feature", &suelo, &nada, 0, 0),
+            (Carril::Fix, 0, "fix", &suelo, &nada, 1, 1),
+            (Carril::Fix, 4, "fix", &suelo, &rend, 1, 1),
+            (Carril::Fix, 0, "fix", &fix3, &nada, 3, 3),
+            (Carril::Fix, 0, "fix", &fix3, &rend, 3, 4),
+            (Carril::Fix, 0, "fix", &otro, &nada, 1, 1),
+            (Carril::Funcionalidad, 0, "feature", &suelo, &nada, 1, 1),
+            (Carril::Funcionalidad, 1, "feature", &suelo, &rend, 1, 1),
+            (Carril::Funcionalidad, 2, "feature", &suelo, &rend, 2, 3),
+            (Carril::Funcionalidad, 4, "feature", &suelo, &nada, 4, 4),
+            (Carril::Funcionalidad, 4, "hotfix", &suelo, &nada, 1, 1),
+            (Carril::Funcionalidad, 0, "feature", &func5, &nada, 5, 5),
+            (Carril::RiesgoAlto, 0, "feature", &suelo, &nada, 4, 4),
+            (Carril::RiesgoAlto, 0, "hotfix", &suelo, &nada, 4, 4),
+            (Carril::RiesgoAlto, 0, "fix", &suelo, &rend, 4, 5),
+            (Carril::RiesgoAlto, 0, "feature", &otro, &nada, 5, 5),
+        ];
+        for (c, complejidad, tipo, cfg, et, roles, ids) in casos {
+            let n = c.roles_diseno(complejidad, tipo, cfg);
+            assert_eq!(n, roles, "{c:?} {complejidad} {tipo}");
+            assert_eq!(
+                seleccionar(n, et).len(),
+                ids,
+                "{c:?} {complejidad} {tipo} {et:?}"
+            );
+        }
+        assert_eq!(Carril::declarado("fix"), Carril::Fix);
+        for t in ["feature", "hotfix", "release", "otro"] {
+            assert_eq!(Carril::declarado(t), Carril::Funcionalidad, "{t}");
+        }
+    }
 }
diff --git a/crates/tc2/src/config/mod.rs b/crates/tc2/src/config/mod.rs
index f4bcd89..14b0db0 100644
--- a/crates/tc2/src/config/mod.rs
+++ b/crates/tc2/src/config/mod.rs
@@ -260,6 +260,17 @@ pub enum ModoFlujo {
     Automatico,
 }
 
+impl ModoFlujo {
+    /// Nombre en la configuracion y en la columna `modo` de `## Transiciones`.
+    pub fn nombre(self) -> &'static str {
+        match self {
+            Self::Manual => "manual",
+            Self::Semiautomatico => "semiautomatico",
+            Self::Automatico => "automatico",
+        }
+    }
+}
+
 /// `lote` (F1-16): sin cifras por defecto.
 #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
 #[serde(deny_unknown_fields)]
diff --git a/crates/tc2/src/i18n/catalogo/cli.rs b/crates/tc2/src/i18n/catalogo/cli.rs
index 628cab6..5bc161e 100644
--- a/crates/tc2/src/i18n/catalogo/cli.rs
+++ b/crates/tc2/src/i18n/catalogo/cli.rs
@@ -10,8 +10,8 @@ crate::i18n::textos! {
         en: "tc2: could not write the output: {error}",
     }
     USO {
-        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<motivo>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
-        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<reason>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
+        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<motivo>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <carril>] | tc2 plan guardar [--json] TASK-NNN <destino> <fichero> | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
+        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<reason>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <lane>] | tc2 plan guardar [--json] TASK-NNN <destination> <file> | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
     }
     FALTA_LA_ORDEN {
         es: "tc2: falta la orden. {uso}",
diff --git a/crates/tc2/src/i18n/catalogo/mod.rs b/crates/tc2/src/i18n/catalogo/mod.rs
index 30d055a..528ac2d 100644
--- a/crates/tc2/src/i18n/catalogo/mod.rs
+++ b/crates/tc2/src/i18n/catalogo/mod.rs
@@ -14,6 +14,7 @@ pub mod git;
 pub mod identidad;
 pub mod importar;
 pub mod init;
+pub mod plan;
 pub mod plataforma;
 pub mod progreso;
 pub mod roadmap;
@@ -39,6 +40,7 @@ pub fn todos() -> impl Iterator<Item = &'static Texto> {
         identidad::TODOS,
         importar::TODOS,
         init::TODOS,
+        plan::TODOS,
         plataforma::TODOS,
         progreso::TODOS,
         roadmap::TODOS,
diff --git a/crates/tc2/src/i18n/catalogo/plan.rs b/crates/tc2/src/i18n/catalogo/plan.rs
new file mode 100644
index 0000000..9bae4c8
--- /dev/null
+++ b/crates/tc2/src/i18n/catalogo/plan.rs
@@ -0,0 +1,64 @@
+//! Mensajes de `tc2 plan` y `tc2 plan guardar` (F4-1).
+
+crate::i18n::textos! {
+    USO_PLAN {
+        es: "uso: plan [--json] TASK-NNN [--carril errata|fix|funcionalidad|riesgo-alto] | plan guardar [--json] TASK-NNN <destino> <fichero>",
+        en: "usage: plan [--json] TASK-NNN [--carril errata|fix|funcionalidad|riesgo-alto] | plan guardar [--json] TASK-NNN <destination> <file>",
+    }
+    DESTINO_INVALIDO {
+        es: "el destino {destino} no vale: solo plan-final.md o brainstorm/salida-<rol>-<n>.md, con <rol> uno de {roles}",
+        en: "destination {destino} is not valid: only plan-final.md or brainstorm/salida-<rol>-<n>.md, with <rol> one of {roles}",
+    }
+    DE_OTRA_PERSONA {
+        es: "la tarea {id} la lleva {login}: solo ella la planifica (o pide que te la reasigne)",
+        en: "task {id} is taken by {login}: only they plan it (or ask to have it reassigned)",
+    }
+    PAUSADA {
+        es: "la tarea {id} esta en pausa: reanudala con tc2 reanudar {id} antes de planificarla",
+        en: "task {id} is paused: resume it with tc2 reanudar {id} before planning it",
+    }
+    OBJETIVO_VACIO {
+        es: "la tarea {id} no tiene objetivo (el ## Objetivo esta vacio o es el de la plantilla): escribelo antes de planificarla",
+        en: "task {id} has no goal (## Objetivo is empty or the template one): write it before planning it",
+    }
+    SIN_CRITERIOS {
+        es: "la tarea {id} no tiene ningun criterio de aceptacion (- [ ] en ## Criterios de aceptacion): escribelos antes de planificarla",
+        en: "task {id} has no acceptance criteria (- [ ] in ## Criterios de aceptacion): write them before planning it",
+    }
+    CRITERIO_VACIO {
+        es: "el criterio {n} de la tarea {id} esta vacio o es el de la plantilla: escribelo o quitalo antes de planificarla",
+        en: "criterion {n} of task {id} is empty or the template one: write it or remove it before planning it",
+    }
+    DEMASIADOS_CRITERIOS {
+        es: "la tarea {id} tiene {n} criterios (mas de {max}): es demasiado grande para una tarea; partela en varias",
+        en: "task {id} has {n} criteria (more than {max}): it is too big for one task; split it into several",
+    }
+    CARRIL_BAJA {
+        es: "el carril {pedido} esta por debajo del de la tarea {id} ({minimo}): --carril solo sube (ADR F1-10, decision 3)",
+        en: "lane {pedido} is below that of task {id} ({minimo}): --carril only raises it (ADR F1-10, decision 3)",
+    }
+    SIN_PETICION {
+        es: "no hay peticion para {destino} en la tarea {id} (falta {peticion}): guarda la salida de un rol que tc2 plan haya lanzado",
+        en: "there is no request for {destino} in task {id} ({peticion} is missing): save the output of a role launched by tc2 plan",
+    }
+    PLAN_APROBADO {
+        es: "el plan de la tarea {id} ya esta aprobado: no se cambia su planificacion",
+        en: "the plan of task {id} is already approved: its planning is not changed",
+    }
+    NO_SE_LEE {
+        es: "no se pudo leer {ruta} ({error})",
+        en: "could not read {ruta} ({error})",
+    }
+    NO_UTF8 {
+        es: "{ruta} no es texto UTF-8",
+        en: "{ruta} is not UTF-8 text",
+    }
+    VACIO {
+        es: "{ruta} esta vacio: no se publica un fichero sin contenido",
+        en: "{ruta} is empty: a file without content is not published",
+    }
+    GRANDE {
+        es: "{ruta} pasa de {max} bytes: no se publica",
+        en: "{ruta} is larger than {max} bytes: it is not published",
+    }
+}
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 1c31009..30d6811 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -19,7 +19,7 @@
 //! (`board` y `status`, F3-5) y [`toma`] (`start`, F3-6) y [`solape`]
 //! (aviso de solape de `start`, F3-11) y [`ciclo`]
 //! (`descartar`, `pausa` y `reanudar`, F3-7; `liberar` y `reasignar`,
-//! F3-10) y [`doctor`] (F3-8). Las capas
+//! F3-10) y [`doctor`] (F3-8) y [`plan`] (`plan` y `plan guardar`, F4-1). Las capas
 //! que vienen entran como modulos de esta biblioteca.
 
 pub mod actualizar_cli;
@@ -42,6 +42,7 @@ pub mod importar;
 pub mod init;
 pub mod instalacion;
 pub mod inventario;
+pub mod plan;
 pub mod plataforma;
 pub mod progreso;
 pub mod recursos;
@@ -198,6 +199,10 @@ where
                 ciclo::orden_reasignar(&resto, e)
             })
         }
+        [orden, resto @ ..] if orden == plan::NOMBRE => {
+            let (json, resto) = con_json(resto);
+            de_una_tarea(plan::NOMBRE, (json, idioma), |e| plan::orden(&resto, e))
+        }
         [orden, resto @ ..] if orden == estimacion::NOMBRE => {
             let (json, resto) = con_json(resto);
             orden_estimar(&resto, json, idioma)
@@ -348,7 +353,8 @@ fn orden_identidad(args: &[String]) -> Informe {
 }
 
 /// Una orden sobre una tarea (`start`, F3-6; `descartar`, `pausa` y
-/// `reanudar`, F3-7; `liberar` y `reasignar`, F3-10) en la carpeta actual:
+/// `reanudar`, F3-7; `liberar` y `reasignar`, F3-10; `plan`, F4-1) en la
+/// carpeta actual:
 /// quien la lanza sale de
 /// [`identidad::resolver`] con el terminal y los tokens del proceso.
 ///
@@ -964,6 +970,48 @@ mod tests {
         assert!(s.stderr.starts_with("warning: "), "{}", s.stderr);
     }
 
+    /// TASK-036: `plan` y `plan guardar` mal llamados son `uso` (2) sin
+    /// leer nada, con el diagnostico en los dos idiomas.
+    #[test]
+    fn plan_mal_llamado_es_uso() {
+        for (args, motivo) in [
+            (vec!["plan"], "sin-id"),
+            (vec!["plan", "TASK-1"], "id-invalido"),
+            (vec!["plan", "TASK-001", "--carril"], "carril-invalido"),
+            (
+                vec!["plan", "TASK-001", "--carril", "release"],
+                "carril-invalido",
+            ),
+            (
+                vec!["plan", "TASK-001", "--carril", "fix", "--carril", "fix"],
+                "argumentos-de-mas",
+            ),
+            (vec!["plan", "TASK-001", "x"], "argumentos-de-mas"),
+            (vec!["plan", "guardar"], "sin-id"),
+            (vec!["plan", "guardar", "TASK-001"], "sin-destino"),
+            (
+                vec!["plan", "guardar", "TASK-001", "plan-final.md"],
+                "sin-fichero",
+            ),
+            (
+                vec!["plan", "guardar", "TASK-001", "tarea.md", "f"],
+                "destino-invalido",
+            ),
+            (
+                vec!["plan", "guardar", "TASK-001", "plan-final.md", "f", "x"],
+                "argumentos-de-mas",
+            ),
+        ] {
+            let es = ejecutar_en(Idioma::Es, &args);
+            let en = ejecutar_en(Idioma::En, &args);
+            assert_eq!((es.codigo, en.codigo), (2, 2), "{args:?}");
+            assert_eq!(es.stdout, format!("plan uso motivo={motivo}\n"), "{args:?}");
+            assert_eq!(es.stdout, en.stdout);
+            assert_ne!(es.stderr, en.stderr, "{args:?}");
+            assert_eq!(es.stderr.lines().count(), 1, "{args:?}");
+        }
+    }
+
     #[test]
     fn sin_orden_falla() {
         assert_eq!(ejecutar(Vec::<String>::new()).codigo, 2);
diff --git a/crates/tc2/src/plan/guardar.rs b/crates/tc2/src/plan/guardar.rs
new file mode 100644
index 0000000..6edb865
--- /dev/null
+++ b/crates/tc2/src/plan/guardar.rs
@@ -0,0 +1,263 @@
+//! `plan guardar` como operacion de la rama de estado (F4-1, D3): publica en
+//! `TASK-NNN/planificacion/` un fichero de fuera del repo.
+//!
+//! Ningun agente puede escribir sus salidas en `tareas/` como en la 0.16:
+//! con `tareas/` sucio toda transaccion sale `rechazada
+//! motivo=tareas-con-cambios`. Por eso el plan y las salidas de los roles
+//! entran por aqui, por el punto unico de escritura.
+//!
+//! - Destinos: solo `plan-final.md` o `brainstorm/salida-<rol>-<n>.md`, con
+//!   `<rol>` uno de los roles de [`super::roles`] ([`destino_valido`]). Una
+//!   salida exige su peticion (`peticion-<rol>-<n>.md`) en el estado: no se
+//!   guarda la de un rol que `plan` no lanzo.
+//! - Solo con la tarea `en-diseno` y sin el plan aprobado; la de otra
+//!   persona se rechaza como en `plan`.
+//! - El mismo contenido que ya esta publicado: `sin-cambios`. No toca
+//!   `tarea.md` (no es una fase).
+
+use std::cell::RefCell;
+
+use super::plantillas::{self as p, BRAINSTORM, PLAN_FINAL};
+use super::roles;
+use crate::estado::{Cambio, Estado, Fallo, Operacion, de_otra_persona, por_estado, tarea_de};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::plan as txt;
+use crate::identidad::Quien;
+use crate::tarea::Asignado;
+
+use super::operacion::EN_DISENO;
+
+/// Nombre de la operacion (en el commit y en la anotacion).
+pub const NOMBRE_OP: &str = "plan-guardar";
+
+/// Tope del fichero que se publica (el del import, D4 de TASK-029).
+pub const MAX_BYTES: usize = 1 << 20;
+
+/// Si `destino` es uno de la lista: `plan-final.md` o
+/// `brainstorm/salida-<rol>-<n>.md` con un rol conocido y `n` un entero
+/// positivo sin ceros delante. Devuelve, para una salida, el nombre de su
+/// peticion.
+pub fn destino_valido(destino: &str) -> Option<Option<String>> {
+    if destino == PLAN_FINAL {
+        return Some(None);
+    }
+    let resto = destino
+        .strip_prefix(BRAINSTORM)?
+        .strip_prefix("/salida-")?
+        .strip_suffix(".md")?;
+    let (rol, n) = resto.rsplit_once('-')?;
+    let numero = !n.is_empty() && !n.starts_with('0') && n.bytes().all(|b| b.is_ascii_digit());
+    let conocido = roles::todos().any(|r| r.id == rol);
+    (numero && conocido && n.parse::<u32>().is_ok()).then(|| Some(format!("peticion-{rol}-{n}.md")))
+}
+
+/// Publicar `contenido` en `TASK-NNN/planificacion/<destino>`.
+#[derive(Debug)]
+pub struct Guardar {
+    id: String,
+    asignado: Asignado,
+    destino: String,
+    peticion: Option<String>,
+    contenido: String,
+    dueno: RefCell<Option<String>>,
+}
+
+impl Guardar {
+    /// `plan guardar` de `contenido` en `destino` (ya validado con
+    /// [`destino_valido`]) de la tarea `id`, por `quien`. Los finales de
+    /// linea CRLF pasan a LF.
+    pub fn nueva(id: &str, quien: &Quien, destino: &str, contenido: &str) -> Option<Self> {
+        let peticion = destino_valido(destino)?;
+        Some(Self {
+            id: id.to_owned(),
+            asignado: quien.asignado(),
+            destino: destino.to_owned(),
+            peticion,
+            contenido: contenido.replace("\r\n", "\n"),
+            dueno: RefCell::new(None),
+        })
+    }
+
+    /// Login de quien lleva la tarea si el ultimo intento la rechazo por
+    /// `de-otra-persona`.
+    pub fn dueno(&self) -> Option<String> {
+        self.dueno.borrow().clone()
+    }
+
+    fn rechazo(&self, motivo: &str, m: Mensaje) -> Fallo {
+        Fallo::rechazada(motivo, m.con("id", self.id.as_str()))
+    }
+}
+
+impl Operacion for Guardar {
+    fn nombre(&self) -> &str {
+        NOMBRE_OP
+    }
+
+    fn anotacion(&self) -> String {
+        format!(
+            "{NOMBRE_OP}|{}|{}|{}|{}",
+            self.id,
+            self.asignado.id().unwrap_or_default(),
+            self.asignado.login().unwrap_or_default(),
+            self.destino
+        )
+    }
+
+    fn exige_red(&self) -> bool {
+        // El contenido no cabe en la anotacion: sin red falla, nunca queda
+        // pendiente.
+        true
+    }
+
+    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
+        *self.dueno.borrow_mut() = None;
+        let (t, _) = tarea_de(estado, &self.id)?;
+        if let Some((fallo, dueno)) = de_otra_persona(
+            &self.id,
+            t.asignado_a.as_ref(),
+            &self.asignado,
+            super::NOMBRE,
+            ("de-otra-persona", &txt::DE_OTRA_PERSONA),
+        ) {
+            *self.dueno.borrow_mut() = dueno;
+            return Err(fallo);
+        }
+        if t.estado != EN_DISENO {
+            return Err(por_estado(&self.id, &t.estado, super::NOMBRE, &[EN_DISENO]));
+        }
+        if t.plan_aprobado {
+            return Err(self.rechazo("plan-aprobado", Mensaje::nuevo(&txt::PLAN_APROBADO)));
+        }
+        if let Some(pet) = &self.peticion {
+            let ruta = p::ruta_brainstorm(&self.id, pet);
+            if estado.leer(&ruta)?.is_none() {
+                return Err(self.rechazo(
+                    "sin-peticion",
+                    Mensaje::nuevo(&txt::SIN_PETICION)
+                        .con("destino", self.destino.as_str())
+                        .con("peticion", pet.as_str()),
+                ));
+            }
+        }
+        let ruta = format!("{}/{}/{}", self.id, p::PLANIFICACION, self.destino);
+        if estado.leer(&ruta)?.as_deref() == Some(self.contenido.as_str()) {
+            return Ok(Vec::new());
+        }
+        Ok(vec![Cambio::nuevo(ruta, self.contenido.clone())])
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::identidad::Cuenta;
+    use crate::plataforma::Proveedor;
+    use crate::roadmap::tests::EnMemoria;
+    use crate::tarea::{Cuerpo, Tarea};
+    use std::collections::BTreeMap;
+
+    fn ana() -> Quien {
+        Quien::Agente(Cuenta {
+            proveedor: Proveedor::GitHub,
+            id: 7,
+            login: "ana".to_owned(),
+        })
+    }
+
+    /// Criterio 3: la lista de destinos; lo que no esta en ella no vale.
+    #[test]
+    fn destinos_de_la_lista() {
+        assert_eq!(destino_valido("plan-final.md"), Some(None));
+        assert_eq!(
+            destino_valido("brainstorm/salida-brainstorm-riesgos-2.md"),
+            Some(Some("peticion-brainstorm-riesgos-2.md".to_owned()))
+        );
+        assert_eq!(
+            destino_valido("brainstorm/salida-brainstorm-rendimiento-10.md"),
+            Some(Some("peticion-brainstorm-rendimiento-10.md".to_owned()))
+        );
+        for malo in [
+            "tarea.md",
+            "../tarea.md",
+            "plan-final.md.bak",
+            "./plan-final.md",
+            "brainstorm/peticion-plan-1.md",
+            "brainstorm/salida-brainstorm-unificador-1.md",
+            "brainstorm/salida-brainstorm-otro-1.md",
+            "brainstorm/salida-brainstorm-riesgos-0.md",
+            "brainstorm/salida-brainstorm-riesgos-01.md",
+            "brainstorm/salida-brainstorm-riesgos-.md",
+            "brainstorm/salida-brainstorm-riesgos-1.txt",
+            "brainstorm/x/salida-brainstorm-riesgos-1.md",
+            "brainstorm\\salida-brainstorm-riesgos-1.md",
+            "brainstorm/salida-brainstorm-riesgos-99999999999.md",
+        ] {
+            assert_eq!(destino_valido(malo), None, "{malo}");
+        }
+    }
+
+    #[test]
+    fn guarda_solo_en_diseno_con_peticion() {
+        let md = |estado: &str| {
+            let mut t = Tarea::nueva("TASK-001", "Uno", "2026-11-01");
+            t.estado = estado.to_owned();
+            t.escribir(&Cuerpo::plantilla())
+        };
+        let pet = "TASK-001/planificacion/brainstorm/peticion-brainstorm-riesgos-1.md";
+        let e = EnMemoria(BTreeMap::from([
+            ("TASK-001/tarea.md".to_owned(), md("en-diseno")),
+            (pet.to_owned(), "p".to_owned()),
+            (
+                "TASK-001/planificacion/plan-final.md".to_owned(),
+                "viejo\n".to_owned(),
+            ),
+            (
+                "TASK-002/tarea.md".to_owned(),
+                md("planificada").replace("TASK-001", "TASK-002"),
+            ),
+        ]));
+        let g =
+            |id: &str, d: &str, c: &str| Guardar::nueva(id, &ana(), d, c).map(|o| o.aplicar(&e));
+        let ok = g("TASK-001", "plan-final.md", "a\r\nb\n");
+        assert_eq!(
+            ok,
+            Some(Ok(vec![Cambio::nuevo(
+                "TASK-001/planificacion/plan-final.md",
+                "a\nb\n"
+            )]))
+        );
+        let s = g("TASK-001", "brainstorm/salida-brainstorm-riesgos-1.md", "x");
+        assert_eq!(s.map(|r| r.map(|c| c.len())), Some(Ok(1)));
+        let motivo = |r: Option<Result<Vec<Cambio>, Fallo>>| match r {
+            Some(Err(Fallo::Rechazada { motivo, .. })) => motivo,
+            otro => format!("{otro:?}"),
+        };
+        assert_eq!(
+            motivo(g(
+                "TASK-001",
+                "brainstorm/salida-brainstorm-testing-1.md",
+                "x"
+            )),
+            "sin-peticion"
+        );
+        assert_eq!(
+            motivo(g("TASK-002", "plan-final.md", "x")),
+            "estado-no-valido"
+        );
+        assert!(g("TASK-001", "tarea.md", "x").is_none());
+        // Lo mismo que ya esta: nada que escribir.
+        assert_eq!(
+            g("TASK-001", "plan-final.md", "viejo\r\n"),
+            Some(Ok(Vec::new()))
+        );
+        assert!(
+            g(
+                "TASK-001",
+                "brainstorm/peticion-brainstorm-riesgos-1.md",
+                "p"
+            )
+            .is_none()
+        );
+    }
+}
diff --git a/crates/tc2/src/plan/heuristica.rs b/crates/tc2/src/plan/heuristica.rs
new file mode 100644
index 0000000..ab589e2
--- /dev/null
+++ b/crates/tc2/src/plan/heuristica.rs
@@ -0,0 +1,423 @@
+//! Heuristica de complejidad de la 0.16 (`scripts/heuristica-complejidad.yml`
+//! y `src/core/heuristica.ts` del plugin 0.16.2) y validacion del enunciado
+//! que bloquea `plan` (F4-1, D5).
+//!
+//! Aritmetica sobre lo que ya esta en la tarea (etiquetas, tipo,
+//! dependencias, criterios y palabras de riesgo): ninguna llamada a un
+//! modelo. Los pesos van como constantes, no en un YAML: la configuracion
+//! del proyecto solo puede subir roles (`politica_carriles`), nunca bajar
+//! estos numeros.
+//!
+//! De la validacion de la 0.16 (`validacion-tarea.ts`) se queda lo que
+//! bloquea por forma: objetivo vacio, ningun criterio, un criterio vacio y
+//! mas de [`MAX_CRITERIOS`]. No se avisa de palabras vagas ni de varios
+//! frentes (plan de TASK-036).
+
+use crate::tarea::Cuerpo;
+use crate::tarea::texto::plegar;
+
+/// Cada etiqueta mas alla de la primera.
+pub const PESO_ETIQUETA_ADICIONAL: u32 = 1;
+/// Cada palabra de alto riesgo distinta del objetivo o los criterios.
+pub const PESO_PALABRA_ALTO_RIESGO: u32 = 2;
+/// Cada dependencia declarada.
+pub const PESO_DEPENDENCIA: u32 = 1;
+/// Tipo `release`.
+pub const PESO_TIPO_RELEASE: u32 = 1;
+/// Tipo `hotfix` con alguna palabra de riesgo.
+pub const PESO_TIPO_HOTFIX_CON_PALABRA_RIESGO: u32 = 2;
+/// [`UMBRAL_CRITERIOS`] criterios o mas, una sola vez.
+pub const PESO_CRITERIOS: u32 = 1;
+/// Criterios a partir de los que se suma [`PESO_CRITERIOS`].
+pub const UMBRAL_CRITERIOS: usize = 5;
+
+/// Ultimo valor de cada nivel: trivial, simple, media y alta; a partir del
+/// siguiente, critica (cortes 0/3/5/7, critica desde 8).
+pub const CORTES: [u32; 4] = [0, 3, 5, 7];
+
+/// Niveles de complejidad, de menor a mayor (los de la 0.16).
+pub const NIVELES: [&str; 5] = crate::tarea::COMPLEJIDADES;
+
+/// Roles por nivel, en el orden de [`NIVELES`] (0-1-2-3-4).
+pub const ROLES_POR_NIVEL: [u8; 5] = [0, 1, 2, 3, 4];
+
+/// Roles de un `hotfix` como mucho (tope, no sustitucion).
+pub const ROLES_HOTFIX: u8 = 1;
+
+/// Las 14 palabras de alto riesgo, en minusculas y sin tildes; se comparan
+/// por subcadena sobre el texto plegado.
+pub const PALABRAS_ALTO_RIESGO: [&str; 14] = [
+    "migracion",
+    "breaking change",
+    "seguridad",
+    "autenticacion",
+    "autorizacion",
+    "base de datos",
+    "rollback",
+    "cifrado",
+    "datos personales",
+    "concurrencia",
+    "rendimiento",
+    "compatibilidad hacia atras",
+    "integracion externa",
+    "irreversible",
+];
+
+/// Mas criterios que estos bloquea: la tarea es demasiado grande.
+pub const MAX_CRITERIOS: usize = 12;
+
+/// Titulo de la seccion del objetivo.
+const OBJETIVO: &str = "Objetivo";
+/// Titulo de la seccion de los criterios.
+const CRITERIOS: &str = "Criterios de aceptacion";
+
+/// El objetivo y los criterios de una tarea.
+#[derive(Debug, Clone, PartialEq, Eq, Default)]
+pub struct Enunciado {
+    /// Texto de `## Objetivo`, sin el titulo y recortado.
+    pub objetivo: String,
+    /// Cada casilla `- [ ]` (o `- [x]`) de `## Criterios de aceptacion`.
+    pub criterios: Vec<String>,
+}
+
+/// Un hueco de la plantilla (`<que se quiere y por que>`): cuenta como
+/// vacio.
+fn hueco(s: &str) -> bool {
+    s.starts_with('<') && s.ends_with('>') && !s.contains('\n')
+}
+
+impl Enunciado {
+    /// El enunciado del cuerpo de una tarea. Los huecos de la plantilla de
+    /// F1-9 cuentan como vacios.
+    pub fn de(cuerpo: &Cuerpo) -> Self {
+        let sin_titulo = |t: &str| -> String {
+            t.split_once('\n')
+                .map_or("", |(_, resto)| resto)
+                .trim()
+                .to_owned()
+        };
+        let objetivo = cuerpo
+            .seccion(OBJETIVO)
+            .map(|s| sin_titulo(&s.texto))
+            .filter(|o| !hueco(o))
+            .unwrap_or_default();
+        let criterios = cuerpo.seccion(CRITERIOS).map_or_else(Vec::new, |s| {
+            s.texto
+                .lines()
+                .filter_map(|l| {
+                    ["- [ ]", "- [x]", "- [X]"]
+                        .iter()
+                        .find_map(|p| l.strip_prefix(p))
+                })
+                .map(|c| {
+                    let c = c.trim();
+                    if hueco(c) {
+                        String::new()
+                    } else {
+                        c.to_owned()
+                    }
+                })
+                .collect()
+        });
+        Self {
+            objetivo,
+            criterios,
+        }
+    }
+
+    /// Lo que bloquea `plan`: el `motivo=` del rechazo y, para un criterio
+    /// vacio, su numero (desde 1); `Ok` si el enunciado vale.
+    pub fn validar(&self) -> Result<(), (&'static str, usize)> {
+        if self.objetivo.trim().is_empty() {
+            return Err(("objetivo-vacio", 0));
+        }
+        if self.criterios.is_empty() {
+            return Err(("sin-criterios", 0));
+        }
+        if let Some(i) = self.criterios.iter().position(|c| c.trim().is_empty()) {
+            return Err(("criterio-vacio", i + 1));
+        }
+        if self.criterios.len() > MAX_CRITERIOS {
+            return Err(("demasiados-criterios", self.criterios.len()));
+        }
+        Ok(())
+    }
+}
+
+/// Una senal encontrada: clave, detalle y puntos.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Senal {
+    /// Clave de la senal (las de la 0.16).
+    pub clave: &'static str,
+    /// De donde sale.
+    pub detalle: String,
+    /// Puntos que suma.
+    pub puntos: u32,
+}
+
+/// Complejidad declarada frente a heuristica y los roles que piden.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Resolucion {
+    /// Roles por complejidad: el mayor de los dos niveles (con el tope de
+    /// `hotfix` aplicado a cada uno).
+    pub roles: u8,
+    /// Nivel declarado; `None` si la tarea no lo declara (decide la
+    /// heuristica).
+    pub declarado: Option<String>,
+    /// Nivel de la heuristica.
+    pub heuristico: &'static str,
+    /// Puntos.
+    pub puntos: u32,
+    /// Senales.
+    pub senales: Vec<Senal>,
+    /// Si el tope de `hotfix` recorto el numero.
+    pub tope_hotfix: bool,
+}
+
+impl Resolucion {
+    /// Los dos niveles difieren.
+    pub fn discrepa(&self) -> bool {
+        self.declarado
+            .as_deref()
+            .is_some_and(|d| d != self.heuristico)
+    }
+}
+
+/// Las palabras de riesgo distintas que aparecen en el texto, en el orden
+/// de la lista: se recorre la lista, no el texto (una vez por entrada).
+fn palabras(e: &Enunciado) -> Vec<&'static str> {
+    let mut texto = e.objetivo.clone();
+    for c in &e.criterios {
+        texto.push('\n');
+        texto.push_str(c);
+    }
+    let texto = format!(" {} ", plegar(&texto));
+    PALABRAS_ALTO_RIESGO
+        .into_iter()
+        .filter(|p| texto.contains(&plegar(p)))
+        .collect()
+}
+
+/// Puntos y senales de una tarea.
+pub fn puntuar(
+    etiquetas: &[String],
+    tipo: &str,
+    dependencias: &[String],
+    e: &Enunciado,
+) -> (u32, Vec<Senal>) {
+    let mut s = Vec::new();
+    let cuantos = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
+    let adicionales = etiquetas.len().saturating_sub(1);
+    if adicionales > 0 {
+        s.push(Senal {
+            clave: "etiquetas_adicionales",
+            detalle: format!(
+                "{} etiquetas ({adicionales} mas alla de la primera)",
+                etiquetas.len()
+            ),
+            puntos: cuantos(adicionales).saturating_mul(PESO_ETIQUETA_ADICIONAL),
+        });
+    }
+    let encontradas = palabras(e);
+    for p in &encontradas {
+        s.push(Senal {
+            clave: "palabra_alto_riesgo",
+            detalle: (*p).to_owned(),
+            puntos: PESO_PALABRA_ALTO_RIESGO,
+        });
+    }
+    if !dependencias.is_empty() {
+        s.push(Senal {
+            clave: "dependencias",
+            detalle: format!(
+                "{} dependencias ({})",
+                dependencias.len(),
+                dependencias.join(", ")
+            ),
+            puntos: cuantos(dependencias.len()).saturating_mul(PESO_DEPENDENCIA),
+        });
+    }
+    if tipo == "release" {
+        s.push(Senal {
+            clave: "tipo_release",
+            detalle: "tipo release".to_owned(),
+            puntos: PESO_TIPO_RELEASE,
+        });
+    }
+    if tipo == "hotfix" && !encontradas.is_empty() {
+        s.push(Senal {
+            clave: "tipo_hotfix_con_palabra_riesgo",
+            detalle: format!("hotfix que toca {}", encontradas.join(", ")),
+            puntos: PESO_TIPO_HOTFIX_CON_PALABRA_RIESGO,
+        });
+    }
+    if e.criterios.len() >= UMBRAL_CRITERIOS {
+        s.push(Senal {
+            clave: "criterios_aceptacion",
+            detalle: format!(
+                "{} criterios (umbral: {UMBRAL_CRITERIOS})",
+                e.criterios.len()
+            ),
+            puntos: PESO_CRITERIOS,
+        });
+    }
+    let puntos = s.iter().map(|x| x.puntos).fold(0, u32::saturating_add);
+    (puntos, s)
+}
+
+/// El nivel de una puntuacion: cada corte es el ultimo valor del nivel.
+pub fn nivel(puntos: u32) -> &'static str {
+    let i = CORTES
+        .iter()
+        .position(|c| puntos <= *c)
+        .unwrap_or(CORTES.len());
+    NIVELES.get(i).copied().unwrap_or("critica")
+}
+
+/// Roles que pide un nivel sin el tope de `hotfix`.
+fn por_nivel(nivel: &str) -> u8 {
+    NIVELES
+        .iter()
+        .position(|n| *n == nivel)
+        .and_then(|i| ROLES_POR_NIVEL.get(i).copied())
+        .unwrap_or(0)
+}
+
+/// Roles que pide un nivel para un tipo: el tope de `hotfix` es un minimo.
+fn con_tope(nivel: &str, tipo: &str) -> u8 {
+    let n = por_nivel(nivel);
+    if tipo == "hotfix" {
+        n.min(ROLES_HOTFIX)
+    } else {
+        n
+    }
+}
+
+/// La resolucion de la 0.16 (`resolverNumeroAgentes`): sin complejidad
+/// declarada decide solo la heuristica; con ella, el mayor de los dos.
+pub fn resolver(
+    complejidad: Option<&str>,
+    etiquetas: &[String],
+    tipo: &str,
+    dependencias: &[String],
+    e: &Enunciado,
+) -> Resolucion {
+    let (puntos, senales) = puntuar(etiquetas, tipo, dependencias, e);
+    let heuristico = nivel(puntos);
+    let niveles: Vec<&str> = complejidad
+        .into_iter()
+        .filter(|c| NIVELES.contains(c))
+        .chain([heuristico])
+        .collect();
+    let roles = niveles.iter().map(|n| con_tope(n, tipo)).max().unwrap_or(0);
+    let sin_tope = niveles.iter().map(|n| por_nivel(n)).max().unwrap_or(0);
+    Resolucion {
+        roles,
+        declarado: complejidad.map(str::to_owned),
+        heuristico,
+        puntos,
+        senales,
+        tope_hotfix: roles < sin_tope,
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn e(objetivo: &str, criterios: &[&str]) -> Enunciado {
+        Enunciado {
+            objetivo: objetivo.to_owned(),
+            criterios: criterios.iter().map(|c| (*c).to_owned()).collect(),
+        }
+    }
+
+    fn v(x: &[&str]) -> Vec<String> {
+        x.iter().map(|s| (*s).to_owned()).collect()
+    }
+
+    #[test]
+    fn niveles_por_cortes() {
+        for (p, n) in [
+            (0, "trivial"),
+            (1, "simple"),
+            (3, "simple"),
+            (4, "media"),
+            (5, "media"),
+            (6, "alta"),
+            (7, "alta"),
+            (8, "critica"),
+            (40, "critica"),
+        ] {
+            assert_eq!(nivel(p), n, "{p}");
+        }
+    }
+
+    /// Cada senal con sus puntos; una palabra cuenta una vez aunque salga
+    /// en el objetivo y en los criterios, con tildes o mayusculas.
+    #[test]
+    fn senales_y_puntos() {
+        let en = e(
+            "Una Migración de la base de datos",
+            &["la migracion no rompe", "b", "c", "d", "e"],
+        );
+        let (p, s) = puntuar(&v(&["a", "b", "c"]), "hotfix", &v(&["TASK-001"]), &en);
+        let claves: Vec<(&str, u32)> = s.iter().map(|x| (x.clave, x.puntos)).collect();
+        assert_eq!(
+            claves,
+            [
+                ("etiquetas_adicionales", 2),
+                ("palabra_alto_riesgo", 2),
+                ("palabra_alto_riesgo", 2),
+                ("dependencias", 1),
+                ("tipo_hotfix_con_palabra_riesgo", 2),
+                ("criterios_aceptacion", 1),
+            ]
+        );
+        assert_eq!(p, 10);
+        let (p, s) = puntuar(&v(&["a"]), "release", &[], &e("x", &["y"]));
+        assert_eq!((p, s.len()), (1, 1));
+        assert_eq!(puntuar(&[], "feature", &[], &e("x", &["y"])).0, 0);
+    }
+
+    /// La resolucion: el mayor de declarado y heuristico; sin declarar,
+    /// solo la heuristica; el tope de hotfix recorta y lo dice.
+    #[test]
+    fn resolucion() {
+        let simple = e("x", &["y"]);
+        let r = resolver(None, &[], "feature", &[], &simple);
+        assert_eq!((r.roles, r.heuristico, r.discrepa()), (0, "trivial", false));
+        let r = resolver(Some("alta"), &[], "feature", &[], &simple);
+        assert_eq!((r.roles, r.discrepa(), r.tope_hotfix), (3, true, false));
+        let r = resolver(Some("critica"), &[], "hotfix", &[], &simple);
+        assert_eq!((r.roles, r.tope_hotfix), (1, true));
+        let r = resolver(Some("trivial"), &[], "hotfix", &[], &simple);
+        assert_eq!((r.roles, r.tope_hotfix), (0, false));
+        let riesgo = e("seguridad y cifrado y rollback", &["y"]);
+        let r = resolver(Some("simple"), &[], "feature", &[], &riesgo);
+        assert_eq!((r.roles, r.heuristico, r.puntos), (3, "alta", 6));
+    }
+
+    /// La validacion que bloquea, con los huecos de la plantilla como
+    /// vacios.
+    #[test]
+    fn enunciado_y_validacion() {
+        let c = Cuerpo::con_enunciado("Que haga algo", &v(&["uno", "dos"]));
+        let en = Enunciado::de(&c);
+        assert_eq!(en, e("Que haga algo", &["uno", "dos"]));
+        assert_eq!(en.validar(), Ok(()));
+        let plantilla = Enunciado::de(&Cuerpo::plantilla());
+        assert_eq!(plantilla, e("", &[""]));
+        assert_eq!(plantilla.validar(), Err(("objetivo-vacio", 0)));
+        assert_eq!(e("x", &[]).validar(), Err(("sin-criterios", 0)));
+        assert_eq!(e("x", &["a", " "]).validar(), Err(("criterio-vacio", 2)));
+        let doce: Vec<String> = (1..=12).map(|i| format!("c{i}")).collect();
+        let trece: Vec<String> = (1..=13).map(|i| format!("c{i}")).collect();
+        let con = |c: &[String]| Enunciado {
+            objetivo: "x".into(),
+            criterios: c.to_vec(),
+        };
+        assert_eq!(con(&doce).validar(), Ok(()));
+        assert_eq!(con(&trece).validar(), Err(("demasiados-criterios", 13)));
+    }
+}
diff --git a/crates/tc2/src/plan/mod.rs b/crates/tc2/src/plan/mod.rs
new file mode 100644
index 0000000..71f2c08
--- /dev/null
+++ b/crates/tc2/src/plan/mod.rs
@@ -0,0 +1,274 @@
+//! `tc2 plan` y `tc2 plan guardar` (F4-1, RR-02): la fase de diseno con
+//! brainstorm de roles y unificador, proporcional al carril.
+//!
+//! - `plan [--json] TASK-NNN [--carril <c>]`: `fetch` (exige red), quien lo
+//!   lanza (identidad), la configuracion de la base (`modo_flujo` y
+//!   `politica_carriles`; sin base o sin configuracion, el suelo) y la
+//!   operacion [`Planificar`] por el punto unico de escritura
+//!   ([`estado::transaccion`], con el `ROADMAP.md` del mismo commit).
+//!   Publica en la carpeta fija de la tarea, `TASK-NNN/planificacion/`: las
+//!   peticiones de la ronda en `brainstorm/` y el esqueleto de
+//!   `plan-final.md` si no existe; la tarea queda `en-diseno` con el carril
+//!   y el modo congelados. Linea: `plan ok id= commit= ... carril= modo=
+//!   ronda= roles= forma=redaccion|unificador peticiones=`. El CLI no llama
+//!   a ningun modelo (como la 0.16): quien orquesta lanza los agentes.
+//! - `plan guardar [--json] TASK-NNN <destino> <fichero>`: publica un
+//!   fichero (de fuera del repo) como `plan-final.md` o como la salida de un
+//!   rol ([`guardar`]). Destino y fichero se comprueban antes de leer el
+//!   estado (`uso`).
+//!
+//! Lanzadas desde un worktree de tarea escriben el estado en el principal
+//! (lo localiza la transaccion). El entorno es el de `start`
+//! ([`crate::toma::Entorno`]).
+
+pub mod guardar;
+pub mod heuristica;
+pub mod operacion;
+pub mod plantillas;
+pub mod roles;
+
+use std::path::Path;
+
+use crate::carril::Carril;
+use crate::config::Config;
+use crate::contrato::{Informe, Resultado, Valor};
+use crate::estado::{self, Operacion};
+use crate::git::{self, Git};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::plan as txt;
+use crate::toma::Entorno;
+
+pub use guardar::{Guardar, destino_valido};
+pub use operacion::{Forma, Planificar, Resumen};
+
+/// Nombre de la orden y de la operacion de `plan` (y de su fila).
+pub const NOMBRE: &str = "plan";
+
+/// Subcomando que publica un fichero en la planificacion.
+pub const GUARDAR: &str = "guardar";
+
+fn uso(motivo: &str) -> Informe {
+    Informe::nuevo(NOMBRE, Resultado::Uso)
+        .con("motivo", motivo)
+        .diagnostico(&txt::USO_PLAN)
+}
+
+/// `plan ...` o `plan guardar ...` (sin `--json`, que quita `lib.rs`). Nada
+/// se lee ni se escribe antes de comprobar los argumentos.
+pub fn orden(args: &[String], e: &Entorno<'_>) -> Informe {
+    match args.split_first() {
+        Some((g, resto)) if g == GUARDAR => orden_guardar(resto, e),
+        _ => orden_plan(args, e),
+    }
+}
+
+fn orden_plan(args: &[String], e: &Entorno<'_>) -> Informe {
+    let Some((id, resto)) = args.split_first() else {
+        return uso("sin-id");
+    };
+    if !git::es_id(id) {
+        return uso("id-invalido");
+    }
+    let mut carril = None;
+    let mut it = resto.iter();
+    while let Some(a) = it.next() {
+        match a.as_str() {
+            "--carril" if carril.is_none() => match it.next().and_then(|c| Carril::desde(c)) {
+                Some(c) => carril = Some(c),
+                None => return uso("carril-invalido"),
+            },
+            _ => return uso("argumentos-de-mas"),
+        }
+    }
+    plan_en(e, id, carril)
+}
+
+fn orden_guardar(args: &[String], e: &Entorno<'_>) -> Informe {
+    let (id, destino, fichero) = match args {
+        [] => return uso("sin-id"),
+        [id, ..] if !git::es_id(id) => return uso("id-invalido"),
+        [_] => return uso("sin-destino"),
+        [_, _] => return uso("sin-fichero"),
+        [id, d, f] => (id, d, f),
+        _ => return uso("argumentos-de-mas"),
+    };
+    if destino_valido(destino).is_none() {
+        let nombres: Vec<&str> = roles::todos().map(|r| r.id).collect();
+        return Informe::nuevo(NOMBRE, Resultado::Uso)
+            .con("motivo", "destino-invalido")
+            .diagnostico(
+                Mensaje::nuevo(&txt::DESTINO_INVALIDO)
+                    .con("destino", crate::plataforma::errores::sanear(destino))
+                    .con("roles", nombres.join(", ")),
+            );
+    }
+    let contenido = match leer_fichero(&e.dir.join(fichero)) {
+        Ok(c) => c,
+        Err(i) => return i,
+    };
+    guardar_en(e, id, destino, &contenido)
+}
+
+/// Lee el fichero a publicar sin cargar mas del tope: UTF-8, no vacio y de
+/// [`guardar::MAX_BYTES`] como mucho. Un fallo es `uso` (como el fichero de
+/// `import`).
+fn leer_fichero(ruta: &Path) -> Result<String, Informe> {
+    use std::io::Read as _;
+    let mal = |motivo: &str, m: Mensaje| {
+        Informe::nuevo(NOMBRE, Resultado::Uso)
+            .con("motivo", motivo)
+            .diagnostico(m.con("ruta", ruta.display().to_string()))
+    };
+    let no_se_lee = |e: std::io::Error| {
+        mal(
+            "no-se-lee",
+            Mensaje::nuevo(&txt::NO_SE_LEE).con("error", e.to_string()),
+        )
+    };
+    let f = std::fs::File::open(ruta).map_err(no_se_lee)?;
+    let tope = u64::try_from(guardar::MAX_BYTES).unwrap_or(u64::MAX);
+    let mut bytes = Vec::new();
+    f.take(tope.saturating_add(1))
+        .read_to_end(&mut bytes)
+        .map_err(no_se_lee)?;
+    if bytes.len() > guardar::MAX_BYTES {
+        return Err(mal(
+            "grande",
+            Mensaje::nuevo(&txt::GRANDE).con("max", guardar::MAX_BYTES.to_string()),
+        ));
+    }
+    let texto =
+        String::from_utf8(bytes).map_err(|_| mal("no-utf8", Mensaje::nuevo(&txt::NO_UTF8)))?;
+    if texto.trim().is_empty() {
+        return Err(mal("vacio", Mensaje::nuevo(&txt::VACIO)));
+    }
+    Ok(texto)
+}
+
+/// `plan` de la tarea `id` con el carril pedido en el repo de `e`.
+pub fn plan_en(e: &Entorno<'_>, id: &str, carril: Option<Carril>) -> Informe {
+    let i = match paso_plan(e, id, carril) {
+        Ok(i) | Err(i) => i,
+    };
+    de_la_orden(i, id)
+}
+
+/// `plan guardar` de `contenido` en `destino` (ya validado) de la tarea
+/// `id` en el repo de `e`.
+pub fn guardar_en(e: &Entorno<'_>, id: &str, destino: &str, contenido: &str) -> Informe {
+    let i = match paso_guardar(e, id, destino, contenido) {
+        Ok(i) | Err(i) => i,
+    };
+    let mut i = de_la_orden(i, id);
+    i.claves
+        .insert(1, ("destino".to_owned(), Valor::Texto(destino.to_owned())));
+    i
+}
+
+/// `fetch` si hay remoto: `plan` y `plan guardar` exigen red.
+fn traer(g: &Git, dir: &Path) -> Result<(), Informe> {
+    let principal = git::localizar_principal(g, dir)?;
+    if git::estado::hay_remoto(g, &principal, NOMBRE)? {
+        let f = git::fetch(g, &principal);
+        if !f.resultado.es_exito() {
+            return Err(f);
+        }
+    }
+    Ok(())
+}
+
+/// La configuracion de la base (F1-10 decision 3). Sin base o sin
+/// configuracion, el suelo; una que no vale falla con su informe a nombre
+/// de `plan` (como `ciclo::umbral_de_inactividad`).
+pub fn config_de_base(g: &Git, dir: &Path) -> Result<Config, Informe> {
+    match crate::config::de_base(g, dir) {
+        Ok(c) => Ok(c),
+        Err(i) if matches!(i.valor("motivo"), Some("sin-base" | "sin-config")) => {
+            Ok(Config::suelo())
+        }
+        Err(mut i) => {
+            NOMBRE.clone_into(&mut i.comando);
+            Err(i)
+        }
+    }
+}
+
+fn paso_plan(e: &Entorno<'_>, id: &str, carril: Option<Carril>) -> Result<Informe, Informe> {
+    traer(e.git, e.dir)?;
+    let quien = (e.quien)()?;
+    let config = config_de_base(e.git, e.dir)?;
+    let op = Planificar::nueva(id, &quien, carril, &config, e.ahora);
+    let t = escribir(e, &op);
+    match t.resultado {
+        Resultado::Ok | Resultado::SinCambios => Ok(con_resumen(t, op.resumen())),
+        _ => Err(con_dueno(t, op.dueno())),
+    }
+}
+
+fn paso_guardar(
+    e: &Entorno<'_>,
+    id: &str,
+    destino: &str,
+    contenido: &str,
+) -> Result<Informe, Informe> {
+    traer(e.git, e.dir)?;
+    let quien = (e.quien)()?;
+    let Some(op) = Guardar::nueva(id, &quien, destino, contenido) else {
+        return Err(uso("destino-invalido"));
+    };
+    let t = escribir(e, &op);
+    match t.resultado {
+        Resultado::Ok | Resultado::SinCambios => Ok(t),
+        _ => Err(con_dueno(t, op.dueno())),
+    }
+}
+
+/// La operacion por el punto unico de escritura, con el `ROADMAP.md`
+/// regenerado en el mismo commit.
+fn escribir(e: &Entorno<'_>, op: &dyn Operacion) -> Informe {
+    let personas = crate::roadmap::personas(e.git, e.dir);
+    let generar = crate::roadmap::generador(&personas);
+    let ctx = estado::Contexto::nuevo(e.git, e.dir, e.catalogo, Some(&generar));
+    estado::transaccion(&ctx, op)
+}
+
+/// Las claves de lo que decidio `plan`.
+fn con_resumen(i: Informe, r: Option<Resumen>) -> Informe {
+    let Some(r) = r else {
+        return i;
+    };
+    i.con("carril", r.carril.nombre())
+        .con("modo", r.modo)
+        .con("ronda", r.ronda.to_string())
+        .con_lista("roles", r.roles.iter().map(|x| (*x).to_owned()).collect())
+        .con("forma", r.forma.nombre())
+        .con_lista("peticiones", r.peticiones)
+}
+
+/// Un rechazo con lo que sabe la operacion: quien la lleva o el arreglo de
+/// una tarea de la 0.16.
+fn con_dueno(mut i: Informe, dueno: Option<String>) -> Informe {
+    if i.resultado != Resultado::Rechazada {
+        return i;
+    }
+    match (i.valor("motivo"), dueno) {
+        (Some("de-otra-persona"), Some(l)) => i = i.con("asignado", l),
+        (Some("formato-1"), _) => i = i.con("arreglo", "tc2 actualizar"),
+        _ => {}
+    }
+    i
+}
+
+/// El informe como de la orden: el comando, `id=` el primero y sin la clave
+/// `operacion` de la transaccion. Un `sinred` sin `pendiente` dice
+/// `pendiente=no` (exige red).
+fn de_la_orden(mut i: Informe, id: &str) -> Informe {
+    NOMBRE.clone_into(&mut i.comando);
+    i.claves.retain(|(k, _)| k != "operacion" && k != "id");
+    i.claves
+        .insert(0, ("id".to_owned(), Valor::Texto(id.to_owned())));
+    if i.resultado == Resultado::Sinred && i.valor("pendiente").is_none() {
+        i = i.con("pendiente", "no");
+    }
+    i
+}
diff --git a/crates/tc2/src/plan/operacion.rs b/crates/tc2/src/plan/operacion.rs
new file mode 100644
index 0000000..815c4c1
--- /dev/null
+++ b/crates/tc2/src/plan/operacion.rs
@@ -0,0 +1,591 @@
+//! `plan` como operacion de la rama de estado (F4-1). Como
+//! [`crate::toma::Tomar`], pasa por el punto unico de escritura y decide
+//! solo con el estado que recibe (se re-ejecuta en cada intento).
+//!
+//! - Quien: la tarea de otra persona se rechaza (`de-otra-persona`, o
+//!   `asignado-heredado`); estados de origen `planificada` y `en-diseno`; en
+//!   pausa, `pausada`.
+//! - El enunciado tiene que valer ([`Enunciado::validar`]): si no, se
+//!   rechaza con su motivo sin escribir nada.
+//! - Carril: el mayor de [`Carril::declarado`] por el tipo, el ya congelado
+//!   en la tarea y el pedido con `--carril` (D4); pedir uno por debajo se
+//!   rechaza (`carril-baja`). Queda congelado en `carril:`.
+//! - Modo: el de la ultima fila `plan` de `## Transiciones` si la hay (ya
+//!   congelado); si no, `modo_flujo` de la configuracion de la base. Va en
+//!   la columna `modo` de la fila `plan` (F1-10, decision 5).
+//! - Roles: [`Carril::roles_diseno`] con la complejidad
+//!   ([`heuristica::resolver`]) y [`roles::seleccionar`]; un rol es la forma
+//!   `redaccion` (`peticion-plan-<n>.md`), el resto `unificador` (una
+//!   peticion por rol y `peticion-unificador-<n>.md`).
+//! - Ronda: la mayor con testigo (`peticion-unificador-<n>.md` o
+//!   `peticion-plan-<n>.md`) en `TASK-NNN/planificacion/brainstorm/`. Si sus
+//!   peticiones son exactamente las que tocan ahora, la ronda sigue abierta
+//!   y no se reescribe; si no (no hay ninguna, o cambio el juego de roles),
+//!   se abre la siguiente. Las rondas solo se anaden: la rama de estado no
+//!   borra ficheros.
+//! - `plan-final.md`: el esqueleto de F1-9 solo si no existe; uno que ya
+//!   existe no se pisa.
+//! - `tarea.md`: `estado: en-diseno`, `carril:`, `actualizado` y la fila
+//!   `plan`, solo si algo cambia. Repetir `plan` sobre una ronda abierta con
+//!   el mismo carril no cambia nada (`sin-cambios`).
+
+use std::cell::RefCell;
+
+use super::NOMBRE;
+use super::heuristica::{self, Enunciado, MAX_CRITERIOS};
+use super::plantillas::{self as p, Datos};
+use super::roles::{self, Rol};
+use crate::carril::Carril;
+use crate::config::Config;
+use crate::estado::{Cambio, Estado, Fallo, Operacion, de_otra_persona, por_estado, tarea_de};
+use crate::i18n::catalogo::plan as txt;
+use crate::i18n::{Mensaje, Texto};
+use crate::identidad::Quien;
+use crate::tarea::{Asignado, Cuerpo, Transicion};
+use crate::toma::instante;
+
+/// Estados desde los que se planifica.
+pub const PLANIFICABLES: [&str; 2] = ["planificada", "en-diseno"];
+
+/// Estado en el que queda la tarea.
+pub const EN_DISENO: &str = "en-diseno";
+
+/// Modos validos de la columna `modo` (los de `modo_flujo`).
+const MODOS: [&str; 3] = ["manual", "semiautomatico", "automatico"];
+
+/// Como se cierra la ronda.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Forma {
+    /// Un solo rol, sin unificador: su respuesta es el plan.
+    Redaccion,
+    /// Cero, dos o mas roles: consolida el unificador.
+    Unificador,
+}
+
+impl Forma {
+    /// Nombre en la linea.
+    pub fn nombre(self) -> &'static str {
+        match self {
+            Self::Redaccion => "redaccion",
+            Self::Unificador => "unificador",
+        }
+    }
+}
+
+/// Lo que decidio el ultimo intento, para la linea.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Resumen {
+    /// Carril congelado.
+    pub carril: Carril,
+    /// Modo congelado.
+    pub modo: String,
+    /// Ronda abierta (la escrita o la que ya estaba).
+    pub ronda: u32,
+    /// Titulos de los roles lanzados.
+    pub roles: Vec<&'static str>,
+    /// Forma de la ronda.
+    pub forma: Forma,
+    /// Peticiones de la ronda (nombres de fichero de `brainstorm/`).
+    pub peticiones: Vec<String>,
+}
+
+/// Planificar la tarea `id`.
+#[derive(Debug)]
+pub struct Planificar {
+    id: String,
+    asignado: Asignado,
+    decidido_por: &'static str,
+    ahora: u64,
+    carril: Option<Carril>,
+    modo: &'static str,
+    config: Config,
+    resumen: RefCell<Option<Resumen>>,
+    dueno: RefCell<Option<String>>,
+}
+
+impl Planificar {
+    /// `plan` de `id` por `quien` en el instante `ahora`, con el carril
+    /// pedido (si lo hay) y la configuracion de la base.
+    pub fn nueva(
+        id: &str,
+        quien: &Quien,
+        carril: Option<Carril>,
+        config: &Config,
+        ahora: u64,
+    ) -> Self {
+        Self {
+            id: id.to_owned(),
+            asignado: quien.asignado(),
+            decidido_por: quien.decidido_por(),
+            ahora,
+            carril,
+            modo: config.modo_flujo.nombre(),
+            config: config.clone(),
+            resumen: RefCell::new(None),
+            dueno: RefCell::new(None),
+        }
+    }
+
+    /// Lo que decidio el ultimo intento que llego a decidir.
+    pub fn resumen(&self) -> Option<Resumen> {
+        self.resumen.borrow().clone()
+    }
+
+    /// Login de quien lleva la tarea si el ultimo intento la rechazo por
+    /// `de-otra-persona`.
+    pub fn dueno(&self) -> Option<String> {
+        self.dueno.borrow().clone()
+    }
+
+    fn rechazo(&self, motivo: &str, m: Mensaje) -> Fallo {
+        Fallo::rechazada(motivo, m.con("id", self.id.as_str()))
+    }
+
+    /// El rechazo de un enunciado que no vale.
+    fn enunciado_invalido(&self, motivo: &'static str, n: usize) -> Fallo {
+        let t: &'static Texto = match motivo {
+            "objetivo-vacio" => &txt::OBJETIVO_VACIO,
+            "sin-criterios" => &txt::SIN_CRITERIOS,
+            "criterio-vacio" => &txt::CRITERIO_VACIO,
+            _ => &txt::DEMASIADOS_CRITERIOS,
+        };
+        let m = Mensaje::nuevo(t)
+            .con("n", n.to_string())
+            .con("max", MAX_CRITERIOS.to_string());
+        self.rechazo(motivo, m)
+    }
+}
+
+/// El modo de la ultima fila `plan` de `## Transiciones`, si es uno valido.
+pub fn modo_congelado(cuerpo: &Cuerpo) -> Option<String> {
+    let s = cuerpo.seccion("Transiciones")?;
+    s.texto
+        .lines()
+        .rev()
+        .filter_map(|l| {
+            let mut c = l.trim().strip_prefix('|')?.split('|').map(str::trim);
+            let (_fecha, fase, modo) = (c.next()?, c.next()?, c.next()?);
+            (fase == NOMBRE).then_some(modo)
+        })
+        .find(|m| MODOS.contains(m))
+        .map(str::to_owned)
+}
+
+/// Rondas con testigo y nombres por ronda de `TASK-NNN/planificacion/brainstorm/`.
+fn ronda_mayor(rutas: &[String], id: &str) -> (u32, Vec<String>) {
+    let prefijo = p::ruta_brainstorm(id, "");
+    let nombres: Vec<&str> = rutas
+        .iter()
+        .filter_map(|r| r.strip_prefix(&prefijo))
+        .filter(|n| !n.contains('/'))
+        .collect();
+    let numero = |n: &str| -> Option<u32> {
+        let resto = n
+            .strip_prefix("peticion-unificador-")
+            .or_else(|| n.strip_prefix("peticion-plan-"))?
+            .strip_suffix(".md")?;
+        (!resto.is_empty() && resto.bytes().all(|b| b.is_ascii_digit()))
+            .then(|| resto.parse().ok())
+            .flatten()
+    };
+    let r = nombres.iter().filter_map(|n| numero(n)).max().unwrap_or(0);
+    let sufijo = format!("-{r}.md");
+    let mut de_r: Vec<String> = nombres
+        .iter()
+        .filter(|n| n.starts_with("peticion-") && n.ends_with(&sufijo))
+        .map(|n| (*n).to_owned())
+        .collect();
+    de_r.sort();
+    (r, de_r)
+}
+
+/// Las peticiones de una ronda: nombre y contenido, en orden de nombre.
+fn peticiones(d: &Datos<'_>, roles: &[Rol], forma: Forma) -> Vec<(String, String)> {
+    let mut v = match (forma, roles) {
+        (Forma::Redaccion, [rol]) => vec![(
+            p::nombre_peticion_redaccion(d.ronda),
+            p::peticion_redaccion(d, rol),
+        )],
+        _ => {
+            let mut v: Vec<(String, String)> = roles
+                .iter()
+                .map(|rol| {
+                    let otros: Vec<Rol> =
+                        roles.iter().filter(|r| r.id != rol.id).copied().collect();
+                    (
+                        p::nombre_peticion_rol(rol, d.ronda),
+                        p::peticion_rol(d, rol, &otros),
+                    )
+                })
+                .collect();
+            v.push((
+                p::nombre_peticion_unificador(d.ronda),
+                p::peticion_unificador(d, roles),
+            ));
+            v
+        }
+    };
+    v.sort_by(|a, b| a.0.cmp(&b.0));
+    v
+}
+
+impl Operacion for Planificar {
+    fn nombre(&self) -> &str {
+        NOMBRE
+    }
+
+    fn anotacion(&self) -> String {
+        format!(
+            "{NOMBRE}|{}|{}|{}|{}",
+            self.id,
+            self.asignado.id().unwrap_or_default(),
+            self.asignado.login().unwrap_or_default(),
+            self.carril.map_or("-", Carril::nombre)
+        )
+    }
+
+    fn exige_red(&self) -> bool {
+        // Quien orquesta necesita las peticiones ahora: sin red falla, nunca
+        // queda pendiente (plan de TASK-036). Por eso no tiene catalogo.
+        true
+    }
+
+    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
+        *self.resumen.borrow_mut() = None;
+        *self.dueno.borrow_mut() = None;
+        let (mut t, mut cuerpo) = tarea_de(estado, &self.id)?;
+        if let Some((fallo, dueno)) = de_otra_persona(
+            &self.id,
+            t.asignado_a.as_ref(),
+            &self.asignado,
+            NOMBRE,
+            ("de-otra-persona", &txt::DE_OTRA_PERSONA),
+        ) {
+            *self.dueno.borrow_mut() = dueno;
+            return Err(fallo);
+        }
+        if !PLANIFICABLES.contains(&t.estado.as_str()) {
+            return Err(por_estado(&self.id, &t.estado, NOMBRE, &PLANIFICABLES));
+        }
+        if cuerpo.en_pausa() {
+            return Err(self.rechazo("pausada", Mensaje::nuevo(&txt::PAUSADA)));
+        }
+        let enunciado = Enunciado::de(&cuerpo);
+        if let Err((motivo, n)) = enunciado.validar() {
+            return Err(self.enunciado_invalido(motivo, n));
+        }
+        // El carril: solo sube (D4; lo escrito en la tarea no baja nada).
+        let declarado = Carril::declarado(&t.tipo);
+        let minimo = t
+            .carril
+            .as_deref()
+            .and_then(Carril::desde)
+            .map_or(declarado, |c| c.max(declarado));
+        let carril = match self.carril {
+            Some(c) if c < minimo => {
+                return Err(self.rechazo(
+                    "carril-baja",
+                    Mensaje::nuevo(&txt::CARRIL_BAJA)
+                        .con("pedido", c.nombre())
+                        .con("minimo", minimo.nombre()),
+                ));
+            }
+            Some(c) => c,
+            None => minimo,
+        };
+        let resolucion = heuristica::resolver(
+            t.complejidad.as_deref(),
+            &t.etiquetas,
+            &t.tipo,
+            &t.dependencias,
+            &enunciado,
+        );
+        let n = carril.roles_diseno(resolucion.roles, &t.tipo, &self.config);
+        let roles = roles::seleccionar(n, &t.etiquetas);
+        let forma = if roles.len() == 1 {
+            Forma::Redaccion
+        } else {
+            Forma::Unificador
+        };
+        let modo = modo_congelado(&cuerpo).unwrap_or_else(|| self.modo.to_owned());
+        let instante = instante(self.ahora);
+        let fecha = instante.get(..10).unwrap_or_default().to_owned();
+        let rutas = estado.rutas();
+        let (mayor, de_mayor) = ronda_mayor(&rutas, &self.id);
+        let datos = |ronda: u32| Datos {
+            id: &self.id,
+            titulo: &t.titulo,
+            tipo: &t.tipo,
+            complejidad: t.complejidad.as_deref(),
+            carril,
+            ronda,
+            fecha: &fecha,
+            enunciado: &enunciado,
+            resolucion: &resolucion,
+        };
+        let nombres = |ronda: u32| -> Vec<String> {
+            peticiones(&datos(ronda), &roles, forma)
+                .into_iter()
+                .map(|(n, _)| n)
+                .collect()
+        };
+        let abierta = mayor >= 1 && nombres(mayor) == de_mayor;
+        let ronda = if abierta { mayor } else { mayor + 1 };
+        let mut cambios = Vec::new();
+        if !abierta {
+            for (nombre, texto) in peticiones(&datos(ronda), &roles, forma) {
+                cambios.push(Cambio::nuevo(p::ruta_brainstorm(&self.id, &nombre), texto));
+            }
+        }
+        let plan = p::ruta_plan_final(&self.id);
+        if !rutas.contains(&plan) {
+            cambios.push(Cambio::nuevo(
+                plan,
+                p::plan_final(&self.id, &t.titulo, &roles),
+            ));
+        }
+        *self.resumen.borrow_mut() = Some(Resumen {
+            carril,
+            modo: modo.clone(),
+            ronda,
+            roles: roles.iter().map(|r| r.titulo).collect(),
+            forma,
+            peticiones: nombres(ronda),
+        });
+        let cambia_tarea = t.estado != EN_DISENO
+            || t.carril.as_deref() != Some(carril.nombre())
+            || !cambios.is_empty();
+        if cambia_tarea {
+            EN_DISENO.clone_into(&mut t.estado);
+            t.carril = Some(carril.nombre().to_owned());
+            fecha.clone_into(&mut t.actualizado);
+            cuerpo.anadir_transicion(&Transicion {
+                fecha: &instante,
+                fase: NOMBRE,
+                modo: &modo,
+                decidido_por: self.decidido_por,
+                version: crate::VERSION,
+            });
+            cambios.push(Cambio::nuevo(
+                format!("{}/tarea.md", self.id),
+                t.escribir(&cuerpo),
+            ));
+        }
+        Ok(cambios)
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::config::ModoFlujo;
+    use crate::identidad::Cuenta;
+    use crate::plataforma::Proveedor;
+    use crate::roadmap::tests::EnMemoria;
+    use crate::tarea::Tarea;
+    use std::collections::BTreeMap;
+
+    /// 2026-11-17T10:20:30Z.
+    const AHORA: u64 = 1_794_910_830;
+
+    fn ana() -> Quien {
+        Quien::Agente(Cuenta {
+            proveedor: Proveedor::GitHub,
+            id: 7,
+            login: "ana".to_owned(),
+        })
+    }
+
+    fn md(tipo: &str, estado: &str, criterios: &[&str]) -> String {
+        let mut t = Tarea::nueva("TASK-001", "Uno", "2026-11-01");
+        t.tipo = tipo.to_owned();
+        t.estado = estado.to_owned();
+        let c: Vec<String> = criterios.iter().map(|x| (*x).to_owned()).collect();
+        t.escribir(&Cuerpo::con_enunciado("Que haga algo", &c))
+    }
+
+    fn estado(ficheros: &[(&str, String)]) -> EnMemoria {
+        EnMemoria(
+            ficheros
+                .iter()
+                .map(|(r, t)| ((*r).to_owned(), t.clone()))
+                .collect::<BTreeMap<_, _>>(),
+        )
+    }
+
+    fn rutas(r: &Result<Vec<Cambio>, Fallo>) -> Vec<String> {
+        r.as_ref()
+            .map(|c| c.iter().map(|x| x.ruta.clone()).collect())
+            .unwrap_or_default()
+    }
+
+    fn motivo(r: &Result<Vec<Cambio>, Fallo>) -> Option<&str> {
+        match r {
+            Err(Fallo::Rechazada { motivo, .. }) => Some(motivo),
+            _ => None,
+        }
+    }
+
+    /// Criterio 2 (unitario): uno en fix (solo la peticion de redaccion),
+    /// los cuatro y el unificador con `--carril riesgo-alto`; el carril y
+    /// el modo de la configuracion quedan en la tarea.
+    #[test]
+    fn uno_en_fix_y_todos_en_riesgo_alto() {
+        let e = estado(&[("TASK-001/tarea.md", md("fix", "planificada", &["a"]))]);
+        let mut c = Config::suelo();
+        c.modo_flujo = ModoFlujo::Automatico;
+        let op = Planificar::nueva("TASK-001", &ana(), None, &c, AHORA);
+        let r = op.aplicar(&e);
+        assert_eq!(
+            rutas(&r),
+            [
+                "TASK-001/planificacion/brainstorm/peticion-plan-1.md",
+                "TASK-001/planificacion/plan-final.md",
+                "TASK-001/tarea.md",
+            ]
+        );
+        let s = op.resumen();
+        assert_eq!(
+            s.as_ref()
+                .map(|s| (s.carril, s.modo.as_str(), s.forma, s.roles.clone())),
+            Some((
+                Carril::Fix,
+                "automatico",
+                Forma::Redaccion,
+                vec!["arquitectura"]
+            ))
+        );
+        let tarea = r
+            .as_ref()
+            .ok()
+            .and_then(|c| c.last())
+            .map(|c| c.contenido.clone())
+            .unwrap_or_default();
+        assert!(
+            tarea.contains("\ncarril: \"fix\"\nestado: \"en-diseno\"\n"),
+            "{tarea}"
+        );
+        assert!(tarea.ends_with(&format!(
+            "| 2026-11-17T10:20:30Z | plan | automatico | agente | {} |\n",
+            crate::VERSION
+        )));
+        let op = Planificar::nueva("TASK-001", &ana(), Some(Carril::RiesgoAlto), &c, AHORA);
+        let r = op.aplicar(&e);
+        let b = "TASK-001/planificacion/brainstorm/";
+        assert_eq!(
+            rutas(&r),
+            [
+                format!("{b}peticion-brainstorm-arquitectura-1.md"),
+                format!("{b}peticion-brainstorm-dominio-1.md"),
+                format!("{b}peticion-brainstorm-riesgos-1.md"),
+                format!("{b}peticion-brainstorm-testing-1.md"),
+                format!("{b}peticion-unificador-1.md"),
+                "TASK-001/planificacion/plan-final.md".to_owned(),
+                "TASK-001/tarea.md".to_owned(),
+            ]
+        );
+    }
+
+    /// Repetir sobre la ronda abierta no cambia nada; subir el carril abre
+    /// la ronda 2 sin pisar el plan; el modo congelado gana a la
+    /// configuracion; bajar el carril se rechaza.
+    #[test]
+    fn rondas_y_carril() {
+        let c = Config::suelo();
+        let mut e = estado(&[("TASK-001/tarea.md", md("fix", "planificada", &["a"]))]);
+        let aplicar = |e: &mut EnMemoria, carril: Option<Carril>, c: &Config| {
+            let op = Planificar::nueva("TASK-001", &ana(), carril, c, AHORA);
+            let r = op.aplicar(e);
+            for x in r.as_ref().map(Vec::as_slice).unwrap_or_default() {
+                e.0.insert(x.ruta.clone(), x.contenido.clone());
+            }
+            (r, op.resumen())
+        };
+        let (r, _) = aplicar(&mut e, None, &c);
+        assert_eq!(rutas(&r).len(), 3);
+        let (r, s) = aplicar(&mut e, None, &c);
+        assert_eq!(r, Ok(Vec::new()));
+        assert_eq!(
+            s.map(|s| (s.ronda, s.peticiones)),
+            Some((1, vec!["peticion-plan-1.md".to_owned()]))
+        );
+        // La configuracion cambia de modo: el congelado manda.
+        let mut auto = Config::suelo();
+        auto.modo_flujo = ModoFlujo::Automatico;
+        let (r, s) = aplicar(&mut e, Some(Carril::Funcionalidad), &auto);
+        // Mismo juego de roles (uno): sigue la ronda 1, cambia el carril.
+        assert_eq!(rutas(&r), ["TASK-001/tarea.md"]);
+        assert_eq!(s.map(|s| (s.ronda, s.modo)), Some((1, "manual".to_owned())));
+        e.0.insert(
+            "TASK-001/planificacion/plan-final.md".into(),
+            "mi plan\n".into(),
+        );
+        let (r, s) = aplicar(&mut e, Some(Carril::RiesgoAlto), &c);
+        let r = rutas(&r);
+        assert_eq!(r.len(), 6, "{r:?}");
+        assert!(
+            r.contains(&"TASK-001/planificacion/brainstorm/peticion-unificador-2.md".to_owned())
+        );
+        assert!(!r.contains(&"TASK-001/planificacion/plan-final.md".to_owned()));
+        assert_eq!(s.map(|s| s.ronda), Some(2));
+        let (r, _) = aplicar(&mut e, Some(Carril::Fix), &c);
+        assert_eq!(motivo(&r), Some("carril-baja"));
+        assert_eq!(
+            e.0.get("TASK-001/planificacion/plan-final.md")
+                .map(String::as_str),
+            Some("mi plan\n")
+        );
+    }
+
+    /// Lo que no se planifica: enunciado que no vale, estado, otra persona.
+    #[test]
+    fn rechazos() {
+        let c = Config::suelo();
+        let mut sin_objetivo = Tarea::nueva("TASK-002", "Dos", "2026-11-01");
+        sin_objetivo.estado = "planificada".into();
+        let otra = md("feature", "planificada", &["a"])
+            .replace("asignado_a: null", "asignado_a: {id: 9, login: \"beto\"}");
+        let e = estado(&[
+            ("TASK-001/tarea.md", md("feature", "en-curso", &["a"])),
+            (
+                "TASK-002/tarea.md",
+                sin_objetivo.escribir(&Cuerpo::plantilla()),
+            ),
+            ("TASK-003/tarea.md", otra.replace("TASK-001", "TASK-003")),
+            (
+                "TASK-004/tarea.md",
+                md("feature", "planificada", &[]).replace("TASK-001", "TASK-004"),
+            ),
+        ]);
+        for (id, esperado) in [
+            ("TASK-001", "estado-no-valido"),
+            ("TASK-002", "objetivo-vacio"),
+            ("TASK-003", "de-otra-persona"),
+            ("TASK-004", "criterio-vacio"),
+            ("TASK-009", "no-existe"),
+        ] {
+            let op = Planificar::nueva(id, &ana(), None, &c, AHORA);
+            assert_eq!(motivo(&op.aplicar(&e)), Some(esperado), "{id}");
+        }
+    }
+
+    #[test]
+    fn modo_de_la_ultima_fila_plan() {
+        let mut c = Cuerpo::plantilla();
+        assert_eq!(modo_congelado(&c), None);
+        for (fase, modo) in [
+            ("plan", "automatico"),
+            ("start", "manual"),
+            ("plan", "raro"),
+        ] {
+            c.anadir_transicion(&Transicion {
+                fecha: "2026-11-17T10:20:30Z",
+                fase,
+                modo,
+                decidido_por: "agente",
+                version: "0.1",
+            });
+        }
+        assert_eq!(modo_congelado(&c).as_deref(), Some("automatico"));
+    }
+}
diff --git a/crates/tc2/src/plan/plantillas.rs b/crates/tc2/src/plan/plantillas.rs
new file mode 100644
index 0000000..d2b7a5e
--- /dev/null
+++ b/crates/tc2/src/plan/plantillas.rs
@@ -0,0 +1,419 @@
+//! Las plantillas de `plan` (F4-1): funciones puras de texto, portadas de
+//! `src/core/plan-brainstorm.ts` del plugin 0.16.2 y del `planTemplate` de
+//! `src/commands/plan.ts`, con la plantilla de `plan-final.md` de F1-9
+//! (punto 7).
+//!
+//! El CLI no llama a ningun modelo: deja las peticiones escritas en la rama
+//! de estado y quien orquesta lanza a los agentes. Lo que cambia frente a la
+//! 0.16:
+//!
+//! - Ningun agente escribe en `tareas/` (con `tareas/` sucio la transaccion
+//!   se rechaza): las salidas y el plan se publican con `tc2 plan guardar`,
+//!   y las peticiones lo dicen.
+//! - `taskctl approve` pasa a `tc2 approve`.
+//! - Cada ronda nueva relanza sus roles (una ronda nueva solo se abre si
+//!   cambia el juego de roles), asi que el bloque de re-planificacion no
+//!   dice que las salidas sean de una ronda anterior.
+//! - El bloque de complejidad dice tambien el carril, que puede subir el
+//!   numero de roles por encima de la complejidad.
+
+use super::heuristica::{Enunciado, Resolucion};
+use super::roles::{Rol, UNIFICADOR};
+use crate::carril::Carril;
+
+/// Carpeta de la planificacion dentro de la carpeta de la tarea.
+pub const PLANIFICACION: &str = "planificacion";
+/// Subcarpeta del brainstorm.
+pub const BRAINSTORM: &str = "brainstorm";
+/// Nombre del plan.
+pub const PLAN_FINAL: &str = "plan-final.md";
+
+/// Lo que comparten las plantillas de una ronda.
+#[derive(Debug, Clone, Copy)]
+pub struct Datos<'a> {
+    /// `TASK-NNN`.
+    pub id: &'a str,
+    /// Titulo de la tarea.
+    pub titulo: &'a str,
+    /// Tipo de la tarea.
+    pub tipo: &'a str,
+    /// Complejidad declarada.
+    pub complejidad: Option<&'a str>,
+    /// Carril con el que se planifica.
+    pub carril: Carril,
+    /// Ronda.
+    pub ronda: u32,
+    /// `AAAA-MM-DD`.
+    pub fecha: &'a str,
+    /// Objetivo y criterios.
+    pub enunciado: &'a Enunciado,
+    /// Complejidad declarada frente a heuristica.
+    pub resolucion: &'a Resolucion,
+}
+
+/// `peticion-<rol>-<ronda>.md` (el id ya lleva `brainstorm-`).
+pub fn nombre_peticion_rol(rol: &Rol, ronda: u32) -> String {
+    format!("peticion-{}-{ronda}.md", rol.id)
+}
+
+/// `salida-<rol>-<ronda>.md`: la respuesta del agente de ese rol.
+pub fn nombre_salida_rol(rol: &Rol, ronda: u32) -> String {
+    format!("salida-{}-{ronda}.md", rol.id)
+}
+
+/// `peticion-unificador-<ronda>.md`.
+pub fn nombre_peticion_unificador(ronda: u32) -> String {
+    format!("peticion-unificador-{ronda}.md")
+}
+
+/// `peticion-plan-<ronda>.md`: con un solo rol no hay unificador y esta
+/// peticion va al propio rol (TASK-042 de la 0.16).
+pub fn nombre_peticion_redaccion(ronda: u32) -> String {
+    format!("peticion-plan-{ronda}.md")
+}
+
+/// `TASK-NNN/planificacion/plan-final.md`.
+pub fn ruta_plan_final(id: &str) -> String {
+    format!("{id}/{PLANIFICACION}/{PLAN_FINAL}")
+}
+
+/// `TASK-NNN/planificacion/brainstorm/<nombre>`.
+pub fn ruta_brainstorm(id: &str, nombre: &str) -> String {
+    format!("{id}/{PLANIFICACION}/{BRAINSTORM}/{nombre}")
+}
+
+/// La orden que publica `destino` (relativo a `planificacion/`).
+fn guardar(id: &str, destino: &str) -> String {
+    format!("tc2 plan guardar {id} {destino} <fichero>")
+}
+
+/// «1 rol», «2 roles», «ningun rol».
+fn plural(n: usize) -> String {
+    match n {
+        0 => "ningun rol".to_owned(),
+        1 => "1 rol".to_owned(),
+        n => format!("{n} roles"),
+    }
+}
+
+/// Valla de comillas invertidas mas larga que cualquier cerca del texto
+/// embebido (`fenceFor` de la 0.16): como poco cuatro.
+fn valla(textos: &[&str]) -> String {
+    let mut max = 3;
+    for t in textos {
+        for l in t.lines() {
+            let sin = l.trim_start_matches(' ');
+            if l.len() - sin.len() <= 3 {
+                max = max.max(sin.chars().take_while(|c| *c == '`').count());
+            }
+        }
+    }
+    "`".repeat(max + 1)
+}
+
+fn cabecera(d: &Datos<'_>) -> String {
+    format!(
+        "- Tarea: {} — {}\n- Tipo: {} · Complejidad declarada: {}\n- Carril: {}\n- Ronda: {}\n- Fecha: {}\n",
+        d.id,
+        d.titulo,
+        d.tipo,
+        d.complejidad
+            .unwrap_or("no declarada (decide la heuristica)"),
+        d.carril.nombre(),
+        d.ronda,
+        d.fecha
+    )
+}
+
+/// El enunciado con valla dinamica: es texto de la persona.
+fn enunciado(e: &Enunciado) -> String {
+    let criterios = if e.criterios.is_empty() {
+        "(la tarea no declara criterios de aceptacion)".to_owned()
+    } else {
+        e.criterios.join("\n")
+    };
+    let f = valla(&[&e.objetivo, &criterios]);
+    format!(
+        "## Enunciado de la tarea\n\n### Objetivo\n\n{f}\n{}\n{f}\n\n### Criterios de aceptacion\n\n{f}\n{criterios}\n{f}\n",
+        e.objetivo
+    )
+}
+
+/// Declarada frente a heuristica, siempre (tambien cuando coinciden), y el
+/// carril, que puede subir el numero.
+fn bloque_complejidad(d: &Datos<'_>, n: usize) -> String {
+    let r = d.resolucion;
+    let senales = if r.senales.is_empty() {
+        "  (ninguna senal encontrada — el texto de la tarea no aporta ninguna)\n".to_owned()
+    } else {
+        r.senales
+            .iter()
+            .map(|s| format!("  - {}: {} (+{})\n", s.clave, s.detalle, s.puntos))
+            .collect()
+    };
+    let veredicto = if r.discrepa() {
+        format!(
+            "Los dos niveles NO coinciden. Por complejidad se lanzan {}, que es el mayor de los dos numeros: infraestimar sale caro y sobreestimar es barato. La discrepancia se te senala aqui a proposito — puede significar que el enunciado de la tarea no refleja lo que de verdad cuesta.",
+            plural(usize::from(r.roles))
+        )
+    } else {
+        format!(
+            "Los dos niveles coinciden. Por complejidad se lanzan {}.",
+            plural(usize::from(r.roles))
+        )
+    };
+    let tope = if r.tope_hotfix {
+        "- Se aplico el tope de `hotfix`: un hotfix abrevia el brainstorm previo (nunca la revision posterior).\n"
+    } else {
+        ""
+    };
+    format!(
+        "## Complejidad: declarada frente a heuristica\n\n- Declarada en la tarea: **{}**\n- Heuristica ({} puntos): **{}**\n- Senales encontradas:\n{senales}{tope}\n{veredicto}\n\nNumero final con el carril `{}`: {} (el carril y `politica_carriles` solo suben el numero).\n",
+        r.declarado
+            .as_deref()
+            .unwrap_or("no declarada (decide la heuristica)"),
+        r.puntos,
+        r.heuristico,
+        d.carril.nombre(),
+        plural(n)
+    )
+}
+
+/// Bloque de una ronda 2 o posterior: ya hay un plan de una ronda anterior.
+fn replanificacion(d: &Datos<'_>) -> String {
+    if d.ronda <= 1 {
+        return String::new();
+    }
+    format!(
+        "## Esto es una re-planificacion, no un primer pase\n\nYa existe un plan en `{}` de una ronda anterior y el juego de roles ha cambiado desde entonces (por ejemplo, subio el carril). **Leelo antes que nada.** Incorpora lo nuevo al plan que ya hay, no lo replantees desde el enunciado como si fuera la primera vez, y **devuelve el plan COMPLETO**: tu respuesta sustituye al fichero entero.\n\nSi el enfoque entero esta mal, eso NO se arregla aqui: dilo en el plan y que alguien decida si se replantea.\n\n",
+        ruta_plan_final(d.id)
+    )
+}
+
+fn lista(xs: &[&str]) -> String {
+    xs.iter().map(|x| format!("- {x}\n")).collect()
+}
+
+/// La peticion de un rol (con dos o mas roles, o en una ronda con
+/// unificador).
+pub fn peticion_rol(d: &Datos<'_>, rol: &Rol, otros: &[Rol]) -> String {
+    let salida = nombre_salida_rol(rol, d.ronda);
+    let companeros = if otros.is_empty() {
+        "Eres el unico rol que se lanza en esta tarea.\n".to_owned()
+    } else {
+        let nombres: Vec<String> = otros.iter().map(|r| format!("**{}**", r.titulo)).collect();
+        format!(
+            "Trabajan en paralelo contigo, sin verte: {}. No cubras lo suyo — si lo haces, el unificador recibira el mismo punto de vista repetido y lo leera como confirmacion.\n",
+            nombres.join(", ")
+        )
+    };
+    format!(
+        "# Peticion de brainstorm — {id}, rol {titulo} (ronda {ronda})\n\n{cab}- Rol: `{rid}` — lanzalo con el agente de ese mismo nombre\n- Vuelca tu respuesta en: `{salida}` (se publica con `{orden}`)\n\n## Tu pregunta\n\n> {pregunta}\n\n## Que miras\n\n{mira}\n## Que NO miras\n\nNo es una lista de cortesia. Si la ignoras y lo cubres todo \"por si acaso\", el acotado de contexto se deshace sin que se note y esta tarea deja de tener varios puntos de vista:\n\n{no_mira}\n{enun}\n## Como entregas\n\nEscribe en `{salida}`, con estas secciones y en este orden:\n\n{entregables}\n## Reglas\n\n- **Propones enfoque; no implementas.** No escribas codigo de produccion ni modifiques ficheros del repo: tu salida es un documento.\n- Evidencia, no suposicion: si afirmas que algo se comporta de cierta manera, es porque lo has mirado. Di de donde lo sacas.\n- Prefiere lo concreto: rutas, nombres y casos reales por encima de recomendaciones genericas.\n- {companeros}",
+        id = d.id,
+        titulo = rol.titulo,
+        ronda = d.ronda,
+        cab = cabecera(d),
+        rid = rol.id,
+        orden = guardar(d.id, &format!("{BRAINSTORM}/{salida}")),
+        pregunta = rol.pregunta,
+        mira = lista(rol.mira),
+        no_mira = lista(rol.no_mira),
+        enun = enunciado(d.enunciado),
+        entregables = lista(rol.entregables),
+    )
+}
+
+/// Lo que tiene que traer el plan final.
+fn que_trae(seccion_roles: &str) -> String {
+    format!(
+        "## Que tiene que traer el plan final\n\n- Enfoque propuesto, concreto: que se crea, que se extiende, en que orden.\n{seccion_roles}- Riesgos aceptados y que los contiene.\n- Plan de pruebas.\n- Lo que necesita decision de una persona antes de implementar. **El checkpoint humano es obligatorio**: este plan no vale hasta que alguien lo apruebe con `tc2 approve`.\n\n"
+    )
+}
+
+/// La peticion de redaccion: un solo rol y sin unificador; su respuesta es
+/// el plan.
+pub fn peticion_redaccion(d: &Datos<'_>, rol: &Rol) -> String {
+    format!(
+        "# Peticion de redaccion del plan — {id} (ronda {ronda})\n\n{cab}- Rol: `{rid}` — lanzalo con el agente de ese mismo nombre\n- Quien orquesta publica tu respuesta con: `{orden}`\n\n**Esta tarea se planifica con un solo rol y no hay unificador**: tu respuesta ES el plan final. Redactala con las secciones de abajo; quien orquesta la publica tal cual en `plan-final.md` (tu no escribes ficheros), sin salida intermedia que consolidar.\n\n{replan}## Tu pregunta\n\n> {pregunta}\n\n## Que miras\n\n{mira}\n## Que NO miras\n\nQue lo mires solo tu no te autoriza a cubrirlo todo: lo que queda fuera de tu pregunta no se mira, pero se DICE. Anota en el plan lo que no cubres, para que quien lo apruebe sepa que ese lado no se ha revisado:\n\n{no_mira}\n{complejidad}\n{trae}## Reglas\n\n- **Propones enfoque; no implementas.** No escribas codigo de produccion ni modifiques ficheros del repo: tu salida es el plan.\n- Evidencia, no suposicion: si afirmas que algo se comporta de cierta manera, es porque lo has mirado. Di de donde lo sacas.\n\n{enun}",
+        id = d.id,
+        ronda = d.ronda,
+        cab = cabecera(d),
+        rid = rol.id,
+        orden = guardar(d.id, PLAN_FINAL),
+        replan = replanificacion(d),
+        pregunta = rol.pregunta,
+        mira = lista(rol.mira),
+        no_mira = lista(rol.no_mira),
+        complejidad = bloque_complejidad(d, 1),
+        trae = que_trae(
+            "- Lo que el rol no cubrio, contrastado contra los criterios de aceptacion.\n"
+        ),
+        enun = enunciado(d.enunciado),
+    )
+}
+
+/// La peticion al unificador: consolida las salidas de los `roles` de esta
+/// ronda (con cero roles, redacta desde el enunciado).
+pub fn peticion_unificador(d: &Datos<'_>, roles: &[Rol]) -> String {
+    let salidas: String = if roles.is_empty() {
+        "(ninguna: esta tarea no lanza brainstorm, ver el bloque de complejidad)\n".to_owned()
+    } else {
+        roles
+            .iter()
+            .map(|r| {
+                format!(
+                    "- `{}` — {}\n",
+                    ruta_brainstorm(d.id, &nombre_salida_rol(r, d.ronda)),
+                    r.titulo
+                )
+            })
+            .collect()
+    };
+    let como = match roles.len() {
+        0 => "No hay salidas de brainstorm que consolidar: redacta el plan directamente a partir del enunciado. Deja dicho en el plan que se redacto sin brainstorm y por que (el numero de roles sale del carril y del lookup de complejidad, no de un descuido).\n".to_owned(),
+        1 => "**Esta tarea se planifico con un solo rol**, asi que aqui no hay desacuerdos que resolver: no te los inventes ni trates la ausencia de discrepancia como una senal de nada.\n\n1. Lee la salida de arriba. Si falta o esta sin rellenar, dilo en el plan en vez de suplirla en silencio.\n2. **Contrasta esa propuesta contra el enunciado de la tarea**, que tienes al final. Lo util que puedes aportar aqui no es mediar entre puntos de vista, es senalar QUE QUEDO SIN CUBRIR: criterios de aceptacion que el rol no toca, riesgos que no mira porque no era su papel, decisiones que da por hechas.\n3. Cada afirmacion del plan que venga del rol se atribuye a el; lo que anadas tu, tambien.\n".to_owned(),
+        _ => "1. Lee las salidas de arriba. **Si alguna falta o esta sin rellenar, sigue adelante con las que haya y escribe en el plan cual falto**: un plan con un punto de vista menos, dicho, vale mas que un plan que finge estar completo.\n2. **No promedies.** Donde dos roles discrepen, el plan dice quien propone que, cual gana y por que. Un desacuerdo resuelto con una frase intermedia que no defiende nadie es la peor salida posible de este paso.\n3. **Si no discrepan en nada, eso es la alarma, no la nota de calidad**: significa que los roles recibieron el mismo contexto o que alguno no hizo su trabajo. Dilo en el plan.\n4. Cada afirmacion del plan que venga de un rol se atribuye a ese rol.\n".to_owned(),
+    };
+    let seccion = match roles.len() {
+        0 => "",
+        1 => "- Lo que el rol no cubrio, contrastado contra los criterios de aceptacion.\n",
+        _ => "- Los desacuerdos entre roles y como se resuelve cada uno.\n",
+    };
+    format!(
+        "# Peticion al unificador — {id} (ronda {ronda})\n\n{cab}- Rol: `{UNIFICADOR}` — lanzalo con el agente de ese mismo nombre\n- Vuelca el resultado con: `{orden}`\n\n{replan}## Salidas que tienes que consolidar\n\n{salidas}\n{complejidad}\n## Como consolidas\n\n{como}\n{trae}{enun}",
+        id = d.id,
+        ronda = d.ronda,
+        cab = cabecera(d),
+        orden = guardar(d.id, PLAN_FINAL),
+        replan = replanificacion(d),
+        complejidad = bloque_complejidad(d, roles.len()),
+        trae = que_trae(seccion),
+        enun = enunciado(d.enunciado),
+    )
+}
+
+/// El esqueleto de `plan-final.md` (F1-9, punto 7): los titulos de la 0.16
+/// en su orden, la cabecera de origen en una linea, `Duracion estimada: N
+/// min`, «Desacuerdos» con dos o mas roles o «Lo que el rol no cubrio» con
+/// uno (nunca las dos) y `## Troceo` opcional.
+pub fn plan_final(id: &str, titulo: &str, roles: &[Rol]) -> String {
+    let titulos: Vec<&str> = roles.iter().map(|r| r.titulo).collect();
+    let (origen, seccion) = match roles.len() {
+        0 => (
+            "(Sin brainstorm: el carril y la complejidad resuelven cero roles; el plan se redacta desde el enunciado.)".to_owned(),
+            "",
+        ),
+        1 => (
+            format!(
+                "(Respuesta del unico rol, {}, publicada con tc2 plan guardar; sin unificador: el plan senala lo que el rol no cubrio.)",
+                titulos.join(", ")
+            ),
+            "## Lo que el rol no cubrio\n\n\n",
+        ),
+        n => (
+            format!(
+                "(Lo consolida el unificador a partir de {n} roles: {}; los desacuerdos se senalan, no se promedian.)",
+                titulos.join(", ")
+            ),
+            "## Desacuerdos entre roles, y como se resuelven\n\n\n",
+        ),
+    };
+    format!(
+        "# Plan — {id}: {titulo}\n\n{origen}\n\n## Enfoque propuesto\n\nDuracion estimada: N min\n\n\n{seccion}## Riesgos aceptados y que los contiene\n\n\n## Plan de pruebas\n\nTests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):\n\n\n## Lo que necesita decision de una persona\n\n\n## Troceo\n\n"
+    )
+}
+
+#[cfg(test)]
+mod tests {
+    use super::super::heuristica;
+    use super::super::roles::{ESCALERA, seleccionar};
+    use super::*;
+
+    fn datos<'a>(e: &'a Enunciado, r: &'a Resolucion, ronda: u32) -> Datos<'a> {
+        Datos {
+            id: "TASK-007",
+            titulo: "Siete",
+            tipo: "feature",
+            complejidad: None,
+            carril: Carril::Funcionalidad,
+            ronda,
+            fecha: "2026-11-18",
+            enunciado: e,
+            resolucion: r,
+        }
+    }
+
+    #[test]
+    fn nombres_de_fichero_de_la_016() {
+        let a = &ESCALERA[0];
+        assert_eq!(
+            nombre_peticion_rol(a, 2),
+            "peticion-brainstorm-arquitectura-2.md"
+        );
+        assert_eq!(
+            nombre_salida_rol(a, 1),
+            "salida-brainstorm-arquitectura-1.md"
+        );
+        assert_eq!(nombre_peticion_unificador(3), "peticion-unificador-3.md");
+        assert_eq!(nombre_peticion_redaccion(1), "peticion-plan-1.md");
+        assert_eq!(
+            ruta_plan_final("TASK-001"),
+            "TASK-001/planificacion/plan-final.md"
+        );
+    }
+
+    /// La valla crece con el texto embebido; las peticiones llevan la
+    /// pregunta literal, el «que no miras», la orden de guardar y
+    /// `tc2 approve`.
+    #[test]
+    fn peticiones_con_lo_suyo() {
+        assert_eq!(valla(&["a", "b"]), "````");
+        assert_eq!(valla(&["x\n  `````\ny"]), "``````");
+        assert_eq!(valla(&["    `````"]), "````");
+        let e = Enunciado {
+            objetivo: "Hacer ```x```".into(),
+            criterios: vec!["uno".into()],
+        };
+        let r = heuristica::resolver(None, &[], "feature", &[], &e);
+        let d = datos(&e, &r, 1);
+        let roles = seleccionar(2, &[]);
+        let p = peticion_rol(&d, &roles[0], &roles[1..]);
+        assert!(p.contains("> Dado lo que ya existe, ¿donde encaja"), "{p}");
+        assert!(p.contains("Trabajan en paralelo contigo, sin verte: **riesgos**."));
+        assert!(p.contains(
+            "`tc2 plan guardar TASK-007 brainstorm/salida-brainstorm-arquitectura-1.md <fichero>`"
+        ));
+        let u = peticion_unificador(&d, &roles);
+        assert!(u.contains("`brainstorm-unificador`") && u.contains("tc2 approve"));
+        assert!(u.contains(
+            "- `TASK-007/planificacion/brainstorm/salida-brainstorm-riesgos-1.md` — riesgos"
+        ));
+        assert!(!u.contains("taskctl") && !u.contains("re-planificacion"));
+        let red = peticion_redaccion(&datos(&e, &r, 2), &roles[0]);
+        assert!(
+            red.contains("re-planificacion")
+                && red.contains("tc2 plan guardar TASK-007 plan-final.md")
+        );
+        assert!(red.contains("Numero final con el carril `funcionalidad`: 1 rol"));
+    }
+
+    /// El esqueleto: «Lo que el rol no cubrio» con uno, «Desacuerdos» con
+    /// dos o mas, ninguna de las dos con cero.
+    #[test]
+    fn esqueleto_segun_los_roles() {
+        let uno = plan_final("TASK-001", "Uno", &seleccionar(1, &[]));
+        assert!(
+            uno.starts_with("# Plan — TASK-001: Uno\n\n(Respuesta del unico rol, arquitectura,")
+        );
+        assert!(uno.contains("## Lo que el rol no cubrio") && !uno.contains("Desacuerdos"));
+        assert!(uno.contains("\nDuracion estimada: N min\n"));
+        let cuatro = plan_final("TASK-001", "Uno", &seleccionar(4, &[]));
+        assert!(cuatro.contains("## Desacuerdos entre roles") && !cuatro.contains("no cubrio"));
+        assert!(cuatro.contains("a partir de 4 roles: arquitectura, riesgos, testing, dominio;"));
+        let cero = plan_final("TASK-001", "Uno", &[]);
+        assert!(!cero.contains("Desacuerdos") && !cero.contains("no cubrio"));
+        for p in [&uno, &cuatro, &cero] {
+            assert!(p.ends_with("## Troceo\n\n"), "{p}");
+        }
+    }
+}
diff --git a/crates/tc2/src/plan/roles.rs b/crates/tc2/src/plan/roles.rs
new file mode 100644
index 0000000..d7a8889
--- /dev/null
+++ b/crates/tc2/src/plan/roles.rs
@@ -0,0 +1,334 @@
+//! Los roles del brainstorm de la fase de diseno (F4-1, «mismos roles que
+//! la 0.16»): copia literal de `src/core/roles-brainstorm.ts` del plugin
+//! 0.16.2 (ids, preguntas, «que miras», «que no miras» y entregables).
+//!
+//! La heuristica y el carril fijan CUANTOS; este modulo fija CUALES, de
+//! forma determinista: la escalera en orden de prioridad (arquitectura,
+//! riesgos, testing, dominio) y los N primeros. Con un rol entra
+//! arquitectura, el unico que propone una forma; con tres se cae dominio.
+//!
+//! Los roles condicionales no compiten por el presupuesto: entran por las
+//! etiquetas de la tarea y se suman a la escalera, solo si esta lanza
+//! [`MIN_ROLES_PARA_CONDICIONALES`] o mas (con 0 o 1 rol no hay unificador
+//! y el condicional quedaria de autor unico de un plan cuya forma no le toca
+//! proponer, TASK-080 de la 0.16).
+//!
+//! Cada `id` es el `name:` del agente que se lanza (`brainstorm-*`): tc2 no
+//! trae agentes propios y no comprueba que esten instalados (inventario,
+//! TASK-080).
+
+/// Un rol del brainstorm.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub struct Rol {
+    /// El `name:` del agente de ese rol.
+    pub id: &'static str,
+    /// Nombre corto para cabeceras y la linea.
+    pub titulo: &'static str,
+    /// La unica pregunta del rol, literal en su peticion.
+    pub pregunta: &'static str,
+    /// Que mira este rol (el recorte de contexto).
+    pub mira: &'static [&'static str],
+    /// Que no mira, dicho explicitamente.
+    pub no_mira: &'static [&'static str],
+    /// Secciones que su salida tiene que traer.
+    pub entregables: &'static [&'static str],
+}
+
+/// La escalera, en orden de prioridad (0.16.2).
+pub const ESCALERA: [Rol; 4] = [
+    Rol {
+        id: "brainstorm-arquitectura",
+        titulo: "arquitectura",
+        pregunta: "Dado lo que ya existe, ¿donde encaja este cambio y que forma tiene?",
+        mira: &[
+            "Los modulos y ficheros que ya resuelven algo parecido, para extenderlos en vez de duplicarlos.",
+            "Que se crea nuevo, que se extiende y en que orden se construye.",
+            "Los limites que el cambio cruza: contratos publicos, formatos de fichero, esquemas.",
+            "El precedente interno mas cercano: como se resolvio la ultima vez un problema de esta forma.",
+        ],
+        no_mira: &[
+            "Como se prueba (es del rol de testing).",
+            "Por donde se rompe (es del rol de riesgos).",
+            "Las reglas de negocio (son del rol de dominio).",
+        ],
+        entregables: &[
+            "Enfoque propuesto, con rutas y nombres concretos",
+            "Que se extiende y que se crea",
+            "Limites que cruza",
+            "La decision de diseño que mas te preocupa (UNA sola)",
+        ],
+    },
+    Rol {
+        id: "brainstorm-riesgos",
+        titulo: "riesgos",
+        pregunta: "¿Por donde se rompe esto?",
+        mira: &[
+            "Bordes y estados intermedios: que queda a medias si el proceso muere a mitad.",
+            "Fallos parciales y concurrencia: dos ejecuciones, un recurso ocupado, un permiso denegado.",
+            "Compatibilidad hacia atras con los datos y ficheros que YA existen.",
+            "La vuelta atras: si esto sale mal, como se deshace y que queda inservible.",
+        ],
+        no_mira: &[
+            "Donde encaja el cambio (es del rol de arquitectura).",
+            "Que aserciones escribir (es del rol de testing).",
+            "Las reglas de negocio (son del rol de dominio).",
+        ],
+        entregables: &[
+            "Modos de fallo, ordenados por gravedad, con el escenario concreto de cada uno",
+            "Estados intermedios y fallos parciales",
+            "Compatibilidad hacia atras",
+            "Vuelta atras",
+            "El riesgo que mas te preocupa (UNO solo)",
+        ],
+    },
+    Rol {
+        id: "brainstorm-testing",
+        titulo: "testing",
+        pregunta: "¿Como se demuestra que esto funciona, y como envejece?",
+        mira: &[
+            "Que es observable desde fuera, y contra que recurso real se comprueba.",
+            "Para cada prueba propuesta, LA MUTACION DEL CODIGO FUENTE QUE LA PONDRIA ROJA. Si no sabes decirla, esa prueba no vale.",
+            "Que pruebas ya existentes cambian de expectativa y cuales siguen valiendo de red de regresion.",
+            "Como envejece: que se rompe dentro de seis meses cuando alguien extienda esto.",
+        ],
+        no_mira: &[
+            "Donde encaja el cambio (es del rol de arquitectura).",
+            "Por donde se rompe en produccion (es del rol de riesgos).",
+            "Las reglas de negocio (son del rol de dominio).",
+        ],
+        entregables: &[
+            "Que es observable sin llamar a ningun agente",
+            "Plan de pruebas, cada una con su mutacion",
+            "Aserciones trampa que hay que evitar en esta tarea concreta",
+            "Que pruebas existentes cambian de expectativa",
+            "Como envejece",
+        ],
+    },
+    Rol {
+        id: "brainstorm-dominio",
+        titulo: "dominio",
+        pregunta: "¿Que sabe quien va a usar esto que no se deduce leyendo el codigo?",
+        mira: &[
+            "El vocabulario real: que nombra cada cosa quien trabaja en este dominio.",
+            "Los invariantes del negocio que ningun tipo del lenguaje impone y que aun asi no se pueden violar.",
+            "Los casos que en el codigo son simetricos y en el negocio no lo son.",
+            "Cuando una solucion correcta en codigo es incorrecta para quien la va a usar.",
+        ],
+        no_mira: &[
+            "Donde encaja el cambio (es del rol de arquitectura).",
+            "Por donde se rompe (es del rol de riesgos).",
+            "Como se prueba (es del rol de testing).",
+        ],
+        entregables: &[
+            "Vocabulario e invariantes que el enfoque tiene que respetar",
+            "Donde una solucion tecnicamente correcta seria incorrecta aqui",
+            "Lo que hace falta preguntarle a una persona antes de implementar",
+        ],
+    },
+];
+
+/// Un rol que entra por las etiquetas de la tarea, fuera del presupuesto.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub struct Condicional {
+    /// El rol.
+    pub rol: Rol,
+    /// Etiquetas que lo disparan, en minusculas; basta una.
+    pub etiquetas: &'static [&'static str],
+}
+
+/// Los condicionales (0.16.2): hoy, rendimiento.
+pub const CONDICIONALES: [Condicional; 1] = [Condicional {
+    rol: Rol {
+        id: "brainstorm-rendimiento",
+        titulo: "rendimiento",
+        pregunta: "¿Cuanto cuesta esto en ejecucion con los datos reales, y donde se va ese coste?",
+        mira: &[
+            "El coste algoritmico con el volumen real de datos: que crece mas que lineal y a partir de que tamaño se nota.",
+            "Las asignaciones y copias en los bucles calientes: lo que se crea en cada iteracion o en cada frame.",
+            "Las fronteras caras: copias entre JS y WASM, mensajes y transferibles hacia Web Workers, lo que bloquea el hilo principal.",
+            "Las cargas grandes: mallas 3D, teselas e imagenes pesadas, cuanta memoria ocupan y cuando se liberan.",
+            "Como se mide antes de optimizar: con que datos reales, que metrica y que umbral decide si hace falta.",
+        ],
+        no_mira: &[
+            "La forma del cambio y la escalabilidad que fija el diseño (es del rol de arquitectura).",
+            "Por donde se rompe (es del rol de riesgos).",
+            "Que aserciones escribir (es del rol de testing).",
+            "Las reglas de negocio (son del rol de dominio).",
+        ],
+        entregables: &[
+            "Donde se va el coste, con el volumen de datos real de cada punto",
+            "Fronteras caras: JS-WASM, Web Workers e hilo principal",
+            "Que medir antes de optimizar, con que datos y que umbral",
+            "La optimizacion que NO harias todavia (UNA sola)",
+        ],
+    },
+    etiquetas: &["rendimiento", "wasm", "workers", "3d"],
+}];
+
+/// Por debajo de este numero de roles de la escalera no se suma ningun
+/// condicional.
+pub const MIN_ROLES_PARA_CONDICIONALES: usize = 2;
+
+/// El agente unificador: no es un rol del brainstorm, consolida los que hay.
+pub const UNIFICADOR: &str = "brainstorm-unificador";
+
+/// Todos los roles (la escalera y los condicionales), para buscar uno por
+/// su id (las salidas de `plan guardar`).
+pub fn todos() -> impl Iterator<Item = &'static Rol> {
+    ESCALERA.iter().chain(CONDICIONALES.iter().map(|c| &c.rol))
+}
+
+/// Si alguna de `etiquetas` dispara `c` (sin distinguir mayusculas).
+fn dispara(c: &Condicional, etiquetas: &[String]) -> bool {
+    etiquetas
+        .iter()
+        .any(|e| c.etiquetas.contains(&e.trim().to_lowercase().as_str()))
+}
+
+/// Los roles que lanza una tarea con `n` roles de diseno: los `n` primeros
+/// de la escalera y, si son al menos [`MIN_ROLES_PARA_CONDICIONALES`], los
+/// condicionales que disparan sus `etiquetas`, al final y en su orden.
+///
+/// `n` por encima de la escalera (la configuracion admite hasta 5,
+/// `politica_carriles.<carril>.roles_diseno`) suma los condicionales que
+/// falten, en su orden, aunque ninguna etiqueta los pida: 5 son los cinco
+/// roles. Es la unica diferencia con `seleccionarRolesDeTarea` de la 0.16,
+/// que con mas de 4 fallaba (alli el numero no podia pasar de 4).
+pub fn seleccionar(n: u8, etiquetas: &[String]) -> Vec<Rol> {
+    let n = usize::from(n);
+    let mut roles: Vec<Rol> = ESCALERA.iter().take(n).copied().collect();
+    if roles.len() < MIN_ROLES_PARA_CONDICIONALES {
+        return roles;
+    }
+    let mut extra = n.saturating_sub(ESCALERA.len());
+    for c in &CONDICIONALES {
+        if dispara(c, etiquetas) {
+            roles.push(c.rol);
+        } else if extra > 0 {
+            extra -= 1;
+            roles.push(c.rol);
+        }
+    }
+    roles
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn ids(roles: &[Rol]) -> Vec<&'static str> {
+        roles.iter().map(|r| r.id).collect()
+    }
+
+    fn etiquetas(e: &[&str]) -> Vec<String> {
+        e.iter().map(|x| (*x).to_owned()).collect()
+    }
+
+    /// Criterio 1 de TASK-036: los mismos roles que la 0.16.2, en el mismo
+    /// orden, con sus preguntas literales, y el mismo unificador.
+    #[test]
+    fn mismos_roles_y_preguntas_que_la_016() {
+        let leidos: Vec<(&str, &str, &str)> =
+            todos().map(|r| (r.id, r.titulo, r.pregunta)).collect();
+        assert_eq!(
+            leidos,
+            [
+                (
+                    "brainstorm-arquitectura",
+                    "arquitectura",
+                    "Dado lo que ya existe, ¿donde encaja este cambio y que forma tiene?"
+                ),
+                ("brainstorm-riesgos", "riesgos", "¿Por donde se rompe esto?"),
+                (
+                    "brainstorm-testing",
+                    "testing",
+                    "¿Como se demuestra que esto funciona, y como envejece?"
+                ),
+                (
+                    "brainstorm-dominio",
+                    "dominio",
+                    "¿Que sabe quien va a usar esto que no se deduce leyendo el codigo?"
+                ),
+                (
+                    "brainstorm-rendimiento",
+                    "rendimiento",
+                    "¿Cuanto cuesta esto en ejecucion con los datos reales, y donde se va ese coste?"
+                ),
+            ]
+        );
+        assert_eq!(UNIFICADOR, "brainstorm-unificador");
+        assert_eq!(
+            CONDICIONALES[0].etiquetas,
+            ["rendimiento", "wasm", "workers", "3d"]
+        );
+        assert_eq!(MIN_ROLES_PARA_CONDICIONALES, 2);
+        // Ningun rol se deja una lista vacia.
+        for r in todos() {
+            assert!(
+                !r.mira.is_empty() && !r.no_mira.is_empty() && !r.entregables.is_empty(),
+                "{}",
+                r.id
+            );
+        }
+    }
+
+    /// La seleccion: prefijo de la escalera; el condicional por etiqueta
+    /// solo con 2 o mas; 5 son los cinco aunque no haya etiqueta.
+    #[test]
+    fn seleccion_de_roles() {
+        let rend = etiquetas(&["UI", " WASM "]);
+        for (n, e, esperado) in [
+            (0, etiquetas(&[]), vec![]),
+            (0, rend.clone(), vec![]),
+            (1, rend.clone(), vec!["brainstorm-arquitectura"]),
+            (
+                2,
+                etiquetas(&["ui"]),
+                vec!["brainstorm-arquitectura", "brainstorm-riesgos"],
+            ),
+            (
+                2,
+                rend.clone(),
+                vec![
+                    "brainstorm-arquitectura",
+                    "brainstorm-riesgos",
+                    "brainstorm-rendimiento",
+                ],
+            ),
+            (
+                4,
+                etiquetas(&[]),
+                vec![
+                    "brainstorm-arquitectura",
+                    "brainstorm-riesgos",
+                    "brainstorm-testing",
+                    "brainstorm-dominio",
+                ],
+            ),
+            (
+                5,
+                etiquetas(&[]),
+                vec![
+                    "brainstorm-arquitectura",
+                    "brainstorm-riesgos",
+                    "brainstorm-testing",
+                    "brainstorm-dominio",
+                    "brainstorm-rendimiento",
+                ],
+            ),
+            (
+                5,
+                rend,
+                vec![
+                    "brainstorm-arquitectura",
+                    "brainstorm-riesgos",
+                    "brainstorm-testing",
+                    "brainstorm-dominio",
+                    "brainstorm-rendimiento",
+                ],
+            ),
+        ] {
+            assert_eq!(ids(&seleccionar(n, &e)), esperado, "n={n} {e:?}");
+        }
+    }
+}
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-036/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-036/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-036/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/plan.rs"
````

````
crates/tc2/tests/plan.rs | 389 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 389 insertions(+)
````
