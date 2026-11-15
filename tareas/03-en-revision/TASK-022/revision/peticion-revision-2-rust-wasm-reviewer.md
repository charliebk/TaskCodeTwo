# Peticion de revision — TASK-022 (ronda 2)

- Tarea: TASK-022 — F2-9 Adaptador GitLab
- Rama revisada: feature/task-022-f2-9-adaptador-gitlab
- Rama base: develop
- Commit revisado (HEAD): 7b4697f67ade1f22276dd31d65c166ba74f6224d
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-022 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 304782b8e59e4293ddbdd512d739b3979a0ac971 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (8 hallazgo(s)))

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

- tareas/03-en-revision/TASK-022/tarea.md
- tareas/03-en-revision/TASK-022/planificacion/
- crates/tc2/src/plataforma/gitlab.rs
- crates/tc2/src/plataforma/mod.rs
- crates/tc2/tests/plataforma_gitlab.rs

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 1 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/plataforma/gitlab.rs:564 | informe-revision-1-rust-wasm-reviewer.md |

## Commits a revisar (git log 304782b8e59e4293ddbdd512d739b3979a0ac971..HEAD)

````
7b4697f fix(TASK-022): hallazgos de la revision Rust ronda 1
8e38964 fix(TASK-022): hallazgos de la revision de calidad ronda 1
3d2f141 chore(TASK-022): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 304782b8e59e4293ddbdd512d739b3979a0ac971..HEAD)

````diff
diff --git a/crates/tc2/src/plataforma/gitlab.rs b/crates/tc2/src/plataforma/gitlab.rs
index faa49e8..661d0a8 100644
--- a/crates/tc2/src/plataforma/gitlab.rs
+++ b/crates/tc2/src/plataforma/gitlab.rs
@@ -17,7 +17,9 @@ use super::credenciales::{Credenciales, Rol};
 use super::errores;
 use super::http::{Base, Metodo, codificar};
 use super::puertas::{self, EstadoCheck, Lecturas};
-use super::{Firma, ModeloPuertas, Paso, Personas, Plataforma, Proveedor, ReglasLeidas, Usuario};
+use super::{
+    Clave, Firma, ModeloPuertas, Paso, Personas, Plataforma, Proveedor, ReglasLeidas, Usuario,
+};
 use crate::contrato::{Informe, Resultado, Valor, corto};
 
 const ABRIR_PR: &str = "abrir-pr";
@@ -97,7 +99,7 @@ impl Endpoint<'_> {
             Self::RamasProtegidas => (Metodo::Get, format!("{p}/protected_branches?per_page=100")),
             Self::Estados(sha) => (
                 Metodo::Get,
-                format!("{p}/repository/commits/{sha}/statuses?per_page=100"),
+                format!("{p}/repository/commits/{sha}/statuses?all=true&per_page=100"),
             ),
             Self::Aprobaciones(n) => (Metodo::Get, format!("{p}/merge_requests/{n}/approvals")),
             Self::Firma(sha) => (
@@ -146,11 +148,17 @@ struct MrPuertasApi {
     target_branch: String,
     has_conflicts: bool,
     diverged_commits_count: u64,
+    #[serde(default)]
+    detailed_merge_status: Option<String>,
 }
 
 #[derive(Deserialize)]
 struct NivelApi {
     access_level: u64,
+    #[serde(default)]
+    user_id: Option<u64>,
+    #[serde(default)]
+    group_id: Option<u64>,
 }
 
 #[derive(Deserialize)]
@@ -208,6 +216,8 @@ struct ClaveApi {
 
 #[derive(Deserialize)]
 struct FirmaApi {
+    #[serde(default)]
+    signature_type: String,
     verification_status: String,
     #[serde(default)]
     gpg_key_id: Option<u64>,
@@ -487,9 +497,9 @@ impl Plataforma for GitLab {
         let g01 = protegida.is_some_and(|p| {
             !p.allow_force_push
                 && p.merge_access_levels.len() == 1
-                && p.merge_access_levels
-                    .iter()
-                    .all(|n| n.access_level == MAINTAINER)
+                && p.merge_access_levels.iter().all(|n| {
+                    n.access_level == MAINTAINER && n.user_id.is_none() && n.group_id.is_none()
+                })
         });
         if !g01 {
             return puertas::rechazada_por_config(
@@ -549,7 +559,14 @@ impl Plataforma for GitLab {
             aprobaciones,
             ignoradas: aprobadores.len() - aprobaciones,
             al_dia: m.diverged_commits_count == 0,
-            sin_conflictos: !m.has_conflicts,
+            // `has_conflicts` se calcula en segundo plano: mientras GitLab
+            // comprueba el MR no se da por libre de conflictos (falla
+            // cerrado; la siguiente lectura lo resuelve).
+            sin_conflictos: !m.has_conflicts
+                && !matches!(
+                    m.detailed_merge_status.as_deref(),
+                    Some("checking" | "unchecked" | "preparing" | "conflict")
+                ),
         })
     }
 
@@ -604,6 +621,18 @@ impl Plataforma for GitLab {
             Ok(r) => r,
             Err(i) => return i,
         };
+        if r.estado == 401 {
+            // El token de persona ya se valido con `GET /user`: GitLab da
+            // 401 en el merge cuando la cuenta no puede mergear.
+            return Informe::nuevo(MERGE_PR, Resultado::Denegada)
+                .con("pr", pr.to_string())
+                .con("motivo", "permisos")
+                .con("rol", Rol::Persona.nombre())
+                .diagnostico(format!(
+                    "GitLab no deja mergear a esta cuenta: necesita {PERMISO_MERGE} (GitLab: «{}»)",
+                    errores::mensaje_api(&r)
+                ));
+        }
         if !r.exito() {
             let mut i = self.con.traducir(MERGE_PR, Rol::Persona, PERMISO_MERGE, &r);
             i.claves
@@ -738,7 +767,11 @@ impl Plataforma for GitLab {
             verificada: f.verification_status == "verified",
             motivo: errores::sanear(&f.verification_status),
             committer: None,
-            clave: f.key.map(|k| k.id).or(f.gpg_key_id),
+            clave: match f.signature_type.as_str() {
+                "SSH" => f.key.map(|k| Clave::Ssh(k.id)),
+                "PGP" => f.gpg_key_id.map(Clave::Gpg),
+                _ => None,
+            },
         })
     }
 
diff --git a/crates/tc2/src/plataforma/mod.rs b/crates/tc2/src/plataforma/mod.rs
index 5a136ed..5e02e82 100644
--- a/crates/tc2/src/plataforma/mod.rs
+++ b/crates/tc2/src/plataforma/mod.rs
@@ -68,7 +68,18 @@ impl Proveedor {
 #[derive(Debug, Clone, Default, PartialEq, Eq)]
 pub struct Personas {
     ids: BTreeSet<u64>,
-    claves: BTreeSet<u64>,
+    claves: BTreeSet<Clave>,
+}
+
+/// Clave de firma registrada en la plataforma, con su tipo: los ids de las
+/// claves SSH y GPG de GitLab son espacios distintos y no se comparan entre
+/// si.
+#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
+pub enum Clave {
+    /// Clave SSH (`key.id`).
+    Ssh(u64),
+    /// Clave GPG (`gpg_key_id`).
+    Gpg(u64),
 }
 
 impl Personas {
@@ -80,10 +91,9 @@ impl Personas {
         }
     }
 
-    /// La misma lista con estas claves de firma (ids de clave GPG o SSH de
-    /// la plataforma).
+    /// La misma lista con estas claves de firma.
     #[must_use]
-    pub fn con_claves(mut self, claves: impl IntoIterator<Item = u64>) -> Self {
+    pub fn con_claves(mut self, claves: impl IntoIterator<Item = Clave>) -> Self {
         self.claves = claves.into_iter().collect();
         self
     }
@@ -94,7 +104,7 @@ impl Personas {
     }
 
     /// Si la clave de firma es de una persona de la lista.
-    pub fn contiene_clave(&self, clave: u64) -> bool {
+    pub fn contiene_clave(&self, clave: Clave) -> bool {
         self.claves.contains(&clave)
     }
 }
@@ -140,7 +150,7 @@ pub struct Firma {
     /// La cuenta `committer`, si la plataforma la asocia (GitHub).
     pub committer: Option<Usuario>,
     /// Id de la clave que firmo, si la plataforma lo da (GitLab).
-    pub clave: Option<u64>,
+    pub clave: Option<Clave>,
 }
 
 impl Firma {
````

## Excluido del diff (8 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 304782b8e59e4293ddbdd512d739b3979a0ac971..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-022/evidencia/e2e-gitlab.log                    |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-022/planificacion/brainstorm/peticion-plan-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-022/planificacion/plan-final.md                 |    0
 tareas/03-en-revision/TASK-022/revision/informe-revision-1-code-quality-reviewer.md         |   21 +
 tareas/03-en-revision/TASK-022/revision/informe-revision-1-rust-wasm-reviewer.md            |   21 +
 tareas/03-en-revision/TASK-022/revision/peticion-revision-1-code-quality-reviewer.md        |   98 +++++
 tareas/03-en-revision/TASK-022/revision/peticion-revision-1-rust-wasm-reviewer.md           | 2374 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-022/tarea.md                                    |    6 +-
 8 files changed, 2518 insertions(+), 2 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff 304782b8e59e4293ddbdd512d739b3979a0ac971..HEAD -- ":(literal)crates/tc2/tests/plataforma_gitlab.rs"
````

````
crates/tc2/tests/plataforma_gitlab.rs | 63 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++----
 1 file changed, 59 insertions(+), 4 deletions(-)
````
