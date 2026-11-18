# Informe de revision — TASK-127 (ronda 1)

- Commit revisado: 7d1963a0d80f19af3edc996e533d258f37d41a69
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/git/estado.rs:267 | implementacion |
| MEN-1 | MENOR | corregido | docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md:578 | implementacion |
| MEN-2 | MENOR | corregido | crates/tc2/src/git/estado.rs:267 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

## Puerta y alcance

- Suite completa: se cita `cargo xtask verificar --perfil rapida` en verde (exit 0, "xtask: verificacion Rapida en verde", log `verif-127.log`) sobre 22ffc96. `git diff --stat 22ffc96 7d1963a` solo toca `tareas/02-en-curso/TASK-127/tarea.md` (+5 lineas): el codigo es identico al revisado. No se repitio la suite.
- Pase generico, profundidad completa sobre los 6 ficheros del diff (puerto.rs, estado.rs, catalogo/git.rs, estado_casos.rs, progreso.rs, ADR).
- Ejecutado por mi en un clon en 7d1963a (borrado al terminar):
  - `cargo test -p tc2 --test estado_casos mil_ficheros`: 1 pasa (60 s).
  - Mutacion A (hash-object una vez por fichero, por stdin): el test de 1000 se pone ROJO (`left: 28, right: 1018`: los procesos crecen con N).
  - Mutacion B (`update-index --add --cacheinfo` por argumentos, con hash-object ya por stdin): ROJO (`1000: estado error git=update-index`; en Windows excede la linea de ordenes). Confirma que el plan acertaba al tratar los dos argumentos que crecen.
  - Mutacion C (quitar a la vez la comprobacion `s.codigo == 0` de `con_entrada` y la de `shas.len() != cambios.len()`): `estado_casos` (6, saltando el de 1000), `estado_bordes` (10), `git_casos_limite` (6) y `estado_concurrencia` (5) siguen VERDES. Ver IMP-1.
  - Comprobado por lectura y grep: `HASH_OBJECT_STDIN`/`UPDATE_INDEX_STDIN` solo se aceptan en su forma exacta; `ejecutar_con_indice_y_entrada` rechaza otro subcomando, otros argumentos e indice relativo antes de lanzar git; los tests unitarios de puerto.rs cubren esos rechazos y el caso `Lanzar` con binario inexistente. El hilo escritor evita el interbloqueo de tuberias; un fallo de escritura se ve en codigo/stderr.
  - `progreso.rs`: el test perdio el rodeo `chunks(100)` y ahora llama a `poner` una vez con las ~1000 tareas (`repo_con(.., 1000, ..)`); es ejercicio real del arreglo (la suite rapida lo paso en verde). Sin hallazgo.
  - Test de 1000 ficheros: discrimina el criterio 1 (en Windows por la longitud; en Linux, por la asercion de "sin rutas en la linea de ordenes") y el criterio 2 (igualdad de procesos entre 10 y 1000).

## IMP-1 — Los caminos de error nuevos de `con_entrada` y el control `HASH_OBJECT_INCOMPLETO` no tienen ningun test

- Donde: `crates/tc2/src/git/estado.rs:267` (`con_entrada`, rama `s.codigo != 0`) y `estado.rs:~345` (`shas.len() != cambios.len()`).
- Que pasa: `con_entrada` es codigo nuevo y su rama de fallo (git sale con codigo distinto de 0 con la entrada por stdin) no la ejecuta ningun test. El plan decia expresamente (apartado "Lo que el rol no cubrio / Riesgos"): "salida de `--stdin-paths` con menos lineas de las esperadas (la cubre `HASH_OBJECT_INCOMPLETO`, se comprueba con un test)"; ese test no esta en el diff y no hay ninguna referencia a `HASH_OBJECT_INCOMPLETO` en `crates/**/tests`. Tampoco hay test de "git muere a mitad del stdin".
- Reproduccion: en el clon, cambiar `if s.codigo == 0 {` por `if true {` en `con_entrada` y `if shas.len() != cambios.len() {` por `if false {` en `construir_commit`. Correr `cargo test -p tc2 --test estado_casos`, `--test estado_bordes`, `--test git_casos_limite` y `--test estado_concurrencia` (saltando el de 1000): 6+10+6+5 pasan, todos verdes. (No se corrieron `init`, `ciclo` ni el resto de ficheros de tests; la busqueda por texto no encuentra tests que fuercen ese fallo.)
- Impacto: la proteccion que impide commitear un arbol con blobs que no se escribieron (si `hash-object` muere con una ruta ilegible o disco lleno, el indice recibiria entradas con `sha` ausente o desplazado) no tiene red de regresion. Con la salida truncada, sin el control, `zip` emparejaria rutas con los hashes equivocados sin error: corrupcion silenciosa del estado. Hoy el codigo es correcto por lectura; lo que falta es que un cambio futuro lo rompa en rojo.
- Sugerencia: un test de integracion con `Git` apuntando a un `git` falso (un script o binario que lee stdin y sale 1, o que imprime solo la mitad de los hashes) o con una ruta de blob que desaparece entre la escritura y el `hash-object`; que compruebe `Resultado::Error` y `HASH_OBJECT_INCOMPLETO`/`GIT_FALLO` en el diagnostico, y que la rama de estado no avanzo.

## MEN-1 — La adenda del ADR dice "tres formas exactas y en ninguna otra", pero `cat-file` no esta restringido a una forma exacta

- Donde: `docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md:578` (ultima linea del fichero) y `crates/tc2/src/git/puerto.rs:300-304`.
- Que pasa: la adenda afirma que el puerto admite stdin "en tres formas exactas y en ninguna otra: `cat-file` (F2-4b), `hash-object -w --no-filters --stdin-paths` ... y `update-index --add -z --index-info`". En el codigo, `hash-object` y `update-index` si exigen la lista exacta de argumentos, pero `Subcomando::CatFile => true`: `ejecutar_con_entrada` acepta `cat-file` con cualquier argumento que pase `comprobar_argumentos` (que para `cat-file` no mira nada). El plan tambien hablaba de `cat-file --batch` como forma exacta. Lo de `cat-file` es anterior a la tarea y es de solo lectura, asi que no es un riesgo nuevo; lo inexacto es la frase, no el codigo.
- Reproduccion: lectura; el test del puerto solo prueba `CatFile` con `["--batch"]` (-> `Lanzar`) y nunca con otro argumento (p. ej. `["--batch", "--textconv"]`).
- Impacto: un lector del ADR cree que hay una lista cerrada de tres formas verificada por la capa, y para `cat-file` no lo hay.
- Sugerencia: decir "`cat-file` (solo lectura, F2-4b) y dos formas exactas: ..." o fijar `cat-file` a `--batch` en el puerto (y anadirlo al test).

## MEN-2 — `con_entrada` reimplementa `git_fallo` y el mensaje de fallo diverge

- Donde: `crates/tc2/src/git/estado.rs:267-304`; el original es `git_fallo` en `crates/tc2/src/git/mod.rs:67` con `SalidaGit::primera_linea_error` (`puerto.rs:142`).
- Que pasa: la rama de error de `con_entrada` construye a mano el mismo `GIT_FALLO` que `git_fallo`, pero la primera linea de stderr la saca sin `trim` y con `unwrap_or_default()`, mientras `primera_linea_error` hace `trim` y usa "sin mensaje". Es la misma decision escrita dos veces; la incoherencia concreta: si git falla con stderr vacio (por ejemplo, muere con una senal, codigo -1), `con_entrada` da un diagnostico con `error` vacio ("git fallo en hash-object: ") y `exigir`/`con_indice` dan "sin mensaje"; y una linea de stderr con espacios al inicio se muestra distinta.
- Reproduccion: lectura (no se ejecuto; la rama no tiene test, ver IMP-1).
- Impacto: diagnosticos distintos para el mismo hecho segun el camino; menor.
- Sugerencia: dar a `SalidaCruda` el mismo `primera_linea_error` (o un helper comun sobre `&str` de stderr) y llamar a un `git_fallo` compartido.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/git/estado.rs`, `crates/tc2/src/git/puerto.rs` y `crates/tc2/src/git/mod.rs` completos (alrededor del diff): para comprobar `git_fallo`, `primera_linea_error`, `comprobar_argumentos` y la rama de `CatFile` (MEN-1, MEN-2).
- `crates/tc2/tests/progreso.rs` (alrededor de `repo_con` y `poner`) y `crates/tc2/tests/comun/mod.rs` (`registro_permitido`): para el test de progreso que perdio el rodeo y los tests de registro.
- `crates/tc2/src/i18n/catalogo/git.rs`: para verificar el mensaje `SIN_ENTRADA` (dice la verdad: lista las tres formas y el indice temporal).
- Ficheros de tests de `estado_*` y `git_casos_limite`: ejecutados para la mutacion C.
- `verif-127.log` (ultimas lineas): para citar la pasada de la suite.
