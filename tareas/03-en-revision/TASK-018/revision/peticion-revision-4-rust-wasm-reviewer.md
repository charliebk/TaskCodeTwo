# Peticion de revision — TASK-018 (ronda 4)

- Tarea: TASK-018 — F2-3 Rama de estado
- Rama revisada: feature/task-018-f2-3-rama-de-estado
- Rama base: develop
- Commit revisado (HEAD): b2f565d50585629c79df41974f1d7af5ee648930
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-018 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 36d96aab6cbf310f59acaea255107de76aa9218e (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (2 hallazgo(s)))

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
informe de esta ronda (informe-revision-4-rust-wasm-reviewer.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-018/tarea.md
- tareas/03-en-revision/TASK-018/planificacion/
- crates/tc2/src/estado/transaccion.rs
- crates/tc2/src/git/remoto.rs
- crates/tc2/tests/estado_bordes.rs
- docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md

## Hallazgos de la ronda 3 pendientes de verificar

Tus 1 hallazgo(s) MENOR de la ronda 3 no se reverifican (los comprueba
quien coordina): recibes esta ronda porque la correccion
toca ficheros de tu dominio. Revisa en el diff de esta ronda lo que la correccion
cambia en tu terreno y si abre un fallo.

## Commits a revisar (git log 36d96aab6cbf310f59acaea255107de76aa9218e..HEAD)

````
b2f565d chore(TASK-018): autorrevision y ronda extra 4
965ef23 fix(TASK-018): correcciones de la revision ronda 3
83177a9 chore(TASK-018): peticion de revision ronda 3
````

## Diff desde la ronda anterior (git diff 36d96aab6cbf310f59acaea255107de76aa9218e..HEAD)

````diff
diff --git a/crates/tc2/src/estado/transaccion.rs b/crates/tc2/src/estado/transaccion.rs
index 31f7672..01e435c 100644
--- a/crates/tc2/src/estado/transaccion.rs
+++ b/crates/tc2/src/estado/transaccion.rs
@@ -341,7 +341,11 @@ fn intento(
     anotaciones.push(anotacion.to_owned());
     if let Some(generar) = c.roadmap {
         let texto = generar(&vista).map_err(de_fallo)?;
-        vista.superponer(vec![Cambio::nuevo("ROADMAP.md", texto)]);
+        let roadmap = vec![Cambio::nuevo("ROADMAP.md", texto)];
+        if let Some(i) = duplicada(&vista.rutas(), &roadmap, op) {
+            return Ok(Intento::Hecho(i));
+        }
+        vista.superponer(roadmap);
     }
 
     let mensaje = mensaje(op.nombre(), &anotaciones);
@@ -394,16 +398,21 @@ fn intento(
 }
 
 /// Primera ruta de `cambios` que solo difiere en mayusculas de otra ya
-/// presente en `rutas` (en Windows o macOS las dos chocarian al sacarlas al
-/// worktree): (la nueva, la que estaba).
+/// presente en `rutas` o de otra del mismo lote (en Windows o macOS las dos
+/// chocarian al sacarlas al worktree): (la nueva, la otra).
 fn choque(rutas: &[String], cambios: &[Cambio]) -> Option<(String, String)> {
-    cambios.iter().find_map(|c| {
+    let mut vistas: Vec<&str> = rutas.iter().map(String::as_str).collect();
+    for c in cambios {
         let minus = c.ruta.to_lowercase();
-        rutas
+        if let Some(otra) = vistas
             .iter()
             .find(|r| **r != c.ruta && r.to_lowercase() == minus)
-            .map(|otra| (c.ruta.clone(), otra.clone()))
-    })
+        {
+            return Some((c.ruta.clone(), (*otra).to_owned()));
+        }
+        vistas.push(&c.ruta);
+    }
+    None
 }
 
 /// `rechazada motivo=ruta-duplicada` si los cambios de la operacion chocan.
diff --git a/crates/tc2/src/git/remoto.rs b/crates/tc2/src/git/remoto.rs
index 3d98c09..0cac440 100644
--- a/crates/tc2/src/git/remoto.rs
+++ b/crates/tc2/src/git/remoto.rs
@@ -70,6 +70,16 @@ impl FalloRemoto {
 pub fn clasificar_fallo_remoto(stderr: &str) -> FalloRemoto {
     let t = stderr.to_ascii_lowercase();
     let alguno = |patrones: &[&str]| patrones.iter().any(|p| t.contains(p));
+    // Marcas del transporte: curl o ssh que no llegan a conectar. Con ellas,
+    // un `permission denied` es EACCES de la red (firewall, SELinux).
+    let transporte = alguno(&[
+        "unable to access",
+        "failed to connect",
+        "connect to host",
+        "could not resolve",
+        "couldn't connect",
+        "connection",
+    ]);
     if alguno(&["couldn't find remote ref", "could not find remote ref"]) {
         FalloRemoto::RamaInexistente
     } else if alguno(&[
@@ -83,7 +93,8 @@ pub fn clasificar_fallo_remoto(stderr: &str) -> FalloRemoto {
         "remote: permission to ",
         "access denied",
     ]) || (t.contains("permission denied")
-        && t.contains("could not read from remote repository"))
+        && t.contains("could not read from remote repository")
+        && !transporte)
     {
         // `permission denied` suelto tambien lo da el sistema de ficheros
         // local (no poder escribir en `.git`): solo cuenta como credenciales
@@ -96,9 +107,10 @@ pub fn clasificar_fallo_remoto(stderr: &str) -> FalloRemoto {
         "returned error: 404",
     ]) {
         FalloRemoto::RemotoNoExiste
-    } else if t.contains("permission denied") {
-        // Sin las marcas del remoto de arriba, es el sistema de ficheros del
-        // clon (`cannot open .git/FETCH_HEAD: Permission denied`).
+    } else if t.contains("permission denied") && !transporte {
+        // Sin las marcas del remoto de arriba ni las del transporte (curl o
+        // ssh con EACCES, p. ej. un firewall, son la red), es el sistema de
+        // ficheros del clon (`cannot open .git/FETCH_HEAD: Permission denied`).
         FalloRemoto::Local
     } else {
         FalloRemoto::SinRed
@@ -591,6 +603,17 @@ mod tests {
         }
     }
 
+    #[test]
+    fn red_bloqueada_con_permiso_denegado_es_sin_red() {
+        // curl con EACCES (firewall, SELinux): es la red, no el clon.
+        for t in [
+            "fatal: unable to access 'https://host/a.git/': Failed to connect to host port 443 after 3 ms: Permission denied",
+            "ssh: connect to host github.com port 22: Permission denied\nfatal: Could not read from remote repository.",
+        ] {
+            assert_eq!(clasificar_fallo_remoto(t), FalloRemoto::SinRed, "{t}");
+        }
+    }
+
     #[test]
     fn contencion_frente_a_denegacion() {
         assert!(es_contencion(
diff --git a/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md b/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
index f9e20b7..5f98f67 100644
--- a/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
+++ b/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
@@ -457,3 +457,14 @@ non-fast-forward; se trata como contención (ver la adenda de F1-4).
   chocaría con lo ya publicado va a `rechazadas`; solo la operación actual
   puede devolver `rechazada motivo=ruta-duplicada`. Antes, la pendiente
   bloqueaba todas las escrituras del clon.
+
+### Tras la revisión de la ronda 3 de TASK-018
+
+- **`permission denied` de la red**: con marcas de transporte (curl o ssh
+  que no llegan a conectar: `unable to access`, `failed to connect`,
+  `connect to host`, `could not resolve`, `connection`) es `sinred`, aunque
+  traiga «permission denied» (EACCES de un firewall o SELinux). `error-local`
+  queda para el sistema de ficheros del clon.
+- **Choque por mayúsculas dentro del lote**: dos rutas de la misma operación
+  que solo difieren en mayúsculas también dan `ruta-duplicada`, y el
+  `ROADMAP.md` generado se comprueba igual.
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 36d96aab6cbf310f59acaea255107de76aa9218e..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/03-en-revision/TASK-018/revision/informe-revision-2-code-quality-reviewer.md  |  45 ++++++++--
 tareas/03-en-revision/TASK-018/revision/informe-revision-2-rust-wasm-reviewer.md     |  41 ++++++++--
 tareas/03-en-revision/TASK-018/revision/informe-revision-3-code-quality-reviewer.md  |  21 +++++
 tareas/03-en-revision/TASK-018/revision/informe-revision-3-rust-wasm-reviewer.md     |  21 +++++
 tareas/03-en-revision/TASK-018/revision/peticion-revision-3-code-quality-reviewer.md | 480 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-018/revision/peticion-revision-3-rust-wasm-reviewer.md    | 480 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-018/tarea.md                                              |   8 +-
 7 files changed, 1079 insertions(+), 17 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff 36d96aab6cbf310f59acaea255107de76aa9218e..HEAD -- ":(literal)crates/tc2/tests/estado_bordes.rs"
````

````
crates/tc2/tests/estado_bordes.rs | 41 +++++++++++++++++++++++++++++++++++++++++
 1 file changed, 41 insertions(+)
````
