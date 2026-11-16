# Informe de revision — TASK-027 (ronda 3)

- Commit revisado: e3ac6d97508ab26cf4f7e713e7ed37534d286f2f
- Revisor: rust-wasm-reviewer
- Veredicto: PENDIENTE (escribelo con: taskctl veredicto TASK-027 aprobada | aprobada-con-correcciones | cambios-solicitados)

Propuesta del revisor: `aprobada`. No hay CRITICOS ni IMPORTANTES. MEN-6 (ronda 2) queda `corregido`
y verificado. Quedan tres MENORES nuevos (MEN-7, MEN-8, MEN-9) introducidos por la correccion, documentados
abajo, y los MENOR abiertos de la ronda 2 (MEN-1 y MEN-5, a proposito).

El codigo revisado (crates/) es identico en e3ac6d9 y en 6acf391 (la punta de la rama solo cambia `tareas/`).

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/actualizar_cli.rs | implementacion |
| MEN-2 | MENOR | corregido | crates/tc2/src/actualizar_cli.rs | implementacion |
| MEN-3 | MENOR | corregido | crates/tc2/src/actualizar_cli.rs | implementacion |
| MEN-4 | MENOR | corregido | crates/tc2/src/actualizar_cli.rs | implementacion |
| MEN-5 | MENOR | abierto | crates/tc2/tests/actualizar_cli.rs | implementacion |
| MEN-6 | MENOR | corregido | crates/tc2/src/actualizar_cli.rs:633 | implementacion |
| MEN-7 | MENOR | abierto | crates/tc2/src/actualizar_cli.rs:633 | implementacion |
| MEN-8 | MENOR | abierto | crates/tc2/src/descarga.rs:279 | implementacion |
| MEN-9 | MENOR | abierto | crates/tc2/src/descarga.rs:279 | implementacion |

Las lineas de MEN-1 a MEN-5 son las de 39ef7a1 (ronda 2), que no se han reverificado en esta ronda salvo MEN-6.
Las de MEN-6 a MEN-9 son las de e3ac6d9.

### MEN-6 — nieto que retiene las tuberias y el plazo de `ejecutar_con_limite` — corregido
- Donde: crates/tc2/src/actualizar_cli.rs:633-634 (lectura con `recv_timeout` acotado por el plazo).
- Que pasa, antes: `join` esperaba al cierre de la tuberia; con un nieto que heredaba stderr, la llamada no volvia en 30 s ni en 60 s.
- Verificacion (ronda 3): el test `ejecutar_con_limite_cumple_el_plazo_con_un_nieto` pasa (tests lib, 65 de 65). Se mutó el codigo cambiando `recv_timeout(restante())` por `recv()` en ambos lectores, y el test se puso rojo: `tardo 29.36 s` con la aserción de `< 10 s` fallando. Es decir, el test si detecta la regresion.
- Verificado por lectura: `try_wait` con error (`Err(e)`) mata y recoge al hijo antes de devolver el error (actualizar_cli.rs, bucle de `ejecutar_con_limite`). Sin test que lo cubra: si se quita el `kill`, ningun test se pone rojo (ver Notas).
- Estado: corregido.

### MEN-7 — un hijo que sale bien con un nieto en la tuberia pierde su salida y espera el plazo entero — abierto (nuevo)
- Donde: crates/tc2/src/actualizar_cli.rs:633-634, y la doc de la funcion (actualizar_cli.rs:574-575, «se devuelve lo leido hasta entonces»).
- Que pasa: si el hijo termina con codigo 0 pero un nieto sigue con la tuberia de stdout abierta, el lector no envia nada hasta ver EOF (`read_to_end` acumula dentro del hilo y envia al final). El `recv_timeout` espera hasta el plazo completo y devuelve `stdout` vacio con `status` de exito.
- Reproduccion: test temporal en un clon desechable (no commiteado), `cmd /c "start /b ping -n 20 127.0.0.1 & echo hola"` con limite de 8 s:
  `TMPRES tardo=8.13 s Ok(Some) success=true stdout=""`. El `hola` del hijo se pierde y la llamada tarda todo el plazo.
- Impacto: solo con un envoltorio (`TC2_GH`) o un binario que deja un proceso en segundo plano heredando la tuberia. Con el `gh` por defecto no ocurre. El resultado es una espera de 30 s o 60 s y una salida vacia que `comprobar_nuevo` o la attestation tratan como fallo de respuesta. Antes de la correccion esto era un cuelgue; ahora es una salida silenciosamente perdida. No compromete el estado (no se instala nada que no haya pasado sha256 y attestation).
- Sugerencia: o bien enviar lo leido a mitad (un canal con lo ya acumulado) y devolverlo al cerrar el plazo, o bien corregir la doc para decir que la salida se pierde si la tuberia no cierra. Lo primero es lo que la doc promete.
- Origen: implementacion. Severidad MENOR: exige un envoltorio o un binario que deje un proceso en segundo plano, no es el caso real del `gh` por defecto.

### MEN-8 — un fallo de TLS en el handshake (y de TLS en el cuerpo) se reintenta — abierto (nuevo)
- Donde: crates/tc2/src/descarga.rs:279 (clasificacion `FalloHttp::Sinred(e) => sinred(e, true)`); y crates/tc2/src/descarga.rs:343 (cualquier `io::Error` de la lectura del cuerpo es transitorio).
- Que pasa: la doc de `Fallo::Sinred` dice que TLS se repite igual y no se reintenta. En la practica, un handshake que falla con EOF llega como `ureq::Error::Io` y se clasifica como transitorio.
- Reproduccion (verificada): servidor local en 127.0.0.1 que responde HTTP plano, descarga con `https://`:
  `TMPRES https-a-servidor-http: peticiones_o_conexiones=2 tardo=1.06 s resultado=Err(Sinred { error: "io: unexpected end of file", transitorio: true })`. Dos conexiones: se reintento.
- Por lectura, no reproducido: en el cuerpo, ureq envuelve sus errores en `io::Error` (`ureq-3.5.2/src/error.rs:200-204`, `into_io`), y el bucle de descarga.rs:343 los mapea todos a `transitorio: true`, incluido un error de TLS del cuerpo. Un certificado invalido sale por `Tls` y no se reintenta (verificado por lectura de `clasificar`, plataforma/http.rs:347-355), pero no se pudo probar sin infraestructura de certificados.
- Impacto: un reintento de mas (una conexion extra y 1 s) ante un fallo que se repetira. El resultado final es correcto (`sinred` con exit 4). No corrompe nada.
- Sugerencia: distinguir el error de TLS dentro del `io::Error` (`get_ref()` / `downcast` a `ureq::Error`) o, mas simple, decir en la doc que un fallo de TLS en el handshake puede reintentarse.
- Origen: implementacion. Severidad MENOR: el reintento de mas es inocuo y el resultado final es el correcto.

### MEN-9 — el plazo global de 120 s ahora se reintenta, y el peor caso dobla la espera — abierto (nuevo)
- Donde: crates/tc2/src/descarga.rs:279 (`Timeout` cae en `Sinred`, transitorio) y crates/tc2/src/plataforma/http.rs:347 (`ureq::Error::Timeout(_)` -> `FalloHttp::Sinred`).
- Que pasa: `timeout_global` de `ESPERA_DESCARGA` (120 s, descarga.rs:50) produce `Timeout`, que ahora es transitorio. Una descarga que supera 120 s se reintenta con otro plazo completo: el peor caso son unos 240 s mas 1 s antes de fallar. Antes de la correccion solo se reintentaba el corte del cuerpo.
- Reproduccion: por lectura, no ejecutado (duraria cuatro minutos). El codigo lo muestra directamente: `Timeout(_)` -> `Sinred` -> `transitorio: true` -> `bajar_con_un_reintento` vuelve a llamar a `a_escritor`.
- Impacto: en una red lenta, el fallo tarda el doble. El resultado final sigue siendo correcto (`sinred`, exit 4).
- Sugerencia: no reintentar `Timeout` global, o reintentarlo solo si la descarga no llego a empezar (conexion o cabeceras).
- Origen: implementacion. Severidad MENOR: solo latencia en el fallo, sin efecto en el resultado.

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Sin CRITICOS ni IMPORTANTES.

### Pendientes de la ronda 2 que siguen abiertos (no reverificados en esta ronda)
- MEN-1 — `.old` bloqueado impide actualizar (Windows): abierto a proposito (ver ronda 2).
- MEN-5 — sin test de integracion del reintento, la redireccion y el limite: abierto a proposito. En esta ronda se ha hecho una matriz de reintento con un test temporal (ver Reproduccion), pero no se ha añadido ningun test al repo.

## Alcance
- Ficheros revisados (los del diff que casan con los patrones): crates/tc2/src/actualizar_cli.rs (`ejecutar_con_limite`, `bajar_con_un_reintento`, `fallo_de_escritura`, `comprobar_nuevo`, tests), crates/tc2/src/descarga.rs (`Fallo`, `a_escritor`, `texto`), crates/tc2/tests/actualizar_cli.rs (lectura del fixture y del test de corte).
- Contexto adicional pedido: ninguno. Llamadores de JavaScript y `.d.ts`: no aplica, el diff no toca `#[wasm_bindgen]` ni codigo `wasm32`. Es CLI nativo.
- Limite JS-WASM: sin cambios.

## Reproduccion
- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-027-r3-rust`, rama `feature/task-027-f2-12-taskctl-actualizar-cli-y-aviso-de`, HEAD 6acf391 (codigo identico a e3ac6d9). Borrado al terminar; el repo original no se toco (ni checkout ni switch, ni commits).
- Build: `cargo build -p tc2 --examples --locked` → Finished, exit 0.
- Formato y lints: `cargo clippy --all-targets --locked -- -D warnings` → "No issues found", exit 0. `cargo fmt --check` → exit 0.
- Suite: `cargo test -p tc2 --locked`, una pasada sobre e3ac6d9 → exit 0. Lib: 65 passed, 0 failed (incluido `ejecutar_con_limite_cumple_el_plazo_con_un_nieto`). Binarios de integracion: todos en verde (incluido `tests/actualizar_cli.rs`; los tests E2E de GitHub y GitLab quedan `ignored` por diseno, requieren credenciales). Ningun corte de red en esta ejecucion: las pruebas que tardan (20 s, 22 s, 27 s) pasaron sin reintento.
- Sin `unwrap`, `expect`, `panic!`, `unreachable!`, `todo!` ni `unsafe` en el codigo de produccion de actualizar_cli.rs, descarga.rs y plataforma/http.rs (grep sobre lo anterior a `#[cfg(test)]`; solo `unwrap_or_default`, `unwrap_or`, `saturating_*`).
- Mutacion 1 (MEN-6): `recv_timeout(restante())` → `recv()` en ambos lectores; el test del nieto FALLA (`tardo 29.36 s`). Restaurado.
- Matriz del reintento (test temporal en el clon, llamando a `bajar_con_un_reintento` contra servidor local en 127.0.0.1):
  - 500: 1 peticion, `Http { estado: 500 }`, 0.10 s. No se reintenta.
  - 404: 1 peticion, `NoPublicada { estado: 404 }`. No se reintenta.
  - 302 a `https://evil.example/x`: 1 peticion, `Redireccion`. No se reintenta.
  - 200 con `Content-Length` de 64 MiB + 1: 1 peticion, `Grande { limite: 67108864 }`. No se reintenta.
  - https contra servidor HTTP plano: 2 conexiones, `Sinred { transitorio: true }`. SI se reintenta (ver MEN-8).
  - sin servidor (conexion rechazada): 2 intentos, `Sinred { transitorio: true }`, 5.1 s en total (no investigado por que tanto; el reintento de 1 s esta incluido).
  - No probado: certificado TLS invalido (sin infraestructura de certificados) y el limite a mitad del cuerpo. Por lectura: el cuerpo se lee con `take(limite + 1)` y el exceso devuelve `Grande` (descarga.rs:348-357), que no es transitorio.
- Prueba del hijo con nieto (MEN-7): test temporal en el clon, `cmd /c "start /b ping -n 20 127.0.0.1 & echo hola"`, limite 8 s → `tardo 8.13 s`, `success=true`, `stdout=""`.
- Hilos y procesos: los lectores terminan al cerrarse la tuberia. Con el hijo muerto y sin nieto, no quedan hilos. Con un nieto vivo (test del plazo), el nieto no se mata (decision documentada) y los dos hilos lectores siguen bloqueados hasta que el nieto acabe; el hilo no se espera mas alla del plazo.
- Attestation y `gh` real: no se ejecutaron (sin sesion).
- Mutacion no hecha: quitar el `kill` de la rama `Err(e)` de `try_wait` no rompe ningun test (sin cobertura; MENOR de cobertura, no de codigo: la rama es correcta por lectura).
- Nota de limpieza: los tests temporales de la matriz crean carpetas `tmp-r3-*` en el directorio temporal del sistema; se borran al terminar.

## Revisado sin hallazgos
- Limite de ejecucion: `try_wait` con error mata y recoge al hijo; el plazo se mide desde antes de leer las tuberias; `kill` y `wait` tras el plazo; el margen de 100 ms para recoger la salida es acotado.
- `fallo_de_escritura`: `PermissionDenied` → `sin-permiso`; el resto → `escritura`. Sin cambios en esta ronda.
- Reintento: `transitorio` cubre solo fallos de transporte (sin conexion, rechazada, tiempo agotado, E/S). 4xx, 5xx, redireccion no permitida, limite de tamano y respuesta invalida no se reintentan (matriz).
- `bajar_con_un_reintento`: el fichero se vacia (`set_len(0)`) y se rebobina antes del segundo intento; la pausa de 1 s es solo tras un fallo transitorio.
- Sin unwrap/panic/unsafe en produccion (ver Reproduccion).

## Notas (no van en la tabla)
- La doc de `ejecutar_con_limite` (actualizar_cli.rs:574-575) promete devolver lo leido hasta el plazo; con la implementacion actual, si el lector no ve EOF no devuelve nada (MEN-7). Es la misma raiz.
- Los MENOR de la ronda 2 que se corrigieron (MEN-2, MEN-3, MEN-4) no se han vuelto a verificar en esta ronda; no estan en su diff.

## Lecturas fuera del contexto permitido
- `crates/tc2/src/plataforma/http.rs`: funcion `clasificar` (lineas 343-359) y `FalloHttp` (166-180). Para entender como se clasifica cada error de ureq en `Fallo::Sinred` (MEN-8 y MEN-9).
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ureq-3.5.2/src/error.rs` (lineas 196-220): `into_io` y `From<io::Error>`. Para confirmar que los errores de ureq en el cuerpo llegan como `io::Error` (MEN-8).
