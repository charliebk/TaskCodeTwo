---
id: TASK-105
titulo: "F10-10 Etiquetado lexico e IA acotada en finish"
tipo: feature
sprint: 10
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-105-f10-10-etiquetado-lexico-e-ia-acotada-en
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
dependencias: [TASK-104, TASK-054]
---
## Objetivo

Etiquetado léxico e IA acotada al vocabulario en finish, con cola de propuestas (RF-82)

Tarea F10-10 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F10-9, F7-1.

Contexto (Carlos, 2026-11-16, RR-20, hito H10): memoria del proyecto propia que sustituye a engram. Git es la fuente de verdad (tareas, etiquetas, ficheros tocados, memoria importada) y una base SQLite local es un índice reconstruible que nunca se versiona y nunca responde con datos viejos: comprueba en cada lectura hasta qué commit está indexado, porque el merge-pr lo hace la persona en la plataforma y ningún hook local se entera. Etiquetado en tres capas con vocabulario cerrado, `tc2 buscar` como única búsqueda del plugin, conversores de tareas y de engram, grafo y visor 3D opcional con dockview. Detalle completo: docs/plan/AMPLIACION-2026-11-16.md.

## Criterios de aceptacion
- [ ] La IA nunca escribe una etiqueta fuera del vocabulario
- [ ] Una propuesta entra solo con persona o N apariciones
- [ ] Cumple lo que pide la tarea F10-10 del plan y los requisitos que cita
