# Roadmap de tareas

<!-- generado por taskctl; formato 1; no editar fuera de la zona manual -->

<!-- manual:inicio (se edita a mano; taskctl la conserva) -->
## Hitos

Las fases del [plan](../docs/plan/PLAN-TASKCODE-2.md); cada tarea lleva su F-id en el título. La Fase 0 (0.16.2) se hace en el repo TaskCode.

### H1 — Arranque y decisiones de arquitectura (ADR)
Sin código hasta cerrar las decisiones de fondo.

### H2 — Núcleo
Operaciones Git deterministas, rama de estado, IDs, adaptadores GitHub y GitLab, distribución.

### H3 — Comandos del flujo
init, new/import, next, roadmap, board/status, start, descartar, doctor.

### H4 — Diseño, aprobación y relevo
Brainstorm de roles, aprobación verificable, modo automático; relevo de la 0.16 a la 2.0 en este repo.

### H5 — Verificación sin redundancia
CI con perfiles, caché por SHA, runner propio.

### H6 — Revisión con identidad y búsqueda de errores
Revisor IA con identidad propia, revisoras por dominio, rondas incrementales.

### H7 — Integración y publicación
finish en develop, Resultado obligatorio, release y hotfix.

### H8 — Equipo, métricas e iteración del método
Métricas, presupuesto de tokens, ciclo de iteración, guía.

### H9 — Actualización de proyectos, piloto y 2.0.0
taskctl actualizar, E2E, seguridad, auditoría externa, piloto en OpenDockTrading y ToolsaWeb.
<!-- manual:fin -->

## Estado del trabajo

Orden en que se hacen las tareas, agrupadas por hito (campo `sprint` de cada tarea). Sale de las dependencias: ninguna tarea va antes que aquellas de las que depende y, a igualdad, va primero la del hito anterior (en el orden de los `### Hn — título` de la zona manual) y luego la de número menor. taskctl lo regenera en cada transición que cae en la rama de integración; solo la zona manual se edita a mano.

- **En marcha:** TASK-001 (en diseño)
- **Listas para empezar:** ninguna

## H1 — Arranque y decisiones de arquitectura (ADR)

0 de 15 terminadas.

| Paso | Tarea | Título | Complejidad | Depende de | Estado |
|---|---|---|---|---|---|
| 1 | TASK-001 | F1-0 Crear TaskCodeTwo | — | — | ▶ en diseño |
| 2 | TASK-002 | F1-1 ADR carpeta fija por tarea y rama de estado | — | TASK-001 | espera a TASK-001 |
| 3 | TASK-003 | F1-2 ADR escrituras atomicas, reserva de IDs y toma de tareas | — | TASK-002 | espera a TASK-002 |
| 4 | TASK-004 | F1-3 ADR flujo de ramas develop/main, rama por tarea, worktrees, release | — | TASK-001 | espera a TASK-001 |
| 5 | TASK-005 | F1-4 ADR capa de operaciones Git/GitHub deterministas y su contrato | — | TASK-004 | espera a TASK-004 |
| 6 | TASK-006 | F1-5 ADR puertas en GitHub | — | TASK-004 | espera a TASK-004 |
| 7 | TASK-007 | F1-6 ADR identidades del agente, del revisor y de la persona | — | TASK-006 | espera a TASK-006 |
| 8 | TASK-008 | F1-7 ADR verificacion en CI, perfiles, cache por SHA y runners | — | TASK-006 | espera a TASK-006 |
| 9 | TASK-009 | F1-8 ADR seguridad | — | TASK-007, TASK-008 | espera a TASK-007, TASK-008 |
| 10 | TASK-010 | F1-9 ADR formato de tarea, plan y especificacion | — | TASK-002 | espera a TASK-002 |
| 11 | TASK-011 | F1-10 ADR carriles y modo automatico | — | TASK-006 | espera a TASK-006 |
| 12 | TASK-012 | F1-11 ADR extensiones declaradas | — | TASK-002, TASK-008 | espera a TASK-002, TASK-008 |
| 13 | TASK-013 | F1-12 ADR actualizacion de proyectos entre versiones | — | TASK-002, TASK-010 | espera a TASK-002, TASK-010 |
| 14 | TASK-014 | F1-14 ADR distribucion e instalacion | — | TASK-001 | espera a TASK-001 |
| 15 | TASK-015 | F1-13 ADR adaptador de plataforma GitHub y GitLab | — | TASK-005 | espera a TASK-005 |

## H2 — Núcleo

0 de 12 terminadas.

| Paso | Tarea | Título | Complejidad | Depende de | Estado |
|---|---|---|---|---|---|
| 16 | TASK-016 | F2-1 Esqueleto: TypeScript, build reproducible | — | TASK-001, TASK-002, TASK-003, TASK-004, TASK-005, TASK-006, TASK-007, TASK-008, TASK-009, TASK-010, TASK-011, TASK-012, TASK-013, TASK-014, TASK-015 | espera a TASK-001, TASK-002, TASK-003, TASK-004, TASK-005, TASK-006, TASK-007, TASK-008, TASK-009, TASK-010, TASK-011, TASK-012, TASK-013, TASK-014, TASK-015 |
| 17 | TASK-017 | F2-2 Capa de operaciones Git deterministas | — | TASK-005, TASK-016 | espera a TASK-005, TASK-016 |
| 18 | TASK-018 | F2-3 Rama de estado | — | TASK-002, TASK-003, TASK-017 | espera a TASK-002, TASK-003, TASK-017 |
| 19 | TASK-019 | F2-4 Reserva de IDs por push atomico | — | TASK-018 | espera a TASK-018 |
| 20 | TASK-020 | F2-5 Modelo de tarea y parser/serializador de frontmatter YAML valido | — | TASK-010 | espera a TASK-010 |
| 21 | TASK-021 | F2-6 Adaptador GitHub | — | TASK-015, TASK-017 | espera a TASK-015, TASK-017 |
| 22 | TASK-022 | F2-9 Adaptador GitLab | — | TASK-015, TASK-017 | espera a TASK-015, TASK-017 |
| 23 | TASK-023 | F2-7 Configuracion minima con esquema validado y valores por defecto | — | TASK-011 | espera a TASK-011 |
| 24 | TASK-024 | F2-8 i18n de mensajes | — | TASK-016 | espera a TASK-016 |
| 25 | TASK-025 | F2-10 Build de ejecutables por sistema en la CI, firmados | — | TASK-014, TASK-016 | espera a TASK-014, TASK-016 |
| 26 | TASK-026 | F2-11 Instaladores de una linea | — | TASK-025 | espera a TASK-025 |
| 27 | TASK-027 | F2-12 taskctl actualizar --cli y aviso de version nueva | — | TASK-025 | espera a TASK-025 |

## H3 — Comandos del flujo

0 de 8 terminadas.

| Paso | Tarea | Título | Complejidad | Depende de | Estado |
|---|---|---|---|---|---|
| 28 | TASK-028 | F3-1 taskctl init / setup | — | TASK-018, TASK-021 | espera a TASK-018, TASK-021 |
| 29 | TASK-029 | F3-2 new e import en bloque con dependencias e hito | — | TASK-019, TASK-020 | espera a TASK-019, TASK-020 |
| 30 | TASK-030 | F3-3 next: lo listo para empezar por dependencias y prioridad | — | TASK-029 | espera a TASK-029 |
| 31 | TASK-031 | F3-4 roadmap: la vista para todos | — | TASK-018 | espera a TASK-018 |
| 32 | TASK-032 | F3-5 board y status | — | TASK-018, TASK-021 | espera a TASK-018, TASK-021 |
| 33 | TASK-033 | F3-6 start: toma atomica de la tarea, rama y worktree desde develop | — | TASK-017, TASK-018 | espera a TASK-017, TASK-018 |
| 34 | TASK-034 | F3-7 descartar y pausa | — | TASK-018 | espera a TASK-018 |
| 35 | TASK-035 | F3-8 doctor: entorno, rama de estado, permisos, rulesets y checks | — | TASK-028 | espera a TASK-028 |

## H4 — Diseño, aprobación y relevo

0 de 5 terminadas.

| Paso | Tarea | Título | Complejidad | Depende de | Estado |
|---|---|---|---|---|---|
| 36 | TASK-036 | F4-1 plan con brainstorm de roles | — | TASK-029 | espera a TASK-029 |
| 37 | TASK-037 | F4-2 approve con la identidad de la persona | — | TASK-007, TASK-036 | espera a TASK-007, TASK-036 |
| 38 | TASK-038 | F4-3 Aprobacion de un hito en lote | — | TASK-037 | espera a TASK-037 |
| 47 | TASK-039 | F4-4 Orquestador del modo automatico | — | TASK-033, TASK-037, TASK-048 | espera a TASK-033, TASK-037, TASK-048 |
| 64 | TASK-040 | F4-5 Punto de relevo | — | TASK-039, TASK-064 | espera a TASK-039, TASK-064 |

## H5 — Verificación sin redundancia

0 de 6 terminadas.

| Paso | Tarea | Título | Complejidad | Depende de | Estado |
|---|---|---|---|---|---|
| 39 | TASK-041 | F5-1 Workflow reutilizable de GitHub Actions y plantilla de GitLab | — | TASK-008 | espera a TASK-008 |
| 40 | TASK-042 | F5-2 Cache por SHA del arbol compartida | — | TASK-041 | espera a TASK-041 |
| 41 | TASK-043 | F5-3 Verificacion por impacto como paso de CI y completa en la cola | — | TASK-041 | espera a TASK-041 |
| 42 | TASK-044 | F5-4 Informe de verificacion en JSON para el revisor IA | — | TASK-041 | espera a TASK-041 |
| 43 | TASK-045 | F5-5 Runner self-hosted Windows por perfil | — | TASK-009, TASK-041 | espera a TASK-009, TASK-041 |
| 44 | TASK-046 | F5-6 Hook local rapido, opcional y con confianza | — | TASK-009 | espera a TASK-009 |

## H6 — Revisión con identidad y búsqueda de errores

0 de 7 terminadas.

| Paso | Tarea | Título | Complejidad | Depende de | Estado |
|---|---|---|---|---|---|
| 45 | TASK-047 | F6-1 Identidades: agente implementador | — | TASK-007 | espera a TASK-007 |
| 46 | TASK-048 | F6-2 review: abre el PR contra develop con la identidad del agente | — | TASK-021, TASK-047 | espera a TASK-021, TASK-047 |
| 48 | TASK-049 | F6-3 Revisor en CI con las skills de dominio de la 0.16 | — | TASK-044, TASK-048 | espera a TASK-044, TASK-048 |
| 49 | TASK-050 | F6-4 Busqueda de errores | — | TASK-049 | espera a TASK-049 |
| 50 | TASK-051 | F6-5 Hallazgos estructurados | — | TASK-050 | espera a TASK-050 |
| 51 | TASK-052 | F6-6 Rondas incrementales | — | TASK-051 | espera a TASK-051 |
| 52 | TASK-053 | F6-7 Defensa contra prompt injection en PR y tareas | — | TASK-009, TASK-049 | espera a TASK-009, TASK-049 |

## H7 — Integración y publicación

0 de 4 terminadas.

| Paso | Tarea | Título | Complejidad | Depende de | Estado |
|---|---|---|---|---|---|
| 53 | TASK-054 | F7-1 finish: merge en develop por la cola cuando pasan las puertas | — | TASK-018, TASK-051 | espera a TASK-018, TASK-051 |
| 54 | TASK-055 | F7-2 Resultado obligatorio en cada tarea | — | TASK-054 | espera a TASK-054 |
| 55 | TASK-056 | F7-3 release y hotfix con tag y notas desde los PR | — | TASK-017 | espera a TASK-017 |
| 56 | TASK-057 | F7-4 CHANGELOG generado al publicar, no en cada cierre | — | TASK-056 | espera a TASK-056 |

## H8 — Equipo, métricas e iteración del método

0 de 5 terminadas.

| Paso | Tarea | Título | Complejidad | Depende de | Estado |
|---|---|---|---|---|---|
| 57 | TASK-058 | F8-1 metricas: tiempo por fase, rondas, hallazgos, tokens y minutos | — | TASK-042, TASK-051 | espera a TASK-042, TASK-051 |
| 58 | TASK-059 | F8-2 Presupuesto de tokens por carril con desglose | — | TASK-058 | espera a TASK-058 |
| 59 | TASK-060 | F8-3 Ciclo de iteracion | — | TASK-058 | espera a TASK-058 |
| 60 | TASK-061 | F8-4 Espejo opcional en issues de GitHub o GitLab | — | TASK-032 | espera a TASK-032 |
| 61 | TASK-062 | F8-5 Guia de una pagina y referencia generada desde el codigo | — | TASK-028, TASK-029, TASK-030, TASK-031, TASK-032, TASK-033, TASK-034, TASK-035, TASK-054, TASK-055, TASK-056, TASK-057 | espera a TASK-028, TASK-029, TASK-030, TASK-031, TASK-032, TASK-033, TASK-034, TASK-035, TASK-054, TASK-055, TASK-056, TASK-057 |

## H9 — Actualización de proyectos, piloto y 2.0.0

0 de 10 terminadas.

| Paso | Tarea | Título | Complejidad | Depende de | Estado |
|---|---|---|---|---|---|
| 62 | TASK-063 | F9-1 taskctl actualizar --simular | — | TASK-013, TASK-020 | espera a TASK-013, TASK-020 |
| 63 | TASK-064 | F9-2 taskctl actualizar | — | TASK-018, TASK-063 | espera a TASK-018, TASK-063 |
| 65 | TASK-065 | F9-3 Actualizaciones entre versiones de la 2.x | — | TASK-064 | espera a TASK-064 |
| 66 | TASK-066 | F9-4 Suite E2E del flujo con dos personas a la vez | — | TASK-033, TASK-054, TASK-055, TASK-056, TASK-057 | espera a TASK-033, TASK-054, TASK-055, TASK-056, TASK-057 |
| 67 | TASK-067 | F9-5 Prueba de seguridad | — | TASK-053 | espera a TASK-053 |
| 68 | TASK-068 | F9-6 Auditoria externa repetida con el mismo encargo | — | TASK-066, TASK-067 | espera a TASK-066, TASK-067 |
| 69 | TASK-069 | F9-7 Piloto: OpenDockTrading actualizado y 10 tareas con la 2.0 | — | TASK-064, TASK-068 | espera a TASK-064, TASK-068 |
| 70 | TASK-070 | F9-8 ToolsaWeb actualizado y 10 tareas con la 2.0 | — | TASK-069 | espera a TASK-069 |
| 71 | TASK-071 | F9-9a Prueba de instalacion en maquinas limpias | — | TASK-026, TASK-062 | espera a TASK-026, TASK-062 |
| 72 | TASK-072 | F9-9 Publicar 2.0.0 | — | TASK-068, TASK-069, TASK-070, TASK-071 | espera a TASK-068, TASK-069, TASK-070, TASK-071 |
