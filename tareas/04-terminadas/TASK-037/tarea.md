---
id: TASK-037
titulo: "F4-2 approve con la identidad de la persona"
tipo: feature
sprint: 4
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-037-f4-2-approve-con-la-identidad-de-la-pers
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 2138683
tokens_implementacion: 24406246
tokens_revision: 4219489
creado: 2026-11-14
actualizado: 2026-11-19
dependencias: [TASK-007, TASK-036]
---
## Objetivo

`approve` con la identidad de la persona; el agente no puede aprobar (RR-04)

Tarea F4-2 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-6, F4-1.

## Criterios de aceptacion
- [x] Sin identidad de persona → rechazo
- [x] test
- [x] Cumple lo que pide la tarea F4-2 del plan y los requisitos que cita

## Resultado

Hecho: `tc2 approve TASK-NNN` (operacion `Aprobar` en `plan/aprobar.rs`), que solo puede lanzar una persona: con `Quien::Agente` (sin TTY) o sin identidad, `approve rechazada motivo=identidad rol=persona` sin lanzar ningun git (la identidad va antes del `fetch`). El commit de `approve` va firmado (`commit-tree -S` por `Git::ejecutar_firmado`, la unica llamada que firma; el puerto prohibe `-S`/`--gpg-sign` en cualquier otro `commit-tree`, en `commit`, y `-s`/`--sign`/`-u` en `tag`) y lleva el trailer `Plan-Blob` con el blob de `plan-final.md` del arbol del propio commit (F1-6 decision 3). Sin firma, `rechazada motivo=firma` con el arreglo (configurar `gpg.format` y `user.signingkey`) y nada publicado. `--decidido-por automatico` da `uso` (F1-10 decision 6). Con `Sello::Firmado` las pendientes no entran en el commit firmado (D4a). Ya aprobado: `sin-cambios` sin volver a firmar. Plan esqueleto, otra persona, pausa o estado no valido: rechazada. `--pedir-cambios <fichero>` (D1a) publica `brainstorm/cambios-pedidos-<n>.md`, cierra la ronda y el siguiente `tc2 plan` abre la `n+1` citando ese fichero (cierra el hueco declarado en TASK-036). Adendas en F1-6, F1-4 y F1-10. `cargo xtask verificar --perfil rapida` en verde.

Descubierto en Windows (Git 2.56.0.windows.4): `commit-tree -S` firma con `gpg.format=ssh` y una clave ed25519; `commit-tree` NO respeta `commit.gpgSign` (por eso el `-S` va explicito y ninguna otra operacion firma). La configuracion global de git de Carlos no tiene `gpg.format` ni `user.signingkey`: un `tc2 approve` real saldra `motivo=firma` hasta configurarlos.

Divergencias con el plan (en las adendas): stderr de `commit-tree -S` capturado (stdin heredado para la frase de paso); `--pedir-cambios` no escribe fila en `## Transiciones` (quien lo pidio queda en la anotacion del commit); `Sello` vive en `git::estado`; el esqueleto se detecta comparando con `plan_final` para 0 a 5 roles.

Implementado por un subagente con el plan aprobado (D1a-D4a); revisado por dos agentes independientes.

No verificado: la frase de paso y el toque FIDO con una persona real en el terminal de Windows; el «Verified» de GitHub; el paso por el binario con `Quien::Persona` (necesita terminal y token reales; probado con la biblioteca).

Revision ronda 1: aprobada con correcciones por los dos, sin CRITICOS ni IMPORTANTES. Corregidos en 9399fbe: MEN-1 de los dos (D4a sin test; el test va en `tests/estado_casos.rs`, asi que `verificar-menores` lo deja en `menores-pendientes.md` aunque esta corregido por test); MEN-2 rust (test de `Plan-Blob` con un cambio en el fichero del trailer); MEN-3 rust (comentario del puerto y prohibicion de firmar en `commit`/`tag`); MEN-2 calidad (la re-planificacion cita `cambios-pedidos-<n-1>.md`); MEN-3 calidad (la adenda de F1-4 lista `no-se-lee`).

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-19T04:59:16Z | plan | automatico | persona | 0.16.1 |
| 2026-11-19T05:05:00Z | approve | automatico | persona | 0.16.1 |
| 2026-11-19T05:05:01Z | start | automatico | persona | 0.16.1 |
| 2026-11-19T05:47:44Z | review | automatico | persona | 0.16.1 |
| 2026-11-19T06:28:03Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-19T05:04:59Z | diseno | agente:ac9a3cbc6b8d25b98 | 48 | 125281 | 1995222 | 18132 | 31 | 2138683 | - | - |
| 2026-11-19T05:47:44Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 16 | 14032 | 5087140 | 4043 | 0 | 5105231 | - | - |
| 2026-11-19T05:47:44Z | implementacion | agente:a3a1ca06d00d1cb52 | 150 | 328040 | 18896870 | 75955 | 28 | 19301015 | - | - |
| 2026-11-19T05:56:44Z | revision | agente:a1a31f66f13841e9a | 28 | 141610 | 1595487 | 12020 | 4 | 1749145 | 1 | rust-wasm-reviewer |
| 2026-11-19T05:56:44Z | revision | agente:a5663b5c690198b6c | 40 | 148224 | 2306425 | 15655 | 3 | 2470344 | 1 | code-quality-reviewer |

## Autorrevision

- rust-wasm-reviewer — 7b98d98
- code-quality-reviewer — 7b98d98

## Verificacion ligera

- 2026-11-19T06:07:10Z: ronda 1; commit revisado b9a6a55f86af56fe615da1af20a1480ea2d9ae0a; commit verificado 9399fbeb1b6a5e71ac294c11d53f0f9c2dbcaf11; ficheros: crates/tc2/src/git/estado.rs, crates/tc2/src/git/puerto.rs, crates/tc2/src/plan/plantillas.rs, crates/tc2/tests/aprobar.rs, crates/tc2/tests/estado_casos.rs, docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md; corregidos: MEN-2, MEN-3, MEN-2, MEN-3; pendientes: MEN-1, MEN-1

## Duracion

- Estimada: 115 min
- Real (de la rama abierta al cierre con merge, sin pausas): 83 min
- Paso de la hora: 83 min reales frente a 115 estimados
- Cierre (finish): 1222 s, de 2026-11-19T06:07:41Z a 2026-11-19T06:28:03Z
