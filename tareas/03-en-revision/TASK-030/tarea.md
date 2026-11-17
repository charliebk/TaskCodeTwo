---
id: TASK-030
titulo: "F3-3 tc2 next: lo listo para empezar por dependencias y prioridad"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-030-f3-3-next-lo-listo-para-empezar-por-depe
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: 18164018
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
- [ ] C3: `tc2 next [--json]` imprime `next ok listas=... en-marcha=... commit=<sha>` (claves con guiones, contrato F1-4) en es y en, `--json` valido, sin red funciona sobre la copia del clon y sin rama de estado da `rechazada` (`tests/siguiente.rs`)
- [ ] C4: un `tarea.md` ilegible no se salta en silencio: sale en `ilegibles=`

## Notas de implementacion

- Fixture de C1: ToolsaWeb `1fc94425964a5d33b5e2c3c4c39691dbd580c641` (`grafo-tareas.py --comprobar` sale 0 en un clon temporal), en `crates/tc2/tests/fixtures/toolsaweb-1fc94425/`.
- Suposiciones del plan comprobadas en ese SHA: 404 tareas, todos los IDs de 3 cifras y sin sufijo; los 404 `tarea.md` reales pasan por el lector estricto de formato 1 y se proyectan igual que la fixture (test `#[ignore]` `toolsaweb_reales_se_leen_y_proyectan_como_la_fixture` con `TC2_TOOLSAWEB`); ninguna descartada esta terminada. La rama de estado plana (`TASK-NNN/tarea.md`) es la del ADR F1-1 y la que escriben `new` e `import`.
- Desviacion: las claves van con guiones (`en-marcha=`, `dependencias-inexistentes=`, `depende-de-descartada=`, `hitos-sin-titulo=`) y no con `_` como dicen C3 y el plan: el contrato de F1-4 fija «claves en minusculas con guiones» y `tests/comun::contrato` lo comprueba.
- Desviacion: un `tarea.md` ilegible bloquea a quien depende de el (`Fase::Ilegible`); tratarlo como inexistente lo dejaba como lista. Aviso nuevo `duplicadas=` para IDs repetidos.
- Desviacion: el lector de hitos (paso 4) entro con el nucleo en el paso 1 (`Hitos::del_roadmap`).
- Diferencia conocida con el oraculo: una dependencia de formato 1 con etiqueta `descartada` y estado `terminada` bloquea aqui (D3) y no en `grafo-tareas.py`; en el SHA de la fixture no hay ninguna.

## Autorrevision

- rust-wasm-reviewer — 484e64a
- code-quality-reviewer — 484e64a

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T05:57:46Z | plan | automatico | persona | 0.16.1 |
| 2026-11-17T06:02:15Z | approve | automatico | persona | 0.16.1 |
| 2026-11-17T06:02:16Z | start | automatico | persona | 0.16.1 |
| 2026-11-17T06:34:57Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-17T06:34:57Z | implementacion | sesion:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 12 | 10454 | 3475726 | 2486 | 0 | 3488678 | - | - |
| 2026-11-17T06:34:57Z | implementacion | agente:a4fc4ee548cfbb29f | 182 | 228763 | 14374815 | 71580 | 42 | 14675340 | - | - |
