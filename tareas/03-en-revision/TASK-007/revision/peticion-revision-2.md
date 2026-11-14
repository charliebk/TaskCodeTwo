# Peticion de revision — TASK-007 (ronda 2)

- Tarea: TASK-007 — F1-6 ADR identidades del agente, del revisor y de la persona
- Rama revisada: feature/task-007-f1-6-adr-identidades-del-agente-del-revi
- Rama base: develop
- Commit revisado (HEAD): 3e5bc5759cbe1e6ea885006723839294d66bb540
- Fecha: 2026-11-14
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-007 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 63a62a21b916d356c77fc1e607cb55b6db2aefab (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (9 hallazgo(s)))

## Instrucciones para el agente revisor (verificacion acotada)

Eres un revisor INDEPENDIENTE del agente que implemento. La rama ya paso una
revision completa: esta ronda verifica las correcciones de la ronda anterior,
no vuelve a revisar la rama entera ni los MENORES de la ronda anterior (tu
tabla solo trae CRITICOS e IMPORTANTES). Haz esto, y nada mas:

1. Comprueba, uno por uno, que cada hallazgo pendiente de tu tabla de abajo
   queda resuelto por el diff de esta ronda (los marcados `corregido` tambien:
   verificalo, no lo des por bueno). Para un CRITICO o IMPORTANTE, reproduce el
   fallo original y comprueba que ya no ocurre; puedes leer el repo entero si la
   causa no esta en el diff.
2. Ejecuta la suite (o el comando de verificacion del proyecto) sobre el commit
   revisado y di el resultado: verificar sin ejecutar no vale. Si esta peticion
   ya trae la verificacion de la ronda en verde para ese mismo commit, no la
   repitas: cita ese resultado y corre solo los tests de lo que verificas.
3. Busca en el diff de esta ronda el fallo que la correccion abre: es justo
   donde se cuelan los fallos nuevos. Si encuentras uno, construye el caso que
   lo rompe antes de reportarlo.

Solo abre un hallazgo NUEVO si es CRITICO o si lo introduce la propia correccion
(algo que el diff de esta ronda rompe). Lo demas que veas fuera de eso va como
nota al final del informe, no en la tabla: no abre otra ronda.

Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. En la tabla, cada hallazgo pendiente que ya este
resuelto va como `corregido`; el que no, `abierto`. Vuelca tu salida en el
informe de esta ronda (informe-revision-2.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-007/tarea.md
- tareas/03-en-revision/TASK-007/planificacion/
- docs/adr/F1-6-identidades-agente-revisor-persona.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 6 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-6-identidades-agente-revisor-persona.md:333 | informe-revision-1.md |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-6-identidades-agente-revisor-persona.md:337 | informe-revision-1.md |
| IMP-3 | IMPORTANTE | corregido | tareas/03-en-revision/TASK-007/poc/poc-f1-6.sh:81 | informe-revision-1.md |

## Commits a revisar (git log 63a62a21b916d356c77fc1e607cb55b6db2aefab..HEAD)

````
3e5bc57 chore(TASK-007): constancia de autorrevision ronda 2
ed9f807 docs(adr): F1-6 corrige los hallazgos de la revision 1 (lectura de checks, can_approve de persona, tablas desde el ADR, menores)
6d13175 chore(TASK-007): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 63a62a21b916d356c77fc1e607cb55b6db2aefab..HEAD)

````diff
diff --git a/docs/adr/F1-6-identidades-agente-revisor-persona.md b/docs/adr/F1-6-identidades-agente-revisor-persona.md
index 838fa10..187231e 100644
--- a/docs/adr/F1-6-identidades-agente-revisor-persona.md
+++ b/docs/adr/F1-6-identidades-agente-revisor-persona.md
@@ -64,18 +64,22 @@ Guion `poc-f1-6.sh` y su salida completa `poc-f1-6.log`, en la carpeta
 `poc/` de TASK-007 (sin enlace: la carpeta de la tarea cambia de estado al
 cerrarse). Ejecutado en Git Bash con `gh` 2.95.0, Python 3.12 (`python -I`),
 Git 2.56.0 y OpenSSH 10.6p1 bajo
-`%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-007\poc\` (borrada al
-terminar); cada descarga va a su propia carpeta vacía (`esquema/`, `docs/`) y
-las claves desechables a `claves/`, nunca al repo. **Solo lectura en
-GitHub**: toda llamada pasa por una función con `gh api --method GET`, y el
-guion las registra y cuenta las de otro método.
+`%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-007\poc\` y, tras la
+revisión 1, `...\task-007-c1\` (borradas al terminar); cada descarga va a su
+propia carpeta vacía (`esquema/`, `docs/`), las claves desechables a
+`claves/` y las copias mutadas del ADR a `mutaciones/`, nunca al repo.
+**Las tablas de las decisiones 1 y 2 se extraen de este fichero**, entre
+los marcadores `tabla-identidades` y `tabla-permisos` (revisión 1, IMP-3).
+**Solo lectura en GitHub**: toda llamada pasa por una función con
+`gh api --method GET`, y el guion las registra y cuenta las de otro método.
 
 ```bash
-bash poc-f1-6.sh "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-007/poc" > poc-f1-6.log 2>&1
+bash poc-f1-6.sh "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-007-c1" \
+  docs/adr/F1-6-identidades-agente-revisor-persona.md > poc-f1-6.log 2>&1
 ```
 
 ```text
-RESULTADO llamadas gh api con GET=10 ; gh api con otro metodo=0 ; gh repo view=2 ; descargas GET=3
+RESULTADO llamadas gh api con GET=9 ; gh api con otro metodo=0 ; gh repo view=2 ; descargas GET=3
 ```
 
 Fuentes descargadas:
@@ -90,83 +94,123 @@ Fuentes descargadas:
   Apps». Hay dos versiones de API (`fpt-2022-11-28` y `fpt-2026-03-10`);
   los dos ficheros son idénticos (433 876 bytes, mismo sha256).
 
-### A — Permisos de cada identidad contra `app-permissions`
+### A — Permisos de cada App contra `app-permissions`
+
+Los permisos se leen de la tabla de la decisión 2.
 
 ```text
 app-permissions: 57 permisos; additionalProperties=(no declarado)
-  contents        enum=['read', 'write']
+  checks          enum=['read', 'write']
+  statuses        enum=['read', 'write']
   workflows       enum=['write']
-agente   contents:write, pull_requests:write, metadata:read             -> VALIDO
-revisor  checks:write, contents:read, pull_requests:read, metadata:read -> VALIDO
+agente   contents:write, pull_requests:write, checks:read, statuses:read, metadata:read -> VALIDO
+revisor  checks:write, contents:read, pull_requests:read, metadata:read                 -> VALIDO
+agente lee checks y statuses sin escribirlos (checks:read, statuses:read): si
   permiso inventado approvals:write             -> rechazado: permiso desconocido approvals
   permiso inventado merge:write                 -> rechazado: permiso desconocido merge
   valor inventado contents:admin                -> rechazado: contents='admin' fuera del enum ['read', 'write']
   workflows:read (el enum solo admite write)    -> rechazado: workflows='read' fuera del enum ['write']
-RESULTADO A identidades=2 errores=0 ; negativos rechazados=4/4
+RESULTADO A apps=2 errores=0 ; agente checks/statuses solo lectura=si ; negativos rechazados=4/4
 ```
 
 No existe un permiso «aprobar» ni «mergear» que se pueda quitar aparte: se
 aprueba con `pull_requests` y se mergea con `contents` (B).
 
-### B — Endpoints que alcanza cada identidad
+### B — Las tablas del ADR frente a los permisos por endpoint
 
 La descripción OpenAPI **no declara permisos por endpoint**: `x-github` solo
 trae `category`, `subcategory`, `enabledForGitHubApps`, `githubCloudOnly` y
 avisos de obsolescencia. La fuente es la tabla de `github/docs`. Una
 identidad «alcanza» un endpoint si tiene alguno de los permisos que la tabla
 le asigna con nivel suficiente (cota superior: la tabla marca algunos con
-`additional-permissions`); «no alcanza» es firme, porque ninguno de sus
-permisos aparece para ese endpoint.
+`additional-permissions`, como `forks` o `generate`, que además exigen
+crear repos); «no alcanza» es firme, porque ninguno de sus permisos aparece
+para ese endpoint. Cada operación de la decisión 1 tiene en el guion sus
+endpoints (las de Git, por su equivalente REST: `push` → `POST`/`PATCH
+/git/refs`, `fetch` → `GET /git/matching-refs` y `/commits`). El guion
+comprueba, para cada fila: que cada App asignada alcanza todos sus
+endpoints; que las de solo persona (`approve`, `merge-pr`, `tag-anotado`,
+`aplicar-puertas`, `leer-permisos-actions`) no están asignadas a una App;
+que `aplicar-puertas` y `leer-permisos-actions` no las alcanza ninguna App;
+que cada permiso se usa; y los invariantes de la decisión 4.
 
 ```text
+decision 1 extraida del ADR: 13 operaciones
 -- tabla fpt-2022-11-28: 940 endpoints server-to-server
-  crear o mover rama (equivale a push)   POST  /git/refs                        exige=contents:write,workflows:write  agente=si revisor=no
-  abrir-pr                               POST  /pulls                           exige=pull_requests:write             agente=si revisor=no
-  editar el PR (titulo, descripcion)     PATCH /pulls/{pull_number}             exige=pull_requests:write             agente=si revisor=no
-  actualizar rama del PR (boton)         PUT   /pulls/{pull_number}/update-branch exige=pull_requests:write           agente=si revisor=no
-  merge-pr                               PUT   /pulls/{pull_number}/merge       exige=contents:write                  agente=si revisor=no (limite)
-  merge directo de ramas                 POST  /merges                          exige=contents:write                  agente=si revisor=no (limite)
-  review (tambien APPROVE)               POST  /pulls/{pull_number}/reviews     exige=pull_requests:write             agente=si revisor=no (limite)
-  descartar una review                   PUT   /pulls/{n}/reviews/{id}/dismissals exige=pull_requests:write           agente=si revisor=no (limite)
-  check revision-ia                      POST  /check-runs                      exige=checks:write                    agente=no revisor=si
-  estado de commit (statuses)            POST  /statuses/{sha}                  exige=statuses:write                  agente=no revisor=no
-  aplicar-puertas (crear ruleset)        POST  /rulesets                        exige=administration:write            agente=no revisor=no
-  permisos de workflows del repo         PUT   /actions/permissions/workflow    exige=administration:write            agente=no revisor=no
-  secreto de Actions                     PUT   /actions/secrets/{secret_name}   exige=secrets:write                   agente=no revisor=no
-  fichero de .github/workflows/ por API  PUT   /contents/{path} [workflow]      exige=contents:write,workflows:write  agente=no revisor=no
-  tag (objeto anotado)                   POST  /git/tags                        exige=contents:write                  agente=si revisor=no (limite)
-  release                                POST  /releases                        exige=contents:write,workflows:write  agente=si revisor=no (limite)
-  leer commit y su firma                 GET   /commits/{ref}                   exige=contents:read                   agente=si revisor=si
-  leer-puertas (reglas de la rama)       GET   /rules/branches/{branch}         exige=metadata:read                   agente=si revisor=si
-  comentar en el PR                      POST  /issues/{issue_number}/comments  exige=issues:write,pull_requests:write agente=si revisor=no
-  ... (23 filas; completas en el log)
-las 2 versiones dan el mismo resultado en las 23 filas: si
-minimo agente   usados=contents,pull_requests ; sin uso=ninguno
-minimo revisor  usados=checks,contents,pull_requests ; sin uso=ninguno
-RESULTADO B filas=23 x versiones=2 fallos=0 ; versiones iguales=si ; permisos sin uso=0
+  fetch                  identidad=agente           agente=si revisor=si  exige=contents:read + contents:read
+  push                   identidad=agente           agente=si revisor=no  exige=contents:write/workflows:write + ...
+  commit-estado          identidad=agente           agente=si revisor=no  exige=contents:write/workflows:write
+  approve                identidad=persona          agente=si revisor=no  exige=contents:write/workflows:write
+  abrir-pr               identidad=agente           agente=si revisor=no  exige=pull_requests:read + pull_requests:write
+  estado-puertas         identidad=agente,persona   agente=si revisor=no  exige=contents:read/pull_requests:read + pull_requests:read + checks:read + statuses:read
+  merge-pr               identidad=persona          agente=si revisor=no (limite)  exige=contents:write
+  tag-anotado            identidad=persona          agente=si revisor=no (limite)  exige=contents:write/workflows:write
+  aplicar-puertas        identidad=persona          agente=no revisor=no  exige=administration:write + administration:write
+  leer-puertas           identidad=agente,persona   agente=si revisor=si  exige=metadata:read + metadata:read
+  leer-permisos-actions  identidad=persona          agente=no revisor=no  exige=administration:read
+  revision-ia            identidad=revisor          agente=no revisor=si  exige=checks:write + checks:write + pull_requests:read + contents:read
+  minimo agente   usados=checks,contents,metadata,pull_requests,statuses ; sin uso=ninguno
+  minimo revisor  usados=checks,contents,pull_requests ; sin uso=ninguno
+  fallos: ninguno
+las 2 versiones dan el mismo resultado: si
+-- enumeracion completa de escrituras (verbo distinto de GET) que alcanza cada App
+  agente: 77 escrituras   (lista completa en el log)
+  revisor: 10 escrituras
+    POST   /markdown                                   contents:read
+    POST   /check-runs                                 checks:write
+    PATCH  /check-runs/{check_run_id}                  checks:write
+    POST   /check-runs/{check_run_id}/rerequest        checks:write
+    POST   /check-suites                               checks:write
+    PATCH  /check-suites/preferences                   checks:write
+    POST   /check-suites/{check_suite_id}/rerequest    checks:write
+    POST   /commits/{commit_sha}/comments              contents:read
+    POST   /forks                                      administration:write,contents:read,repository_creation:write
+    POST   /repos/{template_owner}/{template_repo}/generate  administration:write,contents:read,repository_creation:write
+-- copias mutadas del ADR (en la carpeta temporal): tienen que fallar
+  revisor-pr-y-contents-write        -> detectada: el revisor puede crear reviews (APPROVE) (+2)
+  merge-pr-del-agente                -> detectada: merge-pr asignada a agente (solo persona)
+  agente-sin-checks-read             -> detectada: agente no alcanza estado-puertas
+  leer-permisos-actions-del-agente   -> detectada: leer-permisos-actions asignada a agente (solo persona) (+1)
+  agente-con-administration          -> detectada: agente alcanza aplicar-puertas (exige administration) (+4)
+  revisor-sin-fila-de-operacion      -> detectada: tablas ilegibles: fila sin operacion entre comillas: check revision
+RESULTADO B operaciones=13 x versiones=2 fallos=0 ; versiones iguales=si ; escrituras agente=77 revisor=10 ; mutaciones del ADR detectadas=6/6
 ```
 
 Lectura:
 
-- **El revisor no alcanza ninguna escritura** salvo los checks: ni reviews
-  (tampoco `APPROVE`), ni merge, ni refs, ni `contents`, ni rulesets, ni
-  workflows. Tampoco puede comentar en el PR: sus hallazgos van en la salida
-  del check (resumen y anotaciones).
+- **Las escrituras del revisor son exactamente estas 10**: las seis de
+  checks; `POST /markdown` (renderiza texto, no escribe en el repo); `POST
+  /commits/{sha}/comments`, que la tabla da con `contents:read` (un
+  comentario en un commit); y `forks` y `generate`, que además exigen
+  `administration:write` y `repository_creation:write`, que no tiene. **No
+  alcanza** reviews (tampoco `APPROVE`), merge, refs, escritura de `contents`,
+  rulesets ni workflows, ni comentarios en el PR.
 - **El agente no alcanza** rulesets, permisos de workflows, colaboradores,
-  secretos, checks ni statuses, ni ficheros de `.github/workflows/` por la
-  API.
-- **El agente sí alcanza**, con `contents:write` y `pull_requests:write`:
-  `PUT /pulls/{n}/merge`, `POST /merges`, reviews (incluido `APPROVE` en un PR
-  que no sea suyo), descartar reviews, tags y releases. Son los límites que la
-  decisión 4 deja escritos.
-- **Mínimo**: cada permiso de cada App lo usa al menos una operación que esa
-  App tiene asignada (`metadata:read` es obligatorio en toda App).
+  secretos, escritura de checks y statuses, ni ficheros de
+  `.github/workflows/` por la API. Con `checks:read` y `statuses:read` lee lo
+  que necesita `estado-puertas` (revisión 1, IMP-1).
+- **El agente sí alcanza** 77 escrituras con `contents:write` y
+  `pull_requests:write`. Las que importan aquí: `PUT /pulls/{n}/merge` y
+  `/merge-async`, `POST /merges` y `/merge-upstream`; reviews (crear, incluido
+  `APPROVE` en un PR ajeno, editar, borrar y descartar); crear, **mover y
+  borrar refs** (`POST`, `PATCH` y `DELETE /git/refs`) y renombrar ramas
+  (`POST /branches/{b}/rename`); tags; crear, editar y **borrar releases**;
+  y `POST /dispatches` (`repository_dispatch`, que lanza los workflows
+  suscritos a ese evento). Son los límites que la decisión 4 deja escritos.
+- `approve` sale alcanzable por el agente: el push a `taskcode/tareas` lo
+  puede hacer cualquiera con escritura; lo que hace de un commit una
+  aprobación es la firma de la persona (decisión 3, PoC E), no el permiso.
+- **Las tablas del ADR son las que se prueban**: una copia del ADR con el
+  revisor con `pull_requests:write` y `contents:write`, con `merge-pr` o
+  `leer-permisos-actions` asignadas al agente, con el agente sin
+  `checks:read` o con `administration:write`, o con una fila ilegible, falla.
 - La descripción de `pull_requests` en el esquema habla de «merges», pero la
   tabla exige `contents:write` (y solo ese) para `PUT /pulls/{n}/merge`.
 
 ### C — Permisos de los workflows y tipo de cuenta (solo `GET`)
 
 ```text
+token usado: charliebk (User) -- la persona; ninguna App
 --- charliebk/TaskCodeTwo (visibilidad: PUBLIC)
 actions/permissions/workflow:      HTTP 200 {"default_workflow_permissions":"read","can_approve_pull_request_reviews":false}
 --- charliebk/taskcodetwo-poc-f1-1 (visibilidad: PRIVATE)
@@ -182,33 +226,39 @@ Lectura:
   Actions **no puede aprobar PR** (`can_approve_pull_request_reviews:
   false`) y los workflows solo leen por defecto. Es lo que tiene que seguir
   siendo: `carril` y `revision-ia` corren en Actions.
+- Se ha leído **con el token de la persona** (administradora): el endpoint
+  exige `administration:read` (PoC B), que ninguna App tiene. Por eso
+  `leer-permisos-actions` es de la persona (revisión 1, IMP-2).
 - **`web-flow` es `type: User`**, no `Bot`: el tipo de cuenta no basta para
   excluir la firma de la propia plataforma; la exclusión tiene que ser
   explícita (decisión 3).
 
-### D — Contrato del firmante y controles negativos
+### D — Contrato del firmante y regla de reviews
 
-Simulación en Python con los `type` reales de C.
+Simulación en Python con los `type` e `id` reales de C; del esquema, que el
+`committer` de un commit y el `user` de una review traen `id` y `type`.
 
 ```text
-  firmante: persona verificada en la lista                 -> cuenta
-  firmante: Bot verificado (github-actions[bot])           -> no cuenta
-  firmante: Bot metido en la lista por error               -> no cuenta
-  firmante: persona verificada fuera de la lista           -> no cuenta
-  firmante: web-flow (firma de la plataforma)              -> no cuenta
-  firmante: web-flow metido en la lista por error          -> no cuenta
-  firmante: persona sin firma                              -> no cuenta
-  firmante: firmado, correo sin cuenta (committer null)    -> no cuenta
-  invariantes sobre la tabla del ADR: se cumplen
-  negativo revisor con pull_requests:write    -> detectado: el revisor puede crear reviews (APPROVE)
-  negativo revisor con contents:write         -> detectado: el revisor puede mergear
-  negativo agente con administration:write    -> detectado: agente administra rulesets
-  negativo agente con workflows:write         -> detectado: agente tiene workflows
-  negativo agente con checks:write            -> detectado: el agente puede publicar checks (revision-ia)
-RESULTADO D firmante 8/8 casos ; invariantes=ok ; negativos detectados=5/5
+esquema: commit.committer tiene id=True type=True ; review.user tiene id=True type=True ; review.state=True
+  approve: persona verificada en la lista (login e id)    -> cuenta
+  approve: Bot verificado (github-actions[bot])           -> no cuenta
+  approve: Bot metido en la lista por error               -> no cuenta
+  approve: persona verificada fuera de la lista           -> no cuenta
+  approve: web-flow (firma de la plataforma)              -> no cuenta
+  approve: web-flow metido en la lista por error          -> no cuenta
+  approve: persona sin firma                              -> no cuenta
+  approve: firmado, correo sin cuenta (committer null)    -> no cuenta
+  approve: login reutilizado por otra cuenta (otro id)    -> no cuenta
+  review:  APPROVED de la persona                         -> cuenta
+  review:  APPROVED de la App del agente                  -> no cuenta
+  review:  APPROVED de github-actions[bot]                -> no cuenta
+  review:  COMMENTED de la persona                        -> no cuenta
+  review:  APPROVED de persona fuera de la lista          -> no cuenta
+RESULTADO D approve 9/9 casos ; reviews 5/5 casos ; esquema con id y type=si
 ```
 
-La simulación comprueba la lógica del contrato, no GitHub.
+La simulación comprueba la lógica del contrato, no GitHub. Los controles
+negativos sobre los permisos están en B, sobre el propio ADR.
 
 ### E — Firma SSH local con claves desechables
 
@@ -217,7 +267,7 @@ Un repo local hace de `taskcode/tareas`: el agente escribe
 con su clave SSH y el trailer `Plan-Blob: <sha del blob>`.
 
 ```text
-commit de approve ab86416 ; firma=G ; trailer Plan-Blob=1608d7cd2379f7dcb0576f1981d650e21d2304e6
+commit de approve f3770a0 ; firma=G ; trailer Plan-Blob=1608d7cd2379f7dcb0576f1981d650e21d2304e6
 verify-commit approve, allowed_signers bueno:            ok
 verify-commit approve, allowed_signers con otra clave:  falla
 plan aprobado (firma ok y Plan-Blob = blob actual):     cuenta
@@ -230,6 +280,9 @@ OpenSSH admite claves FIDO ed25519-sk: si (crear una exige llave fisica: no se p
 RESULTADO E casos=8/8 como se esperaba ; claves FIDO admitidas por OpenSSH=1
 ```
 
+(El hash del commit de `approve` cambia en cada ejecución: la clave
+desechable es nueva.)
+
 Lectura:
 
 - El mecanismo de F1-5 funciona con Git y OpenSSH tal cual: firma SSH
@@ -259,34 +312,63 @@ Lectura:
    explícita de `web-flow` (decisión 3, PoC C y D).
 4. **`can_approve_pull_request_reviews`** responde 200 también en el repo
    privado gratuito (el plan lo dejaba sin verificar) y vale `false` en los
-   dos. Se añade a lo que comprueba `leer-puertas` (decisión 4).
+   dos. Se comprueba con la nueva operación `leer-permisos-actions`, de la
+   persona (decisión 1 y 4).
 5. **El agente puede crear tags y releases** con `contents:write` (PoC B):
    el ruleset de tags de F1-5 impide borrarlos o moverlos, no crearlos. Queda
    como límite escrito y pregunta abierta (F1-10).
-6. **El revisor no puede comentar en el PR** con estos permisos (PoC B): sus
-   hallazgos van en la salida del check `revision-ia`, no en comentarios.
-   RF-32 habla de «JSON + comentarios»; lo concreta F6.
-7. **`taskctl doctor`** del plan de pruebas no se ejecuta en esta
-   implementación (el orquestador lo reserva); `comprobar-repo.sh` sí.
+6. **El revisor no publica reviews ni comenta en el PR** con estos permisos
+   (PoC B). La fila F6-1 del plan dice que el revisor publica «reviews y
+   checks», y RF-32 habla de «JSON + comentarios»: aquí sus hallazgos van en
+   la salida del check `revision-ia` (resumen y anotaciones); la única vía de
+   comentario que le queda es un comentario de commit (`POST
+   /commits/{sha}/comments`, con `contents:read`). **Hay que actualizar la
+   fila F6-1 del plan** para que diga «checks» en vez de «reviews y checks»;
+   lo concreta F6.
+7. **`taskctl doctor`** del plan de pruebas lo ejecutó el orquestador tras
+   la implementación, con 0 errores; `comprobar-repo.sh`, también en verde.
+8. **El agente lee checks y statuses** (revisión 1, IMP-1, decidido por
+   Carlos con la consigna «la opción recomendada»): el plan le daba solo
+   `contents:write`, `pull_requests:write` y `metadata:read`, con los que
+   `estado-puertas` no podía leer los checks (`checks:read`) ni el estado
+   combinado (`statuses:read`). Se añaden ambos en lectura, nunca en
+   escritura.
+9. **`can_approve_pull_request_reviews` lo lee la persona** (revisión 1,
+   IMP-2, misma consigna): exige `administration:read`; pasa de
+   `leer-puertas` y `review` a la nueva operación `leer-permisos-actions`, en
+   `doctor` e `init` con el token de la persona.
+10. **La PoC lee las tablas del ADR** (revisión 1, IMP-3): en la primera
+    versión los permisos estaban copiados en el guion y una mutación del ADR
+    no ponía nada en rojo.
+11. **Menores de la revisión 1**: enumeración completa de escrituras por App
+    (MEN-1, MEN-2), regla de reviews en el contrato (MEN-3), lectura del
+    remoto con identidad (MEN-4), fila F6-1 y caja de merge (MEN-5) y lista
+    de personas con `id` numérico (MEN-6).
 
 ### No verificado
 
 - **Permisos efectivos de una App real**: no se ha creado ninguna (D1a); el
   `app_id` del revisor sigue sin valor y los JSON de F1-5 conservan
   `revision-ia@0`. Lo cierran **F6-1** y **F2-6**.
+- **Que GitHub impida a una App aprobar el PR que abrió la misma App**: la
+  regla «no se puede aprobar el PR propio» es conocida para usuarios; para
+  una App, sin comprobar.
 - **Que la plataforma no cuente la aprobación de una App** para
   `required_approving_review_count`: la tabla dice que el agente puede crear
   una review `APPROVE` en un PR ajeno; si GitHub la cuenta, solo la contiene
-  `carril` (que solo cuenta personas).
+  `carril` (regla 6 de la decisión 3).
 - **Que `contents:write` baste para `PUT /pulls/{n}/merge` con todo en
   verde** y que un ruleset sin bypass lo bloquee con algo pendiente: según la
   tabla, sí; no ejecutado.
 - **Que `git push` de un cambio en `.github/workflows/` falle sin
   `workflows`**: la tabla cubre la API REST (y lo confirma para
   `PUT /contents`); para `git push` es documentación de GitHub, sin probar.
+- **Que `fetch` por Git con un token de instalación con `contents:read`
+  funcione en un repo privado**: se usa el equivalente REST.
 - **La marca «Verified» de GitHub** en un commit de `approve` firmado por la
-  persona con su clave registrada, y qué cuenta devuelve la API como
-  `committer`: PoC E prueba el mecanismo con `allowed_signers` local.
+  persona con su clave registrada, y que la API devuelva en `committer` la
+  cuenta (con su `id`) de quien firmó: PoC E prueba el mecanismo con
+  `allowed_signers` local; PoC D, que el esquema trae `id`.
 - **Claves FIDO**: OpenSSH 10.6 admite `sk-ssh-ed25519`; crear y usar una
   exige una llave física que no se ha probado, ni que Git for Windows la use
   con el toque.
@@ -304,11 +386,11 @@ se ha creado ninguna App):
 
 | Dónde | Qué se ve |
 |---|---|
-| Settings → GitHub Apps (instaladas) | dos Apps, `taskcode-agente` y `taskcode-revisor` (nombres de ejemplo), instaladas solo en los repos del proyecto, cada una con sus permisos: la del agente «Contents: Read and write, Pull requests: Read and write, Metadata: Read-only»; la del revisor «Checks: Read and write, Contents: Read-only, Pull requests: Read-only, Metadata: Read-only» |
+| Settings → GitHub Apps (instaladas) | dos Apps, `taskcode-agente` y `taskcode-revisor` (nombres de ejemplo), instaladas solo en los repos del proyecto, cada una con sus permisos: la del agente «Contents: Read and write, Pull requests: Read and write, Checks: Read-only, Commit statuses: Read-only, Metadata: Read-only»; la del revisor «Checks: Read and write, Contents: Read-only, Pull requests: Read-only, Metadata: Read-only» |
 | Lista de PR | autor `taskcode-agente[bot]`, con la etiqueta «bot» |
 | Commits de la rama de la tarea | empujados por el agente, con su cuenta `[bot]` o sin cuenta asociada; sin «Verified» (no se exige) |
 | Checks del PR | `revision-ia` publicado por `taskcode-revisor`, con sus hallazgos en el resumen y las anotaciones; `verificacion` y `carril` por GitHub Actions |
-| Caja de merge | «Review required» hasta que la persona aprueba; una review del agente o del revisor no aparece (el revisor no puede crearla; la del agente no cuenta para `carril`) |
+| Caja de merge | «Review required» hasta que la persona aprueba; una review del agente en un PR ajeno aparece en el PR, pero no cuenta para `carril`; el revisor no puede crearla |
 | Botón de merge | lo pulsa la persona, o `taskctl finish` con su identidad (`merge-pr`); en el historial, el merge commit es de la persona |
 | Commit de `approve` en `taskcode/tareas` | «Verified» con la cuenta de la persona; el trailer `Plan-Blob` en el mensaje |
 | Settings → Actions → General | «Allow GitHub Actions to create and approve pull requests» desmarcado (`can_approve_pull_request_reviews: false`) |
@@ -323,34 +405,40 @@ se ha creado ninguna App):
    código 3), sin probar con otra.
 
    <!-- tabla-identidades:inicio -->
-   | Operación (F1-4, F1-5) | Comando dueño | Identidad | Por qué |
+   | Operación (F1-4, F1-5 y esta) | Comando dueño | Identidad | Por qué |
    |---|---|---|---|
+   | `fetch`, `comprobar-id`, `actualizar-principal` (lectura del remoto) | todo comando que lee el remoto | agente | basta `contents:read`; el CLI usa la identidad del agente (también en un repo privado) |
    | `push` (ramas de tarea) | `review`, `actualizar-rama` | agente | `require_last_push_approval` de F1-5: si empuja la persona, no puede aprobar |
    | `actualizar-rama` (merge de la base + `push`) | `actualizar-rama`, `finish` | agente | ídem: el último push tiene que ser del agente |
    | `commit-estado` (push a `taskcode/tareas`) | todos los de fase | agente | push directo sin PR (F1-1, F1-2) |
-   | `commit-estado` del `approve` | `approve` (F4-2) | persona, **con su firma** | la firma verificada es la aprobación del plan (F1-5, decisión 4) |
+   | `approve` (commit firmado publicado con `commit-estado`) | `approve` (F4-2) | persona, **con su firma** | la firma verificada es la aprobación del plan (F1-5, decisión 4) |
    | `abrir-pr` (también el de release y el de vuelta del hotfix) | `review`, `release`, `hotfix` | agente | la persona tiene que poder aprobarlo |
-   | `estado-puertas` | `review`, `finish` | cualquiera | lectura |
+   | `estado-puertas` | `review` (agente), `finish` (persona) | agente, persona | lee PR, reviews, check-runs (`checks:read`) y estado combinado (`statuses:read`) |
    | `merge-pr` | `finish`, `release` | persona | ninguna IA mergea (decisión 4) |
    | `tag-anotado` (y su push) | `release`, `finish` de `hotfix` | persona | publicar una versión es de la persona (quién lanza la release: F1-10) |
    | `aplicar-puertas` | `init` | persona | exige `administration`, que ninguna IA tiene |
-   | `leer-puertas` | `doctor`, `review` | cualquiera | lectura |
+   | `leer-puertas` (rulesets y reglas de rama) | `doctor`, `review` | agente, persona | lectura con `metadata:read` |
+   | `leer-permisos-actions` (nueva: `can_approve_pull_request_reviews`) | `doctor`, `init` | persona | exige `administration:read`, que ninguna IA tiene |
    | check `revision-ia` | workflow del revisor (F6) | revisor | su `app_id` es el `integration_id` de `revision-ia` |
    <!-- tabla-identidades:fin -->
 
-   Las operaciones solo locales de F1-4 (`fetch`, `guardar`,
-   `crear-rama-y-worktree`, …) no hablan con la plataforma: corren con Git y
-   la configuración local; `fetch` y `push` usan el token del rol que diga la
-   tabla.
+   La PoC (B) extrae esta tabla y la de la decisión 2 de este fichero y
+   comprueba que cada identidad de App alcanza los endpoints de cada
+   operación que tiene asignada. Las operaciones solo locales de F1-4
+   (`guardar`, `crear-rama-y-worktree`, `quitar-worktree`, …) no hablan con el
+   remoto. `leer-permisos-actions` entra en la lista cerrada del puerto
+   `Plataforma` por este ADR.
 
 2. **Permisos mínimos por identidad** (nombres de permiso de GitHub Apps,
    validados contra `app-permissions`, PoC A; alcance por endpoint, PoC B):
 
+   <!-- tabla-permisos:inicio -->
    | Identidad | Forma | Permisos | Sin | Credencial |
    |---|---|---|---|---|
-   | Agente implementador | GitHub App (D2a) | `contents:write`, `pull_requests:write`, `metadata:read` | `administration`, `workflows`, `checks`, `statuses`, `secrets`, `issues` | clave privada de la App en la máquina del agente; tokens de instalación de corta vida que pide el CLI |
-   | Revisor IA | otra GitHub App (D2a) | `checks:write`, `contents:read`, `pull_requests:read`, `metadata:read` | cualquier otra escritura | clave privada como secreto de entorno de Actions (D5a); fuera de la máquina del agente |
-   | Persona | su cuenta de usuario | su rol en el repo: admin para `aplicar-puertas`, escritura para `merge-pr` | — | token propio, usado solo con TTY (H6); clave de firma registrada en su cuenta, FIDO `ed25519-sk` si tiene llave (D4a), si no SSH o GPG con frase de paso y sin caché en el agente de claves (D4b) |
+   | `agente` (implementador) | GitHub App (D2a) | `contents:write`, `pull_requests:write`, `checks:read`, `statuses:read`, `metadata:read` | `administration`, `workflows`, escritura de checks y statuses, `secrets`, `issues` | clave privada de la App en la máquina del agente; tokens de instalación de corta vida que pide el CLI |
+   | `revisor` (IA) | otra GitHub App (D2a) | `checks:write`, `contents:read`, `pull_requests:read`, `metadata:read` | cualquier otra escritura | clave privada como secreto de entorno de Actions (D5a); fuera de la máquina del agente |
+   | `persona` | su cuenta de usuario | su rol en el repo: admin para `aplicar-puertas`, escritura para `merge-pr` | — | token propio, usado solo con TTY (H6); clave de firma registrada en su cuenta, FIDO `ed25519-sk` si tiene llave (D4a), si no SSH o GPG con frase de paso y sin caché en el agente de claves (D4b) |
+   <!-- tabla-permisos:fin -->
 
    Las Apps se instalan **solo en los repos del proyecto**. Sin `workflows`,
    el agente no puede cambiar `.github/workflows/` (por la API, PoC B; por
@@ -366,36 +454,56 @@ se ha creado ninguna App):
    3. esa cuenta es `type: User` en `GET /users/{login}` (no `Bot`);
    4. no es `web-flow` (exclusión explícita: es `type: User`, PoC C);
    5. el trailer `Plan-Blob` del commit es el blob actual de `plan-final.md`
-      en `taskcode/tareas` (F1-5).
+      en `taskcode/tareas` (F1-5);
+
+   y, para las aprobaciones del PR (`GET /repos/{o}/{r}/pulls/{n}/reviews`):
+
+   6. una review cuenta para `carril` solo si su `state` es `APPROVED`, su
+      `user` es `type: User`, está en la lista de personas y no es
+      `web-flow`. Una review de una App (`[bot]`) o de una cuenta fuera de la
+      lista no cuenta nunca, la cuente o no la plataforma (PoC D).
 
    La **lista de personas** es un fichero en `.taskcode/` de la rama de código
    (D6a), bajo CODEOWNERS (F1-5, decisión 6): cambiarla exige un PR aprobado
-   por una persona. Nunca en `taskcode/tareas`. Un Bot o `web-flow` metidos en
-   la lista por error siguen sin contar (PoC D). Es la misma lista que da a
+   por una persona. Nunca en `taskcode/tareas`. Cada entrada guarda el
+   `login` **y el `id` numérico** de la cuenta, y las reglas 2 y 6 comparan
+   el `id` (`committer.id` del commit, `user.id` de la review): un `login`
+   renombrado y reutilizado por otra cuenta no cuenta (PoC D; revisión 1,
+   MEN-6). Un Bot o `web-flow` metidos en la lista por error siguen sin
+   contar (PoC D). Es la misma lista que da a
    F2-13 la identidad de persona de cada tarea (RR-12).
 
 4. **«Ninguna IA aprueba ni mergea»**, en tres capas:
    1. **El revisor, por permisos**: sin `pull_requests:write` no crea reviews
       (tampoco `APPROVE`); sin `contents:write` no mergea ni empuja (PoC B).
-      Su única escritura es el check.
+      Sus escrituras son los checks, más un comentario de commit
+      (`POST /commits/{sha}/comments`) que la tabla da con `contents:read`;
+      ninguna aprueba ni integra (PoC B, enumeración completa).
    2. **El agente, por `carril` y por la plataforma**: no puede aprobar su
-      propio PR (regla de GitHub); una aprobación suya en un PR ajeno no la
-      cuenta `carril`, que solo cuenta personas (contrato de la decisión 3);
+      propio PR (regla de GitHub para el autor; para una App, no verificado);
+      una aprobación suya en un PR ajeno no la cuenta `carril` (regla 6 de la
+      decisión 3);
       la aprobación del plan exige la firma de la persona (PoC E).
    3. **El merge, por `merge-pr`**: solo corre con la identidad de persona; y
       los rulesets sin `bypass_actors` (F1-5) impiden integrar en `develop` o
       `main` sin la aprobación de la persona y los checks en verde, sea quien
       sea quien pulse.
 
-   Además, `leer-puertas` (y `doctor`) comprueba que
-   `can_approve_pull_request_reviews` siga en `false` (PoC C): si no, un
-   workflow con `GITHUB_TOKEN` podría aprobar.
+   Además, `doctor` e `init`, con el token de la persona, comprueban con
+   `leer-permisos-actions` que `can_approve_pull_request_reviews` siga en
+   `false` (PoC C): si no, un workflow con `GITHUB_TOKEN` podría aprobar. No
+   lo hace `review` ni ninguna App: exige `administration:read` (PoC B).
 
    **Límite explícito (D3a).** GitHub no separa empujar de mergear:
    `contents:write`, que el agente necesita para empujar ramas y
    `taskcode/tareas`, incluye `PUT /pulls/{n}/merge`, `POST /merges`, tags y
    releases; `pull_requests:write`, que necesita para abrir PR, incluye crear
-   y descartar reviews en PR ajenos (PoC B). El agente **técnicamente puede
+   y descartar reviews en PR ajenos (PoC B). La enumeración completa (77
+   escrituras, PoC B) suma: mover y borrar refs (`PATCH` y `DELETE
+   /git/refs`) y renombrar ramas, que en los destinos protegidos frenan los
+   rulesets de F1-5 y en el resto no; borrar y editar releases; `merge-async`
+   y `merge-upstream`; y `repository_dispatch` (`POST /dispatches`), que lanza
+   los workflows suscritos a ese evento. El agente **técnicamente puede
    mergear un PR que ya tenga la aprobación de la persona y los checks en
    verde**, y crear un tag. Lo que no puede es integrar sin esa aprobación
    (rulesets sin bypass) ni hacer pasar su aprobación por la de una persona
@@ -421,14 +529,14 @@ se ha creado ninguna App):
 
 | Requisito | Decisión | Evidencia |
 |---|---|---|
-| P3 / RR-08 las garantías las impone la plataforma | permisos mínimos por App; revisor sin escritura salvo checks; agente sin `administration` ni `workflows` | PoC A (permisos válidos, 4/4 inventados rechazados), B (23 filas × 2 versiones, 0 fallos); permisos efectivos de una App real: no verificado (F6-1) |
-| RR-04 / RF-30 revisor independiente con identidad propia; aprobación de persona | revisor = otra App con `checks:write`; su `app_id` = `integration_id` de `revision-ia`; aprobación solo de personas | PoC B (revisor alcanza `POST /check-runs` y nada más que escriba), D (invariantes; 5/5 tablas alteradas detectadas) |
-| Criterio de F1-6: que ninguna IA pueda aprobar ni mergear | tres capas y límite explícito (decisión 4) | PoC B, C (`can_approve_pull_request_reviews: false` en los dos repos), D, E; que GitHub no cuente la aprobación de una App: no verificado |
-| Criterio de F1-6: permisos mínimos | tabla de la decisión 2 | PoC B (cada permiso usado por una operación asignada; 0 sin uso) |
+| P3 / RR-08 las garantías las impone la plataforma | permisos mínimos por App (decisión 2); revisor sin más escrituras que checks y comentario de commit; agente sin `administration` ni `workflows` | PoC A (permisos leídos del ADR, válidos; 4/4 inventados rechazados), B (13 operaciones del ADR × 2 versiones, 0 fallos; enumeración completa de escrituras: agente 77, revisor 10); permisos efectivos de una App real: no verificado (F6-1) |
+| RR-04 / RF-30 revisor independiente con identidad propia; aprobación de persona | revisor = otra App con `checks:write`; su `app_id` = `integration_id` de `revision-ia`; aprobación solo de personas | PoC B (el revisor alcanza `revision-ia`; ninguna de sus 10 escrituras aprueba ni integra; 6/6 copias mutadas del ADR detectadas, entre ellas el revisor con `pull_requests:write` y `contents:write`), D (regla 6 de reviews, 5/5 casos) |
+| Criterio de F1-6: que ninguna IA pueda aprobar ni mergear | tres capas y límite explícito (decisión 4); `leer-permisos-actions` de la persona | PoC B (invariantes sobre las tablas del ADR), C (`can_approve_pull_request_reviews: false` en los dos repos, con el token de la persona), D, E; que GitHub no cuente la aprobación de una App ni deje a una App aprobar su propio PR: no verificado |
+| Criterio de F1-6: permisos mínimos | tabla de la decisión 2 | PoC B (cada operación asignada a una App la alcanza, incluida `estado-puertas` con `checks:read` y `statuses:read`; cada permiso usado; 0 sin uso) |
 | RN-01 identidad propia del agente; abre PR; la persona aprueba | agente = GitHub App; `abrir-pr` y `push` con su identidad | PoC B (`POST /pulls`, `POST /git/refs`); la firma de los commits del agente no se exige (pregunta abierta) |
-| RR-12 la persona identificada por su usuario; el agente figura como agente | lista de personas en `.taskcode/`; tipo de cuenta `User` frente a `Bot` | PoC C (`type` real), D (8/8 casos); implementación: F2-13 |
+| RR-12 la persona identificada por su usuario; el agente figura como agente | lista de personas en `.taskcode/` con `login` e `id`; tipo de cuenta `User` frente a `Bot` | PoC C (`type` e `id` reales), D (9/9 casos de `approve`, con el `login` reutilizado); implementación: F2-13 |
 | H5 la aprobación humana se puede comprobar | commit de `approve` firmado por la persona, con `Plan-Blob`; contrato del firmante | PoC E (verify-commit bueno/malo, blob cambiado y revertido, clave del agente), D; «Verified» de GitHub: no verificado (F2-6) |
-| H6 sin identidad de persona no hay atajos | `merge-pr`, `aplicar-puertas` y `tag-anotado` solo con la persona; token de persona solo con TTY | decisión 1; PoC E (clave con frase sin TTY no firma); implementación: F4-2 |
+| H6 sin identidad de persona no hay atajos | `merge-pr`, `aplicar-puertas`, `leer-permisos-actions` y `tag-anotado` solo con la persona; token de persona solo con TTY | decisión 1; PoC B (ninguna fila de solo persona asignada a una App); PoC E (clave con frase sin TTY no firma); implementación: F4-2 |
 | RN-06 una persona con agentes | tres identidades aunque haya una sola persona; clave FIDO o con frase para separar persona y agente en la misma máquina | PoC E; FIDO: no verificado |
 | RR-15 mismas garantías en GitLab | correspondencia (decisión 6) | no verificado (F1-13, F6-1) |
 
@@ -436,12 +544,19 @@ se ha creado ninguna App):
 
 - **F1-5**: los JSON de rulesets sustituyen `revision-ia@0` por el `app_id`
   de la App del revisor cuando exista (F6-1); `aplicar-puertas` sigue
-  negándose con el relleno. `leer-puertas` suma la comprobación de
-  `can_approve_pull_request_reviews`.
+  negándose con el relleno.
+- **F1-4**: la lista cerrada del puerto `Plataforma` suma
+  `leer-permisos-actions` (identidad de persona).
+- **F2-6**: el CLI lee el remoto (`fetch`, `comprobar-id`,
+  `actualizar-principal`) con la identidad del agente.
 - **F1-8** cubre el modelo de amenazas de las credenciales: clave de la App
   del agente en local, secreto del revisor en Actions, clave de la persona
-  frente a un agente en la misma sesión, y qué workflow puede leer qué
-  secreto.
+  frente a un agente en la misma sesión, qué workflow puede leer qué
+  secreto, y los workflows que se puedan lanzar con `repository_dispatch`
+  (`POST /dispatches`, alcanzable por el agente con `contents:write`).
+- **F1-10** (además de lo de abajo): renombrar, mover o borrar ramas no
+  protegidas y borrar releases, que el agente alcanza; qué ramas protege un
+  ruleset aparte de las de F1-5.
 - **F1-10** decide quién lanza una release y qué pasa con los tags que el
   agente puede crear.
 - **F1-13** y **F6-1** verifican la correspondencia en GitLab.
@@ -450,12 +565,14 @@ se ha creado ninguna App):
 - **F2-13** usa la lista de personas como identidad de quién lleva cada
   tarea.
 - **F3-1** (`init`) guía la creación e instalación de las dos Apps y el
-  registro de la clave de firma; **F3-8** (`doctor`) comprueba permisos,
-  instalación y `can_approve_pull_request_reviews`.
+  registro de la clave de firma; **F3-8** (`doctor`) comprueba permisos e
+  instalación y, con el token de la persona, `can_approve_pull_request_reviews`
+  (`leer-permisos-actions`).
 - **F4-2** (`approve`) firma con la clave de la persona y escribe el trailer
   `Plan-Blob`; sin firma o sin TTY, rechaza.
 - **F6** (y **F6-1**) crea las Apps, el workflow del revisor y la salida de
-  hallazgos en el check (el revisor no comenta en el PR).
+  hallazgos en el check (el revisor no publica reviews ni comenta en el PR);
+  la fila F6-1 del plan se actualiza («checks», no «reviews y checks»).
 - **Este repo no cambia** (RR-14): sigue sin Apps y con la cuenta de la
   persona hasta el relevo.
 
@@ -498,7 +615,8 @@ como riesgos sin prueba:
   firmar con una clave propia; lo decide F6-1.
 - **Qué cuenta devuelve la API como `committer` de un commit verificado** y
   si otra identidad con el correo de la persona puede obtener `verified`
-  (heredada de F1-5). No verificado.
+  (heredada de F1-5). No verificado; la comparación por `id` cubre el
+  `login` renombrado, no este caso.
 - **Con varias personas**: si el riesgo alto exige dos personas distintas y
   quién mantiene la lista de personas (F1-10).
 - **GitLab**: si un token de proyecto puede quedar fuera de los aprobadores y
````

## Excluido del diff (9 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 63a62a21b916d356c77fc1e607cb55b6db2aefab..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/02-en-curso/TASK-007/poc/poc-f1-6.log                                                | 130 ----------------------
 tareas/02-en-curso/TASK-007/poc/poc-f1-6.sh                                                 | 370 --------------------------------------------------------------
 tareas/{02-en-curso => 03-en-revision}/TASK-007/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-007/planificacion/plan-final.md                 |   0
 tareas/03-en-revision/TASK-007/poc/poc-f1-6.log                                             | 223 ++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-007/poc/poc-f1-6.sh                                              | 449 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-007/revision/informe-revision-1.md                               |  21 ++++
 tareas/03-en-revision/TASK-007/revision/peticion-revision-1.md                              | 591 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-007/tarea.md                                    |  11 +-
 9 files changed, 1292 insertions(+), 503 deletions(-)
````
