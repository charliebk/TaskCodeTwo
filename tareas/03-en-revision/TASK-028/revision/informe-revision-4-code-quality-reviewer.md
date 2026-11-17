# Informe de revision — TASK-028 (ronda 4)

- Commit revisado: c9b549c33536f3809efaa1417675c5750db90d33
- Revisor: code-quality-reviewer
- Veredicto: aprobada

Nota: la rama `feature/task-028-f3-1-taskctl-init-setup` esta en 8b09d62, que solo anade ficheros de `tareas/`. `git diff c9b549c 8b09d62 -- crates` esta vacio, asi que el codigo revisado es el de c9b549c.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/init/inventario.rs:369 | implementacion |
| IMP-2 | IMPORTANTE | corregido | crates/tc2/src/i18n/catalogo/init.rs:44 | implementacion |
| IMP-3 | IMPORTANTE | corregido | crates/tc2/src/init/inventario.rs:314 | implementacion |
| MEN-8 | MENOR | corregido | crates/tc2/src/init/inventario.rs:431 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Resumen: los cuatro hallazgos quedan resueltos por c9b549c. No hay hallazgos CRITICO ni IMPORTANTE nuevos. Propuesta de veredicto: `aprobada` (lo decide quien coordina).

## Reproduccion y verificacion por hallazgo

### IMP-2 (corregido): el aviso ya no dice que no se publica ninguna rama

El texto de `AVISO_SIN_RELACION` (es y en) dice ahora "no propone publicar ninguna rama de codigo (la rama de estado {rama}, huerfana, si se publica)". `mod.rs` sustituye `{rama}` por `crate::huella::RAMA_ESTADO`. El test `historia_local_sin_relacion_no_propone_publicar` (`crates/tc2/tests/init.rs:843`) comprueba el texto en espanol y en ingles, y que la rama de estado `taskcodetwo/tareas` se publico en el remoto.

### IMP-3 (corregido): ancestro presente con la ref remota ausente

Reproduccion a mano con el binario del clon (`target\debug\tc2.exe`, `TC2_IDIOMA=es`, `tc2 init --simular`). Repos en `pruebas-claude\taskcodetwo\task-028-r4-repro`:

- Caso C (la ronda 3): `loc_c` tiene `master` en A (ancestro de B), sin `refs/remotes/origin/main` ni el commit B. `origin` tiene `main` en B. Salida: `develop` con `motivo=no-comprobado`, `orden=git fetch origin main && git branch develop origin/main`, sin `git push`. El aviso dice "no se pudo comprobar ... (faltan sus commits en el clon) ... ejecuta git fetch origin y repite tc2 init". Es cierto, y ya no afirma que no haya relacion.
- Caso C tras `git fetch origin`: `develop` pasa a `motivo=con-historia` con `&& git push origin develop`. Lo que dice el aviso lleva al resultado correcto.
- Caso D (ancestro traido por sha, es la punta remota): `loc_d` tiene `master` en A y `origin/main` esta en A, sin ref remota. Salida: `develop` con `motivo=con-historia` y `git push origin develop`. El ancestro traido por sha cuenta como relacionado.
- Test: `ancestro_traido_por_sha_cuenta_como_relacionada` (`crates/tc2/tests/init.rs:940`) pasa. Cubre el caso con la punta presente en el clon.

### MEN-8 (corregido): toda linea `persona` lleva orden

Revisadas todas las lineas con `Decision::Persona` de `crates/tc2/src/init/inventario.rs` (versionar, HEAD sin nacer, puertas, y la rama local de `ramas()`): todas pasan por `persona()` o llevan `.orden(...)`. La rama local sin relacion ahora sale con `persona(a, m, comprobar)` (linea 431-439). El test `rama_local_sin_relacion_lleva_su_orden` (`init.rs:982`) asevera `all(|a| a.orden.is_some())` sobre las lineas `persona`.

### IMP-1 (corregido, sin cambio en c9b549c): orden de la rama que falta

Cubierto por la suite: `remoto_con_historia_no_es_repo_vacio` y las demas pruebas de `init.rs` pasan.

## Ejecucion y mutacion

- Puerta: build y tests. `cargo test --workspace` en el clon (commit 8b09d62, codigo de crates igual a c9b549c): exit 0, 0 fallos en todas las pasadas. `tests/init.rs`: 20 passed, 0 failed. Linter no ejecutado (la peticion no lo pide y el proyecto no lo trae en la puerta de esta ronda).
- Mutacion: en `crates/tc2/src/init/inventario.rs`, `Ok(if falta_alguna {` cambiado a `Ok(if false && falta_alguna {` (NoComprobable nunca sale, se convierte en SinRelacion). `cargo test -p tc2 --test init`: 19 passed, 1 failed. Falla `historia_local_sin_relacion_no_propone_publicar` en `crates/tc2/tests/init.rs:852` (espera `no-comprobado`). El fichero se restauro con `git checkout` y el clon quedo limpio (0 cambios).

## Notas (no abren ronda, no van en la tabla)

- Caso ancestro presente con la punta remota ausente (caso C): en la ronda 2 la orden llevaba `git push`, y ahora sale sin push hasta que hay `git fetch origin`. Es el lado prudente y el aviso lo indica. No cambia lo que `init` escribe ni publica de forma incorrecta, asi que no se cuenta como hallazgo.
- El texto `"la rama de estado {rama}, huerfana, si se publica"` usa un condicional. No afirma nada falso: la rama de estado se publica en el caso probado (test `init.rs:900-906`).
- El aviso en ingles solo se asevera en el test de la historia sin relacion; el aviso de no comprobado no tiene aserto en ingles. Menor, sin hallazgo.
- Sin hallazgos CRITICOS.

## Lecturas fuera del contexto permitido

ninguna
