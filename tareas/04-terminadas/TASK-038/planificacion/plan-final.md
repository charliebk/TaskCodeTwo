# Plan — TASK-038: F4-3 Aprobacion de un hito en lote

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Operacion nueva `AprobarHito` en `crates/tc2/src/plan/aprobar.rs`, que reutiliza lo de F4-2 y produce **un unico commit firmado** con un trailer `Plan-Blob` por tarea. Orden: `tc2 approve --hito <n>` (`n` = campo `sprint`; se acepta `4` o `H4`). Todo o nada: si alguna tarea del hito no se puede aprobar, no se firma nada y la salida dice cuales fallan y por que.

Que se crea y que se extiende:
- **Se extiende `Aprobar` sin cambiar su contrato**: las comprobaciones de `Aprobar::aplicar` salen a una funcion pura `aprobable(t, cuerpo, plan, rutas, asignado, instante, modo) -> Result<Option<Cambio>, Fallo>` (`None` si ya estaba aprobada). `Aprobar` la llama y se comporta igual (`sin-cambios`, `dueno`, anotacion `approve|ID|id|login`), como fijan sus tests.
- **Se crea `AprobarHito { hito, asignado, ahora, modo, fallos: RefCell<..> }`** (precedente `dueno`), que solo construye `Quien::Persona` (`nueva(..) -> Option<Self>`, RR-04). `aplicar`: lee todas las tareas con `estado::tareas(estado, &estado.rutas())` y filtra por `sprint == hito`; el lote son las de ese sprint no `terminada` ni `descartada` y sin `plan_aprobado`; lee sus `plan-final.md` con `leer_varios` y llama a `aprobable`. Todas las filas comparten el instante. Si alguna falla: `rechazada motivo=no-listas` con `tareas=ID:motivo,...` (motivos de F4-2: `sin-plan`, `plan-esqueleto`, `pausada`, `estado-no-valido`, `de-otra-persona`, `formato-1`). Ademas `hito-vacio` sin ninguna abierta y `demasiadas max=20` si se pasa del tope. Todas ya aprobadas: `Ok(vec![])` (`sin-cambios`, no firma).
- **`sello()`** da `Sello::Firmado { trailers }` con un `(PLAN_BLOB, ruta_plan_final(id))` por tarea en orden de ID. No se toca `git/estado.rs`: `construir_commit_sellado` ya escribe una linea por trailer con el blob del arbol del propio commit. Como el lote solo se conoce en `aplicar`, un `RefCell<Vec<String>>` de IDs que rellena `aplicar` y lee `sello()`; antes de implementar se confirma en `transaccion.rs` que `sello()` se llama despues de `aplicar` en cada intento (si no, otra forma).
- **«Con nombre y fecha»** sin formatos nuevos (D4a): cada tarea recibe la fila `| <instante> | approve | <modo congelado> | persona | <version> |` y `actualizado`; el nombre esta en la firma de la persona y en la linea `Operacion: approve-hito|<n>|<id>|<login>` del commit.
- **Tope** `MAX_HITO = 20` (D5a), contando solo las que cambiarian, comprobado en `aplicar` antes de firmar (el criterio pide 10; el lote real de F1-16 firmaba 14). Coste: un `rev-parse` por trailer.
- **CLI**: `plan::orden_approve` acepta `--hito <n>` en lugar del ID; `uso` para `--hito` con ID, con `--pedir-cambios` o no numerico. `aprobar_hito_en(e, hito)` como `paso_aprobar`: identidad primero (sin persona, `rechazada motivo=identidad rol=persona` sin git), luego `traer`, `config_de_base` y `escribir`. Salida con `hito=`, `aprobadas=ID,...` y, si se rechaza, `tareas=ID:motivo,...`. Textos en `i18n::catalogo::plan` (es/en). `lib.rs` no cambia.
- Sin dependencia de `roadmap`: el hito se resuelve por el `sprint` de cada tarea.

Orden: 1) extraer `aprobable` con los tests de F4-2 en verde sin tocarlos; 2) `AprobarHito` con `aplicar`, `sello`, `anotacion` y unitarios con `EnMemoria`; 3) CLI, catalogo y `aprobar_hito_en`; 4) test de integracion firmado y adendas F4-3 en `docs/adr/F1-6-*` (N trailers en un commit; la regla 5 del check `carril` de F6 pasara a «el blob actual esta entre los trailers del commit que pone `plan_aprobado`») y en `docs/adr/F1-16-*` (D7a: la firma previa al lote puede ser la de este comando).

Descartadas: N commits firmados (diez firmas, no atomico) y generalizar `Aprobar` a `Vec<id>` (cambia contratos que fijan los tests de F4-2).

Duracion estimada: 55 min
Senales: 1 dependencia (TASK-037); reutiliza `Sello::Firmado` multi-trailer, `estado::tareas` y el arnes de firma de `tests/aprobar.rs`; el test pesado de firma es lo que mas tarda.

## Lo que el rol no cubrio

- Riesgos (sin rol): una tarea que entra en el sprint entre que la persona mira el roadmap y firma (el CAS reaplica `aplicar` y el lote cambia); dos planes identicos comparten blob.
- Testing (sin rol): propuesta minima por criterio.
- Dominio (sin rol): si aprobar un hito bloquea por tareas aun `planificada` sin plan (D2) y que «con nombre» es el de la persona (D4).
- El check `carril` (F6) tendra que aceptar varios `Plan-Blob` en un commit: no se implementa aqui; va en la adenda.
- C2: F4-3 solo cita F4-2/RR-04, que se respeta; F1-16 D7a no impone forma; F1-10 (riesgo alto no entra en lote) afecta a lanzar el lote, no a aprobar.

## Riesgos aceptados y que los contiene

- El lote se calcula en `aplicar` y puede firmar una tarea inesperada: lo contienen la salida `aprobadas=` y el tope de 20; `--pedir-cambios` no deshace `plan_aprobado`.
- `Plan-Blob` sin ID de tarea: dos planes identicos comparten blob; el commit solo cambia los `tarea.md` del lote.
- Un `rev-parse` por trailer: acotado por `MAX_HITO`.
- Firma real con persona y FIDO en Windows: sin verificar, como en F4-2; firma una sola vez.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio):
- C1 (diez de una vez, con nombre y fecha): 3 tests. Unitario `aplicar_hito_aprueba_diez` (`EnMemoria` con 10 del sprint 4 en `en-diseno` con plan, una ya aprobada, una terminada y una de otro sprint: 10 `Cambio`, misma fila e instante, `sello()` con 10 trailers en orden). Unitario `hito_todo_o_nada` (sin plan y en pausa: `no-listas` con las dos; `hito-vacio`, `demasiadas` con 21, todas aprobadas = `Ok(vec![])`, otra persona). **Pesado** en `tests/aprobar.rs`: `tc2 approve --hito 4` con 10 tareas y clave ssh desechable: un solo commit nuevo, `verify-commit` correcto, 10 `Plan-Blob` iguales a `rev-parse <commit>:TASK-NNN/planificacion/plan-final.md`, linea `Operacion: approve-hito|4|<id>|<login>` y `aprobadas=` con las 10.
- C2 (F4-3, RR-04, F4-2): 2 tests. Unitario: `AprobarHito::nueva` con `Quien::Agente` da `None`; los tests de F4-2 siguen sin editar. Integracion ligera sin firma: `--hito` con agente da `rechazada motivo=identidad` sin git; `--hito` con ID, con `--pedir-cambios` y `--hito x` dan `uso`.

## Lo que necesita decision de una persona

Carlos dejo dicho el 2026-11-18 que el H4 se hace con la opcion recomendada y sin trocear; se aplican:

- D1 forma: (a) solo `tc2 approve --hito <n>`. **Elegida.**
- D2 tarea no lista: (a) todo o nada sobre todas las abiertas del hito (incluidas `planificada` sin plan), listando las que fallan (encaja con F1-16 D7a). **Elegida.**
- D3 registro: (a) un commit firmado con N trailers `Plan-Blob: <blob>` (formato de F4-2). **Elegida.**
- D4 nombre y fecha: (a) fecha en la fila de Transiciones; nombre en la firma y en la linea `Operacion:`. **Elegida.**
- D5 tope: (a) constante 20, contando solo las que cambiarian. **Elegida.**
