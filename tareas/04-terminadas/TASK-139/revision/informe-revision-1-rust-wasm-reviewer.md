# Informe de revision — TASK-139 (ronda 1)

- Commit revisado: ff05668b1bee0f89850a7d1ffb3ca0ed3cf3d2b0
- Revisor: general-purpose (sonnet) con rust-wasm-reviewer
- Veredicto: aprobada

Veredicto propuesto: **aprobada** (sin hallazgos; dos observaciones sin severidad al final).

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|

sin hallazgos. Ningun mutante relevante sobrevivio y no se encontro ninguna linea de contrato cambiada fuera de la decision 1 (ver `## Reproduccion`).

## Regla 4D

- Duplicacion: cumple. En `github.rs` y `gitlab.rs` ya no queda ninguna copia de `validar_puertas`, `sin_reglas_por_plan`, `pr_abierto`/`mr_abierto` con su flujo, el bucle de `prs_abiertos`, la validacion de `abrir_pr`/`merge_pr` ni la lectura en lote (`git grep` de `rol_no_valido`, `rama_valida(` y `sha_valido(` en los dos adaptadores solo deja usos de otras operaciones: `estado-puertas`, `leer-plan`, `firma-commit`, etc., fuera del alcance A1). En los tests, `GitHub::nuevo`/`GitLab::nuevo` solo quedan en `tests/comun/adaptadores.rs` y en casos con otra configuracion (proyecto con nombre y `Base::nueva` propia en `plataforma_gitlab.rs`, variables de entorno en los E2E), no como copia del constructor con otro nombre; la tabla de errores, el servido de fixtures y el cliente HTTP del arnes viven una vez.
- Desviacion: cumple. `git diff origin/develop ff05668 -- crates/tc2/src` no quita ninguna linea `pub` y solo anade `pub(crate)` (`operaciones`, `mergeado_en`, `pr_cerrado_en`); `puertas::mergeado` y `pr_cerrado` conservan su firma y delegan. Las unicas lineas de contrato que cambian son las de la decision 1 (`abrir-pr conflicto motivo=pr-existente`, codigo 6, en GitHub; GitLab pasa de `mr-existente` a `pr-existente`), y la 4 declarada (un 403 de plan que ademas es limite da `agotada`).
- Diluvio: cumple. `operaciones.rs` (215 lineas) tiene un trait de tres metodos con dos implementaciones reales (`AbrePr`, con `antes_de_crear` por defecto que solo usa GitLab), un enum de tres variantes que modela lo que de verdad pasa al crear (`Creado`), un struct de datos (`PrLeido`) y funciones libres con cierres; no hay capas ni genericos sin uso. El `OnceCell` de `leer_puertas` (una sola llamada) es innecesario pero evita una segunda ruta de codigo.
- Disfraz: cumple. Los tests nuevos (`carrera_de_abrir_pr_da_la_misma_linea`, `leer_puertas_pagina_los_rulesets`) comprueban lineas, codigos y peticiones del mock, y se ponen rojos con los mutantes de abajo; ninguno descarta el resultado.

## Alcance

- Ficheros revisados: los 15 del diff de dominio de la peticion (`src/i18n/catalogo/plataforma.rs`, `src/plataforma/{github,gitlab,http,mod,operaciones,puertas}.rs`, `tests/arnes_e2e/mod.rs`, `tests/comun/{adaptadores,mod}.rs`, `tests/{plataforma,plataforma_e2e,plataforma_gitlab,plataforma_gitlab_e2e,prs_abiertos}.rs`).
- Contexto adicional pedido: ver `## Lecturas fuera del contexto permitido`.
- Limite JS-WASM: sin cambios de firma exportada; el diff no toca `#[wasm_bindgen]` ni tipos que crucen el limite (la API publica de `tc2::plataforma` no cambia).

## Reproduccion

- Clon: `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-139-r1\repo`, rama `fix/task-139-auditoria-4d-a1-plataforma-e-identidad-1`, checkout en `ff05668` (el commit revisado); `CARGO_TARGET_DIR` propio en `...\task-139-r1\target`. Carpeta y target borrados al terminar (sin commits ni nada que conservar: solo mutaciones revertidas).
- Build: `cargo test --locked` compila nativo en cada comando de abajo; no se compila a `wasm32` porque el crate `tc2` no tiene objetivo WebAssembly.
- Formato y lints: `cargo fmt --check` sin salida; `cargo clippy -p tc2 --all-targets --locked -- -D warnings` limpio.
- Suite: no se repite la suite completa (la peticion trae 783 antes y 785 ok, 0 fallos, 13 ignorados despues, y el E2E real en verde). Se ejecutaron en `ff05668`: `--lib` 400 ok y 2 ignorados; `--test plataforma` 8 ok; `--test plataforma_gitlab` 7 ok; `--test prs_abiertos` 4 ok; `--test revision` 5 ok; `--test tablero` 6 ok. `--test doctor` solo se ejecuto bajo mutantes (ver abajo); en la linea base de este clon no se corrio.
- Casos construidos: lectura del diff contra la rama base para cada operacion (orden de validaciones, orden de lecturas y escrituras, lineas de resultado). Se comprobo que el orden es el mismo que antes del refactor: abrir-pr (validar rama, validar titulo, leer abierto, antes de crear, crear, releer en la carrera), merge-pr (sha, persona, leer, estado, head, merge), prs-abiertos (rol, propio, rama valida, sha valido, orden por numero), leer-puertas (rama y rol, lo comun una vez en la primera rama valida, checks solo en `checks_de`). RR-15: el test `carrera_de_abrir_pr_da_la_misma_linea` da `abrir-pr conflicto motivo=pr-existente` y codigo 6 en las dos plataformas, byte a byte.
- Mutacion (cada mutante aplicado a un clon limpio y revertido; se corrieron `--lib`, `plataforma`, `plataforma_gitlab`, `prs_abiertos`, y `doctor` donde se indica). Todos en rojo:

| Mutante | Fichero | Test que lo mata |
|---|---|---|
| M1 motivo `pr-existente` cambiado | operaciones.rs | `carrera_de_abrir_pr_da_la_misma_linea` (plataforma_gitlab) |
| M2 abrir-pr no devuelve el PR ya abierto (idempotencia) | operaciones.rs | `bateria_de_la_poc_m_e_idempotencia` (plataforma) y `bateria_compartida_con_github_e_idempotencia` (plataforma_gitlab) |
| M3 la carrera no relee (siempre `conflicto`) | operaciones.rs | las dos baterias de idempotencia |
| M4 `segun_estado`: `!abierto` invertido | operaciones.rs | cuatro tests de plataforma y `envoltorio_y_errores_de_gitlab` |
| M5 `segun_estado`: mergeado devuelve la linea de cerrado | operaciones.rs | las dos baterias de idempotencia |
| M6 lectura de lo comun adelantada (no perezosa) en `leer_en_lote` | operaciones.rs | `permisos_insuficientes_y_ci_que_aprueba_fallan` (doctor, 483 s por los plazos) |
| M7 checks publicados en todas las ramas del lote | operaciones.rs | `ramas_protegidas_sin_fallos_dan_cero` (doctor) |
| M8 `leer-puertas` sin rechazar al revisor | operaciones.rs | `protecciones_y_bordes` y `protecciones_de_gitlab` |
| M9 `merge-pr` sin validar el sha | operaciones.rs | `protecciones_y_bordes` (solo GitHub, ver observacion 2) |
| M10 GitLab sin comprobar commits antes de crear | operaciones.rs | tres tests de plataforma_gitlab |
| M11 `prs-abiertos` sin ordenar | operaciones.rs | `misma_lista_en_github_y_gitlab_sin_forks_y_paginada` |
| M12 `merge_fallido` sin `pr=` delante | operaciones.rs | `errores_dirigidos_a_la_persona` y `envoltorio_y_errores_de_gitlab` |
| M13 `prs-abiertos` sin rechazar al revisor | operaciones.rs | `misma_lista_en_github_y_gitlab_sin_forks_y_paginada` |
| M14 `prs-abiertos` sin filtrar forks | operaciones.rs | `misma_lista_en_github_y_gitlab_sin_forks_y_paginada` |
| M15 sha sin pasar a minusculas | operaciones.rs | `misma_lista_en_github_y_gitlab_sin_forks_y_paginada` |
| M16 sha invalido aceptado | operaciones.rs | `sha_invalido_falla_y_rama_rara_se_salta` |
| M17 403 de plan de rulesets ya no es `None` | github.rs | `protecciones_y_bordes` |
| M18 `rulesets` recorta a menos de 100 (pagina 2 perdida) | github.rs | `leer_puertas_pagina_los_rulesets` |

- Medidas: no aplican (sin cambios en el limite JS-WASM; el refactor no toca caminos calientes).

## Revisado sin hallazgos

- `operaciones.rs`: orden de validaciones y de lecturas igual al previo en `abrir_pr`, `entrada_merge`, `segun_estado`, `prs_abiertos`, `leer_puertas` y `leer_en_lote`; el error de lo comun se cachea en el `OnceCell` y se clona por rama, como antes.
- `github.rs`: `rulesets` pasa a `rulesets_de` (paginado) y recupera el `None` del 403 de plan por `rechazada motivo=plan`; un 403 que es ademas limite de peticiones da ahora `agotada` (decision 4, antes `None`). `puertas_de_rama` con `Option<&[String]>` devuelve lo mismo que `sin_reglas_por_plan` antes. `es_aviso_de_plan` sigue en uso en `estado_puertas` (clippy limpio).
- `gitlab.rs`: `abierto`, `antes_de_crear` y `crear` conservan el orden del flujo previo; el 409 se comprueba antes de `!exito`, y `Creado::YaExiste` lleva el diagnostico `MR_EXISTENTE`.
- `puertas.rs`: `mergeado` y `pr_cerrado` delegan en las variantes `pub(crate)` con la misma linea.
- Catalogo: `PR_EXISTENTE` con `{api}` en es y en (como `MR_EXISTENTE`).
- Tests comunes y arnes E2E: sin copias residuales; los E2E compilan (clippy `--all-targets`).

## Observaciones (sin severidad, no son hallazgos)

1. La lectura perezosa de lo comun y los checks solo en `checks_de` de `leer_en_lote` solo las ejercita `tests/doctor.rs` (M6 y M7 salieron en rojo ahi, y la suite tarda de 200 a 480 s cuando falla por plazos). Ni `operaciones.rs` ni `tests/plataforma*.rs` tienen un test directo del lote; la proteccion existe, pero el rojo llega lejos del cambio.
2. `merge-pr` con sha invalido solo se prueba en GitHub (`protecciones_y_bordes`); el mutante M9 sobrevive en la bateria de GitLab. Al ser el mismo codigo neutral no es un hueco de comportamiento, pero RR-15 no se comprueba byte a byte para esa linea en GitLab.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/plataforma/errores.rs` (lineas 30-180): para confirmar que el 403 de plan sigue dando `rechazada motivo=plan` y el limite `agotada` en `traducir` (la decision 4 y el mapeo de `rulesets`).
- `crates/tc2/src/plataforma/conexion.rs` (lineas 170-240): `leer_paginas`, para confirmar la paginacion y los errores de `rulesets_de`.
- `crates/tc2/tests/doctor.rs` (solo ejecutado, no leido): lo usan `leer_puertas_de` y los mutantes M6 y M7.
