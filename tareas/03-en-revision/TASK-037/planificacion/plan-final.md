# Plan — TASK-037: F4-2 approve con la identidad de la persona

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

`tc2 approve TASK-NNN` es una operacion nueva de la rama de estado, `Aprobar`, con el patron de `Planificar` y `Guardar`: pasa por `estado::transaccion` (con el `ROADMAP.md` en el mismo commit) y `exige_red() = true`, asi que nunca queda pendiente. Solo la ejecuta una persona: la unica fuente de identidad es `identidad::resolver`; con `Quien::Agente` (sin TTY) o un error, `rechazada motivo=identidad rol=persona` antes de tocar el estado. `Quien::Persona` solo existe con TTY, token de persona verificado por `GET /user` e `id` en `.taskcode/personas.yml` de `origin/develop` (`identidad.rs`, `quien_ejecuta` y `resolver`).

El commit de `approve` va **firmado** con la clave de la persona y lleva el trailer `Plan-Blob: <blob de plan-final.md en el arbol de ese commit>` (F1-6 decision 3: «F4-2 firma con la clave de la persona y escribe el trailer `Plan-Blob`; sin firma o sin TTY, rechaza»). Si la firma falla, `rechazada motivo=firma` y no se publica nada. `--decidido-por automatico` no existe en la 2.0 (F1-10 decision 6: la parada `approve` firmado es fija en cualquier modo); pasarlo da `uso` con su diagnostico.

Donde vive cada pieza:
- `crates/tc2/src/plan/aprobar.rs` (nuevo): la operacion `Aprobar` y su orden, dentro de `plan/` porque usa `plantillas::{ruta_plan_final, plan_final}` y el formato de las rondas (`ronda_mayor`). `NOMBRE = "approve"`.
- `Aprobar::aplicar`: `de_otra_persona`, estado `en-diseno`, sin pausa, `plan-final.md` existe y no es el esqueleto (comparado con `plantillas::plan_final` segun los juegos de roles de las rondas, como `planEsPlantilla` en la 0.16). Ya aprobado: `Ok(vec![])` (sale `sin-cambios` antes de `commit-tree`, no se firma otra vez). Si no: `plan_aprobado: true`, `actualizado` y la fila `| … | approve | <modo congelado> | persona | <version> |`.
- `estado::transaccion` (se extiende): metodo nuevo del trait `Operacion` con implementacion por defecto (`fn sello(&self) -> Sello { Sello::Normal }`, `enum Sello { Normal, Firmado { blobs: Vec<(&'static str, String)> } }`). Con `Firmado`, la transaccion no mezcla pendientes en ese commit (D4a) y llama a `construir_commit` con el sello.
- `git::estado::construir_commit` (se extiende): tras `write-tree` resuelve `rev-parse <arbol>:<ruta>` para cada trailer (el blob del propio commit), anade los trailers al mensaje y `-S` a `commit-tree`. En cada reintento de CAS se firma otra vez (se acepta: `approve` es raro).
- `git::puerto` (se extiende con un solo uso): variante interactiva solo para `commit-tree -S` (stdin y stderr heredados, para que la persona vea la peticion de frase de paso o de presencia FIDO) y `comprobar_argumentos` admite `-S` unicamente ahi.
- `lib.rs`: despacho `approve` con `de_una_tarea`; catalogo i18n con uso y rechazos.
- Hueco de F4-1 (D1a): `tc2 approve TASK-NNN --pedir-cambios <fichero>` publica `brainstorm/cambios-pedidos-<n>.md` en la ronda abierta; `ronda_mayor` da por cerrada la ronda con ese testigo y el siguiente `tc2 plan` abre la `n+1`. No concede nada, asi que acepta tambien `Quien::Agente` (`decidido_por: agente`), como en la 0.16.

Orden: 1) `Sello`, `construir_commit` y la variante interactiva del puerto, sin uso; 2) `Aprobar` con el rechazo por identidad, el esqueleto y la idempotencia; 3) orden, despacho y catalogo; 4) `--pedir-cambios` y el testigo en `ronda_mayor`; 5) adenda F4-2 en F1-6 (o en la de F4-1 de F1-10).

Duracion estimada: 115 min
Senales: extension del trait de la transaccion y de `construir_commit` (frontera compartida), variante interactiva del puerto Git, operacion nueva con catalogo i18n, una prueba pesada con firma SSH real y `--pedir-cambios` (D1a).

## Troceo

### Parte a: firma en la transaccion (Sello, construir_commit, puerto interactivo)
### Parte b: operacion Aprobar, orden, catalogo, pruebas y --pedir-cambios

No se trocea: Carlos pidio el H4 sin trocear (2026-11-18).

## Lo que el rol no cubrio

- Riesgos (sin rol): fallos de firma en Windows (OpenSSH del sistema frente al de Git for Windows, `gpg.format`, `user.signingkey` sin configurar, FIDO con toque, `ssh-agent` con la clave cacheada: limite RN-06 de F1-6), corte de red entre firmar y publicar, y que `commit.gpgSign=true` global ya haga firmar hoy cada `commit-tree` de las demas operaciones.
- Testing (sin rol): plan minimo; no se verifica el «Verified» de GitHub.
- Dominio (sin rol): D2 (approve en carriles sin plan aprobado), D1 (si «pedir cambios» lo puede transmitir el agente) y la regla de una hora de la 0.16 sin modo automatico.
- La comprobacion de la firma frente a la plataforma (reglas 1 a 4 con `Firma::cuenta_como_aprobacion` y la 5 con `Plan-Blob`) es del check `carril` (F6), no de `approve` (D3).

## Riesgos aceptados y que los contiene

- `plan_aprobado: true` no prueba nada por si mismo (el agente puede empujar a la rama de estado, F1-6 D3a): lo contiene la firma y el check `carril` de F6; hasta entonces (y en este repo, RR-14) la garantia es local.
- Volver a firmar en cada reintento de CAS: un toque o una frase mas con contencion; se acepta por la frecuencia.
- Las pendientes no se mezclan en el commit firmado: se publican en la siguiente operacion normal.
- `Quien::Persona` se basa en token y TTY: un agente con TTY y token pasaria la identidad; lo contiene la firma con FIDO o frase sin cache (F1-6 D4a/D4b).
- La cache de identidad editable no aprueba nada: `approve` sin TTY rechaza.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio):
- Criterio 1 («sin identidad de persona → rechazo»): 2 tests. Unitario (ligero): `approve` con `Quien::Agente` y con `Err(rechazada motivo=identidad cache=falta)`: `approve rechazada motivo=identidad rol=persona` y ni transaccion ni `commit-tree` en el registro del puerto. **Pesado**: repo temporal con remoto bare, tarea `en-diseno` con plan redactado y `quien` de agente: rechazo y la rama de estado del remoto con el mismo sha.
- Criterio 3 (F4-2, RR-04, F1-6): 3 tests. **Pesado**: clave SSH desechable sin frase (`gpg.format=ssh`, `user.signingkey`) y `Quien::Persona`: `approve ok`; `git verify-commit` en verde con un `allowed_signers` de esa clave, `%(trailers:key=Plan-Blob)` igual a `rev-parse <commit>:TASK-NNN/planificacion/plan-final.md`, `plan_aprobado: true` y la fila `approve | … | persona`; con una clave que no firma, `rechazada motivo=firma` y el remoto intacto. Unitario (ligero): `Aprobar::aplicar` sobre `EnMemoria`: esqueleto (`plan-esqueleto`), otra persona (`de-otra-persona`), `planificada` (`estado-no-valido`), ya aprobado `Ok(vec![])`. Unitario (ligero): `--decidido-por automatico` y argumentos de mas dan `uso`; `exige_red() = true` y `sello() = Firmado` con `("Plan-Blob", ruta_plan_final)`.
- D1a: 1 test ligero: con `cambios-pedidos-1.md`, `Planificar` con los mismos roles abre la ronda 2.
- Criterio 2 («test»): cubierto por los anteriores.

## Lo que necesita decision de una persona

Carlos dejo dicho el 2026-11-18 que el H4 se hace con la opcion recomendada y sin trocear; se aplican:

- D1 hueco de F4-1: (a) incluir `tc2 approve TASK-NNN --pedir-cambios <fichero>` y el testigo en `ronda_mayor`. **Elegida.**
- D2 approve en carriles sin plan aprobado: (a) permitirlo (endurece, nunca baja). **Elegida.**
- D3 comprobar la firma frente a la plataforma tras publicar: (a) no en `approve`; lo hace `carril` (F6). **Elegida.**
- D4 pendientes en el commit firmado: (a) no mezclarlas. **Elegida.**

Suposiciones a confirmar en la implementacion: que `git commit-tree -S` firma con `gpg.format=ssh` y `user.signingkey` como `git commit -S` y respeta `commit.gpgSign`; que con stdin nulo y stderr capturado la frase de paso y el aviso FIDO no llegan a la persona en Windows; que el `modo` de la fila `approve` es el congelado de la ultima fila `plan`.
