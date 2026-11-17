---
id: TASK-032
titulo: "F3-5 tc2 board y tc2 status"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-curso
plan_aprobado: true
rama: feature/task-032-f3-5-board-y-status
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
actualizado: 2026-11-17
dependencias: [TASK-018, TASK-021]
---
## Objetivo

`board` y `status`: estado del equipo (quién lleva qué, qué espera revisión) desde la rama de estado y GitHub

Tarea F3-5 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-3, F2-6.

## Criterios de aceptacion
- [ ] C1: dos clones con el mismo `commit=` de `origin/taskcodetwo/tareas` y la misma respuesta de plataforma (fake) dan `board --json` y `status --json` iguales byte a byte, aunque difieran la cache de identidad, la carpeta `tareas/` con cambios sin publicar o el idioma (test de integracion con dos clones)
- [ ] C2: tras `board` y `status` quedan identicos `git status --porcelain --ignored` (clon y `tareas/`), `git for-each-ref`, el arbol de `develop` y `.git/taskcodetwo/`; `board --escribir` da uso (2) y no existe `docs/BOARD.md`
- [ ] C3a: sobre el mismo estado, las listas de `board` son las de `next`, el «quien» de cada tarea es el mismo texto que da `roadmap` y `commit=` es el mismo en los tres (unitario sobre `EnMemoria`)
- [ ] C3b: `board` lista En diseno, En curso, En revision y Listas para empezar con ID, titulo, quien y PR, y avisa `revision-sin-pr=` y `pr-sin-revision=` (unitario con fake de `Plataforma`)
- [ ] C3c: sin plataforma o sin red `board` sale 0 con la parte de estado completa y `plataforma=no-configurada|sin-credenciales|sin-red|error`, sin pendientes; `status` no consulta nunca la plataforma (fakes que fallan y test de que `status` no construye ningun adaptador)

## Autorrevision

- rust-wasm-reviewer — 8a3a2c5
- code-quality-reviewer — 8a3a2c5

Hallazgos propios (pase de «Que se revisa» de cada revisora; ninguno
CRITICO ni IMPORTANTE):

- MENOR: `conectar_produccion` reduce a `no-configurada` cualquier fallo al
  leer `origin` o la configuracion de la base (incluida una configuracion
  invalida); el detalle lo da `tc2 identidad` o `doctor`. Se deja asi:
  `board` es una vista y no debe fallar por la plataforma (C3c).
- MENOR: con terminal y `TC2_TOKEN_PERSONA`, `board` consulta con el rol
  `persona` sin comprobar que este en la lista de personas (es una lectura;
  `estado-puertas` si lo comprueba porque decide).
- MENOR: `texto_de_avisos` de `tablero` repite la forma de leer una lista
  del informe que tiene `roadmap`; no hay decision duplicada.
- Mutacion comprobada: `min_by_key` → `max_by_key` en `cruzar` hace fallar
  `board_cruza_los_pr_y_avisa`.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T08:16:40Z | plan | automatico | persona | 0.16.1 |
| 2026-11-17T08:21:35Z | approve | automatico | persona | 0.16.1 |
| 2026-11-17T08:21:36Z | start | automatico | persona | 0.16.1 |
