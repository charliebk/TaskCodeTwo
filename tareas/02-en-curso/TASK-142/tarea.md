---
id: TASK-142
titulo: "Auditoria 4D: A4 git, rama de estado y toma (9 IMPORTANTE)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-curso
plan_aprobado: true
rama: fix/task-142-auditoria-4d-a4-git-rama-de-estado-y-tom
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 3349940
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-23
actualizado: 2026-11-24
dependencias: []
---
## Objetivo

Corregir los 9 casos IMPORTANTE del grupo A4 de `docs/auditoria-4d.md` (commit auditado d82ea9d): 8 Duplicaciones (dos `divergente`) y 1 Disfraz en código en la guarda de argumentos de git. A4-09 va en la tarea de soporte de tests compartido. No se piden los MENOR (A4-11, A4-12). No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.

## Criterios de aceptacion
- [ ] A4-01 (`divergente`, va primero): la consulta de ramas de tarea locales y remotas (`git/rama.rs:88-110` ~ `:160-196`) queda escrita una vez. El fallo del remoto se clasifica igual (sinred / error local / denegada) en `git/remoto.rs:175-181` y `git/tag.rs:100-114`, y lo prueba un test con un remoto que deniega y que hoy sale `sinred`.
- [ ] A4-02 (`divergente`): el envoltorio «lanzar git y traducir el fallo» (`git/estado.rs:243-265`, `:269-298`, `:135-150` y `git/mod.rs:52-89`) es uno solo, y `leer_blobs` usa `primera_linea_error()`.
- [ ] A4-05 y A4-07: `hay_remoto` es `url_origin(..)?.is_some()` (`git/estado.rs:451-460` ~ `:463-473`), y hay un solo analizador de `git status --porcelain -z` (`git/guardar.rs:158-176` ~ `git/mod.rs:216-228`).
- [ ] A4-03 y A4-04: las 19 líneas de `intento` y `sin_red` (`estado/transaccion.rs:353-371` ~ `:533-551`) y el informe de un `publicar` fallido (`git/tag.rs:81-91` ~ `git/remoto.rs:516-526`) quedan escritos una vez cada uno.
- [ ] A4-06: `crear` y `reanudar_paso` del worktree de tarea (`git/rama.rs:318-380` ~ `:388-456`) comparten el arranque y el bloque de carpeta ocupada.
- [ ] A4-08: los ayudantes `rapida`, `escribir_estado` y `rama_en` y el repo sin remoto (`tests/estado_bordes.rs:19-53` ~ `tests/estado_casos.rs:19-44`, `:124-129` ~ `tests/estado_concurrencia.rs:216-220`) quedan en un módulo común de tests.
- [ ] A4-10: `es_prohibido` (`git/puerto.rs:613-646`) rechaza las opciones cortas agrupadas que contienen una prohibida, o pasa a una lista permitida como pide F1-4 (decisión 3). `rechaza_cada_argumento_prohibido_sin_lanzar_git` (`puerto.rs:759-846`) incluye `(Push, ["-qf", "origin", "r"])`, `(Branch, ["-qD", "x"])` y `(Worktree, ["remove", "-ff", "w"])`, y sale en rojo con el código de d82ea9d.
- [ ] `cargo xtask verificar` en verde, y el `## Resultado` lista los 9 IDs con su commit.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-24T22:28:15Z | plan | automatico | persona | 0.17.1 |
| 2026-11-24T22:39:34Z | approve | automatico | persona | 0.17.1 |
| 2026-11-24T22:39:38Z | start | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-24T22:39:33Z | diseno | agente:a0b46b8c08da6c566 | 80 | 129835 | 3192842 | 27183 | 51 | 3349940 | - | - |
