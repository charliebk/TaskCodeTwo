# Peticion de revision — TASK-018 (ronda 1)

- Tarea: TASK-018 — F2-3 Rama de estado
- Rama revisada: feature/task-018-f2-3-rama-de-estado
- Rama base: develop
- Commit revisado (HEAD): 4f80989fc93337d12966d31c9be87260371a8ddd
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-018 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-018/tarea.md
- tareas/03-en-revision/TASK-018/planificacion/
- crates/tc2/src/contrato.rs
- crates/tc2/src/estado/cerrojo.rs
- crates/tc2/src/estado/lectura.rs
- crates/tc2/src/estado/mod.rs
- crates/tc2/src/estado/montaje.rs
- crates/tc2/src/estado/pendientes.rs
- crates/tc2/src/estado/transaccion.rs
- crates/tc2/src/git/estado.rs
- crates/tc2/src/git/mod.rs
- crates/tc2/src/git/puerto.rs
- crates/tc2/src/git/rama.rs
- crates/tc2/src/git/remoto.rs
- crates/tc2/src/git/tag.rs
- crates/tc2/src/lib.rs
- crates/tc2/tests/comun/mod.rs
- crates/tc2/tests/comun/ops.rs
- crates/tc2/tests/estado_casos.rs
- crates/tc2/tests/estado_concurrencia.rs
- crates/tc2/tests/git_casos_limite.rs
- crates/tc2/tests/git_push.rs

## Commits a revisar (git log develop..HEAD)

````
4f80989 chore(TASK-018): criterios y autorrevision
d78931e fix(TASK-018): no perder el error al borrar la ref temporal
24fd49c feat(TASK-018): rama de estado con escritura transaccional
a14ad39 chore(TASK-018): tarea en curso
````

## Diff de tu dominio (20 fichero(s) de 23; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/contrato.rs b/crates/tc2/src/contrato.rs
index 5ceb5c2..0c22af6 100644
--- a/crates/tc2/src/contrato.rs
+++ b/crates/tc2/src/contrato.rs
@@ -5,8 +5,9 @@
 //! diagnostico va a stderr y solo si la operacion no salio bien. Con `--json`
 //! la misma informacion sale como un objeto JSON en una linea.
 //!
-//! La tabla de codigos de salida vive aqui y solo aqui: la propone F1-4 y la
-//! fija F2-3, que solo tendra que tocar este fichero.
+//! La tabla de codigos de salida vive aqui y solo aqui: la propuso F1-4 y la
+//! fija F2-3 (0 ok y sin-cambios, 1 error, 2 uso, 3 rechazada, 4 sinred,
+//! 5 agotada, 6 conflicto, 7 esperando, 8 denegada).
 
 use crate::Salida;
 
@@ -27,6 +28,9 @@ pub enum Resultado {
     Conflicto,
     /// Puertas pendientes en la plataforma (lo emite el puerto Plataforma).
     Esperando,
+    /// El remoto deniega la operacion (credenciales, permisos, un hook o una
+    /// regla, o no existe): no se reintenta, se avisa a la persona (F2-3).
+    Denegada,
     /// La llamada esta mal formada.
     Uso,
     /// Fallo interno: no se reintenta, se avisa a la persona.
@@ -44,12 +48,13 @@ impl Resultado {
             Self::Agotada => "agotada",
             Self::Conflicto => "conflicto",
             Self::Esperando => "esperando",
+            Self::Denegada => "denegada",
             Self::Uso => "uso",
             Self::Error => "error",
         }
     }
 
-    /// Codigo de salida del proceso (tabla de F1-4, decision 5).
+    /// Codigo de salida del proceso (tabla de F1-4, decision 5, fijada en F2-3).
     pub fn codigo(self) -> u8 {
         match self {
             Self::Ok | Self::SinCambios => 0,
@@ -60,6 +65,7 @@ impl Resultado {
             Self::Agotada => 5,
             Self::Conflicto => 6,
             Self::Esperando => 7,
+            Self::Denegada => 8,
         }
     }
 
@@ -261,7 +267,7 @@ mod tests {
     use super::*;
 
     #[test]
-    fn codigos_de_la_tabla_de_f1_4() {
+    fn codigos_de_la_tabla_fijada_en_f2_3() {
         let tabla = [
             (Resultado::Ok, "ok", 0),
             (Resultado::SinCambios, "sin-cambios", 0),
@@ -272,6 +278,7 @@ mod tests {
             (Resultado::Agotada, "agotada", 5),
             (Resultado::Conflicto, "conflicto", 6),
             (Resultado::Esperando, "esperando", 7),
+            (Resultado::Denegada, "denegada", 8),
         ];
         for (r, t, c) in tabla {
             assert_eq!((r.texto(), r.codigo()), (t, c));
diff --git a/crates/tc2/src/estado/cerrojo.rs b/crates/tc2/src/estado/cerrojo.rs
new file mode 100644
index 0000000..0230e75
--- /dev/null
+++ b/crates/tc2/src/estado/cerrojo.rs
@@ -0,0 +1,56 @@
+//! Cerrojo local del clon (`.git/taskcode-estado.lock`, ADR F1-2): la
+//! persona y sus agentes del mismo clon no escriben la rama de estado a la
+//! vez.
+//!
+//! Es un cerrojo del sistema operativo sobre el fichero (`File::try_lock`):
+//! si el proceso muere, el sistema lo suelta, asi que no puede quedar
+//! huerfano (pregunta abierta de F1-2). El fichero se queda; lo que cuenta es
+//! el cerrojo, no su existencia.
+
+use std::fs::{File, OpenOptions, TryLockError};
+use std::path::Path;
+use std::time::{Duration, Instant};
+
+/// Nombre del fichero del cerrojo en la carpeta comun del repo.
+pub const FICHERO: &str = "taskcode-estado.lock";
+
+/// Cerrojo tomado; se suelta al soltar el valor.
+#[derive(Debug)]
+pub struct Cerrojo {
+    _fichero: File,
+}
+
+/// Por que no se tomo el cerrojo.
+#[derive(Debug)]
+pub enum ErrorCerrojo {
+    /// Otro proceso lo tuvo durante toda la espera.
+    Ocupado,
+    /// No se pudo abrir el fichero o pedir el cerrojo.
+    Io(std::io::Error),
+}
+
+impl Cerrojo {
+    /// Toma el cerrojo de la carpeta comun `comun`, esperando como mucho
+    /// `espera`.
+    pub fn tomar(comun: &Path, espera: Duration) -> Result<Self, ErrorCerrojo> {
+        let fichero = OpenOptions::new()
+            .create(true)
+            .truncate(false)
+            .write(true)
+            .open(comun.join(FICHERO))
+            .map_err(ErrorCerrojo::Io)?;
+        let limite = Instant::now() + espera;
+        loop {
+            match fichero.try_lock() {
+                Ok(()) => return Ok(Self { _fichero: fichero }),
+                Err(TryLockError::WouldBlock) => {
+                    if Instant::now() >= limite {
+                        return Err(ErrorCerrojo::Ocupado);
+                    }
+                    std::thread::sleep(Duration::from_millis(20));
+                }
+                Err(TryLockError::Error(e)) => return Err(ErrorCerrojo::Io(e)),
+            }
+        }
+    }
+}
diff --git a/crates/tc2/src/estado/lectura.rs b/crates/tc2/src/estado/lectura.rs
new file mode 100644
index 0000000..64db458
--- /dev/null
+++ b/crates/tc2/src/estado/lectura.rs
@@ -0,0 +1,107 @@
+//! Lectura del estado: una vista sobre un commit de la rama de estado, con
+//! los cambios que la transaccion va acumulando encima. La misma vista la
+//! usan la transaccion (para que la operacion decida sobre el estado fresco)
+//! y la lectura publica.
+
+use std::collections::{BTreeMap, BTreeSet};
+use std::path::{Path, PathBuf};
+
+use super::transaccion::Fallo;
+use super::{CARPETA, OP, RAMA};
+use crate::contrato::{Informe, Resultado};
+use crate::git::estado::{hay_remoto, leer_blob, listar_arbol};
+use crate::git::{Cambio, Git, REMOTO, localizar_principal};
+
+/// Lo que una operacion puede leer del estado.
+pub trait Estado {
+    /// Rutas de todos los ficheros, ordenadas.
+    fn rutas(&self) -> Vec<String>;
+    /// Contenido de `ruta`, o `None` si no existe.
+    fn leer(&self, ruta: &str) -> Result<Option<String>, Fallo>;
+}
+
+/// Vista de un commit de la rama de estado (o del estado vacio si la rama no
+/// existe todavia), con cambios superpuestos.
+#[derive(Debug)]
+pub struct Vista<'a> {
+    git: &'a Git,
+    dir: PathBuf,
+    commit: Option<String>,
+    rutas: BTreeSet<String>,
+    encima: BTreeMap<String, String>,
+}
+
+impl<'a> Vista<'a> {
+    /// Vista de `commit`, leyendo con `git` desde `dir`.
+    pub(crate) fn nueva(git: &'a Git, dir: &Path, commit: Option<String>) -> Result<Self, Informe> {
+        let rutas = match &commit {
+            Some(c) => listar_arbol(git, dir, OP, c)?.into_iter().collect(),
+            None => BTreeSet::new(),
+        };
+        Ok(Self {
+            git,
+            dir: dir.to_path_buf(),
+            commit,
+            rutas,
+            encima: BTreeMap::new(),
+        })
+    }
+
+    /// Commit sobre el que se lee (`None`: la rama no existe).
+    pub fn commit(&self) -> Option<&str> {
+        self.commit.as_deref()
+    }
+
+    /// Superpone cambios (los posteriores ganan).
+    pub(crate) fn superponer(&mut self, cambios: Vec<Cambio>) {
+        for c in cambios {
+            self.encima.insert(c.ruta, c.contenido);
+        }
+    }
+
+    /// Cambios superpuestos, en orden de ruta.
+    pub(crate) fn cambios(&self) -> Vec<Cambio> {
+        self.encima
+            .iter()
+            .map(|(r, c)| Cambio::nuevo(r.clone(), c.clone()))
+            .collect()
+    }
+}
+
+impl Estado for Vista<'_> {
+    fn rutas(&self) -> Vec<String> {
+        let mut v: BTreeSet<String> = self.rutas.clone();
+        v.extend(self.encima.keys().cloned());
+        v.into_iter().collect()
+    }
+
+    fn leer(&self, ruta: &str) -> Result<Option<String>, Fallo> {
+        if let Some(c) = self.encima.get(ruta) {
+            return Ok(Some(c.clone()));
+        }
+        match &self.commit {
+            Some(c) if self.rutas.contains(ruta) => leer_blob(self.git, &self.dir, OP, c, ruta)
+                .map(Some)
+                .map_err(|i| Fallo::Error(i.diagnostico.clone().unwrap_or_else(|| i.linea()))),
+            _ => Ok(None),
+        }
+    }
+}
+
+/// Vista del estado tal como lo conoce este clon: lo ultimo traido de
+/// `origin/taskcode/tareas` (sin `fetch`: la lectura fresca es de F3-5b) o,
+/// en un repo sin remoto, la rama local. Sin rama de estado → `rechazada`.
+pub fn leer<'a>(git: &'a Git, dir: &Path) -> Result<Vista<'a>, Informe> {
+    let principal = localizar_principal(git, dir)?;
+    let referencia = if hay_remoto(git, &principal, OP)? {
+        format!("refs/remotes/{REMOTO}/{RAMA}")
+    } else {
+        format!("refs/heads/{RAMA}")
+    };
+    match crate::git::commit_de(git, &principal, OP, &referencia)? {
+        Some(c) => Vista::nueva(git, &principal, Some(c)),
+        None => Err(Informe::nuevo(OP, Resultado::Rechazada)
+            .con("motivo", "sin-rama-de-estado")
+            .diagnostico(format!("monta la rama de estado en {CARPETA}/ primero"))),
+    }
+}
diff --git a/crates/tc2/src/estado/mod.rs b/crates/tc2/src/estado/mod.rs
new file mode 100644
index 0000000..df882a2
--- /dev/null
+++ b/crates/tc2/src/estado/mod.rs
@@ -0,0 +1,47 @@
+//! Rama de estado `taskcode/tareas` (F2-3; ADR F1-1 y F1-2).
+//!
+//! - [`montar`] crea la rama huerfana si no existe y la monta como worktree
+//!   en `tareas/` del worktree principal (idempotente; la vuelve a montar si
+//!   se desmonto o se borro la carpeta).
+//! - [`transaccion`] es el **unico punto de escritura**: cerrojo local,
+//!   `fetch`, la operacion sobre el estado fresco, commit construido sin
+//!   tocar ningun worktree y `push` sin force; si el push se rechaza por
+//!   non-fast-forward, re-ejecuta la operacion sobre lo nuevo (nunca rebase).
+//!   Sin red, las operaciones que no reservan nada compartido quedan como
+//!   pendientes y se re-ejecutan en la siguiente transaccion con red.
+//! - [`leer`] da una vista del estado (lo ultimo traido del remoto, o la
+//!   rama local en un repo sin remoto).
+//!
+//! La correccion no depende de `tareas/`: se lee y se escribe sobre commits;
+//! la carpeta montada solo se adelanta (fast-forward) tras publicar.
+//!
+//! Salida con el contrato de F1-4 (`estado <resultado> clave=valor`) y los
+//! codigos de la tabla de `contrato.rs`.
+
+mod cerrojo;
+mod lectura;
+mod montaje;
+mod pendientes;
+mod transaccion;
+
+pub use crate::git::Cambio;
+pub use cerrojo::Cerrojo;
+pub use lectura::{Estado, Vista, leer};
+pub use montaje::montar;
+pub use transaccion::{Catalogo, Contexto, Fallo, Operacion, Politica, Roadmap, transaccion};
+
+/// Rama huerfana del estado (ADR F1-1).
+pub const RAMA: &str = "taskcode/tareas";
+
+/// Carpeta del worktree principal donde se monta (ADR F1-1).
+pub const CARPETA: &str = "tareas";
+
+/// Refs de las operaciones pendientes de publicar (sin red, D2b).
+pub const REFS_PENDIENTES: &str = "refs/taskcode/pendientes/";
+
+/// Refs de las pendientes que al re-ejecutarse ya no procedian: su contenido
+/// queda en un objeto de git y no se pierde (pregunta abierta de F1-2).
+pub const REFS_RECHAZADAS: &str = "refs/taskcode/rechazadas/";
+
+/// Nombre de la operacion en la linea de contrato.
+pub(crate) const OP: &str = "estado";
diff --git a/crates/tc2/src/estado/montaje.rs b/crates/tc2/src/estado/montaje.rs
new file mode 100644
index 0000000..40b696f
--- /dev/null
+++ b/crates/tc2/src/estado/montaje.rs
@@ -0,0 +1,200 @@
+//! Montaje de la rama de estado en `tareas/` (ADR F1-1, decision 2).
+//!
+//! La rama huerfana se crea con un commit raiz hecho con `commit-tree` y
+//! publicado por la misma transaccion que cualquier escritura: no hace falta
+//! `worktree add --orphan` (Git 2.42), y si dos clones la crean a la vez el
+//! segundo recibe non-fast-forward, re-ejecuta, la ve creada y solo la monta.
+
+use std::path::Path;
+
+use super::lectura::Estado;
+use super::transaccion::{Contexto, Fallo, Operacion, Politica, transaccion};
+use super::{CARPETA, RAMA};
+use crate::contrato::{Informe, Resultado, corto};
+use crate::git::estado::hay_remoto;
+use crate::git::rama::excluir;
+use crate::git::{
+    Cambio, FalloRemoto, Git, REMOTO, Subcomando, clasificar_fallo_remoto, commit_de, correr,
+    es_ancestro, exigir, localizar_principal, misma_ruta, worktrees,
+};
+
+const OP: &str = "montar-estado";
+
+/// Fichero que deja el commit raiz.
+pub(crate) const LEEME: &str = "LEEME.md";
+
+const TEXTO_LEEME: &str = "# Rama de estado de TaskCodeTwo\n\nEsta rama (`taskcode/tareas`) guarda las tareas y el roadmap (ADR F1-1).\nNo tiene historia comun con las ramas de codigo: no la mezcles con ellas.\nLa escribe el CLI; se monta como worktree en `tareas/`.\n";
+
+/// Operacion que crea la rama: solo escribe [`LEEME`] si no esta.
+struct Inicializar;
+
+impl Operacion for Inicializar {
+    fn nombre(&self) -> &str {
+        "inicializar"
+    }
+    fn anotacion(&self) -> String {
+        "inicializar".to_owned()
+    }
+    fn exige_red(&self) -> bool {
+        true
+    }
+    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
+        if estado.rutas().iter().any(|r| r == LEEME) {
+            Ok(Vec::new())
+        } else {
+            Ok(vec![Cambio::nuevo(LEEME, TEXTO_LEEME)])
+        }
+    }
+}
+
+fn ninguna(_: &str) -> Option<Box<dyn Operacion>> {
+    None
+}
+
+fn renombrado(mut i: Informe) -> Informe {
+    OP.clone_into(&mut i.comando);
+    i
+}
+
+/// Crea la rama de estado si no existe (local, en el remoto o en ningun
+/// sitio) y la monta en `tareas/` del worktree principal. Idempotente: ya
+/// montada → `sin-cambios`. Si la carpeta se borro (tambien con
+/// `git clean -ffdx`), la vuelve a montar. `tareas/` ocupada por otra cosa
+/// (un repo de la 0.16) → `rechazada motivo=tareas-ocupada`.
+pub fn montar(git: &Git, dir: &Path) -> Informe {
+    montar_con(git, dir, Politica::default())
+}
+
+/// Como [`montar`], con otra politica de reintentos para crear la rama.
+pub fn montar_con(git: &Git, dir: &Path, politica: Politica) -> Informe {
+    paso(git, dir, politica).unwrap_or_else(renombrado)
+}
+
+fn paso(git: &Git, dir: &Path, politica: Politica) -> Result<Informe, Informe> {
+    let principal = localizar_principal(git, dir)?;
+    let carpeta = principal.join(CARPETA);
+    let ok = |r: Resultado, commit: Option<String>| {
+        let i = Informe::nuevo(OP, r).con("rama", RAMA).con("ruta", CARPETA);
+        match commit {
+            Some(c) => i.con("commit", corto(&c)),
+            None => i,
+        }
+    };
+
+    for w in worktrees(git, &principal, OP)?
+        .iter()
+        .filter(|w| w.rama.as_deref() == Some(RAMA))
+    {
+        if !w.ruta.exists() {
+            exigir(git, &principal, OP, Subcomando::Worktree, &["prune"])?;
+        } else if misma_ruta(&w.ruta, &carpeta) {
+            excluir(git, &principal, OP, &format!("/{CARPETA}/"))?;
+            return Ok(ok(Resultado::SinCambios, None));
+        } else {
+            return Ok(Informe::nuevo(OP, Resultado::Rechazada)
+                .con("rama", RAMA)
+                .con("motivo", "montada-en-otra-ruta")
+                .diagnostico(format!(
+                    "{RAMA} ya esta montada en {}: quita ese worktree",
+                    w.ruta.display()
+                )));
+        }
+    }
+    let ocupada = std::fs::read_dir(&carpeta)
+        .map(|mut d| d.next().is_some())
+        .unwrap_or(carpeta.exists());
+    if ocupada {
+        return Ok(Informe::nuevo(OP, Resultado::Rechazada)
+            .con("ruta", CARPETA)
+            .con("motivo", "tareas-ocupada")
+            .diagnostico(format!(
+                "{CARPETA}/ ya existe y no es la rama de estado (¿un repo de la 0.16? migralo con actualizar)"
+            )));
+    }
+
+    let remoto = hay_remoto(git, &principal, OP)?;
+    let local = format!("refs/heads/{RAMA}");
+    let remota = format!("refs/remotes/{REMOTO}/{RAMA}");
+
+    if commit_de(git, &principal, OP, &local)?.is_none() {
+        let mut en_remoto = false;
+        if remoto {
+            let refspec = format!("refs/heads/{RAMA}:{remota}");
+            let s = correr(
+                git,
+                &principal,
+                OP,
+                Subcomando::Fetch,
+                &["-q", REMOTO, &refspec],
+            )?;
+            if s.exito() {
+                en_remoto = true;
+            } else {
+                match clasificar_fallo_remoto(&s.stderr) {
+                    FalloRemoto::RamaInexistente => {}
+                    FalloRemoto::SinRed => {
+                        return Ok(Informe::nuevo(OP, Resultado::Sinred)
+                            .con("pendiente", "no")
+                            .diagnostico(format!(
+                                "{}; montar la rama de estado exige red la primera vez",
+                                s.primera_linea_error()
+                            )));
+                    }
+                    otro => {
+                        return Ok(Informe::nuevo(OP, Resultado::Denegada)
+                            .con("motivo", otro.motivo())
+                            .diagnostico(format!(
+                                "{}; revisa credenciales, permisos o la URL de origin",
+                                s.primera_linea_error()
+                            )));
+                    }
+                }
+            }
+        }
+        if en_remoto {
+            exigir(git, &principal, OP, Subcomando::Branch, &[RAMA, &remota])?;
+        } else {
+            let mut c = Contexto::nuevo(git, &principal, &ninguna);
+            c.politica = politica;
+            let i = transaccion(&c, &Inicializar);
+            if !i.resultado.es_exito() {
+                return Ok(renombrado(i));
+            }
+
+            if commit_de(git, &principal, OP, &local)?.is_none() {
+                // Otro clon la creo entre medias: la transaccion la vio en
+                // el remoto y no escribio nada.
+                exigir(git, &principal, OP, Subcomando::Branch, &[RAMA, &remota])?;
+            }
+        }
+    }
+
+    let ruta = carpeta.to_str().ok_or_else(|| {
+        Informe::nuevo(OP, Resultado::Error).diagnostico("la ruta de tareas/ no es UTF-8")
+    })?;
+    exigir(
+        git,
+        &principal,
+        OP,
+        Subcomando::Worktree,
+        &["add", "-q", ruta, RAMA],
+    )?;
+    excluir(git, &principal, OP, &format!("/{CARPETA}/"))?;
+    if remoto
+        && let Some(r) = commit_de(git, &principal, OP, &remota)?
+        && let Some(l) = commit_de(git, &principal, OP, &local)?
+        && l != r
+        && es_ancestro(git, &principal, OP, &l, &r)?
+    {
+        // Lo publicado va por delante de la rama local: fast-forward.
+        let _ = correr(
+            git,
+            &carpeta,
+            OP,
+            Subcomando::Merge,
+            &["-q", "--ff-only", &r],
+        );
+    }
+    let head = commit_de(git, &carpeta, OP, "HEAD")?;
+    Ok(ok(Resultado::Ok, head))
+}
diff --git a/crates/tc2/src/estado/pendientes.rs b/crates/tc2/src/estado/pendientes.rs
new file mode 100644
index 0000000..4eb556e
--- /dev/null
+++ b/crates/tc2/src/estado/pendientes.rs
@@ -0,0 +1,158 @@
+//! Operaciones pendientes de publicar (sin red, ADR F1-2 D2b).
+//!
+//! Cada pendiente es un commit (hecho con `commit-tree`, como los demas) con
+//! la operacion anotada en el mensaje, apuntado por
+//! `refs/taskcode/pendientes/NNNNNN`. La siguiente transaccion con red no
+//! publica ese commit: reconstruye la operacion desde su anotacion y la
+//! re-ejecuta sobre el estado nuevo. Publicada, la ref se borra; si ya no
+//! procede, pasa a `refs/taskcode/rechazadas/` y su contenido sigue en git.
+
+use std::path::Path;
+
+use super::{OP, REFS_PENDIENTES, REFS_RECHAZADAS};
+use crate::git::estado::mensaje_de;
+use crate::git::{Git, Subcomando};
+
+type Paso<T> = Result<T, crate::contrato::Informe>;
+
+/// Prefijo de la linea del mensaje que anota la operacion.
+pub(crate) const ANOTACION: &str = "Operacion: ";
+
+/// Valor "no existe" de `update-ref` (compare-and-swap de creacion).
+const CERO: &str = "0000000000000000000000000000000000000000";
+
+/// Una operacion pendiente.
+#[derive(Debug, Clone)]
+pub(crate) struct Pendiente {
+    pub referencia: String,
+    pub sha: String,
+    /// `None` si el mensaje no trae la anotacion.
+    pub anotacion: Option<String>,
+}
+
+/// Que hacer con una pendiente tras la transaccion.
+#[derive(Debug, Clone)]
+pub(crate) enum Destino {
+    /// Se publico: se borra su ref.
+    Publicada,
+    /// Ya no procede: pasa a rechazadas.
+    Rechazada,
+}
+
+fn refs(git: &Git, dir: &Path, prefijo: &str) -> Paso<Vec<(String, String)>> {
+    let s = crate::git::exigir(
+        git,
+        dir,
+        OP,
+        Subcomando::ForEachRef,
+        &["--format=%(refname) %(objectname)", prefijo],
+    )?;
+    let mut v: Vec<(String, String)> = s
+        .lines()
+        .filter_map(|l| l.split_once(' '))
+        .map(|(r, o)| (r.to_owned(), o.to_owned()))
+        .collect();
+    v.sort();
+    Ok(v)
+}
+
+/// Ultima linea `Operacion: ...` del mensaje.
+pub(crate) fn anotacion_de(mensaje: &str) -> Option<String> {
+    mensaje
+        .lines()
+        .rev()
+        .find_map(|l| l.strip_prefix(ANOTACION))
+        .map(|a| a.trim().to_owned())
+        .filter(|a| !a.is_empty())
+}
+
+/// Pendientes en orden de llegada.
+pub(crate) fn listar(git: &Git, dir: &Path) -> Paso<Vec<Pendiente>> {
+    let mut v = Vec::new();
+    for (referencia, sha) in refs(git, dir, REFS_PENDIENTES)? {
+        let anotacion = anotacion_de(&mensaje_de(git, dir, OP, &sha)?);
+        v.push(Pendiente {
+            referencia,
+            sha,
+            anotacion,
+        });
+    }
+    Ok(v)
+}
+
+fn numero(referencia: &str) -> u64 {
+    referencia
+        .rsplit('/')
+        .next()
+        .and_then(|n| n.parse().ok())
+        .unwrap_or(0)
+}
+
+/// Guarda `sha` como la siguiente pendiente y devuelve su ref. Los numeros
+/// no se reutilizan mientras quede una rechazada con ese numero.
+pub(crate) fn guardar(git: &Git, dir: &Path, sha: &str) -> Paso<String> {
+    let mut maximo = 0;
+    for prefijo in [REFS_PENDIENTES, REFS_RECHAZADAS] {
+        for (r, _) in refs(git, dir, prefijo)? {
+            maximo = maximo.max(numero(&r));
+        }
+    }
+    let referencia = format!("{REFS_PENDIENTES}{:06}", maximo + 1);
+    crate::git::exigir(
+        git,
+        dir,
+        OP,
+        Subcomando::UpdateRef,
+        &[&referencia, sha, CERO],
+    )?;
+    Ok(referencia)
+}
+
+/// Aplica los destinos. Devuelve (publicadas, rechazadas).
+pub(crate) fn resolver(
+    git: &Git,
+    dir: &Path,
+    destinos: &[(Pendiente, Destino)],
+) -> Paso<(usize, usize)> {
+    let (mut publicadas, mut rechazadas) = (0, 0);
+    for (p, d) in destinos {
+        if let Destino::Rechazada = d {
+            let nombre = p.referencia.trim_start_matches(REFS_PENDIENTES);
+            let destino = format!("{REFS_RECHAZADAS}{nombre}");
+            crate::git::exigir(
+                git,
+                dir,
+                OP,
+                Subcomando::UpdateRef,
+                &[&destino, &p.sha, CERO],
+            )?;
+            rechazadas += 1;
+        } else {
+            publicadas += 1;
+        }
+        crate::git::exigir(
+            git,
+            dir,
+            OP,
+            Subcomando::UpdateRef,
+            &["-d", &p.referencia, &p.sha],
+        )?;
+    }
+    Ok((publicadas, rechazadas))
+}
+
+#[cfg(test)]
+mod tests {
+    use super::{anotacion_de, numero};
+
+    #[test]
+    fn anotacion_y_numero() {
+        assert_eq!(
+            anotacion_de("estado: anotar\n\nOperacion: anotar|a.md\n").as_deref(),
+            Some("anotar|a.md")
+        );
+        assert_eq!(anotacion_de("sin anotacion\n"), None);
+        assert_eq!(numero("refs/taskcode/pendientes/000012"), 12);
+        assert_eq!(numero("refs/taskcode/pendientes/x"), 0);
+    }
+}
diff --git a/crates/tc2/src/estado/transaccion.rs b/crates/tc2/src/estado/transaccion.rs
new file mode 100644
index 0000000..b78c067
--- /dev/null
+++ b/crates/tc2/src/estado/transaccion.rs
@@ -0,0 +1,550 @@
+//! Punto unico de escritura de la rama de estado (ADR F1-2, decision 1 y 2).
+//!
+//! Por intento: cerrojo local, `fetch` solo de la rama de estado, las
+//! pendientes y la operacion sobre ese estado fresco, `ROADMAP.md`
+//! regenerado si hay generador, commit con `commit-tree` y `push` sin force.
+//! El rechazo non-fast-forward es el compare-and-swap: no queda nada local
+//! que descartar y se vuelve a empezar re-ejecutando la operacion, con
+//! espera `base * 2^(n-1) * U(0,5; 1)`. Agotados los intentos, `agotada` sin
+//! nada escrito.
+
+use std::hash::{BuildHasher, Hasher, RandomState};
+use std::path::Path;
+use std::time::Duration;
+
+use super::cerrojo::{Cerrojo, ErrorCerrojo};
+use super::lectura::{Estado, Vista};
+use super::pendientes::{self, ANOTACION, Destino};
+use super::{CARPETA, OP, RAMA};
+use crate::contrato::{Informe, Resultado, corto};
+use crate::git::estado::{construir_commit, dir_comun, hay_remoto};
+use crate::git::remoto::publicar;
+use crate::git::{
+    Cambio, FalloRemoto, Git, REMOTO, Subcomando, clasificar_fallo_remoto, commit_de, correr,
+    es_ancestro, exigir, localizar_principal, worktrees,
+};
+
+/// Por que una operacion no produce cambios.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum Fallo {
+    /// La operacion no procede sobre este estado (la tarea la lleva otro, no
+    /// existe...): `rechazada` (3), no se reintenta.
+    Rechazada {
+        /// Valor de `motivo=`.
+        motivo: String,
+        /// Que hacer, en una linea.
+        diagnostico: String,
+    },
+    /// Fallo interno: `error` (1).
+    Error(String),
+}
+
+impl Fallo {
+    /// Atajo para [`Fallo::Rechazada`].
+    pub fn rechazada(motivo: impl Into<String>, diagnostico: impl Into<String>) -> Self {
+        Self::Rechazada {
+            motivo: motivo.into(),
+            diagnostico: diagnostico.into(),
+        }
+    }
+}
+
+/// Una operacion sobre el estado. Se re-ejecuta en cada intento: tiene que
+/// decidir solo a partir del estado que recibe.
+pub trait Operacion {
+    /// Nombre corto (`alta`, `start`...), para la linea y el commit.
+    fn nombre(&self) -> &str;
+    /// Una linea que permite reconstruirla con el [`Catalogo`] (las
+    /// pendientes se re-ejecutan desde ella).
+    fn anotacion(&self) -> String;
+    /// Si reserva algo compartido (`new`, `import`, `start`, `liberar`,
+    /// `reasignar`): sin red falla en vez de quedar pendiente (D2b).
+    fn exige_red(&self) -> bool;
+    /// Los cambios sobre `estado`, o por que no procede.
+    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo>;
+}
+
+/// Reconstruye una operacion desde su anotacion (`None`: desconocida).
+pub type Catalogo<'a> = &'a dyn Fn(&str) -> Option<Box<dyn Operacion>>;
+
+/// Genera `ROADMAP.md` desde el estado ya cambiado (se regenera, nunca se
+/// mergea: ADR F1-1). Lo pone F3-4; hasta entonces, `None`.
+pub type Roadmap<'a> = &'a dyn Fn(&dyn Estado) -> Result<String, Fallo>;
+
+/// Reintentos y esperas (ADR F1-2, D3a: 5 intentos, base 2 s).
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub struct Politica {
+    /// Intentos de publicar antes de `agotada`.
+    pub intentos: u32,
+    /// Base de la espera exponencial con jitter.
+    pub base: Duration,
+    /// Espera maxima por el cerrojo local.
+    pub espera_cerrojo: Duration,
+}
+
+impl Default for Politica {
+    fn default() -> Self {
+        Self {
+            intentos: 5,
+            base: Duration::from_secs(2),
+            espera_cerrojo: Duration::from_secs(60),
+        }
+    }
+}
+
+/// Todo lo que necesita una transaccion.
+pub struct Contexto<'a> {
+    /// Puerto Git.
+    pub git: &'a Git,
+    /// Carpeta desde la que se lanza (cualquier worktree del repo).
+    pub dir: &'a Path,
+    /// Reintentos y esperas.
+    pub politica: Politica,
+    /// Para re-ejecutar las pendientes.
+    pub catalogo: Catalogo<'a>,
+    /// Generador de `ROADMAP.md`, si lo hay.
+    pub roadmap: Option<Roadmap<'a>>,
+}
+
+impl<'a> Contexto<'a> {
+    /// Contexto con la politica por defecto y sin generador de roadmap.
+    pub fn nuevo(git: &'a Git, dir: &'a Path, catalogo: Catalogo<'a>) -> Self {
+        Self {
+            git,
+            dir,
+            politica: Politica::default(),
+            catalogo,
+            roadmap: None,
+        }
+    }
+}
+
+enum Intento {
+    Hecho(Informe),
+    Reintentar,
+}
+
+/// Como quedo la rama local y `tareas/` tras publicar.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+enum Tareas {
+    /// `tareas/` montada y adelantada.
+    AlDia,
+    /// Sin montar: se adelanto (o creo) la rama local.
+    SinMontar,
+    /// Montada, pero con cambios de la persona que impiden adelantarla.
+    ConCambios,
+    /// La rama local tiene commits que no estan en lo publicado.
+    Diverge,
+}
+
+impl Tareas {
+    fn texto(self) -> &'static str {
+        match self {
+            Self::AlDia => "al-dia",
+            Self::SinMontar => "sin-montar",
+            Self::ConCambios => "con-cambios",
+            Self::Diverge => "diverge",
+        }
+    }
+}
+
+fn informe(r: Resultado, op: &dyn Operacion) -> Informe {
+    Informe::nuevo(OP, r).con("operacion", op.nombre())
+}
+
+fn error(diagnostico: impl Into<String>) -> Informe {
+    Informe::nuevo(OP, Resultado::Error).diagnostico(diagnostico)
+}
+
+fn renombrado(mut i: Informe) -> Informe {
+    OP.clone_into(&mut i.comando);
+    i
+}
+
+/// Ejecuta `op` sobre la rama de estado con el protocolo de F1-2.
+///
+/// Resultados: `ok` (publicada; con `tareas=` dice si la carpeta montada se
+/// adelanto), `sin-cambios`, `rechazada` (3, la operacion no procede),
+/// `sinred` (4, `pendiente=si|no`), `agotada` (5, nada escrito), `denegada`
+/// (8, credenciales, hook o regla del remoto) o `error`.
+pub fn transaccion(c: &Contexto<'_>, op: &dyn Operacion) -> Informe {
+    ejecutar(c, op).unwrap_or_else(renombrado)
+}
+
+fn ejecutar(c: &Contexto<'_>, op: &dyn Operacion) -> Result<Informe, Informe> {
+    let anotacion = op.anotacion();
+    if anotacion.trim().is_empty() || anotacion.contains(['\n', '\r']) {
+        return Ok(informe(Resultado::Uso, op).con("motivo", "anotacion-invalida"));
+    }
+    if c.politica.intentos == 0 {
+        return Ok(informe(Resultado::Uso, op).con("motivo", "sin-intentos"));
+    }
+    let principal = localizar_principal(c.git, c.dir)?;
+    let comun = dir_comun(c.git, &principal, OP)?;
+    let remoto = hay_remoto(c.git, &principal, OP)?;
+    let mut n = 0;
+    loop {
+        n += 1;
+        let cerrojo = match Cerrojo::tomar(&comun, c.politica.espera_cerrojo) {
+            Ok(x) => x,
+            Err(ErrorCerrojo::Ocupado) => {
+                return Ok(informe(Resultado::Agotada, op)
+                    .con("motivo", "cerrojo")
+                    .diagnostico(
+                        "otro proceso de este clon esta escribiendo la rama de estado: reintenta",
+                    ));
+            }
+            Err(ErrorCerrojo::Io(e)) => return Err(error(format!("cerrojo: {e}"))),
+        };
+        let r = intento(c, op, &anotacion, &principal, remoto, n);
+        drop(cerrojo);
+        match r? {
+            Intento::Hecho(i) => return Ok(i),
+            Intento::Reintentar if n >= c.politica.intentos => {
+                return Ok(informe(Resultado::Agotada, op)
+                    .con("intentos", n.to_string())
+                    .diagnostico(
+                        "la rama de estado cambio en cada intento: no se ha escrito nada; reintenta",
+                    ));
+            }
+            Intento::Reintentar => std::thread::sleep(espera(c.politica.base, n)),
+        }
+    }
+}
+
+/// `base * 2^(n-1) * U(0,5; 1)`. El azar sale de `RandomState` (claves
+/// aleatorias por proceso), sin dependencias.
+fn espera(base: Duration, n: u32) -> Duration {
+    let azar = RandomState::new().build_hasher().finish() % 1000;
+    let factor = 0.5 + f64::from(u32::try_from(azar).unwrap_or(0)) / 2000.0;
+    let exp = 2u32.saturating_pow(n.saturating_sub(1)).min(1024);
+    base.mul_f64(f64::from(exp) * factor)
+}
+
+fn mensaje(nombre: &str, anotaciones: &[String]) -> String {
+    let mut m = format!("estado: {nombre}\n\n");
+    for a in anotaciones {
+        m.push_str(ANOTACION);
+        m.push_str(a);
+        m.push('\n');
+    }
+    m
+}
+
+fn rama_remota() -> String {
+    format!("refs/remotes/{REMOTO}/{RAMA}")
+}
+
+fn rama_local() -> String {
+    format!("refs/heads/{RAMA}")
+}
+
+fn de_fallo(f: Fallo) -> Informe {
+    match f {
+        Fallo::Error(d) => error(d),
+        Fallo::Rechazada {
+            motivo,
+            diagnostico,
+        } => error(format!("{motivo}: {diagnostico}")),
+    }
+}
+
+fn intento(
+    c: &Contexto<'_>,
+    op: &dyn Operacion,
+    anotacion: &str,
+    principal: &Path,
+    remoto: bool,
+    n: u32,
+) -> Result<Intento, Informe> {
+    let git = c.git;
+    let base = if remoto {
+        let refspec = format!("refs/heads/{RAMA}:{}", rama_remota());
+        let s = correr(
+            git,
+            principal,
+            OP,
+            Subcomando::Fetch,
+            &["-q", REMOTO, &refspec],
+        )?;
+        if s.exito() {
+            commit_de(git, principal, OP, &rama_remota())?
+        } else {
+            match clasificar_fallo_remoto(&s.stderr) {
+                FalloRemoto::RamaInexistente => None,
+                FalloRemoto::SinRed => {
+                    return sin_red(c, op, anotacion, principal, &s.primera_linea_error());
+                }
+                otro => return Ok(Intento::Hecho(denegada(op, otro.motivo(), &s.stderr))),
+            }
+        }
+    } else {
+        commit_de(git, principal, OP, &rama_local())?
+    };
+
+    let mut vista = Vista::nueva(git, principal, base.clone())?;
+    let mut anotaciones = Vec::new();
+    let mut destinos = Vec::new();
+    if remoto {
+        for p in pendientes::listar(git, principal)? {
+            let reconstruida = p.anotacion.as_deref().and_then(|a| (c.catalogo)(a));
+            let Some(o) = reconstruida else {
+                destinos.push((p, Destino::Rechazada));
+                continue;
+            };
+            match o.aplicar(&vista) {
+                Ok(cambios) => {
+                    vista.superponer(cambios);
+                    anotaciones.push(o.anotacion());
+                    destinos.push((p, Destino::Publicada));
+                }
+                Err(Fallo::Rechazada { .. }) => destinos.push((p, Destino::Rechazada)),
+                Err(Fallo::Error(d)) => return Err(error(d)),
+            }
+        }
+    }
+    match op.aplicar(&vista) {
+        Ok(cambios) => vista.superponer(cambios),
+        Err(Fallo::Rechazada {
+            motivo,
+            diagnostico,
+        }) => {
+            return Ok(Intento::Hecho(
+                informe(Resultado::Rechazada, op)
+                    .con("motivo", motivo)
+                    .diagnostico(diagnostico),
+            ));
+        }
+        Err(Fallo::Error(d)) => return Err(error(d)),
+    }
+    anotaciones.push(anotacion.to_owned());
+    if let Some(generar) = c.roadmap {
+        let texto = generar(&vista).map_err(de_fallo)?;
+        vista.superponer(vec![Cambio::nuevo("ROADMAP.md", texto)]);
+    }
+
+    let mensaje = mensaje(op.nombre(), &anotaciones);
+    let Some(sha) = construir_commit(
+        git,
+        principal,
+        OP,
+        base.as_deref(),
+        &vista.cambios(),
+        &mensaje,
+    )?
+    else {
+        let (publicadas, rechazadas) = pendientes::resolver(git, principal, &destinos)?;
+        return Ok(Intento::Hecho(con_pendientes(
+            informe(Resultado::SinCambios, op),
+            publicadas,
+            rechazadas,
+        )));
+    };
+
+    if !remoto {
+        // Sin remoto el commit local es la transaccion (RN-06); el cerrojo
+        // serializa y `adelantar` comprueba que la rama no se movio.
+        return Ok(match adelantar(git, principal, &sha)? {
+            Tareas::Diverge => Intento::Reintentar,
+            Tareas::ConCambios => Intento::Hecho(
+                informe(Resultado::Rechazada, op)
+                    .con("motivo", "tareas-con-cambios")
+                    .diagnostico(format!(
+                        "{CARPETA}/ tiene cambios que chocan con la operacion: guardalos o deshazlos y reintenta"
+                    )),
+            ),
+            t => Intento::Hecho(ok(op, &sha, n, t, 0, 0)),
+        });
+    }
+
+    let p = publicar(git, principal, OP, &format!("{sha}:refs/heads/{RAMA}"), &[])?;
+    match p.resultado {
+        Resultado::Ok | Resultado::SinCambios => {
+            let (publicadas, rechazadas) = pendientes::resolver(git, principal, &destinos)?;
+            let t = adelantar(git, principal, &sha)?;
+            Ok(Intento::Hecho(ok(op, &sha, n, t, publicadas, rechazadas)))
+        }
+        Resultado::Rechazada => Ok(Intento::Reintentar),
+        Resultado::Denegada => Ok(Intento::Hecho(denegada(op, p.motivo, &p.ultimo))),
+        _ => sin_red(c, op, anotacion, principal, &p.ultimo),
+    }
+}
+
+fn denegada(op: &dyn Operacion, motivo: &str, detalle: &str) -> Informe {
+    let primera = detalle
+        .lines()
+        .map(str::trim)
+        .find(|l| !l.is_empty())
+        .unwrap_or("sin mensaje");
+    informe(Resultado::Denegada, op)
+        .con("motivo", motivo)
+        .diagnostico(format!(
+            "{primera}; el remoto deniega la escritura: revisa credenciales, permisos o reglas, no se reintenta"
+        ))
+}
+
+fn con_pendientes(mut i: Informe, publicadas: usize, rechazadas: usize) -> Informe {
+    if publicadas > 0 {
+        i = i.con("pendientes", publicadas.to_string());
+    }
+    if rechazadas > 0 {
+        i = i.con("rechazadas", rechazadas.to_string());
+    }
+    i
+}
+
+fn ok(
+    op: &dyn Operacion,
+    sha: &str,
+    n: u32,
+    t: Tareas,
+    publicadas: usize,
+    rechazadas: usize,
+) -> Informe {
+    con_pendientes(
+        informe(Resultado::Ok, op)
+            .con("commit", corto(sha))
+            .con("intentos", n.to_string())
+            .con("tareas", t.texto()),
+        publicadas,
+        rechazadas,
+    )
+}
+
+/// Sin red: si la operacion exige red, `sinred pendiente=no` sin escribir
+/// nada; si no, se comprueba contra lo ultimo conocido (una tarea ajena se
+/// rechaza tambien sin red) y queda como pendiente.
+fn sin_red(
+    c: &Contexto<'_>,
+    op: &dyn Operacion,
+    anotacion: &str,
+    principal: &Path,
+    detalle: &str,
+) -> Result<Intento, Informe> {
+    if op.exige_red() {
+        return Ok(Intento::Hecho(
+            informe(Resultado::Sinred, op)
+                .con("pendiente", "no")
+                .diagnostico(format!(
+                    "{detalle}; esta operacion exige red: no se ha escrito nada, reintenta con red"
+                )),
+        ));
+    }
+    let git = c.git;
+    let previas = pendientes::listar(git, principal)?;
+    let conocida = match previas.last() {
+        Some(p) => Some(p.sha.clone()),
+        None => commit_de(git, principal, OP, &rama_remota())?,
+    };
+    let mut vista = Vista::nueva(git, principal, conocida.clone())?;
+    match op.aplicar(&vista) {
+        Ok(cambios) => vista.superponer(cambios),
+        Err(Fallo::Rechazada {
+            motivo,
+            diagnostico,
+        }) => {
+            return Ok(Intento::Hecho(
+                informe(Resultado::Rechazada, op)
+                    .con("motivo", motivo)
+                    .diagnostico(diagnostico),
+            ));
+        }
+        Err(Fallo::Error(d)) => return Err(error(d)),
+    }
+    let mensaje = mensaje(op.nombre(), &[anotacion.to_owned()]);
+    let Some(sha) = construir_commit(
+        git,
+        principal,
+        OP,
+        conocida.as_deref(),
+        &vista.cambios(),
+        &mensaje,
+    )?
+    else {
+        return Ok(Intento::Hecho(informe(Resultado::SinCambios, op)));
+    };
+    pendientes::guardar(git, principal, &sha)?;
+    Ok(Intento::Hecho(
+        informe(Resultado::Sinred, op)
+            .con("pendiente", "si")
+            .con("pendientes", (previas.len() + 1).to_string())
+            .diagnostico(format!(
+                "{detalle}; queda pendiente de publicar: se publicara en la siguiente operacion con red"
+            )),
+    ))
+}
+
+/// Lleva la rama local `taskcode/tareas` (y `tareas/` si esta montada) a
+/// `sha`, solo por fast-forward.
+fn adelantar(git: &Git, principal: &Path, sha: &str) -> Result<Tareas, Informe> {
+    let local = commit_de(git, principal, OP, &rama_local())?;
+    if let Some(l) = &local
+        && !es_ancestro(git, principal, OP, l, sha)?
+    {
+        return Ok(Tareas::Diverge);
+    }
+    let wts = worktrees(git, principal, OP)?;
+    let mut huerfano = false;
+    for w in wts.iter().filter(|w| w.rama.as_deref() == Some(RAMA)) {
+        if !w.ruta.exists() {
+            huerfano = true;
+            continue;
+        }
+        let s = correr(
+            git,
+            &w.ruta,
+            OP,
+            Subcomando::Merge,
+            &["-q", "--ff-only", sha],
+        )?;
+        return Ok(if s.exito() {
+            Tareas::AlDia
+        } else {
+            Tareas::ConCambios
+        });
+    }
+    if huerfano {
+        // La carpeta se borro a mano: git aun cree la rama en uso.
+        exigir(git, principal, OP, Subcomando::Worktree, &["prune"])?;
+    }
+    match local {
+        None => {
+            exigir(git, principal, OP, Subcomando::Branch, &[RAMA, sha])?;
+        }
+        Some(_) => {
+            // La rama no esta en ningun worktree: se adelanta con un fetch
+            // local sin `+` (solo fast-forward) desde una ref temporal propia.
+            let tmp = format!(
+                "{}tmp/{}",
+                crate::git::PREFIJO_REFS_PROPIAS,
+                std::process::id()
+            );
+            exigir(git, principal, OP, Subcomando::UpdateRef, &[&tmp, sha])?;
+            let s = correr(
+                git,
+                principal,
+                OP,
+                Subcomando::Fetch,
+                &["-q", ".", &format!("{tmp}:{}", rama_local())],
+            );
+            // La ref temporal se borra tambien si el fetch fallo; si no se
+            // puede borrar, se dice (no se deja basura sin avisar).
+            let borrado = exigir(
+                git,
+                principal,
+                OP,
+                Subcomando::UpdateRef,
+                &["-d", &tmp, sha],
+            );
+            let s = s?;
+            if !s.exito() {
+                return Err(error(format!(
+                    "no se pudo adelantar {RAMA}: {}",
+                    s.primera_linea_error()
+                )));
+            }
+            borrado?;
+        }
+    }
+    Ok(Tareas::SinMontar)
+}
diff --git a/crates/tc2/src/git/estado.rs b/crates/tc2/src/git/estado.rs
new file mode 100644
index 0000000..87cb44f
--- /dev/null
+++ b/crates/tc2/src/git/estado.rs
@@ -0,0 +1,274 @@
+//! Operacion `commit-estado` de la tabla de F1-4: commits de la rama de
+//! estado construidos sin tocar ningun worktree ni el indice del repo
+//! (F2-3). El arbol se arma en un indice temporal sobre el commit base y el
+//! commit sale de `commit-tree`; publicarlo es un `push <sha>:<rama>` sin
+//! force. Asi, tras un push rechazado no hay nada local que descartar: no
+//! hace falta `reset` (prohibido por F1-4 D3).
+
+use std::path::{Component, Path, PathBuf};
+use std::sync::atomic::{AtomicUsize, Ordering};
+
+use super::{Git, Paso, Subcomando, correr, exigir};
+use crate::contrato::{Informe, Resultado};
+
+/// Un fichero que la operacion anade o cambia en la rama de estado. No hay
+/// borrado: las carpetas de tarea no se borran nunca (ADR F1-1).
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Cambio {
+    /// Ruta relativa a la raiz de la rama, con `/`.
+    pub ruta: String,
+    /// Contenido completo del fichero.
+    pub contenido: String,
+}
+
+impl Cambio {
+    /// Cambio de `ruta` con `contenido`.
+    pub fn nuevo(ruta: impl Into<String>, contenido: impl Into<String>) -> Self {
+        Self {
+            ruta: ruta.into(),
+            contenido: contenido.into(),
+        }
+    }
+}
+
+/// Ruta valida dentro de la rama: relativa, con `/`, sin `.`, `..`, partes
+/// vacias ni `.git`.
+pub(crate) fn ruta_valida(ruta: &str) -> bool {
+    !ruta.is_empty()
+        && !ruta.contains('\\')
+        && !ruta.contains(',')
+        && !ruta.contains(':')
+        && !ruta.chars().any(char::is_control)
+        && ruta
+            .split('/')
+            .all(|p| !p.is_empty() && p != "." && p != ".." && !p.eq_ignore_ascii_case(".git"))
+        && Path::new(ruta)
+            .components()
+            .all(|c| matches!(c, Component::Normal(_)))
+}
+
+/// Carpeta comun del repo (`.git` del principal), absoluta.
+pub(crate) fn dir_comun(git: &Git, dir: &Path, op: &str) -> Paso<PathBuf> {
+    let s = exigir(
+        git,
+        dir,
+        op,
+        Subcomando::RevParse,
+        &["--path-format=absolute", "--git-common-dir"],
+    )?;
+    Ok(PathBuf::from(s.trim()))
+}
+
+/// Rutas de todos los ficheros de `commit`, ordenadas.
+pub(crate) fn listar_arbol(git: &Git, dir: &Path, op: &str, commit: &str) -> Paso<Vec<String>> {
+    let s = exigir(
+        git,
+        dir,
+        op,
+        Subcomando::LsTree,
+        &["-r", "-z", "--name-only", "--end-of-options", commit],
+    )?;
+    let mut v: Vec<String> = s
+        .split('\0')
+        .filter(|r| !r.is_empty())
+        .map(str::to_owned)
+        .collect();
+    v.sort();
+    Ok(v)
+}
+
+/// Contenido de `ruta` en `commit`.
+pub(crate) fn leer_blob(git: &Git, dir: &Path, op: &str, commit: &str, ruta: &str) -> Paso<String> {
+    exigir(
+        git,
+        dir,
+        op,
+        Subcomando::Show,
+        &[&format!("{commit}:{ruta}")],
+    )
+}
+
+/// Mensaje completo de un commit.
+pub(crate) fn mensaje_de(git: &Git, dir: &Path, op: &str, commit: &str) -> Paso<String> {
+    exigir(
+        git,
+        dir,
+        op,
+        Subcomando::Show,
+        &["-s", "--format=%B", "--end-of-options", commit],
+    )
+}
+
+static TEMPORALES: AtomicUsize = AtomicUsize::new(0);
+
+/// Carpeta temporal bajo la carpeta comun; se borra al soltarla, tambien si
+/// la operacion termina con error.
+struct Temporal(PathBuf);
+
+impl Temporal {
+    fn nueva(comun: &Path, op: &str) -> Paso<Self> {
+        let n = TEMPORALES.fetch_add(1, Ordering::SeqCst);
+        let ruta = comun
+            .join("taskcode-estado-tmp")
+            .join(format!("{}-{n}", std::process::id()));
+        let _ = std::fs::remove_dir_all(&ruta);
+        std::fs::create_dir_all(&ruta).map_err(|e| {
+            Informe::nuevo(op, Resultado::Error).diagnostico(format!("{}: {e}", ruta.display()))
+        })?;
+        Ok(Self(ruta))
+    }
+}
+
+impl Drop for Temporal {
+    fn drop(&mut self) {
+        let _ = std::fs::remove_dir_all(&self.0);
+    }
+}
+
+fn texto(op: &str, p: &Path) -> Paso<String> {
+    p.to_str()
+        .map(str::to_owned)
+        .ok_or_else(|| Informe::nuevo(op, Resultado::Error).diagnostico("ruta temporal no UTF-8"))
+}
+
+fn con_indice(
+    git: &Git,
+    dir: &Path,
+    op: &str,
+    sub: Subcomando,
+    args: &[&str],
+    indice: &Path,
+) -> Paso<String> {
+    let s = git
+        .ejecutar_con_indice(dir, sub, args, indice)
+        .map_err(|e| {
+            Informe::nuevo(op, Resultado::Error)
+                .con("git", sub.nombre())
+                .diagnostico(e.to_string())
+        })?;
+    if s.exito() {
+        Ok(s.stdout)
+    } else {
+        Err(Informe::nuevo(op, Resultado::Error)
+            .con("git", sub.nombre())
+            .diagnostico(format!(
+                "git {} fallo: {}",
+                sub.nombre(),
+                s.primera_linea_error()
+            )))
+    }
+}
+
+/// Construye un commit con el arbol de `base` mas `cambios` y `base` como
+/// padre (sin padre si `base` es `None`: la raiz de la rama huerfana). No
+/// mueve ninguna rama. `None` si el arbol queda igual que el de `base`.
+pub(crate) fn construir_commit(
+    git: &Git,
+    dir: &Path,
+    op: &str,
+    base: Option<&str>,
+    cambios: &[Cambio],
+    mensaje: &str,
+) -> Paso<Option<String>> {
+    if let Some(malo) = cambios.iter().find(|c| !ruta_valida(&c.ruta)) {
+        return Err(Informe::nuevo(op, Resultado::Error)
+            .con("motivo", "ruta-invalida")
+            .diagnostico(format!("ruta invalida en la rama de estado: {}", malo.ruta)));
+    }
+    let comun = dir_comun(git, dir, op)?;
+    let tmp = Temporal::nueva(&comun, op)?;
+    let indice = tmp.0.join("indice");
+    if let Some(b) = base {
+        con_indice(git, dir, op, Subcomando::ReadTree, &[b], &indice)?;
+    }
+    if !cambios.is_empty() {
+        let mut ficheros = Vec::with_capacity(cambios.len());
+        for (i, c) in cambios.iter().enumerate() {
+            let f = tmp.0.join(format!("blob-{i}"));
+            std::fs::write(&f, c.contenido.as_bytes()).map_err(|e| {
+                Informe::nuevo(op, Resultado::Error).diagnostico(format!("{}: {e}", f.display()))
+            })?;
+            ficheros.push(texto(op, &f)?);
+        }
+        let mut args = vec!["-w", "--no-filters", "--"];
+        args.extend(ficheros.iter().map(String::as_str));
+        let shas = exigir(git, dir, op, Subcomando::HashObject, &args)?;
+        let shas: Vec<&str> = shas.lines().map(str::trim).collect();
+        if shas.len() != cambios.len() {
+            return Err(Informe::nuevo(op, Resultado::Error)
+                .diagnostico("git hash-object no devolvio un hash por fichero"));
+        }
+        let entradas: Vec<String> = cambios
+            .iter()
+            .zip(&shas)
+            .map(|(c, sha)| format!("100644,{sha},{}", c.ruta))
+            .collect();
+        let mut args = vec!["--add"];
+        for e in &entradas {
+            args.push("--cacheinfo");
+            args.push(e);
+        }
+        con_indice(git, dir, op, Subcomando::UpdateIndex, &args, &indice)?;
+    }
+    let arbol = con_indice(git, dir, op, Subcomando::WriteTree, &[], &indice)?;
+    let arbol = arbol.trim();
+    if let Some(b) = base {
+        let previo = exigir(
+            git,
+            dir,
+            op,
+            Subcomando::RevParse,
+            &["--verify", "--end-of-options", &format!("{b}^{{tree}}")],
+        )?;
+        if previo.trim() == arbol {
+            return Ok(None);
+        }
+    }
+    let mut args = vec![arbol];
+    if let Some(b) = base {
+        args.extend(["-p", b]);
+    }
+    args.extend(["-m", mensaje]);
+    let sha = exigir(git, dir, op, Subcomando::CommitTree, &args)?;
+    Ok(Some(sha.trim().to_owned()))
+}
+
+/// Si `origin` esta configurado. `ls-remote --get-url` no habla con el
+/// remoto: sin `origin`, devuelve el nombre tal cual.
+pub(crate) fn hay_remoto(git: &Git, dir: &Path, op: &str) -> Paso<bool> {
+    let s = correr(
+        git,
+        dir,
+        op,
+        Subcomando::LsRemote,
+        &["--get-url", super::REMOTO],
+    )?;
+    Ok(s.exito() && s.stdout.trim() != super::REMOTO && !s.stdout.trim().is_empty())
+}
+
+#[cfg(test)]
+mod tests {
+    use super::ruta_valida;
+
+    #[test]
+    fn rutas_de_la_rama_de_estado() {
+        for ok in ["ROADMAP.md", "TASK-001/tarea.md", "a b/c.md"] {
+            assert!(ruta_valida(ok), "{ok}");
+        }
+        for malo in [
+            "",
+            "/abs.md",
+            "a//b",
+            "./a",
+            "a/../b",
+            "..",
+            ".git/config",
+            "a\\b",
+            "C:/x",
+            "a,b",
+            "a\nb",
+        ] {
+            assert!(!ruta_valida(malo), "{malo}");
+        }
+    }
+}
diff --git a/crates/tc2/src/git/mod.rs b/crates/tc2/src/git/mod.rs
index 8c0f2bd..f800d6a 100644
--- a/crates/tc2/src/git/mod.rs
+++ b/crates/tc2/src/git/mod.rs
@@ -10,18 +10,23 @@
 //! `origin/main` (ADR F1-3).
 
 mod diff;
+pub(crate) mod estado;
 mod guardar;
 mod puerto;
-mod rama;
-mod remoto;
+pub(crate) mod rama;
+pub(crate) mod remoto;
 mod tag;
 mod worktree;
 
 pub use diff::diff_tarea;
+pub use estado::Cambio;
 pub use guardar::guardar;
-pub use puerto::{ErrorGit, Git, SalidaGit, Subcomando};
+pub use puerto::{ErrorGit, Git, PREFIJO_REFS_PROPIAS, SalidaGit, Subcomando};
 pub use rama::{comprobar_id, crear_rama_y_worktree, localizar_principal, reanudar, validar_base};
-pub use remoto::{ESPERAS_PUSH, actualizar_principal, actualizar_rama, fetch, push};
+pub use remoto::{
+    ESPERAS_PUSH, FalloRemoto, actualizar_principal, actualizar_rama, clasificar_fallo_remoto,
+    fetch, push,
+};
 pub use tag::tag_anotado;
 pub use worktree::quitar_worktree;
 
diff --git a/crates/tc2/src/git/puerto.rs b/crates/tc2/src/git/puerto.rs
index 220669e..f14d8a4 100644
--- a/crates/tc2/src/git/puerto.rs
+++ b/crates/tc2/src/git/puerto.rs
@@ -12,7 +12,8 @@ use std::path::Path;
 use std::process::Command;
 
 /// Subcomandos de `git` que la capa puede lanzar (lista permitida de F1-4 B2b,
-/// reducida a los que usan las operaciones de F2-2).
+/// reducida a los que usan las operaciones de F2-2 y la rama de estado de
+/// F2-3).
 #[derive(Debug, Clone, Copy, PartialEq, Eq)]
 pub enum Subcomando {
     /// `git add`
@@ -21,14 +22,20 @@ pub enum Subcomando {
     Branch,
     /// `git commit`
     Commit,
+    /// `git commit-tree` (crea un objeto commit; no mueve ninguna rama)
+    CommitTree,
     /// `git diff`
     Diff,
     /// `git fetch`
     Fetch,
     /// `git for-each-ref`
     ForEachRef,
+    /// `git hash-object`
+    HashObject,
     /// `git ls-remote`
     LsRemote,
+    /// `git ls-tree`
+    LsTree,
     /// `git merge`
     Merge,
     /// `git merge-base`
@@ -37,16 +44,26 @@ pub enum Subcomando {
     Pull,
     /// `git push`
     Push,
+    /// `git read-tree` (solo con indice temporal)
+    ReadTree,
     /// `git rev-parse`
     RevParse,
+    /// `git show`
+    Show,
     /// `git show-ref`
     ShowRef,
     /// `git status`
     Status,
     /// `git tag`
     Tag,
+    /// `git update-index` (solo con indice temporal)
+    UpdateIndex,
+    /// `git update-ref` (solo bajo `refs/taskcode/`)
+    UpdateRef,
     /// `git worktree`
     Worktree,
+    /// `git write-tree` (solo con indice temporal)
+    WriteTree,
 }
 
 impl Subcomando {
@@ -56,23 +73,41 @@ impl Subcomando {
             Self::Add => "add",
             Self::Branch => "branch",
             Self::Commit => "commit",
+            Self::CommitTree => "commit-tree",
             Self::Diff => "diff",
             Self::Fetch => "fetch",
             Self::ForEachRef => "for-each-ref",
+            Self::HashObject => "hash-object",
             Self::LsRemote => "ls-remote",
+            Self::LsTree => "ls-tree",
             Self::Merge => "merge",
             Self::MergeBase => "merge-base",
             Self::Pull => "pull",
             Self::Push => "push",
+            Self::ReadTree => "read-tree",
             Self::RevParse => "rev-parse",
+            Self::Show => "show",
             Self::ShowRef => "show-ref",
             Self::Status => "status",
             Self::Tag => "tag",
+            Self::UpdateIndex => "update-index",
+            Self::UpdateRef => "update-ref",
             Self::Worktree => "worktree",
+            Self::WriteTree => "write-tree",
         }
     }
+
+    /// Los que trabajan sobre un indice: solo se lanzan con un indice
+    /// temporal ([`Git::ejecutar_con_indice`]), nunca sobre el del repo.
+    pub fn exige_indice_temporal(self) -> bool {
+        matches!(self, Self::ReadTree | Self::UpdateIndex | Self::WriteTree)
+    }
 }
 
+/// Prefijo de las unicas refs que puede escribir `update-ref` (pendientes y
+/// rechazadas de la rama de estado, F2-3). Nunca `refs/heads/*` (F1-4 D3).
+pub const PREFIJO_REFS_PROPIAS: &str = "refs/taskcode/";
+
 /// Lo que devolvio una llamada a `git` que llego a ejecutarse.
 #[derive(Debug, Clone)]
 pub struct SalidaGit {
@@ -183,6 +218,47 @@ impl Git {
         dir: &Path,
         sub: Subcomando,
         args: &[&str],
+    ) -> Result<SalidaGit, ErrorGit> {
+        if sub.exige_indice_temporal() {
+            return Err(ErrorGit::Prohibido(format!(
+                "{} sin indice temporal",
+                sub.nombre()
+            )));
+        }
+        self.lanzar(dir, sub, args, None)
+    }
+
+    /// Como [`Git::ejecutar`], pero con `GIT_INDEX_FILE=indice`: solo para
+    /// `read-tree`, `update-index` y `write-tree`, que asi nunca tocan el
+    /// indice del repo. `indice` tiene que ser una ruta absoluta.
+    pub fn ejecutar_con_indice(
+        &self,
+        dir: &Path,
+        sub: Subcomando,
+        args: &[&str],
+        indice: &Path,
+    ) -> Result<SalidaGit, ErrorGit> {
+        if !sub.exige_indice_temporal() {
+            return Err(ErrorGit::Prohibido(format!(
+                "{} con indice temporal",
+                sub.nombre()
+            )));
+        }
+        if !indice.is_absolute() {
+            return Err(ErrorGit::Prohibido(format!(
+                "{} con indice relativo",
+                sub.nombre()
+            )));
+        }
+        self.lanzar(dir, sub, args, Some(indice))
+    }
+
+    fn lanzar(
+        &self,
+        dir: &Path,
+        sub: Subcomando,
+        args: &[&str],
+        indice: Option<&Path>,
     ) -> Result<SalidaGit, ErrorGit> {
         comprobar_argumentos(sub, args)?;
         let mut cmd = Command::new(&self.binario);
@@ -199,6 +275,9 @@ impl Git {
         for (k, v) in &self.entorno_extra {
             cmd.env(k, v);
         }
+        if let Some(i) = indice {
+            cmd.env("GIT_INDEX_FILE", i);
+        }
         self.registro
             .borrow_mut()
             .push(format!("git {} {}", sub.nombre(), args.join(" ")));
@@ -239,9 +318,28 @@ fn comprobar_argumentos(sub: Subcomando, args: &[&str]) -> Result<(), ErrorGit>
     if sub == Subcomando::Pull && !args.contains(&"--ff-only") {
         return prohibido("sin --ff-only");
     }
+    if sub == Subcomando::UpdateRef && !forma_de_update_ref(args) {
+        return prohibido(&args.join(" "));
+    }
     Ok(())
 }
 
+/// `update-ref` solo con `<ref> <nuevo> [<viejo>]` o `-d <ref> <viejo>`, y la
+/// ref bajo [`PREFIJO_REFS_PROPIAS`]: nunca una rama, nunca `--stdin`.
+fn forma_de_update_ref(args: &[&str]) -> bool {
+    let propia = |r: &str| {
+        r.strip_prefix(PREFIJO_REFS_PROPIAS)
+            .is_some_and(|resto| !resto.is_empty() && !resto.contains(".."))
+    };
+    let valor = |v: &str| !v.is_empty() && !v.starts_with('-');
+    match args {
+        ["-d", r, viejo] => propia(r) && valor(viejo),
+        [r, nuevo] => propia(r) && valor(nuevo),
+        [r, nuevo, viejo] => propia(r) && valor(nuevo) && valor(viejo),
+        _ => false,
+    }
+}
+
 fn es_prohibido(sub: Subcomando, a: &str) -> bool {
     if a.starts_with("--force") {
         return true;
@@ -262,6 +360,7 @@ fn es_prohibido(sub: Subcomando, a: &str) -> bool {
         Subcomando::Tag => matches!(a, "-d" | "-f" | "--delete"),
         Subcomando::Merge => a == "--abort",
         Subcomando::Commit => a == "--amend",
+        Subcomando::ReadTree => matches!(a, "-u" | "-m" | "--reset") || a.starts_with("--prefix"),
         _ => false,
     }
 }
@@ -316,6 +415,24 @@ mod tests {
             (Subcomando::Merge, &["--abort"]),
             (Subcomando::Commit, &["--amend", "-m", "x"]),
             (Subcomando::Pull, &["origin", "develop"]),
+            (Subcomando::UpdateRef, &["refs/heads/develop", "abc"]),
+            (Subcomando::UpdateRef, &["refs/remotes/origin/x", "abc"]),
+            (Subcomando::UpdateRef, &["-d", "refs/heads/develop", "abc"]),
+            (Subcomando::UpdateRef, &["--stdin"]),
+            (Subcomando::UpdateRef, &["-d", "refs/taskcode/p/1"]),
+            (Subcomando::UpdateRef, &["refs/taskcode/", "abc"]),
+            (Subcomando::UpdateRef, &["refs/taskcode/../heads/x", "abc"]),
+            (
+                Subcomando::UpdateRef,
+                &["--no-deref", "refs/taskcode/p/1", "abc"],
+            ),
+            (
+                Subcomando::UpdateRef,
+                &["refs/taskcode/p/1", "abc", "def", "x"],
+            ),
+            (Subcomando::ReadTree, &["abc"]),
+            (Subcomando::UpdateIndex, &["--add", "a"]),
+            (Subcomando::WriteTree, &[]),
         ];
         for (sub, args) in casos {
             let r = git.ejecutar(Path::new("."), *sub, args);
@@ -348,6 +465,44 @@ mod tests {
                 "{sub:?} {args:?}: {r:?}"
             );
         }
+        // Con indice temporal solo los tres de indice, con ruta absoluta y
+        // sin las formas que tocarian el worktree.
+        let indice = std::env::temp_dir().join("indice-de-prueba");
+        let r = git.ejecutar_con_indice(Path::new("."), Subcomando::Commit, &[], &indice);
+        assert!(matches!(r, Err(ErrorGit::Prohibido(_))), "{r:?}");
+        let r = git.ejecutar_con_indice(
+            Path::new("."),
+            Subcomando::WriteTree,
+            &[],
+            Path::new("relativo"),
+        );
+        assert!(matches!(r, Err(ErrorGit::Prohibido(_))), "{r:?}");
+        for malo in ["-u", "-m", "--reset", "--prefix=x/"] {
+            let r = git.ejecutar_con_indice(
+                Path::new("."),
+                Subcomando::ReadTree,
+                &[malo, "abc"],
+                &indice,
+            );
+            assert!(matches!(r, Err(ErrorGit::Prohibido(_))), "{malo}: {r:?}");
+        }
+        let r = git.ejecutar_con_indice(
+            Path::new("."),
+            Subcomando::UpdateIndex,
+            &["--force-remove", "a"],
+            &indice,
+        );
+        assert!(matches!(r, Err(ErrorGit::Prohibido(_))), "{r:?}");
+        let r = git.ejecutar_con_indice(Path::new("."), Subcomando::ReadTree, &["abc"], &indice);
+        assert!(matches!(r, Err(ErrorGit::Lanzar(_))), "{r:?}");
+        for ok in [
+            &["refs/taskcode/pendientes/000001", "abc"][..],
+            &["refs/taskcode/pendientes/000001", "abc", "0000"][..],
+            &["-d", "refs/taskcode/pendientes/000001", "abc"][..],
+        ] {
+            let r = git.ejecutar(Path::new("."), Subcomando::UpdateRef, ok);
+            assert!(matches!(r, Err(ErrorGit::Lanzar(_))), "{ok:?}: {r:?}");
+        }
         // Tras `--` en push no hay rutas: se sigue mirando.
         let r = git.ejecutar(Path::new("."), Subcomando::Push, &["origin", "--", "+r:r"]);
         assert!(matches!(r, Err(ErrorGit::Prohibido(_))));
diff --git a/crates/tc2/src/git/rama.rs b/crates/tc2/src/git/rama.rs
index c8f6d99..4ad5812 100644
--- a/crates/tc2/src/git/rama.rs
+++ b/crates/tc2/src/git/rama.rs
@@ -128,6 +128,11 @@ fn ruta_texto(op: &str, p: &Path) -> Paso<String> {
 /// Que `git status` del principal no muestre los worktrees de tarea:
 /// `/.taskcode/worktrees/` en `info/exclude` del repo comun (no versionado).
 fn excluir_worktrees(git: &Git, principal: &Path, op: &str) -> Paso<()> {
+    excluir(git, principal, op, &format!("/{CARPETA_WORKTREES}/"))
+}
+
+/// Anade `linea` a `info/exclude` del repo comun si no esta (no versionado).
+pub(crate) fn excluir(git: &Git, principal: &Path, op: &str, linea: &str) -> Paso<()> {
     let comun = exigir(
         git,
         principal,
@@ -138,7 +143,6 @@ fn excluir_worktrees(git: &Git, principal: &Path, op: &str) -> Paso<()> {
     let comun = principal.join(comun.trim());
     let info = comun.join("info");
     let fichero = info.join("exclude");
-    let linea = format!("/{CARPETA_WORKTREES}/");
     let error = |e: std::io::Error| {
         Informe::nuevo(op, Resultado::Error).diagnostico(format!("{}: {e}", fichero.display()))
     };
@@ -157,7 +161,7 @@ fn excluir_worktrees(git: &Git, principal: &Path, op: &str) -> Paso<()> {
     if !nuevo.is_empty() && !nuevo.ends_with('\n') {
         nuevo.push('\n');
     }
-    nuevo.push_str(&linea);
+    nuevo.push_str(linea);
     nuevo.push('\n');
     std::fs::write(&fichero, nuevo).map_err(error)
 }
diff --git a/crates/tc2/src/git/remoto.rs b/crates/tc2/src/git/remoto.rs
index 96a07b1..54fcf2f 100644
--- a/crates/tc2/src/git/remoto.rs
+++ b/crates/tc2/src/git/remoto.rs
@@ -34,7 +34,79 @@ fn sinred(op: &str) -> Informe {
         .diagnostico(format!("{op}: no se pudo leer {REMOTO}; reintenta con red"))
 }
 
-/// Trae el remoto. Nada nuevo → `sin-cambios`; sin remoto → `sinred`.
+/// Por que fallo una llamada que habla con el remoto (F2-3: responde las
+/// preguntas abiertas de F1-2 sobre credenciales y rechazos del remoto).
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum FalloRemoto {
+    /// No hubo respuesta: red caida, DNS, servidor parado. Se reintenta.
+    SinRed,
+    /// El remoto respondio pero no acepta estas credenciales o permisos.
+    Credenciales,
+    /// La URL de `origin` no es un repositorio.
+    RemotoNoExiste,
+    /// El remoto existe pero no tiene la rama pedida.
+    RamaInexistente,
+}
+
+impl FalloRemoto {
+    /// Valor de `motivo=` en la linea de contrato.
+    pub fn motivo(self) -> &'static str {
+        match self {
+            Self::SinRed => "sin-red",
+            Self::Credenciales => "credenciales",
+            Self::RemotoNoExiste => "remoto-no-existe",
+            Self::RamaInexistente => "rama-no-existe",
+        }
+    }
+}
+
+/// Clasifica el stderr de `fetch`, `push` o `ls-remote` (en `LC_ALL=C`, que
+/// fija el puerto). Lo que no se reconoce es `SinRed`: manda reintentar, que
+/// es lo prudente ante un texto desconocido.
+pub fn clasificar_fallo_remoto(stderr: &str) -> FalloRemoto {
+    let t = stderr.to_ascii_lowercase();
+    let alguno = |patrones: &[&str]| patrones.iter().any(|p| t.contains(p));
+    if alguno(&["couldn't find remote ref", "could not find remote ref"]) {
+        FalloRemoto::RamaInexistente
+    } else if alguno(&[
+        "authentication failed",
+        "permission denied",
+        "could not read username",
+        "could not read password",
+        "terminal prompts disabled",
+        "returned error: 401",
+        "returned error: 403",
+        "access denied",
+        "permission to ",
+    ]) {
+        FalloRemoto::Credenciales
+    } else if alguno(&[
+        "does not appear to be a git repository",
+        "repository not found",
+        "not a git repository",
+        "returned error: 404",
+    ]) {
+        FalloRemoto::RemotoNoExiste
+    } else {
+        FalloRemoto::SinRed
+    }
+}
+
+/// Informe de un fallo del remoto que no es de red: `denegada` (8).
+pub(crate) fn denegada(op: &str, fallo: FalloRemoto, detalle: &str) -> Informe {
+    let que_hacer = match fallo {
+        FalloRemoto::Credenciales => "revisa tus credenciales o permisos en el remoto",
+        FalloRemoto::RemotoNoExiste => "revisa la URL de origin",
+        _ => "revisa las reglas del remoto o avisa a la persona",
+    };
+    Informe::nuevo(op, Resultado::Denegada)
+        .con("remoto", REMOTO)
+        .con("motivo", fallo.motivo())
+        .diagnostico(format!("{op}: {detalle}; {que_hacer}"))
+}
+
+/// Trae el remoto. Nada nuevo → `sin-cambios`; sin red → `sinred`; remoto
+/// inexistente o sin permiso → `denegada`.
 pub fn fetch(git: &Git, dir: &Path) -> Informe {
     traer(git, dir, "fetch").unwrap_or_else(|i| i)
 }
@@ -43,7 +115,10 @@ fn traer(git: &Git, dir: &Path, op: &str) -> Paso<Informe> {
     let antes = refs_remotas(git, dir, op)?;
     let s = correr(git, dir, op, Subcomando::Fetch, &["-q", REMOTO])?;
     if !s.exito() {
-        return Err(sinred(op));
+        return Err(match clasificar_fallo_remoto(&s.stderr) {
+            FalloRemoto::SinRed | FalloRemoto::RamaInexistente => sinred(op),
+            otro => denegada(op, otro, &s.primera_linea_error()),
+        });
     }
     let despues = refs_remotas(git, dir, op)?;
     let r = if antes == despues {
@@ -59,9 +134,10 @@ enum Intento {
     Publicado,
     AlDia,
     NoFastForward,
-    /// Fallo con su resumen; `true` si el remoto respondio y lo rechazo
-    /// (`[remote rejected]`: un hook o una regla), no un fallo de red.
-    Fallo(String, bool),
+    /// Fallo con su resumen y, si el remoto respondio denegandolo
+    /// (`[remote rejected]` por un hook o una regla, credenciales, remoto
+    /// inexistente), el motivo: eso no se arregla reintentando.
+    Fallo(String, Option<&'static str>),
 }
 
 fn intento_push(git: &Git, dir: &Path, op: &str, refspec: &str) -> Paso<Intento> {
@@ -83,46 +159,92 @@ fn intento_push(git: &Git, dir: &Path, op: &str, refspec: &str) -> Paso<Intento>
         Some('=') if s.exito() => Intento::AlDia,
         Some(' ' | '*') if s.exito() => Intento::Publicado,
         Some('!') if resumen.starts_with("[rejected]") => Intento::NoFastForward,
-        _ if resumen.is_empty() => Intento::Fallo(s.primera_linea_error(), false),
-        _ => Intento::Fallo(resumen.to_owned(), resumen.starts_with("[remote rejected]")),
+        _ if resumen.starts_with("[remote rejected]") && es_contencion(resumen) => {
+            Intento::NoFastForward
+        }
+        _ if resumen.starts_with("[remote rejected]") => {
+            Intento::Fallo(resumen.to_owned(), Some("remoto-rechaza"))
+        }
+        _ => {
+            let detalle = if resumen.is_empty() {
+                s.primera_linea_error()
+            } else {
+                resumen.to_owned()
+            };
+            match clasificar_fallo_remoto(&s.stderr) {
+                FalloRemoto::SinRed | FalloRemoto::RamaInexistente => Intento::Fallo(detalle, None),
+                otro => Intento::Fallo(detalle, Some(otro.motivo())),
+            }
+        }
     })
 }
 
-/// Publica `refspec` con reintentos ante fallos transitorios. Un
-/// non-fast-forward no se reintenta. Agotados los intentos: `agotada` si el
-/// remoto respondio rechazando (hook o regla: con red no se arregla) y
-/// `sinred` si no. Devuelve el resultado, los intentos y el ultimo fallo.
+/// `[remote rejected]` por contencion: otro push actualizo la misma ref a la
+/// vez y el remoto no pudo tomar su cerrojo (visto en F2-3 con dos procesos
+/// contra el mismo remoto). Es lo mismo que un non-fast-forward: la ref se
+/// movio; no es una denegacion.
+fn es_contencion(resumen: &str) -> bool {
+    let t = resumen.to_ascii_lowercase();
+    [
+        "cannot lock ref",
+        "failed to lock",
+        "unable to lock",
+        "failed to update ref",
+        "but expected",
+        "incorrect old value",
+    ]
+    .iter()
+    .any(|p| t.contains(p))
+}
+
+/// Como termino [`publicar`].
+pub(crate) struct Publicacion {
+    /// `Ok`, `SinCambios`, `Rechazada` (non-fast-forward), `Denegada` o
+    /// `Sinred`.
+    pub resultado: Resultado,
+    /// Intentos hechos.
+    pub intentos: usize,
+    /// Ultimo fallo, para el diagnostico.
+    pub ultimo: String,
+    /// Motivo de una `Denegada`.
+    pub motivo: &'static str,
+}
+
+/// Publica `refspec` con reintentos ante fallos transitorios (sin red). Un
+/// non-fast-forward (`rechazada`) y una denegacion del remoto (`denegada`:
+/// hook, regla, credenciales; reintentar no la arregla) no se reintentan.
+/// Agotados los intentos sin red: `sinred`.
 pub(crate) fn publicar(
     git: &Git,
     dir: &Path,
     op: &str,
     refspec: &str,
     esperas: &[Duration],
-) -> Paso<(Resultado, usize, String)> {
+) -> Paso<Publicacion> {
+    let fin = |resultado, intentos, ultimo: String, motivo| Publicacion {
+        resultado,
+        intentos,
+        ultimo,
+        motivo,
+    };
     let mut ultimo = String::new();
-    let mut rechazo_remoto = false;
     for intento in 0..=esperas.len() {
         match intento_push(git, dir, op, refspec)? {
-            Intento::AlDia => return Ok((Resultado::SinCambios, intento + 1, String::new())),
-            Intento::Publicado => return Ok((Resultado::Ok, intento + 1, String::new())),
+            Intento::AlDia => return Ok(fin(Resultado::SinCambios, intento + 1, ultimo, "")),
+            Intento::Publicado => return Ok(fin(Resultado::Ok, intento + 1, ultimo, "")),
             Intento::NoFastForward => {
-                return Ok((Resultado::Rechazada, intento + 1, String::new()));
+                return Ok(fin(Resultado::Rechazada, intento + 1, ultimo, ""));
             }
-            Intento::Fallo(m, remoto) => {
-                ultimo = m;
-                rechazo_remoto = remoto;
+            Intento::Fallo(m, Some(motivo)) => {
+                return Ok(fin(Resultado::Denegada, intento + 1, m, motivo));
             }
+            Intento::Fallo(m, None) => ultimo = m,
         }
         if let Some(e) = esperas.get(intento) {
             std::thread::sleep(*e);
         }
     }
-    let r = if rechazo_remoto {
-        Resultado::Agotada
-    } else {
-        Resultado::Sinred
-    };
-    Ok((r, esperas.len() + 1, ultimo))
+    Ok(fin(Resultado::Sinred, esperas.len() + 1, ultimo, ""))
 }
 
 /// Publica la rama actual en `origin` con el mismo nombre, sin force.
@@ -135,7 +257,8 @@ fn push_paso(git: &Git, dir: &Path, op: &str, esperas: &[Duration]) -> Paso<Info
     let rama = rama_de_tarea(git, dir, op)?;
     let head = commit_de(git, dir, op, "HEAD")?.unwrap_or_default();
     let refspec = format!("refs/heads/{rama}:refs/heads/{rama}");
-    let (r, intentos, ultimo) = publicar(git, dir, op, &refspec, esperas)?;
+    let p = publicar(git, dir, op, &refspec, esperas)?;
+    let (r, intentos, ultimo) = (p.resultado, p.intentos, p.ultimo);
     let i = Informe::nuevo(op, r).con("rama", rama.as_str());
     Ok(match r {
         Resultado::Ok => i
@@ -145,11 +268,11 @@ fn push_paso(git: &Git, dir: &Path, op: &str, esperas: &[Duration]) -> Paso<Info
         Resultado::Rechazada => i
             .con("motivo", "no-fast-forward")
             .diagnostico("lanza actualizar-rama"),
-        Resultado::Agotada => i
-            .con("motivo", "remoto-rechaza")
+        Resultado::Denegada => i
+            .con("motivo", p.motivo)
             .con("intentos", intentos.to_string())
             .diagnostico(format!(
-                "push: {ultimo}; el remoto rechaza el push: revisa sus reglas o avisa a la persona"
+                "push: {ultimo}; el remoto deniega el push: revisa sus reglas, tus permisos o avisa a la persona"
             )),
         _ => i
             .con("intentos", intentos.to_string())
@@ -274,7 +397,8 @@ fn actualizar(git: &Git, dir: &Path, esperas: &[Duration]) -> Paso<Informe> {
     let mut publicada = false;
     if hay_remota && !es_ancestro(git, dir, OP, "HEAD", &remota)? {
         let refspec = format!("refs/heads/{rama}:refs/heads/{rama}");
-        let (r, intentos, ultimo) = publicar(git, dir, OP, &refspec, esperas)?;
+        let p = publicar(git, dir, OP, &refspec, esperas)?;
+        let (r, intentos, ultimo) = (p.resultado, p.intentos, p.ultimo);
         match r {
             Resultado::Ok | Resultado::SinCambios => publicada = true,
             Resultado::Rechazada => {
@@ -286,17 +410,18 @@ fn actualizar(git: &Git, dir: &Path, esperas: &[Duration]) -> Paso<Informe> {
                     ));
             }
             otro => {
-                let que_hacer = if otro == Resultado::Agotada {
-                    "el remoto rechaza el push: revisa sus reglas o avisa a la persona"
+                let que_hacer = if otro == Resultado::Denegada {
+                    "el remoto deniega el push: revisa sus reglas, tus permisos o avisa a la persona"
                 } else {
                     "reintenta con red"
                 };
-                return Ok(Informe::nuevo(OP, otro)
-                    .con("rama", rama.as_str())
-                    .con("intentos", intentos.to_string())
-                    .diagnostico(format!(
-                        "push: {ultimo}; la rama esta actualizada en local; {que_hacer}"
-                    )));
+                let mut i = Informe::nuevo(OP, otro).con("rama", rama.as_str());
+                if otro == Resultado::Denegada {
+                    i = i.con("motivo", p.motivo);
+                }
+                return Ok(i.con("intentos", intentos.to_string()).diagnostico(format!(
+                    "push: {ultimo}; la rama esta actualizada en local; {que_hacer}"
+                )));
             }
         }
     }
@@ -358,3 +483,68 @@ fn actualizar_principal_paso(git: &Git, dir: &Path, rama: &str) -> Paso<Informe>
         .con("rama", rama)
         .con("commit", corto(&r)))
 }
+
+#[cfg(test)]
+mod tests {
+    use super::{FalloRemoto, clasificar_fallo_remoto, es_contencion};
+
+    #[test]
+    fn clasifica_los_fallos_del_remoto() {
+        let casos = [
+            (
+                "fatal: couldn't find remote ref refs/heads/taskcode/tareas",
+                FalloRemoto::RamaInexistente,
+            ),
+            (
+                "remote: Invalid username or password.\nfatal: Authentication failed for 'https://github.com/a/b.git/'",
+                FalloRemoto::Credenciales,
+            ),
+            (
+                "fatal: could not read Username for 'https://github.com': terminal prompts disabled",
+                FalloRemoto::Credenciales,
+            ),
+            (
+                "git@github.com: Permission denied (publickey).\nfatal: Could not read from remote repository.",
+                FalloRemoto::Credenciales,
+            ),
+            (
+                "remote: Permission to a/b.git denied to c.\nfatal: unable to access 'https://github.com/a/b.git/': The requested URL returned error: 403",
+                FalloRemoto::Credenciales,
+            ),
+            (
+                "fatal: '/x/no-existe' does not appear to be a git repository\nfatal: Could not read from remote repository.",
+                FalloRemoto::RemotoNoExiste,
+            ),
+            (
+                "remote: Repository not found.\nfatal: repository 'https://github.com/a/b.git/' not found",
+                FalloRemoto::RemotoNoExiste,
+            ),
+            (
+                "fatal: unable to access 'http://127.0.0.1:1/x.git/': Failed to connect to 127.0.0.1 port 1",
+                FalloRemoto::SinRed,
+            ),
+            (
+                "ssh: Could not resolve hostname github.com: Name or service not known\nfatal: Could not read from remote repository.",
+                FalloRemoto::SinRed,
+            ),
+            ("algo que no se reconoce", FalloRemoto::SinRed),
+        ];
+        for (stderr, esperado) in casos {
+            assert_eq!(clasificar_fallo_remoto(stderr), esperado, "{stderr}");
+        }
+    }
+
+    #[test]
+    fn contencion_frente_a_denegacion() {
+        assert!(es_contencion(
+            "[remote rejected] (cannot lock ref 'refs/heads/taskcode/tareas': is at 1a but expected 2b)"
+        ));
+        assert!(es_contencion("[remote rejected] (failed to update ref)"));
+        assert!(!es_contencion(
+            "[remote rejected] (pre-receive hook declined)"
+        ));
+        assert!(!es_contencion(
+            "[remote rejected] (protected branch hook declined)"
+        ));
+    }
+}
diff --git a/crates/tc2/src/git/tag.rs b/crates/tc2/src/git/tag.rs
index 784cc84..f5d03bd 100644
--- a/crates/tc2/src/git/tag.rs
+++ b/crates/tc2/src/git/tag.rs
@@ -75,13 +75,22 @@ fn tag_paso(
     }
     if remoto.is_none() {
         let refspec = format!("refs/tags/{tag}:refs/tags/{tag}");
-        let (r, intentos, ultimo) = publicar(git, dir, OP, &refspec, esperas)?;
-        if !r.es_exito() {
-            return Ok(Informe::nuevo(OP, r)
-                .con("tag", tag)
-                .con("intentos", intentos.to_string())
+        let p = publicar(git, dir, OP, &refspec, esperas)?;
+        if !p.resultado.es_exito() {
+            let que_hacer = if p.resultado == Resultado::Denegada {
+                "el remoto lo deniega: revisa sus reglas o tus permisos"
+            } else {
+                "relanza con red"
+            };
+            let mut i = Informe::nuevo(OP, p.resultado).con("tag", tag);
+            if p.resultado == Resultado::Denegada {
+                i = i.con("motivo", p.motivo);
+            }
+            return Ok(i
+                .con("intentos", p.intentos.to_string())
                 .diagnostico(format!(
-                    "no se pudo publicar el tag: {ultimo}; el tag queda en local, relanza con red"
+                    "no se pudo publicar el tag: {}; el tag queda en local, {que_hacer}",
+                    p.ultimo
                 )));
         }
     }
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 6fa118d..e0b2f50 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -1,10 +1,12 @@
 //! Nucleo del CLI `tc2` de TaskCodeTwo.
 //!
-//! Modulos: [`contrato`] (salida de toda operacion, ADR F1-4) y [`git`]
-//! (capa de operaciones Git deterministas, F2-2). Las capas que vienen
-//! (modelo de tarea de F2-5...) entran como modulos de esta biblioteca.
+//! Modulos: [`contrato`] (salida de toda operacion, ADR F1-4), [`git`]
+//! (capa de operaciones Git deterministas, F2-2) y [`estado`] (rama de
+//! estado `taskcode/tareas`, F2-3). Las capas que vienen (modelo de tarea de
+//! F2-5...) entran como modulos de esta biblioteca.
 
 pub mod contrato;
+pub mod estado;
 pub mod git;
 
 use contrato::{Informe, Resultado};
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-018/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-018/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-018/tarea.md                                    | 14 ++++++++++----
 3 files changed, 10 insertions(+), 4 deletions(-)
````

## Tests del diff (6 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/comun/mod.rs" ":(literal)crates/tc2/tests/comun/ops.rs" ":(literal)crates/tc2/tests/estado_casos.rs" ":(literal)crates/tc2/tests/estado_concurrencia.rs" ":(literal)crates/tc2/tests/git_casos_limite.rs" ":(literal)crates/tc2/tests/git_push.rs"
````

````
crates/tc2/tests/comun/mod.rs           |  17 +++++++-
 crates/tc2/tests/comun/ops.rs           |  92 +++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/estado_casos.rs        | 347 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/estado_concurrencia.rs | 192 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/git_casos_limite.rs    |   9 ++--
 crates/tc2/tests/git_push.rs            |  52 +++++++++++++++++------
 6 files changed, 692 insertions(+), 17 deletions(-)
````
