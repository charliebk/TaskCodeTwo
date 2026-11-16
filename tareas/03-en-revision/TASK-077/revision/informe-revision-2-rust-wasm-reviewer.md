# Informe de revision — TASK-077 (ronda 2)

- Commit revisado: 2fae3e4b7d864f66f5668dac349b21c345ce3f9a
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

Veredicto que propone el revisor: **aprobada** (0 CRITICO, 0 IMPORTANTE, 1 MENOR nuevo; los 3 MENOR de la ronda 1 quedan corregidos y verificados).

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | corregido | crates/tc2/src/plataforma/personas.rs:100 | implementacion |
| MEN-2 | MENOR | corregido | crates/tc2/src/identidad.rs:229 | implementacion |
| MEN-3 | MENOR | corregido | crates/tc2/src/identidad.rs:107 | implementacion |
| MEN-4 | MENOR | abierto | crates/tc2/src/git/remoto.rs:185 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### Verificacion de las correcciones de la ronda 1

- **MEN-1 (correos sin distinguir mayusculas fuera de ASCII): corregido.** `mismo_correo` compara con
  `to_lowercase` en los dos lados (`personas.rs:100`) y lo usan los tres sitios (`personas.rs:200`
  duplicados y `:245` `por_correo`). No queda ningun `eq_ignore_ascii_case` sobre correos. El test
  `correos_sin_distinguir_mayusculas_fuera_de_ascii` nombra las dos entradas y cubre `É@x` / `é@x` y
  `por_correo("é@X")`. Mutacion: volver a `a.eq_ignore_ascii_case(b)` pone rojo ese test.
- **MEN-2 (cache sin tope y login sin longitud maxima): corregido.** `leer` lee con
  `File::open(..).take(LIMITE_CACHE + 1)` y rechaza (`None`) si el texto pasa de 64 KiB
  (`identidad.rs:229-244`). `login_valido` acota a `LOGIN_MAX = 255` caracteres (`tarea/modelo.rs:213-222`),
  y `Asignado::persona` y `asignable` lo usan. Mutacion: quitar el bloque `if texto.len() ... return None`
  pone rojo `cache_con_tope_y_login_con_longitud_maxima` ("mayor que el tope: ausente"). Con 255 caracteres
  multibyte (510 bytes) el tope de 64 KiB no los corta, asi que el login valido cabe.
- **MEN-3 (cuenta no-User sale como «es un bot»): corregido.** Existe `NoAsignable::NoPersona`
  (`identidad.rs:107`) con `nombre() == "no-persona"` (`:121`) y mensaje propio `CUENTA_NO_PERSONA` con
  `{tipo}` (`:133`). `asignable` distingue `"User"`, `"Bot"` y el resto (`:161-164`). El test
  `otro_tipo_de_cuenta_es_no_persona_y_no_bot` cubre `Organization` y `Mannequin` y comprueba la linea
  `cuenta=no-persona`. El caso `Bot` sigue dando `cuenta=bot`. GitLab solo produce `"Bot"` o `"User"`
  (`plataforma/gitlab.rs:130`), asi que el cambio no altera ese proveedor.

### MEN-4 — `traer_rama` clasifica «rama inexistente» como «sin red»

- Donde: crates/tc2/src/git/remoto.rs:185 (`traer_rama`) y :175-181 (`fallo_fetch`)
- Que pasa: `fallo_fetch` manda `FalloRemoto::RamaInexistente` a `sinred(op)`. `traer_rama` se usa para
  traer la rama base, asi que si `origin` no tiene `develop` (p. ej. se borro o se renombro en el remoto),
  `tc2 identidad` sale con `sinred` y el mensaje «reintenta con red», cuando la red funciona. Antes de la
  correccion, con `origin/develop` local presente, la orden seguia.
- Reproduccion: en el clon, `git fetch -q --end-of-options origin rama-que-no-existe-xyz` (git 2.56,
  `LC_ALL=C`) devuelve `fatal: couldn't find remote ref rama-que-no-existe-xyz`; `clasificar_fallo_remoto`
  lo reconoce como `RamaInexistente` (`remoto.rs:91-92`), y `fallo_fetch` (`remoto.rs:177`) lo convierte en
  `sinred`. La prueba de la clasificacion es esa cadena y el `match`; no se ha ejecutado `tc2 identidad`
  de extremo a extremo porque exige terminal.
- Impacto: diagnostico erroneo en un caso poco frecuente (el remoto no tiene la rama base). Quien lo ve
  reintenta sin resultado. No pierde datos ni deja estado a medias: `sinred` no escribe la cache.
- Sugerencia: en `traer_rama`, tratar `RamaInexistente` aparte (un informe `denegada` con motivo
  `rama-no-existe` o un mensaje que nombre la rama base), o al menos no decir «sin red». Es MENOR: no
  bloquea. Se introduce con esta correccion (la llamada nueva en `orden_paso`), por eso se documenta aqui.
  Si la persona decide no corregirlo, basta marcarlo `aceptado` con este motivo.

## Alcance

- Ficheros revisados (del diff de la ronda, los que casan con los patrones): `crates/tc2/src/git/remoto.rs`,
  `crates/tc2/src/identidad.rs`, `crates/tc2/src/plataforma/personas.rs`, `crates/tc2/src/tarea/mod.rs`,
  `crates/tc2/src/tarea/modelo.rs`, `crates/tc2/tests/identidad.rs` (tests del diff, leidos del clon).
- Commit: el `HEAD` de la rama al revisar era 8ea5cb4 (peticion). Entre 2fae3e4 y 8ea5cb4 solo cambian
  ficheros de `tareas/` (peticiones e informes); ningun `.rs`, asi que el codigo revisado es el de 2fae3e4.
- Contexto adicional pedido: ninguno.
- Limite JS-WASM: sin cambios de firma exportada con `#[wasm_bindgen]`; estos crates no tienen objetivo
  `wasm32` en el diff. No se pidio llamador TypeScript.

## Reproduccion

- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-077-r2-rust`, rama
  `feature/task-077-f2-13-identidad-de-la-persona-desde-la-p`, HEAD 8ea5cb4 (codigo = 2fae3e4). Borrado al terminar.
- Build: `cargo test -p tc2 --locked` compila sin errores ni avisos.
- Formato y lints: `cargo fmt --check` sin diferencias; `cargo clippy --all-targets --locked -- -D warnings`
  sin avisos (exit 0).
- Suite: `cargo test -p tc2 --locked`: 21 suites, 0 fallos. Lib 83 tests pasan; `tests/identidad.rs` 5/5;
  `tests/tarea.rs` 7 pasan y 1 ignorado (no lo he comparado con la base). No hubo cuelgues de 127.0.0.1 que obligaran a
  repetir. Linea base: la misma pasada, en verde para el commit revisado.
- Casos construidos:
  - `git fetch --end-of-options origin <rama-inexistente>`: stderr `couldn't find remote ref` (MEN-4).
  - `git fetch -q --end-of-options origin develop` en el clon: exit 0, `origin/develop` actualizado (camino feliz).
- Mutacion (cada una sobre el fichero de la correccion, restaurada despues por borrado del clon):
  - MEN-2: quitar el corte `texto.len() > LIMITE_CACHE` en `identidad.rs` → `cache_con_tope_y_login_con_longitud_maxima`
    falla ("mayor que el tope: ausente").
  - MEN-1: `mismo_correo` con `eq_ignore_ascii_case` → `correos_sin_distinguir_mayusculas_fuera_de_ascii` falla.
  - MEN-3: no se mutó; el test `otro_tipo_de_cuenta_es_no_persona_y_no_bot` compara la variante y la linea
    de contrato exactas, y la suite pasa en verde con el codigo actual.
- Medidas: no aplican.

## Revisado sin hallazgos

- `traer_rama` usa el puerto `Git` (`correr` → `git.ejecutar`), no un `Command` directo, y pasa
  `--end-of-options` antes de `REMOTO` y `rama` (`remoto.rs:191`). El nombre de rama sale de `config::BASE`
  sin `origin/`, y no puede empezar por `-` porque ya va detras del separador.
- Clasificacion: `traer_rama` reutiliza `fallo_fetch`, que es el mismo mapeo que tenia `traer`, movido sin
  cambios de comportamiento (`traer` solo cambia para llamar a `fallo_fetch`). Lo unico que se cuestiona es
  MEN-4.
- Sin `unwrap`, `expect`, `panic!`, indexado ni `unsafe` en el codigo nuevo de `remoto.rs`,
  `identidad.rs` y `personas.rs` (produccion). El workspace tiene `unsafe_code = "forbid"`. Los `unwrap_or`
  y `unwrap_or_default` del diff son seguros. En los tests, `usize::try_from(...).unwrap_or(0) - texto.len() - 2`
  resta sin comprobar, pero `texto` es pequeno y no hay riesgo de desbordamiento.
- `leer` de la cache: `take(LIMITE+1)` con `read_to_string` acota la lectura; el error de UTF-8 o de `open`
  da `None`; sin `panic`.
- `Asignado::persona` y `asignable` pasan por `login_valido` (chars().count() <= 255, sin espacios ni control).

## Lecturas fuera del contexto permitido

- `.claude/.../skills/task-workflow/revision.md` (reglas comunes de revision y de la linea de veredicto).
- `crates/tc2/src/git/mod.rs` (`correr`, `Paso`, `REMOTO`): para ver que `traer_rama` pasa por el puerto `Git`.
- `crates/tc2/src/config/mod.rs` (`BASE = "origin/develop"`): para el prefijo que quita `traer_rama`.
- `crates/tc2/src/plataforma/gitlab.rs` (`tipo: if u.bot { "Bot" } else { "User" }`): para la afirmacion sobre GitLab en MEN-3.
- `crates/tc2/src/plataforma/mod.rs` (`self.tipo == "User"` en `Usuario`): para comprobar que no hay otro sitio que trate `tipo` distinto de `User`.
- `tareas/03-en-revision/TASK-077/revision/informe-revision-1-rust-wasm-reviewer.md` y la peticion de ronda 2: para los hallazgos pendientes.
