# Peticion de revision — TASK-115 (ronda 1)

- Tarea: TASK-115 — Seguimiento de F2-12: menores de actualizar --cli y mock HTTP estable
- Rama revisada: fix/task-115-seguimiento-de-f2-12-menores-de-actualiz
- Rama base: develop
- Commit revisado (HEAD): 71fb3c554a0f6ab6091d54200bdeb4ba74520ce5
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-115 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-115/tarea.md
- tareas/03-en-revision/TASK-115/planificacion/
- crates/tc2/src/actualizar_cli.rs
- crates/tc2/src/descarga.rs
- crates/tc2/tests/actualizar_cli.rs
- crates/tc2/tests/comun/http_mock.rs
- crates/tc2/tests/plataforma.rs
- crates/tc2/tests/plataforma_gitlab.rs

## Commits a revisar (git log develop..HEAD)

````
71fb3c5 test(TASK-115): evidencia del 10 de 10 con los tests del mock en serie
1e81d0e test(TASK-115): los tests con el mock HTTP van en serie con un cerrojo comun
789bf92 chore(TASK-115): resultado con la evidencia de las series y autorrevision
c99fb99 test(TASK-115): redireccion no permitida y binario mayor que el limite en actualizar --cli
cf24f2c test(TASK-115): el test del mock cambia el plazo sobre el socket que lee
8766cc6 fix(TASK-115): TLS y plazo global no se reintentan, salida del hijo con nieto y doc del reintento
e28e303 test(TASK-115): el mock HTTP comun no manda FIN tras una respuesta completa
ee0dc13 chore(TASK-115): tarea en curso
````

## Diff de tu dominio (6 fichero(s) de 6; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/actualizar_cli.rs b/crates/tc2/src/actualizar_cli.rs
index fb05d3a..3d88c22 100644
--- a/crates/tc2/src/actualizar_cli.rs
+++ b/crates/tc2/src/actualizar_cli.rs
@@ -508,34 +508,58 @@ fn ultima_linea(bytes: &[u8]) -> Option<String> {
 /// 1 MiB cada uno, para que un hijo que escribe mucho no se bloquee.
 ///
 /// El plazo vale para todo, tambien para leer la salida: si el hijo deja un
-/// nieto con las tuberias heredadas (un `.bat` que lanza otro proceso), los
-/// hilos lectores pueden seguir bloqueados despues de que el hijo acabe o se
-/// mate. No se les espera mas alla del plazo: se devuelve lo leido hasta
-/// entonces y los hilos acaban solos cuando el nieto cierre las tuberias (o
-/// con el proceso). Al nieto no se le mata: no hay una forma portable de
-/// matar el arbol sin dependencias.
+/// nieto con las tuberias heredadas (un `.bat` que lanza otro proceso), las
+/// tuberias no se cierran al acabar el hijo y los hilos lectores siguen
+/// bloqueados. Entonces se espera al cierre hasta que se cumple el plazo (no
+/// mas) y se devuelve lo leido hasta ese momento: la salida del hijo, que
+/// acabo, no se pierde, pero la llamada tarda el plazo entero. Los hilos
+/// acaban solos cuando el nieto cierre las tuberias (o con el proceso). Al
+/// nieto no se le mata: no hay una forma portable de matar el arbol sin
+/// dependencias.
 pub(crate) fn ejecutar_con_limite(
     mut cmd: Command,
     limite: Duration,
 ) -> std::io::Result<Option<Output>> {
     use std::io::Read;
-    use std::sync::mpsc;
+    use std::sync::{Arc, Mutex, PoisonError, mpsc};
+    const MAXIMO: usize = 1024 * 1024;
     let mut hijo = cmd
         .stdin(Stdio::null())
         .stdout(Stdio::piped())
         .stderr(Stdio::piped())
         .spawn()?;
+    // Cada hilo va dejando lo leido en un buffer compartido y avisa al ver
+    // el final de la tuberia; quien espera se lleva lo que haya, acabe o no.
     let leer = |p: Option<Box<dyn Read + Send>>| {
-        let (tx, rx) = mpsc::channel();
+        let leido: Arc<Mutex<Vec<u8>>> = Arc::default();
+        let (tx, rx) = mpsc::channel::<()>();
+        let destino = leido.clone();
         std::thread::spawn(move || {
-            let mut v = Vec::new();
-            if let Some(p) = p {
-                let _ = p.take(1024 * 1024).read_to_end(&mut v);
+            if let Some(mut p) = p {
+                let mut trozo = [0u8; 8192];
+                loop {
+                    let n = match p.read(&mut trozo) {
+                        Ok(0) => break,
+                        Ok(n) => n,
+                        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
+                        Err(_) => break,
+                    };
+                    let mut v = destino.lock().unwrap_or_else(PoisonError::into_inner);
+                    let cabe = MAXIMO.saturating_sub(v.len()).min(n);
+                    v.extend_from_slice(trozo.get(..cabe).unwrap_or_default());
+                    if v.len() >= MAXIMO {
+                        break;
+                    }
+                }
             }
             // Si nadie espera ya (plazo cumplido), se descarta.
-            let _ = tx.send(v);
+            let _ = tx.send(());
         });
-        rx
+        (leido, rx)
+    };
+    let recoger = |(leido, rx): (Arc<Mutex<Vec<u8>>>, mpsc::Receiver<()>), espera: Duration| {
+        let _ = rx.recv_timeout(espera);
+        std::mem::take(&mut *leido.lock().unwrap_or_else(PoisonError::into_inner))
     };
     let out = leer(
         hijo.stdout
@@ -572,8 +596,8 @@ pub(crate) fn ejecutar_con_limite(
         fin.saturating_duration_since(Instant::now())
             .max(Duration::from_millis(100))
     };
-    let stdout = out.recv_timeout(restante()).unwrap_or_default();
-    let stderr = err.recv_timeout(restante()).unwrap_or_default();
+    let stdout = recoger(out, restante());
+    let stderr = recoger(err, restante());
     Ok(status.map(|status| Output {
         status,
         stdout,
@@ -597,18 +621,19 @@ fn fallo_de_escritura(ruta: &Path, tipo: std::io::ErrorKind, error: &str) -> Inf
         )
 }
 
-/// Baja el binario a `fichero` y, ante un fallo de transporte transitorio
-/// (no conecta, la conexion se corta antes o despues de las cabeceras,
-/// tiempo agotado), lo vacia y lo intenta una vez mas. Son unos MiB por una
-/// red que puede fallar a medias; y en una maquina de desarrollo con Windows
-/// se ha visto (TASK-027) que la red local corta a veces a los ~19 s las
-/// descargas de mas de ~450 KB. No se reintentan los fallos que se
-/// repetirian igual: 4xx, 5xx, TLS, una redireccion no permitida, el limite
-/// de tamano y una respuesta invalida.
 /// Pausa antes del reintento de la descarga.
 const PAUSA_REINTENTO: Duration = Duration::from_secs(1);
 
-fn bajar_con_un_reintento(
+/// Baja el binario a `fichero` y, ante un fallo de transporte transitorio
+/// (no conecta, la conexion se corta antes o despues de las cabeceras, plazo
+/// de conexion agotado; ver [`descarga::es_transitorio`]), lo vacia y lo
+/// intenta una vez mas. Son unos MiB por una red que puede fallar a medias;
+/// y en una maquina de desarrollo con Windows se ha visto (TASK-027) que la
+/// red local corta a veces a los ~19 s las descargas de mas de ~450 KB. No
+/// se reintentan los fallos que se repetirian igual: 4xx, 5xx, TLS, una
+/// redireccion no permitida, el limite de tamano y una respuesta invalida;
+/// ni el plazo global agotado, que doblaria la espera.
+pub(crate) fn bajar_con_un_reintento(
     d: &Descargador,
     url: &str,
     fichero: &mut std::fs::File,
@@ -795,6 +820,84 @@ mod tests {
         );
     }
 
+    /// TASK-115 (MEN-7 de TASK-027): si el hijo acaba bien pero deja un
+    /// nieto con stdout heredado, la llamada vuelve al cumplirse el plazo con
+    /// la salida que el hijo ya escribio. Antes se perdia (stdout vacio). El
+    /// nieto dura poco (unos 5 s) para no dejarlo huerfano mucho tiempo.
+    #[test]
+    fn ejecutar_con_limite_devuelve_lo_leido_si_un_nieto_retiene_stdout() {
+        let mut cmd = if cfg!(windows) {
+            let mut c = Command::new("cmd");
+            c.args(["/c", "start /b ping -n 6 127.0.0.1 & echo hola"]);
+            c
+        } else {
+            let mut c = Command::new("sh");
+            c.args(["-c", "sleep 5 & echo hola"]);
+            c
+        };
+        cmd.env("TC2_PRUEBA", "1");
+        let inicio = Instant::now();
+        let r = ejecutar_con_limite(cmd, Duration::from_secs(2));
+        let tardo = inicio.elapsed();
+        let s = r.ok().flatten();
+        assert_eq!(s.as_ref().map(|o| o.status.success()), Some(true));
+        let stdout = s
+            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
+            .unwrap_or_default();
+        assert!(
+            stdout.lines().any(|l| l.trim() == "hola"),
+            "stdout: {stdout:?}"
+        );
+        assert!(tardo < Duration::from_secs(4), "tardo {tardo:?}");
+    }
+
+    /// TASK-115: el doc de `bajar_con_un_reintento` esta pegado a su funcion
+    /// (no detras de una `const`, que se lo quedaria) y sin lineas repetidas.
+    #[test]
+    fn el_doc_de_bajar_con_un_reintento_esta_sobre_su_funcion() {
+        let fuente = include_str!("actualizar_cli.rs");
+        let firma = concat!("fn ", "bajar_con_un_reintento(");
+        let lineas: Vec<&str> = fuente.lines().collect();
+        let pos = lineas.iter().position(|l| {
+            let l = l.trim_start();
+            l.strip_prefix("pub(crate) ")
+                .unwrap_or(l)
+                .starts_with(firma)
+        });
+        assert!(pos.is_some(), "no esta la funcion");
+        let pos = pos.unwrap_or_default();
+        let doc: Vec<&str> = lineas
+            .get(..pos)
+            .unwrap_or_default()
+            .iter()
+            .rev()
+            .take_while(|l| l.trim_start().starts_with("///"))
+            .map(|l| l.trim())
+            .collect();
+        assert!(
+            doc.iter()
+                .any(|l| !l.trim_start_matches('/').trim().is_empty()),
+            "sin doc justo encima de la funcion"
+        );
+        let mut vistas = std::collections::BTreeSet::new();
+        for l in &doc {
+            assert!(
+                l.trim_start_matches('/').trim().is_empty() || vistas.insert(*l),
+                "linea repetida: {l}"
+            );
+        }
+        // Lo de encima del bloque no es una `const` con el doc colgando.
+        let encima = lineas
+            .get(..pos.saturating_sub(doc.len()))
+            .and_then(|a| a.last())
+            .map(|l| l.trim())
+            .unwrap_or_default();
+        assert!(
+            !encima.starts_with("const ") && !encima.starts_with("pub const "),
+            "el doc viene detras de: {encima}"
+        );
+    }
+
     /// Revision 1, IMP-1: que `gh` se usa, segun `TC2_GH` y el origen.
     #[test]
     fn gh_para_la_attestation() {
diff --git a/crates/tc2/src/descarga.rs b/crates/tc2/src/descarga.rs
index f3c10f8..7e5354d 100644
--- a/crates/tc2/src/descarga.rs
+++ b/crates/tc2/src/descarga.rs
@@ -102,9 +102,10 @@ pub enum Fallo {
         /// Error del cliente.
         error: String,
         /// Si es un fallo de transporte transitorio, antes o despues de las
-        /// cabeceras (no conecta, se corta, tiempo agotado, cuerpo parado):
-        /// lo unico que se reintenta. TLS y una respuesta invalida se
-        /// repetirian igual.
+        /// cabeceras (no resuelve, no conecta, se corta, tiempo de conexion
+        /// agotado): lo unico que se reintenta (ver [`es_transitorio`]). TLS,
+        /// una respuesta invalida y el plazo global agotado no lo son: se
+        /// repetirian igual, y repetir el plazo global doblaria la espera.
         transitorio: bool,
     },
     /// 404: no existe esa release o ese fichero.
@@ -220,6 +221,39 @@ pub fn redireccion_permitida(destino: &str) -> bool {
     HOSTS_PERMITIDOS.contains(&host)
 }
 
+/// Si un error de ureq es un fallo de transporte que puede no repetirse y
+/// merece un reintento:
+///
+/// - si: no resuelve, no conecta, se corta (E/S), el plazo de conexion u
+///   otro plazo parcial, cuerpo parado;
+/// - no: el plazo global (`ESPERA_DESCARGA`) agotado, porque repetirlo
+///   doblaria la espera (TASK-027, MEN-9); TLS, incluido el handshake, que
+///   rustls devuelve como E/S de tipo `InvalidData` (MEN-8); y el resto.
+///
+/// Un cierre de la otra parte a mitad del handshake llega como un fin de
+/// datos (`UnexpectedEof`) que no se distingue de un corte, y se reintenta.
+pub fn es_transitorio(e: &ureq::Error) -> bool {
+    match e {
+        ureq::Error::Timeout(ureq::Timeout::Global) => false,
+        ureq::Error::Io(io) => es_transitorio_io(io),
+        ureq::Error::HostNotFound
+        | ureq::Error::ConnectionFailed
+        | ureq::Error::Timeout(_)
+        | ureq::Error::BodyStalled => true,
+        _ => false,
+    }
+}
+
+/// Como [`es_transitorio`] para un error de E/S: el de la lectura del cuerpo
+/// (ureq envuelve en el sus propios errores) o el del transporte.
+fn es_transitorio_io(e: &std::io::Error) -> bool {
+    if let Some(u) = e.get_ref().and_then(|x| x.downcast_ref::<ureq::Error>()) {
+        return es_transitorio(u);
+    }
+    // `InvalidData`: rustls rechazo lo recibido (certificado, protocolo).
+    e.kind() != std::io::ErrorKind::InvalidData
+}
+
 /// Quien baja ficheros de la release.
 pub struct Descargador {
     agente: ureq::Agent,
@@ -240,8 +274,14 @@ impl Default for Descargador {
 impl Descargador {
     /// Con el agente de `tc2` y el tiempo de una descarga.
     pub fn nuevo() -> Self {
+        Self::con_espera(ESPERA_DESCARGA)
+    }
+
+    /// Con el agente de `tc2` y `espera` como plazo global de cada descarga
+    /// (los tests lo acortan).
+    pub(crate) fn con_espera(espera: Duration) -> Self {
         Self {
-            agente: http::agente(ESPERA_DESCARGA),
+            agente: http::agente(espera),
         }
     }
 
@@ -273,10 +313,12 @@ impl Descargador {
             let mut r = match self.agente.get(&actual).call() {
                 Ok(r) => r,
                 Err(e) => {
+                    let transitorio = es_transitorio(&e);
                     return Err(match http::clasificar(e) {
                         // DNS, conexion, tiempo agotado, E/S o cuerpo
-                        // parado: transitorio.
-                        FalloHttp::Sinred(e) => sinred(e, true),
+                        // parado; transitorio salvo el plazo global y TLS
+                        // dentro de la E/S.
+                        FalloHttp::Sinred(e) => sinred(e, transitorio),
                         FalloHttp::Tls(e)
                         | FalloHttp::Otro(e)
                         | FalloHttp::RespuestaInvalida(e) => sinred(e, false),
@@ -340,7 +382,7 @@ impl Descargador {
                     Ok(0) => break,
                     Ok(n) => n,
                     Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
-                    Err(e) => return Err(sinred(e.to_string(), true)),
+                    Err(e) => return Err(sinred(e.to_string(), es_transitorio_io(&e))),
                 };
                 total = total.saturating_add(n as u64);
                 if total > limite {
@@ -365,6 +407,119 @@ impl Descargador {
 #[cfg(test)]
 mod tests {
     use super::*;
+    use std::net::{TcpListener, TcpStream};
+    use std::sync::Arc;
+    use std::sync::atomic::{AtomicUsize, Ordering};
+    use std::time::Instant;
+
+    /// Servidor en 127.0.0.1 que atiende cada conexion con `atender` y las
+    /// cuenta.
+    fn servidor(atender: fn(TcpStream)) -> Option<(u16, Arc<AtomicUsize>)> {
+        let l = TcpListener::bind("127.0.0.1:0").ok()?;
+        let puerto = l.local_addr().ok()?.port();
+        let conexiones = Arc::new(AtomicUsize::new(0));
+        let c = conexiones.clone();
+        std::thread::spawn(move || {
+            for s in l.incoming().flatten() {
+                c.fetch_add(1, Ordering::SeqCst);
+                std::thread::spawn(move || atender(s));
+            }
+        });
+        Some((puerto, conexiones))
+    }
+
+    /// Espera a que el cliente cierre (como mucho 10 s), sin mandar FIN.
+    fn esperar_cierre(mut s: TcpStream) {
+        let _ = s.set_read_timeout(Some(Duration::from_secs(10)));
+        let _ = s.read_to_end(&mut Vec::new());
+    }
+
+    /// Fichero de destino de una prueba en la carpeta temporal.
+    struct Temporal(std::path::PathBuf);
+
+    impl Temporal {
+        fn nuevo(nombre: &str) -> Option<(Self, std::fs::File)> {
+            let ruta = std::env::temp_dir()
+                .join(format!("tc2-descarga-{nombre}-{}.tmp", std::process::id()));
+            let f = std::fs::File::create(&ruta).ok()?;
+            Some((Self(ruta), f))
+        }
+    }
+
+    impl Drop for Temporal {
+        fn drop(&mut self) {
+            let _ = std::fs::remove_file(&self.0);
+        }
+    }
+
+    /// TASK-115 (MEN-8 de TASK-027): un fallo de TLS en el handshake no es
+    /// transitorio y la descarga del binario no lo reintenta. El servidor
+    /// responde en claro a una peticion `https`: rustls lo rechaza.
+    #[test]
+    fn tls_fallido_no_es_transitorio() {
+        let srv = servidor(|mut s| {
+            let _ = s.write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n");
+            let _ = s.flush();
+            esperar_cierre(s);
+        });
+        let tmp = Temporal::nuevo("tls");
+        assert!(srv.is_some() && tmp.is_some());
+        let (Some((puerto, conexiones)), Some((_t, mut f))) = (srv, tmp) else {
+            return;
+        };
+        let d = Descargador::con_espera(Duration::from_secs(10));
+        let url = format!("https://127.0.0.1:{puerto}/tc2");
+        let r = crate::actualizar_cli::bajar_con_un_reintento(&d, &url, &mut f);
+        assert!(
+            matches!(
+                r,
+                Err(Fallo::Sinred {
+                    transitorio: false,
+                    ..
+                })
+            ),
+            "{r:?}"
+        );
+        assert_eq!(conexiones.load(Ordering::SeqCst), 1, "{r:?}");
+    }
+
+    /// TASK-115 (MEN-9 de TASK-027): el plazo global agotado a mitad del
+    /// cuerpo no es transitorio: la descarga no se repite y la espera no se
+    /// dobla (margen de 1,5 veces el plazo).
+    #[test]
+    fn timeout_global_no_se_dobla() {
+        let srv = servidor(|mut s| {
+            let mut buf = [0u8; 4096];
+            let _ = s.read(&mut buf);
+            // Anuncia 100 bytes, manda 10 y se para.
+            let _ = s.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n0123456789");
+            let _ = s.flush();
+            esperar_cierre(s);
+        });
+        let tmp = Temporal::nuevo("plazo");
+        assert!(srv.is_some() && tmp.is_some());
+        let (Some((puerto, conexiones)), Some((_t, mut f))) = (srv, tmp) else {
+            return;
+        };
+        let espera = Duration::from_secs(2);
+        let d = Descargador::con_espera(espera);
+        let url = format!("http://127.0.0.1:{puerto}/tc2");
+        let inicio = Instant::now();
+        let r = crate::actualizar_cli::bajar_con_un_reintento(&d, &url, &mut f);
+        let tardo = inicio.elapsed();
+        assert!(
+            matches!(
+                r,
+                Err(Fallo::Sinred {
+                    transitorio: false,
+                    ..
+                })
+            ),
+            "{r:?}"
+        );
+        assert_eq!(conexiones.load(Ordering::SeqCst), 1, "{r:?}");
+        assert!(tardo < espera.mul_f32(1.5), "tardo {tardo:?}");
+    }
 
     #[test]
     fn solo_redirecciones_https_a_github() {
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/01-en-diseno/TASK-115/tarea.md                                                     |  41 ---------------------------------
 tareas/{01-en-diseno => 02-en-curso}/TASK-115/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{01-en-diseno => 02-en-curso}/TASK-115/planificacion/plan-final.md                 |   0
 tareas/02-en-curso/TASK-115/tarea.md                                                      | 126 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 4 files changed, 126 insertions(+), 41 deletions(-)
````

## Tests del diff (4 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/actualizar_cli.rs" ":(literal)crates/tc2/tests/comun/http_mock.rs" ":(literal)crates/tc2/tests/plataforma.rs" ":(literal)crates/tc2/tests/plataforma_gitlab.rs"
````

````
crates/tc2/tests/actualizar_cli.rs    | 117 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++----------
 crates/tc2/tests/comun/http_mock.rs   |  86 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++--
 crates/tc2/tests/plataforma.rs        |   6 +++++-
 crates/tc2/tests/plataforma_gitlab.rs |   5 ++++-
 4 files changed, 200 insertions(+), 14 deletions(-)
````
