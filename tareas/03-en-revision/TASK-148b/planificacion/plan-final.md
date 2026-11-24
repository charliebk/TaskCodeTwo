# Plan — TASK-148b: escenario ssh, git de sistema y operacion de prueba (A8-01, A8-16, A3-06, A4-09, A8-20 integracion) (parte b de TASK-148)

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 120 min

Señales: unos 27 ficheros (5 de `tests/comun`: `mod.rs`, `lanzador.rs`, `git.rs` nuevo, `remoto.rs` nuevo, `ops.rs`; `agente_git/mod.rs`, `plataforma_gitlab_e2e.rs`, `soporte.rs`; los 14 de la lista; y `verificar.rs`, `revision.rs`, `roadmap.rs`, `actualizar_cli.rs`, `git_binario.rs`, que el criterio o la Regla 4D obligan a tocar). Una capa (tests de integración, sin `src`). 4 criterios, 5 IDs. El plan padre estimaba 90 min; la diferencia es el alcance 4D de la decisión 2 (sin él, ~95).

**Forma.** Precedente de la parte a: un submódulo por concepto en `tests/comun`, reexportado en `comun/mod.rs`. Dependencias en una dirección: `git.rs` es la base; `mod.rs` (`Fixture`) usa `git.rs`; `remoto.rs` usa `Fixture`, `lanzador` y `ops`.

**`comun/git.rs` (nuevo, A8-20)**: `pub fn git_en(dir: &Path, entorno: &[(OsString, OsString)], args: &[&str]) -> String`. Fija `GIT_CONFIG_NOSYSTEM=1`, `LC_ALL=C`, `GIT_TERMINAL_PROMPT=0`; `panic!` si `entorno` no trae `GIT_CONFIG_GLOBAL`; si git falla, el assert lleva argumentos, carpeta y `stderr`. `Fixture::crudo` pasa a `git_en(dir, &self.entorno(), args)`; `agente_git::Clon::git` a `git_en(dir, &self.env, args).trim().to_owned()` (su `env` ya trae `aislado`, `agente_git/mod.rs:78-83`). `plataforma_gitlab_e2e.rs` declara `mod comun;`.

**`comun/lanzador.rs` (se extiende)**: `texto` (12 copias), `valor(linea, clave)` única, primera línea con `trim_end` (decisión 1); `salida(&Output) -> (Option<i32>, String)` (contrato de `ciclo.rs:77` = `liberar.rs:78`, también `toma::start` y el `tc2` de `plan.rs:63`); `lanzar(dir, entorno, args) -> Output` (6 copias: aprobar, ciclo, inyeccion, liberar, plan, toma); `volcado(&Output)` (código, stdout y stderr, de `verificar.rs:209`), que sustituye a `inyeccion::salida` y a `verificar::texto`.

**`comun/ops.rs` (se extiende)**: `pub struct Escribir { pub ruta: String, pub cambiar: Box<dyn Fn(Option<String>) -> String> }` con la forma de `toma.rs:183` (cubre ciclo, plan, solape y liberar con el «envejecer»); `sin_catalogo` junto a `catalogo`.

**`comun/remoto.rs` (nuevo, A8-01 y A8-16)**: constantes `ORIGIN`, `PERSONAS` (ana y beto) y `CONFIG_AUTOMATICO`; `Fixture::entorno_ssh()`, `Fixture::entorno_sin_red()`, `Fixture::git_ssh(dir, args)`; `cuenta(id, login)` e `identificar(dir, id, login)`; `Fixture::clon_github(bare, nombre)` (absorbe `clon` de estimacion y `clon_ssh` de frescura); `Fixture::preparar(&Preparar) -> (bare, a)` con `Preparar { personas, config: Option<&str>, altas: &[&[&str]] }` y `Default`; `Fixture::cambiar_estado(dir, ruta, cambiar)`, `editar(f, dir, id, de, a)` y `con_fila(antes, despues, fase, por)`; `worktree(dir, id)`, `Fixture::punta(bare)` y `Fixture::publicado(bare, ruta)` (decisión 2c); `como_persona(dir, entorno, cuenta, op)` (decisión 2f).

**Comprobaciones del plan padre (hechas):** los siete `preparar` caben en `Preparar` (solape cambia `PERSONAS`, liberar la config; lo propio va detrás de la llamada; frescura `con_tarea` no es este escenario y se queda local). El mensaje de commit no importa (los `log` solo miran la rama de estado; estimacion agrupa por ID). `CONFIG` igual en aprobar, plan e inyeccion.

**Renombres que obliga el criterio 1:** `revision.rs:154` `preparar` → `empezada`; `verificar` usa `comun::texto` y `comun::volcado`; el `preparar` local de aprobar → `planificada`.

**Orden de construcción** (un commit por ID): 1) A8-20: `git.rs`, `crudo` y `Clon::git`, `mod comun;` en el E2E de GitLab, tests de aislamiento. 2) A3-06: `texto`, `valor`, `volcado`; migrar copias (incluidas `verificar::clave` y `roadmap::lista`); `como_persona`; test de `valor`. 3) A8-01: `remoto.rs` con entornos, `git_ssh`, `cuenta`, `identificar`, `clon_github`; `lanzar`; migrar los 10 ficheros con `entorno_ssh`. 4) A8-16 + A4-09: `Escribir`, `sin_catalogo`, `salida`, `con_fila`, `Preparar`, `cambiar_estado`, `editar`, `worktree`, `punta`, `publicado`; migrar toma, liberar, ciclo, solape, plan, aprobar, estimacion, progreso, frescura, inyeccion y alta. 5) Cierre: greps del criterio 1, suite completa y `cargo xtask verificar`, en serie.

## Reutilizacion (Regla 4D)

Lo que ya existe y se usa (helpers, funciones, fixtures, tests), buscado en el repo antes de disenar; y lo que se crea nuevo, con el motivo de no reutilizar:

- `comun::Fixture::{entorno, crudo, remoto_y_clon, otro_clon}` (`comun/mod.rs:48-107`): base de `remoto.rs`.
- `comun::orden_tc2` y `BINARIO` (parte a): base de `lanzar`.
- `comun::ops::catalogo`: junto a él van `Escribir` y `sin_catalogo`.
- `Escribir` con `Fn` de `toma.rs:183-202`: se mueve. `escribir_estado` (toma) y `transaccion` (liberar) pasan a `cambiar_estado`.
- `tc2::identidad::{Cache, Cuenta}`, `tc2::toma::instante`: igual que hoy.
- `tests/soporte.rs`: destino de los tests nuevos.

Nuevo, con motivo: `git.rs` (no hay git de sistema común que aísle; `agente_git` usa otro gitconfig); `remoto.rs` (10 copias del escenario, ninguna común); `volcado` (unifica dos volcados que el criterio obliga a renombrar). No se reutiliza `Informe::valor` (`src/contrato.rs:136`): trabaja sobre un `Informe`, no sobre una línea de texto.

Duplicados que se quedan: `Poner` de estimacion/progreso (otra operación, A6-03, fuera de esta parte); `entorno_denegado` de frescura (una sola copia); `start` de solape (devuelve el stderr de un éxito, no puede usar `salida`).

## Lo que el rol no cubrio

- Modos de fallo (riesgos, no lanzado): si algún test de frescura busca una clave fuera de la primera línea (hoy `split_whitespace` sobre todo el texto); si `verificar::clave` depende de lo mismo; los E2E `#[ignore]` son los únicos que ejercitan `Clon::git` con red.
- Estrategia de pruebas (testing, no lanzado): no se ha comprobado que cada migración conserve la fuerza de sus asserts (p. ej. el `assert_ne!` de `Terminar` en alta).
- Reglas de negocio (dominio, no lanzado): que la línea de contrato sea siempre la primera de stdout se toma de F1-4; no comprobado en `next` y `roadmap --comparar`.
- Coste de compilar un `comun` más grande en ~45 binarios: no medido.

## Riesgos aceptados y que los contiene

- `valor` pasa a leer solo la primera línea: lo contiene la suite completa; si un test se pone en rojo, ese test pasa la línea explícitamente, sin relajar el ayudante.
- `git_en` hace `panic!` sin `GIT_CONFIG_GLOBAL`: los dos llamadores ya lo pasan; lo contienen el `should_panic` y la suite.
- `Clon::git` solo compila en la suite (tests `#[ignore]`): lo contienen el compilador y la decisión 5.
- Las partes c-f comparten ficheros con esta: se hacen en serie en la misma rama.
- Recuento esperado: el de antes + 3 tests nuevos en `soporte.rs` + 1 copia del test de `http_mock` (por `mod comun;` en `plataforma_gitlab_e2e.rs`), justificado en `## Resultado`.

### Suposiciones no verificadas
- Que `next` y `roadmap --comparar` traigan sus claves en la primera línea (`frescura.rs:291`, `:638`).
- Que `#![allow(dead_code)]` de `comun/mod.rs` cubra `remoto.rs` en los binarios que no lo usan (como ya pasa con `lanzador.rs`).

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- C1 (0, verificación): `git grep -nE "fn (entorno_ssh|entorno_sin_red|identificar|otro_clon|editar|sin_catalogo|salida|con_fila|preparar)\b|struct Escribir\b" -- crates/tc2/tests ':!crates/tc2/tests/comun'` vacío.
- C2 (1, ligero): `soporte.rs::valor_recorta_y_lee_la_linea_de_contrato`: `valor("k=v\n","k")==Some("v")`, solo la primera línea (`"a k=1\nb k=2\n"` → `"1"`), sin confundir prefijos (`"x kk=1 k=2"` → `"2"`). Además `git grep -n "fn valor" -- crates/tc2/tests` solo en `comun/lanzador.rs`.
- C3 (2): `soporte.rs::git_en_aisla_la_configuracion_del_usuario` (medio, lanza git: `git config --global user.email` da `prueba@example.com`) y un `#[should_panic(expected = "GIT_CONFIG_GLOBAL")]` (ligero). Además ningún `Command::new("git")` en `agente_git/mod.rs` ni en `comun/mod.rs`.
- C4 (pesados): `cargo test -p tc2 --no-fail-fast` antes y después con el recuento en `## Resultado`, y `cargo xtask verificar`, en serie.

## Lo que necesita decision de una persona

1. `valor` única: primera línea con `trim_end`. Recomendado.
2. Alcance 4D más allá del criterio: a) `lanzar` a `lanzador.rs`; b) `texto` en los 12 ficheros; c) `worktree`, `punta`, `publicado` a `remoto.rs` (A4-09 los nombra; cerrarlo sin ellos sería Disfraz); d) `verificar::clave` y `roadmap::lista` sobre `comun::valor`; e) `Terminar` de alta como `Escribir`; f) `como_persona` fundido en `remoto.rs` y usado por aprobar y liberar (se aparta del texto «en `aprobar.rs`»: dejarlo sería duplicación conocida). Recomendado: sí a todo.
3. Renombres que obliga el criterio 1 (`empezada`, `volcado`, `planificada`). Recomendado.
4. `git_en` con `panic!` sin `GIT_CONFIG_GLOBAL`. Recomendado.
5. Lanzar una vez un E2E real de `plataforma_e2e` (ignorado) para ejercitar `Clon::git` sobre `git_en`, sin coste. Recomendado: un solo test al final; sin red, se acepta compilado y se anota.

**Respuestas de la persona (Carlos, 2026-11-23, consigna de modo automático con la opción recomendada):** se aplican las recomendaciones 1 a 5.

**Regla de una hora:** la estimación (120 min) pasa de 60, pero no se trocea: Carlos descartó trocear más las partes de TASK-148 (consigna del 2026-11-23); la tarea ya es una parte de un troceo.
