# Peticion de revision — TASK-016 (ronda 1)

- Tarea: TASK-016 — F2-1 Esqueleto: Rust, build reproducible
- Rama revisada: feature/task-016-f2-1-esqueleto-typescript-build-reproduc
- Rama base: develop
- Commit revisado (HEAD): 9d72eae2729bfc231921d0eb9a25373365dc10b3
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
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
informe de esta ronda (informe-revision-1-rust-wasm-reviewer.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-016/tarea.md
- tareas/03-en-revision/TASK-016/planificacion/
- .cargo/config.toml
- Cargo.toml
- crates/tc2/Cargo.toml
- crates/tc2/src/lib.rs
- crates/tc2/src/main.rs
- crates/tc2/tests/cli.rs
- rust-toolchain.toml
- rustfmt.toml
- xtask/Cargo.toml
- xtask/src/main.rs
- xtask/src/sha256.rs

## Commits a revisar (git log develop..HEAD)

````
9d72eae chore(TASK-016): autorrevision
e904ef8 feat(TASK-016): esqueleto Rust de tc2 con xtask verificar y CI en tres sistemas
b20cdd7 docs(TASK-016): F2-1 corregida a Rust y reproducibilidad solo en Windows
c7b299d chore(TASK-016): tarea en curso
````

## Diff de tu dominio (11 fichero(s) de 16; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/.cargo/config.toml b/.cargo/config.toml
new file mode 100644
index 0000000..c0fae5a
--- /dev/null
+++ b/.cargo/config.toml
@@ -0,0 +1,12 @@
+# Configuracion de cargo versionada (F2-1, ADR F1-14).
+# /Brepro: sin el, MSVC mete la fecha en el PE y dos builds en limpio dan
+# distinto sha256 (PoC B de F1-14). Un RUSTFLAGS en el entorno anularia estos
+# rustflags en silencio: `cargo xtask verificar` se niega a seguir si existe.
+[target.x86_64-pc-windows-msvc]
+rustflags = ["-C", "link-arg=/Brepro"]
+
+[target.aarch64-pc-windows-msvc]
+rustflags = ["-C", "link-arg=/Brepro"]
+
+[alias]
+xtask = "run --quiet --locked -p xtask --"
diff --git a/Cargo.toml b/Cargo.toml
new file mode 100644
index 0000000..d2a7994
--- /dev/null
+++ b/Cargo.toml
@@ -0,0 +1,31 @@
+# Workspace de TaskCodeTwo (F2-1, ADR F1-14): el CLI `tc2` y la herramienta
+# de verificacion `xtask` (`cargo xtask verificar --perfil rapida|completa`).
+[workspace]
+resolver = "3"
+members = ["crates/tc2", "xtask"]
+
+[workspace.package]
+version = "0.0.0"
+edition = "2024"
+rust-version = "1.100"
+license = "MIT"
+repository = "https://github.com/charliebk/TaskCodeTwo"
+publish = false
+
+[workspace.lints.rust]
+unsafe_code = "forbid"
+unused = { level = "warn", priority = -1 }
+
+[workspace.lints.clippy]
+all = { level = "warn", priority = -1 }
+unwrap_used = "warn"
+expect_used = "warn"
+panic = "warn"
+
+# Perfil de release de F1-14 (punto 3). Los perfiles solo valen en la raiz.
+[profile.release]
+opt-level = 3
+lto = "thin"
+codegen-units = 1
+strip = true
+panic = "abort"
diff --git a/crates/tc2/Cargo.toml b/crates/tc2/Cargo.toml
new file mode 100644
index 0000000..000b656
--- /dev/null
+++ b/crates/tc2/Cargo.toml
@@ -0,0 +1,16 @@
+[package]
+name = "tc2"
+description = "CLI de TaskCodeTwo"
+version.workspace = true
+edition.workspace = true
+rust-version.workspace = true
+license.workspace = true
+repository.workspace = true
+publish.workspace = true
+
+[[bin]]
+name = "tc2"
+path = "src/main.rs"
+
+[lints]
+workspace = true
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
new file mode 100644
index 0000000..69e8f3a
--- /dev/null
+++ b/crates/tc2/src/lib.rs
@@ -0,0 +1,65 @@
+//! Nucleo del CLI `tc2` de TaskCodeTwo.
+//!
+//! El esqueleto (F2-1) solo sabe decir su version. Las capas que vienen
+//! (operaciones Git de F2-2, modelo de tarea de F2-5...) entran como modulos
+//! de esta biblioteca.
+
+/// Version del CLI, la de `Cargo.toml`.
+pub const VERSION: &str = env!("CARGO_PKG_VERSION");
+
+/// Resultado de interpretar la linea de ordenes: que escribir y con que
+/// codigo salir. Sin efectos, para poder probarlo sin lanzar procesos.
+#[derive(Debug, PartialEq, Eq)]
+pub struct Salida {
+    /// Codigo de salida del proceso.
+    pub codigo: u8,
+    /// Texto para la salida estandar (vacio si no hay).
+    pub stdout: String,
+    /// Texto para la salida de error (vacio si no hay).
+    pub stderr: String,
+}
+
+/// Interpreta los argumentos (sin el nombre del programa).
+pub fn ejecutar<I, S>(args: I) -> Salida
+where
+    I: IntoIterator<Item = S>,
+    S: AsRef<str>,
+{
+    let args: Vec<String> = args.into_iter().map(|a| a.as_ref().to_owned()).collect();
+    match args.as_slice() {
+        [a] if a == "--version" || a == "-V" => Salida {
+            codigo: 0,
+            stdout: format!("tc2 {VERSION}\n"),
+            stderr: String::new(),
+        },
+        [] => Salida {
+            codigo: 2,
+            stdout: String::new(),
+            stderr: "tc2: falta la orden. Uso: tc2 --version\n".to_owned(),
+        },
+        [otro, ..] => Salida {
+            codigo: 2,
+            stdout: String::new(),
+            stderr: format!("tc2: orden desconocida «{otro}». Uso: tc2 --version\n"),
+        },
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn version_corta_y_larga() {
+        for a in ["--version", "-V"] {
+            let s = ejecutar([a]);
+            assert_eq!(s.codigo, 0);
+            assert_eq!(s.stdout, format!("tc2 {VERSION}\n"));
+        }
+    }
+
+    #[test]
+    fn sin_orden_falla() {
+        assert_eq!(ejecutar(Vec::<String>::new()).codigo, 2);
+    }
+}
diff --git a/crates/tc2/src/main.rs b/crates/tc2/src/main.rs
new file mode 100644
index 0000000..4e003bc
--- /dev/null
+++ b/crates/tc2/src/main.rs
@@ -0,0 +1,21 @@
+//! Punto de entrada de `tc2`: delega en la biblioteca y solo escribe.
+
+use std::io::Write;
+use std::process::ExitCode;
+
+fn main() -> ExitCode {
+    let mut args = Vec::new();
+    for a in std::env::args_os().skip(1) {
+        match a.into_string() {
+            Ok(s) => args.push(s),
+            Err(_) => {
+                let _ = writeln!(std::io::stderr(), "tc2: argumento que no es UTF-8");
+                return ExitCode::from(2);
+            }
+        }
+    }
+    let salida = tc2::ejecutar(args);
+    let _ = std::io::stdout().write_all(salida.stdout.as_bytes());
+    let _ = std::io::stderr().write_all(salida.stderr.as_bytes());
+    ExitCode::from(salida.codigo)
+}
diff --git a/rust-toolchain.toml b/rust-toolchain.toml
new file mode 100644
index 0000000..5160948
--- /dev/null
+++ b/rust-toolchain.toml
@@ -0,0 +1,5 @@
+# Toolchain fija (F2-1): subirla es una tarea propia, no un efecto secundario.
+[toolchain]
+channel = "1.100.0"
+components = ["rustfmt", "clippy"]
+profile = "minimal"
diff --git a/rustfmt.toml b/rustfmt.toml
new file mode 100644
index 0000000..f216078
--- /dev/null
+++ b/rustfmt.toml
@@ -0,0 +1 @@
+edition = "2024"
diff --git a/xtask/Cargo.toml b/xtask/Cargo.toml
new file mode 100644
index 0000000..d33a2fb
--- /dev/null
+++ b/xtask/Cargo.toml
@@ -0,0 +1,11 @@
+[package]
+name = "xtask"
+description = "Verificacion del workspace: cargo xtask verificar --perfil rapida|completa"
+version.workspace = true
+edition.workspace = true
+rust-version.workspace = true
+license.workspace = true
+publish.workspace = true
+
+[lints]
+workspace = true
diff --git a/xtask/src/main.rs b/xtask/src/main.rs
new file mode 100644
index 0000000..2607322
--- /dev/null
+++ b/xtask/src/main.rs
@@ -0,0 +1,220 @@
+//! `cargo xtask verificar --perfil rapida|completa`: el unico punto de entrada
+//! de verificacion del workspace, el mismo en local y en la CI (F2-1).
+//!
+//! - `rapida`: `cargo fmt --check`, `clippy -D warnings` y `cargo test`, todo
+//!   con `--locked`.
+//! - `completa`: lo anterior y la comprobacion de build reproducible: copia el
+//!   arbol a dos rutas distintas, compila `tc2` en release en limpio en cada
+//!   una y exige el mismo sha256 (ADR F1-14). La CI solo la corre en Windows.
+//!
+//! El resultado va en el codigo de salida: 0 verde, 1 rojo, 2 mal invocado.
+
+mod sha256;
+
+use std::path::{Path, PathBuf};
+use std::process::{Command, ExitCode};
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+enum Perfil {
+    Rapida,
+    Completa,
+}
+
+fn main() -> ExitCode {
+    let args: Vec<String> = std::env::args().skip(1).collect();
+    let perfil = match leer_perfil(&args) {
+        Ok(p) => p,
+        Err(e) => {
+            eprintln!("xtask: {e}");
+            eprintln!("Uso: cargo xtask verificar --perfil rapida|completa");
+            return ExitCode::from(2);
+        }
+    };
+    match verificar(perfil) {
+        Ok(()) => {
+            println!("xtask: verificacion {perfil:?} en verde");
+            ExitCode::SUCCESS
+        }
+        Err(e) => {
+            eprintln!("xtask: ROJO: {e}");
+            ExitCode::from(1)
+        }
+    }
+}
+
+fn leer_perfil(args: &[String]) -> Result<Perfil, String> {
+    match args {
+        [orden, flag, valor] if orden == "verificar" && flag == "--perfil" => {
+            match valor.as_str() {
+                "rapida" => Ok(Perfil::Rapida),
+                "completa" => Ok(Perfil::Completa),
+                otro => Err(format!("perfil desconocido «{otro}»")),
+            }
+        }
+        _ => Err("argumentos no validos".to_owned()),
+    }
+}
+
+fn verificar(perfil: Perfil) -> Result<(), String> {
+    for var in ["RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"] {
+        if std::env::var_os(var).is_some() {
+            return Err(format!(
+                "{var} esta definido y anularia los rustflags de .cargo/config.toml (/Brepro); quitalo"
+            ));
+        }
+    }
+    let raiz = raiz_workspace()?;
+    cargo(&raiz, &["fmt", "--all", "--check"], None)?;
+    cargo(
+        &raiz,
+        &[
+            "clippy",
+            "--workspace",
+            "--all-targets",
+            "--locked",
+            "--",
+            "-D",
+            "warnings",
+        ],
+        None,
+    )?;
+    cargo(&raiz, &["test", "--workspace", "--locked"], None)?;
+    if perfil == Perfil::Completa {
+        reproducible(&raiz)?;
+    }
+    Ok(())
+}
+
+fn raiz_workspace() -> Result<PathBuf, String> {
+    Path::new(env!("CARGO_MANIFEST_DIR"))
+        .parent()
+        .map(Path::to_path_buf)
+        .ok_or_else(|| "no se encuentra la raiz del workspace".to_owned())
+}
+
+/// Lanza cargo en `dir`; con `target`, fija su `CARGO_TARGET_DIR`.
+fn cargo(dir: &Path, args: &[&str], target: Option<&Path>) -> Result<(), String> {
+    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
+    println!("xtask: cargo {}", args.join(" "));
+    let mut cmd = Command::new(&cargo);
+    cmd.args(args).current_dir(dir);
+    if let Some(t) = target {
+        cmd.env("CARGO_TARGET_DIR", t);
+    }
+    let estado = cmd
+        .status()
+        .map_err(|e| format!("no se pudo lanzar {cargo}: {e}"))?;
+    if estado.success() {
+        Ok(())
+    } else {
+        Err(format!("falla `cargo {}` ({estado})", args.join(" ")))
+    }
+}
+
+/// Dos builds en limpio, en dos rutas distintas, con el mismo sha256.
+fn reproducible(raiz: &Path) -> Result<(), String> {
+    let base = std::env::temp_dir().join(format!("tc2-reproducible-{}", std::process::id()));
+    let resultado = reproducible_en(raiz, &base);
+    let _ = std::fs::remove_dir_all(&base);
+    resultado
+}
+
+fn reproducible_en(raiz: &Path, base: &Path) -> Result<(), String> {
+    let ficheros = ficheros_del_arbol(raiz)?;
+    let mut hashes = Vec::new();
+    // Rutas de distinta longitud a proposito: si el binario incrusta la ruta
+    // del checkout, los hashes no coinciden.
+    for nombre in ["a", "copia-b"] {
+        let dir = base.join(nombre);
+        copiar(raiz, &dir, &ficheros)?;
+        let target = dir.join("target");
+        cargo(
+            &dir,
+            &["build", "--release", "--locked", "-p", "tc2"],
+            Some(&target),
+        )?;
+        let bin = target
+            .join("release")
+            .join(format!("tc2{}", std::env::consts::EXE_SUFFIX));
+        let bytes =
+            std::fs::read(&bin).map_err(|e| format!("no se puede leer {}: {e}", bin.display()))?;
+        let hash = sha256::hex(&bytes);
+        println!(
+            "xtask: sha256 {hash}  {} ({} bytes)",
+            bin.display(),
+            bytes.len()
+        );
+        hashes.push(hash);
+    }
+    match hashes.as_slice() {
+        [a, b] if a == b => {
+            println!("xtask: build reproducible: mismo sha256 en las dos rutas");
+            Ok(())
+        }
+        _ => Err(format!("build NO reproducible: {hashes:?}")),
+    }
+}
+
+/// Ficheros versionados o nuevos no ignorados, como los ve git, salvo los que
+/// no entran en el binario.
+fn ficheros_del_arbol(raiz: &Path) -> Result<Vec<PathBuf>, String> {
+    let salida = Command::new("git")
+        .args([
+            "ls-files",
+            "-z",
+            "--cached",
+            "--others",
+            "--exclude-standard",
+        ])
+        .current_dir(raiz)
+        .output()
+        .map_err(|e| format!("no se pudo lanzar git: {e}"))?;
+    if !salida.status.success() {
+        return Err(format!("falla git ls-files ({})", salida.status));
+    }
+    let texto = String::from_utf8(salida.stdout)
+        .map_err(|_| "git ls-files no devolvio UTF-8".to_owned())?;
+    Ok(texto
+        .split('\0')
+        .filter(|s| !s.is_empty() && !s.starts_with("tareas/") && !s.starts_with("docs/"))
+        .map(PathBuf::from)
+        .collect())
+}
+
+fn copiar(raiz: &Path, destino: &Path, ficheros: &[PathBuf]) -> Result<(), String> {
+    for f in ficheros {
+        let origen = raiz.join(f);
+        if !origen.is_file() {
+            continue; // borrado en el arbol de trabajo
+        }
+        let dst = destino.join(f);
+        if let Some(padre) = dst.parent() {
+            std::fs::create_dir_all(padre).map_err(|e| format!("{}: {e}", padre.display()))?;
+        }
+        std::fs::copy(&origen, &dst).map_err(|e| format!("{}: {e}", origen.display()))?;
+    }
+    Ok(())
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn v(a: &[&str]) -> Vec<String> {
+        a.iter().map(|s| (*s).to_owned()).collect()
+    }
+
+    #[test]
+    fn perfiles_validos_e_invalidos() {
+        assert_eq!(
+            leer_perfil(&v(&["verificar", "--perfil", "rapida"])),
+            Ok(Perfil::Rapida)
+        );
+        assert_eq!(
+            leer_perfil(&v(&["verificar", "--perfil", "completa"])),
+            Ok(Perfil::Completa)
+        );
+        assert!(leer_perfil(&v(&["verificar", "--perfil", "otra"])).is_err());
+        assert!(leer_perfil(&v(&["verificar"])).is_err());
+    }
+}
diff --git a/xtask/src/sha256.rs b/xtask/src/sha256.rs
new file mode 100644
index 0000000..94ee677
--- /dev/null
+++ b/xtask/src/sha256.rs
@@ -0,0 +1,87 @@
+//! SHA-256 (FIPS 180-4) minimo, para no traer dependencias a la verificacion.
+
+const K: [u32; 64] = [
+    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
+    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
+    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
+    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
+    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
+    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
+    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
+    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
+];
+
+/// sha256 de `datos` en hexadecimal en minusculas.
+pub fn hex(datos: &[u8]) -> String {
+    let mut h: [u32; 8] = [
+        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
+        0x5be0cd19,
+    ];
+    let mut msg = datos.to_vec();
+    let bits = (datos.len() as u64).wrapping_mul(8);
+    msg.push(0x80);
+    while msg.len() % 64 != 56 {
+        msg.push(0);
+    }
+    msg.extend_from_slice(&bits.to_be_bytes());
+    for bloque in msg.as_chunks::<64>().0 {
+        let mut w = [0u32; 64];
+        for (i, palabra) in bloque.as_chunks::<4>().0.iter().enumerate() {
+            w[i] = u32::from_be_bytes(*palabra);
+        }
+        for i in 16..64 {
+            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
+            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
+            w[i] = w[i - 16]
+                .wrapping_add(s0)
+                .wrapping_add(w[i - 7])
+                .wrapping_add(s1);
+        }
+        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
+        for i in 0..64 {
+            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
+            let ch = (e & f) ^ (!e & g);
+            let t1 = hh
+                .wrapping_add(s1)
+                .wrapping_add(ch)
+                .wrapping_add(K[i])
+                .wrapping_add(w[i]);
+            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
+            let maj = (a & b) ^ (a & c) ^ (b & c);
+            let t2 = s0.wrapping_add(maj);
+            hh = g;
+            g = f;
+            f = e;
+            e = d.wrapping_add(t1);
+            d = c;
+            c = b;
+            b = a;
+            a = t1.wrapping_add(t2);
+        }
+        for (x, y) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
+            *x = x.wrapping_add(y);
+        }
+    }
+    h.iter().map(|x| format!("{x:08x}")).collect()
+}
+
+#[cfg(test)]
+mod tests {
+    use super::hex;
+
+    #[test]
+    fn vectores_de_fips_180() {
+        assert_eq!(
+            hex(b""),
+            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
+        );
+        assert_eq!(
+            hex(b"abc"),
+            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
+        );
+        assert_eq!(
+            hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
+            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
+        );
+    }
+}
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

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/cli.rs"
````

````
crates/tc2/tests/cli.rs | 29 +++++++++++++++++++++++++++++
 1 file changed, 29 insertions(+)
````
