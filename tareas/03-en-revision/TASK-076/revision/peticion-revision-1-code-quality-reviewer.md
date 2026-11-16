# Peticion de revision — TASK-076 (ronda 1)

- Tarea: TASK-076 — F2-4b Nombres unicos de tarea y rama
- Rama revisada: feature/task-076-f2-4b-nombres-unicos-de-tarea-y-rama
- Rama base: develop
- Commit revisado (HEAD): ebc7fa5647f8386bf0266e16eab63a8a34d7e8a8
- Fecha: 2026-11-16
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-076 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-076/tarea.md
- tareas/03-en-revision/TASK-076/planificacion/
- docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md

## Commits a revisar (git log develop..HEAD)

````
ebc7fa5 docs(TASK-076): autorrevision con rust-wasm-reviewer y code-quality-reviewer
0f11a56 fix(TASK-076): el alta fija tipo y rama desde la misma constante
40eeeb6 test(TASK-076): aviso de parecidas en espanol e ingles
da479a2 feat(TASK-076): aviso de titulo duplicado o parecido a una tarea abierta
bada16c feat(TASK-076): leer_varios con un solo git cat-file --batch
7bd6455 feat(TASK-076): el ID nuevo salta las ramas de tarea locales y remotas
24d4c81 feat(TASK-076): plegado, slug y rama derivada del ID en tarea.md
d66e30f chore(TASK-076): tarea en curso
````

## Diff de tu dominio (1 fichero(s) de 17; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md b/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
index 5f98f67..f06e374 100644
--- a/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
+++ b/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
@@ -468,3 +468,41 @@ non-fast-forward; se trata como contención (ver la adenda de F1-4).
 - **Choque por mayúsculas dentro del lote**: dos rutas de la misma operación
   que solo difieren en mayúsculas también dan `ruta-duplicada`, y el
   `ROADMAP.md` generado se comprueba igual.
+
+## Adenda (F2-4b, TASK-076) — el ID cuenta también las ramas
+
+Se reabre por escrito el **punto 3** (decisión D3 de TASK-076, aceptada por
+Carlos el 2026-11-16).
+
+**Nuevo texto del punto 3**: ID = **máximo de las carpetas `TASK-*` y de los
+números de las ramas de tarea `*/task-nnn-*`, más 1**. Las ramas se leen una
+sola vez por `new`, antes de la transacción: las del remoto con
+`ls-remote --heads origin '*/task-*'` y las locales con
+`branch --list '*/task-*'`. Así ninguna alta nombra una rama que ya existe
+(una rama creada a mano sin carpeta, o una tarea de la 0.16 cuya carpeta no
+se migró): la tarea no nace bloqueada en `start`.
+
+- **Qué cuenta como rama de tarea**: un componente que no sea el primero (el
+  tipo) de la forma `task-<cifras>` o `task-<cifras>-<resto>`, en
+  minúsculas. Comprobado con Git 2.56: `ls-remote` casa el patrón por
+  sufijo (también da `refs/heads/task-7-x`, sin tipo, y `a/b/task-1-x`),
+  mientras que `branch --list` lo casa contra el nombre entero; el parser
+  común (`id_de_rama`) decide igual para las dos, y `task-7-x` sin tipo no
+  cuenta.
+- **Sin remoto** (RN-06): solo se miran las ramas locales.
+- **Fallo del remoto**: como en `fetch`. Red caída → `sinred
+  pendiente=no` (4), sin `fetch`, sin commit y sin refs nuevas; credenciales
+  o remoto inexistente → `denegada` (8); fallo del propio clon → `error`.
+- **La rama se escribe en `tarea.md`**: `rama: feature/task-nnn-<slug>`
+  (F1-9 punto 6), con el slug del título plegado (ASCII `[a-z0-9-]`, como
+  mucho 40 bytes cortando en guion; sin nada latino → `tarea`). Pasa la
+  misma validación que usan `crear-rama-y-worktree` y `reanudar`.
+- **Riesgo aceptado**: una rama creada a mano entre el `ls-remote` y el push
+  de la rama de estado no se ve; la para `comprobar-id` en `start` (F3-6).
+- **Coste**: un viaje de red más por `new`, uno solo y no por intento.
+
+Evidencia: `crates/tc2/tests/alta.rs` (`rama_solo_en_el_remoto_ocupa_su_id`,
+`rama_solo_en_local_ocupa_su_id`,
+`ls_remote_previo_sin_red_o_denegado_no_deja_nada`) y los unitarios de
+`alta.rs` (`siguiente_id_es_el_maximo_mas_uno`) y `git/rama.rs`
+(`id_de_las_ramas_de_tarea`).
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-076/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-076/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-076/tarea.md                                    | 14 +++++++++++++-
 3 files changed, 13 insertions(+), 1 deletion(-)
````
