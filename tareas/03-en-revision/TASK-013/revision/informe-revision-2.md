# Informe de revision — TASK-013 (ronda 2)

- Commit revisado: b706b33020e14c52351e27cd2167c8d2e42b016e
- Revisor: code-quality-reviewer
- Alcance: verificacion acotada al diff de las correcciones
- Veredicto: cambios-solicitados

Pase generico y acotado (el entregable es un ADR y un guion de PoC en Python; ningun revisor de dominio casa). Desde `b706b33` el HEAD de la rama es `7fd5b8d`: los commits posteriores (`f448fc1`, `7fd5b8d`) solo tocan `tareas/` (peticion y marcas), asi que el codigo y el ADR revisados son los de `b706b33`.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-12-actualizacion-entre-versiones.md:277 | implementacion |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-12-actualizacion-entre-versiones.md:476 | implementacion |
| MEN-7 | MENOR | corregido | docs/adr/F1-12-actualizacion-entre-versiones.md:399 | implementacion |
| MEN-8 | MENOR | corregido | tareas/03-en-revision/TASK-013/poc/poc-f1-12.py:1373 | implementacion |

Los MENOR de la ronda 1 (MEN-1 a MEN-6) se comprueban abajo, en su seccion, y estan todos corregidos.

### IMPORTANTE-1 (IMP-1) — corregido

- Donde: docs/adr/F1-12-actualizacion-entre-versiones.md:277 (fila `subtree split: tiempo en OpenDockTrading` de la tabla `cifras`, ya eliminada); prosa en :134-138 y :587-589.
- Que pasa (ronda 1): la gate G comparaba un cubo de reloj (`menos de 600 s`) con la tabla y fallaba al reejecutar (594/614/907 s).
- Verificado en esta ronda:
  - El guion ya no tiene la funcion de cubos ni la medida `subtree split: tiempo en OpenDockTrading` (`MEDIDO`): no queda ninguna referencia a `cubos(` (busqueda en poc-f1-12.py). El tiempo solo sale por stderr (`tiempo(...)`, poc-f1-12.py:652 y :78).
  - El ADR ya no contiene `menos de 600 s` ni `907` en prosa ni en la tabla (busqueda en el ADR). La unica aparicion de `menos de 600 s` es la cadena de la mutacion `tiempo-de-reloj-en-la-tabla` (poc-f1-12.py:1319), que es el control que debe fallar.
  - El rango de la prosa (ADR:134-138: 594, 614 y 647 s) coincide con las fuentes que existen: 614 s en `informe-revision-1.md`, 647 s en `poc-f1-12.tiempos.log` linea 2. El 594 s no tiene fichero de respaldo (ver nota).
- Reproduccion: no reejecute el guion completo (~1190 s). Lo que IMP-1 pedia (que la gate no dependa del reloj) se comprueba por construccion en el codigo y con la ejecucion de la comprobacion pura de G descrita en "Que se reprodujo".
- Origen: implementacion.

### IMPORTANTE-2 (IMP-2) — abierto (nuevo; lo introduce la correccion)

- Donde: docs/adr/F1-12-actualizacion-entre-versiones.md:476 (punto 7 de «Divergencias»: «aprobados por Carlos el 2026-11-14»). La misma afirmacion aparece en :47 (fila D1a de la tabla de alternativas).
- Que pasa: el ADR da por aprobados los nombres `tc2` / `taskcodetwo` por Carlos en una fecha concreta. La version de la ronda 1 decia lo contrario: «Carlos puede confirmarlos o cambiarlos antes de F1-14» (pregunta abierta, que la correccion ha borrado). Ningun fichero del repo recoge esa aprobacion.
- Reproduccion: busqueda `aprob|Carlos|D1|nombre` en `tareas/03-en-revision/TASK-013/tarea.md`: solo aparece `plan_aprobado: true` (el plan, no los nombres). En el historial, `git log --grep=aprob` solo devuelve «plan aprobado» de otras tareas y la ronda 1 no menciona ninguna aprobacion. Esta revision no tiene acceso a la conversacion con la persona: si la aprobacion existe, debe citarse donde corresponda; si no, la afirmacion es falsa.
- Impacto: el ADR de decisiones registra como aprobada una decision que no consta como aprobada. F1-14 (paquete npm `taskcodetwo`, binario `tc2`) se apoyaria en esa constancia.
- Sugerencia: o bien citar la aprobacion con su evidencia (medio y fecha), o bien devolver el texto a «pendiente de confirmacion de Carlos» y dejar D1 en «Preguntas abiertas». Si se aporta la evidencia, el hallazgo pasa a `corregido` sin nueva ronda de codigo.
- Origen: implementacion.

### MENOR-7 (MEN-7) — la regla de comillas de la config cita a F1-9 y se extiende mas alla de lo que hace la PoC

- Donde: docs/adr/F1-12-actualizacion-entre-versiones.md:399 (y la divergencia 7, :574-579).
- Que pasa: el ADR dice que las claves heredadas van entre comillas dobles «como pide F1-9 para las cadenas». F1-9 (docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md:280-285) regula el frontmatter de `tarea.md`, no `.taskcode/config.yml`. Ademas, «las demas claves base heredadas» no tiene ninguna instancia: `config_formato2` (poc-f1-12.py:440-442) solo entrecomilla en dobles `modo_flujo`; `formato: 2` va sin comillas y el resto de cadenas nuevas van en simples.
- Reproduccion: lectura de poc-f1-12.py:434-471 frente a los parrafos citados; F1-11 (docs/adr/F1-11-extensiones-declaradas.md:570) fija las comillas simples para rutas y ordenes, que si coincide con la PoC.
- Impacto: menor. Quien implemente F2-7 puede extender la regla a todas las cadenas de la config, o justificarla con F1-9, que no la cubre.
- Sugerencia: citar F1-11 para las comillas simples y decir que las dobles se usan solo en `modo_flujo`, la unica clave heredada que la PoC entrecomilla; quitar «y las demas claves base heredadas».
- Origen: implementacion.

### MENOR-8 (MEN-8) — el guard de MEN-5 valida la forma del nombre, no la raiz

- Donde: tareas/03-en-revision/TASK-013/poc/poc-f1-12.py:1373-1383 (`comprobar_trabajo`).
- Que pasa: el guard exige la subcadena `pruebas-claude` en cualquier parte de la ruta y al menos dos niveles debajo, pero no exige que la raiz sea `%LOCALAPPDATA%\pruebas-claude`. Una ruta nueva fuera de esa raiz (p. ej. `...\Documents\pruebas-claude\a\b`) se acepta y la PoC la crea. El borrado sigue acotado porque solo borra una carpeta vacia o con la marca `.poc-f1-12`, asi que el riesgo es bajo y por eso es MENOR. Hallazgo leido en el codigo; no lo ejecute con esa ruta.
- Reproduccion: lectura de poc-f1-12.py:1373-1383 (la comprobacion es sobre la lista de partes de la ruta, sin prefijo de `LOCALAPPDATA`).
- Impacto: una carpeta de trabajo fuera de la convencion de la regla de la tarea, sin borrado de datos ajenos.
- Sugerencia: comparar la ruta normalizada con el prefijo `os.path.join(LOCALAPPDATA, "pruebas-claude")`, o pasar la raiz como argumento y comprobarla.
- Origen: implementacion.

## Verificacion de los MENOR de la ronda 1

- MEN-1 (ROADMAP de 94 a 0 filas) — corregido. Medida nueva en el guion (poc-f1-12.py:1119-1132 y :1159); el log la registra para A, B, C y D (log lineas 55, 76, 98, 119: `94 -> 94`, `94 -> 0`, `94 -> 94`, `94 -> 94`) y el ADR la tiene en la tabla (ADR:171 y filas de `cifras`).
- MEN-2 (comillas de la config contradictorias) — corregido en lo principal: la divergencia 7 y la fila de la config ya no se contradicen; la extension de la regla queda como MEN-7 (abierto).
- MEN-3 (ToolsaWeb 403 frente a 404, sin anotar) — corregido. Divergencia 9 (ADR:582-586) frente a `docs/plan/PLAN-TASKCODE-2.md:425` (F9-2 dice 403) y la cifra 404 de F1-9 (`docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md:79`). La suma 234 + 161 + 9 = 404 es coherente dentro del ADR. Los recuentos por carpeta de ToolsaWeb no los reejecute (regla de la tarea: no ejecutar nada dentro de ToolsaWeb); se apoyan en la medida de la ronda 1, que no he podido reproducir.
- MEN-4 (cabecera del guion: tres variantes frente a cuatro) — corregido. poc-f1-12.py:22-23 dice «cuatro variantes».
- MEN-5 (guard de `pruebas-claude` demasiado laxo) — corregido en lo principal: dos niveles y carpeta vacia o marcada. Reproducido (ver abajo); el caso de la raiz fuera de `LOCALAPPDATA` queda como MEN-8.
- MEN-6 («No verificado» no recoge que la retirada de parches no se ejecuta) — corregido. ADR:652-655. El log (linea 34) confirma que los unicos ficheros de codigo cambiados por la migracion son `.gitignore` y `.taskcode/config.yml`.

## Que se reprodujo

- Diff de la correccion: `git diff -M c12ec5ec..b706b33` sobre docs/ y poc/ (ADR +43/-22; poc-f1-12.py con el guard, la medida de filas, la gate pura de G y las 8 mutaciones; log y tiempos).
- Comprobacion pura de G, ejecutada aislada con `python -I -B` sobre el propio guion cargado sin `main()` (script de prueba en el scratchpad de la sesion, no en el repo):
  - ADR real: 111 filas de la tabla `cifras`; 0 errores estructurales (secciones, IDs, enlaces, resumen e indice).
  - Las 8 mutaciones del ADR fallan cada una por su motivo: cifra cambiada, fila borrada, fila sin medida, tiempo de reloj en la tabla, resumen con numero inventado, enlace a `tareas/`, ID inexistente y seccion borrada.
  - Limite: la comparacion entre lo medido y la tabla (111 = 111, 0 errores) no la he reproducido, porque necesita las secciones N, M, C y A del guion (`MEDIDO` se llena en ellas) y la ejecucion completa son ~1190 s. La cito del log: lineas 182-184 (`cifras en la tabla del ADR: 111 ; medidas: 111`, `errores del ADR: 0`).
- Guard de MEN-5 con el guion real (`python -I -B poc-f1-12.py <trabajo> <repo> <Personal> <plugin>`):
  - Trabajo = `%LOCALAPPDATA%\pruebas-claude`: rechazado, exit 1, «hacen falta dos niveles bajo pruebas-claude», antes de borrar nada.
  - Trabajo = `%LOCALAPPDATA%\pruebas-claude\taskcodetwo`: rechazado, exit 1, mismo motivo.
  - Inventario de `%LOCALAPPDATA%\pruebas-claude`: 27664 entradas antes y despues; `opendocktrading`, `toolsaweb` y `taskcode` siguen en su sitio.
  - Con la funcion `comprobar_trabajo` cargada aislada, 7 casos mas: `toolsaweb` (un nivel) rechazado; `toolsaweb\task-403` (no vacia, sin marca) rechazado; rutas con `..` que salen de la raiz rechazadas; `taskcodetwo\task-013-r2-no-existe` aceptada; carpeta vacia aceptada; carpeta con un fichero sin marca rechazada, y el fichero sigue en disco.
  - La carpeta de prueba `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-013-r2` la cree y la borre al acabar; el recuento vuelve a 27664.
- No ejecute: la PoC completa (~1190 s), la suite de la aplicacion (no existe: el entregable es documentacion y un guion), ni nada dentro de OpenDockTrading ni ToolsaWeb. GitHub no se toco. No se hizo push ni commit; el veredicto se ha escrito a mano en la cabecera, porque `taskctl veredicto` es un commit y no estaba permitido en esta ronda.

## Nota (no en la tabla)

- ADR:136: el 594 s de la «entrega» no tiene fichero de respaldo en el repo: `poc-f1-12.tiempos.log` es nuevo y solo guarda 647 s. Si se quiere citar, o se cita como «no conservado», o se quita del rango.
- El rango observado omite el 907 s que el autor reporto en otro clon (la ronda 1 ya señalo que no estaba en ningun log). Quitarlo es correcto.
- poc-f1-12.py:1119: `filas0` lee `tareas/ROADMAP.md` antes de cada orden y no solo antes de `roadmap --escribir`, con `open` sin cerrar. Inocuo en esta copia (el fichero existe), pero si faltara la PoC caeria en una orden que no lo necesita.

## Lecturas fuera del contexto permitido

- docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md (lineas 79 y 280-285): para contrastar el 404 de ToolsaWeb y el alcance de las comillas dobles.
- docs/adr/F1-11-extensiones-declaradas.md (lineas 81 y 570): para contrastar las comillas simples de rutas y ordenes.
- docs/plan/PLAN-TASKCODE-2.md (linea 425): para contrastar el 403 de F9-2.
- informe-revision-1.md, peticion-revision-2.md, poc-f1-12.py, poc-f1-12.log y poc-f1-12.tiempos.log: los pide la peticion o son el objeto de la revision.
- Skill task-workflow/revision.md (plugin 0.16.1): para el formato de la cabecera y del veredicto.

## Veredicto

cambios-solicitados: queda abierto IMP-2 (IMPORTANTE). Si la persona confirma la aprobacion de los nombres con evidencia, o el texto vuelve a «pendiente», IMP-2 pasa a corregido y el resto de la ronda queda aprobado con los MENOR abiertos (MEN-7 y MEN-8) documentados.
