# Informe de revision — TASK-094 (ronda 1)

- Commit revisado: 70e685d5f42f815aa972ac9890ab5c8771692018 (HEAD de la rama al reproducir: ced700d, solo anade la peticion)
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

Pase generico (el diff es un ADR, un indice y una PoC: ningun revisor de dominio casa). Profundidad: ADR y PoC a fondo; F1-10, F1-11 y F1-12 contrastados en los puntos que el ADR cita (codigos de salida, carril efectivo, hash/PL1/CR1), no releidos enteros.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-18-hito-de-decisiones.md:452 | plan |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-18-hito-de-decisiones.md:476 | implementacion |
| IMP-3 | IMPORTANTE | corregido | docs/adr/F1-18-hito-de-decisiones.md:416 | plan |
| MEN-1 | MENOR | corregido | docs/adr/F1-18-hito-de-decisiones.md:178 | implementacion |
| MEN-2 | MENOR | corregido | docs/adr/F1-18-hito-de-decisiones.md:64 | implementacion |
| MEN-3 | MENOR | corregido | docs/adr/F1-18-hito-de-decisiones.md:41 | implementacion |
| MEN-4 | MENOR | corregido | docs/adr/F1-18-hito-de-decisiones.md:480 | implementacion |

## Reproduccion de cada hallazgo

### IMP-1 — «cerrado» y la lista del registro se pueden manipular sin firma de persona (contradice D3a / F1-6)

Segun el punto 5 (y `hito_estado`, `poc-f1-18.py:445-457`), `cerrado` = lista no vacia con todas las tareas `terminada` o `descartada`; y la puerta deja pasar a toda tarea cuyo ID este en `tareas` (`poc-f1-18.py:545`). Un registro bien formado pero cambiado no cuenta como «roto» y no falla cerrado:

1. `tareas: ["TASK-001"]` (una tarea ya terminada) da `cerrado` (el codigo de `hito_estado` solo mira el estado de lo listado).
2. Descartar las tareas del hito (`descartada` esta en `FIN`) da `cerrado` sin que ninguna persona firme.
3. Anadir a `tareas` el ID de una tarea de codigo hace que `puerta` devuelva 0 («tarea del hito»).

Esto es exactamente lo que D3a decia impedir («solo lo firma una persona»): el efecto de «no necesario» se consigue sin firma. La PoC R solo prueba registros ilegibles o incompletos (6/6), no registros validos editados; el ADR (`Preguntas abiertas`, «Riesgos») solo menciona el borrado y lo da por contenido por F1-5, no verificado. Solucion posible: que el registro guarde el `informe_hash` y que `cerrado` exija que la lista incluya lo que propuso el informe, o que cambiar la lista y descartar tareas del hito exija firma de persona; como minimo, anotarlo en «Riesgos» y en «No verificado» y anadir un caso a la PoC R (lista reducida a una tarea terminada ajena al hito). No lo he podido ejecutar contra `hito_estado` con un repo de prueba; la reproduccion es por lectura del codigo citado (marcado como no ejecutado).

### IMP-2 — el codigo de precondicion «4, el de F1-12» no es el de F1-12 y choca con F1-2/F1-4

El ADR (punto 7, linea 476-478) y la PoC (`poc-f1-18.py:49`, comentario «el de F1-12») dicen que 4 es el codigo de precondicion de F1-12. Reproduccion: `grep -n "codigo [0-9]" docs/adr/F1-12-*.md` solo da 5 (hueco bloqueante), 6 (vuelta atras), 3 (hash distinto), 2; ninguna precondicion con numero (F1-12:412 las lista sin codigo). En F1-2/F1-4 el 4 es **SinRed** (`F1-4:155 fetch sinred ... codigo 4`, `F1-2:166 start: SinRed (codigo 4)`). Es una afirmacion no respaldada por la fuente y, ademas, colisiona con un codigo ya asignado a otra cosa en `start`. Correccion: no fijar numero (decir «codigo de precondicion, numero en F2/F1-4») o proponerlo como divergencia explicita, y quitar «el de F1-12» del ADR y de la PoC; la PoC P (`codigos [0, 4]`) debe usar una constante nombrada.

### IMP-3 — nadie crea `hito-decisiones.yml` en `actualizar`; con `ausente` la puerta niega

Punto 3 (linea 416-421): tras migrar, «`tc2 import` del fichero propuesto y escritura del registro»; consecuencias: F3-2 «anade las tareas a la lista del hito». Pero con el estado `ausente` la puerta niega (PoC P: «hito ausente: start pasa 0 / se niega 81»), `tc2 hito proponer` «no toca el repo», y los comandos del punto 6 son `estado`, `proponer`, `no-necesario`: ninguno crea el registro (en `init` lo crea F2-3, en `actualizar` no). Ademas `tc2 hito no-necesario` no puede firmar sobre un registro inexistente (el esquema exige `formato`, `origen`, `informe_hash`). Secuencia que se atasca: proyecto migrado → registro `ausente` → ni se puede importar las tareas del hito a una lista que no existe ni arrancar codigo. Correccion: nombrar quien escribe el registro (p. ej. `tc2 import --hito` o `tc2 hito abrir --desde <informe>`) y que `no-necesario` pueda crearlo con `origen: "actualizar"`.

### MEN-1 — «las 4 lineas de `verificar.bat`»: son 4 huecos (tareas), no 4 lineas

Linea 178-180 dice «las 4 lineas de `verificar.bat` se retiran solas»; el log muestra 4 tareas (TASK-100..103) con rangos de 3, 3+3+4, 1 y 1 lineas. Con la 2.0.0 prevista el bloque pasa de 7 a 3 tareas. Cifra de unidad equivocada, el numero (4) sale del log. Ademas: de 15 lineas «solas» no se demuestra que sean justo esas 4 tareas.

### MEN-2 — la PoC no borra su carpeta de trabajo al terminar, el ADR dice que si

Linea 64: «(borrada al terminar; ...)». Tras `python -I -B` completo la carpeta `.../poc/` quedo con `estado-prueba` y la copia `odt` (las borre yo a mano). El guion solo la borra **al empezar** (`poc-f1-18.py:777-778`). Dos cambios posibles: borrar tambien al final (`finally`) o redactar «se borra al empezar la siguiente ejecucion; quien la lanza la borra al terminar».

### MEN-3 — motivo de la descarte de N2 atribuido a Carlos sin fuente en el repo

Linea 41: «Descartada por Carlos: en un proyecto existente es donde mas se hace a mano (PoC A: 9 huecos)». La decision (nombre «Hito de decisiones (ADR)», no «Hito 0») esta en `tarea.md` y en el plan; el motivo «donde mas se hace a mano» y la cifra «9 huecos» (que ademas es con la 2.0 de hoy, 5 con la 2.0.0 prevista) son del redactor. Reformular como «motivo del ADR» o citar la fuente de Carlos.

### MEN-4 — la parada local de `start` solo esta medida con carril ausente

`tareas con carril declarado ... : 0` (log): la parada local se prueba con todas las tareas «no errata» por falta de campo; el caso `declarado=errata` en `start` no tiene medida (se apoya en F1-10 y en el check). Anadirlo a «No verificado».

## Lo que reproduje

- **PoC completa** (`python -I -B ... poc-f1-18.py` con trabajo en `$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-094-r1/poc`): exit 0, 214 s. Log **identico** al entregado salvo los finales de linea (el entregado en el arbol de trabajo tiene CRLF; en el indice es LF), `RESULTADO: todo como se espera`, 69 cifras en la tabla y 69 medidas, 0 errores del ADR y las 11 mutaciones de control fallan cada una con su motivo (cifra-cambiada, fila-borrada, fila-sin-medida, tiempo-de-reloj, resumen-inventado, id-inexistente, enlace-a-tareas, enlace-roto, seccion-borrada, sin-fila-f1-18, estado-propuesta).
- **Cifras H recalculadas con un script propio** (distinto del de la PoC, leyendo `git show 0cc5bc3:tareas/04-terminadas/TASK-00x/tarea.md` de TASK-002..TASK-013): minutos Real suma 739, mediana 45,5, p75 50, rango 12-165; estimados 895; 3 de 12 con Real > estimado; rondas `1:3, 2:4, 3:4, 4:1`; tokens total suma 237266233, mediana 13942446,5, p75 19081290; sin diseno mediana 13503447, p75 18502746; salida mediana 147315,5, p75 204929. Todo coincide con el ADR. Aritmetica derivada comprobada: media 61,6 mayor que 9 de 12 minutos (PR2), 94,6 veces total/salida, presupuestos 2 ADR (91-100 min; 27,88-38,16 M) y 7 tareas (392-539 min).
- **OpenDockTrading** (clon `--no-hardlinks` en `8141a08`, original solo leido): 96 tareas, 44 con `## Coste`, 34 con «Real», minutos suma 2965, mediana 56, p75 77, tokens por tarea mediana 19383656,5 y p75 27935332. Coincide con el ADR.
- **F1-10/F1-11/F1-12**: los 8 campos sin uso en OpenDockTrading incluyen `tipo` (F1-12:337) y F1-9 da `6 / 0 / 0` para `tipo`; el ADR los cita bien. El estado calculado, la puerta en el check `estado` (no en un hook; el atajo PreToolUse solo llama a `tc2 hito estado --json`) y la firma de persona estan en el diseno como pide el plan (D1a..D5a); la unica desviacion es IMP-1 (se puede obtener `cerrado` sin firma) y IMP-2 (codigo).
- No hay fila F1-18 en el plan y el ADR la propone (D5a); las divergencias estan anotadas (10).
- Carpeta temporal borrada; no se ejecuto nada dentro de OpenDockTrading ni se usaron GitHub, push ni commit.

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-094/poc/poc-f1-18.py` y `poc-f1-18.log`, `docs/adr/F1-2`, `F1-4`, `F1-9`, `F1-10`, `F1-12` (y salidas de `git show` de TASK-002..TASK-013 en `0cc5bc3` y del clon de OpenDockTrading): los pidio la tarea del encargo para reproducir y contrastar.

> Nota del orquestador: el revisor escribio `aprobada-con-correcciones` con tres IMPORTANTE abiertos; la skill de revision solo admite ese veredicto sin CRITICO ni IMPORTANTE abiertos, asi que se corrige a `cambios-solicitados` y las correcciones (283d19c) se verifican en la ronda 2.
