# Peticion de revision — TASK-012 (ronda 3)

- Tarea: TASK-012 — F1-11 ADR extensiones declaradas
- Rama revisada: feature/task-012-f1-11-adr-extensiones-declaradas
- Rama base: develop
- Commit revisado (HEAD): 72901f6338a1ace9a6f84d7bdbc379517af85c28
- Fecha: 2026-11-14
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-012 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 054468e9c3f2315bb351d2ecb0e970649becb914 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (5 hallazgo(s)))

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
informe de esta ronda (informe-revision-3.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-012/tarea.md
- tareas/03-en-revision/TASK-012/planificacion/
- docs/adr/F1-11-extensiones-declaradas.md

## Hallazgos de la ronda 2 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMPORTANTE-1 | IMPORTANTE | corregido | docs/adr/F1-11-extensiones-declaradas.md:590 | informe-revision-2.md |
| IMPORTANTE-2 | IMPORTANTE | corregido | docs/adr/F1-11-extensiones-declaradas.md:531 | informe-revision-2.md |
| IMPORTANTE-3 | IMPORTANTE | corregido | docs/adr/F1-11-extensiones-declaradas.md:600 | informe-revision-2.md |
| IMPORTANTE-4 | IMPORTANTE | corregido | docs/adr/F1-11-extensiones-declaradas.md:629 | informe-revision-2.md |
| IMPORTANTE-5 | IMPORTANTE | corregido | docs/adr/F1-11-extensiones-declaradas.md:591 | informe-revision-2.md |

## Commits a revisar (git log 054468e9c3f2315bb351d2ecb0e970649becb914..HEAD)

````
72901f6 chore(TASK-012): autorrevision y ronda extra 3
8f63e07 docs(adr): F1-11 corrige IMP-5 de la revision 2 (check estado lee los comprobadores de la base)
5a36de1 chore(TASK-012): peticion de revision ronda 2
````

## Diff desde la ronda anterior (git diff 054468e9c3f2315bb351d2ecb0e970649becb914..HEAD)

````diff
diff --git a/docs/adr/F1-11-extensiones-declaradas.md b/docs/adr/F1-11-extensiones-declaradas.md
index 2dcfb51..70467eb 100644
--- a/docs/adr/F1-11-extensiones-declaradas.md
+++ b/docs/adr/F1-11-extensiones-declaradas.md
@@ -75,7 +75,7 @@ validador) o **fuera**.
 | 6 | Tiempo máximo por perfil y presupuesto del freno, que no bloquea | `TW .taskcode/config.yml:10` «timeout_verificacion: 4800»; `TW .taskcode/config.yml:12-13` «no bloquea el turno»; `TW .taskcode/config.yml:20` «timeout_verificacion_selectiva: 1800»; `TW docs/verificacion/flujo-de-verificacion.md:296` «deja parar el turno»; `ODT .taskcode/config.yml:10` «timeout_verificacion: 1200» | clave | clave | propio | comprobado | `verificacion.perfiles.<p>.timeout`: en `rapida`, presupuesto que no bloquea; en las demás, rojo |
 | 7 | Caché propia por paso y «completa del mismo árbol» | `TW scripts/cache-verificacion.py:1` «Caché por paso de verificar.bat»; `TW scripts/verificar.bat:91-93` «completa-leer»; `TW docs/verificacion/flujo-de-verificacion.md:164-165` «mismo árbol exacto»; `TW docs/verificacion/flujo-de-verificacion.md:4` «Repetirla en cada turno» | a mano | clave | parche | dentro del comando | `verificacion.huella_arbol` y `verificacion.huella_local` (F1-7 dec. 2): caché por árbol; lo más fino (por paso) no lo da la 2.0 y se pierde al retirarla |
 | 8 | Canales de texto: el x/N de `TASKCTL_PROGRESO` y la línea «# tests N» | `TW scripts/verificar.bat:51-52` «TASKCTL_PROGRESO»; `TW .taskcode/config.yml:7-8` «# tests N»; `PL src/fs/progreso.ts:33` «TASKCTL_PROGRESO» | plugin | nativo | parche | proyectado | informe JSON de la verificación (F5-4), con pasos y recuento; sin canal de texto |
-| 9 | `RAMA_BASE` como entrada del comando | `TW scripts/clasificar-verificacion.py:14-15` «RAMA_BASE (la pasa TaskCode 0.13)»; `TW scripts/clasificar-verificacion.py:54-59` «os.environ.get("RAMA_BASE")» | plugin | nativo | propio | comprobado | `RAMA_BASE` solo en `rapida` e `impacto` (F1-7 dec. 1); la completa de ToolsaWeb no depende de ella (PoC E) |
+| 9 | `RAMA_BASE` como entrada del comando | `TW scripts/clasificar-verificacion.py:14-15` «RAMA_BASE (la pasa TaskCode 0.13)»; `TW scripts/clasificar-verificacion.py:54-59` «os.environ.get("RAMA_BASE")» | plugin | nativo | propio | comprobado | `RAMA_BASE` solo en `rapida` e `impacto` (F1-7 dec. 1); la completa de ToolsaWeb no depende de ella (PoC E). Lo propio es la variable como entrada del contrato; `clasificar-verificacion.py`, que hoy la lee, es parche y se retira (desglose) |
 | 10 | Pasos que dependen de la red | `ODT scripts/verificar.bat:68-69` «cargo audit --no-yanked --deny unsound»; `ODT scripts/verificar.bat:70` «npm --prefix web audit» | a mano | clave | propio | comprobado | `verificacion.programada.comando` con `verificacion.programada.cron`: job programado, fuera del check cacheado (F1-7 dec. 2) |
 | 11 | Cerrojo propio con códigos 3 y 4 | `TW scripts/cerrojo-verificacion.py:6-7` «3 si hay otra»; `TW scripts/cerrojo-verificacion.py:19` «el freno de fin de turno de TaskCode»; `TW docs/verificacion/flujo-de-verificacion.md:182` «código 3»; `ODT scripts/verificar.bat:3-4` «lo lanzan el freno de fin de turno»; `ODT scripts/verificar.bat:16-17` «impide dos» | a mano | clave | parche | dentro del comando | hoy, dentro de `verificacion.comando`; las puertas de la 2.0 corren en CI, un job por runner con el directorio limpio (F1-7, F1-8 dec. 6); cualquier código distinto de 0 es rojo (D6a) |
 | 12 | Ruta de Windows en YAML (`"\v"`) | `ODT .taskcode/config.yml:5-6` «seria un escape»; `TW .taskcode/config.yml:9` «"scripts\verificar.bat"» | a mano | clave | parche | convención | `verificacion.comando` y `derivados[].comando` entre comillas simples o con `/`; el validador rechaza un carácter de control (PoC B) |
@@ -159,7 +159,7 @@ propia (no verificado).
 |---|---|---|
 | **D1a** | Sin clave de estados propios: `descartada` y `pausa` nativos (F3-7) | **Elegida**: cubren los 11 de 11 casos medidos de `05-archivadas` (PoC A) |
 | D1b | Clave `estados` con estados del tablero configurables (lo que sugiere RF-71) | Descartada: una máquina de estados configurable para 11 casos que ya cubre un estado nativo; el validador la rechaza como clave desconocida (PoC B) |
-| **D2a** | Los derivados solo escriben bajo `tareas/` (rama de estado); lo generado que hoy vive en `docs/` (tablas y Mermaid de `hitos.md` de ToolsaWeb) pasa a `tareas/` | **Elegida**: el árbol de `develop` no cambia en 10 transiciones (PoC C, RR-11) y el `hitos.md` de ToolsaWeb pasa de rojo permanente a 0 errores (PoC C2) |
+| **D2a** | Los derivados solo escriben bajo `tareas/` (rama de estado); lo generado que hoy vive en `docs/` (tablas y Mermaid de `hitos.md` de ToolsaWeb) pasa a `tareas/` | **Elegida**: el árbol de `develop` no cambia en 11 transiciones (PoC C, RR-11) y el `hitos.md` de ToolsaWeb pasa de rojo permanente a 0 errores (PoC C2) |
 | D2b | Los derivados escriben también en ramas de código (`docs/planificacion/hitos.md`) | Descartada: cada transición cambiaría el árbol verificado (F1-1, RR-11) y la escritura saltaría la protección de `develop` |
 | **D3a** | Comprobaciones que cruzan código y estado como derivados `comprueba` en cada transición (avisan) y como check **obligatorio** `estado` en el PR, que los ejecuta, no se cachea y bloquea | **Elegida**: un resultado que depende del estado no puede vivir en un perfil cacheado por árbol (F1-7 dec. 2); el bloqueo pasa de la verificación al check `estado` (PoC C, T6) |
 | D3b | Dejarlas dentro del comando de verificación | Descartada: el check `verificacion` se cachea por árbol y el estado no está en el árbol: un acierto escondería un fallo nuevo |
@@ -324,7 +324,12 @@ haría `taskctl <fase>`: escribe la tarea, ejecuta los derivados de esa fase
 compara **por contenido** lo que cambió (en la rama de estado, todo fichero
 fuera de `escribe`, también el `tarea.md` de la transición; en el código,
 también lo ignorado), descarta lo que no le toca y commitea todo en un commit
-de la rama de estado. `check_estado` hace lo que haría el check del PR.
+de la rama de estado; antes de ejecutar, rechaza un `escribe` que pase por
+un enlace simbólico o una unión de Windows o cuyo destino real salga de
+`tareas/`. `check_estado` hace lo que haría el check del PR: toma la
+configuración y los comprobadores de una copia de la **rama base**
+(`git archive develop`) y los ejecuta contra el árbol del PR y la rama de
+estado.
 
 ```text
 merge-base develop taskcode/tareas: NINGUNA (huerfana)
@@ -341,6 +346,10 @@ merge-base develop taskcode/tareas: NINGUNA (huerfana)
       aviso: derivados[1] fallo (codigo 1: citadas en el codigo y sin tarea: TASK-404); la transicion sigue; arreglalo y ejecuta `taskctl derivados`
   check estado en el PR con ese codigo: ROJO (bloquea): derivados[1] codigo 1: citadas en el codigo y sin tarea: TASK-404
   check estado sin la cita rota: verde
+-- PR que cita TASK-404 y ademas cambia el comprobador y la config de derivados para ponerse en verde
+  ficheros del PR: ['.taskcode/config.yml', 'scripts/comprueba.py', 'src/nuevo.txt'] ; toca .taskcode/ -> riesgo alto por ruta (F1-10): True
+  check estado con config y comprobadores de la base: ROJO (bloquea): derivados[1] codigo 1: citadas en el codigo y sin tarea: TASK-404
+  lo vulnerable, con los del PR (revision 2, IMP-5): verde
 -- hostiles en ejecucion (lo escrito se compara por contenido)
   T7 pausa TASK-003, derivado que escribe en codigo    commit=['TASK-003/tarea.md'] ; grafo regenerado=False
       aviso: derivados[0] escribio fuera de `escribe` (['codigo:src/intruso.txt']): descartado; la transicion sigue
@@ -351,10 +360,13 @@ merge-base develop taskcode/tareas: NINGUNA (huerfana)
       aviso: derivados[0] escribio fuera de `escribe` (['codigo:.estado/x.txt']): descartado; la transicion sigue
   T10 review TASK-003, escribe una ruta ignorada del estado commit=['TASK-003/tarea.md'] ; grafo regenerado=False
       aviso: derivados[0] escribe una ruta ignorada en la rama de estado (['tareas/borrador.tmp']): no se commitearia; no se ejecuta
+  T11 plan TASK-003, escribe a traves de una union     commit=['TASK-003/tarea.md'] ; grafo regenerado=False
+      aviso: derivados[0] escribe a traves de un enlace o fuera de tareas/ (['tareas/enlace/x.md']): no se ejecuta
+  enlaces simbolicos en este sistema: no verificado (sin privilegios: WinError 1314); la comprobacion es la misma (islink y realpath)
 arbol de develop antes=8f4565fa2d86 despues=8f4565fa2d86 identico=True ; commits en develop 1->1 ; status limpio, tambien lo ignorado=True
-commits en taskcode/tareas: 11 (alta + 10 transiciones, uno por transicion)
+commits en taskcode/tareas: 12 (alta + 11 transiciones, uno por transicion)
 grafo-hitos.md final: TASK-001 fuera (terminada)=True ; TASK-003 en H2=True
-RESULTADO C transiciones=10 ; como se espera=10/10 ; develop identico=True
+RESULTADO C transiciones=11 ; como se espera=11/11 ; develop identico=True
 ```
 
 Lectura: lo generado va en el mismo commit que la transición; un derivado que
@@ -362,8 +374,14 @@ falla, que no está confiado o que escribe fuera de lo declarado no para la
 transición, deja su aviso con el comando que lo arregla y no ensucia nada,
 **también si reescribe el `tarea.md` de la propia transición** (T8: el estado
 se queda en `en-diseno`) **o escribe en una ruta ignorada** (T9). Un
-`comprueba` en rojo **avisa en la transición y bloquea en el PR** (T6). El
-árbol de `develop` es idéntico tras 10 transiciones (RR-11).
+`comprueba` en rojo **avisa en la transición y bloquea en el PR** (T6), y
+**sigue en rojo aunque el PR cambie el comprobador y la configuración** para
+ponerse en verde, porque el check los lee de la base (revisión 2, IMP-5; con
+los del PR, el mismo PR saldría verde); ese PR, además, toca `.taskcode/` y
+sale riesgo alto por ruta (F1-10). Un `escribe` a través de una unión de
+Windows se rechaza sin ejecutar nada (T11); los enlaces simbólicos usan la
+misma comprobación, pero este Windows no deja crearlos sin privilegios: no
+verificado. El árbol de `develop` es idéntico tras 11 transiciones (RR-11).
 
 ### C2 — `hitos.md` de ToolsaWeb, hoy y con la disposición nueva
 
@@ -457,8 +475,9 @@ como el clasificador escribe la completa con plan por grupos desde TASK-344.)
 2. **Citas reajustadas** a la línea exacta del commit fijo (p. ej. el aviso de
    comillas de OpenDockTrading, en 5-6). El inventario lleva las que la PoC
    comprueba.
-3. **PoC C con 10 transiciones**, no 5: las 5 del plan, un `comprueba` en rojo
-   con el check `estado`, y cuatro hostiles en ejecución.
+3. **PoC C con 11 transiciones**, no 5: las 5 del plan, un `comprueba` en rojo
+   con el check `estado` (también con un PR que cambia el comprobador), y
+   cinco hostiles en ejecución.
 4. **PoC C2 y D con los dos pilotos y los consumidores reales**: el plan solo
    pedía `lector-tareas.py`.
 5. **`hitos.md` de ToolsaWeb** (revisión 1, IMP-1): sus tablas y su Mermaid
@@ -470,6 +489,9 @@ como el clasificador escribe la completa con plan por grupos desde TASK-344.)
    `taskctl roadmap --escribir` en OpenDockTrading). El plan no lo nombraba.
 8. **Clasificación parche/propio y desglose de líneas**: decisión de la
    persona posterior al plan (2026-11-14).
+9. **El check `estado` lee los comprobadores de la base** (revisión 2,
+   IMP-5): la primera corrección los tomaba del árbol del PR, y el PR podía
+   ponerse en verde cambiando el script.
 
 ### No verificado
 
@@ -483,6 +505,13 @@ como el clasificador escribe la completa con plan por grupos desde TASK-344.)
   transición a la vez, un derivado que no termina): sin rol de riesgos; el
   reintento por rebase es de F1-2, y lo generado se **regenera** tras el
   rebase, como el `ROADMAP.md` (F1-1).
+- **Enlaces simbólicos en `escribe`**: la comprobación (`islink` y
+  `realpath`) es la misma que para las uniones, que sí se prueban (T11), pero
+  este Windows no deja crear enlaces simbólicos sin privilegios (WinError
+  1314): no verificado en Windows sin privilegios ni en Linux o macOS.
+- **El check `estado` real**: la PoC lo hace con una copia de `git archive`
+  de la base en local; en GitHub, el checkout de la base y del PR en el mismo
+  job es de F5-1.
 - **El coste de comparar por contenido** en un árbol de código con muchos
   ficheros ignorados (`target/`, `node_modules/`): la PoC lo hace en un repo
   pequeño; F2 decide cómo acotarlo.
@@ -528,7 +557,7 @@ workflow de TaskCode en cada PR (F5-1).
    | `derivados` | lista | comandos que `taskctl` ejecuta en las transiciones y que el check `estado` repite en el PR | lista de mapas |
    | `derivados[].comando` | orden | qué ejecuta | como `verificacion.comando` |
    | `derivados[].tipo` | `genera` o `comprueba` | si escribe ficheros o solo comprueba | uno de los dos |
-   | `derivados[].escribe` | rutas | lo único que un `genera` puede escribir, siempre bajo `tareas/` | obligatoria en `genera`, prohibida en `comprueba`; se **normaliza** antes de comparar (segmentos `.` y vacíos, `..` resuelto, sin `\`, `:`, comodines, caracteres de control ni nombres reservados de Windows); bajo `tareas/` y nunca `tareas/` entera ni `.git`; las reservadas (`tareas/ROADMAP.md`, `tareas/TASK-NNN/`) se comparan **sin distinguir mayúsculas**; en ejecución, una ruta ignorada en la rama de estado no se acepta |
+   | `derivados[].escribe` | rutas | lo único que un `genera` puede escribir, siempre bajo `tareas/` | obligatoria en `genera`, prohibida en `comprueba`; se **normaliza** antes de comparar (segmentos `.` y vacíos, `..` resuelto, sin `\`, `:`, comodines, caracteres de control ni nombres reservados de Windows); bajo `tareas/` y nunca `tareas/` entera ni `.git`; las reservadas (`tareas/ROADMAP.md`, `tareas/TASK-NNN/`) se comparan **sin distinguir mayúsculas**; en ejecución, no se acepta una ruta ignorada en la rama de estado, ni una que pase por un enlace simbólico o una unión, ni una cuyo destino real salga de `tareas/` |
    | `derivados[].fases` | lista | en qué transiciones corre (por defecto, en todas) | subconjunto no vacío de las fases de `taskctl` |
    | `derivados[].timeout` | segundos | tiempo máximo (por defecto 60, como `timeout_sincronizacion`) | entero de 1 a 600 |
    | `confianza_incluye` | globs | ficheros que la huella de confianza suma al cierre de cada orden (F1-8 dec. 5) | sin negación, rutas absolutas ni `..`, y nunca todo el árbol (`**`, `*`) |
@@ -587,14 +616,19 @@ workflow de TaskCode en cada PR (F5-1).
    pendiente. Un derivado que necesite ficheros temporales usa la carpeta
    temporal del sistema, nunca los dos árboles.
 
-4. **Check `estado`** (D3a): un check **obligatorio** en cada PR que ejecuta
-   los `comprueba` de la configuración **de la base** contra el árbol del PR
-   y la rama `taskcode/tareas`; **no se cachea** (su resultado depende del
-   estado, que no está en el árbol) y un `comprueba` en rojo lo pone en rojo
-   y **bloquea** el merge. Es un job de la clase que ejecuta código del PR sin
-   privilegios (F1-8 dec. 1: sin secretos, permisos de solo lectura), así que
-   en CI no usa la confianza local. Sustituye la puerta que hoy da el paso
-   1/22 de `verificar.bat` en ToolsaWeb: el bloqueo pasa de la verificación al
+4. **Check `estado`** (D3a): un check **obligatorio** en cada PR. Toma la
+   configuración `derivados` **y los scripts de los `comprueba` de la rama
+   base** (`origin/develop`), **nunca del árbol del PR**, y los ejecuta contra
+   el árbol del PR (como datos, en `TASKCODE_CODIGO`) y la rama
+   `taskcode/tareas`; **no se cachea** (su resultado depende del estado, que no
+   está en el árbol) y un `comprueba` en rojo lo pone en rojo y **bloquea** el
+   merge. Si el PR cambia un comprobador o la configuración de derivados, ese
+   cambio **no se usa hasta que esté en la base**; un PR que toca
+   `.taskcode/` sale, además, riesgo alto por ruta (F1-10). Es un job que
+   ejecuta código de la base sobre datos del PR, como `carril` en F1-8 (D1b):
+   sin secretos y con permisos de solo lectura; el comprobador no importa ni
+   ejecuta nada del árbol del PR. Sustituye la puerta que hoy da el paso 1/22
+   de `verificar.bat` en ToolsaWeb: el bloqueo pasa de la verificación al
    check `estado`, y en la transición un `comprueba` solo avisa.
 
 5. **Lectura del estado**: `taskctl tareas --json` es la **API estable** para
@@ -667,8 +701,11 @@ una cifra medida falta de la tabla.
 | ordenes de la traduccion que no existen hoy | `1` | medido | B |
 | arbol de develop tras las transiciones | `identico` | medido | C |
 | check estado con un comprueba en rojo | `rojo (bloquea)` | medido | C |
-| hostiles en ejecucion descartados | `4/4` | medido | C |
-| transiciones con derivados en la rama de estado | `10` | medido | C |
+| check estado de un PR que cambia el comprobador | `rojo (bloquea)` | medido | C |
+| enlaces simbolicos en escribe | `no verificado (sin privilegios: WinError 1314)` | medido | C |
+| escribe a traves de una union de Windows | `rechazado` | medido | C |
+| hostiles en ejecucion descartados | `5/5` | medido | C |
+| transiciones con derivados en la rama de estado | `11` | medido | C |
 | comprobar de ToolsaWeb tras una transicion, hitos en docs (hoy) | `codigo 1` | medido | C2 |
 | comprueba de ToolsaWeb tras una transicion, disposicion nueva | `0 errores` | medido | C2 |
 | lineas generadas que salen de docs/planificacion/hitos.md | `1554 de 1779` | medido | C2 |
@@ -700,7 +737,7 @@ necesita una orden de huella local y `--perfil`; y la lectura del estado
 | Requisito | Decisión | Evidencia |
 |---|---|---|
 | P8 / RN-04 extensiones declaradas en vez de convenciones a mano; revisoras propias | tres claves, derivados, check `estado`, `tareas --json`, skill revisora (decisiones 1 a 7) | PoC A: 14 prácticas, 13 con destino, 48/48 citas; de 8 a mano, 3 comprobadas; A: 0 revisoras propias hoy, `revisores.ts:113-116` |
-| RF-70 regenerar derivados en cada transición | `derivados` que ejecuta `taskctl` en la transición, en la rama de estado (decisión 3); **no** un job de CI ni un hook, como sugería RF-70 | PoC C: 10 transiciones, lo generado en el commit de la transición; C2: `hitos.md` de ToolsaWeb |
+| RF-70 regenerar derivados en cada transición | `derivados` que ejecuta `taskctl` en la transición, en la rama de estado (decisión 3); **no** un job de CI ni un hook, como sugería RF-70 | PoC C: 11 transiciones, lo generado en el commit de la transición; C2: `hitos.md` de ToolsaWeb |
 | RR-11 roadmap sin interferir con la verificación | los derivados solo escriben bajo `tareas/` (D2a) | PoC C: árbol de `develop` idéntico; PoC B: `docs/planificacion/hitos.md` y 8 rutas disfrazadas rechazadas; C2: `docs/` sin tocar |
 | RF-71 estados propios / RF-07 descartar con motivo / RF-08 pausar | sin clave de estados (D1a): `descartada` y `pausa` nativos (F3-7) | PoC A: 11/11 archivadas llevan `descartada`, ninguna con un `estado` coherente hoy; PoC B: `estados` rechazada |
 | RF-40 / RF-41 comando del proyecto con perfiles y timeout | `verificacion.comando` + `perfiles.<p>.timeout` (decisión 1) | PoC B: 10 claves de ToolsaWeb → 2 bloques y 16 valores; perfil fuera de F1-7 rechazado |
````

## Excluido del diff (6 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 054468e9c3f2315bb351d2ecb0e970649becb914..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/03-en-revision/TASK-012/poc/poc-f1-11.log               |   20 ++-
 tareas/03-en-revision/TASK-012/poc/poc-f1-11.py                |   89 +++++++++--
 tareas/03-en-revision/TASK-012/revision/informe-revision-1.md  |  104 ++++++++++++-
 tareas/03-en-revision/TASK-012/revision/informe-revision-2.md  |   21 +++
 tareas/03-en-revision/TASK-012/revision/peticion-revision-2.md | 1080 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-012/tarea.md                        |    9 +-
 6 files changed, 1302 insertions(+), 21 deletions(-)
````
