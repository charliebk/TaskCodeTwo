# Peticion de revision — TASK-049 (ronda 3)

- Tarea: TASK-049 — F6-3 Revisor en CI con las skills de dominio de la 0.16
- Rama revisada: feature/task-049-f6-3-revisor-en-ci-con-las-skills-de-dom
- Rama base: develop
- Commit revisado (HEAD): ca4de580f20504840ec08a8d48b2aae9340b3b30
- Fecha: 2026-11-20
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-049 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde fc65d9e024c04921628c2087ca6622c7efaaade0 (el commit revisado en la ronda anterior)
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
informe de esta ronda (informe-revision-3-rust-wasm-reviewer.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-049/tarea.md
- tareas/03-en-revision/TASK-049/planificacion/
- .github/workflows/revision-ia.yml
- docs/adr/F1-8-seguridad-modelo-de-amenazas.md
- docs/ci/revision-ia.md
- xtask/src/workflows.rs

## Hallazgos de la ronda 2 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 1 hallazgo(s) MENOR de la ronda 2 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | xtask/src/workflows.rs:1502 y :1506 | informe-revision-2-rust-wasm-reviewer.md |

## Commits a revisar (git log fc65d9e024c04921628c2087ca6622c7efaaade0..HEAD)

````
ca4de58 chore(TASK-049): informes de la ronda 2 y resultado
9c14787 fix(TASK-049): hallazgos de la ronda 2 de revision
1353bc3 chore(TASK-049): peticion de revision ronda 2
````

## Diff desde la ronda anterior (git diff fc65d9e024c04921628c2087ca6622c7efaaade0..HEAD)

````diff
diff --git a/.github/workflows/revision-ia.yml b/.github/workflows/revision-ia.yml
index 7b95f22..0fad971 100644
--- a/.github/workflows/revision-ia.yml
+++ b/.github/workflows/revision-ia.yml
@@ -359,8 +359,10 @@ jobs:
               codigo=$(curl -sS --max-time 900 -o "$respuesta" -w '%{http_code}' "${cabeceras[@]}" --data-binary "@$cuerpo" https://api.anthropic.com/v1/messages) || codigo=000
               parada=$(jq -r '.stop_reason // ""' "$respuesta" 2>/dev/null || echo "")
               if [ "$codigo" != 200 ]; then
-                # El mensaje de error de la API (sin la clave), recortado y en una linea.
-                motivo=$(jq -r '"\(.error.type // ""): \(.error.message // "")"' "$respuesta" 2>/dev/null | tr -d '\r\n' | cut -c1-200 || true)
+                # Tipo y mensaje de error de la API, en una linea, con la clave
+                # sustituida si apareciera (jq la lee del entorno, no de argv) y
+                # recortado a 200 caracteres, no bytes.
+                motivo=$(jq -r '[.error.type, .error.message] | map(select(type == "string" and . != "")) | join(": ") | gsub("[\r\n]"; " ") | split(env.ANTHROPIC_API_KEY) | join("***") | .[0:200]' "$respuesta" 2>/dev/null || true)
                 estado="error-http-$codigo ${motivo:-sin-cuerpo}"
               elif [ "$parada" = refusal ]; then
                 estado="negativa"
diff --git a/docs/adr/F1-8-seguridad-modelo-de-amenazas.md b/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
index ff9f7da..b5ff11c 100644
--- a/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
+++ b/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
@@ -724,6 +724,12 @@ Los ADR aceptados no se modifican; lo que este ADR cambia queda aquí y en
   hereda llega a un workflow que fija el SHA y cuyo lint prohíbe secretos
   fuera del job del entorno `revisor`; la alternativa (secretos de
   repositorio pasados uno a uno) perdía la regla de rama del entorno.
+  Dos límites, dichos: `inherit` entrega al reutilizable **todos** los
+  secretos de repositorio y de organización del llamador, no solo los del
+  entorno; y el lint es de tests (`cargo test` de este repo, sobre la guía y
+  los workflows de TaskCodeTwo), así que en un proyecto consumidor lo que
+  restringe el llamador es su revisión humana y el SHA fijado, hasta que
+  `doctor` lo compruebe (F3-8).
 - **Verificado en el E2E** (`charliebk/taskcodetwo-e2e-github`, PR 14,
   Rust + Vue): `preparar` sin secretos y sin ejecutar nada del PR elige
   `angular-vue-reviewer` y `rust-wasm-reviewer` (reglas S08, S12); el
diff --git a/docs/ci/revision-ia.md b/docs/ci/revision-ia.md
index cad0a58..096135b 100644
--- a/docs/ci/revision-ia.md
+++ b/docs/ci/revision-ia.md
@@ -86,7 +86,10 @@ leída en `taskcodetwo/tareas`; sin tarea, complejidad `media` con aviso.
    llegan vacíos (las dos cosas comprobadas en el E2E de TASK-049). El lint
    (perfil `llamador-revision`) solo admite `inherit` hacia
    `charliebk/TaskCodeTwo/.github/workflows/revision-ia.yml` fijado por SHA,
-   y el del reutilizable impide usar secretos fuera de ese job:
+   y el del reutilizable impide usar secretos fuera de ese job. Ojo:
+   `inherit` entrega al reutilizable todos los secretos de repositorio y de
+   organización, no solo los del entorno, y ese lint corre en los tests de
+   TaskCodeTwo, no en tu repo: revisa el llamador a mano y fija el SHA:
 
 ```yaml
 name: revision-ia
diff --git a/xtask/src/workflows.rs b/xtask/src/workflows.rs
index 4cd14c8..886c7b2 100644
--- a/xtask/src/workflows.rs
+++ b/xtask/src/workflows.rs
@@ -1499,11 +1499,13 @@ fn cada_mutacion_del_revisor_da_su_error() {
 #[test]
 fn llamador_de_revision_ia_hereda_solo_hacia_revision_ia() {
     let guia = include_str!("../../docs/ci/revision-ia.md").replace("\r\n", "\n");
-    let inicio = guia
-        .find("```yaml\nname: revision-ia\n")
-        .expect("llamador en la guia")
-        + 8;
-    let fin = inicio + guia[inicio..].find("```").expect("fin del bloque");
+    let Some(inicio) = guia.find("```yaml\nname: revision-ia\n").map(|i| i + 8) else {
+        panic!("la guia no tiene el bloque del llamador");
+    };
+    let Some(largo) = guia[inicio..].find("```") else {
+        panic!("el bloque del llamador no se cierra");
+    };
+    let fin = inicio + largo;
     let llamador = guia[inicio..fin].replace("@<sha>", "@0000000000000000000000000000000000000000");
     assert_eq!(lint_llamador_revision(&llamador), Vec::<String>::new());
     let malos = [
@@ -1529,4 +1531,21 @@ fn llamador_de_revision_ia_hereda_solo_hacia_revision_ia() {
             .iter()
             .any(|e| e.contains("secrets: inherit"))
     );
+    // El reutilizable (perfil revision-ia) sigue sin admitir la herencia,
+    // tampoco hacia el destino admitido en el llamador.
+    assert!(
+        lint_revision(&llamador)
+            .iter()
+            .any(|e| e.contains("secrets: inherit"))
+    );
+    // El llamador no escribe con el GITHUB_TOKEN.
+    let con_escritura = llamador.replace("      contents: read\n", "      contents: write\n");
+    assert_ne!(con_escritura, llamador);
+    assert!(
+        lint_llamador_revision(&con_escritura)
+            .iter()
+            .any(|e| e.contains("escritura")),
+        "{:?}",
+        lint_llamador_revision(&con_escritura)
+    );
 }
````

## Excluido del diff (5 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff fc65d9e024c04921628c2087ca6622c7efaaade0..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/03-en-revision/TASK-049/revision/informe-revision-2-code-quality-reviewer.md  |  59 +++++++++++++++++
 tareas/03-en-revision/TASK-049/revision/informe-revision-2-rust-wasm-reviewer.md     |  45 +++++++++++++
 tareas/03-en-revision/TASK-049/revision/peticion-revision-2-code-quality-reviewer.md | 359 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-049/revision/peticion-revision-2-rust-wasm-reviewer.md    | 369 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-049/tarea.md                                              |  38 ++++++++++-
 5 files changed, 867 insertions(+), 3 deletions(-)
````
