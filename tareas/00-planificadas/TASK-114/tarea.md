---
id: TASK-114
titulo: "F2-12b Lanzador del plugin que fija la version del CLI"
tipo: feature
sprint: 0
etiquetas: [distribucion, plugin]
complejidad: media
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-114-f2-12b-lanzador-del-plugin-que-fija-la-v
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

Lanzador del plugin (tc2, tc2.cmd, tc2.ps1) que lee cli.lock, comprueba la version con tc2 version --json y tc2 compatible, y descarga o avisa con el comando que lo arregla. Sale del plan de TASK-027 (F2-12), que entrega version --json, compatible y cargo xtask cli-lock.

## Criterios de aceptacion
- [ ] El lanzador arranca el tc2 de la version de cli.lock en Windows, macOS y Linux (test por sistema)
- [ ] Con un tc2 de protocolo incompatible el lanzador avisa con el comando que lo arregla y sale distinto de 0 (test)
