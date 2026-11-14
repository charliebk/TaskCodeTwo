---
id: TASK-006
titulo: "F1-5 ADR puertas en GitHub"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-006-f1-5-adr-puertas-en-github
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 416216
tokens_implementacion: 5655771
tokens_revision: 8267586
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-004]
---
## Objetivo

ADR: puertas en GitHub (rulesets, checks obligatorios, CODEOWNERS) por carril (P3, P7)

Tarea F1-5 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-3.

## Criterios de aceptacion
- [x] Lista exacta de checks y quién aprueba cada carril
- [x] Cumple lo que pide la tarea F1-5 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T05:45:04Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T05:48:33Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T05:48:34Z | start | automatico | persona | 0.16.1 |
| 2026-11-14T05:59:01Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T06:11:38Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T06:20:42Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T06:26:12Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T05:48:32Z | diseno | agente:acdbecb96c791d418 | 20 | 65550 | 337543 | 13103 | 17 | 416216 | - | - |
| 2026-11-14T05:59:01Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 8 | 9390 | 988704 | 2229 | 0 | 1000331 | - | - |
| 2026-11-14T05:59:01Z | implementacion | agente:a47add20da8cde43c | 68 | 179766 | 4428518 | 47088 | 15 | 4655440 | - | - |
| 2026-11-14T06:04:46Z | revision | agente:a8a9fe6f70978482f | 30 | 147481 | 1601377 | 22200 | 10 | 1771088 | 1 | - |
| 2026-11-14T06:17:16Z | revision | agente:acab03903595e6d25 | 54 | 170742 | 3237833 | 40821 | 11 | 3449450 | 2 | - |
| 2026-11-14T06:25:45Z | revision | agente:a14332b844588cfa1 | 46 | 128451 | 2876521 | 42030 | 6 | 3047048 | 3 | - |

## Autorrevision

- code-quality-reviewer — 5ee0c03
- code-quality-reviewer — cbe4d2d
- code-quality-reviewer — 2cd1f20

## Rondas extra

- Ronda 3: Carlos, con la consigna «responder la opcion recomendada»; la correccion de la ronda 1 introdujo tres IMPORTANTE (IMP-3, IMP-4, IMP-5) que se corrigen siempre y salen baratos, asi que se corrigen en una ronda mas en vez de cerrar con ellos abiertos o volver a diseno.

## Verificacion ligera

- 2026-11-14T06:25:48Z: ronda 3; commit revisado 38166f29fef407ae4ef1c93b19b8d98bea0499a4; commit verificado a7bafb0dc379338b41116214f1e4e6465df2fba1; ficheros: docs/adr/F1-5-puertas-en-github.md; corregidos: MEN-5, MEN-6; pendientes: ninguno

## Resultado

- **Implementado**: ADR [F1-5](../../../docs/adr/F1-5-puertas-en-github.md) (aceptada) y su fila en `docs/adr/README.md`. Modelo de puertas neutral traducido a rulesets (GitLab en F1-13); `aplicar-puertas` y `leer-puertas` en el puerto `Plataforma`. Reglas por rama: `develop` (PR, 1 aprobacion, ultimo push aprobado, solo merge commit, `strict`, checks `verificacion`, `revision-ia`, `carril`), `main` (igual con perfil completo y `strict=false`, porque con releases por merge commit `main` deja de ser ancestro de `develop`), ramas de tarea y `taskcode/tareas` sin borrado ni force-push, tags `v*` inmutables. Checks atados a su origen con `integration_id`. Check `carril`: aprobacion del plan valida solo con commit de `approve` firmado y verificado de la persona y atado al blob de `plan-final.md`; carril efectivo = mayor entre el declarado y el minimo por ficheros (errata solo si todo esta bajo `docs/`); la errata tambien tiene tarea (`fix/task-NNN-<slug>`). Tabla exacta carril → checks → quien aprueba. CODEOWNERS sobre `.github/**`, `CODEOWNERS` y `.taskcode/` (mitigacion parcial, F1-8). JSON de rulesets validados contra el esquema OpenAPI en `poc/rulesets/`.
- **Decisiones de la persona** (consigna: opcion recomendada): D1a solo lectura en GitHub, D2a Pro/Team/publico para la garantia, D3a `taskctl merge` y cola solo en organizacion, D4a, D5a, D6a; ronda extra 3 (ver `## Rondas extra`).
- **Evidencia**: GitHub solo con GET. El repo privado temporal da 403 «Upgrade to GitHub Pro» en rulesets y en la proteccion clasica; `charliebk/TaskCodeTwo` (publico) da `[]`.
- **Divergencias** (en el ADR): claves del esquema no decididas por el plan; `integration_id` de relleno para `revision-ia` hasta F1-6; validacion estricta del guion; errata con tarea, que lee «PR directo» de P7 como «sin plan aprobado»; numero de aprobaciones; IMP-3/IMP-4.
- **No verificado**: comportamiento real de las puertas aplicadas (F2-6, F9-4), cola solo en organizaciones, Pro/Team en privados, que la API acepte los JSON, la interfaz, la firma del approve (F4-2/F6).
- **Revision**: ronda 1 cambios (IMP-1 carril falsificable desde `taskcode/tareas`, IMP-2 `strict` en `main`, MEN-1..4); ronda 2 cambios (IMP-1/IMP-2 corregidos, la correccion introdujo IMP-3 plan no atado al contenido, IMP-4 errata con cualquier `.md`, IMP-5 errata sin ID); ronda extra 3 aprobada con correcciones: MEN-5 (blob frente a historia) y MEN-6 (RR-12) corregidos. Notas no bloqueantes: renombrados en el carril minimo (anadido a preguntas abiertas) y que el carril minimo nunca sube a funcionalidad (F1-10).
- Implementacion por un agente implementador; revision por tres agentes independientes. Temporales borrados.

## Duracion

- Estimada: 60 min
- Real (de la rama abierta al cierre con merge, sin pausas): 38 min
- Cierre (finish): 6 s, de 2026-11-14T06:26:06Z a 2026-11-14T06:26:12Z
