# Informe de revision — TASK-022 (ronda 1)

- Commit revisado: 304782b8e59e4293ddbdd512d739b3979a0ac971
- Revisor: rust-wasm-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/plataforma/gitlab.rs:564 | implementacion |
| MEN-1 | MENOR | corregido | crates/tc2/src/plataforma/gitlab.rs:487 | implementacion |
| MEN-2 | MENOR | aceptado | crates/tc2/src/plataforma/gitlab.rs:447 | plan |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### IMP-1 — `merge-pr` de GitLab: ningun test cubre `exigir_persona`

`GitLab::merge_pr` llama a `self.exigir_persona(MERGE_PR)` antes de leer el MR
(gitlab.rs:564-566). Es la proteccion que impide mergear con un token de persona
que no esta en la lista. Reproduccion (mutacion): se borra ese bloque en el clon y
se corre `cargo test -p tc2 --test plataforma_gitlab -- --test-threads=1`: **3 passed,
0 failed**. Ningun test de GitLab ejerce un `merge-pr` con una persona fuera de la
lista (los de `merge-pr` usan siempre `persona(&mock, PERSONA, ..)`, que si esta en la
lista; el caso `4242` solo se prueba en `estado-puertas`). La proteccion compartida
(`Conexion::exigir_persona`) si esta cubierta por `estado-puertas` y
`leer-permisos-ci`, pero la llamada concreta de `merge_pr` no: quitarla dejaria que
un token de otra cuenta mergee sin que la bateria se entere. Arreglo: un caso en
`protecciones_de_gitlab` con `persona(&mock, 4242, false)` y
`merge_pr(12, SHA)` que espere `merge-pr rechazada motivo=identidad rol=persona` sin
ningun `PUT`.

### MEN-1 — G01 de GitLab: `merge_access_levels` por usuario o grupo, y mutante superviviente

`g01` solo mira `access_level` de cada entrada de `merge_access_levels`
(gitlab.rs:487-493; `NivelApi` no lee `user_id` ni `group_id`). Segun la
documentacion de la API de GitLab (de memoria, no reproducido contra un GitLab real:
la regla por usuario o grupo es de Premium), una entrada concedida a un usuario o
grupo concreto trae `access_level: 40` junto con `user_id`/`group_id`. Con el mock:

```
merge_access_levels = [{access_level:40, user_id:null}]            => estado-puertas ok
merge_access_levels = [{access_level:40, user_id:5}]               => estado-puertas ok   (acepta un usuario concreto)
merge_access_levels = [{40, user_id:null},{40, user_id:5}]         => rechazada motivo=config garantia=G01
```

El segundo caso pasa G01 aunque quien puede mergear sea un usuario concreto (que
puede no ser Maintainer). El ADR F1-13 define G01 como «sin borrado ni force-push»
y la parte «solo Maintainers» es un añadido del adaptador, por eso es MENOR. Ademas,
la mutacion que borra `p.merge_access_levels.len() == 1` **sobrevive** (suite verde):
ningun test pone dos entradas ni una lista vacia. Arreglo: o bien leer `user_id` y
`group_id` y exigir que sean nulos, o bien documentar que G01 solo cubre el nivel por
rol; y en cualquier caso un test con dos entradas de nivel 40.

### MEN-2 — Un 403 de permisos en `approval_rules` se informa como `motivo=plan`

Con una garantia Premium exigida (G03...), `estado_puertas` convierte **cualquier**
403 de `GET /approval_rules` en `rechazada motivo=plan` (gitlab.rs:447-459,
`puertas::rechazada_por_plan`). Un token sin el scope o sin el rol adecuado tambien
da 403 en ese endpoint, y entonces la persona recibe «el plan no da esto» y no
«tu token no tiene permiso». El plan ya lista este riesgo («un 403 de permisos que
se confunda con el de plan en `approval_rules`») y lo deja sin contener; el
codigo lo sigue y no hay test del caso (el unico 403 probado es `message: "403
Forbidden"` sin garantia Premium). Es un diagnostico equivocado, no un verde falso
(el resultado sigue siendo un rechazo), por eso MENOR. Arreglo posible: distinguir
por el mensaje de la API, o decir en el diagnostico «plan o permisos».

## Alcance
- Ficheros revisados: crates/tc2/src/plataforma/{conexion,deteccion,errores,github,gitlab,http,mod,puertas}.rs, crates/tc2/tests/comun/http_mock.rs, crates/tc2/tests/plataforma_gitlab.rs y plataforma_gitlab_e2e.rs (este ultimo solo leido, no ejecutado), mas la evidencia `evidencia/e2e-gitlab.log` y los fixtures de GitLab que usan los tests.
- Contexto adicional pedido: ver «Lecturas fuera del contexto permitido».
- Limite JS-WASM: sin cambios; el diff no toca ninguna firma `#[wasm_bindgen]` y el crate `tc2` es nativo.

## Reproduccion
- Clon: `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-022-r1-rust`, commit 304782b (el HEAD de la rama, 3d2f141, solo añade la peticion de revision), con su propio `CARGO_TARGET_DIR` dentro. Borrado al terminar.
- Build: `cargo test`/`clippy` compilan con `--locked` sin errores en nativo; no hay objetivo wasm32 en este crate.
- Formato y lints: `cargo fmt --check -p tc2` limpio; `cargo clippy -p tc2 --all-targets --locked -- -D warnings`: sin avisos.
- Suite: `cargo test -p tc2`. Primera pasada: `tests/plataforma.rs` (GitHub, el de F2-6) fallo con `sinred` (3 de 4 en la primera, 4 de 4 en la segunda con `--no-fail-fast`, 80-120 s: el atasco de red local descrito en la peticion; tambien lo daba en develop). Repetido en solitario con `cargo test -p tc2 --test plataforma -- --test-threads=1`: 4 passed en 4,4 s. El resto de binarios en verde en la pasada con `--no-fail-fast`: lib 41, alta 3, cli 2, estado_bordes 9, estado_casos 4, estado_concurrencia 3, git_* 9, plataforma_gitlab 3 (2 s), tarea 5; los E2E (`plataforma_e2e` y `plataforma_gitlab_e2e`) quedan `ignored` y no se ejecutaron. Conclusion: el unico fallo es el atasco de red local, no un defecto del diff.
- Casos construidos: tres variantes de `merge_access_levels` en el fixture `verde.json` (MEN-1, test temporal, borrado); lectura de la evidencia E2E de GitLab (`conflicto` da `falta=...sin-conflictos` y `merge-pr rechazada motivo=no-mergeable`, `autoaprobacion` da `ignoradas=1`).
- Mutacion (28 mutantes, tests `lib` + `plataforma_gitlab`, un hilo): MATADOS 22: `has_conflicts`, divergencia, aprobacion del autor, `skipped` como verde, `author.id` del estado, `verified*` por prefijo, sha del merge, `approvals_required >= 1`, force-push de G01, `any` de commits, decodificar `%2F`, 406, 429, orden por id, bot, `es_persona` de conexion, 409 de `abrir-pr`, clave `error` de la API, `clave` de la firma, 403 de `approval_rules` ampliado a 4xx, `sha_valido`, `.lock`, `ruta_de`. SOBREVIVEN: `merge_access_levels.len() == 1` (MEN-1) y quitar `exigir_persona` de `merge_pr` (IMP-1); y tres que no son hallazgo: `*s != ".."` en `codificar_proyecto` (redundante con `starts_with('.')`, mutante equivalente), `MAX_PAGINAS` (lo cubre `plataforma.rs:889`, codigo compartido en `conexion.rs`, no corrido en este lote) y el `p == GitHub` del 403 de plan (un 403 de GitLab con el texto de GitHub no existe).
- Medidas: no aplican (sin limite JS-WASM ni rendimiento tocado).

## Revisado sin hallazgos
- `http.rs`: `validar_ruta_con` admite solo `%2F` exacto y decodifica antes de mirar `.`/`..` (`g%2F..` rechazado, `%2f` y `%252F` rechazados); `camino[1..]` no entra en panico porque se comprueba `starts_with('/')` antes.
- `conexion.rs`: la cache de identidades con `RefCell` no re-entra (el prestamo se suelta antes de `leer()`); paginacion con tope y destino fijo a la misma base; sin `unwrap`, `expect` ni `panic!` en el codigo nuevo.
- `errores.rs`: el token no sale en ningun diagnostico (probado en el test con el token en claro); el texto de la API se sanea y recorta; el refactor de GitHub conserva los mensajes (los tests de F2-6 pasan en solitario).
- `gitlab.rs`: todo id o sha que entra en una ruta esta validado (`sha_valido`, `u64`, rama codificada); campo ausente da `respuesta-invalida`, no un verde por defecto; `abrir-pr` es idempotente con la carrera 409.
- No verificado contra un GitLab real (el E2E esta `ignored` y no se corrio): el codigo exacto con que GitLab responde a un merge no mergeable (el log muestra 405 y el plan prevé 405/406; no se sabe si hay un 422 «Branch cannot be merged»), y que `has_conflicts` pueda ser `false` mientras GitLab aun calcula el estado del MR. Ninguno es un hallazgo sin caso que lo demuestre.

## Lecturas fuera del contexto permitido

- Base de la peticion: la peticion llegaba truncada a 1245 de 2375 lineas; el resto del diff de `gitlab.rs` y los ficheros se leyeron del clon en el commit revisado (los mismos ficheros de la lista).
- `crates/tc2/tests/fixtures/plataforma/gitlab/verde.json` y `docs/adr/F1-13-adaptador-de-plataforma.md` (solo la fila de G01): para MEN-1, ver el fixture que usan los tests y la definicion de G01.
- `tareas/03-en-revision/TASK-022/evidencia/e2e-gitlab.log`: para contrastar lo observado en el E2E con el codigo.

**MEN-2 aceptado** (orquestador): con una sola cuenta Owner no se puede distinguir un 403 de plan de uno de permisos en approval_rules; es el riesgo ya listado en el plan («Lo que el rol no cubrio») y en los dos casos se falla cerrado (rechazada). Lo afinan F6-1 (Apps con permisos propios) y F3-8 (doctor).
