---
id: TASK-030
titulo: "F3-3 tc2 next: lo listo para empezar por dependencias y prioridad"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-curso
plan_aprobado: true
rama: feature/task-030-f3-3-next-lo-listo-para-empezar-por-depe
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
dependencias: [TASK-029]
---
## Objetivo

`next`: lo listo para empezar por dependencias y prioridad

Tarea F3-3 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F3-2.

## Criterios de aceptacion
- [ ] C1: con la instantanea fija de ToolsaWeb (fixture `crates/tc2/tests/fixtures/toolsaweb-<sha>/` con `tareas.tsv`, `hitos.txt` y `esperado.txt` = linea «Listas para empezar» del `ROADMAP.md` generado por `scripts/grafo-tareas.py` en ese SHA), el nucleo de `tc2 next` da exactamente el mismo vector de listas, en el mismo orden, y `en_marcha` coincide
- [ ] C2: reglas del nucleo `grafo.rs` con un test por regla: dependencia inexistente (no bloquea, avisa), dependencia descartada (bloquea, avisa), ciclo (atascadas, avisa), dependencia terminada de un hito posterior, IDs con sufijo, hito sin titulo
- [ ] C3: `tc2 next [--json]` imprime `next ok listas=... en_marcha=... commit=<sha>` en es y en, `--json` valido, sin red funciona sobre la copia del clon y sin rama de estado da `rechazada` (`tests/siguiente.rs`)
- [ ] C4: un `tarea.md` ilegible no se salta en silencio: sale en `ilegibles=`

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T05:57:46Z | plan | automatico | persona | 0.16.1 |
| 2026-11-17T06:02:15Z | approve | automatico | persona | 0.16.1 |
| 2026-11-17T06:02:16Z | start | automatico | persona | 0.16.1 |
