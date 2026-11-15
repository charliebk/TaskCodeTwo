# Informe de revision — TASK-018 (ronda 2)

- Commit revisado: 09e487ed1a5f3f43221c6cb4a968ea0549237c73
- Revisor: rust-wasm-reviewer (agente general-purpose, haiku)
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| R2-01 | IMPORTANTE | corregido | crates/tc2/src/estado/transaccion.rs (`intento`: `choque_de_mayusculas` sale antes de `pendientes::resolver`) | implementacion |

### R2-01 — Una pendiente que choca en mayúsculas con el remoto bloquea todas las escrituras del clon (introducido por la corrección de la ronda 1)

Clon A escribe `notas/Mayus.md` sin red (pendiente); clon B publica `notas/mayus.md`; A vuelve a tener red y escribe `notas/g.md`, `notas/z.md`, `notas/y.md`: las tres salen `rechazada motivo=ruta-duplicada`, la pendiente sigue en `refs/taskcode/pendientes/000001` y el remoto no recibe nada de A. Causa: `choque_de_mayusculas` devuelve `Intento::Hecho` antes de `pendientes::resolver`, así que la pendiente se re-aplica en cada operación. Bloqueo permanente sin pérdida ni corrupción. Propuesta: resolver como rechazada la pendiente que choca, o mirar el choque solo sobre los cambios de la operación actual.

## Estado tras la ronda 2 (lo anota quien implementa)

- R2-01 corregido: el choque por mayúsculas se mira por etapas; una pendiente cuyos cambios chocan con lo ya publicado va a rechazadas (no se superpone), y solo la operación actual devuelve `ruta-duplicada`. Test `estado_bordes::pendiente_que_choca_por_mayusculas_no_bloquea_el_clon` (la pendiente a rechazadas y las dos escrituras siguientes publicadas), rojo con el mutante que quita la guarda.
- Notas sin hallazgo: `COM¹`-`COM³`/`LPT¹`-`LPT³` y la carrera del `Drop` de `Temporal` (los temporales se crean siempre bajo el cerrojo de la transacción) quedan como están.

## Verificado de las correcciones

- Cerrojo en `montar`: sin autobloqueo (suelta antes de `transaccion` y vuelve a entrar); mutante sin `drop(cerrojo)` → 6 fallos en `estado_bordes`.
- `ruta_valida` y `reservado_en_windows`: correctos (CON, PRN, AUX, NUL, COM1-9, LPT1-9; COM0 y COM10 permitidos; final `.` o espacio; `<>"|?*`).
- `choque_de_mayusculas` sobre la propia operación: correcto. El fallo es R2-01.
- Pendiente con error → rechazada: cubierto y en verde.
- Patrones de credenciales: SSH, GitHub, 401/403 cubiertos; permiso local da `SinRed`.

## Notas sin hallazgo

- `reservado_en_windows` no cubre `COM¹`-`COM³` ni `LPT¹`-`LPT³` (no introducido por la corrección).
- Quitar `access denied` hace que un servidor que diga solo eso caiga en `SinRed` (no reproducido).
- `Drop` de `Temporal` borra la carpeta padre si queda vacía: posible carrera fuera del cerrojo (no verificado).

## Verificación ejecutada

- Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-018-r2-rust`, en 09e487e.
- `cargo test --locked -p tc2`: 41 pasados (12 suites, 111 s). `cargo clippy --locked --all-targets -p tc2 -- -D warnings`: limpio.
- Reproducción de R2-01 con un test temporal: `PASO3 notas/g.md: estado rechazada operacion=anotar motivo=ruta-duplicada` (igual con z e y), pendiente 000001 sin resolver.
- Mutante: `drop(cerrojo)` comentado en `montaje.rs` → 6 fallos en `estado_bordes`. Restaurado.
- Limpieza: el revisor no pudo borrar su carpeta (permiso denegado); la borró después quien implementa.

## Lecturas fuera del contexto permitido

`estado/cerrojo.rs` (para el cerrojo de `montar`) y `tests/comun/mod.rs` (fixture de la reproducción).
