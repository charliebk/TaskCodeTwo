---
id: TASK-109
titulo: "F10-14 PoC del motor 3D del visor"
tipo: feature
sprint: 10
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-109-f10-14-poc-del-motor-3d-del-visor
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
dependencias: [TASK-107]
---
## Objetivo

3d-force-graph frente a Babylon.js con d3-force-3d, con el grafo real de ToolsaWeb (RF-84)

Tarea F10-14 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F10-12.

Contexto (Carlos, 2026-11-16, RR-20, hito H10): memoria del proyecto propia que sustituye a engram. Git es la fuente de verdad (tareas, etiquetas, ficheros tocados, memoria importada) y una base SQLite local es un índice reconstruible que nunca se versiona y nunca responde con datos viejos: comprueba en cada lectura hasta qué commit está indexado, porque el merge-pr lo hace la persona en la plataforma y ningún hook local se entera. Etiquetado en tres capas con vocabulario cerrado, `tc2 buscar` como única búsqueda del plugin, conversores de tareas y de engram, grafo y visor 3D opcional con dockview. Detalle completo: docs/plan/AMPLIACION-2026-11-16.md.

## Criterios de aceptacion
- [ ] Cifras de FPS, memoria y tamaño del bundle con el grafo completo y en modo foco
- [ ] Halo y pulso funcionando en los dos; decisión registrada
- [ ] Cumple lo que pide la tarea F10-14 del plan y los requisitos que cita
