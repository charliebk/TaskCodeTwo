---
id: TASK-053
titulo: "F6-7 Defensa contra prompt injection en PR y tareas"
tipo: feature
sprint: 6
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-053-f6-7-defensa-contra-prompt-injection-en
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 2837471
tokens_implementacion: 37286137
tokens_revision: 6809317
creado: 2026-11-14
actualizado: 2026-11-22
dependencias: [TASK-009, TASK-049]
---
## Objetivo

Defensa contra prompt injection en PR y tareas

Tarea F6-7 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-8, F6-3.

## Criterios de aceptacion
- [x] Batería de PR maliciosos
- [x] Cumple lo que pide la tarea F6-7 del plan y los requisitos que cita

## Resultado

Defensa contra prompt injection (F6-7, A9, A10, RR-08). La defensa sigue siendo por capacidades (F1-8 decision 4): sin detector de patrones. Se cierran los huecos que F6-3..F6-6 dejaron para F6-7 y la bateria queda como catalogo ejecutable reutilizable por F9-5.
- Modulo puro `crates/tc2/src/dato.rs`: unica regla de cerca del crate (`cerca`, minimo 4 acentos y una mas que el tramo mas largo del contenido), `bloque` rotulado «dato no fiable», y `plano`/`es_de_formato` movidos desde `hallazgos` (que los reexporta). `plan` deja su `valla` y el titulo de la tarea va por `plano` en la peticion de plan.
- `tc2 hallazgos prompt` (`hallazgos::prompt::componer`) compone el prompt de cada revisora: todo dato del PR o del modelo anterior (diff, abiertos, seccion F5-4, contexto de ronda) en su bloque; `SKILL.md` de la base en su cerca «de confianza»; un unico «## Lo que devuelves» fuera de cercas y al final. El workflow ya no arma el prompt con `echo`/`awk`.
- `tc2 hallazgos validar --prosa`: la prosa del modelo va al check en un bloque cercado, sin caracteres Cf y con el bloque entero (cercas incluidas) acotado a `TOPE_PROSA`.
- Lint de `xtask` `datos_inertes_en_el_workflow`: el prompt solo lo escribe `tc2 hallazgos prompt`, la prosa solo entra por `--prosa` y ningun `run:` saca a stdout respuesta, diff ni prompt (analiza tuberias y redirecciones; heuristico, y asi lo dice la guia).
- Bateria `crates/tc2/tests/fixtures/inyeccion/casos.json` (INY-01..INY-15, superficies prompt-ci, check, carpeta-tarea y plan-local) ejecutada por `crates/tc2/tests/inyeccion.rs`; un caso sin test da rojo.
- Adenda F6-7 en F1-8 (A9 y A10, sin tocar la matriz) y «Limites conocidos» de `docs/ci/revision-ia.md`.

Decisiones (opcion recomendada, delegada por Carlos, 2026-11-22): las ocho del plan. Riesgo aceptado: un modelo inyectado puede devolver 0 hallazgos o cerrar un IMPORTANTE con un cierre anclado; lo contiene que `revision-ia` verde no es aprobacion (residual de A9).

Evidencia (2026-11-22):
- C1: `tests/inyeccion.rs` con binario real y repo Git real: `el_prompt_encierra_todo_dato_del_pr` (INY-01..08; fuera de cercas el prompt es identico al de datos inocuos), `las_respuestas_hostiles_no_cambian_el_veredicto` (INY-09..13) y `la_carpeta_de_la_tarea_y_la_peticion_de_plan_quedan_inertes` (INY-14, INY-15). Mutaciones en rojo: diff sin bloque, cerca fija, prosa sin quitar Cf, prosa sin bloque, titulo y descripcion sin `plano`.
- C1 en GitHub real (sin modelo): check-run 137232652526 publicado con la App del revisor 5846307 en un commit temporal con la prosa hostil de INY-13: dentro del bloque no se presenta ningun enlace, imagen, `<details>` ni mencion; un enlace de control fuera del bloque si. Repo E2E limpio despues (rama borrada, sin PR ni workflows).
- C1 local: el paso «Seleccion y prompts» extraido tal cual del workflow y ejecutado con Git Bash sobre un PR hostil, en modo completa e incremental: ninguna linea hostil fuera de cercas.
- C2: unitarios de `dato` con tabla adversaria; la revisora de Rust paso ademas un fuzz determinista de 20 000 textos hostiles contra `bloque`, `componer` y `prosa` sin romper ninguna cerca. Lint con una mutacion por regla (35 tests de xtask).
- `cargo xtask verificar --perfil rapida` en verde sobre bc834c8 (18 min). Las correcciones de la ronda 1 (51ba9f4) se comprobaron con `cargo test -p xtask` (35), `cargo test -p tc2 --lib` (392), `--test inyeccion --test hallazgos` (12), clippy y fmt, 11 mutantes en rojo, y con `taskctl verificar-menores` (06126a6). `finish` vuelve a verificar.

Sin comprobar: un run real del workflow `revision-ia.yml` modificado (el reutilizable compila `tc2` desde un SHA publicado de TaskCodeTwo y la rama no se sube sin permiso); como responde un modelo real a la bateria (F9-5; sin saldo); que una mencion dentro de un bloque no notifique; inyecciones en otros idiomas, codificadas, repartidas u homoglifos.

Revision por pares:
- Ronda 1 (bc834c8): rust-wasm-reviewer y code-quality-reviewer, aprobada-con-correcciones, sin CRITICO ni IMPORTANTE. Los cuatro MENOR se corrigieron en 51ba9f4 y los reconocio `verificar-menores`.
  - Rust: MEN-1 el lint de stdout se saltaba con `>&2`, un `>` en otra orden o `grep`/`cut`; MEN-2 `TOPE_PROSA` no acotaba el bloque con sus cercas (32 KiB de acentos daban 98 KB).
  - Calidad: MEN-1 el mismo lint con mas variantes (`$respuesta` sin comillas, `${...}`, `prompts/0.md`); MEN-2 la guia describia el prompt anterior a F6-7.

## Divergencias

- `Partes` lleva `bytes` y `tope` en vez del booleano `cortado`; el contexto de ronda incremental lo escribe el workflow en `contexto.txt` y va en un bloque.
- `--skill` es opcional y `SKILL.md` va en su propia cerca rotulada «de confianza».
- La cerca usa como minimo 4 acentos (el awk del workflow usaba 3).
- INY-14 se prueba a nivel de biblioteca (`InformeTarea`): el camino Git ya lo cubre `tests/revision.rs`.
- INY-06 e INY-15: `tc2 new` no admite titulos multilinea ni carril o complejidad fuera de lista; se comprueba que salen de sus listas cerradas y que titulo y objetivo no llegan al prompt.
- INY-07 solo comprueba que la rama no llega al prompt; ningun test ejecuta el `sed` del workflow.
- E2E: no se ejecuto el workflow real (ver «Sin comprobar»); se sustituyo por la simulacion local del paso y el check-run real.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-22T13:06:31Z | plan | automatico | persona | 0.16.1 |
| 2026-11-22T13:16:03Z | approve | automatico | persona | 0.16.1 |
| 2026-11-22T13:16:05Z | start | automatico | persona | 0.16.1 |
| 2026-11-22T14:21:37Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-22T13:16:03Z | diseno | agente:abd337c8ada9470fb | 70 | 119915 | 2690589 | 26897 | 46 | 2837471 | - | - |
| 2026-11-22T14:21:37Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 22 | 16433 | 3670211 | 4713 | 0 | 3691379 | - | - |
| 2026-11-22T14:21:37Z | implementacion | agente:adb56f3f403a0bc63 | 290 | 675360 | 32793408 | 125700 | 38 | 33594758 | - | - |
| 2026-11-22T14:29:42Z | revision | agente:a52c89e7e1e0f5db3 | 62 | 147209 | 3884951 | 21255 | 4 | 4053477 | 1 | rust-wasm-reviewer |
| 2026-11-22T14:29:42Z | revision | agente:a77c229bd25e5de96 | 44 | 154953 | 2580472 | 20371 | 2 | 2755840 | 1 | code-quality-reviewer |

## Verificacion ligera

- 2026-11-22T14:44:28Z: ronda 1; commit revisado bc834c8d5e302c27d69f32e2d7ac3de286836974; commit verificado 51ba9f4a48189a2db09281d13295f633e31741c3; ficheros: crates/tc2/src/hallazgos/mod.rs, docs/ci/revision-ia.md, xtask/src/workflows.rs; corregidos: MEN-1, MEN-2, MEN-1, MEN-2; pendientes: ninguno
