---
id: TASK-137
titulo: "F6-3 seguimiento: revision-ia cita el informe del run de origen cuando la verificacion sale de cache"
tipo: fix
sprint: 6
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-diseno
plan_aprobado: false
rama: fix/task-137-f6-3-seguimiento-revision-ia-cita-el-inf
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-20
actualizado: 2026-11-20
dependencias: []
---
## Objetivo

Cuando verificacion reutiliza un perfil de la cache por arbol (F5-2), su run no sube informe-verificacion-<perfil> y revision-ia.yml (job preparar) lo busca solo en ese run: el check sale con informe=sin-informe y las revisoras van sin la seccion del informe (F5-4). Se cierra el hueco citando el informe del run de origen. Descubierto en el E2E de TASK-049 (run de verificacion 44740343425 reutilizado, run de revision-ia 44740402515 con informe=sin-informe).

## Criterios de aceptacion
- [ ] Un PR cuya verificacion sale de cache produce prompts con la seccion del informe del run de origen, probado en charliebk/taskcodetwo-e2e-github (un PR que ejecuta la verificacion y un commit vacio que la reutiliza)
- [ ] Si el informe del origen ha caducado o no se encuentra, el check lo dice y las revisoras van sin seccion, como hoy
- [ ] docs/ci/revision-ia.md (Limites conocidos) actualizado
- [ ] El lint de xtask/src/workflows.rs sigue en verde

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-20T14:01:02Z | plan | automatico | persona | 0.16.1 |
