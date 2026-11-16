# Peticion de revision — TASK-027 (ronda 2)

- Tarea: TASK-027 — F2-12 taskctl actualizar --cli y aviso de version nueva
- Rama revisada: feature/task-027-f2-12-taskctl-actualizar-cli-y-aviso-de
- Rama base: develop
- Commit revisado (HEAD): 39ef7a1862bb2047f844adb168309dc4dd92d19c
- Fecha: 2026-11-16
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-027 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 991b24cb3fbdf8c89c7441744cea1e1d03cc4f96 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (12 hallazgo(s)))

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
informe de esta ronda (informe-revision-2-code-quality-reviewer.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-027/tarea.md
- tareas/03-en-revision/TASK-027/planificacion/
- crates/distribucion/src/lib.rs
- crates/tc2/examples/gh_falso.rs
- crates/tc2/src/actualizar_cli.rs
- crates/tc2/src/descarga.rs
- crates/tc2/src/i18n/catalogo/actualizar.rs
- crates/tc2/tests/actualizar_cli.rs
- docs/distribucion/instalar.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 6 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | abierto | crates/tc2/src/actualizar_cli.rs:332 | informe-revision-1-code-quality-reviewer.md |

## Commits a revisar (git log 991b24cb3fbdf8c89c7441744cea1e1d03cc4f96..HEAD)

````
39ef7a1 fix(TASK-027): attestation con gh y menores de la revision 1
b21ba2c chore(TASK-027): veredictos y coste de la revision ronda 1
3a37cd4 chore(TASK-027): informes de revision ronda 1
4ecddba chore(TASK-027): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 991b24cb3fbdf8c89c7441744cea1e1d03cc4f96..HEAD)

````diff
diff --git a/crates/distribucion/src/lib.rs b/crates/distribucion/src/lib.rs
index ec3649e..7805a61 100644
--- a/crates/distribucion/src/lib.rs
+++ b/crates/distribucion/src/lib.rs
@@ -15,6 +15,13 @@ pub mod sha256;
 /// mismo.
 pub const PROTOCOLO: u32 = 1;
 
+/// Repositorio de las releases (`gh attestation verify --repo`).
+pub const REPO: &str = "charliebk/TaskCodeTwo";
+
+/// Workflow que firma la attestation de los binarios
+/// (`gh attestation verify --signer-workflow`), como en los instaladores.
+pub const WORKFLOW_RELEASE: &str = "charliebk/TaskCodeTwo/.github/workflows/release.yml";
+
 /// Releases del repositorio: `cli.lock` apunta a `<RELEASES>/download/<tag>/`.
 pub const RELEASES: &str = "https://github.com/charliebk/TaskCodeTwo/releases";
 
diff --git a/crates/tc2/examples/gh_falso.rs b/crates/tc2/examples/gh_falso.rs
new file mode 100644
index 0000000..7e86450
--- /dev/null
+++ b/crates/tc2/examples/gh_falso.rs
@@ -0,0 +1,54 @@
+//! Doble de `gh` para `tests/actualizar_cli.rs` (TASK-027, revision 1,
+//! IMP-1): simula la attestation sin red. `tc2 actualizar --cli` lo usa si
+//! `TC2_GH` apunta a el.
+//!
+//! - `auth status` → 0 (hay sesion).
+//! - `attestation verify <fichero> --repo <r> --signer-workflow <w>` → 0 si
+//!   `GH_FALSO=ok`; si no, 1 con una linea en stderr, como `gh` cuando la
+//!   attestation no vale.
+//! - Con `GH_FALSO_REGISTRO=<fichero>`, anade al fichero una linea con los
+//!   argumentos de cada llamada, separados por tabuladores.
+//!
+//! Solo usa `std`. No es parte de la distribucion.
+
+use std::io::Write;
+use std::process::ExitCode;
+
+fn main() -> ExitCode {
+    let args: Vec<String> = std::env::args().skip(1).collect();
+    if let Ok(registro) = std::env::var("GH_FALSO_REGISTRO") {
+        let linea = format!("{}\n", args.join("\t"));
+        let escrito = std::fs::OpenOptions::new()
+            .create(true)
+            .append(true)
+            .open(registro)
+            .and_then(|mut f| f.write_all(linea.as_bytes()));
+        if escrito.is_err() {
+            return ExitCode::from(2);
+        }
+    }
+    let args: Vec<&str> = args.iter().map(String::as_str).collect();
+    match args.as_slice() {
+        ["auth", "status"] => ExitCode::SUCCESS,
+        [
+            "attestation",
+            "verify",
+            _fichero,
+            "--repo",
+            _,
+            "--signer-workflow",
+            _,
+        ] => {
+            if std::env::var("GH_FALSO").as_deref() == Ok("ok") {
+                ExitCode::SUCCESS
+            } else {
+                let _ = writeln!(
+                    std::io::stderr(),
+                    "X Verification failed: no attestations found (gh falso)"
+                );
+                ExitCode::from(1)
+            }
+        }
+        _ => ExitCode::from(2),
+    }
+}
diff --git a/crates/tc2/src/actualizar_cli.rs b/crates/tc2/src/actualizar_cli.rs
index ad44d70..5b308fb 100644
--- a/crates/tc2/src/actualizar_cli.rs
+++ b/crates/tc2/src/actualizar_cli.rs
@@ -6,8 +6,13 @@
 //!   estable, o la de `--version`): una peticion, la misma fuente que los
 //!   instaladores de F2-11.
 //! - `--comprobar` solo lee: `actualizar ok actual=X disponible=Y` si hay una
-//!   mas nueva, `sin-cambios` si no. Es el aviso de version nueva: lo usan
-//!   `doctor` y un hook opcional, nunca una consulta pasiva.
+//!   mas nueva, `sin-cambios` si no (con `--version` de una anterior, `ok`
+//!   con `cambio=anterior`). Es el aviso de version nueva: es una orden
+//!   explicita, nunca una consulta pasiva; la llamara `doctor` cuando exista
+//!   (F3-8) y, si se quiere, un hook de inicio de sesion.
+//! - Si hay `gh` con sesion, comprueba la attestation de procedencia del
+//!   binario nuevo como los instaladores de F2-11; sin `gh`, sigue con el
+//!   sha256 y lo dice en la linea (`attestation=sin-gh`).
 //! - Un `tc2` de la cache del plugin no se toca (lo gestiona el plugin con su
 //!   `cli.lock`), ni uno de un gestor de paquetes.
 //! - El binario nuevo se baja junto al actual (`.tc2-<v>-<pid>.descarga`),
@@ -18,6 +23,8 @@
 
 use std::cmp::Ordering;
 use std::path::{Path, PathBuf};
+use std::process::{Command, Output, Stdio};
+use std::time::{Duration, Instant};
 
 use distribucion::Target;
 
@@ -59,18 +66,37 @@ pub fn detectar_modo(ruta: &str) -> Modo {
     {
         return Modo::Plugin;
     }
-    for p in &partes {
-        let gestor = match p.as_str() {
-            "node_modules" => "npm",
-            "cellar" | "homebrew" | ".linuxbrew" | "linuxbrew" => "homebrew",
-            "winget" => "winget",
-            "scoop" => "scoop",
-            ".cargo" => "cargo",
-            _ => continue,
-        };
-        return Modo::Gestor(gestor);
-    }
-    Modo::Instalador
+    // Cada gestor por su propia estructura de carpetas, no por un nombre
+    // suelto: una carpeta `Scoop\bin` o `~/.cargo/otra` de la persona no es
+    // de un gestor. `dir` son las carpetas, sin el fichero.
+    let dir = partes.get(..n.saturating_sub(1)).unwrap_or_default();
+    let seguidas = |a: &str, b: &str| dir.windows(2).any(|w| w[0] == a && w[1] == b);
+    // `<raiz>/node_modules/<paquete>/...`: el binario dentro de un paquete.
+    let npm = dir
+        .iter()
+        .position(|p| p == "node_modules")
+        .is_some_and(|i| i + 1 < dir.len());
+    // `Cellar/<formula>/<version>/...`, o el prefijo de Homebrew.
+    let homebrew = dir
+        .iter()
+        .position(|p| p == "cellar")
+        .is_some_and(|i| i + 2 < dir.len())
+        || dir.starts_with(&["opt".to_owned(), "homebrew".to_owned()])
+        || seguidas("linuxbrew", ".linuxbrew");
+    let gestor = if npm {
+        Some("npm")
+    } else if homebrew {
+        Some("homebrew")
+    } else if seguidas("winget", "packages") || seguidas("winget", "links") {
+        Some("winget")
+    } else if seguidas("scoop", "apps") || seguidas("scoop", "shims") {
+        Some("scoop")
+    } else if dir.len() >= 2 && dir[dir.len() - 2] == ".cargo" && dir[dir.len() - 1] == "bin" {
+        Some("cargo")
+    } else {
+        None
+    };
+    gestor.map_or(Modo::Instalador, Modo::Gestor)
 }
 
 /// Compara dos versiones `X.Y.Z[-pre]` con las reglas de semver: el nucleo
@@ -193,7 +219,24 @@ pub fn orden(args: &[String]) -> Informe {
             );
         }
     };
-    actualizar(&exe, &op, &origen, &Descargador::nuevo())
+    let gh = gh_de_variable(std::env::var(VARIABLE_GH).ok().as_deref(), &origen);
+    actualizar(&exe, &op, &origen, &Descargador::nuevo(), gh.as_deref())
+}
+
+/// Variable con el `gh` que se usa para la attestation (por defecto, el del
+/// PATH).
+pub const VARIABLE_GH: &str = "TC2_GH";
+
+/// El `gh` para la attestation: el de `TC2_GH` si esta definida y, si no,
+/// `gh` del PATH. En un ensayo (`TC2_BASE_URL`) los binarios no tienen
+/// attestation, asi que solo se comprueba si `TC2_GH` lo pide, como en los
+/// instaladores, que no la comprueban con `TC2_BASE_URL`.
+pub fn gh_de_variable(valor: Option<&str>, origen: &Origen) -> Option<String> {
+    match (valor.filter(|v| !v.is_empty()), origen) {
+        (Some(v), _) => Some(v.to_owned()),
+        (None, Origen::GitHub) => Some("gh".to_owned()),
+        (None, Origen::Ensayo(_)) => None,
+    }
 }
 
 /// Lo que publica la release para un target.
@@ -234,9 +277,15 @@ fn elegir(pares: &[(String, String)], target: Target, tag: Option<&str>) -> Opti
     }
 }
 
-/// `actualizar --cli` sobre el ejecutable `exe`.
-pub fn actualizar(exe: &Path, op: &Opciones, origen: &Origen, d: &Descargador) -> Informe {
-    limpiar_old(exe);
+/// `actualizar --cli` sobre el ejecutable `exe`, con `gh` para la
+/// attestation (`None`: no se comprueba, en un ensayo).
+pub fn actualizar(
+    exe: &Path,
+    op: &Opciones,
+    origen: &Origen,
+    d: &Descargador,
+    gh: Option<&str>,
+) -> Informe {
     match detectar_modo(&exe.to_string_lossy()) {
         Modo::Instalador => {}
         Modo::Plugin => {
@@ -319,9 +368,15 @@ pub fn actualizar(exe: &Path, op: &Opciones, origen: &Origen, d: &Descargador) -
         return base(Resultado::SinCambios);
     }
     if op.comprobar {
-        return base(Resultado::Ok);
+        // Con --version de una anterior, `ok` no quiere decir que haya una
+        // mas nueva: se dice.
+        return if orden == Some(Ordering::Greater) {
+            base(Resultado::Ok).con("cambio", "anterior")
+        } else {
+            base(Resultado::Ok)
+        };
     }
-    instalar(exe, target, &publicado, origen, d)
+    instalar(exe, target, &publicado, origen, d, gh)
 }
 
 /// Baja el binario junto a `exe`, comprueba su sha256 contra `SHA256SUMS`
@@ -334,7 +389,11 @@ fn instalar(
     publicado: &Publicado,
     origen: &Origen,
     d: &Descargador,
+    gh: Option<&str>,
 ) -> Informe {
+    // Un `.old` de una actualizacion anterior (Windows). Aqui y no antes:
+    // `--comprobar` no toca nada.
+    limpiar_old(exe);
     let Some(dir) = exe.parent() else {
         return sin_permiso(
             exe,
@@ -359,13 +418,13 @@ fn instalar(
     let url = origen.url_binario(&publicado.tag, &publicado.fichero);
     let bajada = bajar_con_un_reintento(d, &url, &mut fichero)
         .map_err(|f| match f {
-            Fallo::Escritura { error } => sin_permiso(&temporal, &error),
+            Fallo::Escritura { error, tipo } => fallo_de_escritura(&temporal, tipo, &error),
             otro => otro.informe(NOMBRE),
         })
         .and_then(|_| {
             fichero
                 .sync_all()
-                .map_err(|e| sin_permiso(&temporal, &e.to_string()))
+                .map_err(|e| fallo_de_escritura(&temporal, e.kind(), &e.to_string()))
         });
     drop(fichero);
     if let Err(i) = bajada {
@@ -373,7 +432,7 @@ fn instalar(
     }
     let obtenido = match std::fs::read(&temporal) {
         Ok(b) => distribucion::sha256::hex(&b),
-        Err(e) => return sin_permiso(&temporal, &e.to_string()),
+        Err(e) => return fallo_de_escritura(&temporal, e.kind(), &e.to_string()),
     };
     if obtenido != publicado.sha256 {
         return Informe::nuevo(NOMBRE, Resultado::Rechazada)
@@ -388,8 +447,22 @@ fn instalar(
                     .con("esperado", publicado.sha256.clone()),
             );
     }
+    // La procedencia, antes de ejecutar nada del binario nuevo.
+    let attestation = match comprobar_attestation(gh, &temporal) {
+        Ok(a) => a,
+        Err(detalle) => {
+            return Informe::nuevo(NOMBRE, Resultado::Rechazada)
+                .con("motivo", "attestation-no-valida")
+                .con("fichero", publicado.fichero.clone())
+                .diagnostico(
+                    Mensaje::nuevo(&txt::ATTESTATION_NO_VALIDA)
+                        .con("fichero", publicado.fichero.clone())
+                        .con("detalle", detalle),
+                );
+        }
+    };
     if let Err(e) = hacer_ejecutable(&temporal) {
-        return sin_permiso(&temporal, &e.to_string());
+        return fallo_de_escritura(&temporal, e.kind(), &e.to_string());
     }
     if let Err(detalle) = comprobar_nuevo(&temporal, &publicado.version, target.alias, &obtenido) {
         return Informe::nuevo(NOMBRE, Resultado::Rechazada)
@@ -419,13 +492,150 @@ fn instalar(
         .con("anterior", crate::VERSION)
         .con("version", publicado.version.clone())
         .con("sha256", obtenido)
+        .con("attestation", attestation.texto())
+}
+
+/// Como acabo la comprobacion de la attestation.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Attestation {
+    /// `gh attestation verify` la dio por buena.
+    Comprobada,
+    /// No hay `gh`, o no tiene sesion: solo el sha256 (como el instalador).
+    SinGh,
+    /// Ensayo con `TC2_BASE_URL`: sus binarios no tienen attestation.
+    Ensayo,
+}
+
+impl Attestation {
+    /// Valor de la clave `attestation=`.
+    pub fn texto(self) -> &'static str {
+        match self {
+            Self::Comprobada => "comprobada",
+            Self::SinGh => "sin-gh",
+            Self::Ensayo => "ensayo",
+        }
+    }
+}
+
+/// Tiempo maximo de cada llamada a `gh`.
+const ESPERA_GH: Duration = Duration::from_secs(60);
+
+/// Comprueba la attestation de `fichero` con `gh` (paridad con los
+/// instaladores de F2-11): `gh auth status` y, si hay sesion,
+/// `gh attestation verify <fichero> --repo <REPO> --signer-workflow
+/// <WORKFLOW_RELEASE>`. Sin `gh` o sin sesion, `SinGh`. `Err` con el detalle
+/// si `gh` la rechaza o no responde.
+pub fn comprobar_attestation(gh: Option<&str>, fichero: &Path) -> Result<Attestation, String> {
+    let Some(gh) = gh else {
+        return Ok(Attestation::Ensayo);
+    };
+    let mut sesion = Command::new(gh);
+    sesion.args(["auth", "status"]);
+    match ejecutar_con_limite(sesion, ESPERA_GH) {
+        Ok(Some(s)) if s.status.success() => {}
+        // Sin gh, gh sin sesion, o un gh que no responde a `auth status`.
+        _ => return Ok(Attestation::SinGh),
+    }
+    let mut verificar = Command::new(gh);
+    verificar
+        .args(["attestation", "verify"])
+        .arg(fichero)
+        .args(["--repo", distribucion::REPO])
+        .args(["--signer-workflow", distribucion::WORKFLOW_RELEASE]);
+    match ejecutar_con_limite(verificar, ESPERA_GH) {
+        Ok(Some(s)) if s.status.success() => Ok(Attestation::Comprobada),
+        Ok(Some(s)) => Err(ultima_linea(&s.stderr)
+            .unwrap_or_else(|| format!("gh attestation verify salio con {}", s.status))),
+        Ok(None) => Err(format!(
+            "gh attestation verify no respondio en {} s",
+            ESPERA_GH.as_secs()
+        )),
+        Err(e) => Err(e.to_string()),
+    }
 }
 
-/// Baja el binario a `fichero` y, si la conexion se corta (`sinred`), lo
-/// vacia y lo intenta una vez mas. Son unos MiB por una red que puede fallar
-/// a medias; y en Windows se ha visto (TASK-027) que la inspeccion de red de
-/// un antivirus corta a veces la primera descarga de un ejecutable a los
-/// ~19 s. Los demas fallos (404, hash, limite) no se reintentan.
+/// La ultima linea no vacia de una salida, para el diagnostico.
+fn ultima_linea(bytes: &[u8]) -> Option<String> {
+    String::from_utf8_lossy(bytes)
+        .lines()
+        .rev()
+        .map(str::trim)
+        .find(|l| !l.is_empty())
+        .map(|l| l.chars().take(300).collect())
+}
+
+/// Lanza `cmd` sin entrada y espera como mucho `limite`. `Ok(None)` si no
+/// termino a tiempo (se mata). stdout y stderr se leen en hilos aparte, hasta
+/// 1 MiB cada uno, para que un hijo que escribe mucho no se bloquee.
+fn ejecutar_con_limite(mut cmd: Command, limite: Duration) -> std::io::Result<Option<Output>> {
+    use std::io::Read;
+    let mut hijo = cmd
+        .stdin(Stdio::null())
+        .stdout(Stdio::piped())
+        .stderr(Stdio::piped())
+        .spawn()?;
+    let leer = |p: Option<Box<dyn Read + Send>>| {
+        std::thread::spawn(move || {
+            let mut v = Vec::new();
+            if let Some(p) = p {
+                let _ = p.take(1024 * 1024).read_to_end(&mut v);
+            }
+            v
+        })
+    };
+    let out = leer(
+        hijo.stdout
+            .take()
+            .map(|p| Box::new(p) as Box<dyn Read + Send>),
+    );
+    let err = leer(
+        hijo.stderr
+            .take()
+            .map(|p| Box::new(p) as Box<dyn Read + Send>),
+    );
+    let inicio = Instant::now();
+    let status = loop {
+        if let Some(s) = hijo.try_wait()? {
+            break Some(s);
+        }
+        if inicio.elapsed() >= limite {
+            let _ = hijo.kill();
+            let _ = hijo.wait();
+            break None;
+        }
+        std::thread::sleep(Duration::from_millis(20));
+    };
+    let stdout = out.join().unwrap_or_default();
+    let stderr = err.join().unwrap_or_default();
+    Ok(status.map(|status| Output {
+        status,
+        stdout,
+        stderr,
+    }))
+}
+
+/// Un fallo de E/S sobre `ruta`: `sin-permiso` si es un permiso denegado y,
+/// si no (disco lleno, por ejemplo), `error motivo=escritura`.
+fn fallo_de_escritura(ruta: &Path, tipo: std::io::ErrorKind, error: &str) -> Informe {
+    if tipo == std::io::ErrorKind::PermissionDenied {
+        return sin_permiso(ruta, error);
+    }
+    Informe::nuevo(NOMBRE, Resultado::Error)
+        .con("motivo", "escritura")
+        .con("ruta", ruta.display().to_string())
+        .diagnostico(
+            Mensaje::nuevo(&txt::ESCRITURA)
+                .con("ruta", ruta.display().to_string())
+                .con("error", error),
+        )
+}
+
+/// Baja el binario a `fichero` y, si la conexion se corta a mitad del cuerpo,
+/// lo vacia y lo intenta una vez mas. Son unos MiB por una red que puede
+/// fallar a medias; y en una maquina de desarrollo con Windows se ha visto
+/// (TASK-027) que la red local corta a veces a los ~19 s las descargas de mas
+/// de ~450 KB. Los demas fallos no se reintentan: no conectar, TLS, un estado
+/// HTTP, el limite o una redireccion se repetirian igual.
 fn bajar_con_un_reintento(
     d: &Descargador,
     url: &str,
@@ -433,13 +643,14 @@ fn bajar_con_un_reintento(
 ) -> Result<u64, Fallo> {
     use std::io::{Seek, SeekFrom};
     match d.a_escritor(url, fichero, descarga::LIMITE_BINARIO) {
-        Err(Fallo::Sinred { .. }) => {
+        Err(Fallo::Sinred { corte: true, .. }) => {
             let vaciar = fichero
                 .set_len(0)
                 .and_then(|()| fichero.seek(SeekFrom::Start(0)).map(|_| ()));
             if let Err(e) = vaciar {
                 return Err(Fallo::Escritura {
                     error: e.to_string(),
+                    tipo: e.kind(),
                 });
             }
             d.a_escritor(url, fichero, descarga::LIMITE_BINARIO)
@@ -488,14 +699,23 @@ fn hacer_ejecutable(_ruta: &Path) -> std::io::Result<()> {
     Ok(())
 }
 
-/// Lanza `nuevo version --json` y exige su version, su target y el sha256
-/// que ya calculo este proceso. `Err` con el detalle si no.
+/// Tiempo maximo del `version --json` del binario nuevo.
+const ESPERA_NUEVO: Duration = Duration::from_secs(30);
+
+/// Lanza `nuevo version --json` (como mucho [`ESPERA_NUEVO`]) y exige su
+/// version, su target y el sha256 que ya calculo este proceso. `Err` con el
+/// detalle si no.
 fn comprobar_nuevo(nuevo: &Path, version: &str, alias: &str, sha256: &str) -> Result<(), String> {
-    let salida = std::process::Command::new(nuevo)
-        .args(["version", "--json"])
-        .stdin(std::process::Stdio::null())
-        .output()
-        .map_err(|e| e.to_string())?;
+    let mut cmd = Command::new(nuevo);
+    cmd.args(["version", "--json"]);
+    let salida = ejecutar_con_limite(cmd, ESPERA_NUEVO)
+        .map_err(|e| e.to_string())?
+        .ok_or_else(|| {
+            format!(
+                "version --json no respondio en {} s",
+                ESPERA_NUEVO.as_secs()
+            )
+        })?;
     if !salida.status.success() {
         return Err(format!("version --json salio con {}", salida.status));
     }
@@ -607,9 +827,47 @@ mod tests {
             ),
             (r"C:\Users\ana\scoop\apps\tc2\current\tc2.exe", "scoop"),
             ("/home/ana/.cargo/bin/tc2", "cargo"),
+            ("/home/linuxbrew/.linuxbrew/bin/tc2", "homebrew"),
+            (r"C:\Users\ana\scoop\shims\tc2.exe", "scoop"),
         ] {
             assert_eq!(detectar_modo(ruta), Modo::Gestor(gestor), "{ruta}");
         }
+        // Revision 1, MEN-1: un nombre de gestor suelto en la ruta no basta.
+        for propia in [
+            r"C:\Users\ana\Scoop\bin\tc2.exe",
+            r"C:\herramientas\scoop\tc2.exe",
+            "/home/ana/.cargo/otra/tc2",
+            "/home/ana/.cargo/bin/sub/tc2",
+            "/home/ana/node_modules/tc2",
+            "/home/ana/cellar/tc2",
+            "/home/ana/homebrew/tc2",
+            r"C:\Users\ana\winget\tc2.exe",
+        ] {
+            assert_eq!(detectar_modo(propia), Modo::Instalador, "{propia}");
+        }
+    }
+
+    /// Revision 1, IMP-1: que `gh` se usa, segun `TC2_GH` y el origen.
+    #[test]
+    fn gh_para_la_attestation() {
+        let ensayo = Origen::Ensayo("http://127.0.0.1:1".into());
+        assert_eq!(gh_de_variable(None, &Origen::GitHub).as_deref(), Some("gh"));
+        assert_eq!(
+            gh_de_variable(Some(""), &Origen::GitHub).as_deref(),
+            Some("gh")
+        );
+        assert_eq!(gh_de_variable(None, &ensayo), None);
+        assert_eq!(
+            gh_de_variable(Some("/x/gh"), &ensayo).as_deref(),
+            Some("/x/gh")
+        );
+        // Sin gh: solo el sha256; en ensayo: no se comprueba.
+        let f = Path::new("no-importa");
+        assert_eq!(comprobar_attestation(None, f), Ok(Attestation::Ensayo));
+        assert_eq!(
+            comprobar_attestation(Some("tc2-gh-que-no-existe-en-ningun-path"), f),
+            Ok(Attestation::SinGh)
+        );
     }
 
     /// TASK-027, criterio 3: orden de versiones con prereleases.
diff --git a/crates/tc2/src/descarga.rs b/crates/tc2/src/descarga.rs
index 79a8605..93e73ca 100644
--- a/crates/tc2/src/descarga.rs
+++ b/crates/tc2/src/descarga.rs
@@ -101,6 +101,9 @@ pub enum Fallo {
         url: String,
         /// Error del cliente.
         error: String,
+        /// Si fue un corte a mitad del cuerpo (lo unico que se reintenta):
+        /// un fallo de conexion, de TLS o de protocolo se repetiria igual.
+        corte: bool,
     },
     /// 404: no existe esa release o ese fichero.
     NoPublicada {
@@ -130,10 +133,19 @@ pub enum Fallo {
         /// Limite en bytes.
         limite: u64,
     },
+    /// La respuesta no es texto UTF-8 (un `SHA256SUMS` roto).
+    NoEsTexto {
+        /// URL pedida.
+        url: String,
+        /// Error de la decodificacion.
+        error: String,
+    },
     /// No se pudo escribir el fichero de destino.
     Escritura {
         /// Error del sistema.
         error: String,
+        /// Su tipo: distingue un permiso denegado de un disco lleno.
+        tipo: std::io::ErrorKind,
     },
 }
 
@@ -141,7 +153,7 @@ impl Fallo {
     /// El informe de `comando` para este fallo.
     pub fn informe(&self, comando: &str) -> Informe {
         match self {
-            Self::Sinred { url, error } => Informe::nuevo(comando, Resultado::Sinred)
+            Self::Sinred { url, error, .. } => Informe::nuevo(comando, Resultado::Sinred)
                 .con("url", url.clone())
                 .diagnostico(
                     Mensaje::nuevo(&txt::SIN_RED)
@@ -181,13 +193,17 @@ impl Fallo {
                         .con("url", url.clone())
                         .con("limite", limite.to_string()),
                 ),
-            Self::Escritura { error } => Informe::nuevo(comando, Resultado::Error)
-                .con("motivo", "escritura")
+            Self::NoEsTexto { url, error } => Informe::nuevo(comando, Resultado::Rechazada)
+                .con("motivo", "sumas-invalidas")
+                .con("url", url.clone())
                 .diagnostico(
-                    Mensaje::nuevo(&txt::SIN_PERMISO)
-                        .con("ruta", "-")
+                    Mensaje::nuevo(&txt::SUMAS_INVALIDAS)
+                        .con("url", url.clone())
                         .con("error", error.clone()),
                 ),
+            Self::Escritura { error, .. } => Informe::nuevo(comando, Resultado::Error)
+                .con("motivo", "escritura")
+                .diagnostico(Mensaje::nuevo(&txt::ESCRITURA_DESCARGA).con("error", error.clone())),
         }
     }
 }
@@ -231,9 +247,9 @@ impl Descargador {
     pub fn texto(&self, url: &str, limite: u64) -> Result<String, Fallo> {
         let mut v = Vec::new();
         self.a_escritor(url, &mut v, limite)?;
-        String::from_utf8(v).map_err(|_| Fallo::Grande {
+        String::from_utf8(v).map_err(|e| Fallo::NoEsTexto {
             url: url.to_owned(),
-            limite,
+            error: e.to_string(),
         })
     }
 
@@ -247,16 +263,19 @@ impl Descargador {
     ) -> Result<u64, Fallo> {
         let mut actual = url.to_owned();
         for _ in 0..=MAX_SALTOS {
-            let sinred = |e: String| Fallo::Sinred {
+            let sinred = |e: String, corte: bool| Fallo::Sinred {
                 url: url.to_owned(),
                 error: e,
+                corte,
             };
             let mut r = match self.agente.get(&actual).call() {
                 Ok(r) => r,
                 Err(e) => {
                     return Err(match http::clasificar(e) {
-                        FalloHttp::Sinred(e) | FalloHttp::Tls(e) | FalloHttp::Otro(e) => sinred(e),
-                        FalloHttp::RespuestaInvalida(e) => sinred(e),
+                        FalloHttp::Sinred(e)
+                        | FalloHttp::Tls(e)
+                        | FalloHttp::Otro(e)
+                        | FalloHttp::RespuestaInvalida(e) => sinred(e, false),
                         FalloHttp::CuerpoGrande => Fallo::Grande {
                             url: url.to_owned(),
                             limite,
@@ -317,7 +336,7 @@ impl Descargador {
                     Ok(0) => break,
                     Ok(n) => n,
                     Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
-                    Err(e) => return Err(sinred(e.to_string())),
+                    Err(e) => return Err(sinred(e.to_string(), true)),
                 };
                 total = total.saturating_add(n as u64);
                 if total > limite {
@@ -327,6 +346,7 @@ impl Descargador {
                     .write_all(buf.get(..n).unwrap_or_default())
                     .map_err(|e| Fallo::Escritura {
                         error: e.to_string(),
+                        tipo: e.kind(),
                     })?;
             }
             return Ok(total);
diff --git a/crates/tc2/src/i18n/catalogo/actualizar.rs b/crates/tc2/src/i18n/catalogo/actualizar.rs
index 668757f..02726e9 100644
--- a/crates/tc2/src/i18n/catalogo/actualizar.rs
+++ b/crates/tc2/src/i18n/catalogo/actualizar.rs
@@ -31,7 +31,7 @@ crate::i18n::textos! {
         en: "this tc2 belongs to the plugin ({ruta}) and the plugin updates it: motivo=gestionado-por-plugin arreglo=\"update the taskcodetwo plugin with /plugin in Claude Code\"",
     }
     GESTIONADO_POR_GESTOR {
-        es: "este tc2 lo instalo {gestor} ({ruta}): actualizalo con {gestor}, no con tc2 actualizar --cli",
+        es: "este tc2 lo ha instalado {gestor} ({ruta}): actualizalo con {gestor}, no con tc2 actualizar --cli",
         en: "this tc2 was installed by {gestor} ({ruta}): update it with {gestor}, not with tc2 actualizar --cli",
     }
     BASE_URL_INVALIDA {
@@ -86,4 +86,16 @@ crate::i18n::textos! {
         es: "no se pudo sustituir {ruta}: {error}. Queda el binario anterior; cierra los tc2 abiertos y repite",
         en: "could not replace {ruta}: {error}. The previous binary is kept; close any running tc2 and retry",
     }
+    ESCRITURA {
+        es: "no se pudo escribir {ruta}: {error}. Comprueba el espacio libre en disco y repite; no se ha cambiado nada",
+        en: "could not write {ruta}: {error}. Check the free disk space and retry; nothing was changed",
+    }
+    ESCRITURA_DESCARGA {
+        es: "no se pudo escribir la descarga: {error}. Comprueba el espacio libre en disco y repite",
+        en: "could not write the download: {error}. Check the free disk space and retry",
+    }
+    ATTESTATION_NO_VALIDA {
+        es: "la attestation de procedencia de {fichero} no es valida ({detalle}): no se ha cambiado nada. Avisa en https://github.com/charliebk/TaskCodeTwo/issues",
+        en: "the provenance attestation of {fichero} is not valid ({detalle}): nothing was changed. Report it at https://github.com/charliebk/TaskCodeTwo/issues",
+    }
 }
diff --git a/docs/distribucion/instalar.md b/docs/distribucion/instalar.md
index 11c52b0..e9b2844 100644
--- a/docs/distribucion/instalar.md
+++ b/docs/distribucion/instalar.md
@@ -88,21 +88,30 @@ tc2 actualizar --cli --version 2.1.0-rc.1   # una versión concreta (también ha
   `--version`. Una sola petición, la misma fuente que los instaladores.
 - **`--comprobar`** escribe `actualizar ok actual=X disponible=Y` si hay una
   más nueva y `actualizar sin-cambios actual=X disponible=Y` si no; código 0
-  en los dos casos. Es el aviso de versión nueva: lo llaman `doctor` y, si se
-  quiere, un hook de inicio de sesión. Nunca se consulta la red por sí solo.
+  en los dos casos. Con `--version` de una anterior a la instalada añade
+  `cambio=anterior`, para que no se lea como una versión nueva. Es el aviso
+  de versión nueva y es una orden explícita: nunca se consulta la red por sí
+  solo. La llamará `doctor` cuando exista (F3-8) y, si se quiere, un hook de
+  inicio de sesión.
 - **Cómo sustituye**: baja el binario junto al actual
   (`.tc2-<versión>-<pid>.descarga`), calcula su sha256 (lo calcula el `tc2`
   viejo, nunca el binario nuevo sobre sí mismo) y lo compara con
-  `SHA256SUMS`; le pide `version --json` y comprueba versión, sistema y hash;
-  y solo entonces lo pone en su sitio. En Windows lo hace por renombrados
+  `SHA256SUMS`; si hay `gh` con sesión, comprueba la attestation de
+  procedencia (abajo); le pide `version --json` (como mucho 30 s) y comprueba
+  versión, sistema y hash; y solo entonces lo pone en su sitio. En Windows lo hace por renombrados
   (el actual pasa a `tc2.exe.old`, el nuevo a `tc2.exe`) con vuelta atrás si
   el segundo falla; el `tc2.exe.old` lo borra la siguiente ejecución de `tc2`.
-  Sale `actualizar ok anterior=X version=Y sha256=...`.
+  Sale `actualizar ok anterior=X version=Y sha256=... attestation=...`.
+  Si la descarga del binario se corta a mitad, se repite una vez; los demás
+  fallos de red no se reintentan.
 - **Si algo falla, no se cambia nada** y no quedan ficheros a medias:
   - sin red: `actualizar sinred url=...` (código 4);
   - el sha256 no casa: `rechazada motivo=sha256-no-casa` (3);
   - sin permiso de escritura en la carpeta del binario: `rechazada
     motivo=sin-permiso ruta=...` (3), antes de bajar nada;
+  - otro fallo al escribir la descarga (disco lleno): `error
+    motivo=escritura ruta=...` (1);
+  - `gh` rechaza la attestation: `rechazada motivo=attestation-no-valida` (3);
   - el binario nuevo no arranca o no dice lo esperado: `rechazada
     motivo=binario-no-arranca` (3);
   - no se pudo renombrar (otro `tc2` abierto, un antivirus):
@@ -115,10 +124,18 @@ tc2 actualizar --cli --version 2.1.0-rc.1   # una versión concreta (también ha
 - Sin `--cli`, `tc2 actualizar` da `uso`: actualizar un proyecto es de F9-2.
 - `TC2_BASE_URL` vale igual que en los instaladores (solo ensayos): la carpeta
   con los ficheros de una release; aquí, https o http a `127.0.0.1`.
-- Como en los instaladores, el hash prueba que el fichero llegó entero, no
-  quién lo construyó: sin Authenticode (D4a), comprometer la release
-  compromete binario y `SHA256SUMS` a la vez. La attestation se comprueba con
-  `gh attestation verify` (ver [releases.md](releases.md)).
+- **Attestation**, como en los instaladores: el hash solo prueba que el
+  fichero llegó entero. Si hay `gh` con sesión, `tc2 actualizar --cli` ejecuta
+  `gh attestation verify <descarga> --repo charliebk/TaskCodeTwo
+  --signer-workflow charliebk/TaskCodeTwo/.github/workflows/release.yml`
+  antes de ejecutar nada del binario nuevo, y si no la da por buena no
+  cambia nada. La línea dice cómo fue: `attestation=comprobada`,
+  `attestation=sin-gh` (no hay `gh` o no tiene sesión: solo el sha256, como
+  el instalador) o `attestation=ensayo` (con `TC2_BASE_URL`, cuyos binarios
+  no tienen attestation). `TC2_GH` fija el `gh` que se usa; en un ensayo, solo
+  se comprueba si `TC2_GH` está definida. Sin `gh` ni Authenticode (D4a),
+  comprometer la release compromete binario y `SHA256SUMS` a la vez (ver
+  [releases.md](releases.md)).
 
 ## Compatibilidad con el plugin
 
@@ -178,7 +195,9 @@ hay `gh`.
 - `cargo test -p tc2` (`crates/tc2/tests/actualizar_cli.rs`): una copia del
   `tc2` compilado se actualiza a un doble de la versión 9.9.9
   (`examples/tc2_falso.rs`) servido en `127.0.0.1`; el hash alterado, sin red,
-  `--comprobar` y la caché del plugin. El fallo del segundo renombrado se
+  `--comprobar` y la caché del plugin; la attestation con un `gh` simulado
+  (`examples/gh_falso.rs`: ausente, válida y rechazada), y en Windows que la
+  ejecución siguiente borra el `.old`. El fallo del segundo renombrado se
   prueba con un fallo inyectado.
 - el job `actualizar-ensayo` de `release.yml`
   (`.github/scripts/probar-actualizar.sh`): en los 6 runners nativos, instala
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 991b24cb3fbdf8c89c7441744cea1e1d03cc4f96..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-027/planificacion/brainstorm/peticion-plan-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-027/planificacion/plan-final.md                 |    0
 tareas/03-en-revision/TASK-027/revision/informe-revision-1-code-quality-reviewer.md         |   83 +++
 tareas/03-en-revision/TASK-027/revision/informe-revision-1-rust-wasm-reviewer.md            |  111 ++++
 tareas/03-en-revision/TASK-027/revision/peticion-revision-1-code-quality-reviewer.md        |  425 +++++++++++++++
 tareas/03-en-revision/TASK-027/revision/peticion-revision-1-rust-wasm-reviewer.md           | 2859 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-027/tarea.md                                    |   16 +-
 7 files changed, 3491 insertions(+), 3 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff 991b24cb3fbdf8c89c7441744cea1e1d03cc4f96..HEAD -- ":(literal)crates/tc2/tests/actualizar_cli.rs"
````

````
crates/tc2/tests/actualizar_cli.rs | 199 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-----------
 1 file changed, 185 insertions(+), 14 deletions(-)
````
