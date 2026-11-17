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
tokens_implementacion: 34541197
tokens_revision: 32241090
creado: 2026-11-14
actualizado: 2026-11-17
dependencias: [TASK-018, TASK-021]
---
## Objetivo

`taskctl init` / `setup`: rama de estado, carpetas, config, workflow de CI y propuesta de rulesets

Tarea F3-1 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-3, F2-6.

## Criterios de aceptacion
- [x] C1: repo vacio (`git init`, sin remoto) → `tc2 init` sale `init ok`: rama `taskcodetwo/tareas` montada en `tareas/` (en `git worktree list`), `main` y `develop` con commit raiz que contiene `.taskcode/config.yml` (pasa `config::leer` = `Config::suelo()`) y `.gitignore` con `/tareas/` y `/.taskcodetwo/`, y `.git/info/exclude` con `/tareas/` y `/.taskcodetwo/worktrees/` (`crates/tc2/tests/init.rs::repo_vacio_sin_remoto`)
- [x] C2: la segunda ejecucion sale `init sin-cambios` sin ninguna accion `crea`: lo que la primera creo sale `decision=existe` y el resto (`pendiente`: personas, workflow y rulesets sin remoto ni plantilla) igual que en la primera; refs, worktrees, `exclude` y arbol (hashes) iguales (`init_dos_veces_sin_cambios`)
- [x] C3: `tc2 init --simular [--json]` imprime el plan y `init <res> simular=si` sin cambiar ni un byte (arbol, `.git`, refs; no crea `.git/taskcodetwo/`) y el plan simulado es igual a las acciones de la ejecucion real posterior (`simular_no_toca_nada_y_coincide`)
- [x] C4: con `config.yml` sin `formato: 2` o `tareas/` como carpeta normal → `init rechazada motivo=proyecto-016 arreglo="tc2 actualizar --simular"` (3) sin escribir nada; con `formato-nuevo` o config invalida tampoco la pisa (`init_en_repo_016_rechaza`, `init_con_config_invalida_no_la_pisa`)
- [x] C5: sin remoto `init ok` con `personas.yml` y puertas `decision=pendiente motivo=sin-remoto`, y tras `git remote add origin` un segundo `init` las completa; con remoto sin red → `init sinred pendiente=no` sin escribir nada (`sin_remoto_completa_al_anadirlo`, `sin_red_no_escribe`)
- [x] C6: una linea por ruleset `taskcodetwo-*` con `decision=persona` y la orden que lo aplica, sin ninguna escritura en la plataforma (0 POST/PUT/PATCH en el doble HTTP)
- [x] C7: tras `init`, `tc2 desinstalar --proyecto --simular` lista todo lo que creo `init` con `dueno=taskcodetwo` (`init_y_desinstalar_se_reconocen`)

## Autorrevision

- rust-wasm-reviewer — 1713060
- code-quality-reviewer — 1713060

## Rondas extra

- Ronda 4: Carlos (consigna del 2026-11-16, opcion recomendada); IMP-2 e IMP-3 de calidad r3 nacen de la correccion (texto del aviso falso y estado «no comprobable» tratado como «sin relacion»), son baratos.
- Ronda 3: Carlos (consigna del 2026-11-16, «aceptamos todas las respuestas de forma recomendada»); IMP-1 rust de la ronda 2 (orden de develop desde HEAD con historia sin relacion) y MEN-5 (comillas en PowerShell) son baratos y hay que verificarlos.

## Resultado

Hecho: `tc2 init` con plan como dato (inventario → plan → aplicacion, misma pieza que `desinstalar`) y `--simular`. Crea rama de estado `taskcodetwo/tareas` y su worktree (`estado::previsto` de solo lectura y `montar` sobre el), `main`/`develop` con commit raiz solo en un repo vacio cuyo `origin` no tiene ninguna rama, `.taskcode/config.yml` minima, `personas.yml` con la persona de la plataforma, `.git/info/exclude` y `.gitignore` (solo anade lineas). Con historia, todo lo que no puede hacer con seguridad queda `persona` con su orden; la relacion entre historia local y remota tiene tres estados (relacionada, sin relacion, no comprobable) y solo con relacion se propone push. Rulesets `taskcodetwo-*` solo como propuesta (los aplicara TASK-118; adenda en F1-5); workflow de CI pendiente de F5-1; hito y manifiesto fuera de `init` (adendas en F1-14 y F1-18). `desinstalar` reconoce lo que crea `init`. 20 tests en `tests/init.rs`, un test por criterio; `cargo xtask verificar --perfil rapida` en verde.

No verificado: GitHub/GitLab reales (doble HTTP), GitLab sin test propio en `init`, dos `init` concurrentes, `.gitignore` en proyectos reales; las ordenes encadenan con `&&`, que no vale en Windows PowerShell 5.1.

Revision (rust-wasm-reviewer y code-quality-reviewer):
- Ronda 1: cambios solicitados por IMP-1 (repo local vacio con `origin` con historia publicaba un `develop` sin relacion); corregido con los MENORES en 4a25582 y 2da8748.
- Ronda 2: calidad aprobada; rust abrio el caso de historia local sin relacion (orden desde `HEAD` publicaria historia ajena) y MEN-5 (comillas en PowerShell); corregidos en baef020.
- Ronda 3 (extra): calidad abrio IMP-2 (aviso falso sobre la rama de estado) e IMP-3 (no comprobable tratado como sin relacion); corregidos en c9b549c.
- Ronda 4 (extra): aprobada por los dos.

Incidentes: durante la implementacion un `cd` fallido hizo que `git symbolic-ref HEAD` y `git reset -q` corrieran en el repo real (restaurado en el acto; comprobado HEAD, `develop` y arbol limpio); dos revisores lanzaron ordenes de solo lectura o de compilacion en el repo original (sin cambios en git, comprobado).

MENORES aceptados (a seguimiento, TASK-119): MEN-5 rust (comillas tipograficas y shell decidida por `cfg!(windows)`), MEN-9 rust r4 (`git merge-base` que falla por otro motivo se toma por «sin ancestro»), MEN-3 calidad r1 (separadores mezclados en rutas) y las ordenes con `&&` en PowerShell 5.1.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T01:17:11Z | plan | automatico | persona | 0.16.1 |
| 2026-11-17T01:22:52Z | approve | automatico | persona | 0.16.1 |
| 2026-11-17T01:22:53Z | start | automatico | persona | 0.16.1 |
| 2026-11-17T02:13:42Z | review | automatico | persona | 0.16.1 |
| 2026-11-17T03:01:32Z | review | automatico | persona | 0.16.1 |
| 2026-11-17T03:20:58Z | review | automatico | persona | 0.16.1 |
| 2026-11-17T03:35:00Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-17T02:13:42Z | implementacion | sesion:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 36 | 13412 | 8788676 | 4602 | 0 | 8806726 | - | - |
| 2026-11-17T02:13:42Z | implementacion | agente:ad54a31f3b7594c06 | 208 | 657510 | 24227540 | 107292 | 47 | 24992550 | - | - |
| 2026-11-17T02:38:56Z | revision | agente:aa56fabbf82a2c151 | 128 | 170672 | 8032094 | 35315 | 7 | 8238209 | 1 | rust-wasm-reviewer |
| 2026-11-17T02:38:58Z | revision | agente:ab6086f3b91d9fb40 | 94 | 149722 | 5299433 | 24886 | 10 | 5474135 | 1 | code-quality-reviewer |
| 2026-11-17T03:01:32Z | implementacion | sesion:dif:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 2 | 706 | 740790 | 423 | 0 | 741921 | - | - |
| 2026-11-17T03:12:48Z | revision | agente:a7c1ea3e82c2b3c51 | 86 | 132344 | 4706548 | 37990 | 9 | 4876968 | 2 | rust-wasm-reviewer |
| 2026-11-17T03:12:48Z | revision | agente:af2a0f919aa9c7879 | 40 | 153768 | 2059515 | 44829 | 14 | 2258152 | 2 | code-quality-reviewer |
| 2026-11-17T03:27:20Z | revision | agente:a76a05dea73fb8f4d | 44 | 135234 | 2154556 | 39201 | 6 | 2329035 | 3 | rust-wasm-reviewer |
| 2026-11-17T03:27:21Z | revision | agente:a8d5a2eb573d3ccab | 42 | 136513 | 2071643 | 40679 | 4 | 2248877 | 3 | code-quality-reviewer |
| 2026-11-17T03:43:36Z | revision | agente:acb0c9d47f339c9bf | 70 | 144210 | 3679594 | 41796 | 20 | 3865670 | 4 | rust-wasm-reviewer |
| 2026-11-17T03:43:36Z | revision | agente:a764267aa7b913b51 | 60 | 132555 | 2789028 | 28401 | 9 | 2950044 | 4 | code-quality-reviewer |
