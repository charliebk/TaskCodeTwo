# Peticion de revision — TASK-004 (ronda 2)

- Tarea: TASK-004 — F1-3 ADR flujo de ramas develop/main, rama por tarea, worktrees, release
- Rama revisada: feature/task-004-f1-3-adr-flujo-de-ramas-develop-main-ram
- Rama base: develop
- Commit revisado (HEAD): 431c4009e320e268c9c89dd2267054ebf8a3092b
- Fecha: 2026-11-14
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-004 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 4d69f6ce61fe4c8de6a9cdfb96a28acb4bdde614 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (4 hallazgo(s)))

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

- tareas/03-en-revision/TASK-004/tarea.md
- tareas/03-en-revision/TASK-004/planificacion/
- docs/adr/F1-3-flujo-de-ramas-worktrees-release.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 3 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMPORTANTE-1 | IMPORTANTE | corregido | docs/adr/F1-3-flujo-de-ramas-worktrees-release.md:312 | informe-revision-1.md |

## Commits a revisar (git log 4d69f6ce61fe4c8de6a9cdfb96a28acb4bdde614..HEAD)

````
431c400 chore(TASK-004): constancia de autorrevision ronda 2
0bdc9d8 docs(adr): F1-3 corrige los hallazgos de la revision 1 (start idempotente, hotfix sin tags, ID en local, patron)
1137d41 chore(TASK-004): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 4d69f6ce61fe4c8de6a9cdfb96a28acb4bdde614..HEAD)

````diff
diff --git a/docs/adr/F1-3-flujo-de-ramas-worktrees-release.md b/docs/adr/F1-3-flujo-de-ramas-worktrees-release.md
index d0c5a06..a3138fe 100644
--- a/docs/adr/F1-3-flujo-de-ramas-worktrees-release.md
+++ b/docs/adr/F1-3-flujo-de-ramas-worktrees-release.md
@@ -103,10 +103,10 @@ bash poc-f1-3.sh "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-004/poc" > poc-f
 [start] TASK-007: rama feature/task-007-login desde origin/develop; worktree principal/.taskcode/worktrees/TASK-007
 [estado] TASK-008 -> en-curso (beto); escrito desde principal en principal/tareas
 [start] TASK-008: rama fix/task-008-crash-al-guardar desde origin/develop; worktree principal/.taskcode/worktrees/TASK-008
-principal                              be06ce0 [develop]
-principal/.taskcode/worktrees/TASK-007 be06ce0 [feature/task-007-login]
-principal/.taskcode/worktrees/TASK-008 be06ce0 [fix/task-008-crash-al-guardar]
-principal/tareas                       535be49 [taskcode/tareas]
+principal                              e0a123a [develop]
+principal/.taskcode/worktrees/TASK-007 e0a123a [feature/task-007-login]
+principal/.taskcode/worktrees/TASK-008 e0a123a [fix/task-008-crash-al-guardar]
+principal/tareas                       ef91e61 [taskcode/tareas]
 === 1b. Limites ===
 worktree add de feature/task-007-login otra vez (exit 128): fatal: 'feature/task-007-login' is already used by worktree at '.../principal/.taskcode/worktrees/TASK-007'
 worktree add de taskcode/tareas dentro de TASK-007 (exit 128): fatal: 'taskcode/tareas' is already used by worktree at '.../principal/tareas'
@@ -120,8 +120,30 @@ HEAD de TASK-007 sin cambios tras escribir estado: si
 commits de las ramas de codigo que tocan tareas/: 0
 === 1d. review: push de las ramas y unicidad del ID contra el remoto sin mirar el slug ===
 ramas del remoto con el ID 007: feature/task-007-login
-[start] TASK-007 Rechazada: el ID ya tiene rama en el remoto: feature/task-007-login
+[start] TASK-007 Rechazada: el ID ya tiene rama (remoto: feature/task-007-login; local: feature/task-007-login) y la lleva ana
 start de feature/task-007-otro-slug: codigo 3 ; existe la rama en local: 0
+=== 1e. Patron de unicidad: '*/task-007-*' no casa task-0070-* (remoto aparte) ===
+ramas en el remoto aparte: feature/task-007-login feature/task-0070-x
+ls-remote '*/task-007-*' : feature/task-007-login
+ls-remote '*/task-007*'  : feature/task-007-login feature/task-0070-x  (mutacion sin el guion final)
+ls-remote '*/task-0070-*': feature/task-0070-x
+=== 1f. start repetido: antes de review, tras review con el worktree perdido, otra persona, rama solo local ===
+[start] TASK-011: rama feature/task-011-exportar desde origin/develop; worktree principal/.taskcode/worktrees/TASK-011
+-- carla repite start de TASK-011 antes de review
+[start] TASK-011: ya la lleva carla; sin cambios (rama feature/task-011-exportar)
+codigos: 0 y 0 ; commits de estado del segundo start: 0 ; rama igual: si
+-- TASK-007 tras review: se pierden su worktree y su rama local (git clean, clon nuevo)
+worktree de TASK-007: no existe ; rama local: 0 ; rama en el remoto: feature/task-007-login
+[start] TASK-007: ya la lleva ana; rama local desde origin/feature/task-007-login, worktree (rama feature/task-007-login)
+worktree recreado: si ; HEAD igual a origin/feature/task-007-login: si ; estado en origin: en-revision (sin reescribir)
+[start] TASK-007 Rechazada: el ID ya tiene rama (remoto: feature/task-007-login; local: feature/task-007-login) y la lleva ana
+-- TASK-012: una rama del ID solo en local, sin publicar y sin asignado
+[start] TASK-012 Rechazada: el ID ya tiene rama (remoto: ninguna; local: feature/task-012-a-mano) y no la lleva nadie
+=== 1g. Hotfix con main sin tags: la base es la punta de main; una base invalida se rechaza antes de escribir estado ===
+tags: 0 ; base del hotfix: origin/main
+[start] TASK-013 Rechazada: la base 'no-existe' no existe; no se escribe estado
+commits de estado: 0 ; asignado en origin: '' ; ramas del ID: ''
+[start] TASK-013: rama hotfix/task-013-sin-tags desde origin/main; worktree principal/.taskcode/worktrees/TASK-013
 ```
 
 Lectura: cada tarea tiene su rama y su worktree en `.taskcode/worktrees/`, y
@@ -131,27 +153,39 @@ de tarea (el límite de F1-1). Desde el worktree de tarea, el CLI localiza el
 principal con la primera entrada de `git worktree list --porcelain` y escribe
 el estado allí: se publica, el worktree de tarea no tiene `tareas/` y los
 cuatro `git status` quedan limpios (el `.gitignore` de `develop` lleva
-`/tareas/` y `/.taskcode/worktrees/`). Un segundo `start` con otro slug y el
-mismo ID se rechaza con `ls-remote --heads origin '*/task-007-*'` (no casa
-`task-0070-*`, comprobado aparte) sin crear rama ni worktree.
+`/tareas/` y `/.taskcode/worktrees/`). Un segundo `start` de otra persona con
+otro slug y el mismo ID se rechaza (`ls-remote --heads origin '*/task-007-*'`
+y `git branch --list '*/task-007-*'`) sin crear rama ni worktree. La 1e
+muestra que el guion final del patrón es lo que separa `task-007-` de
+`task-0070-`: sin él, la mutación `*/task-007*` casaría las dos.
+
+La 1f prueba la regla de reanudación de `start` (decisión 2): repetir `start`
+antes de `review` da código 0 sin escribir estado ni tocar la rama; tras
+`review`, con el worktree y la rama local perdidos, el `start` del dueño
+recrea la rama local desde `origin` y el worktree (código 0, sin reescribir
+el estado `en-revision`); otra persona recibe `Rechazada` (3). Una rama del
+ID que solo existe en local (sin publicar y sin `asignado`) también bloquea
+el `start`. La 1g prueba el hotfix con `main` sin tags (base = punta de
+`origin/main`) y que una base inexistente se rechaza **antes** de publicar el
+`asignado` (0 commits de estado).
 
 ### 2 — Ciclo completo y release v0.1.0
 
 ```text
 === 2a. finish de TASK-007 y TASK-008 ===
-[pr #1] feature/task-007-login -> develop: merge 95c68da
-[pr #2] fix/task-008-crash-al-guardar -> develop: merge ad78394
+[pr #1] feature/task-007-login -> develop: merge 5fccd0f
+[pr #2] fix/task-008-crash-al-guardar -> develop: merge 3ba65a6
 [finish] TASK-007: worktree quitado; rama conservada; develop del principal al dia
 [finish] TASK-008: worktree quitado; rama conservada; develop del principal al dia
 ramas de tarea en local: feature/task-007-login fix/task-008-crash-al-guardar
 ramas de tarea en origin: feature/task-007-login fix/task-008-crash-al-guardar
 primer padre de develop (origin):
-  ad78394 95c68da 9cae59d | Merge pull request #2 from fix/task-008-crash-al-guardar
-  95c68da be06ce0 cddf762 | Merge pull request #1 from feature/task-007-login
-  be06ce0  | inicial
+  3ba65a6 5fccd0f f35334e | Merge pull request #2 from fix/task-008-crash-al-guardar
+  5fccd0f e0a123a c7800c8 | Merge pull request #1 from feature/task-007-login
+  e0a123a  | inicial
 === 2b. release v0.1.0 ===
-[pr #3] develop -> main: merge 459c64a
-[release] v0.1.0 sobre 459c64a; notas desde el inicio (git log --merges --first-parent):
+[pr #3] develop -> main: merge 3265016
+[release] v0.1.0 sobre 3265016; notas desde el inicio (git log --merges --first-parent):
   - Merge pull request #2 from fix/task-008-crash-al-guardar: TASK-008 crash al guardar
   - Merge pull request #1 from feature/task-007-login: TASK-007 login
 arbol de v0.1.0 (main): c32fba51c9e27007c24c5ee73e14bb884a50083b
@@ -175,16 +209,16 @@ hotfix no sea trivial).
 
 ```text
 === 3b. hotfix TASK-009 desde el ultimo tag de main ===
-ultimo tag de main: v0.1.0
+base del hotfix (ultimo tag de main): v0.1.0
 [start] TASK-009: rama hotfix/task-009-corrupcion-al-exportar desde v0.1.0; worktree principal/.taskcode/worktrees/TASK-009
-[pr #5] hotfix/task-009-corrupcion-al-exportar -> main: merge 2284057
-[hotfix] tag v0.1.1 sobre 2284057
-[pr #6] hotfix/task-009-corrupcion-al-exportar -> develop: merge 32a04ed
+[pr #5] hotfix/task-009-corrupcion-al-exportar -> main: merge 6522f67
+[hotfix] tag v0.1.1 sobre 6522f67
+[pr #6] hotfix/task-009-corrupcion-al-exportar -> develop: merge 200ebbe
 diferencia v0.1.0..v0.1.1:  1 file changed, 1 insertion(+), 1 deletion(-)
 v0.1.1 contiene trabajo de develop posterior a v0.1.0 (informes.txt): no
 === 3c. release v0.2.0 tras el hotfix ===
-[pr #7] develop -> main: merge 30c4708
-[release] v0.2.0 sobre 30c4708; notas desde v0.1.1 (git log --merges --first-parent):
+[pr #7] develop -> main: merge bc4f2df
+[release] v0.2.0 sobre bc4f2df; notas desde v0.1.1 (git log --merges --first-parent):
   - Merge pull request #6 from hotfix/task-009-corrupcion-al-exportar: TASK-009 hotfix (vuelta a develop)
   - Merge pull request #4 from feature/task-010-informes: TASK-010 informes
 [release] v0.2.0 notas filtradas (sin los PR cuya rama ya estaba en v0.1.1):
@@ -204,25 +238,25 @@ repiten el PR de vuelta del hotfix (ver «Divergencia con el plan»).
 ### 4 — Forma de la historia en el remoto
 
 ```text
-*   30c4708 (tag: v0.2.0, origin/main) Merge pull request #7 from develop
+*   bc4f2df (tag: v0.2.0, origin/main) Merge pull request #7 from develop
 |\
-| *   32a04ed (origin/develop) Merge pull request #6 from hotfix/task-009-corrupcion-al-exportar
+| *   200ebbe (origin/develop) Merge pull request #6 from hotfix/task-009-corrupcion-al-exportar
 | |\
-| * \   551359a Merge pull request #4 from feature/task-010-informes
+| * \   532be2e Merge pull request #4 from feature/task-010-informes
 | |\ \
-| | * | 4c3326c feat(TASK-010): informes
+| | * | 542f4ad feat(TASK-010): informes
 | |/ /
-* | |   2284057 (tag: v0.1.1) Merge pull request #5 from hotfix/task-009-corrupcion-al-exportar
+* | |   6522f67 (tag: v0.1.1) Merge pull request #5 from hotfix/task-009-corrupcion-al-exportar
 |\ \ \
 | | |/
 | |/|
-| * | b9a911c fix(TASK-009): corrupcion al exportar
+| * | 21daac4 fix(TASK-009): corrupcion al exportar
 |/ /
-* | 459c64a (tag: v0.1.0) Merge pull request #3 from develop
+* | 3265016 (tag: v0.1.0) Merge pull request #3 from develop
 |\|
-| *   ad78394 Merge pull request #2 from fix/task-008-crash-al-guardar
+| *   3ba65a6 Merge pull request #2 from fix/task-008-crash-al-guardar
 ...
-* be06ce0 inicial
+* e0a123a inicial
 primer padre de main: Merge pull request #7 from develop|Merge pull request #5 from hotfix/...|Merge pull request #3 from develop|inicial|
 pushes con --force en el guion: 0 ; remoto con receive.denyNonFastForwards=true
 ```
@@ -247,6 +281,13 @@ no tenga conflicto.
    `v0.2.0` se escriben a mano. Cómo se calcula la versión (semver desde los
    PR, o la da quien lanza la release) queda **no verificado** y abierto
    (F7-3).
+3. **Precisiones de la revisión 1** (no estaban en el plan): la primera
+   versión decía a la vez que `start` rechaza un ID con rama y que es
+   «idempotente», y la revisión mostró que el dueño se rechazaba a sí mismo
+   tras `review`; ahora `start` distingue los tres casos de la decisión 2 y
+   la PoC 1f los prueba. También se añaden la base del hotfix sin tags, la
+   validación de la base antes de escribir estado (PoC 1g), la comprobación
+   del ID en local (PoC 1f) y la del patrón (PoC 1e).
 
 El resto de lo esperado por el plan se cumplió tal cual: dos tareas en
 paralelo, límite de un worktree por rama, estado escrito en `tareas/` del
@@ -297,11 +338,15 @@ esta PoC: **no verificado** en la plataforma.
 1. **Nombres** (D1a):
    - Rama de código: `feature/task-NNN-<slug>` y `fix/task-NNN-<slug>` desde
      `origin/develop`; `hotfix/task-NNN-<slug>` desde el **último tag de
-     `main`**. El `NNN` es el del ID (`TASK-007` → `task-007`) y el slug sale
+     `main`** o, si `main` aún no tiene tags (el caso de todos los proyectos
+     medidos), desde la **punta de `origin/main`** (PoC 1g). El `NNN` es el del ID (`TASK-007` → `task-007`) y el slug sale
      del título (ASCII, minúsculas, guiones, longitud acotada: F2-2).
    - **Unicidad por el ID** (F1-2, RR-12): antes de crear la rama, `start`
-     comprueba `*/task-NNN-*` contra el remoto (y en local) sin mirar el slug
-     ni el tipo; si existe, `Rechazada` (F2-4b).
+     comprueba `*/task-NNN-*` contra el remoto (`ls-remote --heads`) y en
+     local (`git branch --list`) sin mirar el slug ni el tipo; si existe y la
+     tarea no la lleva quien lanza `start`, `Rechazada` (F2-4b; PoC 1d, 1f).
+     El guion final del patrón es necesario: sin él `task-007` casaría
+     `task-0070-*` (PoC 1e).
    - Prefijos reservados: `taskcode/*` (estado, F1-1), `lote/*` (F1-16),
      `release/*` (no se usa, D3a; reservado para una extensión).
    - Tags: `vX.Y.Z`, siempre **anotados**, solo sobre merges en `main`.
@@ -309,11 +354,11 @@ esta PoC: **no verificado** en la plataforma.
 
    | Comando | Qué hace con Git (todo desde el CLI, RR-01) |
    |---|---|
-   | `start` | `fetch`; comprueba el ID contra el remoto; publica el `asignado` por el punto único de escritura (F1-2) y, solo con él publicado, crea la rama desde `origin/develop` (o desde el último tag para `hotfix`) **sin tracking** y su worktree en `.taskcode/worktrees/TASK-NNN`. Idempotente: repetirlo recrea lo que falte desde `origin`. |
+   | `start` | `fetch`; **valida la base** (`origin/develop`, o el último tag de `main` o su punta para `hotfix`) antes de escribir nada; lee el `asignado` y las ramas del ID (remoto y local). Tres casos: **(a)** la lleva otra persona, o el ID ya tiene rama y no la lleva quien lanza → `Rechazada` (3), sin escribir; **(b)** nadie la lleva y no hay rama → publica el `asignado` por el punto único de escritura (F1-2) y, solo con él publicado, crea la rama **sin tracking** y su worktree en `.taskcode/worktrees/TASK-NNN`; **(c)** ya la lleva quien lanza → **reanuda** sin escribir estado: crea solo lo que falte (rama local desde `origin/<rama>` si existe en el remoto, o desde la base si no existe en ningún sitio; worktree) y, si no falta nada, termina con 0 «sin cambios» (PoC 1f). Lo no publicado de un worktree perdido no se recupera. |
    | `review` | push de la rama y PR contra `develop` (o contra `main` si es `hotfix`). |
    | `finish` | con las puertas en verde, merge del PR por la cola o por `taskctl merge` (API de la plataforma, nunca `git merge` local; D4a: merge commit sin fast-forward); después `git worktree remove`, `pull --ff-only` de `develop` en el principal y estado `terminada`. **La rama se conserva** en local y en el remoto. En un `hotfix`: tag de parche sobre el merge en `main` y segundo PR de la rama contra `develop`. |
    | `release vX.Y.Z` | PR `develop`→`main` con merge commit; tag anotado sobre el merge; notas = merges del primer padre de `develop` desde el tag anterior, **omitiendo los PR cuya rama ya está en ese tag** (divergencia 1). |
-   | `hotfix` | alta y `start` de una tarea de tipo `hotfix` (rama desde el último tag de `main`); el resto, por `review` y `finish`. Si necesita tarea propia o basta un carril de errata es de F1-10 (pregunta abierta). |
+   | `hotfix` | alta y `start` de una tarea de tipo `hotfix` (rama desde el último tag de `main`, o desde su punta si no hay tags); el resto, por `review` y `finish`. Si necesita tarea propia o basta un carril de errata es de F1-10 (pregunta abierta). |
 
 3. **Worktrees y estado** (D2a; concreta el límite de F1-1):
    - El **worktree principal** tiene `develop` y monta `tareas/`
@@ -342,7 +387,7 @@ esta PoC: **no verificado** en la plataforma.
    | `start-work.sh` | `start` (el `fetch` de `develop`; ya no cambia de rama el principal) |
    | `create-feature.sh` | `start` de una tarea `feature` |
    | `create-fix.sh` | `start` de una tarea `fix` |
-   | `create-hotfix.sh` | `hotfix` / `start` de una tarea `hotfix` (desde el último tag, no desde la punta de `main`) |
+   | `create-hotfix.sh` | `hotfix` / `start` de una tarea `hotfix` (desde el último tag; desde la punta de `main` solo si no hay tags) |
    | `create-release.sh` | **retirado** (D3a: sin rama de release) |
    | `update-feature.sh`, `update-fix.sh` | operación «actualizar la rama de la tarea con `develop`» de F1-4 (merge de `develop` en la rama, sin rebase: la rama ya está publicada y no se fuerza el push) |
    | `update-hotfix.sh` | la misma operación, con `main` como base |
@@ -353,7 +398,7 @@ esta PoC: **no verificado** en la plataforma.
    | `pause-work.sh`, `resume-work.sh` | **retirados**: con un worktree por tarea no hay que guardar trabajo para cambiar de rama; además `resume` mezcla trabajo de ramas (H4) |
    | `abort-merge.sh` | **retirado**: sin merges locales no hay merge a medias que abortar (RF-24) |
    | `diagnose-repo.sh` | `doctor` (F3-8) |
-   | `recover-branch.sh` | `start` idempotente (recrea rama local y worktree desde `origin`; RF-24) |
+   | `recover-branch.sh` | `start` del dueño de la tarea, que reanuda: recrea rama local y worktree desde `origin` (caso c; PoC 1f; RF-24) |
    | `mirror-to-remote.sh`, `switch-working-remote.sh`, `push-back-to-remote.sh` | **retirados** del CLI: migrar de remoto no es parte del flujo de tareas; si hace falta, extensión (P8) |
    | `_gitflow-common.sh` | **retirado**: su lógica (detección del remoto, nombres válidos) pasa a la capa de operaciones (F1-4, F2-2) |
    | `_sonda-bash.sh` | **retirado**: el CLI de la 2.0 no ejecuta scripts bash |
@@ -364,10 +409,10 @@ esta PoC: **no verificado** en la plataforma.
 |---|---|---|
 | RR-06 / RF-26 tareas a `develop` por PR; `main` solo releases y hotfix | ciclo del punto 2; merge por la plataforma | PoC 2a (merges de PR en el primer padre de `develop`), 4 (primer padre de `main`: solo releases y hotfix) |
 | RR-01 / RF-25 toda operación de Git desde el CLI | tabla de scripts: cada uno pasa a un comando o se retira | tabla de la decisión; la lista cerrada y el contrato de salida son de F1-4 (no verificado aquí) |
-| RR-12 nombres de rama únicos aunque se creen a la vez | rama derivada del ID publicado (F1-2) + comprobación `*/task-NNN-*` contra el remoto | PoC 1d (otro slug, mismo ID → `Rechazada`); la carrera entre dos `start` la cubre F1-2 (PoC 2 de F1-2) |
+| RR-12 nombres de rama únicos aunque se creen a la vez | rama derivada del ID publicado (F1-2) + comprobación `*/task-NNN-*` contra el remoto y en local | PoC 1d (otro slug, mismo ID → `Rechazada`), 1e (el patrón no casa `task-0070-*`), 1f (rama solo local → `Rechazada`); la carrera entre dos `start` la cubre F1-2 (PoC 2 de F1-2) |
 | RF-20 una rama por tarea, aislada | rama + worktree por tarea; el estado fuera de las ramas de código | PoC 1a, 1b, 1c (0 commits de código que toquen `tareas/`, `git status` limpio); el «límite de trabajo en curso» de RF-20 no lo decide este ADR (F3-6, F3-11) |
 | RF-53 release con tag y notas desde los PR | tag anotado sobre el merge; notas del primer padre, filtradas | PoC 2b, 3c (y divergencia 1) |
-| RF-24 recuperación de estados de Git | sin merges locales: fuera `abort-merge`; `recover` pasa a `start` idempotente, diagnóstico a `doctor` | tabla de scripts; razonado, no probado aquí |
+| RF-24 recuperación de estados de Git | sin merges locales: fuera `abort-merge`; `recover` pasa a la reanudación de `start` por su dueño, diagnóstico a `doctor` | PoC 1f (worktree y rama local perdidos tras `review` → recreados desde `origin`, código 0); `doctor`: razonado, no probado aquí |
 | §2 punto 5: de Git-Flow se usa la mitad | se queda `develop` + rama por tarea; release y hotfix como comandos sin rama propia de release | historia de este repo (0 tags, `main` con 1 commit); tabla de scripts |
 | H4 `resume` aplica el stash de otra rama | `pause`/`resume` retirados: un worktree por tarea | PoC 1a, 1c (dos tareas a la vez sin cambiar de rama ni `stash`) |
 | RR-07 / RF-42 nada se verifica dos veces sobre el mismo árbol | release por merge con el árbol de `develop` | PoC 2b y 3c (árbol de `main` = árbol de `develop`) |
@@ -413,7 +458,7 @@ Este ADR se planificó con un solo rol (arquitectura). Quedan sin cubrir:
 
 - **`git clean -ffdx` en el principal** borra `.taskcode/worktrees/` (son
   ignorados) igual que borra `tareas/` (hueco abierto desde F1-1); lo
-  commiteado y publicado se recupera con `start` idempotente, lo no
+  commiteado y publicado lo recupera el `start` del dueño (PoC 1f), lo no
   publicado se pierde. F2-2/F2-3 tienen que avisar o protegerse.
 - **Worktree de tarea abierto tras `liberar` o `reasignar`** (F3-10): qué
   pasa con el worktree y lo no publicado de quien la tenía.
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 4d69f6ce61fe4c8de6a9cdfb96a28acb4bdde614..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-004/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-004/planificacion/plan-final.md                 |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-004/poc/poc-f1-3.log                            | 115 +++++++++++++++-------
 tareas/{02-en-curso => 03-en-revision}/TASK-004/poc/poc-f1-3.sh                             | 105 +++++++++++++++++---
 tareas/03-en-revision/TASK-004/revision/informe-revision-1.md                               |  21 ++++
 tareas/03-en-revision/TASK-004/revision/peticion-revision-1.md                              | 520 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-004/tarea.md                                    |  11 ++-
 7 files changed, 722 insertions(+), 50 deletions(-)
````
