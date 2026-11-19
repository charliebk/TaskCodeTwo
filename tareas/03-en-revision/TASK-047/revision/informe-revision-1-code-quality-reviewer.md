# Informe de revision — TASK-047 (ronda 1)

- Commit revisado: 8123fa68e74571def0591f22da57cfa2536e687c
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

Alcance: completo (primera ronda). Reviso el diff entero de `develop..HEAD` fuera de `tareas/` (18 ficheros) con foco en `plataforma/credenciales.rs`, `app_github.rs`, `entorno_git.rs`, `conexion.rs`, `git/puerto.rs`, los 4 sitios que dejan de usar `CredencialesEntorno`, los tests nuevos y la adenda F6-1. Profundidad: a fondo en credenciales, app_github, entorno_git, conexion y `tests/identidades.rs`; por encima (lectura, no ejecutables) los dos E2E ignorados y `tests/agente_git/mod.rs`. La peticion no trae verificacion de la ronda, y por indicacion del orquestador no he ejecutado la suite completa ni `cargo xtask`: solo los tests relevantes.

## Verificacion empirica (clon `task-047-r1-cq`, target propio, commit ebb6ad8)

| Orden | Resultado |
|---|---|
| `cargo test -p tc2 --test identidades` | 4 passed |
| `cargo test -p tc2 --test init` | 25 passed |
| `cargo test -p tc2 --test doctor` | 12 passed |
| `cargo test -p tc2 --test identidad` | 7 passed |
| `cargo test -p tc2 --lib plataforma::` | 27 passed |
| `cargo fmt --check -p tc2` | limpio |
| `cargo clippy -p tc2 --all-targets` | sin avisos |

Mutaciones y bordes ejecutados en el clon (todo revertido, `git status` limpio al acabar):

- `MARGEN = 0` y quitar el `self.instalacion.set(None)` del 404: `--test identidades` sigue en **4 passed** (ver IMP-1).
- Prueba temporal en `app_github.rs` (`leer_pem` y `firmar`): PEM PKCS#1 con LF y con CRLF, y PKCS#8, se leen; un client id `Iv1.abc` firma un JWT; un PEM con BOM UTF-8 o con una linea de texto antes de `BEGIN` da `clave-ilegible detalle=pem` (ver MEN-2).
- Por lectura: la logica de los E2E es coherente con la adenda (empujan con `entorno_git` y sin la config del usuario, abren PR/MR con el adaptador, auto-aprobacion del agente y aprobacion de la persona hasta `aprobaciones=1`). No ejecutables aqui.

Lo que sale bien (comprobado): el JWT se verifica con la clave publica exportada por `ssh-keygen`, no con el codigo que firma; el cuerpo de `access_tokens` se compara entero; el test de git comprueba el commit `[bot]`, la cabecera `Basic` recibida por el servidor simulado y la ausencia del token en el registro y en `GIT_TRACE`; la desaparicion de `CredencialesEntorno` no deja referencias en `src/` ni en `tests/` (solo en documentos historicos); `init/inventario.rs` pasa a `tiene(Persona)` sin tocar la red, con el mismo resultado `sin-token` (el test de init lo cubre). Las divergencias 1 a 6 de la adenda son razonables y estan justificadas (en particular 1: dos de los cuatro sitios crean las credenciales antes de conocer el destino; y 3: el autor tiene que valer tambien para `guardar`).

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | abierto | crates/tc2/src/plataforma/app_github.rs:33 y :254-258, :324-339 | implementacion |
| MEN-1 | MENOR | abierto | crates/tc2/src/i18n/catalogo/plataforma.rs (APP_SIN_TOKEN) y crates/tc2/src/plataforma/credenciales.rs:242-245 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/plataforma/app_github.rs:354-360 | implementacion |
| MEN-3 | MENOR | abierto | crates/tc2/src/plataforma/app_github.rs:310-320 y crates/tc2/src/plataforma/github.rs:314-321 | implementacion |
| MEN-4 | MENOR | abierto | crates/tc2/src/plataforma/entorno_git.rs:74 | implementacion |
| MEN-5 | MENOR | abierto | crates/tc2/tests/agente_git/mod.rs:55-72 y crates/tc2/src/plataforma/credenciales.rs:71 | implementacion |

### IMPORTANTE-1 — Protecciones nuevas del token de instalacion sin ningun test que se ponga rojo
- Donde: `crates/tc2/src/plataforma/app_github.rs:33` (`MARGEN`), `:254-258` (reinicio de la instalacion en 404) y `:324-339` (`fallo_estado`, mapa de estado a `causa`).
- Que pasa: tres comportamientos que la adenda presenta como garantias no tienen red de regresion.
  1. **Margen de renovacion** (`expira - MARGEN > ahora`, «hasta `expires_at` menos 5 min»): el test usa un token caducado el año 2000 y otro valido hasta 2099, asi que cualquier margen entre 0 y siglos da lo mismo.
  2. **Olvidar la instalacion cuando `access_tokens` da 404** (App reinstalada con otro id): ningun test la ejercita.
  3. **Las causas `jwt-rechazado` (401), `permisos-no-concedidos` (422), `sinred` y `respuesta-inesperada`**: la adenda las lista como contrato (`denegada motivo=credenciales causa=...`), pero solo estan probadas `sin-instalacion` (404 de `GET installation`), `clave-ilegible`, `app-incompleta` y `app-solo-github`. Un cambio que devolviera `respuesta-inesperada` para un 401, o `denegada` para un fallo de red, deja todo en verde.
- Reproduccion:
  ```
  # en el clon
  sed -i 's/pub const MARGEN: u64 = 5 \* 60;/pub const MARGEN: u64 = 0;/' crates/tc2/src/plataforma/app_github.rs
  sed -i 's/                self.instalacion.set(None);/                \/\/ MUT/' crates/tc2/src/plataforma/app_github.rs
  cargo test -p tc2 --test identidades
  # test la_app_pide_un_token_reducido_lo_guarda_y_lo_renueva ... ok
  # test result: ok. 4 passed; 0 failed
  ```
  Para el punto 3, `grep -rn "jwt-rechazado\|permisos-no-concedidos\|respuesta-inesperada\|causa=sinred" crates/tc2/tests crates/tc2/src` fuera de `credenciales.rs`/`app_github.rs` no da ningun test.
- Impacto: con `MARGEN` a 0 un token con segundos de vida se entrega a una operacion larga (p. ej. un push) y caduca a medias; sin el reinicio, una App reinstalada falla en cada operacion hasta reiniciar el proceso (en un CLI de proceso corto es raro, pero el reinicio existe justo para eso); y el contrato de causas que lee la persona (y `doctor` en F3-8) puede cambiar sin que nada lo note. Arreglo barato, solo tests: un token que vence dentro del margen (`expires_at` = ahora + 4 min) se renueva y uno a +6 min se reutiliza; un 404 en `access_tokens` seguido de otro `obtener` vuelve a pedir `GET installation`; una tabla 401/422/403/sin servidor contra `fallo_estado` y el mock.

### MENOR-1 — El mensaje de `app-solo-github` manda a la persona en la direccion equivocada
- Donde: `crates/tc2/src/i18n/catalogo/plataforma.rs` (`APP_SIN_TOKEN`) y `crates/tc2/src/plataforma/credenciales.rs:242-245`.
- Que pasa: con una App configurada (`TC2_APP_AGENTE_*`) y un remoto GitLab, la salida es `denegada motivo=credenciales rol=agente causa=app-solo-github` y el diagnostico dice «revisa TC2_APP_AGENTE_ID, TC2_APP_AGENTE_CLAVE, que la App este instalada en el repo y que tenga los permisos del rol». Para esa causa la accion correcta es otra: quitar las variables de la App y poner el token de la cuenta de servicio en `TC2_TOKEN_AGENTE` (lo dice la adenda, no el mensaje). Ademas el `detalle` es `app 123`, que no explica nada.
- Reproduccion: por lectura del catalogo y de `credenciales.rs` (la causa se produce en `CredencialesDeEntorno::obtener` con `SinAtar`; el unitario `la_fabrica_da_precedencia_a_la_app_sin_paso_entre_roles` solo comprueba el valor de `causa`, no el texto). No he ejecutado el CLI contra un remoto GitLab real.
- Impacto: quien migra de token a App y trabaja tambien con un repo GitLab recibe una instruccion que no resuelve el problema. Sugerencia: un texto propio para `app-solo-github` («en GitLab no hay Apps: quita TC2_APP_AGENTE_* y usa TC2_TOKEN_AGENTE con un token de proyecto o de cuenta de servicio»).

### MENOR-2 — PEM con BOM o con texto antes de `BEGIN` se rechaza con un detalle opaco
- Donde: `crates/tc2/src/plataforma/app_github.rs:354-360` (`leer_pem`).
- Que pasa: `skip_while(|l| l.is_empty())` no tolera un BOM UTF-8 ni lineas de comentario previas. `Set-Content -Encoding utf8` y `Out-File` de Windows PowerShell 5.1 escriben BOM, y esta herramienta se usa en Windows.
- Reproduccion: prueba temporal (revertida) con `leer_pem(&format!("\u{feff}{pem}"))` y `leer_pem(&format!("# clave\n{pem}"))` da `Err(Acunar { causa: "clave-ilegible", detalle: "pem" })`; LF, CRLF y PKCS#8 si se leen.
- Impacto: el mensaje dice `clave-ilegible: pem` sin indicar la ruta ni el motivo probable. Basta quitar un `\u{feff}` inicial con `trim_start_matches` y/o decir en el detalle «no empieza por -----BEGIN».

### MENOR-3 — Validacion de `dueno/repo` escrita dos veces
- Donde: `crates/tc2/src/plataforma/app_github.rs:310-320` (`repo_valido`, cuyo comentario dice «la misma forma que exige el adaptador») y `crates/tc2/src/plataforma/github.rs:314-321`.
- Que pasa: es la misma decision en dos sitios. Si el adaptador admite un caracter nuevo en el nombre del repo (o endurece una regla), `GitHub::nuevo` lo acepta y `atar_github` lo convierte en `Fallo(app-incompleta, "repo")` solo para quien usa App, con un mensaje que habla de variables de la App.
- Reproduccion: lectura; las dos funciones son identicas hoy (`repo_valido("Foo/bar.baz-1") == true` en la prueba temporal).
- Impacto: incoherencia futura entre token y App. Una funcion `pub(crate) fn repo_github_valido` reutilizada por ambos la evita.

### MENOR-4 — `entorno_git` ocupa `GIT_CONFIG_COUNT=1` y pisa las entradas `GIT_CONFIG_*` que ya hubiera en el entorno
- Donde: `crates/tc2/src/plataforma/entorno_git.rs:74-79`.
- Que pasa: el docstring avisa de que «no se combinan con otras definidas igual», pero `Git::con_entorno_extra` (`git/puerto.rs`) se presenta como «añade variables a las que ya tenga el puerto, sin quitarlas» y para `GIT_CONFIG_*` no es cierto: si el puerto o el proceso ya llevaban `GIT_CONFIG_COUNT=N`, el valor 1 las descarta sin avisar. Hoy ningun llamador lo hace (el arnes E2E usa `GIT_CONFIG_GLOBAL`/`NOSYSTEM`, que no colisionan), por eso es MENOR.
- Reproduccion: lectura de `git/puerto.rs` (`entorno_extra` es un `Vec` aplicado con `cmd.env(k, v)`, el ultimo gana) y `entorno_git.rs`.
- Impacto: cuando F6-2 lo conecte a `review`, una entrada `safe.directory` o similar pasada por `GIT_CONFIG_*` desapareceria sin error. Conviene una nota en `con_entorno_extra` o que `entorno_git` reciba el indice libre.

### MENOR-5 — El arnes E2E saca el token del agente decodificando su propia cabecera `Basic`
- Donde: `crates/tc2/tests/agente_git/mod.rs:55-72` (`en_claro`), consecuencia de `Secreto::exponer` con visibilidad `pub(crate)` (`credenciales.rs:71`).
- Que pasa: para llamar a la API con el token del agente el arnes llama a `entorno_git`, localiza `GIT_CONFIG_VALUE_0`, decodifica el base64 y parte por `:`. Funciona, pero acopla el arnes al formato interno de la cabecera (si cambia el orden de variables o el usuario contiene `:`, rompe el E2E, que nadie puede ejecutar hoy).
- Reproduccion: lectura; no ejecutable (los E2E estan `#[ignore]` y faltan identidades).
- Impacto: fragil, sin riesgo para produccion. Alternativa: que el E2E use `CredencialesApp::obtener` y un `Secreto::exponer` publico solo con `cfg(test)`/feature de pruebas, o un metodo del arnes que haga la peticion con `&Secreto`.

## Divergencias de la adenda y suposiciones

- Divergencias 1 a 6: razonables y bien documentadas; ninguna contradice un criterio de aceptacion. La 4 (cabecera limitada a la URL del remoto) es mejor que el plan. La 6 (PKCS#8) esta probada solo por la prueba temporal de esta ronda; no hay test permanente (no es un hallazgo: es un añadido sin pedir, pero su unico test es el de `pem_que_no_vale`, que no cubre el camino PKCS#8 correcto).
- `de_entorno()` sin argumentos y `atar_github` en el trait: acepto la razon; el coste es que `Credenciales` gana un metodo mutable que solo importa a un implementador (`atar_github`). Nota, no hallazgo.
- Pendiente de verificar contra GitHub y GitLab reales (marcado «no verificado» en la adenda y que sigue asi tras esta ronda): `permissions` + `repositories` en `access_tokens`, `bot: true` de la cuenta de servicio de GitLab, auto-aprobacion de la App rechazada por GitHub, y firma `Basic oauth2:<token>` de GitLab. Los dos E2E ignorados los cubririan; no los he podido ejecutar.
- El criterio de la tarea «la persona aprueba un PR/MR del agente en GitHub y en GitLab» **no queda demostrado** por esta ronda (depende de los E2E con identidades reales). La forma esta cubierta, como dice el plan.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/plataforma/credenciales.rs`, `app_github.rs`, `entorno_git.rs`, `conexion.rs`, `github.rs`, `git/puerto.rs`, `identidad.rs`, `init/inventario.rs`, `inventario.rs` (`ms_de_iso`): son el codigo del diff o lo que sustituye, necesarios para reproducir.
- `crates/tc2/tests/identidades.rs`, `tests/agente_git/mod.rs`, `tests/plataforma_e2e.rs`, `tests/plataforma_gitlab_e2e.rs`: los tests del diff.
- `~/.claude/plugins/.../task-workflow/revision.md`: plantilla y reglas del informe (lo exige la skill).
