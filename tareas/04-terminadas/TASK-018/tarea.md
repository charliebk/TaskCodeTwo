---
id: TASK-018
titulo: "F2-3 Rama de estado"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-018-f2-3-rama-de-estado
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 351031
tokens_implementacion: 11772269
tokens_revision: 29508739
creado: 2026-11-14
actualizado: 2026-11-15
dependencias: [TASK-002, TASK-003, TASK-017]
---
## Objetivo

Rama de estado: crear, montar como worktree en `tareas/`, leer y escribir de forma atómica (P1, P2)

Tarea F2-3 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-1, F1-2, F2-2.

## Criterios de aceptacion
- [x] Dos procesos escriben a la vez sin perder nada (`tests/estado_concurrencia.rs`: dos clones y dos procesos del mismo clon)
- [x] test (`cargo xtask verificar --perfil completa` en verde: 33 tests de tc2)
- [x] Cumple lo que pide la tarea F2-3 del plan y los requisitos que cita (`tests/estado_casos.rs`; adendas en F1-1, F1-2 y F1-4)

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-15T13:18:19Z | plan | automatico | persona | 0.16.1 |
| 2026-11-15T13:24:44Z | approve | automatico | persona | 0.16.1 |
| 2026-11-15T13:24:46Z | start | automatico | persona | 0.16.1 |
| 2026-11-15T13:52:56Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T14:21:47Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T14:35:32Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T14:45:13Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T14:54:30Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-15T13:24:44Z | diseno | agente:ad963b707cfa50972 | 14 | 77235 | 250282 | 23500 | 15 | 351031 | - | - |
| 2026-11-15T13:52:56Z | implementacion | sesion:0c02a589-064f-41b2-898e-aecc1141ae21 | 90 | 157794 | 11517081 | 97304 | 3 | 11772269 | - | - |
| 2026-11-15T14:14:07Z | revision | agente:aaee6d3c5fc9c5e05 | 56 | 196204 | 4414663 | 28648 | 11 | 4639571 | 1 | rust-wasm-reviewer |
| 2026-11-15T14:14:07Z | revision | agente:a3be1dff0db736040 | 96 | 216195 | 8142986 | 48079 | 10 | 8407356 | 1 | code-quality-reviewer |
| 2026-11-15T14:29:30Z | revision | agente:af38693194bfe9653 | 110 | 176252 | 6457777 | 44771 | 6 | 6678910 | 2 | code-quality-reviewer |
| 2026-11-15T14:29:30Z | revision | agente:a234246fefe90c88a | 34 | 135312 | 1805947 | 23206 | 1 | 1964499 | 2 | rust-wasm-reviewer |
| 2026-11-15T14:41:31Z | revision | agente:a4bb1806bca1301ea | 50 | 143472 | 2529559 | 20797 | 2 | 2693878 | 3 | code-quality-reviewer |
| 2026-11-15T14:41:31Z | revision | agente:a07493d29b6b10a53 | 34 | 130249 | 1763848 | 21128 | 2 | 1915259 | 3 | rust-wasm-reviewer |
| 2026-11-15T14:49:59Z | revision | agente:aaf68033a523f6908 | 32 | 119699 | 1347493 | 18736 | 1 | 1485960 | 4 | code-quality-reviewer |
| 2026-11-15T14:49:59Z | revision | agente:af65c249616d7d696 | 32 | 122744 | 1577084 | 23446 | 2 | 1723306 | 4 | rust-wasm-reviewer |

## Autorrevision

- rust-wasm-reviewer — d78931e
- code-quality-reviewer — d78931e
- rust-wasm-reviewer — f96b878
- code-quality-reviewer — f96b878
- rust-wasm-reviewer — 9e99510
- code-quality-reviewer — 9e99510
- rust-wasm-reviewer — 965ef23
- code-quality-reviewer — 965ef23

## Rondas extra

- Ronda 3: la decide Carlos (persona) al llegar al tope: la ronda 2 abrió dos IMPORTANTES introducidos por las correcciones de la ronda 1 (IMPORTANTE-2 de calidad y R2-01 de Rust), con arreglos acotados y test.
- Ronda 4: la decide Carlos (persona) al llegar otra vez al tope: la ronda 3 abrió el IMPORTANTE-3 (un `permission denied` de la red clasificado como fallo local, introducido por la corrección de la ronda 2), confirmado con un test que falla; arreglo acotado con test, junto al MENOR R3-01.

## Verificacion ligera

- 2026-11-15T14:50:48Z: ronda 4; commit revisado b2f565d50585629c79df41974f1d7af5ee648930; commit verificado 152c4ee1a79fa50ed34e539fd402bf4c3330ea2e; ficheros: crates/tc2/src/git/remoto.rs; corregidos: MENOR-1, R4-01; pendientes: ninguno

## Resultado

**Implementado** (Rust, `crates/tc2`, sin dependencias nuevas):

- Módulo `estado/`: `montar` crea la rama huérfana `taskcode/tareas` con un commit raíz hecho con `commit-tree` (sin `worktree add --orphan`, así que no exige Git 2.42) y la monta en `tareas/`; es idempotente y vuelve a montarla si la carpeta se borró. `transaccion` es el punto único de escritura de F1-2: cerrojo del sistema operativo en `.git/taskcode-estado.lock`, `fetch` solo de la rama, operación sobre el estado fresco, commit construido sin tocar ningún worktree y `push <sha>` sin force; ante non-fast-forward re-ejecuta (5 intentos, base 2 s, jitter) y si se agotan da `agotada` sin escribir nada. Sin red: las operaciones que reservan algo dan `sinred pendiente=no`; las de la tarea propia quedan en `refs/taskcode/pendientes/` y se re-ejecutan con red, y las que ya no proceden pasan a `refs/taskcode/rechazadas/`. Repo sin remoto: el commit local es la transacción. `leer` da una vista del estado.
- Capa Git: operación `commit-estado` (`git/estado.rs`), puerto ampliado (índice temporal para `read-tree`/`update-index`/`write-tree`; `update-ref` solo bajo `refs/taskcode/`), clasificación de fallos del remoto (red, credenciales, remoto inexistente, rama inexistente, fallo local) y código **8 `denegada`** fijado en `contrato.rs`.
- Cambio de comportamiento en F2-2: un rechazo del remoto (`[remote rejected]` que no es contención) es `denegada` sin reintento en `push`, `actualizar-rama` y `tag-anotado`.
- Adendas en las ADR F1-1, F1-2 y F1-4.

**Tests**: 45 en `tc2`, entre ellos dos pesados de concurrencia con procesos reales (dos clones, y dos procesos del mismo clon sin remoto, que solo separa el cerrojo), y los casos de montaje, remoto, sin red y bordes. `cargo xtask verificar --perfil completa` en verde, con build reproducible en Windows.

**Hallazgo empírico**: con dos pushes simultáneos al mismo remoto, el perdedor recibe `[remote rejected] (cannot lock ref …)`; es contención, no denegación, y se trata como non-fast-forward (anotado en F1-4).

**Revisión por pares** (4 rondas; la 3 y la 4, extra, las decidió Carlos al llegar al tope):

- Ronda 1: IMPORTANTE-1 (rama montada sin remoto y publicada después: raíz nueva y local divergente), H1 (`montar` sin cerrojo), H2 (ramas `con-cambios`/`diverge` sin test), H3 (rutas no portables y choques por mayúsculas), H4 (pendiente con error que bloqueaba) y MENOR-2/MENOR-4 (tests de caminos de error; credenciales demasiado amplias): **corregidos**. **Aceptados**: MENOR-1 (`tareas=diverge|con-cambios` sale con `ok`: lo escrito está publicado y avisar de forma visible es de `status`, F3-5b) y MENOR-3 (contenido limitado a UTF-8; el formato de tarea de F1-9 lo es).
- Ronda 2: IMPORTANTE-2 (`access denied` del remoto y un permiso local salían como `sinred`) y R2-01 (una pendiente que chocaba por mayúsculas bloqueaba el clon), ambos introducidos por correcciones: **corregidos**.
- Ronda 3: IMPORTANTE-3 (un `permission denied` de la red, curl o ssh con EACCES, clasificado como fallo local; confirmado con un test que fallaba) y R3-01 (choque por mayúsculas dentro de un mismo lote): **corregidos**.
- Ronda 4: aprobada. MENOR-1/R4-01 (marca `connection` demasiado amplia): **corregidos** con `verificar-menores`.
- **Aceptados como límite** (en la adenda de F1-2): un `[remote rejected]` con «cannot lock ref» o «failed to update ref» permanente acabaría en `agotada` y no en `denegada`. Notas de los revisores sin abrir: `COM¹`-`COM³` y `LPT¹`-`LPT³` no se tratan como reservados.

**Sin verificar**: pushes simultáneos contra GitHub o GitLab y entre máquinas distintas (F9-4); valores por defecto de intentos y espera con el CLI contra la plataforma (F9-4); Git traducido y Git anterior a 2.56; `File::lock` en unidades de red.

## Duracion

- Estimada: — (no se entiende: 240 min (de la rama abierta al cierre con merge; señales:)
- Real (de la rama abierta al cierre con merge, sin pausas): 90 min
- Cierre (finish): 88 s, de 2026-11-15T14:53:02Z a 2026-11-15T14:54:30Z
