# Peticion de revision — TASK-024 (ronda 2)

- Tarea: TASK-024 — F2-8 i18n de mensajes
- Rama revisada: feature/task-024-f2-8-i18n-de-mensajes
- Rama base: develop
- Commit revisado (HEAD): 93e40c6be0c023cde0e0699da2d7873e1f3be9b8
- Fecha: 2026-11-16
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-024 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde a6713930b5f17122ab4d850a1293fc185328c47d (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (5 hallazgo(s)))

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

- tareas/03-en-revision/TASK-024/tarea.md
- tareas/03-en-revision/TASK-024/planificacion/
- crates/tc2/src/git/puerto.rs
- crates/tc2/src/i18n/catalogo/git.rs
- crates/tc2/src/i18n/catalogo/plataforma.rs
- crates/tc2/src/i18n/mod.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/plataforma/github.rs

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 3 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/plataforma/github.rs:329 | informe-revision-1-rust-wasm-reviewer.md |

## Commits a revisar (git log a6713930b5f17122ab4d850a1293fc185328c47d..HEAD)

````
93e40c6 chore(TASK-024): autorrevision ronda 2
f46e10e fix(TASK-024): correcciones de la ronda 1 de revision
4739b43 chore(TASK-024): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff a6713930b5f17122ab4d850a1293fc185328c47d..HEAD)

````diff
diff --git a/crates/tc2/src/git/puerto.rs b/crates/tc2/src/git/puerto.rs
index ced1111..92d8790 100644
--- a/crates/tc2/src/git/puerto.rs
+++ b/crates/tc2/src/git/puerto.rs
@@ -142,8 +142,9 @@ impl SalidaGit {
 /// La llamada no llego a ejecutarse.
 #[derive(Debug, Clone, PartialEq, Eq)]
 pub enum ErrorGit {
-    /// Argumento prohibido dentro de un subcomando permitido.
-    Prohibido(String),
+    /// Llamada prohibida por la capa (argumento o indice), ya como mensaje
+    /// del catalogo.
+    Prohibido(Mensaje),
     /// No se pudo lanzar el binario.
     Lanzar(String),
 }
@@ -160,7 +161,7 @@ impl ErrorGit {
     /// se traduce; `Display` es su texto en espanol.
     pub fn mensaje(&self) -> Mensaje {
         match self {
-            Self::Prohibido(a) => Mensaje::nuevo(&txt::ARGUMENTO_PROHIBIDO).con("argumento", a),
+            Self::Prohibido(m) => m.clone(),
             Self::Lanzar(e) => Mensaje::nuevo(&txt::NO_SE_PUDO_LANZAR_GIT).con("error", e),
         }
     }
@@ -232,10 +233,9 @@ impl Git {
         args: &[&str],
     ) -> Result<SalidaGit, ErrorGit> {
         if sub.exige_indice_temporal() {
-            return Err(ErrorGit::Prohibido(format!(
-                "{} sin indice temporal",
-                sub.nombre()
-            )));
+            return Err(ErrorGit::Prohibido(
+                Mensaje::nuevo(&txt::SIN_INDICE_TEMPORAL).con("sub", sub.nombre()),
+            ));
         }
         self.lanzar(dir, sub, args, None)
     }
@@ -251,16 +251,14 @@ impl Git {
         indice: &Path,
     ) -> Result<SalidaGit, ErrorGit> {
         if !sub.exige_indice_temporal() {
-            return Err(ErrorGit::Prohibido(format!(
-                "{} con indice temporal",
-                sub.nombre()
-            )));
+            return Err(ErrorGit::Prohibido(
+                Mensaje::nuevo(&txt::CON_INDICE_TEMPORAL).con("sub", sub.nombre()),
+            ));
         }
         if !indice.is_absolute() {
-            return Err(ErrorGit::Prohibido(format!(
-                "{} con indice relativo",
-                sub.nombre()
-            )));
+            return Err(ErrorGit::Prohibido(
+                Mensaje::nuevo(&txt::INDICE_RELATIVO).con("sub", sub.nombre()),
+            ));
         }
         self.lanzar(dir, sub, args, Some(indice))
     }
@@ -308,7 +306,12 @@ impl Git {
 /// rutas tras `--` en los subcomandos que las aceptan (`add`, `commit`,
 /// `status`, `diff`): ahi una palabra como `--force` es dato, no opcion.
 fn comprobar_argumentos(sub: Subcomando, args: &[&str]) -> Result<(), ErrorGit> {
-    let prohibido = |a: &str| Err(ErrorGit::Prohibido(format!("{} {a}", sub.nombre())));
+    let prohibido = |a: &str| {
+        Err(ErrorGit::Prohibido(
+            Mensaje::nuevo(&txt::ARGUMENTO_PROHIBIDO)
+                .con("argumento", format!("{} {a}", sub.nombre())),
+        ))
+    };
     let con_rutas = matches!(
         sub,
         Subcomando::Add | Subcomando::Commit | Subcomando::Status | Subcomando::Diff
diff --git a/crates/tc2/src/i18n/catalogo/git.rs b/crates/tc2/src/i18n/catalogo/git.rs
index ece9886..b2ef392 100644
--- a/crates/tc2/src/i18n/catalogo/git.rs
+++ b/crates/tc2/src/i18n/catalogo/git.rs
@@ -6,6 +6,18 @@ crate::i18n::textos! {
         es: "argumento prohibido para la capa: {argumento}",
         en: "argument not allowed for the layer: {argumento}",
     }
+    SIN_INDICE_TEMPORAL {
+        es: "argumento prohibido para la capa: {sub} sin indice temporal",
+        en: "argument not allowed for the layer: {sub} without a temporary index",
+    }
+    CON_INDICE_TEMPORAL {
+        es: "argumento prohibido para la capa: {sub} con indice temporal",
+        en: "argument not allowed for the layer: {sub} with a temporary index",
+    }
+    INDICE_RELATIVO {
+        es: "argumento prohibido para la capa: {sub} con indice relativo",
+        en: "argument not allowed for the layer: {sub} with a relative index",
+    }
     NO_SE_PUDO_LANZAR_GIT {
         es: "no se pudo lanzar git: {error}",
         en: "could not run git: {error}",
diff --git a/crates/tc2/src/i18n/catalogo/plataforma.rs b/crates/tc2/src/i18n/catalogo/plataforma.rs
index 612dd15..81720e8 100644
--- a/crates/tc2/src/i18n/catalogo/plataforma.rs
+++ b/crates/tc2/src/i18n/catalogo/plataforma.rs
@@ -160,6 +160,10 @@ crate::i18n::textos! {
         es: "(ninguno)",
         en: "(none)",
     }
+    PERMISO_PR_Y_CHECKS {
+        es: "pull_requests:read y checks:read",
+        en: "pull_requests:read and checks:read",
+    }
     PERMISO_LECTURA_GITLAB {
         es: "el scope api (o read_api) y el rol Developer en el proyecto",
         en: "the api scope (or read_api) and the Developer role in the project",
diff --git a/crates/tc2/src/i18n/mod.rs b/crates/tc2/src/i18n/mod.rs
index ff78158..acce58e 100644
--- a/crates/tc2/src/i18n/mod.rs
+++ b/crates/tc2/src/i18n/mod.rs
@@ -160,6 +160,14 @@ impl Mensaje {
                         Some((_, Arg::Texto(t))) => s.push_str(t),
                         Some((_, Arg::Mensaje(m))) => s.push_str(&m.texto(idioma)),
                         None => {
+                            // Un hueco sin valor es un fallo de quien arma el
+                            // mensaje: en los tests (debug) revienta; en
+                            // release se deja visible, sin perder el resto.
+                            debug_assert!(
+                                false,
+                                "{}: falta el valor del hueco {{{nombre}}}",
+                                self.texto.clave
+                            );
                             let _ = write!(s, "{{{nombre}}}");
                         }
                     }
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 7c98b7f..d6cdf58 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -240,4 +240,30 @@ mod tests {
 "
         );
     }
+
+    #[test]
+    fn new_y_actualizar_rama_en_ingles_mismo_stdout_y_codigo() {
+        for args in [
+            vec!["new"],
+            vec!["new", "a\tb"],
+            vec!["new", "--json", "a", "b"],
+            vec!["actualizar-rama", "de-mas"],
+            vec!["guardar"],
+        ] {
+            let es = ejecutar_en(Idioma::Es, &args);
+            let en = ejecutar_en(Idioma::En, &args);
+            assert_eq!(es.codigo, en.codigo, "{args:?}");
+            assert!(!en.stderr.is_empty(), "{args:?}");
+            assert_ne!(es.stderr, en.stderr, "{args:?}: stderr sin traducir");
+            if args.contains(&"--json") {
+                assert!(en.stdout.contains("\"diagnostico\":"), "{args:?}");
+            } else {
+                assert_eq!(es.stdout, en.stdout, "{args:?}");
+            }
+        }
+        assert_eq!(
+            ejecutar_en(Idioma::En, ["new"]).stderr,
+            "usage: new [--json] \"<title>\"\n"
+        );
+    }
 }
diff --git a/crates/tc2/src/plataforma/github.rs b/crates/tc2/src/plataforma/github.rs
index e65ba70..5ee1adf 100644
--- a/crates/tc2/src/plataforma/github.rs
+++ b/crates/tc2/src/plataforma/github.rs
@@ -326,7 +326,8 @@ impl Plataforma for GitHub {
         {
             return i;
         }
-        let permiso = "pull_requests:read y checks:read";
+        // Dos alcances unidos por una conjuncion: se traduce (IMP-1 de F2-8).
+        let permiso = &t::PERMISO_PR_Y_CHECKS;
         let p: PrApi = match self.con.leer(op, rol, permiso, self.ep(&Endpoint::Pr(pr))) {
             Ok(p) => p,
             Err(i) => return i,
````

## Excluido del diff (5 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff a6713930b5f17122ab4d850a1293fc185328c47d..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-024/planificacion/brainstorm/peticion-plan-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-024/planificacion/plan-final.md                 |    0
 tareas/03-en-revision/TASK-024/revision/informe-revision-1-rust-wasm-reviewer.md            |   21 +
 tareas/03-en-revision/TASK-024/revision/peticion-revision-1-rust-wasm-reviewer.md           | 4682 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-024/tarea.md                                    |   12 +-
 5 files changed, 4712 insertions(+), 3 deletions(-)
````
