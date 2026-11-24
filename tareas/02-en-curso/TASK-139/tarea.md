---
id: TASK-139
titulo: "Auditoria 4D: A1 plataforma e identidad (11 IMPORTANTE)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-curso
plan_aprobado: true
rama: fix/task-139-auditoria-4d-a1-plataforma-e-identidad-1
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 2055168
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-23
actualizado: 2026-11-24
dependencias: []
---
## Objetivo

Corregir los 11 casos IMPORTANTE del grupo A1 de `docs/auditoria-4d.md` (commit auditado d82ea9d): 6 Duplicaciones (una `divergente`) y 5 Disfraces en `crates/tc2/src/plataforma/` y sus tests. Cada caso tiene su ficha con evidencia literal y, para los D4 de test, el diff del mutante que hoy sobrevive. Los MENOR del grupo (A1-12) no se piden. No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.

## Criterios de aceptacion
- [x] A1-02 (`divergente`, va primero): la lógica neutral de `abrir-pr` de `plataforma/github.rs:434` ~ `plataforma/gitlab.rs:561` queda en una sola función, y la carrera sin PR que releer da la misma línea byte a byte en los dos adaptadores (RR-15). Lo prueba un test nuevo con el mock para las dos plataformas.
- [x] A1-01 y A1-03: la lógica neutral de `leer_puertas_de` (`github.rs:742` ~ `gitlab.rs:864`) y las reglas de `prs-abiertos` (`github.rs:412` ~ `gitlab.rs:538`) viven una sola vez fuera de los adaptadores. Cada adaptador conserva solo la lectura de su API (F1-13).
- [x] A1-04 y A1-05: el bucle de la tabla de errores HTTP y el servido de fixtures (`tests/plataforma.rs:468` ~ `tests/plataforma_gitlab.rs:476`) y `adaptador` ~ `con_personas` (`tests/plataforma.rs:32` ~ `:568`) quedan en un ayudante común de los tests de plataforma.
- [x] A1-06: el cliente HTTP del arnés E2E (`tests/plataforma_e2e.rs:74` ~ `tests/plataforma_gitlab_e2e.rs:83`) queda en un solo módulo compartido por los dos E2E. Los dos E2E siguen compilando con `cargo test -p tc2 --test plataforma_e2e --no-run` y `--test plataforma_gitlab_e2e --no-run`.
- [x] A1-07: `rulesets` (`github.rs:960`) pagina con `leer_paginas`, o se funde con `rulesets_de`, y conserva el `None` del 403 de plan. Lo prueba un test nuevo con 100 rulesets y `Link rel=next` cuya página 2 trae `taskcodetwo-develop`: `leer_puertas` lo devuelve.
- [x] A1-08 y A1-11: con el diff del mutante de cada ficha aplicado, el test nombrado sale en rojo. A1-08 es la regla de host que empieza por `.` o `-` en `http.rs:100`, con `cargo test -p tc2 --lib plataforma::http::tests::bases_validas_e_invalidas`. A1-11 es el 403 con `retry-after` y sin `x-ratelimit-remaining` de `errores.rs:57`, con `cargo test -p tc2 --test plataforma errores_dirigidos_a_la_persona`. Hoy los dos salen en verde.
- [x] A1-09 y A1-10: con los mutantes de las fichas aplicados, los tests nombrados salen en rojo. A1-09 son las reglas de GitLab de `gitlab.rs:546`, con `cargo test -p tc2 --test prs_abiertos`. A1-10 es la comprobación de rol de `gitlab.rs:1251` con un token de revisor configurado, con `cargo test -p tc2 --test plataforma_gitlab protecciones_de_gitlab`.
- [x] `cargo xtask verificar` en verde, y el `## Resultado` lista los 11 IDs con su commit.

## Resultado

La logica neutral de `abrir-pr`, `merge-pr`, `prs-abiertos` y `leer-puertas` sale de los adaptadores a `crates/tc2/src/plataforma/operaciones.rs` (`pub(crate)`); cada adaptador solo lee su API (F1-13) y la misma situacion da la misma linea en GitHub y GitLab (RR-15). Los tests de plataforma comparten constructor, tabla de errores y servido de fixtures (`tests/comun/adaptadores.rs`) y los E2E su cliente HTTP (`tests/arnes_e2e/mod.rs`).

| Ficha | Commit |
|---|---|
| A1-09 | 1dc5af3 |
| A1-10 | b5cd0a9 |
| A1-08 | c2c33f4 |
| A1-11 | 84fbaa9 |
| A1-02 (`divergente`) | 82a0514 |
| A1-03 | 1786fca |
| A1-01 | c8a7acf |
| A1-07 | 13a43ba |
| A1-05 | 26ad763 |
| A1-04 | 26b6928 |
| A1-06 | c65d986 |

- Decision 1: la carrera de `abrir-pr` sin PR que releer da `abrir-pr conflicto motivo=pr-existente` (codigo 6) en las dos plataformas; antes GitHub daba `error motivo=validacion` y GitLab `conflicto motivo=mr-existente`. Test `carrera_de_abrir_pr_da_la_misma_linea` (antes del arreglo: `("abrir-pr error motivo=validacion", 1)` frente a `("abrir-pr conflicto motivo=pr-existente", 6)`).
- A1-07: `leer_puertas_pagina_los_rulesets` (antes del arreglo leia 100 de 101); el 403 de plan vuelve a `None` en un solo sitio y uno que ademas es limite da `agotada` (decision 4).
- Mutantes D4 (decision 5), todos en rojo: antes del refactor, el diff literal de A1-09 (tres: sha sin validar, rama sin validar, sha sin minusculas), A1-10 (GitHub y GitLab), A1-08 y A1-11; despues del refactor, el equivalente de A1-09 y A1-10 en `operaciones.rs`.
- Recuento `cargo test -p tc2 --no-fail-fast` (target propio): antes 783 (cierre de TASK-148f); despues 48 binarios, 785 ok, 0 fallos, 13 ignorados (783 + 2). `plataforma_e2e` y `plataforma_gitlab_e2e` compilan (`--no-run`).
- E2E real `plataforma_e2e::recorrido_contra_github` contra `charliebk/taskcodetwo-e2e-github`: ok. El E2E de GitLab solo se ha compilado.
- Fuera de alcance (anotado en el plan): constructores de adaptadores en `tests/doctor.rs`, `identidad.rs` e `identidades.rs`; paginacion de `approval_rules`; MENOR A1-12.
- `cargo xtask verificar` lo ejecuta `finish`.

## Divergencias

- Texto nuevo `PR_EXISTENTE` en el catalogo para el diagnostico de GitHub (precedente `PR_CERRADO`/`MR_CERRADO`).
- `puertas::mergeado` y `pr_cerrado` no se parametrizan por `op` (cambiaria la API publica): variantes `pub(crate)` `mergeado_en` y `pr_cerrado_en`, en las que delegan las publicas.
- El `intentar_con` del E2E de GitLab manda ahora las cabeceras comunes tambien en GET.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-24T13:55:55Z | plan | automatico | persona | 0.17.1 |
| 2026-11-24T14:02:51Z | approve | automatico | persona | 0.17.1 |
| 2026-11-24T14:02:52Z | start | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-24T14:02:50Z | diseno | agente:a27838b01603442e1 | 52 | 117461 | 1918047 | 19608 | 41 | 2055168 | - | - |

## Autorrevision

- rust-wasm-reviewer — c65d986
- 4D — c65d986: helper buscado, criterios citados, 11 mutantes vistos en rojo
