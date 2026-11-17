# Peticion de revision — TASK-032 (ronda 1)

- Tarea: TASK-032 — F3-5 tc2 board y tc2 status
- Rama revisada: feature/task-032-f3-5-board-y-status
- Rama base: develop
- Commit revisado (HEAD): 5a5a9e9e08d7cc4180d8dd4ee3760a24ec78677a
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-032 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-032/tarea.md
- tareas/03-en-revision/TASK-032/planificacion/
- docs/adr/F1-13-adaptador-de-plataforma.md
- docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md

## Commits a revisar (git log develop..HEAD)

````
5a5a9e9 docs(TASK-032): autorrevision con rust-wasm-reviewer y code-quality-reviewer
8a3a2c5 feat(TASK-032): board cruza los PR por rama con modos degradados
55b3bf0 feat(TASK-032): prs-abiertos en el puerto Plataforma con adenda a F1-4 y F1-13
7591793 feat(TASK-032): tc2 status y tc2 board sobre el estado, sin plataforma
db88831 refactor(TASK-032): quien y rama de la tarea en situacion
03b47aa chore(TASK-032): tarea en curso
````

## Diff de tu dominio (2 fichero(s) de 17; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/docs/adr/F1-13-adaptador-de-plataforma.md b/docs/adr/F1-13-adaptador-de-plataforma.md
index d0d2c85..ff2c18a 100644
--- a/docs/adr/F1-13-adaptador-de-plataforma.md
+++ b/docs/adr/F1-13-adaptador-de-plataforma.md
@@ -4,6 +4,7 @@
 - Tarea: TASK-015 · Plan: fila F1-13 del [PLAN-TASKCODE-2](../plan/PLAN-TASKCODE-2.md) y su apartado P11
 - Requisitos: RR-15 (GitHub y GitLab desde la 2.0, con la diferencia documentada y dicha por `doctor`), RF-54 (siempre por PR o MR, GitLab detrás del adaptador), RF-64 (`doctor` con el arreglo de cada fallo), RN-01 (identidad propia del agente) y RR-08 (las garantías las impone la plataforma) de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); RR-12 (equipo) para la interferencia en un MR compartido
 - Depende de: [ADR F1-4](F1-4-operaciones-git-y-contrato-de-salida.md) (puerto `Plataforma`, lista cerrada, contrato de salida y códigos 0–7), [ADR F1-5](F1-5-puertas-en-github.md) (modelo neutral de puertas, `aplicar-puertas`, `leer-puertas`, D2a y D3a), [ADR F1-6](F1-6-identidades-agente-revisor-persona.md) (operación → identidad, contrato del firmante, decisión 6), [ADR F1-7](F1-7-verificacion-ci-perfiles-cache-runners.md) (caché en el check `verificacion`), [ADR F1-8](F1-8-seguridad-modelo-de-amenazas.md) (dos clases de jobs, `carril` con la definición de la base, runner propio), [ADR F1-10](F1-10-carriles-y-modo-automatico.md) (carril mínimo con los dos nombres de cada renombrado), [ADR F1-11](F1-11-extensiones-declaradas.md) (comandos sí, hooks solo como atajo opcional), [ADR F1-12](F1-12-actualizacion-entre-versiones.md) (nombres `tc2` y `taskcodetwo`) y [ADR F1-14](F1-14-distribucion-e-instalacion.md) (HTTP con TLS del sistema, sin `gh` ni `glab`)
+- Adenda (2026-11-17, TASK-032, F3-5): operación nueva de lectura `prs-abiertos(rol)` → lista de `{número, rama, sha, borrador}` de los PR/MR abiertos **del propio repo**, por número; campos mínimos (D2 y D3 de TASK-032: sin checks ni aprobaciones). Rol `agente` o `persona` (el `revisor` se rechaza); permiso `pull_requests:read` en GitHub (el mismo que ya usa `abrir-pr` para buscar el PR existente) y el permiso de lectura del adaptador en GitLab (`read_api` o `api` y rol Developer, el mismo de `abrir-pr`). Los PR de forks no cuentan: en GitHub se descarta el que tiene `head.repo.id` distinto de `base.repo.id` (o nulo, fork borrado); en GitLab, el que tiene `source_project_id` distinto de `target_project_id`. Borrador: `draft` en GitHub; `draft` o `work_in_progress` (obsoleto) en GitLab. Campos de GitLab comprobados contra la documentación de la API de MR (2026-11-17), no contra la API real. Una rama que no pasa `rama_valida` se salta (no puede casar con una tarea); un sha que no es de 40 hex es `respuesta-invalida`. Paginación con `Link rel=next` hasta `MAX_PAGINAS` (10 × 100); con más, `error motivo=demasiadas-paginas` y `board` sale con `plataforma=error`. El tope HTTP de 30 s es **por llamada**, no por operación: en el peor caso una consulta paginada podría esperar 10 × 30 s; sin red cae en la primera llamada (tope de conexión de 10 s). Probado contra el mock (`tests/prs_abiertos.rs`), no contra GitHub ni GitLab reales
 - Decide: Carlos, con sus decisiones del 2026-11-14 (nombres `tc2`, `taskcodetwo` y rama `taskcodetwo/tareas`; comandos deterministas antes que hooks; seguridad y trabajo en grupo imprescindibles; sin trocear y fiel a los datos; GitHub y GitLab solo lectura) y la consigna «la opción recomendada» (D1a a D8a del plan de TASK-015)
 
 ## Contexto
@@ -377,6 +378,7 @@ no se ha configurado nada en ninguna plataforma):
    | `leer-puertas` | `GET /rulesets`, `GET /rules/branches/{rama}` | `GET /protected_branches`, `GET /protected_tags`, `GET /projects/:id` con sesión, `GET /approval_rules`, `GET /approvals` |
    | `leer-permisos-ci` | `GET /actions/permissions/workflow` | ninguna: el job token es fijo (G16); informa `fuente=plataforma` |
    | firma del `approve` (`carril`) | `GET /commits/{sha}` | `GET /repository/commits/{sha}/signature` |
+   | `prs-abiertos` (adenda de TASK-032) | `GET /pulls?state=open&per_page=100`, paginado | `GET /merge_requests?state=opened&per_page=100`, paginado |
 
    La caché de F1-7 cuesta lo mismo en las dos: una lectura por commit (hasta
    7, F1-7) con `check-runs` o con `pipelines?sha=`.
diff --git a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
index 6c9486c..2954d6e 100644
--- a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
+++ b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
@@ -5,6 +5,7 @@
 - Requisitos: RR-01, RR-16, RF-23, RF-24, RF-25 y RN-05 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); H9, H21 y H22 de la [auditoría externa](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md)
 - Depende de: [ADR F1-2](F1-2-escrituras-atomicas-ids-y-toma.md) (punto único de escritura, re-ejecución en vez de rebase, códigos 0/3/4/5) y [ADR F1-3](F1-3-flujo-de-ramas-worktrees-release.md) (ramas, worktrees, ciclo por comando y tabla de scripts de Git-Flow)
 - Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a, D5a del plan de TASK-005)
+- Adenda (2026-11-17, TASK-032, F3-5): entra `prs-abiertos` en la lista cerrada (decisión 2) para `tc2 board` (D2 de TASK-032, aceptada por Carlos el 2026-11-16); detalle de llamadas en la [adenda de F1-13](F1-13-adaptador-de-plataforma.md). `board` y `status`, como `roadmap`, escriben avisos de una línea en stderr con código 0; el canal fiable son las claves de la línea y del JSON
 - Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
 
 ## Contexto
@@ -406,6 +407,7 @@ para esta PoC: **no verificado** en la plataforma.
    | `abrir-pr` | `review` | plataforma; devuelve el PR que haya (no verificado) |
    | `estado-puertas` | `review`, `finish` | plataforma; lectura (no verificado) |
    | `merge-pr` | `finish`, `release` | plataforma; merge commit (no verificado) |
+   | `prs-abiertos` | `board` (adenda de TASK-032) | plataforma; lectura de los PR abiertos del propio repo (número, rama, sha, borrador); sin token del rol → `rechazada motivo=identidad`; más de 10 páginas → `error motivo=demasiadas-paginas` (contra el mock, no contra la API real) |
 
 3. **Prohibidas** para la capa y para el agente: `rebase`, `push --force` (y
    `-f`, `--force-with-lease`), `reset --hard`, `stash`, `clean`, `merge` con
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-032/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-032/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-032/tarea.md                                    | 23 ++++++++++++++++++++++-
 3 files changed, 22 insertions(+), 1 deletion(-)
````
