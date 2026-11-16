---
id: TASK-096
titulo: "F10-1 Indice SQLite reconstruible"
tipo: feature
sprint: 10
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-096-f10-1-indice-sqlite-reconstruible
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
dependencias: [TASK-095, TASK-020]
---
## Objetivo

Índice SQLite (rusqlite con FTS5 empaquetado), esquema y migraciones, tc2 indice reconstruir y estado (RF-77)

Tarea F10-1 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-19, F2-5.

Contexto (Carlos, 2026-11-16, RR-20, hito H10): memoria del proyecto propia que sustituye a engram. Git es la fuente de verdad (tareas, etiquetas, ficheros tocados, memoria importada) y una base SQLite local es un índice reconstruible que nunca se versiona y nunca responde con datos viejos: comprueba en cada lectura hasta qué commit está indexado, porque el merge-pr lo hace la persona en la plataforma y ningún hook local se entera. Etiquetado en tres capas con vocabulario cerrado, `tc2 buscar` como única búsqueda del plugin, conversores de tareas y de engram, grafo y visor 3D opcional con dockview. Detalle completo: docs/plan/AMPLIACION-2026-11-16.md.

## Criterios de aceptacion
- [ ] Dos máquinas con los mismos SHA dan el mismo contenido
- [ ] El índice nunca aparece en git status
- [ ] Cumple lo que pide la tarea F10-1 del plan y los requisitos que cita
