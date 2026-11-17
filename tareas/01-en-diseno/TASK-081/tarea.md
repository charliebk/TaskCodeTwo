---
id: TASK-081
titulo: "F3-10 Liberar y reasignar tareas abandonadas"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-diseno
plan_aprobado: true
rama: feature/task-081-f3-10-liberar-y-reasignar-tareas-abandon
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 2790273
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-17
dependencias: [TASK-033]
---
## Objetivo

Liberar y reasignar tareas: `taskctl liberar` y `reasignar`, aviso de tareas tomadas sin actividad en N días, con aviso a quien la tenía (RR-12)

Tarea F3-10 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F3-6.

Contexto (Carlos, 2026-11-14, RR-12 reforzado): el trabajo con varias personas debe estar garantizado: todos saben qué tarea tiene cada uno y no se pueden pisar ni los nombres ni las tareas. Además de lo ya previsto (IDs por push atómico, toma atómica en start, roadmap con quién lleva cada tarea, status), se cierran los huecos: nombres de rama y títulos únicos, identidad real de la plataforma (no texto libre), liberar y reasignar tareas abandonadas, aviso de solape de ficheros entre tareas en curso y estado siempre fresco con hora de sincronización. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 ter.

## Criterios de aceptacion
- [ ] C1: quien lleva una tarea `en-curso` lanza `tc2 liberar TASK-NNN`: se publica `estado: planificada`, `asignado_a: null` y la fila `liberar`, con `rama:` y `plan_aprobado` sin cambios y el resto del `tarea.md` byte a byte; `ROADMAP.md` del mismo commit; codigo 0 con `asignado-anterior=`; la rama local y la remota siguen en el mismo commit
- [ ] C2: tras C1, otra persona en otro clon hace `tc2 start TASK-NNN` y sale `ok` (no `ocupada`); su rama local apunta a la punta de `origin/<rama>` con los commits del dueno anterior en su historia y el worktree en `.taskcodetwo/worktrees/TASK-NNN`
- [ ] C3: `tc2 reasignar TASK-NNN beto` con `beto` en `personas.yml` solo cambia `asignado_a` a `{id, login}` de la lista y anade la fila `reasignar`, sin cambiar el estado; `beto` hace `start` y obtiene la rama existente sin perder commits
- [ ] C4: reasignar a un login que no esta en `personas.yml`, mal escrito o bot da 3 `motivo=fuera-de-lista` sin commit en `taskcodetwo/tareas`
- [ ] C5: tarea de otra persona con actividad hace menos de N dias: `liberar` y `reasignar` dan 3 `motivo=activa dias=K umbral=N` sin escribir; con N dias o mas (filas y commit de la rama envejecidos) proceden
- [ ] C6: lanzado sin terminal (agente) sobre una tarea de otra persona da `motivo=requiere-persona` aunque haya pasado N; sobre una tarea propia procede
- [ ] C7: sin red `liberar` y `reasignar` dan 4 `sinred pendiente=no` sin commit ni ref en `refs/taskcodetwo/pendientes/`
- [ ] C8: el dueno con commits sin publicar o cambios sin guardar en el worktree recibe `motivo=sin-publicar` antes de la transaccion con el estado intacto; publicado y limpio sale `worktree=quitado` y la rama sigue
- [ ] C9: `status` muestra `inactivas=1` cuando una tarea en curso supera N dias y `board` marca su fila; un commit reciente en `origin/<rama>` evita la marca aunque la ultima fila sea antigua
- [ ] C10: en el clon del dueno anterior, `status` avisa `ya-no-tuyas=TASK-NNN` con quien la lleva ahora, codigo 0
- [ ] C11: `liberar` de `terminada`, `descartada` o `en-revision` da su rechazo por estado; repetir `liberar` sobre una ya libre da `sin-cambios`
- [ ] C12: una tarea con `asignado_a` heredado de la 0.16 se reasigna a una persona de la lista y queda como objeto `{id, login}`

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T21:53:28Z | plan | automatico | persona | 0.16.1 |
| 2026-11-17T22:01:41Z | approve | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-17T22:01:40Z | diseno | agente:a16270a0169d2167e | 66 | 118729 | 2645939 | 25539 | 49 | 2790273 | - | - |
