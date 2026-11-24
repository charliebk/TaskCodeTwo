# Plan — TASK-148f: dobles de Plataforma con Parcial y Doble (A8-05, absorbe A2-06) (parte f de TASK-148)

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 50 min

Señales: 2 ficheros nuevos (`src/plataforma/prueba.rs`, `tests/comun/plataforma.rs`) y 8 tocados (`plataforma/mod.rs`, `identidad.rs`, `revision/mod.rs`, `tablero.rs`, `tests/revision.rs`, `tests/tablero.rs`, `tests/comun/mod.rs`, `tests/soporte.rs`); dos capas (unitarios e integración); 3 criterios.

**Forma** (la del plan padre, decisiones 4 y 5, contrastada con el trait actual `plataforma/mod.rs:413-530`):
- `src/plataforma/prueba.rs`, declarado `#[cfg(test)] pub(crate) mod prueba;` (precedente `estado::prueba`, `git::prueba`). `pub(crate) trait Parcial` con las firmas de los 8 métodos obligatorios (`prs_abiertos`, `abrir_pr`, `estado_puertas`, `merge_pr`, `leer_puertas`, `leer_permisos_ci`, `firma_commit`, `identidad`) más `entorno_push` y `leer_revision`; todos por defecto `panic!("doble de Plataforma: <op> no se esperaba")` con el nombre de la operación del contrato (`merge-pr`, `prs-abiertos`…), con `#[allow(clippy::panic)]` (precedente `app_github.rs:434`). `pub(crate) struct Doble<P>(pub P)` con `impl<P: Parcial> Plataforma for Doble<P>` que reenvía; `proveedor`, `leer_plan`, `leer_puertas_de`, `preparar_puertas`, `aplicar_puertas` siguen con su defecto. Test `#[should_panic(expected = "merge-pr")]` sobre `Doble(Vacio)`.
- `tests/comun/plataforma.rs`, misma forma con `pub`, registrado en `comun/mod.rs`.
- Migraciones (cada doble implementa `Parcial` solo con lo que usa; se borran `sin_uso` y el relleno): `identidad.rs:631-679` (`Falsa`: solo `identidad`, con su contador); `revision/mod.rs:532-568` (`Puertas`: solo `estado_puertas`, con su `assert_eq!(rol, Agente)`); `tablero.rs:998-1025` (`Falsa`: solo `prs_abiertos`); `tests/tablero.rs:113-140` (`Falsa`: solo `prs_abiertos`, sin los `panic!("board no …")` a mano); `tests/revision.rs:48-130` (el struct local `Doble` pasa a `Apuntes`, desaparece `Compartida`, queda `impl Parcial for Rc<Apuntes>`; las respuestas «sin dato» de sus `Option` se conservan como `Err(Informe::nuevo(.., Resultado::Error))` con `puertas::ESTADO_PUERTAS` y `plataforma::LEER_REVISION`).
- Comprobación estática: ningún test llega hoy a un método de relleno (`review_en`, `revision_ia`, `board` y `quien_ejecuta` solo llaman a lo que sus dobles implementan); el único `Error` del que dependen es el de los `Option` vacíos de `tests/revision.rs`, que se conserva. Lo confirma la pasada completa.

Orden: (1) `plataforma::prueba` con su test y los tres dobles de `src`; (2) `comun/plataforma.rs` con su test en `soporte.rs` y los dos de `tests/`; (3) grep del criterio y pasadas pesadas.

## Reutilizacion (Regla 4D)

Lo que ya existe y se usa (helpers, funciones, fixtures, tests), buscado en el repo antes de disenar; y lo que se crea nuevo, con el motivo de no reutilizar:

- Patrón `#[cfg(test)] pub(crate) mod prueba` de `estado::prueba` y `git::prueba`; `tests/soporte.rs` para el segundo `should_panic`.
- Constantes públicas `plataforma::LEER_REVISION` (`mod.rs:278`) y `puertas::ESTADO_PUERTAS` (`puertas.rs:10`).
- Defectos del trait (`no_soportada`, `leer_puertas_de`) tal cual.
- Nuevo: `Parcial` y `Doble<P>` en dos copias (frontera de crate y regla de huérfanos E0210): duplicado aceptado, decisión 5 del padre.
- No se reutilizan `MERGE_PR` y hermanas (`github.rs:29-35`, `gitlab.rs:29-35`): son privadas; hacerlas `pub` cambiaría la API.

## Lo que el rol no cubrio

- Riesgos (no lanzado): análisis estático; un `?` o `.ok()` que trague un `Err` de relleno en una rama no cubierta lo destapa la pasada completa.
- Testing (no lanzado): no valorado si quitar los `panic!("board no …")` de `tests/tablero.rs` pierde diagnóstico (el mensaje común da la operación).
- Dominio: no aplica.

## Riesgos aceptados y que los contiene

- Método inesperado: de `Error` silencioso a `panic!`. Si un test se pone en rojo, se añade el método a su doble; nunca se relaja `Parcial`. Lo contiene la suite completa antes y después.
- `Parcial` copia firmas en dos sitios (antes cinco dobles): lo contiene el compilador.
- `entorno_push` y `leer_revision` pasan a `panic!` en los dobles en vez del defecto «para adaptadores de prueba» (ningún doble depende de él); se corrigen esos comentarios (decisión 1).
- API pública sin cambios: todo es `#[cfg(test)] pub(crate)` o vive en `tests/`.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- Criterio 1 (0): `git grep -n 'impl Plataforma for' -- crates/tc2` solo `github.rs`, `gitlab.rs`, `plataforma/prueba.rs` y `tests/comun/plataforma.rs`; `git grep -n sin_uso -- crates/tc2` vacío.
- Criterio 2 (2, ligeros): `#[should_panic(expected = "merge-pr")]` sobre `Doble(Vacio)` llamando a `merge_pr(1, "a")`, en `plataforma::prueba` y en `tests/soporte.rs`.
- Criterio 3 (pesados): `cargo test -p tc2 --no-fail-fast` (recuento antes + 2) y `cargo xtask verificar`, sin solapar.

## Lo que necesita decision de una persona

1. Quitar «adaptadores de prueba» de los comentarios de `plataforma/mod.rs:428` y `:446` en este commit (solo documentación). Recomendado.
2. Nombre de la operación en el `panic!` como literal (las constantes son privadas y duplicadas entre adaptadores; unificarlas es otro D1, fuera de esta parte, anotado como hallazgo nuevo). Recomendado.
3. `impl Parcial for Rc<Apuntes>` en `tests/revision.rs`, sin `Compartida`. Recomendado.
4. Respuestas «sin dato» de los `Option` de `tests/revision.rs` como `Error`. Recomendado.

**Respuestas de la persona (Carlos, consigna del 2026-11-23: modo automatico con la opcion recomendada):** se aplican las recomendaciones 1 a 4.
