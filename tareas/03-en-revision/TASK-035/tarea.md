---
id: TASK-035
titulo: "F3-8 doctor: entorno, rama de estado, permisos, rulesets y checks"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-035-f3-8-doctor-entorno-rama-de-estado-permi
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1603906
tokens_implementacion: 43364003
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-17
dependencias: [TASK-028]
---
## Objetivo

`doctor`: entorno, rama de estado, permisos, rulesets y checks reales

Tarea F3-8 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F3-1.

## Criterios de aceptacion
- [ ] C1: con el mock de API sin ruleset `taskcodetwo-develop` (GitHub) o sin `develop` protegida (GitLab) sale `doctor rechazada fallos>=1` con el hallazgo `proteccion develop falla motivo=proteccion-ausente arreglo=tc2-init` y codigo 3
- [ ] C2: con `taskcodetwo-develop` y `taskcodetwo-main` presentes con `pull_request` no hay hallazgo de proteccion y, sin otros fallos, el codigo es 0
- [ ] C3: rama de estado ausente da `falla motivo=sin-rama-de-estado arreglo=tc2-init`; corrupta (un `tarea.md` ilegible o la ref ilegible) da `falla` con los ID o la ref
- [ ] C4: un 403 en la lectura de rulesets da `falla motivo=permiso-insuficiente permiso=...`; un 403 en los permisos de la CI da su permiso; si la CI puede aprobar, `falla motivo=ci-puede-aprobar`
- [ ] C5: un check requerido que no aparece en los checks publicados de la cabeza de `develop` da `falla motivo=check-sin-publicar` (o `aviso` si no hay ninguno publicado); un check del modelo no obligatorio da `falla motivo=check-no-obligatorio`
- [ ] C6: sin `git` o con version irreconocible, `falla`; por debajo de 2.56, `aviso motivo=git-no-verificado`; fuera de un repo, uso o rechazada con diagnostico en stderr
- [ ] C7: como mucho 5 lineas en stdout con la primera fija y `y N mas: tc2 doctor --json`; `--json` lleva todos los hallazgos y nunca ANSI
- [ ] C8: codigo 0 sin ninguna `falla` aunque haya avisos o no-comprobados; 3 con alguna `falla`
- [ ] C9: sin red, sin credenciales o sin plataforma sale `plataforma=sin-red|sin-credenciales|no-configurada` y los hallazgos de plataforma `no-comprobado`; nunca da por buena una proteccion sin leerla
- [ ] C10: no escribe nada: refs, `git worktree list`, arbol de trabajo, `<git-common-dir>/taskcodetwo/` y configuracion identicos antes y despues; el mock solo recibe GET
- [ ] C11: `capacidades` (F1-13 decision 3) con la tabla G01..G20 y el plan observado: con el mock de distintos planes en GitHub y GitLab, `doctor` dice las garantias sustituidas o ausentes con su motivo y arreglo, y el mismo caso da la misma linea en las dos plataformas

## Autorrevision

- rust-wasm-reviewer — d505e92
- code-quality-reviewer — d505e92

Hallazgos propios (ninguno CRITICO ni IMPORTANTE):

- MENOR: todo lo de plataforma esta probado contra el mock, no contra
  GitHub ni GitLab reales; algunos campos (`required_status_checks` de
  `rules/branches`, `only_allow_merge_if_pipeline_succeeds`, `plan` de
  namespaces) solo constan en esquemas o ejemplos de la documentacion.
- MENOR: `enterprise-privado-org` no se distingue de Team con lo
  observable; sale como Team (lo conservador).
- MENOR: la construccion del adaptador repetida en `init/inventario.rs`
  sigue como deuda (doctor no la copia).
- Desviaciones del plan anotadas en las adendas de F1-13 y F1-4: plan de
  GitLab por `GET /namespaces/:id`; la linea `doctor puertas` de F1-13 C
  pasa a la primera linea y a hallazgos `garantia Gnn`; identidad de la
  persona via `leer-permisos-ci`.
- Mutaciones comprobadas (6, rojo y restauradas): `solo_pr` en la
  proteccion, `check-no-obligatorio`, ref ilegible, `NoComprobado` sin
  plataforma, garantia no disponible por config, codigo 3.
- Verificacion propia: `--test doctor --test tablero` 16 de 16.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T13:23:19Z | plan | automatico | persona | 0.16.1 |
| 2026-11-17T13:29:49Z | approve | automatico | persona | 0.16.1 |
| 2026-11-17T13:29:50Z | start | automatico | persona | 0.16.1 |
| 2026-11-17T14:36:45Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-17T13:29:48Z | diseno | agente:a1395d1a890a33e28 | 48 | 99388 | 1485737 | 18733 | 40 | 1603906 | - | - |
| 2026-11-17T14:36:42Z | implementacion | agente:ae9f386d5583740b3 | 234 | 436140 | 39943070 | 144068 | 52 | 40523512 | - | - |
| 2026-11-17T14:36:45Z | implementacion | sesion:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 30 | 16531 | 2819010 | 4920 | 0 | 2840491 | - | - |
