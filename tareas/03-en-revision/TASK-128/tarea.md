---
id: TASK-128
titulo: "Seguimiento de F3-11 y F3-12: aviso de solape contra uno mismo, textos al catalogo y test intermitente de progreso"
tipo: fix
sprint: 0
etiquetas: [solape, progreso]
complejidad: simple
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: fix/task-128-seguimiento-de-f3-11-y-f3-12-aviso-de-so
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1157361
tokens_implementacion: 20000464
tokens_revision: 4873148
creado: 2026-11-18
actualizado: 2026-11-18
dependencias: []
---
## Objetivo

Cerrar lo que quedo fuera de los ficheros citados en la revision de TASK-082 y el test intermitente que aparecio en ella.

## Criterios de aceptacion
- [ ] start no avisa de solape entre dos tareas de la misma persona (la propia lleva su quien hasta solape::calcular en la toma nueva); el test de liberar c5_c6_c10 pasa a exigir stderr vacio (test)
- [ ] AVISO_Y_MAS y NOTA_Y_MAS viven en i18n/catalogo/toma.rs y los recorre el test del catalogo
- [ ] progreso::c3_transiciones_reales_reescriben_el_progreso pasa 10 de 10 bajo carga: la rotacion de .anterior.json no se ignora en silencio o el test no depende de ella

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-18T17:54:47Z | plan | automatico | persona | 0.16.1 |
| 2026-11-18T18:00:59Z | approve | automatico | persona | 0.16.1 |
| 2026-11-18T18:01:00Z | start | automatico | persona | 0.16.1 |
| 2026-11-18T19:33:18Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-18T18:00:44Z | diseno | agente:acc352a6b97b6ff67 | 44 | 73228 | 1068011 | 16078 | 31 | 1157361 | - | - |
| 2026-11-18T19:33:18Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 114 | 75729 | 19890312 | 34309 | 1 | 20000464 | - | - |
| 2026-11-18T19:53:11Z | revision | agente:aad07a7702b52cb5d | 52 | 127517 | 2644008 | 9962 | 4 | 2781539 | 1 | rust-wasm-reviewer |
| 2026-11-18T19:53:11Z | revision | agente:ac476662f6d4e5db6 | 38 | 123611 | 1957052 | 10908 | 3 | 2091609 | 1 | code-quality-reviewer |

## Autorrevision

- rust-wasm-reviewer — 78f965e
- code-quality-reviewer — 78f965e

## Verificacion ligera

- 2026-11-18T19:57:26Z: ronda 1; commit revisado 6fdb49100d3066da1c76a255b171cb8ed232d052; commit verificado 5bd30202122bcc294ee819e46da6da2421c304c8; ficheros: crates/tc2/tests/progreso.rs; corregidos: ninguno; pendientes: MEN-1, MEN-1
