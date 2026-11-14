# ADR F1-4 — Capa de operaciones Git/GitHub deterministas y su contrato de salida

- Estado: **aceptada** (2026-11-14)
- Tarea: TASK-005 · Plan: [F1-4 en PLAN-TASKCODE-2](../plan/PLAN-TASKCODE-2.md) (§4 P5, P8, P12, §9 y tabla de la Fase 1)
- Requisitos: RR-01, RR-16, RF-23, RF-24, RF-25 y RN-05 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); H9, H21 y H22 de la [auditoría externa](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md)
- Depende de: [ADR F1-2](F1-2-escrituras-atomicas-ids-y-toma.md) (punto único de escritura, re-ejecución en vez de rebase, códigos 0/3/4/5) y [ADR F1-3](F1-3-flujo-de-ramas-worktrees-release.md) (ramas, worktrees, ciclo por comando y tabla de scripts de Git-Flow)
- Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a, D5a del plan de TASK-005)

## Contexto

F1-3 dejó cada script de Git-Flow de la 0.16 asignado a un comando de la 2.0
o retirado, y remitió a este ADR la **lista cerrada de operaciones** y su
**salida**. El marco:

- **RR-01 / P5**: el agente no compone `git` ni `gh`. Toda operación (rama,
  worktree, commit de lo que escribe la herramienta, push, PR, merge, tag,
  rama de estado) la hace un comando del CLI, idempotente, con reintento, con
  salida corta y sin colores y con un código de salida que dice qué hacer. Se
  mide en tokens por tarea.
- **RF-25**: capa única de operaciones Git/GitHub en el CLI, con salida para el
  agente sin ruido. **RF-23**: commitear solo lo que escribe la herramienta,
  nunca `git add -A`. **RF-24**: quedan `doctor` y una recuperación
  determinista de la rama de la tarea.
- **P12 / RR-16**: el ejecutable no necesita bash ni Node; **la única
  dependencia externa es `git`**. Eso deja fuera `gh` y `glab`.
- **P8**: superficie mínima; lo avanzado, en extensiones declaradas.
- **H22**: los scripts de la 0.16 emiten ANSI aunque la salida no sea un TTY.
  **H9**: el guardián de merges de la 0.16 solo reconoce `git merge` literal y
  deja pasar 8 de 10 variantes.
- **§9 del plan**: ningún paso pide al agente escribir `git` o `gh`, y los
  tokens por tarea bajan frente a la 0.16.

Preguntas que responde: dónde vive la capa, qué operaciones hay y quién es
dueño de cada una, qué ve el agente, qué códigos de salida hay y qué objetivo
de tokens por tarea se fija, con qué línea base.

## Alternativas

| | Descripción | Veredicto |
|---|---|---|
| **D1a** | Las operaciones no son subcomandos públicos: el agente solo ve los comandos de fase (`start`, `review`, `finish`, `release`, `hotfix`, `init`, `doctor`) y dos comandos más, `guardar` y `actualizar-rama`, porque ninguna fase los cubre. | **Elegida**: superficie mínima (P8); el agente no puede encadenar operaciones sueltas en un orden no previsto. |
| D1b | Cada operación como subcomando público (`taskctl git push`, …). | Descartada: es `git` con otro nombre; el agente vuelve a componer el flujo y la salida se multiplica. |
| **D2a** | Commits de código del agente con `taskctl guardar "<mensaje>" <rutas…>`, rutas explícitas, sin `add -A`. | **Elegida** (RF-23; PoC B2). |
| D2b | El agente commitea con `git commit` y la capa solo publica. | Descartada: vuelve a poner `git` en manos del agente (RR-01) y no protege del `add -A`. |
| **D3a** | Objetivo de tokens en tres cifras: 0 llamadas `git`/`gh` de escritura o red del agente por tarea; salida visible de las operaciones por tarea bajo un tope; tokens totales por tarea por debajo de la 0.16. | **Elegida**, con las cifras de la PoC (A). |
| D3b | Una sola cifra de tokens totales por tarea. | Descartada: la línea base dice que la salida de Git es una parte pequeña del total (ver PoC A); una cifra total no dice si la capa funciona. |
| **D4a** | El flujo usa `diff-tarea` y `status`; las lecturas crudas de Git no se bloquean pero cuentan en la métrica; un hook bloquea las de escritura y red. | **Elegida**: en la línea base, 35 de 57 llamadas fueron lecturas (A). |
| D4b | Bloquear también las lecturas crudas. | Descartada: el agente investiga con `git log`/`show`; bloquearlo cuesta más turnos de los que ahorra. |
| **D5a** | Primera línea `comando resultado clave=valor`, con `--json` opcional. | **Elegida**: legible para persona y agente, filtrable con `grep`; `--json` para CI. |
| D5b | Solo JSON. | Descartada: más bytes por operación y peor para la persona en el terminal. |
| **Capa** | Módulo interno del CLI con dos puertos: `Git` (lanza el binario `git` con entorno fijo) y `Plataforma` (PR, puertas, merge por la API HTTP). | **Elegida** (decisión 1). |
| Capa con `gh`/`glab` | El puerto Plataforma lanza `gh` o `glab`, como `CLI_PLATAFORMA` en `plataforma-remota.js` de la 0.16.1. | Descartada: añade una dependencia externa por plataforma (P12, RR-16). |
| Capa con libgit2 o una librería Git | Sin binario `git`. | Descartada: worktrees, `ls-remote` con patrones y credenciales del sistema ya funcionan con `git`, que es requisito igualmente. |

## Pruebas de concepto

Guion `poc-f1-4.sh` y su salida completa `poc-f1-4.log`, en la carpeta `poc/`
de TASK-005 (sin enlace: la carpeta de la tarea cambia de estado al
cerrarse), ejecutados en Git Bash (Git 2.56.0 en Windows) bajo
`%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-005\poc\` (borrado al
terminar). La parte (B) usa remotos bare locales con ruta absoluta y
`receive.denyNonFastForwards=true`, `git -c core.autocrlf=false`, fechas de
commit fijas (hashes reproducibles), sin GitHub ni red externa.

```bash
bash poc-f1-4.sh "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-005/poc" > poc-f1-4.log 2>&1
```

### A — Línea base: Git y `gh` lanzados por los agentes en TASK-002..004

El guion lee **solo en lectura** las transcripciones de Claude Code de la
sesión que construyó TASK-002..004: los subagentes listados en la sección
`## Coste` de cada tarea (2, 3 y 4 agentes; **estaban todos**) y la sesión
principal en la ventana de cada tarea (del commit «tarea en diseño» al de
«tarea terminada», UTC). Cuenta las llamadas Bash/PowerShell cuyo comando
lanza `git` o `gh` (sin contar texto entre comillas ni cuerpos de heredoc),
las separa en lectura y escritura/red, suma los bytes de su resultado y los
turnos del asistente que quedan después de cada una en esa transcripción
(cada turno relee el resultado de la caché). Solo saca cifras y nombres de
subcomando.

```text
TASK-002: agentes 2 (faltan: ninguno) + sesion principal; turnos del asistente: 63
  llamadas con git/gh: 17 (lectura 5, escritura/red 12; solo git 1, compuestas 16)
  bytes de su resultado: 21715 (lectura 12724, escritura/red 8991; solo git 385)
  turnos restantes tras cada una: media 12.3, max 36 ; bytes x turnos restantes: 252630 (~63157 tokens releidos)
  subcomandos: gh api=9, gh pr=6, git commit=4, git status=3, git clean=3, ...
  llamadas a taskctl (sin git en la misma linea): 9, bytes 12411
TASK-003: agentes 3 (faltan: ninguno) + sesion principal; turnos del asistente: 87
  llamadas con git/gh: 14 (lectura 10, escritura/red 4; solo git 0, compuestas 14)
  bytes de su resultado: 24877 (lectura 21995, escritura/red 2882; solo git 0)
  turnos restantes tras cada una: media 10.9, max 46 ; bytes x turnos restantes: 588919 (~147229 tokens releidos)
  llamadas a taskctl (sin git en la misma linea): 3, bytes 2641
TASK-004: agentes 4 (faltan: ninguno) + sesion principal; turnos del asistente: 125
  llamadas con git/gh: 26 (lectura 20, escritura/red 6; solo git 2, compuestas 24)
  bytes de su resultado: 53542 (lectura 49073, escritura/red 4469; solo git 613)
  turnos restantes tras cada una: media 16.9, max 58 ; bytes x turnos restantes: 1579526 (~394881 tokens releidos)
  subcomandos: git log=16, git status=12, git rev-list=10, git branch=9, git commit=5, ...
  llamadas a taskctl (sin git en la misma linea): 2, bytes 1664
RESULTADO A: total 3 tareas llamadas git/gh=57 escritura/red=22 bytes=100134 bytes*turnos=2421075 taskctl=14 (bytes 16716)
```

Lectura:

- **Escritura/red**: 12, 4 y 6 llamadas por tarea. En TASK-002 son sobre todo
  `gh api`/`gh pr`/`gh repo` de su PoC contra un repo real de GitHub; en
  TASK-003 y TASK-004, `git commit`/`add`/`push` del agente (los commits del
  ADR) y `git init`/`clone` en repos de prueba.
- **Lecturas**: 35 de 57 (`log`, `status`, `rev-list`, `branch`, `show`).
- **Bytes**: 21,7 KB, 24,9 KB y 53,5 KB por tarea. Es una **cota superior**:
  casi todas las llamadas son compuestas (`git … && cat …`) y su resultado
  incluye salida que no es de Git; las llamadas solo de Git suman 0–613 bytes.
- **Peso en el total**: bytes × turnos restantes / 4 da ~63 k, ~147 k y
  ~395 k tokens releídos, frente a 4,65 M, 9,91 M y 11,06 M tokens totales de
  cada tarea (frontmatter `tokens_*`): **1,4 %, 1,5 % y 3,6 %** como mucho.
  La conversión de 4 bytes por token es una estimación (no verificada contra
  el tokenizador).
- La salida de `taskctl` de la 0.16 (que ya envuelve Git) fue de 1,7 a
  12,4 KB por tarea.

### B1 — Operaciones de la lista cerrada: locale e idempotencia

Cada operación local está simulada en bash (`op_*`) con el entorno fijo de la
decisión: `LC_ALL=C`, `GIT_TERMINAL_PROMPT=0`, `GIT_PAGER=cat`,
`-c color.ui=never`, `-c core.quotepath=false`, `-q` o `--porcelain`. Cada
paso se ejecuta **dos veces**; tras la segunda se compara una huella de refs
(clon y remoto), worktrees y `git status` de cada worktree. La misma secuencia
se repite en dos fixtures idénticas, una lanzada con `LC_ALL=C` y otra con
`LC_ALL=es_ES.UTF-8`.

```text
$ taskctl crear-rama-y-worktree TASK-007 feature login origin/develop   [en principal]
  1a: crear-rama-y-worktree ok rama=feature/task-007-login worktree=.taskcode/worktrees/TASK-007 base=33956fe8dd2e   (codigo 0, 109 bytes)
  2a: crear-rama-y-worktree sin-cambios rama=feature/task-007-login worktree=.taskcode/worktrees/TASK-007   (codigo 0) ; estado igual tras la 2a: si
$ taskctl guardar feat(TASK-007): login g.txt   [en principal/.taskcode/worktrees/TASK-007]
  1a: guardar ok commit=48d72d1e513b ficheros=1 sin-guardar=0   (codigo 0, 56 bytes)
  2a: guardar sin-cambios   (codigo 0) ; estado igual tras la 2a: si
$ taskctl comprobar-id TASK-007   [en principal]
  1a: comprobar-id ocupada id=TASK-007 ramas=feature/task-007-login   (codigo 3, 62 bytes)
$ taskctl reanudar TASK-007   [en principal]     (tras perder worktree y rama local)
  1a: reanudar ok rama=feature/task-007-login worktree=.taskcode/worktrees/TASK-007 creado=rama,worktree   (codigo 0, 99 bytes)
  2a: reanudar sin-cambios rama=feature/task-007-login worktree=.taskcode/worktrees/TASK-007   (codigo 0) ; estado igual tras la 2a: si
$ taskctl tag-anotado v0.1.0 origin/main   [en principal]
  1a: tag-anotado ok tag=v0.1.0 commit=47ea7e825b84   (codigo 0, 46 bytes)
  2a: tag-anotado sin-cambios tag=v0.1.0 commit=47ea7e825b84   (codigo 0) ; estado igual tras la 2a: si
$ taskctl fetch    [en principal]     (origin apunta a una ruta inexistente)
  1a: fetch sinred remoto=origin   (codigo 4, 27 bytes)
      stderr: fetch: no se pudo leer origin
...
RESULTADO B1: operaciones idempotentes (2a ejecucion sin cambios y sin-cambios o igual): C 22 ok / 0 mal ; es_ES 22 ok / 0 mal
RESULTADO B1: salidas distintas entre LC_ALL=C y LC_ALL=es_ES.UTF-8: 0 de 22
RESULTADO B1: salidas que incumplen el contrato (<=5 lineas, sin ANSI, 'op resultado clave=valor'): 0
RESULTADO B1: bytes de las lineas de contrato de las 22 operaciones del ciclo (1a ejecucion): 1147
RESULTADO B1: codigos de salida observados (codigo:veces): 0:19 3:1 4:2
```

Lectura: las 15 operaciones locales (más `commit-estado`, la del punto único
de F1-2) dan, en la segunda ejecución, `sin-cambios` o la misma línea, sin
cambiar refs, worktrees ni status. La salida es la misma con los dos locales.
**Matiz**: este Git for Windows no trae catálogos de traducción (B0: el
`git status` crudo sale en inglés con los dos locales), así que la prueba no
demuestra que el entorno fijo neutralice un Git traducido; lo da por hecho
el `LC_ALL=C` de cada llamada (**no verificado** con un Git traducido).

### B2 — Ciclo con conflicto provocado: código 6, sin rebase ni force

`crear-rama-y-worktree` → `guardar` → `push`; la plataforma integra en
`develop` otro PR que toca la misma línea → `fetch` → `actualizar-rama`.

```text
$ taskctl actualizar-rama    [en principal/.taskcode/worktrees/TASK-008]
  1a: actualizar-rama conflicto base=origin/develop ficheros=f.txt   (codigo 6, 61 bytes)
      stderr: resuelve f.txt y lanza guardar
  2a: actualizar-rama conflicto base=origin/develop ficheros=f.txt   (codigo 6) ; estado igual tras la 2a: si
estado del worktree: MERGE_HEAD=si ; status: UU f.txt
$ taskctl guardar otra cosa nota.txt
  1a: guardar rechazada motivo=conflicto-sin-resolver ficheros=f.txt   (codigo 3, 63 bytes)
$ taskctl guardar resolver f.txt          (f.txt aún con marcas)
  1a: guardar rechazada motivo=marcas-de-conflicto ficheros=f.txt   (codigo 3, 60 bytes)
-- el agente resuelve f.txt (lo hace el guion) y lanza guardar
$ taskctl guardar merge(TASK-008): develop en la rama f.txt
  1a: guardar ok commit=6fdf3ce3366c tipo=merge ficheros=1   (codigo 0, 53 bytes)
padres del commit guardado: 2
... push ok, actualizar-rama sin-cambios, merge del PR, quitar-worktree ok, actualizar-principal ok
f.txt en develop: linea 1 de TASK-008 y de otro PR
control del patron de prohibidos: casa 7 de 7 lineas prohibidas y 0 de 3 permitidas (esperado 7)
RESULTADO B2: comandos prohibidos en el registro (rebase, stash, clean, reset, push --force/--delete, branch -d/-D): 0 ; merge con el principal en develop/main: 0 ; reset de commit-estado ejecutado: 0
```

Lectura: el conflicto deja el merge abierto en el worktree de la tarea y
termina con 6; repetirlo no vuelve a mergear. `guardar` no deja cerrar el
merge con otro fichero ni con marcas de conflicto, y al resolver crea el
merge commit (2 padres) con solo las rutas dadas. En las 283 llamadas a `git`
de las operaciones (tres fixtures) no hay `rebase`, `stash`, `clean`, `reset`,
push forzado ni borrado de ramas, ni un `merge` con el principal en
`develop`/`main`; el patrón del `grep` se controla con 7 líneas prohibidas
sintéticas (las casa todas) y 3 permitidas (ninguna).

### B3 — Bytes: salida cruda frente a la línea de contrato

Los mismos pasos con `git` sin `-q`, en otra fixture, frente a la línea de
contrato de B1:

```text
worktree add           crudo  154 bytes /  3 lineas ; contrato 109 bytes / 1 linea
push                   crudo  289 bytes /  3 lineas ; contrato  67 bytes / 1 linea
fetch                  crudo  186 bytes /  2 lineas ; contrato  16 bytes / 1 linea
merge (actualizar)     crudo  105 bytes /  4 lineas ; contrato  59 bytes / 1 linea
status / diff-tarea    crudo  191 bytes /  5 lineas ; contrato  56 bytes / 1 linea
RESULTADO B3: 5 operaciones: crudo 925 bytes, contrato 307 bytes (67% menos)
```

Lectura: en un repo pequeño la salida cruda ya es corta; la línea de contrato
ahorra un 67 % en conjunto, poco en `worktree add` (29 %). Los bytes del
`push` crudo dependen de la longitud de la ruta del remoto. La ganancia
principal no es el tamaño de cada operación sino que el agente **no lanza**
las operaciones ni lee su salida variable (traducida, con avisos, con
progreso).

### Divergencia con el plan

1. **El peso de Git en los tokens es menor de lo que suponía el plan.** El
   plan razonaba que «cada byte que una operación muestra al agente se relee
   en cada turno siguiente; de ahí el tope de 5 líneas». La medición confirma
   el mecanismo, pero la salida de `git`/`gh` del agente fue como mucho el
   1,4–3,6 % de los tokens de cada tarea (A). El tope de 5 líneas se mantiene
   (cuesta poco y quita ruido), pero **la capa sola no basta** para la tercera
   cifra de D3a (tokens totales por debajo de la 0.16): esa la tienen que dar
   sobre todo otras tareas (contexto que se carga, informes que se releen,
   número de turnos). Se deja dicho para F8-2.
2. **`reset --hard` dentro de `commit-estado`.** El plan prohíbe `reset
   --hard` «para la capa y el agente» sin excepciones, pero el protocolo de
   F1-2 descarta el commit propio no publicado de `taskcode/tareas` tras un
   push rechazado, que es un reset de esa rama. Ajuste: la prohibición vale
   para el agente y para los worktrees de código; la única excepción es ese
   descarte dentro de `commit-estado`, sobre su propio commit sin publicar.
   En la PoC el reset no llegó a ejecutarse (sin contención).
3. **La escritura/red de la línea base es sobre todo trabajo de PoC**, no del
   flujo de tareas: los `gh api`/`gh repo` de TASK-002 y los `git init`/`clone`
   en repos de prueba. El hook de D4a tiene que bloquear la escritura y la red
   **sobre el repo del proyecto**, no en repos temporales de una PoC (pregunta
   abierta).
4. **Locale**: la prueba de salida igual con `LC_ALL=es_ES` pasa, pero de forma
   trivial en esta máquina (ver B1).

### No verificado

- **Puerto Plataforma** (`abrir-pr`, `estado-puertas`, `merge-pr`): ni la API
  de GitHub ni la de GitLab se han llamado; en la PoC un clon aparte hace de
  botón de merge. Pasa a **F1-13**, **F2-6** y **F2-9**.
- **Git con traducciones** (Linux, macOS o un Git for Windows con catálogos):
  que la salida cruda cambie con el locale y que el entorno fijo la neutralice.
- **Salida de `git` entre versiones**: solo se ha probado Git 2.56.0.
- **Conversión bytes → tokens** (4 bytes por token).
- **El hook de D4a**: no se ha escrito ni probado.
- **`--json`**: decidido, no implementado en la PoC.

## Cómo se ve en GitHub

El puerto **Plataforma** es la única parte de la capa que habla con GitHub (o
GitLab), por su API HTTP con el token de la identidad que decida F1-6/F6-1,
sin `gh`:

| Operación | Qué hace en GitHub | Idempotencia |
|---|---|---|
| `abrir-pr` | abre el PR de la rama contra su base (F1-3: `develop`, o `main` para `hotfix` y release) | si ya hay un PR abierto de esa rama, devuelve ese |
| `estado-puertas` | lee checks obligatorios y aprobaciones del PR | lectura; pendiente → `esperando` (7) |
| `merge-pr` | mergea el PR con merge commit (D4a de F1-3) o lo pone en la cola | si ya está mergeado, `sin-cambios` |

Los endpoints concretos, el método de merge permitido, la cola y su
equivalencia en GitLab son de F1-5 y F1-13. No se ha creado un repo de GitHub
para esta PoC: **no verificado** en la plataforma.

## Decisión

1. **Dónde vive la capa.** Un módulo interno del CLI con dos puertos:
   - **`Git`**: lanza el binario `git` con entorno fijo en cada llamada
     (`LC_ALL=C`, `LANG=C`, `GIT_TERMINAL_PROMPT=0`, `GIT_PAGER=cat`,
     `GIT_EDITOR=true`, `-c color.ui=never`, `-c core.quotepath=false`, `-q` o
     `--porcelain`/`--format`). Nunca interpreta texto pensado para personas;
     lee formatos estables (porcelana, `for-each-ref`, `rev-parse`, códigos de
     salida).
   - **`Plataforma`**: PR, estado de las puertas y merge por la API HTTP de la
     plataforma (sustituye a `CLI_PLATAFORMA` de la 0.16.1; P12/RR-16 dejan
     `git` como única dependencia externa). Su equivalencia en GitLab es de
     F1-13.
   - Los comandos de fase componen operaciones; **el agente solo ve la salida
     del comando**. Superficie pública (D1a): `start`, `review`, `finish`,
     `release`, `hotfix`, `init`, `doctor`, más `guardar` y `actualizar-rama`.
2. **Lista cerrada de operaciones** (hereda la tabla de scripts de F1-3 y la
   lógica de `_gitflow-common.sh`). Una operación nueva entra por ADR o por
   extensión declarada (P8).

   | Operación | Comando dueño | Idempotencia (comprobada en B1/B2 salvo plataforma) |
   |---|---|---|
   | `fetch` | todo comando que lee el remoto | 2.ª vez `sin-cambios`; sin remoto → `sinred` (4) |
   | `validar-base` | `start`, `hotfix` | lectura; base inexistente → `rechazada` (3) |
   | `comprobar-id` | `start` (F2-4b) | lectura; `ls-remote --heads` + `branch --list`, patrón `*/task-NNN-*`; ocupada → 3 |
   | `crear-rama-y-worktree` | `start` (caso b de F1-3) | ya creada → `sin-cambios`; rama existente sin worktree → 3 (lo cubre `reanudar`) |
   | `reanudar` | `start` (caso c de F1-3) | crea solo lo que falte (rama local desde `origin`, worktree) |
   | `guardar` | `guardar` | commit de rutas explícitas, sin `add -A` (RF-23); nada que guardar → `sin-cambios`; con un merge abierto, cierra el merge y rechaza si queda conflicto o marcas; informa `sin-guardar=N` si hay otros cambios |
   | `commit-estado` | punto único de F1-2 (referenciado, no redefinido) | ya en ese estado → `sin-cambios`; re-ejecución tras rechazo |
   | `push` | `review`, `actualizar-rama` | ya publicado → `sin-cambios`; reintento ante fallo transitorio; non-fast-forward → `rechazada`; **sin force** |
   | `actualizar-rama` | `actualizar-rama` | merge de la base en la rama de la tarea, **sin rebase**; base ya contenida → `sin-cambios`; conflicto → `conflicto` (6), y repetir no vuelve a mergear |
   | `quitar-worktree` | `finish` | no existe → `sin-cambios`; rechaza con cambios sin guardar o sin publicar; la rama se conserva |
   | `actualizar-principal` | `finish`, `release` | `pull --ff-only` de `develop`; al día → `sin-cambios` |
   | `localizar-principal` | todo comando lanzado desde un worktree | lectura (primera entrada de `worktree list --porcelain`, F1-3) |
   | `tag-anotado` | `release`, `finish` de `hotfix` | mismo commit → `sin-cambios`; en otro commit → 3 |
   | `notas-release` | `release` | reescribe el mismo fichero; filtro de F1-3 |
   | `diff-tarea` | `review`; lectura del agente | lectura; la base la calcula el CLI (`merge-base` con la base de la rama); en el caso normal, resumen de una línea; el diff completo solo a petición |
   | `abrir-pr` | `review` | plataforma; devuelve el PR que haya (no verificado) |
   | `estado-puertas` | `review`, `finish` | plataforma; lectura (no verificado) |
   | `merge-pr` | `finish`, `release` | plataforma; merge commit (no verificado) |

3. **Prohibidas** para la capa y para el agente: `rebase`, `push --force` (y
   `-f`, `--force-with-lease`), `reset --hard`, `stash`, `clean`, `merge` con
   el principal en `develop` o `main`, borrar ramas (local o remota). Única
   excepción: el descarte del commit propio sin publicar de `taskcode/tareas`
   dentro de `commit-estado` (F1-2; divergencia 2).
4. **Contrato de salida** (D5a, H22):
   - Caso normal: como mucho **5 líneas** en stdout; la primera,
     `<comando> <resultado> clave=valor…`, con `resultado` ∈ `ok`,
     `sin-cambios`, `rechazada`, `sinred`, `agotada`, `conflicto`,
     `esperando`, `uso`, `error`; claves en minúsculas con guiones, valores
     sin espacios; hashes a 12 caracteres; rutas relativas al worktree
     principal.
   - Sin ANSI nunca; nada de progreso ni avisos de Git en stdout.
   - Diagnóstico solo en stderr y solo si falla (una línea que dice qué hacer).
   - `--json` con la misma información, para CI.
5. **Códigos de salida** (extienden los de F1-2; propuestos, los fija F2-3):

   | Código | Resultado | Qué hace quien lo recibe |
   |---|---|---|
   | 0 | `ok` / `sin-cambios` (`Publicada` en F1-2) | sigue |
   | 1 | `error` interno | para y avisa a la persona; no reintenta |
   | 2 | `uso` | corrige la llamada (argumentos) |
   | 3 | `rechazada` | no reintenta: la operación no procede (motivo en la línea) |
   | 4 | `sinred` | si `pendiente=no`, reintenta con red; si `pendiente=sí`, sigue (F1-2) |
   | 5 | `agotada` | reintenta más tarde; no se ha escrito nada (F1-2) |
   | 6 | `conflicto` | resuelve los ficheros de `ficheros=` y lanza `guardar` con ellos |
   | 7 | `esperando` | puertas pendientes: espera o atiende lo que diga la línea; no mergea |

6. **Objetivo de tokens por tarea** (D3a, con la línea base de A; lo comprueba
   F8-2):

   | Cifra | Línea base 0.16 (TASK-002 / 003 / 004) | Objetivo 2.0 |
   |---|---|---|
   | Llamadas `git`/`gh` de escritura o red del agente sobre el repo del proyecto | 12 / 4 / 6 | **0** |
   | Salida visible de operaciones por tarea | `git`/`gh`: 21,7 / 24,9 / 53,5 KB (cota superior) + `taskctl`: 12,4 / 2,6 / 1,7 KB | **≤ 2 KB** (salida de comandos de fase, `guardar` y `actualizar-rama`; el ciclo de 22 operaciones de B1 sumó 1147 bytes) |
   | Tokens totales por tarea | 4,65 M / 9,91 M / 11,06 M | **por debajo** de la 0.16 en tareas comparables (§9) |

   Las lecturas crudas (`git log`, `show`…) no se bloquean y se cuentan aparte
   (35 de 57 llamadas en la línea base, D4a).

## Requisitos y evidencia

| Requisito | Decisión | Evidencia |
|---|---|---|
| RR-01 / P5 el agente no compone `git`/`gh`; idempotente, salida corta, códigos | capa con lista cerrada; contrato; códigos 0–7; hook de D4a | PoC B1 (22 pasos × 2 fixtures, todos idempotentes, 0 fuera de contrato), B2 (código 6), A (línea base); hook: no verificado |
| RR-01 «se mide en tokens por tarea» | objetivo de tres cifras | PoC A (cifras por tarea) y B3 (bytes); comprobación: F8-2 |
| RF-25 capa única con salida sin ruido | módulo con puertos `Git` y `Plataforma` | PoC B1, B3 (67 % menos bytes; 1 línea por operación) |
| RF-23 solo lo que escribe la herramienta, nunca `add -A` | `guardar` con rutas explícitas; `commit-estado` con su fichero | PoC B2 (`guardar` rechaza otra ruta con el conflicto abierto; merge con solo `f.txt`); registro: 0 `add -A` |
| RF-24 recuperación determinista | `reanudar`; conflicto sin `abort-merge`: se resuelve y se guarda | PoC B1 (`reanudar` tras perder worktree y rama), B2 |
| RN-05 el estado no se pierde por Git | prohibidas `rebase`, `reset --hard`, `stash`, `clean`, force | PoC B2 (`grep` del registro: 0 prohibidos; patrón controlado) |
| RR-16 / P12 única dependencia `git` | puerto Plataforma por API HTTP, sin `gh`/`glab` | razonado; Plataforma no verificada |
| P8 superficie mínima | D1a: operaciones no públicas salvo `guardar` y `actualizar-rama` | decisión 1 |
| H22 ANSI en la salida | `color.ui=never`, contrato sin ANSI | PoC B1 (0 salidas con ESC) |
| H9 guardián por texto literal | el hook de D4a no puede ser una lista negra de cadenas | pregunta abierta; no verificado |
| H21 transcripciones como formato no documentado | la línea base se midió leyéndolas; F8-2 depende de ellas | PoC A (leídas en solo lectura; las 9 de agentes estaban) |

## Consecuencias

- **F2-2** implementa el puerto `Git` y las operaciones locales con este
  contrato y sus tests contra repos reales; verifica salida en otras versiones
  de Git y con un Git traducido.
- **F2-3** fija los códigos 0–7 junto a los de F1-2 e implementa
  `commit-estado` con su única excepción a la lista de prohibidas.
- **F2-6 / F2-9** implementan el puerto `Plataforma` en GitHub y GitLab por la
  API (con **F1-13** y **F1-5**); `abrir-pr` idempotente.
- **F3-6, F6-2, F7-1, F7-3** componen `start`, `review`, `finish`, `release` y
  `hotfix` con estas operaciones; **F3-8** `doctor`.
- **F8-2** comprueba las tres cifras del objetivo con la misma medición que la
  PoC (A).
- El hook de D4a (bloquear escritura y red de `git`/`gh` del agente en el
  repo del proyecto) se diseña con la lista de prohibidas y H9.
- **Este repo no cambia** (RR-14): sigue con la 0.16.

## Fuera de alcance

- Reglas de ramas, checks obligatorios, método de merge y cola: F1-5.
- Identidad y token de la API: F1-6, F6-1.
- Equivalencias GitHub/GitLab del puerto Plataforma: F1-13.
- Quién puede lanzar `merge-pr`, `tag-anotado` y `release`: F1-6, F1-10.
- Formato del mensaje de commit de `guardar`: F1-9 o F2-2.
- Tests de cada operación contra repos reales: F2-2.

## Preguntas abiertas

Este ADR se planificó con un solo rol (arquitectura). Quedan sin cubrir:

- **Conflicto a mitad de `actualizar-rama`** si el agente abandona: el
  worktree queda con el merge abierto; `quitar-worktree` lo rechaza por
  cambios sin guardar, pero falta decidir si `doctor` ofrece abortar el merge
  (sin `reset`).
- **PR abierto dos veces a la vez** (dos `review` simultáneos): la
  idempotencia de `abrir-pr` es «devolver el que hay», pero entre leer y crear
  hay carrera; depende de lo que la API rechace (no verificado).
- **Token de la API caducado o sin permisos**: tiene que dar un código propio,
  no `sinred` (la misma pregunta que dejó F1-2 para el `fetch`).
- **`push` rechazado por reglas de la plataforma** (F1-5) y no por
  non-fast-forward: hoy cae en `sinred` tras los reintentos; necesita
  distinguirse.
- **Salida de `git` que cambia entre versiones**: la capa lee porcelana y
  formatos explícitos, pero no se ha probado más que Git 2.56.0.
- **Worktree con cambios ajenos al llamar a `guardar`**: se informa con
  `sin-guardar=N` y no se commitean; falta decidir si `review` lo bloquea.
- **Alcance del hook de D4a**: bloquear `git`/`gh` de escritura y red sobre el
  repo del proyecto sin impedir las PoC en repos temporales (divergencia 3), y
  sin caer en el problema de H9 (variantes que una lista de cadenas no ve).
- **Dominio**: quién puede lanzar `merge-pr`, `tag-anotado` y `release`;
  formato del mensaje de `guardar`.
- **Testing**: cómo prueba F2-2 cada operación contra repos reales y la
  plataforma; aquí solo se prueba el contrato.
- **Línea base medida en tareas de ADR**, no de código: puede quedarse corta
  frente a ToolsaWeb y OpenDockTrading; la recalibran F8-2 y F9-7.
