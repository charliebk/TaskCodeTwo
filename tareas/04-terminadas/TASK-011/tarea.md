---
id: TASK-011
titulo: "F1-10 ADR carriles y modo automatico"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-011-f1-10-adr-carriles-y-modo-automatico
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 461024
tokens_implementacion: 9566139
tokens_revision: 12426821
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-006]
---
## Objetivo

ADR: carriles y modo automático (qué para y dónde) (P7)

Tarea F1-10 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-5.

## Criterios de aceptacion
- [x] Política por carril, configurable
- [x] Cumple lo que pide la tarea F1-10 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T09:09:11Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T09:12:57Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T09:12:58Z | start | automatico | persona | 0.16.1 |
| 2026-11-14T09:30:38Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T09:43:39Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T09:51:37Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T11:34:55Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T11:39:35Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T09:12:50Z | diseno | agente:ab43d0f671eafb514 | 18 | 76303 | 373195 | 11508 | 16 | 461024 | - | - |
| 2026-11-14T09:30:38Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 8 | 4633 | 1567218 | 2252 | 0 | 1574111 | - | - |
| 2026-11-14T09:30:38Z | implementacion | agente:a680e063f95f50532 | 96 | 200771 | 7709541 | 81620 | 18 | 7992028 | - | - |
| 2026-11-14T09:37:14Z | revision | agente:a319c3cadfb076d57 | 62 | 128372 | 4190788 | 26790 | 3 | 4346012 | 1 | - |
| 2026-11-14T09:47:28Z | revision | agente:ada6025bdb1180fa2 | 30 | 167075 | 1916134 | 29158 | 9 | 2112397 | 2 | - |
| 2026-11-14T09:55:35Z | revision | agente:a9be7fa0f720a02ab | 40 | 113244 | 2373887 | 24481 | 8 | 2511652 | 3 | - |
| 2026-11-14T11:39:05Z | revision | agente:a7cca49dfc7c1eb52 | 56 | 155141 | 3272443 | 29120 | 6 | 3456760 | 4 | - |

## Autorrevision

- code-quality-reviewer — 4f82305
- code-quality-reviewer — 232628f
- code-quality-reviewer — e8e3306

## Rondas extra

- Ronda 3: Carlos, con la consigna «responder la opcion recomendada»; la correccion de IMP-1 en la ronda 1 quedo incompleta (release contra main sin exigir head develop) y un IMPORTANTE se corrige siempre; es barato, asi que se verifica en una ronda mas.
- code-quality-reviewer — 4f79940
- Ronda 4: Carlos (2026-11-14: «me interesa que los ADR de este plugin sean lo mas fiel posibles a los datos recolectados»); la correccion de la ronda 2 introdujo IMP-2 (union de puertas sin la politica configurada), que se corrige y se verifica en una ronda mas.

## Verificacion ligera

- 2026-11-14T11:39:09Z: ronda 4; commit revisado 8ef8213643323201b94815602d928e4cffb6fb8a; commit verificado 466563dc8a98632b0dba6501e528258f3c511236; ficheros: docs/adr/F1-10-carriles-y-modo-automatico.md; corregidos: MEN-1; pendientes: ninguno

## Resultado

- **Implementado**: ADR [F1-10](../../../docs/adr/F1-10-carriles-y-modo-automatico.md) (aceptada) y su fila en `docs/adr/README.md`. Cuatro carriles ordenados (`errata < fix < funcionalidad < riesgo-alto`) mas release y hotfix por `tipo`; tabla unica de politica por carril con su funcion pura (`carrilMinimo`, `carrilEfectivo`, `politica`, `accion`) que extiende `siguiente --json`; configuracion que solo endurece (`politica_carriles`, `rutas_carril`) y se lee de la base; carril minimo por ficheros con los dos nombres de cada renombrado y suelo por primer componente de la ruta; release solo en un PR `develop`→`main` y hotfix solo desde `hotfix/*` contra `main`; las puertas de una fila de `main` son la union con las del carril efectivo ya endurecido por la configuracion; paradas fijas del automatico (`approve`, `merge-pr`, tag, tope de rondas, subida de carril). Tabla de 15 preguntas heredadas de F1-3/F1-5/F1-6/F1-8.
- **Cifras de la PoC** (commits fijos de ToolsaWeb, OpenDockTrading y TaskCodeTwo): carril minimo ToolsaWeb 160 tareas = 22 errata, 134 fix, 4 riesgo alto, 0 funcionalidad; OpenDockTrading 43 = 1/41/1/0. Paradas humanas por tarea hoy → nuevo (cota inferior): ToolsaWeb 0,85 → 2,52 (1,52), OpenDockTrading 2,12 → 3,19 (2,16), TaskCodeTwo 1,22 → 2,22 (1,22). En ToolsaWeb, 114 approve automaticos y 43 de persona sobre 156 `finish`.
- **Decisiones de la persona**: D0a no trocear (Carlos: «NO troceamos ninguna tarea»), D1a..D7a (consigna: opcion recomendada). Eleccion de quien orquesta, para que Carlos la revise: con la union de puertas toda release exige tambien `revision-ia` (F1-5 no la pedia); se mantiene por endurecer (divergencia 5 con F1-5).
- **Divergencias** (en el ADR): `politica` recibe `tipo`, base y rama; las paradas humanas por tarea suben (`merge-pr` de persona y approve firmado que la 0.16 se salta), en tension con RR-07; la cifra «hoy» es aproximada; A excluye `tareas/`; `complejidad` no entra en el carril (el numero de roles lo fija `plan`), en contra de la tabla de campos de F1-9.
- **No verificado**: `previous_filename`, el corte de 3000 ficheros y que `pull_request_target` lea la configuracion de la base (F2-6); que GitHub admita D7a; el carril declarado de las tareas de la 0.16 en C es una suposicion; las reglas de negocio de la tabla no las ha revisado un rol de dominio.
- **Revision** (4 rondas, dos extra): ronda 1 cambios (IMP-1 `tipo` de la tarea bajaba la puerta; MEN-1..4); ronda 2 cambios (IMP-1 incompleto: release sin exigir `develop`); ronda 3 extra cambios (IMP-2 introducido: la union no usaba la politica configurada); ronda 4 extra aprobada con correcciones: MEN-1 (cifra historica de la PoC C) corregido. Cada correccion de puertas abrio otra via para bajarlas: dato para F1-15 sobre roles en tareas de puertas y seguridad.
- **Coste**: ~21 M tokens entre diseno, implementacion y cuatro rondas de revision; Carlos prefiere fidelidad a los datos sobre el coste.
- Implementacion por un agente implementador; revision por cuatro agentes independientes. `taskctl doctor` con 0 errores. Temporales borrados.

## Duracion

- Estimada: 75 min
- Real (de la rama abierta al cierre con merge, sin pausas): 147 min
- Paso de la hora: 147 min reales frente a 75 estimados
- Cierre (finish): 7 s, de 2026-11-14T11:39:28Z a 2026-11-14T11:39:35Z
