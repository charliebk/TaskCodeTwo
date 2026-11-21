# Informe de revision — TASK-045 (ronda 1)

- Commit revisado: addfcb2da94056304689989a9516567c77a7c466
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | corregido (e141ed1) | xtask/src/workflows.rs (aislamiento_antes_del_checkout) | implementacion |
| MEN-2 | MENOR | aceptado: la asercion de `sha256sum < "$HUELLA"` cubre el riesgo real, como propone la revisora | xtask/src/workflows.rs (sha256sum_solo_por_la_entrada_estandar, `con_ruta`) | implementacion |

Sin hallazgos CRITICO ni IMPORTANTE.

### MEN-1 — el test del aislamiento solo mira que el texto exista, no que el PowerShell haga lo que dice
- Caso: en el clon, sobre `.github/workflows/verificacion.yml`, tres mutaciones del script del paso «aislamiento del runner propio» dejan `cargo test -p xtask` en verde (34 passed):
  1. quitar la comprobacion `if ($LASTEXITCODE -ne 0) {...}` de `whoami /groups`;
  2. neutralizar la guarda de SYSTEM (`if ($false) { # S-1-5-18`), de modo que el literal sigue presente;
  3. neutralizar la guarda de sistema operativo (`$false -and "RUNNER_OS"`).
- Causa: `aislamiento_antes_del_checkout` solo exige que `run` contenga las cadenas `S-1-5-18`, `System32\whoami.exe" /groups`, `RUNNER_OS`, etc. Solo la mutacion «sin el grupo Administradores» (que reemplaza una expresion completa) esta cubierta por contenido.
- Impacto: acotado. El E2E real (ejecuciones 44901386742 y 44922050486, usuario `tc2-runner` sin administracion) demuestra el camino verde, pero no un camino de rechazo del propio PowerShell (el test no puede ejecutarlo en CI de Linux). Una regresion silenciosa en esas guardas pasaria la suite.
- Correccion sugerida: exigir la expresion completa de cada guarda (p. ej. `-eq 'S-1-5-18') {`, `-ne 'Windows'`, `$LASTEXITCODE -ne 0`) en lugar de la subcadena suelta, y anadir esas mutaciones a la lista. Alternativa: aceptarlo y dejarlo documentado, porque la verificacion real es el E2E.

### MEN-2 — `con_ruta` es un patron estrecho (aceptable)
- `con_ruta` solo detecta un argumento que empieza por comilla tras `sha256sum`. Probado: `sha256sum -b "$HUELLA"` y `sha256sum $HUELLA` tambien ponen el test en rojo, porque ademas se exige la cadena literal `sha256sum < "$HUELLA"`. Sin impacto real; se anota por constar. Estado propuesto: aceptado.

## Alcance
- Ficheros revisados: xtask/src/workflows.rs (diff completo del dominio). Para juzgar los tests se leyo ademas el `run:` real de `.github/workflows/verificacion.yml` (ver lecturas fuera de contexto).
- Contexto adicional pedido: el workflow, porque los tests ejecutan y mutan su contenido.
- Limite JS-WASM: sin cambios de firma exportada; el diff no toca `#[wasm_bindgen]` ni ningun tipo que cruce el limite (es un xtask nativo).

## Reproduccion
- Clon: C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-045-r1-rust, checkout de addfcb2da94056304689989a9516567c77a7c466 (HEAD verificado igual a `git log -1`).
- Build / formato / lints / suite completa: no repetidos por indicacion de la peticion: `cargo xtask verificar --perfil completa` en verde sobre este commit (2026-11-21, hecha a solas). No aplica wasm32.
- Suite del dominio: `cargo test -p xtask` => `34 passed; 0 failed` (linea base en el clon, incluye los tests nuevos `el_runner_propio_solo_en_la_release_de_un_privado`, `el_aislamiento_va_antes_del_checkout_en_el_runner_propio`, `sha256sum_solo_por_la_entrada_estandar`, y las dos mutaciones nuevas de `clean:`).
- Casos construidos (mutaciones sobre el workflow, cada una con `cargo test -p xtask`):
  - guarda de repo privado quitada => rojo (`el_runner_propio_solo_en_la_release_de_un_privado`);
  - guarda de head de otro repo quitada => rojo;
  - comprobacion de `ORIGEN`, de `BASE` o de `EVENTO` quitada, cada una por separado => rojo;
  - `exit 2` sustituido por `exit 0` en la guarda => rojo (2 tests);
  - `sha256sum -b "$HUELLA"` y `sha256sum $HUELLA` en lugar de `sha256sum < "$HUELLA"` => rojo;
  - literal `S-1-5-18` retirado => rojo;
  - lint de `clean:` neutralizado en `lint_con` (`opcion("clean")...` -> `false`) => rojo (`cada_mutacion_prohibida_da_su_error`).
  - Mutaciones que NO se detectan: las tres de MEN-1.
- E2E (no consultado por esta revisora con `gh`; se toman de la peticion): 44894375522 (fallo AllSigned), 44901386742 (fallo jq), 44922050486 (verde, pesada en runner propio), 44922284948 (verde, huella sin `\`), 44831249131 (negativo en repo publico, runner propio rechazado). Cada fallo real que cita la peticion tiene su correccion y su test en el diff.
- Medidas: no aplican (sin codigo de rendimiento ni wasm).

## Revisado sin hallazgos
- Paso «Runner propio» de `buscar`: datos del evento por `env:`, no interpolados en el script; `set -euo pipefail`; un `runs-on` ilegible falla cerrado con exit 2; etiqueta sin distincion de mayusculas; grupos de runners cuentan como propio; sin perfiles no hace nada. Los 8 casos malos mas los positivos del test coinciden con el script real.
- Regla de lint `clean:` (acepta `true`, rechaza `false` y expresiones `${{ }}`), con dos casos en `cada_mutacion_prohibida_da_su_error`.
- Orden del aislamiento antes del checkout, `if`, `shell` con `-ExecutionPolicy Bypass` y `jq`: cubiertos por mutacion (quitado, movido, otro `if`, otro shell, sin jq, sin grupo Administradores).
- Test de `sha256sum`: cubre la regresion que dio el E2E.
- Codigo Rust del diff: solo tests; sin `unwrap` sobre datos externos fuera de tests, sin `unsafe`, sin numerica.

## Lecturas fuera del contexto permitido
- `.github/workflows/verificacion.yml` (script real del paso «Runner propio», aislamiento y huella): hace falta para saber que las mutaciones de los tests casan con el workflow real.
