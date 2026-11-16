---
id: TASK-113
titulo: "Puerta de valor F10: informe y datos del A/B indice frente a grep"
tipo: feature
sprint: 0
etiquetas: [fase10, medicion, docs]
complejidad: simple
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-113-puerta-de-valor-f10-informe-y-datos-del
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
dependencias: []
---
## Objetivo

Dejar en el repo la evidencia de la puerta de valor de la Fase 10 (experimento A/B de 2026-11-16: 64 corridas, juez a ciegas, veredicto NO-GO por el criterio fijado) para que la decision de Carlos sobre la Fase 10 se apoye en datos versionados y no en una carpeta temporal que ya se perdio una vez.

## Criterios de aceptacion
- [ ] docs/mediciones/fase10-puerta-valor/informe.md y corridas.tsv existen y son byte a byte los del experimento (sha256 coincide con el de la carpeta de pruebas)
- [ ] El informe recoge criterio fijado antes de medir, cifras por repo y modo, veredicto NO-GO, uso real del indice y factores que contaminan la medida
- [ ] corridas.tsv lleva un README que explica cada columna y como se midio
- [ ] docs/INDEX.md enlaza el informe
