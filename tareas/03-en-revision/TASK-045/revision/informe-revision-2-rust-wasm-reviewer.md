# Informe de revision — TASK-045 (ronda 2)

- Commit revisado: e141ed154087ea00e2e7fa9536bed2fb42c093f7
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 (ronda 1) | MENOR | corregido (76daad5, junto con MENOR-1) | xtask/src/workflows.rs (el_aislamiento_va_antes_del_checkout_en_el_runner_propio) | implementacion |
| MENOR-1 (nuevo, residual de MEN-1) | MENOR | corregido (76daad5) | xtask/src/workflows.rs (aislamiento_antes_del_checkout); .github/workflows/verificacion.yml:373 y :376 | implementacion |
| MEN-2 (ronda 1) | MENOR | no reverificado (la peticion lo excluye; estado de ronda 1: aceptado) | xtask/src/workflows.rs (sha256sum_solo_por_la_entrada_estandar) | implementacion |

No hay CRITICO ni IMPORTANTE abiertos en esta ronda. Las correcciones de MEN-1
(guardas comprobadas por su expresion completa, y la regla del evento repetida
en `ejecutar` con sus tests) quedan verificadas, salvo el residuo de salida que
recoge MENOR-1.

### MENOR-1 — las guardas del aislamiento pueden salir con `exit 0` y los tests siguen en verde
- Donde: `.github/workflows/verificacion.yml:373` (guarda de `RUNNER_OS`) y `:376` (guarda de `SYSTEM`); la comprobacion de la forma completa esta en `xtask/src/workflows.rs`, funcion `aislamiento_antes_del_checkout` (lista `for exigido in [...]`).
- Que pasa: la lista exige la expresion `if (...) {`, pero no el `exit 2` del cuerpo. Si la guarda queda como `if ($id.User.Value -eq 'S-1-5-18') { Write-Output "..."; exit 0 }`, el test sigue en verde y el paso del runner propio termina con codigo 0: imprime el rechazo y el checkout siguiente corre igual.
- Reproduccion: en el clon `task-045-r2-rust` (commit e141ed1), con `cargo test -p xtask --locked`:
  - M8: en la guarda de SYSTEM, `exit 2` -> `exit 0`. Resultado: `test result: ok. 34 passed; 0 failed`.
  - M9: en la guarda de `RUNNER_OS`, `exit 2` -> `exit 0`. Resultado: `test result: ok. 34 passed; 0 failed`.
  - Comprobacion del comportamiento de PowerShell con el mismo patron (`$ErrorActionPreference = 'Stop'`, `Write-Output` y `exit 0`, lanzado con `-NoProfile -ExecutionPolicy Bypass -Command`): el proceso devuelve `exit=0`. Es decir, el paso queda en verde.
- Impacto: hoy el codigo del workflow es correcto (leido en el clon: las guardas salen con `exit 2`; ningun test lo exige). Lo que falta es la red: una regresion de ese tipo en la guarda de SYSTEM pasaria la suite y dejaria correr el checkout del PR con la cuenta de sistema. Es una degradacion de la proteccion de test, no un fallo de runtime actual.
- Sugerencia: en la lista de `aislamiento_antes_del_checkout`, exigir el cuerpo completo de cada guarda de salida, por ejemplo `if ($id.User.Value -eq 'S-1-5-18') { Write-Output "verificacion: runner propio como SYSTEM; F1-8, decision 6"; exit 2 }`, y anadir M8 y M9 a la lista de mutaciones del test. Es un cambio barato y no abre ronda: si quien orquesta prefiere dejarlo como MENOR aceptado, debe anotarse con el motivo.

## Alcance
- Ficheros revisados: `xtask/src/workflows.rs` (diff completo de la ronda: tests nuevos en `el_runner_propio_solo_en_la_release_de_un_privado` y `el_aislamiento_va_antes_del_checkout_en_el_runner_propio`, y `aislamiento_antes_del_checkout`). El `.github/workflows/verificacion.yml` no es Rust: se leyeron sus lineas del paso `buscar` y del paso de aislamiento porque los tests lo leen y lo mutan.
- Contexto adicional pedido: ninguno. El `.github/workflows/verificacion.yml` y el informe de la ronda 1 estan en el contexto permitido.
- Limite JS-WASM: sin cambios de firma exportada. El diff es un xtask nativo y no toca `#[wasm_bindgen]` ni tipos que crucen el limite. No aplica llamador TypeScript ni `.d.ts`.

## Reproduccion
- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-045-r2-rust`, clonado del repo local y checkout de `e141ed154087ea00e2e7fa9536bed2fb42c093f7` (verificado con `git log -1`; arbol limpio). El clon se borra al terminar esta revision.
- Build: `cargo test -p xtask --locked` compila y ejecuta. No se compilo para `wasm32` (el diff no toca codigo de wasm).
- Formato y lints: no ejecutados en esta ronda. La peticion es una verificacion acotada de las guardas del workflow y de `xtask/src/workflows.rs`; la puerta completa (`cargo fmt --check`, `cargo clippy`, suite del workspace) no se ha repetido aqui y no se afirma.
- Suite: `cargo test -p xtask --locked` => `34 passed; 0 failed` (17.05 s), linea base sobre e141ed1.
- Casos construidos (mutaciones sobre el workflow en el clon; tras cada una se restauro con `git checkout`, el arbol quedo limpio):
  - M1 comprobacion de `$LASTEXITCODE` tras `whoami /groups` desactivada (`if ($false) {`) => ROJO (`el_aislamiento_va_antes_del_checkout_en_el_runner_propio`).
  - M2 guarda de SYSTEM neutralizada con `if ($false -and ...)` => ROJO (mismo test).
  - M3 guarda de `RUNNER_OS` neutralizada con `if ($false -and ...)` => ROJO (mismo test).
  - M4 guarda de grupo Administradores desactivada (`if ($false) {`) => ROJO (mismo test).
  - M5 comprobacion de `bash` desactivada => ROJO (mismo test).
  - M6 comprobacion de `jq` desactivada con `$false -and` => ROJO (mismo test).
  - M7 regla del evento neutralizada (`if ($false -and $env:PRIVADO ...`) => ROJO (mismo test).
  - M8 guarda de SYSTEM con `exit 2` cambiado por `exit 0` => VERDE (34 passed). Superviviente, ver MENOR-1.
  - M9 guarda de `RUNNER_OS` con `exit 2` cambiado por `exit 0` => VERDE (34 passed). Superviviente, ver MENOR-1.
  - M10 variable `EVENTO` quitada del `env:` del paso de aislamiento => ROJO (4 tests: `el_aislamiento_va_antes_del_checkout_en_el_runner_propio`, `el_informe_se_sube_siempre_y_solo_desde_ejecutar`, `el_runner_propio_solo_en_la_release_de_un_privado`, `el_workflow_reutilizable_pasa_el_lint`).
  - M11 en `buscar`, vuelta al `for p in $(printf ... | jq ...)` sin la lista previa: con `PERFILES=basura` el jq falla dentro de `$()` y el paso ya no falla cerrado => ROJO (`el_runner_propio_solo_en_la_release_de_un_privado`, test de "perfiles ilegibles").
  - M12 en `buscar`, `if [ "$PRIVADO" = false ]` cambiado por `!= true` (un `private` vacio cae en "repo publico") => ROJO (`el_runner_propio_solo_en_la_release_de_un_privado`, test de "private vacio").
- Medidas: no aplican (sin codigo de rendimiento ni wasm).

## Revisado sin hallazgos
- Correccion de MEN-1, la parte de los tests: las guardas de `RUNNER_OS`, `SYSTEM`, `whoami` (`$LASTEXITCODE`), grupo Administradores, `bash` y `jq` se exigen por su expresion completa; las tres mutaciones de la ronda 1 (quitar `$LASTEXITCODE`, neutralizar SYSTEM, neutralizar `RUNNER_OS`) ahora ponen rojo. El test de `env:` del paso de aislamiento compara el valor exacto de los seis datos del evento: quitar uno o cambiarlo lo tumba (M10).
- Regla del evento repetida en `ejecutar` (`.github/workflows/verificacion.yml:373-374`): los datos llegan por `env:` (no interpolados en el script), la comparacion es sensible a mayusculas (`-cne`) y el repo del head se compara con el del repo. Ningun dato del evento queda sin comparar en `if` y no hay una ruta que deje pasar un valor vacio: `private` vacio o ausente falla con "no consta si el repo es privado" (M12).
- `buscar`: el cambio a `lista=$(...) || { ...; exit 2; }` hace que un `PERFILES` ilegible falle cerrado con `set -euo pipefail` (M11 lo demuestra). El motivo nuevo nombra el origen y el evento y dice que hacer; el test lo comprueba.
- Cambios de `xtask/src/workflows.rs` (solo tests y una funcion de comprobacion): compilan, pasan y no hay `unwrap` sobre datos externos fuera de tests, ni `unsafe`, ni numerica.
- Nota no hallazgo: el test `etiqueta suelta: limite conocido de buscar` fija como resultado esperado (`Ok("Runner propio: ningun perfil de este evento lo usa")`) el limite de que `buscar` no reconoce una etiqueta sin `self-hosted`. Es correcto mientras ese limite exista; si `buscar` cambia, el test debera cambiarse con el.
- MEN-2 de la ronda 1 no se reverifica aqui, como indica la peticion.

## Lecturas fuera del contexto permitido
- `C:\Users\nullcad2025\.claude\plugins\cache\taskcode-marketplace\taskcode-plugin\0.16.2\skills\task-workflow\revision.md`: reglas comunes de la revision (formato de hallazgos, veredicto y rondas). Hace falta para escribir la linea de veredicto y la tabla con la forma exigida.
