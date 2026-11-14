# ADR F1-6 — Identidades del agente implementador, del revisor IA y de la persona

- Estado: **aceptada** (2026-11-14)
- Tarea: TASK-007 · Plan: [F1-6 en PLAN-TASKCODE-2](../plan/PLAN-TASKCODE-2.md) (§4 P3 y tabla de la Fase 1)
- Requisitos: RR-04, RR-08, RR-12, RR-15, RF-30, RF-32, RF-51, RF-64, RN-01 y RN-06 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); H5 y H6 de la [auditoría externa](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md)
- Depende de: [ADR F1-4](F1-4-operaciones-git-y-contrato-de-salida.md) (puerto `Plataforma` y su lista cerrada) y [ADR F1-5](F1-5-puertas-en-github.md) (check `carril`, firma verificada del `approve` atada al blob con el trailer `Plan-Blob`, `integration_id` de `revision-ia`, CODEOWNERS, `strict`, sin `bypass_actors`); nombres de rama y tags de [ADR F1-3](F1-3-flujo-de-ramas-worktrees-release.md)
- Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a —FIDO si hay llave; si no, D4b—, D5a y D6a del plan de TASK-007)

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
`%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-007\poc\` (borrada al
terminar); cada descarga va a su propia carpeta vacía (`esquema/`, `docs/`) y
las claves desechables a `claves/`, nunca al repo. **Solo lectura en
GitHub**: toda llamada pasa por una función con `gh api --method GET`, y el
guion las registra y cuenta las de otro método.

```bash
bash poc-f1-6.sh "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-007/poc" > poc-f1-6.log 2>&1
```

```text
RESULTADO llamadas gh api con GET=10 ; gh api con otro metodo=0 ; gh repo view=2 ; descargas GET=3
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

### A — Permisos de cada identidad contra `app-permissions`

```text
app-permissions: 57 permisos; additionalProperties=(no declarado)
  contents        enum=['read', 'write']
  workflows       enum=['write']
agente   contents:write, pull_requests:write, metadata:read             -> VALIDO
revisor  checks:write, contents:read, pull_requests:read, metadata:read -> VALIDO
  permiso inventado approvals:write             -> rechazado: permiso desconocido approvals
  permiso inventado merge:write                 -> rechazado: permiso desconocido merge
  valor inventado contents:admin                -> rechazado: contents='admin' fuera del enum ['read', 'write']
  workflows:read (el enum solo admite write)    -> rechazado: workflows='read' fuera del enum ['write']
RESULTADO A identidades=2 errores=0 ; negativos rechazados=4/4
```

No existe un permiso «aprobar» ni «mergear» que se pueda quitar aparte: se
aprueba con `pull_requests` y se mergea con `contents` (B).

### B — Endpoints que alcanza cada identidad

La descripción OpenAPI **no declara permisos por endpoint**: `x-github` solo
trae `category`, `subcategory`, `enabledForGitHubApps`, `githubCloudOnly` y
avisos de obsolescencia. La fuente es la tabla de `github/docs`. Una
identidad «alcanza» un endpoint si tiene alguno de los permisos que la tabla
le asigna con nivel suficiente (cota superior: la tabla marca algunos con
`additional-permissions`); «no alcanza» es firme, porque ninguno de sus
permisos aparece para ese endpoint.

```text
-- tabla fpt-2022-11-28: 940 endpoints server-to-server
  crear o mover rama (equivale a push)   POST  /git/refs                        exige=contents:write,workflows:write  agente=si revisor=no
  abrir-pr                               POST  /pulls                           exige=pull_requests:write             agente=si revisor=no
  editar el PR (titulo, descripcion)     PATCH /pulls/{pull_number}             exige=pull_requests:write             agente=si revisor=no
  actualizar rama del PR (boton)         PUT   /pulls/{pull_number}/update-branch exige=pull_requests:write           agente=si revisor=no
  merge-pr                               PUT   /pulls/{pull_number}/merge       exige=contents:write                  agente=si revisor=no (limite)
  merge directo de ramas                 POST  /merges                          exige=contents:write                  agente=si revisor=no (limite)
  review (tambien APPROVE)               POST  /pulls/{pull_number}/reviews     exige=pull_requests:write             agente=si revisor=no (limite)
  descartar una review                   PUT   /pulls/{n}/reviews/{id}/dismissals exige=pull_requests:write           agente=si revisor=no (limite)
  check revision-ia                      POST  /check-runs                      exige=checks:write                    agente=no revisor=si
  estado de commit (statuses)            POST  /statuses/{sha}                  exige=statuses:write                  agente=no revisor=no
  aplicar-puertas (crear ruleset)        POST  /rulesets                        exige=administration:write            agente=no revisor=no
  permisos de workflows del repo         PUT   /actions/permissions/workflow    exige=administration:write            agente=no revisor=no
  secreto de Actions                     PUT   /actions/secrets/{secret_name}   exige=secrets:write                   agente=no revisor=no
  fichero de .github/workflows/ por API  PUT   /contents/{path} [workflow]      exige=contents:write,workflows:write  agente=no revisor=no
  tag (objeto anotado)                   POST  /git/tags                        exige=contents:write                  agente=si revisor=no (limite)
  release                                POST  /releases                        exige=contents:write,workflows:write  agente=si revisor=no (limite)
  leer commit y su firma                 GET   /commits/{ref}                   exige=contents:read                   agente=si revisor=si
  leer-puertas (reglas de la rama)       GET   /rules/branches/{branch}         exige=metadata:read                   agente=si revisor=si
  comentar en el PR                      POST  /issues/{issue_number}/comments  exige=issues:write,pull_requests:write agente=si revisor=no
  ... (23 filas; completas en el log)
las 2 versiones dan el mismo resultado en las 23 filas: si
minimo agente   usados=contents,pull_requests ; sin uso=ninguno
minimo revisor  usados=checks,contents,pull_requests ; sin uso=ninguno
RESULTADO B filas=23 x versiones=2 fallos=0 ; versiones iguales=si ; permisos sin uso=0
```

Lectura:

- **El revisor no alcanza ninguna escritura** salvo los checks: ni reviews
  (tampoco `APPROVE`), ni merge, ni refs, ni `contents`, ni rulesets, ni
  workflows. Tampoco puede comentar en el PR: sus hallazgos van en la salida
  del check (resumen y anotaciones).
- **El agente no alcanza** rulesets, permisos de workflows, colaboradores,
  secretos, checks ni statuses, ni ficheros de `.github/workflows/` por la
  API.
- **El agente sí alcanza**, con `contents:write` y `pull_requests:write`:
  `PUT /pulls/{n}/merge`, `POST /merges`, reviews (incluido `APPROVE` en un PR
  que no sea suyo), descartar reviews, tags y releases. Son los límites que la
  decisión 4 deja escritos.
- **Mínimo**: cada permiso de cada App lo usa al menos una operación que esa
  App tiene asignada (`metadata:read` es obligatorio en toda App).
- La descripción de `pull_requests` en el esquema habla de «merges», pero la
  tabla exige `contents:write` (y solo ese) para `PUT /pulls/{n}/merge`.

### C — Permisos de los workflows y tipo de cuenta (solo `GET`)

```text
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
- **`web-flow` es `type: User`**, no `Bot`: el tipo de cuenta no basta para
  excluir la firma de la propia plataforma; la exclusión tiene que ser
  explícita (decisión 3).

### D — Contrato del firmante y controles negativos

Simulación en Python con los `type` reales de C.

```text
  firmante: persona verificada en la lista                 -> cuenta
  firmante: Bot verificado (github-actions[bot])           -> no cuenta
  firmante: Bot metido en la lista por error               -> no cuenta
  firmante: persona verificada fuera de la lista           -> no cuenta
  firmante: web-flow (firma de la plataforma)              -> no cuenta
  firmante: web-flow metido en la lista por error          -> no cuenta
  firmante: persona sin firma                              -> no cuenta
  firmante: firmado, correo sin cuenta (committer null)    -> no cuenta
  invariantes sobre la tabla del ADR: se cumplen
  negativo revisor con pull_requests:write    -> detectado: el revisor puede crear reviews (APPROVE)
  negativo revisor con contents:write         -> detectado: el revisor puede mergear
  negativo agente con administration:write    -> detectado: agente administra rulesets
  negativo agente con workflows:write         -> detectado: agente tiene workflows
  negativo agente con checks:write            -> detectado: el agente puede publicar checks (revision-ia)
RESULTADO D firmante 8/8 casos ; invariantes=ok ; negativos detectados=5/5
```

La simulación comprueba la lógica del contrato, no GitHub.

### E — Firma SSH local con claves desechables

Un repo local hace de `taskcode/tareas`: el agente escribe
`plan-final.md` sin firmar; la persona crea el commit de `approve` firmado
con su clave SSH y el trailer `Plan-Blob: <sha del blob>`.

```text
commit de approve ab86416 ; firma=G ; trailer Plan-Blob=1608d7cd2379f7dcb0576f1981d650e21d2304e6
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
   dos. Se añade a lo que comprueba `leer-puertas` (decisión 4).
5. **El agente puede crear tags y releases** con `contents:write` (PoC B):
   el ruleset de tags de F1-5 impide borrarlos o moverlos, no crearlos. Queda
   como límite escrito y pregunta abierta (F1-10).
6. **El revisor no puede comentar en el PR** con estos permisos (PoC B): sus
   hallazgos van en la salida del check `revision-ia`, no en comentarios.
   RF-32 habla de «JSON + comentarios»; lo concreta F6.
7. **`taskctl doctor`** del plan de pruebas no se ejecuta en esta
   implementación (el orquestador lo reserva); `comprobar-repo.sh` sí.

### No verificado

- **Permisos efectivos de una App real**: no se ha creado ninguna (D1a); el
  `app_id` del revisor sigue sin valor y los JSON de F1-5 conservan
  `revision-ia@0`. Lo cierran **F6-1** y **F2-6**.
- **Que la plataforma no cuente la aprobación de una App** para
  `required_approving_review_count`: la tabla dice que el agente puede crear
  una review `APPROVE` en un PR ajeno; si GitHub la cuenta, solo la contiene
  `carril` (que solo cuenta personas).
- **Que `contents:write` baste para `PUT /pulls/{n}/merge` con todo en
  verde** y que un ruleset sin bypass lo bloquee con algo pendiente: según la
  tabla, sí; no ejecutado.
- **Que `git push` de un cambio en `.github/workflows/` falle sin
  `workflows`**: la tabla cubre la API REST (y lo confirma para
  `PUT /contents`); para `git push` es documentación de GitHub, sin probar.
- **La marca «Verified» de GitHub** en un commit de `approve` firmado por la
  persona con su clave registrada, y qué cuenta devuelve la API como
  `committer`: PoC E prueba el mecanismo con `allowed_signers` local.
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
| Settings → GitHub Apps (instaladas) | dos Apps, `taskcode-agente` y `taskcode-revisor` (nombres de ejemplo), instaladas solo en los repos del proyecto, cada una con sus permisos: la del agente «Contents: Read and write, Pull requests: Read and write, Metadata: Read-only»; la del revisor «Checks: Read and write, Contents: Read-only, Pull requests: Read-only, Metadata: Read-only» |
| Lista de PR | autor `taskcode-agente[bot]`, con la etiqueta «bot» |
| Commits de la rama de la tarea | empujados por el agente, con su cuenta `[bot]` o sin cuenta asociada; sin «Verified» (no se exige) |
| Checks del PR | `revision-ia` publicado por `taskcode-revisor`, con sus hallazgos en el resumen y las anotaciones; `verificacion` y `carril` por GitHub Actions |
| Caja de merge | «Review required» hasta que la persona aprueba; una review del agente o del revisor no aparece (el revisor no puede crearla; la del agente no cuenta para `carril`) |
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
   | Operación (F1-4, F1-5) | Comando dueño | Identidad | Por qué |
   |---|---|---|---|
   | `push` (ramas de tarea) | `review`, `actualizar-rama` | agente | `require_last_push_approval` de F1-5: si empuja la persona, no puede aprobar |
   | `actualizar-rama` (merge de la base + `push`) | `actualizar-rama`, `finish` | agente | ídem: el último push tiene que ser del agente |
   | `commit-estado` (push a `taskcode/tareas`) | todos los de fase | agente | push directo sin PR (F1-1, F1-2) |
   | `commit-estado` del `approve` | `approve` (F4-2) | persona, **con su firma** | la firma verificada es la aprobación del plan (F1-5, decisión 4) |
   | `abrir-pr` (también el de release y el de vuelta del hotfix) | `review`, `release`, `hotfix` | agente | la persona tiene que poder aprobarlo |
   | `estado-puertas` | `review`, `finish` | cualquiera | lectura |
   | `merge-pr` | `finish`, `release` | persona | ninguna IA mergea (decisión 4) |
   | `tag-anotado` (y su push) | `release`, `finish` de `hotfix` | persona | publicar una versión es de la persona (quién lanza la release: F1-10) |
   | `aplicar-puertas` | `init` | persona | exige `administration`, que ninguna IA tiene |
   | `leer-puertas` | `doctor`, `review` | cualquiera | lectura |
   | check `revision-ia` | workflow del revisor (F6) | revisor | su `app_id` es el `integration_id` de `revision-ia` |
   <!-- tabla-identidades:fin -->

   Las operaciones solo locales de F1-4 (`fetch`, `guardar`,
   `crear-rama-y-worktree`, …) no hablan con la plataforma: corren con Git y
   la configuración local; `fetch` y `push` usan el token del rol que diga la
   tabla.

2. **Permisos mínimos por identidad** (nombres de permiso de GitHub Apps,
   validados contra `app-permissions`, PoC A; alcance por endpoint, PoC B):

   | Identidad | Forma | Permisos | Sin | Credencial |
   |---|---|---|---|---|
   | Agente implementador | GitHub App (D2a) | `contents:write`, `pull_requests:write`, `metadata:read` | `administration`, `workflows`, `checks`, `statuses`, `secrets`, `issues` | clave privada de la App en la máquina del agente; tokens de instalación de corta vida que pide el CLI |
   | Revisor IA | otra GitHub App (D2a) | `checks:write`, `contents:read`, `pull_requests:read`, `metadata:read` | cualquier otra escritura | clave privada como secreto de entorno de Actions (D5a); fuera de la máquina del agente |
   | Persona | su cuenta de usuario | su rol en el repo: admin para `aplicar-puertas`, escritura para `merge-pr` | — | token propio, usado solo con TTY (H6); clave de firma registrada en su cuenta, FIDO `ed25519-sk` si tiene llave (D4a), si no SSH o GPG con frase de paso y sin caché en el agente de claves (D4b) |

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
      en `taskcode/tareas` (F1-5).

   La **lista de personas** es un fichero en `.taskcode/` de la rama de código
   (D6a), bajo CODEOWNERS (F1-5, decisión 6): cambiarla exige un PR aprobado
   por una persona. Nunca en `taskcode/tareas`. Un Bot o `web-flow` metidos en
   la lista por error siguen sin contar (PoC D). Es la misma lista que da a
   F2-13 la identidad de persona de cada tarea (RR-12).

4. **«Ninguna IA aprueba ni mergea»**, en tres capas:
   1. **El revisor, por permisos**: sin `pull_requests:write` no crea reviews
      (tampoco `APPROVE`); sin `contents:write` no mergea ni empuja (PoC B).
      Su única escritura es el check.
   2. **El agente, por `carril` y por la plataforma**: no puede aprobar su
      propio PR (regla de GitHub); una aprobación suya en un PR ajeno no la
      cuenta `carril`, que solo cuenta personas (contrato de la decisión 3);
      la aprobación del plan exige la firma de la persona (PoC E).
   3. **El merge, por `merge-pr`**: solo corre con la identidad de persona; y
      los rulesets sin `bypass_actors` (F1-5) impiden integrar en `develop` o
      `main` sin la aprobación de la persona y los checks en verde, sea quien
      sea quien pulse.

   Además, `leer-puertas` (y `doctor`) comprueba que
   `can_approve_pull_request_reviews` siga en `false` (PoC C): si no, un
   workflow con `GITHUB_TOKEN` podría aprobar.

   **Límite explícito (D3a).** GitHub no separa empujar de mergear:
   `contents:write`, que el agente necesita para empujar ramas y
   `taskcode/tareas`, incluye `PUT /pulls/{n}/merge`, `POST /merges`, tags y
   releases; `pull_requests:write`, que necesita para abrir PR, incluye crear
   y descartar reviews en PR ajenos (PoC B). El agente **técnicamente puede
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
| P3 / RR-08 las garantías las impone la plataforma | permisos mínimos por App; revisor sin escritura salvo checks; agente sin `administration` ni `workflows` | PoC A (permisos válidos, 4/4 inventados rechazados), B (23 filas × 2 versiones, 0 fallos); permisos efectivos de una App real: no verificado (F6-1) |
| RR-04 / RF-30 revisor independiente con identidad propia; aprobación de persona | revisor = otra App con `checks:write`; su `app_id` = `integration_id` de `revision-ia`; aprobación solo de personas | PoC B (revisor alcanza `POST /check-runs` y nada más que escriba), D (invariantes; 5/5 tablas alteradas detectadas) |
| Criterio de F1-6: que ninguna IA pueda aprobar ni mergear | tres capas y límite explícito (decisión 4) | PoC B, C (`can_approve_pull_request_reviews: false` en los dos repos), D, E; que GitHub no cuente la aprobación de una App: no verificado |
| Criterio de F1-6: permisos mínimos | tabla de la decisión 2 | PoC B (cada permiso usado por una operación asignada; 0 sin uso) |
| RN-01 identidad propia del agente; abre PR; la persona aprueba | agente = GitHub App; `abrir-pr` y `push` con su identidad | PoC B (`POST /pulls`, `POST /git/refs`); la firma de los commits del agente no se exige (pregunta abierta) |
| RR-12 la persona identificada por su usuario; el agente figura como agente | lista de personas en `.taskcode/`; tipo de cuenta `User` frente a `Bot` | PoC C (`type` real), D (8/8 casos); implementación: F2-13 |
| H5 la aprobación humana se puede comprobar | commit de `approve` firmado por la persona, con `Plan-Blob`; contrato del firmante | PoC E (verify-commit bueno/malo, blob cambiado y revertido, clave del agente), D; «Verified» de GitHub: no verificado (F2-6) |
| H6 sin identidad de persona no hay atajos | `merge-pr`, `aplicar-puertas` y `tag-anotado` solo con la persona; token de persona solo con TTY | decisión 1; PoC E (clave con frase sin TTY no firma); implementación: F4-2 |
| RN-06 una persona con agentes | tres identidades aunque haya una sola persona; clave FIDO o con frase para separar persona y agente en la misma máquina | PoC E; FIDO: no verificado |
| RR-15 mismas garantías en GitLab | correspondencia (decisión 6) | no verificado (F1-13, F6-1) |

## Consecuencias

- **F1-5**: los JSON de rulesets sustituyen `revision-ia@0` por el `app_id`
  de la App del revisor cuando exista (F6-1); `aplicar-puertas` sigue
  negándose con el relleno. `leer-puertas` suma la comprobación de
  `can_approve_pull_request_reviews`.
- **F1-8** cubre el modelo de amenazas de las credenciales: clave de la App
  del agente en local, secreto del revisor en Actions, clave de la persona
  frente a un agente en la misma sesión, y qué workflow puede leer qué
  secreto.
- **F1-10** decide quién lanza una release y qué pasa con los tags que el
  agente puede crear.
- **F1-13** y **F6-1** verifican la correspondencia en GitLab.
- **F2-6** implementa el proveedor de credenciales por rol del puerto
  `Plataforma` y prueba en E2E lo marcado «no verificado».
- **F2-13** usa la lista de personas como identidad de quién lleva cada
  tarea.
- **F3-1** (`init`) guía la creación e instalación de las dos Apps y el
  registro de la clave de firma; **F3-8** (`doctor`) comprueba permisos,
  instalación y `can_approve_pull_request_reviews`.
- **F4-2** (`approve`) firma con la clave de la persona y escribe el trailer
  `Plan-Blob`; sin firma o sin TTY, rechaza.
- **F6** (y **F6-1**) crea las Apps, el workflow del revisor y la salida de
  hallazgos en el check (el revisor no comenta en el PR).
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
  (heredada de F1-5). No verificado.
- **Con varias personas**: si el riesgo alto exige dos personas distintas y
  quién mantiene la lista de personas (F1-10).
- **GitLab**: si un token de proyecto puede quedar fuera de los aprobadores y
  publicar un estado que la regla de merge exija (F1-13).
