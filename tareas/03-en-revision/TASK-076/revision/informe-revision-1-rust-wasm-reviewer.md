# Informe de revision — TASK-076 (ronda 1)

- Commit revisado: ebc7fa5647f8386bf0266e16eab63a8a34d7e8a8
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/lib.rs:91 | plan |
| MEN-2 | MENOR | abierto | crates/tc2/src/git/rama.rs:119 | implementacion |
| MEN-3 | MENOR | abierto | crates/tc2/src/tarea/texto.rs:16 | especificacion |
| MEN-4 | MENOR | abierto | crates/tc2/src/alta.rs:106 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### MEN-1 — Aviso por stderr con codigo 0 contradice el punto 4 del ADR F1-4 y el ADR no se enmienda
El punto 4 de `docs/adr/F1-4-*.md` dice «Diagnostico solo en stderr y solo si falla». `lib.rs:91` escribe el aviso de parecidas en stderr con codigo 0 y lo reconoce en un comentario («unica excepcion»). Valoracion: la desviacion es razonable (stdout sigue cumpliendo el contrato, con `parecidas=<IDs>` como clave y `--json` con la lista; el aviso en stderr es solo ayuda humana, y D1 pide no bloquear). El defecto es de trazabilidad: el diff anade adenda al ADR F1-2 (punto 3) pero no al F1-4 (punto 4), asi que la excepcion solo vive en un comentario de codigo. Pedir una adenda de una linea al punto 4 del F1-4, o dejar anotado en el plan que se acepta sin adenda.

### MEN-2 — `id_de_rama` acepta cualquier `u32`: una rama ajena mueve o bloquea el ID
`task-<cifras>` se acepta mientras quepa en `u32`. Una rama `x/task-4294967295-y` en el remoto hace que `siguiente_id` devuelva `Fallo::Error(SIN_IDS)` y que todo `tc2 new` falle para todo el equipo (el unitario de `siguiente_id` con `u32::MAX` lo confirma; no lo reproduje de extremo a extremo). Mas realista: `feature/task-20261116-x` (fecha en el nombre) salta el ID a 20261117 sin avisar. Mitigacion barata: ignorar numeros con mas de, por ejemplo, 6 cifras (como ya se ignoran los que no caben en `u32`). De paso: la comparacion es sensible a mayusculas (`fix/TASK-11-m` no cuenta, test `id_de_las_ramas_de_tarea`) mientras `numero()` de las carpetas las ignora; es coherente con `start`, que crea en minuscula, pero queda anotado.

### MEN-3 — El umbral 0,80 avisa en series numeradas
Medido con la implementacion: «Migrar modulo A» / «Migrar modulo B» da casi 91; «Sprint 3 revision» / «Sprint 4 revision» da casi 85. Son tareas distintas que avisan. Cumple C5 tal como esta escrito y solo es aviso (codigo 0), y D2 ya preve que F9-4 mida el ruido; se deja constancia como dato para esa medicion.

### MEN-4 — `ABIERTOS` duplica la lista de estados
`alta.rs:106` repite a mano los cuatro estados abiertos de `tarea::modelo::ESTADOS` (los seis menos `terminada` y `descartada`). Si se anade un estado abierto, el aviso lo ignorara en silencio. Derivarlo de `ESTADOS` o anadir un test que cruce ambas listas.

## Alcance
- Ficheros revisados: `crates/tc2/src/alta.rs`, `estado/lectura.rs`, `git/estado.rs`, `git/mod.rs`, `git/puerto.rs`, `git/rama.rs`, `i18n/catalogo/estado.rs`, `i18n/catalogo/git.rs`, `lib.rs`, `tarea/mod.rs`, `tarea/modelo.rs`, `tarea/texto.rs` y los tests `tests/alta.rs`, `tests/estado_casos.rs`, `tests/comun/mod.rs`, `tests/tarea.rs`.
- Contexto adicional pedido: `docs/adr/F1-4-*.md` (punto 4, contrato de salida) para valorar MEN-1.
- Limite JS-WASM: sin cambios; no hay `#[wasm_bindgen]` en el diff (CLI nativo).

## Reproduccion
- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-076-r1-rust`, rama `feature/task-076-f2-4b-nombres-unicos-de-tarea-y-rama` en `710bc92`. Borrado al terminar.
- Build: nativo, via `cargo test` y `cargo clippy`; no se compila a wasm32 (el diff no toca ese limite).
- Formato y lints: `cargo clippy --all-targets --locked -- -D warnings` sin avisos; `cargo fmt --check` limpio.
- Suite: `cargo test -p tc2`. Primera pasada: `plataforma_gitlab::protecciones_de_gitlab` fallo con `estado-puertas sinred` (el cuelgue conocido de 127.0.0.1; test ajeno al diff). Segunda pasada con `--no-fail-fast`: todo en verde (72 unitarios, `alta` 9/9, `estado_casos` 5/5, `plataforma_gitlab` 3/3, los E2E contra GitHub y GitLab ignorados por diseno).
- Casos construidos (sonda temporal en el clon, ya eliminada):
  - Fuzz de 200.000 titulos aleatorios (combinantes, `İ`, `ß`, ancho completo, emoji, NUL, `\u{200b}`, saltos de linea): el slug siempre mide <= 40 bytes, no esta vacio, no empieza ni acaba en guion, no tiene `--` y es solo `[a-z0-9-]`; `plegar` es idempotente; `parecido(s,s)` es duplicado y `parecido(a,b) == parecido(b,a)`.
  - Cortes del slug: 41 caracteres con espacio en el 41 da 40; guion justo en la posicion 40 da 40 sin guion final; guion en la 39 da 39.
  - Dice: la tabla de C5 coincide; ademas se midieron los pares de MEN-3.
  - `ejecutar_con_entrada(cat-file --batch)`: repo real con un blob de 8 MiB leido 10 veces, 30.000 rutas inexistentes y una ruta con espacio y `ñ`: 85,9 MB de salida, codigo 0, sin bloqueo (37 s en esta maquina, dominados por git en Windows); un argumento invalido con 1,5 MB de entrada pendiente devuelve codigo 129 sin colgarse. Con `UpdateRef`, `Push` y `Show` da `Prohibido`.
- Mutacion:
  - Quitar `.chain(ocupados)` de `siguiente_id`: se ponen rojos `rama_solo_en_local_ocupa_su_id` y `rama_solo_en_el_remoto_ocupa_su_id` (C2 y C3).
  - Subir `UMBRAL` de 80 a 81: se pone rojo `umbral_de_casi_igual` (C5).
  - Quitar la comprobacion del sufijo `-` en `id_de_rama`: se pone rojo `id_de_las_ramas_de_tarea`.
- Medidas: no aplican (sin limite JS-WASM); el alta hace un `git cat-file --batch` por intento, comprobado por `el_alta_lee_las_tareas_en_un_solo_proceso_por_intento`.

## Revisado sin hallazgos
- Plegado y slug: tabla Latin-1 y Latin Extended-A, marcas combinantes, corte en guion; vacio o sin letras latinas da `tarea` (C7). Sin `unwrap`, `expect`, `panic!` ni `unsafe` en produccion en los ficheros del diff.
- Dice: multiconjunto de bigramas con fusion ordenada correcta; la division entera truncada hace que `>= 80` equivalga a `Dice >= 0,80` exacto; `total == 0` devuelve 0 sin dividir.
- `ejecutar_con_entrada`: la escritura de stdin va en otro hilo y la salida se lee con `wait_with_output`, asi que no hay interbloqueo con entradas ni salidas grandes; solo admite `cat-file`; los argumentos de `cat-file` solo pasan el filtro comun (`--force*`), pero la orden es de lectura y su unico llamador fija `--batch`.
- `partir_batch`: lee el tamano en bytes, exige el salto final, trata `missing` y `ambiguous` como ausente, solo da contenido a `blob`, rechaza salidas truncadas o con sobrante; las rutas con salto de linea van aparte por `leer_blob`.
- `Vista::leer_varios`: la correspondencia entre el filtro de rutas y el recorrido final es coherente (superpuestas, luego commit, luego ausentes).
- `ids_con_rama`: sin remoto mira solo lo local; un fallo del remoto da `sinred` (4) o `denegada` (8); sin refs nuevas (C6).
- Carreras entre dos altas: el ID lo decide el push de la rama de estado; el perdedor reintenta sobre el estado fresco y recalcula ID y parecidas (E2E `mismo_titulo_a_la_vez_desde_dos_clones`, en verde, C1). El hueco entre `ls-remote` y el push lo declara el plan como riesgo aceptado hasta F3-6.
- `Vec::insert` en `orden_new`: los indices 0, 1 y 2 son siempre validos porque cada insercion alarga el vector.
