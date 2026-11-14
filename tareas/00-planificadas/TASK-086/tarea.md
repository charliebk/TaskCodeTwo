---
id: TASK-086
titulo: "F4-7 taskctl lote: secuencia de tareas sin preguntas"
tipo: feature
sprint: 4
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-086-f4-7-taskctl-lote-secuencia-de-tareas-si
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
dependencias: [TASK-039, TASK-074, TASK-085]
---
## Objetivo

`taskctl lote TASK-a..TASK-b` (o lista): secuencia sin preguntas, sin trocear, aceptando recomendaciones, integración en la rama del lote y PR final contra `develop` (RF-27, RR-18)

Tarea F4-7 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F4-4, F4-6, F1-16.

Contexto (Carlos, 2026-11-14, RR-18, imprescindible): hoy se hace por instrucción al agente («arrancar consecutivamente TASK-xxx a TASK-yyy, sin trocear y aceptando las recomendaciones en todas las preguntas») en ToolsaWeb, OpenDockTrading y TaskCode, para listas de documentación o programación que no necesitan revisión humana hasta el final, o para dejar trabajo por la noche. Diseño propuesto (P13 del plan): rama `lote/<nombre>` desde develop; cada tarea pasa sus puertas automáticas y se integra en la rama del lote (la siguiente ve el código de la anterior); al final un único PR del lote contra develop que aprueba la persona. Solo carriles permitidos; una tarea en rojo o con un CRÍTICO/IMPORTANTE sin resolver se aparta con sus dependientes y el lote sigue; nunca se integra nada en rojo. Presupuesto de tiempo y tokens; `--reanudar` sin repetir; informe de la mañana (terminadas, apartadas y por qué, hallazgos sin corregir, tokens, tiempo). Lo ejecuta el CLI de forma determinista, no el agente. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 bis.

## Criterios de aceptacion
- [ ] Un lote de 5 tareas termina sin intervención y deja un único PR para la persona
- [ ] Cumple lo que pide la tarea F4-7 del plan y los requisitos que cita
