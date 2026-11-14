# Informe de revision — TASK-013 (ronda 1)

- Commit revisado: c12ec5ecd741fc30a825d846d728b1b952189e85
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

Pase generico (el diff es documentacion de decision y una PoC en Python; ningun revisor de dominio casa). Profundidad: el ADR se ha contrastado entero contra el log y contra las fuentes reales; el guion de 1354 lineas se ha leido por las secciones M, M2, C, A y G y se ha ejecutado completo.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | abierto | docs/adr/F1-12-actualizacion-entre-versiones.md:277 | implementacion |
| MEN-1 | MENOR | abierto | docs/adr/F1-12-actualizacion-entre-versiones.md:441 | implementacion |
| MEN-2 | MENOR | abierto | docs/adr/F1-12-actualizacion-entre-versiones.md:389 | implementacion |
| MEN-3 | MENOR | abierto | docs/adr/F1-12-actualizacion-entre-versiones.md:536 | implementacion |
| MEN-4 | MENOR | abierto | tareas/03-en-revision/TASK-013/poc/poc-f1-12.py:22 | implementacion |
| MEN-5 | MENOR | abierto | tareas/03-en-revision/TASK-013/poc/poc-f1-12.py:1330 | implementacion |
| MEN-6 | MENOR | abierto | docs/adr/F1-12-actualizacion-entre-versiones.md:640 | implementacion |

### IMPORTANTE-1 — La PoC no es reproducible: la cifra del tiempo del `subtree split` esta en el borde del cubo y mi ejecucion sale con 1
- Donde: docs/adr/F1-12-actualizacion-entre-versiones.md:277 (y :128-132, :222-225); guion en tareas/03-en-revision/TASK-013/poc/poc-f1-12.py:848-853
- Que pasa: el guion convierte los segundos del split en un cubo (`menos de 60/300/600/1200 s`) y la gate G exige que el cubo coincida con el de la tabla del ADR (`menos de 600 s`). El propio ADR admite 594 s en la ejecucion del log y 907 s en otra. Con 594 s a 6 s del corte, la gate es una moneda al aire; la afirmacion «Python ... sale con 0 solo si todo sale como se espera» del log de la entrega solo se cumple por suerte, y el ADR dice a la vez «menos de 600 s» y «907 s en una prueba anterior» (el 907 no esta en ningun log: solo en la prosa).
- Reproduccion: `python -I -B poc-f1-12.py <pruebas-claude/taskcodetwo/task-013-r1/trabajo> <repo> <Personal> ~/.claude/plugins/cache/taskcode-marketplace/taskcode-plugin/0.16.1`, sin cache, sobre el commit revisado. stderr: `subtree split calculado en 614 s (664 s desde el inicio)`, `M (925 s)`, `C (1190 s)`. Salida: `FALLO en G: subtree split: tiempo en OpenDockTrading: el ADR dice 'menos de 600 s' y lo medido es 'menos de 1200 s'`, `RESULTADO: 1 fallos`, exit 1. El resto del log es identico al entregado: `diff poc-f1-12.log mi-log` solo difiere en esas lineas y en el resultado.
- Impacto: el criterio de la tarea («el guion falla si una cifra no coincide con lo medido», precedente F1-11) queda incumplido en la direccion opuesta: el guion falla con datos honestos. Quien lo reejecute en otra maquina o en otro momento no puede distinguir una regresion de ruido, y el ADR presenta como medida una cifra que cambia entre ejecuciones (594, 614 y 907 s).
- Sugerencia: no meter en la tabla `cifras` ni en la gate un valor dependiente del reloj (dejar el tiempo como dato informativo de stderr, o comprobar solo un tope holgado por encima del peor valor visto, p. ej. «menos de 1800 s»), y reescribir el ADR con el rango observado (594 a 907 s, 614 s en la ronda de revision) en lugar de «menos de 600 s». Quitar de la prosa el 907 s o dejar su ejecucion en el log.

### MENOR-1 — «reescribe el `ROADMAP.md` ... con 0 tareas» no esta en el log
- Donde: docs/adr/F1-12-actualizacion-entre-versiones.md:441
- Que pasa: el log solo dice que `roadmap --escribir` escribe `tareas/ROADMAP.md` en la variante A; el contenido resultante no se mide ni esta en la tabla `cifras`.
- Reproduccion: sobre la copia migrada de mi ronda, config A (sin `---`), `taskctl roadmap --escribir` de la 0.16.1: el ROADMAP pasa de 354 a 197 lineas y de 94 filas de tarea a 0 (misma expresion regular que usa la PoC, `RX_FILA_ROADMAP`). La afirmacion es cierta, pero se debe a mi medida, no a la PoC.
- Impacto: afirmacion cierta pero sin respaldo en la evidencia entregada, contra el foco de fidelidad.
- Sugerencia: anadir esa medida (filas antes y despues) a la seccion C y a la tabla `cifras`, o marcarla «no medido».

### MENOR-2 — Comillas de la config: la decision y la divergencia se leen como contradictorias
- Donde: docs/adr/F1-12-actualizacion-entre-versiones.md:389 (frente a :566)
- Que pasa: la fila de la tabla dice que los valores de la 0.16 pasan tal cual y que `verificacion.comando` va entre comillas simples; la divergencia 7 dice que los valores de la 0.16 van entre comillas dobles porque con simples la 0.16 conserva las comillas. En el guion solo `modo_flujo` (valor de una clave de la 0.16) va en dobles; los de claves nuevas van en simples.
- Reproduccion: lectura de `config_formato2` (poc-f1-12.py:434-470) y del log de la variante D (`modo_flujo "'automatico'" no es valido`).
- Impacto: un implementador de F2-7 puede entrecomillar con simples un valor de clave heredada y romper la lectura en la 0.16 (que es justo lo que se quiere frenar de forma controlada, no por accidente).
- Sugerencia: decir en el punto 2 que las claves heredadas de la 0.16 llevan comillas dobles y las nuevas, simples (F1-9 pide dobles para cadenas: verificar que no choca).

### MENOR-3 — Divergencia con el plan sin anotar: ToolsaWeb tiene 404 tareas, el plan dice 403
- Donde: docs/adr/F1-12-actualizacion-entre-versiones.md:536 (seccion «Divergencia con el plan»); plan en docs/plan/PLAN-TASKCODE-2.md:425 (F9-2)
- Que pasa: el ADR usa 404 (:591, :611) y yo cuento 404 en `develop` de ToolsaWeb (234 planificadas, 161 terminadas, 9 archivadas, 0 activas); F1-9 tambien dice 404. El plan (F9-2) dice «403 tareas» y la lista de divergencias, que si anota el recuento de OpenDockTrading (50/44/2 frente a 33/61/2), no lo menciona.
- Reproduccion: `git ls-tree --name-only develop tareas/<NN-estado>/ | grep -c TASK-` en ToolsaWeb (solo lectura).
- Impacto: F9-2 heredaria un criterio de exito («403») que no se cumple con los datos reales.
- Sugerencia: anadir una divergencia 9 (403 frente a 404) o proponer el cambio del criterio de F9-2 junto con las dependencias de D3.

### MENOR-4 — La cabecera del guion dice tres variantes de config; el guion mide cuatro
- Donde: tareas/03-en-revision/TASK-013/poc/poc-f1-12.py:22
- Que pasa: la cabecera dice «con tres variantes de la config formato 2»; la seccion C ejecuta A, B, C y D (linea ~1114) y el ADR habla de cuatro.
- Reproduccion: el log tiene los cuatro bloques `-- config formato 2, variante X`.
- Impacto: documentacion del guion desactualizada.
- Sugerencia: «cuatro variantes».

### MENOR-5 — El guard de la carpeta de trabajo acepta cualquier ruta que contenga `pruebas-claude`
- Donde: tareas/03-en-revision/TASK-013/poc/poc-f1-12.py:1330
- Que pasa: `borrar(trabajo)` hace `rmtree` de la ruta pasada si contiene la subcadena `pruebas-claude`; pasar `%LOCALAPPDATA%\pruebas-claude` (o `...\pruebas-claude\taskcodetwo`) borraria las carpetas de prueba de todos los proyectos. No lo he ejecutado: lo marco como no verificado, con el caso `python -I -B poc-f1-12.py "%LOCALAPPDATA%\pruebas-claude" ...`.
- Impacto: borrado de datos ajenos por un argumento mal escrito; bajo en la practica (las rutas de la memoria del proyecto tienen un subdirectorio de tarea).
- Sugerencia: exigir al menos dos niveles bajo `pruebas-claude` (`<proyecto>/<motivo>`) y que la carpeta no exista o este vacia/lleve un fichero marca de la PoC.

### MENOR-6 — «No verificado» no recoge que la retirada de parches no se ejecuta en la PoC
- Donde: docs/adr/F1-12-actualizacion-entre-versiones.md:640
- Que pasa: la entrada dice que no se ha probado que `verificar.bat` funcione «tras quitar sus 15 lineas», pero la PoC ni siquiera las quita: los unicos ficheros de codigo cambiados son `.gitignore` y `.taskcode/config.yml` (log, linea 34) y las capacidades de la PoC no incluyen `puertas-en-ci`. El «91 / 973» es un calculo sobre ficheros y lineas, honestamente rotulado «2.0.0 prevista» en el informe A, pero el punto 8 de la Decision («se retira solo en el commit de `actualizar/formato-2`») describe un comportamiento sin ejecutar.
- Reproduccion: log, lineas 34 y 114-130; `CAP_POC` en el guion.
- Impacto: el lector cree que la retirada se probo y solo se desconoce su efecto.
- Sugerencia: anadir «la retirada de parches (editar `verificar.bat` y los lectores) no se ejecuta en la PoC; solo se calcula su alcance».

## Que se reprodujo (y coincide con el ADR)

- PoC completa (`python -I -B`, ~1190 s) en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-013-r1\`: log identico al entregado salvo la linea del tiempo (IMP-1). Cifras M, M2, C, A y G: 108 de 108 coinciden (96 tareas, 50/0/0/0/44/2, 1151 commits de split, 96 de 96 con formato 2, 0 diferencias, `merge-base` vacio, 96/96 en commits y en `--follow`, 3 `--follow` ya rotos en `develop`, 192 claves, 2 descartadas con motivo, segunda pasada 0 y 0, vuelta atras con HEAD previo/0 refs/status limpio/remoto sin cambios).
- Mutaciones de control: M2 (dependencia borrada en TASK-002) da codigo 6 con `TASK-002: dependencias`; el control A de convivencia (config sin `---`) deja pasar `roadmap --escribir` y `new` (crea `TASK-001` duplicado) y el hook `pre-tool-merge` sale con 0; las variantes B, C y D cierran. La gate G detecto de verdad un desajuste (el del tiempo).
- Fuente real (solo lectura): OpenDockTrading `8141a08` tiene 96 tareas, 50/0/0/0/44/2 por carpeta, 1151 commits que tocan `tareas/`, 1350 en total, `.gitkeep` en las seis carpetas, `## Descartada` en TASK-015. `where tc`/`tc2` sin resultado; `where taskctl` da 2 rutas (bin y bin\taskctl.cmd del plugin 0.16.1). npm: `tc` ocupado con bin `tc`, `tc2` ocupado sin bin, `tctwo` y `taskcodetwo` 404 (libres). El fallo cerrado con `---` y con listas en bloque: lanzado por la PoC y leido en `dist/src/core/frontmatter.js:62` (`Linea de config invalida (falta ":")`).
- Los 33 recuentos de lineas de la nota de nombres (`taskctl`, `taskcode/tareas`, `.git/taskcode` por ADR F1-1..F1-11) coinciden con mi `grep -c`. Los enlaces, la fila del indice y los IDs citados resuelven.
- Coherencia con F1-1/F1-9/F1-11: el `91 / 973` coincide con F1-11 (lineas 138 y 689); las dependencias del plan (F9-7/F9-8/F4-5, D3) coinciden con las filas 370, 431 y 432 del plan.
- No verificado por mi parte: winget y Homebrew (solo se repitio npm), WSL con iproute2, `--deshacer`, macOS/Linux, GitHub real.

## Lecturas fuera del contexto permitido

- tareas/03-en-revision/TASK-013/poc/poc-f1-12.py y poc-f1-12.log: el encargo de la persona pidio reproducir la PoC.
- docs/plan/PLAN-TASKCODE-2.md (filas F9, F4-5, F2-*) y docs/adr/F1-1..F1-11: para contrastar la nota de nombres, las dependencias de D3 y las cifras citadas.
- OpenDockTrading y ToolsaWeb (solo `git ls-tree`/`rev-list`/`show`) y el plugin 0.16.1 en `~/.claude/plugins/cache`: para contrastar datos con la fuente real y el fallo cerrado.
- tareas/03-en-revision/TASK-013/tarea.md, planificacion/plan-final.md y docs/adr/README.md: permitidas.

## Veredicto

cambios-solicitados: un IMPORTANTE abierto (IMP-1, la gate de la PoC falla al reejecutarla y la cifra del tiempo es inestable). El resto del contenido del ADR esta respaldado por las medidas o marcado como no verificado.
