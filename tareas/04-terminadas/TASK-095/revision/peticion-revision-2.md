# Peticion de revision — TASK-095 (ronda 2)

- Tarea: TASK-095 — F1-19 ADR memoria del proyecto: indice de contexto, etiquetas y busqueda
- Rama revisada: feature/task-095-f1-19-adr-memoria-del-proyecto-indice-de
- Rama base: develop
- Commit revisado (HEAD): 63e9c84280307befcf399d9343db83e84189c140
- Fecha: 2026-11-16
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-095 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 6e70621ac2c507bb073b6cb6643785dce88ee9a7 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (7 hallazgo(s)))

## Instrucciones para el agente revisor (verificacion acotada)

Eres un revisor INDEPENDIENTE del agente que implemento. La rama ya paso una
revision completa: esta ronda verifica las correcciones de la ronda anterior,
no vuelve a revisar la rama entera ni los MENORES de la ronda anterior (tu
tabla solo trae CRITICOS e IMPORTANTES). Haz esto, y nada mas:

1. Comprueba, uno por uno, que cada hallazgo pendiente de tu tabla de abajo
   queda resuelto por el diff de esta ronda (los marcados `corregido` tambien:
   verificalo, no lo des por bueno). Para un CRITICO o IMPORTANTE, reproduce el
   fallo original y comprueba que ya no ocurre; puedes leer el repo entero si la
   causa no esta en el diff.
2. Ejecuta la suite (o el comando de verificacion del proyecto) sobre el commit
   revisado y di el resultado: verificar sin ejecutar no vale. Si esta peticion
   ya trae la verificacion de la ronda en verde para ese mismo commit, no la
   repitas: cita ese resultado y corre solo los tests de lo que verificas.
3. Busca en el diff de esta ronda el fallo que la correccion abre: es justo
   donde se cuelan los fallos nuevos. Si encuentras uno, construye el caso que
   lo rompe antes de reportarlo.

Solo abre un hallazgo NUEVO si es CRITICO o si lo introduce la propia correccion
(algo que el diff de esta ronda rompe). Lo demas que veas fuera de eso va como
nota al final del informe, no en la tabla: no abre otra ronda.

Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. En la tabla, cada hallazgo pendiente que ya este
resuelto va como `corregido`; el que no, `abierto`. Vuelca tu salida en el
informe de esta ronda (informe-revision-2.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-095/tarea.md
- tareas/03-en-revision/TASK-095/planificacion/
- docs/adr/F1-19-memoria-del-proyecto.md
- docs/adr/README.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 5 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-19-memoria-del-proyecto.md (PoC 1 «Dato real», decision 6, tabla de evidencia RF-78/RF-79) | informe-revision-1.md |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-19-memoria-del-proyecto.md (PoC 2 «6 ms», decision 2, tabla de evidencia F10-6) | informe-revision-1.md |

## Commits a revisar (git log 6e70621ac2c507bb073b6cb6643785dce88ee9a7..HEAD)

````
63e9c84 docs(TASK-095): ADR F1-19 corrige hallazgos de la revision 1
0a47228 chore(TASK-095): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 6e70621ac2c507bb073b6cb6643785dce88ee9a7..HEAD)

````diff
diff --git a/docs/adr/F1-19-memoria-del-proyecto.md b/docs/adr/F1-19-memoria-del-proyecto.md
index 3c10d3d..69199e9 100644
--- a/docs/adr/F1-19-memoria-del-proyecto.md
+++ b/docs/adr/F1-19-memoria-del-proyecto.md
@@ -9,8 +9,8 @@
 
 Hoy la memoria entre sesiones es engram: una base **por máquina**
 (`~/.engram/engram.db`), fuera de git y que el equipo no comparte. El modelo
-de tarea ya tiene `etiquetas` y `dependencias`, pero las 94 tareas tienen
-`etiquetas: []`. RR-20 pide una memoria propia con estas propiedades:
+de tarea ya tiene `etiquetas` y `dependencias`, pero las 112 `tarea.md` de
+`develop` (en `e1bf456`) tienen `etiquetas: []`. RR-20 pide una memoria propia con estas propiedades:
 
 - git como fuente de verdad;
 - un índice SQLite que **nunca se versiona** y se puede reconstruir;
@@ -97,12 +97,24 @@ Lectura:
 - Un árbol de trabajo modificado con `core.autocrlf=true` no la cambia,
   porque se leen blobs y no ficheros.
 - Un commit que cambia una tarea sí la cambia, así que no es un falso verde.
-- Los bytes de dos `.db` reconstruidos en esta máquina también coincidieron
-  (`8a352130…`). Pero el formato de página de SQLite y la fusión de segmentos
-  de FTS5 **no son contrato**: la huella es la del volcado canónico.
-- Dato real: de los 482 commits de `develop`, 120 no se atribuyen a ninguna
-  de las 30 tareas terminadas. Son commits como `docs: ROADMAP regenerado…` o
-  `chore: alta de 18 tareas…`, más los de tareas no terminadas.
+- Los bytes de dos `.db` reconstruidos en la misma máquina también
+  coincidieron entre sí; el revisor lo reprodujo, con otro valor de hash.
+  Pero el formato de página de SQLite y la fusión de segmentos de FTS5 **no
+  son contrato**: la huella es la del volcado canónico.
+- **La atribución solo por asunto se queda corta** (hallazgo IMP-1 de la
+  revisión). De los 482 commits de `develop`, la PoC no atribuye 120 a ninguna
+  de las 30 terminadas. Solo 3 son de tareas no terminadas. El grueso son
+  commits del trabajo de las propias tareas sin `(TASK-NNN)` en el asunto:
+  18 `merge(feature): feature/task-N…`, 16 `chore(taskctl): import…` y unos
+  34 `docs(adr): F1-N corrige hallazgos…`. Consecuencia: de las tareas ADR,
+  solo 2 tienen su `docs/adr/F1-*` entre los ficheros tocados.
+- Medido aparte con `git` sobre el mismo `e1bf456`, se añade la regla del
+  rango del merge: todo commit de `merge^1..merge^2` de un merge cuyo asunto
+  nombra `feature/task-NNN`. Con ella quedan **467 de 482** commits
+  atribuidos y 15 sin atribuir, todos `docs:` del plan o del ROADMAP. Las 18
+  tareas ADR con fichero de ADR quedan **todas** con su ADR. La 19.ª, TASK-001
+  («F1-0 Crear TaskCodeTwo»), no tiene ADR. Esta regla no la ejecuta la PoC;
+  la fija la decisión 3.
 
 ### PoC 2 — merge en la plataforma sin hooks (criterio 2)
 
@@ -129,7 +141,8 @@ fresco=desconocido remoto=no-consultado(185 ms) indexado_hasta=c635c0f692bf... t
 ```
 
 Medición de `ls-remote` con tiempo límite y sin prompt de credenciales
-(`GIT_TERMINAL_PROMPT=0`, `GCM_INTERACTIVE=never`), 15 tomas cada una:
+(`GIT_TERMINAL_PROMPT=0`, `GCM_INTERACTIVE=never`), 15 tomas cada una, en
+esta máquina y en ese momento:
 
 | Remoto | Mín. | Mediana | Máx. |
 |---|---|---|---|
@@ -137,7 +150,10 @@ Medición de `ls-remote` con tiempo límite y sin prompt de credenciales
 | bare local | 143 ms | 149 ms | 155 ms |
 
 Lanzar `git --version` cuesta unos 130 ms en esta máquina, medido desde bash.
-Con F3, ninguna búsqueda cabe en 200 ms, ni siquiera contra un bare local.
+El revisor, en la misma máquina pero con otra carga, midió 609 ms contra el
+bare y 1440 ms contra GitHub. Las cifras varían mucho con la carga, pero la
+conclusión es la misma en las dos mediciones: con F3, ninguna búsqueda cabe
+en 200 ms, ni siquiera contra un bare local.
 
 Segunda pasada, con F4 (TTL de 60 s; la caducidad se simula con `--ttl-s 0`):
 
@@ -155,22 +171,52 @@ hooks ejecutados: 0
 
 Lectura:
 
-- Con F4, las ráfagas de búsquedas responden en **6 ms** desde el índice.
+- Con F4, la parte de la búsqueda **dentro del proceso** tarda de 6 a 7 ms:
+  abrir el índice, comprobar el TTL y consultar FTS5, con `--db` explícito.
+  Esa cifra **no incluye** el arranque del proceso, resolver el directorio
+  común ni comparar las refs locales (hallazgo IMP-2). El revisor midió el
+  proceso completo dentro del TTL: **177 a 289 ms** (6 tomas). Lanzar
+  `git rev-parse --git-common-dir` cuesta por sí solo de 145 a 253 ms. Con F4,
+  la consulta al remoto deja de ser el cuello de botella, pero los 200 ms
+  **no quedan demostrados**: dependen de no lanzar `git` en el camino
+  caliente (decisión 2 y decisión 4), y eso está sin verificar.
 - Dentro del TTL, un merge recién hecho **no aparece, pero la respuesta lo
   dice** («hace 17 s, remoto no consultado»).
 - La primera búsqueda tras caducar el TTL lo encuentra sin hooks ni `fetch`
   manual.
-- La reconstrucción completa de la PoC tarda de 4 a 12 s, casi todo en
-  `log --numstat` sobre 482 commits. El reindexado incremental de F10-2 es lo
-  que lo reduce. Aquí está sin medir.
-
-No verificado:
-
-- la búsqueda que lee la versión anterior con el cerrojo ocupado (decisión 5);
-- la lectura de refs sin lanzar `git` (decisión 4);
-- `rusqlite` con `bundled` en Linux y macOS;
-- copiar `engram.db` con la API de backup;
-- que la 0.16 tolere los campos y secciones nuevas de `tarea.md`.
+- La reconstrucción completa de la PoC tarda de 4 a 12 s en mis
+  ejecuciones y de 15 a 19 s en las del revisor, casi todo en
+  `log --numstat` sobre 482 commits. La dispersión es grande y depende de la
+  carga. El reindexado incremental de F10-2 es lo que lo reduce. Aquí está
+  sin medir.
+- **El criterio 2 se cumple en la primera búsqueda tras caducar el TTL**, no
+  en cualquier búsqueda siguiente: dentro del TTL, el merge no aparece y la
+  respuesta lo avisa. Ver «Textos que cambian» en Consecuencias.
+
+No verificado (diseño sin PoC):
+
+- **Decisión 2 y 4**: resolver el directorio común y leer las refs **sin
+  lanzar `git`** (fichero de ref o `packed-refs`; reftable con `git`). De esto
+  dependen los 200 ms de F10-6.
+- **Decisión 4**: la comparación en cada lectura con las refs de seguimiento
+  locales. La PoC solo mira el TTL y el remoto.
+- **Decisión 3**: la regla de atribución por el rango del merge. Se midió con
+  `git` aparte, y la PoC no la ejecuta.
+- **Decisión 5**:
+  - que `rename` sobre un `.db` abierto falle en Windows;
+  - el modo WAL: la PoC usa el journal por defecto;
+  - la lectura de la versión anterior con el cerrojo ocupado;
+  - la hora de sincronización en la misma transacción que los SHA: la PoC la
+    escribe antes de reindexar y en otra transacción, justo el fallo que la
+    decisión evita.
+- **Decisión 6**:
+  - la huella en dos máquinas o sistemas distintos: se probó en una sola
+    máquina Windows, dos veces y con el revisor;
+  - las tablas `etiqueta`, `arista` y `memoria` y la columna `cambio`: la PoC
+    cubre 5 tablas.
+- `rusqlite` con `bundled` en Linux y macOS.
+- Copiar `engram.db` con la API de backup.
+- Que la 0.16 tolere los campos y secciones nuevas de `tarea.md`.
 
 ## Decisión
 
@@ -183,9 +229,12 @@ No verificado:
    - Fuera de git: el índice y las cachés del grafo.
    - Borrar el índice nunca pierde nada.
 
-2. **Ubicación.** `<git-common-dir>/taskcode/indice.db`, calculada con
-   `git rev-parse --git-common-dir`. En un worktree enlazado `.git` es un
-   fichero, así que una ruta literal fallaría. La comparten todos los
+2. **Ubicación.** `<git-common-dir>/taskcode/indice.db`. En un worktree
+   enlazado `.git` es un fichero, así que una ruta literal fallaría. El
+   directorio común se resuelve **sin lanzar `git` en el camino caliente**:
+   se lee `.git`, o el fichero `gitdir:` y su `commondir`, que es lo que hace
+   `git rev-parse --git-common-dir`. `git` solo se usa como respaldo (no
+   verificado; ver «No verificado»). La comparten todos los
    worktrees y nunca sale en `git status`. Se usa `taskcode/` por coherencia
    con `refs/taskcode/` y `taskcode-estado.lock`. Va junto a
    `taskcode-indice.lock`.
@@ -195,6 +244,16 @@ No verificado:
      - `refs/remotes/origin/taskcode/tareas`: tareas y memoria;
      - `refs/remotes/origin/develop`: vocabulario y diffs;
      - en un repo sin remoto, las ramas locales.
+   - **Regla de atribución commit → tarea** (fija para F10-3 y F10-4), en
+     este orden:
+     1. el commit es de una tarea si su asunto lleva `(TASK-NNN)`;
+     2. si no, si está en `merge^1..merge^2` de un merge `--no-ff` cuyo
+        asunto o rama nombra `feature/task-NNN` (o `hotfix/…`); el merge
+        mismo también es de esa tarea;
+     3. si nada lo atribuye, va a `commit_huerfano` y se cuenta.
+     Un commit que cumple 1 para una tarea y 2 para otra se queda con la del
+     asunto. Medido en `e1bf456`: 467 de 482 atribuidos frente a 362 con la
+     regla 1 sola.
    - Lectura en lote:
      - `cat-file --batch`, un proceso para todos los blobs y no uno por
        fichero: se añade `Subcomando::CatFile` al puerto de F1-4;
@@ -334,13 +393,13 @@ No verificado:
 | RR-20, verdad en git e índice no versionado | A, decisiones 1 y 2 | Índice en `<git-common-dir>`; la PoC lee solo objetos de git |
 | RR-20, nunca datos viejos sin decirlo | F4, decisión 4 | PoC 2: con el TTL caducado aparece TASK-999; dentro del TTL, la marca «hace 17 s» |
 | RF-77, mismo resultado con los mismos SHA | Decisiones 3 y 6 | PoC 1: huella `9527d653…` dos veces; un árbol sucio con `autocrlf` no la cambia |
-| RF-78, conversor de tareas | Decisión 3 | PoC 1: 30 tareas y 362 commits atribuidos, 120 huérfanos contados |
-| RF-79, ficheros tocados | Decisión 6 | PoC 1: 1307 filas tarea-fichero con líneas; A/M/D/R queda para F10-4 |
+| RF-78, conversor de tareas | Decisión 3 | PoC 1: 30 tareas; solo por asunto, 362 commits atribuidos y 120 huérfanos. Con la regla del rango del merge, medida con `git`, 467 de 482 |
+| RF-79, ficheros tocados | Decisión 3 y 6 | PoC 1: 1307 filas tarea-fichero, pero **solo 2 tareas ADR con su ADR**. Con la regla del rango, 18 de 18 (medido con `git`; sin ejecutar en la PoC). A/M/D/R queda para F10-4 |
 | RF-80, conversor de engram | A, decisión 8 | Diseño. La base real (146/116) se verifica en F10-5 |
-| RF-81, `tc2 buscar` único con JSON | Decisión 4 | PoC 2: búsqueda FTS5 con BM25 en 6 ms dentro del TTL |
+| RF-81, `tc2 buscar` único con JSON | Decisión 4 | PoC 2: búsqueda FTS5 con BM25; de 6 a 7 ms dentro del proceso y dentro del TTL |
 | RF-82, etiquetado en tres capas | Decisión 7 | Diseño; las capas escriben en `finish`, no al reconstruir |
 | RF-83 y RF-84, grafo y visor | — | Fuera de alcance (Fase 10, F10-12 a F10-17) |
-| F10-6, menos de 200 ms | F4 frente a F3 | `ls-remote` contra GitHub: 760 ms de mediana (F3 descartada); con F4, de 6 a 7 ms dentro del TTL |
+| F10-6, menos de 200 ms | F4 frente a F3 | `ls-remote` contra GitHub: 760 ms de mediana (F3 descartada). Con F4, de 6 a 7 ms dentro del proceso, pero de 177 a 289 ms el proceso completo de la PoC: **no demostrado**, depende de no lanzar `git` (decisiones 2 y 4) |
 
 ## Consecuencias
 
@@ -361,9 +420,11 @@ No verificado:
   renombrados entre carpetas y la categoría de huérfanos.
 - **F10-4**: `## Archivos tocados` entra en la tabla de secciones de F1-9.
 - **F10-5**: las salvaguardas de la decisión 8 son su criterio de seguridad.
-- **F10-6**: los 200 ms se miden **con el índice al día y dentro del TTL**.
-  La primera búsqueda tras el TTL paga el `ls-remote`, unos 0,8 s con GitHub,
-  y lo dice en la marca.
+- **F10-6**: los 200 ms se miden **con el índice al día y dentro del TTL**, y
+  **de punta a punta, con el arranque del proceso incluido**, que es lo que
+  paga el agente que llama a `tc2 buscar`. La primera búsqueda tras el TTL
+  paga el `ls-remote`, de 0,8 a 1,4 s con GitHub según la carga, y lo dice en
+  la marca.
 - **F10-8 a F10-10**: vocabulario en `.taskcode/vocabulario.yml`,
   `etiquetas_origen`, `tc2 tags aplicar` como única puerta, y cola de
   propuestas en la rama de estado.
@@ -371,12 +432,24 @@ No verificado:
 - **F1-9**: campo nuevo `etiquetas_origen` y sección generada
   `## Archivos tocados`. Antes de F10-4 hay que comprobar que `taskctl` 0.16
   conserva campos y secciones que no conoce.
-- **F1-4**: el puerto gana `Subcomando::CatFile` y `Subcomando::Log`.
-- **F3-5b**: usa la misma política y la misma marca de sincronización.
-- **Texto de RR-20**: hoy dice «comprueba en cada lectura hasta qué commit
-  está indexado». Con esta decisión, la comprobación remota es por TTL y
-  siempre lleva la marca. Ajustar la redacción del requisito lo decide Carlos
-  fuera de este ADR.
+- **F1-4**: el puerto gana `Subcomando::CatFile`, `Subcomando::LsTree` y
+  `Subcomando::Log`, y **la lista permitida del detector de subcomandos de
+  F1-4 se amplía con `cat-file` y `ls-tree`**. Sin eso, el detector marcaría
+  como violación las lecturas del índice.
+- **F3-5b (cambio)**: su fila dice «traen la rama de estado antes de mostrar».
+  Con la política única de la decisión 4, `status` y `roadmap` no traen nada
+  dentro del TTL y muestran «sincronizado hace N s». Es un cambio a la letra
+  de F3-5b que **Carlos tiene que confirmar**. Si no lo acepta, F3-5b
+  conserva el `fetch` previo y la política deja de ser única.
+- **Textos que cambian** y que este ADR no edita (los ajusta Carlos):
+  - **RR-20**: dice «comprueba en cada lectura hasta qué commit está
+    indexado». Pasaría a decir: el SHA local en cada lectura, el remoto como
+    mucho una vez por TTL y siempre se dice desde cuándo.
+  - **Fila F1-19 del plan** y **criterio 2 de TASK-095**: «un merge hecho en
+    la plataforma se refleja en la siguiente búsqueda». Pasaría a decir: «en
+    la primera búsqueda tras caducar el TTL; antes, con la marca».
+  - **Fila F10-2**: «comprobación en cada lectura… la siguiente búsqueda
+    responde con lo actual». Mismo ajuste.
 
 ## Fuera de alcance
 
diff --git a/docs/adr/README.md b/docs/adr/README.md
index c43a246..8f5c0a7 100644
--- a/docs/adr/README.md
+++ b/docs/adr/README.md
@@ -26,7 +26,6 @@ directo a la tabla de la Fase 1 y a su tarea.
 | [F1-16](F1-16-lotes-desatendidos.md) | Lotes desatendidos con revisión humana al final: `tc2 lote` determinista con núcleo puro (`planificar`, `siguiente_paso`), rama `lote/<nombre>`, operación `integrar-en-lote` (merge por API de la App del agente con el `sha` revisado como CAS) y ruleset `taskcodetwo-lote`, comprobación del primer padre con el árbol de cada merge (D3a), carril del diff recalculado, manifiesto con el núcleo de cada tarea (todo `tarea.md` y el plan menos lo que escriben el CLI y el ciclo, medido en los tres repos: 0 de 1442 escrituras durante una tarea lo cambian; lo excluido es texto no confiable), salida de un commit ajeno en `lote/*` solo por una persona de la lista (reversión exacta o `ajeno-aceptado`), guarda de procedencia antes de `review` y de integrar, escrituras del agente declaradas (la de la persona en el mismo fichero no se detecta), diario `lotes/<nombre>.yml` reconciliado desde Git para `--reanudar`, presupuesto por lote y por tarea, tope de rondas sin ronda extra automática, `[DECIDE CARLOS]` que no entra y nunca para el lote, informe de la mañana en el PR del lote y equivalente en GitLab; «ninguna IA mergea en `develop`» no garantizado en privados gratuitos, solo detectado después; probado con 17 tareas reales de ToolsaWeb sobre una copia y una Plataforma simulada (GitHub y GitLab reales no verificados) | TASK-074 | aceptada |
 | [F1-17](F1-17-progreso-y-estimacion.md) | Progreso en vivo y estimación de tiempo: lector único del histórico con el real recalculado desde `## Transiciones` (huecos de más de 45 min a 45, pausas fuera), estimación por fase (implementación, rondas, cierre; diseño sin calibrar) con mediana y p80 de tareas parecidas en cascada, que fuera de muestra no gana con claridad a las líneas base (ToolsaWeb 0,45 frente a 0,52 de la mediana ingenua; OpenDockTrading 0,43 frente a 0,41) y por eso se enseña como rango junto a la estimación del agente; `estado.json` derivado para la status line O(1) con último valor bueno y rename con reintento en Windows; `tc2 resumen`; `next --personas N` con camino crítico y zonas declaradas (sin zona choca con todas, siempre aviso); `start` que exige la predecesora integrada en `develop` por ascendencia (`merge-base --is-ancestor`, no por el mensaje del merge) y se niega sin red | TASK-075 | aceptada |
 | [F1-18](F1-18-hito-de-decisiones.md) | Hito de decisiones (ADR) en `init` y en `actualizar`: estudio de adopción que lo dimensiona (bloque «uso medido» del informe de F1-12 y `proponer_hito` con un elemento por hueco, bloqueantes antes de migrar), proporcionalidad con mediana y p75 de la Fase 1 (45,5–50 min y 13,9–19,1 M tokens por ADR), registro `hito-decisiones.yml` en `taskcodetwo/tareas` cuya lista, descartes y «no necesario» firma una persona, con estado calculado que falla cerrado, y puerta determinista en `tc2 start` y en el check `estado` (código `rechazada`) | TASK-094 | aceptada |
-
 | [F1-19](F1-19-memoria-del-proyecto.md) | Memoria del proyecto: git como verdad e índice SQLite en `<git-common-dir>/taskcode/indice.db` reconstruible desde objetos de git (huella SHA-256 de volcado canónico, misma dos veces sobre las 30 tareas terminadas), frescura con una sola política compartida con F3-5b (SHA local en cada lectura, `ls-remote` como mucho una vez por TTL de 60 s y marca «sincronizado hace N s»; `ls-remote` en cada lectura descartado por medición: 760 ms de mediana contra GitHub frente a 200 ms), transacción en sitio con cerrojo propio, vocabulario en la rama de código con `tc2 tags aplicar` como única puerta y etiquetas que se escriben en `finish`, conversor de engram A con salvaguardas obligatorias y retirada de engram condicionada a F10-13 | TASK-095 | aceptada |
 
 ## Plantilla
````

## Excluido del diff (9 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 6e70621ac2c507bb073b6cb6643785dce88ee9a7..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-095/planificacion/brainstorm/peticion-brainstorm-arquitectura-1.md |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-095/planificacion/brainstorm/peticion-brainstorm-riesgos-1.md      |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-095/planificacion/brainstorm/peticion-unificador-1.md              |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-095/planificacion/brainstorm/salida-brainstorm-arquitectura-1.md   |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-095/planificacion/brainstorm/salida-brainstorm-riesgos-1.md        |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-095/planificacion/plan-final.md                                    |   0
 tareas/03-en-revision/TASK-095/revision/informe-revision-1.md                                                  |  21 ++++
 tareas/03-en-revision/TASK-095/revision/peticion-revision-1.md                                                 | 478 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-095/tarea.md                                                       |   6 +-
 9 files changed, 503 insertions(+), 2 deletions(-)
````
