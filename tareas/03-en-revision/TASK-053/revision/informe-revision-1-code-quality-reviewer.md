# Informe de revision — TASK-053 (ronda 1)

- Commit revisado: bc834c8d5e302c27d69f32e2d7ac3de286836974
- Revisor: code-quality-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | xtask/src/workflows.rs:2299 | implementacion |
| MEN-2 | MENOR | abierto | docs/ci/revision-ia.md:29 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Sin CRITICO ni IMPORTANTE. Dos MENOR, reproducidos.

### Puerta y pruebas ejecutadas

- Verificacion de la ronda (`cargo xtask verificar --perfil rapida`) en verde en bc834c8, segun la peticion: no repetida.
- Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-053-r1-calidad` en bc834c8; borrado al terminar (sin cambios ni commits; el repo original no se toco).
- `cargo test -p xtask`: 34 passed, 0 failed.
- `cargo test -p tc2 --test inyeccion`: 4 passed, 0 failed.
- Mutacion sobre el codigo, con `inyeccion.rs` como red (cada una restaurada despues):
  - M1: el diff sin `bloque` en `prompt::componer` pone en rojo `el_prompt_encierra_todo_dato_del_pr`.
  - M2: `dato::cerca` fija a cuatro acentos pone en rojo el mismo test.
  - M3: `hallazgos::prosa` sin el filtro Cf pone en rojo `las_respuestas_hostiles_no_cambian_el_veredicto`.
  - Los tres tests discriminan.
- `tc2 hallazgos prompt` con el binario real: diff Latin-1, diff cortado a mitad de un caracter UTF-8, diff con NUL, con CRLF y vacio dan `ok` (rc=0). `--revisora` con salto de linea, `--bytes abc`, `--skill` inexistente y `--salida` en carpeta inexistente dan error limpio (rc=2 o 1), sin escribir prompt. El paso del workflow aborta en cerrado (`set -eu`).
- Paso «Seleccion y prompts» completo en bash: no se pudo reproducir (necesita el checkout `base`, `tc2 revisoras` y el artefacto de F5-4, es decir el runner). Se leyo la logica: `partes=()` con `set -u` es valido en bash 4.4 o superior (ubuntu-latest trae 5.x); un `git show` fallido deja un `skill-$i.md` vacio que no se usa (la condicion es falsa). Marcado como no verificado en ejecucion.
- El catalogo `casos.json` (INY-01 a INY-15) cubre las superficies de la adenda (prompt-ci, check, carpeta-tarea, plan-local). El test `comprobar_catalogo` obliga a que cada id tenga test. El plan solo pide «Bateria de PR maliciosos» y esta cubierta.
- La adenda de F1-8 dice la verdad en lo comprobado: `cerca` (minimo cuatro, +1 sobre el tramo mas largo), `plano`, `componer`, `--prosa` con tope de 32 KiB, y el `awk` y la `valla` desaparecidos (grep sin resultados). Su residual de A9 queda declarado.

### MEN-1 — El lint `datos_inertes_en_el_workflow` se salta con variantes triviales
- Donde: xtask/src/workflows.rs:2299 (`saca_datos_a_stdout`) y :2332 (`datos_inertes_en_el_workflow`)
- Que pasa: el lint solo reconoce el nombre de variable entrecomillado y exacto, y un conjunto cerrado de lectoras. Cada variante sale `false` (no detectada):
  - `cat $respuesta` y `cat ${respuesta}`, sin comillas.
  - `cat "$RUNNER_TEMP/diff-${i}-cortado.txt"`, que no casa con `diff-$i`.
  - `cat "$respuesta" >&2` y `1>&2`: cualquier `>` cuenta como redirigido, y el runner de Actions lee tambien stderr.
  - `grep . "$respuesta"`, `cut -c1-200 ...`, `xxd ...` y `tr a b < "$respuesta"`: no estan en `LECTORAS`.
  - `cat "$respuesta"; echo ok > /dev/null`: el `>` de la segunda orden de la misma linea la da por redirigida.
  - `cat "$salida/prompts/0.md"`: no casa con `/prompts/$i.md`. Tampoco lo detecta la comprobacion de que el prompt lo escribe solo `tc2` (`prompts/${i}.md` o `cp x prompts/0.md`).
- Reproduccion: se anadio un test temporal en el clon que llama a `saca_datos_a_stdout` con las lineas de arriba. Salida: `cat "$respuesta"` da `true` (correcto); `cat $respuesta`, `cat ${respuesta}`, `>&2`, `1>&2`, `grep`, `cut`, `xxd`, `tr <`, `diff-${i}`, `prompts/0.md` y la linea con `;` dan `false`. Test descartado con `git checkout`.
- Impacto: el lint es una red de regresion, no una garantia. La adenda de F1-8 y la guia ya dicen que es heuristico, pero la frase «ningun `run:` saque a stdout la respuesta, el diff o el prompt» promete mas de lo que comprueba. Hoy el workflow no vuelca nada (el lint pasa y la lectura a mano coincide), asi que no hay fallo real. Se acepta si se matiza el texto («lo que ve el lint es literal y entrecomillado»), o se endurece: variable con o sin comillas y con llaves, redireccion solo a un fichero (`>` no seguido de `&`), mas lectoras.

### MEN-2 — La guia conserva descripciones del prompt anteriores a F6-7
- Donde: docs/ci/revision-ia.md:29
- Que pasa: el parrafo del prompt dice que solo el diff va entre cercas, con «minimo tres» acentos, y que el prompt lleva «la linea de la seleccion». Tras F6-7 el minimo es cuatro, el contexto, el informe F5-4, los abiertos, el diff y el `SKILL.md` van cada uno en su cerca, y la seleccion va dentro del contexto cercado. La adenda y el nuevo punto de limites conocidos lo dicen bien; el parrafo de arriba no se actualizo.
- Reproduccion: `sed -n 29,34p docs/ci/revision-ia.md` dice «(minimo tres)». `dato::cerca` devuelve como poco cuatro acentos: test `bloque_nunca_contiene_su_cerca_antes_del_final`, `assert_eq!(cerca(&[]), "````")`.
- Impacto: quien lea la guia para auditar la defensa ve una regla distinta de la que aplica el codigo. Corregir el parrafo (cuatro acentos, todo dato cercado) o remitir a la adenda.

## Lecturas fuera del contexto permitido

- crates/tc2/src/dato.rs, crates/tc2/src/hallazgos/prompt.rs, crates/tc2/src/hallazgos/orden.rs, crates/tc2/src/hallazgos/mod.rs (prosa): necesarios para ejercitar `tc2 hallazgos prompt` y `validar --prosa` y para las mutaciones.
- crates/tc2/tests/inyeccion.rs: para comprobar que cada caso del catalogo tiene test y para elegir los mutantes.
- xtask/src/workflows.rs (lint y su test de mutaciones): es la implementacion del lint cuyo salto se pedia probar.
- docs/plan/PLAN-TASKCODE-2.md (fila F6-7): para comprobar que el catalogo cubre lo que pide el plan.

Veredicto propuesto: aprobada-con-correcciones
