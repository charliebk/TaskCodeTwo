---
id: TASK-031
titulo: "F3-4 tc2 roadmap: la vista para todos"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-curso
plan_aprobado: true
rama: feature/task-031-f3-4-roadmap-la-vista-para-todos
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
dependencias: [TASK-018]
---
## Objetivo

`roadmap`: la vista para todos (en curso con quién, en revisión, planificadas por hito, terminadas, descartadas) con dependencias y Mermaid, regenerada en la rama de estado tras cada cambio (RR-10, RR-11)

Tarea F3-4 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-3.

## Criterios de aceptacion
- [ ] C1: dos clones lanzan `tc2 new` a la vez y los dos acaban `ok`; el `ROADMAP.md` final tiene las dos tareas, `git log --merges taskcodetwo/tareas` vacio y `tc2 roadmap --comparar` al dia (ampliando `tests/alta.rs` y `tests/estado_concurrencia.rs`)
- [ ] C2: editada la zona manual, tras `new`, `import` y `roadmap --escribir` sigue identica byte a byte, tambien con CRLF (tests en `roadmap/zona.rs` y `tests/roadmap.rs`)
- [ ] C3: `for-each-ref`, `status --porcelain` y el arbol de `HEAD` antes y despues de `new`, `import`, `roadmap` y `roadmap --escribir`: solo cambian `refs/heads/taskcodetwo/tareas` y su ref remota; `roadmap` y `--comparar` no cambian ninguna ref
- [ ] C4: sobre la instantanea de ToolsaWeb y en un repo de prueba, «Listas para empezar» y «En marcha» del ROADMAP son iguales a `listas=` y `en-marcha=` de `tc2 next`
- [ ] C5: la salida de referencia cubre cada seccion (con quien, revision, hito con dependencias y Mermaid, terminadas y descartadas) y `--escribir` dos veces seguidas da `sin-cambios` la segunda
- [ ] C6: con la instantanea de ToolsaWeb (titulos de 60 caracteres) el fichero no pasa de 100 KB y cada bloque Mermaid tiene menos de 500 aristas y menos de 50 000 caracteres

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T06:49:37Z | plan | automatico | persona | 0.16.1 |
| 2026-11-17T06:54:06Z | approve | automatico | persona | 0.16.1 |
| 2026-11-17T06:54:07Z | start | automatico | persona | 0.16.1 |
