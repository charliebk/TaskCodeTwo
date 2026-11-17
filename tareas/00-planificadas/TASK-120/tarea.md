---
id: TASK-120
titulo: "Seguimiento de F3-2: menores aceptados de tc2 import"
tipo: fix
sprint: 0
etiquetas: [importar, tests]
complejidad: simple
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: fix/task-120-seguimiento-de-f3-2-menores-aceptados-de
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

Cerrar los MENORES aceptados en TASK-029.

## Criterios de aceptacion
- [ ] Una cabecera con --- y espacio final o con linea en blanco previa se reconoce o se avisa (test en importar/fichero.rs)
- [ ] Sin cabecera, el lote no depende solo del slug del nombre del fichero o se avisa de ello; renombrar no duplica (test)
- [ ] Un tarea.md ilegible no hace que un import repetido la vuelva a crear (test)
- [ ] Una dependencia a una tarea descartada avisa; el error de TASK-NNN externa lleva referencia= y linea=; sprint 0 sin cabecera ni --sprint avisa (tests)
