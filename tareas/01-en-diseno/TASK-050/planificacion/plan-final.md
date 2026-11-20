# Plan — TASK-050: F6-4 Busqueda de errores

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Un validador determinista en `tc2` fija el contrato de evidencia: esquema JSON, orden `tc2 hallazgos` y ejemplos de prueba. Cada hallazgo tiene que traer evidencia de su tipo (`reproduccion`, `mutante` o `borde`) y apuntar a una linea real del diff que vio esa revisora. El workflow solo lo cablea: la respuesta del modelo pasa por `tc2 hallazgos` antes de publicar, y el check enumera los aceptados y los rechazados con su motivo. El check sigue `neutral` (D4 de F6-3); el rojo es de F6-5. El modelo no decide: lo que devuelve es un dato que se valida.

Duracion estimada: 140 min

Senales: unos 6 ficheros de codigo (modulo nuevo `hallazgos/` con `mod.rs` y `orden.rs`, `lib.rs`, catalogo i18n, workflow y lint), 3 capas (tc2, workflow, lint y docs) y 2 criterios.

### Piezas y limites

- `crates/tc2/src/hallazgos/mod.rs` (nuevo): funcion pura `validar(respuesta, diff, informe) -> Validados`, sin red ni disco. Es un modulo propio, como `seleccion/`: F6-5, F6-6 y `review` dependen de el, y el no depende de nada. El nombre `Hallazgo` ya existe en `doctor/mod.rs`, asi que los tipos llevan un nombre propio. El lector de cabeceras `@@` del diff va aqui si no hay uno.
- `crates/tc2/src/hallazgos/orden.rs` y `lib.rs` (nuevo y extension), con el patron de `seleccion/orden.rs`:
  - `tc2 hallazgos validar --respuesta <md> --diff <txt> --revisora <id> [--informe <json>] [--json <salida>]`, con la linea `hallazgos ok revisora=<id> aceptados=N rechazados=M motivos=<id:motivo,...>`;
  - `tc2 hallazgos instrucciones`: el texto del prompt sale del mismo sitio que el validador, para que no se separen.
- `crates/tc2/esquemas/hallazgos.schema.json` (version 1) y `tests/fixtures/hallazgos/` (nuevo), con el precedente de `informe-verificacion.schema.json` y su lector serde con `deny_unknown_fields` (`verificacion/informe.rs`).
- `.github/workflows/revision-ia.yml` (extension): el job actual `publicar` se parte en tres y quedan cuatro jobs.
  - `preparar` anade al artefacto `diff-<i>.txt`, el diff recortado que vio el modelo.
  - `consultar` (privilegio, solo `ANTHROPIC_API_KEY`) pasa las respuestas por `outputs`, con un tope de 64 KiB por respuesta y aviso si se corta.
  - `validar` (clase ejecuta, `contents/actions: read`, sin secretos) compila `tc2` como `preparar`, corre `tc2 hallazgos` y sube `revision-ia-hallazgos`.
  - `publicar` (privilegio, solo la App) lee ese artefacto como dato.
- `xtask/src/workflows.rs` (extension): el test del workflow del revisor exige los cuatro jobs, cada secreto en su job y `validar` sin secretos. El perfil `REVISION` no cambia.
- `docs/ci/revision-ia.md` y una adenda F6-4 en `docs/adr/F1-8-*.md` (extension). La adenda dice que la ejecucion de la decision 3 de F1-8 pasa a F6-6.

### Orden de construccion

1. Esquema v1, tipos y `validar` pura, con ejemplos de prueba. Esto ya demuestra la regla sin modelo.
2. `tc2 hallazgos validar|instrucciones`, con su linea, su JSON y el texto del catalogo i18n.
3. Workflow con los cuatro jobs: el prompt usa `tc2 hallazgos instrucciones` y el check lista aceptados y rechazados.
4. Test del lint actualizado, guia y adenda del ADR.

### Alternativas descartadas

- **Crear ya `revision-reproducir`, que ejecuta las reproducciones, mutantes y bordes (decision 3 de F1-8).** El codigo del PR tiene que correr en `pull_request` (clase ejecuta), y la ronda siguiente no existe hasta F6-6. Se aplaza a F6-6 con adenda (decision de la persona, abajo).
- **Validar con `jq` en `publicar`.** La regla quedaria fuera de Rust, sin tests de cargo, y repetida donde F6-5 la necesita.
- **Ejecutar en `publicar` el binario de `tc2` que sube `preparar`.** Seria ejecutar un artefacto en un job de privilegio, y F1-8 lo prohibe.

## Troceo

Pasaba de 60 min. Se propuso partirla en 050a (validador en tc2, unos 75 min) y 050b (cableado en el workflow, unos 65 min). **La persona decidio no trocear** (2026-11-20): una sola tarea, como en las cadenas anteriores. El orden de construccion deja el validador probado antes de tocar el workflow.

## Lo que el rol no cubrio

- Criterio 1, por donde se rompe: JSON que no parsea, un bloque repetido, inyeccion dentro del JSON (F6-7), un diff recortado que deja fuera la linea citada. Se cubren con ejemplos de prueba en la implementacion.
- Criterio 2: «reproduccion, mutantes y bordes» queda como evidencia declarada y anclada al diff, no ejecutada (aplazado a F6-6 por decision de la persona).
- La estrategia de pruebas no la ha revisado el rol de testing.

## Riesgos aceptados y que los contiene

- **Evidencia declarada, no ejecutada.** La evidencia la declara el modelo y no se ejecuta en esta ronda. La contienen el anclaje al diff, que el `linea_original` del mutante tenga que coincidir con el texto de esa linea, la aprobacion de la persona y F6-6.
- **E2E sin el modelo.** La cuenta de la API no tiene saldo, asi que el E2E con el modelo puede no ser posible. Lo contienen las pruebas de `tc2` con ejemplos y un E2E del cableado: `consultar` falla y aun asi se publica un check que lo dice.
- **Una compilacion mas de `tc2` por ejecucion**, en `validar`. Es un coste fijo y no anade superficie.
- **Tope de `outputs` de GitHub.** El tope de 64 KiB por respuesta y el aviso de corte lo contienen; se comprueba en el E2E.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

Criterio «Hallazgo sin evidencia = rechazado por el propio workflow» (3 tests):
1. `crates/tc2/tests/hallazgos.rs`, con el binario real. El ejemplo tiene un hallazgo valido de cada tipo, uno sin `evidencia` y uno con la linea fuera del diff. Se espera `aceptados=3 rechazados=2` con los motivos `sin-evidencia` y `linea-fuera-del-diff`.
2. Una respuesta sin bloque JSON (por ejemplo, un informe de revision en prosa) sale con todo rechazado y el motivo `sin-bloque`.
3. El test del lint ampliado exige los cuatro jobs, que `validar` corra `tc2 hallazgos` sin secretos y que `publicar` lea `revision-ia-hallazgos` y no las respuestas en bruto.

Criterio «Cumple F6-4 y RR-03» (2 tests):
1. La salida `--json` valida contra `hallazgos.schema.json`, como el test de `verificacion/informe.rs`.
2. E2E en un PR real del repo de pruebas: la App publica el check `revision-ia` con las lineas `aceptados=` y `rechazados=`. Con el modelo si hay saldo; si no, con `consultar` en error, y se deja escrito.

Ninguno es pesado.

## Lo que necesita decision de una persona

Decidido por Carlos el 2026-11-20:
1. **Ejecucion de la evidencia**: se aplaza a F6-6, con adenda F6-4 en F1-8 («F6-4 deja la reproduccion como artefacto» queda en «F6-4 exige y valida la evidencia; F6-6 la ejecuta»).
2. **Que hallazgos necesitan evidencia**: todos, tambien los MENORES. Evidencia minima por tipo:
   - `reproduccion`: pasos y fallo esperado, o un paso en rojo del informe F5-4;
   - `mutante`: `linea_original` (igual al texto de esa linea en el diff), la mutacion y el test que deberia detectarla;
   - `borde`: entrada, resultado esperado y resultado segun el codigo, distintos entre si.
3. **Sin trocear.**
4. **Compilar `tc2` tambien en `validar`**, sin pasar binarios entre jobs. Lo decide quien orquesta con la opcion recomendada: no anade superficie.

## Suposiciones no verificadas

- Que GitHub deja pasar unos 64 KiB por N revisoras en `outputs` de un reutilizable, sin enmascarar ni truncar.
- Que la clave del modelo y la App pueden ir en jobs distintos del mismo entorno `revisor` con la herencia actual (adenda R9 de F1-8).
- Que el modelo puede dar `linea_original` literal con el diff recortado que ve (`tope-diff`).
