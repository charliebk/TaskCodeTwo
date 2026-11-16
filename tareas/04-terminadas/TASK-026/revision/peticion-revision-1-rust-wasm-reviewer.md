# Peticion de revision — TASK-026 (ronda 1)

- Tarea: TASK-026 — F2-11 Instaladores de una linea
- Rama revisada: feature/task-026-f2-11-instaladores-de-una-linea
- Rama base: develop
- Commit revisado (HEAD): 6609149e64b240c697b4eba4036d5e8006865279
- Fecha: 2026-11-16
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-026 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-026/tarea.md
- tareas/03-en-revision/TASK-026/planificacion/
- xtask/src/instaladores.rs
- xtask/src/main.rs

## Commits a revisar (git log develop..HEAD)

````
6609149 chore(TASK-026): constancia de autorrevision
f309ec2 fix(TASK-026): TC2_INSTALL_DIR relativo se hace absoluto antes de ir al PATH
4e365f9 fix(TASK-026): el coste de PowerShell del runner entra en la medida
29ebd59 fix(TASK-026): medir en Windows con el PSModulePath de un Windows limpio
a359612 test(TASK-026): desglose de la carga del modulo de irm en Windows
15042a8 fix(TASK-026): el arnes no cambia LOCALAPPDATA al medir en Windows
c886987 test(TASK-026): desglose del tiempo de instalacion en Windows
5db2dfe fix(TASK-026): el primer arranque de powershell.exe queda fuera de la medida
ab7a9cb feat(TASK-026): instaladores de una linea install.sh e install.ps1
fe585a6 chore(TASK-026): tarea en curso
````

## Diff de tu dominio (2 fichero(s) de 9; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/xtask/src/instaladores.rs b/xtask/src/instaladores.rs
new file mode 100644
index 0000000..02bcc86
--- /dev/null
+++ b/xtask/src/instaladores.rs
@@ -0,0 +1,90 @@
+//! Pruebas de los instaladores de una linea de `instalar/` (TASK-026, F2-11).
+//!
+//! Los scripts repiten la tabla de alias de P12 (no se generan): estas
+//! pruebas exigen que sea la de `dist::TARGETS`. Ademas, que sean ASCII (con
+//! `irm | iex`, Windows PowerShell 5.1 puede decodificar mal una tilde), que
+//! `install.sh` tenga finales LF y que todo vaya dentro de una funcion que se
+//! llama en la ultima linea (una descarga cortada no ejecuta medio script).
+#![allow(clippy::panic)] // modulo solo de test: panic! es la forma de fallar
+
+use std::collections::BTreeSet;
+use std::path::PathBuf;
+
+use crate::dist::TARGETS;
+
+fn leer(nombre: &str) -> String {
+    let ruta = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
+        .join("..")
+        .join("instalar")
+        .join(nombre);
+    std::fs::read_to_string(&ruta).unwrap_or_else(|e| panic!("{}: {e}", ruta.display()))
+}
+
+/// Alias de la forma de P12 que aparecen en `texto`: `win-`, `darwin-` o
+/// `linux-` al principio de una palabra, seguido de minusculas y digitos.
+fn alias_en(texto: &str) -> BTreeSet<String> {
+    let mut encontrados = BTreeSet::new();
+    for prefijo in ["win-", "darwin-", "linux-"] {
+        for (i, _) in texto.match_indices(prefijo) {
+            let a_mitad_de_palabra = texto[..i]
+                .chars()
+                .next_back()
+                .is_some_and(|c| c.is_ascii_alphanumeric());
+            if a_mitad_de_palabra {
+                continue;
+            }
+            let resto = &texto[i + prefijo.len()..];
+            let fin = resto
+                .find(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit()))
+                .unwrap_or(resto.len());
+            encontrados.insert(format!("{prefijo}{}", &resto[..fin]));
+        }
+    }
+    encontrados
+}
+
+fn alias_de_p12(windows: bool) -> BTreeSet<String> {
+    TARGETS
+        .iter()
+        .filter(|t| t.es_windows() == windows)
+        .map(|t| t.alias.to_owned())
+        .collect()
+}
+
+#[test]
+fn alias_en_solo_cuenta_palabras_enteras() {
+    let encontrados = alias_en("x linux-x64) twin-x win-arm64.exe 'darwin-' ");
+    let esperado: BTreeSet<String> = ["darwin-", "linux-x64", "win-arm64"]
+        .iter()
+        .map(|s| (*s).to_owned())
+        .collect();
+    assert_eq!(encontrados, esperado);
+}
+
+#[test]
+fn la_tabla_de_alias_de_los_instaladores_es_la_de_p12() {
+    assert_eq!(alias_en(&leer("install.sh")), alias_de_p12(false));
+    assert_eq!(alias_en(&leer("install.ps1")), alias_de_p12(true));
+}
+
+#[test]
+fn instaladores_ascii_y_con_la_llamada_en_la_ultima_linea() {
+    for (nombre, llamada) in [
+        ("install.sh", "main \"$@\""),
+        ("install.ps1", "Install-Tc2"),
+    ] {
+        let texto = leer(nombre);
+        assert!(texto.is_ascii(), "{nombre} tiene caracteres no ASCII");
+        let ultima = texto.lines().rev().find(|l| !l.trim().is_empty());
+        assert_eq!(ultima.map(str::trim), Some(llamada), "{nombre}");
+    }
+    assert!(
+        !leer("install.sh").contains('\r'),
+        "install.sh tiene finales CRLF"
+    );
+    // Con `irm | iex`, un `exit` cerraria la ventana de la persona.
+    let sin_exit = leer("install.ps1")
+        .lines()
+        .all(|l| !l.trim_start().to_ascii_lowercase().starts_with("exit"));
+    assert!(sin_exit, "install.ps1 usa exit");
+}
diff --git a/xtask/src/main.rs b/xtask/src/main.rs
index 56c7f3c..c0ee217 100644
--- a/xtask/src/main.rs
+++ b/xtask/src/main.rs
@@ -20,6 +20,8 @@
 //! El resultado va en el codigo de salida: 0 verde, 1 rojo, 2 mal invocado.
 
 mod dist;
+#[cfg(test)]
+mod instaladores;
 mod sha256;
 
 use std::path::{Path, PathBuf};
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-026/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-026/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-026/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````
