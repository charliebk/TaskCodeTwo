# Peticion de revision — TASK-042 (ronda 1)

- Tarea: TASK-042 — F5-2 Cache por SHA del arbol compartida
- Rama revisada: feature/task-042-f5-2-cache-por-sha-del-arbol-compartida
- Rama base: develop
- Commit revisado (HEAD): aee1621fc055573ce506eeeb31dda3082f1027a2
- Fecha: 2026-11-19
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-042 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-042/tarea.md
- tareas/03-en-revision/TASK-042/planificacion/
- xtask/src/workflows.rs

## Commits a revisar (git log develop..HEAD)

````
aee1621 docs(TASK-042): E2E de la cache por arbol, medicion y divergencias
132a9dd fix(TASK-042): buscar con filter=all
88e7a1c fix(TASK-042): ejecucion del check-run por su check suite
c3e803c fix(TASK-042): version del workflow leida de la propia ejecucion
1028bbe feat(TASK-042): cache por SHA del arbol en el workflow de verificacion
6f87439 chore(TASK-042): tarea en curso
````

## Diff de tu dominio (1 fichero(s) de 4; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/xtask/src/workflows.rs b/xtask/src/workflows.rs
index b1373f6..212adc4 100644
--- a/xtask/src/workflows.rs
+++ b/xtask/src/workflows.rs
@@ -8,11 +8,12 @@
 //!   `merge_group` y `workflow_dispatch` (fuera `pull_request_target`,
 //!   `repository_dispatch`, `workflow_run`, comentarios...);
 //! - bloque `permissions` arriba y en cada job, sin `write-all` ni
-//!   `read-all`; un job con algun `write` es de clase privilegio;
-//! - acciones admitidas: `actions/checkout`, fijada por SHA y con
-//!   `persist-credentials: false`; en clase privilegio, sin `ref:` ni
-//!   `repository:`; y `actions/upload-artifact` (informe de F5-4), fijada
-//!   por SHA y solo en clase ejecuta;
+//!   `read-all`; un job con algun `write` es de clase privilegio; solo se
+//!   leen `contents`, `checks` y `actions` (estos dos, para `buscar` de F5-2);
+//! - acciones admitidas: `actions/checkout`, fijada por SHA, con
+//!   `persist-credentials: false` y nunca en clase privilegio; y
+//!   `actions/upload-artifact` (informe de F5-4), fijada por SHA y solo en
+//!   clase ejecuta;
 //! - ni `secrets: inherit`, ni secretos en clase ejecuta, ni
 //!   `github.event.*`, `github.head_ref` o `toJSON(github...)` en `run:`
 //!   (los datos del evento se pasan por `env:`).
@@ -190,6 +191,10 @@ fn lista(lineas: &mut [Linea], i: &mut usize, sangria: usize) -> Result<Nodo, St
                 }
                 _ => items.push(Nodo::Escalar(String::new())),
             }
+        } else if resto.starts_with('|') || resto.starts_with('>') {
+            // `- |`: un item que es un bloque literal (after_script de F5-2).
+            *i += 1;
+            items.push(Nodo::Escalar(bloque_literal(lineas, i, sangria)));
         } else if parece_clave(&resto) {
             // `- clave: valor`: el item es un mapa cuya primera clave empieza
             // donde acaba el guion; el resto de sus claves van a esa sangria.
@@ -330,6 +335,10 @@ const SUBIDA: &str = "actions/upload-artifact";
 
 const ACCIONES_ADMITIDAS: [&str; 2] = ["actions/checkout", SUBIDA];
 
+/// Alcances que un job puede leer: el codigo, y (F5-2, `buscar`) los
+/// check-runs y las ejecuciones donde vive la cache por arbol.
+const LECTURAS_ADMITIDAS: [&str; 3] = ["contents", "checks", "actions"];
+
 /// Datos del evento que el autor del PR controla: nunca dentro de `run:`.
 /// Se buscan sobre el valor normalizado (ver `normalizar`).
 const DATOS_DEL_EVENTO: [&str; 5] = [
@@ -459,6 +468,14 @@ fn lint_github(texto: &str) -> Vec<String> {
                 false
             }
         };
+        if let Some(Nodo::Mapa(pares)) = job.get("permissions") {
+            for (alcance, valor) in pares {
+                if valor.texto() == Some("read") && !LECTURAS_ADMITIDAS.contains(&alcance.as_str())
+                {
+                    errores.push(format!("job {nombre}: lectura no admitida: {alcance}"));
+                }
+            }
+        }
         // Un job que llama a otro workflow reutilizable: fijado por SHA.
         if let Some(uses) = job.get("uses").and_then(Nodo::texto) {
             let version = uses.split_once('@').map_or("", |(_, v)| v);
@@ -515,6 +532,11 @@ fn lint_github(texto: &str) -> Vec<String> {
                     "job {nombre}: checkout sin persist-credentials: false"
                 ));
             }
+            // F1-8: un job con privilegio nunca hace checkout (en un PR, sin
+            // `ref:` saca el merge ref, que es codigo del PR).
+            if privilegio {
+                errores.push(format!("job {nombre}: checkout en clase privilegio"));
+            }
             if privilegio && (opcion("ref").is_some() || opcion("repository").is_some()) {
                 errores.push(format!(
                     "job {nombre}: checkout con ref o repository en clase privilegio"
@@ -673,8 +695,19 @@ g: |
   tras vacia
 h:
 - i
+j:
+  - |
+    if x; then
+      y
+    fi
 ";
     let raiz = yaml(doc);
+    assert_eq!(
+        raiz.get("j"),
+        Some(&Nodo::Lista(vec![Nodo::Escalar(
+            "if x; then\n  y\nfi".into()
+        )]))
+    );
     assert_eq!(raiz.get("a"), Some(&Nodo::Escalar("1".into())));
     let Some(Nodo::Lista(b)) = raiz.get("b") else {
         panic!("b no es lista: {raiz:?}");
@@ -745,7 +778,7 @@ fn workflow_y_plantilla_tienen_las_mismas_entradas() {
         .into_iter()
         .filter(|k| !k.starts_with("tags-") && *k != "stage")
         .collect();
-    assert_eq!(github.len(), 6, "{github:?}");
+    assert_eq!(github.len(), 8, "{github:?}");
     assert_eq!(github, gitlab);
 }
 
@@ -1086,6 +1119,107 @@ fn el_informe_se_sube_siempre_y_solo_desde_ejecutar() {
     );
 }
 
+/// F5-2: `buscar` lee check-runs y ejecuciones sin escribir; el comando se
+/// salta con un acierto; el publicador escribe la linea `arbol=` por perfil;
+/// la plantilla deja la linea en el artefacto. Mutaciones negativas: `buscar`
+/// con escritura (hace checkout: clase privilegio con codigo del PR) y una
+/// lectura fuera de la lista.
+#[test]
+fn la_cache_por_arbol_respeta_las_clases() {
+    let texto = leer(WORKFLOW);
+    let raiz = yaml(&texto);
+    let job = |n: &str| raiz.get("jobs").and_then(|j| j.get(n));
+    let permiso = |n: &str, alcance: &str| {
+        job(n)
+            .and_then(|j| j.get("permissions"))
+            .and_then(|p| p.get(alcance))
+            .and_then(Nodo::texto)
+            .map(str::to_owned)
+    };
+    assert_eq!(permiso("buscar", "checks").as_deref(), Some("read"));
+    assert_eq!(permiso("buscar", "actions").as_deref(), Some("read"));
+    assert_eq!(permiso("ejecutar", "checks"), None);
+    let pasos = |n: &str| match job(n).and_then(|j| j.get("steps")) {
+        Some(Nodo::Lista(p)) => p.clone(),
+        _ => Vec::new(),
+    };
+    let comando = pasos("ejecutar")
+        .into_iter()
+        .find(|p| {
+            p.get("name")
+                .and_then(Nodo::texto)
+                .is_some_and(|n| n.starts_with("comando del proyecto"))
+        })
+        .unwrap_or_else(|| panic!("sin paso del comando"));
+    assert_eq!(
+        comando.get("if").and_then(Nodo::texto),
+        Some("steps.cache.outputs.acierto != 'si'")
+    );
+    let publicar = pasos("verificacion")
+        .first()
+        .and_then(|p| p.get("run"))
+        .and_then(Nodo::texto)
+        .map(str::to_owned)
+        .unwrap_or_default();
+    assert!(
+        publicar.contains(
+            "arbol=$arbol perfil=$p runner=$r clave=$k local=$huella origen=$origen $estado"
+        ),
+        "{publicar}"
+    );
+
+    let original = "      contents: read\n      checks: read\n";
+    assert!(texto.contains(original));
+    let con_escritura = texto.replacen(original, "      contents: read\n      checks: write\n", 1);
+    let errores = lint_github(&con_escritura);
+    assert!(
+        errores
+            .iter()
+            .any(|e| e.contains("job buscar: checkout en clase privilegio")),
+        "{errores:?}"
+    );
+    let otra_lectura = texto.replacen(
+        original,
+        "      contents: read\n      pull-requests: read\n",
+        1,
+    );
+    let errores = lint_github(&otra_lectura);
+    assert!(
+        errores
+            .iter()
+            .any(|e| e.contains("job buscar: lectura no admitida: pull-requests")),
+        "{errores:?}"
+    );
+
+    let plantilla_texto = leer(PLANTILLA);
+    let (_, cuerpo) = plantilla(&plantilla_texto);
+    let base = cuerpo.get(".tc2-verificacion");
+    let mut rutas = Vec::new();
+    if let Some(p) = base
+        .and_then(|b| b.get("artifacts"))
+        .and_then(|a| a.get("paths"))
+    {
+        p.escalares("paths", &mut rutas);
+    }
+    assert!(
+        rutas
+            .iter()
+            .any(|(_, r)| *r == ".tc2/cache-verificacion-*.txt"),
+        "{rutas:?}"
+    );
+    let mut despues = Vec::new();
+    if let Some(s) = base.and_then(|b| b.get("after_script")) {
+        s.escalares("after_script", &mut despues);
+    }
+    assert!(
+        despues
+            .iter()
+            .any(|(_, l)| l.contains("CI_JOB_STATUS")
+                && l.contains("cache-verificacion-$TC2_PERFIL")),
+        "{despues:?}"
+    );
+}
+
 /// `rules:if` no ve las `variables:` de los jobs: la plantilla no las usa en
 /// las reglas, y el lint lo detecta si alguien lo vuelve a hacer.
 #[test]
````

## Excluido del diff (5 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/02-en-curso/TASK-042/medicion/medir.sh                                             | 49 +++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/02-en-curso/TASK-042/medicion/salida.txt                                           | 21 +++++++++++++++++++++
 tareas/{01-en-diseno => 02-en-curso}/TASK-042/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-042/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-042/tarea.md                                    |  3 ++-
 5 files changed, 72 insertions(+), 1 deletion(-)
````
