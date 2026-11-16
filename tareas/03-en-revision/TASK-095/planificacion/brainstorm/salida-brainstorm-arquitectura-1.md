# Brainstorm — TASK-095, rol arquitectura (ronda 1)

- Rol: `brainstorm-arquitectura`
- Agente: taskcode-plugin:brainstorm-arquitectura (ronda 1)

## Enfoque propuesto, con rutas y nombres concretos

- Módulo nuevo `crates/tc2/src/indice/` (`mod.rs`, `esquema.rs`, `fuente.rs`, `frescura.rs`, `huella.rs`, `vocabulario.rs`). Depende de `git`, `tarea` y `estado::Estado`. Ni `estado` ni `git` dependen de él.
- El índice lee siempre objetos de git sobre un SHA concreto, nunca el árbol de trabajo de `tareas/`: «mismos SHA, mismo índice» por construcción. La fuente es el trait existente `estado::Estado` (`rutas`, `leer`), con dos vistas: la rama `taskcode/tareas` (2.0) y `develop:tareas/NN-estado/` (0.16, lo que hay hoy en TaskCodeTwo y lo que necesitan la PoC y F10-3).
- Ubicación: `<git-common-dir>/taskcode/indice.db`, calculada con `git::estado::dir_comun`, compartida por los worktrees. `taskcode/` y no `taskcodetwo/`, por coherencia con `refs/taskcode/` y `taskcode-estado.lock`.
- «Mismo hash» = huella SHA-256 de un volcado canónico: tablas lógicas, filas ordenadas por clave, sin `indice_estado` ni tablas internas de FTS5. No es el hash de los bytes del `.db`. Se expone como `tc2 indice estado --huella`.
- Frescura, en cada `buscar` o `grafo`: un `git rev-parse` de las dos referencias remotas frente a `indice_estado`. Si hay diferencia, reindexado incremental, o reconstrucción completa si `es_ancestro` falla. Todo en una transacción SQLite y bajo cerrojo propio.
- Engram, opción A: `memoria.md` por tarea en `TASK-NNN/` y `MEMORIA.md` de proyecto en la raíz de la rama de estado. Lo escribe una `Operacion` nueva registrada en `lib.rs::catalogo`, con el patrón de `alta`.
- Etiquetas: vocabulario en `.taskcode/vocabulario.yml` (rama de código, junto a `config.yml`); etiquetas en el frontmatter de `tarea.md` (rama de estado). Capas determinista y léxica: funciones puras de `tc2`. Capa IA (skill en `finish`): solo propone. Única puerta de escritura: `tc2 tags aplicar`, que rechaza lo que esté fuera del vocabulario.
- Retirada de engram: una línea en el suelo del manifiesto (F1-14, decisión 8) que pase engram de «conveniente» a «estorba» cuando F10-13 dé el visto bueno. El ADR F1-19 no lo retira todavía; lo condiciona a F10-13.

## Que se extiende y que se crea

- Se extiende `git::puerto::Subcomando`, hoy lista cerrada sin `log` ni `cat-file` (líneas 21-99): añadir `Log` y `CatFile` (un `git log --numstat --name-status` en flujo y un `cat-file --batch` para blobs). `Vista::leer` usa `leer_blob`, un proceso por fichero: con 404 tareas, 404 procesos por reconstrucción. Decidirlo en el ADR.
- Se extiende `estado::cerrojo::Cerrojo` (hoy `FICHERO = "taskcode-estado.lock"` fijo): `tomar_en(comun, fichero, espera)`, para que el índice use `taskcode-indice.lock` y una reconstrucción larga no bloquee `new`.
- Se extiende `Vista::nueva` (hoy `pub(crate)` y atada a la rama de estado): aceptar cualquier commit con un prefijo, para leer la 0.16.
- Se extiende `lib.rs`: órdenes `indice`, `buscar`, `tags` y `grafo`. El atajo de actualizar tras `new`, transiciones y `finish` va en la orquestación de `lib.rs`, después de `estado::transaccion`, nunca dentro.
- Se extiende `tarea::modelo`: `etiquetas` ya existe (`modelo.rs:170`). La sección generada `## Archivos tocados` entra en la tabla de secciones de F1-9.
- Se crea: `indice/` completo, `rusqlite` con `bundled` (FTS5) como dependencia nueva de `tc2/Cargo.toml`, `.taskcode/vocabulario.yml` y la `Operacion` de importar memoria.

## Limites que cruza

- Formato de fichero: `tarea.md` (F1-9) gana `## Archivos tocados` y usa `etiquetas`. La rama de estado (F1-1) gana `memoria.md` y `MEMORIA.md`. La config (F2-7) gana el mapa de alias de proyectos para engram.
- Contrato público: `tc2 buscar --json` es la API de búsqueda del plugin (RF-81), como `taskctl tareas --json` lo es de lectura (F1-11, decisión 5). Esquema JSON versionado como el contrato de F1-4.
- Hooks: ninguno necesario. `post-merge` y `post-checkout` solo como atajo del mismo comando (F1-11, decisión 10).
- Manifiesto de F1-14, decisión 8: cambia el nivel de engram, condicionado a F10-13.
- Dos referencias de verdad: `develop` (vocabulario y diffs) y `taskcode/tareas` (tareas y memoria). `indice_estado` guarda las dos.

## La decision de diseño que mas te preocupa (UNA sola)

Con qué se compara el SHA indexado en cada lectura. Un `rev-parse` local no ve un merge hecho en la plataforma hasta un `fetch`, y `estado::leer` declara «sin fetch: la lectura fresca es de F3-5b» (`lectura.rs:99-101`). Así el criterio 2 no se cumple. Posición: `git ls-remote origin develop taskcode/tareas` en cada lectura (`LsRemote` ya existe en el puerto); si difiere, `fetch` dirigido y reindexado. Sin red, responder marcando `fresco=no sincronizado=<hora>`. Coste: una ida y vuelta de red por búsqueda, que choca con los 200 ms de F10-6. La política tiene que ser una sola para `status` (F3-5b) y `buscar`, implementada una vez en `git::remoto`.

## Alternativa descartada

- Índice versionado en la rama de estado o en `develop`: contradice RR-20 y reintroduce conflictos de binarios.
- Tantivy en lugar de SQLite FTS5: buen BM25 pero sin relacional para aristas y grafo; dos almacenes.

## Desacuerdos previstos

- Riesgos: `ls-remote` en cada lectura frente a caché con TTL. Posición: sin TTL (un TTL es «datos viejos» por diseño); marcar `fresco=no`.
- Testing: huella por volcado canónico, no hash del `.db`.
- Dominio: vocabulario en la rama de código (mapea rutas de código y lo revisa el PR).

## Suposiciones no verificadas

- Que `ls-remote` cabe en el presupuesto de búsqueda: medir contra GitHub desde Windows en la PoC.
- Que `rusqlite` con `bundled` compila con FTS5 en los tres sistemas de la CI (F1-7): PoC mínima bajo `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-095\`.
- Que no hay política de fetch ya decidida en F1-2 que valga para leer.
- Que el conjunto de tareas terminadas se puede sacar de `develop:tareas/` con el formato 0.16.
