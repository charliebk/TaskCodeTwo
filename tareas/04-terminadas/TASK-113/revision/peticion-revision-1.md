# Peticion de revision — TASK-113 (ronda 1)

- Tarea: TASK-113 — Puerta de valor F10: informe, datos y descarte del H10
- Rama revisada: feature/task-113-puerta-de-valor-f10-informe-y-datos-del
- Rama base: develop
- Commit revisado (HEAD): faaaba2f9fcda500f56fc13d74d758f762ffd4be
- Fecha: 2026-11-16
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-113 (criterios de aceptacion y plan)
- Alcance: completa (motivo: primera ronda de revision)

## Instrucciones para el agente revisor

Eres un revisor INDEPENDIENTE del agente que implemento. Tu trabajo es
reproducir empiricamente, no leer el diff y opinar: clona el repo a un
directorio temporal, corre la suite tu mismo (salvo que esta
peticion ya traiga la verificacion de la ronda en verde para el commit revisado;
entonces vale ese resultado) y construye el caso que
rompe el codigo antes de reportarlo. Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. Vuelca tu salida en el
informe de esta ronda (informe-revision-1.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-113/tarea.md
- tareas/03-en-revision/TASK-113/planificacion/
- .gitattributes
- docs/BOARD.md
- docs/INDEX.md
- docs/adr/F1-19-memoria-del-proyecto.md
- docs/adr/README.md
- docs/mediciones/fase10-puerta-valor/README.md
- docs/mediciones/fase10-puerta-valor/corridas.tsv
- docs/mediciones/fase10-puerta-valor/informe.md
- docs/plan/AMPLIACION-2026-11-16.md
- docs/plan/PLAN-TASKCODE-2.md
- docs/plan/REQUISITOS-FUNCIONALES.md

## Commits a revisar (git log develop..HEAD)

````
faaaba2 docs(TASK-113): ROADMAP regenerado sin el H10
75574ec docs(TASK-113): evidencia de la puerta de valor y descarte del H10
470332b chore(TASK-113): tarea en curso
````

## Diff completo (git diff develop..HEAD)

````diff
diff --git a/.gitattributes b/.gitattributes
index 5389f20..d3e48a6 100644
--- a/.gitattributes
+++ b/.gitattributes
@@ -8,3 +8,5 @@ Cargo.lock text eol=lf
 # Esquema publicado de la configuracion (F2-7): se compara byte a byte con
 # el que genera el codigo.
 crates/tc2/esquemas/*.json text eol=lf
+# Evidencia de mediciones (TASK-113): se compara por sha256 con el original.
+docs/mediciones/** -text
diff --git a/docs/BOARD.md b/docs/BOARD.md
index 7e5f5b6..2e77ab2 100644
--- a/docs/BOARD.md
+++ b/docs/BOARD.md
@@ -1,8 +1,8 @@
 # Tablero de tareas
 
-> Generado automaticamente por taskctl finish el 2026-11-16. No editar a mano.
+> Generado automaticamente por taskctl board --escribir el 2026-11-16. No editar a mano.
 
-## Planificadas (00-planificadas) — 81
+## Planificadas (00-planificadas) — 64
 
 ```text
 ID        Titulo                                                               Asignado
@@ -71,23 +71,14 @@ TASK-090  F8-7 Metricas finales de cada tarea y de cada lote                   (
 TASK-091  F9-4b Prueba nocturna de un lote de 10 tareas                        (sin asignar)
 TASK-092  F9-8b Piloto final: TaskCodeTwo con la 2.0 completa                  (sin asignar)
 TASK-093  F9-10 Desinstalacion limpia de TaskCode 0.16                         (sin asignar)
-TASK-096  F10-1 Indice SQLite reconstruible                                    (sin asignar)
-TASK-097  F10-2 Frescura del indice por SHA                                    (sin asignar)
-TASK-098  F10-3 Conversor de las tareas existentes al indice                   (sin asignar)
-TASK-099  F10-4 Ficheros tocados en finish                                     (sin asignar)
-TASK-100  F10-5 Conversor de engram por proyecto                               (sin asignar)
-TASK-101  F10-6 tc2 buscar                                                     (sin asignar)
-TASK-102  F10-7 El plugin busca en el indice                                   (sin asignar)
-TASK-103  F10-8 Etiquetado determinista                                        (sin asignar)
-TASK-104  F10-9 Vocabulario cerrado y tags doctor                              (sin asignar)
-TASK-105  F10-10 Etiquetado lexico e IA acotada en finish                      (sin asignar)
-TASK-106  F10-11 Relaciones derivadas entre tareas                             (sin asignar)
-TASK-107  F10-12 Grafo de tareas, etiquetas y ficheros en JSON                 (sin asignar)
-TASK-108  F10-13 Medicion y retirada de engram                                 (sin asignar)
-TASK-109  F10-14 PoC del motor 3D del visor                                    (sin asignar)
-TASK-110  F10-15 tc2 visor: servidor local y canal de eventos                  (sin asignar)
-TASK-111  F10-16 App Vue del visor con dockview                                (sin asignar)
-TASK-112  F10-17 Efectos del visor y panel de actividad en tiempo real         (sin asignar)
+```
+
+## En curso (02-en-curso) — 1
+
+```text
+ID        Titulo                                                  Asignado
+--------  ------------------------------------------------------  --------------------
+TASK-113  Puerta de valor F10: informe, datos y descarte del H10  charlie.bk@gmail.com
 ```
 
 ## Terminadas (04-terminadas) — 31
diff --git a/docs/INDEX.md b/docs/INDEX.md
index 37a5dc7..3c0714e 100644
--- a/docs/INDEX.md
+++ b/docs/INDEX.md
@@ -3,6 +3,10 @@
 Indice determinista para la recuperacion de contexto por etiquetas
 (seccion 6.1 de la metodologia). Lo actualiza taskctl finish.
 
+## Mediciones
+
+- [Puerta de valor de la Fase 10 (2026-11-16)](mediciones/fase10-puerta-valor/README.md): A/B índice frente a grep, veredicto NO-GO; [informe](mediciones/fase10-puerta-valor/informe.md) y [corridas](mediciones/fase10-puerta-valor/corridas.tsv).
+
 ## Tareas terminadas
 
 - TASK-095 — F1-19 ADR memoria del proyecto: indice de contexto, etiquetas y busqueda · etiquetas: (sin etiquetas) · rama feature/task-095-f1-19-adr-memoria-del-proyecto-indice-de · terminada 2026-11-16 · tareas/04-terminadas/TASK-095/
diff --git a/docs/adr/F1-19-memoria-del-proyecto.md b/docs/adr/F1-19-memoria-del-proyecto.md
index 0bd92ed..6070af4 100644
--- a/docs/adr/F1-19-memoria-del-proyecto.md
+++ b/docs/adr/F1-19-memoria-del-proyecto.md
@@ -1,5 +1,7 @@
 # ADR F1-19 — Memoria del proyecto: git como verdad, índice SQLite reconstruible, frescura, etiquetado y conversor de engram
 
+> **Descartado el 2026-11-16** (TASK-113): la puerta de valor dio NO-GO y Carlos descartó la Fase 10; ver [informe](../mediciones/fase10-puerta-valor/informe.md). Este ADR queda como registro; no se implementa.
+
 - Estado: **aceptada** (2026-11-16)
 - Tarea: TASK-095 · Plan: [F1-19 en PLAN-TASKCODE-2](../plan/PLAN-TASKCODE-2.md) (tabla de la Fase 1 y Fase 10) · Diseño de partida: [AMPLIACION-2026-11-16 §3](../plan/AMPLIACION-2026-11-16.md)
 - Requisitos: RR-20, RF-77 a RF-84 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); se apoya en F1-1, F1-2, F1-9, F1-11 (decisión 10) y F1-14 (manifiesto)
diff --git a/docs/adr/README.md b/docs/adr/README.md
index 8f5c0a7..1201cc1 100644
--- a/docs/adr/README.md
+++ b/docs/adr/README.md
@@ -26,7 +26,7 @@ directo a la tabla de la Fase 1 y a su tarea.
 | [F1-16](F1-16-lotes-desatendidos.md) | Lotes desatendidos con revisión humana al final: `tc2 lote` determinista con núcleo puro (`planificar`, `siguiente_paso`), rama `lote/<nombre>`, operación `integrar-en-lote` (merge por API de la App del agente con el `sha` revisado como CAS) y ruleset `taskcodetwo-lote`, comprobación del primer padre con el árbol de cada merge (D3a), carril del diff recalculado, manifiesto con el núcleo de cada tarea (todo `tarea.md` y el plan menos lo que escriben el CLI y el ciclo, medido en los tres repos: 0 de 1442 escrituras durante una tarea lo cambian; lo excluido es texto no confiable), salida de un commit ajeno en `lote/*` solo por una persona de la lista (reversión exacta o `ajeno-aceptado`), guarda de procedencia antes de `review` y de integrar, escrituras del agente declaradas (la de la persona en el mismo fichero no se detecta), diario `lotes/<nombre>.yml` reconciliado desde Git para `--reanudar`, presupuesto por lote y por tarea, tope de rondas sin ronda extra automática, `[DECIDE CARLOS]` que no entra y nunca para el lote, informe de la mañana en el PR del lote y equivalente en GitLab; «ninguna IA mergea en `develop`» no garantizado en privados gratuitos, solo detectado después; probado con 17 tareas reales de ToolsaWeb sobre una copia y una Plataforma simulada (GitHub y GitLab reales no verificados) | TASK-074 | aceptada |
 | [F1-17](F1-17-progreso-y-estimacion.md) | Progreso en vivo y estimación de tiempo: lector único del histórico con el real recalculado desde `## Transiciones` (huecos de más de 45 min a 45, pausas fuera), estimación por fase (implementación, rondas, cierre; diseño sin calibrar) con mediana y p80 de tareas parecidas en cascada, que fuera de muestra no gana con claridad a las líneas base (ToolsaWeb 0,45 frente a 0,52 de la mediana ingenua; OpenDockTrading 0,43 frente a 0,41) y por eso se enseña como rango junto a la estimación del agente; `estado.json` derivado para la status line O(1) con último valor bueno y rename con reintento en Windows; `tc2 resumen`; `next --personas N` con camino crítico y zonas declaradas (sin zona choca con todas, siempre aviso); `start` que exige la predecesora integrada en `develop` por ascendencia (`merge-base --is-ancestor`, no por el mensaje del merge) y se niega sin red | TASK-075 | aceptada |
 | [F1-18](F1-18-hito-de-decisiones.md) | Hito de decisiones (ADR) en `init` y en `actualizar`: estudio de adopción que lo dimensiona (bloque «uso medido» del informe de F1-12 y `proponer_hito` con un elemento por hueco, bloqueantes antes de migrar), proporcionalidad con mediana y p75 de la Fase 1 (45,5–50 min y 13,9–19,1 M tokens por ADR), registro `hito-decisiones.yml` en `taskcodetwo/tareas` cuya lista, descartes y «no necesario» firma una persona, con estado calculado que falla cerrado, y puerta determinista en `tc2 start` y en el check `estado` (código `rechazada`) | TASK-094 | aceptada |
-| [F1-19](F1-19-memoria-del-proyecto.md) | Memoria del proyecto: git como verdad e índice SQLite en `<git-common-dir>/taskcode/indice.db` reconstruible desde objetos de git (huella SHA-256 de volcado canónico, misma dos veces sobre las 30 tareas terminadas), frescura con una sola política compartida con F3-5b (SHA local en cada lectura, `ls-remote` como mucho una vez por TTL de 60 s y marca «sincronizado hace N s»; `ls-remote` en cada lectura descartado por medición: 760 ms de mediana contra GitHub frente a 200 ms), transacción en sitio con cerrojo propio, vocabulario en la rama de código con `tc2 tags aplicar` como única puerta y etiquetas que se escriben en `finish`, conversor de engram A con salvaguardas obligatorias y retirada de engram condicionada a F10-13 | TASK-095 | aceptada |
+| [F1-19](F1-19-memoria-del-proyecto.md) | **Descartada el 2026-11-16** (TASK-113, [informe](../mediciones/fase10-puerta-valor/informe.md)). Memoria del proyecto: git como verdad e índice SQLite en `<git-common-dir>/taskcode/indice.db` reconstruible desde objetos de git (huella SHA-256 de volcado canónico, misma dos veces sobre las 30 tareas terminadas), frescura con una sola política compartida con F3-5b (SHA local en cada lectura, `ls-remote` como mucho una vez por TTL de 60 s y marca «sincronizado hace N s»; `ls-remote` en cada lectura descartado por medición: 760 ms de mediana contra GitHub frente a 200 ms), transacción en sitio con cerrojo propio, vocabulario en la rama de código con `tc2 tags aplicar` como única puerta y etiquetas que se escriben en `finish`, conversor de engram A con salvaguardas obligatorias y retirada de engram condicionada a F10-13 | TASK-095 | aceptada |
 
 ## Plantilla
 
diff --git a/docs/mediciones/fase10-puerta-valor/README.md b/docs/mediciones/fase10-puerta-valor/README.md
new file mode 100644
index 0000000..d7d672c
--- /dev/null
+++ b/docs/mediciones/fase10-puerta-valor/README.md
@@ -0,0 +1,55 @@
+# Puerta de valor de la Fase 10 (2026-11-16)
+
+Evidencia del experimento que decidió si la Fase 10 (memoria del proyecto:
+índice SQLite, `tc2 buscar`, grafo y visor) aportaba valor a Claude. Resultado:
+**NO-GO** por el criterio fijado antes de medir. Carlos descartó la Fase 10 el
+mismo día (TASK-113).
+
+- [`informe.md`](informe.md): montaje, cifras, criterio, uso real del índice,
+  factores que contaminan la medida y recomendación.
+- [`corridas.tsv`](corridas.tsv): una fila por corrida (64).
+
+Los dos ficheros son copia byte a byte de los del experimento (sha256
+`b50f22cb…` el informe y `b98f1769…` las corridas); por eso no se editan y
+llevan `-text` en `.gitattributes`.
+
+## Cómo se midió
+
+- **Preguntas**: 16 multicapa (necesitan juntar tarea, historial de git y
+  código o ADR), 8 de TaskCodeTwo en `288f332` (T1..T8) y 8 de ToolsaWeb en
+  `882c8bd7` (W1..W8), cada una con su clave de hechos y trampas.
+- **Modos**: `grep` (Grep, Glob, Read y `git`) e `indice` (prototipo
+  `buscar.py` de `tc2 buscar` con FTS5/BM25 sobre tareas, commits con ficheros
+  y docs; más Read y `git`; Grep y Glob prohibidos).
+- **Corridas**: 16 preguntas × 2 modos × 2 repeticiones = 64 subagentes sonnet,
+  sobre copias fijas de los repos.
+- **Métricas**: sacadas de la transcripción jsonl de cada subagente.
+- **Juez**: un agente por pregunta, a ciegas (respuestas barajadas como A-D y
+  sin nombres de herramienta), contra la clave.
+
+## Columnas de `corridas.tsv`
+
+| Columna | Qué es |
+|---|---|
+| `corrida` | Identificador `<pregunta>-<modo>-r<repetición>` |
+| `repo` | `taskcodetwo` o `toolsaweb` |
+| `pregunta` | T1..T8 o W1..W8 |
+| `modo` | `grep` o `indice` |
+| `rep` | Repetición, 1 o 2 |
+| `agent_id` | Id del subagente en la sesión que lo lanzó |
+| `tokens_total` | Suma, por llamada a la API, de entrada + caché creada + caché leída + salida |
+| `tokens_ponderados` | Igual, ponderado por precio relativo: entrada ×1, caché creada ×1,25, caché leída ×0,1, salida ×5 |
+| `tokens_salida` | Tokens de salida |
+| `contexto_final` | Tamaño del contexto en la última llamada |
+| `llamadas_api` | Llamadas a la API (mensajes distintos del asistente) |
+| `segundos` | Del primer al último evento de la transcripción |
+| `llamadas_herr` | Llamadas a herramientas |
+| `bytes_herr` | Bytes devueltos por las herramientas al contexto |
+| `usos_indice` | Llamadas a `buscar.py` |
+| `intentos_grep` | Grep, Glob o grep/rg/find/findstr/Select-String por shell (en modo `indice`, intentos de volver a grep) |
+| `lecturas` | Read o lecturas por shell (cat, head, sed -n…) |
+| `git` | Comandos de shell que usan `git` |
+| `denegadas` | Llamadas rechazadas por el sistema de permisos (sobre todo el clasificador del modo auto) |
+
+La nota del juez no está en esta tabla: el agregado por repo y modo está en el
+informe.
diff --git a/docs/mediciones/fase10-puerta-valor/corridas.tsv b/docs/mediciones/fase10-puerta-valor/corridas.tsv
new file mode 100644
index 0000000..8caac6e
--- /dev/null
+++ b/docs/mediciones/fase10-puerta-valor/corridas.tsv
@@ -0,0 +1,65 @@
+corrida	repo	pregunta	modo	rep	agent_id	tokens_total	tokens_ponderados	tokens_salida	contexto_final	llamadas_api	segundos	llamadas_herr	bytes_herr	usos_indice	intentos_grep	lecturas	git	denegadas
+T1-grep-r1	taskcodetwo	T1	grep	1	afc0c5ca717bde43d	439699	141720	3604	69635	7	111.3	10	17985	0	5	4	5	1
+T1-indice-r1	taskcodetwo	T1	indice	1	afa987a02dc9e86e7	553768	152412	3850	67962	9	114.0	12	18990	4	0	6	5	2
+T2-grep-r1	taskcodetwo	T2	grep	1	a0d2eab7643dd150c	324186	130117	2933	72452	5	91.0	8	25583	0	3	3	3	0
+T2-indice-r1	taskcodetwo	T2	indice	1	a0458680cce3ced0c	595416	166051	4064	75289	9	113.7	12	30011	3	1	4	5	3
+T3-grep-r1	taskcodetwo	T3	grep	1	ad400feaae5f25a85	428103	111498	3769	68135	7	108.9	9	17745	0	6	1	5	1
+T3-indice-r1	taskcodetwo	T3	indice	1	a341900648b6eeaea	718371	155558	5197	75117	11	127.3	16	28846	3	1	10	6	2
+T4-grep-r1	taskcodetwo	T4	grep	1	aa1dab92022a629fd	367917	103485	3502	67540	6	100.3	8	14963	0	2	4	3	0
+T4-indice-r1	taskcodetwo	T4	indice	1	a3617b967ad7497c7	1141413	217689	6020	88843	16	149.7	20	53317	3	1	14	9	2
+T5-grep-r1	taskcodetwo	T5	grep	1	af777100deb04685b	447628	123824	4509	74003	7	107.7	9	26952	0	5	2	3	0
+T5-indice-r1	taskcodetwo	T5	indice	1	a96c286deb27af6cf	616072	147619	4933	78237	9	126.9	10	33625	2	1	4	4	1
+T6-grep-r1	taskcodetwo	T6	grep	1	aefcd07a0d326002e	671517	154714	5598	76750	10	132.2	11	31137	0	7	2	8	1
+T6-indice-r1	taskcodetwo	T6	indice	1	a03021bdc807f1906	774118	175964	5431	87017	11	133.3	14	52300	5	0	7	6	2
+T7-grep-r1	taskcodetwo	T7	grep	1	af5a3eee85e07b7fa	438599	114847	3757	70186	7	111.1	8	20746	0	5	1	4	0
+T7-indice-r1	taskcodetwo	T7	indice	1	a2761946c48727a74	386532	112043	2979	75592	6	98.0	7	33252	4	0	2	0	0
+T8-grep-r1	taskcodetwo	T8	grep	1	ae45bb99d2212940c	642797	153038	4509	82432	9	107.6	12	41408	0	8	2	2	0
+T8-indice-r1	taskcodetwo	T8	indice	1	a95b55a3a8945f27d	382284	106980	2870	72023	6	97.2	8	25553	2	1	3	2	1
+W1-grep-r1	toolsaweb	W1	grep	1	a97eea1d9bacd66e1	514522	135558	3360	83285	7	127.2	8	48758	0	3	3	4	0
+W1-indice-r1	toolsaweb	W1	indice	1	a02bc4adb32264a07	243733	87593	2645	68174	4	91.3	5	19444	1	0	3	1	0
+W2-grep-r1	toolsaweb	W2	grep	1	ab2bfdaadeb637584	455861	138538	4192	87434	6	100.7	13	52319	0	5	6	1	0
+W2-indice-r1	toolsaweb	W2	indice	1	a4cea566b15e49fdc	604741	170206	4691	99896	8	109.5	13	81429	2	1	7	1	2
+W3-grep-r1	toolsaweb	W3	grep	1	aebe3cbe26840d9b3	534609	136074	4650	76489	8	127.9	10	31542	0	7	1	5	1
+W3-indice-r1	toolsaweb	W3	indice	1	aad7fc6152cfed3ee	925702	211333	7416	96132	12	171.3	15	65731	5	1	7	4	2
+W4-grep-r1	toolsaweb	W4	grep	1	a59849904eee1d89f	692059	153931	4447	79187	10	148.2	11	35336	0	8	1	3	1
+W4-indice-r1	toolsaweb	W4	indice	1	acd7d994e7eced6a4	812416	211546	6121	111689	10	147.1	14	100137	4	2	6	2	2
+W5-grep-r1	toolsaweb	W5	grep	1	a5d27acbc70cb6331	588318	146342	5957	75177	9	147.7	10	26613	0	7	1	5	0
+W5-indice-r1	toolsaweb	W5	indice	1	aac6f7af5dd71a4fb	841234	196355	7140	91628	12	163.2	18	57664	6	1	8	3	2
+W6-grep-r1	toolsaweb	W6	grep	1	a119610062e2c918d	505383	120648	3661	69830	8	130.6	8	20859	0	3	3	5	0
+W6-indice-r1	toolsaweb	W6	indice	1	ad5eb06a8a911f374	539642	136667	4324	77956	8	134.6	10	36196	2	2	4	4	1
+W7-grep-r1	toolsaweb	W7	grep	1	a8e7c0fa67e62b031	680401	155120	4867	79445	10	140.0	12	37503	0	7	3	6	0
+W7-indice-r1	toolsaweb	W7	indice	1	a175ba41d9c6b9f44	910781	213389	6558	102874	11	146.8	18	81427	4	1	9	2	3
+W8-grep-r1	toolsaweb	W8	grep	1	a76c6b5a7064adae6	735001	175184	6420	85527	10	153.6	12	51494	0	9	1	4	1
+W8-indice-r1	toolsaweb	W8	indice	1	a4d6ba66ccded85bb	1038952	233098	7872	103265	13	167.1	18	80771	5	1	10	4	2
+T1-grep-r2	taskcodetwo	T1	grep	2	a853a354722c718b8	294585	90767	3066	64717	5	94.3	6	10444	0	2	3	5	1
+T1-indice-r2	taskcodetwo	T1	indice	2	aea9c57bdbdfc7f23	699985	154346	5624	73842	11	123.8	12	23117	3	1	6	5	2
+T2-grep-r2	taskcodetwo	T2	grep	2	a23f8151172054f33	231353	77915	2160	62902	4	84.9	6	11209	0	3	2	3	0
+T2-indice-r2	taskcodetwo	T2	indice	2	ad190d87020542efe	576027	131197	3829	72143	9	106.6	12	28437	3	1	5	5	2
+T3-grep-r2	taskcodetwo	T3	grep	2	a28d668e9b962a451	426693	111297	3718	68300	7	105.5	8	18421	0	5	2	5	0
+T3-indice-r2	taskcodetwo	T3	indice	2	a5ea67d4f642e6aea	448492	121509	4156	73419	7	109.7	9	28247	2	1	4	4	2
+T4-grep-r2	taskcodetwo	T4	grep	2	a94243aa8823dae75	232689	80662	2647	63099	4	80.7	6	11203	0	2	1	2	0
+T4-indice-r2	taskcodetwo	T4	indice	2	a06070f5cd0deca9c	892610	184755	5088	85816	13	131.0	15	50337	4	1	8	5	2
+T5-grep-r2	taskcodetwo	T5	grep	2	a81c06db096cd75dd	330739	115151	4165	78094	5	83.5	5	36668	0	2	1	1	0
+T5-indice-r2	taskcodetwo	T5	indice	2	af5c59f045db6076e	555285	165317	8965	81734	8	122.5	10	32783	2	1	5	3	1
+T6-grep-r2	taskcodetwo	T6	grep	2	a17645952cdee3e14	572548	133495	4533	71444	9	120.1	10	22206	0	8	1	6	1
+T6-indice-r2	taskcodetwo	T6	indice	2	ac36022e8ed240c88	838139	194734	6341	93894	11	138.7	15	66014	4	2	5	8	2
+T7-grep-r2	taskcodetwo	T7	grep	2	a4fc08799fd951edc	316835	102650	3703	70401	5	90.6	9	26092	0	5	2	3	0
+T7-indice-r2	taskcodetwo	T7	indice	2	ae925b54cb92dd05d	663162	148160	4348	77104	10	116.3	12	33880	5	1	4	1	1
+T8-grep-r2	taskcodetwo	T8	grep	2	a00642adf16bfc8ba	500698	135539	3812	82545	7	87.2	10	42426	0	6	2	2	0
+T8-indice-r2	taskcodetwo	T8	indice	2	a467065b7f5dd8145	440840	115720	3547	71645	7	93.4	10	23092	4	1	2	2	1
+W1-grep-r2	toolsaweb	W1	grep	2	abbc44b97c9c6208c	630357	145985	3954	79745	9	205.6	9	41161	0	7	1	4	0
+W1-indice-r2	toolsaweb	W1	indice	2	ac4d116b7be2674c0	387492	109897	3143	72943	6	193.2	7	28541	3	0	3	2	1
+W2-grep-r2	toolsaweb	W2	grep	2	abeb7d7d1d17e8546	282558	119956	4681	84265	4	140.6	12	49982	0	5	5	1	0
+W2-indice-r2	toolsaweb	W2	indice	2	a7d327b6f8e71eb14	612006	171011	3899	103339	8	186.4	14	90540	3	2	7	1	2
+W3-grep-r2	toolsaweb	W3	grep	2	aa78c918234ef4973	478996	130196	4302	77698	7	195.0	9	33565	0	5	2	3	0
+W3-indice-r2	toolsaweb	W3	indice	2	ab5535e6329029976	721933	191998	9542	87982	10	245.9	12	44523	3	1	5	5	1
+W4-grep-r2	toolsaweb	W4	grep	2	acf5c450e54ad3c07	847116	193461	6519	91248	11	197.2	20	55289	0	13	5	1	0
+W4-indice-r2	toolsaweb	W4	indice	2	aced9a2cff222addd	1167167	236299	6954	98808	15	201.6	18	73020	7	0	9	1	3
+W5-grep-r2	toolsaweb	W5	grep	2	a32ef4256c8076b18	537915	146687	5765	80679	8	213.0	9	38339	0	7	1	4	1
+W5-indice-r2	toolsaweb	W5	indice	2	a515729942c4d47ef	844032	201095	7571	93672	11	226.7	18	61447	2	2	12	5	2
+W6-grep-r2	toolsaweb	W6	grep	2	a4bbe0eba380401a8	427656	109884	3560	67661	7	200.2	7	16620	0	3	3	4	0
+W6-indice-r2	toolsaweb	W6	indice	2	ac50ed8dd7e14b409	543100	143040	5080	79976	8	208.2	13	38495	3	3	4	3	2
+W7-grep-r2	toolsaweb	W7	grep	2	a68a741b3f72cf4cd	837310	187920	5835	90197	11	228.2	17	60643	0	7	8	7	0
+W7-indice-r2	toolsaweb	W7	indice	2	a3bfb70d9f0f23bb6	1040237	220344	6244	99000	13	233.5	16	74163	5	1	7	7	4
+W8-grep-r2	toolsaweb	W8	grep	2	a55b94caac79e2f25	818267	188198	6550	89048	11	225.0	12	53107	0	7	3	4	1
+W8-indice-r2	toolsaweb	W8	indice	2	a58b4d744de501e48	999497	214310	6742	95174	13	214.4	16	66114	4	1	9	2	1
diff --git a/docs/mediciones/fase10-puerta-valor/informe.md b/docs/mediciones/fase10-puerta-valor/informe.md
new file mode 100644
index 0000000..dc657e8
--- /dev/null
+++ b/docs/mediciones/fase10-puerta-valor/informe.md
@@ -0,0 +1,49 @@
+# Puerta de valor de la Fase 10 — informe (2026-11-16)
+
+## Montaje
+- 16 preguntas multicapa con clave (8 TaskCodeTwo @288f332, 8 ToolsaWeb @882c8bd7), 2 modos × 2 repeticiones = 64 corridas sonnet.
+- Modo grep: Grep/Glob/Read + git. Modo índice: `buscar.py` (prototipo de `tc2 buscar`: FTS5/BM25 sobre tareas troceadas, commits con ficheros, docs .md; ficha de tarea, búsqueda por fichero y etiqueta) + Read + git; Grep/Glob prohibidos.
+- Juez a ciegas (16 agentes, respuestas barajadas A-D y saneadas) contra la clave: aciertos, errores, cobertura extra, nota 0-10.
+- Métricas de la transcripción de cada subagente (`metricas.py`): tokens procesados (suma de entrada+caché+salida por llamada), tokens ponderados por precio relativo, segundos, llamadas, bytes devueltos por herramientas.
+
+## Resultado (medianas)
+
+| | TaskCodeTwo grep | TaskCodeTwo índice | ToolsaWeb grep | ToolsaWeb índice |
+|---|---:|---:|---:|---:|
+| tokens procesados | 427 k | 606 k (+42 %) | 563 k | 827 k (+47 %) |
+| tokens ponderados | 115 k | 153 k (+33 %) | 146 k | 199 k (+36 %) |
+| segundos | 103 | 119 (+16 %) | 151 | 179 (+19 %) |
+| llamadas a herramientas | 8,5 | 12 | 10,5 | 14,5 |
+| bytes de herramientas | 21 k | 31 k | 40 k | 66 k (+66 %) |
+| acierto (hechos clave) | 1,00 | 1,00 | 0,83 | 1,00 |
+| nota del juez | 9,0 | 9,0 | 8,0 | 9,0 |
+
+Medias: acierto TaskCodeTwo 0,98 / 0,98, ToolsaWeb 0,89 / 0,94; nota TaskCodeTwo 9,1 / 8,8, ToolsaWeb 8,3 / 8,9; errores TaskCodeTwo 2 / 6, ToolsaWeb 4 / 3.
+
+Por pregunta (media de 2 repeticiones), el índice gasta menos tokens en 2 de 16 (T8, W1) y menos tiempo en 2 de 16; en ToolsaWeb acierta más en 4 de 8 (W2, W3, W5, W7) y menos en ninguna.
+
+## Criterio fijado antes de medir
+«Si en ToolsaWeb el índice no baja la mediana de tokens o de tiempo sin perder acierto, la Fase 10 no se construye como está planteada.»
+ToolsaWeb: tokens +47 %, tiempo +19 %, acierto no baja (sube). **No baja ni tokens ni tiempo → NO-GO.**
+
+## Uso real del índice
+- Todas las corridas en modo índice usaron `buscar.py` (de 1 a 6 veces; mediana 3).
+- 26 de 32 (13 por repo) intentaron aun así volver a grep (Grep/Glob o grep por shell) pese a la prohibición.
+
+## Por qué cuesta más
+- El índice no sustituye la lectura: devuelve fragmentos y rutas y el agente lee igual los ficheros (más lecturas, +66 % de bytes en ToolsaWeb).
+- Cada llamada extra cuesta el contexto entero (~47 k tokens fijos de arranque por subagente y creciendo), así que +4 llamadas pesan más que lo que ahorra encontrar antes.
+- Lo que sí aporta: en el repo grande da mejor respuesta (más hechos de historia: tarea, commit, hallazgo de revisión).
+
+## Factores que contaminan el resultado (no ocultos)
+1. Denegaciones del clasificador de auto mode («Undermining Oversight») en `git show/log`: 56 en modo índice frente a 10 en grep. Infla el coste del índice. Estimación gruesa: unas 1,5 llamadas por corrida ≈ 80 k tokens; descontadas, el índice sigue más caro en tokens (≈ +13 % en ToolsaWeb) y en tiempo queda cerca de empate. No cambia el veredicto del criterio en tokens; en tiempo, sí lo deja dudoso.
+2. Prototipo, no `tc2 buscar`: sin la regla 2 de atribución de F1-19, merges sin ficheros, salida no optimizada para tokens.
+3. Preguntas redactadas por un agente que buscó con grep: pueden favorecer al grep.
+4. El juez es un solo agente por pregunta; las diferencias de acierto son pequeñas (n = 16 respuestas por modo y repo).
+5. No se midió el escenario real: índice **y** grep disponibles a la vez.
+
+## Recomendación
+**NO-GO a la Fase 10 tal como está planteada** (índice como motor de búsqueda que sustituye a Grep/Read, y todo lo que cuelga de él: grafo y visor F10-11..F10-17).
+Si se quiere rescatar algo, que sea una propuesta nueva y acotada, con su propia puerta medida:
+- `tc2 buscar` como **brújula de historia** (tarea ↔ commits ↔ ficheros ↔ hallazgos), con salida compacta, **junto a** Grep, no en su lugar;
+- medirlo en el escenario mixto y sin el ruido de denegaciones, con el mismo criterio (bajar tokens o tiempo en ToolsaWeb sin perder acierto).
diff --git a/docs/plan/AMPLIACION-2026-11-16.md b/docs/plan/AMPLIACION-2026-11-16.md
index 1e0b326..b807172 100644
--- a/docs/plan/AMPLIACION-2026-11-16.md
+++ b/docs/plan/AMPLIACION-2026-11-16.md
@@ -1,5 +1,7 @@
 # Pendiente de añadir a TaskCodeTwo: memoria del proyecto (índice de contexto, etiquetas, búsqueda y grafo)
 
+> **Descartado el 2026-11-16** (TASK-113): la puerta de valor dio NO-GO y Carlos descartó la Fase 10; ver [informe](../mediciones/fase10-puerta-valor/informe.md). Las tareas TASK-096..TASK-112 y el hito H10 se borraron.
+
 > **APLICADO el 2026-11-16** en TaskCodeTwo tras integrar TASK-026: plan y
 > requisitos (RR-20, RF-77..RF-84, F1-19 y Fase 10), tareas TASK-095..TASK-112
 > creadas con sus dependencias e hito H10 en la zona manual del ROADMAP. Este
diff --git a/docs/plan/PLAN-TASKCODE-2.md b/docs/plan/PLAN-TASKCODE-2.md
index d8e883f..629d1bf 100644
--- a/docs/plan/PLAN-TASKCODE-2.md
+++ b/docs/plan/PLAN-TASKCODE-2.md
@@ -45,7 +45,7 @@ Detalle en [REQUISITOS-FUNCIONALES.md §0](REQUISITOS-FUNCIONALES.md).
 | RR-17 | Skills, agentes y modelo adecuados a cada tarea | Selección determinista entre lo instalado, anotada en la tarea y medida (F1-15, F3-9, F4-6, F8-6) |
 | RR-18 | Lotes desatendidos con revisión humana al final | `taskctl lote`: rama de lote, sin preguntas, un único PR final (P13; F1-16, F4-7, F4-8, F9-4b) |
 | RR-19 | Progreso en vivo: fases que faltan y tiempo estimado | Estimación por fase, status line y `status --seguir`, métricas finales (F1-17, F3-12, F3-13, F4-9, F8-7) |
-| RR-20 | Memoria del proyecto propia, compartida y nunca desactualizada | Git como verdad e índice SQLite reconstruible con frescura por SHA, etiquetado acotado, búsqueda única `tc2 buscar`, conversores de tareas y de engram, visor 3D opcional (F1-19, Fase 10) |
+| RR-20 | Memoria del proyecto propia, compartida y nunca desactualizada — **descartado el 2026-11-16** | Git como verdad e índice SQLite reconstruible con frescura por SHA, etiquetado acotado, búsqueda única `tc2 buscar`, conversores de tareas y de engram, visor 3D opcional (F1-19, Fase 10) |
 
 ## 3. Qué existe y qué aprendemos
 
@@ -439,6 +439,8 @@ dependencias. Las fases 1 a 9 son tareas de TaskCodeTwo; la 0, del repo TaskCode
 
 ### Fase 10 — Memoria del proyecto (RR-20)
 
+> **Descartado el 2026-11-16** (TASK-113): la puerta de valor dio NO-GO y Carlos descartó la Fase 10; ver [informe](../mediciones/fase10-puerta-valor/informe.md). Se borraron sus tareas (TASK-096..TASK-112) y el hito H10; lo que sigue queda como registro.
+
 Índice de contexto propio que sustituye a engram: git es la fuente de verdad y
 una base SQLite local es un índice reconstruible que nunca queda desactualizado;
 etiquetado automático acotado a un vocabulario, búsqueda única para el plugin,
@@ -499,7 +501,7 @@ motivo en [AMPLIACION-2026-11-16](AMPLIACION-2026-11-16.md).
 4. **F4-5: relevo**, TaskCodeTwo pasa a la 2.0 (requiere F9-2).
 5. Fases 6–8, y la Fase 9 al final, con el piloto en OpenDockTrading antes que
    ToolsaWeb.
-6. **Fase 10** (memoria del proyecto): F1-19 → F10-1 → F10-2 y F10-3 → resto;
+6. ~~**Fase 10** (memoria del proyecto)~~ — descartada el 2026-11-16 (TASK-113): F1-19 → F10-1 → F10-2 y F10-3 → resto;
    el conversor de tareas va pronto para medir con datos reales antes de la IA.
 
 ## Fuentes
diff --git a/docs/plan/REQUISITOS-FUNCIONALES.md b/docs/plan/REQUISITOS-FUNCIONALES.md
index b1295aa..cc82fb7 100644
--- a/docs/plan/REQUISITOS-FUNCIONALES.md
+++ b/docs/plan/REQUISITOS-FUNCIONALES.md
@@ -33,7 +33,7 @@ de ellos, gana el requisito rector.
 | RR-17 | **Skills, agentes y modelo adecuados a cada tarea** | Para cada tarea y cada rol (diseño, implementación, revisión) se eligen las skills, los agentes y el modelo que corresponden según su dominio, complejidad y carril, **solo entre los que están instalados** en la máquina y el proyecto; la elección queda anotada en la tarea y se mide si acierta |
 | RR-18 | **Lotes desatendidos con revisión humana solo al final** | Un comando lanza una secuencia de tareas (rango o lista) que se ejecutan una tras otra sin preguntar: sin trocear, aceptando las recomendaciones, con revisión IA y verificación en cada tarea, y **una sola revisión humana al final del lote** (rama de lote y un único PR contra `develop`). Respeta dependencias, tiene presupuesto, se puede reanudar y deja un informe |
 | RR-19 | **Progreso en vivo en el terminal: fases que faltan y tiempo estimado** | En todo momento, para la tarea en curso: fase actual, fases que faltan y tiempo estimado restante (por fase y total), recalculado con lo real; en un lote, además, tareas que faltan y hora estimada de fin. Se ve en la status line de Claude Code y en `taskctl status --seguir`, lo escribe el CLI (sin tokens del agente). Al cerrar, resumen final: estimado frente a real por fase, tokens y rondas |
-| RR-20 | **Memoria del proyecto propia, compartida y nunca desactualizada** | Git es la fuente de verdad (tareas, etiquetas, ficheros tocados, memoria importada); una base SQLite local es un **índice reconstruible** que nunca se versiona. Toda búsqueda del plugin pasa por ese índice y **nunca responde con datos viejos**: comprueba en cada lectura hasta qué commit está indexado. Etiquetado automático acotado a un vocabulario cerrado. Sin dependencia de engram |
+| RR-20 | **Memoria del proyecto propia, compartida y nunca desactualizada** — **descartado el 2026-11-16** | Git es la fuente de verdad (tareas, etiquetas, ficheros tocados, memoria importada); una base SQLite local es un **índice reconstruible** que nunca se versiona. Toda búsqueda del plugin pasa por ese índice y **nunca responde con datos viejos**: comprueba en cada lectura hasta qué commit está indexado. Etiquetado automático acotado a un vocabulario cerrado. Sin dependencia de engram |
 | RR-14 | **Se construye con la versión actual de TaskCode** | El plan entero se ejecuta como tareas de TaskCode 0.16 en este repo (`taskctl import`, plan, revisión, finish). La 2.0 sustituye a la 0.16 en TaskCode solo cuando puede llevar su propio desarrollo |
 
 ## 1. Uso medido
@@ -203,6 +203,8 @@ Prioridad: **I** imprescindible (uso medido alto o sin él no hay producto),
 
 ### Memoria del proyecto (RR-20, 2026-11-16)
 
+> **Descartado el 2026-11-16** (TASK-113): la puerta de valor dio NO-GO y Carlos descartó la Fase 10; ver [informe](../mediciones/fase10-puerta-valor/informe.md). RF-77..RF-84 no se implementan; quedan como registro sin renumerar.
+
 Uso medido: engram, recurso «conveniente» del ADR F1-14, con 146 observaciones, 59
 sesiones y 228 relaciones en la máquina de Carlos (116 observaciones citan un
 `TASK-NNN`); `etiquetas` vacío en las 94 tareas de TaskCodeTwo.
````

## Excluido del diff (21 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/00-planificadas/TASK-096/tarea.md                                                  | 36 ------------------------------------
 tareas/00-planificadas/TASK-097/tarea.md                                                  | 35 -----------------------------------
 tareas/00-planificadas/TASK-098/tarea.md                                                  | 36 ------------------------------------
 tareas/00-planificadas/TASK-099/tarea.md                                                  | 35 -----------------------------------
 tareas/00-planificadas/TASK-100/tarea.md                                                  | 36 ------------------------------------
 tareas/00-planificadas/TASK-101/tarea.md                                                  | 36 ------------------------------------
 tareas/00-planificadas/TASK-102/tarea.md                                                  | 36 ------------------------------------
 tareas/00-planificadas/TASK-103/tarea.md                                                  | 36 ------------------------------------
 tareas/00-planificadas/TASK-104/tarea.md                                                  | 35 -----------------------------------
 tareas/00-planificadas/TASK-105/tarea.md                                                  | 36 ------------------------------------
 tareas/00-planificadas/TASK-106/tarea.md                                                  | 35 -----------------------------------
 tareas/00-planificadas/TASK-107/tarea.md                                                  | 36 ------------------------------------
 tareas/00-planificadas/TASK-108/tarea.md                                                  | 36 ------------------------------------
 tareas/00-planificadas/TASK-109/tarea.md                                                  | 36 ------------------------------------
 tareas/00-planificadas/TASK-110/tarea.md                                                  | 37 -------------------------------------
 tareas/00-planificadas/TASK-111/tarea.md                                                  | 36 ------------------------------------
 tareas/00-planificadas/TASK-112/tarea.md                                                  | 39 ---------------------------------------
 tareas/{01-en-diseno => 02-en-curso}/TASK-113/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-113/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-113/tarea.md                                    |  3 ++-
 tareas/ROADMAP.md                                                                         | 32 +++-----------------------------
 21 files changed, 5 insertions(+), 642 deletions(-)
````
