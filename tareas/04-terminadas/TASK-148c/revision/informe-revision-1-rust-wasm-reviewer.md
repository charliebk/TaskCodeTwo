# Informe de revision — TASK-148c (ronda 1)

- Commit revisado: 56223308c105aa55a88b7d10ae2cb92dd1133ef7
- Revisor: general-purpose (sonnet) con rust-wasm-reviewer
- Veredicto: aprobada con correcciones

Veredicto propuesto: **aprobada** (con un MENOR sin bloqueo, que se puede aceptar o cerrar con una asercion).

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/tests/comun/http_mock.rs:212 | implementacion |

**MEN-1. La variante `Corte::AntesDeCabeceras` del doble nuevo no la fija ningun test.**
Mutante: `if respuesta.corte != Some(Corte::AntesDeCabeceras)` pasa a `if true` (el mock envia
las cabeceras aunque la variante prometa cerrar sin enviar nada). Resultado: `soporte::http_mock_corta_una_vez_y_sirve_bytes_exactos`
solo ejerce `AMitadDelCuerpo`, y `actualizar_cli::un_corte_en_la_descarga_del_binario_se_reintenta_una_vez`
(caso `corte-antes`) sigue **verde** (1 passed), porque al cliente le da igual ver «cabeceras con
Content-Length y cero bytes» que «nada»: reintenta igual. La rama que el comentario del enum
describe («el cliente no ve cabeceras») queda sin red en el helper compartido. Con el `Servidor`
propio anterior pasaba lo mismo respecto a ese mutante, asi que no es una regresion de la
migracion; es una brecha del codigo nuevo de `http_mock`. Arreglo minimo: segundo tramo en
`soporte.rs` con `Simulada::bytes(..).cortada(Corte::AntesDeCabeceras)` que compruebe que
`read_to_end` devuelve 0 bytes.

Sobre el fallo `identidad::cuenta_fuera_de_lista_bot_o_web_flow_no_toca_la_cache` con `sinred`: **no creo que lo
introduzca la migracion** (ver Reproduccion > Casos construidos). No se abre hallazgo.

## Regla 4D

- Duplicacion: cumple (`git grep` de `TcpListener::bind`, `fn atender`, `struct Servidor`, `incoming()` en `crates/tc2/tests`: solo `comun/http_mock.rs` sirve; los otros `bind` —actualizar_cli.rs:218, identidad.rs:317, identidades.rs:407, plataforma.rs:524— son puertos que se abren y se sueltan para obtener uno cerrado, no servidores)
- Desviacion: cumple (solo cambian tests; ninguna firma publica ni de produccion tocada; `Simulada::json` conserva su firma, `cuerpo` pasa a `Vec<u8>` y los usos externos compilan)
- Diluvio: cumple (un enum de dos variantes y dos metodos pequenos; `servir()` en actualizar_cli es un helper de 8 lineas que evita repetir el bucle, no una capa nueva)
- Disfraz: cumple salvo lo anotado en MEN-1 (variante sin test que la mate); el resto de tests migrados mantienen asserts reales (mutados mas abajo)

## Alcance
- Ficheros revisados: crates/tc2/tests/actualizar_cli.rs, comun/http_mock.rs, identidad.rs, init.rs, soporte.rs (todos son tests; ningun fichero de produccion en el diff)
- Contexto adicional pedido: ninguno
- Limite JS-WASM: sin cambios (no hay `#[wasm_bindgen]` en el diff)
- Asercion por asercion: `p.iter().all(|r| r == "/SHA256SUMS")` pasa a `r.linea == "GET /SHA256SUMS"` (igual o mas fuerte: incluye el metodo); `contains("/<fichero>")` a `any(r.linea == "GET /<fichero>")`; los conteos `al_binario == 2` / `== 1` se conservan con `r.linea == "GET /<fichero>"`; `peticiones() == ["GET /user"]` pasa a `recibidas().map(linea) == ["GET /user"]`. Ninguna asercion se debilita.

## Reproduccion
- Clon: %LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-148c-r1, detached en 5622330 (borrado al terminar); CARGO_TARGET_DIR dentro del clon.
- Build: `cargo build -p tc2 --examples --locked` (nativo): ok. No aplica wasm32 (solo tests).
- Formato y lints: `cargo fmt --check` ok; `cargo clippy -p tc2 --all-targets --locked -- -D warnings` ok, sin avisos.
- Suite: no se repite la completa (linea base de la peticion: 776 ok, 13 ignorados, 1 fallo `sinred`). Binarios ejecutados uno a uno, todos en verde: soporte 10 ok; actualizar_cli 11 ok (48 s); init 25 ok (94 s); identidad 7 ok (70 s); doctor 12 ok (178 s); identidades 12 ok; plataforma 7 ok; plataforma_gitlab 6 ok; prs_abiertos 4 ok; revision 5 ok; tablero 6 ok.
- Casos construidos (sinred): el binario `identidad` ejecutado 3 veces mas por mi, 7 ok cada vez (70 s, 45 s, 45 s); sumadas las 3 del autor, 6 de 6 en verde. Razones para no atribuirlo a la migracion: (1) el protocolo del mock para una respuesta completa no cambia (sin FIN, `read_to_end` con plazo de 30 s, un hilo por conexion; es el mismo que ya tenia el `Servidor` de identidad.rs y el de `comun`); (2) la migracion **anade** `serie()` a los tests de identidad, que en `develop` no la tenian (`git show origin/develop:crates/tc2/tests/identidad.rs | grep serie` vacio), asi que reduce la concurrencia contra la red local, que es la causa documentada en TASK-115; (3) el cliente no cambia (solo tests); (4) el mismo sintoma salio en TASK-148b con `prs_abiertos`, sin tocar identidad. Es la intermitencia conocida de Windows con carga alta (la suite entera ejecuta ~48 binarios, algunos de mas de 3 minutos).
- Mutacion (todas sobre el commit revisado, restauradas tras cada una):
  - M1 cuerpo con `from_utf8_lossy` en `Simulada::bytes`: rojo (`soporte.rs:181`).
  - M2 `AMitadDelCuerpo` envia el cuerpo entero: rojo (`soporte.rs:182`).
  - M3 sin `shutdown(Write)` en el corte: rojo (`soporte.rs:174`, el plazo de 5 s agota).
  - M4 `AntesDeCabeceras` envia cabeceras: **verde** (MEN-1).
  - M5 quitar `.cortada(corte)` en `un_corte_en_la_descarga_del_binario_se_reintenta_una_vez`: rojo (`actualizar_cli.rs:496`), la migracion no debilita el test del corte.
  - M6 sustituir el `302` por `200` en `redireccion_no_permitida_no_se_sigue_ni_reintenta`: rojo (`actualizar_cli.rs:527`), la redireccion sigue probando lo mismo.
- Medidas: no aplican.

## Revisado sin hallazgos
- `Simulada::bytes` / `json`: los llamadores antiguos (`doctor`, `identidades`, `plataforma*`, `prs_abiertos`, `revision`, `tablero`) compilan y pasan sin cambios.
- Cola de respuestas de `Mock::responder` (primera cortada, segunda entera; la ultima se repite): comprobado por el test de soporte y por el de reintento (2 peticiones al binario).
- Cuerpo grande de `binario_mayor_que_el_limite_se_rechaza` (`Simulada` se clona por peticion): sin problema, 1 peticion al binario.
- `serie()` anadido en todos los tests que arrancan un mock en identidad, init y actualizar_cli; los que no lo arrancan (`actualizar_sin_cli_es_uso`) no lo necesitan.
- Linea de estado `HTTP/1.1 <n> X` (antes `200 OK`/`404 Not Found`): el cliente la acepta; ningun test comprueba el texto de motivo.

## Lecturas fuera del contexto permitido

- `git diff origin/develop..HEAD` de los cinco ficheros y `git show origin/develop:crates/tc2/tests/identidad.rs`: para contrastar con el estado anterior (sirve el `serie()` y las aserciones); `crates/tc2/src` por grep de `timeout` para valorar los plazos del cliente en el analisis de `sinred`.
