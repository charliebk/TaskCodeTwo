# Peticion de revision — TASK-049 (ronda 1)

- Tarea: TASK-049 — F6-3 Revisor en CI con las skills de dominio de la 0.16
- Rama revisada: feature/task-049-f6-3-revisor-en-ci-con-las-skills-de-dom
- Rama base: develop
- Commit revisado (HEAD): 17ad62c057ca6f147c5df63ee7124444224f7693
- Fecha: 2026-11-20
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-049 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-049/tarea.md
- tareas/03-en-revision/TASK-049/planificacion/
- crates/tc2/Cargo.toml
- crates/tc2/src/i18n/catalogo/cli.rs
- crates/tc2/src/i18n/catalogo/mod.rs
- crates/tc2/src/i18n/catalogo/seleccion.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/revision/mod.rs
- crates/tc2/src/revision/operacion.rs
- crates/tc2/src/seleccion/catalogo.rs
- crates/tc2/src/seleccion/mod.rs
- crates/tc2/src/seleccion/orden.rs
- crates/tc2/src/seleccion/tabla.rs
- crates/tc2/tests/revision.rs
- crates/tc2/tests/revisoras.rs
- xtask/src/main.rs
- xtask/src/workflows.rs

## Commits a revisar (git log develop..HEAD)

````
17ad62c fix(TASK-049): el build reproducible copia docs/adr, que tc2 incrusta
6f7e891 feat(TASK-049): workflow reutilizable revision-ia, lint y adendas
9459d56 feat(TASK-049): seleccion determinista de revisoras y tc2 revisoras
f1f0529 chore(TASK-049): tarea en curso
````

## Diff de tu dominio (15 fichero(s) de 24; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/Cargo.toml b/crates/tc2/Cargo.toml
index 3de43e0..2cbf00c 100644
--- a/crates/tc2/Cargo.toml
+++ b/crates/tc2/Cargo.toml
@@ -36,6 +36,9 @@ distribucion = { path = "../distribucion" }
 # (alternativa descartada: `jsonwebtoken`, que trae su propia cripto).
 ring = "0.17.14"
 base64 = "0.23.1"
+# F6-3: comparador de los globs de `patrones_archivo` de las revisoras (F1-15
+# decision 3: sin distinguir mayusculas). Sin la feature `log`.
+globset = { version = "0.4.21", default-features = false }
 
 # Parser YAML de otra base de codigo (port de libyaml, como el PyYAML del PoC
 # de F1-9) para la ida y vuelta de los tests: con el de produccion seria
diff --git a/crates/tc2/src/i18n/catalogo/cli.rs b/crates/tc2/src/i18n/catalogo/cli.rs
index 28636a6..c6beca2 100644
--- a/crates/tc2/src/i18n/catalogo/cli.rs
+++ b/crates/tc2/src/i18n/catalogo/cli.rs
@@ -10,8 +10,8 @@ crate::i18n::textos! {
         en: "tc2: could not write the output: {error}",
     }
     USO {
-        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<motivo>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <carril>] | tc2 plan guardar [--json] TASK-NNN <destino> <fichero> | tc2 approve [--json] TASK-NNN [--pedir-cambios <fichero>] | tc2 approve [--json] --hito <n> | tc2 review [--json] TASK-NNN | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
-        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<reason>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <lane>] | tc2 plan guardar [--json] TASK-NNN <destination> <file> | tc2 approve [--json] TASK-NNN [--pedir-cambios <file>] | tc2 approve [--json] --hito <n> | tc2 review [--json] TASK-NNN | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
+        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<motivo>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <carril>] | tc2 plan guardar [--json] TASK-NNN <destino> <fichero> | tc2 approve [--json] TASK-NNN [--pedir-cambios <fichero>] | tc2 approve [--json] --hito <n> | tc2 review [--json] TASK-NNN | tc2 revisoras [--json] --rutas <fichero|-> [opciones] | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
+        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<reason>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <lane>] | tc2 plan guardar [--json] TASK-NNN <destination> <file> | tc2 approve [--json] TASK-NNN [--pedir-cambios <file>] | tc2 approve [--json] --hito <n> | tc2 review [--json] TASK-NNN | tc2 revisoras [--json] --rutas <file|-> [options] | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
     }
     FALTA_LA_ORDEN {
         es: "tc2: falta la orden. {uso}",
diff --git a/crates/tc2/src/i18n/catalogo/mod.rs b/crates/tc2/src/i18n/catalogo/mod.rs
index b6aeea5..a9238af 100644
--- a/crates/tc2/src/i18n/catalogo/mod.rs
+++ b/crates/tc2/src/i18n/catalogo/mod.rs
@@ -19,6 +19,7 @@ pub mod plataforma;
 pub mod progreso;
 pub mod revision;
 pub mod roadmap;
+pub mod seleccion;
 pub mod siguiente;
 pub mod tablero;
 pub mod toma;
@@ -46,6 +47,7 @@ pub fn todos() -> impl Iterator<Item = &'static Texto> {
         progreso::TODOS,
         revision::TODOS,
         roadmap::TODOS,
+        seleccion::TODOS,
         siguiente::TODOS,
         tablero::TODOS,
         toma::TODOS,
diff --git a/crates/tc2/src/i18n/catalogo/seleccion.rs b/crates/tc2/src/i18n/catalogo/seleccion.rs
new file mode 100644
index 0000000..cc2f4b7
--- /dev/null
+++ b/crates/tc2/src/i18n/catalogo/seleccion.rs
@@ -0,0 +1,16 @@
+//! Mensajes de `tc2 revisoras` (F6-3).
+
+crate::i18n::textos! {
+    USO_REVISORAS {
+        es: "uso: revisoras [--json] --rutas <fichero|-> [--tarea TASK-NNN] [--carril <carril>] [--complejidad <nivel>] [--plugin <nombre>=<carpeta>]... [--informe <fichero> --seccion <salida> [--artefacto <nombre>]] [--detalle <salida.json>]",
+        en: "usage: revisoras [--json] --rutas <file|-> [--tarea TASK-NNN] [--carril <lane>] [--complejidad <level>] [--plugin <name>=<folder>]... [--informe <file> --seccion <output> [--artefacto <name>]] [--detalle <output.json>]",
+    }
+    NO_SE_LEE {
+        es: "no se puede leer ni escribir {ruta}: {error}",
+        en: "cannot read or write {ruta}: {error}",
+    }
+    TABLA_ILEGIBLE {
+        es: "las tablas del ADR F1-15 compiladas en tc2 no se leen: {error}",
+        en: "the ADR F1-15 tables compiled into tc2 cannot be read: {error}",
+    }
+}
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 9a4ada2..42b84b1 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -20,7 +20,8 @@
 //! (aviso de solape de `start`, F3-11) y [`ciclo`]
 //! (`descartar`, `pausa` y `reanudar`, F3-7; `liberar` y `reasignar`,
 //! F3-10) y [`doctor`] (F3-8) y [`plan`] (`plan` y `plan guardar`, F4-1;
-//! `approve`, F4-2) y [`revision`] (`review`, F6-2). Las capas
+//! `approve`, F4-2) y [`revision`] (`review`, F6-2) y [`seleccion`]
+//! (`revisoras`, revisoras y modelo de F1-15, F6-3). Las capas
 //! que vienen entran como modulos de esta biblioteca.
 
 pub mod actualizar_cli;
@@ -49,6 +50,7 @@ pub mod progreso;
 pub mod recursos;
 pub mod revision;
 pub mod roadmap;
+pub mod seleccion;
 pub mod siguiente;
 pub mod situacion;
 pub mod solape;
@@ -218,6 +220,10 @@ where
                 orden_review(&resto, e)
             })
         }
+        [orden, resto @ ..] if orden == seleccion::orden::NOMBRE => {
+            let (json, resto) = con_json(resto);
+            orden_revisoras(&resto).salida_en(json, idioma)
+        }
         [orden, resto @ ..] if orden == estimacion::NOMBRE => {
             let (json, resto) = con_json(resto);
             orden_estimar(&resto, json, idioma)
@@ -511,12 +517,57 @@ fn orden_review(args: &[String], e: &toma::Entorno<'_>) -> Informe {
         let personas = identidad::personas_de_base(e.git, e.dir)?;
         identidad::adaptador(destino, plataforma::de_entorno(), personas)
     };
+    let claude = dir_claude();
     revision::orden(
         args,
         &revision::Entorno {
             base: e,
             conectar: &conectar,
             esperas: &git::ESPERAS_PUSH,
+            claude: claude.as_deref(),
+        },
+    )
+}
+
+/// La carpeta de Claude Code (`CLAUDE_CONFIG_DIR` o `~/.claude`), solo para
+/// leer el inventario (F3-9, F6-3).
+fn dir_claude() -> Option<std::path::PathBuf> {
+    instalacion::Entorno::de_variables(
+        instalacion::Sistema::actual(),
+        std::path::PathBuf::new(),
+        std::process::id(),
+        |k| std::env::var(k).ok(),
+    )
+    .dir_claude()
+}
+
+/// `revisoras [--json] ...` (F6-3) en la carpeta actual: la entrada
+/// estandar para `--rutas -`.
+fn orden_revisoras(args: &[String]) -> Informe {
+    let dir = match std::env::current_dir() {
+        Ok(d) => d,
+        Err(e) => {
+            return Informe::nuevo(seleccion::orden::NOMBRE, Resultado::Error)
+                .diagnostico(Mensaje::nuevo(&txt::CARPETA_ACTUAL).con("error", e.to_string()));
+        }
+    };
+    let g = git::Git::nuevo();
+    let claude = dir_claude();
+    let stdin = || {
+        use std::io::Read;
+        let mut s = String::new();
+        std::io::stdin()
+            .take(seleccion::orden::TOPE_RUTAS)
+            .read_to_string(&mut s)
+            .map(|_| s)
+    };
+    seleccion::orden::orden(
+        args,
+        &seleccion::orden::Entorno {
+            dir: &dir,
+            git: &g,
+            claude: claude.as_deref(),
+            stdin: &stdin,
         },
     )
 }
@@ -578,15 +629,7 @@ fn orden_doctor(args: &[String], json: bool, idioma: Idioma) -> Salida {
     };
     let g = git::Git::nuevo();
     let conectar = tablero::conectar_produccion(&g, &dir);
-    // La carpeta de Claude Code (`CLAUDE_CONFIG_DIR` o `~/.claude`), solo para
-    // leer el inventario (F3-9).
-    let claude = instalacion::Entorno::de_variables(
-        instalacion::Sistema::actual(),
-        std::path::PathBuf::new(),
-        std::process::id(),
-        |k| std::env::var(k).ok(),
-    )
-    .dir_claude();
+    let claude = dir_claude();
     let e = doctor::Entorno {
         dir: &dir,
         git: &g,
diff --git a/crates/tc2/src/revision/mod.rs b/crates/tc2/src/revision/mod.rs
index 6afb809..aeb9b9d 100644
--- a/crates/tc2/src/revision/mod.rs
+++ b/crates/tc2/src/revision/mod.rs
@@ -24,7 +24,9 @@
 //! rama.
 //!
 //! Linea: `review ok id=TASK-NNN rama=<r> pr=<n> estado=en-revision
-//! revision-ia=<ok|pendiente|fallo|sin-revisor|sin-leer>`. Repetirla sin
+//! revision-ia=<ok|pendiente|fallo|sin-revisor|sin-leer> revisoras=<ids>
+//! modelo=<m> reglas=<ids> avisos=<...>` (la seleccion de F6-3, anotada en
+//! `roles.revision`). Repetirla sin
 //! nada nuevo sale `sin-cambios`.
 
 pub mod operacion;
@@ -39,10 +41,11 @@ use crate::git::{self, Git};
 use crate::i18n::Mensaje;
 use crate::i18n::catalogo::revision as txt;
 use crate::plataforma::{ModeloPuertas, Paso, Plataforma, Rol};
+use crate::seleccion;
 use crate::tarea::Tarea;
 use crate::toma;
 
-pub use operacion::{EN_REVISION, Revisar, desde_anotacion};
+pub use operacion::{AnotacionRevision, EN_REVISION, Revisar, desde_anotacion};
 
 /// Nombre de la orden y de la operacion de la rama de estado.
 pub const NOMBRE: &str = "review";
@@ -67,6 +70,9 @@ pub struct Entorno<'a> {
     pub conectar: &'a Conectar<'a>,
     /// Reintentos del push (en produccion, [`git::ESPERAS_PUSH`]).
     pub esperas: &'a [Duration],
+    /// Carpeta de Claude Code para el inventario de la seleccion (F6-3);
+    /// `None`, solo lo del proyecto en la base y lo integrado.
+    pub claude: Option<&'a Path>,
 }
 
 /// `review TASK-NNN` (sin `--json`, que quita `lib.rs`).
@@ -227,8 +233,8 @@ fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
             Fallo::Error(d) => Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d),
         });
     }
-    let t = match estado::tarea_de(&vista, id) {
-        Ok((t, _)) => t,
+    let (t, cuerpo) = match estado::tarea_de(&vista, id) {
+        Ok(tc) => tc,
         Err(Fallo::Rechazada {
             motivo,
             diagnostico,
@@ -270,7 +276,10 @@ fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
     let Some(url) = git::estado::url_origin(g, &wt, NOMBRE)? else {
         return Err(rechazada("plataforma", Mensaje::nuevo(&txt::SIN_ORIGIN)));
     };
-    // 4. Push y PR con el agente.
+    // 4. La seleccion de revision (F6-3), con el diff de la rama y lo de la
+    // base; no cambia nada si falla: lo dice en `avisos`.
+    let sel = seleccion_de(g, &principal, &rama, (&t, &cuerpo), e.claude);
+    // 5. Push y PR con el agente.
     let p = (e.conectar)()?;
     let publicado = publicar(
         p.as_ref(),
@@ -281,11 +290,15 @@ fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
         e.esperas,
     )?;
     let pr = publicado.pr.to_string();
-    // 5. El estado, con el PR ya abierto.
+    // 6. El estado, con el PR ya abierto (y la seleccion en `roles.revision`).
     let personas = crate::roadmap::personas(g, dir);
     let generar = crate::roadmap::generador(&personas);
     let ctx = estado::Contexto::nuevo(g, dir, e.base.catalogo, Some(&generar));
-    let op = Revisar::nueva(id, &quien, ahora);
+    let op = Revisar::nueva(id, &quien, ahora).con_seleccion(AnotacionRevision {
+        agente: sel.agentes.first().cloned(),
+        modelo: sel.modelo.clone(),
+        skills: sel.revisoras.clone(),
+    });
     let tr = estado::transaccion(&ctx, &op);
     match tr.resultado {
         Resultado::Ok | Resultado::SinCambios => {}
@@ -297,7 +310,7 @@ fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
             return Err(tr);
         }
     }
-    // 6. `revision-ia`, leido con el agente.
+    // 7. `revision-ia`, leido con el agente.
     let revisor = config::de_base(g, &principal)
         .ok()
         .and_then(|c| c.plataforma)
@@ -314,7 +327,49 @@ fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
         .con("rama", rama.as_str())
         .con("pr", pr)
         .con("estado", EN_REVISION)
-        .con(REVISION_IA, ria))
+        .con(REVISION_IA, ria)
+        .con_lista("revisoras", sel.revisoras)
+        .con("modelo", sel.modelo)
+        .con_lista("reglas", sel.reglas)
+        .con_lista("avisos", sel.avisos))
+}
+
+/// La seleccion de revision de la tarea `t` en la rama `rama` (F6-3): rutas
+/// de `git diff --no-renames origin/develop...<rama>`, inventario y catalogo
+/// de [`seleccion::orden::fuentes`], carril y complejidad de
+/// [`seleccion::orden::de_tarea`] y `modelos_permitidos` de la base. Lo que
+/// no se puede leer sale en `avisos` (`sin-diff`, `sin-base`...).
+pub fn seleccion_de(
+    g: &Git,
+    principal: &Path,
+    rama: &str,
+    (t, cuerpo): (&Tarea, &crate::tarea::Cuerpo),
+    claude: Option<&Path>,
+) -> seleccion::Seleccion {
+    let mut avisos = Vec::new();
+    let rutas: Vec<String> = git::ficheros_de_rama(g, principal, NOMBRE, config::BASE, rama)
+        .map(|r| r.into_iter().collect())
+        .unwrap_or_else(|_| {
+            avisos.push("sin-diff".to_owned());
+            Vec::new()
+        });
+    let f = seleccion::orden::fuentes(g, principal, claude, &[]);
+    avisos.extend(f.avisos.iter().cloned());
+    let (carril, cx) = seleccion::orden::de_tarea(t, cuerpo);
+    let cfg = config::de_base(g, principal).unwrap_or_else(|_| {
+        avisos.push("config-de-base-no-leida".to_owned());
+        config::Config::suelo()
+    });
+    let ctx = seleccion::Contexto {
+        carril,
+        complejidad: Some(cx),
+        modelos_permitidos: &cfg.seleccion.modelos_permitidos,
+        ronda: 1,
+    };
+    let mut s = seleccion::revision(&rutas, &f.catalogo, &f.inventario, &ctx).unwrap_or_default();
+    avisos.append(&mut s.avisos);
+    s.avisos = avisos;
+    s
 }
 
 #[cfg(test)]
diff --git a/crates/tc2/src/revision/operacion.rs b/crates/tc2/src/revision/operacion.rs
index eaec532..978c8ad 100644
--- a/crates/tc2/src/revision/operacion.rs
+++ b/crates/tc2/src/revision/operacion.rs
@@ -8,8 +8,18 @@
 //!   `asignado-heredado` si es un texto de la 0.16).
 //! - `en-curso` pasa a `en-revision`, con `actualizado` y una fila `review`
 //!   en `## Transiciones`; el resto del `tarea.md`, byte a byte.
-//! - Ya `en-revision`: nada que escribir (`sin-cambios`).
+//! - Ya `en-revision`: nada que escribir (`sin-cambios`), salvo que la
+//!   seleccion de revision haya cambiado (F6-3): entonces solo se reescribe
+//!   `roles.revision`, sin fila nueva.
 //! - Cualquier otro estado se rechaza con su motivo.
+//!
+//! F6-3 (F1-15 decision 10): con [`Revisar::con_seleccion`] la operacion
+//! anota en `roles.revision` el lanzador (`agente`), el modelo y las
+//! revisoras (`skills`). `regla` se deja como estaba: el modelo de tarea de
+//! F2-5 la limita al vocabulario de la 0.16, asi que los ids de fila van en
+//! la linea de `review` y en el check (divergencia de la adenda F6-3). La
+//! anotacion de la rama de estado no lleva la seleccion: `review` exige red
+//! y nunca queda pendiente.
 
 use std::cell::RefCell;
 
@@ -25,6 +35,17 @@ use super::NOMBRE;
 /// Estado en el que queda la tarea revisada.
 pub const EN_REVISION: &str = "en-revision";
 
+/// Lo que la seleccion de revision anota en `roles.revision`.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct AnotacionRevision {
+    /// El lanzador (S12), si hay.
+    pub agente: Option<String>,
+    /// Modelo de la escala.
+    pub modelo: String,
+    /// Revisoras.
+    pub skills: Vec<String>,
+}
+
 /// Pasar la tarea `id` a revision.
 #[derive(Debug)]
 pub struct Revisar {
@@ -33,6 +54,7 @@ pub struct Revisar {
     decidido_por: &'static str,
     ahora: u64,
     dueno: RefCell<Option<String>>,
+    seleccion: Option<AnotacionRevision>,
 }
 
 impl Revisar {
@@ -49,9 +71,27 @@ impl Revisar {
             decidido_por,
             ahora,
             dueno: RefCell::new(None),
+            seleccion: None,
         }
     }
 
+    /// La misma operacion, anotando la seleccion en `roles.revision`.
+    #[must_use]
+    pub fn con_seleccion(mut self, s: AnotacionRevision) -> Self {
+        self.seleccion = Some(s);
+        self
+    }
+
+    /// `roles.revision` de `t` con la seleccion; `None` si no hay seleccion.
+    fn anotar(&self, t: &crate::tarea::Tarea) -> Option<crate::tarea::RolRevision> {
+        let s = self.seleccion.as_ref()?;
+        let mut rv = t.roles.revision.clone();
+        rv.agente.clone_from(&s.agente);
+        rv.modelo = Some(s.modelo.clone());
+        rv.skills.clone_from(&s.skills);
+        Some(rv)
+    }
+
     /// Login de quien la lleva si el ultimo intento la rechazo por
     /// `de-otra-persona`.
     pub fn dueno(&self) -> Option<String> {
@@ -92,11 +132,26 @@ impl Operacion for Revisar {
             *self.dueno.borrow_mut() = dueno;
             return Err(fallo);
         }
+        let anotada = self.anotar(&t);
         match t.estado.as_str() {
-            EN_REVISION => return Ok(Vec::new()),
+            EN_REVISION => {
+                return Ok(match anotada {
+                    Some(rv) if rv != t.roles.revision => {
+                        t.roles.revision = rv;
+                        vec![Cambio::nuevo(
+                            format!("{}/tarea.md", self.id),
+                            t.escribir(&cuerpo),
+                        )]
+                    }
+                    _ => Vec::new(),
+                });
+            }
             EN_CURSO => {}
             otro => return Err(por_estado(&self.id, otro, NOMBRE, &[EN_CURSO])),
         }
+        if let Some(rv) = anotada {
+            t.roles.revision = rv;
+        }
         let instante = instante(self.ahora);
         EN_REVISION.clone_into(&mut t.estado);
         t.actualizado = instante.get(..10).unwrap_or_default().to_owned();
@@ -201,6 +256,36 @@ mod tests {
         assert!(op.exige_red());
     }
 
+    /// F6-3: la seleccion va a `roles.revision` (con `regla` intacta); ya en
+    /// revision, solo se reescribe si cambia, sin fila nueva.
+    #[test]
+    fn la_seleccion_se_anota_en_roles_revision() {
+        let sel = |skills: &[&str]| AnotacionRevision {
+            agente: Some("general-purpose".to_owned()),
+            modelo: "opus".to_owned(),
+            skills: skills.iter().map(|s| (*s).to_owned()).collect(),
+        };
+        let antes = md("en-curso", ana().asignado().into());
+        let op = Revisar::nueva("TASK-001", &ana(), AHORA).con_seleccion(sel(&["p:rust"]));
+        let texto = match op.aplicar(&estado(antes)) {
+            Ok(c) => c.first().map(|c| c.contenido.clone()).unwrap_or_default(),
+            Err(_) => String::new(),
+        };
+        assert!(
+            texto.contains("revision: {agente: \"general-purpose\", modelo: \"opus\", skills: [\"p:rust\"], regla: null}"),
+            "{texto}"
+        );
+        let ya = texto.replace("estado: \"en-curso\"", "estado: \"en-revision\"");
+        assert_eq!(op.aplicar(&estado(ya.clone())), Ok(Vec::new()));
+        let otra = Revisar::nueva("TASK-001", &ana(), AHORA).con_seleccion(sel(&["p:vue"]));
+        let r = otra.aplicar(&estado(ya.clone()));
+        let nuevo = r
+            .ok()
+            .and_then(|c| c.first().map(|c| c.contenido.clone()))
+            .unwrap_or_default();
+        assert_eq!(nuevo, ya.replace("[\"p:rust\"]", "[\"p:vue\"]"));
+    }
+
     #[test]
     fn anotacion_ida_y_vuelta() {
         let op = Revisar::nueva("TASK-001", &ana(), AHORA);
diff --git a/crates/tc2/src/seleccion/catalogo.rs b/crates/tc2/src/seleccion/catalogo.rs
new file mode 100644
index 0000000..827facd
--- /dev/null
+++ b/crates/tc2/src/seleccion/catalogo.rs
@@ -0,0 +1,296 @@
+//! Catalogo de revisoras (F1-15 decisiones 2 y 8): los metadatos de seleccion
+//! que cada skill declara en su bloque ```` ```yaml ```` (`rol: revisor`,
+//! `patrones_archivo`, y en la generica `fallback` y `umbral_dominios`).
+//!
+//! Se apoya en [`crate::inventario`] sin cambiarlo: el inventario dice que
+//! existe; el catalogo lee, de las mismas carpetas, que declara cada skill.
+//! Solo se lee la cabecera de cada `SKILL.md` ([`MAX_CABECERA`]): un bloque
+//! que cae mas alla no cuenta (la skill queda sin metadatos y no se elige).
+//!
+//! Fuentes:
+//! - [`de_carpeta`]: `<dir>/<n>/SKILL.md` de un plugin dado por ruta (el
+//!   revisor en CI, con el plugin a una version fijada) o de una carpeta de
+//!   skills;
+//! - [`de_instalados`]: los plugins habilitados del inventario, por su
+//!   `installPath` de `installed_plugins.json`;
+//! - [`de_base`]: `.claude/skills` del proyecto **en la base**
+//!   (`origin/develop`), nunca del worktree del PR (decision 2).
+
+use std::io::Read;
+use std::path::{Path, PathBuf};
+
+use serde::Deserialize;
+use serde_json::Value;
+
+use crate::contrato::Informe;
+use crate::git::{Git, Subcomando};
+use crate::inventario::{self, Entrada, Inventario, MAX_CABECERA, Origen, Tipo};
+
+/// Una revisora del catalogo.
+#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
+pub struct Revisora {
+    /// Id del inventario (`plugin:n` en un plugin).
+    pub id: String,
+    /// Globs de `patrones_archivo`.
+    pub patrones: Vec<String>,
+    /// `fallback: true` (la generica): no compite por patrones.
+    pub fallback: bool,
+    /// `umbral_dominios` de la generica.
+    pub umbral_dominios: Option<u32>,
+    /// De donde se leyo (`SKILL.md` en disco o `<base>:<ruta>`).
+    pub ruta: String,
+}
+
+/// Las revisoras del catalogo, ordenadas por id y sin repetir.
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct Catalogo {
+    revisoras: Vec<Revisora>,
+}
+
+impl Catalogo {
+    /// Catalogo de `revisoras` en cualquier orden: el resultado no depende
+    /// del orden (se queda la primera de cada id tras ordenar).
+    pub fn nuevo(mut revisoras: Vec<Revisora>) -> Self {
+        revisoras.sort();
+        revisoras.dedup_by(|b, a| a.id == b.id);
+        Self { revisoras }
+    }
+
+    /// Las revisoras, por id.
+    pub fn revisoras(&self) -> &[Revisora] {
+        &self.revisoras
+    }
+
+    /// La revisora de id `id`.
+    pub fn revisora(&self, id: &str) -> Option<&Revisora> {
+        self.revisoras.iter().find(|r| r.id == id)
+    }
+}
+
+/// El bloque yaml de una skill.
+#[derive(Debug, Deserialize)]
+struct Bloque {
+    rol: Option<String>,
+    #[serde(default)]
+    patrones_archivo: Vec<String>,
+    #[serde(default)]
+    fallback: bool,
+    umbral_dominios: Option<u32>,
+}
+
+/// El primer bloque ```` ```yaml ```` de `cabecera` (el texto entre la
+/// valla de apertura y la de cierre), o `None` si no hay o no se cierra.
+pub fn bloque_yaml(cabecera: &str) -> Option<&str> {
+    let mut desde = None;
+    let mut pos = 0;
+    for l in cabecera.split_inclusive('\n') {
+        let t = l.trim();
+        match desde {
+            None if t == "```yaml" => desde = Some(pos + l.len()),
+            Some(d) if t == "```" => return cabecera.get(d..pos),
+            _ => {}
+        }
+        pos += l.len();
+    }
+    None
+}
+
+/// La revisora que declara el texto de una skill: `None` si no tiene bloque
+/// yaml, no es `rol: revisor` o el bloque no se entiende (lo instalado sin
+/// metadatos no se elige, decision 2).
+pub fn revisora_de(id: &str, texto: &str, ruta: &str) -> Option<Revisora> {
+    let mut corte = texto
+        .len()
+        .min(usize::try_from(MAX_CABECERA).unwrap_or(usize::MAX));
+    while !texto.is_char_boundary(corte) {
+        corte -= 1;
+    }
+    let cabecera = texto.get(..corte).unwrap_or_default();
+    let b: Bloque =
+        serde_saphyr::from_str_with_options(bloque_yaml(cabecera)?, crate::yaml::opciones())
+            .ok()?;
+    (b.rol.as_deref() == Some("revisor")).then(|| Revisora {
+        id: id.to_owned(),
+        patrones: b.patrones_archivo,
+        fallback: b.fallback,
+        umbral_dominios: b.umbral_dominios,
+        ruta: ruta.to_owned(),
+    })
+}
+
+/// Lo leido de una fuente: las entradas que existen y las revisoras.
+#[derive(Debug, Clone, Default)]
+pub struct Lectura {
+    /// Skills encontradas (para el inventario).
+    pub entradas: Vec<Entrada>,
+    /// Revisoras con metadatos.
+    pub revisoras: Vec<Revisora>,
+}
+
+/// Lee como mucho [`MAX_CABECERA`] bytes de `p`.
+fn cabecera(p: &Path) -> Option<String> {
+    let mut b = Vec::new();
+    std::fs::File::open(p)
+        .ok()?
+        .take(MAX_CABECERA)
+        .read_to_end(&mut b)
+        .ok()?;
+    Some(String::from_utf8_lossy(&b).into_owned())
+}
+
+/// Las skills de `skills` (`<n>/SKILL.md`) con el `prefijo` de ids
+/// (`plugin:` o vacio) y su `origen`.
+pub fn de_carpeta(skills: &Path, prefijo: &str, origen: &Origen) -> Lectura {
+    let mut l = Lectura::default();
+    let mut hijos: Vec<PathBuf> = std::fs::read_dir(skills)
+        .map(|d| d.filter_map(Result::ok).map(|e| e.path()).collect())
+        .unwrap_or_default();
+    hijos.sort();
+    for d in hijos {
+        let f = d.join("SKILL.md");
+        let (Some(carpeta), Some(texto)) = (
+            d.file_name().and_then(|n| n.to_str()).map(str::to_owned),
+            f.is_file().then(|| cabecera(&f)).flatten(),
+        ) else {
+            continue;
+        };
+        let nombre = inventario::nombre_del_frontmatter(&texto).unwrap_or(carpeta);
+        let id = format!("{prefijo}{nombre}");
+        l.entradas.push(Entrada {
+            tipo: Tipo::Skill,
+            id: id.clone(),
+            origen: origen.clone(),
+        });
+        if let Some(r) = revisora_de(&id, &texto, &f.display().to_string()) {
+            l.revisoras.push(r);
+        }
+    }
+    l
+}
+
+/// Las revisoras de los plugins habilitados de `inv`, leidas de su
+/// `installPath` en `installed_plugins.json` de `dir_claude` (la instalacion
+/// con la version del inventario o, si no la dice, la primera con ruta
+/// absoluta que existe). Los sincronizados de claude.ai no se leen (no traen
+/// revisoras con metadatos).
+pub fn de_instalados(dir_claude: &Path, inv: &Inventario) -> Vec<Revisora> {
+    let Ok(Some(Ok(v))) = inventario::leer_instalados(dir_claude) else {
+        return Vec::new();
+    };
+    let mut salida = Vec::new();
+    for p in inv.plugins() {
+        let Some(lista) = v
+            .get("plugins")
+            .and_then(|m| m.get(&p.clave))
+            .and_then(Value::as_array)
+        else {
+            continue;
+        };
+        let ruta = |x: &Value| {
+            x.get("installPath")
+                .and_then(Value::as_str)
+                .map(PathBuf::from)
+                .filter(|d| d.is_absolute() && d.is_dir())
+        };
+        let misma = lista.iter().find(|x| {
+            p.version.is_some() && x.get("version").and_then(Value::as_str) == p.version.as_deref()
+        });
+        let Some(dir) = misma.and_then(ruta).or_else(|| lista.iter().find_map(ruta)) else {
+            continue;
+        };
+        let origen = Origen::Plugin {
+            nombre: p.nombre.clone(),
+            version: p.version.clone(),
+        };
+        salida
+            .extend(de_carpeta(&dir.join("skills"), &format!("{}:", p.nombre), &origen).revisoras);
+    }
+    salida
+}
+
+/// Las skills de `.claude/skills` del proyecto en `base` (un commit o una
+/// ref, en produccion `origin/develop`), leidas con git: lo que el PR anada
+/// en su arbol no cuenta (decision 2).
+pub fn de_base(git: &Git, dir: &Path, base: &str) -> Result<Lectura, Informe> {
+    const OP: &str = "revisoras";
+    let listado = crate::git::exigir(
+        git,
+        dir,
+        OP,
+        Subcomando::LsTree,
+        &[
+            "-r",
+            "--full-tree",
+            "--name-only",
+            "--end-of-options",
+            base,
+            "--",
+            ".claude/skills",
+        ],
+    )?;
+    let rutas: Vec<&str> = listado
+        .lines()
+        .filter(|r| {
+            r.strip_prefix(".claude/skills/")
+                .and_then(|r| r.strip_suffix("/SKILL.md"))
+                .is_some_and(|n| !n.is_empty() && !n.contains('/'))
+        })
+        .collect();
+    let contenidos = crate::git::estado::leer_blobs(git, dir, OP, base, &rutas)?;
+    let mut l = Lectura::default();
+    for (ruta, texto) in rutas.iter().zip(contenidos) {
+        let Some(texto) = texto else { continue };
+        let carpeta = ruta
+            .trim_start_matches(".claude/skills/")
+            .trim_end_matches("/SKILL.md");
+        let nombre =
+            inventario::nombre_del_frontmatter(&texto).unwrap_or_else(|| carpeta.to_owned());
+        l.entradas.push(Entrada {
+            tipo: Tipo::Skill,
+            id: nombre.clone(),
+            origen: Origen::Proyecto,
+        });
+        if let Some(r) = revisora_de(&nombre, &texto, &format!("{base}:{ruta}")) {
+            l.revisoras.push(r);
+        }
+    }
+    Ok(l)
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn el_bloque_yaml_se_lee_y_lo_demas_no_es_revisora() {
+        let t = "---\nname: x\n---\n\ntexto\n\n```yaml\nrol: revisor\npatrones_archivo: [\"**/*.rs\"]   # nota\n```\n\nmas\n";
+        let r = revisora_de("p:x", t, "x");
+        assert_eq!(r.map(|r| r.patrones), Some(vec!["**/*.rs".to_owned()]));
+        // Sin bloque, sin cierre, otro rol o yaml roto: no es revisora.
+        assert!(revisora_de("p:x", "---\nname: x\n---\n", "x").is_none());
+        assert!(revisora_de("p:x", "```yaml\nrol: revisor\n", "x").is_none());
+        assert!(revisora_de("p:x", "```yaml\nrol: ejecucion\n```\n", "x").is_none());
+        assert!(revisora_de("p:x", "```yaml\nrol: [\n```\n", "x").is_none());
+        // Un bloque mas alla de la cabecera no cuenta.
+        let lejos = format!(
+            "{}\n```yaml\nrol: revisor\n```\n",
+            "a".repeat(MAX_CABECERA as usize)
+        );
+        assert!(revisora_de("p:x", &lejos, "x").is_none());
+    }
+
+    #[test]
+    fn el_catalogo_no_depende_del_orden() {
+        let r = |id: &str| Revisora {
+            id: id.to_owned(),
+            patrones: Vec::new(),
+            fallback: false,
+            umbral_dominios: None,
+            ruta: String::new(),
+        };
+        let a = Catalogo::nuevo(vec![r("b"), r("a"), r("b")]);
+        let b = Catalogo::nuevo(vec![r("a"), r("b")]);
+        assert_eq!(a, b);
+        assert_eq!(a.revisoras().len(), 2);
+    }
+}
diff --git a/crates/tc2/src/seleccion/mod.rs b/crates/tc2/src/seleccion/mod.rs
new file mode 100644
index 0000000..0b63154
--- /dev/null
+++ b/crates/tc2/src/seleccion/mod.rs
@@ -0,0 +1,555 @@
+//! Seleccion determinista de revisoras y modelo (F6-3; ADR F1-15 decision 1,
+//! adenda F6-3).
+//!
+//! [`revision`] es una funcion pura: recibe las rutas del diff (los dos
+//! nombres de cada renombrado), el catalogo y el inventario de la base, el
+//! carril efectivo, la complejidad y los modelos permitidos, y devuelve las
+//! revisoras, el lanzador, el modelo, las filas que se aplicaron y los
+//! avisos. No lee ficheros, no llama a la red ni a un modelo: recorre las
+//! filas de `revision` y `todos` de `tabla-seleccion` en su orden ([`tabla`])
+//! y la columna «Revisión» de `tabla-modelo`.
+//!
+//! Lo que F6-3 no evalua todavia (adenda F6-3 de F1-15): sin el detector
+//! `dominios()` de F4-6 la lista de dominios es vacia, asi que S10 y S11 no
+//! se cumplen nunca; sin rondas de F6-6 la ronda es 1 y S13 no se cumple.
+//! Las filas se leen igual y su vocabulario se valida.
+
+pub mod catalogo;
+pub mod orden;
+pub mod tabla;
+
+use std::collections::BTreeMap;
+
+use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
+
+use crate::carril::Carril;
+use crate::inventario::{Inventario, Tipo};
+
+pub use catalogo::{Catalogo, Revisora};
+use tabla::{FilaSeleccion, RolFila};
+
+/// Umbral de dominios si la generica no lo declara (el de la 0.16).
+pub const UMBRAL_POR_DEFECTO: u32 = 3;
+
+/// Lo que necesita la seleccion ademas de las rutas, el catalogo y el
+/// inventario.
+#[derive(Debug, Clone)]
+pub struct Contexto<'a> {
+    /// Carril efectivo (F1-10).
+    pub carril: Carril,
+    /// Complejidad: la mayor entre la declarada y la heuristica; `None` si
+    /// no se sabe (se toma `media`, con aviso).
+    pub complejidad: Option<&'a str>,
+    /// `seleccion.modelos_permitidos` de la configuracion de la base.
+    pub modelos_permitidos: &'a [String],
+    /// Ronda de revision (1 hasta F6-6).
+    pub ronda: u32,
+}
+
+/// La seleccion de revision.
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct Seleccion {
+    /// Revisoras (skills), por id.
+    pub revisoras: Vec<String>,
+    /// Agentes que lanzan cada revisora (S12).
+    pub agentes: Vec<String>,
+    /// Modelo de la escala (`haiku`, `sonnet` u `opus`).
+    pub modelo: String,
+    /// Ids de las filas que se aplicaron (la de `tabla-modelo` primero).
+    pub reglas: Vec<String>,
+    /// Avisos (`no-instalado:<id>`, `complejidad-desconocida`...).
+    pub avisos: Vec<String>,
+    /// Rutas que recibe cada revisora (la de dominio, las que casan con sus
+    /// patrones; la generica, las sueltas o todas).
+    pub rutas: BTreeMap<String, Vec<String>>,
+}
+
+/// Id del modelo en la API de Anthropic para un modelo de la escala.
+pub fn modelo_api(modelo: &str) -> &'static str {
+    match modelo {
+        "haiku" => "claude-haiku-5-5",
+        "opus" => "claude-opus-5-5",
+        _ => "claude-sonnet-5-5",
+    }
+}
+
+/// Los patrones de una revisora como un conjunto de globs: sin distinguir
+/// mayusculas (F1-15 decision 3) y con `*` sin cruzar `/`. Un patron que no
+/// es un glob valido no casa con nada.
+fn globs(patrones: &[String]) -> GlobSet {
+    let mut b = GlobSetBuilder::new();
+    for p in patrones {
+        if let Ok(g) = GlobBuilder::new(p)
+            .case_insensitive(true)
+            .literal_separator(true)
+            .build()
+        {
+            b.add(g);
+        }
+    }
+    b.build().unwrap_or_else(|_| GlobSet::empty())
+}
+
+/// Ruta con `/` y sin `./` delante.
+fn normalizar(ruta: &str) -> String {
+    let r = ruta.trim().replace('\\', "/");
+    r.strip_prefix("./").map_or(r.clone(), str::to_owned)
+}
+
+/// Lo que se calcula de las rutas antes de recorrer la tabla.
+struct Casan {
+    /// Revisora de dominio -> rutas que casan.
+    por_revisora: BTreeMap<String, Vec<String>>,
+    /// Rutas que no casan con ninguna.
+    sueltas: Vec<String>,
+}
+
+fn casar(rutas: &[String], cat: &Catalogo) -> Casan {
+    let mut por_revisora: BTreeMap<String, Vec<String>> = BTreeMap::new();
+    let mut sueltas = Vec::new();
+    let conjuntos: Vec<(&str, GlobSet)> = cat
+        .revisoras()
+        .iter()
+        .filter(|r| !r.fallback && !r.patrones.is_empty())
+        .map(|r| (r.id.as_str(), globs(&r.patrones)))
+        .collect();
+    for ruta in rutas {
+        let mut alguna = false;
+        for (id, g) in &conjuntos {
+            if g.is_match(ruta) {
+                alguna = true;
+                por_revisora
+                    .entry((*id).to_owned())
+                    .or_default()
+                    .push(ruta.clone());
+            }
+        }
+        if !alguna {
+            sueltas.push(ruta.clone());
+        }
+    }
+    Casan {
+        por_revisora,
+        sueltas,
+    }
+}
+
+/// Modelo de revision de `tabla-modelo` para el carril y la complejidad.
+fn fila_modelo(filas: &[tabla::FilaModelo], carril: Carril, cx: &str) -> Option<(String, String)> {
+    filas
+        .iter()
+        .find(|f| {
+            f.carril == carril.nombre()
+                && (f.complejidades.is_empty() || f.complejidades.iter().any(|x| x == cx))
+        })
+        .map(|f| (f.id.clone(), f.revision.clone()))
+}
+
+/// El permitido mas cercano por debajo de `modelo`; si no hay, por encima.
+fn restringir(modelo: &str, permitidos: &[String]) -> Option<String> {
+    let i = tabla::ESCALA.iter().position(|m| *m == modelo)?;
+    let ok = |j: usize| {
+        tabla::ESCALA
+            .get(j)
+            .filter(|m| permitidos.iter().any(|p| p == *m))
+            .map(|m| (*m).to_owned())
+    };
+    (0..i)
+        .rev()
+        .find_map(ok)
+        .or_else(|| (i + 1..tabla::ESCALA.len()).find_map(ok))
+}
+
+/// Estado mientras se recorren las filas.
+struct Estado<'a> {
+    sel: Seleccion,
+    casan: Casan,
+    todas: Vec<String>,
+    umbral: u32,
+    generica: Option<String>,
+    inv: &'a Inventario,
+    ctx: &'a Contexto<'a>,
+}
+
+impl Estado<'_> {
+    fn instalado(&self, id: &str) -> bool {
+        self.inv.contiene(Tipo::Skill, id) || self.inv.contiene(Tipo::Agente, id)
+    }
+
+    fn atomo(&self, texto: &str) -> bool {
+        let n = u32::try_from(self.casan.por_revisora.len()).unwrap_or(u32::MAX);
+        let elegidos = || self.sel.revisoras.iter().chain(&self.sel.agentes);
+        match texto {
+            "siempre" => true,
+            "rutas-casan-entre-1-y-umbral" => (1..=self.umbral).contains(&n),
+            "rutas-no-casan-o-pasan-umbral" => n == 0 || n > self.umbral,
+            "hay-no-instalado" => elegidos().any(|id| !self.instalado(id)),
+            "modelo-no-permitido" => !self.ctx.modelos_permitidos.contains(&self.sel.modelo),
+            // Sin detector de dominios (F4-6) ni skills de diseno o ejecucion
+            // en la revision: no se cumplen.
+            "dominio-con-revisora"
+            | "dominio-sin-revisora"
+            | "hay-skill-diseno-del-dominio"
+            | "hay-skill-ejecucion-con-solape" => false,
+            t => {
+                if let Some(c) = t.strip_prefix("carril!=") {
+                    c != self.ctx.carril.nombre()
+                } else if let Some(c) = t.strip_prefix("carril=") {
+                    c == self.ctx.carril.nombre()
+                } else if let Some(r) = t.strip_prefix("ronda>=") {
+                    r.parse().is_ok_and(|r: u32| self.ctx.ronda >= r)
+                } else {
+                    // `dominio=` (lista vacia) y `roles>=` (solo diseno).
+                    false
+                }
+            }
+        }
+    }
+
+    /// Aplica la accion de `f`; `true` si cambio algo.
+    fn aplicar(&mut self, f: &FilaSeleccion) -> bool {
+        let antes = self.sel.clone();
+        match f.accion.as_str() {
+            "revisoras-de-rutas+generica-si-sueltas" => {
+                for (id, rutas) in &self.casan.por_revisora {
+                    self.sel.revisoras.push(id.clone());
+                    self.sel.rutas.insert(id.clone(), rutas.clone());
+                }
+                if !self.casan.sueltas.is_empty()
+                    && let Some(g) = &self.generica
+                {
+                    self.sel.revisoras.push(g.clone());
+                    self.sel.rutas.insert(g.clone(), self.casan.sueltas.clone());
+                }
+            }
+            "generica-del-plugin" => {
+                if let Some(g) = &self.generica {
+                    self.sel.revisoras = vec![g.clone()];
+                    self.sel.rutas = BTreeMap::from([(g.clone(), self.todas.clone())]);
+                }
+            }
+            "lanzador" => self.sel.agentes.extend(f.ids.iter().cloned()),
+            "quitar-no-instalado" => {
+                let fuera: Vec<String> = self
+                    .sel
+                    .revisoras
+                    .iter()
+                    .chain(&self.sel.agentes)
+                    .filter(|id| !self.instalado(id))
+                    .cloned()
+                    .collect();
+                for id in &fuera {
+                    self.sel.avisos.push(format!("no-instalado:{id}"));
+                    self.sel.rutas.remove(id);
+                }
+                self.sel.revisoras.retain(|id| !fuera.contains(id));
+                self.sel.agentes.retain(|id| !fuera.contains(id));
+            }
+            "restringir-modelo" => {
+                if let Some(m) = restringir(&self.sel.modelo, self.ctx.modelos_permitidos) {
+                    self.sel
+                        .avisos
+                        .push(format!("modelo-no-permitido:{}->{m}", self.sel.modelo));
+                    self.sel.modelo = m;
+                }
+            }
+            a => {
+                if let Some(n) = a
+                    .strip_prefix("modelo-baja=")
+                    .and_then(|n| n.parse::<usize>().ok())
+                {
+                    let i = tabla::ESCALA
+                        .iter()
+                        .position(|m| *m == self.sel.modelo)
+                        .unwrap_or(0);
+                    if let Some(m) = tabla::ESCALA.get(i.saturating_sub(n)) {
+                        (*m).clone_into(&mut self.sel.modelo);
+                    }
+                }
+                // `revisora-del-dominio` y `aviso=dominio-sin-revisora`: sin
+                // dominios no hay nada que anadir.
+            }
+        }
+        self.sel.revisoras.sort();
+        self.sel.revisoras.dedup();
+        self.sel.agentes.sort();
+        self.sel.agentes.dedup();
+        self.sel != antes
+    }
+}
+
+/// La seleccion de revision para las `rutas` del diff. `Err` si las tablas
+/// del ADR no se leen (no pasa con el ADR compilado: lo prueba
+/// `las_tablas_del_adr_se_leen`).
+pub fn revision(
+    rutas: &[String],
+    cat: &Catalogo,
+    inv: &Inventario,
+    ctx: &Contexto<'_>,
+) -> Result<Seleccion, String> {
+    revision_con(tabla::ADR, rutas, cat, inv, ctx)
+}
+
+/// Como [`revision`], con las tablas de `adr` (para probar variaciones).
+pub fn revision_con(
+    adr: &str,
+    rutas: &[String],
+    cat: &Catalogo,
+    inv: &Inventario,
+    ctx: &Contexto<'_>,
+) -> Result<Seleccion, String> {
+    let filas = tabla::seleccion(adr)?;
+    let modelos = tabla::modelo(adr)?;
+    let mut todas: Vec<String> = rutas
+        .iter()
+        .map(|r| normalizar(r))
+        .filter(|r| !r.is_empty())
+        .collect();
+    todas.sort();
+    todas.dedup();
+    // La generica es la de la fila `generica-del-plugin` (decision 6: solo
+    // la del plugin; una skill del proyecto con `fallback: true` no la
+    // sustituye).
+    let generica = filas
+        .iter()
+        .find(|f| f.accion == "generica-del-plugin")
+        .and_then(|f| f.ids.first().cloned());
+    let umbral = generica
+        .as_deref()
+        .and_then(|g| cat.revisora(g))
+        .and_then(|r| r.umbral_dominios)
+        .unwrap_or(UMBRAL_POR_DEFECTO);
+    let mut sel = Seleccion::default();
+    let cx = match ctx.complejidad {
+        Some(c) if crate::tarea::COMPLEJIDADES.contains(&c) => c,
+        _ => {
+            sel.avisos.push("complejidad-desconocida".to_owned());
+            "media"
+        }
+    };
+    let (id_modelo, modelo) = fila_modelo(&modelos, ctx.carril, cx)
+        .ok_or_else(|| format!("tabla-modelo sin fila para {}/{cx}", ctx.carril.nombre()))?;
+    sel.modelo = modelo;
+    sel.reglas.push(id_modelo);
+    let mut e = Estado {
+        sel,
+        casan: casar(&todas, cat),
+        todas,
+        umbral,
+        generica,
+        inv,
+        ctx,
+    };
+    for f in filas
+        .iter()
+        .filter(|f| matches!(f.rol, RolFila::Revision | RolFila::Todos))
+    {
+        let cumple = f.condicion.iter().all(|a| e.atomo(&a.texto) != a.negado);
+        if cumple && e.aplicar(f) {
+            e.sel.reglas.push(f.id.clone());
+        }
+    }
+    Ok(e.sel)
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::inventario::{Entrada, Origen};
+
+    /// El bloque yaml de las revisoras del plugin 0.16.2, copiado tal cual
+    /// (fixtures de `tests/fixtures/revisoras/`).
+    fn fixture(n: &str) -> String {
+        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
+            .join("tests/fixtures/revisoras/skills")
+            .join(n)
+            .join("SKILL.md");
+        std::fs::read_to_string(p).unwrap_or_default()
+    }
+
+    const REVISORAS: [&str; 5] = [
+        "angular-vue-reviewer",
+        "code-quality-reviewer",
+        "csharp-autocad-ifc-reviewer",
+        "java-spring-reviewer",
+        "rust-wasm-reviewer",
+    ];
+
+    fn catalogo(sin: &[&str]) -> (Catalogo, Inventario) {
+        let mut rs = Vec::new();
+        let mut es = vec![Entrada {
+            tipo: Tipo::Agente,
+            id: "general-purpose".to_owned(),
+            origen: Origen::Integrado,
+        }];
+        for n in REVISORAS {
+            let id = format!("taskcode-plugin:{n}");
+            if let Some(r) = catalogo::revisora_de(&id, &fixture(n), n) {
+                rs.push(r);
+            }
+            if !sin.contains(&n) {
+                es.push(Entrada {
+                    tipo: Tipo::Skill,
+                    id,
+                    origen: Origen::Plugin {
+                        nombre: "taskcode-plugin".to_owned(),
+                        version: Some("0.16.2".to_owned()),
+                    },
+                });
+            }
+        }
+        (Catalogo::nuevo(rs), Inventario::nuevo(es, Vec::new()))
+    }
+
+    fn todos() -> Vec<String> {
+        tabla::ESCALA.iter().map(|m| (*m).to_owned()).collect()
+    }
+
+    fn sel(rutas: &[&str], sin: &[&str], cx: Option<&str>) -> Seleccion {
+        let (c, i) = catalogo(sin);
+        let permitidos = todos();
+        let ctx = Contexto {
+            carril: Carril::Funcionalidad,
+            complejidad: cx,
+            modelos_permitidos: &permitidos,
+            ronda: 1,
+        };
+        let rutas: Vec<String> = rutas.iter().map(|r| (*r).to_owned()).collect();
+        revision(&rutas, &c, &i, &ctx).unwrap_or_default()
+    }
+
+    /// Las cinco revisoras de los fixtures tienen su bloque yaml dentro de
+    /// la cabecera (suposicion del plan).
+    #[test]
+    fn las_cinco_fixtures_son_revisoras() {
+        let (c, _) = catalogo(&[]);
+        assert_eq!(c.revisoras().len(), 5);
+        let g = c.revisora("taskcode-plugin:code-quality-reviewer");
+        assert_eq!(
+            g.map(|g| (g.fallback, g.umbral_dominios)),
+            Some((true, Some(3)))
+        );
+    }
+
+    /// C1: un diff Rust + Vue lanza las dos de dominio, sin generica, con la
+    /// regla S08, el lanzador S12 y el modelo M04; sin red ni modelo.
+    #[test]
+    fn rust_y_vue_lanzan_las_dos_de_dominio() {
+        let s = sel(&["src/a.rs", "web/App.vue"], &[], Some("simple"));
+        assert_eq!(
+            s.revisoras,
+            [
+                "taskcode-plugin:angular-vue-reviewer",
+                "taskcode-plugin:rust-wasm-reviewer"
+            ]
+        );
+        assert_eq!(s.agentes, ["general-purpose"]);
+        assert_eq!(s.modelo, "sonnet");
+        assert_eq!(s.reglas, ["M04", "S08", "S12"]);
+        assert!(s.avisos.is_empty(), "{:?}", s.avisos);
+        assert_eq!(
+            s.rutas.get("taskcode-plugin:rust-wasm-reviewer"),
+            Some(&vec!["src/a.rs".to_owned()])
+        );
+        // Sin distinguir mayusculas y con `\`.
+        let s = sel(&["SRC\\B.RS", "web/x.VUE"], &[], Some("simple"));
+        assert_eq!(s.revisoras.len(), 2, "{s:?}");
+    }
+
+    /// C2: un README suelto va a la generica (S09); con Rust y un README, la
+    /// de Rust y la generica con las sueltas (S08); mas de tres dominios, la
+    /// generica sola (S09).
+    #[test]
+    fn la_generica_por_descarte_y_por_exceso() {
+        let s = sel(&["README.md"], &[], Some("simple"));
+        assert_eq!(s.revisoras, ["taskcode-plugin:code-quality-reviewer"]);
+        assert_eq!(s.reglas, ["M04", "S09", "S12"]);
+        let s = sel(&["src/a.rs", "README.md"], &[], Some("simple"));
+        assert_eq!(s.revisoras.len(), 2);
+        assert_eq!(
+            s.rutas.get("taskcode-plugin:code-quality-reviewer"),
+            Some(&vec!["README.md".to_owned()])
+        );
+        let s = sel(&["a.rs", "b.vue", "C.java", "d.cs"], &[], Some("media"));
+        assert_eq!(s.revisoras, ["taskcode-plugin:code-quality-reviewer"]);
+        assert_eq!(s.reglas, ["M05", "S09", "S12"]);
+        assert_eq!(s.modelo, "opus");
+        assert_eq!(s.rutas.values().map(Vec::len).sum::<usize>(), 4);
+    }
+
+    /// C2: lo que la seleccion elige y no esta instalado se quita con aviso
+    /// `no-instalado` y la fila S14.
+    #[test]
+    fn la_no_instalada_se_quita_con_aviso() {
+        let s = sel(
+            &["src/a.rs", "web/App.vue"],
+            &["angular-vue-reviewer"],
+            Some("simple"),
+        );
+        assert_eq!(s.revisoras, ["taskcode-plugin:rust-wasm-reviewer"]);
+        assert_eq!(
+            s.avisos,
+            ["no-instalado:taskcode-plugin:angular-vue-reviewer"]
+        );
+        assert_eq!(s.reglas, ["M04", "S08", "S12", "S14"]);
+        let s = sel(&["README.md"], &["code-quality-reviewer"], Some("simple"));
+        assert!(s.revisoras.is_empty());
+        assert_eq!(
+            s.avisos,
+            ["no-instalado:taskcode-plugin:code-quality-reviewer"]
+        );
+    }
+
+    /// Modelo: complejidad desconocida (media con aviso), carril y
+    /// `modelos_permitidos` (S15); el orden de entrada no cambia nada.
+    #[test]
+    fn modelo_por_carril_complejidad_y_permitidos() {
+        let s = sel(&["a.rs"], &[], None);
+        assert_eq!((s.modelo.as_str(), s.reglas[0].as_str()), ("opus", "M05"));
+        assert_eq!(s.avisos, ["complejidad-desconocida"]);
+        let (c, i) = catalogo(&[]);
+        let permitidos = vec!["haiku".to_owned(), "sonnet".to_owned()];
+        let ctx = Contexto {
+            carril: Carril::RiesgoAlto,
+            complejidad: Some("trivial"),
+            modelos_permitidos: &permitidos,
+            ronda: 1,
+        };
+        let r = |v: &[&str]| {
+            let v: Vec<String> = v.iter().map(|x| (*x).to_owned()).collect();
+            revision(&v, &c, &i, &ctx).unwrap_or_default()
+        };
+        let s = r(&["a.rs", "b.vue"]);
+        assert_eq!(s.modelo, "sonnet");
+        assert_eq!(s.reglas, ["M07", "S08", "S12", "S15"]);
+        assert_eq!(s.avisos, ["modelo-no-permitido:opus->sonnet"]);
+        assert_eq!(r(&["b.vue", "a.rs", "a.rs"]), s);
+        assert_eq!(modelo_api("opus"), "claude-opus-5-5");
+        assert_eq!(modelo_api("haiku"), "claude-haiku-5-5");
+        assert_eq!(modelo_api("sonnet"), "claude-sonnet-5-5");
+    }
+
+    /// Las reglas salen de la tabla: cambiar la accion de S12 en una copia
+    /// del ADR cambia el resultado (no estan copiadas en el codigo).
+    #[test]
+    fn las_reglas_se_leen_de_la_tabla() {
+        let (c, i) = catalogo(&[]);
+        let permitidos = todos();
+        let ctx = Contexto {
+            carril: Carril::Funcionalidad,
+            complejidad: Some("simple"),
+            modelos_permitidos: &permitidos,
+            ronda: 1,
+        };
+        let rutas = vec!["a.rs".to_owned()];
+        let adr = tabla::ADR.replace(
+            "| S12 | `revision` | `siempre` | `lanzador` | `general-purpose` |",
+            "| S12 | `revision` | `carril=fix` | `lanzador` | `general-purpose` |",
+        );
+        assert_ne!(adr, tabla::ADR);
+        let s = revision_con(&adr, &rutas, &c, &i, &ctx).unwrap_or_default();
+        assert!(s.agentes.is_empty(), "{s:?}");
+        let roto = tabla::ADR.replace("`lanzador`", "`a-ojo`");
+        assert!(revision_con(&roto, &rutas, &c, &i, &ctx).is_err());
+    }
+}
diff --git a/crates/tc2/src/seleccion/orden.rs b/crates/tc2/src/seleccion/orden.rs
new file mode 100644
index 0000000..974f8a0
--- /dev/null
+++ b/crates/tc2/src/seleccion/orden.rs
@@ -0,0 +1,395 @@
+//! `tc2 revisoras [--json] --rutas <fichero|-> [--tarea TASK-NNN]
+//! [--carril <c>] [--complejidad <x>] [--plugin <nombre>=<dir>]...
+//! [--informe <fichero> --seccion <salida> [--artefacto <n>]]
+//! [--detalle <salida.json>]` (F6-3).
+//!
+//! Las rutas son dato (una por linea; los dos nombres de cada renombrado):
+//! en CI salen de `git diff --name-only --no-renames base...head` sobre el
+//! checkout de la base; en local, de [`crate::git::ficheros_de_rama`]. La
+//! configuracion y las skills del proyecto se leen de la base
+//! (`origin/develop`), nunca del worktree del PR (F1-15 decision 2).
+//!
+//! Inventario y catalogo:
+//! - con `--plugin <nombre>=<dir>` (el revisor en CI, con el plugin a una
+//!   version fijada), solo esos plugins (`<dir>/skills/<n>/SKILL.md`), las
+//!   skills del proyecto en la base y los agentes integrados: no se lee la
+//!   carpeta de Claude Code de la maquina, y el resultado no depende de lo
+//!   que tenga instalado el runner;
+//! - sin `--plugin`, el inventario de la maquina ([`crate::inventario::leer`])
+//!   con las skills del proyecto sustituidas por las de la base.
+//!
+//! Carril y complejidad: con `--tarea`, los de la tarea en la rama de estado
+//! (carril: el mayor del declarado por el tipo y el congelado; complejidad:
+//! la mayor entre la declarada y la heuristica de la 0.16). `--carril` solo
+//! sube; `--complejidad` manda sobre la de la tarea. Sin nada: funcionalidad
+//! y complejidad desconocida (media, con aviso).
+//!
+//! `--informe` (seguimiento de TASK-044): el informe de verificacion de F5-4
+//! se lee con un tope de [`TOPE_INFORME`] bytes; si vale, `--seccion` recibe
+//! [`seccion_revisor`] y la linea dice `informe=verde|rojo`; si no,
+//! `informe=grande|invalido|sin-informe` y no se escribe seccion.
+//!
+//! Linea: `revisoras ok revisoras=<ids> agentes=<ids> modelo=<m>
+//! modelo-api=<id> reglas=<ids> avisos=<...> carril=<c> complejidad=<x>
+//! [informe=<...>]`. Lo de cada revisora (su `SKILL.md` y las rutas que
+//! recibe) no cabe en claves del contrato F1-4: va, con `--detalle`, a un
+//! JSON `{"revisoras":[{"id","skill","rutas"}]}` que lee el workflow.
+
+use std::io::Read;
+use std::path::{Path, PathBuf};
+
+use super::catalogo::{self, Catalogo};
+use super::{Contexto, Seleccion};
+use crate::carril::Carril;
+use crate::contrato::{Informe, Resultado};
+use crate::git::Git;
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::seleccion as txt;
+use crate::inventario::{self, Inventario, Origen};
+use crate::plan::heuristica;
+use crate::tarea::{Cuerpo, Tarea};
+use crate::verificacion::informe::{self, seccion_revisor};
+
+/// Nombre de la orden.
+pub const NOMBRE: &str = "revisoras";
+
+/// Bytes que se leen como mucho del informe de verificacion: un informe v1
+/// real ocupa menos de 2 KiB; lo que pase de aqui no se cita.
+pub const TOPE_INFORME: u64 = 64 * 1024;
+
+/// Bytes que se leen como mucho del fichero de rutas.
+pub const TOPE_RUTAS: u64 = 4 * 1024 * 1024;
+
+/// Lo que necesita la orden del entorno.
+pub struct Entorno<'a> {
+    /// Carpeta del repo (el checkout de la base en CI).
+    pub dir: &'a Path,
+    /// Puerto Git.
+    pub git: &'a Git,
+    /// Carpeta de Claude Code, si se sabe.
+    pub claude: Option<&'a Path>,
+    /// Lee la entrada estandar (para `--rutas -`).
+    pub stdin: &'a dyn Fn() -> std::io::Result<String>,
+}
+
+/// Inventario y catalogo con los que se selecciona, y los avisos de
+/// leerlos.
+#[derive(Debug, Clone, Default)]
+pub struct Fuentes {
+    /// Lo instalado.
+    pub inventario: Inventario,
+    /// Las revisoras con metadatos.
+    pub catalogo: Catalogo,
+    /// Avisos (`sin-base`).
+    pub avisos: Vec<String>,
+}
+
+/// Inventario y catalogo para `dir` (ver el comentario del modulo).
+pub fn fuentes(
+    g: &Git,
+    dir: &Path,
+    claude: Option<&Path>,
+    plugins: &[(String, PathBuf)],
+) -> Fuentes {
+    let mut avisos = Vec::new();
+    let base = match crate::git::commit_de(g, dir, NOMBRE, crate::config::BASE) {
+        Ok(Some(_)) => catalogo::de_base(g, dir, crate::config::BASE).unwrap_or_else(|_| {
+            avisos.push("sin-skills-de-base".to_owned());
+            catalogo::Lectura::default()
+        }),
+        _ => {
+            avisos.push("sin-base".to_owned());
+            catalogo::Lectura::default()
+        }
+    };
+    let mut entradas = base.entradas;
+    let mut revisoras = base.revisoras;
+    let mut lista_plugins = Vec::new();
+    if plugins.is_empty() {
+        let l = inventario::leer(claude, dir);
+        if let Some(c) = claude {
+            revisoras.extend(catalogo::de_instalados(c, &l.inventario));
+        }
+        entradas.extend(
+            l.inventario
+                .entradas()
+                .iter()
+                .filter(|e| e.origen != Origen::Proyecto)
+                .cloned(),
+        );
+        lista_plugins.extend(l.inventario.plugins().iter().cloned());
+    } else {
+        for (nombre, d) in plugins {
+            let origen = Origen::Plugin {
+                nombre: nombre.clone(),
+                version: None,
+            };
+            let l = catalogo::de_carpeta(&d.join("skills"), &format!("{nombre}:"), &origen);
+            entradas.extend(l.entradas);
+            revisoras.extend(l.revisoras);
+        }
+        for id in inventario::INTEGRADOS {
+            entradas.push(inventario::Entrada {
+                tipo: inventario::Tipo::Agente,
+                id: id.to_owned(),
+                origen: Origen::Integrado,
+            });
+        }
+    }
+    Fuentes {
+        inventario: Inventario::nuevo(entradas, lista_plugins),
+        catalogo: Catalogo::nuevo(revisoras),
+        avisos,
+    }
+}
+
+/// Carril efectivo y complejidad de una tarea: el carril mayor entre el
+/// declarado por el tipo y el congelado; la complejidad mayor entre la
+/// declarada y la heuristica (F1-15 decision 1).
+pub fn de_tarea(t: &Tarea, c: &Cuerpo) -> (Carril, &'static str) {
+    let declarado = Carril::declarado(&t.tipo);
+    let carril = t
+        .carril
+        .as_deref()
+        .and_then(Carril::desde)
+        .map_or(declarado, |x| x.max(declarado));
+    let e = heuristica::Enunciado::de(c);
+    let r = heuristica::resolver(
+        t.complejidad.as_deref(),
+        &t.etiquetas,
+        &t.tipo,
+        &t.dependencias,
+        &e,
+    );
+    let indice = |n: &str| heuristica::NIVELES.iter().position(|x| *x == n);
+    let cx = r
+        .declarado
+        .as_deref()
+        .and_then(indice)
+        .into_iter()
+        .chain(indice(r.heuristico))
+        .max()
+        .and_then(|i| heuristica::NIVELES.get(i).copied())
+        .unwrap_or(r.heuristico);
+    (carril, cx)
+}
+
+/// Las claves de la seleccion en un informe.
+pub fn con_seleccion(i: Informe, s: &Seleccion) -> Informe {
+    i.con_lista("revisoras", s.revisoras.clone())
+        .con_lista("agentes", s.agentes.clone())
+        .con("modelo", s.modelo.as_str())
+        .con("modelo-api", super::modelo_api(&s.modelo))
+        .con_lista("reglas", s.reglas.clone())
+        .con_lista("avisos", s.avisos.clone())
+}
+
+/// El detalle por revisora: `{"revisoras":[{"id","skill","rutas"}]}`, en el
+/// orden de la seleccion.
+pub fn detalle(s: &Seleccion, cat: &Catalogo) -> String {
+    let v: Vec<serde_json::Value> = s
+        .revisoras
+        .iter()
+        .map(|id| {
+            serde_json::json!({
+                "id": id,
+                "skill": cat.revisora(id).map(|r| r.ruta.clone()),
+                "rutas": s.rutas.get(id).cloned().unwrap_or_default(),
+            })
+        })
+        .collect();
+    serde_json::json!({ "revisoras": v }).to_string()
+}
+
+/// Lee como mucho `tope` bytes de `p`: `Ok(None)` si pasa del tope.
+fn leer_con_tope(p: &Path, tope: u64) -> std::io::Result<Option<String>> {
+    let mut b = Vec::new();
+    std::fs::File::open(p)?.take(tope + 1).read_to_end(&mut b)?;
+    if b.len() as u64 > tope {
+        return Ok(None);
+    }
+    Ok(Some(String::from_utf8_lossy(&b).into_owned()))
+}
+
+/// Estado del informe de verificacion y, si vale, su seccion para el
+/// revisor (artefacto `artefacto`).
+pub fn informe_revisor(p: &Path, artefacto: &str) -> (&'static str, Option<String>) {
+    match leer_con_tope(p, TOPE_INFORME) {
+        Err(_) => ("sin-informe", None),
+        Ok(None) => ("grande", None),
+        Ok(Some(t)) => match informe::leer(&t) {
+            Ok(i) => {
+                let estado = match i.resultado {
+                    informe::Resultado::Verde => "verde",
+                    informe::Resultado::Rojo => "rojo",
+                };
+                (estado, Some(seccion_revisor(&i, artefacto)))
+            }
+            Err(_) => ("invalido", None),
+        },
+    }
+}
+
+/// Argumentos de la orden.
+#[derive(Default)]
+struct Args {
+    rutas: Option<String>,
+    tarea: Option<String>,
+    carril: Option<Carril>,
+    complejidad: Option<String>,
+    plugins: Vec<(String, PathBuf)>,
+    informe: Option<PathBuf>,
+    artefacto: Option<String>,
+    seccion: Option<PathBuf>,
+    detalle: Option<PathBuf>,
+}
+
+fn uso(motivo: &str) -> Informe {
+    Informe::nuevo(NOMBRE, Resultado::Uso)
+        .con("motivo", motivo)
+        .diagnostico(&txt::USO_REVISORAS)
+}
+
+fn args(a: &[String]) -> Result<Args, Informe> {
+    let mut r = Args::default();
+    let mut it = a.iter();
+    while let Some(x) = it.next() {
+        let mut valor = || it.next().cloned().ok_or_else(|| uso("falta-valor"));
+        match x.as_str() {
+            "--rutas" if r.rutas.is_none() => r.rutas = Some(valor()?),
+            "--tarea" if r.tarea.is_none() => {
+                let v = valor()?;
+                if !crate::git::es_id(&v) {
+                    return Err(uso("id-invalido"));
+                }
+                r.tarea = Some(v);
+            }
+            "--carril" if r.carril.is_none() => {
+                r.carril = Some(Carril::desde(&valor()?).ok_or_else(|| uso("carril-invalido"))?);
+            }
+            "--complejidad" if r.complejidad.is_none() => {
+                let v = valor()?;
+                if !crate::tarea::COMPLEJIDADES.contains(&v.as_str()) {
+                    return Err(uso("complejidad-invalida"));
+                }
+                r.complejidad = Some(v);
+            }
+            "--plugin" => {
+                let v = valor()?;
+                let Some((n, d)) = v.split_once('=').filter(|(n, d)| {
+                    !n.is_empty()
+                        && !d.is_empty()
+                        && n.chars()
+                            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
+                }) else {
+                    return Err(uso("plugin-invalido"));
+                };
+                r.plugins.push((n.to_owned(), PathBuf::from(d)));
+            }
+            "--informe" if r.informe.is_none() => r.informe = Some(PathBuf::from(valor()?)),
+            "--artefacto" if r.artefacto.is_none() => r.artefacto = Some(valor()?),
+            "--seccion" if r.seccion.is_none() => r.seccion = Some(PathBuf::from(valor()?)),
+            "--detalle" if r.detalle.is_none() => r.detalle = Some(PathBuf::from(valor()?)),
+            _ => return Err(uso("argumento-desconocido")),
+        }
+    }
+    if r.rutas.is_none() {
+        return Err(uso("sin-rutas"));
+    }
+    if r.informe.is_some() != r.seccion.is_some() {
+        return Err(uso("informe-sin-seccion"));
+    }
+    Ok(r)
+}
+
+/// `revisoras ...` (sin `--json`, que quita `lib.rs`).
+pub fn orden(a: &[String], e: &Entorno<'_>) -> Informe {
+    match paso(a, e) {
+        Ok(i) | Err(i) => i,
+    }
+}
+
+fn paso(a: &[String], e: &Entorno<'_>) -> Result<Informe, Informe> {
+    let a = args(a)?;
+    let fallo_lectura = |ruta: &str, error: String| {
+        Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(
+            Mensaje::nuevo(&txt::NO_SE_LEE)
+                .con("ruta", ruta)
+                .con("error", error),
+        )
+    };
+    let texto = match a.rutas.as_deref() {
+        Some("-") => (e.stdin)().map_err(|x| fallo_lectura("-", x.to_string()))?,
+        Some(f) => leer_con_tope(Path::new(f), TOPE_RUTAS)
+            .map_err(|x| fallo_lectura(f, x.to_string()))?
+            .ok_or_else(|| fallo_lectura(f, format!("mas de {TOPE_RUTAS} bytes")))?,
+        None => return Err(uso("sin-rutas")),
+    };
+    let rutas: Vec<String> = texto
+        .lines()
+        .map(|l| l.trim_end_matches('\r'))
+        .filter(|l| !l.trim().is_empty())
+        .map(str::to_owned)
+        .collect();
+    let f = fuentes(e.git, e.dir, e.claude, &a.plugins);
+    let mut avisos = f.avisos.clone();
+    let mut carril = Carril::Funcionalidad;
+    let mut cx: Option<String> = None;
+    if let Some(id) = &a.tarea {
+        let leida = crate::estado::leer(e.git, e.dir)
+            .map_err(|i| i.valor("motivo").unwrap_or("estado").to_owned())
+            .and_then(|v| {
+                crate::estado::tarea_de(&v, id).map_err(|f| match f {
+                    crate::estado::Fallo::Rechazada { motivo, .. } => motivo,
+                    crate::estado::Fallo::Error(_) => "error".to_owned(),
+                })
+            });
+        match leida {
+            Ok((t, c)) => {
+                let (k, x) = de_tarea(&t, &c);
+                carril = k;
+                cx = Some(x.to_owned());
+            }
+            Err(m) => avisos.push(format!("tarea-no-leida:{m}")),
+        }
+    }
+    if let Some(c) = a.carril {
+        carril = carril.max(c);
+    }
+    if a.complejidad.is_some() {
+        cx = a.complejidad.clone();
+    }
+    let config = crate::config::de_base(e.git, e.dir).unwrap_or_else(|_| {
+        avisos.push("config-de-base-no-leida".to_owned());
+        crate::config::Config::suelo()
+    });
+    let ctx = Contexto {
+        carril,
+        complejidad: cx.as_deref(),
+        modelos_permitidos: &config.seleccion.modelos_permitidos,
+        ronda: 1,
+    };
+    let mut s = super::revision(&rutas, &f.catalogo, &f.inventario, &ctx).map_err(|x| {
+        Informe::nuevo(NOMBRE, Resultado::Error)
+            .diagnostico(Mensaje::nuevo(&txt::TABLA_ILEGIBLE).con("error", x))
+    })?;
+    avisos.append(&mut s.avisos);
+    s.avisos = avisos;
+    let mut i = con_seleccion(Informe::nuevo(NOMBRE, Resultado::Ok), &s)
+        .con("carril", carril.nombre())
+        .con("complejidad", cx.as_deref().unwrap_or("desconocida"));
+    if let (Some(inf), Some(sec)) = (&a.informe, &a.seccion) {
+        let artefacto = a.artefacto.as_deref().unwrap_or("informe-verificacion");
+        let (estado, seccion) = informe_revisor(inf, artefacto);
+        if let Some(texto) = seccion {
+            std::fs::write(sec, texto)
+                .map_err(|x| fallo_lectura(&sec.display().to_string(), x.to_string()))?;
+        }
+        i = i.con("informe", estado);
+    }
+    if let Some(d) = &a.detalle {
+        std::fs::write(d, detalle(&s, &f.catalogo))
+            .map_err(|x| fallo_lectura(&d.display().to_string(), x.to_string()))?;
+    }
+    Ok(i)
+}
diff --git a/crates/tc2/src/seleccion/tabla.rs b/crates/tc2/src/seleccion/tabla.rs
new file mode 100644
index 0000000..fb76737
--- /dev/null
+++ b/crates/tc2/src/seleccion/tabla.rs
@@ -0,0 +1,312 @@
+//! Las tablas `tabla-seleccion` y `tabla-modelo` del ADR F1-15, leidas del
+//! propio ADR (decision 1: las reglas no se escriben aparte en el codigo).
+//!
+//! El ADR entra en el binario con `include_str!`: cambiar una fila del ADR
+//! cambia la seleccion al recompilar, y una condicion o una accion fuera del
+//! vocabulario cerrado de la decision 1 es un error de lectura (el test
+//! `las_tablas_del_adr_se_leen` lo comprueba en cada compilacion de tests).
+
+/// El ADR F1-15 entero.
+pub const ADR: &str =
+    include_str!("../../../../docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md");
+
+/// Escala de modelos, de menor a mayor (F1-15 decision 7).
+pub const ESCALA: [&str; 3] = ["haiku", "sonnet", "opus"];
+
+/// Carriles que puede nombrar un atomo `carril=`.
+const CARRILES: [&str; 4] = ["errata", "fix", "funcionalidad", "riesgo-alto"];
+
+/// Dominios cerrados (F1-15 decision 3).
+const DOMINIOS: [&str; 2] = ["puertas-seguridad", "rendimiento"];
+
+/// Atomos sin argumento del vocabulario cerrado (decision 1).
+const ATOMOS_FIJOS: [&str; 9] = [
+    "siempre",
+    "hay-skill-diseno-del-dominio",
+    "hay-skill-ejecucion-con-solape",
+    "rutas-casan-entre-1-y-umbral",
+    "rutas-no-casan-o-pasan-umbral",
+    "dominio-con-revisora",
+    "dominio-sin-revisora",
+    "hay-no-instalado",
+    "modelo-no-permitido",
+];
+
+/// Rol de una fila de `tabla-seleccion`.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum RolFila {
+    /// `diseno`.
+    Diseno,
+    /// `implementacion`.
+    Implementacion,
+    /// `revision`.
+    Revision,
+    /// `todos`.
+    Todos,
+}
+
+/// Un atomo de una condicion, con su negacion.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Atomo {
+    /// Si va con `no` delante.
+    pub negado: bool,
+    /// El atomo sin el `no` (`carril=fix`, `ronda>=2`, `siempre`...).
+    pub texto: String,
+}
+
+/// Una fila de `tabla-seleccion`.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct FilaSeleccion {
+    /// `S01`...
+    pub id: String,
+    /// Rol al que se aplica.
+    pub rol: RolFila,
+    /// Atomos unidos por «y».
+    pub condicion: Vec<Atomo>,
+    /// Token de la accion.
+    pub accion: String,
+    /// Ids del inventario de la fila (vacio con «—»).
+    pub ids: Vec<String>,
+}
+
+/// Una fila de `tabla-modelo`.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct FilaModelo {
+    /// `M01`...
+    pub id: String,
+    /// Carril.
+    pub carril: String,
+    /// Complejidades a las que se aplica; vacia es «cualquiera».
+    pub complejidades: Vec<String>,
+    /// Modelo de revision (columna «Revisión»).
+    pub revision: String,
+}
+
+/// Las filas entre `<!-- <marca>:inicio -->` y `<!-- <marca>:fin -->`, sin
+/// la cabecera ni el separador, como listas de celdas sin espacios ni
+/// comillas invertidas.
+fn filas(texto: &str, marca: &str) -> Result<Vec<Vec<String>>, String> {
+    let inicio = format!("<!-- {marca}:inicio -->");
+    let fin = format!("<!-- {marca}:fin -->");
+    let desde = texto
+        .find(&inicio)
+        .ok_or_else(|| format!("falta la marca {inicio}"))?;
+    let resto = texto.get(desde + inicio.len()..).unwrap_or_default();
+    let hasta = resto
+        .find(&fin)
+        .ok_or_else(|| format!("falta la marca {fin}"))?;
+    let cuerpo = resto.get(..hasta).unwrap_or_default();
+    let mut salida = Vec::new();
+    for l in cuerpo.lines().map(str::trim).filter(|l| l.starts_with('|')) {
+        let celdas: Vec<String> = l
+            .trim_matches('|')
+            .split('|')
+            .map(|c| c.trim().to_owned())
+            .collect();
+        let primera = celdas.first().map(String::as_str).unwrap_or_default();
+        if primera == "Id" || primera.starts_with("---") {
+            continue;
+        }
+        salida.push(celdas);
+    }
+    if salida.is_empty() {
+        return Err(format!("{marca} sin filas"));
+    }
+    Ok(salida)
+}
+
+fn sin_comillas(c: &str) -> String {
+    c.replace('`', "").trim().to_owned()
+}
+
+/// `n` de un atomo `<prefijo>n` con n entero.
+fn numero(texto: &str, prefijo: &str) -> Option<u32> {
+    texto.strip_prefix(prefijo).and_then(|n| n.parse().ok())
+}
+
+fn atomo_valido(a: &str) -> bool {
+    ATOMOS_FIJOS.contains(&a)
+        || a.strip_prefix("carril=")
+            .or_else(|| a.strip_prefix("carril!="))
+            .is_some_and(|c| CARRILES.contains(&c))
+        || a.strip_prefix("dominio=")
+            .is_some_and(|d| DOMINIOS.contains(&d))
+        || numero(a, "roles>=").is_some()
+        || numero(a, "ronda>=").is_some()
+}
+
+/// Accion valida y, si necesita candidatos, el atomo que los garantiza.
+fn accion_valida(accion: &str) -> Result<Option<&'static str>, String> {
+    let fija = match accion {
+        "sin-plan"
+        | "roles=max(roles_min,complejidad)"
+        | "lanzador"
+        | "generica-del-plugin"
+        | "quitar-no-instalado"
+        | "restringir-modelo"
+        | "aviso=dominio-sin-revisora" => Some(None),
+        "skills-diseno-del-dominio" => Some(Some("hay-skill-diseno-del-dominio")),
+        "skill-ejecucion=solape,prioridad,id" => Some(Some("hay-skill-ejecucion-con-solape")),
+        "revisora-del-dominio" => Some(Some("dominio-con-revisora")),
+        "revisoras-de-rutas+generica-si-sueltas" => Some(Some("rutas-casan-entre-1-y-umbral")),
+        _ => None,
+    };
+    if let Some(f) = fija {
+        return Ok(f);
+    }
+    if numero(accion, "suelo=").is_some() || numero(accion, "modelo-baja=").is_some() {
+        return Ok(None);
+    }
+    if accion
+        .strip_prefix("mas-rol=")
+        .is_some_and(|r| ["rendimiento", "unificador"].contains(&r))
+    {
+        return Ok(None);
+    }
+    Err(format!("accion fuera del vocabulario: {accion}"))
+}
+
+/// Lee `tabla-seleccion` de `texto` y valida su vocabulario.
+pub fn seleccion(texto: &str) -> Result<Vec<FilaSeleccion>, String> {
+    let mut v = Vec::new();
+    for c in filas(texto, "tabla-seleccion")? {
+        let [id, rol, condicion, accion, ids, ..] = c.as_slice() else {
+            return Err(format!("fila de tabla-seleccion incompleta: {c:?}"));
+        };
+        let rol = match sin_comillas(rol).as_str() {
+            "diseno" => RolFila::Diseno,
+            "implementacion" => RolFila::Implementacion,
+            "revision" => RolFila::Revision,
+            "todos" => RolFila::Todos,
+            otro => return Err(format!("{id}: rol desconocido {otro}")),
+        };
+        let mut atomos = Vec::new();
+        for parte in sin_comillas(condicion).split(" y ") {
+            let parte = parte.trim();
+            let (negado, texto) = match parte.strip_prefix("no ") {
+                Some(r) => (true, r.trim()),
+                None => (false, parte),
+            };
+            if !atomo_valido(texto) {
+                return Err(format!("{id}: condicion fuera del vocabulario: {texto}"));
+            }
+            atomos.push(Atomo {
+                negado,
+                texto: texto.to_owned(),
+            });
+        }
+        let accion = sin_comillas(accion);
+        if let Some(exige) = accion_valida(&accion).map_err(|e| format!("{id}: {e}"))?
+            && !atomos.iter().any(|a| !a.negado && a.texto == exige)
+        {
+            return Err(format!("{id}: la accion {accion} exige el atomo {exige}"));
+        }
+        let ids = sin_comillas(ids);
+        let ids = if ids == "—" || ids.is_empty() {
+            Vec::new()
+        } else {
+            ids.split(',').map(|s| s.trim().to_owned()).collect()
+        };
+        if v.iter().any(|f: &FilaSeleccion| f.id == *id) {
+            return Err(format!("{id} repetida"));
+        }
+        v.push(FilaSeleccion {
+            id: id.clone(),
+            rol,
+            condicion: atomos,
+            accion,
+            ids,
+        });
+    }
+    Ok(v)
+}
+
+/// Lee `tabla-modelo` de `texto`.
+pub fn modelo(texto: &str) -> Result<Vec<FilaModelo>, String> {
+    let mut v = Vec::new();
+    for c in filas(texto, "tabla-modelo")? {
+        let [id, carril, cx, _diseno, _impl, revision, ..] = c.as_slice() else {
+            return Err(format!("fila de tabla-modelo incompleta: {c:?}"));
+        };
+        let carril = sin_comillas(carril);
+        if !CARRILES.contains(&carril.as_str()) {
+            return Err(format!("{id}: carril desconocido {carril}"));
+        }
+        let cx = sin_comillas(cx);
+        let complejidades = if cx == "cualquiera" {
+            Vec::new()
+        } else {
+            cx.split(',').map(|s| s.trim().to_owned()).collect()
+        };
+        for x in &complejidades {
+            if !crate::tarea::COMPLEJIDADES.contains(&x.as_str()) {
+                return Err(format!("{id}: complejidad desconocida {x}"));
+            }
+        }
+        let revision = sin_comillas(revision);
+        if !ESCALA.contains(&revision.as_str()) {
+            return Err(format!("{id}: modelo fuera de la escala {revision}"));
+        }
+        v.push(FilaModelo {
+            id: id.clone(),
+            carril,
+            complejidades,
+            revision,
+        });
+    }
+    // Cada combinacion de carril y complejidad, exactamente una fila.
+    for carril in CARRILES {
+        for cx in crate::tarea::COMPLEJIDADES {
+            let n = v
+                .iter()
+                .filter(|f| {
+                    f.carril == carril
+                        && (f.complejidades.is_empty() || f.complejidades.iter().any(|x| x == cx))
+                })
+                .count();
+            if n != 1 {
+                return Err(format!("tabla-modelo: {carril}/{cx} tiene {n} filas"));
+            }
+        }
+    }
+    Ok(v)
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    /// Las dos tablas del ADR real se leen con el vocabulario cerrado.
+    #[test]
+    fn las_tablas_del_adr_se_leen() {
+        let s = seleccion(ADR).unwrap_or_default();
+        assert_eq!(s.len(), 15, "{:?}", seleccion(ADR).err());
+        let s08 = s.iter().find(|f| f.id == "S08");
+        assert_eq!(
+            s08.map(|f| (f.rol, f.accion.as_str())),
+            Some((RolFila::Revision, "revisoras-de-rutas+generica-si-sueltas"))
+        );
+        let s13 = s
+            .iter()
+            .find(|f| f.id == "S13")
+            .map(|f| f.condicion.clone());
+        assert_eq!(s13.map(|c| c.len()), Some(3));
+        let m = modelo(ADR).unwrap_or_default();
+        assert_eq!(m.len(), 7, "{:?}", modelo(ADR).err());
+    }
+
+    /// Lo que no es del vocabulario no se lee.
+    #[test]
+    fn fuera_del_vocabulario_es_error() {
+        let base = "<!-- tabla-seleccion:inicio -->\n| Id | Rol | Condición | Acción | Ids | Qué |\n|---|---|---|---|---|---|\n| S01 | `revision` | COND | ACC | — | x |\n<!-- tabla-seleccion:fin -->\n";
+        let con = |c: &str, a: &str| seleccion(&base.replace("COND", c).replace("ACC", a));
+        assert!(con("`siempre`", "`lanzador`").is_ok());
+        assert!(con("`a-ojo`", "`lanzador`").is_err());
+        assert!(con("`siempre`", "`preguntar-al-modelo`").is_err());
+        assert!(con("`carril=otro`", "`lanzador`").is_err());
+        // Una accion con candidatos exige su atomo.
+        assert!(con("`siempre`", "`revisora-del-dominio`").is_err());
+        assert!(con("`dominio-con-revisora`", "`revisora-del-dominio`").is_ok());
+        assert!(con("`siempre`", "`revisoras-de-rutas+generica-si-sueltas`").is_err());
+    }
+}
diff --git a/xtask/src/main.rs b/xtask/src/main.rs
index 0f8e7fc..dbc8c10 100644
--- a/xtask/src/main.rs
+++ b/xtask/src/main.rs
@@ -532,7 +532,8 @@ fn comparar_hashes(hashes: &[String]) -> Result<(), String> {
 }
 
 /// Ficheros versionados o nuevos no ignorados, como los ve git, salvo los que
-/// no entran en el binario.
+/// no entran en el binario. `docs/adr/` si entra: `tc2` incrusta las tablas de
+/// seleccion del ADR F1-15 con `include_str!` (TASK-049).
 fn ficheros_del_arbol(raiz: &Path) -> Result<Vec<PathBuf>, String> {
     let salida = Command::new("git")
         .args([
@@ -552,7 +553,11 @@ fn ficheros_del_arbol(raiz: &Path) -> Result<Vec<PathBuf>, String> {
         .map_err(|_| "git ls-files no devolvio UTF-8".to_owned())?;
     Ok(texto
         .split('\0')
-        .filter(|s| !s.is_empty() && !s.starts_with("tareas/") && !s.starts_with("docs/"))
+        .filter(|s| {
+            !s.is_empty()
+                && !s.starts_with("tareas/")
+                && (!s.starts_with("docs/") || s.starts_with("docs/adr/"))
+        })
         .map(PathBuf::from)
         .collect())
 }
diff --git a/xtask/src/workflows.rs b/xtask/src/workflows.rs
index 212adc4..0ee5892 100644
--- a/xtask/src/workflows.rs
+++ b/xtask/src/workflows.rs
@@ -18,6 +18,15 @@
 //!   `github.event.*`, `github.head_ref` o `toJSON(github...)` en `run:`
 //!   (los datos del evento se pasan por `env:`).
 //!
+//! El workflow del revisor IA (`revision-ia.yml`, F6-3) pasa el mismo lint
+//! con su perfil ([`REVISION`]): solo `workflow_call`; un job con
+//! `environment:` es de clase privilegio aunque su GITHUB_TOKEN no escriba
+//! (sus secretos estan ahi), y el unico entorno admitido es `revisor`;
+//! ningun job escribe con GITHUB_TOKEN (el check lo publica la App del
+//! revisor); y nada saca el head del PR: cada checkout lleva `ref:` sin
+//! `head`, y `run:` no lleva `git checkout`, `switch`, `worktree`, `restore`,
+//! `reset` ni `inputs.`.
+//!
 //! En la plantilla de GitLab se aplica lo que le corresponde: ningun
 //! `secrets:` y ninguna variable que escribe el autor del MR o del commit en
 //! `script:`. Ademas, las entradas de la plantilla son las del workflow.
@@ -405,7 +414,53 @@ fn es_sha(texto: &str) -> bool {
     texto.len() == 40 && texto.chars().all(|c| c.is_ascii_hexdigit())
 }
 
+/// Lo que cambia entre los workflows de la lista blanca: el de verificacion
+/// (F5-1) y el del revisor IA (F6-3).
+struct Perfil {
+    /// Eventos admitidos.
+    eventos: &'static [&'static str],
+    /// Entornos de despliegue admitidos (`environment:` de un job). Un job
+    /// con entorno es de clase privilegio aunque su GITHUB_TOKEN no escriba:
+    /// sus secretos estan ahi (F6-3).
+    entornos: &'static [&'static str],
+    /// Ningun job escribe con GITHUB_TOKEN (lo que se publica va con una
+    /// App).
+    sin_escritura: bool,
+    /// Ningun checkout del head del PR (ni `ref:` con `head`, ni `git
+    /// checkout`, `switch`, `worktree`, `restore` o `reset` en `run:`), y
+    /// ninguna `inputs.` en `run:` (la rama del PR llega por una entrada).
+    sin_head: bool,
+}
+
+/// `verificacion.yml` (F5-1, F5-2).
+const VERIFICACION: Perfil = Perfil {
+    eventos: &EVENTOS_ADMITIDOS,
+    entornos: &[],
+    sin_escritura: false,
+    sin_head: false,
+};
+
+/// `revision-ia.yml` (F6-3): solo reutilizable, secretos solo en el job del
+/// entorno `revisor`, sin escritura del GITHUB_TOKEN y sin el head.
+const REVISION: Perfil = Perfil {
+    eventos: &["workflow_call"],
+    entornos: &["revisor"],
+    sin_escritura: true,
+    sin_head: true,
+};
+
+/// Ordenes de git que en `run:` cambiarian el arbol de trabajo a otro commit.
+const CAMBIAN_EL_ARBOL: [&str; 5] = ["checkout", "switch", "worktree", "restore", "reset"];
+
 fn lint_github(texto: &str) -> Vec<String> {
+    lint_con(texto, &VERIFICACION)
+}
+
+fn lint_revision(texto: &str) -> Vec<String> {
+    lint_con(texto, &REVISION)
+}
+
+fn lint_con(texto: &str, perfil: &Perfil) -> Vec<String> {
     let raiz = match leer_yaml(texto) {
         Ok(r) => r,
         Err(e) => return vec![e],
@@ -415,7 +470,7 @@ fn lint_github(texto: &str) -> Vec<String> {
         None => errores.push("sin bloque on".to_owned()),
         Some(on) => {
             for e in eventos(on) {
-                if !EVENTOS_ADMITIDOS.contains(&e.as_str()) {
+                if !perfil.eventos.contains(&e.as_str()) {
                     errores.push(format!("evento no admitido: {e}"));
                 }
             }
@@ -461,13 +516,24 @@ fn lint_github(texto: &str) -> Vec<String> {
         let Some(job) = jobs.get(nombre) else {
             continue;
         };
-        let privilegio = match escribe(job.get("permissions")) {
+        let escritura = match escribe(job.get("permissions")) {
             Ok(p) => p,
             Err(e) => {
                 errores.push(format!("job {nombre}: {e}"));
                 false
             }
         };
+        if escritura && perfil.sin_escritura {
+            errores.push(format!("job {nombre}: escritura con GITHUB_TOKEN"));
+        }
+        let entorno = job.get("environment");
+        if let Some(n) = entorno {
+            let valor = n.texto().or_else(|| n.get("name").and_then(Nodo::texto));
+            if !valor.is_some_and(|v| perfil.entornos.contains(&v)) {
+                errores.push(format!("job {nombre}: entorno no admitido: {valor:?}"));
+            }
+        }
+        let privilegio = escritura || entorno.is_some();
         if let Some(Nodo::Mapa(pares)) = job.get("permissions") {
             for (alcance, valor) in pares {
                 if valor.texto() == Some("read") && !LECTURAS_ADMITIDAS.contains(&alcance.as_str())
@@ -501,6 +567,24 @@ fn lint_github(texto: &str) -> Vec<String> {
                         errores.push(format!("job {nombre}: {dato} en run"));
                     }
                 }
+                if perfil.sin_head {
+                    if normalizado.contains("inputs.") {
+                        errores.push(format!("job {nombre}: inputs. en run"));
+                    }
+                    // `git [-C dir] checkout ...`: la orden de git en
+                    // cualquier posicion de una linea que llama a git.
+                    for linea in valor.lines() {
+                        let t: Vec<&str> = linea.split_whitespace().collect();
+                        if !t.contains(&"git") {
+                            continue;
+                        }
+                        for orden in CAMBIAN_EL_ARBOL {
+                            if t.contains(&orden) {
+                                errores.push(format!("job {nombre}: git {orden} en run"));
+                            }
+                        }
+                    }
+                }
             }
         }
         let pasos = match job.get("steps") {
@@ -542,6 +626,17 @@ fn lint_github(texto: &str) -> Vec<String> {
                     "job {nombre}: checkout con ref o repository en clase privilegio"
                 ));
             }
+            // F6-3: la base y el plugin se sacan; el head del PR, nunca (sin
+            // `ref:` en un evento de PR sale el merge ref, codigo del PR).
+            if perfil.sin_head {
+                match opcion("ref").and_then(Nodo::texto) {
+                    Some(r) if normalizar(r).contains("head") => {
+                        errores.push(format!("job {nombre}: checkout del head del PR"));
+                    }
+                    Some(_) => {}
+                    None => errores.push(format!("job {nombre}: checkout sin ref fija")),
+                }
+            }
         }
     }
     errores
@@ -1246,3 +1341,117 @@ fn las_reglas_de_gitlab_no_leen_variables_de_job() {
         "{errores:?}"
     );
 }
+
+// --- Revisor IA (F6-3) -----------------------------------------------------------
+
+const REVISION_IA: &str = ".github/workflows/revision-ia.yml";
+
+/// El workflow del revisor pasa el lint con el perfil revision-ia: dos
+/// jobs, el publicador en el entorno `revisor`, sin checkout, con los tres
+/// secretos y sin escritura del GITHUB_TOKEN.
+#[test]
+fn el_workflow_del_revisor_pasa_el_lint() {
+    let texto = leer(REVISION_IA);
+    let raiz = yaml(&texto);
+    let jobs = raiz.get("jobs").map(Nodo::claves).unwrap_or_default();
+    assert_eq!(jobs, ["preparar", "publicar"]);
+    assert_eq!(lint_revision(&texto), Vec::<String>::new());
+    let publicar = raiz.get("jobs").and_then(|j| j.get("publicar"));
+    assert_eq!(
+        publicar
+            .and_then(|p| p.get("environment"))
+            .and_then(Nodo::texto),
+        Some("revisor")
+    );
+    let mut valores = Vec::new();
+    if let Some(p) = publicar {
+        p.escalares("", &mut valores);
+    }
+    for s in [
+        "TC2_APP_REVISOR_ID",
+        "TC2_APP_REVISOR_CLAVE",
+        "ANTHROPIC_API_KEY",
+    ] {
+        let buscado = format!("secrets.{}", s.to_ascii_lowercase());
+        let en_publicar = valores
+            .iter()
+            .any(|(_, v)| normalizar(v).contains(&buscado));
+        assert!(en_publicar, "falta el secreto {s} en publicar");
+    }
+    // Con el perfil de verificacion, el entorno y el evento no valen.
+    assert!(!lint_github(&texto).is_empty());
+}
+
+#[test]
+fn cada_mutacion_del_revisor_da_su_error() {
+    let texto = leer(REVISION_IA);
+    let casos: [(&str, &str, &str); 9] = [
+        (
+            "  workflow_call:\n",
+            "  pull_request:\n  workflow_call:\n",
+            "evento no admitido: pull_request",
+        ),
+        (
+            "          ref: ${{ inputs.base }}\n",
+            "          ref: ${{ inputs.head-sha }}\n",
+            "checkout del head del PR",
+        ),
+        (
+            "          ref: ${{ inputs.base }}\n",
+            "",
+            "checkout sin ref fija",
+        ),
+        (
+            "          git -C base fetch --no-tags -q origin \"$HEAD_SHA\"\n",
+            "          git -C base fetch --no-tags -q origin \"$HEAD_SHA\"\n          git -C base checkout \"$HEAD_SHA\"\n",
+            "git checkout en run",
+        ),
+        (
+            "    environment: revisor\n",
+            "",
+            "job publicar: secreto en clase ejecuta",
+        ),
+        (
+            "    environment: revisor\n",
+            "    environment: produccion\n",
+            "entorno no admitido",
+        ),
+        (
+            "    permissions:\n      actions: read\n    steps:\n      - name: Revisar",
+            "    permissions:\n      actions: read\n      checks: write\n    steps:\n      - name: Revisar",
+            "job publicar: escritura con GITHUB_TOKEN",
+        ),
+        (
+            "          FORZADA: ${{ inputs.tc2-sha }}\n",
+            "          FORZADA: ${{ secrets.ANTHROPIC_API_KEY }}\n",
+            "job preparar: secreto en clase ejecuta",
+        ),
+        (
+            "          set -eu\n          rustup toolchain install\n",
+            "          set -eu\n          echo ${{ inputs.rama }}\n          rustup toolchain install\n",
+            "inputs. en run",
+        ),
+    ];
+    for (original, mutado, esperado) in casos {
+        assert!(texto.contains(original), "el caso no casa: {original}");
+        let errores = lint_revision(&texto.replacen(original, mutado, 1));
+        assert!(
+            errores.iter().any(|e| e.contains(esperado)),
+            "«{esperado}» no salio; errores: {errores:?}"
+        );
+    }
+    // Un checkout en el publicador: privilegio por el entorno.
+    let con_checkout = texto.replacen(
+        "    steps:\n      - name: Revisar",
+        "    steps:\n      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1\n        with:\n          persist-credentials: false\n          ref: develop\n      - name: Revisar",
+        1,
+    );
+    assert_ne!(con_checkout, texto);
+    let errores = lint_revision(&con_checkout);
+    assert!(
+        errores
+            .iter()
+            .any(|e| e.contains("job publicar: checkout en clase privilegio")),
+        "{errores:?}"
+    );
+}
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
Cargo.lock                                                                                | 49 +++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{01-en-diseno => 02-en-curso}/TASK-049/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-049/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-049/tarea.md                                    |  3 ++-
 4 files changed, 51 insertions(+), 1 deletion(-)
````

## Tests del diff (2 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/revision.rs" ":(literal)crates/tc2/tests/revisoras.rs"
````

````
crates/tc2/tests/revision.rs  |  16 ++++++++++++++--
 crates/tc2/tests/revisoras.rs | 171 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 2 files changed, 185 insertions(+), 2 deletions(-)
````
