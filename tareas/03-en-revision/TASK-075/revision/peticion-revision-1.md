# Peticion de revision — TASK-075 (ronda 1)

- Tarea: TASK-075 — F1-17 ADR progreso en vivo y estimacion de tiempo
- Rama revisada: feature/task-075-f1-17-adr-progreso-en-vivo-y-estimacion
- Rama base: develop
- Commit revisado (HEAD): 02291790d1bb0ea610cb0a3fdeaaac8e36557400
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-075 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-075/tarea.md
- tareas/03-en-revision/TASK-075/planificacion/
- docs/adr/F1-17-progreso-y-estimacion.md
- docs/adr/README.md

## Commits a revisar (git log develop..HEAD)

````
0229179 chore(TASK-075): autorrevision
18f6f7c docs(adr): F1-17 progreso en vivo y estimacion, con PoC sobre ToolsaWeb y OpenDockTrading
d9a5b10 chore(TASK-075): coste de diseno
e4df637 chore(TASK-075): tarea en curso
````

## Diff completo (git diff develop..HEAD)

````diff
diff --git a/docs/adr/F1-17-progreso-y-estimacion.md b/docs/adr/F1-17-progreso-y-estimacion.md
new file mode 100644
index 0000000..fd84960
--- /dev/null
+++ b/docs/adr/F1-17-progreso-y-estimacion.md
@@ -0,0 +1,618 @@
+# ADR F1-17 — Progreso en vivo y estimación de tiempo: lector único del histórico con el real recalculado desde `## Transiciones` (huecos de más de 45 min a 45), estimación por fase con mediana y p80 de tareas parecidas, `estado.json` derivado para la status line, `tc2 resumen`, `next --personas N` con zonas declaradas y precondiciones de `start`
+
+- Estado: **aceptada** (2026-11-15)
+- Tarea: TASK-075
+- Plan: [F1-17](../plan/PLAN-TASKCODE-2.md) (dependencias F1-9 y F1-10) y detalle en [la ampliación del 2026-11-14](../plan/AMPLIACION-2026-11-14.md), §5 quater; en lo que toca, §5 ter (F3-11, F3-5b)
+- Requisitos: RR-19 y RR-12 ([catálogo](../plan/REQUISITOS-FUNCIONALES.md)); en lo que toca, RR-01 (lo hace el CLI, sin tokens del agente)
+- Depende de: [ADR F1-9](F1-9-formato-de-tarea-plan-y-especificacion.md) (campo `estimacion`, `## Duracion`, línea `Duracion estimada`), [ADR F1-10](F1-10-carriles-y-modo-automatico.md) (carril declarado y carril mínimo por ficheros), [ADR F1-2](F1-2-escrituras-atomicas-ids-y-toma.md) (punto único de escritura, `SinRed`, `Rechazada`), [ADR F1-3](F1-3-flujo-de-ramas-worktrees-release.md) (`start`, worktree por tarea, merge sin fast-forward), [ADR F1-14](F1-14-distribucion-e-instalacion.md) (la status line llama al binario de Rust), [ADR F1-15](F1-15-seleccion-de-skills-agentes-y-modelo.md) (texto de la tarea no confiable), [ADR F1-16](F1-16-lotes-desatendidos.md) (CAS de procedencia, mediana de una ronda) y [ADR F1-18](F1-18-hito-de-decisiones.md) (presupuesto con mediana y p75)
+- Decide: Carlos, el 2026-11-14 y el 2026-11-15: RR-19 imprescindible; requisitos de la persona (1) `resumen` tipo dashboard, (2) tareas en paralelo en terminales distintas con `start` que se niega si una predecesora no está terminada y avisa del solape, y (3) reparto por camino crítico con `--personas N` y zonas declaradas; seguridad y trabajo en grupo imprescindibles; comandos deterministas sobre hooks; nombres `tc2` y `taskcodetwo`; sin trocear. Respuestas al plan de TASK-075: **D1a a D8a**; D3a y D4a se le preguntaron expresamente por cambiar su forma de trabajar y respondió «Apruebo las dos». El 2026-11-15, durante la implementación, redujo el alcance de la PoC: completa en el lector y en la estimación fuera de muestra, mínima en lo demás.
+
+Este ADR fija **qué estima la 2.0, con qué datos, cuándo lo recalcula y dónde
+lo enseña sin gastar tokens**. Hay un solo lector del histórico,
+`historico(tareas)`, que saca de cada `tarea.md` terminado las muestras con el
+tiempo real **recalculado desde `## Transiciones`**. Encima va un núcleo
+puro: `estimar` da la mediana y el p80 por fase de las tareas parecidas, y
+`restante` da lo que falta a partir de la fase en curso. El CLI recalcula en
+cada transición y deja el resultado en el campo `estimacion` de F1-9 y en un
+`estado.json` derivado que la status line lee de una vez. De ese mismo núcleo
+salen `tc2 resumen`, `next --personas N` (con `repartir`), la hora de fin de un
+lote y las métricas finales.
+
+La PoC se ejecutó sobre las 164 tareas terminadas de ToolsaWeb y las 48 de
+OpenDockTrading, leídas en solo lectura en commits fijos. Su resultado
+principal es incómodo y se dice tal cual: **la estimación por tareas
+parecidas no gana con claridad a las líneas base**. Fuera de muestra, en
+ToolsaWeb mejora a la mediana ingenua del repo (0,45 frente a 0,52 de mediana
+del error relativo), pero pierde contra la «Estimada» del agente en las 21
+tareas que la tienen. En OpenDockTrading empata con la mediana ingenua
+(0,43 frente a 0,41) y gana con holgura a la «Estimada», que allí sobreestima
+2,68 veces. Con la regla que aprobó Carlos en D8a, el modelo «vale solo si
+gana a las dos líneas base»: no lo consigue en ningún repo. Por eso la
+decisión no vende una estimación calibrada. Lo que se enseña es el **rango de
+las tareas parecidas** (mediana y p80, con cuántas y de qué nivel), al lado de
+la línea del agente (D1a), y la calibración se mide fuera de muestra en cada
+versión (F8-7).
+
+## Contexto
+
+**RR-19** pide que, mientras se trabaja, se vea para la tarea en curso la
+fase actual, las fases que faltan y el tiempo estimado restante, por fase y en
+total, recalculado con lo real; en un lote, además, las tareas que faltan y la
+hora estimada de fin. Se ve en la status line de Claude Code y en
+`tc2 status --seguir`, lo escribe el CLI sin tokens del agente, y al cerrar
+hay un resumen final con estimado frente a real por fase, tokens, rondas y
+hallazgos. La tarea F1-17 añade que la estimación sea **por fase**, a partir
+del carril, la complejidad, el tamaño y las tareas parecidas, y que se diga
+cuándo se recalcula, dónde se muestra y cómo se hace sin tokens.
+
+Carlos añadió tres requisitos el 2026-11-14: un comando de resumen tipo
+dashboard que solo lee `tarea.md`; dos tareas en paralelo en terminales
+distintas sin pisarse, con un `start` que se niega si una predecesora no está
+terminada y que avisa si otra tarea en curso toca los mismos ficheros (RR-12);
+y un reparto por camino crítico con `--personas N` y zonas de ficheros
+declaradas. Hubo un prototipo de ese reparto en la sesión: camino crítico de
+16,6 h y 77,1, 38,6, 25,8 y 19,3 h con 1 a 4 personas.
+
+Lo que hay hoy en la 0.16: `taskctl estado [--seguir]`, la status line (fase,
+paso, x/N, minutos), la línea `Duracion estimada: N min` del plan, que escribe
+el agente, y la sección `## Duracion` que el CLI escribe al cerrar, con
+«Estimada» y «Real (de la rama abierta al cierre con merge, sin pausas)».
+`## Transiciones` solo guarda `plan`, `approve`, `start`, `review` (una por
+ronda), `finish` y, a veces, `pausa`. No hay marca de inicio del diseño.
+
+## Alternativas
+
+| Alternativa | Descripción | Veredicto |
+|---|---|---|
+| **El CLI estima con las tareas parecidas ya terminadas** | Núcleo puro `estimar` sobre las muestras de `historico`, cascada de niveles, mediana y p80 | **Elegida**, como rango y no como estimación calibrada (D8a no se cumple; ver Decisión, punto 4) |
+| Que estime el agente (la línea del plan de la 0.16) | Lo de hoy | Se conserva como «estimación del agente» y se mide (D1a). No es el modelo: gasta tokens, no se recalcula y en OpenDockTrading sobreestima 2,68 veces |
+| Estimar con hooks | Un hook calcula y pinta | Descartada: «comandos deterministas sobre hooks» |
+| Recalcular en la status line en cada refresco | O(N) sobre todo el histórico | Descartada: el valor solo cambia en las transiciones; la status line lee un valor derivado (L1) |
+| **D1a** El CLI estima y la línea del plan queda como estimación del agente | Se miden las dos | **Elegida** |
+| **D2a** Mediana y p80 | Colas a la derecha medidas | **Elegida** |
+| **D3a** Reparto, zonas y precondiciones de `start` como contratos aquí | Código en F3-3, F3-6 y F3-11 | **Elegida** |
+| **D4a** Zonas declaradas, cruce con el diff real, sin zona choca con todas, siempre «aviso» | | **Elegida** |
+| **D5a** `estado.json` derivado con el último valor bueno | Para la status line | **Elegida** |
+| **D6a** Real recalculado desde Transiciones con corte de huecos de más de 45 min | Frente a la línea «sin pausas» | **Elegida**, concretada como tope (divergencia 6) |
+| **D7a** Diseño «sin calibrar» | No hay inicio del diseño en el histórico | **Elegida** |
+| **D8a** Mediana, p80 y p90 fuera de muestra, y el error medio también | Frente a la «Estimada» y la mediana ingenua | **Elegida**; el modelo no gana a las dos |
+
+## Pruebas de concepto
+
+Una PoC en Python (`poc-f1-17.py`, en la carpeta de la tarea, con su log
+determinista) implementa las funciones puras del diseño y las ejecuta sobre
+ToolsaWeb en `43ab5907e` (su `develop`), OpenDockTrading en `2b0064f2c` y
+TaskCodeTwo en `d9a5b10fd` (con TASK-075 en curso) y en `8b0184792` (el
+commit en que se hizo el prototipo). Lee los repos **solo con `git log`,
+`git ls-tree` y `git cat-file`**, sin clonarlos y sin escribir nada en ellos.
+Lo que necesita escribir (un remoto bare simulado y ficheros de estado) va a
+`%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-075\`, que se borra al empezar
+y al terminar. Se ejecuta con `python -I -B` en algo más de un minuto.
+
+**Alcance reducido.** El 2026-11-15 Carlos pidió que la PoC tardara pocos
+minutos. Quedan completos el lector del histórico y la estimación fuera de
+muestra con su error frente a las líneas base, que son el criterio de
+aceptación. El corte de huecos, `resumen` y `repartir`, `start` y el solape,
+la status line y la hora de fin de lote se quedan en una comprobación mínima.
+Lo que deja de probarse va en «No verificado».
+
+### H — El histórico
+
+El lector es tolerante: reconoce las secciones por su título, acepta las
+versiones 0.12.0 a 0.16.1 y cuenta los descartes por motivo (fila ilegible,
+sin `start`, sin `review`, sin `finish`, duración negativa). En los dos repos
+no descarta ninguna tarea terminada. La «Estimada» se lee de `## Duracion` y,
+si no se entiende, del plan, quedándose con el primer número que va antes de
+«min»; así se recuperan 1 tarea en ToolsaWeb y 6 en OpenDockTrading que la
+0.16 daba por ilegibles. Las tareas troceadas se agrupan en una sola muestra
+con una regla textual sobre el `## Objetivo` («Segunda parte de TASK-NNN»,
+«Troceo de TASK-NNN», «Troceada en el diseño…», «continuación de»,
+«Sigue a», «TASK-NNN dejó resuelto»).
+
+<!-- tabla-historico:inicio -->
+| Repo | Commit | Terminadas | Descartadas | Usables | Grupos de troceo | Muestras tras agrupar | Con «Estimada» de la 0.16 | De ellas, leída con tolerancia | Con «Real» de la 0.16 |
+|---|---|---|---|---|---|---|---|---|---|
+| ToolsaWeb | `43ab5907e` | 164 | 0 | 164 | 8 (18 tareas) | 154 | 26 | 1 | 27 |
+| OpenDockTrading | `2b0064f2c` | 48 | 0 | 48 | 4 (11 tareas) | 41 | 37 | 6 | 38 |
+<!-- tabla-historico:fin -->
+
+Los carriles salen del diff real de cada tarea con la función de F1-10 (solo
+`docs/` es errata). Es un sustituto del carril declarado, que el histórico no
+tiene. En ToolsaWeb hay 22 erratas, 5 fix y 137 funcionalidades; en
+OpenDockTrading, 47 de 48 son funcionalidad. La complejidad está declarada en
+las 164 tareas de ToolsaWeb y en ninguna de OpenDockTrading.
+
+### R — El real, recalculado (D6a)
+
+Los eventos de una tarea son sus transiciones, las filas de `## Coste` y los
+commits que la nombran en el asunto o que están en su rama (M^1..M^2 del
+merge). El real de cada fase es la suma de los huecos entre eventos
+consecutivos, sin las ventanas de `pausa` (desde la `pausa` hasta la
+transición siguiente, como hace la 0.16). **Un hueco de más de 45 min cuenta
+45 min.**
+
+- **TASK-096 de OpenDockTrading**: la 0.16 dice 414 min. Entre el `start` de
+  las 01:51 y el primer commit de las 08:13 (UTC) pasa una noche sin un solo
+  evento. Recalculado da 76 min (implementación 47, rondas 21, cierre 8). Si
+  el hueco se cortara a 0, la implementación se quedaría en 2 min: es el
+  motivo de leer «cortar» como tope (divergencia 6).
+- **TASK-296 de ToolsaWeb**: tiene una `pausa` sin `reanudar` seguida de
+  `finish`. El reloj de `start` a `finish` da 137 min, y la 0.16 y el
+  recálculo dan 90, porque los dos sacan la pausa. Lo que la ensucia como
+  muestra es otra cosa: es la primera de tres partes (con TASK-299 y TASK-300)
+  y se agrupa.
+- En los dos repos, el «Real» de la 0.16 coincide (±2 min) con el reloj de
+  `start` a `finish` menos las pausas en 27 de 27 tareas de ToolsaWeb y 38 de
+  38 de OpenDockTrading: la 0.16 no corta las noches. Hay 81 huecos de más de
+  45 min en ToolsaWeb y 8 en OpenDockTrading; el corte cambia 56 de 164 tareas
+  de ToolsaWeb y 7 de 48 de OpenDockTrading.
+
+<!-- tabla-corte:inicio -->
+| Regla | Mediana del real, ToolsaWeb (min) | Media, ToolsaWeb | Mediana, OpenDockTrading | Media, OpenDockTrading | TASK-096 de OpenDockTrading | TASK-296 de ToolsaWeb |
+|---|---|---|---|---|---|---|
+| huecos > 45 min a 45 min (D6a) | 111 | 127 | 56 | 64 | 76 | 90 |
+| huecos > 45 min a 0 min | 90 | 105 | 51 | 57 | 31 | 90 |
+| sin corte (reloj menos pausas, como la 0.16) | 116 | 170 | 56 | 90 | 414 | 90 |
+<!-- tabla-corte:fin -->
+
+### E — Estimación fuera de muestra (D7a, D8a)
+
+**Método.** Cada muestra (cada grupo de troceo) se estima solo con las que
+terminaron antes de que se aprobara su plan (rolling origin). Se hace por
+repo, con el otro repo como semilla, y cruzado, solo con el otro repo. Las
+fases medibles son la **implementación** (de `start` a la primera `review`),
+las **rondas** (de la primera a la última `review`) y el **cierre** (de la
+última `review` a `finish`). El diseño queda sin calibrar (D7a).
+
+La cascada usa el primer nivel con al menos 5 muestras, un mínimo fijado antes
+de medir. Los niveles son carril + complejidad + tamaño, carril +
+complejidad, carril, repo, semilla del carril y semilla. El tamaño es el
+número de criterios de aceptación (1-3, 4-6, 7 o más), que se conoce al
+planificar.
+
+El error relativo es |estimado − real| / real. Se da la mediana, el p80 y el
+p90, el porcentaje dentro de ±50 %, el error medio en minutos y en porcentaje
+(el «error medio» del criterio) y el sesgo como mediana de estimado/real.
+Las líneas base son la «Estimada» de la 0.16, la mediana ingenua del repo y
+la mediana por complejidad. En OpenDockTrading no hay complejidad, así que la
+mediana por complejidad es la ingenua.
+
+<!-- tabla-error:inicio -->
+| Ámbito | Estimador | n | Mediana del error relativo | p80 | p90 | Dentro de ±50 % | Error medio (min) | Error medio relativo | Sesgo (estimado/real) |
+|---|---|---|---|---|---|---|---|---|---|
+| ToolsaWeb, por repo | cascada (modelo) | 149 | 0,45 | 0,71 | 1,07 | 55,0 % | 69 | 61,6 % | 0,74 |
+| ToolsaWeb, por repo | mediana ingenua del repo | 149 | 0,52 | 0,72 | 1,37 | 47,7 % | 76 | 66,7 % | 0,70 |
+| ToolsaWeb, por repo | mediana por complejidad | 149 | 0,46 | 0,71 | 1,20 | 53,0 % | 72 | 62,0 % | 0,71 |
+| ToolsaWeb, con «Estimada» | cascada (modelo) | 21 | 0,56 | 0,78 | 1,40 | 42,9 % | 93 | 69,9 % | 0,94 |
+| ToolsaWeb, con «Estimada» | «Estimada» de la 0.16 | 21 | 0,32 | 0,75 | 1,08 | 66,7 % | 94 | 76,4 % | 1,32 |
+| ToolsaWeb, con «Estimada» | mediana ingenua del repo | 21 | 0,55 | 0,90 | 1,44 | 38,1 % | 101 | 65,1 % | 0,64 |
+| ToolsaWeb, con «Estimada» y 3 anteriores | 0.16 calibrada | 18 | 0,29 | 0,61 | 1,90 | 72,2 % | 71 | 61,0 % | 1,14 |
+| ToolsaWeb, cruzado | cascada solo con OpenDockTrading | 22 | 0,67 | 0,76 | 0,79 | 27,3 % | 124 | 59,4 % | 0,38 |
+| ToolsaWeb, cruzado, con «Estimada» | «Estimada» de la 0.16 | 21 | 0,32 | 0,75 | 1,08 | 66,7 % | 94 | 76,4 % | 1,32 |
+| OpenDockTrading, por repo | cascada (modelo) | 40 | 0,43 | 0,77 | 1,57 | 55,0 % | 38 | 66,5 % | 1,21 |
+| OpenDockTrading, por repo | mediana ingenua del repo | 40 | 0,41 | 0,74 | 1,51 | 60,0 % | 37 | 62,7 % | 1,15 |
+| OpenDockTrading, por repo | mediana por complejidad (= ingenua: sin complejidad) | 40 | 0,41 | 0,74 | 1,51 | 60,0 % | 37 | 62,7 % | 1,15 |
+| OpenDockTrading, con «Estimada» | cascada (modelo) | 30 | 0,37 | 0,77 | 1,51 | 53,3 % | 34 | 60,8 % | 1,00 |
+| OpenDockTrading, con «Estimada» | «Estimada» de la 0.16 | 30 | 1,68 | 3,23 | 3,98 | 20,0 % | 111 | 208,0 % | 2,68 |
+| OpenDockTrading, con «Estimada» | mediana ingenua del repo | 30 | 0,40 | 0,73 | 1,51 | 56,7 % | 36 | 62,8 % | 0,95 |
+| OpenDockTrading, con «Estimada» y 3 anteriores | 0.16 calibrada | 26 | 0,61 | 1,00 | 1,91 | 46,2 % | 38 | 73,3 % | 1,48 |
+| OpenDockTrading, cruzado | cascada solo con ToolsaWeb | 41 | 0,72 | 2,04 | 2,77 | 26,8 % | 59 | 124,5 % | 1,72 |
+| OpenDockTrading, cruzado, con «Estimada» | «Estimada» de la 0.16 | 30 | 1,68 | 3,23 | 3,98 | 20,0 % | 111 | 208,0 % | 2,68 |
+<!-- tabla-error:fin -->
+
+Lectura:
+
+- **Contra la mediana ingenua**: en ToolsaWeb el modelo gana (0,45 frente a
+  0,52; dentro de ±50 %, 55,0 % frente a 47,7 %). En un remuestreo pareado, el
+  modelo tiene menor mediana en el 94,2 % de las remuestras. En
+  OpenDockTrading no gana (0,43 frente a 0,41; el modelo es mejor solo en el
+  43,9 % de las remuestras). Allí la cascada no tiene con qué distinguir: no
+  hay complejidad y 47 de 48 tareas son funcionalidad. Por complejidad, en
+  ToolsaWeb, queda casi igual que el modelo (0,46).
+- **Contra la «Estimada» del agente**: en ToolsaWeb, en las 21 tareas que la
+  tienen, gana el agente (0,32 frente a 0,56). En OpenDockTrading, en sus 30,
+  gana el modelo con holgura (0,37 frente a 1,68): allí el agente sobreestima
+  2,68 veces de mediana. La «Estimada» calibrada con la razón real/estimada
+  de las tareas anteriores es lo mejor en ToolsaWeb (0,29) y peor que el
+  modelo en OpenDockTrading (0,61 frente a 0,37 en las mismas 26 tareas).
+- **Cobertura del p80**: el real queda por debajo del p80 estimado en 98 de
+  149 tareas de ToolsaWeb (65,8 %) y en 33 de 41 de OpenDockTrading (80,5 %).
+  En ToolsaWeb el p80 se queda corto: el sesgo de 0,74 dice que las tareas se
+  han ido alargando con el tiempo.
+- **Cruzado**: estimar un repo solo con el otro es peor que con el propio
+  (0,67 en ToolsaWeb y 0,72 en OpenDockTrading), y en OpenDockTrading el p80
+  del error sube a 2,04. Una semilla de otro proyecto da poco: se enseña como
+  «sin calibrar».
+- **Control de fuga**: estimando dentro de muestra el error baja (0,40 en
+  ToolsaWeb, 0,38 en OpenDockTrading), como tiene que pasar si la medida
+  fuera de muestra no tiene fuga.
+- **Tokens**: con las tareas que tienen `## Coste`, la cascada no gana a la
+  mediana ingenua (ToolsaWeb 0,63 frente a 0,53; OpenDockTrading 0,37 frente
+  a 0,37).
+
+Por fase:
+
+<!-- tabla-fases:inicio -->
+| Repo | Fase | n | Real mediano (min) | Error absoluto mediano (min) | p80 del error absoluto (min) | Error relativo mediano |
+|---|---|---|---|---|---|---|
+| ToolsaWeb | implementacion | 149 | 51 | 23 | 47 | 0,46 |
+| ToolsaWeb | rondas | 149 | 37 | 20 | 66 | 0,64 |
+| ToolsaWeb | cierre | 149 | 9 | 4 | 15 | 0,49 |
+| OpenDockTrading | implementacion | 41 | 18 | 9 | 24 | 0,46 |
+| OpenDockTrading | rondas | 41 | 26 | 13 | 26 | 0,49 |
+| OpenDockTrading | cierre | 41 | 10 | 4 | 11 | 0,43 |
+<!-- tabla-fases:fin -->
+
+El recálculo en la primera `review` (lo que falta, rondas + cierre) no mejora
+si el tamaño se mide por el diff en vez de por los criterios: 0,58 frente a
+0,59 en ToolsaWeb y 0,39 frente a 0,39 en OpenDockTrading. Las variantes de la
+cascada y del corte, medidas igual:
+
+<!-- tabla-variantes:inicio -->
+| Variante | ToolsaWeb: mediana | ToolsaWeb: ±50 % | OpenDockTrading: mediana | OpenDockTrading: ±50 % |
+|---|---|---|---|---|
+| cascada entera | 0,45 | 55,0 % | 0,43 | 55,0 % |
+| sin tamano | 0,46 | 55,0 % | 0,43 | 55,0 % |
+| solo carril | 0,48 | 52,3 % | 0,43 | 55,0 % |
+| solo repo (con semilla) | 0,52 | 47,7 % | 0,45 | 57,5 % |
+| semilla del carril antes que el repo | 0,45 | 55,0 % | 0,43 | 55,0 % |
+| sin agrupar las troceadas | 0,43 | 57,2 % | 0,37 | 59,6 % |
+| umbral 30 min | 0,43 | 55,7 % | 0,37 | 57,5 % |
+| umbral 60 min | 0,44 | 54,4 % | 0,47 | 52,5 % |
+| umbral 90 min | 0,45 | 55,0 % | 0,50 | 50,0 % |
+| sin corte | 0,51 | 49,0 % | 0,50 | 50,0 % |
+<!-- tabla-variantes:fin -->
+
+Quitar niveles empeora poco a poco en ToolsaWeb; en OpenDockTrading da igual.
+Un umbral de 30 min da algo menos de error en los dos repos. No se cambia el
+de D6a, porque elegir el umbral con los mismos datos que lo evalúan sería
+ajustarlo dentro de muestra; queda como dato para F8-7.
+
+### D — `resumen` y `repartir` sobre TaskCodeTwo
+
+**El prototipo se reproduce exacto.** En `8b0184792` hay 93 tareas y 82
+pendientes. Con la duración del plan donde la hay (solo TASK-012, 170 min) y
+55 min por defecto, el camino crítico es de 16,6 h y el reparto da 77,1, 38,6,
+25,8 y 19,3 h con 1 a 4 personas, sin mirar zonas.
+
+**Hoy** (`d9a5b10fd`), con la estimación de este ADR, hay 94 tareas, 18
+terminadas y 76 pendientes. Las 18 terminadas son ADR (17 erratas por el diff),
+así que para las 75 funcionalidades pendientes la cascada cae al nivel «repo»,
+que son esos ADR. Para tareas de código es una estimación floja, y se dice
+(«No verificado»). El `resumen` por sprint:
+
+<!-- tabla-resumen:inicio -->
+| Sprint | Terminadas | Pendientes | Tiempo real (h) | Tokens (M) | Falta, mediana (h) | Falta, p80 (h) | Tokens que faltan, mediana (M) |
+|---|---|---|---|---|---|---|---|
+| 1 | 18 | 1 | 22,3 | 550,3 | 0,5 | 1,1 | 19,1 |
+| 2 | 0 | 15 | 0,0 | 0,0 | 18,0 | 22,5 | 446,6 |
+| 3 | 0 | 14 | 0,0 | 0,0 | 16,8 | 21,0 | 417,6 |
+| 4 | 0 | 9 | 0,0 | 0,0 | 10,0 | 14,1 | 263,7 |
+| 5 | 0 | 6 | 0,0 | 0,0 | 6,4 | 10,4 | 174,6 |
+| 6 | 0 | 7 | 0,0 | 0,0 | 7,5 | 11,6 | 204,6 |
+| 7 | 0 | 4 | 0,0 | 0,0 | 4,1 | 7,2 | 111,4 |
+| 8 | 0 | 7 | 0,0 | 0,0 | 7,5 | 11,6 | 204,6 |
+| 9 | 0 | 13 | 0,0 | 0,0 | 15,5 | 19,7 | 388,1 |
+| Total | 18 | 76 | 22,3 | 550,3 | 93,3 | 102,9 | 2320,6 |
+<!-- tabla-resumen:fin -->
+
+Lo que falta en total, sumando medianas, son 61,8 h; la mediana de la suma
+remuestreada da 93,3 h y su p80, 102,9 h. Sumar medianas sale optimista
+porque las colas van a la derecha.
+
+<!-- tabla-reparto:inicio -->
+| Personas | Prototipo de la sesión (h) | Estimación de F1-17, sin mirar zonas (h) | D4a sin zonas declaradas (h) | D4a con zonas de ejemplo (h) |
+|---|---|---|---|---|
+| 1 | 77,1 | 61,8 | 61,8 | 61,8 |
+| 2 | 38,6 | 31,1 | 61,8 | 36,5 |
+| 3 | 25,8 | 21,0 | 61,8 | 29,9 |
+| 4 | 19,3 | 15,5 | 61,8 | 26,2 |
+| camino crítico | 16,6 | 10,3 | 10,3 | 10,3 |
+<!-- tabla-reparto:fin -->
+
+Con D4a y **ninguna zona declarada** (hoy ninguna tarea la tiene) todo va en
+serie: 61,8 h con cualquier número de personas. Con zonas de ejemplo (una por
+fase del plan; las 13 de la Fase 9 y sin F-id, sin zona), el reparto baja a
+36,5, 29,9 y 26,2 h con 2 a 4 personas. Ninguna tarea sin zona corre a la vez
+que otra. Las zonas de ejemplo son sintéticas; lo que se mide es la regla, no
+el reparto real de TaskCodeTwo.
+
+### S — `start` y solape (D3a, D4a), con un remoto bare local
+
+<!-- tabla-start:inicio -->
+| ID | Caso | Resultado |
+|---|---|---|
+| S1 | TASK-002 con su predecesora TASK-001 en curso | se niega (`predecesora-sin-terminar`) |
+| S2 | TASK-001 dice `terminada` en la rama de estado y su merge no está en `develop` | se niega (`predecesora-sin-merge`) |
+| S3 | TASK-001 terminada y con su merge empujado a `develop` | arranca |
+| S4 | sin red, con la caché local diciendo que TASK-001 está terminada e integrada | se niega (`sin-red`) |
+| S5 | `start` de TASK-005 (zona `core/a/x/**`) con TASK-003 en curso en `core/a/**` | aviso: solapa con TASK-003 (companero) |
+| S6 | `start` de TASK-006 (zona `web/**`) con TASK-004 en curso sin zona | aviso: TASK-004 sin zona choca con todas |
+| S7 | `start` de TASK-007 (zona `core/b/**`); la rama de TASK-003 toca `core/b/dos.rs` fuera de su zona | aviso: cruce con el diff real |
+<!-- tabla-start:fin -->
+
+La predecesora tiene que estar `terminada` en la rama de estado **y** tener su
+merge en `origin/develop` (un merge del primer padre con trailer
+`Tarea: TASK-NNN`), después de un `fetch`. Sin red, `start` se niega aunque la
+caché local diga que todo está integrado.
+
+### L — Status line y escrituras concurrentes (D5a)
+
+<!-- tabla-statusline:inicio -->
+| ID | Caso | Resultado |
+|---|---|---|
+| L1 | lectura de la status line con 10 y con 1000 tareas | 1 fichero y los mismos bytes en los dos casos |
+| L2 | `os.replace` con la status line leyendo el fichero (Windows) | sin reintento falla con PermissionError; con reintento entra |
+| L3 | `estado.json` a medio escribir | la status line enseña el último valor bueno con su antigüedad |
+| L4 | dos escritores añaden filas de `## Coste` a la vez | leer-añadir-reescribir pierde 1 de 2 filas; un fichero por fuente, ninguna de 400 |
+| L5 | status line de TASK-075 a las 12:00 UTC | `TASK-075 · implementacion · fases que faltan: rondas, cierre · faltan ≈31 min (p80 66) · n=17 carril+complejidad+tamano · sin actividad registrada hace 127 min` |
+<!-- tabla-statusline:fin -->
+
+En Windows, `os.replace` sobre un fichero que otro proceso tiene abierto
+falla con `PermissionError` (WinError 5). La escritura atómica necesita
+reintento, y el lector, una salida al último valor bueno. La línea de L5
+compara la estimación del CLI para esta misma tarea (mediana 50 min y p80 96
+de 17 ADR parecidos) con los 300 min que puso el agente en el plan.
+
+### F — Hora de fin de un lote
+
+Para cada tramo de 5 tareas seguidas se estima con lo terminado antes de
+lanzar y se compara el real con la suma de medianas y con el p50 y el p80 de
+la suma remuestreada.
+
+<!-- tabla-lote:inicio -->
+| Repo | Lotes de 5 tareas seguidas | Real ≤ suma de medianas | Real ≤ p50 remuestreado | Real ≤ p80 remuestreado |
+|---|---|---|---|---|
+| ToolsaWeb | 145 | 22,1 % | 29,7 % | 46,2 % |
+| OpenDockTrading | 37 | 40,5 % | 37,8 % | 70,3 % |
+<!-- tabla-lote:fin -->
+
+El intervalo remuestreado es más honesto que la suma de medianas, pero **su
+p80 no cubre el 80 %**: el real queda por debajo en el 46,2 % de los tramos de
+ToolsaWeb y en el 70,3 % de los de OpenDockTrading. Las tareas seguidas de un
+mismo tramo no son independientes y el histórico deriva. Con la serie de F1-16
+(17 muestras, lanzada el 2026-11-04 09:28 UTC), la suma de medianas da 30,4 h,
+el p50 34,3 h y el p80 40,1 h, y el real fue de 25,5 h.
+
+### G — Este ADR como dato
+
+La PoC lee este fichero. Las diez tablas entre marcadores tienen que ser
+iguales fila a fila a lo calculado; los IDs de plan y de requisitos tienen que
+existir; los enlaces, resolver; y las secciones, estar. Cuatro copias mutadas
+(una cifra del error cambiada, un ID inexistente, un enlace roto y una sección
+borrada) fallan cada una por su motivo.
+
+## Decisión
+
+1. **Lector único `historico(tareas) -> (muestras, descartes)`** encima del
+   parser de F1-9. Es el único sitio que interpreta `## Transiciones`,
+   `## Duracion` y `## Coste`. Es tolerante (versiones de la 0.12 a la 2.0) y
+   cuenta los descartes por motivo; `doctor` los enseña. **Nunca reescribe** el
+   histórico de una tarea terminada: lo derivado va aparte.
+   - **Real** de cada fase: la suma de los huecos entre eventos (transiciones,
+     filas de `## Coste` y commits de la tarea), sin las ventanas de `pausa`, y
+     con **cada hueco de más de 45 min contado como 45** (D6a). Una duración
+     negativa es un dato inválido y se descarta.
+   - **Troceo**: las partes de una tarea troceada (y los remates declarados en
+     el objetivo) son una sola muestra. En la 2.0 lo declara el propio
+     `trocear` (campo de origen) y no hace falta adivinarlo por el texto.
+2. **Fases medibles** (D7a): implementación (de `start` a la primera
+   `review`), rondas (de la primera a la última `review`) y cierre (de la
+   última `review` a `finish`). El **diseño queda «sin calibrar»**: la 2.0
+   empieza a registrar su inicio (la transición `plan` como marca de inicio
+   del diseño), y cuando haya datos lo calibra F8-7.
+3. **`estimar(tarea, muestras) -> Estimacion`**, función pura. Usa una
+   cascada con al menos 5 muestras por nivel: carril + complejidad + tamaño,
+   carril + complejidad, carril, repo, semilla del carril y semilla. El carril
+   es el declarado (F1-10); el tamaño, el número de criterios; las semillas,
+   las muestras de otros proyectos. Devuelve por fase y en total la **mediana
+   y el p80** (D2a), el **nivel** usado y **n**. Si el nivel es una semilla,
+   se marca «sin calibrar». El tamaño del diff no se usa: medido, no mejora
+   (0,58 frente a 0,59 y 0,39 frente a 0,39).
+4. **Qué se promete (D8a).** Fuera de muestra, el modelo no gana a las dos
+   líneas base en ningún repo. El CLI lo enseña como **rango de tareas
+   parecidas** («≈50 min, p80 96, n=17 ADR») y **nunca como estimación
+   calibrada**. Al lado va la **estimación del agente** del plan (D1a). F8-7
+   publica en el informe por versión el error fuera de muestra de los dos
+   (mediana, p80, p90, ±50 % y error medio), y F3-13 adopta otra regla
+   (por ejemplo, la estimación del agente calibrada por proyecto, que gana en
+   ToolsaWeb y pierde en OpenDockTrading) solo si gana fuera de muestra en
+   los dos pilotos. Nunca se publica una calibración dentro de muestra.
+5. **`restante(estimacion, transiciones, ahora)`**: la distribución de lo que
+   falta entre las muestras del nivel. Si al menos 5 de ellas ya pasaron de lo
+   transcurrido en la fase en curso, se condiciona a esas. Lo transcurrido
+   sigue la regla del real hasta el último evento, más el hueco abierto
+   entero: alguien puede estar trabajando sin commit todavía, y la siguiente
+   transición lo recalcula. Si ese hueco pasa de 45 min, la vista dice «sin
+   actividad registrada hace N min».
+6. **Cuándo se recalcula**: en `approve`, en cada transición y en `finish`.
+   En `finish` se registra el error (estimado frente a real por fase), que es
+   la entrada de F8-7. Recalcularlo todo cuesta O(N) con N tareas y solo pasa
+   en las transiciones.
+7. **Dónde queda**:
+   - campo **`estimacion`** de F1-9, en el mismo commit de estado (F1-2), con
+     esta forma:
+
+     ```yaml
+     estimacion:
+       calculada: "2026-11-15T12:00:00Z"
+       nivel: "carril+complejidad+tamano"
+       n: 17
+       por_fase: {implementacion: [21, 52], rondas: [16, 34], cierre: [6, 12]}
+       total: [50, 96]
+       agente: 300
+     ```
+
+     (pares [mediana, p80] en minutos; `diseno` no aparece mientras esté sin
+     calibrar; las cifras de por fase son de ejemplo);
+   - **`estado.json`** derivado (D5a), fuera de las ramas y sin versionar, en
+     el directorio git del worktree de la tarea (`git rev-parse --git-dir`):
+     tarea, fase, fases que faltan, restante [mediana, p80], nivel, n, hora
+     del cálculo y avisos. Antes de cada escritura se guarda el anterior como
+     `estado.anterior.json`. Cada paso es temporal + rename **con reintento**,
+     porque en Windows el rename falla con el lector abierto (L2).
+8. **Vistas**, todas del CLI y sin tokens:
+   - **status line** (F3-12): abre **un fichero** (`estado.json`), resta el
+     reloj desde la hora del cálculo y pinta la tarea, la fase, las fases que
+     faltan, el rango restante y los avisos. Si el fichero no se lee, enseña el
+     último valor bueno con su antigüedad (L3). Nunca recorre el histórico (L1);
+   - **`tc2 status --seguir`**: la misma lectura, repetida al cambiar el
+     fichero;
+   - **salida de cada transición**: una línea con lo mismo;
+   - **`tc2 resumen`** (requisito 1; F3-5, F8-1): tabla por sprint con
+     terminadas y pendientes, tiempo real y tokens (parcial y total) y lo que
+     falta (mediana y p80 de la **suma remuestreada**, nunca la suma de
+     medianas). O(N), solo cuando se pide; lee solo `tarea.md`;
+   - **lote** (F4-9): tareas que faltan, apartadas y hora de fin como
+     intervalo p50/p80 remuestreado, con el aviso de que ese p80 queda corto
+     (cubre el 46,2 % en ToolsaWeb).
+9. **`## Coste` con varios escritores**: nunca se lee, se añade y se reescribe
+   fuera del punto único de escritura de F1-2. Añadir una fila es una
+   operación de esa transacción, que se re-ejecuta sobre el estado fresco; lo
+   que escribe cada proceso por su cuenta va a un fichero por fuente y se
+   agrega al leer (L4).
+10. **`repartir(dag, duraciones, zonas, n) -> Reparto`** para
+    `next --personas N` (requisito 3; F3-3): listas con prioridad al trabajo
+    restante por el camino más largo; nunca a la vez dos tareas cuyas zonas
+    solapan. Devuelve la lista seguida de cada persona, el total y el camino
+    crítico. Con 1 persona da la suma (comprobación). Las duraciones son las
+    medianas de `estimar`; en la tarea en curso, su restante.
+11. **Zonas** (D4a): campo `zonas` (lista de globs) que se declara en el plan.
+    **Una tarea sin zona choca con todas** y va en serie. Dos zonas solapan si
+    el prefijo literal de un glob es prefijo, por componentes, del de otro. La
+    zona es siempre un **aviso, nunca una garantía**: en `start` y en `review`
+    se cruza con el diff real de cada rama en curso (F3-11), y lo que una rama
+    toca fuera de su zona también avisa (S7).
+12. **Precondiciones de `start`** (D3a, requisito 2; código en F3-6). Primero,
+    `fetch` de `develop` y de la rama de estado. Cada predecesora tiene que
+    estar `terminada` en la rama de estado **y** tener su merge en
+    `origin/develop`. Si no, `Rechazada` (código 3 de F1-2) con el motivo
+    (`predecesora-sin-terminar`, `predecesora-sin-merge`). **Sin red se niega**
+    (`SinRed`, código 4) y explica por qué; nunca arranca a ciegas con la
+    caché. Después avisa del solape (punto 11), con nombres y personas.
+13. **Aviso de cambios ajenos**: en cada transición, el CAS de F1-16 (la
+    cabeza y el `status --porcelain` esperados). La status line solo enseña el
+    aviso; la decisión no depende de ella.
+14. **Métricas finales** (F8-7): `finish` termina con estimado frente a real
+    por fase, tokens, rondas y hallazgos por severidad; el informe por versión
+    da la calibración fuera de muestra del punto 4.
+
+## Divergencias con el plan
+
+1. **F1-9**: la forma del campo `estimacion` la fija este ADR (punto 7), y se
+   añade el campo **`zonas`** (punto 11), que F1-9 no tenía. Además, el
+   `estado.json` derivado es un fichero que no prevén F1-9 ni F1-2: no está
+   versionado, no va en ninguna rama y lo escribe solo el CLI.
+2. **F3-3**: `next` gana `--personas N` con el reparto del punto 10.
+3. **F1-16**: la «mediana de una ronda» del presupuesto del lote sale de
+   `estimar` (la fase de rondas entre el número de rondas de las parecidas),
+   no de una mediana aparte.
+4. **F1-14**: el presupuesto de la status line queda en abrir un fichero y
+   restar el reloj (O(1), L1). El tiempo en Rust dentro de Claude Code lo
+   mide F3-12.
+5. **F1-18**: su proporcionalidad usa mediana y p75; con D2a, cuando exista
+   F3-13 pasa a la mediana y el p80 de `estimar`. F1-18 ya decía que lo
+   recalibraría F1-17.
+6. **D6a** dice «cortando huecos de más de 45 min». Se concreta como **tope**
+   (el hueco cuenta 45), no como eliminación: con el hueco a 0, la
+   implementación de TASK-096 de OpenDockTrading quedaría en 2 min. Además,
+   las ventanas de `pausa` declaradas se descuentan, como hace la 0.16.
+7. **F1-17 en el plan** pide estimar con el «tamaño»: el tamaño del diff
+   medido no mejora la estimación, así que el tamaño es el de la
+   especificación (número de criterios).
+8. **Plan del unificador**: la línea base «mediana ingenua por complejidad»
+   no existe en OpenDockTrading (sin complejidad). Allí la sustituye la
+   mediana del repo.
+9. **D8a**: el modelo no cumple la regla «vale si gana a las dos líneas base».
+   En vez de no enseñar nada, que contradiría RR-19, se enseña como rango sin
+   llamarlo calibrado (punto 4).
+10. **RR-19** cita `taskctl status --seguir`; el comando es
+    `tc2 status --seguir` (F1-12).
+11. **Prototipo de la sesión**: se reproduce exacto (16,6 h; 77,1, 38,6, 25,8
+    y 19,3 h), pero con duraciones de 55 min por defecto y sin zonas. Con la
+    estimación de este ADR y D4a sin zonas declaradas, el reparto de hoy es de
+    61,8 h con cualquier número de personas.
+12. **Lo que se deja a otros**: F3-11 implementa el cruce con el diff y el
+    aviso; F9-4 prueba con tres personas a la vez; F3-6, las precondiciones de
+    `start`; F3-12 y F4-9, las vistas.
+
+## Requisitos y evidencia
+
+| Requisito | Decisión | Evidencia |
+|---|---|---|
+| RR-19 fase actual y fases que faltan | `restante`, `estado.json`, status line | PoC L5: la línea de TASK-075 |
+| RR-19 tiempo restante por fase y total, recalculado con lo real | puntos 3, 5 y 6 | PoC E (fuera de muestra) y L5 |
+| RR-19 sin tokens del agente | todo lo calcula el CLI en las transiciones | PoC L1: la status line abre 1 fichero |
+| RR-19 en un lote, tareas que faltan y hora de fin | intervalo remuestreado (punto 8) | PoC F; su p80 queda corto, y se dice |
+| RR-19 resumen final, estimado frente a real | punto 14 (F8-7) | PoC E: error por fase |
+| Criterio de F1-17: probada contra ToolsaWeb y OpenDockTrading, con su error medio | puntos 3 y 4 | PoC E: `tabla-error` (error medio 69 y 38 min; mediana 0,45 y 0,43) |
+| Requisito (1) de Carlos: `resumen` | punto 8 | PoC D: `tabla-resumen` |
+| Requisito (2) de Carlos: paralelo sin pisarse | puntos 11 y 12 | PoC S1-S7 |
+| Requisito (3) de Carlos: `--personas N` | puntos 10 y 11 | PoC D: el prototipo reproducido; serie sin zonas (D4a) |
+| RR-12 `start` avisa del solape | punto 11 | PoC S5-S7 |
+| RR-12 estado más reciente | `fetch` antes de decidir; sin red se niega | PoC S4 |
+| RR-01 lo hace el CLI | núcleo puro, sin hooks | toda la PoC |
+
+## Consecuencias
+
+- **F3-13** implementa `historico`, `estimar` y `restante`, y escribe
+  `estimacion`; **F3-12**, la status line y `status --seguir` sobre
+  `estado.json`; **F4-9**, el progreso del lote; **F8-7**, las métricas
+  finales y la calibración fuera de muestra por versión.
+- **F3-3**: `next --personas N` con `repartir`; **F3-11**: zonas y aviso de
+  solape con el cruce del diff; **F3-6**: precondiciones de `start`.
+- **F3-5** y **F8-1**: `tc2 resumen`.
+- **F2-5**: el campo `zonas` y la forma de `estimacion` en las plantillas de
+  F1-9; **F4-1**: el plan pide las zonas.
+- Los pilotos empiezan a declarar zonas; hasta entonces, `--personas N` da la
+  serie.
+- **Este repo no cambia** (RR-14): sigue con la 0.16.
+
+## Fuera de alcance
+
+- Lotes en paralelo y varias personas lanzando lotes: F9-4.
+- Cómo se registran los tokens por ronda del agente: F1-16 y F8-1.
+- El aspecto exacto de la status line (colores, anchura): F3-12.
+- La elección de modelo por tarea: F1-15.
+
+## No verificado
+
+- **Alcance reducido por decisión de la persona; se prueba en la
+  implementación (F3-x).** Quedan sin probar en esta PoC:
+  - la sensibilidad del real a otros umbrales en cada tarea (solo se midió el
+    de 45 min y los dos casos conocidos; el efecto de otros umbrales en el
+    error del modelo sí está en `tabla-variantes`) (F3-13);
+  - `start` con el merge de la predecesora hecho en otra máquina sin empujar,
+    con la caché vieja sin `fetch`, y el aviso de cambios ajenos con el CAS de
+    F1-16 (F3-6, F3-11);
+  - la lectura de `estado.json` con escritores y lectores en procesos reales
+    durante un rato (solo se probó el intercalado determinista y el rename con
+    el lector abierto) (F3-12);
+  - las mutaciones de control de `start`, del solape y de la status line, y
+    la comprobación de las cifras de la prosa (G solo mira tablas, IDs,
+    enlaces y secciones).
+- **La status line real de Claude Code**: cómo la invoca, con qué shell y
+  cuánto tarda el binario de Rust leyendo `estado.json`. La PoC es Python
+  (F3-12, F1-14).
+- **La status line y `taskctl estado --seguir` de la 0.16.1**: no se
+  ejecutaron; la base se tomó de la documentación del plan.
+- **El carril del histórico** es el del diff, como sustituto del declarado: un
+  plan que declara errata coincide con un diff solo de `docs/`, pero no está
+  comprobado.
+- **Las tareas de código de TaskCodeTwo** se estiman con sus ADR (nivel
+  «repo»): no hay ninguna terminada de código, así que el `resumen` de hoy
+  probablemente se queda corto. Poner la semilla del carril antes que el repo
+  no cambió nada en ToolsaWeb ni en OpenDockTrading, así que no se pudo
+  decidir con datos.
+- **Que los commits sean una señal de actividad suficiente**: un agente que
+  trabaja más de 45 min sin commit ni coste pierde lo que pase del tope.
+- **El rename en Linux y macOS**: allí no falla con el lector abierto (POSIX);
+  no se ejecutó.
+- **La regla textual de troceo**: agrupa 8 grupos en ToolsaWeb y 4 en
+  OpenDockTrading leyendo el objetivo; no se comprobó a mano que no falte
+  ninguno.
+- **La estimación de tokens**: la cascada no gana a la mediana ingenua; que
+  sirva para el presupuesto de F1-16 no está comprobado.
+- **Las zonas de ejemplo** de TaskCodeTwo son sintéticas; el reparto con
+  zonas reales depende de las que se declaren.
+- **F9-4**: tres personas a la vez no se han probado.
diff --git a/docs/adr/README.md b/docs/adr/README.md
index d1c022c..2aa7c17 100644
--- a/docs/adr/README.md
+++ b/docs/adr/README.md
@@ -24,6 +24,7 @@ directo a la tabla de la Fase 1 y a su tarea.
 | [F1-14](F1-14-distribucion-e-instalacion.md) | Distribución e instalación: CLI en Rust por la regla D1a (gana el 1.º y el 2.º, medidos, y el 4.º, un juicio sobre postject; sin el 4.º la regla no decide) frente a TypeScript con Node SEA (mismo comando, salida idéntica sobre las 404 tareas de ToolsaWeb y las 96 de OpenDockTrading; más rápido en caliente en los 3 repos y 32,1 veces más pequeño), TLS con el almacén del sistema, lanzador sin runtime con `cli.lock` y descarga comprobada por sha256, compatibilidad por protocolo con `tc2 version --json`, canales y firma por etapas (hash y attestation en la 2.0; Authenticode y Developer ID al entrar en winget y Homebrew) y contrato del manifiesto de recursos (`tc2 doctor --recursos`: obligatorio, necesario, conveniente y estorba); Linux y macOS no verificados | TASK-014 | aceptada |
 | [F1-15](F1-15-seleccion-de-skills-agentes-y-modelo.md) | Selección de skills, agentes y modelo por tarea y por rol: función pura `seleccionar` que interpreta tres tablas entre marcadores (reglas, modelo por carril y complejidad, dominios con lista cerrada que la config solo amplía), solo entre lo instalado (inventario de F3-9 y metadatos en la propia skill o en `recursos:`), empate por id sin modelo, suelo de 2 roles con `puertas-seguridad` (17 de 33 IMPORTANTES de la Fase 1 fueron de esa clase; detector con recall 7/8 y precisión 7/9 dentro de muestra), texto de la tarea no confiable que solo sube desde `approve`, revisora del dominio o aviso medido con la config de la base, anotación en `roles.<rol>` y columna «Clase» para medir; 10 tareas reales de ToolsaWeb y OpenDockTrading que Carlos comprobó a mano («Coincide las 10») | TASK-073 | aceptada |
 | [F1-16](F1-16-lotes-desatendidos.md) | Lotes desatendidos con revisión humana al final: `tc2 lote` determinista con núcleo puro (`planificar`, `siguiente_paso`), rama `lote/<nombre>`, operación `integrar-en-lote` (merge por API de la App del agente con el `sha` revisado como CAS) y ruleset `taskcodetwo-lote`, comprobación del primer padre con el árbol de cada merge (D3a), carril del diff recalculado, manifiesto con el núcleo de cada tarea (todo `tarea.md` y el plan menos lo que escriben el CLI y el ciclo, medido en los tres repos: 0 de 1442 escrituras durante una tarea lo cambian; lo excluido es texto no confiable), salida de un commit ajeno en `lote/*` solo por una persona de la lista (reversión exacta o `ajeno-aceptado`), guarda de procedencia antes de `review` y de integrar, escrituras del agente declaradas (la de la persona en el mismo fichero no se detecta), diario `lotes/<nombre>.yml` reconciliado desde Git para `--reanudar`, presupuesto por lote y por tarea, tope de rondas sin ronda extra automática, `[DECIDE CARLOS]` que no entra y nunca para el lote, informe de la mañana en el PR del lote y equivalente en GitLab; «ninguna IA mergea en `develop`» no garantizado en privados gratuitos, solo detectado después; probado con 17 tareas reales de ToolsaWeb sobre una copia y una Plataforma simulada (GitHub y GitLab reales no verificados) | TASK-074 | aceptada |
+| [F1-17](F1-17-progreso-y-estimacion.md) | Progreso en vivo y estimación de tiempo: lector único del histórico con el real recalculado desde `## Transiciones` (huecos de más de 45 min a 45, pausas fuera), estimación por fase (implementación, rondas, cierre; diseño sin calibrar) con mediana y p80 de tareas parecidas en cascada, que fuera de muestra no gana con claridad a las líneas base (ToolsaWeb 0,45 frente a 0,52 de la mediana ingenua; OpenDockTrading 0,43 frente a 0,41) y por eso se enseña como rango junto a la estimación del agente; `estado.json` derivado para la status line O(1) con último valor bueno y rename con reintento en Windows; `tc2 resumen`; `next --personas N` con camino crítico y zonas declaradas (sin zona choca con todas, siempre aviso); `start` que exige el merge de la predecesora en `develop` y se niega sin red | TASK-075 | aceptada |
 | [F1-18](F1-18-hito-de-decisiones.md) | Hito de decisiones (ADR) en `init` y en `actualizar`: estudio de adopción que lo dimensiona (bloque «uso medido» del informe de F1-12 y `proponer_hito` con un elemento por hueco, bloqueantes antes de migrar), proporcionalidad con mediana y p75 de la Fase 1 (45,5–50 min y 13,9–19,1 M tokens por ADR), registro `hito-decisiones.yml` en `taskcodetwo/tareas` cuya lista, descartes y «no necesario» firma una persona, con estado calculado que falla cerrado, y puerta determinista en `tc2 start` y en el check `estado` (código `rechazada`) | TASK-094 | aceptada |
 
 ## Plantilla
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-075/planificacion/brainstorm/peticion-plan-1.md       |    0
 tareas/{01-en-diseno => 02-en-curso}/TASK-075/planificacion/brainstorm/salida-arquitectura-1.md |    0
 tareas/{01-en-diseno => 02-en-curso}/TASK-075/planificacion/brainstorm/salida-riesgos-1.md      |    0
 tareas/{01-en-diseno => 02-en-curso}/TASK-075/planificacion/plan-final.md                       |    0
 tareas/02-en-curso/TASK-075/poc/poc-f1-17.log                                                   |  254 ++++++++++++++
 tareas/02-en-curso/TASK-075/poc/poc-f1-17.py                                                    | 1712 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{01-en-diseno => 02-en-curso}/TASK-075/tarea.md                                          |   17 +-
 7 files changed, 1981 insertions(+), 2 deletions(-)
````
