# Plan — TASK-051: F6-5 Hallazgos estructurados

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

El veredicto lo calcula `tc2` (como la validacion en F6-4): una funcion pura junta los `Validados` de todas las revisoras y saca la conclusion del check, su resumen y las anotaciones por linea. El workflow solo transporta: `validar` lo genera y `publicar` lo envia tal cual con la App del revisor. Conclusion = `failure` si hay algun hallazgo CRITICO o IMPORTANTE aceptado (en F6-5 todo aceptado esta abierto: el cierre llega con F6-6) o si alguna revisora no dejo respuesta valida (veredicto `incompleta`); `success` en otro caso. Los «comentarios» son anotaciones del check-run (sin permisos nuevos). El informe en la carpeta de la tarea lo escribe `tc2 review` en local, leyendolo del check publicado con la identidad del agente (`checks:read`) y guardandolo por el punto unico de escritura de la rama de estado, como `plan guardar`.

Duracion estimada: 220 min

Senales: ~11 ficheros de codigo (hallazgos/mod.rs, hallazgos/orden.rs, esquema + fixtures, revision-ia.yml, xtask/src/workflows.rs, plataforma/mod.rs, plataforma/github.rs, revision/mod.rs + operacion nueva, catalogo i18n), 4 capas (tc2 puro, workflow+lint, puerto de plataforma, rama de estado), 2 criterios.

### Piezas y limites

- `crates/tc2/src/hallazgos/mod.rs` (extension):
  - Columna «Clase»: la decision 12 de F1-15 y su consecuencia «F6-5 anade la columna Clase» la asignan a esta tarea. Enum `Clase` con la lista cerrada `puerta-seguridad`, `contrato`, `correccion`, `fidelidad-evidencia`, `reproducibilidad`; obligatoria en `Cabecera`; si falta o no es de la lista, motivo `hallazgo-invalido` (ya existe). `instrucciones()` la incluye desde las mismas constantes. `VERSION` pasa a 2.
  - Funcion pura `veredicto(revisoras: &[Option<Validados>]) -> Veredicto` con `conclusion`, `veredicto=rojo|verde|incompleta` y recuentos por severidad; no lee disco. «Incompleta» = revisora sin JSON (`None`), rechazo de toda la respuesta (`indice: None`) o ninguna revisora.
  - `anotaciones(&Veredicto) -> Vec<Vec<Anotacion>>` en lotes de 50 (tope de la API por peticion). Cada aceptado da una anotacion: `path`/`start_line` = `fichero`/`linea`; nivel `failure` para CRITICO e IMPORTANTE, `warning` para MENOR; `title` = `id SEVERIDAD clase (revisora)` por `plano`; `message` = `descripcion` por `plano`; `raw_details` = JSON del `Aceptado`.
  - Lectura de vuelta: `Validados`/`Aceptado` ganan `Deserialize`; el motivo se reconstruye contra `MOTIVOS` y uno desconocido es error.
- `crates/tc2/src/hallazgos/orden.rs` (extension): `tc2 hallazgos veredicto --hallazgos <json>... --salida <dir>`. Escribe `check.json` (`conclusion`, `output.title`, `output.summary`, primeras 50 anotaciones) y `anotaciones-<k>.json` con el resto. Linea: `hallazgos ok veredicto=<v> conclusion=<c> criticos=N importantes=M menores=K revisoras=R incompletas=I`. Sale `ok` aunque el veredicto sea rojo (patron de `validar`, linea de contrato de F1-4).
- `crates/tc2/esquemas/hallazgos.schema.json` (v2): `clase` en `$defs/hallazgo` y `$defs` nuevos `veredicto` e `informe-tarea`. Fixtures de `tests/fixtures/hallazgos/` actualizados y nuevos (con IMPORTANTE, solo MENOR, en prosa). Sin esquema aparte: el test esquema-codigo de F6-4 ya cubre este fichero.
- `.github/workflows/revision-ia.yml` (sin jobs nuevos):
  - `validar`: tras el bucle, `tc2 hallazgos veredicto` deja `check.json` y los lotes en `revision-ia-hallazgos`.
  - `publicar`: POST de `check.json` con `name` y `head_sha` puestos por el job y `conclusion` aceptada solo si es `success` o `failure`; despues un PATCH por lote sobre el id devuelto. Si falta el artefacto o `validar` fallo, publica `failure` con veredicto `incompleta` (en lugar del `neutral` actual). `publicar` no calcula nada (sin jq con logica, descartado en TASK-050).
- `xtask/src/workflows.rs`: el lint exige que `validar` corra `tc2 hallazgos veredicto`, que `publicar` no tenga `conclusion` literal y lea `check.json` del artefacto, y que las conclusiones admitidas sean exactamente `success` y `failure`. Perfil y clases de job sin cambios.
- `crates/tc2/src/plataforma/mod.rs` y `github.rs`: `leer_revision(rol, pr, origen) -> Paso<RevisionPublicada>` (`sha`, `conclusion`, `summary`, `aceptados` leidos de `raw_details`). Metodo con defecto `no-soportada` (como `entorno_push` y `leer_plan`): GitLab y los dobles no cambian. GitHub reutiliza la lectura de check-runs de `estado_puertas` (mismo filtro por `name` y `app.id`) y pagina `check-runs/{id}/annotations`. Operacion nueva del puerto (lista cerrada de F1-4/F1-13): adenda F6-5, como el precedente `entorno-push`.
- `crates/tc2/src/revision/` (+ operacion `GuardarRevision`): cuando `revision_ia()` da `ok` o `fallo`, `review` llama a `leer_revision` y publica `TASK-NNN/revision/revision-ia-<sha12>.json` (`$defs/informe-tarea`) y `.md` (generado por tc2 con veredicto y tablas por `plano`, sin la prosa del modelo); `exige_red = true` (precedente `plan/guardar.rs`); contenido igual da `sin-cambios`. La linea de `review` gana `hallazgos=<veredicto>` e `informe=<ruta>`.
- Docs: `docs/ci/revision-ia.md` seccion F6-5; adendas F6-5 en F1-15 (clase; la D4 de F6-3 queda sustituida por la regla de la conclusion) y en F1-13/F1-4 (operacion `leer-revision`).

### Orden de construccion

1. Clase, esquema v2, `veredicto` y `anotaciones` puros con fixtures (la regla del rojo demostrada sin GitHub).
2. `tc2 hallazgos veredicto` con linea, ficheros y catalogo i18n.
3. Workflow (`validar`/`publicar`) y lint.
4. `leer_revision` en el puerto y GitHub.
5. `GuardarRevision` y enganche en `review`; guia y adendas.

Cada paso deja el arbol en verde. Pasos 1-3 cumplen «el check refleja los hallazgos»; 4-5, «informe tambien en la carpeta de la tarea».

### Alternativas descartadas

- Comentarios de revision en el PR (`pulls/{n}/comments`): la App del revisor solo tiene `pull_requests: read` (F1-6; D2 de TASK-049); darle `write` le permitiria aprobar, que F1-6 prohibe. Las anotaciones del check se ven igual en «Files changed».
- Que la CI escriba el informe en la rama de estado: exigiria `contents: write` (solo el agente) en un job disparado por un PR; rompe F1-6 y F1-8.
- Bajar el artefacto desde `tc2` en local: el agente no tiene `actions: read` (F1-6), haria falta un lector de zip y la retencion es de 7 dias. El check-run se lee con `checks:read`, que el agente ya tiene.
- Mantener `neutral` con la revision incompleta: GitHub da por cumplido un check obligatorio en `neutral` (`github.rs:528-534`); una revision que no ocurrio (sin saldo o respuesta sin bloque por inyeccion) pasaria la puerta (RR-04). Ver decision 1.

## Troceo

Pasa de 60 min (220). **La persona decidio no trocear**: consigna de Carlos para el H6 (2026-11-21), como en TASK-049 y TASK-050. El orden de construccion deja la regla del rojo probada en `tc2` antes de tocar el workflow, y este antes del puerto y la rama de estado; un corte a medias deja algo coherente.

## Lo que el rol no cubrio

- Criterio 1, por donde se rompe (rol de riesgos, no revisado): anotaciones con ruta que GitHub no acepta o mas de 50 por peticion; PATCH que falla a medias y deja anotaciones incompletas; check-runs duplicados en el mismo SHA por reintentos; `raw_details` alterado al leerlo de vuelta.
- Criterio 2: que «abierto» = «aceptado en esta ronda» es lectura de arquitectura, no de dominio; hasta F6-6 no hay cerrados. La lista de clases y su precedencia se copian de F1-15 sin revisar si el modelo las aplica bien.
- La estrategia de pruebas no la reviso el rol de testing; el presupuesto lo propone arquitectura.
- GitLab: la revision en CI solo existe en GitHub; `leer_revision` queda `no-soportada` alli.
- Inyeccion en el texto del modelo (F6-7): este plan solo garantiza que todo pasa por `plano` y que la prosa no llega a la carpeta de la tarea.

## Riesgos aceptados y que los contiene

- Sin saldo en la API, `revision-ia` sale siempre en rojo «incompleta» en el repo E2E y en cualquier repo que lo exija. Lo contiene: es lo honesto (no hubo revision), el titulo distingue `incompleta` de un rojo por hallazgos, y este repo integra con taskctl 0.16 en local, no por la puerta del PR.
- Contrato v2: respuestas sin `clase` se rechazan. Lo contienen `instrucciones()` generado desde las mismas constantes y el motivo estable `hallazgo-invalido`.
- El informe local se reconstruye desde el check, no del artefacto: los rechazados quedan solo como recuento y motivos del resumen. Lo contienen el veredicto y los aceptados completos en `raw_details`, que es lo que pide F6-5.
- Hasta 12 llamadas a la API en `publicar` (600 aceptados como mucho en lotes de 50), acotado por `TOPE_HALLAZGOS` x 6 huecos.
- El informe se guarda solo al relanzar `review` con el check ya terminado. Lo contiene que `review` converge (`sin-cambios`) y F4-4 lo encadenara.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

Criterio «El check refleja los hallazgos» (3 tests):
1. `crates/tc2/tests/hallazgos.rs` con el binario real y tres revisoras de ejemplo: un IMPORTANTE aceptado da `conclusion=failure veredicto=rojo` y su anotacion es `failure` con `raw_details` igual al aceptado; solo MENOR da `success`; respuesta en prosa (`sin-bloque`) da `failure veredicto=incompleta`; un CRITICO rechazado por falta de evidencia no pone el check en rojo.
2. Lint ampliado de `xtask/src/workflows.rs`: `validar` corre `veredicto`; `publicar` lee `check.json` sin `conclusion` literal y con solo `success`/`failure`. Una mutacion por regla da rojo.
3. **Pesado** (E2E real, `#[ignore]`, red y App del revisor) en `charliebk/taskcodetwo-e2e-github`, sin el modelo: en un PR temporal publica con la App del revisor el `check.json` y lotes de `tc2 hallazgos veredicto` sobre el ejemplo del IMPORTANTE; `revision_ia()` con el agente da `fallo`; `leer_revision` devuelve las anotaciones; limpia PR y rama. Ademas, una ejecucion real del workflow con `consultar` sin saldo publica `failure` `incompleta` (se deja en `## Resultado`, no es test de cargo).

Criterio «Cumple F6-5 y lo que cita» (3 tests):
1. Test de esquema ampliado: `clase` y su lista cerrada coinciden entre esquema y codigo; la salida de `veredicto` y de `informe-tarea` validan contra `hallazgos.schema.json` v2.
2. `crates/tc2/tests/revision.rs` con remoto bare (Git real) y plataforma doble cuyo `leer_revision` devuelve un ejemplo: `review` publica `TASK-NNN/revision/revision-ia-<sha12>.json` y `.md` en la rama de estado; repetido da `sin-cambios`; con el check pendiente no escribe nada.
3. Unitario de `github.rs` con la conexion doble existente: `leer_revision` filtra por `app.id`, pagina anotaciones y rechaza un `raw_details` que no es un `Aceptado`.

Pesado: solo el E2E (criterio 1, test 3). El de `revision.rs` usa Git real en local, como los de TASK-048.

## Lo que necesita decision de una persona

Todas decididas con la opcion recomendada, delegada por Carlos (2026-11-21); sujetas a `taskctl approve`.

1. Conclusion con revision incompleta: `failure` con veredicto `incompleta` (falla en cerrado), no `neutral` (GitHub lo cuenta como verde). Sustituye la D4 de F6-3 (adenda F6-5 en F1-15).
2. «Comentarios» = anotaciones del check-run, sin comentarios de PR ni permisos nuevos.
3. «Clase» obligatoria en el contrato v2, con la lista cerrada de F1-15.
4. Informe en la carpeta de la tarea: lo escribe `tc2 review` en local, leido del check (operacion nueva `leer-revision` con adenda), como `revision/revision-ia-<sha12>.json|.md`, sin la prosa del modelo.
5. Sin trocear.

## Suposiciones no verificadas

- Que la lista de check-runs devuelve `output.summary` y que `check-runs/{id}/annotations` se lee con `checks:read` del agente (docs de la API o E2E).
- Que un PATCH con `output.annotations` anade sin borrar las anteriores (se comprueba en el E2E).
- Que la clave de la App del revisor esta disponible en local para el E2E (ver identidades E2E).
- Que `CheckRunApi` en `github.rs` admite `id` y `output` sin romper `estado_puertas`.
- Que `tc2 revisoras` nunca da cero revisoras en un PR con cambios (si no, el veredicto sale `incompleta`).
