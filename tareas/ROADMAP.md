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
### H10 — Memoria del proyecto
Índice de contexto propio (sustituye a engram), etiquetas, `tc2 buscar`, conversores de tareas y de engram, grafo y visor 3D con dockview.
<!-- manual:fin -->

## Estado del trabajo

Orden en que se hacen las tareas, agrupadas por hito (campo `sprint` de cada tarea). Sale de las dependencias: ninguna tarea va antes que aquellas de las que depende y, a igualdad, va primero la del hito anterior (en el orden de los `### Hn — título` de la zona manual) y luego la de número menor. taskctl lo regenera en cada transición que cae en la rama de integración; solo la zona manual se edita a mano.

- **En marcha:** ninguna
- **Listas para empezar:** TASK-027, TASK-076, TASK-077, TASK-028, TASK-029, TASK-031, TASK-032, TASK-033, TASK-034, TASK-080, TASK-041, TASK-046, TASK-047, TASK-056, TASK-063, TASK-096

## H1 — Arranque y decisiones de arquitectura (ADR)

20 de 20 terminadas.

| Paso | Tarea | Título | Complejidad | Depende de | Estado |
|---|---|---|---|---|---|
| — | TASK-001 | F1-0 Crear TaskCodeTwo | — | — | ✅ terminada |
| — | TASK-002 | F1-1 ADR carpeta fija por tarea y rama de estado | — | TASK-001 | ✅ terminada |
| — | TASK-003 | F1-2 ADR escrituras atomicas, reserva de IDs y toma de tareas | — | TASK-002 | ✅ terminada |
| — | TASK-004 | F1-3 ADR flujo de ramas develop/main, rama por tarea, worktrees, release | — | TASK-001 | ✅ terminada |
| — | TASK-005 | F1-4 ADR capa de operaciones Git/GitHub deterministas y su contrato | — | TASK-004 | ✅ terminada |
| — | TASK-006 | F1-5 ADR puertas en GitHub | — | TASK-004 | ✅ terminada |
| — | TASK-007 | F1-6 ADR identidades del agente, del revisor y de la persona | — | TASK-006 | ✅ terminada |
| — | TASK-008 | F1-7 ADR verificacion en CI, perfiles, cache por SHA y runners | — | TASK-006 | ✅ terminada |
| — | TASK-009 | F1-8 ADR seguridad | — | TASK-007, TASK-008 | ✅ terminada |
| — | TASK-010 | F1-9 ADR formato de tarea, plan y especificacion | — | TASK-002 | ✅ terminada |
| — | TASK-011 | F1-10 ADR carriles y modo automatico | — | TASK-006 | ✅ terminada |
| — | TASK-012 | F1-11 ADR extensiones declaradas | — | TASK-002, TASK-008 | ✅ terminada |
| — | TASK-013 | F1-12 ADR actualizacion de proyectos entre versiones | — | TASK-002, TASK-010 | ✅ terminada |
| — | TASK-014 | F1-14 ADR distribucion e instalacion | — | TASK-001 | ✅ terminada |
| — | TASK-015 | F1-13 ADR adaptador de plataforma GitHub y GitLab | — | TASK-005 | ✅ terminada |
| — | TASK-073 | F1-15 ADR seleccion de skills, agentes y modelo por tarea y rol | — | TASK-010, TASK-011 | ✅ terminada |
| — | TASK-074 | F1-16 ADR lotes desatendidos con revision humana al final | — | TASK-006, TASK-011 | ✅ terminada |
| — | TASK-075 | F1-17 ADR progreso en vivo y estimacion de tiempo | — | TASK-010, TASK-011 | ✅ terminada |
| — | TASK-094 | F1-18 ADR hito de decisiones (ADR) en init y en actualizar | — | TASK-012, TASK-013 | ✅ terminada |
| — | TASK-095 | F1-19 ADR memoria del proyecto: indice de contexto, etiquetas y busqueda | — | TASK-002, TASK-010, TASK-012, TASK-014 | ✅ terminada |

## H2 — Núcleo

11 de 15 terminadas.

| Paso | Tarea | Título | Complejidad | Depende de | Estado |
|---|---|---|---|---|---|
| — | TASK-016 | F2-1 Esqueleto: Rust, build reproducible | — | TASK-001, TASK-002, TASK-003, TASK-004, TASK-005, TASK-006, TASK-007, TASK-008, TASK-009, TASK-010, TASK-011, TASK-012, TASK-013, TASK-014, TASK-015, TASK-073, TASK-074, TASK-075, TASK-094 | ✅ terminada |
| — | TASK-017 | F2-2 Capa de operaciones Git deterministas | — | TASK-005, TASK-016 | ✅ terminada |
| — | TASK-018 | F2-3 Rama de estado | — | TASK-002, TASK-003, TASK-017 | ✅ terminada |
| — | TASK-019 | F2-4 Reserva de IDs por push atomico | — | TASK-018 | ✅ terminada |
| — | TASK-020 | F2-5 Modelo de tarea y parser/serializador de frontmatter YAML valido | — | TASK-010 | ✅ terminada |
| — | TASK-021 | F2-6 Adaptador GitHub | — | TASK-015, TASK-017 | ✅ terminada |
| — | TASK-022 | F2-9 Adaptador GitLab | — | TASK-015, TASK-017 | ✅ terminada |
| — | TASK-023 | F2-7 Configuracion minima con esquema validado y valores por defecto | — | TASK-011 | ✅ terminada |
| — | TASK-024 | F2-8 i18n de mensajes | — | TASK-016 | ✅ terminada |
| — | TASK-025 | F2-10 Build de ejecutables por sistema en la CI, firmados | — | TASK-014, TASK-016 | ✅ terminada |
| — | TASK-026 | F2-11 Instaladores de una linea | — | TASK-025 | ✅ terminada |
| 1 | TASK-027 | F2-12 taskctl actualizar --cli y aviso de version nueva | — | TASK-025 | lista para empezar |
| 2 | TASK-076 | F2-4b Nombres unicos de tarea y rama | — | TASK-019 | lista para empezar |
| 3 | TASK-077 | F2-13 Identidad de la persona desde la plataforma | — | TASK-007, TASK-021, TASK-022 | lista para empezar |
| 4 | TASK-078 | F2-14 taskctl desinstalar: quitar TaskCodeTwo de forma limpia | — | TASK-018, TASK-025, TASK-027 | espera a TASK-027 |

## H3 — Comandos del flujo

0 de 14 terminadas.

| Paso | Tarea | Título | Complejidad | Depende de | Estado |
|---|---|---|---|---|---|
| 5 | TASK-028 | F3-1 taskctl init / setup | — | TASK-018, TASK-021 | lista para empezar |
| 6 | TASK-029 | F3-2 new e import en bloque con dependencias e hito | — | TASK-019, TASK-020 | lista para empezar |
| 7 | TASK-030 | F3-3 next: lo listo para empezar por dependencias y prioridad | — | TASK-029 | espera a TASK-029 |
| 8 | TASK-031 | F3-4 roadmap: la vista para todos | — | TASK-018 | lista para empezar |
| 9 | TASK-032 | F3-5 board y status | — | TASK-018, TASK-021 | lista para empezar |
| 10 | TASK-033 | F3-6 start: toma atomica de la tarea, rama y worktree desde develop | — | TASK-017, TASK-018 | lista para empezar |
| 11 | TASK-034 | F3-7 descartar y pausa | — | TASK-018 | lista para empezar |
| 12 | TASK-035 | F3-8 doctor: entorno, rama de estado, permisos, rulesets y checks | — | TASK-028 | espera a TASK-028 |
| 13 | TASK-079 | F3-5b status y roadmap siempre frescos | — | TASK-032 | espera a TASK-032 |
| 14 | TASK-080 | F3-9 Inventario de skills y agentes instalados | — | TASK-016, TASK-073 | lista para empezar |
| 15 | TASK-081 | F3-10 Liberar y reasignar tareas abandonadas | — | TASK-033 | espera a TASK-033 |
| 16 | TASK-083 | F3-13 Estimacion por fase de cada tarea al planificar | — | TASK-029, TASK-075 | espera a TASK-029 |
| 17 | TASK-084 | F3-12 Progreso en vivo: fases que faltan y tiempo restante | — | TASK-018, TASK-083 | espera a TASK-083 |
| 19 | TASK-082 | F3-11 Aviso de solape entre tareas en curso | — | TASK-033, TASK-036 | espera a TASK-033, TASK-036 |

## H4 — Diseño, aprobación y relevo

0 de 9 terminadas.

| Paso | Tarea | Título | Complejidad | Depende de | Estado |
|---|---|---|---|---|---|
| 18 | TASK-036 | F4-1 plan con brainstorm de roles | — | TASK-029 | espera a TASK-029 |
| 20 | TASK-037 | F4-2 approve con la identidad de la persona | — | TASK-007, TASK-036 | espera a TASK-036 |
| 21 | TASK-038 | F4-3 Aprobacion de un hito en lote | — | TASK-037 | espera a TASK-037 |
| 31 | TASK-085 | F4-6 plan propone skills, agentes y modelo por rol | — | TASK-036, TASK-049, TASK-080 | espera a TASK-036, TASK-049, TASK-080 |
| 32 | TASK-039 | F4-4 Orquestador del modo automatico | — | TASK-033, TASK-037, TASK-048, TASK-085 | espera a TASK-033, TASK-037, TASK-048, TASK-085 |
| 33 | TASK-086 | F4-7 taskctl lote: secuencia de tareas sin preguntas | — | TASK-039, TASK-074, TASK-085 | espera a TASK-039, TASK-085 |
| 34 | TASK-088 | F4-9 Progreso del lote en el terminal | — | TASK-084, TASK-086 | espera a TASK-084, TASK-086 |
| 44 | TASK-087 | F4-8 Fallos, presupuesto, reanudacion e informe del lote | — | TASK-058, TASK-086 | espera a TASK-058, TASK-086 |
| 58 | TASK-040 | F4-5 Punto de relevo | — | TASK-039, TASK-064, TASK-069 | espera a TASK-039, TASK-064, TASK-069 |

## H5 — Verificación sin redundancia

0 de 6 terminadas.

| Paso | Tarea | Título | Complejidad | Depende de | Estado |
|---|---|---|---|---|---|
| 22 | TASK-041 | F5-1 Workflow reutilizable de GitHub Actions y plantilla de GitLab | — | TASK-008 | lista para empezar |
| 23 | TASK-042 | F5-2 Cache por SHA del arbol compartida | — | TASK-041 | espera a TASK-041 |
| 24 | TASK-043 | F5-3 Verificacion por impacto como paso de CI y completa en la cola | — | TASK-041 | espera a TASK-041 |
| 25 | TASK-044 | F5-4 Informe de verificacion en JSON para el revisor IA | — | TASK-041 | espera a TASK-041 |
| 26 | TASK-045 | F5-5 Runner self-hosted Windows por perfil | — | TASK-009, TASK-041 | espera a TASK-041 |
| 27 | TASK-046 | F5-6 Hook local rapido, opcional y con confianza | — | TASK-009 | lista para empezar |

## H6 — Revisión con identidad y búsqueda de errores

0 de 7 terminadas.

| Paso | Tarea | Título | Complejidad | Depende de | Estado |
|---|---|---|---|---|---|
| 28 | TASK-047 | F6-1 Identidades: agente implementador | — | TASK-007 | lista para empezar |
| 29 | TASK-048 | F6-2 review: abre el PR contra develop con la identidad del agente | — | TASK-021, TASK-047 | espera a TASK-047 |
| 30 | TASK-049 | F6-3 Revisor en CI con las skills de dominio de la 0.16 | — | TASK-044, TASK-048 | espera a TASK-044, TASK-048 |
| 35 | TASK-050 | F6-4 Busqueda de errores | — | TASK-049 | espera a TASK-049 |
| 36 | TASK-051 | F6-5 Hallazgos estructurados | — | TASK-050 | espera a TASK-050 |
| 37 | TASK-052 | F6-6 Rondas incrementales | — | TASK-051 | espera a TASK-051 |
| 38 | TASK-053 | F6-7 Defensa contra prompt injection en PR y tareas | — | TASK-009, TASK-049 | espera a TASK-049 |

## H7 — Integración y publicación

0 de 4 terminadas.

| Paso | Tarea | Título | Complejidad | Depende de | Estado |
|---|---|---|---|---|---|
| 39 | TASK-054 | F7-1 finish: merge en develop por la cola cuando pasan las puertas | — | TASK-018, TASK-051 | espera a TASK-051 |
| 40 | TASK-055 | F7-2 Resultado obligatorio en cada tarea | — | TASK-054 | espera a TASK-054 |
| 41 | TASK-056 | F7-3 release y hotfix con tag y notas desde los PR | — | TASK-017 | lista para empezar |
| 42 | TASK-057 | F7-4 CHANGELOG generado al publicar, no en cada cierre | — | TASK-056 | espera a TASK-056 |

## H8 — Equipo, métricas e iteración del método

0 de 7 terminadas.

| Paso | Tarea | Título | Complejidad | Depende de | Estado |
|---|---|---|---|---|---|
| 43 | TASK-058 | F8-1 metricas: tiempo por fase, rondas, hallazgos, tokens y minutos | — | TASK-042, TASK-051 | espera a TASK-042, TASK-051 |
| 45 | TASK-059 | F8-2 Presupuesto de tokens por carril con desglose | — | TASK-058 | espera a TASK-058 |
| 46 | TASK-060 | F8-3 Ciclo de iteracion | — | TASK-058 | espera a TASK-058 |
| 47 | TASK-061 | F8-4 Espejo opcional en issues de GitHub o GitLab | — | TASK-032 | espera a TASK-032 |
| 48 | TASK-062 | F8-5 Guia de una pagina y referencia generada desde el codigo | — | TASK-028, TASK-029, TASK-030, TASK-031, TASK-032, TASK-033, TASK-034, TASK-035, TASK-054, TASK-055, TASK-056, TASK-057, TASK-079, TASK-080, TASK-081, TASK-082, TASK-083, TASK-084 | espera a TASK-028, TASK-029, TASK-030, TASK-031, TASK-032, TASK-033, TASK-034, TASK-035, TASK-054, TASK-055, TASK-056, TASK-057, TASK-079, TASK-080, TASK-081, TASK-082, TASK-083, TASK-084 |
| 49 | TASK-089 | F8-6 Medicion del acierto de skills, agentes y modelo | — | TASK-058, TASK-085 | espera a TASK-058, TASK-085 |
| 50 | TASK-090 | F8-7 Metricas finales de cada tarea y de cada lote | — | TASK-058, TASK-083 | espera a TASK-058, TASK-083 |

## H9 — Actualización de proyectos, piloto y 2.0.0

0 de 13 terminadas.

| Paso | Tarea | Título | Complejidad | Depende de | Estado |
|---|---|---|---|---|---|
| 51 | TASK-063 | F9-1 taskctl actualizar --simular | — | TASK-013, TASK-020 | lista para empezar |
| 52 | TASK-064 | F9-2 taskctl actualizar | — | TASK-018, TASK-063 | espera a TASK-063 |
| 53 | TASK-065 | F9-3 Actualizaciones entre versiones de la 2.x | — | TASK-064 | espera a TASK-064 |
| 54 | TASK-066 | F9-4 Suite E2E del flujo con dos personas a la vez | — | TASK-033, TASK-054, TASK-055, TASK-056, TASK-057, TASK-076, TASK-077, TASK-081, TASK-082 | espera a TASK-033, TASK-054, TASK-055, TASK-056, TASK-057, TASK-076, TASK-077, TASK-081, TASK-082 |
| 55 | TASK-067 | F9-5 Prueba de seguridad | — | TASK-053 | espera a TASK-053 |
| 56 | TASK-068 | F9-6 Auditoria externa repetida con el mismo encargo | — | TASK-066, TASK-067 | espera a TASK-066, TASK-067 |
| 57 | TASK-069 | F9-7 Piloto: OpenDockTrading actualizado y 10 tareas con la 2.0 | — | TASK-064 | espera a TASK-064 |
| 59 | TASK-070 | F9-8 ToolsaWeb actualizado y 10 tareas con la 2.0 | — | TASK-069, TASK-040 | espera a TASK-069, TASK-040 |
| 60 | TASK-071 | F9-9a Prueba de instalacion en maquinas limpias | — | TASK-026, TASK-062 | espera a TASK-062 |
| 61 | TASK-091 | F9-4b Prueba nocturna de un lote de 10 tareas | — | TASK-066, TASK-087 | espera a TASK-066, TASK-087 |
| 62 | TASK-092 | F9-8b Piloto final: TaskCodeTwo con la 2.0 completa | — | TASK-040, TASK-070 | espera a TASK-040, TASK-070 |
| 63 | TASK-093 | F9-10 Desinstalacion limpia de TaskCode 0.16 | — | TASK-013, TASK-064 | espera a TASK-064 |
| 64 | TASK-072 | F9-9 Publicar 2.0.0 | — | TASK-068, TASK-069, TASK-070, TASK-071, TASK-092, TASK-093 | espera a TASK-068, TASK-069, TASK-070, TASK-071, TASK-092, TASK-093 |

## H10 — Memoria del proyecto

0 de 17 terminadas.

| Paso | Tarea | Título | Complejidad | Depende de | Estado |
|---|---|---|---|---|---|
| 65 | TASK-096 | F10-1 Indice SQLite reconstruible | — | TASK-095, TASK-020 | lista para empezar |
| 66 | TASK-097 | F10-2 Frescura del indice por SHA | — | TASK-096, TASK-018 | espera a TASK-096 |
| 67 | TASK-098 | F10-3 Conversor de las tareas existentes al indice | — | TASK-096 | espera a TASK-096 |
| 68 | TASK-099 | F10-4 Ficheros tocados en finish | — | TASK-098, TASK-054 | espera a TASK-098, TASK-054 |
| 69 | TASK-100 | F10-5 Conversor de engram por proyecto | — | TASK-096, TASK-098 | espera a TASK-096, TASK-098 |
| 70 | TASK-101 | F10-6 tc2 buscar | — | TASK-097, TASK-098 | espera a TASK-097, TASK-098 |
| 71 | TASK-102 | F10-7 El plugin busca en el indice | — | TASK-101, TASK-035 | espera a TASK-101, TASK-035 |
| 72 | TASK-103 | F10-8 Etiquetado determinista | — | TASK-098 | espera a TASK-098 |
| 73 | TASK-104 | F10-9 Vocabulario cerrado y tags doctor | — | TASK-103 | espera a TASK-103 |
| 74 | TASK-105 | F10-10 Etiquetado lexico e IA acotada en finish | — | TASK-104, TASK-054 | espera a TASK-104, TASK-054 |
| 75 | TASK-106 | F10-11 Relaciones derivadas entre tareas | — | TASK-098, TASK-103 | espera a TASK-098, TASK-103 |
| 76 | TASK-107 | F10-12 Grafo de tareas, etiquetas y ficheros en JSON | — | TASK-106 | espera a TASK-106 |
| 77 | TASK-108 | F10-13 Medicion y retirada de engram | — | TASK-100, TASK-101, TASK-102, TASK-103, TASK-104, TASK-105, TASK-106, TASK-107 | espera a TASK-100, TASK-101, TASK-102, TASK-103, TASK-104, TASK-105, TASK-106, TASK-107 |
| 78 | TASK-109 | F10-14 PoC del motor 3D del visor | — | TASK-107 | espera a TASK-107 |
| 79 | TASK-110 | F10-15 tc2 visor: servidor local y canal de eventos | — | TASK-097, TASK-101, TASK-109 | espera a TASK-097, TASK-101, TASK-109 |
| 80 | TASK-111 | F10-16 App Vue del visor con dockview | — | TASK-110 | espera a TASK-110 |
| 81 | TASK-112 | F10-17 Efectos del visor y panel de actividad en tiempo real | — | TASK-111 | espera a TASK-111 |
