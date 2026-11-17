# Peticion de revision — TASK-028 (ronda 1)

- Tarea: TASK-028 — F3-1 tc2 init
- Rama revisada: feature/task-028-f3-1-taskctl-init-setup
- Rama base: develop
- Commit revisado (HEAD): 2c3706b7cb68b2e455377ec33ab0766aae68db39
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-028 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-028/tarea.md
- tareas/03-en-revision/TASK-028/planificacion/
- crates/tc2/src/config/mod.rs
- crates/tc2/src/contrato.rs
- crates/tc2/src/desinstalar/mod.rs
- crates/tc2/src/desinstalar/proyecto.rs
- crates/tc2/src/estado/mod.rs
- crates/tc2/src/estado/montaje.rs
- crates/tc2/src/huella.rs
- crates/tc2/src/i18n/catalogo/cli.rs
- crates/tc2/src/i18n/catalogo/init.rs
- crates/tc2/src/i18n/catalogo/mod.rs
- crates/tc2/src/init/aplicar.rs
- crates/tc2/src/init/inventario.rs
- crates/tc2/src/init/mod.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/plataforma/personas.rs
- crates/tc2/src/plataforma/puertas.rs
- crates/tc2/tests/init.rs

## Commits a revisar (git log develop..HEAD)

````
2c3706b docs(TASK-028): autorrevision con rust-wasm-reviewer y code-quality-reviewer
1713060 fix(TASK-028): orden de rama con historia desde la otra rama, fallo de init simplificado y tests de HEAD en master, historia y fallo al aplicar
07664f6 docs(TASK-028): adenda de F1-5, init propone las puertas y las aplica TASK-118
7f459fb feat(TASK-028): desinstalar reconoce el .gitignore, el workflow y los rulesets de init
128aaec feat(TASK-028): tc2 init con plan como dato, --simular, rama de estado, ramas raiz, config, personas y propuesta de rulesets
cad3998 feat(TASK-028): modelo neutral de puertas de F1-5 y su ruleset de GitHub
fe88f07 refactor(TASK-028): montaje::previsto de solo lectura y montar sobre el
01f9faf refactor(TASK-028): huella de init, linea de accion en contrato y generadores de config y personas
d4b0b7d chore(TASK-028): tarea en curso
````

## Diff de tu dominio (17 fichero(s) de 18; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/config/mod.rs b/crates/tc2/src/config/mod.rs
index 4becc1b..2ef5a98 100644
--- a/crates/tc2/src/config/mod.rs
+++ b/crates/tc2/src/config/mod.rs
@@ -401,6 +401,12 @@ impl Invalida {
     }
 }
 
+/// Texto de la configuracion minima que escribe `tc2 init` (F3-1): solo
+/// `formato: 2`, que [`leer`] da como [`Config::suelo`].
+pub fn texto_minimo() -> &'static str {
+    "---\nformato: 2\n"
+}
+
 /// Lee y valida el texto de `.taskcode/config.yml`. Sin E/S.
 pub fn leer(texto: &str) -> Result<Config, Invalida> {
     let primera = texto.split('\n').next().unwrap_or("");
@@ -530,5 +536,6 @@ mod tests {
     fn config_minima_da_los_valores_por_defecto() {
         assert_eq!(leer("---\nformato: 2\n"), Ok(Config::suelo()));
         assert_eq!(leer("---\r\nformato: 2\r\n"), Ok(Config::suelo()));
+        assert_eq!(leer(texto_minimo()), Ok(Config::suelo()));
     }
 }
diff --git a/crates/tc2/src/contrato.rs b/crates/tc2/src/contrato.rs
index 6f71178..4864435 100644
--- a/crates/tc2/src/contrato.rs
+++ b/crates/tc2/src/contrato.rs
@@ -242,6 +242,61 @@ impl From<Informe> for Salida {
     }
 }
 
+/// Una linea del plan de una orden que trabaja con «plan como dato» (F1-12
+/// PL1: `desinstalar`, `init`): `accion ruta=... clase=... dueno=...
+/// decision=...` y las claves opcionales que haya, en el orden dado. Cada
+/// orden tiene su enum de decisiones; aqui llega ya como texto.
+#[derive(Debug, Clone, Copy)]
+pub struct LineaAccion<'a> {
+    /// Ruta, ref o nombre.
+    pub ruta: &'a str,
+    /// Que es.
+    pub clase: &'a str,
+    /// De quien es.
+    pub dueno: &'a str,
+    /// Que se decide.
+    pub decision: &'a str,
+    /// Claves opcionales (`motivo`, `orden`, `sha`...).
+    pub extra: &'a [(&'a str, Option<&'a str>)],
+}
+
+impl LineaAccion<'_> {
+    /// La linea de texto, con los valores escapados como el contrato.
+    pub fn texto(&self) -> String {
+        let mut s = format!(
+            "accion ruta={} clase={} dueno={} decision={}",
+            escapar_valor(self.ruta, false),
+            self.clase,
+            self.dueno,
+            self.decision
+        );
+        for (k, v) in self.extra {
+            if let Some(v) = v {
+                s.push_str(&format!(" {k}={}", escapar_valor(v, false)));
+            }
+        }
+        s
+    }
+
+    /// La misma linea en JSON, en una linea.
+    pub fn json(&self) -> String {
+        let mut s = format!(
+            "{{\"accion\":{{\"ruta\":{},\"clase\":{},\"dueno\":{},\"decision\":{}",
+            cadena_json(self.ruta),
+            cadena_json(self.clase),
+            cadena_json(self.dueno),
+            cadena_json(self.decision)
+        );
+        for (k, v) in self.extra {
+            if let Some(v) = v {
+                s.push_str(&format!(",{}:{}", cadena_json(k), cadena_json(v)));
+            }
+        }
+        s.push_str("}}");
+        s
+    }
+}
+
 /// Hash abreviado a 12 caracteres, como fija el contrato.
 pub fn corto(hash: &str) -> String {
     hash.chars().take(12).collect()
diff --git a/crates/tc2/src/desinstalar/mod.rs b/crates/tc2/src/desinstalar/mod.rs
index f453cc9..861f638 100644
--- a/crates/tc2/src/desinstalar/mod.rs
+++ b/crates/tc2/src/desinstalar/mod.rs
@@ -25,7 +25,7 @@ pub mod texto;
 use std::path::{Path, PathBuf};
 
 use crate::Salida;
-use crate::contrato::{Informe, Resultado, cadena_json, escapar_valor};
+use crate::contrato::{Informe, LineaAccion, Resultado};
 use crate::git::Git;
 use crate::i18n::catalogo::desinstalar as txt;
 use crate::i18n::{Idioma, Mensaje};
@@ -248,47 +248,29 @@ impl Accion {
         self
     }
 
-    /// Linea `accion ruta=... clase=... dueno=... decision=...`.
+    /// Linea `accion ruta=... clase=... dueno=... decision=...` (formato
+    /// comun de [`LineaAccion`]).
     pub fn linea(&self) -> String {
-        let mut s = format!(
-            "accion ruta={} clase={} dueno={} decision={}",
-            escapar_valor(&self.ruta, false),
-            self.clase,
-            self.dueno,
-            self.decision.texto()
-        );
-        for (k, v) in [
-            ("motivo", &self.motivo),
-            ("orden", &self.orden),
-            ("sha", &self.sha),
-        ] {
-            if let Some(v) = v {
-                s.push_str(&format!(" {k}={}", escapar_valor(v, false)));
-            }
-        }
-        s
+        self.comun(|l| l.texto())
     }
 
     /// La misma linea en JSON.
     pub fn json(&self) -> String {
-        let mut s = format!(
-            "{{\"accion\":{{\"ruta\":{},\"clase\":{},\"dueno\":{},\"decision\":{}",
-            cadena_json(&self.ruta),
-            cadena_json(self.clase),
-            cadena_json(self.dueno),
-            cadena_json(self.decision.texto())
-        );
-        for (k, v) in [
-            ("motivo", &self.motivo),
-            ("orden", &self.orden),
-            ("sha", &self.sha),
-        ] {
-            if let Some(v) = v {
-                s.push_str(&format!(",\"{k}\":{}", cadena_json(v)));
-            }
-        }
-        s.push_str("}}");
-        s
+        self.comun(|l| l.json())
+    }
+
+    fn comun(&self, f: impl Fn(&LineaAccion<'_>) -> String) -> String {
+        f(&LineaAccion {
+            ruta: &self.ruta,
+            clase: self.clase,
+            dueno: self.dueno,
+            decision: self.decision.texto(),
+            extra: &[
+                ("motivo", self.motivo.as_deref()),
+                ("orden", self.orden.as_deref()),
+                ("sha", self.sha.as_deref()),
+            ],
+        })
     }
 
     /// Aviso para la persona por stderr (aunque la orden salga bien): la
diff --git a/crates/tc2/src/desinstalar/proyecto.rs b/crates/tc2/src/desinstalar/proyecto.rs
index 83d7432..236e257 100644
--- a/crates/tc2/src/desinstalar/proyecto.rs
+++ b/crates/tc2/src/desinstalar/proyecto.rs
@@ -9,7 +9,7 @@ use std::time::Duration;
 use super::texto::hooks_con_tc2;
 use super::{Accion, Contexto, Decision, Efecto, NOMBRE, Opciones, leer, t};
 use crate::contrato::{Informe, Resultado};
-use crate::estado::{CARPETA, Cerrojo};
+use crate::estado::Cerrojo;
 use crate::git::estado::dir_comun;
 use crate::git::rama::renombrar;
 use crate::git::{
@@ -20,11 +20,6 @@ use crate::huella::{self, RAMA_ESTADO};
 use crate::i18n::Mensaje;
 use crate::i18n::catalogo::desinstalar as txt;
 
-/// Lineas que la 2.0 escribe en `info/exclude` (`git::rama::excluir`).
-fn lineas_exclude() -> [String; 2] {
-    [format!("/{CARPETA_WORKTREES}/"), format!("/{CARPETA}/")]
-}
-
 /// Si la carpeta `dir` esta dentro de `ruta` (o es ella).
 fn dentro(dir: &Path, ruta: &Path) -> bool {
     match (std::fs::canonicalize(dir), std::fs::canonicalize(ruta)) {
@@ -161,10 +156,10 @@ pub(super) fn plan(op: &Opciones, c: &Contexto<'_>, v: &mut Vec<Accion>) -> Resu
     if let Some(bytes) = leer(&exclude)?
         && let Ok(texto) = String::from_utf8(bytes.clone())
     {
-        let propias = lineas_exclude();
+        let propias = huella::LINEAS_EXCLUDE;
         let nuevo: String = texto
             .split_inclusive('\n')
-            .filter(|l| !propias.iter().any(|p| l.trim() == p))
+            .filter(|l| !propias.iter().any(|p| l.trim() == *p))
             .collect();
         if nuevo != texto {
             v.push(Accion::borra(
@@ -292,6 +287,45 @@ pub(super) fn plan(op: &Opciones, c: &Contexto<'_>, v: &mut Vec<Accion>) -> Resu
                 .motivo("sin-config")
         });
     }
+    // Lo versionado que deja `tc2 init` (F3-1): las lineas de `.gitignore` y
+    // el workflow estan en la historia; los quita la persona con un PR.
+    let gitignore = principal.join(".gitignore");
+    if let Some(bytes) = leer(&gitignore)? {
+        let texto = String::from_utf8_lossy(&bytes);
+        if huella::LINEAS_GITIGNORE
+            .iter()
+            .any(|l| texto.lines().any(|x| x.trim() == *l))
+        {
+            v.push(
+                Accion::nueva(t(&gitignore), "gitignore", "taskcodetwo", Decision::Persona)
+                    .motivo("versionado"),
+            );
+        }
+    }
+    let workflow = principal.join(huella::RUTA_WORKFLOW);
+    if workflow.is_file() {
+        let mut a = Accion::nueva(t(&workflow), "workflow", "taskcodetwo", Decision::Conserva)
+            .motivo("versionado")
+            .orden(format!("git rm {}", huella::RUTA_WORKFLOW));
+        a.decision = Decision::Persona;
+        v.push(a);
+    }
+    // Los rulesets que propone `init` estan en la plataforma: desinstalar no
+    // usa la red ni la toca; si existen, los quita la persona.
+    let config_de_la_2 = config.is_file() && !de_la_016;
+    if config_de_la_2 && crate::git::estado::hay_remoto(g, &principal, NOMBRE)? {
+        for p in crate::plataforma::puertas::modelo() {
+            v.push(
+                Accion::nueva(
+                    p.nombre_completo(),
+                    "ruleset",
+                    "taskcodetwo",
+                    Decision::Persona,
+                )
+                .motivo("si-existe"),
+            );
+        }
+    }
     // Hooks del proyecto que nombran tc2: aviso, nunca se editan (C4).
     for nombre in ["settings.json", "settings.local.json"] {
         let p = principal.join(".claude").join(nombre);
diff --git a/crates/tc2/src/estado/mod.rs b/crates/tc2/src/estado/mod.rs
index a6b78bd..c9d501b 100644
--- a/crates/tc2/src/estado/mod.rs
+++ b/crates/tc2/src/estado/mod.rs
@@ -28,7 +28,7 @@ mod transaccion;
 pub use crate::git::Cambio;
 pub use cerrojo::Cerrojo;
 pub use lectura::{Estado, Vista, leer};
-pub use montaje::{montar, montar_con};
+pub use montaje::{Previsto, montar, montar_con, previsto};
 pub use transaccion::{Catalogo, Contexto, Fallo, Operacion, Politica, Roadmap, transaccion};
 
 /// Rama huerfana del estado (ADR F1-1; nombre en [`crate::huella`]).
diff --git a/crates/tc2/src/estado/montaje.rs b/crates/tc2/src/estado/montaje.rs
index eb907fc..f9b767b 100644
--- a/crates/tc2/src/estado/montaje.rs
+++ b/crates/tc2/src/estado/montaje.rs
@@ -4,8 +4,12 @@
 //! publicado por la misma transaccion que cualquier escritura: no hace falta
 //! `worktree add --orphan` (Git 2.42), y si dos clones la crean a la vez el
 //! segundo recibe non-fast-forward, re-ejecuta, la ve creada y solo la monta.
+//!
+//! [`previsto`] dice, sin escribir nada, que haria [`montar`]; `montar` parte
+//! de esa misma clasificacion, asi que `tc2 init --simular` (F3-1) y la
+//! ejecucion no se pueden separar.
 
-use std::path::Path;
+use std::path::{Path, PathBuf};
 
 use super::cerrojo::{Cerrojo, ErrorCerrojo};
 use super::lectura::Estado;
@@ -60,6 +64,88 @@ fn renombrado(mut i: Informe) -> Informe {
     i
 }
 
+/// Lo que haria [`montar`] ahora mismo.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum Previsto {
+    /// Ya montada en `tareas/`: nada que hacer.
+    Montada,
+    /// La rama local existe y falta montarla.
+    Local,
+    /// La rama solo esta en el remoto: se trae y se monta.
+    DelRemoto,
+    /// No esta en ningun sitio: se crea (y se publica si hay remoto).
+    Crear,
+    /// `tareas/` existe con contenido que no es la rama (un repo de la 0.16).
+    Ocupada,
+    /// La rama esta montada en otra carpeta.
+    OtraRuta(PathBuf),
+    /// No se pudo hablar con el remoto (el detalle de git).
+    SinRed(String),
+    /// El remoto deniega la lectura (credenciales, no existe...).
+    Denegada(FalloRemoto, String),
+}
+
+/// Que haria [`montar`] en el repo de `dir`, sin escribir nada: lee los
+/// worktrees, la carpeta, las ramas y, si hace falta, el remoto con
+/// `ls-remote` (que no escribe refs, a diferencia de `fetch`).
+pub fn previsto(git: &Git, dir: &Path) -> Result<Previsto, Informe> {
+    let principal = localizar_principal(git, dir)?;
+    previsto_en(git, &principal)
+}
+
+fn previsto_en(git: &Git, principal: &Path) -> Result<Previsto, Informe> {
+    let carpeta = principal.join(CARPETA);
+    for w in worktrees(git, principal, OP)?
+        .iter()
+        .filter(|w| w.rama.as_deref() == Some(RAMA))
+    {
+        // Un registro cuya carpeta ya no existe no cuenta: `montar` lo poda.
+        if !w.ruta.exists() {
+            continue;
+        }
+        return Ok(if misma_ruta(&w.ruta, &carpeta) {
+            Previsto::Montada
+        } else {
+            Previsto::OtraRuta(w.ruta.clone())
+        });
+    }
+    let ocupada = std::fs::read_dir(&carpeta)
+        .map(|mut d| d.next().is_some())
+        .unwrap_or(carpeta.exists());
+    if ocupada {
+        return Ok(Previsto::Ocupada);
+    }
+    if commit_de(git, principal, OP, &format!("refs/heads/{RAMA}"))?.is_some() {
+        return Ok(Previsto::Local);
+    }
+    if !hay_remoto(git, principal, OP)? {
+        return Ok(Previsto::Crear);
+    }
+    let s = correr(
+        git,
+        principal,
+        OP,
+        Subcomando::LsRemote,
+        &["--heads", REMOTO, &format!("refs/heads/{RAMA}")],
+    )?;
+    if s.exito() {
+        return Ok(if s.stdout.trim().is_empty() {
+            Previsto::Crear
+        } else {
+            Previsto::DelRemoto
+        });
+    }
+    match clasificar_fallo_remoto(&s.stderr) {
+        FalloRemoto::RamaInexistente => Ok(Previsto::Crear),
+        FalloRemoto::SinRed => Ok(Previsto::SinRed(s.primera_linea_error())),
+        FalloRemoto::Local => Err(crate::git::remoto::error_local(
+            OP,
+            &s.primera_linea_error(),
+        )),
+        otro => Ok(Previsto::Denegada(otro, s.primera_linea_error())),
+    }
+}
+
 /// Crea la rama de estado si no existe (local, en el remoto o en ningun
 /// sitio) y la monta en `tareas/` del worktree principal. Idempotente: ya
 /// montada → `sin-cambios`. Si la carpeta se borro (tambien con
@@ -102,101 +188,87 @@ fn paso(git: &Git, dir: &Path, politica: Politica, puede_crear: bool) -> Result<
         }
     };
 
-    for w in worktrees(git, &principal, OP)?
-        .iter()
-        .filter(|w| w.rama.as_deref() == Some(RAMA))
-    {
-        if !w.ruta.exists() {
-            exigir(git, &principal, OP, Subcomando::Worktree, &["prune"])?;
-        } else if misma_ruta(&w.ruta, &carpeta) {
+    let previsto = previsto_en(git, &principal)?;
+    match &previsto {
+        Previsto::Montada => {
             excluir(git, &principal, OP, &format!("/{CARPETA}/"))?;
             return Ok(ok(Resultado::SinCambios, None));
-        } else {
+        }
+        Previsto::OtraRuta(ruta) => {
             return Ok(Informe::nuevo(OP, Resultado::Rechazada)
                 .con("rama", RAMA)
                 .con("motivo", "montada-en-otra-ruta")
                 .diagnostico(
                     Mensaje::nuevo(&txt::MONTADA_EN_OTRA_RUTA)
                         .con("rama", RAMA)
-                        .con("ruta", w.ruta.display().to_string()),
+                        .con("ruta", ruta.display().to_string()),
                 ));
         }
+        Previsto::Ocupada => {
+            return Ok(Informe::nuevo(OP, Resultado::Rechazada)
+                .con("ruta", CARPETA)
+                .con("motivo", "tareas-ocupada")
+                .diagnostico(Mensaje::nuevo(&txt::CARPETA_OCUPADA).con("carpeta", CARPETA)));
+        }
+        Previsto::SinRed(detalle) => return Ok(sin_red(detalle)),
+        Previsto::Denegada(f, detalle) => return Ok(denegada(*f, detalle)),
+        Previsto::Local | Previsto::DelRemoto | Previsto::Crear => {}
     }
-    let ocupada = std::fs::read_dir(&carpeta)
-        .map(|mut d| d.next().is_some())
-        .unwrap_or(carpeta.exists());
-    if ocupada {
-        return Ok(Informe::nuevo(OP, Resultado::Rechazada)
-            .con("ruta", CARPETA)
-            .con("motivo", "tareas-ocupada")
-            .diagnostico(Mensaje::nuevo(&txt::CARPETA_OCUPADA).con("carpeta", CARPETA)));
+    // Un registro de la rama cuya carpeta se borro impide volver a montarla.
+    if worktrees(git, &principal, OP)?
+        .iter()
+        .any(|w| w.rama.as_deref() == Some(RAMA) && !w.ruta.exists())
+    {
+        exigir(git, &principal, OP, Subcomando::Worktree, &["prune"])?;
     }
 
     let remoto = hay_remoto(git, &principal, OP)?;
     let local = format!("refs/heads/{RAMA}");
     let remota = format!("refs/remotes/{REMOTO}/{RAMA}");
 
-    if commit_de(git, &principal, OP, &local)?.is_none() {
-        let mut en_remoto = false;
-        if remoto {
-            let refspec = format!("refs/heads/{RAMA}:{remota}");
-            let s = correr(
-                git,
-                &principal,
-                OP,
-                Subcomando::Fetch,
-                &["-q", REMOTO, &refspec],
-            )?;
-            if s.exito() {
-                en_remoto = true;
-            } else {
-                match clasificar_fallo_remoto(&s.stderr) {
-                    FalloRemoto::RamaInexistente => {}
-                    FalloRemoto::SinRed => {
-                        return Ok(Informe::nuevo(OP, Resultado::Sinred)
-                            .con("pendiente", "no")
-                            .diagnostico(
-                                Mensaje::nuevo(&txt::MONTAR_EXIGE_RED)
-                                    .con("error", s.primera_linea_error()),
-                            ));
-                    }
-                    FalloRemoto::Local => {
-                        return Err(crate::git::remoto::error_local(
-                            OP,
-                            &s.primera_linea_error(),
-                        ));
-                    }
-                    otro => {
-                        return Ok(Informe::nuevo(OP, Resultado::Denegada)
-                            .con("motivo", otro.motivo())
-                            .diagnostico(
-                                Mensaje::nuevo(&txt::MONTAR_DENEGADA)
-                                    .con("error", s.primera_linea_error()),
-                            ));
-                    }
-                }
-            }
-        }
-        if en_remoto {
+    let mut crear = previsto == Previsto::Crear;
+    if previsto == Previsto::DelRemoto {
+        let refspec = format!("refs/heads/{RAMA}:{remota}");
+        let s = correr(
+            git,
+            &principal,
+            OP,
+            Subcomando::Fetch,
+            &["-q", REMOTO, &refspec],
+        )?;
+        if s.exito() {
             exigir(git, &principal, OP, Subcomando::Branch, &[RAMA, &remota])?;
         } else {
-            if !puede_crear {
-                return Err(Informe::nuevo(OP, Resultado::Error)
-                    .diagnostico(Mensaje::nuevo(&txt::RAMA_SIGUE_SIN_EXISTIR).con("rama", RAMA)));
-            }
-            // La transaccion toma el cerrojo ella misma: se suelta antes y,
-            // creada la rama, se vuelve a empezar para montarla.
-            drop(cerrojo);
-            let mut c = Contexto::nuevo(git, &principal, &ninguna);
-            c.politica = politica;
-            let i = transaccion(&c, &Inicializar);
-            if !i.resultado.es_exito() {
-                return Ok(renombrado(i));
+            match clasificar_fallo_remoto(&s.stderr) {
+                // Desaparecio del remoto entre la lectura y el fetch.
+                FalloRemoto::RamaInexistente => crear = true,
+                FalloRemoto::SinRed => return Ok(sin_red(&s.primera_linea_error())),
+                FalloRemoto::Local => {
+                    return Err(crate::git::remoto::error_local(
+                        OP,
+                        &s.primera_linea_error(),
+                    ));
+                }
+                otro => return Ok(denegada(otro, &s.primera_linea_error())),
             }
-
-            return paso(git, dir, politica, false);
         }
     }
+    if crear {
+        if !puede_crear {
+            return Err(Informe::nuevo(OP, Resultado::Error)
+                .diagnostico(Mensaje::nuevo(&txt::RAMA_SIGUE_SIN_EXISTIR).con("rama", RAMA)));
+        }
+        // La transaccion toma el cerrojo ella misma: se suelta antes y,
+        // creada la rama, se vuelve a empezar para montarla.
+        drop(cerrojo);
+        let mut c = Contexto::nuevo(git, &principal, &ninguna);
+        c.politica = politica;
+        let i = transaccion(&c, &Inicializar);
+        if !i.resultado.es_exito() {
+            return Ok(renombrado(i));
+        }
+        return paso(git, dir, politica, false);
+    }
 
     let ruta = carpeta.to_str().ok_or_else(|| {
         Informe::nuevo(OP, Resultado::Error).diagnostico(&txt::RUTA_TAREAS_NO_UTF8)
@@ -227,3 +299,15 @@ fn paso(git: &Git, dir: &Path, politica: Politica, puede_crear: bool) -> Result<
     let head = commit_de(git, &carpeta, OP, "HEAD")?;
     Ok(ok(Resultado::Ok, head))
 }
+
+fn sin_red(detalle: &str) -> Informe {
+    Informe::nuevo(OP, Resultado::Sinred)
+        .con("pendiente", "no")
+        .diagnostico(Mensaje::nuevo(&txt::MONTAR_EXIGE_RED).con("error", detalle))
+}
+
+fn denegada(f: FalloRemoto, detalle: &str) -> Informe {
+    Informe::nuevo(OP, Resultado::Denegada)
+        .con("motivo", f.motivo())
+        .diagnostico(Mensaje::nuevo(&txt::MONTAR_DENEGADA).con("error", detalle))
+}
diff --git a/crates/tc2/src/huella.rs b/crates/tc2/src/huella.rs
index 1f9e646..e510fa7 100644
--- a/crates/tc2/src/huella.rs
+++ b/crates/tc2/src/huella.rs
@@ -56,6 +56,23 @@ pub const FICHERO_IDENTIDAD: &str = "identidad.yml";
 /// Indices temporales de la transaccion, en [`CARPETA_COMUN`].
 pub const CARPETA_TEMPORALES: &str = "estado-tmp";
 
+/// Workflow de CI que deja `tc2 init` (F3-1). Hasta que F5-1 entregue el
+/// reutilizable, `init` solo deja la ranura (`pendiente motivo=sin-plantilla`).
+pub const RUTA_WORKFLOW: &str = concat!(".github/workflows/", nombre!(), ".yml");
+
+/// Lineas que `tc2 init` anade a `.gitignore` (F1-3 encargaba a F3-1 ignorar
+/// la carpeta montada y la propia): versionadas, se anaden y nunca se quitan
+/// solas.
+pub const LINEAS_GITIGNORE: [&str; 2] = ["/tareas/", concat!("/.", nombre!(), "/")];
+
+/// Lineas que la 2.0 escribe en `info/exclude` (`git::rama::excluir`): los
+/// worktrees de tarea y la rama de estado montada.
+pub const LINEAS_EXCLUDE: [&str; 2] = [concat!("/.", nombre!(), "/worktrees/"), "/tareas/"];
+
+/// Prefijo de los rulesets que propone `tc2 init` (F1-12 D1a; F1-5 los
+/// llamaba `taskcode-*`).
+pub const PREFIJO_RULESETS: &str = concat!(nombre!(), "-");
+
 /// Texto del `LEEME.md` del commit raiz de la rama de estado.
 pub(crate) const TEXTO_LEEME: &str = concat!(
     "# Rama de estado de TaskCodeTwo\n\nEsta rama (`",
@@ -87,6 +104,19 @@ mod tests {
         assert_eq!(CARPETA_COMUN, NOMBRE);
         assert_eq!(CARPETA_WORKTREES, format!(".{NOMBRE}/worktrees"));
         assert!(TEXTO_LEEME.contains(&format!("(`{RAMA_ESTADO}`)")));
+        assert_eq!(RUTA_WORKFLOW, format!(".github/workflows/{NOMBRE}.yml"));
+        assert_eq!(PREFIJO_RULESETS, format!("{NOMBRE}-"));
+    }
+
+    /// Las lineas de `exclude` y `.gitignore` nombran las carpetas reales.
+    #[test]
+    fn lineas_de_exclude_y_gitignore() {
+        let tareas = format!("/{}/", crate::estado::CARPETA);
+        assert_eq!(
+            LINEAS_EXCLUDE,
+            [format!("/{CARPETA_WORKTREES}/"), tareas.clone()]
+        );
+        assert_eq!(LINEAS_GITIGNORE, [tareas, format!("/.{NOMBRE}/")]);
     }
 
     /// C5: las refs propias viven bajo `refs/taskcodetwo/`.
diff --git a/crates/tc2/src/i18n/catalogo/cli.rs b/crates/tc2/src/i18n/catalogo/cli.rs
index dd8f6f6..6016b01 100644
--- a/crates/tc2/src/i18n/catalogo/cli.rs
+++ b/crates/tc2/src/i18n/catalogo/cli.rs
@@ -10,8 +10,8 @@ crate::i18n::textos! {
         en: "tc2: could not write the output: {error}",
     }
     USO {
-        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
-        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
+        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
+        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
     }
     FALTA_LA_ORDEN {
         es: "tc2: falta la orden. {uso}",
diff --git a/crates/tc2/src/i18n/catalogo/init.rs b/crates/tc2/src/i18n/catalogo/init.rs
new file mode 100644
index 0000000..a9ce500
--- /dev/null
+++ b/crates/tc2/src/i18n/catalogo/init.rs
@@ -0,0 +1,48 @@
+//! Mensajes de `tc2 init` (F3-1).
+
+crate::i18n::textos! {
+    USO_INIT {
+        es: "uso: init [--simular] [--json]",
+        en: "usage: init [--simular] [--json]",
+    }
+    PROYECTO_016 {
+        es: "{ruta} es de TaskCode 0.16: init no lo pisa. No se ha tocado nada; para pasar el proyecto a la 2.0: {arreglo}",
+        en: "{ruta} belongs to TaskCode 0.16: init does not overwrite it. Nothing was touched; to move the project to 2.0: {arreglo}",
+    }
+    CONFIG_NO_VALE {
+        es: "{ruta} existe y no vale ({motivo}): init no la pisa. No se ha tocado nada; arreglala (tc2 config la valida) y repite",
+        en: "{ruta} exists and is not valid ({motivo}): init does not overwrite it. Nothing was touched; fix it (tc2 config validates it) and try again",
+    }
+    PERSONAS_NO_VALE {
+        es: "{ruta} existe y no vale: init no la pisa. No se ha tocado nada; arreglala y repite",
+        en: "{ruta} exists and is not valid: init does not overwrite it. Nothing was touched; fix it and try again",
+    }
+    BLOQUEADA {
+        es: "no se ha tocado nada: {ruta} ({motivo}). Resuelvelo y repite tc2 init",
+        en: "nothing was touched: {ruta} ({motivo}). Resolve it and run tc2 init again",
+    }
+    SIN_RED {
+        es: "init necesita hablar con origin y no pudo: {error}. No se ha tocado nada: repite con red",
+        en: "init needs to talk to origin and could not: {error}. Nothing was touched: try again with a network",
+    }
+    DENEGADA {
+        es: "origin no deja leer ({motivo}): {error}. No se ha tocado nada: revisa la URL y las credenciales y repite",
+        en: "origin does not allow reading ({motivo}): {error}. Nothing was touched: check the URL and the credentials and try again",
+    }
+    FALLO_AL_APLICAR {
+        es: "no se pudo crear {ruta}: {error}. Lo de las lineas anteriores ya esta hecho: arregla la causa y repite tc2 init",
+        en: "could not create {ruta}: {error}. What the previous lines list is already done: fix the cause and run tc2 init again",
+    }
+    NO_SE_LEE {
+        es: "no se puede leer {ruta}: {error}",
+        en: "cannot read {ruta}: {error}",
+    }
+    CAMBIO_DESDE_EL_PLAN {
+        es: "{ruta} cambio despues de hacer el plan",
+        en: "{ruta} changed after the plan was made",
+    }
+    AVISO_PERSONA {
+        es: "{n} acciones las haces tu (decision=persona): cada linea trae la orden",
+        en: "{n} actions are yours to do (decision=persona): each line has the command",
+    }
+}
diff --git a/crates/tc2/src/i18n/catalogo/mod.rs b/crates/tc2/src/i18n/catalogo/mod.rs
index d4e49fa..f8f8c74 100644
--- a/crates/tc2/src/i18n/catalogo/mod.rs
+++ b/crates/tc2/src/i18n/catalogo/mod.rs
@@ -9,6 +9,7 @@ pub mod desinstalar;
 pub mod estado;
 pub mod git;
 pub mod identidad;
+pub mod init;
 pub mod plataforma;
 
 use super::Texto;
@@ -24,6 +25,7 @@ pub fn todos() -> impl Iterator<Item = &'static Texto> {
         estado::TODOS,
         git::TODOS,
         identidad::TODOS,
+        init::TODOS,
         plataforma::TODOS,
     ]
     .into_iter()
diff --git a/crates/tc2/src/init/aplicar.rs b/crates/tc2/src/init/aplicar.rs
new file mode 100644
index 0000000..4a5abb7
--- /dev/null
+++ b/crates/tc2/src/init/aplicar.rs
@@ -0,0 +1,158 @@
+//! Aplicacion de las acciones `crea` del plan de `tc2 init`. Cada efecto
+//! vuelve a comprobar lo que lee: si algo cambio desde el plan, falla sin
+//! pisarlo.
+
+use std::io::Write;
+use std::path::Path;
+
+use super::{Contexto, Efecto, NOMBRE};
+use crate::contrato::{Informe, Resultado};
+use crate::git::estado::construir_commit;
+use crate::git::remoto::publicar;
+use crate::git::{Cambio, Git, Subcomando, exigir};
+use crate::huella::LINEAS_EXCLUDE;
+use crate::i18n::Idioma;
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::init as txt;
+
+/// Mensaje del commit raiz de `main` y `develop`.
+const MENSAJE_RAIZ: &str = "chore: inicia TaskCodeTwo (tc2 init)";
+
+/// Texto de un informe para la linea de fallo.
+fn texto(i: &Informe) -> String {
+    i.diagnostico_en(Idioma::Es).unwrap_or_else(|| i.linea())
+}
+
+/// Aplica un efecto. `Err` con el texto del fallo.
+pub(super) fn aplicar(e: &Efecto, c: &Contexto<'_>) -> Result<(), String> {
+    match e {
+        Efecto::Ninguno => Ok(()),
+        Efecto::Escribir { ruta, contenido } => escribir_nuevo(ruta, contenido),
+        Efecto::Anadir { ruta, lineas } => anadir(ruta, lineas),
+        Efecto::Excluir { principal } => {
+            for l in LINEAS_EXCLUDE {
+                crate::git::rama::excluir(c.git, principal, NOMBRE, l).map_err(|i| texto(&i))?;
+            }
+            Ok(())
+        }
+        Efecto::RamasRaiz {
+            principal,
+            ficheros,
+            ramas,
+            head,
+            publicar,
+        } => ramas_raiz(c.git, principal, ficheros, ramas, *head, *publicar).map_err(|i| texto(&i)),
+        Efecto::MontarEstado { principal } => {
+            let i = crate::estado::montar(c.git, principal);
+            if i.resultado.es_exito() {
+                Ok(())
+            } else {
+                Err(texto(&i))
+            }
+        }
+    }
+}
+
+/// Crea `ruta` con `contenido`; si ya existe (la creo alguien despues del
+/// plan), falla sin tocarla.
+fn escribir_nuevo(ruta: &Path, contenido: &str) -> Result<(), String> {
+    if let Some(padre) = ruta.parent() {
+        std::fs::create_dir_all(padre).map_err(|e| e.to_string())?;
+    }
+    let mut f = std::fs::OpenOptions::new()
+        .write(true)
+        .create_new(true)
+        .open(ruta)
+        .map_err(|e| {
+            if e.kind() == std::io::ErrorKind::AlreadyExists {
+                Mensaje::nuevo(&txt::CAMBIO_DESDE_EL_PLAN)
+                    .con("ruta", ruta.display().to_string())
+                    .texto(Idioma::Es)
+            } else {
+                e.to_string()
+            }
+        })?;
+    f.write_all(contenido.as_bytes()).map_err(|e| e.to_string())
+}
+
+/// Anade a `ruta` las `lineas` que le falten, sin tocar las demas (y con
+/// el fin de linea que ya use el fichero).
+fn anadir(ruta: &Path, lineas: &[String]) -> Result<(), String> {
+    let actual = match std::fs::read_to_string(ruta) {
+        Ok(t) => t,
+        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
+        Err(e) => return Err(e.to_string()),
+    };
+    let fin = if actual.contains("\r\n") {
+        "\r\n"
+    } else {
+        "\n"
+    };
+    let mut nuevo = actual.clone();
+    for l in lineas {
+        if actual.lines().any(|x| x.trim() == l) {
+            continue;
+        }
+        if !nuevo.is_empty() && !nuevo.ends_with('\n') {
+            nuevo.push_str(fin);
+        }
+        nuevo.push_str(l);
+        nuevo.push_str(fin);
+    }
+    if nuevo == actual {
+        return Ok(());
+    }
+    std::fs::write(ruta, nuevo).map_err(|e| e.to_string())
+}
+
+/// Commit raiz con `ficheros` (tal como estan en el disco) y `ramas` en el.
+/// Si HEAD estaba sin nacer en una de ellas, el indice se pone al dia con
+/// esos ficheros (si no, `git status` los daria por borrados). Con remoto,
+/// se publican sin force.
+fn ramas_raiz(
+    git: &Git,
+    principal: &Path,
+    ficheros: &[String],
+    ramas: &[String],
+    head: bool,
+    con_remoto: bool,
+) -> Result<(), Informe> {
+    let mut cambios = Vec::with_capacity(ficheros.len());
+    for f in ficheros {
+        let ruta = principal.join(f);
+        let bytes = std::fs::read(&ruta).map_err(|e| super::no_se_lee(&ruta, &e))?;
+        cambios.push(Cambio::nuevo(
+            f.as_str(),
+            String::from_utf8_lossy(&bytes).into_owned(),
+        ));
+    }
+    let sha = construir_commit(git, principal, NOMBRE, None, &cambios, MENSAJE_RAIZ)?
+        .ok_or_else(|| Informe::nuevo(NOMBRE, Resultado::Error))?;
+    for r in ramas {
+        exigir(git, principal, NOMBRE, Subcomando::Branch, &[r, &sha])?;
+    }
+    if head {
+        let mut args = vec!["--"];
+        args.extend(ficheros.iter().map(String::as_str));
+        exigir(git, principal, NOMBRE, Subcomando::Add, &args)?;
+    }
+    if con_remoto {
+        for r in ramas {
+            let p = publicar(
+                git,
+                principal,
+                NOMBRE,
+                &format!("refs/heads/{r}:refs/heads/{r}"),
+                &[],
+            )?;
+            if !p.resultado.es_exito() {
+                return Err(Informe::nuevo(NOMBRE, p.resultado).diagnostico(
+                    Mensaje::nuevo(&txt::FALLO_AL_APLICAR)
+                        .con("ruta", format!("refs/heads/{r}"))
+                        .con("error", p.ultimo),
+                ));
+            }
+        }
+    }
+    Ok(())
+}
diff --git a/crates/tc2/src/init/inventario.rs b/crates/tc2/src/init/inventario.rs
new file mode 100644
index 0000000..a21e763
--- /dev/null
+++ b/crates/tc2/src/init/inventario.rs
@@ -0,0 +1,450 @@
+//! Inventario de `tc2 init`: de solo lectura (ficheros, refs, worktrees,
+//! `ls-remote` y lecturas `GET` de la plataforma), calcula el plan.
+
+use std::path::Path;
+
+use super::{ARREGLO_016, Accion, Contexto, Decision, Efecto, NOMBRE, ORDEN_PUERTAS, leer, t};
+use crate::config::{self, Motivo};
+use crate::contrato::{Informe, Resultado};
+use crate::estado::{self, CARPETA, Previsto};
+use crate::git::estado::{dir_comun, hay_remoto, url_origin};
+use crate::git::rama::renombrar;
+use crate::git::{
+    FalloRemoto, REMOTO, Subcomando, clasificar_fallo_remoto, commit_de, correr, exigir,
+    localizar_principal, worktrees,
+};
+use crate::huella::{LINEAS_EXCLUDE, LINEAS_GITIGNORE, RUTA_WORKFLOW};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::init as txt;
+use crate::plataforma::personas::{self, Personas};
+use crate::plataforma::puertas::{modelo, ruleset_github};
+use crate::plataforma::{GitHub, GitLab, Plataforma, Proveedor, Rol, detectar};
+
+/// El adaptador de la plataforma y su proveedor, o el motivo por el que no
+/// lo hay (`sin-remoto`, `sin-plataforma`, `sin-token`).
+type Conexion = Result<(Box<dyn Plataforma>, Proveedor), &'static str>;
+
+/// Ramas de codigo que `init` deja (F1-3).
+const RAMAS: [&str; 2] = ["main", "develop"];
+
+/// Lineas de `lineas` que no estan en `texto` (comparadas sin espacios).
+fn faltan(texto: &str, lineas: &[&str]) -> Vec<String> {
+    lineas
+        .iter()
+        .filter(|l| !texto.lines().any(|x| x.trim() == **l))
+        .map(|l| (*l).to_owned())
+        .collect()
+}
+
+fn sinred(detalle: &str) -> Informe {
+    Informe::nuevo(NOMBRE, Resultado::Sinred)
+        .con("pendiente", "no")
+        .diagnostico(Mensaje::nuevo(&txt::SIN_RED).con("error", detalle))
+}
+
+fn denegada(f: FalloRemoto, detalle: &str) -> Informe {
+    Informe::nuevo(NOMBRE, Resultado::Denegada)
+        .con("motivo", f.motivo())
+        .diagnostico(
+            Mensaje::nuevo(&txt::DENEGADA)
+                .con("motivo", f.motivo())
+                .con("error", detalle),
+        )
+}
+
+pub(super) fn plan(c: &Contexto<'_>) -> Result<Vec<Accion>, Informe> {
+    let g = c.git;
+    let principal = localizar_principal(g, c.dir).map_err(|i| renombrar(i, NOMBRE))?;
+    let mut v = Vec::new();
+
+    // Configuracion: la 0.16 o una que no vale no se pisan (C4).
+    let ruta_config = principal.join(config::RUTA);
+    let mut clave: Option<(&'static str, String)> = None;
+    match leer(&ruta_config)? {
+        None => v.push(Accion::crea(
+            t(&ruta_config),
+            "config",
+            Efecto::Escribir {
+                ruta: ruta_config.clone(),
+                contenido: config::texto_minimo().to_owned(),
+            },
+        )),
+        Some(bytes) => match config::leer(&String::from_utf8_lossy(&bytes)) {
+            Ok(cfg) => {
+                clave = cfg.plataforma.map(|p| (p.proveedor.nombre(), p.api));
+                v.push(Accion::nueva(
+                    t(&ruta_config),
+                    "config",
+                    "taskcodetwo",
+                    Decision::Existe,
+                ));
+            }
+            Err(e) if e.motivo == Motivo::Formato => {
+                v.push(rechazo_016(t(&ruta_config), "config"));
+            }
+            Err(e) => {
+                let mut a = Accion::nueva(t(&ruta_config), "config", "ajeno", Decision::Rechaza)
+                    .motivo(e.motivo.nombre());
+                a.diagnostico = Some(
+                    Mensaje::nuevo(&txt::CONFIG_NO_VALE)
+                        .con("ruta", config::RUTA)
+                        .con("motivo", e.motivo.nombre()),
+                );
+                v.push(a);
+            }
+        },
+    }
+
+    // `.gitignore`: solo se anaden lineas.
+    let gitignore = principal.join(".gitignore");
+    let texto = leer(&gitignore)?
+        .map(|b| String::from_utf8_lossy(&b).into_owned())
+        .unwrap_or_default();
+    let lineas = faltan(&texto, &LINEAS_GITIGNORE);
+    v.push(if lineas.is_empty() {
+        Accion::nueva(t(&gitignore), "gitignore", "taskcodetwo", Decision::Existe)
+    } else {
+        Accion::crea(
+            t(&gitignore),
+            "gitignore",
+            Efecto::Anadir {
+                ruta: gitignore.clone(),
+                lineas,
+            },
+        )
+    });
+
+    // La rama de estado y el remoto: lo que se lee aqui decide si hay red.
+    let previsto = estado::previsto(g, &principal).map_err(|i| renombrar(i, NOMBRE))?;
+    match &previsto {
+        Previsto::SinRed(d) => return Err(sinred(d)),
+        Previsto::Denegada(f, d) => return Err(denegada(*f, d)),
+        _ => {}
+    }
+    let remoto = hay_remoto(g, &principal, NOMBRE)?;
+    let remotas = if remoto {
+        ramas_remotas(c, &principal)?
+    } else {
+        Vec::new()
+    };
+
+    // `main` y `develop` (F1-3): con un commit raiz solo en un repo vacio.
+    ramas(c, &principal, remoto, &remotas, &mut v)?;
+
+    // `info/exclude`.
+    let comun = dir_comun(g, &principal, NOMBRE)?;
+    let exclude = comun.join("info").join("exclude");
+    let texto = leer(&exclude)?
+        .map(|b| String::from_utf8_lossy(&b).into_owned())
+        .unwrap_or_default();
+    v.push(if faltan(&texto, &LINEAS_EXCLUDE).is_empty() {
+        Accion::nueva(t(&exclude), "exclude", "taskcodetwo", Decision::Existe)
+    } else {
+        Accion::crea(
+            t(&exclude),
+            "exclude",
+            Efecto::Excluir {
+                principal: principal.clone(),
+            },
+        )
+    });
+
+    // Rama de estado montada en `tareas/`.
+    let carpeta = principal.join(CARPETA);
+    v.push(match previsto {
+        Previsto::Montada => Accion::nueva(
+            t(&carpeta),
+            "worktree-estado",
+            "taskcodetwo",
+            Decision::Existe,
+        ),
+        Previsto::Local | Previsto::DelRemoto | Previsto::Crear => Accion::crea(
+            t(&carpeta),
+            "worktree-estado",
+            Efecto::MontarEstado {
+                principal: principal.clone(),
+            },
+        ),
+        Previsto::Ocupada => rechazo_016(t(&carpeta), "worktree-estado"),
+        Previsto::OtraRuta(r) => Accion::nueva(
+            t(&carpeta),
+            "worktree-estado",
+            "taskcodetwo",
+            Decision::Rechaza,
+        )
+        .motivo("montada-en-otra-ruta")
+        .orden(format!("git worktree move {} {}", t(&r), CARPETA)),
+        Previsto::SinRed(_) | Previsto::Denegada(..) => {
+            // Ya devuelto arriba.
+            Accion::nueva(
+                t(&carpeta),
+                "worktree-estado",
+                "taskcodetwo",
+                Decision::Pendiente,
+            )
+        }
+    });
+
+    // Personas y puertas: necesitan la plataforma y el token de la persona.
+    let plataforma = if remoto {
+        plataforma(c, &principal, clave.as_ref())?
+    } else {
+        Err("sin-remoto")
+    };
+    v.push(personas_yml(&principal, &plataforma)?);
+
+    // Workflow de CI: ranura hasta que F5-1 entregue el reutilizable (D2).
+    let workflow = principal.join(RUTA_WORKFLOW);
+    v.push(if workflow.is_file() {
+        Accion::nueva(t(&workflow), "workflow", "taskcodetwo", Decision::Existe)
+    } else {
+        Accion::nueva(t(&workflow), "workflow", "taskcodetwo", Decision::Pendiente)
+            .motivo("sin-plantilla")
+    });
+
+    // Rulesets: solo propuesta (D1); los aplica TASK-118.
+    puertas(&plataforma, &mut v);
+    Ok(v)
+}
+
+/// Linea `rechaza motivo=proyecto-016` (C4).
+fn rechazo_016(ruta: String, clase: &'static str) -> Accion {
+    let mut a = Accion::nueva(ruta.clone(), clase, "taskcode-016", Decision::Rechaza)
+        .motivo("proyecto-016")
+        .orden(ARREGLO_016);
+    a.diagnostico = Some(
+        Mensaje::nuevo(&txt::PROYECTO_016)
+            .con("ruta", ruta)
+            .con("arreglo", ARREGLO_016),
+    );
+    a
+}
+
+/// Ramas de codigo que tiene `origin` (de [`RAMAS`]). Sin red, `sinred`.
+fn ramas_remotas(c: &Contexto<'_>, principal: &Path) -> Result<Vec<String>, Informe> {
+    let mut args = vec!["--heads".to_owned(), REMOTO.to_owned()];
+    args.extend(RAMAS.iter().map(|r| format!("refs/heads/{r}")));
+    let args: Vec<&str> = args.iter().map(String::as_str).collect();
+    let s = correr(c.git, principal, NOMBRE, Subcomando::LsRemote, &args)?;
+    if !s.exito() {
+        return Err(match clasificar_fallo_remoto(&s.stderr) {
+            FalloRemoto::SinRed | FalloRemoto::RamaInexistente => sinred(&s.primera_linea_error()),
+            FalloRemoto::Local => crate::git::remoto::error_local(NOMBRE, &s.primera_linea_error()),
+            otro => denegada(otro, &s.primera_linea_error()),
+        });
+    }
+    Ok(s.stdout
+        .lines()
+        .filter_map(|l| l.split_whitespace().nth(1))
+        .filter_map(|r| r.strip_prefix("refs/heads/"))
+        .map(str::to_owned)
+        .collect())
+}
+
+/// Lineas de `main` y `develop` (D3): en un repo sin commits, un commit raiz
+/// con la configuracion y `.gitignore` y las dos ramas en el (publicadas si
+/// hay remoto); con historia, lo hace la persona.
+fn ramas(
+    c: &Contexto<'_>,
+    principal: &Path,
+    remoto: bool,
+    remotas: &[String],
+    v: &mut Vec<Accion>,
+) -> Result<(), Informe> {
+    let g = c.git;
+    let hay_commits = !exigir(
+        g,
+        principal,
+        NOMBRE,
+        Subcomando::ForEachRef,
+        &["--count=1", "--format=%(refname)", "refs/heads/"],
+    )?
+    .trim()
+    .is_empty();
+    let head = worktrees(g, principal, NOMBRE)?
+        .first()
+        .and_then(|w| w.rama.clone());
+    let mut locales = Vec::new();
+    for r in RAMAS {
+        if commit_de(g, principal, NOMBRE, &format!("refs/heads/{r}"))?.is_some() {
+            locales.push(r);
+        }
+    }
+    let mut nuevas: Vec<String> = Vec::new();
+    let mut lineas = Vec::new();
+    for r in RAMAS {
+        let ruta = format!("refs/heads/{r}");
+        let local = locales.contains(&r);
+        let en_remoto = remotas.iter().any(|x| x == r);
+        let a = Accion::nueva(ruta, "rama", "proyecto", Decision::Existe);
+        lineas.push(if local {
+            if remoto && !en_remoto {
+                persona(a, "sin-publicar", format!("git push {REMOTO} {r}"))
+            } else {
+                a
+            }
+        } else if en_remoto {
+            persona(
+                a,
+                "en-el-remoto",
+                format!("git fetch {REMOTO} {r} && git branch {r} {REMOTO}/{r}"),
+            )
+        } else if hay_commits {
+            // Desde la otra rama si existe (HEAD puede estar sin nacer en
+            // esta, tras un `init` cortado a medias); si no, desde HEAD.
+            let desde = locales.first().copied().unwrap_or("HEAD");
+            let mut o = format!("git branch {r} {desde}");
+            if remoto {
+                o.push_str(&format!(" && git push {REMOTO} {r}"));
+            }
+            persona(a, "con-historia", o)
+        } else {
+            nuevas.push(r.to_owned());
+            let mut a = a;
+            a.decision = Decision::Crea;
+            a
+        });
+    }
+    if let Some(primera) = lineas.iter_mut().find(|a| a.decision == Decision::Crea) {
+        primera.efecto = Efecto::RamasRaiz {
+            principal: principal.to_path_buf(),
+            ficheros: vec![config::RUTA.to_owned(), ".gitignore".to_owned()],
+            ramas: nuevas.clone(),
+            head: head.as_ref().is_some_and(|h| nuevas.contains(h)),
+            publicar: remoto,
+        };
+    }
+    v.extend(lineas);
+    // HEAD sin nacer en otra rama (`master`): se queda en ella; la persona
+    // la pasa a `develop` (con los ficheros ya en el disco, `git switch` se
+    // negaria).
+    if !hay_commits
+        && !nuevas.is_empty()
+        && let Some(h) = head.filter(|h| !nuevas.contains(h))
+    {
+        v.push(persona(
+            Accion::nueva("HEAD", "head", "proyecto", Decision::Persona),
+            &format!("head-en-{h}"),
+            "git symbolic-ref HEAD refs/heads/develop && git reset -q".to_owned(),
+        ));
+    }
+    Ok(())
+}
+
+fn persona(mut a: Accion, motivo: &str, orden: String) -> Accion {
+    a.decision = Decision::Persona;
+    a.motivo(motivo).orden(orden)
+}
+
+/// El adaptador de la plataforma con el token de la persona, o el motivo
+/// por el que no lo hay (que queda en las lineas `pendiente`).
+fn plataforma(
+    c: &Contexto<'_>,
+    principal: &Path,
+    clave: Option<&(&'static str, String)>,
+) -> Result<Conexion, Informe> {
+    let Some(url) = url_origin(c.git, principal, NOMBRE)? else {
+        return Ok(Err("sin-remoto"));
+    };
+    let Ok(destino) = detectar(&url, clave.map(|(p, a)| (*p, a.as_str()))) else {
+        return Ok(Err("sin-plataforma"));
+    };
+    let credenciales = (c.credenciales)();
+    if credenciales.token(Rol::Persona).is_none() {
+        return Ok(Err("sin-token"));
+    }
+    let p: Result<Box<dyn Plataforma>, Informe> = match destino.proveedor {
+        Proveedor::GitHub => GitHub::nuevo(
+            destino.api,
+            &destino.proyecto,
+            credenciales,
+            Personas::default(),
+        )
+        .map(|x| Box::new(x) as Box<dyn Plataforma>),
+        Proveedor::GitLab => GitLab::nuevo(
+            destino.api,
+            &destino.proyecto,
+            credenciales,
+            Personas::default(),
+        )
+        .map(|x| Box::new(x) as Box<dyn Plataforma>),
+    };
+    Ok(p.map(|p| (p, destino.proveedor))
+        .map_err(|_| "sin-plataforma"))
+}
+
+/// Motivo de un informe de la plataforma para la linea `pendiente`.
+fn motivo_de(i: &Informe) -> String {
+    i.valor("motivo")
+        .map_or_else(|| i.resultado.texto().to_owned(), str::to_owned)
+}
+
+/// `personas.yml`: la persona de `GET /user` (F1-6), solo si falta.
+fn personas_yml(principal: &Path, plataforma: &Conexion) -> Result<Accion, Informe> {
+    let ruta = principal.join(personas::RUTA);
+    let nueva = |d: Decision| Accion::nueva(t(&ruta), "personas", "taskcodetwo", d);
+    if let Some(bytes) = leer(&ruta)? {
+        return Ok(match Personas::leer(&String::from_utf8_lossy(&bytes)) {
+            Ok(_) => nueva(Decision::Existe),
+            Err(_) => {
+                let mut a = nueva(Decision::Rechaza).motivo("personas");
+                a.dueno = "ajeno";
+                a.diagnostico =
+                    Some(Mensaje::nuevo(&txt::PERSONAS_NO_VALE).con("ruta", personas::RUTA));
+                a
+            }
+        });
+    }
+    let (p, _) = match plataforma {
+        Ok(p) => p,
+        Err(m) => return Ok(nueva(Decision::Pendiente).motivo(*m)),
+    };
+    let u = match p.identidad(Rol::Persona) {
+        Ok(u) => u,
+        Err(i) => return Ok(nueva(Decision::Pendiente).motivo(motivo_de(&i))),
+    };
+    if u.tipo != "User" {
+        return Ok(nueva(Decision::Pendiente).motivo("no-es-persona"));
+    }
+    let texto = personas::texto_de(&u);
+    if Personas::leer(&texto).is_err() {
+        return Ok(nueva(Decision::Pendiente).motivo("login-no-valido"));
+    }
+    Ok(Accion::crea(
+        t(&ruta),
+        "personas",
+        Efecto::Escribir {
+            ruta: ruta.clone(),
+            contenido: texto,
+        },
+    ))
+}
+
+/// Una linea por ruleset del modelo de F1-5: `existe` si la plataforma ya
+/// lo tiene; si no, `persona` con la orden que lo aplicara (TASK-118) y la
+/// propuesta en JSON. Solo lecturas: ninguna escritura en la plataforma.
+fn puertas(plataforma: &Conexion, v: &mut Vec<Accion>) {
+    let leidas = match plataforma {
+        Ok((p, prov)) => Ok((p.leer_puertas(Rol::Persona, "develop"), *prov)),
+        Err(m) => Err((*m).to_owned()),
+    };
+    for puerta in modelo() {
+        let nombre = puerta.nombre_completo();
+        let nueva = |d: Decision| Accion::nueva(nombre.clone(), "ruleset", "taskcodetwo", d);
+        v.push(match &leidas {
+            Err(m) => nueva(Decision::Pendiente).motivo(m.clone()),
+            Ok((Err(i), _)) => nueva(Decision::Pendiente).motivo(motivo_de(i)),
+            Ok((Ok(r), _)) if !r.plan_da_reglas => nueva(Decision::Pendiente).motivo("plan"),
+            Ok((Ok(r), _)) if r.rulesets.contains(&nombre) => nueva(Decision::Existe),
+            Ok((Ok(_), prov)) => {
+                let mut a = nueva(Decision::Persona)
+                    .motivo("sin-checks")
+                    .orden(ORDEN_PUERTAS);
+                if *prov == Proveedor::GitHub {
+                    a.propuesta = Some(ruleset_github(&puerta));
+                }
+                a
+            }
+        });
+    }
+}
diff --git a/crates/tc2/src/init/mod.rs b/crates/tc2/src/init/mod.rs
new file mode 100644
index 0000000..059efb9
--- /dev/null
+++ b/crates/tc2/src/init/mod.rs
@@ -0,0 +1,484 @@
+//! `tc2 init` (F3-1): deja un repo listo para TaskCodeTwo.
+//!
+//! Sigue el patron de `tc2 desinstalar` («plan como dato», F1-12 PL1): un
+//! inventario de solo lectura calcula el plan (una [`Accion`] por artefacto,
+//! con su decision) y la ejecucion aplica ese mismo plan; `--simular` lo
+//! imprime y para. Lo que se crea sale de los nombres de [`crate::huella`],
+//! los mismos que lee `desinstalar`, asi que se reconocen sin mas.
+//!
+//! Que crea, solo si falta: la configuracion minima, las lineas de
+//! `.gitignore` y de `info/exclude`, `main` y `develop` con un commit raiz
+//! (solo en un repo sin commits; con historia lo hace la persona), la rama
+//! de estado montada en `tareas/` y `personas.yml` con la persona del token.
+//! El workflow de CI queda como ranura `pendiente` hasta F5-1 y los rulesets
+//! `taskcodetwo-*` solo se proponen (`persona`): aplicarlos es de TASK-118.
+//!
+//! Nunca pisa la 0.16 ni una configuracion que no vale: `rechazada` sin
+//! escribir nada. Con `origin` y sin red, `sinred pendiente=no` sin escribir.
+//!
+//! Salida: una linea `accion ruta=... clase=... dueno=... decision=...` por
+//! accion y al final `init <resultado> [simular=si] crea=N existe=N
+//! persona=N pendiente=N`.
+
+mod aplicar;
+mod inventario;
+
+use std::path::{Path, PathBuf};
+
+use crate::Salida;
+use crate::contrato::{Informe, LineaAccion, Resultado};
+use crate::git::Git;
+use crate::i18n::catalogo::init as txt;
+use crate::i18n::{Idioma, Mensaje};
+use crate::plataforma::Credenciales;
+
+/// Nombre de la orden.
+pub const NOMBRE: &str = "init";
+
+/// Orden que aplicara las puertas (TASK-118).
+pub const ORDEN_PUERTAS: &str = "tc2 init --aplicar-puertas";
+
+/// Arreglo para un proyecto de la 0.16.
+pub const ARREGLO_016: &str = "tc2 actualizar --simular";
+
+/// Opciones de la orden.
+#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
+pub struct Opciones {
+    /// Solo imprimir el plan.
+    pub simular: bool,
+}
+
+/// Lee las opciones (sin `--json`).
+pub fn leer_opciones(args: &[String]) -> Result<Opciones, Informe> {
+    let mut op = Opciones::default();
+    for a in args {
+        match a.as_str() {
+            "--simular" if !op.simular => op.simular = true,
+            _ => {
+                return Err(Informe::nuevo(NOMBRE, Resultado::Uso)
+                    .con("motivo", "argumentos")
+                    .diagnostico(&txt::USO_INIT));
+            }
+        }
+    }
+    Ok(op)
+}
+
+/// Que se decide hacer con un artefacto.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Decision {
+    /// Falta y lo crea `init`.
+    Crea,
+    /// Ya esta: no se toca.
+    Existe,
+    /// Lo hace la persona con la orden de la linea.
+    Persona,
+    /// No se puede hacer ahora (sin remoto, sin token, sin plantilla...).
+    Pendiente,
+    /// Impide aplicar el plan: `rechazada` sin tocar nada.
+    Rechaza,
+}
+
+impl Decision {
+    /// Valor de `decision=`.
+    pub fn texto(self) -> &'static str {
+        match self {
+            Self::Crea => "crea",
+            Self::Existe => "existe",
+            Self::Persona => "persona",
+            Self::Pendiente => "pendiente",
+            Self::Rechaza => "rechaza",
+        }
+    }
+}
+
+/// Lo que hace una accion `crea` al aplicarse.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub(crate) enum Efecto {
+    /// Nada (la hace otra linea, o es informativa).
+    Ninguno,
+    /// Crear un fichero que no existe.
+    Escribir {
+        /// El fichero.
+        ruta: PathBuf,
+        /// Su contenido.
+        contenido: String,
+    },
+    /// Anadir a un fichero las lineas que le falten.
+    Anadir {
+        /// El fichero.
+        ruta: PathBuf,
+        /// Las lineas.
+        lineas: Vec<String>,
+    },
+    /// Anadir lineas a `info/exclude` (`git::rama::excluir`).
+    Excluir {
+        /// Worktree principal.
+        principal: PathBuf,
+    },
+    /// Commit raiz con `ficheros` y las ramas `ramas` en el (repo vacio).
+    RamasRaiz {
+        /// Worktree principal.
+        principal: PathBuf,
+        /// Rutas relativas que lleva el commit, leidas del disco al aplicar.
+        ficheros: Vec<String>,
+        /// Ramas que se crean.
+        ramas: Vec<String>,
+        /// HEAD esta sin nacer en una de ellas: se pone el indice al dia.
+        head: bool,
+        /// Se publican en `origin`.
+        publicar: bool,
+    },
+    /// `estado::montar`.
+    MontarEstado {
+        /// Worktree principal.
+        principal: PathBuf,
+    },
+}
+
+/// Una linea del plan.
+#[derive(Debug, Clone, PartialEq)]
+pub struct Accion {
+    /// Ruta, ref o nombre.
+    pub ruta: String,
+    /// Que es: `config`, `gitignore`, `rama`, `worktree-estado`...
+    pub clase: &'static str,
+    /// De quien es: `taskcodetwo` o `proyecto`.
+    pub dueno: &'static str,
+    /// Que se hace.
+    pub decision: Decision,
+    /// Por que (si no es evidente).
+    pub motivo: Option<String>,
+    /// La orden de la persona.
+    pub orden: Option<String>,
+    /// Propuesta para la plataforma (el ruleset), solo en JSON.
+    pub propuesta: Option<serde_json::Value>,
+    /// Que decir a la persona si bloquea.
+    pub(crate) diagnostico: Option<Mensaje>,
+    /// Lo que se hace al aplicar.
+    pub(crate) efecto: Efecto,
+}
+
+impl Accion {
+    pub(crate) fn nueva(
+        ruta: impl Into<String>,
+        clase: &'static str,
+        dueno: &'static str,
+        decision: Decision,
+    ) -> Self {
+        Self {
+            ruta: ruta.into(),
+            clase,
+            dueno,
+            decision,
+            motivo: None,
+            orden: None,
+            propuesta: None,
+            diagnostico: None,
+            efecto: Efecto::Ninguno,
+        }
+    }
+
+    /// Accion `crea` de la 2.0 con su efecto.
+    pub(crate) fn crea(ruta: impl Into<String>, clase: &'static str, efecto: Efecto) -> Self {
+        let mut a = Self::nueva(ruta, clase, "taskcodetwo", Decision::Crea);
+        a.efecto = efecto;
+        a
+    }
+
+    #[must_use]
+    pub(crate) fn motivo(mut self, m: impl Into<String>) -> Self {
+        self.motivo = Some(m.into());
+        self
+    }
+
+    #[must_use]
+    pub(crate) fn orden(mut self, o: impl Into<String>) -> Self {
+        self.orden = Some(o.into());
+        self
+    }
+
+    fn comun(&self) -> LineaAccion<'_> {
+        LineaAccion {
+            ruta: &self.ruta,
+            clase: self.clase,
+            dueno: self.dueno,
+            decision: self.decision.texto(),
+            extra: &[],
+        }
+    }
+
+    /// Linea `accion ruta=... clase=... dueno=... decision=...`.
+    pub fn linea(&self) -> String {
+        LineaAccion {
+            extra: &[
+                ("motivo", self.motivo.as_deref()),
+                ("orden", self.orden.as_deref()),
+            ],
+            ..self.comun()
+        }
+        .texto()
+    }
+
+    /// La misma linea en JSON, con la propuesta si la hay.
+    pub fn json(&self) -> String {
+        let mut s = LineaAccion {
+            extra: &[
+                ("motivo", self.motivo.as_deref()),
+                ("orden", self.orden.as_deref()),
+            ],
+            ..self.comun()
+        }
+        .json();
+        if let Some(p) = &self.propuesta
+            && let Some(sin_cierre) = s.strip_suffix("}}")
+        {
+            s = format!("{sin_cierre},\"propuesta\":{p}}}}}");
+        }
+        s
+    }
+}
+
+/// Lo que necesita `init`, inyectable en los tests.
+pub struct Contexto<'a> {
+    /// Carpeta del proyecto.
+    pub dir: &'a Path,
+    /// Puerto Git.
+    pub git: &'a Git,
+    /// Tokens por rol (se piden al construir el adaptador de la plataforma).
+    pub credenciales: &'a dyn Fn() -> Box<dyn Credenciales>,
+}
+
+/// Resultado: las acciones (ya aplicadas, salvo con `--simular`) y la linea
+/// final.
+#[derive(Debug, Clone)]
+pub struct Hecho {
+    /// Una por artefacto.
+    pub acciones: Vec<Accion>,
+    /// La linea de contrato final.
+    pub informe: Informe,
+}
+
+impl Hecho {
+    fn solo(informe: Informe) -> Self {
+        Self {
+            acciones: Vec::new(),
+            informe,
+        }
+    }
+
+    /// La salida del proceso: las lineas de accion y la final; en stderr, el
+    /// diagnostico si fallo y un aviso si hay acciones de la persona.
+    pub fn salida_en(&self, json: bool, idioma: Idioma) -> Salida {
+        let fin = self.informe.salida_en(json, idioma);
+        let mut stdout = String::new();
+        for a in &self.acciones {
+            stdout.push_str(&if json { a.json() } else { a.linea() });
+            stdout.push('\n');
+        }
+        stdout.push_str(&fin.stdout);
+        let mut stderr = String::new();
+        let personas = self
+            .acciones
+            .iter()
+            .filter(|a| a.decision == Decision::Persona)
+            .count();
+        if personas > 0 && self.informe.resultado.es_exito() {
+            stderr.push_str(
+                &Mensaje::nuevo(&txt::AVISO_PERSONA)
+                    .con("n", personas.to_string())
+                    .texto(idioma),
+            );
+            stderr.push('\n');
+        }
+        stderr.push_str(&fin.stderr);
+        Salida {
+            codigo: fin.codigo,
+            stdout,
+            stderr,
+        }
+    }
+}
+
+/// Ruta como texto para la linea.
+pub(crate) fn t(p: &Path) -> String {
+    p.display().to_string()
+}
+
+/// Informe de error al leer `ruta`.
+pub(crate) fn no_se_lee(ruta: &Path, e: &std::io::Error) -> Informe {
+    Informe::nuevo(NOMBRE, Resultado::Error)
+        .con("ruta", t(ruta))
+        .diagnostico(
+            Mensaje::nuevo(&txt::NO_SE_LEE)
+                .con("ruta", t(ruta))
+                .con("error", e.to_string()),
+        )
+}
+
+/// Lee un fichero; `None` si no existe.
+pub(crate) fn leer(p: &Path) -> Result<Option<Vec<u8>>, Informe> {
+    match std::fs::read(p) {
+        Ok(b) => Ok(Some(b)),
+        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
+        Err(e) => Err(no_se_lee(p, &e)),
+    }
+}
+
+/// El plan, sin aplicar nada.
+pub fn plan(c: &Contexto<'_>) -> Result<Vec<Accion>, Informe> {
+    inventario::plan(c)
+}
+
+/// `init` con las opciones ya leidas.
+pub fn init(op: Opciones, c: &Contexto<'_>) -> Hecho {
+    let mut acciones = match plan(c) {
+        Ok(v) => v,
+        Err(i) => return Hecho::solo(i),
+    };
+    let cuenta = |r: Resultado, acciones: &[Accion], cabeza: &[(&str, String)]| {
+        let n = |d: Decision| acciones.iter().filter(|a| a.decision == d).count();
+        let mut i = Informe::nuevo(NOMBRE, r);
+        for (k, v) in cabeza {
+            i = i.con(k, v.clone());
+        }
+        if op.simular {
+            i = i.con("simular", "si");
+        }
+        i.con("crea", n(Decision::Crea).to_string())
+            .con("existe", n(Decision::Existe).to_string())
+            .con("persona", n(Decision::Persona).to_string())
+            .con("pendiente", n(Decision::Pendiente).to_string())
+    };
+    if let Some(b) = acciones
+        .iter()
+        .find(|a| a.decision == Decision::Rechaza)
+        .cloned()
+    {
+        // No se toca nada: lo que se habria creado queda pendiente.
+        for a in acciones.iter_mut().filter(|a| a.decision == Decision::Crea) {
+            a.decision = Decision::Pendiente;
+            a.motivo = Some("no-aplicada".to_owned());
+        }
+        let motivo = b.motivo.clone().unwrap_or_default();
+        let mut cabeza = vec![("motivo", motivo.clone())];
+        if let Some(o) = &b.orden {
+            cabeza.push(("arreglo", o.clone()));
+        }
+        cabeza.push(("ruta", b.ruta.clone()));
+        let diag = b.diagnostico.clone().unwrap_or_else(|| {
+            Mensaje::nuevo(&txt::BLOQUEADA)
+                .con("ruta", b.ruta.clone())
+                .con("motivo", motivo)
+        });
+        let informe = cuenta(Resultado::Rechazada, &acciones, &cabeza).diagnostico(diag);
+        return Hecho { acciones, informe };
+    }
+    if !op.simular {
+        // (ruta, error) del primer fallo; lo que venia detras no se aplica.
+        let mut fallo: Option<(String, String)> = None;
+        for a in acciones.iter_mut() {
+            if a.decision != Decision::Crea {
+                continue;
+            }
+            if fallo.is_some() {
+                a.decision = Decision::Pendiente;
+                a.motivo = Some("no-aplicada".to_owned());
+                continue;
+            }
+            if let Err(e) = aplicar::aplicar(&a.efecto, c) {
+                a.decision = Decision::Pendiente;
+                a.motivo = Some("error".to_owned());
+                fallo = Some((a.ruta.clone(), e));
+            }
+        }
+        if let Some((ruta, e)) = fallo {
+            let informe = cuenta(Resultado::Error, &acciones, &[("ruta", ruta.clone())])
+                .diagnostico(
+                    Mensaje::nuevo(&txt::FALLO_AL_APLICAR)
+                        .con("ruta", ruta)
+                        .con("error", e),
+                );
+            return Hecho { acciones, informe };
+        }
+    }
+    let r = if acciones.iter().any(|a| a.decision == Decision::Crea) {
+        Resultado::Ok
+    } else {
+        Resultado::SinCambios
+    };
+    let informe = cuenta(r, &acciones, &[]);
+    Hecho { acciones, informe }
+}
+
+/// La orden completa desde la linea de ordenes, en la carpeta actual y con
+/// los tokens del proceso.
+pub fn orden(args: &[String], json: bool, idioma: Idioma) -> Salida {
+    let op = match leer_opciones(args) {
+        Ok(o) => o,
+        Err(i) => return i.salida_en(json, idioma),
+    };
+    let dir = match std::env::current_dir() {
+        Ok(d) => d,
+        Err(e) => {
+            return Informe::nuevo(NOMBRE, Resultado::Error)
+                .diagnostico(
+                    Mensaje::nuevo(&crate::i18n::catalogo::cli::CARPETA_ACTUAL)
+                        .con("error", e.to_string()),
+                )
+                .salida_en(json, idioma);
+        }
+    };
+    let git = Git::nuevo();
+    let credenciales =
+        || -> Box<dyn Credenciales> { Box::new(crate::plataforma::CredencialesEntorno) };
+    let c = Contexto {
+        dir: &dir,
+        git: &git,
+        credenciales: &credenciales,
+    };
+    init(op, &c).salida_en(json, idioma)
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn opciones() {
+        let v = |a: &[&str]| a.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
+        assert_eq!(leer_opciones(&v(&[])), Ok(Opciones { simular: false }));
+        assert_eq!(
+            leer_opciones(&v(&["--simular"])),
+            Ok(Opciones { simular: true })
+        );
+        for args in [v(&["--simular", "--simular"]), v(&["--dry-run"]), v(&["x"])] {
+            let i = leer_opciones(&args).err();
+            assert_eq!(
+                i.as_ref().map(|i| (i.resultado, i.valor("motivo"))),
+                Some((Resultado::Uso, Some("argumentos"))),
+                "{args:?}"
+            );
+        }
+    }
+
+    #[test]
+    fn linea_y_json_con_propuesta() {
+        let mut a = Accion::nueva(
+            "taskcodetwo-main",
+            "ruleset",
+            "taskcodetwo",
+            Decision::Persona,
+        )
+        .motivo("sin-checks")
+        .orden(ORDEN_PUERTAS);
+        assert_eq!(
+            a.linea(),
+            "accion ruta=taskcodetwo-main clase=ruleset dueno=taskcodetwo decision=persona motivo=sin-checks orden=tc2%20init%20--aplicar-puertas"
+        );
+        a.propuesta = Some(serde_json::json!({"name": "taskcodetwo-main"}));
+        let v: serde_json::Value = serde_json::from_str(&a.json()).unwrap_or_default();
+        assert_eq!(v["accion"]["propuesta"]["name"], "taskcodetwo-main");
+        assert_eq!(v["accion"]["orden"], ORDEN_PUERTAS);
+    }
+}
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 0d9264c..724d883 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -10,7 +10,8 @@
 //! F2-8), [`version`] (`version --json` y `compatible`, F2-12) y
 //! [`actualizar_cli`] (`actualizar --cli`, F2-12) e [`identidad`] (quien
 //! ejecuta y `tc2 identidad`, F2-13) y [`huella`] (los nombres que la 2.0
-//! deja en un repo, F1-12 D1a) e [`instalacion`] (donde vive el `tc2` de la
+//! deja en un repo, F1-12 D1a) e [`init`] (deja un repo listo, F3-1) e
+//! [`instalacion`] (donde vive el `tc2` de la
 //! maquina, F2-12 y F2-14) y [`desinstalar`] (quitar TaskCodeTwo, F2-14). Las capas
 //! que vienen entran como modulos de esta biblioteca.
 
@@ -26,6 +27,7 @@ pub mod git;
 pub mod huella;
 pub mod i18n;
 pub mod identidad;
+pub mod init;
 pub mod instalacion;
 pub mod plataforma;
 pub mod tarea;
@@ -120,6 +122,10 @@ where
             let (json, resto) = con_json(resto);
             desinstalar::orden(&resto, json, idioma)
         }
+        [orden, resto @ ..] if orden == init::NOMBRE => {
+            let (json, resto) = con_json(resto);
+            init::orden(&resto, json, idioma)
+        }
         [orden, resto @ ..] if orden == identidad::NOMBRE => {
             let (json, resto) = con_json(resto);
             orden_identidad(&resto).salida_en(json, idioma)
diff --git a/crates/tc2/src/plataforma/personas.rs b/crates/tc2/src/plataforma/personas.rs
index b559768..12511d5 100644
--- a/crates/tc2/src/plataforma/personas.rs
+++ b/crates/tc2/src/plataforma/personas.rs
@@ -251,11 +251,43 @@ impl Personas {
     }
 }
 
+/// Texto de una lista con una sola persona, la de `GET /user` (lo que
+/// escribe `tc2 init`, F3-1). El login va entre comillas dobles; quien lo
+/// escribe lo valida antes con [`Personas::leer`].
+pub fn texto_de(u: &super::Usuario) -> String {
+    format!(
+        "---\npersonas:\n  - {{id: {}, login: \"{}\"}}\n",
+        u.id,
+        u.login.replace('\\', "\\\\").replace('"', "\\\"")
+    )
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
     use crate::i18n::Idioma;
 
+    /// Lo que escribe `init` se vuelve a leer igual.
+    #[test]
+    fn texto_de_un_usuario_se_lee() -> Result<(), Informe> {
+        let u = super::super::Usuario {
+            login: "carlos".into(),
+            id: 1001,
+            tipo: "User".into(),
+        };
+        let l = Personas::leer(&texto_de(&u))?;
+        assert_eq!(
+            l.entradas(),
+            [Persona {
+                id: 1001,
+                login: "carlos".into(),
+                nombre: None,
+                correos: Vec::new(),
+            }]
+        );
+        Ok(())
+    }
+
     fn error(texto: &str) -> String {
         Personas::leer(texto)
             .err()
diff --git a/crates/tc2/src/plataforma/puertas.rs b/crates/tc2/src/plataforma/puertas.rs
index 42d65bf..c8ee03b 100644
--- a/crates/tc2/src/plataforma/puertas.rs
+++ b/crates/tc2/src/plataforma/puertas.rs
@@ -127,10 +127,201 @@ pub fn rechazada_por_config(pr: u64, garantia: &str, arreglo: &'static Texto) ->
         )
 }
 
+/// A que se aplica una puerta del modelo neutral (F1-5, decision 1).
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum Objetivo {
+    /// Ramas, como refs completas o patrones (`refs/heads/feature/**`).
+    Ramas(Vec<String>),
+    /// Tags, como patrones (`refs/tags/v*`).
+    Tags(Vec<String>),
+}
+
+/// Solo por PR (F1-5, decision 2).
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub struct SoloPr {
+    /// Aprobaciones de personas.
+    pub aprobaciones: u8,
+}
+
+/// Una puerta del modelo neutral, sin nombres de GitHub ni de GitLab.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Puerta {
+    /// Nombre sin el prefijo (`develop`, `tags-version`...).
+    pub nombre: &'static str,
+    /// Ramas o tags.
+    pub objetivo: Objetivo,
+    /// Solo por PR, con sus aprobaciones; `None` si se empuja directo.
+    pub solo_pr: Option<SoloPr>,
+    /// Rama al dia con la base antes del merge (`strict`).
+    pub al_dia: bool,
+    /// Checks obligatorios. Su origen (`integration_id`) llega con F6-1.
+    pub checks: Vec<&'static str>,
+}
+
+impl Puerta {
+    /// Nombre completo, con el prefijo de la huella (`taskcodetwo-develop`).
+    pub fn nombre_completo(&self) -> String {
+        format!("{}{}", crate::huella::PREFIJO_RULESETS, self.nombre)
+    }
+}
+
+/// El modelo de puertas de F1-5 (decision 1), con los nombres de la huella
+/// (F1-12 D1a): `develop`, `main`, ramas de tarea, rama de estado y tags.
+pub fn modelo() -> Vec<Puerta> {
+    let rama = |r: &str| format!("refs/heads/{r}");
+    vec![
+        Puerta {
+            nombre: "develop",
+            objetivo: Objetivo::Ramas(vec![rama("develop")]),
+            solo_pr: Some(SoloPr { aprobaciones: 1 }),
+            al_dia: true,
+            checks: vec!["verificacion", "revision-ia", "carril"],
+        },
+        Puerta {
+            nombre: "main",
+            objetivo: Objetivo::Ramas(vec![rama("main")]),
+            solo_pr: Some(SoloPr { aprobaciones: 1 }),
+            // F1-5 decision 2: `main` sin `strict` (PoC E).
+            al_dia: false,
+            checks: vec!["verificacion", "carril"],
+        },
+        Puerta {
+            nombre: "ramas-de-tarea",
+            objetivo: Objetivo::Ramas(vec![rama("feature/**"), rama("fix/**"), rama("hotfix/**")]),
+            solo_pr: None,
+            al_dia: false,
+            checks: Vec::new(),
+        },
+        Puerta {
+            nombre: "tareas",
+            objetivo: Objetivo::Ramas(vec![rama(crate::huella::RAMA_ESTADO)]),
+            solo_pr: None,
+            al_dia: false,
+            checks: Vec::new(),
+        },
+        Puerta {
+            nombre: "tags-version",
+            objetivo: Objetivo::Tags(vec!["refs/tags/v*".to_owned()]),
+            solo_pr: None,
+            al_dia: false,
+            checks: Vec::new(),
+        },
+    ]
+}
+
+/// La puerta traducida a un ruleset de GitHub (la forma de los JSON de la
+/// PoC C de F1-5). Solo genera la propuesta: aplicarla es de TASK-118, y
+/// los checks van sin `integration_id` hasta que F6-1 de el real.
+pub fn ruleset_github(p: &Puerta) -> serde_json::Value {
+    use serde_json::json;
+    let (target, include) = match &p.objetivo {
+        Objetivo::Ramas(r) => ("branch", r),
+        Objetivo::Tags(t) => ("tag", t),
+    };
+    let mut reglas = vec![json!({"type": "deletion"})];
+    match p.objetivo {
+        Objetivo::Ramas(_) => reglas.push(json!({"type": "non_fast_forward"})),
+        Objetivo::Tags(_) => reglas.push(json!({
+            "type": "update",
+            "parameters": {"update_allows_fetch_and_merge": false}
+        })),
+    }
+    if let Some(pr) = p.solo_pr {
+        reglas.push(json!({
+            "type": "pull_request",
+            "parameters": {
+                "allowed_merge_methods": ["merge"],
+                "dismiss_stale_reviews_on_push": true,
+                "require_code_owner_review": true,
+                "require_last_push_approval": true,
+                "required_approving_review_count": pr.aprobaciones,
+                "required_review_thread_resolution": false
+            }
+        }));
+    }
+    if !p.checks.is_empty() {
+        let checks: Vec<serde_json::Value> =
+            p.checks.iter().map(|c| json!({"context": c})).collect();
+        reglas.push(json!({
+            "type": "required_status_checks",
+            "parameters": {
+                "strict_required_status_checks_policy": p.al_dia,
+                "required_status_checks": checks
+            }
+        }));
+    }
+    json!({
+        "name": p.nombre_completo(),
+        "target": target,
+        "enforcement": "active",
+        "bypass_actors": [],
+        "conditions": {"ref_name": {"include": include, "exclude": []}},
+        "rules": reglas
+    })
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
 
+    /// Los rulesets propuestos llevan el prefijo de la huella y la forma de
+    /// la PoC C de F1-5 (`main` sin `strict`, tags sin mover).
+    #[test]
+    fn modelo_y_rulesets_de_github() {
+        let m = modelo();
+        let nombres: Vec<String> = m.iter().map(Puerta::nombre_completo).collect();
+        assert_eq!(
+            nombres,
+            [
+                "taskcodetwo-develop",
+                "taskcodetwo-main",
+                "taskcodetwo-ramas-de-tarea",
+                "taskcodetwo-tareas",
+                "taskcodetwo-tags-version"
+            ]
+        );
+        let tipos = |v: &serde_json::Value| -> Vec<String> {
+            v["rules"]
+                .as_array()
+                .map(|r| {
+                    r.iter()
+                        .filter_map(|x| x["type"].as_str().map(str::to_owned))
+                        .collect()
+                })
+                .unwrap_or_default()
+        };
+        let develop = ruleset_github(&m[0]);
+        assert_eq!(develop["name"], "taskcodetwo-develop");
+        assert_eq!(develop["target"], "branch");
+        assert_eq!(
+            tipos(&develop),
+            [
+                "deletion",
+                "non_fast_forward",
+                "pull_request",
+                "required_status_checks"
+            ]
+        );
+        assert_eq!(
+            develop["rules"][3]["parameters"]["strict_required_status_checks_policy"],
+            true
+        );
+        let main = ruleset_github(&m[1]);
+        assert_eq!(
+            main["rules"][3]["parameters"]["strict_required_status_checks_policy"],
+            false
+        );
+        let tareas = ruleset_github(&m[3]);
+        assert_eq!(
+            tareas["conditions"]["ref_name"]["include"][0],
+            "refs/heads/taskcodetwo/tareas"
+        );
+        assert_eq!(tipos(&tareas), ["deletion", "non_fast_forward"]);
+        let tags = ruleset_github(&m[4]);
+        assert_eq!(tags["target"], "tag");
+        assert_eq!(tipos(&tags), ["deletion", "update"]);
+    }
+
     fn l(checks: &[(&str, EstadoCheck)], aprob: usize, al_dia: bool) -> Lecturas {
         Lecturas {
             pr: 7,
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-028/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-028/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-028/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/init.rs"
````

````
crates/tc2/tests/init.rs | 783 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 783 insertions(+)
````
