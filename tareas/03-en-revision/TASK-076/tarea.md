---
id: TASK-076
titulo: "F2-4b Nombres unicos de tarea y rama"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-076-f2-4b-nombres-unicos-de-tarea-y-rama
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: 23113641
tokens_revision: 6755469
creado: 2026-11-14
actualizado: 2026-11-16
dependencias: [TASK-019]
---
## Objetivo

Nombres únicos: rama derivada del ID comprobada también contra el remoto; título duplicado o casi igual a otro abierto → aviso con la tarea parecida (RR-12)

Tarea F2-4b del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-4.

Contexto (Carlos, 2026-11-14, RR-12 reforzado): el trabajo con varias personas debe estar garantizado: todos saben qué tarea tiene cada uno y no se pueden pisar ni los nombres ni las tareas. Además de lo ya previsto (IDs por push atómico, toma atómica en start, roadmap con quién lleva cada tarea, status), se cierran los huecos: nombres de rama y títulos únicos, identidad real de la plataforma (no texto libre), liberar y reasignar tareas abandonadas, aviso de solape de ficheros entre tareas en curso y estado siempre fresco con hora de sincronización. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 ter.

## Criterios de aceptacion
- [ ] C1: dos `tc2 new "X"` a la vez desde dos clones dan IDs distintos, `rama` distintas en los `tarea.md` publicados y la segunda en publicar sale con `parecidas=<ID de la otra>` (E2E en `crates/tc2/tests/alta.rs`)
- [ ] C2: con `feature/task-005-x` solo en el remoto (sin carpeta), el siguiente `tc2 new` da un ID mayor que 5 (test)
- [ ] C3: lo mismo con una rama `*/task-NNN-*` que solo existe en local (test)
- [ ] C4: un titulo que plegado es igual al de una tarea abierta da `parecidas=TASK-x (duplicado)` con codigo 0; contra una terminada no avisa (test)
- [ ] C5: «casi igual» es Dice sobre bigramas del titulo plegado >= 0,80, fijado con una tabla unitaria («Exportar informes»/«Exprotar informes» avisa; «Login con Google»/«Login con GitHub» no)
- [ ] C6: sin red, el `ls-remote` previo sale `sinred` (4) y no deja refs nuevas (test)
- [ ] C7: `tarea.md` lleva `rama: feature/task-nnn-<slug>` que pasa `rama_valida`; un titulo sin letras latinas da el slug `tarea` (unitarios)

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-16T19:49:05Z | plan | automatico | persona | 0.16.1 |
| 2026-11-16T19:54:15Z | approve | automatico | persona | 0.16.1 |
| 2026-11-16T19:54:16Z | start | automatico | persona | 0.16.1 |
| 2026-11-16T20:26:32Z | review | automatico | persona | 0.16.1 |

## Autorrevision

- rust-wasm-reviewer — 0f11a56
- code-quality-reviewer — 0f11a56

Corregido en la autorrevision: el tipo `feature` de la rama estaba decidido
en dos sitios (`Tarea::nueva` y `alta::TIPO`); el alta lo fija ahora desde
su constante (0f11a56). Sin `unwrap`, `expect` ni `panic` nuevos en
produccion; el unico error tragado (`let _ = write_all` hacia `cat-file`)
lo delata el parseo de la salida, que exige un objeto por ruta.

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-16T20:26:32Z | implementacion | sesion:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 32 | 14671 | 5145151 | 3552 | 1 | 5163406 | - | - |
| 2026-11-16T20:26:32Z | implementacion | agente:a95f8bf4e6c3190ce | 174 | 276857 | 17597027 | 76177 | 42 | 17950235 | - | - |
| 2026-11-16T20:43:11Z | revision | agente:a3e8d6cf98a7bbb7a | 68 | 143802 | 3641661 | 27040 | 4 | 3812571 | 1 | rust-wasm-reviewer |
| 2026-11-16T20:43:11Z | revision | agente:a74cbe02b54441e1c | 58 | 131670 | 2788046 | 23124 | 4 | 2942898 | 1 | code-quality-reviewer |
