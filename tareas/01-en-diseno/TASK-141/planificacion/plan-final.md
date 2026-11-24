# Plan — TASK-141: Auditoria 4D: A3 plan, importar y alta (5 IMPORTANTE)

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 210 min

Señales: unos 9 ficheros (`plan/mod.rs`, `plan/guardar.rs`, `importar/mod.rs`, `importar/fichero.rs`, `alta.rs`, `plan/aprobar.rs`, `lib.rs`, `i18n/catalogo/importar.rs`, módulo nuevo `lectura.rs`); tres capas (CLI e informe, operación y regla pura, catálogo); 6 criterios; A3-08 obliga a tocar fixtures de varios E2E; suites E2E lentas y `cargo xtask verificar`.

**Las cinco fichas siguen vivas en develop:** A3-03 (`plan/mod.rs:139-170` ~ `importar/mod.rs:246-271`; `import` no sanea ruta ni error ni rechaza el vacío; tope duplicado en `guardar.rs:34` y `fichero.rs:33`), A3-05 (`paso_aprobar_hito` ~ `paso_aprobar`; `aprobar_hito_en` repite `de_la_orden`), A3-08 (`Entrada::listo`, `alta.rs:75-108`, no mira objetivo ni tope de 12; `Enunciado::validar` en `heuristica.rs:130-144`), A3-01 (`desde_anotacion`, `alta.rs:568-571`; el test solo mira `is_some()`), A3-02 (`es_esqueleto`, rama `aprobar.rs:117`; el test solo llama con `&[]`).

**Orden:**
1. A3-03: módulo privado `mod lectura;` en `lib.rs` con `pub const MAX_BYTES: usize = 1 << 20;`, `NoSeLeyo { motivo: Motivo, ruta (saneada), error (saneado) }`, `enum Motivo { NoSeLee, Grande, NoUtf8, Vacio }` y `fn fichero(ruta, tope) -> Result<String, NoSeLeyo>` (take(tope+1), UTF-8 estricto, rechazo del vacío, `plataforma::errores::sanear` de ruta y error). `plan::leer_fichero` e `importar::leer_fichero` quedan como traducciones finas a su `Informe` (cada orden conserva su línea: `plan uso motivo=grande`, `import uso motivo=fichero-grande limite=1048576`). Un solo tope: `importar::fichero::MAX_BYTES` pasa a `pub use crate::lectura::MAX_BYTES;` y se borra `plan::guardar::MAX_BYTES` (decisión 3). **Cambio de comportamiento en `import`:** el fichero vacío o solo con espacios pasa de `import uso motivo=sin-entradas` a `import uso motivo=vacio` (texto nuevo `importar::VACIO` es/en); prosa sin `###` sigue en `sin-entradas`; en `no-se-lee`/`no-utf8` la ruta y el error salen saneados (controles a espacio, recorte a 200, como ya hace `plan`). La línea de stdout no cambia salvo el motivo `vacio`.
2. A3-05: privada `fn persona_y_config<O>(e, nueva: impl FnOnce(&Quien) -> Option<O>) -> Result<(O, Config), Informe>`, usada por `paso_aprobar` y `paso_aprobar_hito`; `de_la_orden` gana la clave de cabeza (`"id"`/`"hito"`) y la usa `aprobar_hito_en`. Sin cambio observable.
3. A3-08: `Entrada::listo` llama al final a `plan::heuristica::Enunciado { objetivo, criterios }.validar()`; `objetivo-vacio` y `demasiados-criterios` se traducen a `NoListo::SinObjetivo` y `NoListo::DemasiadosCriterios` (dos textos nuevos). **Cambio de comportamiento:** `tc2 new "X" --criterio c` sin `--objetivo` sale `new uso motivo=enunciado-no-listo causa=objetivo-vacio`; con 13 criterios, `causa=demasiados-criterios`; en `import`, una entrada sin líneas `>` da `causa=objetivo-vacio` en su línea. El borrador vacío (solo título) se sigue admitiendo (decisión 1). Dependencia nueva `alta → plan::heuristica` (pura, sin ciclo).
4. A3-01: `desde_anotacion` envuelve una privada `fn alta_de_anotacion(a) -> Option<Alta>`; la doc explica que basta el título porque `exige_red() == true` (F1-2 D2b): un `new` nunca queda pendiente. El test lo prueba (decisión 2).
5. A3-02: solo un test en `plan/aprobar.rs`.
6. `cargo xtask verificar`; `## Resultado` con los 5 IDs y su commit.

Para A3-01 y A3-02: test primero contra el código actual (verde), mutante de la ficha (rojo), revertir.

## Reutilizacion (Regla 4D)

Lo que ya existe y se usa (helpers, funciones, fixtures, tests), buscado en el repo antes de disenar; y lo que se crea nuevo, con el motivo de no reutilizar:

- `plataforma::errores::sanear` (único saneado); la lectura acotada de `plan/mod.rs:152-169` se mueve a `lectura::fichero`; `importar::fichero::grande` e `importar::de_error`; `Enunciado::validar` y `MAX_CRITERIOS`; `NoListo::causa`/`mensaje` (se extienden); `de_la_orden`, `sin_persona`, `traer`, `config_de_base`.
- Patrón de ida y vuelta de `ciclo/operacion.rs:449-455`; fixtures `importar()`/`fichero()` de `tests/importar.rs`, el bucle de rechazos de `tests/alta.rs:353-377`, `Carpeta`, y `Fixture`/`texto`/`valor` de `tests/comun`.
- Nuevo: `mod lectura` (ni `importar` —crearía `plan → importar` por E/S— ni `importar/fichero.rs` —es puro—; es la casa natural del «leer con tope» único que unificará TASK-150, A8-17); `persona_y_config` (no existe esa secuencia); `alta_de_anotacion` (la mitad sin `Box` de la que ya existe).

## Lo que el rol no cubrio

- Riesgos: casos límite de `lectura::fichero` (FIFO, directorio, permisos, BOM) y carreras.
- Testing: solo el mínimo por criterio.
- Dominio: si el borrador vacío de `new` es un «enunciado no listo» según RF-01 (decisión 1); orden de causas.

## Riesgos aceptados y que los contiene

- A3-08 rompe fixtures que dan de alta con criterios y sin objetivo (`tests/estimacion.rs:288,347,423,741`, `tests/progreso.rs:103,158,260`, `tests/toma.rs:432` vía `comun/remoto.rs`, unitarios de `alta.rs:878-966`, quizá ficheros de import sin `>`): cambio mecánico (`--objetivo` o una línea `>`), lo destapa la suite completa.
- Cambio visible en `import` (`vacio`, ruta recortada en stderr): lo contienen el E2E nuevo y la nota en `## Resultado`; ningún test de `tests/` usa `sin-entradas`.
- El E2E del salto de línea no se pondría en rojo hoy (`una_linea` ya junta líneas): el test que cuenta es unitario en `importar::tests`, con `\n` y `ESC`, que hoy sí falla.
- A3-01 no reconstruye el alta completa: documentado y probado; si `new` dejara de exigir red, el test se pondría en rojo.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- A3-03 (2): unitario en `importar::tests` con la ruta inexistente `"no\nexiste\u{1b}[31m.md"`: `motivo=no-se-lee`, diagnóstico sin `\n` ni `ESC` (rojo hoy); E2E en `c7_limites_y_sin_red` de `tests/importar.rs`: fichero `" \n\t\n"` → `import uso motivo=vacio`, código 2, huella sin cambios (pesado).
- A3-05 (0): refactor; lo cubren los tests de `approve` y `cargo test -p tc2 --test aprobar` (pesado).
- A3-08 (2): E2E en el bucle de `c9_` de `tests/alta.rs` (`causa=objetivo-vacio` y `causa=demasiados-criterios`, código 2, sin refs nuevas; pesado); unitario en `importar/fichero.rs` (entrada sin `>` → `causa=objetivo-vacio linea=N`).
- A3-01 (1): `anotacion_ida_y_vuelta` reescrito: `alta_de_anotacion(&a.anotacion())` coincide en `(nombre, anotacion, exige_red, entrada)` y `a.exige_red()`; mutante `Alta::nueva("otro")` en rojo.
- A3-02 (1): tarea con `etiquetas=["wasm"]`, juego arquitectura + rendimiento y sus rutas: `es_esqueleto` da `true`, con `&[]` da `false`; mutante borrar la rama de `aprobar.rs:117` en rojo.
- Final: `cargo test -p tc2 --no-fail-fast` y `cargo xtask verificar` (pesados), en serie.

## Lo que necesita decision de una persona

1. A3-08: se sigue admitiendo el borrador vacío de `new` (solo título); la regla de `plan` se exige cuando hay algo escrito. Recomendado.
2. A3-01: documentar y probar en vez de reconstruir el alta completa. Recomendado.
3. Borrar `tc2::plan::guardar::MAX_BYTES` (sin uso fuera de `src`); `importar::fichero::MAX_BYTES` se conserva como reexportación. Recomendado.
4. `mod lectura` privado aquí; anotar en TASK-150 que su «leer con tope» vaya a ese módulo. Recomendado.
5. No mover `Enunciado` a `tarea` en esta tarea. Recomendado.

**Respuestas de la persona (Carlos, consigna del 2026-11-23: modo automatico con la opcion recomendada):** se aplican las recomendaciones 1 a 5.

**Regla de una hora:** 210 min, sin trocear: Carlos descartó trocear las tareas de corrección 4D (consigna del 2026-11-23).
