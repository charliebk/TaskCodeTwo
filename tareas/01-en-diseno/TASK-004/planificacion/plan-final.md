# Plan — TASK-004: F1-3 ADR flujo de ramas develop/main, rama por tarea, worktrees, release

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 55 min

De la rama abierta al cierre con merge. Senales: un ADR nuevo y una fila en el indice de `docs/adr/README.md`, sin codigo de la 2.0; un guion de PoC con remoto bare local que recorre el ciclo completo (tarea, release, hotfix, otra release), sin GitHub; 2 criterios, el primero con tres partes (nombres, ciclo, Git-Flow). Precedente: TASK-003 estimo 45 y tardo 48. Cabe en 60 min: no se trocea.

### Que se crea y que no se toca

1. **Se crea** `docs/adr/F1-3-flujo-de-ramas-worktrees-release.md` con la plantilla de `docs/adr/README.md`. «Como se ve en GitHub»: «las reglas y la cola de merge son de F1-5; aqui solo la forma del PR».
2. **Se extiende** `docs/adr/README.md` con la fila F1-3 / TASK-004.
3. **Se crean** `poc/poc-f1-3.sh` y `poc/poc-f1-3.log` con la forma de `poc-f1-2.sh` (argumento = carpeta de trabajo, secciones `===`), ejecutados en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-004\` con remoto bare local y `receive.denyNonFastForwards=true`; la carpeta se borra al terminar.
4. **No se tocan** los ADR F1-1 y F1-2, `.github/`, `.taskcode/` ni el flujo de la 0.16 de este repo (RR-14).

### Que decide el ADR

- **Nombres** (D1): `feature/task-NNN-<slug>` y `fix/task-NNN-<slug>` desde `develop`; `hotfix/task-NNN-<slug>` desde el ultimo tag de `main` (mismos prefijos que la 0.16: `create-feature.sh`, `create-hotfix.sh`). Unicidad por el ID (F1-2): F2-4b comprueba `*/task-NNN-*` contra el remoto sin mirar el slug. Prefijos reservados: `taskcode/*` (estado, F1-1), `lote/*` (F1-16), `release/*` (no se usa, D3). Tag `vX.Y.Z` anotado.
- **Ciclo completo**: `start` con el `asignado` ya publicado (F1-2), rama desde `origin/develop` y worktree en su carpeta (D2); `review`: push de la rama y PR contra `develop`; `finish`: merge del PR (D4) por la cola o `taskctl merge`, `git worktree remove`, la rama se conserva; `release vX.Y.Z`: PR `develop`→`main`, tag sobre el merge, notas desde los PR desde el tag anterior; `hotfix`: PR a `main`, tag de parche y un segundo PR automatico de la rama de hotfix contra `develop`.
- **Worktrees y estado**: concreta el limite de F1-1. El worktree principal tiene `develop` y `tareas/`; cada tarea su worktree. El CLI de un worktree de tarea localiza el principal (primera entrada de `git worktree list --porcelain`) y escribe el estado por el punto unico de F1-2. Ningun worktree de tarea monta `tareas/`.
- **Que queda de Git-Flow** (tabla en el ADR; base: REQUISITOS §2 punto 5, 0 release y 0 hotfix en los proyectos): se quedan `develop`, `main`, `feature/`, `fix/`, `hotfix/` y el merge sin fast-forward; se va `release/*` con su backmerge (`create-release.sh`, `merge-release-to-main.sh`); el backmerge queda solo para el hotfix, como PR; se retiran `pause-work.sh` y `resume-work.sh` (H4 de la auditoria: sobran con worktrees). Los scripts de `scripts/gitflow/` del plugin 0.16.1 se asignan a un comando del CLI o a «retirado»; la lista cerrada de operaciones es de F1-4.

### Orden

1. `poc-f1-3.sh`, ejecutado en la carpeta temporal guardando el log.
2. ADR con comandos, salida observada y tabla de requisitos (RR-06, RR-01, RR-12, RF-20, RF-25, RF-26, RF-53, H4; cada ID se verifica contra los documentos antes de citarlo). Incluye el resumen de `git log --graph` de este repo (el rol no pudo ejecutarlo).
3. Fila del indice.
4. Borrar la carpeta temporal tras comprobar que no queda nada que conservar.

## Lo que el rol no cubrio

Un solo rol (arquitectura). No revisado:

- **Riesgos** (criterio 1): rutas largas en Windows con worktrees anidados; `git clean -ffdx` en el principal borraria los worktrees de tarea (mismo hueco que F1-1); worktree de tarea abierto tras `liberar` o `reasignar`; hotfix cuyo PR contra `develop` choca con trabajo nuevo; release con `develop` en rojo; tags creados a la vez desde dos clones. Van como preguntas abiertas.
- **Dominio**: quien puede lanzar `release` y `hotfix` (identidad y carril, P3/P7), como se calcula la version (semver desde los PR), si un hotfix necesita tarea propia.
- **Rendimiento/escala**: worktrees vivos por clon, disco por worktree, ramas conservadas que se acumulan en el remoto.
- **Plataforma**: la PoC usa merges locales en el papel del PR; GitHub y GitLab, en F1-5, F1-13, F9-4.
- **Sin verificar por el rol**: `git log --graph` de este repo (solo leyo `.git/refs`: `main`, `develop`, tres `feature/` conservadas, ningun tag). Lo ejecuta el implementador.

## Riesgos aceptados y que los contiene

- **El ADR decide sobre la 2.0 sin cambiar este repo (RR-14)**: «Consecuencias» nombra quien ejecuta cada parte: F1-4 (operaciones), F1-5 (reglas de `main`, `develop`, `hotfix/*` y metodo de merge), F2-2 (rama, worktree, tag), F3-6 (`start`), F7-3 (`release`/`hotfix`), F1-12 (migracion).
- **Merges locales en lugar de PR**: equivalentes para la forma de la historia; el metodo exacto de la plataforma queda «no verificado» (F1-5).
- **Release sin rama `release/*`**: sin ventana de estabilizacion; con 0 usos medidos se acepta; si hace falta vuelve como extension (P8).
- **Worktrees dentro del proyecto** (D2) pueden confundir a herramientas que no respetan `.gitignore` (workspace de cargo, indexado del IDE): lo contiene el `.gitignore`; la herramienta concreta queda «no verificado» (F2-2).

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio). Todos son secciones de `poc/poc-f1-3.sh` en Git Bash con remoto bare local, sin GitHub; el ADR copia comandos y salida; la carpeta se borra al terminar. Ninguno pesado.

- **Nombres, ciclo completo y que queda de Git-Flow → 3**
  1. **Dos tareas en paralelo**: dos worktrees en `.taskcode/worktrees/`; la misma rama no se puede anadir dos veces (limite de F1-1); desde el worktree de tarea el estado se escribe en `tareas/` del principal y `git status` de las dos ramas de codigo queda limpio; `feature/task-007-otro-slug` se detecta como el mismo ID con `ls-remote`.
  2. **Ciclo completo**: merge sin fast-forward de las dos tareas en `develop`, `worktree remove`, ramas conservadas; release v0.1.0: merge `develop`→`main`, tag anotado, notas desde `git log --merges --first-parent`, arbol de `main` igual al de `develop` (la cache por arbol de P6 se reutiliza).
  3. **Hotfix y segunda release**: hotfix desde v0.1.0 a `main`, tag v0.1.1, merge de la rama de hotfix en `develop`; tarea nueva y release v0.2.0: sin conflicto, arbol de `main` igual al de `develop`, la correccion del hotfix presente en v0.2.0.
- **Cumple F1-3 y los requisitos que cita → 1**
  - Tabla requisitos-decision-evidencia y tabla de scripts de Git-Flow con su destino; al cerrar: fila del indice, `bash .github/scripts/comprobar-repo.sh` en verde y `taskctl doctor` sin errores.

## Lo que necesita decision de una persona

Decisiones respondidas por la persona (Carlos) el 2026-11-14 con la consigna «responder la opcion recomendada a todas las preguntas»:

- **D1. Nombre de la rama de codigo** → **(a)** `<tipo>/task-NNN-<slug>` con `feature`, `fix`, `hotfix`, como la 0.16. Descartadas (b) `task/NNN` y (c) solo el ID.
- **D2. Donde viven los worktrees de tarea** → **(a)** dentro del proyecto, `.taskcode/worktrees/TASK-NNN`, ignorado (que quede dentro de lo que Claude Code permite editar sin pedir permisos: no verificado, se anota en el ADR). Descartadas (b) carpeta hermana y (c) `%LOCALAPPDATA%`.
- **D3. Forma de la release** → **(a)** sin rama de release: PR `develop`→`main` con merge commit y tag en el merge. Descartadas (b) `release/*` con backmerge y (c) fast-forward de `main` (choca con P3).
- **D4. Metodo de merge en `develop` y vida de las ramas** → **(a)** merge commit sin fast-forward; al terminar se borra el worktree y la rama se conserva. Descartadas (b) squash y (c) borrar la rama remota.

Sin troceo: la duracion estimada no pasa de 60 min.
