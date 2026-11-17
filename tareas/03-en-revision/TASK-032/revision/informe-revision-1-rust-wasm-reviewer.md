# Informe de revision — TASK-032 (ronda 1)

- Commit revisado: 5a5a9e9e08d7cc4180d8dd4ee3760a24ec78677a
- Revisor: rust-wasm-reviewer
- Veredicto: PENDIENTE (escribelo con: taskctl veredicto TASK-032 aprobada | aprobada-con-correcciones | cambios-solicitados)

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/tablero.rs:255 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/tablero.rs:431 | plan |
| MEN-3 | MENOR | abierto | crates/tc2/src/plataforma/github.rs:321 | implementacion |
| MEN-4 | MENOR | abierto | crates/tc2/src/tablero.rs:151 | implementacion |

Ninguno CRITICO ni IMPORTANTE.

### MEN-1 — `Ilegibles:` se imprime sin sanear

`texto_board` imprime `t.ilegibles.join(", ")` tal cual. Cada entrada es el
primer segmento de una ruta de la rama de estado (`situacion.rs:131`), no un
ID validado: un nombre de carpeta con `ESC[...` o caracteres bidi llega crudo
a stdout. El resto del texto ajeno (titulo, quien) si pasa por `sanear`
(F1-17, decision 8). Efecto acotado: exige poder escribir en la rama de
estado y solo afecta al texto, no al `--json`. Arreglo: `sanear(x, MAX_QUIEN)`
por entrada.

### MEN-2 — Varios PR en la misma rama: gana el de menor numero, aunque sea borrador

`cruzar` usa `min_by_key(numero)`. Si la misma rama tiene un PR borrador antiguo
(p. ej. contra otra base) y otro listo mas nuevo, `board` muestra el borrador
y avisa `revision-sin-pr` para una tarea en revision con PR listo. La regla
esta en el plan y en la docstring, y GitHub solo admite dos abiertos con la
misma cabeza si cambia la base, asi que es un caso raro. Se podria preferir el
no borrador y, entre iguales, el menor numero.

### MEN-3 — Mensaje de respuesta invalida reutilizado de otra operacion

`prs_abiertos` falla con `t::RAMA_BASE_O_SHA` (GitHub) y `RAMA_DESTINO_O_SHA`
(GitLab), textos escritos para `estado-puertas`. El usuario de `board` ve
`plataforma=error` y la causa solo en el diagnostico. No afecta al resultado.
Pasa tambien con un PR propio sin `sha` (default vacio) en la lista, que falla
cerrado toda la consulta por un solo PR raro: es la politica elegida y esta
probada.

### MEN-4 — `sanear` deja pasar algunos caracteres de formato

El filtro cubre U+200B..200F, U+202A..202E, U+2066..2069 y U+FEFF, pero no
U+2060..2064 (word joiner, separadores invisibles), U+00AD ni U+061C. Los
espacios Unicode (U+2028/2029) si caen en `split_whitespace`. Sin efecto en
terminales normales; es una cobertura mas estrecha que la que dice la
docstring («sin caracteres de formato»).

## Alcance
- Ficheros revisados: `crates/tc2/src/tablero.rs`, `plataforma/{mod,github,gitlab}.rs`, `lib.rs`, `i18n/catalogo/{mod,tablero,cli}.rs`, `identidad.rs`, `roadmap/{mod,generar}.rs`, `siguiente.rs`, `situacion.rs`, `tests/prs_abiertos.rs`, `tests/tablero.rs`.
- Contexto adicional pedido: `plataforma/conexion.rs` (`leer_paginas`, `MAX_PAGINAS`) y `plataforma/credenciales.rs` (`CredencialesEntorno`), para juzgar la paginacion y la eleccion de rol.
- Limite JS-WASM: sin cambios; el diff no toca `#[wasm_bindgen]` ni `.d.ts`.

## Reproduccion
- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-032-r1-rust`, rama `feature/task-032-f3-5-board-y-status` (HEAD ec47b9f, codigo igual al revisado 5a5a9e9).
- Build: lo hace `cargo test` y `cargo clippy` (nativo; sin objetivo wasm32 en este crate).
- Formato y lints: `cargo clippy --all-targets --locked -- -D warnings` limpio.
- Suite: `cargo test -p tc2` en verde, todos los binarios `ok`, 0 fallos (153 unitarios en lib, mas integracion con `prs_abiertos` y `tablero`).
- Casos construidos: lectura de los literales de los ficheros editados con scripts (ver abajo); no hizo falta caso nuevo, los hallazgos son de lectura y se marcan como MENOR.
- Mutacion: `propio = true` (quita la exclusion de forks) en `github.rs` pone en rojo `tests/prs_abiertos.rs` (1 fallo de 4). `min_by_key` → `max_by_key`, ya comprobado por el implementador en su autorrevision, no repetido. Clon restaurado con `git checkout`.
- Medidas: no aplican.

## Revisado sin hallazgos
- Literales de los ficheros editados con scripts: `github.rs` (`{r}/pulls?state=open&per_page=100`, `rename = "ref"`), `gitlab.rs` (`{p}/merge_requests?state=opened&per_page=100`), `lib.rs` (registro y bloque `orden_tablero`), `i18n/catalogo/mod.rs` (`pub mod tablero;` y `tablero::TODOS`), cadenas de uso de `cli.rs` con `\"<mensaje>\"` bien escapadas y la constante JSON de `tests/tablero.rs`, y `roadmap/mod.rs` (solo `pub(crate)` en `tests`, `EnMemoria`, `estado_de_referencia`, `personas`). Sin escapes perdidos.
- Cruce por rama: forks excluidos por id de repo (GitHub, con `repo: null` tambien excluido) y por `source_project_id != target_project_id` (GitLab); ramas invalidas saltadas por `rama_valida`; sha invalido falla cerrado; resultado ordenado por numero; tarea sin rama no casa.
- Paginacion: `leer_paginas` con `MAX_PAGINAS = 10` y 100 por pagina; con mas, `error motivo=demasiadas-paginas` y `board` sale 0 con `plataforma=error`. Probado en `fallos_tipados`.
- Eleccion de rol: `Revisor` rechazado en `prs_abiertos`; `board` usa persona si hay token y TTY, si no agente, sin reintentar con otro rol. Es lectura; la falta de comprobar la lista de personas ya esta como MENOR en la autorrevision.
- `status` nunca construye adaptador: `orden_status` no toca `e.conectar`, y `lib.rs` solo construye el cierre, no el adaptador; probado en `status_no_construye_adaptador_y_board_aguanta_los_fallos`.
- Titulos y logins: `sanear` (control, bidi, anchura cero, espacios juntados, tope con `…`) se aplica a titulo, quien en texto y en `quien=` del JSON, y a la linea de `status`.
- Sin `unwrap`, `expect`, `panic!`, `unreachable!` ni `unsafe` en `tablero.rs` ni en los `prs_abiertos`; los `unwrap_or_default` son seguros.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/plataforma/conexion.rs` y `credenciales.rs`: para comprobar paginacion, tope y el TTY del token de persona.
- `crates/tc2/src/situacion.rs:131` ya estaba en la lista; `tarea/modelo.rs` (`id_valido`): para saber si los IDs impresos estan validados.
