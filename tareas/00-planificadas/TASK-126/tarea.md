---
id: TASK-126
titulo: "Seguimiento de F3-13: historia=sin-log en new e import y generador de la paridad"
tipo: fix
sprint: 0
etiquetas: [estimacion]
complejidad: simple
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: fix/task-126-seguimiento-de-f3-13-historia-sin-log-en
asignado_a: null
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-18
actualizado: 2026-11-18
dependencias: []
---
## Objetivo

Cerrar lo que quedo fuera de los ficheros citados en la revision de TASK-083.

## Criterios de aceptacion
- [ ] tc2 new y tc2 import anaden historia=sin-log a su salida cuando el git log de develop falla (test)
- [ ] El script que genera tests/fixtures/estimacion/paridad-poc-f1-17.json vive en el repo junto al fixture y regenerarlo da el mismo fichero
