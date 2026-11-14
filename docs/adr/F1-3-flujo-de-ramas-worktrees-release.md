# ADR F1-3 — Flujo de ramas `develop`/`main`, rama por tarea, worktrees, release y hotfix

- Estado: **aceptada** (2026-11-14)
- Tarea: TASK-004 · Plan: [F1-3 en PLAN-TASKCODE-2](../plan/PLAN-TASKCODE-2.md) (§4 P4, P5, P6, §5 y tabla de la Fase 1)
- Requisitos: RR-06, RR-01, RR-12 (la parte de ramas), RR-07, RF-20, RF-24, RF-25, RF-26, RF-42, RF-53 y §2 punto 5 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); H4 de la [auditoría externa](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md)
- Depende de: [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) (estado en `taskcode/tareas`, montada en `tareas/`; límite de un worktree por rama) y [ADR F1-2](F1-2-escrituras-atomicas-ids-y-toma.md) (punto único de escritura, re-ejecución en vez de rebase, `start` publica el `asignado` antes de crear la rama)
- Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a del plan de TASK-004)

## Contexto

La 0.16 hereda de Git-Flow cinco tipos de rama (`feature`, `fix`, `hotfix`,
`release` y `develop`/`main`) y 22 scripts bash que las crean, actualizan,
mergean en local, pausan con `stash` y recuperan. El uso real dice que se usa
la mitad:

- **REQUISITOS §2 punto 5**: `develop` como integración y una rama por tarea,
  sí (todas las tareas); hotfix, release y tags, **no** (0 en los dos
  proyectos). La 2.0 conserva `develop` + rama por tarea y deja release y
  hotfix «como comandos sencillos».
- **RR-06 / RF-26**: cada tarea entra en `develop` por PR cuando pasa sus
  puertas; `main` solo recibe releases y hotfix; el merge lo hace el CLI o la
  cola de merge, nunca el agente con `git merge`.
- **RR-01 / RF-25**: toda operación de Git la hace un comando del CLI; los 22
  scripts tienen que pasar a ese CLI o retirarse.
- **RF-20**: una rama por tarea, aislada; en la 2.0, «rama + worktree».
- **RF-53**: release con tag y notas desde los PR (19 tags en TaskCode, 0 en
  los proyectos).
- **H4** (auditoría, ALTO): `resume-work.sh` aplica el `stash` de otra rama.
  El plan (§5) retira `pause`/`resume` porque «sobran con worktrees».
- **RR-12**: nombres de rama únicos aunque se creen a la vez.
- **F1-1** dejó un límite: una rama solo puede estar en un worktree por clon,
  así que los worktrees de tarea no pueden montar su propio `tareas/`.

Preguntas que responde: cómo se llaman las ramas, qué hace cada fase con
ellas, dónde viven los worktrees y cómo escriben el estado, cómo son la
release y el hotfix, y qué queda de Git-Flow.

### Cómo es hoy la historia de este repo

`git log --graph --oneline --all | head -60` en TaskCodeTwo (rama de TASK-004,
2026-11-14), resumido:

- `develop`: 61 commits; 43 en el primer padre, de los que **3 son merges**
  `merge(feature): feature/task-00N-... -> develop` (TASK-001, 002 y 003, sin
  fast-forward). Los otros 40 son commits directos en `develop`; de los 58
  commits sin merge de `develop`, 42 tocan solo `tareas/` (diseño, aprobación,
  cierre, imports): es el estado viajando con el código que retira F1-1.
- Las ramas `feature/task-001-...`, `-002-...`, `-003-...` siguen vivas
  (conservadas tras el merge, como decide D4a).
- `main`: **1 commit** (el inicial), ningún tag: nunca se ha hecho una release
  ni un hotfix, igual que en los proyectos medidos.
- Una vuelta atrás a mano en `develop` (`Revert "chore(TASK-001): plan
  aprobado"`), sin rama.

Es la forma que este ADR conserva para el código (merge sin fast-forward por
tarea en el primer padre de `develop`) y lo que cambia: sin commits de estado
en `develop` (F1-1) y con `main` recibiendo releases.

## Alternativas

| | Descripción | Veredicto |
|---|---|---|
| **D1a** | Rama de código `<tipo>/task-NNN-<slug>` con `tipo` ∈ `feature`, `fix`, `hotfix`, como la 0.16. | **Elegida**: legible en GitHub y en el IDE, mismos prefijos que hoy (migración sin sorpresas); la unicidad la da el ID (PoC 1d). |
| D1b | `task/NNN`. | Descartada: pierde el tipo, que decide la base (`develop` o el último tag) y el destino del PR. |
| D1c | Solo el ID (`TASK-NNN`). | Descartada: ilegible en la lista de ramas y PR; mismo problema que D1b. |
| **D2a** | Worktrees de tarea dentro del proyecto, en `.taskcode/worktrees/TASK-NNN`, ignorado. | **Elegida**: el CLI y el agente los encuentran sin configuración; se borran con el proyecto. |
| D2b | Carpeta hermana del proyecto (`../<repo>-TASK-NNN`). | Descartada: ensucia la carpeta padre y queda fuera de lo que el editor y Claude Code tienen abierto. |
| D2c | Bajo `%LOCALAPPDATA%` o `~/.cache`. | Descartada: rutas distintas por sistema y fuera de la vista de la persona. |
| **D3a** | Release sin rama `release/*`: PR `develop`→`main` con merge commit y tag anotado sobre el merge. | **Elegida**: 0 releases medidas no justifican una rama de estabilización; el árbol de `main` queda igual al de `develop` (PoC 2b, 3c). |
| D3b | `release/*` con backmerge, como Git-Flow. | Descartada: dos ramas y un backmerge para algo que no se usa; vuelve como extensión si hace falta (P8). |
| D3c | Fast-forward de `main` a `develop`. | Descartada: choca con P3 (solo por PR, con checks) y no deja un commit de release al que colgar el tag y las notas. |
| **D4a** | Merge commit sin fast-forward en `develop`; al terminar se quita el worktree y la rama se conserva. | **Elegida**: es la forma que ya tiene este repo; el primer padre de `develop` queda como lista de tareas integradas y da las notas (PoC 2a, 2b). |
| D4b | Squash merge. | Descartada: pierde la relación rama-merge que usan las notas y el filtro del hotfix (PoC 3c); la historia de la tarea desaparece. |
| D4c | Merge commit y borrar la rama remota al terminar. | Descartada: la 0.16 conserva las ramas y se mantiene; el coste en el remoto queda como pregunta abierta. |

## Pruebas de concepto

Guion `poc-f1-3.sh` y su salida completa `poc-f1-3.log`, en la carpeta
`poc/` de TASK-004 (sin enlace: la carpeta de la tarea cambia de estado al
cerrarse), ejecutados en Git Bash (Git 2.56.0 en Windows) bajo
`%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-004\poc\` (borrado al
terminar). Un remoto bare local con ruta absoluta y
`receive.denyNonFastForwards=true`, sin GitHub ni red externa; ningún push
con `--force`.

El guion simula en bash los comandos de la 2.0 (`cli_start`, `cli_estado`,
`cli_review`, `cli_finish`, `cli_release`). El papel de la plataforma (botón
de merge del PR, cola de merge, tag) lo hace un clon aparte, `plataforma`, con
`git merge --no-ff` y mensajes «Merge pull request #N from <rama>»; el
estado se escribe como en F1-2 (fetch, operación, commit y push sin force en
`taskcode/tareas`) pero sin reintentos, porque las escrituras de la PoC son
secuenciales.

```bash
bash poc-f1-3.sh "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-004/poc" > poc-f1-3.log 2>&1
```

### 1 — Dos tareas en paralelo con worktrees

```text
=== 1a. Dos tareas en paralelo: start de TASK-007 (feature) y TASK-008 (fix), cada una en su worktree ===
[estado] TASK-007 -> en-curso (ana); escrito desde principal en principal/tareas
[start] TASK-007: rama feature/task-007-login desde origin/develop; worktree principal/.taskcode/worktrees/TASK-007
[estado] TASK-008 -> en-curso (beto); escrito desde principal en principal/tareas
[start] TASK-008: rama fix/task-008-crash-al-guardar desde origin/develop; worktree principal/.taskcode/worktrees/TASK-008
principal                              be06ce0 [develop]
principal/.taskcode/worktrees/TASK-007 be06ce0 [feature/task-007-login]
principal/.taskcode/worktrees/TASK-008 be06ce0 [fix/task-008-crash-al-guardar]
principal/tareas                       535be49 [taskcode/tareas]
=== 1b. Limites ===
worktree add de feature/task-007-login otra vez (exit 128): fatal: 'feature/task-007-login' is already used by worktree at '.../principal/.taskcode/worktrees/TASK-007'
worktree add de taskcode/tareas dentro de TASK-007 (exit 128): fatal: 'taskcode/tareas' is already used by worktree at '.../principal/tareas'
=== 1c. Trabajo en cada worktree y estado escrito desde el worktree de tarea en tareas/ del principal ===
principal detectado desde TASK-007: principal
[estado] TASK-007 -> en-curso:implementando (ana); escrito desde principal/.taskcode/worktrees/TASK-007 en principal/tareas
tareas/ existe dentro del worktree de TASK-007: no
TASK-007 en tareas/ del principal: en-curso:implementando ; en origin/taskcode/tareas: en-curso:implementando
git status TASK-007: '' ; TASK-008: '' ; principal (develop): '' ; tareas/: '' (vacio = limpio)
HEAD de TASK-007 sin cambios tras escribir estado: si
commits de las ramas de codigo que tocan tareas/: 0
=== 1d. review: push de las ramas y unicidad del ID contra el remoto sin mirar el slug ===
ramas del remoto con el ID 007: feature/task-007-login
[start] TASK-007 Rechazada: el ID ya tiene rama en el remoto: feature/task-007-login
start de feature/task-007-otro-slug: codigo 3 ; existe la rama en local: 0
```

Lectura: cada tarea tiene su rama y su worktree en `.taskcode/worktrees/`, y
el principal sigue en `develop` con `tareas/` montado. Git impide por sí solo
poner la misma rama en dos worktrees y montar `taskcode/tareas` en un worktree
de tarea (el límite de F1-1). Desde el worktree de tarea, el CLI localiza el
principal con la primera entrada de `git worktree list --porcelain` y escribe
el estado allí: se publica, el worktree de tarea no tiene `tareas/` y los
cuatro `git status` quedan limpios (el `.gitignore` de `develop` lleva
`/tareas/` y `/.taskcode/worktrees/`). Un segundo `start` con otro slug y el
mismo ID se rechaza con `ls-remote --heads origin '*/task-007-*'` (no casa
`task-0070-*`, comprobado aparte) sin crear rama ni worktree.

### 2 — Ciclo completo y release v0.1.0

```text
=== 2a. finish de TASK-007 y TASK-008 ===
[pr #1] feature/task-007-login -> develop: merge 95c68da
[pr #2] fix/task-008-crash-al-guardar -> develop: merge ad78394
[finish] TASK-007: worktree quitado; rama conservada; develop del principal al dia
[finish] TASK-008: worktree quitado; rama conservada; develop del principal al dia
ramas de tarea en local: feature/task-007-login fix/task-008-crash-al-guardar
ramas de tarea en origin: feature/task-007-login fix/task-008-crash-al-guardar
primer padre de develop (origin):
  ad78394 95c68da 9cae59d | Merge pull request #2 from fix/task-008-crash-al-guardar
  95c68da be06ce0 cddf762 | Merge pull request #1 from feature/task-007-login
  be06ce0  | inicial
=== 2b. release v0.1.0 ===
[pr #3] develop -> main: merge 459c64a
[release] v0.1.0 sobre 459c64a; notas desde el inicio (git log --merges --first-parent):
  - Merge pull request #2 from fix/task-008-crash-al-guardar: TASK-008 crash al guardar
  - Merge pull request #1 from feature/task-007-login: TASK-007 login
arbol de v0.1.0 (main): c32fba51c9e27007c24c5ee73e14bb884a50083b
arbol de develop       : c32fba51c9e27007c24c5ee73e14bb884a50083b
tipo del tag: tag
```

(Los hashes de commit cambian en cada ejecución; los de árbol no. Los
bloques son del log guardado.)

Lectura: el primer padre de `develop` solo tiene merges de PR (más el commit
inicial), así que `git log --merges --first-parent` da las notas sin
convención de mensajes. El tag es anotado (`tag`) y el árbol de `main` tras la
release es **el mismo** que el de `develop`: la verificación ya hecha sobre
ese árbol vale para la release (caché por árbol de P6, RF-42).

### 3 — Hotfix v0.1.1 y release v0.2.0

Antes del hotfix, TASK-010 se integra en `develop` (para que la vuelta del
hotfix no sea trivial).

```text
=== 3b. hotfix TASK-009 desde el ultimo tag de main ===
ultimo tag de main: v0.1.0
[start] TASK-009: rama hotfix/task-009-corrupcion-al-exportar desde v0.1.0; worktree principal/.taskcode/worktrees/TASK-009
[pr #5] hotfix/task-009-corrupcion-al-exportar -> main: merge 2284057
[hotfix] tag v0.1.1 sobre 2284057
[pr #6] hotfix/task-009-corrupcion-al-exportar -> develop: merge 32a04ed
diferencia v0.1.0..v0.1.1:  1 file changed, 1 insertion(+), 1 deletion(-)
v0.1.1 contiene trabajo de develop posterior a v0.1.0 (informes.txt): no
=== 3c. release v0.2.0 tras el hotfix ===
[pr #7] develop -> main: merge 30c4708
[release] v0.2.0 sobre 30c4708; notas desde v0.1.1 (git log --merges --first-parent):
  - Merge pull request #6 from hotfix/task-009-corrupcion-al-exportar: TASK-009 hotfix (vuelta a develop)
  - Merge pull request #4 from feature/task-010-informes: TASK-010 informes
[release] v0.2.0 notas filtradas (sin los PR cuya rama ya estaba en v0.1.1):
  (omitido: Merge pull request #6 from hotfix/task-009-corrupcion-al-exportar, su rama ya esta en v0.1.1)
  - Merge pull request #4 from feature/task-010-informes: TASK-010 informes
arbol de v0.2.0 (main): 9ffa95255cd2f3336260a7a2be4b13798efbc9c2
arbol de develop       : 9ffa95255cd2f3336260a7a2be4b13798efbc9c2
v0.2.0 contiene la correccion de v0.1.1: 1 ; la de TASK-008: 1 ; v0.1.1 es antecesor de v0.2.0: si
```

Lectura: el hotfix sale del último tag de `main`, así que v0.1.1 lleva solo
la corrección y nada de lo que `develop` integró después. El PR de vuelta
contra `develop` entra sin conflicto, y la release v0.2.0 también: árbol de
`main` igual al de `develop` y la corrección presente. Las notas literales
repiten el PR de vuelta del hotfix (ver «Divergencia con el plan»).

### 4 — Forma de la historia en el remoto

```text
*   30c4708 (tag: v0.2.0, origin/main) Merge pull request #7 from develop
|\
| *   32a04ed (origin/develop) Merge pull request #6 from hotfix/task-009-corrupcion-al-exportar
| |\
| * \   551359a Merge pull request #4 from feature/task-010-informes
| |\ \
| | * | 4c3326c feat(TASK-010): informes
| |/ /
* | |   2284057 (tag: v0.1.1) Merge pull request #5 from hotfix/task-009-corrupcion-al-exportar
|\ \ \
| | |/
| |/|
| * | b9a911c fix(TASK-009): corrupcion al exportar
|/ /
* | 459c64a (tag: v0.1.0) Merge pull request #3 from develop
|\|
| *   ad78394 Merge pull request #2 from fix/task-008-crash-al-guardar
...
* be06ce0 inicial
primer padre de main: Merge pull request #7 from develop|Merge pull request #5 from hotfix/...|Merge pull request #3 from develop|inicial|
pushes con --force en el guion: 0 ; remoto con receive.denyNonFastForwards=true
```

Lectura: el primer padre de `main` es la lista de versiones (cada commit, una
release o un hotfix con su tag) y el de `develop`, la lista de tareas. El
commit del hotfix cuelga del merge de v0.1.0, así que la vuelta a `develop`
trae también ese merge: es lo esperado y lo que hace que la siguiente release
no tenga conflicto.

### Divergencia con el plan

1. **Notas de la release.** El plan dice «notas desde los PR desde el tag
   anterior» y el plan de pruebas, `git log --merges --first-parent`. Con esa
   regla literal, las notas de v0.2.0 incluyen el PR de vuelta del hotfix
   (#6), que ya se publicó en v0.1.1 (RESULTADO 3c: 1 PR repetido). Ajuste,
   dentro de D3a: las notas **omiten los merges cuyo segundo padre ya está en
   el tag anterior** (`merge-base --is-ancestor <segundo-padre> <tag>`); con
   ese filtro, 0 repetidos. Es una regla genérica (no mira el prefijo
   `hotfix/`) y depende de D4a: con squash (D4b) no habría segundo padre.
2. **Versión del parche.** El plan dice «tag de parche»; en la PoC `v0.1.1` y
   `v0.2.0` se escriben a mano. Cómo se calcula la versión (semver desde los
   PR, o la da quien lanza la release) queda **no verificado** y abierto
   (F7-3).

El resto de lo esperado por el plan se cumplió tal cual: dos tareas en
paralelo, límite de un worktree por rama, estado escrito en `tareas/` del
principal, unicidad del ID por `ls-remote`, merges sin fast-forward, ramas
conservadas, árbol de `main` igual al de `develop` en las dos releases y la
corrección del hotfix en v0.2.0.

### No verificado

- **El método de merge de GitHub y GitLab.** La PoC usa merges locales con
  `--no-ff` en el papel del botón del PR y de la cola; la forma de la
  historia es la misma, pero el merge commit de la cola de merge de GitHub
  (que, por conocimiento general, prueba sobre un commit temporal) y el de GitLab no se han probado.
  Pasa a **F1-5** (reglas y método) y **F9-4**.
- **Que Claude Code deje editar en `.taskcode/worktrees/` sin pedir
  permisos** (D2a lo da por bueno): no comprobado. Pasa a **F2-2**.
- **Herramientas que no respetan `.gitignore`** dentro del proyecto (workspace
  de cargo, indexado del IDE, `tsc` sin `exclude`) y ven los worktrees
  anidados: no comprobado. Pasa a **F2-2**.
- **Rutas largas en Windows**: la ruta del worktree en la PoC tenía unos 160
  caracteres (`C:/Users/.../pruebas-claude/taskcodetwo/task-004/poc/principal/.taskcode/worktrees/TASK-007`)
  y funcionó, pero no se probó un proyecto con rutas internas profundas.
- **Escrituras de estado concurrentes desde varios worktrees del mismo
  clon**: la PoC las hace en secuencia; el cerrojo local de F1-2 lo cubre
  sobre el papel y su test es de F2-3.
- **Cálculo de la versión** de release y hotfix (ver divergencia 2).

## Cómo se ve en GitHub

Las reglas de las ramas (`main`, `develop`, `hotfix/*`), los checks
obligatorios, la cola de merge y el método de merge permitido son de **F1-5**;
aquí solo la forma del PR:

| Operación | PR | Base ← cabeza | Tras el merge |
|---|---|---|---|
| `review` de una tarea `feature`/`fix` | uno por tarea, con la identidad del agente (F6-2) | `develop` ← `feature/task-NNN-<slug>` | merge commit en `develop`; la rama se conserva |
| `release vX.Y.Z` | uno, titulado «Release vX.Y.Z», con las notas en la descripción | `main` ← `develop` | tag anotado `vX.Y.Z` sobre el merge |
| `review` de una tarea `hotfix` | el de la corrección | `main` ← `hotfix/task-NNN-<slug>` | tag de parche sobre el merge |
| vuelta del hotfix | segundo PR, abierto por el CLI al mergear el primero | `develop` ← `hotfix/task-NNN-<slug>` | merge commit en `develop` |

Propuesta (la fija F6-2): en la lista de PR de GitHub, el título lleva el ID (`TASK-NNN ...`) y la
rama; en la lista de ramas aparecen las de tarea conservadas, `develop`,
`main` y `taskcode/tareas` (F1-1). No se ha creado un repo de GitHub para
esta PoC: **no verificado** en la plataforma.

## Decisión

1. **Nombres** (D1a):
   - Rama de código: `feature/task-NNN-<slug>` y `fix/task-NNN-<slug>` desde
     `origin/develop`; `hotfix/task-NNN-<slug>` desde el **último tag de
     `main`**. El `NNN` es el del ID (`TASK-007` → `task-007`) y el slug sale
     del título (ASCII, minúsculas, guiones, longitud acotada: F2-2).
   - **Unicidad por el ID** (F1-2, RR-12): antes de crear la rama, `start`
     comprueba `*/task-NNN-*` contra el remoto (y en local) sin mirar el slug
     ni el tipo; si existe, `Rechazada` (F2-4b).
   - Prefijos reservados: `taskcode/*` (estado, F1-1), `lote/*` (F1-16),
     `release/*` (no se usa, D3a; reservado para una extensión).
   - Tags: `vX.Y.Z`, siempre **anotados**, solo sobre merges en `main`.
2. **Ciclo completo**:

   | Comando | Qué hace con Git (todo desde el CLI, RR-01) |
   |---|---|
   | `start` | `fetch`; comprueba el ID contra el remoto; publica el `asignado` por el punto único de escritura (F1-2) y, solo con él publicado, crea la rama desde `origin/develop` (o desde el último tag para `hotfix`) **sin tracking** y su worktree en `.taskcode/worktrees/TASK-NNN`. Idempotente: repetirlo recrea lo que falte desde `origin`. |
   | `review` | push de la rama y PR contra `develop` (o contra `main` si es `hotfix`). |
   | `finish` | con las puertas en verde, merge del PR por la cola o por `taskctl merge` (API de la plataforma, nunca `git merge` local; D4a: merge commit sin fast-forward); después `git worktree remove`, `pull --ff-only` de `develop` en el principal y estado `terminada`. **La rama se conserva** en local y en el remoto. En un `hotfix`: tag de parche sobre el merge en `main` y segundo PR de la rama contra `develop`. |
   | `release vX.Y.Z` | PR `develop`→`main` con merge commit; tag anotado sobre el merge; notas = merges del primer padre de `develop` desde el tag anterior, **omitiendo los PR cuya rama ya está en ese tag** (divergencia 1). |
   | `hotfix` | alta y `start` de una tarea de tipo `hotfix` (rama desde el último tag de `main`); el resto, por `review` y `finish`. Si necesita tarea propia o basta un carril de errata es de F1-10 (pregunta abierta). |

3. **Worktrees y estado** (D2a; concreta el límite de F1-1):
   - El **worktree principal** tiene `develop` y monta `tareas/`
     (`taskcode/tareas`). Cada tarea en curso tiene su worktree en
     `.taskcode/worktrees/TASK-NNN`; el `.gitignore` de las ramas de código
     lleva `/tareas/` y `/.taskcode/worktrees/`.
   - El CLI ejecutado en un worktree de tarea localiza el principal con la
     **primera entrada de `git worktree list --porcelain`** y escribe el
     estado allí, por el punto único de escritura de F1-2 (cerrojo,
     re-ejecución, nunca rebase). Ningún worktree de tarea monta `tareas/`
     (Git lo impide: PoC 1b).
   - El principal no se cambia de rama nunca para trabajar en una tarea; solo
     se actualiza con `pull --ff-only`.
4. **Qué queda de Git-Flow** (REQUISITOS §2 punto 5): se quedan `develop`,
   `main`, `feature/`, `fix/`, `hotfix/` y el merge sin fast-forward; se van
   `release/*` y su backmerge; el backmerge queda solo para el hotfix, como
   PR; se van `pause`/`resume` con `stash` (H4) y los merges locales.
   Destino de cada script de `scripts/gitflow/` del plugin 0.16.1 (24
   ficheros `.sh`: 22 de acción, `_gitflow-common.sh` y `_sonda-bash.sh`; más
   `test/smoke-test.sh`). Los nombres son los del plan; la lista cerrada de
   operaciones es de **F1-4**:

   | Script 0.16.1 | Destino en la 2.0 |
   |---|---|
   | `create-develop.sh` | `init` (crea `develop` y `main` si faltan; F3-1) |
   | `start-work.sh` | `start` (el `fetch` de `develop`; ya no cambia de rama el principal) |
   | `create-feature.sh` | `start` de una tarea `feature` |
   | `create-fix.sh` | `start` de una tarea `fix` |
   | `create-hotfix.sh` | `hotfix` / `start` de una tarea `hotfix` (desde el último tag, no desde la punta de `main`) |
   | `create-release.sh` | **retirado** (D3a: sin rama de release) |
   | `update-feature.sh`, `update-fix.sh` | operación «actualizar la rama de la tarea con `develop`» de F1-4 (merge de `develop` en la rama, sin rebase: la rama ya está publicada y no se fuerza el push) |
   | `update-hotfix.sh` | la misma operación, con `main` como base |
   | `update-release.sh` | **retirado** (D3a) |
   | `merge-feature-to-develop.sh`, `merge-fix-to-develop.sh` | `finish` / `taskctl merge` (merge del PR por la plataforma, no local; RR-06) |
   | `merge-hotfix-to-main.sh` | `finish` de una tarea `hotfix`: PR a `main`, tag y segundo PR contra `develop` (el backmerge pasa a PR) |
   | `merge-release-to-main.sh` | `release vX.Y.Z` (PR `develop`→`main` y tag; sin rama de release) |
   | `pause-work.sh`, `resume-work.sh` | **retirados**: con un worktree por tarea no hay que guardar trabajo para cambiar de rama; además `resume` mezcla trabajo de ramas (H4) |
   | `abort-merge.sh` | **retirado**: sin merges locales no hay merge a medias que abortar (RF-24) |
   | `diagnose-repo.sh` | `doctor` (F3-8) |
   | `recover-branch.sh` | `start` idempotente (recrea rama local y worktree desde `origin`; RF-24) |
   | `mirror-to-remote.sh`, `switch-working-remote.sh`, `push-back-to-remote.sh` | **retirados** del CLI: migrar de remoto no es parte del flujo de tareas; si hace falta, extensión (P8) |
   | `_gitflow-common.sh` | **retirado**: su lógica (detección del remoto, nombres válidos) pasa a la capa de operaciones (F1-4, F2-2) |
   | `_sonda-bash.sh` | **retirado**: el CLI de la 2.0 no ejecuta scripts bash |

## Requisitos y evidencia

| Requisito | Decisión | Evidencia |
|---|---|---|
| RR-06 / RF-26 tareas a `develop` por PR; `main` solo releases y hotfix | ciclo del punto 2; merge por la plataforma | PoC 2a (merges de PR en el primer padre de `develop`), 4 (primer padre de `main`: solo releases y hotfix) |
| RR-01 / RF-25 toda operación de Git desde el CLI | tabla de scripts: cada uno pasa a un comando o se retira | tabla de la decisión; la lista cerrada y el contrato de salida son de F1-4 (no verificado aquí) |
| RR-12 nombres de rama únicos aunque se creen a la vez | rama derivada del ID publicado (F1-2) + comprobación `*/task-NNN-*` contra el remoto | PoC 1d (otro slug, mismo ID → `Rechazada`); la carrera entre dos `start` la cubre F1-2 (PoC 2 de F1-2) |
| RF-20 una rama por tarea, aislada | rama + worktree por tarea; el estado fuera de las ramas de código | PoC 1a, 1b, 1c (0 commits de código que toquen `tareas/`, `git status` limpio); el «límite de trabajo en curso» de RF-20 no lo decide este ADR (F3-6, F3-11) |
| RF-53 release con tag y notas desde los PR | tag anotado sobre el merge; notas del primer padre, filtradas | PoC 2b, 3c (y divergencia 1) |
| RF-24 recuperación de estados de Git | sin merges locales: fuera `abort-merge`; `recover` pasa a `start` idempotente, diagnóstico a `doctor` | tabla de scripts; razonado, no probado aquí |
| §2 punto 5: de Git-Flow se usa la mitad | se queda `develop` + rama por tarea; release y hotfix como comandos sin rama propia de release | historia de este repo (0 tags, `main` con 1 commit); tabla de scripts |
| H4 `resume` aplica el stash de otra rama | `pause`/`resume` retirados: un worktree por tarea | PoC 1a, 1c (dos tareas a la vez sin cambiar de rama ni `stash`) |
| RR-07 / RF-42 nada se verifica dos veces sobre el mismo árbol | release por merge con el árbol de `develop` | PoC 2b y 3c (árbol de `main` = árbol de `develop`) |

## Consecuencias

- **F1-4** fija la lista cerrada de operaciones (crear rama y worktree,
  comprobar el ID en el remoto, actualizar la rama, push, PR, merge, quitar
  worktree, tag, notas) y su salida; parte de la tabla de scripts.
- **F1-5** decide las reglas de `main`, `develop` y `hotfix/*`, el método de
  merge permitido (merge commit, D4a) y la cola; y confirma que la cola de
  merge conserva la forma de la PoC.
- **F1-10** decide si un hotfix necesita tarea propia o un carril de errata.
- **F2-2** implementa rama, worktree en `.taskcode/worktrees/`, slug y tag; y
  verifica los permisos de Claude Code y las herramientas que ven worktrees
  anidados.
- **F2-3** localiza el principal desde cualquier worktree y escribe el estado
  allí.
- **F2-4b** comprueba `*/task-NNN-*` contra el remoto.
- **F3-1** `init` crea `develop`, `main` y el `.gitignore` con
  `/tareas/` y `/.taskcode/worktrees/`.
- **F3-6** `start` con rama y worktree; **F6-2** `review` abre el PR;
  **F7-1** `finish` mergea por la cola, quita el worktree y conserva la rama.
- **F7-3** `release` y `hotfix` con tag anotado, notas filtradas y el segundo
  PR del hotfix; **F7-4** genera el CHANGELOG desde esas notas.
- **F1-12** decide cómo migran las ramas `feature/` conservadas y los
  proyectos sin `main` usada.
- **Este repo no cambia** (RR-14): sigue con la 0.16 y sus scripts de
  Git-Flow hasta el relevo.

## Fuera de alcance

- Reglas de las ramas, checks obligatorios, cola de merge: F1-5.
- Lista cerrada de operaciones y contrato de salida: F1-4.
- Identidad de quien abre, aprueba y mergea los PR: F1-6.
- Verificación y caché por árbol: F1-7, F5-2.
- Rama de lote (`lote/*`) y su PR único: F1-16.
- Equivalencias en GitLab (MR, tren de merge): F1-13.

## Preguntas abiertas

Este ADR se planificó con un solo rol (arquitectura). Quedan sin cubrir:

- **`git clean -ffdx` en el principal** borra `.taskcode/worktrees/` (son
  ignorados) igual que borra `tareas/` (hueco abierto desde F1-1); lo
  commiteado y publicado se recupera con `start` idempotente, lo no
  publicado se pierde. F2-2/F2-3 tienen que avisar o protegerse.
- **Worktree de tarea abierto tras `liberar` o `reasignar`** (F3-10): qué
  pasa con el worktree y lo no publicado de quien la tenía.
- **Hotfix cuyo PR contra `develop` choca** con trabajo nuevo: en la PoC no
  chocó (tocaban líneas distintas); quién resuelve el conflicto y en qué rama
  (¿una rama de vuelta propia?) está sin decidir.
- **Release con `develop` en rojo**: la release debería exigir el check verde
  del árbol de `develop` (F1-5, F1-7).
- **Tags creados a la vez desde dos clones** o dos releases simultáneas: el
  remoto debería rechazar un tag que ya existe (conocimiento general de Git,
  no probado aquí): F7-3, F9-4.
- **Dominio**: quién puede lanzar `release` y `hotfix` (identidad y carril,
  P3/P7), cómo se calcula la versión y si un hotfix necesita tarea propia.
- **Escala**: worktrees vivos por clon, disco por worktree (cada uno es una
  copia del árbol) y ramas conservadas que se acumulan en el remoto (D4a);
  quizá un `taskctl podar` para ramas de tareas terminadas hace más de N
  meses.
- **Plataforma**: la PoC no usa GitHub ni GitLab; F1-5, F1-13 y F9-4.
