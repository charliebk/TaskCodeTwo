---
id: TASK-049
titulo: "F6-3 Revisor en CI con las skills de dominio de la 0.16"
tipo: feature
sprint: 6
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-049-f6-3-revisor-en-ci-con-las-skills-de-dom
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
dependencias: [TASK-044, TASK-048]
---
## Objetivo

Revisor en CI con las skills de dominio de la 0.16 (angular-vue, rust-wasm, C#, Java, genérica), enrutado por lo que toca el diff (RR-02)

Tarea F6-3 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F6-2, F5-4.

Contexto (Carlos, 2026-11-14, RR-17, imprescindible): para cada tarea y cada rol (diseño, implementación, revisión) hay que elegir las skills y los agentes adecuados y el modelo según complejidad y carril, solo entre lo que está instalado en la máquina y el proyecto, anotar la elección en la tarea y medir si acierta (hallazgos por revisora y coste por modelo). En la 0.16 existe con `catalogo-skills.yml`, `skills_recomendados` y `modelo_sugerido`, pero se usó en 21 / 0 / 2 tareas de ToolsaWeb / OpenDockTrading / TaskCode porque el catálogo apenas se configuró: hoy la elección la hace el agente sin criterio registrado. Nada debe decidirlo el modelo «a ojo»: reglas deterministas. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §1–4.

## Criterios de aceptacion
- [ ] Un diff Rust + Vue lanza las dos de dominio
- [ ] Cumple lo que pide la tarea F6-3 del plan y los requisitos que cita
