---
id: TASK-044
titulo: "F5-4 Informe de verificacion en JSON para el revisor IA"
tipo: feature
sprint: 5
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-044-f5-4-informe-de-verificacion-en-json-par
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 906460
tokens_implementacion: 8030498
tokens_revision: 5379695
creado: 2026-11-14
actualizado: 2026-11-19
dependencias: [TASK-041]
---
## Objetivo

Informe de verificación en JSON para el revisor IA

Tarea F5-4 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F5-1.

## Criterios de aceptacion
- [x] El revisor lo cita y no repite tests
- [x] Cumple lo que pide la tarea F5-4 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-19T17:30:57Z | plan | automatico | persona | 0.16.1 |
| 2026-11-19T17:37:24Z | approve | automatico | persona | 0.16.1 |
| 2026-11-19T17:37:26Z | start | automatico | persona | 0.16.1 |
| 2026-11-19T18:26:58Z | review | automatico | persona | 0.16.1 |
| 2026-11-19T19:19:31Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-19T17:37:24Z | diseno | agente:a6a0f3668d16a3ed8 | 38 | 70079 | 818575 | 17768 | 33 | 906460 | - | - |
| 2026-11-19T18:26:58Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 16 | 12031 | 2849797 | 3565 | 0 | 2865409 | - | - |
| 2026-11-19T18:26:58Z | implementacion | agente:ada778475cbcc980c | 78 | 156620 | 4972774 | 35617 | 23 | 5165089 | - | - |
| 2026-11-19T18:55:52Z | revision | agente:a3a9ec27d689838e8 | 16 | 119870 | 730713 | 11383 | 1 | 861982 | 1 | rust-wasm-reviewer |
| 2026-11-19T18:55:52Z | revision | agente:ac0a09bc0d1079a32 | 74 | 144215 | 4358082 | 15342 | 4 | 4517713 | 1 | code-quality-reviewer |

## Resultado

Informe de verificacion en JSON (F5-4): esquema v1 `crates/tc2/esquemas/informe-verificacion.schema.json` (`perfil`, `resultado`, `commit`, `inicio`, `fin`, `pasos[]`, `tests` opcional; sin claves extra); lector `tc2::verificacion::informe::leer` (serde con `deny_unknown_fields` y validaciones de forma) y `seccion_revisor`, el texto con el que el revisor cita el informe: artefacto, perfil, resultado, commit, tabla de pasos y la regla (manda el check `verificacion` de `github-actions`, app 15368; no se relanzan los pasos en verde; el JSON nunca decide). `cargo xtask verificar` escribe el informe en `TC2_INFORME`, tambien en rojo. El workflow sube el artefacto `informe-verificacion-<perfil>` desde `ejecutar` (`actions/upload-artifact` v4.6.2 por SHA, `if: always()`) y el publicador anade `informe=` al resumen; la plantilla de GitLab lo guarda como artefacto `when: always`. El lint admite `upload-artifact` solo por SHA y en clase ejecuta. Guia: seccion "Informe JSON (F5-4)" y divergencias 7-10 en `docs/ci/verificacion-reutilizable.md`.

Evidencia (2026-11-19):
- Verificacion completa en verde sobre cf29ab0; `finish` vuelve a verificar.
- Tests: el informe de xtask (verde y rojo) es byte a byte el fixture que lee tc2; `seccion_revisor` cita artefacto, perfil, pasos y regla; 11 mutaciones del contrato dan error; el lint exige la subida en `ejecutar` y la rechaza con escritura o sin SHA. Los fixtures pasan en un clon de Windows con autocrlf.
- API `ci/lint` de GitLab con la plantilla en linea: valida, con `artifacts` y `TC2_INFORME` en los cuatro jobs.
- El revisor de calidad ejercito el comando real por el camino en rojo: escribe el informe y el JSON es valido.

C1: "lo cita" queda demostrado con `seccion_revisor`; "no repite tests" solo se comprueba con el revisor de F6-3: divergencia explicita para TASK-049, que debe usar `seccion_revisor`. Sin push en esta tarea (D6): la ejecucion real en GitHub (artefacto presente, `informe=` en el resumen, descarga con `actions: read`) se hace en TASK-049.

Revision por pares, ronda 1: las dos aprobadas. MENORES aceptados como seguimiento para TASK-049: tope de tamano y de pasos al leer (un JSON de 96 MB se lee); en `pull_request` el `commit` del informe es el del merge, no `head.sha` (no exigir igualdad o pasar el head real); `tests: null` y las reglas descriptivas del esquema no casan del todo con el lector.

## Duracion

- Estimada: 80 min
- Real (de la rama abierta al cierre con merge, sin pausas): 102 min
- Paso de la hora: 102 min reales frente a 80 estimados
- Cierre (finish): 1384 s, de 2026-11-19T18:56:27Z a 2026-11-19T19:19:31Z
