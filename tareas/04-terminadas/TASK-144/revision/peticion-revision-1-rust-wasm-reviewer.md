# Peticion de revision — TASK-144 (ronda 1)

- Tarea: TASK-144 — Auditoria 4D: A6 estimacion, progreso, roadmap y tablero (6 IMPORTANTE)
- Rama revisada: fix/task-144-auditoria-4d-a6-estimacion-progreso-road
- Rama base: develop
- Commit revisado (HEAD): 67f6db4fc47f3b152df82d2002f75c14b5f9669f
- Fecha: 2026-11-25
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-144 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-144/tarea.md
- tareas/03-en-revision/TASK-144/planificacion/
- crates/tc2/src/grafo.rs
- crates/tc2/src/progreso/mod.rs
- crates/tc2/tests/comun/mod.rs
- crates/tc2/tests/comun/ops.rs
- crates/tc2/tests/comun/remoto.rs
- crates/tc2/tests/comun/tareas.rs
- crates/tc2/tests/doctor.rs
- crates/tc2/tests/estimacion.rs
- crates/tc2/tests/progreso.rs
- crates/tc2/tests/recursos.rs
- crates/tc2/tests/roadmap.rs
- crates/tc2/tests/siguiente.rs
- crates/tc2/tests/solape.rs
- crates/tc2/tests/tablero.rs

## Commits a revisar (git log develop..HEAD)

````
67f6db4 docs(TASK-144): resultado, criterios y autorrevision
250cf46 fix(TASK-144): A6-07 c7_fuera_de_un_repo prueba el respaldo a la carpeta del proceso
e40a036 fix(TASK-144): A6-06 c5_sin_red_y_sin_origin_sin_claves llama a start_de_luis
3a27379 fix(TASK-144): A6-03 Poner, fila y terminada en el modulo comun de tests
2a027aa fix(TASK-144): A6-02 linea_esperada usa ids_de_la_linea
e5a3dd8 fix(TASK-144): A6-04 juego de tareas de next y publicacion de estado en comun
5473d94 chore(TASK-144): tarea en curso
````

## Diff de tu dominio (14 fichero(s) de 14; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/grafo.rs b/crates/tc2/src/grafo.rs
index 38e7f18..336feff 100644
--- a/crates/tc2/src/grafo.rs
+++ b/crates/tc2/src/grafo.rs
@@ -640,19 +640,7 @@ mod tests {
 
     /// IDs de la linea `- **<titulo>:** TASK-1 (x), TASK-2` del ROADMAP.
     fn linea_esperada(titulo: &str) -> Vec<String> {
-        let prefijo = format!("- **{titulo}:** ");
-        let resto = TW_ESPERADO
-            .lines()
-            .find_map(|l| l.trim_end_matches('\r').strip_prefix(prefijo.as_str()))
-            .unwrap_or_default();
-        if resto == "ninguna" {
-            return Vec::new();
-        }
-        resto
-            .split(", ")
-            .filter_map(|p| p.split(' ').next())
-            .map(str::to_owned)
-            .collect()
+        crate::roadmap::generar::ids_de_la_linea(TW_ESPERADO, titulo).unwrap_or_default()
     }
 
     /// C1: el nucleo da exactamente las listas del ROADMAP de ToolsaWeb,
diff --git a/crates/tc2/src/progreso/mod.rs b/crates/tc2/src/progreso/mod.rs
index 370bc40..03f0cda 100644
--- a/crates/tc2/src/progreso/mod.rs
+++ b/crates/tc2/src/progreso/mod.rs
@@ -477,23 +477,42 @@ mod tests {
     }
 
     /// C7: fuera de un repo, sin tarea; el JSON que no vale usa la carpeta
-    /// del proceso.
+    /// del proceso (adenda de TASK-084 en F1-17): con una tarea en curso en
+    /// ella, da su linea.
     #[test]
-    fn c7_fuera_de_un_repo() {
-        let d = dir("fuera");
-        let _ = std::fs::create_dir_all(&d);
-        // La carpeta temporal no esta dentro de un repo (si lo estuviera, el
-        // test no probaria nada: se comprueba).
-        if buscar(&d).is_none() {
-            let json = format!(
-                "{{\"workspace\":{{\"current_dir\":{}}}}}",
-                serde_json::Value::from(d.display().to_string())
-            );
-            assert_eq!(orden_estado(&json, None, 0), linea::SIN_TAREA);
-            assert_eq!(orden_estado("no es json", Some(&d), 0), linea::SIN_TAREA);
-        }
+    fn c7_fuera_de_un_repo() -> std::io::Result<()> {
+        let raiz = dir("fuera");
+        let fuera = raiz.join("fuera");
+        std::fs::create_dir_all(&fuera)?;
+        assert!(
+            buscar(&fuera).is_none(),
+            "{} esta dentro de un repo: el test no probaria nada",
+            fuera.display()
+        );
+        let json = format!(
+            "{{\"workspace\":{{\"current_dir\":{}}}}}",
+            serde_json::Value::from(fuera.display().to_string())
+        );
+        assert_eq!(orden_estado(&json, None, 0), linea::SIN_TAREA);
+        assert_eq!(
+            orden_estado("no es json", Some(&fuera), 0),
+            linea::SIN_TAREA
+        );
         assert_eq!(orden_estado("", None, 0), linea::SIN_TAREA);
-        let _ = std::fs::remove_dir_all(&d);
+        // La carpeta del proceso es un principal con el progreso de una
+        // tarea: el JSON que no vale da la linea de esa carpeta.
+        let principal = raiz.join("p");
+        let comun = principal.join(".git");
+        std::fs::create_dir_all(&comun)?;
+        fichero::escribir(
+            &fichero::carpeta(&comun),
+            &fichero::tests::ejemplo("TASK-001"),
+        )?;
+        let esperada = linea_en(&principal, 0);
+        assert_ne!(esperada, linea::SIN_TAREA);
+        assert_eq!(orden_estado("no es json", Some(&principal), 0), esperada);
+        let _ = std::fs::remove_dir_all(&raiz);
+        Ok(())
     }
 
     /// C9 (biblioteca): `seguir` imprime la linea inicial, otra al
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/01-en-diseno/TASK-144/tarea.md                                                     | 48 ------------------------------------------------
 tareas/{01-en-diseno => 02-en-curso}/TASK-144/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-144/planificacion/plan-final.md                 |  0
 tareas/02-en-curso/TASK-144/tarea.md                                                      | 78 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 4 files changed, 78 insertions(+), 48 deletions(-)
````

## Tests del diff (12 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/comun/mod.rs" ":(literal)crates/tc2/tests/comun/ops.rs" ":(literal)crates/tc2/tests/comun/remoto.rs" ":(literal)crates/tc2/tests/comun/tareas.rs" ":(literal)crates/tc2/tests/doctor.rs" ":(literal)crates/tc2/tests/estimacion.rs" ":(literal)crates/tc2/tests/progreso.rs" ":(literal)crates/tc2/tests/recursos.rs" ":(literal)crates/tc2/tests/roadmap.rs" ":(literal)crates/tc2/tests/siguiente.rs" ":(literal)crates/tc2/tests/solape.rs" ":(literal)crates/tc2/tests/tablero.rs"
````

````
crates/tc2/tests/comun/mod.rs    |  1 +
 crates/tc2/tests/comun/ops.rs    | 22 ++++++++++++++++++++++
 crates/tc2/tests/comun/remoto.rs | 27 ++++++++++++++++++++++++++-
 crates/tc2/tests/comun/tareas.rs | 98 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/doctor.rs       | 29 +++++++++++------------------
 crates/tc2/tests/estimacion.rs   | 83 +++++++++++++++++------------------------------------------------------------------
 crates/tc2/tests/progreso.rs     | 72 ++++++++----------------------------------------------------------------
 crates/tc2/tests/recursos.rs     | 19 +++++++------------
 crates/tc2/tests/roadmap.rs      | 61 +++++++++----------------------------------------------------
 crates/tc2/tests/siguiente.rs    | 61 ++++---------------------------------------------------------
 crates/tc2/tests/solape.rs       | 18 +-----------------
 crates/tc2/tests/tablero.rs      | 31 +++++++++----------------------
 12 files changed, 213 insertions(+), 309 deletions(-)
````
## Candidatos 4D

Pistas generadas desde el diff, no hallazgos: comprueba cada una (Regla 4D de `revision.md`).

Sin candidatos: ninguna definicion nueva se parece a otra y ningun test nuevo promete algo que haya que mutar.

