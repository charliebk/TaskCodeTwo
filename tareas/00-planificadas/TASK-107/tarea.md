---
id: TASK-107
titulo: "F10-12 Grafo de tareas, etiquetas y ficheros en JSON"
tipo: feature
sprint: 10
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-107-f10-12-grafo-de-tareas-etiquetas-y-fiche
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
dependencias: [TASK-106]
---
## Objetivo

tc2 grafo --json: nodos, aristas, pesos y proyecciones (completa, por ficheros, por etiquetas, etiquetas y ficheros), con modo foco, agrupación por carpeta y umbral (RF-83)

Tarea F10-12 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F10-11.

Contexto (Carlos, 2026-11-16, RR-20, hito H10): memoria del proyecto propia que sustituye a engram. Git es la fuente de verdad (tareas, etiquetas, ficheros tocados, memoria importada) y una base SQLite local es un índice reconstruible que nunca se versiona y nunca responde con datos viejos: comprueba en cada lectura hasta qué commit está indexado, porque el merge-pr lo hace la persona en la plataforma y ningún hook local se entera. Etiquetado en tres capas con vocabulario cerrado, `tc2 buscar` como única búsqueda del plugin, conversores de tareas y de engram, grafo y visor 3D opcional con dockview. Detalle completo: docs/plan/AMPLIACION-2026-11-16.md.

## Criterios de aceptacion
- [ ] El JSON de ToolsaWeb en modo foco tiene menos de 300 nodos
- [ ] Mismo resultado con los mismos SHA
- [ ] Cumple lo que pide la tarea F10-12 del plan y los requisitos que cita
