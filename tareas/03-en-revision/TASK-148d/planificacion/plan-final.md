# Plan — TASK-148d: instantanea de carpeta comun (A8-12, absorbe A7-06) (parte d de TASK-148)

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 50 min

Señales: 1 fichero nuevo (`comun/instantanea.rs`), 8 tocados (`comun/mod.rs`, `soporte.rs` y los seis tests), una capa (soporte de tests, sin producción), 3 criterios, una suite completa para el verde final.

- `crates/tc2/tests/comun/instantanea.rs`: `pub fn instantanea(dir: &Path, excluir: &[&str]) -> BTreeMap<String, String>`. Recorre todo el árbol; claves relativas con `/`; carpeta = `"dir"`, fichero = `"<len> <sha256>"` con `distribucion::sha256::hex`; carpeta inexistente → mapa vacío; fichero ilegible → `panic`. `excluir` compara el nombre de cada entrada a cualquier profundidad y poda el subárbol de una carpeta excluida. Es la forma de `desinstalar.rs:37-62` (la más fuerte), como fijó el plan padre.
- `comun/mod.rs`: `pub mod instantanea;` y `pub use instantanea::instantanea;` (como `lanzador`). Los tests llaman a `comun::instantanea(dir, &[])`.
- Cada test (leídos en develop `bbf5d27`):
  - `init.rs:131-152`: hoy contenido en bruto y carpetas `"x/"`; solo se usa en `assert_eq!` antes/después (248/283, 294/305, 326/347, 365/369, 497/504, 1580/1611): detecta los mismos cambios.
  - `desinstalar.rs:37-62`: se borra la función local; `desinstalar.rs:525` (`read_dir` para «vacía o no existe») pasa a `comun::instantanea(&wt_reg, &[]).is_empty()` (decisión 3).
  - `importar.rs:380-394`: hoy claves absolutas, sin carpetas y falla si no existe; gana la detección de carpetas vacías que cree `--simular`.
  - `tablero.rs:297-324`: se queda la parte de git; el bucle de `.git/taskcodetwo` pasa a `comun::instantanea(.., &[tc2::huella::FICHERO_SINCRONIZACION])`; gana carpetas.
  - `frescura.rs:261-291`: se queda la parte de git; el recorrido (hoy solo primer nivel) pasa a recursivo con la misma exclusión (decisión 2).
  - `doctor.rs:964-993`: se quedan git e `index`/`config`/`info/exclude`; el recorrido hoy no excluye la marca: `instantanea(.., &[])` (decisión 1).
- Las tres `foto` no se fusionan (su parte de git difiere a propósito); se descarta `Opciones { carpetas, contenido }`: las seis caben en la forma más fuerte.
- Orden: 1) `instantanea.rs`, `mod.rs` y tests en `soporte.rs`; 2) desinstalar e init (A7-06); 3) importar; 4) las tres `foto`; 5) grep, suite completa y `cargo xtask verificar`.

## Reutilizacion (Regla 4D)

Lo que ya existe y se usa (helpers, funciones, fixtures, tests), buscado en el repo antes de disenar; y lo que se crea nuevo, con el motivo de no reutilizar:

- `distribucion::sha256::hex` (dependencia normal, ya la usa `desinstalar.rs`).
- La forma de recorrer de `desinstalar.rs:37-62`: se mueve, no se reescribe.
- `tc2::huella::FICHERO_SINCRONIZACION` (`src/huella.rs:61`).
- `comun::Fixture::nueva` y `comun::escribir` para el test nuevo; el patrón de reexportar de `comun/mod.rs` y el binario `tests/soporte.rs`.
- `Fixture::huella` no sirve (solo refs, worktrees y status).
- Nuevo: `comun/instantanea.rs`, porque nada en `tests/comun` recorre carpetas. Se borran seis copias y queda una.

## Lo que el rol no cubrio

- Riesgos (no lanzado): no estudiado si cachés o el cerrojo (`estado.lock`, `src/estado/cerrojo.rs:76`) crean subcarpetas en `.git/taskcodetwo` durante lecturas, lo que pondría en rojo `frescura` o `tablero` al pasar a recursivo.
- Testing (no lanzado): plan de pruebas mínimo.
- Dominio (no lanzado): no revisado qué es «no cambiar» para cada orden; se conserva lo que miraba cada `foto`, más carpetas y recursión.

## Riesgos aceptados y que los contiene

- Fotos más fuertes (`frescura` recursiva; `importar`, `tablero`, `frescura` ven carpetas): si alguna se pone en rojo es un hallazgo de producción, se documenta y no se resuelve ampliando `excluir`. Lo contiene la suite completa.
- Mensajes de fallo con hashes en vez de bytes: aceptado.
- Carpeta inexistente da mapa vacío también en `importar`: aceptado (compara antes y después de la misma carpeta, que existe).

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- Criterio 2 (2, ligeros, en `tests/soporte.rs`, sin git): `instantanea_ve_carpetas_vacias_y_subcarpetas` (carpeta vacía nueva, `a/b/f.txt`, contenido cambiado con el mismo tamaño: cada uno cambia la foto); `instantanea_ignora_los_nombres_excluidos` (fichero y carpeta excluidos en raíz y subcarpeta no la cambian; carpeta inexistente da mapa vacío).
- Criterio 1 (0): `git grep -n -e read_dir -e "fn instantanea" -e "fn recorrer" -- crates/tc2/tests/{init,desinstalar,importar,tablero,frescura,doctor}.rs` vacío; los tests de esos seis ficheros en verde.
- Criterio 3 (pesados): `cargo test -p tc2 --no-fail-fast` (recuento antes + 2) y `cargo xtask verificar`, sin solapar.

## Lo que necesita decision de una persona

1. Marca de sincronización en `doctor`: `&[]` (mantiene la fuerza actual). Recomendado.
2. `frescura` recursiva y las tres `foto` viendo carpetas; un rojo es hallazgo de producción. Recomendado.
3. `desinstalar.rs:525` a `comun::instantanea(..).is_empty()`. Recomendado.
4. `excluir` compara el nombre a cualquier profundidad. Recomendado.

**Respuestas de la persona (Carlos, consigna del 2026-11-23: modo automatico con la opcion recomendada):** se aplican las recomendaciones 1 a 4.
