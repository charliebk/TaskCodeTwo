---
id: TASK-091
titulo: "F9-4b Prueba nocturna de un lote de 10 tareas"
tipo: feature
sprint: 9
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-091-f9-4b-prueba-nocturna-de-un-lote-de-10-t
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
dependencias: [TASK-066, TASK-087]
---
## Objetivo

Prueba nocturna: lote de 10 tareas reales desatendido en un repo de pruebas (RR-18)

Tarea F9-4b del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F4-8, F9-4.

Contexto (Carlos, 2026-11-14, RR-18, imprescindible): hoy se hace por instrucción al agente («arrancar consecutivamente TASK-xxx a TASK-yyy, sin trocear y aceptando las recomendaciones en todas las preguntas») en ToolsaWeb, OpenDockTrading y TaskCode, para listas de documentación o programación que no necesitan revisión humana hasta el final, o para dejar trabajo por la noche. Diseño propuesto (P13 del plan): rama `lote/<nombre>` desde develop; cada tarea pasa sus puertas automáticas y se integra en la rama del lote (la siguiente ve el código de la anterior); al final un único PR del lote contra develop que aprueba la persona. Solo carriles permitidos; una tarea en rojo o con un CRÍTICO/IMPORTANTE sin resolver se aparta con sus dependientes y el lote sigue; nunca se integra nada en rojo. Presupuesto de tiempo y tokens; `--reanudar` sin repetir; informe de la mañana (terminadas, apartadas y por qué, hallazgos sin corregir, tokens, tiempo). Lo ejecuta el CLI de forma determinista, no el agente. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 bis.

## Criterios de aceptacion
- [ ] Por la mañana: PR del lote, informe y ninguna integración en rojo
- [ ] Cumple lo que pide la tarea F9-4b del plan y los requisitos que cita
