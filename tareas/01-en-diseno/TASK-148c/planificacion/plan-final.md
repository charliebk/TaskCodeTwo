# Plan — TASK-148c: servidor HTTP de los tests con comun::http_mock (A8-11) (parte c de TASK-148)

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 55 min

Señales: 4 ficheros de código (`comun/http_mock.rs`, `identidad.rs`, `init.rs`, `actualizar_cli.rs`), una capa (tests de integración), 2 criterios, 1 ID. Sube 20 min sobre los 35 del plan padre porque entra `actualizar_cli.rs`.

**Hallazgo que cambia el alcance.** El grep del criterio 1 también encuentra `crates/tc2/tests/actualizar_cli.rs:57` (`struct Servidor`) y sus 8 llamadas `Servidor::nuevo`, `con_corte` y `con_redirecciones` (líneas 300-676). La ficha A8-11 ya cita `actualizar_cli.rs:121-127` (`base` y `peticiones`). Sin tocarlo, el criterio 1 no se cumple (decisión 1).

**Forma.** Un solo doble HTTP, `comun::http_mock::Mock`, extendido lo justo; ningún servidor nuevo.

1. `identidad.rs`: se borran `Servidor` y `atender` (31-98) y los `use` sobrantes. `Servidor::nuevo(&[("/user", c)])` → `Mock::nuevo()` + `responder("GET", "/user", Simulada::json(200, c))` (sin `Mock::con_rutas_get`, como decidió el plan padre). `peticiones()` → `recibidas()` mapeado a `.linea` (mismo formato `"GET /user"`, `http_mock.rs:152`). Cada test con mock coge `let _serie = serie();`. El `//!` de cabecera remite a `comun::http_mock`.
2. `init.rs`: se borran `Servidor` y `atender` (29-105); `github()` devuelve un `Mock` con sus tres rutas GET; `peticiones()` (495, 544) → `recibidas()`; `sin_remoto_completa_al_anadirlo` y `repo_vacio_con_remoto_publica` cogen `serie()`. Equivalente: `http_mock` quita la consulta (157) y da 404 a lo no registrado (171), como el filtro de `init.rs:91`.
3. `comun/http_mock.rs` (se extiende): `Simulada.cuerpo` pasa de `String` a `Vec<u8>` (`json(&str)` igual, nuevo `Simulada::bytes(estado, &[u8])`, porque `tc2_falso` no es UTF-8); `Simulada::cortada(Corte)` con `enum Corte { AntesDeCabeceras, AMitadDelCuerpo }` movido desde `actualizar_cli.rs:63` (el corte de una sola vez sale de la cola por ruta que ya existe; solo ese camino hace `shutdown(Write)`, el normal sigue sin FIN); la redirección es `Simulada::bytes(302, b"").con("Location", destino)`.
4. `actualizar_cli.rs`: se borran `Servidor`, `Ficheros`, `Cortes`, `Corte` y `atender` (55-202); ayudante local `fn servir(release: BTreeMap<String, Vec<u8>>) -> Mock` (traducción del `release()` de este fichero); corte y redirección con `responder` en cada test; las comprobaciones de `peticiones()` pasan a `"GET /<fichero>"`.

**Orden.** (1) y (2) primero, después (3) y (4); un commit por paso; A8-11 cita el último.

## Reutilizacion (Regla 4D)

Lo que ya existe y se usa (helpers, funciones, fixtures, tests), buscado en el repo antes de disenar; y lo que se crea nuevo, con el motivo de no reutilizar:

- Se reutiliza: `comun::http_mock::{Mock, Simulada, Recibida, serie}` (ya lo usan doctor, identidades, init, plataforma, plataforma_gitlab, prs_abiertos, revision y tablero); `Mock::base()` sustituye a los tres `base()`; `Mock::recibidas()` a los tres `peticiones()`; la cola de respuestas por ruta da el corte de una vez sin estado nuevo (sustituye `Cortes`); `Simulada::con` para `Location`; el test `tras_una_respuesta_completa_el_mock_no_manda_fin` (`http_mock.rs:198`).
- Se crea: `Simulada::bytes` (`json(&str)` no admite no-UTF-8); `Simulada::cortada` y `Corte` en `comun` (se mueven, no se copian); `servir()` local en `actualizar_cli.rs`.
- No se crea: `Mock::con_rutas_get` ni `Mock::ficheros` (otro punto de entrada para lo mismo).

## Lo que el rol no cubrio

- Riesgos (no lanzado): no repasado caso por caso que la línea de estado (`200 X` en vez de `200 OK`, `http_mock.rs:177`) y el `Content-Type: application/json` en cuerpos binarios no alteren lo que lee `actualizar --cli`; lo cubren los tests de `actualizar_cli.rs`.
- Testing (no lanzado): no valorado si cada comprobación conserva su fuerza al pasar de `peticiones()` (ruta) a `recibidas()` (método y ruta).
- Dominio (no lanzado): sin reglas de negocio.

## Riesgos aceptados y que los contiene

- `Simulada.cuerpo` a `Vec<u8>` toca un campo público usado en ~10 binarios: solo `http_mock.rs` lo lee (fuera de `comun` solo aparece `Recibida.cuerpo`, que no cambia); lo contiene el compilador.
- `serie()` en `actualizar_cli.rs` serializa tests lentos: tiempo de pared mayor (se anota); el contrato de `serie()` existe por el antivirus de esta máquina.
- El camino del corte manda FIN dentro del doble común: solo con `Simulada::cortada`; el test del camino normal sigue fijándolo.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- Criterio 1 (0 tests): el `git grep` del criterio vacío, y `git grep -n 'TcpListener::bind' -- crates/tc2/tests` solo en `comun/` y en los puertos cerrados a propósito (`identidad.rs:374`).
- Criterio 2 (1, ligero, en `tests/soporte.rs`): una ruta con `Simulada::cortada(AMitadDelCuerpo)` seguida de `Simulada::bytes(200, &[0xff, 0x00, 0x80])` da primero una respuesta truncada con FIN y después los bytes exactos. `AntesDeCabeceras` lo ejercita `un_corte_en_la_descarga_del_binario_se_reintenta_una_vez`.
- Pesados: ninguno nuevo; `cargo test -p tc2 --no-fail-fast` (recuento antes + 1) y `cargo xtask verificar`.

## Lo que necesita decision de una persona

1. Alcance: (a) migrar `actualizar_cli.rs` a `Mock` extendiendo `http_mock` (recomendado); (b) renombrar su `Servidor` (Disfraz); (c) reescribir el criterio.
2. `Simulada.cuerpo` a `Vec<u8>` en vez de un segundo campo (recomendado: cambiar el tipo).
3. `serie()` también en `actualizar_cli.rs` (recomendado: sí).

**Respuestas de la persona (Carlos, consigna del 2026-11-23: modo automatico con la opcion recomendada):** 1 (a), 2 cambiar el tipo, 3 sí.
