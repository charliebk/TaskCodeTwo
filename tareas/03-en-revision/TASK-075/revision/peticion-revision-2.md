# Peticion de revision — TASK-075 (ronda 2)

- Tarea: TASK-075 — F1-17 ADR progreso en vivo y estimacion de tiempo
- Rama revisada: feature/task-075-f1-17-adr-progreso-en-vivo-y-estimacion
- Rama base: develop
- Commit revisado (HEAD): b516dee870028718a61296eec40038c470640855
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-075 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 02291790d1bb0ea610cb0a3fdeaaac8e36557400 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (8 hallazgo(s)))

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

- tareas/03-en-revision/TASK-075/tarea.md
- tareas/03-en-revision/TASK-075/planificacion/
- docs/adr/F1-17-progreso-y-estimacion.md
- docs/adr/README.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 6 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-17-progreso-y-estimacion.md:226 (Control de fuga) y tareas/03-en-revision/TASK-075/poc/poc-f1-17.py:1145 | informe-revision-1.md |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-17-progreso-y-estimacion.md:335 y :486 (decision 12) y tareas/03-en-revision/TASK-075/poc/poc-f1-17.py:950 (`mergeada`) | informe-revision-1.md |

## Commits a revisar (git log 02291790d1bb0ea610cb0a3fdeaaac8e36557400..HEAD)

````
b516dee docs(adr): F1-17 corrige hallazgos de la revision 1
1be8607 chore(TASK-075): informe de revision 1 (cambios solicitados) y coste
89298a2 chore(TASK-075): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 02291790d1bb0ea610cb0a3fdeaaac8e36557400..HEAD)

````diff
diff --git a/docs/adr/F1-17-progreso-y-estimacion.md b/docs/adr/F1-17-progreso-y-estimacion.md
index fd84960..9bbb010 100644
--- a/docs/adr/F1-17-progreso-y-estimacion.md
+++ b/docs/adr/F1-17-progreso-y-estimacion.md
@@ -223,9 +223,16 @@ Lectura:
   (0,67 en ToolsaWeb y 0,72 en OpenDockTrading), y en OpenDockTrading el p80
   del error sube a 2,04. Una semilla de otro proyecto da poco: se enseña como
   «sin calibrar».
-- **Control de fuga**: estimando dentro de muestra el error baja (0,40 en
-  ToolsaWeb, 0,38 en OpenDockTrading), como tiene que pasar si la medida
-  fuera de muestra no tiene fuga.
+- **Control de fuga**: `disponibles` solo deja entrar las muestras que
+  terminaron antes de aprobarse el plan de la tarea estimada, y nunca la
+  propia. El control lo comprueba sobre lo que usó cada estimación: fuera de
+  muestra, 0 muestras posteriores en los dos repos. Con una fuga metida a
+  propósito (todo el pool menos la propia tarea) salen 2515 en ToolsaWeb y 734
+  en OpenDockTrading, y el control falla. La estimación dentro de muestra se
+  imprime solo como referencia y no prueba nada: en ToolsaWeb baja la mediana
+  (0,40 frente a 0,45) y sube el p80 (1,16 frente a 0,71), el p90 (2,04 frente
+  a 1,07) y el error medio (78,8 % frente a 61,6 %); además su n es otra (154
+  frente a 149).
 - **Tokens**: con las tareas que tienen `## Coste`, la cascada no gana a la
   mediana ingenua (ToolsaWeb 0,63 frente a 0,53; OpenDockTrading 0,37 frente
   a 0,37).
@@ -256,17 +263,18 @@ cascada y del corte, medidas igual:
 | solo carril | 0,48 | 52,3 % | 0,43 | 55,0 % |
 | solo repo (con semilla) | 0,52 | 47,7 % | 0,45 | 57,5 % |
 | semilla del carril antes que el repo | 0,45 | 55,0 % | 0,43 | 55,0 % |
-| sin agrupar las troceadas | 0,43 | 57,2 % | 0,37 | 59,6 % |
-| umbral 30 min | 0,43 | 55,7 % | 0,37 | 57,5 % |
-| umbral 60 min | 0,44 | 54,4 % | 0,47 | 52,5 % |
-| umbral 90 min | 0,45 | 55,0 % | 0,50 | 50,0 % |
-| sin corte | 0,51 | 49,0 % | 0,50 | 50,0 % |
+| sin agrupar las troceadas (cambia las muestras) | 0,43 | 57,2 % | 0,37 | 59,6 % |
+| umbral 30 min (cambia también el real) | 0,43 | 55,7 % | 0,37 | 57,5 % |
+| umbral 60 min (cambia también el real) | 0,44 | 54,4 % | 0,47 | 52,5 % |
+| umbral 90 min (cambia también el real) | 0,45 | 55,0 % | 0,50 | 50,0 % |
+| sin corte (cambia también el real) | 0,51 | 49,0 % | 0,50 | 50,0 % |
 <!-- tabla-variantes:fin -->
 
 Quitar niveles empeora poco a poco en ToolsaWeb; en OpenDockTrading da igual.
-Un umbral de 30 min da algo menos de error en los dos repos. No se cambia el
-de D6a, porque elegir el umbral con los mismos datos que lo evalúan sería
-ajustarlo dentro de muestra; queda como dato para F8-7.
+Las filas de umbral y de agrupado **no son comparables** con las demás: cambian
+también el real que se estima (o las muestras), así que un umbral más bajo
+acorta y homogeneiza el objetivo y baja el error relativo por construcción. No
+dicen qué umbral es mejor, y D6a no se toca.
 
 ### D — `resumen` y `repartir` sobre TaskCodeTwo
 
@@ -276,9 +284,10 @@ pendientes. Con la duración del plan donde la hay (solo TASK-012, 170 min) y
 25,8 y 19,3 h con 1 a 4 personas, sin mirar zonas.
 
 **Hoy** (`d9a5b10fd`), con la estimación de este ADR, hay 94 tareas, 18
-terminadas y 76 pendientes. Las 18 terminadas son ADR (17 erratas por el diff),
-así que para las 75 funcionalidades pendientes la cascada cae al nivel «repo»,
-que son esos ADR. Para tareas de código es una estimación floja, y se dice
+terminadas y 76 pendientes. De las 18 terminadas, 17 son ADR (erratas por el
+diff) y una, F1-0, es la creación del repo (funcionalidad por el diff). Para
+las 75 funcionalidades pendientes la cascada cae al nivel «repo», que son esas
+18. Para tareas de código es una estimación floja, y se dice
 («No verificado»). El `resumen` por sprint:
 
 <!-- tabla-resumen:inicio -->
@@ -324,17 +333,24 @@ el reparto real de TaskCodeTwo.
 |---|---|---|
 | S1 | TASK-002 con su predecesora TASK-001 en curso | se niega (`predecesora-sin-terminar`) |
 | S2 | TASK-001 dice `terminada` en la rama de estado y su merge no está en `develop` | se niega (`predecesora-sin-merge`) |
-| S3 | TASK-001 terminada y con su merge empujado a `develop` | arranca |
-| S4 | sin red, con la caché local diciendo que TASK-001 está terminada e integrada | se niega (`sin-red`) |
-| S5 | `start` de TASK-005 (zona `core/a/x/**`) con TASK-003 en curso en `core/a/**` | aviso: solapa con TASK-003 (companero) |
-| S6 | `start` de TASK-006 (zona `web/**`) con TASK-004 en curso sin zona | aviso: TASK-004 sin zona choca con todas |
-| S7 | `start` de TASK-007 (zona `core/b/**`); la rama de TASK-003 toca `core/b/dos.rs` fuera de su zona | aviso: cruce con el diff real |
+| S3 | un merge de otra tarea en `develop` copia el trailer `Tarea: TASK-001`; la rama de TASK-001 no está integrada | se niega (`predecesora-sin-merge`) |
+| S4 | la rama de TASK-001 integrada en `develop` con un merge como los de la 0.16 (sin trailer) | arranca |
+| S5 | sin red, con la caché local diciendo que TASK-001 está terminada e integrada | se niega (`sin-red`) |
+| S6 | `start` de TASK-005 (zona `core/a/x/**`) con TASK-003 en curso en `core/a/**` | aviso: solapa con TASK-003 (companero) |
+| S7 | `start` de TASK-006 (zona `web/**`) con TASK-004 en curso sin zona | aviso: TASK-004 sin zona choca con todas |
+| S8 | `start` de TASK-007 (zona `core/b/**`); la rama de TASK-003 toca `core/b/dos.rs` fuera de su zona | aviso: cruce con el diff real |
+| S9 | `review` de TASK-008 (zona `core/*.rs`), cuya rama toca `core/c.rs` y `core/a/uno.rs` | aviso: `core/a/uno.rs` fuera de su zona |
 <!-- tabla-start:fin -->
 
-La predecesora tiene que estar `terminada` en la rama de estado **y** tener su
-merge en `origin/develop` (un merge del primer padre con trailer
-`Tarea: TASK-NNN`), después de un `fetch`. Sin red, `start` se niega aunque la
-caché local diga que todo está integrado.
+La predecesora tiene que estar `terminada` en la rama de estado **y** estar
+integrada en `origin/develop` **por ascendencia**: la cabeza de su rama remota
+(`<tipo>/task-nnn-*`, que F1-3 conserva) es antecesora de `origin/develop`
+(`git merge-base --is-ancestor`), después de un `fetch`. No se mira el mensaje
+del merge: en los tres repos ningún merge del primer padre lleva un trailer
+`Tarea:` (0 de 50 en ToolsaWeb, 0 de 48 en OpenDockTrading y 0 de 18 en
+TaskCodeTwo, contado en la revisión 1), y un trailer copiado no prueba nada
+(S3). S4 usa un merge con el mensaje de la 0.16. Sin red, `start` se niega
+aunque la caché local diga que todo está integrado.
 
 ### L — Status line y escrituras concurrentes (D5a)
 
@@ -342,15 +358,18 @@ caché local diga que todo está integrado.
 | ID | Caso | Resultado |
 |---|---|---|
 | L1 | lectura de la status line con 10 y con 1000 tareas | 1 fichero y los mismos bytes en los dos casos |
-| L2 | `os.replace` con la status line leyendo el fichero (Windows) | sin reintento falla con PermissionError; con reintento entra |
+| L2 | `os.replace` con un lector de Python abierto (Windows; sin `FILE_SHARE_DELETE`) | sin reintento falla con PermissionError; con reintento entra |
 | L3 | `estado.json` a medio escribir | la status line enseña el último valor bueno con su antigüedad |
 | L4 | dos escritores añaden filas de `## Coste` a la vez | leer-añadir-reescribir pierde 1 de 2 filas; un fichero por fuente, ninguna de 400 |
-| L5 | status line de TASK-075 a las 12:00 UTC | `TASK-075 · implementacion · fases que faltan: rondas, cierre · faltan ≈31 min (p80 66) · n=17 carril+complejidad+tamano · sin actividad registrada hace 127 min` |
+| L5 | status line de TASK-075 a las 12:00 UTC | `TASK-075 · implementacion · fases que faltan: rondas, cierre · faltan ≥31 min (fase en curso por encima de sus parecidas) · n=17 carril+complejidad+tamano · sin actividad registrada hace 127 min` |
 <!-- tabla-statusline:fin -->
 
-En Windows, `os.replace` sobre un fichero que otro proceso tiene abierto
-falla con `PermissionError` (WinError 5). La escritura atómica necesita
-reintento, y el lector, una salida al último valor bueno. La línea de L5
+En Windows, `os.replace` sobre un fichero que un lector de Python tiene
+abierto falla con `PermissionError` (WinError 5): Python, como Node, abre sin
+`FILE_SHARE_DELETE`. Con un lector de Rust (`std::fs::File::open`, que abre con
+`FILE_SHARE_DELETE`) el rename entra; lo midió la revisión 1 con un fichero
+suelto. El reintento se queda como defensa ante otros lectores (Python, Node,
+un antivirus), y el lector tiene su salida al último valor bueno. La línea de L5
 compara la estimación del CLI para esta misma tarea (mediana 50 min y p80 96
 de 17 ADR parecidos) con los 300 min que puso el agente en el plan.
 
@@ -424,7 +443,11 @@ borrada) fallan cada una por su motivo.
    sigue la regla del real hasta el último evento, más el hueco abierto
    entero: alguien puede estar trabajando sin commit todavía, y la siguiente
    transición lo recalcula. Si ese hueco pasa de 45 min, la vista dice «sin
-   actividad registrada hace N min».
+   actividad registrada hace N min». **Si menos de 5 muestras pasaron de lo
+   transcurrido**, la fase en curso está desbordada: lo que queda de ella no
+   se sabe, y lo que se enseña es solo una cota inferior (la mediana de las
+   fases siguientes), como «faltan ≥31 min (fase en curso por encima de sus
+   parecidas)», nunca un número limpio con su p80 (L5).
 6. **Cuándo se recalcula**: en `approve`, en cada transición y en `finish`.
    En `finish` se registra el error (estimado frente a real por fase), que es
    la entrada de F8-7. Recalcularlo todo cuesta O(N) con N tareas y solo pasa
@@ -450,7 +473,9 @@ borrada) fallan cada una por su motivo.
      tarea, fase, fases que faltan, restante [mediana, p80], nivel, n, hora
      del cálculo y avisos. Antes de cada escritura se guarda el anterior como
      `estado.anterior.json`. Cada paso es temporal + rename **con reintento**,
-     porque en Windows el rename falla con el lector abierto (L2).
+     porque en Windows el rename falla si un lector lo tiene abierto sin
+     `FILE_SHARE_DELETE` (Python o Node, L2; el binario de Rust de F1-14 abre
+     con él y no lo necesita).
 8. **Vistas**, todas del CLI y sin tokens:
    - **status line** (F3-12): abre **un fichero** (`estado.json`), resta el
      reloj desde la hora del cálculo y pinta la tarea, la fase, las fases que
@@ -458,6 +483,11 @@ borrada) fallan cada una por su motivo.
      último valor bueno con su antigüedad (L3). Nunca recorre el histórico (L1);
    - **`tc2 status --seguir`**: la misma lectura, repetida al cambiar el
      fichero;
+   - **texto ajeno saneado**: lo que llega de `tarea.md`, de la rama de estado
+     o de un nombre de persona o de rama es texto no confiable (F1-15) y se
+     imprime sin caracteres de control ni secuencias de escape, y con longitud
+     acotada, en la status line, `status --seguir`, `resumen` y los avisos de
+     `start`. El caso de prueba es de F3-12;
    - **salida de cada transición**: una línea con lo mismo;
    - **`tc2 resumen`** (requisito 1; F3-5, F8-1): tabla por sprint con
      terminadas y pendientes, tiempo real y tokens (parcial y total) y lo que
@@ -482,14 +512,20 @@ borrada) fallan cada una por su motivo.
     el prefijo literal de un glob es prefijo, por componentes, del de otro. La
     zona es siempre un **aviso, nunca una garantía**: en `start` y en `review`
     se cruza con el diff real de cada rama en curso (F3-11), y lo que una rama
-    toca fuera de su zona también avisa (S7).
+    toca fuera de su zona también avisa (S8, S9). La pertenencia de un fichero
+    a su zona es el emparejamiento real del glob (`**` cruza carpetas, `*` no),
+    distinto de la comparación zona con zona: con el prefijo, `core/a/uno.rs`
+    estaría dentro de `core/*.rs` (S9).
 12. **Precondiciones de `start`** (D3a, requisito 2; código en F3-6). Primero,
-    `fetch` de `develop` y de la rama de estado. Cada predecesora tiene que
-    estar `terminada` en la rama de estado **y** tener su merge en
-    `origin/develop`. Si no, `Rechazada` (código 3 de F1-2) con el motivo
-    (`predecesora-sin-terminar`, `predecesora-sin-merge`). **Sin red se niega**
-    (`SinRed`, código 4) y explica por qué; nunca arranca a ciegas con la
-    caché. Después avisa del solape (punto 11), con nombres y personas.
+    `fetch` de `develop`, de la rama de estado y de las ramas. Cada predecesora
+    tiene que estar `terminada` en la rama de estado **y** integrada en
+    `origin/develop` por ascendencia: `git merge-base --is-ancestor` de su
+    `ultimo_commit_revisado` (2.0) o de la cabeza de su rama remota (historia
+    de la 0.16, cuyas ramas se conservan, F1-3) contra `origin/develop`. Nunca
+    por el mensaje del merge. Si no, `Rechazada` (código 3 de F1-2) con el
+    motivo (`predecesora-sin-terminar`, `predecesora-sin-merge`). **Sin red se
+    niega** (`SinRed`, código 4) y explica por qué; nunca arranca a ciegas con
+    la caché. Después avisa del solape (punto 11), con nombres y personas.
 13. **Aviso de cambios ajenos**: en cada transición, el CAS de F1-16 (la
     cabeza y el `status --porcelain` esperados). La status line solo enseña el
     aviso; la decisión no depende de ella.
@@ -547,10 +583,10 @@ borrada) fallan cada una por su motivo.
 | RR-19 resumen final, estimado frente a real | punto 14 (F8-7) | PoC E: error por fase |
 | Criterio de F1-17: probada contra ToolsaWeb y OpenDockTrading, con su error medio | puntos 3 y 4 | PoC E: `tabla-error` (error medio 69 y 38 min; mediana 0,45 y 0,43) |
 | Requisito (1) de Carlos: `resumen` | punto 8 | PoC D: `tabla-resumen` |
-| Requisito (2) de Carlos: paralelo sin pisarse | puntos 11 y 12 | PoC S1-S7 |
+| Requisito (2) de Carlos: paralelo sin pisarse | puntos 11 y 12 | PoC S1-S9 |
 | Requisito (3) de Carlos: `--personas N` | puntos 10 y 11 | PoC D: el prototipo reproducido; serie sin zonas (D4a) |
-| RR-12 `start` avisa del solape | punto 11 | PoC S5-S7 |
-| RR-12 estado más reciente | `fetch` antes de decidir; sin red se niega | PoC S4 |
+| RR-12 `start` avisa del solape | punto 11 | PoC S6-S9 |
+| RR-12 estado más reciente | `fetch` antes de decidir; sin red se niega | PoC S5 |
 | RR-01 lo hace el CLI | núcleo puro, sin hooks | toda la PoC |
 
 ## Consecuencias
@@ -607,7 +643,13 @@ borrada) fallan cada una por su motivo.
 - **Que los commits sean una señal de actividad suficiente**: un agente que
   trabaja más de 45 min sin commit ni coste pierde lo que pase del tope.
 - **El rename en Linux y macOS**: allí no falla con el lector abierto (POSIX);
-  no se ejecutó.
+  no se ejecutó. Que el binario de Rust de F1-14 abra con `FILE_SHARE_DELETE`
+  y el rename entre en Windows lo midió la revisión 1 con un fichero suelto,
+  no con el binario.
+- **La precondición por ascendencia en los pilotos reales**: se probó en el
+  remoto simulado (S2-S4); en los pilotos depende de que las ramas de las
+  tareas antiguas sigan en el remoto (F1-3 las conserva). Si una falta,
+  `start` se niega con `predecesora-sin-merge` (F3-6).
 - **La regla textual de troceo**: agrupa 8 grupos en ToolsaWeb y 4 en
   OpenDockTrading leyendo el objetivo; no se comprobó a mano que no falte
   ninguno.
diff --git a/docs/adr/README.md b/docs/adr/README.md
index 2aa7c17..9136507 100644
--- a/docs/adr/README.md
+++ b/docs/adr/README.md
@@ -24,7 +24,7 @@ directo a la tabla de la Fase 1 y a su tarea.
 | [F1-14](F1-14-distribucion-e-instalacion.md) | Distribución e instalación: CLI en Rust por la regla D1a (gana el 1.º y el 2.º, medidos, y el 4.º, un juicio sobre postject; sin el 4.º la regla no decide) frente a TypeScript con Node SEA (mismo comando, salida idéntica sobre las 404 tareas de ToolsaWeb y las 96 de OpenDockTrading; más rápido en caliente en los 3 repos y 32,1 veces más pequeño), TLS con el almacén del sistema, lanzador sin runtime con `cli.lock` y descarga comprobada por sha256, compatibilidad por protocolo con `tc2 version --json`, canales y firma por etapas (hash y attestation en la 2.0; Authenticode y Developer ID al entrar en winget y Homebrew) y contrato del manifiesto de recursos (`tc2 doctor --recursos`: obligatorio, necesario, conveniente y estorba); Linux y macOS no verificados | TASK-014 | aceptada |
 | [F1-15](F1-15-seleccion-de-skills-agentes-y-modelo.md) | Selección de skills, agentes y modelo por tarea y por rol: función pura `seleccionar` que interpreta tres tablas entre marcadores (reglas, modelo por carril y complejidad, dominios con lista cerrada que la config solo amplía), solo entre lo instalado (inventario de F3-9 y metadatos en la propia skill o en `recursos:`), empate por id sin modelo, suelo de 2 roles con `puertas-seguridad` (17 de 33 IMPORTANTES de la Fase 1 fueron de esa clase; detector con recall 7/8 y precisión 7/9 dentro de muestra), texto de la tarea no confiable que solo sube desde `approve`, revisora del dominio o aviso medido con la config de la base, anotación en `roles.<rol>` y columna «Clase» para medir; 10 tareas reales de ToolsaWeb y OpenDockTrading que Carlos comprobó a mano («Coincide las 10») | TASK-073 | aceptada |
 | [F1-16](F1-16-lotes-desatendidos.md) | Lotes desatendidos con revisión humana al final: `tc2 lote` determinista con núcleo puro (`planificar`, `siguiente_paso`), rama `lote/<nombre>`, operación `integrar-en-lote` (merge por API de la App del agente con el `sha` revisado como CAS) y ruleset `taskcodetwo-lote`, comprobación del primer padre con el árbol de cada merge (D3a), carril del diff recalculado, manifiesto con el núcleo de cada tarea (todo `tarea.md` y el plan menos lo que escriben el CLI y el ciclo, medido en los tres repos: 0 de 1442 escrituras durante una tarea lo cambian; lo excluido es texto no confiable), salida de un commit ajeno en `lote/*` solo por una persona de la lista (reversión exacta o `ajeno-aceptado`), guarda de procedencia antes de `review` y de integrar, escrituras del agente declaradas (la de la persona en el mismo fichero no se detecta), diario `lotes/<nombre>.yml` reconciliado desde Git para `--reanudar`, presupuesto por lote y por tarea, tope de rondas sin ronda extra automática, `[DECIDE CARLOS]` que no entra y nunca para el lote, informe de la mañana en el PR del lote y equivalente en GitLab; «ninguna IA mergea en `develop`» no garantizado en privados gratuitos, solo detectado después; probado con 17 tareas reales de ToolsaWeb sobre una copia y una Plataforma simulada (GitHub y GitLab reales no verificados) | TASK-074 | aceptada |
-| [F1-17](F1-17-progreso-y-estimacion.md) | Progreso en vivo y estimación de tiempo: lector único del histórico con el real recalculado desde `## Transiciones` (huecos de más de 45 min a 45, pausas fuera), estimación por fase (implementación, rondas, cierre; diseño sin calibrar) con mediana y p80 de tareas parecidas en cascada, que fuera de muestra no gana con claridad a las líneas base (ToolsaWeb 0,45 frente a 0,52 de la mediana ingenua; OpenDockTrading 0,43 frente a 0,41) y por eso se enseña como rango junto a la estimación del agente; `estado.json` derivado para la status line O(1) con último valor bueno y rename con reintento en Windows; `tc2 resumen`; `next --personas N` con camino crítico y zonas declaradas (sin zona choca con todas, siempre aviso); `start` que exige el merge de la predecesora en `develop` y se niega sin red | TASK-075 | aceptada |
+| [F1-17](F1-17-progreso-y-estimacion.md) | Progreso en vivo y estimación de tiempo: lector único del histórico con el real recalculado desde `## Transiciones` (huecos de más de 45 min a 45, pausas fuera), estimación por fase (implementación, rondas, cierre; diseño sin calibrar) con mediana y p80 de tareas parecidas en cascada, que fuera de muestra no gana con claridad a las líneas base (ToolsaWeb 0,45 frente a 0,52 de la mediana ingenua; OpenDockTrading 0,43 frente a 0,41) y por eso se enseña como rango junto a la estimación del agente; `estado.json` derivado para la status line O(1) con último valor bueno y rename con reintento en Windows; `tc2 resumen`; `next --personas N` con camino crítico y zonas declaradas (sin zona choca con todas, siempre aviso); `start` que exige la predecesora integrada en `develop` por ascendencia (`merge-base --is-ancestor`, no por el mensaje del merge) y se niega sin red | TASK-075 | aceptada |
 | [F1-18](F1-18-hito-de-decisiones.md) | Hito de decisiones (ADR) en `init` y en `actualizar`: estudio de adopción que lo dimensiona (bloque «uso medido» del informe de F1-12 y `proponer_hito` con un elemento por hueco, bloqueantes antes de migrar), proporcionalidad con mediana y p75 de la Fase 1 (45,5–50 min y 13,9–19,1 M tokens por ADR), registro `hito-decisiones.yml` en `taskcodetwo/tareas` cuya lista, descartes y «no necesario» firma una persona, con estado calculado que falla cerrado, y puerta determinista en `tc2 start` y en el check `estado` (código `rechazada`) | TASK-094 | aceptada |
 
 ## Plantilla
````

## Excluido del diff (9 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 02291790d1bb0ea610cb0a3fdeaaac8e36557400..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-075/planificacion/brainstorm/peticion-plan-1.md       |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-075/planificacion/brainstorm/salida-arquitectura-1.md |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-075/planificacion/brainstorm/salida-riesgos-1.md      |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-075/planificacion/plan-final.md                       |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-075/poc/poc-f1-17.log                                 |  62 +++++----
 tareas/{02-en-curso => 03-en-revision}/TASK-075/poc/poc-f1-17.py                                  | 132 ++++++++++++++----
 tareas/03-en-revision/TASK-075/revision/informe-revision-1.md                                     |  92 +++++++++++++
 tareas/03-en-revision/TASK-075/revision/peticion-revision-1.md                                    | 708 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-075/tarea.md                                          |  10 +-
 9 files changed, 948 insertions(+), 56 deletions(-)
````
