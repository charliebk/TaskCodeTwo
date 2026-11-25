# Informe de revision — TASK-142 (ronda 1)

- Commit revisado: d472e5db9a48898501273db538aa655aaf828c5f
- Revisor: general-purpose (sonnet) con rust-wasm-reviewer
- Veredicto: aprobada

Veredicto propuesto: **aprobada**. Sin hallazgos CRITICO, IMPORTANTE ni MENOR.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|

sin hallazgos

Reproduccion, resumen:

- Todos los mutantes aplicados se detectan (ver `## Reproduccion`).
- Los unicos cambios de comportamiento observables son los tres aceptados por
  el plan (decisiones 1 y 4 y A4-01/A4-02); ninguna otra linea de contrato de
  git cambia (las lineas completas de `actualizar-rama`, `tag-anotado`,
  `crear-rama-y-worktree` y `reanudar` estan fijadas por tests y pasan).
- La guarda desplegada de A4-10 no rechaza ninguna llamada real de la capa:
  no hay en `crates/tc2/src` ningun argumento literal de un guion y varias
  letras (`"-[A-Za-z]{2,}"`, fuera de `puerto.rs`). Los valores dinamicos
  (refs, rutas) no empiezan por `-`.
- `lanzado` y `exigida` no ocultan un codigo de salida: con el mutante de
  `exigida` que no mira el codigo se ponen rojos
  `con_entrada_da_error_si_git_falla` y
  `leer_blobs_diagnostica_la_primera_linea_no_vacia`.
- API publica: sin cambios. Lo nuevo es `pub(crate)` o privado;
  `sin_red_remoto` pasa de `pub(crate)` a privada y `SalidaGit::primera_linea_error`
  conserva su firma.

## Regla 4D

- Duplicacion: cumple. Ya no quedan en `git/` ni en `estado/transaccion.rs`
  copias de lo unificado: no hay `map_err` a `Informe::nuevo(.., Error).con("git", ..)`
  fuera de `lanzado`/`exigida` (solo en `git/mod.rs:60` y `:72`), ni otro
  analizador de `status --porcelain -z` (solo `registros_status`), ni
  `push_fallo`/`fallo_fetch`, ni la doble copia de `op.aplicar` en
  `transaccion.rs`. Las clasificaciones del fallo del remoto de
  `estado/montaje.rs:139`, `init/inventario.rs:472` y `git/remoto.rs:291`
  (push) quedan aparte; tienen otra forma de retorno (`Previsto`, `Informe`
  de init, `Intento`) y el plan las declara fuera a proposito (decision 5), asi
  que no las cuento como hallazgo. Los candidatos 4D de la peticion son falsos
  positivos por nombre parecido (`git_falso`/`git_fallo`, `lanzado`/`Lanzada`,
  `exigida`/`exigidas`, `preparar`, `rechazo`, `anadir`): ninguno repite logica.
- Desviacion: cumple. Los cambios de comportamiento son los aceptados y cada
  uno tiene su test (A4-01 en `git_push.rs`, A4-02 en `estado.rs`, A4-10 en
  `puerto.rs`); no hay cambios de firma publica ni de contrato fuera de ellos.
- Diluvio: cumple. `Salida` (trait de dos metodos) sirve a `SalidaGit` y
  `SalidaCruda`, que no tenian forma comun; `DestinoWt` agrupa lo que `crear` y
  `reanudar` repetian; sin capas sobrantes.
- Disfraz: cumple. Los tests nuevos comprueban lineas y diagnosticos enteros, y
  se ponen rojos con sus mutantes.

## Alcance

- Ficheros revisados: `estado/transaccion.rs`, `git/{estado,guardar,mod,puerto,rama,remoto,tag}.rs`
  y los tests de `crates/tc2/tests` de la peticion (los de git leidos; el resto,
  ejecutados).
- Contexto adicional pedido: ninguno.
- Limite JS-WASM: sin cambios de firma exportada con `#[wasm_bindgen]`; el
  diff es Rust nativo (CLI), no toca el limite.

## Reproduccion

- Clon: `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-142-r1\repo`, rama `fix/task-142-auditoria-4d-a4-git-rama-de-estado-y-tom` (HEAD ab56267; el codigo es el de d472e5d, el ultimo commit solo anade la peticion). `CARGO_TARGET_DIR` en la misma carpeta.
- Build: via `cargo test`/`clippy` con `--locked`, nativo; sin wasm32 (el diff no afecta al limite).
- Formato y lints: `cargo fmt --check` limpio; `cargo clippy -p tc2 --all-targets --locked -- -D warnings` limpio.
- Suite: linea base de la peticion (789 antes; 795 ok + 1 intermitente de `plataforma`, 13 ignorados). Yo corri solo lo pedido, uno a uno: `--lib` 408 ok (un fallo de `descarga::tests::timeout_global_no_se_dobla` por plazo de 10 s con la maquina cargada por otro `cargo test --workspace` concurrente; repetido en solitario: ok); `git_push` 3 ok; `git_ciclo` 2 ok; `git_casos_limite` 7 ok; `git_conflicto` 2 ok; `estado_bordes` 10 ok; `estado_casos` 8 ok; `estado_concurrencia` 5 ok; `toma` 13 ok; `liberar` 12 ok.
- Casos construidos: los de los tests nuevos ejecutados en verde: `comprobar-id`/`tag-anotado` contra un remoto que no es repositorio dan `denegada ... motivo=remoto-no-existe` (codigo 8); `comprobar-id` sin `origin` da `ok` y `rechazada ... ocupada` con lo local; `cat-file` con stderr `"\nfatal: x"` da `fatal: x` en el diagnostico; copia `C` en el analizador de status; opciones agrupadas `-qf`, `-qD`, `-ff` rechazadas y `-qv` permitida.
- Mutacion (todos en rojo; tras cada uno, `git checkout` del clon):
  - `exigida` sin mirar el codigo → rojo en `--lib` (2 tests).
  - `registros_status` pierde el origen → rojo en `--lib` (2) y `git_casos_limite::guardar_renombrados_y_borrados_ya_en_el_indice`.
  - `fallo_del_remoto` clasifica todo como `sinred` → rojo en `git_push::push_reintenta_lo_transitorio_y_rechaza_lo_que_no_procede`.
  - `informe_publicacion` con `intentos` antes de `motivo` → rojo en `git_casos_limite::rechazo_del_remoto_es_denegada_sin_reintento`.
  - `es_prohibido` sin desplegar grupos → rojo en `rechaza_cada_argumento_prohibido_sin_lanzar_git`.
  - `ramas_de_tarea` sin mirar `hay_remoto` → rojo en `git_push::comprobar_id_sin_origin_mira_lo_local`.
  - `aplicar_en` sin la comprobacion `duplicada` → rojo en `estado_bordes` (2 tests).
- Medidas: no aplican (sin tamano de `.wasm` ni rendimiento). Coste anotado por el plan: un `ls-remote --get-url` local por `comprobar-id`.

## Revisado sin hallazgos

- `ramas_de_tarea`: orden, deduplicado y `branch --list` iguales a los de las dos copias anteriores; para `ids_con_rama` el comportamiento es identico; `sinred` genera el mismo diagnostico que antes.
- `consultar_remoto`/`fallo_del_remoto`: `tag::commit_remoto` ya no deja un fallo no-red como `sinred`; `fetch` y `traer_rama` conservan su clasificacion.
- `exigida`/`lanzado`/`git_fallo`: `con_indice`, `con_entrada`, `leer_blobs` y `construir_commit_sellado` mantienen `git=<sub>` y el diagnostico; en `leer_blobs` el `error=` pasa a la primera linea no vacia y recortada (aceptado).
- `hay_remoto` = `url_origin(..).is_some()`: misma condicion que antes.
- `registros_status`/`entradas_con_origen`: `R` aporta `D origen`, `C` no, igual que antes.
- `informe_publicacion` y sus tres llamadores (`push_paso`, `actualizar`, `tag_paso`): mismo orden de claves; la linea completa de `actualizar-rama` queda fijada por test.
- `DestinoWt`: `base-no-existe` sigue entre `carpeta_ocupada` y `anadir` en `crear`; `reanudar` conserva el orden.
- `es_prohibido`: sin llamadas reales de la capa afectadas por el despliegue; falla cerrando con valores pegados, como declara el plan.
