# Informe de revision — TASK-052 (ronda 2)

- Commit revisado: 3df9643e52dc72400bde144775a4131f7e726ca0
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

sin hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|

Verificacion de lo corregido en la ronda 1:

- IMP-1 (ligera como base): corregido de verdad. `elegir` salta los checks `ligera` (se reconocen por la linea `Ronda N (ligera): ` del resumen, que el texto del modelo no puede fingir porque va en fila de tabla o elemento de lista) y devuelve `ligera_pendiente`; `decidir` no admite otra `ligera` si hay una pendiente y sale `incremental` desde el ultimo commit con modelo. Cubierto por `una_ligera_no_sirve_de_ronda_anterior` y `tras_una_ligera_la_ronda_siguiente_va_con_modelo`. El workflow resta lo que entro de la base con `tc2 hallazgos desde` (`merge-tree --write-tree`); la regex del workflow solo casa `desde=<oid de 40 o 64>`, y `desde=base` deja `desde` vacio y vuelve a `base...head` (falla en cerrado).
- MEN-1 (cuenta del informe de la tarea): corregido. `InformeTarea::nuevo` lee el modo del resumen; `el_informe_de_la_tarea_cuenta_como_el_check` compara informe y check en ligera, incremental, completa y tope.
- MEN-2 (`## Rondas extra`): corregido. Cuenta `## Ronda extra` y `## Rondas extra`, ignora elementos vacios y bloques de codigo (``` y ~~~).
- MEN-3 (ancla del cierre): `aceptado`; documentado en `docs/ci/revision-ia.md` (lineas 460-463: el ancla solo prueba que la linea esta en el lado nuevo del diff incremental, de cualquier fichero; que arregle el abierto lo juzga el modelo). Ademas queda documentado el limite de la ligera (lineas 452-459).
- MEN-4 (objeto sin `hallazgos`): corregido. Una clave ausente es lista vacia en las dos; `{"cierres":[...]}` conserva el cierre valido (1 cierre, 0 rechazos); `cierres: null` sigue siendo `json-invalido`.

Regresiones buscadas en el diff, sin encontrar ninguna: `desde_incremental` (ramas `no-ancestro`, `directo`, `merge`, `conflicto` y fallo de git cubiertas por `el_diff_incremental_no_arrastra_lo_que_trae_la_base`; `es_sha` y `es_ref` validan los argumentos y `es_ref` rechaza opciones y rangos); el subcomando `MergeTree` solo se invoca con `--write-tree` (escribe objetos, no mueve refs ni toca worktree); `elegir` ordena por `id` descendente dentro del commit, venga como venga; el caso en que solo hay checks `ligera` degrada a ronda 1 `completa` (cerrado).

## Alcance
- Ficheros revisados: `crates/tc2/src/git/puerto.rs`, `hallazgos/mod.rs`, `hallazgos/orden.rs`, `hallazgos/ronda.rs`, `tests/hallazgos.rs`, `.github/workflows/revision-ia.yml` (solo los pasos de ronda), `docs/ci/revision-ia.md` (solo las secciones de limites).
- Contexto adicional pedido: ninguno.
- Limite JS-WASM: sin cambios de firma exportada (no hay `#[wasm_bindgen]` en el diff).

## Reproduccion
- Clon: `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-052-r2-rust` en 3df9643 (detached); borrado al terminar.
- Build y lints: no repetidos (`cargo xtask verificar --perfil rapida` en verde en d032160, mismo codigo).
- Suite: `cargo test -p tc2 --lib hallazgos` 21 ok; `cargo test -p tc2 --test hallazgos` 8 ok; `--test revisoras` 5 ok.
- Casos construidos: lectura contra el codigo de los casos de las tres reglas (ligera pendiente, ligera fingida por el modelo en un elemento de lista, objeto sin `hallazgos`, variantes de rondas extra) y del repo real de git del test de `desde`. No se escribieron tests nuevos propios.
- Mutacion: no repetida en esta ronda (acotada a verificar las correcciones); los tests nuevos de IMP-1 ejercen `ligera_pendiente` y fallarian si se quita la condicion `!ligera_pendiente` o el salto de la ligera en `elegir`.
- Medidas: no aplican.

## Revisado sin hallazgos
- Aritmetica y errores en codigo no-test del diff: sin `unwrap`/`expect`/indexado de entrada nuevo.
- Workflow: `desde.txt` se escribe siempre (vacio si no hay `desde`) y el paso siguiente lo lee; el texto del prompt distingue los dos casos.

Veredicto propuesto: aprobada
