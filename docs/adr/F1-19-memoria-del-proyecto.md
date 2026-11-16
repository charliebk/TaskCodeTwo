# ADR F1-19 — Memoria del proyecto: git como verdad, índice SQLite reconstruible, frescura, etiquetado y conversor de engram

- Estado: **aceptada** (2026-11-16)
- Tarea: TASK-095 · Plan: [F1-19 en PLAN-TASKCODE-2](../plan/PLAN-TASKCODE-2.md) (tabla de la Fase 1 y Fase 10) · Diseño de partida: [AMPLIACION-2026-11-16 §3](../plan/AMPLIACION-2026-11-16.md)
- Requisitos: RR-20, RF-77 a RF-84 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); se apoya en F1-1, F1-2, F1-9, F1-11 (decisión 10) y F1-14 (manifiesto)
- Decide: Carlos, el 2026-11-16, en el diseño de TASK-095. No trocear la tarea. Conversor de engram con la opción A y salvaguardas. Vocabulario en la rama de código. Para la frescura, tras ver la medición de la PoC 2: «TTL corto con marca». Al planificar había elegido `ls-remote` en cada lectura, con la condición de volver a preguntar si se pasaba de 200 ms, y se pasó: 760 ms de mediana

## Contexto

Hoy la memoria entre sesiones es engram: una base **por máquina**
(`~/.engram/engram.db`), fuera de git y que el equipo no comparte. El modelo
de tarea ya tiene `etiquetas` y `dependencias`, pero las 94 tareas tienen
`etiquetas: []`. RR-20 pide una memoria propia con estas propiedades:

- git como fuente de verdad;
- un índice SQLite que **nunca se versiona** y se puede reconstruir;
- una búsqueda única para el plugin (`tc2 buscar`);
- etiquetado acotado a un vocabulario cerrado;
- no depender de engram.

El problema de fondo es la frescura. El `merge-pr` lo hace la persona **en la
plataforma** (F1-6, F1-10), y ningún hook local se entera, ni de eso ni del
push de un compañero. La AMPLIACION (§3.2) proponía comparar en cada lectura
el SHA indexado con `git rev-parse` local. Los dos roles del diseño (arquitectura y
riesgos) señalaron, cada uno por su lado, que eso **no ve** un merge hecho en
la plataforma hasta que alguien haga `fetch`. `estado::leer` ya lo dice:
«sin `fetch`: la lectura fresca es de F3-5b» (`crates/tc2/src/estado/lectura.rs`).

## Alternativas

### Dónde vive cada cosa

| | Descripción | Veredicto |
|---|---|---|
| **A** | Verdad en git (`tarea.md`, vocabulario, memoria importada). Índice en `<git-common-dir>/taskcode/indice.db`, fuera de cualquier árbol versionado. | **Elegida** |
| B | Índice versionado en la rama de estado o en `develop`. | Descartada: contradice RR-20 («nunca se versiona») y trae conflictos de binarios, la misma clase de choque que el `ROADMAP.md` de F1-1. |
| C | Tantivy en lugar de SQLite con FTS5. | Descartada: da BM25 pero no tiene modelo relacional para aristas y grafo (RF-83), así que harían falta dos almacenes. |

### Frescura

| | Descripción | Veredicto |
|---|---|---|
| F1 | Comparar con `rev-parse` local (AMPLIACION §3.2 tal cual). | Descartada: en la PoC 2, `origin/develop` del clon A seguía en `e1bf456` con el bare ya en `8bb0ad0`. Comparando en local, el índice diría «al día». |
| F2 | Solo hooks `post-merge` y `post-checkout`. | Descartada: un merge en la plataforma no dispara ningún hook local, y F1-11 (decisión 10) deja los hooks como atajo opcional. |
| F3 | `ls-remote` en cada lectura, con `fetch` dirigido si cambió. | Descartada **por medición**: 760 ms de mediana contra GitHub frente a los menos de 200 ms de F10-6 (ver PoC 2). |
| **F4** | `ls-remote` como mucho una vez por TTL (60 s), con la marca «sincronizado hace N s» en cada respuesta. | **Elegida** (Carlos). |
| F5 | Remoto solo bajo demanda (`start`, `finish`, `status` o `--fresco`). | Descartada: el criterio de RR-20 dependería de que alguien lance otro comando. |

### Memoria de engram

| | Descripción | Veredicto |
|---|---|---|
| **A** | Se materializa en git: `TASK-NNN/memoria.md` y `MEMORIA.md` de proyecto, en la rama de estado. | **Elegida**, con las salvaguardas de la decisión 8. |
| B | Solo en el índice local. | Descartada: no se comparte y se pierde al reconstruir, justo lo contrario de RR-20. Evitaba el riesgo de fuga sin vuelta atrás, y por eso A lleva salvaguardas obligatorias. |

## Pruebas de concepto

Un crate desechable `poc-indice` (Rust 1.100, `rusqlite` 0.43 con `bundled`,
`sha2` 0.11), en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-095\`.
Se borra al terminar la tarea y no entra en el repo (Fase 1). Git
2.56.0.windows.4, Windows 11.

Qué hace la PoC:

- Lee **siempre objetos de git** del SHA fijado: `ls-tree -r`, un único
  `cat-file --batch` para los 30 `tarea.md` y un único `log --numstat`.
- Atribuye cada commit por `(TASK-NNN)` en el asunto.
- Escribe todo en una transacción SQLite. Los SHA indexados van en la misma
  transacción.

Simplificaciones: sin A/M/D/R ni renombrados (`--no-renames`), y siempre
reconstruye entero. El incremental es de F10-2 y el A/M/D/R, de F10-4.
`rusqlite` con `bundled` compila con FTS5 en Windows. En Linux y macOS no
está verificado.

### PoC 1 — mismo hash dos veces (criterio 1)

```bash
poc-indice reconstruir --repo TaskCodeTwo --db i1.db --ref develop --prefijo tareas/04-terminadas/   # x2, borrando i1.db entre medias
```

```text
== 1a reconstruccion
indexado e1bf4566b3e4 tareas=30 commits=362 archivos=1307 huerfanos=120 huella=9527d653c76d0fde26aacbb3043f051f83c8164c62aaea77af1c1fc40a994fe4
== 1b reconstruccion (db borrada)
indexado e1bf4566b3e4 tareas=30 commits=362 archivos=1307 huerfanos=120 huella=9527d653c76d0fde26aacbb3043f051f83c8164c62aaea77af1c1fc40a994fe4
== control: arbol sucio + autocrlf=true en un clon
 M tareas/04-terminadas/TASK-001/tarea.md
indexado e1bf4566b3e4 ... huella=9527d653c76d0fde26aacbb3043f051f83c8164c62aaea77af1c1fc40a994fe4
== control: cambiar una tarea en un commit
indexado 53d52d6cc852 tareas=30 commits=363 ... huella=96cb285c9cf19a94bddd75d994f97991a1e52c30f94c5b6683cf90a77385bcaa
```

Lectura:

- **Misma huella** en las dos reconstrucciones.
- Un árbol de trabajo modificado con `core.autocrlf=true` no la cambia,
  porque se leen blobs y no ficheros.
- Un commit que cambia una tarea sí la cambia, así que no es un falso verde.
- Los bytes de dos `.db` reconstruidos en esta máquina también coincidieron
  (`8a352130…`). Pero el formato de página de SQLite y la fusión de segmentos
  de FTS5 **no son contrato**: la huella es la del volcado canónico.
- Dato real: de los 482 commits de `develop`, 120 no se atribuyen a ninguna
  de las 30 tareas terminadas. Son commits como `docs: ROADMAP regenerado…` o
  `chore: alta de 18 tareas…`, más los de tareas no terminadas.

### PoC 2 — merge en la plataforma sin hooks (criterio 2)

Montaje:

- Un bare `origen.git` hace de plataforma.
- El clon A tiene el índice y `core.hooksPath` apuntando a una carpeta vacía.
- El clon B crea `TASK-999` («zarzaparrilla») en una rama, hace `merge --no-ff`
  en `develop` y `push`.
- A ejecuta `buscar` **sin `fetch` manual**.

Primera pasada, sin TTL (equivale a F3):

```text
origin/develop en A antes de buscar: e1bf456  bare: 8bb0ad0
== A: busqueda sin fetch manual ni hooks
fresco=si remoto=consultado(145 ms) reindexado=incremental (la PoC reconstruye entera) indexado=8bb0ad0 total=4321 ms
  TASK-999 Prueba zarzaparrilla
== B: force-push que reescribe develop
fresco=si remoto=consultado(155 ms) reindexado=completa (el SHA viejo no es antecesor o no existe) indexado=c635c0f total=4828 ms
  (sin resultados)
== remoto inaccesible
fresco=desconocido remoto=no-consultado(185 ms) indexado_hasta=c635c0f692bf... total=191 ms
```

Medición de `ls-remote` con tiempo límite y sin prompt de credenciales
(`GIT_TERMINAL_PROMPT=0`, `GCM_INTERACTIVE=never`), 15 tomas cada una:

| Remoto | Mín. | Mediana | Máx. |
|---|---|---|---|
| GitHub (`origin` de TaskCodeTwo) | 732 ms | **760 ms** | 850 ms |
| bare local | 143 ms | 149 ms | 155 ms |

Lanzar `git --version` cuesta unos 130 ms en esta máquina, medido desde bash.
Con F3, ninguna búsqueda cabe en 200 ms, ni siquiera contra un bare local.

Segunda pasada, con F4 (TTL de 60 s; la caducidad se simula con `--ttl-s 0`):

```text
== A: 5 busquedas dentro del TTL
fresco=hace-11s remoto=no-consultado(ttl 60s) indexado=e1bf456 total=6 ms     (x5, 6-7 ms)
== A: justo tras el merge, dentro del TTL
fresco=hace-17s remoto=no-consultado(ttl 60s) indexado=e1bf456 total=5 ms
  (sin resultados)
== A: TTL caducado (simulado con ttl 0)
fresco=si remoto=consultado(249 ms) reindexado=incremental (la PoC reconstruye entera) indexado=446453d total=7367 ms
  TASK-999 Prueba zarzaparrilla
hooks ejecutados: 0
```

Lectura:

- Con F4, las ráfagas de búsquedas responden en **6 ms** desde el índice.
- Dentro del TTL, un merge recién hecho **no aparece, pero la respuesta lo
  dice** («hace 17 s, remoto no consultado»).
- La primera búsqueda tras caducar el TTL lo encuentra sin hooks ni `fetch`
  manual.
- La reconstrucción completa de la PoC tarda de 4 a 12 s, casi todo en
  `log --numstat` sobre 482 commits. El reindexado incremental de F10-2 es lo
  que lo reduce. Aquí está sin medir.

No verificado:

- la búsqueda que lee la versión anterior con el cerrojo ocupado (decisión 5);
- la lectura de refs sin lanzar `git` (decisión 4);
- `rusqlite` con `bundled` en Linux y macOS;
- copiar `engram.db` con la API de backup;
- que la 0.16 tolere los campos y secciones nuevas de `tarea.md`.

## Decisión

1. **Git es la verdad; el índice se puede reconstruir y no se versiona.**
   - En git van:
     - `tarea.md`: frontmatter con `etiquetas` y `etiquetas_origen`, y la
       sección generada `## Archivos tocados`;
     - `.taskcode/vocabulario.yml`;
     - `TASK-NNN/memoria.md` y `MEMORIA.md` en la rama de estado.
   - Fuera de git: el índice y las cachés del grafo.
   - Borrar el índice nunca pierde nada.

2. **Ubicación.** `<git-common-dir>/taskcode/indice.db`, calculada con
   `git rev-parse --git-common-dir`. En un worktree enlazado `.git` es un
   fichero, así que una ruta literal fallaría. La comparten todos los
   worktrees y nunca sale en `git status`. Se usa `taskcode/` por coherencia
   con `refs/taskcode/` y `taskcode-estado.lock`. Va junto a
   `taskcode-indice.lock`.

3. **Fuente: objetos de git sobre un SHA fijado, nunca el árbol de trabajo.**
   - El índice representa lo **integrado**:
     - `refs/remotes/origin/taskcode/tareas`: tareas y memoria;
     - `refs/remotes/origin/develop`: vocabulario y diffs;
     - en un repo sin remoto, las ramas locales.
   - Lectura en lote:
     - `cat-file --batch`, un proceso para todos los blobs y no uno por
       fichero: se añade `Subcomando::CatFile` al puerto de F1-4;
     - un único `log --numstat --name-status` en flujo: se añade
       `Subcomando::Log`.
   - Durante la convivencia con la 0.16, una segunda vista lee
     `develop:tareas/NN-estado/` (F10-3).
   - Las ramas en curso se añaden como capa **provisional** marcada (F10-2).

4. **Frescura: una sola política en `git::remoto`, compartida con
   `status`/`roadmap` (F3-5b).**
   - **En cada lectura**: el SHA indexado se compara con las refs de
     seguimiento locales. Es barato: se lee el fichero de ref o `packed-refs`
     sin lanzar `git`, y si el repo usa reftable, con `git` (no verificado).
     Cualquier `fetch` o `pull` de la persona se ve en la siguiente búsqueda.
   - **Como mucho una vez por TTL** (60 s por defecto; la config solo puede
     bajarlo, con el patrón de F1-10):
     - `git ls-remote origin develop taskcode/tareas`, con tiempo límite
       (2 s), `GIT_TERMINAL_PROMPT=0` y `GCM_INTERACTIVE=never`;
     - si difiere: `fetch` dirigido de esas dos refs y reindexado;
     - la hora de la consulta se guarda en la tabla `sincronizacion`.
   - Reindexado:
     - incremental (`log` y `diff --name-status` entre los SHA) si el viejo
       es antecesor del nuevo;
     - **reconstrucción completa** si no lo es, o ante cualquier fallo al
       resolverlo (SHA podado, force-push, `bad object`).
   - **Toda respuesta lleva la marca**: `indexado_hasta` (los dos SHA),
     `sincronizado_hace` (segundos) y
     `remoto: consultado | no-consultado(ttl) | inaccesible`.
   - Sin red, con el tiempo agotado o con el cerrojo ocupado, se responde
     igual, con la marca, y nunca se pide credencial.
   - `new`, `import`, las transiciones y `finish` actualizan el índice al
     terminar su transacción, como atajo y fuera de `estado::transaccion`.
     Los hooks `post-merge` y `post-checkout` solo llaman al mismo comando
     (F1-11, decisión 10).
   - **Cómo se lee RR-20** (Carlos): «nunca responde con datos viejos **sin
     decirlo**». La comprobación por SHA local es en cada lectura; la del
     remoto, como mucho una vez por TTL, y siempre se dice desde cuándo.

5. **Atomicidad y concurrencia.**
   - Reindexado y reconstrucción en **una transacción SQLite, en sitio**. No
     se escribe un fichero nuevo con `rename`: en Windows falla si otro
     proceso tiene el `.db` abierto.
   - Los SHA y la hora de sincronización se escriben en la misma
     transacción, así que un corte no los avanza y nunca deja un índice vacío
     con los SHA ya movidos.
   - **Cerrojo propio** `taskcode-indice.lock`, separado del de estado: se
     generaliza `Cerrojo` con `tomar_en(comun, fichero, espera)`. Una
     reconstrucción larga no bloquea `new`.
   - Si una búsqueda encuentra el cerrojo ocupado, **lee la versión anterior**
     (SQLite en modo WAL) con la marca `desfasado` y no espera.
   - La versión de esquema va en `PRAGMA user_version`. Si no coincide, se
     reconstruye; no hay migraciones (lo hace la PoC).

6. **Esquema y huella.**
   - Las tablas de AMPLIACION §3.3, más:
     - `commit_huerfano(sha, asunto)`: los commits sin tarea atribuible, que
       se cuentan en `tc2 indice estado` y no se reparten a ojo;
     - `sincronizacion(ref, consultado_en)`.
   - `tarea_archivo` lleva `cambio` A/M/D/R con los dos nombres de un
     renombrado (F10-4).
   - Las etiquetas se indexan con su `origen`.
   - **Huella** = SHA-256 de un volcado canónico:
     - las tablas lógicas, con las filas ordenadas por su clave;
     - texto escapado (`\\`, `\t`, `\n`) y `NULL` como `\N`;
     - sin `indice_estado`, `sincronizacion` ni las tablas internas de FTS5.
   - Se expone en `tc2 indice estado --huella`: dos máquinas con los mismos
     SHA dan la misma huella (RF-77).

7. **Vocabulario y tres capas de etiquetado.**
   - **Dónde vive** (Carlos): `.taskcode/vocabulario.yml`, en la rama de
     código. Tiene etiqueta canónica, alias y mapa de rutas a etiqueta, y se
     revisa en el PR. Se renombra con alias y nunca se borra una etiqueta
     usada.
   - **Formato en `tarea.md`** (cambio de F1-9):
     - `etiquetas` sigue siendo una lista plana de nombres canónicos, para
       que la selección de F1-15 no cambie;
     - un campo nuevo `etiquetas_origen` (mapa
       `nombre: determinista | lexica | ia | persona`) dice de dónde sale cada
       una.
   - **Capas**:
     - **Determinista**: rutas del diff según el mapa del vocabulario, ADR
       citados, fase, `tipo` y `dependencias`.
     - **Léxica**: BM25; solo propone etiquetas del vocabulario.
     - **IA acotada en `finish`**: elige de 3 a 5 etiquetas del vocabulario.
       Una etiqueta nueva va a la cola `propuestas-etiquetas.yml` de la rama
       de estado, con su motivo. Entra en el vocabulario por PR si la aprueba
       una persona o si aparece en N tareas.
   - **Puertas**:
     - La única puerta de escritura es `tc2 tags aplicar`, que **rechaza todo
       lo que esté fuera del vocabulario**.
     - Las capas escriben en git en `finish`, **nunca en `reconstruir`**: el
       índice solo lee etiquetas ya escritas. Si no fuera así, el mismo SHA
       daría otra huella.
   - **`tc2 tags doctor`**:
     - etiquetas con una sola tarea;
     - etiquetas en más del 40 % de las tareas;
     - etiquetas casi iguales;
     - alias sin resolver.
     Los ficheros comodín (`CHANGELOG.md`, `Cargo.lock`, `ROADMAP.md`…)
     tienen peso casi nulo en las relaciones.

8. **Conversor de engram, opción A con salvaguardas obligatorias.**
   - **Copia**: se copia `engram.db` con la API de backup de SQLite o con
     `VACUUM INTO`, nunca copiando el fichero: en modo WAL perdería escrituras
     recientes. Solo se lee la copia.
   - **Proyecto**: por `sessions.directory` **normalizado** (mayúsculas y
     minúsculas en Windows, `C:\` frente a `/c/`, separadores) igual a la raíz
     del repo, más un mapa de alias en la config
     (`opendocktranding → opendocktrading`). Lo ambiguo **no se escribe**: se
     lista para que lo confirme una persona.
   - **Escritura**:
     - `--simular` va **antes de la primera escritura**, y la escritura exige
       el hash del plan simulado (el patrón de F1-12);
     - filtro de secretos antes de entrar en git;
     - idempotente por `sync_id`, contando también lo que está pendiente de
       publicar (F1-2).
   - **Destino**: se materializa en `TASK-NNN/memoria.md` (observaciones con
     `TASK-NNN`) y `MEMORIA.md` (el resto), en la rama de estado, por la
     transacción de F1-2. Las relaciones juzgadas pasan a aristas con
     procedencia `engram`.
   - **Motivo de las salvaguardas**: lo que llega al remoto se queda en su
     historia aunque se revierta, así que no tiene vuelta atrás.

9. **Retirada de engram del manifiesto: condicionada, no ahora.**
   - engram sigue como «conveniente» en el manifiesto de F1-14.
   - Cuando F10-13 mida que `tc2 buscar` gana y Carlos lo registre, engram
     pasa a «estorba»: sus instrucciones piden `mem_save` y competirían con
     `tc2 buscar`. Entonces `doctor` avisa.
   - Hasta ese momento, el conversor es incremental. Retirarlo antes haría
     perder la base de comparación de F10-13.

## Requisitos y evidencia

| Requisito | Alternativa | Evidencia |
|---|---|---|
| RR-20, verdad en git e índice no versionado | A, decisiones 1 y 2 | Índice en `<git-common-dir>`; la PoC lee solo objetos de git |
| RR-20, nunca datos viejos sin decirlo | F4, decisión 4 | PoC 2: con el TTL caducado aparece TASK-999; dentro del TTL, la marca «hace 17 s» |
| RF-77, mismo resultado con los mismos SHA | Decisiones 3 y 6 | PoC 1: huella `9527d653…` dos veces; un árbol sucio con `autocrlf` no la cambia |
| RF-78, conversor de tareas | Decisión 3 | PoC 1: 30 tareas y 362 commits atribuidos, 120 huérfanos contados |
| RF-79, ficheros tocados | Decisión 6 | PoC 1: 1307 filas tarea-fichero con líneas; A/M/D/R queda para F10-4 |
| RF-80, conversor de engram | A, decisión 8 | Diseño. La base real (146/116) se verifica en F10-5 |
| RF-81, `tc2 buscar` único con JSON | Decisión 4 | PoC 2: búsqueda FTS5 con BM25 en 6 ms dentro del TTL |
| RF-82, etiquetado en tres capas | Decisión 7 | Diseño; las capas escriben en `finish`, no al reconstruir |
| RF-83 y RF-84, grafo y visor | — | Fuera de alcance (Fase 10, F10-12 a F10-17) |
| F10-6, menos de 200 ms | F4 frente a F3 | `ls-remote` contra GitHub: 760 ms de mediana (F3 descartada); con F4, de 6 a 7 ms dentro del TTL |

## Consecuencias

- **F10-1**:
  - módulo `crates/tc2/src/indice/`, con `rusqlite` (`bundled`) como
    dependencia nueva;
  - `PRAGMA user_version`;
  - `tc2 indice reconstruir` y `tc2 indice estado --huella`;
  - criterio de «nunca en `git status`».
- **F10-2**:
  - la política de la decisión 4 en `git::remoto`, la misma que usará F3-5b;
  - incremental entre SHA;
  - `Cerrojo::tomar_en`;
  - lectura de la versión anterior con el cerrojo ocupado;
  - coste de la reconstrucción completa: la PoC tarda de 4 a 12 s con 482
    commits.
- **F10-3**: vista de la 0.16 (`develop:tareas/NN-estado/`), con los
  renombrados entre carpetas y la categoría de huérfanos.
- **F10-4**: `## Archivos tocados` entra en la tabla de secciones de F1-9.
- **F10-5**: las salvaguardas de la decisión 8 son su criterio de seguridad.
- **F10-6**: los 200 ms se miden **con el índice al día y dentro del TTL**.
  La primera búsqueda tras el TTL paga el `ls-remote`, unos 0,8 s con GitHub,
  y lo dice en la marca.
- **F10-8 a F10-10**: vocabulario en `.taskcode/vocabulario.yml`,
  `etiquetas_origen`, `tc2 tags aplicar` como única puerta, y cola de
  propuestas en la rama de estado.
- **F10-13**: decide la retirada de engram (decisión 9).
- **F1-9**: campo nuevo `etiquetas_origen` y sección generada
  `## Archivos tocados`. Antes de F10-4 hay que comprobar que `taskctl` 0.16
  conserva campos y secciones que no conoce.
- **F1-4**: el puerto gana `Subcomando::CatFile` y `Subcomando::Log`.
- **F3-5b**: usa la misma política y la misma marca de sincronización.
- **Texto de RR-20**: hoy dice «comprueba en cada lectura hasta qué commit
  está indexado». Con esta decisión, la comprobación remota es por TTL y
  siempre lleva la marca. Ajustar la redacción del requisito lo decide Carlos
  fuera de este ADR.

## Fuera de alcance

- Grafo y visor 3D (RF-83, RF-84): F10-12 a F10-17.
- Umbrales exactos de `tc2 tags doctor` y la N de apariciones de la cola de
  propuestas: F10-9 y F10-10.
- Pesos IDF de las relaciones: F10-11.
- Formato de salida JSON de `tc2 buscar`: F10-6, versionado como el contrato
  de F1-4.
