---
id: TASK-121
titulo: "Seguimiento de F3-3: menores aceptados de tc2 next"
tipo: fix
sprint: 0
etiquetas: [grafo, tests]
complejidad: simple
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: fix/task-121-seguimiento-de-f3-3-menores-aceptados-de
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

Cerrar los MENORES aceptados en TASK-030.

## Criterios de aceptacion
- [ ] Hitos::del_roadmap tolera un fin de zona suelto y lee mas de una zona manual o avisa (test en grafo.rs)
- [ ] Una tarea cuyo blob no se puede leer sale en ilegibles= (test en estado/tareas.rs)
- [ ] Una carpeta ilegible en minusculas casa con la dependencia en mayusculas (test)
- [ ] Una tarea en curso dentro de un ciclo sale en en-marcha= y ciclos= (test)
- [ ] Hay test que fija la diferencia con el oraculo para una dependencia descartada por etiqueta y terminada (grafo.rs)
