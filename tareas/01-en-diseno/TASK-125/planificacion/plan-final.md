# Plan — TASK-125: Seguimiento de F3-10: quitar la copia vieja de la regla de actividad

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Se borra `situacion::actividad` (no se hace delegar) y se reescriben los dos comentarios de `crates/tc2/src/situacion.rs` que dicen que una actividad del futuro cuenta como de hoy. Diff minimo:

1. `situacion.rs` l. 42-48: se borran el comentario y `pub fn actividad(ficha: &Ficha, commit_rama: Option<u64>) -> Option<u64>`. Grep `actividad\(|fn actividad` en `crates/`: los unicos usos son las l. 225-230 de su propio test; no hay enlaces intra-doc a `[actividad]` (los de `liberar.rs:16` y `tablero.rs:172` apuntan a `actividad_conocida`).
2. `situacion.rs` l. 50-51: `dias_sin_actividad` se queda (la usan `ciclo/liberar.rs:36,141` y `tablero.rs:192`); solo cambia su comentario: recibe la actividad ya filtrada por `crate::ciclo::liberar::actividad_conocida`, y si `ultima` va por delante de `ahora` (solo dentro de `MARGEN_FUTURO`) da 0 porque la resta satura. El codigo no cambia.
3. Test `actividad_y_dias_sin_actividad` (l. 215-234): se quitan las aserciones de `actividad`; se conservan la de `ultima_transicion` y las tres de `dias_sin_actividad`; se renombra a `dias_sin_actividad_completos` y su comentario deja de decir que el futuro cuenta como de hoy.
4. Una asercion `actividad_conocida(None, Some(rama), AHORA) == Some(rama)` en `liberar.rs::actividad_conocida_sin_fechas_del_futuro` (l. 566), para el caso «sin fila, solo commit de rama» que cubria el test borrado.

Precedente: TASK-081 movio la regla a `ciclo::liberar::actividad_conocida`; esto termina aquella limpieza. Borrar y no delegar: delegar crearia el ciclo `situacion` <-> `ciclo::liberar` (que ya importa `dias_sin_actividad`) y dejaria dos entradas publicas para la misma regla.

Duracion estimada: 15 min
Senales: un fichero de produccion (`situacion.rs`) y una linea de test en `liberar.rs`; sin cambios de comportamiento; el coste real es la suite `tests/liberar.rs`.

## Lo que el rol no cubrio

- Riesgos, testing y dominio: sin roles propios. Con cero llamadores el riesgo es casi nulo; no se toca ninguna regla (`actividad_conocida`, `liberar.rs:59-66`, se da por buena).
- Criterio 2 (tests en verde): se ejecuta en la implementacion.

## Riesgos aceptados y que los contiene

- Un uso que el grep no vea (`use situacion::*`, macro): lo contiene la compilacion.
- Perder cobertura: el caso (None, Some) lo recupera el paso 4; la frontera de 6/7 dias y `ultima_transicion` se conservan en el test de situacion.
- Que el comentario nuevo de `dias_sin_actividad` vuelva a leerse como «el futuro cuenta como hoy»: se redacta ligado a `MARGEN_FUTURO` y a `actividad_conocida`.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- Criterio 1: 0 tests nuevos (borrar no se prueba con un test): `cargo build -p tc2` sin la funcion; grep `fn actividad\(` en `situacion.rs` vacio; grep `cuenta como de hoy` en `crates/` solo da los de `liberar.rs` ligados al margen (D1a). Ninguno pesado.
- Criterio 2: 0 tests nuevos, una asercion mas en un test existente de `liberar.rs`. `cargo test -p tc2 --lib situacion`, `--lib tablero`, `--lib liberar`, `--test liberar` (pesado) y `--test tablero`; `cargo clippy -p tc2 --all-targets` y `cargo doc -p tc2 --no-deps` (enlaces intra-doc).

## Lo que necesita decision de una persona

Carlos dejo dicho el 2026-11-18 que en estas cuatro tareas del H0 se acepta la opcion recomendada y sin trocear; se aplican:

- D1 comentarios de `liberar.rs` («dentro del margen cuenta como de hoy», ligados a `MARGEN_FUTURO`): (a) el criterio se aplica solo a los de `situacion.rs`, sin margen, que contradicen la regla; los de `liberar.rs` se quedan. **Elegida.**
- D2 caso (None, Some(rama)): (a) una asercion en `liberar.rs:566`. **Elegida.**
- D3 borrar o delegar: (a) borrar. **Elegida.**
