# Informe de revision — TASK-075 (ronda 1)

- Commit revisado: 02291790d1bb0ea610cb0a3fdeaaac8e36557400
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

Pase generico (el diff es un ADR, su fila del indice, una PoC en Python y su log; ningun revisor de dominio encaja). Profundidad: a fondo la medicion fuera de muestra (`disponibles`, `estimar`, `evaluar`, `seccion_e`), el real recalculado (`tramo`, `real`), las cifras de prosa y tablas del ADR contrastadas con el log, y la decision 12 (`start`) con `mergeada`/`puede_empezar`; por encima `repartir`, `restante`, la status line y el resto de S/L/F. Alcance de la PoC: reducido por decision de Carlos (2026-11-15); un hueco declarado en «No verificado» no se cuenta como hallazgo.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-17-progreso-y-estimacion.md:226 (Control de fuga) y tareas/03-en-revision/TASK-075/poc/poc-f1-17.py:1145 | implementacion |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-17-progreso-y-estimacion.md:335 y :486 (decision 12) y tareas/03-en-revision/TASK-075/poc/poc-f1-17.py:950 (`mergeada`) | plan |
| MEN-1 | MENOR | corregido | docs/adr/F1-17-progreso-y-estimacion.md:267 y tareas/03-en-revision/TASK-075/poc/poc-f1-17.py:1204 | implementacion |
| MEN-2 | MENOR | corregido | docs/adr/F1-17-progreso-y-estimacion.md:352 y :453 | implementacion |
| MEN-3 | MENOR | corregido | docs/adr/F1-17-progreso-y-estimacion.md:480 y tareas/03-en-revision/TASK-075/poc/poc-f1-17.py:993 | plan |
| MEN-4 | MENOR | corregido | docs/adr/F1-17-progreso-y-estimacion.md:421 y :348 | plan |
| MEN-5 | MENOR | corregido | docs/adr/F1-17-progreso-y-estimacion.md:279 | implementacion |
| MEN-6 | MENOR | corregido | docs/adr/F1-17-progreso-y-estimacion.md:457 y :492 | plan |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

No hay CRITICO. Lo que SI cuadra, comprobado: (a) la PoC completa reproducida (`python -I -B poc-f1-17.py <TaskCodeTwo> <ToolsaWeb> <OpenDockTrading>`, 41 s en mi maquina, exit 0, `RESULTADO: todo como se espera`) da un log **identico** al commiteado una vez quitado el CR; no deja cambios en el repo y borra su carpeta de trabajo; ToolsaWeb y OpenDockTrading intactos; (b) a mano, contra `git show` de OpenDockTrading: TASK-096 tiene `start` 01:51:20Z, primera `review` 08:15:45Z, segunda 08:36:41Z y `finish` 08:44:54Z, los commits de la tarea caen en 08:13-08:15Z; con tope de 45 min salen implementacion 47, rondas 21, cierre 8 = 76, y con el hueco a 0 la implementacion queda en 2: las cifras del ADR (414 → 76) son ciertas; (c) la mediana y el p80 de la prosa, las 10 tablas (n, medianas, ±50 %, sesgos, cobertura del p80 98/149 y 33/41, remuestreos 94,2 % y 43,9 %, tokens, lote, resumen, reparto) salen del log; el prototipo (16,6 h; 77,1/38,6/25,8/19,3 h) se reproduce con `comprueba`; (d) la medicion fuera de muestra **no tiene fuga**: `disponibles` exige `finish <= approve` y excluye la propia, los grupos de troceo entran por su `approve` minimo y su `finish` maximo (conservador), y la «Estimada» se compara con el mismo real recalculado que el modelo; (e) el carril sacado del diff final (conocido solo tras terminar) casi no pesa: con todas las muestras forzadas a `funcionalidad` la mediana de ToolsaWeb pasa de 0,45 a 0,47 (sigue por debajo de la ingenua, 0,52) y la de OpenDockTrading de 0,43 a 0,47, asi que la conclusion del ADR sobrevive y el hueco ya esta en «No verificado»; (f) la sobreestimacion de la 0.16 en OpenDockTrading no es un artefacto del real recortado: con el «Real» de la 0.16 la razon estimado/real mediana es 2,57 (frente a 2,68); (g) D1a..D8a estan recogidas con su letra y D8a, que el modelo incumple, se declara en la Divergencia 9 y en la Decision 4 en lugar de maquillarse.

## Reproduccion

Carpeta de trabajo: `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-075-r1` (log y salida; la PoC usa ademas su propia `...\task-075`, que borra ella), borrada al terminar. ToolsaWeb y OpenDockTrading solo lectura (`git log`, `git show`, `git ls-tree`; nada ejecutado dentro). Mis guiones estan en el scratchpad de la sesion, no en el repo, e importan la PoC por ruta.

### IMP-1 — El «control de fuga» no detecta una fuga y la frase del ADR solo es cierta para la mediana

- Donde: ADR:226-228 («estimando dentro de muestra el error baja (0,40 en ToolsaWeb, 0,38 en OpenDockTrading), como tiene que pasar si la medida fuera de muestra no tiene fuga»); `poc-f1-17.py:1143-1145` (`comprueba("E", md["med"] < comun["modelo"]["med"], ...)`); criterio de aceptacion 1 y regla del plan «publicar una calibracion dentro de muestra es punto sin retorno».
- Que pasa: (1) la prueba solo mira la mediana y solo en una direccion: «dentro < fuera». Una fuga (usar muestras posteriores) tambien hace bajar la medida fuera de muestra, y dentro de muestra seguira siendo menor o igual, asi que la comprobacion pasa con y sin fuga. (2) La frase del ADR es parcial: en el log, dentro de muestra en ToolsaWeb (n=154) el error medio relativo **sube** de 61,6 % a 78,8 %, el p80 de 0,71 a 1,16 y el p90 de 1,07 a 2,04; en las mismas 149 tareas: 0,41 / 77,7 % / 1,02 / 1,88. En OpenDockTrading el p80 queda igual (0,77) y solo bajan la mediana y el p90. Solo la mediana y el ±50 % cumplen «baja»; ademas la n difiere (154 y 41 frente a 149 y 40), y el ADR lo presenta como prueba de que no hay fuga.
- Reproduccion (ejecutada): sustituyo `disponibles` por una version con fuga (todo el pool menos la propia tarea, futuro incluido) y vuelvo a evaluar. ToolsaWeb: fuera 0,454, dentro 0,405, **con fuga fuera de muestra 0,409; la comprobacion del PoC (`dentro < fuera`) sigue pasando (True)**. OpenDockTrading: fuera 0,432, dentro 0,383, con fuga 0,383 (empate, pasa por redondeo). Es decir, la unica proteccion nueva contra la fuga no se pone roja al introducirla. (La fuga real no existe en el codigo, lo comprobe leyendo `disponibles` y con los resultados anteriores; lo que falla es el control y la frase que lo vende.) La tabla del ADR si cambiaria con la fuga y G lo cazaria, pero por cifras distintas, no por este control.
- Impacto: el ADR dice «Nunca se publica una calibracion dentro de muestra» y apoya la ausencia de fuga en un control que no discrimina; la conclusion que cuelga (el modelo gana a la ingenua en ToolsaWeb) es correcta segun mi lectura del codigo, pero la evidencia citada no la sostiene.
- Correccion: sustituir el control por uno que pueda fallar, p. ej. un control negativo con las muestras anteriores a cada tarea barajadas (permutar `finish`/`approve` entre tareas no debe dar mejor que la ingenua) o una mutacion que meta una muestra posterior y exija que el error baje por debajo del de dentro de muestra; y reescribir la frase del ADR con las cuatro metricas (mediana, p80, p90, error medio) y la misma n, diciendo que solo la mediana baja.

### IMP-2 — Las precondiciones de `start` exigen un trailer `Tarea:` que no tiene ni un merge de los repos medidos ni de TaskCodeTwo

- Donde: ADR:335-336 («un merge del primer padre con trailer `Tarea: TASK-NNN`»), ADR:486-492 (decision 12, `predecesora-sin-merge`), `poc-f1-17.py:950-953` (`mergeada`: `--first-parent --merges` y `^Tarea: T$` en el cuerpo) y `poc-f1-17.py:930-931` (el mundo de S crea los merges con ese trailer: `merge(feature): ... \n\nTarea: {t}`).
- Que pasa: en ToolsaWeb@43ab5907e, 0 de 50 merges del primer padre llevan una linea `Tarea: TASK-NNN`; en OpenDockTrading@2b0064f2c, 0 de 48; en TaskCodeTwo `develop`, 0 de 18. El mensaje real es `merge(feature): feature/task-206-... -> develop` y nada mas. S3 («arranca») y S2 solo funcionan porque la PoC fabrica los merges con el trailer; la PoC no prueba la regla sobre ningun merge real. Con la regla escrita, todo repo con historia anterior a la 2.0 (los pilotos y este mismo repo, F1-12 y F1-18 los tratan como destino de la migracion) niega `start` por `predecesora-sin-merge` para cualquier predecesora terminada antes del cambio.
- Ademas la prueba es debil por otra via: es un patron en el mensaje (lo escribe cualquiera con push) y no mira que la rama de la predecesora este realmente integrada; un merge revertido o un mensaje copiado pasan.
- Reproduccion (ejecutada): los tres `git log --first-parent --merges <fijo> --format=%B | grep -c "^Tarea: TASK"` dan 0 (50, 48 y 18 merges respectivamente). No ejecute `start`: no hay CLI; la PoC lo simula.
- Impacto: comportamiento incorrecto en el caso real que el ADR persigue (arrancar sobre una base integrada en repos que ya existen); el ADR no declara la migracion ni lo pone en «No verificado».
- Correccion: definir la integracion por ascendencia, no por mensaje: `git merge-base --is-ancestor <ultimo_commit_revisado o cabeza de la rama de la predecesora> origin/develop` (determinista, sirve en 0.16 y 2.0), con el trailer como atajo; y añadir a la PoC un caso con un merge «de la 0.16» (sin trailer) y otro con un merge de otra tarea cuyo mensaje copia el trailer, o declarar en «Divergencias» que la regla solo vale para merges de la 2.0 y como convive con la historia anterior.

### MEN-1 — Las variantes de umbral comparan el error contra reales distintos

ADR:267 («Un umbral de 30 min da algo menos de error en los dos repos») y `poc-f1-17.py:1204-1209`: cada umbral recalcula tambien la **variable objetivo** (`muestras_agrupadas(..., u)` cambia el real), asi que un umbral mas bajo acorta y homogeneiza el real y baja el error relativo por construccion; «sin corte» (0,51/0,50) frente a la cascada entera (0,45/0,43) tiene el mismo problema. El ADR no cambia D6a (bien) y lo deja «como dato para F8-7», pero ese dato no es comparable. Correccion: decir que la variante cambia tambien el objetivo, o medir todas las variantes contra el real de 45 min.

### MEN-2 — «En Windows el rename falla con el lector abierto» solo se probo con un lector de Python

ADR:352-355 y :453 (decision 7: «porque en Windows el rename falla con el lector abierto (L2)»); L2 usa `os.replace` con un `open()` de Python. Comprobado: con un lector de Python el rename falla (`PermissionError`), con `fs.openSync` de Node tambien (`EPERM`), pero con un lector de Rust (`std::fs::File::open`, el binario de F1-14) `fs::rename` **entra** con el fichero abierto (std abre con `FILE_SHARE_DELETE`). El reintento sigue siendo una buena defensa (otros lectores, antivirus), pero la causa afirmada depende del lector. Correccion: «con un lector sin `FILE_SHARE_DELETE` (Python, Node)» y añadir a «No verificado» que el lector de Rust no lo necesita (lo medi en un fichero suelto, no en el binario de F1-14).

### MEN-3 — El cruce con el diff usa la funcion de solape de zonas como pertenencia de un fichero a su zona

`poc-f1-17.py:993` (`not solapan([f], x["zonas"])`) y ADR:480-485 (punto 11, S7). `solapan` compara solo el prefijo literal del glob, correcto como aviso conservador entre zonas, pero para «fichero fuera de su zona» da el sentido contrario: ejecutado, `solapan(["src/otro/x.rs"], ["src/a*/**"])`, `solapan(["src/deep/x.rs"], ["src/*.rs"])` y `solapan(["web/x.rs"], ["**/*.rs"])` dan `True`, o sea «dentro de la zona» (el ultimo es dudoso por el glob, los dos primeros no). Con zonas con comodines el aviso de S7 no salta. Es una divergencia entre lo que promete el punto 11 («lo que una rama toca fuera de su zona tambien avisa») y su prueba, que solo usa `core/a/**` (prefijo sin comodin). Correccion: para la pertenencia usar el emparejamiento real del glob (y dejar `solapan` solo para zona contra zona), con un caso S7b con comodin; o decir en el ADR que F3-11 lo define.

### MEN-4 — `restante` no dice que pasa cuando menos de 5 muestras han superado el tiempo de la fase en curso, y L5 lo muestra optimista

ADR:421-427 (punto 5) y :348 (L5: «faltan ≈31 min (p80 66)» tras 127 min de implementacion). Si sobreviven menos de 5 muestras, el codigo (`poc-f1-17.py:744-760`) usa todas y resta `min(transcurrido, real de la fase)`: da por **terminada ya** la fase en curso y enseña solo rondas+cierre. Para una tarea que ya rebaso a casi todas sus parecidas, la linea promete 31 min cuando el tiempo que falta de la propia implementacion es desconocido (el aviso de «sin actividad hace 127 min» ayuda, pero no distingue «desbordada» de «parada»). Correccion: una frase en el punto 5 con la regla (y p. ej. enseñar «≥31 min, fase en curso ya por encima del p80 de las parecidas» en vez de un numero limpio).

### MEN-5 — «Las 18 terminadas son ADR» no es cierto

ADR:279. Son 17 ADR (F1-1 a F1-18 sin F1-17) y F1-0 «Crear TaskCodeTwo», que es la que sale `funcionalidad` por el diff (el log: errata 17, funcionalidad 1). Correccion: «17 ADR y la creacion del repo».

### MEN-6 — El ADR depende de F1-15 («texto de la tarea no confiable») pero no dice que lo que se pinta desde campos de otros se sanea

ADR:457 y :492 (la status line y el aviso enseñan «nombres y personas» y avisos que salen de `asignado_a` y de ramas de otras personas o del agente) y ADR:7 (dependencia de F1-15). Ningun punto de F1-17 exige quitar secuencias de escape/caracteres de control antes de pintarlos en el terminal de quien trabaja (status line, `status --seguir`, `resumen`, avisos de `start`). No reproducido (no hay CLI); es un riesgo de trabajo en grupo que el plan pedia tratar («aviso de cambios ajenos», seguridad imprescindible). Correccion: una linea en la decision 8: lo que viene de `tarea.md`, la rama de estado o un nombre se imprime sin caracteres de control (y con longitud acotada), y se deja a F3-12 el caso de prueba.

## Mutaciones propias (ninguna esta entre las 4 de la PoC)

| Mutacion | La PoC la detecta | Nota |
|---|---|---|
| `disponibles` con fuga (usa muestras posteriores) | **No** (el control `dentro < fuera` sigue en verde en ToolsaWeb; G falla solo por las cifras de las tablas) | IMP-1 |
| Carril forzado a `funcionalidad` en todas las muestras | n/a (medida, no mutacion de control) | ToolsaWeb 0,45 → 0,47; OpenDockTrading 0,43 → 0,47; no cambia la conclusion |
| Merge de `develop` sin trailer `Tarea:` (tal como son los reales) | **No** (la PoC solo crea merges con trailer) | IMP-2 |

## Criterios de aceptacion y plan-final

- «Estimacion probada contra las tareas terminadas de ToolsaWeb y OpenDockTrading, con su error medio»: cumplido y reproducido (error medio 69 y 38 min; 61,6 % y 66,5 %; mediana 0,45 y 0,43), con rolling origin sin fuga, cruzado, lineas base y la regla D8a aplicada con honestidad. Con la salvedad de IMP-1: la prueba de que no hay fuga que cita el ADR no discrimina.
- «Cumple lo que pide F1-17 y los requisitos que cita»: estimacion por fase, cuando se recalcula, donde se muestra y sin tokens estan; D1a..D8a recogidas con su letra; las divergencias con F1-9, F3-3, F1-16, F1-14, F1-18 y F1-10 estan declaradas. Falta IMP-2 (la precondicion de `start` de D3a no funciona sobre los merges reales de los repos que el ADR mide).
- La fila del indice (`docs/adr/README.md`, entre F1-16 y F1-18) existe, resuelve y sus cifras (0,45/0,52; 0,43/0,41) cuadran con el ADR.

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-075/poc/poc-f1-17.py` y `poc-f1-17.log`: es mi parte del diff (excluida del diff embebido por `excluir_de_revision`); sin ellos no se puede reproducir.
- `tareas/04-terminadas/TASK-074/revision/informe-revision-1.md`: formato del informe, lo pidio el encargo.
- ToolsaWeb (`...\Personal\ToolsaWeb`) y OpenDockTrading (`...\Personal\OpenDockTrading`), solo lectura con `git log`/`git show`/`git ls-tree`: `tarea.md` de TASK-096 y los mensajes de los merges del primer padre (IMP-2); TaskCodeTwo `git log` de `develop` y los titulos de las 18 terminadas en `d9a5b10fd` (IMP-2, MEN-5). Los tres fuera del contexto listado pero necesarios para reproducir las cifras que cita el ADR.
- `.cargo` de la maquina (`rustc`) solo para compilar un fichero suelto de 5 lineas en la carpeta temporal (MEN-2); sin descargas.
