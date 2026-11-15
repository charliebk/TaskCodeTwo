# Informe de revision — TASK-021 (ronda 1)

- Commit revisado: aa704e87c860ed49942176ea8c711ba40a83e567
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/tests/plataforma.rs:377 | implementacion |
| IMP-2 | IMPORTANTE | corregido | crates/tc2/src/plataforma/github.rs:660 | implementacion |
| MEN-1 | MENOR | corregido | crates/tc2/src/plataforma/github.rs:554 | implementacion |
| MEN-2 | MENOR | corregido | crates/tc2/src/plataforma/github.rs:534 | plan |
| MEN-3 | MENOR | corregido | crates/tc2/src/plataforma/github.rs:518 | plan |
| MEN-4 | MENOR | corregido | crates/tc2/src/plataforma/deteccion.rs:82 | implementacion |
| MEN-5 | MENOR | corregido | crates/tc2/tests/plataforma_e2e.rs:379 | implementacion |
| MEN-6 | MENOR | corregido | crates/tc2/src/plataforma/errores.rs:50 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Puerta determinista (en un clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-021-r1-calidad`, rama `feature/task-021-f2-6-adaptador-github`, HEAD 14aae54, que solo anade la peticion sobre aa704e8): `cargo fmt --check -p tc2` limpio; `cargo clippy -p tc2 --all-targets` sin avisos; `cargo test -p tc2` una vez: lib 39 ok, `plataforma` 3 ok, `plataforma_e2e` 2 ignorados (no se ejecutaron: su evidencia es `evidencia/e2e.log`), resto en verde, 0 fallos. Esa pasada fue verde, pero IMP-1 muestra que `plataforma` no es determinista.

Mutantes manuales (sobre el clon; por mutante, `cargo test -p tc2 --lib` y `--test plataforma`; los que dan flake se repitieron con `-- bateria errores` para no confundirlos):

| Mutante | Resultado |
|---|---|
| quitar `merge_method`/cambiarlo a squash (github.rs:591) | muerto (bateria) |
| quitar el `sha` evaluado: `head-cambiado` (github.rs:585) | muerto (bateria) |
| quitar el relleno `already exists` de abrir-pr (github.rs:434) | muerto (bateria) |
| quitar el atajo de PR mergeado en estado-puertas (github.rs:471) | muerto (bateria) |
| ignorar `GARANTIAS_DE_RULESETS` (github.rs:495) | muerto (bateria) |
| contar cualquier review en vez de la ultima por id (github.rs:537) | muerto (bateria, paginas) |
| 403 de plan, 422 sin commits (errores.rs) | muertos (errores) |
| `integration_id` ignorado en los checks (github.rs:514, G09) | SUPERVIVE |
| `al_dia` siempre true (github.rs:554) | SUPERVIVE |
| `conclusion == success` siempre true (github.rs:518) | SUPERVIVE |
| invertir `can_approve_pull_request_reviews` en `leer_permisos_ci` (github.rs:660, G16) | SUPERVIVE |
| `es_persona` sin la exclusion `web-flow` (mod.rs:76) | SUPERVIVE |
| `es_persona` sin comprobar `type == User` (mod.rs:76) | SUPERVIVE |
| `firma_commit` con `verified` negado (github.rs:687) | SUPERVIVE |
| `Firma::cuenta_como_aprobacion` sin exigir `verificada` (mod.rs:93) | SUPERVIVE |
| `leer_puertas` con `plan_da_reglas: true` en el aviso de plan (github.rs:628) | SUPERVIVE |
| quitar `pr-cerrado` en merge-pr (github.rs:579) y en estado-puertas (github.rs:474) | SUPERVIVEN |
| quitar la comprobacion `sha-invalido` de merge-pr (github.rs:560) | SUPERVIVE |
| `MAX_PAGINAS` de 10 a 1000 (github.rs:30; `demasiadas-paginas`) | SUPERVIVE |
| limite de peticiones sin la rama `x-ratelimit-remaining: 0` (errores.rs:51) | SUPERVIVE |
| espera por `x-ratelimit-reset` (errores.rs:57) | SUPERVIVE |

### IMP-1 — El test `el_envoltorio_rechaza_antes_de_la_red` es intermitente (y la respuesta de mas de 1 MiB a veces sale como `sinred` tras 19 s)
- Donde: crates/tc2/tests/plataforma.rs:377 (caso «Cuerpo de mas de 1 MiB»); camino de produccion: crates/tc2/src/plataforma/http.rs:249 y :282
- Que pasa: con el mock sirviendo un cuerpo de 1,1 MB, a veces el adaptador devuelve `identidad sinred` (codigo 4) en vez de `identidad error motivo=respuesta-invalida` (codigo 1), y la llamada tarda 19 s (o hasta 32 s en una pasada de la suite completa) en lugar de 0,05 s.
- Reproduccion: en el clon, `cargo test -p tc2 --test plataforma el_envoltorio` repetido 8 veces: 3 fallos (`left: "identidad sinred"`, `right: "identidad error motivo=respuesta-invalida"`, `plataforma.rs:377`, `finished in 18.97s`) y 5 pasadas de 0,05 s. `cargo test -p tc2 --test plataforma` entero repetido 6 veces: 2 fallos, y en otra pasada la suite tardo 32,50 s aun pasando. La pasada unica de la puerta salio verde de casualidad. Los mutantes lanzados con la suite entera dieron falsos «muertos» por este fallo hasta que se filtro por nombre de test.
- Impacto: el test cuenta como cobertura del limite de 1 MiB (criterio 2, prueba negativa del envoltorio) pero falla un tercio de las veces en esta maquina Windows, de modo que `cargo xtask verificar` puede dar rojo sin que cambie el codigo, o verde sin que se haya comprobado nada si se reintenta. No se ha atribuido con certeza a uno de los dos lados: o el mock de un solo hilo (`tests/comun/http_mock.rs`, `write_all` de 1,1 MB a un cliente que corta al llegar al limite) o el cliente, que clasifica como `Sinred` (`BodyStalled`/`Io`, http.rs:282-288) un corte que debia ser `RespuestaInvalida`. Si fuera el cliente, un servidor real que mande un cuerpo enorme daria «comprueba la red» tras 19 s en vez del error dirigido. Hay que hacerlo determinista (p. ej. que el mock lea/cierre limpio, o que el cliente trate el limite excedido antes de cualquier error de E/S) y volver a correrlo en bucle.

### IMP-2 — Protecciones de la identidad y de las puertas sin ningun test que se ponga rojo al quitarlas
- Donde: crates/tc2/src/plataforma/github.rs:660 (G16), github.rs:514 (G09), crates/tc2/src/plataforma/mod.rs:76 y :93 (contrato del firmante), github.rs:687 (`firma_commit`)
- Que pasa: la bateria offline (`tests/plataforma.rs`) solo recorre `estado-puertas`, `abrir-pr`, `merge-pr` y los errores de `identidad`. El resto de lo que el ADR F1-6 (decisiones 1, 3 y 5) y F1-13 (decision 4) piden como proteccion solo lo toca el E2E `#[ignore]`, y este unicamente observa el camino benigno (`ci-aprueba=false`, firma `web-flow` verificada, 0 aprobaciones).
- Reproduccion: mutantes del cuadro de arriba, todos con `cargo test -p tc2 --lib` + `--test plataforma` en verde:
  - `leer_permisos_ci`: `if !p.can_approve_pull_request_reviews` (invertido) sigue verde. Nadie prueba que `can_approve_pull_request_reviews: true` da `rechazada motivo=ci-puede-aprobar`, que es toda la garantia G16 de F1-6 decision 4.
  - `estado_puertas`: quitar `a.id == *origen` (que el check lo haya publicado su App, G09) sigue verde: ningun fixture trae un check con el nombre bueno y el `integration_id` ajeno.
  - `Usuario::es_persona`: quitar `login != "web-flow"` o `tipo == "User"` sigue verde (el fixture `aprobacion-bot` falla por el `id` fuera de la lista, no por el tipo, asi que la regla 3/4/6 no se discrimina).
  - `firma_commit` y `Firma::cuenta_como_aprobacion`: negar `verified` o no exigirlo sigue verde; no hay test offline de la lectura ni de la regla 1 a 4 del firmante.
  - `leer_puertas`: poner `plan_da_reglas: true` en el aviso de plan sigue verde; `leer_puertas` entera no tiene test offline.
- Impacto: son precisamente las comprobaciones que impiden que una IA o un bot cuente como persona, o que un check falsificado cuente como verde. Se pueden romper en un refactor (o F2-9 puede divergir) sin que la suite lo note. Falta un test offline por proteccion contra el mock (el mock y los fixtures ya existen): CI puede aprobar, check con otro `integration_id`, review/committer `Bot` y `web-flow` con el `id` en la lista, firma no verificada, aviso de plan en `leer-puertas`.

### MEN-1 — Caminos de error y bordes sin cubrir en el offline
- Donde: crates/tc2/src/plataforma/github.rs:554, :474, :579, :560, :309 y crates/tc2/src/plataforma/errores.rs:51, :57
- Que pasa: sobreviven los mutantes de `al_dia` (`mergeable_state == behind` nunca produce `falta=al-dia` en un test del adaptador; solo lo prueba `puertas::evaluar` con `Lecturas` ya construidas), `pr-cerrado` de `merge-pr` y de `estado-puertas`, `sha-invalido` de `merge-pr`, el tope `demasiadas-paginas` y, en el limite de peticiones, el caso real de GitHub (403 con `x-ratelimit-remaining: 0` y `x-ratelimit-reset`, sin `retry-after`) y el calculo de espera por `reset`: los tests siempre mandan `retry-after`.
- Reproduccion: mutantes del cuadro de arriba, suite verde en todos.
- Impacto: ninguno hoy (el codigo que he leido es correcto); es solo cobertura. Se anota para que quien amplie la bateria (F2-9) los incluya, sobre todo el limite primario de GitHub, que es el caso mas probable en produccion.

### MEN-2 — La «ultima review por cuenta» descarta una aprobacion si despues llega un comentario
- Donde: crates/tc2/src/plataforma/github.rs:534
- Que pasa: `ultima.insert(u.id, (rv.state, ...))` deja la ultima review de cada cuenta sea cual sea su estado. Una persona de la lista que aprueba y despues comenta (`COMMENTED`) pasa a `aprobaciones=0` y `falta=aprobacion`; en GitHub un comentario posterior no retira la aprobacion (si lo hacen `CHANGES_REQUESTED` y `DISMISSED`). Tampoco distingue una review `PENDING` (borrador, solo la ve su autor).
- Reproduccion: lectura del codigo y del test (`plataforma.rs:264-320`, que solo prueba `CHANGES_REQUESTED`). No verificado contra GitHub: habria que aprobar un PR con una segunda cuenta, comentar, y comparar `estado-puertas` con la caja de merge.
- Impacto: falla hacia el lado seguro (bloquea en vez de dejar pasar), pero `finish` diria «falta aprobacion» con la aprobacion puesta. El ADR F1-6 regla 6 no dice «ultima»; viene de la PoC y el plan lo dejo sin validar («Lo que el rol no cubrio»). Decidir y anotar en el ADR o en el `## Resultado`.

### MEN-3 — Un check obligatorio `neutral` o `skipped` sale como rojo
- Donde: crates/tc2/src/plataforma/github.rs:518
- Que pasa: todo `completed` cuya `conclusion` no sea `success` cuenta como `Fallo` y da `rechazada motivo=check-rojo`. GitHub trata `neutral` y `skipped` como correctos para los checks requeridos.
- Reproduccion: lectura del codigo; el mutante `conclusion == success` siempre true sobrevive, es decir, ninguna prueba cubre este mapeo. No verificado contra GitHub.
- Impacto: un workflow condicional que se salta deja un PR «rojo» en `tc2` aunque la plataforma deje mergear; el diagnostico («corrige y vuelve a empujar») manda a la persona en la direccion equivocada. Hoy los checks del modelo (`verificacion`, `carril`, `revision-ia`) no se saltan, de ahi MENOR.

### MEN-4 — `detectar` acepta una URL de API ajena con `origin` de `github.com`
- Donde: crates/tc2/src/plataforma/deteccion.rs:82
- Que pasa: el caso `(Some(h), Some((_, base))) => (h, base)` usa la URL de la clave aunque el host sea `github.com` o `gitlab.com`, siempre que sea `https`.
- Reproduccion: en el clon, test temporal `detectar("https://github.com/o/r", Some(("github", "https://api.evil.example")))` da `Ok` con `api=https://api.evil.example` (salida `REPRO api=https://api.evil.example`). El test `falla_cerrado` solo cubre el contradictorio por proveedor, no por host.
- Impacto: el ADR F1-13 decision 7 dice «sin ella, o si contradice el host, fallan cerrado»; con un `origin` de github.com el unico destino valido es `api.github.com`. Quien edite `.taskcode/config.yml` (que esta bajo CODEOWNERS segun F1-5, de ahi MENOR) enviaria el token del agente a otro host. Cerrarlo es una linea: en host conocido exigir la base canonica.

### MEN-5 — El E2E de puertas no limpia si una asercion falla
- Donde: crates/tc2/tests/plataforma_e2e.rs:379 (limpieza al final de `puertas_reales_contra_github`) y 149 en adelante
- Que pasa: el ruleset de `develop` (PR y 1 aprobacion), el PR y la rama se borran en las ultimas lineas; si falla cualquier `assert` anterior (p. ej. el 422 de auto-aprobacion que cambie GitHub), el test aborta con el ruleset `active` en `develop` y la rama/PR abiertos, y la siguiente ejecucion arranca sobre un repo con puertas que no esperaba.
- Reproduccion: lectura del codigo; no ejecutado (prohibido escribir en GitHub). Habria que forzar un fallo tras crear el ruleset.
- Impacto: restos en el repo de pruebas y un E2E que falla por el repo y no por el codigo. Con un guard `Drop` o limpiando antes de aseverar basta.

### MEN-6 — Un 403 de limite secundario sin cabeceras sale como «sin permiso»
- Donde: crates/tc2/src/plataforma/errores.rs:50 (`es_limite`) y la rama `403` de `traducir`
- Que pasa: `es_limite` exige `x-ratelimit-remaining: 0` o `retry-after`. GitHub a veces contesta el limite secundario con 403 y el mensaje «secondary rate limit» sin ninguna de las dos cabeceras; ese caso cae en `denegada motivo=permisos` con «necesita <permiso>».
- Reproduccion: lectura del codigo y de la tabla de errores del plan (que solo contempla las cabeceras). No verificado contra GitHub.
- Impacto: la persona recibe un arreglo equivocado (pedir mas permisos) y el codigo 8 en vez del 5 que reintenta. Cubrirlo con el texto del mensaje (como se hace con el aviso de plan) cuesta poco.

Sin hallazgos CRITICOS. Lo bueno que he comprobado: el envoltorio valida ruta, base y metodo antes de la red y las pruebas lo ejercitan; el token no sale en `Debug`, stdout ni stderr; `identidad` se pide una vez por rol; abrir-pr y merge-pr son idempotentes, el 422 de carrera relee, y el merge lleva el `sha` evaluado (probado por mutacion); la paginacion de reviews sigue `Link` solo en la misma base. Los errores salen dirigidos a la persona y con codigos de contrato.

## Lecturas fuera del contexto permitido

- crates/tc2/src/plataforma/github.rs, http.rs, errores.rs, puertas.rs, credenciales.rs, deteccion.rs, mod.rs: el diff de mi parte son solo fixtures; para revisar el adaptador y mutarlo hay que leer su codigo.
- crates/tc2/tests/plataforma.rs, tests/comun/http_mock.rs, tests/plataforma_e2e.rs: los tests contra los que se mutan los hallazgos.
- crates/tc2/Cargo.toml y lib.rs (diff): dependencias anadidas.
- docs/adr/F1-6-identidades-agente-revisor-persona.md y docs/adr/F1-13-adaptador-de-plataforma.md: contrasta decisiones 1, 3 y 5 de F1-6 y 1, 4, 7 y 9 de F1-13, como pidio la peticion del orquestador.
- tareas/03-en-revision/TASK-021/evidencia/e2e.log: evidencia del E2E, no ejecutado.
- Los fixtures de `tests/fixtures/plataforma/github/` no se abrieron: su efecto se comprobo por los mutantes de la bateria. No se leyo la fila F2-6 de `docs/plan/PLAN-TASKCODE-2.md` (el plan-final de la tarea la resume).
