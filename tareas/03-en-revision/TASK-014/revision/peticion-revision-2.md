# Peticion de revision — TASK-014 (ronda 2)

- Tarea: TASK-014 — F1-14 ADR distribucion e instalacion
- Rama revisada: feature/task-014-f1-14-adr-distribucion-e-instalacion
- Rama base: develop
- Commit revisado (HEAD): ef826d85f6c32ba273cb3ffd54e9f1da27506f09
- Fecha: 2026-11-14
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-014 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 36828cc4bd191733e0f3aa8fcc8b29c1c6f0ac7e (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (6 hallazgo(s)))

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

- tareas/03-en-revision/TASK-014/tarea.md
- tareas/03-en-revision/TASK-014/planificacion/
- docs/adr/F1-14-distribucion-e-instalacion.md
- docs/adr/README.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 5 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | tareas/03-en-revision/TASK-014/poc/poc-f1-14.py:247 | informe-revision-1.md |

## Commits a revisar (git log 36828cc4bd191733e0f3aa8fcc8b29c1c6f0ac7e..HEAD)

````
ef826d8 chore(TASK-014): informe de revision 1 corregido en 4456a70 y constancia de autorrevision ronda 2
4456a70 docs(adr): F1-14 corrige los hallazgos de la revision 1
2ba383f chore(TASK-014): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 36828cc4bd191733e0f3aa8fcc8b29c1c6f0ac7e..HEAD)

````diff
diff --git a/docs/adr/F1-14-distribucion-e-instalacion.md b/docs/adr/F1-14-distribucion-e-instalacion.md
index dc83e35..b2d2ada 100644
--- a/docs/adr/F1-14-distribucion-e-instalacion.md
+++ b/docs/adr/F1-14-distribucion-e-instalacion.md
@@ -35,7 +35,7 @@ primer criterio de la regla D1a, declarada antes de medir.
 
 | | Descripción | Veredicto |
 |---|---|---|
-| **L1** | CLI en **Rust**, binario nativo por target | **Elegida** por la regla D1a (PoC: gana el criterio 1 y otros dos; ver «Decisión», punto 1) |
+| **L1** | CLI en **Rust**, binario nativo por target | **Elegida** por la regla D1a (PoC: gana el criterio 1, medido, el 2, medido, y el 4, que es un juicio sobre postject; **sin el 4.º la regla no decide** y Rust se sostiene por el margen en el 1.º y el 2.º; ver «Decisión», punto 1) |
 | L2 | CLI en TypeScript empaquetado con Node SEA (o bun compile) | Descartada: pierde el arranque en caliente en los 3 repos y pesa 32,1 veces más; su cadena de empaquetado depende de postject en alfa |
 | **D1a** | Regla declarada antes de medir: 1.º arranque en caliente, 2.º tamaño, 3.º compilación cruzada reproducible, 4.º librerías mantenidas que cumplan F1-9, 5.º coste del agente; gana quien gane el 1.º y al menos dos más | **Elegida** (consigna) |
 | D1b / D1c | TypeScript siempre / Rust directamente | Descartadas: decidir sin medir |
@@ -94,22 +94,32 @@ python -I -B tareas/<estado>/TASK-014/poc/poc-f1-14.py \
   `develop` `8bebfd049` y de OpenDockTrading en `develop` `8141a08a7` (el de
   F1-12 y F1-18). Los originales solo se leen; el hash del árbol de las
   copias, con `.git`, se compara antes y después.
-- **Nada descargado**: sin bun, zig, targets de rustup ni imágenes Docker. La
-  red solo se usa para `npm ci` (dependencias fijadas), para bajar de
-  crates.io las crates propias de Linux y macOS en el intento de compilación
-  cruzada (con `Cargo.lock` fijado) y para el GET del propio comando, fuera
-  de la tabla comprobada.
+- **Ni toolchains, ni targets, ni imágenes descargados**: sin bun, zig,
+  targets de rustup ni imágenes Docker. Sí se baja de la red: las
+  dependencias de `npm ci` (fijadas en `package-lock.json`); en la sección X,
+  las crates propias de Linux y macOS (p. ej. `openssl-probe`), que el guion
+  baja de crates.io con `cargo build --target …` **sin `--offline`** porque no
+  estaban en la caché (con `Cargo.lock` fijado; son dependencias, no
+  toolchains); y el GET del propio comando, fuera de la tabla comprobada.
+- **El guion se ejecuta desde cualquier estado de la tarea**: toma su
+  carpeta `poc/` de su propia ruta (`__file__`), no de una ruta fija.
 - **Determinismo**: dos ejecuciones completas del guion, cada una con sus
   propias builds y copias, dieron el mismo log salvo la sección T.
 - **Comprobación del ADR (G)**: el guion lee la tabla `cifras` y sale con 1
   si una cifra no es la medida, falta o sobra, o si su fuente no es la
   sección que la mide; también el `resumen`, las secciones, el estado, las
   reglas obligatorias, el ganador de D1 en la decisión, los IDs (en el plan,
-  en los requisitos o con ADR propio), los enlaces y la fila del índice.
-  Catorce copias mutadas del ADR fallan cada una por su motivo.
+  en los requisitos o con ADR propio), los enlaces y la fila del índice, y
+  **las cifras repetidas en la prosa** fuera de la tabla (las «veces», los
+  MiB, «en los 3 repos», las líneas, los bytes, los criterios que gana el
+  ganador de D1 y que los dos rechazan la clave duplicada). Veintiuna copias
+  mutadas del ADR fallan cada una por su motivo.
 - **Tiempos de reloj**: van al log en su sección T y no son cifras
   comprobadas; en la tabla solo entran las relaciones estables («Rust más
   rápido en caliente en los 3 repos»).
+- **Criterios 4 y 5 de D1a**: no son medidas del guion sino **juicios**
+  declarados en él con su motivo (constantes `CRITERIO_4` y `CRITERIO_5`, a
+  partir de `DEPENDENCIAS.md` y `NOTAS.md`); su fila en la tabla lo dice.
 
 Salida observada, recortada (la completa está en el log):
 
@@ -208,6 +218,12 @@ TypeScript sobre repo vacio: frio (1.a ejecucion de copia nueva) 1180 ms ; calie
 TypeScript sobre ToolsaWeb: frio (1.a ejecucion de copia nueva) 1753 ms ; caliente mediana de 20 671 ms (min 661, max 751)
 TypeScript sobre OpenDockTrading: frio (1.a ejecucion de copia nueva) 1333 ms ; caliente mediana de 20 308 ms (min 299, max 337)
 build en limpio Rust: 38.0 s ; TypeScript (npm ci + tsc + esbuild + SEA + postject): 17.2 s
+lanzador directo + Rust sobre repo vacio: caliente mediana de 20 55 ms (+0 ms sobre el binario directo)
+lanzador tc2.cmd + Rust sobre repo vacio: caliente mediana de 20 93 ms (+38 ms sobre el binario directo)
+lanzador sh tc2 + Rust sobre repo vacio: caliente mediana de 20 102 ms (+47 ms sobre el binario directo)
+lanzador sh tc2-fecha + Rust sobre repo vacio: caliente mediana de 20 139 ms (+84 ms sobre el binario directo)
+lanzador sh tc2-sha256 + Rust sobre repo vacio: caliente mediana de 20 182 ms (+127 ms sobre el binario directo)
+lanzador tc2.ps1 + Rust sobre repo vacio: caliente mediana de 20 359 ms (+304 ms sobre el binario directo)
 ```
 
 Lectura: son cifras de una pasada y cambian entre ejecuciones (por eso no
@@ -231,12 +247,16 @@ gana: Rust (el 1.o y al menos dos mas)
 
 ### G — El ADR y sus mutaciones
 
-Las catorce copias mutadas (una cifra cambiada, una fila borrada, una fila
-sin medida, un tiempo de reloj en la tabla, una fuente cambiada, un número
-inventado en el resumen, un ID que no existe, un enlace a `tareas/`, un
-enlace roto, una sección borrada, el estado «propuesta», «engram es
-obligatorio», el ganador de D1 cambiado y «No verificado» sin macOS) fallan
-cada una por su motivo (log, sección G).
+Las veintiuna copias mutadas (una cifra cambiada, una fila borrada, una
+fila sin medida, un tiempo de reloj en la tabla, una fuente cambiada, un
+número inventado en el resumen, un ID que no existe, un enlace a `tareas/`,
+un enlace roto, una sección borrada, el estado «propuesta», «engram es
+obligatorio», el ganador de D1 cambiado, «No verificado» sin macOS y, en la
+prosa, unos MiB cambiados, las «veces» cambiadas, el número de repos
+cambiado, las líneas de código cambiadas, los bytes de la decisión
+cambiados, los criterios que gana Rust quitados y «rechazan» la clave
+duplicada cambiado a «aceptan») fallan cada una por su motivo (log,
+sección G).
 
 ### Cifras de la PoC
 
@@ -313,8 +333,8 @@ si la fuente no es la sección que la mide.
 | D1: criterio 1 arranque en caliente | `Rust` | D1 |
 | D1: criterio 2 tamano | `Rust` | D1 |
 | D1: criterio 3 compilacion cruzada reproducible | `ninguno` | D1 |
-| D1: criterio 4 librerias mantenidas que cumplan F1-9 | `Rust` | D1 |
-| D1: criterio 5 coste del agente | `ninguno` | D1 |
+| D1: criterio 4 librerias mantenidas que cumplan F1-9 (juicio, no medida) | `Rust` | D1 |
+| D1: criterio 5 coste del agente (juicio, no medida) | `ninguno` | D1 |
 | D1: criterios ganados por el ganador del 1.o sin contar el 4 ni el 5 | `1` | D1 |
 | D1: ganador (el 1.o y al menos dos mas) | `Rust` | D1 |
 <!-- cifras:fin -->
@@ -356,8 +376,8 @@ se ejecutaron, y los importes de Authenticode no se pudieron consultar.
    | 1.º arranque en caliente | **Rust** | más rápido en caliente (mediana de 20) en los 3 repos y en frío en los 3 (T; los milisegundos son de reloj y están en el log, no en la tabla) |
    | 2.º tamaño | **Rust** | 2 720 768 frente a 87 430 656 bytes en win-x64, 32,1 veces menos (M) |
    | 3.º compilación cruzada reproducible | ninguno | ninguno de los dos compila para Linux ni macOS con lo instalado (X); en win-x64, dos builds en limpio dan el mismo sha256 en TypeScript, y en Rust solo con `/Brepro` (B) |
-   | 4.º librerías mantenidas que cumplan F1-9 | **Rust** | las dos librerías YAML están mantenidas y rechazan la clave duplicada en una tarea real (S); la cadena de TypeScript depende de postject `1.0.0-alpha.6`, sin publicar desde 2023-05-19 (D, `poc-ts/DEPENDENCIAS.md`) |
-   | 5.º coste del agente | ninguno | ~8 frente a ~9 min de reloj y 0 errores de compilación en los dos (`NOTAS.md`); los tokens de cada implementación no se anotaron |
+   | 4.º librerías mantenidas que cumplan F1-9 (**juicio**, no medida) | **Rust** | las dos librerías YAML están mantenidas y rechazan la clave duplicada en una tarea real (S); la cadena de TypeScript depende de postject `1.0.0-alpha.6`, sin publicar desde 2023-05-19 (D, `poc-ts/DEPENDENCIAS.md`) |
+   | 5.º coste del agente (**juicio**, no medida) | ninguno | ~8 frente a ~9 min de reloj y 0 errores de compilación en los dos (`NOTAS.md`); Rust tuvo un fallo funcional propio (TLS) y TypeScript necesitó **3 iteraciones extra** de compilar y comparar para igualar a Rust (147 frente a 215 líneas, `poc-ts/NOTAS.md`); los tokens de cada implementación no se anotaron |
 
    Rust gana el 1.º y otros dos (2.º y 4.º): la regla se cumple.
    **Sensibilidad**: el 4.º se decide por una herramienta de build, no de
@@ -427,6 +447,27 @@ se ejecutaron, y los importes de Authenticode no se pudieron consultar.
      deja con un renombrado atómico y lo ejecuta. Si el hash no casa, borra
      la descarga y sale con el código de precondición y el arreglo; nunca
      ejecuta un binario sin comprobar;
+   - **binario ya en la caché: no se reverifica en cada llamada.** La caché
+     es por versión (`cli/<version>/`), así que un `cli.lock` nuevo apunta a
+     otra carpeta y obliga a descargar y comprobar. El sha256 contra
+     `cli.lock` se calcula al descargar, en `tc2 doctor --recursos` (que
+     borra y vuelve a bajar un binario que no casa) y con
+     `tc2 actualizar --cli`. Motivo, medido en la PoC (sección T, de reloj,
+     una pasada): sobre el binario directo, un lanzador sh mínimo suma unos
+     47 ms; si además compara tamaño y fecha con `stat`, unos 84; si
+     recalcula el sha256 con `sha256sum`, unos 127. Lo que separa a Rust de
+     TypeScript en vacío son unos 85 ms: reverificar en cada llamada desde el
+     lanzador se comería la ventaja que decidió el criterio 1. Y no protege
+     de nada nuevo: quien puede reescribir la caché es un proceso del mismo
+     usuario, que en el modelo de F1-8 (zona Z1) también puede cambiar el
+     lanzador o `cli.lock`. La corrupción accidental la detecta `doctor`;
+   - **coste del lanzador**: la status line y los hooks pasan por él. En la
+     misma pasada, `tc2.cmd` sumó unos 38 ms y `tc2.ps1` unos 304 (arranque
+     de PowerShell). Por eso en Windows el lanzador por defecto es
+     `tc2.cmd`, nunca el `.ps1` en el camino caliente, y la status line
+     puede apuntar al binario de la caché directamente. Cuánto cuesta en el
+     camino real de Claude Code (con qué shell lanza hooks y status line) no
+     está verificado;
    - **sin red en el primer uso**: el lanzador lo dice en una línea, con la
      URL exacta y la ruta de la caché donde dejar el fichero a mano; un
      `tc2` instalado aparte se usa si su protocolo es compatible (punto 5);
@@ -477,11 +518,12 @@ se ejecutaron, y los importes de Authenticode no se pudieron consultar.
        ([azure.microsoft.com, precios](https://azure.microsoft.com/en-us/pricing/details/artifact-signing/)): **importe no verificado**. Su guía
        de inicio dice que los certificados Public Trust para **personas
        físicas** solo se emiten en EE. UU. y Canadá, y para organizaciones
-       también en la UE ([learn.microsoft.com, quickstart](https://learn.microsoft.com/en-us/azure/artifact-signing/quickstart)): Carlos
+       en EE. UU., Canadá, la Unión Europea, Reino Unido, Australia, Nueva
+       Zelanda, Japón, Corea del Sur, Singapur, Suiza, Noruega e Israel ([learn.microsoft.com, quickstart](https://learn.microsoft.com/en-us/azure/artifact-signing/quickstart)): Carlos
        como persona en España no puede usarlo; haría falta una organización
        o un certificado OV de una CA (coste no consultado).
-     - winget: las políticas del repositorio (documento con fecha
-       2026-08-30) piden que `InstallerUrl` sea el sitio de releases del
+     - winget: las políticas del repositorio (documento de 2021-05-22,
+       versión 1.0; página actualizada el 2026-08-30) piden que `InstallerUrl` sea el sitio de releases del
        autor y que no haya malware, y **no exigen firma**
        ([learn.microsoft.com, políticas de winget](https://learn.microsoft.com/en-us/windows/package-manager/package/windows-package-manager-policies)). Se firma al entrar
        igualmente por SmartScreen y antivirus (D4a); que la validación de
@@ -635,6 +677,13 @@ Este ADR no cambia el plan (D9a); propone estos cambios, para hacerlos con
   binario descargado por `curl`.
 - **Marca de la web (MOTW)** en Windows al descargar con `irm` o con el
   lanzador, y falsos positivos de antivirus en otras máquinas.
+- **El coste real del lanzador** en el camino de Claude Code (qué shell
+  lanza hooks y status line); la PoC solo midió lanzadores mínimos `.cmd`,
+  sh y `.ps1` sobre el binario de Rust en esta máquina (sección T, de reloj).
+- **Node SEA con `useCodeCache` o `useSnapshot`**: el binario de TypeScript
+  se midió sin ellos (Rust, en cambio, con LTO y `codegen-units = 1`);
+  podrían acortar su arranque, aunque con el margen medido es improbable
+  que cambien la relación.
 - **El lanzador, `cli.lock`, `tc2 version --json` y `tc2 doctor --recursos`**
   no existen: son contratos de este ADR.
 - **Attestation de GitHub y provenance de npm**: no ejecutadas (GitHub solo
diff --git a/docs/adr/README.md b/docs/adr/README.md
index 27e4acd..14d5a62 100644
--- a/docs/adr/README.md
+++ b/docs/adr/README.md
@@ -20,7 +20,7 @@ directo a la tabla de la Fase 1 y a su tarea.
 | [F1-10](F1-10-carriles-y-modo-automatico.md) | Carriles y modo automático: cuatro carriles ordenados más release y hotfix por tipo, tabla única de política por carril con su función pura (`accion` en `siguiente --json`), configuración que solo endurece (`politica_carriles`, `rutas_carril`), carril mínimo por ficheros con los dos nombres de cada renombrado, y paradas fijas (`approve`, `merge-pr`, tag, tope de rondas, subida de carril) | TASK-011 | aceptada |
 | [F1-11](F1-11-extensiones-declaradas.md) | Extensiones declaradas: inventario comprobado de las 14 prácticas que los pilotos hacen a mano, clasificadas en parche de TaskCode (se retira al migrar) o propio del proyecto, con el desglose de líneas; tres claves (`verificacion`, `derivados`, `confianza_incluye`) con esquema que falla cerrado y rutas normalizadas; derivados que ejecuta `taskctl` en la transición solo en la rama de estado, comparados por contenido; check obligatorio `estado`; `taskctl tareas --json` como lectura estable; sin estados propios; revisoras como skill del proyecto; «comandos sí, hooks solo como atajo opcional»; y la ventaja medida frente a la 0.16, con lo que no gana | TASK-012 | aceptada |
 | [F1-12](F1-12-actualizacion-entre-versiones.md) | Actualización entre versiones: el plan de migración como dato (`tc2 actualizar --simular` sin cambiar un byte, mismo hash, vacío en la segunda pasada), qué migra (aplanado, formato 2, archivadas a `descartada`, config a los bloques de F1-11), historia con `git subtree split`, transacción local con criterio de salida estructural y vuelta atrás automática, retirada de parches con capacidades y huecos bloqueantes, convivencia con la 0.16 (su lector de config aborta con `---`: ninguna orden de escritura entra en un proyecto migrado), nombres definitivos `tc2` y `taskcodetwo`, y orden de pilotos OpenDockTrading → TaskCodeTwo → ToolsaWeb | TASK-013 | aceptada |
-| [F1-14](F1-14-distribucion-e-instalacion.md) | Distribución e instalación: CLI en Rust por la regla D1a medida frente a TypeScript con Node SEA (mismo comando, salida idéntica sobre las 404 tareas de ToolsaWeb y las 96 de OpenDockTrading; más rápido en caliente en los 3 repos y 32,1 veces más pequeño), TLS con el almacén del sistema, lanzador sin runtime con `cli.lock` y descarga comprobada por sha256, compatibilidad por protocolo con `tc2 version --json`, canales y firma por etapas (hash y attestation en la 2.0; Authenticode y Developer ID al entrar en winget y Homebrew) y contrato del manifiesto de recursos (`tc2 doctor --recursos`: obligatorio, necesario, conveniente y estorba); Linux y macOS no verificados | TASK-014 | aceptada |
+| [F1-14](F1-14-distribucion-e-instalacion.md) | Distribución e instalación: CLI en Rust por la regla D1a (gana el 1.º y el 2.º, medidos, y el 4.º, un juicio sobre postject; sin el 4.º la regla no decide) frente a TypeScript con Node SEA (mismo comando, salida idéntica sobre las 404 tareas de ToolsaWeb y las 96 de OpenDockTrading; más rápido en caliente en los 3 repos y 32,1 veces más pequeño), TLS con el almacén del sistema, lanzador sin runtime con `cli.lock` y descarga comprobada por sha256, compatibilidad por protocolo con `tc2 version --json`, canales y firma por etapas (hash y attestation en la 2.0; Authenticode y Developer ID al entrar en winget y Homebrew) y contrato del manifiesto de recursos (`tc2 doctor --recursos`: obligatorio, necesario, conveniente y estorba); Linux y macOS no verificados | TASK-014 | aceptada |
 | [F1-18](F1-18-hito-de-decisiones.md) | Hito de decisiones (ADR) en `init` y en `actualizar`: estudio de adopción que lo dimensiona (bloque «uso medido» del informe de F1-12 y `proponer_hito` con un elemento por hueco, bloqueantes antes de migrar), proporcionalidad con mediana y p75 de la Fase 1 (45,5–50 min y 13,9–19,1 M tokens por ADR), registro `hito-decisiones.yml` en `taskcodetwo/tareas` cuya lista, descartes y «no necesario» firma una persona, con estado calculado que falla cerrado, y puerta determinista en `tc2 start` y en el check `estado` (código `rechazada`) | TASK-094 | aceptada |
 
 ## Plantilla
````

## Excluido del diff (20 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 36828cc4bd191733e0f3aa8fcc8b29c1c6f0ac7e..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-014/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-014/planificacion/plan-final.md                 |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-014/poc/especificacion-comando.md               |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-014/poc/poc-f1-14.log                           |  43 ++++--
 tareas/{02-en-curso => 03-en-revision}/TASK-014/poc/poc-f1-14.py                            | 134 ++++++++++++++++--
 tareas/{02-en-curso => 03-en-revision}/TASK-014/poc/poc-rust/Cargo.lock                     |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-014/poc/poc-rust/Cargo.toml                     |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-014/poc/poc-rust/DEPENDENCIAS.md                |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-014/poc/poc-rust/NOTAS.md                       |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-014/poc/poc-rust/src/main.rs                    |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-014/poc/poc-ts/DEPENDENCIAS.md                  |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-014/poc/poc-ts/NOTAS.md                         |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-014/poc/poc-ts/package-lock.json                |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-014/poc/poc-ts/package.json                     |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-014/poc/poc-ts/scripts/build.mjs                |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-014/poc/poc-ts/src/main.ts                      |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-014/poc/poc-ts/tsconfig.json                    |   0
 tareas/03-en-revision/TASK-014/revision/informe-revision-1.md                               |  80 +++++++++++
 tareas/03-en-revision/TASK-014/revision/peticion-revision-1.md                              | 747 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-014/tarea.md                                    |  10 +-
 20 files changed, 988 insertions(+), 26 deletions(-)
````
