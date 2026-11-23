# Informe de revision — TASK-148a (ronda 1)

- Commit revisado: f7970589d6f2080942bd030da79babc481b8a140
- Revisor: general-purpose (sonnet) con rust-wasm-reviewer
- Veredicto: aprobada con correcciones

Veredicto propuesto: **aprobada** (un solo hallazgo MENOR, sin CRITICO ni IMPORTANTE).

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/tests/actualizar_cli.rs:265 | especificacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### MEN-1 — Quedan lanzamientos de `tc2` fuera del lanzador unico

`actualizar_cli.rs` lanza copias del binario (`instalar_vx` copia `comun::BINARIO` y los tests ejecutan la copia, no el original) con `Command::new(exe)` directo: `lanzar_con` (linea 265) y los `Command::new(&exe).arg("--version")` de las lineas 418, 431, 470 y 708. Esos lanzamientos no reciben la base comun (`LC_ALL=C`, `GIT_TERMINAL_PROMPT=0`, ni quitan `TC2_TOKEN_*` / `TC2_APP_*`); `lanzar_con` solo fija `TC2_BASE_URL` y `TC2_IDIOMA` y quita `TC2_GH`, `GH_FALSO*`.

Evidencia: `git grep -n 'Command::new(' -- crates/tc2/tests` (sin `git`) en el clon deja esas cinco apariciones de `tc2` en `actualizar_cli.rs`, ademas de `ssh-keygen` y `current_exe()` en `estado_concurrencia.rs:75`. El criterio 1 de la tarea (`git grep CARGO_BIN_EXE_tc2`) se cumple al pie de la letra porque la ruta pasa por una copia, por eso el origen es `especificacion`: el criterio mide la constante, no el objetivo (un unico lanzador). No he demostrado que un token del entorno cambie hoy el resultado de esos tests (pasan en verde); es una asimetria que reabre el riesgo de A8-02 en el unico fichero que no puede usar `orden_tc2()` tal cual porque cambia el ejecutable. Se arregla con una variante `orden_tc2_de(ruta)` en `comun/lanzador.rs` que `orden_tc2()` reutilice, o con dejar anotado en `## Divergencias` que las copias quedan fuera a proposito. No bloquea.

## Regla 4D

- Duplicacion: cumple. El bloque de entorno repetido en ~26 ficheros pasa a una sola funcion; las 4 copias del test de `http_mock` por los binarios nuevos con `mod comun;` (`soporte`, `cli`, `actualizar_cli`, `plataforma_e2e`) estan justificadas en el `## Resultado` y cuadran con el recuento (763 + 1 + 4 = 768). Resto de lanzamientos sin base: MEN-1.
- Desviacion: cumple. `orden_tc2()` en vez de `tc2(dir)` y `texto` sin crear estan en `## Divergencias` con motivo (choque con el crate `tc2` y codigo muerto); el criterio 2 se comprueba igual sobre `orden_tc2()`. Ninguna API publica de `src` cambia.
- Diluvio: cumple. Un modulo de 31 lineas y una funcion; el test de soporte son 45 lineas.
- Disfraz: cumple. Todo test nuevo falla ante mutantes (ver Mutacion); no hay `Ok(())` provisional ni resultados descartados.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/plataforma/credenciales.rs` (`Rol::variable`, `Rol::variables_app`): para comprobar que el lanzador cubre todas las variables de credenciales del crate (tres tokens y dos pares de App; `Persona` no tiene App). Cubre todo.

## Alcance
- Ficheros revisados: los 29 de la peticion (26 tests migrados, `comun/lanzador.rs`, `comun/mod.rs`, `soporte.rs`); diff leido completo de 6f699e7 a f797058.
- Contexto adicional pedido: solo `credenciales.rs` (arriba).
- Limite JS-WASM: sin cambios; el diff solo toca tests de integracion nativos, ninguna firma `#[wasm_bindgen]`.

## Reproduccion
- Clon: `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-148a-r1`, commit f7970589d6f2080942bd030da79babc481b8a140; `CARGO_TARGET_DIR` propio dentro del clon. Carpeta borrada al terminar.
- Build: `cargo test --locked -p tc2 ...` compila bien (nativo; no aplica `wasm32`: solo tests de integracion).
- Formato y lints: `cargo fmt --check` limpio; `cargo clippy --locked -p tc2 --tests --all-features -- -D warnings` sin avisos.
- Suite: no repetida entera; se acepta la medicion del implementador (47 binarios 762 ok 1 fallo de tiempo antes, 48 binarios 768 ok 0 fallos despues). Muestreo propio con `--locked`: `soporte` 2 ok, `git_binario` 2 ok, `cli` 5 ok, `frescura` 12 ok, `progreso` 12 ok, `verificar` 15 ok / 1 ignorado, `actualizar_cli` 11 ok (nota: en una primera tirada solo con `--test actualizar_cli` fallaron 4 por faltar los ejemplos `tc2_falso`/`gh_falso`, que no se compilan al elegir binarios; tras `cargo build --examples` pasan los 11; es artefacto de la seleccion, no del cambio). Segunda tirada: `doctor` 12 ok, `hallazgos` 8 ok, `inyeccion` 4 ok, `plataforma_e2e` 1 ok / 5 ignorados, `revisoras` 5 ok, `siguiente` 6 ok. Observacion sin juicio: `doctor` tardo 577 s (el resto, segundos); su lanzador solo cambio de forma (mismas variables que antes), no he medido la linea base de tiempo antes del cambio, asi que no lo reporto como hallazgo.
- Criterio 1: `git grep -n 'CARGO_BIN_EXE_tc2' -- crates` devuelve solo `crates/tc2/tests/comun/lanzador.rs:15`. Cumple.
- Casos construidos / revision de entorno por fichero: leidos los 26 diffs. En todos la base va primero y lo propio del test despues, asi que en `Command` gana lo posterior. Se conservan `TC2_IDIOMA` variable (`cli`, `frescura`, `siguiente`, `tablero`), `GIT_CEILING_DIRECTORIES`, `CLAUDE_CONFIG_DIR`, `PATH`, `GIT_TRACE`, `GIT_DIR`, `LOCALAPPDATA`/`XDG_CACHE_HOME`, `stdin`/`current_dir`. `git_binario` sigue quitando `TC2_IDIOMA`, `LC_ALL`, `LC_MESSAGES` y `LANG` despues de la base, asi que su prueba de locale sigue ejerciendo el caso sin idioma. Ninguna variable significativa perdida: las que se quitaban (`TC2_TOKEN_*`) las cubre la base, que ademas quita las `TC2_APP_*`.
- Mutacion sobre `comun/lanzador.rs` (ejecutando `cargo test -p tc2 --test soporte el_lanzador`); todos se ponen en rojo:
  - M1 `LC_ALL` = `es` en vez de `C`: rojo.
  - M2 quitar `GIT_TERMINAL_PROMPT`: rojo.
  - M3 `TC2_IDIOMA` = `en`: rojo.
  - M4 no quitar `Rol::variable()` (tokens): rojo.
  - M5 no quitar la clave de la App (`env_remove(clave)`): rojo.
  - M6 bucle sin `Rol::Persona`: rojo.
  - M7 bucle sin `Rol::Revisor`: rojo.
  - M8 quitar `TC2_IDIOMA`: rojo.
  Fichero restaurado tras cada mutante.
- Medidas: no aplican.

## Revisado sin hallazgos
- `comun/lanzador.rs` y `comun/mod.rs`: reexport `#[allow(unused_imports)]` con motivo; `BINARIO` como `pub const` de `env!`; sin `unwrap`.
- `soporte.rs`: el test compara `get_envs()` con `Some(&None)` (quitada) y `Some(&Some(valor))` (fijada), y el programa; distingue de verdad "quitada" de "no tocada".
- Duplicacion del criterio 2: coherente con la divergencia documentada de nombre.
- Imports sobrantes (`std::process::Command`) retirados donde ya no se usan; clippy limpio lo confirma.
