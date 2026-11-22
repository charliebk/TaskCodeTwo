---
id: TASK-055
titulo: "F7-2 Resultado obligatorio en cada tarea"
tipo: feature
sprint: 7
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-055-f7-2-resultado-obligatorio-en-cada-tarea
asignado_a: null
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-054]
---
## Objetivo

`## Resultado` obligatorio (qué se hizo, divergencias, hallazgos no corregidos)

Tarea F7-2 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F7-1.

Ampliacion (Carlos, 2026-11-22; equivalente a TASK-137 del repo TaskCode): regla 4D. El cierre y la revision aseguran que no se entrega codigo 4D: 1) Duplicacion: dos funciones o tipos que hacen lo mismo con pequenas diferencias; se reutiliza y generaliza (POO, traits o composicion como eje). 2) Desviacion: salir del objetivo de la tarea; si hay que cambiar la expectativa se bifurca una tarea o se propone el cambio sin salir del objetivo. 3) Diluvio: avalanchas de lineas y tipos donde bastan pocas lineas eficientes. 4) Disfraz: premisas de caso resuelto que no hacen nada o no se comprueban (tests que no prueban, stubs que aparentan).

## Criterios de aceptacion
- [ ] `finish` lo exige
- [ ] test
- [ ] Cumple lo que pide la tarea F7-2 del plan y los requisitos que cita
- [ ] Las instrucciones de las revisoras (local y `tc2 hallazgos instrucciones` en CI) describen las 4D con un ejemplo de cada una y clasifican como IMPORTANTE la duplicacion y el disfraz reales
- [ ] El informe de revision lleva una seccion `## Regla 4D` con una linea por D (cumple o ID del hallazgo) y `finish` la exige, con su test
- [ ] La peticion de implementacion exige antes de pedir revision: buscar el helper existente antes de crear uno, citar el criterio que motiva cada cambio y comprobar por mutacion cada test nuevo, anotado en `## Autorrevision`
