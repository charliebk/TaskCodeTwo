---
id: TASK-049
titulo: "F6-3 Revisor en CI con las skills de dominio de la 0.16"
tipo: feature
sprint: 6
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-049-f6-3-revisor-en-ci-con-las-skills-de-dom
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 998648
tokens_implementacion: 45593290
tokens_revision: 6840056
creado: 2026-11-14
actualizado: 2026-11-20
dependencias: [TASK-044, TASK-048]
---
## Objetivo

Revisor en CI con las skills de dominio de la 0.16 (angular-vue, rust-wasm, C#, Java, genérica), enrutado por lo que toca el diff (RR-02)

Tarea F6-3 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F6-2, F5-4.

Contexto (Carlos, 2026-11-14, RR-17, imprescindible): para cada tarea y cada rol (diseño, implementación, revisión) hay que elegir las skills y los agentes adecuados y el modelo según complejidad y carril, solo entre lo que está instalado en la máquina y el proyecto, anotar la elección en la tarea y medir si acierta (hallazgos por revisora y coste por modelo). En la 0.16 existe con `catalogo-skills.yml`, `skills_recomendados` y `modelo_sugerido`, pero se usó en 21 / 0 / 2 tareas de ToolsaWeb / OpenDockTrading / TaskCode porque el catálogo apenas se configuró: hoy la elección la hace el agente sin criterio registrado. Nada debe decidirlo el modelo «a ojo»: reglas deterministas. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §1–4.

## Criterios de aceptacion
- [x] Un diff Rust + Vue lanza las dos de dominio
- [x] Cumple lo que pide la tarea F6-3 del plan y los requisitos que cita


## Resultado

Revisor IA en CI (F6-3). La seleccion es determinista: `crates/tc2/src/seleccion/` lee las tablas de F1-15 incrustadas del ADR y elige las revisoras, las reglas y el modelo a partir de las rutas del diff. La exponen `tc2 revisoras --rutas <f|-> [--nul] [--json]` y `tc2 review`, que la anota en `roles.revision`. Workflow reutilizable `.github/workflows/revision-ia.yml` con dos jobs:
- `preparar`, de clase ejecuta y sin secretos: base, plugin y `tc2` fijados por SHA; el head solo como dato; prompts con el `SKILL.md`, la seccion del informe F5-4 y el diff por revisora;
- `publicar`, en el entorno `revisor`: llama a la Messages API y publica el check `revision-ia` `neutral` con la App del revisor.

Lint del perfil `revision-ia`, guia en `docs/ci/revision-ia.md` y adendas en F1-15 y F1-8.

Evidencia (2026-11-20):
- C1: el unitario y la integracion de `tc2 revisoras` (Rust + Vue da `angular-vue-reviewer` y `rust-wasm-reviewer`, sin generica, reglas S08 y S12). Confirmado en el E2E real.
- C2, E2E real en `charliebk/taskcodetwo-e2e-github` (PR 14, Rust + Vue; App del revisor `tc2-revisor-charliebk`, id 5846307, y entorno `revisor` con sus tres secretos, creados por Carlos):
  - el check `revision-ia` lo publica la App 5846307 con `neutral`; el resumen lleva `revisoras=taskcode-plugin:angular-vue-reviewer,taskcode-plugin:rust-wasm-reviewer`, `modelo-api=claude-opus-5-5` y `reglas=M05,S08,S12` (run 44740402515);
  - los prompts citan `## Informe de verificacion (F5-4)` con el informe del run de verificacion (run 44733610887);
  - `preparar` solo saca la base, el plugin y TaskCodeTwo y compila `tc2`, sin ejecutar nada del PR;
  - con el entorno sin secretos utilizables, `publicar` termina en verde con el aviso y sin check (run 44733610887).
  - Las revisoras no llegaron a responder: la cuenta de la API de Carlos no tiene saldo (`error-http-400 invalid_request_error: Your credit balance is too low...`, que ahora sale en el check). El formato de la llamada se comprobo contra la API: el error es de saldo, no de la peticion.
  - Repo E2E limpio: PR cerrado, rama borrada y workflows retirados de `main` y `develop`. El entorno `revisor` y la App se dejan creados.
- Los tests de lo tocado estan en verde: `cargo test -p xtask` (29), `cargo test -p tc2 --test revisoras` y `cargo clippy -p xtask --all-targets -D warnings`. `finish` relanza la verificacion completa.

Divergencias halladas en el E2E (corregidas y documentadas):
- GitHub solo entrega los secretos del entorno al reutilizable si el llamador usa `secrets: inherit`. Sin `secrets:` llegan vacios, y tambien declarandolos en `on.workflow_call.secrets` (run 44735329636). Carlos decidio una excepcion acotada a R9 de F1-8: `inherit` solo en el llamador de `revision-ia.yml` de TaskCodeTwo fijado por SHA (perfil `llamador-revision` del lint). Esta en la adenda F6-3 de F1-8, que tambien dice que se heredan todos los secretos y que el lint es de tests.
- Si la verificacion del head sale de la cache por arbol (F5-2), su run no sube el informe y el check dice `informe=sin-informe`. Queda como limite en la guia, para una tarea de seguimiento.
- Mejora apuntada: la federacion de identidades de la API de Anthropic con GitHub Actions evitaria guardar la clave.

Revision por pares:
- Ronda 1: rust-wasm-reviewer aprobada (MEN-1, tope de stdin); code-quality-reviewer cambios-solicitados (IMP-1 rutas no ASCII, IMP-2 fetch sin credencial, MEN-3 pipefail, MEN-4 artefacto ausente, MEN-5 cerca del diff). Todo corregido en e399b23.
- Ronda 2, tras el E2E: rust-wasm-reviewer aprobada-con-correcciones (IMP-1 `expect` que rompia clippy, MEN-1 dos huecos del test); code-quality-reviewer aprobada (MEN-1 clave en el motivo de error, MEN-2 limites de la excepcion sin documentar, MEN-3 `sin-cuerpo` y recorte por bytes). Todo corregido en 9c14787.
- Ronda 3 (extra, acotada a 9c14787): rust-wasm-reviewer y code-quality-reviewer aprobada, sin hallazgos (clippy -D warnings limpio, 29 tests de xtask, mutaciones de los asserts nuevos en rojo, motivo de error probado con clave, sin error, no JSON y 250 tildes).

## Rondas extra

- Ronda 3: la decide quien orquesta, con la consigna de Carlos de seguir con la opcion recomendada; solo verifica las correcciones de la ronda 2 (9c14787: un IMPORTANTE de clippy y MENORES baratos), sin codigo nuevo.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-19T21:59:15Z | plan | automatico | persona | 0.16.1 |
| 2026-11-19T22:23:19Z | approve | automatico | persona | 0.16.1 |
| 2026-11-19T22:23:21Z | start | automatico | persona | 0.16.1 |
| 2026-11-20T00:26:36Z | review | automatico | persona | 0.16.1 |
| 2026-11-20T13:03:28Z | review | automatico | persona | 0.16.1 |
| 2026-11-20T13:13:22Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-19T22:23:18Z | diseno | agente:a7d15d7dc5efda9d6 | 40 | 71750 | 909469 | 17389 | 32 | 998648 | - | - |
| 2026-11-20T00:26:36Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 46 | 68052 | 10530571 | 10298 | 4 | 10608967 | - | - |
| 2026-11-20T00:26:36Z | implementacion | agente:a26939dbbe09eb062 | 248 | 422570 | 34426743 | 134762 | 68 | 34984323 | - | - |
| 2026-11-20T13:11:42Z | revision | agente:a36f7278c0863ff4c | 36 | 117665 | 1825100 | 10773 | 4 | 1953574 | 2 | rust-wasm-reviewer |
| 2026-11-20T13:11:42Z | revision | agente:abca2794732b167b2 | 28 | 120494 | 1353502 | 12392 | 2 | 1486416 | 2 | code-quality-reviewer |
| 2026-11-20T13:17:24Z | revision | agente:a6ca23dd30b9cf77f | 42 | 119028 | 2008837 | 21109 | 2 | 2149016 | 3 | rust-wasm-reviewer |
| 2026-11-20T13:17:24Z | revision | agente:ace6396f2f01a22a1 | 26 | 105562 | 1131385 | 14077 | 2 | 1251050 | 3 | code-quality-reviewer |
