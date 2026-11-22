# Informe de revision — TASK-051 (ronda 1)

- Commit revisado: 02000f5a7c78e8404c5a4257bdb7eb7ecc6bfa1b
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/plataforma/github.rs:662 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/hallazgos/mod.rs:491 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### MEN-1 — `leer_revision` no contrasta el titulo del check con las anotaciones leidas

`leer_revision` toma el dictamen del titulo (`Dictamen::de_titulo`) y los aceptados de las anotaciones, y `InformeTarea::nuevo` cuenta los recuentos desde las anotaciones. Nada comprueba que ambos casen. Si el check queda publicado con menos anotaciones de las que dice su titulo (un PATCH de lote que falla tras el POST, o los lotes que pasan del `seq 1 20` del workflow), el informe que `tc2 review` guarda en la carpeta de la tarea sale incoherente y sin aviso.

Reproduccion (clon en el commit revisado, test temporal anadido a `tests/plataforma.rs` con el doble HTTP existente): check completado con `conclusion=success`, titulo `revision-ia: rojo (criticos=3 importantes=2 menores=0)` y `[]` en `check-runs/5/annotations`. `leer_revision` devuelve `Ok`; `InformeTarea::nuevo` produce `conclusion=success veredicto=rojo criticos=0 importantes=0`, que `GuardarRevision` publicaria tal cual. Esperado: `sin-leer` / error por incoherencia (el plan promete que un `raw_details` alterado falla cerrado).

### MEN-2 — `raw_details` de la anotacion sin tope: pasa del limite de GitHub y el POST entero se rechaza

`anotaciones()` limita `title` (255) y `message` (4000 por `plano`), pero `raw_details` es el JSON completo del `AceptadoDe` y ni `descripcion` ni la evidencia tienen tope en `validar`. GitHub admite 64 KB en `raw_details`.

Reproduccion: salida de `validar` a mano con un unico IMPORTANTE de `descripcion` de 70000 caracteres; `tc2 hallazgos veredicto --salida out --hallazgos v.json` sale `ok veredicto=rojo ... lotes=1` y `check.json` lleva una anotacion con `raw_details` de 70210 bytes. El POST de `publicar` recibiria un 422, el paso sale con `exit 1` y no se publica ningun check (ni siquiera el `failure` de `incompleta`). Falla en cerrado por ausencia del check, pero sin el veredicto ni el motivo. Solucion: topar o rechazar en `validar` los campos de texto, o comprobar el tamano de `raw_details` en `anotaciones`.

## Lecturas fuera del contexto permitido

- `.github/workflows/revision-ia.yml` (paso `publicar`) y `crates/tc2/src/hallazgos/mod.rs` completo: para comprobar como se publica el check y los topes de `validar` (MEN-1 y MEN-2).

## Alcance
- Ficheros revisados: los 13 de la peticion.
- Contexto adicional pedido: el workflow `revision-ia.yml` (ver arriba).
- Limite JS-WASM: sin cambios; el diff no toca firmas `#[wasm_bindgen]`.

## Reproduccion
- Clon: `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-051-r1-rust` en 02000f5 (borrado al terminar).
- Suite: `cargo test -p tc2 --test hallazgos --test revision --test plataforma` en verde (hallazgos 5, plataforma 7, revision 4); unitarios de `hallazgos` 8 en verde; `cargo test -p xtask` 34 en verde. No se repitio `verificar --perfil rapida` (declarada en verde); sin red ni API de Anthropic.
- Casos construidos: los de MEN-1 y MEN-2.
- Mutacion: no se hizo.

## Revisado sin hallazgos
- Clase y esquema v2, `veredicto` (incompleta/rojo/verde), `anotaciones` en lotes, deserializacion con `deny_unknown_fields`, `Rechazo` con motivo contrastado.
- `GuardarRevision`: rutas derivadas de sha validado e id de tarea, idempotencia, regla de duenio.
- Lint de xtask para `publicar`.
- Sin `unsafe`, sin `unwrap` ni indexado sobre datos de entrada en el codigo nuevo.

Veredicto propuesto: aprobada-con-correcciones
