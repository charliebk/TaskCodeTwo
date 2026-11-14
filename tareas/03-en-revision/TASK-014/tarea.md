---
id: TASK-014
titulo: "F1-14 ADR distribucion e instalacion"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-014-f1-14-adr-distribucion-e-instalacion
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 941903
tokens_implementacion: 55522368
tokens_revision: 9092608
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-001]
---
## Objetivo

ADR: distribución e instalación (RR-16): ejecutable por sistema (empaquetador, tamaño, arranque), firma de código en Windows y macOS, instaladores de una línea, canales (releases, winget, Homebrew, npm, marketplace), compatibilidad entre ejecutable y plugin

Tarea F1-14 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-0.

## Criterios de aceptacion
- [ ] Prueba de concepto de un ejecutable en los tres sistemas
- [ ] decisión sobre firma y canales
- [ ] Cumple lo que pide la tarea F1-14 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T17:40:08Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T17:49:18Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T17:50:22Z | start | automatico | persona | 0.16.1 |
| 2026-11-14T18:46:57Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T19:15:49Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T17:49:17Z | diseno | agente:a4df657c5212133fd | 38 | 66186 | 850363 | 25316 | 46 | 941903 | - | - |
| 2026-11-14T18:46:57Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 32 | 25868 | 3273309 | 11250 | 1 | 3310459 | - | - |
| 2026-11-14T18:46:57Z | implementacion | agente:a0eca4c8e372e949e | 64 | 120384 | 2904015 | 27722 | 4 | 3052185 | - | - |
| 2026-11-14T18:46:57Z | implementacion | agente:a16159101b5cab76d | 216 | 312242 | 25648201 | 119097 | 21 | 26079756 | - | - |
| 2026-11-14T18:46:57Z | implementacion | agente:af6a6cc9afb80a9c8 | 52 | 103191 | 2233495 | 19993 | 1 | 2356731 | - | - |
| 2026-11-14T19:36:03Z | revision | agente:aa70fada045e639ee | 66 | 223301 | 5240364 | 29494 | 13 | 5493225 | 1 | - |
| 2026-11-14T19:36:03Z | revision | agente:a9e8d9d301c09fc46 | 58 | 155517 | 3424631 | 19177 | 9 | 3599383 | 2 | - |
| 2026-11-14T19:36:06Z | implementacion | agente:dif:a16159101b5cab76d | 108 | 838002 | 19839564 | 45563 | 17 | 20723237 | - | - |

## Autorrevision

- code-quality-reviewer — 10027fd
- code-quality-reviewer — 4456a70

## Verificacion ligera

- 2026-11-14T19:36:02Z: ronda 2; commit revisado ef826d85f6c32ba273cb3ffd54e9f1da27506f09; commit verificado 463f9fc8a2a66f20d0cfa0a9ff0f5d972e6477de; ficheros: docs/adr/F1-14-distribucion-e-instalacion.md; corregidos: MEN-6, MEN-8, MEN-9, MEN-10; pendientes: MEN-7

## Decisiones de la persona

- 2026-11-14, Carlos: el lenguaje del CLI se decide con una comparacion medida Rust frente a TypeScript ("Si", a la propuesta de meterlo en TASK-014).
- 2026-11-14, Carlos: "Haz la comparacion con lo que ya esta instalado": sin descargar bun, zig, targets de rustup ni imagenes Docker.

## Resultado

- **Implementado**: ADR [F1-14](../../../docs/adr/F1-14-distribucion-e-instalacion.md) (aceptada) y su fila en `docs/adr/README.md`. PoC del MISMO comando (especificacion unica) en Rust y en TypeScript empaquetado con Node SEA, escritos por dos agentes distintos, con salida identica byte a byte y mismo codigo de salida en 11 casos limite. Decide: **Rust** para el CLI `tc2` (propuesto como divergencia en F2-1, con `/Brepro`), lanzador sin runtime + `cli.lock` con sha256 por target y descarga comprobada (D3a), `tc2 version --json` con protocolo, canales y firma por fases (D4a, D5a), TLS con el almacen del sistema y contrato de `tc2 doctor --recursos` (obligatorio, necesario, conveniente con engram no obligatorio, y lo que estorba; D6a, D7a).
- **Medido (Windows x64)**: recuentos 404/0 (ToolsaWeb 8bebfd049) y 96/0 (OpenDockTrading 8141a08a7) iguales a PyYAML; los dos rechazan una clave duplicada que `safe_load` acepta; ejecutable 2 720 768 B frente a 87 430 656 B (32,1 veces); Rust mas rapido en caliente y en frio en los 3 repos (de reloj, sin comprobar en la tabla); los dos funcionan sin node en el PATH y firmados con un certificado desechable (sin tocar el almacen); seis targets en el repo del marketplace ~15,6 MiB por version con Rust y ~500,3 MiB con TypeScript (estimado). Regla D1a: Rust gana el 1.o, el 2.o y el 4.o (el 4.o es juicio: postject en alfa desde 2023; sin el, la regla no decide y Rust se sostiene por el margen). Coste del lanzador medido: no se reverifica la cache en cada llamada; el sha256 lo calcula el lanzador o un proceso distinto al descargar, en `doctor --recursos` y al actualizar.
- **Decisiones de la persona**: ver `## Decisiones de la persona`; D1a..D9a (consigna: opcion recomendada); sin troceo.
- **Divergencias** (en el ADR): F2-1 de TypeScript a Rust; P12 (el plugin trae lanzador y `cli.lock`, no el binario); P12 y F2-11 (hash y attestation en la 2.0, Authenticode y Developer ID al entrar en winget y Homebrew); F2-10 recibe la ejecucion en Linux y macOS; F2-11/F2-12 con `tc2`; F3-8 con `doctor --recursos`; criterio «tres sistemas» de F1-14 solo cumplido en Windows; bun no medido (solo Node SEA); activo nuevo para F1-8 (binario en cache y `cli.lock`).
- **No verificado**: Linux, macOS y Windows ARM64 (sin targets ni imagenes, por decision de Carlos); compilacion cruzada reproducible; firma real (SmartScreen, sello de tiempo, Gatekeeper); importe de Azure Artifact Signing (solo para personas de EE. UU. y Canada) y de un certificado OV; MOTW y antivirus; attestation y provenance de npm; scope `@taskcodetwo`; tokens por implementacion; `useCodeCache`/`useSnapshot` y `node --build-sea`; lanzador `.cmd` con certutil/PowerShell; el lanzador, `cli.lock` y `doctor --recursos` no existen aun.
- **Revision**: ronda 1 cambios (IMP-1 el guion fijaba la ruta `02-en-curso`; MEN-1..5), corregido en 4456a70; ronda 2 aprobada con correcciones; MEN-6..10 (contradicciones de la cache, MiB por contexto, cifras de reloj marcadas, activo para F1-8, quien calcula el hash) corregidos en 463f9fc (`verificar-menores` no reconoce MEN-7 por vivir en `poc/`, ver `revision/menores-pendientes.md`).
- Tres agentes implementadores (Rust, TypeScript, medicion y ADR; el de TypeScript se paro para que no se pisara con el medidor) y dos revisores independientes. `taskctl doctor` con 0 errores. Pilotos solo leidos. Temporales borrados.

