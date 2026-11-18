---
id: TASK-125
titulo: "Seguimiento de F3-10: quitar la copia vieja de la regla de actividad"
tipo: fix
sprint: 0
etiquetas: [tablero]
complejidad: simple
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: fix/task-125-seguimiento-de-f3-10-quitar-la-copia-vie
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 160923
tokens_implementacion: 3392978
tokens_revision: null
creado: 2026-11-18
actualizado: 2026-11-18
dependencias: []
---
## Objetivo

Quitar situacion::actividad, que quedo sin usos fuera de sus tests tras TASK-081 (la regla D2a vive en ciclo::liberar::actividad_conocida) y cuyo comentario dice que una fecha futura cuenta como de hoy.

## Criterios de aceptacion
- [ ] situacion::actividad no existe o delega en ciclo::liberar::actividad_conocida, y no queda ningun comentario que diga que una fecha futura cuenta como de hoy
- [ ] Los tests de situacion, tablero y liberar siguen en verde

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-18T20:40:58Z | plan | automatico | persona | 0.16.1 |
| 2026-11-18T20:42:59Z | approve | automatico | persona | 0.16.1 |
| 2026-11-18T20:43:01Z | start | automatico | persona | 0.16.1 |
| 2026-11-18T21:11:12Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-18T20:42:58Z | diseno | agente:a34ddf615abf1fb2e | 14 | 29956 | 124657 | 6296 | 9 | 160923 | - | - |
| 2026-11-18T21:11:12Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 16 | 9984 | 3378548 | 4430 | 2 | 3392978 | - | - |

## Autorrevision

- rust-wasm-reviewer — d52d640
- code-quality-reviewer — d52d640
