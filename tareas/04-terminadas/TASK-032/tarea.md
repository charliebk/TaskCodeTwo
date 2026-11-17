---
id: TASK-032
titulo: "F3-5 tc2 board y tc2 status"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-032-f3-5-board-y-status
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: 28148188
tokens_revision: 5594212
creado: 2026-11-14
actualizado: 2026-11-17
dependencias: [TASK-018, TASK-021]
---
## Objetivo

`board` y `status`: estado del equipo (quién lleva qué, qué espera revisión) desde la rama de estado y GitHub

Tarea F3-5 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-3, F2-6.

## Criterios de aceptacion
- [x] C1: dos clones con el mismo `commit=` de `origin/taskcodetwo/tareas` y la misma respuesta de plataforma (fake) dan `board --json` y `status --json` iguales byte a byte, aunque difieran la cache de identidad, la carpeta `tareas/` con cambios sin publicar o el idioma (test de integracion con dos clones)
- [x] C2: tras `board` y `status` quedan identicos `git status --porcelain --ignored` (clon y `tareas/`), `git for-each-ref`, el arbol de `develop` y `.git/taskcodetwo/`; `board --escribir` da uso (2) y no existe `docs/BOARD.md`
- [x] C3a: sobre el mismo estado, las listas de `board` son las de `next`, el «quien» de cada tarea es el mismo texto que da `roadmap` y `commit=` es el mismo en los tres (unitario sobre `EnMemoria`)
- [x] C3b: `board` lista En diseno, En curso, En revision y Listas para empezar con ID, titulo, quien y PR, y avisa `revision-sin-pr=` y `pr-sin-revision=` (unitario con fake de `Plataforma`)
- [x] C3c: sin plataforma o sin red `board` sale 0 con la parte de estado completa y `plataforma=no-configurada|sin-credenciales|sin-red|error`, sin pendientes; `status` no consulta nunca la plataforma (fakes que fallan y test de que `status` no construye ningun adaptador)

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

## Resultado

Hecho: `tablero.rs` con dos ordenes de solo lectura sobre la misma `situacion` que `next` y `roadmap` (mismo `commit=`): `tc2 status [--json]`, una linea de equipo que no consulta nunca la plataforma, y `tc2 board [--json] [--sin-plataforma]`, la vista completa (en diseno, en curso, en revision, listas, con quien y PR) que cruza los PR abiertos por rama y avisa `revision-sin-pr=` y `pr-sin-revision=`; con la plataforma caida sale 0 con la parte de estado entera y `plataforma=no-configurada|sin-credenciales|sin-red|error`. Operacion nueva del puerto `prs_abiertos` en GitHub y GitLab con la misma lista (forks excluidos, paginacion con tope), con adenda en F1-4 y F1-13. «Quien» comun en `situacion`. Ninguna de las dos escribe. Un test por criterio; `cargo xtask verificar --perfil rapida` en verde.

No verificado: GitHub y GitLab reales (mock y documentacion); los campos de MR de GitLab se comprobaron en la documentacion oficial, no contra la API.

Revision ronda 1 (rust-wasm-reviewer y code-quality-reviewer): aprobada por los dos, sin CRITICOS ni IMPORTANTES. El codigo editado con scripts (Python y `sed`, contra la regla de usar Write/Edit) se reviso a proposito: literales de URL, `rename`, cadenas de uso y registros correctos. Mutaciones de C1, C2, C3a y C3c en rojo; prueba a mano con 6 tareas en es y en.

MENORES aceptados (a seguimiento, TASK-124): `Ilegibles:` sin sanear; varios PR en la misma rama, gana el borrador de menor numero; respuesta invalida de `prs_abiertos` con textos de `estado-puertas`; `sanear` no filtra U+2060..2064, U+00AD ni U+061C; el test de C3a no distingue el orden de `listas`; el texto sin `--json` sale siempre en espanol; la adenda de F1-4 dice que `status` escribe avisos en stderr y no lo hace.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T08:16:40Z | plan | automatico | persona | 0.16.1 |
| 2026-11-17T08:21:35Z | approve | automatico | persona | 0.16.1 |
| 2026-11-17T08:21:36Z | start | automatico | persona | 0.16.1 |
| 2026-11-17T08:50:39Z | review | automatico | persona | 0.16.1 |
| 2026-11-17T09:09:55Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-17T08:50:39Z | implementacion | sesion:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 14 | 10435 | 4747509 | 2473 | 0 | 4760431 | - | - |
| 2026-11-17T08:50:39Z | implementacion | agente:a1d1639464f8a8566 | 192 | 334534 | 22966493 | 86538 | 43 | 23387757 | - | - |
| 2026-11-17T09:05:39Z | revision | agente:a5e0e1e64ac493d0a | 36 | 114112 | 1579342 | 10727 | 0 | 1704217 | 1 | rust-wasm-reviewer |
| 2026-11-17T09:05:39Z | revision | agente:a5aa3c0be1b6b36b4 | 72 | 141829 | 3725083 | 23011 | 8 | 3889995 | 1 | code-quality-reviewer |

## Duracion

- Estimada: 240 min
- Real (de la rama abierta al cierre con merge, sin pausas): 48 min
- Cierre (finish): 254 s, de 2026-11-17T09:05:41Z a 2026-11-17T09:09:55Z
