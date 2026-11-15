# Plan — TASK-021: F2-6 Adaptador GitHub

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 180 min

**Dónde vive.** Módulo nuevo `crates/tc2/src/plataforma/`, hermano de `git/`, `estado/` y `alta`. Depende solo de `contrato` (no de `git` ni de `estado`), y nadie de lo existente depende de él. Sigue el precedente de la capa Git (TASK-017):
- operaciones públicas en la biblioteca que devuelven `Informe`, o `Result<T, Informe>` (el `Paso<T>` de `git/mod.rs`) en las lecturas tipadas que otros componen;
- lista cerrada por construcción, como el `enum Subcomando` de `git/puerto.rs`.

**El binario `tc2` no gana subcomandos.** Los llamadores son F6-2 (`review`), F3-5, F3-8 y F7 (`finish`), igual que la capa Git solo publicó `guardar` y `actualizar-rama` (D1a de F1-4).

**Piezas** (ficheros nuevos):
- `plataforma/mod.rs`: trait del puerto `Plataforma` (D1a de F1-13) y tipos neutrales. El trait se crea ya, aunque solo lo implemente GitHub, porque F2-9 tiene que pasar la misma batería byte a byte (decisión 9 de F1-13). Tipos: `Rol {Agente, Revisor, Persona}`, `Pr {numero, sha_head, base, estado}`, `EstadoPuertas`, `ModeloPuertas {checks: [(nombre, integration_id)], exigir_impuestas}`, `Personas` (ids), `Firma {verificada, committer_id, tipo}`, `Usuario {login, id, tipo}`, `ReglasLeidas`.
  ```text
  trait Plataforma { fn abrir_pr(&self, rama, base, titulo, cuerpo) -> Informe;
    fn estado_puertas(&self, pr, &ModeloPuertas, &Personas) -> Informe;
    fn merge_pr(&self, pr, sha_esperado) -> Informe;
    fn leer_puertas(&self, rama) -> Paso<ReglasLeidas>; fn leer_permisos_ci(&self) -> Informe;
    fn firma_commit(&self, sha) -> Paso<Firma>; fn identidad(&self, Rol) -> Paso<Usuario>; }
  ```
- `plataforma/puertas.rs`: función pura `evaluar(lecturas neutrales, modelo, personas) -> Informe`, el `_evaluar` de `poc-f1-13.py` en Rust. Fuera del adaptador para que GitHub y GitLab compartan la línea `estado-puertas …`.
- `plataforma/http.rs`: envoltorio único `Cliente` sobre `ureq` (bloqueante, como el resto del CLI). Antes de la red comprueba el método exacto; la base (`https://api.github.com`, la URL de la API de la configuración, o `http://127.0.0.1:<puerto>` solo en tests); y la ruta sin `.`, `..`, `%2e` ni `\`. Sin redirecciones (`max_redirects(0)`), con límite de respuesta (1 MiB) y tiempos de espera, y registro de llamadas `MÉTODO ruta` sin cabeceras. No hay `get(url)` genérico: cada endpoint es una variante de `github::Endpoint` que construye método y ruta (`/repos/{o}/{r}/…`).
- `plataforma/errores.rs`: tabla HTTP → línea de contrato (abajo); la única que conoce códigos HTTP, reutilizable por GitLab.
- `plataforma/credenciales.rs`: trait `Credenciales { fn token(&self, Rol) -> Option<Secreto> }`; `Secreto` newtype sin `Display` y con `Debug` enmascarado; `CredencialesEntorno` lee `TC2_TOKEN_AGENTE`, `TC2_TOKEN_REVISOR` y `TC2_TOKEN_PERSONA` (el de persona exige TTY, `IsTerminal`; decisión 5 de F1-6). Cada operación declara su `Rol` por la tabla de identidades de F1-6; sin el token de ese rol, `rechazada motivo=identidad falta=<rol>` (3), nunca con otro rol. En `merge-pr`, `leer-permisos-ci` y `estado-puertas` como persona, el `id` de `GET /user` tiene que estar en `Personas`, si no `rechazada motivo=identidad`. F6-1 cambia esta pieza por el proveedor de Apps sin tocar el trait.
- `plataforma/github.rs`: el adaptador, con structs serde solo de los campos que lee (la respuesta es dato no confiable, Z2 de F1-8: campo ausente → `error motivo=respuesta-invalida`, nunca pánico). Llamadas (decisión 4 de F1-13):

  | Operación | Llamadas | Resultado |
  |---|---|---|
  | `abrir-pr` | `GET /pulls?head=o:rama&state=open` y, si no hay, `POST /pulls` | `ok pr=N`, o `sin-cambios pr=N` si existía; un 422 «already exists» relee y da `sin-cambios` |
  | `estado-puertas` | 4 GET | lectura |
  | `merge-pr` | `GET /pulls/{n}` (mergeado → `sin-cambios`) y `PUT /pulls/{n}/merge` con `merge_method=merge` y `sha` | el `sha` es el evaluado; si el head se movió, 409 y no se mergea |
  | `leer-puertas` | `GET /rulesets`, `GET /rules/branches/{rama}` → `ReglasLeidas` | lectura |
  | `leer-permisos-ci` | `GET /actions/permissions/workflow` | identidad de persona |
  | `firma_commit` | `GET /commits/{sha}` | lectura |
  | `identidad` | `GET /user` | una vez por rol y proceso |

  Las listas paginadas (`reviews`, `check-runs`) piden `per_page=100` y siguen `Link rel=next` solo al mismo host, por el envoltorio y con tope de páginas.
- `plataforma/deteccion.rs`: función pura `detectar(url_origin, clave_config) -> Result<Destino, Informe>` (decisión 7 de F1-13). La URL la obtiene quien llama con `git ls-remote --get-url origin` (`LsRemote` ya está en `Subcomando`).

**Tabla de errores** (códigos de `contrato.rs`; el 8 `denegada` existe desde F2-3):

| HTTP | Línea | Arreglo en stderr |
|---|---|---|
| 401 | `denegada motivo=credenciales rol=<rol>` (8) | define o renueva `TC2_TOKEN_<ROL>` |
| 403 con «Upgrade to GitHub Pro» | `rechazada motivo=plan` (3) | repo público o GitHub Pro |
| 403/429 con `x-ratelimit-remaining: 0` o `retry-after` | `agotada motivo=limite-peticiones reintenta-en=<s>` (5) | espera y relanza |
| 403 sin aviso de plan | `denegada motivo=permisos` (8) | el token necesita `<permiso>` |
| 404 | `denegada motivo=no-existe-o-sin-acceso` (8) | revisa el repo y el alcance del token |
| 405 en el merge | `rechazada motivo=no-mergeable` (3) | lanza `estado-puertas` |
| 409 en el merge | `rechazada motivo=head-cambiado` (3) | vuelve a evaluar las puertas |
| 422 «No commits between» | `rechazada motivo=sin-commits` (3) | — |
| 422 otro | `error motivo=validacion` (1) | — |
| 3xx | `error motivo=redireccion` (1) | — |
| DNS, conexión, timeout | `sinred` (4) | reintenta con red |
| TLS | `error motivo=tls` (1) | instala la CA en el almacén del sistema |
| cuerpo > 1 MiB o JSON inválido | `error motivo=respuesta-invalida` (1) | — |

Ningún mensaje lleva el token ni cabeceras; el texto de error de la API se recorta y escapa.

**Dependencias:** `ureq = { version = "3.5", default-features = false, features = ["rustls", "platform-verifier"] }` (el mismo conjunto que la PoC de F1-14, `TASK-014/poc/poc-rust/Cargo.toml`) y `serde_json = "1"`. Sin dev-dependencias nuevas: el mock HTTP se escribe con `std::net::TcpListener` en `tests/comun/`. `ring` compila C: que el build siga reproducible con `/Brepro` no está verificado y se comprueba en el paso 1.

**Orden** (un commit por paso, con `cargo xtask verificar --perfil completa` en verde):
1. Dependencias, esqueleto de `plataforma/`, `http.rs` (`Cliente`, lista blanca, `Secreto`) y `errores.rs`; verificación completa para detectar pronto si `ring` rompe la reproducibilidad.
2. Mock en `tests/comun/http_mock.rs` y fixtures copiados a `crates/tc2/tests/fixtures/plataforma/github/` (6 casos y `observado.json`).
3. Tipos neutrales, `puertas::evaluar` y `estado_puertas` de GitHub hasta pasar la batería de 6 casos.
4. `abrir_pr`, `merge_pr`, `leer_puertas`, `leer_permisos_ci`, `firma_commit`, `identidad`, credenciales y `deteccion`, con fixtures nuevos marcados «no observado».
5. E2E `#[ignore]` contra el repo de pruebas; las formas reales pasan a `observado.json`.

**Fuera de F2-6, con su dueño:** `aplicar-puertas` y la comparación de `ReglasLeidas` con el modelo de F1-5 (F3-1, F3-8); `capacidades` y `doctor` (F3-8); Apps, JWT y tokens de instalación (F6-1); el «quién» de una tarea y el fichero de la lista de personas (F2-13, F6-1; aquí `Personas` es parámetro); lectura de `.taskcode/config.yml` (F3-1; `deteccion` recibe la clave).

**Descartadas:** `get(url)` genérico con lista blanca por regex (la superficie se cierra por `enum`); subcomando oculto `tc2 plataforma <op>` (sin usuario hasta F6-2); `httpmock`/`wiremock` (arrastran tokio).

## Troceo

No se trocea aunque pase de 60 min: Carlos pidió no trocear ninguna tarea del H2, y ninguna pieza sola cumple el criterio (el E2E necesita envoltorio, credenciales y las tres operaciones). Los cinco pasos son commits de la misma rama.

## Lo que el rol no cubrio

- **Riesgos** en detalle: carreras de `abrir-pr` entre dos procesos, token que caduca a mitad, A5 de F1-8 (el agente lee la variable del token de persona).
- **Dominio:** si `estado-puertas` cuenta la última review por usuario (como la PoC) o todas; `mergeable_state` `unknown` o `blocked`; si el 405 debe ser `esperando` (7). Se sigue la PoC y F1-6 sin validarlo.
- **Pruebas:** solo presupuesto y recorrido del E2E; cobertura y mutantes sin diseñar.
- Criterio 1 cubierto por el paso 5; criterio 2 por la lista cerrada, salvo `aplicar-puertas` (divergencias).

## Riesgos aceptados y que los contiene

- **Build no reproducible por `ring`:** el paso 1 lo detecta; si falla, se para y se lleva a Carlos.
- **Fixtures de escritura y error inventados:** marcados «no observado» hasta que el E2E los contrasta.
- **Token de persona en variable de entorno legible por el agente** (A5): el TTY lo frena en parte; aceptado hasta F6-1 y dicho en el `## Resultado`.
- **Puertas no comprobables con una cuenta** (descartar aprobaciones viejas, último push, cola, force-push): siguen «no verificado» y pasan a F6-1 y F9-4.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- **Criterio 1, E2E — 2 tests `#[ignore]`** con `TC2_E2E_REPO` y `TC2_E2E_TOKEN`, lanzados a mano con `cargo test -- --ignored`; la salida va de evidencia al `## Resultado`.
  - (a) Recorrido: rama empujada → `abrir-pr ok` → `abrir-pr sin-cambios` con el mismo número → `estado-puertas` → `merge-pr ok` → `merge-pr sin-cambios`; `firma_commit` del merge (`web-flow`, verificada, la regla 4 de F1-6 no la cuenta); `leer-permisos-ci`; `identidad`; `estado-puertas` con token falso (401) y repo inexistente (404) dan la línea dirigida.
  - (b) Puertas reales: el arnés crea con un cliente propio un ruleset en `develop` (PR y 1 aprobación) y comprueba `estado-puertas esperando falta=aprobacion`, `merge-pr rechazada motivo=no-mergeable`, que el autor no puede aprobar su PR (422) y la forma real de `GET /reviews` con una review `COMMENTED`.
- **Criterio 2, cumple F2-6 — 3 tests contra el mock** (sin red, dentro de `xtask verificar`): la batería de los 6 casos de la PoC M con la misma línea y el mismo número de llamadas, más `abrir-pr` y `merge-pr` idempotentes; la prueba negativa del envoltorio (otro método, otro host, `..`, `%2e`, `http` fuera de `127.0.0.1`, redirección, cuerpo > 1 MiB; 0 peticiones en el mock); la tabla de errores con su línea y código, un rol sin token, y el token ausente de stdout, stderr y `Debug`.
- Cada test de tabla cuenta como uno; los unitarios de `deteccion` y escapado quedan fuera del presupuesto.

## Lo que necesita decision de una persona

1. **Repo de pruebas en GitHub** — decidido por Carlos (2026-11-15, pregunta explícita): repo **público** nuevo `charliebk/taskcodetwo-e2e-github`, lo crea el orquestador con `gh`; como el token no tiene `delete_repo`, lo borra Carlos al terminar.
2. **Identidades en el E2E** — opción recomendada (consigna de Carlos): una sola cuenta, su token hace de agente y de persona; la separación llega con las Apps en F6-1.
3. **`aplicar-puertas` a F3-1** — opción recomendada: sí; F2-6 no escribe rulesets y el E2E los crea desde su arnés.
4. **Exposición en el binario** — opción recomendada: solo biblioteca, sin subcomando hasta F6-2.

**Divergencias con el plan y los ADR:**
- F1-5, Consecuencias: «F2-6 implementa `aplicar-puertas` y `leer-puertas` (con comparación)»; aquí la escritura y la comparación pasan a F3-1 y F3-8.
- F1-6, decisión 1, `leer-permisos-actions` → `leer-permisos-ci` (divergencia 2 de F1-13).
- 401, 403 y 404 van a `denegada` (8), que F1-4 no tenía y añadió F2-3; la identidad equivocada sigue siendo `rechazada` (3) como en F1-6.

**Suposiciones no verificadas:** nombres exactos de la API de ureq 3.5 (`max_redirects(0)`, `http_status_as_error(false)`, `RootCerts::PlatformVerifier`, límite del cuerpo), que se miran antes del paso 1; que `ring` no incruste rutas del checkout; `GET /user` no vale con token de App (F6-1 cambiará `identidad`).
