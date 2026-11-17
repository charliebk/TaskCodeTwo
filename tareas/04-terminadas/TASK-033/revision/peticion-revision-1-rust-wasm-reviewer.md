# Peticion de revision — TASK-033 (ronda 1)

- Tarea: TASK-033 — F3-6 start: toma atomica de la tarea, rama y worktree desde develop
- Rama revisada: feature/task-033-f3-6-start-toma-atomica-de-la-tarea-rama
- Rama base: develop
- Commit revisado (HEAD): b769913f99fc6cdbc738184abbd86a1ecb8b7606
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-033 (criterios de aceptacion y plan)
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
informe de esta ronda (informe-revision-1-rust-wasm-reviewer.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-033/tarea.md
- tareas/03-en-revision/TASK-033/planificacion/
- crates/tc2/src/alta.rs
- crates/tc2/src/git/mod.rs
- crates/tc2/src/git/rama.rs
- crates/tc2/src/grafo.rs
- crates/tc2/src/i18n/catalogo/mod.rs
- crates/tc2/src/i18n/catalogo/toma.rs
- crates/tc2/src/identidad.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/tablero.rs
- crates/tc2/src/tarea/cuerpo.rs
- crates/tc2/src/tarea/mod.rs
- crates/tc2/src/toma/mod.rs
- crates/tc2/src/toma/operacion.rs
- crates/tc2/tests/toma.rs

## Commits a revisar (git log develop..HEAD)

````
b769913 docs(TASK-033): autorrevision con rust-wasm-reviewer y code-quality-reviewer
d4557dc test(TASK-033): unitarios de Tomar sin panic y con comparacion byte a byte
a672498 docs(TASK-033): adenda de F1-4 con la linea y los motivos de start
8808e24 test(TASK-033): E2E de tc2 start para C1..C9 y la base del hotfix
19b0445 feat(TASK-033): tc2 start con toma atomica, rama y worktree
df756d4 feat(TASK-033): operacion Tomar registrada en el catalogo
ca571cf feat(TASK-033): Cuerpo::anadir_transicion y git::partes_de_rama
269ae42 refactor(TASK-033): extrae grafo::dependencia_cumplida e identidad::resolver
30c6d68 chore(TASK-033): tarea en curso
````

## Diff de tu dominio (14 fichero(s) de 15; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/alta.rs b/crates/tc2/src/alta.rs
index 1edfab0..0f8afc7 100644
--- a/crates/tc2/src/alta.rs
+++ b/crates/tc2/src/alta.rs
@@ -605,7 +605,7 @@ pub(crate) fn hoy() -> String {
 
 /// Fecha civil de un numero de dias desde 1970-01-01 (algoritmo de Howard
 /// Hinnant, `civil_from_days`).
-fn fecha_de_dias(dias: i64) -> String {
+pub(crate) fn fecha_de_dias(dias: i64) -> String {
     let z = dias + 719_468;
     let era = z.div_euclid(146_097);
     let doe = z.rem_euclid(146_097);
diff --git a/crates/tc2/src/git/mod.rs b/crates/tc2/src/git/mod.rs
index 9a3d4de..061683c 100644
--- a/crates/tc2/src/git/mod.rs
+++ b/crates/tc2/src/git/mod.rs
@@ -23,7 +23,8 @@ pub use estado::Cambio;
 pub use guardar::guardar;
 pub use puerto::{ErrorGit, Git, PREFIJO_REFS_PROPIAS, SalidaCruda, SalidaGit, Subcomando};
 pub use rama::{
-    comprobar_id, crear_rama_y_worktree, ids_con_rama, localizar_principal, reanudar, validar_base,
+    comprobar_id, crear_rama_y_worktree, ids_con_rama, localizar_principal, partes_de_rama,
+    reanudar, validar_base,
 };
 pub use remoto::{
     ESPERAS_PUSH, FalloRemoto, actualizar_principal, actualizar_rama, clasificar_fallo_remoto,
diff --git a/crates/tc2/src/git/rama.rs b/crates/tc2/src/git/rama.rs
index e2676f1..7d52940 100644
--- a/crates/tc2/src/git/rama.rs
+++ b/crates/tc2/src/git/rama.rs
@@ -198,6 +198,17 @@ pub(crate) fn rama_valida(op: &str, id: &str, tipo: &str, slug: &str) -> Paso<St
     }
 }
 
+/// Inversa de [`nombre_de_rama`]: el `tipo` y el `slug` de `rama` si es la
+/// rama de la tarea `id` (`<tipo>/<task-nnn>-<slug>`) y pasa la validacion
+/// de las operaciones de rama. `start` (F3-6) crea asi la rama que dejo el
+/// alta en `rama:`.
+pub fn partes_de_rama(id: &str, rama: &str) -> Option<(String, String)> {
+    let (tipo, resto) = rama.split_once('/')?;
+    let slug = resto.strip_prefix(&format!("{}-", id.to_lowercase()))?;
+    let valida = rama_valida("partes-de-rama", id, tipo, slug).ok()?;
+    (valida == rama).then(|| (tipo.to_owned(), slug.to_owned()))
+}
+
 fn ruta_texto(op: &str, p: &Path) -> Paso<String> {
     p.to_str().map(str::to_owned).ok_or_else(|| {
         Informe::nuevo(op, Resultado::Error).diagnostico(&txt::RUTA_WORKTREE_NO_UTF8)
@@ -415,7 +426,37 @@ pub(crate) fn renombrar(mut i: Informe, op: &str) -> Informe {
 
 #[cfg(test)]
 mod tests {
-    use super::{id_de_rama, nombres};
+    use super::{id_de_rama, nombre_de_rama, nombres, partes_de_rama};
+
+    /// F3-6: ida y vuelta con `nombre_de_rama`; lo que no es la rama de esa
+    /// tarea o no pasa la validacion, `None`.
+    #[test]
+    fn partes_de_rama_es_la_inversa_de_nombre_de_rama() {
+        for (id, tipo, slug) in [
+            ("TASK-033", "feature", "f3-6-start"),
+            ("TASK-1000", "hotfix", "x"),
+            ("TASK-007", "fix", "a-b-c-1"),
+        ] {
+            let rama = nombre_de_rama(id, tipo, slug);
+            assert_eq!(
+                partes_de_rama(id, &rama),
+                Some((tipo.to_owned(), slug.to_owned())),
+                "{rama}"
+            );
+        }
+        for (id, rama) in [
+            ("TASK-033", "feature/task-034-x"),
+            ("TASK-033", "feature/task-033"),
+            ("TASK-033", "feature/task-033-"),
+            ("TASK-033", "task-033-x"),
+            ("TASK-033", "Feature/task-033-x"),
+            ("TASK-033", "feature/task-033-con espacio"),
+            ("TASK-033", "a/b/task-033-x"),
+            ("TASK-33", "feature/task-33-x"),
+        ] {
+            assert_eq!(partes_de_rama(id, rama), None, "{rama}");
+        }
+    }
 
     /// Lo que devuelve `ls-remote` por sufijo (comprobado con git 2.56):
     /// cuenta un componente `task-<cifras>` que no sea el primero.
diff --git a/crates/tc2/src/grafo.rs b/crates/tc2/src/grafo.rs
index bfef091..38e7f18 100644
--- a/crates/tc2/src/grafo.rs
+++ b/crates/tc2/src/grafo.rs
@@ -308,6 +308,13 @@ pub fn orden_de_trabajo(nodos: &[Nodo], hitos: &Hitos) -> Orden {
     }
 }
 
+/// Si una dependencia deja empezar a quien depende de ella: no existe (no
+/// bloquea; se avisa aparte) o esta terminada. Una descartada o ilegible
+/// bloquea. La regla es una sola para `next`, `roadmap` y `start` (F3-6).
+pub fn dependencia_cumplida(dep: Option<&Nodo>) -> bool {
+    dep.is_none_or(|m| m.estado() == Some(TERMINADA))
+}
+
 /// Las listas para empezar, en el orden de trabajo: `planificada` y con
 /// todas sus dependencias existentes terminadas (una descartada bloquea).
 pub fn listas(o: &Orden, nodos: &[Nodo]) -> Vec<String> {
@@ -317,10 +324,9 @@ pub fn listas(o: &Orden, nodos: &[Nodo]) -> Vec<String> {
         .filter(|id| {
             idx.get(id.as_str()).is_some_and(|n| {
                 n.estado() == Some(PLANIFICADA)
-                    && n.dependencias.iter().all(|d| match idx.get(d.as_str()) {
-                        None => true,
-                        Some(m) => m.estado() == Some(TERMINADA),
-                    })
+                    && n.dependencias
+                        .iter()
+                        .all(|d| dependencia_cumplida(idx.get(d.as_str()).copied()))
             })
         })
         .cloned()
@@ -412,6 +418,32 @@ mod tests {
         assert_eq!(o.avisos.depende_de_descartada, ids(&["TASK-002>TASK-001"]));
     }
 
+    /// La regla de una dependencia (F3-6): inexistente o terminada deja
+    /// empezar; cualquier otro estado, descartada o ilegible, no.
+    #[test]
+    fn dependencia_cumplida_inexistente_o_terminada() {
+        assert!(dependencia_cumplida(None));
+        assert!(dependencia_cumplida(Some(&n(
+            "TASK-001",
+            0,
+            TERMINADA,
+            &[]
+        ))));
+        for e in [
+            "planificada",
+            "en-diseno",
+            "en-curso",
+            "en-revision",
+            DESCARTADA,
+        ] {
+            assert!(
+                !dependencia_cumplida(Some(&n("TASK-001", 0, e, &[]))),
+                "{e}"
+            );
+        }
+        assert!(!dependencia_cumplida(Some(&Nodo::ilegible("TASK-001"))));
+    }
+
     /// Un ID repetido no se ignora en silencio: vale el primero y se avisa.
     #[test]
     fn id_repetido_avisa() {
diff --git a/crates/tc2/src/i18n/catalogo/mod.rs b/crates/tc2/src/i18n/catalogo/mod.rs
index e3cab54..8c8c27b 100644
--- a/crates/tc2/src/i18n/catalogo/mod.rs
+++ b/crates/tc2/src/i18n/catalogo/mod.rs
@@ -15,6 +15,7 @@ pub mod plataforma;
 pub mod roadmap;
 pub mod siguiente;
 pub mod tablero;
+pub mod toma;
 
 use super::Texto;
 
@@ -35,6 +36,7 @@ pub fn todos() -> impl Iterator<Item = &'static Texto> {
         roadmap::TODOS,
         siguiente::TODOS,
         tablero::TODOS,
+        toma::TODOS,
     ]
     .into_iter()
     .flatten()
diff --git a/crates/tc2/src/i18n/catalogo/toma.rs b/crates/tc2/src/i18n/catalogo/toma.rs
new file mode 100644
index 0000000..ae64cfe
--- /dev/null
+++ b/crates/tc2/src/i18n/catalogo/toma.rs
@@ -0,0 +1,52 @@
+//! Mensajes de `tc2 start` (F3-6).
+
+crate::i18n::textos! {
+    USO_START {
+        es: "uso: start [--json] TASK-NNN",
+        en: "usage: start [--json] TASK-NNN",
+    }
+    RAMA_INVALIDA {
+        es: "la rama {rama} de la tarea {id} no tiene la forma <tipo>/task-nnn-<slug>: corrigela en su tarea.md antes de empezarla",
+        en: "branch {rama} of task {id} is not of the form <tipo>/task-nnn-<slug>: fix it in its tarea.md before starting it",
+    }
+    TIPO_RELEASE {
+        es: "la tarea {id} es de tipo release: una release no se empieza con start, sale de develop con tc2 release (ADR F1-3)",
+        en: "task {id} is of type release: a release is not started with start, it comes from develop with tc2 release (ADR F1-3)",
+    }
+    NO_EXISTE {
+        es: "la tarea {id} no existe en la rama de estado: comprueba el ID con tc2 status",
+        en: "task {id} does not exist in the state branch: check the ID with tc2 status",
+    }
+    FORMATO_1 {
+        es: "la tarea {id} esta en el formato de la 0.16: conviertela con tc2 actualizar antes de empezarla",
+        en: "task {id} is in the 0.16 format: convert it with tc2 actualizar before starting it",
+    }
+    ILEGIBLE {
+        es: "el tarea.md de {id} no se entiende ({error}): corrigelo antes de empezarla",
+        en: "the tarea.md of {id} cannot be read ({error}): fix it before starting it",
+    }
+    TOMADA {
+        es: "la tarea {id} ya la lleva {login}: elige otra con tc2 next o pide que te la reasigne",
+        en: "task {id} is already taken by {login}: pick another one with tc2 next or ask to have it reassigned",
+    }
+    ASIGNADO_HEREDADO {
+        es: "la tarea {id} tiene un asignado_a heredado de la 0.16 ({texto}): hay que reasignarla a una persona de la lista antes de empezarla",
+        en: "task {id} has an assignee inherited from 0.16 ({texto}): it has to be reassigned to a person on the list before starting it",
+    }
+    TERMINADA {
+        es: "la tarea {id} ya esta terminada: no se puede empezar",
+        en: "task {id} is already finished: it cannot be started",
+    }
+    DESCARTADA {
+        es: "la tarea {id} esta descartada: no se puede empezar",
+        en: "task {id} is discarded: it cannot be started",
+    }
+    ESTADO_NO_VALIDO {
+        es: "la tarea {id} esta {estado}: start solo empieza tareas planificadas o en diseno",
+        en: "task {id} is {estado}: start only starts planned or in-design tasks",
+    }
+    DEPENDENCIAS_PENDIENTES {
+        es: "la tarea {id} depende de tareas sin terminar ({ids}): terminalas antes o elige otra con tc2 next",
+        en: "task {id} depends on unfinished tasks ({ids}): finish them first or pick another one with tc2 next",
+    }
+}
diff --git a/crates/tc2/src/identidad.rs b/crates/tc2/src/identidad.rs
index 62e9b69..3101c52 100644
--- a/crates/tc2/src/identidad.rs
+++ b/crates/tc2/src/identidad.rs
@@ -30,7 +30,7 @@ use crate::i18n::catalogo::identidad as t;
 use crate::i18n::{Mensaje, Texto};
 use crate::plataforma::personas::RUTA as RUTA_PERSONAS;
 use crate::plataforma::{
-    Credenciales, GitHub, GitLab, Personas, Plataforma, Proveedor, Rol, Usuario, detectar,
+    Credenciales, Destino, GitHub, GitLab, Personas, Plataforma, Proveedor, Rol, Usuario, detectar,
 };
 use crate::tarea::Asignado;
 
@@ -454,54 +454,92 @@ fn orden_paso(e: Entorno<'_>) -> Paso<Informe> {
             .con("rol", "persona")
             .diagnostico(&t::NECESITA_TERMINAL));
     }
-    let comun = git::estado::dir_comun(e.git, e.dir, NOMBRE)?;
-    let Some(url) = git::estado::url_origin(e.git, e.dir, NOMBRE)? else {
+    let (g, dir, ahora) = (e.git, e.dir, e.ahora);
+    let quien = resolver(e)?;
+    let c = quien.cuenta();
+    let comun = git::estado::dir_comun(g, dir, NOMBRE)?;
+    Cache::en(&comun).guardar(c, ahora)?;
+    Ok(Informe::nuevo(NOMBRE, Resultado::Ok)
+        .con("rol", "persona")
+        .con("id", c.id.to_string())
+        .con("login", c.login.clone()))
+}
+
+/// A donde habla la plataforma de `origin`: su URL y la clave `plataforma`
+/// de la configuracion de la base (F1-13, decision 7). Sin `origin` o con
+/// una plataforma que no se reconoce, `rechazada motivo=plataforma`. La
+/// comparten `identidad`, `start` y `board`.
+pub fn destino_de(g: &Git, dir: &Path) -> Paso<Destino> {
+    let Some(url) = git::estado::url_origin(g, dir, NOMBRE)? else {
         return Err(Informe::nuevo(NOMBRE, Resultado::Rechazada)
             .con("motivo", "plataforma")
             .diagnostico(&t::SIN_ORIGIN));
     };
-    // Con terminal ya hace falta red (la API): se trae antes la rama base,
-    // para que la lista de personas y la configuracion sean las de ahora
-    // (quien acaba de entrar en la lista por un PR fusionado). Sin red,
-    // `sinred` y la cache sin tocar.
-    let rama_base = config::BASE.strip_prefix("origin/").unwrap_or(config::BASE);
-    git::remoto::traer_rama(e.git, e.dir, NOMBRE, rama_base)?;
-    let clave = clave_plataforma(e.git, e.dir)?;
-    let destino =
-        detectar(&url, clave.as_ref().map(|(p, a)| (*p, a.as_str()))).map_err(|mut i| {
-            NOMBRE.clone_into(&mut i.comando);
-            i
-        })?;
-    let personas = personas_de_base(e.git, e.dir)?;
-    let plataforma: Box<dyn Plataforma> = match destino.proveedor {
+    destino_de_url(g, dir, &url)
+}
+
+fn destino_de_url(g: &Git, dir: &Path, url: &str) -> Paso<Destino> {
+    let clave = clave_plataforma(g, dir)?;
+    detectar(url, clave.as_ref().map(|(p, a)| (*p, a.as_str()))).map_err(|mut i| {
+        NOMBRE.clone_into(&mut i.comando);
+        i
+    })
+}
+
+/// El adaptador de `destino` con `credenciales` y la lista de `personas`.
+pub fn adaptador(
+    destino: Destino,
+    credenciales: Box<dyn Credenciales>,
+    personas: Personas,
+) -> Paso<Box<dyn Plataforma>> {
+    Ok(match destino.proveedor {
         Proveedor::GitHub => Box::new(GitHub::nuevo(
             destino.api,
             &destino.proyecto,
-            e.credenciales,
-            personas.clone(),
+            credenciales,
+            personas,
         )?),
         Proveedor::GitLab => Box::new(GitLab::nuevo(
             destino.api,
             &destino.proyecto,
-            e.credenciales,
-            personas.clone(),
+            credenciales,
+            personas,
         )?),
+    })
+}
+
+/// Quien ejecuta en el repo de `e` (D6 de TASK-033): la plataforma de
+/// `origin`, la lista de personas de la base y [`quien_ejecuta`] con el
+/// terminal de `e`. Con terminal trae antes la rama base (hace falta red
+/// para la API); sin terminal no habla con la red. No escribe la cache: eso
+/// solo lo hace `tc2 identidad`.
+pub fn resolver(e: Entorno<'_>) -> Paso<Quien> {
+    let comun = git::estado::dir_comun(e.git, e.dir, NOMBRE)?;
+    let Some(url) = git::estado::url_origin(e.git, e.dir, NOMBRE)? else {
+        return Err(Informe::nuevo(NOMBRE, Resultado::Rechazada)
+            .con("motivo", "plataforma")
+            .diagnostico(&t::SIN_ORIGIN));
     };
-    let cache = Cache::en(&comun);
-    let quien = quien_ejecuta(
+    if e.tty {
+        // Con terminal ya hace falta red (la API): se trae antes la rama
+        // base, para que la lista de personas y la configuracion sean las de
+        // ahora (quien acaba de entrar en la lista por un PR fusionado). Sin
+        // red, `sinred` y la cache sin tocar.
+        let rama_base = config::BASE.strip_prefix("origin/").unwrap_or(config::BASE);
+        git::remoto::traer_rama(e.git, e.dir, NOMBRE, rama_base)?;
+    }
+    let destino = destino_de_url(e.git, e.dir, &url)?;
+    let proveedor = destino.proveedor;
+    let personas = personas_de_base(e.git, e.dir)?;
+    let plataforma = adaptador(destino, e.credenciales, personas.clone())?;
+    quien_ejecuta(
         plataforma.as_ref(),
-        destino.proveedor,
+        proveedor,
         &personas,
-        &cache,
-        true,
+        &Cache::en(&comun),
+        e.tty,
         e.ahora,
-    )?;
-    let c = quien.cuenta();
-    cache.guardar(c, e.ahora)?;
-    Ok(Informe::nuevo(NOMBRE, Resultado::Ok)
-        .con("rol", "persona")
-        .con("id", c.id.to_string())
-        .con("login", c.login.clone()))
+    )
 }
 
 #[cfg(test)]
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index fd7cfd4..9853288 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -16,7 +16,7 @@
 //! [`importar`] (alta en bloque con dependencias e hito, F3-2) y [`grafo`] y
 //! [`siguiente`] (orden de trabajo y `next`, F3-3) y [`situacion`] y
 //! [`roadmap`] (el calculo comun y `roadmap`, F3-4) y [`tablero`]
-//! (`board` y `status`, F3-5). Las capas
+//! (`board` y `status`, F3-5) y [`toma`] (`start`, F3-6). Las capas
 //! que vienen entran como modulos de esta biblioteca.
 
 pub mod actualizar_cli;
@@ -41,6 +41,7 @@ pub mod siguiente;
 pub mod situacion;
 pub mod tablero;
 pub mod tarea;
+pub mod toma;
 pub mod version;
 mod yaml;
 
@@ -146,6 +147,10 @@ where
             let (json, resto) = con_json(resto);
             init::orden(&resto, json, idioma)
         }
+        [orden, resto @ ..] if orden == toma::NOMBRE => {
+            let (json, resto) = con_json(resto);
+            orden_start(&resto).salida_en(json, idioma)
+        }
         [orden, resto @ ..] if orden == identidad::NOMBRE => {
             let (json, resto) = con_json(resto);
             orden_identidad(&resto).salida_en(json, idioma)
@@ -277,6 +282,38 @@ fn orden_identidad(args: &[String]) -> Informe {
     )
 }
 
+/// `start [--json] TASK-NNN` (F3-6) en la carpeta actual: quien la toma sale
+/// de [`identidad::resolver`] con el terminal y los tokens del proceso.
+fn orden_start(args: &[String]) -> Informe {
+    use std::io::IsTerminal;
+    let dir = match std::env::current_dir() {
+        Ok(d) => d,
+        Err(e) => {
+            return Informe::nuevo(toma::NOMBRE, Resultado::Error)
+                .diagnostico(Mensaje::nuevo(&txt::CARPETA_ACTUAL).con("error", e.to_string()));
+        }
+    };
+    let g = git::Git::nuevo();
+    let ahora = identidad::ahora();
+    let quien = || {
+        identidad::resolver(identidad::Entorno {
+            dir: &dir,
+            git: &g,
+            tty: std::io::stdin().is_terminal(),
+            credenciales: Box::new(plataforma::CredencialesEntorno),
+            ahora,
+        })
+    };
+    let e = toma::Entorno {
+        dir: &dir,
+        git: &g,
+        quien: &quien,
+        catalogo: &catalogo,
+        ahora,
+    };
+    toma::orden(args, &e)
+}
+
 /// `status` o `board` (F3-5) en la carpeta actual.
 fn orden_tablero(orden: &str, args: &[String], json: bool, idioma: Idioma) -> Salida {
     let dir = match tablero::carpeta_actual(orden) {
@@ -300,7 +337,9 @@ fn orden_tablero(orden: &str, args: &[String], json: bool, idioma: Idioma) -> Sa
 /// Catalogo de produccion: reconstruye las operaciones de la rama de estado
 /// desde su anotacion. Las operaciones nuevas se anaden aqui.
 fn catalogo(anotacion: &str) -> Option<Box<dyn estado::Operacion>> {
-    alta::desde_anotacion(anotacion).or_else(|| roadmap::desde_anotacion(anotacion))
+    alta::desde_anotacion(anotacion)
+        .or_else(|| roadmap::desde_anotacion(anotacion))
+        .or_else(|| toma::desde_anotacion(anotacion))
 }
 
 /// `new "<titulo>" [campos de RF-01]`: alta de una tarea con su ID reservado
@@ -466,6 +505,45 @@ mod tests {
         }
     }
 
+    /// F3-6: el catalogo de produccion reconstruye la toma desde su
+    /// anotacion, igual que el alta y el roadmap.
+    #[test]
+    fn el_catalogo_reconstruye_la_toma() {
+        let op = catalogo("start|TASK-001|7|ana");
+        assert_eq!(
+            op.map(|o| (o.nombre().to_owned(), o.anotacion())),
+            Some(("start".to_owned(), "start|TASK-001|7|ana".to_owned()))
+        );
+        assert!(catalogo("start|TASK-001|7").is_none());
+    }
+
+    /// TASK-033 C5: un ID mal formado (o sin ID, o de mas) es `uso` (2) sin
+    /// tocar nada, con el diagnostico en los dos idiomas.
+    #[test]
+    fn start_con_id_mal_formado_es_uso() {
+        for (args, motivo) in [
+            (vec!["start"], "sin-id"),
+            (vec!["start", "TASK-1"], "id-invalido"),
+            (vec!["start", "task-001"], "id-invalido"),
+            (vec!["start", "TASK-001a"], "id-invalido"),
+            (
+                vec!["start", "--json", "TASK-001", "x"],
+                "argumentos-de-mas",
+            ),
+        ] {
+            let es = ejecutar_en(Idioma::Es, &args);
+            let en = ejecutar_en(Idioma::En, &args);
+            assert_eq!((es.codigo, en.codigo), (2, 2), "{args:?}");
+            assert!(es.stdout.contains(motivo), "{args:?}: {}", es.stdout);
+            assert_eq!(es.stderr, "uso: start [--json] TASK-NNN\n");
+            assert_eq!(en.stderr, "usage: start [--json] TASK-NNN\n");
+        }
+        assert_eq!(
+            ejecutar(["start", "TASK-1"]).stdout,
+            "start uso motivo=id-invalido\n"
+        );
+    }
+
     #[test]
     fn sin_orden_falla() {
         assert_eq!(ejecutar(Vec::<String>::new()).codigo, 2);
diff --git a/crates/tc2/src/tablero.rs b/crates/tc2/src/tablero.rs
index fba2a8f..357b4f7 100644
--- a/crates/tc2/src/tablero.rs
+++ b/crates/tc2/src/tablero.rs
@@ -28,10 +28,7 @@ use crate::i18n::catalogo::cli as txt_cli;
 use crate::i18n::catalogo::roadmap as txt_roadmap;
 use crate::i18n::catalogo::tablero as txt;
 use crate::i18n::{Idioma, Mensaje};
-use crate::plataforma::{
-    Credenciales, CredencialesEntorno, GitHub, GitLab, Personas, Plataforma, PrAbierto, Proveedor,
-    Rol, detectar,
-};
+use crate::plataforma::{Credenciales, CredencialesEntorno, Personas, Plataforma, PrAbierto, Rol};
 use crate::situacion::{self, Ficha, Quien, Situacion};
 
 /// Nombre de `board`.
@@ -369,14 +366,8 @@ pub fn conectar_produccion<'a>(
     dir: &'a Path,
 ) -> impl Fn(&Personas) -> Result<Conectada, Consulta> + 'a {
     move |personas: &Personas| {
-        let url = crate::git::estado::url_origin(g, dir, BOARD)
-            .ok()
-            .flatten()
-            .ok_or(Consulta::NoConfigurada)?;
-        let clave =
-            crate::identidad::clave_plataforma(g, dir).map_err(|_| Consulta::NoConfigurada)?;
-        let destino = detectar(&url, clave.as_ref().map(|(p, a)| (*p, a.as_str())))
-            .map_err(|_| Consulta::NoConfigurada)?;
+        // La misma deteccion que `identidad` y `start` (D6 de TASK-033).
+        let destino = crate::identidad::destino_de(g, dir).map_err(|_| Consulta::NoConfigurada)?;
         let credenciales = CredencialesEntorno;
         let rol = if credenciales.token(Rol::Persona).is_some() {
             Rol::Persona
@@ -385,26 +376,9 @@ pub fn conectar_produccion<'a>(
         } else {
             return Err(Consulta::SinCredenciales);
         };
-        let plataforma: Box<dyn Plataforma> = match destino.proveedor {
-            Proveedor::GitHub => Box::new(
-                GitHub::nuevo(
-                    destino.api,
-                    &destino.proyecto,
-                    Box::new(credenciales),
-                    personas.clone(),
-                )
-                .map_err(|_| Consulta::NoConfigurada)?,
-            ),
-            Proveedor::GitLab => Box::new(
-                GitLab::nuevo(
-                    destino.api,
-                    &destino.proyecto,
-                    Box::new(credenciales),
-                    personas.clone(),
-                )
-                .map_err(|_| Consulta::NoConfigurada)?,
-            ),
-        };
+        let plataforma =
+            crate::identidad::adaptador(destino, Box::new(credenciales), personas.clone())
+                .map_err(|_| Consulta::NoConfigurada)?;
         Ok(Conectada { plataforma, rol })
     }
 }
diff --git a/crates/tc2/src/tarea/cuerpo.rs b/crates/tc2/src/tarea/cuerpo.rs
index 1aba934..c6dd0c2 100644
--- a/crates/tc2/src/tarea/cuerpo.rs
+++ b/crates/tc2/src/tarea/cuerpo.rs
@@ -177,6 +177,80 @@ impl Cuerpo {
     pub fn seccion(&self, titulo: &str) -> Option<&Seccion> {
         self.secciones.iter().find(|s| s.titulo == titulo)
     }
+
+    /// Anade una fila a la tabla de `## Transiciones` (F1-9: la escribe el
+    /// CLI en cada transicion), justo detras de su ultima fila; el resto del
+    /// cuerpo queda byte a byte. Sin tabla, la crea con su cabecera al final
+    /// de la seccion; sin seccion, la crea al final del cuerpo.
+    pub fn anadir_transicion(&mut self, t: &Transicion<'_>) {
+        let fila = format!(
+            "| {} | {} | {} | {} | {} |\n",
+            t.fecha, t.fase, t.modo, t.decidido_por, t.version
+        );
+        if let Some(s) = self.secciones.iter_mut().find(|s| s.titulo == TRANSICIONES) {
+            let lineas: Vec<&str> = s.texto.split_inclusive('\n').collect();
+            let texto = match lineas.iter().rposition(|l| l.trim_start().starts_with('|')) {
+                Some(i) => {
+                    let mut antes = lineas[..=i].concat();
+                    if !antes.ends_with('\n') {
+                        antes.push('\n');
+                    }
+                    format!("{antes}{fila}{}", lineas[i + 1..].concat())
+                }
+                None => {
+                    let mut t = s.texto.clone();
+                    separar(&mut t);
+                    format!("{t}{CABECERA_TRANSICIONES}{fila}")
+                }
+            };
+            s.texto = texto;
+            return;
+        }
+        match self.secciones.last_mut() {
+            Some(s) => separar(&mut s.texto),
+            None => separar(&mut self.preambulo),
+        }
+        self.secciones.push(Seccion {
+            titulo: TRANSICIONES.to_owned(),
+            texto: format!("## {TRANSICIONES}\n\n{CABECERA_TRANSICIONES}{fila}"),
+        });
+    }
+}
+
+/// Titulo de la seccion de transiciones.
+const TRANSICIONES: &str = "Transiciones";
+
+/// Cabecera de la tabla de transiciones (plantilla de F1-9, punto 6).
+const CABECERA_TRANSICIONES: &str =
+    "| fecha | fase | modo | decidido_por | version |\n|---|---|---|---|---|\n";
+
+/// Deja `texto` terminado en una linea en blanco (si no esta vacio), para
+/// que lo que se anada detras empiece un bloque nuevo.
+fn separar(texto: &mut String) {
+    if texto.is_empty() {
+        return;
+    }
+    if !texto.ends_with('\n') {
+        texto.push('\n');
+    }
+    if !texto.ends_with("\n\n") {
+        texto.push('\n');
+    }
+}
+
+/// Una fila de `## Transiciones` (F1-9, punto 6).
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub struct Transicion<'a> {
+    /// Instante, `AAAA-MM-DDTHH:MM:SSZ` en UTC.
+    pub fecha: &'a str,
+    /// Fase que se entra (`start`, `review`...).
+    pub fase: &'a str,
+    /// Modo de la tarea (`manual`, `automatico`...).
+    pub modo: &'a str,
+    /// `persona` o `agente` (F2-13).
+    pub decidido_por: &'a str,
+    /// Version del CLI que la escribe.
+    pub version: &'a str,
 }
 
 #[cfg(test)]
@@ -245,6 +319,66 @@ mod tests {
         assert!(t.contains("\n\n- [ ] real\n\n"), "{t}");
     }
 
+    const T: Transicion<'static> = Transicion {
+        fecha: "2026-11-17T10:00:00Z",
+        fase: "start",
+        modo: "manual",
+        decidido_por: "agente",
+        version: "0.1.0",
+    };
+    const FILA: &str = "| 2026-11-17T10:00:00Z | start | manual | agente | 0.1.0 |\n";
+
+    /// F3-6: la fila va detras de la ultima de la tabla y el resto del
+    /// cuerpo queda byte a byte (tambien lo que sigue a la tabla).
+    #[test]
+    fn anadir_transicion_conserva_el_resto() {
+        let mut p = Cuerpo::plantilla();
+        p.anadir_transicion(&T);
+        assert_eq!(p.escribir(), format!("{PLANTILLA}{FILA}"));
+        // Con filas previas, texto detras y otra seccion despues.
+        let t = "## Objetivo\n\nx\n\n## Transiciones\n\n| fecha | fase | modo | decidido_por | version |\n|---|---|---|---|---|\n| a | plan | manual | persona | 0.1.0 |\n\nnota final\n## Otra\r\ny";
+        let mut c = Cuerpo::leer(t);
+        c.anadir_transicion(&T);
+        let esperado = t.replace(
+            "| a | plan | manual | persona | 0.1.0 |\n",
+            &format!("| a | plan | manual | persona | 0.1.0 |\n{FILA}"),
+        );
+        assert_eq!(c.escribir(), esperado);
+        // Ultima fila sin salto final.
+        let mut c = Cuerpo::leer("## Transiciones\n|---|");
+        c.anadir_transicion(&T);
+        assert_eq!(c.escribir(), format!("## Transiciones\n|---|\n{FILA}"));
+    }
+
+    /// Sin tabla se crea con su cabecera; sin seccion, al final del cuerpo.
+    #[test]
+    fn anadir_transicion_sin_tabla_o_sin_seccion() {
+        let mut c = Cuerpo::leer("## Transiciones\ntexto");
+        c.anadir_transicion(&T);
+        assert_eq!(
+            c.escribir(),
+            format!("## Transiciones\ntexto\n\n{CABECERA_TRANSICIONES}{FILA}")
+        );
+        let mut c = Cuerpo::leer("## Objetivo\nx");
+        c.anadir_transicion(&T);
+        assert_eq!(
+            c.escribir(),
+            format!("## Objetivo\nx\n\n## Transiciones\n\n{CABECERA_TRANSICIONES}{FILA}")
+        );
+        assert_eq!(
+            c.seccion("Transiciones").map(|s| s.titulo.as_str()),
+            Some("Transiciones")
+        );
+        let mut c = Cuerpo::leer("");
+        c.anadir_transicion(&T);
+        assert_eq!(
+            c.escribir(),
+            format!("## Transiciones\n\n{CABECERA_TRANSICIONES}{FILA}")
+        );
+        // Lo escrito se vuelve a leer igual.
+        assert_eq!(Cuerpo::leer(&c.escribir()), c);
+    }
+
     #[test]
     fn cercas_como_commonmark() {
         let titulos = |t: &str| -> Vec<String> {
diff --git a/crates/tc2/src/tarea/mod.rs b/crates/tc2/src/tarea/mod.rs
index bd99610..de9ebde 100644
--- a/crates/tc2/src/tarea/mod.rs
+++ b/crates/tc2/src/tarea/mod.rs
@@ -17,7 +17,7 @@ pub(crate) mod yaml;
 
 use std::fmt;
 
-pub use cuerpo::{Cuerpo, Seccion};
+pub use cuerpo::{Cuerpo, Seccion, Transicion};
 pub use modelo::{
     Asignado, COMPLEJIDADES, ESTADOS, Heredado, LOGIN_MAX, REGLAS, Rol, RolRevision, Roles, TIPOS,
     Tarea, TareaV1, id_valido, login_valido, titulo_valido,
diff --git a/crates/tc2/src/toma/mod.rs b/crates/tc2/src/toma/mod.rs
new file mode 100644
index 0000000..d298882
--- /dev/null
+++ b/crates/tc2/src/toma/mod.rs
@@ -0,0 +1,285 @@
+//! `tc2 start [--json] TASK-NNN` (F3-6): toma atomica de la tarea, rama y
+//! worktree (ADR F1-2, decision 4; F1-3, decision 2, fila `start`).
+//!
+//! Dos mitades separadas por el push:
+//!
+//! 1. La toma ([`Tomar`]) pasa por el punto unico de escritura
+//!    ([`estado::transaccion`]): dos clones a la vez, uno gana y el otro sale
+//!    `rechazada motivo=tomada` con el login del ganador.
+//! 2. Fuera de la transaccion y solo con la toma publicada, la rama (sin
+//!    upstream, desde la punta de la base) y el worktree en
+//!    `.taskcodetwo/worktrees/TASK-NNN` ([`git::crear_rama_y_worktree`]). Si
+//!    la tarea ya es de quien la lanza y esta empezada, no se escribe estado:
+//!    se crea solo lo que falte ([`git::reanudar`]).
+//!
+//! Pasos de [`start_en`]: `fetch`; lectura del estado; quien lo lanza
+//! (identidad, D6); la toma en seco sobre lo leido (los rechazos salen sin
+//! esperar al push); la base valida (`origin/develop`, o para `hotfix` el
+//! ultimo tag de `origin/main` o su punta, D4; `release` se rechaza);
+//! `comprobar-id` (una rama `*/task-nnn-*` que no es de quien la lanza,
+//! `ocupada`); la transaccion; la rama y el worktree.
+//!
+//! Lanzada desde un worktree de tarea escribe el estado en el principal (lo
+//! localiza la transaccion) y crea el worktree junto al principal.
+
+pub mod operacion;
+
+use std::path::Path;
+
+use crate::contrato::{Informe, Resultado, Valor};
+use crate::estado::{self, Catalogo, Fallo, Operacion};
+use crate::git::{self, Git, Subcomando};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::toma as txt;
+use crate::identidad::Quien;
+
+pub use operacion::{Tomar, desde_anotacion, instante};
+
+/// Nombre de la orden y de la operacion de la rama de estado.
+pub const NOMBRE: &str = "start";
+
+/// Base de un `hotfix` sin tags en `main` (ADR F1-3, decision 1).
+const MAIN: &str = "origin/main";
+
+/// Lo que necesita `start` del entorno, para poder probarlo sin terminal ni
+/// tokens (como [`crate::tablero::Entorno`]). El orquestador de F4-4 lo usa
+/// directamente.
+pub struct Entorno<'a> {
+    /// Carpeta desde la que se lanza (el principal o un worktree de tarea).
+    pub dir: &'a Path,
+    /// Puerto Git.
+    pub git: &'a Git,
+    /// Quien lanza `start`; en produccion, [`crate::identidad::resolver`].
+    pub quien: &'a dyn Fn() -> Result<Quien, Informe>,
+    /// Para re-ejecutar las pendientes de la transaccion.
+    pub catalogo: Catalogo<'a>,
+    /// Instante de la toma, en segundos desde 1970 (UTC).
+    pub ahora: u64,
+}
+
+/// `start TASK-NNN` (sin `--json`, que quita `lib.rs`).
+pub fn orden(args: &[String], e: &Entorno<'_>) -> Informe {
+    let uso = |motivo: &str| {
+        Informe::nuevo(NOMBRE, Resultado::Uso)
+            .con("motivo", motivo)
+            .diagnostico(&txt::USO_START)
+    };
+    match args {
+        [id] if git::es_id(id) => start_en(e, id),
+        [_] => uso("id-invalido"),
+        [] => uso("sin-id"),
+        _ => uso("argumentos-de-mas"),
+    }
+}
+
+/// `start` de la tarea `id` en el repo de `e`.
+pub fn start_en(e: &Entorno<'_>, id: &str) -> Informe {
+    let i = match paso(e, id) {
+        Ok(i) | Err(i) => i,
+    };
+    de_start(i, id)
+}
+
+/// El informe como de `start`: el comando, `id=` el primero y sin la clave
+/// `operacion` de la transaccion. Un `sinred` dice siempre `pendiente=no`
+/// (`start` exige red, F1-2 D2b).
+fn de_start(mut i: Informe, id: &str) -> Informe {
+    NOMBRE.clone_into(&mut i.comando);
+    i.claves.retain(|(k, _)| k != "operacion" && k != "id");
+    i.claves
+        .insert(0, ("id".to_owned(), Valor::Texto(id.to_owned())));
+    if i.resultado == Resultado::Sinred && i.valor("pendiente").is_none() {
+        i = i.con("pendiente", "no");
+    }
+    i
+}
+
+fn rechazada(motivo: &str, m: Mensaje) -> Informe {
+    Informe::nuevo(NOMBRE, Resultado::Rechazada)
+        .con("motivo", motivo)
+        .diagnostico(m)
+}
+
+/// Un rechazo de la toma con lo que sabe la operacion: el login de quien la
+/// lleva o las dependencias sin cumplir.
+fn rechazo_de_toma(mut i: Informe, op: &Tomar) -> Informe {
+    match i.valor("motivo") {
+        Some("tomada") => {
+            if let Some(l) = op.dueno() {
+                i = i.con("asignado", l);
+            }
+        }
+        Some("dependencias-pendientes") => i = i.con_lista("dependencias", op.pendientes()),
+        Some("formato-1") => i = i.con("arreglo", "tc2 actualizar"),
+        _ => {}
+    }
+    i
+}
+
+fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
+    let g = e.git;
+    let principal = git::localizar_principal(g, e.dir)?;
+    // 1. Lo de ahora del remoto: el estado, la base y las ramas.
+    if git::estado::hay_remoto(g, &principal, NOMBRE)? {
+        let f = git::fetch(g, &principal);
+        if !f.resultado.es_exito() {
+            return Err(f);
+        }
+    }
+    let vista = estado::leer(g, e.dir)?;
+    // 2. Quien la toma.
+    let quien = (e.quien)()?;
+    // 3. La toma en seco sobre lo leido: los rechazos, sin esperar al push.
+    let op = Tomar::nueva(id, &quien, e.ahora);
+    let empezada = match op.aplicar(&vista) {
+        Ok(c) => c.is_empty(),
+        Err(Fallo::Rechazada {
+            motivo,
+            diagnostico,
+        }) => return Err(rechazo_de_toma(rechazada(&motivo, diagnostico), &op)),
+        Err(Fallo::Error(d)) => return Err(Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d)),
+    };
+    let rama = op.rama().unwrap_or_default();
+    let Some((tipo, slug)) = git::partes_de_rama(id, &rama) else {
+        return Err(rechazada(
+            "rama-invalida",
+            Mensaje::nuevo(&txt::RAMA_INVALIDA)
+                .con("id", id)
+                .con("rama", rama.as_str()),
+        )
+        .con("rama", rama.as_str()));
+    };
+    // 4. La base, valida antes de escribir nada (F1-3).
+    let base = base_de(g, &principal, id, &rama)?;
+    let v = git::validar_base(g, &principal, &base);
+    if !v.resultado.es_exito() {
+        return Err(v);
+    }
+    let destino = Destino {
+        g,
+        principal: &principal,
+        id,
+        rama: &rama,
+        tipo: &tipo,
+        slug: &slug,
+        base: &base,
+    };
+    if empezada {
+        // 5c. Ya es suya y esta empezada: crear solo lo que falte.
+        return Ok(destino.crear(None, &quien));
+    }
+    // 5. El ID no tiene rama de nadie mas.
+    let c = git::comprobar_id(g, &principal, id);
+    if !c.resultado.es_exito() {
+        return Err(c);
+    }
+    // 6. La toma, por el punto unico de escritura.
+    let personas = crate::roadmap::personas(g, e.dir);
+    let generar = crate::roadmap::generador(&personas);
+    let ctx = estado::Contexto::nuevo(g, e.dir, e.catalogo, Some(&generar));
+    let t = estado::transaccion(&ctx, &op);
+    match t.resultado {
+        Resultado::Ok => {
+            let commit = t.valor("commit").map(str::to_owned);
+            Ok(destino.crear(commit, &quien))
+        }
+        // Entre la lectura y el push paso a ser suya y empezada.
+        Resultado::SinCambios => Ok(destino.crear(None, &quien)),
+        Resultado::Rechazada => Err(rechazo_de_toma(t, &op)),
+        _ => Err(t),
+    }
+}
+
+/// Base de la rama (ADR F1-3, decision 1; D4 de TASK-033): `origin/develop`;
+/// para `hotfix/`, el ultimo tag (por version) contenido en `origin/main` o,
+/// sin tags, su punta; `release/` no se empieza con `start`.
+fn base_de(g: &Git, principal: &Path, id: &str, rama: &str) -> Result<String, Informe> {
+    if rama.starts_with("release/") {
+        return Err(rechazada(
+            "tipo-release",
+            Mensaje::nuevo(&txt::TIPO_RELEASE).con("id", id),
+        ));
+    }
+    if !rama.starts_with("hotfix/") {
+        return Ok(git::base_de_rama(rama).to_owned());
+    }
+    if git::commit_de(g, principal, NOMBRE, MAIN)?.is_none() {
+        // `validar-base` lo rechaza con su motivo.
+        return Ok(MAIN.to_owned());
+    }
+    let tags = git::exigir(
+        g,
+        principal,
+        NOMBRE,
+        Subcomando::ForEachRef,
+        &[
+            "--merged",
+            MAIN,
+            "--sort=-v:refname",
+            "--count=1",
+            "--format=%(refname)",
+            "refs/tags/",
+        ],
+    )?;
+    Ok(tags
+        .lines()
+        .map(str::trim)
+        .find(|t| !t.is_empty())
+        .map_or_else(|| MAIN.to_owned(), str::to_owned))
+}
+
+/// La rama y el worktree de la tarea.
+struct Destino<'a> {
+    g: &'a Git,
+    principal: &'a Path,
+    id: &'a str,
+    rama: &'a str,
+    tipo: &'a str,
+    slug: &'a str,
+    base: &'a str,
+}
+
+impl Destino<'_> {
+    /// Crea lo que falte (F1-3, casos b y c): sin rama en local ni en el
+    /// remoto, rama nueva sin upstream desde la base y su worktree; si la
+    /// rama existe, [`git::reanudar`]. `commit` es el de la toma publicada,
+    /// si se acaba de escribir.
+    fn crear(&self, commit: Option<String>, quien: &Quien) -> Informe {
+        let r = self.crear_paso(commit.is_some());
+        let mut i = match r {
+            Ok(i) | Err(i) => i,
+        };
+        let ok = i.resultado.es_exito();
+        let mut s = Informe::nuevo(NOMBRE, i.resultado);
+        s.claves = std::mem::take(&mut i.claves);
+        s.diagnostico = i.diagnostico.take();
+        if let Some(c) = commit {
+            // La toma quedo publicada aunque la rama no se pudiera crear:
+            // repetir `start` crea lo que falte (C2).
+            s = s.con("estado", c);
+        }
+        if ok {
+            s = s.con("asignado", quien.cuenta().login.clone());
+        }
+        s
+    }
+
+    fn crear_paso(&self, nueva: bool) -> Result<Informe, Informe> {
+        let (g, p) = (self.g, self.principal);
+        let local = git::commit_de(g, p, NOMBRE, &format!("refs/heads/{}", self.rama))?;
+        let remota = git::commit_de(
+            g,
+            p,
+            NOMBRE,
+            &format!("refs/remotes/{}/{}", git::REMOTO, self.rama),
+        )?;
+        if local.is_none() && remota.is_none() {
+            let mut i = git::crear_rama_y_worktree(g, p, self.id, self.tipo, self.slug, self.base);
+            if i.resultado == Resultado::Ok && !nueva {
+                i = i.con_lista("creado", vec!["rama".to_owned(), "worktree".to_owned()]);
+            }
+            return Ok(i);
+        }
+        Ok(git::reanudar(g, p, self.id, self.tipo, self.slug))
+    }
+}
diff --git a/crates/tc2/src/toma/operacion.rs b/crates/tc2/src/toma/operacion.rs
new file mode 100644
index 0000000..5109489
--- /dev/null
+++ b/crates/tc2/src/toma/operacion.rs
@@ -0,0 +1,499 @@
+//! La toma de una tarea como operacion de la rama de estado (F3-6; ADR
+//! F1-2, decision 4).
+//!
+//! [`Tomar`] pasa por el punto unico de escritura ([`crate::estado::transaccion`])
+//! y se re-ejecuta en cada intento: decide solo con el estado que recibe.
+//! Lee unicamente el `tarea.md` de la tarea y los de sus dependencias.
+//!
+//! - Ya la lleva quien la toma y esta en curso o en revision: no cambia
+//!   nada (`sin-cambios`, sin commit).
+//! - La lleva otra persona: `rechazada motivo=tomada` (o
+//!   `asignado-heredado` si es un texto de la 0.16).
+//! - Estado de origen (D1 de TASK-033): `planificada` o `en-diseno`; una
+//!   terminada, descartada o en otro estado se rechaza con su motivo.
+//! - Dependencias (D3): con la regla de [`crate::grafo::dependencia_cumplida`];
+//!   alguna sin cumplir, `rechazada motivo=dependencias-pendientes`.
+//! - Si procede escribe `estado: en-curso`, `asignado_a`, `actualizado`,
+//!   `rama` si faltaba y una fila nueva en `## Transiciones`; el resto del
+//!   `tarea.md`, byte a byte.
+//!
+//! Crear la rama y el worktree no es cosa de la operacion: lo hace `start`
+//! fuera de la transaccion y solo con la toma publicada.
+
+use std::cell::RefCell;
+
+use crate::estado::{Cambio, Estado, Fallo, Operacion};
+use crate::grafo::{Nodo, dependencia_cumplida};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::toma as txt;
+use crate::identidad::Quien;
+use crate::tarea::{Asignado, Documento, Transicion};
+
+use super::NOMBRE;
+
+/// Estados en los que la tarea de quien la toma ya esta empezada: `start`
+/// la reanuda sin escribir estado (F1-3, decision 2, caso c).
+pub const EMPEZADA: [&str; 2] = ["en-curso", "en-revision"];
+
+/// Estados desde los que se puede empezar (D1 de TASK-033).
+pub const DE_ORIGEN: [&str; 2] = ["planificada", "en-diseno"];
+
+/// Estado en el que queda la tarea tomada.
+pub const EN_CURSO: &str = "en-curso";
+
+/// Modo de la fila de transicion: la 2.0 aun no congela el modo en la
+/// tarea (F1-10 lo hara en `plan`); hasta entonces, `manual`.
+pub const MODO: &str = "manual";
+
+/// Tomar la tarea `id` para una persona.
+#[derive(Debug)]
+pub struct Tomar {
+    id: String,
+    asignado: Asignado,
+    decidido_por: &'static str,
+    ahora: u64,
+    dueno: RefCell<Option<String>>,
+    pendientes: RefCell<Vec<String>>,
+    rama: RefCell<Option<String>>,
+}
+
+impl Tomar {
+    /// La toma de `id` por `quien` en el instante `ahora` (segundos desde
+    /// 1970, UTC).
+    pub fn nueva(id: &str, quien: &Quien, ahora: u64) -> Self {
+        Self::con(id, quien.asignado(), quien.decidido_por(), ahora)
+    }
+
+    fn con(id: &str, asignado: Asignado, decidido_por: &'static str, ahora: u64) -> Self {
+        Self {
+            id: id.to_owned(),
+            asignado,
+            decidido_por,
+            ahora,
+            dueno: RefCell::new(None),
+            pendientes: RefCell::new(Vec::new()),
+            rama: RefCell::new(None),
+        }
+    }
+
+    /// Login de quien la llevaba si el ultimo intento la rechazo por
+    /// `tomada`.
+    pub fn dueno(&self) -> Option<String> {
+        self.dueno.borrow().clone()
+    }
+
+    /// Dependencias sin cumplir del ultimo intento.
+    pub fn pendientes(&self) -> Vec<String> {
+        self.pendientes.borrow().clone()
+    }
+
+    /// Rama de la tarea vista en el ultimo intento: la de su `rama:` o, si
+    /// faltaba, la derivada del ID, el tipo y el titulo (la que se escribe).
+    pub fn rama(&self) -> Option<String> {
+        self.rama.borrow().clone()
+    }
+
+    fn rechazo(&self, motivo: &str, m: Mensaje) -> Fallo {
+        Fallo::rechazada(motivo, m.con("id", self.id.as_str()))
+    }
+}
+
+impl Operacion for Tomar {
+    fn nombre(&self) -> &str {
+        NOMBRE
+    }
+
+    fn anotacion(&self) -> String {
+        format!(
+            "{NOMBRE}|{}|{}|{}",
+            self.id,
+            self.asignado.id().unwrap_or_default(),
+            self.asignado.login().unwrap_or_default()
+        )
+    }
+
+    fn exige_red(&self) -> bool {
+        // Reserva algo compartido: sin red falla, nunca queda pendiente (D2b).
+        true
+    }
+
+    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
+        *self.dueno.borrow_mut() = None;
+        self.pendientes.borrow_mut().clear();
+        *self.rama.borrow_mut() = None;
+        let ruta = format!("{}/tarea.md", self.id);
+        let Some(texto) = estado.leer(&ruta)? else {
+            return Err(self.rechazo("no-existe", Mensaje::nuevo(&txt::NO_EXISTE)));
+        };
+        let (mut t, mut cuerpo) = match crate::tarea::leer(&texto) {
+            Ok(Documento::Formato2(t, c)) => (t, c),
+            Ok(Documento::Formato1(..)) => {
+                return Err(self.rechazo("formato-1", Mensaje::nuevo(&txt::FORMATO_1)));
+            }
+            Err(e) => {
+                return Err(self.rechazo(
+                    "ilegible",
+                    Mensaje::nuevo(&txt::ILEGIBLE).con("error", e.to_string()),
+                ));
+            }
+        };
+        let rama = t
+            .rama
+            .clone()
+            .unwrap_or_else(|| crate::alta::rama_de(&t.id, &t.tipo, &t.titulo));
+        *self.rama.borrow_mut() = Some(rama.clone());
+        let mia = t
+            .asignado_a
+            .as_ref()
+            .is_some_and(|a| a.misma_persona(&self.asignado));
+        if mia && EMPEZADA.contains(&t.estado.as_str()) {
+            return Ok(Vec::new());
+        }
+        match &t.asignado_a {
+            Some(a) if !mia => {
+                return Err(match (a.login(), a.heredado()) {
+                    (Some(login), _) => {
+                        *self.dueno.borrow_mut() = Some(login.to_owned());
+                        self.rechazo("tomada", Mensaje::nuevo(&txt::TOMADA).con("login", login))
+                    }
+                    (None, texto) => self.rechazo(
+                        "asignado-heredado",
+                        Mensaje::nuevo(&txt::ASIGNADO_HEREDADO)
+                            .con("texto", texto.unwrap_or_default()),
+                    ),
+                });
+            }
+            _ => {}
+        }
+        match t.estado.as_str() {
+            e if DE_ORIGEN.contains(&e) => {}
+            "terminada" => {
+                return Err(self.rechazo("terminada", Mensaje::nuevo(&txt::TERMINADA)));
+            }
+            "descartada" => {
+                return Err(self.rechazo("descartada", Mensaje::nuevo(&txt::DESCARTADA)));
+            }
+            otro => {
+                return Err(self.rechazo(
+                    "estado-no-valido",
+                    Mensaje::nuevo(&txt::ESTADO_NO_VALIDO).con("estado", otro),
+                ));
+            }
+        }
+        let rutas: Vec<String> = t
+            .dependencias
+            .iter()
+            .map(|d| format!("{d}/tarea.md"))
+            .collect();
+        let textos = estado.leer_varios(&rutas)?;
+        let pendientes: Vec<String> = t
+            .dependencias
+            .iter()
+            .zip(textos)
+            .filter(|(d, texto)| {
+                let nodo = texto.as_ref().map(|x| match crate::tarea::leer(x) {
+                    Ok(doc) => Nodo::de(&doc),
+                    Err(_) => Nodo::ilegible(d),
+                });
+                !dependencia_cumplida(nodo.as_ref())
+            })
+            .map(|(d, _)| d.clone())
+            .collect();
+        if !pendientes.is_empty() {
+            let m = Mensaje::nuevo(&txt::DEPENDENCIAS_PENDIENTES).con("ids", pendientes.join(", "));
+            *self.pendientes.borrow_mut() = pendientes;
+            return Err(self.rechazo("dependencias-pendientes", m));
+        }
+        let instante = instante(self.ahora);
+        EN_CURSO.clone_into(&mut t.estado);
+        t.asignado_a = Some(self.asignado.clone());
+        t.actualizado = instante.get(..10).unwrap_or_default().to_owned();
+        if t.rama.is_none() {
+            t.rama = Some(rama);
+        }
+        cuerpo.anadir_transicion(&Transicion {
+            fecha: &instante,
+            fase: NOMBRE,
+            modo: MODO,
+            decidido_por: self.decidido_por,
+            version: crate::VERSION,
+        });
+        Ok(vec![Cambio::nuevo(ruta, t.escribir(&cuerpo))])
+    }
+}
+
+/// `AAAA-MM-DDTHH:MM:SSZ` (UTC) de `segundos` desde 1970.
+pub fn instante(segundos: u64) -> String {
+    let dias = i64::try_from(segundos / 86_400).unwrap_or(0);
+    let resto = segundos % 86_400;
+    format!(
+        "{}T{:02}:{:02}:{:02}Z",
+        crate::alta::fecha_de_dias(dias),
+        resto / 3600,
+        resto % 3600 / 60,
+        resto % 60
+    )
+}
+
+/// Reconstruye un [`Tomar`] desde su anotacion `start|TASK-NNN|<id>|<login>`
+/// (para el catalogo). `start` exige red y nunca queda pendiente; si se
+/// re-ejecutara, lo haria el CLI sin persona delante: `decidido_por: agente`.
+pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
+    let mut partes = a.splitn(4, '|');
+    if partes.next()? != NOMBRE {
+        return None;
+    }
+    let id = partes.next()?;
+    let cuenta: u64 = partes.next()?.parse().ok()?;
+    let asignado = Asignado::persona(cuenta, partes.next()?)?;
+    crate::git::es_id(id).then(|| {
+        Box::new(Tomar::con(
+            id,
+            asignado,
+            "agente",
+            crate::identidad::ahora(),
+        )) as Box<dyn Operacion>
+    })
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::identidad::Cuenta;
+    use crate::plataforma::Proveedor;
+    use crate::roadmap::tests::EnMemoria;
+    use crate::tarea::{Cuerpo, Tarea};
+    use std::collections::BTreeMap;
+
+    /// 2026-11-17T10:20:30Z.
+    const AHORA: u64 = 1_794_910_830;
+
+    fn ana() -> Quien {
+        Quien::Agente(Cuenta {
+            proveedor: Proveedor::GitHub,
+            id: 7,
+            login: "ana".to_owned(),
+        })
+    }
+
+    fn md(id: &str, estado: &str, deps: &[&str], asignado: Option<Asignado>) -> String {
+        let mut t = Tarea::nueva(id, &format!("Tarea {id}"), "2026-11-01");
+        t.estado = estado.to_owned();
+        if estado == "descartada" {
+            t.motivo = Some("no hace falta".to_owned());
+        }
+        t.dependencias = deps.iter().map(|d| (*d).to_owned()).collect();
+        t.asignado_a = asignado;
+        t.rama = Some(crate::alta::rama_de(id, &t.tipo, &t.titulo));
+        t.escribir(&Cuerpo::plantilla())
+    }
+
+    fn estado(tareas: &[(&str, String)]) -> EnMemoria {
+        EnMemoria(
+            tareas
+                .iter()
+                .map(|(id, t)| (format!("{id}/tarea.md"), t.clone()))
+                .collect::<BTreeMap<_, _>>(),
+        )
+    }
+
+    fn motivo(r: &Result<Vec<Cambio>, Fallo>) -> Option<&str> {
+        match r {
+            Err(Fallo::Rechazada { motivo, .. }) => Some(motivo),
+            _ => None,
+        }
+    }
+
+    #[test]
+    fn instante_en_utc() {
+        assert_eq!(instante(0), "1970-01-01T00:00:00Z");
+        assert_eq!(instante(AHORA), "2026-11-17T10:20:30Z");
+        assert_eq!(instante(86_399), "1970-01-01T23:59:59Z");
+    }
+
+    /// C7 (unitario): la toma escribe estado, asignado, actualizado y una
+    /// fila de transicion; el resto del `tarea.md`, byte a byte.
+    #[test]
+    fn la_toma_escribe_solo_lo_suyo() {
+        let antes = md("TASK-001", "planificada", &[], None);
+        let e = estado(&[("TASK-001", antes.clone())]);
+        let op = Tomar::nueva("TASK-001", &ana(), AHORA);
+        let cambios = op.aplicar(&e);
+        assert_eq!(cambios.as_ref().map(Vec::len), Ok(1), "{cambios:?}");
+        let Ok([c]) = cambios.as_deref() else {
+            return;
+        };
+        assert_eq!(c.ruta, "TASK-001/tarea.md");
+        let esperado = antes
+            .replace("estado: \"planificada\"", "estado: \"en-curso\"")
+            .replace("asignado_a: null", "asignado_a: {id: 7, login: \"ana\"}")
+            .replace("actualizado: \"2026-11-01\"", "actualizado: \"2026-11-17\"")
+            + &format!(
+                "| 2026-11-17T10:20:30Z | start | manual | agente | {} |\n",
+                crate::VERSION
+            );
+        assert_eq!(c.contenido, esperado);
+        assert_eq!(
+            op.rama().as_deref(),
+            Some("feature/task-001-tarea-task-001")
+        );
+        // Lo escrito se vuelve a leer como formato 2 sin claves desconocidas.
+        let leida = match crate::tarea::leer(&c.contenido) {
+            Ok(Documento::Formato2(t, _)) => Some(t.claves_desconocidas().len()),
+            _ => None,
+        };
+        assert_eq!(leida, Some(0));
+        assert_eq!(op.anotacion(), "start|TASK-001|7|ana");
+        assert!(op.exige_red());
+    }
+
+    /// Suposicion 1 del plan: una clave desconocida y el cuerpo se conservan
+    /// byte a byte al reescribir; suposicion 4: sin `rama:`, la toma la
+    /// escribe derivada del ID.
+    #[test]
+    fn conserva_claves_desconocidas_y_escribe_la_rama_si_falta() {
+        let base = md("TASK-002", "en-diseno", &[], None).replace(
+            "dependencias: []\n",
+            "dependencias: []\nzonas: [ui, \"api\"] # comentario\n",
+        );
+        let sin_rama = base.replace("rama: \"feature/task-002-tarea-task-002\"", "rama: null");
+        assert_ne!(base, sin_rama);
+        let e = estado(&[("TASK-002", sin_rama.clone())]);
+        let op = Tomar::nueva("TASK-002", &ana(), AHORA);
+        let r = op.aplicar(&e);
+        let texto = r
+            .as_ref()
+            .ok()
+            .and_then(|c| c.first())
+            .map(|c| c.contenido.clone())
+            .unwrap_or_default();
+        // Todo igual que en `base` (que lleva la rama) salvo lo de la toma.
+        let esperado = base
+            .replace("estado: \"en-diseno\"", "estado: \"en-curso\"")
+            .replace("asignado_a: null", "asignado_a: {id: 7, login: \"ana\"}")
+            .replace("actualizado: \"2026-11-01\"", "actualizado: \"2026-11-17\"")
+            + &format!(
+                "| 2026-11-17T10:20:30Z | start | manual | agente | {} |\n",
+                crate::VERSION
+            );
+        assert_eq!(texto, esperado, "{r:?}");
+        assert!(
+            texto.contains("\ndependencias: []\nzonas: [ui, \"api\"] # comentario\n---\n"),
+            "{texto}"
+        );
+        assert_eq!(
+            op.rama().as_deref(),
+            Some("feature/task-002-tarea-task-002")
+        );
+    }
+
+    /// C2 (unitario): siendo el dueno de una tarea en curso o en revision,
+    /// nada que escribir; el dueno de una planificada si la empieza.
+    #[test]
+    fn dueno_no_reescribe_una_empezada() {
+        let yo = ana().asignado();
+        for e in EMPEZADA {
+            let s = estado(&[("TASK-003", md("TASK-003", e, &[], Some(yo.clone())))]);
+            let op = Tomar::nueva("TASK-003", &ana(), AHORA);
+            assert_eq!(op.aplicar(&s), Ok(Vec::new()), "{e}");
+            assert!(op.rama().is_some());
+        }
+        let s = estado(&[("TASK-003", md("TASK-003", "planificada", &[], Some(yo)))]);
+        let r = Tomar::nueva("TASK-003", &ana(), AHORA).aplicar(&s);
+        assert_eq!(r.map(|c| c.len()), Ok(1));
+    }
+
+    /// C1 (unitario): la lleva otra persona: `tomada` con su login; un
+    /// texto heredado de la 0.16, `asignado-heredado`.
+    #[test]
+    fn otra_persona_la_lleva() {
+        let beto = Asignado::persona(9, "beto");
+        for estado_tarea in ["planificada", "en-curso"] {
+            let s = estado(&[("TASK-004", md("TASK-004", estado_tarea, &[], beto.clone()))]);
+            let op = Tomar::nueva("TASK-004", &ana(), AHORA);
+            let r = op.aplicar(&s);
+            assert_eq!(motivo(&r), Some("tomada"));
+            assert_eq!(op.dueno().as_deref(), Some("beto"));
+        }
+        let heredado = md("TASK-005", "planificada", &[], None)
+            .replace("asignado_a: null", "asignado_a: \"ana@x.com\"");
+        let s = estado(&[("TASK-005", heredado)]);
+        let op = Tomar::nueva("TASK-005", &ana(), AHORA);
+        assert_eq!(motivo(&op.aplicar(&s)), Some("asignado-heredado"));
+        assert_eq!(op.dueno(), None);
+    }
+
+    /// C4 y C5 (unitarios): dependencias sin cumplir, estados que no se
+    /// empiezan, formato 1, ilegible e inexistente.
+    #[test]
+    fn rechazos_por_estado_dependencias_y_formato() {
+        let s = estado(&[
+            ("TASK-010", md("TASK-010", "terminada", &[], None)),
+            ("TASK-011", md("TASK-011", "en-curso", &[], None)),
+            ("TASK-012", md("TASK-012", "descartada", &[], None)),
+            (
+                "TASK-013",
+                md(
+                    "TASK-013",
+                    "planificada",
+                    &["TASK-010", "TASK-011", "TASK-012", "TASK-099", "TASK-014"],
+                    None,
+                ),
+            ),
+            ("TASK-014", "---\nformato: 2\nid: x\n---\n".to_owned()),
+            ("TASK-015", md("TASK-015", "en-revision", &[], None)),
+        ]);
+        let op = Tomar::nueva("TASK-013", &ana(), AHORA);
+        assert_eq!(motivo(&op.aplicar(&s)), Some("dependencias-pendientes"));
+        // La inexistente no bloquea (regla de `next`); la ilegible si.
+        assert_eq!(op.pendientes(), ["TASK-011", "TASK-012", "TASK-014"]);
+        for (id, esperado) in [
+            ("TASK-010", "terminada"),
+            ("TASK-012", "descartada"),
+            ("TASK-011", "estado-no-valido"),
+            ("TASK-015", "estado-no-valido"),
+            ("TASK-014", "ilegible"),
+            ("TASK-020", "no-existe"),
+        ] {
+            let r = Tomar::nueva(id, &ana(), AHORA).aplicar(&s);
+            assert_eq!(motivo(&r), Some(esperado), "{id}");
+        }
+        let v1 = "---\nid: TASK-030\ntitulo: \"t\"\ntipo: feature\nsprint: 0\netiquetas: []\n\
+                  complejidad: null\nmodelo_sugerido: sonnet\nestado: planificada\nplan_aprobado: false\n\
+                  rama: feature/task-030-t\nasignado_a: null\nagente_revisor: general-purpose\n\
+                  skills_recomendados: []\nregla_seleccion_skill: null\nultimo_commit_revisado: null\n\
+                  revision_codex: false\ntokens_diseno: null\ntokens_implementacion: null\n\
+                  tokens_revision: null\ncreado: 2026-01-01\nactualizado: 2026-01-01\n\
+                  dependencias: []\n---\n";
+        let s = estado(&[("TASK-030", v1.to_owned())]);
+        let r = Tomar::nueva("TASK-030", &ana(), AHORA).aplicar(&s);
+        assert_eq!(motivo(&r), Some("formato-1"));
+        // El diagnostico lleva el ID y se traduce.
+        if let Err(Fallo::Rechazada { diagnostico, .. }) = r {
+            for idioma in [crate::i18n::Idioma::Es, crate::i18n::Idioma::En] {
+                let d = diagnostico.texto(idioma);
+                assert!(
+                    d.contains("TASK-030") && d.contains("tc2 actualizar"),
+                    "{d}"
+                );
+            }
+        }
+    }
+
+    #[test]
+    fn anotacion_ida_y_vuelta() {
+        let op = Tomar::nueva("TASK-001", &ana(), AHORA);
+        let r = desde_anotacion(&op.anotacion());
+        assert_eq!(r.map(|o| o.anotacion()), Some(op.anotacion()));
+        for mala in [
+            "start|TASK-1|7|ana",
+            "start|TASK-001|x|ana",
+            "start|TASK-001|0|ana",
+            "start|TASK-001|7|",
+            "start|TASK-001|7",
+            "new|TASK-001|7|ana",
+        ] {
+            assert!(desde_anotacion(mala).is_none(), "{mala}");
+        }
+    }
+}
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-033/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-033/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-033/tarea.md                                    | 22 ++++++++++++++++++++--
 3 files changed, 20 insertions(+), 2 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/toma.rs"
````

````
crates/tc2/tests/toma.rs | 726 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 726 insertions(+)
````
