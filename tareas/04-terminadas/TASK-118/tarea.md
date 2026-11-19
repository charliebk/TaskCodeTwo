---
id: TASK-118
titulo: "Aplicar las puertas (rulesets) desde tc2 init"
tipo: feature
sprint: 0
etiquetas: [plataforma, puertas]
complejidad: media
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-118-aplicar-las-puertas-rulesets-desde-tc2-i
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1769062
tokens_implementacion: 34895921
tokens_revision: 11460158
creado: 2026-11-17
actualizado: 2026-11-19
dependencias: []
---
## Objetivo

TASK-028 (F3-1) solo propone los rulesets taskcodetwo-*. Aplicarlos exige los checks verificacion, revision-ia y carril (F5-1, F6) y el integration_id real (F6-1). Esta tarea anade aplicar_puertas al trait Plataforma (GitHub y GitLab) y la opcion de init que las aplica con permiso de la persona.

## Criterios de aceptacion
- [x] tc2 init --aplicar-puertas crea los rulesets taskcodetwo-* en GitHub y GitLab (test con doble HTTP)
- [x] Sin los checks ni el integration_id real, la opcion se rechaza con el motivo (test)

## Resultado

Hecho: `tc2 init --aplicar-puertas` aplica las puertas `taskcodetwo-*` con permiso de la persona. Sin la opcion, `init` sigue igual (las lineas `ruleset` salen `persona`).
- Puerto `Plataforma`: `preparar_puertas` (solo lecturas) y `aplicar_puertas` (en lote), con defecto `rechazada motivo=no-soportada`. `puertas.rs`: `Origenes`, `origenes()`, `ruleset_github(p, Option<&Origenes>)` con `integration_id` y sin `_nota`. Clave nueva `plataforma.revisor_id` (origen de `revision-ia`; 15368, GitHub Actions, para `verificacion` y `carril` en GitHub).
- Precondicion antes de cualquier escritura (D1, D2, D5): la persona con `personas.yml`, cada check visto ya publicado por su origen en la cabeza de `develop`, y `develop`/`main` en el remoto sin el local por delante. Si no: `init rechazada motivo=puertas-sin-origen faltan=...` (o `sin-personas`, `sin-persona`, `ramas-sin-remoto`, `ramas-por-delante`, `plan`), sin escribir nada.
- GitHub: `POST /rulesets` de los que faltan; los existentes no se tocan (por nombre); el 422 cuenta como existente solo si la relectura ya lo trae. GitLab: ramas y tags protegidos con niveles explicitos (push=Developer y sin force-push en ramas de tarea y estado; `develop`/`main` push=0 y merge=Maintainer, por G01 de F1-13), ajustes de merge del proyecto, regla de aprobacion solo en Premium (`sin-aprobaciones=si` en Free); lo existente distinto sale `difiere` y no se toca; nunca `DELETE`.
- Corte a mitad: la linea lleva `aplicadas=`, `a-medias=` y `faltan=`, y repetir converge.
- Adendas en F1-5 (operacion, `revisor_id`, precondicion, divergencia: lo existente y distinto no se actualiza) y F1-13 (fila de `aplicar-puertas`).
- En la practica la opcion se rechaza hasta que existan los checks de F5-1 y el revisor de F6-1: es el comportamiento del criterio 2.
- `cargo xtask verificar --perfil rapida` en verde.

Implementado por un subagente con el plan aprobado; revisado por dos agentes independientes.

No verificado contra plataformas reales (solo doble HTTP): que GitHub acepte el `POST` con `integration_id` y de 422 por nombre repetido; en GitLab, el 409, que `feature/*` abarque nombres con `/` y el cuerpo de la regla de aprobacion.

Revision ronda 1: rust-wasm-reviewer aprobada con correcciones (MEN-1) y code-quality-reviewer cambios solicitados (IMP-1, MEN-1, MEN-2). IMP-1 = MEN-1 de rust: en GitLab, repetir tras un corte entre proteger `develop`/`main` y el `PUT` de ajustes no convergia; corregido (b9f6621, ad859f7): si la proteccion ya es la del modelo y solo faltan ajustes o regla, se completan; test `aplicar_puertas_gitlab_corte_tras_proteger_converge`. MEN-1 y MEN-2 (precision de las adendas de F1-5 y F1-13) corregidos.
Revision ronda 2 (acotada): aprobada por los dos; MEN-3 (la adenda de F1-5 no contaba el `PUT` al completar una puerta a medias) corregido en 23e2133. El comentario «Nunca reescribe ni borra» de `plataforma/mod.rs` habla de protecciones y rulesets; no se cambio.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-18T23:15:26Z | plan | automatico | persona | 0.16.1 |
| 2026-11-18T23:23:36Z | approve | automatico | persona | 0.16.1 |
| 2026-11-18T23:23:38Z | start | automatico | persona | 0.16.1 |
| 2026-11-19T00:54:24Z | review | automatico | persona | 0.16.1 |
| 2026-11-19T01:59:45Z | review | automatico | persona | 0.16.1 |
| 2026-11-19T02:38:47Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-18T23:21:41Z | diseno | agente:a796a41beee33a64f,aadccfc80a35137de | 64 | 143574 | 1522090 | 28166 | 54 | 1693894 | - | - |
| 2026-11-18T23:23:25Z | diseno | agente:a5e8dec083c520ba2 | 6 | 30359 | 36713 | 8090 | 4 | 75168 | - | - |
| 2026-11-19T00:54:24Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 48 | 48953 | 9566914 | 12725 | 4 | 9628640 | - | - |
| 2026-11-19T00:54:24Z | implementacion | agente:a61e78c9f4a1bc6b5 | 200 | 680697 | 24484996 | 101388 | 17 | 25267281 | - | - |
| 2026-11-19T01:10:19Z | revision | agente:a5991d3c8d5bc39b2 | 44 | 156428 | 2894895 | 18292 | 4 | 3069659 | 1 | rust-wasm-reviewer |
| 2026-11-19T01:10:19Z | revision | agente:acfb29742fe2d94f3 | 50 | 168278 | 3198890 | 24402 | 1 | 3391620 | 1 | code-quality-reviewer |
| 2026-11-19T02:09:18Z | revision | agente:a1b0bb07e5d9cc9a0 | 48 | 138083 | 2586534 | 27703 | 3 | 2752368 | 2 | code-quality-reviewer |
| 2026-11-19T02:09:18Z | revision | agente:adf5364785fb9f4a1 | 36 | 149044 | 2068012 | 29419 | 9 | 2246511 | 2 | rust-wasm-reviewer |

## Autorrevision

- rust-wasm-reviewer — a7f212e
- code-quality-reviewer — a7f212e
- rust-wasm-reviewer — ad859f7
- code-quality-reviewer — ad859f7

## Verificacion ligera

- 2026-11-19T02:09:24Z: ronda 2; commit revisado 29d24a7abacf092fb86a6d23d91ddc0b2cc75977; commit verificado 23e21334d093cdf8bf326a4b8ce659dd8ef75c2e; ficheros: docs/adr/F1-5-puertas-en-github.md; corregidos: MEN-3; pendientes: ninguno

## Duracion

- Estimada: 150 min
- Real (de la rama abierta al cierre con merge, sin pausas): 195 min
- Paso de la hora: 195 min reales frente a 150 estimados
- Cierre (finish): 1704 s, de 2026-11-19T02:10:23Z a 2026-11-19T02:38:47Z
