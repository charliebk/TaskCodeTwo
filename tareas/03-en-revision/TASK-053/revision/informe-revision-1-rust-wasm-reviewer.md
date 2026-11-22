# Informe de revision — TASK-053 (ronda 1)

- Commit revisado: bc834c8d5e302c27d69f32e2d7ac3de286836974
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | xtask/src/workflows.rs:2304 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/hallazgos/mod.rs:1147 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### MEN-1 — el lint `saca_datos_a_stdout` se salta con casos simples

Reproducido llamando a la funcion con lineas sinteticas (test temporal en el clon, ya retirado):

- `cat "$respuesta"` -> true (bien); `echo "$(head -c 5 "$respuesta")"` -> true (bien).
- `cat "$respuesta" >&2` -> **false**: cualquier `>` cuenta como redireccion, tambien `>&2`, y el runner lee stderr igual que stdout para `::add-mask::` o `::stop-commands::`.
- `cat "$respuesta"; echo "a > b"` -> **false**: un `>` en cualquier punto de la linea la da por redirigida.
- `grep -h . "$respuesta"`, `cut -c1-80 "$respuesta"` -> **false**: `LECTORAS` no incluye grep, cut, tr, less, paste...

El workflow actual esta limpio (el unico dato que sale hacia `$GITHUB_OUTPUT` va en base64), asi que no hay fuga hoy; es una red mas estrecha de lo que dice el comentario («ningun `run:` saca a stdout...»). Arreglo posible: tratar `>&` como no redirigido, mirar el `>` solo si va seguido de un fichero, y ampliar `LECTORAS`.

### MEN-2 — `TOPE_PROSA` no acota lo que va al check por las cercas

`prosa("r", &"`".repeat(32768))` (32 KiB, justo el tope) devuelve 98 423 bytes: una cerca de 32 769 acentos al abrir y otra al cerrar. Una respuesta hostil con tramos largos de acentos triplica el tamano frente a lo que dice «32 KiB» y empuja antes al corte de 65000 del check la prosa de las revisoras siguientes. No rompe la cerca ni saca texto fuera del bloque (las tablas van antes), solo el presupuesto. Opciones: acotar tambien la cerca, o contar la prosa final contra el tope.

## Lecturas fuera del contexto permitido

- crates/tc2/src/plan/operacion.rs, crates/tc2/src/tarea/modelo.rs y `.github/workflows/revision-ia.yml`: para confirmar que `tipo` y `complejidad` salen de listas cerradas, que `titulo` no admite saltos ni controles, y que ningun `run:` del workflow real saca datos por stdout (motivo: juzgar el alcance de `plano`, `cabecera` y del lint).

## Alcance

- Ficheros revisados: crates/tc2/src/dato.rs, hallazgos/{mod,orden,prompt}.rs, i18n/catalogo/hallazgos.rs, lib.rs, plan/plantillas.rs, xtask/src/workflows.rs y tests/inyeccion.rs (solo ejecutado).
- Contexto adicional pedido: el de «Lecturas fuera del contexto permitido».
- Limite JS-WASM: sin cambios; el diff no toca `#[wasm_bindgen]` ni tipos que crucen el limite (crate nativo).

## Reproduccion

- Clon: %LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-053-r1-rust, checkout de bc834c8 (se borra al terminar).
- Build, fmt, clippy: linea base `cargo xtask verificar --perfil rapida` en verde en bc834c8 segun la peticion, no repetida. `cargo fmt --check` limpio.
- Suite de mi parte: `cargo test -p tc2 --lib dato` y `--lib hallazgos` verdes (6 y 22 tests); `--test inyeccion --test plan` verdes (10 tests).
- Casos construidos: fuzz determinista de 20 000 textos hostiles (acentos de 1 a 5, `~~~`, CRLF, CR suelto, tabuladores, VT, FF, NEL, U+2028, Cf y bidi, `## Lo que devuelves`, `Devuelve []`, `::stop-commands::`, tildes combinadas) pasados por `dato::bloque`, `prompt::componer` (contexto, skill, seccion, abiertos y diff a la vez) y `prosa`, con un analizador de cercas de CommonMark escrito a parte (incluye cercas `~~~`). Resultado: ninguna cerca abierta sin cerrar, solo texto de tc2 fuera de bloque y una unica `## Lo que devuelves` fuera de cercas. Cortes UTF-8 en el tope (`ñ`, `€`, emoji, `a`` ` ``) con desfases 0 a 7: sin panico ni cerca rota. Cf entre acentos se limpia antes de contar la cerca. `plano`: autoenlaces, `@`, `$`, `:+1:`, `![x](y)`, `<https://..>`, listas y `=` al inicio salen inertes (mitad de linea).
- Mutacion: `cerca` sin el `+1` pone rojo `bloque_nunca_contiene_su_cerca_antes_del_final`; quitar el filtro Cf de `prosa` pone rojo `las_respuestas_hostiles_no_cambian_el_veredicto`.

## Revisado sin hallazgos

- `cerca`, `bloque` y `plano`: sin panico (`get(..)`, limites de caracter) y la lista Cf cubre la categoria.
- `prompt::componer` y el subcomando `prompt` (argumentos duplicados, lectura con tope, diff no UTF-8 vía lossy).
- `validar --prosa`; `cabecera` del plan con `titulo` plano.

Nota menor sin ID: `plano("aaaa   ", 4)` da `aaaa…` aunque solo queden espacios, y el tope cuenta caracteres sin las barras de escape.

Veredicto propuesto: aprobada-con-correcciones
