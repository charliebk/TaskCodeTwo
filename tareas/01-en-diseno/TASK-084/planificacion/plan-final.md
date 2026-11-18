# Plan — TASK-084: F3-12 progreso en vivo en el terminal

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 300 min

Modulo nuevo `crates/tc2/src/progreso/`. Las transiciones escriben un fichero derivado por tarea; dos vistas lo leen sin red y sin recorrer el historico: `tc2 estado --stdin` (status line) y `tc2 status --seguir` (terminal). Sin segunda formula: el calculo reutiliza las funciones puras de TASK-083 (`estimacion::{parecidas, restante, eventos, ventanas_pausa, tramo}`).

Comprobado: la orden de la status line ya esta fijada, `distribucion::STATUS_LINE = "tc2 estado --stdin"` (`crates/distribucion/src/lib.rs:41`), y `tc2 desinstalar` la reconoce por ese texto (`docs/distribucion/instalar.md:171`). `taskctl status --seguir` es `tc2 status --seguir` (F1-17 divergencia 10). Todas las transiciones de la 2.0 (start, descartar, pausa, reanudar, liberar, reasignar) pasan por `de_una_tarea` (`lib.rs:324`). `review`, `finish` y `plan` aun no existen: la fase «rondas» solo se alcanza con datos de prueba. Precedente de escritura local con temporal + `rename`: `estado/frescura.rs`, `identidad.rs` y `desinstalar/mod.rs:571`, ninguno con reintento (F1-17 punto 7 lo exige).

Piezas:
- `huella.rs`: `CARPETA_PROGRESO = "progreso"` bajo `.git/taskcodetwo/` y un unico `escribir_local(ruta, bytes)` (temporal + rename con reintento); frescura e identidad lo usan.
- `progreso/fichero.rs`: `progreso/TASK-NNN.json` (anterior en `TASK-NNN.anterior.json`) con version, id, estado, fase, fases que faltan, transcurrido al calcular, `calculada`, ultimo evento, nivel, n, tabla de lo que falta (de 0 a 480 min cada 5: mediana, p80, desbordada, sacada de `restante()`) y avisos saneados. Tamano independiente de N (L1).
- `progreso/calculo.rs` (puro): `calcular(&Tarea, &Cuerpo, &Historico, ahora) -> Option<Progreso>`; `en-curso` = implementacion, `en-revision` = rondas; la pausa congela el reloj; transcurrido con `tramo` (huecos a 45 min, D6a).
- `progreso/linea.rs` (puro): `linea(&Progreso, ahora, ultima_actividad)`; transcurrido = el del calculo + reloj desde el ultimo evento con el mismo tope; mas de 45 min: «sin actividad registrada hace N min»; ultima actividad = max de la del calculo y el mtime de `logs/HEAD` del gitdir del worktree (sin lanzar git).
- `progreso/mod.rs`: `refrescar(g, dir, id, ahora)` (estado local sin `fetch`, todos los `tarea.md` y un `git log`; escribe o borra; O(N), solo en transiciones); `orden_estado(stdin, ahora)` sin `Git` en la firma (sin red ni subprocesos por construccion); `seguir(dir, &mut dyn Write, reloj, parar)`. Busqueda sin git: `workspace.current_dir` del JSON, subir hasta `.git` (carpeta o fichero `gitdir:` con `commondir`); dentro de `.taskcodetwo/worktrees/<ID>` la tarea es esa; en el principal, el `calculada` mas reciente de `progreso/` mas «+N en curso».
- `lib.rs` y `main.rs`: `de_una_tarea` llama a `refrescar` en exito con `id=`; un fallo no cambia la transicion (`progreso=no-escrito` y aviso en stderr). `estado --stdin` al principio del despacho, sin identidad ni catalogo. `status --seguir` en `main.rs` (`Salida` no escribe a trozos); `tablero` solo rechaza `--seguir` y presta `sanear` (sin ciclo).
- `init` y `docs/distribucion/instalar.md`: `init` imprime la propuesta de `statusLine` con el comando exacto; nunca escribe `settings.json`.

Formato de la linea (L5 de F1-17): `TASK-NNN · implementacion · fases que faltan: rondas, cierre · faltan ≈X min (p80 Y) · n=N <nivel>`; desbordada `faltan ≥X min (fase en curso por encima de sus parecidas)`; sin datos `faltan: sin estimacion (n=N)`; pausada `TASK-NNN · pausada · ...`; sin tarea `tc2 · sin tarea en curso`; fichero ilegible: lo ultimo bueno de `.anterior` con «(dato de hace N min)». Siempre una linea y codigo 0.

`--seguir`: cada 2 s mira (mtime, tamano) y repinta; imprime una linea nueva solo si cambia el texto; sin ANSI; termina con Ctrl+C o al cerrarse stdout (codigo 0); `parar` y `reloj` inyectados solo en pruebas.

Lote: queda en F4-9 (`tc2 lote`, F4-7, no existe); el fichero lleva version para anadir `lote`.

Orden: (1) `huella` y `escribir_local` con migracion de frescura e identidad; (2) `fichero` y `calculo`; (3) `linea` y `tc2 estado --stdin` con la busqueda sin git; (4) `refrescar` en `de_una_tarea` y borrado en descartar, liberar y reasignar; (5) `status --seguir`, propuesta de `init` y documentacion.

Descartado: `estado.json` en el gitdir del worktree (F1-17 punto 7): no se ve desde el principal y queda fuera de lo que borra `desinstalar` (divergencia a anotar en el Resultado y en una adenda); recalcular en cada refresco (O(N), L1) o restar el reloj a una sola mediana (duplica `restante`); refresco dentro de `estado::transaccion` (dependencia al reves).

## Lo que el rol no cubrio

- Riesgos: dos clones de la misma persona, reloj desajustado, antivirus y `rename`, `current_dir` ausente en el JSON.
- Dominio: que la pausa congele el reloj, el tope de 45 min en la lectura, que se ensena en diseno.
- Testing sin revisar.
- «Durante una tarea real» no se cumple del todo con la 2.0 de hoy (las tareas reales se hacen con la 0.16, RR-14; faltan review y finish): la comprobacion real queda para F9-7 / F9-8b.
- Suposiciones a verificar: que Claude Code manda `workspace.current_dir` en el JSON de stdin y con que shell lanza la status line en Windows; que `logs/HEAD` del worktree existe por defecto (`core.logAllRefUpdates`); que `tramo` sirve para una tarea abierta (fin = ahora).

## Riesgos aceptados y que los contiene

- Refresco O(N) por transicion (todos los `tarea.md` mas un `git log`; en `start` se calcula dos veces): solo en transiciones; se mide en el E2E.
- La actividad posterior al calculo solo se ve por el mtime de `logs/HEAD`: el trabajo sin commit sale como «sin actividad».
- La fase de rondas no se alcanza en real hasta F6-2 y F7-1, que deberan llamar a `progreso::refrescar` (anotado en el modulo).

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- Unitarias puras: `calcular` (fase, pausa, desborde); la tabla coincide con `restante()` en cada punto; `linea` en sus 7 formatos; busqueda de carpeta (`.git` fichero o carpeta, subcarpetas, principal o worktree); `escribir_local` con un lector abierto.
- Integracion (`crates/tc2/tests/progreso.rs`): repos con 10 y 1000 tareas, transiciones reales, `estado --stdin` como proceso, `--seguir` con `parar` inyectado, propuesta de `init`.
- Medicion del tiempo de `estado --stdin` en Windows anotada en el Resultado.

## Lo que necesita decision de una persona

Carlos dejo dicho que en el H3 se acepta la opcion recomendada y que no se trocea; se aplican:

- D1 donde vive el fichero: (A) `.git/taskcodetwo/progreso/TASK-NNN.json` (huella F1-12), con divergencia del punto 7 de F1-17 anotada. **Elegida.**
- D2 instalacion de la status line: (A) `init` imprime la propuesta e `instalar.md` la documenta; la persona la pega (se recomienda `.claude/settings.local.json` del proyecto para no sustituir la de la 0.16 en `~/.claude/settings.json`). **Elegida.**
- D3 el lote: (A) queda en F4-9. **Elegida.**
- D4 «durante una tarea real»: (A) se acepta con el E2E de las transiciones existentes y la comprobacion real queda para F9-7. **Elegida.**
