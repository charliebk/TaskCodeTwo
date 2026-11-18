---
id: TASK-129
titulo: "F11-1 ADR panel web de tc2: servidor local de solo lectura, Vue y origen de los datos"
tipo: feature
sprint: 11
etiquetas: [panel, adr]
complejidad: media
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-129-f11-1-adr-panel-web-de-tc2-servidor-loca
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

Decidir como sirve tc2 un panel web para cada programador (tc2 panel: el ejecutable sirve una pagina Vue en un puerto local) y de donde salen sus datos: tareas (carpetas de la 0.16 o rama de estado de la 2.0), historial de git y metricas de F8-1. Nace del dashboard de progreso y coste hecho para ToolsaWeb el 2026-11-18 (progreso, tareas terminadas por dia, coste diario y acumulado, avance y coste por hito y coste por tarea), que se toma como referencia funcional. Toma lo aprovechable del servidor local del visor descartado (F10-15) sin la parte 3D.

Tarea F11-1 del plan (docs/plan/PLAN-TASKCODE-2.md, Fase 11).

## Criterios de aceptacion
- [ ] Un ADR en docs/adr/ con alternativas (servidor embebido en tc2, pagina estatica generada, vista en la plataforma), decision y motivo
- [ ] Fija el contrato JSON v1 que el servidor entrega a la pagina
- [ ] Seguridad: escucha solo en 127.0.0.1 con token, no escribe nada en el repo ni en la rama de estado y no expone secretos
