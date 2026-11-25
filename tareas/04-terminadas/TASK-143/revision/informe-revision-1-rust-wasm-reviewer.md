# Informe de revision — TASK-143 (ronda 1)

- Commit revisado: 38bd5b5a2a0260a0046096e4c1764798472d8dea
- Revisor: general-purpose (sonnet) con rust-wasm-reviewer
- Veredicto: aprobada

Veredicto propuesto: **aprobada** (sin hallazgos; todos los mutantes relevantes se ponen en rojo).

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|

Sin hallazgos. Reproduccion en `## Reproduccion`.

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

## Regla 4D

- Duplicacion: cumple. `inventario::skills_de` es la unica lectura de skills de una carpeta: `grep SKILL.md` en `src` solo da `inventario.rs:504`; `catalogo/de_carpeta` ya no tiene `cabecera`, `read_dir` ni `MAX_CABECERA` propios (solo conserva el tope en `recurso_de`, que recorta texto ya leido). `nombre_de` es unica y la usan `leer_una` y `catalogo::de_base`. `de_instalados` ya no relee `installed_plugins.json` ni repite la regla de eleccion de instalacion: usa `Plugin::carpeta`. En tests, `conectar`/`doctor_contra_en` sustituyen los tres cierres `Conectada {..}` (queda uno, `doctor.rs:510`) y `comun::salida_acotada` sustituye a las dos copias de `doctor.rs` y `recursos.rs` (`grep` de `malos > 4` y `lineas.len(), 5`: solo en el ayudante). Sin gemelas.
- Desviacion: cumple. API publica: solo cambian `Plugin::carpeta` (campo nuevo), `catalogo::de_instalados(&Inventario)` (sin `dir_claude`) y `catalogo::Lectura` (campos `ilegibles` y `avisos`). `nombre_de`, `skills_de`, `Leida` son `pub(crate)`; `Fuente` es privado. Los cambios de comportamiento son solo los aceptados: aviso `ilegible:<motivo>:<id>` unicamente con `--plugin` (`orden.rs`); revisoras tomadas de la instalacion que elige el inventario (`aplica`); S11 con `dominio-sin-revisora` para una revisora de dominio no instalada (`revisoras_de`). `Inventario::huella` no incluye `carpeta` (confirmado en `inventario.rs:204-227`).
- Diluvio: cumple. Un enum privado de dos variantes (`Fuente`) sustituye al booleano `omitir`; `Leida` es el minimo para pasar cabecera y ruta al catalogo.
- Disfraz: cumple. Los tests nuevos (`la_carpeta_del_plugin_es_la_del_scope_del_proyecto`, `skills_de_una_carpeta_deja_incidencias`, `plugin_con_skills_ilegible_da_aviso`, `revisora_de_dominio_no_instalada_es_dominio_sin_revisora`) fallan con su mutante; `las_reglas_se_leen_de_la_tabla` ahora exige `Ok` y la regla `M04`.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/doctor/mod.rs:307` (solo esa zona) y `crates/tc2/src/inventario.rs` fuera del diff (`aplica`, `leer_plugins`, `huella`): para aplicar mutantes sobre el producto (`y N mas`) y comprobar que `huella` no incluye `carpeta`.

## Alcance

- Ficheros revisados: `inventario.rs`, `recursos.rs`, `seleccion/catalogo.rs`, `seleccion/mod.rs`, `seleccion/orden.rs`, `tests/comun/lanzador.rs`, `tests/comun/mod.rs`, `tests/doctor.rs`, `tests/recursos.rs`, `tests/revisoras.rs`.
- Contexto adicional pedido: ninguno aparte de lo anotado arriba.
- Limite JS-WASM: sin cambios; el diff no toca `#[wasm_bindgen]` ni tipos que crucen el limite.

## Reproduccion

- Clon: `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-143-r1\repo`, rama `fix/task-143-auditoria-4d-a5-seleccion-inventario-rec`, con `CARGO_TARGET_DIR` en la misma carpeta (borrado al terminar).
- Build: nativo (`cargo clippy --locked -p tc2 --all-targets` compila todo). No hay objetivo wasm32 en esta tarea.
- Formato y lints: `cargo fmt --check` limpio; `cargo clippy --locked -p tc2 --all-targets -- -D warnings` limpio.
- Suite: se acepta la linea base de la peticion (48 binarios, 800 ok, 0 fallos, 13 ignorados; antes 796). Ademas, ejecutados uno a uno: `--lib` 411 ok / 2 ignorados; `--test revisoras` 6 ok; `--test seleccion` 2 ok; `--test doctor` 12 ok (193 s); `--test recursos` 12 ok; `--test desinstalar` 19 ok. Sin intermitencias.
- Casos construidos: instalacion de otro proyecto delante de la de usuario, con y sin `version` (el inventario elige la de usuario y el catalogo lee `p:de-b`); carpeta `skills` que es un fichero (aviso `ilegible:no-se-lee:roto:skills` con `--plugin`); revisora de dominio declarada en `recursos:` y no instalada (S11 y `dominio-sin-revisora`, sin S10 ni S14), e instalada (S10 sin aviso).
- Mutacion (cada mutante aplicado sobre el clon y revertido con `git checkout`):
  - `skills_de` no anota ilegibles (`hijos(..)` con un vector desechable): rojo en `skills_de_una_carpeta_deja_incidencias`.
  - `orden::fuentes` sin el `avisos.extend` de ilegibles: rojo en `plugin_con_skills_ilegible_da_aviso`.
  - `aplica` acepta cualquier `scope`: rojo en `la_carpeta_del_plugin_es_la_del_scope_del_proyecto`.
  - `revisoras_de` sin `self.instalado(&r.id)`: rojo en `revisora_de_dominio_no_instalada_es_dominio_sin_revisora`.
  - `huella` con la `carpeta`: verde en `--lib` y `revisoras`, rojo en `tests/recursos.rs::determinista` (la red es de integracion, y es la correcta: la huella debe ser igual entre carpetas).
  - `salida_acotada` sin comprobar `lineas[4]`: sobrevive, pero es quitar la propia asercion del ayudante, no un fallo del producto. Mutantes equivalentes sobre el producto (`doctor/mod.rs:307`, cuenta de `y N mas` y texto del detalle): rojo en `recursos::texto_acotado_y_json_completo` y en `doctor::salida_acotada_y_json_con_todo`.
- Medidas: no aplican.

## Revisado sin hallazgos

- Orden de `ilegibles` en `leer_carpetas`: ahora se intercalan las de skills y agentes; no hay test ni consumidor que dependa del orden entre ambas listas.
- Observacion, sin demostracion de daño: para un plugin instalado, las skills se leen una vez en el inventario y otra en `de_instalados` (`de_carpeta`). Cada lectura es unica en el codigo (A5-01), no hay copia, y las incidencias de la segunda se descartan porque ya estan en el inventario.
- `de_instalados` sin `claude`: antes se saltaba si `claude` era `None`; ahora el inventario sin plugins devuelve vacio. Mismo resultado.
- `Plugin::carpeta` es `None` en sincronizados: no se leen revisoras de ellos, como antes.
