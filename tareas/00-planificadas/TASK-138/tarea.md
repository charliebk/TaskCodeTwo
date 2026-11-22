---
id: TASK-138
titulo: "Auditoria 4D de todo el codigo de TaskCodeTwo"
tipo: feature
sprint: 0
etiquetas: []
complejidad: media
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-138-auditoria-4d-de-todo-el-codigo-de-taskco
asignado_a: null
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-22
actualizado: 2026-11-22
dependencias: []
---
## Objetivo

Auditar todo el codigo existente de TaskCodeTwo (crates/tc2/src, crates/tc2/tests, xtask, crates/distribucion y .github/workflows) contra la regla 4D de Carlos, recomendada por TaskCode 0.17 al actualizar: 1) Duplicacion: dos funciones o tipos que hacen lo mismo con pequenas diferencias. 2) Desviacion: codigo que se sale del objetivo de su tarea o de su ADR (o un criterio dado por cumplido sin estarlo). 3) Diluvio: lineas y tipos de mas donde bastan pocos. 4) Disfraz: codigo o tests que aparentan resolver algo y no lo comprueban. La auditoria solo diagnostica; las correcciones van en tareas de seguimiento.

## Criterios de aceptacion
- [ ] docs/auditoria-4d.md lista cada caso con su regla (D1 a D4), fichero:linea, la evidencia que lo prueba (los dos fragmentos duplicados, el mutante que sobrevive, el criterio desviado) y su severidad CRITICO, IMPORTANTE o MENOR
- [ ] Cubre todos los modulos de crates/tc2/src, los tests, xtask, crates/distribucion y los workflows; el documento lleva una tabla de cobertura por modulo con el numero de casos
- [ ] Cada caso IMPORTANTE o CRITICO tiene su tarea de seguimiento en tareas/00-planificadas, agrupadas por modulo
- [ ] Los casos de Disfraz en tests se prueban con un mutante que sobrevive, ejecutado de verdad
