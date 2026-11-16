# Peticion de revision — TASK-025 (ronda 1)

- Tarea: TASK-025 — F2-10 Build de ejecutables por sistema en la CI, firmados
- Rama revisada: feature/task-025-f2-10-build-de-ejecutables-por-sistema-e
- Rama base: develop
- Commit revisado (HEAD): e531238f359fa059a30904b61c3017ec947a5318
- Fecha: 2026-11-16
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-025 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-025/tarea.md
- tareas/03-en-revision/TASK-025/planificacion/
- crates/tc2/tests/git_binario.rs
- xtask/src/dist.rs
- xtask/src/main.rs

## Commits a revisar (git log develop..HEAD)

````
e531238 chore(TASK-025): autorrevision ronda 1
7527334 fix(TASK-025): CARGO_TARGET_DIR relativo se resuelve desde la raiz del workspace
5e5348e fix(TASK-025): git_binario no hereda el idioma del runner (CI de macOS en rojo desde TASK-024)
d4889a3 feat(TASK-025): release con build por sistema, humo sin Node, hash y attestation
77b036f chore(TASK-025): tarea en curso
````

## Diff de tu dominio (3 fichero(s) de 7; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/xtask/src/dist.rs b/xtask/src/dist.rs
new file mode 100644
index 0000000..68bf587
--- /dev/null
+++ b/xtask/src/dist.rs
@@ -0,0 +1,210 @@
+//! Distribucion (F2-10, ADR F1-14): los 6 targets de P12, el nombre de cada
+//! binario publicado, el formato de `SHA256SUMS` y la regla tag↔version.
+//!
+//! Es logica pura, sin E/S, para probarla en local: el empaquetado y la
+//! release de `.github/workflows/release.yml` llaman a `cargo xtask`, que
+//! usa este modulo.
+
+/// Un target de P12: el alias publico (contrato de `cli.lock`) y el triple de
+/// Rust con el que se compila.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub struct Target {
+    pub alias: &'static str,
+    pub triple: &'static str,
+}
+
+/// Los 6 targets de P12 (ADR F1-14, punto 3). En Linux, musl estatico
+/// (decision de la persona en TASK-025): el binario no depende de la glibc
+/// del sistema.
+pub const TARGETS: [Target; 6] = [
+    Target {
+        alias: "win-x64",
+        triple: "x86_64-pc-windows-msvc",
+    },
+    Target {
+        alias: "win-arm64",
+        triple: "aarch64-pc-windows-msvc",
+    },
+    Target {
+        alias: "darwin-x64",
+        triple: "x86_64-apple-darwin",
+    },
+    Target {
+        alias: "darwin-arm64",
+        triple: "aarch64-apple-darwin",
+    },
+    Target {
+        alias: "linux-x64",
+        triple: "x86_64-unknown-linux-musl",
+    },
+    Target {
+        alias: "linux-arm64",
+        triple: "aarch64-unknown-linux-musl",
+    },
+];
+
+/// Nombre del fichero de suma de un solo binario: `<fichero>.sha256`.
+pub const SUFIJO_SUMA: &str = ".sha256";
+/// Fichero con las sumas de todos los binarios de la release.
+pub const SUMAS: &str = "SHA256SUMS";
+
+impl Target {
+    pub fn de_triple(triple: &str) -> Option<Target> {
+        TARGETS.iter().copied().find(|t| t.triple == triple)
+    }
+
+    pub fn es_windows(self) -> bool {
+        self.alias.starts_with("win-")
+    }
+
+    pub fn es_macos(self) -> bool {
+        self.alias.starts_with("darwin-")
+    }
+
+    /// Binario que deja `cargo build --target`.
+    pub fn binario_cargo(self) -> &'static str {
+        if self.es_windows() { "tc2.exe" } else { "tc2" }
+    }
+
+    /// Nombre publicado (ADR F1-14, punto 4): `tc2-<version>-<alias>[.exe]`.
+    pub fn fichero(self, version: &str) -> String {
+        let ext = if self.es_windows() { ".exe" } else { "" };
+        format!("tc2-{version}-{}{ext}", self.alias)
+    }
+}
+
+/// Una linea con el formato de `sha256sum` (modo texto): `<hex>  <fichero>`.
+pub fn linea_suma(hash: &str, fichero: &str) -> String {
+    format!("{hash}  {fichero}\n")
+}
+
+/// `SHA256SUMS` a partir de pares (fichero, hash), ordenado por fichero para
+/// que el resultado no dependa del orden en que se listan los ficheros. Falla
+/// si un hash no son 64 hex en minusculas, o si un fichero se repite o trae
+/// un separador o un salto de linea (romperia el formato).
+pub fn sumas(entradas: &[(String, String)]) -> Result<String, String> {
+    let mut ordenadas: Vec<&(String, String)> = entradas.iter().collect();
+    ordenadas.sort_by(|a, b| a.0.cmp(&b.0));
+    let mut texto = String::new();
+    let mut anterior: Option<&str> = None;
+    for (fichero, hash) in ordenadas {
+        if hash.len() != 64 || !hash.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
+            return Err(format!("hash no valido para {fichero}: «{hash}»"));
+        }
+        if fichero.is_empty() || fichero.contains(['/', '\\', '\n', '\r']) {
+            return Err(format!("nombre de fichero no valido: «{fichero}»"));
+        }
+        if anterior == Some(fichero.as_str()) {
+            return Err(format!("fichero repetido: {fichero}"));
+        }
+        anterior = Some(fichero);
+        texto.push_str(&linea_suma(hash, fichero));
+    }
+    if texto.is_empty() {
+        return Err("no hay ficheros que sumar".to_owned());
+    }
+    Ok(texto)
+}
+
+/// Clase de la release segun su tag.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Clase {
+    Estable,
+    Prerelease,
+}
+
+/// El tag tiene que ser `v<version>` (estable) o `v<version>-<sufijo>`
+/// (prerelease), con `<version>` la del workspace: asi el `tc2 --version` del
+/// binario publicado coincide con el tag.
+pub fn clase_de_tag(tag: &str, version: &str) -> Result<Clase, String> {
+    let resto = tag
+        .strip_prefix('v')
+        .and_then(|r| r.strip_prefix(version))
+        .ok_or_else(|| format!("el tag «{tag}» no es v{version} ni v{version}-<sufijo>"))?;
+    if resto.is_empty() {
+        return Ok(Clase::Estable);
+    }
+    match resto.strip_prefix('-') {
+        Some(sufijo)
+            if !sufijo.is_empty()
+                && sufijo
+                    .bytes()
+                    .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-') =>
+        {
+            Ok(Clase::Prerelease)
+        }
+        _ => Err(format!(
+            "el tag «{tag}» no es v{version} ni v{version}-<sufijo>"
+        )),
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn seis_targets_de_p12_con_exe_solo_en_windows() {
+        let alias: Vec<&str> = TARGETS.iter().map(|t| t.alias).collect();
+        assert_eq!(
+            alias,
+            [
+                "win-x64",
+                "win-arm64",
+                "darwin-x64",
+                "darwin-arm64",
+                "linux-x64",
+                "linux-arm64"
+            ]
+        );
+        for t in TARGETS {
+            assert_eq!(Target::de_triple(t.triple), Some(t));
+            let f = t.fichero("2.0.0");
+            assert!(f.starts_with(&format!("tc2-2.0.0-{}", t.alias)));
+            assert_eq!(f.ends_with(".exe"), t.es_windows(), "{f}");
+            assert_eq!(t.binario_cargo().ends_with(".exe"), t.es_windows());
+        }
+        assert_eq!(TARGETS[0].fichero("2.0.0"), "tc2-2.0.0-win-x64.exe");
+        assert_eq!(TARGETS[4].fichero("2.0.0"), "tc2-2.0.0-linux-x64");
+        assert!(TARGETS[4].triple.ends_with("-musl"));
+        assert!(TARGETS[5].triple.ends_with("-musl"));
+        assert_eq!(Target::de_triple("x86_64-unknown-linux-gnu"), None);
+    }
+
+    #[test]
+    fn sumas_con_formato_de_sha256sum_y_orden_estable() {
+        let vacio = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
+        let abc = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
+        let a = (String::from("tc2-0.0.0-win-x64.exe"), abc.to_owned());
+        let b = (String::from("tc2-0.0.0-linux-x64"), vacio.to_owned());
+        let esperado = format!("{vacio}  tc2-0.0.0-linux-x64\n{abc}  tc2-0.0.0-win-x64.exe\n");
+        assert_eq!(sumas(&[a.clone(), b.clone()]), Ok(esperado.clone()));
+        assert_eq!(sumas(&[b.clone(), a.clone()]), Ok(esperado));
+        assert!(sumas(&[a.clone(), a.clone()]).is_err());
+        assert!(sumas(&[]).is_err());
+        assert!(sumas(&[(String::from("x"), abc.to_uppercase())]).is_err());
+        assert!(sumas(&[(String::from("x"), String::from("ab"))]).is_err());
+        assert!(sumas(&[(String::from("dir/x"), abc.to_owned())]).is_err());
+        assert!(sumas(&[(String::from("x\ny"), abc.to_owned())]).is_err());
+    }
+
+    #[test]
+    fn tag_y_version_tienen_que_casar() {
+        assert_eq!(clase_de_tag("v0.0.0", "0.0.0"), Ok(Clase::Estable));
+        assert_eq!(
+            clase_de_tag("v0.0.0-prueba.1", "0.0.0"),
+            Ok(Clase::Prerelease)
+        );
+        for malo in [
+            "v0.0.1",
+            "0.0.0",
+            "v0.0.0-",
+            "v0.0.00",
+            "v0.0.0.1",
+            "v0.0.0-a b",
+            "v0.0.0-a/b",
+        ] {
+            assert!(clase_de_tag(malo, "0.0.0").is_err(), "{malo}");
+        }
+    }
+}
diff --git a/xtask/src/main.rs b/xtask/src/main.rs
index c654584..1bb140d 100644
--- a/xtask/src/main.rs
+++ b/xtask/src/main.rs
@@ -7,34 +7,77 @@
 //!   arbol a dos rutas distintas, compila `tc2` en release en limpio en cada
 //!   una y exige el mismo sha256 (ADR F1-14). La CI solo la corre en Windows.
 //!
+//! Distribucion (F2-10), las mismas ordenes en local y en `release.yml`:
+//!
+//! - `empaquetar --target <triple> --salida <dir>`: compila `tc2` en release
+//!   para un target de P12, lo copia como `tc2-<version>-<alias>[.exe]`, lo
+//!   firma ad hoc en macOS y deja al lado su `<fichero>.sha256`.
+//! - `sumas <dir>`: escribe `<dir>/SHA256SUMS` con los binarios de `<dir>`.
+//! - `comprobar-tag <tag>`: escribe `estable` o `prerelease`, o falla si el
+//!   tag no casa con la version del workspace.
+//! - `version`: escribe la version del workspace.
+//!
 //! El resultado va en el codigo de salida: 0 verde, 1 rojo, 2 mal invocado.
 
+mod dist;
 mod sha256;
 
 use std::path::{Path, PathBuf};
 use std::process::{Command, ExitCode};
 
+use dist::{Clase, Target};
+
+/// Version del workspace (`version.workspace = true`): la misma que `tc2`.
+const VERSION: &str = env!("CARGO_PKG_VERSION");
+
+const USO: &str = "Uso: cargo xtask verificar --perfil rapida|completa
+     cargo xtask empaquetar --target <triple> --salida <dir>
+     cargo xtask sumas <dir>
+     cargo xtask comprobar-tag <tag>
+     cargo xtask version";
+
 #[derive(Debug, Clone, Copy, PartialEq, Eq)]
 enum Perfil {
     Rapida,
     Completa,
 }
 
+#[derive(Debug, Clone, PartialEq, Eq)]
+enum Orden {
+    Verificar(Perfil),
+    Empaquetar { target: Target, salida: PathBuf },
+    Sumas(PathBuf),
+    ComprobarTag(String),
+    Version,
+}
+
 fn main() -> ExitCode {
     let args: Vec<String> = std::env::args().skip(1).collect();
-    let perfil = match leer_perfil(&args) {
-        Ok(p) => p,
+    let orden = match leer_orden(&args) {
+        Ok(o) => o,
         Err(e) => {
             eprintln!("xtask: {e}");
-            eprintln!("Uso: cargo xtask verificar --perfil rapida|completa");
+            eprintln!("{USO}");
             return ExitCode::from(2);
         }
     };
-    match verificar(perfil) {
-        Ok(()) => {
-            println!("xtask: verificacion {perfil:?} en verde");
-            ExitCode::SUCCESS
+    let resultado = match orden {
+        Orden::Verificar(perfil) => {
+            verificar(perfil).map(|()| println!("xtask: verificacion {perfil:?} en verde"))
+        }
+        Orden::Empaquetar { target, salida } => empaquetar(target, &salida),
+        Orden::Sumas(dir) => escribir_sumas(&dir),
+        Orden::ComprobarTag(tag) => dist::clase_de_tag(&tag, VERSION).map(|c| match c {
+            Clase::Estable => println!("estable"),
+            Clase::Prerelease => println!("prerelease"),
+        }),
+        Orden::Version => {
+            println!("{VERSION}");
+            Ok(())
         }
+    };
+    match resultado {
+        Ok(()) => ExitCode::SUCCESS,
         Err(e) => {
             eprintln!("xtask: ROJO: {e}");
             ExitCode::from(1)
@@ -42,20 +85,32 @@ fn main() -> ExitCode {
     }
 }
 
-fn leer_perfil(args: &[String]) -> Result<Perfil, String> {
-    match args {
-        [orden, flag, valor] if orden == "verificar" && flag == "--perfil" => {
-            match valor.as_str() {
-                "rapida" => Ok(Perfil::Rapida),
-                "completa" => Ok(Perfil::Completa),
-                otro => Err(format!("perfil desconocido «{otro}»")),
-            }
+fn leer_orden(args: &[String]) -> Result<Orden, String> {
+    let a: Vec<&str> = args.iter().map(String::as_str).collect();
+    match a.as_slice() {
+        ["verificar", "--perfil", valor] => match *valor {
+            "rapida" => Ok(Orden::Verificar(Perfil::Rapida)),
+            "completa" => Ok(Orden::Verificar(Perfil::Completa)),
+            otro => Err(format!("perfil desconocido «{otro}»")),
+        },
+        ["empaquetar", "--target", triple, "--salida", salida] => {
+            let target = Target::de_triple(triple)
+                .ok_or_else(|| format!("target «{triple}» fuera de los 6 de P12"))?;
+            Ok(Orden::Empaquetar {
+                target,
+                salida: PathBuf::from(salida),
+            })
         }
+        ["sumas", dir] => Ok(Orden::Sumas(PathBuf::from(dir))),
+        ["comprobar-tag", tag] => Ok(Orden::ComprobarTag((*tag).to_owned())),
+        ["version"] => Ok(Orden::Version),
         _ => Err("argumentos no validos".to_owned()),
     }
 }
 
-fn verificar(perfil: Perfil) -> Result<(), String> {
+/// Un RUSTFLAGS en el entorno anularia en silencio los rustflags de
+/// `.cargo/config.toml` (/Brepro).
+fn guarda_rustflags() -> Result<(), String> {
     for var in ["RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"] {
         if std::env::var_os(var).is_some() {
             return Err(format!(
@@ -63,6 +118,113 @@ fn verificar(perfil: Perfil) -> Result<(), String> {
             ));
         }
     }
+    Ok(())
+}
+
+/// Compila `tc2` para `target`, lo deja en `salida` con su nombre publicado,
+/// lo firma ad hoc en macOS y escribe al lado `<fichero>.sha256`.
+fn empaquetar(target: Target, salida: &Path) -> Result<(), String> {
+    guarda_rustflags()?;
+    let raiz = raiz_workspace()?;
+    cargo(
+        &raiz,
+        &[
+            "build",
+            "--release",
+            "--locked",
+            "-p",
+            "tc2",
+            "--target",
+            target.triple,
+        ],
+        None,
+    )?;
+    // cargo corre en `raiz`: un CARGO_TARGET_DIR relativo es relativo a ella.
+    // (`join` con una ruta absoluta la devuelve tal cual.)
+    let dir_target = std::env::var_os("CARGO_TARGET_DIR")
+        .map(|d| raiz.join(d))
+        .unwrap_or_else(|| raiz.join("target"));
+    let origen = dir_target
+        .join(target.triple)
+        .join("release")
+        .join(target.binario_cargo());
+    std::fs::create_dir_all(salida).map_err(|e| format!("{}: {e}", salida.display()))?;
+    let fichero = target.fichero(VERSION);
+    let destino = salida.join(&fichero);
+    std::fs::copy(&origen, &destino).map_err(|e| {
+        format!(
+            "no se puede copiar {} a {}: {e}",
+            origen.display(),
+            destino.display()
+        )
+    })?;
+    if target.es_macos() {
+        // Firma ad hoc (D4a): sin ella Gatekeeper mata un binario arm64.
+        ejecutar("codesign", &["--force", "--sign", "-"], &destino)?;
+        ejecutar(
+            "codesign",
+            &["--verify", "--strict", "--verbose=2"],
+            &destino,
+        )?;
+    }
+    let bytes = std::fs::read(&destino).map_err(|e| format!("{}: {e}", destino.display()))?;
+    let hash = sha256::hex(&bytes);
+    let suma = salida.join(format!("{fichero}{}", dist::SUFIJO_SUMA));
+    std::fs::write(&suma, dist::linea_suma(&hash, &fichero))
+        .map_err(|e| format!("{}: {e}", suma.display()))?;
+    println!(
+        "xtask: empaquetado {} ({} bytes)\nxtask: sha256 {hash}  {fichero}",
+        destino.display(),
+        bytes.len()
+    );
+    Ok(())
+}
+
+/// Lanza `programa args... fichero` y exige codigo 0.
+fn ejecutar(programa: &str, args: &[&str], fichero: &Path) -> Result<(), String> {
+    println!("xtask: {programa} {} {}", args.join(" "), fichero.display());
+    let estado = Command::new(programa)
+        .args(args)
+        .arg(fichero)
+        .status()
+        .map_err(|e| format!("no se pudo lanzar {programa}: {e}"))?;
+    if estado.success() {
+        Ok(())
+    } else {
+        Err(format!("falla `{programa} {}` ({estado})", args.join(" ")))
+    }
+}
+
+/// Escribe `<dir>/SHA256SUMS` con los binarios de `<dir>`: todo fichero que
+/// no sea el propio `SHA256SUMS` ni un `<fichero>.sha256`.
+fn escribir_sumas(dir: &Path) -> Result<(), String> {
+    let lectura = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
+    let mut entradas = Vec::new();
+    for entrada in lectura {
+        let entrada = entrada.map_err(|e| format!("{}: {e}", dir.display()))?;
+        let ruta = entrada.path();
+        let nombre = entrada
+            .file_name()
+            .into_string()
+            .map_err(|_| format!("nombre no UTF-8 en {}", dir.display()))?;
+        if nombre == dist::SUMAS || nombre.ends_with(dist::SUFIJO_SUMA) {
+            continue;
+        }
+        if !ruta.is_file() {
+            return Err(format!("{} no es un fichero", ruta.display()));
+        }
+        let bytes = std::fs::read(&ruta).map_err(|e| format!("{}: {e}", ruta.display()))?;
+        entradas.push((nombre, sha256::hex(&bytes)));
+    }
+    let texto = dist::sumas(&entradas)?;
+    let destino = dir.join(dist::SUMAS);
+    std::fs::write(&destino, &texto).map_err(|e| format!("{}: {e}", destino.display()))?;
+    print!("{texto}");
+    Ok(())
+}
+
+fn verificar(perfil: Perfil) -> Result<(), String> {
+    guarda_rustflags()?;
     let raiz = raiz_workspace()?;
     cargo(&raiz, &["fmt", "--all", "--check"], None)?;
     cargo(
@@ -221,15 +383,52 @@ mod tests {
     #[test]
     fn perfiles_validos_e_invalidos() {
         assert_eq!(
-            leer_perfil(&v(&["verificar", "--perfil", "rapida"])),
-            Ok(Perfil::Rapida)
+            leer_orden(&v(&["verificar", "--perfil", "rapida"])),
+            Ok(Orden::Verificar(Perfil::Rapida))
+        );
+        assert_eq!(
+            leer_orden(&v(&["verificar", "--perfil", "completa"])),
+            Ok(Orden::Verificar(Perfil::Completa))
+        );
+        assert!(leer_orden(&v(&["verificar", "--perfil", "otra"])).is_err());
+        assert!(leer_orden(&v(&["verificar"])).is_err());
+    }
+
+    #[test]
+    fn ordenes_de_distribucion() {
+        assert_eq!(
+            leer_orden(&v(&[
+                "empaquetar",
+                "--target",
+                "aarch64-apple-darwin",
+                "--salida",
+                "dist"
+            ])),
+            Ok(Orden::Empaquetar {
+                target: dist::TARGETS[3],
+                salida: PathBuf::from("dist")
+            })
+        );
+        assert!(
+            leer_orden(&v(&[
+                "empaquetar",
+                "--target",
+                "x86_64-unknown-linux-gnu",
+                "--salida",
+                "dist"
+            ]))
+            .is_err()
+        );
+        assert_eq!(
+            leer_orden(&v(&["sumas", "dist"])),
+            Ok(Orden::Sumas(PathBuf::from("dist")))
         );
         assert_eq!(
-            leer_perfil(&v(&["verificar", "--perfil", "completa"])),
-            Ok(Perfil::Completa)
+            leer_orden(&v(&["comprobar-tag", "v1.0.0"])),
+            Ok(Orden::ComprobarTag("v1.0.0".to_owned()))
         );
-        assert!(leer_perfil(&v(&["verificar", "--perfil", "otra"])).is_err());
-        assert!(leer_perfil(&v(&["verificar"])).is_err());
+        assert_eq!(leer_orden(&v(&["version"])), Ok(Orden::Version));
+        assert!(leer_orden(&v(&["version", "x"])).is_err());
     }
 
     #[test]
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-025/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-025/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-025/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/git_binario.rs"
````

````
crates/tc2/tests/git_binario.rs | 5 +++++
 1 file changed, 5 insertions(+)
````
