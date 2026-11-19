# Informe de revision — TASK-118 (ronda 2)

- Commit revisado: 29d24a7abacf092fb86a6d23d91ddc0b2cc75977
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | corregido | crates/tc2/src/plataforma/gitlab.rs:1201 | implementacion |

Sin hallazgos nuevos CRITICO ni IMPORTANTE. La correccion cierra MEN-1 (verificado abajo).

### MEN-1 (ronda 1) — verificado: corregido

- Donde (ahora): `crates/tc2/src/plataforma/gitlab.rs:1201` (`preparar_puertas`, bloque `if let Some(pr) = p.solo_pr`): la puerta pasa a `Falta` en lugar de `Difiere` cuando la rama ya esta protegida como el modelo y faltan los ajustes de merge o la regla de aprobacion.
- Reproduccion en el clon, ronda 1 y ronda 2 del escenario original: `cargo test -p tc2 --test plataforma_gitlab aplicar_puertas_gitlab_corte_tras_proteger_converge` ok. Ronda 1: POST `protected_branches` 201 y PUT del proyecto 500 dejan `aplicar-puertas error motivo=http estado=500 aplicadas= a-medias=taskcodetwo-develop faltan=...`. Ronda 2 con `develop` ya protegida como el modelo (push 0, merge 40) y proyecto `ff`: `preparar_puertas` no devuelve `Difiere`, y `aplicar_puertas` escribe un unico PUT y un unico POST de `approval_rules`, sin volver a crear `develop`. Es el caso exacto de MEN-1.
- Mutacion: `Falta` -> `Difiere` en `gitlab.rs:1201` en el clon. El test anterior pasa a rojo en `plataforma_gitlab.rs:1043` (la aserion de que no hay `Difiere`). Se restauro con `git checkout`; `git status` limpio.
- Nuevo camino: el `Falta` solo sale cuando la puerta ya es `Existe`, y `Existe` exige que cada rama de la puerta case con el modelo (`solo_nivel` en push y merge, sin `allow_force_push`). Por tanto el camino no toca una proteccion distinta del modelo: si una rama difiere, la puerta queda en `Difiere` y no entra en `faltan`. En la escritura, las ramas ya existentes se saltan (`pa.ramas.contains`), asi que no hay POST sobre ellas.
- Sin DELETE ni PATCH: `crates/tc2/src/plataforma/http.rs` declara los metodos permitidos sin `DELETE` ni `PATCH`, y las escrituras de GitLab en el diff son POST y PUT del proyecto.
- Sin unwrap/panic sobre datos externos en el diff: una busqueda sobre las lineas anadidas no encuentra `unwrap`, `expect`, `panic!`. El indexado `faltan[k..]` ya existia y esta acotado por el bucle. `init` usa `faltan.first()` (Option).

## Alcance

- Ficheros revisados: `crates/tc2/src/plataforma/gitlab.rs`, `crates/tc2/src/plataforma/mod.rs` (`aplicar_en_lote`, clave `a-medias=`), `crates/tc2/src/plataforma/github.rs` (solo la firma de la clausura, `_escrito`), `crates/tc2/src/init/mod.rs` (motivos `a-medias` / `error` / `no-aplicada`), `crates/tc2/tests/plataforma_gitlab.rs` (test de convergencia, ejecutado).
- Contexto adicional pedido: ninguno. No hizo falta el llamador ni el `.d.ts`.
- Limite JS-WASM: sin cambios de firma exportada. El diff no contiene `#[wasm_bindgen]`; `tc2` es un crate nativo del flujo de `init`.

## Reproduccion

- Clon: `%LOCALAPPDATA%/pruebas-claude/TaskCodeTwo/task-118-r2-rust` (la ruta real resuelta por el shell es `...\AppData\Local\Packages\Claude_pzs8sxrjxfjjc\LocalCache\Local\pruebas-claude\taskcodetwo\task-118-r2-rust`), checkout de `29d24a7`. `git diff --stat ad859f7 29d24a7 -- crates docs` vacio: codigo identico al de `ad859f7`.
- Build: `cargo test` y `cargo clippy` compilaron con `--locked` en nativo (`dev`). No se compilo para `wasm32`: el crate no tiene objetivo WASM en este diff.
- Formato y lints: `cargo fmt --check` exit=0. `cargo clippy -p tc2 --all-targets --locked` exit=0, sin avisos.
- Suite: no repetida. Se cita la pasada de la ronda en verde: `cargo xtask verificar --perfil rapida` con `exit=0` y "xtask: verificacion Rapida en verde" en `...\pruebas-claude\TaskCodeTwo\verif-118.log`. El log no nombra el commit; se corresponde con `ad859f7` por dos pruebas: su hora de escritura (22:59 del 2026-11-18) es posterior al commit `ad859f7` (22:22 del mismo dia) y el diff de `crates` y `docs` entre `ad859f7` y `29d24a7` es vacio.
- Tests del diff: `cargo test -p tc2 --test plataforma_gitlab aplicar_puertas -- --test-threads=4`: 2 passed (`aplicar_puertas_gitlab_corte_tras_proteger_converge`, `aplicar_puertas_gitlab_premium_y_carrera`), exit=0. Log en `...\pruebas-claude\TaskCodeTwo\r2-test-gitlab.log`.
- Casos construidos: el escenario de MEN-1 (ronda 1 y ronda 2 del test anterior), y la mutacion `Falta` -> `Difiere` que lo reabre.
- Mutacion: `gitlab.rs:1201` `Falta` -> `Difiere`: `aplicar_puertas_gitlab_corte_tras_proteger_converge` rojo (exit=101, panic en `plataforma_gitlab.rs:1043`). Log en `...\pruebas-claude\TaskCodeTwo\r2-mutante-gitlab.log`.
- Medidas: no aplican.

## Notas fuera de la tabla (no son hallazgos)

- Regla de aprobacion en una puerta ya protegida: el nuevo camino (`Falta` por `premium && !con_aprobacion`) hace un POST de `approval_rules` donde antes salia `difiere`. Si el proyecto ya tiene una regla llamada `taskcodetwo-aprobacion` con 0 aprobaciones, GitLab podria rechazar el nombre duplicado con un codigo distinto de 409; en ese caso `aplicar-puertas` daria `error` y no convergiria. No se verifico contra GitLab real, y los tests con mock no lo cubren. Es un caso raro y el fallo sale visible (no silencioso), asi que no lo abro como hallazgo.
- Cambio de comportamiento esperado: con ramas ya protegidas como el modelo y ajustes de merge distintos en el proyecto, `init` ahora hace `PUT /projects/:id` donde antes no tocaba nada (`difiere`). Es lo que se pidio en la ronda 1 (aplicar ajustes y regla aunque la puerta salga `Difiere` solo por ellos) y la ADR F1-5 ya lo documenta. Lo dejo constancia para que quien revise la ADR lo sepa.

## Revisado sin hallazgos

- Ownership: `escrito: &mut bool` pasa como prestamo mutable puntual; no hay `clone` ni `Rc` nuevos.
- Errores: el camino nuevo devuelve `Err` por `?`, no `panic`; `a-medias` y `faltan` se construyen con `Option` y listas. No hay `let _ =` ni `.ok()` nuevos.
- Limite JS-WASM: sin cambios de firma exportada.
- Numerica, `unsafe`, rendimiento: no aplican al diff de la correccion.
- Herramientas: `cargo fmt --check` y `cargo clippy --all-targets` limpios.
- Tests: el test de convergencia cubre el escenario de MEN-1 y falla con la mutacion.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/plataforma/http.rs`: leido solo con busqueda, para confirmar que el cliente no declara `DELETE` ni `PATCH`.
- `tareas/03-en-revision/TASK-118/revision/informe-revision-1-rust-wasm-reviewer.md`: para el texto exacto de MEN-1 (lo pasa quien coordina en la peticion).
- `...\pruebas-claude\TaskCodeTwo\verif-118.log`: para citar la pasada de la suite de la ronda (la peticion la senala).
