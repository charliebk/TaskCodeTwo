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
tokens_implementacion: 52351018
tokens_revision: 11782150
creado: 2026-11-14
actualizado: 2026-11-17
dependencias: [TASK-032]
---
## Objetivo

`status` y `roadmap` siempre frescos: traen la rama de estado antes de mostrar y dicen la hora de la última sincronización (RR-12)

Tarea F3-5b del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F3-5.

Contexto (Carlos, 2026-11-14, RR-12 reforzado): el trabajo con varias personas debe estar garantizado: todos saben qué tarea tiene cada uno y no se pueden pisar ni los nombres ni las tareas. Además de lo ya previsto (IDs por push atómico, toma atómica en start, roadmap con quién lleva cada tarea, status), se cierran los huecos: nombres de rama y títulos únicos, identidad real de la plataforma (no texto libre), liberar y reasignar tareas abandonadas, aviso de solape de ficheros entre tareas en curso y estado siempre fresco con hora de sincronización. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 ter.

## Criterios de aceptacion
- [x] C1: con dos clones A y B de un remoto bare, A publica con `tc2 new` y `tc2 status --json` en B, sin nada antes, ve la tarea nueva con el `commit=` que A publico; igual con `next --json`, `board --sin-plataforma --json` y `roadmap --json`
- [x] C2: sin red (tras un `status` con red), `status` sale 0 con la ultima foto, `remoto=sin-red`, `sincronizado=` igual a la hora del ultimo fetch con exito y un aviso en stderr con esa hora y el «hace N»; sin sincronizar nunca, `sincronizado=nunca` con aviso
- [x] C3: la hora sale en texto (linea de `status` y cabecera de `board`; en `roadmap`, stderr) y en `--json` de las cuatro ordenes con `sincronizado` y `remoto`
- [x] C4: tras las cuatro ordenes con red no cambian `refs/heads/taskcodetwo/tareas`, `git status --porcelain` del principal ni de `tareas/`, el bare no gana commits, el registro del puerto no tiene `push`, `commit-tree` ni `update-ref`; solo cambian `refs/remotes/origin/taskcodetwo/tareas` y `.git/taskcodetwo/sincronizacion`
- [x] C5: tras un `tc2 new` con exito, `status` sin red da `sincronizado=` igual o posterior a la hora de ese `new`
- [x] C6: con credenciales denegadas sale 0 con `remoto=denegado` y aviso; sin remoto no hace fetch y sale `remoto=sin-remoto`
- [x] C7: `status` dice `pendientes=N` con las refs de `refs/taskcodetwo/pendientes/` y avisa `tareas=con-cambios|diverge` con codigo 0
- [x] C8: `doctor` no hace fetch (no aparece en su registro) y muestra `sincronizado=`
- [x] C9: el stdout de `roadmap` es byte a byte el ROADMAP que `--escribir` publicaria

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

## Resultado

Hecho: `status`, `board`, `next` y `roadmap` traen la rama de estado antes de mostrar (`estado::leer_fresco`, un solo `fetch` comun `traer_estado` que tambien usan la transaccion y `montar`) y dicen la hora de la ultima sincronizacion (`.git/taskcodetwo/sincronizacion`, que renuevan lectura, publicacion y `montar`): claves `remoto=consultado|sin-red|denegado|sin-remoto|sin-rama|rechazado|error-local` y `sincronizado=<UTC>|nunca`, aviso en stderr con «hace N» cuando el remoto falla, y el stdout de `roadmap` sigue siendo byte a byte el ROADMAP. `status` dice ademas `pendientes` (texto con N>0 y JSON) y avisa `tareas=con-cambios|diverge`. `doctor` no hace fetch y muestra `sincronizado=`. Una marca con hora futura cuenta como «nunca». Rama de estado reescrita en origin o ref remota bloqueada: la lectura lo dice (`rechazado`, `error-local`) en vez de «sin red» y la escritura sale `error` sin dejar pendiente (antes las tomaba por `sinred`). La lectura reintenta dos veces ante una ref bloqueada. Adendas en F1-2 y F1-4. D1a-D6a. `cargo xtask verificar --perfil rapida` en verde.

No verificado: contra GitHub o GitLab reales (coste de ~1 s por lectura solo extrapolado); el cableado del reintento en `leer_fresco` no lo protege ningun test (exigiria un `.lock` soltado por otro proceso a mitad de la lectura).

Revision ronda 1: los dos sin CRITICOS ni IMPORTANTES; rust-wasm-reviewer aprobada (H1 fallos que no son de red salian `sin-red`; H2 marca futura mostrada como valida; H3 renovacion por la lectura sostenida por un solo test; H4 carrera lectura/transaccion; H5 test intermitente de `board`); code-quality-reviewer escribio «aprobada» y se registro `cambios-solicitados` porque Carlos pidio corregir todos los MENORES y la correccion tocaba ficheros no citados (adendas, i18n) (M1 `roadmap --comparar` sin test de frescura; M2 `pendientes` solo en JSON; M3 `montar` repetia el refspec; M4 brazo muerto). Corregidos en 2af3390, 68f054e y ef977f1; H5 aceptado (no reproducido en 18 ejecuciones; el test ahora vuelca valor, paso, tiempo, stderr y peticiones del mock). Ronda 2: aprobada por los dos; M5 (brazos `Reescrita`/`Bloqueada` de `montar` sin test) corregido en e0d4b21 y verificado con `verificar-menores`.

Nota de higiene: el commit 68f054e lleva un mensaje de tests pero incluye tambien el refactor de `montaje.rs` y `tablero.rs` (M3 y M2), porque el commit que iba aparte fallo; no se reescribio la historia.

Cierre: `taskctl finish` se cortaba porque la verificacion rapida ya pasa de 600 s en esta maquina (ejecutada aparte: verde, 36 grupos sin fallos); por decision de Carlos se sube `timeout_verificacion` a 1500 s en `.taskcode/config.yml` (4fe19b5, ad04288).

Riesgos aceptados: sin tiempo limite en el puerto para un host mudo; `GCM_INTERACTIVE` sin fijar; una lectura y una escritura a la vez en el mismo clon pueden hacer que la escritura salga `error` y haya que repetirla.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T16:04:12Z | plan | automatico | persona | 0.16.1 |
| 2026-11-17T16:10:32Z | approve | automatico | persona | 0.16.1 |
| 2026-11-17T16:10:35Z | start | automatico | persona | 0.16.1 |
| 2026-11-17T17:01:27Z | review | automatico | persona | 0.16.1 |
| 2026-11-17T18:03:21Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-17T16:10:29Z | diseno | agente:a802b338c5472820b | 36 | 99024 | 1160524 | 17125 | 31 | 1276709 | - | - |
| 2026-11-17T17:01:23Z | implementacion | agente:a16a3488b3179ab29 | 254 | 666368 | 29853976 | 111297 | 48 | 30631895 | - | - |
| 2026-11-17T17:01:27Z | implementacion | sesion:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 18 | 13901 | 2171331 | 3720 | 0 | 2188970 | - | - |
| 2026-11-17T17:19:18Z | revision | agente:a48175601d602934b | 52 | 112447 | 2870766 | 16595 | 5 | 2999860 | 1 | rust-wasm-reviewer |
| 2026-11-17T17:23:31Z | revision | agente:af58a9d4b7038777f | 52 | 143557 | 2940900 | 13774 | 3 | 3098283 | 1 | code-quality-reviewer |
| 2026-11-17T18:03:19Z | implementacion | agente:a6bb8a234a42dd73d | 194 | 406862 | 15809473 | 64809 | 49 | 16281338 | - | - |
| 2026-11-17T18:18:10Z | revision | agente:acb7cce17ed5d00b4 | 70 | 117127 | 3150037 | 12787 | 8 | 3280021 | 2 | code-quality-reviewer |
| 2026-11-17T18:26:48Z | revision | agente:ace8b66b51d6dfc7d | 48 | 112711 | 2279518 | 11709 | 4 | 2403986 | 2 | rust-wasm-reviewer |
| 2026-11-17T18:38:38Z | implementacion | agente:ac474771b4459c773 | 76 | 80121 | 3146334 | 22284 | 25 | 3248815 | - | - |

## Verificacion ligera

- 2026-11-17T18:38:21Z: ronda 2; commit revisado ef977f1ce8994e9da5faa4b817fb815f24af56ef; commit verificado e0d4b2180dd2f66bce5248e68cd55e547074d7d6; ficheros: crates/tc2/src/estado/montaje.rs; corregidos: M5; pendientes: ninguno
