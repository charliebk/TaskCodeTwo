# Peticion de revision — TASK-084 (ronda 1)

- Tarea: TASK-084 — F3-12 Progreso en vivo: fases que faltan y tiempo restante
- Rama revisada: feature/task-084-f3-12-progreso-en-vivo-fases-que-faltan
- Rama base: develop
- Commit revisado (HEAD): efee1212394fb2f28018b63e4dc78f56a39b4f6e
- Fecha: 2026-11-18
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-084 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-084/tarea.md
- tareas/03-en-revision/TASK-084/planificacion/
- crates/tc2/src/estado/frescura.rs
- crates/tc2/src/huella.rs
- crates/tc2/src/i18n/catalogo/mod.rs
- crates/tc2/src/i18n/catalogo/progreso.rs
- crates/tc2/src/i18n/catalogo/tablero.rs
- crates/tc2/src/identidad.rs
- crates/tc2/src/init/inventario.rs
- crates/tc2/src/init/mod.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/main.rs
- crates/tc2/src/progreso/calculo.rs
- crates/tc2/src/progreso/fichero.rs
- crates/tc2/src/progreso/linea.rs
- crates/tc2/src/progreso/mod.rs
- crates/tc2/src/tablero.rs
- crates/tc2/src/toma/mod.rs
- crates/tc2/src/toma/operacion.rs
- crates/tc2/tests/progreso.rs

## Commits a revisar (git log develop..HEAD)

````
efee121 docs(TASK-084): autorrevision con rust-wasm-reviewer y code-quality-reviewer
37f97c1 docs(TASK-084): adenda de F1-17 y status line en instalar.md
47f812e feat(TASK-084): tc2 status --seguir y propuesta de la status line en init
e058b03 feat(TASK-084): las transiciones refrescan el progreso en vivo
0371de9 feat(TASK-084): linea de progreso y tc2 estado --stdin sin git
e9cf891 feat(TASK-084): fichero de progreso y calculo puro
29fee1d refactor(TASK-084): escribir_local en la huella con reintento del rename
a9a5c7e chore(TASK-084): tarea en curso
````

## Diff de tu dominio (18 fichero(s) de 20; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/estado/frescura.rs b/crates/tc2/src/estado/frescura.rs
index fde31c3..2bab2e8 100644
--- a/crates/tc2/src/estado/frescura.rs
+++ b/crates/tc2/src/estado/frescura.rs
@@ -186,18 +186,10 @@ impl Marca {
         t.parse().ok()
     }
 
-    /// Guarda `ahora`. Escribe un temporal y lo renombra.
+    /// Guarda `ahora`. Escribe un temporal y lo renombra
+    /// ([`crate::huella::escribir_local`]).
     pub fn registrar(&self, ahora: u64) -> std::io::Result<()> {
-        if let Some(padre) = self.ruta.parent() {
-            std::fs::create_dir_all(padre)?;
-        }
-        let tmp = self
-            .ruta
-            .with_extension(format!("tmp-{}", std::process::id()));
-        std::fs::write(&tmp, format!("{ahora}\n"))?;
-        std::fs::rename(&tmp, &self.ruta).inspect_err(|_| {
-            let _ = std::fs::remove_file(&tmp);
-        })
+        crate::huella::escribir_local(&self.ruta, format!("{ahora}\n").as_bytes())
     }
 }
 
diff --git a/crates/tc2/src/huella.rs b/crates/tc2/src/huella.rs
index ef98994..7eb5b3b 100644
--- a/crates/tc2/src/huella.rs
+++ b/crates/tc2/src/huella.rs
@@ -9,9 +9,10 @@
 //! - la carpeta de los worktrees de tarea, [`CARPETA_WORKTREES`];
 //! - lo que se guarda en la carpeta comun de git (`.git` del principal),
 //!   bajo [`CARPETA_COMUN`]: el cerrojo, la cache de identidad, la marca de
-//!   la ultima sincronizacion y los indices temporales de la transaccion.
-//!   `tc2 desinstalar` borra la carpeta entera, asi que un fichero nuevo
-//!   aqui queda reconocido sin tocarlo.
+//!   la ultima sincronizacion, los indices temporales de la transaccion y el
+//!   progreso en vivo ([`CARPETA_PROGRESO`]). `tc2 desinstalar` borra la
+//!   carpeta entera, asi que un fichero nuevo aqui queda reconocido sin
+//!   tocarlo. Sus ficheros se escriben con [`escribir_local`].
 //!
 //! La configuracion del proyecto (`.taskcode/config.yml`, `personas.yml` y
 //! el vocabulario) no esta aqui: F1-12 D4a la deja compartida con la 0.16.
@@ -62,6 +63,59 @@ pub const FICHERO_SINCRONIZACION: &str = "sincronizacion";
 /// Indices temporales de la transaccion, en [`CARPETA_COMUN`].
 pub const CARPETA_TEMPORALES: &str = "estado-tmp";
 
+/// Progreso en vivo de cada tarea en curso del clon, en [`CARPETA_COMUN`]
+/// (F3-12): `TASK-NNN.json` y el anterior, `TASK-NNN.anterior.json`. Es
+/// derivado y sin versionar; lo escriben las transiciones y lo leen la
+/// status line y `tc2 status --seguir` (adenda de TASK-084 en F1-17).
+pub const CARPETA_PROGRESO: &str = "progreso";
+
+/// Esperas antes de repetir el `rename` de [`escribir_local`]: en Windows
+/// falla mientras un lector tiene el destino abierto sin `FILE_SHARE_DELETE`
+/// (Python, Node o un antivirus; L2 de F1-17, punto 7). Unos 300 ms en total.
+const ESPERAS_RENAME: [std::time::Duration; 5] = [
+    std::time::Duration::from_millis(10),
+    std::time::Duration::from_millis(20),
+    std::time::Duration::from_millis(40),
+    std::time::Duration::from_millis(80),
+    std::time::Duration::from_millis(150),
+];
+
+/// Escribe `ruta` de una vez, para los ficheros locales de [`CARPETA_COMUN`]
+/// (la marca de sincronizacion, la cache de identidad y el progreso): crea la
+/// carpeta si falta, escribe un temporal `<nombre>.tmp-<pid>` al lado y lo
+/// renombra encima, de modo que quien lee no ve nunca un fichero a medias.
+/// Si el `rename` falla, se repite tras cada espera de [`ESPERAS_RENAME`]; si
+/// sigue fallando, se borra el temporal y se devuelve el ultimo error.
+pub fn escribir_local(ruta: &Path, contenido: &[u8]) -> io::Result<()> {
+    escribir_local_con(ruta, contenido, &ESPERAS_RENAME)
+}
+
+fn escribir_local_con(
+    ruta: &Path,
+    contenido: &[u8],
+    esperas: &[std::time::Duration],
+) -> io::Result<()> {
+    if let Some(padre) = ruta.parent() {
+        std::fs::create_dir_all(padre)?;
+    }
+    let mut nombre = ruta.file_name().unwrap_or_default().to_os_string();
+    nombre.push(format!(".tmp-{}", std::process::id()));
+    let tmp = ruta.with_file_name(nombre);
+    std::fs::write(&tmp, contenido)?;
+    let mut r = std::fs::rename(&tmp, ruta);
+    for e in esperas {
+        if r.is_ok() {
+            break;
+        }
+        std::thread::sleep(*e);
+        r = std::fs::rename(&tmp, ruta);
+    }
+    if r.is_err() {
+        let _ = std::fs::remove_file(&tmp);
+    }
+    r
+}
+
 /// Workflow de CI que deja `tc2 init` (F3-1). Hasta que F5-1 entregue el
 /// reutilizable, `init` solo deja la ranura (`pendiente motivo=sin-plantilla`).
 pub const RUTA_WORKFLOW: &str = concat!(".github/workflows/", nombre!(), ".yml");
@@ -118,6 +172,7 @@ mod tests {
             FICHERO_IDENTIDAD,
             FICHERO_SINCRONIZACION,
             CARPETA_TEMPORALES,
+            CARPETA_PROGRESO,
         ];
         for (i, a) in ficheros.iter().enumerate() {
             assert!(ficheros[i + 1..].iter().all(|b| a != b), "{a} repetido");
@@ -170,4 +225,60 @@ mod tests {
         let _ = std::fs::remove_dir_all(&comun);
         Ok(())
     }
+
+    fn carpeta(nombre: &str) -> PathBuf {
+        let d = std::env::temp_dir()
+            .join("tc2-tests")
+            .join(format!("huella-{nombre}-{}", std::process::id()));
+        let _ = std::fs::remove_dir_all(&d);
+        d
+    }
+
+    /// `escribir_local` crea la carpeta, sustituye el contenido y no deja
+    /// temporales sueltos.
+    #[test]
+    fn escribir_local_de_una_vez_y_sin_temporales() -> io::Result<()> {
+        let d = carpeta("escribir");
+        let ruta = d.join("sub").join("f.json");
+        escribir_local(&ruta, b"uno")?;
+        escribir_local(&ruta, b"dos")?;
+        assert_eq!(std::fs::read(&ruta)?, b"dos");
+        let n = std::fs::read_dir(d.join("sub"))?.count();
+        assert_eq!(n, 1);
+        let _ = std::fs::remove_dir_all(&d);
+        Ok(())
+    }
+
+    /// L2 de F1-17 (punto 7): en Windows, con un lector abierto sin
+    /// `FILE_SHARE_DELETE` (como Python o Node), el `rename` falla; con el
+    /// reintento entra en cuanto el lector cierra. Sin reintento, falla.
+    #[cfg(windows)]
+    #[test]
+    fn escribir_local_reintenta_con_un_lector_abierto() -> io::Result<()> {
+        use std::os::windows::fs::OpenOptionsExt;
+        const FILE_SHARE_READ: u32 = 1;
+        let d = carpeta("lector");
+        let ruta = d.join("f.json");
+        escribir_local(&ruta, b"viejo")?;
+        let abrir = || {
+            std::fs::OpenOptions::new()
+                .read(true)
+                .share_mode(FILE_SHARE_READ)
+                .open(&ruta)
+        };
+        // Sin reintento, con el lector abierto: falla y no deja temporal.
+        let lector = abrir()?;
+        assert!(escribir_local_con(&ruta, b"nuevo", &[]).is_err());
+        assert_eq!(std::fs::read_dir(&d)?.count(), 1);
+        // Con reintento: el lector cierra a los 30 ms y la escritura entra.
+        let hilo = std::thread::spawn(move || {
+            std::thread::sleep(std::time::Duration::from_millis(30));
+            drop(lector);
+        });
+        escribir_local(&ruta, b"nuevo")?;
+        let _ = hilo.join();
+        assert_eq!(std::fs::read(&ruta)?, b"nuevo");
+        let _ = std::fs::remove_dir_all(&d);
+        Ok(())
+    }
 }
diff --git a/crates/tc2/src/i18n/catalogo/mod.rs b/crates/tc2/src/i18n/catalogo/mod.rs
index bb90661..30d055a 100644
--- a/crates/tc2/src/i18n/catalogo/mod.rs
+++ b/crates/tc2/src/i18n/catalogo/mod.rs
@@ -15,6 +15,7 @@ pub mod identidad;
 pub mod importar;
 pub mod init;
 pub mod plataforma;
+pub mod progreso;
 pub mod roadmap;
 pub mod siguiente;
 pub mod tablero;
@@ -39,6 +40,7 @@ pub fn todos() -> impl Iterator<Item = &'static Texto> {
         importar::TODOS,
         init::TODOS,
         plataforma::TODOS,
+        progreso::TODOS,
         roadmap::TODOS,
         siguiente::TODOS,
         tablero::TODOS,
diff --git a/crates/tc2/src/i18n/catalogo/progreso.rs b/crates/tc2/src/i18n/catalogo/progreso.rs
new file mode 100644
index 0000000..5e57fa9
--- /dev/null
+++ b/crates/tc2/src/i18n/catalogo/progreso.rs
@@ -0,0 +1,8 @@
+//! Mensajes del progreso en vivo (F3-12).
+
+crate::i18n::textos! {
+    AVISO_NO_ESCRITO {
+        es: "aviso: no se pudo escribir el progreso de {id} ({error}); la transicion se hizo igual y la status line lo recupera en la siguiente",
+        en: "warning: the progress of {id} could not be written ({error}); the transition was done anyway and the status line catches up on the next one",
+    }
+}
diff --git a/crates/tc2/src/i18n/catalogo/tablero.rs b/crates/tc2/src/i18n/catalogo/tablero.rs
index 0173d8d..401ee1c 100644
--- a/crates/tc2/src/i18n/catalogo/tablero.rs
+++ b/crates/tc2/src/i18n/catalogo/tablero.rs
@@ -2,8 +2,8 @@
 
 crate::i18n::textos! {
     USO_STATUS {
-        es: "uso: status [--json]",
-        en: "usage: status [--json]",
+        es: "uso: status [--json] | status --seguir",
+        en: "usage: status [--json] | status --seguir",
     }
     USO_BOARD {
         es: "uso: board [--sin-plataforma] [--json]",
diff --git a/crates/tc2/src/identidad.rs b/crates/tc2/src/identidad.rs
index 27f8877..debebab 100644
--- a/crates/tc2/src/identidad.rs
+++ b/crates/tc2/src/identidad.rs
@@ -266,23 +266,13 @@ impl Cache {
                     .con("error", e.to_string()),
             )
         };
-        if let Some(padre) = self.ruta.parent() {
-            std::fs::create_dir_all(padre).map_err(error)?;
-        }
         let texto = format!(
             "---\nproveedor: \"{}\"\nid: {}\nlogin: {}\nverificado: {ahora}\n",
             cuenta.proveedor.nombre(),
             cuenta.id,
             crate::tarea::yaml::cadena(&cuenta.login),
         );
-        let tmp = self
-            .ruta
-            .with_extension(format!("yml.tmp-{}", std::process::id()));
-        std::fs::write(&tmp, texto).map_err(error)?;
-        std::fs::rename(&tmp, &self.ruta).map_err(|e| {
-            let _ = std::fs::remove_file(&tmp);
-            error(e)
-        })
+        crate::huella::escribir_local(&self.ruta, texto.as_bytes()).map_err(error)
     }
 }
 
diff --git a/crates/tc2/src/init/inventario.rs b/crates/tc2/src/init/inventario.rs
index dbe37fb..5411d1e 100644
--- a/crates/tc2/src/init/inventario.rs
+++ b/crates/tc2/src/init/inventario.rs
@@ -230,11 +230,39 @@ pub(super) fn plan(c: &Contexto<'_>) -> Result<Vec<Accion>, Informe> {
             .motivo("sin-plantilla")
     });
 
+    // Status line de Claude Code (F3-12, D2A de TASK-084): solo propuesta;
+    // `init` nunca lee ni escribe un `settings.json`.
+    v.push(status_line(&principal));
+
     // Rulesets: solo propuesta (D1); los aplica TASK-118.
     puertas(&plataforma, &mut v);
     Ok(v)
 }
 
+/// Donde se recomienda pegar la status line: el `settings.local.json` del
+/// proyecto, que no sustituye la de la 0.16 de `~/.claude/settings.json`.
+pub(super) const RUTA_STATUS_LINE: &str = ".claude/settings.local.json";
+
+/// La propuesta de la status line: la orden exacta en `orden=` y el bloque
+/// `statusLine` en `--json`. No mira el fichero, asi que no puede saber si
+/// ya esta: sale siempre `pendiente motivo=pegar-a-mano` (no `persona`, que
+/// en una segunda ejecucion diria que queda algo por hacer que quiza ya se
+/// hizo; C2 de TASK-028).
+fn status_line(principal: &Path) -> Accion {
+    let mut a = Accion::nueva(
+        t(&principal.join(RUTA_STATUS_LINE)),
+        "status-line",
+        "taskcodetwo",
+        Decision::Pendiente,
+    )
+    .motivo("pegar-a-mano")
+    .orden(distribucion::STATUS_LINE);
+    a.propuesta = Some(serde_json::json!({
+        "statusLine": {"type": "command", "command": distribucion::STATUS_LINE}
+    }));
+    a
+}
+
 /// Linea `rechaza motivo=proyecto-016` (C4).
 fn rechazo_016(ruta: String, clase: &'static str) -> Accion {
     let mut a = Accion::nueva(ruta.clone(), clase, "taskcode-016", Decision::Rechaza)
diff --git a/crates/tc2/src/init/mod.rs b/crates/tc2/src/init/mod.rs
index 52a399f..a733980 100644
--- a/crates/tc2/src/init/mod.rs
+++ b/crates/tc2/src/init/mod.rs
@@ -12,6 +12,9 @@
 //! de estado montada en `tareas/` y `personas.yml` con la persona del token.
 //! El workflow de CI queda como ranura `pendiente` hasta F5-1 y los rulesets
 //! `taskcodetwo-*` solo se proponen (`persona`): aplicarlos es de TASK-118.
+//! La status line de Claude Code (F3-12) tambien se propone, como
+//! `pendiente motivo=pegar-a-mano` con la orden exacta: `init` nunca lee ni
+//! escribe un `settings.json`.
 //!
 //! Nunca pisa la 0.16 ni una configuracion que no vale: `rechazada` sin
 //! escribir nada. Con `origin` y sin red, `sinred pendiente=no` sin escribir.
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 48633c5..de3f5d0 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -42,6 +42,7 @@ pub mod init;
 pub mod instalacion;
 pub mod inventario;
 pub mod plataforma;
+pub mod progreso;
 pub mod recursos;
 pub mod roadmap;
 pub mod siguiente;
@@ -92,6 +93,8 @@ where
 {
     let args: Vec<String> = args.into_iter().map(|a| a.as_ref().to_owned()).collect();
     match args.as_slice() {
+        // F3-12: la status line, lo primero y sin identidad, catalogo ni git.
+        [a, b] if a == progreso::ESTADO && b == "--stdin" => progreso::estado_del_proceso(),
         [a] if a == "--version" || a == "-V" => Salida {
             codigo: 0,
             stdout: format!(
@@ -160,27 +163,32 @@ where
         }
         [orden, resto @ ..] if orden == toma::NOMBRE => {
             let (json, resto) = con_json(resto);
-            de_una_tarea(toma::NOMBRE, |e| toma::orden(&resto, e)).salida_en(json, idioma)
+            de_una_tarea(toma::NOMBRE, (json, idioma), |e| toma::orden(&resto, e))
         }
         [orden, resto @ ..] if orden == ciclo::DESCARTAR => {
             let (json, resto) = con_json(resto);
-            de_una_tarea(ciclo::DESCARTAR, |e| ciclo::orden_descartar(&resto, e))
-                .salida_en(json, idioma)
+            de_una_tarea(ciclo::DESCARTAR, (json, idioma), |e| {
+                ciclo::orden_descartar(&resto, e)
+            })
         }
         [orden, resto @ ..] if orden == ciclo::PAUSA || orden == ciclo::REANUDAR => {
             let (json, resto) = con_json(resto);
             let reanudar = orden == ciclo::REANUDAR;
-            de_una_tarea(orden, |e| ciclo::orden_pausa(&resto, reanudar, e)).salida_en(json, idioma)
+            de_una_tarea(orden, (json, idioma), |e| {
+                ciclo::orden_pausa(&resto, reanudar, e)
+            })
         }
         [orden, resto @ ..] if orden == ciclo::LIBERAR => {
             let (json, resto) = con_json(resto);
-            de_una_tarea(ciclo::LIBERAR, |e| ciclo::orden_liberar(&resto, e))
-                .salida_en(json, idioma)
+            de_una_tarea(ciclo::LIBERAR, (json, idioma), |e| {
+                ciclo::orden_liberar(&resto, e)
+            })
         }
         [orden, resto @ ..] if orden == ciclo::REASIGNAR => {
             let (json, resto) = con_json(resto);
-            de_una_tarea(ciclo::REASIGNAR, |e| ciclo::orden_reasignar(&resto, e))
-                .salida_en(json, idioma)
+            de_una_tarea(ciclo::REASIGNAR, (json, idioma), |e| {
+                ciclo::orden_reasignar(&resto, e)
+            })
         }
         [orden, resto @ ..] if orden == estimacion::NOMBRE => {
             let (json, resto) = con_json(resto);
@@ -321,29 +329,124 @@ fn orden_identidad(args: &[String]) -> Informe {
 /// `reanudar`, F3-7; `liberar` y `reasignar`, F3-10) en la carpeta actual:
 /// quien la lanza sale de
 /// [`identidad::resolver`] con el terminal y los tokens del proceso.
-fn de_una_tarea(nombre: &str, orden: impl FnOnce(&toma::Entorno<'_>) -> Informe) -> Informe {
-    use std::io::IsTerminal;
+///
+/// Si la transicion sale bien, refresca el progreso en vivo de la tarea
+/// (F3-12): [`progreso::refrescar`], o [`progreso::quitar`] tras
+/// `descartar`, `liberar` y `reasignar`. Un fallo ahi no cambia la
+/// transicion: la linea gana `progreso=no-escrito` y stderr un aviso, con el
+/// mismo codigo.
+fn de_una_tarea(
+    nombre: &str,
+    (json, idioma): (bool, Idioma),
+    orden: impl FnOnce(&toma::Entorno<'_>) -> Informe,
+) -> Salida {
     let dir = match std::env::current_dir() {
         Ok(d) => d,
         Err(e) => {
             return Informe::nuevo(nombre, Resultado::Error)
-                .diagnostico(Mensaje::nuevo(&txt::CARPETA_ACTUAL).con("error", e.to_string()));
+                .diagnostico(Mensaje::nuevo(&txt::CARPETA_ACTUAL).con("error", e.to_string()))
+                .salida_en(json, idioma);
         }
     };
     let g = git::Git::nuevo();
     let ahora = identidad::ahora();
+    let i = transicion(&dir, &g, ahora, orden);
+    // `start` que publico la toma (`estado=`) ya escribio el progreso con lo
+    // que calculo dentro (un solo `git log` y una sola lectura de todos los
+    // `tarea.md`, R1-05 y R1-06 de TASK-083).
+    let hecho = nombre == toma::NOMBRE && i.valor("estado").is_some();
+    con_progreso(i, |id| {
+        if hecho {
+            Ok(progreso::Refresco::Escrito)
+        } else if [ciclo::DESCARTAR, ciclo::LIBERAR, ciclo::REASIGNAR].contains(&nombre) {
+            progreso::quitar(&g, &dir, id)
+        } else {
+            progreso::refrescar(&g, &dir, id, ahora)
+        }
+    })
+    .salida(json, idioma)
+}
+
+/// El informe de una transicion y, si fallo el refresco del progreso, el
+/// aviso para stderr.
+struct ConProgreso {
+    informe: Informe,
+    aviso: Option<Mensaje>,
+}
+
+impl ConProgreso {
+    fn salida(self, json: bool, idioma: Idioma) -> Salida {
+        let mut s = self.informe.salida_en(json, idioma);
+        if let Some(a) = self.aviso {
+            s.stderr.push_str(&a.texto(idioma));
+            s.stderr.push('\n');
+        }
+        s
+    }
+}
+
+/// Refresca el progreso de la tarea (`id=` de la linea) si la transicion
+/// salio bien; un fallo de `refrescar` no toca el resultado. Si la propia
+/// transicion ya dice `progreso=no-escrito` (`start`), solo el aviso.
+fn con_progreso(
+    i: Informe,
+    refrescar: impl FnOnce(&str) -> Result<progreso::Refresco, String>,
+) -> ConProgreso {
+    let aviso = |id: &str, e: &str| {
+        Some(
+            Mensaje::nuevo(&i18n::catalogo::progreso::AVISO_NO_ESCRITO)
+                .con("id", id)
+                .con("error", tablero::sanear(e, 200)),
+        )
+    };
+    let id = match i.valor("id") {
+        Some(id) if i.resultado.es_exito() => id.to_owned(),
+        _ => {
+            return ConProgreso {
+                informe: i,
+                aviso: None,
+            };
+        }
+    };
+    if i.valor(progreso::CLAVE) == Some(progreso::NO_ESCRITO) {
+        return ConProgreso {
+            aviso: aviso(&id, "escritura"),
+            informe: i,
+        };
+    }
+    match refrescar(&id) {
+        Ok(_) => ConProgreso {
+            informe: i,
+            aviso: None,
+        },
+        Err(e) => ConProgreso {
+            informe: i.con(progreso::CLAVE, progreso::NO_ESCRITO),
+            aviso: aviso(&id, &e),
+        },
+    }
+}
+
+/// La transicion en `dir`, con quien la lanza resuelto por
+/// [`identidad::resolver`].
+fn transicion(
+    dir: &std::path::Path,
+    g: &git::Git,
+    ahora: u64,
+    orden: impl FnOnce(&toma::Entorno<'_>) -> Informe,
+) -> Informe {
+    use std::io::IsTerminal;
     let quien = || {
         identidad::resolver(identidad::Entorno {
-            dir: &dir,
-            git: &g,
+            dir,
+            git: g,
             tty: std::io::stdin().is_terminal(),
             credenciales: Box::new(plataforma::CredencialesEntorno),
             ahora,
         })
     };
     let e = toma::Entorno {
-        dir: &dir,
-        git: &g,
+        dir,
+        git: g,
         quien: &quien,
         catalogo: &catalogo,
         ahora,
@@ -774,6 +877,59 @@ mod tests {
         }
     }
 
+    /// F3-12: un fallo al refrescar el progreso nunca cambia el resultado de
+    /// la transicion (mismo codigo; la linea gana `progreso=no-escrito` y
+    /// stderr un aviso); una transicion que no sale bien no lo refresca.
+    #[test]
+    fn el_progreso_no_cambia_la_transicion() {
+        let ok = Informe::nuevo("start", Resultado::Ok).con("id", "TASK-001");
+        let s =
+            con_progreso(ok.clone(), |_| Err("disco lleno".to_owned())).salida(false, Idioma::Es);
+        assert_eq!(s.codigo, 0);
+        assert_eq!(s.stdout, "start ok id=TASK-001 progreso=no-escrito\n");
+        assert!(
+            s.stderr
+                .starts_with("aviso: no se pudo escribir el progreso de TASK-001 (disco lleno)"),
+            "{}",
+            s.stderr
+        );
+        let s = con_progreso(ok, |_| Ok(progreso::Refresco::Escrito)).salida(false, Idioma::Es);
+        assert_eq!(
+            (s.codigo, s.stdout.as_str(), s.stderr.as_str()),
+            (0, "start ok id=TASK-001\n", "")
+        );
+        let mut llamado = false;
+        let mal = Informe::nuevo("pausa", Resultado::Rechazada).con("id", "TASK-001");
+        let s = con_progreso(mal, |_| {
+            llamado = true;
+            Ok(progreso::Refresco::Escrito)
+        })
+        .salida(false, Idioma::Es);
+        assert_eq!((s.codigo, llamado), (3, false));
+        // `start` que ya lo intento: solo el aviso, sin volver a refrescar.
+        let ya = Informe::nuevo("start", Resultado::Ok)
+            .con("id", "TASK-001")
+            .con("progreso", "no-escrito");
+        let mut llamado = false;
+        let s = con_progreso(ya, |_| {
+            llamado = true;
+            Ok(progreso::Refresco::Escrito)
+        })
+        .salida(false, Idioma::Es);
+        assert_eq!((s.codigo, llamado), (0, false));
+        assert_eq!(s.stdout, "start ok id=TASK-001 progreso=no-escrito\n");
+        assert_eq!(s.stderr.lines().count(), 1, "{}", s.stderr);
+        let sin_cambios = Informe::nuevo("pausa", Resultado::SinCambios).con("id", "TASK-001");
+        let s = con_progreso(sin_cambios, |_| Err("x".to_owned())).salida(true, Idioma::En);
+        assert_eq!(s.codigo, 0);
+        assert!(
+            s.stdout.contains("\"progreso\":\"no-escrito\""),
+            "{}",
+            s.stdout
+        );
+        assert!(s.stderr.starts_with("warning: "), "{}", s.stderr);
+    }
+
     #[test]
     fn sin_orden_falla() {
         assert_eq!(ejecutar(Vec::<String>::new()).codigo, 2);
diff --git a/crates/tc2/src/main.rs b/crates/tc2/src/main.rs
index 7e2c429..c481558 100644
--- a/crates/tc2/src/main.rs
+++ b/crates/tc2/src/main.rs
@@ -35,6 +35,21 @@ fn main() -> ExitCode {
             }
         }
     }
+    // F3-12: `status --seguir` escribe a trozos (una linea cada vez que
+    // cambia), que `Salida` no sabe hacer. Termina con Ctrl+C o al cerrarse
+    // stdout, con codigo 0.
+    if args.len() == 2 && args[0] == tc2::tablero::STATUS && args[1] == tc2::tablero::SEGUIR {
+        let dir = std::env::current_dir().unwrap_or_default();
+        let mut stdout = std::io::stdout();
+        let _ = tc2::progreso::seguir(
+            &dir,
+            &mut stdout,
+            &tc2::identidad::ahora,
+            &|| false,
+            tc2::progreso::ESPERA_SEGUIR,
+        );
+        return ExitCode::SUCCESS;
+    }
     let salida = tc2::ejecutar_en(idioma, args);
     // Si no se puede escribir la salida (stdout cerrado), no se da por bueno:
     // quien lee el codigo 0 creeria que recibio la respuesta.
diff --git a/crates/tc2/src/progreso/calculo.rs b/crates/tc2/src/progreso/calculo.rs
new file mode 100644
index 0000000..85968e3
--- /dev/null
+++ b/crates/tc2/src/progreso/calculo.rs
@@ -0,0 +1,258 @@
+//! El calculo del progreso de una tarea abierta (puro: ni git ni ficheros).
+//!
+//! Sin segunda formula: reutiliza [`crate::estimacion`] (`parecidas`,
+//! `restante`, `eventos`, `ventanas_pausa` y `tramo`, de TASK-083).
+//!
+//! - `en-curso` es la fase de implementacion; `en-revision`, la de rondas
+//!   (con solo el cierre por delante).
+//! - Lo transcurrido en la fase es el real de F1-17 desde su inicio (`start`
+//!   o la primera `review`) hasta el ultimo evento: huecos de mas de 45 min
+//!   contados como 45 y sin las ventanas de pausa. Lo que pase despues lo
+//!   suma la vista con el reloj ([`super::linea`]).
+//! - La tabla guarda lo que falta para cada transcurrido de 0 a 480 min cada
+//!   5: la vista solo busca, no recalcula.
+
+use super::fichero::{MAX_AVISOS, PASO_MIN, PUNTOS, Progreso, Punto, VERSION};
+use crate::estimacion::real::{tramo, ventanas_pausa};
+use crate::estimacion::{self, Commits, Fase, Historico, eventos, parecidas, restante};
+use crate::tarea::{Cuerpo, Tarea};
+
+/// Estado de una tarea en la fase de implementacion.
+pub const EN_CURSO: &str = "en-curso";
+
+/// Estado de una tarea en la fase de rondas.
+pub const EN_REVISION: &str = "en-revision";
+
+/// Longitud maxima de un aviso.
+pub const MAX_AVISO: usize = 120;
+
+/// Aviso cuando el `git log` de `develop` no se pudo leer.
+pub const AVISO_SIN_LOG: &str = "historia sin git log: lo transcurrido no cuenta los commits";
+
+/// La fase en curso de una tarea en `estado`, su nombre y las que faltan.
+pub fn fase_de(estado: &str) -> Option<(Fase, &'static str, &'static [&'static str])> {
+    match estado {
+        EN_CURSO => Some((
+            Fase::Implementacion,
+            "implementacion",
+            &["rondas", "cierre"],
+        )),
+        EN_REVISION => Some((Fase::Rondas, "rondas", &["cierre"])),
+        _ => None,
+    }
+}
+
+/// `calcular(tarea, cuerpo, historico, commits, ahora)`: el progreso de `t`
+/// si esta en curso o en revision; si no, `None`. `ahora` en segundos desde
+/// 1970.
+pub fn calcular(
+    t: &Tarea,
+    c: &Cuerpo,
+    h: &Historico,
+    commits: &Commits,
+    ahora: u64,
+) -> Option<Progreso> {
+    let (fase, nombre, faltan) = fase_de(&t.estado)?;
+    let ahora_i = i64::try_from(ahora).unwrap_or(i64::MAX);
+    let ev = eventos(c, commits.de_tarea(&t.id));
+    let marca = if fase == Fase::Rondas {
+        "review"
+    } else {
+        "start"
+    };
+    let inicio = ev
+        .transiciones
+        .iter()
+        .filter(|p| p.fase == marca && p.ts <= ahora_i)
+        .map(|p| p.ts)
+        .min()
+        .unwrap_or(ahora_i);
+    let mut todos: Vec<i64> = ev.commits.clone();
+    todos.extend(&ev.coste);
+    todos.extend(ev.transiciones.iter().map(|p| p.ts));
+    todos.retain(|x| *x <= ahora_i);
+    todos.sort_unstable();
+    todos.dedup();
+    let ultimo = todos.last().copied().unwrap_or(inicio).max(inicio);
+    let pausas = ventanas_pausa(&ev.transiciones);
+    let transcurrido = tramo(inicio, ultimo, &todos, &pausas);
+    let obj = estimacion::objetivo(t, c);
+    let est = estimacion::estimar(&obj, &h.muestras, "");
+    let tabla = match parecidas(&obj, &h.muestras) {
+        Some((_, ms)) => (0..PUNTOS as u64)
+            .filter_map(|i| restante(&ms, fase, (i * PASO_MIN) as f64))
+            .map(|r| Punto {
+                mediana: r.mediana,
+                p80: r.p80,
+                desbordada: r.desbordada,
+            })
+            .collect(),
+        None => Vec::new(),
+    };
+    let mut avisos = Vec::new();
+    if !commits.leido() {
+        avisos.push(AVISO_SIN_LOG.to_owned());
+    }
+    avisos.truncate(MAX_AVISOS);
+    Some(Progreso {
+        version: VERSION,
+        id: t.id.clone(),
+        fase: nombre.to_owned(),
+        pausada: c.en_pausa(),
+        faltan: faltan.iter().map(|f| (*f).to_owned()).collect(),
+        transcurrido,
+        ultimo_evento: u64::try_from(ultimo).unwrap_or_default(),
+        calculada: ahora,
+        nivel: est.nivel,
+        n: est.n,
+        tabla,
+        avisos: avisos
+            .iter()
+            .map(|a| crate::tablero::sanear(a, MAX_AVISO))
+            .collect(),
+    })
+}
+
+#[cfg(test)]
+pub(crate) mod tests {
+    use super::*;
+    use crate::estimacion::{Muestra, Real};
+
+    /// 2026-11-17T10:00:00Z.
+    pub(crate) const T0: u64 = 1_794_909_600;
+
+    /// Seis muestras de repo: implementacion 30..80, rondas 10 y cierre 5.
+    pub(crate) fn historico() -> Historico {
+        let muestras = [30.0, 40.0, 50.0, 60.0, 70.0, 80.0]
+            .iter()
+            .enumerate()
+            .map(|(i, imp)| Muestra {
+                id: format!("TASK-{:03}", i + 1),
+                miembros: vec![format!("TASK-{:03}", i + 1)],
+                carril: None,
+                complejidad: None,
+                criterios: 1,
+                real: Real {
+                    implementacion: *imp,
+                    rondas: 10.0,
+                    cierre: 5.0,
+                },
+                start: 0,
+                finish: 0,
+            })
+            .collect();
+        Historico {
+            muestras,
+            ..Historico::default()
+        }
+    }
+
+    /// Una tarea `TASK-007` en `estado` con las filas de `filas` ((segundos
+    /// desde [`T0`], fase)).
+    pub(crate) fn tarea(estado: &str, filas: &[(u64, &str)]) -> (Tarea, Cuerpo) {
+        let mut t = Tarea::nueva("TASK-007", "Abierta", "2026-11-17");
+        estado.clone_into(&mut t.estado);
+        let mut texto = "## Objetivo\n\nX.\n\n## Criterios de aceptacion\n\n- [ ] a\n\n## Transiciones\n\n| fecha | fase | modo | decidido_por | version |\n|---|---|---|---|---|\n".to_owned();
+        for (s, f) in filas {
+            texto.push_str(&format!(
+                "| {} | {f} | manual | persona | 0.1 |\n",
+                crate::toma::instante(T0 + s)
+            ));
+        }
+        (t, Cuerpo::leer(&texto))
+    }
+
+    /// C2 (calculo): `en-curso` es implementacion con rondas y cierre por
+    /// delante; la tabla coincide con `restante` en cada punto.
+    #[test]
+    fn en_curso_es_implementacion_y_la_tabla_es_restante() {
+        let h = historico();
+        let (t, c) = tarea(EN_CURSO, &[(0, "start")]);
+        let p = calcular(&t, &c, &h, &Commits::vacios(), T0 + 1200);
+        let p = p.unwrap_or_else(|| unreachable!());
+        assert_eq!(p.fase, "implementacion");
+        assert_eq!(p.faltan, ["rondas", "cierre"]);
+        assert_eq!((p.nivel.as_str(), p.n), ("repo", 6));
+        assert_eq!((p.transcurrido, p.ultimo_evento), (0.0, T0));
+        assert_eq!(p.calculada, T0 + 1200);
+        assert_eq!(p.tabla.len(), PUNTOS);
+        let ms: Vec<&Muestra> = h.muestras.iter().collect();
+        for (i, punto) in p.tabla.iter().enumerate() {
+            let r = restante(&ms, Fase::Implementacion, (i as u64 * PASO_MIN) as f64);
+            assert_eq!(
+                r.map(|r| (r.mediana, r.p80, r.desbordada)),
+                Some((punto.mediana, punto.p80, punto.desbordada)),
+                "{i}"
+            );
+        }
+        // A los 20 minutos: ≈50 (p80 65), sin desbordar.
+        assert_eq!(
+            p.tabla[4],
+            Punto {
+                mediana: 50.0,
+                p80: 65.0,
+                desbordada: false
+            }
+        );
+        assert!(!p.pausada && p.avisos.is_empty());
+    }
+
+    /// C4: `en-revision` es la fase de rondas con solo el cierre por delante,
+    /// y lo transcurrido cuenta desde la primera `review`.
+    #[test]
+    fn c4_en_revision_es_rondas_con_solo_cierre() {
+        let (t, c) = tarea(
+            EN_REVISION,
+            &[(0, "start"), (1800, "review"), (2400, "review")],
+        );
+        let p = calcular(&t, &c, &historico(), &Commits::vacios(), T0 + 3000);
+        let p = p.unwrap_or_else(|| unreachable!());
+        assert_eq!(p.fase, "rondas");
+        assert_eq!(p.faltan, ["cierre"]);
+        assert_eq!(p.transcurrido, 10.0);
+        assert_eq!(p.ultimo_evento, T0 + 2400);
+        // En rondas lo que falta son rondas y cierre (15 a los 0 min).
+        assert_eq!(p.tabla[0].mediana, 15.0);
+        // Otros estados no tienen progreso.
+        for e in ["planificada", "terminada", "descartada", "en-diseno"] {
+            let (t, c) = tarea(e, &[(0, "start")]);
+            assert!(
+                calcular(&t, &c, &historico(), &Commits::vacios(), T0).is_none(),
+                "{e}"
+            );
+        }
+    }
+
+    /// La pausa congela: la ventana no cuenta, y con la ultima fila `pausa`
+    /// la tarea sale pausada. Los huecos de mas de 45 min cuentan 45.
+    #[test]
+    fn pausa_y_tope_de_45() {
+        let (t, c) = tarea(
+            EN_CURSO,
+            &[(0, "start"), (600, "pausa"), (4200, "reanudar")],
+        );
+        let p = calcular(&t, &c, &historico(), &Commits::vacios(), T0 + 4800);
+        let p = p.unwrap_or_else(|| unreachable!());
+        assert_eq!((p.transcurrido, p.pausada), (10.0, false));
+        let (t, c) = tarea(EN_CURSO, &[(0, "start"), (600, "pausa")]);
+        let p = calcular(&t, &c, &historico(), &Commits::vacios(), T0 + 9000);
+        assert_eq!(p.map(|p| (p.transcurrido, p.pausada)), Some((10.0, true)));
+        // Un hueco de 2 h entre start y un commit cuenta 45.
+        let (t, c) = tarea(EN_CURSO, &[(0, "start")]);
+        let log = format!("a\u{1f}\u{1f}{}\u{1f}TASK-007 algo\n", T0 + 7200);
+        let p = calcular(&t, &c, &historico(), &Commits::de_log(&log), T0 + 7300);
+        assert_eq!(p.map(|p| p.transcurrido), Some(45.0));
+    }
+
+    /// Sin parecidas, la tabla va vacia con `sin-datos`; sin `git log`, el
+    /// aviso.
+    #[test]
+    fn sin_parecidas_y_sin_log() {
+        let (t, c) = tarea(EN_CURSO, &[(0, "start")]);
+        let p = calcular(&t, &c, &Historico::default(), &Commits::sin_log(), T0);
+        let p = p.unwrap_or_else(|| unreachable!());
+        assert!(p.tabla.is_empty());
+        assert_eq!((p.nivel.as_str(), p.n), ("sin-datos", 0));
+        assert_eq!(p.avisos, [AVISO_SIN_LOG]);
+    }
+}
diff --git a/crates/tc2/src/progreso/fichero.rs b/crates/tc2/src/progreso/fichero.rs
new file mode 100644
index 0000000..b0b0ab4
--- /dev/null
+++ b/crates/tc2/src/progreso/fichero.rs
@@ -0,0 +1,294 @@
+//! El fichero de progreso de una tarea (F3-12): `progreso/TASK-NNN.json` en
+//! la carpeta propia de la carpeta comun de git ([`crate::huella`]), con el
+//! anterior en `TASK-NNN.anterior.json`.
+//!
+//! Lleva lo que la vista necesita para pintar sin recorrer el historico
+//! (L1 de F1-17): la fase, las que faltan, lo transcurrido al calcular, el
+//! ultimo evento, el nivel, `n` y la tabla de lo que falta de 0 a
+//! [`TOPE_MIN`] minutos cada [`PASO_MIN`] (sacada de
+//! [`crate::estimacion::restante`]). Su tamano no depende del numero de
+//! tareas. Lleva `version` para poder anadir el lote (F4-9).
+//!
+//! Lo que se lee es texto no confiable (otro proceso o una persona puede
+//! tocarlo): se valida al leer y se sanea al pintar.
+
+use std::io::Read;
+use std::path::{Path, PathBuf};
+
+use serde::{Deserialize, Serialize};
+
+/// Version del formato del fichero.
+pub const VERSION: u32 = 1;
+
+/// Minutos entre dos puntos de la tabla.
+pub const PASO_MIN: u64 = 5;
+
+/// Minutos del ultimo punto de la tabla (8 h).
+pub const TOPE_MIN: u64 = 480;
+
+/// Puntos de la tabla: de 0 a [`TOPE_MIN`] cada [`PASO_MIN`].
+pub const PUNTOS: usize = (TOPE_MIN / PASO_MIN) as usize + 1;
+
+/// Bytes como mucho de un fichero de progreso: uno mayor no es de `tc2`.
+pub const LIMITE: u64 = 64 * 1024;
+
+/// Avisos como mucho en un fichero.
+pub const MAX_AVISOS: usize = 4;
+
+/// Lo que falta con un transcurrido dado.
+#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
+pub struct Punto {
+    /// Mediana de lo que falta (con `desbordada`, cota inferior).
+    pub mediana: f64,
+    /// p80 de lo que falta.
+    pub p80: f64,
+    /// La fase en curso va por encima de sus parecidas.
+    pub desbordada: bool,
+}
+
+/// El progreso de una tarea en curso, tal como lo deja la ultima transicion.
+#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
+pub struct Progreso {
+    /// [`VERSION`].
+    pub version: u32,
+    /// `TASK-NNN`.
+    pub id: String,
+    /// `implementacion` o `rondas`.
+    pub fase: String,
+    /// Si la ultima de sus filas `pausa`/`reanudar` es `pausa`: el reloj no
+    /// corre.
+    pub pausada: bool,
+    /// Fases que faltan despues de la actual, en orden.
+    pub faltan: Vec<String>,
+    /// Minutos activos de la fase en curso hasta [`Progreso::ultimo_evento`]
+    /// (regla del real de F1-17: huecos a 45 min, sin pausas).
+    pub transcurrido: f64,
+    /// Instante del ultimo evento de la tarea (segundos desde 1970).
+    pub ultimo_evento: u64,
+    /// Instante del calculo (segundos desde 1970).
+    pub calculada: u64,
+    /// Nivel de la cascada o `sin-datos`.
+    pub nivel: String,
+    /// Muestras del nivel (con `sin-datos`, las disponibles).
+    pub n: usize,
+    /// Lo que falta cada [`PASO_MIN`] minutos transcurridos; vacia sin
+    /// estimacion.
+    pub tabla: Vec<Punto>,
+    /// Avisos para la vista (se sanean al escribir y al pintar).
+    pub avisos: Vec<String>,
+}
+
+impl Progreso {
+    /// Si lo leido tiene la forma que escribe este `tc2`.
+    pub fn valido(&self) -> bool {
+        self.version == VERSION
+            && crate::git::es_id(&self.id)
+            && matches!(self.fase.as_str(), "implementacion" | "rondas")
+            && self
+                .faltan
+                .iter()
+                .all(|f| crate::estimacion::FASES.contains(&f.as_str()))
+            && self.tabla.len() <= PUNTOS
+            && self.avisos.len() <= MAX_AVISOS
+            && self.transcurrido.is_finite()
+            && self.transcurrido >= 0.0
+            && self
+                .tabla
+                .iter()
+                .all(|p| p.mediana.is_finite() && p.p80.is_finite())
+    }
+
+    /// El JSON del fichero.
+    pub fn json(&self) -> Vec<u8> {
+        let mut v = serde_json::to_vec(self).unwrap_or_default();
+        v.push(b'\n');
+        v
+    }
+}
+
+/// Carpeta del progreso del repo cuya carpeta comun de git es `comun`.
+pub fn carpeta(comun: &Path) -> PathBuf {
+    crate::huella::en_comun(comun).join(crate::huella::CARPETA_PROGRESO)
+}
+
+/// `TASK-NNN.json` en `carpeta`.
+pub fn ruta(carpeta: &Path, id: &str) -> PathBuf {
+    carpeta.join(format!("{id}.json"))
+}
+
+/// `TASK-NNN.anterior.json` en `carpeta`.
+pub fn ruta_anterior(carpeta: &Path, id: &str) -> PathBuf {
+    carpeta.join(format!("{id}.anterior.json"))
+}
+
+/// Bytes de `ruta`, con tope; `None` si no se lee o pasa de [`LIMITE`].
+fn bytes(ruta: &Path) -> Option<Vec<u8>> {
+    let mut v = Vec::new();
+    std::fs::File::open(ruta)
+        .ok()?
+        .take(LIMITE + 1)
+        .read_to_end(&mut v)
+        .ok()?;
+    (v.len() as u64 <= LIMITE).then_some(v)
+}
+
+/// Un progreso valido de `ruta`, si lo hay.
+fn de(ruta: &Path, id: &str) -> Option<Progreso> {
+    let p: Progreso = serde_json::from_slice(&bytes(ruta)?).ok()?;
+    (p.valido() && p.id == id).then_some(p)
+}
+
+/// Lo que da la lectura de una tarea.
+#[derive(Debug, Clone, PartialEq)]
+pub enum Lectura {
+    /// No hay fichero: la tarea no esta en curso en este clon.
+    Ninguno,
+    /// El fichero, valido.
+    Bueno(Progreso),
+    /// El fichero no se entiende (a medias o tocado): el anterior, que si.
+    Anterior(Progreso),
+    /// Ni el fichero ni el anterior se entienden.
+    Ilegible,
+}
+
+/// Lee el progreso de `id` en `carpeta`. Sin fichero, [`Lectura::Ninguno`]
+/// (el anterior no cuenta: la tarea dejo de estar en curso).
+pub fn leer(carpeta: &Path, id: &str) -> Lectura {
+    let r = ruta(carpeta, id);
+    if !r.exists() {
+        return Lectura::Ninguno;
+    }
+    if let Some(p) = de(&r, id) {
+        return Lectura::Bueno(p);
+    }
+    de(&ruta_anterior(carpeta, id), id).map_or(Lectura::Ilegible, Lectura::Anterior)
+}
+
+/// Escribe `p` en `carpeta`: antes, el que habia pasa a `.anterior.json`.
+/// Cada paso, de una vez ([`crate::huella::escribir_local`]).
+pub fn escribir(carpeta: &Path, p: &Progreso) -> std::io::Result<()> {
+    let r = ruta(carpeta, &p.id);
+    // Solo un anterior valido: si el actual quedo a medias, el anterior que
+    // habia sigue siendo el ultimo bueno.
+    if let Some(previo) = de(&r, &p.id) {
+        crate::huella::escribir_local(&ruta_anterior(carpeta, &p.id), &previo.json())?;
+    }
+    crate::huella::escribir_local(&r, &p.json())
+}
+
+/// Borra el progreso de `id` (y su anterior). Que no exista no es un fallo.
+pub fn borrar(carpeta: &Path, id: &str) -> std::io::Result<()> {
+    for r in [ruta(carpeta, id), ruta_anterior(carpeta, id)] {
+        match std::fs::remove_file(&r) {
+            Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e),
+            _ => {}
+        }
+    }
+    Ok(())
+}
+
+/// IDs con fichero de progreso en `carpeta` (sin los anteriores), en orden.
+pub fn ids(carpeta: &Path) -> Vec<String> {
+    let mut v: Vec<String> = std::fs::read_dir(carpeta)
+        .into_iter()
+        .flatten()
+        .flatten()
+        .filter_map(|e| {
+            let n = e.file_name().into_string().ok()?;
+            let id = n.strip_suffix(".json")?;
+            crate::git::es_id(id).then(|| id.to_owned())
+        })
+        .collect();
+    v.sort();
+    v
+}
+
+#[cfg(test)]
+pub(crate) mod tests {
+    use super::*;
+
+    /// Un progreso de ejemplo.
+    pub(crate) fn ejemplo(id: &str) -> Progreso {
+        Progreso {
+            version: VERSION,
+            id: id.to_owned(),
+            fase: "implementacion".to_owned(),
+            pausada: false,
+            faltan: vec!["rondas".to_owned(), "cierre".to_owned()],
+            transcurrido: 0.0,
+            ultimo_evento: 1_000,
+            calculada: 1_000,
+            nivel: "repo".to_owned(),
+            n: 6,
+            tabla: vec![
+                Punto {
+                    mediana: 50.0,
+                    p80: 65.0,
+                    desbordada: false,
+                };
+                PUNTOS
+            ],
+            avisos: Vec::new(),
+        }
+    }
+
+    fn dir(nombre: &str) -> PathBuf {
+        let d = std::env::temp_dir()
+            .join("tc2-tests")
+            .join(format!("progreso-fichero-{nombre}-{}", std::process::id()));
+        let _ = std::fs::remove_dir_all(&d);
+        d
+    }
+
+    /// C3 (en pequeno) y C8: al reescribir, el bueno pasa a `.anterior`; un
+    /// fichero a medias da el anterior; sin fichero, ninguno.
+    #[test]
+    fn anterior_y_fichero_a_medias() -> std::io::Result<()> {
+        let d = dir("anterior");
+        assert_eq!(leer(&d, "TASK-001"), Lectura::Ninguno);
+        let mut p = ejemplo("TASK-001");
+        escribir(&d, &p)?;
+        assert_eq!(leer(&d, "TASK-001"), Lectura::Bueno(p.clone()));
+        assert!(!ruta_anterior(&d, "TASK-001").exists());
+        let primero = p.clone();
+        p.calculada = 2_000;
+        escribir(&d, &p)?;
+        assert_eq!(leer(&d, "TASK-001"), Lectura::Bueno(p.clone()));
+        assert_eq!(
+            std::fs::read(ruta_anterior(&d, "TASK-001"))?,
+            primero.json()
+        );
+        // A medias: el anterior.
+        let json = p.json();
+        std::fs::write(ruta(&d, "TASK-001"), &json[..json.len() / 2])?;
+        assert_eq!(leer(&d, "TASK-001"), Lectura::Anterior(primero.clone()));
+        // Escribir sobre uno a medias conserva el anterior bueno.
+        p.calculada = 3_000;
+        escribir(&d, &p)?;
+        assert_eq!(
+            std::fs::read(ruta_anterior(&d, "TASK-001"))?,
+            primero.json()
+        );
+        // Los dos mal: ilegible.
+        std::fs::write(ruta(&d, "TASK-001"), b"{")?;
+        std::fs::write(ruta_anterior(&d, "TASK-001"), b"x")?;
+        assert_eq!(leer(&d, "TASK-001"), Lectura::Ilegible);
+        // Otro ID dentro, una version que no es o enorme: no valen.
+        let mut otro = ejemplo("TASK-002");
+        std::fs::write(ruta(&d, "TASK-001"), otro.json())?;
+        assert_eq!(leer(&d, "TASK-001"), Lectura::Ilegible);
+        otro.id = "TASK-001".to_owned();
+        otro.version = 2;
+        std::fs::write(ruta(&d, "TASK-001"), otro.json())?;
+        assert_eq!(leer(&d, "TASK-001"), Lectura::Ilegible);
+        std::fs::write(ruta(&d, "TASK-001"), vec![b' '; LIMITE as usize + 1])?;
+        assert_eq!(leer(&d, "TASK-001"), Lectura::Ilegible);
+        assert_eq!(ids(&d), ["TASK-001"]);
+        borrar(&d, "TASK-001")?;
+        borrar(&d, "TASK-001")?;
+        assert_eq!(leer(&d, "TASK-001"), Lectura::Ninguno);
+        assert!(ids(&d).is_empty());
+        let _ = std::fs::remove_dir_all(&d);
+        Ok(())
+    }
+}
diff --git a/crates/tc2/src/progreso/linea.rs b/crates/tc2/src/progreso/linea.rs
new file mode 100644
index 0000000..6ffc56b
--- /dev/null
+++ b/crates/tc2/src/progreso/linea.rs
@@ -0,0 +1,267 @@
+//! La linea de progreso (puro): lo que pintan la status line y
+//! `tc2 status --seguir` (L5 de F1-17).
+//!
+//! ```text
+//! TASK-075 · implementacion · fases que faltan: rondas, cierre · faltan ≈50 min (p80 96) · n=17 carril+complejidad+tamano
+//! TASK-075 · implementacion · fases que faltan: rondas, cierre · faltan ≥31 min (fase en curso por encima de sus parecidas) · n=17 carril · sin actividad registrada hace 127 min
+//! TASK-075 · pausada en implementacion · fases que faltan: rondas, cierre · faltan: sin estimacion (n=3)
+//! tc2 · sin tarea en curso
+//! ```
+//!
+//! Lo transcurrido es el del calculo mas el reloj desde el ultimo evento
+//! (o desde la ultima actividad que se ve sin lanzar git: el `logs/HEAD` del
+//! worktree), cada hueco con el tope de 45 min de F1-17. Pausada, el reloj
+//! no corre. Todo lo que viene del fichero se sanea (F1-17, punto 8).
+
+use super::fichero::{PASO_MIN, Progreso};
+use crate::estimacion::{UMBRAL_MIN, minutos};
+use crate::tablero::sanear;
+
+/// La linea cuando no hay tarea en curso (o fuera de un repo).
+pub const SIN_TAREA: &str = "tc2 · sin tarea en curso";
+
+/// Separador de las partes de la linea.
+const SEP: &str = " · ";
+
+/// Longitud maxima del nivel en la linea.
+const MAX_NIVEL: usize = 40;
+
+/// Longitud maxima de cada aviso en la linea.
+pub const MAX_AVISO_LINEA: usize = 80;
+
+/// Avisos como mucho en la linea.
+const AVISOS_EN_LINEA: usize = 2;
+
+/// Minutos (con decimales) de `desde` a `hasta`; 0 si `hasta` no es
+/// posterior.
+fn min_entre(desde: u64, hasta: u64) -> f64 {
+    hasta.saturating_sub(desde) as f64 / 60.0
+}
+
+/// Lo transcurrido en la fase a las `ahora` y los minutos sin actividad
+/// (el hueco abierto, sin tope).
+pub fn transcurrido(p: &Progreso, ahora: u64, actividad: Option<u64>) -> (f64, f64) {
+    if p.pausada {
+        return (p.transcurrido, 0.0);
+    }
+    // Una actividad posterior al ultimo evento (un commit despues del
+    // calculo) parte el hueco en dos; una del futuro no cuenta.
+    let act = actividad
+        .filter(|a| *a <= ahora)
+        .unwrap_or(0)
+        .max(p.ultimo_evento);
+    let antes = min_entre(p.ultimo_evento, act).min(UMBRAL_MIN);
+    let abierto = min_entre(act, ahora);
+    (p.transcurrido + antes + abierto.min(UMBRAL_MIN), abierto)
+}
+
+/// La parte de lo que falta.
+fn lo_que_falta(p: &Progreso, t: f64) -> String {
+    let i = ((t / PASO_MIN as f64).floor().max(0.0) as usize).min(p.tabla.len().saturating_sub(1));
+    let nivel = sanear(&p.nivel, MAX_NIVEL);
+    match p.tabla.get(i) {
+        None => format!("faltan: sin estimacion (n={})", p.n),
+        Some(x) if x.desbordada => format!(
+            "faltan ≥{} min (fase en curso por encima de sus parecidas){SEP}n={} {nivel}",
+            minutos(x.mediana),
+            p.n
+        ),
+        Some(x) => format!(
+            "faltan ≈{} min (p80 {}){SEP}n={} {nivel}",
+            minutos(x.mediana),
+            minutos(x.p80),
+            p.n
+        ),
+    }
+}
+
+/// `linea(progreso, ahora, ultima_actividad)`: la linea de la tarea.
+pub fn linea(p: &Progreso, ahora: u64, actividad: Option<u64>) -> String {
+    let (t, abierto) = transcurrido(p, ahora, actividad);
+    let id = sanear(&p.id, 16);
+    let fase = sanear(&p.fase, 20);
+    let mut partes = vec![id];
+    partes.push(if p.pausada {
+        format!("pausada en {fase}")
+    } else {
+        fase
+    });
+    if !p.faltan.is_empty() {
+        let faltan: Vec<String> = p.faltan.iter().map(|f| sanear(f, 20)).collect();
+        partes.push(format!("fases que faltan: {}", faltan.join(", ")));
+    }
+    partes.push(lo_que_falta(p, t));
+    if abierto > UMBRAL_MIN {
+        partes.push(format!(
+            "sin actividad registrada hace {} min",
+            abierto.floor() as u64
+        ));
+    }
+    for a in p.avisos.iter().take(AVISOS_EN_LINEA) {
+        let a = sanear(a, MAX_AVISO_LINEA);
+        if !a.is_empty() {
+            partes.push(a);
+        }
+    }
+    partes.join(SEP)
+}
+
+/// La linea de un dato que no es el ultimo (el `.anterior` de un fichero
+/// ilegible): con su antiguedad.
+pub fn con_antiguedad(l: &str, p: &Progreso, ahora: u64) -> String {
+    format!(
+        "{l}{SEP}(dato de hace {} min)",
+        min_entre(p.calculada, ahora).floor() as u64
+    )
+}
+
+/// La linea de una tarea cuyo progreso no se lee.
+pub fn ilegible(id: &str) -> String {
+    format!("{}{SEP}progreso ilegible", sanear(id, 16))
+}
+
+/// La linea del principal con varias en curso: la mas reciente y `+N en
+/// curso`.
+pub fn con_mas(l: &str, otras: usize) -> String {
+    if otras == 0 {
+        l.to_owned()
+    } else {
+        format!("{l}{SEP}+{otras} en curso")
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::super::calculo::tests::{T0, historico, tarea};
+    use super::super::calculo::{EN_CURSO, calcular};
+    use super::super::fichero::tests::ejemplo;
+    use super::*;
+    use crate::estimacion::{Commits, Historico};
+
+    /// C2: muestras conocidas y 20 min transcurridos dan exactamente la
+    /// linea (fase, fases que faltan, ≈mediana, p80, n y nivel).
+    #[test]
+    fn c2_linea_exacta_a_los_20_min() {
+        let (t, c) = tarea(EN_CURSO, &[(0, "start")]);
+        let p = calcular(&t, &c, &historico(), &Commits::vacios(), T0);
+        let p = p.unwrap_or_else(|| unreachable!());
+        assert_eq!(
+            linea(&p, T0 + 20 * 60, None),
+            "TASK-007 · implementacion · fases que faltan: rondas, cierre · faltan ≈50 min (p80 65) · n=6 repo"
+        );
+        // Recien empezada: todo lo de las parecidas (imp + 15).
+        assert_eq!(
+            linea(&p, T0, None),
+            "TASK-007 · implementacion · fases que faltan: rondas, cierre · faltan ≈70 min (p80 85) · n=6 repo"
+        );
+    }
+
+    /// C5: sin parecidas, `sin estimacion (n=N)`.
+    #[test]
+    fn c5_sin_parecidas() {
+        let (t, c) = tarea(EN_CURSO, &[(0, "start")]);
+        let h = Historico {
+            muestras: historico().muestras[..3].to_vec(),
+            ..Historico::default()
+        };
+        let p = calcular(&t, &c, &h, &Commits::vacios(), T0);
+        let p = p.unwrap_or_else(|| unreachable!());
+        assert_eq!(
+            linea(&p, T0 + 60, None),
+            "TASK-007 · implementacion · fases que faltan: rondas, cierre · faltan: sin estimacion (n=3)"
+        );
+    }
+
+    /// C6: pasado lo de las parecidas, la cota `≥X` con el aviso; mas de 45
+    /// min sin eventos, «sin actividad registrada» y el reloj con el tope.
+    #[test]
+    fn c6_desbordada_y_sin_actividad() {
+        let (t, c) = tarea(
+            EN_CURSO,
+            &[(0, "start"), (40 * 60, "pausa"), (41 * 60, "reanudar")],
+        );
+        let p = calcular(&t, &c, &historico(), &Commits::vacios(), T0 + 41 * 60);
+        let p = p.unwrap_or_else(|| unreachable!());
+        assert_eq!(p.transcurrido, 40.0);
+        // 40 + 30 = 70: solo una parecida (80) pasa de 70: desbordada; la
+        // cota es lo de las fases siguientes (10 + 5).
+        assert_eq!(
+            linea(&p, T0 + 71 * 60, None),
+            "TASK-007 · implementacion · fases que faltan: rondas, cierre · faltan ≥15 min (fase en curso por encima de sus parecidas) · n=6 repo"
+        );
+        // 127 min sin nada: el reloj suma 45 y lo dice.
+        let l = linea(&p, T0 + (41 + 127) * 60, None);
+        assert!(
+            l.ends_with("(fase en curso por encima de sus parecidas) · n=6 repo · sin actividad registrada hace 127 min"),
+            "{l}"
+        );
+        assert_eq!(transcurrido(&p, T0 + (41 + 127) * 60, None).0, 85.0);
+        // Una actividad de `logs/HEAD` a los 100 min parte el hueco: 45 + 27.
+        let (t2, abierto) = transcurrido(&p, T0 + (41 + 127) * 60, Some(T0 + (41 + 100) * 60));
+        assert_eq!((t2, abierto), (40.0 + 45.0 + 27.0, 27.0));
+        // Una actividad del futuro no cuenta.
+        assert_eq!(
+            transcurrido(&p, T0 + 50 * 60, Some(T0 + 99_999)),
+            transcurrido(&p, T0 + 50 * 60, None)
+        );
+    }
+
+    /// Pausada: el reloj no corre y la linea lo dice.
+    #[test]
+    fn pausada_congela() {
+        let (t, c) = tarea(EN_CURSO, &[(0, "start"), (20 * 60, "pausa")]);
+        let p = calcular(&t, &c, &historico(), &Commits::vacios(), T0 + 20 * 60);
+        let p = p.unwrap_or_else(|| unreachable!());
+        let a = linea(&p, T0 + 20 * 60, None);
+        assert_eq!(a, linea(&p, T0 + 500 * 60, None));
+        assert_eq!(
+            a,
+            "TASK-007 · pausada en implementacion · fases que faltan: rondas, cierre · faltan ≈50 min (p80 65) · n=6 repo"
+        );
+    }
+
+    /// C10: avisos con control, ANSI o bidi salen saneados y acotados; el
+    /// nivel tambien.
+    #[test]
+    fn c10_avisos_saneados_y_acotados() {
+        let mut p = ejemplo("TASK-001");
+        p.avisos = vec![
+            format!(
+                "rojo \u{1b}[31mpeligro\u{1b}[0m \u{202e}al reves\u{2066} {}",
+                "x".repeat(500)
+            ),
+            "linea\nnueva\r\ttab".to_owned(),
+            "tercero no sale".to_owned(),
+        ];
+        p.nivel = format!("repo\u{200b}\u{7}{}", "n".repeat(100));
+        let l = linea(&p, 1_000, None);
+        assert!(
+            !l.chars().any(|c| c.is_control()
+                || matches!(c, '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{FEFF}')),
+            "{l:?}"
+        );
+        assert!(!l.contains('\u{1b}') && !l.contains("tercero"), "{l}");
+        assert!(l.contains("linea nueva tab"), "{l}");
+        assert!(l.chars().count() <= 400, "{}", l.chars().count());
+        let partes: Vec<&str> = l.split(SEP).collect();
+        assert!(
+            partes
+                .iter()
+                .all(|x| x.chars().count() <= MAX_AVISO_LINEA.max(MAX_NIVEL + 10)),
+            "{l}"
+        );
+    }
+
+    #[test]
+    fn otras_formas() {
+        let p = ejemplo("TASK-001");
+        assert_eq!(
+            con_antiguedad("L", &p, p.calculada + 7 * 60 + 59),
+            "L · (dato de hace 7 min)"
+        );
+        assert_eq!(ilegible("TASK-001"), "TASK-001 · progreso ilegible");
+        assert_eq!(con_mas("L", 0), "L");
+        assert_eq!(con_mas("L", 2), "L · +2 en curso");
+        assert_eq!(SIN_TAREA, "tc2 · sin tarea en curso");
+    }
+}
diff --git a/crates/tc2/src/progreso/mod.rs b/crates/tc2/src/progreso/mod.rs
new file mode 100644
index 0000000..85773a5
--- /dev/null
+++ b/crates/tc2/src/progreso/mod.rs
@@ -0,0 +1,552 @@
+//! Progreso en vivo de una tarea (F3-12; ADR F1-17, puntos 5, 7 y 8, con la
+//! adenda de TASK-084).
+//!
+//! - [`fichero`]: `progreso/TASK-NNN.json` bajo `.git/taskcodetwo/`, con el
+//!   anterior en `TASK-NNN.anterior.json`.
+//! - [`calculo`]: `calcular`, puro, sobre la estimacion de TASK-083.
+//! - [`linea`]: la linea de la status line y de `status --seguir`, pura.
+//! - [`orden_estado`]: `tc2 estado --stdin`, la status line de Claude Code.
+//!   No recibe [`crate::git::Git`]: sin red ni subprocesos por
+//!   construccion. Busca el repo subiendo desde `workspace.current_dir` del
+//!   JSON (o `cwd`, o la carpeta del proceso) hasta un `.git` (carpeta, o
+//!   fichero `gitdir:` con `commondir`), y lee uno o pocos ficheros. Dentro
+//!   de `.taskcodetwo/worktrees/<ID>` la tarea es esa; en el principal, la
+//!   de calculo mas reciente y `+N en curso`. Siempre una linea y codigo 0.
+
+pub mod calculo;
+pub mod fichero;
+pub mod linea;
+
+use std::io::Read;
+use std::path::{Path, PathBuf};
+
+pub use calculo::calcular;
+pub use fichero::{Lectura, Progreso, Punto};
+
+use crate::Salida;
+use crate::git::Git;
+
+/// Orden de la status line (`tc2 estado --stdin`, la de
+/// [`distribucion::STATUS_LINE`]).
+pub const ESTADO: &str = "estado";
+
+/// Bytes como mucho del JSON de la status line que se leen de stdin.
+pub const LIMITE_STDIN: u64 = 1024 * 1024;
+
+/// Bytes como mucho de un fichero `.git` o `commondir`.
+const LIMITE_GIT: u64 = 4096;
+
+/// Un repo encontrado sin lanzar git.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Repo {
+    /// Carpeta git del worktree (`.git`, o la de `gitdir:`).
+    pub gitdir: PathBuf,
+    /// Carpeta comun de git (la `.git` del principal).
+    pub comun: PathBuf,
+    /// La tarea si el worktree es `.taskcodetwo/worktrees/<ID>`.
+    pub id: Option<String>,
+}
+
+/// Texto de un fichero pequeno, sin el salto final.
+fn texto_corto(p: &Path) -> Option<String> {
+    let mut s = String::new();
+    std::fs::File::open(p)
+        .ok()?
+        .take(LIMITE_GIT + 1)
+        .read_to_string(&mut s)
+        .ok()?;
+    (s.len() as u64 <= LIMITE_GIT).then(|| s.trim_end_matches(['\r', '\n']).to_owned())
+}
+
+/// `rel` relativo a `base` si no es absoluto.
+fn junto(base: &Path, rel: &str) -> PathBuf {
+    let r = Path::new(rel);
+    if r.is_absolute() {
+        r.to_path_buf()
+    } else {
+        base.join(r)
+    }
+}
+
+/// El ID si `raiz` es `<principal>/.taskcodetwo/worktrees/<ID>`.
+fn id_de_worktree(raiz: &Path) -> Option<String> {
+    let id = raiz.file_name()?.to_str()?;
+    let padre = raiz.parent()?;
+    let abuelo = padre.parent()?;
+    let carpeta = format!(
+        "{}/{}",
+        abuelo.file_name()?.to_str()?,
+        padre.file_name()?.to_str()?
+    );
+    (carpeta == crate::huella::CARPETA_WORKTREES && crate::git::es_id(id)).then(|| id.to_owned())
+}
+
+/// Busca el repo de `desde` subiendo por sus carpetas, sin lanzar git.
+pub fn buscar(desde: &Path) -> Option<Repo> {
+    for d in desde.ancestors() {
+        let punto_git = d.join(".git");
+        let Ok(m) = std::fs::metadata(&punto_git) else {
+            continue;
+        };
+        if m.is_dir() {
+            return Some(Repo {
+                gitdir: punto_git.clone(),
+                comun: punto_git,
+                id: None,
+            });
+        }
+        let t = texto_corto(&punto_git)?;
+        let gitdir = junto(d, t.strip_prefix("gitdir:")?.trim());
+        let comun = texto_corto(&gitdir.join("commondir"))
+            .map_or_else(|| gitdir.clone(), |c| junto(&gitdir, c.trim()));
+        return Some(Repo {
+            gitdir,
+            comun,
+            id: id_de_worktree(d),
+        });
+    }
+    None
+}
+
+/// Segundos desde 1970 de la ultima modificacion de `p`.
+fn mtime(p: &Path) -> Option<u64> {
+    std::fs::metadata(p)
+        .ok()?
+        .modified()
+        .ok()?
+        .duration_since(std::time::UNIX_EPOCH)
+        .ok()
+        .map(|d| d.as_secs())
+}
+
+/// La ultima actividad que se ve sin lanzar git: el `logs/HEAD` del
+/// worktree (con `core.logAllRefUpdates`, por defecto en un repo con
+/// worktree, cada commit lo toca).
+fn actividad(gitdir: &Path) -> Option<u64> {
+    mtime(&gitdir.join("logs").join("HEAD"))
+}
+
+/// La linea de una tarea leida.
+fn linea_de(l: &Lectura, id: &str, gitdir: &Path, ahora: u64) -> Option<(u64, String)> {
+    match l {
+        Lectura::Ninguno => None,
+        Lectura::Bueno(p) => Some((p.calculada, linea::linea(p, ahora, actividad(gitdir)))),
+        Lectura::Anterior(p) => Some((
+            p.calculada,
+            linea::con_antiguedad(&linea::linea(p, ahora, actividad(gitdir)), p, ahora),
+        )),
+        Lectura::Ilegible => Some((0, linea::ilegible(id))),
+    }
+}
+
+/// La linea de progreso para la carpeta `dir` a las `ahora`.
+pub fn linea_en(dir: &Path, ahora: u64) -> String {
+    let Some(repo) = buscar(dir) else {
+        return linea::SIN_TAREA.to_owned();
+    };
+    let carpeta = fichero::carpeta(&repo.comun);
+    if let Some(id) = &repo.id {
+        return linea_de(&fichero::leer(&carpeta, id), id, &repo.gitdir, ahora)
+            .map_or_else(|| linea::SIN_TAREA.to_owned(), |(_, l)| l);
+    }
+    // En el principal (o en `tareas/`): la de calculo mas reciente.
+    let ids = fichero::ids(&carpeta);
+    let mejor = ids
+        .iter()
+        .filter_map(|id| {
+            let gitdir = repo.comun.join("worktrees").join(id);
+            linea_de(&fichero::leer(&carpeta, id), id, &gitdir, ahora)
+        })
+        .max_by_key(|(calculada, _)| *calculada);
+    match mejor {
+        Some((_, l)) => linea::con_mas(&l, ids.len().saturating_sub(1)),
+        None => linea::SIN_TAREA.to_owned(),
+    }
+}
+
+/// La carpeta que dice el JSON de la status line: `workspace.current_dir`
+/// o, si no viene, `cwd` (documentacion de Claude Code: los dos llevan lo
+/// mismo).
+pub fn carpeta_del_json(json: &str) -> Option<PathBuf> {
+    // Sin la marca de orden de bytes que anaden algunos escritores de
+    // Windows (`StandardInput` de .NET), que serde_json no acepta.
+    let json = json.trim_start_matches('\u{feff}');
+    let v: serde_json::Value = serde_json::from_str(json).ok()?;
+    v.pointer("/workspace/current_dir")
+        .and_then(serde_json::Value::as_str)
+        .or_else(|| v.get("cwd").and_then(serde_json::Value::as_str))
+        .filter(|s| !s.is_empty())
+        .map(PathBuf::from)
+}
+
+/// `tc2 estado --stdin` con lo leido de stdin, la carpeta del proceso como
+/// respaldo y la hora. Sin `Git` en la firma: sin red ni subprocesos.
+pub fn orden_estado(stdin: &str, cwd: Option<&Path>, ahora: u64) -> String {
+    match carpeta_del_json(stdin).or_else(|| cwd.map(Path::to_path_buf)) {
+        Some(d) => linea_en(&d, ahora),
+        None => linea::SIN_TAREA.to_owned(),
+    }
+}
+
+/// Que hizo un refresco.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Refresco {
+    /// Escribio el progreso de la tarea.
+    Escrito,
+    /// La tarea no esta en curso ni en revision (o la deja quien la lanza):
+    /// borro su progreso.
+    Borrado,
+}
+
+/// Nombre de la operacion en los informes de error.
+const OP: &str = "progreso";
+
+/// La carpeta de progreso del repo de `dir` (localiza el principal: lanza
+/// git).
+fn carpeta_de(g: &Git, dir: &Path) -> Result<(PathBuf, PathBuf), String> {
+    let principal = crate::git::localizar_principal(g, dir).map_err(|i| i.linea())?;
+    let comun = crate::git::estado::dir_comun(g, &principal, OP).map_err(|i| i.linea())?;
+    Ok((principal, fichero::carpeta(&comun)))
+}
+
+/// `refrescar(g, dir, id, ahora)`: recalcula y escribe el progreso de `id`
+/// tras una transicion, con el estado que conoce el clon (sin `fetch`):
+/// todos los `tarea.md` y un `git log` de `develop` (O(N): solo en las
+/// transiciones). Si la tarea no esta en curso ni en revision, borra su
+/// progreso. Lo llaman las transiciones (`lib.rs`, `de_una_tarea`); `review`
+/// y `finish` (F6-2, F7-1) lo llamaran igual cuando existan.
+pub fn refrescar(g: &Git, dir: &Path, id: &str, ahora: u64) -> Result<Refresco, String> {
+    let (principal, carpeta) = carpeta_de(g, dir)?;
+    let vista = crate::estado::leer(g, dir).map_err(|i| i.linea())?;
+    let texto = |f: crate::estado::Fallo| match f {
+        crate::estado::Fallo::Rechazada { motivo, .. } => motivo,
+        crate::estado::Fallo::Error(d) => d.texto(crate::i18n::Idioma::Es),
+    };
+    let (t, c) = match crate::estado::tarea_de(&vista, id) {
+        Ok(x) => x,
+        // No existe, es de la 0.16 o no se entiende: no hay progreso.
+        Err(crate::estado::Fallo::Rechazada { .. }) => return quitar_en(&carpeta, id),
+        Err(f) => return Err(texto(f)),
+    };
+    if calculo::fase_de(&t.estado).is_none() {
+        return quitar_en(&carpeta, id);
+    }
+    let leidas =
+        crate::estado::tareas(&vista, &crate::estado::Estado::rutas(&vista)).map_err(texto)?;
+    let commits = crate::estimacion::commits_de(g, &principal);
+    let h = crate::estimacion::historico(&leidas, &commits);
+    match calcular(&t, &c, &h, &commits, ahora) {
+        Some(p) => fichero::escribir(&carpeta, &p)
+            .map(|()| Refresco::Escrito)
+            .map_err(|e| e.to_string()),
+        None => quitar_en(&carpeta, id),
+    }
+}
+
+fn quitar_en(carpeta: &Path, id: &str) -> Result<Refresco, String> {
+    fichero::borrar(carpeta, id)
+        .map(|()| Refresco::Borrado)
+        .map_err(|e| e.to_string())
+}
+
+/// Clave de la linea de una transicion cuyo progreso no se pudo escribir.
+pub const CLAVE: &str = "progreso";
+
+/// Valor de [`CLAVE`].
+pub const NO_ESCRITO: &str = "no-escrito";
+
+/// Escribe `p`, ya calculado, en el clon de `dir` (lo usa `start`, que lo
+/// calcula dentro de la toma con el historico que ya leyo).
+pub fn escribir_en(g: &Git, dir: &Path, p: &Progreso) -> Result<(), String> {
+    let (_, carpeta) = carpeta_de(g, dir)?;
+    fichero::escribir(&carpeta, p).map_err(|e| e.to_string())
+}
+
+/// Borra el progreso de `id` en el clon de `dir` (tras `descartar`,
+/// `liberar` o `reasignar`: la tarea deja de estar en curso aqui).
+pub fn quitar(g: &Git, dir: &Path, id: &str) -> Result<Refresco, String> {
+    let (_, carpeta) = carpeta_de(g, dir)?;
+    quitar_en(&carpeta, id)
+}
+
+/// Cada cuanto mira `tc2 status --seguir`.
+pub const ESPERA_SEGUIR: std::time::Duration = std::time::Duration::from_secs(2);
+
+/// `tc2 status --seguir` (F1-17, punto 8): la misma lectura que la status
+/// line para `dir`, repetida cada `espera`. Escribe la linea al empezar y
+/// otra solo cuando cambia el texto (porque se reescribio el fichero o
+/// porque corrio el reloj), sin ANSI. Para cuando `parar()` es verdadero
+/// (despues de cada mirada) o cuando no se puede escribir (stdout cerrado:
+/// `Ok`). `reloj` y `parar` se inyectan en las pruebas; en el binario son la
+/// hora del sistema y nunca (lo corta Ctrl+C).
+pub fn seguir(
+    dir: &Path,
+    out: &mut dyn std::io::Write,
+    reloj: &dyn Fn() -> u64,
+    parar: &dyn Fn() -> bool,
+    espera: std::time::Duration,
+) -> std::io::Result<()> {
+    let mut ultima: Option<String> = None;
+    loop {
+        let l = linea_en(dir, reloj());
+        if ultima.as_deref() != Some(l.as_str()) {
+            if writeln!(out, "{l}").and_then(|()| out.flush()).is_err() {
+                return Ok(());
+            }
+            ultima = Some(l);
+        }
+        if parar() {
+            return Ok(());
+        }
+        std::thread::sleep(espera);
+    }
+}
+
+/// `tc2 estado --stdin` en el proceso: lee stdin (con tope), la carpeta
+/// actual y el reloj. Siempre codigo 0 y stderr vacio.
+pub fn estado_del_proceso() -> Salida {
+    let mut entrada = String::new();
+    let _ = std::io::stdin()
+        .lock()
+        .take(LIMITE_STDIN)
+        .read_to_string(&mut entrada);
+    let cwd = std::env::current_dir().ok();
+    let l = orden_estado(&entrada, cwd.as_deref(), crate::identidad::ahora());
+    Salida {
+        codigo: 0,
+        stdout: format!("{l}\n"),
+        stderr: String::new(),
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn dir(nombre: &str) -> PathBuf {
+        let d = std::env::temp_dir()
+            .join("tc2-tests")
+            .join(format!("progreso-busca-{nombre}-{}", std::process::id()));
+        let _ = std::fs::remove_dir_all(&d);
+        d
+    }
+
+    /// La busqueda sin git: `.git` carpeta o fichero (`gitdir:` relativo o
+    /// absoluto, con `commondir`), desde una subcarpeta; en un worktree de
+    /// tarea, su ID.
+    #[test]
+    fn busca_sin_git() -> std::io::Result<()> {
+        let raiz = dir("busca");
+        let principal = raiz.join("p");
+        let comun = principal.join(".git");
+        std::fs::create_dir_all(principal.join("src").join("a"))?;
+        std::fs::create_dir_all(&comun)?;
+        let r = buscar(&principal.join("src").join("a"));
+        assert_eq!(
+            r,
+            Some(Repo {
+                gitdir: comun.clone(),
+                comun: comun.clone(),
+                id: None
+            })
+        );
+        // Worktree de tarea con `gitdir:` absoluto y `commondir` relativo.
+        let wt = principal
+            .join(".taskcodetwo")
+            .join("worktrees")
+            .join("TASK-012");
+        let admin = comun.join("worktrees").join("TASK-012");
+        std::fs::create_dir_all(wt.join("sub"))?;
+        std::fs::create_dir_all(&admin)?;
+        std::fs::write(wt.join(".git"), format!("gitdir: {}\n", admin.display()))?;
+        std::fs::write(admin.join("commondir"), "../..\n")?;
+        let r = buscar(&wt.join("sub")).unwrap_or_else(|| unreachable!());
+        assert_eq!(r.gitdir, admin);
+        assert_eq!(r.comun, admin.join("../.."));
+        assert_eq!(r.id.as_deref(), Some("TASK-012"));
+        // `tareas/` (gitdir relativo, sin commondir): no es de una tarea.
+        let tareas = principal.join("tareas");
+        std::fs::create_dir_all(&tareas)?;
+        std::fs::write(tareas.join(".git"), "gitdir: ../.git/worktrees/tareas")?;
+        let r = buscar(&tareas).unwrap_or_else(|| unreachable!());
+        assert_eq!(r.id, None);
+        assert_eq!(r.gitdir, tareas.join("../.git/worktrees/tareas"));
+        // Un worktree con nombre que no es un ID, o fuera de la carpeta.
+        let otro = principal.join(".taskcodetwo").join("worktrees").join("x");
+        std::fs::create_dir_all(&otro)?;
+        std::fs::write(otro.join(".git"), format!("gitdir: {}", admin.display()))?;
+        assert_eq!(buscar(&otro).and_then(|r| r.id), None);
+        // Un `.git` fichero que no es `gitdir:`: nada.
+        let raro = raiz.join("raro");
+        std::fs::create_dir_all(&raro)?;
+        std::fs::write(raro.join(".git"), "basura")?;
+        assert_eq!(buscar(&raro), None);
+        let _ = std::fs::remove_dir_all(&raiz);
+        Ok(())
+    }
+
+    /// La orden es la que reconoce `tc2 desinstalar` (F1-12 D1a).
+    #[test]
+    fn la_orden_es_la_de_la_distribucion() {
+        assert_eq!(format!("tc2 {ESTADO} --stdin"), distribucion::STATUS_LINE);
+    }
+
+    #[test]
+    fn carpeta_del_json_con_respaldo() {
+        assert_eq!(
+            carpeta_del_json(r#"{"workspace":{"current_dir":"/a/b"},"cwd":"/c"}"#),
+            Some(PathBuf::from("/a/b"))
+        );
+        assert_eq!(
+            carpeta_del_json(r#"{"cwd":"/c"}"#),
+            Some(PathBuf::from("/c"))
+        );
+        assert_eq!(
+            carpeta_del_json("\u{feff}{\"cwd\":\"/c\"}"),
+            Some(PathBuf::from("/c"))
+        );
+        for malo in [
+            "",
+            "x",
+            "{}",
+            r#"{"workspace":{"current_dir":7}}"#,
+            r#"{"cwd":""}"#,
+        ] {
+            assert_eq!(carpeta_del_json(malo), None, "{malo}");
+        }
+    }
+
+    /// C7: fuera de un repo, sin tarea; el JSON que no vale usa la carpeta
+    /// del proceso.
+    #[test]
+    fn c7_fuera_de_un_repo() {
+        let d = dir("fuera");
+        let _ = std::fs::create_dir_all(&d);
+        // La carpeta temporal no esta dentro de un repo (si lo estuviera, el
+        // test no probaria nada: se comprueba).
+        if buscar(&d).is_none() {
+            let json = format!(
+                "{{\"workspace\":{{\"current_dir\":{}}}}}",
+                serde_json::Value::from(d.display().to_string())
+            );
+            assert_eq!(orden_estado(&json, None, 0), linea::SIN_TAREA);
+            assert_eq!(orden_estado("no es json", Some(&d), 0), linea::SIN_TAREA);
+        }
+        assert_eq!(orden_estado("", None, 0), linea::SIN_TAREA);
+        let _ = std::fs::remove_dir_all(&d);
+    }
+
+    /// C9 (biblioteca): `seguir` imprime la linea inicial, otra al
+    /// reescribirse el fichero, ninguna si no cambia, y termina cuando
+    /// `parar()` es verdadero; con stdout cerrado, termina bien.
+    #[test]
+    fn c9_seguir_con_reloj_y_parar_inyectados() -> std::io::Result<()> {
+        let raiz = dir("seguir");
+        let principal = raiz.join("p");
+        let comun = principal.join(".git");
+        std::fs::create_dir_all(&comun)?;
+        let carpeta = fichero::carpeta(&comun);
+        let p = fichero::tests::ejemplo("TASK-001");
+        fichero::escribir(&carpeta, &p)?;
+        let vueltas = std::cell::Cell::new(0);
+        let parar = || {
+            vueltas.set(vueltas.get() + 1);
+            if vueltas.get() == 1 {
+                let mut q = fichero::tests::ejemplo("TASK-001");
+                q.fase = "rondas".to_owned();
+                q.faltan = vec!["cierre".to_owned()];
+                let _ = fichero::escribir(&carpeta, &q);
+            }
+            vueltas.get() >= 3
+        };
+        let mut out = Vec::new();
+        seguir(
+            &principal,
+            &mut out,
+            &|| 1_000,
+            &parar,
+            std::time::Duration::ZERO,
+        )?;
+        let texto = String::from_utf8_lossy(&out).into_owned();
+        let lineas: Vec<&str> = texto.lines().collect();
+        assert_eq!(vueltas.get(), 3);
+        assert_eq!(lineas.len(), 2, "{texto}");
+        assert!(
+            lineas[0].starts_with("TASK-001 · implementacion · "),
+            "{texto}"
+        );
+        assert!(
+            lineas[1].starts_with("TASK-001 · rondas · fases que faltan: cierre · "),
+            "{texto}"
+        );
+        assert!(!texto.contains('\u{1b}'));
+        // Stdout cerrado: termina bien a la primera.
+        struct Cerrado;
+        impl std::io::Write for Cerrado {
+            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
+                Err(std::io::ErrorKind::BrokenPipe.into())
+            }
+            fn flush(&mut self) -> std::io::Result<()> {
+                Ok(())
+            }
+        }
+        seguir(
+            &principal,
+            &mut Cerrado,
+            &|| 1_000,
+            &|| false,
+            std::time::Duration::ZERO,
+        )?;
+        let _ = std::fs::remove_dir_all(&raiz);
+        Ok(())
+    }
+
+    /// La linea del principal: la de calculo mas reciente y `+N en curso`;
+    /// en un worktree de tarea, la suya.
+    #[test]
+    fn principal_y_worktree() -> std::io::Result<()> {
+        let raiz = dir("principal");
+        let principal = raiz.join("p");
+        let comun = principal.join(".git");
+        std::fs::create_dir_all(&comun)?;
+        let carpeta = fichero::carpeta(&comun);
+        assert_eq!(linea_en(&principal, 2_000), linea::SIN_TAREA);
+        let mut a = fichero::tests::ejemplo("TASK-001");
+        a.calculada = 1_000;
+        let mut b = fichero::tests::ejemplo("TASK-002");
+        b.calculada = 1_500;
+        fichero::escribir(&carpeta, &a)?;
+        fichero::escribir(&carpeta, &b)?;
+        let l = linea_en(&principal, 1_500);
+        assert!(
+            l.starts_with("TASK-002 · ") && l.ends_with(" · +1 en curso"),
+            "{l}"
+        );
+        let wt = principal
+            .join(".taskcodetwo")
+            .join("worktrees")
+            .join("TASK-001");
+        std::fs::create_dir_all(&wt)?;
+        std::fs::write(wt.join(".git"), "gitdir: ../../../.git/worktrees/TASK-001")?;
+        std::fs::create_dir_all(comun.join("worktrees").join("TASK-001"))?;
+        std::fs::write(
+            comun.join("worktrees").join("TASK-001").join("commondir"),
+            "../..",
+        )?;
+        let l = linea_en(&wt, 1_000);
+        assert!(
+            l.starts_with("TASK-001 · ") && !l.contains("en curso ·"),
+            "{l}"
+        );
+        // La tarea de otro worktree sin fichero: sin tarea.
+        let wt3 = principal
+            .join(".taskcodetwo")
+            .join("worktrees")
+            .join("TASK-003");
+        std::fs::create_dir_all(&wt3)?;
+        std::fs::write(wt3.join(".git"), "gitdir: ../../../.git/worktrees/TASK-003")?;
+        assert_eq!(linea_en(&wt3, 1_000), linea::SIN_TAREA);
+        let _ = std::fs::remove_dir_all(&raiz);
+        Ok(())
+    }
+}
diff --git a/crates/tc2/src/tablero.rs b/crates/tc2/src/tablero.rs
index 3f7b925..4344e1e 100644
--- a/crates/tc2/src/tablero.rs
+++ b/crates/tc2/src/tablero.rs
@@ -8,6 +8,9 @@
 //! traido con un aviso en stderr y codigo 0). No escriben nada mas que
 //! `origin/taskcodetwo/tareas` y la marca de sincronizacion.
 //!
+//! - `status --seguir` (F3-12): la linea de progreso de la tarea en curso,
+//!   repetida al cambiar; la escribe [`crate::progreso::seguir`] desde
+//!   `main.rs`. Aqui solo se rechaza con `--json`.
 //! - `status [--json]`: una linea de equipo; no consulta la plataforma. Con
 //!   `--json`, `status ok en-diseno=... en-curso=... en-revision=... listas=...
 //!   commit=... remoto=... sincronizado=... pendientes=N [tareas=...]`. En
@@ -49,6 +52,9 @@ use crate::situacion::{self, Ficha, Quien, Situacion};
 pub const BOARD: &str = "board";
 /// Nombre de `status`.
 pub const STATUS: &str = "status";
+/// `status --seguir` (F3-12): la linea de progreso, repetida al cambiar
+/// ([`crate::progreso::seguir`], desde `main.rs`).
+pub const SEGUIR: &str = "--seguir";
 
 /// Tareas por grupo en la linea de `status` (las demas, `+N`).
 pub const K_STATUS: usize = 3;
@@ -526,6 +532,19 @@ pub fn cruzar(t: &Tablero, prs: &[PrAbierto]) -> Cruce {
 /// `status [--json]` (sin `--json`, que quita `lib.rs`). No consulta nunca
 /// la plataforma.
 pub fn orden_status(args: &[String], json: bool, idioma: Idioma, e: &Entorno<'_>) -> Salida {
+    // F3-12: `status --seguir` lo atiende `main.rs` (escribe a trozos); aqui
+    // solo llega con `--json` o desde la biblioteca, y se rechaza.
+    if args.iter().any(|a| a == SEGUIR) {
+        let motivo = if json {
+            "seguir-con-json"
+        } else {
+            "seguir-sin-terminal"
+        };
+        return Informe::nuevo(STATUS, Resultado::Uso)
+            .con("motivo", motivo)
+            .diagnostico(&txt::USO_STATUS)
+            .salida_en(json, idioma);
+    }
     if !args.is_empty() {
         return Informe::nuevo(STATUS, Resultado::Uso)
             .con("motivo", "argumentos-de-mas")
diff --git a/crates/tc2/src/toma/mod.rs b/crates/tc2/src/toma/mod.rs
index 1dd0644..dcaeaeb 100644
--- a/crates/tc2/src/toma/mod.rs
+++ b/crates/tc2/src/toma/mod.rs
@@ -203,6 +203,15 @@ fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
             if !leido {
                 i = i.con("historia", crate::estimacion::SIN_LOG);
             }
+            // El progreso en vivo (F3-12), con lo que ya calculo la toma.
+            // Un fallo no cambia el resultado: `progreso=no-escrito`, y
+            // `lib.rs` lo avisa por stderr.
+            if i.resultado.es_exito()
+                && let Some(p) = op.progreso()
+                && crate::progreso::escribir_en(g, &principal, &p).is_err()
+            {
+                i = i.con(crate::progreso::CLAVE, crate::progreso::NO_ESCRITO);
+            }
             Ok(i)
         }
         // Entre la lectura y el push paso a ser suya y empezada.
diff --git a/crates/tc2/src/toma/operacion.rs b/crates/tc2/src/toma/operacion.rs
index 8de9035..b4ff72b 100644
--- a/crates/tc2/src/toma/operacion.rs
+++ b/crates/tc2/src/toma/operacion.rs
@@ -60,6 +60,7 @@ pub struct Tomar {
     pendientes: RefCell<Vec<String>>,
     rama: RefCell<Option<String>>,
     estimacion: RefCell<Option<Estimacion>>,
+    progreso: RefCell<Option<crate::progreso::Progreso>>,
 }
 
 impl Tomar {
@@ -80,9 +81,18 @@ impl Tomar {
             pendientes: RefCell::new(Vec::new()),
             rama: RefCell::new(None),
             estimacion: RefCell::new(None),
+            progreso: RefCell::new(None),
         }
     }
 
+    /// El progreso en vivo de la tarea tomada en el ultimo intento (F3-12),
+    /// calculado con el mismo historico y los mismos commits que la
+    /// estimacion: `start` lo escribe sin volver a leer el estado ni la
+    /// historia (R1-05 y R1-06 de TASK-083).
+    pub fn progreso(&self) -> Option<crate::progreso::Progreso> {
+        self.progreso.borrow().clone()
+    }
+
     /// Los commits de `develop` para el historico de la estimacion (un
     /// `git log` leido antes de la transaccion, F3-13).
     #[must_use]
@@ -142,6 +152,7 @@ impl Operacion for Tomar {
         self.pendientes.borrow_mut().clear();
         *self.rama.borrow_mut() = None;
         *self.estimacion.borrow_mut() = None;
+        *self.progreso.borrow_mut() = None;
         let ruta = format!("{}/tarea.md", self.id);
         let (mut t, mut cuerpo) = tarea_de(estado, &self.id)?;
         let rama = t
@@ -217,6 +228,8 @@ impl Operacion for Tomar {
             decidido_por: self.decidido_por,
             version: crate::VERSION,
         });
+        *self.progreso.borrow_mut() =
+            crate::progreso::calcular(&t, &cuerpo, &h, &self.commits, self.ahora);
         Ok(vec![Cambio::nuevo(ruta, t.escribir(&cuerpo))])
     }
 }
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-084/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-084/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-084/tarea.md                                    | 36 ++++++++++++++++++++++++++++++++++--
 3 files changed, 34 insertions(+), 2 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/progreso.rs"
````

````
crates/tc2/tests/progreso.rs | 638 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 638 insertions(+)
````
