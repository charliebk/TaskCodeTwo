# Informe de revision — TASK-047 (ronda 1)

- Commit revisado: 8123fa68e74571def0591f22da57cfa2536e687c
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MENOR-1 | MENOR | abierto | crates/tc2/src/plataforma/credenciales.rs:242 | implementacion |
| MENOR-2 | MENOR | abierto | crates/tc2/src/plataforma/app_github.rs:171 | implementacion |
| MENOR-3 | MENOR | abierto | crates/tc2/src/plataforma/app_github.rs:256 (y conexion.rs:103, app_github.rs:33) | implementacion |
| MENOR-4 | MENOR | abierto | crates/tc2/src/plataforma/app_github.rs:214 | implementacion |
| MENOR-5 | MENOR | abierto | crates/tc2/src/git/puerto.rs:251 | implementacion |

Sin CRITICO ni IMPORTANTE. Seguridad del token, JWT RS256, cache y renovacion, falla cerrada y errores: sin defectos
demostrables (detalle en `## Revisado sin hallazgos`).

### MENOR-1 — Con variables de App presentes, un destino GitLab ignora `TC2_TOKEN_<ROL>` y el mensaje no dice que hacer
- Donde: crates/tc2/src/plataforma/credenciales.rs:242 (rama `SinAtar` de `obtener`); texto en crates/tc2/src/i18n/catalogo/plataforma.rs:105 (`APP_SIN_TOKEN`).
- Que pasa: si `TC2_APP_AGENTE_ID` y `TC2_APP_AGENTE_CLAVE` estan definidas y el repo es de GitLab, el adaptador nunca ata la App (`atar_github` solo lo llama `GitHub::nuevo`), asi que el rol agente sale `denegada motivo=credenciales causa=app-solo-github` aunque `TC2_TOKEN_AGENTE` este definido y sea el valido para GitLab. El mensaje habla de "la GitHub App de agente" y manda a "revisar ... que la App este instalada en el repo", sin decir que en GitLab hay que quitar las variables `TC2_APP_*` y usar `TC2_TOKEN_AGENTE`.
- Reproduccion: test temporal en el clon (borrado) con `GitLab::nuevo(.., Box::new(de_entorno_con(vars, false)), ..)` y vars = App id + ruta de PEM + `TC2_TOKEN_AGENTE=glpat-x`, `prs_abiertos(Rol::Agente)`. Salida: codigo 8, stdout `prs-abiertos denegada motivo=credenciales rol=agente causa=app-solo-github`, stderr `la GitHub App de agente no consiguio su token de instalacion (app-solo-github: app 1): revisa TC2_APP_AGENTE_ID, TC2_APP_AGENTE_CLAVE, que la App este instalada en el repo y que tenga los permisos del rol`. El mock no recibio ninguna peticion (falla cerrada correcta).
- Impacto: quien trabaje con un repo de GitHub y otro de GitLab en la misma shell (o lo tenga en el entorno global) no puede usar el agente en GitLab y el mensaje lo manda por un camino que no lo arregla. Hay salida (quitar las variables), por eso MENOR; no es paso entre roles.
- Sugerencia: mensaje propio para `app-solo-github` ("en GitLab el agente usa TC2_TOKEN_AGENTE: quita TC2_APP_AGENTE_*"), o, si se prefiere, que `SinAtar` en un destino que no es GitHub caiga al token del mismo rol (sigue sin pasar entre roles). Es decision de diseno; lo minimo es el texto.

### MENOR-2 — Un id de App numerico con ceros a la izquierda produce un JWT con JSON invalido
- Donde: crates/tc2/src/plataforma/app_github.rs:171 (`firmar`, `iss`).
- Que pasa: si el id son solo digitos se emite sin comillas. `0123` pasa `id_valido` y la carga queda `{"iat":..,"exp":..,"iss":0123}`, que no es JSON.
- Reproduccion: test temporal en el clon (borrado) con `CredencialesApp::nueva("0123", ..)` y mock; se decodifico la carga del JWT enviado: `{"iat":1795083738,"exp":1795084338,"iss":0123} valido_json=false`. Con `Iv1.abc` sale `"iss":"Iv1.abc"`, valido.
- Impacto: GitHub respondera 401 (`jwt-rechazado`) a una configuracion que tc2 dio por buena. Los ids reales de App no empiezan por 0, asi que es de borde.
- Sugerencia: emitir como numero solo si `id.parse::<u64>()` lo reproduce tal cual (sin ceros), si no, como texto; o rechazar con `app-incompleta`.

### MENOR-3 — Tres protecciones del diff sobreviven a la mutacion
- Donde: crates/tc2/src/plataforma/app_github.rs:256 (reinicio de la instalacion en 404), crates/tc2/src/plataforma/conexion.rs:103 (`causa == SINRED` -> `Resultado::Sinred`), crates/tc2/src/plataforma/app_github.rs:33 (`MARGEN`).
- Que pasa: ningun test se pone rojo si se quitan.
- Reproduccion (en el clon, revertida cada una con `git checkout`; se ejecuto `cargo test -p tc2 --test identidades` tras cada una):
  - M6 `if causa == causas::SINRED` -> `if false`: 4/4 verdes. Sin red al acunar saldria `denegada ... causa=sinred` en lugar de `sinred`.
  - M7 quitar `self.instalacion.set(None)` en 404 de `access_tokens`: 4/4 verdes. Una App reinstalada con otro id quedaria con el id viejo cacheado hasta reiniciar el proceso.
  - M12 `MARGEN = 0`: 4/4 verdes. Nadie comprueba el margen de 5 min (los tests usan 2000 y 2099).
  El comportamiento SI es correcto cuando lo comprobe a mano con un test temporal (borrado): tras 404 se vuelve a pedir `GET /repos/o/r/installation` (lineas `GET installation, POST 5/access_tokens, GET installation, POST 6/access_tokens`) y tras ello sale token; sin red da `sinred|io: ...` y `sinred` en la causa; pero no queda red de seguridad en la suite.
- Impacto: una regresion futura en esas tres zonas pasaria la suite.
- Sugerencia: tres asserts baratos con `http_mock`: 404 en `access_tokens` seguido de segundo `GET installation`; base inalcanzable (`http://127.0.0.1:9`) -> `Resultado::Sinred`; `expires_at` a ahora+4 min (se pide de nuevo) y ahora+10 min (cacheado).
- Tambien sin cobertura en la suite (comprobados a mano, funcionan): clave PKCS#8 (`BEGIN PRIVATE KEY`, de `ssh-keygen -m PKCS8`), PEM con CRLF, clave de 1024 bits (`clave-ilegible|TooSmall`), 401 -> `jwt-rechazado`, 422 -> `permisos-no-concedidos`, `expires_at` con fraccion u offset, token con caracteres de control -> `respuesta-inesperada|token`.

### MENOR-4 — Un fallo TLS (u otro `FalloHttp` que no es `Sinred`) sale como `respuesta-inesperada` con el `Debug` de la variante
- Donde: crates/tc2/src/plataforma/app_github.rs:214 (`llamar`, brazo `otro`).
- Que pasa: solo `Sinred` se traduce; `Tls(String)`, `Redireccion`, `CuerpoGrande`, etc. salen con `causa=respuesta-inesperada` y `detalle` = `format!("{otro:?}")` saneado (p. ej. `Tls("...")`). Hallazgo por lectura del codigo; no se pudo provocar un fallo TLS real en el clon.
- Impacto: el diagnostico dice "respuesta inesperada: Tls(...)" a quien tiene un problema de certificado o de almacen del sistema; no se distingue de un fallo de formato. No filtra secretos (`Cliente` no pone cabeceras en sus errores).
- Sugerencia: causa propia o al menos mapear `Tls` a un texto que diga que revise el almacen de certificados.

### MENOR-5 — `con_entorno_extra` y `con_entorno` se pisan segun el orden
- Donde: crates/tc2/src/git/puerto.rs:251 (`con_entorno` asigna `self.entorno_extra = vars`) frente a :261 (`con_entorno_extra` extiende).
- Que pasa: `Git::nuevo().con_entorno_extra(entorno_git(..)).con_entorno(otro)` descarta sin aviso la cabecera de autenticacion y el autor del agente; en el otro orden se conservan. Ademas `GIT_CONFIG_COUNT=1` pisa cualquier `GIT_CONFIG_COUNT` del entorno del proceso (el doc de `entorno_git` ya avisa de que ocupa la entrada 0 y no se combina).
- Reproduccion: por lectura de `comando` (puerto.rs:468: aplica `entorno_extra` tal cual). No se ejecuto: hoy `con_entorno` solo lo usan los tests y F6-2 aun no conecta el agente.
- Impacto: ninguno hoy; el fallo seria cerrado (git sin cabecera falla con 401/prompt desactivado), no una fuga. Riesgo de confusion cuando F6-2 lo conecte.
- Sugerencia: nota en el doc de `con_entorno` ("sustituye, tambien lo anadido por `con_entorno_extra`") o que `con_entorno` tambien extienda.

## Alcance
- Ficheros revisados: crates/tc2/Cargo.toml, src/git/puerto.rs, src/i18n/catalogo/plataforma.rs, src/identidad.rs, src/init/inventario.rs, src/init/mod.rs, src/lib.rs, src/plataforma/{app_github,conexion,credenciales,entorno_git,github,mod}.rs, tests/{agente_git/mod,identidades,plataforma_e2e,plataforma_gitlab_e2e}.rs.
- Contexto adicional pedido: ver `## Lecturas fuera del contexto permitido`.
- Limite JS-WASM: no aplica. El diff no toca `#[wasm_bindgen]` ni tipos que cruzan a JavaScript; el crate es solo nativo.

## Reproduccion
- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-047-r1-rust`, rama feature/task-047-f6-1-identidades-agente-implementador, HEAD ebb6ad8 (contenido de codigo igual a 8123fa6; el ultimo commit solo anade la peticion). Target propio dentro del clon. Sin commits ni push.
- Build: compilan `tc2` y sus tests en nativo (via `cargo test`/`cargo clippy`, `--locked` en clippy). No se compilo `wasm32` (el crate no es WASM).
- Formato y lints: `cargo fmt --check` sin diferencias; `cargo clippy -p tc2 --all-targets --locked -- -D warnings` sin avisos.
- Suite: no se ejecuto la suite completa (instruccion del encargo) y la peticion no traia verificacion en verde. Se ejecutaron: `--test identidades` 4/4 verdes; `--lib plataforma::` 27/27 verdes; `--test identidad` y `--test plataforma` verdes. Los E2E `persona_aprueba_pr_del_agente` y `persona_aprueba_mr_del_agente` (ignorados) no se pueden ejecutar: se revisaron leyendo.
- Casos construidos (tests temporales en el clon, borrados): PKCS#8, PEM CRLF, clave 1024 bits, rol Persona y repo `o/../r` en `CredencialesApp::nueva`, `expires_at` con fraccion/offset/solo fecha, 401, 422, 404 en access_tokens con reinstalacion, sin red, token con control, GitLab con variables de App, id `0123` y `Iv1.abc`. Tambien `git config --get-urlmatch http.extraheader` con git 2.56 y la clave `http.https://github.com/.extraheader` (y sin barra final, y vacia): la cabecera solo casa con `https://github.com/...`; no casa con `evil.example`, `github.com.evil.example` ni `github.comx`; con URL vacia (`http..extraheader`) no casa con nada (falla cerrado).
- Mutacion (cada una revertida con `git checkout`, contra `--test identidades`; M5 contra `--lib`):
  - M1 `ATRAS` 60->0: rojo (`la_app_pide_un_token_reducido...`).
  - M2 cache desactivada: rojo (mismo test).
  - M3 `rol != self.rol` -> `false`: rojo (mismo test).
  - M4 `repositories: []`: rojo (mismo test).
  - M5 persona sin tty: rojo (`la_fabrica_da_precedencia_a_la_app_sin_paso_entre_roles`).
  - M9 usuario git `x-access-token` -> `oauth2`: rojo (`el_entorno_git_del_agente...`).
  - M13 vida del JWT 9->11 min: rojo (mismo test que M1).
  - Sobreviven: M6, M7, M12 (ver MENOR-3).
- Medidas: no aplican (sin bucles calientes ni limite WASM).

## Revisado sin hallazgos
- Token nunca en argv: `entorno_git` lo pone solo en `GIT_CONFIG_VALUE_0` (base64 de `usuario:token`) y el test `el_entorno_git_del_agente...` hace commit y push reales contra el mock HTTP, comprueba la cabecera recibida y que ni el token ni su base64 estan en `Git::registro()` ni en `GIT_TRACE` (incluye `git-remote-http`). La API usa el token por `Authorization: Bearer`, no por URL ni consulta.
- Cabecera limitada a la URL del remoto: comprobado con `--get-urlmatch` (arriba); la URL vacia no abre el ambito.
- `Debug` opaco: `Secreto`, `CredencialesApp` (id/rol/repo, sin clave ni cache), `CredencialesDeEntorno` (solo claves de los mapas) y `FalloCredencial` (sin secretos); `detalle` pasa por `sanear` y el test comprueba que no lleva el token de la variable.
- JWT RS256: cabecera `{"alg":"RS256","typ":"JWT"}`, `iat` = ahora-60, `exp` = ahora+540 (<10 min), `iss` numerico o texto; firma `RSA_PKCS1_SHA256` de `ring`, buffer del tamano del modulo; el test la verifica con la clave publica sacada por `ssh-keygen -e` (independiente del codigo probado). `from_der` acepta el PKCS#1 de GitHub (suposicion del plan confirmada) y PKCS#8; PEM cifrado, mal cerrado, no RSA o base64 invalido dan `clave-ilegible` sin panic (test `pem_que_no_vale`).
- Cache y renovacion: se reutiliza hasta `expires_at - MARGEN`, se renueva al caducar, la instalacion se busca una vez; el `RefCell` no se toma en mutable mientras hay un prestamo (el test de tres llamadas pasa y no hay `BorrowMutError`).
- Falla cerrada sin paso entre roles: la App de un rol devuelve `Falta` para otro; con App configurada nunca se cae al token de la variable (ni a medias: `app-incompleta`); persona solo con terminal; `Conexion::token` distingue `Falta` (rechazada/identidad) de `Acunar` (denegada/credenciales o sinred).
- Errores: no hay `unwrap`/`expect`/indexado sobre datos de entrada en codigo de produccion del diff (`unwrap_or_default` sobre `split('/').nth(1)` y `permisos()` esta protegido por `repo_valido` y `permisos(rol).is_none()` en `nueva`); `u64::try_from` y `saturating_sub` evitan desbordes; los `panic!` estan en tests con `#[allow(clippy::panic)]`.
- Permisos por rol: coinciden con la tabla 2 del ADR (test `los_permisos_por_rol...`); el cuerpo pide `repositories` y `permissions` exactos y reducidos al repo.
- Mensajes: cada causa lleva su texto con las variables a revisar; el de sin-instalacion dice la comprobacion ("que la App este instalada en el repo", variables `TC2_APP_*`), salvo el caso de MENOR-1.
- Dependencias: `ring 0.17.14` y `base64 0.23.1` ya estaban en `Cargo.lock` (por rustls/ureq); no entran crates nuevos. El crate es nativo, sin impacto en `.wasm`.
- E2E ignorados (lectura): el agente empuja con `tc2::git::push` con entorno propio y fuera de `~/.gitconfig` (`GIT_CONFIG_NOSYSTEM`/`GIT_CONFIG_GLOBAL`), el PR/MR lo abre el agente, la autoaprobacion no cuenta y la persona aprueba (`aprobaciones` 0 -> 1); la limpieza va en `Drop`. `en_claro` saca el token de la cabecera para llamadas del arnes: solo en tests y sin imprimirlo.
- `lib.rs`, `init/mod.rs`, `identidad.rs`, `init/inventario.rs`: la sustitucion por `de_entorno()` y `tiene()` mantiene el comportamiento (`tiene` no toca la red).

## Lecturas fuera del contexto permitido
- crates/tc2/src/plataforma/http.rs: para ver como `Cliente` pone la cabecera, valida la ruta y clasifica errores (alcance de MENOR-4 y seguridad del token en errores).
- crates/tc2/src/plataforma/errores.rs: `sanear` (que `detalle` no arrastra caracteres de control ni mas de 200 caracteres).
- crates/tc2/src/inventario.rs: `ms_de_iso` (formatos de `expires_at` admitidos).
- crates/tc2/tests/comun/http_mock.rs: API del mock para construir los casos extra.
- crates/tc2/src/git/puerto.rs (alrededor de las lineas 196-265 y 440-490): `ENTORNO_FIJO`, `ENTORNO_QUITADO` y `comando`, para el orden de variables (MENOR-5).
- `git diff` de plataforma_e2e.rs, plataforma_gitlab_e2e.rs y agente_git/mod.rs contra `origin/develop` (la peticion no los embebe).
- Carpeta de la tarea: tarea.md y planificacion/plan-final.md (permitidos).
