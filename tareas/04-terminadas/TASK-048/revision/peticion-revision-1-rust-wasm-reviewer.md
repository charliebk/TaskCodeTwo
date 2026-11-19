# Peticion de revision — TASK-048 (ronda 1)

- Tarea: TASK-048 — F6-2 review: abre el PR contra develop con la identidad del agente
- Rama revisada: feature/task-048-f6-2-review-abre-el-pr-contra-develop-co
- Rama base: develop
- Commit revisado (HEAD): a30688c63e099c28376a9d6dcd3f9ffd50f0a95d
- Fecha: 2026-11-19
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-048 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-048/tarea.md
- tareas/03-en-revision/TASK-048/planificacion/
- crates/tc2/src/git/puerto.rs
- crates/tc2/src/i18n/catalogo/cli.rs
- crates/tc2/src/i18n/catalogo/mod.rs
- crates/tc2/src/i18n/catalogo/plataforma.rs
- crates/tc2/src/i18n/catalogo/revision.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/plataforma/entorno_git.rs
- crates/tc2/src/plataforma/github.rs
- crates/tc2/src/plataforma/gitlab.rs
- crates/tc2/src/plataforma/mod.rs
- crates/tc2/src/revision/mod.rs
- crates/tc2/src/revision/operacion.rs
- crates/tc2/tests/agente_git/mod.rs
- crates/tc2/tests/identidades.rs
- crates/tc2/tests/plataforma_e2e.rs
- crates/tc2/tests/plataforma_gitlab_e2e.rs
- crates/tc2/tests/revision.rs

## Commits a revisar (git log develop..HEAD)

````
a30688c test(TASK-048): GitLab guarda la descripcion del MR sin el salto final
87fe06f feat(TASK-048): tc2 review abre el PR contra develop con la identidad del agente
1710a51 chore(TASK-048): tarea en curso
````

## Diff de tu dominio (17 fichero(s) de 18; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/git/puerto.rs b/crates/tc2/src/git/puerto.rs
index 424211a..28067aa 100644
--- a/crates/tc2/src/git/puerto.rs
+++ b/crates/tc2/src/git/puerto.rs
@@ -274,6 +274,21 @@ impl Git {
         self
     }
 
+    /// Un puerto nuevo con el binario y el entorno de este mas `vars` (como
+    /// [`Git::con_entorno_extra`]) y el registro vacio: el entorno del agente
+    /// para una sola peticion (el `push` de `review`, F6-2) sin tocar el
+    /// puerto con el que corre la transaccion.
+    #[must_use]
+    pub fn derivado(&self, vars: Vec<(OsString, OsString)>) -> Self {
+        Self {
+            binario: self.binario.clone(),
+            entorno_extra: self.entorno_extra.clone(),
+            config: self.config.clone(),
+            registro: RefCell::new(Vec::new()),
+        }
+        .con_entorno_extra(vars)
+    }
+
     /// Copia del registro: una linea `git <subcomando> <args>` por llamada
     /// ejecutada.
     pub fn registro(&self) -> Vec<String> {
diff --git a/crates/tc2/src/i18n/catalogo/cli.rs b/crates/tc2/src/i18n/catalogo/cli.rs
index fd27afc..28636a6 100644
--- a/crates/tc2/src/i18n/catalogo/cli.rs
+++ b/crates/tc2/src/i18n/catalogo/cli.rs
@@ -10,8 +10,8 @@ crate::i18n::textos! {
         en: "tc2: could not write the output: {error}",
     }
     USO {
-        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<motivo>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <carril>] | tc2 plan guardar [--json] TASK-NNN <destino> <fichero> | tc2 approve [--json] TASK-NNN [--pedir-cambios <fichero>] | tc2 approve [--json] --hito <n> | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
-        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<reason>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <lane>] | tc2 plan guardar [--json] TASK-NNN <destination> <file> | tc2 approve [--json] TASK-NNN [--pedir-cambios <file>] | tc2 approve [--json] --hito <n> | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
+        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<motivo>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <carril>] | tc2 plan guardar [--json] TASK-NNN <destino> <fichero> | tc2 approve [--json] TASK-NNN [--pedir-cambios <fichero>] | tc2 approve [--json] --hito <n> | tc2 review [--json] TASK-NNN | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
+        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<reason>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <lane>] | tc2 plan guardar [--json] TASK-NNN <destination> <file> | tc2 approve [--json] TASK-NNN [--pedir-cambios <file>] | tc2 approve [--json] --hito <n> | tc2 review [--json] TASK-NNN | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
     }
     FALTA_LA_ORDEN {
         es: "tc2: falta la orden. {uso}",
diff --git a/crates/tc2/src/i18n/catalogo/mod.rs b/crates/tc2/src/i18n/catalogo/mod.rs
index 528ac2d..b6aeea5 100644
--- a/crates/tc2/src/i18n/catalogo/mod.rs
+++ b/crates/tc2/src/i18n/catalogo/mod.rs
@@ -17,6 +17,7 @@ pub mod init;
 pub mod plan;
 pub mod plataforma;
 pub mod progreso;
+pub mod revision;
 pub mod roadmap;
 pub mod siguiente;
 pub mod tablero;
@@ -43,6 +44,7 @@ pub fn todos() -> impl Iterator<Item = &'static Texto> {
         plan::TODOS,
         plataforma::TODOS,
         progreso::TODOS,
+        revision::TODOS,
         roadmap::TODOS,
         siguiente::TODOS,
         tablero::TODOS,
diff --git a/crates/tc2/src/i18n/catalogo/plataforma.rs b/crates/tc2/src/i18n/catalogo/plataforma.rs
index b839002..c55c71d 100644
--- a/crates/tc2/src/i18n/catalogo/plataforma.rs
+++ b/crates/tc2/src/i18n/catalogo/plataforma.rs
@@ -303,6 +303,16 @@ crate::i18n::textos! {
         en: "character not allowed in the query",
     }
 
+    // --- entorno-push (F6-2) ---
+    REMOTO_NO_HTTPS {
+        es: "el remoto origin no es una URL https://: el agente solo empuja con su token por https (por ssh empujaria tu identidad y por http el token iria en claro). Cambialo con git remote set-url origin https://... y repite",
+        en: "the origin remote is not an https:// URL: the agent only pushes with its token over https (over ssh it would push your identity and over http the token would travel in clear). Change it with git remote set-url origin https://... and try again",
+    }
+    ENTORNO_PUSH_NO_SOPORTADO {
+        es: "esta plataforma no da el entorno para que el agente empuje con su identidad",
+        en: "this platform does not provide the environment for the agent to push with its identity",
+    }
+
     // --- aplicar-puertas (TASK-118) ---
     APLICAR_NO_SOPORTADA {
         es: "esta plataforma no admite aplicar las puertas desde tc2: aplicalas a mano con la propuesta de tc2 init --json",
diff --git a/crates/tc2/src/i18n/catalogo/revision.rs b/crates/tc2/src/i18n/catalogo/revision.rs
new file mode 100644
index 0000000..38af9bd
--- /dev/null
+++ b/crates/tc2/src/i18n/catalogo/revision.rs
@@ -0,0 +1,28 @@
+//! Mensajes de `tc2 review` (F6-2).
+
+crate::i18n::textos! {
+    USO_REVIEW {
+        es: "uso: review [--json] TASK-NNN",
+        en: "usage: review [--json] TASK-NNN",
+    }
+    DE_OTRA_PERSONA {
+        es: "la tarea {id} la lleva {login}: solo quien la lleva la pasa a revision",
+        en: "task {id} is taken by {login}: only whoever has it sends it to review",
+    }
+    SIN_RAMA {
+        es: "la tarea {id} no tiene rama: empiezala con tc2 start {id}",
+        en: "task {id} has no branch: start it with tc2 start {id}",
+    }
+    CARRIL_NO_SOPORTADO {
+        es: "la rama {rama} de la tarea {id} no va contra develop: review solo abre PR contra develop por ahora (los hotfix llegan con F7)",
+        en: "branch {rama} of task {id} does not target develop: review only opens PRs against develop for now (hotfixes come with F7)",
+    }
+    SIN_WORKTREE {
+        es: "la tarea {id} no tiene su worktree en {ruta} con la rama {rama}: lanza tc2 start {id} para recrearlo y repite",
+        en: "task {id} has no worktree at {ruta} on branch {rama}: run tc2 start {id} to recreate it and try again",
+    }
+    SIN_ORIGIN {
+        es: "el repo no tiene remoto origin: review empuja la rama y abre el PR en la plataforma de origin",
+        en: "the repo has no origin remote: review pushes the branch and opens the PR on the origin platform",
+    }
+}
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 4675f64..8ded35b 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -20,7 +20,7 @@
 //! (aviso de solape de `start`, F3-11) y [`ciclo`]
 //! (`descartar`, `pausa` y `reanudar`, F3-7; `liberar` y `reasignar`,
 //! F3-10) y [`doctor`] (F3-8) y [`plan`] (`plan` y `plan guardar`, F4-1;
-//! `approve`, F4-2). Las capas
+//! `approve`, F4-2) y [`revision`] (`review`, F6-2). Las capas
 //! que vienen entran como modulos de esta biblioteca.
 
 pub mod actualizar_cli;
@@ -47,6 +47,7 @@ pub mod plan;
 pub mod plataforma;
 pub mod progreso;
 pub mod recursos;
+pub mod revision;
 pub mod roadmap;
 pub mod siguiente;
 pub mod situacion;
@@ -210,6 +211,12 @@ where
                 plan::orden_approve(&resto, e)
             })
         }
+        [orden, resto @ ..] if orden == revision::NOMBRE => {
+            let (json, resto) = con_json(resto);
+            de_una_tarea(revision::NOMBRE, (json, idioma), |e| {
+                orden_review(&resto, e)
+            })
+        }
         [orden, resto @ ..] if orden == estimacion::NOMBRE => {
             let (json, resto) = con_json(resto);
             orden_estimar(&resto, json, idioma)
@@ -493,6 +500,26 @@ fn transicion(
     orden(&e)
 }
 
+/// `review [--json] TASK-NNN` (F6-2): el entorno de `start` mas el
+/// adaptador de la plataforma de `origin` (la deteccion de `start`, la lista
+/// de personas de la base y los tokens o la App del agente del proceso);
+/// `review` le pide siempre con el rol de agente.
+fn orden_review(args: &[String], e: &toma::Entorno<'_>) -> Informe {
+    let conectar = || -> plataforma::Paso<Box<dyn plataforma::Plataforma>> {
+        let destino = identidad::destino_de(e.git, e.dir)?;
+        let personas = identidad::personas_de_base(e.git, e.dir)?;
+        identidad::adaptador(destino, plataforma::de_entorno(), personas)
+    };
+    revision::orden(
+        args,
+        &revision::Entorno {
+            base: e,
+            conectar: &conectar,
+            esperas: &git::ESPERAS_PUSH,
+        },
+    )
+}
+
 /// `estimar [--json] [TASK-NNN | --error TASK-NNN]` (F3-13) en la carpeta
 /// actual. Sin `--json` y con exito, la linea para la persona va detras de
 /// la de contrato (como mucho 5 lineas, F1-4 decision 4).
@@ -575,6 +602,7 @@ fn catalogo(anotacion: &str) -> Option<Box<dyn estado::Operacion>> {
         .or_else(|| roadmap::desde_anotacion(anotacion))
         .or_else(|| toma::desde_anotacion(anotacion))
         .or_else(|| ciclo::desde_anotacion(anotacion))
+        .or_else(|| revision::desde_anotacion(anotacion))
 }
 
 /// `new "<titulo>" [campos de RF-01]`: alta de una tarea con su ID reservado
diff --git a/crates/tc2/src/plataforma/entorno_git.rs b/crates/tc2/src/plataforma/entorno_git.rs
index f9974de..171a59a 100644
--- a/crates/tc2/src/plataforma/entorno_git.rs
+++ b/crates/tc2/src/plataforma/entorno_git.rs
@@ -6,21 +6,29 @@
 //! `GIT_CONFIG_COUNT`, `GIT_CONFIG_KEY_<n>` y `GIT_CONFIG_VALUE_<n>` (config
 //! de ambito «command», Git >= 2.31) se define
 //! `http.<url>.extraheader = Authorization: Basic <base64(usuario:token)>`,
-//! limitado a la URL del remoto, y el autor y committer con
+//! limitado a la URL del remoto, y, si se da, el autor y committer con
 //! `GIT_AUTHOR_*`/`GIT_COMMITTER_*`. Ni el registro del puerto `Git` ni los
 //! argumentos de los hijos de git (`git-remote-https`) lo ven.
 //!
-//! Quien llama compone el puerto con [`crate::git::Git::con_entorno_extra`]:
-//! la capa `git` no depende de `plataforma`. Conectarlo a `review` es de
-//! F6-2.
+//! Quien llama compone el puerto con [`crate::git::Git::con_entorno_extra`]
+//! o [`crate::git::Git::derivado`]: la capa `git` no depende de
+//! `plataforma`. `review` (F6-2) lo pide al adaptador con
+//! [`super::Plataforma::entorno_push`], que lo arma con [`entorno_push`].
 
 use std::ffi::OsString;
 
 use base64::Engine as _;
 use base64::engine::general_purpose::STANDARD;
 
+use super::Paso;
 use super::Proveedor;
-use super::credenciales::Secreto;
+use super::conexion::Conexion;
+use super::credenciales::{Rol, Secreto};
+use crate::contrato::{Informe, Resultado};
+use crate::i18n::catalogo::plataforma as t;
+
+/// Nombre de la operacion del puerto (F6-2).
+pub const ENTORNO_PUSH: &str = "entorno-push";
 
 /// Autor (y committer) de los commits del agente.
 #[derive(Debug, Clone, PartialEq, Eq)]
@@ -60,28 +68,124 @@ fn usuario(p: Proveedor) -> &'static str {
 
 /// Variables para que `git` hable con `url_remoto` (p. ej.
 /// `https://github.com/`; la cabecera solo va a las URL que empiezan asi)
-/// con `token` y firme como `autor`. La cabecera es un lote de una entrada
-/// de `GIT_CONFIG_*` numerado desde 0: [`crate::git::Git::con_entorno_extra`]
-/// lo renumera tras las entradas que ya tengan el puerto y el proceso, sin
-/// pisarlas. Pasadas a otro `Command` a mano, ocupan la entrada 0.
+/// con `token` y, si se da, firme como `autor` (`review` solo empuja: sin
+/// autor, los commits siguen con la identidad local). La cabecera es un
+/// lote de una entrada de `GIT_CONFIG_*` numerado desde 0:
+/// [`crate::git::Git::con_entorno_extra`] lo renumera tras las entradas que
+/// ya tengan el puerto y el proceso, sin pisarlas. Pasadas a otro `Command`
+/// a mano, ocupan la entrada 0.
 pub fn entorno_git(
     proveedor: Proveedor,
     url_remoto: &str,
     token: &Secreto,
-    autor: &AutorGit,
+    autor: Option<&AutorGit>,
 ) -> Vec<(OsString, OsString)> {
     let basica = STANDARD.encode(format!("{}:{}", usuario(proveedor), token.exponer()));
     let par = |k: &str, v: String| (OsString::from(k), OsString::from(v));
-    vec![
+    let mut v = vec![
         par("GIT_CONFIG_COUNT", "1".to_owned()),
         par("GIT_CONFIG_KEY_0", format!("http.{url_remoto}.extraheader")),
         par(
             "GIT_CONFIG_VALUE_0",
             format!("Authorization: Basic {basica}"),
         ),
-        par("GIT_AUTHOR_NAME", autor.nombre.clone()),
-        par("GIT_AUTHOR_EMAIL", autor.correo.clone()),
-        par("GIT_COMMITTER_NAME", autor.nombre.clone()),
-        par("GIT_COMMITTER_EMAIL", autor.correo.clone()),
-    ]
+    ];
+    if let Some(a) = autor {
+        v.extend([
+            par("GIT_AUTHOR_NAME", a.nombre.clone()),
+            par("GIT_AUTHOR_EMAIL", a.correo.clone()),
+            par("GIT_COMMITTER_NAME", a.nombre.clone()),
+            par("GIT_COMMITTER_EMAIL", a.correo.clone()),
+        ]);
+    }
+    v
+}
+
+/// `entorno-push` de los adaptadores (F6-2): el entorno de [`entorno_git`]
+/// con el token del agente de `con`, sin autor, limitado a `url_remoto`.
+/// Solo `https://`: por ssh la cabecera no aplica (empujaria la identidad
+/// local) y por `http://` el token viajaria en claro. Un fallo al obtener el
+/// token sale como el de cualquier operacion del agente.
+pub(crate) fn entorno_push(con: &Conexion, url_remoto: &str) -> Paso<Vec<(OsString, OsString)>> {
+    let valida = url_remoto
+        .strip_prefix("https://")
+        .is_some_and(|r| !r.is_empty() && !r.chars().any(|c| c.is_control() || c == ' '));
+    if !valida {
+        return Err(Informe::nuevo(ENTORNO_PUSH, Resultado::Rechazada)
+            .con("motivo", "remoto-no-https")
+            .diagnostico(&t::REMOTO_NO_HTTPS));
+    }
+    let token = con.token(ENTORNO_PUSH, Rol::Agente)?;
+    Ok(entorno_git(con.proveedor(), url_remoto, &token, None))
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::plataforma::credenciales::CredencialesFijas;
+    use crate::plataforma::http::Base;
+    use crate::plataforma::{GitHub, GitLab, Personas, Plataforma};
+
+    fn valor<'a>(v: &'a [(OsString, OsString)], k: &str) -> Option<&'a str> {
+        v.iter().find(|(x, _)| x == k).and_then(|(_, y)| y.to_str())
+    }
+
+    /// F6-2: los dos adaptadores dan la cabecera del agente limitada a la
+    /// URL del remoto, sin autor; un remoto que no es `https://` se rechaza
+    /// sin pedir el token, y sin token del agente sale `motivo=identidad`.
+    #[test]
+    fn entorno_push_de_los_adaptadores() {
+        let fijas = || {
+            Box::new(CredencialesFijas::nuevas().con(Rol::Agente, Secreto::nuevo("tok")))
+                as Box<dyn crate::plataforma::Credenciales>
+        };
+        let gh = GitHub::nuevo(Base::github(), "o/r", fijas(), Personas::default());
+        let base_gl = Base::nueva("https://gitlab.example/api/v4").map_err(|_| ());
+        let gl = base_gl
+            .and_then(|b| GitLab::nuevo(b, "g/p", fijas(), Personas::default()).map_err(|_| ()));
+        assert!(gh.is_ok() && gl.is_ok());
+        let (Ok(gh), Ok(gl)) = (gh, gl) else {
+            return;
+        };
+        let casos: [(&dyn Plataforma, &str); 2] =
+            [(&gh, "x-access-token:tok"), (&gl, "oauth2:tok")];
+        for (p, basica) in casos {
+            let url = "https://forja.example/o/r.git";
+            let v = p.entorno_push(url).unwrap_or_default();
+            assert_eq!(
+                valor(&v, "GIT_CONFIG_KEY_0"),
+                Some("http.https://forja.example/o/r.git.extraheader")
+            );
+            let esperado = format!("Authorization: Basic {}", STANDARD.encode(basica));
+            assert_eq!(valor(&v, "GIT_CONFIG_VALUE_0"), Some(esperado.as_str()));
+            assert_eq!(valor(&v, "GIT_AUTHOR_NAME"), None);
+            for mala in [
+                "git@forja.example:o/r.git",
+                "http://forja.example/o/r.git",
+                "/tmp/r.git",
+                "https://",
+            ] {
+                let linea = p.entorno_push(mala).err().map(|i| i.linea());
+                assert_eq!(
+                    linea.as_deref(),
+                    Some("entorno-push rechazada motivo=remoto-no-https"),
+                    "{mala}"
+                );
+            }
+        }
+        let sin = GitHub::nuevo(
+            Base::github(),
+            "o/r",
+            Box::new(CredencialesFijas::nuevas()),
+            Personas::default(),
+        );
+        let i = sin
+            .ok()
+            .and_then(|p| p.entorno_push("https://github.com/o/r.git").err());
+        assert_eq!(
+            i.and_then(|i| i.valor("motivo").map(str::to_owned))
+                .as_deref(),
+            Some("identidad")
+        );
+    }
 }
diff --git a/crates/tc2/src/plataforma/github.rs b/crates/tc2/src/plataforma/github.rs
index a79e158..bedc67a 100644
--- a/crates/tc2/src/plataforma/github.rs
+++ b/crates/tc2/src/plataforma/github.rs
@@ -741,6 +741,13 @@ impl Plataforma for GitHub {
         Some(Proveedor::GitHub)
     }
 
+    fn entorno_push(
+        &self,
+        url_remoto: &str,
+    ) -> Paso<Vec<(std::ffi::OsString, std::ffi::OsString)>> {
+        super::entorno_git::entorno_push(&self.con, url_remoto)
+    }
+
     fn leer_plan(&self, rol: Rol) -> Paso<PlanObservado> {
         if rol == Rol::Revisor {
             return Err(rol_no_valido(LEER_PLAN, rol));
diff --git a/crates/tc2/src/plataforma/gitlab.rs b/crates/tc2/src/plataforma/gitlab.rs
index f55836f..7060ff3 100644
--- a/crates/tc2/src/plataforma/gitlab.rs
+++ b/crates/tc2/src/plataforma/gitlab.rs
@@ -957,6 +957,13 @@ impl Plataforma for GitLab {
         Some(Proveedor::GitLab)
     }
 
+    fn entorno_push(
+        &self,
+        url_remoto: &str,
+    ) -> Paso<Vec<(std::ffi::OsString, std::ffi::OsString)>> {
+        super::entorno_git::entorno_push(&self.con, url_remoto)
+    }
+
     fn leer_plan(&self, rol: Rol) -> Paso<PlanObservado> {
         if rol == Rol::Revisor {
             return Err(rol_no_valido(LEER_PLAN, rol));
diff --git a/crates/tc2/src/plataforma/mod.rs b/crates/tc2/src/plataforma/mod.rs
index 060123b..5d7e53a 100644
--- a/crates/tc2/src/plataforma/mod.rs
+++ b/crates/tc2/src/plataforma/mod.rs
@@ -401,6 +401,24 @@ pub trait Plataforma {
     /// el que ya haya abierto (`sin-cambios`).
     fn abrir_pr(&self, rama: &str, base: &str, titulo: &str, cuerpo: &str) -> Informe;
 
+    /// `entorno-push` (agente; adenda F6-2 a F1-4 y F1-6): las variables de
+    /// entorno para que `git` empuje a `url_remoto` con el token del agente
+    /// ([`entorno_git::entorno_push`]: cabecera limitada a esa URL, sin
+    /// autor). El token no sale del adaptador mas que en esa cabecera. Por
+    /// defecto (adaptadores de prueba), `rechazada motivo=no-soportada`.
+    fn entorno_push(
+        &self,
+        url_remoto: &str,
+    ) -> Paso<Vec<(std::ffi::OsString, std::ffi::OsString)>> {
+        let _ = url_remoto;
+        Err(Informe::nuevo(
+            entorno_git::ENTORNO_PUSH,
+            crate::contrato::Resultado::Rechazada,
+        )
+        .con("motivo", "no-soportada")
+        .diagnostico(&crate::i18n::catalogo::plataforma::ENTORNO_PUSH_NO_SOPORTADO))
+    }
+
     /// `estado-puertas` (agente o persona): checks con su origen,
     /// aprobaciones de personas y rama al dia.
     fn estado_puertas(&self, rol: Rol, pr: u64, modelo: &ModeloPuertas) -> Informe;
diff --git a/crates/tc2/src/revision/mod.rs b/crates/tc2/src/revision/mod.rs
new file mode 100644
index 0000000..6afb809
--- /dev/null
+++ b/crates/tc2/src/revision/mod.rs
@@ -0,0 +1,417 @@
+//! `tc2 review [--json] TASK-NNN` (F6-2): empuja la rama de la tarea y abre
+//! su PR contra `develop` con la identidad del agente (ADR F1-6, decision 1;
+//! adenda F6-2), pasa la tarea a `en-revision` y dice como esta el check
+//! `revision-ia`.
+//!
+//! Une piezas que ya existen, sin capacidades de plataforma nuevas salvo
+//! `entorno-push`:
+//!
+//! 1. `push` de la rama desde el worktree de la tarea con el token del
+//!    agente: [`crate::git::push`] con el puerto
+//!    [`Git::derivado`] del entorno que da
+//!    [`Plataforma::entorno_push`] (cabecera limitada a la URL de `origin`;
+//!    el puerto de la transaccion no se toca).
+//! 2. [`Plataforma::abrir_pr`] de la rama contra `develop`, idempotente.
+//! 3. `en-curso` -> `en-revision` por el punto unico de escritura
+//!    ([`Revisar`]), **despues** de tener el PR: el estado nunca dice «en
+//!    revision» sin PR. Si falla, la linea lleva el `pr=` y repetir converge.
+//! 4. [`Plataforma::estado_puertas`] con el agente para leer `revision-ia`
+//!    (D1: se lee y se informa; lo publica el revisor en F6-3).
+//!
+//! «PR enlazado a la tarea» sin tocar el esquema (F1-9): titulo
+//! `TASK-NNN: <titulo>` y cuerpo de [`cuerpo_pr`] con las rutas de la tarea
+//! y de su plan en la rama de estado; `board` ya enlaza PR y tarea por la
+//! rama.
+//!
+//! Linea: `review ok id=TASK-NNN rama=<r> pr=<n> estado=en-revision
+//! revision-ia=<ok|pendiente|fallo|sin-revisor|sin-leer>`. Repetirla sin
+//! nada nuevo sale `sin-cambios`.
+
+pub mod operacion;
+
+use std::path::Path;
+use std::time::Duration;
+
+use crate::config;
+use crate::contrato::{Informe, Resultado, Valor};
+use crate::estado::{self, Fallo, Operacion};
+use crate::git::{self, Git};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::revision as txt;
+use crate::plataforma::{ModeloPuertas, Paso, Plataforma, Rol};
+use crate::tarea::Tarea;
+use crate::toma;
+
+pub use operacion::{EN_REVISION, Revisar, desde_anotacion};
+
+/// Nombre de la orden y de la operacion de la rama de estado.
+pub const NOMBRE: &str = "review";
+
+/// Rama contra la que se abre el PR (D3: los `hotfix` llegan con F7).
+pub const BASE: &str = "develop";
+
+/// Nombre del check del revisor IA.
+pub const REVISION_IA: &str = "revision-ia";
+
+/// Construye el adaptador de la plataforma de `origin` con las credenciales
+/// del proceso (en produccion, la deteccion de `start` y los tokens
+/// `TC2_TOKEN_*` o la App del agente).
+pub type Conectar<'a> = dyn Fn() -> Paso<Box<dyn Plataforma>> + 'a;
+
+/// Lo que necesita `review` del entorno, para probarlo sin terminal ni
+/// tokens: lo de `start` mas la plataforma y las esperas del push.
+pub struct Entorno<'a> {
+    /// Carpeta, puerto Git, quien la lanza, catalogo e instante.
+    pub base: &'a toma::Entorno<'a>,
+    /// El adaptador; `review` le pide siempre con el rol de agente.
+    pub conectar: &'a Conectar<'a>,
+    /// Reintentos del push (en produccion, [`git::ESPERAS_PUSH`]).
+    pub esperas: &'a [Duration],
+}
+
+/// `review TASK-NNN` (sin `--json`, que quita `lib.rs`).
+pub fn orden(args: &[String], e: &Entorno<'_>) -> Informe {
+    let uso = |motivo: &str| {
+        Informe::nuevo(NOMBRE, Resultado::Uso)
+            .con("motivo", motivo)
+            .diagnostico(&txt::USO_REVIEW)
+    };
+    match args {
+        [id] if git::es_id(id) => review_en(e, id),
+        [_] => uso("id-invalido"),
+        [] => uso("sin-id"),
+        _ => uso("argumentos-de-mas"),
+    }
+}
+
+/// `review` de la tarea `id` en el repo de `e`.
+pub fn review_en(e: &Entorno<'_>, id: &str) -> Informe {
+    let i = match paso(e, id) {
+        Ok(i) | Err(i) => i,
+    };
+    de_review(i, id)
+}
+
+/// Titulo del PR: `TASK-NNN: <titulo>`.
+pub fn titulo_pr(t: &Tarea) -> String {
+    format!("{}: {}", t.id, t.titulo)
+}
+
+/// Cuerpo del PR: el ID y las rutas de la tarea y de su plan en la rama de
+/// estado (el enlace PR -> tarea; el de tarea -> PR lo da `board` por la
+/// rama).
+pub fn cuerpo_pr(t: &Tarea) -> String {
+    let (id, rama) = (&t.id, estado::RAMA);
+    format!(
+        "Tarea {id}: {}\n\n\
+         - Tarea: `{rama}:{id}/tarea.md`\n\
+         - Plan: `{rama}:{id}/planificacion/plan-final.md`\n\n\
+         Abierto por `tc2 review {id}` con la identidad del agente (F6-2).\n",
+        t.titulo
+    )
+}
+
+/// Lo que dejo [`publicar`].
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub struct Publicado {
+    /// Numero del PR (en GitLab, el `iid` del MR).
+    pub pr: u64,
+    /// Si el push publico algo (`push ok`; `sin-cambios`, no).
+    pub empujado: bool,
+    /// Si el PR se acaba de abrir (`abrir-pr ok`; ya abierto, no).
+    pub abierto: bool,
+}
+
+/// Empuja la rama actual del worktree `wt` a `url_remoto` (la de `origin`)
+/// con el entorno del agente que da `p` y abre su PR contra [`BASE`] con
+/// `titulo` y `cuerpo`. El puerto `g` no cambia: el push corre con un
+/// [`Git::derivado`]. Lo usan `review` y su E2E.
+pub fn publicar(
+    p: &dyn Plataforma,
+    g: &Git,
+    wt: &Path,
+    url_remoto: &str,
+    (rama, titulo, cuerpo): (&str, &str, &str),
+    esperas: &[Duration],
+) -> Result<Publicado, Informe> {
+    let agente = g.derivado(p.entorno_push(url_remoto)?);
+    let push = git::push(&agente, wt, esperas);
+    if !push.resultado.es_exito() {
+        return Err(push);
+    }
+    let pr = p.abrir_pr(rama, BASE, titulo, cuerpo);
+    let numero = pr.valor("pr").and_then(|n| n.parse::<u64>().ok());
+    match (pr.resultado, numero) {
+        (Resultado::Ok | Resultado::SinCambios, Some(n)) => Ok(Publicado {
+            pr: n,
+            empujado: push.resultado == Resultado::Ok,
+            abierto: pr.resultado == Resultado::Ok,
+        }),
+        _ => Err(pr),
+    }
+}
+
+/// Como esta `revision-ia` en el PR `pr`, leido con el agente: `sin-revisor`
+/// si la configuracion no da su origen (`plataforma.revisor_id`, TASK-118),
+/// y si no `ok`, `pendiente` o `fallo` segun `estado-puertas` con solo ese
+/// check en el modelo; `sin-leer` si la lectura falla (no cambia el
+/// resultado de `review`).
+pub fn revision_ia(p: &dyn Plataforma, pr: u64, revisor_id: Option<u64>) -> &'static str {
+    let Some(id) = revisor_id.filter(|id| *id != 0) else {
+        return "sin-revisor";
+    };
+    let modelo = ModeloPuertas {
+        checks: vec![(REVISION_IA.to_owned(), id)],
+        exigir_impuestas: Vec::new(),
+    };
+    let i = p.estado_puertas(Rol::Agente, pr, &modelo);
+    let en = |clave: &str| {
+        i.claves.iter().any(|(k, v)| {
+            k == clave && matches!(v, Valor::Lista(l) if l.iter().any(|c| c == REVISION_IA))
+        })
+    };
+    match i.resultado {
+        Resultado::Rechazada if i.valor("motivo") == Some("check-rojo") && en("checks") => "fallo",
+        Resultado::Esperando if en("falta") => "pendiente",
+        Resultado::Ok | Resultado::Esperando => "ok",
+        _ => "sin-leer",
+    }
+}
+
+/// El informe como de `review`: el comando, `id=` el primero y sin la clave
+/// `operacion` de la transaccion. Un `sinred` dice `pendiente=no` (`review`
+/// exige red).
+fn de_review(mut i: Informe, id: &str) -> Informe {
+    NOMBRE.clone_into(&mut i.comando);
+    i.claves.retain(|(k, _)| k != "operacion" && k != "id");
+    i.claves
+        .insert(0, ("id".to_owned(), Valor::Texto(id.to_owned())));
+    if i.resultado == Resultado::Sinred && i.valor("pendiente").is_none() {
+        i = i.con("pendiente", "no");
+    }
+    i
+}
+
+fn rechazada(motivo: &str, m: Mensaje) -> Informe {
+    Informe::nuevo(NOMBRE, Resultado::Rechazada)
+        .con("motivo", motivo)
+        .diagnostico(m)
+}
+
+fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
+    let (g, dir, ahora) = (e.base.git, e.base.dir, e.base.ahora);
+    let principal = git::localizar_principal(g, dir)?;
+    // 1. Lo de ahora del remoto: el estado y la rama.
+    if git::estado::hay_remoto(g, &principal, NOMBRE)? {
+        let f = git::fetch(g, &principal);
+        if !f.resultado.es_exito() {
+            return Err(f);
+        }
+    }
+    let vista = estado::leer(g, dir)?;
+    let quien = (e.base.quien)()?;
+    // 2. El paso a revision en seco: los rechazos, antes de empujar nada.
+    let sondeo = Revisar::nueva(id, &quien, ahora);
+    if let Err(f) = sondeo.aplicar(&vista) {
+        return Err(match f {
+            Fallo::Rechazada {
+                motivo,
+                diagnostico,
+            } => {
+                let i = rechazada(&motivo, diagnostico);
+                match sondeo.dueno() {
+                    Some(l) => i.con("asignado", l),
+                    None => i,
+                }
+            }
+            Fallo::Error(d) => Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d),
+        });
+    }
+    let t = match estado::tarea_de(&vista, id) {
+        Ok((t, _)) => t,
+        Err(Fallo::Rechazada {
+            motivo,
+            diagnostico,
+        }) => return Err(rechazada(&motivo, diagnostico)),
+        Err(Fallo::Error(d)) => return Err(Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d)),
+    };
+    // 3. La rama, contra `develop` (D3) y con su worktree.
+    let Some(rama) = t.rama.clone() else {
+        return Err(rechazada(
+            "sin-rama",
+            Mensaje::nuevo(&txt::SIN_RAMA).con("id", id),
+        ));
+    };
+    if git::base_de_rama(&rama) != config::BASE {
+        return Err(rechazada(
+            "carril-no-soportado",
+            Mensaje::nuevo(&txt::CARRIL_NO_SOPORTADO)
+                .con("id", id)
+                .con("rama", rama.as_str()),
+        )
+        .con("rama", rama.as_str()));
+    }
+    let wt = principal.join(git::CARPETA_WORKTREES).join(id);
+    let registrado = wt.exists()
+        && git::worktrees(g, &principal, NOMBRE)?
+            .iter()
+            .any(|w| w.rama.as_deref() == Some(rama.as_str()) && git::misma_ruta(&w.ruta, &wt));
+    if !registrado {
+        return Err(rechazada(
+            "sin-worktree",
+            Mensaje::nuevo(&txt::SIN_WORKTREE)
+                .con("id", id)
+                .con("ruta", git::worktree_relativo(id))
+                .con("rama", rama.as_str()),
+        )
+        .con("rama", rama.as_str())
+        .con("arreglo", format!("tc2 start {id}")));
+    }
+    let Some(url) = git::estado::url_origin(g, &wt, NOMBRE)? else {
+        return Err(rechazada("plataforma", Mensaje::nuevo(&txt::SIN_ORIGIN)));
+    };
+    // 4. Push y PR con el agente.
+    let p = (e.conectar)()?;
+    let publicado = publicar(
+        p.as_ref(),
+        g,
+        &wt,
+        &url,
+        (&rama, &titulo_pr(&t), &cuerpo_pr(&t)),
+        e.esperas,
+    )?;
+    let pr = publicado.pr.to_string();
+    // 5. El estado, con el PR ya abierto.
+    let personas = crate::roadmap::personas(g, dir);
+    let generar = crate::roadmap::generador(&personas);
+    let ctx = estado::Contexto::nuevo(g, dir, e.base.catalogo, Some(&generar));
+    let op = Revisar::nueva(id, &quien, ahora);
+    let tr = estado::transaccion(&ctx, &op);
+    match tr.resultado {
+        Resultado::Ok | Resultado::SinCambios => {}
+        _ => {
+            let mut tr = tr.con("rama", rama.as_str()).con("pr", pr);
+            if let Some(l) = op.dueno() {
+                tr = tr.con("asignado", l);
+            }
+            return Err(tr);
+        }
+    }
+    // 6. `revision-ia`, leido con el agente.
+    let revisor = config::de_base(g, &principal)
+        .ok()
+        .and_then(|c| c.plataforma)
+        .and_then(|p| p.revisor_id);
+    let ria = revision_ia(p.as_ref(), publicado.pr, revisor);
+    let nada_nuevo =
+        !publicado.empujado && !publicado.abierto && tr.resultado == Resultado::SinCambios;
+    let resultado = if nada_nuevo {
+        Resultado::SinCambios
+    } else {
+        Resultado::Ok
+    };
+    Ok(Informe::nuevo(NOMBRE, resultado)
+        .con("rama", rama.as_str())
+        .con("pr", pr)
+        .con("estado", EN_REVISION)
+        .con(REVISION_IA, ria))
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::plataforma::{Firma, PrAbierto, ReglasLeidas, Usuario};
+    use std::cell::RefCell;
+
+    /// Plataforma de prueba: solo `estado-puertas`, con el informe fijo y
+    /// el modelo recibido.
+    struct Puertas {
+        informe: Informe,
+        modelo: RefCell<Option<ModeloPuertas>>,
+    }
+
+    fn sin_uso(op: &str) -> Informe {
+        Informe::nuevo(op, Resultado::Error)
+    }
+
+    impl Plataforma for Puertas {
+        fn prs_abiertos(&self, _: Rol) -> Paso<Vec<PrAbierto>> {
+            Err(sin_uso("prs-abiertos"))
+        }
+        fn abrir_pr(&self, _: &str, _: &str, _: &str, _: &str) -> Informe {
+            sin_uso("abrir-pr")
+        }
+        fn estado_puertas(&self, rol: Rol, _: u64, modelo: &ModeloPuertas) -> Informe {
+            assert_eq!(rol, Rol::Agente);
+            *self.modelo.borrow_mut() = Some(modelo.clone());
+            self.informe.clone()
+        }
+        fn merge_pr(&self, _: u64, _: &str) -> Informe {
+            sin_uso("merge-pr")
+        }
+        fn leer_puertas(&self, _: Rol, _: &str) -> Paso<ReglasLeidas> {
+            Err(sin_uso("leer-puertas"))
+        }
+        fn leer_permisos_ci(&self) -> Informe {
+            sin_uso("leer-permisos-ci")
+        }
+        fn firma_commit(&self, _: &str) -> Paso<Firma> {
+            Err(sin_uso("firma-commit"))
+        }
+        fn identidad(&self, _: Rol) -> Paso<Usuario> {
+            Err(sin_uso("identidad"))
+        }
+    }
+
+    fn con(informe: Informe) -> Puertas {
+        Puertas {
+            informe,
+            modelo: RefCell::new(None),
+        }
+    }
+
+    /// Criterio «check `revision-ia`» (unitario): sin `revisor_id` no se
+    /// pregunta (`sin-revisor`); con el, el modelo lleva solo ese check con
+    /// su origen y la linea de `estado-puertas` se traduce.
+    #[test]
+    fn revision_ia_de_estado_puertas() {
+        let p = con(Informe::nuevo("estado-puertas", Resultado::Ok));
+        assert_eq!(revision_ia(&p, 7, None), "sin-revisor");
+        assert_eq!(revision_ia(&p, 7, Some(0)), "sin-revisor");
+        assert!(p.modelo.borrow().is_none());
+        assert_eq!(revision_ia(&p, 7, Some(99)), "ok");
+        assert_eq!(
+            p.modelo.borrow().as_ref().map(|m| m.checks.clone()),
+            Some(vec![("revision-ia".to_owned(), 99)])
+        );
+        let esperando = |falta: &[&str]| {
+            Informe::nuevo("estado-puertas", Resultado::Esperando)
+                .con_lista("falta", falta.iter().map(|s| (*s).to_owned()).collect())
+        };
+        let p = con(esperando(&["revision-ia", "aprobacion"]));
+        assert_eq!(revision_ia(&p, 7, Some(99)), "pendiente");
+        let p = con(esperando(&["aprobacion"]));
+        assert_eq!(revision_ia(&p, 7, Some(99)), "ok");
+        let rojo = Informe::nuevo("estado-puertas", Resultado::Rechazada)
+            .con("motivo", "check-rojo")
+            .con_lista("checks", vec!["revision-ia".to_owned()]);
+        assert_eq!(revision_ia(&con(rojo), 7, Some(99)), "fallo");
+        let p = con(Informe::nuevo("estado-puertas", Resultado::Sinred));
+        assert_eq!(revision_ia(&p, 7, Some(99)), "sin-leer");
+    }
+
+    /// Criterio «PR enlazado» (unitario): titulo con el ID y cuerpo con las
+    /// rutas de la tarea y del plan en la rama de estado.
+    #[test]
+    fn titulo_y_cuerpo_del_pr() {
+        let t = Tarea::nueva("TASK-048", "Review abre el PR", "2026-11-19");
+        assert_eq!(titulo_pr(&t), "TASK-048: Review abre el PR");
+        assert_eq!(
+            cuerpo_pr(&t),
+            "Tarea TASK-048: Review abre el PR\n\n\
+             - Tarea: `taskcodetwo/tareas:TASK-048/tarea.md`\n\
+             - Plan: `taskcodetwo/tareas:TASK-048/planificacion/plan-final.md`\n\n\
+             Abierto por `tc2 review TASK-048` con la identidad del agente (F6-2).\n"
+        );
+    }
+}
diff --git a/crates/tc2/src/revision/operacion.rs b/crates/tc2/src/revision/operacion.rs
new file mode 100644
index 0000000..eaec532
--- /dev/null
+++ b/crates/tc2/src/revision/operacion.rs
@@ -0,0 +1,220 @@
+//! El paso a revision de una tarea como operacion de la rama de estado
+//! (F6-2; ADR F1-2). Como [`crate::toma::Tomar`] y
+//! [`crate::ciclo::Pausar`], pasa por el punto unico de escritura y decide
+//! solo con el estado que recibe; lee unicamente el `tarea.md` de su tarea.
+//!
+//! - Quien: quien la lleva o cualquiera si no la lleva nadie (la regla de
+//!   `start`); la de otra persona, `rechazada motivo=de-otra-persona` (o
+//!   `asignado-heredado` si es un texto de la 0.16).
+//! - `en-curso` pasa a `en-revision`, con `actualizado` y una fila `review`
+//!   en `## Transiciones`; el resto del `tarea.md`, byte a byte.
+//! - Ya `en-revision`: nada que escribir (`sin-cambios`).
+//! - Cualquier otro estado se rechaza con su motivo.
+
+use std::cell::RefCell;
+
+use crate::estado::{Cambio, Estado, Fallo, Operacion, de_otra_persona, por_estado, tarea_de};
+use crate::i18n::catalogo::revision as txt;
+use crate::identidad::Quien;
+use crate::tarea::{Asignado, Transicion};
+use crate::toma::instante;
+use crate::toma::operacion::{EN_CURSO, MODO};
+
+use super::NOMBRE;
+
+/// Estado en el que queda la tarea revisada.
+pub const EN_REVISION: &str = "en-revision";
+
+/// Pasar la tarea `id` a revision.
+#[derive(Debug)]
+pub struct Revisar {
+    id: String,
+    asignado: Asignado,
+    decidido_por: &'static str,
+    ahora: u64,
+    dueno: RefCell<Option<String>>,
+}
+
+impl Revisar {
+    /// El paso a revision de `id` por `quien` en el instante `ahora`
+    /// (segundos desde 1970, UTC).
+    pub fn nueva(id: &str, quien: &Quien, ahora: u64) -> Self {
+        Self::con(id, quien.asignado(), quien.decidido_por(), ahora)
+    }
+
+    fn con(id: &str, asignado: Asignado, decidido_por: &'static str, ahora: u64) -> Self {
+        Self {
+            id: id.to_owned(),
+            asignado,
+            decidido_por,
+            ahora,
+            dueno: RefCell::new(None),
+        }
+    }
+
+    /// Login de quien la lleva si el ultimo intento la rechazo por
+    /// `de-otra-persona`.
+    pub fn dueno(&self) -> Option<String> {
+        self.dueno.borrow().clone()
+    }
+}
+
+impl Operacion for Revisar {
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
+            self.ahora
+        )
+    }
+
+    fn exige_red(&self) -> bool {
+        // `review` ya empujo la rama y abrio el PR: sin red no hay nada.
+        true
+    }
+
+    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
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
+        match t.estado.as_str() {
+            EN_REVISION => return Ok(Vec::new()),
+            EN_CURSO => {}
+            otro => return Err(por_estado(&self.id, otro, NOMBRE, &[EN_CURSO])),
+        }
+        let instante = instante(self.ahora);
+        EN_REVISION.clone_into(&mut t.estado);
+        t.actualizado = instante.get(..10).unwrap_or_default().to_owned();
+        cuerpo.anadir_transicion(&Transicion {
+            fecha: &instante,
+            fase: NOMBRE,
+            modo: MODO,
+            decidido_por: self.decidido_por,
+            version: crate::VERSION,
+        });
+        Ok(vec![Cambio::nuevo(
+            format!("{}/tarea.md", self.id),
+            t.escribir(&cuerpo),
+        )])
+    }
+}
+
+/// Reconstruye un [`Revisar`] desde su anotacion
+/// `review|TASK-NNN|<id>|<login>|<instante>` (para el catalogo). Exige red y
+/// nunca queda pendiente; si se re-ejecutara, sin persona delante:
+/// `decidido_por: agente`.
+pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
+    let mut partes = a.splitn(5, '|');
+    if partes.next()? != NOMBRE {
+        return None;
+    }
+    let id = partes.next()?;
+    let cuenta: u64 = partes.next()?.parse().ok()?;
+    let asignado = Asignado::persona(cuenta, partes.next()?)?;
+    let ahora: u64 = partes.next()?.parse().ok()?;
+    crate::git::es_id(id)
+        .then(|| Box::new(Revisar::con(id, asignado, "agente", ahora)) as Box<dyn Operacion>)
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
+    fn md(estado: &str, asignado: Option<Asignado>) -> String {
+        let mut t = Tarea::nueva("TASK-001", "Uno", "2026-11-01");
+        t.estado = estado.to_owned();
+        t.asignado_a = asignado;
+        t.rama = Some("feature/task-001-uno".to_owned());
+        t.escribir(&Cuerpo::plantilla())
+    }
+
+    fn estado(t: String) -> EnMemoria {
+        EnMemoria(BTreeMap::from([("TASK-001/tarea.md".to_owned(), t)]))
+    }
+
+    fn motivo(r: &Result<Vec<Cambio>, Fallo>) -> Option<&str> {
+        match r {
+            Err(Fallo::Rechazada { motivo, .. }) => Some(motivo),
+            _ => None,
+        }
+    }
+
+    /// `en-curso` pasa a `en-revision` con su fila; ya en revision, nada;
+    /// otro estado o de otra persona, rechazada.
+    #[test]
+    fn de_en_curso_a_en_revision() {
+        let antes = md("en-curso", ana().asignado().into());
+        let op = Revisar::nueva("TASK-001", &ana(), AHORA);
+        let r = op.aplicar(&estado(antes.clone()));
+        let texto = r
+            .as_ref()
+            .ok()
+            .and_then(|c| c.first())
+            .map(|c| c.contenido.clone())
+            .unwrap_or_default();
+        let esperado = antes
+            .replace("estado: \"en-curso\"", "estado: \"en-revision\"")
+            .replace("actualizado: \"2026-11-01\"", "actualizado: \"2026-11-17\"")
+            + &format!(
+                "| 2026-11-17T10:20:30Z | review | manual | agente | {} |\n",
+                crate::VERSION
+            );
+        assert_eq!(texto, esperado, "{r:?}");
+        let ya = md("en-revision", ana().asignado().into());
+        assert_eq!(op.aplicar(&estado(ya)), Ok(Vec::new()));
+        let r = op.aplicar(&estado(md("planificada", None)));
+        assert_eq!(motivo(&r), Some("estado-no-valido"));
+        let r = op.aplicar(&estado(md("en-curso", Asignado::persona(9, "beto"))));
+        assert_eq!(motivo(&r), Some("de-otra-persona"));
+        assert_eq!(op.dueno().as_deref(), Some("beto"));
+        assert!(op.exige_red());
+    }
+
+    #[test]
+    fn anotacion_ida_y_vuelta() {
+        let op = Revisar::nueva("TASK-001", &ana(), AHORA);
+        assert_eq!(op.anotacion(), format!("review|TASK-001|7|ana|{AHORA}"));
+        let r = desde_anotacion(&op.anotacion());
+        assert_eq!(r.map(|o| o.anotacion()), Some(op.anotacion()));
+        for mala in [
+            "review|TASK-1|7|ana|1",
+            "review|TASK-001|x|ana|1",
+            "review|TASK-001|7|ana|x",
+            "review|TASK-001|7|ana",
+            "start|TASK-001|7|ana|1",
+        ] {
+            assert!(desde_anotacion(mala).is_none(), "{mala}");
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
tareas/{01-en-diseno => 02-en-curso}/TASK-048/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-048/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-048/tarea.md                                    | 3 ++-
 3 files changed, 2 insertions(+), 1 deletion(-)
````

## Tests del diff (5 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/agente_git/mod.rs" ":(literal)crates/tc2/tests/identidades.rs" ":(literal)crates/tc2/tests/plataforma_e2e.rs" ":(literal)crates/tc2/tests/plataforma_gitlab_e2e.rs" ":(literal)crates/tc2/tests/revision.rs"
````

````
crates/tc2/tests/agente_git/mod.rs        |  32 ++++++++++++++++---
 crates/tc2/tests/identidades.rs           |   4 +--
 crates/tc2/tests/plataforma_e2e.rs        | 112 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/plataforma_gitlab_e2e.rs | 105 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/revision.rs              | 263 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 5 files changed, 509 insertions(+), 7 deletions(-)
````
