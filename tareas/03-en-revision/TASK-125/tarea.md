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
tokens_revision: 1224273
creado: 2026-11-18
actualizado: 2026-11-18
dependencias: []
---
## Objetivo

Quitar situacion::actividad, que quedo sin usos fuera de sus tests tras TASK-081 (la regla D2a vive en ciclo::liberar::actividad_conocida) y cuyo comentario dice que una fecha futura cuenta como de hoy.

## Criterios de aceptacion
- [x] situacion::actividad no existe o delega en ciclo::liberar::actividad_conocida, y no queda ningun comentario que diga que una fecha futura cuenta como de hoy
- [x] Los tests de situacion, tablero y liberar siguen en verde

## Resultado

Hecho: se borra `situacion::actividad`, que no tenia llamadores (la regla D2a vive en `ciclo::liberar::actividad_conocida` desde TASK-081). Borrar y no delegar (D3a): delegar habria creado el ciclo `situacion` <-> `ciclo::liberar`. El comentario de `dias_sin_actividad` ya no dice que una fecha futura cuenta como de hoy: dice que recibe la actividad filtrada por `actividad_conocida` y que solo dentro de `MARGEN_FUTURO` puede ir por delante (la resta satura). El test de situacion pasa a `dias_sin_actividad_completos`. El caso «sin fila, solo commit de rama» que cubria el test borrado pasa a una asercion de `actividad_conocida_sin_fechas_del_futuro` en `liberar.rs` (D2a). Los dos comentarios de `liberar.rs` ligados al margen se quedan (D1a). `cargo doc` con los mismos 10 avisos de antes; `cargo xtask verificar --perfil rapida` en verde.

Revision ronda 1: aprobada sin hallazgos (rust-wasm-reviewer; la asercion nueva se pone roja con un mutante).

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
| 2026-11-18T21:17:07Z | revision | agente:aa6df471886d8e807 | 26 | 100259 | 1118841 | 5147 | 1 | 1224273 | 1 | rust-wasm-reviewer |

## Autorrevision

- rust-wasm-reviewer — d52d640
- code-quality-reviewer — d52d640
