---
id: TASK-033
titulo: "F3-6 start: toma atomica de la tarea, rama y worktree desde develop"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-033-f3-6-start-toma-atomica-de-la-tarea-rama
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 2173261
tokens_implementacion: 27824718
tokens_revision: 7661928
creado: 2026-11-14
actualizado: 2026-11-17
dependencias: [TASK-017, TASK-018]
---
## Objetivo

`start`: toma atómica de la tarea, rama y worktree desde `develop`

Tarea F3-6 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-3, F2-2.

## Criterios de aceptacion
- [x] C1: dos clones del mismo remoto bare lanzan `tc2 start TASK-001` a la vez (Barrier): uno sale 0 `start ok` y el otro 3 `motivo=tomada` con el login del ganador; en `origin/taskcodetwo/tareas` `asignado_a` es el ganador y solo el tiene rama y worktree (el perdedor no tiene rama, worktree ni commit de estado local)
- [x] C2: repetir `start` siendo el dueno sale 0 `sin-cambios` sin commit nuevo en el estado remoto; si se borra el worktree o la rama local, se recrea solo lo que falta; una tarea `en-revision` mia se reanuda sin reescribir el estado
- [x] C3: con el remoto inaccesible sale 4 `sinred pendiente=no` y no cambia el HEAD de `tareas/`, no hay `refs/taskcodetwo/pendientes/*` ni rama ni worktree
- [x] C4: con una dependencia no `terminada` (o descartada) sale 3 `motivo=dependencias-pendientes` con los IDs y nada escrito
- [x] C5: tarea `terminada`, `descartada` o en formato 1 da 3 con su motivo; inexistente da 3 `motivo=no-existe`; ID mal formado da uso (2); nada escrito
- [x] C6: si el ID ya tiene rama `*/task-NNN-*` en el remoto y no la lleva nadie, sale 3 `motivo=ocupada` y nada escrito
- [x] C7: tras `ok` el `tarea.md` publicado tiene `estado: en-curso`, `asignado_a` {id, login}, `actualizado` de hoy y una fila nueva en `## Transiciones`, con el resto byte a byte; `ROADMAP.md` regenerado con la tarea en curso; rama desde la punta de `origin/develop` sin upstream; worktree en `.taskcodetwo/worktrees/TASK-NNN`; `git status` del principal limpio
- [x] C8: sin TTY y sin cache de identidad valida sale 3 `motivo=identidad` y nada escrito
- [x] C9: lanzado desde un worktree de tarea escribe el estado en el principal; salida de una linea de contrato o JSON con `--json`

## Autorrevision

- rust-wasm-reviewer — d4557dc
- code-quality-reviewer — d4557dc

Hallazgos propios (ninguno CRITICO ni IMPORTANTE):

- MENOR: la fila de `## Transiciones` lleva `modo: manual` fijo porque la
  2.0 aun no guarda el modo en la tarea (llega con F1-10).
- MENOR: si una tarea planificada ya estuviera asignada a quien la lanza y
  tuviera rama propia, `comprobar-id` daria `ocupada`; hoy no se da porque
  el alta no asigna.
- Desviacion del plan: C3 se prueba con un ssh que responde «Connection
  refused» (una ruta inexistente da `denegada`, no `sinred`).
- Mutacion comprobada: anular el rechazo por tarea ajena en `Tomar::aplicar`
  hace fallar `otra_persona_la_lleva` y `c1_dos_clones_a_la_vez_solo_uno_la_toma`.

## Resultado

Hecho: `tc2 start [--json] TASK-NNN` (modulo `toma/`). La toma es una operacion pura `Tomar` en el punto unico de escritura (`estado::transaccion`, se reejecuta en cada intento y regenera el ROADMAP) que escribe `estado: en-curso`, `asignado_a`, `actualizado`, la fila de `## Transiciones` y `rama` si falta; solo con la toma publicada se crean la rama (desde `origin/develop`, o el ultimo tag de `origin/main` para `hotfix`; `release` se rechaza) y el worktree en `.taskcodetwo/worktrees/TASK-NNN` (F1-2 D4, F1-3). Si ya es mia y empezada, `sin-cambios` y se recrea solo lo que falte. Antes de publicar: `comprobar-id` (rama ajena = `ocupada`) y carpeta del worktree libre. Extracciones compartidas: `grafo::dependencia_cumplida`, `identidad::{resolver,destino_de,adaptador}` (tambien en `tablero`), `Cuerpo::anadir_transicion`, `git::partes_de_rama`. Adenda en F1-4 con la linea y los motivos de `start`. Decisiones D1-D6 con la opcion recomendada. 13 E2E con dos clones contra un remoto bare y 8 unitarios; `cargo xtask verificar --perfil rapida` en verde.

No verificado: plataforma real (identidad desde cache y mock); Linux y macOS.

Revision ronda 1: rust-wasm-reviewer aprobada (MENOR-1: con la carpeta del worktree ocupada la toma se publicaba y luego salia `rechazada`); code-quality-reviewer escribio «aprobada» con un IMPORTANTE abierto y se registro como `cambios-solicitados` (I1: quitar `--merged origin/main` en la base del hotfix no ponia rojo ningun test; M1: `creado=` en la primera toma sin test; M2: `rama-invalida` sin E2E). Corregidos en b723444 con test cada uno. Ronda 2: aprobada por los dos; las cuatro correcciones verificadas por mutacion.

MENORES aceptados (sin seguimiento; casos que hoy no se dan o decisiones ya tomadas): la fila de transicion lleva `modo: manual` fijo hasta F1-10; una tarea planificada ya asignada a quien la lanza y con rama propia daria `ocupada`; una dependencia inexistente no bloquea (regla de `next`, solo con unitario); al reanudar se valida igualmente la base y `tipo-release`; `Destino::crear` reconstruye el `Informe` con `std::mem::take`; carrera inherente entre `carpeta_libre` y `worktree add` (sale con `estado=`); un enlace simbolico roto en la ruta del worktree no se detecta antes de publicar.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T09:10:06Z | plan | automatico | persona | 0.16.1 |
| 2026-11-17T09:17:34Z | approve | automatico | persona | 0.16.1 |
| 2026-11-17T09:17:40Z | start | automatico | persona | 0.16.1 |
| 2026-11-17T09:56:25Z | review | automatico | persona | 0.16.1 |
| 2026-11-17T10:24:45Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-17T09:56:25Z | implementacion | sesion:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 22 | 12468 | 947116 | 4846 | 0 | 964452 | - | - |
| 2026-11-17T09:56:25Z | implementacion | agente:a5619a3e8f10a58f4 | 178 | 351143 | 23437074 | 96647 | 28 | 23885042 | - | - |
| 2026-11-17T10:43:26Z | revision | agente:a6502ba78cc36cc96 | 24 | 110564 | 1065311 | 7933 | 2 | 1183832 | 1 | rust-wasm-reviewer |
| 2026-11-17T10:43:27Z | revision | agente:a769196cca62715b6 | 56 | 131072 | 2893870 | 15497 | 3 | 3040495 | 1 | code-quality-reviewer |
| 2026-11-17T10:43:28Z | revision | agente:a128daea1c93f9fb4 | 52 | 148379 | 1698047 | 9038 | 4 | 1855516 | 2 | rust-wasm-reviewer |
| 2026-11-17T10:43:29Z | revision | agente:a64ca61e74183ba9c | 44 | 158487 | 1416030 | 7524 | 2 | 1582085 | 2 | code-quality-reviewer |
| 2026-11-17T10:43:30Z | diseno | agente:ab8819b089d7eb5f4 | 50 | 128206 | 2023869 | 21136 | 43 | 2173261 | - | - |
| 2026-11-17T10:43:31Z | implementacion | agente:a362cdb91f4b6c502 | 60 | 124090 | 2835588 | 15486 | 11 | 2975224 | - | - |
