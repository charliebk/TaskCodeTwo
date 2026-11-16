---
id: TASK-095
titulo: "F1-19 ADR memoria del proyecto: indice de contexto, etiquetas y busqueda"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-diseno
plan_aprobado: true
rama: feature/task-095-f1-19-adr-memoria-del-proyecto-indice-de
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1093445
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-16
actualizado: 2026-11-16
dependencias: [TASK-002, TASK-010, TASK-012, TASK-014]
---
## Objetivo

ADR: memoria del proyecto (RR-20): git como verdad e índice SQLite reconstruible, ubicación del índice, regla de frescura por SHA, esquema, vocabulario y tres capas de etiquetado, conversor de engram (decisión A/B) y retirada de engram del manifiesto

Tarea F1-19 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-1, F1-9, F1-11, F1-14.

Contexto (Carlos, 2026-11-16, RR-20, hito H10): memoria del proyecto propia que sustituye a engram. Git es la fuente de verdad (tareas, etiquetas, ficheros tocados, memoria importada) y una base SQLite local es un índice reconstruible que nunca se versiona y nunca responde con datos viejos: comprueba en cada lectura hasta qué commit está indexado, porque el merge-pr lo hace la persona en la plataforma y ningún hook local se entera. Etiquetado en tres capas con vocabulario cerrado, `tc2 buscar` como única búsqueda del plugin, conversores de tareas y de engram, grafo y visor 3D opcional con dockview. Detalle completo: docs/plan/AMPLIACION-2026-11-16.md.

## Criterios de aceptacion
- [ ] Índice de las tareas terminadas de TaskCodeTwo reconstruido dos veces con el mismo hash
- [ ] Un merge hecho en la plataforma se refleja en la siguiente búsqueda sin hooks
- [ ] Cumple lo que pide la tarea F1-19 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-16T12:55:18Z | plan | automatico | persona | 0.16.1 |
| 2026-11-16T13:06:31Z | approve | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-16T13:03:25Z | diseno | agente:ab69920f34bade972,ac5f927590e26cbe9,a4a936ae6ac2f3123 | 56 | 154965 | 906531 | 31893 | 42 | 1093445 | - | - |
