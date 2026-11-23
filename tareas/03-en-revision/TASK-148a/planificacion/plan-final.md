# Plan — TASK-148: Auditoria 4D: soporte de tests compartido en tests/comun y modulos prueba (9 IMPORTANTE)

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 60 min (parte a del troceo; la tarea entera estimaba 330 min)

Señales: unos 38 ficheros de `crates/tc2/tests` y 10 de `crates/tc2/src` tocados; dos capas (integración en `tests/comun`, unitarios en `#[cfg(test)] pub(crate) mod prueba`); 8 criterios y 9 IDs; 26 ficheros con lanzador propio (`git grep CARGO_BIN_EXE_tc2`: 38 apariciones en 26 ficheros). Pasa de 60 min: ver `## Troceo`.

**Forma.** `tests/comun` deja de ser un solo `mod.rs` y pasa a tener submódulos por concepto (precedente: `comun/http_mock.rs` y `comun/ops.rs`, ya `pub mod` en `comun/mod.rs:9-10`).
- `comun/mod.rs`: `Fixture` y `escribir`, como ahora. `crudo` delega en el git de sistema común.
- `comun/lanzador.rs` (nuevo):
  - `pub const BINARIO: &str = env!("CARGO_BIN_EXE_tc2")`.
  - `pub fn tc2(dir: &Path) -> Command`: `Command` ya configurado con `LC_ALL=C`, `GIT_TERMINAL_PROMPT=0` y `TC2_IDIOMA=es` (con `tc2::i18n::VARIABLE`), y quita los tres tokens con `tc2::plataforma::Rol::{Agente,Revisor,Persona}.variable()`.
  - Devuelve `Command` y no `Output` a propósito: las variantes reales necesitan `spawn` (`progreso.rs:507`), `stdin` (`verificar.rs:150`, `progreso.rs:746`), `CLAUDE_CONFIG_DIR` (`recursos.rs`) o `GIT_CEILING_DIRECTORIES` (`doctor.rs`). Cada test añade lo suyo con `.env(..)` después: gana la última operación por clave.
  - `texto`, `valor` (una sola, con `trim_end`) y `salida` (contrato de una línea, la de `ciclo.rs:81`).
  - `actualizar_cli.rs` y `cli.rs` usan `BINARIO` para copiar o leer el ejecutable. Así `CARGO_BIN_EXE_tc2` queda solo en `comun/`.
- `comun/git.rs` (nuevo): `pub fn git_en(dir, entorno, args) -> String`, el git de sistema para preparar escenarios.
  - `LC_ALL=C` y `GIT_TERMINAL_PROMPT=0`; el assert lleva el `stderr` y la carpeta.
  - Lo usan `Fixture::crudo` (con `self.entorno()`) y `agente_git::Clon::git` (conserva su `trim()` en la llamada).
  - `agente_git` pasa a usar `crate::comun`: `plataforma_e2e.rs` y `plataforma_gitlab_e2e.rs` declaran `mod comun;`.
- `comun/remoto.rs` (nuevo): el escenario «remoto bare con ssh falso», como `impl Fixture` en el submódulo.
  - `ORIGIN`, `Fixture::entorno_ssh()`, `Fixture::entorno_sin_red()`, `identificar(dir, id, login)`.
  - `Fixture::clon_github(bare, nombre)`: el `otro_clon` local con `remote set-url origin ORIGIN`, distinto de `Fixture::otro_clon`.
  - `Fixture::preparar(&Preparar { personas, config: Option<&str>, altas: &[&[&str]] }) -> (bare, clon_a)`.
  - `con_fila(antes, despues, fase, por)` y `editar(f, dir, id, de, a)`.
- `comun/ops.rs` (se extiende):
  - `Escribir { ruta, cambiar: Box<dyn Fn(Option<String>) -> String> }`; el `Fn` cubre las cinco variantes, incluido el «envejecer» de `liberar.rs:244-252`.
  - `sin_catalogo`, junto a `catalogo`.
- `comun/instantanea.rs` (nuevo):
  - `instantanea(dir, excluir: &[&str]) -> BTreeMap<String, String>`: la forma más fuerte de las seis (`desinstalar.rs:37-62`): recursiva, anota carpetas (`"dir"`) y guarda `"<len> <sha256>"` con `tc2::distribucion::sha256::hex`.
  - Las tres `foto` (tablero, frescura, doctor) conservan su parte de git, que difiere a propósito (frescura filtra la ref remota de estado); su recorrido de `.git/taskcodetwo` llama a `instantanea(.., &[FICHERO_SINCRONIZACION])`.
- `comun/plataforma.rs` (nuevo):
  - `trait Parcial` con las firmas de los métodos obligatorios de `Plataforma`, más `entorno_push` y `leer_revision` (los sobrescribe el doble de `tests/revision.rs`). Todos por defecto con `panic!("doble de Plataforma: <op> no se esperaba")`.
  - `struct Doble<P>(pub P)` con `impl<P: Parcial> Plataforma for Doble<P>`, que reenvía.
  - Los métodos con defecto del trait (`proveedor`, `leer_plan`, `leer_puertas_de`, `preparar_puertas`, `aplicar_puertas`) no se reenvían: se quedan con el defecto documentado «para adaptadores de prueba» (`plataforma/mod.rs:428`, `:446`).

**Unitarios de `src`, tres módulos `#[cfg(test)] pub(crate) mod prueba` según hacia dónde van las dependencias:**
- `estado::prueba`: `EnMemoria` (se mueve desde `roadmap::tests`, `roadmap/mod.rs:579`), `estado(&[(&str, String)])` y `motivo(&Result<..>)`. Lo usan `ciclo/operacion.rs`, `ciclo/liberar.rs`, `toma/operacion.rs`, `plan/operacion.rs`, `plan/aprobar.rs`, `plan/guardar.rs`, `revision/operacion.rs`, `roadmap` y `siguiente` (borra su copia, `siguiente.rs:140`). Motivo: `EnMemoria` implementa `estado::Estado`; hoy 8 módulos dependen de `roadmap::tests` para algo del estado.
- `git::prueba`: `CONFIG`, `entorno(raiz)` y `sh(raiz, dir, args)`, sacados de `ciclo/mod.rs:446-485`. Lo usan `ciclo`, `estado/montaje.rs` (deja sus `-c`) y el `git init` de `git/estado.rs:554-566`.
- `plataforma::prueba`: `Parcial` y `Doble<P>`, con la misma forma que en integración. Los dobles `Falsa` (identidad, tablero) y `Puertas` (revision) implementan `Parcial` y solo escriben el método que usan.

**Lo que se queda en su fichero:** `como_persona` y `hito_como_persona` de `aprobar.rs` se funden en una función local que recibe la llamada final como `Fn`; no sale de `aprobar.rs` porque no se repite fuera. El `como_persona` de `liberar.rs:294` es otra cosa y no se toca.

## Reutilizacion (Regla 4D)

Lo que ya existe y se usa (helpers, funciones, fixtures, tests), buscado en el repo antes de disenar; y lo que se crea nuevo, con el motivo de no reutilizar:

- `comun::Fixture::{entorno, crudo, otro_clon, remoto_y_clon}` (`tests/comun/mod.rs:44-103`): base de `remoto.rs` y de `git_en`.
- `comun/ops.rs`: destino de `Escribir` y `sin_catalogo` (ya tiene `catalogo` y operaciones de prueba).
- `comun::http_mock::{Mock, Simulada, serie}`: sustituye a los dos `Servidor` (`identidad.rs:33-98`, `init.rs:31-105`). Ya responde por `METODO ruta` con o sin consulta, lee el cuerpo, no manda FIN (`http_mock.rs:157-192`) y registra en `recibidas()`, que cubre `peticiones()`. Sin `Mock::con_rutas_get`: los tests llaman a `responder("GET", ruta, Simulada::json(200, ..))`.
- `tc2::plataforma::Rol::variable()` (`plataforma/credenciales.rs:41`) y `tc2::i18n::VARIABLE` (`i18n/mod.rs:34`): nombres de variables sin escribirlos a mano.
- `tc2::distribucion::sha256::hex`: contenido de la instantánea (ya lo usa `desinstalar.rs:54`).
- `roadmap::tests::EnMemoria` se mueve a `estado::prueba`; `ciclo::tests::{CONFIG, entorno, sh}` se mueven a `git::prueba`. No se copian.

Nuevo, con motivo:
- `comun/lanzador.rs`, `git.rs`, `remoto.rs` e `instantanea.rs`: no hay ayudante común para eso (`comun/mod.rs` no lanza el binario ni recorre carpetas; `Fixture::huella` solo mira refs y status).
- `Parcial` y `Doble<P>`, dos copias, una por crate: los tests de integración no ven los `#[cfg(test)]` de la biblioteca; en el crate de tests la regla de huérfanos (E0210) impide `impl<T: Parcial> Plataforma for T`, de ahí el envoltorio `Doble<P>`. Misma forma en los dos lados.
- `CONFIG` de git en dos sitios (`comun/mod.rs:22` y `git::prueba`) por la misma frontera de crate: duplicado aceptado y nombrado (decisión 5).

## Lo que el rol no cubrio

- **Modos de fallo** (rol de riesgos, no lanzado): tests que dependen sin decirlo del defecto `Resultado::Error` de los dobles y que con `panic!` se pondrán en rojo; tests que prueban la caída a `LANG`/`LC_MESSAGES` y que con `TC2_IDIOMA=es` de base necesitarán `env_remove`; que `montaje.rs` dependa de `init.defaultBranch=main`. No revisados.
- **Estrategia de pruebas** (rol de testing, no lanzado): solo el presupuesto mínimo de abajo; no se ha valorado si cada migración mantiene la fuerza de cada comprobación, salvo la instantánea.
- **Reglas de negocio** (rol de dominio, no lanzado): no se ha mirado si `PERSONAS` y `CONFIG` difieren entre ficheros de forma que importe al `preparar` común.
- **Coste de compilar** un `comun` más grande en unos 44 binarios: no medido; se espera neutro o menor.
- **Criterio del recuento**: «mismo número de tests» choca con los tests nuevos y con el test propio de `http_mock` replicado en cada binario que declara `mod comun;` (decisión 1).

## Riesgos aceptados y que los contiene

- Cambio de comportamiento deliberado en tres puntos; cada test que se ponga en rojo se arregla explicitando lo que el test ya hacía, nunca relajando el ayudante común:
  - Los 13 ficheros sin `env_remove` de tokens y los 6 sin `TC2_IDIOMA` pasan a aislarse.
  - Los dobles pasan de `Error` silencioso a `panic!`.
  - La instantánea de importar pasa a ver carpetas, y la `foto` de frescura, subcarpetas.
  - Lo contiene: `cargo test -p tc2 --no-fail-fast` completo antes y después de cada parte.
- `Parcial` copia las firmas de `Plataforma`: si el trait gana un método obligatorio hay que tocar dos sitios (antes, cinco). Lo contiene el compilador.
- `agente_git` pasa a depender de `comun` y los dos E2E declaran `mod comun;`: de lo específico a lo común, sin ciclo.
- Las partes comparten ficheros (p. ej. `tablero.rs` en a, d y f): sin dependencia lógica, pero se hacen en serie.

### Suposiciones no verificadas
- Que `estado/montaje.rs` no dependa de que la rama inicial sea `main`; si depende, conserva `-c init.defaultBranch=main`.
- Que los siete `preparar` quepan en `Preparar { personas, config, altas }` y que el mensaje de commit no importe (`aprobar.rs:130`, `estimacion.rs:266`, `progreso.rs:79`); el que no quepa llama a `Fixture::preparar` y añade sus pasos detrás.
- Que ningún test de `cli.rs`, `git_binario.rs` o `siguiente.rs` pruebe la caída a `LANG` sin `TC2_IDIOMA` (`cli.rs:13`, `git_binario.rs:15`).
- Que `#![allow(dead_code)]` de `comun/mod.rs:6` cubra los submódulos nuevos; se confirma con `cargo xtask verificar`.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo). Los tests propios de `comun` van en **un solo binario nuevo**, `tests/soporte.rs`, para no multiplicarse por ~40 binarios.

- **A8-02** (1, ligero): `comun::tc2(dir).get_envs()` lleva los tres tokens como `None` y `TC2_IDIOMA=es`. Verificación aparte: `git grep -n 'CARGO_BIN_EXE_tc2' -- crates/tc2/tests` solo en `tests/comun/`.
- **A8-11** (0): sustituye un doble por otro que ya tiene su test (`http_mock.rs:198`); lo cubren `identidad.rs` e `init.rs` en verde. Verificación: `git grep -n 'Servidor::nuevo' -- crates/tc2/tests` vacío.
- **A8-12** (1, ligero; lo exige el criterio): una carpeta vacía nueva y un fichero en una subcarpeta cambian la foto, y un nombre de `excluir` no la cambia.
- **A8-16 y A3-06** (1, ligero): `valor("k=v\n", "k") == Some("v")`, el caso que `alta.rs:393` compensaba a mano.
- **A8-20** (1, medio, lanza git): unitario en `git::prueba`: con `entorno(raiz)`, `git config user.email` devuelve el de `CONFIG` y no el del usuario.
- **A8-05** (2, ligeros): `#[should_panic(expected = "merge-pr")]` sobre un `Doble` vacío, en `plataforma::prueba` y en `soporte.rs`.
- **A8-01 y A8-18** (0): traslados de cuerpos idénticos; los cubren las suites que los usan.
- **Criterio final** (pesados): `cargo test -p tc2 --no-fail-fast` antes y después, con el recuento en `## Resultado`, y `cargo xtask verificar`. Nunca dos a la vez.

## Lo que necesita decision de una persona

1. **Recuento del criterio final.** Propuesta: «igual que antes, más los tests nuevos, más una copia del test de `http_mock` por cada binario que pasa a declarar `mod comun;`» (`soporte.rs`, `actualizar_cli.rs`, `cli.rs`, `plataforma_e2e.rs`, `plataforma_gitlab_e2e.rs`), justificado en `## Resultado`.
2. **`TC2_IDIOMA=es` en la base del lanzador** (hoy 6 ficheros no lo fijan). Propuesta: sí.
3. **Quitar también `TC2_APP_*_ID/CLAVE`** (`Rol::variables_app`) en el lanzador, por las identidades E2E reales de esta máquina. Propuesta: sí, en la parte a.
4. **`panic!` como comportamiento único de los dobles** ante un método no esperado, frente a `Error`. Propuesta: `panic!`.
5. **Aceptar dos copias de `Parcial`/`Doble` y de `CONFIG`** (unitarios e integración); descartado exponerlos en la API pública o con una feature `prueba`.
6. **Troceo** en seis partes (ver abajo).

**Respuestas de la persona (Carlos, 2026-11-23):** acepta las propuestas de las decisiones 1 a 5 tal como están escritas, y acepta el troceo en seis partes (6).

Alternativas descartadas: `Opciones { carpetas, contenido }` en la instantánea (mantendría las variantes débiles); un `struct Lanzador` (duplica la API de `Command`); un doble con `Option<Box<dyn Fn>>` por método (obliga a reescribir los dobles con estado; `Parcial` los conserva).

## Troceo

Orden: a → b obligado; c, d, e y f son independientes entre sí y de a/b, y se hacen en serie por los ficheros que comparten. Cada parte cierra con `cargo test -p tc2 --no-fail-fast` (recuento antes y después) y `cargo xtask verificar` en verde.

### Parte a: lanzador del binario en tests/comun (A8-02, absorbe A5-07)
Crea `comun/lanzador.rs` (`BINARIO`, `tc2`, `texto`) con `LC_ALL=C`, `GIT_TERMINAL_PROMPT=0`, `TC2_IDIOMA=es` y sin tokens ni `TC2_APP_*`, y migra los 26 ficheros que lanzan `CARGO_BIN_EXE_tc2`.
Duracion estimada: 60 min
- [ ] `git grep -n 'CARGO_BIN_EXE_tc2' -- crates/tc2/tests` solo devuelve ficheros de `tests/comun/`.
- [ ] Un test de `tests/soporte.rs` comprueba que `comun::tc2(dir)` quita `TC2_TOKEN_PERSONA`, `TC2_TOKEN_AGENTE`, `TC2_TOKEN_REVISOR` y las variables `TC2_APP_*`, y fija `TC2_IDIOMA=es`.
- [ ] `cargo test -p tc2 --no-fail-fast` da el recuento de antes más los tests nuevos y las copias del test de `http_mock` justificadas, y `cargo xtask verificar` sale en verde; el `## Resultado` lista A8-02 y A5-07 con su commit.

### Parte b: escenario ssh, git de sistema y operacion de prueba (A8-01, A8-16, A3-06, A4-09, A8-20 integracion)
Crea `comun/remoto.rs` y `comun/git.rs`, extiende `comun/ops.rs`, pone `valor` y `salida` en `lanzador.rs`, migra aprobar, ciclo, estimacion, frescura, inyeccion, liberar, plan, progreso, solape, toma, alta, importar y `agente_git`, y funde `como_persona` y `hito_como_persona` en `aprobar.rs`.
Duracion estimada: 90 min
- [ ] `entorno_ssh`, `entorno_sin_red`, `identificar`, `otro_clon`, `editar`, `sin_catalogo`, `Escribir`, `salida`, `con_fila` y `preparar` solo se definen en `crates/tc2/tests/comun/`.
- [ ] Hay una sola `valor`, con `trim_end`, y un test de `tests/soporte.rs` comprueba `valor("k=v\n", "k") == Some("v")`.
- [ ] `agente_git::Clon::git` y `Fixture::crudo` usan `comun::git::git_en`, que aísla la configuración del usuario.
- [ ] `cargo test -p tc2 --no-fail-fast` y `cargo xtask verificar` en verde, con el recuento en `## Resultado` y los IDs con su commit.

### Parte c: servidor HTTP de los tests con comun::http_mock (A8-11)
`tests/identidad.rs` y `tests/init.rs` usan `comun::http_mock::Mock` y `serie()` en lugar de su `Servidor` propio.
Duracion estimada: 35 min
- [ ] `git grep -n -e 'Servidor::nuevo' -e 'struct Servidor' -- crates/tc2/tests` no devuelve nada.
- [ ] `cargo test -p tc2 --no-fail-fast` y `cargo xtask verificar` en verde; el `## Resultado` lista A8-11 con su commit.

### Parte d: instantanea de carpeta comun (A8-12, absorbe A7-06)
Crea `comun/instantanea.rs` (recursiva, anota carpetas, `"<len> <sha256>"`, con lista de exclusión) y migra init, desinstalar, importar, tablero, frescura y doctor.
Duracion estimada: 45 min
- [ ] Ninguno de `tests/init.rs`, `tests/desinstalar.rs`, `tests/importar.rs`, `tests/tablero.rs`, `tests/frescura.rs` y `tests/doctor.rs` recorre carpetas por su cuenta: usan `comun::instantanea`.
- [ ] Un test de `tests/soporte.rs` comprueba que una carpeta vacía nueva y un fichero en una subcarpeta cambian la foto, y que un nombre excluido no la cambia.
- [ ] `cargo test -p tc2 --no-fail-fast` y `cargo xtask verificar` en verde; el `## Resultado` lista A8-12 y A7-06 con su commit.

### Parte e: soporte comun de los tests unitarios (A8-18, A8-20 unitarios)
Crea `estado::prueba` (con `EnMemoria` traído de `roadmap::tests`, `estado` y `motivo`) y `git::prueba` (`CONFIG`, `entorno`, `sh`), y migra ciclo/operacion, ciclo/liberar, toma/operacion, plan/operacion, plan/aprobar, plan/guardar, revision/operacion, roadmap, siguiente, ciclo/mod, estado/montaje y git/estado.
Duracion estimada: 50 min
- [ ] Las funciones de prueba `estado` y `motivo` y el doble `EnMemoria` solo se definen en `estado::prueba`.
- [ ] `fn sh` y el `git init` de prueba de `estado/montaje.rs` y `git/estado.rs` aíslan con `git::prueba::entorno`, y un unitario comprueba que `git config user.email` devuelve el de `CONFIG`.
- [ ] `cargo test -p tc2 --no-fail-fast` y `cargo xtask verificar` en verde; el `## Resultado` lista A8-18 y A8-20 con su commit.

### Parte f: dobles de Plataforma con Parcial y Doble (A8-05, absorbe A2-06)
Crea `plataforma::prueba::{Parcial, Doble}` y `comun/plataforma.rs` con `panic!` ante un método no esperado, y migra `identidad.rs`, `revision/mod.rs`, `tablero.rs`, `tests/revision.rs` y `tests/tablero.rs`.
Duracion estimada: 50 min
- [ ] `git grep -n 'impl Plataforma for' -- crates/tc2` solo devuelve `github.rs`, `gitlab.rs` y los dos `Doble<P>`, y no queda ningún `sin_uso`.
- [ ] Dos tests `#[should_panic(expected = "merge-pr")]` sobre un `Doble` vacío, uno en `plataforma::prueba` y otro en `tests/soporte.rs`.
- [ ] `cargo test -p tc2 --no-fail-fast` y `cargo xtask verificar` en verde; el `## Resultado` lista A8-05 y A2-06 con su commit.

## Troceo aceptado

Troceo aceptado por una persona el 2026-11-23: TASK-148a, TASK-148b, TASK-148c, TASK-148d, TASK-148e, TASK-148f, todas en la misma rama. Este plan es el de TASK-148a; las demas partes se planifican al empezarlas.
