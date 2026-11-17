# Peticion de revision — TASK-028 (ronda 1)

- Tarea: TASK-028 — F3-1 tc2 init
- Rama revisada: feature/task-028-f3-1-taskctl-init-setup
- Rama base: develop
- Commit revisado (HEAD): 2c3706b7cb68b2e455377ec33ab0766aae68db39
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-028 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-028/tarea.md
- tareas/03-en-revision/TASK-028/planificacion/
- docs/adr/F1-5-puertas-en-github.md

## Commits a revisar (git log develop..HEAD)

````
2c3706b docs(TASK-028): autorrevision con rust-wasm-reviewer y code-quality-reviewer
1713060 fix(TASK-028): orden de rama con historia desde la otra rama, fallo de init simplificado y tests de HEAD en master, historia y fallo al aplicar
07664f6 docs(TASK-028): adenda de F1-5, init propone las puertas y las aplica TASK-118
7f459fb feat(TASK-028): desinstalar reconoce el .gitignore, el workflow y los rulesets de init
128aaec feat(TASK-028): tc2 init con plan como dato, --simular, rama de estado, ramas raiz, config, personas y propuesta de rulesets
cad3998 feat(TASK-028): modelo neutral de puertas de F1-5 y su ruleset de GitHub
fe88f07 refactor(TASK-028): montaje::previsto de solo lectura y montar sobre el
01f9faf refactor(TASK-028): huella de init, linea de accion en contrato y generadores de config y personas
d4b0b7d chore(TASK-028): tarea en curso
````

## Diff de tu dominio (1 fichero(s) de 18; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/docs/adr/F1-5-puertas-en-github.md b/docs/adr/F1-5-puertas-en-github.md
index 44181e0..fa3e631 100644
--- a/docs/adr/F1-5-puertas-en-github.md
+++ b/docs/adr/F1-5-puertas-en-github.md
@@ -6,6 +6,7 @@
 - Depende de: [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) (rama de estado con push directo del CLI), [ADR F1-3](F1-3-flujo-de-ramas-worktrees-release.md) (nombres de rama, merge commit, ramas conservadas, tags `vX.Y.Z`) y [ADR F1-4](F1-4-operaciones-git-y-contrato-de-salida.md) (puerto `Plataforma` y su lista cerrada)
 - Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a, D5a y D6a del plan de TASK-006)
 - Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
+- Adenda (2026-11-17, TASK-028, D1 de su plan): `init` (F3-1) **no aplica** las puertas, aunque la decisión 1 (`aplicar-puertas` en `init`) y las consecuencias de este ADR lo pedían. Las **propone**: imprime una línea por ruleset `taskcodetwo-*` del modelo neutral con `decision=persona` y la orden que lo aplicará (`tc2 init --aplicar-puertas`), y en `--json` el ruleset de GitHub generado. No hace ninguna escritura en la plataforma; solo lee (`GET /user` y `leer-puertas`). Motivo: los rulesets exigen los checks `verificacion`, `revision-ia` y `carril`, que no existen hasta F5-1 y F6, y el `integration_id` real de F6-1. Aplicados hoy, bloquearían todos los PR. La operación `aplicar-puertas` del puerto y su opción en `init` pasan a TASK-118, tras F5-1, F6 y F6-1.
 
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
tareas/{01-en-diseno => 02-en-curso}/TASK-028/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-028/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-028/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````
