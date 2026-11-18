# Peticion de revision — TASK-084 (ronda 1)

- Tarea: TASK-084 — F3-12 Progreso en vivo: fases que faltan y tiempo restante
- Rama revisada: feature/task-084-f3-12-progreso-en-vivo-fases-que-faltan
- Rama base: develop
- Commit revisado (HEAD): efee1212394fb2f28018b63e4dc78f56a39b4f6e
- Fecha: 2026-11-18
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-084 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-084/tarea.md
- tareas/03-en-revision/TASK-084/planificacion/
- docs/adr/F1-17-progreso-y-estimacion.md
- docs/distribucion/instalar.md

## Commits a revisar (git log develop..HEAD)

````
efee121 docs(TASK-084): autorrevision con rust-wasm-reviewer y code-quality-reviewer
37f97c1 docs(TASK-084): adenda de F1-17 y status line en instalar.md
47f812e feat(TASK-084): tc2 status --seguir y propuesta de la status line en init
e058b03 feat(TASK-084): las transiciones refrescan el progreso en vivo
0371de9 feat(TASK-084): linea de progreso y tc2 estado --stdin sin git
e9cf891 feat(TASK-084): fichero de progreso y calculo puro
29fee1d refactor(TASK-084): escribir_local en la huella con reintento del rename
a9a5c7e chore(TASK-084): tarea en curso
````

## Diff de tu dominio (2 fichero(s) de 20; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/docs/adr/F1-17-progreso-y-estimacion.md b/docs/adr/F1-17-progreso-y-estimacion.md
index 4fcf8dd..1a4c1b6 100644
--- a/docs/adr/F1-17-progreso-y-estimacion.md
+++ b/docs/adr/F1-17-progreso-y-estimacion.md
@@ -15,6 +15,14 @@
   - **Commits de una tarea (punto 1)**: los de `git log <origin/develop | develop>` cuyo asunto la nombra (`TASK-N`, sin mayúsculas, normalizado a tres cifras) y los de `M^1..M^2` del merge `merge(feature|fix): feature|fix/task-nnn-…` de la 0.16; sin `develop`, ninguno. Verificado con los asuntos de los merges de este repo.
   - **Caso de C1 de TASK-083**: con `start` 10:00, commit 10:20, `pausa` 10:30, `reanudar` 11:30, commit 11:40, `review` 13:00 y 13:30 y `finish` 13:40, la regla de este ADR (y la PoC) da implementación **85** (20 + 10 + 10 + 45), rondas 30 y cierre 10; el 75 del enunciado es el de la misma tarea sin el commit de las 11:40. Se sigue la regla, con paridad exacta con `poc-f1-17.py` (fixture de `tests/estimacion.rs`).
   - **`doctor`**: el hallazgo `estado historico ok usables=N descartadas=M` (sin `git log`: los descartes no dependen de los commits).
+- Adenda (2026-11-18, TASK-084, F3-12; decisiones D1A a D4A de su plan, con la consigna del H3 «la opción recomendada»):
+  - **Dónde vive el fichero (D1A, divergencia del punto 7)**: no es `estado.json` en el directorio git del worktree de la tarea, sino **`.git/taskcodetwo/progreso/TASK-NNN.json`** en la carpeta común (la huella de F1-12, `huella::CARPETA_PROGRESO`), con el anterior en `TASK-NNN.anterior.json`. Motivo: el del worktree no se ve desde el principal y queda fuera de lo que borra `tc2 desinstalar --proyecto` (que borra `.git/taskcodetwo/` entero). Cada paso es temporal + `rename` con reintento (`huella::escribir_local`, que usan también la marca de sincronización y la caché de identidad); el reintento entra con un lector abierto sin `FILE_SHARE_DELETE` (probado en Windows en `huella::tests`).
+  - **Qué guarda**: versión, ID, fase (`en-curso` es implementación; `en-revision`, rondas), si está pausada, las fases que faltan, lo transcurrido en la fase hasta el último evento (regla del real, punto 1), el último evento, la hora del cálculo, nivel, `n`, avisos saneados y la **tabla de lo que falta** de 0 a 480 min cada 5 (de `restante`). Su tamaño no depende del número de tareas (L1). Sin semillas (adenda de TASK-083), sin parecidas la tabla va vacía y la línea dice `faltan: sin estimacion (n=N)` (el texto de C5 de TASK-084).
+  - **Cuándo se escribe**: en cada transición de la 2.0 que sale bien (`start`, `pausa`, `reanudar`); `descartar`, `liberar` y `reasignar` lo borran en el clon que las lanza. `start` lo calcula dentro de la toma con el mismo histórico y el mismo `git log` que la estimación (un solo `git log`, R1-05 de TASK-083); las demás releen el estado conocido (sin `fetch`) y un `git log` (O(N), solo en transiciones). Un fallo al escribirlo no cambia la transición: `progreso=no-escrito` y un aviso por stderr con el mismo código. `review` y `finish` (F6-2, F7-1) lo llamarán por `progreso::refrescar` cuando existan; hasta entonces la fase de rondas solo se alcanza con datos de prueba.
+  - **Lectura (`tc2 estado --stdin`, la `STATUS_LINE` de F1-12)**: su firma no recibe `Git`; busca el repo subiendo desde `workspace.current_dir` del JSON (o `cwd`, o la carpeta del proceso) hasta un `.git` (carpeta, o fichero `gitdir:` con `commondir`) y abre un fichero (en el principal, los de las tareas en curso: la de cálculo más reciente y `+N en curso`). Lo transcurrido es el del cálculo más el reloj desde el último evento o desde el `mtime` de `logs/HEAD` del worktree; **cada hueco con el tope de 45 min** (el plan de TASK-084 lo concreta así frente al «hueco abierto entero» del punto 5: con el tope, la vista no salta de golpe a «desbordada» tras una pausa no declarada), y con más de 45 min sin actividad añade `sin actividad registrada hace N min`. Pausada, el reloj no corre. Un fichero ilegible da el anterior con `(dato de hace N min)`; sin ninguno, `progreso ilegible`. Siempre una línea y código 0; sin tarea, `tc2 · sin tarea en curso`. Medido en Windows 11 (2026-11-18), de lanzar el proceso a leer su salida: binario release, mediana de 17 ms en caliente (15 ejecuciones, 16-19 ms; la primera en frío, 83 ms); binario de pruebas, 18-20 ms con 10 y con 1000 tareas (C1 de TASK-084, que falla por encima de 200 ms). No incluye el arranque del shell con el que Claude Code la lanza (Git Bash o PowerShell en Windows).
+  - **`tc2 status --seguir`**: la misma lectura cada 2 s desde la carpeta actual; imprime una línea nueva solo si cambia el texto, sin ANSI, y termina al cerrarse stdout (código 0) o con Ctrl+C (lo corta el sistema).
+  - **Instalación (D2A)**: `tc2 init` solo propone la `statusLine` (línea `clase=status-line decision=pendiente motivo=pegar-a-mano orden=tc2 estado --stdin`, con el bloque JSON en `--json`); nunca lee ni escribe un `settings.json`, así que no sabe si ya está pegada y la propone en cada ejecución como `pendiente` (no `persona`: la segunda ejecución de `init` no pide nada nuevo, C2 de TASK-028). Se recomienda pegarla en `.claude/settings.local.json` del proyecto para no sustituir la de la 0.16 en `~/.claude/settings.json`; `tc2 desinstalar` solo reconoce la de `~/.claude/settings.json`.
+  - **Lote (D3A)**: queda en F4-9; el fichero lleva `version` para añadirlo. **Tarea real (D4A)**: se acepta con el E2E de las transiciones existentes; la comprobación durante una tarea real queda para F9-7.
 
 Este ADR fija **qué estima la 2.0, con qué datos, cuándo lo recalcula y dónde
 lo enseña sin gastar tokens**. Hay un solo lector del histórico,
diff --git a/docs/distribucion/instalar.md b/docs/distribucion/instalar.md
index c4c98fa..35d85b7 100644
--- a/docs/distribucion/instalar.md
+++ b/docs/distribucion/instalar.md
@@ -206,6 +206,56 @@ tc2 desinstalar --maquina
   --proyecto` fuera de un repo sale `rechazada motivo=no-es-repo` sin hacer
   nada, tampoco en la máquina.
 
+## La status line de Claude Code y `tc2 status --seguir`
+
+Lo construyó TASK-084 (F3-12; ADR F1-17, adenda de TASK-084). Mientras se
+trabaja en una tarea, la línea dice la fase, las fases que faltan y lo que
+falta según las tareas parecidas, sin gastar tokens:
+
+```text
+TASK-007 · implementacion · fases que faltan: rondas, cierre · faltan ≈50 min (p80 65) · n=6 repo
+```
+
+`tc2 init` **no lee ni escribe** ningún `settings.json`: imprime la
+propuesta como una línea `clase=status-line decision=pendiente
+motivo=pegar-a-mano orden=tc2%20estado%20--stdin` (con `--json`, el bloque
+entero en `propuesta`). Para usarla,
+añade a mano a `.claude/settings.local.json` del proyecto (recomendado: no
+sustituye la status line de la 0.16 de `~/.claude/settings.json`) o a tu
+`~/.claude/settings.json`:
+
+```json
+{
+  "statusLine": {
+    "type": "command",
+    "command": "tc2 estado --stdin"
+  }
+}
+```
+
+- **Qué lee**: Claude Code le pasa por stdin un JSON con
+  `workspace.current_dir` (y `cwd`, el mismo valor); `tc2 estado --stdin`
+  sube desde esa carpeta hasta el `.git` y abre el fichero de progreso de la
+  tarea en `.git/taskcodetwo/progreso/` (sin lanzar git ni abrir la red). Si
+  el JSON no trae la carpeta, usa la del proceso. Dentro de
+  `.taskcodetwo/worktrees/TASK-NNN` enseña esa tarea; en el principal, la
+  última calculada y `+N en curso`. Sin tarea: `tc2 · sin tarea en curso`.
+  Siempre una línea y código 0.
+- **Cuándo cambia**: lo escriben `start`, `pausa` y `reanudar` (y lo borran
+  `descartar`, `liberar` y `reasignar`); entre transiciones, la línea suma el
+  reloj (con el tope de 45 min por hueco de F1-17) y, si pasan más de 45 min
+  sin commits, dice `sin actividad registrada hace N min`. Claude Code
+  repinta la status line con sus eventos; para que corra el reloj aunque la
+  sesión esté quieta, añade `"refreshInterval": 60` al bloque.
+- **En Windows**, Claude Code la lanza con Git Bash si está instalado (si
+  no, con PowerShell); `tc2` tiene que estar en el PATH de ese shell.
+- **`tc2 status --seguir`** hace lo mismo en un terminal aparte: imprime la
+  línea, y otra cada vez que cambia (mira cada 2 s); se para con Ctrl+C.
+- `tc2 desinstalar --maquina` quita la `statusLine` de
+  `~/.claude/settings.json` si su `command` es exactamente
+  `tc2 estado --stdin`; la de `.claude/settings.local.json` del proyecto la
+  quita la persona.
+
 ## Comprobar los recursos: skills, agentes y plugin
 
 Lo construyó TASK-080 (F3-9; ADR F1-14, punto 8, y F1-15, decisión 2).
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-084/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-084/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-084/tarea.md                                    | 36 ++++++++++++++++++++++++++++++++++--
 3 files changed, 34 insertions(+), 2 deletions(-)
````
