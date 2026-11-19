# Informe de revision — TASK-037 (ronda 1)

- Commit revisado: b9a6a55f86af56fe615da1af20a1480ea2d9ae0a
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/estado/transaccion.rs:325 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/git/estado.rs:431 | implementacion |
| MEN-3 | MENOR | abierto | crates/tc2/src/git/puerto.rs:351 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Propuesta de veredicto: aprobada-con-correcciones (sin CRITICO ni IMPORTANTE; tres MENORES, dos de ellos
huecos de test que la mutacion deja al descubierto).

### MEN-1 — D4a (sin pendientes en el commit firmado) no tiene ningun test

- Donde: `crates/tc2/src/estado/transaccion.rs:325` (`if remoto && sello == Sello::Normal`).
- Que pasa: el codigo hace lo que pide D4a, pero ningun test lo ejerce. Quitada la guarda
  (`if remoto {`), siguen en verde `--test aprobar`, `--test estado_casos` y `--lib` (331 passed,
  6 passed, 7 passed). Si alguien la revierte, las pendientes entrarian en el commit firmado con
  la firma de la persona sobre cambios que no aprobo.
- Reproduccion: en el clon de b9a6a55, `sed -i 's/if remoto \&\& sello == Sello::Normal {/if remoto {/'`
  sobre transaccion.rs y `cargo test -p tc2 --test aprobar --test estado_casos --lib -- --test-threads=4`:
  todo `ok`. Restaurado con `git checkout`.
- Impacto: la proteccion que el plan marca como decision (D4a) no tiene red. Hoy el comportamiento es
  correcto.
- Sugerencia: un test de transaccion con una pendiente en cola y una `Operacion` de sello `Firmado`
  (no hace falta firma real: basta el `GIT` de pruebas o comprobar que `pendientes::listar` no se
  consulta) que afirme que el commit solo lleva la operacion firmada y la pendiente sigue en cola.

### MEN-2 — El test de `Plan-Blob` no distingue "arbol del propio commit" de "arbol de la base"

- Donde: `crates/tc2/src/git/estado.rs:431` (`format!("{arbol}:{ruta}")`) y
  `crates/tc2/tests/aprobar.rs` (`c3_la_persona_aprueba_con_su_firma_y_el_plan_blob`).
- Que pasa: `approve` no modifica `plan-final.md`, asi que el blob es el mismo en el arbol de la base
  y en el del commit nuevo. Cambiando el codigo para resolver contra la base
  (`format!("{}:{ruta}", base.unwrap_or(arbol))`) todos los tests siguen en verde (`--test aprobar`
  6 passed, `--lib` 331 passed). El requisito "blob del arbol del propio commit" (F1-6 D3) solo se
  distingue si el commit cambia `plan-final.md`, caso que ninguna operacion actual produce.
- Reproduccion: mutacion indicada, `cargo test -p tc2 --test aprobar --lib -- --test-threads=4`: verde.
- Impacto: ninguno hoy (el codigo es correcto); la propiedad no esta protegida si una operacion
  futura firmada toca el fichero del trailer.
- Sugerencia: test unitario de `construir_commit_sellado` con un `Cambio` sobre la ruta del trailer
  y una base con otro contenido, comprobando que el trailer es el blob nuevo.

### MEN-3 — El comentario de `ejecutar_firmado` afirma mas de lo que garantiza la lista

- Donde: `crates/tc2/src/git/puerto.rs:351` ("fuera de aqui `-S` y `--gpg-sign` estan prohibidos").
- Que pasa: la prohibicion se anade solo para `Subcomando::CommitTree` (`es_prohibido`). Para
  `Subcomando::Commit` solo se prohibe `--amend` y para `Tag` solo `-d/-f/--delete`, de modo que
  `commit -S` o `tag -s` no los rechaza `comprobar_argumentos` (lectura del codigo; no ejecutado).
  Ningun llamador actual pasa opciones de fuente externa a `commit` o `tag` (`guardar.rs` solo
  pasa `-q -m <mensaje>` y rutas tras `--`), por lo que no hay camino explotable.
- Impacto: documentacion algo engañosa sobre la lista cerrada; en `commit-tree`, que es lo que
  importa a esta tarea, la lista si es cerrada (tests de puerto en verde para `-S`, `-Sclave`,
  `--gpg-sign`, `--gpg-sign=clave`).
- Sugerencia: precisar el comentario ("en `commit-tree`") o prohibir tambien `-S`/`--gpg-sign` en `commit` y `tag`.

## Alcance

- Ficheros revisados: los 14 de dominio de la peticion (estado/mod.rs, estado/transaccion.rs,
  git/estado.rs, git/mod.rs, git/puerto.rs, i18n/catalogo/{cli,git,plan}.rs, lib.rs, plan/aprobar.rs,
  plan/mod.rs, plan/operacion.rs, plan/plantillas.rs, tests/aprobar.rs).
- Contexto adicional pedido: ver `## Lecturas fuera del contexto permitido`.
- Limite JS-WASM: sin cambios; el diff no toca `#[wasm_bindgen]` ni tipos que crucen el limite.

## Reproduccion

- Clon: `%LOCALAPPDATA%/pruebas-claude/TaskCodeTwo/task-037-r1-rust`, checkout b9a6a55f86af56fe615da1af20a1480ea2d9ae0a
  (borrado al terminar).
- Suite de la ronda: `cargo xtask verificar --perfil rapida` en verde (exit=0) sobre 7b98d98, log
  `%LOCALAPPDATA%/pruebas-claude/TaskCodeTwo/verif-037.log` (ultima linea "verificacion Rapida en verde");
  `git diff --stat 7b98d98 b9a6a55 -- crates docs` vacio, asi que es el mismo codigo. No repetida.
- En el clon, con `-- --test-threads=4`: `--test aprobar` 6 passed; `--lib plan` 29 passed;
  `--lib git::` 19 passed; `--test estado_casos` 7 passed; `cargo clippy -p tc2 --all-targets` sin avisos;
  `cargo fmt --check` exit 0.
- Mutacion: (1) quitar `&& sello == Sello::Normal` → todo verde (MEN-1); (2) resolver el blob contra la
  base en vez del arbol nuevo → todo verde (MEN-2). Las protecciones de la lista del puerto si tienen
  test rojo-si-se-quita (`la_firma_solo_por_ejecutar_firmado` y la tabla de prohibidos).
- Casos comprobados aparte: `git commit-tree` ignora `commit.gpgsign=true` (con `gpg.format=ssh` y una clave
  inexistente crea el commit sin firmar, exit 0), de modo que la suposicion del plan se cumple y las demas
  operaciones no firman por configuracion global.
- Build wasm32 y medidas de tamaño: no aplican (crate nativo, sin limite JS-WASM).

## Revisado sin hallazgos

- Lista cerrada del puerto: `ejecutar_firmado` valida con `comprobar_argumentos` antes de lanzar y antepone
  el unico `-S`; el texto del mensaje (`-m -S de texto`) no cuenta como opcion; no hay otra funcion que firme.
- Trailer `Plan-Blob`: se resuelve con `rev-parse --verify --end-of-options <arbol>:<ruta>` sobre el arbol
  del propio commit; el mensaje termina en el bloque `Operacion: ...` + `Plan-Blob: ...` y
  `%(trailers:key=Plan-Blob,valueonly)` lo lee (lo comprueba el test pesado, en verde).
- Con `Firmado` no se listan ni mezclan pendientes (codigo correcto; cobertura en MEN-1); si el arbol no
  cambia devuelve `None` antes de `commit-tree` (no se firma otra vez).
- Errores: sin `unwrap`/`expect`/`panic` en codigo de produccion del diff; los fallos de lanzar son
  `Informe` `error`, y la firma fallida es `rechazada motivo=firma` sin publicar (test `c3_una_clave_que_no_firma...`).
- stdin heredado: los tests pasan con `--test-threads=4` sin bloquearse (clave SSH sin frase); `wait_with_output`
  con stdout/stderr en pipe no se bloquea.
- `PedirCambios`/`ronda_mayor`: testigo `cambios-pedidos-<n>` solo de la ronda mayor; idempotente con el mismo contenido;
  `plan` abre la ronda siguiente (tests unitarios y de integracion en verde).

## Lecturas fuera del contexto permitido

- `crates/tc2/src/git/puerto.rs` (`comando`) y `crates/tc2/src/git/estado.rs` (cuerpo de `construir_commit_sellado`) completos,
  y `estado/pendientes.rs` (constante `ANOTACION`): son los ficheros del diff pero fuera de los hunks embebidos; para comprobar
  el formato del trailer y el entorno de lanzamiento.
- `tareas/03-en-revision/TASK-037/revision/peticion-revision-1-rust-wasm-reviewer.md`: la propia peticion (diff completo).
