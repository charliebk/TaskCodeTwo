# Informe de revision — TASK-012 (ronda 1)

- Commit revisado: 7c532cc7aa97b984ec835c5159843ecedeb6f4df
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

Veredicto propuesto por el revisor: **cambios-solicitados** (4 IMPORTANTE abiertos). Lo escribe quien orquesta con `taskctl veredicto`.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMPORTANTE-1 | IMPORTANTE | corregido | docs/adr/F1-11-extensiones-declaradas.md:375 | implementacion |
| IMPORTANTE-2 | IMPORTANTE | corregido | docs/adr/F1-11-extensiones-declaradas.md:350 | implementacion |
| IMPORTANTE-3 | IMPORTANTE | corregido | docs/adr/F1-11-extensiones-declaradas.md:452 | implementacion |
| IMPORTANTE-4 | IMPORTANTE | corregido | docs/adr/F1-11-extensiones-declaradas.md:457 | implementacion |
| MENOR-1 | MENOR | corregido | docs/adr/F1-11-extensiones-declaradas.md:340 | implementacion |
| MENOR-2 | MENOR | corregido | docs/adr/F1-11-extensiones-declaradas.md:341 | implementacion |
| MENOR-3 | MENOR | corregido | docs/adr/F1-11-extensiones-declaradas.md:457 | implementacion |
| MENOR-4 | MENOR | corregido | docs/adr/F1-11-extensiones-declaradas.md:423 | implementacion |
| MENOR-5 | MENOR | corregido | docs/adr/F1-11-extensiones-declaradas.md:477 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### IMPORTANTE-1 — Las traducciones de los pilotos no son funcionales con las reglas del propio ADR (criterio 1 de tarea.md)
- Donde: docs/adr/F1-11-extensiones-declaradas.md:375 (ejemplo de ToolsaWeb), docs/adr/F1-11-extensiones-declaradas.md:60 (práctica 3) y tareas/03-en-revision/TASK-012/poc/poc-f1-11.py:516 (`CFG_ODT`).
- Que pasa: la PoC B solo comprueba que las dos traducciones **validan** y que el fichero de la orden existe; no comprueba que la orden declarada haga lo que se espera tras D2a. Con D2a el estado vive en `taskcode/tareas` y los derivados no pueden escribir en `docs/`, pero:
  1. El `comprueba` de ToolsaWeb (`grafo-tareas.py --comprobar`) verifica, además de ciclos y dependencias, que el Mermaid de `docs/planificacion/hitos.md` §4 y las tablas de §1 (rama de código) están al día con `tareas/`. Nadie puede regenerarlas (la PoC B rechaza precisamente `docs/planificacion/hitos.md` como hostil), así que tras la primera transición que cambie algo el derivado queda rojo para siempre. El ADR solo menciona la mitad del problema (las tablas de §1, en «Preguntas abiertas»); el Mermaid de §4 no aparece.
  2. La práctica 3 de OpenDockTrading cita `comprobar-fallos.py:302-307` (`tareas_vivas()`), pero `CFG_ODT` solo mueve `comprobar-modulos.py` a `derivados`. `comprobar-fallos.py --probar` (verificar.bat:53) sigue dentro del perfil cacheado y lee el estado: justo lo que D3b descarta («un acierto escondería un fallo nuevo»).
  3. El `comprueba` pasa de bloquear (en ToolsaWeb es el paso 1/22 de `verificar.bat`, rojo = verificación en rojo) a avisar sin parar la transición (PoC C, T6). El ADR no lo dice como cambio de garantía, y la decisión no define qué ejecuta el check `estado` que debería sustituir esa puerta (solo «lee la rama de estado», líneas 318-325; no consta que ejecute los `comprueba` ni desde dónde se leen ni con qué confianza). El contrato de la práctica 3 queda a medias.
- Reproduccion: copia de ToolsaWeb `cb46c0f1b` (scripts/grafo-tareas.py, docs/planificacion/hitos.md, tareas/) en la carpeta temporal; línea base: `python -I -B scripts/grafo-tareas.py --comprobar` -> `[ok] ... hitos.md al día ...`, código 0. Se cambia `sprint: 99` en `tareas/00-planificadas/TASK-009/tarea.md` (lo que haría una transición en la rama de estado, con `hitos.md` intocable por D2a): salen `[error] hitos.md §1: TASK-009 está en la sección H12 y su sprint es 99`, `[error] el Mermaid de docs\planificacion\hitos.md está desfasado: ejecuta --escribir` y `[error] tareas\ROADMAP.md está desfasado`, código 1. Lectura de `ODT scripts/comprobar-fallos.py:328-331` (`probar()` llama a `tareas_vivas()` y asevera «el catálogo real no pasa») y de `verificar.bat:53`.
- Impacto: el ADR afirma «8 prácticas a mano pasan a 0» y que el contrato cubre lo que ToolsaWeb y OpenDockTrading hacen a mano; para las prácticas 2 y 3 de ToolsaWeb el ejemplo que se da como «la traducción que valida la PoC B» daría un rojo permanente, y en OpenDockTrading queda un comprobador de estado dentro de un perfil que se cachea por árbol.
- Sugerencia: (a) decidir qué pasa con el Mermaid/tablas de `hitos.md` de ToolsaWeb (documento manual sin bloque generado, o bloque generado en `tareas/`) y partir `grafo-tareas.py --comprobar` en la parte pura de estado (ciclos, dependencias, criterios) y la que toca `docs/`; corregir el ejemplo de la línea 375 y la práctica 2; (b) añadir `comprobar-fallos.py` a los `derivados` de `CFG_ODT` (y a la tabla de la práctica 3); (c) escribir en «Decisión» qué ejecuta el check `estado` (los `comprueba`, leídos de la base, con `confianza`) y declarar que el bloqueo pasa de la verificación al check `estado`; (d) añadir a la PoC un caso que ejecute el script real de ToolsaWeb tras un cambio de estado.

### IMPORTANTE-2 — La garantía «un derivado solo escribe bajo `tareas/` y lo demás se descarta» se salta con rutas triviales y con ficheros ya modificados
- Donde: docs/adr/F1-11-extensiones-declaradas.md:350 (validación de `escribe`), docs/adr/F1-11-extensiones-declaradas.md:388 (decisión 3), tareas/03-en-revision/TASK-012/poc/poc-f1-11.py:252 (reservadas) y tareas/03-en-revision/TASK-012/poc/poc-f1-11.py:781 (`permitidas` y detección por `git status --porcelain`, poc-f1-11.py:744).
- Que pasa: dos huecos distintos, ambos fuera de lo que cubren los 13 hostiles de B y los 7 pasos de C.
  1. **Validación**: `tareas/ROADMAP.md` y `tareas/TASK-NNN/` se rechazan por comparación literal (`r in RESERVADAS_TAREAS`, regex `tareas/TASK-\d+`). Se aceptan `tareas/./ROADMAP.md`, `tareas/roadmap.md` (NTFS no distingue mayúsculas, así que es el mismo fichero), `tareas/./TASK-001/tarea.md`, `tareas//TASK-001/tarea.md`, `tareas/task-001/tarea.md`, `tareas/` (toda la rama de estado), `tareas/.git` (el fichero de enlace del worktree) y `tareas/x:ads`.
  2. **Ejecución**: «escribió fuera» se calcula como diferencia de líneas de `git status --porcelain` antes y después. Un derivado que modifica **otra vez** un fichero que ya estaba sucio no cambia ninguna línea: el `tarea.md` de la propia transición (que `taskctl` acaba de escribir) o la salida de un derivado anterior. Tampoco ve lo escrito en rutas ignoradas del árbol de código.
- Reproduccion: `validar()` del guion con `derivados: [{comando, tipo: genera, escribe: [R]}]` y R en la lista anterior: todas «ACEPTADA» (`tareas/a/../../src/x`, `tareas/..\x`, `/tareas/x` y `C:/x` sí se rechazan). Después, repo temporal de la PoC C con la función `transicion` del guion y un derivado `genera` (`escribe: [tareas/grafo-hitos.md]`) que además reescribe `estado: "en-diseno"` por `estado: "terminada"` en `TASK-001/tarea.md`: sin aviso, el commit de la rama de estado contiene `['TASK-001/tarea.md', 'grafo-hitos.md']` y la tarea queda `terminada`. Con otro derivado que escribe `.estado/x.txt` (ignorado por `.gitignore`) en `TASKCODE_CODIGO`: sin aviso y el fichero queda.
- Impacto: un derivado (o un `comprueba`, que «no escribe nada») puede cambiar el `estado` o la aprobación de una tarea en el commit de la transición sin que nadie se entere; el ADR presenta esa defensa como demostrada («lo suyo se descarta», PoC C T7) cuando solo se prueba un fichero nuevo sin seguimiento en código. Es código de confianza, pero el ADR vende `escribe` como límite.
- Sugerencia: especificar en el ADR que el validador normaliza antes de comparar (separadores, segmentos `.` y vacíos, minúsculas, sin `:` ni nombres reservados de Windows, prohibido `tareas/` entero y `.git`) y compara las reservadas ya normalizadas e insensibles a mayúsculas; que la comprobación de ejecución compara **contenido** (hash de cada fichero de `tareas/` fuera de `escribe`, incluido el `tarea.md` de la transición, y `git status --ignored` en el código) y no líneas de `status`; añadir estos casos a los hostiles de B y C.

### IMPORTANTE-3 — «82 → 3 líneas» y `taskctl tareas --json` no sostienen lo que los pilotos necesitan
- Donde: docs/adr/F1-11-extensiones-declaradas.md:452 (tabla `ventaja`), docs/adr/F1-11-extensiones-declaradas.md:301 («No verificado») y tareas/03-en-revision/TASK-012/poc/poc-f1-11.py:894 (`vivas_desde_json`).
- Que pasa: la PoC D solo compara el **conjunto de IDs vivos** (`v_json == v_hoy`). Pero los consumidores reales leen más: `ODT comprobar-modulos.py:95-103` devuelve `{id: texto de tarea.md}` y lo recorre con una regex para las rutas citadas (líneas 301-304); `ODT comprobar-fallos.py:302-308` cuenta los criterios `- [ ]` del cuerpo; `TW grafo-tareas.py --comprobar` exige «entre 3 y 8 criterios». Los campos mínimos que fija el ADR (`id, estado, etiquetas, dependencias, sprint`) no traen el cuerpo ni los criterios, y esos consumidores conservan su propio glob (`0?-*/TASK-*/tarea.md`) para leer el texto, de modo que seguirían ciegos con la carpeta fija aunque el lector común desapareciera. Además son tres consumidores (dos en ODT, uno en TW), no uno de 3 líneas.
- Reproduccion: `grep`/lectura de los tres ficheros en los commits fijos (comandos `git show f8a9b5f27:scripts/comprobar-modulos.py` y `comprobar-fallos.py`, `cb46c0f1b:scripts/grafo-tareas.py`); el log de la PoC D imprime solo `mismo conjunto que hoy=True` sobre IDs. La cifra `82 -> 3` sale de contar `lector-tareas.py` entero (44) y 3 funciones de `grafo-tareas.py` (38) frente a `inspect.getsource(vivas_desde_json)`.
- Impacto: la ventaja medible más llamativa del apartado D es una proyección sobre una interfaz simulada y que no cubre los datos que se consumen; presentada en la tabla de cifras «medidas» (el guion falla si no cuadra consigo mismo, no con la realidad).
- Sugerencia: añadir a la salida mínima de `tareas --json` el cuerpo (o la lista de criterios y el texto) y que la PoC D recalcule con ellos lo que hoy calculan los consumidores (rutas citadas y recuento de criterios, comparados con los de hoy); o rotular la cifra «proyectada, no medida» y contar los 3 consumidores. Recoger en «Consecuencias» que `tareas --json` es contrato de F2/F3 con esos campos.

### IMPORTANTE-4 — La ventaja en claves y en prácticas mide anidación, no esfuerzo ni cobertura real
- Donde: docs/adr/F1-11-extensiones-declaradas.md:442 (`10 -> 2`), docs/adr/F1-11-extensiones-declaradas.md:457 («de 8 prácticas a mano se pasa a 0») y docs/adr/F1-11-extensiones-declaradas.md:74.
- Que pasa: (a) `10 -> 2` y «12 claves caben en 2 bloques» comparan claves planas de la 0.16 con claves de **primer nivel** del esquema. Contando valores a rellenar en las traducciones de la propia PoC (`CFG_TW`, `CFG_ODT`, calculado con `yaml.safe_load` sobre el guion): ToolsaWeb **10 → 14** y OpenDockTrading **2 → 12** (por `shell`, `runner`, `programada.{comando,cron,runner}`, `huella_arbol`, `derivados`, `confianza_incluye`). Buena parte son capacidades nuevas (caché, runner, job programado), pero la cifra no distingue «menos que escribir» de «más ordenado». El esquema tiene 17 claves hoja; el tope de 12 de F2-7 se interpreta como primer nivel sin decirlo. (b) «8 → 0 prácticas a mano»: de las 8, la 7 (caché por paso) y la 11 (cerrojo) «siguen dentro del comando» (texto del propio inventario), la 12 es una convención de comillas más un validador, la 14 queda fuera de alcance, y la 4 depende de una orden que no existe. Con mecanismo nuevo y comprobado quedan 1, 3 (ver IMPORTANTE-1) y 10. El ADR sí dice que OpenDockTrading no baja de claves (2 → 2), pero no que su configuración sube a 12 valores.
- Reproduccion: `python -I -B` con `importlib` sobre el guion, función de hojas sobre `yaml.safe_load(CFG_TW)` y `CFG_ODT`: `CFG_TW 14 [...]`, `CFG_ODT 12 [...]`. Lectura del inventario (columnas «En la 0.16» y «Punto de extensión») para las prácticas 7, 11 y 12.
- Impacto: la restricción rectora de la persona es «eficiente y con ventajas medibles» y fiel a los datos; esta cifra puede leerse como una reducción de trabajo que no existe y la frase final de «Ventaja medida» no la matiza.
- Sugerencia: dar las dos medidas (primer nivel y valores a rellenar) por piloto, separar «medido» (A, B, C, E) de «proyectado» (D, check `estado`, `tareas --json`), y reescribir la conclusión: de 8 prácticas a mano, N con mecanismo declarado y comprobado, 2 siguen dentro del comando del proyecto, 1 es convención y 1 queda fuera; aclarar que el tope de 12 de F2-7 cuenta primer nivel (o redefinirlo).

### MENOR-1 — `presupuesto_freno_s` no es un `timeout`
- Donde: docs/adr/F1-11-extensiones-declaradas.md:340.
- Que pasa: la fila del esquema dice que `perfiles.<p>.timeout` sustituye a `presupuesto_freno_s`, pero el dato recogido (`TW config.yml:17` y `flujo-de-verificacion.md:35`, «un corte es excedido, no bloquea») es un presupuesto blando; el ADR no dice qué pasa al agotar un `timeout` (¿qué código sale?, D6a dice que todo distinto de 0 es rojo).
- Reproduccion: `git show cb46c0f1b:docs/verificacion/flujo-de-verificacion.md | sed -n 35p` y `:296`.
- Impacto: la `rapida` de ToolsaWeb pasaría de no bloquear a bloquear por tiempo sin que el ADR lo anote.
- Sugerencia: anotar el cambio de semántica (o conservar un campo «blando» para la `rapida`) y fijar el resultado de un `timeout`.

### MENOR-2 — Los nombres de runner no coinciden con F1-7 decisión 5 y la divergencia no consta
- Donde: docs/adr/F1-11-extensiones-declaradas.md:341.
- Que pasa: F1-7 (decisión 5) escribe `github` «con el sistema: `windows` o `linux`»; el ADR define `github-windows`, `github-linux` y `github-macos` (macOS no aparece en F1-7) y el validador rechaza `github` (comprobado: `runner 'github': github-windows`). No figura en «Divergencia con el plan» ni en las consecuencias para F5-1.
- Reproduccion: `validar()` con `runner: github`; lectura de `docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md` decisión 5.
- Impacto: dos vocabularios para la misma clave entre dos ADR aceptados.
- Sugerencia: alinear con F1-7 o declarar la divergencia (y el precio de macOS, que F1-7 no calcula).

### MENOR-3 — Las cifras repetidas en la prosa no las guarda la PoC
- Donde: docs/adr/F1-11-extensiones-declaradas.md:457.
- Que pasa: el guion falla si cambia una cifra de la tabla `ventaja` (comprobado: `2 -> 2` cambiado a `2 -> 1` da `FALLOS (1)` y salida 1), pero el párrafo «En resumen» repite 82, «5 de las 12», 8 y 12 sin ninguna comprobación.
- Reproduccion: copia del ADR con `**82 líneas**` -> `**99 líneas**` y `el esquema usa **5 de las 12**` -> `**3 de las 12**`: la PoC termina con código 0 y «todo como se espera».
- Impacto: la frase que más se cita puede dejar de ser cierta sin aviso.
- Sugerencia: que el guion extraiga las cifras del párrafo (o que el párrafo las genere desde la tabla) y fallar si no coinciden; o quitar los números de la prosa.

### MENOR-4 — Pregunta abierta de F1-10 sin respuesta ni traspaso
- Donde: docs/adr/F1-11-extensiones-declaradas.md:423.
- Que pasa: F1-10 (`Preguntas abiertas`, «`.claude/` y otros ficheros que cambian el comportamiento del agente… si debe ir en el suelo, lo decide F1-8 o F1-11») queda tocada por la práctica 14 («se quedan en `.claude/`, la 2.0 no cuenta con ellas»), pero este ADR no dice si `.claude/` entra en el suelo de riesgo alto ni lo devuelve a F1-8.
- Reproduccion: `sed -n 662,666p docs/adr/F1-10-carriles-y-modo-automatico.md`.
- Impacto: la costura queda sin dueño.
- Sugerencia: una línea en «Fuera de alcance» o «Preguntas abiertas»: decidido sí/no o pendiente de F1-8.

### MENOR-5 — RF-72 está enlazado a algo que no pide
- Donde: docs/adr/F1-11-extensiones-declaradas.md:477.
- Que pasa: RF-72 es «skills y agentes instalables en cualquier proyecto, sin referencias internas» (requisito sobre el propio plugin); la fila lo usa para las revisoras propias del proyecto (D5a).
- Reproduccion: `sed -n 197p docs/plan/REQUISITOS-FUNCIONALES.md`.
- Impacto: trazabilidad forzada; no cambia la decisión.
- Sugerencia: citar RN-04 (ya citado) para D5a y quitar RF-72 de la lista o explicar el vínculo.

## Verificacion realizada

- Entregable: documentación y guion, sin build, linter ni suite propios en el repo (se dice expresamente). Puerta aplicable: `bash .github/scripts/comprobar-repo.sh` -> `Tareas: 93; enlaces relativos: 91; fallos: 0`.
- PoC reproducida con `python -I -B tareas/03-en-revision/TASK-012/poc/poc-f1-11.py <trabajo> docs/adr/F1-11-extensiones-declaradas.md .. ~/.claude/plugins/cache/taskcode-marketplace/taskcode-plugin/0.16.1`, con la carpeta de trabajo en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-012-r1`: salida 0 en 1 min 22 s; el log es **idéntico** al versionado salvo la línea 2 (ruta de trabajo): A 14 prácticas, 43/43 citas; B 2/2 válidas, 13/13 hostiles, 5/12; C 7/7 con `develop` idéntico; D 0 y 0 en carpeta fija; E 7 grupos iguales; tabla `ventaja` 22/22 OK. ToolsaWeb y OpenDockTrading solo con `git show`/`git archive` de los commits fijos; copia de ToolsaWeb ejecutada solo en la carpeta temporal.
- Citas abiertas por mi cuenta (más de 4) y semántica comprobada: `TW tareas/05-archivadas/TASK-171/tarea.md:6,9` («descartada», `estado: planificada`); `TW scripts/verificar.bat:124-127` (`grafo-tareas.py --comprobar`); `ODT scripts/verificar.bat:51,53,59,68-70` (y que el .bat no lee argumentos, así que `--perfil` no lo rompe); `ODT scripts/comprobar-fallos.py:302-308` y `comprobar-modulos.py:87-104`; `TW docs/verificacion/flujo-de-verificacion.md:164-166,180-184` (mismo árbol; código 3); `TW scripts/cerrojo-verificacion.py:6-7`; `ODT tareas/05-archivadas/TASK-068/tarea.md:30-40` (la «transición sigue y avisa» es del roadmap nativo de TaskCode, no una extensión de ODT); `TW .taskcode/config.yml:1-32` (incluida la nota de que el presupuesto del freno no bloquea). Dicen lo que el ADR afirma, con los matices de IMPORTANTE-1, MENOR-1 y esta última (la práctica 2 de ODT no tiene derivado propio hoy).
- Mutaciones fuera de la PoC: (1) cifra de la tabla `ventaja` `2 -> 2` a `2 -> 1`: detectada (`FALLOS (1)`, salida 1); (2) cifras de la prosa: no detectadas (MENOR-3); (3) `escribe` con 14 rutas alternativas: 10 aceptadas (IMPORTANTE-2); (4) derivado que reescribe `tarea.md` y otro que escribe en una ruta ignorada: sin aviso (IMPORTANTE-2); (5) 15 configuraciones adicionales: `hooks`, `verificacion.hook`, `derivados[].hook` y `fases: post-checkout` se rechazan (ningún punto de extensión puede depender de un hook: cumplido); `timeout: true`, cron de 6 campos, `huella_arbol: **/*`, timeout de derivado 0 y falta de `completa` se rechazan; `cron: a b c d e`, `huella_arbol: src/**` y `comando` con `&` se aceptan (por diseño: el esquema solo exige 5 campos y las órdenes son libres bajo `confianza`; sin hallazgo).
- Honestidad de la ventaja: OpenDockTrading 2 → 2 sí se dice (tabla, «Lectura» de B y divergencia 6); lo que no se dice está en IMPORTANTE-3 e IMPORTANTE-4.
- Coherencia con F1-1..F1-10: confianza (F1-8 dec. 5), claves desconocidas (F1-9 D4a), tope y falla cerrada (F1-10), «el push a `taskcode/tareas` no dispara Actions» (F1-1, línea 100), `RAMA_BASE` solo en rápida e impacto (F1-7 dec. 1), job programado (F1-7 dec. 2): coherentes. Divergencias: MENOR-2 (runners) y MENOR-4 (pregunta de F1-10). Las divergencias con RF-70 (no job de CI ni hook), RF-71 (sin estados configurables, D1b) y F5-1 («sin scripts nuevos» en duda para ToolsaWeb: `--perfil` y orden de huella local) constan y son ciertas (`verificar.bat:68-79` solo acepta `--rapida`, `--selectiva`, `--pesada`).
- IDs citados: existen RN-04, RF-07, RF-08, RF-40..43, RF-70..72, RR-11, RR-14, H2, H16, P8 y las tareas F0-2, F1-1..F1-12, F1-15, F2-7, F3-4, F3-7, F3-8, F5-1, F5-4, F5-6 (con su texto en el plan). Único vínculo dudoso: MENOR-5.
- Criterios de tarea.md: el 1 («cubre lo que hacen a mano») no se cumple del todo por IMPORTANTE-1, -3 y -4; el 2 (lo que pide F1-11 y los requisitos) se cumple en forma, con las reservas anteriores. Plan aprobado: seguido (7 transiciones en C en vez de 5 y PoC D con los dos pilotos, declarado en «Divergencia con el plan»); `taskctl doctor` no ejecutado, declarado en el ADR.
- Carpeta temporal `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-012-r1` borrada al terminar; sin cambios en el repo salvo este informe (`git status` limpio antes de escribirlo); `ToolsaWeb` y `OpenDockTrading` intactos.

## Lecturas fuera del contexto permitido

- tareas/03-en-revision/TASK-012/poc/poc-f1-11.py y poc-f1-11.log: pedidos por quien orquesta; hacen falta para reproducir.
- docs/plan/REQUISITOS-FUNCIONALES.md y docs/plan/PLAN-TASKCODE-2.md (líneas de RF/RN/RR, F1-11, F2-7, F5-1): comprobar que los IDs existen y qué dicen.
- docs/auditoria/AUDITORIA-EXTERNA-2026-11-13.md (H2, H16): comprobar IDs.
- docs/adr/F1-1, F1-7, F1-8 y F1-10: coherencia con las decisiones que el ADR cita (runners, confianza, push sin Actions, pregunta abierta de `.claude/`).
- ToolsaWeb (`cb46c0f1b`) y OpenDockTrading (`f8a9b5f27`), solo lectura con `git show` y `git archive`: citas del inventario y reproducción de IMPORTANTE-1 e IMPORTANTE-3.
- ~/.claude/plugins/cache/taskcode-marketplace/taskcode-plugin/0.16.1 (`skills/task-workflow/revision.md`, `src/core/revisores.ts`): procedimiento de revisión y cita 13.
- tareas/03-en-revision/TASK-012/revision/peticion-revision-1.md: la petición.
