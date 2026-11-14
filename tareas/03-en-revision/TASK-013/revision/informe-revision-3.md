# Informe de revision — TASK-013 (ronda 3)

- Commit revisado: 3b62dcff555fdf7c29e9f5763c0a4d8693e905be
- Revisor: code-quality-reviewer
- Alcance: verificacion acotada al diff de las correcciones
- Veredicto: aprobada

Pase generico y acotado. El entregable es un ADR y un guion de PoC en Python; ningun revisor de dominio casa. El delta revisado es `b706b33..3b62dcf`: `d4799bc` (ADR y guion) y `3b62dcf` (tarea.md). `f08b409` y `aaed4be` solo tocan `tareas/`. Profundidad: IMP-1, IMP-2, MEN-7 y MEN-8 a fondo; la nota de los 594 s y la gate G, reproducidas con las funciones del guion. El resto del ADR no se ha revisado en esta ronda.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-12-actualizacion-entre-versiones.md:277 | implementacion |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-12-actualizacion-entre-versiones.md:477 | implementacion |
| MEN-7 | MENOR | corregido | docs/adr/F1-12-actualizacion-entre-versiones.md:400 | implementacion |
| MEN-8 | MENOR | corregido | tareas/03-en-revision/TASK-013/poc/poc-f1-12.py:1379 | implementacion |
| MEN-9 | MENOR | abierto | tareas/03-en-revision/TASK-013/poc/poc-f1-12.py:1383 | implementacion |
| MEN-10 | MENOR | abierto | docs/adr/F1-12-actualizacion-entre-versiones.md:136 | implementacion |

Sin CRITICO ni IMPORTANTE abiertos. Los dos MENOR abiertos son nuevos o arrastrados y no bloquean el cierre; quedan documentados con su motivo.

La puerta determinista no aplica como en un proyecto de codigo: no hay build, linter ni suite en el repo, y `.taskcode/config.yml` declara que aun no hay comando de verificacion (se declara con F2-1). Lo que se ejecuto en su lugar esta en «Que se reprodujo».

### IMP-1 — corregido (fila `subtree split: tiempo en OpenDockTrading`)

- Donde: la fila de la tabla `cifras` ya no existe (era ADR:277 en la ronda 1). El guion no tiene la funcion de cubos: `grep` de `cubos(` sin resultados; el tiempo solo sale por stderr (`tiempo()`, poc-f1-12.py:78). `menos de 600 s` aparece solo en la cadena de la mutacion `tiempo-de-reloj-en-la-tabla` (poc-f1-12.py:1319), que es el control que debe fallar.
- Que pasa: la gate ya no depende del reloj. Se verifico con `errores_adr` (ver «Que se reprodujo»): con la tabla del ADR como medido salen 0 errores, y la mutacion del tiempo falla como se espera.
- Origen: implementacion.

### IMP-2 — corregido (aprobacion de los nombres)

- Donde: ADR:477 (punto 7 de «Divergencias»); la fila D1a (ADR:47) dice lo mismo sin remitir a la evidencia, pero el punto 7 la cita.
- Que pasa: la evidencia ya existe en el repo. `tareas/03-en-revision/TASK-013/tarea.md:35-37`, seccion «Decisiones de la persona»: fecha 2026-11-14, Carlos, medio «la conversacion de la tarea», cita literal «Aprobamos los nombres» y lo aprobado (`tc2`, `taskcodetwo` con sus usos y el paquete npm con binario `tc2`). Coincide con D1a y con el punto 7.
- Limite: no tengo acceso a la conversacion, asi que la cita no se contrasta desde aqui. La constancia en el repo es lo que pedia IMP-2 y la acepto como tal. Si la persona no confirma esa aprobacion, la afirmacion vuelve a ser falsa.
- Origen: implementacion.

### MEN-7 — corregido (comillas de la config)

- Donde: ADR:400 y divergencia 7 (ADR:575-579).
- Verificado: el guion (`config_formato2`) solo entrecomilla en dobles `modo_flujo` (variante B). `formato: 2`, `comando`, `shell`, `cron`, `derivados` y `confianza_incluye` van en simples o sin comillas. El ADR ya no cita F1-9 para la config, ya no dice «y las demas claves base heredadas» (`grep` sin resultados) y cita F1-11 (decision 1) para las simples, que es lo que fijaba la ronda 2.
- Origen: implementacion.

### MEN-8 — corregido (guard de `comprobar_trabajo` sin raiz)

- Donde: poc-f1-12.py:1379-1380.
- Verificado: la raiz `os.path.join(LOCALAPPDATA, "pruebas-claude")` se normaliza y la ruta tiene que empezar por `raiz + os.sep`. Se reprodujo con la funcion aislada y con la PoC por CLI (tabla en «Que se reprodujo»): la ruta del MEN-8 (`C:\Users\nullcad2025\Documents\pruebas-claude\a\b`) se rechaza; la raiz y `pruebas-claude\taskcodetwo` se rechazan; `task-013-r3-noexiste` se acepta y no se crea.
- Origen: implementacion.

### MEN-9 — abierto (mayusculas distintas en las dos comprobaciones; mensaje de la raiz)

- Donde: poc-f1-12.py:1383 (`if "pruebas-claude" not in partes`) y :1379-1380 (la raiz se compara sin distinguir mayusculas).
- Que pasa: la comprobacion de la raiz usa `normcase`, pero la de las partes distingue mayusculas. Una ruta valida en Windows como `%LOCALAPPDATA%\PRUEBAS-CLAUDE\TaskCodeTwo\x` pasa la raiz y se rechaza despues con «tiene que estar bajo pruebas-claude/<proyecto>/<motivo>», que es un mensaje falso. Ademas, la propia raiz da «no esta bajo %LOCALAPPDATA%/pruebas-claude» en lugar de «hacen falta dos niveles», y la nueva comprobacion deja ese mensaje algo confuso.
- Reproduccion: `comprobar_trabajo(LOCALAPPDATA + r"\PRUEBAS-CLAUDE\TaskCodeTwo\x\\")` → `ERROR: la carpeta de trabajo tiene que estar bajo pruebas-claude/<proyecto>/<motivo>`.
- Impacto: rechazo indebido en Windows (fail-closed), sin borrado de nada ajeno. El problema viene de la ronda 2; la correccion no lo introduce, pero deja dos criterios de mayusculas distintos.
- Sugerencia: hacer la comparacion de partes sin distinguir mayusculas (o calcular la parte relativa a la raiz con la ruta normalizada) y dar un mensaje propio para la raiz exacta.
- Origen: implementacion.

### MEN-10 — abierto (los 594 s siguen figurando como cifra observada)

- Donde: ADR:136-137.
- Que pasa: el texto nuevo indica bien que el log del commit `7f431ff` solo guardo el cubo: en su linea 29 pone `subtree split: 1151 commits ; tiempo: menos de 600 s`. Pero a continuacion sigue «594 s en la entrega» como parte del rango observado, y ningun fichero del repo contiene 594. La ronda 2 pedia citarlo como «no conservado» o quitarlo del rango; ahora el valor sigue pareciendo medido.
- Reproduccion: `git show 7f431ff:tareas/02-en-curso/TASK-013/poc/poc-f1-12.log` (linea 29) → `menos de 600 s`; `grep -rn 594` en el repo solo devuelve ADR:136.
- Impacto: un lector puede tomar 594 s como medida de la entrega. No afecta a la tabla `cifras` ni a la gate.
- Sugerencia: poner el rango como «614 s y 647 s medidos; la entrega solo dejo constancia de menos de 600 s», o quitar el 594 s.
- Origen: implementacion.

## Que se reprodujo

- **Puerta**: no hay build, linter ni suite en el repo (`ls` de la raiz y `.taskcode/config.yml`). Sintaxis del guion: `ast.parse` sin errores, con `python -I -B` y sin escribir bytecode.
- **G, funciones puras** (`python -I -B`, guion cargado con importlib sin ejecutar `main()`, con el ADR real):
  - Tabla `cifras` del ADR usada como medido: 111 filas. `errores_adr(adr, ruta, repo, medido)` devuelve 0 errores.
  - Mutaciones de `mutaciones(adr)`: las 8 fallan por el motivo esperado (cifra cambiada, fila borrada, fila sin medida, tiempo de reloj en la tabla, resumen con 9999, enlace a `tareas/`, ID `F7-99`, seccion borrada).
- **`comprobar_trabajo` aislada** (LOCALAPPDATA real):

  | Ruta | Resultado |
  |---|---|
  | `%LOCALAPPDATA%\pruebas-claude` | rechaza (no esta bajo la raiz) |
  | `%LOCALAPPDATA%\pruebas-claude\taskcodetwo` | rechaza (dos niveles) |
  | `C:/otro/pruebas-claude/a/b` | rechaza (no esta bajo la raiz) |
  | `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-013-r3-noexiste` | acepta; no se crea |
  | `C:\Users\nullcad2025\Documents\pruebas-claude\a\b` (MEN-8) | rechaza |
  | `%LOCALAPPDATA%\pruebas-claude-x\a\b` | rechaza |
  | `%LOCALAPPDATA%\pruebas-claude\..\pruebas-claude-x\a\b` | rechaza |
  | `pruebas-claude\a\b` (relativa desde fuera) | rechaza |
  | `%LOCALAPPDATA%\PRUEBAS-CLAUDE\TaskCodeTwo\x\` | rechaza con mensaje falso (MEN-9) |

- **PoC por CLI** (`python -I -B poc-f1-12.py <trabajo> <repo> <repo> <repo>`, cada una con rutas que el guard debe rechazar antes de borrar): `%LOCALAPPDATA%\pruebas-claude` y `%LOCALAPPDATA%\pruebas-claude\taskcodetwo` salen con el mensaje de la tabla, sin borrar nada. El inventario de `pruebas-claude` no cambio para estas dos.
- **Incidente de la revision**: en la tercera llamada por CLI pase la ruta `%LOCALAPPDATA%\pruebas-claude\PRUEBAS-CLAUDE\x\y`. Es una ruta valida bajo la raiz (Windows no distingue mayusculas), asi que el guard la acepto: el guion creo `pruebas-claude\PRUEBAS-CLAUDE\x\y` con su marca `.poc-f1-12`, imprimio unas lineas y termino. Los argumentos apuntaban solo al repo de TaskCodeTwo, `git status` queda limpio y HEAD sin cambios. La carpeta creada se borro. Fue un error del revisor al elegir la ruta, no un fallo del guion.
- **No ejecutado**: la PoC completa (~1190 s; no es una comprobacion de esta correccion), nada dentro de OpenDockTrading ni ToolsaWeb, GitHub, push y commits.
- **Limite de G**: la comparacion entre las cifras medidas y la tabla (111 = 111) solo la cita del log (lineas 182-184), porque necesita las secciones N, M, C y A del guion.

## Nota (no en la tabla)

- poc-f1-12.py:1119: `filas0` lee `tareas/ROADMAP.md` antes de cada orden y no solo antes de `roadmap --escribir`, con `open` sin cerrar. Arrastrado de la ronda 2; inocuo mientras el fichero exista.

## Lecturas fuera del contexto permitido

- `git show 7f431ff:tareas/02-en-curso/TASK-013/poc/poc-f1-12.log`: para verificar la linea 29 y el cubo `menos de 600 s` (MEN-10 y IMP-1).
- `tareas/03-en-revision/TASK-013/poc/poc-f1-12.tiempos.log`: para verificar el 647 s citado por el ADR (MEN-10).
- `tareas/03-en-revision/TASK-013/revision/informe-revision-1.md` (linea 24): para verificar el 614 s citado por el ADR (MEN-10).
- `.taskcode/config.yml`: para confirmar que no hay comando de verificacion declarado.
- Skill `task-workflow/revision.md` (plugin 0.16.1): para el formato de la cabecera, la tabla y el veredicto.
