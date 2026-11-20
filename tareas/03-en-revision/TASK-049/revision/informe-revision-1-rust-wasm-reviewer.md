# Informe de revision — TASK-049 (ronda 1)

- Commit revisado: 609cca1 (la peticion cita 17ad62c; 609cca1 solo anade la peticion)
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| # | Severidad | Estado | Fichero:linea | Hallazgo | Caso que lo demuestra |
|---|-----------|---|---------------|----------|-----------------------|
| 1 | MENOR | corregido | crates/tc2/src/lib.rs:208-215 (`orden_revisoras`, stdin con `take(TOPE_RUTAS)`) | Con `--rutas -` la entrada que pasa de 4 MiB se trunca en silencio y la orden sale `ok` con una seleccion calculada sobre una lista incompleta (el corte puede caer a mitad de ruta). Con `--rutas <fichero>` el mismo exceso es error (`orden.rs:204-211`, `leer_con_tope`). Incoherente y silencioso. | 12,5 MB de rutas (300001 lineas, la ultima `ultimo.vue`) por stdin: `revisoras ok`, sin aviso. Un diff de ~100k ficheros es de borde, de ahi MENOR. Arreglo: leer `TOPE_RUTAS + 1` y fallar si se pasa, igual que el fichero. |

## Alcance
- Ficheros revisados: crates/tc2/src/seleccion/{mod,catalogo,orden,tabla}.rs, crates/tc2/src/revision/{mod,operacion}.rs, lib.rs (`orden_revisoras`), tests/revisoras.rs, catalogo i18n y Cargo.toml.
- Contexto adicional pedido: ninguno.
- Limite JS-WASM: sin cambios de firma exportada (no hay `#[wasm_bindgen]` en el diff).

## Reproduccion
- Clon: `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-049-r1-a`, commit 609cca1, sin cambios (mutacion revertida con `git checkout`).
- Build: `cargo build -p tc2` en nativo, bien. No se compilo para wasm32 (el diff no toca el limite).
- Formato y lints: `cargo clippy -p tc2 --all-targets --locked -- -D warnings`, sin avisos.
- Suite (alcance acotado, la verificacion completa llega en verde de la peticion): `--lib seleccion::` 10/10; `--test revisoras` 3/3; `--lib revision::` 5/5; `--test revision` 3/3.
- Casos construidos: binario real `tc2 revisoras --rutas -` con `src/x.rs`, con bytes no UTF-8 (`a\xff.rs`: error limpio "no se puede leer ni escribir -", sin panic), y con 12,5 MB (hallazgo 1).
- Mutacion: `case_insensitive(true)` a `false` en `globs` (seleccion/mod.rs). Se pone rojo `seleccion::tests` (1 test, el de mayusculas `SRC/B.RS`). Red presente.
- Medidas: no aplican.

## Revisado sin hallazgos
- Tablas del ADR (`tabla.rs`): `include_str!` del ADR, vocabulario cerrado validado, error y no panic ante marca ausente, fila incompleta, atomo o accion fuera de vocabulario, y cobertura exacta carril x complejidad; `las_tablas_del_adr_se_leen` lo ejerce.
- Globs: `GlobBuilder` sin distinguir mayusculas y con `literal_separator`; patron invalido no casa (sin panic); `**/` casa en raiz.
- Catalogo: `de_base` usa `git ls-tree`/`leer_blobs` sobre `origin/develop` (con `--end-of-options`), no el worktree; el test de integracion `skills_de_la_base_e_informe_con_tope` cubre que una skill solo del worktree no entra. Corte de cabecera en limite de caracter.
- Tope del informe (64 KiB, `take(tope+1)`) y de rutas por fichero: correctos.
- Errores: sin `unwrap`/`expect`/`panic!` en codigo no de test de seleccion/ ni revision/; todo `Result` o aviso en `avisos`.
- `Revisar::con_seleccion`: idempotente ya en revision, solo reescribe si cambia, con test.
