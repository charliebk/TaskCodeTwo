---
id: TASK-082
titulo: "F3-11 Aviso de solape entre tareas en curso"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-082-f3-11-aviso-de-solape-entre-tareas-en-cu
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 651332
tokens_implementacion: 42002552
tokens_revision: 5037541
creado: 2026-11-14
actualizado: 2026-11-18
dependencias: [TASK-033]
---
## Objetivo

Aviso de solape: al tomar una tarea, `start` dice qué tareas en curso (y de quién) tocan los mismos ficheros o módulos (RR-12)

Tarea F3-11 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F3-6, F4-1.

Contexto (Carlos, 2026-11-14, RR-12 reforzado): el trabajo con varias personas debe estar garantizado: todos saben qué tarea tiene cada uno y no se pueden pisar ni los nombres ni las tareas. Además de lo ya previsto (IDs por push atómico, toma atómica en start, roadmap con quién lleva cada tarea, status), se cierran los huecos: nombres de rama y títulos únicos, identidad real de la plataforma (no texto libre), liberar y reasignar tareas abandonadas, aviso de solape de ficheros entre tareas en curso y estado siempre fresco con hora de sincronización. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 ter.

## Criterios de aceptacion
- [ ] C1: con dos clones (@ana y @luis) y TASK-001 y TASK-002 en curso con ramas publicadas que tocan `src/x/a.rs`, `start TASK-002` en el clon de luis sale 0 con una linea `aviso:` en stderr que nombra TASK-001, @ana y `src/x/a.rs`, y en la linea (y en `--json`) `solape=TASK-001`, `solape-ficheros=TASK-001:src/x/a.rs` y `solape-asignados=TASK-001:@ana`
- [ ] C2: con `src/x/a.rs` frente a `src/x/b.rs` sale el aviso con `solape-modulos=TASK-001:src/x` y sin `solape-ficheros`
- [ ] C3: con `src/x/a.rs` frente a `web/y/c.ts` no sale ninguna linea `aviso:` de solape ni ninguna clave `solape*`
- [ ] C4: resultado, codigo, toma publicada, rama y worktree son los mismos con solape y sin el
- [ ] C5: tras el `fetch` de `start`, el solape solo lanza `for-each-ref`, `diff` y `cat-file` (nunca `fetch`, `ls-remote` ni `push`); un repo sin `origin` no da claves `solape*`
- [ ] C6: una tarea en curso sin `origin/<rama>` no da aviso y sale `solape-sin-rama=TASK-NNN`
- [ ] C7: una tarea `terminada` cuya rama conservada toca el mismo fichero no da aviso
- [ ] C8: `Cargo.toml` frente a `README.md` en la raiz no da aviso de modulo; el mismo `Cargo.toml` en las dos da aviso de fichero
- [ ] C9: en el primer `start` (rama propia sin commits) sale una linea `nota:` (no `aviso:`) con las tareas en curso, su @ y sus modulos tocados (como mucho 5 por tarea) y `en-curso-modulos=` en la linea
- [ ] C10: el registro del puerto muestra exactamente 1 + k + 1 procesos de git del solape, con k = tareas en curso con rama publicada
- [ ] C11: un login o una ruta con caracteres de control salen saneados

## Autorrevision

- rust-wasm-reviewer — 277a5b9
- code-quality-reviewer — 277a5b9

Hallazgos propios (ninguno CRITICO ni IMPORTANTE de esta tarea):

- Fuera de esta tarea (de TASK-084): `progreso::c3_transiciones_reales_reescriben_el_progreso`
  es intermitente bajo carga (2 de 4); causa probable: `progreso::fichero::escribir`
  ignora el error al rotar `.anterior.json` y en Windows el rename puede no entrar
  a tiempo. Pasa a una tarea nueva.
- MENOR (previo, todas las claves): U+202E y otros caracteres de formato no se
  escapan en la linea de contrato (el contrato solo escapa controles); en stderr
  si salen saneados.
- MENOR: el aviso cuenta tambien las tareas de la misma persona.
- Desviaciones del plan: las tareas en curso salen de `Ficha::de` reutilizando la
  lectura de `Tomar` (con `situacion::calcular` habria un tercer `cat-file` y se
  rompia `estimacion::r1_05`); clave nueva `solape-incompleto=TASK-NNN` para un
  diff que falla; sin otras tareas en curso no se lanza ningun proceso;
  `tests/liberar.rs` acepta una linea `nota:` en stderr en un exito; el aviso de
  `start` es otra excepcion a «diagnostico solo si falla» (en la adenda de F1-17).
- Mutaciones comprobadas (7, rojo y restauradas): filtro de estado, modulo de la
  raiz, modulo con fichero comun, saneado, corte a 20, rama no publicada, sin
  refs de `origin`.
- Verificacion propia: `--test solape` 10 de 10.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-18T08:45:26Z | plan | automatico | persona | 0.16.1 |
| 2026-11-18T08:50:06Z | approve | automatico | persona | 0.16.1 |
| 2026-11-18T08:50:07Z | start | automatico | persona | 0.16.1 |
| 2026-11-18T12:02:52Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-18T08:50:06Z | diseno | agente:acde2811b6529af53 | 26 | 70403 | 564348 | 16555 | 19 | 651332 | - | - |
| 2026-11-18T12:02:51Z | implementacion | agente:a54a98153a1f299a8 | 212 | 1666931 | 23718530 | 88938 | 51 | 25474611 | - | - |
| 2026-11-18T12:02:52Z | implementacion | sesion:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 42 | 742737 | 9842493 | 6932 | 0 | 10592204 | - | - |
| 2026-11-18T12:12:17Z | revision | agente:a0349b1d9f1162ba2 | 50 | 125365 | 2471576 | 11360 | 2 | 2608351 | 1 | code-quality-reviewer |
| 2026-11-18T12:19:59Z | revision | agente:a27b98544e3982df8 | 46 | 120136 | 2294100 | 14908 | 1 | 2429190 | 1 | rust-wasm-reviewer |
| 2026-11-18T12:43:15Z | implementacion | agente:ad2dce0be43478d68 | 96 | 153847 | 5736666 | 45128 | 20 | 5935737 | - | - |

## Verificacion ligera

- 2026-11-18T12:43:16Z: ronda 1; commit revisado 1866457c7b052bf73ffa8205973f0d364b28b111; commit verificado 5bb28557661b6c8b423cfb36a85bbf0086b1d856; ficheros: crates/tc2/src/solape.rs, crates/tc2/tests/liberar.rs, crates/tc2/tests/solape.rs; corregidos: R1-01, R1-02, R1-03, R1-04, H1, H3; pendientes: H2
