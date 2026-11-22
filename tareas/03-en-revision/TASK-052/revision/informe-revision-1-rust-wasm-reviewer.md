# Informe de revision — TASK-052 (ronda 1)

- Commit revisado: 8822e8ee1e21657de2fd037cc0d334b27fe58006
- Revisor: rust-wasm-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | abierto | crates/tc2/src/hallazgos/ronda.rs:199 | plan |
| MEN-1 | MENOR | abierto | crates/tc2/src/hallazgos/mod.rs:1868 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/hallazgos/ronda.rs:284 | implementacion |
| MEN-3 | MENOR | abierto | crates/tc2/src/hallazgos/mod.rs:863 | plan |
| MEN-4 | MENOR | abierto | crates/tc2/src/hallazgos/mod.rs:924 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### IMP-1 — La ronda `ligera` da `success` a codigo que nadie revisa y lo convierte en la base de la ronda siguiente

- Reproduccion: con el test existente `decidir_aplica_tope_ligera_e_incremental` (anterior verde, abiertos M1 en `b.rs` y M2 en `c.rs`, cambios `["b.rs"]`) la salida es `Modo::Ligera`, ronda sin subir. `veredicto_ronda(&[], Some(&d))` da `Dictamen::Verde` (`success`) sin ninguna revisora. La condicion solo mira que el fichero cambiado tenga un MENOR abierto, no que el cambio sea pequeno ni que arregle el MENOR: cualquier codigo nuevo en ese fichero (incluido un CRITICO) pasa el check en verde. Ademas `elegir` acepta ese check `verde` como ronda anterior, asi que el siguiente diff incremental parte del commit ligero y ese cambio no lo ve nunca ningun modelo en rondas posteriores.
- Impacto: la puerta de la revision IA se salta para el codigo tocado en ficheros con un MENOR abierto. Es lo que pide la decision 4 del plan, por eso el origen es `plan`; se reporta porque el plan no lo reconoce en «Riesgos aceptados».
- Sugerencia: que `sha_anterior` siga siendo el ultimo check con modelo (la ligera no sirve de base), limitar la ligera a cambios que no anadan codigo, o documentarlo como riesgo aceptado.

### MEN-1 — En una ronda ligera el informe de la tarea y el check cuentan distinto `nuevos` y `persisten`

- Reproduccion (test temporal en el clon): anterior verde de ronda 2 con M1 (`b.rs`, ronda 2) y M2 (`c.rs`, ronda 2), cambios `["b.rs"]`. `veredicto_ronda` da `nuevos=0 persisten=1 cerrados=1` (resumen del check: «Ronda 2 (ligera): 0 nuevos, 1 persisten, 1 cerrados»). `InformeTarea::nuevo(.., (pr, sha, 2), ..)` con esos mismos aceptados da `nuevos=1 persisten=0 cerrados=1`. Causa: `InformeTarea::nuevo` reconstruye «nuevo» como `a.ronda >= ronda` y la ligera no sube la ronda.
- Impacto: el criterio «hallazgos nuevos por ronda, medidos» da dos cifras distintas para la misma ronda ligera (check frente a `revision-ia-<sha12>.json` y linea de `review`).

### MEN-2 — `## Rondas extra` cuenta lineas que no son rondas extra y no lee `## Ronda extra`

- Reproduccion (test temporal sobre `rondas_extra`): `"## Rondas extra\n\n- \n"` da 1 (un elemento vacio suma una ronda); un bloque de codigo con `- ejemplo` y `- otro` da 2 (se lee el texto de la seccion linea a linea, sin respetar vallas); `"## Ronda extra\n\n- a\n"` da 0 aunque F1-16 nombra las dos secciones (`## Rondas extra`, `## Ronda extra`) como la peticion de ronda extra.
- Impacto: el tope sube o no por un detalle de formato; solo cuesta tokens o una ronda `tope`, no salta puertas.

### MEN-3 — El ancla del cierre no tiene relacion con el abierto

- Reproduccion (test temporal): abierto `I1` IMPORTANTE en `viejo.rs`, que el diff incremental borra, mas un cambio en `docs/n.md`. `{"cierres":[{"id":"I1","fichero":"docs/n.md","linea":1,...}]}` se acepta (`cierres=1`); el mismo cierre en `viejo.rs:1` se rechaza (`cierre-fuera-del-diff`). Vale cualquier linea del lado nuevo del diff incremental, de cualquier fichero.
- Impacto: «cierre anclado al diff» solo comprueba que el diff tenga esa linea; el cierre de un IMPORTANTE depende por entero del modelo y el ancla no lo contrasta. Exigir el mismo fichero rompe los arreglos en otro fichero (p. ej. el test), por eso es MENOR y de plan.

### MEN-4 — Un objeto sin `hallazgos` tira tambien los cierres validos

- Reproduccion (test temporal): con un abierto `I1` valido, `{"cierres":[{"id":"I1","fichero":"a.rs","linea":1,"motivo":"m"}]}` da `cierres=0` y un unico rechazo global `json-invalido`; `{"hallazgos":[],"cierres":null}` igual. En cambio `{"hallazgos":[]}` sin `cierres` se acepta: el codigo trata `cierres` ausente como lista vacia pero `hallazgos` ausente como error.
- Impacto: falla en cerrado (el IMPORTANTE sigue abierto y el check sale rojo), pero la ronda se pierde por una omision que el codigo tolera en la otra clave.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/hallazgos/mod.rs` (`Diff::leer`) y `crates/tc2/src/tarea/cuerpo.rs` (`seccion`): para saber que lineas cuenta `Diff::linea` (lado nuevo, con contexto) y como entrega el texto de la seccion a `rondas_extra`.
- `docs/adr/F1-16-lotes-desatendidos.md` y `docs/adr/F1-15*.md` (S13): para contrastar el nombre de la seccion de rondas extra y que S13 no reduce revisoras.
- `.github/workflows/revision-ia.yml` (solo `grep` de `is-ancestor` y `ronda.json`): para ver que existe el fallback de force-push; no revisado.

## Alcance
- Ficheros revisados: los de dominio Rust de la peticion; `hallazgos/mod.rs`, `ronda.rs` y `orden.rs` en profundidad.
- Contexto adicional pedido: ver «Lecturas fuera del contexto permitido».
- Limite JS-WASM: sin cambios de firma exportada (no hay `#[wasm_bindgen]` en el diff).

## Reproduccion
- Clon: `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-052-r1-rust` en 8822e8e (detached); borrado al terminar.
- Build y lints: no repetidos (`cargo xtask verificar --perfil rapida` en verde en 8822e8e, por el orquestador).
- Suite: `cargo test -p tc2 --lib hallazgos` 16 ok; `--test hallazgos` 6, `--test plataforma` 7, `--test revision` 5, `--test revisoras` 5, `cargo test -p xtask` 34, `--lib seleccion` 12: todo en verde.
- Casos construidos: modulo de tests temporal `adversarial` en `ronda.rs` del clon (ligera con `nuevos/persisten`, cierre sobre fichero borrado y sobre otro fichero, formas del objeto `{hallazgos, cierres}`, cierres duplicados, id reutilizado de un MENOR, ids iguales de dos revisoras, verde y rojo sin cambios, variantes de `## Rondas extra`). No se conserva.
- Mutacion: en `validar_cierre`, (a) aceptar cierres de MENOR, (b) quitar la comprobacion de `paso_rojo`, (c) no mirar el diff: las tres ponen en rojo `cada_regla_del_cierre_rechaza_con_su_motivo`. Hay red.
- Medidas: no aplican.

## Revisado sin hallazgos
- Tope: `tope_rondas=0` sube a 1; tras un check `tope`, `elegir` lo salta y la siguiente ronda vuelve a decidir `tope` o, con una ronda extra, `incremental` desde el ultimo check con modelo.
- Compatibilidad v2: `raw_details` sin `ronda`/`estado` y titulos v2 se leen como ronda 1 abierto; `ronda=0` o no numerico falla en cerrado.
- Aritmetica: `saturating_add` y restas protegidas; sin `unwrap`/`expect`/indexado de entrada en codigo no-test.
- Un cierre solo cierra graves (los MENOR dan `cierre-desconocido`); un hallazgo nuevo no reutiliza el id de un abierto grave de su lista (un M1 nuevo si puede repetir el id de un MENOR persistente, que no va en la lista; sin consecuencias para el cierre).

Veredicto propuesto: aprobada-con-correcciones
