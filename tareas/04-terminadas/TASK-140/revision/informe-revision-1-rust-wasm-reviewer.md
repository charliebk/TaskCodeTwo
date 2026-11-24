# Informe de revision — TASK-140 (ronda 1)

- Commit revisado: 468ef2f20633db8d12b6b4b92a44665c0eefd3f7
- Revisor: general-purpose (sonnet) con rust-wasm-reviewer
- Veredicto: aprobada con correcciones

Veredicto propuesto: aprobada-con-correcciones (dos MENOR de cobertura de tests, ninguno bloquea; el codigo refactorizado es equivalente al anterior salvo el caso aceptado de la decision 2).

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/hallazgos/mod.rs:2439 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/hallazgos/mod.rs:1176 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### MEN-1 — el indice de `fuera-de-tope` de los cierres no lo fija ningun test cuando hay hallazgos

`cortar(&cierres, lista.len(), "cierres")` calcula `primero + TOPE_HALLAZGOS`. El test nuevo
(`mod.rs:2439`) solo cubre el caso sin hallazgos (`primero == 0`), asi que `primero` podria
perderse sin que nada se ponga rojo. Reproduccion: mutante `indice: Some(primero + TOPE_HALLAZGOS)`
-> `indice: Some(TOPE_HALLAZGOS)` en `cortar`; `--lib hallazgos`, `--test hallazgos`,
`--test revision` y `--test inyeccion` siguen en verde (sobrevive). El codigo actual es correcto
(equivale al `lista.len() + TOPE` anterior); falta el caso con al menos un hallazgo y mas de 100
cierres, que afirme `#<lista.len()+100>:fuera-de-tope`.

### MEN-2 — los topes de `plano` de `celdas` (64, 200 y 400) no estan fijados por ningun test

La tarea concentra en `celdas` los topes de id, fichero y descripcion de las tres tablas, pero
`la_fila_de_un_hallazgo_sale_entera_en_cada_tabla` usa textos cortos. Mutantes que sobreviven en
las cuatro suites: `plano(&h.fichero, 200)` -> 100, `plano(&h.descripcion, 400)` -> 300 y
`plano(&h.id, 64)` -> 32. (Los topes de otras tablas tampoco estaban fijados antes; no es una
regresion, pero ahora hay un unico sitio que bastaria blindar con un hallazgo de descripcion de
401 caracteres, fichero de 201 e id de 65.)

## Regla 4D

- Duplicacion: cumple. En `hallazgos/` no quedan copias de lo unificado: el `grep` de `get("id")`,
  `Rechazo {`, `TOPE_HALLAZGOS`, `plano(&.., 64|200|400)` y de bucles `it.next()` solo deja
  `fallo_de`, `Fallo::rechazo`, `cortar`, `celdas`/`fila` y `opciones` como unica implementacion.
  Lo que sigue aparte es distinto a proposito: `de_toda` (rechazo sin indice, `mod.rs:932`), la
  tabla de rechazados (3 columnas) y los titulos de anotaciones. `cortar` de `prompt.rs:51`
  trunca texto, no listas. Los candidatos del listado (`Opciones` vs `opciones`: tipo y funcion)
  quedan descartados.
- Desviacion: cumple. API publica sin cambios (`validar_con`, `resumen_markdown`,
  `abiertos_markdown`, `desde_incremental`; lo nuevo es privado); sin firma `#[wasm_bindgen]`.
- Diluvio: cumple. Un struct (`Opciones`) y cuatro funciones pequenas para siete subordenes.
- Disfraz: cumple. Los tests nuevos afirman valores (los 36 casos de uso por suborden, el indice y
  el detalle del rechazo, el texto exacto de las filas); los mutantes de abajo los ponen rojos.

## Alcance

- Ficheros revisados: crates/tc2/src/hallazgos/mod.rs, orden.rs, ronda.rs y
  crates/tc2/tests/hallazgos.rs.
- Contexto adicional pedido: ninguno.
- Limite JS-WASM: sin cambios de firma exportada (crate nativo, sin `#[wasm_bindgen]`).

## Reproduccion

- Clon: %LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-140-r1, rama
  fix/task-140-auditoria-4d-a2-hallazgos-y-revision-7-i (HEAD 8d3c295, solo anade la peticion
  sobre 468ef2f), `CARGO_TARGET_DIR` dentro del clon.
- Build: `cargo test --locked` compila; `cargo clippy -p tc2 --all-targets --locked -- -D warnings`
  sin avisos.
- Formato y lints: `cargo fmt --check` limpio; clippy limpio.
- Suite: no repetida entera (linea base de la peticion: 785 antes; 787 ok, 0 fallos, 13 ignorados
  despues). Ejecutadas: `--lib hallazgos` 24 ok; `--test hallazgos` 9 ok; `--test inyeccion` 4
  ok; `--test revision` 5 ok.
- Equivalencia por lectura del diff (sin ejecucion diferencial contra develop):
  - Markdown: las tres tablas dan el mismo orden de columnas y el mismo separador (`| a | b |\n`,
    tambien con celdas vacias); `ronda.to_string()` equivale a la interpolacion anterior.
  - `cortar`: mismos rechazados, mismo indice (`i == 100` y `lista.len() + 100`), mismo detalle
    (`N hallazgos|cierres pasan del tope de 100`) y mismo orden (el de tope va tras los de su
    lista, y los de cierres tras los de hallazgos), como antes del `break`.
  - `fallo_de`: mismo `id` (solo si `es_id`).
  - Analizador: la repetida de una unica se rechaza antes de leer el valor (igual que la guarda
    anterior), `falta-valor` al final igual, `--hallazgos`/`--check-runs`/`--anotaciones`
    repetibles. Unico cambio de linea: la validacion del valor (revisora, tarea, origen,
    bytes/tope) ya no ocurre al leerlo sino al terminar de analizar, de modo que solo cambia
    el motivo cuando hay dos errores a la vez (decision 2 aceptada). Con un error, igual.
- Mutacion (rojo = cubierto):
  - `cortar` indice +1: rojo (lib); `cortar` min(TOPE-1): rojo (lib); `cortar` sin `primero`:
    SOBREVIVE (MEN-1); cierre `rechazo(lista.len()+j)` -> `rechazo(j)`: rojo (lib); detalle de
    `cortar` +1: rojo (lib).
  - `fallo_de` sin filtro `es_id`: rojo (lib); `Fallo::rechazo` con `indice: None`: rojo (lib).
  - `celdas` fichero 200->100, descripcion 400->300, id 64->32: SOBREVIVEN (MEN-2); `fila` con
    separador `|`: rojo (lib).
  - Analizador: sin detectar repetida: rojo (`--test hallazgos`); falta-valor tragado: rojo;
    sin `comprobar` de revisora: rojo; sin `comprobar` de tarea: rojo; `--hallazgos` como unica:
    rojo; `rutas` en orden inverso: rojo. `una` con `last()` en vez de `first()` sobrevive pero es
    equivalente (una opcion unica solo tiene un valor). El mutante de "obligatoria ausente" no se
    pudo aplicar (el `sed` no cambio nada) y no lo cuento; la obligatoria la cubren los casos
    `sin-respuesta`, `sin-diff`, `sin-salida`, `sin-rutas` y `faltan-argumentos` del test, que
    pasan en verde sobre el codigo real.
  - Todos los mutantes se revirtieron; `git status` del clon limpio.
- Medidas: no aplican (sin cambios de rendimiento ni de limite WASM).

## Revisado sin hallazgos

- `Fallo::rechazo` y `fallo_de` (`use<>` correcto: el cierre no captura `v`).
- `cortar`: `split_at(min(len, TOPE))` no hace panic; con lista vacia o justa no genera rechazo.
- Orden de `rechazados` (hallazgos, tope de hallazgos, cierres, tope de cierres) y API publica.
- `Opciones`/`opciones`/`comprobar` y las siete subordenes (`validar`, `veredicto`, `anterior`,
  `ronda`, `desde`, `abiertos`, `prompt`): sin `unwrap`, sin indexado, errores por `Result`.
- Cambio de `Args` (se quita `Default`): sin uso restante.

## Lecturas fuera del contexto permitido

ninguna
