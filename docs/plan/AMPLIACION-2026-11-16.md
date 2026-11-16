# Pendiente de añadir a TaskCodeTwo: memoria del proyecto (índice de contexto, etiquetas, búsqueda y grafo)

> **APLICADO el 2026-11-16** en TaskCodeTwo tras integrar TASK-026: plan y
> requisitos (RR-20, RF-77..RF-84, F1-19 y Fase 10), tareas TASK-095..TASK-112
> creadas con sus dependencias e hito H10 en la zona manual del ROADMAP. Este
> documento queda como registro.

> Decidido por Carlos el 2026-11-16: hito nuevo. Se añade a TaskCodeTwo (plan,
> requisitos y tareas) **cuando TASK-026 (F2-11) esté integrada en `develop`**,
> para no entorpecer la rama en curso. Este documento tiene lo necesario para
> hacerlo de una vez, igual que [AMPLIACION-2026-11-14](AMPLIACION-2026-11-14.md).

## 0. Motivo

- Hoy la memoria entre sesiones depende de **engram**, que el ADR F1-14 deja
  como recurso «conveniente, no obligatorio». Su base es **por máquina**
  (`~/.engram/engram.db`), no está en git y no la comparte el equipo.
- El modelo de tarea ya tiene `etiquetas` y `dependencias`
  (`crates/tc2/src/tarea/modelo.rs`), pero **las 94 tareas tienen
  `etiquetas: []`**: si etiquetar depende de la persona, no se hace.
- La convención de commits `tipo(TASK-NNN)` permite saber, sin adivinar, qué
  ficheros tocó cada tarea (comprobado en TASK-020 y TASK-025).
- Datos reales de engram el 2026-11-16: 146 observaciones, 59 sesiones,
  228 relaciones juzgadas por el agente; **116 de 146 observaciones citan un
  `TASK-NNN`**; un proyecto aparece con dos nombres por errata
  (`opendocktrading` / `opendocktranding`).

Objetivo: un sistema **propio** de memoria del proyecto, centrado en tareas,
con relaciones basadas en hechos (ficheros tocados) y no solo en palabras,
compartido por el equipo y que sustituya a engram.

## 1. Requisito rector nuevo (REQUISITOS-FUNCIONALES.md §0)

| ID | Requisito rector | Qué significa en la 2.0 |
|---|---|---|
| RR-20 | **Memoria del proyecto propia, compartida y nunca desactualizada** | Git es la fuente de verdad (tareas, etiquetas, ficheros tocados, memoria importada); una base SQLite local es un **índice reconstruible** que nunca se versiona. Toda búsqueda del plugin pasa por ese índice y **nunca responde con datos viejos**: comprueba en cada lectura hasta qué commit está indexado. Etiquetado automático acotado a un vocabulario cerrado. Sin dependencia de engram |

## 2. Cambios en el catálogo de requisitos

| ID | Cambio |
|---|---|
| RF-77 (nuevo) | **I** — Índice SQLite reconstruible desde git (`tc2 indice reconstruir`), mismo resultado en cualquier máquina con los mismos SHA |
| RF-78 (nuevo) | **I** — Conversor de las tareas existentes: todas las `tarea.md` y su historial git entran en el índice |
| RF-79 (nuevo) | **I** — Ficheros tocados por tarea (ruta, nombre, A/M/D/R, líneas), generados en `finish` y sacados de git para las antiguas |
| RF-80 (nuevo) | **I** — Conversor de engram por proyecto: importa observaciones, sesiones y relaciones, las enlaza a sus tareas y las materializa en git |
| RF-81 (nuevo) | **I** — `tc2 buscar` como único punto de búsqueda del plugin (texto, etiqueta, fichero, tareas vecinas), con salida JSON |
| RF-82 (nuevo) | **I** — Etiquetado automático en tres capas (determinista, léxica, IA acotada) con vocabulario cerrado y `tc2 tags doctor` |
| RF-83 (nuevo) | **E** — Grafo de tareas, etiquetas, ficheros y relaciones (`tc2 grafo --json`) |
| RF-84 (nuevo) | **E** — Visor 3D opcional (`tc2 visor`): página Vue servida por el propio ejecutable en un puerto local, con paneles de dockview (como ToolsaWeb) y varias vistas del grafo; la búsqueda ilumina etiquetas y relaciones unos milisegundos, los cambios de código destellan un halo verde (líneas añadidas) o rojo (líneas borradas) en el fichero y los ficheros creados aparecen en la red; un panel muestra en tiempo real qué se busca, con qué etiquetas y qué ficheros se tocan, sincronizado con el grafo |

## 3. Diseño propuesto (lo fija el ADR F1-19)

### 3.1 Qué va en git y qué no

| En git (fuente de verdad) | Fuera de git (índice reconstruible) |
|---|---|
| `tarea.md`: `etiquetas` en el frontmatter y sección generada `## Archivos tocados` | `indice.db` (SQLite con FTS5), propuesta `.git/taskcodetwo/indice.db`: nunca se commitea por error y la comparten los worktrees |
| `.taskcode/vocabulario.yml`: etiqueta canónica, alias y mapa de rutas → etiqueta | cachés del grafo |
| memoria importada de engram (§3.4) | |

### 3.2 Cuándo se actualiza el índice: nunca desactualizado

Un evento no basta: el `merge-pr` lo hace la persona **en la plataforma**
(F1-6, F1-10) y ningún hook local se entera; tampoco de un `pull` ni del push
de otra persona.

1. **Comprobación en cada lectura (la garantía).** El índice guarda los SHA
   indexados de la rama de estado y de `develop`. Antes de cada `buscar` o
   `grafo`, `tc2` los compara con `git rev-parse`:
   - iguales → responde;
   - distintos → reindexa solo lo cambiado (`git log` y
     `git diff --name-status` entre ambos SHA) y responde;
   - el SHA viejo no es antecesor del nuevo (rebase, force-push) →
     reconstrucción completa.
2. **Actualización inmediata en los comandos de `tc2` (atajo de velocidad).**
   Al final de la transacción de `new`, `import`, transiciones y `finish`.
   `finish`, tras el merge en `develop`, fija ficheros tocados, etiquetas y
   relaciones definitivas.
3. **Hooks** `post-merge` y `post-checkout`: solo atajo opcional (F1-11,
   decisión 10).
4. **Atomicidad.** Reindexar en una transacción SQLite bajo el cerrojo de
   `tc2`; los SHA se escriben en la misma transacción. Un corte a medias no
   avanza los SHA y la siguiente lectura repite el trabajo.

Qué representa: lo **integrado** (rama de estado + `develop`). Las ramas de
tareas en curso, como capa **provisional** marcada.

### 3.3 Esquema mínimo

```sql
tarea(id, titulo, estado, tipo, sprint, creado, actualizado, cuerpo)
tarea_dependencia(tarea, depende_de)
tarea_commit(tarea, sha, fecha, autor)
tarea_archivo(tarea, ruta, nombre, cambio, lineas_mas, lineas_menos)
etiqueta(id, nombre, canonica)
tarea_etiqueta(tarea, etiqueta, origen)       -- determinista | lexica | ia | persona
memoria(id, tarea, proyecto, tipo, titulo, contenido, origen, ref_origen)
arista(origen, destino, tipo, peso, procedencia) -- dependencia, cocambio, etiqueta, engram
indice_estado(ref, sha)
fts USING fts5(titulo, cuerpo, contenido)
```

### 3.4 Conversor de engram

- Lee `~/.engram/engram.db` **solo lectura, sobre una copia** (la base está
  en modo WAL; abrirla en vivo falla).
- Proyecto: por `sessions.directory` igual a la raíz del repo y por un mapa
  de alias en la config (`opendocktranding → opendocktrading`).
- Cada observación con `TASK-NNN` → memoria de esa tarea; sin él → memoria
  del proyecto. Conserva `type`, `topic_key` y fechas; las relaciones
  juzgadas (`related`, `supersedes`, `compatible`) pasan a aristas con
  procedencia `engram`.
- Idempotente (por `sync_id` o `normalized_hash`), con `--simular`, y con
  filtro de secretos antes de escribir en git.
- **Decisión para el ADR, recomendada A:** materializar en git (un
  `memoria.md` por tarea en la rama de estado y uno de proyecto), para que el
  índice siga siendo reconstruible y lo vea el equipo. Alternativa B: solo en
  el índice local (no se comparte y se pierde al reconstruir).

### 3.5 Etiquetado sin caer en la locura

1. **Determinista (sin IA):** rutas del diff → etiqueta de módulo por el mapa
   del vocabulario; ADR citados; fase del plan; `tipo`; `dependencias`.
2. **Léxica:** BM25 sobre el cuerpo; solo **propone** etiquetas del vocabulario.
3. **IA acotada, en `finish`:** elige 3 a 5 del vocabulario cerrado; una
   nueva va a una cola de propuestas con motivo y entra si la aprueba una
   persona o aparece en N tareas.

`tc2 tags doctor`: etiquetas con una sola tarea, etiquetas en más del 40 % de
las tareas, casi iguales (distancia de edición o coaparición) y alias sin
resolver. Peso IDF en las relaciones. Ficheros comodín (`CHANGELOG.md`,
`docs/BOARD.md`, `docs/INDEX.md`, `Cargo.lock`) excluidos o con peso casi nulo.

### 3.6 Grafo

Nodos: tareas, etiquetas, ficheros (agrupables por carpeta) y ADR. Aristas:
dependencia, cocambio, etiqueta compartida, engram. Disposición por fuerzas,
modo foco (una tarea y sus vecinas a dos saltos) y umbral de peso para evitar
la maraña.

### 3.7 Visor 3D (`tc2 visor`)

- **Opcional:** solo corre si la persona lo abre. `tc2 visor` levanta un
  servidor HTTP en `127.0.0.1` (puerto configurable, token en la URL) que
  sirve la app Vue **ya compilada y embebida en el ejecutable**: sin Node en
  la máquina de uso, como exige RR-16.
- **Canal de eventos** (SSE o WebSocket) del índice al visor:
  `busqueda {consulta, etiquetas, ficheros, tareas, aristas}`,
  `fichero_cambiado {ruta, mas, menos}`, `fichero_creado {ruta}`,
  `fichero_borrado {ruta}` e `indice_actualizado {sha}`. Los cambios salen del reindexado (§3.2) y,
  para verlos mientras se edita, de un vigilante de ficheros con
  `git diff --numstat` frente a HEAD.
- **Paneles con dockview** (`dockview-vue`, el de ToolsaWeb) y PrimeVue:
  red completa 3D, por ficheros (tareas y ficheros agrupados por carpeta),
  por etiquetas, etiquetas y ficheros juntos, más búsqueda y detalle de la
  tarea. Cada panel es una proyección distinta de los mismos datos.
- **Panel de actividad en tiempo real** (decisión de Carlos, 2026-11-16):
  junto al panel del grafo, otro panel muestra la **pila de búsquedas y de
  ficheros cambiados** según llegan (lo último arriba): qué se está buscando
  (la consulta), con qué etiquetas y qué ficheros y tareas salen; qué fichero
  se modifica (`+líneas` / `-líneas`), se crea o se borra. Así se ve **qué
  está haciendo el agente y dónde**: el grafo muestra el lugar de la red y la
  pila el detalle. Los dos paneles se
  alimentan del **mismo canal de eventos y a la vez**: cada evento entra en la
  pila y dispara su efecto en el grafo en el mismo instante. Pasar el ratón o
  pulsar una entrada de la pila resalta y centra sus nodos en el grafo, y
  pulsar un nodo filtra la pila por él.
- **Efectos:** búsqueda → las etiquetas y aristas del resultado brillan unos
  cientos de milisegundos y emiten partículas a lo largo de las relaciones
  (el «impulso»); líneas añadidas → halo verde en el fichero; líneas
  borradas → halo rojo; los dos, si hay ambos; fichero creado → el nodo
  aparece en la red con un destello propio (azul) unido a su carpeta y a la
  tarea en curso; fichero borrado → el nodo se apaga en rojo y desaparece.
- **Librería propuesta: `3d-force-graph`** (three.js con d3-force-3d, MIT).
  Trae de serie resaltado de nodos y aristas, partículas emitidas a demanda,
  cambios dinámicos de datos y bloom (halo). Es JavaScript puro: se envuelve
  en un componente Vue propio, sin depender de envoltorios sin mantener.
  Su hermana `force-graph` (2D, misma API) sirve para las vistas planas.
  - Descartadas: Reagraph (solo React); cosmos.gl (GPU y enorme escala, pero
    2D; el 3D está en la librería Cosmograph, con licencia por confirmar);
    Sigma.js y Cytoscape.js (2D); `vue-force-graph` (envoltorio sin
    mantenimiento desde 2022).
  - Alternativa a medir: Babylon.js (ya está en ToolsaWeb, con GlowLayer y
    HighlightLayer nativos) más d3-force-3d; un solo motor 3D en los dos
    productos, a cambio de escribir más código.
- **Límite:** cada panel 3D es un contexto WebGL y el navegador admite
  pocos; como máximo 3 o 4 paneles 3D abiertos a la vez, o un renderizador
  compartido.

## 4. Tareas nuevas del plan (PLAN-TASKCODE-2.md)

Fase nueva **Fase 10 — Memoria del proyecto** (sprint 10, 17 tareas) y un ADR en la Fase 1.

| ID | Tarea | Criterio de aceptación | Dep. |
|---|---|---|---|
| F1-19 | ADR: memoria del proyecto (RR-20): git como verdad e índice SQLite reconstruible, ubicación del índice, regla de frescura por SHA, esquema, vocabulario y tres capas de etiquetado, conversor de engram (decisión A/B), retirada de engram del manifiesto | PoC: índice de las tareas terminadas de TaskCodeTwo reconstruido dos veces con el mismo hash; un merge hecho en la plataforma se refleja en la siguiente búsqueda sin hooks | F1-1, F1-9, F1-11, F1-14 |
| F10-1 | Índice SQLite (rusqlite con FTS5 empaquetado), esquema y migraciones, `tc2 indice reconstruir` y `estado` (RF-77) | Dos máquinas con los mismos SHA → mismo contenido; nunca aparece en `git status` | F1-19, F2-5 |
| F10-2 | Frescura por SHA: comprobación en cada lectura, reindexado incremental, reconstrucción ante historia reescrita, transacción con cerrojo (RR-20) | Merge en la plataforma, push de otra persona, rebase y corte a mitad de reindexado: la siguiente búsqueda responde con lo actual; test | F10-1, F2-3 |
| F10-3 | Conversor de las tareas existentes: todas las `tarea.md` y su historial por `tipo(TASK-NNN)`, con renombrados y ficheros comodín (RF-78) | Las tareas de TaskCodeTwo y una copia de ToolsaWeb (404) indexadas; repetirlo no cambia nada | F10-1 |
| F10-4 | Ficheros tocados en `finish`: sección generada `## Archivos tocados` (ruta, nombre, A/M/D/R, líneas) y su relleno para las tareas antiguas (RF-79) | Coincide con `git diff --name-status` del merge; test | F10-3, F7-1 |
| F10-5 | Conversor de engram por proyecto: copia de solo lectura, alias de proyecto, enlace por `TASK-NNN`, relaciones como aristas, idempotente, `--simular`, filtro de secretos, materializado según F1-19 (RF-80) | Con la base real: 146 observaciones, 116 enlazadas a su tarea, alias resuelto, cero duplicados al repetir | F10-1, F10-3 |
| F10-6 | `tc2 buscar`: texto (BM25), etiqueta, fichero y vecinas de una tarea, con salida JSON (RF-81) | Respuesta en menos de 200 ms con 400 tareas; encuentra lo importado de engram | F10-2, F10-3 |
| F10-7 | El plugin busca en el índice: skills y agentes usan `tc2 buscar`; `doctor` avisa de un índice desfasado o de una skill que busca con grep en `tareas/` (RF-81) | Test que busca algo que solo existe en el índice; `doctor` detecta una skill con grep | F10-6, F3-8 |
| F10-8 | Etiquetado determinista: mapa de rutas a etiquetas, ADR citados, fase y tipo en `vocabulario.yml` (RF-82) | Las tareas terminadas de TaskCodeTwo etiquetadas sin IA; resultado reproducible | F10-3 |
| F10-9 | Vocabulario cerrado, alias y `tc2 tags doctor` (RF-82) | Detecta etiqueta única, comodín (>40 %), casi iguales y alias sin resolver | F10-8 |
| F10-10 | Etiquetado léxico e IA acotada en `finish` con cola de propuestas (RF-82) | La IA nunca escribe una etiqueta fuera del vocabulario; una propuesta entra solo con persona o N apariciones | F10-9, F7-1 |
| F10-11 | Relaciones derivadas: cocambio de ficheros con IDF, etiquetas compartidas, dependencias y engram, con peso y procedencia | Las vecinas de una tarea de TaskCodeTwo coinciden con las que daría una persona en 8 de 10 casos | F10-3, F10-8 |
| F10-12 | `tc2 grafo --json`: nodos, aristas, pesos y proyecciones (completa, por ficheros, por etiquetas, etiquetas y ficheros), con modo foco, agrupación por carpeta y umbral (RF-83) | El JSON de ToolsaWeb en modo foco tiene menos de 300 nodos; mismo resultado con los mismos SHA | F10-11 |
| F10-13 | Medición y retirada de engram: preguntas reales en TaskCodeTwo, ToolsaWeb y OpenDockTrading frente a la búsqueda de engram; si gana, engram sale del manifiesto de F1-14 | Informe con aciertos de cada uno; decisión de Carlos registrada | F10-5..F10-12 |
| F10-14 | Prueba de concepto del motor 3D: `3d-force-graph` frente a Babylon.js con d3-force-3d, con el grafo real de ToolsaWeb (RF-84) | Cifras de FPS, memoria y tamaño del bundle con el grafo completo y en modo foco; halo y pulso funcionando en los dos; decisión registrada | F10-12 |
| F10-15 | `tc2 visor`: servidor local en `127.0.0.1` con token, app embebida en el ejecutable y canal de eventos (`busqueda`, `fichero_cambiado`, `fichero_creado`, `fichero_borrado`, `indice_actualizado`) desde el índice y un vigilante de ficheros (RF-84) | Se abre en el navegador sin Node instalado; un cambio en un fichero llega al visor en menos de 1 s; cerrar el visor no afecta a `tc2` | F10-2, F10-6, F10-14 |
| F10-16 | App Vue del visor con dockview y PrimeVue: paneles de red completa 3D, por ficheros, por etiquetas, etiquetas y ficheros, búsqueda y detalle de tarea, con su disposición guardada (RF-84) | Las cuatro vistas abiertas a la vez con el grafo de ToolsaWeb; la disposición se conserva al volver a abrir | F10-15 |
| F10-17 | Efectos del visor y panel de actividad: pulso de la búsqueda (brillo y partículas en etiquetas y relaciones durante unos cientos de ms), halo verde o rojo en el fichero según líneas añadidas o borradas, aparición de los ficheros creados y apagado de los borrados, y panel de dockview con la pila de búsquedas (consulta, etiquetas, ficheros y tareas) y ficheros cambiados, creados o borrados en tiempo real, sincronizado con el grafo (RF-84) | Una búsqueda ilumina exactamente los nodos y aristas devueltos por `tc2 buscar`; añadir y borrar líneas destella verde y rojo en el fichero correcto; un fichero creado aparece en la red unido a la tarea en curso; cada evento aparece en la pila y en el grafo a la vez; pulsar una entrada de la pila resalta y centra sus nodos | F10-16 |

Además: añadir RR-20 a la tabla de requisitos rectores del plan (§2) y al
criterio de salida (§9); F3-11 (aviso de solape) puede usar `tarea_archivo`
de F10-4; F4-1 (`plan`) puede usar `tc2 buscar` para traer tareas parecidas.

Orden (§10): F1-19 → F10-1 → F10-2 y F10-3 → F10-4, F10-5, F10-6 y F10-8 →
F10-7, F10-9 → F10-10, F10-11 → F10-12 → F10-14 → F10-15 → F10-16 →
F10-17, y F10-13 al final. El conversor (F10-3) va
pronto a propósito: con las tareas reales se mide si la red aporta antes de
invertir en la IA.

## 5. Fichero de import para TaskCodeTwo

Copiarlo **fuera del repo** y ejecutar un `taskctl import` por sprint
(F1-19 → sprint 1; F10-1..F10-17 → sprint 10); después, escribir sus
dependencias en el frontmatter con el mapa F-id → TASK del ROADMAP y
regenerar el roadmap.

```markdown
### F1-19 ADR memoria del proyecto: indice de contexto, etiquetas y busqueda
> ADR: memoria del proyecto (RR-20): git como verdad e índice SQLite reconstruible, ubicación del índice, regla de frescura por SHA, esquema, vocabulario y tres capas de etiquetado, conversor de engram (decisión A/B) y retirada de engram del manifiesto.
- Índice de las tareas terminadas de TaskCodeTwo reconstruido dos veces con el mismo hash
- Un merge hecho en la plataforma se refleja en la siguiente búsqueda sin hooks
- Cumple lo que pide la tarea F1-19 del plan y los requisitos que cita

### F10-1 Indice SQLite reconstruible
> Índice SQLite (rusqlite con FTS5 empaquetado), esquema y migraciones, tc2 indice reconstruir y estado (RF-77).
- Dos máquinas con los mismos SHA dan el mismo contenido
- El índice nunca aparece en git status
- Cumple lo que pide la tarea F10-1 del plan y los requisitos que cita

### F10-2 Frescura del indice por SHA
> Comprobación en cada lectura, reindexado incremental, reconstrucción ante historia reescrita y transacción con cerrojo (RR-20).
- Merge en la plataforma, push de otra persona, rebase y corte a mitad de reindexado: la siguiente búsqueda responde con lo actual
- Cumple lo que pide la tarea F10-2 del plan y los requisitos que cita

### F10-3 Conversor de las tareas existentes al indice
> Todas las tarea.md y su historial por tipo(TASK-NNN), con renombrados y ficheros comodín (RF-78).
- Las tareas de TaskCodeTwo y una copia de ToolsaWeb indexadas
- Repetirlo no cambia nada
- Cumple lo que pide la tarea F10-3 del plan y los requisitos que cita

### F10-4 Ficheros tocados en finish
> Sección generada Archivos tocados (ruta, nombre, A/M/D/R, líneas) y su relleno para las tareas antiguas (RF-79).
- Coincide con git diff --name-status del merge
- Cumple lo que pide la tarea F10-4 del plan y los requisitos que cita

### F10-5 Conversor de engram por proyecto
> Copia de solo lectura, alias de proyecto, enlace por TASK-NNN, relaciones como aristas, idempotente, --simular, filtro de secretos y materializado según F1-19 (RF-80).
- Con la base real: 146 observaciones, 116 enlazadas a su tarea, alias resuelto
- Cero duplicados al repetir
- Cumple lo que pide la tarea F10-5 del plan y los requisitos que cita

### F10-6 tc2 buscar
> Búsqueda por texto (BM25), etiqueta, fichero y vecinas de una tarea, con salida JSON (RF-81).
- Respuesta en menos de 200 ms con 400 tareas
- Encuentra lo importado de engram
- Cumple lo que pide la tarea F10-6 del plan y los requisitos que cita

### F10-7 El plugin busca en el indice
> Skills y agentes usan tc2 buscar; doctor avisa de un índice desfasado o de una skill que busca con grep en tareas/ (RF-81).
- Test que busca algo que solo existe en el índice
- doctor detecta una skill con grep
- Cumple lo que pide la tarea F10-7 del plan y los requisitos que cita

### F10-8 Etiquetado determinista
> Mapa de rutas a etiquetas, ADR citados, fase y tipo en vocabulario.yml (RF-82).
- Las tareas terminadas de TaskCodeTwo etiquetadas sin IA
- Resultado reproducible
- Cumple lo que pide la tarea F10-8 del plan y los requisitos que cita

### F10-9 Vocabulario cerrado y tags doctor
> Vocabulario cerrado con alias y tc2 tags doctor (RF-82).
- Detecta etiqueta única, comodín (más del 40 %), casi iguales y alias sin resolver
- Cumple lo que pide la tarea F10-9 del plan y los requisitos que cita

### F10-10 Etiquetado lexico e IA acotada en finish
> Etiquetado léxico e IA acotada al vocabulario en finish, con cola de propuestas (RF-82).
- La IA nunca escribe una etiqueta fuera del vocabulario
- Una propuesta entra solo con persona o N apariciones
- Cumple lo que pide la tarea F10-10 del plan y los requisitos que cita

### F10-11 Relaciones derivadas entre tareas
> Cocambio de ficheros con IDF, etiquetas compartidas, dependencias y engram, con peso y procedencia.
- Las vecinas de una tarea coinciden con las que daría una persona en 8 de 10 casos
- Cumple lo que pide la tarea F10-11 del plan y los requisitos que cita

### F10-12 Grafo de tareas, etiquetas y ficheros en JSON
> tc2 grafo --json: nodos, aristas, pesos y proyecciones (completa, por ficheros, por etiquetas, etiquetas y ficheros), con modo foco, agrupación por carpeta y umbral (RF-83).
- El JSON de ToolsaWeb en modo foco tiene menos de 300 nodos
- Mismo resultado con los mismos SHA
- Cumple lo que pide la tarea F10-12 del plan y los requisitos que cita

### F10-13 Medicion y retirada de engram
> Preguntas reales en TaskCodeTwo, ToolsaWeb y OpenDockTrading frente a la búsqueda de engram; si gana, engram sale del manifiesto de F1-14.
- Informe con aciertos de cada uno
- Decisión de Carlos registrada
- Cumple lo que pide la tarea F10-13 del plan y los requisitos que cita

### F10-14 PoC del motor 3D del visor
> 3d-force-graph frente a Babylon.js con d3-force-3d, con el grafo real de ToolsaWeb (RF-84).
- Cifras de FPS, memoria y tamaño del bundle con el grafo completo y en modo foco
- Halo y pulso funcionando en los dos; decisión registrada
- Cumple lo que pide la tarea F10-14 del plan y los requisitos que cita

### F10-15 tc2 visor: servidor local y canal de eventos
> Servidor en 127.0.0.1 con token, app embebida en el ejecutable y canal de eventos (busqueda, fichero_cambiado, fichero_creado, fichero_borrado, indice_actualizado) desde el índice y un vigilante de ficheros (RF-84).
- Se abre en el navegador sin Node instalado
- Un cambio en un fichero llega al visor en menos de 1 s
- Cerrar el visor no afecta a tc2
- Cumple lo que pide la tarea F10-15 del plan y los requisitos que cita

### F10-16 App Vue del visor con dockview
> Paneles de red completa 3D, por ficheros, por etiquetas, etiquetas y ficheros, búsqueda y detalle de tarea, con dockview y PrimeVue y su disposición guardada (RF-84).
- Las cuatro vistas abiertas a la vez con el grafo de ToolsaWeb
- La disposición se conserva al volver a abrir
- Cumple lo que pide la tarea F10-16 del plan y los requisitos que cita

### F10-17 Efectos del visor y panel de actividad en tiempo real
> Pulso de la búsqueda (brillo y partículas en etiquetas y relaciones unos cientos de ms), halo verde o rojo en el fichero según líneas añadidas o borradas, aparición de los ficheros creados y apagado de los borrados, y panel de dockview con la pila de búsquedas (consulta, etiquetas, ficheros y tareas) y ficheros cambiados, creados o borrados en tiempo real, sincronizado con el grafo (RF-84).
- Una búsqueda ilumina exactamente los nodos y aristas devueltos por tc2 buscar
- Añadir y borrar líneas destella verde y rojo en el fichero correcto
- Un fichero creado aparece en la red unido a la tarea en curso
- Cada evento aparece en la pila y en el grafo a la vez
- Pulsar una entrada de la pila resalta y centra sus nodos en el grafo
- Cumple lo que pide la tarea F10-17 del plan y los requisitos que cita
```

Dependencias (F-id del plan): F1-19 ← F1-1, F1-9, F1-11, F1-14 ·
F10-1 ← F1-19, F2-5 · F10-2 ← F10-1, F2-3 · F10-3 ← F10-1 ·
F10-4 ← F10-3, F7-1 · F10-5 ← F10-1, F10-3 · F10-6 ← F10-2, F10-3 ·
F10-7 ← F10-6, F3-8 · F10-8 ← F10-3 · F10-9 ← F10-8 ·
F10-10 ← F10-9, F7-1 · F10-11 ← F10-3, F10-8 · F10-12 ← F10-11 ·
F10-13 ← F10-5, F10-6, F10-7, F10-8, F10-9, F10-10, F10-11, F10-12 ·
F10-14 ← F10-12 · F10-15 ← F10-2, F10-6, F10-14 · F10-16 ← F10-15 ·
F10-17 ← F10-16.
