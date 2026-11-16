# Informe de revision — TASK-027 (ronda 2)

- Commit revisado: 39ef7a1862bb2047f844adb168309dc4dd92d19c
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

Propuesta del revisor: `aprobada`. No hay CRITICOS ni IMPORTANTES. Quedan MENORES abiertos:
MEN-1 y MEN-5 (sin corregir a proposito) y MEN-6 (nuevo, introducido por la correccion de MEN-4).

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/actualizar_cli.rs:763 | implementacion |
| MEN-2 | MENOR | corregido | crates/tc2/src/actualizar_cli.rs:646 | implementacion |
| MEN-3 | MENOR | corregido | crates/tc2/src/actualizar_cli.rs:619 | implementacion |
| MEN-4 | MENOR | corregido | crates/tc2/src/actualizar_cli.rs:708 | implementacion |
| MEN-5 | MENOR | abierto | crates/tc2/tests/actualizar_cli.rs (hueco: sin test de reintento, redireccion ni limite) | implementacion |
| MEN-6 | MENOR | abierto | crates/tc2/src/actualizar_cli.rs:570 (lectura de las tuberias en 608-609) | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Sin CRITICOS ni IMPORTANTES.

Los numeros de linea son los de 39ef7a1.

### MEN-1 — `.old` bloqueado impide actualizar (Windows) — abierto, a proposito
Sin cambios en esta ronda: `sustituir_con` sigue ignorando el error de `remove_file(&old)`
(actualizar_cli.rs:763). Queda como en la ronda 1; el estado del binario no se corrompe.

### MEN-2 — el reintento no se limita a `sinred` de verdad — corregido
Verificado. `Fallo::Sinred` lleva ahora `corte: bool`; solo `corte: true` (corte a mitad del cuerpo,
descarga.rs:339) se reintenta (actualizar_cli.rs:646). Conexion, TLS, otros errores de cabecera,
respuesta invalida, redireccion, 404 y 5xx no se reintentan. `Descargador::texto` ya no informa de
`Grande` cuando `SHA256SUMS` no es UTF-8: devuelve `NoEsTexto` (descarga.rs:250).
Reproducido con un servidor local en 127.0.0.1 (ver Reproduccion):
- corte a mitad una vez, luego correcto: 2 peticiones, `actualizar ok` (exit 0);
- corte a mitad siempre: 2 peticiones, `actualizar sinred` (exit 4);
- 404: 1 peticion, `rechazada motivo=no-publicada` (exit 3);
- 500: 1 peticion, `actualizar sinred motivo=http estado=500` (exit 4).
No reproducido: TLS o `NoEsTexto` con un servidor malo (lectura del codigo).

### MEN-3 — disco lleno se informa como `sin-permiso` — corregido (por lectura)
`fallo_de_escritura` (actualizar_cli.rs:619) solo da `sin-permiso` si `ErrorKind::PermissionDenied`;
cualquier otro fallo de escritura sale como `error motivo=escritura` con el texto `ESCRITURA`
(«Comprueba el espacio libre en disco»). Los puntos de escritura de `instalar` y
`Fallo::Escritura` (con su `tipo`) pasan por ahi. El permiso denegado al crear el temporal sigue
siendo `sin-permiso`, antes de bajar nada. No reproducido: no se puede llenar el disco aqui.

### MEN-4 — `comprobar_nuevo` sin tiempo limite y `--comprobar` con `limpiar_old` — corregido
- Limite: `comprobar_nuevo` usa `ejecutar_con_limite` con `ESPERA_NUEVO` de 30 s (actualizar_cli.rs:703-708).
  Reproducido con un proceso directo (`ping -n 150`) y limite de 30 s: `Ok(None)` a los 30.0 s y el
  proceso muerto. Con un envoltorio que lanza un nieto, el limite no se cumple: ver MEN-6.
- `--comprobar`: `limpiar_old` se ha movido a `instalar` (actualizar_cli.rs:396) y ya no se llama en
  `actualizar` antes de `--comprobar`. Comprobado: con `--comprobar` y un `.old` previo, el `.old`
  desaparece igual, porque `main.rs:18` lo limpia al arrancar cualquier `tc2` en Windows. Es
  comportamiento previo, no nuevo (ver Notas).

### MEN-5 — sin test de integracion del reintento, la redireccion y el limite — abierto, a proposito
Sin cambios en esta ronda. El reintento se ha verificado a mano (MEN-2), pero no hay test de
regresion en `crates/tc2/tests/actualizar_cli.rs`.

### MEN-6 — un nieto del `gh` o de `version --json` retiene las tuberias y el limite no se cumple — abierto (nuevo)
Origen: la correccion de MEN-4 y la de la attestation introducen `ejecutar_con_limite`. Tras matar el
hijo, la funcion espera a que los hilos de lectura de stdout y stderr terminen (actualizar_cli.rs:608-609).
Si el hijo ha lanzado un proceso nieto que hereda las tuberias, el cierre de la tuberia depende del
nieto, no del hijo muerto. Reproducido en Windows con un envoltorio `.bat` como `TC2_GH` (el `gh` por
defecto es un binario unico y no lanza nietos):
- `comprobar_nuevo` con un `.bat` que lanza `ping -n 150`: devuelve `Err("version --json no respondio en 30 s")`
  a los 150.5 s, no a los 30 s;
- `comprobar_attestation` con un `.bat` que responde a `auth status` y lanza `ping -n 150` en
  `attestation verify`: devuelve `Err("gh attestation verify no respondio en 60 s")` a los 150.5 s, no a los 60 s.
Con un nieto que no termina, la llamada no volveria nunca (lectura del codigo: `join` espera al cierre
de la tuberia). Impacto real bajo: exige que `TC2_GH` apunte a un envoltorio, o un `version --json` de
un binario que lance nietos. Correccion sugerida: no esperar a los hilos mas que un tiempo acotado tras
el kill, o matar el arbol de procesos.
Tambien por lectura (sin reproducir): si `try_wait()` falla con `?` (actualizar_cli.rs:598), el hijo
no se mata y queda vivo.

## Alcance
- Ficheros revisados (los del diff que casan con los patrones): crates/tc2/src/actualizar_cli.rs
  (`detectar_modo`, `gh_de_variable`, `actualizar`, `instalar`, `comprobar_attestation`,
  `ejecutar_con_limite`, `ultima_linea`, `fallo_de_escritura`, `bajar_con_un_reintento`,
  `comprobar_nuevo`), crates/tc2/src/descarga.rs, crates/distribucion/src/lib.rs,
  crates/tc2/examples/gh_falso.rs, crates/tc2/tests/actualizar_cli.rs (lectura del fixture),
  crates/tc2/src/i18n/catalogo/actualizar.rs y docs/distribucion/instalar.md (solo por contraste).
- Contexto adicional pedido: ninguno. Se leyeron el diff de la peticion y los ficheros del clon.
- Limite JS-WASM: sin cambios. El diff no toca `#[wasm_bindgen]` ni codigo `wasm32`; es CLI nativo.

## Reproduccion
- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-027-r2-rust`, rama
  `feature/task-027-f2-12-taskctl-actualizar-cli-y-aviso-de`, checkout de 39ef7a1. Borrado al terminar.
  El HEAD de la rama es b3d2e77 (solo cambia `tareas/`); el codigo revisado es el de 39ef7a1.
- Build: `cargo test --locked -p tc2` compila el workspace nativo. Sin objetivo `wasm32`.
- Formato y lints: `cargo clippy --all-targets --locked -- -D warnings` → exit 0, sin avisos.
  `cargo fmt --check` no se ejecuto en esta ronda.
- Suite: `cargo test --locked -p tc2` sobre 39ef7a1 → todo verde: lib 64 passed; los binarios de
  integracion pasan (incluido `tests/actualizar_cli.rs`), 0 fallos. Sin caidas de red en esta ronda.
  Se ejecuto de nuevo sobre 39ef7a1; no se reutiliza ninguna linea base.
- Casos MEN-2 (servidor Python en 127.0.0.1, `TC2_BASE_URL`, `tc2.exe` de debug copiado fuera del
  repo, `TC2_IDIOMA=es`): ver la lista de MEN-2 (exit codes 0, 4, 3 y 4).
- Casos MEN-4: `--comprobar` con `.old` previo (el `.old` se borra por `main.rs:18`); proceso directo
  `ping -n 150` con limite de 30 s → `Ok(None)` a 30.0 s; envoltorios `.bat` → 150.5 s (MEN-6).
- Casos MEN-3: lectura de `fallo_de_escritura` y de sus llamadas; no reproducido (disco lleno).
- Attestation: `gh` real no se ejecuto (sin red ni sesion de prueba). Los tests del repo con
  `gh_falso` pasan. Verificado por lectura: `gh` se lanza con `Command` y un array de argumentos,
  sin shell; `--repo` y `--signer-workflow` son constantes de `distribucion`; la ruta del temporal
  va como argumento separado. El orden es correcto: la attestation se comprueba antes de
  `hacer_ejecutable` y de lanzar el binario nuevo (actualizar_cli.rs:448-466).
- Mutacion: no se paso `cargo mutants`.

## Notas (no van en la tabla)
- `--comprobar` sigue diciendo «no toca nada» en el comentario de `instalar` y en el de
  `actualizar`. En Windows `main.rs:18` limpia el `.old` en cada arranque, asi que el texto es inexacto.
  Es comportamiento previo a esta ronda.
- Un fallo de red del propio `gh attestation verify` se informa como `rechazada
  motivo=attestation-no-valida` con el detalle de `gh`. Es una eleccion de diseno, pero el mensaje
  puede llevar a pensar que el binario es malo.
- Si `gh auth status` falla o no hay sesion, la comprobacion se salta en silencio con
  `attestation=sin-gh`. Es la paridad pedida con los instaladores y esta documentada en `instalar.md`.
- Si el hijo escribe mas de 1 MiB, el lector deja de leer y el hijo se bloquea hasta el limite; el
  resultado es «no respondio» y no un error de salida. No es un caso real con `gh` ni con `tc2`.

## Revisado sin hallazgos
- Ningun `unwrap()`, `expect(`, `panic!` ni `unsafe` en el codigo de produccion de los ficheros del
  diff (solo `unwrap_or_default`). El `unsafe_code = "forbid"` del workspace se comprobo en la ronda 1.
- Indices de `detectar_modo`: `dir.len() >= 2` antes de `dir[len-2]`; `position` con `i + k < len`.
- `gh_de_variable`: `TC2_GH` vacia cuenta como no definida; en ensayo solo se comprueba si `TC2_GH` esta definida.
- Lectura de salidas: `take(1 MiB)` en ambos hilos, stdin nulo, `kill` y `wait` tras el limite cuando
  el padre sigue vivo.
- `comprobar_nuevo` y la attestation se lanzan solo tras el sha256 y antes de ejecutar el binario nuevo.

## Lecturas fuera del contexto permitido
- `crates/tc2/src/main.rs`: solo la linea 18 (`limpiar_old` al arrancar), por grep. Para entender el caso de `--comprobar` (MEN-4).
- `.github/workflows/release.yml`: solo grep de `attest`. Para confirmar que la release genera la attestation que `--signer-workflow` espera.
