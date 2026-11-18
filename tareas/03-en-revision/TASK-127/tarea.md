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
tokens_revision: 16457765
creado: 2026-11-18
actualizado: 2026-11-18
dependencias: []
---
## Objetivo

Una operacion de estado que escribe muchos ficheros a la vez (p. ej. 1000 en un import grande) falla con git=hash-object en Windows, probablemente por la longitud de la linea de ordenes (visto en TASK-084).

## Criterios de aceptacion
- [x] Una transaccion que escribe 1000 ficheros en una sola operacion termina en ok en Windows (test)
- [x] El numero de procesos de git no crece con el numero de ficheros mas alla de un tope fijo (hash-object por stdin o por tandas)

## Resultado

Hecho: `construir_commit` (`git/estado.rs`) pasa las rutas de los blobs a `hash-object -w --no-filters --stdin-paths` y las entradas del indice a `update-index --add -z --index-info` por stdin, no por argumentos. Con 1000 ficheros la linea de ordenes pasaba del tope de Windows y fallaba `git=hash-object`; ahora un commit de estado lanza un numero fijo de procesos sea cual sea el numero de ficheros. El puerto `Git` admite stdin solo en tres subcomandos: `cat-file` (como antes) y las dos formas exactas nuevas (`ejecutar_con_entrada`, `ejecutar_con_indice_y_entrada` con indice temporal absoluto); `update-ref --stdin` sigue prohibido. Adenda en el ADR F1-4. El control de un sha por fichero vive en `hashear`. Se quita el rodeo `chunks(100)` de `tests/progreso.rs`. D1a, D2, D3 decididas por Carlos al planificar. Test `mil_ficheros_en_una_transaccion_con_procesos_acotados` rojo sin el arreglo (`git=hash-object`) y verde con el; `cargo xtask verificar --perfil rapida` en verde.

No verificado: la rama unix del git falso de `construir_commit_rechaza_shas_incompletos` no se ha ejecutado en esta maquina (la correra la CI de Linux y macOS); revisada a mano.

Revision: rust-wasm-reviewer aprobo en las cuatro rondas sin hallazgos. code-quality-reviewer: IMP-1 (el camino de fallo de git con stdin y el control de `HASH_OBJECT_INCOMPLETO` sin test) se cerro en tres pasos: tests de `con_entrada` y de la cuenta (ronda 2), el control movido a `hashear` con test de git real (ronda 3) y un test de `construir_commit` con un git falso que devuelve un solo sha (ronda 4); cada mutante del control y del cableado pone un test en rojo. MEN-1 (la adenda decia que `cat-file` tenia forma exacta) y MEN-2 (`con_entrada` reimplementaba `git_fallo`) corregidos en la ronda 2. Rondas 3 y 4 son rondas extra por la consigna de Carlos (opcion recomendada), anotadas en `## Rondas extra`. Sin MENORES pendientes.

Incidencias de proceso: el freno de fin de turno lanzo la suite a la vez que la verificacion del orquestador y se corto a los 1500 s (las dos compitiendo); una verificacion de la ronda 4 cayo por el test intermitente `progreso::c3_transiciones_reales_reescriben_el_progreso` (lo arregla TASK-128), que solo paso 3 de 3 y en la repeticion la suite salio en verde.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-18T13:17:32Z | plan | automatico | persona | 0.16.1 |
| 2026-11-18T13:23:44Z | approve | automatico | persona | 0.16.1 |
| 2026-11-18T13:23:58Z | start | automatico | persona | 0.16.1 |
| 2026-11-18T14:10:54Z | review | automatico | persona | 0.16.1 |
| 2026-11-18T15:12:21Z | review | automatico | persona | 0.16.1 |
| 2026-11-18T16:02:28Z | review | automatico | persona | 0.16.1 |
| 2026-11-18T17:26:49Z | review | automatico | persona | 0.16.1 |

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
| 2026-11-18T17:32:13Z | revision | agente:a2061ce820889c1be | 24 | 115230 | 1073518 | 10833 | 5 | 1199605 | 4 | code-quality-reviewer |
| 2026-11-18T17:32:13Z | revision | agente:ab47282db13c87e91 | 38 | 118885 | 1835920 | 15629 | 2 | 1970472 | 4 | rust-wasm-reviewer |

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
