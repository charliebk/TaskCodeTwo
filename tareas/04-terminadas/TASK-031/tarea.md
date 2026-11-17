---
id: TASK-031
titulo: "F3-4 tc2 roadmap: la vista para todos"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-031-f3-4-roadmap-la-vista-para-todos
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: 31389330
tokens_revision: 14889520
creado: 2026-11-14
actualizado: 2026-11-17
dependencias: [TASK-018]
---
## Objetivo

`roadmap`: la vista para todos (en curso con quién, en revisión, planificadas por hito, terminadas, descartadas) con dependencias y Mermaid, regenerada en la rama de estado tras cada cambio (RR-10, RR-11)

Tarea F3-4 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-3.

## Criterios de aceptacion
- [x] C1: dos clones lanzan `tc2 new` a la vez y los dos acaban `ok`; el `ROADMAP.md` final tiene las dos tareas, `git log --merges taskcodetwo/tareas` vacio y `tc2 roadmap --comparar` al dia (ampliando `tests/alta.rs` y `tests/estado_concurrencia.rs`)
- [x] C2: editada la zona manual, tras `new`, `import` y `roadmap --escribir` sigue identica byte a byte, tambien con CRLF (tests en `roadmap/zona.rs` y `tests/roadmap.rs`)
- [x] C3: `for-each-ref`, `status --porcelain` y el arbol de `HEAD` antes y despues de `new`, `import`, `roadmap` y `roadmap --escribir`: solo cambian `refs/heads/taskcodetwo/tareas` y su ref remota; `roadmap` y `--comparar` no cambian ninguna ref
- [x] C4: sobre la instantanea de ToolsaWeb y en un repo de prueba, «Listas para empezar» y «En marcha» del ROADMAP son iguales a `listas=` y `en-marcha=` de `tc2 next`
- [x] C5: la salida de referencia cubre cada seccion (con quien, revision, hito con dependencias y Mermaid, terminadas y descartadas) y `--escribir` dos veces seguidas da `sin-cambios` la segunda
- [x] C6: con la instantanea de ToolsaWeb (titulos de 60 caracteres) el fichero no pasa de 100 KB y cada bloque Mermaid tiene menos de 500 aristas y menos de 50 000 caracteres

## Resultado

Implementado en `crates/tc2/src/situacion.rs` (calculo comun de `next` y `roadmap`), `crates/tc2/src/roadmap/` (`zona.rs`, `generar.rs`, `mermaid.rs`, `mod.rs` con `de_estado`, la operacion `Regenerar` y la orden) y `estado::Contexto::nuevo`, que ahora recibe el generador explicito (alta e import lo pasan; el montaje no).

Comprobaciones previas del plan:
- `Documento` da `titulo` y `asignado_a` en los dos formatos (`TareaV1.asignado_a` siempre `Heredado`): verificado en `tarea/modelo.rs` y por el test de referencia (TASK-003 y TASK-008 en formato 1).
- El camino sin red no regenera: `transaccion::sin_red` no llama al generador; como `new`, `import` y `roadmap --escribir` exigen red, nunca quedan pendientes, y las pendientes se re-ejecutan en un intento con red, que si regenera.

Test por criterio:
- C1: `tests/alta.rs::dos_clones_crean_a_la_vez_con_ids_distintos` (binario, ROADMAP con las 6 tareas y `--comparar` ok en los dos clones) y `tests/estado_concurrencia.rs::dos_clones_dan_de_alta_a_la_vez_y_el_roadmap_queda_al_dia` (dos procesos, generador de produccion, sin merges).
- C2: `roadmap/zona.rs::extrae_byte_a_byte_con_lf_y_crlf` y `tests/roadmap.rs::zona_manual_editada_sobrevive_a_new_import_y_escribir` (zona con CRLF tras `--escribir`, `new`, `import` y otro `--escribir`).
- C3: `tests/roadmap.rs::solo_cambia_la_rama_de_estado`.
- C4: `tests/roadmap.rs::toolsaweb_listas_y_en_marcha_como_next_y_su_roadmap` y `roadmap_y_next_coinciden_y_escribir_es_idempotente`.
- C5: `roadmap::tests::salida_de_referencia` (fixture `tests/fixtures/roadmap/referencia.md`) y `roadmap_y_next_coinciden_y_escribir_es_idempotente`.
- C6: `tests/roadmap.rs::toolsaweb_cabe` (90 548 bytes, 14 hitos abiertos, todos con su Mermaid) y `mermaid::tests::grafo_demasiado_grande_es_una_nota`.

Verificacion sobre 3a1dd0f, sin otra en paralelo: `cargo xtask verificar --perfil rapida` en verde (`cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`: 299 pasados, 0 fallidos, 5 ignorados, sumados de las 30 lineas «test result»; codigo de salida 0).

Pendiente para Carlos (D7, no se publico nada en GitHub): pegar en un repo de prueba de GitHub el `ROADMAP.md` que da `tc2 roadmap` (o `tests/fixtures/roadmap/referencia.md`) y comprobar que cada bloque Mermaid se pinta.

Desviaciones del plan:
- `de_estado` recibe ademas la lista de personas (D6); `roadmap::generador(&personas)` le da la forma del tipo `Roadmap`. Las personas salen de `origin/develop`; sin lista, el texto heredado se muestra tal cual.
- D4: la zona editada en `tareas/ROADMAP.md` se prepara con `git add` antes de publicar y, tras publicar, la carpeta se deja como lo publicado y se adelanta; sin esto el fast-forward de `tareas/` falla con el fichero cambiado.
- `--comparar` desfasado o con zona rota sale `rechazada` (3), util en la CI.
- Para que ToolsaWeb quepa en 100 KB (daba 106 007 bytes): los nodos de borde del Mermaid van sin titulo y cada bloque solo lleva los `classDef` que usa.
- La zona por defecto es la de la 0.16 con «tc2» en lugar de «taskctl».
- `estado::ROADMAP` es el unico literal del nombre del fichero (antes `transaccion.rs` lo tenia a mano).

Hallazgos de la autorrevision (sin CRITICO ni IMPORTANTE):
- MENOR (corregido): el nombre `ROADMAP.md` estaba en dos sitios (`transaccion.rs` y el modulo nuevo).
- MENOR (no corregido): una lista de personas ilegible se trata como vacia sin avisar; el heredado se muestra tal cual.
- MENOR (no corregido): dos clones con distinto `origin/develop` pueden convertir distinto un heredado y `--comparar` lo vera desfasado.
- MENOR (no corregido): si `--escribir` falla tras preparar la zona editada, `tareas/ROADMAP.md` queda preparado (`M `) en vez de modificado; no se pierde nada.
- MENOR (no corregido): tras publicar, `tareas/ROADMAP.md` se sobrescribe con lo publicado; si la persona lo edita justo en ese momento, ese ultimo cambio se pierde.
- MENOR (no verificado): con `core.autocrlf=true`, una zona editada con finales mixtos se normaliza al prepararla; los tests usan `autocrlf=false`.


Revision (rust-wasm-reviewer y code-quality-reviewer):
- Ronda 1: calidad aprobada; rust cambios solicitados por IMP-1 (`--escribir` pisaba en silencio una zona manual publicada por otro). Corregido en 48d4390 (`motivo=zona-divergente`, la edicion de la persona se conserva) junto con MEN-1/MEN-2 rust y MEN-1/MEN-3/MEN-6 calidad (154c1e5, e0cc487); fallo nuevo del primer ROADMAP con H0 corregido en `situacion.rs`.
- Ronda 2 (solo rust; calidad ya habia aprobado): aprobada, IMP-1 verificado con dos clones y mutacion en rojo.

Pendiente para Carlos (D7): pegar `tests/fixtures/roadmap/referencia.md` (o la salida de `tc2 roadmap`) en un repo de prueba de GitHub y comprobar que cada bloque Mermaid se pinta.

MENORES aceptados (a seguimiento, TASK-123): `restaurar_indice` sin test que lo ejercite; ventana de milisegundos al adelantar la carpeta tras publicar (MEN-4 calidad); heredado convertido distinto con `origin/develop` distinto (MEN-5 calidad); zona con CRLF puro o `` sueltos sin probar (MEN-7 calidad); un ROADMAP de la 0.16 con texto dentro de la marca de fin pasa a `roadmap-roto` (consecuencia buscada de MEN-2).

## Autorrevision

- rust-wasm-reviewer — 3a1dd0f
- code-quality-reviewer — 3a1dd0f

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T06:49:37Z | plan | automatico | persona | 0.16.1 |
| 2026-11-17T06:54:06Z | approve | automatico | persona | 0.16.1 |
| 2026-11-17T06:54:07Z | start | automatico | persona | 0.16.1 |
| 2026-11-17T07:34:02Z | review | automatico | persona | 0.16.1 |
| 2026-11-17T08:04:46Z | review | automatico | persona | 0.16.1 |
| 2026-11-17T08:16:27Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-17T07:34:02Z | implementacion | sesion:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 16 | 14813 | 4509759 | 2246 | 0 | 4526834 | - | - |
| 2026-11-17T07:34:02Z | implementacion | agente:aaa954d4d28d87f09 | 242 | 317775 | 25506272 | 126223 | 21 | 25950512 | - | - |
| 2026-11-17T07:50:35Z | revision | agente:ace2aa3dc468d04ff | 100 | 137119 | 5205970 | 27519 | 6 | 5370708 | 1 | rust-wasm-reviewer |
| 2026-11-17T07:50:36Z | revision | agente:a1d85b913c26ea9d8 | 102 | 161017 | 6127438 | 36154 | 31 | 6324711 | 1 | code-quality-reviewer |
| 2026-11-17T08:04:46Z | implementacion | sesion:dif:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 2 | 1039 | 910619 | 324 | 0 | 911984 | - | - |
| 2026-11-17T08:13:07Z | revision | agente:a99bbfd808af23319 | 66 | 115814 | 3063597 | 14624 | 3 | 3194101 | 2 | rust-wasm-reviewer |

## Revisores no relanzados

- ronda 2: code-quality-reviewer - solo tenia hallazgos MENOR pendientes y la correccion no toca sus ficheros

## Duracion

- Estimada: 300 min
- Real (de la rama abierta al cierre con merge, sin pausas): 82 min
- Paso de la hora: 82 min reales frente a 300 estimados
- Cierre (finish): 199 s, de 2026-11-17T08:13:08Z a 2026-11-17T08:16:27Z
