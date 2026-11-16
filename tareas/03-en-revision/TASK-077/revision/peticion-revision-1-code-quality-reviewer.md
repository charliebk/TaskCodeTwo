# Peticion de revision — TASK-077 (ronda 1)

- Tarea: TASK-077 — F2-13 Identidad de la persona desde la plataforma
- Rama revisada: feature/task-077-f2-13-identidad-de-la-persona-desde-la-p
- Rama base: develop
- Commit revisado (HEAD): eebf5f753da191740858ecfa9eea2af9c918b956
- Fecha: 2026-11-16
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-077 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-077/tarea.md
- tareas/03-en-revision/TASK-077/planificacion/
- docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md

## Commits a revisar (git log develop..HEAD)

````
eebf5f7 chore(TASK-077): criterios C1 y C7 con el formato del contrato F1-4
3b4e626 docs(TASK-077): autorrevision con las dos revisoras
ce6cc61 fix(TASK-077): autorrevision, login de la cuenta sin rodeos y doc del modulo identidad
b934342 feat(TASK-077): conversion de asignado_a heredado por correos y nota en el ADR F1-9
2474c3a feat(TASK-077): orden tc2 identidad con su catalogo y tests contra GitHub y GitLab simulados
15ee5b7 feat(TASK-077): quien ejecuta, cache local de identidad con TTL y rechazo de bots
6bdcf37 feat(TASK-077): asignado_a como persona {id, login} o texto heredado de la 0.16
b3ed37d feat(TASK-077): lista de personas con login, nombre y correos y unicidad
f175cc6 chore(TASK-077): tarea en curso
````

## Diff de tu dominio (1 fichero(s) de 13; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md b/docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md
index 58ec50f..c6a119c 100644
--- a/docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md
+++ b/docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md
@@ -463,6 +463,15 @@ Pasa como criterio a F2-5 (verlo en el repo de pruebas de F2-6).
 - **F1-10** fija los valores de `carril`; **F1-15** el contenido de `roles`;
   **F1-17** la forma de `estimacion`; **F2-13** el valor de `asignado_a`;
   **F3-4** lee `estado` (y no carpetas) para el roadmap.
+
+  > **Nota (TASK-077, F2-13, 2026-11-16)**: `asignado_a` es `null` o el
+  > objeto en flujo `{id: <entero sin comillas>, login: "<login>"}` de la
+  > cuenta de GitHub o GitLab. Se compara solo por `id` (F1-6, decisión 3);
+  > `login` es una copia para mostrar que refresca `.taskcode/personas.yml`.
+  > Un texto libre de la 0.16 se lee como `Heredado`: la 2.0 nunca crea
+  > uno; si lo lee en formato 2 lo conserva tal cual (cadena entre
+  > comillas) hasta que se convierta por los `correos:` de
+  > `personas.yml` o lo resuelva `reasignar` (F3-10). No reabre este ADR.
 - **F1-11**: las claves desconocidas son la costura de las extensiones.
 - **Este repo no cambia de formato** (RR-14): sus tareas siguen en la 0.16
   hasta el relevo (F4-5) y la migración de F1-12.
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-077/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-077/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-077/tarea.md                                    | 12 +++++++++---
 3 files changed, 9 insertions(+), 3 deletions(-)
````
