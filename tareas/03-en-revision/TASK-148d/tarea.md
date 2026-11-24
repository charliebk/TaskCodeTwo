---
id: TASK-148d
titulo: "instantanea de carpeta comun (A8-12, absorbe A7-06) (parte d de TASK-148)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: fix/task-148-auditoria-4d-soporte-de-tests-compartido
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 316853
tokens_implementacion: 6372184
tokens_revision: null
creado: 2026-11-23
actualizado: 2026-11-24
dependencias: [TASK-148c]
---
## Objetivo

Parte d del troceo de TASK-148 (Auditoria 4D: soporte de tests compartido en tests/comun y modulos prueba (9 IMPORTANTE)), partes TASK-148a, TASK-148b, TASK-148c, TASK-148d, TASK-148e, TASK-148f en la rama fix/task-148-auditoria-4d-soporte-de-tests-compartido.

Crea `comun/instantanea.rs` (recursiva, anota carpetas, `"<len> <sha256>"`, con lista de exclusión) y migra init, desinstalar, importar, tablero, frescura y doctor.

## Criterios de aceptacion
- [x] Ninguno de `tests/init.rs`, `tests/desinstalar.rs`, `tests/importar.rs`, `tests/tablero.rs`, `tests/frescura.rs` y `tests/doctor.rs` recorre carpetas por su cuenta: usan `comun::instantanea`.
- [x] Un test de `tests/soporte.rs` comprueba que una carpeta vacía nueva y un fichero en una subcarpeta cambian la foto, y que un nombre excluido no la cambia.
- [x] `cargo test -p tc2 --no-fail-fast` y `cargo xtask verificar` en verde; el `## Resultado` lista A8-12 y A7-06 con su commit.

## Resultado

Una sola instantanea de carpeta en `crates/tc2/tests/comun/instantanea.rs` (`instantanea(dir, excluir)`: recursiva, claves relativas, carpetas como `"dir"`, ficheros como `"<len> <sha256>"`, `excluir` por nombre a cualquier profundidad), usada por los seis tests que tenian la suya.

- **A8-12**: 9cbdb86 (instantanea comun y sus tests), 2499a38 (importar), bb9fcfa (tablero, frescura y doctor; cierra A8-12).
- **A7-06**: c321db0 (init y desinstalar).
- Criterio 1: `git grep -n -e read_dir -e "fn instantanea" -e "fn recorrer"` sobre los seis ficheros, vacio.
- Tests nuevos en `soporte.rs`: `instantanea_ve_carpetas_vacias_y_subcarpetas` e `instantanea_ignora_los_nombres_excluidos`, en rojo con cuatro mutantes (sin carpetas, sin recursion, sin exclusion, hash solo por tamano).
- Fotos mas fuertes (frescura recursiva; importar, tablero y frescura ven carpetas): ningun test en rojo, sin hallazgo de produccion.
- Recuento `cargo test -p tc2 --no-fail-fast` (target propio): antes 777 (cierre de TASK-148c); despues 48 binarios, 779 ok, 0 fallos, 13 ignorados (777 + 2).
- `cargo xtask verificar` lo ejecuta `finish`.

## Divergencias

- frescura leia los ficheros con `.ok()`; ahora un fichero ilegible hace fallar el test (la forma mas fuerte fijada en el plan).

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-24T05:55:38Z | plan | automatico | persona | 0.17.1 |
| 2026-11-24T05:58:24Z | approve | automatico | persona | 0.17.1 |
| 2026-11-24T05:58:25Z | start | automatico | persona | 0.17.1 |
| 2026-11-24T06:36:52Z | review | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-24T05:58:24Z | diseno | agente:aa5704f48c80de0c1 | 16 | 56118 | 250824 | 9895 | 20 | 316853 | - | - |
| 2026-11-24T06:36:52Z | implementacion | sesion:f5474097-7166-4a1c-ae42-79e139302dce | 14 | 10709 | 2892188 | 3563 | 0 | 2906474 | - | - |
| 2026-11-24T06:36:52Z | implementacion | agente:a96e8b1a8f7a90623 | 70 | 123050 | 3329500 | 13090 | 2 | 3465710 | - | - |

## Autorrevision

- rust-wasm-reviewer — bb9fcfa
- 4D — bb9fcfa: helper buscado, criterios citados, 2 tests mutados
