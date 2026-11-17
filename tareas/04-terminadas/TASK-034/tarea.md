---
id: TASK-034
titulo: "F3-7 descartar y pausa"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-034-f3-7-descartar-y-pausa
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 2348184
tokens_implementacion: 28188137
tokens_revision: 29519494
creado: 2026-11-14
actualizado: 2026-11-17
dependencias: [TASK-018]
---
## Objetivo

`descartar` y `pausa` (RF-07, RF-08)

Tarea F3-7 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-3.

## Criterios de aceptacion
- [x] C1: `tc2 descartar TASK-NNN "<motivo>"` sobre una tarea planificada, en diseno, en curso o en revision sale 0 `descartar ok`; el `tarea.md` publicado tiene `estado: descartada`, `motivo`, `actualizado` de hoy y fila `descartar` en Transiciones, con el resto byte a byte
- [x] C2: sin motivo, vacio, solo espacios o con caracteres de control da uso (2) `motivo=sin-motivo|motivo-invalido` y la punta del estado no cambia
- [x] C3: el `ROADMAP.md` del mismo commit lista la tarea en Descartadas con su motivo, fuera de hitos y listas, y `tc2 next` no la ofrece
- [x] C4: tras descartar la tarea de numero mas alto, `tc2 new` y un `import` posterior siguen la numeracion sin reutilizar el ID; la carpeta sigue en la rama de estado
- [x] C5: al descartar, con worktree limpio y rama publicada sale `worktree=quitado` y la rama local y remota siguen; con cambios sin guardar sale `ok ... worktree=conservado` y la carpeta queda intacta
- [x] C6: tarea de otra persona da 3 `motivo=de-otra-persona asignado=<login>` sin publicar nada; sin asignado o propia, `ok`; igual para `pausa` y `reanudar`
- [x] C7: descartar una descartada da `sin-cambios` sin commit y con el motivo original; descartar una terminada da 3 `motivo=terminada`; pausar o reanudar una terminada o descartada da 3
- [x] C8: sin red `descartar` da 4 `sinred pendiente=no` sin commit ni ref pendiente; `pausa` propia da `sinred pendiente=si` y la siguiente transaccion con red la publica
- [x] C9: `tc2 pausa` anade la fila `pausa` sin cambiar estado, asignado, rama ni worktree; una planificada en pausa sale de `next`, el ROADMAP la marca en pausa, `start` la rechaza `motivo=pausada`; repetir da `sin-cambios`
- [x] C10: `tc2 reanudar` anade la fila `reanudar` y la tarea vuelve a `next`; reanudar una que no esta en pausa da `sin-cambios`
- [x] C11: ID mal formado, sin ID o argumentos de mas dan uso (2) como `start`; `--json` da el mismo contenido en una linea

## Autorrevision

- rust-wasm-reviewer — e4c2c67
- code-quality-reviewer — e4c2c67

Hallazgos propios (ninguno CRITICO ni IMPORTANTE):

- MENOR: `Descartar::exige_red` solo lo vigilan unitarios; el `fetch` previo
  de `descartar` ya da `sinred pendiente=no` antes de la transaccion.
- MENOR: la carrera `descartar` contra `start` simultaneos la cubre la
  reejecucion de la transaccion, sin test de dos procesos.
- Desviacion del plan: la anotacion de `pausa`/`reanudar` lleva el instante
  para que una pausa pendiente publique la hora real (F1-17).
- Mutaciones comprobadas (rojo y restauradas): dueno en `Lanzada::tarea`
  (c6 y unitarios), guarda `dentro_de` del worktree (c5), filtro de pausadas
  en `listas` (c9 y roadmap), rechazo `pausada` en `Tomar` (c9 y unitario).
- Verificacion propia: `--test ciclo --test toma` 25 de 25.

## Resultado

Hecho: modulo `ciclo/` con `tc2 descartar TASK-NNN "<motivo>"`, `tc2 pausa TASK-NNN` y `tc2 reanudar TASK-NNN` (y `--json`), operaciones puras `Descartar` y `Pausar` en `estado::transaccion` como `start`. `descartar` escribe `estado: descartada`, `motivo` (obligatorio, sin control, tope 500) y la fila de transicion, conserva `asignado_a` y `rama`, no borra la carpeta (el ID no se reutiliza) y, tras publicar, quita el worktree solo si esta limpio y publicado (`worktree=quitado|conservado|ninguno` con `worktree-motivo`); la rama nunca se borra. `pausa`/`reanudar` son filas de `## Transiciones` (sin estado nuevo; `Cuerpo::en_pausa`), con el instante en la anotacion para que una pausa pendiente sin red publique la hora real (F1-17). Las pausadas salen de `next` y el ROADMAP las marca; `start` las rechaza (`pausada`). Las descartadas muestran su motivo en el ROADMAP. Rechazos por asignado y por estado en un solo sitio (`estado::de_otra_persona`, `estado::por_estado`) para `start` y `ciclo`. Arreglado de paso: `personas.yml` y la config de la base se leian con pathspec relativo al cwd (`--full-tree`); `start`, `pausa` y `descartar` ya funcionan desde subcarpetas. Adendas en F1-2 (`descartar` exige red; `pausa`/`reanudar` quedan pendientes) y F1-4. D1-D7 con la opcion recomendada. `cargo xtask verificar --perfil rapida` en verde.

No verificado: `descartar` y `start` lanzados a la vez por dos procesos (lo cubre la reejecucion de la transaccion; probado solo en secuencia); plataforma real; el PR abierto de una tarea en revision descartada queda abierto (sin adaptador aqui, F6/F7).

Revision ronda 1: rust-wasm-reviewer aprobada (MENOR heredado: `sin-personas` desde una subcarpeta); code-quality-reviewer cambios-solicitados (IMPORTANTE: el `fetch` previo de `descartar` sin test que lo detecte; MENORES: `sin-publicar`/`sin-rama` sin E2E, adenda de F1-4 de `pausa`/`reanudar` desactualizada, decision duplicada entre `toma` y `ciclo`). Los cinco corregidos en c8acfda, a22dee0, ff5b236 y 7fffc65. Ronda 2: aprobada por los dos; cada correccion verificada por mutacion (siete mutaciones en rojo).

MENORES aceptados (sin seguimiento): el unitario comun no ejercita el texto de `asignado-heredado` (solo redaccion del diagnostico; el motivo lo cubren otros tests); `Descartar::exige_red` solo lo vigilan unitarios porque el `fetch` previo ya da `sinred`; con formato 1 mutilado (sin `formato: 2`) sale `ilegible` en vez de `formato-1`.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T10:51:12Z | plan | automatico | persona | 0.16.1 |
| 2026-11-17T10:59:25Z | approve | automatico | persona | 0.16.1 |
| 2026-11-17T10:59:29Z | start | automatico | persona | 0.16.1 |
| 2026-11-17T11:48:13Z | review | automatico | persona | 0.16.1 |
| 2026-11-17T12:36:36Z | review | automatico | persona | 0.16.1 |
| 2026-11-17T13:23:04Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-17T10:59:22Z | diseno | agente:ac028239097a018af | 66 | 106812 | 2218605 | 22701 | 44 | 2348184 | - | - |
| 2026-11-17T11:48:10Z | implementacion | agente:a0e0ce1e3af214169 | 160 | 575375 | 16695903 | 89028 | 49 | 17360466 | - | - |
| 2026-11-17T11:48:13Z | implementacion | sesion:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 28 | 16402 | 1633035 | 4285 | 0 | 1653750 | - | - |
| 2026-11-17T12:05:53Z | revision | agente:a8ae555df517d6b48 | 198 | 181361 | 13640333 | 38708 | 29 | 13860600 | 1 | rust-wasm-reviewer |
| 2026-11-17T12:05:54Z | revision | agente:a3cf195f52e197cb1 | 60 | 145864 | 3405597 | 18918 | 4 | 3570439 | 1 | code-quality-reviewer |
| 2026-11-17T12:36:36Z | implementacion | agente:aea032866151172ae | 138 | 176462 | 8953528 | 43793 | 21 | 9173921 | - | - |
| 2026-11-17T12:45:50Z | revision | agente:ac6c12a140af2e17f | 160 | 122715 | 7887379 | 19801 | 19 | 8030055 | 2 | code-quality-reviewer |
| 2026-11-17T13:13:48Z | revision | agente:a3d57c8359552e047 | 84 | 224418 | 3821419 | 12479 | 9 | 4058400 | 2 | rust-wasm-reviewer |

## Duracion

- Estimada: 210 min
- Real (de la rama abierta al cierre con merge, sin pausas): 144 min
- Paso de la hora: 144 min reales frente a 210 estimados
- Cierre (finish): 534 s, de 2026-11-17T13:14:10Z a 2026-11-17T13:23:04Z
