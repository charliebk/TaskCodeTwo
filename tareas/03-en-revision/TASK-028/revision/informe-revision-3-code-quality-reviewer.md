# Informe de revision — TASK-028 (ronda 3)

- Commit revisado: baef020e65d468e9fae8fcb9d6b62530091616dd (la rama esta en 5c1d4a5, que solo anade `tareas/` sobre baef020; `git diff baef020 5c1d4a5 -- crates` vacio)
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/init/inventario.rs:369 | implementacion |
| IMP-2 | IMPORTANTE | abierto | crates/tc2/src/i18n/catalogo/init.rs:44 | implementacion |
| IMP-3 | IMPORTANTE | abierto | crates/tc2/src/init/inventario.rs:303 | implementacion |
| MEN-8 | MENOR | abierto | crates/tc2/src/init/inventario.rs:382 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Resumen:
- IMP-1 corregido. Verificado con el binario del clon: con origin con historia y sin rama local, la orden de `develop` parte de `origin/main` y no crea una raiz propia.
- N-1 de la ronda 2 (mismo caso que IMP-1 de Rust): resuelto. Con historia local sin relacion, ninguna orden propone `git push`, y sale el aviso en stderr en espanol y en ingles. Con clon de verdad, la orden de `develop` si lleva `git push origin develop`.
- La correccion introduce dos IMPORTANTES nuevos (IMP-2 y IMP-3): dos avisos que afirman algo falso. Sin ellos, la verificacion de la ronda queda en verde.
- Propuesta de veredicto: cambios-solicitados (IMP-2 e IMP-3 abiertos). Lo decide quien coordina.

## Reproduccion y verificacion por hallazgo

### IMP-1 (corregido): orden de la rama que falta

Verificado con `tc2 init --simular` (binario `target\debug\tc2.exe` del clon, `TC2_IDIOMA=es`). Repos de prueba en `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\r3-repro`.

- `loc_d` (`git init -b master`, sin commits, `origin` con `main`): `develop` sale `persona motivo=remoto-con-historia orden=git fetch origin main && git branch develop origin/main && git push origin develop`. No hay `git branch develop HEAD` ni raiz propia. Sin aviso (correcto: la historia local esta vacia, no hay nada que comparar).
- Lo que el diff cambia de la ronda 2 esta cubierto por `remoto_con_historia_no_es_repo_vacio` (`crates/tc2/tests/init.rs`), que pasa en la ejecucion de esta ronda.

Linea de la ronda 2: `inventario.rs:309` (`let vacio = ...`); en el codigo actual es `inventario.rs:369`.

### N-1 (resuelto; mismo caso que IMP-1 de Rust)

Escenario A, historia local sin relacion (`loc_a`: `master` con un commit propio; `origin` con `main` de otra historia). `tc2 init --simular`:

- `develop`: `persona motivo=historia-sin-relacion orden=git fetch origin main && git branch develop origin/main`. Ninguna orden contiene `git push`.
- stderr en espanol: `aviso: la historia de este clon no tiene nada en comun con la de origin; init no propone publicar ninguna rama: decide tu si este clon es de ese remoto`.
- stderr en ingles (`TC2_IDIOMA=en`): `warning: this clone's history has nothing in common with origin's; init does not suggest pushing any branch: decide yourself whether this clone belongs to that remote`.

Aviso sobre el texto del catalogo: ver IMP-2, que lo contradice en la ejecucion real.

Contraste, escenario B, clon de verdad (`loc_b`, `git clone` de un remoto con `main`, sin `develop`): `develop` sale `persona motivo=con-historia orden=git fetch origin main && git branch develop origin/main && git push origin develop`, sin aviso. El `git push` esta. Ambos idiomas.

Test de cobertura: `historia_local_sin_relacion_no_propone_publicar` (`crates/tc2/tests/init.rs:810`) asevera la orden sin `push`, el motivo y el aviso en espanol. Mutacion M1 (`inventario.rs:382`, `if remoto && !sin_relacion {` cambiado a `if remoto {`): el test falla en `init.rs:830` (`develop.orden`/motivo). Restaurado con `git checkout`; el clon queda limpio.

### IMP-2 (abierto): el aviso dice que init no publica ninguna rama, y si publica una

Fichero: `crates/tc2/src/i18n/catalogo/init.rs:44` (texto `AVISO_SIN_RELACION`, es y en: "init no propone publicar ninguna rama" / "init does not suggest pushing any branch").

Reproduccion (copia `loc_a2` del escenario A, remoto `rem_a2.git`, `tc2 init` real, no `--simular`):

- stderr: el aviso en espanol, sin ninguna linea de accion que publique.
- Remoto antes: solo `refs/heads/main` (`35ae2cf`). Remoto despues: `refs/heads/main` y `refs/heads/taskcodetwo/tareas` (`434ea1a`). `init` publico la rama de estado en `origin`, con una historia que no tiene nada en comun con la de `main`.

Es el punto N-2 de la ronda 2: la rama de estado (`taskcodetwo/tareas`, huerfana por diseno) se publica igual en el caso normal. Lo que es nuevo es que el aviso afirma lo contrario. El usuario lee "ninguna rama" y cree que no se ha subido nada. Es un mensaje que dice algo falso y manda a la persona en la direccion equivocada, asi que IMPORTANTE. Correccion posible: acotar el texto a ramas de codigo, o mencionar que la rama de estado se publica (lo que decida quien coordina sobre N-2).

### IMP-3 (abierto): historia relacionada que se da por sin relacion

Fichero: `crates/tc2/src/init/inventario.rs:303` (`historia_relacionada` solo mira `refs/remotes/origin/<rama>` y el sha de `ls-remote` si ese commit ya esta en el clon).

Reproduccion, escenario C2 (`loc_c2`): `rem_c2.git` tiene solo `main` (commit B). `loc_c2` tiene `master` en el commit A, que es ancestro de B (`git log` del remoto: `05f73e5 B`, `3933e41 A`). El commit A se trajo por sha (`git fetch origin <sha A>`), sin `refs/remotes/origin/main`. `tc2 init --simular`:

- `develop`: `persona motivo=historia-sin-relacion orden=git fetch origin main && git branch develop origin/main`. Sin `git push`.
- Aviso en stderr (es y en): "no tiene nada en comun con la de origin". Falso: A es ancestro de B.

Por el diff, antes de la correccion (ronda 2) la orden llevaba `&& git push origin develop` desde `master`. Es decir, la correccion quita una orden correcta en este caso (no reproducido con el binario de la ronda 2; se deduce de `ramas()` en el diff). El fallo es seguro (no se propone publicar nada), pero el mensaje es falso y la orden se pierde. No hay test que cubra este caso (el test de la seccion anterior solo cubre el caso sin relacion real). Caso real: un clon de una rama concreta cuyo remoto ya no tiene esa rama, o un `fetch` parcial. Si quien coordina lo considera de borde, puede bajarse a MENOR; por el criterio de la skill (mensaje que dice algo falso) lo mantengo IMPORTANTE.

### MEN-8 (abierto): linea `persona` sin orden

Fichero: `crates/tc2/src/init/inventario.rs:382`. Cuando la rama local ya existe, no esta en `origin` y la historia no tiene relacion, se marca `Decision::Persona` sin llamar a `persona()`, asi que la linea no lleva `orden`.

Reproduccion (`loc_a3`: escenario A mas una rama `develop` local): la linea sale `decision=persona motivo=historia-sin-relacion`, sin `orden`. A la vez, el aviso de personas (`AVISO_PERSONA`) dice "cada linea trae la orden". Efecto: no hay comando que seguir, y el aviso de sin relacion lo explica; es informativo. MENOR.

## Metodo y entorno

- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-028-r3-calidad`, rama `feature/task-028-f3-1-taskctl-init-setup`, HEAD `5c1d4a5` (codigo igual a `baef020`). Todos los comandos git y cargo con esa ruta absoluta; el `cd` se comprobo con `Get-Location`. El repo original no se ha tocado (solo se ha escrito este informe, sin commit).
- `cargo test -p tc2 --test init`: 18 passed, 0 failed (21,65 s de ejecucion).
- `cargo test -p tc2 --lib init::inventario`: 3 passed (`comillas_para_sh`, `comillas_para_powershell`, `misma_regla_con_o_sin_barra_final`). Estos tests no los ejecuta `--test init`.
- Mutacion M1 (ver N-1): falla el test de historia sin relacion. Restaurada.
- Limite: no se ha corrido la suite entera del workspace. La peticion pide `--test init` y no trae verificacion de la ronda en verde para `baef020`. Tampoco se ha probado la version en ingles del aviso mas alla de la salida con `TC2_IDIOMA=en`.
- Las pruebas de `tc2 init` (real y `--simular`) se hicieron en repos de prueba de `r3-repro`. Borrados al terminar, con el clon.

## Notas (no abren ronda, no van en la tabla)

- N-2 de la ronda 2 (publicacion de la rama de estado) se confirma en la ejecucion real: ver IMP-2. Queda pendiente la decision de quien coordina.
- El aviso en ingles no tiene test (solo se asevera el texto en espanol). Menor, sin hallazgo.
- Sin hallazgos CRITICOS.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/init/aplicar.rs:130-160`: para ver cuando se publica la rama de estado (IMP-2).
- `crates/tc2/src/huella.rs` (grep de `RAMA_ESTADO`) y `crates/tc2/src/git/remoto.rs` (`fn publicar`): para ver el nombre de la rama de estado y que `publicar` hace un `push` (IMP-2).
- `crates/tc2/src/i18n/mod.rs` (grep de `TC2_IDIOMA`): para elegir el idioma en las pruebas.
