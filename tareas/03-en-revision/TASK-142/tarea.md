---
id: TASK-142
titulo: "Auditoria 4D: A4 git, rama de estado y toma (9 IMPORTANTE)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: fix/task-142-auditoria-4d-a4-git-rama-de-estado-y-tom
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 3349940
tokens_implementacion: 33945651
tokens_revision: null
creado: 2026-11-23
actualizado: 2026-11-25
dependencias: []
---
## Objetivo

Corregir los 9 casos IMPORTANTE del grupo A4 de `docs/auditoria-4d.md` (commit auditado d82ea9d): 8 Duplicaciones (dos `divergente`) y 1 Disfraz en código en la guarda de argumentos de git. A4-09 va en la tarea de soporte de tests compartido. No se piden los MENOR (A4-11, A4-12). No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.

## Criterios de aceptacion
- [x] A4-01 (`divergente`, va primero): la consulta de ramas de tarea locales y remotas (`git/rama.rs:88-110` ~ `:160-196`) queda escrita una vez. El fallo del remoto se clasifica igual (sinred / error local / denegada) en `git/remoto.rs:175-181` y `git/tag.rs:100-114`, y lo prueba un test con un remoto que deniega y que hoy sale `sinred`.
- [x] A4-02 (`divergente`): el envoltorio «lanzar git y traducir el fallo» (`git/estado.rs:243-265`, `:269-298`, `:135-150` y `git/mod.rs:52-89`) es uno solo, y `leer_blobs` usa `primera_linea_error()`.
- [x] A4-05 y A4-07: `hay_remoto` es `url_origin(..)?.is_some()` (`git/estado.rs:451-460` ~ `:463-473`), y hay un solo analizador de `git status --porcelain -z` (`git/guardar.rs:158-176` ~ `git/mod.rs:216-228`).
- [x] A4-03 y A4-04: las 19 líneas de `intento` y `sin_red` (`estado/transaccion.rs:353-371` ~ `:533-551`) y el informe de un `publicar` fallido (`git/tag.rs:81-91` ~ `git/remoto.rs:516-526`) quedan escritos una vez cada uno.
- [x] A4-06: `crear` y `reanudar_paso` del worktree de tarea (`git/rama.rs:318-380` ~ `:388-456`) comparten el arranque y el bloque de carpeta ocupada.
- [x] A4-08: los ayudantes `rapida`, `escribir_estado` y `rama_en` y el repo sin remoto (`tests/estado_bordes.rs:19-53` ~ `tests/estado_casos.rs:19-44`, `:124-129` ~ `tests/estado_concurrencia.rs:216-220`) quedan en un módulo común de tests.
- [x] A4-10: `es_prohibido` (`git/puerto.rs:613-646`) rechaza las opciones cortas agrupadas que contienen una prohibida, o pasa a una lista permitida como pide F1-4 (decisión 3). `rechaza_cada_argumento_prohibido_sin_lanzar_git` (`puerto.rs:759-846`) incluye `(Push, ["-qf", "origin", "r"])`, `(Branch, ["-qD", "x"])` y `(Worktree, ["remove", "-ff", "w"])`, y sale en rojo con el código de d82ea9d.
- [x] `cargo xtask verificar` en verde, y el `## Resultado` lista los 9 IDs con su commit.

## Resultado

Corregidos los IMPORTANTE del grupo A4 (A4-09 se cerro en TASK-148), con piezas `pub(crate)` o privadas; la API publica no cambia.

| Ficha | Commit |
|---|---|
| A4-01 (`divergente`) | 53bc8bf |
| A4-02 (`divergente`) | 6360f38 |
| A4-05 | bb0afbf |
| A4-07 | 68bd853 |
| A4-03 | 204f3f9 |
| A4-04 | 41a5bc5 |
| A4-06 | 6b682dd |
| A4-08 | b52ed8d |
| A4-10 (D4) | c06d185 (test en rojo) y deffddc (arreglo) |
| A4-09 | TASK-148 |

- Cambios de comportamiento aceptados: con un remoto que deniega, `comprobar-id` y `tag-anotado` dan `denegada motivo=...` (codigo 8) o `error motivo=error-local` en vez de `sinred`; sin `origin`, `comprobar-id` decide con lo local (`ok`/`rechazada`); el `error=` del diagnostico de `leer_blobs` es la primera linea no vacia del stderr.
- Tests en rojo antes del arreglo: `comprobar-id sinred` frente a `denegada ... remoto-no-existe` y frente a `ok id=TASK-007`; `leer_blobs` con `error=` vacio; A4-10 `Push ["-qf", "origin", "r"]: Err(Lanzar(..))` (y las filas de `-qD` y `-ff`).
- Mutantes en rojo: `tag::commit_remoto` con `Sinred`; `ramas_de_tarea` sin `hay_remoto`; `aplicar_en` sin `duplicada`; `actualizar` con `PUSH_FALLO`; `carpeta_ocupada` desactivada (sobrevivia: se anadio `carpeta_de_worktree_ocupada_en_crear_y_reanudar`); `es_prohibido` sin desplegar.
- Suposiciones comprobadas: `ls-remote` contra una ruta que no es repo da 128 y «does not appear to be a git repository» (se clasifica `remoto-no-existe`); ningun test fijaba la linea de `actualizar-rama` al fallar el publicar: se fija ahora en `rechazo_del_remoto_es_denegada_sin_reintento`.
- A4-10: la guarda despliega las opciones cortas agrupadas y cierra ante la duda (rechaza tambien valores pegados como `-mfoo`; ningun llamador los usa). Se mantiene la desviacion del ADR F1-4 D3: lista de prohibidas, no lista permitida.
- Recuento `cargo test -p tc2 --no-fail-fast` (target propio): antes 789 (cierre de TASK-141); despues 48 binarios, 795 ok, 1 fallo, 13 ignorados = 796 (789 + 7). El fallo es `plataforma::aplicar_puertas_carrera_corte_y_plan` con `sinred` del mock (otro cargo en marcha; `plataforma.rs` no cambia): repetido solo, 8 ok.
- Fuera de alcance (decision 5, solo se documentan): `init/inventario.rs:463-477` (clasifica el fallo de `ls-remote` con un `sinred` con detalle), `estado/montaje.rs:139-147` (`RamaInexistente` significa `Crear`), `toma::carpeta_libre` (acepta el worktree registrado).
- `cargo xtask verificar` lo ejecuta `finish`.

## Divergencias

- A4-06: `DestinoWt::registrado` agrupa tambien la busqueda del worktree de la rama (repetida en `crear` y `reanudar`), y se anade un test que el plan no preveia porque el mutante de `carpeta_ocupada` sobrevivia.
- A4-08: se migran tambien `convivencia_016` (copia de `rapida`/`escribir_estado`) y el repo sin remoto de `git_push`.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-24T22:28:15Z | plan | automatico | persona | 0.17.1 |
| 2026-11-24T22:39:34Z | approve | automatico | persona | 0.17.1 |
| 2026-11-24T22:39:38Z | start | automatico | persona | 0.17.1 |
| 2026-11-25T00:20:28Z | review | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-24T22:39:33Z | diseno | agente:a0b46b8c08da6c566 | 80 | 129835 | 3192842 | 27183 | 51 | 3349940 | - | - |
| 2026-11-25T00:20:28Z | implementacion | sesion:f5474097-7166-4a1c-ae42-79e139302dce | 34 | 18844 | 11185875 | 7098 | 0 | 11211851 | - | - |
| 2026-11-25T00:20:28Z | implementacion | agente:a9b3c8ab489805709 | 206 | 301111 | 22357522 | 74961 | 25 | 22733800 | - | - |

## Autorrevision

- rust-wasm-reviewer — deffddc
- 4D — deffddc: helper buscado, criterios citados, 7 mutantes vistos en rojo
