---
id: TASK-079
titulo: "F3-5b status y roadmap siempre frescos"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-079-f3-5b-status-y-roadmap-siempre-frescos
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1276709
tokens_implementacion: 32820865
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-17
dependencias: [TASK-032]
---
## Objetivo

`status` y `roadmap` siempre frescos: traen la rama de estado antes de mostrar y dicen la hora de la última sincronización (RR-12)

Tarea F3-5b del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F3-5.

Contexto (Carlos, 2026-11-14, RR-12 reforzado): el trabajo con varias personas debe estar garantizado: todos saben qué tarea tiene cada uno y no se pueden pisar ni los nombres ni las tareas. Además de lo ya previsto (IDs por push atómico, toma atómica en start, roadmap con quién lleva cada tarea, status), se cierran los huecos: nombres de rama y títulos únicos, identidad real de la plataforma (no texto libre), liberar y reasignar tareas abandonadas, aviso de solape de ficheros entre tareas en curso y estado siempre fresco con hora de sincronización. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 ter.

## Criterios de aceptacion
- [ ] C1: con dos clones A y B de un remoto bare, A publica con `tc2 new` y `tc2 status --json` en B, sin nada antes, ve la tarea nueva con el `commit=` que A publico; igual con `next --json`, `board --sin-plataforma --json` y `roadmap --json`
- [ ] C2: sin red (tras un `status` con red), `status` sale 0 con la ultima foto, `remoto=sin-red`, `sincronizado=` igual a la hora del ultimo fetch con exito y un aviso en stderr con esa hora y el «hace N»; sin sincronizar nunca, `sincronizado=nunca` con aviso
- [ ] C3: la hora sale en texto (linea de `status` y cabecera de `board`; en `roadmap`, stderr) y en `--json` de las cuatro ordenes con `sincronizado` y `remoto`
- [ ] C4: tras las cuatro ordenes con red no cambian `refs/heads/taskcodetwo/tareas`, `git status --porcelain` del principal ni de `tareas/`, el bare no gana commits, el registro del puerto no tiene `push`, `commit-tree` ni `update-ref`; solo cambian `refs/remotes/origin/taskcodetwo/tareas` y `.git/taskcodetwo/sincronizacion`
- [ ] C5: tras un `tc2 new` con exito, `status` sin red da `sincronizado=` igual o posterior a la hora de ese `new`
- [ ] C6: con credenciales denegadas sale 0 con `remoto=denegado` y aviso; sin remoto no hace fetch y sale `remoto=sin-remoto`
- [ ] C7: `status` dice `pendientes=N` con las refs de `refs/taskcodetwo/pendientes/` y avisa `tareas=con-cambios|diverge` con codigo 0
- [ ] C8: `doctor` no hace fetch (no aparece en su registro) y muestra `sincronizado=`
- [ ] C9: el stdout de `roadmap` es byte a byte el ROADMAP que `--escribir` publicaria

## Autorrevision

- rust-wasm-reviewer — 69d39bd
- code-quality-reviewer — 69d39bd

Hallazgos propios (ninguno CRITICO ni IMPORTANTE):

- MENOR: riesgos del plan sin arreglar: sin tiempo limite para un host
  que no responde, `GCM_INTERACTIVE` sin fijar, y dos `fetch` a la vez
  pueden dar `cannot lock ref` clasificado como `sin-red`.
- Desviaciones del plan: `traer_estado` devuelve `Traida` (la transaccion
  necesita el stderr); `Remoto` gana `SinRama`; la transaccion renueva la
  marca tras un push con exito (C5 en un remoto vacio); `pendientes=N` con
  `pendientes::contar`; firmas de `status_en`, `board_en` y
  `siguiente::orden` cambian.
- Tests anteriores ajustados: `siguiente.rs` (sin red con ssh que rechaza),
  `tablero.rs` (comparacion sin la hora).
- Mutaciones comprobadas (7, rojo y restauradas): registrar la marca en
  `traer_estado`, fetch en `leer_fresco`, `aviso`, `diverge`, hora en el
  stdout de `roadmap`, `doctor` sin fetch, marca tras el push.
- Verificacion propia: `--test frescura --test tablero --test siguiente`
  21 de 21.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T16:04:12Z | plan | automatico | persona | 0.16.1 |
| 2026-11-17T16:10:32Z | approve | automatico | persona | 0.16.1 |
| 2026-11-17T16:10:35Z | start | automatico | persona | 0.16.1 |
| 2026-11-17T17:01:27Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-17T16:10:29Z | diseno | agente:a802b338c5472820b | 36 | 99024 | 1160524 | 17125 | 31 | 1276709 | - | - |
| 2026-11-17T17:01:23Z | implementacion | agente:a16a3488b3179ab29 | 254 | 666368 | 29853976 | 111297 | 48 | 30631895 | - | - |
| 2026-11-17T17:01:27Z | implementacion | sesion:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 18 | 13901 | 2171331 | 3720 | 0 | 2188970 | - | - |
