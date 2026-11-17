# Informe de revision — TASK-032 (ronda 1)

- Commit revisado: 5a5a9e9e08d7cc4180d8dd4ee3760a24ec78677a (el clon estaba en ec47b9f, que solo anade la peticion; el codigo es el mismo)
- Revisor: code-quality-reviewer
- Veredicto: PENDIENTE (escribelo con: taskctl veredicto TASK-032 aprobada | aprobada-con-correcciones | cambios-solicitados)

Propuesta del revisor: **aprobada** (solo hallazgos MENORES).

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/tablero.rs:706 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/tablero.rs:207 | plan |
| MEN-3 | MENOR | abierto | docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md:8 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### MEN-1 — el test de C3a no discrimina el orden de `listas`

Reproduccion: en `crates/tc2/src/tablero.rs:136` se cambia `s.listas.iter().filter_map(fila)` por
`s.listas.iter().rev().filter_map(fila)` y se corre `cargo test -p tc2 --lib board_next_y_roadmap`.
Resultado: **sigue verde** (1 passed). El estado de referencia solo tiene una tarea lista, asi que
`assert_eq!(lista(&board, "listas"), lista(&next, "listas"))` no puede ver un cambio de orden. El
criterio C3a dice «las listas de `board` son las de `next`». El codigo actual es correcto; falta
red de regresion para el orden de `listas` (con 2 o mas listas en el estado de referencia, o un
test propio). Es MENOR porque `en-marcha` si queda cubierto (la mutacion del «quien» pone el test
en rojo) y el codigo hoy esta bien.

### MEN-2 — el texto de `board` y `status` sale en espanol con `TC2_IDIOMA=en`

Reproduccion: en un clon con rama de estado, `TC2_IDIOMA=en tc2 status` da
`en diseño: TASK-006 Carlos G · en curso: ...` y `TC2_IDIOMA=en tc2 board --sin-plataforma` da
`Tablero · commit ... En diseño (1) ... ninguna ... Terminadas: 1 · Descartadas: 0`. Los avisos de
stderr si salen traducidos (`warning: pull requests were not read...`). El motivo es razonable:
C1 exige el mismo `board --json` en los dos idiomas y `texto=` va dentro del JSON. Pero el texto
sin `--json` no tiene esa restriccion y podria traducirse (solo `texto=` tendria que quedarse
fijo). Queda mezcla de idiomas en una misma ejecucion en ingles. MENOR: no rompe ningun criterio;
conviene decidirlo (traducir o dejarlo dicho en la tarea de i18n).

### MEN-3 — la adenda de F1-4 atribuye a `status` avisos en stderr que no escribe

`docs/adr/F1-4-...md:8` dice «`board` y `status`, como `roadmap`, escriben avisos de una linea en
stderr con codigo 0». `status` nunca escribe en stderr (el propio test C3c asevera
`stderr == ""` en `status`) y no consulta la plataforma; solo `board` los escribe. Texto
inexacto en un ADR; se arregla con «`board`, como `roadmap`,».

## Verificacion de criterios (en clon limpio de la rama)

Suite completa: `cargo test --workspace` en el clon, 0 fallos (302 tests pasan en los binarios de
tests, incluidos `tablero` y `prs_abiertos`). La peticion no traia verificacion de ronda, asi que
la corri yo.

- C1: `tests/tablero.rs::dos_clones_dan_lo_mismo` existe, prueba lo que dice (dos clones, uno con
  cache de identidad, `tareas/` montada y un `tarea.md` cambiado sin publicar, idiomas es/en,
  `board --json` y `status --json` iguales byte a byte; ademas `commit=` igual al de `next` y
  `roadmap` con el binario). Pasa. Mutacion: anadir a `status_en` una clave que depende del clon
  (`clon=<ruta>`) pone el test en rojo (tablero.rs:214).
- C2: `board_y_status_no_escriben_nada` existe y compara `status --porcelain --ignored` del clon y
  de `tareas/`, `for-each-ref`, arbol de `develop` y `.git/taskcodetwo/` (contenido), tras los seis
  modos; `board --escribir` da codigo 2 y no existe `docs/BOARD.md`. Pasa. Mutacion: `board`
  escribe un fichero en `tareas/` -> rojo (tests/tablero.rs:320).
- C3a: `tablero::tests::board_next_y_roadmap_dicen_lo_mismo` existe, compara contra `next` y contra
  el texto de la tabla de `roadmap`, con los tres casos de «quien». Pasa. Mutacion del «quien»
  (`quien(None, ..)`) -> rojo. Mutacion del orden de `listas` -> **verde** (MEN-1).
- C3b: cubierto por `board_cruza_los_pr_y_avisa` (unitario con `Plataforma` falsa) y por el e2e
  contra el mock (`board_con_la_plataforma_de_produccion`: PR en borrador cuenta como sin PR
  listo, avisos en stderr con codigo 0). Pasan. No mute aqui; la autorrevision declara la mutacion
  `min_by_key`->`max_by_key`.
- C3c: `status_no_construye_adaptador_y_board_aguanta_los_fallos` cuenta las llamadas a `conectar`
  (0 en `status` y en `--sin-plataforma`, 1 en `board`), y recorre los cuatro valores de
  `plataforma=` con codigo 0, estado entero, sin `prs`/`revision-sin-pr`/`pr-sin-revision` y una
  sola linea en stderr. Pasa. Mutaciones: `status` llama a `conectar` -> rojo
  (tests/tablero.rs:364); `board` degradado calcula pendientes con lista vacia -> rojo
  (tests/tablero.rs:418).

## Prueba a mano

Repo de prueba: remoto bare, clon, rama `taskcodetwo/tareas` publicada con 6 tareas (terminada,
en curso, dos en revision, planificada, en diseno con asignado heredado «Carlos G»). Ejecutado el
binario `tc2` del clon de revision, en es y en.

- `status` / `status --json`: `en diseño: TASK-006 Carlos G · en curso: TASK-002 ana · en revisión: TASK-003 ana, TASK-004 · listas para empezar: TASK-005 · commit affdf82349b2`; el JSON da las cuatro listas y `commit`, y `ilegibles` solo si los hay. Codigo 0.
- `board --sin-plataforma`: tabla por grupo con ID, titulo recortado a 60 caracteres con `…`,
  quien (`ana`, heredado tal cual, `—`) y recuento de terminadas/descartadas; `plataforma no-consultada`.
- `board` sin remoto reconocible: mismo cuerpo, `plataforma no-configurada` y un aviso de una linea
  en stderr que dice que hacer («configura la clave plataforma o usa --sin-plataforma»), codigo 0.
  La rama con PR reales (borradores, `prs=`, `revision-sin-pr`) la cubre el e2e contra el mock; a
  mano no la repeti (no hay GitHub real).
- Errores: `board --extra` y `status x` dan codigo 2 con la linea de uso; sin rama de estado da
  `rechazada motivo=sin-rama-de-estado`, codigo 3, como `next`.
- Lectura para un equipo: sirve. En una pantalla se ve quien lleva que y que espera revision; los
  avisos van a stderr y el JSON da el mismo dato en claves planas. Limitacion conocida: `status`
  no dice cuantas tareas hay en total ni muestra PR (por diseno, D1).
- Nota de fixture: dos tareas mias salieron como `ilegibles` por un `rama: null` en el
  `tarea.md` de prueba; `next` las da igual, es comportamiento previo a esta tarea, no un hallazgo.

## Desviaciones del implementador

- `quien=ID:<texto>` en vez de `ID@login`: acertada. `texto()` ya incluye `@` para cuentas y
  admite heredados («Carlos G», codificado con `%20` en la linea de contrato), con el mismo texto
  que `roadmap`. Sin ambiguedad: se parte por el primer `:`.
- `en-diseno=` (sin tilde) como clave: coherente con las demas claves ASCII del contrato.
- Claves extra (`terminadas`, `descartadas`, `ilegibles`, `borradores`, `avisos`, `texto`):
  aceptable; son planas y las de pendientes solo salen si hay consulta hecha y valor. `texto=`
  duplica la vista completa dentro del JSON (peso, pero es lo que permite C1 con una sola cadena).
- PR en borrador: cuenta como «sin PR listo» (`revision-sin-pr`) y no como `pr-sin-revision`.
  Decision de dominio razonable y consistente en codigo, texto (`#15 (borrador)`) y test e2e.
- Eleccion de rol (persona si hay token y terminal, si no agente): es una eleccion previa y no un
  reintento; la comprobacion de terminal vive en `CredencialesEntorno` (credenciales.rs:80). El
  MENOR de la autorrevision (no comprueba lista de personas) lo comparto como MENOR aceptable
  para una lectura.
- Pasos 2 y 3 del plan juntos: no cambia el resultado; el contrato final es el del plan.

## Adenda de F1-4 y F1-13

F1-13 (adenda y fila de la tabla de endpoints): coincide con el codigo (rol revisor rechazado,
paginacion con `MAX_PAGINAS`, forks descartados por `head.repo.id`/`source_project_id`, borrador
por `draft`/`work_in_progress`, sha de 40 hex, rama que no pasa `rama_valida` se salta). Dice con
honestidad que GitLab se comprobo contra la documentacion y no contra la API real, y que el tope
de 30 s es por llamada. F1-4: la fila de la tabla es correcta; la frase de stderr es inexacta
para `status` (MEN-3).

## Lecturas fuera del contexto permitido

- `crates/tc2/src/tablero.rs`, `crates/tc2/tests/tablero.rs`, `crates/tc2/src/plataforma/{github,gitlab,mod,conexion,credenciales}.rs`, `crates/tc2/src/situacion.rs`, `crates/tc2/src/lib.rs`: leidos para reproducir y mutar los criterios C1..C3c y valorar las desviaciones; la peticion solo daba el diff de los dos ADR.
- `tareas/04-terminadas/TASK-001/tarea.md`: solo para copiar el formato de un `tarea.md` en el repo de prueba.
