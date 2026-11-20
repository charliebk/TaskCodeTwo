# Peticion de revision — TASK-049 (ronda 2)

- Tarea: TASK-049 — F6-3 Revisor en CI con las skills de dominio de la 0.16
- Rama revisada: feature/task-049-f6-3-revisor-en-ci-con-las-skills-de-dom
- Rama base: develop
- Commit revisado (HEAD): fc65d9e024c04921628c2087ca6622c7efaaade0
- Fecha: 2026-11-20
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-049 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 17ad62c057ca6f147c5df63ee7124444224f7693 (el commit revisado en la ronda anterior)
- Alcance: completa (motivo: un informe de la ronda anterior no tiene tabla de hallazgos legible)

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
informe de esta ronda (informe-revision-2-rust-wasm-reviewer.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-049/tarea.md
- tareas/03-en-revision/TASK-049/planificacion/
- crates/tc2/src/i18n/catalogo/seleccion.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/seleccion/orden.rs
- crates/tc2/tests/revisoras.rs
- xtask/src/workflows.rs

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.


Tabla de hallazgos ausente o ilegible en: informe-revision-1-code-quality-reviewer.md, informe-revision-1-rust-wasm-reviewer.md. Lee esos informes enteros.

## Commits a revisar (git log 17ad62c057ca6f147c5df63ee7124444224f7693..HEAD)

````
fc65d9e docs(TASK-049): limites del E2E real (informe con cache, errores de la API)
8e6eff3 fix(TASK-049): secrets inherit solo en el llamador de revision-ia
c294bc9 fix(TASK-049): secretos declarados en workflow_call y motivo del error de la API
f22cc95 chore(TASK-049): informes de la ronda 1 con los hallazgos corregidos
e399b23 fix(TASK-049): hallazgos de la ronda 1 de revision
609cca1 chore(TASK-049): peticion de revision ronda 1
````

## Diff de tu dominio (5 fichero(s) de 8; git diff 17ad62c057ca6f147c5df63ee7124444224f7693..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/i18n/catalogo/seleccion.rs b/crates/tc2/src/i18n/catalogo/seleccion.rs
index cc2f4b7..de8a751 100644
--- a/crates/tc2/src/i18n/catalogo/seleccion.rs
+++ b/crates/tc2/src/i18n/catalogo/seleccion.rs
@@ -2,8 +2,8 @@
 
 crate::i18n::textos! {
     USO_REVISORAS {
-        es: "uso: revisoras [--json] --rutas <fichero|-> [--tarea TASK-NNN] [--carril <carril>] [--complejidad <nivel>] [--plugin <nombre>=<carpeta>]... [--informe <fichero> --seccion <salida> [--artefacto <nombre>]] [--detalle <salida.json>]",
-        en: "usage: revisoras [--json] --rutas <file|-> [--tarea TASK-NNN] [--carril <lane>] [--complejidad <level>] [--plugin <name>=<folder>]... [--informe <file> --seccion <output> [--artefacto <name>]] [--detalle <output.json>]",
+        es: "uso: revisoras [--json] --rutas <fichero|-> [--nul] [--tarea TASK-NNN] [--carril <carril>] [--complejidad <nivel>] [--plugin <nombre>=<carpeta>]... [--informe <fichero> --seccion <salida> [--artefacto <nombre>]] [--detalle <salida.json>]",
+        en: "usage: revisoras [--json] --rutas <file|-> [--nul] [--tarea TASK-NNN] [--carril <lane>] [--complejidad <level>] [--plugin <name>=<folder>]... [--informe <file> --seccion <output> [--artefacto <name>]] [--detalle <output.json>]",
     }
     NO_SE_LEE {
         es: "no se puede leer ni escribir {ruta}: {error}",
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 42b84b1..49da780 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -553,13 +553,14 @@ fn orden_revisoras(args: &[String]) -> Informe {
     };
     let g = git::Git::nuevo();
     let claude = dir_claude();
+    // Un byte de mas que el tope: la orden falla si llega, como con fichero.
     let stdin = || {
         use std::io::Read;
-        let mut s = String::new();
+        let mut b = Vec::new();
         std::io::stdin()
-            .take(seleccion::orden::TOPE_RUTAS)
-            .read_to_string(&mut s)
-            .map(|_| s)
+            .take(seleccion::orden::TOPE_RUTAS + 1)
+            .read_to_end(&mut b)
+            .map(|_| b)
     };
     seleccion::orden::orden(
         args,
diff --git a/crates/tc2/src/seleccion/orden.rs b/crates/tc2/src/seleccion/orden.rs
index 974f8a0..a07c582 100644
--- a/crates/tc2/src/seleccion/orden.rs
+++ b/crates/tc2/src/seleccion/orden.rs
@@ -1,11 +1,14 @@
-//! `tc2 revisoras [--json] --rutas <fichero|-> [--tarea TASK-NNN]
+//! `tc2 revisoras [--json] --rutas <fichero|-> [--nul] [--tarea TASK-NNN]
 //! [--carril <c>] [--complejidad <x>] [--plugin <nombre>=<dir>]...
 //! [--informe <fichero> --seccion <salida> [--artefacto <n>]]
 //! [--detalle <salida.json>]` (F6-3).
 //!
-//! Las rutas son dato (una por linea; los dos nombres de cada renombrado):
-//! en CI salen de `git diff --name-only --no-renames base...head` sobre el
-//! checkout de la base; en local, de [`crate::git::ficheros_de_rama`]. La
+//! Las rutas son dato (una por linea, o separadas por NUL con `--nul`; los
+//! dos nombres de cada renombrado), con un tope de [`TOPE_RUTAS`] bytes
+//! tanto por fichero como por stdin: en CI salen de
+//! `git -c core.quotepath=false diff --name-only -z --no-renames base...head`
+//! sobre el checkout de la base (con `-z` las rutas no ASCII llegan literales,
+//! sin comillas ni escapes); en local, de [`crate::git::ficheros_de_rama`]. La
 //! configuracion y las skills del proyecto se leen de la base
 //! (`origin/develop`), nunca del worktree del PR (F1-15 decision 2).
 //!
@@ -68,8 +71,10 @@ pub struct Entorno<'a> {
     pub git: &'a Git,
     /// Carpeta de Claude Code, si se sabe.
     pub claude: Option<&'a Path>,
-    /// Lee la entrada estandar (para `--rutas -`).
-    pub stdin: &'a dyn Fn() -> std::io::Result<String>,
+    /// Lee la entrada estandar (para `--rutas -`): como mucho
+    /// `TOPE_RUTAS + 1` bytes, para que el exceso se vea y falle igual que
+    /// con fichero.
+    pub stdin: &'a dyn Fn() -> std::io::Result<Vec<u8>>,
 }
 
 /// Inventario y catalogo con los que se selecciona, y los avisos de
@@ -234,6 +239,7 @@ pub fn informe_revisor(p: &Path, artefacto: &str) -> (&'static str, Option<Strin
 #[derive(Default)]
 struct Args {
     rutas: Option<String>,
+    nul: bool,
     tarea: Option<String>,
     carril: Option<Carril>,
     complejidad: Option<String>,
@@ -244,6 +250,22 @@ struct Args {
     detalle: Option<PathBuf>,
 }
 
+/// Las rutas del texto: con `nul`, separadas por NUL y literales (la salida
+/// de `git diff --name-only -z`, sin las comillas ni los escapes que git
+/// pone a lo no ASCII); sin `nul`, una por linea (sin el `\r` final).
+fn separar_rutas(texto: &str, nul: bool) -> Vec<String> {
+    let trozos: Vec<&str> = if nul {
+        texto.split('\0').collect()
+    } else {
+        texto.lines().map(|l| l.trim_end_matches('\r')).collect()
+    };
+    trozos
+        .into_iter()
+        .filter(|l| !l.trim().is_empty())
+        .map(str::to_owned)
+        .collect()
+}
+
 fn uso(motivo: &str) -> Informe {
     Informe::nuevo(NOMBRE, Resultado::Uso)
         .con("motivo", motivo)
@@ -257,6 +279,7 @@ fn args(a: &[String]) -> Result<Args, Informe> {
         let mut valor = || it.next().cloned().ok_or_else(|| uso("falta-valor"));
         match x.as_str() {
             "--rutas" if r.rutas.is_none() => r.rutas = Some(valor()?),
+            "--nul" if !r.nul => r.nul = true,
             "--tarea" if r.tarea.is_none() => {
                 let v = valor()?;
                 if !crate::git::es_id(&v) {
@@ -319,18 +342,19 @@ fn paso(a: &[String], e: &Entorno<'_>) -> Result<Informe, Informe> {
         )
     };
     let texto = match a.rutas.as_deref() {
-        Some("-") => (e.stdin)().map_err(|x| fallo_lectura("-", x.to_string()))?,
+        Some("-") => {
+            let b = (e.stdin)().map_err(|x| fallo_lectura("-", x.to_string()))?;
+            if b.len() as u64 > TOPE_RUTAS {
+                return Err(fallo_lectura("-", format!("mas de {TOPE_RUTAS} bytes")));
+            }
+            String::from_utf8(b).map_err(|x| fallo_lectura("-", x.to_string()))?
+        }
         Some(f) => leer_con_tope(Path::new(f), TOPE_RUTAS)
             .map_err(|x| fallo_lectura(f, x.to_string()))?
             .ok_or_else(|| fallo_lectura(f, format!("mas de {TOPE_RUTAS} bytes")))?,
         None => return Err(uso("sin-rutas")),
     };
-    let rutas: Vec<String> = texto
-        .lines()
-        .map(|l| l.trim_end_matches('\r'))
-        .filter(|l| !l.trim().is_empty())
-        .map(str::to_owned)
-        .collect();
+    let rutas = separar_rutas(&texto, a.nul);
     let f = fuentes(e.git, e.dir, e.claude, &a.plugins);
     let mut avisos = f.avisos.clone();
     let mut carril = Carril::Funcionalidad;
diff --git a/xtask/src/workflows.rs b/xtask/src/workflows.rs
index 0ee5892..4cd14c8 100644
--- a/xtask/src/workflows.rs
+++ b/xtask/src/workflows.rs
@@ -14,7 +14,8 @@
 //!   `persist-credentials: false` y nunca en clase privilegio; y
 //!   `actions/upload-artifact` (informe de F5-4), fijada por SHA y solo en
 //!   clase ejecuta;
-//! - ni `secrets: inherit`, ni secretos en clase ejecuta, ni
+//! - ni `secrets: inherit` (salvo el llamador de `revision-ia.yml`, perfil
+//!   [`LLAMADOR_REVISION`]), ni secretos en clase ejecuta, ni
 //!   `github.event.*`, `github.head_ref` o `toJSON(github...)` en `run:`
 //!   (los datos del evento se pasan por `env:`).
 //!
@@ -430,6 +431,11 @@ struct Perfil {
     /// checkout`, `switch`, `worktree`, `restore` o `reset` en `run:`), y
     /// ninguna `inputs.` en `run:` (la rama del PR llega por una entrada).
     sin_head: bool,
+    /// Unico workflow reutilizable al que un job puede llamar con `secrets:
+    /// inherit` (prefijo de `uses:` hasta la `@`, fijado por SHA). GitHub
+    /// solo da los secretos del entorno `revisor` al job de `revision-ia.yml`
+    /// si el llamador hereda (E2E de TASK-049; adenda de F1-8).
+    herencia_admitida: Option<&'static str>,
 }
 
 /// `verificacion.yml` (F5-1, F5-2).
@@ -438,6 +444,7 @@ const VERIFICACION: Perfil = Perfil {
     entornos: &[],
     sin_escritura: false,
     sin_head: false,
+    herencia_admitida: None,
 };
 
 /// `revision-ia.yml` (F6-3): solo reutilizable, secretos solo en el job del
@@ -447,6 +454,18 @@ const REVISION: Perfil = Perfil {
     entornos: &["revisor"],
     sin_escritura: true,
     sin_head: true,
+    herencia_admitida: None,
+};
+
+/// El llamador de `revision-ia.yml` en el proyecto (F6-3): solo
+/// `workflow_run`, sin escritura del GITHUB_TOKEN y con `secrets: inherit`
+/// unicamente hacia el `revision-ia.yml` de TaskCodeTwo fijado por SHA.
+const LLAMADOR_REVISION: Perfil = Perfil {
+    eventos: &["workflow_run"],
+    entornos: &[],
+    sin_escritura: true,
+    sin_head: false,
+    herencia_admitida: Some("charliebk/TaskCodeTwo/.github/workflows/revision-ia.yml"),
 };
 
 /// Ordenes de git que en `run:` cambiarian el arbol de trabajo a otro commit.
@@ -460,6 +479,24 @@ fn lint_revision(texto: &str) -> Vec<String> {
     lint_con(texto, &REVISION)
 }
 
+#[cfg(test)]
+fn lint_llamador_revision(texto: &str) -> Vec<String> {
+    lint_con(texto, &LLAMADOR_REVISION)
+}
+
+/// `secrets: inherit` solo hacia el reutilizable que admite el perfil, y
+/// fijado por SHA.
+fn hereda_admitido(job: &Nodo, perfil: &Perfil) -> bool {
+    let (Some(admitido), Some(uses)) = (
+        perfil.herencia_admitida,
+        job.get("uses").and_then(Nodo::texto),
+    ) else {
+        return false;
+    };
+    uses.split_once('@')
+        .is_some_and(|(destino, version)| destino == admitido && es_sha(version))
+}
+
 fn lint_con(texto: &str, perfil: &Perfil) -> Vec<String> {
     let raiz = match leer_yaml(texto) {
         Ok(r) => r,
@@ -554,7 +591,7 @@ fn lint_con(texto: &str, perfil: &Perfil) -> Vec<String> {
         let mut valores = Vec::new();
         job.escalares("", &mut valores);
         for (clave, valor) in &valores {
-            if *clave == "secrets" && *valor == "inherit" {
+            if *clave == "secrets" && *valor == "inherit" && !hereda_admitido(job, perfil) {
                 errores.push(format!("job {nombre}: secrets: inherit"));
             }
             let normalizado = normalizar(valor);
@@ -1455,3 +1492,41 @@ fn cada_mutacion_del_revisor_da_su_error() {
         "{errores:?}"
     );
 }
+
+/// El llamador de la guia de F6-3 pasa el lint de su perfil; `secrets:
+/// inherit` hacia otro reutilizable, sin SHA, o con el perfil de
+/// verificacion, no (adenda F6-3 de F1-8).
+#[test]
+fn llamador_de_revision_ia_hereda_solo_hacia_revision_ia() {
+    let guia = include_str!("../../docs/ci/revision-ia.md").replace("\r\n", "\n");
+    let inicio = guia
+        .find("```yaml\nname: revision-ia\n")
+        .expect("llamador en la guia")
+        + 8;
+    let fin = inicio + guia[inicio..].find("```").expect("fin del bloque");
+    let llamador = guia[inicio..fin].replace("@<sha>", "@0000000000000000000000000000000000000000");
+    assert_eq!(lint_llamador_revision(&llamador), Vec::<String>::new());
+    let malos = [
+        llamador.replace(
+            "charliebk/TaskCodeTwo/.github/workflows/revision-ia.yml",
+            "o/r/.github/workflows/revision-ia.yml",
+        ),
+        llamador.replace(
+            "revision-ia.yml@0000000000000000000000000000000000000000",
+            "revision-ia.yml@main",
+        ),
+    ];
+    for malo in &malos {
+        assert_ne!(malo, &llamador);
+        let errores = lint_llamador_revision(malo);
+        assert!(
+            errores.iter().any(|e| e.contains("secrets: inherit")),
+            "{errores:?}"
+        );
+    }
+    assert!(
+        lint_github(&llamador)
+            .iter()
+            .any(|e| e.contains("secrets: inherit"))
+    );
+}
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 17ad62c057ca6f147c5df63ee7124444224f7693..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-049/planificacion/brainstorm/peticion-plan-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-049/planificacion/plan-final.md                 |    0
 tareas/03-en-revision/TASK-049/revision/informe-revision-1-code-quality-reviewer.md         |   38 ++
 tareas/03-en-revision/TASK-049/revision/informe-revision-1-rust-wasm-reviewer.md            |   33 ++
 tareas/03-en-revision/TASK-049/revision/peticion-revision-1-code-quality-reviewer.md        |  772 ++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-049/revision/peticion-revision-1-rust-wasm-reviewer.md           | 2437 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-049/tarea.md                                    |    9 +-
 7 files changed, 3286 insertions(+), 3 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff 17ad62c057ca6f147c5df63ee7124444224f7693..HEAD -- ":(literal)crates/tc2/tests/revisoras.rs"
````

````
crates/tc2/tests/revisoras.rs | 58 ++++++++++++++++++++++++++++++++++++++++++++++++++++------
 1 file changed, 52 insertions(+), 6 deletions(-)
````
