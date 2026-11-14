# Peticion de revision — TASK-073 (ronda 2)

- Tarea: TASK-073 — F1-15 ADR seleccion de skills, agentes y modelo por tarea y rol
- Rama revisada: feature/task-073-f1-15-adr-seleccion-de-skills-agentes-y
- Rama base: develop
- Commit revisado (HEAD): 3a11b7d5f06ab04c7f968196e2ff98b1822451a3
- Fecha: 2026-11-14
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-073 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 2c541f98e2eb395180e4b1d28834ea517dcdcc67 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (10 hallazgo(s)))

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

- tareas/03-en-revision/TASK-073/tarea.md
- tareas/03-en-revision/TASK-073/planificacion/
- docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md
- docs/adr/README.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 7 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md:61 | informe-revision-1.md |
| IMP-2 | IMPORTANTE | corregido | tareas/03-en-revision/TASK-073/poc/poc-f1-15.py:1174 | informe-revision-1.md |
| IMP-3 | IMPORTANTE | corregido | docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md:341 | informe-revision-1.md |

## Commits a revisar (git log 2c541f98e2eb395180e4b1d28834ea517dcdcc67..HEAD)

````
3a11b7d chore(TASK-073): informe de revision 1 corregido en 806f024 y constancia de autorrevision ronda 2
806f024 docs(adr): F1-15 corrige los hallazgos de la revision 1
5768e3c chore(TASK-073): comprobacion a mano de Carlos sobre las 10 tareas (coinciden las 10)
f219e3f chore(TASK-073): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 2c541f98e2eb395180e4b1d28834ea517dcdcc67..HEAD)

````diff
diff --git a/docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md b/docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md
index a465da4..69c1040 100644
--- a/docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md
+++ b/docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md
@@ -4,16 +4,16 @@
 - Tarea: TASK-073
 - Plan: [F1-15](../plan/PLAN-TASKCODE-2.md) (dependencias F1-9 y F1-10), con el detalle de [la ampliación del 2026-11-14](../plan/AMPLIACION-2026-11-14.md), §1 a §4
 - Requisitos: RR-17, RF-12, RF-13, RF-14, RF-15 y RR-05 ([catálogo](../plan/REQUISITOS-FUNCIONALES.md)); en lo de equipo, RR-09 y RR-12
-- Decide: Carlos, con sus decisiones del 2026-11-14 (RR-17 imprescindible; elegir solo entre lo instalado y con reglas deterministas, nada «a ojo»; el número de roles tiene que mirar el dominio, no solo la complejidad; manifiesto de recursos de F1-14 sin duplicarlo; nombres `tc2` y `taskcodetwo`; no trocear) y la consigna «la opción recomendada» (D1a a D10a del plan de TASK-073). La columna «comprobado a mano» de las 10 tareas la marca Carlos (D10a): hoy está **pendiente de Carlos**.
+- Decide: Carlos, con sus decisiones del 2026-11-14 (RR-17 imprescindible; elegir solo entre lo instalado y con reglas deterministas, nada «a ojo»; el número de roles tiene que mirar el dominio, no solo la complejidad; manifiesto de recursos de F1-14 sin duplicarlo; nombres `tc2` y `taskcodetwo`; no trocear) y la consigna «la opción recomendada» (D1a a D10a del plan de TASK-073). La columna «comprobado a mano» de las 10 tareas la marcó Carlos (D10a) el 2026-11-14: «Coincide las 10», registrado en la sección «Decisiones de la persona» de `tarea.md` de TASK-073 (commit `5768e3c`).
 
 Este ADR fija **cómo se eligen**, para cada tarea y cada rol (diseño,
 implementación, revisión), los agentes, las skills y el modelo: una función
-pura, `seleccionar`, que lee tres tablas de este fichero (`tabla-seleccion`,
-`tabla-modelo` y `tabla-dominios`), el inventario de lo instalado y los
-metadatos que cada skill declara, y que nunca llama a un modelo. Antes de
-decidir se midió lo que pasó en la Fase 1: los 33 IMPORTANTES de la Fase 1 se
-clasificaron con una lista cerrada, y 16 de 33 IMPORTANTES son de
-puerta-seguridad, 12 de ellos en planes de un solo rol. Con esa medida el
+pura, `seleccionar`, que interpreta tres tablas de este fichero
+(`tabla-seleccion`, `tabla-modelo` y `tabla-dominios`), el inventario de lo
+instalado y los metadatos que cada skill declara, y que nunca llama a un
+modelo. Antes de decidir se midió lo que pasó en la Fase 1: los 33 IMPORTANTES
+de la Fase 1 se clasificaron con una lista cerrada, y 17 de 33 IMPORTANTES son
+de puerta-seguridad, 13 de ellos en planes de un solo rol. Con esa medida el
 número de roles de diseño pasa a mirar el dominio de la tarea.
 
 ## Contexto
@@ -58,7 +58,7 @@ solo toca `docs/` y el reparto por rutas no ve su dominio.
 | Decisión | Alternativa | Veredicto |
 |---|---|---|
 | **D1a** | Roles de diseño = máx(`roles_min` del carril de F1-10, tabla de complejidad, suelo por dominio), con suelo 2 si salta `puertas-seguridad` | **Elegida**: la medida de la Fase 1 dice que el dominio, no la complejidad, predice los IMPORTANTES de puerta |
-| D1b | Sumar puntos de dominio a la heurística | Descartada: «seguridad» ya suma 2 puntos y F1-13 y F1-18 salieron «simple»; los puntos no distinguen dominio |
+| D1b | Sumar puntos de dominio a la heurística | Descartada: la lista de palabras de riesgo de la 0.16 no tiene ninguna palabra de puertas (puerta, check, ruleset, runner, permiso…), y en la Fase 1 solo 1 de las 15 tareas de la Fase 1 contiene alguna palabra de riesgo de la 0.16 (F1-8, «seguridad»); por eso 14 salieron «simple». Sumar puntos exigiría añadir esas palabras a la heurística, y el número de roles seguiría dependiendo de cuántas aparecen, no de si el dominio está; el suelo de D1a lo decide la presencia del dominio |
 | D1c | Dos roles en toda funcionalidad | Descartada: sube el coste en las tareas sin dominio delicado, que en la Fase 1 no tuvieron IMPORTANTES de puerta |
 | **D2a** | Detector de dominio con lista cerrada en el binario que la configuración solo amplía | **Elegida** |
 | D2b | Que el dominio lo clasifique un modelo | Descartada: es elegir «a ojo» |
@@ -81,9 +81,9 @@ solo toca `docs/` y el reparto por rutas no ve su dominio.
 
 ## Pruebas de concepto
 
-Guion `tareas/02-en-curso/TASK-073/poc/poc-f1-15.py` (al cerrar la tarea,
-bajo `04-terminadas`), con su log `poc-f1-15.log` y la tabla
-`clasificacion-hallazgos.md` en la misma carpeta:
+Guion `tareas/<estado>/TASK-073/poc/poc-f1-15.py` (la carpeta de la tarea
+cambia con su estado; el guion se sitúa por su propia ruta), con su log
+`poc-f1-15.log` y la tabla `clasificacion-hallazgos.md` en la misma carpeta:
 
 ```text
 python -I -B poc-f1-15.py %LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-073\poc <repo-TaskCodeTwo> <carpeta-Personal>
@@ -124,24 +124,35 @@ puntos que anotó la 0.16 en la petición de plan en 15 de 15 tareas.
 | TASK-009 | F1-8 | 2 (arquitectura, riesgos) | 4 (media) | 4 | 4 | 0 | 0 | 0 | 0 | `code-quality-reviewer` |
 | TASK-010 | F1-9 | 1 (arquitectura) | 1 (simple) | 0 | 0 | 0 | 0 | 0 | 0 | `code-quality-reviewer` |
 | TASK-011 | F1-10 | 1 (arquitectura) | 1 (simple) | 2 | 2 | 0 | 0 | 0 | 0 | `code-quality-reviewer` |
-| TASK-012 | F1-11 | 1 (arquitectura) | 2 (simple) | 5 | 2 | 0 | 1 | 2 | 0 | `code-quality-reviewer` |
+| TASK-012 | F1-11 | 1 (arquitectura) | 2 (simple) | 5 | 3 | 0 | 0 | 2 | 0 | `code-quality-reviewer` |
 | TASK-013 | F1-12 | 1 (arquitectura) | 2 (simple) | 2 | 0 | 0 | 0 | 1 | 1 | `code-quality-reviewer` |
 | TASK-014 | F1-14 | 1 (arquitectura) | 1 (simple) | 1 | 0 | 0 | 0 | 0 | 1 | `code-quality-reviewer` |
 | TASK-015 | F1-13 | 1 (arquitectura) | 1 (simple) | 2 | 1 | 0 | 0 | 1 | 0 | `code-quality-reviewer` |
 | TASK-094 | F1-18 | 1 (arquitectura) | 2 (simple) | 3 | 1 | 1 | 1 | 0 | 0 | `code-quality-reviewer` |
 <!-- tabla-retro:fin -->
 
-Lectura: 16 de 33 IMPORTANTES son de puerta-seguridad, 12 de ellos en planes
-de un solo rol; por clase, el log da puerta-seguridad 16, contrato 3,
-correccion 6, fidelidad-evidencia 6 y reproducibilidad 2. Como solo F1-8 tuvo
-rol de riesgos, 12 IMPORTANTES de puerta-seguridad escaparon a un plan sin rol
-de riesgos, y F1-8, con él, salió aun así con cuatro: el rol de riesgos sube la
+Lectura: 17 de 33 IMPORTANTES son de puerta-seguridad, 13 de ellos en planes
+de un solo rol; por clase, el log da puerta-seguridad 17, contrato 3,
+correccion 5, fidelidad-evidencia 6 y reproducibilidad 2 (TASK-012 IMP-1 es un
+caso límite: lo diseñado no funcionaba y además un `comprueba` pasaba de
+bloquear a avisar; por la precedencia de la lista cuenta como puerta). Como
+solo F1-8 tuvo rol de riesgos, 13 IMPORTANTES de puerta-seguridad escaparon a
+un plan sin rol de riesgos, y F1-8, con él, salió aun así con cuatro: el rol de riesgos sube la
 probabilidad de verlos, no la garantiza, y por eso la decisión también actúa
 en la revisión (decisión 6). Además, 7 IMPORTANTES aparecieron en una ronda de
 verificación, 5 de puerta-seguridad: la regla de la 0.16 de bajar un escalón
 el modelo en esas rondas no se aplica en ese dominio (fila S13). La
-heurística dejó 14 tareas en «simple» y una (F1-8) en «media»; la revisora
-fue `code-quality-reviewer` en las 15.
+heurística dejó 14 tareas en «simple» y una (F1-8) en «media», porque solo
+1 de las 15 tareas de la Fase 1 contiene alguna palabra de riesgo de la 0.16
+(F1-8, «seguridad»; la lista no tiene ninguna palabra de puertas): sus puntos
+salen de dependencias y criterios. La revisora fue `code-quality-reviewer` en
+las 15. Y 14 de las 15 tareas de la Fase 1 tienen una fila de implementación
+hecha por un subagente en `## Coste` (todas menos F1-1), lo que fija qué hace
+la fila de implementación de la `tabla-modelo` (decisión 5).
+
+La PoC identifica cada hallazgo por su ID dentro de la tarea y no puede
+detectar un ID reutilizado para otro hallazgo; eso se comprobó leyendo los
+informes (los IDs nuevos de cada ronda van marcados «nuevo»).
 
 ### D — El detector `dominios` sobre la Fase 1 (antes de la decisión)
 
@@ -152,8 +163,13 @@ tareas): **recall 7/8 y precisión 7/9**. El falso negativo es F1-11 (sus
 IMPORTANTES de puerta hablan de derivados y del check `estado`, palabras que
 no están en la tabla); los dos falsos positivos son F1-4, por «tokens» en el
 sentido de coste, y F1-14, por «firma» de binarios, que es seguridad pero no
-tuvo IMPORTANTES. Las palabras son las que fijó el plan aprobado; no se
-ajustaron a la vista del resultado, y F8-6 las recalibra con más tareas.
+tuvo IMPORTANTES. Las palabras son las que fijó el plan aprobado y la PoC no
+las ajustó a la vista del resultado, pero **es una medida dentro de muestra**:
+el plan eligió esa lista después de una lectura preliminar de estos mismos 15
+informes, y algunas palabras (`ruleset`, `runner`, `amenaza`, `identidad`)
+salen de ellos. Recall y precisión dicen cómo encaja la tabla con los datos con
+que se escribió, no cómo generaliza; la validación fuera de muestra es de F8-6,
+con tareas nuevas.
 
 Con esos datos, el suelo subiría de 1 a 2 roles en 8 de 15 tareas (F1-4,
 F1-5, F1-6, F1-7, F1-10, F1-13, F1-14 y F1-18), más el unificador. El coste
@@ -182,7 +198,16 @@ integrado); las otras 86 están instaladas pero no declaradas, entre ellas la
 skill `security-review` y el agente `security-auditor`, que serían las
 candidatas naturales a revisora de `puertas-seguridad`.
 
-### Muestra de las 10 tareas (regla fijada antes de los resultados)
+### Muestra de las 10 tareas (regla reproducible)
+
+La muestra es una regla, no una lista: aplicada a los commits fijos de los
+pilotos da exactamente las 10 tareas de la tabla T, y la PoC lo comprueba (la
+sección G falla si la tabla tiene una tarea que la regla no da o le falta una
+que sí da). Se escribió antes de ejecutar T, pero **eso no lo prueba el
+historial**: la regla, la PoC y los resultados entraron en el mismo commit, y
+el plan aprobado (D9a) solo fijaba los estratos por su nombre, no el desempate
+por ID ni el estrato `un-dominio`. Quien dude del orden puede comprobar que la
+regla es la más simple que cabe (ID más alto por estrato) y aplicarla de nuevo.
 
 Por cada piloto, estratos en este orden; en cada estrato, la tarea terminada
 de **ID más alto** con merge en `develop` y algún fichero fuera de `tareas/`
@@ -211,24 +236,27 @@ Carril declarado como en F1-10 (`fix` si el tipo es `fix`, `funcionalidad`
 si no), mínimo por ficheros y efectivo con sus funciones; complejidad con la
 heurística de la 0.16 (el mayor entre la declarada y la calculada); la
 propuesta es la salida de `seleccionar` con el inventario y el catálogo de la
-sección I y sin configuración. «Comprobado a mano» lo marca Carlos (D10a).
+sección I y sin configuración. «Comprobado a mano» lo marcó Carlos (D10a): «Coincide las 10» (registro en
+`tarea.md` de TASK-073, «Decisiones de la persona», commit `5768e3c`). Las correcciones de la revisión 1
+no cambiaron ninguna de las 10 propuestas: la PoC recalcula cada fila con las tablas de este fichero y la
+compara con la tabla.
 
 <!-- tabla-muestra:inicio -->
 | Tarea | Estrato | Carril (declarado → efectivo) | Complejidad | Dominios (señal) | Diseño | Implementación | Revisión | Modelo D / I / R | Reglas D · I · R | Lo que usó la 0.16 | Comprobado a mano (Carlos) |
 |---|---|---|---|---|---|---|---|---|---|---|---|
-| ToolsaWeb/TASK-306 | skills-016 | funcionalidad → funcionalidad | media (3 p) | — | 2: arquitectura, riesgos + unificador | task-workflow | rust-wasm-reviewer | opus / sonnet / opus | M05 S02 S05 · M05 S07 · M05 S08 S12 | roles 2 (arquitectura, riesgos); rev rust-wasm-reviewer; modelo sonnet; skills task-workflow | pendiente de Carlos |
-| ToolsaWeb/TASK-399 | fix | fix → fix | simple (1 p) | — | 1: arquitectura | — | rust-wasm-reviewer, code-quality-reviewer | sonnet / sonnet / sonnet | M02 S02 · M02 · M02 S08 S12 | roles 1 (arquitectura); rev code-quality-reviewer; modelo sonnet; skills — | pendiente de Carlos |
-| ToolsaWeb/TASK-403 | riesgo-alto | funcionalidad → riesgo-alto | media (3 p) | puertas-seguridad (palabra:puerta) | 4: arquitectura, riesgos, testing, dominio + unificador | — | code-quality-reviewer; aviso: puertas-seguridad sin revisora | opus / opus / opus | M07 S02 S05 · M07 · M07 S09 S11 S12 | roles 2 (arquitectura, riesgos); rev code-quality-reviewer; modelo sonnet; skills — | pendiente de Carlos |
-| ToolsaWeb/TASK-299 | solo-docs | funcionalidad → funcionalidad | media (3 p) | — | 2: arquitectura, riesgos + unificador | task-workflow | code-quality-reviewer | opus / sonnet / opus | M05 S02 S05 · M05 S07 · M05 S09 S12 | roles 2 (arquitectura, riesgos); rev code-quality-reviewer; modelo sonnet; skills task-workflow | pendiente de Carlos |
-| ToolsaWeb/TASK-402 | multidominio | funcionalidad → funcionalidad | media (5 p) | puertas-seguridad (palabra:firma) | 2: arquitectura, riesgos + unificador | — | angular-vue-reviewer, rust-wasm-reviewer, code-quality-reviewer; aviso: puertas-seguridad sin revisora | opus / sonnet / opus | M05 S02 S05 · M05 · M05 S08 S11 S12 | roles 2 (arquitectura, riesgos); rev angular-vue-reviewer, code-quality-reviewer, rust-wasm-reviewer; modelo sonnet; skills — | pendiente de Carlos |
-| OpenDockTrading/TASK-063 | seguridad | funcionalidad → funcionalidad | media (5 p) | puertas-seguridad (etiqueta:seguridad) | 2: arquitectura, riesgos + unificador | — | angular-vue-reviewer, rust-wasm-reviewer, code-quality-reviewer; aviso: puertas-seguridad sin revisora | opus / sonnet / opus | M05 S02 S05 · M05 · M05 S08 S11 S12 | roles 2 (arquitectura, riesgos); rev angular-vue-reviewer, code-quality-reviewer, rust-wasm-reviewer; modelo sonnet; skills — | pendiente de Carlos |
-| OpenDockTrading/TASK-014 | riesgo-alto | funcionalidad → riesgo-alto | alta (6 p) | puertas-seguridad (palabra:check) | 4: arquitectura, riesgos, testing, dominio + unificador | — | rust-wasm-reviewer, code-quality-reviewer; aviso: puertas-seguridad sin revisora | opus / opus / opus | M07 S02 S05 · M07 · M07 S08 S11 S12 | roles 3 (arquitectura, riesgos, testing); rev code-quality-reviewer, rust-wasm-reviewer; modelo sonnet; skills — | pendiente de Carlos |
-| OpenDockTrading/TASK-067 | solo-docs | funcionalidad → funcionalidad | media (4 p) | — | 2: arquitectura, riesgos + unificador | — | code-quality-reviewer | opus / sonnet / opus | M05 S02 S05 · M05 · M05 S09 S12 | roles 2 (arquitectura, riesgos); rev general-purpose; modelo sonnet; skills — | pendiente de Carlos |
-| OpenDockTrading/TASK-069 | multidominio | funcionalidad → funcionalidad | simple (1 p) | — | 1: arquitectura | — | rust-wasm-reviewer, code-quality-reviewer | sonnet / sonnet / sonnet | M04 S02 · M04 · M04 S08 S12 | roles 1 (arquitectura); rev code-quality-reviewer, rust-wasm-reviewer; modelo sonnet; skills — | pendiente de Carlos |
-| OpenDockTrading/TASK-096 | un-dominio | funcionalidad → funcionalidad | simple (3 p) | puertas-seguridad (palabra:check) | 2: arquitectura, riesgos + unificador | — | code-quality-reviewer; aviso: puertas-seguridad sin revisora | sonnet / sonnet / sonnet | M04 S02 S03 S05 · M04 · M04 S09 S11 S12 | roles 1 (arquitectura); rev code-quality-reviewer; modelo sonnet; skills — | pendiente de Carlos |
+| ToolsaWeb/TASK-306 | skills-016 | funcionalidad → funcionalidad | media (3 p) | — | 2: arquitectura, riesgos + unificador | task-workflow | rust-wasm-reviewer | opus / sonnet / opus | M05 S02 S05 · M05 S07 · M05 S08 S12 | roles 2 (arquitectura, riesgos); rev rust-wasm-reviewer; modelo sonnet; skills task-workflow | coincide (Carlos, 2026-11-14) |
+| ToolsaWeb/TASK-399 | fix | fix → fix | simple (1 p) | — | 1: arquitectura | — | rust-wasm-reviewer, code-quality-reviewer | sonnet / sonnet / sonnet | M02 S02 · M02 · M02 S08 S12 | roles 1 (arquitectura); rev code-quality-reviewer; modelo sonnet; skills — | coincide (Carlos, 2026-11-14) |
+| ToolsaWeb/TASK-403 | riesgo-alto | funcionalidad → riesgo-alto | media (3 p) | puertas-seguridad (palabra:puerta) | 4: arquitectura, riesgos, testing, dominio + unificador | — | code-quality-reviewer; aviso: puertas-seguridad sin revisora | opus / opus / opus | M07 S02 S05 · M07 · M07 S09 S11 S12 | roles 2 (arquitectura, riesgos); rev code-quality-reviewer; modelo sonnet; skills — | coincide (Carlos, 2026-11-14) |
+| ToolsaWeb/TASK-299 | solo-docs | funcionalidad → funcionalidad | media (3 p) | — | 2: arquitectura, riesgos + unificador | task-workflow | code-quality-reviewer | opus / sonnet / opus | M05 S02 S05 · M05 S07 · M05 S09 S12 | roles 2 (arquitectura, riesgos); rev code-quality-reviewer; modelo sonnet; skills task-workflow | coincide (Carlos, 2026-11-14) |
+| ToolsaWeb/TASK-402 | multidominio | funcionalidad → funcionalidad | media (5 p) | puertas-seguridad (palabra:firma) | 2: arquitectura, riesgos + unificador | — | angular-vue-reviewer, rust-wasm-reviewer, code-quality-reviewer; aviso: puertas-seguridad sin revisora | opus / sonnet / opus | M05 S02 S05 · M05 · M05 S08 S11 S12 | roles 2 (arquitectura, riesgos); rev angular-vue-reviewer, code-quality-reviewer, rust-wasm-reviewer; modelo sonnet; skills — | coincide (Carlos, 2026-11-14) |
+| OpenDockTrading/TASK-063 | seguridad | funcionalidad → funcionalidad | media (5 p) | puertas-seguridad (etiqueta:seguridad) | 2: arquitectura, riesgos + unificador | — | angular-vue-reviewer, rust-wasm-reviewer, code-quality-reviewer; aviso: puertas-seguridad sin revisora | opus / sonnet / opus | M05 S02 S05 · M05 · M05 S08 S11 S12 | roles 2 (arquitectura, riesgos); rev angular-vue-reviewer, code-quality-reviewer, rust-wasm-reviewer; modelo sonnet; skills — | coincide (Carlos, 2026-11-14) |
+| OpenDockTrading/TASK-014 | riesgo-alto | funcionalidad → riesgo-alto | alta (6 p) | puertas-seguridad (palabra:check) | 4: arquitectura, riesgos, testing, dominio + unificador | — | rust-wasm-reviewer, code-quality-reviewer; aviso: puertas-seguridad sin revisora | opus / opus / opus | M07 S02 S05 · M07 · M07 S08 S11 S12 | roles 3 (arquitectura, riesgos, testing); rev code-quality-reviewer, rust-wasm-reviewer; modelo sonnet; skills — | coincide (Carlos, 2026-11-14) |
+| OpenDockTrading/TASK-067 | solo-docs | funcionalidad → funcionalidad | media (4 p) | — | 2: arquitectura, riesgos + unificador | — | code-quality-reviewer | opus / sonnet / opus | M05 S02 S05 · M05 · M05 S09 S12 | roles 2 (arquitectura, riesgos); rev general-purpose; modelo sonnet; skills — | coincide (Carlos, 2026-11-14) |
+| OpenDockTrading/TASK-069 | multidominio | funcionalidad → funcionalidad | simple (1 p) | — | 1: arquitectura | — | rust-wasm-reviewer, code-quality-reviewer | sonnet / sonnet / sonnet | M04 S02 · M04 · M04 S08 S12 | roles 1 (arquitectura); rev code-quality-reviewer, rust-wasm-reviewer; modelo sonnet; skills — | coincide (Carlos, 2026-11-14) |
+| OpenDockTrading/TASK-096 | un-dominio | funcionalidad → funcionalidad | simple (3 p) | puertas-seguridad (palabra:check) | 2: arquitectura, riesgos + unificador | — | code-quality-reviewer; aviso: puertas-seguridad sin revisora | sonnet / sonnet / sonnet | M04 S02 S03 S05 · M04 · M04 S09 S11 S12 | roles 1 (arquitectura); rev code-quality-reviewer; modelo sonnet; skills — | coincide (Carlos, 2026-11-14) |
 <!-- tabla-muestra:fin -->
 
-Lectura, sin juicio de persona todavía (es aritmética sobre la tabla): el
+Lectura (aritmética sobre la tabla; el juicio es el de Carlos, que dice que coinciden las diez): el
 número de roles de diseño propuesto coincide con el que lanzó la 0.16 en siete
 de las diez filas; sube en las dos de riesgo alto (todos, por F1-10) y en
 TASK-096 de OpenDockTrading, donde el suelo por dominio (S03) pone el rol de
@@ -241,10 +269,20 @@ skill, así que no se pueden comparar. El modelo deja de ser `sonnet` para todo:
 instalado y declarado hoy, ninguna revisora declara `puertas-seguridad`.
 Ninguna fila de la muestra cae en un empate.
 
-### S — `seleccionar` es determinista y no sale de la máquina
-
-- Las 10 tareas × 3 roles dos veces: mismo hash (`539d5857d7cd`); con el
-  inventario y el catálogo permutados, el mismo hash.
+### S — `seleccionar` interpreta la tabla, es determinista y no sale de la máquina
+
+- `seleccionar` no lleva las reglas escritas aparte: **lee `tabla-seleccion`**
+  de este fichero y aplica cada fila en su orden; la columna «Condición» es una
+  lista de átomos de un vocabulario cerrado y «Acción» un token de otro
+  (decisión 5). Una condición o acción fuera del vocabulario es un error.
+- Un caso de control por cada fila S01 a S15, más cuatro de S13 (ronda 2 sin
+  dominio baja de `opus` a `sonnet`; en riesgo alto, con `puertas-seguridad`
+  por el texto y con `puertas-seguridad` por la ruta `.claude/settings.json`
+  no baja). S01, S04, S06 y S10, que no salen en la muestra, tienen el suyo
+  (S06 y S10 con una skill y una revisora sintéticas que declaran
+  `puertas-seguridad`). La PoC falla si una fila no la ejercita ningún caso.
+- Las 10 tareas × 3 roles dos veces: mismo hash; con el inventario y el
+  catálogo permutados, el mismo hash.
 - Quitar `angular-vue-reviewer` del inventario la saca de las 2 selecciones
   que la tenían, con aviso `no-instalado` y fila S14.
 - Empate total (tres skills de ejecución con el mismo solape y la misma
@@ -262,20 +300,40 @@ Ninguna fila de la muestra cae en un empate.
 - El modelo cambia con el carril (funcionalidad/media frente a
   riesgo-alto/media) y con la complejidad (funcionalidad/simple frente a
   funcionalidad/media) en filas reales de la muestra: el criterio de F4-6.
+- **Evasión por vocabulario** (el mismo trabajo con otras palabras; decisión
+  3, «Texto de la tarea»). Lo que la PoC reproduce y su resultado esperado,
+  fijado en el guion:
+
+  | Caso | Resultado | ¿Lo contiene? |
+  |---|---|---|
+  | 1. `plan`, texto con «puerta», «check», «token», «aprobación» | 2 roles (suelo) | sí |
+  | 2. `plan`, mismo trabajo escrito con «validación», «credencial», «conformidad» | 1 rol, sin dominio | **no el binario**: solo la persona en `approve`, que ve «dominios: ninguno» y el suelo calculado |
+  | 3. revisión ronda 2, texto aprobado con puertas y texto vivo reescrito | sigue `opus` y el aviso | sí: el texto aprobado suma |
+  | 4. revisión ronda 2, otro vocabulario y el diff toca `.claude/settings.json` | sigue `opus` y el aviso | sí: la ruta, con la configuración de la base |
+  | 5. carril `fix` (sin `approve`), otro vocabulario y diff solo en `src/` | sin dominio, sin aviso, S13 baja el modelo | **no**: riesgo aceptado |
+
+- **Catálogo del árbol del PR** (decisión 2): un PR que añade una genérica
+  propia no la cuela (la genérica solo puede ser la del plugin, y la función
+  lo impone); un PR que añade una revisora con patrones `**/*.rs` sí entraría
+  si su catálogo se usara: la función no lo puede impedir, por eso el
+  inventario y el catálogo de la revisión se leen siempre de la base.
 
 ### G — Este ADR como dato
 
 El guion lee las tres tablas de la decisión, la muestra y la retrospectiva de
 este fichero y sale con 1 si una fila no tiene id o lo repite, un modelo no
 está en la escala, una combinación de carril y complejidad tiene cero o dos
-filas de modelo, un dominio no es de la lista cerrada, una regla da un id que
-no está en el inventario, una fila de la muestra o de la retrospectiva no es
-la calculada, la columna «a mano» no dice «pendiente de Carlos» ni lleva
-persona y fecha, falta una sección, una regla obligatoria, una divergencia o
+filas de modelo, un dominio no es de la lista cerrada, una condición o una
+acción de `tabla-seleccion` no es del vocabulario, un caso de control da otra
+cosa con las tablas del fichero, una regla da un id que no está en el
+inventario, una fila de la muestra (recalculada con las tablas del fichero) o
+de la retrospectiva no es la calculada, la columna «a mano» no lleva persona y
+fecha, falta una sección, una regla obligatoria, una divergencia o
 un «no verificado», un ID del plan o de los requisitos no existe, un enlace no
 resuelve o una cifra de la prosa (IMPORTANTES, recall, precisión) está fuera
-de la frase que dice a qué corresponde. Veinte copias mutadas fallan cada una
-por su motivo.
+de la frase que dice a qué corresponde. Veinticinco copias mutadas fallan cada
+una por su motivo, entre ellas S13 cambiada a «siempre», el suelo de S03 a 3 y
+la acción de S10 cambiada por la genérica.
 
 ## Cómo se ve en la tarea y en GitHub
 
@@ -293,6 +351,7 @@ roles:
     regla: ["M04", "S02", "S03", "S05"]
     senales: ["puertas-seguridad:palabra:check"]
     avisos: []
+    texto: "<blob de tarea.md en el commit firmado de approve>"
     inventario: "2ecd98236392"
     catalogo: "07d5ec762090"
   implementacion: {modelo: "sonnet", agentes: [], skills: [], regla: ["M04"], senales: [...], avisos: [], inventario: "...", catalogo: "..."}
@@ -302,13 +361,16 @@ roles:
 En GitHub no hay nada nuevo que configurar: el revisor en CI (F6-3) lee
 `roles.revision` calculado con la configuración de la base, nunca la del
 worktree del PR (decisión 3 de F1-10), y el check `revision-ia` deja en su
-resumen la fila y los avisos. `tc2 doctor --recursos` lista lo que la
+resumen la fila, las señales y los avisos. En `approve`, `tc2` muestra a la
+persona los dominios detectados con su señal y el número de roles que salen
+(«dominios: ninguno; roles 1» también se muestra), antes de que firme.
+`tc2 doctor --recursos` lista lo que la
 selección de cada tarea en curso necesita y no está instalado.
 
 ## Decisión
 
-1. **Una función pura, tres tablas.** Sin E/S, en el núcleo del binario de
-   `tc2`:
+1. **Una función pura que interpreta tres tablas.** Sin E/S, en el núcleo
+   del binario de `tc2`:
 
    ```text
    seleccionar(rol: Diseno|Implementacion|Revision, t: TareaLeida, carril: Carril, cx: Complejidad,
@@ -322,7 +384,18 @@ selección de cada tarea en curso necesita y no está instalado.
    declarada y la de la heurística (la de la 0.16, sin cambios). La salida va
    ordenada; dos llamadas con la misma entrada, en cualquier orden de
    inventario y catálogo, dan lo mismo. **Nunca llama a un modelo ni a la
-   red**: un empate se resuelve por id lexicográfico con aviso. La usan `plan`
+   red**: un empate se resuelve por id lexicográfico con aviso. Las reglas no
+   están escritas en el código aparte de la tabla: `seleccionar` recorre
+   `tabla-seleccion` en su orden y, para las filas de su rol y las de `todos`,
+   evalúa la «Condición» (átomos de un vocabulario cerrado unidos por «y», cada
+   uno con «no» delante si se niega: `siempre`, `carril=…`, `carril!=…`,
+   `dominio=…`, `roles>=n`, `ronda>=n`, `hay-skill-diseno-del-dominio`,
+   `hay-skill-ejecucion-con-solape`, `rutas-casan-entre-1-y-umbral`,
+   `rutas-no-casan-o-pasan-umbral`, `dominio-con-revisora`,
+   `dominio-sin-revisora`, `hay-no-instalado`, `modelo-no-permitido`) y aplica
+   la «Acción» (un token de otro vocabulario cerrado). Lo que no es del
+   vocabulario es un error al leer la tabla. Una fila se anota si su condición
+   se cumple y su acción cambia algo. La usan `plan`
    (F4-6, F4-1), `review` y el revisor en CI (F6-3), el orquestador (F4-4),
    `lote` (F1-16) y `doctor` (F3-8, F3-9).
 
@@ -330,7 +403,13 @@ selección de cada tarea en curso necesita y no está instalado.
    fuente de lo que existe: skills y agentes del plugin `taskcodetwo`, de otros
    plugins instalados, de `~/.claude/skills` y `~/.claude/agents`, de
    `.claude/skills` y `.claude/agents` del proyecto, y los agentes integrados
-   de la plataforma (`general-purpose`). El **catálogo** son los metadatos de
+   de la plataforma (`general-purpose`). **Árbol del proyecto**: en `plan`, el
+   del commit en que se calcula (anotado por hash); en `review` y en el revisor
+   en CI, siempre el de la base del PR (`develop`), leído con `git show` de la
+   base, nunca el worktree del PR, y lo mismo para la configuración. La PoC S
+   muestra por qué: una revisora que el PR añade a su árbol entraría en la
+   selección si se leyera de allí; la genérica, en cambio, solo puede ser la del
+   plugin y la función lo impone. El **catálogo** son los metadatos de
    selección de cada entrada: los del bloque ```` ```yaml ```` de la propia
    skill (decisión 8) o, para una skill o un agente ajeno, los que declara el
    bloque `recursos:` del proyecto ([F1-14](F1-14-distribucion-e-instalacion.md)).
@@ -346,6 +425,33 @@ selección de cada tarea en curso necesita y no está instalado.
    una palabra del objetivo o de los criterios, sin tildes ni mayúsculas.
    Ruta: glob sobre los dos nombres de cada renombrado.
 
+   **Texto de la tarea: no confiable, solo sube.** Objetivo, criterios y
+   etiquetas los escribe el agente en `tarea.md`, como el `tipo` que F1-10 ya
+   trató como no confiable. Con otras palabras para el mismo trabajo
+   desaparecerían el suelo, el aviso `dominio-sin-revisora` y la excepción de
+   S13. Cómo se contiene:
+
+   - **Hasta `approve`** el binario no lo puede contener: el texto es lo único
+     que hay. Lo contiene la persona: `approve` le muestra los dominios
+     detectados con su señal y el número de roles («dominios: ninguno; roles
+     1» incluido), y ella firma eso; puede subirlo con la etiqueta `puertas`
+     (D02) o con más roles. Riesgo aceptado.
+   - **Desde `approve`**, el texto aprobado queda congelado con la firma (su
+     blob se anota en `roles.<rol>.texto`) y `dominios()` usa la unión de las
+     señales del texto aprobado y del vivo: reescribir `tarea.md` después solo
+     puede añadir dominios, nunca quitarlos.
+   - **En la revisión**, las rutas del diff se evalúan en cada ronda con la
+     configuración de la base: un cambio en `.github/`, `.taskcode/`,
+     `CODEOWNERS` o `.claude/` da `puertas-seguridad` diga lo que diga el texto,
+     y S13 no baja el modelo si hay `puertas-seguridad` por cualquier señal o el
+     carril es `riesgo-alto`.
+   - **Lo que queda fuera**: una tarea de `fix` o de `errata` no pasa por
+     `approve`, así que si su texto evita las palabras y su diff no toca esas
+     rutas, no hay dominio, ni aviso, y S13 baja el modelo (caso 5 de la PoC
+     S). Lo contiene solo la persona que aprueba el PR (`merge-pr`), que ve en
+     el check las señales y los avisos. Riesgo aceptado, medido por F8-6 como
+     escapados.
+
    <!-- tabla-dominios:inicio -->
    | Id | Dominio | Señal | Valor |
    |---|---|---|---|
@@ -389,42 +495,54 @@ selección de cada tarea en curso necesita y no está instalado.
 5. **Reglas de selección.** Cada fila que aplica queda anotada por su id.
 
    <!-- tabla-seleccion:inicio -->
-   | Id | Rol | Cuándo | Qué da | Ids del inventario |
-   |---|---|---|---|---|
-   | S01 | `diseno` | carril `errata` | nada: sin plan | — |
-   | S02 | `diseno` | cualquier otro carril | roles = máx(`roles_min` del carril, tabla de complejidad); un agente por rol en el orden arquitectura, riesgos, testing, dominio | `taskcode-plugin:brainstorm-arquitectura`, `taskcode-plugin:brainstorm-riesgos`, `taskcode-plugin:brainstorm-testing`, `taskcode-plugin:brainstorm-dominio` |
-   | S03 | `diseno` | `dominios()` da `puertas-seguridad` | suelo de 2 roles (arquitectura y riesgos) | `taskcode-plugin:brainstorm-arquitectura`, `taskcode-plugin:brainstorm-riesgos` |
-   | S04 | `diseno` | `dominios()` da `rendimiento` | más el rol de rendimiento | `taskcode-plugin:brainstorm-rendimiento` |
-   | S05 | `diseno` | dos o más roles | más el unificador | `taskcode-plugin:brainstorm-unificador` |
-   | S06 | `diseno` | una skill declarada con `rol: diseno` y un dominio de `dominios()` | esa skill | — |
-   | S07 | `implementacion` | skills declaradas con `rol: ejecucion` con solape de etiquetas o dominios | una: mayor solape, después mayor prioridad, después menor id, con aviso si hubo empate | `taskcode-plugin:task-workflow` |
-   | S08 | `revision` | las rutas del diff (los dos nombres de cada renombrado) casan con los patrones de 1 a `umbral_dominios` revisoras | esas revisoras, más la genérica si queda algún fichero sin dominio | `taskcode-plugin:angular-vue-reviewer`, `taskcode-plugin:csharp-autocad-ifc-reviewer`, `taskcode-plugin:java-spring-reviewer`, `taskcode-plugin:rust-wasm-reviewer` |
-   | S09 | `revision` | ninguna revisora casa, o casan más de `umbral_dominios` | la genérica | `taskcode-plugin:code-quality-reviewer` |
-   | S10 | `revision` | un dominio de `dominios()` con revisora declarada e instalada | esa revisora, aunque las rutas no la pidan | — |
-   | S11 | `revision` | un dominio de `dominios()` sin revisora declarada | aviso `dominio-sin-revisora`, anotado y medido | — |
-   | S12 | `revision` | siempre | el agente que lanza cada revisora | `general-purpose` |
-   | S13 | `revision` | ronda de verificación (2 o más), salvo carril `riesgo-alto` o dominio `puertas-seguridad` | el modelo baja un escalón | — |
-   | S14 | `todos` | lo elegido no está en el inventario | se quita, con aviso `no-instalado` | — |
-   | S15 | `todos` | `modelos_permitidos` no admite el modelo de la fila | el permitido más cercano por debajo (si no hay, por encima), con aviso | — |
+   | Id | Rol | Condición | Acción | Ids del inventario | Qué significa |
+   |---|---|---|---|---|---|
+   | S01 | `diseno` | `carril=errata` | `sin-plan` | — | sin plan: no se lanza ningún rol |
+   | S02 | `diseno` | `carril!=errata` | `roles=max(roles_min,complejidad)` | `taskcode-plugin:brainstorm-arquitectura`, `taskcode-plugin:brainstorm-riesgos`, `taskcode-plugin:brainstorm-testing`, `taskcode-plugin:brainstorm-dominio` | número de roles = máx(`roles_min` del carril, tabla de complejidad); un agente por rol en el orden arquitectura, riesgos, testing, dominio |
+   | S03 | `diseno` | `dominio=puertas-seguridad` | `suelo=2` | `taskcode-plugin:brainstorm-arquitectura`, `taskcode-plugin:brainstorm-riesgos` | suelo de 2 roles: arquitectura y riesgos |
+   | S04 | `diseno` | `dominio=rendimiento` | `mas-rol=rendimiento` | `taskcode-plugin:brainstorm-rendimiento` | más el rol de rendimiento |
+   | S05 | `diseno` | `roles>=2` | `mas-rol=unificador` | `taskcode-plugin:brainstorm-unificador` | con dos o más roles, más el unificador |
+   | S06 | `diseno` | `hay-skill-diseno-del-dominio` | `skills-diseno-del-dominio` | — | las skills declaradas con `rol: diseno` y un dominio detectado |
+   | S07 | `implementacion` | `hay-skill-ejecucion-con-solape` | `skill-ejecucion=solape,prioridad,id` | `taskcode-plugin:task-workflow` | una skill de `rol: ejecucion`: mayor solape de etiquetas o dominios, después mayor prioridad, después menor id, con aviso si hubo empate |
+   | S08 | `revision` | `rutas-casan-entre-1-y-umbral` | `revisoras-de-rutas+generica-si-sueltas` | `taskcode-plugin:angular-vue-reviewer`, `taskcode-plugin:csharp-autocad-ifc-reviewer`, `taskcode-plugin:java-spring-reviewer`, `taskcode-plugin:rust-wasm-reviewer` | las revisoras cuyos patrones casan con las rutas del diff (los dos nombres de cada renombrado), más la genérica si queda algún fichero sin dominio |
+   | S09 | `revision` | `rutas-no-casan-o-pasan-umbral` | `generica-del-plugin` | `taskcode-plugin:code-quality-reviewer` | ninguna casa, o casan más de `umbral_dominios`: la genérica, que solo puede ser la del plugin |
+   | S10 | `revision` | `dominio-con-revisora` | `revisora-del-dominio` | — | por cada dominio detectado con revisora declarada e instalada, esa revisora, aunque las rutas no la pidan |
+   | S11 | `revision` | `dominio-sin-revisora` | `aviso=dominio-sin-revisora` | — | por cada dominio detectado sin revisora declarada, el aviso, anotado y medido |
+   | S12 | `revision` | `siempre` | `lanzador` | `general-purpose` | el agente que lanza cada revisora |
+   | S13 | `revision` | `ronda>=2` y no `carril=riesgo-alto` y no `dominio=puertas-seguridad` | `modelo-baja=1` | — | ronda de verificación fuera de riesgo alto y de puertas: el modelo baja un escalón |
+   | S14 | `todos` | `hay-no-instalado` | `quitar-no-instalado` | — | lo elegido que no está en el inventario se quita, con aviso `no-instalado` |
+   | S15 | `todos` | `modelo-no-permitido` | `restringir-modelo` | — | `modelos_permitidos` no admite el modelo: el permitido más cercano por debajo (si no hay, por encima), con aviso |
    <!-- tabla-seleccion:fin -->
 
    Los ids de la última columna son los del inventario de hoy (plugin 0.16.1);
    en la 2.0 llevan el prefijo del plugin `taskcodetwo` y la comprobación de
-   la PoC G es la misma. La implementación no lanza subagentes: la hace la
-   sesión principal, por eso no recibe agentes.
+   la PoC G es la misma.
+
+   **El agente de implementación no lo elige `seleccionar`.** En la Fase 1, 14
+   de las 15 tareas tienen en `## Coste` una fila de implementación hecha por
+   un subagente: la implementación la hace un agente que lanza quien orquesta,
+   y en el modo automático (F4-4) será igual. Ese agente es siempre el integrado
+   `general-purpose` (no hay candidatos entre los que elegir), así que
+   `seleccionar` le da el modelo (columna «Implementación» de la
+   `tabla-modelo`) y las skills (S07), y quien lo lanza anota su id en
+   `roles.implementacion.agentes`. Si la implementación la hace la sesión de la
+   persona, el modelo de la fila es un consejo, no se impone.
 
 6. **Revisoras por dominio (D6a).** La revisión suma dos caminos: por rutas
    (S08 y S09, el reparto de la 0.16 con los dos nombres de cada renombrado) y
    por dominio (S10): si `dominios()` da un dominio y hay una revisora
    instalada que lo declara en sus metadatos, entra aunque las rutas no la
-   pidan; si no la hay, va la genérica y la tarea lleva el aviso
+   pidan; si no la hay, va la genérica (siempre la del plugin: una skill del
+   proyecto con `fallback: true` no la sustituye) y la tarea lleva el aviso
    `dominio-sin-revisora` (S11), que F8-6 cuenta. No se inventa una revisora:
    un proyecto que quiera la de puertas declara la suya (por ejemplo, la skill
    `security-review` que ya está instalada) en `recursos:` con
    `rol: revisor` y `dominios: [puertas-seguridad]`. La selección de revisión
    se calcula siempre con la configuración de la base y las skills de la base,
    nunca con las del worktree del PR (decisión 3 de F1-10): un PR no puede
-   quitarse su revisora.
+   quitarse su revisora ni ponerse una. La PoC solo lo comprueba en la
+   función (catálogo de la base frente al del PR); que el revisor en CI lea
+   de verdad la base es de F6-3.
 
 7. **Modelo (D4a).** Escala `haiku` < `sonnet` < `opus`. Una fila por carril y
    complejidad; cada combinación tiene exactamente una (la PoC G lo
@@ -457,21 +575,25 @@ selección de cada tarea en curso necesita y no está instalado.
    agentes ajenos declaran lo mismo en `recursos:` (F1-14).
 
 9. **Cuándo se calcula (D5a).** Diseño e implementación, en `plan`; quedan
-   congelados con el `approve` firmado y `start` no los recalcula (si el
-   inventario ya no tiene algo, `start` para con el arreglo). Revisión, en cada
-   ronda de `review`, con el diff real y la configuración de la base.
+   congelados con el `approve` firmado, junto con el texto de la tarea que los
+   produjo, y `start` no los recalcula (si el inventario ya no tiene algo,
+   `start` para con el arreglo). Revisión, en cada ronda de `review`, con el
+   diff real, la configuración y el inventario de la base y la unión de las
+   señales del texto aprobado y del vivo.
 
 10. **Anotación en `roles.<rol>` (D8a).** Por rol: `modelo`, `agentes` (lista),
     `skills`, `regla` (lista de ids de fila de las tres tablas), `senales`
-    (dominio y señal), `avisos`, y el hash del inventario y del catálogo con los
-    que se calculó; en revisión, además, `ronda`. Con esos datos cualquiera
+    (dominio y señal), `avisos`, el blob del texto de la tarea aprobado
+    (`texto`) y el hash del inventario y del catálogo con los que se calculó;
+    en revisión, además, `ronda`. Con esos datos cualquiera
     reproduce la selección con `seleccionar`.
 
 11. **«Necesario para el proyecto» (F1-14).** Lo que `seleccionar` elige para
     una tarea es lo necesario para esa tarea: `tc2 doctor --recursos` lo lista
     y `start` para antes de tomarla si falta. Las skills de fase y los agentes
     de brainstorm del plugin son «obligatorio»; engram sigue siendo
-    «conveniente». Este ADR no duplica `evaluar_recursos`: le da la lista.
+    «conveniente». Este ADR no duplica `evaluar_recursos`: le da la lista. Es
+    una ampliación de F1-14 (divergencia 9).
 
 12. **Medición (D7a, RF-15).** La tabla de hallazgos de cada informe de
     revisión gana la columna **«Clase»** con la lista cerrada de la PoC R
@@ -488,7 +610,7 @@ selección de cada tarea en curso necesita y no está instalado.
 1. **F1-9, mapa `roles`**: la plantilla tiene `roles.revision.agente`
    (singular) y `regla` como texto; pasa a `agentes` (lista) en los tres roles
    y `regla` a lista de ids de fila, y se añaden `senales`, `avisos`,
-   `inventario`, `catalogo` y, en revisión, `ronda`. F2-5 lo recoge.
+   `inventario`, `catalogo`, `texto` y, en revisión, `ronda`. F2-5 lo recoge.
 2. **F1-11, bloque yaml (D5a de F1-11)**: era solo de revisoras; se amplía a
    toda skill y agente del plugin con `rol`, `etiquetas`, `dominios` y
    `prioridad`, y `catalogo-skills.yml` desaparece.
@@ -505,18 +627,33 @@ selección de cada tarea en curso necesita y no está instalado.
    (`plan-desempate-skill.ts`): el empate se resuelve por id lexicográfico con
    aviso (D3a). No se migra.
 6. **Cifras preliminares del plan**: el plan leía 15 hallazgos de puerta o
-   seguridad, 11 de ellos en planes de un rol, con TASK-012 en uno;
-   la clasificación con lista cerrada y cita da dos en TASK-012 (IMP-2, un
-   derivado que cambia el estado de una tarea, e IMP-5, un check obligatorio
-   que el PR deja verde), y con ellos las cifras de puerta-seguridad de la PoC
-   R. El plan decía «como mucho 8 de 15» tareas con suelo; son 8, pero no las
+   seguridad, 11 de ellos en planes de un rol, con TASK-012 en uno; la
+   clasificación con lista cerrada y cita da tres en TASK-012 (IMP-2, un
+   derivado que cambia el estado de una tarea; IMP-5, un check obligatorio que
+   el PR deja verde; e IMP-1, caso límite que la revisión 1 pasó de
+   correccion a puerta-seguridad por la precedencia de la lista, porque un
+   `comprueba` pasaba de bloquear a avisar), y con ellos las cifras de
+   puerta-seguridad de la PoC R. El plan también descartaba D1b diciendo que
+   «seguridad» ya sumaba puntos a F1-13 y F1-18; no es así (ninguna de las
+   dos tiene esa palabra) y el motivo se ha sustituido por el medido. El plan decía «como mucho 8 de 15» tareas con suelo; son 8, pero no las
    mismas que la lectura preliminar: entran F1-4 y F1-14 (falsos positivos) y
    sale F1-11 (falso negativo).
 7. **Señal de ruta `.claude/**`** (D22): el plan no la listaba; se añade porque
    `settings.json` y los hooks cambian lo que el agente puede hacer, la pregunta
    que F1-10 dejó abierta. Solo añade el dominio; el carril mínimo no cambia.
 8. **Muestra**: el estrato `fix` no puede salir de OpenDockTrading (0 tareas de
-   tipo `fix`); su hueco lo ocupa `un-dominio`, decidido antes de ejecutar T.
+   tipo `fix`); su hueco lo ocupa `un-dominio`. Se decidió antes de ejecutar
+   T, pero el historial no lo prueba (ver «Muestra de las 10 tareas»).
+9. **F1-14, manifiesto de recursos**: F1-14 pone como «obligatorio» las skills
+   de fase del plugin y como «necesario para el proyecto» lo que declara su
+   configuración. Este ADR añade a «obligatorio» los agentes de brainstorm del
+   plugin y crea un «necesario» **por tarea** (lo que `seleccionar` elige para
+   ella), con `start` parando si falta. El contrato de `evaluar_recursos` no
+   cambia, pero su lista de entrada sí.
+10. **Texto de la tarea como señal**: el plan usaba las palabras del objetivo y
+    de los criterios sin decir que son texto no confiable; la decisión 3 añade
+    que solo suben (unión con el texto aprobado, rutas de la base en la
+    revisión) y deja el resto como riesgo aceptado.
 
 ## Requisitos y evidencia
 
@@ -531,8 +668,9 @@ selección de cada tarea en curso necesita y no está instalado.
 | F1-9, contenido de `roles` | decisión 10 | divergencia 1 |
 | F1-10, roles por carril y revisoras | decisiones 4 y 6 | PoC T (carril efectivo con sus funciones); divergencia 3 |
 | F1-11 D5a, revisoras como skill del proyecto | decisiones 6 y 8 | divergencia 2 |
-| F1-14, «necesario para el proyecto» | decisión 11 | contrato de `evaluar_recursos` sin cambios |
-| Criterio 1 de la tarea, probadas a mano en 10 tareas | muestra fijada | PoC T; la columna «a mano», pendiente de Carlos |
+| F1-14, «necesario para el proyecto» | decisión 11 | contrato de `evaluar_recursos` sin cambios; divergencia 9 |
+| Criterio 1 de la tarea, probadas a mano en 10 tareas | muestra por regla reproducible | PoC T; Carlos: «Coincide las 10» (`tarea.md` de TASK-073, «Decisiones de la persona», commit `5768e3c`): **cumplido** |
+| Criterio 2, nada «a ojo» | decisiones 1 y 3 | PoC S: tabla interpretada, un caso por fila, guardia; el texto no confiable solo sube desde `approve` |
 
 ## Consecuencias
 
@@ -540,8 +678,8 @@ selección de cada tarea en curso necesita y no está instalado.
   por un catálogo que nombra algo no instalado.
 - **F4-6** implementa `seleccionar` y `dominios` con estas tablas, escribe
   `roles.<rol>` en `plan` y su criterio («en 10 tareas reales la propuesta
-  coincide con la que haría una persona») se mide sobre la muestra de este ADR
-  cuando Carlos la marque.
+  coincide con la que haría una persona») tiene ya una primera medida: en la
+  muestra de este ADR, Carlos dice que coinciden las diez.
 - **F4-1** lanza los roles que da `seleccionar` (con el suelo por dominio).
 - **F6-3** lanza las revisoras y el modelo de `roles.revision` con la
   configuración de la base; **F4-4** y **F1-16** consumen la misma selección.
@@ -561,20 +699,26 @@ selección de cada tarea en curso necesita y no está instalado.
 
 ## No verificado
 
-- **La comprobación a mano de las 10 tareas**: la columna «comprobado a mano
-  (Carlos)» está pendiente de Carlos (D10a); hasta que la marque, el
-  criterio 1 de la tarea no está cumplido, solo preparado.
+- **La comprobación a mano de las 10 tareas** la hizo Carlos («Coincide las
+  10»); es el juicio de una persona sobre 10 tareas, no una medida, y no dice
+  cómo se comporta la regla en otras.
+- **La evasión por vocabulario antes de `approve`** y en `fix` o `errata` sin
+  `approve`: el binario no la contiene (casos 2 y 5 de la PoC S); solo la
+  persona que firma el plan o aprueba el PR. Que esa persona mire las señales
+  que se le muestran no se puede verificar; F8-6 lo mide como escapados.
 - **El acierto por modelo**: la 0.16 anota `modelo_sugerido`, no el modelo con
   que se ejecutó cada fase; la `tabla-modelo` es una propuesta sin datos de
   resultado por modelo. F8-6 lo mide.
 - **La clasificación de los hallazgos** la hizo el implementador; la cita
   permite contrastarla, pero ninguna otra persona la ha repetido.
-- **Muestra pequeña**: 15 tareas de un solo proyecto, todas ADR; recall y
-  precisión del detector valen para esa muestra.
+- **Muestra pequeña y dentro de muestra**: 15 tareas de un solo proyecto,
+  todas ADR, con las que se escribió la lista de palabras; recall y precisión
+  describen ese ajuste, no la generalización.
 - **El coste del suelo** es una estimación con el coste medio por agente, no
   una medida.
 - **Si algún consumidor (orquestador, revisor en CI) puede saltarse la
-  selección**: no revisado.
+  selección**, y que el revisor en CI lea de verdad el inventario y la
+  configuración de la base: no revisado (F6-3).
 - **El inventario en Linux y macOS** y en un runner de CI: solo se leyó en esta
   máquina Windows; dos personas o una máquina y un runner con inventarios
   distintos dan selecciones distintas, que el hash de inventario anotado deja
diff --git a/docs/adr/README.md b/docs/adr/README.md
index eaf9c6f..cfc18cd 100644
--- a/docs/adr/README.md
+++ b/docs/adr/README.md
@@ -22,7 +22,7 @@ directo a la tabla de la Fase 1 y a su tarea.
 | [F1-12](F1-12-actualizacion-entre-versiones.md) | Actualización entre versiones: el plan de migración como dato (`tc2 actualizar --simular` sin cambiar un byte, mismo hash, vacío en la segunda pasada), qué migra (aplanado, formato 2, archivadas a `descartada`, config a los bloques de F1-11), historia con `git subtree split`, transacción local con criterio de salida estructural y vuelta atrás automática, retirada de parches con capacidades y huecos bloqueantes, convivencia con la 0.16 (su lector de config aborta con `---`: ninguna orden de escritura entra en un proyecto migrado), nombres definitivos `tc2` y `taskcodetwo`, y orden de pilotos OpenDockTrading → TaskCodeTwo → ToolsaWeb | TASK-013 | aceptada |
 | [F1-13](F1-13-adaptador-de-plataforma.md) | Adaptador de plataforma GitHub y GitLab: tabla de 20 garantías con su tier, verificada contra la documentación oficial con fecha y con GET de solo lectura donde se pudo (GitLab: 11 en Free, 6 en Premium y 3 en ningún tier, entre ellas el origen de un check, G09, que ni Ultimate da); sustitutos declarados que `doctor` dice siempre, contrato de `capacidades` con el plan observado, identidades en GitLab y misma batería para los dos adaptadores; tiers de pago y self-managed no verificados | TASK-015 | aceptada |
 | [F1-14](F1-14-distribucion-e-instalacion.md) | Distribución e instalación: CLI en Rust por la regla D1a (gana el 1.º y el 2.º, medidos, y el 4.º, un juicio sobre postject; sin el 4.º la regla no decide) frente a TypeScript con Node SEA (mismo comando, salida idéntica sobre las 404 tareas de ToolsaWeb y las 96 de OpenDockTrading; más rápido en caliente en los 3 repos y 32,1 veces más pequeño), TLS con el almacén del sistema, lanzador sin runtime con `cli.lock` y descarga comprobada por sha256, compatibilidad por protocolo con `tc2 version --json`, canales y firma por etapas (hash y attestation en la 2.0; Authenticode y Developer ID al entrar en winget y Homebrew) y contrato del manifiesto de recursos (`tc2 doctor --recursos`: obligatorio, necesario, conveniente y estorba); Linux y macOS no verificados | TASK-014 | aceptada |
-| [F1-15](F1-15-seleccion-de-skills-agentes-y-modelo.md) | Selección de skills, agentes y modelo por tarea y por rol: función pura `seleccionar` con tres tablas entre marcadores (reglas, modelo por carril y complejidad, dominios con lista cerrada que la config solo amplía), solo entre lo instalado (inventario de F3-9 y metadatos en la propia skill o en `recursos:`), empate por id sin modelo, suelo de 2 roles con `puertas-seguridad` (16 de 33 IMPORTANTES de la Fase 1 fueron de esa clase; detector con recall 7/8 y precisión 7/9), revisora del dominio o aviso medido con la config de la base, anotación en `roles.<rol>` y columna «Clase» para medir; 10 tareas reales de ToolsaWeb y OpenDockTrading con la comprobación a mano pendiente de Carlos | TASK-073 | aceptada |
+| [F1-15](F1-15-seleccion-de-skills-agentes-y-modelo.md) | Selección de skills, agentes y modelo por tarea y por rol: función pura `seleccionar` que interpreta tres tablas entre marcadores (reglas, modelo por carril y complejidad, dominios con lista cerrada que la config solo amplía), solo entre lo instalado (inventario de F3-9 y metadatos en la propia skill o en `recursos:`), empate por id sin modelo, suelo de 2 roles con `puertas-seguridad` (17 de 33 IMPORTANTES de la Fase 1 fueron de esa clase; detector con recall 7/8 y precisión 7/9 dentro de muestra), texto de la tarea no confiable que solo sube desde `approve`, revisora del dominio o aviso medido con la config de la base, anotación en `roles.<rol>` y columna «Clase» para medir; 10 tareas reales de ToolsaWeb y OpenDockTrading que Carlos comprobó a mano («Coincide las 10») | TASK-073 | aceptada |
 | [F1-18](F1-18-hito-de-decisiones.md) | Hito de decisiones (ADR) en `init` y en `actualizar`: estudio de adopción que lo dimensiona (bloque «uso medido» del informe de F1-12 y `proponer_hito` con un elemento por hueco, bloqueantes antes de migrar), proporcionalidad con mediana y p75 de la Fase 1 (45,5–50 min y 13,9–19,1 M tokens por ADR), registro `hito-decisiones.yml` en `taskcodetwo/tareas` cuya lista, descartes y «no necesario» firma una persona, con estado calculado que falla cerrado, y puerta determinista en `tc2 start` y en el check `estado` (código `rechazada`) | TASK-094 | aceptada |
 
 ## Plantilla
````

## Excluido del diff (8 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 2c541f98e2eb395180e4b1d28834ea517dcdcc67..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-073/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-073/planificacion/plan-final.md                 |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-073/poc/clasificacion-hallazgos.md              |  12 +-
 tareas/{02-en-curso => 03-en-revision}/TASK-073/poc/poc-f1-15.log                           |  71 ++++++++---
 tareas/{02-en-curso => 03-en-revision}/TASK-073/poc/poc-f1-15.py                            | 516 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-----------------
 tareas/03-en-revision/TASK-073/revision/informe-revision-1.md                               | 119 ++++++++++++++++++
 tareas/03-en-revision/TASK-073/revision/peticion-revision-1.md                              | 675 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-073/tarea.md                                    |  12 +-
 8 files changed, 1270 insertions(+), 135 deletions(-)
````
