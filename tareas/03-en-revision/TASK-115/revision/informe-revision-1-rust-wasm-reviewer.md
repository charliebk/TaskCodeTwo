# Informe de revision — TASK-115 (ronda 1)

- Commit revisado: 71fb3c554a0f6ab6091d54200bdeb4ba74520ce5
- Revisor: rust-wasm-reviewer
- Veredicto: PENDIENTE (escribelo con: taskctl veredicto TASK-115 aprobada | aprobada-con-correcciones | cambios-solicitados)

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/descarga.rs:235 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### MEN-1 — `es_transitorio` es `pub` y expone `ureq::Error` en la API de la biblioteca

`descarga::es_transitorio(e: &ureq::Error)` se declara `pub` en un modulo `pub`
(`lib.rs:23`), de modo que el tipo de `ureq` pasa a formar parte de la API
publica del crate `tc2`. Solo lo usan `Descargador::a_escritor` y su par
`es_transitorio_io` (privado), todo dentro de `descarga.rs`; la unica otra
mencion es un enlace de documentacion desde `actualizar_cli.rs:629`. Basta
`pub(crate)` (el enlace de rustdoc sigue resolviendo). Reproduccion:
`grep -rn es_transitorio crates` no da ningun uso fuera del crate. No afecta al
comportamiento: es superficie de API innecesaria.

## Alcance

- Ficheros revisados: crates/tc2/src/actualizar_cli.rs, crates/tc2/src/descarga.rs,
  crates/tc2/tests/actualizar_cli.rs, crates/tc2/tests/comun/http_mock.rs,
  crates/tc2/tests/plataforma.rs, crates/tc2/tests/plataforma_gitlab.rs.
- Contexto adicional pedido: `crates/tc2/src/plataforma/http.rs` (`agente` y
  `clasificar`, para ver que plazos configura el agente y como clasifica) y el
  fuente de ureq 3.5.2 / rustls 0.23.46 del registro de cargo (de donde sale el
  `InvalidData` del handshake y como ureq envuelve sus errores en `io::Error`).
- Limite JS-WASM: sin cambios; el diff no toca firmas exportadas con
  `#[wasm_bindgen]` ni tipos que crucen el limite.

## Reproduccion

- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-115-r1-rust`,
  rama `fix/task-115-seguimiento-de-f2-12-menores-de-actualiz` (HEAD ada73b2, que solo
  anade la peticion sobre el 71fb3c5 revisado). Borrado al terminar.
- Build: no se ejecuto `cargo build` aparte; lo compilan `cargo test` y `clippy`.
  El diff es nativo (Windows), sin codigo wasm32.
- Formato y lints: `cargo clippy --all-targets --locked -- -D warnings`: sin avisos.
  No se ejecuto `cargo fmt --check` (no se pidio).
- Suite: `cargo test -p tc2` en verde (3 m 15 s, todos los binarios `ok`; los tests
  de lib y de `actualizar_cli` se repitieron aparte: 119 pasados). Sin linea base
  previa al diff.
- Serie de `plataforma` (`cargo test -p tc2 --test plataforma -q`, 3 seguidas): 5 de 5
  cada vez, `finished in` 3,84 s / 3,79 s / 3,83 s. Ninguna parada de ~10 s.
- Casos construidos: la propia suite; ademas se leyo el fuente de ureq/rustls para
  comprobar que el handshake fallido llega como `Error::Io(InvalidData)`
  (`rustls conn.rs:365/701`) y que el cuerpo envuelve los errores de ureq en `io::Error`
  (`error.rs`, `From<io::Error>`), lo que `es_transitorio_io` desenvuelve con
  `downcast_ref`.
- Mutacion (una por criterio, cada una revertida con `git checkout`):
  - C1: `recoger` devuelve vacio si vence el plazo → `ejecutar_con_limite_devuelve_lo_leido_si_un_nieto_retiene_stdout` rojo (`stdout: ""`).
  - C2: `Timeout(Global) => true` → `timeout_global_no_se_dobla` rojo; `InvalidData` transitorio → `tls_fallido_no_es_transitorio` rojo (cada una por separado).
  - C3: reintentar cualquier error → `redireccion_no_permitida_no_se_sigue_ni_reintenta` rojo y el de limite de tamano tambien (2 peticiones al binario; 2 fallos en 10).
  - C4: el mock suelta el socket tras responder → `tras_una_respuesta_completa_el_mock_no_manda_fin` rojo («cerro la conexion (leidos 0 bytes)»).
  - C5: una `const` entre el doc y la funcion → rojo («sin doc justo encima»); linea de doc duplicada → rojo («linea repetida»).
- Medidas: tiempos de las tres series arriba; sin medidas de `.wasm` (no aplica).

## Revisado sin hallazgos

- Mock con hilo por conexion y `read_to_end` con plazo de 30 s: cada hilo acaba al
  cerrar el cliente o, como tope, a los 30 s (el error de plazo se ignora y se suelta
  el socket); la espera de una conexion no frena a las demas. El hilo del listener vive
  lo que el proceso de tests, como antes. Sin fugas apreciables ni hilos que cuelguen la suite.
- `serie()`: un `static Mutex<()>`; se recupera del envenenamiento con
  `unwrap_or_else(PoisonError::into_inner)`. No hay deadlock: la guarda se coge una vez al
  empezar cada test y ningun ayudante (`modelo`, `verde_con`, `Mock::nuevo`) la vuelve a
  coger. Cubre todos los tests con mock de `plataforma.rs` (4) y `plataforma_gitlab.rs` (3)
  y el del propio mock; el resto de la suite sigue en paralelo.
- `es_transitorio`/`es_transitorio_io`: `Timeout(Global)` → false tanto en `call()` como
  envuelto en el cuerpo; el resto de plazos (`Connect`, etc.), `HostNotFound`,
  `ConnectionFailed`, `BodyStalled` e `Io` no `InvalidData` → true. `InvalidData` solo lo
  genera aqui rustls (en las lecturas de red de ureq no aparece otro; el de `socks.rs` no es
  transitorio con razon), asi que no hay falsos positivos con la configuracion del proyecto.
  El `UnexpectedEof` en pleno handshake se sigue reintentando una vez (documentado, coste 1 s).
- `Descargador::con_espera`: `nuevo()` la usa con el mismo `ESPERA_DESCARGA` (120 s), comportamiento igual.
- `ejecutar_con_limite`: buffer compartido con `Mutex` que tolera envenenamiento, tope de 1 MiB
  igual que antes (`saturating_sub`/`get`, sin indexado), `Interrupted` reintentado, plazo total
  respetado; la doc dice que con un nieto la llamada tarda el plazo entero, y el test lo acota (< 4 s con plazo de 2 s).
- `bajar_con_un_reintento` como `pub(crate)`: lo necesitan los tests de `descarga`; sin otro efecto.
- Test de doc con `include_str!`: no se auto-detecta (la firma se compone con `concat!`) y cubre
  doc ausente, duplicado y `const` interpuesta (las tres mutaciones salieron rojas).
- Produccion sin `unwrap`, `expect`, `panic!` ni `unsafe` en `actualizar_cli.rs` y `descarga.rs`.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/plataforma/http.rs` (`agente`, `clasificar`): para saber que plazos configura el agente y que clasifica como `Sinred`.
- `crates/tc2/src/lib.rs`: para ver que `descarga` es modulo publico (MEN-1).
- Fuente de ureq 3.5.2 y rustls 0.23.46 en el registro de cargo: origen del `InvalidData` TLS y envoltorio de errores.
- `planificacion/plan-final.md` no se abrio por separado; los criterios y el plan resumido estan en `tarea.md` y en la peticion.
