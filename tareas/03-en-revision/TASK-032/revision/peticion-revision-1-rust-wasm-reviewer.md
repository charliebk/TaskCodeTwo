# Peticion de revision — TASK-032 (ronda 1)

- Tarea: TASK-032 — F3-5 tc2 board y tc2 status
- Rama revisada: feature/task-032-f3-5-board-y-status
- Rama base: develop
- Commit revisado (HEAD): 5a5a9e9e08d7cc4180d8dd4ee3760a24ec78677a
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-032 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-032/tarea.md
- tareas/03-en-revision/TASK-032/planificacion/
- crates/tc2/src/i18n/catalogo/cli.rs
- crates/tc2/src/i18n/catalogo/mod.rs
- crates/tc2/src/i18n/catalogo/tablero.rs
- crates/tc2/src/identidad.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/plataforma/github.rs
- crates/tc2/src/plataforma/gitlab.rs
- crates/tc2/src/plataforma/mod.rs
- crates/tc2/src/roadmap/generar.rs
- crates/tc2/src/roadmap/mod.rs
- crates/tc2/src/siguiente.rs
- crates/tc2/src/situacion.rs
- crates/tc2/src/tablero.rs
- crates/tc2/tests/prs_abiertos.rs
- crates/tc2/tests/tablero.rs

## Commits a revisar (git log develop..HEAD)

````
5a5a9e9 docs(TASK-032): autorrevision con rust-wasm-reviewer y code-quality-reviewer
8a3a2c5 feat(TASK-032): board cruza los PR por rama con modos degradados
55b3bf0 feat(TASK-032): prs-abiertos en el puerto Plataforma con adenda a F1-4 y F1-13
7591793 feat(TASK-032): tc2 status y tc2 board sobre el estado, sin plataforma
db88831 refactor(TASK-032): quien y rama de la tarea en situacion
03b47aa chore(TASK-032): tarea en curso
````

## Diff de tu dominio (15 fichero(s) de 17; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/i18n/catalogo/cli.rs b/crates/tc2/src/i18n/catalogo/cli.rs
index bf908a1..212ad1f 100644
--- a/crates/tc2/src/i18n/catalogo/cli.rs
+++ b/crates/tc2/src/i18n/catalogo/cli.rs
@@ -10,8 +10,8 @@ crate::i18n::textos! {
         en: "tc2: could not write the output: {error}",
     }
     USO {
-        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
-        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
+        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
+        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
     }
     FALTA_LA_ORDEN {
         es: "tc2: falta la orden. {uso}",
diff --git a/crates/tc2/src/i18n/catalogo/mod.rs b/crates/tc2/src/i18n/catalogo/mod.rs
index adba285..e3cab54 100644
--- a/crates/tc2/src/i18n/catalogo/mod.rs
+++ b/crates/tc2/src/i18n/catalogo/mod.rs
@@ -14,6 +14,7 @@ pub mod init;
 pub mod plataforma;
 pub mod roadmap;
 pub mod siguiente;
+pub mod tablero;
 
 use super::Texto;
 
@@ -33,6 +34,7 @@ pub fn todos() -> impl Iterator<Item = &'static Texto> {
         plataforma::TODOS,
         roadmap::TODOS,
         siguiente::TODOS,
+        tablero::TODOS,
     ]
     .into_iter()
     .flatten()
diff --git a/crates/tc2/src/i18n/catalogo/tablero.rs b/crates/tc2/src/i18n/catalogo/tablero.rs
new file mode 100644
index 0000000..dc8ab96
--- /dev/null
+++ b/crates/tc2/src/i18n/catalogo/tablero.rs
@@ -0,0 +1,36 @@
+//! Mensajes de `tc2 board` y `tc2 status` (F3-5).
+
+crate::i18n::textos! {
+    USO_STATUS {
+        es: "uso: status [--json]",
+        en: "usage: status [--json]",
+    }
+    USO_BOARD {
+        es: "uso: board [--sin-plataforma] [--json]",
+        en: "usage: board [--sin-plataforma] [--json]",
+    }
+    AVISO_PLATAFORMA_NO_CONFIGURADA {
+        es: "aviso: no se consultaron los PR: no se reconoce la plataforma de origin; configura la clave plataforma o usa --sin-plataforma",
+        en: "warning: pull requests were not read: the platform of origin is not recognised; set the plataforma key or use --sin-plataforma",
+    }
+    AVISO_PLATAFORMA_SIN_CREDENCIALES {
+        es: "aviso: no se consultaron los PR: falta un token que valga (TC2_TOKEN_AGENTE, o TC2_TOKEN_PERSONA en un terminal)",
+        en: "warning: pull requests were not read: no valid token (TC2_TOKEN_AGENTE, or TC2_TOKEN_PERSONA in a terminal)",
+    }
+    AVISO_PLATAFORMA_SIN_RED {
+        es: "aviso: no se consultaron los PR: sin red; el estado de las tareas es el que conoce este clon",
+        en: "warning: pull requests were not read: no network; task state is the one this clone knows",
+    }
+    AVISO_PLATAFORMA_ERROR {
+        es: "aviso: no se pudieron leer los PR de la plataforma; el estado de las tareas sale completo",
+        en: "warning: the platform pull requests could not be read; task state is shown in full",
+    }
+    AVISO_REVISION_SIN_PR {
+        es: "aviso: en revision sin PR abierto listo: {ids}",
+        en: "warning: in review without an open ready pull request: {ids}",
+    }
+    AVISO_PR_SIN_REVISION {
+        es: "aviso: con PR abierto listo y sin pasar a revision: {ids}",
+        en: "warning: with an open ready pull request but not in review: {ids}",
+    }
+}
diff --git a/crates/tc2/src/identidad.rs b/crates/tc2/src/identidad.rs
index bf12802..62e9b69 100644
--- a/crates/tc2/src/identidad.rs
+++ b/crates/tc2/src/identidad.rs
@@ -416,7 +416,7 @@ pub fn personas_de_base(g: &Git, dir: &Path) -> Paso<Personas> {
 /// La clave `plataforma` de la configuracion de la base, si la hay. Sin
 /// base o sin configuracion se detecta solo por el host; una configuracion
 /// que no vale falla.
-fn clave_plataforma(g: &Git, dir: &Path) -> Paso<Option<(&'static str, String)>> {
+pub(crate) fn clave_plataforma(g: &Git, dir: &Path) -> Paso<Option<(&'static str, String)>> {
     match config::de_base(g, dir) {
         Ok(c) => Ok(c.plataforma.map(|p| (p.proveedor.nombre(), p.api))),
         Err(i) if matches!(i.valor("motivo"), Some("sin-base" | "sin-config")) => Ok(None),
@@ -534,6 +534,9 @@ mod tests {
     }
 
     impl Plataforma for Falsa {
+        fn prs_abiertos(&self, _: Rol) -> Paso<Vec<crate::plataforma::PrAbierto>> {
+            Err(sin_uso("prs-abiertos"))
+        }
         fn abrir_pr(&self, _: &str, _: &str, _: &str, _: &str) -> Informe {
             sin_uso("abrir-pr")
         }
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 35e01be..fd7cfd4 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -15,7 +15,8 @@
 //! maquina, F2-12 y F2-14) y [`desinstalar`] (quitar TaskCodeTwo, F2-14) e
 //! [`importar`] (alta en bloque con dependencias e hito, F3-2) y [`grafo`] y
 //! [`siguiente`] (orden de trabajo y `next`, F3-3) y [`situacion`] y
-//! [`roadmap`] (el calculo comun y `roadmap`, F3-4). Las capas
+//! [`roadmap`] (el calculo comun y `roadmap`, F3-4) y [`tablero`]
+//! (`board` y `status`, F3-5). Las capas
 //! que vienen entran como modulos de esta biblioteca.
 
 pub mod actualizar_cli;
@@ -38,6 +39,7 @@ pub mod plataforma;
 pub mod roadmap;
 pub mod siguiente;
 pub mod situacion;
+pub mod tablero;
 pub mod tarea;
 pub mod version;
 mod yaml;
@@ -120,6 +122,10 @@ where
             let (json, resto) = con_json(resto);
             roadmap::orden(&resto, json, idioma, &catalogo)
         }
+        [orden, resto @ ..] if orden == tablero::STATUS || orden == tablero::BOARD => {
+            let (json, resto) = con_json(resto);
+            orden_tablero(orden, &resto, json, idioma)
+        }
         [orden, resto @ ..] if orden == version::VERSION => {
             let (json, resto) = con_json(resto);
             orden_version(json, &resto, idioma)
@@ -271,6 +277,26 @@ fn orden_identidad(args: &[String]) -> Informe {
     )
 }
 
+/// `status` o `board` (F3-5) en la carpeta actual.
+fn orden_tablero(orden: &str, args: &[String], json: bool, idioma: Idioma) -> Salida {
+    let dir = match tablero::carpeta_actual(orden) {
+        Ok(d) => d,
+        Err(i) => return i.salida_en(json, idioma),
+    };
+    let g = git::Git::nuevo();
+    let conectar = tablero::conectar_produccion(&g, &dir);
+    let e = tablero::Entorno {
+        dir: &dir,
+        git: &g,
+        conectar: &conectar,
+    };
+    if orden == tablero::STATUS {
+        tablero::orden_status(args, json, idioma, &e)
+    } else {
+        tablero::orden_board(args, json, idioma, &e)
+    }
+}
+
 /// Catalogo de produccion: reconstruye las operaciones de la rama de estado
 /// desde su anotacion. Las operaciones nuevas se anaden aqui.
 fn catalogo(anotacion: &str) -> Option<Box<dyn estado::Operacion>> {
diff --git a/crates/tc2/src/plataforma/github.rs b/crates/tc2/src/plataforma/github.rs
index 5ee1adf..ca32cde 100644
--- a/crates/tc2/src/plataforma/github.rs
+++ b/crates/tc2/src/plataforma/github.rs
@@ -16,7 +16,9 @@ use super::credenciales::{Credenciales, Rol};
 use super::errores::{self, es_aviso_de_plan};
 use super::http::{Base, Metodo, codificar};
 use super::puertas::{self, EstadoCheck, Lecturas};
-use super::{Firma, ModeloPuertas, Paso, Personas, Plataforma, Proveedor, ReglasLeidas, Usuario};
+use super::{
+    Firma, ModeloPuertas, Paso, Personas, Plataforma, PrAbierto, Proveedor, ReglasLeidas, Usuario,
+};
 use crate::contrato::{Informe, Resultado, Valor, corto};
 use crate::i18n::catalogo::plataforma as t;
 
@@ -26,6 +28,7 @@ const LEER_PUERTAS: &str = "leer-puertas";
 const LEER_PERMISOS_CI: &str = "leer-permisos-ci";
 const FIRMA_COMMIT: &str = "firma-commit";
 const IDENTIDAD: &str = "identidad";
+const PRS_ABIERTOS: &str = "prs-abiertos";
 
 /// Garantias que en GitHub dan los rulesets (tier «Free publico;
 /// Pro/Team/Enterprise» de la tabla de F1-13): sin ellos, el 403 de plan.
@@ -38,6 +41,7 @@ enum Endpoint<'a> {
     Usuario,
     Pr(u64),
     PrsAbiertos { cabeza: &'a str, base: &'a str },
+    TodosLosPrsAbiertos,
     CrearPr,
     Merge(u64),
     ReglasRama(&'a str),
@@ -62,6 +66,9 @@ impl Endpoint<'_> {
                     codificar(base)
                 ),
             ),
+            Self::TodosLosPrsAbiertos => {
+                (Metodo::Get, format!("{r}/pulls?state=open&per_page=100"))
+            }
             Self::CrearPr => (Metodo::Post, format!("{r}/pulls")),
             Self::Merge(n) => (Metodo::Put, format!("{r}/pulls/{n}/merge")),
             Self::ReglasRama(rama) => (Metodo::Get, format!("{r}/rules/branches/{rama}")),
@@ -116,6 +123,33 @@ struct PrApi {
     base: RefApi,
 }
 
+/// Un PR de la lista de abiertos: la cabeza con su repo, para no contar los
+/// de forks.
+#[derive(Deserialize)]
+struct PrListaApi {
+    number: u64,
+    #[serde(default)]
+    draft: bool,
+    head: CabezaApi,
+    base: CabezaApi,
+}
+
+#[derive(Deserialize)]
+struct CabezaApi {
+    #[serde(rename = "ref")]
+    nombre: String,
+    #[serde(default)]
+    sha: String,
+    /// `null` si el repo de un fork se borro.
+    #[serde(default)]
+    repo: Option<RepoApi>,
+}
+
+#[derive(Deserialize)]
+struct RepoApi {
+    id: u64,
+}
+
 #[derive(Deserialize)]
 struct CheckRunsApi {
     check_runs: Vec<CheckRunApi>,
@@ -266,6 +300,37 @@ impl GitHub {
 }
 
 impl Plataforma for GitHub {
+    fn prs_abiertos(&self, rol: Rol) -> Paso<Vec<PrAbierto>> {
+        if rol == Rol::Revisor {
+            return Err(rol_no_valido(PRS_ABIERTOS, rol));
+        }
+        let (_, ruta) = self.ep(&Endpoint::TodosLosPrsAbiertos);
+        let prs: Vec<PrListaApi> =
+            self.con
+                .leer_paginas(PRS_ABIERTOS, rol, "pull_requests:read", ruta, |c| {
+                    self.con.parsear(PRS_ABIERTOS, c)
+                })?;
+        let mut v = Vec::new();
+        for p in prs {
+            let propio = matches!((&p.head.repo, &p.base.repo), (Some(h), Some(b)) if h.id == b.id);
+            // Una rama que tc2 no habria creado no casa con ninguna tarea.
+            if !propio || !rama_valida(&p.head.nombre) {
+                continue;
+            }
+            if !sha_valido(&p.head.sha) {
+                return Err(self.con.invalida_por(PRS_ABIERTOS, &t::RAMA_BASE_O_SHA));
+            }
+            v.push(PrAbierto {
+                numero: p.number,
+                rama: p.head.nombre,
+                sha: p.head.sha.to_ascii_lowercase(),
+                borrador: p.draft,
+            });
+        }
+        v.sort_by_key(|p| p.numero);
+        Ok(v)
+    }
+
     fn abrir_pr(&self, rama: &str, base: &str, titulo: &str, cuerpo: &str) -> Informe {
         if !rama_valida(rama) || !rama_valida(base) {
             return uso(ABRIR_PR, "rama-invalida", &t::RAMA_NO_VALIDA);
diff --git a/crates/tc2/src/plataforma/gitlab.rs b/crates/tc2/src/plataforma/gitlab.rs
index 91f7587..e34cb33 100644
--- a/crates/tc2/src/plataforma/gitlab.rs
+++ b/crates/tc2/src/plataforma/gitlab.rs
@@ -18,7 +18,8 @@ use super::errores;
 use super::http::{Base, Metodo, codificar};
 use super::puertas::{self, EstadoCheck, Lecturas};
 use super::{
-    Clave, Firma, ModeloPuertas, Paso, Personas, Plataforma, Proveedor, ReglasLeidas, Usuario,
+    Clave, Firma, ModeloPuertas, Paso, Personas, Plataforma, PrAbierto, Proveedor, ReglasLeidas,
+    Usuario,
 };
 use crate::contrato::{Informe, Resultado, Valor, corto};
 use crate::i18n::catalogo::plataforma as t;
@@ -30,6 +31,7 @@ const LEER_PUERTAS: &str = "leer-puertas";
 const LEER_PERMISOS_CI: &str = "leer-permisos-ci";
 const FIRMA_COMMIT: &str = "firma-commit";
 const IDENTIDAD: &str = "identidad";
+const PRS_ABIERTOS: &str = "prs-abiertos";
 
 /// Permiso de lectura que necesita un token de GitLab.
 static PERMISO_LECTURA: &Texto = &t::PERMISO_LECTURA_GITLAB;
@@ -54,6 +56,7 @@ enum Endpoint<'a> {
     Mr(u64),
     MrConDivergencia(u64),
     MrsAbiertos { rama: &'a str, base: &'a str },
+    TodosLosMrsAbiertos,
     CommitsEntre { base: &'a str, rama: &'a str },
     CrearMr,
     Merge(u64),
@@ -91,6 +94,10 @@ impl Endpoint<'_> {
                     codificar(&format!("{base}..{rama}"))
                 ),
             ),
+            Self::TodosLosMrsAbiertos => (
+                Metodo::Get,
+                format!("{p}/merge_requests?state=opened&per_page=100"),
+            ),
             Self::CrearMr => (Metodo::Post, format!("{p}/merge_requests")),
             Self::Merge(n) => (Metodo::Put, format!("{p}/merge_requests/{n}/merge")),
             Self::ReglasAprobacion => (Metodo::Get, format!("{p}/approval_rules?per_page=100")),
@@ -142,6 +149,22 @@ struct MrApi {
     source_branch: String,
 }
 
+/// Un MR de la lista de abiertos. `draft` desde GitLab 14; antes,
+/// `work_in_progress` (que sigue saliendo, obsoleto).
+#[derive(Deserialize)]
+struct MrListaApi {
+    iid: u64,
+    source_branch: String,
+    #[serde(default)]
+    sha: Option<String>,
+    #[serde(default)]
+    draft: bool,
+    #[serde(default)]
+    work_in_progress: bool,
+    source_project_id: u64,
+    target_project_id: u64,
+}
+
 #[derive(Deserialize)]
 struct MrPuertasApi {
     state: String,
@@ -339,6 +362,38 @@ fn estado_check(s: &str) -> EstadoCheck {
 }
 
 impl Plataforma for GitLab {
+    fn prs_abiertos(&self, rol: Rol) -> Paso<Vec<PrAbierto>> {
+        if rol == Rol::Revisor {
+            return Err(rol_no_valido(PRS_ABIERTOS, rol));
+        }
+        let (_, ruta) = self.ep(&Endpoint::TodosLosMrsAbiertos);
+        let mrs: Vec<MrListaApi> =
+            self.con
+                .leer_paginas(PRS_ABIERTOS, rol, PERMISO_LECTURA, ruta, |c| {
+                    self.con.parsear(PRS_ABIERTOS, c)
+                })?;
+        let mut v = Vec::new();
+        for m in mrs {
+            // Los MR desde un fork no cuentan; una rama que tc2 no habria
+            // creado no casa con ninguna tarea.
+            if m.source_project_id != m.target_project_id || !rama_valida(&m.source_branch) {
+                continue;
+            }
+            let sha = m.sha.unwrap_or_default();
+            if !sha_valido(&sha) {
+                return Err(self.con.invalida_por(PRS_ABIERTOS, &t::RAMA_DESTINO_O_SHA));
+            }
+            v.push(PrAbierto {
+                numero: m.iid,
+                rama: m.source_branch,
+                sha: sha.to_ascii_lowercase(),
+                borrador: m.draft || m.work_in_progress,
+            });
+        }
+        v.sort_by_key(|p| p.numero);
+        Ok(v)
+    }
+
     fn abrir_pr(&self, rama: &str, base: &str, titulo: &str, cuerpo: &str) -> Informe {
         if !rama_valida(rama) || !rama_valida(base) {
             return uso(ABRIR_PR, "rama-invalida", &t::RAMA_NO_VALIDA);
diff --git a/crates/tc2/src/plataforma/mod.rs b/crates/tc2/src/plataforma/mod.rs
index a6a8fbb..dd6b340 100644
--- a/crates/tc2/src/plataforma/mod.rs
+++ b/crates/tc2/src/plataforma/mod.rs
@@ -138,9 +138,29 @@ pub struct ReglasLeidas {
     pub reglas_rama: Vec<String>,
 }
 
+/// Un PR (o MR) abierto del propio repo, con lo minimo que necesita `board`
+/// (F3-5, D3 de TASK-032: sin checks ni aprobaciones).
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct PrAbierto {
+    /// Numero del PR (en GitLab, el `iid` del MR).
+    pub numero: u64,
+    /// Rama de origen.
+    pub rama: String,
+    /// Sha de la cabeza (40 hex).
+    pub sha: String,
+    /// Si es borrador.
+    pub borrador: bool,
+}
+
 /// Puerto `Plataforma`: lista cerrada de operaciones (F1-4, F1-5, F1-6,
 /// F1-13). Las escrituras de puertas (`aplicar-puertas`) son de F3-1.
 pub trait Plataforma {
+    /// `prs-abiertos` (agente o persona; adenda de TASK-032 a F1-4 y F1-13):
+    /// los PR abiertos cuya rama de origen es del propio repo (los de forks
+    /// no cuentan), ordenados por numero. Pagina hasta
+    /// [`conexion::MAX_PAGINAS`]; con mas, falla cerrado.
+    fn prs_abiertos(&self, rol: Rol) -> Paso<Vec<PrAbierto>>;
+
     /// `abrir-pr` (agente): abre el PR de `rama` contra `base`, o devuelve
     /// el que ya haya abierto (`sin-cambios`).
     fn abrir_pr(&self, rama: &str, base: &str, titulo: &str, cuerpo: &str) -> Informe;
diff --git a/crates/tc2/src/roadmap/generar.rs b/crates/tc2/src/roadmap/generar.rs
index a0a845f..3af7722 100644
--- a/crates/tc2/src/roadmap/generar.rs
+++ b/crates/tc2/src/roadmap/generar.rs
@@ -12,9 +12,8 @@ use std::fmt::Write as _;
 
 use super::mermaid;
 use crate::grafo::{self, Fase, PLANIFICADA, TERMINADA};
-use crate::identidad::convertir_heredado;
 use crate::plataforma::Personas;
-use crate::situacion::{Ficha, Situacion};
+use crate::situacion::{self, Ficha, Quien, Situacion};
 use crate::tarea::Asignado;
 
 /// Tercera linea del fichero: lo generado y su formato.
@@ -119,15 +118,11 @@ impl<'a> Datos<'a> {
         }
     }
 
+    /// El «quien» comun de [`situacion::quien`], con el heredado como celda.
     fn con_quien(&self, a: Option<&Asignado>) -> String {
-        let Some(a) = a else {
-            return "—".to_owned();
-        };
-        let a = convertir_heredado(a.clone(), self.personas);
-        match (a.login(), a.heredado()) {
-            (Some(l), _) => format!("@{l}"),
-            (None, Some(h)) => celda(h),
-            (None, None) => "—".to_owned(),
+        match situacion::quien(a, self.personas) {
+            Quien::Heredado(h) => celda(&h),
+            q => q.texto(),
         }
     }
 }
diff --git a/crates/tc2/src/roadmap/mod.rs b/crates/tc2/src/roadmap/mod.rs
index 0500e3e..489789d 100644
--- a/crates/tc2/src/roadmap/mod.rs
+++ b/crates/tc2/src/roadmap/mod.rs
@@ -532,14 +532,14 @@ fn adelantar_editada(g: &Git, carpeta: &Path, commit: &str) -> bool {
 }
 
 #[cfg(test)]
-mod tests {
+pub(crate) mod tests {
     use super::*;
     use std::collections::BTreeMap;
 
     use crate::tarea::{Asignado, Cuerpo, Tarea};
 
     /// Estado en memoria.
-    struct EnMemoria(BTreeMap<String, String>);
+    pub(crate) struct EnMemoria(pub(crate) BTreeMap<String, String>);
 
     impl Estado for EnMemoria {
         fn rutas(&self) -> Vec<String> {
@@ -587,7 +587,7 @@ mod tests {
     const ZONA: &str = "<!-- manual:inicio (a mano) -->\r\n## Hitos\r\n\r\n### H2 — Dos\r\nEl motivo.\r\n### H1 — Uno\r\n### H3 — Tres\r\n<!-- manual:fin -->\r\n";
 
     /// Un estado que pasa por cada seccion.
-    fn estado_de_referencia() -> EnMemoria {
+    pub(crate) fn estado_de_referencia() -> EnMemoria {
         let mut m = BTreeMap::new();
         for (id, md) in [
             (
@@ -679,7 +679,7 @@ mod tests {
         EnMemoria(m)
     }
 
-    fn personas() -> Personas {
+    pub(crate) fn personas() -> Personas {
         Personas::leer("personas:\n  - {id: 9, login: \"ana-x\", correos: [\"ana@x.com\"]}\n")
             .unwrap_or_default()
     }
diff --git a/crates/tc2/src/siguiente.rs b/crates/tc2/src/siguiente.rs
index 0cdd6f7..f37e3c8 100644
--- a/crates/tc2/src/siguiente.rs
+++ b/crates/tc2/src/siguiente.rs
@@ -98,6 +98,12 @@ impl Calculo {
     }
 }
 
+/// El informe de `next` sobre `estado` con la foto `commit` (para comparar
+/// con `board` y `roadmap`, TASK-032 C3a).
+pub fn informe_de(estado: &dyn Estado, commit: &str) -> Result<Informe, Fallo> {
+    calcular(estado).map(|c| c.informe(commit))
+}
+
 /// Listas, en marcha e ilegibles de `estado`.
 fn calcular(estado: &dyn Estado) -> Result<Calculo, Fallo> {
     situacion::calcular(estado).map(Calculo)
diff --git a/crates/tc2/src/situacion.rs b/crates/tc2/src/situacion.rs
index b4667b1..2ec13bf 100644
--- a/crates/tc2/src/situacion.rs
+++ b/crates/tc2/src/situacion.rs
@@ -8,6 +8,8 @@
 
 use crate::estado::{Estado, Fallo, es_tarea_md};
 use crate::grafo::{self, Hitos, Nodo};
+use crate::identidad::convertir_heredado;
+use crate::plataforma::Personas;
 use crate::roadmap::zona::{self, Rota};
 use crate::tarea::{Asignado, Documento};
 
@@ -22,23 +24,65 @@ pub struct Ficha {
     pub titulo: String,
     /// Quien la lleva (en formato 1, siempre texto heredado).
     pub asignado: Option<Asignado>,
+    /// Rama de la tarea, si la tiene (la que casa con su PR, F3-5).
+    pub rama: Option<String>,
 }
 
 impl Ficha {
     /// Ficha de un `tarea.md` leido.
     pub fn de(doc: &Documento) -> Self {
-        let (titulo, asignado) = match doc {
-            Documento::Formato1(t, _) => (t.titulo.clone(), t.asignado_a.clone()),
-            Documento::Formato2(t, _) => (t.titulo.clone(), t.asignado_a.clone()),
+        let (titulo, asignado, rama) = match doc {
+            Documento::Formato1(t, _) => {
+                (t.titulo.clone(), t.asignado_a.clone(), Some(t.rama.clone()))
+            }
+            Documento::Formato2(t, _) => (t.titulo.clone(), t.asignado_a.clone(), t.rama.clone()),
         };
         Self {
             nodo: Nodo::de(doc),
             titulo,
             asignado,
+            rama: rama.filter(|r| !r.trim().is_empty()),
         }
     }
 }
 
+/// Quien lleva una tarea, como lo dicen `roadmap`, `board` y `status`
+/// (TASK-032 C3a): un solo calculo para las tres vistas.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum Quien {
+    /// Nadie.
+    Nadie,
+    /// Una cuenta de la plataforma, por su login.
+    Persona(String),
+    /// Texto heredado de la 0.16 que no casa con ninguna persona de la lista.
+    Heredado(String),
+}
+
+impl Quien {
+    /// El texto de la vista: `@login`, el heredado tal cual o `—`.
+    pub fn texto(&self) -> String {
+        match self {
+            Self::Nadie => "—".to_owned(),
+            Self::Persona(l) => format!("@{l}"),
+            Self::Heredado(h) => h.clone(),
+        }
+    }
+}
+
+/// Quien lleva la tarea de `a`, con los heredados convertidos por la lista
+/// de personas de la base (D6 de TASK-031).
+pub fn quien(a: Option<&Asignado>, personas: &Personas) -> Quien {
+    let Some(a) = a else {
+        return Quien::Nadie;
+    };
+    let a = convertir_heredado(a.clone(), personas);
+    match (a.login(), a.heredado()) {
+        (Some(l), _) => Quien::Persona(l.to_owned()),
+        (None, Some(h)) => Quien::Heredado(h.to_owned()),
+        (None, None) => Quien::Nadie,
+    }
+}
+
 /// Todo lo que se calcula sobre un estado.
 #[derive(Debug, Clone, Default)]
 pub struct Situacion {
diff --git a/crates/tc2/src/tablero.rs b/crates/tc2/src/tablero.rs
new file mode 100644
index 0000000..fba2a8f
--- /dev/null
+++ b/crates/tc2/src/tablero.rs
@@ -0,0 +1,1008 @@
+//! `tc2 board` y `tc2 status` (F3-5): el estado del equipo, quien lleva que
+//! y que espera revision.
+//!
+//! Las dos ordenes solo leen. Salen de la misma [`situacion::calcular`] sobre
+//! [`estado::leer`] que `next` y `roadmap` (misma foto y mismo `commit=`), sin
+//! `fetch` (la frescura es de F3-5b) y sin escribir nada.
+//!
+//! - `status [--json]`: una linea de equipo, sin red. Con `--json`, `status
+//!   ok en-diseno=... en-curso=... en-revision=... listas=... commit=...`.
+//! - `board [--sin-plataforma] [--json]`: la vista completa por grupos (en
+//!   diseno, en curso, en revision y listas para empezar), con ID, titulo,
+//!   quien y PR; terminadas y descartadas solo como recuento.
+//!
+//! El texto ajeno (titulos, logins, asignados heredados) se sanea al
+//! imprimirlo (F1-17, decision 8): sin caracteres de control ni de formato y
+//! con longitud acotada.
+
+use std::collections::BTreeMap;
+use std::fmt::Write as _;
+use std::path::Path;
+
+use crate::Salida;
+use crate::contrato::{Informe, Resultado, corto};
+use crate::estado::{self, Fallo};
+use crate::git::Git;
+use crate::grafo::{Fase, TERMINADA};
+use crate::i18n::catalogo::cli as txt_cli;
+use crate::i18n::catalogo::roadmap as txt_roadmap;
+use crate::i18n::catalogo::tablero as txt;
+use crate::i18n::{Idioma, Mensaje};
+use crate::plataforma::{
+    Credenciales, CredencialesEntorno, GitHub, GitLab, Personas, Plataforma, PrAbierto, Proveedor,
+    Rol, detectar,
+};
+use crate::situacion::{self, Ficha, Quien, Situacion};
+
+/// Nombre de `board`.
+pub const BOARD: &str = "board";
+/// Nombre de `status`.
+pub const STATUS: &str = "status";
+
+/// Tareas por grupo en la linea de `status` (las demas, `+N`).
+pub const K_STATUS: usize = 3;
+/// Caracteres como mucho de un titulo impreso.
+const MAX_TITULO: usize = 60;
+/// Caracteres como mucho de un «quien» impreso.
+const MAX_QUIEN: usize = 40;
+
+/// Una tarea de un grupo.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Fila {
+    /// ID.
+    pub id: String,
+    /// Titulo, tal cual (se sanea al imprimir).
+    pub titulo: String,
+    /// Quien la lleva.
+    pub quien: Quien,
+    /// Su rama, si la tiene.
+    pub rama: Option<String>,
+}
+
+/// Lo que se ve del equipo sobre un estado.
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct Tablero {
+    /// Commit de la foto del estado (12 caracteres).
+    pub commit: String,
+    /// En diseno, en el orden de trabajo.
+    pub en_diseno: Vec<Fila>,
+    /// En curso, en el orden de trabajo.
+    pub en_curso: Vec<Fila>,
+    /// En revision, en el orden de trabajo.
+    pub en_revision: Vec<Fila>,
+    /// Listas para empezar, en el orden de `next`.
+    pub listas: Vec<Fila>,
+    /// Cuantas terminadas.
+    pub terminadas: usize,
+    /// Cuantas descartadas.
+    pub descartadas: usize,
+    /// Carpetas con un `tarea.md` que no se entiende.
+    pub ilegibles: Vec<String>,
+}
+
+impl Tablero {
+    /// Los cuatro grupos con su clave de contrato y su titulo.
+    pub fn grupos(&self) -> [(&'static str, &'static str, &[Fila]); 4] {
+        [
+            ("en-diseno", "En diseño", &self.en_diseno),
+            ("en-curso", "En curso", &self.en_curso),
+            ("en-revision", "En revisión", &self.en_revision),
+            ("listas", "Listas para empezar", &self.listas),
+        ]
+    }
+
+    /// Todas las filas, grupo tras grupo.
+    pub fn filas(&self) -> impl Iterator<Item = &Fila> {
+        self.en_diseno
+            .iter()
+            .chain(&self.en_curso)
+            .chain(&self.en_revision)
+            .chain(&self.listas)
+    }
+}
+
+/// El tablero de `s` (funcion pura): las listas de `next` repartidas por
+/// estado, con el «quien» de `roadmap` ([`situacion::quien`]).
+pub fn tablero(s: &Situacion, personas: &Personas, commit: &str) -> Tablero {
+    // Si un ID se repite, vale el primero (como en `roadmap`).
+    let mut fichas: BTreeMap<&str, &Ficha> = BTreeMap::new();
+    for f in &s.fichas {
+        fichas.entry(f.nodo.id.as_str()).or_insert(f);
+    }
+    let fila = |id: &String| -> Option<Fila> {
+        let f = fichas.get(id.as_str())?;
+        Some(Fila {
+            id: id.clone(),
+            titulo: f.titulo.clone(),
+            quien: situacion::quien(f.asignado.as_ref(), personas),
+            rama: f.rama.clone(),
+        })
+    };
+    let mut t = Tablero {
+        commit: commit.to_owned(),
+        ilegibles: s.ilegibles.clone(),
+        ..Tablero::default()
+    };
+    for id in &s.en_marcha {
+        let estado = fichas.get(id.as_str()).and_then(|f| f.nodo.estado());
+        let grupo = match estado {
+            Some("en-diseno") => &mut t.en_diseno,
+            Some("en-curso") => &mut t.en_curso,
+            Some("en-revision") => &mut t.en_revision,
+            _ => continue,
+        };
+        grupo.extend(fila(id));
+    }
+    t.listas = s.listas.iter().filter_map(fila).collect();
+    for f in fichas.values() {
+        match &f.nodo.fase {
+            Fase::Viva(e) if e == TERMINADA => t.terminadas += 1,
+            Fase::Descartada => t.descartadas += 1,
+            _ => {}
+        }
+    }
+    t
+}
+
+/// Texto ajeno listo para imprimir (F1-17, decision 8): sin caracteres de
+/// control ni de formato (bidi, anchura cero), espacios juntados y como
+/// mucho `max` caracteres (con `…` si se corta).
+pub fn sanear(t: &str, max: usize) -> String {
+    let formato = |c: char| matches!(c, '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{FEFF}');
+    let limpio: String = t
+        .chars()
+        .map(|c| if c.is_control() { ' ' } else { c })
+        .filter(|c| !formato(*c))
+        .collect();
+    let limpio = limpio.split_whitespace().collect::<Vec<_>>().join(" ");
+    if limpio.chars().count() <= max {
+        return limpio;
+    }
+    let mut corto: String = limpio.chars().take(max.saturating_sub(1)).collect();
+    corto.push('…');
+    corto
+}
+
+/// El «quien» impreso de una fila.
+fn quien_impreso(q: &Quien) -> String {
+    sanear(&q.texto(), MAX_QUIEN)
+}
+
+/// IDs de unas filas.
+fn ids(v: &[Fila]) -> Vec<String> {
+    v.iter().map(|f| f.id.clone()).collect()
+}
+
+/// La linea de `status` sin `--json`: los grupos con tareas, `K_STATUS`
+/// por grupo como mucho y `+N` con las demas, y el commit.
+pub fn linea_status(t: &Tablero) -> String {
+    let mut partes = Vec::new();
+    for (_, titulo, filas) in t.grupos() {
+        if filas.is_empty() {
+            continue;
+        }
+        let mut v: Vec<String> = filas
+            .iter()
+            .take(K_STATUS)
+            .map(|f| match f.quien {
+                Quien::Nadie => f.id.clone(),
+                ref q => format!("{} {}", f.id, quien_impreso(q)),
+            })
+            .collect();
+        if filas.len() > K_STATUS {
+            v.push(format!("+{}", filas.len() - K_STATUS));
+        }
+        partes.push(format!("{}: {}", titulo.to_lowercase(), v.join(", ")));
+    }
+    if partes.is_empty() {
+        partes.push("nada en marcha ni listo para empezar".to_owned());
+    }
+    partes.push(format!("commit {}", t.commit));
+    partes.join(" · ")
+}
+
+/// El texto de `board`: un apartado por grupo con ID, titulo, quien y PR, y
+/// el recuento de terminadas y descartadas. `pr` da la celda del PR de cada
+/// fila (`None`: sin columna).
+pub fn texto_board(t: &Tablero, plataforma: &str, pr: Option<&dyn Fn(&Fila) -> String>) -> String {
+    let mut o = format!("Tablero · commit {} · plataforma {plataforma}\n", t.commit);
+    for (_, titulo, filas) in t.grupos() {
+        let _ = writeln!(o, "\n{titulo} ({})", filas.len());
+        if filas.is_empty() {
+            o.push_str("  ninguna\n");
+            continue;
+        }
+        let celdas: Vec<[String; 4]> = filas
+            .iter()
+            .map(|f| {
+                [
+                    f.id.clone(),
+                    sanear(&f.titulo, MAX_TITULO),
+                    quien_impreso(&f.quien),
+                    pr.map(|p| p(f)).unwrap_or_default(),
+                ]
+            })
+            .collect();
+        let ancho = |i: usize| {
+            celdas
+                .iter()
+                .map(|c| c[i].chars().count())
+                .max()
+                .unwrap_or(0)
+        };
+        let (a0, a1, a2) = (ancho(0), ancho(1), ancho(2));
+        for c in &celdas {
+            let mut l = format!(
+                "  {}  {}  {}",
+                rellenar(&c[0], a0),
+                rellenar(&c[1], a1),
+                rellenar(&c[2], a2)
+            );
+            if pr.is_some() {
+                l.push_str("  ");
+                l.push_str(&c[3]);
+            }
+            o.push_str(l.trim_end());
+            o.push('\n');
+        }
+    }
+    let _ = writeln!(
+        o,
+        "\nTerminadas: {} · Descartadas: {}",
+        t.terminadas, t.descartadas
+    );
+    if !t.ilegibles.is_empty() {
+        let _ = writeln!(o, "Ilegibles: {}", t.ilegibles.join(", "));
+    }
+    o
+}
+
+fn rellenar(t: &str, ancho: usize) -> String {
+    let n = t.chars().count();
+    format!("{t}{}", " ".repeat(ancho.saturating_sub(n)))
+}
+
+/// Aviso: la lista de personas existe y no se pudo leer.
+const PERSONAS_ILEGIBLES: &str = "personas-ilegibles";
+
+/// El tablero del estado que conoce el clon de `dir`, con los avisos de
+/// personas. Sin rama de estado, `rechazada motivo=sin-rama-de-estado`.
+fn tablero_en(
+    g: &Git,
+    dir: &Path,
+    op: &str,
+) -> Result<(Tablero, Vec<&'static str>, Personas), Informe> {
+    let vista = estado::leer(g, dir).map_err(|mut i| {
+        op.clone_into(&mut i.comando);
+        i
+    })?;
+    let commit = vista.commit().map(corto).unwrap_or_default();
+    let s = situacion::calcular(&vista).map_err(|f| match f {
+        Fallo::Rechazada {
+            motivo,
+            diagnostico,
+        } => Informe::nuevo(op, Resultado::Rechazada)
+            .con("motivo", motivo)
+            .diagnostico(diagnostico),
+        Fallo::Error(d) => Informe::nuevo(op, Resultado::Error).diagnostico(d),
+    })?;
+    let (personas, ilegibles) = crate::roadmap::personas_y_aviso(g, dir);
+    let avisos = if ilegibles {
+        vec![PERSONAS_ILEGIBLES]
+    } else {
+        Vec::new()
+    };
+    Ok((tablero(&s, &personas, &commit), avisos, personas))
+}
+
+/// Lo que se supo de la plataforma: la clave `plataforma=` de `board`.
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
+    /// La de un fallo de `prs-abiertos`.
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
+/// Lo que necesitan las dos ordenes del entorno.
+pub struct Entorno<'a> {
+    /// Carpeta desde la que se lanza.
+    pub dir: &'a Path,
+    /// Puerto Git.
+    pub git: &'a Git,
+    /// Construye el adaptador; solo lo llama `board` (C3c: `status` no).
+    pub conectar: &'a Conectar<'a>,
+}
+
+/// El adaptador de produccion: plataforma por el host de `origin` y la
+/// clave `plataforma` de la configuracion de la base (F1-13, decision 7),
+/// tokens de las variables `TC2_TOKEN_*`. Pregunta con el token de persona
+/// si lo hay (solo con terminal, F1-6 decision 5) y si no con el de agente;
+/// es una eleccion previa, no un reintento con otro rol.
+pub fn conectar_produccion<'a>(
+    g: &'a Git,
+    dir: &'a Path,
+) -> impl Fn(&Personas) -> Result<Conectada, Consulta> + 'a {
+    move |personas: &Personas| {
+        let url = crate::git::estado::url_origin(g, dir, BOARD)
+            .ok()
+            .flatten()
+            .ok_or(Consulta::NoConfigurada)?;
+        let clave =
+            crate::identidad::clave_plataforma(g, dir).map_err(|_| Consulta::NoConfigurada)?;
+        let destino = detectar(&url, clave.as_ref().map(|(p, a)| (*p, a.as_str())))
+            .map_err(|_| Consulta::NoConfigurada)?;
+        let credenciales = CredencialesEntorno;
+        let rol = if credenciales.token(Rol::Persona).is_some() {
+            Rol::Persona
+        } else if credenciales.token(Rol::Agente).is_some() {
+            Rol::Agente
+        } else {
+            return Err(Consulta::SinCredenciales);
+        };
+        let plataforma: Box<dyn Plataforma> = match destino.proveedor {
+            Proveedor::GitHub => Box::new(
+                GitHub::nuevo(
+                    destino.api,
+                    &destino.proyecto,
+                    Box::new(credenciales),
+                    personas.clone(),
+                )
+                .map_err(|_| Consulta::NoConfigurada)?,
+            ),
+            Proveedor::GitLab => Box::new(
+                GitLab::nuevo(
+                    destino.api,
+                    &destino.proyecto,
+                    Box::new(credenciales),
+                    personas.clone(),
+                )
+                .map_err(|_| Consulta::NoConfigurada)?,
+            ),
+        };
+        Ok(Conectada { plataforma, rol })
+    }
+}
+
+/// El PR de cada tarea y los avisos (C3b).
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct Cruce {
+    /// `ID -> (numero, borrador)` de las filas con PR abierto.
+    pub pr: BTreeMap<String, (u64, bool)>,
+    /// En revision sin un PR abierto listo (sin PR o solo en borrador).
+    pub revision_sin_pr: Vec<String>,
+    /// En diseno, en curso o lista con un PR abierto listo (no borrador).
+    pub pr_sin_revision: Vec<String>,
+}
+
+/// Casa cada fila con el PR abierto de su rama (`head.ref == rama`; con
+/// varios, el de numero menor). Una tarea sin rama no casa.
+pub fn cruzar(t: &Tablero, prs: &[PrAbierto]) -> Cruce {
+    let mut c = Cruce::default();
+    for f in t.filas() {
+        let pr = f
+            .rama
+            .as_deref()
+            .and_then(|r| prs.iter().filter(|p| p.rama == r).min_by_key(|p| p.numero));
+        if let Some(p) = pr {
+            c.pr.insert(f.id.clone(), (p.numero, p.borrador));
+        }
+    }
+    let listo = |id: &str| c.pr.get(id).is_some_and(|(_, borrador)| !borrador);
+    c.revision_sin_pr = t
+        .en_revision
+        .iter()
+        .filter(|f| !listo(&f.id))
+        .map(|f| f.id.clone())
+        .collect();
+    c.pr_sin_revision = t
+        .en_diseno
+        .iter()
+        .chain(&t.en_curso)
+        .chain(&t.listas)
+        .filter(|f| listo(&f.id))
+        .map(|f| f.id.clone())
+        .collect();
+    c
+}
+
+/// `status [--json]` (sin `--json`, que quita `lib.rs`). No consulta nunca
+/// la plataforma.
+pub fn orden_status(args: &[String], json: bool, idioma: Idioma, e: &Entorno<'_>) -> Salida {
+    if !args.is_empty() {
+        return Informe::nuevo(STATUS, Resultado::Uso)
+            .con("motivo", "argumentos-de-mas")
+            .diagnostico(&txt::USO_STATUS)
+            .salida_en(json, idioma);
+    }
+    match status_en(e) {
+        Ok((linea, _)) if !json => Salida {
+            codigo: 0,
+            stdout: format!("{linea}\n"),
+            stderr: String::new(),
+        },
+        Ok((_, i)) | Err(i) => i.salida_en(json, idioma),
+    }
+}
+
+/// `status` sobre el estado que conoce el clon: la linea de texto y el
+/// informe.
+pub fn status_en(e: &Entorno<'_>) -> Result<(String, Informe), Informe> {
+    let (t, _, _) = tablero_en(e.git, e.dir, STATUS)?;
+    let mut i = Informe::nuevo(STATUS, Resultado::Ok);
+    for (clave, _, filas) in t.grupos() {
+        i = i.con_lista(clave, ids(filas));
+    }
+    i = i.con("commit", t.commit.clone());
+    if !t.ilegibles.is_empty() {
+        i = i.con_lista("ilegibles", t.ilegibles.clone());
+    }
+    Ok((linea_status(&t), i))
+}
+
+/// Que hace `board` con la plataforma.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Modo {
+    /// La consulta (por defecto).
+    ConPlataforma,
+    /// `--sin-plataforma`: no la consulta.
+    SinPlataforma,
+}
+
+/// `board [--sin-plataforma] [--json]` (sin `--json`, que quita `lib.rs`).
+/// El texto va a stdout y los avisos a stderr con codigo 0.
+pub fn orden_board(args: &[String], json: bool, idioma: Idioma, e: &Entorno<'_>) -> Salida {
+    let modo = match args {
+        [] => Modo::ConPlataforma,
+        [a] if a == "--sin-plataforma" => Modo::SinPlataforma,
+        _ => {
+            return Informe::nuevo(BOARD, Resultado::Uso)
+                .con("motivo", "argumentos")
+                .diagnostico(&txt::USO_BOARD)
+                .salida_en(json, idioma);
+        }
+    };
+    match board_en(e, modo) {
+        Ok((texto, i)) => {
+            let avisos = texto_de_avisos(&i, idioma);
+            let mut s = if json {
+                i.con("texto", texto).salida_en(true, idioma)
+            } else {
+                Salida {
+                    codigo: 0,
+                    stdout: texto,
+                    stderr: String::new(),
+                }
+            };
+            s.stderr.push_str(&avisos);
+            s
+        }
+        Err(i) => i.salida_en(json, idioma),
+    }
+}
+
+/// `board` sobre el estado que conoce el clon: el texto y el informe (sin
+/// `texto=`).
+pub fn board_en(e: &Entorno<'_>, modo: Modo) -> Result<(String, Informe), Informe> {
+    // El estado primero: si no hay rama de estado, no se habla con nadie.
+    let (t, avisos, personas) = tablero_en(e.git, e.dir, BOARD)?;
+    let consulta = match modo {
+        Modo::SinPlataforma => Consulta::NoConsultada,
+        Modo::ConPlataforma => match (e.conectar)(&personas) {
+            Ok(c) => match c.plataforma.prs_abiertos(c.rol) {
+                Ok(prs) => Consulta::Hecha(prs),
+                Err(i) => Consulta::de_fallo(&i),
+            },
+            Err(c) => c,
+        },
+    };
+    Ok(board_de(&t, &consulta, &avisos))
+}
+
+/// Texto e informe de `board` para un tablero y lo que se supo de la
+/// plataforma (funcion pura, C3b y C3c).
+pub fn board_de(t: &Tablero, consulta: &Consulta, avisos: &[&str]) -> (String, Informe) {
+    let cruce = match consulta {
+        Consulta::Hecha(prs) => Some(cruzar(t, prs)),
+        _ => None,
+    };
+    let celda = |f: &Fila| -> String {
+        match cruce.as_ref().and_then(|c| c.pr.get(&f.id)) {
+            Some((n, true)) => format!("#{n} (borrador)"),
+            Some((n, false)) => format!("#{n}"),
+            None => "—".to_owned(),
+        }
+    };
+    let texto = texto_board(
+        t,
+        consulta.nombre(),
+        cruce.as_ref().map(|_| &celda as &dyn Fn(&Fila) -> String),
+    );
+    (
+        texto,
+        informe_board(t, consulta.nombre(), cruce.as_ref(), avisos),
+    )
+}
+
+fn informe_board(t: &Tablero, plataforma: &str, cruce: Option<&Cruce>, avisos: &[&str]) -> Informe {
+    let mut i = Informe::nuevo(BOARD, Resultado::Ok)
+        .con("commit", t.commit.clone())
+        .con("plataforma", plataforma);
+    for (clave, _, filas) in t.grupos() {
+        i = i.con_lista(clave, ids(filas));
+    }
+    let quien: Vec<String> = t
+        .filas()
+        .filter(|f| f.quien != Quien::Nadie)
+        .map(|f| format!("{}:{}", f.id, quien_impreso(&f.quien)))
+        .collect();
+    i = i.con_lista("quien", quien);
+    // Sin consulta hecha no hay pendientes que avisar (C3c).
+    if let Some(c) = cruce {
+        let con_pr: Vec<&Fila> = t.filas().filter(|f| c.pr.contains_key(&f.id)).collect();
+        let prs = con_pr
+            .iter()
+            .filter_map(|f| c.pr.get(&f.id).map(|(n, _)| format!("{}#{n}", f.id)))
+            .collect();
+        i = i.con_lista("prs", prs);
+        let borradores: Vec<String> = con_pr
+            .iter()
+            .filter(|f| c.pr.get(&f.id).is_some_and(|(_, b)| *b))
+            .map(|f| f.id.clone())
+            .collect();
+        for (clave, v) in [
+            ("borradores", borradores),
+            ("revision-sin-pr", c.revision_sin_pr.clone()),
+            ("pr-sin-revision", c.pr_sin_revision.clone()),
+        ] {
+            if !v.is_empty() {
+                i = i.con_lista(clave, v);
+            }
+        }
+    }
+    i = i
+        .con("terminadas", t.terminadas.to_string())
+        .con("descartadas", t.descartadas.to_string());
+    if !t.ilegibles.is_empty() {
+        i = i.con_lista("ilegibles", t.ilegibles.clone());
+    }
+    if !avisos.is_empty() {
+        i = i.con_lista("avisos", avisos.iter().map(|a| (*a).to_owned()).collect());
+    }
+    i
+}
+
+/// Los avisos de un informe de `board`, una linea cada uno, para stderr.
+fn texto_de_avisos(i: &Informe, idioma: Idioma) -> String {
+    let mut s = String::new();
+    let mut linea = |m: Mensaje| {
+        s.push_str(&m.texto(idioma));
+        s.push('\n');
+    };
+    let lista = |clave: &str| -> Vec<String> {
+        i.claves
+            .iter()
+            .find_map(|(k, v)| match v {
+                crate::contrato::Valor::Lista(l) if k == clave => Some(l.clone()),
+                _ => None,
+            })
+            .unwrap_or_default()
+    };
+    let plataforma = match i.valor("plataforma") {
+        Some("no-configurada") => Some(&txt::AVISO_PLATAFORMA_NO_CONFIGURADA),
+        Some("sin-credenciales") => Some(&txt::AVISO_PLATAFORMA_SIN_CREDENCIALES),
+        Some("sin-red") => Some(&txt::AVISO_PLATAFORMA_SIN_RED),
+        Some("error") => Some(&txt::AVISO_PLATAFORMA_ERROR),
+        _ => None,
+    };
+    if let Some(m) = plataforma {
+        linea(Mensaje::nuevo(m));
+    }
+    for (clave, m) in [
+        ("revision-sin-pr", &txt::AVISO_REVISION_SIN_PR),
+        ("pr-sin-revision", &txt::AVISO_PR_SIN_REVISION),
+    ] {
+        let v = lista(clave);
+        if !v.is_empty() {
+            linea(Mensaje::nuevo(m).con("ids", v.join(", ")));
+        }
+    }
+    if lista("avisos").iter().any(|a| a == PERSONAS_ILEGIBLES) {
+        linea(Mensaje::nuevo(&txt_roadmap::AVISO_PERSONAS_ILEGIBLES));
+    }
+    s
+}
+
+/// La carpeta actual, o el error de no poder leerla.
+pub fn carpeta_actual(op: &str) -> Result<std::path::PathBuf, Informe> {
+    std::env::current_dir().map_err(|e| {
+        Informe::nuevo(op, Resultado::Error)
+            .diagnostico(Mensaje::nuevo(&txt_cli::CARPETA_ACTUAL).con("error", e.to_string()))
+    })
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::plataforma::{Firma, ModeloPuertas, Paso, ReglasLeidas, Usuario};
+    use crate::roadmap::tests::{estado_de_referencia, personas};
+
+    const COMMIT: &str = "0123456789ab";
+
+    /// Valor de lista de `clave=` en una linea de contrato.
+    fn lista(linea: &str, clave: &str) -> Vec<String> {
+        linea
+            .split(' ')
+            .find_map(|kv| kv.strip_prefix(clave)?.strip_prefix('='))
+            .unwrap_or_default()
+            .split(',')
+            .filter(|s| !s.is_empty())
+            .map(str::to_owned)
+            .collect()
+    }
+
+    /// La celda «Con quien» de `id` en las tablas de en curso y en revision
+    /// de un ROADMAP.
+    fn quien_del_roadmap(roadmap: &str, id: &str) -> Option<String> {
+        let prefijo = format!("| {id} | ");
+        roadmap
+            .lines()
+            .filter(|l| l.starts_with(&prefijo))
+            // Las tablas de en marcha: | Tarea | Titulo | Con quien | Hito | Depende de |
+            .find(|l| l.split(" | ").nth(3).is_some_and(|h| h.starts_with('H')))
+            .and_then(|l| l.split(" | ").nth(2).map(str::to_owned))
+    }
+
+    /// C3a: sobre el mismo estado, las listas de `board` son las de `next`
+    /// (en diseno + en curso + en revision = `en-marcha`, en su orden;
+    /// `listas` igual), el «quien» es el texto de `roadmap` y `commit=` es el
+    /// mismo.
+    #[test]
+    fn board_next_y_roadmap_dicen_lo_mismo() {
+        let e = estado_de_referencia();
+        let s = situacion::calcular(&e);
+        assert!(s.is_ok(), "{s:?}");
+        let Ok(s) = s else { return };
+        let t = tablero(&s, &personas(), COMMIT);
+        let next = crate::siguiente::informe_de(&e, COMMIT).map(|i| i.linea());
+        assert!(next.is_ok(), "{next:?}");
+        let Ok(next) = next else { return };
+        let board = board_de(&t, &Consulta::NoConsultada, &[]).1.linea();
+        let mut en_marcha: Vec<(usize, String)> = Vec::new();
+        for g in ["en-diseno", "en-curso", "en-revision"] {
+            for id in lista(&board, g) {
+                let pos = lista(&next, "en-marcha").iter().position(|x| *x == id);
+                assert!(pos.is_some(), "{id} de {g} no esta en next: {next}");
+                en_marcha.push((pos.unwrap_or_default(), id));
+            }
+        }
+        // Cada grupo conserva el orden de `next`.
+        for g in ["en-diseno", "en-curso", "en-revision"] {
+            let v = lista(&board, g);
+            let pos: Vec<usize> = v
+                .iter()
+                .filter_map(|id| lista(&next, "en-marcha").iter().position(|x| x == id))
+                .collect();
+            assert!(pos.windows(2).all(|w| w[0] < w[1]), "{g}: {board}");
+        }
+        en_marcha.sort();
+        let ids: Vec<String> = en_marcha.into_iter().map(|(_, id)| id).collect();
+        assert_eq!(ids, lista(&next, "en-marcha"));
+        assert_eq!(lista(&board, "listas"), lista(&next, "listas"));
+        assert!(!lista(&board, "listas").is_empty());
+        assert_eq!(lista(&board, "commit"), lista(&next, "commit"));
+        // El «quien»: el texto de la tabla del ROADMAP.
+        let roadmap = crate::roadmap::de_estado(&e, &personas()).unwrap_or_default();
+        let mut vistos = 0;
+        for f in t.en_diseno.iter().chain(&t.en_curso).chain(&t.en_revision) {
+            assert_eq!(
+                Some(f.quien.texto()),
+                quien_del_roadmap(&roadmap, &f.id),
+                "{}",
+                f.id
+            );
+            vistos += 1;
+        }
+        assert_eq!(vistos, 3, "{board}");
+        // Los tres casos: login, heredado convertido por la lista y heredado
+        // tal cual.
+        assert_eq!(
+            lista(&board, "quien"),
+            [
+                "TASK-008:alguien%20de%20fuera",
+                "TASK-002:@ana",
+                "TASK-003:@ana-x"
+            ]
+        );
+        assert_eq!(
+            (t.terminadas, t.descartadas, t.ilegibles.clone()),
+            (2, 1, vec!["TASK-010".to_owned()])
+        );
+        // `status` da las mismas listas.
+        assert!(linea_status(&t).ends_with(&format!("commit {COMMIT}")));
+    }
+
+    /// Titulos y logins ajenos se sanean al imprimir.
+    #[test]
+    fn el_texto_ajeno_se_sanea() {
+        let fila = |id: &str, titulo: &str, quien: Quien| Fila {
+            id: id.to_owned(),
+            titulo: titulo.to_owned(),
+            quien,
+            rama: None,
+        };
+        let t = Tablero {
+            commit: COMMIT.to_owned(),
+            en_curso: vec![fila(
+                "TASK-001",
+                &format!("rojo \u{1b}[31m{}", "x".repeat(100)),
+                Quien::Heredado("ana\u{202E}\nluis".to_owned()),
+            )],
+            ..Tablero::default()
+        };
+        let texto = texto_board(&t, "no-consultada", None);
+        assert!(!texto.contains('\u{1b}') && !texto.contains('\u{202E}'));
+        assert!(texto.contains("ana luis"), "{texto}");
+        assert!(texto.lines().all(|l| l.chars().count() < 120), "{texto}");
+        let s = linea_status(&t);
+        assert!(!s.contains('\u{1b}') && !s.contains('\n'), "{s}");
+    }
+
+    /// `status` corta en K tareas por grupo con `+N`.
+    #[test]
+    fn status_acota_con_mas_n() {
+        let filas: Vec<Fila> = (1..=5)
+            .map(|n| Fila {
+                id: format!("TASK-00{n}"),
+                titulo: "t".to_owned(),
+                quien: Quien::Nadie,
+                rama: None,
+            })
+            .collect();
+        let t = Tablero {
+            commit: COMMIT.to_owned(),
+            listas: filas,
+            ..Tablero::default()
+        };
+        assert_eq!(
+            linea_status(&t),
+            format!("listas para empezar: TASK-001, TASK-002, TASK-003, +2 · commit {COMMIT}")
+        );
+        assert_eq!(
+            linea_status(&Tablero {
+                commit: COMMIT.to_owned(),
+                ..Tablero::default()
+            }),
+            format!("nada en marcha ni listo para empezar · commit {COMMIT}")
+        );
+    }
+
+    #[test]
+    fn sanear_quita_controles_y_formato_y_acota() {
+        assert_eq!(sanear("a\u{1b}[31mb\tc\n", 60), "a [31mb c");
+        assert_eq!(sanear("x\u{202E}y\u{200B}z", 60), "xyz");
+        assert_eq!(sanear("abcdef", 4), "abc…");
+        assert_eq!(sanear("  dos   espacios ", 60), "dos espacios");
+    }
+
+    /// Plataforma falsa: solo `prs-abiertos`, con la respuesta dada.
+    struct Falsa(Paso<Vec<PrAbierto>>);
+
+    impl Plataforma for Falsa {
+        fn prs_abiertos(&self, _: Rol) -> Paso<Vec<PrAbierto>> {
+            self.0.clone()
+        }
+        fn abrir_pr(&self, _: &str, _: &str, _: &str, _: &str) -> Informe {
+            Informe::nuevo("abrir-pr", Resultado::Error)
+        }
+        fn estado_puertas(&self, _: Rol, _: u64, _: &ModeloPuertas) -> Informe {
+            Informe::nuevo("estado-puertas", Resultado::Error)
+        }
+        fn merge_pr(&self, _: u64, _: &str) -> Informe {
+            Informe::nuevo("merge-pr", Resultado::Error)
+        }
+        fn leer_puertas(&self, _: Rol, _: &str) -> Paso<ReglasLeidas> {
+            Err(Informe::nuevo("leer-puertas", Resultado::Error))
+        }
+        fn leer_permisos_ci(&self) -> Informe {
+            Informe::nuevo("leer-permisos-ci", Resultado::Error)
+        }
+        fn firma_commit(&self, _: &str) -> Paso<Firma> {
+            Err(Informe::nuevo("firma-commit", Resultado::Error))
+        }
+        fn identidad(&self, _: Rol) -> Paso<Usuario> {
+            Err(Informe::nuevo("identidad", Resultado::Error))
+        }
+    }
+
+    fn pr(numero: u64, rama: &str, borrador: bool) -> PrAbierto {
+        PrAbierto {
+            numero,
+            rama: rama.to_owned(),
+            sha: "a".repeat(40),
+            borrador,
+        }
+    }
+
+    fn fila(id: &str, rama: Option<&str>, quien: Quien) -> Fila {
+        Fila {
+            id: id.to_owned(),
+            titulo: format!("Titulo de {id}"),
+            quien,
+            rama: rama.map(str::to_owned),
+        }
+    }
+
+    /// Un tablero con cada caso de cruce.
+    fn de_prueba() -> Tablero {
+        Tablero {
+            commit: COMMIT.to_owned(),
+            en_diseno: vec![fila("TASK-001", Some("feature/a"), Quien::Nadie)],
+            en_curso: vec![fila(
+                "TASK-002",
+                Some("feature/b"),
+                Quien::Persona("ana".into()),
+            )],
+            en_revision: vec![
+                fila("TASK-003", Some("feature/c"), Quien::Persona("luis".into())),
+                fila("TASK-004", Some("feature/d"), Quien::Nadie),
+                fila("TASK-005", Some("feature/e"), Quien::Nadie),
+            ],
+            listas: vec![fila("TASK-006", None, Quien::Nadie)],
+            terminadas: 4,
+            descartadas: 1,
+            ilegibles: Vec::new(),
+        }
+    }
+
+    /// C3b: `board` lista los cuatro grupos con ID, titulo, quien y PR, y
+    /// avisa `revision-sin-pr=` (sin PR o solo en borrador) y
+    /// `pr-sin-revision=` (PR listo de una tarea que no esta en revision).
+    #[test]
+    fn board_cruza_los_pr_y_avisa() {
+        let falsa = Falsa(Ok(vec![
+            pr(3, "feature/c", false),
+            pr(5, "feature/b", true),
+            pr(7, "feature/a", false),
+            pr(9, "feature/e", true),
+            pr(11, "feature/c", false),
+            pr(12, "feature/otra", false),
+        ]));
+        let consulta = match falsa.prs_abiertos(Rol::Agente) {
+            Ok(v) => Consulta::Hecha(v),
+            Err(i) => Consulta::de_fallo(&i),
+        };
+        let t = de_prueba();
+        let (texto, i) = board_de(&t, &consulta, &[]);
+        assert_eq!(
+            i.linea(),
+            "board ok commit=0123456789ab plataforma=ok en-diseno=TASK-001 en-curso=TASK-002 \
+             en-revision=TASK-003,TASK-004,TASK-005 listas=TASK-006 \
+             quien=TASK-002:@ana,TASK-003:@luis \
+             prs=TASK-001#7,TASK-002#5,TASK-003#3,TASK-005#9 borradores=TASK-002,TASK-005 \
+             revision-sin-pr=TASK-004,TASK-005 pr-sin-revision=TASK-001 terminadas=4 descartadas=1"
+        );
+        for esperado in [
+            "Tablero · commit 0123456789ab · plataforma ok\n",
+            "\nEn diseño (1)\n  TASK-001  Titulo de TASK-001  —  #7\n",
+            "\nEn curso (1)\n  TASK-002  Titulo de TASK-002  @ana  #5 (borrador)\n",
+            "  TASK-003  Titulo de TASK-003  @luis  #3\n",
+            "  TASK-004  Titulo de TASK-004  —      —\n",
+            "\nListas para empezar (1)\n  TASK-006  Titulo de TASK-006  —  —\n",
+            "\nTerminadas: 4 · Descartadas: 1\n",
+        ] {
+            assert!(texto.contains(esperado), "falta {esperado:?} en:\n{texto}");
+        }
+        // Los avisos van a stderr, en los dos idiomas.
+        let es = texto_de_avisos(&i, Idioma::Es);
+        assert_eq!(
+            es,
+            "aviso: en revision sin PR abierto listo: TASK-004, TASK-005\n\
+             aviso: con PR abierto listo y sin pasar a revision: TASK-001\n"
+        );
+        assert!(texto_de_avisos(&i, Idioma::En).starts_with("warning: in review"));
+    }
+
+    /// C3c (parte pura): con la plataforma fallando, la parte de estado sale
+    /// entera, `plataforma=` dice por que y no hay pendientes (ni `prs=` ni
+    /// avisos de revision); sin columna de PR en el texto.
+    #[test]
+    fn plataforma_que_falla_deja_el_estado_entero() {
+        let t = de_prueba();
+        let (base, _) = board_de(&t, &Consulta::NoConsultada, &[]);
+        for (fallo, nombre) in [
+            (Informe::nuevo("prs-abiertos", Resultado::Sinred), "sin-red"),
+            (
+                Informe::nuevo("prs-abiertos", Resultado::Rechazada).con("motivo", "identidad"),
+                "sin-credenciales",
+            ),
+            (
+                Informe::nuevo("prs-abiertos", Resultado::Denegada).con("motivo", "credenciales"),
+                "sin-credenciales",
+            ),
+            (
+                Informe::nuevo("prs-abiertos", Resultado::Error)
+                    .con("motivo", "demasiadas-paginas"),
+                "error",
+            ),
+            (
+                Informe::nuevo("prs-abiertos", Resultado::Denegada).con("motivo", "permisos"),
+                "error",
+            ),
+        ] {
+            let falsa = Falsa(Err(fallo));
+            let consulta = match falsa.prs_abiertos(Rol::Agente) {
+                Ok(v) => Consulta::Hecha(v),
+                Err(i) => Consulta::de_fallo(&i),
+            };
+            let (texto, i) = board_de(&t, &consulta, &[]);
+            assert_eq!(i.resultado, Resultado::Ok);
+            assert_eq!(i.valor("plataforma"), Some(nombre));
+            let linea = i.linea();
+            for clave in [
+                "prs=",
+                "borradores=",
+                "revision-sin-pr=",
+                "pr-sin-revision=",
+            ] {
+                assert!(!linea.contains(clave), "{linea}");
+            }
+            assert!(linea.contains(
+                "en-diseno=TASK-001 en-curso=TASK-002 en-revision=TASK-003,TASK-004,TASK-005 listas=TASK-006"
+            ));
+            // El mismo texto que sin consultar, salvo la cabecera.
+            assert_eq!(
+                texto.replace(&format!("plataforma {nombre}"), "plataforma no-consultada"),
+                base
+            );
+            let avisos = texto_de_avisos(&i, Idioma::Es);
+            assert_eq!(avisos.lines().count(), 1, "{avisos}");
+            assert!(avisos.starts_with("aviso: no se"), "{avisos}");
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
tareas/{01-en-diseno => 02-en-curso}/TASK-032/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-032/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-032/tarea.md                                    | 23 ++++++++++++++++++++++-
 3 files changed, 22 insertions(+), 1 deletion(-)
````

## Tests del diff (2 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/prs_abiertos.rs" ":(literal)crates/tc2/tests/tablero.rs"
````

````
crates/tc2/tests/prs_abiertos.rs | 234 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/tablero.rs      | 535 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 2 files changed, 769 insertions(+)
````
