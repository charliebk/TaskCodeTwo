---
id: TASK-113
titulo: "Puerta de valor F10: informe, datos y descarte del H10"
tipo: feature
sprint: 0
etiquetas: [fase10, medicion, docs]
complejidad: simple
modelo_sugerido: sonnet
estado: en-diseno
plan_aprobado: false
rama: feature/task-113-puerta-de-valor-f10-informe-y-datos-del
asignado_a: charlie.bk@gmail.com
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
dependencias: []
---
## Objetivo

Dejar en el repo la evidencia de la puerta de valor de la Fase 10 (experimento A/B de 2026-11-16: 64 corridas, juez a ciegas, veredicto NO-GO por el criterio fijado) y aplicar la decision de Carlos (2026-11-16): la Fase 10 se descarta. Se borran sus tareas y el hito H10 del roadmap, y los documentos que la definian quedan marcados como descartados con enlace a la evidencia, para conservar el porque.

## Criterios de aceptacion
- [ ] docs/mediciones/fase10-puerta-valor/informe.md y corridas.tsv existen y son byte a byte los del experimento (sha256 coincide con el de la carpeta de pruebas)
- [ ] El informe recoge criterio fijado antes de medir, cifras por repo y modo, veredicto NO-GO, uso real del indice y factores que contaminan la medida
- [ ] corridas.tsv lleva un README que explica cada columna y como se midio
- [ ] docs/INDEX.md enlaza el informe
- [ ] Las tareas TASK-096..TASK-112 (F10-1..F10-17) ya no existen en tareas/ y ninguna tarea viva depende de ellas
- [ ] tareas/ROADMAP.md y docs/BOARD.md, regenerados con taskctl, no contienen el hito H10 ni esas tareas
- [ ] RR-20 y RF-77..RF-84 (requisitos y plan), la seccion Fase 10 del plan, el ADR F1-19 (y su fila en docs/adr/README.md) y docs/plan/AMPLIACION-2026-11-16.md quedan marcados como descartados el 2026-11-16 con enlace al informe

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-16T16:43:04Z | plan | automatico | persona | 0.16.1 |
