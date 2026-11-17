---
id: TASK-079
titulo: "F3-5b status y roadmap siempre frescos"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-diseno
plan_aprobado: false
rama: feature/task-079-f3-5b-status-y-roadmap-siempre-frescos
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
dependencias: [TASK-032]
---
## Objetivo

`status` y `roadmap` siempre frescos: traen la rama de estado antes de mostrar y dicen la hora de la última sincronización (RR-12)

Tarea F3-5b del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F3-5.

Contexto (Carlos, 2026-11-14, RR-12 reforzado): el trabajo con varias personas debe estar garantizado: todos saben qué tarea tiene cada uno y no se pueden pisar ni los nombres ni las tareas. Además de lo ya previsto (IDs por push atómico, toma atómica en start, roadmap con quién lleva cada tarea, status), se cierran los huecos: nombres de rama y títulos únicos, identidad real de la plataforma (no texto libre), liberar y reasignar tareas abandonadas, aviso de solape de ficheros entre tareas en curso y estado siempre fresco con hora de sincronización. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 ter.

## Criterios de aceptacion
- [ ] C1: con dos clones A y B de un remoto bare, A publica con `tc2 new` y `tc2 status --json` en B, sin nada antes, ve la tarea nueva con el `commit=` que A publico; igual con `next --json`, `board --sin-plataforma --json` y `roadmap --json`
- [ ] C2: sin red (tras un `status` con red), `status` sale 0 con la ultima foto, `remoto=sin-red`, `sincronizado=` igual a la hora del ultimo fetch con exito y un aviso en stderr con esa hora y el «hace N»; sin sincronizar nunca, `sincronizado=nunca` con aviso
- [ ] C3: la hora sale en texto (linea de `status` y cabecera de `board`; en `roadmap`, stderr) y en `--json` de las cuatro ordenes con `sincronizado` y `remoto`
- [ ] C4: tras las cuatro ordenes con red no cambian `refs/heads/taskcodetwo/tareas`, `git status --porcelain` del principal ni de `tareas/`, el bare no gana commits, el registro del puerto no tiene `push`, `commit-tree` ni `update-ref`; solo cambian `refs/remotes/origin/taskcodetwo/tareas` y `.git/taskcodetwo/sincronizacion`
- [ ] C5: tras un `tc2 new` con exito, `status` sin red da `sincronizado=` igual o posterior a la hora de ese `new`
- [ ] C6: con credenciales denegadas sale 0 con `remoto=denegado` y aviso; sin remoto no hace fetch y sale `remoto=sin-remoto`
- [ ] C7: `status` dice `pendientes=N` con las refs de `refs/taskcodetwo/pendientes/` y avisa `tareas=con-cambios|diverge` con codigo 0
- [ ] C8: `doctor` no hace fetch (no aparece en su registro) y muestra `sincronizado=`
- [ ] C9: el stdout de `roadmap` es byte a byte el ROADMAP que `--escribir` publicaria

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T16:04:12Z | plan | automatico | persona | 0.16.1 |
