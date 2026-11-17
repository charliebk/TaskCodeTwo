---
id: TASK-034
titulo: "F3-7 descartar y pausa"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-curso
plan_aprobado: true
rama: feature/task-034-f3-7-descartar-y-pausa
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 2348184
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-17
dependencias: [TASK-018]
---
## Objetivo

`descartar` y `pausa` (RF-07, RF-08)

Tarea F3-7 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-3.

## Criterios de aceptacion
- [ ] C1: `tc2 descartar TASK-NNN "<motivo>"` sobre una tarea planificada, en diseno, en curso o en revision sale 0 `descartar ok`; el `tarea.md` publicado tiene `estado: descartada`, `motivo`, `actualizado` de hoy y fila `descartar` en Transiciones, con el resto byte a byte
- [ ] C2: sin motivo, vacio, solo espacios o con caracteres de control da uso (2) `motivo=sin-motivo|motivo-invalido` y la punta del estado no cambia
- [ ] C3: el `ROADMAP.md` del mismo commit lista la tarea en Descartadas con su motivo, fuera de hitos y listas, y `tc2 next` no la ofrece
- [ ] C4: tras descartar la tarea de numero mas alto, `tc2 new` y un `import` posterior siguen la numeracion sin reutilizar el ID; la carpeta sigue en la rama de estado
- [ ] C5: al descartar, con worktree limpio y rama publicada sale `worktree=quitado` y la rama local y remota siguen; con cambios sin guardar sale `ok ... worktree=conservado` y la carpeta queda intacta
- [ ] C6: tarea de otra persona da 3 `motivo=de-otra-persona asignado=<login>` sin publicar nada; sin asignado o propia, `ok`; igual para `pausa` y `reanudar`
- [ ] C7: descartar una descartada da `sin-cambios` sin commit y con el motivo original; descartar una terminada da 3 `motivo=terminada`; pausar o reanudar una terminada o descartada da 3
- [ ] C8: sin red `descartar` da 4 `sinred pendiente=no` sin commit ni ref pendiente; `pausa` propia da `sinred pendiente=si` y la siguiente transaccion con red la publica
- [ ] C9: `tc2 pausa` anade la fila `pausa` sin cambiar estado, asignado, rama ni worktree; una planificada en pausa sale de `next`, el ROADMAP la marca en pausa, `start` la rechaza `motivo=pausada`; repetir da `sin-cambios`
- [ ] C10: `tc2 reanudar` anade la fila `reanudar` y la tarea vuelve a `next`; reanudar una que no esta en pausa da `sin-cambios`
- [ ] C11: ID mal formado, sin ID o argumentos de mas dan uso (2) como `start`; `--json` da el mismo contenido en una linea

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T10:51:12Z | plan | automatico | persona | 0.16.1 |
| 2026-11-17T10:59:25Z | approve | automatico | persona | 0.16.1 |
| 2026-11-17T10:59:29Z | start | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-17T10:59:22Z | diseno | agente:ac028239097a018af | 66 | 106812 | 2218605 | 22701 | 44 | 2348184 | - | - |
