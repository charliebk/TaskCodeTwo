---
id: TASK-020
titulo: "F2-5 Modelo de tarea y parser/serializador de frontmatter YAML valido"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-020-f2-5-modelo-de-tarea-y-parser-serializad
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 325030
tokens_implementacion: 11473406
tokens_revision: 8564420
creado: 2026-11-14
actualizado: 2026-11-15
dependencias: [TASK-010]
---
## Objetivo

Modelo de tarea y parser/serializador de frontmatter YAML válido

Tarea F2-5 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-9.

## Criterios de aceptacion
- [x] Ida y vuelta con un parser estándar
- [x] compatible con tareas de la 0.16
- [x] Cumple lo que pide la tarea F2-5 del plan y los requisitos que cita

## Resultado

**Qué se hizo.** Módulo nuevo `crates/tc2/src/tarea/` (hoja pura, sin git ni red):
- `tarea::leer` distingue formato 2 (`formato: 2`, `Tarea`) y formato 1 de la 0.16 (sin `formato`, `TareaV1`, los 22 campos de `TASK_FIELD_ORDER`), que se lee pero no se escribe (F1-9 punto 3).
- `Tarea::escribir` emite la forma fija de F1-9: cadenas siempre entre comillas dobles con `\` y `"` escapados; control, U+2028/2029, U+FEFF, U+FFFE y U+FFFF como `\uXXXX`.
- El cuerpo se lee por secciones reconocidas por título, con cercas de código como CommonMark, y se conserva byte a byte.
- Esquema estricto (F1-8, F1-9 punto 5): valores permitidos de `tipo`, `estado`, `complejidad` y regla; `motivo` obligatorio con `descartada`; IDs, fechas y obligatorias. Bool y enteros solo con grafía canónica sin comillas. Claves duplicadas, anclas, alias y claves de fusión dan error. Las claves desconocidas de primer nivel se conservan (`zonas`, `hito`); un campo desconocido dentro de `roles` se rechaza, porque se perdería al reescribir.
- Formato 1: una `\` dentro de comillas dobles que no escape `"` da un error claro (H11) en vez de un valor cambiado.
- `serde` + `serde-saphyr` (`default-features = false, features = ["deserialize"]`) solo para leer, detrás de `tarea::yaml` (F1-14); `serde_yaml` 0.9 (port de libyaml) solo en `dev-dependencies` como parser estándar independiente.
- `alta` usa el modelo en vez de su plantilla de texto, con la misma salida byte a byte (test `DORADA`).

**Evidencia.**
- `cargo xtask verificar --perfil completa` en verde, con build reproducible.
- `tests/tarea.rs`: ida y vuelta byte a byte (plantilla y tarea completa); `leer(escribir(m)) == m` con títulos hostiles y contraste campo a campo con `serde_yaml`; corpus de las 94 tareas de este repo; esquema estricto; casos ambiguos del formato 1.
- Corpus externo manual (`TC2_CORPUS`, test ignorado): ToolsaWeb 404 y OpenDockTrading 97 tareas leídas, iguales a `serde_yaml`, cuerpo byte a byte y sin claves desconocidas.

**Revisión.** Dos rondas con `rust-wasm-reviewer`.
- **Ronda 1** (cambios solicitados; el revisor proponía `aprobada-con-correcciones`, pero había un IMPORTANTE abierto):
  - IMP-1, corregido: `plan_aprobado: yes` o `"false"` y `sprint: '7'` o `0x10` se convertían donde un parser estándar ve una cadena.
  - 7 MENORES: seis corregidos (U+FFFE/FFFF, duplicadas lineales, comentario inicial, cercas CommonMark, H11 solo dentro de comillas dobles, campos privados).
  - MEN-5 **aceptado**: al reescribir un fichero con CRLF, el frontmatter sale en LF y el cuerpo queda como estaba; el BOM se pierde. El modelo no recuerda el final de línea, la doc de `escribir` lo dice y 0 de 595 tareas reales llevan CRLF o BOM.
- **Ronda 2** (aprobada con correcciones): 2 MENORES corregidos con `verificar-menores` (un comentario o línea en blanco tras un bool o entero se rechazaba; huecos de mutación en cercas y H11).
- Error de proceso propio: el informe de la ronda 1 se volcó con una tabla distinta de la plantilla, taskctl no la leyó y la ronda 2 fue completa en vez de acotada; el informe se rehízo con la plantilla.

**Divergencias con el plan y con otros ADR.**
- D3: la ida y vuelta con un proyecto Spec Kit y ver en GitHub cómo se pinta `roles` (F1-9, Consecuencias) quedan fuera; pasan a la tarea de comandos de Spec Kit y a F2-6.
- No se modelan `zonas` (F1-17), `hito` (F1-18) ni el `roles` ampliado de F1-15. `zonas` y `hito` se conservan como claves desconocidas; el `roles` ampliado romperá la lectura hasta que su tarea lo modele.
- `estimacion` se conserva como texto crudo hasta que F1-17 se implemente.
- No se hace la conversión de formato 1 a 2 (F1-12 la deja para F9-2).
- Se fija `rama: null` como en `alta` hasta F2-4b.
- Los comentarios del frontmatter no se conservan al reescribir (documentado).
- Se planificó con 55 min.

**CI en develop.** Run 43735844533 sobre 72571a1: codigo (ubuntu, macOS, Windows) y comprobar-repo en success.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-15T15:32:31Z | plan | automatico | persona | 0.16.1 |
| 2026-11-15T15:37:00Z | approve | automatico | persona | 0.16.1 |
| 2026-11-15T15:37:21Z | start | automatico | persona | 0.16.1 |
| 2026-11-15T15:53:45Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T16:14:52Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T16:30:13Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-15T15:37:00Z | diseno | agente:abe862ead99e9122e | 16 | 56796 | 255414 | 12804 | 18 | 325030 | - | - |
| 2026-11-15T15:53:45Z | implementacion | sesion:0c02a589-064f-41b2-898e-aecc1141ae21 | 108 | 86332 | 11332910 | 54056 | 12 | 11473406 | - | - |
| 2026-11-15T16:14:41Z | revision | agente:a0a5ea30f94b82940 | 66 | 172227 | 4589114 | 37893 | 3 | 4799300 | 1 | rust-wasm-reviewer |
| 2026-11-15T16:23:58Z | revision | agente:acaee80f2f2678c83 | 62 | 136878 | 3605340 | 22840 | 2 | 3765120 | 2 | rust-wasm-reviewer |

## Autorrevision

- rust-wasm-reviewer — edd0790
- rust-wasm-reviewer — 53c942f

## Verificacion ligera

- 2026-11-15T16:27:49Z: ronda 2; commit revisado 28ac126257f93838179e03a8e9905745575499a1; commit verificado ca5407411dbd1009fb4ae179b222905a55980ed1; ficheros: crates/tc2/src/tarea/cuerpo.rs, crates/tc2/src/tarea/yaml.rs, crates/tc2/tests/tarea.rs; corregidos: MEN-1, MEN-2; pendientes: ninguno

## Duracion

- Estimada: 55 min
- Real (de la rama abierta al cierre con merge, sin pausas): 53 min
- Cierre (finish): 111 s, de 2026-11-15T16:28:22Z a 2026-11-15T16:30:13Z
