# ADR F1-15 — Selección de skills, agentes y modelo por tarea y por rol: función pura `seleccionar` con tres tablas (reglas, modelo, dominios), solo entre lo instalado, suelo de roles por dominio, revisora del dominio o aviso medido, anotación en `roles.<rol>` y medición con «Clase»

- Estado: **aceptada** (2026-11-14)
- Tarea: TASK-073
- Plan: [F1-15](../plan/PLAN-TASKCODE-2.md) (dependencias F1-9 y F1-10), con el detalle de [la ampliación del 2026-11-14](../plan/AMPLIACION-2026-11-14.md), §1 a §4
- Requisitos: RR-17, RF-12, RF-13, RF-14, RF-15 y RR-05 ([catálogo](../plan/REQUISITOS-FUNCIONALES.md)); en lo de equipo, RR-09 y RR-12
- Decide: Carlos, con sus decisiones del 2026-11-14 (RR-17 imprescindible; elegir solo entre lo instalado y con reglas deterministas, nada «a ojo»; el número de roles tiene que mirar el dominio, no solo la complejidad; manifiesto de recursos de F1-14 sin duplicarlo; nombres `tc2` y `taskcodetwo`; no trocear) y la consigna «la opción recomendada» (D1a a D10a del plan de TASK-073). La columna «comprobado a mano» de las 10 tareas la marcó Carlos (D10a) el 2026-11-14: «Coincide las 10», registrado en la sección «Decisiones de la persona» de `tarea.md` de TASK-073 (commit `5768e3c`).

Este ADR fija **cómo se eligen**, para cada tarea y cada rol (diseño,
implementación, revisión), los agentes, las skills y el modelo: una función
pura, `seleccionar`, que interpreta tres tablas de este fichero
(`tabla-seleccion`, `tabla-modelo` y `tabla-dominios`), el inventario de lo
instalado y los metadatos que cada skill declara, y que nunca llama a un
modelo. Antes de decidir se midió lo que pasó en la Fase 1: los 33 IMPORTANTES
de la Fase 1 se clasificaron con una lista cerrada, y 17 de 33 IMPORTANTES son
de puerta-seguridad, 13 de ellos en planes de un solo rol. Con esa medida el
número de roles de diseño pasa a mirar el dominio de la tarea.

## Contexto

**RR-17** pide elegir para cada tarea y cada rol las skills, los agentes y el
modelo según dominio, complejidad y carril, solo entre lo instalado en la
máquina y en el proyecto, anotar la elección en la tarea y medir si acierta
(RF-12 a RF-15, RR-05).

**Cómo lo hace hoy la 0.16.1** (leído en su caché de plugin, en solo
lectura):

- `scripts/catalogo-skills.yml` es un fichero aplanado (`skill_N_*`) con 6
  entradas, todas del propio plugin; `seleccionarSkill` (en
  `src/core/catalogo-skills.ts`) cruza las etiquetas de la tarea con las de
  cada entrada, ordena por solape y prioridad y, si siguen empatadas, **pide
  el desempate a un modelo** (`src/core/plan-desempate-skill.ts`): justo lo
  que RR-17 prohíbe.
- Las revisoras se reparten por las rutas del diff
  (`clasificarPorDominio` en `src/core/revisores.ts`): si no casa ninguna, o
  casan más de `umbral_dominios` (3), va la genérica. Los patrones viven a la
  vez en el catálogo y en el bloque ```` ```yaml ```` de cada skill.
- `modelo_sugerido` vale `sonnet` por defecto y nada lo calcula; las rondas
  de verificación de correcciones bajan un escalón
  (`src/core/modelo-revision.ts`).
- El número de roles de diseño sale de `scripts/heuristica-complejidad.yml`
  (etiquetas, palabras de riesgo, dependencias, tipo y criterios): 0 a 4
  roles según el nivel, en el orden arquitectura, riesgos, testing,
  dominio, y el de rendimiento con ciertas etiquetas.
- Uso real en los pilotos: en ToolsaWeb, 21 tareas con `skills_recomendados`
  (15 `task-workflow`, 3 `rust-wasm-reviewer`, 3 `angular-vue-reviewer`) y 8
  con `modelo_sugerido: opus`; en OpenDockTrading, ninguna. La elección la
  hace hoy el agente sin criterio registrado.

**Lo que se midió antes de decidir** (PoC R y D, abajo): los planes de la Fase
1 se hicieron con un solo rol salvo F1-8, la heurística los dejó a todos en
«simple» menos F1-8, y la revisora fue siempre la genérica, porque un ADR
solo toca `docs/` y el reparto por rutas no ve su dominio.

## Alternativas

| Decisión | Alternativa | Veredicto |
|---|---|---|
| **D1a** | Roles de diseño = máx(`roles_min` del carril de F1-10, tabla de complejidad, suelo por dominio), con suelo 2 si salta `puertas-seguridad` | **Elegida**: la medida de la Fase 1 dice que el dominio, no la complejidad, predice los IMPORTANTES de puerta |
| D1b | Sumar puntos de dominio a la heurística | Descartada: la lista de palabras de riesgo de la 0.16 no tiene ninguna palabra de puertas (puerta, check, ruleset, runner, permiso…), y en la Fase 1 solo 1 de las 15 tareas de la Fase 1 contiene alguna palabra de riesgo de la 0.16 (F1-8, «seguridad»); por eso 14 salieron «simple». Sumar puntos exigiría añadir esas palabras a la heurística, y el número de roles seguiría dependiendo de cuántas aparecen, no de si el dominio está; el suelo de D1a lo decide la presencia del dominio |
| D1c | Dos roles en toda funcionalidad | Descartada: sube el coste en las tareas sin dominio delicado, que en la Fase 1 no tuvieron IMPORTANTES de puerta |
| **D2a** | Detector de dominio con lista cerrada en el binario que la configuración solo amplía | **Elegida** |
| D2b | Que el dominio lo clasifique un modelo | Descartada: es elegir «a ojo» |
| **D3a** | Desempate por solape, prioridad e id lexicográfico, con aviso | **Elegida** |
| D3b | Desempate por modelo, como la 0.16 | Descartada: RR-17 |
| **D4a** | Tabla de modelo por carril, complejidad y rol; `modelos_permitidos` solo restringe | **Elegida** |
| D4b | Un `modelo_sugerido` por tarea | Descartada: un modelo para tres roles con necesidades distintas |
| **D5a** | Diseño e implementación se calculan en `plan` y se congelan con `approve`; revisión en cada ronda con el diff real y la configuración de la base | **Elegida** |
| D5b | Todo recalculado siempre | Descartada: un plan aprobado cambiaría de agentes sin que nadie lo apruebe |
| **D6a** | Revisora del dominio si está instalada y declarada; si no, genérica con aviso `dominio-sin-revisora` medido | **Elegida** |
| D6b | Solo por rutas | Descartada: deja ciega la revisión de un ADR de puertas, que es lo que pasó en la Fase 1 |
| **D7a** | Columna «Clase» con lista cerrada, más modelo y tokens por rol | **Elegida** |
| D7b | Solo por severidad | Descartada: no dice qué rol habría visto el hallazgo |
| **D8a** | Metadatos en el bloque yaml de cada skill (y en `recursos:` para las ajenas); `roles.<rol>.agentes` como lista y `regla` con ids de fila | **Elegida** |
| D8b | Catálogo aparte, como `catalogo-skills.yml` | Descartada: es la doble copia que la 0.16 ya vio divergir |
| **D9a** | 10 tareas, 5 de ToolsaWeb y 5 de OpenDockTrading, por estratos fijados antes de mirar resultados | **Elegida** |
| D9b | Las 10 más recientes | Descartada: no cubre docs, riesgo alto ni seguridad |
| **D10a** | Carlos marca la columna «comprobado a mano»; el revisor por pares contrasta la aritmética | **Elegida** |
| D10b | Que la marque el revisor IA | Descartada: el criterio pide una persona |

## Pruebas de concepto

Guion `tareas/<estado>/TASK-073/poc/poc-f1-15.py` (la carpeta de la tarea
cambia con su estado; el guion se sitúa por su propia ruta), con su log
`poc-f1-15.log` y la tabla `clasificacion-hallazgos.md` en la misma carpeta:

```text
python -I -B poc-f1-15.py %LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-073\poc <repo-TaskCodeTwo> <carpeta-Personal>
```

Solo lectura: ToolsaWeb (commit `93c9618ac156`) y OpenDockTrading (commit
`fab8d7e05eee`, los mismos de las PoC de F1-9 y F1-10) se leen de copias
`git clone --no-hardlinks --no-checkout` en la carpeta de trabajo;
TaskCodeTwo, de su historia en `df615c14ca09` (`develop` al empezar la tarea)
y de sus ficheros; la máquina, de `~/.claude` y de la caché del plugin
0.16.1. La carpeta de trabajo lleva una marca, se borra al empezar y al
terminar, y el log es determinista (los tiempos van a stderr). Sale con 0 solo
si todo sale como se espera.

### R — Retrospectiva de la Fase 1

Una fila por IMPORTANTE de los 15 informes de la Fase 1 (todas las rondas; un
hallazgo reabierto cuenta una vez, en la ronda en que aparece), con su clase
de la lista cerrada (puerta-seguridad, contrato, correccion,
fidelidad-evidencia, reproducibilidad, con precedencia en ese orden) y una
cita del informe. El guion comprueba que las filas son exactamente los IDs de
las tablas de los informes (33), que cada cita está en el informe de su ronda
y que la clase es de la lista; cinco mutaciones de la tabla (clase fuera de
lista, cita inexistente, fila borrada, fila duplicada, ronda cambiada) fallan
cada una por su motivo. La heurística de la 0.16 reimplementada da los mismos
puntos que anotó la 0.16 en la petición de plan en 15 de 15 tareas.

<!-- tabla-retro:inicio -->
| Tarea | ADR | Roles del plan | Puntos (nivel) | IMP | puerta-seguridad | contrato | correccion | fidelidad-evidencia | reproducibilidad | Revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| TASK-002 | F1-1 | 1 (arquitectura) | 1 (simple) | 0 | 0 | 0 | 0 | 0 | 0 | `code-quality-reviewer` |
| TASK-003 | F1-2 | 1 (arquitectura) | 1 (simple) | 0 | 0 | 0 | 0 | 0 | 0 | `code-quality-reviewer` |
| TASK-004 | F1-3 | 1 (arquitectura) | 1 (simple) | 1 | 0 | 0 | 1 | 0 | 0 | `code-quality-reviewer` |
| TASK-005 | F1-4 | 1 (arquitectura) | 1 (simple) | 3 | 0 | 1 | 1 | 1 | 0 | `code-quality-reviewer` |
| TASK-006 | F1-5 | 1 (arquitectura) | 1 (simple) | 5 | 3 | 1 | 1 | 0 | 0 | `code-quality-reviewer` |
| TASK-007 | F1-6 | 1 (arquitectura) | 1 (simple) | 3 | 2 | 0 | 0 | 1 | 0 | `code-quality-reviewer` |
| TASK-008 | F1-7 | 1 (arquitectura) | 1 (simple) | 2 | 1 | 0 | 1 | 0 | 0 | `code-quality-reviewer` |
| TASK-009 | F1-8 | 2 (arquitectura, riesgos) | 4 (media) | 4 | 4 | 0 | 0 | 0 | 0 | `code-quality-reviewer` |
| TASK-010 | F1-9 | 1 (arquitectura) | 1 (simple) | 0 | 0 | 0 | 0 | 0 | 0 | `code-quality-reviewer` |
| TASK-011 | F1-10 | 1 (arquitectura) | 1 (simple) | 2 | 2 | 0 | 0 | 0 | 0 | `code-quality-reviewer` |
| TASK-012 | F1-11 | 1 (arquitectura) | 2 (simple) | 5 | 3 | 0 | 0 | 2 | 0 | `code-quality-reviewer` |
| TASK-013 | F1-12 | 1 (arquitectura) | 2 (simple) | 2 | 0 | 0 | 0 | 1 | 1 | `code-quality-reviewer` |
| TASK-014 | F1-14 | 1 (arquitectura) | 1 (simple) | 1 | 0 | 0 | 0 | 0 | 1 | `code-quality-reviewer` |
| TASK-015 | F1-13 | 1 (arquitectura) | 1 (simple) | 2 | 1 | 0 | 0 | 1 | 0 | `code-quality-reviewer` |
| TASK-094 | F1-18 | 1 (arquitectura) | 2 (simple) | 3 | 1 | 1 | 1 | 0 | 0 | `code-quality-reviewer` |
<!-- tabla-retro:fin -->

Lectura: 17 de 33 IMPORTANTES son de puerta-seguridad, 13 de ellos en planes
de un solo rol; por clase, el log da puerta-seguridad 17, contrato 3,
correccion 5, fidelidad-evidencia 6 y reproducibilidad 2 (TASK-012 IMP-1 es un
caso límite: lo diseñado no funcionaba y además un `comprueba` pasaba de
bloquear a avisar; por la precedencia de la lista cuenta como puerta). Como
solo F1-8 tuvo rol de riesgos, 13 IMPORTANTES de puerta-seguridad escaparon a
un plan sin rol de riesgos, y F1-8, con él, salió aun así con cuatro: el rol de riesgos sube la
probabilidad de verlos, no la garantiza, y por eso la decisión también actúa
en la revisión (decisión 6). Además, 7 IMPORTANTES aparecieron en una ronda de
verificación, 5 de puerta-seguridad: la regla de la 0.16 de bajar un escalón
el modelo en esas rondas no se aplica en ese dominio (fila S13). La
heurística dejó 14 tareas en «simple» y una (F1-8) en «media», porque solo
1 de las 15 tareas de la Fase 1 contiene alguna palabra de riesgo de la 0.16
(F1-8, «seguridad»; la lista no tiene ninguna palabra de puertas): sus puntos
salen de dependencias y criterios. La revisora fue `code-quality-reviewer` en
las 15. Y 14 de las 15 tareas de la Fase 1 tienen una fila de implementación
hecha por un subagente en `## Coste` (todas menos F1-1), lo que fija qué hace
la fila de implementación de la `tabla-modelo` (decisión 5).

La PoC identifica cada hallazgo por su ID dentro de la tarea y no puede
detectar un ID reutilizado para otro hallazgo; eso se comprobó leyendo los
informes (los IDs nuevos de cada ronda van marcados «nuevo»).

### D — El detector `dominios` sobre la Fase 1 (antes de la decisión)

La `tabla-dominios` de la decisión 3 aplicada a las 15 tareas (objetivo,
criterios, etiquetas y los ficheros de su merge en `develop`, todos bajo
`docs/`), frente a «tiene al menos un IMPORTANTE de puerta-seguridad» (8
tareas): **recall 7/8 y precisión 7/9**. El falso negativo es F1-11 (sus
IMPORTANTES de puerta hablan de derivados y del check `estado`, palabras que
no están en la tabla); los dos falsos positivos son F1-4, por «tokens» en el
sentido de coste, y F1-14, por «firma» de binarios, que es seguridad pero no
tuvo IMPORTANTES. Las palabras son las que fijó el plan aprobado y la PoC no
las ajustó a la vista del resultado, pero **es una medida dentro de muestra**:
el plan eligió esa lista después de una lectura preliminar de estos mismos 15
informes, y algunas palabras (`ruleset`, `runner`, `amenaza`, `identidad`)
salen de ellos. Recall y precisión dicen cómo encaja la tabla con los datos con
que se escribió, no cómo generaliza; la validación fuera de muestra es de F8-6,
con tareas nuevas.

Con esos datos, el suelo subiría de 1 a 2 roles en 8 de 15 tareas (F1-4,
F1-5, F1-6, F1-7, F1-10, F1-13, F1-14 y F1-18), más el unificador. El coste
medio de un agente de diseño, de las filas `diseno` de `## Coste` de las 15
tareas (total entre agentes, la misma fuente que F1-18), es 747 469 tokens;
el suelo añade dos agentes (riesgos y unificador) en cada una de las 8:
11 959 504 tokens en total, unos 1,5 M por tarea que lo dispara, el 10,7 % de
la mediana de 13,9 M tokens por ADR de [F1-18](F1-18-hito-de-decisiones.md).

### I — Inventario real de esta máquina

| Origen | Skills | Agentes |
|---|---|---|
| plugin TaskCode 0.16.1 | 14 | 7 |
| otros 10 plugins instalados (`installed_plugins.json`) | 33 | 1 |
| usuario (`~/.claude/skills`, `~/.claude/agents`) | 6 | 37 |
| integrado en Claude Code (`general-purpose`) | — | 1 |
| total | 53 | 46 |

Proyectos: TaskCodeTwo no tiene `.claude/`; ToolsaWeb solo tiene
`.claude/settings.json` en el commit fijo; OpenDockTrading no lo tiene.
Ninguno trae skills ni agentes propios. Del inventario, solo 13 entradas
tienen metadatos de selección (las revisoras del plugin con su bloque yaml,
`task-workflow` en `catalogo-skills.yml`, los seis agentes de brainstorm y el
integrado); las otras 86 están instaladas pero no declaradas, entre ellas la
skill `security-review` y el agente `security-auditor`, que serían las
candidatas naturales a revisora de `puertas-seguridad`.

### Muestra de las 10 tareas (regla reproducible)

La muestra es una regla, no una lista: aplicada a los commits fijos de los
pilotos da exactamente las 10 tareas de la tabla T, y la PoC lo comprueba (la
sección G falla si la tabla tiene una tarea que la regla no da o le falta una
que sí da). Se escribió antes de ejecutar T, pero **eso no lo prueba el
historial**: la regla, la PoC y los resultados entraron en el mismo commit, y
el plan aprobado (D9a) solo fijaba los estratos por su nombre, no el desempate
por ID ni el estrato `un-dominio`. Quien dude del orden puede comprobar que la
regla es la más simple que cabe (ID más alto por estrato) y aplicarla de nuevo.

Por cada piloto, estratos en este orden; en cada estrato, la tarea terminada
de **ID más alto** con merge en `develop` y algún fichero fuera de `tareas/`
que cumple el criterio y no está ya elegida:

- ToolsaWeb: `skills-016` (`skills_recomendados` no vacío en la 0.16), `fix`
  (tipo `fix`), `riesgo-alto` (carril mínimo por ficheros de F1-10),
  `solo-docs` (carril mínimo `errata`), `multidominio` (tipo `feature` y dos
  o más grupos del reparto por rutas de la 0.16, contando el de ficheros sin
  dominio).
- OpenDockTrading: `seguridad` (alguna de seguridad, autenticación,
  autorización, cifrado o datos personales, de la lista de palabras de riesgo
  de la 0.16, en objetivo o criterios), `riesgo-alto`, `solo-docs`,
  `multidominio` y `un-dominio` (tipo `feature`, un solo grupo y carril mínimo
  `fix`). OpenDockTrading no tiene ninguna tarea de tipo `fix` (sus 43
  terminadas son `feature`), y ToolsaWeb es el único con `skills_recomendados`;
  por eso esos dos estratos van solo a ToolsaWeb y el hueco de
  OpenDockTrading lo ocupa `un-dominio`.

El criterio de `seguridad` usa las palabras de la 0.16, no la `tabla-dominios`,
para que la muestra no dependa del detector que se prueba.

### T — Las 10 tareas reales

Carril declarado como en F1-10 (`fix` si el tipo es `fix`, `funcionalidad`
si no), mínimo por ficheros y efectivo con sus funciones; complejidad con la
heurística de la 0.16 (el mayor entre la declarada y la calculada); la
propuesta es la salida de `seleccionar` con el inventario y el catálogo de la
sección I y sin configuración. «Comprobado a mano» lo marcó Carlos (D10a): «Coincide las 10» (registro en
`tarea.md` de TASK-073, «Decisiones de la persona», commit `5768e3c`). Las correcciones de la revisión 1
no cambiaron ninguna de las 10 propuestas: la PoC recalcula cada fila con las tablas de este fichero y la
compara con la tabla.

<!-- tabla-muestra:inicio -->
| Tarea | Estrato | Carril (declarado → efectivo) | Complejidad | Dominios (señal) | Diseño | Implementación | Revisión | Modelo D / I / R | Reglas D · I · R | Lo que usó la 0.16 | Comprobado a mano (Carlos) |
|---|---|---|---|---|---|---|---|---|---|---|---|
| ToolsaWeb/TASK-306 | skills-016 | funcionalidad → funcionalidad | media (3 p) | — | 2: arquitectura, riesgos + unificador | task-workflow | rust-wasm-reviewer | opus / sonnet / opus | M05 S02 S05 · M05 S07 · M05 S08 S12 | roles 2 (arquitectura, riesgos); rev rust-wasm-reviewer; modelo sonnet; skills task-workflow | coincide (Carlos, 2026-11-14) |
| ToolsaWeb/TASK-399 | fix | fix → fix | simple (1 p) | — | 1: arquitectura | — | rust-wasm-reviewer, code-quality-reviewer | sonnet / sonnet / sonnet | M02 S02 · M02 · M02 S08 S12 | roles 1 (arquitectura); rev code-quality-reviewer; modelo sonnet; skills — | coincide (Carlos, 2026-11-14) |
| ToolsaWeb/TASK-403 | riesgo-alto | funcionalidad → riesgo-alto | media (3 p) | puertas-seguridad (palabra:puerta) | 4: arquitectura, riesgos, testing, dominio + unificador | — | code-quality-reviewer; aviso: puertas-seguridad sin revisora | opus / opus / opus | M07 S02 S05 · M07 · M07 S09 S11 S12 | roles 2 (arquitectura, riesgos); rev code-quality-reviewer; modelo sonnet; skills — | coincide (Carlos, 2026-11-14) |
| ToolsaWeb/TASK-299 | solo-docs | funcionalidad → funcionalidad | media (3 p) | — | 2: arquitectura, riesgos + unificador | task-workflow | code-quality-reviewer | opus / sonnet / opus | M05 S02 S05 · M05 S07 · M05 S09 S12 | roles 2 (arquitectura, riesgos); rev code-quality-reviewer; modelo sonnet; skills task-workflow | coincide (Carlos, 2026-11-14) |
| ToolsaWeb/TASK-402 | multidominio | funcionalidad → funcionalidad | media (5 p) | puertas-seguridad (palabra:firma) | 2: arquitectura, riesgos + unificador | — | angular-vue-reviewer, rust-wasm-reviewer, code-quality-reviewer; aviso: puertas-seguridad sin revisora | opus / sonnet / opus | M05 S02 S05 · M05 · M05 S08 S11 S12 | roles 2 (arquitectura, riesgos); rev angular-vue-reviewer, code-quality-reviewer, rust-wasm-reviewer; modelo sonnet; skills — | coincide (Carlos, 2026-11-14) |
| OpenDockTrading/TASK-063 | seguridad | funcionalidad → funcionalidad | media (5 p) | puertas-seguridad (etiqueta:seguridad) | 2: arquitectura, riesgos + unificador | — | angular-vue-reviewer, rust-wasm-reviewer, code-quality-reviewer; aviso: puertas-seguridad sin revisora | opus / sonnet / opus | M05 S02 S05 · M05 · M05 S08 S11 S12 | roles 2 (arquitectura, riesgos); rev angular-vue-reviewer, code-quality-reviewer, rust-wasm-reviewer; modelo sonnet; skills — | coincide (Carlos, 2026-11-14) |
| OpenDockTrading/TASK-014 | riesgo-alto | funcionalidad → riesgo-alto | alta (6 p) | puertas-seguridad (palabra:check) | 4: arquitectura, riesgos, testing, dominio + unificador | — | rust-wasm-reviewer, code-quality-reviewer; aviso: puertas-seguridad sin revisora | opus / opus / opus | M07 S02 S05 · M07 · M07 S08 S11 S12 | roles 3 (arquitectura, riesgos, testing); rev code-quality-reviewer, rust-wasm-reviewer; modelo sonnet; skills — | coincide (Carlos, 2026-11-14) |
| OpenDockTrading/TASK-067 | solo-docs | funcionalidad → funcionalidad | media (4 p) | — | 2: arquitectura, riesgos + unificador | — | code-quality-reviewer | opus / sonnet / opus | M05 S02 S05 · M05 · M05 S09 S12 | roles 2 (arquitectura, riesgos); rev general-purpose; modelo sonnet; skills — | coincide (Carlos, 2026-11-14) |
| OpenDockTrading/TASK-069 | multidominio | funcionalidad → funcionalidad | simple (1 p) | — | 1: arquitectura | — | rust-wasm-reviewer, code-quality-reviewer | sonnet / sonnet / sonnet | M04 S02 · M04 · M04 S08 S12 | roles 1 (arquitectura); rev code-quality-reviewer, rust-wasm-reviewer; modelo sonnet; skills — | coincide (Carlos, 2026-11-14) |
| OpenDockTrading/TASK-096 | un-dominio | funcionalidad → funcionalidad | simple (3 p) | puertas-seguridad (palabra:check) | 2: arquitectura, riesgos + unificador | — | code-quality-reviewer; aviso: puertas-seguridad sin revisora | sonnet / sonnet / sonnet | M04 S02 S03 S05 · M04 · M04 S09 S11 S12 | roles 1 (arquitectura); rev code-quality-reviewer; modelo sonnet; skills — | coincide (Carlos, 2026-11-14) |
<!-- tabla-muestra:fin -->

Lectura (aritmética sobre la tabla; el juicio es el de Carlos, que dice que coinciden las diez): el
número de roles de diseño propuesto coincide con el que lanzó la 0.16 en siete
de las diez filas; sube en las dos de riesgo alto (todos, por F1-10) y en
TASK-096 de OpenDockTrading, donde el suelo por dominio (S03) pone el rol de
riesgos que la 0.16 no lanzó. Las revisoras coinciden en ocho; en TASK-399 de
ToolsaWeb la propuesta añade `rust-wasm-reviewer` (el diff toca `.rs`) y en
TASK-067 de OpenDockTrading la 0.16 anotó el agente (`general-purpose`), no la
skill, así que no se pueden comparar. El modelo deja de ser `sonnet` para todo:
`opus` en diseño y revisión desde complejidad media y en todo el riesgo alto
(siete filas). En cinco filas sale el aviso de dominio sin revisora: con lo
instalado y declarado hoy, ninguna revisora declara `puertas-seguridad`.
Ninguna fila de la muestra cae en un empate.

### S — `seleccionar` interpreta la tabla, es determinista y no sale de la máquina

- `seleccionar` no lleva las reglas escritas aparte: **lee `tabla-seleccion`**
  de este fichero y aplica cada fila en su orden; la columna «Condición» es una
  lista de átomos de un vocabulario cerrado y «Acción» un token de otro
  (decisión 5). Una condición o acción fuera del vocabulario es un error.
- Un caso de control por cada fila S01 a S15, más cuatro de S13 (ronda 2 sin
  dominio baja de `opus` a `sonnet`; en riesgo alto, con `puertas-seguridad`
  por el texto y con `puertas-seguridad` por la ruta `.claude/settings.json`
  no baja). S01, S04, S06 y S10, que no salen en la muestra, tienen el suyo
  (S06 y S10 con una skill y una revisora sintéticas que declaran
  `puertas-seguridad`). La PoC falla si una fila no la ejercita ningún caso.
- Las 10 tareas × 3 roles dos veces: mismo hash; con el inventario y el
  catálogo permutados, el mismo hash.
- Quitar `angular-vue-reviewer` del inventario la saca de las 2 selecciones
  que la tenían, con aviso `no-instalado` y fila S14.
- Empate total (tres skills de ejecución con el mismo solape y la misma
  prioridad): elige `alfa-skill` con aviso `empate-por-id`, también en orden
  inverso.
- Sin red ni modelo: un *audit hook* de Python que rechaza `socket.*`,
  `subprocess.*`, `urllib.*`, `http.*` y `os.system` durante esas llamadas
  registra 0 eventos; el fuente de `seleccionar` y sus funciones no nombra
  red, procesos, ficheros ni modelos, y el guion no importa ninguna biblioteca
  de red ni de modelos.
- La configuración solo restringe y solo añade: `modelos_permitidos: [haiku,
  sonnet]` convierte `opus` en `sonnet` con aviso (S15); un dominio nuevo
  (`datos`), un modelo fuera de la escala o una clave desconocida se
  rechazan; `dominios_extra` añade una ruta a `puertas-seguridad`.
- El modelo cambia con el carril (funcionalidad/media frente a
  riesgo-alto/media) y con la complejidad (funcionalidad/simple frente a
  funcionalidad/media) en filas reales de la muestra: el criterio de F4-6.
- **Evasión por vocabulario** (el mismo trabajo con otras palabras; decisión
  3, «Texto de la tarea»). Lo que la PoC reproduce y su resultado esperado,
  fijado en el guion:

  | Caso | Resultado | ¿Lo contiene? |
  |---|---|---|
  | 1. `plan`, texto con «puerta», «check», «token», «aprobación» | 2 roles (suelo) | sí |
  | 2. `plan`, mismo trabajo escrito con «validación», «credencial», «conformidad» | 1 rol, sin dominio | **no el binario**: solo la persona en `approve`, que ve «dominios: ninguno» y el suelo calculado |
  | 3. revisión ronda 2, texto aprobado con puertas y texto vivo reescrito | sigue `opus` y el aviso | sí: el texto aprobado suma |
  | 4. revisión ronda 2, otro vocabulario y el diff toca `.claude/settings.json` | sigue `opus` y el aviso | sí: la ruta, con la configuración de la base |
  | 5. carril `fix` (sin `approve`), otro vocabulario y diff solo en `src/` | sin dominio, sin aviso, S13 baja el modelo | **no**: riesgo aceptado |

- **Catálogo del árbol del PR** (decisión 2): un PR que añade una genérica
  propia no la cuela (la genérica solo puede ser la del plugin, y la función
  lo impone); un PR que añade una revisora con patrones `**/*.rs` sí entraría
  si su catálogo se usara: la función no lo puede impedir, por eso el
  inventario y el catálogo de la revisión se leen siempre de la base.

### G — Este ADR como dato

El guion lee las tres tablas de la decisión, la muestra y la retrospectiva de
este fichero y sale con 1 si una fila no tiene id o lo repite, un modelo no
está en la escala, una combinación de carril y complejidad tiene cero o dos
filas de modelo, un dominio no es de la lista cerrada, una condición o una
acción de `tabla-seleccion` no es del vocabulario, un caso de control da otra
cosa con las tablas del fichero, una regla da un id que no está en el
inventario, una fila de la muestra (recalculada con las tablas del fichero) o
de la retrospectiva no es la calculada, la columna «a mano» no lleva persona y
fecha, falta una sección, una regla obligatoria, una divergencia o
un «no verificado», un ID del plan o de los requisitos no existe, un enlace no
resuelve o una cifra de la prosa (IMPORTANTES, recall, precisión) está fuera
de la frase que dice a qué corresponde. Veinticinco copias mutadas fallan cada
una por su motivo, entre ellas S13 cambiada a «siempre», el suelo de S03 a 3 y
la acción de S10 cambiada por la genérica.

## Cómo se ve en la tarea y en GitHub

En `tarea.md` (formato 2 de [F1-9](F1-9-formato-de-tarea-plan-y-especificacion.md)),
`plan` escribe diseño e implementación y `review` reescribe revisión en cada
ronda:

```yaml
# TASK-096 de OpenDockTrading, tal como la propone la PoC T (ids con el prefijo de la 2.0)
roles:
  diseno:
    modelo: "sonnet"
    agentes: ["taskcodetwo:brainstorm-arquitectura", "taskcodetwo:brainstorm-riesgos", "taskcodetwo:brainstorm-unificador"]
    skills: []
    regla: ["M04", "S02", "S03", "S05"]
    senales: ["puertas-seguridad:palabra:check"]
    avisos: []
    texto: "<blob de tarea.md en el commit firmado de approve>"
    inventario: "2ecd98236392"
    catalogo: "07d5ec762090"
  implementacion: {modelo: "sonnet", agentes: [], skills: [], regla: ["M04"], senales: [...], avisos: [], inventario: "...", catalogo: "..."}
  revision: {modelo: "sonnet", agentes: ["general-purpose"], skills: ["taskcodetwo:code-quality-reviewer"], regla: ["M04", "S09", "S11", "S12"], avisos: ["dominio-sin-revisora:puertas-seguridad"], ronda: 1, ...}
```

En GitHub no hay nada nuevo que configurar: el revisor en CI (F6-3) lee
`roles.revision` calculado con la configuración de la base, nunca la del
worktree del PR (decisión 3 de F1-10), y el check `revision-ia` deja en su
resumen la fila, las señales y los avisos. En `approve`, `tc2` muestra a la
persona los dominios detectados con su señal y el número de roles que salen
(«dominios: ninguno; roles 1» también se muestra), antes de que firme.
`tc2 doctor --recursos` lista lo que la
selección de cada tarea en curso necesita y no está instalado.

## Decisión

1. **Una función pura que interpreta tres tablas.** Sin E/S, en el núcleo
   del binario de `tc2`:

   ```text
   seleccionar(rol: Diseno|Implementacion|Revision, t: TareaLeida, carril: Carril, cx: Complejidad,
               cambios: [Cambio], inv: Inventario, cat: Catalogo, cfg: ConfigValidada, ronda: n = 1) -> Seleccion
     Seleccion = {agentes: [id], skills: [id], modelo: haiku|sonnet|opus|null, reglas: [IdFila], avisos: [Aviso], senales: [Senal]}
   dominios(t: TareaLeida, cambios: [Cambio], tabla: TablaDominios, cfg: ConfigValidada) -> [(dominio, [senal])]
     Cambio = {estado, rutas: [ruta] | [ruta_vieja, ruta_nueva]}    # los dos nombres, como en F1-10
   ```

   `carril` es el **efectivo** de F1-10 y `cx` el mayor entre la complejidad
   declarada y la de la heurística (la de la 0.16, sin cambios). La salida va
   ordenada; dos llamadas con la misma entrada, en cualquier orden de
   inventario y catálogo, dan lo mismo. **Nunca llama a un modelo ni a la
   red**: un empate se resuelve por id lexicográfico con aviso. Las reglas no
   están escritas en el código aparte de la tabla: `seleccionar` recorre
   `tabla-seleccion` en su orden y, para las filas de su rol y las de `todos`,
   evalúa la «Condición» (átomos de un vocabulario cerrado unidos por «y», cada
   uno con «no» delante si se niega: `siempre`, `carril=…`, `carril!=…`,
   `dominio=…`, `roles>=n`, `ronda>=n`, `hay-skill-diseno-del-dominio`,
   `hay-skill-ejecucion-con-solape`, `rutas-casan-entre-1-y-umbral`,
   `rutas-no-casan-o-pasan-umbral`, `dominio-con-revisora`,
   `dominio-sin-revisora`, `hay-no-instalado`, `modelo-no-permitido`) y aplica
   la «Acción» (un token de otro vocabulario cerrado). Lo que no es del
   vocabulario es un error al leer la tabla. Una fila se anota si su condición
   se cumple y su acción cambia algo. La usan `plan`
   (F4-6, F4-1), `review` y el revisor en CI (F6-3), el orquestador (F4-4),
   `lote` (F1-16) y `doctor` (F3-8, F3-9).

2. **Fuentes: solo lo instalado.** El **inventario** (F3-9) es la única
   fuente de lo que existe: skills y agentes del plugin `taskcodetwo`, de otros
   plugins instalados, de `~/.claude/skills` y `~/.claude/agents`, de
   `.claude/skills` y `.claude/agents` del proyecto, y los agentes integrados
   de la plataforma (`general-purpose`). **Árbol del proyecto**: en `plan`, el
   del commit en que se calcula (anotado por hash); en `review` y en el revisor
   en CI, siempre el de la base del PR (`develop`), leído con `git show` de la
   base, nunca el worktree del PR, y lo mismo para la configuración. La PoC S
   muestra por qué: una revisora que el PR añade a su árbol entraría en la
   selección si se leyera de allí; la genérica, en cambio, solo puede ser la del
   plugin y la función lo impone. El **catálogo** son los metadatos de
   selección de cada entrada: los del bloque ```` ```yaml ```` de la propia
   skill (decisión 8) o, para una skill o un agente ajeno, los que declara el
   bloque `recursos:` del proyecto ([F1-14](F1-14-distribucion-e-instalacion.md)).
   Lo instalado sin metadatos no se elige. Lo que el catálogo nombra y no está
   instalado se quita de la selección con aviso `no-instalado` (S14), y en
   `tc2 doctor --recursos` es un error con el arreglo.

3. **Detector de dominio.** La lista de dominios es cerrada y va en el
   binario: `puertas-seguridad` y `rendimiento`. Las señales de cada uno son
   las de esta tabla; la configuración (`dominios_extra`) **solo añade**
   etiquetas, palabras o rutas a un dominio de la lista, nunca un dominio
   nuevo ni quita señales. Etiqueta: igualdad exacta. Palabra: al principio de
   una palabra del objetivo o de los criterios, sin tildes ni mayúsculas.
   Ruta: glob sobre los dos nombres de cada renombrado.

   **Texto de la tarea: no confiable, solo sube.** Objetivo, criterios y
   etiquetas los escribe el agente en `tarea.md`, como el `tipo` que F1-10 ya
   trató como no confiable. Con otras palabras para el mismo trabajo
   desaparecerían el suelo, el aviso `dominio-sin-revisora` y la excepción de
   S13. Cómo se contiene:

   - **Hasta `approve`** el binario no lo puede contener: el texto es lo único
     que hay. Lo contiene la persona: `approve` le muestra los dominios
     detectados con su señal y el número de roles («dominios: ninguno; roles
     1» incluido), y ella firma eso; puede subirlo con la etiqueta `puertas`
     (D02) o con más roles. Riesgo aceptado.
   - **Desde `approve`**, el texto aprobado queda congelado con la firma (su
     blob se anota en `roles.<rol>.texto`) y `dominios()` usa la unión de las
     señales del texto aprobado y del vivo: reescribir `tarea.md` después solo
     puede añadir dominios, nunca quitarlos.
   - **En la revisión**, las rutas del diff se evalúan en cada ronda con la
     configuración de la base: un cambio en `.github/`, `.taskcode/`,
     `CODEOWNERS` o `.claude/` da `puertas-seguridad` diga lo que diga el texto,
     y S13 no baja el modelo si hay `puertas-seguridad` por cualquier señal o el
     carril es `riesgo-alto`.
   - **Lo que queda fuera**: una tarea de `fix` o de `errata` no pasa por
     `approve`, así que si su texto evita las palabras y su diff no toca esas
     rutas, no hay dominio, ni aviso, y S13 baja el modelo (caso 5 de la PoC
     S). Lo contiene solo la persona que aprueba el PR (`merge-pr`), que ve en
     el check las señales y los avisos. Riesgo aceptado, medido por F8-6 como
     escapados.

   <!-- tabla-dominios:inicio -->
   | Id | Dominio | Señal | Valor |
   |---|---|---|---|
   | D01 | `puertas-seguridad` | etiqueta | `seguridad` |
   | D02 | `puertas-seguridad` | etiqueta | `puertas` |
   | D03 | `puertas-seguridad` | etiqueta | `permisos` |
   | D04 | `puertas-seguridad` | palabra | `puerta` |
   | D05 | `puertas-seguridad` | palabra | `check` |
   | D06 | `puertas-seguridad` | palabra | `ruleset` |
   | D07 | `puertas-seguridad` | palabra | `permiso` |
   | D08 | `puertas-seguridad` | palabra | `identidad` |
   | D09 | `puertas-seguridad` | palabra | `firma` |
   | D10 | `puertas-seguridad` | palabra | `token` |
   | D11 | `puertas-seguridad` | palabra | `secreto` |
   | D12 | `puertas-seguridad` | palabra | `aprobacion` |
   | D13 | `puertas-seguridad` | palabra | `carril` |
   | D14 | `puertas-seguridad` | palabra | `merge` |
   | D15 | `puertas-seguridad` | palabra | `workflow` |
   | D16 | `puertas-seguridad` | palabra | `runner` |
   | D17 | `puertas-seguridad` | palabra | `amenaza` |
   | D18 | `puertas-seguridad` | palabra | `seguridad` |
   | D19 | `puertas-seguridad` | ruta | `.github/**` |
   | D20 | `puertas-seguridad` | ruta | `.taskcode/**` |
   | D21 | `puertas-seguridad` | ruta | `**/CODEOWNERS` |
   | D22 | `puertas-seguridad` | ruta | `.claude/**` |
   | D23 | `rendimiento` | etiqueta | `rendimiento` |
   | D24 | `rendimiento` | etiqueta | `wasm` |
   | D25 | `rendimiento` | etiqueta | `workers` |
   | D26 | `rendimiento` | etiqueta | `3d` |
   <!-- tabla-dominios:fin -->

4. **Roles de diseño (D1a).** Número = máx(`roles_min` del carril efectivo
   según la `tabla-politica` de [F1-10](F1-10-carriles-y-modo-automatico.md),
   tabla de complejidad de la 0.16 en funcionalidad —0, 1, 2, 3 o 4 según el
   nivel, nunca menos de 1—, **suelo por dominio**). El suelo de
   `puertas-seguridad` es 2: arquitectura y riesgos. En `errata` no hay plan;
   en `riesgo-alto`, todos. Con `rendimiento` se añade su rol; con dos o más
   roles, el unificador. El orden de roles no cambia (arquitectura, riesgos,
   testing, dominio), así que con 2 siempre está riesgos.

5. **Reglas de selección.** Cada fila que aplica queda anotada por su id.

   <!-- tabla-seleccion:inicio -->
   | Id | Rol | Condición | Acción | Ids del inventario | Qué significa |
   |---|---|---|---|---|---|
   | S01 | `diseno` | `carril=errata` | `sin-plan` | — | sin plan: no se lanza ningún rol |
   | S02 | `diseno` | `carril!=errata` | `roles=max(roles_min,complejidad)` | `taskcode-plugin:brainstorm-arquitectura`, `taskcode-plugin:brainstorm-riesgos`, `taskcode-plugin:brainstorm-testing`, `taskcode-plugin:brainstorm-dominio` | número de roles = máx(`roles_min` del carril, tabla de complejidad); un agente por rol en el orden arquitectura, riesgos, testing, dominio |
   | S03 | `diseno` | `dominio=puertas-seguridad` | `suelo=2` | `taskcode-plugin:brainstorm-arquitectura`, `taskcode-plugin:brainstorm-riesgos` | suelo de 2 roles: arquitectura y riesgos |
   | S04 | `diseno` | `dominio=rendimiento` | `mas-rol=rendimiento` | `taskcode-plugin:brainstorm-rendimiento` | más el rol de rendimiento |
   | S05 | `diseno` | `roles>=2` | `mas-rol=unificador` | `taskcode-plugin:brainstorm-unificador` | con dos o más roles, más el unificador |
   | S06 | `diseno` | `hay-skill-diseno-del-dominio` | `skills-diseno-del-dominio` | — | las skills declaradas con `rol: diseno` y un dominio detectado |
   | S07 | `implementacion` | `hay-skill-ejecucion-con-solape` | `skill-ejecucion=solape,prioridad,id` | `taskcode-plugin:task-workflow` | una skill de `rol: ejecucion`: mayor solape de etiquetas o dominios, después mayor prioridad, después menor id, con aviso si hubo empate |
   | S08 | `revision` | `rutas-casan-entre-1-y-umbral` | `revisoras-de-rutas+generica-si-sueltas` | `taskcode-plugin:angular-vue-reviewer`, `taskcode-plugin:csharp-autocad-ifc-reviewer`, `taskcode-plugin:java-spring-reviewer`, `taskcode-plugin:rust-wasm-reviewer` | las revisoras cuyos patrones casan con las rutas del diff (los dos nombres de cada renombrado), más la genérica si queda algún fichero sin dominio |
   | S09 | `revision` | `rutas-no-casan-o-pasan-umbral` | `generica-del-plugin` | `taskcode-plugin:code-quality-reviewer` | ninguna casa, o casan más de `umbral_dominios`: la genérica, que solo puede ser la del plugin |
   | S10 | `revision` | `dominio-con-revisora` | `revisora-del-dominio` | — | por cada dominio detectado con revisora declarada e instalada, esa revisora, aunque las rutas no la pidan |
   | S11 | `revision` | `dominio-sin-revisora` | `aviso=dominio-sin-revisora` | — | por cada dominio detectado sin revisora declarada, el aviso, anotado y medido |
   | S12 | `revision` | `siempre` | `lanzador` | `general-purpose` | el agente que lanza cada revisora |
   | S13 | `revision` | `ronda>=2` y no `carril=riesgo-alto` y no `dominio=puertas-seguridad` | `modelo-baja=1` | — | ronda de verificación fuera de riesgo alto y de puertas: el modelo baja un escalón |
   | S14 | `todos` | `hay-no-instalado` | `quitar-no-instalado` | — | lo elegido que no está en el inventario se quita, con aviso `no-instalado` |
   | S15 | `todos` | `modelo-no-permitido` | `restringir-modelo` | — | `modelos_permitidos` no admite el modelo: el permitido más cercano por debajo (si no hay, por encima), con aviso |
   <!-- tabla-seleccion:fin -->

   Los ids de la última columna son los del inventario de hoy (plugin 0.16.1);
   en la 2.0 llevan el prefijo del plugin `taskcodetwo` y la comprobación de
   la PoC G es la misma.

   **El agente de implementación no lo elige `seleccionar`.** En la Fase 1, 14
   de las 15 tareas tienen en `## Coste` una fila de implementación hecha por
   un subagente: la implementación la hace un agente que lanza quien orquesta,
   y en el modo automático (F4-4) será igual. Ese agente es siempre el integrado
   `general-purpose` (no hay candidatos entre los que elegir), así que
   `seleccionar` le da el modelo (columna «Implementación» de la
   `tabla-modelo`) y las skills (S07), y quien lo lanza anota su id en
   `roles.implementacion.agentes`. Si la implementación la hace la sesión de la
   persona, el modelo de la fila es un consejo, no se impone.

6. **Revisoras por dominio (D6a).** La revisión suma dos caminos: por rutas
   (S08 y S09, el reparto de la 0.16 con los dos nombres de cada renombrado) y
   por dominio (S10): si `dominios()` da un dominio y hay una revisora
   instalada que lo declara en sus metadatos, entra aunque las rutas no la
   pidan; si no la hay, va la genérica (siempre la del plugin: una skill del
   proyecto con `fallback: true` no la sustituye) y la tarea lleva el aviso
   `dominio-sin-revisora` (S11), que F8-6 cuenta. No se inventa una revisora:
   un proyecto que quiera la de puertas declara la suya (por ejemplo, la skill
   `security-review` que ya está instalada) en `recursos:` con
   `rol: revisor` y `dominios: [puertas-seguridad]`. La selección de revisión
   se calcula siempre con la configuración de la base y las skills de la base,
   nunca con las del worktree del PR (decisión 3 de F1-10): un PR no puede
   quitarse su revisora ni ponerse una. La PoC solo lo comprueba en la
   función (catálogo de la base frente al del PR); que el revisor en CI lea
   de verdad la base es de F6-3.

7. **Modelo (D4a).** Escala `haiku` < `sonnet` < `opus`. Una fila por carril y
   complejidad; cada combinación tiene exactamente una (la PoC G lo
   comprueba).

   <!-- tabla-modelo:inicio -->
   | Id | Carril | Complejidad | Diseño | Implementación | Revisión |
   |---|---|---|---|---|---|
   | M01 | `errata` | cualquiera | — | `haiku` | `sonnet` |
   | M02 | `fix` | `trivial`, `simple` | `sonnet` | `sonnet` | `sonnet` |
   | M03 | `fix` | `media`, `alta`, `critica` | `opus` | `sonnet` | `opus` |
   | M04 | `funcionalidad` | `trivial`, `simple` | `sonnet` | `sonnet` | `sonnet` |
   | M05 | `funcionalidad` | `media` | `opus` | `sonnet` | `opus` |
   | M06 | `funcionalidad` | `alta`, `critica` | `opus` | `opus` | `opus` |
   | M07 | `riesgo-alto` | cualquiera | `opus` | `opus` | `opus` |
   <!-- tabla-modelo:fin -->

   `modelos_permitidos` en la configuración **solo restringe** (S15); el valor
   por defecto del binario es la escala completa y `doctor` lo dice. Las rondas
   de verificación bajan un escalón como en la 0.16, salvo en riesgo alto y en
   `puertas-seguridad` (S13), porque ahí aparecieron la mayoría de los
   hallazgos nuevos de esas rondas en la Fase 1.

8. **Metadatos en la propia skill (D8a).** El bloque ```` ```yaml ```` que
   F1-11 fijó para las revisoras (`rol: revisor`, `patrones_archivo`) se amplía
   a toda skill y agente del plugin con `rol` ∈ {`diseno`, `ejecucion`,
   `revisor`}, `etiquetas`, `dominios` y `prioridad` (YAML estándar, F1-9);
   los agentes de brainstorm declaran su rol de diseño. Desaparece
   `catalogo-skills.yml` y con él la doble copia de los patrones. Las skills y
   agentes ajenos declaran lo mismo en `recursos:` (F1-14).

9. **Cuándo se calcula (D5a).** Diseño e implementación, en `plan`; quedan
   congelados con el `approve` firmado, junto con el texto de la tarea que los
   produjo, y `start` no los recalcula (si el inventario ya no tiene algo,
   `start` para con el arreglo). Revisión, en cada ronda de `review`, con el
   diff real, la configuración y el inventario de la base y la unión de las
   señales del texto aprobado y del vivo.

10. **Anotación en `roles.<rol>` (D8a).** Por rol: `modelo`, `agentes` (lista),
    `skills`, `regla` (lista de ids de fila de las tres tablas), `senales`
    (dominio y señal), `avisos`, el blob del texto de la tarea aprobado
    (`texto`) y el hash del inventario y del catálogo con los que se calculó;
    en revisión, además, `ronda`. Con esos datos cualquiera
    reproduce la selección con `seleccionar`.

11. **«Necesario para el proyecto» (F1-14).** Lo que `seleccionar` elige para
    una tarea es lo necesario para esa tarea: `tc2 doctor --recursos` lo lista
    y `start` para antes de tomarla si falta. Las skills de fase y los agentes
    de brainstorm del plugin son «obligatorio»; engram sigue siendo
    «conveniente». Este ADR no duplica `evaluar_recursos`: le da la lista. Es
    una ampliación de F1-14 (divergencia 9).

12. **Medición (D7a, RF-15).** La tabla de hallazgos de cada informe de
    revisión gana la columna **«Clase»** con la lista cerrada de la PoC R
    (puerta-seguridad, contrato, correccion, fidelidad-evidencia,
    reproducibilidad; precedencia en ese orden). F8-6 cruza por tarea: los
    hallazgos por revisora, por rol y por clase; los **escapados** (IMPORTANTES
    de una clase cuyo rol de diseño no se lanzó, o de un dominio con aviso
    `dominio-sin-revisora`); y los tokens y el resultado por modelo y rol. Con
    eso se recalibra la `tabla-dominios` (recall y precisión) y la
    `tabla-modelo`.

## Divergencias con el plan

1. **F1-9, mapa `roles`**: la plantilla tiene `roles.revision.agente`
   (singular) y `regla` como texto; pasa a `agentes` (lista) en los tres roles
   y `regla` a lista de ids de fila, y se añaden `senales`, `avisos`,
   `inventario`, `catalogo`, `texto` y, en revisión, `ronda`. F2-5 lo recoge.
2. **F1-11, bloque yaml (D5a de F1-11)**: era solo de revisoras; se amplía a
   toda skill y agente del plugin con `rol`, `etiquetas`, `dominios` y
   `prioridad`, y `catalogo-skills.yml` desaparece.
3. **F1-10, roles de diseño**: la `tabla-politica` da `fix` con 1 rol
   (arquitectura) y funcionalidad «1 a 4 según la complejidad»; con el suelo
   por dominio, una tarea de `fix` o de funcionalidad simple con
   `puertas-seguridad` lleva 2 roles y unificador. Es el endurecimiento que
   F1-10 dejó abierto para F1-15; la tabla de F1-10 no se toca porque da
   mínimos.
4. **F5-4, columna «Clase»**: el plan aprobado la asignaba a F5-4, pero F5-4 es
   el informe JSON de verificación y no lleva hallazgos; la columna va a los
   hallazgos estructurados de F6-5 y la agrega F8-6.
5. **Eliminación del desempate por modelo de la 0.16**
   (`plan-desempate-skill.ts`): el empate se resuelve por id lexicográfico con
   aviso (D3a). No se migra.
6. **Cifras preliminares del plan**: el plan leía 15 hallazgos de puerta o
   seguridad, 11 de ellos en planes de un rol, con TASK-012 en uno; la
   clasificación con lista cerrada y cita da tres en TASK-012 (IMP-2, un
   derivado que cambia el estado de una tarea; IMP-5, un check obligatorio que
   el PR deja verde; e IMP-1, caso límite que la revisión 1 pasó de
   correccion a puerta-seguridad por la precedencia de la lista, porque un
   `comprueba` pasaba de bloquear a avisar), y con ellos las cifras de
   puerta-seguridad de la PoC R. El plan también descartaba D1b diciendo que
   «seguridad» ya sumaba puntos a F1-13 y F1-18; no es así (ninguna de las
   dos tiene esa palabra) y el motivo se ha sustituido por el medido. El plan decía «como mucho 8 de 15» tareas con suelo; son 8, pero no las
   mismas que la lectura preliminar: entran F1-4 y F1-14 (falsos positivos) y
   sale F1-11 (falso negativo).
7. **Señal de ruta `.claude/**`** (D22): el plan no la listaba; se añade porque
   `settings.json` y los hooks cambian lo que el agente puede hacer, la pregunta
   que F1-10 dejó abierta. Solo añade el dominio; el carril mínimo no cambia.
8. **Muestra**: el estrato `fix` no puede salir de OpenDockTrading (0 tareas de
   tipo `fix`); su hueco lo ocupa `un-dominio`. Se decidió antes de ejecutar
   T, pero el historial no lo prueba (ver «Muestra de las 10 tareas»).
9. **F1-14, manifiesto de recursos**: F1-14 pone como «obligatorio» las skills
   de fase del plugin y como «necesario para el proyecto» lo que declara su
   configuración. Este ADR añade a «obligatorio» los agentes de brainstorm del
   plugin y crea un «necesario» **por tarea** (lo que `seleccionar` elige para
   ella), con `start` parando si falta. El contrato de `evaluar_recursos` no
   cambia, pero su lista de entrada sí.
10. **Texto de la tarea como señal**: el plan usaba las palabras del objetivo y
    de los criterios sin decir que son texto no confiable; la decisión 3 añade
    que solo suben (unión con el texto aprobado, rutas de la base en la
    revisión) y deja el resto como riesgo aceptado.

## Requisitos y evidencia

| Requisito | Decisión | Evidencia |
|---|---|---|
| RR-17, elección por tarea y rol solo entre lo instalado | decisiones 1, 2 y 5 | PoC I (inventario real) y S (quitar una skill la saca con aviso) |
| RR-17, nada «a ojo» | decisión 1; D2a, D3a | PoC S: guardia de auditoría con 0 eventos, grep del núcleo, empate total resuelto por id |
| RF-12, recomendar skills y agentes por rol | decisiones 4, 5 y 6 | PoC T: propuesta por rol en las 10 tareas |
| RF-13, modelo por complejidad y carril | decisión 7 | PoC S: el modelo cambia con carril y con complejidad en filas reales |
| RF-14, inventario y nunca algo que no existe | decisión 2 | PoC I y G (cada id de `tabla-seleccion` está en el inventario) |
| RF-15 y RR-05, medir el acierto | decisión 12 | PoC R (clases con cita) y D (recall y precisión declarados) |
| F1-9, contenido de `roles` | decisión 10 | divergencia 1 |
| F1-10, roles por carril y revisoras | decisiones 4 y 6 | PoC T (carril efectivo con sus funciones); divergencia 3 |
| F1-11 D5a, revisoras como skill del proyecto | decisiones 6 y 8 | divergencia 2 |
| F1-14, «necesario para el proyecto» | decisión 11 | contrato de `evaluar_recursos` sin cambios; divergencia 9 |
| Criterio 1 de la tarea, probadas a mano en 10 tareas | muestra por regla reproducible | PoC T; Carlos: «Coincide las 10» (`tarea.md` de TASK-073, «Decisiones de la persona», commit `5768e3c`): **cumplido** |
| Criterio 2, nada «a ojo» | decisiones 1 y 3 | PoC S: tabla interpretada, un caso por fila, guardia; el texto no confiable solo sube desde `approve` |

## Consecuencias

- **F3-9** construye el inventario de la decisión 2 y el error de `doctor`
  por un catálogo que nombra algo no instalado.
- **F4-6** implementa `seleccionar` y `dominios` con estas tablas, escribe
  `roles.<rol>` en `plan` y su criterio («en 10 tareas reales la propuesta
  coincide con la que haría una persona») tiene ya una primera medida: en la
  muestra de este ADR, Carlos dice que coinciden las diez.
- **F4-1** lanza los roles que da `seleccionar` (con el suelo por dominio).
- **F6-3** lanza las revisoras y el modelo de `roles.revision` con la
  configuración de la base; **F4-4** y **F1-16** consumen la misma selección.
- **F6-5** añade la columna «Clase» a los hallazgos; **F8-6** mide escapados,
  avisos y coste por modelo y rol, y recalibra las tablas.
- **F2-5** recoge el mapa `roles` ampliado en el modelo de tarea.
- **F3-8** (`doctor`) dice el valor efectivo de `modelos_permitidos`.

## Fuera de alcance

- El esquema exacto de `recursos:` (F2-5 con F1-11) y el informe JSON de
  verificación (F5-4).
- Escribir una revisora de puertas: se usa la que el proyecto declare.
- Si datos o migraciones deben tener suelo de roles: la Fase 1 no tuvo
  IMPORTANTES de ese dominio; F8-6 lo decide con datos.
- Lotes (F1-16) y estimación de tiempo (F1-17).

## No verificado

- **La comprobación a mano de las 10 tareas** la hizo Carlos («Coincide las
  10»); es el juicio de una persona sobre 10 tareas, no una medida, y no dice
  cómo se comporta la regla en otras.
- **La evasión por vocabulario antes de `approve`** y en `fix` o `errata` sin
  `approve`: el binario no la contiene (casos 2 y 5 de la PoC S); solo la
  persona que firma el plan o aprueba el PR. Que esa persona mire las señales
  que se le muestran no se puede verificar; F8-6 lo mide como escapados.
- **El acierto por modelo**: la 0.16 anota `modelo_sugerido`, no el modelo con
  que se ejecutó cada fase; la `tabla-modelo` es una propuesta sin datos de
  resultado por modelo. F8-6 lo mide.
- **La clasificación de los hallazgos** la hizo el implementador; la cita
  permite contrastarla, pero ninguna otra persona la ha repetido.
- **Muestra pequeña y dentro de muestra**: 15 tareas de un solo proyecto,
  todas ADR, con las que se escribió la lista de palabras; recall y precisión
  describen ese ajuste, no la generalización.
- **El coste del suelo** es una estimación con el coste medio por agente, no
  una medida.
- **Si algún consumidor (orquestador, revisor en CI) puede saltarse la
  selección**, y que el revisor en CI lea de verdad el inventario y la
  configuración de la base: no revisado (F6-3).
- **El inventario en Linux y macOS** y en un runner de CI: solo se leyó en esta
  máquina Windows; dos personas o una máquina y un runner con inventarios
  distintos dan selecciones distintas, que el hash de inventario anotado deja
  ver pero no evita.
- **`general-purpose`** se da por presente como agente integrado de Claude
  Code: no se lee del disco.
- **La reproducción de la PoC en otra máquina**: el log es determinista en
  esta (dos ejecuciones dan el mismo fichero), pero las secciones I, T y S
  dependen del inventario instalado; en otra máquina cambian el hash del
  inventario y, si cambia lo declarado, la propuesta.
