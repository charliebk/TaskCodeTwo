# Informe de revision — TASK-023 (ronda 2)

- Commit revisado: 964898f7d7e9298ce912ae2a54c2b8b39e88f998
- Revisor: agente general-purpose (modelo haiku) con la skill code-quality-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/config/validar.rs:341 | implementacion |
| MEN-6 | MENOR | abierto | crates/tc2/src/config/validar.rs:262 | implementacion |
| MEN-7 | MENOR | abierto | crates/tc2/src/config/validar.rs:341 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

## Entorno y puerta

- Clon limpio en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-023-r2-calidad`, rama de la tarea. El HEAD del clon es efb4720 (la peticion de revision); respecto a 964898f solo anade ficheros de `tareas/` (`git diff --stat`), asi que el codigo revisado es el de 964898f. No he tocado el repo original ni he hecho commits.
- `cargo test -p tc2 --test config --test tarea --lib`: 59 passed, 1 ignored, 0 failed, codigo de salida 0. Una sola ejecucion en la ronda. El ignorado es `lee_las_tareas_de_la_0_16_de_otros_repos` (pesado: necesita `TC2_CORPUS`).
- Reproduccion de IMP-1 con un test temporal en el clon (`crates/tc2/tests/r2_temporal.rs`, no forma parte de la entrega), contra `config::leer` con `derivados: [{comando: x, tipo: genera, escribe: [<ruta>]}]`:
  - Rechazadas, todas en `derivados[0].escribe[0]` con el mensaje de punto o espacio final: `tareas/ROADMAP.md.`, `tareas/ROADMAP.md ` (con espacio), `tareas/TASK-001./x`, `tareas/TASK-001 /x`, `tareas/.git./x`, `tareas/.git /x`, `tareas/aux./x`, `tareas/x./y`, `tareas/.../y`. `tareas/ROADMAP.md/` se rechaza por la regla de ROADMAP.
  - Aceptadas (legitimas): `tareas/hitos.md`, `tareas/informes/semana-1.md`, `tareas/v1.2/a.md`, `tareas/TASK-mapa.md`, `tareas/ROADMAP.md.bak`, `tareas/x/.hidden.md`.
  - IMP-1 queda `corregido`.
- Comprobacion de la correccion de MEN-2 (aunque no esta en la tabla de esta ronda): `modo_flujo: tc2 actualizar` ya no elige el hint de la 0.16 (la decision depende de `ruta`, no del texto del mensaje, y `CLAVES_016` es disjunta de la tabla 2.0 segun el test de `claves.rs:654`). `rama_base: develop` si da el hint de la 0.16. `formato: 3` da `formato-nuevo` con el mensaje correcto.
- Segunda comprobacion (test temporal `crates/tc2/tests/r2_temporal2.rs`) de falsos positivos introducidos por la correccion, con los resultados que se citan en MEN-6 y MEN-7.

## Hallazgos

### IMP-1 — `derivados[].escribe`: la reserva de `ROADMAP.md`, `TASK-NNN/` y `.git` se saltaba con punto o espacio finales

Estado: `corregido`. La reproduccion de la ronda 1 ya no pasa: ver la seccion Entorno. La regla nueva (`validar.rs:341`) rechaza cualquier segmento que acabe en `.` o espacio antes de comparar con las reservas.

### MEN-6 — `confianza_incluye`: `*.*`, `./*.*` y `.*` se rechazan con «nunca todo el arbol»

Reproducido: `confianza_incluye: ['*.*']`, `['./*.*']` y `['.*']` dan `RECHAZADA` con «glob no admitido «...»: nunca todo el arbol». La version anterior los aceptaba (no estaban en la lista literal), asi que el rechazo es nuevo de esta correccion.

Causa: `todo_el_arbol` (`validar.rs:262`) da verdadero para cualquier glob cuyos segmentos solo tienen `*` y `.`. `*.*` y `.*` solo casan con ficheros de la raiz (o con ocultos de la raiz), no con todo el arbol. El caso es discutible: la especificacion ya prohibia `*` (que tambien es solo raiz), pero el mensaje dice «todo el arbol», que aqui no es cierto.

Impacto: rechazo seguro, sin perdida de datos, con mensaje engañoso para una configuracion que el autor puede querer. MENOR. Arreglo posible: exigir que el glob sea `**` o `*` seguidos de `/**` o `/*` (o acotar el mensaje y documentar que `*.*` tambien se prohibe).

### MEN-7 — `ruta_tareas`: `~` rechaza cualquier nombre con tilde, no solo los nombres cortos de 8.3

Reproducido: `escribe: ['tareas/notas~v2.md']` y `escribe: ['tareas/x/a~b.md']` dan `RECHAZADA` («sin punto ni espacio al final de un segmento ni ~»). Son nombres validos en Windows, Linux y macOS. La regla (`validar.rs:341`, `p.contains('~')`) es nueva de esta correccion: la version anterior los aceptaba.

Impacto: rechazo seguro, sin perdida de datos, pero es una restriccion mayor que la que pedia el hallazgo de la ronda 1 (que solo hablaba de los nombres cortos, `ROADMA~1.MD`, y proponia aceptarlos y documentarlos). MENOR. Arreglo posible: rechazar solo el patron de nombre corto (`~` seguido de digitos) o documentar la restriccion en la descripcion del esquema.

## Notas (fuera de la tabla, no abren hallazgo)

- Barra invertida en globs: `confianza_incluye: ['**\*']` pasa como OK. `todo_el_arbol` solo divide por `/`, mientras que `glob_malo` trata `\` como separador. Ya pasaba con la lista literal de la ronda anterior, asi que no es fallo de esta correccion. No he verificado como lo consume el lector del glob en Windows. Si el arreglo pretende cerrar la clase «ningun segmento solo de * y .», deberia cubrir `\` tambien.
- Cron: aceptados `0 3 * * 1-5`, `*/15 9-17 * * 1-5`, `0 0 1,15 * *`, `5/10 * * * *`, `0 0 * * 7`, `0 0 1 1 0-7`. Rechazados `60 * * * *`, `0 0 0 * *`, `0 0 * 13 *`, `*/0 * * * *` y `5-1 * * * *`. Correcto.
- Orden multilinea (`comando: |` con dos lineas): rechazada con el mensaje nuevo «la orden tiene saltos de linea». No es regresion: el `\n` ya era caracter de control y se rechazaba antes; la correccion solo mejora el mensaje.
- Lista con duplicados: `find_map` informa solo del primer duplicado. Sin impacto practico.
- Sin verificar en esta ronda: mutantes. La peticion limita la verificacion al diff de la correccion, y no se han repetido.

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-023/revision/informe-revision-1-code-quality-reviewer.md`: para contrastar el detalle de IMP-1 y MEN-2 (la peticion solo trae la fila de la tabla).
- `git diff --stat 964898f efb4720`: para confirmar que el commit de la peticion solo anade ficheros de `tareas/` respecto al commit revisado.
