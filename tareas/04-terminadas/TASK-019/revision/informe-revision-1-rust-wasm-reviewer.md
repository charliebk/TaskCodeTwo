# Informe de revision — TASK-019 (ronda 1)

- Commit revisado: f7f9446522807d5cea787a20dc1286abdf525a6d
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/alta.rs:215 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/lib.rs:447 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### MEN-1 — `siguiente_id` distingue mayusculas, pero la rama de estado las trata como iguales

`numero()` solo reconoce `TASK-<cifras>` en mayusculas, mientras que `transaccion::choque` rechaza rutas que solo difieren en mayusculas. Si alguien deja a mano en la rama de estado una carpeta `task-013/` (minusculas), `siguiente_id` no la cuenta, calcula `TASK-013`, la transaccion lo rechaza y todos los `new` siguientes fallan igual, porque el maximo no cambia.

Reproduccion: remoto con `TASK-001..012` y una carpeta `task-013/tarea.md` creada con `git mktree` y publicada a mano. Luego `tc2 new "x"` sale con codigo 3: `new rechazada motivo=ruta-duplicada` y el diagnostico `TASK-013/tarea.md solo difiere en mayusculas de task-013/tarea.md`. Sin reintento posible hasta que alguien corrija la rama. El diagnostico es claro y solo se llega editando la rama a mano, de ahi MENOR. Arreglo: comparar el prefijo sin distinguir mayusculas al calcular el maximo (o saltar los numeros ocupados).

### MEN-2 — mensaje de `titulo-invalido` incompleto

`orden_new` rechaza tambien tabuladores, otros caracteres de control, U+2028 y U+2029 (comportamiento que pedia el plan), pero el diagnostico dice solo `el titulo no puede estar vacio ni llevar saltos de linea`. Con `tc2 new $'tab\there'` sale ese mensaje, que no explica el rechazo. Basta decir «ni caracteres de control».

## Alcance
- Ficheros revisados: crates/tc2/src/alta.rs, crates/tc2/src/git/remoto.rs, crates/tc2/src/lib.rs, crates/tc2/tests/alta.rs (mas, para entender el flujo, estado/transaccion.rs y estado/lectura.rs: ver lecturas fuera del contexto).
- Contexto adicional pedido: ninguno mas alla de lo anotado abajo.
- Limite JS-WASM: sin cambios; no hay `#[wasm_bindgen]` en el diff (CLI nativo).

## Reproduccion
- Clon: %LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-019-r1-rust, rama feature/task-019-f2-4-reserva-de-ids-por-push-atomico (HEAD ce0ffda; el codigo es el de f7f9446, ce0ffda solo anade la peticion).
- Build/suite: `cargo xtask verificar --perfil completa` en verde (suite completa, build reproducible con mismo sha256). `cargo fmt --check` y `cargo clippy --all-targets --locked -- -D warnings` limpios. No aplica wasm32 (el crate es un CLI nativo).
- Estabilidad: `cargo test -p tc2 --test alta` 5 veces seguidas, 3/3 en verde cada vez (12-15 s).
- Casos construidos con el binario real: 4 clones x 3 `new` simultaneos desde cero (la rama de estado no existia): 12 IDs consecutivos TASK-001..012, sin repetidos ni `agotada`. Titulos con `{id}`, `{fecha}`, `\`, `"`, `|`, 5000 caracteres, espacios en los bordes, tabulador, U+202E; `new` sin titulo, `new "x" --json` (uso 2, coherente con el USO), carpeta `task-013` en minusculas (MEN-1).
- Mutacion (cada una puso rojo algun test): maximo + 1 cambiado por recuento (unit `siguiente_id_es_el_maximo_mas_uno`); quitar U+2028/U+2029 (unit `anotacion_ida_y_vuelta`); quitar el escape del titulo (unit de plantilla); `exige_red` a false (E2E `sin_red_falla_sin_dejar_nada`); quitar `reference already exists` de `es_contencion` (unit `contencion_frente_a_denegacion` y, en las 3 pasadas del E2E `dos_clones_crean_a_la_vez...`, el E2E tambien falla con «reference already exists ... no se reintenta»).
- Medidas: no aplican.

## Revisado sin hallazgos
- Calculo de ID: maximo + 1 sobre el estado fresco de cada intento; `checked_add`, desbordamiento de `u32` y nombres que no casan (`TASK-1a`, `TASK-`, `TASK-+9`) cubiertos. `TASK-1000` sin renumerar.
- `Cell<Option<u32>>`: el ID que se informa es el del intento que publico (`aplicar` se llama una vez por intento y las pendientes usan otras instancias).
- Plantilla: escape de `\` y `"`; titulo sustituido el ultimo, asi que `{id}` o `{fecha}` en el titulo no se expanden (verificado con el binario). Fechas civiles verificadas a mano (11016 = 2000-02-29, 20772 = 2026-11-15).
- Sin `unwrap`, `expect`, `panic!` ni indexado en el codigo de produccion del diff; sin `unsafe`.
- `es_contencion` con `reference already exists`: necesario de verdad (la mutacion rompe el E2E).
- Divergencia menor con el plan, sin efecto: el plan citaba `tests/alta_concurrencia.rs` y los E2E viven en `tests/alta.rs`.

## Lecturas fuera del contexto permitido

- crates/tc2/src/estado/transaccion.rs y crates/tc2/src/estado/lectura.rs: para comprobar que `aplicar` se llama una vez por intento, el papel de `choque` y que `rutas()` incluye las pendientes (necesario para juzgar `Cell` y MEN-1).
- crates/tc2/src/main.rs y crates/tc2/tests/comun/mod.rs: para saber si el fixture crea ya la rama de estado (no la crea: el E2E cubre la carrera de primera creacion).
