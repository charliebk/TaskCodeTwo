---
id: TASK-017
titulo: "F2-2 Capa de operaciones Git deterministas"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-017-f2-2-capa-de-operaciones-git-determinist
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 235247
tokens_implementacion: 6797656
tokens_revision: 4324166
creado: 2026-11-14
actualizado: 2026-11-15
dependencias: [TASK-005, TASK-016]
---
## Objetivo

Capa de operaciones Git deterministas: rama, worktree, commit de lo escrito, push con reintento, tag (P5)

Tarea F2-2 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-4, F2-1.

## Criterios de aceptacion
- [x] Tests contra repos reales
- [x] salida ≤ 5 líneas en el caso normal
- [x] Cumple lo que pide la tarea F2-2 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-15T12:29:47Z | plan | automatico | persona | 0.16.1 |
| 2026-11-15T12:33:25Z | approve | automatico | persona | 0.16.1 |
| 2026-11-15T12:33:27Z | start | automatico | persona | 0.16.1 |
| 2026-11-15T12:49:09Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T13:05:50Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-15T12:33:18Z | diseno | agente:a5e8e6789a0fd006c | 14 | 43470 | 180062 | 11701 | 15 | 235247 | - | - |
| 2026-11-15T12:49:09Z | implementacion | sesion:0c02a589-064f-41b2-898e-aecc1141ae21 | 60 | 92766 | 6631285 | 73545 | 3 | 6797656 | - | - |
| 2026-11-15T12:57:32Z | revision | agente:a0fc86c42bdd6b290 | 38 | 143695 | 2682613 | 29127 | 1 | 2855473 | 1 | rust-wasm-reviewer |
| 2026-11-15T13:11:42Z | revision | agente:a9cc0cf9ae42e9cfd | 26 | 128420 | 1320945 | 19302 | 2 | 1468693 | 2 | rust-wasm-reviewer |

## Autorrevision

- rust-wasm-reviewer — 743a5d3
- rust-wasm-reviewer — ca600a5

## Resultado

**Implementado** (Rust, sin dependencias nuevas; manda la ADR F1-4):

- `crates/tc2/src/contrato.rs`: `Resultado` (9 resultados) con la tabla de
  codigos 0–7 en un unico sitio (la fija F2-3), `Informe` con la linea
  `<op> <resultado> clave=valor...` (valores sin espacios, escapados `%XX` de
  forma reversible, hashes a 12), `--json` en una linea y diagnostico solo en
  stderr y solo si falla.
- `crates/tc2/src/git/puerto.rs`: puerto `Git` que lanza el binario `git` con
  el entorno fijo de F1-4 (`LC_ALL=C`, `LANG=C`, `GIT_TERMINAL_PROMPT=0`,
  `GIT_PAGER=cat`, `GIT_EDITOR=true`, `color.ui=never`, `core.quotepath=false`)
  y sin `GIT_DIR`/`GIT_WORK_TREE`/`GIT_INDEX_FILE` heredados. Lista permitida
  como `enum` (16 subcomandos: `reset`, `rebase`, `stash`, `clean` no se pueden
  escribir) y argumentos prohibidos rechazados antes de lanzar nada. Registro
  de cada llamada.
- `crates/tc2/src/git/`: `fetch`, `validar-base`, `comprobar-id`,
  `crear-rama-y-worktree`, `reanudar`, `localizar-principal`, `guardar`,
  `push` (3 intentos, 1 s y 2 s), `actualizar-rama`, `actualizar-principal`,
  `quitar-worktree`, `tag-anotado` y `diff-tarea`. Publicas en la biblioteca;
  en el binario solo `tc2 guardar [--json] "<mensaje>" <rutas...>` y
  `tc2 actualizar-rama [--json]` (D1a).
- `crear-rama-y-worktree`/`reanudar` anaden `/.taskcode/worktrees/` a
  `info/exclude` del repo comun (no versionado) para que el principal no vea
  los worktrees de tarea como ficheros sin seguimiento.
- Tests: 11 unitarios en la biblioteca y 6 ficheros de integracion contra repos
  reales con remoto bare local (`git_ciclo`, `git_conflicto`, `git_push`,
  `git_binario`, `git_casos_limite` y el helper `tests/comun`), en
  `std::env::temp_dir()/tc2-tests`, borrados al terminar.

**Criterios:**

- C1 Tests contra repos reales: ciclo completo con cada paso dos veces y
  huella de refs, worktrees y status igual tras la segunda; conflicto (codigo
  6, sin rebase ni force, merge de dos padres); push con fallo transitorio
  (`intentos=2`), non-fast-forward sin reintento (3), tags en otro commit
  (local y remoto, sin publicar), sin red (4).
- C2 Salida ≤ 5 lineas: `comun::contrato` comprueba cada salida (≤ 5 lineas,
  sin ESC, primera linea del contrato, claves en minusculas, hashes de 12,
  stderr vacio en exito); el binario como proceso da una linea.
- C3 F2-2 y requisitos: RF-23 (solo las rutas dadas, `--only`, indice ajeno
  respetado), RN-05/H9 (registro de las 4 suites contra un detector de lista
  permitida independiente; 30 variantes prohibidas en el test del puerto),
  RR-01/RF-25 (contrato), salida identica con `LANG/LC_ALL=es_ES.UTF-8`.
- Verificacion: `cargo xtask verificar --perfil completa` en verde en Windows
  (build reproducible: mismo sha256 `8ff393fb...2490`, 323 072 bytes, en las
  dos rutas). CI run 43715509044 (primer commit de tests) y 43717203196
  (correcciones, c943b1c): 4 jobs `success` en ubuntu, macos y windows.
- Versiones de `git` en la CI (decision 6): ubuntu 2.56.1, macos 2.56.0,
  windows 2.56.0.windows.4; local 2.56.0.windows.4.

**Revision por pares** (rust-wasm-reviewer, 2 rondas):

- Ronda 1, `cambios-solicitados`: 3 IMPORTANTES y 4 MENORES, todos corregidos
  en ca600a5 con su test:
  - IMPORTANTE-1: `quitar-worktree` con HEAD suelto perdia commits → rechaza
    `motivo=sin-rama`.
  - IMPORTANTE-2: `guardar` con `git mv`/`git rm` ya en el indice (commit a
    medias u `error`) → reparto de rutas por estado, origen del renombrado.
  - IMPORTANTE-3: 9 reglas del puerto sin test → tabla con 30 variantes.
  - MENOR-1: rechazo persistente del remoto (`[remote rejected]`) → `agotada`
    (5) con `motivo=remoto-rechaza`, no `sinred`.
  - MENOR-2: `--force` en mensajes o rutas daba `error` → no se miran el valor
    de `-m` ni las rutas tras `--`.
  - MENOR-3: test de `--only` con un fichero ajeno en el indice.
  - MENOR-4: lo publicado en la propia rama entra por fast-forward; la base,
    con `--no-ff`.
- Ronda 2, `aprobada`: los tres IMPORTANTES verificados con 20 mutantes, todos
  en rojo. Nota sin hallazgo: un worktree con HEAD suelto sin commits propios
  tambien se rechaza; decision de diseno (no se pierde nada, `reanudar` lo
  repone en la rama).

**Desviaciones y limites:**

- `comprobar-id` ocupado sale como `rechazada motivo=ocupada` (la PoC de F1-4
  escribia `ocupada` como resultado, fuera del conjunto del contrato).
- `actualizar-rama` hace `fetch` y, si la rama ya estaba publicada y va por
  delante, publica (es dueno de `push` segun F1-4); tambien trae lo publicado
  por otra persona en la misma rama, que es lo que pide el diagnostico del
  non-fast-forward.
- Fuera de alcance, como fijo el plan: `commit-estado` y pendientes sin red
  (F2-3), puerto Plataforma (F2-6/F2-9), `start` y slug (F3-6), `release`,
  `hotfix` y `notas-release` (F7-3), `doctor` (F3-8), hook de D4a; el diff
  completo a peticion (F6-2).
- **No verificado**: Git traducido (ningun runner se comprobo con catalogos de
  traduccion; el test de locale pasa, pero puede ser trivial, como en F1-4);
  rutas no UTF-8; `git` anterior a 2.56.
- Coste: los tests de integracion tardan 4–18 s cada uno en local (Windows) y
  0,2–7 s en la CI; no se separan como pesados.
