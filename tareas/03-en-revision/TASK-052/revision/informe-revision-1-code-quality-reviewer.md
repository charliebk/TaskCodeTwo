# Informe de revision — TASK-052 (ronda 1)

- Commit revisado: 8822e8ee1e21657de2fd037cc0d334b27fe58006
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | .github/workflows/revision-ia.yml:363 | implementacion |
| MEN-1 | MENOR | corregido | crates/tc2/src/hallazgos/ronda.rs:110 | implementacion |
| MEN-2 | MENOR | corregido | docs/ci/revision-ia.md:149 | implementacion |
| MEN-3 | MENOR | corregido | crates/tc2/src/hallazgos/ronda.rs:199 | plan |
| MEN-4 | MENOR | corregido | crates/tc2/src/hallazgos/ronda.rs:284 | implementacion |
| MEN-5 | MENOR | corregido | .github/workflows/revision-ia.yml:303 | plan |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Alcance: pase generico completo sobre mis 9 ficheros (workflow, esquema, guia y adendas). Verificacion de la ronda en verde en 8822e8e (peticion): no repetida la suite. Ejecutado por mi en un clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-052-r1-calidad`: `cargo test -p xtask` (34 ok) y `cargo test -p tc2 --test hallazgos` (6 ok). No se ejecuto nada de red ni workflows de GitHub; el paso «Ronda anterior» no se pudo correr entero (necesita `gh`), asi que se probo por partes: `tc2 hallazgos anterior|ronda` con volcados fabricados y `git`/`jq` reales sobre un repo de juguete.

Probado y sin hallazgo: volcados de check-runs vacios, `[]`, HTML, `{"message":...}` y `{"check_runs":null}` hacen que `tc2 hallazgos anterior` salga con codigo 1 y, con `set -euo pipefail`, el paso aborta (falla en cerrado). Check de otra App: ignorado. Check con paginas de anotaciones ausentes: error `ANTERIOR_INCOHERENTE`. `jq -j ... "\u0000"` emite NUL real (separador correcto para `--nul`). Mutaciones sobre `decidir` (quitar la condicion de ficheros con MENOR) y `repartir` (quitar el respaldo por ruta) ponen rojo la suite.

### IMP-1 — el diff incremental arrastra lo que entra por un merge de la base en la rama

Fichero: `.github/workflows/revision-ia.yml:363` (igual en `:335`, paso «Ronda anterior»).

En modo incremental el rango es `<sha_anterior> <head>` (dos puntos). Si entre la ronda anterior y el head se trajo la base a la rama (merge de `develop`, o el boton «Update branch» de GitHub), el diff incluye los cambios ajenos de la base. La ronda 1 usa `base...head` y los excluye; la incremental, no. Efectos: (1) cada revisora ve y revisa codigo que no es del PR, con el gasto del tope de diff y hallazgos validos contra ese diff que ponen el check en rojo por codigo ajeno; (2) `rutas.txt` incluye esos ficheros y distorsiona la seleccion de revisoras y S13; (3) `decidir` cierra como cambiados los MENOR de ficheros que solo cambio la base, y puede disparar `ligera`/`incremental` equivocadamente.

Reproduccion (repo de juguete, mismos comandos que el workflow):

````
git init -b develop; echo 1 > pr.rs; echo 1 > otro.rs; git add .; git commit -m base
git checkout -b feat; echo 2 >> pr.rs; git commit -am A          # commit de la ronda anterior
git checkout develop; echo ajeno >> otro.rs; git commit -am "develop avanza"
git checkout feat; git merge --no-edit develop; echo 3 >> pr.rs; git commit -am B
git rev-list --first-parent develop..HEAD | wc -l                 # 3: A esta en la cadena, el fallback no se usa
git diff --name-only -z --no-renames A HEAD | tr '\0' ' '         # -> otro.rs pr.rs   (otro.rs es de develop)
git diff --name-only A HEAD -- pr.rs                               # lo unico del PR: pr.rs
git diff --name-only develop...HEAD                                # -> pr.rs
````

Arreglo posible: restar lo que ya estaba en la base (por ejemplo, ficheros de `base...head` intersectados con los del rango y diff acotado a ellos, o `git diff A HEAD` con `--` sobre las rutas de `base...head`), o documentarlo como limite conocido. Nota: la rama de «el sha anterior no es ancestro» es codigo muerto en el workflow porque `rev-list` solo da ancestros del head; lo que cubre un rebase es que el check no se encuentra (ronda 1).

### MEN-1 — varias ejecuciones del check en el mismo commit: la ultima `incompleta` borra la ronda anterior

Fichero: `crates/tc2/src/hallazgos/ronda.rs:110` (`revision_de_origen`, usada por `elegir`).

`elegir` mira solo el check mas nuevo de la App por commit. Si en el commit A hay un `rojo` (ronda 1, un IMPORTANTE) y luego una repeticion del workflow en A da `incompleta` (justo el caso documentado de «sin saldo en la API»), `elegir` salta A, no encuentra anterior y la ronda siguiente sale `completa` con `abiertos=0`: el IMPORTANTE se olvida (solo reaparece si el modelo lo vuelve a encontrar). Ademas el workflow pide `check-runs` sin `filter=all`, y el valor por defecto de la API devuelve el ultimo por nombre, de modo que el rojo anterior ni siquiera llegaria en el volcado (esto ultimo, comportamiento de GitHub, no verificado sin red).

Reproduccion (verificada con `tc2` real, `--origen 7`): volcado con `check_runs: [ {id 9, app 7, "revision-ia: incompleta (...)", completed}, {id 8, app 7, "revision-ia: rojo (ronda=1 criticos=0 importantes=1 menores=0)", completed} ]` ambos con el mismo `head_sha`:

````
tc2 hallazgos anterior --origen 7 --check-runs a.json   ->  hallazgos ok anterior=ninguno
tc2 hallazgos anterior --origen 7 --check-runs b.json   ->  hallazgos ok anterior=8 sha=bbbb...   (solo el rojo)
````

Arreglo posible: que `elegir` recorra todos los runs de cada commit (mas nuevo primero) y quede con el primero valido, y pedir `filter=all` en el volcado; un test con dos runs en el mismo commit.

### MEN-2 — la guia y la adenda de F1-8 afirman que sin `checks: read` el check sale `incompleta`

Fichero: `docs/ci/revision-ia.md:149` (tambien `:375`/`:384` en parte, y `docs/adr/F1-8-seguridad-modelo-de-amenazas.md:830`).

Segun el comportamiento documentado de GitHub, si un reutilizable pide un permiso que el llamador no concede, el run entero falla al validarse («nested job ... is requesting 'checks: read', but is only allowed 'checks: none'») antes de arrancar ningun job. Entonces `publicar` no corre y no se publica ningun check `revision-ia` (ni `incompleta`). Es relevante porque todos los llamadores ya existentes (que fijan el reutilizable por SHA) rompen al subir el SHA hasta que añadan el permiso. La guia deberia decir lo que realmente pasa y como se ve («el workflow de revision no arranca: el check no aparece»). No verificado (sin red), pero la afirmacion actual tampoco esta verificada y el E2E previsto en el plan no la cubre.

### MEN-3 — `ligera` da verde sin ninguna revision de un cambio de codigo

Fichero: `crates/tc2/src/hallazgos/ronda.rs:199`.

Decision 4 del plan (consciente): con anterior `verde` y cambios solo en ficheros con MENOR abiertos, la ronda es `ligera`, sin modelo, verde, y esos MENOR se dan por cerrados sin comprobar que se arreglaron. Como casi cualquier fichero acaba con un MENOR, cualquier cambio posterior en ellos (incluido uno que introduzca un CRITICO) pasa verde sin que nadie lo mire. Los «Limites conocidos» de la guia no lo mencionan. Reproduccion por lectura y por el test unitario `decidir_aplica_tope_ligera_e_incremental` (verde con `["b.rs"]` cambiado y MENOR en `b.rs`). Sugerencia: anotarlo en «Limites conocidos» (y valorar limitar `ligera` a cambios en la linea del MENOR o a diffs pequeños).

### MEN-4 — falta test de los elementos `* ` en `## Rondas extra`

Fichero: `crates/tc2/src/hallazgos/ronda.rs:284`.

La ADR F1-16 documenta `- ...` o `* ...`, pero el test `repartir_y_rondas_extra` solo usa `- `. Mutacion: quitar `|| l.starts_with("* ")` deja `cargo test -p tc2 --lib repartir_y_rondas` en verde (la proteccion no tiene test que se ponga rojo al quitarla).

### MEN-5 — un check publicado con anotaciones incompletas bloquea la ronda para siempre

Fichero: `.github/workflows/revision-ia.yml:303` (paso «Ronda anterior», `tc2 hallazgos ronda`).

Si un `PATCH` de lote falla a medias (limite ya reconocido en la guia), ese check queda `rojo`/`verde` con menos anotaciones que su titulo. Las rondas siguientes lo eligen siempre (las `incompleta` que publican despues se saltan), `tc2 hallazgos ronda` da `ANTERIOR_INCOHERENTE` y el paso falla en cada push: no hay forma de avanzar salvo un force-push que reescriba el commit. El mensaje dice «vuelve a lanzar el workflow», que no ayuda en este caso. Reproduccion: `tc2 hallazgos ronda --origen 7 --check-runs b.json --anotaciones '[]'` -> `rc=1` con ese mensaje, de forma determinista. Sugerencia: que `elegir` pueda saltar un check incoherente al anterior (con aviso), o documentar la salida.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/hallazgos/ronda.rs`, `crates/tc2/src/hallazgos/orden.rs` y `crates/tc2/src/hallazgos/mod.rs` (`veredicto_ronda`, `dictamen_publicado`): para reproducir el comportamiento de `tc2 hallazgos anterior|ronda` que el workflow invoca (la guia y el workflow delegan en ellos) y probar los casos con volcados malformados.
- `crates/tc2/src/i18n/catalogo/hallazgos.rs` (solo la busqueda del texto `ANTERIOR_INCOHERENTE`).

Veredicto propuesto: aprobada-con-correcciones
