# Peticion de revision — TASK-042 (ronda 2)

- Tarea: TASK-042 — F5-2 Cache por SHA del arbol compartida
- Rama revisada: feature/task-042-f5-2-cache-por-sha-del-arbol-compartida
- Rama base: develop
- Commit revisado (HEAD): 4f4a62d3640e923a9eb5296a8889503e5967fd17
- Fecha: 2026-11-19
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-042 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde aee1621fc055573ce506eeeb31dda3082f1027a2 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (4 hallazgo(s)))

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
informe de esta ronda (informe-revision-2.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-042/tarea.md
- tareas/03-en-revision/TASK-042/planificacion/
- .github/workflows/verificacion.yml
- docs/ci/verificacion-reutilizable.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 2 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | .github/workflows/verificacion.yml:197 | informe-revision-1-code-quality-reviewer.md |

## Commits a revisar (git log aee1621fc055573ce506eeeb31dda3082f1027a2..HEAD)

````
4f4a62d fix(TASK-042): regla de rutas con --no-renames y ordenes en la clave
fc8caf0 chore(TASK-042): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff aee1621fc055573ce506eeeb31dda3082f1027a2..HEAD)

````diff
diff --git a/.github/workflows/verificacion.yml b/.github/workflows/verificacion.yml
index 04055b4..e01cce1 100644
--- a/.github/workflows/verificacion.yml
+++ b/.github/workflows/verificacion.yml
@@ -176,7 +176,10 @@ jobs:
           runners='{}'
           for p in $(printf '%s' "$PERFILES" | jq -r '.[]'); do
             r=$(printf '%s' "$ENTRADAS" | jq -r --arg p "$p" '.["runner-" + $p] | fromjson | if type == "array" then join(",") else tostring end' | tr -d ' ')
-            k=$(printf '%s|%s|%s|%s' "$p" "$r" "$huella" "$VERSION" | sha256sum | cut -d' ' -f1)
+            # Las ordenes entran en la clave: un verde de otro comando,
+            # argumentos, shell o huella local no se reutiliza.
+            ordenes=$(printf '%s' "$ENTRADAS" | jq -c --arg p "$p" '[.comando, .["argumentos-" + $p], .shell, .["huella-local"]]' | sha256sum | cut -d' ' -f1)
+            k=$(printf '%s|%s|%s|%s|%s' "$p" "$r" "$huella" "$VERSION" "$ordenes" | sha256sum | cut -d' ' -f1)
             claves=$(printf '%s' "$claves" | jq -c --arg p "$p" --arg k "$k" '. + {($p): $k}')
             runners=$(printf '%s' "$runners" | jq -c --arg p "$p" --arg r "$r" '. + {($p): $r}')
           done
@@ -194,7 +197,7 @@ jobs:
           # F1-8, decision 7: un PR que toca los workflows o .taskcode no
           # reutiliza nada, tampoco la linea de su head anterior.
           if git rev-parse -q --verify 'HEAD^1' > /dev/null; then
-            tocadas=$(git diff --name-only 'HEAD^1' HEAD | grep -E '^(\.github/workflows/|\.taskcode/)' || true)
+            tocadas=$(git diff --no-renames --name-only 'HEAD^1' HEAD | grep -E '^(\.github/workflows/|\.taskcode/)' || true)
             if [ -n "$tocadas" ]; then
               echo "Regla de rutas: el diff frente a la base toca $(printf '%s' "$tocadas" | tr '\n' ' '); sin acierto"
               echo "candidatos={}" >> "$GITHUB_OUTPUT"
diff --git a/docs/ci/verificacion-reutilizable.md b/docs/ci/verificacion-reutilizable.md
index bf0bd41..00352a6 100644
--- a/docs/ci/verificacion-reutilizable.md
+++ b/docs/ci/verificacion-reutilizable.md
@@ -234,9 +234,15 @@ arbol=<árbol de V> perfil=<p> runner=<r> clave=<k> local=<h> origen=<sha> reuti
 
 - `runner`: el `runner-<perfil>` declarado, sin comillas ni espacios (una
   lista de etiquetas, unida por comas).
-- `clave = sha256(perfil|runner|huella-arbol|versión del workflow)` (el `sha`
-  de `referenced_workflows` de la ejecución), con
-  `huella-arbol = sha256(git ls-tree -r HEAD -- <rutas>)` (o de nada sin rutas).
+- `clave = sha256(perfil|runner|huella-arbol|versión del workflow|órdenes)` (la
+  versión es el `sha` de `referenced_workflows` de la ejecución), con
+  `huella-arbol = sha256(git ls-tree -r HEAD -- <rutas>)` (o de nada sin rutas)
+  y `órdenes = sha256` del JSON `[comando, argumentos-<perfil>, shell,
+  huella-local]`: un verde de otras órdenes (p. ej. pasadas por `vars` o por
+  un `workflow_dispatch`) no se reutiliza. La regla de rutas usa
+  `git diff --no-renames`, para que mover un fichero fuera de
+  `.github/workflows/` o `.taskcode/` cuente como tocarlo (revisión de la
+  ronda 1).
 - `local = sha256` de lo que imprime `huella-local` en el runner del perfil.
 - `origen`: el commit (head del PR) donde se ejecutó de verdad; una línea
   reutilizada conserva el del candidato. `reutilizada de <commit>`: el commit
````

## Excluido del diff (9 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff aee1621fc055573ce506eeeb31dda3082f1027a2..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-042/medicion/medir.sh                           |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-042/medicion/salida.txt                         |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-042/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-042/planificacion/plan-final.md                 |   0
 tareas/03-en-revision/TASK-042/revision/informe-revision-1-code-quality-reviewer.md         |  21 ++++
 tareas/03-en-revision/TASK-042/revision/informe-revision-1-rust-wasm-reviewer.md            |  21 ++++
 tareas/03-en-revision/TASK-042/revision/peticion-revision-1-code-quality-reviewer.md        | 674 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-042/revision/peticion-revision-1-rust-wasm-reviewer.md           | 279 +++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-042/tarea.md                                    |   7 +-
 9 files changed, 1000 insertions(+), 2 deletions(-)
````
