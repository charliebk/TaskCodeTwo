# Peticion de revision — TASK-038 (ronda 1)

- Tarea: TASK-038 — F4-3 Aprobacion de un hito en lote
- Rama revisada: feature/task-038-f4-3-aprobacion-de-un-hito-en-lote
- Rama base: develop
- Commit revisado (HEAD): 5ae20d8b11347556bddbfe0fe074e9827f4825f5
- Fecha: 2026-11-19
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-038 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-038/tarea.md
- tareas/03-en-revision/TASK-038/planificacion/
- docs/adr/F1-16-lotes-desatendidos.md
- docs/adr/F1-6-identidades-agente-revisor-persona.md

## Commits a revisar (git log develop..HEAD)

````
5ae20d8 chore(TASK-038): autorrevision
5112652 feat(TASK-038): adendas F4-3 en F1-6 y F1-16 y test del tope
5a39354 feat(TASK-038): tc2 approve --hito <n> aprueba un hito en lote
1f2243d chore(TASK-038): tarea en curso
````

## Diff de tu dominio (2 fichero(s) de 8; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/docs/adr/F1-16-lotes-desatendidos.md b/docs/adr/F1-16-lotes-desatendidos.md
index 1b3fd08..cd09ddb 100644
--- a/docs/adr/F1-16-lotes-desatendidos.md
+++ b/docs/adr/F1-16-lotes-desatendidos.md
@@ -916,3 +916,19 @@ taskcodetwo/tareas: lotes/noche-173-210.yml (diario), tareas/TASK-NNN/tarea.md (
   midió; D4a no lo necesita.
 - Que el plan de ToolsaWeb sea privado y gratuito se toma de F1-5 D2a (403);
   no se repitió el GET.
+
+## Adenda (F4-3, TASK-038) — la firma previa al lote puede ser `approve --hito`
+
+- **D7a** exige que cada funcionalidad del lote tenga su `approve` firmado.
+  Esa firma puede ser la de `tc2 approve --hito <n>` (F4-3): la persona
+  aprueba antes de la noche todos los planes del hito en **un único commit
+  firmado** con un trailer `Plan-Blob` por tarea, todo o nada (si una tarea
+  abierta del hito no está lista, no se aprueba ninguna y la salida dice
+  cuáles y por qué) y con un tope de 20 planes por orden.
+- `planificar` no distingue de dónde viene la aprobación: para una tarea
+  cuenta `plan_aprobado: true` con el blob actual de su `plan-final.md`
+  entre los `Plan-Blob` del commit que lo puso (la regla 5 del contrato del
+  firmante, tal como queda en la adenda F4-3 de F1-6). Esa comprobación es
+  del check `carril` (F6) y no se implementa en F4-3.
+- No cambia F1-10: que una tarea de riesgo alto no entre en el lote afecta a
+  lanzarlo, no a aprobar su plan.
diff --git a/docs/adr/F1-6-identidades-agente-revisor-persona.md b/docs/adr/F1-6-identidades-agente-revisor-persona.md
index 62b8995..d3c2c66 100644
--- a/docs/adr/F1-6-identidades-agente-revisor-persona.md
+++ b/docs/adr/F1-6-identidades-agente-revisor-persona.md
@@ -685,3 +685,47 @@ ADR. Lo que fija la implementación, con su evidencia en
   (`estado-no-valido`) o de otra persona (`de-otra-persona`).
 - La comprobación de la firma frente a la plataforma (reglas 1 a 5 de la
   decisión 3) sigue siendo del check `carril` (F6), no de `approve` (D3a).
+
+## Adenda (F4-3, TASK-038) — `tc2 approve --hito <n>`: N trailers en un commit
+
+- **Forma** (D1a): `tc2 approve [--json] --hito <n>` (`n` es el campo
+  `sprint`; vale `4` o `H4`). Aprueba de una vez los planes de las tareas
+  abiertas del sprint (ni `terminada` ni `descartada`) que aún no lo están,
+  con las mismas comprobaciones que `approve TASK-NNN` y la misma regla de
+  identidad: con el agente o sin identidad, `approve rechazada hito=<n>
+  motivo=identidad rol=persona` antes de lanzar git. Con un ID, con
+  `--pedir-cambios` o con un hito que no es un número, `uso`.
+- **Un commit firmado con N trailers** (D3a): el lote entero va en un único
+  `commit-tree -S` de la persona, con una línea `Plan-Blob: <blob>` por
+  tarea, en orden de ID, cada una con el blob de su
+  `TASK-NNN/planificacion/plan-final.md` en el árbol de ese mismo commit, y
+  la línea `Operacion: approve-hito|<n>|<id>|<login>`. El trailer no lleva
+  el ID de la tarea: dos planes idénticos comparten blob (el commit solo
+  cambia los `tarea.md` del lote y `ROADMAP.md`).
+- **Todo o nada** (D2a): si alguna tarea abierta del hito no se puede
+  aprobar (también una `planificada` sin plan), no se escribe ni se firma
+  nada: `rechazada motivo=no-listas tareas=ID:motivo,...` con los motivos
+  de F4-2. Sin ninguna abierta, `hito-vacio`; con más de 20 por aprobar,
+  `demasiadas max=20` (D5a: cuentan solo las que cambiarían); todas ya
+  aprobadas, `sin-cambios` sin firmar.
+- **Nombre y fecha** (D4a): cada tarea recibe la fila `| <instante> |
+  approve | <modo congelado> | persona | <versión> |` (el mismo instante
+  para todo el lote) y `actualizado`; el nombre está en la firma y en la
+  línea `Operacion:`.
+- **Regla 5 del contrato del firmante** (decisión 3): cuando el check
+  `carril` de F6 la implemente pasará a «**el blob actual de `plan-final.md`
+  está entre los trailers `Plan-Blob` del commit que pone `plan_aprobado:
+  true`**», en vez de «el trailer». Un commit de `approve TASK-NNN` tiene
+  uno; uno de `approve --hito`, varios. No se implementa aquí.
+- **Divergencia con el plan de TASK-038**: el plan suponía que la
+  transacción pide `sello()` después de `aplicar`; la pedía antes (para
+  decidir las pendientes). Como el lote solo se conoce en `aplicar`, la
+  transacción ahora vuelve a pedir el sello después de `aplicar` en cada
+  intento del CAS para construir el commit; el de antes solo decide si
+  entran las pendientes. Las operaciones que ya existían devuelven el mismo
+  sello las dos veces.
+- **Medido** (`tests/aprobar.rs`, Git de Windows, clave ed25519 desechable):
+  `approve --hito 4` con 10 tareas deja un solo commit nuevo, `git
+  verify-commit` lo valida y sus 10 `Plan-Blob` son los `rev-parse
+  <commit>:TASK-NNN/planificacion/plan-final.md`. **No verificado**: la
+  firma con una persona real y FIDO en Windows (como en F4-2).
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-038/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-038/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-038/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````
