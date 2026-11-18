---
id: TASK-131
titulo: "F11-3 Coste por tarea: tarifas por modelo configurables y desglose de entrada, salida y cache"
tipo: feature
sprint: 11
etiquetas: [panel, coste, metricas]
complejidad: media
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-131-f11-3-coste-por-tarea-tarifas-por-modelo
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
dependencias: [TASK-130, TASK-058]
---
## Objetivo

El dashboard de referencia estima el coste con un precio medio por millon porque la 0.16 guarda un solo total de tokens por fase. Con los tokens por tipo que mide F8-1, el coste pasa a ser exacto.

Tarea F11-3 del plan (docs/plan/PLAN-TASKCODE-2.md, Fase 11).

## Criterios de aceptacion
- [ ] Tarifas por modelo en .taskcode/config.yml, con la tarifa publica de la API como valor por defecto
- [ ] Coste por tarea y por fase con desglose de entrada, salida, escritura y lectura de cache cuando existe, y estimacion marcada como tal cuando solo hay un total (test)
- [ ] El coste del proyecto es la suma exacta de las tareas (test)
