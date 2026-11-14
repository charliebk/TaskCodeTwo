---
id: TASK-009
titulo: "F1-8 ADR seguridad"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-009-f1-8-adr-seguridad
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 578544
tokens_implementacion: 11528585
tokens_revision: 6974161
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-007, TASK-008]
---
## Objetivo

ADR: seguridad (qué corre en local y en CI, confianza, prompt injection, runner propio)

Tarea F1-8 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-6, F1-7.

## Criterios de aceptacion
- [x] Modelo de amenazas con mitigaciones
- [x] Cumple lo que pide la tarea F1-8 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T07:43:12Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T07:53:47Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T07:54:17Z | start | automatico | persona | 0.16.1 |
| 2026-11-14T08:17:36Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T08:28:50Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T08:33:25Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T08:37:32Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T07:53:46Z | diseno | agente:a75a8274814f57c60,a1721f1a4019a860a,a39127c0f46feec50 | 42 | 139358 | 412897 | 26247 | 28 | 578544 | - | - |
| 2026-11-14T08:17:36Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 10 | 5342 | 1829306 | 2478 | 0 | 1837136 | - | - |
| 2026-11-14T08:17:36Z | implementacion | agente:ae0317153b121b4af | 98 | 276273 | 9312752 | 102326 | 19 | 9691449 | - | - |
| 2026-11-14T08:22:42Z | revision | agente:ad7595ebc10b9a87d | 44 | 164649 | 2786814 | 25127 | 11 | 2976634 | 1 | - |
| 2026-11-14T08:33:09Z | revision | agente:a29cafed91767d7e4 | 30 | 182125 | 1939960 | 35638 | 15 | 2157753 | 2 | - |
| 2026-11-14T08:37:03Z | revision | agente:a0200fd81b95ad75a | 34 | 89433 | 1727110 | 23197 | 9 | 1839774 | 3 | - |

## Autorrevision

- code-quality-reviewer — e1d15c5
- code-quality-reviewer — 19f74bb
- code-quality-reviewer — 18370f1

## Rondas extra

- Ronda 3: Carlos, con la consigna «responder la opcion recomendada»; la correccion de la ronda 1 introdujo un IMPORTANTE (IMP-4, una celda de la tabla de capacidades) que se corrige siempre y es barato, asi que se verifica en una ronda mas en vez de cerrar con el abierto.

## Verificacion ligera

- 2026-11-14T08:37:06Z: ronda 3; commit revisado 15920490783032be788e78412e2adf495fc805ca; commit verificado 65d91ca1a4ffe8a3f05a4772f63ac76f86796132; ficheros: docs/adr/F1-8-seguridad-modelo-de-amenazas.md; corregidos: MEN-7; pendientes: ninguno

## Resultado

- **Implementado**: ADR [F1-8](../../../docs/adr/F1-8-seguridad-modelo-de-amenazas.md) (aceptada) y su fila en `docs/adr/README.md`. Modelo de amenazas por fronteras de confianza (Z1 maquina de la persona, Z2 contenido del repo como dato no confiable, Z3 runners de GitHub, Z4 runner propio, Z5 plataforma) con matriz de 16 activos → mitigacion → tarea duena (o riesgo aceptado nombrado). Dos clases de jobs: los que ejecutan codigo del PR (`pull_request`, solo lectura, sin secretos) y los privilegiados (publican con checkout solo de la base y leen artefactos como datos). Defensa contra prompt injection por capacidades. Revisor IA partido en dos jobs con la clave de la App y la del modelo solo en el publicador. Runner propio aislado (usuario de servicio, directorio limpio, `main` separado). Un solo modelo de confianza local por hash de la orden y de los scripts que ejecuta. Ningun workflow escucha `repository_dispatch` ni interpola datos del evento en `run:` (lint heuristico; lista blanca en F5-1). `buscar` sin acierto si el PR toca `.github/workflows/**` o `.taskcode/**`. Procedimiento de rotacion de secretos. Tabla de 13 preguntas heredadas de F1-5/F1-6/F1-7.
- **Decisiones de la persona** (consigna: opcion recomendada): D1b `carril` con `pull_request_target` como mitigacion parcial; D2b clave del modelo en el job publicador; D3b runner aislado; D4b hash de la orden y lo que ejecuta; D5a `arbol=` solo vale si lo publico el job publicador; ronda extra 3 (ver `## Rondas extra`).
- **Desacuerdos entre roles** (resueltos en el plan a favor de riesgos): `carril` con `pull_request_target` no cubre `pull_request_review`; el runner propio necesita aislamiento; el hash cubre lo que se ejecuta; las clases de workflow no son mitigacion cerrada sin el E2E de F2-6.
- **Divergencias** (en el ADR): con F1-7 (`arbol=` solo del publicador y sin acierto si el PR toca workflows o `.taskcode/`), F1-5 (`carril` parcial), F1-6 (claves del revisor y del modelo en el publicador; activo A5: el token de la persona esta en el llavero con scope `workflow` y al alcance de la sesion del agente, asi que la decision 4 queda condicionada a resolver A5); «lo que ejecuta» se limita a scripts versionados de forma transitiva; el hash consume casi todo el presupuesto de 1 s del hook de F5-6.
- **No verificado** (a F2-6 salvo indicacion): `GITHUB_TOKEN` creando check-runs con cualquier nombre, `pull_request_review` con el workflow del PR, `workflow_run` con artefactos, identificar el job desde el check-run, entornos que niegan secretos a `pull_request`, «Verified» tras retirar una clave, llave FIDO, si hace falta clave del modelo (F6-3), que el agente con el token de la persona no empuje workflows.
- **Revision**: ronda 1 cambios (IMP-1 A7 dado por cerrado con el publicador en `pull_request`, IMP-2 lint con huecos M1–M5, IMP-3 decision 4 contradecia A4/A5; MENOR-1..3), todo corregido; ronda 2 cambios (IMP-4 introducido por la correccion en la celda de capacidades); ronda extra 3 aprobada con correcciones: MEN-7 (nota del revisor sobre A13/A8, registrada como MENOR) corregido.
- **Incidencia**: el primer `taskctl start` fallo con EPERM porque el directorio de trabajo de la sesion estaba dentro de la carpeta de la tarea; la rama quedo creada y la tarea en diseno; al volver a la raiz, `start` funciono.
- Plan con dos roles (arquitectura y riesgos) y unificador; implementacion por un agente implementador; revision por tres agentes independientes. Temporales borrados.

## Duracion

- Estimada: 60 min
- Real (de la rama abierta al cierre con merge, sin pausas): 43 min
- Cierre (finish): 7 s, de 2026-11-14T08:37:25Z a 2026-11-14T08:37:32Z
