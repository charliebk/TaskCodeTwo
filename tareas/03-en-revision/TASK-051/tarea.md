---
id: TASK-051
titulo: "F6-5 Hallazgos estructurados"
tipo: feature
sprint: 6
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-051-f6-5-hallazgos-estructurados
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 2257604
tokens_implementacion: 46031187
tokens_revision: 5995294
creado: 2026-11-14
actualizado: 2026-11-22
dependencias: [TASK-050]
---
## Objetivo

Hallazgos estructurados (JSON + comentarios); un CRÍTICO o IMPORTANTE abierto deja el check en rojo; informe también en la carpeta de la tarea

Tarea F6-5 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F6-4.

## Criterios de aceptacion
- [x] El check refleja los hallazgos
- [x] Cumple lo que pide la tarea F6-5 del plan y los requisitos que cita

## Resultado

Hallazgos estructurados (F6-5). El veredicto lo calcula `tc2` y el workflow solo lo transporta.
- Contrato v2 de hallazgos (`crates/tc2/src/hallazgos/`, esquema `hallazgos.schema.json` v2): `clase` obligatoria con la lista cerrada de F1-15; funciones puras `veredicto` y `anotaciones` (lotes de 50, `raw_details` acotado a 64 KB, mensaje nunca vacio).
- `tc2 hallazgos veredicto` escribe `check.json` y `anotaciones-<k>.json`. Regla: `failure` (rojo) con algun CRITICO o IMPORTANTE aceptado; `failure` (incompleta) si alguna revisora no dejo respuesta valida; `success` en otro caso. Ya no se publica `neutral`, que GitHub cuenta como verde (sustituye la D4 de F6-3).
- `revision-ia.yml`: `validar` corre `veredicto`; `publicar` hace POST de `check.json` y PATCH por lote. Sin artefacto, o si la API rechaza el veredicto, publica `failure` incompleta. El lint de `xtask` exige todo ello (`success` solo una vez, en el `case` que copia `check.json`).
- Operacion nueva del puerto `leer-revision` (GitHub; `no-soportada` en GitLab y dobles): lee el check de la App del revisor, pagina anotaciones y exige que titulo, conclusion y aceptados casen. `tc2 review` guarda `TASK-NNN/revision/revision-ia-<sha12>.json|.md` en la rama de estado (sin la prosa del modelo) y su linea gana `hallazgos=` e `informe=`.
- Guia `docs/ci/revision-ia.md` y adendas F6-5 en F1-15, F1-13 y F1-4.

Decisiones (opcion recomendada, delegada por Carlos, 2026-11-21): sin trocear; incompleta = `failure`; «comentarios» = anotaciones del check, sin permisos nuevos; clase obligatoria; informe leido del check por `tc2 review`.

Evidencia (2026-11-21/22):
- C1 en `tc2`: `crates/tc2/tests/hallazgos.rs::el_check_refleja_los_hallazgos` con el binario real (IMPORTANTE da rojo, solo MENOR verde, prosa incompleta, CRITICO rechazado no pone rojo); 7 mutaciones en rojo.
- C1 en el workflow: lint de `xtask` con mutaciones por regla en rojo (34 tests).
- C1 E2E real en `charliebk/taskcodetwo-e2e-github` (`revision_ia_publica_el_veredicto_y_review_lo_lee`, `#[ignore]`): la App del revisor 5846307 publico el check con 50 anotaciones y el PATCH anadio la 51 sin borrar; con la App del agente 5832411, `revision-ia` dio `fallo` y `leer-revision` devolvio las 51. PR y rama temporales borrados.
- C2: el esquema v2 y el codigo coinciden (clases, `required`, `maxLength`); `tests/revision.rs::review_guarda_el_informe_de_la_revision_ia` con remoto bare real publica el informe, repetido da `sin-cambios` y con el check pendiente no escribe; `tests/plataforma.rs::leer_revision_filtra_por_origen_y_pagina`.
- `cargo xtask verificar --perfil rapida` en verde sobre 02000f5 (unos 29 min). Las correcciones de la ronda 1 (4f7ba28) se comprobaron con `cargo test -p tc2 --lib` (378), `--test hallazgos --test revision --test plataforma` (16), `cargo test -p xtask` (34), clippy `-D warnings`, fmt y `taskctl verificar-menores` (e213191). `finish` vuelve a verificar.

Sin comprobar: la ejecucion real del workflow sin saldo (publicaria `failure` incompleta; no hay saldo en la API y no se pondra); que GitHub acepte rutas y lineas de un PR con diff real (el E2E usa la linea 1 de un fichero del PR); un PATCH que falle a medias o mas de ~1050 anotaciones dejan el check incompleto (`leer-revision` lo detecta y falla cerrado, el workflow no lo repara).

Revision por pares:
- Ronda 1: rust-wasm-reviewer y code-quality-reviewer, aprobada-con-correcciones, sin CRITICO ni IMPORTANTE. Los seis MENOR se corrigieron en 4f7ba28 y los reconocio `verificar-menores`.
  - Rust: MEN-1 `leer_revision` no comprobaba que titulo y anotaciones casaran; MEN-2 `raw_details` sin tope de 64 KB.
  - Calidad: MEN-1 el lint de `publicar` se saltaba con la clave entrecomillada; MEN-2 dos `required` del esquema sin test; MEN-3 `publicar` sin salida si la API rechaza el veredicto, y `message` vacio posible; MEN-4 orden del resumen mal descrito en la guia.

## Divergencias

- `tc2 hallazgos veredicto` gana `--extra <txt>` para que el resumen lleve la seleccion y el estado de cada revisora sin logica jq en `publicar`.
- `raw_details` es `{revisora, hallazgo}` (`AceptadoDe`) en vez del `Aceptado` a secas, para no perder la revisora (medicion por revisora de F1-15 D7a).
- `failure` si puede aparecer literal en `publicar` (fallback de revision incompleta); `success` nunca.
- La adenda de `leer-revision` va en F1-13 con nota en F1-4 (el plan decia F1-13 o F1-4).
- El rechazo por titulo incoherente reutiliza el mensaje i18n de `raw_details` invalido: un mensaje propio exigia tocar el catalogo fuera de los ficheros de los MENOR. Queda como deuda menor.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-21T22:33:21Z | plan | automatico | persona | 0.16.1 |
| 2026-11-21T22:43:05Z | approve | automatico | persona | 0.16.1 |
| 2026-11-21T22:43:07Z | start | automatico | persona | 0.16.1 |
| 2026-11-22T00:01:08Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-21T22:42:51Z | diseno | agente:a682e2bdc819cd773 | 58 | 119893 | 2108867 | 28786 | 42 | 2257604 | - | - |
| 2026-11-22T00:01:08Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 24 | 8484 | 2178763 | 5596 | 0 | 2192867 | - | - |
| 2026-11-22T00:01:08Z | implementacion | agente:a1fed6bc99f9de948 | 306 | 817675 | 42892641 | 127698 | 56 | 43838320 | - | - |
| 2026-11-22T00:10:00Z | revision | agente:a938c42a3c6b7c9cc | 26 | 147288 | 1517577 | 11798 | 2 | 1676689 | 1 | rust-wasm-reviewer |
| 2026-11-22T00:10:00Z | revision | agente:a290814c6e4245a2a | 64 | 161449 | 4132686 | 24406 | 2 | 4318605 | 1 | code-quality-reviewer |

## Verificacion ligera

- 2026-11-22T00:24:55Z: ronda 1; commit revisado 02000f5a7c78e8404c5a4257bdb7eb7ecc6bfa1b; commit verificado 4f7ba280fe89791a52e5745ba2f936f347d49154; ficheros: .github/workflows/revision-ia.yml, crates/tc2/esquemas/hallazgos.schema.json, crates/tc2/src/hallazgos/mod.rs, crates/tc2/src/plataforma/github.rs, docs/ci/revision-ia.md, xtask/src/workflows.rs; corregidos: MEN-1, MEN-2, MEN-3, MEN-4, MEN-1, MEN-2; pendientes: ninguno
