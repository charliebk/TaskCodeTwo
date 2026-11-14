# Informe de revision — TASK-007 (ronda 1)

- Commit revisado: 63a62a21b916d356c77fc1e607cb55b6db2aefab
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-6-identidades-agente-revisor-persona.md:333 | implementacion |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-6-identidades-agente-revisor-persona.md:337 | implementacion |
| IMP-3 | IMPORTANTE | corregido | tareas/03-en-revision/TASK-007/poc/poc-f1-6.sh:81 | implementacion |
| MEN-1 | MENOR | corregido | docs/adr/F1-6-identidades-agente-revisor-persona.md:151 | implementacion |
| MEN-2 | MENOR | corregido | docs/adr/F1-6-identidades-agente-revisor-persona.md:158 | implementacion |
| MEN-3 | MENOR | corregido | docs/adr/F1-6-identidades-agente-revisor-persona.md:383 | implementacion |
| MEN-4 | MENOR | corregido | docs/adr/F1-6-identidades-agente-revisor-persona.md:341 | implementacion |
| MEN-5 | MENOR | corregido | docs/adr/F1-6-identidades-agente-revisor-persona.md:266 | implementacion |
| MEN-6 | MENOR | corregido | docs/adr/F1-6-identidades-agente-revisor-persona.md:371 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

## Verificacion realizada

- Puerta: `bash .github/scripts/comprobar-repo.sh` -> `Tareas: 93; enlaces relativos: 47; fallos: 0` (exit 0). No hay build, linter ni suite de codigo (la tarea es un ADR con PoC en bash).
- Reproduccion de la PoC: `poc-f1-6.sh` ejecutado desde una copia, con carpeta de trabajo `%LOCALAPPDATA%/pruebas-claude/taskcodetwo/task-007-r1` (fuera de `C:\` raiz y del repo). Resultado: A 4/4 negativos, B 23 filas x 2 versiones, 0 fallos, C `can_approve_pull_request_reviews=false` en los dos repos, D 8/8 y 5/5, E 8/8, `gh api` con otro metodo = 0. El log obtenido es identico al `poc-f1-6.log` entregado salvo el hash del commit de `approve` (la clave desechable es nueva en cada ejecucion). Mismos commits fijados (`2a33d9cc5d1d`, `d59b2631959d`), mismos bytes y sha256.
- Solo lectura en GitHub: en mi parte solo `GET` (el propio guion) y descargas `raw.githubusercontent.com`; ningun POST/PUT/PATCH/DELETE, App, repo, gist, PR ni token.
- Casos que el guion no cubre (analisis sobre la tabla descargada `fpt-2022-11-28.json`, 940 endpoints):
  1. Enumeracion completa de verbos de escritura alcanzables por cada identidad (el guion solo evalua 23 filas escogidas a mano): revisor = 10 entradas, agente = 80 (ver MEN-1 y MEN-2).
  2. Alcance de lectura de las operaciones marcadas `cualquiera` en la tabla operacion -> identidad (IMP-1 e IMP-2).
  3. Mutacion de la tabla del ADR (dar `pull_requests:write` y `contents:write` al revisor en la decision 2, copia en la carpeta temporal): ningun mecanismo del guion lo detecta, porque no lee el ADR (IMP-3).
  4. Contrato del firmante: los casos Bot `[bot]` verificado y `web-flow` (en lista o no) ya estan en D y salen "no cuenta"; reproducidos sin diferencias.
- Lo que cierra bien (no es hallazgo): los permisos de cada App son validos contra `app-permissions`; el revisor no alcanza reviews, merge, refs, contents ni rulesets (confirmado en la enumeracion completa); el agente no alcanza `administration`, `workflows` (salvo la cota de `additional-permissions`), checks ni statuses; el limite de `contents:write` (merge, tags, releases) y de `pull_requests:write` (reviews ajenos) esta declarado con honestidad en la decision 4; los IDs RR-04, RR-08, RR-12, RR-15, RF-30, RF-32, RF-51, RF-64, RN-01, RN-06, H5, H6, P3 existen en `docs/plan/*.md` y `docs/auditoria/`; coherencia con F1-4 (nombres de operacion de la lista cerrada y las dos de F1-5), F1-5 (check `carril`, `Plan-Blob`, `integration_id`, sin `bypass_actors`) y la fuente de permisos por endpoint (`github/docs`, con la advertencia de que OpenAPI no la trae) correcta.
- Criterios de `tarea.md`: «Permisos minimos» y «que ninguna IA pueda aprobar ni mergear» estan tratados; el segundo, con el limite escrito. Quedan abiertos IMP-1 e IMP-2, que afectan al primero (el minimo declarado no cubre lo que las operaciones asignadas necesitan leer).

## Reproduccion de cada hallazgo

### IMP-1 — `estado-puertas` figura como `cualquiera`, pero el agente no puede leer checks ni statuses
- Donde: docs/adr/F1-6-identidades-agente-revisor-persona.md:333
- Que pasa: la tabla de la decision 1 asigna `estado-puertas` (comandos `review` y `finish`) a «cualquiera». La tabla de permisos de `github/docs` que el propio ADR usa dice que `GET /repos/{o}/{r}/commits/{ref}/check-runs` exige `checks:read` y `GET .../commits/{ref}/status` exige `statuses:read`. El agente (decision 2: `contents:write`, `pull_requests:write`, `metadata:read`) no tiene ninguno; el revisor tiene `checks` pero no `statuses`.
- Reproduccion: sobre `docs/fpt-2022-11-28.json` de la carpeta temporal, con la misma regla de nivel que usa el guion (`NIVEL`, ampliada con `admin`):
  `estado-puertas: check-runs   GET /commits/{ref}/check-runs   exige=[checks:read]    agente=False revisor=True`
  `estado-puertas: status       GET /commits/{ref}/status       exige=[statuses:read]  agente=False revisor=False`
  `estado-puertas: reviews      GET /pulls/{n}/reviews          exige=[pull_requests:read] agente=True revisor=True`
- Impacto: `review` corre con el agente (`abrir-pr` es suyo) y llamaria a `estado-puertas` con un token que recibe 403 al leer los checks; F2-6 tendria que ampliar permisos por su cuenta, rompiendo el «minimo» (PoC B, `permisos sin uso=0`) o decidir en la implementacion con que identidad se lee. El ADR es quien tiene que cerrarlo.
- Correccion concreta: o bien anadir `checks:read` (y `statuses:read` si `estado-puertas` lee el estado combinado) a los permisos del agente en la decision 2 (nunca `checks:write`), actualizar la lista `USO` del guion y el invariante «agente sin checks:write»; o bien fijar `estado-puertas` a la identidad que si lee (`persona` en `finish`; en `review`, la del revisor o solo `reviews`/`mergeable_state` con el agente) y decir que checks se leen con cual. La fila deja de decir «cualquiera».

### IMP-2 — `leer-puertas` y `doctor` (comprobacion de `can_approve_pull_request_reviews`) figuran como `cualquiera`, pero exigen `administration:read`
- Donde: docs/adr/F1-6-identidades-agente-revisor-persona.md:337
- Que pasa: la decision 4 (linea 390) anade a `leer-puertas` y `doctor` la lectura de `can_approve_pull_request_reviews`, y la tabla dice `leer-puertas` | `doctor`, `review` | «cualquiera». La fuente de permisos dice que `GET /repos/{o}/{r}/actions/permissions/workflow` exige `administration:read`, que ni el agente ni el revisor tienen (y que la decision 2 declara a proposito «Sin `administration`»). El PoC C obtuvo el valor con el token `gh` de la persona (`charliebk`, admin), no con una identidad de App, y no lo advierte.
- Reproduccion:
  `doctor/leer-puertas: workflow perms  GET /actions/permissions/workflow  exige=[administration:read]  agente=False revisor=False`
  (mismo analisis que IMP-1; `GET /rulesets`, `/rulesets/{id}` y `/rules/branches/{branch}` si salen `metadata:read`, `agente=True`).
- Impacto: `review` (agente) o un `doctor` lanzado con el token del agente recibirian 403 en esa comprobacion; la unica salida con el diseno actual seria dar `administration:read` al agente, que contradice la decision 2 y la linea «ninguna IA tiene permiso de administrar el repo». La comprobacion que cierra el hueco `GITHUB_TOKEN` aprueba PR queda sin identidad que la ejecute.
- Correccion concreta: partir la operacion en la tabla: `leer-puertas` (rulesets y reglas de rama) = cualquiera; comprobacion de `can_approve_pull_request_reviews` = `persona` (solo `doctor`/`init`, con su token), y quitarla de `review`; o aceptar `administration:read` solo si se demuestra que no habilita escrituras (la tabla marca `administration:read` solo para lecturas, pero habria que declararlo y reflejarlo en la tabla de permisos). Actualizar la decision 4 (linea 390) y la consecuencia de F3-8.

### IMP-3 — La PoC no lee la tabla del ADR: modificar la decision 2 no pone nada en rojo, y la evidencia dice lo contrario
- Donde: tareas/03-en-revision/TASK-007/poc/poc-f1-6.sh:81
- Que pasa: los permisos de cada identidad estan escritos a mano en `identidades.json` dentro del guion; el guion no recibe ni abre ningun `.md` (`grep` de `docs/adr`, `F1-6-identidades` o `.md` en el guion: solo un comentario). El ADR marca su tabla con `<!-- tabla-identidades:inicio/fin -->`, como F1-5 (cuyo guion si la extrae con una regex, `poc-f1-5.sh:251`), pero nada la consume (`grep -rn tabla-identidades` fuera del ADR: ninguna coincidencia util). Las filas de «Requisitos y evidencia» (lineas 425 y siguientes) y la salida D («invariantes sobre la tabla del ADR: se cumplen», «5/5 tablas alteradas detectadas») presentan como comprobada la tabla del ADR lo que solo comprueba una copia.
- Reproduccion: en la carpeta temporal, copia del ADR con el revisor modificado (`pull_requests:write`, `contents:write` en la fila de la decision 2). El guion se invoca con un unico argumento (`poc-f1-6.sh <carpeta>`), no hay manera de apuntarlo al ADR modificado y su salida A/B/D no cambia: la mutacion «dar `pull_requests:write` al revisor en la tabla» no se detecta. Solo lo detecta la mutacion del JSON interno, que es el control negativo de D. Tampoco se comprueba nunca la tabla operacion -> identidad (decision 1): las 23 filas de B llevan identidad esperada escrita en el guion; por eso IMP-1 e IMP-2 no salieron.
- Impacto: el ADR atribuye evidencia (PoC B, D) a una tabla que ningun test lee; un cambio posterior del ADR o de F6-1 puede dejar permisos que violan «ninguna IA aprueba ni mergea» y el guion seguiria en verde.
- Correccion concreta: que el guion extraiga entre los marcadores del ADR (como `poc-f1-6.sh` de F1-5) las dos tablas (decision 1: operacion -> identidad; decision 2: permisos) y derive de ellas `identidades.json` y la lista de operaciones, y que B compruebe para cada fila de la decision 1 que la identidad asignada alcanza el endpoint del puerto (`abrir-pr` -> `POST /pulls`, `estado-puertas` -> checks, reviews y statuses, `leer-puertas` -> rulesets y `actions/permissions/workflow`, etc.). Anadir un marcador para la tabla de la decision 2 (hoy solo tiene marcadores la 1). Si no se hace, corregir las frases de evidencia del ADR para decir que se comprueba «una copia de la tabla incluida en el guion».

### MEN-1 — «El revisor no alcanza ninguna escritura salvo los checks» / «Su unica escritura es el check» es falso con la propia fuente
- Donde: docs/adr/F1-6-identidades-agente-revisor-persona.md:151
- Que pasa: lineas 151 y 380 (y la fila de PoC B del RR-04). La enumeracion completa de verbos no-GET alcanzables por el revisor (`checks:write`, `contents:read`, `pull_requests:read`, `metadata:read`) da 10 entradas: las de checks (`check-runs`, `check-suites`, `rerequest`, `check-suites/preferences`), `POST /markdown` y `POST /repos/{o}/{r}/commits/{sha}/comments` (la tabla de `github/docs` la asigna a `contents:read`), mas `forks` y `generate`, que exigen ademas permisos de creacion que no tiene. El guion solo evalua 23 filas elegidas, asi que no podia ver esto.
- Reproduccion: `python -I` sobre `docs/fpt-2022-11-28.json`, imprimiendo todo `(verbo != GET)` con alcance suficiente: la salida incluye `POST /repos/{owner}/{repo}/commits/{commit_sha}/comments [('contents','read')]`.
- Impacto: ninguno para aprobar o mergear (comentario de commit); pero el ADR afirma un universal que no se ha comprobado y que la fuente contradice. Ademas la salida de hallazgos del revisor (RF-32, «comentarios») si tendria una via: comentarios de commit.
- Correccion concreta: acotar la frase a lo comprobado («ni reviews, ni merge, ni refs, ni contents, ni rulesets, ni workflows») y anadir a PoC B la enumeracion completa de escrituras por identidad (lo hecho aqui), diciendo que el revisor alcanza ademas `POST /commits/{sha}/comments`. Mencionarlo en la divergencia 6.

### MEN-2 — La lista de limites del agente omite escrituras que alcanza con `contents:write` y `pull_requests:write`
- Donde: docs/adr/F1-6-identidades-agente-revisor-persona.md:158
- Que pasa: la lista de lo que el agente «si alcanza» y los limites de la decision 4 citan merge, `/merges`, reviews, descartar reviews, tags y releases. La enumeracion completa del agente (80 entradas) incluye ademas `DELETE` y `PATCH /git/refs/{ref}` (borrar y mover refs), `POST /branches/{b}/rename`, `POST /dispatches` (`repository_dispatch`, que dispara workflows suscritos a ese evento), `PUT /pulls/{n}/merge-async`, `POST /merge-upstream`, `DELETE`/`PATCH /releases/{id}` y `DELETE /pulls/{n}/reviews/{id}`.
- Reproduccion: la misma enumeracion que MEN-1 con `agente`.
- Impacto: ninguna rompe «ninguna IA aprueba ni mergea» por si sola, pero borrar o mover refs queda contenido solo por los rulesets de F1-5 (y solo en los destinos protegidos) y `repository_dispatch` es una puerta de entrada a workflows que F1-8 debe conocer.
- Correccion concreta: anadir una frase y una fila de limite («borrar/mover refs no protegidos, `repository_dispatch`, borrar releases»), y pasar `repository_dispatch` y el renombrado de ramas protegidas a F1-8 y F1-10 en las consecuencias.

### MEN-3 — «contrato de la decision 3» no define como `carril` cuenta las reviews del PR; la prohibicion de aprobar el PR propio se da por hecha sin marcar «no verificado»
- Donde: docs/adr/F1-6-identidades-agente-revisor-persona.md:383
- Que pasa: la decision 4.2 dice que la aprobacion del agente en un PR ajeno «no la cuenta `carril`, que solo cuenta personas (contrato de la decision 3)». La decision 3 solo define el firmante del commit de `approve` del plan (verified, lista, `User`, no `web-flow`, `Plan-Blob`), no cuando una review de PR es de persona (`user.type`, lista de personas). Y «no puede aprobar su propio PR (regla de GitHub)» (linea 381) no esta en «No verificado» pese a no haberse comprobado con una App.
- Reproduccion: lectura cruzada de las decisiones 3 y 4 y de la lista «No verificado» (lineas 272-293): la regla de reviews de PR no aparece; la de «PR propio» no aparece.
- Impacto: la unica defensa declarada si GitHub cuenta la aprobacion de una App es `carril`, y su regla de reviews queda sin especificar en el ADR que tenia que fijarla.
- Correccion concreta: anadir a la decision 3 una regla 6: una review `APPROVED` cuenta para `carril` solo si `user.type == User`, el login esta en la lista de personas y no es `web-flow`; y anadir a «No verificado» que GitHub impide a una App aprobar el PR abierto por la misma App.

### MEN-4 — `fetch` y `actualizar-principal` sin fila; `fetch` aparece a la vez como «solo local» y «con token de rol»
- Donde: docs/adr/F1-6-identidades-agente-revisor-persona.md:341
- Que pasa: la nota dice que las operaciones solo locales de F1-4 (`fetch`, ...) «no hablan con la plataforma» y a continuacion que `fetch` y `push` usan el token del rol de la tabla; la tabla no tiene fila de `fetch` ni de `actualizar-principal` (`pull --ff-only`, que lee el remoto), ni de `comprobar-id` (`ls-remote`).
- Impacto: con repo privado, `fetch` necesita credencial y el ADR no dice cual (el agente con `contents:write` o la persona).
- Correccion concreta: sacar `fetch` de la lista de «solo locales» y anadir una fila «operaciones de lectura del remoto (`fetch`, `comprobar-id`, `actualizar-principal`) | cualquiera con `contents:read`; el CLI usa la identidad del agente».

### MEN-5 — Divergencia 6 no recoge que el plan F6-1 da al revisor «reviews y checks»; y la caja de merge dice algo inexacto
- Donde: docs/adr/F1-6-identidades-agente-revisor-persona.md:266
- Que pasa: la divergencia 6 contrasta con RF-32 (comentarios) pero `PLAN-TASKCODE-2.md` (F6-1, linea 391) dice que el revisor publica «reviews y checks». El ADR le quita las reviews sin marcarlo como divergencia con esa fila. Ademas, en «Como se ve en GitHub» (linea 311) se dice que «una review del agente o del revisor no aparece»: una review `APPROVE` del agente en un PR ajeno aparece en el PR (es lo que el ADR admite en la decision 4); solo no cuenta para `carril`.
- Correccion concreta: anadir F6-1 a la divergencia 6 y pedir que se actualice esa fila del plan; reformular la fila de la caja de merge: «una review del agente aparece pero no cuenta para `carril`; el revisor no puede crearla».

### MEN-6 — La lista de personas se basa en el `login`, que GitHub permite renombrar y reutilizar
- Donde: docs/adr/F1-6-identidades-agente-revisor-persona.md:371
- Que pasa: el contrato compara el `login` de `committer` con la lista, y D simula con logins. Si la persona renombra su cuenta, el nombre antiguo puede registrarlo otra cuenta, que subiria su propia clave y daria commits `verified` con ese login. El PoC C ya obtiene el `id` numerico (`11063961`) y no se usa. Es un caso de borde, no verificado.
- Correccion concreta: guardar en la lista `login` e `id` numerico y comparar el `id` (campo `committer.id` de la API de commits); anadirlo a «No verificado» como pregunta si no se cambia.

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-007/poc/poc-f1-6.sh` y `poc-f1-6.log`: son parte del entregable revisado que se me pidio reproducir, aunque no estan en la lista de `## Contexto permitido`.
- `docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md` y `docs/adr/F1-5-puertas-en-github.md`: motivo, comprobar la lista cerrada de operaciones, `carril`, `Plan-Blob`, `integration_id`, `require_last_push_approval` y que no hay `bypass_actors`, como ordena la peticion.
- `docs/plan/PLAN-TASKCODE-2.md` y `docs/plan/REQUISITOS-FUNCIONALES.md`, `docs/auditoria/AUDITORIA-EXTERNA-2026-11-13.md`: motivo, comprobar que los IDs citados existen y la fila F6-1 (MEN-5).
- `.github/scripts/comprobar-repo.sh` (ejecutado, no leido) y `tareas/04-terminadas/TASK-006/poc/poc-f1-5.sh` (una busqueda de `tabla-carriles`): motivo, IMP-3.
- `C:\Users\nullcad2025\.claude\plugins\cache\taskcode-marketplace\taskcode-plugin\0.16.1\skills\task-workflow\revision.md`: lo exige la skill revisora.
- Descargas de solo lectura de `github/rest-api-description` y `github/docs` (las del guion), en la carpeta temporal.
