---
id: TASK-028
titulo: "F3-1 tc2 init"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-028-f3-1-taskctl-init-setup
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: 33799276
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-17
dependencias: [TASK-018, TASK-021]
---
## Objetivo

`taskctl init` / `setup`: rama de estado, carpetas, config, workflow de CI y propuesta de rulesets

Tarea F3-1 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-3, F2-6.

## Criterios de aceptacion
- [ ] C1: repo vacio (`git init`, sin remoto) → `tc2 init` sale `init ok`: rama `taskcodetwo/tareas` montada en `tareas/` (en `git worktree list`), `main` y `develop` con commit raiz que contiene `.taskcode/config.yml` (pasa `config::leer` = `Config::suelo()`) y `.gitignore` con `/tareas/` y `/.taskcodetwo/`, y `.git/info/exclude` con las dos lineas (`crates/tc2/tests/init.rs::repo_vacio_sin_remoto`)
- [ ] C2: la segunda ejecucion sale `init sin-cambios` con todas las acciones `decision=existe` y refs, worktrees, `exclude` y arbol (hashes) iguales (`init_dos_veces_sin_cambios`)
- [ ] C3: `tc2 init --simular [--json]` imprime el plan y `init <res> simular=si` sin cambiar ni un byte (arbol, `.git`, refs; no crea `.git/taskcodetwo/`) y el plan simulado es igual a las acciones de la ejecucion real posterior (`simular_no_toca_nada_y_coincide`)
- [ ] C4: con `config.yml` sin `formato: 2` o `tareas/` como carpeta normal → `init rechazada motivo=proyecto-016 arreglo="tc2 actualizar --simular"` (3) sin escribir nada; con `formato-nuevo` o config invalida tampoco la pisa (`init_en_repo_016_rechaza`, `init_con_config_invalida_no_la_pisa`)
- [ ] C5: sin remoto `init ok` con `personas.yml` y puertas `decision=pendiente motivo=sin-remoto`, y tras `git remote add origin` un segundo `init` las completa; con remoto sin red → `init sinred pendiente=no` sin escribir nada (`sin_remoto_completa_al_anadirlo`, `sin_red_no_escribe`)
- [ ] C6: una linea por ruleset `taskcodetwo-*` con `decision=persona` y la orden que lo aplica, sin ninguna escritura en la plataforma (0 POST/PUT/PATCH en el doble HTTP)
- [ ] C7: tras `init`, `tc2 desinstalar --proyecto --simular` lista todo lo que creo `init` con `dueno=taskcodetwo` (`init_y_desinstalar_se_reconocen`)

## Autorrevision

- rust-wasm-reviewer — 1713060
- code-quality-reviewer — 1713060

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T01:17:11Z | plan | automatico | persona | 0.16.1 |
| 2026-11-17T01:22:52Z | approve | automatico | persona | 0.16.1 |
| 2026-11-17T01:22:53Z | start | automatico | persona | 0.16.1 |
| 2026-11-17T02:13:42Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-17T02:13:42Z | implementacion | sesion:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 36 | 13412 | 8788676 | 4602 | 0 | 8806726 | - | - |
| 2026-11-17T02:13:42Z | implementacion | agente:ad54a31f3b7594c06 | 208 | 657510 | 24227540 | 107292 | 47 | 24992550 | - | - |
