# Informe de revision — TASK-025 (ronda 1)

- Commit revisado: e531238f359fa059a30904b61c3017ec947a5318
- Revisor: rust-wasm-reviewer (general-purpose, sonnet)
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | xtask/src/dist.rs:77 | implementacion |
| MEN-2 | MENOR | abierto | xtask/src/main.rs:425 | implementacion |
| MEN-3 | MENOR | abierto | xtask/src/dist.rs:114 | implementacion |
| MEN-4 | MENOR | abierto | xtask/src/main.rs:462 | implementacion |

Sin hallazgos CRITICO ni IMPORTANTE.

### MEN-1 — Los triples de win-arm64, darwin-x64 y darwin-arm64 no estan fijados por ningun test

El test `seis_targets_de_p12_con_exe_solo_en_windows` fija los alias y comprueba `de_triple(t.triple) == Some(t)`. Eso solo prueba que la tabla es coherente consigo misma; de los triples solo fija que los de Linux acaban en `-musl`.

- Mutacion `aarch64-pc-windows-msvc` → `aarch64-pc-windows-gnu`: los 7 tests de xtask siguen en verde.
- Mutacion `x86_64-apple-darwin` → `x86_64-apple-darwinX`: los 7 tests siguen en verde.
- Duplicar un triple (darwin-arm64 con el de darwin-x64): ese mutante si muere, porque `de_triple` devuelve el primero.

Un triple mal escrito solo se veria en la CI real, que ademas lleva sus propios triples en la matriz de `release.yml`. Correccion sugerida: aseverar los 6 triples literales, o compararlos con los de la matriz del workflow.

### MEN-2 — El arreglo de `CARGO_TARGET_DIR` relativo (7527334) no tiene test

La resolucion `raiz.join(d)` esta inline en `empaquetar` y ningun test la ejerce: quitar el `join` no pone nada en rojo.

- Reproduccion a mano en win-x64: `CARGO_TARGET_DIR=relt cargo xtask empaquetar ...` desde un subdirectorio. El binario se encuentra en `<raiz>/relt/...` y se copia a `out/`; el `.sha256` coincide con `sha256sum` y `tc2 --version` escribe `tc2 0.0.0`. El comportamiento actual es correcto.
- Correccion sugerida: una funcion pura `dir_target(raiz, var)` con tests para ruta relativa y absoluta.
- Limitacion: no se respeta `build.target-dir` de `.cargo/config.toml`; hoy el repo no lo usa.

### MEN-3 — Bordes de validacion de `sumas` y de `es_macos` sin cobertura

Mutantes que sobreviven con los 7 tests en verde:

- `es_macos()` → `false`. Es lo que activa `codesign` y ningun test lo toca.
- Aceptar como hex cualquier minuscula `a..z`.
- Aceptar `fichero.is_empty()`.

Los separadores `'\\'` y `'\r'` tampoco estan probados (el test solo usa `/` y `\n`); esto es por lectura, porque su mutante no aplico. Correccion sugerida: probar un hash con `g`, un nombre vacio y nombres con `\` y `\r`, y aseverar `es_macos`.

### MEN-4 — `leer_orden` acepta mas de lo que fijan los tests, y `escribir_sumas` no tiene test de E/S

- Sobrevive el mutante `["sumas", dir, ..]`: ningun test comprueba que `sumas a b` se rechaza.
- El filtrado de `SHA256SUMS` y `*.sha256` y el error ante un subdirectorio solo se ejercen en la CI. A mano se comportan bien.
- `empaquetar` exige `--target` antes de `--salida`; al reves responde «argumentos no validos» (codigo 2). Es una incomodidad de uso, no un fallo.

## Alcance

- Ficheros revisados: `xtask/src/dist.rs`, `xtask/src/main.rs` y `crates/tc2/tests/git_binario.rs`. No hay limite JS-WASM: el diff no tiene `#[wasm_bindgen]`.
- `release.yml` no se reviso aqui; el humo y la attestation se dan por validos por los runs 43784921536 y 43785561970.

## Reproduccion

- **Clones** en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-025-r1-rust`, en e531238, borrados al terminar.
- **Suite**:
  - `cargo xtask verificar --perfil rapida` salio en ROJO por 2 tests de `plataforma_gitlab.rs` (`sinred`, mas de 60 s), mientras corrian a la vez los mutantes.
  - Repetido en solitario: 3 de 3 en 1,22 s.
  - Se anota como fragilidad bajo carga, ajena al diff (ese fichero no esta en el diff).
  - El resto, en verde, incluido `git_binario`.
- **Binario**:
  - `empaquetar` win-x64 da 485376 bytes, `tc2 --version` = `tc2 0.0.0` y su `.sha256` coincide con `sha256sum`.
  - `comprobar-tag`: `v0.0.0` da estable, `v0.0.0-prueba.1` da prerelease y `v9` falla con codigo 1.
  - `sumas`: con un fichero con espacio en el nombre y un `.sha256` excluido, `sha256sum -c` lo valida; un subdirectorio da error y una carpeta vacia da «no hay ficheros que sumar».
  - `RUSTFLAGS` definido: codigo 1. Target fuera de P12: codigo 2.
- **Mutacion**: 19 mutantes, 17 aplicados. Mueren 11 y sobreviven 6 (los de MEN-1, MEN-3 y MEN-4).

## Revisado sin hallazgos

- `clase_de_tag` y los bordes de `sumas` que ya cubren los tests (orden, duplicados, longitud y lista vacia).
- La guarda de `RUSTFLAGS`, la firma ad hoc con `codesign --verify --strict` y el `.sha256` con el formato de `sha256sum`.
- El diff no tiene `unwrap`, `expect`, `panic!` ni `unsafe`.
- `git_binario.rs` quita las cuatro variables que lee `i18n/mod.rs`, y `Fixture::entorno` no las repone.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/i18n/mod.rs` y `crates/tc2/tests/comun/mod.rs`: variables de idioma.
- `xtask/Cargo.toml`, `.cargo/config.toml`, `Cargo.toml` y `crates/tc2/Cargo.toml`: version del workspace y `/Brepro`.
- `crates/tc2/tests/plataforma_gitlab.rs`: solo la salida del fallo bajo carga.

## Tratamiento de los MENORES (commit e368253, quien implementa)

- **MEN-1**: el test asevera los 6 triples literales y que `es_macos` es verdadero solo en darwin-x64 y darwin-arm64.
- **MEN-2**: `dir_target(raiz, var)` es una funcion pura, con test de ruta relativa, absoluta y sin variable. `build.target-dir` sigue sin mirarse (anotado en su doc).
- **MEN-3**: se prueban un hash de 64 caracteres con `g` y los nombres `""`, `dir\x` y `x\ry`.
- **MEN-4**: se prueba que `sumas a b` y `sumas` se rechazan. No se ha probado `escribir_sumas` con E/S en un test (lo ejercen el ensayo y la reproduccion a mano). El orden fijo de `--target`/`--salida` se mantiene a proposito: lo invoca solo el workflow.
