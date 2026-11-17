---
id: TASK-123
titulo: "Seguimiento de F3-4: menores aceptados de tc2 roadmap"
tipo: fix
sprint: 0
etiquetas: [roadmap, tests]
complejidad: simple
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: fix/task-123-seguimiento-de-f3-4-menores-aceptados-de
asignado_a: null
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-17
actualizado: 2026-11-17
dependencias: []
---
## Objetivo

Cerrar los MENORES aceptados en TASK-031.

## Criterios de aceptacion
- [ ] restaurar_indice tiene un test que falla si se anula su llamada (crates/tc2/tests/roadmap.rs)
- [ ] Una zona manual con CRLF puro o con CR sueltos se conserva byte a byte (test)
- [ ] Dos clones con distinto origin/develop dan el mismo ROADMAP para un asignado_a heredado o --comparar lo explica (test)
- [ ] Un ROADMAP de la 0.16 con texto dentro de la marca de fin se migra o se avisa con el arreglo (test)
