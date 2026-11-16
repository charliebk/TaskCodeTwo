---
id: TASK-111
titulo: "F10-16 App Vue del visor con dockview"
tipo: feature
sprint: 10
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-111-f10-16-app-vue-del-visor-con-dockview
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
dependencias: [TASK-110]
---
## Objetivo

Paneles de red completa 3D, por ficheros, por etiquetas, etiquetas y ficheros, búsqueda y detalle de tarea, con dockview y PrimeVue y su disposición guardada (RF-84)

Tarea F10-16 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F10-15.

Contexto (Carlos, 2026-11-16, RR-20, hito H10): memoria del proyecto propia que sustituye a engram. Git es la fuente de verdad (tareas, etiquetas, ficheros tocados, memoria importada) y una base SQLite local es un índice reconstruible que nunca se versiona y nunca responde con datos viejos: comprueba en cada lectura hasta qué commit está indexado, porque el merge-pr lo hace la persona en la plataforma y ningún hook local se entera. Etiquetado en tres capas con vocabulario cerrado, `tc2 buscar` como única búsqueda del plugin, conversores de tareas y de engram, grafo y visor 3D opcional con dockview. Detalle completo: docs/plan/AMPLIACION-2026-11-16.md.

## Criterios de aceptacion
- [ ] Las cuatro vistas abiertas a la vez con el grafo de ToolsaWeb
- [ ] La disposición se conserva al volver a abrir
- [ ] Cumple lo que pide la tarea F10-16 del plan y los requisitos que cita
