# Informe de revision — TASK-007 (ronda 2)

- Commit revisado: 3e5bc5759cbe1e6ea885006723839294d66bb540
- Revisor: code-quality-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-6-identidades-agente-revisor-persona.md:416 | implementacion |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-6-identidades-agente-revisor-persona.md:421 | implementacion |
| IMP-3 | IMPORTANTE | corregido | tareas/03-en-revision/TASK-007/poc/poc-f1-6.sh:241 | implementacion |
| MEN-7 | MENOR | abierto | tareas/03-en-revision/TASK-007/poc/poc-f1-6.sh:98 | implementacion |
| MEN-8 | MENOR | abierto | tareas/03-en-revision/TASK-007/poc/poc-f1-6.sh:285 | implementacion |
| MEN-9 | MENOR | abierto | docs/adr/F1-6-identidades-agente-revisor-persona.md:420 | plan |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Los IMP-1 a IMP-3 se dan por `corregido` tras verificarlos (las lineas son las actuales, no las de la ronda 1). Los MENOR MEN-7 a MEN-9 son nuevos, los abre esta ronda porque los introduce la correccion; no bloquean el cierre, y el coordinador decide si se corrigen o se aceptan con motivo.

### IMP-1 — `estado-puertas` sin lectura de checks y statuses para el agente: corregido

- Verificacion: el ADR asigna `estado-puertas` a `agente, persona` (ADR:416) y el agente tiene `checks:read` y `statuses:read`, sin escritura (ADR:438). El guion, con la tabla del ADR, da `estado-puertas identidad=agente,persona agente=si`, con `exige=... + checks:read + statuses:read`, y sin fallos. Control: en una copia del ADR con `checks:read` quitado al agente, el guion sale con `agente no alcanza estado-puertas` (mutacion `agente-sin-checks-read`, detectada en mi ejecucion).

### IMP-2 — `can_approve_pull_request_reviews` lo leia una App: corregido

- Verificacion: `leer-permisos-actions` figura solo como `persona` (ADR:421), con `administration:read`. El guion la da `identidad=persona agente=no revisor=no`, y ninguna App la alcanza. `leer-puertas` (ADR:420) ya no incluye esa comprobacion. Control propio, M5 (asignarla tambien al revisor): detectada, `leer-permisos-actions asignada a persona,revisor (solo persona); revisor no alcanza leer-permisos-actions`. Efecto en `review`: ver nota 3.

### IMP-3 — La PoC no leia la tabla del ADR: corregido

- Verificacion: el guion recibe la ruta del ADR (poc-f1-6.sh:241) y extrae las dos tablas entre marcadores. Con una copia del ADR en la que el agente tiene `workflows:write` (no esta en el ADR), `poc-f1-6.sh <trabajo> <copia>` sale con `fallos=2` (`agente tiene workflows`) en las dos versiones de la tabla de endpoints. Las mutaciones propias de la tabla (M1 a M8) salen con el mismo modulo: se detectan siete de ocho, la que falla es MEN-7. Las seis mutaciones del propio guion dan 5/6 en mi ejecucion: ver MEN-8.

### MEN-7 — El guion comprueba solo el primer identificador de cada fila de la decision 1

- Donde: tareas/03-en-revision/TASK-007/poc/poc-f1-6.sh:98
- Que pasa: `operaciones()` toma solo el primer `...` de la columna Operacion. La fila `fetch`, `comprobar-id`, `actualizar-principal` (ADR:410) se evalua solo como `fetch`. El ADR (linea 131) dice que el guion comprueba «para cada fila» que cada App asignada alcanza sus endpoints.
- Reproduccion: copia del ADR con la fila ADR:410 asignada solo a `persona` (`comprobar-id` y `actualizar-principal` pasan a persona). Salida del guion: `fallos: ninguno`. No detectada (M7).
- Impacto: bajo, son operaciones de lectura (`ls-remote`, `pull --ff-only`), y `fetch` cubre el mismo tipo de endpoint. Pero la PoC presenta como comprobado algo que no lo esta, el mismo tipo de fallo que IMP-3.
- Sugerencia: evaluar cada `...` de la columna Operacion, o cubrir `comprobar-id` y `actualizar-principal` con sus endpoints, o acotar la frase del ADR (linea 131) a la primera operacion de cada fila.

### MEN-8 — El guion sale con codigo 0 con fallos y no marca las mutaciones no aplicadas

- Donde: tareas/03-en-revision/TASK-007/poc/poc-f1-6.sh:285
- Que pasa: el resultado B sale con `fallos=2` y `mutaciones del ADR detectadas=5/6` con codigo 0. Una mutacion cuyo texto no aparece en el ADR cuenta como «no detectada» sin fallar. Las copias se buscan por texto literal de la fila, asi que cualquier cambio en esa fila la desactiva.
- Reproduccion: `poc-f1-6.sh <trabajo> <copia con workflows:write en agente>`, salida: `agente-con-administration -> NO APLICADA (texto no encontrado)`, `RESULTADO B ... fallos=2 ... mutaciones del ADR detectadas=5/6`, y `exit=0`. La copia cambia la fila `contents:write, pull_requests:write` que usa esa mutacion, y deja la mutacion sin aplicar.
- Impacto: un fallo de la PoC, o una mutacion que no se aplica, no corta la ejecucion ni cambia el codigo de salida; quien mire solo el codigo ve verde. La afirmacion del ADR («tienen que fallar») sigue siendo cierta, pero el guion no lo impone.
- Sugerencia: salir con codigo distinto de 0 si `fallos>0` o si alguna mutacion no se aplica.

### MEN-9 — `doctor` no dice con que token comprueba `can_approve_pull_request_reviews`

- Donde: docs/adr/F1-6-identidades-agente-revisor-persona.md:420
- Que pasa: la fila de `leer-puertas` asigna `doctor` a `agente, persona` (ADR:420); la de `leer-permisos-actions` asigna `doctor` a `persona` (ADR:421). El ADR no dice que hace un `doctor` lanzado con el token del agente: si falla, si omite la comprobacion en silencio o si la marca como no comprobada.
- Reproduccion: lectura cruzada de ADR:420, ADR:421 y de la decision 4 (ADR:492). No ejecutable hasta F3-8. Con el token del agente, `GET /actions/permissions/workflow` exige `administration:read` (PoC B), que el agente no tiene.
- Impacto: la unica comprobacion de que `GITHUB_TOKEN` no aprueba PR (decision 4) puede pasar sin ejecutarse y sin aviso si `doctor` se lanza con el token del agente.
- Sugerencia: fijar en la tabla que esa comprobacion de `doctor` es solo de persona, y que sin ese token `doctor` informa «no comprobado» (no la omite en silencio). No verificado: depende de F3-8.

## Notas (fuera de la tabla, no bloquean)

1. Coherencia con F1-4 y F1-5: F1-4 (aceptada) y F1-5 no contienen `leer-permisos-actions` en la lista cerrada del puerto `Plataforma`. F1-6 lo declara como consecuencia (`F1-4: la lista cerrada del puerto Plataforma suma leer-permisos-actions`), igual que F1-5 declara `aplicar-puertas` y `leer-puertas` para F1-4. No hay contradiccion; queda pendiente la edicion de F1-4, que la tarea debe recoger.
2. La fila `approve` de la tabla de la decision 1 nombra un comando (F4-2) que se apoya en `commit-estado`; la tabla mezcla comandos y operaciones. No cambia permisos.
3. Con IMP-2, `review` ya no comprueba `can_approve_pull_request_reviews` (antes lo hacia via `leer-puertas`): la comprobacion depende de que se ejecute `doctor` o `init`. Es la alternativa que propuse en la ronda 1 y queda escrita en la decision 4; conviene recordarlo a F3-8.
4. La fila F6-1 del plan (`docs/plan/PLAN-TASKCODE-2.md:391`) sigue diciendo «reviews y checks». El ADR lo declara como pendiente de actualizar.

## Verificacion realizada

- Clon temporal de la rama en `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-007-r2-clon` (fuera de `C:\` y del repo), en el commit `24ad853`. El ADR y el guion son identicos a `3e5bc57`: `git diff 3e5bc57 HEAD` solo toca `tareas/`.
- Puerta: `bash .github/scripts/comprobar-repo.sh` -> `Tareas: 93; enlaces relativos: 47; fallos: 0` (exit 0). No hay build, linter ni suite de codigo: el repo son un ADR y un guion en bash.
- PoC: `poc-f1-6.sh C:\...\task-007-r2 <ADR del clon>`, exit 0. A: apps=2, errores=0, negativos 4/4. B: operaciones=13 x 2 versiones, fallos=0, escrituras agente=77, revisor=10, mutaciones del ADR 6/6. C: `can_approve_pull_request_reviews` = false en los dos repos, leido con el token de la persona. D: approve 9/9, reviews 5/5. E: 8/8. Llamadas: GET=9, otro metodo=0. El log entregado (`poc/poc-f1-6.log`) coincide con mi ejecucion salvo el hash del commit de `approve`.
- Mutaciones propias sobre las tablas del ADR (modulo `tablas.py` del guion, copia del ADR): M1 `workflows:write` al agente, detectada; M2 `aplicar-puertas` a agente y persona, detectada; M3 `approve` tambien al agente, detectada; M4 `statuses:write` al revisor, detectada (permiso sin uso); M5 `leer-permisos-actions` tambien al revisor, detectada; M6 agente sin `pull_requests:write`, detectada; M7 `comprobar-id` solo a persona, NO detectada (MEN-7); M8 `estado-puertas` sin agente, detectada solo como permiso sin uso.
- Punto de entrada: `poc-f1-6.sh` con la copia M1 (`workflows:write` al agente): `fallos=2`, `5/6`, exit 0 (MEN-8).
- Solo lectura en GitHub: unicamente GET (guion) y `gh auth status`. Ningun POST, PUT, PATCH ni DELETE.
- Carpetas temporales: se borran al terminar. No hubo commits en el clon.

## Lecturas fuera del contexto permitido

- `docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md` y `docs/adr/F1-5-puertas-en-github.md`: coherencia de la lista cerrada del puerto `Plataforma` (`leer-permisos-actions`, `leer-puertas`, `aplicar-puertas`) y del contrato de `can_approve`.
- `docs/plan/PLAN-TASKCODE-2.md`: filas F1-6, F3-8, F4-2 y F6-1 (la tarea lo exige como criterio, y F6-1 para la nota 4).
- `tareas/03-en-revision/TASK-007/revision/informe-revision-1.md`: hallazgos y verificacion de la ronda 1, para reproducir IMP-1 a IMP-3.
- `tareas/03-en-revision/TASK-007/poc/poc-f1-6.sh` y `poc-f1-6.log`: son el entregable revisado; el guion se ejecuto y leyo entero.
- `C:\Users\nullcad2025\.claude\plugins\cache\taskcode-marketplace\taskcode-plugin\0.16.1\skills\task-workflow\revision.md`: lo exige la skill revisora.
