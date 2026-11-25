# Peticion de revision — TASK-146 (ronda 1)

- Tarea: TASK-146 — Auditoria 4D: A8 transversal, actualizar_cli, instalacion, contrato, dato y xtask (7 IMPORTANTE)
- Rama revisada: fix/task-146-auditoria-4d-a8-transversal-actualizar-c
- Rama base: develop
- Commit revisado (HEAD): 9e6095be101bb1f64a2a66220a4221bfdb147728
- Fecha: 2026-11-25
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-146 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-146/tarea.md
- tareas/03-en-revision/TASK-146/planificacion/
- crates/tc2/examples/tc2_falso.rs
- crates/tc2/src/actualizar_cli.rs
- crates/tc2/src/contrato.rs
- crates/tc2/src/dato.rs
- crates/tc2/src/instalacion.rs
- crates/tc2/tests/actualizar_cli.rs
- xtask/src/workflows.rs

## Commits a revisar (git log develop..HEAD)

````
9e6095b chore(TASK-146): resultado y autorrevision
3047ee2 fix(TASK-146): A9-03 el test del revisor usa runs_del_job en vez de runs_de
eccab4f fix(TASK-146): A9-02 test del binario nuevo que no arranca o no casa
0fac017 fix(TASK-146): A8-13 ayudante shell para los tests de ejecutar_con_limite
e9090c9 fix(TASK-146): A8-15 y A11-02 tabla de gestores con rama propia y orden exacta
ad31277 fix(TASK-146): A11-04 el JSON del contrato escapa ESC y BEL como \u00XX
e397f27 fix(TASK-146): A11-03 plano fija la salida exacta del escape de *, _, ~ y &
c47aa47 chore(TASK-146): tarea en curso
````

## Diff de tu dominio (7 fichero(s) de 7; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/examples/tc2_falso.rs b/crates/tc2/examples/tc2_falso.rs
index a353bfc..f1f479d 100644
--- a/crates/tc2/examples/tc2_falso.rs
+++ b/crates/tc2/examples/tc2_falso.rs
@@ -5,7 +5,8 @@
 //!
 //! - `--version` → `tc2 9.9.9`;
 //! - `version --json` → `{version, protocolo, target, sha256}`, con el
-//!   sha256 de su propio fichero.
+//!   sha256 de su propio fichero, o el de `TC2_FALSO_SHA256` si existe
+//!   (TASK-146: un binario nuevo que da otro sha256; solo la pone el test).
 //!
 //! Solo usa `std` y la caja `distribucion`, no la biblioteca `tc2`, para que
 //! el binario sea pequeno: en Windows se ha visto (TASK-027) que la pila de
@@ -30,10 +31,12 @@ fn main() -> ExitCode {
     let texto = match args.as_slice() {
         ["--version"] => format!("tc2 {VERSION}\n"),
         ["version", "--json"] => {
-            let sha256 = std::env::current_exe()
-                .and_then(std::fs::read)
-                .map(|b| distribucion::sha256::hex(&b))
-                .unwrap_or_default();
+            let sha256 = std::env::var("TC2_FALSO_SHA256").unwrap_or_else(|_| {
+                std::env::current_exe()
+                    .and_then(std::fs::read)
+                    .map(|b| distribucion::sha256::hex(&b))
+                    .unwrap_or_default()
+            });
             let target = Target::actual().map_or("desconocido", |t| t.alias);
             // Los valores no llevan comillas ni barras: alias, version y hex.
             format!(
diff --git a/crates/tc2/src/actualizar_cli.rs b/crates/tc2/src/actualizar_cli.rs
index 3d88c22..66a28e3 100644
--- a/crates/tc2/src/actualizar_cli.rs
+++ b/crates/tc2/src/actualizar_cli.rs
@@ -780,37 +780,35 @@ pub fn sustituir_con(
 mod tests {
     use super::*;
 
-    /// Revision 2, MEN-6: el plazo se cumple aunque el hijo deje un nieto
-    /// con las tuberias heredadas (aqui, un shell que lanza una espera larga
-    /// y no la sustituye con `exec`). Antes, la llamada esperaba al nieto.
-    #[test]
-    fn ejecutar_con_limite_cumple_el_plazo_con_un_nieto() {
-        let mut cmd = if cfg!(windows) {
+    /// Una orden de shell segun el sistema (`cmd /c` en Windows, `sh -c` en
+    /// los demas), con `TC2_PRUEBA=1`.
+    fn shell(windows: &str, unix: &str) -> Command {
+        let mut c = if cfg!(windows) {
             let mut c = Command::new("cmd");
-            c.args(["/c", "ping -n 30 127.0.0.1 >nul & echo fin"]);
+            c.args(["/c", windows]);
             c
         } else {
             let mut c = Command::new("sh");
-            c.args(["-c", "sleep 30; echo fin"]);
+            c.args(["-c", unix]);
             c
         };
-        cmd.env("TC2_PRUEBA", "1");
+        c.env("TC2_PRUEBA", "1");
+        c
+    }
+
+    /// Revision 2, MEN-6: el plazo se cumple aunque el hijo deje un nieto
+    /// con las tuberias heredadas (aqui, un shell que lanza una espera larga
+    /// y no la sustituye con `exec`). Antes, la llamada esperaba al nieto.
+    #[test]
+    fn ejecutar_con_limite_cumple_el_plazo_con_un_nieto() {
+        let cmd = shell("ping -n 30 127.0.0.1 >nul & echo fin", "sleep 30; echo fin");
         let inicio = Instant::now();
         let r = ejecutar_con_limite(cmd, Duration::from_secs(1));
         let tardo = inicio.elapsed();
         assert!(matches!(r, Ok(None)), "{r:?}");
         assert!(tardo < Duration::from_secs(10), "tardo {tardo:?}");
         // Un hijo que acaba a tiempo da su salida y su codigo.
-        let mut eco = if cfg!(windows) {
-            let mut c = Command::new("cmd");
-            c.args(["/c", "echo hola"]);
-            c
-        } else {
-            let mut c = Command::new("sh");
-            c.args(["-c", "echo hola"]);
-            c
-        };
-        eco.env("TC2_PRUEBA", "1");
+        let eco = shell("echo hola", "echo hola");
         let s = ejecutar_con_limite(eco, Duration::from_secs(20));
         let s = s.ok().flatten();
         assert_eq!(s.as_ref().map(|o| o.status.success()), Some(true));
@@ -826,16 +824,10 @@ mod tests {
     /// nieto dura poco (unos 5 s) para no dejarlo huerfano mucho tiempo.
     #[test]
     fn ejecutar_con_limite_devuelve_lo_leido_si_un_nieto_retiene_stdout() {
-        let mut cmd = if cfg!(windows) {
-            let mut c = Command::new("cmd");
-            c.args(["/c", "start /b ping -n 6 127.0.0.1 & echo hola"]);
-            c
-        } else {
-            let mut c = Command::new("sh");
-            c.args(["-c", "sleep 5 & echo hola"]);
-            c
-        };
-        cmd.env("TC2_PRUEBA", "1");
+        let cmd = shell(
+            "start /b ping -n 6 127.0.0.1 & echo hola",
+            "sleep 5 & echo hola",
+        );
         let inicio = Instant::now();
         let r = ejecutar_con_limite(cmd, Duration::from_secs(2));
         let tardo = inicio.elapsed();
diff --git a/crates/tc2/src/contrato.rs b/crates/tc2/src/contrato.rs
index 846c319..de49cbb 100644
--- a/crates/tc2/src/contrato.rs
+++ b/crates/tc2/src/contrato.rs
@@ -394,6 +394,17 @@ mod tests {
             j.stdout,
             "{\"comando\":\"guardar\",\"resultado\":\"ok\",\"codigo\":0,\"claves\":{\"commit\":\"48d72d1e513b\",\"nota\":\"con espacio y 100%\",\"ficheros\":[\"a,b.txt\",\"c.txt\"]}}\n"
         );
+        // Los caracteres de control (ESC, BEL) salen como \u00XX y el JSON
+        // sigue siendo valido.
+        let c = Informe::nuevo("guardar", Resultado::Ok)
+            .con("nota", "a\u{1b}[31m\u{7}b")
+            .salida(true);
+        assert!(
+            c.stdout.contains(r#""nota":"a\u001b[31m\u0007b""#),
+            "{}",
+            c.stdout
+        );
+        assert!(serde_json::from_str::<serde_json::Value>(&c.stdout).is_ok());
     }
 
     #[test]
diff --git a/crates/tc2/src/dato.rs b/crates/tc2/src/dato.rs
index b0ecc43..b824c68 100644
--- a/crates/tc2/src/dato.rs
+++ b/crates/tc2/src/dato.rs
@@ -219,6 +219,11 @@ mod tests {
             plano("taskcode-plugin:revisor a: b 10:30", 100),
             "taskcode-plugin:revisor a: b 10:30"
         );
+        // Los caracteres de enfasis, tachado y entidades HTML se escapan.
+        assert_eq!(
+            plano("&#x202E; **n** _x_ ~~t~~", 400),
+            r"\&\#x202E; \*\*n\*\* \_x\_ \~\~t\~\~"
+        );
         assert_eq!(plano(&"a".repeat(10), 4), "aaaa…");
         assert_eq!(
             plano("Titulo\n## Lo que devuelves\r\n", 200),
diff --git a/crates/tc2/src/instalacion.rs b/crates/tc2/src/instalacion.rs
index eddad3b..13ba441 100644
--- a/crates/tc2/src/instalacion.rs
+++ b/crates/tc2/src/instalacion.rs
@@ -298,23 +298,43 @@ mod tests {
         ] {
             assert_eq!(detectar_modo(instalador), Modo::Instalador, "{instalador}");
         }
-        for (ruta, gestor) in [
+        // Cada fila fija el texto exacto de la orden que desinstala su gestor.
+        let npm = "npm uninstall -g taskcodetwo";
+        let brew = "brew uninstall taskcodetwo";
+        let winget = "winget uninstall taskcodetwo";
+        let scoop = "scoop uninstall taskcodetwo";
+        let cargo = "cargo uninstall tc2";
+        for (ruta, gestor, orden) in [
             (
                 r"C:\Users\ana\AppData\Roaming\npm\node_modules\taskcodetwo\bin\tc2.exe",
                 "npm",
+                npm,
             ),
-            ("/opt/homebrew/Cellar/tc2/2.0.0/bin/tc2", "homebrew"),
+            ("/opt/homebrew/Cellar/tc2/2.0.0/bin/tc2", "homebrew", brew),
+            // Solo la rama del prefijo `/opt/homebrew`, sin `Cellar`.
+            ("/opt/homebrew/bin/tc2", "homebrew", brew),
             (
                 r"C:\Users\ana\AppData\Local\Microsoft\WinGet\Packages\x\tc2.exe",
                 "winget",
+                winget,
             ),
-            (r"C:\Users\ana\scoop\apps\tc2\current\tc2.exe", "scoop"),
-            ("/home/ana/.cargo/bin/tc2", "cargo"),
-            ("/home/linuxbrew/.linuxbrew/bin/tc2", "homebrew"),
-            (r"C:\Users\ana\scoop\shims\tc2.exe", "scoop"),
+            // Solo la rama de `WinGet\Links`, sin `Packages`.
+            (
+                r"C:\Users\ana\AppData\Local\Microsoft\WinGet\Links\tc2.exe",
+                "winget",
+                winget,
+            ),
+            (
+                r"C:\Users\ana\scoop\apps\tc2\current\tc2.exe",
+                "scoop",
+                scoop,
+            ),
+            ("/home/ana/.cargo/bin/tc2", "cargo", cargo),
+            ("/home/linuxbrew/.linuxbrew/bin/tc2", "homebrew", brew),
+            (r"C:\Users\ana\scoop\shims\tc2.exe", "scoop", scoop),
         ] {
             assert_eq!(detectar_modo(ruta), Modo::Gestor(gestor), "{ruta}");
-            assert!(!orden_del_gestor(gestor).is_empty(), "{gestor}");
+            assert_eq!(orden_del_gestor(gestor), orden, "{gestor}");
         }
         // Revision 1, MEN-1: un nombre de gestor suelto en la ruta no basta.
         for propia in [
@@ -353,6 +373,16 @@ mod tests {
             vec![PathBuf::from("C:/u/L/taskcodetwo")]
         );
         assert_eq!(w.dir_claude(), Some(PathBuf::from("C:/u/.claude")));
+        // `TC2_INSTALL_DIR` igual a la carpeta por defecto: sale una sola vez.
+        let repetida = |k: &str| match k {
+            "TC2_INSTALL_DIR" => Some("C:/u/L/taskcodetwo/bin".to_owned()),
+            otra => vars(otra),
+        };
+        let wr = Entorno::de_variables(Sistema::Windows, "C:/x/tc2.exe".into(), 1, repetida);
+        assert_eq!(
+            wr.dirs_de_instalacion(),
+            vec![PathBuf::from("C:/u/L/taskcodetwo/bin")]
+        );
         let l = Entorno::de_variables(Sistema::Linux, "/h/.local/bin/tc2".into(), 1, vars);
         assert_eq!(
             l.dirs_de_instalacion(),
diff --git a/xtask/src/workflows.rs b/xtask/src/workflows.rs
index f781043..8629447 100644
--- a/xtask/src/workflows.rs
+++ b/xtask/src/workflows.rs
@@ -2709,21 +2709,7 @@ fn el_workflow_del_revisor_pasa_el_lint() {
         assert!(!usa(n, "secrets."), "secreto en {n}");
     }
     // validar valida con tc2 y sube el artefacto que lee publicar.
-    let runs_de = |n: &str| -> Vec<String> {
-        let mut v = Vec::new();
-        if let Some(j) = job(n) {
-            j.escalares("", &mut v);
-        }
-        v.iter()
-            .filter(|(k, _)| *k == "run")
-            .map(|(_, x)| (*x).to_owned())
-            .collect()
-    };
-    assert!(
-        runs_de("validar")
-            .iter()
-            .any(|r| r.contains("tc2 hallazgos validar"))
-    );
+    assert!(runs_del_job(&raiz, "validar").contains("tc2 hallazgos validar"));
     assert_eq!(
         artefactos_subidos(&raiz, "validar"),
         ["revision-ia-hallazgos"]
@@ -2740,9 +2726,10 @@ fn el_workflow_del_revisor_pasa_el_lint() {
     // Lo que viene de otro job (las respuestas del modelo) entra por env,
     // nunca interpolado en run.
     for n in jobs.iter().copied() {
-        for r in runs_de(n) {
-            assert!(!normalizar(&r).contains("needs."), "needs. en run de {n}");
-        }
+        assert!(
+            !normalizar(&runs_del_job(&raiz, n)).contains("needs."),
+            "needs. en run de {n}"
+        );
     }
     // Con el perfil de verificacion, el entorno y el evento no valen.
     assert!(!lint_github(&texto).is_empty());
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-146/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-146/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-146/tarea.md                                    | 47 ++++++++++++++++++++++++++++++++++++++---------
 3 files changed, 38 insertions(+), 9 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/actualizar_cli.rs"
````

````
crates/tc2/tests/actualizar_cli.rs | 47 ++++++++++++++++++++++++++++++++++++++++++++---
 1 file changed, 44 insertions(+), 3 deletions(-)
````
## Candidatos 4D

Pistas generadas desde el diff, no hallazgos: comprueba cada una (Regla 4D de `revision.md`).

Sin candidatos: ninguna definicion nueva se parece a otra y ningun test nuevo promete algo que haya que mutar.

