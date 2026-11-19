# Peticion de revision — TASK-047 (ronda 1)

- Tarea: TASK-047 — F6-1 Identidades: agente implementador
- Rama revisada: feature/task-047-f6-1-identidades-agente-implementador
- Rama base: develop
- Commit revisado (HEAD): 8123fa68e74571def0591f22da57cfa2536e687c
- Fecha: 2026-11-19
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-047 (criterios de aceptacion y plan)
- Alcance: completa (motivo: primera ronda de revision)

## Instrucciones para el agente revisor

Eres un revisor INDEPENDIENTE del agente que implemento. Tu trabajo es
reproducir empiricamente, no leer el diff y opinar: clona el repo a un
directorio temporal, corre la suite tu mismo (salvo que esta
peticion ya traiga la verificacion de la ronda en verde para el commit revisado;
entonces vale ese resultado) y construye el caso que
rompe el codigo antes de reportarlo. Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. Vuelca tu salida en el
informe de esta ronda (informe-revision-1-code-quality-reviewer.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-047/tarea.md
- tareas/03-en-revision/TASK-047/planificacion/
- docs/adr/F1-6-identidades-agente-revisor-persona.md
- docs/plan/PLAN-TASKCODE-2.md

## Commits a revisar (git log develop..HEAD)

````
8123fa6 docs(TASK-047): adenda F6-1 en el ADR F1-6 y fila F6-1 del plan
a04e030 feat(TASK-047): identidad propia del agente como GitHub App
6e42d6e chore(TASK-047): tarea en curso
````

## Diff de tu dominio (2 fichero(s) de 19; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/docs/adr/F1-6-identidades-agente-revisor-persona.md b/docs/adr/F1-6-identidades-agente-revisor-persona.md
index d3c2c66..1099a7d 100644
--- a/docs/adr/F1-6-identidades-agente-revisor-persona.md
+++ b/docs/adr/F1-6-identidades-agente-revisor-persona.md
@@ -729,3 +729,134 @@ ADR. Lo que fija la implementación, con su evidencia en
   verify-commit` lo valida y sus 10 `Plan-Blob` son los `rev-parse
   <commit>:TASK-NNN/planificacion/plan-final.md`. **No verificado**: la
   firma con una persona real y FIDO en Windows (como en F4-2).
+
+## Adenda (F6-1, TASK-047) — identidad propia del agente
+
+Implementa la decisión 1 (proveedor de credenciales por rol), la tabla 2
+(permisos de cada App) y la correspondencia de GitLab (decisión 6) en
+`crates/tc2/src/plataforma/` (`credenciales.rs`, `app_github.rs`,
+`entorno_git.rs`). No hay módulo de identidades nuevo ni comando nuevo.
+
+- **Fábrica de producción** `plataforma::de_entorno()`: por rol, una GitHub
+  App si están `TC2_APP_<ROL>_ID` y `TC2_APP_<ROL>_CLAVE` (ruta al PEM;
+  `<ROL>` es `AGENTE` o `REVISOR`); si no, el token de `TC2_TOKEN_<ROL>`. La
+  App gana al token del mismo rol y nunca se pasa de un rol a otro; una App a
+  medias (id sin clave, o al revés) es un fallo, no se cae al token. La
+  persona no tiene App y su token sigue usándose solo con terminal
+  (decisión 5). Sustituye a `CredencialesEntorno` en los cuatro sitios que lo
+  creaban (`lib.rs` ×2, `init`, `identidad::conectar_produccion`).
+- **Token de la App** (`CredencialesApp`): JWT RS256 firmado con `ring`
+  (`iss` = id de la App, `iat` 60 s en el pasado, `exp` a 9 min), instalación
+  con `GET /repos/{o}/{r}/installation` (una vez por proceso) y token con
+  `POST /app/installations/{id}/access_tokens` con `repositories: [<repo>]` y
+  los `permissions` exactos de la tabla 2 para el rol: agente
+  `contents:write`, `pull_requests:write`, `checks:read`, `statuses:read`,
+  `metadata:read`; revisor `checks:write`, `contents:read`,
+  `pull_requests:read`, `metadata:read`. Se reutiliza hasta `expires_at`
+  menos 5 min y luego se pide otro. Habla por el mismo `http::Cliente` y la
+  misma `Base` que el adaptador.
+- **Fallo al acuñar ≠ falta de identidad**: el trait gana `obtener(rol) ->
+  Result<Secreto, FalloCredencial>`; `Conexion::token` lo usa. Falta →
+  `rechazada motivo=identidad falta=<rol>` (como antes); una App que no
+  consigue su token → `denegada motivo=credenciales rol=<rol> causa=<causa>`
+  (código 8), con `causa` entre `app-incompleta`, `app-solo-github`,
+  `clave-ilegible`, `jwt-rechazado` (401), `sin-instalacion` (404),
+  `permisos-no-concedidos` (422) y `respuesta-inesperada`; sin red,
+  `sinred`. El diagnóstico nombra las variables de la App, nunca el token.
+- **GitLab sin código nuevo**: el agente es una cuenta de servicio (o un
+  token de proyecto) con rol Developer y su token va en `TC2_TOKEN_AGENTE`;
+  el adaptador ya trata `bot: true` como `Bot`. Una App configurada con
+  GitLab da `causa=app-solo-github`.
+- **Git del agente** (D3 del plan: solo el punto de extensión; conectarlo a
+  `review` es de F6-2): `plataforma::entorno_git(proveedor, url, token,
+  autor)` devuelve `GIT_CONFIG_COUNT/KEY_0/VALUE_0` con
+  `http.<url>.extraheader = Authorization: Basic <base64(usuario:token)>`
+  (`x-access-token` en GitHub, `oauth2` en GitLab), limitado a la URL del
+  remoto, y `GIT_AUTHOR_*`/`GIT_COMMITTER_*` con `<slug>[bot]` y
+  `<id>+<slug>[bot]@users.noreply.github.com`. Se compone con
+  `Git::con_entorno_extra`: el token no pasa por la línea de órdenes, ni por
+  el registro del puerto, ni por los argumentos de `git-remote-http`
+  (comprobado con `GIT_TRACE`). `git` no depende de `plataforma`.
+- **`identidad(Rol::Agente)` en GitHub** hace `GET /user`, que no funciona
+  con un token de instalación (es un endpoint de usuario). Hoy solo se llama
+  con `Rol::Persona`; quien necesite la cuenta del agente usa
+  `CredencialesApp::slug()` (`GET /app` con el JWT) y la cuenta
+  `<slug>[bot]`. En GitLab, `GET /user` sí funciona con la cuenta de
+  servicio.
+- **D4 — firma de los commits del agente**: no se exige. Los commits del
+  agente salen sin firma (sin «Verified»); la firma que cuenta sigue siendo
+  la de la persona en `approve` (decisión 3). Responde la pregunta abierta
+  «Firma de los commits del agente».
+- **Dependencias**: `ring` 0.17.14 y `base64` 0.23.1, directas en la misma
+  versión que ya tenía `Cargo.lock` (por rustls y ureq); ningún crate nuevo.
+
+**Evidencia** (`crates/tc2/tests/identidades.rs` y unitarios de
+`credenciales.rs` y `app_github.rs`; clave RSA desechable generada en cada
+test con `ssh-keygen -t rsa -m PEM`, ninguna clave en el repo):
+
+- El JWT de cada petición se verifica con la clave pública que exporta
+  `ssh-keygen -e -m PEM` (no con el código que firma); `iss` numérico,
+  `exp - iat` ≤ 600 s.
+- El cuerpo de `access_tokens` es exactamente `{"repositories":["r"],
+  "permissions":{…}}` con la tabla 2 del rol (agente y revisor); la
+  instalación se busca una vez; un token ya caducado se renueva y uno vigente
+  se reutiliza (3 llamadas a `obtener` → 1 `GET` + 2 `POST`).
+- Por la fábrica, `prs-abiertos` con la App lleva `Bearer <token de
+  instalación>` y no el de `TC2_TOKEN_AGENTE`; con la App sin instalar,
+  `prs-abiertos denegada motivo=credenciales rol=agente
+  causa=sin-instalacion` (8), sin probar con el token de la variable.
+- Con un repo git real, el commit sale con autor y committer `[bot]`, el
+  push manda `Authorization: Basic …` al remoto HTTP simulado y el token no
+  aparece en el registro del puerto ni en `GIT_TRACE`.
+- **Mutaciones**: dar `pull_requests:write` al revisor pone en rojo el
+  unitario de permisos y el de integración; pasar la credencial a la URL con
+  `url.<url-con-token>.insteadOf` pone en rojo el test de git (por la
+  cabecera ausente y, con la cabecera mantenida, por el token en `GIT_TRACE`).
+
+**Divergencias con el plan de TASK-047**:
+
+1. **`de_entorno()` sin argumentos y `atar_github(base, repo)` en el
+   trait.** El plan pedía `de_entorno(base, repo)`, pero dos de los cuatro
+   sitios (`lib.rs`) crean las credenciales antes de conocer el destino. La
+   App se ata al repo cuando `GitHub::nuevo` recibe el proveedor; por
+   defecto `atar_github` no hace nada.
+2. **`tiene(rol)` nuevo en el trait**: elegir con qué rol se pregunta
+   (`identidad::conectar_produccion`) o si hay persona (`init`) sin hablar con
+   la red; con App, cuenta aunque el token se acuñe después.
+3. **Entorno git en el puerto `Git`, no como parámetro de `git::remoto`**:
+   el autor tiene que valer también para el commit (`guardar`), no solo para
+   el push. `Git::con_entorno_extra` añade variables a las que ya haya.
+4. **Cabecera básica limitada a la URL del remoto** (`http.<url>.extraheader`)
+   en vez de una cabecera global.
+5. **Correo del autor**: con el id de la cuenta `[bot]` cuando se conoce;
+   el `Cliente` no puede leerlo (`/users/<slug>[bot]` lleva `[`, que la
+   validación de rutas rechaza), así que lo pasa quien llama (el E2E lo lee
+   con el arnés); sin id, `<slug>[bot]@users.noreply.github.com`.
+6. **PKCS#8 además de PKCS#1**: `RsaKeyPair::from_der` lee el DER PKCS#1 que
+   da GitHub (y `ssh-keygen -m PEM`); el PEM se decodifica a mano (base64 sin
+   cabeceras; uno cifrado se rechaza) y `BEGIN PRIVATE KEY` va por
+   `from_pkcs8`.
+
+**Suposiciones del plan**:
+
+- `RsaKeyPair::from_der` acepta el PEM PKCS#1 (una vez decodificado a DER):
+  **verificado** (test de integración).
+- `access_tokens` admite `permissions` y `repositories`: lo dice la
+  documentación de GitHub y el test comprueba lo que se manda; **contra
+  GitHub real, no verificado** hasta ejecutar el E2E.
+- Tier del proyecto de GitLab y `bot: true` de la cuenta de servicio: **no
+  verificado**; el E2E lo afirma.
+
+**E2E ignorados, no ejecutados** (necesitan la App de D1 y la cuenta de
+servicio de D2): `persona_aprueba_pr_del_agente` en
+`tests/plataforma_e2e.rs` (variables `TC2_E2E_REPO`, `TC2_E2E_TOKEN`,
+`TC2_E2E_APP_ID`, `TC2_E2E_APP_CLAVE`) y `persona_aprueba_mr_del_agente` en
+`tests/plataforma_gitlab_e2e.rs` (`TC2_E2E_GITLAB_PROYECTO`,
+`TC2_E2E_GITLAB_TOKEN`, `TC2_E2E_GITLAB_TOKEN_AGENTE`). Los dos empujan por
+git con el token del agente y sin la configuración de git del usuario (así
+cubren también «`fetch` por git con un token de instalación»), abren el
+PR/MR con el adaptador, intentan que el agente apruebe el suyo (en GitHub
+tiene que fallar: cierra «que GitHub impida a una App aprobar su propio PR»;
+en GitLab, si lo deja, no cuenta) y la persona aprueba hasta
+`aprobaciones=1`. Hasta ejecutarlos, esos puntos de «No verificado» siguen
+abiertos.
diff --git a/docs/plan/PLAN-TASKCODE-2.md b/docs/plan/PLAN-TASKCODE-2.md
index 75e7e4e..dfbc4d4 100644
--- a/docs/plan/PLAN-TASKCODE-2.md
+++ b/docs/plan/PLAN-TASKCODE-2.md
@@ -390,7 +390,7 @@ dependencias. Las fases 1 a 9 son tareas de TaskCodeTwo; la 0, del repo TaskCode
 
 | ID | Tarea | Criterio de aceptación | Dep. |
 |---|---|---|---|
-| F6-1 | Identidades: agente implementador (commits y PR/MR) y revisor (reviews y checks; nunca aprobar ni mergear), como GitHub App y como usuario bot o token de proyecto en GitLab | La persona aprueba un PR/MR del agente en GitHub y en GitLab | F1-6 |
+| F6-1 | Identidades: agente implementador (commits y PR/MR) y revisor (checks; nunca aprobar ni mergear; ver divergencia 6 y adenda F6-1 del ADR F1-6), como GitHub App y como usuario bot o token de proyecto en GitLab | La persona aprueba un PR/MR del agente en GitHub y en GitLab | F1-6 |
 | F6-2 | `review`: abre el PR contra `develop` con la identidad del agente y lanza el revisor | PR enlazado a la tarea; check `revision-ia` | F6-1, F2-6 |
 | F6-3 | Revisor en CI con las skills de dominio de la 0.16 (angular-vue, rust-wasm, C#, Java, genérica), enrutado por lo que toca el diff y con el modelo que fije F4-6 (RR-02, RR-17) | Un diff Rust + Vue lanza las dos de dominio | F6-2, F5-4 |
 | F6-4 | Búsqueda de errores: reproducción, mutantes y bordes con evidencia en cada hallazgo (RR-03) | Hallazgo sin evidencia = rechazado por el propio workflow | F6-3 |
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
Cargo.lock                                                                                | 2 ++
 tareas/{01-en-diseno => 02-en-curso}/TASK-047/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-047/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-047/tarea.md                                    | 3 ++-
 4 files changed, 4 insertions(+), 1 deletion(-)
````
