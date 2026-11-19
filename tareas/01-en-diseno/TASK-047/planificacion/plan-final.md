# Plan — TASK-047: F6-1 Identidades: agente implementador

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 120 min
Senales: proveedor nuevo de credenciales con JWT RS256 y cache (unas 250 lineas mas tests), un punto de extension de credenciales para git, 2 E2E nuevos (GitHub y GitLab), adenda en el ADR F1-6 y cambio de la fila F6-1 del plan; sin comando nuevo.

No se crea un modulo de identidades: se extiende el proveedor de credenciales por rol que ya existe. El ADR F1-6 (decision 1) fija que el puerto `Plataforma` recibe un proveedor por rol, y ya esta implementado: `credenciales.rs` (trait `Credenciales::token(rol) -> Option<Secreto>`, `CredencialesEntorno`, `CredencialesFijas`) y `conexion.rs:76` (`Conexion::token`, que falla cerrada y nunca prueba con otro rol). Lo que falta:

- **GitHub necesita codigo.** Una App no tiene token fijo: se firma un JWT RS256 con su clave privada y se pide un token de instalacion (`POST /app/installations/{id}/access_tokens`), que dura poco.
- **GitLab no necesita codigo nuevo.** Un token de proyecto, de cuenta de servicio o de cuenta de maquina es un bearer normal y entra por `TC2_TOKEN_AGENTE` / `TC2_TOKEN_REVISOR`; `gitlab.rs:218` ya pone `bot: true` como `Bot`.

Piezas y limites:
- `plataforma/app_github.rs` (nuevo): `CredencialesApp { app_id, clave, base, repo }` implementa `Credenciales`.
  - Firma un JWT RS256 con `iss=app_id`, `iat` 60 s en el pasado y `exp` a 9 min o menos.
  - Busca la instalacion con `GET /repos/{o}/{r}/installation`.
  - Pide el token **reducido al repo y a los permisos de la tabla 2 del ADR para ese rol**. Agente: `contents:write`, `pull_requests:write`, `checks:read`, `statuses:read`, `metadata:read`. Revisor: `checks:write`, `contents:read`, `pull_requests:read`, `metadata:read`.
  - Cache hasta `expires_at` menos un margen.
  - Usa `http::Cliente` con la misma `Base` que el adaptador (probable contra `tests/comun/http_mock.rs`).
- `credenciales.rs` (se extiende):
  - Metodo nuevo con implementacion por defecto `fn obtener(&self, rol) -> Result<Secreto, FalloCredencial>`, para que un JWT rechazado o una App sin instalar no se confunda con «falta identidad». `Conexion::token` pasa a usarlo; lo existente no cambia.
  - Fabrica `credenciales::de_entorno(base, repo) -> Box<dyn Credenciales>`: por rol, con `TC2_APP_<ROL>_ID` y `TC2_APP_<ROL>_CLAVE` (ruta al PEM) usa la App; si no, `TC2_TOKEN_<ROL>`. Nunca pasa de un rol a otro; la persona sigue limitada a terminal.
  - Sustituye los 4 sitios que hacen `CredencialesEntorno` a pelo: `lib.rs:356`, `lib.rs:482`, `init/mod.rs:566`, `identidad.rs:574`.
- Cripto: `ring` y `base64` ya estan en `Cargo.lock` (por ureq/rustls); se declaran como dependencias directas en la misma version, sin crates nuevos. Sin `gh` (F1-14).
- Commits del agente: `git::remoto` recibe un parametro generico de entorno extra (`&[(OsString, OsString)]`); `plataforma` le pasa `GIT_CONFIG_COUNT/KEY/VALUE` con `http.extraheader` (`x-access-token`) y el autor `<slug>[bot]`. El token va por entorno, nunca en argv. `git` no depende de `plataforma`. Conectarlo a `review` es de F6-2 (TASK-048).
- Documentacion: adenda F6-1 en `docs/adr/F1-6-identidades-agente-revisor-persona.md` y fila F6-1 del plan con «checks» en vez de «reviews y checks» (divergencia 6 del ADR).

Orden de construccion:
1. `obtener` en el trait y la fabrica `de_entorno`, sin cambio de comportamiento.
2. `CredencialesApp` (JWT, instalacion, token reducido, cache) con su doble HTTP.
3. Entorno extra en `git::remoto` (push y autor del agente).
4. E2E ignorados de GitHub y GitLab.
5. Adenda del ADR y fila del plan.

Alternativas descartadas:
- Crate `jsonwebtoken`: trae su propia pila de cripto, cuando `ring` ya esta en el arbol.
- Pegar a mano un token de instalacion en `TC2_TOKEN_AGENTE`: caduca en 1 h y el JWT se firmaria fuera de tc2. Solo vale de apano.
- Un trait `Identidad` aparte: duplicaria `Credenciales` y contradice la decision 1 de F1-6.

Credenciales que ya existen: los tokens de `gh` (charliebk) y `glab` (charlie.bk) valen solo para el lado de la persona (aprobar, mergear, montar y limpiar E2E). No valen como agente: con el mismo usuario no se aprueba el PR propio y se romperia RN-01.

## Troceo

### Parte a: credenciales por App y fabrica (pasos 1-2)
### Parte b: entorno git del agente (paso 3)
### Parte c: E2E y documentacion (pasos 4-5)

No se trocea: Carlos pidio estas tareas sin trocear (2026-11-18).

## Lo que el rol no cubrio

- **C1** (la persona aprueba un PR/MR del agente en GitHub y GitLab): la forma esta cubierta; que pase de verdad depende de identidades reales (D1, D2).
- **C2**: cubiertos en forma RN-01, RR-04, RR-08 y RR-15. No se han revisado modos de fallo (riesgos), estrategia de pruebas a fondo (testing) ni reglas de GitLab por tier (dominio).
- El revisor: el proveedor lo soporta (`TC2_APP_REVISOR_*`); publicar `revision-ia` y poner su `app_id` en `plataforma.revisor_id` y en los rulesets es de F6-2/F6-3.
- Que `doctor` avise si el token del agente es de una persona (RR-15): fuera; es de F3-8.

## Riesgos aceptados y que los contiene

- `identidad(Rol::Agente)` hace `GET /user` (`github.rs:80`), que no funciona con token de instalacion. Hoy solo se llama con `Rol::Persona`. Lo contiene una nota en la adenda.
- La clave de la App queda en la maquina del agente (D5 de F1-6). Lo contienen el token reducido por peticion y la instalacion solo en los repos del proyecto.
- El token acunado vive en memoria y en el entorno del hijo git. Lo contienen el `Debug` opaco de `Secreto`, que no va en argv y que caduca en 1 h.
- No se verifica la firma de commits del agente (D4).

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- **C1:** 3 tests.
  1. Integracion con `http_mock`: `CredencialesApp` manda un JWT verificable con la clave publica, pide el token con `repositories` y los `permissions` exactos del rol, reutiliza la cache y renueva al caducar. La clave RSA se genera en el test con `ssh-keygen -t rsa -m PEM`; ninguna clave al repo.
  2. **Pesado**, E2E GitHub ignorado `persona_aprueba_pr_del_agente`: la App crea rama, commit y `abrir_pr`; la persona aprueba (`APPROVE`) y `estado_puertas(Persona)` da `aprobaciones=1`; la App intenta aprobar su propio PR y recibe rechazo.
  3. **Pesado**, E2E GitLab ignorado con el mismo recorrido y `approve` de la persona.
- **C2:** 3 tests ligeros.
  1. Unitario: la fabrica da precedencia a la App sobre el token, sin paso entre roles, persona limitada a terminal, fallo al acunar con motivo distinto de `falta`.
  2. Unitario: los permisos por rol coinciden con la tabla 2 del ADR; el revisor no pide `pull_requests:write` ni `contents:write`.
  3. Integracion: el entorno git del agente lleva la cabecera y el autor `[bot]`, y el token no aparece en los argumentos (repo desnudo local).

## Lo que necesita decision de una persona

Consigna de Carlos (2026-11-18): la opcion recomendada y sin trocear. Las que exigen crear identidades reales se le preguntan.

- **D1. Identidades reales.** Elegida por Carlos (2026-11-19): (b) Carlos crea la GitHub App del agente, instalada solo en `charliebk/taskcodetwo-e2e-github`, y pasa id y ruta al PEM por variables antes de la revision. La App del revisor queda para F6-3.
- **D2. Identidad del agente en GitLab.** Elegida por Carlos (2026-11-19): (a) grupo de primer nivel con el proyecto de E2E y una cuenta de servicio Developer (GitLab.com las permite en Free desde la 18.11, solo en grupos de primer nivel). La ruta del proyecto de E2E puede cambiar: se lee de la configuracion de los E2E, no se fija en codigo.
- **D3. Push y autor del agente.** Elegida: (a) solo el punto de extension en `git::remoto`, sin conectarlo a `review` (eso es F6-2).
- **D4. Firma de commits del agente.** Elegida: (a) no se exige; se anota en la adenda.
- **D5. Configuracion de la App.** Elegida: (a) variables `TC2_APP_<ROL>_ID` y `TC2_APP_<ROL>_CLAVE`, con la instalacion descubierta por repo.

Suposiciones por verificar en la implementacion:
- `ring::signature::RsaKeyPair::from_der` acepta el PEM PKCS#1 de GitHub (paso 2).
- `access_tokens` admite `permissions` y `repositories` para reducir el token.
- El tier del proyecto de GitLab y que una cuenta de servicio salga con `bot: true`.
