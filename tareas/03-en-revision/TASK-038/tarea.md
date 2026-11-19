---
id: TASK-038
titulo: "F4-3 Aprobacion de un hito en lote"
tipo: feature
sprint: 4
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-038-f4-3-aprobacion-de-un-hito-en-lote
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 758815
tokens_implementacion: 16653897
tokens_revision: 6779674
creado: 2026-11-14
actualizado: 2026-11-19
dependencias: [TASK-037]
---
## Objetivo

Aprobación de un hito en lote

Tarea F4-3 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F4-2.

## Criterios de aceptacion
- [x] Diez planes aprobados de una vez, con nombre y fecha
- [x] Cumple lo que pide la tarea F4-3 del plan y los requisitos que cita

## Resultado

Hecho: `tc2 approve --hito <n>` (operacion `AprobarHito` en `plan/aprobar.rs`; `n` = campo `sprint`, acepta `4` o `H4`) aprueba de una vez los planes de las tareas abiertas del hito en un unico commit firmado con un trailer `Plan-Blob` por tarea (blob del arbol del propio commit), todo o nada (D2a): si alguna no se puede aprobar, `rechazada motivo=no-listas tareas=ID:motivo,...` y no se firma nada. Nombre y fecha (D4a): la fila `approve | <modo> | persona` con el mismo instante en cada tarea, y el nombre en la firma y en la linea `Operacion: approve-hito|<n>|<id>|<login>` del commit. Tope de 20 tareas que cambiarian (D5a). Solo persona (RR-04): sin ella, `rechazada motivo=identidad rol=persona` sin lanzar git. `aprobable` se extrajo de `Aprobar::aplicar` sin cambiar su contrato (los tests de F4-2 no se editaron). Adendas F4-3 en F1-6 (N trailers; la regla 5 del check `carril` de F6 pasara a «el blob actual esta entre los trailers») y F1-16 (D7a). `cargo xtask verificar --perfil rapida` en verde.

Divergencia con el plan: en `estado/transaccion.rs` el sello se pedia antes de `aplicar`; ahora el primero solo decide las pendientes y se vuelve a pedir justo antes de `construir_commit_sellado` en cada intento del CAS (las demas operaciones devuelven el mismo sello las dos veces; documentado en el trait y en la adenda).

Implementado por un subagente con el plan aprobado (D1a-D5a); revisado por dos agentes independientes.

No verificado: la firma con una persona real y FIDO en Windows; el CAS con el lote cambiando entre dos intentos.

Revision ronda 1: aprobada con correcciones por los dos, sin CRITICOS ni IMPORTANTES. MEN-1 (los dos): un `tarea.md` ilegible se saltaba en silencio y el lote se firmaba sin el; corregido en 6eba760 fallando cerrado (cuenta como `ilegible` en el todo o nada), con test y verificado con `verificar-menores`.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-19T06:28:20Z | plan | automatico | persona | 0.16.1 |
| 2026-11-19T06:32:55Z | approve | automatico | persona | 0.16.1 |
| 2026-11-19T06:32:57Z | start | automatico | persona | 0.16.1 |
| 2026-11-19T07:19:24Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-19T06:32:55Z | diseno | agente:abf932fe85759b33e | 32 | 65731 | 677119 | 15933 | 22 | 758815 | - | - |
| 2026-11-19T07:19:24Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 16 | 13092 | 5438518 | 3872 | 0 | 5455498 | - | - |
| 2026-11-19T07:19:24Z | implementacion | agente:a64eb252bf5410ce1 | 146 | 196105 | 10948704 | 53444 | 37 | 11198399 | - | - |
| 2026-11-19T07:30:49Z | revision | agente:acfcf45fa7f570dd4 | 48 | 138034 | 2820294 | 11695 | 9 | 2970071 | 1 | rust-wasm-reviewer |
| 2026-11-19T07:30:49Z | revision | agente:af5a7fb131100fd60 | 62 | 144046 | 3649490 | 16005 | 3 | 3809603 | 1 | code-quality-reviewer |

## Autorrevision

- rust-wasm-reviewer — 5112652
- code-quality-reviewer — 5112652

## Verificacion ligera

- 2026-11-19T07:33:23Z: ronda 1; commit revisado 5ae20d8b11347556bddbfe0fe074e9827f4825f5; commit verificado 6eba76090902d1e281d93651d09a05028c5b1d9c; ficheros: crates/tc2/src/plan/aprobar.rs; corregidos: MEN-1, MEN-1; pendientes: ninguno
