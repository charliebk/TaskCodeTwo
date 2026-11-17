---
id: TASK-080
titulo: "F3-9 Inventario de skills y agentes instalados"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-curso
plan_aprobado: true
rama: feature/task-080-f3-9-inventario-de-skills-y-agentes-inst
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1287739
tokens_implementacion: 40495366
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

## Autorrevision

- rust-wasm-reviewer — b1c1c92
- code-quality-reviewer — b1c1c92

Hallazgos propios:

- IMPORTANTE corregido antes de la revision (b1c1c92): el inventario no
  veia los plugins sincronizados desde claude.ai (`plugins/synced/`), y en
  esta maquina contaba 25 skills en vez de 90. Ahora se leen segun la
  documentacion «Plugin loading reference» (`<name>@synced`, ceden ante un
  instalado del mismo nombre, `.trash/` no cuenta, `syncClaudeAiPlugins:
  false` los apaga).
- MENOR: siguen sin leerse los `commands/` de los plugins,
  `CLAUDE_CODE_PLUGIN_CACHE_DIR`, los plugins de `skills/` y los
  obligatorios de la organizacion; `subagent-catalog:*` y
  `anthropic-skills:*` no aparecen (otra fuente).
- MENOR: de varias cuentas sincronizadas se toma la de `lastUpdated` mas
  reciente (suposicion, no documentada).
- MENOR: los arreglos `claude plugin install taskcodetwo` y `claude plugin
  disable <clave>` son provisionales (marketplace desconocido).
- Consecuencia de D3a: `doctor` y `doctor --recursos` dan codigo 3 en todas
  las maquinas hasta que se publique el plugin `taskcodetwo`.
- Mutaciones comprobadas (8, rojo y restauradas): `enabledPlugins: false`,
  plugin obligatorio ilegible, argumentos de `--recursos`, orden del
  inventario, filtro de terminadas, agregado del `doctor` normal,
  sincronizado que no cede, cuenta mas antigua.
- Verificacion propia: `--test recursos --test desinstalar` 31 de 31 y
  `--lib inventario` 10 de 10.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T19:27:49Z | plan | automatico | persona | 0.16.1 |
| 2026-11-17T19:35:53Z | approve | automatico | persona | 0.16.1 |
| 2026-11-17T19:35:54Z | start | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-17T19:35:52Z | diseno | agente:aadf9663d0b6fec03 | 36 | 91525 | 1179081 | 17097 | 33 | 1287739 | - | - |
| 2026-11-17T21:16:41Z | implementacion | agente:af287d4b3fb93c05e | 298 | 394394 | 39961012 | 139662 | 50 | 40495366 | - | - |
