---
id: TASK-013
titulo: "F1-12 ADR actualizacion de proyectos entre versiones"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-013-f1-12-adr-actualizacion-de-proyectos-ent
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1028522
tokens_implementacion: 50980816
tokens_revision: 14824757
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-002, TASK-010]
---
## Objetivo

ADR: actualización de proyectos entre versiones (RR-13)

Tarea F1-12 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-1, F1-9.

## Criterios de aceptacion
- [ ] Qué migra, simulación, vuelta atrás, convivencia 0.16/2.0
- [ ] Cumple lo que pide la tarea F1-12 del plan y los requisitos que cita

## Decisiones de la persona

- 2026-11-14, Carlos, en la conversacion de la tarea tras ver la propuesta D1a: "Aprobamos los nombres". Quedan aprobados `tc2` (CLI), `taskcodetwo` (plugin, prefijo de skills, `.git/taskcodetwo/`, rama `taskcodetwo/tareas`) y el paquete npm `taskcodetwo` con binario `tc2`.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T13:24:05Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T13:30:16Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T13:30:19Z | start | automatico | persona | 0.16.1 |
| 2026-11-14T15:07:32Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T15:57:40Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T16:07:58Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T13:30:13Z | diseno | agente:ae62d302be5a9fd09 | 42 | 81404 | 929104 | 17972 | 31 | 1028522 | - | - |
| 2026-11-14T15:07:32Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 12 | 69560 | 1732286 | 3615 | 1 | 1805473 | - | - |
| 2026-11-14T15:07:32Z | implementacion | agente:a16bae7be7f2e33f9 | 274 | 391337 | 35860826 | 148431 | 40 | 36400868 | - | - |
| 2026-11-14T16:15:02Z | revision | agente:a893d9895c90d0cf7 | 124 | 181541 | 8942964 | 29581 | 8 | 9154210 | 1 | - |
| 2026-11-14T16:15:03Z | revision | agente:ac1aea671b7f56f28 | 42 | 199186 | 2770103 | 57222 | 26 | 3026553 | 2 | - |
| 2026-11-14T16:15:04Z | revision | agente:a77af8304a3254130 | 42 | 119807 | 2484825 | 39320 | 10 | 2643994 | 3 | - |
| 2026-11-14T16:15:05Z | implementacion | agente:dif:a16bae7be7f2e33f9 | 62 | 442641 | 12309678 | 22094 | 11 | 12774475 | - | - |

## Autorrevision

- code-quality-reviewer — 7f431ff
- code-quality-reviewer — 3c44c18
- code-quality-reviewer — d4799bc

## Rondas extra

- Ronda 3: Carlos (consigna «opcion recomendada» y fidelidad sobre coste); la ronda 2 encontro un IMPORTANTE introducido por la correccion (IMP-2), ya corregido en d4799bc.

## Verificacion ligera

- 2026-11-14T16:14:38Z: ronda 3; commit revisado 3b62dcff555fdf7c29e9f5763c0a4d8693e905be; commit verificado 1f1e433977d5c11db4c0ab67107f0f8e7fbcd1a7; ficheros: docs/adr/F1-12-actualizacion-entre-versiones.md; corregidos: MEN-10; pendientes: MEN-9

## Resultado

- **Implementado**: ADR [F1-12](../../../docs/adr/F1-12-actualizacion-entre-versiones.md) (aceptada) y su fila en `docs/adr/README.md`. `actualizar` como `planificar(proyecto) -> PlanMigracion` puro, `--simular` sin cambiar bytes, transaccion local con `subtree split` de `tareas/` a la rama `taskcodetwo/tareas`, aplanado en un commit solo de renombres, formato 2 de F1-9, criterio de salida estructural con vuelta atras automatica (codigo 6) y push solo con el criterio en verde. Convivencia 0.16/2.0 con nombres propios y bloqueo de escritura de la 0.16 mediante `---` en la primera linea de la config (fallo cerrado medido, D2 opcion a: no hace falta una 0.16.2). Retirada de los parches de la 0.16 calculada desde la clasificacion de F1-11.
- **Medido sobre una copia de OpenDockTrading (8141a08)**: 96 tareas (50/0/0/0/44/2), 1151 commits en el split, 96/96 en formato 2 con 0 diferencias, `--follow` 96/96 igual o mejor, `merge-base` vacio, segunda pasada 0 y 0, vuelta atras exacta; sin la capacidad `tareas-json` se niega con codigo 5 (el lector propio veria 0 de 94); retirada de parches 91 / 973 lineas (15 solas, 76 con sustitucion). Convivencia: con `---` la 0.16 no escribe (`new` y `roadmap --escribir` salen con 1, guardian de merges 2); sin el, crea un TASK-001 duplicado y deja el ROADMAP en 0 filas.
- **Decisiones de la persona**: D1a..D4a (consigna: opcion recomendada), sin troceo, orden de migracion OpenDockTrading primero, retirada de parches al migrar, nombres propios y definitivos para la 2.0. **Nombres aprobados por Carlos el 2026-11-14** (ver `## Decisiones de la persona`): `tc2` (CLI) y `taskcodetwo` (plugin, prefijo de skills, `.git/taskcodetwo/`, rama `taskcodetwo/tareas`, paquete npm). Ronda extra 3 (ver `## Rondas extra`).
- **Divergencias** (en el ADR): OpenDockTrading tiene 50/44/2 tareas por carpeta y no 33/61/2; `--follow` ya roto en develop en TASK-019, TASK-020 y TASK-080 (el criterio usa dos medidas); un hueco que lee la forma de carpetas es bloqueante; la retirada distingue parches que se quitan solos y con sustitucion; se borran los `.gitkeep`; lo que hace abortar a la 0.16 es `---` o una lista en bloque, no los bloques anidados; paquete npm `taskcodetwo` porque `tc2` esta ocupado; ToolsaWeb tiene 404 tareas y F9-2 dice 403; el tiempo de reloj del split no entra en la tabla comprobada.
- **Al cerrar (con taskctl)**: dependencias de D3a (F9-7 sin F9-6, F4-5 depende de F9-7, F9-8 depende de F4-5) y criterio de F9-2 corregido a 404 tareas. Antes de F9-7, los lectores propios de OpenDockTrading tienen que aceptar la carpeta fija.
- **No verificado**: choque de `tc` con iproute2 (sin distro WSL con iproute2), `--deshacer` despues del push, que `verificar.bat` de OpenDockTrading funcione tras quitar sus 15 lineas (la PoC no aplica la retirada, solo calcula su alcance), GitHub real, macOS y Linux; npm, Homebrew y winget dependen de la red.
- **Revision**: ronda 1 cambios (IMP-1 la PoC fallaba al reejecutarla por el tiempo de reloj en la tabla; MEN-1..6), todo corregido en 3c44c18; ronda 2 cambios (IMP-2 introducido: aprobacion de nombres sin evidencia en el repo; MEN-7, MEN-8), corregido en d4799bc; ronda extra 3 aprobada; MEN-9 y MEN-10 corregidos en 1f1e433 (`verificar-menores` no reconoce MEN-9 por vivir en `poc/`, ver `revision/menores-pendientes.md`).
- Implementacion por un agente implementador; revision por tres agentes independientes. `taskctl doctor` con 0 errores. OpenDockTrading y ToolsaWeb solo leidos. Temporales borrados.
