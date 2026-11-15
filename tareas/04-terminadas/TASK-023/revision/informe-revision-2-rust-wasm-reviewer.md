# Informe de revision — TASK-023 (ronda 2)

- Commit revisado: 964898f7d7e9298ce912ae2a54c2b8b39e88f998
- Revisor: agente general-purpose (haiku) con la skill rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|

Sin hallazgos nuevos. Verificacion de los pendientes de la ronda 1:

- IMP-1 (`comprueba` con `escribe` no tiene test): **corregido**. El test nuevo
  de `configs_hostiles_rechazadas_con_la_ruta` (`derivados[0].escribe`,
  fragmento «prohibida») muere con el mutante que sustituye la guarda
  `Some("comprueba") if escribe` por `if false && escribe` (ver Reproduccion).
  Antes de la correccion ese mutante sobrevivia.

Notas (no abren ronda, no son hallazgos):

- `confianza_incluye`: `*.*` y `.*` se rechazan con «nunca todo el arbol». Es lo
  que dice la regla escrita en el esquema («ningun segmento solo de * y .»), pero
  `*.*` y `.*` solo cubren la raiz, no todo el arbol. Rechazo mas estricto de lo
  necesario; falla cerrado, sin dano real.
- `ruta_tareas` rechaza cualquier `~` en un segmento, tambien en nombres
  legitimos (`notas~v1`). Falla cerrado; coherente con la regla escrita.
- `formato: -1` (y otros valores que no son entero sin signo) sigue cayendo en
  `Motivo::Formato` y el arreglo dice «migralo con tc2 actualizar». Comportamiento
  previo a esta ronda, no introducido por la correccion.

## Alcance

- Ficheros revisados: crates/tc2/src/config/validar.rs (cambios de `comprobar`
  Orden/Glob/Cron/RutaTareas, `todo_el_arbol`, `cron_valido`, repetidos en listas),
  crates/tc2/src/config/mod.rs (`FormatoNuevo`, arreglo de 0.16 por ruta),
  crates/tc2/src/config/claves.rs (textos del esquema y `CLAVES_016`),
  crates/tc2/tests/config.rs, crates/tc2/esquemas/config.schema.json.
- Contexto adicional pedido: ninguno. El diff no cambia firmas exportadas con
  `#[wasm_bindgen]` ni toca WebAssembly.
- Limite JS-WASM: no aplica (CLI nativo `tc2`, sin `wasm_bindgen`).

## Reproduccion

- Clon: `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-023-r2-rust`, rama
  `feature/task-023-f2-7-configuracion-minima-con-esquema-va`. HEAD del clon
  `efb4720`; el codigo de `crates/` es identico a `964898f` (la diferencia son
  solo ficheros de `tareas/`). El clon se borra al terminar.
- Build: compilado dentro de `cargo test --locked`. Sin objetivo `wasm32` (no aplica).
- Formato y lints: no ejecutados (fuera de la suite pedida en la peticion).
- Suite: `cargo test -p tc2 --test config --lib --locked` sobre `964898f`: verde.
  lib 46 pasan, config 8 pasan, otro target 5 pasan (1 ignorado), 0 fallos.
- Casos construidos (sonda temporal con `config::leer`, borrada despues):
  - Cron validos aceptados: `0 0 * * 0`, `59 23 31 12 7`, `*/5 * * * *`,
    `0,30 */6 * * 1,3,5`, `0 12 1-31 * 1-5/2`, `00 00 * * *`, `5-5 * * * *`.
  - Cron rechazados (correcto): `0 0 * * MON` (ya se rechazaba antes),
    `*/5 * * * 1-5/0`.
  - Rutas de `escribe` aceptadas: `tareas/./x.md`, `tareas/.oculta/x.md`,
    `tareas/a.b/x.md`, `tareas/hitos/`, `tareas/a/../hitos.md` (normalizada).
    Rechazada: `tareas/a /x.md` (espacio final de segmento).
  - `formato: 3` → `FormatoNuevo` con arreglo «actualiza tc2» (correcto).
  - Clave 0.16 (`rama_base`) → arreglo «tc2 actualizar»; `modo_flujo: 'bogus'`
    → arreglo generico (correcto). `CLAVES_016` solo contiene claves que no
    existen en la 2.0, asi que el arreglo por ruta no se dispara en errores de
    valor de claves de la 2.0.
  - `fases: ['new', 'new']` → «repetido» (correcto).
- Mutacion:
  - IMP-1: `validar.rs`, guarda `Some("comprueba") if escribe` → `if false && escribe`.
    Resultado: `test configs_hostiles_rechazadas_con_la_ruta ... FAILED`
    (panic en `tests/config.rs:226`). Mutante MUERTO. Clon restaurado despues con
    `git checkout`.
- Medidas: no aplican.

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-023/revision/informe-revision-1-rust-wasm-reviewer.md`:
  para conocer la descripcion exacta de IMP-1 y los MENORES de la ronda anterior.
- `crates/tc2/Cargo.toml` y `Cargo.toml` raiz (solo cabecera `rust-version`/`edition`):
  para comprobar que el crate compila con la edicion y el MSRV del workspace.
