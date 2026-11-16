---
id: TASK-095
titulo: "F1-19 ADR memoria del proyecto: indice de contexto, etiquetas y busqueda"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-095-f1-19-adr-memoria-del-proyecto-indice-de
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1093445
tokens_implementacion: 4692745
tokens_revision: 8373351
creado: 2026-11-16
actualizado: 2026-11-16
dependencias: [TASK-002, TASK-010, TASK-012, TASK-014]
---
## Objetivo

ADR: memoria del proyecto (RR-20): git como verdad e índice SQLite reconstruible, ubicación del índice, regla de frescura por SHA, esquema, vocabulario y tres capas de etiquetado, conversor de engram (decisión A/B) y retirada de engram del manifiesto

Tarea F1-19 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-1, F1-9, F1-11, F1-14.

Contexto (Carlos, 2026-11-16, RR-20, hito H10): memoria del proyecto propia que sustituye a engram. Git es la fuente de verdad (tareas, etiquetas, ficheros tocados, memoria importada) y una base SQLite local es un índice reconstruible que nunca se versiona y nunca responde con datos viejos: comprueba en cada lectura hasta qué commit está indexado, porque el merge-pr lo hace la persona en la plataforma y ningún hook local se entera. Etiquetado en tres capas con vocabulario cerrado, `tc2 buscar` como única búsqueda del plugin, conversores de tareas y de engram, grafo y visor 3D opcional con dockview. Detalle completo: docs/plan/AMPLIACION-2026-11-16.md.

## Criterios de aceptacion
- [x] Índice de las tareas terminadas de TaskCodeTwo reconstruido dos veces con el mismo hash
- [x] Un merge hecho en la plataforma se refleja en la siguiente búsqueda sin hooks (con la politica TTL elegida por Carlos: en la primera busqueda tras caducar el TTL; antes, con la marca «sincronizado hace N s»)
- [x] Cumple lo que pide la tarea F1-19 del plan y los requisitos que cita

## Resultado

ADR [F1-19](../../../docs/adr/F1-19-memoria-del-proyecto.md) aceptado y fila en el indice de `docs/adr/README.md`. Sin codigo de la 2.0 en el repo (Fase 1); PoC minimas desechables (`poc-indice`, Rust con `rusqlite` bundled) en `%LOCALAPPDATA%\pruebas-claude	askcodetwo	ask-095\`.

Decisiones (Carlos, 2026-11-16): no trocear (140 min estimados); indice en `<git-common-dir>/taskcode/indice.db` leido siempre de objetos de git; huella SHA-256 de volcado canonico; frescura con una sola politica compartida con F3-5b: SHA local en cada lectura y `ls-remote` como mucho una vez por TTL de 60 s, con marca (se eligio tras medir `ls-remote` contra GitHub: 760 ms de mediana frente a los 200 ms de F10-6; la primera eleccion, `ls-remote` en cada lectura, quedo descartada por esa medicion); regla de atribucion commit → tarea por asunto y rango del merge; vocabulario en la rama de codigo; conversor de engram A con salvaguardas; retirada de engram condicionada a F10-13.

Evidencia: criterio 1, huella `9527d653…` dos veces sobre las 30 terminadas, arbol sucio con `autocrlf` no la cambia, un commit si; criterio 2, bare local como plataforma, clon con `core.hooksPath` vacio: el merge aparece sin `fetch` manual ni hooks en la primera busqueda tras el TTL, force-push lleva a reconstruccion completa, remoto caido responde con marca. Ambas PoC reproducidas por el revisor.

Revision: 3 rondas (la 3.ª extra, decidida por Carlos). Ronda 1: IMP-1 (atribucion solo por asunto pierde el trabajo real: 2 de 18 ADR), IMP-2 (los 6-7 ms no respaldan los 200 ms; proceso completo 177-289 ms) y MEN-1..MEN-5 (fila del README, textos que cambian, cifras, efectos en F1-4 y F3-5b, no verificado incompleto): todos corregidos. Ronda 2: IMP-3 (cifras de atribucion mal citadas, 467 en vez de 447/449): corregido y verificado en la ronda 3 (aprobada). Nada se dejo sin corregir.

Pendiente de Carlos (anotado en Consecuencias del ADR, no editado aqui): confirmar el cambio a F3-5b (sin `fetch` previo dentro del TTL) y ajustar la redaccion de RR-20, fila F1-19, fila F10-2 y este criterio 2. No verificado: 200 ms de F10-6 de punta a punta (depende de no lanzar `git` en el camino caliente), WAL y lectura con cerrojo ocupado, huella en otra maquina o sistema, `rusqlite` bundled en Linux y macOS.

Nota de proceso: en la ronda 1 el revisor rodeo en su clon temporal un hook que bloquea `git merge` usando `commit-tree` y `update-ref`; lo declaro en su informe.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-16T12:55:18Z | plan | automatico | persona | 0.16.1 |
| 2026-11-16T13:06:31Z | approve | automatico | persona | 0.16.1 |
| 2026-11-16T13:06:45Z | start | automatico | persona | 0.16.1 |
| 2026-11-16T13:30:43Z | review | automatico | persona | 0.16.1 |
| 2026-11-16T13:52:50Z | review | automatico | persona | 0.16.1 |
| 2026-11-16T14:49:35Z | review | automatico | persona | 0.16.1 |
| 2026-11-16T14:52:57Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-16T13:03:25Z | diseno | agente:ab69920f34bade972,ac5f927590e26cbe9,a4a936ae6ac2f3123 | 56 | 154965 | 906531 | 31893 | 42 | 1093445 | - | - |
| 2026-11-16T13:30:43Z | implementacion | sesion:a5dfc352-0a22-4b82-bbd5-fc62af318858 | 52 | 76920 | 4570682 | 45091 | 10 | 4692745 | - | - |
| 2026-11-16T13:45:20Z | revision | agente:aa43da2fc92710b52 | 62 | 153463 | 3823611 | 32512 | 10 | 4009648 | 1 | - |
| 2026-11-16T14:08:46Z | revision | agente:ad7f1fb1813d74cdb | 56 | 170686 | 3362321 | 33738 | 9 | 3566801 | 2 | - |
| 2026-11-16T14:52:05Z | revision | agente:a609d4649c88230bb | 16 | 109084 | 669077 | 18725 | 1 | 796902 | 3 | - |

## Autorrevision

- code-quality-reviewer — 72d9c0e
- code-quality-reviewer — 63e9c84
- code-quality-reviewer — 1e73ccf

## Rondas extra

- Ronda 3: Carlos (2026-11-16), para verificar solo la correccion de cifras de IMP-3; la regla y las conclusiones del ADR no cambian.

## Verificacion de documentacion

- 2026-11-16T14:52:48Z: la tarea solo cambia documentacion (22 fichero(s) dentro de rutas_documentacion de "develop"); no se ejecuta `cargo xtask verificar --perfil rapida`, sin comprobacion de documentacion declarada

## Duracion

- Estimada: 140 min
- Real (de la rama abierta al cierre con merge, sin pausas): 106 min
- Paso de la hora: 106 min reales frente a 140 estimados
- Cierre (finish): 9 s, de 2026-11-16T14:52:48Z a 2026-11-16T14:52:57Z
