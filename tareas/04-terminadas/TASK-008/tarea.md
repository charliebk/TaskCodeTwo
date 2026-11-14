---
id: TASK-008
titulo: "F1-7 ADR verificacion en CI, perfiles, cache por SHA y runners"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-008-f1-7-adr-verificacion-en-ci-perfiles-cac
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1298087
tokens_implementacion: 7943285
tokens_revision: 3769334
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-006]
---
## Objetivo

ADR: verificación en CI, perfiles, caché por SHA y runners (GitHub y self-hosted Windows) (P6)

Tarea F1-7 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-5.

## Criterios de aceptacion
- [x] Coste estimado con las suites de ToolsaWeb y OpenDockTrading
- [x] Cumple lo que pide la tarea F1-7 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T07:06:31Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T07:12:24Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T07:12:25Z | start | automatico | persona | 0.16.1 |
| 2026-11-14T07:27:02Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T07:39:14Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T07:43:04Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T07:12:23Z | diseno | agente:af104e72ebb7fb2bf | 42 | 96950 | 1182579 | 18516 | 30 | 1298087 | - | - |
| 2026-11-14T07:27:02Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 8 | 9650 | 1231125 | 2147 | 0 | 1242930 | - | - |
| 2026-11-14T07:27:02Z | implementacion | agente:a3bb03f6be501e154 | 86 | 221121 | 6412845 | 66303 | 15 | 6700355 | - | - |
| 2026-11-14T07:31:46Z | revision | agente:aa8f7011cd12b481e | 38 | 156583 | 2215961 | 27022 | 1 | 2399604 | 1 | - |
| 2026-11-14T07:42:56Z | revision | agente:a1f05917f3be444d4 | 20 | 166329 | 1174397 | 28984 | 7 | 1369730 | 2 | - |

## Autorrevision

- code-quality-reviewer — 1a083e6
- code-quality-reviewer — fee7038

## Resultado

- **Implementado**: ADR [F1-7](../../../docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md) (aceptada) y su fila en `docs/adr/README.md`. Un solo comando del proyecto con `--perfil rapida|impacto|completa|pesada` (sustituye a los seis `comando_*`); clave de cache = perfil, runner declarado, huella del arbol y version del workflow, mas una huella local calculada en el runner del perfil (sin ella no hay acierto); el resultado vive en el check `verificacion` de Actions y se reutiliza buscando a dos niveles (padres y abuelos) un verde de la App 15368 con el mismo arbol del merge ref y la misma clave; cada perfil se busca con su clave (en release la completa se reutiliza y la pesada se ejecuta si no hay acierto); runner por perfil, self-hosted caido falla cerrado; tabla de eventos con `merge_group` y `push`. Tabla de coste reproducible por el guion.
- **Coste mensual estimado** (precios comprobados en docs.github.com el 2026-11-14: Windows 0,010 $/min, Linux 0,006 $/min, self-hosted gratis, 12 $ incluidos en Free): todo en runners de GitHub 175,69 $/mes (ToolsaWeb 133,63 $, OpenDockTrading 45,15 $, jobs `buscar` en Linux); con la completa de ToolsaWeb en la maquina de Carlos 42,06 $/mes. Supuestos sin verificar: factor de velocidad del runner 1,0, 0,5 actualizaciones de rama por tarea, 1 ejecucion por ronda.
- **Decisiones de la persona** (consigna: opcion recomendada): D1a completa en PR a `develop`, D2a resultado en el check, D3a falla cerrado, D4a pesada dentro de `verificacion` en release, D5a, D6a no ejecutar las suites.
- **Divergencias** (en el ADR): busqueda a dos niveles; `arbol=` es el del merge ref; la cache no abarata `strict` (contradice la decision 7 de F1-5); job `buscar` aparte en Linux; pesada solo en release; `push` a `develop`/`main` no ejecuta `verificacion`; tasa self-hosted no documentada; correcciones de la ronda 1.
- **No verificado**: comportamiento de Actions (check en el head del PR, nombre con workflow reutilizable, `output.summary`, arbol de `merge_group`, espera del runner self-hosted), releases al mes, duracion de la pesada sola, GitLab, que la completa no varie con `RAMA_BASE` (nota de la ronda 2: falta un caso «mismo arbol, distinta base» en la PoC; para F5-2).
- **Revision**: ronda 1 cambios (IMP-1 release verde sin pesada, IMP-2 huella calculada fuera del runner; MEN-1..3), todo corregido; ronda 2 aprobada sin hallazgos nuevos.
- Implementacion por un agente implementador; revision por dos agentes independientes. ToolsaWeb y OpenDockTrading solo leidos. `taskctl doctor` con 0 errores. Temporales borrados.

## Duracion

- Estimada: 60 min
- Real (de la rama abierta al cierre con merge, sin pausas): 31 min
- Cierre (finish): 7 s, de 2026-11-14T07:42:57Z a 2026-11-14T07:43:04Z
