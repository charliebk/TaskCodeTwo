# Informe de revision — TASK-004 (ronda 2)

- Commit revisado: 431c4009e320e268c9c89dd2267054ebf8a3092b
- Revisor: code-quality-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMPORTANTE-1 | IMPORTANTE | corregido | docs/adr/F1-3-flujo-de-ramas-worktrees-release.md:357 | implementacion |

Hallazgos nuevos: sin hallazgos. Ni CRITICO ni IMPORTANTE nuevos: la correccion no abre ningun fallo que se pueda construir, y lo que se ha visto fuera de su alcance va en las notas de abajo.

### Verificacion de IMPORTANTE-1 (ronda 1): corregido, verificado

Hallazgo original: la fila `start` del ADR pedia a la vez rechazar un ID con rama y ser «idempotente». En la ronda 1 se reprodujo que (1) un segundo `start` antes de `push` fallaba con `fatal` de Git (rc 1) y (2) tras `review`, con el worktree y la rama local perdidos, el `start` del dueño se rechazaba a si mismo (rc 3).

Con el commit revisado (fila `start` en la linea 357; en la ronda 1 estaba en la 312, que ahora es otro texto):

- Segundo `start` del dueño antes de `review`: rc 0, «sin cambios», 0 commits de estado, rama igual. Lo ejercita la PoC 1f (carla repite `start`) y el mismo caso con el dueño en el escenario extra E1.
- `start` del dueño tras `review` con worktree y rama local perdidos: rc 0, rama local y worktree recreados desde `origin/feature/task-007-login`, HEAD igual al remoto, estado `en-revision` sin reescribir. PoC 1f.
- Otra persona sobre la misma tarea tras `review`: rc 3, `Rechazada` (la lleva ana). PoC 1f.
- Rama del ID solo local, sin publicar y sin `asignado`: rc 3, `Rechazada`, sin estado ni segunda rama. PoC 1f (TASK-012).
- Mutacion: se quita la comprobacion de dueño de `cli_start` (`if [ "$asig" != "$quien" ]` sustituido por `if true`). La salida de la 1f cambia: la repeticion del dueño da rc 3 y, tras `review`, el HEAD recreado no coincide con el remoto (HEAD=NO). El guion detecta el fallo, aunque no aserta: lo detecta quien lee el RESULTADO.

Conclusion: IMPORTANTE-1 queda resuelto y la redaccion del ADR (fila `start`, decision 2, divergencia 3, RF-24 y «Preguntas abiertas») es coherente con lo que hace el guion.

## Notas (fuera de la tabla; no abren ronda)

1. **Base invalida al reanudar.** La validacion de la base (`rev-parse` de `baseref`) ocurre antes de mirar el `asignado`. Un dueño cuya rama ya existe no puede reanudar si su base ya no existe. Es lo que dice el ADR («valida la base antes de escribir nada»), y hoy no hay caso real (`origin/develop` y `origin/main` existen). Sin accion.
2. **Rama recreada desde la base no esta en el log.** La subrama de la decision 2 (c) «desde la base si no existe en ningun sitio» (dueño que publico el `asignado` y murio antes de crear la rama) no aparece en `poc-f1-3.log`; la 1f del ADR cubre solo el caso «rama en el remoto» y «rama local». Lo ejercite en el escenario extra E1 (rc 0, rama creada desde `origin/develop`, worktree creado). Si se quiere trazabilidad, anadir E1 al guion.
3. **Tracking distinto.** La rama creada en (b) va «sin tracking» (`--no-track`, como dice el ADR); la recreada desde el remoto en (c) va con `--track`. El ADR no fija nada para (c), asi que no es contradiccion. Cosmetico.
4. **El guion no aserta.** Los `RESULTADO` se imprimen con los valores reales y no se comparan con un esperado; la correccion se verifica leyendolos. Con la mutacion anterior fue suficiente, pero un fallo futuro solo se ve si alguien lee la salida.
5. **MENOR-1, MENOR-2 y MENOR-3 de la ronda 1** no se revisan aqui (los comprueba quien coordina). Como verificacion de paso, la PoC 1e muestra el patron con guion final (`task-007-` no casa `task-0070-`), y la 1g la base del hotfix sin tags; ambas coinciden con su texto en el ADR.

## Verificacion realizada

- Puerta del proyecto: no hay build, linter ni suite de tests. La puerta aplicable es `bash .github/scripts/comprobar-repo.sh`: `Tareas: 93; enlaces relativos: 27; fallos: 0`, exit 0.
- PoC completa: `tareas/03-en-revision/TASK-004/poc/poc-f1-3.sh` ejecutado en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-004-r2\run`, exit 0, 192 lineas de salida. Compare las secciones 1e, 1f y 1g con el ADR (`docs/adr/F1-3-flujo-de-ramas-worktrees-release.md`):
  - 1e: `'*/task-007-*'` casa solo `feature/task-007-login`; sin el guion final tambien casa `task-0070-x`. Coincide.
  - 1f: repeticion del dueño, codigo 0 y 0, 0 commits de estado, rama igual; tras `review`, worktree recreado, HEAD igual, estado sin reescribir; otra persona codigo 3; ID solo local codigo 3. Coincide.
  - 1g: base invalida, codigo 3, 0 commits de estado, `asignado` vacio en origin, sin ramas del ID; `main` sin tags, `start` codigo 0 desde `origin/main`. Coincide.
  - Los mensajes literales de `Rechazada` y `[start]` de los bloques 1d-1g del ADR coinciden con la salida. Los hashes cambian, como dice el ADR.
- Escenarios extra fuera del guion (en la carpeta temporal, no en el repo): E1 (reanudacion desde la base) y E3 (liberada la tarea con rama publicada, `start` de otra persona: rc 3, `Rechazada`). Un tercer intento tenia un error de mi parte en el argumento de la base y no se cuenta.
- Mutacion: descrita en el hallazgo IMPORTANTE-1 (verificacion).
- Diff de la correccion: `git diff 4d69f6c..431c400` mas `750d4ae`. El commit `750d4ae` solo toca `revision/` y `tarea.md`; el ADR y la PoC son los mismos que en `431c400`.
- Sin push ni remotos reales: solo remotos bare locales de la PoC. Se borro la carpeta temporal `task-004-r2` al terminar; su ruta real resulto estar bajo `Packages\...\LocalCache` por la virtualizacion de `AppData`, y se borro por la ruta logica.

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-004/revision/informe-revision-1.md`: origen y reproduccion de los hallazgos de la ronda 1, que la peticion nombra como fuente.
- `C:\Users\nullcad2025\.claude\plugins\cache\taskcode-marketplace\taskcode-plugin\0.16.1\skills\task-workflow\revision.md`: reglas comunes de revision que la skill obliga a leer antes de empezar.
- `tareas/03-en-revision/TASK-004/poc/poc-f1-3.log`: comparacion de las secciones 1e a 1g con su salida real (entregable que la peticion pide reproducir).
- `.github/scripts/comprobar-repo.sh`: ejecutado, como pide la peticion.
- Historial de git (`git show --stat`, `git diff`) para acotar el delta de la correccion.
