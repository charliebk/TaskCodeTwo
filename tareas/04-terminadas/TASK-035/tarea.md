---
id: TASK-035
titulo: "F3-8 doctor: entorno, rama de estado, permisos, rulesets y checks"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-035-f3-8-doctor-entorno-rama-de-estado-permi
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1603906
tokens_implementacion: 70044367
tokens_revision: 14651011
creado: 2026-11-14
actualizado: 2026-11-17
dependencias: [TASK-028]
---
## Objetivo

`doctor`: entorno, rama de estado, permisos, rulesets y checks reales

Tarea F3-8 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F3-1.

## Criterios de aceptacion
- [x] C1: con el mock de API sin ruleset `taskcodetwo-develop` (GitHub) o sin `develop` protegida (GitLab) sale `doctor rechazada fallos>=1` con el hallazgo `proteccion develop falla motivo=proteccion-ausente arreglo=tc2-init` y codigo 3
- [x] C2: con `taskcodetwo-develop` y `taskcodetwo-main` presentes con `pull_request` no hay hallazgo de proteccion y, sin otros fallos, el codigo es 0
- [x] C3: rama de estado ausente da `falla motivo=sin-rama-de-estado arreglo=tc2-init`; corrupta (un `tarea.md` ilegible o la ref ilegible) da `falla` con los ID o la ref
- [x] C4: un 403 en la lectura de rulesets da `falla motivo=permiso-insuficiente permiso=...`; un 403 en los permisos de la CI da su permiso; si la CI puede aprobar, `falla motivo=ci-puede-aprobar`
- [x] C5: un check requerido que no aparece en los checks publicados de la cabeza de `develop` da `falla motivo=check-sin-publicar` (o `aviso` si no hay ninguno publicado); un check del modelo no obligatorio da `falla motivo=check-no-obligatorio`
- [x] C6: sin `git` o con version irreconocible, `falla`; por debajo de 2.56, `aviso motivo=git-no-verificado`; fuera de un repo, uso o rechazada con diagnostico en stderr
- [x] C7: como mucho 5 lineas en stdout con la primera fija y `y N mas: tc2 doctor --json`; `--json` lleva todos los hallazgos y nunca ANSI
- [x] C8: codigo 0 sin ninguna `falla` aunque haya avisos o no-comprobados; 3 con alguna `falla`
- [x] C9: sin red, sin credenciales o sin plataforma sale `plataforma=sin-red|sin-credenciales|no-configurada` y los hallazgos de plataforma `no-comprobado`; nunca da por buena una proteccion sin leerla
- [x] C10: no escribe nada: refs, `git worktree list`, arbol de trabajo, `<git-common-dir>/taskcodetwo/` y configuracion identicos antes y despues; el mock solo recibe GET
- [x] C11: `capacidades` (F1-13 decision 3) con la tabla G01..G20 y el plan observado: con el mock de distintos planes en GitHub y GitLab, `doctor` dice las garantias sustituidas o ausentes con su motivo y arreglo, y el mismo caso da la misma linea en las dos plataformas

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

## Resultado

Hecho: `tc2 doctor [--json]` (modulo `doctor/`), orden de solo lectura que junta entorno (version de git con aviso por debajo de 2.56, tc2, repo, config de la base), rama de estado (ausente, ilegible o ref corrupta, sobre la misma ref que `estado::leer`), instalacion (inventario de `init` en simulacion, sin escribir) y plataforma: proteccion de develop y main contra `plataforma::puertas::modelo()`, permisos del token y de la CI, checks requeridos frente a publicados y `capacidades` G01..G20 por plan observado (F1-13 decision 3, D1b dentro de esta tarea por no trocear). Salida de 5 lineas como mucho (primera con `fallos= avisos= no-comprobado= plataforma= plan= impuestas= sustituidas= no-disponibles= [no-comprobadas=] commit=`) y `--json` con todo; codigo 3 con alguna `falla`, 0 si no. Sin red, sin credenciales o sin plataforma, lo de plataforma sale `no-comprobado`; una garantia cuya configuracion no se pudo leer nunca sale `impuesta`. La tabla compilada se compara por test con la del ADR F1-13. `conectar_produccion` se movio a `identidad.rs`; el puerto `Plataforma` gana `proveedor()`, `leer_plan()` y `leer_puertas_de()` (una lectura por recurso). Adendas en F1-13 y F1-4. `cargo xtask verificar --perfil rapida` en verde.

No verificado: GitHub y GitLab reales (todo contra el mock; varios campos solo constan en esquemas o ejemplos de la documentacion, citada con fecha en la adenda); `PERMISO_CHECKS = checks:read` no contrastado en la ronda 2; `doctor --recursos` queda para TASK-080 (D2a).

Revision ronda 1: los dos pidieron cambios. rust-wasm-reviewer: H1 IMPORTANTE (sin leer las puertas, G01/G02/G03/G06/G09 salian `impuesta`), H2 IMPORTANTE (la configuracion de G01, G03 y G09 sin test que la detectara), H3-H5 MENORES (errores tragados en checks, 403 de `approval_rules` leido como config, `null` que rompia la lectura). code-quality-reviewer: R1-01 IMPORTANTE (la tabla G01..G20 sin red de regresion; tres sustitutos mutados sin rojo), R1-02..R1-06 MENORES (G04/G05 sin leer y adenda imprecisa, ref de estado duplicada, check ajeno al modelo como `falla`, sin red incompleto, lecturas repetidas). Los once corregidos en 82d7239, af26182, 0158805 y f5b75c0. Ronda 2: aprobada por los dos; correcciones verificadas por mutacion.

Test intermitente observado: `capacidades_por_plan_en_las_dos_plataformas` fallo una vez con `plataforma=sin-red` en paralelo bajo carga y paso al repetir; no reproducido en tres ejecuciones completas posteriores. Causa compatible con la de TASK-115 (paradas de red del antivirus en Windows; `serie()` solo excluye dentro del mismo binario).

MENORES aceptados (sin seguimiento): `lee_config` y el `match` de `config` listan las garantias por separado (fijado por test); redaccion «cuatro parametros» frente a «cinco booleanos y el contador»; con `null` en `approvals_required` o `code_owner_approval_required` sale `config` en vez de `no-comprobado` (conservador); `leer_puertas_de` por defecto no anula `checks_publicados` fuera de `checks_de` (inocuo); Enterprise de GitHub sale como Team; la construccion del adaptador repetida en `init/inventario.rs` sigue como deuda.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T13:23:19Z | plan | automatico | persona | 0.16.1 |
| 2026-11-17T13:29:49Z | approve | automatico | persona | 0.16.1 |
| 2026-11-17T13:29:50Z | start | automatico | persona | 0.16.1 |
| 2026-11-17T14:36:45Z | review | automatico | persona | 0.16.1 |
| 2026-11-17T15:31:57Z | review | automatico | persona | 0.16.1 |
| 2026-11-17T16:03:49Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-17T13:29:48Z | diseno | agente:a1395d1a890a33e28 | 48 | 99388 | 1485737 | 18733 | 40 | 1603906 | - | - |
| 2026-11-17T14:36:42Z | implementacion | agente:ae9f386d5583740b3 | 234 | 436140 | 39943070 | 144068 | 52 | 40523512 | - | - |
| 2026-11-17T14:36:45Z | implementacion | sesion:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 30 | 16531 | 2819010 | 4920 | 0 | 2840491 | - | - |
| 2026-11-17T14:51:58Z | revision | agente:a58913829876db5d1 | 82 | 180872 | 5373427 | 24098 | 10 | 5578479 | 1 | rust-wasm-reviewer |
| 2026-11-17T14:51:59Z | revision | agente:a4f5036071c9f8a1b | 52 | 171630 | 3283368 | 20811 | 11 | 3475861 | 1 | code-quality-reviewer |
| 2026-11-17T15:31:56Z | implementacion | agente:a37c8c9f7b9350566 | 258 | 238746 | 26360780 | 80580 | 54 | 26680364 | - | - |
| 2026-11-17T15:52:26Z | revision | agente:a9d6cbb705d540520 | 50 | 125464 | 2534782 | 11472 | 4 | 2671768 | 2 | code-quality-reviewer |
| 2026-11-17T15:53:50Z | revision | agente:a8e658e6ac6eaf7eb | 54 | 124863 | 2788986 | 11000 | 2 | 2924903 | 2 | rust-wasm-reviewer |

## Duracion

- Estimada: 450 min
- Real (de la rama abierta al cierre con merge, sin pausas): 154 min
- Paso de la hora: 154 min reales frente a 450 estimados
- Cierre (finish): 581 s, de 2026-11-17T15:54:08Z a 2026-11-17T16:03:49Z
