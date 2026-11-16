---
id: TASK-108
titulo: "F10-13 Medicion y retirada de engram"
tipo: feature
sprint: 10
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-108-f10-13-medicion-y-retirada-de-engram
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
dependencias: [TASK-100, TASK-101, TASK-102, TASK-103, TASK-104, TASK-105, TASK-106, TASK-107]
---
## Objetivo

Preguntas reales en TaskCodeTwo, ToolsaWeb y OpenDockTrading frente a la búsqueda de engram; si gana, engram sale del manifiesto de F1-14

Tarea F10-13 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F10-5, F10-6, F10-7, F10-8, F10-9, F10-10, F10-11, F10-12.

Contexto (Carlos, 2026-11-16, RR-20, hito H10): memoria del proyecto propia que sustituye a engram. Git es la fuente de verdad (tareas, etiquetas, ficheros tocados, memoria importada) y una base SQLite local es un índice reconstruible que nunca se versiona y nunca responde con datos viejos: comprueba en cada lectura hasta qué commit está indexado, porque el merge-pr lo hace la persona en la plataforma y ningún hook local se entera. Etiquetado en tres capas con vocabulario cerrado, `tc2 buscar` como única búsqueda del plugin, conversores de tareas y de engram, grafo y visor 3D opcional con dockview. Detalle completo: docs/plan/AMPLIACION-2026-11-16.md.

## Criterios de aceptacion
- [ ] Informe con aciertos de cada uno
- [ ] Decisión de Carlos registrada
- [ ] Cumple lo que pide la tarea F10-13 del plan y los requisitos que cita
