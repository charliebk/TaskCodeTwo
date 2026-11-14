# Peticion de revision — TASK-005 (ronda 2)

- Tarea: TASK-005 — F1-4 ADR capa de operaciones Git/GitHub deterministas y su contrato
- Rama revisada: feature/task-005-f1-4-adr-capa-de-operaciones-git-github
- Rama base: develop
- Commit revisado (HEAD): 530d9ddd14c7646dc183e5010d8963d24fe1a058
- Fecha: 2026-11-14
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-005 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 61f15c5436be0aa7c50535710c6bb9f576fcad50 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (5 hallazgo(s)))

## Instrucciones para el agente revisor (verificacion acotada)

Eres un revisor INDEPENDIENTE del agente que implemento. La rama ya paso una
revision completa: esta ronda verifica las correcciones de la ronda anterior,
no vuelve a revisar la rama entera ni los MENORES de la ronda anterior (tu
tabla solo trae CRITICOS e IMPORTANTES). Haz esto, y nada mas:

1. Comprueba, uno por uno, que cada hallazgo pendiente de tu tabla de abajo
   queda resuelto por el diff de esta ronda (los marcados `corregido` tambien:
   verificalo, no lo des por bueno). Para un CRITICO o IMPORTANTE, reproduce el
   fallo original y comprueba que ya no ocurre; puedes leer el repo entero si la
   causa no esta en el diff.
2. Ejecuta la suite (o el comando de verificacion del proyecto) sobre el commit
   revisado y di el resultado: verificar sin ejecutar no vale. Si esta peticion
   ya trae la verificacion de la ronda en verde para ese mismo commit, no la
   repitas: cita ese resultado y corre solo los tests de lo que verificas.
3. Busca en el diff de esta ronda el fallo que la correccion abre: es justo
   donde se cuelan los fallos nuevos. Si encuentras uno, construye el caso que
   lo rompe antes de reportarlo.

Solo abre un hallazgo NUEVO si es CRITICO o si lo introduce la propia correccion
(algo que el diff de esta ronda rompe). Lo demas que veas fuera de eso va como
nota al final del informe, no en la tabla: no abre otra ronda.

Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. En la tabla, cada hallazgo pendiente que ya este
resuelto va como `corregido`; el que no, `abierto`. Vuelca tu salida en el
informe de esta ronda (informe-revision-2.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-005/tarea.md
- tareas/03-en-revision/TASK-005/planificacion/
- docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 2 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md:286 | informe-revision-1.md |
| IMP-2 | IMPORTANTE | corregido | tareas/03-en-revision/TASK-005/poc/poc-f1-4.sh:324 | informe-revision-1.md |
| IMP-3 | IMPORTANTE | corregido | docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md:349 | informe-revision-1.md |

## Commits a revisar (git log 61f15c5436be0aa7c50535710c6bb9f576fcad50..HEAD)

````
530d9dd chore(TASK-005): constancia de autorrevision ronda 2
9000b48 docs(adr): F1-4 corrige los hallazgos de la revision 1 (commit-tree sin reset, tag, cobertura, lista permitida)
4a37538 chore(TASK-005): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 61f15c5436be0aa7c50535710c6bb9f576fcad50..HEAD)

````diff
diff --git a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
index 3d6e79c..fa81bdb 100644
--- a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
+++ b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
@@ -49,6 +49,7 @@ de tokens por tarea se fija, con qué línea base.
 | **D5a** | Primera línea `comando resultado clave=valor`, con `--json` opcional. | **Elegida**: legible para persona y agente, filtrable con `grep`; `--json` para CI. |
 | D5b | Solo JSON. | Descartada: más bytes por operación y peor para la persona en el terminal. |
 | **Capa** | Módulo interno del CLI con dos puertos: `Git` (lanza el binario `git` con entorno fijo) y `Plataforma` (PR, puertas, merge por la API HTTP). | **Elegida** (decisión 1). |
+| `commit-estado` con commit en la rama y `reset --hard` | Lo que hacía la PoC de F1-2 para descartar el commit propio tras un rechazo. | Descartada: rompe la prohibición de `reset --hard` (RN-05) sin necesidad; `commit-tree` + `push <sha>:<rama>` hace el mismo compare-and-swap sin commit local (PoC B1b). |
 | Capa con `gh`/`glab` | El puerto Plataforma lanza `gh` o `glab`, como `CLI_PLATAFORMA` en `plataforma-remota.js` de la 0.16.1. | Descartada: añade una dependencia externa por plataforma (P12, RR-16). |
 | Capa con libgit2 o una librería Git | Sin binario `git`. | Descartada: worktrees, `ls-remote` con patrones y credenciales del sistema ya funcionan con `git`, que es requisito igualmente. |
 
@@ -143,20 +144,30 @@ $ taskctl reanudar TASK-007   [en principal]     (tras perder worktree y rama lo
 $ taskctl tag-anotado v0.1.0 origin/main   [en principal]
   1a: tag-anotado ok tag=v0.1.0 commit=47ea7e825b84   (codigo 0, 46 bytes)
   2a: tag-anotado sin-cambios tag=v0.1.0 commit=47ea7e825b84   (codigo 0) ; estado igual tras la 2a: si
+-- el tag ya publicado, visto desde un clon que no lo tiene en local, pedido sobre otro commit
+$ taskctl tag-anotado v0.1.0 origin/develop   [en principal]
+  1a: tag-anotado rechazada tag=v0.1.0 motivo=existe-en-otro-commit donde=remoto   (codigo 3, 75 bytes)
+-- tag local sin publicar en otro commit (p. ej. una release abortada sin red)
+$ taskctl tag-anotado v0.0.9 origin/main   [en principal]
+  1a: tag-anotado rechazada tag=v0.0.9 motivo=existe-en-otro-commit donde=local   (codigo 3, 74 bytes)
+v0.0.9 en origin tras la operacion: '' (vacio = no se publico)
 $ taskctl fetch    [en principal]     (origin apunta a una ruta inexistente)
   1a: fetch sinred remoto=origin   (codigo 4, 27 bytes)
       stderr: fetch: no se pudo leer origin
 ...
-RESULTADO B1: operaciones idempotentes (2a ejecucion sin cambios y sin-cambios o igual): C 22 ok / 0 mal ; es_ES 22 ok / 0 mal
-RESULTADO B1: salidas distintas entre LC_ALL=C y LC_ALL=es_ES.UTF-8: 0 de 22
+RESULTADO B1: operaciones idempotentes (2a ejecucion sin cambios y sin-cambios o igual): C 24 ok / 0 mal ; es_ES 24 ok / 0 mal
+RESULTADO B1: salidas distintas entre LC_ALL=C y LC_ALL=es_ES.UTF-8: 0 de 24
 RESULTADO B1: salidas que incumplen el contrato (<=5 lineas, sin ANSI, 'op resultado clave=valor'): 0
-RESULTADO B1: bytes de las lineas de contrato de las 22 operaciones del ciclo (1a ejecucion): 1147
-RESULTADO B1: codigos de salida observados (codigo:veces): 0:19 3:1 4:2
+RESULTADO B1: bytes de las lineas de contrato de las 24 operaciones del ciclo (1a ejecucion): 1296
+RESULTADO B1: codigos de salida observados (codigo:veces): 0:19 3:3 4:2
 ```
 
-Lectura: las 15 operaciones locales (más `commit-estado`, la del punto único
-de F1-2) dan, en la segunda ejecución, `sin-cambios` o la misma línea, sin
+Lectura: las 14 operaciones locales, más `commit-estado` (la del punto único
+de F1-2), dan, en la segunda ejecución, `sin-cambios` o la misma línea, sin
 cambiar refs, worktrees ni status. La salida es la misma con los dos locales.
+`tag-anotado` compara el commit pedido con el del tag local (aunque no esté
+publicado) y con el del remoto: en otro commit, local o remoto, devuelve 3 y
+no publica nada (v0.0.9 no llega a `origin`).
 **Matiz**: este Git for Windows no trae catálogos de traducción (B0: el
 `git status` crudo sale en inglés con los dos locales), así que la prueba no
 demuestra que el entorno fijo neutralice un Git traducido; lo da por hecho
@@ -183,18 +194,83 @@ $ taskctl guardar merge(TASK-008): develop en la rama f.txt
 padres del commit guardado: 2
 ... push ok, actualizar-rama sin-cambios, merge del PR, quitar-worktree ok, actualizar-principal ok
 f.txt en develop: linea 1 de TASK-008 y de otro PR
-control del patron de prohibidos: casa 7 de 7 lineas prohibidas y 0 de 3 permitidas (esperado 7)
-RESULTADO B2: comandos prohibidos en el registro (rebase, stash, clean, reset, push --force/--delete, branch -d/-D): 0 ; merge con el principal en develop/main: 0 ; reset de commit-estado ejecutado: 0
 ```
 
 Lectura: el conflicto deja el merge abierto en el worktree de la tarea y
 termina con 6; repetirlo no vuelve a mergear. `guardar` no deja cerrar el
 merge con otro fichero ni con marcas de conflicto, y al resolver crea el
-merge commit (2 padres) con solo las rutas dadas. En las 283 llamadas a `git`
-de las operaciones (tres fixtures) no hay `rebase`, `stash`, `clean`, `reset`,
-push forzado ni borrado de ramas, ni un `merge` con el principal en
-`develop`/`main`; el patrón del `grep` se controla con 7 líneas prohibidas
-sintéticas (las casa todas) y 3 permitidas (ninguna).
+merge commit (2 padres) con solo las rutas dadas. El registro de comandos de
+todas las fixtures se comprueba en B2b.
+
+### B1b — Caminos de rechazo, uso, reintento y dos escritores en `commit-estado`
+
+Fixture aparte (`LC_ALL=C`), cada paso dos veces como en B1:
+
+```text
+$ taskctl validar-base no-existe
+  1a: validar-base rechazada base=no-existe motivo=no-existe   (codigo 3, 55 bytes)
+$ taskctl guardar sin rutas
+  1a: guardar uso motivo=sin-rutas   (codigo 2, 29 bytes)
+      stderr: uso: guardar "<mensaje>" <rutas...>
+$ taskctl quitar-worktree TASK-007          (g.txt sin guardar)
+  1a: quitar-worktree rechazada id=TASK-007 motivo=cambios-sin-guardar   (codigo 3, 65 bytes)
+$ taskctl quitar-worktree TASK-007          (guardado, sin publicar)
+  1a: quitar-worktree rechazada id=TASK-007 motivo=sin-publicar   (codigo 3, 58 bytes)
+-- el remoto falla una vez (hook pre-receive que rechaza el primer push: fallo transitorio)
+$ taskctl push
+  1a: push ok rama=feature/task-007-login commit=48d72d1e513b intentos=2   (codigo 0, 67 bytes)
+-- otra persona publica en la misma rama de tarea; el agente guarda y publica encima
+$ taskctl push
+  1a: push rechazada rama=feature/task-007-login motivo=no-fast-forward   (codigo 3, 66 bytes)
+      stderr: lanza actualizar-rama
+-- worktree perdido con la rama local presente: crear-rama-y-worktree no la pisa
+$ taskctl crear-rama-y-worktree TASK-007 feature login origin/develop
+  1a: crear-rama-y-worktree rechazada rama=feature/task-007-login motivo=rama-existe   (codigo 3, 79 bytes)
+-- dos escritores en commit-estado: otro clon publica TASK-007 en-revision entre la construccion y el push
+$ taskctl commit-estado TASK-008 terminada
+  1a: commit-estado ok id=TASK-008 estado=terminada intentos=2   (codigo 0, 57 bytes)
+  2a: commit-estado sin-cambios id=TASK-008 estado=terminada   (codigo 0) ; estado igual tras la 2a: si
+en origin: TASK-007=en-revision (del otro escritor) ; TASK-008=terminada ; tareas/ igual a origin: si ; status de tareas/: ''
+commits en origin/taskcode/tareas: 3 (altas + otro + este) ; merges: 0
+RESULTADO B1b: commit-estado con un segundo escritor -> codigo 0 ; TASK-007=en-revision TASK-008=terminada en origin ; tareas/ al dia: si ; resets en el registro de N: 0
+RESULTADO B1b: pasos idempotentes 11 ok / 0 mal ; salidas fuera de contrato: 0
+```
+
+Lectura: los rechazos no cambian nada y repetirlos da la misma línea; el
+`push` reintenta un fallo transitorio y no reintenta un non-fast-forward (lo
+resuelve `actualizar-rama`). En `commit-estado`, el primer push se rechaza
+porque otro escritor publicó entre medias; como el commit se construyó con
+`commit-tree` sobre `origin/taskcode/tareas` y no está en ninguna rama local,
+no hay nada que descartar: la re-ejecución construye otro commit sobre el
+estado nuevo y lo publica. Quedan los dos cambios, historia lineal, `tareas/`
+al día y limpio, 0 `reset`.
+
+### B2b — Registro de comandos contra una lista permitida
+
+Cada llamada a `git` de las operaciones queda en un registro
+(`<op> [carpeta] rama=<rama> git <args>`). Un detector por **lista
+permitida** marca toda línea cuyo subcomando no esté en la lista (`add`,
+`branch`, `commit`, `commit-tree`, `describe`, `diff`, `fetch`,
+`for-each-ref`, `hash-object`, `log`, `ls-remote`, `merge`, `merge-base`,
+`pull`, `push`, `read-tree`, `rev-parse`, `show`, `show-ref`, `status`, `tag`,
+`update-index`, `worktree`, `write-tree`) o que lleve un argumento prohibido
+dentro de uno permitido (`push` con `+`, `:ref`, `--force*`, `-f`, `--mirror`,
+`--delete`; `branch -f/-d/-D/-m`; `worktree --force`; `tag -d/-f`; `merge
+--abort`; `pull` sin `--ff-only`; `merge` con el principal en
+`develop`/`main`). El detector se controla con 22 líneas prohibidas y 13
+permitidas.
+
+```text
+control del detector: marca 22 de 22 lineas prohibidas y 0 de 13 permitidas
+   (push +f, push +refs/heads/f:refs/heads/develop, push --mirror, push :f, update-ref -d,
+    branch -f, checkout -B, branch -m, worktree remove --force, reset, rebase, stash, ...)
+-- registro de comandos de las operaciones (fixtures C, ES, K, N): 396 llamadas a git
+RESULTADO B2b: detector por lista permitida: controles 22/22 prohibidas marcadas, 0/13 permitidas marcadas ; lineas del registro fuera de la lista: 0 de 396 ; resets: 0
+```
+
+Lectura: ninguna de las 396 llamadas sale de la lista; las variantes que una
+lista de cadenas no veía (H9; refspec `+`, `--mirror`, `update-ref -d`,
+`branch -f`, `checkout -B`) quedan fuera por construcción.
 
 ### B3 — Bytes: salida cruda frente a la línea de contrato
 
@@ -203,11 +279,11 @@ contrato de B1:
 
 ```text
 worktree add           crudo  154 bytes /  3 lineas ; contrato 109 bytes / 1 linea
-push                   crudo  289 bytes /  3 lineas ; contrato  67 bytes / 1 linea
-fetch                  crudo  186 bytes /  2 lineas ; contrato  16 bytes / 1 linea
+push                   crudo  288 bytes /  3 lineas ; contrato  67 bytes / 1 linea
+fetch                  crudo  185 bytes /  2 lineas ; contrato  16 bytes / 1 linea
 merge (actualizar)     crudo  105 bytes /  4 lineas ; contrato  59 bytes / 1 linea
 status / diff-tarea    crudo  191 bytes /  5 lineas ; contrato  56 bytes / 1 linea
-RESULTADO B3: 5 operaciones: crudo 925 bytes, contrato 307 bytes (67% menos)
+RESULTADO B3: 5 operaciones: crudo 923 bytes, contrato 307 bytes (67% menos)
 ```
 
 Lectura: en un repo pequeño la salida cruda ya es corta; la línea de contrato
@@ -228,13 +304,17 @@ progreso).
    cifra de D3a (tokens totales por debajo de la 0.16): esa la tienen que dar
    sobre todo otras tareas (contexto que se carga, informes que se releen,
    número de turnos). Se deja dicho para F8-2.
-2. **`reset --hard` dentro de `commit-estado`.** El plan prohíbe `reset
-   --hard` «para la capa y el agente» sin excepciones, pero el protocolo de
-   F1-2 descarta el commit propio no publicado de `taskcode/tareas` tras un
-   push rechazado, que es un reset de esa rama. Ajuste: la prohibición vale
-   para el agente y para los worktrees de código; la única excepción es ese
-   descarte dentro de `commit-estado`, sobre su propio commit sin publicar.
-   En la PoC el reset no llegó a ejecutarse (sin contención).
+2. **`commit-estado` sin `reset`: la PoC de F1-2 usaba `reset --hard`; F1-4
+   lo sustituye por `commit-tree`.** F1-2 dice «descartar el commit local» tras
+   un push rechazado y su PoC lo hacía con `reset --hard`, que el plan de
+   F1-4 prohíbe sin excepciones. Aquí `commit-estado` no crea commit local:
+   construye el árbol en un índice temporal (`GIT_INDEX_FILE`) sobre
+   `origin/taskcode/tareas`, crea el commit con `git commit-tree` y lo publica
+   con `git push origin <sha>:refs/heads/taskcode/tareas`; solo con el push
+   aceptado adelanta `tareas/` con `merge --ff-only`. Si el push se rechaza
+   no hay nada que descartar y se re-ejecuta (F1-2 D1a). PoC B1b: con un
+   segundo escritor que publica entre la construcción y el push, el primero
+   reintenta y publica sin conflicto, con 0 `reset` en el registro.
 3. **La escritura/red de la línea base es sobre todo trabajo de PoC**, no del
    flujo de tareas: los `gh api`/`gh repo` de TASK-002 y los `git init`/`clone`
    en repos de prueba. El hook de D4a tiene que bloquear la escritura y la red
@@ -254,6 +334,20 @@ progreso).
 - **Conversión bytes → tokens** (4 bytes por token).
 - **El hook de D4a**: no se ha escrito ni probado.
 - **`--json`**: decidido, no implementado en la PoC.
+- **Pendientes sin red de F1-2 D2b con `commit-tree`.** F1-2 deja, sin red,
+  un commit local «pendiente de publicar» en la rama; con la decisión de este
+  ADR ese commit no puede vivir en `taskcode/tareas` (descartarlo exigiría
+  mover la rama). Propuesta, sin probar: la pendiente se guarda como commit
+  creado con `commit-tree` fuera de las ramas, en `refs/taskcode/pendientes/NNN`
+  (con la operación anotada en el mensaje), y `tareas/` no se toca; la vista
+  local (`status`) la muestra leyendo esa ref. Con red, la siguiente
+  transacción re-ejecuta la operación anotada sobre `origin/taskcode/tareas`
+  (el commit pendiente nunca se empuja tal cual) y, publicada, quita la ref de
+  la pendiente (una ref propia, no una rama). Lo decide e implementa F2-3.
+- **Caminos sin ejercitar**: código 1 (`error` interno), 5 (`agotada`, también
+  la de `commit-estado` tras 5 rechazos seguidos), 7 (`esperando`, es del
+  puerto Plataforma) y la línea `push sinred` tras agotar los reintentos;
+  `crear-rama-y-worktree rechazada` con la rama solo en el remoto.
 
 ## Cómo se ve en GitHub
 
@@ -291,21 +385,21 @@ para esta PoC: **no verificado** en la plataforma.
    lógica de `_gitflow-common.sh`). Una operación nueva entra por ADR o por
    extensión declarada (P8).
 
-   | Operación | Comando dueño | Idempotencia (comprobada en B1/B2 salvo plataforma) |
+   | Operación | Comando dueño | Idempotencia y resultados (PoC: segunda ejecución sin cambios en B1, B1b y B2 para los caminos `ok`, `sin-cambios`, `sinred`, `rechazada`, `uso` y `conflicto` que se citan; lo marcado «no verificado», sin ejercitar) |
    |---|---|---|
    | `fetch` | todo comando que lee el remoto | 2.ª vez `sin-cambios`; sin remoto → `sinred` (4) |
-   | `validar-base` | `start`, `hotfix` | lectura; base inexistente → `rechazada` (3) |
+   | `validar-base` | `start`, `hotfix` | lectura; base inexistente → `rechazada` (3, B1b) |
    | `comprobar-id` | `start` (F2-4b) | lectura; `ls-remote --heads` + `branch --list`, patrón `*/task-NNN-*`; ocupada → 3 |
-   | `crear-rama-y-worktree` | `start` (caso b de F1-3) | ya creada → `sin-cambios`; rama existente sin worktree → 3 (lo cubre `reanudar`) |
+   | `crear-rama-y-worktree` | `start` (caso b de F1-3) | ya creada → `sin-cambios`; rama local existente sin worktree → 3, lo cubre `reanudar` (B1b); rama solo en el remoto → 3 (no verificado) |
    | `reanudar` | `start` (caso c de F1-3) | crea solo lo que falte (rama local desde `origin`, worktree) |
    | `guardar` | `guardar` | commit de rutas explícitas, sin `add -A` (RF-23); nada que guardar → `sin-cambios`; con un merge abierto, cierra el merge y rechaza si queda conflicto o marcas; informa `sin-guardar=N` si hay otros cambios |
-   | `commit-estado` | punto único de F1-2 (referenciado, no redefinido) | ya en ese estado → `sin-cambios`; re-ejecución tras rechazo |
-   | `push` | `review`, `actualizar-rama` | ya publicado → `sin-cambios`; reintento ante fallo transitorio; non-fast-forward → `rechazada`; **sin force** |
+   | `commit-estado` | punto único de F1-2 (referenciado, no redefinido) | commit con `commit-tree` sobre `origin/taskcode/tareas` (índice temporal) y `push <sha>:refs/heads/taskcode/tareas`, **sin commit local ni `reset`**; ya en ese estado → `sin-cambios`; push rechazado → re-ejecución (B1b, dos escritores); `agotada` tras 5 rechazos (no verificado) |
+   | `push` | `review`, `actualizar-rama` | ya publicado → `sin-cambios`; reintento ante fallo transitorio (B1b: `intentos=2`); non-fast-forward → `rechazada` (B1b); `sinred` tras agotar reintentos (no verificado); **sin force** |
    | `actualizar-rama` | `actualizar-rama` | merge de la base en la rama de la tarea, **sin rebase**; base ya contenida → `sin-cambios`; conflicto → `conflicto` (6), y repetir no vuelve a mergear |
-   | `quitar-worktree` | `finish` | no existe → `sin-cambios`; rechaza con cambios sin guardar o sin publicar; la rama se conserva |
+   | `quitar-worktree` | `finish` | no existe → `sin-cambios`; rechaza con cambios sin guardar o sin publicar (B1b); la rama se conserva |
    | `actualizar-principal` | `finish`, `release` | `pull --ff-only` de `develop`; al día → `sin-cambios` |
    | `localizar-principal` | todo comando lanzado desde un worktree | lectura (primera entrada de `worktree list --porcelain`, F1-3) |
-   | `tag-anotado` | `release`, `finish` de `hotfix` | mismo commit → `sin-cambios`; en otro commit → 3 |
+   | `tag-anotado` | `release`, `finish` de `hotfix` | compara el commit pedido con el del tag local (`refs/tags/<t>^{commit}`, aunque no esté publicado) y con el remoto (anotado o ligero); mismo commit → `sin-cambios`; en otro commit, local o remoto → 3 sin publicar nada (B1) |
    | `notas-release` | `release` | reescribe el mismo fichero; filtro de F1-3 |
    | `diff-tarea` | `review`; lectura del agente | lectura; la base la calcula el CLI (`merge-base` con la base de la rama); en el caso normal, resumen de una línea; el diff completo solo a petición |
    | `abrir-pr` | `review` | plataforma; devuelve el PR que haya (no verificado) |
@@ -314,9 +408,12 @@ para esta PoC: **no verificado** en la plataforma.
 
 3. **Prohibidas** para la capa y para el agente: `rebase`, `push --force` (y
    `-f`, `--force-with-lease`), `reset --hard`, `stash`, `clean`, `merge` con
-   el principal en `develop` o `main`, borrar ramas (local o remota). Única
-   excepción: el descarte del commit propio sin publicar de `taskcode/tareas`
-   dentro de `commit-estado` (F1-2; divergencia 2).
+   el principal en `develop` o `main`, borrar ramas (local o remota), mover
+   ramas a la fuerza (`branch -f`, `checkout -B`, `update-ref` sobre
+   `refs/heads/*`), push por refspec `+` o `--mirror`. **Sin excepciones**:
+   `commit-estado` publica con `commit-tree` y no necesita descartar nada
+   (divergencia 2). La PoC lo comprueba con una **lista permitida** de
+   subcomandos y argumentos, no con una lista de cadenas prohibidas (H9).
 4. **Contrato de salida** (D5a, H22):
    - Caso normal: como mucho **5 líneas** en stdout; la primera,
      `<comando> <resultado> clave=valor…`, con `resultado` ∈ `ok`,
@@ -329,16 +426,16 @@ para esta PoC: **no verificado** en la plataforma.
    - `--json` con la misma información, para CI.
 5. **Códigos de salida** (extienden los de F1-2; propuestos, los fija F2-3):
 
-   | Código | Resultado | Qué hace quien lo recibe |
-   |---|---|---|
-   | 0 | `ok` / `sin-cambios` (`Publicada` en F1-2) | sigue |
-   | 1 | `error` interno | para y avisa a la persona; no reintenta |
-   | 2 | `uso` | corrige la llamada (argumentos) |
-   | 3 | `rechazada` | no reintenta: la operación no procede (motivo en la línea) |
-   | 4 | `sinred` | si `pendiente=no`, reintenta con red; si `pendiente=sí`, sigue (F1-2) |
-   | 5 | `agotada` | reintenta más tarde; no se ha escrito nada (F1-2) |
-   | 6 | `conflicto` | resuelve los ficheros de `ficheros=` y lanza `guardar` con ellos |
-   | 7 | `esperando` | puertas pendientes: espera o atiende lo que diga la línea; no mergea |
+   | Código | Resultado | Qué hace quien lo recibe | Observado en la PoC |
+   |---|---|---|---|
+   | 0 | `ok` / `sin-cambios` (`Publicada` en F1-2) | sigue | sí (B1, B1b, B2) |
+   | 1 | `error` interno | para y avisa a la persona; no reintenta | no: propuesto |
+   | 2 | `uso` | corrige la llamada (argumentos) | sí (B1b: `guardar` sin rutas) |
+   | 3 | `rechazada` | no reintenta: la operación no procede (motivo en la línea) | sí (B1, B1b, B2) |
+   | 4 | `sinred` | si `pendiente=no`, reintenta con red; si `pendiente=sí`, sigue (F1-2) | sí (B1: `fetch`, `comprobar-id`); `pendiente=sí`, no |
+   | 5 | `agotada` | reintenta más tarde; no se ha escrito nada (F1-2) | no: propuesto (en F1-2 sí) |
+   | 6 | `conflicto` | resuelve los ficheros de `ficheros=` y lanza `guardar` con ellos | sí (B2) |
+   | 7 | `esperando` | puertas pendientes: espera o atiende lo que diga la línea; no mergea | no: es del puerto Plataforma |
 
 6. **Objetivo de tokens por tarea** (D3a, con la línea base de A; lo comprueba
    F8-2):
@@ -346,7 +443,7 @@ para esta PoC: **no verificado** en la plataforma.
    | Cifra | Línea base 0.16 (TASK-002 / 003 / 004) | Objetivo 2.0 |
    |---|---|---|
    | Llamadas `git`/`gh` de escritura o red del agente sobre el repo del proyecto | 12 / 4 / 6 | **0** |
-   | Salida visible de operaciones por tarea | `git`/`gh`: 21,7 / 24,9 / 53,5 KB (cota superior) + `taskctl`: 12,4 / 2,6 / 1,7 KB | **≤ 2 KB** (salida de comandos de fase, `guardar` y `actualizar-rama`; el ciclo de 22 operaciones de B1 sumó 1147 bytes) |
+   | Salida visible de operaciones por tarea | `git`/`gh`: 21,7 / 24,9 / 53,5 KB (cota superior) + `taskctl`: 12,4 / 2,6 / 1,7 KB | **≤ 2 KB** (salida de comandos de fase, `guardar` y `actualizar-rama`; el ciclo de 24 operaciones de B1 sumó 1296 bytes) |
    | Tokens totales por tarea | 4,65 M / 9,91 M / 11,06 M | **por debajo** de la 0.16 en tareas comparables (§9) |
 
    Las lecturas crudas (`git log`, `show`…) no se bloquean y se cuentan aparte
@@ -356,12 +453,12 @@ para esta PoC: **no verificado** en la plataforma.
 
 | Requisito | Decisión | Evidencia |
 |---|---|---|
-| RR-01 / P5 el agente no compone `git`/`gh`; idempotente, salida corta, códigos | capa con lista cerrada; contrato; códigos 0–7; hook de D4a | PoC B1 (22 pasos × 2 fixtures, todos idempotentes, 0 fuera de contrato), B2 (código 6), A (línea base); hook: no verificado |
+| RR-01 / P5 el agente no compone `git`/`gh`; idempotente, salida corta, códigos | capa con lista cerrada; contrato; códigos 0–7; hook de D4a | PoC B1 (24 pasos × 2 fixtures), B1b (11) y B2 (14), todos idempotentes y 0 fuera de contrato; códigos observados 0, 2, 3, 4 y 6 (1, 5 y 7 no), A (línea base); hook: no verificado |
 | RR-01 «se mide en tokens por tarea» | objetivo de tres cifras | PoC A (cifras por tarea) y B3 (bytes); comprobación: F8-2 |
 | RF-25 capa única con salida sin ruido | módulo con puertos `Git` y `Plataforma` | PoC B1, B3 (67 % menos bytes; 1 línea por operación) |
 | RF-23 solo lo que escribe la herramienta, nunca `add -A` | `guardar` con rutas explícitas; `commit-estado` con su fichero | PoC B2 (`guardar` rechaza otra ruta con el conflicto abierto; merge con solo `f.txt`); registro: 0 `add -A` |
 | RF-24 recuperación determinista | `reanudar`; conflicto sin `abort-merge`: se resuelve y se guarda | PoC B1 (`reanudar` tras perder worktree y rama), B2 |
-| RN-05 el estado no se pierde por Git | prohibidas `rebase`, `reset --hard`, `stash`, `clean`, force | PoC B2 (`grep` del registro: 0 prohibidos; patrón controlado) |
+| RN-05 el estado no se pierde por Git | prohibidas `rebase`, `reset --hard`, `stash`, `clean`, force | PoC B2b (registro contra la lista permitida: 0 fuera; detector controlado con líneas prohibidas y permitidas), B1b (`commit-estado` con dos escritores, 0 `reset`) |
 | RR-16 / P12 única dependencia `git` | puerto Plataforma por API HTTP, sin `gh`/`glab` | razonado; Plataforma no verificada |
 | P8 superficie mínima | D1a: operaciones no públicas salvo `guardar` y `actualizar-rama` | decisión 1 |
 | H22 ANSI en la salida | `color.ui=never`, contrato sin ANSI | PoC B1 (0 salidas con ESC) |
@@ -374,7 +471,8 @@ para esta PoC: **no verificado** en la plataforma.
   contrato y sus tests contra repos reales; verifica salida en otras versiones
   de Git y con un Git traducido.
 - **F2-3** fija los códigos 0–7 junto a los de F1-2 e implementa
-  `commit-estado` con su única excepción a la lista de prohibidas.
+  `commit-estado` con `commit-tree` (sin commit local ni `reset`) y decide
+  cómo guarda las pendientes sin red de F1-2 D2b (propuesta en «No verificado»).
 - **F2-6 / F2-9** implementan el puerto `Plataforma` en GitHub y GitLab por la
   API (con **F1-13** y **F1-5**); `abrir-pr` idempotente.
 - **F3-6, F6-2, F7-1, F7-3** componen `start`, `review`, `finish`, `release` y
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 61f15c5436be0aa7c50535710c6bb9f576fcad50..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-005/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-005/planificacion/plan-final.md                 |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-005/poc/poc-f1-4.log                            | 118 +++++++++++++++++++----
 tareas/{02-en-curso => 03-en-revision}/TASK-005/poc/poc-f1-4.sh                             | 147 ++++++++++++++++++++++++-----
 tareas/03-en-revision/TASK-005/revision/informe-revision-1.md                               |  21 +++++
 tareas/03-en-revision/TASK-005/revision/peticion-revision-1.md                              | 511 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-005/tarea.md                                    |  11 ++-
 7 files changed, 764 insertions(+), 44 deletions(-)
````
