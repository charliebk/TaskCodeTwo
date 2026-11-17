---
id: TASK-080
titulo: "F3-9 Inventario de skills y agentes instalados"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-diseno
plan_aprobado: false
rama: feature/task-080-f3-9-inventario-de-skills-y-agentes-inst
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-17
dependencias: [TASK-016, TASK-073]
---
## Objetivo

Inventario de skills y agentes instalados (máquina y proyecto) y `doctor` que avisa de un catálogo que nombra algo no instalado (RF-14)

Tarea F3-9 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-1, F1-15.

Contexto (Carlos, 2026-11-14, RR-17, imprescindible): para cada tarea y cada rol (diseño, implementación, revisión) hay que elegir las skills y los agentes adecuados y el modelo según complejidad y carril, solo entre lo que está instalado en la máquina y el proyecto, anotar la elección en la tarea y medir si acierta (hallazgos por revisora y coste por modelo). En la 0.16 existe con `catalogo-skills.yml`, `skills_recomendados` y `modelo_sugerido`, pero se usó en 21 / 0 / 2 tareas de ToolsaWeb / OpenDockTrading / TaskCode porque el catálogo apenas se configuró: hoy la elección la hace el agente sin criterio registrado. Nada debe decidirlo el modelo «a ojo»: reglas deterministas. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §1–4.

## Criterios de aceptacion
- [ ] C1: con `CLAUDE_CONFIG_DIR` y proyecto falsos en temporales (skills y agents de usuario, un plugin habilitado con `installPath`, otro deshabilitado en `settings.json`, y `.claude/skills|agents` del proyecto), `tc2 doctor --recursos --json` da en `inventario` exactamente lo instalado y habilitado, con su origen, ordenado y sin lo deshabilitado
- [ ] C2: con `recursos: [{id: fantasma, tipo: skill}]` en la configuracion sale codigo 3 y `recursos fantasma falla motivo=no-instalado tipo=skill arreglo=...` con arreglo no vacio, y una linea en stderr
- [ ] C3: una tarea abierta con `roles.revision.skills` que nombra algo no instalado da `falla` con `tarea=TASK-NNN` y su arreglo; si existe, nada
- [ ] C4: texto de 5 lineas como mucho con primera linea `doctor <ok|rechazada> fallos=N avisos=N no-comprobado=N skills=N agentes=N inventario=<12hex>` y `y N mas: tc2 doctor --recursos --json`
- [ ] C5: `--json` en una linea con `hallazgos` (tambien los ok) e `inventario`, sin ANSI y legible por serde_json
- [ ] C6: no escribe nada: huellas de las carpetas temporales, `git status --porcelain` y `git for-each-ref` iguales antes y despues
- [ ] C7: con C2 montado, `tc2 doctor` lleva el hallazgo agregado `recursos no-instalados falla` con arreglo `tc2 doctor --recursos`
- [ ] C8: `installed_plugins.json` corrupto o `installPath` inexistente da `no-comprobado` con motivo, nunca ok
- [ ] C9: suelo del manifiesto (F1-14 punto 8): sin el plugin `taskcodetwo` una sola `falla` con el arreglo de instalarlo; con la 0.16 instalada `aviso` de nivel estorba; sin engram, context7 o rtk `aviso` de nivel conveniente con codigo 0
- [ ] C10: determinista: dos ejecuciones, y otra creando las carpetas en otro orden, dan el mismo stdout y la misma huella
- [ ] C11: `doctor --recursos x` da `doctor uso motivo=argumentos`

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T19:27:49Z | plan | automatico | persona | 0.16.1 |
