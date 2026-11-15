# Informe de revision — TASK-021 (ronda 1)

- Commit revisado: aa704e87c860ed49942176ea8c711ba40a83e567
- Revisor: rust-wasm-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/tests/plataforma.rs:326 | implementacion |
| IMP-2 | IMPORTANTE | corregido | crates/tc2/src/plataforma/deteccion.rs:81 | especificacion |
| IMP-3 | IMPORTANTE | corregido | crates/tc2/src/plataforma/github.rs:514 | implementacion |
| IMP-4 | IMPORTANTE | corregido | crates/tc2/src/plataforma/mod.rs:64 | implementacion |
| MEN-1 | MENOR | corregido | crates/tc2/src/plataforma/github.rs:537 | plan |
| MEN-2 | MENOR | corregido | crates/tc2/src/plataforma/http.rs:90 | implementacion |
| MEN-3 | MENOR | corregido | crates/tc2/src/plataforma/github.rs:646 | implementacion |
| MEN-4 | MENOR | corregido | crates/tc2/src/plataforma/github.rs:606 | implementacion |
| MEN-5 | MENOR | corregido | crates/tc2/src/plataforma/github.rs:554 | plan |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### IMP-1 — `el_envoltorio_rechaza_antes_de_la_red` es intermitente (falla cerca de la mitad de las veces aqui) y tarda 19 s al fallar

- Reproduccion: `cargo test -p tc2 --locked` (suite completa, una vez) falla en `tests/plataforma.rs:377` con `left: "identidad sinred"`, `right: "identidad error motivo=respuesta-invalida"`. Repetido en solitario: 1 fallo en 6 ejecuciones; un test aparte que repite 12 veces solo el caso del cuerpo de 1,1 MB contra el mock dio 6 fallos de 12. Los fallos tardan siempre ~19 s y el diagnostico es `io: Peer disconnected`; los aciertos tardan ~20 ms y dicen `the response body is larger than request limit: 1048576`.
- Efecto: el caso «cuerpo > 1 MiB» (criterio del plan: `respuesta-invalida`) depende de una carrera entre el mock (que escribe 1,1 MB con `Connection: close`) y el cliente; cuando pierde, el adaptador contesta `sinred` («comprueba la red y reintenta») a una respuesta demasiado grande. No he podido separar si la causa esta solo en el mock (`tests/comun/http_mock.rs`) o si `clasificar` (`http.rs:282`) tambien deberia tratar `Peer disconnected` tras leer las cabeceras como respuesta invalida. En cualquier caso la suite de la rama no es fiable en Windows.
- Consecuencia para la mutacion: este test es el unico que cubre `LIMITE_CUERPO` (M10, limite a 1 GiB, solo lo detecta esta prueba), de modo que esa red es inestable.

### IMP-2 — `detectar` acepta una URL de API arbitraria para un origin `github.com` o `gitlab.com`

- Reproduccion (test descartable en el clon): `detectar("https://github.com/o/r", Some(("github", "https://evil.example/api")))` devuelve `Ok` con `api = https://evil.example/api`. La rama `(Some(h), Some((_, base))) => (h, base)` (`deteccion.rs:81`) solo comprueba que el proveedor coincida, no que la base sea la de ese host.
- Efecto: quien controle la clave de `.taskcode/config.yml` (F3-1 la leera del repo) redirige a su servidor el token de agente, revisor o persona (`Cliente` envia `Authorization: Bearer` a la base). El ADR F1-13 D8a dice que la clave solo es obligatoria para hosts desconocidos y que si contradice el host se falla cerrado; una URL de API que no es la del host contradice igualmente. Propongo: con host `github.com` solo valer `https://api.github.com` (idem `gitlab.com`) o rechazar con `motivo=plataforma`.
- Origen `especificacion`: el plan dice que la clave declara la URL de la API sin acotarla para los hosts conocidos. El test `falla_cerrado` no tiene este caso.

### IMP-3 — El filtro por `integration_id` (G09) no lo detecta ningun test

- Mutante M01: quitar `&& r.app.as_ref().is_some_and(|a| a.id == *origen)` (`github.rs:514`). `cargo test -p tc2 --lib --test plataforma` sigue en verde (incluido el test inestable).
- Efecto: la suite deja pasar un check `verificacion` publicado por otra App (el ataque que G09 y la PoC M debian cubrir). Ningun fixture de la bateria trae un check con el nombre correcto y un `app.id` distinto; solo se cubre el nombre y el estado.
- Arreglo: caso en la bateria con `check_runs` donde `name` coincide, `app.id` es otro y `conclusion=success` => `esperando falta=<check>`.

### IMP-4 — `Usuario::es_persona` sin test de bot en la lista ni de `web-flow`

- Mutantes M04 (quitar `tipo == "User"`) y M05 (quitar `login != "web-flow"`): sobreviven con la suite sin el test inestable. El caso `aprobacion-bot` usa un bot que no esta en `Personas`, asi que solo ejerce `contiene(id)`.
- Las reglas 3 y 4 de F1-6 («Bot metido en la lista por error», «web-flow metido por error») estan como casos de la PoC D y no tienen equivalente en el adaptador. Falta un test con `Personas` que incluya el id del bot y el de `web-flow`, en reviews y en `Firma::cuenta_como_aprobacion`.

### MEN-1 — Una review `COMMENTED` posterior anula una `APPROVED` previa de la misma persona

- Reproduccion (mock): reviews `[APPROVED id=1001, COMMENTED id=1001]`, persona 1001 en la lista => `estado-puertas esperando pr=12 aprobaciones=0 ignoradas=0 falta=aprobacion`. `github.rs:537` hace `ultima.insert` para cualquier estado. En GitHub un comentario posterior no retira la aprobacion (solo `CHANGES_REQUESTED` o `DISMISSED` la sustituyen), asi que la puerta falla cerrada en un caso corriente (el revisor aprueba y luego comenta).
- Origen `plan`: el plan decide seguir la PoC (ultima review por usuario) y lo marca como duda de dominio sin validar. No hay riesgo de seguridad, si de bloqueo. Sugiero ignorar `COMMENTED` y `PENDING` al elegir la ultima.

### MEN-2 — `Base::nueva` admite una consulta en el prefijo y puertos no validos

- Reproduccion: `Base::nueva("https://h.example/api?x=1")` da `Ok`, y las llamadas formarian `https://h.example/api?x=1/repos/...`; `https://h.example:99999999/api` tambien es `Ok`. Viene de que `validar_ruta` admite `?` y de que el host acepta cualquier `:` y digitos. Solo afecta a configuracion propia; falla despues como `sinred` o `error cliente`, sin consecuencias de seguridad. Basta rechazar `?` y exigir puerto 1..65535.

### MEN-3 — `leer_permisos_ci`, `firma_commit` y `leer_puertas` solo se prueban en el E2E ignorado

- Mutantes M33 (invertir `can_approve_pull_request_reviews`), M42 (`verificada: true` siempre) y M45 (permitir rol revisor en `leer_puertas`) sobreviven. Tambien M15 (`estado-puertas` acepta rol revisor), M29 (PR cerrado no se rechaza) y M43 (`sha_valido` del head). El E2E si ejercita parte (`ci-aprueba=false`, `firma_commit`), pero no se ejecuta en la suite y el plan prometia fixtures «no observado» para estos casos. Falta al menos un caso de mock por operacion con su rama de rechazo.

### MEN-4 — `merge-pr` informa error aunque el merge se haya hecho si la respuesta 2xx no trae `sha`

- `github.rs:606`: con `merged: true` y `sha` ausente o corto, devuelve `error motivo=respuesta-invalida` sin `commit=`. La persona vera un error cuando el PR ya esta mergeado (la siguiente llamada daria `sin-cambios`). Mejor devolver `ok` con `pr=` y sin `commit`, o decir en el diagnostico que el merge consta como hecho.

### MEN-5 — `mergeable_state` desconocido o `dirty` cuenta como al dia

- `github.rs:554`: `al_dia = estado != Some("behind")`; `unknown` (GitHub lo calcula de forma diferida tras un push) y `dirty` (conflictos) dan `al_dia=true`. El mutante M16 (`al_dia: true` siempre) sobrevive: ningun test cubre `behind`. El merge lo frena GitHub (405), pero `estado-puertas` puede decir `ok` antes. El plan lo dejaba como duda de dominio sin resolver; al menos hace falta test de `behind` y decidir `unknown`.

## Lecturas fuera del contexto permitido

- `docs/adr/F1-6-identidades-agente-revisor-persona.md` (lineas ~255-290) y `docs/adr/F1-13-adaptador-de-plataforma.md` (G04): para comprobar si «COMMENTED no cuenta» y si las aprobaciones obsoletas las trata la plataforma (G04 `dismiss_stale_reviews_on_push`; por eso no lo reporto como hallazgo).
- `tareas/03-en-revision/TASK-021/evidencia/e2e.log` (lineas 1-10): para confirmar la forma real de una review `COMMENTED`.

## Alcance
- Ficheros revisados: los 13 de la peticion (Cargo.toml, lib.rs, plataforma/{credenciales,deteccion,errores,github,http,mod,puertas}.rs, tests/plataforma.rs, tests/plataforma_e2e.rs, tests/comun/{http_mock,mod}.rs).
- Contexto adicional pedido: ADR F1-6 y F1-13 (ver arriba).
- Limite JS-WASM: sin cambios; el diff es Rust nativo, sin `wasm_bindgen` ni objetivo `wasm32`.

## Reproduccion
- Clon: `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-021-r1-rust\repo`, rama `feature/task-021-f2-6-adaptador-github` (HEAD 14aae54, solo anade la peticion sobre el commit revisado); `CARGO_TARGET_DIR` en la misma carpeta (se borra al terminar).
- Build: `cargo test` y `cargo clippy --locked` compilan en nativo. No se compila para `wasm32` (no aplica).
- Formato y lints: `cargo fmt --check` sin diferencias; `cargo clippy -p tc2 --all-targets --locked -- -D warnings` limpio. En `src/plataforma` no hay `unwrap`, `expect` ni `panic!` en codigo de produccion.
- Suite: `cargo test -p tc2 --locked` (una vez): lib 39 ok y los binarios anteriores a `plataforma` ok; en `plataforma` falla `el_envoltorio_rechaza_antes_de_la_red` (IMP-1) y cargo corta ahi, de modo que los binarios posteriores no se ejecutaron en la linea base.
- Casos construidos: cuerpo de 1,1 MB x12 (IMP-1); `detectar` github.com con API ajena (IMP-2); reviews `[APPROVED, COMMENTED]` (MEN-1); `Base::nueva` con `?` y puerto enorme (MEN-2). Tests descartables, eliminados.
- Mutacion: 45 mutantes manuales sobre `--lib` y `--test plataforma`. Los que solo cayeron por el test inestable se repitieron con `--skip el_envoltorio_rechaza_antes_de_la_red` para no darlos por muertos por la carrera. Detectados por algun test fiable: ultima review frente a primera (M02), `es_persona` sin lista (M06), validacion de rutas `.`/`..`/`%2e` (M07, M08), `ruta_de` a otro host (M11), redirecciones (M12), merge sin comprobar head (M13), merge sin exigir persona (M14), `rama_valida` y `sha_valido` (M17, M18), tabla de errores 405, limite, plan y 401 (M19-M22, M41), saneado de controles (M23), paginacion (M26), `already exists` (M28), deteccion (M30, M31), http no loopback (M32), `merge_method` (M35), checks pendientes y neutral (M36, M37), orden y aprobacion (M38, M40), `sinred` (M39), cache de identidad (M44), filtro de persona (M34). Sobreviven: M01 (IMP-3), M04 y M05 (IMP-4), M33, M42, M45, M15, M29, M43 (MEN-3), M16 (MEN-5), M24 (`take(200)` del saneado), M25 (TTY del token de persona), M27 (filtro head/base en `pr_abierto`) y M10 (limite de cuerpo, solo cubierto por el test inestable). M03 y M09 son debiles o equivalentes (la clave del mutante colisiona; la barra invertida ya la rechaza el filtro de caracteres).
- Medidas: no aplican (sin limite JS-WASM). Dependencias nuevas: `ureq` 3.5 con `rustls` y `platform-verifier`, y `serde_json`.

## Revisado sin hallazgos
- Envoltorio HTTP (`http.rs`): lista cerrada de metodos, `max_redirects(0)` con `max_redirects_will_error(false)`, limite de cuerpo, `ruta_de` solo a la misma base, registro sin cabeceras, el token no sale en errores (`Secreto` sin `Display`, `Debug` enmascarado).
- Tabla de errores (`errores.rs`): orden de ramas (401, limite, plan, 403), saneado y recorte de texto externo, calculo de `reintenta-en` sin desbordes (`saturating_sub`).
- Idempotencia de `abrir_pr` (lectura previa, 422 `already exists` y relectura); `merge_pr` con `sha` esperado y comprobacion previa del head.
- Credenciales por rol: rol sin token da `rechazada motivo=identidad` sin probar otro; token de persona solo con TTY.
- `puertas::evaluar`: logica pura y orden de `falta`, compartida por GitHub y GitLab.
