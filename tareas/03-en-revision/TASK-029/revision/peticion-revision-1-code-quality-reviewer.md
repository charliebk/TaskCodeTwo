# Peticion de revision — TASK-029 (ronda 1)

- Tarea: TASK-029 — F3-2 tc2 new e import en bloque con dependencias e hito
- Rama revisada: feature/task-029-f3-2-new-e-import-en-bloque-con-dependen
- Rama base: develop
- Commit revisado (HEAD): 80f72ac09fe9e660606b55f88dc82c61307d5b56
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-029 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-029/tarea.md
- tareas/03-en-revision/TASK-029/planificacion/
- docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md

## Commits a revisar (git log develop..HEAD)

````
80f72ac docs(TASK-029): autorrevision con las revisoras de rust y calidad
a2232d9 refactor(TASK-029): correcciones de la autorrevision
895658a feat(TASK-029): new acepta los campos de RF-01 y rechaza un enunciado no listo
a5d2fb1 feat(TASK-029): import --simular como plan sobre el estado conocido
17dde3e feat(TASK-029): tc2 import en una sola transaccion
049e8c1 feat(TASK-029): parser puro del fichero de import
76ac492 refactor(TASK-029): nucleo altas comun a new e import
a0f9452 chore(TASK-029): tarea en curso
````

## Diff de tu dominio (1 fichero(s) de 14; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md b/docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md
index ff75790..41e805c 100644
--- a/docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md
+++ b/docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md
@@ -6,6 +6,7 @@
 - Depende de: [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) (carpeta fija `TASK-NNN/`, estado como campo), [ADR F1-2](F1-2-escrituras-atomicas-ids-y-toma.md) (ID y toma con el «quién»), [ADR F1-3](F1-3-flujo-de-ramas-worktrees-release.md) (nombre de rama), [ADR F1-5](F1-5-puertas-en-github.md) (decisión 4: la aprobación es el commit firmado con `Plan-Blob` sobre el blob de `plan-final.md`) y [ADR F1-8](F1-8-seguridad-modelo-de-amenazas.md) (zona Z2 y activo `A10`: el texto de las tareas es dato no confiable)
 - Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a y D5a del plan de TASK-010)
 - Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
+- Adenda (2026-11-17, TASK-029): clave opcional `lote` para el import en bloque; ver [la adenda al final](#adenda-f3-2-task-029--clave-opcional-lote)
 
 ## Contexto
 
@@ -506,3 +507,35 @@ Este ADR se planificó con un solo rol (arquitectura). Lo que no cubrió:
   ToolsaWeb tiene 404). Lo mide F3-4.
 - **Spec Kit**: la `constitution` y los comandos de Spec Kit no tienen dueño
   en el plan.
+
+## Adenda (F3-2, TASK-029) — clave opcional `lote`
+
+Se reabre por escrito el **punto 3** (campos del frontmatter) para una clave
+nueva, decisión D1 de TASK-029 (aceptada por Carlos con la consigna del
+2026-11-16).
+
+- **Qué es**: `lote: "<lote>/<clave>"` en una tarea creada por
+  `tc2 import`. `<lote>` es el `lote:` de la cabecera del fichero (o, sin
+  cabecera, el slug del nombre del fichero) y `<clave>` la de la entrada
+  (el primer token del `###`, como `F3-2`, o el slug del título). Por ella
+  un import repetido reconoce lo que ya creó y solo crea lo que falta: no
+  depende del título, que la persona puede editar.
+- **Opcional**, a diferencia de las 21 claves del punto 3: si falta vale
+  `null` y solo se escribe con valor, **después de `dependencias`** (la
+  última clave conocida, antes de las desconocidas). Una tarea de antes de
+  esta adenda se lee y se reescribe byte a byte igual (la dorada de F2-5
+  sigue pasando) y `new` no la escribe.
+- **Esquema**: texto sin caracteres de control con una `/` que separa dos
+  partes no vacías; cualquier otra cosa es error de lectura, como el resto
+  del esquema estricto (punto 5). Ya no cuenta como clave desconocida.
+- **Lo que no hace**: un import no sincroniza. La tarea ya importada no se
+  actualiza aunque su entrada cambie en el fichero; sale en `existentes=`.
+- **Hito**: es `sprint` (D2 de TASK-029); el «hito de decisiones» de F1-18
+  sigue fuera y se conserva como clave desconocida.
+
+Evidencia: `crates/tc2/src/alta.rs`
+(`plantilla_escapa_el_titulo_y_sigue_f1_9`, que fija el orden de claves con
+y sin `lote`), `crates/tc2/tests/tarea.rs` (`ida_y_vuelta_byte_a_byte`, sin
+cambios) y `crates/tc2/tests/importar.rs`
+(`c1_c3_importa_37_en_un_commit_y_repetir_no_crea_nada`,
+`c4_dos_entradas_mas_crean_solo_esas_dos`).
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-029/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-029/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-029/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````
