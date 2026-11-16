# Informe de revision — TASK-027 (ronda 3)

- Commit revisado: e3ac6d97508ab26cf4f7e713e7ed37534d286f2f
- Revisor: code-quality-reviewer
- Veredicto: aprobada

Propuesta del revisor: **aprobada**. IMP-2 queda corregido y verificado; no quedan CRITICO ni IMPORTANTE abiertos. Los dos MENOR son nuevos, los introduce la propia correccion y no bloquean (ver abajo).

Alcance: verificacion acotada (ver `## Verificacion de la ronda 2`). El codigo de `crates/` de `e3ac6d9` y de `6acf391` (HEAD, que solo anade la peticion) es identico.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/actualizar_cli.rs:451 | implementacion |
| IMP-2 | IMPORTANTE | corregido | crates/tc2/src/descarga.rs:279 | implementacion |
| MENOR-1 | MENOR | abierto | crates/tc2/src/actualizar_cli.rs:658 | implementacion |
| MENOR-2 | MENOR | abierto | crates/tc2/src/actualizar_cli.rs:574 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

## Verificacion de la ronda 2

- **IMP-2 (reintento ante cortes antes de las cabeceras), corregido.** `descarga.rs:279` clasifica `FalloHttp::Sinred` (DNS, conexion, tiempo agotado, E/S, cuerpo parado) como `transitorio: true`, y `bajar_con_un_reintento` (`actualizar_cli.rs:676`) reintenta ante `transitorio: true`. El test `un_corte_en_la_descarga_del_binario_se_reintenta_una_vez` prueba los dos cortes, cada uno con su propio servidor y dos peticiones al binario (`assert_eq!(al_binario, 2)`), y comprueba exit 0, la salida `actualizar ok ...` y el sha256 de vY.
  - Mutacion M1 (corte antes de cabeceras no transitorio, `descarga.rs:279` a `sinred(e, false)`): el test cae en rojo en `actualizar_cli.rs:578` (exit code, `corte-antes`).
  - Mutacion M2 (sin reintento, guard `transitorio: false`): cae en rojo en `actualizar_cli.rs:578`. Es el comportamiento de `39ef7a1`, asi que el test reproduce el fallo original.
  - Mutacion M3 (corte a mitad del cuerpo no transitorio, `descarga.rs:343`): cae en rojo en `actualizar_cli.rs:578`.
  - Las tres mutaciones se restauraron con `git checkout` en el clon, que se borro al terminar.
  - Conclusion: el test discrimina los dos cortes. Ninguna de las tres mutaciones pasa en verde.
- **IMP-1 (attestation con `gh`), sigue corregido.** Esta ronda no toca `instalar()` ni `comprobar_attestation`. El test `attestation_con_gh_ausente_valido_y_que_falla` pasa. No se ha vuelto a mutar en esta ronda.
- **Nota sobre el reintento y el corte de red.** En esta pasada no hubo cortes de la red local: la suite de `actualizar_cli` termino en 24,78 s. Los dos casos de corte pasaron a la primera.

## Hallazgos nuevos (introducidos por la correccion, severidad MENOR)

### MENOR-1 — doc comment de `bajar_con_un_reintento` desplazado a la constante `PAUSA_REINTENTO`
- Donde: crates/tc2/src/actualizar_cli.rs:658
- Que pasa: el bloque de doc que describe `bajar_con_un_reintento` (lineas 658-665) queda encima de `const PAUSA_REINTENTO` (linea 667). La funcion `bajar_con_un_reintento` (linea 669) no tiene doc, y la constante lleva un parrafo que no le corresponde. Ademas, la linea 666 (`/// Pausa antes del reintento de la descarga.`) es un doc suelto que duplica la intencion de la constante.
- Reproduccion: lectura del fichero en `6acf391` (`sed -n 655,670p crates/tc2/src/actualizar_cli.rs`). No es un fallo de compilacion ni de test: `cargo build` y `cargo test` pasan.
- Impacto: quien lea `rustdoc` o el propio fichero ve la explicacion del reintento colgada de la constante. No cambia el comportamiento.
- Sugerencia: subir el bloque de doc a `bajar_con_un_reintento`, dejar `PAUSA_REINTENTO` con una linea de doc propia y borrar la linea 666.

### MENOR-2 — el doc de `ejecutar_con_limite` dice que se devuelve lo leido hasta el plazo, y no es asi
- Donde: crates/tc2/src/actualizar_cli.rs:574
- Que pasa: el doc dice "se devuelve lo leido hasta entonces". Los hilos lectores solo envian su `Vec` cuando `read_to_end` termina. Si el plazo se cumple con un nieto que retiene las tuberias heredadas, `recv_timeout` agota el plazo y devuelve `unwrap_or_default()`, es decir, un `stdout` vacio, aunque el hijo haya terminado con exito. Esto no lo ejecute: lo afirmo por lectura del codigo (el hilo solo envia al final).
- Reproduccion: no ejecutada en esta ronda. Caso a ejecutar: un `.bat` que escribe una linea a `stdout`, lanza en segundo plano un proceso que hereda las tuberias y termina con codigo 0; el resultado esperado segun el doc seria la linea escrita, y por lectura saldria vacia tras el plazo. El test `ejecutar_con_limite_cumple_el_plazo_con_un_nieto` no cubre este caso: solo mira el plazo y el caso sin nieto.
- Impacto: `version --json` (`actualizar_cli.rs:746`) usa `stdout`; con un `tc2` que no lanza nietos no se da el caso. Es un caso de borde, por eso MENOR.
- Sugerencia: corregir el doc para decir lo que pasa (la salida se pierde si un nieto retiene las tuberias), o devolver lo leido hasta el plazo si la mejora compensa. Cualquiera de las dos vale; no bloquea.

## Ejecucion

- Clon nuevo en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-027-r3-calidad`, rama `feature/task-027-f2-12-taskctl-actualizar-cli-y-aviso-de`, HEAD `6acf391`. Codigo de `crates/` igual a `e3ac6d9`.
- Toolchain del clon: la del equipo.
- Puerta de build: `cargo build -p tc2 --examples`, exit 0 (1 min 00 s).
- `cargo test -p tc2 --test actualizar_cli`: **8 de 8 pasan**, en 24,78 s (los 8 tests del fichero, incluido `un_corte_en_la_descarga_del_binario_se_reintenta_una_vez`).
- `cargo test -p tc2 --lib ejecutar_con_limite`: **1 de 1 pasa** (`ejecutar_con_limite_cumple_el_plazo_con_un_nieto`).
- Mutaciones: M1, M2 y M3 descritas arriba, cada una con `cargo test -p tc2 --test actualizar_cli un_corte`, en rojo.
- No ejecutado en esta ronda: la suite completa de `tc2` (`cargo test -p tc2` sin filtro), `cargo clippy` y `cargo fmt --check`. El encargo acotado pedia `actualizar_cli` y el test del nieto. Si la puerta de la ronda exige la suite entera, hay que correrla; el veredicto de arriba no la cubre.

## Notas (no abren hallazgo)

- Los hallazgos de la ronda 1 que no se verifican aqui (MEN-1 a MEN-6 y N-1 a N-5 de la ronda 2) siguen como estaban; no se reverifican en esta ronda.
- MEN-5 de la ronda 1 (`arreglo` con significados distintos) sigue sin corregir, por decision del coordinador, como indicaba la ronda 2.

## Lecturas fuera del contexto permitido

- crates/tc2/src/plataforma/http.rs (funcion `clasificar`, lineas ~343-360): para saber a que fase corresponde cada `ureq::Error` y comprobar que `Sinred` agrupa solo fallos de transporte transitorios.
- revision.md y la skill code-quality-reviewer (plugin): para el protocolo de la ronda y la plantilla del hallazgo. No es codigo revisado.
