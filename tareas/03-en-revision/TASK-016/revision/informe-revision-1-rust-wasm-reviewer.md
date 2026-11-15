# Informe de revision — TASK-016 (ronda 1)

- Commit revisado: 9d72eae2729bfc231921d0eb9a25373365dc10b3
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada con correcciones

Veredicto propuesto por el revisor: aprobada-con-correcciones (solo hallazgos MENOR; ninguno bloquea).

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/main.rs:31 | implementacion |
| MEN-2 | MENOR | abierto | xtask/src/main.rs:425 | implementacion |
| MEN-3 | MENOR | abierto | xtask/src/main.rs:429 | implementacion |
| MEN-4 | MENOR | abierto | .cargo/config.toml:7 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### MEN-1 — `tc2` sale con 0 aunque no haya podido escribir su salida

`main.rs` descarta con `let _ =` el resultado de `write_all` a stdout y stderr y devuelve siempre `salida.codigo`.
Reproduccion: `tc2.exe --version >&-` (stdout cerrado, desde bash) termina con `exit=0` sin escribir nada ni avisar.
Un script que lea la version con la salida rota recibe exito sin dato. Hoy el impacto es minimo (solo `--version`);
conviene corregirlo antes de que las ordenes de F2-2 en adelante usen este `main` como plantilla: devolver un codigo
distinto de 0 (p. ej. 1) si falla la escritura de stdout.

### MEN-2 — Ningun test protege la comparacion de hashes de `reproducible_en`

Los tests de xtask solo cubren `leer_perfil` y sha256. Mutante: cambiar `[a, b] if a == b =>` por `[_, _] =>` en
`xtask/src/main.rs` deja `cargo xtask verificar --perfil rapida` en verde, y con `completa` seguiria verde
aunque ademas se quite `/Brepro` (la comprobacion de fondo quedaria anulada sin que nada se ponga rojo). La mutacion
de quitar `/Brepro` si se detecta (ver Reproduccion), pero la logica de comparacion no tiene test propio.
Sugerencia: extraer la comparacion a una funcion pura (`comparar(&[String]) -> Result<(), String>`) con un test de
dos hashes iguales y dos distintos.

### MEN-3 — Si el build no es reproducible, los dos binarios se borran antes de poder inspeccionarlos

`reproducible()` hace `remove_dir_all(&base)` tanto en exito como en fallo, y el mensaje de error solo lista los
hashes. En el rojo que justamente busca detectar esta tarea (p. ej. el mutante sin `/Brepro`: hashes `cc74f50c…` y
`e1bf0e08…`, mismo tamano 133120 bytes) no queda nada que comparar con `fc`/diffoscope para saber que difiere.
Sugerencia: en fallo, conservar los directorios e imprimir su ruta (o subir los binarios como artefacto de CI).

### MEN-4 — El plan pedia `--remap-path-prefix` y no esta; la ausencia no queda anotada

El plan-final (enfoque, punto 2) lista `/Brepro`, `--remap-path-prefix` y el alias en `.cargo/config.toml`; el
fichero entregado solo trae `/Brepro` y el alias. Empiricamente no hace falta hoy en Windows: el binario construido
en tres rutas distintas (dos de la prueba y el clon, de longitudes distintas) da el mismo sha256
`e61a9ab4…1933`, coincide con `Get-FileHash` independiente, y el `.exe` no contiene la ruta del checkout, ni
`crates\tc2`, ni rutas de cargo. Pero un mutante que incrusta `env!("CARGO_MANIFEST_DIR")` en el binario si lo
detecta la comprobacion (rojo, hashes distintos), asi que la red existe. Lo que falta es dejar escrito en el
`## Resultado` de la tarea que se omitio a proposito (o anadirlo), porque si en el futuro se incrusta una ruta el
fallo aparecera sin que el `config.toml` diga por que no se mitigo.

## Alcance
- Ficheros revisados: `.cargo/config.toml`, `Cargo.toml`, `crates/tc2/Cargo.toml`, `crates/tc2/src/lib.rs`, `crates/tc2/src/main.rs`, `crates/tc2/tests/cli.rs`, `rust-toolchain.toml`, `rustfmt.toml`, `xtask/Cargo.toml`, `xtask/src/main.rs`, `xtask/src/sha256.rs`; mas `tarea.md` y `planificacion/plan-final.md`.
- Contexto adicional pedido: ninguno (para ver el job `codigo` se leyeron `.gitignore`, `.gitattributes` y `.github/workflows/ci.yml` del clon; ver "Lecturas fuera del contexto permitido").
- Limite JS-WASM: no aplica; no hay `wasm_bindgen`, ni objetivo `wasm32`, ni llamador JS/TS en el diff.

## Reproduccion
- Clon: `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-016-r1-rust\repo`, commit 9d72eae (rama de la tarea). Borrado al terminar; sin commits propios ni nada que conservar.
- Build: Windows x86_64-pc-windows-msvc, rustc 1.100.0. `cargo build --release --locked -p tc2` en verde. No hay objetivo wasm32 en el proyecto.
- Formato y lints: `cargo fmt --all --check` y `cargo clippy --workspace --all-targets --locked -- -D warnings` en verde (dentro de xtask).
- Suite: `cargo xtask verificar --perfil completa` en verde, una sola vez, 13 s en caliente: tests de tc2 (2 unitarios + 2 de integracion), xtask (2) y comprobacion de reproducibilidad (mismo sha256 `e61a9ab4…1933`, 133120 bytes, rutas `a` y `copia-b`). La peticion no traia verificacion de la ronda.
- Casos construidos: tercer build en la ruta del clon, hash igual al de `Get-FileHash`; busqueda de rutas en el `.exe`; `tc2 --version` con stdout cerrado (MEN-1); `tc2 --help` da codigo 2 «orden desconocida» (aceptable: el enunciado solo pide `--version`).
- Mutacion:
  - Quitar `/Brepro` de `.cargo/config.toml`: rojo, «build NO reproducible» (hashes distintos).
  - Incrustar `CARGO_MANIFEST_DIR` en el binario: rojo, hashes distintos (un `const` sin uso no basta, el linker lo elimina; hace falta usarlo).
  - `RUSTFLAGS` definido: rojo inmediato con mensaje claro.
  - `CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS` definido: cargo lo fusiona con el de config, `/Brepro` se mantiene y sale verde y reproducible; no es una via de escape.
  - Anular la comparacion `a == b` de `reproducible_en`: sigue verde (MEN-2).
- Medidas: build release 3,4 s por ruta; binario 133120 bytes.

## Revisado sin hallazgos
- `sha256.rs`: los tres vectores de FIPS 180 (incluido el de dos bloques) pasan y el hash coincide con `Get-FileHash` sobre el `.exe` real; padding y `wrapping_*` correctos.
- `lib.rs` / `tests/cli.rs`: `ejecutar` es pura y sin panics; no hay `unwrap`/`expect`/`panic!`/indexado sobre entrada; argumentos no UTF-8 se tratan con codigo 2 en `main.rs`. Los tests ejercen version corta y larga, sin orden y orden desconocida.
- `Cargo.toml` y perfiles: `unsafe_code = "forbid"`, lints de clippy, perfil release de F1-14 (`panic = "abort"` solo en release, no afecta a los tests), `Cargo.lock` consumido con `--locked`.
- `rust-toolchain.toml` (1.100.0) coherente con `rust-version = "1.100"` y edicion 2024; `rustfmt.toml` coherente.
- Copia del arbol (`ficheros_del_arbol` / `copiar`): usa `git ls-files -z`, respeta ignorados y salta borrados; el directorio temporal por `pid` se limpia.

## Lecturas fuera del contexto permitido

- `.gitignore`, `.gitattributes`, `.github/workflows/ci.yml` (clon): para comprobar que `/target/` se ignora, los finales de linea y que el job `codigo` de Windows usa el perfil `completa`; no se encontraron hallazgos en ellos.
