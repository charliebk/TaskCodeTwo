# Informe de revision — TASK-048 (ronda 1)

- Commit revisado: 999d68b (contenido de codigo identico a a30688c; 999d68b solo anade la peticion)
- Revisor: code-quality-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MENOR-1 | MENOR | aceptado (seguimiento: tests de rechazo de review_en; comportamiento verificado a mano por el revisor) | crates/tc2/src/revision/mod.rs:239-269 | implementacion |

Pase generico completo del dominio asignado (Rust: `revision/`, `entorno_git`, trait `Plataforma`, `Git::derivado`, despacho en `lib.rs`, ADR). Profundidad: a fondo en `revision/mod.rs`, `revision/operacion.rs` y tests; por encima en los adaptadores GitHub/GitLab (solo delegan en `entorno_push`).

### MENOR-1 — Los rechazos propios de `review_en` no tienen test de integracion
- Donde: crates/tc2/src/revision/mod.rs:239-269 (`sin-rama`, `carril-no-soportado`, `sin-worktree`) y 201-237 (ruta de rechazo del sondeo).
- Que pasa: `tests/revision.rs` solo cubre el camino feliz, la repeticion y la cabecera del push. Los rechazos que la adenda promete («los rechazos salen antes de empujar nada», `carril-no-soportado`, `sin-worktree arreglo="tc2 start TASK-NNN"`) no los ejercita ningun test: `grep` de `carril-no-soportado` y `sin-worktree` en `tests/` y en los unitarios de `revision/` no da nada (los unitarios de `operacion.rs` cubren estado y otra persona, no el orquestador). Quitar la comprobacion de carril o de worktree no pondria rojo nada.
- Reproduccion: ejecutados `cargo test -p tc2 --test revision` (3 ok), `--lib revision::` (4 ok), `--test toma` (13 ok). Test temporal (borrado, no commiteado) sobre el fixture de `revision.rs`: `review TASK-099` da `rechazada motivo=no-existe`; `TASK-002` sin empezar da `rechazada motivo=estado-no-valido` (planificada, espera en-curso); tras `git worktree remove --force` del worktree de TASK-001 da `rechazada motivo=sin-worktree rama=... arreglo=tc2%20start%20TASK-001` y el doble no recibe ningun `abrir-pr`; tras `tc2 start TASK-001` (recrea el worktree) `review` vuelve a dar `ok pr=12`. Es decir, el comportamiento es correcto; falta solo la red de regresion. El carril `hotfix/` no se pudo provocar sin editar la rama de estado y se juzga correcto por lectura (`base_de_rama` devuelve `origin/main`, `config::BASE` es `origin/develop`).
- Impacto: una regresion en el orden «rechazar antes de empujar» o en el carril empujaria una rama desde donde no toca sin que la suite lo note.
- Sugerencia: un test de integracion con dos casos (worktree borrado y rama `hotfix/...`) que compruebe `rechazada motivo=...` y que el doble no recibio `entorno-push` ni `abrir-pr`. Barato; puede ir a un MENOR de seguimiento si no compensa ahora.

## Comprobaciones sin hallazgo

- Criterio «PR enlazado a la tarea»: titulo `TASK-NNN: <titulo>` y cuerpo con rutas de tarea y plan, verificados por el unitario y la integracion con remoto bare; la rama del bare queda en la cabeza del worktree; la repeticion converge (`sin-cambios`) sin mover la rama de estado, tambien lanzada desde el worktree.
- Criterio `revision-ia`: la divergencia con el criterio (leido e informado, no publicado) esta documentada en la adenda con su motivo (G09, el agente solo tiene `checks:read`); `revision_ia` traduce `ok/pendiente/fallo/sin-leer/sin-revisor` y el unitario cubre cada rama y el modelo enviado. Nunca altera el resultado de `review`.
- Orden de efectos: push y PR antes de la transaccion de estado, de modo que un fallo de la transaccion devuelve `pr=` y repetir converge; no queda «en revision» sin PR.
- Mensajes: `remoto-no-https`, `sin-worktree`, `sin-origin`, `sin-rama` dicen que hacer (comando concreto).
- Divergencias de la adenda (`Git::derivado`, `estado-puertas` parcial, URL de `origin` vs `pushurl`, E2E acotado): coherentes con el codigo. La mutacion citada (empujar con `derivado(vec![])`) la cubre el test de la cabecera; no la repeti porque la peticion la da por hecha y el test de cabecera pasa en verde.
- Seguridad de la entrada: el token solo sale como cabecera limitada a la URL; `https://` obligatorio; `entorno_git` no registra el secreto. Sin hallazgos.
- E2E reales de GitHub y GitLab: no ejecutados por mi (no tengo credenciales); doy por buena la verificacion de la ronda indicada en la peticion.

## Lecturas fuera del contexto permitido

- crates/tc2/src/revision/mod.rs y operacion.rs, git/puerto.rs, plataforma/{entorno_git,mod,github}.rs, lib.rs, tests/revision.rs: son los ficheros del diff, necesarios para reproducir.
- crates/tc2/src/config/mod.rs y git/mod.rs (solo `BASE` y `base_de_rama`): para comprobar la comparacion del carril.
- i18n/catalogo/revision.rs y plataforma.rs: textos de los mensajes de error.
