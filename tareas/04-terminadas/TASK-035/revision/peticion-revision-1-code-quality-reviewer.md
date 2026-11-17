# Peticion de revision — TASK-035 (ronda 1)

- Tarea: TASK-035 — F3-8 doctor: entorno, rama de estado, permisos, rulesets y checks
- Rama revisada: feature/task-035-f3-8-doctor-entorno-rama-de-estado-permi
- Rama base: develop
- Commit revisado (HEAD): 1aa0a12edd779a7211e471d9132d566be737d3da
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-035 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-035/tarea.md
- tareas/03-en-revision/TASK-035/planificacion/
- docs/adr/F1-13-adaptador-de-plataforma.md
- docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md

## Commits a revisar (git log develop..HEAD)

````
1aa0a12 docs(TASK-035): autorrevision con rust-wasm-reviewer y code-quality-reviewer
d505e92 docs(TASK-035): adendas de F1-13 y F1-4 por doctor
7fb0bf0 feat(TASK-035): capacidades G01..G20 con el plan observado en doctor
7c143e9 feat(TASK-035): checks requeridos y publicados en leer-puertas y doctor
dfc9098 refactor(TASK-035): conexion con la plataforma en identidad y proteccion en doctor
a3363c8 feat(TASK-035): doctor lee la rama de estado y el plan de init
529d54a feat(TASK-035): esqueleto de doctor, agregador, salida y entorno
c2c9e8a chore(TASK-035): tarea en curso
````

## Diff de tu dominio (2 fichero(s) de 18; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/docs/adr/F1-13-adaptador-de-plataforma.md b/docs/adr/F1-13-adaptador-de-plataforma.md
index ff2c18a..a0a2d50 100644
--- a/docs/adr/F1-13-adaptador-de-plataforma.md
+++ b/docs/adr/F1-13-adaptador-de-plataforma.md
@@ -5,6 +5,7 @@
 - Requisitos: RR-15 (GitHub y GitLab desde la 2.0, con la diferencia documentada y dicha por `doctor`), RF-54 (siempre por PR o MR, GitLab detrás del adaptador), RF-64 (`doctor` con el arreglo de cada fallo), RN-01 (identidad propia del agente) y RR-08 (las garantías las impone la plataforma) de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); RR-12 (equipo) para la interferencia en un MR compartido
 - Depende de: [ADR F1-4](F1-4-operaciones-git-y-contrato-de-salida.md) (puerto `Plataforma`, lista cerrada, contrato de salida y códigos 0–7), [ADR F1-5](F1-5-puertas-en-github.md) (modelo neutral de puertas, `aplicar-puertas`, `leer-puertas`, D2a y D3a), [ADR F1-6](F1-6-identidades-agente-revisor-persona.md) (operación → identidad, contrato del firmante, decisión 6), [ADR F1-7](F1-7-verificacion-ci-perfiles-cache-runners.md) (caché en el check `verificacion`), [ADR F1-8](F1-8-seguridad-modelo-de-amenazas.md) (dos clases de jobs, `carril` con la definición de la base, runner propio), [ADR F1-10](F1-10-carriles-y-modo-automatico.md) (carril mínimo con los dos nombres de cada renombrado), [ADR F1-11](F1-11-extensiones-declaradas.md) (comandos sí, hooks solo como atajo opcional), [ADR F1-12](F1-12-actualizacion-entre-versiones.md) (nombres `tc2` y `taskcodetwo`) y [ADR F1-14](F1-14-distribucion-e-instalacion.md) (HTTP con TLS del sistema, sin `gh` ni `glab`)
 - Adenda (2026-11-17, TASK-032, F3-5): operación nueva de lectura `prs-abiertos(rol)` → lista de `{número, rama, sha, borrador}` de los PR/MR abiertos **del propio repo**, por número; campos mínimos (D2 y D3 de TASK-032: sin checks ni aprobaciones). Rol `agente` o `persona` (el `revisor` se rechaza); permiso `pull_requests:read` en GitHub (el mismo que ya usa `abrir-pr` para buscar el PR existente) y el permiso de lectura del adaptador en GitLab (`read_api` o `api` y rol Developer, el mismo de `abrir-pr`). Los PR de forks no cuentan: en GitHub se descarta el que tiene `head.repo.id` distinto de `base.repo.id` (o nulo, fork borrado); en GitLab, el que tiene `source_project_id` distinto de `target_project_id`. Borrador: `draft` en GitHub; `draft` o `work_in_progress` (obsoleto) en GitLab. Campos de GitLab comprobados contra la documentación de la API de MR (2026-11-17), no contra la API real. Una rama que no pasa `rama_valida` se salta (no puede casar con una tarea); un sha que no es de 40 hex es `respuesta-invalida`. Paginación con `Link rel=next` hasta `MAX_PAGINAS` (10 × 100); con más, `error motivo=demasiadas-paginas` y `board` sale con `plataforma=error`. El tope HTTP de 30 s es **por llamada**, no por operación: en el peor caso una consulta paginada podría esperar 10 × 30 s; sin red cae en la primera llamada (tope de conexión de 10 s). Probado contra el mock (`tests/prs_abiertos.rs`), no contra GitHub ni GitLab reales
+- Adenda (2026-11-17, TASK-035, F3-8): **`leer-puertas` gana lecturas** (decisión 4) y devuelve además `aprobaciones`, `checks_requeridos`, `todos_los_checks` y `checks_publicados`. GitHub: los `parameters` de las reglas que ya trae `GET /rules/branches/{rama}` (`required_status_checks[].context`, `required_approving_review_count`) y una llamada más, `GET /commits/heads/{rama}/check-runs` (la página da la rama como `heads/BRANCH_NAME`: [doc](https://docs.github.com/en/rest/checks/runs), 2026-11-17; esquema de reglas: [doc](https://docs.github.com/en/rest/repos/rules), 2026-11-17, que no trae ejemplo de respuesta: los nombres salen del esquema de rulesets). GitLab: `GET /projects/:id` (`only_allow_merge_if_pipeline_succeeds`: GitLab no exige checks por nombre; con «Pipelines must succeed» cuenta el pipeline entero, `todos_los_checks`), `GET /repository/branches/:rama` (`commit.id`: [doc](https://docs.gitlab.com/api/branches/), 2026-11-17) y `GET /repository/commits/:sha/statuses` (la página documenta `:sha` como «hash of the commit» y no dice que acepte una rama: [doc](https://docs.gitlab.com/api/commits/), 2026-11-17; por eso se lee antes la rama). El campo `only_allow_merge_if_pipeline_succeeds` sale en los ejemplos de respuesta de la [API de proyectos](https://docs.gitlab.com/api/projects/) (2026-11-17), no en su tabla de atributos: si no viene, `checks_requeridos` es `None` y `doctor` dice `no-comprobado`. Lo que no se lee nunca se da por bueno. **Operación nueva de lectura `leer-plan(rol)`** (agente o persona): GitHub, `GET /repos/{o}/{r}` (`private`, `visibility`, `owner.type`; [doc](https://docs.github.com/en/rest/repos/repos), 2026-11-17; `internal` cuenta como privado); GitLab, `GET /projects/:id` (`namespace.id`) y `GET /namespaces/:id` (`plan`: la [API de namespaces](https://docs.gitlab.com/api/namespaces/), 2026-11-17, lo da con los ejemplos `free`, `ultimate` y `default` y no dice quién lo ve). Solo `free`, `premium` y `ultimate` se reconocen; cualquier otro valor, o el namespace ilegible, es «plan no comprobado», salvo el 403 de `approval_rules`, que sigue diciendo Free (decisión 3). **Corrige la decisión 3 en GitLab**: el E2E de F2-9 observó que en Free `approval_rules` da 200 con la lista vacía, no 403; por eso el plan sale del namespace. **`capacidades` en F3-8**: la tabla de la decisión 2 compilada (`doctor/capacidades.rs`); nombres de plan `github-free-publico-usuario`, `github-free-privado-usuario`, `github-pro-privado-usuario`, `github-free-publico-org`, `github-team-privado-org`, `github-free-privado-org` (nuevo: 9/1/10) y `gitlab-free|premium|ultimate`; `enterprise-privado-org` no se distingue de Team con lo observable y sale como Team (la cola sustituida: lo conservador). La configuración (`motivo=config arreglo=tc2-init`) solo se comprueba en las garantías que `leer-puertas` lee (G01, G02, G03, G06 en GitLab y G09 en GitHub); las demás siguen al plan. Arreglos: `repo-publico-o-github-pro`, `org-publica-o-enterprise-cloud`, `gitlab-premium`, `ninguno-limite-de-la-plataforma` y `tc2-init`. Línea de `doctor`: las cuentas van en su primera línea (`plan=… impuestas=N sustituidas=M no-disponibles=K`, sin la línea aparte `doctor puertas …` de la sección C, que no cabría en las 5) y cada garantía no impuesta es un hallazgo `garantia Gnn aviso motivo=… capacidad=sustituida por=…|no-disponible arreglo=…`. Probado contra el mock (`tests/doctor.rs`), no contra GitHub ni GitLab reales
 - Decide: Carlos, con sus decisiones del 2026-11-14 (nombres `tc2`, `taskcodetwo` y rama `taskcodetwo/tareas`; comandos deterministas antes que hooks; seguridad y trabajo en grupo imprescindibles; sin trocear y fiel a los datos; GitHub y GitLab solo lectura) y la consigna «la opción recomendada» (D1a a D8a del plan de TASK-015)
 
 ## Contexto
diff --git a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
index 2faab15..7aaa269 100644
--- a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
+++ b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
@@ -8,6 +8,7 @@
 - Adenda (2026-11-17, TASK-032, F3-5): entra `prs-abiertos` en la lista cerrada (decisión 2) para `tc2 board` (D2 de TASK-032, aceptada por Carlos el 2026-11-16); detalle de llamadas en la [adenda de F1-13](F1-13-adaptador-de-plataforma.md). `board` y `status`, como `roadmap`, escriben avisos de una línea en stderr con código 0; el canal fiable son las claves de la línea y del JSON
 - Adenda (2026-11-17, TASK-033, F3-6): `start [--json] TASK-NNN` compone `fetch`, `validar-base`, `comprobar-id`, `commit-estado`, `crear-rama-y-worktree` y `reanudar` sin operaciones nuevas (la base de un `hotfix` es el último tag por versión de `refs/tags/` contenido en `origin/main`, leído con `for-each-ref --merged`, o su punta). Línea de éxito: `start ok id=… rama=… worktree=… base=… estado=<commit de la toma> asignado=<login>`; dueño con la tarea empezada: `sin-cambios` o `ok creado=rama,worktree` sin escribir estado. Motivos de `rechazada` (3): `no-existe`, `formato-1` (con `arreglo=tc2 actualizar`), `ilegible`, `tomada` (con `asignado=<login>`), `asignado-heredado`, `terminada`, `descartada`, `estado-no-valido`, `dependencias-pendientes` (con `dependencias=`), `rama-invalida`, `tipo-release`, `ocupada` (con `ramas=`), `carpeta-ocupada` (la carpeta del worktree existe y no es el worktree registrado de la rama; se comprueba antes de publicar la toma, sin `estado=`) e `identidad`; `uso` (2): `sin-id`, `id-invalido`, `argumentos-de-mas`; sin red, `sinred pendiente=no` (4)
 - Adenda (2026-11-17, TASK-034, F3-7): `descartar [--json] TASK-NNN "<motivo>"`, `pausa [--json] TASK-NNN` y `reanudar [--json] TASK-NNN` componen `fetch` (solo `descartar`), `commit-estado` y `quitar-worktree` (solo `descartar`, tras publicar) sin operaciones nuevas. Líneas: `descartar ok id=… commit=… intentos=… tareas=… worktree=quitado|ninguno|conservado [worktree-motivo=cambios-sin-guardar|sin-publicar|sin-rama|en-uso|error]` (el worktree conservado no hace fallar la orden: el descarte ya está publicado y la rama no se toca nunca); `pausa ok id=… commit=… intentos=… tareas=…` y `reanudar ok id=… commit=… intentos=… tareas=…` (las claves de la transacción, como `descartar`, sin `worktree`). `sin-cambios` (0): descartar una descartada (vale el motivo de entonces y se intenta quitar el worktree), pausar una en pausa o reanudar una que no lo está. Motivos de `rechazada` (3): `no-existe`, `formato-1` (con `arreglo=tc2 actualizar`), `ilegible`, `de-otra-persona` (con `asignado=<login>`; D2: puede quien la lleva o cualquiera si no la lleva nadie), `asignado-heredado`, `terminada`, `descartada` (pausa y reanudar), `estado-no-valido` e `identidad`; `start` gana `pausada` (con `arreglo=tc2 reanudar`). `uso` (2): `sin-id`, `id-invalido`, `argumentos-de-mas` y, en `descartar`, `sin-motivo` y `motivo-invalido` (vacío, solo espacios, saltos de línea o caracteres de control, o más de 500 caracteres). Sin red: `descartar sinred pendiente=no` (4); `pausa`/`reanudar sinred pendiente=si` (4), que la siguiente transacción con red publica (adenda de TASK-034 en F1-2)
+- Adenda (2026-11-17, TASK-035, F3-8): `doctor [--json]` compone lecturas sin escribir nada (ni `fetch`): `git version` entra en la lista permitida (decisión 3; solo lectura), y la lista cerrada de la plataforma gana `leer-plan` (detalle en la [adenda de F1-13](F1-13-adaptador-de-plataforma.md)). Salida: primera línea `doctor <ok|rechazada> fallos=N avisos=N no-comprobado=N plataforma=ok|no-configurada|sin-credenciales|sin-red|error [plan=… impuestas=N sustituidas=N no-disponibles=N] [commit=…]`; detrás, los hallazgos que no son `ok` (`<zona> <id> <falla|aviso|no-comprobado> [motivo=…] [claves] [arreglo=…]`), los peores primero, y si no caben en 5 líneas, 3 y `y N mas: tc2 doctor --json`. `--json`: una línea con las claves y `hallazgos` (todos, también los `ok`). Código 3 (`rechazada`) con alguna `falla` (D5a de TASK-035) y 0 con avisos o no-comprobados; fuera de un repo, `doctor rechazada motivo=no-es-repo` (3); argumentos de más, `uso motivo=argumentos` (2; `--recursos` llega con F3-9). Git por debajo de 2.56: `aviso motivo=git-no-verificado` (D3a de TASK-035); sin git o con una versión que no se entiende, `falla`. Diagnóstico de una línea en stderr solo con alguna `falla`
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
tareas/{01-en-diseno => 02-en-curso}/TASK-035/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-035/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-035/tarea.md                                    | 30 ++++++++++++++++++++++++++++--
 3 files changed, 28 insertions(+), 2 deletions(-)
````
