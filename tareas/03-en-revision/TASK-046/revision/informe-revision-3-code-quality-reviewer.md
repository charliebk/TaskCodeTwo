# Informe de revision — TASK-046 (ronda 3)

- Commit revisado: 3d047d4017e9230ebb149eb4284a71ff22d5f883
- Revisor: code-quality-reviewer
- Veredicto: aprobada

Verificacion acotada a las correcciones de `83f83d6..HEAD` (sobre todo `1c9f9e9`, entrada por nombre de los scripts raros en `crates/tc2/src/confianza/cierre.rs`). Se reviso en un clon propio fuera del checkout original, compilado en debug. El HEAD real de la rama (`d41033b`) solo anade la peticion de la ronda, sin cambios de codigo respecto de `3d047d4`.

Motivo del veredicto: el CRITICO-1 queda corregido de verdad (reproduccion abajo) y la correccion no abre un hueco critico. Queda un MENOR nuevo (MEN-7, proteccion sin test) y una nota sobre un limite que ya existia.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/verificacion/ejecutar.rs:52-75 | implementacion |
| CRITICO-1 | CRITICO | corregido | crates/tc2/src/confianza/cierre.rs (`Indice::raros`, `por_nombre`, `calcular`) | implementacion |
| MEN-7 | MENOR | abierto | crates/tc2/src/confianza/cierre.rs:`calcular` (`pendientes.extend(indice.por_nombre(orden))`) | implementacion (introducido por 1c9f9e9) |

Detalle del estado:
- IMP-1: corregido (83f83d6), reverificado
- CRITICO-1: corregido (1c9f9e9), reverificado
- MEN-7: corregido (70f7f92)

### IMP-1

Sin cambios de codigo en esta ronda en `ejecutar.rs`. La suite sigue verde, incluido `hook_mal_escrito_no_bloquea_la_parada`. No se repite la reproduccion de la ronda 2.

### CRITICO-1 — corregido

Reproduccion de la ronda 2, ejecutada con el binario debug del clon, repo git temporal con `LOCALAPPDATA`, `XDG_CACHE_HOME` y `TEMP` propios:

- cmd: `scripts\verificar.bat` con `echo Don't panic & call "%~dp0lib\mi paso.bat"`. `tc2 confiar verificacion.comando` -> `ficheros=2` (antes 1). Se cambia `lib\mi paso.bat` a `@exit /b 7` sin volver a confiar: `tc2 verificar --perfil rapida` -> `confianza=cambiado motivo=sin-confianza`, rc=3, no se ejecuta (antes: `confiado` y se ejecutaba la version cambiada).
- sh (comillas anidadas): linea `. "$(dirname "$0")/lib/mi paso.sh"` en el cierre: `ficheros=2` y, tras cambiar el script, `confianza=cambiado`.

Intentos de romper la entrada por nombre (mismo montaje: confiar, cambiar el raro, `verificar`):

| Caso | Resultado |
|---|---|
| nombre en mayusculas distintas (`call "%~dp0LIB\MI PASO.BAT"` y fichero `mi paso.bat`) | entra, `cambiado` |
| nombre con apostrofo (`it's mine.bat`, referenciado entre comillas) | entra, `cambiado` |
| raro versionado despues de confiar, con el principal sin tocar y cache caliente | `cambiado`; tras `confiar`, `ficheros=2`, y luego `confiado` |
| raro nombrado desde otro raro (`mi paso.bat` llama a `otro paso.bat`) | `ficheros=3`; cambiar `otro paso.bat` da `cambiado` |
| raro nombrado en la propia orden tras un apostrofo (`echo Don't & call "scripts\mi paso.bat"`) | entra (solo lo recoge `por_nombre(orden)`: ver MEN-7) |
| raro sin versionar (no esta en `git ls-files`) | no entra, igual que cualquier fichero no versionado (diseno declarado) |
| nombrado por una parte (`call "%~dp0lib\mi paso"`, sin `.bat`) | NO entra; ver nota 1 |
| espacio escapado en sh (`. ./lib/mi\ paso.sh`) | NO entra; ver nota 2 |
| nombre partido por comillas (`"...\mi "paso.bat`) o por variable (`%N%.bat`) | NO entra; ver nota 2 |

La condicion de la guia es literal ("su nombre de fichero aparece literalmente"). Los casos que no entran no lo cumplen, y ninguno es una regresion de esta ronda.

### MEN-7 — La entrada por nombre sobre la orden no tiene test

- Donde: `calcular`, la linea `pendientes.extend(indice.por_nombre(orden));`.
- Que pasa: mutacion ejecutada (se quita esa linea): `cargo test -p tc2 --test verificar` -> 14 passed, 0 failed. Los dos tests nuevos nombran el raro dentro de `verificar.bat`, no en la orden de `verificacion.comando`. El comportamiento existe y funciona (caso `echo Don't & call "scripts\mi paso.bat"` en la orden: el raro entra en el cierre), pero nada lo protege.
- Impacto: bajo. Es una proteccion sin red de regresion; una refactorizacion futura puede quitarla sin que nada falle.
- Sugerencia: un test en `verificar.rs` con un raro nombrado en la orden detras de un apostrofo suelto, que cambie el raro sin confiar y espere `cambiado`.

## Verificacion ejecutada

- `cargo build -p tc2` (debug): termina con exito.
- `cargo test -p tc2 --test verificar`: 14 passed, 0 failed, 1 ignored (presupuesto pesado, solo a mano con `--release -- --ignored`). Incluye `apostrofo_suelto_no_deja_fuera_un_script_con_espacios` y `comillas_anidadas_no_dejan_fuera_un_script_con_espacios`.
- No se corrio la suite completa del workspace ni se repitieron `desinstalar` y `--lib confianza`: el diff de esta ronda en codigo solo toca `cierre.rs` (cierre) y un comentario en `confianza/mod.rs`.

Mutaciones sobre el clon (cada una revertida con `git checkout`, arbol limpio):

| Mutacion | Resultado |
|---|---|
| quitar `pendientes.extend(indice.por_nombre(orden))` | sobrevive (MEN-7) |
| quitar la relectura en caliente (`if escanea && !indice.raros.is_empty()` -> `if false`) | mata: los dos tests nuevos |
| quitar `por_nombre` sobre el texto leido en frio | mata: los dos tests nuevos |

## Divergencias y notas (fuera de la tabla, no abren ronda)

1. Referencia sin extension (`call "%~dp0lib\mi paso"`, que cmd resuelve por PATHEXT): el cierre no la sigue, ni para un script raro ni para uno normal (`call lib\paso`). Es anterior a esta ronda (el tokenizador exige `.` o `/` y `script()` busca la ruta exacta). La guia no lo lista como limite.
2. Mismo tipo de hueco que el CRITICO-1, en sh: un nombre con espacio escapado con barra (`. ./lib/mi\ paso.sh`), partido por comillas o formado con una variable no aparece literalmente en el texto, asi que el script no entra en el cierre y un cambio posterior se ejecuta sin volver a confiar (probado: `confianza=confiado`). Ya ocurria antes de `1c9f9e9` y la correccion no lo empeora; la guia declara "literalmente", pero conviene anadir a Limites estas formas (espacio escapado, variable) o aceptarlas como riesgo asumido. Lo dejo como nota y no como hallazgo porque no lo introduce el diff de la ronda; decide quien coordina si lo eleva.
3. Un `.bat` guardado en UTF-16 queda sin leer por `String::from_utf8_lossy` (NUL entre letras): ni tokens ni nombres. Anterior a esta ronda; no probado con un fichero real.
4. La relectura en caliente (sin hashear) cuesta una lectura por script del cierre cuando el repo tiene algun raro. La guia declara las cifras (239-306 ms a mano con un raro); no se remidieron.
5. No verificado: macOS y Linux (sensibilidad a mayusculas en `clave_busqueda`); solo Windows 11.

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-046/revision/informe-revision-2-code-quality-reviewer.md`: es mi informe de la ronda anterior, para fijar los casos a repetir (la peticion lo cita).
- `crates/tc2/tests/verificar.rs`: sin ampliar, solo helpers `config_con` y `raro_entra_en_el_cierre` para montar mis repos de prueba igual que los tests.
