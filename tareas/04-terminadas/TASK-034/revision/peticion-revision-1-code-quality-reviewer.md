# Peticion de revision — TASK-034 (ronda 1)

- Tarea: TASK-034 — F3-7 descartar y pausa
- Rama revisada: feature/task-034-f3-7-descartar-y-pausa
- Rama base: develop
- Commit revisado (HEAD): 5dc73db88b26d5303bba1983e29f5d2de4803f22
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-034 (criterios de aceptacion y plan)
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
informe de esta ronda (informe-revision-1-code-quality-reviewer.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-034/tarea.md
- tareas/03-en-revision/TASK-034/planificacion/
- crates/tc2/tests/fixtures/roadmap/referencia.md
- docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
- docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md

## Commits a revisar (git log develop..HEAD)

````
5dc73db docs(TASK-034): autorrevision con rust-wasm-reviewer y code-quality-reviewer
e4c2c67 test(TASK-034): E2E de descartar, pausa y reanudar y adendas de F1-2 y F1-4
90dc2e1 feat(TASK-034): ordenes descartar, pausa y reanudar en el CLI
5ebae52 feat(TASK-034): operaciones Descartar y Pausar sobre la rama de estado
aee7594 feat(TASK-034): pausa calculada de las transiciones y motivo de las descartadas
a630e8e refactor(TASK-034): lectura comun tarea_de para las operaciones de una tarea
c451c7a chore(TASK-034): tarea en curso
````

## Diff de tu dominio (3 fichero(s) de 21; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md b/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
index 6c3271e..bd27bbc 100644
--- a/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
+++ b/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
@@ -507,3 +507,30 @@ Evidencia: `crates/tc2/tests/alta.rs` (`rama_solo_en_el_remoto_ocupa_su_id`,
 `ls_remote_previo_sin_red_o_denegado_no_deja_nada`) y los unitarios de
 `alta.rs` (`siguiente_id_es_el_maximo_mas_uno`) y `git/rama.rs`
 (`id_de_las_ramas_de_tarea`).
+
+## Adenda (F3-7, TASK-034) — descartar exige red; pausa y reanudar no
+
+Se precisa el **punto 5** (decisión D4 de TASK-034, opción recomendada
+aceptada por Carlos para todo el H3).
+
+- **`descartar` exige red**, como `new`, `import`, `start`, `liberar` y
+  `reasignar`: aunque toca solo la tarea propia (o una sin asignar), cambia
+  lo que `next` ofrece y las dependencias de los demás, y un descarte
+  pendiente que otro adelantara empezando la tarea se perdería. Sin red,
+  `descartar sinred pendiente=no` (4) sin commit ni ref pendiente. Antes de
+  la transacción hace `fetch` (también las ramas): con la rama de la tarea
+  al día se decide si su worktree está publicado y se puede quitar.
+- **`pausa` y `reanudar` no exigen red**: son transiciones de una tarea que
+  lleva quien las lanza (o que no lleva nadie), filas `pausa`/`reanudar` de
+  `## Transiciones` sin estado nuevo (D1; F1-9 sigue cerrada). Sin red quedan
+  pendientes con su anotación (`pausa|TASK-NNN|<id>|<login>|<instante>`, con
+  el instante de la pausa para que la fila lo conserve al publicarse: F1-17
+  descuenta las pausas por sus filas) y la siguiente transacción con red las
+  re-ejecuta como cualquier pendiente, con `decidido_por: agente`.
+- **El ID de una descartada no se reutiliza** (punto 3, ya escrito): la
+  carpeta no se borra nunca y `descartar` no toca la rama de código.
+
+Evidencia: `crates/tc2/tests/ciclo.rs`
+(`c8_sin_red_descartar_falla_y_la_pausa_queda_pendiente`,
+`c4_el_id_de_la_descartada_no_se_reutiliza`) y los unitarios de
+`ciclo/operacion.rs` (`anotacion_ida_y_vuelta`).
diff --git a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
index a708420..1cfd75e 100644
--- a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
+++ b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
@@ -7,6 +7,7 @@
 - Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a, D5a del plan de TASK-005)
 - Adenda (2026-11-17, TASK-032, F3-5): entra `prs-abiertos` en la lista cerrada (decisión 2) para `tc2 board` (D2 de TASK-032, aceptada por Carlos el 2026-11-16); detalle de llamadas en la [adenda de F1-13](F1-13-adaptador-de-plataforma.md). `board` y `status`, como `roadmap`, escriben avisos de una línea en stderr con código 0; el canal fiable son las claves de la línea y del JSON
 - Adenda (2026-11-17, TASK-033, F3-6): `start [--json] TASK-NNN` compone `fetch`, `validar-base`, `comprobar-id`, `commit-estado`, `crear-rama-y-worktree` y `reanudar` sin operaciones nuevas (la base de un `hotfix` es el último tag por versión de `refs/tags/` contenido en `origin/main`, leído con `for-each-ref --merged`, o su punta). Línea de éxito: `start ok id=… rama=… worktree=… base=… estado=<commit de la toma> asignado=<login>`; dueño con la tarea empezada: `sin-cambios` o `ok creado=rama,worktree` sin escribir estado. Motivos de `rechazada` (3): `no-existe`, `formato-1` (con `arreglo=tc2 actualizar`), `ilegible`, `tomada` (con `asignado=<login>`), `asignado-heredado`, `terminada`, `descartada`, `estado-no-valido`, `dependencias-pendientes` (con `dependencias=`), `rama-invalida`, `tipo-release`, `ocupada` (con `ramas=`), `carpeta-ocupada` (la carpeta del worktree existe y no es el worktree registrado de la rama; se comprueba antes de publicar la toma, sin `estado=`) e `identidad`; `uso` (2): `sin-id`, `id-invalido`, `argumentos-de-mas`; sin red, `sinred pendiente=no` (4)
+- Adenda (2026-11-17, TASK-034, F3-7): `descartar [--json] TASK-NNN "<motivo>"`, `pausa [--json] TASK-NNN` y `reanudar [--json] TASK-NNN` componen `fetch` (solo `descartar`), `commit-estado` y `quitar-worktree` (solo `descartar`, tras publicar) sin operaciones nuevas. Líneas: `descartar ok id=… commit=… intentos=… tareas=… worktree=quitado|ninguno|conservado [worktree-motivo=cambios-sin-guardar|sin-publicar|sin-rama|en-uso|error]` (el worktree conservado no hace fallar la orden: el descarte ya está publicado y la rama no se toca nunca); `pausa ok id=… commit=…` y `reanudar ok id=… commit=…`. `sin-cambios` (0): descartar una descartada (vale el motivo de entonces y se intenta quitar el worktree), pausar una en pausa o reanudar una que no lo está. Motivos de `rechazada` (3): `no-existe`, `formato-1` (con `arreglo=tc2 actualizar`), `ilegible`, `de-otra-persona` (con `asignado=<login>`; D2: puede quien la lleva o cualquiera si no la lleva nadie), `asignado-heredado`, `terminada`, `descartada` (pausa y reanudar), `estado-no-valido` e `identidad`; `start` gana `pausada` (con `arreglo=tc2 reanudar`). `uso` (2): `sin-id`, `id-invalido`, `argumentos-de-mas` y, en `descartar`, `sin-motivo` y `motivo-invalido` (vacío, solo espacios, saltos de línea o caracteres de control, o más de 500 caracteres). Sin red: `descartar sinred pendiente=no` (4); `pausa`/`reanudar sinred pendiente=si` (4), que la siguiente transacción con red publica (adenda de TASK-034 en F1-2)
 - Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
 
 ## Contexto
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-034/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-034/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-034/tarea.md                                    | 24 ++++++++++++++++++++++--
 3 files changed, 22 insertions(+), 2 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/fixtures/roadmap/referencia.md"
````

````
crates/tc2/tests/fixtures/roadmap/referencia.md | 2 +-
 1 file changed, 1 insertion(+), 1 deletion(-)
````
