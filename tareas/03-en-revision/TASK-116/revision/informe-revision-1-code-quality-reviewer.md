# Informe de revision — TASK-116 (ronda 1)

- Commit revisado: 92900522ace4f8328546b838b04358cb15a3d298
- Revisor: code-quality-reviewer
- Veredicto: PENDIENTE (escribelo con: taskctl veredicto TASK-116 aprobada | aprobada-con-correcciones | cambios-solicitados)

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | docs/adr/README.md:11 | implementacion |
| MEN-2 | MENOR | abierto | docs/adr/F1-12-actualizacion-entre-versiones.md:551 | implementacion |
| MEN-3 | MENOR | abierto | crates/tc2/tests/convivencia_016.rs:118 | implementacion |
| MEN-4 | MENOR | abierto | crates/tc2/src/git/puerto.rs:514 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Sin CRITICOS ni IMPORTANTES. Alcance: pase generico completo (el diff es Rust + docs;
la parte Rust la cubre tambien rust-wasm-reviewer).

### Reproduccion y verificacion

Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-116-r1-calidad`, rama de la tarea
(HEAD 0097ddc; el commit revisado 9290052 es su padre y 0097ddc solo anade la peticion).

- `cargo test -p tc2`: primera pasada con 1 fallo (`plataforma.rs::protecciones_y_bordes`,
  panic en `plataforma.rs:781`, `firma_commit(...).unwrap()` contra el mock HTTP en 127.0.0.1;
  fichero no tocado por el diff). Repetido solo: 4/4 en verde. Repetida la suite entera con
  `--no-fail-fast`: 22 binarios `test result: ok`, 0 fallos. Fallo transitorio por la red local,
  no por el cambio.
- Grep de nombres viejos (`taskcode/tareas`, `taskcode-estado`, `refs/taskcode/`, `.git/tc2`,
  `.taskcode/worktrees`) en todo el repo salvo `tareas/` y ADR F1-*: solo `puerto.rs:514`
  (a proposito, caso de rechazo) y `docs/adr/README.md:11`.
- C1: `alta.rs::la_rama_de_estado_es_la_de_la_2_0`, `huella::tests::todo_sale_del_mismo_nombre` y los
  literales de `estado_casos.rs` existen y pasan. Mutacion `RAMA_ESTADO = "taskcode/tareas"`:
  rojos `todo_sale_del_mismo_nombre` y `la_rama_de_estado_es_la_de_la_2_0`.
- C2: `cerrojo::tests::el_cerrojo_vive_en_la_carpeta_propia`, `identidad::tests::la_cache_vive_en_la_carpeta_propia`,
  `huella::tests::dir_propio_crea_la_carpeta`. Mutacion `CARPETA_COMUN = "tc2"`: 4 tests rojos
  (los tres citados y `todo_sale_del_mismo_nombre`). Grep de C2 vacio en `crates/tc2/src`.
- C3: `huella::tests::worktrees_fuera_de_las_rutas_de_la_016` pasa. Mutacion
  `CARPETA_WORKTREES = ".taskcode/worktrees"`: rojo `convivencia_016` (linea 164, huella 0.16 cambiada).
  Ojo: el test unitario C3 pasa asercion literal; en la mutacion lo atrapa el de convivencia y el literal `assert_eq!(w, ".taskcodetwo/worktrees")`.
- C4: `convivencia_016::la_huella_de_la_016_queda_intacta_tras_new_y_start` pasa (7 s) y usa el binario
  `tc2 new` real. Mutacion `CARPETA_COMUN = "taskcode"` (la 2.0 escribiendo en `.git/taskcode/`):
  rojo en `convivencia_016.rs:164`. Se cae donde debe (huella de la 0.16 distinta).
- C5: `huella::tests::refs_propias_bajo_su_prefijo` pasa. Mutacion de `PREFIJO_REFS`/`REFS_PENDIENTES` a
  `refs/taskcode/`: el de convivencia sigue verde (ver MEN-3); lo atrapan los literales de huella.
- C6: `cargo xtask verificar --perfil rapida` salio rc=1 por un test ajeno al diff: `plataforma_gitlab::envoltorio_y_errores_de_gitlab` devolvio `abrir-pr sinred` en vez de `abrir-pr rechazada motivo=sin-commits` (mock en 127.0.0.1 agotado bajo carga; `plataforma_gitlab` entero repetido fallo igual, y ese test aislado pasa 3/3 en 0.4 s; la suite entera con `--no-fail-fast` paso 22/22 binarios). `cargo fmt --check` y `cargo clippy -p tc2 --all-targets -D warnings` limpios. C6 queda sin comprobar de principio a fin: conviene repetir la verificacion rapida con la maquina descargada.

Comprobaciones de solo lectura que afirma el implementador:
- Cache del plugin (`~/.claude/plugins/cache/taskcode-marketplace/`, versiones 0.11 a 0.16): solo `.git/taskcode/`
  (`puertas.log`, `gitflow/`, `progreso.json`, `progreso-contador`; `cadena.lock`, `freno-ultimo-verde.json`
  y `doctor-deploy.yml` aparecen en el codigo). Cero coincidencias de `refs/taskcode` y de `taskcodetwo`. Confirmado.
- OpenDockTrading y ToolsaWeb: sin `.git/taskcodetwo`, sin `.git/tc2`, sin `taskcode-estado.lock`, sin refs
  `taskcode*` ni rama `taskcode/...`, sin `.taskcodetwo`; solo `.git/taskcode/` y `.taskcode/config.yml` de la 0.16
  (ToolsaWeb tiene una rama `feature/task-403-...-taskcode-0-16-freno`, que no es de la 2.0). Confirmado.
  No se modifico nada en ellos.

### MEN-1 — indice de ADR con el nombre viejo de la rama
`docs/adr/README.md:11`: la fila de F1-1 sigue titulada «rama de estado `taskcode/tareas`». Es el unico
sitio fuera de los ADR aceptados con el nombre viejo y el indice es lo primero que se lee. Reproduccion:
`grep -n "taskcode/tareas" docs/adr/README.md`. Corregir con `taskcodetwo/tareas` o con una nota «(ahora `taskcodetwo/tareas`, TASK-116)».

### MEN-2 — la adenda dice cubrir F1-16 y F1-1 sin enlazarlas
`F1-12:551` afirma ser la lectura valida de «F1-2, F1-3, F1-4 y F1-16», pero solo F1-2, F1-3 y F1-4
llevan linea de adenda en la cabecera; F1-16 (usa `.taskcode/worktrees` en 3 sitios) y F1-1 (16 menciones de
la rama) no. La nota previa de F1-12 (linea 520) si cubre `taskcode/tareas` en F1-1, pero no los worktrees ni el cerrojo.
Un lector de F1-16 no llega a la tabla. Anadir la linea a F1-16 (y F1-1) o recortar la afirmacion.

### MEN-3 — C4 no ejercita las refs pendientes
`convivencia_016.rs:118`: el test solo recorre `new` y `crear_rama_y_worktree` con red; `refs/taskcodetwo/pendientes`
no se escribe nunca, asi que si `PREFIJO_REFS` volviera a `refs/taskcode/` este test sigue verde (mutacion M5
reproducida). Queda cubierto solo por los literales de `huella::tests`, que no prueban convivencia. Opcional:
anadir un caso sin red (o ampliar con `refs/taskcode/*` sembradas por la 0.16 y comprobar que siguen).

### MEN-4 — comentario de test impreciso (sin consecuencia)
`puerto.rs:513`: «El prefijo viejo ya no es propio» es el caso de rechazo correcto y esta bien probado; solo se anota
que el literal `refs/taskcode/pendientes/` es el unico resto del nombre viejo en `src` y es intencionado.
Sin accion necesaria salvo que se quiera un `// intencionado` para que un grep futuro no lo marque. Se deja abierto por si el equipo prefiere cerrarlo como aceptado.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/huella.rs`, `estado/cerrojo.rs`, `identidad.rs`, `git/estado.rs`, `git/rama.rs`, `tests/convivencia_016.rs`,
  `tests/alta.rs`: para reproducir y mutar C1..C5 (los tests y el codigo que prueban).
- `~/.claude/plugins/cache/taskcode-marketplace/` y los repos OpenDockTrading y ToolsaWeb (solo lectura): lo pedia el encargo.
- `docs/adr/F1-1`, `F1-16` y `docs/adr/README.md`: para MEN-1 y MEN-2.
