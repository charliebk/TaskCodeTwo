# Informe de revision — TASK-146 (ronda 1)

- Commit revisado: 9e6095be101bb1f64a2a66220a4221bfdb147728
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | corregido | tareas/03-en-revision/TASK-146/tarea.md (`## Resultado`) | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

**MEN-1 (MENOR, documental).** El Resultado cita el mutante de A11-02 como "`:79` con otro texto". Esa linea es `"npm" => "npm uninstall -g taskcodetwo"` en `orden_del_gestor`. La misma cadena esta tambien en el test de `instalacion.rs` (`let npm = ...`). Quien reproduzca el mutante con un reemplazo global, como `sed s/texto/otro/`, cambia produccion y test a la vez y obtiene VERDE. Me paso en mi primer intento. Con el cambio solo en la linea 79, el test se pone rojo (ver Reproduccion). No es un defecto del codigo ni del test. Basta precisar en el Resultado que el mutante es solo la linea de produccion. No bloquea. (Corregido por quien orquesta: el Resultado precisa que el mutante es solo la linea 79 de produccion.)

Sin hallazgos CRITICO ni IMPORTANTE. Los 7 tests nuevos o reforzados fallan si se revierte la proteccion que cubren.

## Alcance

- Ficheros revisados: `crates/tc2/examples/tc2_falso.rs`, `crates/tc2/src/actualizar_cli.rs`, `crates/tc2/src/contrato.rs`, `crates/tc2/src/dato.rs`, `crates/tc2/src/instalacion.rs`, `crates/tc2/tests/actualizar_cli.rs`, `xtask/src/workflows.rs`.
- Contexto adicional pedido: ninguno. Solo se leyo ademas `comprobar_nuevo` y los tramos de produccion que cita la tarea, que son los lugares donde se aplicaron los mutantes.
- Limite JS-WASM: sin cambios. El diff no toca ningun `#[wasm_bindgen]`, ni firma ni tipo exportado. Solo cambian tests, el doble `tc2_falso` y un ayudante de test de xtask. Produccion no se modifica.

## Reproduccion

- Clon: `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-146-r1`, rama `fix/task-146-auditoria-4d-a8-transversal-actualizar-c`, HEAD 9115181. Su unica diferencia con el commit revisado es el commit de la peticion, que no toca codigo. `CARGO_TARGET_DIR` apuntaba a la carpeta `target` del clon.
- Build: `cargo build -p tc2 --examples` correcto. Nativo solamente; no se compilo para wasm32 porque el diff no toca nada exportado.
- Formato y lints: no se ejecutaron `cargo fmt --check` ni `cargo clippy`. La puerta de calidad la corre `cargo xtask verificar` en `finish`.
- Suite: no se ejecuto la suite completa del workspace, por la carga de la maquina. El autor declara 852 ok, 0 fallidos y 13 ignorados, y no se reprodujo. Se ejecutaron solo los modulos tocados, todos en verde:
  - `cargo test -p tc2 --lib -- instalacion::tests dato::tests contrato::tests ejecutar_con_limite`: 12 pasan, 0 fallan.
  - `cargo test -p tc2 --test actualizar_cli`: 12 pasan, 0 fallan. Incluye `el_binario_nuevo_que_no_responde_bien_es_rechazado_y_no_toca_nada`.
  - `cargo test -p xtask el_workflow_del_revisor`: `el_workflow_del_revisor_pasa_el_lint` pasa.
- Casos construidos: el test de A9-02 tiene tres escenarios (no arranca, otra version 9.9.8 frente a 9.9.9, otro sha256 por `TC2_FALSO_SHA256`). Los tres exigen codigo 3, `binario-no-arranca`, sha del instalado sin cambios y sin restos. `lanzar_con` retira `TC2_FALSO_SHA256` del entorno heredado, asi que el entorno de quien corre los tests no la contamina.
- Mutacion: mutantes sobre el clon, revertidos tras cada uno con `git checkout`:

| Mutante | Test ejecutado | Resultado |
|---|---|---|
| `dato.rs`: quitar `'*'` de la lista de escape | `dato::tests` | ROJO |
| `contrato.rs`: quitar la rama `c if (c as u32) < 0x20` (la ficha decia `:336`; esta en `:345`) | `contrato::tests` | ROJO |
| `instalacion.rs`: quitar la rama `opt/homebrew` | `instalacion::tests` | ROJO |
| `instalacion.rs`: quitar `seguidas("winget","links")` | `instalacion::tests` | ROJO |
| `instalacion.rs`: `dirs_de_instalacion` sin deduplicar (`if true`) | `instalacion::tests` | ROJO |
| `instalacion.rs:79`: `"npm uninstall taskcodetwo"` solo en produccion | `instalacion::tests` | ROJO (1 fallido) |
| `actualizar_cli.rs`: `comprobar_nuevo` sustituido por `Ok(())` | `--test actualizar_cli` | ROJO |
| `actualizar_cli.rs`: `if false` en la comparacion del sha256 | `--test actualizar_cli` | ROJO |

  El mutante de `:79` dio VERDE con el reemplazo global (artefacto del `sed`, ver MEN-1) y ROJO acotado a la linea. Del A11-03 solo se probo `*`, no `_`, `~` ni `&`. Del A9-02 se probo el mutante del sha256, no el de la version ni el del target. No se comprobo si el test de A9-02 los atrapa por separado.
- Limpieza: clon y script temporal borrados. No quedan procesos cargo en marcha, ni cambios en el repo original.

## Revisado sin hallazgos

- A11-03 (`dato.rs`): la expectativa `r"\&\#x202E; \*\*n\*\* \_x\_ \~\~t\~\~"` es literal e independiente de la implementacion, y cubre los cuatro caracteres.
- A11-04 (`contrato.rs`): exige `\u001b` y `\u0007` exactos en el JSON, y que `serde_json` lo parsee como validacion independiente.
- A8-15 y A11-02 (`instalacion.rs`): la tabla lleva la orden esperada por fila y compara con `assert_eq!` el texto exacto, no solo "no vacio". Hay filas que ejercitan solo `/opt/homebrew/bin` y solo `WinGet\Links`. El caso de `TC2_INSTALL_DIR` igual a la carpeta por defecto exige un solo elemento.
- A8-13 (`actualizar_cli.rs`): el ayudante `shell(windows, unix)` sustituye los tres bloques duplicados y conserva `TC2_PRUEBA=1`. Ambos tests de `ejecutar_con_limite` siguen en verde.
- A9-03 (`xtask/src/workflows.rs`): `runs_del_job` sustituye el cierre `runs_de`. Como declara el Resultado, comprueba el texto unido de los `run` del job en vez de cada uno, igual o mas estricto tras `normalizar`. Esa divergencia es aceptable.
- `tc2_falso.rs`: `TC2_FALSO_SHA256` solo cambia el sha que dice el doble. No altera `--version` ni el target, y solo la pone el test.
- Panics y `unsafe`: el diff de produccion es nulo. No hay `unwrap` ni `unsafe` nuevos fuera de tests.

## Regla 4D

- Duplicacion: cumple. Se elimino la duplicacion del shell de tests (A8-13) y del cierre `runs_de` (A9-03). No anade definiciones duplicadas.
- Desviacion: cumple. No toca firmas publicas ni alcance fuera de los 7 IDs. Las divergencias del Resultado estan declaradas y justificadas.
- Diluvio: cumple. Un ayudante pequeno y una variable de entorno de test. No hay abstracciones sobrantes.
- Disfraz: cumple. Los tests nuevos tienen mutante rojo comprobado (ver Mutacion). Solo queda MEN-1 como precision documental del Resultado.

## Lecturas fuera del contexto permitido

ninguna. Las lecturas de `comprobar_nuevo` y de los tramos de produccion citados por la tarea estan en ficheros de la lista permitida.
