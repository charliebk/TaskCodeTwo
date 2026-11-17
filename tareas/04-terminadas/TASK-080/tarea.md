---
id: TASK-080
titulo: "F3-9 Inventario de skills y agentes instalados"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-080-f3-9-inventario-de-skills-y-agentes-inst
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1287739
tokens_implementacion: 56231819
tokens_revision: 6629490
creado: 2026-11-14
actualizado: 2026-11-17
dependencias: [TASK-016, TASK-073]
---
## Objetivo

Inventario de skills y agentes instalados (máquina y proyecto) y `doctor` que avisa de un catálogo que nombra algo no instalado (RF-14)

Tarea F3-9 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-1, F1-15.

Contexto (Carlos, 2026-11-14, RR-17, imprescindible): para cada tarea y cada rol (diseño, implementación, revisión) hay que elegir las skills y los agentes adecuados y el modelo según complejidad y carril, solo entre lo que está instalado en la máquina y el proyecto, anotar la elección en la tarea y medir si acierta (hallazgos por revisora y coste por modelo). En la 0.16 existe con `catalogo-skills.yml`, `skills_recomendados` y `modelo_sugerido`, pero se usó en 21 / 0 / 2 tareas de ToolsaWeb / OpenDockTrading / TaskCode porque el catálogo apenas se configuró: hoy la elección la hace el agente sin criterio registrado. Nada debe decidirlo el modelo «a ojo»: reglas deterministas. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §1–4.

## Criterios de aceptacion
- [x] C1: con `CLAUDE_CONFIG_DIR` y proyecto falsos en temporales (skills y agents de usuario, un plugin habilitado con `installPath`, otro deshabilitado en `settings.json`, y `.claude/skills|agents` del proyecto), `tc2 doctor --recursos --json` da en `inventario` exactamente lo instalado y habilitado, con su origen, ordenado y sin lo deshabilitado
- [x] C2: con `recursos: [{id: fantasma, tipo: skill}]` en la configuracion sale codigo 3 y `recursos fantasma falla motivo=no-instalado tipo=skill arreglo=...` con arreglo no vacio, y una linea en stderr
- [x] C3: una tarea abierta con `roles.revision.skills` que nombra algo no instalado da `falla` con `tarea=TASK-NNN` y su arreglo; si existe, nada
- [x] C4: texto de 5 lineas como mucho con primera linea `doctor <ok|rechazada> fallos=N avisos=N no-comprobado=N skills=N agentes=N inventario=<12hex>` y `y N mas: tc2 doctor --recursos --json`
- [x] C5: `--json` en una linea con `hallazgos` (tambien los ok) e `inventario`, sin ANSI y legible por serde_json
- [x] C6: no escribe nada: huellas de las carpetas temporales, `git status --porcelain` y `git for-each-ref` iguales antes y despues
- [x] C7: con C2 montado, `tc2 doctor` lleva el hallazgo agregado `recursos no-instalados falla` con arreglo `tc2 doctor --recursos`
- [x] C8: `installed_plugins.json` corrupto o `installPath` inexistente da `no-comprobado` con motivo, nunca ok
- [x] C9: suelo del manifiesto (F1-14 punto 8): sin el plugin `taskcodetwo` una sola `falla` con el arreglo de instalarlo; con la 0.16 instalada `aviso` de nivel estorba; sin engram, context7 o rtk `aviso` de nivel conveniente con codigo 0
- [x] C10: determinista: dos ejecuciones, y otra creando las carpetas en otro orden, dan el mismo stdout y la misma huella
- [x] C11: `doctor --recursos x` da `doctor uso motivo=argumentos`

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

## Resultado

Hecho: modulo `inventario.rs` (skills y agentes instalados y habilitados en la maquina y el proyecto: `installed_plugins.json`, plugins sincronizados desde claude.ai en `plugins/synced/`, `<dir_claude>/skills|agents`, `.claude/skills|agents` del proyecto y `general-purpose`, con su origen, orden estable y huella de 12 hex), `recursos.rs` (`evaluar_recursos`, contrato del manifiesto de F1-14 punto 8 con los niveles obligatorio, necesario, conveniente y estorba) y `tc2 doctor --recursos [--json]`; el `doctor` normal anade un hallazgo agregado de recursos no instalados. Lo que no se puede leer sale `no-comprobado`, nunca `ok` ni «no instalado». `desinstalar` reutiliza la lectura de `installed_plugins.json`. Adenda en F1-14 e `instalar.md`. D1a-D5a. Ejecucion real en esta maquina: 90 skills y 46 agentes.

Consecuencia de D3a: `doctor` y `doctor --recursos` dan codigo 3 en todas las maquinas hasta que se publique el plugin `taskcodetwo`.

No verificado: Linux y macOS; plugins con `scope` de proyecto o local reales; contenido real de `plugins/synced/` (el agente de implementacion tuvo denegada su lectura y se baso en la documentacion «Plugin loading reference» y en la descripcion del orquestador); los `commands/` de los plugins, `CLAUDE_CODE_PLUGIN_CACHE_DIR`, los plugins de `skills/` y los obligatorios de la organizacion no se leen; de varias cuentas sincronizadas se toma la de `lastUpdated` mas reciente (suposicion).

Revision ronda 1: aprobada por los dos, sin CRITICOS ni IMPORTANTES (el IMPORTANTE de los plugins sincronizados se encontro y corrigio antes de la revision, b1c1c92). Once MENORES corregidos en 5aa92cc, 70a6989, 61580c7 y c8bb768 y verificados con `verificar-menores`: H1 entradas de plugin que no son lista, H2 carpetas ilegibles y nombres no UTF-8, H3 `name` sincronizado que saldria de su carpeta e `installPath` relativo, H4 fecha comparada por valor, H5 test de tareas descartadas, MEN-1 codificacion `%XX` en la guia, MEN-2 comentario YAML en `name:`, MEN-3 `rtk` con PATH controlado, MEN-4 pendientes de «necesario» en la adenda, MEN-5 tipo de recurso desconocido como `no-comprobado`, MEN-6 convenientes con inventario incompleto como `no-comprobado`. Limite: la rama de `recomendados()` del tipo desconocido no se alcanza con el binario (la configuracion lo rechaza antes); se prueba la funcion pura.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T19:27:49Z | plan | automatico | persona | 0.16.1 |
| 2026-11-17T19:35:53Z | approve | automatico | persona | 0.16.1 |
| 2026-11-17T19:35:54Z | start | automatico | persona | 0.16.1 |
| 2026-11-17T21:16:42Z | review | automatico | persona | 0.16.1 |
| 2026-11-17T21:53:01Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-17T19:35:52Z | diseno | agente:aadf9663d0b6fec03 | 36 | 91525 | 1179081 | 17097 | 33 | 1287739 | - | - |
| 2026-11-17T21:16:41Z | implementacion | agente:af287d4b3fb93c05e | 298 | 394394 | 39961012 | 139662 | 50 | 40495366 | - | - |
| 2026-11-17T21:16:42Z | implementacion | sesion:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 40 | 26594 | 6629384 | 7561 | 0 | 6663579 | - | - |
| 2026-11-17T21:26:10Z | revision | agente:a2b7afeeb78cf98df | 50 | 130855 | 2667514 | 20181 | 3 | 2818600 | 1 | rust-wasm-reviewer |
| 2026-11-17T21:29:53Z | revision | agente:adae8b495ec722499 | 62 | 131104 | 3659147 | 20577 | 4 | 3810890 | 1 | code-quality-reviewer |
| 2026-11-17T21:44:35Z | implementacion | agente:a201d8ddaa553dfef | 126 | 181332 | 8840140 | 51276 | 27 | 9072874 | - | - |

## Verificacion ligera

- 2026-11-17T21:44:36Z: ronda 1; commit revisado 79c0ce94aa17c4a16e87c010e64ab5ef88e25586; commit verificado c8bb768d4b67a5e972dd86646b7afa49b5b76c08; ficheros: crates/tc2/src/doctor/recursos.rs, crates/tc2/src/inventario.rs, crates/tc2/src/recursos.rs, crates/tc2/tests/recursos.rs, docs/adr/F1-14-distribucion-e-instalacion.md, docs/distribucion/instalar.md; corregidos: MEN-1, MEN-2, MEN-3, MEN-4, MEN-5, MEN-6, H1, H2, H3, H4, H5; pendientes: ninguno

## Duracion

- Estimada: 300 min
- Real (de la rama abierta al cierre con merge, sin pausas): 137 min
- Paso de la hora: 137 min reales frente a 300 estimados
- Cierre (finish): 488 s, de 2026-11-17T21:44:53Z a 2026-11-17T21:53:01Z
