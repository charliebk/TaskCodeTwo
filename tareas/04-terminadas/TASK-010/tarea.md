---
id: TASK-010
titulo: "F1-9 ADR formato de tarea, plan y especificacion"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-010-f1-9-adr-formato-de-tarea-plan-y-especif
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1050188
tokens_implementacion: 9014804
tokens_revision: 2277712
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-002]
---
## Objetivo

ADR: formato de tarea, plan y especificación (compatible con la 0.16 y con Spec Kit)

Tarea F1-9 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-1.

## Criterios de aceptacion
- [x] Plantillas
- [x] qué campos se conservan
- [x] Cumple lo que pide la tarea F1-9 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T08:37:44Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T08:41:37Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T08:41:38Z | start | automatico | persona | 0.16.1 |
| 2026-11-14T08:56:29Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T09:09:01Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T08:41:36Z | diseno | agente:a03ad0577cf0c888b | 38 | 81987 | 956563 | 11600 | 27 | 1050188 | - | - |
| 2026-11-14T08:56:29Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 8 | 9065 | 1483803 | 2217 | 0 | 1495093 | - | - |
| 2026-11-14T08:56:29Z | implementacion | agente:a8334869eb5e76b5e | 102 | 208368 | 7243079 | 68162 | 8 | 7519711 | - | - |
| 2026-11-14T09:05:16Z | revision | agente:a53459c69829bba85 | 36 | 149367 | 2110211 | 18098 | 5 | 2277712 | 1 | - |

## Autorrevision

- code-quality-reviewer — 6042594

## Verificacion ligera

- 2026-11-14T09:08:37Z: ronda 1; commit revisado 43678e65ce225547cfdef2586a1fadce4baf0438; commit verificado 952239170402a7c566e071f354a3db3a503cf883; ficheros: docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md; corregidos: MEN-2, MEN-3; pendientes: MEN-1

## Resultado

- **Implementado**: ADR [F1-9](../../../docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md) (aceptada) y su fila en `docs/adr/README.md`. Modelo canonico unico: `tarea.md` en la carpeta fija de F1-1, con Spec Kit como correspondencia de entrada y salida (Objetivo + Criterios + `## Escenarios` ↔ `spec.md`; `plan-final.md` ↔ `plan.md`; troceo ↔ `tasks.md`). `plan-final.md` conserva nombre y ruta por el `Plan-Blob` de F1-5/F1-6. Tabla de los 22 campos de la 0.16 con su cifra medida: 13 se conservan, 7 cambian de sentido, 2 se retiran (`revision_codex`, `ultimo_commit_revisado`), 4 se anaden (`formato`, `carril`, `motivo`, estimacion). Secciones reconocidas por titulo; contrato YAML estricto (cadenas entre comillas, claves duplicadas rechazadas, desconocidas conservadas con aviso). Plantillas de `tarea.md` y `plan-final.md` en bloques de codigo.
- **Cifras** (ToolsaWeb / OpenDockTrading / TaskCodeTwo, commits fijos): 404 / 96 / 93 tareas, 0 fallos de PyYAML, 0 claves fuera de las 22; `## Resultado` detras de `## Transiciones` en 28 / 14 / 8.
- **Decisiones de la persona** (consigna: opcion recomendada): D1a especificacion dentro de `tarea.md`, D2a plantillas en el ADR, D3a `plan_aprobado` como cache informativa, D4a claves desconocidas conservadas con aviso, D5a se retira `revision_codex`.
- **Divergencias** (en el ADR): `agente_revisor` pasa a `roles.revision.agente` y `ultimo_commit_revisado` se retira; `estado` en una sola categoria; `## Duracion` opcional y `## Descartada` sustituida por `estado: descartada` + `motivo`; la PoC mide commits fijos para que la revision repita las cifras.
- **No verificado**: como pinta GitHub el frontmatter; ida y vuelta real con el CLI de Spec Kit (F2-5); que `modelo_sugerido` corresponda al rol de implementacion (F1-15/F1-12).
- **Revision**: ronda 1 aprobada con correcciones: MEN-1 (orden determinista), MEN-2 (tabla de secciones y recuento vigilados por el guion), MEN-3 (redaccion de la plantilla del plan) corregidos en `9522391` (`verificar-menores` dejo MEN-1 en `menores-pendientes.md` porque no reconoce los ficheros de la carpeta de la tarea; anotado alli).
- Implementacion por un agente implementador; revision por un agente independiente. `taskctl doctor` con 0 errores. Temporales borrados.

## Duracion

- Estimada: 45 min
- Real (de la rama abierta al cierre con merge, sin pausas): 27 min
- Cierre (finish): 7 s, de 2026-11-14T09:08:54Z a 2026-11-14T09:09:01Z
