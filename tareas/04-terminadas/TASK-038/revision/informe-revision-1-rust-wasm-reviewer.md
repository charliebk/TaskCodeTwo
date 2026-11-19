# Informe de revision — TASK-038 (ronda 1)

- Commit revisado: 5ae20d8b11347556bddbfe0fe074e9827f4825f5
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/plan/aprobar.rs:364 | implementacion |

### MEN-1 — un `tarea.md` ilegible del hito se ignora en silencio y el "todo o nada" no lo cuenta

- Donde: `del_hito` (aprobar.rs:364) devuelve `None` si el `tarea.md` no se entiende, y `AprobarHito::aplicar` (aprobar.rs:413-415) hace `continue` sin anotar nada.
- Que pasa: una tarea cuyo `tarea.md` no se puede leer (y de la que por tanto no se sabe el sprint) no entra en `abiertas` ni en `candidatas` ni en `fallos`. Si pertenecia al hito, `approve --hito n` aprueba el resto y ok, sin avisar de que dejo una fuera. Con un solo ilegible y nada mas en el hito, el resultado seria `hito-vacio`, que sugiere un numero equivocado. `approve TASK-NNN` suelto si dice `ilegible` (via `tarea_de`).
- Reproduccion: test temporal en el clon (descartado): `EnMemoria` con `TASK-001` (sprint 4, en-diseno, con plan) y `TASK-002/tarea.md` = "basura sin front matter"; `hito(4).aplicar(..)` devuelve `Ok` con 1 cambio (`Ok(1)`), sin fallo ni rastro de TASK-002.
- Impacto: bajo. El estado lo escribe `tc2` y la lectura de un `tarea.md` roto ya salta en `status`/`board`/`doctor`; se aprueba menos de lo que la persona cree, nunca mas, y la persona vuelve a lanzar. Es una laguna del criterio "todo o nada", no una perdida de datos.
- Sugerencia: contar el ilegible como fallo `ilegible` solo cuando no se pueda descartar que sea del hito (no se puede saber), o al menos documentarlo en el ADR/adenda como limitacion conocida. Aceptable dejarlo como esta si se documenta.

## Alcance

- Ficheros revisados: crates/tc2/src/estado/transaccion.rs, crates/tc2/src/plan/aprobar.rs, crates/tc2/src/plan/mod.rs, crates/tc2/src/i18n/catalogo/{cli,plan}.rs, crates/tc2/tests/aprobar.rs (los tres tests `f4_3_*`).
- Contexto adicional pedido: ver "Lecturas fuera del contexto permitido".
- Limite JS-WASM: sin cambios (no hay `#[wasm_bindgen]` en el diff).

## Reproduccion

- Clon: `%LOCALAPPDATA%/pruebas-claude/TaskCodeTwo/task-038-r1-rust`, checkout de 5ae20d8 (borrado al terminar).
- Build: lo cubre la suite verificada; el clon compila `cargo test`/`clippy` sin errores. wasm32 no aplica (crate nativo).
- Formato y lints: `cargo fmt --check` exit=0; `cargo clippy -p tc2 --all-targets` sin avisos.
- Suite: `cargo xtask verificar --perfil rapida` en verde (exit=0) sobre 5112652 (`git diff --stat 5112652 5ae20d8 -- crates docs` vacio), log `verif-038.log` ("verificacion Rapida en verde"); no se repitio. En el clon con `--test-threads=4`: `--test aprobar` 9 passed (los dos E2E del hito tardan unos 200 s con firma SSH real), `--lib plan` 36 passed, `--lib estado` 28 passed.
- Casos construidos: el test temporal de MEN-1 (descartado). Revision de la transaccion: `sello()` se pide antes y despues de `aplicar`. Antes solo se usa como `!= Sello::Normal`: para `AprobarHito` es siempre `Firmado` (con o sin lote de un intento previo), asi que la decision de las pendientes no cambia. Para `Aprobar` el sello es constante (un `Plan-Blob` fijo por `id`) y `PedirCambios` y el resto son `Normal`: ambas llamadas devuelven lo mismo y el cambio no altera su comportamiento. Los sellos solo se usan en esas dos lineas (`grep .sello()`), y `construir_commit_sellado` ya devolvia `None` antes de firmar si el arbol no cambia (hito todo aprobado => `sin-cambios`, cubierto por el E2E).
- Mutacion (en el clon, con `cargo test -p tc2 --lib plan::aprobar`): (1) quitar el tope (`> MAX_HITO` por `> 100000`): rojo (`hito_todo_o_nada`); (2) abierta siempre `true` (cuenta terminadas/descartadas): rojo (2 tests); (3) quitar el filtro de sprint (`sprint != self.hito`): rojo (2 tests). Todo o nada y la identidad (agente => `None`, `rechazada motivo=identidad`) los ejercen `hito_todo_o_nada`, `solo_una_persona_aprueba_un_hito` y los E2E. No se mutó el segundo `op.sello()` de transaccion.rs: lo cubre el E2E de diez `Plan-Blob` (se aprecia por la lectura; no ejecutado por coste de 200 s).
- Medidas: no aplican.

## Revisado sin hallazgos

- `RefCell` del lote: `aplicar` hace `clear()` al inicio de cada intento y un unico `borrow_mut` de escritura al final; `sello()`/`aprobadas()`/`fallos()` solo toman `borrow()` inmutable, sin solape posible (un hilo, sin retencion de guardas). Entre reintentos el lote se recalcula, no se arrastra uno viejo; si un intento acaba en `Rechazada`/`Error` el lote queda vacio.
- Datos externos: sin `unwrap`/`panic`/indexado en el codigo nuevo; `get(..10).unwrap_or_default()`, `split_once(..).map_or`, `parse().ok()` en `numero_de_hito` (rechaza `-1`, vacio y no digitos; un numero enorme desborda `u32` y da `hito-invalido`). `zip(planes)` es seguro porque `leer_varios` devuelve un elemento por ruta.
- Filtro y tope: cuenta solo las candidatas por aprobar (D5a), 20 justo pasa (`hito_justo_en_el_tope`), 21 da `demasiadas`.
- Orden de comprobaciones de `aprobable` igual que antes de la extraccion (otra persona, ya aprobada, estado, pausa, sin plan, esqueleto); los tests de `Aprobar` existentes siguen en verde.

## Lecturas fuera del contexto permitido

- crates/tc2/src/git/estado.rs (`construir_commit_sellado`) y crates/tc2/src/estado/{lectura,tareas}.rs (`leer_varios`, `tareas`): para comprobar que un arbol sin cambios no se firma, el emparejado de `leer_varios` con `zip` y que `tareas` no propaga los `tarea.md` ilegibles.
