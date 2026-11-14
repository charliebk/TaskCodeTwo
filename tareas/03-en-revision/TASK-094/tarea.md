---
id: TASK-094
titulo: "F1-18 ADR hito de decisiones (ADR) en init y en actualizar"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-094-f1-18-adr-hito-de-decisiones-adr-en-init
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1163455
tokens_implementacion: 23629272
tokens_revision: 5676796
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-012, TASK-013]
---
## Objetivo

ADR: el «Hito de decisiones (ADR)» como paso del producto en todo proyecto (no «Hito 0»): en un proyecto nuevo (`init`), investigar el contexto y escribir los ADR antes de la primera tarea de codigo; en un proyecto existente o con la 0.16 (`actualizar`), un estudio de adopcion que inventaria lo que se hace a mano, mide el uso real (tareas, rondas, tokens, verificacion), contrasta con la 2.0 y deja ADR y tareas para cada hueco. Obligatorio pero proporcional (su tamano sale del propio estudio), y `start` de una tarea de codigo lo exige cerrado o declarado no necesario por una persona, con un comando determinista. Decidido con Carlos el 2026-11-14 durante el H1; precedentes medidos: el H1 de TaskCodeTwo (ADR F1-1..F1-11) y el inventario de F1-11 sobre ToolsaWeb y OpenDockTrading.

## Criterios de aceptacion
- [ ] [ ] Contrato del hito en `init` y en `actualizar` (entradas, salidas, cuando se considera cerrado) con un ejemplo medido sobre OpenDockTrading
- [ ] [ ] Regla de proporcionalidad con cifras del H1 (tokens y tiempo por ADR) y comando que lo exige antes de `start` en tareas de codigo
- [ ] [ ] Cumple RR-13 y lo que deciden F1-11 (inventario parche/propio) y F1-12 (actualizacion)

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T16:16:49Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T16:22:46Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T16:22:54Z | start | automatico | persona | 0.16.1 |
| 2026-11-14T16:59:01Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T17:31:05Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T16:22:45Z | diseno | agente:af45bd1a014dc7307 | 34 | 97122 | 1048253 | 18046 | 23 | 1163455 | - | - |
| 2026-11-14T16:59:01Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 32 | 11430 | 2997940 | 6230 | 5 | 3015632 | - | - |
| 2026-11-14T16:59:01Z | implementacion | agente:a46369616f023d3a8 | 96 | 261116 | 8904177 | 77815 | 18 | 9243204 | - | - |
| 2026-11-14T17:39:16Z | revision | agente:a7f13714f33c41b42 | 40 | 147442 | 2324194 | 16274 | 5 | 2487950 | 1 | - |
| 2026-11-14T17:39:16Z | revision | agente:a218255228f01bb59 | 52 | 149167 | 3018549 | 21078 | 16 | 3188846 | 2 | - |
| 2026-11-14T17:39:17Z | implementacion | agente:dif:a46369616f023d3a8 | 74 | 346785 | 10973253 | 50324 | 10 | 11370436 | - | - |

## Autorrevision

- code-quality-reviewer — 85195c9
- code-quality-reviewer — 283d19c

## Verificacion ligera

- 2026-11-14T17:39:14Z: ronda 2; commit revisado bdc5b032d60f7aa0697c778e8398d62e2c9401a6; commit verificado 639103b65760934d484ccd4694c227650fab9502; ficheros: docs/adr/F1-18-hito-de-decisiones.md; corregidos: MEN-5, MEN-6, MEN-7; pendientes: ninguno

## Resultado

- **Implementado**: ADR [F1-18](../../../docs/adr/F1-18-hito-de-decisiones.md) (aceptada) y su fila en `docs/adr/README.md`. El «Hito de decisiones (ADR)» como paso del producto en `init` y en `actualizar`: el estudio de adopcion amplia el informe de `tc2 actualizar --simular` (F1-12) con el uso medido y `proponer_hito` saca un ADR o una tarea por hueco; el estado vive en `hito-decisiones.yml` de la rama `taskcodetwo/tareas` y se calcula (ausente / abierto / cerrado / no-necesario) desde la ultima version firmada por una persona; la puerta para en `tc2 start` con `rechazada` (3) y la impone el check `estado`; el hook es solo atajo opcional del mismo comando. Comandos `tc2 hito estado|proponer|abrir|descartar|no-necesario`, los que escriben solo por persona y partiendo de la version firmada.
- **Medido**: H1 de TaskCodeTwo (12 ADR, TASK-002..TASK-013): minutos «Real» suma 739, mediana 45,5, p75 50; tokens totales por ADR mediana 13942446,5 y p75 19081290 (salida 147315,5 / 204929). OpenDockTrading (copia en 8141a08): `## Coste` en 44/96 tareas y «Real» en 34/96; `proponer_hito` da 9 elementos con la 2.0 de hoy (3 bloqueantes, 4 tareas de `verificar.bat`, 2 ADR) y 5 con la 2.0.0 prevista; copia sin cambios. Puerta con el hito abierto: TaskCodeTwo 6 pasan / 75 se niegan, OpenDockTrading 0 / 50; con cerrado o no-necesario pasa todo. 22 casos de registro (5 ediciones validas del agente sin firma salen `abierto`) y el flujo de `actualizar` llega a `cerrado` y a `no-necesario` sin atascarse.
- **Decisiones de la persona**: nombre «Hito de decisiones (ADR)», obligatorio pero proporcional y exigido por comando determinista (Carlos, 2026-11-14); D1a..D5a (consigna: opcion recomendada); sin troceo.
- **Divergencias** (en el ADR): falta la fila F1-18 en el plan (propuesta con dependencias F1-11 y F1-12); `init` queda fuera de la puerta (el PR de TASK-001 saldria en rojo); registro roto o ausente niega (fallo cerrado); campo `hito: true` y `descartadas` nuevos para F2-5; choques del 3 y del 4 entre F1-4 y F1-12 para F2-3; verificacion por tarea «no medido» (la 0.16 no la guarda).
- **No verificado**: ningun comando de la 2.0 existe (son funciones de la PoC); GitHub real y la proteccion del registro; la identidad de persona (en la PoC, el correo del autor del commit, suplantable en local: garantia real en F1-6/F2-13); `init` en un proyecto nuevo real; escritura atomica (F1-2); ToolsaWeb; macOS y Linux; la parada local solo medida sin carril declarado.
- **Revision**: ronda 1 con IMP-1 (`cerrado` sin firma de persona), IMP-2 (codigo 4 atribuido y en choque con `sinred`), IMP-3 (nadie creaba el registro en `actualizar`) y MEN-1..4; el revisor escribio `aprobada-con-correcciones` con IMPORTANTE abiertos y se corrigio a `cambios-solicitados` (la skill no lo admite); todo corregido en 283d19c. Ronda 2 aprobada con correcciones: MEN-5..7 (blanqueo de ediciones al firmar, `abrir` sobre registro existente, choque del 3) corregidos en 3154cb6; PoC completa reejecutada con log identico.
- Implementacion por un agente implementador; revision por dos agentes independientes. `taskctl doctor` con 0 errores. OpenDockTrading solo leido. Temporales borrados.

