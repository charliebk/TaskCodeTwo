# Informe de revision — TASK-046 (ronda 2)

- Commit revisado: 83f83d690e1069f9fa2c28c04630593d1c1fd353
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

Verificacion acotada al diff de las correcciones (`e33879a..83f83d6`). El HEAD de la rama (15a1859) solo anade ficheros de `tareas/` (peticiones e informes), sin cambios de codigo. Se reviso en un clon propio fuera del checkout original. Se compilo con `cargo build -p tc2` (debug) y se ejecutaron `cargo test -p tc2 --test verificar` (12 ok, 1 ignorado: el pesado de presupuesto), `--test desinstalar` (19 ok) y `--lib confianza` (3 ok). No se corrio la suite completa del workspace: la peticion pedia estas tres.

Motivo del veredicto: queda un CRITICO abierto (CRITICO-1), que se reproduce abajo. Hasta corregirlo, la ronda no cierra la tarea.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido (83f83d6) | crates/tc2/src/verificacion/ejecutar.rs:52-75 | implementacion |
| MEN-1 | MENOR | corregido (83f83d6) | crates/tc2/src/verificacion/ejecutar.rs:420-422 y tests | implementacion |
| MEN-2 | MENOR | corregido (83f83d6) | crates/tc2/tests/desinstalar.rs:281-306 | implementacion |
| MEN-3 | MENOR | corregido (83f83d6) | crates/tc2/src/verificacion/ejecutar.rs:128-130 | implementacion |
| MEN-4 | MENOR | corregido (83f83d6) | crates/tc2/src/verificacion/ejecutar.rs:151-189 | implementacion |
| MEN-5 | MENOR | corregido en la guia (83f83d6) | docs/ci/verificacion-reutilizable.md (Limites) | plan |
| CRITICO-1 | CRITICO | corregido (1c9f9e9) | crates/tc2/src/confianza/cierre.rs (`entrecomillados`, `candidatos`) | implementacion |
| MEN-6 | MENOR | aceptado: create_new no es observable desde fuera y el nombre es aleatorio de 128 bits | crates/tc2/src/verificacion/ejecutar.rs:162-167 (`crear_captura`) | implementacion |

Notas por fila:

- IMP-1: corregido. Lo verifico con el binario real y con el test `hook_mal_escrito_no_bloquea_la_parada`.
- MEN-1: corregido. Mutacion de `es_rojo` (quitar `timeout`) y de la cola de 15 lineas (salida entera): en ambos casos falla `hook_bloquea_con_timeout_fuera_de_rapida_y_resume_15_lineas`.
- MEN-2: corregido. Mutacion `if confianza.is_dir()` -> `if false` en `desinstalar/maquina.rs:309`: fallan los dos tests, el de Windows y el de Unix.
- MEN-3: corregido. Con YAML invalido y `--hook`, el aviso ocupa una linea y sale con 0.
- MEN-4: corregido en lo que pedia (nombre aleatorio, cola del final, captura borrada al terminar). La exclusividad de `create_new` queda sin test: ver MEN-6.
- MEN-5: corregido. La guia declara que solo cuentan los ficheros versionados.

### CRITICO-1 — Un apostrofe antes de una ruta entre comillas en la misma linea hace que el cierre no incluya el script, y el guard deja pasar un script cambiado

- Donde: crates/tc2/src/confianza/cierre.rs, `entrecomillados` (busqueda del primer `"` o `'` de la linea, y su pareja). El fallo se abre con la regla de textos entre comillas que introduce la correccion.
- Que pasa: `entrecomillados` toma la primera comilla de cualquiera de los dos tipos y busca su pareja. Si la linea tiene un `'` suelto antes (en cmd es un caracter normal, como en `echo Don't panic & call "%~dp0lib\mi paso.bat"`), el `'` no tiene pareja, el bucle se corta (`break`) y el texto entre comillas posterior no se toma como ruta. Los tokens sin comillas no bastan para ese nombre (tiene espacios). El script queda fuera del cierre y su huella no se compara.
- Reproduccion (ejecutada, binario debug del clon, `LOCALAPPDATA` y `TEMP` propios):
  - Repo `apos`: `scripts\verificar.bat` con la linea `echo Don't panic & call "%~dp0lib\mi paso.bat"`. `tc2 confiar verificacion.comando` -> `confiar ok ... ficheros=1` (solo `verificar.bat`; `lib\mi paso.bat` no entra).
  - Se cambia `scripts\lib\mi paso.bat` a `@echo PASO-MODIFICADO` / `@exit /b 7`, sin volver a confiar. `tc2 verificar --perfil rapida` -> `confianza=confiado`, se ejecuta la version cambiada: imprime `PASO-MODIFICADO` y `resultado=rojo codigo=7`, rc=3.
  - Control, repo `base`: la misma linea sin el apostrofe (`call "%~dp0lib\mi paso.bat"`) y el mismo cambio -> `confiar ok ... ficheros=2` y, tras el cambio, `verificar rechazada ... confianza=cambiado motivo=sin-confianza`, rc=3, no se ejecuta.
- Impacto: en cmd, un script referenciado con ruta entre comillas y espacios puede cambiar sin que `verificar` lo note, y se ejecuta sin volver a confiar. Es justo la proteccion que la correccion dice cubrir. La condicion es estrecha (cmd, un comilla simple suelta antes en la misma linea, ruta con espacios entre comillas), por eso se clasifica CRITICO por el guard que deja pasar y no por frecuencia.
- Sugerencia: no parar en la primera comilla. Probar cada tipo de comilla por separado (`"` y `'`, en sh `'` si es comilla de verdad) o recorrer la linea con una regla de emparejado por tipo, y anadir un test con `echo Don't ... & call "lib\mi paso.bat"` que falle hoy y pase tras la correccion. Mientras tanto, el texto de la guia (`call "%~dp0lib\mi paso.bat"` como ejemplo) no advierte de esta limitacion; conviene anadirla a Limites si no se corrige.

### MEN-6 — La exclusividad de `create_new` en la captura no tiene test

- Donde: crates/tc2/src/verificacion/ejecutar.rs:162-167 (`crear_captura`).
- Que pasa: al cambiar `.create_new(true)` por `.create(true)`, toda la suite de `verificar` sigue en verde (12 ok). Ningun test comprueba que el fichero no se pise ni se siga un enlace.
- Reproduccion: mutacion ejecutada, `cargo test -p tc2 --test verificar`: 12 ok con la mutacion.
- Impacto: bajo. El nombre son 128 bits aleatorios en el temporal del usuario, asi que la colision o el enlace predecible son muy improbables. Es defensa en profundidad sin red de regresion.
- Sugerencia: aceptar como MENOR con su motivo, o anadir un test que cree el fichero con el nombre previsto antes de `--hook` y compruebe que no se pisa. No bloquea.

## Verificacion ejecutada

- `cargo build -p tc2` (debug): termina con codigo 0.
- `cargo test -p tc2 --test verificar`: 12 passed, 0 failed, 1 ignored (presupuesto pesado, solo a mano con `--release -- --ignored`). Exit 0.
- `cargo test -p tc2 --test desinstalar`: 19 passed, 0 failed. Exit 0.
- `cargo test -p tc2 --lib confianza`: 3 passed (incluye `un_glob_invalido_es_error_del_cierre`). Exit 0.

Reproducciones manuales de la ronda 1 con el binario debug, sobre un repo git temporal con `LOCALAPPDATA`, `XDG_CACHE_HOME` y `TEMP` propios:

- `echo '{}' | tc2 verificar --perfil nope --hook` -> `rc=0`, una linea de aviso en stderr (antes rc=2). Igual con `--perfil rapida --hook --foo` y con `--hook` sin argumentos.
- `echo '{"stop_hook_active":true}' | tc2 verificar --perfil nope --hook` -> `rc=0`, `sin-cambios motivo=stop-hook-activo` (el guardia se mira antes).
- Sin `--hook`, `tc2 verificar --perfil nope` -> `rc=2` (el uso sigue siendo 2 fuera del hook).
- YAML invalido con `--hook` -> `rc=0`, una linea en stderr (`stderr` = 1 linea).
- Rojo con `--hook` (`rapida`, 30 lineas, codigo 6): `rc=2`, stderr con el motivo y solo `LINEA-16`..`LINEA-30`; tras la ejecucion no queda ningun `tc2-verificar-*.log` en el TEMP propio.
- Control de cambio sin confiar (repo `base`): `confianza=cambiado`, rc=3, no se ejecuta.

Mutaciones (sobre el clon; cada una revertida y el arbol queda limpio):

| Mutacion | Resultado |
|---|---|
| `es_rojo`: quitar `timeout` | mata: `hook_bloquea_con_timeout_fuera_de_rapida_y_resume_15_lineas` |
| cola de 15 lineas -> salida entera | mata: la misma prueba |
| `create_new(true)` -> `create(true)` | sobrevive (MEN-6) |
| `desinstalar/maquina.rs:309` `if confianza.is_dir()` -> `if false` | mata: los dos tests de `maquina` (Windows y Unix) |

Cada mutacion se ejecuto con el fichero de test concreto (`--test verificar` con filtro o `--test desinstalar maquina`).

## Divergencias y notas

- La guia (`docs/ci/verificacion-reutilizable.md`) sigue fijando la regla de cierre sin la limitacion de CRITICO-1. Hay que anadirla al corregir.
- Permisos del temporal en Unix (`/tmp` compartido): el fichero de captura se crea con los permisos por defecto del proceso. No es un cambio de esta ronda; no se abre como hallazgo.
- No verificado: comportamiento dentro de un Claude Code real (el hook solo se ejecuto como comando con JSON por stdin). Enlaces simbolicos a carpetas y mayusculas en macOS/Linux no se probaron en esta maquina.

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-046/revision/` (plugin): `revision.md` de la skill `task-workflow`, para las reglas comunes de revision.
- `crates/tc2/src/confianza/cierre.rs`: solo la funcion `sha_hex`, para comprobar que el nombre de la captura tiene al menos 32 caracteres.
- `crates/tc2/src/desinstalar/maquina.rs`: la linea 309, para la mutacion de MEN-2.
