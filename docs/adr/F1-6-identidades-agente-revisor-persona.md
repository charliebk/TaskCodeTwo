# ADR F1-6 — Identidades del agente implementador, del revisor IA y de la persona

- Estado: **aceptada** (2026-11-14)
- Tarea: TASK-007 · Plan: [F1-6 en PLAN-TASKCODE-2](../plan/PLAN-TASKCODE-2.md) (§4 P3 y tabla de la Fase 1)
- Requisitos: RR-04, RR-08, RR-12, RR-15, RF-30, RF-32, RF-51, RF-64, RN-01 y RN-06 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); H5 y H6 de la [auditoría externa](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md)
- Depende de: [ADR F1-4](F1-4-operaciones-git-y-contrato-de-salida.md) (puerto `Plataforma` y su lista cerrada) y [ADR F1-5](F1-5-puertas-en-github.md) (check `carril`, firma verificada del `approve` atada al blob con el trailer `Plan-Blob`, `integration_id` de `revision-ia`, CODEOWNERS, `strict`, sin `bypass_actors`); nombres de rama y tags de [ADR F1-3](F1-3-flujo-de-ramas-worktrees-release.md)
- Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a —FIDO si hay llave; si no, D4b—, D5a y D6a del plan de TASK-007)
- Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)

## Contexto

F1-5 dejó a este ADR cuatro cosas: la App del revisor (su `app_id` es el
`integration_id` de `revision-ia`, hoy un `0` de relleno), con qué identidad
se abren y actualizan los PR, que ninguna IA tenga permiso de administrar el
repo ni de aprobar, y la identidad de la persona con su clave de firma, la
lista de firmantes que acepta `carril` y que esa clave no esté al alcance del
agente. El marco:

- **P3 / RR-08**: las garantías las impone la plataforma, no las
  instrucciones al agente. El agente implementador y el revisor IA tienen
  **identidad propia**, distinta de la persona: así la persona puede aprobar
  los PR del agente (GitHub no deja aprobar el PR propio) y queda claro quién
  decidió qué. Aprobar el plan lo hace la persona con su identidad; el agente
  no puede, porque no la tiene.
- **RR-04 / RF-30**: revisión independiente por un agente distinto, con
  identidad propia en el PR y check obligatorio; aprobación de una persona
  antes del merge.
- **RN-01**: el agente trabaja con identidad propia (GitHub App o cuenta de
  máquina) y abre PR; la persona aprueba.
- **RR-12**: una tarea la lleva una persona identificada por su usuario de la
  plataforma; el agente figura como agente (lo implementa F2-13).
- **RR-15**: las mismas garantías en GitHub y GitLab, «identidad propia del
  agente» incluida; lo que una plataforma no dé se documenta.
- **RN-06**: un proyecto de una sola persona con agentes funciona igual: la
  persona y el agente comparten máquina y sesión.
- **H5** (la aprobación humana no se puede comprobar) y **H6** (el agente se
  salta revisión y verificación): sin identidad de persona no hay atajos.

Preguntas que responde: qué identidad ejecuta cada operación del puerto
`Plataforma`, qué permisos mínimos tiene cada una, qué firmante acepta
`carril`, cómo se garantiza que ninguna IA apruebe ni mergee y hasta dónde
llega esa garantía.

## Alternativas

| | Descripción | Veredicto |
|---|---|---|
| **D1a** Evidencia de solo lectura | Esquema OpenAPI y tabla pública de permisos por endpoint, `GET` en los dos repos, firma local. | **Elegida**. Descartadas: (b) crear Apps de prueba (escribe en GitHub y deja claves vivas), (c) cuenta de máquina de prueba. |
| **D2a** Dos GitHub Apps | Agente y revisor, cada una con sus permisos. | **Elegida**: permisos por repo y por recurso, tokens de instalación de corta vida, cuentas `[bot]` distinguibles de una persona, y `app_id` estable para el `integration_id`. |
| D2b Cuentas de máquina con PAT | Un usuario de GitHub por IA. | Descartada: un usuario con escritura tiene el rol entero del repo (no hay permiso «sin aprobar»), consume un asiento y su token vive mucho; aparece como `type: User`, igual que una persona. |
| D2c Una sola App | Agente y revisor con la misma identidad. | Descartada: el agente tendría `checks:write` y podría publicar `revision-ia` en verde con el `integration_id` correcto (F1-5, decisión 3). |
| **D3a** El agente conserva `contents:write` | GitHub no separa empujar de mergear; `merge-pr` corre solo con la identidad de persona; rulesets sin bypass y `carril` impiden integrar sin su aprobación. | **Elegida**, con el límite escrito (decisión 4). |
| D3b Agente en un fork | El agente solo escribe en su fork y abre PR desde él. | Descartada: rompe el push directo del CLI a `taskcode/tareas` (F1-1, F1-2) y los worktrees por tarea (F1-3). |
| D3c Solo la cola mergea | El agente no mergea porque solo la cola integra. | Descartada: la cola exige organización (F1-5, D3a). |
| **D4a** Clave FIDO de la persona | SSH `ed25519-sk`: cada firma exige tocar la llave. | **Elegida** si la persona tiene llave. |
| **D4b** Clave con frase de paso | SSH o GPG con frase y sin caché en el agente de claves. | **Elegida** si no hay llave (PoC E: sin agente ni TTY no firma). |
| D4c Firma de la plataforma (`web-flow`) | Commit de `approve` hecho por la API web. | Descartada: F1-5 no la acepta (lo firma GitHub, no la persona). |
| **D5a** Revisor en GitHub Actions | La clave privada de la App del revisor es un secreto de entorno. | **Elegida**: el agente no tiene `workflows` ni `secrets` (PoC B). Detalle en F1-8. |
| D5b Revisor en la máquina del agente | La clave del revisor en disco local. | Descartada: el agente la leería y publicaría `revision-ia` como revisor. |
| **D6a** Lista de firmantes en `.taskcode/` | Fichero en la rama de código, bajo CODEOWNERS (F1-5, decisión 6). | **Elegida**. Descartadas: (b) en `taskcode/tareas` (el agente empuja ahí sin PR), (c) derivada de los colaboradores admin/maintain (mezcla administrar con aprobar y depende de un permiso que leer). |

## Pruebas de concepto

Guion `poc-f1-6.sh` y su salida completa `poc-f1-6.log`, en la carpeta
`poc/` de TASK-007 (sin enlace: la carpeta de la tarea cambia de estado al
cerrarse). Ejecutado en Git Bash con `gh` 2.95.0, Python 3.12 (`python -I`),
Git 2.56.0 y OpenSSH 10.6p1 bajo
`%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-007\poc\` y, tras la
revisión 1, `...\task-007-c1\` (borradas al terminar); cada descarga va a su
propia carpeta vacía (`esquema/`, `docs/`), las claves desechables a
`claves/` y las copias mutadas del ADR a `mutaciones/`, nunca al repo.
**Las tablas de las decisiones 1 y 2 se extraen de este fichero**, entre
los marcadores `tabla-identidades` y `tabla-permisos` (revisión 1, IMP-3).
**Solo lectura en GitHub**: toda llamada pasa por una función con
`gh api --method GET`, y el guion las registra y cuenta las de otro método.

```bash
bash poc-f1-6.sh "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-007-c1" \
  docs/adr/F1-6-identidades-agente-revisor-persona.md > poc-f1-6.log 2>&1
```

```text
RESULTADO llamadas gh api con GET=9 ; gh api con otro metodo=0 ; gh repo view=2 ; descargas GET=3
```

Fuentes descargadas:

- Descripción OpenAPI de `github/rest-api-description`, commit
  `2a33d9cc5d1d` (2026-11-14T00:44:38Z), `api.github.com.json` (14 274 027
  bytes, OpenAPI 3.0.3, versión 1.1.4; el mismo commit que usó F1-5).
- Tabla de permisos por endpoint de las GitHub Apps de `github/docs`, commit
  `d59b2631959d` (2026-11-13T23:26:07Z),
  `src/github-apps/data/fpt-<versión>/server-to-server-permissions.json`: los
  datos de los que sale la página pública «Permissions required for GitHub
  Apps». Hay dos versiones de API (`fpt-2022-11-28` y `fpt-2026-03-10`);
  los dos ficheros son idénticos (433 876 bytes, mismo sha256).

### A — Permisos de cada App contra `app-permissions`

Los permisos se leen de la tabla de la decisión 2.

```text
app-permissions: 57 permisos; additionalProperties=(no declarado)
  checks          enum=['read', 'write']
  statuses        enum=['read', 'write']
  workflows       enum=['write']
agente   contents:write, pull_requests:write, checks:read, statuses:read, metadata:read -> VALIDO
revisor  checks:write, contents:read, pull_requests:read, metadata:read                 -> VALIDO
agente lee checks y statuses sin escribirlos (checks:read, statuses:read): si
  permiso inventado approvals:write             -> rechazado: permiso desconocido approvals
  permiso inventado merge:write                 -> rechazado: permiso desconocido merge
  valor inventado contents:admin                -> rechazado: contents='admin' fuera del enum ['read', 'write']
  workflows:read (el enum solo admite write)    -> rechazado: workflows='read' fuera del enum ['write']
RESULTADO A apps=2 errores=0 ; agente checks/statuses solo lectura=si ; negativos rechazados=4/4
```

No existe un permiso «aprobar» ni «mergear» que se pueda quitar aparte: se
aprueba con `pull_requests` y se mergea con `contents` (B).

### B — Las tablas del ADR frente a los permisos por endpoint

La descripción OpenAPI **no declara permisos por endpoint**: `x-github` solo
trae `category`, `subcategory`, `enabledForGitHubApps`, `githubCloudOnly` y
avisos de obsolescencia. La fuente es la tabla de `github/docs`. Una
identidad «alcanza» un endpoint si tiene alguno de los permisos que la tabla
le asigna con nivel suficiente (cota superior: la tabla marca algunos con
`additional-permissions`, como `forks` o `generate`, que además exigen
crear repos); «no alcanza» es firme, porque ninguno de sus permisos aparece
para ese endpoint. Cada operación de la decisión 1 tiene en el guion sus
endpoints (las de Git, por su equivalente REST: `push` → `POST`/`PATCH
/git/refs`, `fetch` → `GET /git/matching-refs` y `/commits`). El guion
comprueba, para cada operación de cada fila (cada nombre entre comillas
de la columna Operación antes del paréntesis; la primera fila son tres:
`fetch`, `comprobar-id` y `actualizar-principal`): que cada App asignada
alcanza todos sus endpoints; que las que el CLI corre con el agente
(lectura del remoto, `push`, `actualizar-rama`, `commit-estado`, `abrir-pr`)
lo tienen asignado; que las de solo persona (`approve`, `merge-pr`, `tag-anotado`,
`aplicar-puertas`, `leer-permisos-actions`) no están asignadas a una App;
que `aplicar-puertas` y `leer-permisos-actions` no las alcanza ninguna App;
que cada permiso se usa; y los invariantes de la decisión 4. **El guion
sale con código distinto de 0** si alguna sección falla, si hay una llamada
de otro método o si una mutación no se aplica o no se detecta (revisión 2,
MEN-7 y MEN-8).

```text
decision 1 extraida del ADR: 15 operaciones
-- tabla fpt-2022-11-28: 940 endpoints server-to-server
  fetch                  identidad=agente           agente=si revisor=si  exige=contents:read + contents:read
  comprobar-id           identidad=agente           agente=si revisor=si  exige=contents:read
  actualizar-principal   identidad=agente           agente=si revisor=si  exige=contents:read + contents:read
  push                   identidad=agente           agente=si revisor=no  exige=contents:write/workflows:write + ...
  commit-estado          identidad=agente           agente=si revisor=no  exige=contents:write/workflows:write
  approve                identidad=persona          agente=si revisor=no  exige=contents:write/workflows:write
  abrir-pr               identidad=agente           agente=si revisor=no  exige=pull_requests:read + pull_requests:write
  estado-puertas         identidad=agente,persona   agente=si revisor=no  exige=contents:read/pull_requests:read + pull_requests:read + checks:read + statuses:read
  merge-pr               identidad=persona          agente=si revisor=no (limite)  exige=contents:write
  tag-anotado            identidad=persona          agente=si revisor=no (limite)  exige=contents:write/workflows:write
  aplicar-puertas        identidad=persona          agente=no revisor=no  exige=administration:write + administration:write
  leer-puertas           identidad=agente,persona   agente=si revisor=si  exige=metadata:read + metadata:read
  leer-permisos-actions  identidad=persona          agente=no revisor=no  exige=administration:read
  revision-ia            identidad=revisor          agente=no revisor=si  exige=checks:write + checks:write + pull_requests:read + contents:read
  minimo agente   usados=checks,contents,metadata,pull_requests,statuses ; sin uso=ninguno
  minimo revisor  usados=checks,contents,pull_requests ; sin uso=ninguno
  fallos: ninguno
las 2 versiones dan el mismo resultado: si
-- enumeracion completa de escrituras (verbo distinto de GET) que alcanza cada App
  agente: 77 escrituras   (lista completa en el log)
  revisor: 10 escrituras
    POST   /markdown                                   contents:read
    POST   /check-runs                                 checks:write
    PATCH  /check-runs/{check_run_id}                  checks:write
    POST   /check-runs/{check_run_id}/rerequest        checks:write
    POST   /check-suites                               checks:write
    PATCH  /check-suites/preferences                   checks:write
    POST   /check-suites/{check_suite_id}/rerequest    checks:write
    POST   /commits/{commit_sha}/comments              contents:read
    POST   /forks                                      administration:write,contents:read,repository_creation:write
    POST   /repos/{template_owner}/{template_repo}/generate  administration:write,contents:read,repository_creation:write
-- copias mutadas del ADR (en la carpeta temporal): tienen que fallar
  revisor-pr-y-contents-write        -> detectada: el revisor puede crear reviews (APPROVE) (+2)
  merge-pr-del-agente                -> detectada: merge-pr asignada a agente (solo persona)
  agente-sin-checks-read             -> detectada: agente no alcanza estado-puertas
  leer-permisos-actions-del-agente   -> detectada: leer-permisos-actions asignada a agente (solo persona) (+1)
  agente-con-administration          -> detectada: agente alcanza aplicar-puertas (exige administration) (+4)
  revisor-sin-fila-de-operacion      -> detectada: tablas ilegibles: fila sin operacion entre comillas: check revision
  comprobar-id-solo-persona          -> detectada: comprobar-id sin la identidad del agente (+1)
RESULTADO B operaciones=15 x versiones=2 fallos=0 ; versiones iguales=si ; escrituras agente=77 revisor=10 ; mutaciones del ADR detectadas=7/7
mutaciones no aplicadas: 0
...
PoC sin fallos        (código de salida 0)
```

Control del código de salida: con una copia del ADR que da
`workflows:write` al agente, el guion informa `fallos=2`, una mutación «NO
APLICADA», `PoC con fallos en: B` y sale con código 1.

Lectura:

- **Las escrituras del revisor son exactamente estas 10**: las seis de
  checks; `POST /markdown` (renderiza texto, no escribe en el repo); `POST
  /commits/{sha}/comments`, que la tabla da con `contents:read` (un
  comentario en un commit); y `forks` y `generate`, que además exigen
  `administration:write` y `repository_creation:write`, que no tiene. **No
  alcanza** reviews (tampoco `APPROVE`), merge, refs, escritura de `contents`,
  rulesets ni workflows, ni comentarios en el PR.
- **El agente no alcanza** rulesets, permisos de workflows, colaboradores,
  secretos, escritura de checks y statuses, ni ficheros de
  `.github/workflows/` por la API. Con `checks:read` y `statuses:read` lee lo
  que necesita `estado-puertas` (revisión 1, IMP-1).
- **El agente sí alcanza** 77 escrituras con `contents:write` y
  `pull_requests:write`. Las que importan aquí: `PUT /pulls/{n}/merge` y
  `/merge-async`, `POST /merges` y `/merge-upstream`; reviews (crear, incluido
  `APPROVE` en un PR ajeno, editar, borrar y descartar); crear, **mover y
  borrar refs** (`POST`, `PATCH` y `DELETE /git/refs`) y renombrar ramas
  (`POST /branches/{b}/rename`); tags; crear, editar y **borrar releases**;
  y `POST /dispatches` (`repository_dispatch`, que lanza los workflows
  suscritos a ese evento). Son los límites que la decisión 4 deja escritos.
- `approve` sale alcanzable por el agente: el push a `taskcode/tareas` lo
  puede hacer cualquiera con escritura; lo que hace de un commit una
  aprobación es la firma de la persona (decisión 3, PoC E), no el permiso.
- **Las tablas del ADR son las que se prueban**: una copia del ADR con el
  revisor con `pull_requests:write` y `contents:write`, con `merge-pr` o
  `leer-permisos-actions` asignadas al agente, con el agente sin
  `checks:read` o con `administration:write`, con una fila ilegible, o con
  `comprobar-id` y `actualizar-principal` pasadas a la persona, falla.
- La descripción de `pull_requests` en el esquema habla de «merges», pero la
  tabla exige `contents:write` (y solo ese) para `PUT /pulls/{n}/merge`.

### C — Permisos de los workflows y tipo de cuenta (solo `GET`)

```text
token usado: charliebk (User) -- la persona; ninguna App
--- charliebk/TaskCodeTwo (visibilidad: PUBLIC)
actions/permissions/workflow:      HTTP 200 {"default_workflow_permissions":"read","can_approve_pull_request_reviews":false}
--- charliebk/taskcodetwo-poc-f1-1 (visibilidad: PRIVATE)
actions/permissions/workflow:      HTTP 200 {"default_workflow_permissions":"read","can_approve_pull_request_reviews":false}
users/charliebk: charliebk User 11063961
users/web-flow: web-flow User 19864447
users/github-actions[bot]: github-actions[bot] Bot 41898282
```

Lectura:

- En los dos repos, también en el privado gratuito, el `GITHUB_TOKEN` de
  Actions **no puede aprobar PR** (`can_approve_pull_request_reviews:
  false`) y los workflows solo leen por defecto. Es lo que tiene que seguir
  siendo: `carril` y `revision-ia` corren en Actions.
- Se ha leído **con el token de la persona** (administradora): el endpoint
  exige `administration:read` (PoC B), que ninguna App tiene. Por eso
  `leer-permisos-actions` es de la persona (revisión 1, IMP-2).
- **`web-flow` es `type: User`**, no `Bot`: el tipo de cuenta no basta para
  excluir la firma de la propia plataforma; la exclusión tiene que ser
  explícita (decisión 3).

### D — Contrato del firmante y regla de reviews

Simulación en Python con los `type` e `id` reales de C; del esquema, que el
`committer` de un commit y el `user` de una review traen `id` y `type`.

```text
esquema: commit.committer tiene id=True type=True ; review.user tiene id=True type=True ; review.state=True
  approve: persona verificada en la lista (login e id)    -> cuenta
  approve: Bot verificado (github-actions[bot])           -> no cuenta
  approve: Bot metido en la lista por error               -> no cuenta
  approve: persona verificada fuera de la lista           -> no cuenta
  approve: web-flow (firma de la plataforma)              -> no cuenta
  approve: web-flow metido en la lista por error          -> no cuenta
  approve: persona sin firma                              -> no cuenta
  approve: firmado, correo sin cuenta (committer null)    -> no cuenta
  approve: login reutilizado por otra cuenta (otro id)    -> no cuenta
  review:  APPROVED de la persona                         -> cuenta
  review:  APPROVED de la App del agente                  -> no cuenta
  review:  APPROVED de github-actions[bot]                -> no cuenta
  review:  COMMENTED de la persona                        -> no cuenta
  review:  APPROVED de persona fuera de la lista          -> no cuenta
RESULTADO D approve 9/9 casos ; reviews 5/5 casos ; esquema con id y type=si
```

La simulación comprueba la lógica del contrato, no GitHub. Los controles
negativos sobre los permisos están en B, sobre el propio ADR.

### E — Firma SSH local con claves desechables

Un repo local hace de `taskcode/tareas`: el agente escribe
`plan-final.md` sin firmar; la persona crea el commit de `approve` firmado
con su clave SSH y el trailer `Plan-Blob: <sha del blob>`.

```text
commit de approve 0262215 ; firma=G ; trailer Plan-Blob=1608d7cd2379f7dcb0576f1981d650e21d2304e6
verify-commit approve, allowed_signers bueno:            ok
verify-commit approve, allowed_signers con otra clave:  falla
plan aprobado (firma ok y Plan-Blob = blob actual):     cuenta
tras cambiar el plan (blob v2):                         no cuenta
tras revertir el contenido (blob v1 otra vez):          cuenta
approve firmado con la clave del agente, bueno:         falla
approve sin firma, bueno:                               falla
clave con frase, sin ssh-agent ni TTY:                  no firma (error: Enter passphrase for "...")
OpenSSH admite claves FIDO ed25519-sk: si (crear una exige llave fisica: no se prueba)
RESULTADO E casos=8/8 como se esperaba ; claves FIDO admitidas por OpenSSH=1
```

(El hash del commit de `approve` cambia en cada ejecución: la clave
desechable es nueva.)

Lectura:

- El mecanismo de F1-5 funciona con Git y OpenSSH tal cual: firma SSH
  (`gpg.format=ssh`), trailer leído con `%(trailers:key=Plan-Blob)` y
  comparado con `rev-parse HEAD:<plan>`; se compara contenido, no historia.
- Con una clave con frase de paso, sin `ssh-agent` (`SSH_AUTH_SOCK` vacío) y
  sin TTY, **la firma falla** al pedir la frase: un agente que corre sin TTY
  no firma como la persona (D4b). Con la clave cargada en un `ssh-agent`, sí
  podría (no se prueba; lo que se decide es no cachearla).
- El `allowed_signers` local es el equivalente de la marca «Verified» de
  GitHub, no la marca misma.

### Divergencias con el plan

1. **Nombres de operación.** El plan habla de `crear-pr` y `actualizar-pr`;
   la lista cerrada de F1-4 tiene `abrir-pr`, `estado-puertas` y `merge-pr`
   (más `aplicar-puertas` y `leer-puertas` de F1-5). Se usan los de la lista
   cerrada. **Editar la descripción del PR** (para `## Resultado`, RF-51) no
   está en la lista: si F2-6 la necesita, entra por ADR o extensión
   declarada, con la identidad del agente (`pull_requests:write`, PoC B).
2. **Fuente de permisos por endpoint.** El plan suponía una tabla pública;
   existe, pero no en la descripción OpenAPI (sin permisos en `x-github`),
   sino en los datos de `github/docs` que generan la página de permisos de
   las Apps (PoC B).
3. **`web-flow` es `type: User`.** El plan esperaba que el `type` separara
   persona y plataforma; el contrato del firmante suma una exclusión
   explícita de `web-flow` (decisión 3, PoC C y D).
4. **`can_approve_pull_request_reviews`** responde 200 también en el repo
   privado gratuito (el plan lo dejaba sin verificar) y vale `false` en los
   dos. Se comprueba con la nueva operación `leer-permisos-actions`, de la
   persona (decisión 1 y 4).
5. **El agente puede crear tags y releases** con `contents:write` (PoC B):
   el ruleset de tags de F1-5 impide borrarlos o moverlos, no crearlos. Queda
   como límite escrito y pregunta abierta (F1-10).
6. **El revisor no publica reviews ni comenta en el PR** con estos permisos
   (PoC B). La fila F6-1 del plan dice que el revisor publica «reviews y
   checks», y RF-32 habla de «JSON + comentarios»: aquí sus hallazgos van en
   la salida del check `revision-ia` (resumen y anotaciones); la única vía de
   comentario que le queda es un comentario de commit (`POST
   /commits/{sha}/comments`, con `contents:read`). **Hay que actualizar la
   fila F6-1 del plan** para que diga «checks» en vez de «reviews y checks»;
   lo concreta F6.
7. **`taskctl doctor`** del plan de pruebas lo ejecutó el orquestador tras
   la implementación, con 0 errores; `comprobar-repo.sh`, también en verde.
8. **El agente lee checks y statuses** (revisión 1, IMP-1, decidido por
   Carlos con la consigna «la opción recomendada»): el plan le daba solo
   `contents:write`, `pull_requests:write` y `metadata:read`, con los que
   `estado-puertas` no podía leer los checks (`checks:read`) ni el estado
   combinado (`statuses:read`). Se añaden ambos en lectura, nunca en
   escritura.
9. **`can_approve_pull_request_reviews` lo lee la persona** (revisión 1,
   IMP-2, misma consigna): exige `administration:read`; pasa de
   `leer-puertas` y `review` a la nueva operación `leer-permisos-actions`, en
   `doctor` e `init` con el token de la persona.
10. **La PoC lee las tablas del ADR** (revisión 1, IMP-3): en la primera
    versión los permisos estaban copiados en el guion y una mutación del ADR
    no ponía nada en rojo.
11. **Menores de la revisión 1**: enumeración completa de escrituras por App
    (MEN-1, MEN-2), regla de reviews en el contrato (MEN-3), lectura del
    remoto con identidad (MEN-4), fila F6-1 y caja de merge (MEN-5) y lista
    de personas con `id` numérico (MEN-6).
12. **Menores de la revisión 2**: el guion evalúa cada operación de cada
    fila de la decisión 1 (MEN-7), sale con código distinto de 0 si algo
    falla o una mutación no se aplica (MEN-8), y `doctor` sin el token de la
    persona informa «no comprobado» (MEN-9).

### No verificado

- **Permisos efectivos de una App real**: no se ha creado ninguna (D1a); el
  `app_id` del revisor sigue sin valor y los JSON de F1-5 conservan
  `revision-ia@0`. Lo cierran **F6-1** y **F2-6**.
- **Que GitHub impida a una App aprobar el PR que abrió la misma App**: la
  regla «no se puede aprobar el PR propio» es conocida para usuarios; para
  una App, sin comprobar.
- **Que la plataforma no cuente la aprobación de una App** para
  `required_approving_review_count`: la tabla dice que el agente puede crear
  una review `APPROVE` en un PR ajeno; si GitHub la cuenta, solo la contiene
  `carril` (regla 6 de la decisión 3).
- **Que `contents:write` baste para `PUT /pulls/{n}/merge` con todo en
  verde** y que un ruleset sin bypass lo bloquee con algo pendiente: según la
  tabla, sí; no ejecutado.
- **Que `git push` de un cambio en `.github/workflows/` falle sin
  `workflows`**: la tabla cubre la API REST (y lo confirma para
  `PUT /contents`); para `git push` es documentación de GitHub, sin probar.
- **Que `fetch` por Git con un token de instalación con `contents:read`
  funcione en un repo privado**: se usa el equivalente REST.
- **La marca «Verified» de GitHub** en un commit de `approve` firmado por la
  persona con su clave registrada, y que la API devuelva en `committer` la
  cuenta (con su `id`) de quien firmó: PoC E prueba el mecanismo con
  `allowed_signers` local; PoC D, que el esquema trae `id`.
- **Claves FIDO**: OpenSSH 10.6 admite `sk-ssh-ed25519`; crear y usar una
  exige una llave física que no se ha probado, ni que Git for Windows la use
  con el toque.
- **GitLab**: toda la correspondencia (decisión 6).
- **Lo que pinta la interfaz** (sección siguiente).

## Cómo se ve en GitHub

Hoy (observado, PoC C): este repo y el de pruebas no tienen ninguna App de
TaskCode; los workflows solo leen y no pueden aprobar. El flujo 0.16 sigue
con la cuenta de la persona para todo (RR-14).

Con este ADR aplicado (conocimiento general de GitHub, **no verificado**: no
se ha creado ninguna App):

| Dónde | Qué se ve |
|---|---|
| Settings → GitHub Apps (instaladas) | dos Apps, `taskcode-agente` y `taskcode-revisor` (nombres de ejemplo), instaladas solo en los repos del proyecto, cada una con sus permisos: la del agente «Contents: Read and write, Pull requests: Read and write, Checks: Read-only, Commit statuses: Read-only, Metadata: Read-only»; la del revisor «Checks: Read and write, Contents: Read-only, Pull requests: Read-only, Metadata: Read-only» |
| Lista de PR | autor `taskcode-agente[bot]`, con la etiqueta «bot» |
| Commits de la rama de la tarea | empujados por el agente, con su cuenta `[bot]` o sin cuenta asociada; sin «Verified» (no se exige) |
| Checks del PR | `revision-ia` publicado por `taskcode-revisor`, con sus hallazgos en el resumen y las anotaciones; `verificacion` y `carril` por GitHub Actions |
| Caja de merge | «Review required» hasta que la persona aprueba; una review del agente en un PR ajeno aparece en el PR, pero no cuenta para `carril`; el revisor no puede crearla |
| Botón de merge | lo pulsa la persona, o `taskctl finish` con su identidad (`merge-pr`); en el historial, el merge commit es de la persona |
| Commit de `approve` en `taskcode/tareas` | «Verified» con la cuenta de la persona; el trailer `Plan-Blob` en el mensaje |
| Settings → Actions → General | «Allow GitHub Actions to create and approve pull requests» desmarcado (`can_approve_pull_request_reviews: false`) |
| Settings → Secrets and variables → Actions (entorno del revisor) | la clave privada y el id de la App del revisor; ni la persona ni el agente la tienen en local |

## Decisión

1. **Operación → identidad.** Ningún módulo nuevo: el puerto `Plataforma`
   (F1-4) recibe un proveedor de credenciales por rol (`agente`, `revisor`,
   `persona`) y cada operación de la lista cerrada declara con cuál corre.
   Una operación con la identidad equivocada falla cerrada (`rechazada`,
   código 3), sin probar con otra.

   <!-- tabla-identidades:inicio -->
   | Operación (F1-4, F1-5 y esta) | Comando dueño | Identidad | Por qué |
   |---|---|---|---|
   | `fetch`, `comprobar-id`, `actualizar-principal` (lectura del remoto) | todo comando que lee el remoto | agente | basta `contents:read`; el CLI usa la identidad del agente (también en un repo privado) |
   | `push` (ramas de tarea) | `review`, `actualizar-rama` | agente | `require_last_push_approval` de F1-5: si empuja la persona, no puede aprobar |
   | `actualizar-rama` (merge de la base + `push`) | `actualizar-rama`, `finish` | agente | ídem: el último push tiene que ser del agente |
   | `commit-estado` (push a `taskcode/tareas`) | todos los de fase | agente | push directo sin PR (F1-1, F1-2) |
   | `approve` (commit firmado publicado con `commit-estado`) | `approve` (F4-2) | persona, **con su firma** | la firma verificada es la aprobación del plan (F1-5, decisión 4) |
   | `abrir-pr` (también el de release y el de vuelta del hotfix) | `review`, `release`, `hotfix` | agente | la persona tiene que poder aprobarlo |
   | `estado-puertas` | `review` (agente), `finish` (persona) | agente, persona | lee PR, reviews, check-runs (`checks:read`) y estado combinado (`statuses:read`) |
   | `merge-pr` | `finish`, `release` | persona | ninguna IA mergea (decisión 4) |
   | `tag-anotado` (y su push) | `release`, `finish` de `hotfix` | persona | publicar una versión es de la persona (quién lanza la release: F1-10) |
   | `aplicar-puertas` | `init` | persona | exige `administration`, que ninguna IA tiene |
   | `leer-puertas` (rulesets y reglas de rama) | `doctor`, `review` | agente, persona | lectura con `metadata:read` |
   | `leer-permisos-actions` (nueva: `can_approve_pull_request_reviews`) | `doctor`, `init`, **solo con el token de la persona** | persona | exige `administration:read`, que ninguna IA tiene; un `doctor` lanzado sin el token de la persona (p. ej. con el del agente) no la intenta ni la omite en silencio: informa «`can_approve_pull_request_reviews`: no comprobado (falta la identidad de persona)» (no verificado: F3-8) |
   | check `revision-ia` | workflow del revisor (F6) | revisor | su `app_id` es el `integration_id` de `revision-ia` |
   <!-- tabla-identidades:fin -->

   La PoC (B) extrae esta tabla y la de la decisión 2 de este fichero y
   comprueba que cada identidad de App alcanza los endpoints de cada
   operación que tiene asignada. Las operaciones solo locales de F1-4
   (`guardar`, `crear-rama-y-worktree`, `quitar-worktree`, …) no hablan con el
   remoto. `leer-permisos-actions` entra en la lista cerrada del puerto
   `Plataforma` por este ADR.

2. **Permisos mínimos por identidad** (nombres de permiso de GitHub Apps,
   validados contra `app-permissions`, PoC A; alcance por endpoint, PoC B):

   <!-- tabla-permisos:inicio -->
   | Identidad | Forma | Permisos | Sin | Credencial |
   |---|---|---|---|---|
   | `agente` (implementador) | GitHub App (D2a) | `contents:write`, `pull_requests:write`, `checks:read`, `statuses:read`, `metadata:read` | `administration`, `workflows`, escritura de checks y statuses, `secrets`, `issues` | clave privada de la App en la máquina del agente; tokens de instalación de corta vida que pide el CLI |
   | `revisor` (IA) | otra GitHub App (D2a) | `checks:write`, `contents:read`, `pull_requests:read`, `metadata:read` | cualquier otra escritura | clave privada como secreto de entorno de Actions (D5a); fuera de la máquina del agente |
   | `persona` | su cuenta de usuario | su rol en el repo: admin para `aplicar-puertas`, escritura para `merge-pr` | — | token propio, usado solo con TTY (H6); clave de firma registrada en su cuenta, FIDO `ed25519-sk` si tiene llave (D4a), si no SSH o GPG con frase de paso y sin caché en el agente de claves (D4b) |
   <!-- tabla-permisos:fin -->

   Las Apps se instalan **solo en los repos del proyecto**. Sin `workflows`,
   el agente no puede cambiar `.github/workflows/` (por la API, PoC B; por
   `git push`, no verificado): cierra en la plataforma parte del riesgo
   «workflow modificado en el propio PR» de F1-5. Una tarea legítima que toque
   workflows la empuja la persona.

3. **Contrato del firmante** (completa la regla 1 de la decisión 4 de F1-5).
   `carril` cuenta el commit de `approve` solo si, en
   `GET /repos/{o}/{r}/commits/{sha}`:
   1. `commit.verification.verified` es `true`;
   2. la cuenta `committer` no es nula y está en la **lista de personas**;
   3. esa cuenta es `type: User` en `GET /users/{login}` (no `Bot`);
   4. no es `web-flow` (exclusión explícita: es `type: User`, PoC C);
   5. el trailer `Plan-Blob` del commit es el blob actual de `plan-final.md`
      en `taskcode/tareas` (F1-5);

   y, para las aprobaciones del PR (`GET /repos/{o}/{r}/pulls/{n}/reviews`):

   6. una review cuenta para `carril` solo si su `state` es `APPROVED`, su
      `user` es `type: User`, está en la lista de personas y no es
      `web-flow`. Una review de una App (`[bot]`) o de una cuenta fuera de la
      lista no cuenta nunca, la cuente o no la plataforma (PoC D).

   La **lista de personas** es un fichero en `.taskcode/` de la rama de código
   (D6a), bajo CODEOWNERS (F1-5, decisión 6): cambiarla exige un PR aprobado
   por una persona. Nunca en `taskcode/tareas`. Cada entrada guarda el
   `login` **y el `id` numérico** de la cuenta, y las reglas 2 y 6 comparan
   el `id` (`committer.id` del commit, `user.id` de la review): un `login`
   renombrado y reutilizado por otra cuenta no cuenta (PoC D; revisión 1,
   MEN-6). Un Bot o `web-flow` metidos en la lista por error siguen sin
   contar (PoC D). Es la misma lista que da a
   F2-13 la identidad de persona de cada tarea (RR-12).

4. **«Ninguna IA aprueba ni mergea»**, en tres capas:
   1. **El revisor, por permisos**: sin `pull_requests:write` no crea reviews
      (tampoco `APPROVE`); sin `contents:write` no mergea ni empuja (PoC B).
      Sus escrituras son los checks, más un comentario de commit
      (`POST /commits/{sha}/comments`) que la tabla da con `contents:read`;
      ninguna aprueba ni integra (PoC B, enumeración completa).
   2. **El agente, por `carril` y por la plataforma**: no puede aprobar su
      propio PR (regla de GitHub para el autor; para una App, no verificado);
      una aprobación suya en un PR ajeno no la cuenta `carril` (regla 6 de la
      decisión 3);
      la aprobación del plan exige la firma de la persona (PoC E).
   3. **El merge, por `merge-pr`**: solo corre con la identidad de persona; y
      los rulesets sin `bypass_actors` (F1-5) impiden integrar en `develop` o
      `main` sin la aprobación de la persona y los checks en verde, sea quien
      sea quien pulse.

   Además, `doctor` e `init`, con el token de la persona, comprueban con
   `leer-permisos-actions` que `can_approve_pull_request_reviews` siga en
   `false` (PoC C): si no, un workflow con `GITHUB_TOKEN` podría aprobar. No
   lo hace `review` ni ninguna App: exige `administration:read` (PoC B). Sin
   el token de la persona, `doctor` informa «no comprobado»; nunca la da por
   buena ni la calla.

   **Límite explícito (D3a).** GitHub no separa empujar de mergear:
   `contents:write`, que el agente necesita para empujar ramas y
   `taskcode/tareas`, incluye `PUT /pulls/{n}/merge`, `POST /merges`, tags y
   releases; `pull_requests:write`, que necesita para abrir PR, incluye crear
   y descartar reviews en PR ajenos (PoC B). La enumeración completa (77
   escrituras, PoC B) suma: mover y borrar refs (`PATCH` y `DELETE
   /git/refs`) y renombrar ramas, que en los destinos protegidos frenan los
   rulesets de F1-5 y en el resto no; borrar y editar releases; `merge-async`
   y `merge-upstream`; y `repository_dispatch` (`POST /dispatches`), que lanza
   los workflows suscritos a ese evento. El agente **técnicamente puede
   mergear un PR que ya tenga la aprobación de la persona y los checks en
   verde**, y crear un tag. Lo que no puede es integrar sin esa aprobación
   (rulesets sin bypass) ni hacer pasar su aprobación por la de una persona
   (`carril`). La alternativa estricta, el fork (D3b), está descartada.

5. **Dónde vive cada credencial** (detalle de amenazas en F1-8): la clave de
   la App del agente, donde la lea el CLI; la de la App del revisor, solo en
   el entorno de Actions del revisor (D5a); la clave de firma de la persona,
   en una llave FIDO o con frase de paso y sin caché (D4a/D4b); el token de la
   persona, solo usado con TTY. Ninguna clave de persona ni del revisor queda
   en un fichero que el agente pueda leer.

6. **GitLab, como correspondencia** (RR-15; **no verificado**, lo cierran
   F1-13 y F6-1): agente = usuario bot de proyecto creado con un token de
   proyecto con rol Developer; revisor = otro token de proyecto que publica
   su estado como estado de commit o check externo y no figura entre los
   aprobadores; persona = su usuario, único aprobador en las reglas de
   aprobación, con «impedir la aprobación del autor» y «de quien añade
   commits»; merge por la persona. Si el plan de GitLab no da alguna de estas
   piezas, se documenta y `doctor` lo dice (RF-64).

## Requisitos y evidencia

| Requisito | Decisión | Evidencia |
|---|---|---|
| P3 / RR-08 las garantías las impone la plataforma | permisos mínimos por App (decisión 2); revisor sin más escrituras que checks y comentario de commit; agente sin `administration` ni `workflows` | PoC A (permisos leídos del ADR, válidos; 4/4 inventados rechazados), B (13 operaciones del ADR × 2 versiones, 0 fallos; enumeración completa de escrituras: agente 77, revisor 10); permisos efectivos de una App real: no verificado (F6-1) |
| RR-04 / RF-30 revisor independiente con identidad propia; aprobación de persona | revisor = otra App con `checks:write`; su `app_id` = `integration_id` de `revision-ia`; aprobación solo de personas | PoC B (el revisor alcanza `revision-ia`; ninguna de sus 10 escrituras aprueba ni integra; 6/6 copias mutadas del ADR detectadas, entre ellas el revisor con `pull_requests:write` y `contents:write`), D (regla 6 de reviews, 5/5 casos) |
| Criterio de F1-6: que ninguna IA pueda aprobar ni mergear | tres capas y límite explícito (decisión 4); `leer-permisos-actions` de la persona | PoC B (invariantes sobre las tablas del ADR), C (`can_approve_pull_request_reviews: false` en los dos repos, con el token de la persona), D, E; que GitHub no cuente la aprobación de una App ni deje a una App aprobar su propio PR: no verificado |
| Criterio de F1-6: permisos mínimos | tabla de la decisión 2 | PoC B (cada operación asignada a una App la alcanza, incluida `estado-puertas` con `checks:read` y `statuses:read`; cada permiso usado; 0 sin uso) |
| RN-01 identidad propia del agente; abre PR; la persona aprueba | agente = GitHub App; `abrir-pr` y `push` con su identidad | PoC B (`POST /pulls`, `POST /git/refs`); la firma de los commits del agente no se exige (pregunta abierta) |
| RR-12 la persona identificada por su usuario; el agente figura como agente | lista de personas en `.taskcode/` con `login` e `id`; tipo de cuenta `User` frente a `Bot` | PoC C (`type` e `id` reales), D (9/9 casos de `approve`, con el `login` reutilizado); implementación: F2-13 |
| H5 la aprobación humana se puede comprobar | commit de `approve` firmado por la persona, con `Plan-Blob`; contrato del firmante | PoC E (verify-commit bueno/malo, blob cambiado y revertido, clave del agente), D; «Verified» de GitHub: no verificado (F2-6) |
| H6 sin identidad de persona no hay atajos | `merge-pr`, `aplicar-puertas`, `leer-permisos-actions` y `tag-anotado` solo con la persona; token de persona solo con TTY | decisión 1; PoC B (ninguna fila de solo persona asignada a una App); PoC E (clave con frase sin TTY no firma); implementación: F4-2 |
| RN-06 una persona con agentes | tres identidades aunque haya una sola persona; clave FIDO o con frase para separar persona y agente en la misma máquina | PoC E; FIDO: no verificado |
| RR-15 mismas garantías en GitLab | correspondencia (decisión 6) | no verificado (F1-13, F6-1) |

## Consecuencias

- **F1-5**: los JSON de rulesets sustituyen `revision-ia@0` por el `app_id`
  de la App del revisor cuando exista (F6-1); `aplicar-puertas` sigue
  negándose con el relleno.
- **F1-4**: la lista cerrada del puerto `Plataforma` suma
  `leer-permisos-actions` (identidad de persona).
- **F2-6**: el CLI lee el remoto (`fetch`, `comprobar-id`,
  `actualizar-principal`) con la identidad del agente.
- **F1-8** cubre el modelo de amenazas de las credenciales: clave de la App
  del agente en local, secreto del revisor en Actions, clave de la persona
  frente a un agente en la misma sesión, qué workflow puede leer qué
  secreto, y los workflows que se puedan lanzar con `repository_dispatch`
  (`POST /dispatches`, alcanzable por el agente con `contents:write`).
- **F1-10** (además de lo de abajo): renombrar, mover o borrar ramas no
  protegidas y borrar releases, que el agente alcanza; qué ramas protege un
  ruleset aparte de las de F1-5.
- **F1-10** decide quién lanza una release y qué pasa con los tags que el
  agente puede crear.
- **F1-13** y **F6-1** verifican la correspondencia en GitLab.
- **F2-6** implementa el proveedor de credenciales por rol del puerto
  `Plataforma` y prueba en E2E lo marcado «no verificado».
- **F2-13** usa la lista de personas como identidad de quién lleva cada
  tarea.
- **F3-1** (`init`) guía la creación e instalación de las dos Apps y el
  registro de la clave de firma; **F3-8** (`doctor`) comprueba permisos e
  instalación y, con el token de la persona, `can_approve_pull_request_reviews`
  (`leer-permisos-actions`).
- **F4-2** (`approve`) firma con la clave de la persona y escribe el trailer
  `Plan-Blob`; sin firma o sin TTY, rechaza.
- **F6** (y **F6-1**) crea las Apps, el workflow del revisor y la salida de
  hallazgos en el check (el revisor no publica reviews ni comenta en el PR);
  la fila F6-1 del plan se actualiza («checks», no «reviews y checks»).
- **Este repo no cambia** (RR-14): sigue sin Apps y con la cuenta de la
  persona hasta el relevo.

## Fuera de alcance

- Modelo de amenazas, qué corre en local y en CI, `pull_request_target`:
  F1-8.
- Quién lanza una release, política por carril, si con varias personas el
  riesgo alto exige dos distintas: F1-10.
- Equivalencias en GitLab y planes de pago: F1-13.
- Creación real de las Apps, nombres, manifiestos e instalación: F6-1, F3-1.
- Implementación y E2E: F2-6, F2-13, F4-2, F6, F9-4.

## Preguntas abiertas

Este ADR se planificó con un solo rol (arquitectura). Quedan sin cubrir,
como riesgos sin prueba:

- **Aprobaciones de Apps y `required_approving_review_count`**: si GitHub
  cuenta una review `APPROVE` del agente en un PR ajeno, la puerta de la
  plataforma no la distingue y solo queda `carril`. No verificado (F2-6).
- **Merge por el agente con todo en verde**: el límite de la decisión 4; si
  se quiere cerrar en la plataforma, la vía es el fork (D3b) o la cola en
  organización. No verificado que la API lo permita.
- **Tags `v*` y releases creados por el agente**: el ruleset de tags impide
  borrar y mover, no crear; una regla `creation` en tags sin bypass impediría
  también a la persona. Lo decide F1-10.
- **Claves en la misma máquina** (RN-06): la clave de la App del agente y la
  de firma de la persona conviven; si la persona carga su clave en un
  `ssh-agent` o un llavero desbloqueado, el agente puede firmar como ella.
  FIDO con toque o frase sin caché lo contienen; no verificado con una llave
  real (F1-8).
- **Una tarea legítima que toca `.github/workflows/`**: el agente no puede
  empujarla; la persona tiene que hacerlo con su identidad, y entonces
  `require_last_push_approval` le impide aprobar su propio último push. Con
  una sola persona, no hay salida limpia; F1-8 y F1-10.
- **Firma de los commits del agente** (RN-01 dice «firma commits»): aquí no
  se exige; la firma que cuenta es la de la persona en `approve`. Si se
  quiere, la App puede crear commits por la API (los firma GitHub) o el agente
  firmar con una clave propia; lo decide F6-1.
- **Qué cuenta devuelve la API como `committer` de un commit verificado** y
  si otra identidad con el correo de la persona puede obtener `verified`
  (heredada de F1-5). No verificado; la comparación por `id` cubre el
  `login` renombrado, no este caso.
- **Con varias personas**: si el riesgo alto exige dos personas distintas y
  quién mantiene la lista de personas (F1-10).
- **GitLab**: si un token de proyecto puede quedar fuera de los aprobadores y
  publicar un estado que la regla de merge exija (F1-13).

## Adenda (F4-2, TASK-037) — `tc2 approve` firmado con `Plan-Blob`

`tc2 approve [--json] TASK-NNN` implementa la consecuencia de F4-2 de este
ADR. Lo que fija la implementación, con su evidencia en
`crates/tc2/tests/aprobar.rs` y los unitarios de `crates/tc2/src/plan/`:

- **Identidad**: solo `Quien::Persona` (terminal, `GET /user` con el token de
  persona y su `id` en la lista de personas de `origin/develop`). Con el
  agente o con cualquier rechazo de la identidad, `approve rechazada
  motivo=identidad rol=persona` antes de lanzar git (ni `fetch`, ni
  transacción, ni `commit-tree`).
- **Firma**: el commit de `approve` sale de `git commit-tree -S` (la única
  forma firmada del puerto, adenda de TASK-037 en F1-4) con `gpg.format` y
  `user.signingkey` de la configuración de la persona, y lleva el trailer
  `Plan-Blob: <blob>` con el blob de `TASK-NNN/planificacion/plan-final.md`
  en el árbol de ese mismo commit (`rev-parse <árbol>:<ruta>`). Si la firma
  falla, `rechazada motivo=firma` y no se publica nada. En cada reintento
  del CAS se vuelve a firmar. Las pendientes no entran en el commit firmado
  (D4a): se publican con la siguiente operación normal.
- **Medido en Windows (Git 2.56.0.windows.4)**: `commit-tree -S` firma con
  `gpg.format=ssh` y una clave ed25519 desechable sin frase, y `git
  verify-commit` la valida con `gpg.ssh.allowedSignersFile`. `commit-tree`
  **no** respeta `commit.gpgSign=true` (es plumbing): ninguna otra operación
  empieza a firmar por la configuración de la persona, y `approve` pasa `-S`
  explícito. Sin `user.signingkey` git sale con 128; con una clave que no
  existe, o con frase de paso y sin terminal, con 1 (los tres, `motivo=firma`).
- **Terminal**: la entrada estándar de `commit-tree -S` se hereda, para que
  `ssh-keygen` pueda pedir la frase de paso; stderr se captura (git ya
  captura el de `ssh-keygen`, así que heredarlo no enseñaría el aviso FIDO) y
  su primera línea va al diagnóstico. **No verificado**: la petición de la
  frase de paso y el toque FIDO con una persona real en el terminal de
  Windows.
- **Ya aprobado**: `sin-cambios`, sin firmar otra vez. El esqueleto de
  `plan-final.md` no se aprueba (`plan-esqueleto`); tampoco sin plan
  (`sin-plan`), en pausa (`pausada`), fuera de `en-diseno`
  (`estado-no-valido`) o de otra persona (`de-otra-persona`).
- La comprobación de la firma frente a la plataforma (reglas 1 a 5 de la
  decisión 3) sigue siendo del check `carril` (F6), no de `approve` (D3a).

## Adenda (F4-3, TASK-038) — `tc2 approve --hito <n>`: N trailers en un commit

- **Forma** (D1a): `tc2 approve [--json] --hito <n>` (`n` es el campo
  `sprint`; vale `4` o `H4`). Aprueba de una vez los planes de las tareas
  abiertas del sprint (ni `terminada` ni `descartada`) que aún no lo están,
  con las mismas comprobaciones que `approve TASK-NNN` y la misma regla de
  identidad: con el agente o sin identidad, `approve rechazada hito=<n>
  motivo=identidad rol=persona` antes de lanzar git. Con un ID, con
  `--pedir-cambios` o con un hito que no es un número, `uso`.
- **Un commit firmado con N trailers** (D3a): el lote entero va en un único
  `commit-tree -S` de la persona, con una línea `Plan-Blob: <blob>` por
  tarea, en orden de ID, cada una con el blob de su
  `TASK-NNN/planificacion/plan-final.md` en el árbol de ese mismo commit, y
  la línea `Operacion: approve-hito|<n>|<id>|<login>`. El trailer no lleva
  el ID de la tarea: dos planes idénticos comparten blob (el commit solo
  cambia los `tarea.md` del lote y `ROADMAP.md`).
- **Todo o nada** (D2a): si alguna tarea abierta del hito no se puede
  aprobar (también una `planificada` sin plan), no se escribe ni se firma
  nada: `rechazada motivo=no-listas tareas=ID:motivo,...` con los motivos
  de F4-2. Sin ninguna abierta, `hito-vacio`; con más de 20 por aprobar,
  `demasiadas max=20` (D5a: cuentan solo las que cambiarían); todas ya
  aprobadas, `sin-cambios` sin firmar.
- **Nombre y fecha** (D4a): cada tarea recibe la fila `| <instante> |
  approve | <modo congelado> | persona | <versión> |` (el mismo instante
  para todo el lote) y `actualizado`; el nombre está en la firma y en la
  línea `Operacion:`.
- **Regla 5 del contrato del firmante** (decisión 3): cuando el check
  `carril` de F6 la implemente pasará a «**el blob actual de `plan-final.md`
  está entre los trailers `Plan-Blob` del commit que pone `plan_aprobado:
  true`**», en vez de «el trailer». Un commit de `approve TASK-NNN` tiene
  uno; uno de `approve --hito`, varios. No se implementa aquí.
- **Divergencia con el plan de TASK-038**: el plan suponía que la
  transacción pide `sello()` después de `aplicar`; la pedía antes (para
  decidir las pendientes). Como el lote solo se conoce en `aplicar`, la
  transacción ahora vuelve a pedir el sello después de `aplicar` en cada
  intento del CAS para construir el commit; el de antes solo decide si
  entran las pendientes. Las operaciones que ya existían devuelven el mismo
  sello las dos veces.
- **Medido** (`tests/aprobar.rs`, Git de Windows, clave ed25519 desechable):
  `approve --hito 4` con 10 tareas deja un solo commit nuevo, `git
  verify-commit` lo valida y sus 10 `Plan-Blob` son los `rev-parse
  <commit>:TASK-NNN/planificacion/plan-final.md`. **No verificado**: la
  firma con una persona real y FIDO en Windows (como en F4-2).

## Adenda (F6-1, TASK-047) — identidad propia del agente

Implementa la decisión 1 (proveedor de credenciales por rol), la tabla 2
(permisos de cada App) y la correspondencia de GitLab (decisión 6) en
`crates/tc2/src/plataforma/` (`credenciales.rs`, `app_github.rs`,
`entorno_git.rs`). No hay módulo de identidades nuevo ni comando nuevo.

- **Fábrica de producción** `plataforma::de_entorno()`: por rol, una GitHub
  App si están `TC2_APP_<ROL>_ID` y `TC2_APP_<ROL>_CLAVE` (ruta al PEM;
  `<ROL>` es `AGENTE` o `REVISOR`); si no, el token de `TC2_TOKEN_<ROL>`. La
  App gana al token del mismo rol y nunca se pasa de un rol a otro; una App a
  medias (id sin clave, o al revés) es un fallo, no se cae al token. La
  persona no tiene App y su token sigue usándose solo con terminal
  (decisión 5). Sustituye a `CredencialesEntorno` en los cuatro sitios que lo
  creaban (`lib.rs` ×2, `init`, `identidad::conectar_produccion`).
- **Token de la App** (`CredencialesApp`): JWT RS256 firmado con `ring`
  (`iss` = id de la App, `iat` 60 s en el pasado, `exp` a 9 min), instalación
  con `GET /repos/{o}/{r}/installation` (una vez por proceso) y token con
  `POST /app/installations/{id}/access_tokens` con `repositories: [<repo>]` y
  los `permissions` exactos de la tabla 2 para el rol: agente
  `contents:write`, `pull_requests:write`, `checks:read`, `statuses:read`,
  `metadata:read`; revisor `checks:write`, `contents:read`,
  `pull_requests:read`, `metadata:read`. Se reutiliza hasta `expires_at`
  menos 5 min y luego se pide otro. Habla por el mismo `http::Cliente` y la
  misma `Base` que el adaptador.
- **Fallo al acuñar ≠ falta de identidad**: el trait gana `obtener(rol) ->
  Result<Secreto, FalloCredencial>`; `Conexion::token` lo usa. Falta →
  `rechazada motivo=identidad falta=<rol>` (como antes); una App que no
  consigue su token → `denegada motivo=credenciales rol=<rol> causa=<causa>`
  (código 8), con `causa` entre `app-incompleta`, `app-solo-github`,
  `clave-ilegible`, `jwt-rechazado` (401), `sin-instalacion` (404),
  `permisos-no-concedidos` (422, o un 201 cuyos `permissions` no cubren los
  pedidos), `tls` (el diagnóstico manda al almacén de certificados) y
  `respuesta-inesperada` (con un detalle legible, nunca el `Debug` del
  fallo); sin red, `sinred`. El diagnóstico nombra las variables de la App,
  nunca el token. El id numérico de la App no admite ceros a la izquierda
  (`app-incompleta`) y el PEM admite BOM UTF-8 y texto antes de
  `-----BEGIN` (ronda 1 de revisión).
- **GitLab sin código nuevo**: el agente es una cuenta de servicio (o un
  token de proyecto) con rol Developer y su token va en `TC2_TOKEN_AGENTE`;
  el adaptador ya trata `bot: true` como `Bot`. Con una App configurada y un
  destino GitLab se usa `TC2_TOKEN_<ROL>` del mismo rol (nunca de otro); si
  no lo hay, `causa=app-solo-github` y el diagnóstico dice que en GitLab la
  App no sirve y que hay que definir `TC2_TOKEN_<ROL>`.
- **Git del agente** (D3 del plan: solo el punto de extensión; conectarlo a
  `review` es de F6-2): `plataforma::entorno_git(proveedor, url, token,
  autor)` devuelve `GIT_CONFIG_COUNT/KEY_0/VALUE_0` con
  `http.<url>.extraheader = Authorization: Basic <base64(usuario:token)>`
  (`x-access-token` en GitHub, `oauth2` en GitLab), limitado a la URL del
  remoto, y `GIT_AUTHOR_*`/`GIT_COMMITTER_*` con `<slug>[bot]` y
  `<id>+<slug>[bot]@users.noreply.github.com`. Se compone con
  `Git::con_entorno_extra` (que, como `con_entorno`, acumula: las entradas
  `GIT_CONFIG_*` se renumeran tras las del puerto y las del proceso, sin
  pisarlas): el token no pasa por la línea de órdenes, ni por
  el registro del puerto, ni por los argumentos de `git-remote-http`
  (comprobado con `GIT_TRACE`). `git` no depende de `plataforma`.
- **`identidad(Rol::Agente)` en GitHub** hace `GET /user`, que no funciona
  con un token de instalación (es un endpoint de usuario). Hoy solo se llama
  con `Rol::Persona`; quien necesite la cuenta del agente usa
  `CredencialesApp::slug()` (`GET /app` con el JWT) y la cuenta
  `<slug>[bot]`. En GitLab, `GET /user` sí funciona con la cuenta de
  servicio.
- **D4 — firma de los commits del agente**: no se exige. Los commits del
  agente salen sin firma (sin «Verified»); la firma que cuenta sigue siendo
  la de la persona en `approve` (decisión 3). Responde la pregunta abierta
  «Firma de los commits del agente».
- **Dependencias**: `ring` 0.17.14 y `base64` 0.23.1, directas en la misma
  versión que ya tenía `Cargo.lock` (por rustls y ureq); ningún crate nuevo.

**Evidencia** (`crates/tc2/tests/identidades.rs` y unitarios de
`credenciales.rs` y `app_github.rs`; clave RSA desechable generada en cada
test con `ssh-keygen -t rsa -m PEM`, ninguna clave en el repo):

- El JWT de cada petición se verifica con la clave pública que exporta
  `ssh-keygen -e -m PEM` (no con el código que firma); `iss` numérico,
  `exp - iat` ≤ 600 s.
- El cuerpo de `access_tokens` es exactamente `{"repositories":["r"],
  "permissions":{…}}` con la tabla 2 del rol (agente y revisor); la
  instalación se busca una vez; un token ya caducado se renueva y uno vigente
  se reutiliza (3 llamadas a `obtener` → 1 `GET` + 2 `POST`).
- Por la fábrica, `prs-abiertos` con la App lleva `Bearer <token de
  instalación>` y no el de `TC2_TOKEN_AGENTE`; con la App sin instalar,
  `prs-abiertos denegada motivo=credenciales rol=agente
  causa=sin-instalacion` (8), sin probar con el token de la variable.
- Con un repo git real, el commit sale con autor y committer `[bot]`, el
  push manda `Authorization: Basic …` al remoto HTTP simulado y el token no
  aparece en el registro del puerto ni en `GIT_TRACE`.
- **Mutaciones**: dar `pull_requests:write` al revisor pone en rojo el
  unitario de permisos y el de integración; pasar la credencial a la URL con
  `url.<url-con-token>.insteadOf` pone en rojo el test de git (por la
  cabecera ausente y, con la cabecera mantenida, por el token en `GIT_TRACE`).

**Divergencias con el plan de TASK-047**:

1. **`de_entorno()` sin argumentos y `atar_github(base, repo)` en el
   trait.** El plan pedía `de_entorno(base, repo)`, pero dos de los cuatro
   sitios (`lib.rs`) crean las credenciales antes de conocer el destino. La
   App se ata al repo cuando `GitHub::nuevo` recibe el proveedor; por
   defecto `atar_github` no hace nada.
2. **`tiene(rol)` nuevo en el trait**: elegir con qué rol se pregunta
   (`identidad::conectar_produccion`) o si hay persona (`init`) sin hablar con
   la red; con App, cuenta aunque el token se acuñe después.
3. **Entorno git en el puerto `Git`, no como parámetro de `git::remoto`**:
   el autor tiene que valer también para el commit (`guardar`), no solo para
   el push. `Git::con_entorno_extra` añade variables a las que ya haya.
4. **Cabecera básica limitada a la URL del remoto** (`http.<url>.extraheader`)
   en vez de una cabecera global.
5. **Correo del autor**: con el id de la cuenta `[bot]` cuando se conoce;
   el `Cliente` no puede leerlo (`/users/<slug>[bot]` lleva `[`, que la
   validación de rutas rechaza), así que lo pasa quien llama (el E2E lo lee
   con el arnés); sin id, `<slug>[bot]@users.noreply.github.com`.
6. **PKCS#8 además de PKCS#1**: `RsaKeyPair::from_der` lee el DER PKCS#1 que
   da GitHub (y `ssh-keygen -m PEM`); el PEM se decodifica a mano (base64 sin
   cabeceras; uno cifrado se rechaza) y `BEGIN PRIVATE KEY` va por
   `from_pkcs8`.

**Suposiciones del plan**:

- `RsaKeyPair::from_der` acepta el PEM PKCS#1 (una vez decodificado a DER):
  **verificado** (test de integración).
- `access_tokens` admite `permissions` y `repositories`: lo dice la
  documentación de GitHub y el test comprueba lo que se manda; **contra
  GitHub real, no verificado** hasta ejecutar el E2E.
- Tier del proyecto de GitLab y `bot: true` de la cuenta de servicio: **no
  verificado**; el E2E lo afirma.

**E2E ignorados, no ejecutados** (necesitan la App de D1 y la cuenta de
servicio de D2): `persona_aprueba_pr_del_agente` en
`tests/plataforma_e2e.rs` (variables `TC2_E2E_REPO`, `TC2_E2E_TOKEN`,
`TC2_E2E_APP_ID`, `TC2_E2E_APP_CLAVE`) y `persona_aprueba_mr_del_agente` en
`tests/plataforma_gitlab_e2e.rs` (`TC2_E2E_GITLAB_PROYECTO`,
`TC2_E2E_GITLAB_TOKEN`, `TC2_E2E_GITLAB_TOKEN_AGENTE`). Los dos empujan por
git con el token del agente y sin la configuración de git del usuario (así
cubren también «`fetch` por git con un token de instalación»), abren el
PR/MR con el adaptador, intentan que el agente apruebe el suyo (en GitHub
tiene que fallar: cierra «que GitHub impida a una App aprobar su propio PR»;
en GitLab, si lo deja, no cuenta) y la persona aprueba hasta
`aprobaciones=1`. Hasta ejecutarlos, esos puntos de «No verificado» siguen
abiertos.
