---
id: TASK-015
titulo: "F1-13 ADR adaptador de plataforma GitHub y GitLab"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-015-f1-13-adr-adaptador-de-plataforma-github
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 473765
tokens_implementacion: 29518586
tokens_revision: 8098265
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-005]
---
## Objetivo

ADR: adaptador de plataforma GitHub y GitLab (RR-15): equivalencias (rulesets y ramas protegidas, checks y pipelines, reviews y approvals, cola y tren de merge, Apps y tokens de proyecto) y qué depende del plan de pago

Tarea F1-13 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-4.

## Criterios de aceptacion
- [ ] Tabla de equivalencias verificada en las dos
- [ ] cómo se prueba cada adaptador
- [ ] Cumple lo que pide la tarea F1-13 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T19:36:45Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T19:42:06Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T19:42:08Z | start | automatico | persona | 0.16.1 |
| 2026-11-14T20:12:05Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T20:26:08Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T19:42:06Z | diseno | agente:a55220d0c7c37c631 | 22 | 58503 | 397181 | 18059 | 25 | 473765 | - | - |
| 2026-11-14T20:12:05Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 8 | 13611 | 996156 | 3319 | 0 | 1013094 | - | - |
| 2026-11-14T20:12:05Z | implementacion | agente:a4b9d65020a563362 | 172 | 332079 | 16508906 | 140601 | 19 | 16981758 | - | - |
| 2026-11-14T20:37:54Z | revision | agente:a6166227752a84c52 | 60 | 134799 | 3831373 | 27530 | 5 | 3993762 | 1 | - |
| 2026-11-14T20:38:02Z | revision | agente:a041312848d40d172 | 64 | 163645 | 3916490 | 24304 | 10 | 4104503 | 2 | - |
| 2026-11-14T20:38:07Z | implementacion | agente:dif:a4b9d65020a563362 | 66 | 778841 | 10705062 | 39765 | 10 | 11523734 | - | - |

## Autorrevision

- code-quality-reviewer — b94e6fe
- code-quality-reviewer — ca2d75f

## Verificacion ligera

- 2026-11-14T20:37:49Z: ronda 2; commit revisado dd39c4108a460d33676e2d283ea2012b370bd381; commit verificado b6e01816f2517ca8520394f404cdc1b8056da087; ficheros: docs/adr/F1-13-adaptador-de-plataforma.md, docs/adr/README.md; corregidos: MEN-7, MEN-8; pendientes: MEN-6

## Resultado

- **Implementado**: ADR [F1-13](../../../docs/adr/F1-13-adaptador-de-plataforma.md) (aceptada) y su fila en `docs/adr/README.md`. Puerto `Plataforma` de F1-4 con dos implementaciones (GitHub y GitLab por HTTP, sin `gh` ni `glab`), mismo contrato de salida y codigos 0-7, `rechazada motivo=plataforma|plan` cuando no hay equivalente; modelo neutral de puertas de F1-5 traducido a GitLab; tabla de 20 garantias como dato (regla neutral, API y tier en cada plataforma, sustituto, evidencia con URL y fecha); contrato de `capacidades` que lee `doctor` observando el plan real (403/404); identidad del agente en GitLab como token de proyecto (Premium) o cuenta de servicio en Free, nunca token personal.
- **Medido**: 44 fuentes oficiales con fecha (2026-11-14); 38 GET reales de solo lectura (15 GitHub con `gh api --method GET` sobre repos de charliebk, 23 anonimos a GitLab.com) y 16 escrituras o destinos no permitidos rechazados sin salir a la red; 13 filas con evidencia empirica que prueba algo (tabla `PRUEBA`), el resto documental. GitHub: 10 garantias dependen del plan de los rulesets (privado gratuito: 403 de plan observado), 9 no dependen y la cola exige organizacion (privado: Enterprise Cloud). GitLab: 11 en Free, 6 con Premium, ninguna mas con Ultimate y 3 en ningun tier (G01 un Maintainer borra una rama protegida, G08 rama al dia solo en `develop`, G09 origen de un check: un Developer que pueda aprobar responde el check externo); capacidades Free 11/9, Premium y Ultimate 17/3. GitLab cuenta la aprobacion de un bot sin marcarla (hay que contar contra la lista de personas). `estado-puertas`: 4 llamadas en GitHub y 5 en GitLab. Misma bateria contra un mock local para las dos plataformas.
- **Decisiones de la persona**: D1a..D8a (consigna: opcion recomendada); sin troceo; GitHub y GitLab solo lectura.
- **Divergencias** (en el ADR): nombres `taskcodetwo`; F1-4 (`leer-permisos-actions` → `leer-permisos-ci`, coste de `estado-puertas`); F1-5 (`main` sin `strict` no expresable en GitLab, cola en privado exige Enterprise Cloud, sin «sin bypass» en GitLab); F1-6 decision 6 (token de proyecto Premium, firmante por id de clave); F1-7 (la linea de cache ocupa 274 caracteres y el `description` de GitLab admite 255: lo decide F5-1); F1-8 (sin `pull_request_target` en GitLab); F1-10 (limite de ficheros del diff de GitLab sin verificar); P11 (ni cola ni tren en el plan gratuito de una cuenta personal).
- **No verificado**: el comportamiento de las puertas sin escribir (bloquear el merge, descartar aprobaciones, tren, force-push: pasa a F2-6/F2-9); ningun tier de pago probado; el 403 de plan de GitLab Free (sin sesion da 401); GitLab self-managed; un estado externo bloqueando en Free; que el agente Developer no pueda borrar ramas protegidas ni mergear, ni lea `protected_branches`/`approval_rules`; la configuracion de CI en otro proyecto; las escrituras de cada operacion.
- **Revision**: ronda 1 cambios (IMP-1 G09 vendido como garantia en Ultimate; IMP-2 filas «empiricas» con solo un 401/403 anonimo; MEN-1..3), corregido en ca2d75f; ronda 2 aprobada con correcciones; MEN-6..8 (`..` en la lista blanca, GH-03 como prueba debil, fila del indice larga) corregidos en b6e0181 (`verificar-menores` no reconoce MEN-6 por vivir en `poc/`, ver `revision/menores-pendientes.md`).
- Implementacion por un agente implementador; revision por dos agentes independientes. `taskctl doctor` con 0 errores. Remotos solo leidos (incluido el repo temporal `charliebk/taskcodetwo-poc-f1-1`, que sigue pendiente de borrar por Carlos). Temporales borrados.

