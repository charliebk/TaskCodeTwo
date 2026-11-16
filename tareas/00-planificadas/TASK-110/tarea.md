---
id: TASK-110
titulo: "F10-15 tc2 visor: servidor local y canal de eventos"
tipo: feature
sprint: 10
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-110-f10-15-tc2-visor-servidor-local-y-canal
asignado_a: null
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-16
actualizado: 2026-11-16
dependencias: [TASK-097, TASK-101, TASK-109]
---
## Objetivo

Servidor en 127.0.0.1 con token, app embebida en el ejecutable y canal de eventos (busqueda, fichero_cambiado, fichero_creado, fichero_borrado, indice_actualizado) desde el índice y un vigilante de ficheros (RF-84)

Tarea F10-15 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F10-2, F10-6, F10-14.

Contexto (Carlos, 2026-11-16, RR-20, hito H10): memoria del proyecto propia que sustituye a engram. Git es la fuente de verdad (tareas, etiquetas, ficheros tocados, memoria importada) y una base SQLite local es un índice reconstruible que nunca se versiona y nunca responde con datos viejos: comprueba en cada lectura hasta qué commit está indexado, porque el merge-pr lo hace la persona en la plataforma y ningún hook local se entera. Etiquetado en tres capas con vocabulario cerrado, `tc2 buscar` como única búsqueda del plugin, conversores de tareas y de engram, grafo y visor 3D opcional con dockview. Detalle completo: docs/plan/AMPLIACION-2026-11-16.md.

## Criterios de aceptacion
- [ ] Se abre en el navegador sin Node instalado
- [ ] Un cambio en un fichero llega al visor en menos de 1 s
- [ ] Cerrar el visor no afecta a tc2
- [ ] Cumple lo que pide la tarea F10-15 del plan y los requisitos que cita
