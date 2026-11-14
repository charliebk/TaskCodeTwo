---
id: TASK-073
titulo: "F1-15 ADR seleccion de skills, agentes y modelo por tarea y rol"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-073-f1-15-adr-seleccion-de-skills-agentes-y
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 2497389
tokens_implementacion: 38239146
tokens_revision: 18975945
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-010, TASK-011]
---
## Objetivo

ADR: selección de skills, agentes y modelo por tarea y por rol (RR-17): fuentes (catálogo del proyecto, skills y agentes instalados, revisoras del plugin), reglas deterministas por etiquetas, rutas, dominio, complejidad y carril, qué se anota en la tarea y cómo se mide el acierto

Tarea F1-15 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-9, F1-10.

Contexto (Carlos, 2026-11-14, RR-17, imprescindible): para cada tarea y cada rol (diseño, implementación, revisión) hay que elegir las skills y los agentes adecuados y el modelo según complejidad y carril, solo entre lo que está instalado en la máquina y el proyecto, anotar la elección en la tarea y medir si acierta (hallazgos por revisora y coste por modelo). En la 0.16 existe con `catalogo-skills.yml`, `skills_recomendados` y `modelo_sugerido`, pero se usó en 21 / 0 / 2 tareas de ToolsaWeb / OpenDockTrading / TaskCode porque el catálogo apenas se configuró: hoy la elección la hace el agente sin criterio registrado. Nada debe decidirlo el modelo «a ojo»: reglas deterministas. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §1–4.

## Criterios de aceptacion
- [ ] Reglas probadas a mano sobre 10 tareas reales de ToolsaWeb y OpenDockTrading
- [ ] nada lo decide el modelo «a ojo»
- [ ] Cumple lo que pide la tarea F1-15 del plan y los requisitos que cita

## Decisiones de la persona

- 2026-11-14, Carlos, comprobacion a mano (D10a) de la tabla de las 10 tareas reales de F1-15 (ToolsaWeb TASK-306, 399, 403, 299, 402; OpenDockTrading TASK-063, 014, 067, 069, 096): "Coincide las 10". Las 10 propuestas (roles de diseno, revisoras y modelo por rol) coinciden con lo que habria elegido.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T20:39:39Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T20:49:09Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T20:49:11Z | start | automatico | persona | 0.16.1 |
| 2026-11-14T21:24:48Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T22:07:30Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T22:31:03Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T20:49:08Z | diseno | agente:ab10be6189a7b5c59 | 56 | 142811 | 2324615 | 29907 | 48 | 2497389 | - | - |
| 2026-11-14T21:24:48Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 8 | 16061 | 1155072 | 3582 | 0 | 1174723 | - | - |
| 2026-11-14T21:24:48Z | implementacion | agente:af178d5c044e54716 | 124 | 335797 | 12180069 | 132232 | 16 | 12648222 | - | - |
| 2026-11-14T22:30:20Z | revision | agente:aad2a4ecce31d6d50 | 138 | 244179 | 13839966 | 68910 | 7 | 14153193 | 1 | - |
| 2026-11-14T22:30:21Z | revision | agente:af545427dc32b577b | 66 | 138799 | 4656741 | 27146 | 15 | 4822752 | 2 | - |
| 2026-11-14T22:30:22Z | implementacion | agente:dif:af178d5c044e54716 | 118 | 975194 | 23344602 | 96287 | 25 | 24416201 | - | - |

## Autorrevision

- code-quality-reviewer — 90330c6
- code-quality-reviewer — 806f024

## Verificacion ligera

- 2026-11-14T22:30:19Z: ronda 2; commit revisado 3a11b7d5f06ab04c7f968196e2ff98b1822451a3; commit verificado 71bc716772fd994d3a40b5db6793621e5d5ff7a0; ficheros: docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md; corregidos: MEN-8, MEN-11; pendientes: MEN-9, MEN-10

## Resultado

- **Implementado**: ADR [F1-15](../../../docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md) (aceptada) y su fila en `docs/adr/README.md`. Funcion pura `seleccionar(rol, tarea, politica, complejidad, cambios, inventario, catalogo, config)` que interpreta `tabla-seleccion` (vocabulario cerrado), `tabla-modelo` y `tabla-dominios`; nunca llama a un modelo (desempate por `id` lexicografico con aviso; se elimina el desempate por modelo de la 0.16); solo elige entre lo instalado (inventario de F3-9); roles de diseno = max(carril, complejidad, suelo por dominio: 2 roles mas unificador si salta `puertas-seguridad`); revisoras tambien por dominio, calculadas con config e inventario de la base; modelo por carril, complejidad y rol con `modelos_permitidos` que solo restringe; anotacion en `roles.<rol>` con regla, senales y hashes; diseno e implementacion congelados en `approve`; el texto de la tarea es no confiable y solo sube (dominios y suelo visibles en `approve`, texto aprobado congelado, rutas del diff en minusculas con la config de la base); medicion con columna «Clase» de lista cerrada.
- **Medido**: 33 IMPORTANTES distintos en las 15 tareas de la Fase 1, clasificados con cita (puerta-seguridad 17, fidelidad-evidencia 6, correccion 5, contrato 3, reproducibilidad 2); 13 de los 17 de puerta-seguridad en planes de un solo rol (escapados); la heuristica de la 0.16 reimplementada coincide en 15/15 (14 «simple»); revisor generico en las 15. Detector: recall 7/8 y precision 7/9 (dentro de muestra). El suelo subiria a 2 roles en 8 de 15 tareas, unos 1,5 M tokens por tarea (10,7 % de la mediana de un ADR). Inventario de esta maquina: 99 entradas (53 skills, 46 agentes), 13 con metadatos; `security-review` y `security-auditor` instaladas sin declarar. Lista de riesgo de la 0.16 frente a palabras de puertas: 14 frente a 15, solo comparten `seguridad`. Cobertura de reglas: 50/50 variaciones de atomos detectadas; 26 mutaciones de control en G.
- **Comprobacion a mano (criterio 1)**: Carlos marco las 10 tareas reales (5 ToolsaWeb, 5 OpenDockTrading, muestreo estratificado reproducible) y coinciden las 10 (ver `## Decisiones de la persona`); las correcciones posteriores no cambiaron ninguna de las 10 filas (comprobado por el revisor de la ronda 2 y por la PoC).
- **Decisiones de la persona**: D1a..D10a (consigna: opcion recomendada); sin troceo; comprobacion a mano de Carlos.
- **Divergencias** (en el ADR): F1-9 (`roles.<rol>.agentes` como lista, `regla` lista, senales, avisos, hashes, ronda, `texto` aprobado); F1-11 (bloque yaml ampliado a toda skill y agente; desaparece `catalogo-skills.yml`); F1-10 (el suelo sube `fix` y funcionalidad simple a 2 roles con `puertas-seguridad`); F5-4 → F6-5/F8-6 para la columna «Clase»; F1-14 (brainstorm obligatorio y «necesario» por tarea); desempate por modelo de la 0.16 eliminado; senal de ruta `.claude/**`.
- **No verificado / riesgos aceptados**: evasion por vocabulario antes de `approve` (solo la contiene la persona) y en un `fix` sin `approve` cuyo diff solo toca `src/`; acierto por modelo (la 0.16 solo anota `modelo_sugerido`); muestra pequena (15 tareas, un proyecto, todas ADR); coste del suelo estimado con medias; que el orquestador o el revisor en CI no se salten la seleccion ni lean de verdad la base (F4-4, F6-3); inventario solo en esta maquina Windows.
- **Revision**: ronda 1 cambios (IMP-1 motivo de D1b no sostenido por los datos; IMP-2 la PoC no leia `tabla-seleccion` y varias reglas no se ejercitaban; IMP-3 evasion del suelo por vocabulario; MEN-1..7), corregido en 806f024; ronda 2 aprobada con correcciones; MEN-8..11 (lista real de la 0.16, cobertura por atomo, ruta sin distinguir mayusculas, etiquetas aprobadas) corregidos en 71bc716 (`verificar-menores` no reconoce MEN-9 y MEN-10 por vivir en `poc/`, ver `revision/menores-pendientes.md`).
- Implementacion por un agente implementador; revision por dos agentes independientes. `taskctl doctor` con 0 errores. Pilotos solo leidos. Temporales borrados.

## Duracion

- Estimada: 150 min
- Real (de la rama abierta al cierre con merge, sin pausas): 102 min
- Paso de la hora: 102 min reales frente a 150 estimados
- Cierre (finish): 12 s, de 2026-11-14T22:30:51Z a 2026-11-14T22:31:03Z
