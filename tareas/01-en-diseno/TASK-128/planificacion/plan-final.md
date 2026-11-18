# Plan — TASK-128: Seguimiento de F3-11 y F3-12: aviso de solape contra uno mismo, textos al catalogo y test intermitente de progreso

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Tres cambios independientes, cada uno extiende un punto que ya existe; ningun modulo nuevo. Orden: C2, C1, C3 (de menos a mas riesgo; lo pesado es C3, que se verifica bajo carga).

Duracion estimada: 150 min
Senales: 3 criterios independientes; 2 E2E pesados que se retocan (`liberar::c5_c6_c10`, `progreso::c3`) mas 1 nuevo; verificar 10 de 10 bajo carga en Windows; cambia una firma publica (`solape::calcular`) y su uso en 3 sitios de tests.

**C2. Textos al catalogo** (extiende `crates/tc2/src/i18n/catalogo/toma.rs`)
- `AVISO_Y_MAS` y `NOTA_Y_MAS` salen de `solape.rs:157-169` y entran en el bloque `textos!` de `toma.rs` con los mismos `es`/`en`. La macro los mete en `TODOS` (`i18n/mod.rs:222-235`) y los recorre `catalogo_completo`.
- `solape.rs` usa `txt::AVISO_Y_MAS` y `txt::NOTA_Y_MAS`; se quita el comentario que justificaba la excepcion. Nadie fuera de `solape.rs` usa esas claves.

**C1. Sin aviso de solape contra uno mismo** (extiende `solape.rs` y `toma/mod.rs`)
- Fallo (`solape.rs:200`): `calcular` solo excluye por `id`. En la toma nueva `op.en_curso()` se lee en `Tomar::aplicar` (`toma/operacion.rs:223-226`) antes de que la tarea pase a en-curso: las otras tareas en curso de la misma persona cuentan como ajenas.
- `EnCurso` gana `asignado: Option<Asignado>` (ya esta en `Ficha::de`, `solape.rs:57`); `calcular` gana `propio: &Asignado`.
- Funcion pura `otras(en_curso, id, propio)` que quita la propia por id y las de la misma persona por `Asignado::misma_persona` (id numerico, `tarea/modelo.rs:202`, como decide la toma si una tarea es suya, `operacion.rs:173-176`).
- La regla vive en `solape` para que la hereden F4-1 y `review`. En `toma/`, `Destino::con_solape` recibe `&Asignado` (`quien.asignado()`, `identidad.rs:90`) en sus 3 llamadas (`toma/mod.rs:193, 235, 240`).
- Efecto: las tareas propias desaparecen tambien de la `nota:` del primer `start`, de `en-curso-modulos` y de `solape-sin-rama`; `liberar::c5_c6_c10` queda con stderr vacio.

**C3. La rotacion de `.anterior.json` deja de ignorarse en silencio** (extiende `progreso/fichero.rs`, `progreso/mod.rs`, `lib.rs`)
- Fallo (`fichero.rs:176-178`): `let _ = escribir_local(&ruta_anterior(..), ..)` tira el error (decision de TASK-084 R1-04: el progreso se escribe igual, que se mantiene). Causa probable, sin reproducir: el `rename` reintenta ~300 ms (`huella.rs:72-81`) y bajo carga un lector o el antivirus con el fichero abierto lo supera.
- `fichero::escribir` devuelve `io::Result<Anterior>` con `enum Anterior { Rotado, SinPrevio, NoEscrito }`; `Refresco::Escrito` pasa a `Escrito { anterior }` (`progreso/mod.rs:214`); `con_progreso` (`lib.rs:413-449`) anade `progreso=sin-anterior` a la linea de la transicion, sin stderr y sin cambiar codigo ni resultado. `escribir_en` (ruta de `start`) igual.
- Test `c3`: tras `reanudar`, o el `.anterior.json` es igual a `tras_pausa`, o la linea lleva `progreso=sin-anterior`. Precedente: `progreso=no-escrito`.

## Lo que el rol no cubrio

- Riesgos: si la causa de C3 fuera que `de(&r)` (`fichero.rs:176`) devuelve `None` porque no pudo abrir el actual, no habria rotacion ni marca. Se mira en el diagnostico (abajo); si no sale 10 de 10, se para y se re-planifica. No se miraron carreras con dos clones de la misma persona.
- Dominio: el ADR F1-17 (punto 11) cruza «cada rama en curso» sin eximir las propias; el criterio 1 lo restringe (adenda, D5). Tareas con asignado heredado sin id siguen avisando.
- Testing: plan minimo desde arquitectura.
- C1 obliga a cambiar `solape::c10` (ver riesgos).

## Riesgos aceptados y que los contiene

- `tests/solape.rs::c10_uno_mas_k_mas_uno_procesos` refleja el comportamiento que se corrige (Luis con su propia TASK-002 publicada espera `k = 2`): pasa a `k = 1` y solo TASK-001, en el mismo commit y explicado en el Resultado.
- API publica: `EnCurso` un campo mas y `calcular` un parametro mas; 3 llamadas en `tests/solape.rs` y el helper `en_curso()`. No compila hasta adaptarlos.
- Valor nuevo `progreso=sin-anterior` en la linea: adenda en el ADR F1-17 (progreso en vivo).
- La causa de C3 sin reproducir: diagnostico previo; sin reintentos a ciegas.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- C1: 2 nuevos y 2 que se adaptan.
  - Nuevo, ligero: unitario de `otras` (fuera la propia por id; fuera la de la misma persona por id numerico aunque cambie el login; dentro la de otra persona y la heredada).
  - Nuevo, **pesado**: E2E en `tests/solape.rs`: la misma persona con dos tareas publicadas que tocan el mismo fichero; el segundo `start` sale 0 sin claves `solape*` y con stderr vacio.
  - Se adaptan, **pesados**: `liberar::c5_c6_c10` (stderr vacio) y `solape::c10` (`k = 1`).
- C2: 1 nuevo, ligero: las claves de `AVISO_Y_MAS` y `NOTA_Y_MAS` estan en `catalogo::toma::TODOS`.
- C3: 1 nuevo y 1 que se adapta.
  - Nuevo, ligero, en `fichero.rs`: un directorio en la ruta `TASK-NNN.anterior.json` hace fallar la rotacion; `escribir` devuelve `Anterior::NoEscrito` y el actual queda escrito.
  - Se adapta, **pesado**: `progreso::c3` con la condicion de arriba.
  - Diagnostico: `progreso::c3` 10 veces seguidas con carga en paralelo; se anota si aparece `sin-anterior` y cualquier otro fallo. Carpeta en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-128-carga`, borrada al terminar.

## Lo que necesita decision de una persona

Carlos dejo dicho el 2026-11-18 que en estas cuatro tareas del H0 se acepta la opcion recomendada y sin trocear; se aplican:

- D1 como se cierra C3: (a) el codigo informa de que no pudo rotar y el test lo acepta. **Elegida.**
- D2 como se ve: (a) valor nuevo de la clave existente, `progreso=sin-anterior`, sin stderr. **Elegida.**
- D3 misma persona: (a) `Asignado::misma_persona` (id numerico), `asignado` en `EnCurso`. **Elegida.**
- D4 alcance de «no cuenta»: (a) las propias no salen en nada (aviso, nota, `en-curso-modulos`, `solape-sin-rama`). **Elegida.**
- D5 frente al ADR F1-17 punto 11: (a) se acepta el criterio y se anota como adenda del ADR. **Elegida.**

## Troceo

No se trocea aunque pasa de 60 min: Carlos pidio estas cuatro tareas sin trocear (2026-11-18). Los tres cambios son independientes y pequenos; el tiempo lo pone la verificacion bajo carga, no el codigo.
