# Informe de revision — TASK-002 (ronda 1)

- Commit revisado: 6fefde2c4f2945c2736aa409299f5a5cc4d2bc91
- Revisor: code-quality-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MENOR-1 | MENOR | abierto | docs/adr/F1-1-carpeta-fija-y-rama-de-estado.md:86 | implementacion |
| MENOR-2 | MENOR | abierto | docs/adr/F1-1-carpeta-fija-y-rama-de-estado.md:114 | plan |
| MENOR-3 | MENOR | abierto | docs/adr/F1-1-carpeta-fija-y-rama-de-estado.md:198 | implementacion |
| MENOR-4 | MENOR | aceptado | docs/adr/F1-1-carpeta-fija-y-rama-de-estado.md:169-171 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

No hay CRITICO ni IMPORTANTE. Las afirmaciones centrales del ADR estan respaldadas por evidencia reproducida.

### MENOR-1 — El descarte de A' se apoya en un comportamiento de GitHub que no se probo
- Donde: ADR, tabla de alternativas, fila A' («GitHub deja abrir un PR de ella contra `develop` y mezclar historias»).
- Que pasa: el repo temporal solo tiene la rama huerfana; lo observado es que *no* se puede abrir PR de una huerfana. Que con historia comun si se pueda es cierto en general, pero ni se probo ni aparece como «no verificado».
- Reproduccion: `gh api repos/charliebk/taskcodetwo-poc-f1-1/branches` → develop, feature/a-task-001, feature/b-task-001, taskcode/tareas; ninguna rama de estado con historia comun.
- Impacto: nulo para la decision; rompe la regla del README («lo que no se pudo comprobar se marca no verificado»).
- Sugerencia: anadir «no probado, conocimiento general de GitHub» o probarlo.

### MENOR-2 — PoC A.4 es debil: el revert que prueba no podia tocar `tareas/` en ningun caso
- Donde: `poc-f1-1.sh` (A.4), log y ADR («el revert no alcanza `tareas/`»).
- Que pasa: `tareas/` esta en `.gitignore` y es otro worktree; el resultado «6 de 6» se cumple por construccion.
- Reproduccion: guion completo, A.4 → «TASK-* en tareas/ tras el revert: 6 (eran 6)». Ademas, en el repo A, `git clean -fdx` no hizo nada (Git salta el worktree anidado), pero `git clean -ffdx` borro `tareas/` entera y `git worktree list` la marco `prunable`; lo commiteado sigue en la rama, lo no commiteado se pierde.
- Impacto: la fila RN-05 es correcta para revert, merge y rebase; `clean -ffdx` si alcanza el estado. El ADR ya lista «`tareas/` borrada a mano» como pregunta abierta.
- Sugerencia: matizar la fila RN-05 y mencionar `git clean -ffdx` entre los modos de fallo de F2-3.

### MENOR-3 — La fila RR-12 afirma «si: una rama, push inmediato» sin evidencia de frescura
- Donde: ADR, tabla «Requisitos y evidencia», fila RR-12, columna A.
- Que pasa: la evidencia solo muestra visibilidad; la frescura depende de que cada clon haga fetch (A.5: c2 solo conoce lo de c1 tras `git fetch`).
- Impacto: de redaccion; la frescura es del protocolo de F1-2.
- Sugerencia: «si (visibilidad); frescura segun el protocolo de F1-2».

### MENOR-4 — El `## Resultado` de TASK-002 que cita el ADR todavia no existe
- Donde: ADR («queda anotado en el `## Resultado` de TASK-002 para borrarlo a mano»).
- Que pasa: `tarea.md` no tiene `## Resultado`; el repo privado `charliebk/taskcodetwo-poc-f1-1` sigue existiendo.
- Impacto: si al cerrar se olvida la nota, el repo temporal queda huerfano. Esperable en esta fase.
- Sugerencia: al hacer `finish`, escribir el `## Resultado` con el borrado manual pendiente.
- Decision de quien orquesta: **aceptado**; el `## Resultado` se escribe en el cierre (`finish`) con el borrado manual del repo temporal pendiente para Carlos. No cambia ningun fichero ahora.

## Verificacion realizada

- Puerta determinista: no hay build ni suite (ADR + guion). `bash .github/scripts/comprobar-repo.sh` → «Tareas: 93; enlaces relativos: 16; fallos: 0», exit 0. `taskctl doctor` → 0 errores, 0 avisos.
- Reproduccion: `poc-f1-1.sh` con Git 2.56.0.windows.4 en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-002-r1\poc`, exit 0. Salida igual a `poc-f1-1.log` salvo rutas y hashes. Coinciden A.3 (arbol `f1e7cf4b…` identico, 1 commit en develop, 0 tocan `tareas/`, 11 en `taskcode/tareas`), A.5, B.1 (6 arboles), B.2 (conflicto en `tareas/ROADMAP.md`), B.3 (TASK-007 «NO: BORRADA») y A.2 sin merge-base.
- GitHub (solo lectura con `gh`): 5 ejecuciones, ninguna del push a `taskcode/tareas`; PR 1 → `app.txt`; PR 2 → `app.txt`, `estado-b/ROADMAP.md`, `estado-b/TASK-001/tarea.md`; compare → «404 No common ancestor»; `ROADMAP.md` en la rama de estado, size 6811.
- Honestidad: lo no verificado queda dicho (Mermaid de 400 nodos, alternativa sin `worktree --orphan`, borrado del repo temporal); modos de fallo y `05-archivadas` como preguntas abiertas con tarea asignada.
- Criterios: los tres cumplidos. Plan: D1 a D4 y orden respetados; desviacion declarada en el test 2.2 (pasa a F3-4).
- Mutacion: no aplica (sin codigo de produccion); ver MENOR-2.
- Carpeta temporal `task-002-r1` borrada; sin cambios en el repo.

## Lecturas fuera del contexto permitido

- tareas/03-en-revision/TASK-002/poc/ (guiones y log): pedido por el encargo para reproducir.
- La plantilla de este informe.
- docs/plan/PLAN-TASKCODE-2.md (RR-10..RR-12, P1, F1-1) y docs/plan/REQUISITOS-FUNCIONALES.md (RN-05): contraste pedido.
- .github/scripts/comprobar-repo.sh, `revision.md` del plugin y `taskctl doctor`.
