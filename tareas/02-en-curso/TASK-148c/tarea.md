---
id: TASK-148c
titulo: "servidor HTTP de los tests con comun::http_mock (A8-11) (parte c de TASK-148)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-curso
plan_aprobado: true
rama: fix/task-148-auditoria-4d-soporte-de-tests-compartido
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 545027
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-23
actualizado: 2026-11-24
dependencias: [TASK-148b]
---
## Objetivo

Parte c del troceo de TASK-148 (Auditoria 4D: soporte de tests compartido en tests/comun y modulos prueba (9 IMPORTANTE)), partes TASK-148a, TASK-148b, TASK-148c, TASK-148d, TASK-148e, TASK-148f en la rama fix/task-148-auditoria-4d-soporte-de-tests-compartido.

`tests/identidad.rs` y `tests/init.rs` usan `comun::http_mock::Mock` y `serie()` en lugar de su `Servidor` propio.

## Criterios de aceptacion
- [x] `git grep -n -e 'Servidor::nuevo' -e 'struct Servidor' -- crates/tc2/tests` no devuelve nada.
- [x] `cargo test -p tc2 --no-fail-fast` y `cargo xtask verificar` en verde; el `## Resultado` lista A8-11 con su commit.

## Resultado

Un solo doble HTTP en los tests: `comun::http_mock::Mock`, extendido con cuerpos binarios (`Simulada::bytes`, `cuerpo: Vec<u8>`) y respuestas cortadas (`Simulada::cortada(Corte)`, solo ese camino manda FIN). `identidad.rs`, `init.rs` y `actualizar_cli.rs` dejan su `Servidor` propio.

- **A8-11**: 5afdc1e (identidad), b932aeb (init), 4eeafeb (http_mock con bytes y corte, test nuevo), 4e55f40 (actualizar_cli; cierra A8-11).
- Criterio 1: `git grep -n -e 'Servidor::nuevo' -e 'struct Servidor' -- crates/tc2/tests` vacio; `TcpListener::bind` solo en `comun/http_mock.rs` y en cuatro puertos cerrados a proposito (no son servidores).
- Test nuevo `soporte.rs::http_mock_corta_una_vez_y_sirve_bytes_exactos`, en rojo con tres mutantes (corte que no corta, sin `shutdown(Write)`, cuerpo con `from_utf8_lossy`). Quitar `.cortada(..)` en `un_corte_en_la_descarga_del_binario_se_reintenta_una_vez` lo pone en rojo (la migracion no lo debilita).
- Recuento `cargo test -p tc2 --no-fail-fast` (target propio): antes 776 ok, 13 ignorados (cierre de TASK-148b); despues 48 binarios, 776 ok, 1 fallo, 13 ignorados = 777 (776 + 1 test nuevo). El fallo es `identidad::cuenta_fuera_de_lista_bot_o_web_flow_no_toca_la_cache` con `identidad sinred` (conexion lenta al mock HTTP en Windows, intermitencia conocida; otra vez en TASK-148b con `prs_abiertos`); el binario `identidad` repetido tres veces: 7 ok cada vez (48-69 s).
- `serie()` en los tests con mock de identidad, init y actualizar_cli (decision 3).
- `cargo xtask verificar` lo ejecuta `finish`.

## Divergencias

- `Simulada::cortada` es un metodo encadenado sobre una respuesta (`Simulada::bytes(200, &b).cortada(Corte::...)`), no un constructor: el corte a mitad del cuerpo necesita un cuerpo para anunciar `Content-Length`.
- Sin `Mock::lineas()`: el mapeo `recibidas().map(|r| r.linea)` va en linea, como ya hacen doctor e identidades.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-24T04:02:16Z | plan | automatico | persona | 0.17.1 |
| 2026-11-24T04:06:40Z | approve | automatico | persona | 0.17.1 |
| 2026-11-24T04:06:41Z | start | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-24T04:06:39Z | diseno | agente:afe790e928ce211c8 | 22 | 68724 | 466967 | 9314 | 16 | 545027 | - | - |

## Autorrevision

- rust-wasm-reviewer — 4e55f40
- 4D — 4e55f40: helper buscado, criterios citados, 1 test mutado
