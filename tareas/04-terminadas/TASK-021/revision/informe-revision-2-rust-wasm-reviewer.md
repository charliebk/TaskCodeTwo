# Informe de revision — TASK-021 (ronda 2)

- Commit revisado: 80b51da9ce8382316c6717f8bee44e45c9b1feae
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/tests/plataforma.rs:326 | implementacion |
| IMP-2 | IMPORTANTE | corregido | crates/tc2/src/plataforma/deteccion.rs:81 | especificacion |
| IMP-3 | IMPORTANTE | corregido | crates/tc2/src/plataforma/github.rs:514 | implementacion |
| IMP-4 | IMPORTANTE | corregido | crates/tc2/src/plataforma/mod.rs:64 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Verificacion de los cuatro IMPORTANTE de la ronda 1 (todos `corregido`, comprobados en el clon):

### IMP-1 — `el_envoltorio_rechaza_antes_de_la_red` intermitente: corregido

- Que hace el diff: `http.rs:265-273` lee la cabecera `content-length` y, si declara mas de `LIMITE_CUERPO`, devuelve `RespuestaInvalida` sin leer el cuerpo. Sin tamano declarado, el limite de lectura sigue actuando (`http.rs:274-280`).
- Verificacion: `el_envoltorio_rechaza_antes_de_la_red` pasa 10 de 10 ejecuciones seguidas (`cargo test -p tc2 --locked --test plataforma el_envoltorio`, 0,07-0,09 s cada una). La ronda 1 lo daba rojo en aproximadamente la mitad de las pasadas.
- Caso residual revisado: el mismo problema con respuesta `Transfer-Encoding: chunked` mayor de 1 MiB (sin `content-length`), con un servidor local que escribe 1,1 MB. Resultado: 10 de 10 `respuesta-invalida`, no `sinred`. No queda la carrera en ese caso. Prueba descartable, eliminada del clon.
- Veredicto de este punto: resuelto.

### IMP-2 — `detectar` aceptaba una URL de API arbitraria para `github.com`/`gitlab.com`: corregido

- Que hace el diff: `deteccion.rs:81-93`. Con host conocido, la base de la clave tiene que ser igual a `api_oficial(h)`; si no, rechazo `motivo=plataforma` (falla cerrado). Con host conocido y sin clave, usa la API oficial (`deteccion.rs:94`).
- Verificacion: el test de `falla_cerrado` incluye los casos `https://github.com/o/r` con `https://api.evil.example` y `git@gitlab.com:g/p.git` con `https://evil.example/api/v4`, y pasa.
- Mutante: se anula la comprobacion (`if base != oficial` sustituido por `if false && ...`). `deteccion::tests::falla_cerrado` pasa a rojo. Restaurado con `git checkout`.
- Veredicto de este punto: resuelto.

### IMP-3 — el filtro por `integration_id` (G09) no lo detectaba ningun test: corregido

- Que hace el diff: el test `protecciones_y_bordes` (`plataforma.rs:632-658`) anade un check `revision-ia` verde publicado por la App `55555` , que no es la App del modelo (`99001`). Espera `falta=revision-ia`. Y un check `revision-ia` en rojo de la App `99001` que da `motivo=check-rojo`.
- Mutante M01 (quitar `&& r.app.as_ref().is_some_and(|a| a.id == *origen)` en `github.rs:514`): `protecciones_y_bordes` pasa a rojo (`left: ... ok` frente a `right: ... esperando ... falta=revision-ia`). Restaurado.
- Veredicto de este punto: resuelto.

### IMP-4 — `Usuario::es_persona` sin test de bot ni de `web-flow`: corregido

- Que hace el diff: `protecciones_y_bordes` (`plataforma.rs:660-678`) usa `Personas` con el id de `web-flow` (19864447) y el de un bot (2001), y reviews `APPROVED` de ambos, mas la de la persona. Espera `aprobaciones=1 ignoradas=2`. Tambien cubre `firma_commit` con committer `web-flow`, bot y `null` (`plataforma.rs:767-787`).
- Mutante 4a (`mod.rs:65`, quitar `self.tipo == "User" &&`): `protecciones_y_bordes` rojo (`aprobaciones=2 ignoradas=1` frente a `aprobaciones=1 ignoradas=2`).
- Mutante 4b (quitar `self.login != "web-flow" &&`): mismo test rojo con el mismo resultado.
- Restaurados con `git checkout` tras cada mutante.
- Veredicto de este punto: resuelto.

## Hallazgos nuevos

Sin hallazgos nuevos. La correccion no abre ningun CRITICO. Estos cambios se revisaron y no se reportan:

- `github.rs:514-524`: un check obligatorio que acaba `neutral` o `skipped` cuenta como `Ok`. Es la regla de GitHub para checks obligatorios, y la correccion lo documenta; no es un fallo.
- `github.rs:541-550`: solo cambian el estado las reviews `APPROVED`, `CHANGES_REQUESTED` y `DISMISSED`. Comentarios y reviews pendientes no retiran una aprobacion. Coincide con lo pedido en MEN-1 de la ronda 1.
- `github.rs:566-568`: `sin_conflictos` solo falla con `dirty`; `unknown` pasa y lo frena `merge-pr` (405). Coincide con MEN-5.
- `github.rs:616-628`: `merged: true` sin `sha` valido da `ok` sin `commit=`. Coincide con MEN-4.
- `http.rs:71-106`: validacion de la base con puerto y consulta. `user:pass@host` y `host:443@evil` siguen rechazados por el caracter `@` y por `puerto_valido`.
- `errores.rs:43-50`: 429 y el mensaje `rate limit` cuentan como limite. Solo cambia la clasificacion de respuestas 403 que mencionan `rate limit`.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/plataforma/mod.rs` (lineas 60-102): para ver `Usuario::es_persona` y `Firma::cuenta_como_aprobacion`, que mutaban IMP-4.
- `crates/tc2/tests/comun/http_mock.rs` (lineas 92-150): para comprobar que el mock envia `Content-Length` y `Connection: close`, relevante para IMP-1.
- `crates/tc2/src/lib.rs`: solo para confirmar que `plataforma` es publico y el test chunked podia usarlo.
- `tareas/03-en-revision/TASK-021/revision/informe-revision-1-rust-wasm-reviewer.md`: para el texto de los hallazgos de la ronda 1.
- Skill `task-workflow/revision.md` (en el plugin `taskcode-plugin` 0.16.1): para el formato del informe y la regla de la linea de veredicto.

## Alcance
- Ficheros revisados: `crates/tc2/src/plataforma/deteccion.rs`, `errores.rs`, `github.rs`, `http.rs`, `puertas.rs` y `crates/tc2/tests/plataforma.rs`. Son los del diff de la ronda que casan con los patrones de Rust.
- Contexto adicional pedido: ninguno. Se leyeron los ficheros fuera de la lista permitida indicados arriba.
- Limite JS-WASM: sin cambios de firma exportada. El crate no usa `#[wasm_bindgen]` en este diff ni se compila para `wasm32`, asi que no hay llamador TypeScript ni `.d.ts` que pedir.

## Reproduccion
- Clon: `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-021-r2-rust\repo`, rama `feature/task-021-f2-6-adaptador-github`, HEAD `94c2a8b` (solo anade la peticion sobre `80b51da`; `git diff 80b51da..HEAD` solo toca `tareas/`). `CARGO_TARGET_DIR` en la misma carpeta. La carpeta se borra al terminar.
- Build: `cargo test -p tc2 --locked` compila en nativo. No aplica `wasm32`, porque no hay codigo WASM en el diff.
- Formato y lints: `cargo fmt --all -- --check` sin diferencias; `cargo clippy -p tc2 --all-targets --locked -- -D warnings` limpio.
- Suite: `cargo test -p tc2 --locked`, una pasada completa, salida final `EXIT=0`. Lib 39 ok; `plataforma` 4 ok (`el_envoltorio_rechaza_antes_de_la_red`, `bateria_de_la_poc_m_e_idempotencia`, `protecciones_y_bordes`, `errores_dirigidos_a_la_persona`); `tarea` 5 ok y 1 ignorado; `alta`, `cli`, `estado_*`, `git_*` ok; E2E de plataforma (`--ignored`) no ejecutado, como pide la peticion. La peticion no traia verificacion en verde, asi que esta pasada es la de la ronda. La linea base de la ronda 1 la daba roja en `el_envoltorio_rechaza_antes_de_la_red` (IMP-1).
- Repeticion de IMP-1: `cargo test -p tc2 --locked --test plataforma el_envoltorio`, 10 ejecuciones: 10 ok.
- Casos construidos: respuesta chunked de 1,1 MB contra el envoltorio, 10 repeticiones: `respuesta-invalida` en las 10 (descartado).
- Mutacion (sobre `--lib deteccion` y `--test plataforma protecciones_y_bordes`; el mutante se revierte con `git checkout` despues de cada uno):
  - IMP-2: `if base != oficial` anulado en `deteccion.rs` -> `falla_cerrado` rojo.
  - IMP-3: filtro de `app.id` quitado en `github.rs:514` -> `protecciones_y_bordes` rojo.
  - IMP-4a: `tipo == "User"` quitado en `mod.rs:65` -> `protecciones_y_bordes` rojo.
  - IMP-4b: `login != "web-flow"` quitado en `mod.rs:65` -> `protecciones_y_bordes` rojo.
- Medidas: no aplican (sin limite JS-WASM ni cambios de rendimiento).
- Estado del clon al terminar: `git status` limpio tras restaurar los mutantes; el fichero de prueba descartable se borro antes de cerrar.

## Revisado sin hallazgos
- Limite de cuerpo por cabecera `content-length` y lectura acotada cuando no hay tamano declarado (`http.rs:265-280`).
- Validacion de la base: esquema `https`, puerto `1..65535`, sin consulta ni fragmento, sin usuario (`http.rs:71-106`).
- Comprobacion de la base frente al host conocido en `detectar` (`deteccion.rs:81-93`).
- Filtro de origen de checks, aprobaciones de persona, `web-flow` y bots (`github.rs:506-570`, `mod.rs:64-66`, `mod.rs:95-101`).
- Clasificacion de limite de peticiones (`errores.rs:43-50`) y merge sin `sha` (`github.rs:616-628`).
- Ausencia de `unwrap`, `expect` o `panic!` en codigo de produccion del diff.
