---
id: TASK-127
titulo: "Escrituras de estado con muchos ficheros fallan en Windows (git hash-object)"
tipo: fix
sprint: 0
etiquetas: [estado]
complejidad: simple
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: fix/task-127-escrituras-de-estado-con-muchos-ficheros
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 408633
tokens_implementacion: 5806770
tokens_revision: 13287688
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
| 2026-11-18T14:10:54Z | review | automatico | persona | 0.16.1 |
| 2026-11-18T15:12:21Z | review | automatico | persona | 0.16.1 |
| 2026-11-18T16:02:28Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-18T13:23:28Z | diseno | agente:aab8c8c2056b15be3 | 24 | 44265 | 356739 | 7605 | 18 | 408633 | - | - |
| 2026-11-18T14:10:54Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 70 | 56201 | 5734068 | 16431 | 5 | 5806770 | - | - |
| 2026-11-18T14:21:31Z | revision | agente:a324a0b1d78124359 | 20 | 103964 | 860702 | 5539 | 2 | 970225 | 1 | rust-wasm-reviewer |
| 2026-11-18T14:21:31Z | revision | agente:a4b729769f42ff4bd | 38 | 131669 | 1967278 | 16436 | 4 | 2115421 | 1 | code-quality-reviewer |
| 2026-11-18T15:25:46Z | revision | agente:a372d86c1374fda45 | 90 | 156613 | 5380027 | 30622 | 11 | 5567352 | 2 | code-quality-reviewer |
| 2026-11-18T15:25:46Z | revision | agente:af057479711085659 | 24 | 122452 | 1160516 | 11251 | 9 | 1294243 | 2 | rust-wasm-reviewer |
| 2026-11-18T16:15:14Z | revision | agente:a51eefe48fd5d0834 | 44 | 117529 | 2011899 | 16121 | 3 | 2145593 | 3 | code-quality-reviewer |
| 2026-11-18T16:15:14Z | revision | agente:abf3fdeea1c184444 | 24 | 107868 | 1076440 | 10522 | 3 | 1194854 | 3 | rust-wasm-reviewer |

## Autorrevision

- rust-wasm-reviewer — 22ffc96
- code-quality-reviewer — 22ffc96
- rust-wasm-reviewer — 4589b99
- code-quality-reviewer — 4589b99
- rust-wasm-reviewer — eb0eb3d
- code-quality-reviewer — eb0eb3d
- rust-wasm-reviewer — cdbd5ba
- code-quality-reviewer — cdbd5ba


## Rondas extra

- Ronda 3: Carlos (consigna del 2026-11-18: opcion recomendada en las cuatro tareas del H0). IMP-1 seguia abierto solo por el cableado de HASH_OBJECT_INCOMPLETO en construir_commit; la correccion es barata (hashear con test de git real), asi que se verifica en una ronda mas en vez de aceptarlo.
- Ronda 4: Carlos (misma consigna). IMP-1 seguia abierto por el cableado de hashear en construir_commit, sin red con git real; se cierra con un test con git falso (construir_commit_rechaza_shas_incompletos) y el mutante W en rojo.
