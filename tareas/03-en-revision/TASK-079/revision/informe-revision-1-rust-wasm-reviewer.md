# Informe de revisión — TASK-079 (ronda 1)

- Commit revisado: ba24ba8
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| H1 | MENOR | corregido | crates/tc2/src/estado/frescura.rs:307-314 (`traer_estado`); crates/tc2/src/git/remoto.rs:118-126 | heredado (clasificacion previa), expuesto por la lectura |
| H2 | MENOR | corregido | crates/tc2/src/estado/frescura.rs:341-357 (`leer`), :434 (`hace`), :467-476 (`aviso`) | implementacion |
| H3 | MENOR | corregido | crates/tc2/tests/frescura.rs (C1, C2, C3) | implementacion |
| H4 | MENOR | corregido | crates/tc2/src/estado/lectura.rs:758-775 (`leer_fresco`); transaccion.rs:270-299 | implementacion |
| H5 | MENOR | aceptado | crates/tc2/tests/tablero.rs:527 (`board_con_la_plataforma_de_produccion`) | implementacion |

### H1: fallos de `fetch` que no son de red salen `remoto=sin-red`

Todo stderr desconocido se clasifica `SinRed`. Reproducido: rama de estado del bare reescrita a un commit huerfano (fetch sin `+` rechaza non-fast-forward): `status --json` sale 0 con `remoto=sin-red` y congela la foto con un aviso falso; `new` sale 4. Igual con un `refs/remotes/origin/taskcodetwo/tareas.lock` huerfano. La clasificacion de la transaccion no cambia; lo nuevo es que la lectura muestra esa etiqueta. Sugerencia: `rejected` y `cannot lock ref` como `error-local` o motivo propio.

### H2: marca con fecha futura se muestra como valida

Con `99999999999` y sin red, `sincronizado=5138-11-16T09:46:39Z` y «hace menos de 1 min». Contradice «nunca mas fresca de lo real». Tratar `s > ahora` como «nunca».

### H3: la renovacion de la marca por la lectura la protege un solo test

Quitar `marca.registrar` en `traer_estado` solo pone rojo C4; C1-C3 siguen verdes porque la sostiene el `new`. Sugerencia: en C2/C3 borrar la marca antes del `status` con red y comprobar que se recrea.

### H4: carrera de la lectura sin cerrojo contra el `fetch` de la transaccion

Dos `fetch` simultaneos pueden dar `cannot lock ref` → `SinRed` (H1). Estresado con 30 y 8 procesos (lecturas y `new`): 0 fallos. Riesgo teorico; dos lecturas a la vez pueden pisar la marca con un `ahora` algo anterior (conservador).

### H5: test intermitente

`board_con_la_plataforma_de_produccion` fallo 1 de 4 (`plataforma` distinto de `ok`) con la maquina cargada; paso solo y en dos pasadas completas. Causa no determinada.

## Reproduccion

Clon en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-079-r1-rust` (borrado). `fmt --check` y `clippy -D warnings` limpios. `--lib` 200 ok; frescura 10/10; siguiente 6/6; tablero 5/5 (salvo H5); alta 12, ciclo 15, importar 9, estado_bordes 10, estado_casos 6, estado_concurrencia 5, roadmap 13. Mutaciones: M1 (sin registrar en `traer_estado`) solo rojo C4; M2 (sin registrar tras publicar) rojo C5 y C8.

## Revisado sin hallazgos

Escritura atomica de la marca (temporal con PID y `rename`, limpieza si falla; en Windows el `rename` sobre una marca abierta no falla); marca corrupta, vacia, negativa o enorme = «nunca»; `traer_estado` equivalente al anterior (stderr, clasificacion, `RamaInexistente`, reintentos); renovacion tras publicar; `sin-rama`; la lectura no toma el cerrojo ni escribe mas que la ref remota y la marca; sin unsafe/unwrap/panic fuera de tests; instante UTC y «hace N» correctos en los limites. Limite JS-WASM: no aplica.

Nota de quien orquesta (ronda 1): H5 queda `aceptado`, no corregido: no se reprodujo en 18 ejecuciones (6 en paralelo); el test ahora dice en su fallo el valor de `plataforma`, el paso, lo que tardo, el stderr y las peticiones del mock. Causa probable la del antivirus de TASK-115, sin demostrar.
