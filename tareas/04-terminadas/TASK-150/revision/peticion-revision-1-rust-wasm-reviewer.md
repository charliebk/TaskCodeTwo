# Peticion de revision — TASK-150 (ronda 1)

- Tarea: TASK-150 — Auditoria 4D: utilidades comunes de E-S y tiempo (4 IMPORTANTE)
- Rama revisada: fix/task-150-auditoria-4d-utilidades-comunes-de-e-s-y
- Rama base: develop
- Commit revisado (HEAD): 8347d882515db613e63d96e66e2e2f147fad004f
- Fecha: 2026-11-25
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-150 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-150/tarea.md
- tareas/03-en-revision/TASK-150/planificacion/
- crates/tc2/src/actualizar_cli.rs
- crates/tc2/src/alta.rs
- crates/tc2/src/ciclo/operacion.rs
- crates/tc2/src/ciclo/prueba.rs
- crates/tc2/src/confianza/mod.rs
- crates/tc2/src/desinstalar/mod.rs
- crates/tc2/src/escritura.rs
- crates/tc2/src/estado/frescura.rs
- crates/tc2/src/estimacion/mod.rs
- crates/tc2/src/fecha.rs
- crates/tc2/src/hallazgos/orden.rs
- crates/tc2/src/huella.rs
- crates/tc2/src/identidad.rs
- crates/tc2/src/lectura.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/plan/aprobar.rs
- crates/tc2/src/plan/operacion.rs
- crates/tc2/src/proceso.rs
- crates/tc2/src/progreso/calculo.rs
- crates/tc2/src/progreso/fichero.rs
- crates/tc2/src/progreso/mod.rs
- crates/tc2/src/revision/operacion.rs
- crates/tc2/src/seleccion/orden.rs
- crates/tc2/src/toma/mod.rs
- crates/tc2/src/toma/operacion.rs
- crates/tc2/src/verificacion/ejecutar.rs
- crates/tc2/tests/comun/remoto.rs
- crates/tc2/tests/comun/tareas.rs
- crates/tc2/tests/identidades.rs
- crates/tc2/tests/soporte.rs
- crates/tc2/tests/toma.rs
- xtask/src/main.rs

## Commits a revisar (git log develop..HEAD)

````
8347d88 chore(TASK-150): resultado y autorrevision
689cb93 fix(TASK-150): A8-03 fecha UTC una sola vez en tc2::fecha
27b9d9e fix(TASK-150): A9-01 una sola espera con plazo en crate::proceso
6671650 fix(TASK-150): A8-17 un solo leer con tope en crate::lectura
73996b0 fix(TASK-150): A8-04 una sola escritura atomica en crate::escritura
770981c chore(TASK-150): tarea en curso
````

## Diff de tu dominio (32 fichero(s) de 32; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/actualizar_cli.rs b/crates/tc2/src/actualizar_cli.rs
index 66a28e3..88f98f5 100644
--- a/crates/tc2/src/actualizar_cli.rs
+++ b/crates/tc2/src/actualizar_cli.rs
@@ -503,19 +503,21 @@ fn ultima_linea(bytes: &[u8]) -> Option<String> {
         .map(|l| l.chars().take(300).collect())
 }
 
-/// Lanza `cmd` sin entrada y espera como mucho `limite`. `Ok(None)` si no
-/// termino a tiempo (se mata). stdout y stderr se leen en hilos aparte, hasta
-/// 1 MiB cada uno, para que un hijo que escribe mucho no se bloquee.
+/// Lanza `cmd` sin entrada, en un grupo propio, y espera como mucho `limite`
+/// ([`crate::proceso::esperar_con_plazo`]). `Ok(None)` si no termino a
+/// tiempo: entonces se mata el arbol entero (el hijo y lo que haya lanzado,
+/// con `taskkill /T` o `kill` al grupo), y con el se cierran las tuberias.
+/// stdout y stderr se leen en hilos aparte, hasta 1 MiB cada uno, para que
+/// un hijo que escribe mucho no se bloquee.
 ///
-/// El plazo vale para todo, tambien para leer la salida: si el hijo deja un
-/// nieto con las tuberias heredadas (un `.bat` que lanza otro proceso), las
-/// tuberias no se cierran al acabar el hijo y los hilos lectores siguen
-/// bloqueados. Entonces se espera al cierre hasta que se cumple el plazo (no
-/// mas) y se devuelve lo leido hasta ese momento: la salida del hijo, que
-/// acabo, no se pierde, pero la llamada tarda el plazo entero. Los hilos
-/// acaban solos cuando el nieto cierre las tuberias (o con el proceso). Al
-/// nieto no se le mata: no hay una forma portable de matar el arbol sin
-/// dependencias.
+/// El plazo vale para todo, tambien para leer la salida: si el hijo acaba a
+/// tiempo pero deja un nieto en segundo plano con las tuberias heredadas (un
+/// `.bat` que lanza otro proceso), las tuberias no se cierran y los hilos
+/// lectores siguen bloqueados. Entonces se espera al cierre hasta que se
+/// cumple el plazo (no mas) y se devuelve lo leido hasta ese momento: la
+/// salida del hijo, que acabo, no se pierde, pero la llamada tarda el plazo
+/// entero. A ese nieto no se le mata (el hijo acabo bien); los hilos acaban
+/// solos cuando cierre las tuberias (o con el proceso).
 pub(crate) fn ejecutar_con_limite(
     mut cmd: Command,
     limite: Duration,
@@ -523,7 +525,7 @@ pub(crate) fn ejecutar_con_limite(
     use std::io::Read;
     use std::sync::{Arc, Mutex, PoisonError, mpsc};
     const MAXIMO: usize = 1024 * 1024;
-    let mut hijo = cmd
+    let mut hijo = crate::proceso::grupo_propio(&mut cmd)
         .stdin(Stdio::null())
         .stdout(Stdio::piped())
         .stderr(Stdio::piped())
@@ -572,24 +574,7 @@ pub(crate) fn ejecutar_con_limite(
             .map(|p| Box::new(p) as Box<dyn Read + Send>),
     );
     let fin = Instant::now() + limite;
-    let status = loop {
-        match hijo.try_wait() {
-            Ok(Some(s)) => break Some(s),
-            Ok(None) => {}
-            Err(e) => {
-                // Sin poder saber si sigue vivo: se mata antes de salir.
-                let _ = hijo.kill();
-                let _ = hijo.wait();
-                return Err(e);
-            }
-        }
-        if Instant::now() >= fin {
-            let _ = hijo.kill();
-            let _ = hijo.wait();
-            break None;
-        }
-        std::thread::sleep(Duration::from_millis(20));
-    };
+    let status = crate::proceso::esperar_con_plazo(&mut hijo, limite)?;
     // Lo que quede del plazo, con un margen corto para recoger la salida de
     // un hijo que acabo justo al limite.
     let restante = || {
diff --git a/crates/tc2/src/alta.rs b/crates/tc2/src/alta.rs
index 7e24646..1aa2c96 100644
--- a/crates/tc2/src/alta.rs
+++ b/crates/tc2/src/alta.rs
@@ -716,36 +716,7 @@ pub(crate) fn hoy() -> String {
         .duration_since(UNIX_EPOCH)
         .map(|d| d.as_secs())
         .unwrap_or(0);
-    fecha_de_dias(i64::try_from(segundos / 86_400).unwrap_or(0))
-}
-
-/// Instante UTC `AAAA-MM-DDTHH:MM:SSZ` de unos segundos desde 1970 (F3-5b,
-/// D6a), con la fecha de [`fecha_de_dias`].
-pub(crate) fn instante_utc(segundos: u64) -> String {
-    let dias = i64::try_from(segundos / 86_400).unwrap_or(0);
-    let s = segundos % 86_400;
-    format!(
-        "{}T{:02}:{:02}:{:02}Z",
-        fecha_de_dias(dias),
-        s / 3600,
-        s % 3600 / 60,
-        s % 60
-    )
-}
-
-/// Fecha civil de un numero de dias desde 1970-01-01 (algoritmo de Howard
-/// Hinnant, `civil_from_days`).
-pub(crate) fn fecha_de_dias(dias: i64) -> String {
-    let z = dias + 719_468;
-    let era = z.div_euclid(146_097);
-    let doe = z.rem_euclid(146_097);
-    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
-    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
-    let mp = (5 * doy + 2) / 153;
-    let d = doy - (153 * mp + 2) / 5 + 1;
-    let m = if mp < 10 { mp + 3 } else { mp - 9 };
-    let a = yoe + era * 400 + i64::from(m <= 2);
-    format!("{a:04}-{m:02}-{d:02}")
+    crate::fecha::fecha_de_dias(i64::try_from(segundos / 86_400).unwrap_or(0))
 }
 
 #[cfg(test)]
@@ -1088,17 +1059,6 @@ mod tests {
         assert_eq!(cerrados, ["terminada", "descartada"]);
     }
 
-    #[test]
-    fn fechas_civiles() {
-        assert_eq!(fecha_de_dias(0), "1970-01-01");
-        assert_eq!(fecha_de_dias(11_016), "2000-02-29");
-        assert_eq!(fecha_de_dias(20_772), "2026-11-15");
-        assert_eq!(fecha_de_dias(-1), "1969-12-31");
-        assert_eq!(instante_utc(0), "1970-01-01T00:00:00Z");
-        assert_eq!(instante_utc(951_868_799), "2000-02-29T23:59:59Z");
-        assert_eq!(instante_utc(1_794_931_452), "2026-11-17T16:04:12Z");
-    }
-
     /// La regla de `plan` en el alta (A3-08): con objetivo, 12 criterios
     /// valen y 13 no; sin objetivo, tampoco.
     #[test]
diff --git a/crates/tc2/src/ciclo/operacion.rs b/crates/tc2/src/ciclo/operacion.rs
index 3206115..e481d3e 100644
--- a/crates/tc2/src/ciclo/operacion.rs
+++ b/crates/tc2/src/ciclo/operacion.rs
@@ -22,11 +22,11 @@ use std::cell::RefCell;
 use crate::estado::{
     AnotacionConDueno, Cambio, Estado, Fallo, Operacion, por_estado, tarea_propia,
 };
+use crate::fecha::instante;
 use crate::grafo::DESCARTADA;
 use crate::i18n::catalogo::ciclo as txt;
 use crate::identidad::Quien;
 use crate::tarea::{Asignado, Cuerpo, PAUSA, REANUDAR, Tarea, Transicion};
-use crate::toma::instante;
 use crate::toma::operacion::MODO;
 
 /// Nombre de `descartar`, de su operacion y de su fila de transicion.
diff --git a/crates/tc2/src/ciclo/prueba.rs b/crates/tc2/src/ciclo/prueba.rs
index 23a3bbc..d4cc8cb 100644
--- a/crates/tc2/src/ciclo/prueba.rs
+++ b/crates/tc2/src/ciclo/prueba.rs
@@ -49,7 +49,7 @@ pub(crate) fn md(id: &str, estado: &str, asignado: Option<Asignado>, extra: Extr
     let fila = extra.hace.map_or_else(String::new, |hace| {
         format!(
             "| {} | start | manual | persona | 0.1 |\n",
-            crate::toma::instante(AHORA - hace)
+            crate::fecha::instante(AHORA - hace)
         )
     });
     t.escribir(&Cuerpo::plantilla()) + &fila
diff --git a/crates/tc2/src/confianza/mod.rs b/crates/tc2/src/confianza/mod.rs
index 6dfe2f0..0dba999 100644
--- a/crates/tc2/src/confianza/mod.rs
+++ b/crates/tc2/src/confianza/mod.rs
@@ -165,16 +165,8 @@ fn escribir_almacen(ruta: &Path, a: &Almacen, op: &str) -> Result<(), Informe> {
                 .con("error", e.to_string()),
         )
     };
-    if let Some(padre) = ruta.parent() {
-        std::fs::create_dir_all(padre).map_err(|e| error(&e))?;
-    }
     let texto = serde_json::to_string_pretty(a).map_err(|e| error(&e))?;
-    let tmp = ruta.with_extension(format!("json.{}.tmp", std::process::id()));
-    std::fs::write(&tmp, texto).map_err(|e| error(&e))?;
-    std::fs::rename(&tmp, ruta).map_err(|e| {
-        let _ = std::fs::remove_file(&tmp);
-        error(&e)
-    })
+    crate::escritura::atomica(ruta, texto.as_bytes()).map_err(|e| error(&e))
 }
 
 fn error_de_cierre(op: &str, e: &cierre::ErrorCierre) -> Informe {
diff --git a/crates/tc2/src/desinstalar/mod.rs b/crates/tc2/src/desinstalar/mod.rs
index 8a97ef5..a8e8165 100644
--- a/crates/tc2/src/desinstalar/mod.rs
+++ b/crates/tc2/src/desinstalar/mod.rs
@@ -527,30 +527,9 @@ fn reescribir(ruta: &Path, antes: &[u8], despues: &[u8]) -> Result<(), Mensaje>
     if ahora != antes {
         return Err(Mensaje::nuevo(&CAMBIO_DESDE_EL_PLAN).con("ruta", t(ruta)));
     }
-    escribir_atomico(ruta, despues).map_err(externo)
-}
-
-/// Escribe `ruta` de una vez: un temporal en la misma carpeta y un
-/// renombrado, para que un corte no deje el perfil o el `settings.json` de
-/// la persona vacio. Si `ruta` es un enlace simbolico se escribe su destino
-/// (como `cat >` de `install.sh`), y se conservan sus permisos.
-fn escribir_atomico(ruta: &Path, contenido: &[u8]) -> std::io::Result<()> {
-    let destino = if std::fs::symlink_metadata(ruta)?.file_type().is_symlink() {
-        std::fs::canonicalize(ruta)?
-    } else {
-        ruta.to_path_buf()
-    };
-    let permisos = std::fs::metadata(&destino)?.permissions();
-    let mut nombre = destino.file_name().unwrap_or_default().to_os_string();
-    nombre.push(format!(".tc2-tmp-{}", std::process::id()));
-    let tmp = destino.with_file_name(nombre);
-    let r = std::fs::write(&tmp, contenido)
-        .and_then(|()| std::fs::set_permissions(&tmp, permisos))
-        .and_then(|()| std::fs::rename(&tmp, &destino));
-    if r.is_err() {
-        let _ = std::fs::remove_file(&tmp);
-    }
-    r
+    // De una vez: un corte no deja el perfil o el `settings.json` de la
+    // persona vacio; un enlace se escribe en su destino y conserva permisos.
+    crate::escritura::atomica(ruta, despues).map_err(externo)
 }
 
 /// La orden completa desde la linea de ordenes, en la carpeta actual y con
diff --git a/crates/tc2/src/escritura.rs b/crates/tc2/src/escritura.rs
new file mode 100644
index 0000000..0301aa5
--- /dev/null
+++ b/crates/tc2/src/escritura.rs
@@ -0,0 +1,173 @@
+//! Escritura de un fichero de una vez (A8-04, que absorbe A7-02): la de los
+//! ficheros locales de la carpeta comun (marca de sincronizacion, cache de
+//! identidad y progreso), la del almacen de confianza y la de los ficheros
+//! de la persona que reescribe `tc2 desinstalar`, en un solo sitio.
+//!
+//! Un temporal `<nombre>.tc2-tmp-<pid>` al lado del destino y un `rename`
+//! encima: quien lee no ve nunca un fichero a medias, y un corte no deja el
+//! destino vacio.
+
+use std::io;
+use std::path::Path;
+use std::time::Duration;
+
+/// Esperas antes de repetir el `rename` de [`atomica`]: en Windows falla
+/// mientras un lector tiene el destino abierto sin `FILE_SHARE_DELETE`
+/// (Python, Node o un antivirus; L2 de F1-17, punto 7). Unos 300 ms en total.
+const ESPERAS_RENAME: [Duration; 5] = [
+    Duration::from_millis(10),
+    Duration::from_millis(20),
+    Duration::from_millis(40),
+    Duration::from_millis(80),
+    Duration::from_millis(150),
+];
+
+/// Escribe `ruta` de una vez. Si `ruta` es un enlace simbolico se escribe su
+/// destino (como `cat >` de `install.sh`); si el destino ya existe, el nuevo
+/// conserva sus permisos. Crea la carpeta si falta, escribe el temporal
+/// `<nombre>.tc2-tmp-<pid>` al lado y lo renombra encima. Si el `rename`
+/// falla, se repite tras cada espera de [`ESPERAS_RENAME`]. Si falla la
+/// escritura, los permisos o el ultimo `rename`, se borra el temporal y se
+/// devuelve el error.
+pub(crate) fn atomica(ruta: &Path, contenido: &[u8]) -> io::Result<()> {
+    atomica_con(ruta, contenido, &ESPERAS_RENAME)
+}
+
+fn atomica_con(ruta: &Path, contenido: &[u8], esperas: &[Duration]) -> io::Result<()> {
+    let destino = match std::fs::symlink_metadata(ruta) {
+        Ok(m) if m.file_type().is_symlink() => std::fs::canonicalize(ruta)?,
+        _ => ruta.to_path_buf(),
+    };
+    let permisos = std::fs::metadata(&destino).ok().map(|m| m.permissions());
+    if let Some(padre) = destino.parent() {
+        std::fs::create_dir_all(padre)?;
+    }
+    let mut nombre = destino.file_name().unwrap_or_default().to_os_string();
+    nombre.push(format!(".tc2-tmp-{}", std::process::id()));
+    let tmp = destino.with_file_name(nombre);
+    let r = std::fs::write(&tmp, contenido)
+        .and_then(|()| match permisos {
+            Some(p) => std::fs::set_permissions(&tmp, p),
+            None => Ok(()),
+        })
+        .and_then(|()| renombrar(&tmp, &destino, esperas));
+    if r.is_err() {
+        let _ = std::fs::remove_file(&tmp);
+    }
+    r
+}
+
+/// `rename` de `tmp` a `destino`, repetido tras cada espera mientras falle.
+fn renombrar(tmp: &Path, destino: &Path, esperas: &[Duration]) -> io::Result<()> {
+    let mut r = std::fs::rename(tmp, destino);
+    for e in esperas {
+        if r.is_ok() {
+            break;
+        }
+        std::thread::sleep(*e);
+        r = std::fs::rename(tmp, destino);
+    }
+    r
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use std::path::PathBuf;
+
+    fn carpeta(nombre: &str) -> PathBuf {
+        let d = std::env::temp_dir()
+            .join("tc2-tests")
+            .join(format!("escritura-{nombre}-{}", std::process::id()));
+        let _ = std::fs::remove_dir_all(&d);
+        d
+    }
+
+    /// Crea la carpeta, sustituye el contenido y no deja temporales sueltos.
+    #[test]
+    fn de_una_vez_y_sin_temporales() -> io::Result<()> {
+        let d = carpeta("escribir");
+        let ruta = d.join("sub").join("f.json");
+        atomica(&ruta, b"uno")?;
+        atomica(&ruta, b"dos")?;
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
+    fn reintenta_con_un_lector_abierto() -> io::Result<()> {
+        use std::os::windows::fs::OpenOptionsExt;
+        const FILE_SHARE_READ: u32 = 1;
+        let d = carpeta("lector");
+        let ruta = d.join("f.json");
+        atomica(&ruta, b"viejo")?;
+        let abrir = || {
+            std::fs::OpenOptions::new()
+                .read(true)
+                .share_mode(FILE_SHARE_READ)
+                .open(&ruta)
+        };
+        // Sin reintento, con el lector abierto: falla y no deja temporal.
+        let lector = abrir()?;
+        assert!(atomica_con(&ruta, b"nuevo", &[]).is_err());
+        assert_eq!(std::fs::read_dir(&d)?.count(), 1);
+        // Con reintento: el lector cierra a los 30 ms y la escritura entra.
+        let hilo = std::thread::spawn(move || {
+            std::thread::sleep(Duration::from_millis(30));
+            drop(lector);
+        });
+        atomica(&ruta, b"nuevo")?;
+        let _ = hilo.join();
+        assert_eq!(std::fs::read(&ruta)?, b"nuevo");
+        let _ = std::fs::remove_dir_all(&d);
+        Ok(())
+    }
+
+    /// Un `rename` que no puede entrar (el destino es una carpeta no vacia)
+    /// da el error, deja la carpeta como estaba y no deja el temporal.
+    #[test]
+    fn rename_que_falla_borra_el_temporal() -> io::Result<()> {
+        let d = carpeta("rename");
+        let ruta = d.join("destino");
+        std::fs::create_dir_all(&ruta)?;
+        std::fs::write(ruta.join("dentro"), b"x")?;
+        assert!(atomica_con(&ruta, b"nuevo", &[]).is_err());
+        assert!(ruta.is_dir());
+        assert_eq!(std::fs::read(ruta.join("dentro"))?, b"x");
+        let nombres: Vec<_> = std::fs::read_dir(&d)?
+            .filter_map(|e| e.ok().map(|e| e.file_name()))
+            .collect();
+        assert_eq!(nombres, vec![std::ffi::OsString::from("destino")]);
+        let _ = std::fs::remove_dir_all(&d);
+        Ok(())
+    }
+
+    /// Por un enlace simbolico se escribe su destino, que conserva sus
+    /// permisos (`0o600`); el enlace sigue siendo enlace.
+    #[cfg(unix)]
+    #[test]
+    fn sigue_el_enlace_y_conserva_los_permisos() -> io::Result<()> {
+        use std::os::unix::fs::PermissionsExt;
+        let d = carpeta("enlace");
+        std::fs::create_dir_all(&d)?;
+        let real = d.join("real");
+        std::fs::write(&real, b"viejo")?;
+        std::fs::set_permissions(&real, std::fs::Permissions::from_mode(0o600))?;
+        let enlace = d.join("enlace");
+        std::os::unix::fs::symlink(&real, &enlace)?;
+        atomica(&enlace, b"nuevo")?;
+        assert!(std::fs::symlink_metadata(&enlace)?.file_type().is_symlink());
+        assert_eq!(std::fs::read(&real)?, b"nuevo");
+        let modo = std::fs::metadata(&real)?.permissions().mode() & 0o777;
+        assert_eq!(modo, 0o600);
+        let _ = std::fs::remove_dir_all(&d);
+        Ok(())
+    }
+}
diff --git a/crates/tc2/src/estado/frescura.rs b/crates/tc2/src/estado/frescura.rs
index 2bab2e8..35e170b 100644
--- a/crates/tc2/src/estado/frescura.rs
+++ b/crates/tc2/src/estado/frescura.rs
@@ -169,16 +169,7 @@ impl Marca {
     }
 
     fn leer_crudo(&self) -> Option<u64> {
-        use std::io::Read;
-        let mut texto = String::new();
-        std::fs::File::open(&self.ruta)
-            .ok()?
-            .take(LIMITE_MARCA + 1)
-            .read_to_string(&mut texto)
-            .ok()?;
-        if texto.len() as u64 > LIMITE_MARCA {
-            return None;
-        }
+        let texto = crate::lectura::texto_con_tope(&self.ruta, LIMITE_MARCA).ok()??;
         let t = texto.strip_suffix('\n').unwrap_or(&texto);
         if t.is_empty() || !t.bytes().all(|b| b.is_ascii_digit()) {
             return None;
@@ -187,9 +178,9 @@ impl Marca {
     }
 
     /// Guarda `ahora`. Escribe un temporal y lo renombra
-    /// ([`crate::huella::escribir_local`]).
+    /// (`crate::escritura::atomica`).
     pub fn registrar(&self, ahora: u64) -> std::io::Result<()> {
-        crate::huella::escribir_local(&self.ruta, format!("{ahora}\n").as_bytes())
+        crate::escritura::atomica(&self.ruta, format!("{ahora}\n").as_bytes())
     }
 }
 
@@ -260,7 +251,7 @@ pub struct Frescura {
 
 /// Valor de `sincronizado=`: la hora UTC o `nunca`.
 pub fn sincronizado_texto(sincronizado: Option<u64>) -> String {
-    sincronizado.map_or_else(|| "nunca".to_owned(), crate::alta::instante_utc)
+    sincronizado.map_or_else(|| "nunca".to_owned(), crate::fecha::instante)
 }
 
 /// «hace N» entre `entonces` y `ahora` (un `entonces` del futuro, por un
@@ -303,7 +294,7 @@ impl Frescura {
         Some(match self.sincronizado {
             Some(s) => Mensaje::nuevo(&txt::AVISO_NO_FRESCO)
                 .con_mensaje("porque", porque)
-                .con("hora", crate::alta::instante_utc(s))
+                .con("hora", crate::fecha::instante(s))
                 .con_mensaje("hace", hace(s, ahora)),
             None => Mensaje::nuevo(&txt::AVISO_NUNCA_SINCRONIZADO).con_mensaje("porque", porque),
         })
@@ -319,7 +310,7 @@ impl Frescura {
         match (&self.remoto, self.sincronizado) {
             (Remoto::Consultado, Some(s)) => Some(
                 Mensaje::nuevo(&txt::SINCRONIZADO)
-                    .con("hora", crate::alta::instante_utc(s))
+                    .con("hora", crate::fecha::instante(s))
                     .con_mensaje("hace", hace(s, ahora)),
             ),
             _ => None,
diff --git a/crates/tc2/src/estimacion/mod.rs b/crates/tc2/src/estimacion/mod.rs
index 1da471d..89b66eb 100644
--- a/crates/tc2/src/estimacion/mod.rs
+++ b/crates/tc2/src/estimacion/mod.rs
@@ -89,7 +89,7 @@ pub fn recalcular(t: &mut Tarea, cuerpo: &Cuerpo, h: &Historico, ahora: u64) ->
     let mut e = estimar(
         &objetivo(t, cuerpo),
         &h.muestras,
-        &crate::alta::instante_utc(ahora),
+        &crate::fecha::instante(ahora),
     );
     e.agente = t
         .estimacion()
@@ -268,7 +268,7 @@ fn mostrar(e: &Entorno<'_>, id: &str) -> (Informe, Option<String>) {
             let est = estimar(
                 &objetivo(&t, &c),
                 &h.muestras,
-                &crate::alta::instante_utc(e.ahora),
+                &crate::fecha::instante(e.ahora),
             );
             let i = con_historia(
                 con_claves(
diff --git a/crates/tc2/src/fecha.rs b/crates/tc2/src/fecha.rs
new file mode 100644
index 0000000..d222492
--- /dev/null
+++ b/crates/tc2/src/fecha.rs
@@ -0,0 +1,58 @@
+//! Fechas UTC sin dependencias (A8-03): el instante `AAAA-MM-DDTHH:MM:SSZ`
+//! de las transiciones, la marca de sincronizacion y la estimacion (F3-5b,
+//! D6a), y la fecha civil del algoritmo de Howard Hinnant, una sola vez.
+//!
+//! `xtask` (`fecha_utc` de `xtask/src/main.rs`) tiene su propia copia: solo
+//! depende de `distribucion`, no de `tc2`.
+
+/// Instante UTC `AAAA-MM-DDTHH:MM:SSZ` de unos segundos desde 1970, con la
+/// fecha de [`fecha_de_dias`].
+pub fn instante(segundos: u64) -> String {
+    let dias = i64::try_from(segundos / 86_400).unwrap_or(0);
+    let resto = segundos % 86_400;
+    format!(
+        "{}T{:02}:{:02}:{:02}Z",
+        fecha_de_dias(dias),
+        resto / 3600,
+        resto % 3600 / 60,
+        resto % 60
+    )
+}
+
+/// Fecha civil `AAAA-MM-DD` de un numero de dias desde 1970-01-01
+/// (algoritmo de Howard Hinnant, `civil_from_days`).
+pub(crate) fn fecha_de_dias(dias: i64) -> String {
+    let z = dias + 719_468;
+    let era = z.div_euclid(146_097);
+    let doe = z.rem_euclid(146_097);
+    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
+    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
+    let mp = (5 * doy + 2) / 153;
+    let d = doy - (153 * mp + 2) / 5 + 1;
+    let m = if mp < 10 { mp + 3 } else { mp - 9 };
+    let a = yoe + era * 400 + i64::from(m <= 2);
+    format!("{a:04}-{m:02}-{d:02}")
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    /// Los asserts de fechas que estaban en `alta` y en `toma`, juntos.
+    #[test]
+    fn fechas_civiles() {
+        assert_eq!(fecha_de_dias(0), "1970-01-01");
+        assert_eq!(fecha_de_dias(11_016), "2000-02-29");
+        assert_eq!(fecha_de_dias(20_772), "2026-11-15");
+        assert_eq!(fecha_de_dias(-1), "1969-12-31");
+    }
+
+    #[test]
+    fn instante_en_utc() {
+        assert_eq!(instante(0), "1970-01-01T00:00:00Z");
+        assert_eq!(instante(86_399), "1970-01-01T23:59:59Z");
+        assert_eq!(instante(951_868_799), "2000-02-29T23:59:59Z");
+        assert_eq!(instante(1_794_910_830), "2026-11-17T10:20:30Z");
+        assert_eq!(instante(1_794_931_452), "2026-11-17T16:04:12Z");
+    }
+}
diff --git a/crates/tc2/src/hallazgos/orden.rs b/crates/tc2/src/hallazgos/orden.rs
index f188bf7..d1d28ca 100644
--- a/crates/tc2/src/hallazgos/orden.rs
+++ b/crates/tc2/src/hallazgos/orden.rs
@@ -59,7 +59,6 @@
 //!   cortado=<si|no> abiertos=<si|no>`.
 
 use std::collections::BTreeMap;
-use std::io::Read;
 use std::path::{Path, PathBuf};
 
 use crate::Salida;
@@ -213,16 +212,6 @@ fn args(a: &[String]) -> Result<Args, Informe> {
     Ok(r)
 }
 
-/// Lee como mucho `tope` bytes de `p`: `Ok(None)` si pasa del tope.
-fn leer_con_tope(p: &Path, tope: u64) -> std::io::Result<Option<String>> {
-    let mut b = Vec::new();
-    std::fs::File::open(p)?.take(tope + 1).read_to_end(&mut b)?;
-    if b.len() as u64 > tope {
-        return Ok(None);
-    }
-    Ok(Some(String::from_utf8_lossy(&b).into_owned()))
-}
-
 fn fallo_io(p: &Path, error: String) -> Informe {
     Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(
         Mensaje::nuevo(&txt::NO_SE_LEE)
@@ -232,7 +221,7 @@ fn fallo_io(p: &Path, error: String) -> Informe {
 }
 
 fn leer_entrada(p: &Path, tope: u64) -> Result<String, Informe> {
-    leer_con_tope(p, tope)
+    crate::lectura::texto_con_tope(p, tope)
         .map_err(|e| fallo_io(p, e.to_string()))?
         .ok_or_else(|| {
             Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(
@@ -296,7 +285,8 @@ fn validar(a: &[String]) -> Result<Informe, Informe> {
     let diff = super::Diff::leer(&leer_entrada(pd, TOPE_DIFF)?);
     let (estado, inf) = match &a.informe {
         None => (None, None),
-        Some(p) => match leer_con_tope(p, TOPE_INFORME) {
+        Some(p) => match crate::lectura::texto_con_tope(p, TOPE_INFORME) {
+            Err(e) if e.kind() == std::io::ErrorKind::InvalidData => (Some("invalido"), None),
             Err(_) => (Some("sin-informe"), None),
             Ok(None) => (Some("grande"), None),
             Ok(Some(t)) => match informe::leer(&t) {
@@ -337,7 +327,7 @@ fn validar(a: &[String]) -> Result<Informe, Informe> {
 
 /// Una salida de `validar` del contrato vigente, o `None`.
 fn leer_validados(p: &Path) -> Option<super::Validados> {
-    let t = leer_con_tope(p, TOPE_VALIDADOS).ok()??;
+    let t = crate::lectura::texto_con_tope(p, TOPE_VALIDADOS).ok()??;
     serde_json::from_str::<super::Validados>(&t)
         .ok()
         .filter(|v| v.version == super::VERSION)
diff --git a/crates/tc2/src/huella.rs b/crates/tc2/src/huella.rs
index 7eb5b3b..70e3bc4 100644
--- a/crates/tc2/src/huella.rs
+++ b/crates/tc2/src/huella.rs
@@ -12,7 +12,7 @@
 //!   la ultima sincronizacion, los indices temporales de la transaccion y el
 //!   progreso en vivo ([`CARPETA_PROGRESO`]). `tc2 desinstalar` borra la
 //!   carpeta entera, asi que un fichero nuevo aqui queda reconocido sin
-//!   tocarlo. Sus ficheros se escriben con [`escribir_local`].
+//!   tocarlo. Sus ficheros se escriben de una vez con `escritura::atomica`.
 //!
 //! La configuracion del proyecto (`.taskcode/config.yml`, `personas.yml` y
 //! el vocabulario) no esta aqui: F1-12 D4a la deja compartida con la 0.16.
@@ -69,53 +69,6 @@ pub const CARPETA_TEMPORALES: &str = "estado-tmp";
 /// status line y `tc2 status --seguir` (adenda de TASK-084 en F1-17).
 pub const CARPETA_PROGRESO: &str = "progreso";
 
-/// Esperas antes de repetir el `rename` de [`escribir_local`]: en Windows
-/// falla mientras un lector tiene el destino abierto sin `FILE_SHARE_DELETE`
-/// (Python, Node o un antivirus; L2 de F1-17, punto 7). Unos 300 ms en total.
-const ESPERAS_RENAME: [std::time::Duration; 5] = [
-    std::time::Duration::from_millis(10),
-    std::time::Duration::from_millis(20),
-    std::time::Duration::from_millis(40),
-    std::time::Duration::from_millis(80),
-    std::time::Duration::from_millis(150),
-];
-
-/// Escribe `ruta` de una vez, para los ficheros locales de [`CARPETA_COMUN`]
-/// (la marca de sincronizacion, la cache de identidad y el progreso): crea la
-/// carpeta si falta, escribe un temporal `<nombre>.tmp-<pid>` al lado y lo
-/// renombra encima, de modo que quien lee no ve nunca un fichero a medias.
-/// Si el `rename` falla, se repite tras cada espera de [`ESPERAS_RENAME`]; si
-/// sigue fallando, se borra el temporal y se devuelve el ultimo error.
-pub fn escribir_local(ruta: &Path, contenido: &[u8]) -> io::Result<()> {
-    escribir_local_con(ruta, contenido, &ESPERAS_RENAME)
-}
-
-fn escribir_local_con(
-    ruta: &Path,
-    contenido: &[u8],
-    esperas: &[std::time::Duration],
-) -> io::Result<()> {
-    if let Some(padre) = ruta.parent() {
-        std::fs::create_dir_all(padre)?;
-    }
-    let mut nombre = ruta.file_name().unwrap_or_default().to_os_string();
-    nombre.push(format!(".tmp-{}", std::process::id()));
-    let tmp = ruta.with_file_name(nombre);
-    std::fs::write(&tmp, contenido)?;
-    let mut r = std::fs::rename(&tmp, ruta);
-    for e in esperas {
-        if r.is_ok() {
-            break;
-        }
-        std::thread::sleep(*e);
-        r = std::fs::rename(&tmp, ruta);
-    }
-    if r.is_err() {
-        let _ = std::fs::remove_file(&tmp);
-    }
-    r
-}
-
 /// Workflow de CI que deja `tc2 init` (F3-1). Hasta que F5-1 entregue el
 /// reutilizable, `init` solo deja la ranura (`pendiente motivo=sin-plantilla`).
 pub const RUTA_WORKFLOW: &str = concat!(".github/workflows/", nombre!(), ".yml");
@@ -225,60 +178,4 @@ mod tests {
         let _ = std::fs::remove_dir_all(&comun);
         Ok(())
     }
-
-    fn carpeta(nombre: &str) -> PathBuf {
-        let d = std::env::temp_dir()
-            .join("tc2-tests")
-            .join(format!("huella-{nombre}-{}", std::process::id()));
-        let _ = std::fs::remove_dir_all(&d);
-        d
-    }
-
-    /// `escribir_local` crea la carpeta, sustituye el contenido y no deja
-    /// temporales sueltos.
-    #[test]
-    fn escribir_local_de_una_vez_y_sin_temporales() -> io::Result<()> {
-        let d = carpeta("escribir");
-        let ruta = d.join("sub").join("f.json");
-        escribir_local(&ruta, b"uno")?;
-        escribir_local(&ruta, b"dos")?;
-        assert_eq!(std::fs::read(&ruta)?, b"dos");
-        let n = std::fs::read_dir(d.join("sub"))?.count();
-        assert_eq!(n, 1);
-        let _ = std::fs::remove_dir_all(&d);
-        Ok(())
-    }
-
-    /// L2 de F1-17 (punto 7): en Windows, con un lector abierto sin
-    /// `FILE_SHARE_DELETE` (como Python o Node), el `rename` falla; con el
-    /// reintento entra en cuanto el lector cierra. Sin reintento, falla.
-    #[cfg(windows)]
-    #[test]
-    fn escribir_local_reintenta_con_un_lector_abierto() -> io::Result<()> {
-        use std::os::windows::fs::OpenOptionsExt;
-        const FILE_SHARE_READ: u32 = 1;
-        let d = carpeta("lector");
-        let ruta = d.join("f.json");
-        escribir_local(&ruta, b"viejo")?;
-        let abrir = || {
-            std::fs::OpenOptions::new()
-                .read(true)
-                .share_mode(FILE_SHARE_READ)
-                .open(&ruta)
-        };
-        // Sin reintento, con el lector abierto: falla y no deja temporal.
-        let lector = abrir()?;
-        assert!(escribir_local_con(&ruta, b"nuevo", &[]).is_err());
-        assert_eq!(std::fs::read_dir(&d)?.count(), 1);
-        // Con reintento: el lector cierra a los 30 ms y la escritura entra.
-        let hilo = std::thread::spawn(move || {
-            std::thread::sleep(std::time::Duration::from_millis(30));
-            drop(lector);
-        });
-        escribir_local(&ruta, b"nuevo")?;
-        let _ = hilo.join();
-        assert_eq!(std::fs::read(&ruta)?, b"nuevo");
-        let _ = std::fs::remove_dir_all(&d);
-        Ok(())
-    }
 }
diff --git a/crates/tc2/src/identidad.rs b/crates/tc2/src/identidad.rs
index 4c31a40..4b66b25 100644
--- a/crates/tc2/src/identidad.rs
+++ b/crates/tc2/src/identidad.rs
@@ -226,18 +226,9 @@ impl Cache {
     /// Lo guardado; `None` si no hay o no se entiende (se trata como si no
     /// hubiera: hay que volver a lanzar `tc2 identidad`).
     pub fn leer(&self) -> Option<Guardada> {
-        use std::io::Read;
         // Con tope: un fichero mayor no es una cache de `tc2` (se trata como
         // corrupta, es decir, como si no hubiera).
-        let mut texto = String::new();
-        std::fs::File::open(&self.ruta)
-            .ok()?
-            .take(LIMITE_CACHE + 1)
-            .read_to_string(&mut texto)
-            .ok()?;
-        if texto.len() as u64 > LIMITE_CACHE {
-            return None;
-        }
+        let texto = crate::lectura::texto_con_tope(&self.ruta, LIMITE_CACHE).ok()??;
         let c: GuardadaCruda =
             serde_saphyr::from_str_with_options(&texto, crate::yaml::opciones()).ok()?;
         let proveedor = match c.proveedor.as_str() {
@@ -272,7 +263,7 @@ impl Cache {
             cuenta.id,
             crate::tarea::yaml::cadena(&cuenta.login),
         );
-        crate::huella::escribir_local(&self.ruta, texto.as_bytes()).map_err(error)
+        crate::escritura::atomica(&self.ruta, texto.as_bytes()).map_err(error)
     }
 }
 
diff --git a/crates/tc2/src/lectura.rs b/crates/tc2/src/lectura.rs
index eb5f5bc..5d73569 100644
--- a/crates/tc2/src/lectura.rs
+++ b/crates/tc2/src/lectura.rs
@@ -11,8 +11,13 @@
 //! [`opcional`] distingue «no existe» de «no se lee», y [`no_se_lee`] da el
 //! informe `error` de lo segundo a nombre de la orden. Esas rutas las
 //! calcula `tc2`, no las pasa la persona: no se sanean.
+//!
+//! Y el «leer con tope» de todos (A8-17): [`acotado`] lee como mucho un
+//! byte mas del tope y rechaza si llega a leerlo; [`con_tope`] y
+//! [`texto_con_tope`] lo aplican a un fichero. El texto es UTF-8 estricto:
+//! unos bytes que no lo son dan `InvalidData`, nunca `U+FFFD`.
 
-use std::io::Read as _;
+use std::io::{self, Read as _};
 use std::path::Path;
 
 use crate::contrato::{Informe, Resultado};
@@ -21,7 +26,7 @@ use crate::i18n::catalogo::comun::NO_SE_LEE;
 use crate::plataforma::errores::sanear;
 
 /// Informe `{op} error ruta=...` de un fichero que existe y no se lee.
-pub(crate) fn no_se_lee(op: &str, ruta: &Path, e: &std::io::Error) -> Informe {
+pub(crate) fn no_se_lee(op: &str, ruta: &Path, e: &io::Error) -> Informe {
     let r = ruta.display().to_string();
     Informe::nuevo(op, Resultado::Error)
         .con("ruta", r.clone())
@@ -36,11 +41,32 @@ pub(crate) fn no_se_lee(op: &str, ruta: &Path, e: &std::io::Error) -> Informe {
 pub(crate) fn opcional(op: &str, ruta: &Path) -> Result<Option<Vec<u8>>, Informe> {
     match std::fs::read(ruta) {
         Ok(b) => Ok(Some(b)),
-        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
+        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
         Err(e) => Err(no_se_lee(op, ruta, &e)),
     }
 }
 
+/// Lee de `r` como mucho `tope` bytes: se lee un byte de mas, y si llega a
+/// leerse, `None` (sin carrera entre mirar el tamano y leer).
+pub(crate) fn acotado(r: impl io::Read, tope: u64) -> io::Result<Option<Vec<u8>>> {
+    let mut b = Vec::new();
+    r.take(tope.saturating_add(1)).read_to_end(&mut b)?;
+    Ok((b.len() as u64 <= tope).then_some(b))
+}
+
+/// Los bytes de `ruta` con [`acotado`]: `None` si pasa de `tope`.
+pub(crate) fn con_tope(ruta: &Path, tope: u64) -> io::Result<Option<Vec<u8>>> {
+    acotado(std::fs::File::open(ruta)?, tope)
+}
+
+/// El texto de `ruta` con [`con_tope`]: `None` si pasa de `tope` e
+/// `InvalidData` si no es UTF-8.
+pub(crate) fn texto_con_tope(ruta: &Path, tope: u64) -> io::Result<Option<String>> {
+    con_tope(ruta, tope)?
+        .map(|b| String::from_utf8(b).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
+        .transpose()
+}
+
 /// Tope de un fichero externo (D4 de TASK-029).
 pub const MAX_BYTES: usize = 1 << 20;
 
@@ -78,15 +104,11 @@ pub fn fichero(ruta: &Path, tope: usize) -> Result<String, NoSeLeyo> {
         ruta: sanear(&ruta.display().to_string()),
         error,
     };
-    let no_se_lee = |e: std::io::Error| mal(Motivo::NoSeLee, sanear(&e.to_string()));
+    let no_se_lee = |e: io::Error| mal(Motivo::NoSeLee, sanear(&e.to_string()));
     let f = std::fs::File::open(ruta).map_err(no_se_lee)?;
-    let mut bytes = Vec::new();
-    f.take(u64::try_from(tope).unwrap_or(u64::MAX).saturating_add(1))
-        .read_to_end(&mut bytes)
-        .map_err(no_se_lee)?;
-    if bytes.len() > tope {
-        return Err(mal(Motivo::Grande, String::new()));
-    }
+    let bytes = acotado(f, u64::try_from(tope).unwrap_or(u64::MAX))
+        .map_err(no_se_lee)?
+        .ok_or_else(|| mal(Motivo::Grande, String::new()))?;
     let texto = String::from_utf8(bytes).map_err(|_| mal(Motivo::NoUtf8, String::new()))?;
     if texto.trim().is_empty() {
         return Err(mal(Motivo::Vacio, String::new()));
@@ -117,4 +139,32 @@ mod tests {
         assert_eq!(b, Ok("a\u{f1}b".to_owned()));
         Ok(())
     }
+
+    /// Frontera del tope: `tope` bytes se leen enteros; `tope + 1`, `None`.
+    #[test]
+    fn acotado_en_la_frontera() -> io::Result<()> {
+        assert_eq!(acotado(&b"abcd"[..], 4)?, Some(b"abcd".to_vec()));
+        assert_eq!(acotado(&b"abcde"[..], 4)?, None);
+        assert_eq!(acotado(&b""[..], 0)?, Some(Vec::new()));
+        assert_eq!(acotado(&b"a"[..], 0)?, None);
+        Ok(())
+    }
+
+    /// Decision 3 de TASK-150: `texto_con_tope` rechaza lo que no es UTF-8
+    /// con `InvalidData` (sin `U+FFFD`); `con_tope` da los bytes tal cual.
+    #[test]
+    fn texto_con_tope_es_utf8_estricto() -> io::Result<()> {
+        let dir = std::env::temp_dir()
+            .join("tc2-tests")
+            .join(format!("lectura-tope-{}", std::process::id()));
+        std::fs::create_dir_all(&dir)?;
+        let malo = dir.join("binario");
+        std::fs::write(&malo, [b'a', 0xff])?;
+        let texto = texto_con_tope(&malo, 16).map_err(|e| e.kind());
+        let bytes = con_tope(&malo, 16)?;
+        let _ = std::fs::remove_dir_all(&dir);
+        assert_eq!(texto, Err(io::ErrorKind::InvalidData));
+        assert_eq!(bytes, Some(vec![b'a', 0xff]));
+        Ok(())
+    }
 }
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 05150ab..9174b18 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -36,8 +36,10 @@ pub mod dato;
 pub mod descarga;
 pub mod desinstalar;
 pub mod doctor;
+mod escritura;
 pub mod estado;
 pub mod estimacion;
+pub mod fecha;
 pub mod git;
 pub mod grafo;
 pub mod hallazgos;
@@ -52,6 +54,7 @@ mod lectura;
 pub mod plan;
 mod plan_dato;
 pub mod plataforma;
+mod proceso;
 pub mod progreso;
 pub mod recursos;
 pub mod revision;
@@ -583,15 +586,8 @@ fn orden_revisoras(args: &[String]) -> Informe {
     };
     let g = git::Git::nuevo();
     let claude = dir_claude();
-    // Un byte de mas que el tope: la orden falla si llega, como con fichero.
-    let stdin = || {
-        use std::io::Read;
-        let mut b = Vec::new();
-        std::io::stdin()
-            .take(seleccion::orden::TOPE_RUTAS + 1)
-            .read_to_end(&mut b)
-            .map(|_| b)
-    };
+    // Con tope: la orden falla si pasa, como con fichero.
+    let stdin = || lectura::acotado(std::io::stdin(), seleccion::orden::TOPE_RUTAS);
     seleccion::orden::orden(
         args,
         &seleccion::orden::Entorno {
@@ -614,7 +610,7 @@ fn orden_seleccion(args: &[String]) -> Informe {
     };
     let g = git::Git::nuevo();
     let claude = dir_claude();
-    let stdin = || Ok(Vec::new());
+    let stdin = || Ok(Some(Vec::new()));
     seleccion::orden::orden_seleccion(
         args,
         &seleccion::orden::Entorno {
diff --git a/crates/tc2/src/plan/aprobar.rs b/crates/tc2/src/plan/aprobar.rs
index c1243ef..970ab47 100644
--- a/crates/tc2/src/plan/aprobar.rs
+++ b/crates/tc2/src/plan/aprobar.rs
@@ -31,12 +31,12 @@ use crate::estado::{
     AnotacionConDueno, Cambio, Estado, Fallo, Operacion, Sello, de_otra_persona, por_estado,
     tarea_de, tarea_propia, tareas,
 };
+use crate::fecha::instante;
 use crate::grafo::{DESCARTADA, TERMINADA};
 use crate::i18n::catalogo::plan as txt;
 use crate::i18n::{Mensaje, Texto};
 use crate::identidad::Quien;
 use crate::tarea::{Asignado, Cuerpo, Documento, Tarea, Transicion};
-use crate::toma::instante;
 
 /// Nombre de la orden, de la operacion y de su fila.
 pub const NOMBRE: &str = "approve";
diff --git a/crates/tc2/src/plan/operacion.rs b/crates/tc2/src/plan/operacion.rs
index bd84d21..e41d11a 100644
--- a/crates/tc2/src/plan/operacion.rs
+++ b/crates/tc2/src/plan/operacion.rs
@@ -50,13 +50,13 @@ use crate::config::Config;
 use crate::estado::{
     AnotacionConDueno, Cambio, Estado, Fallo, Operacion, por_estado, tarea_propia,
 };
+use crate::fecha::instante;
 use crate::i18n::catalogo::plan as txt;
 use crate::i18n::{Mensaje, Texto};
 use crate::identidad::Quien;
 use crate::inventario::Inventario;
 use crate::seleccion::{self, Catalogo};
 use crate::tarea::{Asignado, Cuerpo, Transicion};
-use crate::toma::instante;
 
 /// Estados desde los que se planifica.
 pub const PLANIFICABLES: [&str; 2] = ["planificada", "en-diseno"];
diff --git a/crates/tc2/src/proceso.rs b/crates/tc2/src/proceso.rs
new file mode 100644
index 0000000..c30b333
--- /dev/null
+++ b/crates/tc2/src/proceso.rs
@@ -0,0 +1,130 @@
+//! Esperar a un proceso hijo con plazo y matarlo si no acaba (A9-01): la
+//! espera de `tc2 verificar` y la de `actualizar_cli::ejecutar_con_limite`,
+//! en un solo sitio. Al cumplirse el plazo se mata el arbol entero (el hijo
+//! y lo que haya lanzado), sin dependencias: `taskkill /T /F` en Windows y
+//! `kill -KILL -- -<pgid>` en Unix, con el hijo en un grupo propio
+//! ([`grupo_propio`]).
+
+use std::io;
+use std::process::{Child, Command, ExitStatus, Stdio};
+use std::time::{Duration, Instant};
+
+/// Cada cuanto se mira si el hijo termino.
+const SONDEO: Duration = Duration::from_millis(10);
+
+/// Pone `c` en un grupo de procesos propio (`process_group(0)` en Unix),
+/// para que [`esperar_con_plazo`] pueda matar el arbol entero. En Windows no
+/// hace nada: `taskkill /T` sigue el arbol por el proceso padre.
+pub(crate) fn grupo_propio(c: &mut Command) -> &mut Command {
+    #[cfg(unix)]
+    {
+        use std::os::unix::process::CommandExt;
+        c.process_group(0);
+    }
+    c
+}
+
+/// Espera a `hijo` como mucho `plazo`: `Ok(Some(estado))` si acabo y
+/// `Ok(None)` si se cumplio el plazo. Al cumplirse, o si falla `try_wait`
+/// (sin saber si sigue vivo), se mata el arbol entero ([`matar_arbol`]) y el
+/// hijo, y se espera a que muera. En Unix el arbol entero solo cae si el
+/// hijo se lanzo con [`grupo_propio`].
+pub(crate) fn esperar_con_plazo(
+    hijo: &mut Child,
+    plazo: Duration,
+) -> io::Result<Option<ExitStatus>> {
+    let fin = Instant::now() + plazo;
+    loop {
+        match hijo.try_wait() {
+            Ok(Some(s)) => return Ok(Some(s)),
+            Ok(None) if Instant::now() >= fin => {
+                matar(hijo);
+                return Ok(None);
+            }
+            Ok(None) => std::thread::sleep(SONDEO),
+            Err(e) => {
+                matar(hijo);
+                return Err(e);
+            }
+        }
+    }
+}
+
+/// Mata el arbol de `hijo` y al propio hijo, y lo recoge.
+fn matar(hijo: &mut Child) {
+    matar_arbol(hijo.id());
+    let _ = hijo.kill();
+    let _ = hijo.wait();
+}
+
+/// Mata el proceso `pid` y lo que haya lanzado (una shell sola dejaria vivos
+/// a sus hijos, con las tuberias heredadas abiertas).
+fn matar_arbol(pid: u32) {
+    let pid = pid.to_string();
+    #[cfg(windows)]
+    let mut c = {
+        let mut c = Command::new("taskkill");
+        c.args(["/T", "/F", "/PID", &pid]);
+        c
+    };
+    #[cfg(not(windows))]
+    let mut c = {
+        let mut c = Command::new("kill");
+        c.args(["-KILL", "--", &format!("-{pid}")]);
+        c
+    };
+    let _ = c
+        .stdin(Stdio::null())
+        .stdout(Stdio::null())
+        .stderr(Stdio::null())
+        .status();
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    /// Un hijo que no acaba y deja un nieto en segundo plano con stdout
+    /// heredado: al cumplirse el plazo cae el arbol entero y la tuberia llega
+    /// al final enseguida. Matando solo al hijo, el nieto la retendria 30 s.
+    #[test]
+    fn al_plazo_mata_el_arbol_entero() -> io::Result<()> {
+        use std::io::Read as _;
+        let mut c = if cfg!(windows) {
+            let mut c = Command::new("cmd");
+            c.args([
+                "/c",
+                "start /b ping -n 30 127.0.0.1 & ping -n 30 127.0.0.1 >nul",
+            ]);
+            c
+        } else {
+            let mut c = Command::new("sh");
+            c.args(["-c", "sleep 30 & sleep 30"]);
+            c
+        };
+        grupo_propio(&mut c)
+            .stdin(Stdio::null())
+            .stdout(Stdio::piped())
+            .stderr(Stdio::null());
+        let mut hijo = c.spawn()?;
+        let mut salida = hijo.stdout.take();
+        let (tx, rx) = std::sync::mpsc::channel();
+        std::thread::spawn(move || {
+            let mut v = Vec::new();
+            if let Some(s) = salida.as_mut() {
+                let _ = s.read_to_end(&mut v);
+            }
+            let _ = tx.send(());
+        });
+        let inicio = Instant::now();
+        let r = esperar_con_plazo(&mut hijo, Duration::from_secs(1))?;
+        assert!(r.is_none(), "{r:?}");
+        let eof = rx.recv_timeout(Duration::from_secs(5));
+        assert!(
+            eof.is_ok(),
+            "la tuberia sigue abierta tras {:?}",
+            inicio.elapsed()
+        );
+        Ok(())
+    }
+}
diff --git a/crates/tc2/src/progreso/calculo.rs b/crates/tc2/src/progreso/calculo.rs
index 85968e3..1f76106 100644
--- a/crates/tc2/src/progreso/calculo.rs
+++ b/crates/tc2/src/progreso/calculo.rs
@@ -156,7 +156,7 @@ pub(crate) mod tests {
         for (s, f) in filas {
             texto.push_str(&format!(
                 "| {} | {f} | manual | persona | 0.1 |\n",
-                crate::toma::instante(T0 + s)
+                crate::fecha::instante(T0 + s)
             ));
         }
         (t, Cuerpo::leer(&texto))
diff --git a/crates/tc2/src/progreso/fichero.rs b/crates/tc2/src/progreso/fichero.rs
index 7ba8937..2713e83 100644
--- a/crates/tc2/src/progreso/fichero.rs
+++ b/crates/tc2/src/progreso/fichero.rs
@@ -12,7 +12,6 @@
 //! Lo que se lee es texto no confiable (otro proceso o una persona puede
 //! tocarlo): se valida al leer y se sanea al pintar.
 
-use std::io::Read;
 use std::path::{Path, PathBuf};
 
 use serde::{Deserialize, Serialize};
@@ -123,13 +122,7 @@ pub fn ruta_anterior(carpeta: &Path, id: &str) -> PathBuf {
 
 /// Bytes de `ruta`, con tope; `None` si no se lee o pasa de [`LIMITE`].
 fn bytes(ruta: &Path) -> Option<Vec<u8>> {
-    let mut v = Vec::new();
-    std::fs::File::open(ruta)
-        .ok()?
-        .take(LIMITE + 1)
-        .read_to_end(&mut v)
-        .ok()?;
-    (v.len() as u64 <= LIMITE).then_some(v)
+    crate::lectura::con_tope(ruta, LIMITE).ok()?
 }
 
 /// Un progreso valido de `ruta`, si lo hay.
@@ -187,7 +180,7 @@ pub enum Anterior {
 }
 
 /// Escribe `p` en `carpeta`: antes, el que habia pasa a `.anterior.json`.
-/// Cada paso, de una vez ([`crate::huella::escribir_local`]). Si el anterior
+/// Cada paso, de una vez (`crate::escritura::atomica`). Si el anterior
 /// no se puede escribir, el progreso se escribe igual (revision 1 de
 /// TASK-084, R1-04): el anterior solo es el respaldo de un fichero a medias,
 /// y sin el nuevo la linea se quedaria con el dato viejo. Lo que paso con el
@@ -204,7 +197,7 @@ pub fn escribir(carpeta: &Path, p: &Progreso) -> std::io::Result<Anterior> {
         // (TASK-128), y el anterior dejaria de ser el fichero que habia.
         match bytes(&r).filter(|b| valido(b, &p.id)) {
             Some(previo) => {
-                match crate::huella::escribir_local(&ruta_anterior(carpeta, &p.id), &previo) {
+                match crate::escritura::atomica(&ruta_anterior(carpeta, &p.id), &previo) {
                     Ok(()) => Anterior::Rotado,
                     Err(_) => Anterior::NoEscrito,
                 }
@@ -212,7 +205,7 @@ pub fn escribir(carpeta: &Path, p: &Progreso) -> std::io::Result<Anterior> {
             None => Anterior::NoEscrito,
         }
     };
-    crate::huella::escribir_local(&r, &p.json())?;
+    crate::escritura::atomica(&r, &p.json())?;
     Ok(anterior)
 }
 
diff --git a/crates/tc2/src/progreso/mod.rs b/crates/tc2/src/progreso/mod.rs
index 03f0cda..c410496 100644
--- a/crates/tc2/src/progreso/mod.rs
+++ b/crates/tc2/src/progreso/mod.rs
@@ -49,13 +49,8 @@ pub struct Repo {
 
 /// Texto de un fichero pequeno, sin el salto final.
 fn texto_corto(p: &Path) -> Option<String> {
-    let mut s = String::new();
-    std::fs::File::open(p)
-        .ok()?
-        .take(LIMITE_GIT + 1)
-        .read_to_string(&mut s)
-        .ok()?;
-    (s.len() as u64 <= LIMITE_GIT).then(|| s.trim_end_matches(['\r', '\n']).to_owned())
+    let s = crate::lectura::texto_con_tope(p, LIMITE_GIT).ok()??;
+    Some(s.trim_end_matches(['\r', '\n']).to_owned())
 }
 
 /// `rel` relativo a `base` si no es absoluto.
diff --git a/crates/tc2/src/revision/operacion.rs b/crates/tc2/src/revision/operacion.rs
index 022eca3..96511de 100644
--- a/crates/tc2/src/revision/operacion.rs
+++ b/crates/tc2/src/revision/operacion.rs
@@ -26,10 +26,10 @@ use std::cell::RefCell;
 use crate::estado::{
     AnotacionConDueno, Cambio, Estado, Fallo, Operacion, por_estado, tarea_propia,
 };
+use crate::fecha::instante;
 use crate::i18n::catalogo::revision as txt;
 use crate::identidad::Quien;
 use crate::tarea::{Asignado, Transicion};
-use crate::toma::instante;
 use crate::toma::operacion::{EN_CURSO, MODO};
 
 use super::NOMBRE;
diff --git a/crates/tc2/src/seleccion/orden.rs b/crates/tc2/src/seleccion/orden.rs
index f20d576..617800b 100644
--- a/crates/tc2/src/seleccion/orden.rs
+++ b/crates/tc2/src/seleccion/orden.rs
@@ -45,7 +45,6 @@
 //! recibe) no cabe en claves del contrato F1-4: va, con `--detalle`, a un
 //! JSON `{"revisoras":[{"id","skill","rutas"}]}` que lee el workflow.
 
-use std::io::Read;
 use std::path::{Path, PathBuf};
 
 use super::catalogo::{self, Catalogo};
@@ -78,10 +77,10 @@ pub struct Entorno<'a> {
     pub git: &'a Git,
     /// Carpeta de Claude Code, si se sabe.
     pub claude: Option<&'a Path>,
-    /// Lee la entrada estandar (para `--rutas -`): como mucho
-    /// `TOPE_RUTAS + 1` bytes, para que el exceso se vea y falle igual que
-    /// con fichero.
-    pub stdin: &'a dyn Fn() -> std::io::Result<Vec<u8>>,
+    /// Lee la entrada estandar (para `--rutas -`) con un tope de
+    /// [`TOPE_RUTAS`] bytes (`crate::lectura::acotado`): `None` si pasa, y
+    /// la orden falla igual que con fichero.
+    pub stdin: &'a dyn Fn() -> std::io::Result<Option<Vec<u8>>>,
 }
 
 /// Inventario y catalogo con los que se selecciona, y los avisos de
@@ -281,20 +280,11 @@ pub fn detalle(s: &Seleccion, cat: &Catalogo) -> String {
     serde_json::json!({ "revisoras": v }).to_string()
 }
 
-/// Lee como mucho `tope` bytes de `p`: `Ok(None)` si pasa del tope.
-fn leer_con_tope(p: &Path, tope: u64) -> std::io::Result<Option<String>> {
-    let mut b = Vec::new();
-    std::fs::File::open(p)?.take(tope + 1).read_to_end(&mut b)?;
-    if b.len() as u64 > tope {
-        return Ok(None);
-    }
-    Ok(Some(String::from_utf8_lossy(&b).into_owned()))
-}
-
 /// Estado del informe de verificacion y, si vale, su seccion para el
 /// revisor (artefacto `artefacto`).
 pub fn informe_revisor(p: &Path, artefacto: &str) -> (&'static str, Option<String>) {
-    match leer_con_tope(p, TOPE_INFORME) {
+    match crate::lectura::texto_con_tope(p, TOPE_INFORME) {
+        Err(e) if e.kind() == std::io::ErrorKind::InvalidData => ("invalido", None),
         Err(_) => ("sin-informe", None),
         Ok(None) => ("grande", None),
         Ok(Some(t)) => match informe::leer(&t) {
@@ -427,13 +417,12 @@ fn paso(a: &[String], e: &Entorno<'_>) -> Result<Informe, Informe> {
     };
     let texto = match a.rutas.as_deref() {
         Some("-") => {
-            let b = (e.stdin)().map_err(|x| fallo_lectura("-", x.to_string()))?;
-            if b.len() as u64 > TOPE_RUTAS {
-                return Err(fallo_lectura("-", format!("mas de {TOPE_RUTAS} bytes")));
-            }
+            let b = (e.stdin)()
+                .map_err(|x| fallo_lectura("-", x.to_string()))?
+                .ok_or_else(|| fallo_lectura("-", format!("mas de {TOPE_RUTAS} bytes")))?;
             String::from_utf8(b).map_err(|x| fallo_lectura("-", x.to_string()))?
         }
-        Some(f) => leer_con_tope(Path::new(f), TOPE_RUTAS)
+        Some(f) => crate::lectura::texto_con_tope(Path::new(f), TOPE_RUTAS)
             .map_err(|x| fallo_lectura(f, x.to_string()))?
             .ok_or_else(|| fallo_lectura(f, format!("mas de {TOPE_RUTAS} bytes")))?,
         None => return Err(uso("sin-rutas")),
@@ -580,7 +569,7 @@ fn paso_seleccion(a: &[String], e: &Entorno<'_>) -> Result<Informe, Informe> {
     };
     let (t, c) = match (&fichero, &id) {
         (Some(f), _) => {
-            let texto = leer_con_tope(Path::new(f), TOPE_RUTAS)
+            let texto = crate::lectura::texto_con_tope(Path::new(f), TOPE_RUTAS)
                 .map_err(|x| error(f, x.to_string()))?
                 .ok_or_else(|| error(f, format!("mas de {TOPE_RUTAS} bytes")))?;
             tarea_de_fichero(&texto).map_err(|x| error(f, x))?
@@ -596,7 +585,7 @@ fn paso_seleccion(a: &[String], e: &Entorno<'_>) -> Result<Informe, Informe> {
     };
     let rutas = match &rutas_f {
         Some(f) => separar_rutas(
-            &leer_con_tope(Path::new(f), TOPE_RUTAS)
+            &crate::lectura::texto_con_tope(Path::new(f), TOPE_RUTAS)
                 .map_err(|x| error(f, x.to_string()))?
                 .ok_or_else(|| error(f, format!("mas de {TOPE_RUTAS} bytes")))?,
             false,
@@ -662,3 +651,26 @@ fn paso_seleccion(a: &[String], e: &Entorno<'_>) -> Result<Informe, Informe> {
         .con_lista("revision-reglas", rv.reglas.clone())
         .con_lista("avisos", avisos))
 }
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    /// Decision 3 de TASK-150: un informe que no es UTF-8 es `invalido`, no
+    /// `sin-informe`; uno que no existe sigue siendo `sin-informe`.
+    #[test]
+    fn informe_no_utf8_es_invalido() -> std::io::Result<()> {
+        let dir = std::env::temp_dir()
+            .join("tc2-tests")
+            .join(format!("seleccion-informe-{}", std::process::id()));
+        std::fs::create_dir_all(&dir)?;
+        let malo = dir.join("informe.json");
+        std::fs::write(&malo, [b'{', 0xff, b'}'])?;
+        let r = informe_revisor(&malo, "artefacto");
+        let falta = informe_revisor(&dir.join("no-existe.json"), "artefacto");
+        let _ = std::fs::remove_dir_all(&dir);
+        assert_eq!(r, ("invalido", None));
+        assert_eq!(falta, ("sin-informe", None));
+        Ok(())
+    }
+}
diff --git a/crates/tc2/src/toma/mod.rs b/crates/tc2/src/toma/mod.rs
index 28d2c10..aa22f09 100644
--- a/crates/tc2/src/toma/mod.rs
+++ b/crates/tc2/src/toma/mod.rs
@@ -41,7 +41,7 @@ use crate::i18n::catalogo::toma as txt;
 use crate::identidad::Quien;
 use crate::solape::{self, EnCurso, Solape};
 
-pub use operacion::{Tomar, desde_anotacion, instante};
+pub use operacion::{Tomar, desde_anotacion};
 
 /// Nombre de la orden y de la operacion de la rama de estado.
 pub const NOMBRE: &str = "start";
diff --git a/crates/tc2/src/toma/operacion.rs b/crates/tc2/src/toma/operacion.rs
index d89aa9d..2aae64a 100644
--- a/crates/tc2/src/toma/operacion.rs
+++ b/crates/tc2/src/toma/operacion.rs
@@ -230,7 +230,7 @@ impl Operacion for Tomar {
         let h = crate::estimacion::historico(&leidas, &self.commits);
         let e = crate::estimacion::recalcular(&mut t, &cuerpo, &h, self.ahora);
         *self.estimacion.borrow_mut() = Some(e);
-        let instante = instante(self.ahora);
+        let instante = crate::fecha::instante(self.ahora);
         EN_CURSO.clone_into(&mut t.estado);
         t.asignado_a = Some(self.asignado.clone());
         t.actualizado = instante.get(..10).unwrap_or_default().to_owned();
@@ -250,19 +250,6 @@ impl Operacion for Tomar {
     }
 }
 
-/// `AAAA-MM-DDTHH:MM:SSZ` (UTC) de `segundos` desde 1970.
-pub fn instante(segundos: u64) -> String {
-    let dias = i64::try_from(segundos / 86_400).unwrap_or(0);
-    let resto = segundos % 86_400;
-    format!(
-        "{}T{:02}:{:02}:{:02}Z",
-        crate::alta::fecha_de_dias(dias),
-        resto / 3600,
-        resto % 3600 / 60,
-        resto % 60
-    )
-}
-
 /// Reconstruye un [`Tomar`] desde su anotacion `start|TASK-NNN|<id>|<login>`
 /// (para el catalogo). `start` exige red y nunca queda pendiente; si se
 /// re-ejecutara, lo haria el CLI sin persona delante: `decidido_por: agente`.
@@ -311,13 +298,6 @@ mod tests {
         t.escribir(&Cuerpo::plantilla())
     }
 
-    #[test]
-    fn instante_en_utc() {
-        assert_eq!(instante(0), "1970-01-01T00:00:00Z");
-        assert_eq!(instante(AHORA), "2026-11-17T10:20:30Z");
-        assert_eq!(instante(86_399), "1970-01-01T23:59:59Z");
-    }
-
     /// C7 (unitario): la toma escribe estado, asignado, actualizado y una
     /// fila de transicion; el resto del `tarea.md`, byte a byte.
     #[test]
diff --git a/crates/tc2/src/verificacion/ejecutar.rs b/crates/tc2/src/verificacion/ejecutar.rs
index 939de95..cf27b5c 100644
--- a/crates/tc2/src/verificacion/ejecutar.rs
+++ b/crates/tc2/src/verificacion/ejecutar.rs
@@ -45,9 +45,6 @@ const LINEAS_RESUMEN: usize = 15;
 /// Tope de lo que se lee de stdin con `--hook`.
 const LIMITE_STDIN: u64 = 1 << 20;
 
-/// Cada cuanto se mira si el comando termino.
-const SONDEO: Duration = Duration::from_millis(10);
-
 /// `tc2 verificar` con los argumentos ya sin `--json`.
 pub fn orden(args: &[String], json: bool, idioma: Idioma) -> Salida {
     // Con `--hook`, nada que no sea el rojo de la rapida bloquea la parada:
@@ -372,48 +369,14 @@ fn lanzar(
             c.stdout(std::io::stderr()).stderr(Stdio::inherit());
         }
     }
-    #[cfg(unix)]
-    {
-        use std::os::unix::process::CommandExt;
-        c.process_group(0);
-    }
-    let mut hijo = c.spawn()?;
-    let limite = Instant::now() + Duration::from_secs(u64::from(timeout));
-    loop {
-        if let Some(s) = hijo.try_wait()? {
-            return Ok(Fin::Codigo(s.code().unwrap_or(-1)));
-        }
-        if Instant::now() >= limite {
-            matar_arbol(hijo.id());
-            let _ = hijo.kill();
-            let _ = hijo.wait();
-            return Ok(Fin::Timeout);
-        }
-        std::thread::sleep(SONDEO);
-    }
-}
-
-/// Mata el comando y lo que haya lanzado (la shell sola dejaria vivos a sus
-/// hijos).
-fn matar_arbol(pid: u32) {
-    let pid = pid.to_string();
-    #[cfg(windows)]
-    let mut c = {
-        let mut c = Command::new("taskkill");
-        c.args(["/T", "/F", "/PID", &pid]);
-        c
-    };
-    #[cfg(not(windows))]
-    let mut c = {
-        let mut c = Command::new("kill");
-        c.args(["-KILL", "--", &format!("-{pid}")]);
-        c
-    };
-    let _ = c
-        .stdin(Stdio::null())
-        .stdout(Stdio::null())
-        .stderr(Stdio::null())
-        .status();
+    // Al plazo cae el comando y lo que haya lanzado (la shell sola dejaria
+    // vivos a sus hijos).
+    let mut hijo = crate::proceso::grupo_propio(&mut c).spawn()?;
+    let plazo = Duration::from_secs(u64::from(timeout));
+    Ok(match crate::proceso::esperar_con_plazo(&mut hijo, plazo)? {
+        Some(s) => Fin::Codigo(s.code().unwrap_or(-1)),
+        None => Fin::Timeout,
+    })
 }
 
 /// Si el informe es un rojo de la verificacion (y no un «no se ejecuta»).
diff --git a/xtask/src/main.rs b/xtask/src/main.rs
index dbc8c10..9f04280 100644
--- a/xtask/src/main.rs
+++ b/xtask/src/main.rs
@@ -426,7 +426,8 @@ fn commit_actual() -> Option<String> {
 }
 
 /// `AAAA-MM-DDTHH:MM:SSZ` de unos segundos Unix (algoritmo de dias civiles de
-/// H. Hinnant).
+/// H. Hinnant). Copia de `tc2::fecha::instante` a proposito (A8-03): xtask
+/// solo depende de `distribucion`, no de `tc2`.
 fn fecha_utc(segundos: u64) -> String {
     let dias = segundos / 86_400;
     let resto = segundos % 86_400;
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/01-en-diseno/TASK-150/tarea.md                                                     | 47 -----------------------------------------------
 tareas/{01-en-diseno => 02-en-curso}/TASK-150/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-150/planificacion/plan-final.md                 |  0
 tareas/02-en-curso/TASK-150/tarea.md                                                      | 78 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 4 files changed, 78 insertions(+), 47 deletions(-)
````

## Tests del diff (5 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/comun/remoto.rs" ":(literal)crates/tc2/tests/comun/tareas.rs" ":(literal)crates/tc2/tests/identidades.rs" ":(literal)crates/tc2/tests/soporte.rs" ":(literal)crates/tc2/tests/toma.rs"
````

````
crates/tc2/tests/comun/remoto.rs |  2 +-
 crates/tc2/tests/comun/tareas.rs |  2 +-
 crates/tc2/tests/identidades.rs  | 34 ++++++++--------------------------
 crates/tc2/tests/soporte.rs      |  2 +-
 crates/tc2/tests/toma.rs         |  2 +-
 5 files changed, 12 insertions(+), 30 deletions(-)
````
## Candidatos 4D

Pistas generadas desde el diff, no hallazgos: comprueba cada una (Regla 4D de `revision.md`).

### Posible duplicacion con lo que ya existia

Definiciones nuevas con nombre igual o parecido a otra de la rama base. Mira si hacen lo mismo con pequenas diferencias (duplicacion real = IMPORTANTE).

- `renombrar` (crates/tc2/src/escritura.rs:61) ~ `renombrar` (crates/tc2/src/git/rama.rs:459)
- `carpeta` (crates/tc2/src/escritura.rs:78) ~ `Carpeta` (crates/tc2/src/ciclo/mod.rs:395), `carpeta` (crates/tc2/src/estado/frescura.rs:339), `carpeta` (crates/tc2/src/estado/montaje.rs:326) y 8 mas
- `instante` (crates/tc2/src/fecha.rs:10) ~ `instante` (crates/tc2/src/toma/operacion.rs:254)
- `fecha_de_dias` (crates/tc2/src/fecha.rs:24) ~ `fecha_de_dias` (crates/tc2/src/alta.rs:738)
- `fechas_civiles` (crates/tc2/src/fecha.rs:43) ~ `fechas_civiles` (crates/tc2/src/alta.rs:1092)
- `instante_en_utc` (crates/tc2/src/fecha.rs:51) ~ `instante_utc` (crates/tc2/src/alta.rs:724), `instante_en_utc` (crates/tc2/src/toma/operacion.rs:315)
- `con_tope` (crates/tc2/src/lectura.rs:58) ~ `con_tope` (crates/tc2/src/plan/heuristica.rs:287)
- `matar_arbol` (crates/tc2/src/proceso.rs:62) ~ `matar_arbol` (crates/tc2/src/verificacion/ejecutar.rs:398)

