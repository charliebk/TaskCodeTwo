# Informe de revision — TASK-019 (ronda 1)

- Commit revisado: f7f9446522807d5cea787a20dc1286abdf525a6d
- Revisor: code-quality-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MENOR-1 | MENOR | abierto | crates/tc2/src/alta.rs:136 | implementacion |
| MENOR-2 | MENOR | abierto | crates/tc2/src/lib.rs:125 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Alcance: pase generico completo (el diff de rust-wasm-reviewer tambien se reviso, con la peticion de calidad solo embebiendo el ADR). Profundidad: `alta.rs`, `lib.rs`, `remoto.rs` y `tests/alta.rs` a fondo.

Puerta determinista (clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-019-r1-calidad`, HEAD ce0ffda): `cargo xtask verificar --perfil completa` en verde, exit 0 (fmt, clippy, suite completa, build reproducible con el mismo sha256 en dos rutas), unos 167 s.

Reproduccion empirica sobre el binario real (`tc2.exe` debug):
- Tres clones, dos `new` cada uno a la vez, contra un remoto bare sin rama de estado, 3 rondas: los 18 `new` salen 0, `TASK-001`..`TASK-006` consecutivas en cada ronda, 0 merges. Nunca salio 5.
- Titulos: espacios laterales, `a{fecha}{id}\"b`, emoji y ñ, 20000 y 31000 caracteres, `--json` como titulo, `--` y argumentos de mas dan `uso` (2); tabulador, DEL, U+2028 dan `titulo-invalido`. Todos los validos publican con `id` igual a su carpeta.
- `tareas/` montada: con un fichero modificado sale `ok ... tareas=al-dia`/`con-cambios` y el cambio local queda intacto; con una `TASK-009/` sin versionar en local y otra publicada en remoto, se publica `TASK-009` y no se pisa nada (`tareas=con-cambios`).
- Rama de estado inexistente: se crea en el primer `new` (tambien con dos clones a la vez).
- `.git/taskcode-estado.lock` queda como fichero tras la operacion pero no bloquea la siguiente (es el cerrojo, no un resto).

Mutantes (con el test concreto): `max` por `count` (cae), `exige_red=false` (cae `sin_red_falla_sin_dejar_nada`), quitar el renombrado `comando` (cae), quitar `reference already exists` (cae el unit y cae el E2E de dos clones, 3 de 3 veces), quitar el escape de `"` o de `\` (cae), `id`→`idx` en la salida (cae). Sobrevive: quitar U+2029 (MENOR-1).

### MENOR-1 — U+2029 sin test: la proteccion se puede quitar sin que nada se ponga rojo
- Donde: crates/tc2/src/alta.rs:136
- Que pasa: `titulo_valido` rechaza U+2028 y U+2029, pero `anotacion_ida_y_vuelta` solo prueba U+2028.
- Reproduccion: en el clon, `matches!(c, '\u{2028}' | '\u{2029}')` pasa a `matches!(c, '\u{2028}')`; `cargo test -p tc2 --lib alta` da 4 passed, 0 failed.
- Impacto: una regresion que deje pasar U+2029 (un salto de linea para YAML dentro de `titulo:`) no la detecta ningun test. Anadir `assert!(!titulo_valido("a\u{2029}b"))`.

### MENOR-2 — El mensaje de `titulo-invalido` describe menos de lo que rechaza
- Donde: crates/tc2/src/lib.rs:125
- Que pasa: el diagnostico dice «no puede estar vacio ni llevar saltos de linea», pero tambien se rechazan tabuladores, DEL y otros caracteres de control. Ademas `new a b` (argumentos de mas) devuelve `motivo=sin-titulo`, que no describe el caso.
- Reproduccion: `tc2 new $'a\tb'` sale `new uso motivo=titulo-invalido` con ese texto, y `tc2 new a b` sale `new uso motivo=sin-titulo`.
- Impacto: quien teclea un tabulador no sabe por que falla. Decir «ni caracteres de control» y, si se quiere, un motivo propio para los argumentos de mas (el titulo va entre comillas).

Sin hallazgos CRITICO ni IMPORTANTE: la reserva se apoya en `estado::transaccion` sin mecanismo nuevo, el ID se calcula sobre el estado fresco en cada intento, `reference already exists` como contencion queda documentado en la adenda del ADR y cubierto por el E2E de dos clones.

## Lecturas fuera del contexto permitido

- crates/tc2/src/alta.rs, crates/tc2/src/lib.rs, crates/tc2/src/git/remoto.rs, crates/tc2/tests/alta.rs: el contexto permitido solo listaba el ADR, y la tarea pedia reproducir el comportamiento del CLI completo.
- crates/tc2/src/estado/transaccion.rs (grep de `comando`): para comprobar que el renombrado a `new` no es redundante.
