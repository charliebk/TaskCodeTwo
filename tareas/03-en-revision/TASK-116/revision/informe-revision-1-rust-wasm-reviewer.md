# Informe de revision — TASK-116 (ronda 1)

- Commit revisado: 0097ddc (HEAD de la rama; el codigo no cambia respecto a 9290052, solo la peticion)
- Revisor: rust-wasm-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | docs/adr/F1-1-carpeta-fija-y-rama-de-estado.md:1 | implementacion |
| MEN-2 | MENOR | abierto | docs/adr/F1-16-lotes-desatendidos.md:102 | implementacion |
| MEN-3 | MENOR | abierto | docs/adr/F1-11-extensiones-declaradas.md:21 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Ninguno afecta al codigo Rust: no hay hallazgos CRITICO ni IMPORTANTE.

### MEN-1 — F1-1 define la rama `taskcode/tareas` y no lleva la linea de adenda
`docs/adr/F1-1-carpeta-fija-y-rama-de-estado.md` tiene 16 apariciones del nombre viejo (titulo en la
linea 1, tabla de alternativas en la 28, comandos `git worktree add --orphan -b taskcode/tareas` en la
42-43). La adenda de F1-12 dice que cubre F1-2, F1-3, F1-4 y F1-16, y solo F1-2, F1-3 y F1-4 recibieron
la linea de puntero. F1-1 es el ADR que fija el nombre de la rama, y el indice `docs/adr/README.md:11`
tambien lo repite. Quien lea F1-1 sin pasar por F1-12 se lleva el nombre equivocado.
Reproduccion: `grep -rnE "taskcode/tareas" docs/adr/F1-1-*.md docs/adr/README.md` en el clon.

### MEN-2 — F1-16 esta nombrado en la adenda pero no tiene la linea de puntero
La adenda de F1-12 lo cita como lectura valida «de F1-2, F1-3, F1-4 y F1-16», pero
`docs/adr/F1-16-lotes-desatendidos.md` no la lleva y conserva `.taskcode/worktrees/TASK-NNN` (linea 102)
y otras dos apariciones. Es una promesa de la adenda que el cambio no cumple.
Reproduccion: `grep -n "TASK-116" docs/adr/F1-16-*.md` devuelve vacio.

### MEN-3 — Otros ADR aceptados conservan el nombre viejo sin adenda ni puntero
Con el grep pedido sobre `docs/` quedan nombres viejos fuera de los permitidos en: F1-5 (13), F1-6 (9),
F1-8 (4), F1-9 (1), F1-7 (1), F1-10 (1, linea 97), F1-11 (9, lineas 21, 173, 317, 335, 367, 534, 536, 611,
627) y F1-13 (1, linea 451, que ya dice `taskcodetwo/tareas` en vez de `taskcode/tareas`, asi que es
correcto). El plan solo preveia las cabeceras de F1-2, F1-3 y F1-4, de modo que esto es consecuente con el
plan; se anota porque la regla «tc2 desinstalar reconoce lo suyo por nombre» (F2-14) y quien lea esos ADR
puede copiar el nombre viejo. Propuesta: ampliar el alcance de la adenda de F1-12 con la frase «y donde
cualquier ADR nombre estas piezas» en lugar de enumerar ADR, o anadir el puntero a F1-1, F1-5, F1-6 y
F1-11. F1-8, F1-9, F1-7 y F1-10 se pueden dejar con la adenda general.
Reproduccion: Grep del patron `taskcode/tareas|refs/taskcode/|taskcode-estado|\.git/tc2|\.taskcode/worktrees`
en `docs/` y `crates/`: 142 apariciones en 17 ficheros; en `crates/` solo una, y es la negativa a proposito.

## Alcance
- Ficheros revisados: `crates/tc2/src/huella.rs`, `estado/cerrojo.rs`, `identidad.rs`, `git/puerto.rs`
  (rechazo de `update-ref`), `git/mod.rs`, `estado/mod.rs`, `tests/convivencia_016.rs` y la tanda de tests
  que importa las constantes; ademas las adendas de `docs/adr/F1-12`, F1-2, F1-3, F1-4 y el plan.
- Contexto adicional pedido: ninguno.
- Limite JS-WASM: sin cambios de firma exportada; el diff no toca `#[wasm_bindgen]` ni tipos que crucen
  el limite (crate nativo).

## Lecturas fuera del contexto permitido
ninguna fuera de `tarea.md`, `planificacion/plan-final.md` y los ficheros del diff.

## Reproduccion
- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-116-r1-rust`, rama
  `fix/task-116-nombres-de-la-2-0-alineados-con-f1-12-d1`, HEAD 0097ddc. Se borra al terminar.
- Build: compilado en nativo como parte de `cargo test` y `cargo clippy`. No se compilo a `wasm32`: el
  diff no toca nada del limite JS-WASM.
- Formato y lints: `cargo fmt --check` limpio; `cargo clippy --all-targets --locked -- -D warnings` sin
  avisos.
- Suite: `cargo test -p tc2 --no-fail-fast`. Todo verde salvo los 4 tests de `tests/plataforma.rs`, que
  cayeron en la primera pasada con salidas `sinred` (la maquina cuelga a veces las conexiones por
  127.0.0.1, y habia otro cargo en paralelo). Se repitio `cargo test -p tc2 --test plataforma`: 4 de 4 en
  verde en 2,64 s. Lo digo expresamente: es la conocida inestabilidad de red local, no un fallo del cambio.
  Unitarios: 89 ok. `convivencia_016`: 1 ok. `alta` 10, `estado_bordes` 9, `estado_casos` 5,
  `estado_concurrencia` 3, `git_*` ok, `identidad` 6, `tarea` 7 (1 ignorado a proposito).
- Casos construidos: busqueda de nombres viejos en `crates/` y `docs/` (ver MEN-3). En `crates/` solo
  queda `refs/taskcode/pendientes/000001` en `git/puerto.rs:514`, que es el caso negativo: `update-ref`
  sobre el prefijo viejo debe rechazarse y se rechaza. No quedan `taskcode-estado`, `.git/tc2`,
  `.taskcode/worktrees` ni `taskcode-estado-tmp` en `crates/`.
- Mutacion: en `huella.rs` se cambio `"taskcodetwo"` por `"taskcode"` en la macro `nombre!()`.
  `tests/convivencia_016.rs` se puso rojo (la huella de la 0.16 cambia: `.git/taskcode/estado.lock`
  y `.taskcode/worktrees/...` aparecen mezclados con lo de la 0.16, en `convivencia_016.rs:164`), y los
  4 unitarios de `huella` tambien. Revertido con `git checkout` en el clon.
- Medidas: no aplican (sin cambios de rendimiento ni de limite WASM).

## Revisado sin hallazgos
- Macro `nombre!()` con `concat!`: todas las constantes salen de un solo literal; `TEXTO_LEEME`,
  `RAMA_ESTADO`, `PREFIJO_REFS`, `REFS_*` y `CARPETA_WORKTREES` se componen sin duplicar el nombre, y los
  reexports de `estado` y `git` apuntan a `huella`.
- `dir_propio` / `en_comun`: `create_dir_all` idempotente, el error de E/S sube con `?` y se
  convierte a `ErrorCerrojo::Io`; el cerrojo y la cache de identidad ya no dependen de que exista el padre.
- `git/puerto.rs`: `forma_de_update_ref` exige el prefijo `refs/taskcodetwo/`, no vacio y sin `..`; los
  casos negativos (`refs/taskcode/...`, `refs/taskcodetwo/` solo, `--no-deref`, `--stdin`, `-d` mal formado)
  estan en la tabla de pruebas.
- Produccion sin `unwrap`, `expect`, `panic!` ni `unsafe`: el unico `unsafe` que aparece en `src` es una
  mencion en un comentario de `i18n/mod.rs:42`.
- `convivencia_016.rs`: la huella cubre ficheros de `.git/taskcode/` con contenido, `.taskcode/`, refs
  de `develop` y `main` (local y remoto) y el arbol de `develop`; ademas exige que todo lo nuevo lleve
  nombre de la 2.0. Los `unwrap` y `expect` estan solo en tests, con `#![allow]` explicito.
