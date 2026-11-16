---
id: TASK-112
titulo: "F10-17 Efectos del visor y panel de actividad en tiempo real"
tipo: feature
sprint: 10
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-112-f10-17-efectos-del-visor-y-panel-de-acti
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
dependencias: [TASK-111]
---
## Objetivo

Pulso de la búsqueda (brillo y partículas en etiquetas y relaciones unos cientos de ms), halo verde o rojo en el fichero según líneas añadidas o borradas, aparición de los ficheros creados y apagado de los borrados, y panel de dockview con la pila de búsquedas (consulta, etiquetas, ficheros y tareas) y ficheros cambiados, creados o borrados en tiempo real, sincronizado con el grafo (RF-84)

Tarea F10-17 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F10-16.

Contexto (Carlos, 2026-11-16, RR-20, hito H10): memoria del proyecto propia que sustituye a engram. Git es la fuente de verdad (tareas, etiquetas, ficheros tocados, memoria importada) y una base SQLite local es un índice reconstruible que nunca se versiona y nunca responde con datos viejos: comprueba en cada lectura hasta qué commit está indexado, porque el merge-pr lo hace la persona en la plataforma y ningún hook local se entera. Etiquetado en tres capas con vocabulario cerrado, `tc2 buscar` como única búsqueda del plugin, conversores de tareas y de engram, grafo y visor 3D opcional con dockview. Detalle completo: docs/plan/AMPLIACION-2026-11-16.md.

## Criterios de aceptacion
- [ ] Una búsqueda ilumina exactamente los nodos y aristas devueltos por tc2 buscar
- [ ] Añadir y borrar líneas destella verde y rojo en el fichero correcto
- [ ] Un fichero creado aparece en la red unido a la tarea en curso
- [ ] Cada evento aparece en la pila y en el grafo a la vez
- [ ] Pulsar una entrada de la pila resalta y centra sus nodos en el grafo
- [ ] Cumple lo que pide la tarea F10-17 del plan y los requisitos que cita
