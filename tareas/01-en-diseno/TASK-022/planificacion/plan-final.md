# Plan — TASK-022: F2-9 Adaptador GitLab

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 150 min

Un adaptador en `crates/tc2/src/plataforma/gitlab.rs` que implementa el mismo trait `Plataforma` de `mod.rs`, sin cambiar ninguna firma. Igual que en F2-6, solo en la biblioteca: el binario no gana subcomandos ni cambia su sha256. Sin `glab`: API REST v4 con el `Cliente` de `http.rs`.

**Lo que hay que extender antes** (leído en `github.rs`, `http.rs`, `errores.rs`, `puertas.rs` y `deteccion.rs`):
1. `Cliente::cabeceras` manda siempre `Accept: application/vnd.github+json` y `X-GitHub-Api-Version`.
2. `validar_ruta` rechaza `%`, y el id de un proyecto con nombre es `grupo%2Fproyecto`.
3. `Base` no admite prefijo de ruta en `http://127.0.0.1`, y los fixtures de la PoC M empiezan por `/api/v4`.
4. `errores.rs` cita «GitHub: «…»» y reconoce el plan solo por `AVISO_PLAN`.
5. El transporte (`token`, `pedir`, `leer`, `leer_paginas`, la caché de identidades, `exigir_persona` y los validadores) es privado de `GitHub`. Copiarlo daría dos comportamientos distintos para la misma paginación y los mismos tokens.

**Piezas**
- **`mod.rs`.**
  - `Proveedor` pasa aquí desde `deteccion.rs`, que lo reexporta con `pub use`.
  - `Personas::con_claves(ids, claves)`.
  - `Firma.clave: Option<u64>`. `cuenta_como_aprobacion` pasa a ser «verificada y (committer persona o clave de persona)»: es el contrato del firmante de GitLab (decisión 6 de F1-13 y GL-15). GitHub deja `None` y no cambia.
  - Comentarios neutrales: en GitLab, el `u64` de cada check es el `author.id` del estado.
- **`conexion.rs` (nuevo, sale de `github.rs`).** `Conexion { cliente, proveedor, credenciales, personas, identidades }` con:
  - `token` y `pedir`;
  - `leer<T>` y `leer_paginas` (`Link rel=next` en la misma base, tope de 10 páginas);
  - `exigir_persona` y los validadores.

  Cada adaptador conserva su `enum Endpoint`.
- **`http.rs`.**
  - `Cliente::para(base, Proveedor)`: GitLab manda `Accept: application/json`, sin las cabeceras de GitHub, y los dos `Bearer`.
  - Lee también `ratelimit-remaining` y `ratelimit-reset`.
  - `validar_ruta` admite **solo** `%2F` dentro de un segmento.
  - `Base` admite en `127.0.0.1` el mismo prefijo validado que en https.
- **`errores.rs`.** Recibe el `Proveedor`, que pone el nombre de la plataforma y el arreglo en el diagnóstico.
  - El plan de GitLab lo decide el adaptador según el endpoint.
  - El 406 del merge da `rechazada motivo=no-mergeable`, como el 405.
  - El 409 del merge da `head-cambiado`.
  - El 409 de «MR existente» lo atiende `abrir_pr`.
- **`gitlab.rs`.** `GitLab::nuevo(base, proyecto, credenciales, personas)`, donde `proyecto` es un id numérico o `grupo[/sub]*/proyecto`, validado y codificado con `%2F`.

| Operación | Llamadas (`/projects/{id}`) | Resultado |
|---|---|---|
| `abrir-pr` | `GET /merge_requests?source_branch&target_branch&state=opened`; si no hay MR, `GET /repository/compare` y `POST /merge_requests` | `ok pr=<iid>` o `sin-cambios`. Sin commits, `rechazada motivo=sin-commits`. Con un 409, relee y da `sin-cambios` |
| `estado-puertas` | MR con `include_diverged_commits_count`, `approval_rules`, `protected_branches/{destino}`, `statuses` del sha (paginado) y `approvals` | 5 lecturas. `merged` da `mergeado` y `closed`, `pr-cerrado`. 403 de `approval_rules` con garantía Premium exigida: `motivo=plan`. `merge_access_levels`≠[40] o `allow_force_push`: `motivo=config garantia=G01`. De cada estado, el último por id, solo si viene de su autor. Aprobaciones por `user.id` en `Personas`. `al_dia` = `diverged_commits_count==0`; `sin_conflictos` = `!has_conflicts`. Todo va a `puertas::evaluar` |
| `merge-pr` | `exigir_persona`, `GET` del MR, `PUT .../merge` con `sha` | ya mergeado: `sin-cambios`. Head distinto: `head-cambiado`, sin PUT |
| `leer-puertas` | `GET /protected_branches` (paginado) y `GET /approval_rules` | 403 de plan: `plan_da_reglas=false` |
| `leer-permisos-ci` | `exigir_persona` | `ok ci-aprueba=false` (G16: el job token es fijo) |
| `firma_commit` | `GET .../signature`; si da 404, `GET` del commit | solo cuenta `verified`. `clave` = `key.id` (SSH) o `gpg_key_id` (GPG) |
| `identidad` | `GET /user` | `Bot` si `bot: true`; en caché por rol |

**Orden.** Un commit por paso, cada uno con la verificación completa en verde:
1. Refactor sin cambio de comportamiento: `Proveedor`, `conexion.rs`, `errores` y `http` parametrizados, `%2F` y el prefijo en `127.0.0.1`. Los tests de F2-6 siguen en verde sin tocar lo que esperan.
2. Fixtures de GitLab copiados, con `has_conflicts` añadido. `estado_puertas` hasta que la batería compartida dé la misma línea que GitHub.
3. El resto de operaciones y las filas de GitLab de la tabla de errores.
4. E2E `#[ignore]` y las formas reales en `observado.json`.

**Sin trocear, aunque pase de 60 min:** es decisión de Carlos para el H2. Además, ninguna pieza cumple sola el criterio.

**Fuera de F2-9, con su dueño:**
- `aplicar-puertas` (F3-1);
- `capacidades`, `doctor` y el resto de `leer-puertas` (F3-8);
- tokens de proyecto y cuentas de servicio (F6-1);
- la lista de personas con sus claves (F2-13);
- la fábrica `Destino → Box<dyn Plataforma>`, que llega con su primer llamador.

**Descartadas:**
- copiar el transporte;
- `head_pipeline` y jobs en vez de `statuses`: pierde el `revision-ia` externo;
- resolver el id numérico del proyecto: una llamada más;
- mandar las cabeceras de GitHub a GitLab.

## Lo que el rol no cubrio

- **Riesgos:**
  - un 403 de permisos que se confunda con el de plan en `approval_rules`;
  - un token que caduque a mitad;
  - carreras de `abrir-pr`;
  - A5.
- **Dominio** (se sigue la PoC y F1-13):
  - el filtro por autor de los jobs;
  - `skipped` y `allow_failure`;
  - un MR en draft;
  - `motivo=config` en `estado-puertas`;
  - las etiquetas de `reglas_rama`.
- **Pruebas:** solo el presupuesto.
- **Contraste con los criterios:**
  - **Criterio 1:** paso 4.
  - **Criterio 2:** pasos 2 y 3, salvo `aplicar-puertas` y el resto de `leer-puertas` (divergencias).

## Riesgos aceptados y que los contiene

- **Refactor de `github.rs`:** lo contienen los tests de F2-6, que no cambian lo que esperan.
- **`%2F` en el envoltorio:** solo esa secuencia exacta, con pruebas negativas de `%2f`, `%2F..` y `%252F`.
- **Formas de GitLab no observadas:** van marcadas «no observado» hasta el E2E; lo que el E2E no alcance queda «sin verificar» en el Resultado.
- **Jobs reales sin runner:** el arnés publica los estados de commit él mismo.
- **Un rol Developer frente a `protected_branches` y `approval_rules`:** pasa a F6-1.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- **Criterio 1, E2E: 2 tests `#[ignore]`** en `tests/plataforma_gitlab_e2e.rs`, con `TC2_E2E_GITLAB_PROYECTO` y `TC2_E2E_GITLAB_TOKEN`. La salida va a `evidencia/e2e-gitlab.log`.
  - **(a) Recorrido:**
    - `abrir-pr` dos veces, idempotente;
    - `estado-puertas esperando`, y luego `falta=aprobacion`;
    - `head-cambiado`, merge ok y otra vez `sin-cambios`;
    - firma, identidad, permisos de la CI y `leer-puertas`, donde se observa el 403 de plan de Free;
    - `sin-commits`, 401 y 404.
  - **(b) Puertas reales:**
    - `develop` protegida;
    - conflicto: `falta=sin-conflictos` y merge `rechazada motivo=no-mergeable`;
    - rama por detrás: `al-dia`;
    - un estado de otro autor no cuenta (si hay una segunda identidad disponible; si no, sin verificar);
    - se intenta la autoaprobación y se anota lo observado.
- **Criterio 2: 3 tests contra el mock**, sin red y dentro de `xtask verificar`:
  - **batería compartida:** una tabla de líneas esperadas contra GitHub y contra GitLab, byte a byte, con su número de llamadas, más la idempotencia de GitLab;
  - **envoltorio y errores de GitLab:** `%2F` sí y `%2e` o `%252F` no; cabeceras; límite con `RateLimit-*`; 401, 403, 404, 405, 406, 409 y 429; el token sin aparecer;
  - **protecciones de GitLab:** estado de otro autor, aprobación de un bot, `verified_system`, clave fuera de la lista, `has_conflicts`, divergencia, `head-cambiado` sin PUT, `compare` vacío y G01 mal configurada.

Ninguno es pesado. Los dos E2E van con `#[ignore]` porque necesitan red y un token, no por coste.

## Lo que necesita decision de una persona

Carlos delega en la opción recomendada en todo el H2. Decidido así:

1. **Proyecto de pruebas:** público nuevo, `charlie.bk/taskcodetwo-e2e-gitlab`, en gitlab.com.
   - El token es el de `glab`, que Carlos renovó el 2026-11-15 (`glab auth login`, usuario `charlie.bk`). Va solo por variable de entorno, nunca al repo ni a los logs.
   - El orquestador crea el proyecto, con permiso de Carlos igual que en F2-6, y lo borra Carlos.
2. **Identidades:** una sola cuenta hace de agente y de persona; la separación llega en F6-1.
3. **Id del proyecto:** se admite solo `%2F` en `validar_ruta`.
4. **Contrato del firmante por clave:** `Firma.clave` y `Personas::con_claves`.
5. **`abrir-pr` con `compare` previo:** sí.
6. **`leer-puertas` a 2 llamadas:** sí; el resto pasa a F3-8.

**Divergencias con los ADR:**
- F1-13, decisión 4: `leer-puertas` lee 2 endpoints y `abrir-pr` suma un `compare`.
- F1-13, decisión 6: el origen de un check es el `author.id`, también para los jobs.
- `errores::traducir` gana el `Proveedor`.

**Suposiciones que se miran en el E2E:**
- que `Bearer` valga con un token personal;
- que `GET /user` traiga `bot`;
- el 403 de plan en Free;
- que `statuses` incluya los jobs;
- la forma del `Link`.
