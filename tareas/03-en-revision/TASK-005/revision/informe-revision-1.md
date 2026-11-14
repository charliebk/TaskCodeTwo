# Informe de revision — TASK-005 (ronda 1)

- Commit revisado: 61f15c5436be0aa7c50535710c6bb9f576fcad50
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md:286 | implementacion |
| IMP-2 | IMPORTANTE | corregido | tareas/03-en-revision/TASK-005/poc/poc-f1-4.sh:324 | implementacion |
| IMP-3 | IMPORTANTE | corregido | docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md:349 | implementacion |
| MEN-1 | MENOR | corregido | tareas/03-en-revision/TASK-005/poc/poc-f1-4.sh:517 | implementacion |
| MEN-2 | MENOR | corregido | docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md:212 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Veredicto propuesto: **aprobada-con-correcciones** (sin CRITICOS; el ADR cumple los 3 criterios de
`tarea.md`, la reproduccion confirma la PoC, y los 3 IMPORTANTES son correcciones de texto o de PoC).

## Verificacion realizada

- `bash .github/scripts/comprobar-repo.sh` en el commit 4a37538 (HEAD, arbol limpio): `Tareas: 93; enlaces relativos: 33; fallos: 0`.
- **PoC reproducida** en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-005-r1` (Git 2.56.0.windows.4, 3 min 24 s) y comparada con `poc-f1-4.log`: **identica salvo B3** (push 288 frente a 289 bytes y fetch 185 frente a 186; total crudo 923 frente a 925 bytes, mismo 67 %), que el ADR ya atribuye a la longitud de la ruta del remoto. Parte (A) sin cambios (57 llamadas, 22 de escritura/red, 100134 bytes, 2421075 bytes x turnos; la sesion principal se acota por fecha, por eso las transcripciones nuevas no mueven las cifras). B1: 22 ok / 0 mal con `C` y con `es_ES`, 0 salidas distintas, 0 fuera de contrato, codigos `0:19 3:1 4:2`. B2: codigo 6, `guardar` rechaza dos veces (3), merge con 2 padres, 283 llamadas a git, 0 prohibidas segun el patron.
- Metodo de (A): el guion abre las transcripciones en solo lectura y solo imprime cifras y nombres de subcomando; el log no contiene texto de transcripciones; el repo no recibe copias. Las 9 transcripciones de agentes existen (2+3+4). Porcentajes del ADR (1,4 / 1,5 / 3,6 %) recalculados a mano: 63157/4,65 M, 147229/9,91 M, 394881/11,06 M; correctos con la hipotesis de 4 bytes por token, que el ADR declara no verificada.
- **Mutacion 1 (idempotencia)**: quitada de `op_guardar` la linea `guardar sin-cambios` en una copia del guion. Resultado: B1 `21 ok / 1 mal` en ambas fixtures y B2 `12 ok / 2 mal`, con `guardar error (codigo 1)` en la 2.a ejecucion. La PoC discrimina: detecta que una operacion deje de ser idempotente.
- **Mutacion 2 (grep de prohibidas)**: ver MEN-1.
- **Caso fuera del guion 1**: ver IMP-2 (`tag-anotado` con tag local sin publicar).
- **Caso fuera del guion 2**: ver IMP-1 (`commit-estado` sin `reset`).
- IDs citados, comprobados con `grep`: RR-01, RR-16 y RR-14 en `docs/plan/PLAN-TASKCODE-2.md` y `REQUISITOS-FUNCIONALES.md`; RF-23, RF-24, RF-25 y RN-05 en `REQUISITOS-FUNCIONALES.md` (lineas 144-146 y 211; el texto coincide con lo que dice el ADR); H9, H21 y H22 en `AUDITORIA-EXTERNA-2026-11-13.md`; P5, P8, P12 y la tabla de la Fase 1 (F1-4 en la linea 308) en el plan; F1-5, F1-6, F1-9, F1-10, F1-13, F2-2, F2-3, F2-4b, F2-6, F2-9, F3-6, F3-8, F6-1, F6-2, F7-1, F7-3, F8-2 y F9-7 existen en la tabla del plan. Sin hallazgo por IDs.
- Coherencia con F1-1/F1-2/F1-3: rutas `.taskcode/worktrees/TASK-NNN`, ramas `<tipo>/task-NNN-<slug>`, merge commit sin ff y ramas conservadas (D4a de F1-3), base `main` para hotfix, y codigos 0/3/4/5 de F1-2: coherentes. Unica tension: IMP-1.
- Criterios de `tarea.md`: «lista cerrada de operaciones» (decision 2, 18 filas) y «objetivo de tokens por tarea» (decision 6, tres cifras con linea base de (A)) cumplidos; «cumple F1-4 del plan y los requisitos que cita»: cumplido con las salvedades de IMP-1 e IMP-3.

## Reproduccion de cada hallazgo y correccion

### IMP-1 — La excepcion `reset --hard` de `commit-estado` no es necesaria, no se ha ejercitado y no se probo ninguna alternativa

Ficheros: `docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md:286-291` (divergencia 2), `:370-374` (decision 3) y `:431-432` (F2-3).

Hechos:

1. El plan aprobado (`planificacion/plan-final.md`, «Prohibidas para la capa y el agente») prohibe `reset --hard` sin excepciones. El ADR abre una excepcion con el argumento de que «el protocolo de F1-2 ... es un reset de esa rama». El ADR F1-2 solo dice «descartar el commit local» (nunca `reset`); el `reset --hard` es la implementacion de su PoC (`TASK-003/poc/poc-f1-2.sh:107,116,137`). Descartar un commit propio no obliga a `reset`.
2. La excepcion **no se ha ejecutado nunca**: B2 informa `reset de commit-estado ejecutado: 0` y el ADR lo reconoce, pero afirma una necesidad sobre un camino sin probar. El reset es la unica pieza del flujo con `--hard` sobre el worktree de estado, donde RN-05 pide no perder nada (`reset --hard` tambien descarta cambios sin commitear de `tareas/`).
3. Alternativa sin reset, reproducida (clon con `taskcode/tareas` montada en `tareas/`, remoto bare con `receive.denyNonFastForwards=true`; script en el scratchpad de la sesion, carpeta temporal borrada):
   - `git branch -f taskcode/tareas origin/taskcode/tareas` dentro del worktree falla: `fatal: cannot force update the branch 'taskcode/tareas' used by worktree`. `update-ref` o `checkout -B` dejarian indice y arbol desincronizados y acabarian necesitando un `read-tree` o un `reset`. Esa via no sirve.
   - Fontaneria: `fetch` -> `hash-object -w` + `read-tree`/`update-index --cacheinfo` con `GIT_INDEX_FILE` temporal sobre `origin/taskcode/tareas` -> `write-tree` -> `commit-tree -p origin/taskcode/tareas` -> `push <sha>:refs/heads/taskcode/tareas` (sin force) -> solo si el push va bien, `merge --ff-only origin/taskcode/tareas` en `tareas/`. Con un segundo escritor que se cuela entre medias: intento 1 `codigo 5`, HEAD local = origin, `status` vacio, **no queda commit local que descartar y no se ejecuta `reset`**; intento 2 (re-ejecucion): `codigo 0`, contenido `estado: terminada`, `status` vacio. Es el mismo compare-and-swap de F1-2 (D1a) sin tocar el worktree.
   - Matiz: F1-2 D2b permite, **sin red**, un commit local de una tarea que ya llevas y publicarlo despues; ese commit pendiente si vive en la rama, y descartarlo si el push posterior se rechaza sigue necesitando mover la rama. Esa variante **no la he verificado**; el ADR tampoco.

Correccion concreta (cualquiera de las dos):

- (preferida) Sustituir la excepcion por la variante de fontaneria para el camino con red: quitar la excepcion de `:370-374`, de la divergencia 2 (`:286-291`) y de `:431-432`; anotar en la tabla de alternativas «`commit-estado` con commit en la rama y `reset --hard`: descartada, rompe la prohibicion sin necesidad»; y dejar en «No verificado» el caso sin red de F1-2 D2b (commit local pendiente), que F2-3 debe resolver.
- (minima) Mantener la excepcion pero: (a) corregir la justificacion (el ADR F1-2 no impone un reset; lo hace su PoC), (b) listar la alternativa de fontaneria como descartada con motivo o como abierta, (c) pasar el camino del reset a «No verificado» (no ejercitado) con un test obligatorio en F2-3 con dos escritores, y (d) acotarla a cuando `tareas/` este limpio.

Clasificacion: IMPORTANTE (decision que rompe una prohibicion del plan aprobado sobre la proteccion de RN-05, sin evidencia y con una alternativa que funciona). No es CRITICO: el reset opera sobre un commit propio sin publicar y el ADR lo acota.

### IMP-2 — `tag-anotado` publica el commit equivocado y dice `ok` si ya existe un tag local sin publicar en otro commit

Fichero: `tareas/03-en-revision/TASK-005/poc/poc-f1-4.sh:324` (`gop rev-parse -q --verify "refs/tags/$t" ... || gop tag -a ...`). El ADR (`docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md:363`) promete «mismo commit → `sin-cambios`; en otro commit → 3», y la cabecera de la tabla dice «comprobada en B1/B2».

Reproduccion (funciones `op_tag_anotado` y `gop` extraidas del guion, remoto bare local):

```
X=d16e757704ec Y=4ff15aa28fca            # X: commit antiguo; Y: origin/main
git tag -a v1.0.0 -m local X             # tag local sin publicar (p. ej. release abortada sin red)
op_tag_anotado v1.0.0 origin/main
tag-anotado ok tag=v1.0.0 commit=4ff15aa28fca    codigo 0   <- dice Y
remoto apunta a: d16e757704ec                                <- publico X
```

El guion solo compara con el tag **remoto** (`ls-remote`); si el tag local ya existe no lo recrea, lo empuja tal cual y la linea de contrato imprime el commit pedido en vez del publicado. Un tag de release en el commit equivocado con un `ok` es lo contrario de lo que el contrato dice. B1 solo prueba «crear» y «mismo commit», no «otro commit».

Correccion concreta: antes de empujar, si existe el tag local comparar `git rev-parse "refs/tags/$t^{commit}"` con `$c`; si difieren, `tag-anotado rechazada tag=$t motivo=existe-en-otro-commit` (3); si coinciden, solo empujar. Añadir a B1 los pasos «tag local en otro commit» y «tag remoto en otro commit» y regenerar el log. Ademas, un tag remoto ligero no casa con `refs/tags/$t^{}` y acabaria en `sinred` en vez de 3 (no ejecutado: razonado); consultar tambien `refs/tags/$t` sin `^{}`.

### IMP-3 — El ADR dice «comprobada en B1/B2» para caminos que ninguna prueba ha ejecutado

Ficheros: `docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md:349` (cabecera «Idempotencia (comprobada en B1/B2 salvo plataforma)») y `:414` (evidencia de RR-01: «22 pasos × 2 fixtures, todos idempotentes»).

El log (y mi reejecucion) solo observa: `ok`, `sin-cambios`, `sinred` (fetch y comprobar-id), `conflicto` y los dos `guardar rechazada`. **No se ejecuta en ninguna parte**: `push rechazada` por non-fast-forward y el reintento ante fallo transitorio (el log no tiene ninguna linea `push rechazada`); `quitar-worktree rechazada` (cambios sin guardar o sin publicar); `crear-rama-y-worktree rechazada rama-existe`; `validar-base rechazada`; `tag-anotado` en otro commit (IMP-2); `commit-estado` con rechazo y reintento (el camino que justifica la divergencia 2); `agotada` (5); `uso` (2); `esperando` (7). El ADR marca como «no verificado» la plataforma, el hook, `--json`, Git traducido y versiones, pero no estos caminos locales; la tabla de codigos 1, 2, 5 y 7 los lista sin decir que no se observaron (solo se observan 0, 3 y 4, y 6 en B2).

Correccion concreta: (a) cambiar la cabecera de `:349` por «comprobada en B1/B2 la segunda ejecucion de los caminos ok, sin-cambios, sinred, conflicto y guardar-rechazada; el resto, no verificado», y añadir a «No verificado» (`:301-311`) los caminos de la lista anterior y los codigos 1, 2, 5 y 7 como «propuestos, no observados»; o (b) añadir a B1/B2 los pasos que faltan (push non-fast-forward con `receive.denyNonFastForwards`, `quitar-worktree` con cambios y sin publicar, `crear-rama-y-worktree` con la rama existente, `validar-base` inexistente, tag en otro commit, `commit-estado` con un segundo escritor) y regenerar `poc-f1-4.log`. Con (a) basta para cerrar un ADR.

### MEN-1 — El `grep` de prohibidas no ve variantes de force o de borrado y cuenta como prohibida la propia excepcion

Fichero: `tareas/03-en-revision/TASK-005/poc/poc-f1-4.sh:517` (`PROHIB`). El ADR (`:248-252` y `:419`) usa el «0 prohibidos» y el «patron controlado» como evidencia de RN-05, y el propio ADR cita H9 (una lista de cadenas deja pasar variantes).

Mutacion: se pasa el patron `PROHIB` literal del guion por lineas de registro sinteticas con el mismo formato.

- NO casan (deberian): `git push origin +f` y `git push origin +refs/heads/f:refs/heads/develop` (push forzado por refspec `+`), `git push --mirror origin`, `git update-ref -d refs/heads/f` (borrar rama), `git branch -f f origin/f`, `git checkout -B f origin/f`, `git branch -m a b`, `git worktree remove --force w`.
- Casan (bien): `--force-with-lease`, `rebase --abort`, `reset --hard`.
- Efecto lateral: la linea `commit-estado ... git reset -q --hard origin/taskcode/tareas` tambien casa, asi que si la excepcion se ejercitara el contador «prohibidos» daria mas de 0 y contradiria la excepcion del ADR (el contador `excep` aparte no se resta).

El control de 7 + 3 lineas solo prueba las variantes que el autor ya conocia. Hoy el «0» es cierto (el guion no emite ninguna de las no detectadas), pero la prueba no demuestra lo que el ADR le hace demostrar.

Correccion concreta: pasar el control a lista permitida: con el `sed -E 's/.* git ([a-z-]+).*/\1/'` que ya se calcula en la linea 526, fallar si aparece un subcomando fuera de {add, branch, commit, describe, diff, fetch, for-each-ref, log, ls-remote, merge, merge-base, pull, push, rev-parse, show-ref, status, tag, worktree}, y para `push`, `branch`, `worktree`, `merge` y `tag` comprobar argumentos (`+`, `--mirror`, `--force`, `--delete`, `-f`, `-d`, `-D`, `-m`, `:ref`). Restar `commit-estado` del contador si se mantiene la excepcion. Reflejar en el ADR que la prueba es de lista permitida.

### MEN-2 — Recuento de operaciones incorrecto en la lectura de B1

Fichero: `docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md:212`: «las 15 operaciones locales (más `commit-estado`...)». La tabla de la decision 2 tiene 15 locales **incluyendo** `commit-estado` (14 mas `commit-estado`) y 3 de plataforma; la PoC tiene exactamente 15 `op_*`. Correccion: «las 14 operaciones locales, más `commit-estado`, ...» (o «las 15 operaciones locales»).

## Observaciones sin hallazgo

- Los puntos 1, 3 y 4 de la «Divergencia con el plan» (peso de Git en los tokens, escritura/red de la linea base, locale trivial) estan bien fundados y se confirman con (A) y B0. El matiz del locale esta declarado como no verificado: correcto.
- El metodo de (A) cuenta como lectura algunas llamadas sin subcomando reconocido (`git =2/3`, p. ej. `git -C ruta` sin verbo); no cambia las cifras de forma material (35 de 57 lecturas), pero la cota de escritura/red podria estar infravalorada en 1-3 llamadas por tarea. No se reporta como hallazgo: el ADR llama «cota» a las cifras y no basa ninguna decision en esa diferencia.
- El guion lleva como valor por defecto la ruta de usuario de las transcripciones y los IDs de sesion y de agente; es coherente con las PoC anteriores y el segundo argumento lo sustituye. Sin hallazgo.

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-005/poc/poc-f1-4.sh` y `.log`: son el entregable reproducido que pide el prompt de lanzamiento.
- `docs/plan/PLAN-TASKCODE-2.md`, `docs/plan/REQUISITOS-FUNCIONALES.md` y `docs/auditoria/AUDITORIA-EXTERNA-2026-11-13.md` (solo `grep` de IDs y las lineas de P5, RN-05, RF-23/24/25, F1-4 y §9): para comprobar que los IDs citados existen y dicen lo que el ADR afirma.
- `docs/adr/F1-1-carpeta-fija-y-rama-de-estado.md`, `F1-2-escrituras-atomicas-ids-y-toma.md` y `F1-3-flujo-de-ramas-worktrees-release.md` (lecturas parciales por `grep` y `sed`): coherencia con las decisiones de las que depende F1-4.
- `tareas/04-terminadas/TASK-003/poc/poc-f1-2.sh` (`grep reset`): para ver si el `reset --hard` lo impone el ADR F1-2 o su PoC (IMP-1).
- `.github/scripts/comprobar-repo.sh` (ejecutado): lo pide el prompt.
- Transcripciones locales de Claude Code, a traves del propio guion de la PoC (parte A), en solo lectura; no se ha copiado contenido al repo ni a este informe.
- `tareas/03-en-revision/TASK-005/revision/peticion-revision-1.md`: la peticion de la ronda.
