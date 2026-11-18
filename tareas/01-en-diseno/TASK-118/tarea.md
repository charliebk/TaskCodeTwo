---
id: TASK-118
titulo: "Aplicar las puertas (rulesets) desde tc2 init"
tipo: feature
sprint: 0
etiquetas: [plataforma, puertas]
complejidad: media
modelo_sugerido: sonnet
estado: en-diseno
plan_aprobado: false
rama: feature/task-118-aplicar-las-puertas-rulesets-desde-tc2-i
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-17
actualizado: 2026-11-18
dependencias: []
---
## Objetivo

TASK-028 (F3-1) solo propone los rulesets taskcodetwo-*. Aplicarlos exige los checks verificacion, revision-ia y carril (F5-1, F6) y el integration_id real (F6-1). Esta tarea anade aplicar_puertas al trait Plataforma (GitHub y GitLab) y la opcion de init que las aplica con permiso de la persona.

## Criterios de aceptacion
- [ ] tc2 init --aplicar-puertas crea los rulesets taskcodetwo-* en GitHub y GitLab (test con doble HTTP)
- [ ] Sin los checks ni el integration_id real, la opcion se rechaza con el motivo (test)

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-18T23:15:26Z | plan | automatico | persona | 0.16.1 |
