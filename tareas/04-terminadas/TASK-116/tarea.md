---
id: TASK-116
titulo: "Nombres de la 2.0 alineados con F1-12 D1a"
tipo: fix
sprint: 0
etiquetas: [convivencia, git]
complejidad: simple
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: fix/task-116-nombres-de-la-2-0-alineados-con-f1-12-d1
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: 10252651
tokens_revision: 25367681
creado: 2026-11-16
actualizado: 2026-11-16
dependencias: []
---
## Objetivo

El codigo de tc2 usa nombres que chocan con la 0.16 y contradicen F1-12 D1a (taskcode/tareas, .git/taskcode-estado.lock, .git/tc2/, .taskcode/worktrees). Alinearlos con D1a para que la 2.0 y la 0.16 no compartan nada y tc2 desinstalar (F2-14) pueda reconocer lo suyo por nombre.

## Criterios de aceptacion
- [x] La rama de estado es taskcodetwo/tareas (estado::RAMA) y los tests de estado y alta pasan con ese nombre
- [x] El cerrojo, la cache de identidad y lo demas que tc2 guarda en el directorio comun de git viven bajo .git/taskcodetwo/ (grep sin taskcode-estado.lock ni .git/tc2 en crates/tc2/src)
- [x] La carpeta de worktrees de tarea no coincide con ninguna ruta de la 0.16 (constante en git/mod.rs y test)
- [x] Un repo con huella de la 0.16 (.git/taskcode/, .taskcode/config.yml y tareas/ en la historia de develop) queda intacto tras new/start de la 2.0 (test)
- [x] Las refs propias de la 2.0 viven bajo refs/taskcodetwo/ y todos los nombres salen de un unico modulo (huella.rs)
- [x] cargo xtask verificar --perfil rapida en verde

## Resultado

Hecho: `crates/tc2/src/huella.rs` es la unica fuente de los nombres que la 2.0 deja en un repo y los alinea con F1-12 D1a: rama `taskcodetwo/tareas`, refs `refs/taskcodetwo/`, `.git/taskcodetwo/` (`estado.lock`, `identidad.yml`, `estado-tmp/`) y worktrees en `.taskcodetwo/worktrees`. `.taskcode/config.yml` y `personas.yml` no se mueven (F1-12 D4a). `update-ref` rechaza refs fuera de `refs/taskcodetwo/`. Adenda unica en la «Nota de nombres» de F1-12 con la tabla vieja → nueva y linea de adenda en F1-1..F1-11 y F1-16; `PLAN-TASKCODE-2.md` y el indice de ADR corregidos. Cada criterio con su test; `tests/convivencia_016.rs` reproduce la huella de la 0.16 (`.git/taskcode/`, `.taskcode/config.yml`, `tareas/` en la historia de develop) y falla con los nombres viejos, tambien por el camino sin red. `cargo xtask verificar --perfil rapida` en verde.

Comprobado en solo lectura: la cache del plugin 0.16 solo usa `.git/taskcode/`; OpenDockTrading y ToolsaWeb no tienen nada de la 2.0, asi que no hay migracion.

No verificado: un `tc2` anterior en marcha durante el cambio de nombre del cerrojo (riesgo aceptado: la 2.0 no esta desplegada); la 0.16 real ejecutada sobre un repo tocado por la 2.0 (el fixture se monta con lo encontrado).

Revision: ronda 1 aprobada con correcciones por los dos revisores (solo MENORES de documentacion y uno de cobertura del test); corregidos en 973642c, 76be757 y 75bb6b2. Como la correccion toco ADR no citados por ningun MENOR, se paso a cambios solicitados y se hizo la ronda 2, aprobada por los dos sin hallazgos. Nota: F1-13:451 ya explica el cambio y F1-19 esta descartado, por eso no llevan adenda.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-16T22:37:06Z | plan | automatico | persona | 0.16.1 |
| 2026-11-16T22:40:57Z | approve | automatico | persona | 0.16.1 |
| 2026-11-16T22:40:58Z | start | automatico | persona | 0.16.1 |
| 2026-11-16T23:02:23Z | review | automatico | persona | 0.16.1 |
| 2026-11-16T23:43:39Z | review | automatico | persona | 0.16.1 |
| 2026-11-16T23:54:24Z | finish | automatico | persona | 0.16.1 |

## Autorrevision

- rust-wasm-reviewer — 03c6f87
- code-quality-reviewer — 03c6f87

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-16T23:02:23Z | implementacion | sesion:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 8 | 9091 | 1905964 | 2056 | 0 | 1917119 | - | - |
| 2026-11-16T23:02:23Z | implementacion | agente:abed89afa4dbae9c1 | 124 | 178848 | 7473266 | 40849 | 7 | 7693087 | - | - |
| 2026-11-16T23:35:39Z | revision | agente:a58e9cdab2de6ec2d | 248 | 184819 | 15445532 | 31867 | 27 | 15662466 | 1 | rust-wasm-reviewer |
| 2026-11-16T23:35:41Z | revision | agente:aba5803665bcc6c05 | 102 | 133313 | 5223543 | 20934 | 8 | 5377892 | 1 | code-quality-reviewer |
| 2026-11-16T23:43:39Z | implementacion | sesion:dif:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 2 | 1046 | 641154 | 243 | 0 | 642445 | - | - |
| 2026-11-16T23:51:29Z | revision | agente:a783ede0f7ecce884 | 44 | 125362 | 2128246 | 18025 | 13 | 2271677 | 2 | rust-wasm-reviewer |
| 2026-11-16T23:51:30Z | revision | agente:a9cf41581be8e536f | 46 | 110275 | 1929779 | 15546 | 3 | 2055646 | 2 | code-quality-reviewer |

## Duracion

- Estimada: 120 min
- Real (de la rama abierta al cierre con merge, sin pausas): 73 min
- Paso de la hora: 73 min reales frente a 120 estimados
- Cierre (finish): 172 s, de 2026-11-16T23:51:32Z a 2026-11-16T23:54:24Z
