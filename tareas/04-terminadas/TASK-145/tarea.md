---
id: TASK-145
titulo: "Auditoria 4D: A7 config, init, desinstalar, i18n, confianza y ciclo (8 IMPORTANTE)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: fix/task-145-auditoria-4d-a7-config-init-desinstalar
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 5414571
tokens_implementacion: 37979959
tokens_revision: 5572899
creado: 2026-11-23
actualizado: 2026-11-25
dependencias: []
---
## Objetivo

Corregir los 8 casos IMPORTANTE del grupo A7 de `docs/auditoria-4d.md` (commit auditado d82ea9d): 4 Duplicaciones (una `divergente`), 1 Desviación de F2-8 que cambia la salida y 3 Disfraces (uno en código y dos de test con mutante confirmado). A7-02 y A7-06 van en las tareas de abstracción común (A8-04 y A8-12). No se pide el MENOR (A7-08). No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.

## Criterios de aceptacion
- [x] A7-04 (`divergente`, va primero): los ayudantes de test `md`, `estado`, `motivo`, `texto` y `fila` de `ciclo/liberar.rs:423-467` y `ciclo/operacion.rs:286-325` quedan en un solo módulo de prueba de `ciclo`, con los parámetros que hoy separan `md` y `fila`.
- [x] A7-01 y A7-03: el esqueleto «plan como dato» de `init/mod.rs` y `desinstalar/mod.rs` (rangos de la ficha A7-01) se comparte, y `estado` y `confiar` de `confianza/mod.rs:198-242` ~ `:246-275` comparten la lectura, el cierre, la huella y la `Evaluacion`.
- [x] A7-05: los dos tests unitarios de `ciclo/mod.rs:505-527` ~ `:570-590` montan el escenario git con un solo ayudante.
- [x] A7-07 y A7-10: ningún texto se renderiza en español en el sitio para meterlo en `{error}` (`init/aplicar.rs:25-27`, `:82-84`, `init/mod.rs:470`, `desinstalar/proyecto.rs:373-376`, `desinstalar/mod.rs:542`). `ningun_literal_fuera_del_catalogo` (`i18n/guarda.rs:17-23`, `:53-83`) caza la forma `Err("...".to_owned())`. Con el mutante de la ficha A7-10 aplicado, `cargo test -p tc2 --lib i18n::guarda::ningun_literal_fuera_del_catalogo` sale en rojo, y con `TC2_IDIOMA=en` el diagnóstico sale entero en inglés.
- [x] A7-09: `config::del_arbol` (`config/mod.rs:534-543`) da `sin-config` solo con `ErrorKind::NotFound`. Un `.taskcode/config.yml` en UTF-16 o sin permiso de lectura da otro motivo con la causa, y no «crea la configuración con tc2 init». Lo prueba un test con un fichero UTF-16LE con BOM.
- [x] A7-11: un test ejercita el rechazo `ramas-sin-remoto` con `develop` sin publicar (`init/inventario.rs:347-352`, criterio de TASK-118), y el docstring de `aplicar_puertas_exige_personas_ramas_y_persona` (`tests/init.rs:1693-1754`) dice lo que provoca. Con el mutante de la ficha aplicado, `cargo test -p tc2 --test init` sale en rojo.
- [x] `cargo xtask verificar` en verde, y el `## Resultado` lista los 8 IDs con su commit.

## Resultado

Corregidos los 8 IMPORTANTE del grupo A7 (A7-02 va en A8-04/TASK-150; A7-06 lo cerro TASK-148; A7-08 es MENOR).

| Ficha | Commit |
|---|---|
| A7-04 (`divergente`) | d04891a |
| A7-05 | 09671c6 |
| A7-03 | 6a155a9 |
| A7-01 | df58237 |
| A7-07 | d85a18a |
| A7-10 (D4) | be71fe0 |
| A7-09 (D4) | ff94d16 |
| A7-11 (D4) | f2b311e |

- Cambios de comportamiento aceptados: el fixture `descartada` queda con el motivo `"el de entonces"`; el stderr de init y desinstalar con `TC2_IDIOMA=en` sale entero en ingles (stdout, motivos y codigos no cambian); con un `.taskcode/config.yml` ilegible, `verificar` y `confiar` dan `<orden> error ruta=.taskcode/config.yml` (codigo 1) en vez de `rechazada motivo=sin-config` (3), y `--hook` sigue saliendo 0 sin ejecutar nada (comprobado en `tests/verificar.rs::config_ilegible_da_error_y_el_hook_no_bloquea`).
- Rojos: A7-10, la guarda ampliada contra el codigo previo a A7-07 (`Err` con prosa en `desinstalar/mod.rs` e `init/aplicar.rs`) y con el mutante de la ficha (EXTERNO en `desinstalar/mod.rs`); A7-09, `sin-config` frente a `config error` (unitario) y codigo 3 frente a 1 (integracion); A7-11, verde con el codigo actual y rojo con `if false && !sin_remoto.is_empty()`.
- Tests nuevos: `ciclo::prueba::tests::md_de_partida`, `plan_dato::tests::{corta_en_el_primer_fallo, rechazar_aplaza_las_aplicables}`, `desinstalar::tests::cambio_desde_el_plan_en_ingles`, `init::aplicar::tests::{escribir_nuevo_sobre_uno_existente_en_ingles, informe_como_mensaje}`, `config::tests::del_arbol_distingue_no_existe_de_no_se_lee`, `tests/verificar.rs::config_ilegible_da_error_y_el_hook_no_bloquea`; caso nuevo en `tests/init.rs::aplicar_puertas_exige_personas_ramas_y_persona`; guarda de i18n ampliada (`LITERALES_ERR` cerrada con `actualizar_cli.rs` y `verificacion/informe.rs`, decision 2).
- Recuento `cargo test -p tc2 --no-fail-fast` (target propio): antes 800 (cierre de TASK-144); despues 48 binarios, 808 ok, 0 fallos, 13 ignorados (800 + 8).
- `cargo xtask verificar` lo ejecuta `finish`.

## Divergencias

- El ayudante de errores externos es `plan_dato::externo(e: impl Display)` (cubre `io::Error`, la linea de stderr de git y la ruta no UTF-8), no `de_io(&io::Error)`.
- La entrada nueva del catalogo es `init::EFECTO_PUERTAS_SIN_PLATAFORMA` (`PUERTAS_SIN_PLATAFORMA` ya existia con otro texto).
- Cambio visible en espanol: el fallo de `desinstalar` al reescribir un fichero cambiado pasa de «...: cambio desde el plan» a «...: {ruta} cambio despues de hacer el plan» (usa `CAMBIO_DESDE_EL_PLAN` del catalogo; la ruta sale dos veces).
- El commit de A7-07 lleva el cambio de `EXTERNOS` en `guarda.rs` para que cada commit pase sus tests.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-25T06:52:16Z | plan | automatico | persona | 0.17.1 |
| 2026-11-25T07:04:06Z | approve | automatico | persona | 0.17.1 |
| 2026-11-25T07:04:07Z | start | automatico | persona | 0.17.1 |
| 2026-11-25T08:59:20Z | review | automatico | persona | 0.17.1 |
| 2026-11-25T11:31:35Z | finish | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-25T07:04:05Z | diseno | agente:a7ef2497f364e6e35 | 118 | 142075 | 5238702 | 33676 | 77 | 5414571 | - | - |
| 2026-11-25T08:59:20Z | implementacion | sesion:f5474097-7166-4a1c-ae42-79e139302dce | 32 | 17929 | 11635827 | 6380 | 0 | 11660168 | - | - |
| 2026-11-25T08:59:20Z | implementacion | agente:ae6af017a6f582157 | 272 | 559467 | 25678498 | 81554 | 15 | 26319791 | - | - |
| 2026-11-25T09:38:30Z | revision | agente:a0b7b704852f76bd0 | 72 | 179299 | 5367402 | 26126 | 3 | 5572899 | 1 | rust-wasm-reviewer |

## Autorrevision

- rust-wasm-reviewer — f2b311e
- 4D — f2b311e: helper buscado, criterios citados, 3 mutantes vistos en rojo

## Verificacion ligera

- 2026-11-25T09:38:35Z: ronda 1; commit revisado fce8bd62afb7e421e03fef98b93e6c8ff949cadf; commit verificado 131dff3c8d2b33ccb0dbc4393174506f7fa86b65; ficheros: crates/tc2/src/i18n/guarda.rs, crates/tc2/src/init/aplicar.rs; corregidos: MEN-1, MEN-2; pendientes: MEN-3

## Duracion

- Estimada: 210 min
- Real (de la rama abierta al cierre con merge, sin pausas): 267 min
- Paso de la hora: 267 min reales frente a 210 estimados
- Cierre (finish): 2826 s, de 2026-11-25T10:44:29Z a 2026-11-25T11:31:35Z
