# Peticion de revision — TASK-016 (ronda 1)

- Tarea: TASK-016 — F2-1 Esqueleto: Rust, build reproducible
- Rama revisada: feature/task-016-f2-1-esqueleto-typescript-build-reproduc
- Rama base: develop
- Commit revisado (HEAD): 9d72eae2729bfc231921d0eb9a25373365dc10b3
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-016 (criterios de aceptacion y plan)
- Alcance: completa (motivo: primera ronda de revision)

## Instrucciones para el agente revisor

Eres un revisor INDEPENDIENTE del agente que implemento. Tu trabajo es
reproducir empiricamente, no leer el diff y opinar: clona el repo a un
directorio temporal, corre la suite tu mismo (salvo que esta
peticion ya traiga la verificacion de la ronda en verde para el commit revisado;
entonces vale ese resultado) y construye el caso que
rompe el codigo antes de reportarlo. Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. Vuelca tu salida en el
informe de esta ronda (informe-revision-1-code-quality-reviewer.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-016/tarea.md
- tareas/03-en-revision/TASK-016/planificacion/
- .gitattributes
- .github/workflows/ci.yml
- .gitignore
- .taskcode/config.yml
- docs/plan/PLAN-TASKCODE-2.md

## Commits a revisar (git log develop..HEAD)

````
9d72eae chore(TASK-016): autorrevision
e904ef8 feat(TASK-016): esqueleto Rust de tc2 con xtask verificar y CI en tres sistemas
b20cdd7 docs(TASK-016): F2-1 corregida a Rust y reproducibilidad solo en Windows
c7b299d chore(TASK-016): tarea en curso
````

## Diff de tu dominio (5 fichero(s) de 16; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/.gitattributes b/.gitattributes
index 540d1e6..85de2bc 100644
--- a/.gitattributes
+++ b/.gitattributes
@@ -1,2 +1,7 @@
 # Los scripts de shell se ejecutan con bash tambien en Windows: siempre LF.
 *.sh text eol=lf
+# Fuentes y manifiestos de Rust: siempre LF, para que fmt y los bytes del
+# binario no dependan de core.autocrlf (F2-1).
+*.rs text eol=lf
+*.toml text eol=lf
+Cargo.lock text eol=lf
diff --git a/.github/workflows/ci.yml b/.github/workflows/ci.yml
index f3650e1..2a0678e 100644
--- a/.github/workflows/ci.yml
+++ b/.github/workflows/ci.yml
@@ -1,5 +1,7 @@
-# CI minima de TaskCodeTwo (TASK-001, F1-0): valida tareas/ y los enlaces de
-# la documentacion. La CI de codigo llega con F2-1 y la de verdad con F5-1.
+# CI de TaskCodeTwo. comprobar-repo (TASK-001, F1-0) valida tareas/ y los
+# enlaces de la documentacion; codigo (F2-1) verifica el workspace de Rust en
+# los tres sistemas con el mismo comando que en local. La CI de verdad (check
+# `verificacion`, cache por arbol) llega con F5-1.
 name: ci
 
 on:
@@ -27,3 +29,35 @@ jobs:
           persist-credentials: false
       - name: Estructura de tareas/ y enlaces de la documentacion
         run: bash .github/scripts/comprobar-repo.sh
+
+  codigo:
+    name: codigo (${{ matrix.os }})
+    runs-on: ${{ matrix.os }}
+    timeout-minutes: 30
+    strategy:
+      fail-fast: false
+      matrix:
+        include:
+          - os: ubuntu-latest
+            perfil: rapida
+          - os: macos-latest
+            perfil: rapida
+          # La reproducibilidad (mismo sha256 en dos builds en limpio) solo se
+          # mide en Windows: decision de la persona en TASK-016.
+          - os: windows-latest
+            perfil: completa
+    steps:
+      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
+        with:
+          persist-credentials: false
+      - name: Toolchain de rust-toolchain.toml
+        run: |
+          rustup toolchain install
+          rustc --version
+          cargo --version
+      - name: cargo xtask verificar --perfil ${{ matrix.perfil }}
+        run: cargo xtask verificar --perfil ${{ matrix.perfil }}
+      - name: Sin artefactos en el arbol tras compilar
+        shell: bash
+        run: |
+          test -z "$(git status --porcelain)" || { git status --porcelain; exit 1; }
diff --git a/.gitignore b/.gitignore
index 20cbe44..8377c64 100644
--- a/.gitignore
+++ b/.gitignore
@@ -14,3 +14,6 @@ dist/
 .vscode/
 .DS_Store
 Thumbs.db
+
+# Builds de Rust (F2-1): nunca binarios en el repo
+/target/
diff --git a/.taskcode/config.yml b/.taskcode/config.yml
index 3d57b7d..5638a30 100644
--- a/.taskcode/config.yml
+++ b/.taskcode/config.yml
@@ -2,6 +2,9 @@
 # automatico: las preguntas se hacen en plan; el resto se encadena hasta finish.
 modo_flujo: automatico
 
-# Mientras no haya codigo (la Fase 1 son decisiones) no hay comando de
-# verificacion. Se declara con la primera tarea de codigo (F2-1), junto con
-# rutas_documentacion.
+# Verificacion (F2-1): el mismo comando que corre la CI en Linux y macOS.
+# La completa (con la comprobacion de build reproducible) solo la corre la CI
+# en Windows. Las tareas que solo tocan documentacion no compilan (tareas/
+# cuenta siempre como documentacion).
+comando_verificacion: "cargo xtask verificar --perfil rapida"
+rutas_documentacion: [docs/**, README.md, CHANGELOG.md, LICENSE]
diff --git a/docs/plan/PLAN-TASKCODE-2.md b/docs/plan/PLAN-TASKCODE-2.md
index 347a3b0..3a85d65 100644
--- a/docs/plan/PLAN-TASKCODE-2.md
+++ b/docs/plan/PLAN-TASKCODE-2.md
@@ -324,7 +324,7 @@ dependencias. Las fases 1 a 9 son tareas de TaskCodeTwo; la 0, del repo TaskCode
 
 | ID | Tarea | Criterio de aceptación | Dep. |
 |---|---|---|---|
-| F2-1 | Esqueleto: TypeScript, build reproducible (sin `dist` versionado), lint, CI en Linux/macOS/Windows | CI verde en los tres desde el primer commit | F1-* |
+| F2-1 | Esqueleto: Rust (`Cargo.lock` versionado, sin binarios en el repo, `/Brepro` en Windows), `cargo fmt` y `clippy`, CI en Linux/macOS/Windows (F1-14, divergencia 1) | CI verde en los tres desde el primer commit y dos builds en limpio con el mismo sha256 en Windows (Linux y macOS sin medir, decision de la persona) | F1-* |
 | F2-2 | Capa de operaciones Git deterministas: rama, worktree, commit de lo escrito, push con reintento, tag (P5) | Tests contra repos reales; salida ≤ 5 líneas en el caso normal | F1-4, F2-1 |
 | F2-3 | Rama de estado: crear, montar como worktree en `tareas/`, leer y escribir de forma atómica (P1, P2) | Dos procesos escriben a la vez sin perder nada; test | F1-1, F1-2, F2-2 |
 | F2-4 | Reserva de IDs por push atómico | Dos clones crean tarea a la vez → IDs distintos; test E2E | F2-3 |
````

## Excluido del diff (12 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
Cargo.lock                                                                                                   | 11 +++++++++++
 tareas/{01-en-diseno => 02-en-curso}/TASK-016/planificacion/brainstorm/peticion-brainstorm-arquitectura-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-016/planificacion/brainstorm/peticion-brainstorm-dominio-1.md      |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-016/planificacion/brainstorm/peticion-brainstorm-riesgos-1.md      |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-016/planificacion/brainstorm/peticion-brainstorm-testing-1.md      |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-016/planificacion/brainstorm/peticion-unificador-1.md              |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-016/planificacion/brainstorm/salida-brainstorm-arquitectura-1.md   |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-016/planificacion/brainstorm/salida-brainstorm-dominio-1.md        |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-016/planificacion/brainstorm/salida-brainstorm-riesgos-1.md        |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-016/planificacion/brainstorm/salida-brainstorm-testing-1.md        |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-016/planificacion/plan-final.md                                    |  5 +++++
 tareas/{01-en-diseno => 02-en-curso}/TASK-016/tarea.md                                                       | 10 ++++++++--
 12 files changed, 24 insertions(+), 2 deletions(-)
````
