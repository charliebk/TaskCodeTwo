---
id: TASK-087
titulo: "F4-8 Fallos, presupuesto, reanudacion e informe del lote"
tipo: feature
sprint: 4
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-087-f4-8-fallos-presupuesto-reanudacion-e-in
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
dependencias: [TASK-058, TASK-086]
---
## Objetivo

Fallos, presupuesto, `--reanudar` e informe del lote

Tarea F4-8 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F4-7, F8-1.

Contexto (Carlos, 2026-11-14, RR-18, imprescindible): hoy se hace por instrucción al agente («arrancar consecutivamente TASK-xxx a TASK-yyy, sin trocear y aceptando las recomendaciones en todas las preguntas») en ToolsaWeb, OpenDockTrading y TaskCode, para listas de documentación o programación que no necesitan revisión humana hasta el final, o para dejar trabajo por la noche. Diseño propuesto (P13 del plan): rama `lote/<nombre>` desde develop; cada tarea pasa sus puertas automáticas y se integra en la rama del lote (la siguiente ve el código de la anterior); al final un único PR del lote contra develop que aprueba la persona. Solo carriles permitidos; una tarea en rojo o con un CRÍTICO/IMPORTANTE sin resolver se aparta con sus dependientes y el lote sigue; nunca se integra nada en rojo. Presupuesto de tiempo y tokens; `--reanudar` sin repetir; informe de la mañana (terminadas, apartadas y por qué, hallazgos sin corregir, tokens, tiempo). Lo ejecuta el CLI de forma determinista, no el agente. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 bis.

## Criterios de aceptacion
- [ ] Un lote cortado se reanuda sin repetir
- [ ] una tarea en rojo se aparta con sus dependientes y el resto sigue
- [ ] informe completo
- [ ] Cumple lo que pide la tarea F4-8 del plan y los requisitos que cita
