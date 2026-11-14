# Informe de revision — TASK-004 (ronda 1)

- Commit revisado: 4d69f6ce61fe4c8de6a9cdfb96a28acb4bdde614
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMPORTANTE-1 | IMPORTANTE | corregido | docs/adr/F1-3-flujo-de-ramas-worktrees-release.md:312 | implementacion |
| MENOR-1 | MENOR | corregido | docs/adr/F1-3-flujo-de-ramas-worktrees-release.md:316 | implementacion |
| MENOR-2 | MENOR | corregido | docs/adr/F1-3-flujo-de-ramas-worktrees-release.md:303 | implementacion |
| MENOR-3 | MENOR | corregido | docs/adr/F1-3-flujo-de-ramas-worktrees-release.md:136 | implementacion |

Alcance: revision completa (primera ronda), un solo rol (code-quality-reviewer; el diff es solo documentacion y un guion bash, sin dominio con revisor propio). Puerta determinista: el proyecto no tiene build, linter ni suite de tests; la puerta aplicable es `bash .github/scripts/comprobar-repo.sh` (salida: `Tareas: 93; enlaces relativos: 27; fallos: 0`, exit 0).

### IMPORTANTE-1 — `start` «idempotente» contradice la comprobacion del ID; el recover de la rama no funciona
- Donde: docs/adr/F1-3-flujo-de-ramas-worktrees-release.md:312 (tambien :356, :370 y :416, que dependen de esa afirmacion)
- Que pasa: la fila `start` pide a la vez (a) rechazar si `*/task-NNN-*` ya existe en el remoto y (b) ser «idempotente: repetirlo recrea lo que falte desde `origin`». Son incompatibles: en cuanto el dueño ha hecho `review` (rama en `origin`), repetir `start` para recrear su worktree se rechaza a si mismo. Y antes de `review` la rama solo es local y repetir `start` falla con error de Git. El ADR retira `recover-branch.sh` apoyandose en esa idempotencia (:356, :370) y la usa como mitigacion de `git clean -ffdx` (:416), sin evidencia en la PoC (la PoC solo prueba idempotencia del estado, en F1-2, no de `start`).
- Reproduccion: se cargaron las funciones de `poc-f1-3.sh` (sin las secciones de caso) en `%LOCALAPPDATA%/pruebas-claude/taskcodetwo/task-004-r1` y se ejecuto:
  1. `cli_start TASK-007 feature login origin/develop ana` -> rc 0; segundo `cli_start` identico (antes de push) -> `fatal` de git (`.../TASK-007' already exists`... rama/worktree ya existen), **rc 1**, no idempotente.
  2. `cli_review TASK-007`; `worktree remove --force` + `branch -D feature/task-007-login` (simula clon/worktree perdido); `cli_start TASK-007 feature login origin/develop ana` -> `[start] TASK-007 Rechazada: el ID ya tiene rama en el remoto: feature/task-007-login`, **rc 3**. La rama publicada del propio dueño no se puede recrear con `start`.
- Impacto: la ruta de recuperacion que sustituye a `recover-branch.sh` y a `diagnose` (RF-24) no existe tal como esta decidida; F2-2/F3-6 implementarian una contradiccion o reinventarian la decision. En el caso mas comun (perdida de worktree tras `git clean`, clon nuevo) el dueño recibe `Rechazada` por su propia rama.
- Sugerencia: separar en el ADR dos casos en `start`: ID con rama remota cuyo `asignado` es la misma persona -> reanudar (recrear rama local y worktree desde esa rama, sin error) frente a ID con rama remota y otro `asignado` (o sin `asignado`) -> `Rechazada`; decir que lo no publicado no se recupera; y anadir a la PoC (o marcar «no verificado») las dos secuencias de arriba. Alternativa: dejar la recuperacion en un comando propio y quitar «idempotente» de `start`.

### MENOR-1 — `hotfix` sin ningun tag en `main` no esta decidido y el orden deja estado huerfano
- Donde: docs/adr/F1-3-flujo-de-ramas-worktrees-release.md:316 (y la regla de base «último tag» de :299)
- Que pasa: la decision fija el hotfix «desde el último tag de `main`», pero el propio ADR documenta que los proyectos y este repo tienen 0 tags (:93-109): el primer hotfix de cualquier proyecto se encuentra sin tag. No esta ni resuelto ni en «Preguntas abiertas».
- Reproduccion: con las funciones de la PoC en un repo sin tags: `git describe --tags --abbrev=0 origin/main` -> `fatal: No names found` (rc 128); `cli_start TASK-009 hotfix x "<esa salida>" beto` imprime `[estado] TASK-009 -> en-curso (beto)` y despues `fatal: invalid reference` (rc 1): el `asignado` ya esta publicado y no hay rama ni worktree (`worktree list` solo muestra principal y tareas).
- Impacto: tarea `en-curso` sin rama tras un fallo evitable; el implementador de F7-3 no sabe que base usar.
- Sugerencia: anadir a la decision (o a preguntas abiertas) el caso sin tag (p. ej. base = punta de `main`, o rechazar antes de publicar el `asignado`) y la regla «comprobar la base antes de escribir estado».

### MENOR-2 — «y en local» de la unicidad del ID se afirma sin evidencia ni marca «no verificado»
- Donde: docs/adr/F1-3-flujo-de-ramas-worktrees-release.md:303
- Que pasa: la Decision 1 dice que `start` comprueba `*/task-NNN-*` «contra el remoto (y en local)». La PoC (`poc-f1-3.sh:66`, `ramas_del_id`) solo hace `ls-remote`; la comprobacion local no existe ni esta probada.
- Reproduccion: `cli_start TASK-008 fix a ...` (rama solo local, sin push) y luego `cli_start TASK-008 fix otro ...` -> no hay `Rechazada`; se crea/intenta crear la segunda rama (`git branch --list '*task-008*'` muestra `fix/task-008-a` y `fix/task-008-otro`). Nota: en esa prueba el guion tambien reescribio `asignado` a `carla`; la PoC no incluye el rechazo por `asignado` de F1-2 (simplificacion que el ADR declara en parte: «sin reintentos»), por lo que no es hallazgo propio.
- Impacto: dos ramas del mismo ID en local; baja probabilidad porque F1-2 protege con el `asignado`.
- Sugerencia: quitar «(y en local)» o marcarlo «no verificado» y anadir la comprobacion `git branch --list '*/task-NNN-*'` al guion.

### MENOR-3 — «comprobado aparte» no consta en el guion ni en el log
- Donde: docs/adr/F1-3-flujo-de-ramas-worktrees-release.md:136
- Que pasa: el ADR afirma que el patron `*/task-007-*` no casa `task-0070-*` («comprobado aparte»). Es cierto (lo reproduje), pero esa comprobacion no esta en `poc-f1-3.sh` ni en `poc-f1-3.log`, asi que el lector no puede reproducirla y la regla de la tarea es «evidencia, no suposicion».
- Reproduccion: tras crear `feature/task-0070-x` en el remoto: `ramas_del_id TASK-007` -> `feature/task-007-login` solamente (no incluye 0070). Mutacion del patron a `*/task-007*` -> devuelve tambien `refs/heads/feature/task-0070-x` (el patron con guion final es lo que protege). `ramas_del_id TASK-0070` -> `feature/task-0070-x`.
- Impacto: ninguno en el comportamiento decidido; solo trazabilidad de la evidencia.
- Sugerencia: anadir esas tres lineas como seccion 1e de la PoC y regenerar el log, o reformular a «comprobado en la revision r1, no en el guion».

## Verificacion realizada

- Reproduccion completa de `poc-f1-3.sh` en `%LOCALAPPDATA%/pruebas-claude/taskcodetwo/task-004-r1/run` (Git 2.56.0.windows.4): exit 0. Comparada con `poc-f1-3.log` normalizando hashes y rutas: unica diferencia, la ruta de trabajo; todos los bloques citados por el ADR (1a-1d, 2a-2b, 3b-3c, 4, resumen RESULTADO) coinciden, incluidos los hashes de arbol (los de commit varian, como dice el ADR). Notas literales con 1 PR repetido y filtradas con 0, confirmado.
- Casos que el guion no cubre (ver hallazgos): `start` repetido antes y despues de `review`; hotfix sin tag; rama local sin publicar con otro slug; patron `task-0070` y mutacion del patron a `*/task-007*`; release inicial con `notas_filtradas` y sin tag previo (funciona, sin omitidos).
- Historia del repo citada en el ADR contrastada con `git rev-list`: develop 61 commits, 43 en primer padre, 3 merges, 58 sin merge, 42 solo `tareas/`, 40 commits directos en el primer padre; `main` 1 commit, 0 tags, un `Revert "chore(TASK-001): plan aprobado"`. Coincide.
- IDs citados: RR-01, RR-06, RR-07, RR-12, RF-20, RF-24, RF-25, RF-26, RF-42, RF-53 existen en `docs/plan/REQUISITOS-FUNCIONALES.md`; H4 en `docs/auditoria/AUDITORIA-EXTERNA-2026-11-13.md:24`; §2 punto 5 existe y dice lo que el ADR cita. Referencias de fase (F1-4, F1-5, F1-10, F1-12, F1-16, F2-2, F2-3, F2-4b, F3-1, F3-6, F3-8, F3-10, F3-11, F5-2, F6-2, F7-1, F7-3, F7-4, F9-4) existen en `docs/plan/PLAN-TASKCODE-2.md`; F1-7, F1-6, F1-13 tambien. «19 tags en TaskCode, 0 en los proyectos» coincide con RF-53.
- Tabla de scripts frente a `C:\Users\nullcad2025\.claude\plugins\cache\taskcode-marketplace\taskcode-plugin\0.16.1\scripts\gitflow\`: 24 `.sh` (22 de accion + `_gitflow-common.sh` + `_sonda-bash.sh`) y `test/smoke-test.sh`; los 22 scripts de accion estan todos en la tabla, ninguno inventado ni omitido.
- Coherencia con F1-1 (limite de un worktree por rama, `tareas/` en `.gitignore`, concreta F1-3/F2-3) y F1-2 (`start` publica el `asignado` antes de crear rama, re-ejecucion en lugar de rebase): consistente, salvo la tension descrita en IMPORTANTE-1. El ADR deja `README.md` con su fila F1-3 / TASK-004 / aceptada, como las anteriores.
- Plan aprobado: cumple los cuatro ejes de D1a-D4a, la PoC tiene la forma prevista y la unica divergencia (filtro de notas de release; version del parche no verificada) esta documentada en «Divergencia con el plan» con evidencia (RESULTADO 3c). Criterios de `tarea.md` (nombres, ciclo completo, que queda de Git-Flow; cumple los requisitos que cita): cubiertos, con la reserva de IMPORTANTE-1 sobre el ciclo de `start`.
- Mutacion del filtro de notas: sin filtro (regla literal), v0.2.0 repite el PR #6 del hotfix; con filtro, 0; el guion ya cubre ambos lados (`RESULTADO 3c` ultima linea).
- `bash .github/scripts/comprobar-repo.sh` -> exit 0, fallos: 0. No se hizo push ni se consulto ningun remoto real (solo remotos bare locales de la PoC). Carpeta temporal `task-004-r1` borrada.

## Lecturas fuera del contexto permitido

- tareas/03-en-revision/TASK-004/poc/poc-f1-3.sh y poc-f1-3.log: son el entregable cuya evidencia se reproduce (pedido en el prompt de la revision).
- docs/plan/REQUISITOS-FUNCIONALES.md y docs/plan/PLAN-TASKCODE-2.md: para comprobar que los IDs de requisito y de fase citados existen y dicen lo citado.
- docs/auditoria/AUDITORIA-EXTERNA-2026-11-13.md: para comprobar H4.
- docs/adr/F1-1-carpeta-fija-y-rama-de-estado.md y docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md: para comprobar la coherencia con los ADR aceptados.
- C:\Users\nullcad2025\.claude\plugins\cache\taskcode-marketplace\taskcode-plugin\0.16.1\scripts\gitflow\ (listado) y skills\task-workflow\revision.md: para contrastar la tabla de scripts y las reglas del informe.
- .github/scripts/comprobar-repo.sh (ejecutado): puerta determinista pedida en el prompt.
- `git log`/`git rev-list` del repo: para contrastar las cifras de la historia citadas por el ADR.
