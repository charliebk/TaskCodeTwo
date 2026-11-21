# Peticion de revision — TASK-046 (ronda 1)

- Tarea: TASK-046 — F5-6 Hook local rapido, opcional y con confianza
- Rama revisada: feature/task-046-f5-6-hook-local-rapido-opcional-y-con-co
- Rama base: develop
- Commit revisado (HEAD): e33879a25a4399d5755559aa147ff96497ca231a
- Fecha: 2026-11-21
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-046 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-046/tarea.md
- tareas/03-en-revision/TASK-046/planificacion/
- docs/ci/verificacion-reutilizable.md

## Commits a revisar (git log develop..HEAD)

````
e33879a docs(TASK-046): hook local de F5-6, entrada de settings.json y cifra del presupuesto de 1 s
b2f573b feat(TASK-046): desinstalar --maquina borra el almacen de confianza
74017d5 feat(TASK-046): tc2 verificar con la puerta de confianza y adaptador --hook
1245aa5 feat(TASK-046): confianza local por clave y tc2 confiar
6e88918 chore(TASK-046): tarea en curso
````

## Diff de tu dominio (1 fichero(s) de 15; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/docs/ci/verificacion-reutilizable.md b/docs/ci/verificacion-reutilizable.md
index 39d0100..1eff885 100644
--- a/docs/ci/verificacion-reutilizable.md
+++ b/docs/ci/verificacion-reutilizable.md
@@ -11,7 +11,8 @@ decisión 1; RR-15): no conocen sus pasos, sus grupos ni sus herramientas.
 
 Fuera de F5-1: `carril` y `revision-ia` (F6), y la definición de CI en otro
 proyecto de GitLab (F2-9). El informe JSON para el revisor (F5-4), la caché
-por árbol (F5-2) y el runner propio (F5-5) se describen más abajo.
+por árbol (F5-2), el runner propio (F5-5) y el atajo local con confianza
+(F5-6) se describen más abajo.
 
 ## Entradas
 
@@ -433,6 +434,122 @@ En una consola de PowerShell **de administrador** en tu máquina, para un repo
   caché por árbol en el runner propio (en el E2E el diff tocaba el workflow y
   la regla de rutas no reutiliza, por diseño).
 
+## Hook local rápido, opcional y con confianza (F5-6)
+
+TASK-046. Fuera de la CI, la misma verificación tiene un atajo local: un
+**comando determinista** y, si la persona quiere, un hook de Claude Code que
+lo llama (ADR [F1-11](../adr/F1-11-extensiones-declaradas.md), decisión 10).
+Quitar el hook no cambia ninguna garantía: las puertas son el check
+`verificacion`, el check `estado` y la transición.
+
+```text
+tc2 confiar verificacion.comando        # una vez, tras revisar la orden y sus scripts
+tc2 verificar --perfil rapida [--json]  # cualquier perfil declarado (D6)
+tc2 verificar --perfil rapida --hook    # lo mismo, devuelto como lo espera el evento Stop
+```
+
+**Nada del repo se ejecuta sin confianza** (ADR
+[F1-8](../adr/F1-8-seguridad-modelo-de-amenazas.md), decisión 5).
+`tc2 verificar` lee `verificacion` de `.taskcode/config.yml` **del árbol de
+trabajo** (D3a), calcula la huella de la orden (con la shell) y de lo que
+ejecuta (los scripts versionados que nombra y, a su vez, los que nombran
+estos, con `%~dp0` resuelto, más los globs de `confianza_incluye`) y la
+compara con la confiada:
+
+| Estado | Qué hace `tc2 verificar` |
+|---|---|
+| `nunca` (sin confianza en este repo) | no ejecuta nada; código 3, `motivo=sin-confianza confianza=nunca` y el aviso dice que hay que revisar y ejecutar `tc2 confiar verificacion.comando` |
+| `cambiado` (la orden, un script del cierre o un fichero de `confianza_incluye` cambió) | igual, con `confianza=cambiado` |
+| `confiado` | lanza `verificacion.shell` con `<comando> --perfil <p>`, `TC2_PERFIL=<p>` y el `timeout` del perfil; la salida del comando va a stderr |
+
+La confianza vive **fuera del árbol**, en
+`%LOCALAPPDATA%\taskcodetwo\confianza\` (Windows) o
+`$XDG_CACHE_HOME/taskcodetwo/confianza/` (`~/.cache/...` si no está
+definida), un fichero por repo con nombre el sha256 del directorio común de
+Git: los worktrees del repo la comparten y otro clon no. Un fichero del repo
+que diga «confiado» no cuenta. Si se borra la carpeta, todo vuelve a `nunca`;
+`tc2 desinstalar --maquina` la borra.
+
+**Resultado** (F1-11, decisión 8): código 0 del comando, `resultado=verde`;
+otro código, `motivo=rojo` y código 3; un `timeout` agotado es rojo
+(`motivo=timeout`) salvo en `rapida`, donde es `resultado=excedido` con
+código 0 (presupuesto que no bloquea). Al agotarse se mata el comando y lo
+que haya lanzado (`taskkill /T` en Windows, el grupo de procesos en Unix).
+
+### Activar el hook (opcional, D4a y D5a)
+
+No hay orden que lo instale: quien lo quiera añade esta entrada a su
+`.claude/settings.local.json` del proyecto (personal, no se versiona) o a su
+`~/.claude/settings.json`:
+
+```json
+{
+  "hooks": {
+    "Stop": [
+      {
+        "hooks": [
+          {
+            "type": "command",
+            "command": "tc2 verificar --perfil rapida --hook",
+            "timeout": 630
+          }
+        ]
+      }
+    ]
+  }
+}
+```
+
+`timeout` (segundos, por defecto 600 en Claude Code) tiene que ser algo
+mayor que `verificacion.perfiles.rapida.timeout`, o Claude Code corta el hook
+antes que `tc2`. El evento `Stop` no admite `matcher`. Comportamiento con
+`--hook` (la decisión es la misma que sin él; solo cambia cómo se devuelve):
+
+| Caso | Código | Qué ve el agente |
+|---|---|---|
+| `stop_hook_active: true` en el JSON de stdin (Claude Code ya continúa por un hook) | 0 | nada; no se ejecuta |
+| sin confianza, sin perfil `rapida` o con la configuración inválida | 0 | no bloquea; una línea en stderr dice qué hacer (`tc2 confiar verificacion.comando`, o el arreglo de la configuración); no se ejecuta |
+| rápida en verde o `excedido` | 0 | nada |
+| rápida en rojo | 2 | bloquea la parada; en stderr el motivo y las últimas 15 líneas de la salida del comando |
+
+`desinstalar` no edita `settings.json`: avisa de los hooks que nombran `tc2`
+y los deja (F2-14).
+
+### Presupuesto de 1 s (D1a)
+
+«≤ 1 s» es el coste propio de `tc2` hasta decidir (leer la configuración,
+cierre, huella, almacén y lanzar o rechazar), en frío y en caliente; cuando no
+se ejecuta nada es el tiempo total. La rápida misma está acotada por su
+`timeout`. La huella reutiliza el sha de cada fichero cuyo `(tamaño, mtime)`
+no cambió (D2a); la orden se hashea siempre y `tc2 confiar` rehashea todo.
+
+Medido con la prueba pesada
+`presupuesto_de_un_segundo_con_un_cierre_como_el_de_toolsaweb` (51 scripts
+versionados, 1 210 462 bytes, orden que sale enseguida; tiempo de pared de
+`tc2 verificar --perfil rapida` entero, con el arranque de `tc2`, de `git` y
+de `cmd`), en Windows 11, binario `--release`, cuatro ejecuciones el
+2026-11-21 sin controlar la carga de la máquina:
+
+| Caso | Tiempo (ms) |
+|---|---|
+| sin confiar (cierre entero hasheado, no se ejecuta) | 250 – 684 |
+| en frío (todos los `mtime` cambiados: 1 210 462 bytes rehasheados) | 318 – 595 |
+| en caliente (0 bytes rehasheados) | 249 – 419 |
+
+Con el binario de depuración, el peor caso fue 530 ms (en frío). Se lanza a
+mano: `cargo test -p tc2 --release --test verificar -- --ignored --nocapture`
+(la verificación del repo no corre las pruebas ignoradas).
+
+**Límites** (F1-8, decisión 5): no entra en el cierre el código al que se
+llega por herramientas (`build.rs`, tests, dependencias) ni lo que se carga
+por nombre de módulo y no por ruta; eso se declara en `confianza_incluye`.
+Un cambio que conserve tamaño y `mtime` no se detecta en caliente. No se han
+revisado enlaces simbólicos ni la carrera entre hashear y ejecutar. En
+Windows y macOS las rutas del cierre se comparan sin distinguir mayúsculas.
+Este repo aún no puede estrenar el hook: su `.taskcode/config.yml` es de la
+0.16 (sin `formato: 2`) y `tc2 verificar` la rechaza; con `--hook` sale con 0
+y avisa.
+
 ## Divergencias con los ADR
 
 1. **Argumentos por perfil** (D1): F1-7, decisión 1, fija `--perfil
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/01-en-diseno/TASK-046/tarea.md                                                     | 47 -----------------------------------------------
 tareas/{01-en-diseno => 02-en-curso}/TASK-046/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-046/planificacion/plan-final.md                 |  0
 tareas/02-en-curso/TASK-046/tarea.md                                                      | 88 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 4 files changed, 88 insertions(+), 47 deletions(-)
````
