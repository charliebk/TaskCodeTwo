# Peticion de revision — TASK-027 (ronda 3)

- Tarea: TASK-027 — F2-12 taskctl actualizar --cli y aviso de version nueva
- Rama revisada: feature/task-027-f2-12-taskctl-actualizar-cli-y-aviso-de
- Rama base: develop
- Commit revisado (HEAD): e3ac6d97508ab26cf4f7e713e7ed37534d286f2f
- Fecha: 2026-11-16
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-027 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 39ef7a1862bb2047f844adb168309dc4dd92d19c (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (8 hallazgo(s)))

## Instrucciones para el agente revisor (verificacion acotada)

Eres un revisor INDEPENDIENTE del agente que implemento. La rama ya paso una
revision completa: esta ronda verifica las correcciones de la ronda anterior,
no vuelve a revisar la rama entera ni los MENORES de la ronda anterior (tu
tabla solo trae CRITICOS e IMPORTANTES). Haz esto, y nada mas:

1. Comprueba, uno por uno, que cada hallazgo pendiente de tu tabla de abajo
   queda resuelto por el diff de esta ronda (los marcados `corregido` tambien:
   verificalo, no lo des por bueno). Para un CRITICO o IMPORTANTE, reproduce el
   fallo original y comprueba que ya no ocurre; puedes leer el repo entero si la
   causa no esta en el diff.
2. Ejecuta la suite (o el comando de verificacion del proyecto) sobre el commit
   revisado y di el resultado: verificar sin ejecutar no vale. Si esta peticion
   ya trae la verificacion de la ronda en verde para ese mismo commit, no la
   repitas: cita ese resultado y corre solo los tests de lo que verificas.
3. Busca en el diff de esta ronda el fallo que la correccion abre: es justo
   donde se cuelan los fallos nuevos. Si encuentras uno, construye el caso que
   lo rompe antes de reportarlo.

Solo abre un hallazgo NUEVO si es CRITICO o si lo introduce la propia correccion
(algo que el diff de esta ronda rompe). Lo demas que veas fuera de eso va como
nota al final del informe, no en la tabla: no abre otra ronda.

Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. En la tabla, cada hallazgo pendiente que ya este
resuelto va como `corregido`; el que no, `abierto`. Vuelca tu salida en el
informe de esta ronda (informe-revision-3-rust-wasm-reviewer.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-027/tarea.md
- tareas/03-en-revision/TASK-027/planificacion/
- crates/tc2/src/actualizar_cli.rs
- crates/tc2/src/descarga.rs
- crates/tc2/tests/actualizar_cli.rs

## Hallazgos de la ronda 2 pendientes de verificar

Tus 6 hallazgo(s) MENOR de la ronda 2 no se reverifican (los comprueba
quien coordina): recibes esta ronda porque la correccion
toca ficheros de tu dominio. Revisa en el diff de esta ronda lo que la correccion
cambia en tu terreno y si abre un fallo.

## Commits a revisar (git log 39ef7a1862bb2047f844adb168309dc4dd92d19c..HEAD)

````
e3ac6d9 chore(TASK-027): ronda extra 3
c8d4368 fix(TASK-027): pausa antes del reintento y servidor de prueba sin FIN tras la respuesta
9312b13 fix(TASK-027): reintento ante cortes antes de las cabeceras y plazo de ejecutar_con_limite
b99f39f chore(TASK-027): veredictos y coste de la revision ronda 2
ad8b2f1 chore(TASK-027): informes de revision ronda 2
b3d2e77 chore(TASK-027): peticion de revision ronda 2
````

## Diff desde la ronda anterior (git diff 39ef7a1862bb2047f844adb168309dc4dd92d19c..HEAD)

````diff
diff --git a/crates/tc2/src/actualizar_cli.rs b/crates/tc2/src/actualizar_cli.rs
index 5b308fb..abe8140 100644
--- a/crates/tc2/src/actualizar_cli.rs
+++ b/crates/tc2/src/actualizar_cli.rs
@@ -567,21 +567,33 @@ fn ultima_linea(bytes: &[u8]) -> Option<String> {
 /// Lanza `cmd` sin entrada y espera como mucho `limite`. `Ok(None)` si no
 /// termino a tiempo (se mata). stdout y stderr se leen en hilos aparte, hasta
 /// 1 MiB cada uno, para que un hijo que escribe mucho no se bloquee.
+///
+/// El plazo vale para todo, tambien para leer la salida: si el hijo deja un
+/// nieto con las tuberias heredadas (un `.bat` que lanza otro proceso), los
+/// hilos lectores pueden seguir bloqueados despues de que el hijo acabe o se
+/// mate. No se les espera mas alla del plazo: se devuelve lo leido hasta
+/// entonces y los hilos acaban solos cuando el nieto cierre las tuberias (o
+/// con el proceso). Al nieto no se le mata: no hay una forma portable de
+/// matar el arbol sin dependencias.
 fn ejecutar_con_limite(mut cmd: Command, limite: Duration) -> std::io::Result<Option<Output>> {
     use std::io::Read;
+    use std::sync::mpsc;
     let mut hijo = cmd
         .stdin(Stdio::null())
         .stdout(Stdio::piped())
         .stderr(Stdio::piped())
         .spawn()?;
     let leer = |p: Option<Box<dyn Read + Send>>| {
+        let (tx, rx) = mpsc::channel();
         std::thread::spawn(move || {
             let mut v = Vec::new();
             if let Some(p) = p {
                 let _ = p.take(1024 * 1024).read_to_end(&mut v);
             }
-            v
-        })
+            // Si nadie espera ya (plazo cumplido), se descarta.
+            let _ = tx.send(v);
+        });
+        rx
     };
     let out = leer(
         hijo.stdout
@@ -593,20 +605,33 @@ fn ejecutar_con_limite(mut cmd: Command, limite: Duration) -> std::io::Result<Op
             .take()
             .map(|p| Box::new(p) as Box<dyn Read + Send>),
     );
-    let inicio = Instant::now();
+    let fin = Instant::now() + limite;
     let status = loop {
-        if let Some(s) = hijo.try_wait()? {
-            break Some(s);
+        match hijo.try_wait() {
+            Ok(Some(s)) => break Some(s),
+            Ok(None) => {}
+            Err(e) => {
+                // Sin poder saber si sigue vivo: se mata antes de salir.
+                let _ = hijo.kill();
+                let _ = hijo.wait();
+                return Err(e);
+            }
         }
-        if inicio.elapsed() >= limite {
+        if Instant::now() >= fin {
             let _ = hijo.kill();
             let _ = hijo.wait();
             break None;
         }
         std::thread::sleep(Duration::from_millis(20));
     };
-    let stdout = out.join().unwrap_or_default();
-    let stderr = err.join().unwrap_or_default();
+    // Lo que quede del plazo, con un margen corto para recoger la salida de
+    // un hijo que acabo justo al limite.
+    let restante = || {
+        fin.saturating_duration_since(Instant::now())
+            .max(Duration::from_millis(100))
+    };
+    let stdout = out.recv_timeout(restante()).unwrap_or_default();
+    let stderr = err.recv_timeout(restante()).unwrap_or_default();
     Ok(status.map(|status| Output {
         status,
         stdout,
@@ -630,12 +655,17 @@ fn fallo_de_escritura(ruta: &Path, tipo: std::io::ErrorKind, error: &str) -> Inf
         )
 }
 
-/// Baja el binario a `fichero` y, si la conexion se corta a mitad del cuerpo,
-/// lo vacia y lo intenta una vez mas. Son unos MiB por una red que puede
-/// fallar a medias; y en una maquina de desarrollo con Windows se ha visto
-/// (TASK-027) que la red local corta a veces a los ~19 s las descargas de mas
-/// de ~450 KB. Los demas fallos no se reintentan: no conectar, TLS, un estado
-/// HTTP, el limite o una redireccion se repetirian igual.
+/// Baja el binario a `fichero` y, ante un fallo de transporte transitorio
+/// (no conecta, la conexion se corta antes o despues de las cabeceras,
+/// tiempo agotado), lo vacia y lo intenta una vez mas. Son unos MiB por una
+/// red que puede fallar a medias; y en una maquina de desarrollo con Windows
+/// se ha visto (TASK-027) que la red local corta a veces a los ~19 s las
+/// descargas de mas de ~450 KB. No se reintentan los fallos que se
+/// repetirian igual: 4xx, 5xx, TLS, una redireccion no permitida, el limite
+/// de tamano y una respuesta invalida.
+/// Pausa antes del reintento de la descarga.
+const PAUSA_REINTENTO: Duration = Duration::from_secs(1);
+
 fn bajar_con_un_reintento(
     d: &Descargador,
     url: &str,
@@ -643,7 +673,9 @@ fn bajar_con_un_reintento(
 ) -> Result<u64, Fallo> {
     use std::io::{Seek, SeekFrom};
     match d.a_escritor(url, fichero, descarga::LIMITE_BINARIO) {
-        Err(Fallo::Sinred { corte: true, .. }) => {
+        Err(Fallo::Sinred {
+            transitorio: true, ..
+        }) => {
             let vaciar = fichero
                 .set_len(0)
                 .and_then(|()| fichero.seek(SeekFrom::Start(0)).map(|_| ()));
@@ -653,6 +685,9 @@ fn bajar_con_un_reintento(
                     tipo: e.kind(),
                 });
             }
+            // Una pausa corta antes de repetir, como cualquier reintento:
+            // un corte suele venir de algo que tarda un momento en pasar.
+            std::thread::sleep(PAUSA_REINTENTO);
             d.a_escritor(url, fichero, descarga::LIMITE_BINARIO)
         }
         otro => otro,
@@ -847,6 +882,46 @@ mod tests {
         }
     }
 
+    /// Revision 2, MEN-6: el plazo se cumple aunque el hijo deje un nieto
+    /// con las tuberias heredadas (aqui, un shell que lanza una espera larga
+    /// y no la sustituye con `exec`). Antes, la llamada esperaba al nieto.
+    #[test]
+    fn ejecutar_con_limite_cumple_el_plazo_con_un_nieto() {
+        let mut cmd = if cfg!(windows) {
+            let mut c = Command::new("cmd");
+            c.args(["/c", "ping -n 30 127.0.0.1 >nul & echo fin"]);
+            c
+        } else {
+            let mut c = Command::new("sh");
+            c.args(["-c", "sleep 30; echo fin"]);
+            c
+        };
+        cmd.env("TC2_PRUEBA", "1");
+        let inicio = Instant::now();
+        let r = ejecutar_con_limite(cmd, Duration::from_secs(1));
+        let tardo = inicio.elapsed();
+        assert!(matches!(r, Ok(None)), "{r:?}");
+        assert!(tardo < Duration::from_secs(10), "tardo {tardo:?}");
+        // Un hijo que acaba a tiempo da su salida y su codigo.
+        let mut eco = if cfg!(windows) {
+            let mut c = Command::new("cmd");
+            c.args(["/c", "echo hola"]);
+            c
+        } else {
+            let mut c = Command::new("sh");
+            c.args(["-c", "echo hola"]);
+            c
+        };
+        eco.env("TC2_PRUEBA", "1");
+        let s = ejecutar_con_limite(eco, Duration::from_secs(20));
+        let s = s.ok().flatten();
+        assert_eq!(s.as_ref().map(|o| o.status.success()), Some(true));
+        assert_eq!(
+            s.map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned()),
+            Some("hola".to_owned())
+        );
+    }
+
     /// Revision 1, IMP-1: que `gh` se usa, segun `TC2_GH` y el origen.
     #[test]
     fn gh_para_la_attestation() {
diff --git a/crates/tc2/src/descarga.rs b/crates/tc2/src/descarga.rs
index 93e73ca..f3c10f8 100644
--- a/crates/tc2/src/descarga.rs
+++ b/crates/tc2/src/descarga.rs
@@ -101,9 +101,11 @@ pub enum Fallo {
         url: String,
         /// Error del cliente.
         error: String,
-        /// Si fue un corte a mitad del cuerpo (lo unico que se reintenta):
-        /// un fallo de conexion, de TLS o de protocolo se repetiria igual.
-        corte: bool,
+        /// Si es un fallo de transporte transitorio, antes o despues de las
+        /// cabeceras (no conecta, se corta, tiempo agotado, cuerpo parado):
+        /// lo unico que se reintenta. TLS y una respuesta invalida se
+        /// repetirian igual.
+        transitorio: bool,
     },
     /// 404: no existe esa release o ese fichero.
     NoPublicada {
@@ -263,17 +265,19 @@ impl Descargador {
     ) -> Result<u64, Fallo> {
         let mut actual = url.to_owned();
         for _ in 0..=MAX_SALTOS {
-            let sinred = |e: String, corte: bool| Fallo::Sinred {
+            let sinred = |e: String, transitorio: bool| Fallo::Sinred {
                 url: url.to_owned(),
                 error: e,
-                corte,
+                transitorio,
             };
             let mut r = match self.agente.get(&actual).call() {
                 Ok(r) => r,
                 Err(e) => {
                     return Err(match http::clasificar(e) {
-                        FalloHttp::Sinred(e)
-                        | FalloHttp::Tls(e)
+                        // DNS, conexion, tiempo agotado, E/S o cuerpo
+                        // parado: transitorio.
+                        FalloHttp::Sinred(e) => sinred(e, true),
+                        FalloHttp::Tls(e)
                         | FalloHttp::Otro(e)
                         | FalloHttp::RespuestaInvalida(e) => sinred(e, false),
                         FalloHttp::CuerpoGrande => Fallo::Grande {
````

## Excluido del diff (5 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 39ef7a1862bb2047f844adb168309dc4dd92d19c..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/03-en-revision/TASK-027/revision/informe-revision-2-code-quality-reviewer.md  |  63 ++++++++
 tareas/03-en-revision/TASK-027/revision/informe-revision-2-rust-wasm-reviewer.md     | 137 +++++++++++++++++
 tareas/03-en-revision/TASK-027/revision/peticion-revision-2-code-quality-reviewer.md | 874 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-027/revision/peticion-revision-2-rust-wasm-reviewer.md    | 870 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-027/tarea.md                                              |   9 +-
 5 files changed, 1952 insertions(+), 1 deletion(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff 39ef7a1862bb2047f844adb168309dc4dd92d19c..HEAD -- ":(literal)crates/tc2/tests/actualizar_cli.rs"
````

````
crates/tc2/tests/actualizar_cli.rs | 122 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++----------------
 1 file changed, 106 insertions(+), 16 deletions(-)
````
