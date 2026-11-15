# ADR F1-1 — Carpeta fija por tarea y rama de estado `taskcode/tareas`

- Estado: **aceptada** (2026-11-14)
- Tarea: TASK-002 · Plan: [F1-1 en PLAN-TASKCODE-2](../plan/PLAN-TASKCODE-2.md) (§4 P1 y tabla de la Fase 1)
- Requisitos: RR-10, RR-11, RR-12 (la parte de estado), RN-05 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md)
- Decide: Carlos, con la consigna «la opción recomendada» (resuelve el `[DECIDE CARLOS]` de P1)

## Contexto

En la 0.16 cada tarea vive en `tareas/<NN-estado>/TASK-NNN/` **dentro de las
ramas de código**: cambiar de estado mueve la carpeta, y el `ROADMAP.md` y los
`tarea.md` viajan con el código. El uso real ha mostrado tres fallos:

- un revert que viajó con el cierre de TASK-167 borró TASK-172 en ToolsaWeb
  (REQUISITOS §2.9, RN-05);
- conflictos de `ROADMAP.md` y `tarea.md` al integrar ramas (H8);
- cada transición cambia el árbol que se verifica: invalida la caché por SHA
  y dispara la CI (RR-11).

RR-10 pide una carpeta fija por tarea con el estado como campo; RR-11, un
roadmap que no interfiera con la verificación; RR-12, que el estado más
reciente esté siempre a la vista de todo el equipo.

## Alternativas

| | Descripción | Veredicto |
|---|---|---|
| **A** | Rama **huérfana** `taskcode/tareas` (sin historia común con `develop`), montada con `git worktree` en `tareas/` del worktree principal; `tareas/` en el `.gitignore` de las ramas de código. En su raíz, `TASK-NNN/` fija y `ROADMAP.md`. Estado = campo del frontmatter. | **Elegida** |
| A' | La misma rama pero nacida de `develop` (con historia común). | Descartada: con historia común GitHub deja abrir un PR de ella contra `develop` y mezclar historias (no probado aquí: conocimiento general de GitHub); la huérfana lo impide por estructura, eso sí comprobado (ver «Cómo se ve en GitHub»). |
| B | Carpeta fija por tarea, estado como campo, **en `develop`**, con commits atómicos del CLI. | Descartada: ver PoC B y la tabla de requisitos. |
| C | Estado en Issues/Projects o en refs fuera de `refs/heads`. | Descartada: RR-10 deja los issues como espejo opcional; refs ocultas no se ven en GitHub ni en el editor. |

## Pruebas de concepto

Guion único con las dos alternativas, ejecutado en repos Git temporales bajo
`%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-002\` (borrados al terminar)
con Git 2.56.0 en Windows. Pasos y salida observada, recortada:

### A — rama huérfana montada como worktree

```bash
git worktree add --orphan -b taskcode/tareas tareas     # requiere Git >= 2.42
git merge-base develop taskcode/tareas                  # -> sin salida, exit 1
```

```text
=== A.2 rama huerfana taskcode/tareas como worktree en tareas/ ===
.../poc/a        e8698c8 [develop]
.../poc/a/tareas 4064699 [taskcode/tareas]
merge-base develop taskcode/tareas: NINGUNA (historias distintas)
=== A.3 diez transiciones de estado ===
arbol develop antes : f1e7cf4b12d12288e86fe83341a1c28a6c1516ba
arbol develop despues: f1e7cf4b12d12288e86fe83341a1c28a6c1516ba
RESULTADO A.3: arbol de develop IDENTICO tras 10 transiciones
git status (develop): (vacio = limpio)
commits en develop: 1 ; que tocan tareas/: 0
commits en taskcode/tareas: 11
=== A.4 revert en una rama de codigo no puede tocar tareas/ ===
TASK-* en tareas/ tras el revert: 6 (eran 6)
=== A.5 dos clones escriben en carpetas TASK distintas ===
c1 push OK
c2 push:  ! [rejected]        taskcode/tareas -> taskcode/tareas (fetch first)
CONFLICT (content): Merge conflict in ROADMAP.md
conflicto solo en: ROADMAP.md
ROADMAP.md regenerado (no mergeado) y rebase continuado
c2 push OK tras rebase
```

Lectura: las transiciones no tocan el árbol de `develop` (la caché por SHA
sobrevive), un revert de código no alcanza las tareas, y dos escrituras
concurrentes en carpetas distintas solo chocan en `ROADMAP.md`, que se
**regenera** en vez de mergearse. El protocolo completo de reintento es de F1-2.

### B — estado en `develop`

```text
=== B.1 estado en develop ===
arboles distintos de develop en 6 estados: 6 (6 = cada transicion invalida la cache)
=== B.2 conflicto de ROADMAP al integrar una rama de tarea ===
CONFLICTO en: tareas/ROADMAP.md
=== B.3 caso TASK-172: un revert que viaja con un cierre borra otra tarea ===
TASK-007 existe en develop: si
tras revertir el merge de TASK-004, TASK-007 existe: NO: BORRADA
```

Lectura: B reproduce los tres fallos del contexto. Además exige que la
identidad del CLI empuje directo a `develop`, saltándose su protección (P3).

## Cómo se ve en GitHub

Repo temporal **privado** `charliebk/taskcodetwo-poc-f1-1` con un workflow
`on: [push, pull_request]` en `develop`, la rama huérfana con 400 tareas
sintéticas, una rama de tarea al estilo A y otra al estilo B (en B el estado
va en `estado-b/`, porque `tareas/` está ignorada en ese repo para A).
Observado con `gh` el 2026-11-14:

| Comprobación | Resultado |
|---|---|
| Ramas | `develop`, `feature/a-task-001`, `feature/b-task-001`, `taskcode/tareas` |
| Ejecuciones de Actions (`gh run list`) | 5: push a `develop`, push y PR de cada rama de tarea. **Ninguna para el push a `taskcode/tareas`**: la rama no tiene `.github/` y GitHub lee el workflow del árbol empujado. |
| PR de `taskcode/tareas` contra `develop` | Rechazado: `The taskcode/tareas branch has no history in common with develop` |
| Compare `develop...taskcode/tareas` (API) | `404 No common ancestor between develop and taskcode/tareas` |
| Ficheros del PR de A | `app.txt` |
| Ficheros del PR de B | `app.txt`, `estado-b/ROADMAP.md`, `estado-b/TASK-001/tarea.md` |
| Lectura de una tarea | `https://github.com/<owner>/<repo>/blob/taskcode/tareas/TASK-NNN/tarea.md`, y el roadmap en `.../blob/taskcode/tareas/ROADMAP.md`; en la lista de ramas aparece como una rama más. |

**No verificado**: cómo pinta GitHub un Mermaid de 400 nodos (ROADMAP
sintético de 6811 bytes y 431 líneas). El repo es privado, el navegador de la
sesión no tenía sesión en GitHub y no se autorizó publicar un gist. Pasa como
criterio de aceptación a F3-4 (roadmap). La decisión de escala de abajo no
depende de ese dato: un grafo por hito abierto es acotado por construcción.

El repo temporal no se ha podido borrar desde el CLI (el token de `gh` no
tiene `delete_repo`): queda anotado en el `## Resultado` de TASK-002 para
borrarlo a mano.

## Decisión

1. **A**: cada tarea es la carpeta fija `TASK-NNN/` (tarea, plan, informes,
   resultado) que no se mueve nunca; el estado es un campo del frontmatter.
2. Todo vive en la rama **huérfana** `taskcode/tareas`, que el CLI crea y
   monta como worktree en `tareas/` del worktree principal. `tareas/` va en el
   `.gitignore` de las ramas de código. Versión mínima de Git: 2.42 (por
   `worktree add --orphan`; con una anterior, la alternativa prevista, no probada aqui y a verificar en F2-3, es crearla con
   `switch --orphan` en un clon temporal y montarla despues).
3. **Contenido del `ROADMAP.md`** (en la raíz de la rama de estado):
   - cabecera `formato N` y zona manual `<!-- manual:inicio/fin -->` de la 0.16;
   - en curso (con quién la lleva), en revisión, planificadas por hito con sus
     dependencias, terminadas y descartadas;
   - **un grafo Mermaid por hito no cerrado**; las terminadas, como recuento
     y lista plegada en `<details>`; el grafo completo, bajo demanda
     (`taskctl roadmap --grafo`, RF-04) y sin commitear;
   - se **regenera** tras cada rebase en la rama de estado y nunca se mergea.

## Requisitos y evidencia

| Requisito | A | B | Evidencia |
|---|---|---|---|
| RR-10 carpeta fija, estado como campo | sí | sí | ambas lo permiten; difieren en dónde viven |
| RR-11 sin invalidar la verificación | sí: árbol de `develop` idéntico tras 10 transiciones; sin CI en el push | no: 6 árboles en 6 estados | PoC A.3, B.1; tabla GitHub |
| RR-11 sin conflictos entre ramas | sí: las ramas de código no llevan tareas | no: conflicto de `ROADMAP.md` | PoC B.2; ficheros de los PR |
| RR-12 estado más reciente a la vista | sí en visibilidad: una rama, push inmediato; la frescura depende de que cada clon traiga la rama (protocolo de F1-2) | sí, pero mezclado con el código | tabla GitHub; PoC A.5 (c2 solo ve lo de c1 tras `fetch`) |
| RN-05 el estado no se pierde por Git | sí para revert, merge y rebase de ramas de código (por construcción: esos commits no contienen `tareas/`); **no** para `git clean -ffdx`, que borra el worktree de `tareas/` y lo no commiteado (reproducido en la revisión) | no: TASK-007 borrada | PoC A.4 (cierta por construcción), B.3; informe de revisión 1 |
| P3 garantías de GitHub | `develop` sigue protegida | exige saltar la protección | razonado; las reglas se deciden en F1-5 |

## Consecuencias

- **Límite de A**: una rama solo puede estar en un worktree por clon. Los
  worktrees de tarea (P4) no tienen su propio `tareas/`: leen y escriben el
  estado del worktree principal a través del CLI. Se concreta en F1-3 y F2-3.
- **La CI y el revisor** no ven el plan en el árbol del PR: tienen que leerlo
  de `taskcode/tareas` (F6, F5-4).
- **Este repo no cambia todavía** (RR-14): sigue con `tareas/` en `develop`
  hasta el relevo. Al llegar, F2-3 crea la rama de estado, F1-12 decide la
  migración desde la 0.16, F3-4 genera el roadmap con este contenido, y las
  comprobaciones a)–c) de `.github/scripts/comprobar-repo.sh` se retiran o se
  mueven a la rama de estado.
- La rama de estado necesita sus propias reglas (push del CLI permitido, sin
  force-push ni borrado): F1-5.

## Fuera de alcance

- Protocolo de push con reintento, reserva de IDs y toma de tareas: F1-2.
- Flujo de ramas y worktrees de tarea: F1-3.
- Formato de `tarea.md` y del plan: F1-9.
- Migración desde la 0.16: F1-12.
- Modos de fallo de la rama de estado (worktree desmontado o corrupto,
  `tareas/` borrada a mano o con `git clean -ffdx`, trabajo sin red, rutas largas en Windows) y si los
  estados de P1 bastan a los proyectos con `05-archivadas` (RF-71): quedan
  como preguntas abiertas para F1-2, F1-11 y F2-3. Este ADR se planificó con
  un solo rol (arquitectura).

## Adenda (F2-3, TASK-018) — Git 2.42 deja de ser el mínimo

La decisión 2 fijaba Git 2.42 por `worktree add --orphan`. F2-3 crea la rama
huérfana con un commit raíz hecho con `commit-tree` (sin padre) y publicado
por la misma transacción que cualquier escritura (`estado::montar`); después
la monta con un `worktree add` normal. No necesita `--orphan`, así que la
rama de estado ya no exige 2.42. Lo que sí queda sin verificar es Git
anterior a 2.56 en general (lo anota F1-4). Además:

- Si dos clones la crean a la vez, el segundo recibe non-fast-forward,
  re-ejecuta, la ve creada y solo la monta.
- La carpeta `tareas/` borrada a mano o con `git clean -ffdx` se vuelve a
  montar con `montar` (poda el worktree y lo añade de nuevo); lo que no
  estaba publicado en ella se pierde, como se dijo en «Requisitos y
  evidencia».
- `tareas/` ocupada por una carpeta normal (un repo de la 0.16) da `rechazada
  motivo=tareas-ocupada`: la migración es de F9-2.
- `tareas/` se añade a `info/exclude` del clon para que el worktree
  principal no la vea como sin seguimiento; el `.gitignore` de las ramas de
  código lo pone F3-1.
