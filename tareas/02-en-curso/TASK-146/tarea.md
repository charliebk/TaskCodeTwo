---
id: TASK-146
titulo: "Auditoria 4D: A8 transversal, actualizar_cli, instalacion, contrato, dato y xtask (7 IMPORTANTE)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-curso
plan_aprobado: true
rama: fix/task-146-auditoria-4d-a8-transversal-actualizar-c
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1360634
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-23
actualizado: 2026-11-25
dependencias: []
---
## Objetivo

Corregir los 7 casos IMPORTANTE del grupo A8 de `docs/auditoria-4d.md` (commit auditado d82ea9d) que no son de abstracción común: 2 Duplicaciones internas y 5 Disfraces de test con mutante confirmado. No se piden los MENOR (A8-14, A8-21, A9-04, A11-05, A11-06, A11-07). No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.

## Criterios de aceptacion
- [x] A8-15: `detectar_modo_con_rutas_de_los_tres_sistemas` (`instalacion.rs:280-332`) tiene una ruta que solo reconoce la rama `opt/homebrew` (`:56`) y otra que solo reconoce `winget/links` (`:62`). Con cada uno de los dos mutantes de la ficha aplicado, `cargo test -p tc2 --lib instalacion::tests::detectar_modo_con_rutas_de_los_tres_sistemas` sale en rojo.
- [x] A9-02: un test hace fallar `comprobar_nuevo` (`actualizar_cli.rs:406-415` y `:710-745`) con un binario que no arranca, otro que dice otra versión o target y otro que da otro sha256, y espera `binario-no-arranca` sin tocar el `tc2` instalado. Con cada uno de los dos mutantes de la ficha aplicado, `cargo test -p tc2 --test actualizar_cli` sale en rojo.
- [x] A11-02: los tests de `instalacion` comparan el texto exacto de las cinco órdenes de `orden_del_gestor` (`instalacion.rs:77-87`), no solo que no estén vacías (`:317`), y un caso de `rutas_del_entorno_por_sistema` con `TC2_INSTALL_DIR` igual a la carpeta por defecto exige un solo elemento en `dirs_de_instalacion()` (`:223-234`). Con cada uno de los dos mutantes de la ficha aplicado, `cargo test -p tc2 --lib instalacion::tests` sale en rojo.
- [x] A11-03: `plano_deja_el_texto_inerte` (`dato.rs:207-227`) cubre el escape de `*`, `_`, `~` y `&` con su salida exacta (p. ej. `&#x202E; **n** _x_ ~~t~~` → `\&\#x202E; \*\*n\*\* \_x\_ \~\~t\~\~`). Con cada uno de los dos mutantes de la ficha aplicado, `cargo test -p tc2 --lib dato::tests` sale en rojo.
- [x] A11-04: `linea_de_contrato_y_json_con_las_mismas_claves` (`contrato.rs:367-406`) pasa un valor con ESC y BEL y exige `\u001b` y `\u0007` en el JSON, que además se parsea. Con el mutante de la ficha aplicado (`contrato.rs:336` quitado), `cargo test -p tc2 --lib contrato::tests` sale en rojo.
- [x] A8-13: el «comando de shell según el sistema» de los tests de `ejecutar_con_limite` (`actualizar_cli.rs:788-797`, `:804-813`, `:829-838`) queda en un ayudante.
- [x] A9-03: el cierre `runs_de` (`xtask/src/workflows.rs:2712-2721`) usa `runs_del_job` (`:2162-2173`) del mismo módulo. `cargo test -p xtask` sigue en verde.
- [x] `cargo xtask verificar` en verde, y el `## Resultado` lista los 7 IDs con su commit.

## Resultado

Corregidos los 7 IMPORTANTE del grupo A8 de la ficha (sin los MENOR). Solo cambian tests y el doble `tc2_falso`; ningun codigo de produccion.

| Ficha | Commit |
|---|---|
| A11-03 (D4) | e397f27 |
| A11-04 (D4) | ad31277 |
| A8-15 y A11-02 (D4) | e9090c9 |
| A8-13 | 0fac017 |
| A9-02 (D4) | eccab4f |
| A9-03 | 3047ee2 |

Mutantes (los 9 de la ficha), todos en rojo con la orden del criterio: `dato.rs:88` y `:76` quitados; `contrato.rs:345` quitado (la auditoria decia `:336`); `instalacion.rs:56` quitado y `:62` solo con `packages`; `:79` con otro texto y `:229-231` como `v.push(d)`; `actualizar_cli.rs:406` como `Ok(())` y `:738` como `if false`.

Suite completa (`cargo test --workspace`, target propio): 852 ok, 0 fallidos, 13 ignorados. `cargo xtask verificar` lo ejecuta `finish`.

### Divergencias

- `lanzar_con` tambien quita `TC2_FALSO_SHA256` del entorno heredado, como ya hacia con `GH_FALSO*`.
- `cargo test -p tc2 --test actualizar_cli` solo no compila los ejemplos: con un target limpio hace falta `cargo build -p tc2 --examples` antes (`cargo test --workspace` si los compila).
- A9-03 comprueba el texto unido de los `run` del job en vez de cada `run` por separado (igual o mas estricto tras `normalizar`).

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-25T11:48:23Z | plan | automatico | persona | 0.17.1 |
| 2026-11-25T11:53:43Z | approve | automatico | persona | 0.17.1 |
| 2026-11-25T11:54:44Z | start | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-25T11:53:41Z | diseno | agente:ada368e0e37ee38a9 | 46 | 80472 | 1266188 | 13928 | 30 | 1360634 | - | - |

## Autorrevision

- rust-wasm-reviewer — 3047ee2
- 4D — 3047ee2: helper buscado, criterios citados, 9 tests mutados
