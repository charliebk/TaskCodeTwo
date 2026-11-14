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
tokens_implementacion: 12258836
tokens_revision: null
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

## Autorrevision

- code-quality-reviewer — 85195c9
- code-quality-reviewer — 283d19c
