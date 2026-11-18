---
id: TASK-127
titulo: "Escrituras de estado con muchos ficheros fallan en Windows (git hash-object)"
tipo: fix
sprint: 0
etiquetas: [estado]
complejidad: simple
modelo_sugerido: sonnet
estado: en-curso
plan_aprobado: true
rama: fix/task-127-escrituras-de-estado-con-muchos-ficheros
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 408633
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-18
actualizado: 2026-11-18
dependencias: []
---
## Objetivo

Una operacion de estado que escribe muchos ficheros a la vez (p. ej. 1000 en un import grande) falla con git=hash-object en Windows, probablemente por la longitud de la linea de ordenes (visto en TASK-084).

## Criterios de aceptacion
- [ ] Una transaccion que escribe 1000 ficheros en una sola operacion termina en ok en Windows (test)
- [ ] El numero de procesos de git no crece con el numero de ficheros mas alla de un tope fijo (hash-object por stdin o por tandas)

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-18T13:17:32Z | plan | automatico | persona | 0.16.1 |
| 2026-11-18T13:23:44Z | approve | automatico | persona | 0.16.1 |
| 2026-11-18T13:23:58Z | start | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-18T13:23:28Z | diseno | agente:aab8c8c2056b15be3 | 24 | 44265 | 356739 | 7605 | 18 | 408633 | - | - |

## Autorrevision

- rust-wasm-reviewer — 22ffc96
- code-quality-reviewer — 22ffc96
