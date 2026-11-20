# Peticion de revision — TASK-043 (ronda 1)

- Tarea: TASK-043 — F5-3 Verificacion por impacto como paso de CI y completa en la cola
- Rama revisada: feature/task-043-f5-3-verificacion-por-impacto-como-paso
- Rama base: develop
- Commit revisado (HEAD): 23039496d45451cb132c8fd041a42333355bf9a1
- Fecha: 2026-11-20
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-043 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-043/tarea.md
- tareas/03-en-revision/TASK-043/planificacion/
- xtask/src/workflows.rs

## Commits a revisar (git log develop..HEAD)

````
2303949 chore(TASK-043): medicion E2E de tiempos con y sin cola de merge
8cbac85 feat(TASK-043): entrada cola-de-merge en el workflow y la plantilla de GitLab
a7d0c4a feat(TASK-043): adenda F5-3 a F1-7 y tabla de eventos con cola de merge
7515e44 chore(TASK-043): tarea en curso
````

## Diff de tu dominio (1 fichero(s) de 5; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/xtask/src/workflows.rs b/xtask/src/workflows.rs
index 813483a..29b0a32 100644
--- a/xtask/src/workflows.rs
+++ b/xtask/src/workflows.rs
@@ -910,10 +910,268 @@ fn workflow_y_plantilla_tienen_las_mismas_entradas() {
         .into_iter()
         .filter(|k| !k.starts_with("tags-") && *k != "stage")
         .collect();
-    assert_eq!(github.len(), 8, "{github:?}");
+    assert_eq!(github.len(), 9, "{github:?}");
     assert_eq!(github, gitlab);
 }
 
+// --- Cola de merge (F5-3) ---------------------------------------------------------
+
+/// El `run:` del paso `nombre` del job `job` del workflow, tal cual.
+fn run_del_paso(texto: &str, job: &str, nombre: &str) -> String {
+    let raiz = yaml(texto);
+    let Some(Nodo::Lista(pasos)) = raiz
+        .get("jobs")
+        .and_then(|j| j.get(job))
+        .and_then(|j| j.get("steps"))
+    else {
+        panic!("job {job} sin pasos");
+    };
+    pasos
+        .iter()
+        .find(|p| p.get("name").and_then(Nodo::texto) == Some(nombre))
+        .and_then(|p| p.get("run"))
+        .and_then(Nodo::texto)
+        .unwrap_or_else(|| panic!("job {job}: sin paso {nombre}"))
+        .to_owned()
+}
+
+/// El `bash` de los runners de GitHub. En Windows, `CreateProcess` mira
+/// `System32` antes que el `PATH` y daria el `bash.exe` de WSL: se busca el
+/// de Git en el `PATH` y, si no esta, junto al `git` instalado.
+fn ruta_bash() -> PathBuf {
+    if !cfg!(windows) {
+        return PathBuf::from("bash");
+    }
+    let sistema = std::env::var("SystemRoot")
+        .unwrap_or_else(|_| r"C:\Windows".to_owned())
+        .to_lowercase();
+    let en_path = std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
+        .map(|d| d.join("bash.exe"))
+        .find(|c| {
+            let texto = c.to_string_lossy().to_lowercase();
+            c.is_file() && !texto.starts_with(&sistema) && !texto.contains("windowsapps")
+        });
+    if let Some(ruta) = en_path {
+        return ruta;
+    }
+    // `git --exec-path` = <git>/mingw64/libexec/git-core; bash en <git>/bin.
+    let salida = std::process::Command::new("git")
+        .arg("--exec-path")
+        .output()
+        .unwrap_or_else(|e| panic!("sin bash en el PATH ni git: {e}"));
+    let nucleo = PathBuf::from(String::from_utf8_lossy(&salida.stdout).trim());
+    let ruta = nucleo
+        .ancestors()
+        .nth(3)
+        .map(|g| g.join("bin").join("bash.exe"))
+        .unwrap_or_default();
+    assert!(ruta.is_file(), "sin bash de Git: {}", ruta.display());
+    ruta
+}
+
+/// Ejecuta con `bash` el `run:` de «Perfiles del evento» extraido de `texto`,
+/// con el entorno del paso simulado, y devuelve lo que escribe en
+/// `GITHUB_OUTPUT` (o el error y el codigo de salida).
+fn perfiles_del_evento(
+    texto: &str,
+    evento: &str,
+    base: &str,
+    origen: &str,
+    cola: &str,
+) -> Result<String, String> {
+    use std::sync::atomic::{AtomicUsize, Ordering};
+    static N: AtomicUsize = AtomicUsize::new(0);
+    let script = run_del_paso(texto, "buscar", "Perfiles del evento");
+    let salida = std::env::temp_dir().join(format!(
+        "tc2-xtask-perfiles-{}-{}.txt",
+        std::process::id(),
+        N.fetch_add(1, Ordering::Relaxed)
+    ));
+    std::fs::write(&salida, "").unwrap_or_else(|e| panic!("{}: {e}", salida.display()));
+    let proceso = std::process::Command::new(ruta_bash())
+        .arg("-c")
+        .arg(&script)
+        .env("EVENTO", evento)
+        .env("BASE", base)
+        .env("ORIGEN", origen)
+        .env("FORZADOS", "")
+        .env("COLA", cola)
+        .env("GITHUB_OUTPUT", &salida)
+        .output()
+        .unwrap_or_else(|e| panic!("bash no arranca (hace falta bash en el PATH): {e}"));
+    let escrito = std::fs::read_to_string(&salida).unwrap_or_default();
+    let _ = std::fs::remove_file(&salida);
+    if proceso.status.success() {
+        Ok(escrito.trim().to_owned())
+    } else {
+        Err(format!(
+            "{:?}: {}",
+            proceso.status.code(),
+            String::from_utf8_lossy(&proceso.stderr).trim()
+        ))
+    }
+}
+
+/// Adenda F5-3 de F1-7 (D1): con `cola-de-merge` el PR corre la impacto y el
+/// `merge_group` la completa; la release sigue con completa y pesada, y con
+/// `false` cada evento da lo mismo que antes de la entrada. Se ejecuta el
+/// `case` real del workflow.
+#[test]
+fn la_cola_de_merge_elige_los_perfiles_del_evento() {
+    let texto = leer(WORKFLOW);
+    let casos: [(&str, &str, &str, &str, &str); 10] = [
+        // (evento, base, origen, cola, perfiles)
+        (
+            "pull_request",
+            "develop",
+            "feature/x",
+            "false",
+            r#"["completa"]"#,
+        ),
+        (
+            "pull_request",
+            "main",
+            "develop",
+            "false",
+            r#"["completa","pesada"]"#,
+        ),
+        (
+            "pull_request",
+            "main",
+            "hotfix/x",
+            "false",
+            r#"["completa"]"#,
+        ),
+        ("merge_group", "", "", "false", r#"["completa"]"#),
+        ("push", "", "", "false", "[]"),
+        (
+            "pull_request",
+            "develop",
+            "feature/x",
+            "true",
+            r#"["impacto"]"#,
+        ),
+        (
+            "pull_request",
+            "main",
+            "develop",
+            "true",
+            r#"["completa","pesada"]"#,
+        ),
+        ("pull_request", "main", "hotfix/x", "true", r#"["impacto"]"#),
+        ("merge_group", "", "", "true", r#"["completa"]"#),
+        ("workflow_dispatch", "", "", "true", r#"["completa"]"#),
+    ];
+    for (evento, base, origen, cola, esperado) in casos {
+        assert_eq!(
+            perfiles_del_evento(&texto, evento, base, origen, cola),
+            Ok(format!("perfiles={esperado}")),
+            "{evento} {base}<-{origen} cola={cola}"
+        );
+    }
+    assert!(
+        perfiles_del_evento(&texto, "pull_request_target", "", "", "true").is_err(),
+        "un evento no admitido falla"
+    );
+}
+
+/// D3: la impacto nunca se cachea. `buscar` no le calcula clave (sin clave no
+/// hay candidatos) y el publicador no le escribe linea `arbol=`; el resumen
+/// lleva `cola-de-merge=`. La entrada es booleana y falsa por defecto en las
+/// dos plataformas, y la plantilla la aplica con las reglas de `merge_train`.
+#[test]
+fn la_impacto_nunca_se_cachea_y_la_cola_es_explicita() {
+    let texto = leer(WORKFLOW);
+    let candidatos = run_del_paso(&texto, "buscar", "Candidatos de la cache por arbol");
+    assert!(
+        candidatos.contains(r#"jq -r '.[] | select(. != "impacto")'"#),
+        "{candidatos}"
+    );
+    assert!(candidatos.contains("if [ \"$claves\" = '{}' ]; then"));
+    let publicar = run_del_paso(&texto, "verificacion", "Publicar el check verificacion");
+    assert!(
+        publicar.contains("if [ \"$p\" = impacto ]; then continue; fi"),
+        "{publicar}"
+    );
+    assert!(publicar.contains("\ncola-de-merge=$COLA\""), "{publicar}");
+    let raiz = yaml(&texto);
+    let entrada = raiz
+        .get("on")
+        .and_then(|o| o.get("workflow_call"))
+        .and_then(|w| w.get("inputs"))
+        .and_then(|i| i.get("cola-de-merge"));
+    assert_eq!(
+        entrada.and_then(|e| e.get("type")).and_then(Nodo::texto),
+        Some("boolean")
+    );
+    assert_eq!(
+        entrada.and_then(|e| e.get("default")).and_then(Nodo::texto),
+        Some("false")
+    );
+
+    let texto_gl = leer(PLANTILLA);
+    let (cabecera, cuerpo) = plantilla(&texto_gl);
+    let entrada_gl = cabecera
+        .get("spec")
+        .and_then(|s| s.get("inputs"))
+        .and_then(|i| i.get("cola-de-merge"));
+    assert_eq!(
+        entrada_gl.and_then(|e| e.get("type")).and_then(Nodo::texto),
+        Some("boolean")
+    );
+    assert_eq!(
+        entrada_gl
+            .and_then(|e| e.get("default"))
+            .and_then(Nodo::texto),
+        Some("false")
+    );
+    let reglas = |job: &str| {
+        let mut condiciones = Vec::new();
+        if let Some(r) = cuerpo.get(job).and_then(|n| n.get("rules")) {
+            r.escalares("rules", &mut condiciones);
+        }
+        condiciones
+            .into_iter()
+            .filter(|(k, _)| *k == "if")
+            .map(|(_, c)| c.to_owned())
+            .collect::<Vec<_>>()
+    };
+    let mr = "$CI_PIPELINE_SOURCE == \"merge_request_event\"";
+    assert_eq!(
+        reglas("verificacion-impacto"),
+        [
+            "$TC2_PERFILES =~ / impacto /".to_owned(),
+            format!(
+                "{mr} && $TC2_COLA_DE_MERGE =~ / true / && $CI_MERGE_REQUEST_EVENT_TYPE != \"merge_train\" && ($CI_MERGE_REQUEST_TARGET_BRANCH_NAME != \"main\" || $CI_MERGE_REQUEST_SOURCE_BRANCH_NAME != \"develop\")"
+            ),
+        ]
+    );
+    assert_eq!(
+        reglas("verificacion-completa"),
+        [
+            "$TC2_PERFILES =~ / completa /".to_owned(),
+            format!("{mr} && $TC2_COLA_DE_MERGE !~ / true /"),
+            format!("{mr} && $CI_MERGE_REQUEST_EVENT_TYPE == \"merge_train\""),
+            format!(
+                "{mr} && $CI_MERGE_REQUEST_TARGET_BRANCH_NAME == \"main\" && $CI_MERGE_REQUEST_SOURCE_BRANCH_NAME == \"develop\""
+            ),
+        ]
+    );
+    let mut despues = Vec::new();
+    if let Some(s) = cuerpo
+        .get(".tc2-verificacion")
+        .and_then(|b| b.get("after_script"))
+    {
+        s.escalares("after_script", &mut despues);
+    }
+    assert!(
+        despues
+            .iter()
+            .any(|(_, l)| l.contains("[ \"$TC2_PERFIL\" != impacto ]")),
+        "{despues:?}"
+    );
+}
+
 /// Un workflow minimo que cumple todo: el caso positivo de las mutaciones.
 const BUENO: &str = "\
 on:
````

## Excluido del diff (9 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/02-en-curso/TASK-043/medicion/stub-verificar.sh                                    | 66 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/02-en-curso/TASK-043/medicion/task-043-antes.txt                                   |  3 +++
 tareas/02-en-curso/TASK-043/medicion/task-043-despues.txt                                 |  3 +++
 tareas/02-en-curso/TASK-043/medicion/task-043-medir.sh                                    | 42 ++++++++++++++++++++++++++++++++++++++++++
 tareas/02-en-curso/TASK-043/medicion/task-043-release.txt                                 |  1 +
 tareas/02-en-curso/TASK-043/medicion/task-043-tiempos.sh                                  | 19 +++++++++++++++++++
 tareas/{01-en-diseno => 02-en-curso}/TASK-043/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-043/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-043/tarea.md                                    |  3 ++-
 9 files changed, 136 insertions(+), 1 deletion(-)
````
