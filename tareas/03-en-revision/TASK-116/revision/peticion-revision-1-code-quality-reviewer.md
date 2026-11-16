# Peticion de revision — TASK-116 (ronda 1)

- Tarea: TASK-116 — Nombres de la 2.0 alineados con F1-12 D1a
- Rama revisada: fix/task-116-nombres-de-la-2-0-alineados-con-f1-12-d1
- Rama base: develop
- Commit revisado (HEAD): 92900522ace4f8328546b838b04358cb15a3d298
- Fecha: 2026-11-16
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-116 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-116/tarea.md
- tareas/03-en-revision/TASK-116/planificacion/
- docs/adr/F1-12-actualizacion-entre-versiones.md
- docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
- docs/adr/F1-3-flujo-de-ramas-worktrees-release.md
- docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
- docs/plan/PLAN-TASKCODE-2.md

## Commits a revisar (git log develop..HEAD)

````
9290052 docs(TASK-116): autorrevision con rust-wasm-reviewer y code-quality-reviewer
03c6f87 test(TASK-116): la huella de la 0.16 queda intacta tras new y start
7f57135 fix(TASK-116): nombres de la 2.0 bajo taskcodetwo segun F1-12 D1a
4e0a8d3 docs(TASK-116): adenda de nombres en F1-12 y cabeceras de F1-2, F1-3 y F1-4
3d3a7f5 refactor(TASK-116): huella.rs como unica fuente de los nombres de la 2.0
7b3b49e chore(TASK-116): tarea en curso
````

## Diff de tu dominio (5 fichero(s) de 31; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/docs/adr/F1-12-actualizacion-entre-versiones.md b/docs/adr/F1-12-actualizacion-entre-versiones.md
index 91c41e3..2f19974 100644
--- a/docs/adr/F1-12-actualizacion-entre-versiones.md
+++ b/docs/adr/F1-12-actualizacion-entre-versiones.md
@@ -544,6 +544,36 @@ Las órdenes que esos ADR nombran para la 2.0 (`taskctl actualizar`,
 `taskctl doctor`, `taskctl tareas --json`, `taskctl confiar`,
 `taskctl derivados`…) son `tc2 actualizar`, `tc2 doctor`, etc.
 
+#### Adenda de TASK-116 (2026-11-16): todos los nombres de la huella
+
+La nota anterior solo cubría la rama y `.git/taskcode`. Al implementar
+F2-2..F2-13 quedaron en el código otros nombres que chocan con la 0.16 o no
+se reconocen como de la 2.0; TASK-116 los alinea con D1a. Son la **única
+lectura válida** de F1-2, F1-3, F1-4 y F1-16 donde nombran estas piezas, y
+salen todos de un único módulo, `crates/tc2/src/huella.rs`:
+
+| Pieza | Nombre en F1-2/F1-3/F1-4/F1-16 y en el código anterior | Nombre de la 2.0 |
+|---|---|---|
+| Rama de estado | `taskcode/tareas` | **`taskcodetwo/tareas`** |
+| Cerrojo local del clon | `.git/taskcode-estado.lock` | **`.git/taskcodetwo/estado.lock`** |
+| Caché de identidad (F2-13) | `.git/tc2/identidad.yml` | **`.git/taskcodetwo/identidad.yml`** |
+| Índices temporales de la transacción | `.git/taskcode-estado-tmp/` | **`.git/taskcodetwo/estado-tmp/`** |
+| Refs de pendientes y rechazadas | `refs/taskcode/{pendientes,rechazadas}/` | **`refs/taskcodetwo/{pendientes,rechazadas}/`** |
+| Worktrees de tarea | `.taskcode/worktrees/TASK-NNN` | **`.taskcodetwo/worktrees/TASK-NNN`** (sigue dentro del proyecto y excluida por `info/exclude`, F1-3 D2a) |
+| Config del proyecto | `.taskcode/config.yml`, `personas.yml`, vocabulario | **sin cambio**: compartida con la 0.16 (D4a) |
+
+Lo que deja la 0.16.1 en un repo (medido en su código y en OpenDockTrading
+y ToolsaWeb el 2026-11-16): `.git/taskcode/` (`puertas.log`, `gitflow/`,
+`cadena.lock`, `progreso.json`, `progreso-contador`,
+`freno-ultimo-verde.json`), `.taskcode/config.yml`,
+`.taskcode/doctor-deploy.yml` y `tareas/` en las ramas de código. No usa
+`refs/taskcode/`, `.taskcode/worktrees` ni nada con `taskcodetwo`, así que
+los nombres nuevos no comparten nada con ella. La 2.0 no estaba desplegada
+en ningún repo real (ni rama `taskcode*`, ni `refs/taskcode`, ni
+`.git/tc2`, ni `.git/taskcode-estado.lock`): el cambio no necesita migración.
+`tests/convivencia_016.rs` comprueba que `alta` y `crear_rama_y_worktree`
+dejan intacta la huella de la 0.16.
+
 ### Divergencia con el plan
 
 1. **Recuento de OpenDockTrading**: el plan decía 33 planificadas, 61
diff --git a/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md b/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
index f06e374..6c3271e 100644
--- a/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
+++ b/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
@@ -5,6 +5,7 @@
 - Requisitos: RR-01, RR-09, RR-12, RN-05, RN-06 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); H3 de la [auditoría externa](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md); se apoya en RF-02 y RF-07
 - Depende de: [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) (carpeta fija `TASK-NNN/` en la rama huérfana `taskcode/tareas`, montada en `tareas/`)
 - Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2b, D3a, D4a del plan de TASK-003)
+- Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
 
 ## Contexto
 
diff --git a/docs/adr/F1-3-flujo-de-ramas-worktrees-release.md b/docs/adr/F1-3-flujo-de-ramas-worktrees-release.md
index a3138fe..b5281dc 100644
--- a/docs/adr/F1-3-flujo-de-ramas-worktrees-release.md
+++ b/docs/adr/F1-3-flujo-de-ramas-worktrees-release.md
@@ -5,6 +5,7 @@
 - Requisitos: RR-06, RR-01, RR-12 (la parte de ramas), RR-07, RF-20, RF-24, RF-25, RF-26, RF-42, RF-53 y §2 punto 5 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); H4 de la [auditoría externa](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md)
 - Depende de: [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) (estado en `taskcode/tareas`, montada en `tareas/`; límite de un worktree por rama) y [ADR F1-2](F1-2-escrituras-atomicas-ids-y-toma.md) (punto único de escritura, re-ejecución en vez de rebase, `start` publica el `asignado` antes de crear la rama)
 - Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a del plan de TASK-004)
+- Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
 
 ## Contexto
 
diff --git a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
index a65cc74..6c9486c 100644
--- a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
+++ b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
@@ -5,6 +5,7 @@
 - Requisitos: RR-01, RR-16, RF-23, RF-24, RF-25 y RN-05 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); H9, H21 y H22 de la [auditoría externa](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md)
 - Depende de: [ADR F1-2](F1-2-escrituras-atomicas-ids-y-toma.md) (punto único de escritura, re-ejecución en vez de rebase, códigos 0/3/4/5) y [ADR F1-3](F1-3-flujo-de-ramas-worktrees-release.md) (ramas, worktrees, ciclo por comando y tabla de scripts de Git-Flow)
 - Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a, D5a del plan de TASK-005)
+- Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
 
 ## Contexto
 
diff --git a/docs/plan/PLAN-TASKCODE-2.md b/docs/plan/PLAN-TASKCODE-2.md
index 0e9a33f..40d54ee 100644
--- a/docs/plan/PLAN-TASKCODE-2.md
+++ b/docs/plan/PLAN-TASKCODE-2.md
@@ -75,7 +75,7 @@ aportación de TaskCodeTwo.
   y descartadas, con su grafo en Mermaid. Se lee en GitHub, en GitLab o en el
   editor.
 - **Nada de esto viaja en las ramas de código.** Vive en una rama propia,
-  `taskcode/tareas`, que el CLI monta como worktree en la carpeta `tareas/` del
+  `taskcodetwo/tareas`, que el CLI monta como worktree en la carpeta `tareas/` del
   proyecto (ignorada en las ramas de código).
 - Consecuencias:
   - **Ninguna rama de código lleva cambios de tareas**: cero conflictos de
@@ -91,7 +91,7 @@ aportación de TaskCodeTwo.
 
 ### P2 — Escrituras de estado atómicas: IDs únicos y nadie pisa a nadie (RR-09, RR-12)
 
-- Cada cambio de estado es un commit pequeño del CLI en `taskcode/tareas` que se
+- Cada cambio de estado es un commit pequeño del CLI en `taskcodetwo/tareas` que se
   publica con push inmediato. Si el push se rechaza porque otro escribió
   antes, el CLI rehace el cambio sobre lo nuevo y reintenta; el roadmap se
   regenera tras el rebase, nunca se mergea.
@@ -303,7 +303,7 @@ dependencias. Las fases 1 a 9 son tareas de TaskCodeTwo; la 0, del repo TaskCode
 | ID | Tarea | Criterio de aceptación | Dep. |
 |---|---|---|---|
 | F1-0 | Crear TaskCodeTwo: repo en GitHub, `develop` y `main`, TaskCode 0.16 instalado, `tareas/`, config, CI mínima; importar este plan con `taskctl import` (RR-14) | `taskctl doctor` en verde; las tareas F1..F9 planificadas con dependencias | — |
-| F1-1 | ADR: carpeta fija por tarea con estado como campo, en rama propia `taskcode/tareas` montada como worktree en `tareas/` (P1), frente a estado en `develop`; contenido del ROADMAP | Decisión con pruebas de concepto de las dos; cómo se ve en GitHub | F1-0 |
+| F1-1 | ADR: carpeta fija por tarea con estado como campo, en rama propia `taskcodetwo/tareas` montada como worktree en `tareas/` (P1), frente a estado en `develop`; contenido del ROADMAP | Decisión con pruebas de concepto de las dos; cómo se ve en GitHub | F1-0 |
 | F1-2 | ADR: escrituras atómicas, reserva de IDs y toma de tareas (P2) | Protocolo de push con reintento; qué pasa sin red | F1-1 |
 | F1-3 | ADR: flujo de ramas `develop`/`main`, rama por tarea, worktrees, release y hotfix (P4) | Nombres, ciclo completo, qué queda de Git-Flow | F1-0 |
 | F1-4 | ADR: capa de operaciones Git/GitHub deterministas y su contrato de salida para el agente (P5) | Lista cerrada de operaciones; objetivo de tokens por tarea | F1-3 |
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-116/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-116/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-116/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````
