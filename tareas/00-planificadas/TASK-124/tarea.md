---
id: TASK-124
titulo: "Seguimiento de F3-5: menores aceptados de tc2 board y status"
tipo: fix
sprint: 0
etiquetas: [tablero, tests]
complejidad: simple
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: fix/task-124-seguimiento-de-f3-5-menores-aceptados-de
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

Cerrar los MENORES aceptados en TASK-032.

## Criterios de aceptacion
- [ ] La linea Ilegibles: pasa por sanear y sanear filtra U+2060..2064, U+00AD y U+061C (test en tablero.rs)
- [ ] Con varios PR en la misma rama se prefiere uno que no es borrador; respuesta invalida de prs_abiertos con texto propio (tests)
- [ ] El test de C3a distingue el orden de listas (estado con al menos dos listas)
- [ ] El texto de board y status sin --json sale en el idioma de la persona sin cambiar el --json (test)
- [ ] La adenda de F1-4 dice que solo board escribe avisos en stderr
