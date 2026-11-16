---
id: TASK-100
titulo: "F10-5 Conversor de engram por proyecto"
tipo: feature
sprint: 10
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-100-f10-5-conversor-de-engram-por-proyecto
asignado_a: null
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-16
actualizado: 2026-11-16
dependencias: [TASK-096, TASK-098]
---
## Objetivo

Copia de solo lectura, alias de proyecto, enlace por TASK-NNN, relaciones como aristas, idempotente, --simular, filtro de secretos y materializado según F1-19 (RF-80)

Tarea F10-5 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F10-1, F10-3.

Contexto (Carlos, 2026-11-16, RR-20, hito H10): memoria del proyecto propia que sustituye a engram. Git es la fuente de verdad (tareas, etiquetas, ficheros tocados, memoria importada) y una base SQLite local es un índice reconstruible que nunca se versiona y nunca responde con datos viejos: comprueba en cada lectura hasta qué commit está indexado, porque el merge-pr lo hace la persona en la plataforma y ningún hook local se entera. Etiquetado en tres capas con vocabulario cerrado, `tc2 buscar` como única búsqueda del plugin, conversores de tareas y de engram, grafo y visor 3D opcional con dockview. Detalle completo: docs/plan/AMPLIACION-2026-11-16.md.

## Criterios de aceptacion
- [ ] Con la base real: 146 observaciones, 116 enlazadas a su tarea, alias resuelto
- [ ] Cero duplicados al repetir
- [ ] Cumple lo que pide la tarea F10-5 del plan y los requisitos que cita
