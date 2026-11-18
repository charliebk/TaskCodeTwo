# Informe de revision — TASK-128 (ronda 1)

- Commit revisado: 6fdb49100d3066da1c76a255b171cb8ed232d052
- Revisor: code-quality-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/progreso/mod.rs:266 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### MEN-1 — Ningun test se pone rojo si `progreso=sin-anterior` deja de salir por `refrescar` o por `start`

- **Donde**: `crates/tc2/src/progreso/mod.rs:266` (mapeo `Anterior::NoEscrito => Refresco::EscritoSinAnterior`) y `crates/tc2/src/toma/mod.rs:235` (rama de `start`).
- **Que pasa**: el comportamiento nuevo (la linea lleva `progreso=sin-anterior`) solo se ejercita de punta a punta en dos sitios: `fichero::escribir` (unitario, devuelve `NoEscrito`) y `lib.rs::con_progreso` (unitario, con un `Refresco` fabricado). Los tramos intermedios (`refrescar` y la rama de `start` en la toma) no tienen ningun test, y el E2E `progreso::c3` acepta `rota || dice`, de modo que nunca fuerza un fallo de rotacion.
- **Reproduccion**: en el clon, cambiar en `progreso/mod.rs:266` `Refresco::EscritoSinAnterior` por `Refresco::Escrito` y correr `cargo test -p tc2 --lib -- progreso`: 25 passed, 0 failed. Es alcanzable con git real (basta una carpeta en la ruta `TASK-NNN.anterior.json`, como hace el test unitario de `fichero.rs`), asi que se podria cubrir con un E2E ligero. La mutacion equivalente en `toma/mod.rs:235` no se ejecuto (los E2E de la toma tardan minutos y ningun test del repo contiene `sin-anterior` fuera de `fichero.rs`, `lib.rs` y el helper `rotado`; comprobado con grep).
- **Impacto**: una regresion que vuelva a callar la rotacion fallida (el defecto F3-12) pasaria la suite en `pausa`/`reanudar`/`start`. No hay fallo hoy.
- **Sugerencia**: un test de `refrescar` (o E2E de `pausa`) con el directorio ocupando `.anterior.json` que aseveren `progreso=sin-anterior` y que stderr siga vacio.

## Resultado de la reproduccion

- Suite: `cargo xtask verificar --perfil rapida` en verde (exit=0) sobre 78f965e, codigo identico al revisado (`git diff --stat 78f965e 6fdb491 -- crates docs` vacio). Log: `%LOCALAPPDATA%/pruebas-claude/TaskCodeTwo/verif-128.log`, ultima linea «xtask: verificacion Rapida en verde». No se repitio.
- Clon en `%LOCALAPPDATA%/pruebas-claude/TaskCodeTwo/task-128-r1-calidad` en 6fdb491. Ejecutado ademas:
  - `--lib -- solape progreso::fichero`: 14 de 14.
  - `--test solape -- --test-threads=4`: 15 de 15 (339 s; incluye `misma_persona_no_se_avisa_contra_si_misma` y `c10`).
  - `--test liberar c5_c6_c10`: 1 de 1 (stderr vacio exigido).
  - `--test progreso c3_transiciones` tres veces seguidas con `--test-threads=4`: 3 de 3 (no hubo intermitencia; no se vio si alguna salio con `sin-anterior`).
- Mutaciones (se revirtieron con `git checkout`):
  - Quitar el filtro `misma_persona` de `otras`: falla `solape::tests::otras_quita_la_propia_y_las_de_la_misma_persona`. La mutacion contra `liberar c5_c6_c10` no se pudo ejecutar: el enlazado fallo con «Acceso denegado» sobre `tc2.exe` (proceso del otro revisor o residuo); no verificado por E2E.
  - Volver a re-serializar el progreso leido en `escribir`: falla `el_anterior_es_el_fichero_que_habia_byte_a_byte`. El test discrimina la causa real de C3.
  - `NoEscrito` mapeado a `Escrito` en `refrescar`: ningun test falla (MEN-1).
- Tests con expectativa cambiada:
  - `solape::c10` pasa a k=1: correcto, TASK-002 es del propio luis; la aserciones de 1+k+1 procesos, de `publicadas` y de `en-curso-modulos` siguen siendo estrictas.
  - `liberar::c5_c6_c10`: ahora exige stderr vacio y ausencia de claves `solape*` y `en-curso-modulos` en las dos tareas de beto; asevera algo real, no esconde regresion.
  - `progreso::c3` con `rotado`: acepta rotado o `sin-anterior`, pero exige exactamente uno de los dos y, si rota, que sean los bytes previos. Es mas debil que antes solo cuando dice `sin-anterior` (ahi no comprueba nada del anterior), lo cual esta cubierto por los unitarios de `fichero.rs`; no esconde regresion de la rotacion normal.
- Adenda del ADR F1-17: coincide con el codigo (filtro por `misma_persona` en `otras`, sin asignar y heredadas cuentan, `progreso=sin-anterior` sin stderr, copia de bytes si son un progreso valido, textos en el catalogo de `toma`). La cifra «cerca del 13 %» de valores `f64` que no hacen ida y vuelta y el «9 de 10 antes» no se pudieron comprobar aqui (no verificado); el test unitario con `0.009_027_842_777_777_777` si demuestra el mecanismo.
- Observacion no clasificada: un progreso actual a medias (tras un corte) ahora marca `sin-anterior` aunque el `.anterior.json` anterior siga intacto; es coherente con la doc de `Anterior::NoEscrito` y sucede solo tras un corte.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/**` y `crates/tc2/tests/**` fuera de los diffs (`tarea/modelo.rs` `misma_persona`, `identidad.rs` `asignado`, `lib.rs`): para comprobar la semantica de `misma_persona` y el origen de `propio`.
- `diff` de crates y tests de la tarea (no listado en el contexto permitido de dominio ADR): necesario para revisar el codigo, no solo la adenda.
- `%LOCALAPPDATA%/pruebas-claude/TaskCodeTwo/verif-128.log`: para citar la suite en verde.
