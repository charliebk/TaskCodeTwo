---
id: TASK-074
titulo: "F1-16 ADR lotes desatendidos con revision humana al final"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-074-f1-16-adr-lotes-desatendidos-con-revisio
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1214551
tokens_implementacion: 59106376
tokens_revision: 54281119
creado: 2026-11-14
actualizado: 2026-11-15
dependencias: [TASK-006, TASK-011]
---
## Objetivo

ADR: lotes desatendidos (RR-18): rama de lote, política de carriles, qué para el lote y qué aparta una tarea, presupuesto, reanudación, informe y su equivalente en GitLab

Tarea F1-16 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-5, F1-10.

Contexto (Carlos, 2026-11-14, RR-18, imprescindible): hoy se hace por instrucción al agente («arrancar consecutivamente TASK-xxx a TASK-yyy, sin trocear y aceptando las recomendaciones en todas las preguntas») en ToolsaWeb, OpenDockTrading y TaskCode, para listas de documentación o programación que no necesitan revisión humana hasta el final, o para dejar trabajo por la noche. Diseño propuesto (P13 del plan): rama `lote/<nombre>` desde develop; cada tarea pasa sus puertas automáticas y se integra en la rama del lote (la siguiente ve el código de la anterior); al final un único PR del lote contra develop que aprueba la persona. Solo carriles permitidos; una tarea en rojo o con un CRÍTICO/IMPORTANTE sin resolver se aparta con sus dependientes y el lote sigue; nunca se integra nada en rojo. Presupuesto de tiempo y tokens; `--reanudar` sin repetir; informe de la mañana (terminadas, apartadas y por qué, hallazgos sin corregir, tokens, tiempo). Lo ejecuta el CLI de forma determinista, no el agente. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 bis.

## Criterios de aceptacion
- [ ] Diseño probado a mano con un lote real de ToolsaWeb
- [ ] Cumple lo que pide la tarea F1-16 del plan y los requisitos que cita

## Autorrevision

- code-quality-reviewer — 8b7ecd7

## Rondas extra

- Ronda 3: Carlos (consigna «opcion recomendada» y fidelidad sobre coste); la ronda 2 encontro un IMPORTANTE introducido por la correccion (NUE-1, nucleo del manifiesto por lista blanca), ya corregido en d574e7f.
- Ronda 4: Carlos (consigna «opcion recomendada» y fidelidad sobre coste); la ronda 3 encontro dos IMPORTANTE introducidos por la correccion (NUE-5 lista de escrituras del CLI medida solo en ToolsaWeb, NUE-6 reversion parcial sin caso), ya corregidos en da557fc.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T22:31:25Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T22:40:40Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T22:40:41Z | start | automatico | persona | 0.16.1 |
| 2026-11-15T01:35:02Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T03:30:20Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T05:32:03Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T07:29:31Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T22:40:38Z | diseno | agente:aaf90b533374b7743 | 26 | 64483 | 568740 | 12650 | 20 | 645899 | - | - |
| 2026-11-14T22:40:39Z | diseno | agente:acee77252357bf7a7 | 24 | 41249 | 351481 | 12111 | 23 | 404865 | - | - |
| 2026-11-14T22:40:39Z | diseno | agente:a07b5e3cf1cddbf60 | 10 | 37027 | 113758 | 12992 | 8 | 163787 | - | - |
| 2026-11-15T01:35:02Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 20 | 74213 | 1609308 | 6203 | 2 | 1689744 | - | - |
| 2026-11-15T01:35:02Z | implementacion | agente:a8fc16ef560a0e868 | 358 | 704231 | 56499646 | 212397 | 23 | 57416632 | - | - |
| 2026-11-15T02:23:24Z | revision | agente:abdc898e2e310828d | 148 | 324074 | 17678060 | 79997 | 18 | 18082279 | 1 | - |
| 2026-11-15T04:37:41Z | revision | agente:acfb15b4baef812b8 | 138 | 257097 | 13283983 | 59074 | 13 | 13600292 | 2 | - |
| 2026-11-15T06:29:49Z | revision | agente:af1f04fcab8b9ff6f | 178 | 777213 | 12708471 | 54329 | 11 | 13540191 | 3 | - |
| 2026-11-15T08:33:54Z | revision | agente:a8db2a077311e16a0 | 116 | 715346 | 8311400 | 31495 | 12 | 9058357 | 4 | - |

## Verificacion ligera

- 2026-11-15T08:33:56Z: ronda 4; commit revisado d57f5cc2451a17816ee38f9763f150a3349ebf15; commit verificado fd5b6e8ac99dbac3e17823e6085d5b75f8673e90; ficheros: ninguno; corregidos: ninguno; pendientes: ninguno

## Resultado

- **Implementado**: ADR [F1-16](../../../docs/adr/F1-16-lotes-desatendidos.md) (aceptada) y su fila en `docs/adr/README.md`. `tc2 lote` determinista: bucle del CLI que reconstruye los hechos desde Git (primer padre de `lote/<nombre>`, ramas de tarea) y del diario `lotes/<nombre>.yml` y llama a la funcion pura `siguiente_paso`; operacion nueva `integrar-en-lote` (merge por API con el `sha` revisado como CAS) y ruleset `taskcodetwo-lote`; guarda del primer padre con comparacion del arbol de cada merge (`merge-tree`) y rutas que difieren; guarda de procedencia antes de `review` y de integrar; manifiesto con nucleo de la tarea por exclusion (solo fuera lo que escribe el CLI y el ciclo, medido en los tres repos) y duplicados que cambian el hash; carril del diff recalculado antes de cada ronda e integracion (`carril-subio`, `rutas_carril`); `escritura-no-declarada` para escrituras concurrentes; `develop-movido-por-agente`; salida de `lote-con-commits-ajenos` por reversion (committer persona de la lista) o `ajeno-aceptado` firmado por persona de la lista; presupuesto por lote y por tarea; tope de rondas sin ronda extra automatica; `[DECIDE CARLOS]` que no entra y nunca para el lote; `--reanudar` sin repetir; informe de la manana como descripcion del PR del lote; equivalente en GitLab.
- **Medido (PoC sobre copia de ToolsaWeb en 93c9618ac, 17 tareas reales: 15 merges de TASK-173..191 mas TASK-205 y TASK-210)**: control del metodo 219/219 arboles; 6/17 chocan por `hitos.md` con los derivados de la 0.16 (0 con la rama de estado); 17 integradas; tope 2 cierra con 7 MENOR sin corregir; cascada media 22,8 % (82,4 % con TASK-174 en rojo); 55 cortes reanudados sin repetir ni saltar puertas; 12 conflictos de develop al informe; 29 inyecciones de interferencia detectadas salvo C10b; 4/17 tareas cambiaron un fichero tras su ultima revision; 3/17 con `approve` firmado por persona. Escrituras del ciclo: 2394 commits `chore(TASK-NNN)` en los tres repos, 0 de las 1442 durante la tarea cambian el nucleo. 22 mutaciones de control en G.
- **Decisiones de la persona**: D1a..D8a; D2a, D6a y D7a preguntadas expresamente a Carlos («Apruebo»); sin troceo.
- **Divergencias** (en el ADR): F1-6 decision 4 (ninguna IA mergea en develop/main, no garantizado en privados gratuitos: se detecta despues); F1-3 (ramas de tarea desde `lote/*`); F1-4 (`integrar-en-lote`); F1-5 (ruleset `taskcodetwo-lote`); F1-7 (perfil para PR con base `lote/*`); F1-10 decisiones 5 y 6 (paradas fijas y `.gitlab-ci.yml`); F1-13 (rama protegida `lote/*` en GitLab); F1-11 D2a requisito previo de los lotes; P13; RF-16 citado no existe (es RF-27); forma real de la serie (D8a).
- **No verificado / riesgos aceptados**: C10b (escritura de la persona en el mismo fichero que el agente no se detecta); registro de escrituras del agente con Bash no concluyente; GitHub y GitLab reales (ruleset, merge por API con 0 aprobaciones, arbol del merge real frente a `merge-tree`, G09, checks con base `lote/*`); firma del merge; committer y `por` escribibles a mano (la garantia real seria la firma); tokens del agente sin terminal; otros canales de entrada del agente real; bloqueo portable en Windows.
- **Revision**: ronda 1 cambios (IMP-1 carril del diff, IMP-2 arbol del merge, IMP-3 manifiesto con blob entero, IMP-4 escrituras durante el agente, IMP-5 garantia de no merge en develop; MEN-1..7), corregido en 08a98c8; ronda 2 cambios (NUE-1 nucleo por lista blanca; NUE-2..4), corregido en d574e7f; ronda extra 3 cambios (NUE-5 lista de escrituras medida solo en ToolsaWeb, NUE-6 reversion parcial sin caso; NUE-7..9), corregido en da557fc; ronda extra 4 aprobada sin hallazgos. Rondas extra justificadas en `## Rondas extra`.
- **Para despues**: en ToolsaWeb, mover lo generado de `hitos.md` a la rama de estado antes del primer lote; firmar los planes sin `approve` de persona antes de lanzar un lote.
- Implementacion por un agente implementador; revision por cuatro agentes independientes. `taskctl doctor` con 0 errores. Pilotos solo leidos. Temporales borrados.
