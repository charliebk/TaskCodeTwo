---
id: TASK-119
titulo: "Seguimiento de F3-1: menores aceptados de tc2 init"
tipo: fix
sprint: 0
etiquetas: [init, tests]
complejidad: simple
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: fix/task-119-seguimiento-de-f3-1-menores-aceptados-de
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

Cerrar los MENORES aceptados en TASK-028.

## Criterios de aceptacion
- [ ] comillas() escapa tambien las comillas tipograficas que PowerShell trata como comillas y las ordenes no dependen de cfg!(windows) sino de la shell indicada (test por shell)
- [ ] Un git merge-base que falla por otro motivo que no sea 'sin ancestro' da no-comprobado, no sin-relacion (test)
- [ ] Las ordenes encadenadas funcionan en Windows PowerShell 5.1 o se dan por separado (test)
- [ ] Rutas de salida con un solo separador por sistema (test)
