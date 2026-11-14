# Peticion de revision — TASK-013 (ronda 2)

- Tarea: TASK-013 — F1-12 ADR actualizacion de proyectos entre versiones
- Rama revisada: feature/task-013-f1-12-adr-actualizacion-de-proyectos-ent
- Rama base: develop
- Commit revisado (HEAD): b706b33020e14c52351e27cd2167c8d2e42b016e
- Fecha: 2026-11-14
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-013 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde c12ec5ecd741fc30a825d846d728b1b952189e85 (el commit revisado en la ronda anterior)
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

- tareas/03-en-revision/TASK-013/tarea.md
- tareas/03-en-revision/TASK-013/planificacion/
- docs/adr/F1-12-actualizacion-entre-versiones.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 6 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | abierto | docs/adr/F1-12-actualizacion-entre-versiones.md:277 | informe-revision-1.md |

## Commits a revisar (git log c12ec5ecd741fc30a825d846d728b1b952189e85..HEAD)

````
b706b33 chore(TASK-013): informe de revision 1 y constancia de autorrevision ronda 2
3c44c18 docs(adr): F1-12 corrige los hallazgos de la revision 1
9625d85 chore(TASK-013): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff c12ec5ecd741fc30a825d846d728b1b952189e85..HEAD)

````diff
diff --git a/docs/adr/F1-12-actualizacion-entre-versiones.md b/docs/adr/F1-12-actualizacion-entre-versiones.md
index 1a276d8..5c0432b 100644
--- a/docs/adr/F1-12-actualizacion-entre-versiones.md
+++ b/docs/adr/F1-12-actualizacion-entre-versiones.md
@@ -44,7 +44,7 @@ deterministas** (ningún paso de la actualización depende de un hook).
 | TR2 | Push y, si algo falla, revertir en el remoto | Descartada: F1-5 prohíbe borrar o forzar la rama de estado y `develop` solo cambia por PR |
 | **CR1** | **Criterio estructural**: instantánea `{id → título, estado, dependencias, sprint, etiquetas}` antes y después, roadmap por estructura, `merge-base` vacío e historia por tarea | **Elegida**: detecta una sola dependencia borrada (PoC M2) |
 | CR2 | Comparar el `ROADMAP.md` byte a byte | Descartada: la 2.0 lo regenera con otro contenido (F1-1, decisión 3) |
-| **D1a** | Nombres `tc2` (CLI) y `taskcodetwo` (plugin, prefijo de skills, `.git/taskcodetwo/`, rama `taskcodetwo/tareas`, paquete npm) | **Elegida** (consigna «la opción recomendada»): `tc2` no está en el PATH; el paquete npm `tc2` está ocupado, `taskcodetwo` libre (PoC N) |
+| **D1a** | Nombres `tc2` (CLI) y `taskcodetwo` (plugin, prefijo de skills, `.git/taskcodetwo/`, rama `taskcodetwo/tareas`, paquete npm) | **Elegida** (consigna «la opción recomendada»; aprobados por Carlos el 2026-11-14): `tc2` no está en el PATH; el paquete npm `tc2` está ocupado, `taskcodetwo` libre (PoC N) |
 | D1b | `tctwo` | Descartada: libre en todo, pero no se lee como «TaskCode» y no gana nada frente a `tc2` |
 | D1c | `taskcodetwo` como CLI | Descartada: largo para escribirlo a diario |
 | D1d | `tc` | Descartada: `tc` es la orden de control de tráfico de iproute2 en Linux (no verificado en una distro con iproute2, ver «No verificado») y el paquete npm `tc` instala un binario `tc` (PoC N) |
@@ -77,7 +77,12 @@ python -I -B tareas/<estado>/TASK-013/poc/poc-f1-12.py \
 El guion lee la tabla `cifras` de este ADR y sale con 1 si una cifra no es
 la medida, si falta una medida o si sobra una fila; también comprueba el
 `resumen`, las secciones, los IDs citados, los enlaces y la fila del índice.
-Salida observada, recortada (la completa está en el log):
+Ocho copias mutadas del ADR (una cifra cambiada, una fila borrada, una fila
+sin medida, el tiempo de reloj metido en la tabla, un número inventado en
+el resumen, un enlace a `tareas/`, un ID que no existe y una sección
+borrada) tienen que fallar, cada una por su motivo. El tiempo de reloj no es
+una cifra comprobada: va a stderr. Salida observada, recortada (la completa
+está en el log):
 
 ### N — Nombres
 
@@ -104,7 +109,7 @@ commit fijo 8141a08a7: 96 tareas ; por carpeta 00..05: 50 / 0 / 0 / 0 / 44 / 2
 plan: tareas a mover=96 ; a descartada=2 (TASK-015, TASK-068) ; con motivo de ## Descartada=2 ; claves retiradas=192
 actualizar sin capacidad tareas-json: codigo 5 ; huecos bloqueantes: scripts/lector-tareas.py (1-44) ...
    ficheros cambiados por la negativa (con .git): 0
-subtree split: 1151 commits ; tiempo: menos de 600 s ; rama de estado tras aplanar y formato 2: 1153 commits
+subtree split: 1151 commits ; rama de estado tras aplanar y formato 2: 1153 commits
 criterio: tareas 96 de 96 ; frontmatter no YAML 0 ; con formato 2 96 ; diferencias 0 ; roadmap iguales (94 filas) ; merge-base ninguna
 historia por tarea: commits por ruta >= develop 96/96 ; git log --follow >= el de develop 96/96 ;
    --follow ya roto en develop (la 0.16 mueve y reescribe en el mismo commit): 3 ['TASK-019 1 de 11', 'TASK-020 1 de 11', 'TASK-080 5 de 6']
@@ -126,10 +131,11 @@ separado del formato 2. Sin la capacidad `tareas-json` la migración se niega
 tareas tras migrar. La PoC fuerza la migración para medir el resto.
 
 El `subtree split` recorre los 1350 commits de `develop` y deja 1151 en la
-rama de estado: tardó 594 s en la ejecución que deja el log y 907 s en una
-prueba anterior sobre otro clon (tiempos por stderr; el log solo da el
-tramo, que puede cambiar entre ejecuciones). Es el paso caro de la
-migración.
+rama de estado. Es el paso caro de la migración y su tiempo de reloj varía
+entre ejecuciones en la misma máquina, así que no entra en la tabla
+comprobada. Rango observado: 594 s en la entrega, 614 s en la reejecución
+de la revisión 1 y 647 s en la ejecución que deja el log de esta
+corrección (su stderr va junto al log, en `poc-f1-12.tiempos.log`).
 
 ### M2 — Vuelta atrás con el criterio roto, sobre una migración a medias
 
@@ -162,7 +168,7 @@ cada fichero de la copia, `.git` incluido, y la restaura.
 | `board` | 0, «No hay tareas» | 0, «No hay tareas» | 0 | 0 |
 | `doctor` | 1: `config`, `rama-principal`, `tareas-carpetas` | 1: `rama-principal`, `tareas-carpetas` y 18 avisos de claves desconocidas | 1, como B | 1, como B |
 | `estado --stdin` | 0, «Nada en marcha» | 0 | 0 | 0 |
-| `roadmap --escribir` | **1, no escribe** | **0, reescribe `tareas/ROADMAP.md`** | 1, no escribe | 1, no escribe |
+| `roadmap --escribir` | **1, no escribe** (el ROADMAP sigue con 94 filas de tarea) | **0, reescribe `tareas/ROADMAP.md`: de 94 filas de tarea a 0** | 1, no escribe | 1, no escribe |
 | `new` | **1, no escribe** (`Linea de config invalida (falta ":"): "---"`) | **1, pero escribe `tareas/00-planificadas/TASK-001/tarea.md`**: un `TASK-001` que ya existe en la carpeta fija | 1, no escribe | 1, no escribe |
 | hook `pre-tool-merge` | **2, bloquea** (la config no se puede leer) | 0, deja pasar | 2 | 2 |
 | hooks `pre-tool-lectura`, `stop-verificacion`, `coste-turno` | 0, sin escribir nada | 0 | 0 | 0 |
@@ -221,8 +227,9 @@ ocho campos de la 0.16 sin ningún valor propio en OpenDockTrading.
 
 Cada fila es una medida del guion, con la sección que la produce. El guion
 falla si un valor no es el medido, si falta una medida o si sobra una fila.
-El tiempo del `subtree split` es el de la ejecución que deja el log; las
-ejecuciones de prueba lo reutilizaron de una caché fuera de la copia.
+Solo entran valores que no dependen del reloj. Las ejecuciones de prueba
+reutilizaron el `subtree split` de una caché fuera de la copia; la del log,
+no.
 
 <!-- cifras:inicio -->
 | Cifra | Valor | Fuente |
@@ -274,7 +281,6 @@ ejecuciones de prueba lo reutilizaron de una caché fuera de la copia.
 | remoto: refs antes del criterio y tras el push | `1 -> 3` | M |
 | segunda pasada: pasos del plan y ficheros cambiados | `0 y 0` | M |
 | subtree split: commits de la rama de estado | `1151` | M |
-| subtree split: tiempo en OpenDockTrading | `menos de 600 s` | M |
 | reanudar: rama de estado a medias reutilizable | `si` | M2 |
 | vuelta atras: codigo con el criterio roto | `6` | M2 |
 | vuelta atras: diferencia detectada | `TASK-002: dependencias` | M2 |
@@ -313,6 +319,10 @@ ejecuciones de prueba lo reutilizaron de una caché fuera de la copia.
 | 0.16 roadmap --escribir, config B: codigo | `1` | C |
 | 0.16 roadmap --escribir, config C: codigo | `1` | C |
 | 0.16 roadmap --escribir, config D: codigo | `1` | C |
+| 0.16 roadmap --escribir, config A: filas de tarea del ROADMAP | `94 -> 0` | C |
+| 0.16 roadmap --escribir, config B: filas de tarea del ROADMAP | `94 -> 94` | C |
+| 0.16 roadmap --escribir, config C: filas de tarea del ROADMAP | `94 -> 94` | C |
+| 0.16 roadmap --escribir, config D: filas de tarea del ROADMAP | `94 -> 94` | C |
 | D2: ordenes de escritura de la 0.16 cerradas con la config A | `no` | C |
 | D2: ordenes de escritura de la 0.16 cerradas con la config B | `si` | C |
 | D2: ordenes de escritura de la 0.16 cerradas con la config C | `si` | C |
@@ -386,7 +396,7 @@ algo: sin `tareas-json` la migración se niega (código y ficheros cambiados:
    | `tareas/NN-estado/.gitkeep` | se borran (las carpetas de estado desaparecen) |
    | frontmatter de la 0.16 | formato 2 de F1-9: `formato: 2`, cadenas y fechas entre comillas dobles, `modelo_sugerido` → `roles.implementacion.modelo`, `skills_recomendados` y `regla_seleccion_skill` → `roles.implementacion`, `agente_revisor` → `roles.revision.agente`; fuera `revision_codex` y `ultimo_commit_revisado`; `carril` y `estimacion` a `null`; las claves desconocidas se conservan (F1-9 D4a). El cuerpo, byte a byte |
    | `tareas/ROADMAP.md` | se lleva tal cual a la raíz de la rama de estado; la 2.0 lo regenera con el contenido de F1-1 en su primera transición (F3-4) |
-   | `.taskcode/config.yml` plana | la misma ruta (D4a) con `---` en la primera línea, `formato: 2`, las claves base de F2-7 y los bloques de F1-11 (`verificacion`, `derivados`, `confianza_incluye`); los valores de la 0.16 pasan tal cual (`comando_verificacion` → `verificacion.comando` entre comillas simples, `timeout_verificacion` → `perfiles.completa.timeout`) y lo que la 0.16 no tenía lo declara el proyecto (F1-11, «valores a rellenar») |
+   | `.taskcode/config.yml` plana | la misma ruta (D4a) con `---` en la primera línea, `formato: 2`, las claves base de F2-7 y los bloques de F1-11 (`verificacion`, `derivados`, `confianza_incluye`); los valores de la 0.16 pasan tal cual (`comando_verificacion` → `verificacion.comando`, `timeout_verificacion` → `perfiles.completa.timeout`) y lo que la 0.16 no tenía lo declara el proyecto (F1-11, «valores a rellenar»). **Comillas**: las claves que la 0.16 también lee (`modo_flujo` y las demás claves base heredadas) van entre **comillas dobles**, como pide F1-9 para las cadenas y porque la 0.16 quita las dobles pero conserva las simples dentro del valor (variante D de la PoC C); las órdenes y rutas de los bloques nuevos, que la 0.16 no lee, van entre **simples** o con `/`, como fija F1-11 (decisión 1), para que el `\v` de una ruta de Windows no sea un escape |
    | `.gitignore` | añade `tareas/` |
    | `.git/taskcode/` (estado interno de la 0.16) | no migra: lo borra la desinstalación de la 0.16 (F9-10) |
 
@@ -463,8 +473,8 @@ algo: sin `tareas-json` la migración se niega (código y ficheros cambiados:
      desinstalar la 0.16 (F9-10); nunca con una tarea en curso; opcional y
      por proyecto.
 
-7. **Nombres definitivos** (D1a). Distintos de los de la 0.16 y los mismos
-   tras desinstalarla:
+7. **Nombres definitivos** (D1a), aprobados por Carlos el 2026-11-14.
+   Distintos de los de la 0.16 y los mismos tras desinstalarla:
 
    | Pieza | 0.16 | 2.0 |
    |---|---|---|
@@ -561,12 +571,22 @@ Las órdenes que esos ADR nombran para la 2.0 (`taskctl actualizar`,
    línea tiene `:`; solo avisa de claves desconocidas): aborta una línea sin
    `:` (`---` o una lista en bloque) o un valor inválido de una clave suya.
    Se elige `---`.
-7. **Nombres**: el paquete npm `tc2` está ocupado; el paquete es
-   `taskcodetwo` y el binario `tc2`. La config de la migración pone los
-   valores de la 0.16 entre comillas dobles: con simples, la 0.16 conserva
-   las comillas dentro del valor (variante D).
+7. **Nombres y comillas**: el paquete npm `tc2` está ocupado; el paquete
+   es `taskcodetwo` y el binario `tc2`. La config de la migración pone entre
+   comillas dobles los valores de las claves que la 0.16 también lee
+   (`modo_flujo`): con simples, la 0.16 conserva las comillas dentro del
+   valor y aborta (variante D). Las órdenes de los bloques nuevos siguen
+   entre simples (F1-11); ver la fila de la config en el punto 2.
 8. **El commit fijo** es `8141a08`, no el `f8a9b5f` de F1-11: los scripts
    del desglose no cambian entre los dos.
+9. **ToolsaWeb tiene 404 tareas**, no las 403 del criterio de F9-2 en el
+   plan: 404 en la PoC A de F1-9 y 404 en `develop` en el recuento de la
+   revisión 1 de esta tarea (234 planificadas, 161 terminadas, 9
+   archivadas). Se propone corregir el criterio de F9-2 a 404 junto con las
+   dependencias de D3; este ADR no cambia el plan.
+10. **El tiempo del `subtree split`** sale de la tabla comprobada
+    (revisión 1): varía entre ejecuciones y la puerta fallaba con datos
+    honestos al cruzar un tramo.
 
 ## Requisitos y evidencia
 
@@ -613,10 +633,6 @@ Las órdenes que esos ADR nombran para la 2.0 (`taskctl actualizar`,
 
 ## Preguntas abiertas
 
-- **D1 (nombre)**: `tc2` y `taskcodetwo` salen de la consigna «la opción
-  recomendada»; Carlos puede confirmarlos o cambiarlos antes de F1-14. Un
-  cambio solo toca la tabla del punto 7 y la nota de nombres.
-
 Este ADR se planificó con un solo rol (arquitectura). Lo que no cubrió:
 
 - **Riesgos**: corte a medias entre el push de la rama de estado y el de
@@ -633,6 +649,11 @@ Este ADR se planificó con un solo rol (arquitectura). Lo que no cubrió:
 
 ## No verificado
 
+- **La retirada de parches no se ejecuta en la PoC**: no se editan
+  `verificar.bat` ni los lectores de OpenDockTrading; solo se calcula su
+  alcance (`fichero:líneas` y el 91 / 973). Los únicos ficheros de código que
+  cambia la migración de la PoC son `.gitignore` y `.taskcode/config.yml`.
+
 - **El choque de `tc` con iproute2** en una distro Linux de uso normal: la
   única distro de WSL de esta máquina (`docker-desktop`) no trae iproute2.
 - **`tc2 actualizar --deshacer`** después del push (revert por PR y rama
````

## Excluido del diff (8 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff c12ec5ecd741fc30a825d846d728b1b952189e85..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-013/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-013/planificacion/plan-final.md                 |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-013/poc/poc-f1-12.log                           |  24 +++-
 tareas/{02-en-curso => 03-en-revision}/TASK-013/poc/poc-f1-12.py                            | 180 +++++++++++++++++--------
 tareas/03-en-revision/TASK-013/poc/poc-f1-12.tiempos.log                                    |   5 +
 tareas/03-en-revision/TASK-013/revision/informe-revision-1.md                               |  87 ++++++++++++
 tareas/03-en-revision/TASK-013/revision/peticion-revision-1.md                              | 733 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-013/tarea.md                                    |   8 +-
 8 files changed, 973 insertions(+), 64 deletions(-)
````
