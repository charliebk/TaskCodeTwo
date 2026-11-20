# Informe de revision — TASK-049 (ronda 3)

- Commit revisado: ca4de580f20504840ec08a8d48b2aae9340b3b30
- Revisor: code-quality-reviewer (ronda 3, verificacion acotada a las correcciones de 9c14787; pase generico, sin revisar la rama entera)
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| sin hallazgos | — | — | — | — |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### Verificacion de los MENORES de la ronda 2

Las tres correcciones se comprueban, no se dan por buenas.

- **MEN-1 (la clave no puede llegar al check): corregido.** La linea `motivo=$(jq ...)` de `.github/workflows/revision-ia.yml` (linea 365) se ejecuto en bash con `ANTHROPIC_API_KEY` exportada como `sk-ant-test-SECRET123`. Con un error de la API que contiene la clave (`invalid x-api-key: sk-ant-test-SECRET123 rejected`), el resultado fue `authentication_error: invalid x-api-key: *** rejected`: la clave no aparece en la salida (0 apariciones). El `split` es literal, asi que una clave con caracteres especiales tampoco rompe el enmascarado.
- **MEN-3 (sin `.error` y recorte por caracteres): corregido.**
  - JSON sin `.error` (`{"type":"message","stop_reason":"end_turn"}`): `motivo` vacio, y el `${motivo:-sin-cuerpo}` del workflow sale como `error-http-<codigo> sin-cuerpo`.
  - Cuerpo no JSON (`<html>...502 Bad Gateway...`): jq falla, se silencia con `2>/dev/null || true`, `motivo` vacio y `sin-cuerpo`. Correcto.
  - Mensaje de 250 `é` con prefijo `x: ` (longitud original 253): el recorte da exactamente 200 caracteres (`jq length` = 200). Recorte por codepoints, no por bytes (jq 1.8.2 probado; el runner usa jq 1.7 o superior, mismo comportamiento de `.[0:200]` sobre cadenas).
  - Mensaje con saltos de linea (`linea1\r\nlinea2`): sale `x: linea1  linea2` en una linea.
- **MEN-2 (texto de `inherit` y del lint): corregido.** `docs/adr/F1-8-seguridad-modelo-de-amenazas.md` (adenda F6-3, lineas 724-730) y `docs/ci/revision-ia.md` (lineas 86-95) dicen que `inherit` entrega todos los secretos de repositorio y de organizacion, y que el lint corre en los tests de TaskCodeTwo y no en el repo del consumidor. Es un texto exacto con el comportamiento del workflow y de `xtask/src/workflows.rs`. La nueva aserion del test que comprueba que `secrets: inherit` del reutilizable sigue rechazado y que el llamador con `contents: write` falla el lint pasa en `cargo test -p xtask`.

### Puertas, en el caso del jq de MEN-1

Se probo fuera del workflow la linea exacta extraida con `sed`, no un reescrito. La ejecucion de jq esta protegida en CI por el paso: si `ANTHROPIC_API_KEY` esta vacia o no existe, el bucle de `resumen` hace `continue` antes de llamar a la API (linea 341, `if [ -z "$ANTHROPIC_API_KEY" ]`). Por eso el caso de clave vacia o sin exportar no llega a esa linea en CI. Sin la clave exportada, la misma linea de jq falla (`split input and separator must be strings`) y deja `motivo` vacio; esto solo ocurriria si el paso se reescribiera sin pasar la variable, y no es un caso real hoy.

## Reproduccion

Clon limpio en `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-049-r3-b`, rama `feature/task-049-f6-3-revisor-en-ci-con-las-skills-de-dom`. Clon borrado al terminar; el repo original no se toco (solo `git diff --stat` de lectura entre `ca4de58` y `3c54cf4`).

- HEAD de la rama al clonar: `3c54cf4`. Entre `ca4de58` y `3c54cf4` solo cambian ficheros de `tareas/03-en-revision/TASK-049/` (informes y peticion de la ronda 3); el workflow, la guia, el ADR y `xtask` son identicos a los revisados en `ca4de58`.
- `cargo test -p xtask` con `CARGO_TARGET_DIR` dentro del clon: **29 passed, 0 failed** (1 suite). No se ejecuto el filtro por nombre del test del llamador por separado: el suite completo es la evidencia.
- Ficheros de respuesta de prueba en `task-049-r3-b\jq-pruebas\` (clave en error, sin `.error`, no JSON, 250 `é`, saltos de linea), todos borrados con el clon.

## Notas (fuera de la tabla, no abren ronda)

- La linea jq del workflow (MEN-1) no la ejecuta ningun test automatico: `xtask` comprueba la guia y el lint, pero no el enmascarado ni el recorte. Se verifico a mano en esta ronda. Un test que ejecute la linea con una respuesta de prueba la cubriria; es mejora, no bloquea.
- La peticion de la ronda 3 indica `ca4de58` como HEAD revisado; el HEAD real de la rama es `3c54cf4` (solo anade ficheros de tareas). El codigo verificado es el mismo.

## Lecturas fuera del contexto permitido

ninguna
