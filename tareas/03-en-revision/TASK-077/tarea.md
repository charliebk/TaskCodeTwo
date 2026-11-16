---
id: TASK-077
titulo: "F2-13 Identidad de la persona desde la plataforma"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-077-f2-13-identidad-de-la-persona-desde-la-p
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: 19113461
tokens_revision: 21854471
creado: 2026-11-14
actualizado: 2026-11-16
dependencias: [TASK-007, TASK-021, TASK-022]
---
## Objetivo

Identidad de la persona: el «quién» de una tarea es el usuario de GitHub o GitLab autenticado, no un texto libre (RR-12)

Tarea F2-13 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-6, F2-6, F2-9.

Contexto (Carlos, 2026-11-14, RR-12 reforzado): el trabajo con varias personas debe estar garantizado: todos saben qué tarea tiene cada uno y no se pueden pisar ni los nombres ni las tareas. Además de lo ya previsto (IDs por push atómico, toma atómica en start, roadmap con quién lleva cada tarea, status), se cierran los huecos: nombres de rama y títulos únicos, identidad real de la plataforma (no texto libre), liberar y reasignar tareas abandonadas, aviso de solape de ficheros entre tareas en curso y estado siempre fresco con hora de sincronización. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 ter.

## Criterios de aceptacion
- [x] C1: `tc2 identidad` con `TC2_TOKEN_PERSONA` y TTY imprime `identidad ok rol=persona id=<N> login=<L>` (formato del contrato F1-4) y guarda la cache local (`tests/identidad.rs`, mock GitHub y GitLab en 127.0.0.1)
- [x] C2: cuenta del token fuera de `personas.yml`, `Bot` o `web-flow` → `rechazada motivo=identidad` y la cache no se toca (`tests/identidad.rs`)
- [x] C3: `personas.yml` con `id` repetido o `login` repetido sin distinguir mayusculas se rechaza nombrando las dos entradas (unitario en `plataforma/personas.rs`)
- [x] C4: en pantalla sale siempre el `login`; `nombre` es opcional y nunca lo sustituye; dos personas con el mismo nombre se distinguen por login e id (unitario)
- [x] C5: `asignado_a: {id: N, login: "L"}` sobrevive a la ida y vuelta del YAML; «ya la lleva» compara por `id` (mismo login con otro id = otra persona; login renombrado con el mismo id = la misma) (`tests/tarea.rs`)
- [x] C6: una cuenta `Bot` nunca acaba en `asignado_a` (error tipado); sin TTY `Quien` es `Agente` y la transicion lleva `decidido_por: agente` (unitario de `identidad.rs`)
- [x] C7: sin red `tc2 identidad` da `identidad sinred` (4, contrato F1-4) sin tocar la cache; con cache vigente `quien_ejecuta` no pide nada a la red (0 peticiones); sin cache o caducada y sin TTY → `rechazada motivo=identidad` pidiendo `tc2 identidad`
- [x] C8: sin plataforma reconocida `detectar` rechaza; no hay vuelta a texto libre (`tests/identidad.rs`)
- [x] C9: un `asignado_a` de texto de la 0.16 se lee como `Heredado` y la 2.0 nunca lo escribe; si coincide con un `correos:` de `personas.yml` se convierte (`tests/tarea.rs`)

## Autorrevision

- rust-wasm-reviewer — ce6cc61
- code-quality-reviewer — ce6cc61

## Rondas extra

- Ronda 3: Carlos (consigna del 2026-11-16, «aceptamos todas las respuestas de forma recomendada»); IMP-2 de la ronda 2 es un texto ya corregido en 66fed63 y hay que verificarlo.

## Resultado

Hecho: `.taskcode/personas.yml` con lector y validador (`plataforma/personas.rs`: `id`, `login`, `correos` sin repetir, sin distinguir mayusculas tambien fuera de ASCII), `asignado_a` como `Asignado::Persona{id, login}` o `Heredado` (solo construible por el lector; nota en ADR F1-9), `identidad.rs` (`quien_ejecuta`, cache `<git-common-dir>/tc2/identidad.yml` con TTL de 7 dias y tope de 64 KiB, cuentas `bot`/`no-persona`/`web-flow`/fuera de lista nunca asignables) y la orden `tc2 identidad [--json]`, que con terminal trae `origin develop` antes de leer la lista. Cada criterio C1..C9 con su test (`tests/identidad.rs`, `tests/tarea.rs`, unitarios); `cargo xtask verificar --perfil rapida` en verde. C1 y C7 se ajustaron al formato del contrato F1-4 (`identidad ok ...`, `identidad sinred`).

No verificado: terminal real con mintty/Git Bash (la deteccion usa `stdin().is_terminal()`; el revisor lo probo con `cmd`), API real de GitHub/GitLab (solo mock en 127.0.0.1) y clones con refspec estrecho. `start` aun no existe (F3-6): la pieza queda en biblioteca; `convertir_heredado` no tiene llamador de produccion hasta F3-6/F3-10.

Revision (rust-wasm-reviewer y code-quality-reviewer):
- Ronda 1: IMP-1 calidad (lista leida sin fetch) y MEN-1 calidad, MEN-1..3 rust corregidos en 8e7a307 y 2fae3e4.
- Ronda 2: IMP-2 calidad (pista inutil con terminal) y MEN-2 calidad / MEN-4 rust (sin rama base salia como sin red) corregidos en 66fed63.
- Ronda 3 (extra, por la consigna de Carlos): aprobada por los dos, sin hallazgos nuevos.

Riesgo aceptado (plan): el agente podria editar la cache local y hacerse pasar por otra persona de la lista; `asignado_a` no aprueba nada (la aprobacion es la firma de F1-6/F4-2).
Entorno: `plataforma::protecciones_y_bordes` fallo una vez y paso al repetir (cuelgue de 127.0.0.1 de esta maquina, en TASK-115).

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-16T20:50:31Z | plan | automatico | persona | 0.16.1 |
| 2026-11-16T20:55:31Z | approve | automatico | persona | 0.16.1 |
| 2026-11-16T20:55:32Z | start | automatico | persona | 0.16.1 |
| 2026-11-16T21:32:43Z | review | automatico | persona | 0.16.1 |
| 2026-11-16T21:59:48Z | review | automatico | persona | 0.16.1 |
| 2026-11-16T22:15:20Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-16T21:32:43Z | implementacion | sesion:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 18 | 13255 | 3342537 | 2983 | 1 | 3358793 | - | - |
| 2026-11-16T21:32:43Z | implementacion | agente:aa8229874a65055ff | 158 | 253090 | 15423275 | 78145 | 60 | 15754668 | - | - |
| 2026-11-16T21:50:44Z | revision | agente:a4df2532b4a058ae3 | 68 | 157049 | 4206458 | 24218 | 8 | 4387793 | 1 | rust-wasm-reviewer |
| 2026-11-16T21:50:45Z | revision | agente:a2b2ee854173661e9 | 94 | 150665 | 5419863 | 25774 | 5 | 5596396 | 1 | code-quality-reviewer |
| 2026-11-16T22:10:03Z | revision | agente:ab7c06ea4cbfc1d09 | 50 | 141768 | 2747604 | 20894 | 12 | 2910316 | 2 | rust-wasm-reviewer |
| 2026-11-16T22:10:03Z | revision | agente:a1b445b145a3ae10b | 58 | 141285 | 2905107 | 35203 | 4 | 3081653 | 2 | code-quality-reviewer |
| 2026-11-16T22:28:48Z | revision | agente:afbfbd15179697350 | 52 | 103649 | 2118951 | 18079 | 1 | 2240731 | 3 | code-quality-reviewer |
| 2026-11-16T22:28:48Z | revision | agente:a1440d1b8cc698f28 | 78 | 122701 | 3491367 | 23436 | 6 | 3637582 | 3 | rust-wasm-reviewer |
