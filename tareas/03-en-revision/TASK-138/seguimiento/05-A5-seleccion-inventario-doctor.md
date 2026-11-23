# Seguimiento de TASK-138, grupo A5 (fuera del repo; para `taskctl import --tipo fix`)

### Auditoria 4D: A5 seleccion, inventario, recursos y doctor (6 IMPORTANTE)
> Corregir los 6 casos IMPORTANTE del grupo A5 de `docs/auditoria-4d.md` (commit auditado d82ea9d): 4 Duplicaciones (dos `divergente`), 1 Desviación de F1-15 que cambia el comportamiento y 1 Disfraz de test con mutante confirmado. A5-07 va en la tarea de soporte de tests compartido. No se piden los MENOR (A5-04, A5-05, A5-10 a A5-14). No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.
- A5-01 (`divergente`, va primero): leer skills de una carpeta (`seleccion/catalogo.rs:232` ~ `inventario.rs:423`) es una sola función. El catálogo deja incidencia de lo que no puede leer, como exige C8 (`inventario.rs:28-30`), y lo prueba un test con una carpeta ilegible.
- A5-02 (`divergente`): la carpeta instalada del plugin habilitado se resuelve con una sola regla, la del `scope` del proyecto (`seleccion/catalogo.rs:265` ~ `inventario.rs:815`). Lo prueba un test con dos instalaciones de `scope` distinto.
- A5-03: con una revisora de dominio declarada y no instalada, la selección da el aviso `dominio-sin-revisora` (F1-15, S10 y S11, líneas 527 y 551-555) en vez de `no-instalado` (`seleccion/mod.rs:344`, `revisoras_de` en `:260`). Lo prueba un test nuevo.
- A5-06: `las_reglas_se_leen_de_la_tabla` (`seleccion/mod.rs:849`) no convierte el `Err` en `default()`. Con el mutante de la ficha aplicado, `cargo test -p tc2 --lib seleccion::tests::las_reglas_se_leen_de_la_tabla` sale en rojo.
- A5-08 y A5-09: la comprobación «5 líneas, JSON con todo y `y N mas`» (`tests/doctor.rs:958` ~ `tests/recursos.rs:378`) queda en un ayudante, y `doctor_no_escribe_nada` (`tests/doctor.rs:1017`) llama a `doctor_contra` (`:519`).
- `cargo xtask verificar` en verde, y el `## Resultado` lista los 6 IDs con su commit.
