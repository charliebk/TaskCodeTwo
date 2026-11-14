---
id: TASK-001
titulo: "F1-0 Crear TaskCodeTwo"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-001-f1-0-crear-taskcodetwo
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: 2244089
tokens_revision: 1193273
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: []
---
## Objetivo

Crear TaskCodeTwo: repo en GitHub, `develop` y `main`, TaskCode 0.16 instalado, `tareas/`, config, CI mínima; importar este plan con `taskctl import` (RR-14)

Tarea F1-0 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: —.

## Criterios de aceptacion
- [x] `taskctl doctor` en verde
- [x] las tareas F1..F9 planificadas con dependencias
- [x] Cumple lo que pide la tarea F1-0 del plan y los requisitos que cita

## Resultado

**Ya estaba hecho antes de la tarea** (comprobado el 2026-11-13): repo público
`charliebk/TaskCodeTwo` con licencia MIT; `develop` (por defecto) y `main` en
local y en `origin`; TaskCode 0.16.1 instalado; `tareas/` con las cinco
carpetas; `.taskcode/config.yml` válida; plan importado (72 tareas,
TASK-001..072 = F1-0..F9-9, sprint = fase).

**Implementado en la tarea:**
- `.github/workflows/ci.yml`: corre en push a cualquier rama y en PR contra
  `develop`/`main`; `permissions: contents: read`; `actions/checkout` fijado
  por SHA (`3d3c42e`, v7.0.1, tomado de `gh api`); sin secretos.
- `.github/scripts/comprobar-repo.sh` (bash, solo builtins): comprueba las cinco
  carpetas de estado, el frontmatter de cada tarea (`id` = carpeta y único,
  `estado` = carpeta de estado), que las dependencias existen y los enlaces
  relativos de `docs/`, `README.md` y `CLAUDE.md`.
- `.gitattributes`: `*.sh text eol=lf`, para que el script funcione en local en
  Windows.
- En `develop`, antes de la rama: `*.iml` en `.gitignore` (`b23f213`).

**Desviaciones del plan:** la CI corre en push a cualquier rama, y no solo a
`develop`/`main`. Con `cierre: merge` no hay PR y, si no, la rama de la tarea no
se verificaría nunca. El `.gitattributes` tampoco estaba previsto.

**Evidencia:**
- `taskctl doctor`: 0 errores, 0 avisos, 15 ok, 4 omitidas.
- `comprobar-repo.sh` sobre el repo: `Tareas: 72; enlaces relativos: 11; fallos: 0`.
- Prueba negativa en una copia temporal (scratchpad, ya borrada): detecta la
  carpeta de estado que falta, el estado en otra carpeta, el `id` cambiado y
  repetido, el frontmatter sin `---`, la dependencia `TASK-999` y el enlace roto.
- Dependencias frente a la tabla «Dep.» del plan: un script fuera del repo da 0
  diferencias reales. Las 3 que marca son comodines (`F1-*`, `F3-*`, `F7-*`),
  bien expandidos en las tareas.
- GitHub Actions en verde: run `43545676298` sobre `5443f0b` y run
  `43546241802` sobre `f1a1969` (el commit revisado más la petición).

**Revisión (ronda 1, code-quality-reviewer, aprobada):** sin CRÍTICO ni
IMPORTANTE. Seis MENORES, todos `aceptado` y con motivo en
`revision/informe-revision-1.md`:
- MEN-1: enlaces en código en línea y ``` sin cerrar.
- MEN-2: `%20`, otros esquemas y paréntesis.
- MEN-3: `dependencias:` escalar.
- MEN-4: comentario YAML en `id`/`estado`.
- MEN-5: lo que no es `TASK-*` se ignora.
- MEN-6: falta un run sobre el commit revisado. Queda resuelto con el run
  `43546241802`.

Se aceptan porque el script es transitorio (F1-1, F2-1 y F2-5 lo sustituyen) y
los casos no se dan con lo que genera `taskctl`.

**Incidentes:**
- Dos terminales abrieron el diseño a la vez (`3481d36` y `c117c4b`). La
  aprobación del otro terminal (`df23f4f`) no correspondía a la ronda vigente y
  se revirtió con `5cded15`, sin reescribir el historial. El otro terminal
  borró además su rama `feature/...` y su commit de `start`, que no tenían nada
  que conservar.
- El revisor dejó un momento el repo real en HEAD desacoplado (`bbc1447`) por
  un error en su comando. Se comprobó después que el repo está limpio, sin
  commits ni push.

**Coste de diseño:** no se registra. El plan se redactó sin subagentes.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T01:58:40Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T01:58:57Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T02:15:35Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T02:16:09Z | start | automatico | persona | 0.16.1 |
| 2026-11-14T02:20:25Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T02:26:34Z | finish | automatico | persona | 0.16.1 |

## Autorrevision

- code-quality-reviewer — 5443f0b

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T02:20:25Z | implementacion | sesion:5e3197c3-350b-4df4-90b0-6379334d5622 | 32 | 19896 | 2214122 | 10039 | 1 | 2244089 | - | - |
| 2026-11-14T02:25:11Z | revision | agente:ae59ffcaea4223f03 | 24 | 108148 | 1074799 | 10302 | 2 | 1193273 | 1 | - |

## Duracion

- Estimada: 30 min
- Real (de la rama abierta al cierre con merge, sin pausas): 10 min
- Cierre (finish): 8 s, de 2026-11-14T02:26:26Z a 2026-11-14T02:26:34Z
