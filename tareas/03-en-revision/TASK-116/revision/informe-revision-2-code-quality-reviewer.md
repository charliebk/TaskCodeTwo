# Informe de revision — TASK-116 (ronda 2)

- Commit revisado: 727d2c560c1126a56d130b778cb830a46d211e21
- Revisor: code-quality-reviewer (verificacion acotada de la ronda 2)
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | corregido | docs/adr/README.md:11 | implementacion |
| MEN-2 | MENOR | corregido | docs/adr/F1-12-actualizacion-entre-versiones.md:551 | implementacion |
| MEN-3 | MENOR | corregido | crates/tc2/tests/convivencia_016.rs:118 | implementacion |
| MEN-4 | MENOR | corregido | crates/tc2/src/git/puerto.rs:514 | implementacion |

Sin CRITICOS ni IMPORTANTES nuevos. Ningun hallazgo nuevo abierto: lo que la correccion
rompe no aparece, y lo que queda fuera de alcance va como nota abajo.

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

## Reproduccion y verificacion

Clon temporal en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-116-r2-calidad` (borrado al terminar,
sin cambios sin subir). Rama `fix/task-116-nombres-de-la-2-0-alineados-con-f1-12-d1`. El HEAD de la
rama es a206230, que solo anade la carpeta de revision ronda 2 (ningun fichero de codigo ni de ADR);
el codigo revisado es el de 727d2c5.

- `cargo test -p tc2 --test convivencia_016`: 1/1 en verde (6-7 s).
- Suite entera `cargo test -p tc2 --no-fail-fast`: 22 binarios `test result: ok`, 0 fallos
  (algunos binarios solo tienen tests ignorados). Sin fallos por la red local en esta pasada.
- Mutacion `REFS_PENDIENTES = "refs/taskcode/pendientes/"` (en `crates/tc2/src/huella.rs:38`):
  `la_huella_de_la_016_queda_intacta_tras_new_y_start` pasa a rojo en `convivencia_016.rs:181`
  (`left: "estado error git=update-ref"`, `right: "estado sinred operacion=anotar pendiente=si pendientes=1"`).
  Nota: el rojo lo produce el guard de `update-ref` (solo acepta `refs/taskcodetwo/`), no la aserción
  final sobre `refs/taskcodetwo/pendientes/000001`. Sirve como red, pero la aserción de la ref
  queda sin ejercitar en esa mutación concreta. Restaurado con `git checkout`; de nuevo en verde.
- Correcciones, una por una:
  - Adenda de TASK-116 presente en F1-1, F1-5, F1-6, F1-7, F1-8, F1-9, F1-10, F1-11, F1-16 (y ya estaba en F1-2, F1-3, F1-4).
  - El enlace `F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella`
    resuelve: el encabezado de F1-12:547 es `#### Adenda de TASK-116 (2026-11-16): todos los nombres de la huella`,
    cuyo slug coincide. Las 13 apariciones del enlace apuntan al mismo ancla. Fichero destino existente.
  - La lista de F1-12 (lineas 549-556) enumera F1-1, F1-2, F1-3, F1-4, F1-5, F1-6, F1-7, F1-8, F1-9, F1-10, F1-11 y F1-16.
  - `docs/adr/README.md:11`: la fila de F1-1 dice `taskcodetwo/tareas` y enlaza la adenda. Corregida.
  - `convivencia_016.rs`: el test ahora hace una escritura de estado sin red (`estado::transaccion` con
    remoto `127.0.0.1:1`), comprueba la linea `estado sinred ... pendientes=1` y que la pendiente queda en
    `refs/taskcodetwo/pendientes/000001`; `huella_016` incluye `refs/taskcode/` como espacio vacio de la 0.16.
    Verificado con la mutacion de arriba.
  - `crates/tc2/src/git/puerto.rs:514`: comentario `// intencionado: caso de rechazo del nombre viejo` presente
    justo antes del literal `refs/taskcode/pendientes/000001`. Solo comentario, sin efecto en el comportamiento.
- Grep de nombres viejos en `docs/` y `crates/`: ver nota 1.

## Notas (fuera de la tabla, no abren ronda)

1. Dos ADR que nombran las piezas viejas no llevan adenda, y la adenda dice ser la lectura valida
   de F1-1..F1-11 y F1-16 (no las menciona): `docs/adr/F1-13-adaptador-de-plataforma.md:451` (contraste
   «en vez de `taskcode` y `taskcode/tareas`», correcto tal cual) y `docs/adr/F1-19-memoria-del-proyecto.md:249,254,284`
   (ADR descartado el 2026-11-16 por TASK-113, con `refs/taskcode/`, `taskcode-estado.lock` y `taskcode/tareas` sin marcar).
   No entra en la lista de F1-12 y no es un fallo de la correccion; si se quiere cerrar la cobertura, basta con
   una frase en F1-13 o en la nota de F1-19.
2. La aserción de la ref pendiente en `convivencia_016` se apoya en que `update-ref` rechaza el prefijo viejo; si
   algun dia se relaja ese guard, la mutacion de `REFS_PENDIENTES` solo se veria por la aserción final de
   `refs/taskcodetwo/` (que sigue ahí). Sin accion necesaria.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/huella.rs` (linea 38): para la mutacion `REFS_PENDIENTES` y la restauracion.
- `docs/adr/F1-13-adaptador-de-plataforma.md` y `docs/adr/F1-19-memoria-del-proyecto.md`: grep de nombres viejos
  para comprobar que la adenda cubre todos los ADR afectados (nota 1).
- `crates/tc2/src/estado/` y `crates/tc2/src/git/puerto.rs` (lineas 415-420 y 509-527): para confirmar que el
  guard de `update-ref` rechaza el prefijo viejo (nota 2).
