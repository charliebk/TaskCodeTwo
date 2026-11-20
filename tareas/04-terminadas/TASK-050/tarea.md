---
id: TASK-050
titulo: "F6-4 Busqueda de errores"
tipo: feature
sprint: 6
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-050-f6-4-busqueda-de-errores
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 820311
tokens_implementacion: 16675155
tokens_revision: 6002636
creado: 2026-11-14
actualizado: 2026-11-20
dependencias: [TASK-049]
---
## Objetivo

Búsqueda de errores: reproducción, mutantes y bordes con evidencia en cada hallazgo (RR-03)

Tarea F6-4 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F6-3.

## Criterios de aceptacion
- [x] Hallazgo sin evidencia = rechazado por el propio workflow
- [x] Cumple lo que pide la tarea F6-4 del plan y los requisitos que cita


## Resultado

Busqueda de errores con evidencia (F6-4, RR-03). `tc2 hallazgos validar` (modulo nuevo `crates/tc2/src/hallazgos/`, esquema `crates/tc2/esquemas/hallazgos.schema.json` v1) lee el unico bloque json de la respuesta de cada revisora y rechaza, con un motivo estable, todo hallazgo que:
- no traiga evidencia de su tipo (`reproduccion`, `mutante` o `borde`), tambien los MENOR;
- no apunte a una linea del lado nuevo del diff que vio esa revisora;
- tenga un mutante cuya `linea_original` no coincide, o un borde sin diferencia.

`tc2 hallazgos instrucciones` da el texto del prompt desde las mismas constantes.

`revision-ia.yml` pasa a cuatro jobs:
- `preparar`: sin secretos; deja los diffs y el informe;
- `consultar`: solo `ANTHROPIC_API_KEY`; respuestas en base64 por `outputs`, cortadas a 64 KiB conservando principio y final;
- `validar`: sin secretos; compila `tc2` y corre `tc2 hallazgos validar`;
- `publicar`: solo la App; publica el check `neutral` con aceptados, rechazados y motivos, primero las tablas y despues la prosa.

El lint exige cada secreto en su job. Guia y adenda F6-4 en F1-8: la ejecucion de la evidencia pasa a F6-6, por decision de Carlos.

Decisiones de Carlos (2026-11-20): sin trocear; la evidencia se exige a todos los hallazgos; la ejecucion, en F6-6.

Evidencia (2026-11-20):
- C1, rechazo sin evidencia, en `tc2`: `crates/tc2/tests/hallazgos.rs` con el binario real da `aceptados=3 rechazados=2` (`sin-evidencia`, `linea-fuera-del-diff`), y una respuesta en prosa sale toda rechazada con `sin-bloque`.
- C1 en el workflow, E2E real en `charliebk/taskcodetwo-e2e-github` (PR 16, run 44785038930):
  - los cuatro jobs en verde;
  - la clave del modelo y la App, en jobs distintos del mismo entorno `revisor`, funcionan con `secrets: inherit`;
  - el check de la App 5846307 lleva `aceptados=0 rechazados=1 motivos=respuesta:sin-bloque` por revisora (la cuenta de la API sigue sin saldo y el modelo devolvio 400) y cita el informe de verificacion.
  - Repo E2E limpio: PR cerrado, rama borrada y workflows retirados.
- C2: la salida `--json` valida contra el esquema, con un test que contrasta los motivos y claves del esquema y del codigo. Mutaciones en tc2 y en el lint dan rojo.
- Verificacion completa en verde sobre 90640c0. Las correcciones de la ronda 1 (cb7e38a) se comprobaron con clippy `-D warnings`, `cargo test -p tc2 --lib hallazgos` (8), `--test hallazgos` (3) y `cargo test -p xtask` (29), y con `taskctl verificar-menores` (57cba7b). `finish` vuelve a verificar.

Sin comprobar con el modelo de verdad: que respete el formato del bloque json, que de `linea_original` literal, y el tamano real de los `outputs` con respuestas largas. Queda para cuando haya saldo.

Revision por pares:
- Ronda 1: rust-wasm-reviewer y code-quality-reviewer, aprobada-con-correcciones, sin CRITICO ni IMPORTANTE. Los seis MENOR se corrigieron en cb7e38a y los reconocio `verificar-menores`.
  - Rust: MEN-1 `plano` dejaba pasar autoenlaces, `@`, `$`, emojis y caracteres de formato; MEN-2 rutas de git entrecomilladas sin decodificar; MEN-3 `trim` de `paso_rojo` sin test.
  - Calidad: MEN-1 el corte a 64 KiB perdia el bloque json del final; MEN-2 el recorte del check se comia tablas; MEN-3 el lint no comprobaba el nombre del artefacto.
- Nota: durante la implementacion, la verificacion rapida del freno de fin de turno se solapo con la completa y se corto por tiempo, sin ningun test en rojo. No fue un fallo del cambio.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-20T16:17:26Z | plan | automatico | persona | 0.16.1 |
| 2026-11-20T17:13:02Z | approve | automatico | persona | 0.16.1 |
| 2026-11-20T17:13:04Z | start | automatico | persona | 0.16.1 |
| 2026-11-20T18:19:32Z | review | automatico | persona | 0.16.1 |
| 2026-11-20T20:02:13Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-20T17:12:52Z | diseno | agente:aeb52ef7598c6f034 | 30 | 75295 | 725456 | 19530 | 28 | 820311 | - | - |
| 2026-11-20T18:19:32Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 28 | 16906 | 3861255 | 7627 | 2 | 3885816 | - | - |
| 2026-11-20T18:19:32Z | implementacion | agente:a9a3db164ed9e1311 | 124 | 283012 | 12386675 | 119528 | 30 | 12789339 | - | - |
| 2026-11-20T18:54:48Z | revision | agente:a3a7aaf5c14c0ca20 | 44 | 163861 | 2932594 | 24320 | 6 | 3120819 | 1 | rust-wasm-reviewer |
| 2026-11-20T18:54:48Z | revision | agente:abc521b1d5ae87c59 | 44 | 153977 | 2701613 | 26183 | 3 | 2881817 | 1 | code-quality-reviewer |

## Verificacion ligera

- 2026-11-20T19:01:31Z: ronda 1; commit revisado 90640c05dc759042baa565655060c3224b753c6f; commit verificado cb7e38a269ef34cfa5a246ea76fa838b9fb4fd01; ficheros: .github/workflows/revision-ia.yml, crates/tc2/src/hallazgos/mod.rs, xtask/src/workflows.rs; corregidos: MEN-1, MEN-2, MEN-3, MEN-1, MEN-2, MEN-3; pendientes: ninguno

## Duracion

- Estimada: 140 min
- Real (de la rama abierta al cierre con merge, sin pausas): 169 min
- Paso de la hora: 169 min reales frente a 140 estimados
- Cierre (finish): 2061 s, de 2026-11-20T19:27:52Z a 2026-11-20T20:02:13Z
