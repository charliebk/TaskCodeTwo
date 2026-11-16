---
id: TASK-076
titulo: "F2-4b Nombres unicos de tarea y rama"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-diseno
plan_aprobado: false
rama: feature/task-076-f2-4b-nombres-unicos-de-tarea-y-rama
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: null
tokens_revision: null
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
