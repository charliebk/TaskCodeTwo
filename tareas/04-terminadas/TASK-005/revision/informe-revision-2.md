# Informe de revision — TASK-005 (ronda 2)

- Commit revisado: 530d9ddd14c7646dc183e5010d8963d24fe1a058
- Revisor: code-quality-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md:286 | implementacion |
| IMP-2 | IMPORTANTE | corregido | tareas/03-en-revision/TASK-005/poc/poc-f1-4.sh:324 | implementacion |
| IMP-3 | IMPORTANTE | corregido | docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md:349 | implementacion |

Sin hallazgos nuevos: la correccion no abre ningun CRITICO ni introduce ningun fallo de los que esta ronda obliga a reportar.

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### Verificacion por hallazgo

- **IMP-1 (corregido)**: el ADR ya no tiene excepcion a `reset --hard`. Las menciones a `reset` que quedan son la alternativa descartada (linea 52), la prohibicion sin excepciones (linea 410-414), la fila de `commit-estado` (linea 396) y F2-3 (linea 474-475, sin excepcion). `commit-estado` se reproduce con `commit-tree` sobre `origin/taskcode/tareas`, indice temporal y `push <sha>:refs/heads/taskcode/tareas` sin force. PoC B1b: con un segundo escritor que publica TASK-007 entre la construccion y el push, el primer intento se rechaza, el segundo construye sobre el estado nuevo y termina con codigo 0, `TASK-007=en-revision` y `TASK-008=terminada` en origin, `tareas/` al dia y 0 `reset` en el registro. Ademas, el registro de las fixtures C, ES, K y N (396 llamadas a git) no contiene ningun `reset` (B2b).
- **IMP-2 (corregido)**: `tag-anotado` compara ahora el commit pedido con el del tag local (`refs/tags/<t>^{commit}`) y con el del remoto (linea `^{}` si hay tag anotado, linea del tag si es ligero, via awk). Reproducido en la PoC: tag remoto v0.1.0 pedido sobre `origin/develop` → `rechazada ... donde=remoto` (3); tag local v0.0.9 sin publicar pedido sobre `origin/main` → `rechazada ... donde=local` (3), y `ls-remote` de v0.0.9 en origin vacio tras la operacion (no se publico nada). El caso de mismo commit sigue dando `ok` y luego `sin-cambios`. Revisado el awk: `$2==t"^{}"` se evalua como `$2 == (t "^{}")` (la concatenacion precede a la comparacion en awk), correcto. No he probado un tag remoto ligero en otro commit de forma separada; el codigo lo cubre por la rama `l=$1`, no reproducido.
- **IMP-3 (corregido)**: la cabecera de la tabla (linea 388) dice ahora que la idempotencia se comprueba en B1, B1b y B2 solo para los caminos `ok`, `sin-cambios`, `sinred`, `rechazada`, `uso` y `conflicto`, y que lo demas esta sin ejercitar. La seccion «No verificado» (lineas 326-350) lista los caminos sin ejercitar: codigo 1, codigo 5 (`agotada`, tambien en `commit-estado`), codigo 7, `push sinred` tras agotar reintentos y `crear-rama-y-worktree rechazada` con la rama solo en el remoto. Los caminos que ahora si se ejercitan (`validar-base` inexistente, `guardar` sin rutas, `quitar-worktree` con cambios y sin publicar, `push` con fallo transitorio y non-fast-forward, `crear-rama-y-worktree` con rama local existente, `tag-anotado` en otro commit) estan en B1b y las filas del ADR lo citan. Los codigos observados en la tabla (0, 2, 3, 4, 6) coinciden con el log; 1, 5 y 7 aparecen como no observados.

### Verificacion realizada

- Commit revisado: 530d9dd (la peticion lo indica). La rama esta ahora en f106906, que solo cambia `peticion-revision-2.md`, `informe-revision-1.md` y `tarea.md`; ningun fichero de codigo, PoC ni ADR cambia entre ambos commits (`git diff --stat 530d9dd f106906`).
- `bash .github/scripts/comprobar-repo.sh` (sobre el arbol de trabajo, en la rama de la tarea): `Tareas: 93; enlaces relativos: 33; fallos: 0`.
- PoC `tareas/03-en-revision/TASK-005/poc/poc-f1-4.sh` ejecutada con carpeta de trabajo `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-005-r2` (Git local, remotos bare locales, sin red ni GitHub). Resultados obtenidos:
  - (A) 57 llamadas git/gh, 22 de escritura/red, 100134 bytes; 2421075 bytes*turnos.
  - B1: C 24 ok / 0 mal; es_ES 24 ok / 0 mal; 0 de 24 salidas distintas entre locales; 0 fuera de contrato; codigos 0:19 3:3 4:2.
  - B2: codigo 6 con conflicto; `guardar` de otra ruta con conflicto abierto → 3; con marcas → 3; 14 pasos idempotentes, 0 fuera de contrato.
  - B1b: 11 pasos idempotentes, 0 fuera de contrato; `validar-base` inexistente 3, `guardar` sin rutas 2, `quitar-worktree` con cambios y sin publicar 3 y 3, push con fallo transitorio 0 (intentos=2), push non-fast-forward 3, `crear-rama-y-worktree` con rama existente 3; commit-estado con dos escritores codigo 0, 0 resets.
  - B2b: detector por lista permitida marca 22/22 prohibidas y 0/13 permitidas; 0 de 396 lineas fuera de la lista; 0 resets.
  - B3: crudo 923 bytes, contrato 307 bytes (67% menos).
  - Comparacion de la salida con el `poc-f1-4.log` committeado: no se hizo una comparacion linea a linea; las cifras citadas en el ADR (B1 24 pasos, B1b 11, B2b 396 llamadas, B3 923/307) coinciden con la salida de esta ejecucion.
- Lectura del ADR `docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md`: comprobado que no queda la excepcion de `reset` y que la verificacion descrita coincide con lo que la PoC ejercita (ver «Verificacion por hallazgo»).
- Ejercitado a mano, fuera de la PoC: nada adicional. No se ha probado el camino `tag-anotado` con tag remoto ligero en otro commit (solo razonado sobre el awk).
- Mutaciones: no repetidas en esta ronda (la ronda 1 ya mutó B1 y el patron de prohibidos).
- Sin push, sin remotos reales. La carpeta temporal `task-005-r2` y su salida se han borrado al terminar.

## Nuevos hallazgos introducidos por la correccion

Ninguno. Revise, dentro del diff de la correccion, el camino `commit-estado` con dos escritores, la rama `exit 10` (estado ya presente: `merge --ff-only` sin comprobar su resultado, que solo deja `tareas/` atras, sin perder datos) y la construccion con `commit-tree` sobre el origin actual (los commits locales no publicados de `tareas/` quedan en su rama, no se descartan). Ninguno da perdida de datos ni corrupcion.

## Notas (fuera de la tabla, no abren ronda)

- `op_commit_estado` y `op_push` devuelven `agotada` (5) cuando el push falla por algo distinto de non-fast-forward tras un fetch correcto, en vez de `sinred` (4). Ya estaba en la ronda 1 (`exit 5` tras el push); no lo introduce esta correccion. MENOR.
- El `grep -qx "estado: $est"` de `commit-estado` asume finales de linea LF en `tarea.md`. Con CRLF en el blob no casaria y el comando no sería idempotente. No lo he probado (no verificado); la PoC fija `core.autocrlf=false` y LF. Tambien estaba en la ronda 1.

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-005/poc/poc-f1-4.sh` (diff de la ronda y funciones `op_tag_anotado`, `op_commit_estado`, `op_push`, `op_quitar_worktree`, `op_validar_base`, y la seccion B1b/B2b): es el entregable que la peticion pide reproducir.
- `tareas/03-en-revision/TASK-005/revision/peticion-revision-2.md` y `informe-revision-1.md`: la peticion de la ronda y los hallazgos a verificar, que la peticion nombra.
- `tareas/03-en-revision/TASK-005/revision/informe-revision-2.md`: plantilla de este informe.
- `.github/scripts/comprobar-repo.sh`: ejecutado porque la peticion lo pide.
- `git diff` de `poc-f1-4.sh` (ruta antigua `tareas/02-en-curso/...`): para ver solo el diff de la ronda.
