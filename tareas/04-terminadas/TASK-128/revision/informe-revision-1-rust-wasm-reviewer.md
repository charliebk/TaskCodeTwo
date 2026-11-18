# Informe de revision — TASK-128 (ronda 1)

- Commit revisado: 6fdb49100d3066da1c76a255b171cb8ed232d052
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/toma/mod.rs:234 | implementacion |

### MEN-1: la rama `Anterior::NoEscrito` de `start` (toma) no tiene ningun test

- **Donde**: crates/tc2/src/toma/mod.rs:234-236 (`Ok(Anterior::NoEscrito) => progreso=sin-anterior`).
- **Que pasa**: la rama que anade `progreso=sin-anterior` a la linea de `start` cuando la toma escribe el progreso por si misma no la ejerce ningun test. En `lib.rs` (`con_progreso`, rama `EscritoSinAnterior`) si hay test unitario, pero el `start` publicado va por `hecho` y no pasa por esa rama.
- **Reproduccion**: en el clon, cambiar esa rama por `Ok(Anterior::NoEscrito) if false => {` y correr `cargo test -p tc2 --lib toma` (10 pasan) y `cargo test -p tc2 --test progreso --test toma -- --test-threads=4` (24 pasan). La mutacion sobrevive.
- **Impacto**: una regresion en esa rama (perder la clave en `start`) no se detectaria. Codigo correcto leyendo; solo falta la red.
- **Sugerencia**: un test de `toma` con la ruta de `.anterior.json` ocupada por una carpeta (como en `escribir_dice_que_paso_con_el_anterior`) que compruebe `progreso=sin-anterior` y stderr vacio.

## Alcance
- Ficheros revisados: los 9 del dominio Rust del diff (catalogo/toma.rs, lib.rs, progreso/fichero.rs, progreso/mod.rs, solape.rs, toma/mod.rs y tests liberar/progreso/solape).
- Contexto adicional pedido: ninguno (`tarea/modelo.rs` leido, ver abajo).
- Limite JS-WASM: sin cambios; el diff no toca `#[wasm_bindgen]`.

## Reproduccion
- Clon: `%LOCALAPPDATA%/pruebas-claude/TaskCodeTwo/task-128-r1-rust`, checkout 6fdb491. `git diff --stat 78f965e 6fdb491 -- crates docs` vacio: el codigo es el de la suite verde.
- Suite: no repetida. `cargo xtask verificar --perfil rapida` en verde (exit=0, "xtask: verificacion Rapida en verde") sobre 78f965e; log `%LOCALAPPDATA%/pruebas-claude/TaskCodeTwo/verif-128.log`.
- Tests concretos en verde: `--lib solape` (10), `--lib progreso::fichero` (4), `--lib el_progreso_no_cambia` (1), `--test solape` y `--test progreso c3_transiciones` con `--test-threads=4` (15 en total).
- Formato y lints: `cargo fmt --check` exit 0; `cargo clippy -p tc2 --all-targets` sin avisos.
- Mutacion (cada una revertida):
  1. Quitar el filtro de misma persona en `otras`: rojo (`--lib solape`, `otras_quita_la_propia...`).
  2. Re-serializar en vez de copiar bytes en `escribir`: rojo (`el_anterior_es_el_fichero_que_habia_byte_a_byte`).
  3. Desactivar la rama `EscritoSinAnterior` en `con_progreso`: rojo (`el_progreso_no_cambia_la_transicion`).
  4. Desactivar `Anterior::NoEscrito` en toma: sigue verde (MEN-1).
- Medidas: no aplican.

## Revisado sin hallazgos
- `escribir`: `SinPrevio`/`Rotado`/`NoEscrito` correctos; el nuevo se escribe siempre y el error de escribirlo se propaga. El `exists()` previo es una carrera teorica inocua.
- `con_progreso`: con `hecho` (start publicado) devuelve `Escrito`, asi que no duplica la clave `progreso`.
- `otras`: las heredadas y sin asignar cuentan; compara por id de cuenta.
- Catalogo: claves movidas a `toma`, test que las busca en `TODOS`.
- Tests E2E `rotado`: aceptan rotado o `sin-anterior` (excluyentes), coherente con la causa explicada.

## Lecturas fuera del contexto permitido
- crates/tc2/src/tarea/modelo.rs (`Asignado::misma_persona`): para comprobar que compara por id de cuenta.
- crates/tc2/src/contrato.rs (`Informe::con`): para ver si repetir una clave la duplica.
