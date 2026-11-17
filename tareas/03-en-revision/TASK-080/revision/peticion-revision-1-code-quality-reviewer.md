# Peticion de revision — TASK-080 (ronda 1)

- Tarea: TASK-080 — F3-9 Inventario de skills y agentes instalados
- Rama revisada: feature/task-080-f3-9-inventario-de-skills-y-agentes-inst
- Rama base: develop
- Commit revisado (HEAD): 79c0ce94aa17c4a16e87c010e64ab5ef88e25586
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-080 (criterios de aceptacion y plan)
- Alcance: completa (motivo: primera ronda de revision)

## Instrucciones para el agente revisor

Eres un revisor INDEPENDIENTE del agente que implemento. Tu trabajo es
reproducir empiricamente, no leer el diff y opinar: clona el repo a un
directorio temporal, corre la suite tu mismo (salvo que esta
peticion ya traiga la verificacion de la ronda en verde para el commit revisado;
entonces vale ese resultado) y construye el caso que
rompe el codigo antes de reportarlo. Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. Vuelca tu salida en el
informe de esta ronda (informe-revision-1-code-quality-reviewer.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-080/tarea.md
- tareas/03-en-revision/TASK-080/planificacion/
- docs/adr/F1-14-distribucion-e-instalacion.md
- docs/distribucion/instalar.md

## Commits a revisar (git log develop..HEAD)

````
79c0ce9 docs(TASK-080): autorrevision con rust-wasm-reviewer y code-quality-reviewer
b1c1c92 feat(TASK-080): inventario con los plugins sincronizados desde claude.ai
185b6ff docs(TASK-080): doctor --recursos en instalar.md y adenda de F1-14
4c609ec feat(TASK-080): tc2 doctor --recursos
d3b0f4e feat(TASK-080): evaluar_recursos del manifiesto de F1-14
74711b3 refactor(TASK-080): desinstalar lee installed_plugins.json con el inventario
60998eb feat(TASK-080): inventario de skills y agentes instalados
7214ed5 chore(TASK-080): tarea en curso
````

## Diff de tu dominio (2 fichero(s) de 12; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/docs/adr/F1-14-distribucion-e-instalacion.md b/docs/adr/F1-14-distribucion-e-instalacion.md
index 22ebc38..285ecc3 100644
--- a/docs/adr/F1-14-distribucion-e-instalacion.md
+++ b/docs/adr/F1-14-distribucion-e-instalacion.md
@@ -6,6 +6,7 @@
 - Depende de: [ADR F1-4](F1-4-operaciones-git-y-contrato-de-salida.md) (git como proceso, contrato de salida y códigos), [ADR F1-6](F1-6-identidades-agente-revisor-persona.md) (GitHub por HTTP con JWT de App: el binario lleva TLS), [ADR F1-8](F1-8-seguridad-modelo-de-amenazas.md) (el CLI en la zona Z1 de la máquina de la persona), [ADR F1-9](F1-9-formato-de-tarea-plan-y-especificacion.md) (YAML estándar que rechaza claves duplicadas), [ADR F1-10](F1-10-carriles-y-modo-automatico.md) (suelo en el binario que la config solo endurece), [ADR F1-11](F1-11-extensiones-declaradas.md) (decisión 10: comandos sí, hooks solo como atajo opcional; `verificacion` y revisoras) y [ADR F1-12](F1-12-actualizacion-entre-versiones.md) (nombres `tc2` y `taskcodetwo`, npm `taskcodetwo` libre, convivencia con la 0.16)
 - Decide: Carlos, con sus decisiones del 2026-11-14 (medir Rust frente a TypeScript con el mismo comando; manifiesto de recursos con cuatro niveles y engram no obligatorio; nombres de F1-12; comandos deterministas sobre hooks; no trocear; no descargar toolchains, targets ni imágenes: «haz la comparación con lo que ya está instalado») y la consigna «la opción recomendada» (D1a a D9a del plan de TASK-014)
 - Adenda (2026-11-17, TASK-028, D4 de su plan): la primera `tc2 init` (F3-1) **no** comprueba el manifiesto de recursos (punto 8). Esa comprobación llega con `tc2 doctor --recursos` (F3-8, con el inventario de F3-9), y `init` la llamará cuando exista.
+- Adenda (2026-11-17, TASK-080, F3-9): `tc2 doctor --recursos [--json]` existe. `evaluar_recursos` (`crates/tc2/src/recursos.rs`) es la función pura del punto 8 sobre el inventario de F3-9 (`crates/tc2/src/inventario.rs`); de momento el suelo es el plugin `taskcodetwo` (obligatorio, una sola `falla` mientras no esté: sus skills de fase llegan con él), engram, context7 y rtk (conveniente) y la 0.16 habilitada (estorba). Lo «necesario» sale de `recursos:` y de `roles.*` de las tareas no terminadas. Los hooks de terceros que estorban y las versiones mínimas de `git` y `tc2` siguen en `tc2 doctor` normal o pendientes. El `doctor` normal lleva el hallazgo agregado `recursos no-instalados`.
 
 ## Contexto
 
diff --git a/docs/distribucion/instalar.md b/docs/distribucion/instalar.md
index b0726b7..15ed710 100644
--- a/docs/distribucion/instalar.md
+++ b/docs/distribucion/instalar.md
@@ -206,6 +206,55 @@ tc2 desinstalar --maquina
   --proyecto` fuera de un repo sale `rechazada motivo=no-es-repo` sin hacer
   nada, tampoco en la máquina.
 
+## Comprobar los recursos: skills, agentes y plugin
+
+Lo construyó TASK-080 (F3-9; ADR F1-14, punto 8, y F1-15, decisión 2).
+`tc2 doctor --recursos` hace el inventario de lo instalado y dice qué falta,
+sin escribir nada:
+
+```bash
+tc2 doctor --recursos          # 5 líneas como mucho
+tc2 doctor --recursos --json   # una línea con todos los hallazgos y el inventario
+```
+
+- **Inventario**: los plugins de `<claude>/plugins/installed_plugins.json`
+  que estén habilitados (`enabledPlugins` de `<claude>/settings.json`,
+  `.claude/settings.json` y `.claude/settings.local.json` del proyecto, el
+  último que lo dice gana; un plugin en `false` no cuenta), con el prefijo
+  `plugin:` en sus skills y agentes; los sincronizados desde claude.ai
+  (`<claude>/plugins/synced/`, id `<name>@synced`: los del `manifest.json`
+  de cuenta más reciente, salvo `"<name>@synced": false` o
+  `syncClaudeAiPlugins: false`, y nunca los de `plugins/.trash/`);
+  `<claude>/skills` y `<claude>/agents`;
+  `.claude/skills` y `.claude/agents` del proyecto; y el agente integrado
+  `general-purpose`. `<claude>` es `CLAUDE_CONFIG_DIR` o `~/.claude`. El id
+  es el `name:` del frontmatter (con aviso `nombre-distinto` si no coincide
+  con la carpeta o el fichero).
+- **Primera línea**: `doctor <ok|rechazada> fallos=N avisos=N
+  no-comprobado=N skills=N agentes=N inventario=<12 hex>`; la huella es la
+  que anotará `roles.<rol>.inventario`. Detrás, los hallazgos que no son `ok`
+  (`recursos <id> <estado> motivo=... tipo=... [tarea=...] [nivel=...]
+  arreglo=...`) y, si no caben, `y N mas: tc2 doctor --recursos --json`.
+- **Qué se comprueba**:
+  - obligatorio: el plugin `taskcodetwo`. Mientras no esté, una sola `falla`
+    con `arreglo=claude plugin install taskcodetwo` (código 3 en todas las
+    máquinas hasta que se publique, D3a);
+  - necesario: lo que nombran `recursos:` de `.taskcode/config.yml` (de
+    `origin/develop`) y `roles.*.skills` y `roles.revision.agente` de las
+    tareas no terminadas: si no está, `falla motivo=no-instalado` con su
+    arreglo (y `tarea=` si viene de una tarea);
+  - conveniente: engram, context7 (por el nombre del servidor MCP o del
+    plugin) y `rtk` en el PATH: `aviso` y sigue;
+  - estorba: la 0.16 (`taskcode-plugin`) habilitada a la vez: `aviso`.
+- **Lo que no se puede leer** (`installed_plugins.json` corrupto, un
+  `installPath` que no existe) sale `no-comprobado` con el motivo, y lo que
+  dependía de eso también: nunca `ok`.
+- El `tc2 doctor` normal lleva un hallazgo agregado, `recursos no-instalados
+  falla ids=N arreglo=tc2 doctor --recursos`, si falta algo.
+- **No se lee**: las rutas propias de `skills` o `agents` en `plugin.json`,
+  los agentes en subcarpetas ni la skill única en la raíz de un plugin;
+  hooks de terceros que estorban, aún no.
+
 ## Compatibilidad con el plugin
 
 El plugin pregunta al `tc2` que encuentra (ADR F1-14, punto 5):
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-080/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-080/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-080/tarea.md                                    | 36 ++++++++++++++++++++++++++++++++++--
 3 files changed, 34 insertions(+), 2 deletions(-)
````
