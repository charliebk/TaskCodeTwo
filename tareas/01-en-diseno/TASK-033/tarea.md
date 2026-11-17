---
id: TASK-033
titulo: "F3-6 start: toma atomica de la tarea, rama y worktree desde develop"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-diseno
plan_aprobado: false
rama: feature/task-033-f3-6-start-toma-atomica-de-la-tarea-rama
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-17
dependencias: [TASK-017, TASK-018]
---
## Objetivo

`start`: toma atómica de la tarea, rama y worktree desde `develop`

Tarea F3-6 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-3, F2-2.

## Criterios de aceptacion
- [ ] C1: dos clones del mismo remoto bare lanzan `tc2 start TASK-001` a la vez (Barrier): uno sale 0 `start ok` y el otro 3 `motivo=tomada` con el login del ganador; en `origin/taskcodetwo/tareas` `asignado_a` es el ganador y solo el tiene rama y worktree (el perdedor no tiene rama, worktree ni commit de estado local)
- [ ] C2: repetir `start` siendo el dueno sale 0 `sin-cambios` sin commit nuevo en el estado remoto; si se borra el worktree o la rama local, se recrea solo lo que falta; una tarea `en-revision` mia se reanuda sin reescribir el estado
- [ ] C3: con el remoto inaccesible sale 4 `sinred pendiente=no` y no cambia el HEAD de `tareas/`, no hay `refs/taskcodetwo/pendientes/*` ni rama ni worktree
- [ ] C4: con una dependencia no `terminada` (o descartada) sale 3 `motivo=dependencias-pendientes` con los IDs y nada escrito
- [ ] C5: tarea `terminada`, `descartada` o en formato 1 da 3 con su motivo; inexistente da 3 `motivo=no-existe`; ID mal formado da uso (2); nada escrito
- [ ] C6: si el ID ya tiene rama `*/task-NNN-*` en el remoto y no la lleva nadie, sale 3 `motivo=ocupada` y nada escrito
- [ ] C7: tras `ok` el `tarea.md` publicado tiene `estado: en-curso`, `asignado_a` {id, login}, `actualizado` de hoy y una fila nueva en `## Transiciones`, con el resto byte a byte; `ROADMAP.md` regenerado con la tarea en curso; rama desde la punta de `origin/develop` sin upstream; worktree en `.taskcodetwo/worktrees/TASK-NNN`; `git status` del principal limpio
- [ ] C8: sin TTY y sin cache de identidad valida sale 3 `motivo=identidad` y nada escrito
- [ ] C9: lanzado desde un worktree de tarea escribe el estado en el principal; salida de una linea de contrato o JSON con `--json`

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T09:10:06Z | plan | automatico | persona | 0.16.1 |
