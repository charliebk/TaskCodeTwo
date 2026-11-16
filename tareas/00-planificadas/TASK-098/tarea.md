---
id: TASK-098
titulo: "F10-3 Conversor de las tareas existentes al indice"
tipo: feature
sprint: 10
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-098-f10-3-conversor-de-las-tareas-existentes
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
dependencias: [TASK-096]
---
## Objetivo

Todas las tarea.md y su historial por tipo(TASK-NNN), con renombrados y ficheros comodín (RF-78)

Tarea F10-3 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F10-1.

Contexto (Carlos, 2026-11-16, RR-20, hito H10): memoria del proyecto propia que sustituye a engram. Git es la fuente de verdad (tareas, etiquetas, ficheros tocados, memoria importada) y una base SQLite local es un índice reconstruible que nunca se versiona y nunca responde con datos viejos: comprueba en cada lectura hasta qué commit está indexado, porque el merge-pr lo hace la persona en la plataforma y ningún hook local se entera. Etiquetado en tres capas con vocabulario cerrado, `tc2 buscar` como única búsqueda del plugin, conversores de tareas y de engram, grafo y visor 3D opcional con dockview. Detalle completo: docs/plan/AMPLIACION-2026-11-16.md.

## Criterios de aceptacion
- [ ] Las tareas de TaskCodeTwo y una copia de ToolsaWeb indexadas
- [ ] Repetirlo no cambia nada
- [ ] Cumple lo que pide la tarea F10-3 del plan y los requisitos que cita
