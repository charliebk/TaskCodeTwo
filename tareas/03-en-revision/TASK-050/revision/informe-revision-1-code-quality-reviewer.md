# Informe de revision — TASK-050 (ronda 1)

- Commit revisado: 90640c05dc759042baa565655060c3224b753c6f
- Revisor: code-quality-reviewer (agente independiente; pase generico, centrado en workflow, docs y ADR)
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | .github/workflows/revision-ia.yml:476 | implementacion |
| MEN-2 | MENOR | abierto | .github/workflows/revision-ia.yml:488 | implementacion |
| MEN-3 | MENOR | abierto | xtask/src/workflows.rs:1446 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

## Verificacion hecha (clon en AppData\Local\pruebas-claude\taskcodetwo\task-050-r1-b, rama en el clon, CARGO_TARGET_DIR propio)

- `cargo test -p xtask`: 29 passed, 0 failed. `cargo test -p tc2 --test hallazgos`: 3 passed. No se lanzo la suite completa ni nada en GitHub.
- Mutaciones del workflow contra el lint de xtask (cada una revertida): clave del modelo en `validar` (rojo), secreto de la App en `consultar` (rojo), `needs.consultar.outputs.r0` interpolado en un `run:` de `validar` (rojo), `contents: write` en `validar` (rojo). La quinta (renombrar el artefacto que sube `validar`) NO se detecta: MEN-3.
- Se extrajo el bucle del `run:` de `validar` y se ejecuto en bash con el `tc2` compilado y datos de prueba: respuesta valida (aceptados=3 rechazados=2), base64 ilegible (`respuesta-ilegible`, sin-bloque), respuesta vacia con `error-http-400` (sin-bloque), hueco sin respuesta e indice 7 fuera de los 6 huecos (sin-bloque), sin `--informe` con `set -u` (el array vacio no rompe). Todos salen con la linea esperada y el job no falla.
- Entradas adversas a `tc2 hallazgos validar`: id con salto de linea y `::error::` (rechazado `hallazgo-invalido`, mostrado como `#0`), fichero y descripcion con `::error::` y `</details>` (escapados o sin saltos de linea: no hay inyeccion de comandos de workflow), UTF-8 cortado a mitad de caracter (no falla), ruta con espacios (cabecera con TAB final), ruta con tilde, lineas `---` y `+++` dentro de un hunk (contenido, no cabecera), `\ No newline`: la numeracion del lado nuevo es correcta en todos.
- Clases de F1-8 leidas en el YAML: `ANTHROPIC_API_KEY` solo en `consultar`; App solo en `publicar`; `preparar` y `validar` sin `secrets.`; `validar` no ejecuta nada del PR (compila tc2 del SHA fijado); respuestas del modelo por `env:` y nunca por `run:`; `consultar` no sube artefactos; `publicar` sin checkout y lee `revision-ia-hallazgos` como dato con `--rawfile`; tope de 64 KiB y 6 huecos coherentes con la aritmetica de la doc (6 x ~87 KB = ~524 KB < 1 MiB; 87 KB < 128 KiB por variable). La adenda F6-4 de F1-8 y `docs/ci/revision-ia.md` coinciden con el YAML.
- Criterio «Hallazgo sin evidencia = rechazado por el propio workflow»: se cumple en la cadena preparar, consultar, validar, publicar (reproducido el tramo validar; el E2E real de quien orquesta cubre la ruta `sin-bloque`; la ruta `sin-evidencia` solo esta cubierta por el test de tc2, no por un run real con respuesta del modelo).
- No verificado (necesita GitHub real): que `outputs` de ~87 KB por hueco pasen sin truncar ni enmascarar entre jobs del reutilizable (la doc y la adenda ya lo declaran); que 4 revisoras lentas quepan en `timeout-minutes: 60` de `consultar` (cada llamada admite `--max-time 900`, 4 x 900 s = 60 min justos).

## Reproduccion

### MEN-1 — el corte a 64 KiB se come justo el bloque json, que va al final (revision-ia.yml:476)

`consultar` hace `head -c 65536` de la respuesta, pero las instrucciones piden "al final, exactamente un bloque json". Cualquier respuesta valida que pase de 64 KiB pierde el bloque y sale rechazada entera como `sin-bloque`, aunque sus hallazgos fueran correctos; el aviso dice "lo que pase del corte no se valida", que no describe lo que pasa (se pierde todo). Reproduccion: 70000 bytes de prosa + `respuesta-mixta.md` da `aceptados=3 rechazados=2`; la misma cortada con `head -c 65536` da `aceptados=0 rechazados=1 motivos=respuesta:sin-bloque`. Con `max_tokens: 16000` es raro, por eso MENOR. Arreglo: avisar con el motivo real y/o pedir el bloque antes del informe, o conservar el ultimo bloque al cortar.

### MEN-2 — una revisora larga expulsa del check las tablas validadas de las siguientes (revision-ia.yml:488)

`validar` concatena en `texto.md`, por revisora, la tabla validada y el informe en prosa completo, y `publicar` corta el conjunto con `.[0:65000]`. Reproduccion con el bucle real y tres respuestas de ~30 KB: `texto.md` mide 97131 bytes y la tabla de la tercera revisora empieza en el byte 64767, es decir, casi fuera del check. Los conteos y motivos siguen en el resumen (una linea por revisora), por eso MENOR; pero el criterio dice que el check lista aceptados y rechazados con motivo. Arreglo: todas las tablas primero y las prosas despues, o recortar la prosa por revisora.

### MEN-3 — el lint no comprueba que `validar` suba el artefacto que lee `publicar` (xtask/src/workflows.rs:1446)

`assert!(usa("validar", "revision-ia-hallazgos"))` se satisface con la ruta `$RUNNER_TEMP/revision-ia-hallazgos` del `run:`, y la de `publicar` con el `-f name=revision-ia-hallazgos`. Mutacion: cambiar `name: revision-ia-hallazgos` por `name: otro-nombre` en el `upload-artifact` de `validar` da `cargo test -p xtask workflows`: 18 passed, sin rojo. Consecuencia real: `publicar` no encontraria el artefacto y el check saldria siempre `neutral` con `sin-hallazgos`, sin que ningun test avise (solo el E2E). Arreglo: asertar que el `with.name` del paso `upload-artifact` de `validar` es exactamente `revision-ia-hallazgos`.

## Observaciones sin hallazgo (ya documentadas en la adenda y los limites conocidos)

- El informe en prosa del modelo (con los hallazgos que la tabla rechazo) se publica tal cual en el check y puede imitar una tabla de hallazgos. Esta declarado como limite (defensa en F6-7) y el check sigue `neutral`.
- Se ha retirado del prompt la linea `veredicto: aprobado|cambios`; no hay consumidor en el repo, asi que no rompe nada hoy, pero F6-5 tendra que decidir de donde sale el veredicto.
- Hardening posible, no explotable hoy: en `validar`, `${!r}` con `r="R$i"` interpretaria subindices aritmeticos si `lista.tsv` trajese un `i` no numerico; `i` sale de `seq` en `preparar`, no del PR.

## Lecturas fuera del contexto permitido

- `xtask/src/workflows.rs` (lineas 1380-1470) y `crates/tc2/src/hallazgos/mod.rs` (busquedas puntuales): para reproducir las mutaciones del lint y probar el validador con entradas adversas.
