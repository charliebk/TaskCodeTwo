---
id: TASK-042
titulo: "F5-2 Cache por SHA del arbol compartida"
tipo: feature
sprint: 5
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-042-f5-2-cache-por-sha-del-arbol-compartida
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 293610
tokens_implementacion: 14727181
tokens_revision: 3917496
creado: 2026-11-14
actualizado: 2026-11-19
dependencias: [TASK-041]
---
## Objetivo

Caché por SHA del árbol compartida por PR, cola de merge y release

Tarea F5-2 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F5-1.

## Criterios de aceptacion
- [x] Cero repeticiones sobre el mismo SHA, medido
- [x] Cumple lo que pide la tarea F5-2 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-19T19:19:46Z | plan | automatico | persona | 0.16.1 |
| 2026-11-19T20:06:12Z | approve | automatico | persona | 0.16.1 |
| 2026-11-19T20:06:14Z | start | automatico | persona | 0.16.1 |
| 2026-11-19T20:59:49Z | review | automatico | persona | 0.16.1 |
| 2026-11-19T21:05:32Z | review | automatico | persona | 0.16.1 |
| 2026-11-19T21:31:39Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-19T20:06:12Z | diseno | agente:a1c0b5a7a377327c4 | 16 | 55667 | 225327 | 12600 | 15 | 293610 | - | - |
| 2026-11-19T20:59:49Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 20 | 13796 | 3904494 | 4611 | 1 | 3922921 | - | - |
| 2026-11-19T20:59:49Z | implementacion | agente:a446c748ef3c41da7 | 128 | 211612 | 10520406 | 72114 | 15 | 10804260 | - | - |
| 2026-11-19T21:08:47Z | revision | agente:ab3b1676dd892ed94 | 22 | 113994 | 1002778 | 6878 | 3 | 1123672 | 1 | rust-wasm-reviewer |
| 2026-11-19T21:08:47Z | revision | agente:a1b6953b84b99d090 | 22 | 139957 | 1160061 | 24647 | 2 | 1324687 | 1 | code-quality-reviewer |
| 2026-11-19T21:08:47Z | revision | agente:a75a1e652b1d4687f | 28 | 122627 | 1325764 | 20718 | 3 | 1469137 | 2 | code-quality-reviewer |

## Revisores no relanzados

- ronda 2: rust-wasm-reviewer - solo tenia hallazgos MENOR pendientes y la correccion no toca sus ficheros

## Resultado

Cache por SHA del arbol en `.github/workflows/verificacion.yml` (F5-2, F1-7 decisiones 2-4, F1-8 decision 7): `buscar` toma V, padres y abuelos del merge ref (`fetch-depth: 3`), lee los check-runs `verificacion` de la App 15368 (`filter=all`), acepta solo los enlazados por su check suite a una ejecucion del workflow de TaskCode en la misma version, y rechaza toda linea si el diff frente a la base toca `.github/workflows/**` o `.taskcode/**` (`--no-renames`). `clave = sha256(perfil|runner|huella-arbol|version|ordenes)`. `ejecutar` calcula la huella local y, si acierta, el paso del comando sale `skipped`. El publicador relee el arbol por la API y escribe una linea por perfil `arbol= perfil= runner= clave= local= origen= ejecutada|reutilizada de <sha>` (contrato para F8-1). Entradas nuevas `huella-arbol` y `huella-local`. GitLab: la linea va en un fichero del artefacto del informe, sin busqueda (D2). El lint admite solo lecturas de `contents`, `checks` y `actions` y prohibe el checkout en un job que escribe. Guia: seccion de la cache y divergencias 11-17.

Evidencia (2026-11-19):
- C1, medido en `charliebk/taskcodetwo-e2e-github` (PR #13, workflow en 132a9dd; guion y salida en `medicion/`): sobre cada arbol el paso del comando se ejecuto una vez; la re-ejecucion (run 44607911390 intento 2) y el commit vacio (44608246992) salen `reutilizada de 10d5e76` con el paso `skipped`; cambiar un fichero (44608436324) y tocar `.github/workflows/**` (44608660673, 44608801152) salen `ejecutada`, el ultimo por la regla de rutas. Repo E2E limpio (PR cerrados sin mergear, ramas borradas).
- Verificacion completa en verde sobre aee1621; la correccion de la ronda 1 (4f4a62d, una opcion de `git diff` y las ordenes en la clave) se probo con un repo temporal y `cargo test -p xtask` (26/26); no se volvio a medir en el E2E. `finish` vuelve a verificar.

Revision por pares:
- Ronda 1: rust-wasm-reviewer aprobada (MEN-1 lecturas admitidas iguales para todas las clases, aceptado); code-quality-reviewer cambios-solicitados (IMP-1 renombrados no detectados por la regla de rutas; MEN-1 ordenes fuera de la clave; MEN-2 fuera de PR la regla solo mira HEAD^1, aceptado). IMP-1 y MEN-1 corregidos en 4f4a62d.
- Ronda 2: code-quality-reviewer aprobada.

Divergencias (en la guia): `github.job_workflow_sha` llega vacio en el reutilizable y la version se lee de `referenced_workflows`; GitHub ignora el `details_url` de un check-run creado con `GITHUB_TOKEN` y el enlace se hace por `check_suite_id`; un re-run saca el check anterior de `latest` y se busca con `filter=all`; una huella local volatil (`uname -sr`) impide el acierto (F1-11); el E2E uso una rama base con el mismo llamador para no tocar siempre los workflows; cola de merge sin probar (exige organizacion); `bash` en runner Windows sin probar.

## Duracion

- Estimada: 110 min
- Real (de la rama abierta al cierre con merge, sin pausas): 85 min
- Paso de la hora: 85 min reales frente a 110 estimados
- Cierre (finish): 1316 s, de 2026-11-19T21:09:43Z a 2026-11-19T21:31:39Z
