# Peticion de revision — TASK-027 (ronda 1)

- Tarea: TASK-027 — F2-12 taskctl actualizar --cli y aviso de version nueva
- Rama revisada: feature/task-027-f2-12-taskctl-actualizar-cli-y-aviso-de
- Rama base: develop
- Commit revisado (HEAD): 991b24cb3fbdf8c89c7441744cea1e1d03cc4f96
- Fecha: 2026-11-16
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-027 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-027/tarea.md
- tareas/03-en-revision/TASK-027/planificacion/
- Cargo.toml
- crates/distribucion/Cargo.toml
- crates/distribucion/src/lib.rs
- crates/distribucion/src/sha256.rs
- crates/tc2/Cargo.toml
- crates/tc2/examples/tc2_falso.rs
- crates/tc2/src/actualizar_cli.rs
- crates/tc2/src/contrato.rs
- crates/tc2/src/descarga.rs
- crates/tc2/src/i18n/catalogo/actualizar.rs
- crates/tc2/src/i18n/catalogo/cli.rs
- crates/tc2/src/i18n/catalogo/mod.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/main.rs
- crates/tc2/src/plataforma/http.rs
- crates/tc2/src/version.rs
- crates/tc2/tests/actualizar_cli.rs
- crates/tc2/tests/cli.rs
- xtask/Cargo.toml
- xtask/src/dist.rs
- xtask/src/instaladores.rs
- xtask/src/main.rs

## Commits a revisar (git log develop..HEAD)

````
991b24c chore(TASK-027): autorrevision con rust-wasm-reviewer y code-quality-reviewer
94fd72e fix(TASK-027): descarga del binario robusta y correcciones de autorrevision
562ac0c feat(TASK-027): cargo xtask cli-lock, job actualizar-ensayo y documentacion
183f822 feat(TASK-027): tc2 actualizar --cli sustituye el binario con vuelta atras
4573706 feat(TASK-027): descarga de la release y tc2 actualizar --cli --comprobar
2d329f6 feat(TASK-027): PROTOCOLO, tc2 version --json y tc2 compatible
6b61fd9 refactor(TASK-027): caja distribucion compartida por xtask y tc2
7177e5e chore(TASK-027): tarea en curso
````

## Diff de tu dominio (22 fichero(s) de 26; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/Cargo.toml b/Cargo.toml
index d2a7994..ae97015 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -2,7 +2,7 @@
 # de verificacion `xtask` (`cargo xtask verificar --perfil rapida|completa`).
 [workspace]
 resolver = "3"
-members = ["crates/tc2", "xtask"]
+members = ["crates/distribucion", "crates/tc2", "xtask"]
 
 [workspace.package]
 version = "0.0.0"
diff --git a/crates/distribucion/Cargo.toml b/crates/distribucion/Cargo.toml
new file mode 100644
index 0000000..9a06ecb
--- /dev/null
+++ b/crates/distribucion/Cargo.toml
@@ -0,0 +1,16 @@
+[package]
+name = "distribucion"
+description = "Distribucion de tc2 (F2-10, F2-12): targets de P12, nombre publicado, sha256 y SHA256SUMS, compartidos por xtask y tc2"
+version.workspace = true
+edition.workspace = true
+rust-version.workspace = true
+license.workspace = true
+repository.workspace = true
+publish.workspace = true
+
+# Caja hoja: sin dependencias, para que xtask no traiga nada y tc2 no cambie
+# de arbol de dependencias por ella.
+[dependencies]
+
+[lints]
+workspace = true
diff --git a/crates/distribucion/src/lib.rs b/crates/distribucion/src/lib.rs
new file mode 100644
index 0000000..ec3649e
--- /dev/null
+++ b/crates/distribucion/src/lib.rs
@@ -0,0 +1,466 @@
+//! Distribucion (F2-10, F2-12, ADR F1-14): los 6 targets de P12, el nombre de
+//! cada binario publicado, el sha256, el formato de `SHA256SUMS` (escribirlo
+//! y leerlo) y la regla tag↔version.
+//!
+//! Es logica pura, sin E/S, para probarla en local. La comparten `xtask`
+//! (empaquetado y release de `.github/workflows/release.yml`) y `tc2`
+//! (`tc2 version --json` y `tc2 actualizar --cli`): asi el nombre que se
+//! publica y el que se busca al actualizar salen de la misma tabla.
+
+pub mod sha256;
+
+/// Protocolo entre el plugin y el CLI (ADR F1-14, punto 5): sube cuando
+/// cambia algo que el plugin usa (subcomandos, `--json`, codigos de salida).
+/// Vive aqui para que `tc2 version --json` y `cargo xtask cli-lock` den el
+/// mismo.
+pub const PROTOCOLO: u32 = 1;
+
+/// Releases del repositorio: `cli.lock` apunta a `<RELEASES>/download/<tag>/`.
+pub const RELEASES: &str = "https://github.com/charliebk/TaskCodeTwo/releases";
+
+/// Un target de P12: el alias publico (contrato de `cli.lock`) y el triple de
+/// Rust con el que se compila.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub struct Target {
+    /// Alias publico: `win-x64`, `linux-arm64`...
+    pub alias: &'static str,
+    /// Triple de Rust con el que se compila.
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
+    /// El target de un triple de P12.
+    pub fn de_triple(triple: &str) -> Option<Target> {
+        TARGETS.iter().copied().find(|t| t.triple == triple)
+    }
+
+    /// El target de un alias de P12.
+    pub fn de_alias(alias: &str) -> Option<Target> {
+        TARGETS.iter().copied().find(|t| t.alias == alias)
+    }
+
+    /// El target para el que se compilo este binario, por sistema y
+    /// arquitectura (un build local en Linux con glibc cuenta como el alias
+    /// de Linux de su arquitectura). `None` fuera de P12.
+    pub fn actual() -> Option<Target> {
+        Self::de_sistema(std::env::consts::OS, std::env::consts::ARCH)
+    }
+
+    /// El target de un sistema y una arquitectura con los nombres de
+    /// `std::env::consts`.
+    pub fn de_sistema(os: &str, arch: &str) -> Option<Target> {
+        let alias = match (os, arch) {
+            ("windows", "x86_64") => "win-x64",
+            ("windows", "aarch64") => "win-arm64",
+            ("macos", "x86_64") => "darwin-x64",
+            ("macos", "aarch64") => "darwin-arm64",
+            ("linux", "x86_64") => "linux-x64",
+            ("linux", "aarch64") => "linux-arm64",
+            _ => return None,
+        };
+        Self::de_alias(alias)
+    }
+
+    /// Si es un target de Windows.
+    pub fn es_windows(self) -> bool {
+        self.alias.starts_with("win-")
+    }
+
+    /// Si es un target de macOS.
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
+        format!("tc2-{version}-{}{}", self.alias, self.extension())
+    }
+
+    /// La version de un nombre publicado de este target, si lo es:
+    /// `tc2-2.0.0-win-x64.exe` → `2.0.0`. La version no puede estar vacia ni
+    /// llevar separadores de ruta ni espacios.
+    pub fn version_de_fichero(self, fichero: &str) -> Option<String> {
+        let sufijo = format!("-{}{}", self.alias, self.extension());
+        let version = fichero.strip_prefix("tc2-")?.strip_suffix(&sufijo)?;
+        version_valida(version).then(|| version.to_owned())
+    }
+
+    fn extension(self) -> &'static str {
+        if self.es_windows() { ".exe" } else { "" }
+    }
+}
+
+/// Una version con la forma de las publicadas: `1.2.3` o `1.2.3-sufijo`,
+/// con el sufijo de alfanumericos, `.` y `-`.
+pub fn version_valida(v: &str) -> bool {
+    let (nucleo, sufijo) = match v.split_once('-') {
+        Some((n, s)) => (n, Some(s)),
+        None => (v, None),
+    };
+    let partes: Vec<&str> = nucleo.split('.').collect();
+    partes.len() == 3
+        && partes
+            .iter()
+            .all(|p| !p.is_empty() && p.len() <= 9 && p.bytes().all(|b| b.is_ascii_digit()))
+        && sufijo.is_none_or(|s| {
+            !s.is_empty()
+                && s.bytes()
+                    .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
+        })
+}
+
+/// Lee un `SHA256SUMS` con el formato de [`sumas`] (tambien el modo binario
+/// de `sha256sum`, `<hex> *<fichero>`, y finales CRLF). Devuelve los pares
+/// (fichero, hash) en el orden del texto. Falla con la linea si una no tiene
+/// el formato o si un fichero se repite.
+pub fn leer_sumas(texto: &str) -> Result<Vec<(String, String)>, String> {
+    let mut pares: Vec<(String, String)> = Vec::new();
+    for (n, linea) in texto.lines().enumerate() {
+        let linea = linea.strip_suffix('\r').unwrap_or(linea);
+        if linea.is_empty() {
+            continue;
+        }
+        let formato = || {
+            format!(
+                "linea {} de {SUMAS} sin el formato `<sha256>  <fichero>`",
+                n + 1
+            )
+        };
+        let (hash, resto) = linea.split_once(' ').ok_or_else(formato)?;
+        let fichero = resto
+            .strip_prefix(' ')
+            .or_else(|| resto.strip_prefix('*'))
+            .ok_or_else(formato)?;
+        if !hash_valido(hash) || fichero.is_empty() || fichero.contains(['/', '\\']) {
+            return Err(formato());
+        }
+        if pares.iter().any(|(f, _)| f == fichero) {
+            return Err(format!("fichero repetido en {SUMAS}: {fichero}"));
+        }
+        pares.push((fichero.to_owned(), hash.to_owned()));
+    }
+    Ok(pares)
+}
+
+/// 64 hex en minusculas.
+pub fn hash_valido(hash: &str) -> bool {
+    hash.len() == 64 && hash.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
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
+        if !hash_valido(hash) {
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
+/// `cli.lock` del plugin (ADR F1-14, punto 4) para la release `tag` de
+/// `version`, desde su `SHA256SUMS`: los 6 targets, cada uno con su fichero y
+/// su sha256, y el rango de protocolo `[PROTOCOLO, PROTOCOLO]`. Falla si el
+/// tag no casa con la version, si `SHA256SUMS` no se lee o si le falta un
+/// target. Sale ordenado por la tabla de targets, asi que no depende del
+/// orden de `SHA256SUMS`.
+pub fn cli_lock(tag: &str, version: &str, texto_sumas: &str) -> Result<String, String> {
+    clase_de_tag(tag, version)?;
+    let pares = leer_sumas(texto_sumas)?;
+    let mut s = format!(
+        "# cli.lock de TaskCodeTwo (ADR F1-14, punto 4). Lo genera `cargo xtask cli-lock`\n\
+         # desde el SHA256SUMS de la release {tag}; no se edita a mano.\n\
+         formato: 1\n\
+         version: \"{version}\"\n\
+         protocolo: [{PROTOCOLO}, {PROTOCOLO}]\n\
+         base_url: \"{RELEASES}/download/{tag}/\"\n\
+         targets:\n"
+    );
+    for t in TARGETS {
+        let fichero = t.fichero(version);
+        let hash = pares
+            .iter()
+            .find(|(f, _)| *f == fichero)
+            .map(|(_, h)| h)
+            .ok_or_else(|| format!("{SUMAS} no trae {fichero}"))?;
+        s.push_str(&format!(
+            "  {}: {{fichero: \"{fichero}\", sha256: \"{hash}\"}}\n",
+            t.alias
+        ));
+    }
+    Ok(s)
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn cli_lock_con_los_6_targets_desde_sha256sums() {
+        let entradas: Vec<(String, String)> = TARGETS
+            .iter()
+            .enumerate()
+            .map(|(i, t)| (t.fichero("2.0.0"), format!("{i}").repeat(64)))
+            .collect();
+        let sumas_texto = sumas(&entradas).unwrap_or_default();
+        let lock = cli_lock("v2.0.0", "2.0.0", &sumas_texto).unwrap_or_default();
+        assert!(lock.contains("\nformato: 1\nversion: \"2.0.0\"\n"));
+        assert!(lock.contains(&format!("\nprotocolo: [{PROTOCOLO}, {PROTOCOLO}]\n")));
+        assert!(lock.contains(
+            "\nbase_url: \"https://github.com/charliebk/TaskCodeTwo/releases/download/v2.0.0/\"\n"
+        ));
+        assert!(lock.contains(&format!(
+            "\n  win-x64: {{fichero: \"tc2-2.0.0-win-x64.exe\", sha256: \"{}\"}}\n",
+            "0".repeat(64)
+        )));
+        assert_eq!(lock.lines().filter(|l| l.starts_with("  ")).count(), 6);
+        // Prerelease: el tag lleva el sufijo y los ficheros, la version.
+        assert!(
+            cli_lock("v2.0.0-rc.1", "2.0.0", &sumas_texto)
+                .is_ok_and(|l| l.contains("/download/v2.0.0-rc.1/\""))
+        );
+        // Falta un target, o el tag no casa.
+        let cinco = sumas(&entradas[..5]).unwrap_or_default();
+        assert!(cli_lock("v2.0.0", "2.0.0", &cinco).is_err());
+        assert!(cli_lock("v2.0.1", "2.0.0", &sumas_texto).is_err());
+    }
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
+        // Los triples, literales: son los que pide rustup en la CI.
+        let triples: Vec<&str> = TARGETS.iter().map(|t| t.triple).collect();
+        assert_eq!(
+            triples,
+            [
+                "x86_64-pc-windows-msvc",
+                "aarch64-pc-windows-msvc",
+                "x86_64-apple-darwin",
+                "aarch64-apple-darwin",
+                "x86_64-unknown-linux-musl",
+                "aarch64-unknown-linux-musl"
+            ]
+        );
+        let macos: Vec<&str> = TARGETS
+            .iter()
+            .filter(|t| t.es_macos())
+            .map(|t| t.alias)
+            .collect();
+        assert_eq!(macos, ["darwin-x64", "darwin-arm64"]);
+        assert_eq!(TARGETS[0].fichero("2.0.0"), "tc2-2.0.0-win-x64.exe");
+        assert_eq!(TARGETS[4].fichero("2.0.0"), "tc2-2.0.0-linux-x64");
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
+        // 64 caracteres, pero con una «g» que no es hex.
+        let no_hex = format!("{}g", &abc[..63]);
+        assert!(sumas(&[(String::from("x"), no_hex)]).is_err());
+        for malo in ["", "dir/x", "dir\\x", "x\ny", "x\ry"] {
+            assert!(
+                sumas(&[(malo.to_owned(), abc.to_owned())]).is_err(),
+                "{malo:?}"
+            );
+        }
+    }
+
+    #[test]
+    fn alias_sistema_y_version_de_fichero() {
+        for t in TARGETS {
+            assert_eq!(Target::de_alias(t.alias), Some(t));
+            assert_eq!(
+                t.version_de_fichero(&t.fichero("2.0.0-rc.1")).as_deref(),
+                Some("2.0.0-rc.1")
+            );
+        }
+        assert_eq!(Target::de_sistema("windows", "x86_64"), Some(TARGETS[0]));
+        assert_eq!(Target::de_sistema("macos", "aarch64"), Some(TARGETS[3]));
+        assert_eq!(Target::de_sistema("linux", "aarch64"), Some(TARGETS[5]));
+        assert_eq!(Target::de_sistema("freebsd", "x86_64"), None);
+        assert!(Target::actual().is_some());
+        let win = TARGETS[0];
+        for malo in [
+            "tc2-2.0.0-linux-x64",
+            "tc2-2.0.0-win-x64",
+            "tc2--win-x64.exe",
+            "tc2-2.0-win-x64.exe",
+            "tc2-../x-win-x64.exe",
+            "tc2-2.0.0-a b-win-x64.exe",
+            "x-2.0.0-win-x64.exe",
+        ] {
+            assert_eq!(win.version_de_fichero(malo), None, "{malo}");
+        }
+        assert!(version_valida("10.20.30"));
+        assert!(!version_valida("1.2.3-"));
+        assert!(!version_valida("1.2.x"));
+    }
+
+    #[test]
+    fn leer_sumas_es_la_inversa_de_sumas() {
+        let abc = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
+        let vacio = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
+        let entradas = vec![
+            (String::from("tc2-0.0.0-linux-x64"), vacio.to_owned()),
+            (String::from("tc2-0.0.0-win-x64.exe"), abc.to_owned()),
+        ];
+        let texto = sumas(&entradas).unwrap_or_default();
+        assert_eq!(leer_sumas(&texto), Ok(entradas.clone()));
+        // CRLF y modo binario de sha256sum.
+        let crlf = format!("{vacio} *tc2-0.0.0-linux-x64\r\n{abc}  tc2-0.0.0-win-x64.exe\r\n");
+        assert_eq!(leer_sumas(&crlf), Ok(entradas));
+        for malo in [
+            format!("{abc} x"),
+            format!("{abc}  dir/x"),
+            format!("{}  x", abc.to_uppercase()),
+            format!("{abc}  x\n{vacio}  x"),
+            String::from("nada"),
+        ] {
+            assert!(leer_sumas(&malo).is_err(), "{malo}");
+        }
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
diff --git a/crates/distribucion/src/sha256.rs b/crates/distribucion/src/sha256.rs
new file mode 100644
index 0000000..94ee677
--- /dev/null
+++ b/crates/distribucion/src/sha256.rs
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
diff --git a/crates/tc2/Cargo.toml b/crates/tc2/Cargo.toml
index 3b70158..1069e1d 100644
--- a/crates/tc2/Cargo.toml
+++ b/crates/tc2/Cargo.toml
@@ -27,6 +27,9 @@ serde-saphyr = { version = "2.1", default-features = false, features = ["deseria
 # `plataforma::http`.
 serde_json = "1"
 ureq = { version = "3.5", default-features = false, features = ["rustls", "platform-verifier"] }
+# F2-12: tabla de targets, sha256 y SHA256SUMS compartidos con xtask
+# (`tc2 version --json` y `tc2 actualizar --cli`).
+distribucion = { path = "../distribucion" }
 
 # Parser YAML de otra base de codigo (port de libyaml, como el PyYAML del PoC
 # de F1-9) para la ida y vuelta de los tests: con el de produccion seria
diff --git a/crates/tc2/examples/tc2_falso.rs b/crates/tc2/examples/tc2_falso.rs
new file mode 100644
index 0000000..a353bfc
--- /dev/null
+++ b/crates/tc2/examples/tc2_falso.rs
@@ -0,0 +1,49 @@
+//! Doble de prueba de TASK-027 (F2-12): un «tc2 9.9.9» para
+//! `tests/actualizar_cli.rs`, que lo sirve como la version nueva de una
+//! release de ensayo. Responde como el `tc2` real a lo unico que
+//! `actualizar --cli` y el test le preguntan:
+//!
+//! - `--version` → `tc2 9.9.9`;
+//! - `version --json` → `{version, protocolo, target, sha256}`, con el
+//!   sha256 de su propio fichero.
+//!
+//! Solo usa `std` y la caja `distribucion`, no la biblioteca `tc2`, para que
+//! el binario sea pequeno: en Windows se ha visto (TASK-027) que la pila de
+//! red local de una maquina con filtros corta a veces a los ~19 s las
+//! descargas de mas de ~450 KB por 127.0.0.1, tambien con curl y
+//! `python -m http.server`.
+//!
+//! No es parte de la distribucion: `cargo xtask empaquetar` solo copia el
+//! binario `tc2`.
+
+use std::io::Write;
+use std::process::ExitCode;
+
+use distribucion::{PROTOCOLO, Target};
+
+/// Version que dice ser.
+const VERSION: &str = "9.9.9";
+
+fn main() -> ExitCode {
+    let args: Vec<String> = std::env::args().skip(1).collect();
+    let args: Vec<&str> = args.iter().map(String::as_str).collect();
+    let texto = match args.as_slice() {
+        ["--version"] => format!("tc2 {VERSION}\n"),
+        ["version", "--json"] => {
+            let sha256 = std::env::current_exe()
+                .and_then(std::fs::read)
+                .map(|b| distribucion::sha256::hex(&b))
+                .unwrap_or_default();
+            let target = Target::actual().map_or("desconocido", |t| t.alias);
+            // Los valores no llevan comillas ni barras: alias, version y hex.
+            format!(
+                "{{\"version\":\"{VERSION}\",\"protocolo\":{PROTOCOLO},\"target\":\"{target}\",\"sha256\":\"{sha256}\"}}\n"
+            )
+        }
+        _ => return ExitCode::from(2),
+    };
+    match std::io::stdout().write_all(texto.as_bytes()) {
+        Ok(()) => ExitCode::SUCCESS,
+        Err(_) => ExitCode::from(1),
+    }
+}
diff --git a/crates/tc2/src/actualizar_cli.rs b/crates/tc2/src/actualizar_cli.rs
new file mode 100644
index 0000000..ad44d70
--- /dev/null
+++ b/crates/tc2/src/actualizar_cli.rs
@@ -0,0 +1,737 @@
+//! `tc2 actualizar --cli [--comprobar] [--version X]` (F2-12, ADR F1-14
+//! puntos 4 y 5): sustituye el `tc2` instalado por la version de la release
+//! sin reinstalar.
+//!
+//! - La version disponible sale de `SHA256SUMS` de la release (la ultima
+//!   estable, o la de `--version`): una peticion, la misma fuente que los
+//!   instaladores de F2-11.
+//! - `--comprobar` solo lee: `actualizar ok actual=X disponible=Y` si hay una
+//!   mas nueva, `sin-cambios` si no. Es el aviso de version nueva: lo usan
+//!   `doctor` y un hook opcional, nunca una consulta pasiva.
+//! - Un `tc2` de la cache del plugin no se toca (lo gestiona el plugin con su
+//!   `cli.lock`), ni uno de un gestor de paquetes.
+//! - El binario nuevo se baja junto al actual (`.tc2-<v>-<pid>.descarga`),
+//!   su sha256 lo calcula este proceso, que no es el comprobado (punto 4), y
+//!   solo si casa con `SHA256SUMS` se pregunta al nuevo `version --json` y se
+//!   sustituye. En Windows, por renombrados (actual → `<exe>.old`, nuevo → su
+//!   sitio) con vuelta atras; el `.old` lo borra la siguiente ejecucion.
+
+use std::cmp::Ordering;
+use std::path::{Path, PathBuf};
+
+use distribucion::Target;
+
+use crate::contrato::{Informe, Resultado};
+use crate::descarga::{self, Descargador, Fallo, Origen};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::actualizar as txt;
+
+/// Nombre de la orden.
+pub const NOMBRE: &str = "actualizar";
+
+/// Quien gestiona el `tc2` que se esta ejecutando.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Modo {
+    /// Instalado con la linea de instalacion (o copiado a mano): se actualiza.
+    Instalador,
+    /// La cache del plugin (`.../taskcodetwo/cli/<version>/tc2[.exe]`): lo
+    /// gestiona el plugin con `cli.lock`.
+    Plugin,
+    /// Un gestor de paquetes (npm, Homebrew, winget, Scoop, cargo): se
+    /// actualiza con el.
+    Gestor(&'static str),
+}
+
+/// Modo de un ejecutable por su ruta (sin tocar el disco). Reconoce las
+/// rutas de Windows, macOS y Linux sin depender del sistema en que corre.
+pub fn detectar_modo(ruta: &str) -> Modo {
+    let partes: Vec<String> = ruta
+        .split(['/', '\\'])
+        .filter(|p| !p.is_empty())
+        .map(str::to_ascii_lowercase)
+        .collect();
+    let n = partes.len();
+    let es_tc2 = partes.last().is_some_and(|f| f == "tc2" || f == "tc2.exe");
+    if es_tc2
+        && n >= 4
+        && partes.get(n - 4).is_some_and(|p| p == "taskcodetwo")
+        && partes.get(n - 3).is_some_and(|p| p == "cli")
+    {
+        return Modo::Plugin;
+    }
+    for p in &partes {
+        let gestor = match p.as_str() {
+            "node_modules" => "npm",
+            "cellar" | "homebrew" | ".linuxbrew" | "linuxbrew" => "homebrew",
+            "winget" => "winget",
+            "scoop" => "scoop",
+            ".cargo" => "cargo",
+            _ => continue,
+        };
+        return Modo::Gestor(gestor);
+    }
+    Modo::Instalador
+}
+
+/// Compara dos versiones `X.Y.Z[-pre]` con las reglas de semver: el nucleo
+/// por numeros y, a igual nucleo, una prerelease es anterior a la estable;
+/// entre prereleases, identificador a identificador (los numericos por valor
+/// y antes que los alfanumericos). `None` si alguna no es una version.
+pub fn comparar(a: &str, b: &str) -> Option<Ordering> {
+    let (na, pa) = partes_version(a)?;
+    let (nb, pb) = partes_version(b)?;
+    Some(na.cmp(&nb).then_with(|| match (pa, pb) {
+        (None, None) => Ordering::Equal,
+        (None, Some(_)) => Ordering::Greater,
+        (Some(_), None) => Ordering::Less,
+        (Some(x), Some(y)) => comparar_pre(x, y),
+    }))
+}
+
+fn partes_version(v: &str) -> Option<([u64; 3], Option<&str>)> {
+    if !distribucion::version_valida(v) {
+        return None;
+    }
+    let (nucleo, pre) = match v.split_once('-') {
+        Some((n, p)) => (n, Some(p)),
+        None => (v, None),
+    };
+    let mut n = [0u64; 3];
+    for (i, p) in nucleo.split('.').enumerate() {
+        *n.get_mut(i)? = p.parse().ok()?;
+    }
+    Some((n, pre))
+}
+
+fn comparar_pre(a: &str, b: &str) -> Ordering {
+    let mut ia = a.split('.');
+    let mut ib = b.split('.');
+    loop {
+        match (ia.next(), ib.next()) {
+            (None, None) => return Ordering::Equal,
+            (None, Some(_)) => return Ordering::Less,
+            (Some(_), None) => return Ordering::Greater,
+            (Some(x), Some(y)) => {
+                let o = match (x.parse::<u64>(), y.parse::<u64>()) {
+                    (Ok(m), Ok(n)) => m.cmp(&n),
+                    (Ok(_), Err(_)) => Ordering::Less,
+                    (Err(_), Ok(_)) => Ordering::Greater,
+                    (Err(_), Err(_)) => x.cmp(y),
+                };
+                if o != Ordering::Equal {
+                    return o;
+                }
+            }
+        }
+    }
+}
+
+/// Opciones de la orden.
+#[derive(Debug, Clone, PartialEq, Eq, Default)]
+pub struct Opciones {
+    /// Solo comprobar, sin tocar nada.
+    pub comprobar: bool,
+    /// Version concreta en lugar de la ultima estable.
+    pub version: Option<String>,
+}
+
+/// Lee `--cli [--comprobar] [--version X]` (sin `--json`). Sin `--cli`, uso:
+/// actualizar un proyecto es de F9-2.
+pub fn leer_opciones(args: &[String]) -> Result<Opciones, Informe> {
+    let uso = |motivo: &str| {
+        Informe::nuevo(NOMBRE, Resultado::Uso)
+            .con("motivo", motivo)
+            .diagnostico(&txt::USO_ACTUALIZAR)
+    };
+    let mut cli = false;
+    let mut op = Opciones::default();
+    let mut i = args.iter();
+    while let Some(a) = i.next() {
+        match a.as_str() {
+            "--cli" if !cli => cli = true,
+            "--comprobar" if !op.comprobar => op.comprobar = true,
+            "--version" if op.version.is_none() => {
+                let Some(v) = i.next() else {
+                    return Err(uso("sin-version"));
+                };
+                let v = v.strip_prefix('v').unwrap_or(v);
+                if !distribucion::version_valida(v) {
+                    return Err(Informe::nuevo(NOMBRE, Resultado::Uso)
+                        .con("motivo", "version-invalida")
+                        .diagnostico(Mensaje::nuevo(&txt::VERSION_INVALIDA).con("version", v)));
+                }
+                op.version = Some(v.to_owned());
+            }
+            _ => return Err(uso("argumentos")),
+        }
+    }
+    if cli { Ok(op) } else { Err(uso("sin-cli")) }
+}
+
+/// La orden completa para el ejecutable actual, con `TC2_BASE_URL` del
+/// entorno.
+pub fn orden(args: &[String]) -> Informe {
+    let op = match leer_opciones(args) {
+        Ok(o) => o,
+        Err(i) => return i,
+    };
+    let origen = match Origen::de_variable(std::env::var(descarga::VARIABLE_BASE).ok().as_deref()) {
+        Ok(o) => o,
+        Err(m) => {
+            return Informe::nuevo(NOMBRE, Resultado::Uso)
+                .con("motivo", "base-url")
+                .diagnostico(m);
+        }
+    };
+    let exe = match std::env::current_exe() {
+        Ok(e) => e,
+        Err(e) => {
+            return Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(
+                Mensaje::nuevo(&txt::NO_SE_LEE_EL_EJECUTABLE)
+                    .con("ruta", "current_exe")
+                    .con("error", e.to_string()),
+            );
+        }
+    };
+    actualizar(&exe, &op, &origen, &Descargador::nuevo())
+}
+
+/// Lo que publica la release para un target.
+#[derive(Debug, Clone, PartialEq, Eq)]
+struct Publicado {
+    /// Version del binario (la de su nombre).
+    version: String,
+    /// Version del tag de la release (`v<tag>`): la del binario, o con un
+    /// sufijo en una prerelease (`v0.0.0-prueba.1` publica `tc2-0.0.0-*`,
+    /// regla de `distribucion::clase_de_tag`).
+    tag: String,
+    fichero: String,
+    sha256: String,
+}
+
+/// El binario de `target` en `SHA256SUMS`. Con `tag` (la de `--version`),
+/// el de esa release: su version es la del tag o la del tag sin el sufijo de
+/// prerelease. Sin `tag`, el de version mas alta, y el tag es su version (la
+/// ultima estable).
+fn elegir(pares: &[(String, String)], target: Target, tag: Option<&str>) -> Option<Publicado> {
+    let mut candidatos = pares.iter().filter_map(|(f, h)| {
+        target.version_de_fichero(f).map(|v| Publicado {
+            tag: tag.map_or_else(|| v.clone(), str::to_owned),
+            version: v,
+            fichero: f.clone(),
+            sha256: h.clone(),
+        })
+    });
+    match tag {
+        Some(t) => candidatos.find(|p| {
+            t == p.version
+                || t.strip_prefix(p.version.as_str())
+                    .is_some_and(|r| r.starts_with('-'))
+        }),
+        None => {
+            candidatos.max_by(|a, b| comparar(&a.version, &b.version).unwrap_or(Ordering::Equal))
+        }
+    }
+}
+
+/// `actualizar --cli` sobre el ejecutable `exe`.
+pub fn actualizar(exe: &Path, op: &Opciones, origen: &Origen, d: &Descargador) -> Informe {
+    limpiar_old(exe);
+    match detectar_modo(&exe.to_string_lossy()) {
+        Modo::Instalador => {}
+        Modo::Plugin => {
+            return Informe::nuevo(NOMBRE, Resultado::Rechazada)
+                .con("motivo", "gestionado-por-plugin")
+                .con("arreglo", "actualizar-el-plugin")
+                .diagnostico(
+                    Mensaje::nuevo(&txt::GESTIONADO_POR_PLUGIN)
+                        .con("ruta", exe.display().to_string()),
+                );
+        }
+        Modo::Gestor(g) => {
+            return Informe::nuevo(NOMBRE, Resultado::Rechazada)
+                .con("motivo", "gestionado-por-gestor")
+                .con("gestor", g)
+                .diagnostico(
+                    Mensaje::nuevo(&txt::GESTIONADO_POR_GESTOR)
+                        .con("gestor", g)
+                        .con("ruta", exe.display().to_string()),
+                );
+        }
+    }
+    let Some(target) = Target::actual() else {
+        return Informe::nuevo(NOMBRE, Resultado::Rechazada)
+            .con("motivo", "target-fuera-de-p12")
+            .diagnostico(
+                Mensaje::nuevo(&txt::TARGET_FUERA_DE_P12)
+                    .con("sistema", crate::version::target_actual()),
+            );
+    };
+    let url = origen.url_sumas(op.version.as_deref());
+    let texto = match d.texto(&url, descarga::LIMITE_SUMAS) {
+        Ok(t) => t,
+        Err(f) => return f.informe(NOMBRE),
+    };
+    let pares = match distribucion::leer_sumas(&texto) {
+        Ok(p) => p,
+        Err(e) => {
+            return Informe::nuevo(NOMBRE, Resultado::Rechazada)
+                .con("motivo", "sumas-invalidas")
+                .con("url", url.clone())
+                .diagnostico(
+                    Mensaje::nuevo(&txt::SUMAS_INVALIDAS)
+                        .con("url", url)
+                        .con("error", e),
+                );
+        }
+    };
+    let Some(publicado) = elegir(&pares, target, op.version.as_deref()) else {
+        let version = op
+            .version
+            .as_deref()
+            .map(|v| format!(" {v}"))
+            .unwrap_or_default();
+        return Informe::nuevo(NOMBRE, Resultado::Rechazada)
+            .con("motivo", "sin-binario")
+            .con("target", target.alias)
+            .con("url", url.clone())
+            .diagnostico(
+                Mensaje::nuevo(&txt::SIN_BINARIO)
+                    .con("url", url)
+                    .con("target", target.alias)
+                    .con("version", version),
+            );
+    };
+    let actual = crate::VERSION;
+    let orden = comparar(actual, &publicado.version);
+    // Sin --version solo se sube; con --version se instala esa, salvo si ya
+    // es la que hay.
+    let nada_que_hacer = match op.version {
+        Some(_) => orden == Some(Ordering::Equal),
+        None => orden != Some(Ordering::Less),
+    };
+    let base = |r| {
+        Informe::nuevo(NOMBRE, r)
+            .con("actual", actual)
+            .con("disponible", publicado.version.clone())
+    };
+    if nada_que_hacer {
+        return base(Resultado::SinCambios);
+    }
+    if op.comprobar {
+        return base(Resultado::Ok);
+    }
+    instalar(exe, target, &publicado, origen, d)
+}
+
+/// Baja el binario junto a `exe`, comprueba su sha256 contra `SHA256SUMS`
+/// (lo calcula este proceso, no el binario nuevo), le pide `version --json`
+/// y lo pone en el sitio de `exe`. Si algo falla antes del ultimo paso, no
+/// queda nada nuevo en la carpeta.
+fn instalar(
+    exe: &Path,
+    target: Target,
+    publicado: &Publicado,
+    origen: &Origen,
+    d: &Descargador,
+) -> Informe {
+    let Some(dir) = exe.parent() else {
+        return sin_permiso(
+            exe,
+            &std::io::Error::from(std::io::ErrorKind::NotFound).to_string(),
+        );
+    };
+    let temporal = dir.join(format!(
+        ".tc2-{}-{}.descarga",
+        publicado.version,
+        std::process::id()
+    ));
+    // Abrirlo es tambien la comprobacion de permisos: antes de bajar nada.
+    let mut fichero = match std::fs::OpenOptions::new()
+        .write(true)
+        .create_new(true)
+        .open(&temporal)
+    {
+        Ok(f) => f,
+        Err(e) => return sin_permiso(dir, &e.to_string()),
+    };
+    let guarda = Borrar(Some(temporal.clone()));
+    let url = origen.url_binario(&publicado.tag, &publicado.fichero);
+    let bajada = bajar_con_un_reintento(d, &url, &mut fichero)
+        .map_err(|f| match f {
+            Fallo::Escritura { error } => sin_permiso(&temporal, &error),
+            otro => otro.informe(NOMBRE),
+        })
+        .and_then(|_| {
+            fichero
+                .sync_all()
+                .map_err(|e| sin_permiso(&temporal, &e.to_string()))
+        });
+    drop(fichero);
+    if let Err(i) = bajada {
+        return i;
+    }
+    let obtenido = match std::fs::read(&temporal) {
+        Ok(b) => distribucion::sha256::hex(&b),
+        Err(e) => return sin_permiso(&temporal, &e.to_string()),
+    };
+    if obtenido != publicado.sha256 {
+        return Informe::nuevo(NOMBRE, Resultado::Rechazada)
+            .con("motivo", "sha256-no-casa")
+            .con("fichero", publicado.fichero.clone())
+            .con("esperado", publicado.sha256.clone())
+            .con("obtenido", obtenido.clone())
+            .diagnostico(
+                Mensaje::nuevo(&txt::SHA256_NO_CASA)
+                    .con("fichero", publicado.fichero.clone())
+                    .con("obtenido", obtenido)
+                    .con("esperado", publicado.sha256.clone()),
+            );
+    }
+    if let Err(e) = hacer_ejecutable(&temporal) {
+        return sin_permiso(&temporal, &e.to_string());
+    }
+    if let Err(detalle) = comprobar_nuevo(&temporal, &publicado.version, target.alias, &obtenido) {
+        return Informe::nuevo(NOMBRE, Resultado::Rechazada)
+            .con("motivo", "binario-no-arranca")
+            .diagnostico(
+                Mensaje::nuevo(&txt::NO_ARRANCA)
+                    .con("version", publicado.version.clone())
+                    .con("target", target.alias)
+                    .con("detalle", detalle),
+            );
+    }
+    if let Err(e) = sustituir_con(exe, &temporal, cfg!(windows), |a, b| std::fs::rename(a, b)) {
+        return Informe::nuevo(NOMBRE, Resultado::Error)
+            .con("motivo", "no-se-sustituye")
+            .con("ruta", exe.display().to_string())
+            .diagnostico(
+                Mensaje::nuevo(&txt::NO_SE_SUSTITUYE)
+                    .con("ruta", exe.display().to_string())
+                    .con("error", e.to_string()),
+            );
+    }
+    guarda.desarmar();
+    // En Unix no hay `.old`; en Windows sigue en uso hasta que este proceso
+    // acabe y lo borra la siguiente ejecucion.
+    limpiar_old(exe);
+    Informe::nuevo(NOMBRE, Resultado::Ok)
+        .con("anterior", crate::VERSION)
+        .con("version", publicado.version.clone())
+        .con("sha256", obtenido)
+}
+
+/// Baja el binario a `fichero` y, si la conexion se corta (`sinred`), lo
+/// vacia y lo intenta una vez mas. Son unos MiB por una red que puede fallar
+/// a medias; y en Windows se ha visto (TASK-027) que la inspeccion de red de
+/// un antivirus corta a veces la primera descarga de un ejecutable a los
+/// ~19 s. Los demas fallos (404, hash, limite) no se reintentan.
+fn bajar_con_un_reintento(
+    d: &Descargador,
+    url: &str,
+    fichero: &mut std::fs::File,
+) -> Result<u64, Fallo> {
+    use std::io::{Seek, SeekFrom};
+    match d.a_escritor(url, fichero, descarga::LIMITE_BINARIO) {
+        Err(Fallo::Sinred { .. }) => {
+            let vaciar = fichero
+                .set_len(0)
+                .and_then(|()| fichero.seek(SeekFrom::Start(0)).map(|_| ()));
+            if let Err(e) = vaciar {
+                return Err(Fallo::Escritura {
+                    error: e.to_string(),
+                });
+            }
+            d.a_escritor(url, fichero, descarga::LIMITE_BINARIO)
+        }
+        otro => otro,
+    }
+}
+
+/// `rechazada motivo=sin-permiso` con la ruta.
+fn sin_permiso(ruta: &Path, error: &str) -> Informe {
+    Informe::nuevo(NOMBRE, Resultado::Rechazada)
+        .con("motivo", "sin-permiso")
+        .con("ruta", ruta.display().to_string())
+        .diagnostico(
+            Mensaje::nuevo(&txt::SIN_PERMISO)
+                .con("ruta", ruta.display().to_string())
+                .con("error", error),
+        )
+}
+
+/// Borra el fichero al salir de su ambito, salvo que se desarme.
+struct Borrar(Option<PathBuf>);
+
+impl Borrar {
+    fn desarmar(mut self) {
+        self.0 = None;
+    }
+}
+
+impl Drop for Borrar {
+    fn drop(&mut self) {
+        if let Some(r) = self.0.take() {
+            let _ = std::fs::remove_file(r);
+        }
+    }
+}
+
+#[cfg(unix)]
+fn hacer_ejecutable(ruta: &Path) -> std::io::Result<()> {
+    use std::os::unix::fs::PermissionsExt;
+    std::fs::set_permissions(ruta, std::fs::Permissions::from_mode(0o755))
+}
+
+#[cfg(not(unix))]
+fn hacer_ejecutable(_ruta: &Path) -> std::io::Result<()> {
+    Ok(())
+}
+
+/// Lanza `nuevo version --json` y exige su version, su target y el sha256
+/// que ya calculo este proceso. `Err` con el detalle si no.
+fn comprobar_nuevo(nuevo: &Path, version: &str, alias: &str, sha256: &str) -> Result<(), String> {
+    let salida = std::process::Command::new(nuevo)
+        .args(["version", "--json"])
+        .stdin(std::process::Stdio::null())
+        .output()
+        .map_err(|e| e.to_string())?;
+    if !salida.status.success() {
+        return Err(format!("version --json salio con {}", salida.status));
+    }
+    let v: serde_json::Value = serde_json::from_slice(&salida.stdout)
+        .map_err(|e| format!("version --json no es JSON: {e}"))?;
+    let campo = |k: &str| {
+        v.get(k)
+            .and_then(serde_json::Value::as_str)
+            .unwrap_or_default()
+            .to_owned()
+    };
+    if campo("version") != version {
+        return Err(format!("dice ser la version {}", campo("version")));
+    }
+    if campo("target") != alias {
+        return Err(format!("dice ser del target {}", campo("target")));
+    }
+    if campo("sha256") != sha256 {
+        return Err(format!("da otro sha256 ({})", campo("sha256")));
+    }
+    if !v.get("protocolo").is_some_and(serde_json::Value::is_u64) {
+        return Err("version --json sin protocolo".to_owned());
+    }
+    Ok(())
+}
+
+/// Pone `nuevo` en el sitio de `viejo` con `renombrar`.
+///
+/// - `por_old` (Windows, donde un ejecutable en marcha se puede renombrar
+///   pero no sobrescribir): `viejo` → `<viejo>.old` y `nuevo` → `viejo`; si
+///   el segundo falla, `<viejo>.old` vuelve a `viejo` (vuelta atras) y el
+///   error lo dice.
+/// - si no (Unix): un solo renombrado atomico de `nuevo` sobre `viejo`.
+pub fn sustituir_con(
+    viejo: &Path,
+    nuevo: &Path,
+    por_old: bool,
+    mut renombrar: impl FnMut(&Path, &Path) -> std::io::Result<()>,
+) -> std::io::Result<()> {
+    if !por_old {
+        return renombrar(nuevo, viejo);
+    }
+    let old = ruta_old(viejo);
+    // Uno de una actualizacion anterior que no se pudo borrar.
+    let _ = std::fs::remove_file(&old);
+    renombrar(viejo, &old)?;
+    if let Err(e) = renombrar(nuevo, viejo) {
+        return match renombrar(&old, viejo) {
+            Ok(()) => Err(e),
+            Err(e2) => Err(std::io::Error::other(format!(
+                "{e}; y al volver atras: {e2}; el binario anterior esta en {}",
+                old.display()
+            ))),
+        };
+    }
+    Ok(())
+}
+
+/// `<exe>.old`: donde queda el binario anterior en Windows hasta la
+/// siguiente ejecucion.
+pub fn ruta_old(exe: &Path) -> PathBuf {
+    let mut s = exe.as_os_str().to_owned();
+    s.push(".old");
+    PathBuf::from(s)
+}
+
+/// Borra `<exe>.old` si lo hay (lo deja una actualizacion en Windows, donde
+/// un ejecutable en marcha no se puede borrar). Si sigue en uso, se queda.
+pub fn limpiar_old(exe: &Path) {
+    let _ = std::fs::remove_file(ruta_old(exe));
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    /// TASK-027, criterio 6: la cache del plugin se reconoce en los tres
+    /// sistemas, y no se confunde con una instalacion normal.
+    #[test]
+    fn detectar_modo_con_rutas_de_los_tres_sistemas() {
+        for plugin in [
+            r"C:\Users\ana\AppData\Local\taskcodetwo\cli\2.0.0\tc2.exe",
+            r"c:\users\ana\appdata\local\TaskCodeTwo\CLI\2.0.0-rc.1\TC2.EXE",
+            "/Users/ana/.cache/taskcodetwo/cli/2.0.0/tc2",
+            "/Users/ana/Library/Caches/taskcodetwo/cli/2.0.0/tc2",
+            "/home/ana/.cache/taskcodetwo/cli/2.0.0/tc2",
+        ] {
+            assert_eq!(detectar_modo(plugin), Modo::Plugin, "{plugin}");
+        }
+        for instalador in [
+            r"C:\Users\ana\AppData\Local\taskcodetwo\bin\tc2.exe",
+            "/Users/ana/.local/bin/tc2",
+            "/home/ana/.local/bin/tc2",
+            "/home/ana/taskcodetwo/cli/tc2",
+            "/home/ana/taskcodetwo/cli/2.0.0/otro",
+            "/opt/herramientas/tc2",
+        ] {
+            assert_eq!(detectar_modo(instalador), Modo::Instalador, "{instalador}");
+        }
+        for (ruta, gestor) in [
+            (
+                r"C:\Users\ana\AppData\Roaming\npm\node_modules\taskcodetwo\bin\tc2.exe",
+                "npm",
+            ),
+            ("/opt/homebrew/Cellar/tc2/2.0.0/bin/tc2", "homebrew"),
+            (
+                r"C:\Users\ana\AppData\Local\Microsoft\WinGet\Packages\x\tc2.exe",
+                "winget",
+            ),
+            (r"C:\Users\ana\scoop\apps\tc2\current\tc2.exe", "scoop"),
+            ("/home/ana/.cargo/bin/tc2", "cargo"),
+        ] {
+            assert_eq!(detectar_modo(ruta), Modo::Gestor(gestor), "{ruta}");
+        }
+    }
+
+    /// TASK-027, criterio 3: orden de versiones con prereleases.
+    #[test]
+    fn comparar_versiones_con_prerelease() {
+        use Ordering::{Equal, Greater, Less};
+        for (a, b, o) in [
+            ("1.0.0", "1.0.0", Equal),
+            ("1.0.0", "1.0.1", Less),
+            ("1.10.0", "1.9.0", Greater),
+            ("2.0.0-rc.1", "2.0.0", Less),
+            ("2.0.0", "2.0.0-rc.1", Greater),
+            ("2.0.0-rc.2", "2.0.0-rc.10", Less),
+            ("2.0.0-alpha", "2.0.0-alpha.1", Less),
+            ("2.0.0-1", "2.0.0-alpha", Less),
+            ("2.0.0-beta", "2.0.0-alpha", Greater),
+            ("1.9.9", "2.0.0-rc.1", Less),
+        ] {
+            assert_eq!(comparar(a, b), Some(o), "{a} {b}");
+        }
+        assert_eq!(comparar("1.0", "1.0.0"), None);
+        assert_eq!(comparar("1.0.0", "x"), None);
+    }
+
+    #[test]
+    fn opciones_de_la_orden() {
+        let v = |a: &[&str]| a.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
+        assert_eq!(leer_opciones(&v(&["--cli"])), Ok(Opciones::default()));
+        assert_eq!(
+            leer_opciones(&v(&["--comprobar", "--cli", "--version", "v2.0.0-rc.1"])),
+            Ok(Opciones {
+                comprobar: true,
+                version: Some("2.0.0-rc.1".into())
+            })
+        );
+        for (args, motivo) in [
+            (v(&[]), "sin-cli"),
+            (v(&["--comprobar"]), "sin-cli"),
+            (v(&["--cli", "--cli"]), "argumentos"),
+            (v(&["--cli", "--version"]), "sin-version"),
+            (v(&["--cli", "--version", "2.0"]), "version-invalida"),
+            (v(&["--cli", "x"]), "argumentos"),
+        ] {
+            let i = leer_opciones(&args).err();
+            assert_eq!(
+                i.as_ref().map(|i| (i.resultado, i.valor("motivo"))),
+                Some((Resultado::Uso, Some(motivo))),
+                "{args:?}"
+            );
+        }
+    }
+
+    /// Vuelta atras (plan, plan de pruebas): si falla el segundo renombrado,
+    /// el original vuelve a su sitio intacto y no queda `.old`.
+    #[test]
+    fn fallo_en_el_segundo_renombrado_deja_el_original() {
+        let dir = std::env::temp_dir().join(format!("tc2-sustituir-{}", std::process::id()));
+        let _ = std::fs::remove_dir_all(&dir);
+        assert!(std::fs::create_dir_all(&dir).is_ok());
+        let viejo = dir.join("tc2.exe");
+        let nuevo = dir.join(".tc2-9.9.9-1.descarga");
+        assert!(std::fs::write(&viejo, b"viejo").is_ok());
+        assert!(std::fs::write(&nuevo, b"nuevo").is_ok());
+        let mut llamadas = 0;
+        let r = sustituir_con(&viejo, &nuevo, true, |a, b| {
+            llamadas += 1;
+            if llamadas == 2 {
+                Err(std::io::Error::other("inyectado"))
+            } else {
+                std::fs::rename(a, b)
+            }
+        });
+        assert_eq!(r.map_err(|e| e.to_string()), Err("inyectado".to_owned()));
+        assert_eq!(llamadas, 3);
+        assert_eq!(std::fs::read(&viejo).ok().as_deref(), Some(&b"viejo"[..]));
+        assert!(!ruta_old(&viejo).exists());
+        // Sin fallos: el nuevo en su sitio y el viejo en `.old`.
+        assert!(sustituir_con(&viejo, &nuevo, true, |a, b| std::fs::rename(a, b)).is_ok());
+        assert_eq!(std::fs::read(&viejo).ok().as_deref(), Some(&b"nuevo"[..]));
+        assert_eq!(
+            std::fs::read(ruta_old(&viejo)).ok().as_deref(),
+            Some(&b"viejo"[..])
+        );
+        limpiar_old(&viejo);
+        assert!(!ruta_old(&viejo).exists());
+        let _ = std::fs::remove_dir_all(&dir);
+    }
+
+    #[test]
+    fn elegir_el_binario_del_target() {
+        let h = "a".repeat(64);
+        let pares: Vec<(String, String)> = [
+            "tc2-2.0.0-win-x64.exe",
+            "tc2-2.1.0-win-x64.exe",
+            "tc2-2.1.0-linux-x64",
+            "install.sh",
+        ]
+        .iter()
+        .map(|f| ((*f).to_owned(), h.clone()))
+        .collect();
+        let win = distribucion::TARGETS[0];
+        let linux = distribucion::TARGETS[4];
+        assert_eq!(
+            elegir(&pares, win, None).map(|p| p.version),
+            Some("2.1.0".into())
+        );
+        assert_eq!(
+            elegir(&pares, win, Some("2.0.0")).map(|p| p.fichero),
+            Some("tc2-2.0.0-win-x64.exe".into())
+        );
+        assert_eq!(elegir(&pares, linux, Some("2.0.0")), None);
+        assert_eq!(elegir(&pares, distribucion::TARGETS[1], None), None);
+        // Sin --version, el tag es la version del binario.
+        assert_eq!(
+            elegir(&pares, win, None).map(|p| p.tag),
+            Some("2.1.0".into())
+        );
+        // Prerelease: el tag v2.1.0-rc.1 publica tc2-2.1.0-*.
+        assert_eq!(
+            elegir(&pares, linux, Some("2.1.0-rc.1")).map(|p| (p.version, p.tag)),
+            Some(("2.1.0".into(), "2.1.0-rc.1".into()))
+        );
+        assert_eq!(elegir(&pares, linux, Some("2.1.00")), None);
+    }
+}
diff --git a/crates/tc2/src/contrato.rs b/crates/tc2/src/contrato.rs
index cc5c1df..523bb3f 100644
--- a/crates/tc2/src/contrato.rs
+++ b/crates/tc2/src/contrato.rs
@@ -268,7 +268,7 @@ fn una_linea(t: &str) -> String {
     t.lines().map(str::trim).collect::<Vec<_>>().join(" ")
 }
 
-fn cadena_json(t: &str) -> String {
+pub(crate) fn cadena_json(t: &str) -> String {
     let mut s = String::with_capacity(t.len() + 2);
     s.push('"');
     for c in t.chars() {
diff --git a/crates/tc2/src/descarga.rs b/crates/tc2/src/descarga.rs
new file mode 100644
index 0000000..79a8605
--- /dev/null
+++ b/crates/tc2/src/descarga.rs
@@ -0,0 +1,393 @@
+//! Descarga de la release para `tc2 actualizar --cli` (F2-12).
+//!
+//! Va aparte del puerto Plataforma: el [`Cliente`](crate::plataforma::http::Cliente)
+//! de la API no sigue redirecciones, corta en 1 MiB y solo admite rutas
+//! cerradas (contrato de F1-13), y bajar una release necesita las tres cosas
+//! al reves. Lo comparten el agente (TLS del sistema, user-agent y tiempos,
+//! [`crate::plataforma::http`]).
+//!
+//! - Origen: las releases de GitHub o, solo para ensayos, `TC2_BASE_URL`
+//!   (https, o http a `127.0.0.1:<puerto>`), la carpeta con los ficheros de
+//!   una release, como en los instaladores de F2-11.
+//! - Redirecciones: se siguen a mano, hasta [`MAX_SALTOS`], y solo a `https`
+//!   en los hosts de [`HOSTS_PERMITIDOS`] (github.com y sus hosts de assets).
+//! - Limites: [`LIMITE_SUMAS`] para `SHA256SUMS` y [`LIMITE_BINARIO`] para el
+//!   binario, que se escribe en un fichero abierto por quien llama.
+
+use std::io::{Read, Write};
+use std::time::Duration;
+
+use crate::contrato::{Informe, Resultado};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::actualizar as txt;
+use crate::plataforma::http::{self, Base, FalloHttp};
+
+/// Releases del repositorio.
+pub const RELEASES: &str = distribucion::RELEASES;
+
+/// Variable de ensayo con la carpeta de una release (la de los instaladores).
+pub const VARIABLE_BASE: &str = "TC2_BASE_URL";
+
+/// Tamano maximo de `SHA256SUMS` (6 lineas de unos 100 bytes).
+pub const LIMITE_SUMAS: u64 = 64 * 1024;
+
+/// Tamano maximo del binario (el de hoy ronda 0,5 a 3 MiB).
+pub const LIMITE_BINARIO: u64 = 64 * 1024 * 1024;
+
+/// Redirecciones que se siguen como mucho.
+pub const MAX_SALTOS: usize = 5;
+
+/// Hosts a los que se sigue una redireccion: el de las releases y los de sus
+/// assets.
+pub const HOSTS_PERMITIDOS: &[&str] = &[
+    "github.com",
+    "objects.githubusercontent.com",
+    "release-assets.githubusercontent.com",
+    "github-releases.githubusercontent.com",
+];
+
+/// Tiempo maximo de una descarga completa (el binario, por una red lenta).
+const ESPERA_DESCARGA: Duration = Duration::from_secs(120);
+
+/// De donde se baja la release.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum Origen {
+    /// Las releases de GitHub.
+    GitHub,
+    /// Ensayo: la carpeta de una release servida en `TC2_BASE_URL`.
+    Ensayo(String),
+}
+
+impl Origen {
+    /// El origen segun el valor de `TC2_BASE_URL` (si esta definida).
+    pub fn de_variable(valor: Option<&str>) -> Result<Self, Mensaje> {
+        match valor.filter(|v| !v.is_empty()) {
+            None => Ok(Self::GitHub),
+            Some(v) => Base::nueva(v)
+                .map(|b| Self::Ensayo(b.texto().to_owned()))
+                .map_err(|_| Mensaje::nuevo(&txt::BASE_URL_INVALIDA).con("url", v)),
+        }
+    }
+
+    /// URL de `SHA256SUMS`: la de la ultima release estable o la de
+    /// `version`.
+    pub fn url_sumas(&self, version: Option<&str>) -> String {
+        match (self, version) {
+            (Self::Ensayo(base), _) => format!("{base}/{}", distribucion::SUMAS),
+            (Self::GitHub, None) => format!("{RELEASES}/latest/download/{}", distribucion::SUMAS),
+            (Self::GitHub, Some(v)) => {
+                format!("{RELEASES}/download/v{v}/{}", distribucion::SUMAS)
+            }
+        }
+    }
+
+    /// URL del binario `fichero` de la release del tag `v<tag>` (por su tag,
+    /// no por `latest`: entre leer `SHA256SUMS` y bajar el binario puede
+    /// salir otra).
+    pub fn url_binario(&self, tag: &str, fichero: &str) -> String {
+        match self {
+            Self::Ensayo(base) => format!("{base}/{fichero}"),
+            Self::GitHub => format!("{RELEASES}/download/v{tag}/{fichero}"),
+        }
+    }
+}
+
+/// Por que no se pudo bajar algo.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum Fallo {
+    /// Sin conexion, DNS, tiempo agotado o TLS.
+    Sinred {
+        /// URL pedida.
+        url: String,
+        /// Error del cliente.
+        error: String,
+    },
+    /// 404: no existe esa release o ese fichero.
+    NoPublicada {
+        /// URL pedida.
+        url: String,
+        /// Estado HTTP.
+        estado: u16,
+    },
+    /// Otro estado que no es 2xx.
+    Http {
+        /// URL pedida.
+        url: String,
+        /// Estado HTTP.
+        estado: u16,
+    },
+    /// Redireccion a un destino no permitido, o demasiadas.
+    Redireccion {
+        /// URL pedida.
+        url: String,
+        /// A donde redirigia.
+        destino: String,
+    },
+    /// Pasa del limite.
+    Grande {
+        /// URL pedida.
+        url: String,
+        /// Limite en bytes.
+        limite: u64,
+    },
+    /// No se pudo escribir el fichero de destino.
+    Escritura {
+        /// Error del sistema.
+        error: String,
+    },
+}
+
+impl Fallo {
+    /// El informe de `comando` para este fallo.
+    pub fn informe(&self, comando: &str) -> Informe {
+        match self {
+            Self::Sinred { url, error } => Informe::nuevo(comando, Resultado::Sinred)
+                .con("url", url.clone())
+                .diagnostico(
+                    Mensaje::nuevo(&txt::SIN_RED)
+                        .con("url", url.clone())
+                        .con("error", error.clone()),
+                ),
+            Self::NoPublicada { url, estado } => Informe::nuevo(comando, Resultado::Rechazada)
+                .con("motivo", "no-publicada")
+                .con("url", url.clone())
+                .diagnostico(
+                    Mensaje::nuevo(&txt::NO_PUBLICADA)
+                        .con("url", url.clone())
+                        .con("estado", estado.to_string()),
+                ),
+            Self::Http { url, estado } => Informe::nuevo(comando, Resultado::Sinred)
+                .con("motivo", "http")
+                .con("estado", estado.to_string())
+                .con("url", url.clone())
+                .diagnostico(
+                    Mensaje::nuevo(&txt::HTTP_ESTADO)
+                        .con("url", url.clone())
+                        .con("estado", estado.to_string()),
+                ),
+            Self::Redireccion { url, destino } => Informe::nuevo(comando, Resultado::Rechazada)
+                .con("motivo", "redireccion-no-permitida")
+                .con("url", url.clone())
+                .diagnostico(
+                    Mensaje::nuevo(&txt::REDIRECCION_NO_PERMITIDA)
+                        .con("url", url.clone())
+                        .con("destino", destino.clone()),
+                ),
+            Self::Grande { url, limite } => Informe::nuevo(comando, Resultado::Rechazada)
+                .con("motivo", "descarga-grande")
+                .con("url", url.clone())
+                .diagnostico(
+                    Mensaje::nuevo(&txt::DESCARGA_GRANDE)
+                        .con("url", url.clone())
+                        .con("limite", limite.to_string()),
+                ),
+            Self::Escritura { error } => Informe::nuevo(comando, Resultado::Error)
+                .con("motivo", "escritura")
+                .diagnostico(
+                    Mensaje::nuevo(&txt::SIN_PERMISO)
+                        .con("ruta", "-")
+                        .con("error", error.clone()),
+                ),
+        }
+    }
+}
+
+/// Si se sigue una redireccion a `destino`: `https` a un host de
+/// [`HOSTS_PERMITIDOS`], sin usuario ni puerto.
+pub fn redireccion_permitida(destino: &str) -> bool {
+    let Some(resto) = destino.strip_prefix("https://") else {
+        return false;
+    };
+    let host = resto.split(['/', '?', '#']).next().unwrap_or_default();
+    HOSTS_PERMITIDOS.contains(&host)
+}
+
+/// Quien baja ficheros de la release.
+pub struct Descargador {
+    agente: ureq::Agent,
+}
+
+impl std::fmt::Debug for Descargador {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        f.debug_struct("Descargador").finish_non_exhaustive()
+    }
+}
+
+impl Default for Descargador {
+    fn default() -> Self {
+        Self::nuevo()
+    }
+}
+
+impl Descargador {
+    /// Con el agente de `tc2` y el tiempo de una descarga.
+    pub fn nuevo() -> Self {
+        Self {
+            agente: http::agente(ESPERA_DESCARGA),
+        }
+    }
+
+    /// Baja `url` como texto, hasta `limite` bytes.
+    pub fn texto(&self, url: &str, limite: u64) -> Result<String, Fallo> {
+        let mut v = Vec::new();
+        self.a_escritor(url, &mut v, limite)?;
+        String::from_utf8(v).map_err(|_| Fallo::Grande {
+            url: url.to_owned(),
+            limite,
+        })
+    }
+
+    /// Baja `url` a `destino`, hasta `limite` bytes. Devuelve los bytes
+    /// escritos.
+    pub fn a_escritor(
+        &self,
+        url: &str,
+        destino: &mut impl Write,
+        limite: u64,
+    ) -> Result<u64, Fallo> {
+        let mut actual = url.to_owned();
+        for _ in 0..=MAX_SALTOS {
+            let sinred = |e: String| Fallo::Sinred {
+                url: url.to_owned(),
+                error: e,
+            };
+            let mut r = match self.agente.get(&actual).call() {
+                Ok(r) => r,
+                Err(e) => {
+                    return Err(match http::clasificar(e) {
+                        FalloHttp::Sinred(e) | FalloHttp::Tls(e) | FalloHttp::Otro(e) => sinred(e),
+                        FalloHttp::RespuestaInvalida(e) => sinred(e),
+                        FalloHttp::CuerpoGrande => Fallo::Grande {
+                            url: url.to_owned(),
+                            limite,
+                        },
+                        FalloHttp::Redireccion(_) | FalloHttp::Rechazada(_) => Fallo::Redireccion {
+                            url: url.to_owned(),
+                            destino: actual,
+                        },
+                    });
+                }
+            };
+            let estado = r.status().as_u16();
+            if (300..400).contains(&estado) {
+                let destino = r
+                    .headers()
+                    .get("location")
+                    .and_then(|v| v.to_str().ok())
+                    .unwrap_or_default()
+                    .to_owned();
+                if !redireccion_permitida(&destino) {
+                    return Err(Fallo::Redireccion {
+                        url: url.to_owned(),
+                        destino,
+                    });
+                }
+                actual = destino;
+                continue;
+            }
+            if estado == 404 {
+                return Err(Fallo::NoPublicada {
+                    url: url.to_owned(),
+                    estado,
+                });
+            }
+            if !(200..300).contains(&estado) {
+                return Err(Fallo::Http {
+                    url: url.to_owned(),
+                    estado,
+                });
+            }
+            let grande = || Fallo::Grande {
+                url: url.to_owned(),
+                limite,
+            };
+            let declarado = r
+                .headers()
+                .get("content-length")
+                .and_then(|v| v.to_str().ok())
+                .and_then(|v| v.trim().parse::<u64>().ok());
+            if declarado.is_some_and(|n| n > limite) {
+                return Err(grande());
+            }
+            let mut lector = r.body_mut().as_reader().take(limite.saturating_add(1));
+            let mut total: u64 = 0;
+            let mut buf = vec![0u8; 64 * 1024];
+            loop {
+                let n = match lector.read(&mut buf) {
+                    Ok(0) => break,
+                    Ok(n) => n,
+                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
+                    Err(e) => return Err(sinred(e.to_string())),
+                };
+                total = total.saturating_add(n as u64);
+                if total > limite {
+                    return Err(grande());
+                }
+                destino
+                    .write_all(buf.get(..n).unwrap_or_default())
+                    .map_err(|e| Fallo::Escritura {
+                        error: e.to_string(),
+                    })?;
+            }
+            return Ok(total);
+        }
+        Err(Fallo::Redireccion {
+            url: url.to_owned(),
+            destino: actual,
+        })
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn solo_redirecciones_https_a_github() {
+        for bien in [
+            "https://github.com/charliebk/TaskCodeTwo/releases/download/v2.0.0/SHA256SUMS",
+            "https://objects.githubusercontent.com/x?y=1",
+            "https://release-assets.githubusercontent.com/x",
+        ] {
+            assert!(redireccion_permitida(bien), "{bien}");
+        }
+        for mal in [
+            "http://github.com/x",
+            "https://github.com.evil.example/x",
+            "https://evil.example/github.com",
+            "https://user@github.com/x",
+            "https://github.com:8443/x",
+            "/relativa",
+            "",
+        ] {
+            assert!(!redireccion_permitida(mal), "{mal}");
+        }
+    }
+
+    #[test]
+    fn urls_de_la_release() {
+        assert_eq!(
+            Origen::GitHub.url_sumas(None),
+            "https://github.com/charliebk/TaskCodeTwo/releases/latest/download/SHA256SUMS"
+        );
+        assert_eq!(
+            Origen::GitHub.url_sumas(Some("2.0.0")),
+            "https://github.com/charliebk/TaskCodeTwo/releases/download/v2.0.0/SHA256SUMS"
+        );
+        assert_eq!(
+            Origen::GitHub.url_binario("2.0.0", "tc2-2.0.0-linux-x64"),
+            "https://github.com/charliebk/TaskCodeTwo/releases/download/v2.0.0/tc2-2.0.0-linux-x64"
+        );
+        let e = Origen::de_variable(Some("http://127.0.0.1:8765/")).unwrap_or(Origen::GitHub);
+        assert_eq!(e, Origen::Ensayo("http://127.0.0.1:8765".into()));
+        assert_eq!(
+            e.url_sumas(Some("9.9.9")),
+            "http://127.0.0.1:8765/SHA256SUMS"
+        );
+        assert_eq!(Origen::de_variable(None), Ok(Origen::GitHub));
+        assert_eq!(Origen::de_variable(Some("")), Ok(Origen::GitHub));
+        for mala in ["http://evil.example", "ftp://x", "http://localhost:80"] {
+            assert!(Origen::de_variable(Some(mala)).is_err(), "{mala}");
+        }
+    }
+}
diff --git a/crates/tc2/src/i18n/catalogo/actualizar.rs b/crates/tc2/src/i18n/catalogo/actualizar.rs
new file mode 100644
index 0000000..668757f
--- /dev/null
+++ b/crates/tc2/src/i18n/catalogo/actualizar.rs
@@ -0,0 +1,89 @@
+//! Mensajes de `tc2 version`, `tc2 compatible` y `tc2 actualizar --cli`
+//! (F2-12, ADR F1-14 puntos 4 y 5).
+
+crate::i18n::textos! {
+    USO_VERSION {
+        es: "uso: version [--json]",
+        en: "usage: version [--json]",
+    }
+    NO_SE_LEE_EL_EJECUTABLE {
+        es: "no se puede leer el ejecutable de tc2 ({ruta}): {error}",
+        en: "cannot read the tc2 executable ({ruta}): {error}",
+    }
+    TARGET_FUERA_DE_P12 {
+        es: "este sistema ({sistema}) no es uno de los 6 de las releases de tc2: no hay binario que bajar",
+        en: "this system ({sistema}) is not one of the 6 in the tc2 releases: there is no binary to download",
+    }
+    USO_COMPATIBLE {
+        es: "uso: compatible [--json] --protocolo <A-B> (A y B enteros, A <= B)",
+        en: "usage: compatible [--json] --protocolo <A-B> (A and B integers, A <= B)",
+    }
+    PROTOCOLO_INCOMPATIBLE {
+        es: "este tc2 habla el protocolo {protocolo} y se pide {rango}: motivo=protocolo-incompatible arreglo=\"tc2 actualizar --cli\"",
+        en: "this tc2 speaks protocol {protocolo} and {rango} is required: motivo=protocolo-incompatible arreglo=\"tc2 actualizar --cli\"",
+    }
+    USO_ACTUALIZAR {
+        es: "uso: actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] (actualizar un proyecto, sin --cli, llega con F9-2)",
+        en: "usage: actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] (updating a project, without --cli, comes with F9-2)",
+    }
+    GESTIONADO_POR_PLUGIN {
+        es: "este tc2 es el del plugin ({ruta}) y lo actualiza el plugin: motivo=gestionado-por-plugin arreglo=\"actualiza el plugin taskcodetwo con /plugin en Claude Code\"",
+        en: "this tc2 belongs to the plugin ({ruta}) and the plugin updates it: motivo=gestionado-por-plugin arreglo=\"update the taskcodetwo plugin with /plugin in Claude Code\"",
+    }
+    GESTIONADO_POR_GESTOR {
+        es: "este tc2 lo instalo {gestor} ({ruta}): actualizalo con {gestor}, no con tc2 actualizar --cli",
+        en: "this tc2 was installed by {gestor} ({ruta}): update it with {gestor}, not with tc2 actualizar --cli",
+    }
+    BASE_URL_INVALIDA {
+        es: "TC2_BASE_URL no vale ({url}): tiene que ser https, o http a 127.0.0.1:<puerto>, sin usuario. Quitala; solo sirve para ensayos",
+        en: "TC2_BASE_URL is not valid ({url}): it must be https, or http to 127.0.0.1:<port>, without a user. Remove it; it is only for rehearsals",
+    }
+    SIN_RED {
+        es: "no se pudo bajar {url}: {error}. Comprueba la conexion o el proxy y repite",
+        en: "could not download {url}: {error}. Check the connection or the proxy and retry",
+    }
+    NO_PUBLICADA {
+        es: "{url} no existe (HTTP {estado}): comprueba la version en https://github.com/charliebk/TaskCodeTwo/releases",
+        en: "{url} does not exist (HTTP {estado}): check the version at https://github.com/charliebk/TaskCodeTwo/releases",
+    }
+    HTTP_ESTADO {
+        es: "{url} respondio HTTP {estado}; repite mas tarde",
+        en: "{url} answered HTTP {estado}; retry later",
+    }
+    REDIRECCION_NO_PERMITIDA {
+        es: "{url} redirige a un destino que no es de las releases de GitHub ({destino}): no se sigue",
+        en: "{url} redirects to a destination outside the GitHub releases ({destino}): not followed",
+    }
+    DESCARGA_GRANDE {
+        es: "{url} pasa de {limite} bytes: no es un binario de tc2",
+        en: "{url} is larger than {limite} bytes: it is not a tc2 binary",
+    }
+    SUMAS_INVALIDAS {
+        es: "{url} no es un SHA256SUMS valido: {error}",
+        en: "{url} is not a valid SHA256SUMS: {error}",
+    }
+    SIN_BINARIO {
+        es: "el SHA256SUMS de {url} no trae el binario de {target}{version}",
+        en: "the SHA256SUMS at {url} has no binary for {target}{version}",
+    }
+    VERSION_INVALIDA {
+        es: "la version tiene que ser X.Y.Z o X.Y.Z-sufijo: «{version}»",
+        en: "the version must be X.Y.Z or X.Y.Z-suffix: «{version}»",
+    }
+    SIN_PERMISO {
+        es: "no se puede escribir en {ruta}: {error}. Da permiso de escritura o reinstala con la linea de instalacion",
+        en: "cannot write to {ruta}: {error}. Grant write permission or reinstall with the install line",
+    }
+    SHA256_NO_CASA {
+        es: "el sha256 de {fichero} ({obtenido}) no casa con SHA256SUMS ({esperado}): no se ha cambiado nada. Repite; si sigue, avisa en el repositorio",
+        en: "the sha256 of {fichero} ({obtenido}) does not match SHA256SUMS ({esperado}): nothing was changed. Retry; if it persists, report it in the repository",
+    }
+    NO_ARRANCA {
+        es: "el binario descargado no responde como tc2 {version} de {target}: {detalle}. No se ha cambiado nada",
+        en: "the downloaded binary does not answer as tc2 {version} for {target}: {detalle}. Nothing was changed",
+    }
+    NO_SE_SUSTITUYE {
+        es: "no se pudo sustituir {ruta}: {error}. Queda el binario anterior; cierra los tc2 abiertos y repite",
+        en: "could not replace {ruta}: {error}. The previous binary is kept; close any running tc2 and retry",
+    }
+}
diff --git a/crates/tc2/src/i18n/catalogo/cli.rs b/crates/tc2/src/i18n/catalogo/cli.rs
index 8412dbc..3cadfae 100644
--- a/crates/tc2/src/i18n/catalogo/cli.rs
+++ b/crates/tc2/src/i18n/catalogo/cli.rs
@@ -10,8 +10,8 @@ crate::i18n::textos! {
         en: "tc2: could not write the output: {error}",
     }
     USO {
-        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\"",
-        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\"",
+        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json]",
+        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json]",
     }
     FALTA_LA_ORDEN {
         es: "tc2: falta la orden. {uso}",
diff --git a/crates/tc2/src/i18n/catalogo/mod.rs b/crates/tc2/src/i18n/catalogo/mod.rs
index 25ded12..b2ff678 100644
--- a/crates/tc2/src/i18n/catalogo/mod.rs
+++ b/crates/tc2/src/i18n/catalogo/mod.rs
@@ -1,6 +1,7 @@
 //! Catalogo de mensajes, un modulo por zona del codigo: cada carpeta de
 //! `src/` escribe sus entradas en el suyo y no se pisan.
 
+pub mod actualizar;
 pub mod cli;
 pub mod comun;
 pub mod config;
@@ -13,6 +14,7 @@ use super::Texto;
 /// Todas las entradas del catalogo.
 pub fn todos() -> impl Iterator<Item = &'static Texto> {
     [
+        actualizar::TODOS,
         cli::TODOS,
         comun::TODOS,
         config::TODOS,
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index d6cdf58..7ff52e7 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -7,20 +7,26 @@
 //! [`plataforma`] (puerto Plataforma y adaptadores GitHub y GitLab, F2-6 y
 //! F2-9), [`carril`] (tabla de suelo de F1-10) y [`config`] (configuracion
 //! con esquema validado, F2-7) e [`i18n`] (mensajes en espanol e ingles,
-//! F2-8). Las capas
+//! F2-8), [`version`] (`version --json` y `compatible`, F2-12) y
+//! [`actualizar_cli`] (`actualizar --cli`, F2-12). Las capas
 //! que vienen entran como modulos de esta biblioteca.
 
+pub mod actualizar_cli;
 pub mod alta;
 pub mod carril;
 pub mod config;
 pub mod contrato;
+pub mod descarga;
 pub mod estado;
 pub mod git;
 pub mod i18n;
 pub mod plataforma;
 pub mod tarea;
+pub mod version;
 mod yaml;
 
+pub use version::PROTOCOLO;
+
 use contrato::{Informe, Resultado};
 use i18n::catalogo::cli as txt;
 use i18n::{Idioma, Mensaje};
@@ -81,6 +87,18 @@ where
             };
             orden_new(resto).salida_en(json, idioma)
         }
+        [orden, resto @ ..] if orden == version::VERSION => {
+            let (json, resto) = con_json(resto);
+            orden_version(json, &resto, idioma)
+        }
+        [orden, resto @ ..] if orden == version::COMPATIBLE => {
+            let (json, resto) = con_json(resto);
+            version::compatible(&resto).salida_en(json, idioma)
+        }
+        [orden, resto @ ..] if orden == actualizar_cli::NOMBRE => {
+            let (json, resto) = con_json(resto);
+            actualizar_cli::orden(&resto).salida_en(json, idioma)
+        }
         [] => Salida {
             codigo: 2,
             stdout: String::new(),
@@ -107,6 +125,44 @@ where
     }
 }
 
+/// Quita `--json` de donde este y dice si estaba.
+fn con_json(args: &[String]) -> (bool, Vec<String>) {
+    let json = args.iter().any(|a| a == "--json");
+    (
+        json,
+        args.iter().filter(|a| *a != "--json").cloned().collect(),
+    )
+}
+
+/// `version [--json]`: con `--json`, exactamente `{version, protocolo,
+/// target, sha256}` (ADR F1-14, punto 5); sin el, la linea de contrato.
+fn orden_version(json: bool, args: &[String], idioma: Idioma) -> Salida {
+    if !args.is_empty() {
+        return Informe::nuevo(version::VERSION, Resultado::Uso)
+            .con("motivo", "argumentos-de-mas")
+            .diagnostico(&i18n::catalogo::actualizar::USO_VERSION)
+            .salida_en(json, idioma);
+    }
+    let datos = std::env::current_exe()
+        .map_err(|e| {
+            Informe::nuevo(version::VERSION, Resultado::Error).diagnostico(
+                Mensaje::nuevo(&i18n::catalogo::actualizar::NO_SE_LEE_EL_EJECUTABLE)
+                    .con("ruta", "current_exe")
+                    .con("error", e.to_string()),
+            )
+        })
+        .and_then(|exe| version::datos(&exe));
+    match datos {
+        Ok(d) if json => Salida {
+            codigo: 0,
+            stdout: format!("{}\n", d.json()),
+            stderr: String::new(),
+        },
+        Ok(d) => version::informe(&d).salida_en(false, idioma),
+        Err(i) => i.salida_en(json, idioma),
+    }
+}
+
 /// Las dos operaciones Git publicas (ADR F1-4 D1a), en la carpeta actual.
 fn orden_git(orden: &str, args: &[String]) -> Informe {
     let dir = match std::env::current_dir() {
@@ -211,6 +267,86 @@ mod tests {
         }
     }
 
+    /// TASK-027, criterio 4: `version --json` da exactamente las cuatro
+    /// claves del ADR F1-14 (punto 5), con el sha256 del ejecutable.
+    #[test]
+    fn version_json_exactamente_version_protocolo_target_sha256() {
+        let s = ejecutar(["version", "--json"]);
+        assert_eq!((s.codigo, s.stderr.as_str()), (0, ""));
+        let v: serde_json::Value = serde_json::from_str(&s.stdout).unwrap_or_default();
+        let claves: Vec<&str> = v
+            .as_object()
+            .map(|o| o.keys().map(String::as_str).collect())
+            .unwrap_or_default();
+        let mut ordenadas = claves.clone();
+        ordenadas.sort_unstable();
+        assert_eq!(ordenadas, ["protocolo", "sha256", "target", "version"]);
+        assert_eq!(v["version"], VERSION);
+        assert_eq!(v["protocolo"], PROTOCOLO);
+        let alias = distribucion::Target::actual().map(|t| t.alias);
+        assert_eq!(v["target"].as_str(), alias);
+        let exe = std::env::current_exe().unwrap_or_default();
+        let bytes = std::fs::read(exe).unwrap_or_default();
+        assert_eq!(v["sha256"], distribucion::sha256::hex(&bytes));
+        assert_eq!(s.stdout.lines().count(), 1);
+        // Sin --json, la linea de contrato con las mismas claves.
+        let l = ejecutar(["version"]);
+        assert_eq!(l.codigo, 0);
+        assert!(l.stdout.starts_with(&format!(
+            "version ok version={VERSION} protocolo={PROTOCOLO} target="
+        )));
+        assert_eq!(ejecutar(["version", "de-mas"]).codigo, 2);
+    }
+
+    /// TASK-027, criterio 5: un rango que no incluye el protocolo sale
+    /// `rechazada` (3) con el motivo y el arreglo, en los dos idiomas.
+    #[test]
+    fn compatible_fuera_de_rango_es_rechazada_con_el_arreglo() {
+        let fuera = format!("{}-{}", PROTOCOLO + 1, PROTOCOLO + 2);
+        let es = ejecutar_en(Idioma::Es, ["compatible", "--protocolo", &fuera]);
+        let en = ejecutar_en(Idioma::En, ["compatible", "--protocolo", &fuera]);
+        for s in [&es, &en] {
+            assert_eq!(s.codigo, 3);
+            assert_eq!(
+                s.stdout,
+                format!(
+                    "compatible rechazada protocolo={PROTOCOLO} rango={fuera} motivo=protocolo-incompatible arreglo=tc2%20actualizar%20--cli\n"
+                )
+            );
+            assert!(
+                s.stderr
+                    .contains("motivo=protocolo-incompatible arreglo=\"tc2 actualizar --cli\""),
+                "{}",
+                s.stderr
+            );
+            assert_eq!(s.stderr.lines().count(), 1);
+        }
+        assert!(es.stderr.starts_with("este tc2 habla el protocolo"));
+        assert!(en.stderr.starts_with("this tc2 speaks protocol"));
+        // En JSON el arreglo va tal cual.
+        let j = ejecutar_en(Idioma::En, ["compatible", "--json", "--protocolo", &fuera]);
+        assert_eq!(j.codigo, 3);
+        assert!(j.stdout.contains("\"arreglo\":\"tc2 actualizar --cli\""));
+        // Dentro del rango, ok; mal escrito, uso.
+        let dentro = format!("1-{PROTOCOLO}");
+        let ok = ejecutar(["compatible", "--protocolo", &dentro]);
+        assert_eq!(
+            (ok.codigo, ok.stdout),
+            (
+                0,
+                format!("compatible ok protocolo={PROTOCOLO} rango={dentro}\n")
+            )
+        );
+        for args in [
+            vec!["compatible"],
+            vec!["compatible", "--protocolo"],
+            vec!["compatible", "--protocolo", "3-1"],
+            vec!["compatible", "--protocolo", "x"],
+        ] {
+            assert_eq!(ejecutar(&args).codigo, 2, "{args:?}");
+        }
+    }
+
     #[test]
     fn sin_orden_falla() {
         assert_eq!(ejecutar(Vec::<String>::new()).codigo, 2);
diff --git a/crates/tc2/src/main.rs b/crates/tc2/src/main.rs
index 3bbfb15..f925ace 100644
--- a/crates/tc2/src/main.rs
+++ b/crates/tc2/src/main.rs
@@ -10,6 +10,13 @@ use tc2::i18n::catalogo::cli as txt;
 use tc2::i18n::{Idioma, Mensaje};
 
 fn main() -> ExitCode {
+    // F2-12: en Windows, `actualizar --cli` deja `tc2.exe.old` (un ejecutable
+    // en marcha no se puede borrar); lo borra la siguiente ejecucion. Una
+    // llamada al sistema si no existe.
+    #[cfg(windows)]
+    if let Ok(exe) = std::env::current_exe() {
+        tc2::actualizar_cli::limpiar_old(&exe);
+    }
     let idioma = Idioma::de_entorno(|k| std::env::var(k).ok());
     let mut args = Vec::new();
     for a in std::env::args_os().skip(1) {
diff --git a/crates/tc2/src/plataforma/http.rs b/crates/tc2/src/plataforma/http.rs
index fb5dbe2..eed2af5 100644
--- a/crates/tc2/src/plataforma/http.rs
+++ b/crates/tc2/src/plataforma/http.rs
@@ -206,24 +206,10 @@ impl Cliente {
     /// Cliente para una base ya validada y su plataforma (que decide las
     /// cabeceras y si la ruta admite `%2F`).
     pub fn para(base: Base, proveedor: Proveedor) -> Self {
-        let tls = ureq::tls::TlsConfig::builder()
-            .root_certs(ureq::tls::RootCerts::PlatformVerifier)
-            .build();
-        let agente: ureq::Agent = ureq::Agent::config_builder()
-            .tls_config(tls)
-            .http_status_as_error(false)
-            .max_redirects(0)
-            .max_redirects_will_error(false)
-            .save_redirect_history(false)
-            .timeout_global(Some(ESPERA_TOTAL))
-            .timeout_connect(Some(ESPERA_CONEXION))
-            .user_agent(format!("taskcodetwo-tc2/{}", crate::VERSION))
-            .build()
-            .into();
         Self {
             base,
             proveedor,
-            agente,
+            agente: agente(ESPERA_TOTAL),
             registro: RefCell::new(Vec::new()),
         }
     }
@@ -330,9 +316,31 @@ impl Cliente {
     }
 }
 
+/// Agente ureq de `tc2`: TLS con el almacen del sistema, user-agent
+/// `taskcodetwo-tc2/<version>`, sin seguir redirecciones (quien lo usa decide
+/// que hacer con un 3xx), sin convertir los estados HTTP en errores, con
+/// `espera_total` por llamada y [`ESPERA_CONEXION`] para conectar. Lo usan el
+/// [`Cliente`] de la API y la descarga de `actualizar --cli` (F2-12).
+pub(crate) fn agente(espera_total: Duration) -> ureq::Agent {
+    let tls = ureq::tls::TlsConfig::builder()
+        .root_certs(ureq::tls::RootCerts::PlatformVerifier)
+        .build();
+    ureq::Agent::config_builder()
+        .tls_config(tls)
+        .http_status_as_error(false)
+        .max_redirects(0)
+        .max_redirects_will_error(false)
+        .save_redirect_history(false)
+        .timeout_global(Some(espera_total))
+        .timeout_connect(Some(ESPERA_CONEXION))
+        .user_agent(format!("taskcodetwo-tc2/{}", crate::VERSION))
+        .build()
+        .into()
+}
+
 /// Clasifica un error de ureq sin copiar nada que pueda llevar el token
 /// (ureq no pone cabeceras en sus errores; aun asi solo se usa el tipo).
-fn clasificar(e: ureq::Error) -> FalloHttp {
+pub(crate) fn clasificar(e: ureq::Error) -> FalloHttp {
     match e {
         ureq::Error::HostNotFound
         | ureq::Error::ConnectionFailed
diff --git a/crates/tc2/src/version.rs b/crates/tc2/src/version.rs
new file mode 100644
index 0000000..f94321f
--- /dev/null
+++ b/crates/tc2/src/version.rs
@@ -0,0 +1,162 @@
+//! `tc2 version [--json]` y `tc2 compatible --protocolo A-B` (F2-12, ADR
+//! F1-14 punto 5): lo que el lanzador del plugin pregunta a un `tc2`
+//! instalado aparte para decidir si lo usa.
+//!
+//! `version --json` devuelve exactamente `{"version", "protocolo", "target",
+//! "sha256"}`. El sha256 es el del fichero del propio ejecutable: sirve para
+//! compararlo con `cli.lock`, pero quien comprueba es siempre otro proceso
+//! (punto 4): un binario manipulado podria mentir sobre si mismo.
+
+use std::path::Path;
+
+use distribucion::Target;
+
+use crate::contrato::{Informe, Resultado, cadena_json};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::actualizar as txt;
+
+/// Protocolo entre el plugin y el CLI: sube cuando cambia algo que el plugin
+/// usa (subcomandos, `--json`, codigos de salida). ADR F1-14, punto 5. Es
+/// el de la caja `distribucion`, el mismo que escribe `cargo xtask cli-lock`.
+pub const PROTOCOLO: u32 = distribucion::PROTOCOLO;
+
+/// Nombre de la orden `version`.
+pub const VERSION: &str = "version";
+/// Nombre de la orden `compatible`.
+pub const COMPATIBLE: &str = "compatible";
+
+/// Lo que devuelve `version`.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct DatosVersion {
+    /// Version del CLI.
+    pub version: String,
+    /// [`PROTOCOLO`].
+    pub protocolo: u32,
+    /// Alias de P12 (`win-x64`...) o `<os>-<arch>` fuera de P12.
+    pub target: String,
+    /// sha256 del fichero del ejecutable.
+    pub sha256: String,
+}
+
+impl DatosVersion {
+    /// El JSON de una linea con exactamente las cuatro claves, en este orden.
+    pub fn json(&self) -> String {
+        format!(
+            "{{\"version\":{},\"protocolo\":{},\"target\":{},\"sha256\":{}}}",
+            cadena_json(&self.version),
+            self.protocolo,
+            cadena_json(&self.target),
+            cadena_json(&self.sha256)
+        )
+    }
+}
+
+/// Alias de P12 de este binario o, fuera de P12, `<os>-<arch>`.
+pub fn target_actual() -> String {
+    Target::actual().map_or_else(
+        || format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
+        |t| t.alias.to_owned(),
+    )
+}
+
+/// Datos de `version` para el ejecutable de `ruta`.
+pub fn datos(ruta: &Path) -> Result<DatosVersion, Informe> {
+    let bytes = std::fs::read(ruta).map_err(|e| {
+        Informe::nuevo(VERSION, Resultado::Error).diagnostico(
+            Mensaje::nuevo(&txt::NO_SE_LEE_EL_EJECUTABLE)
+                .con("ruta", ruta.display().to_string())
+                .con("error", e.to_string()),
+        )
+    })?;
+    Ok(DatosVersion {
+        version: crate::VERSION.to_owned(),
+        protocolo: PROTOCOLO,
+        target: target_actual(),
+        sha256: distribucion::sha256::hex(&bytes),
+    })
+}
+
+/// `version` sin `--json`: la linea de contrato.
+pub fn informe(d: &DatosVersion) -> Informe {
+    Informe::nuevo(VERSION, Resultado::Ok)
+        .con("version", d.version.clone())
+        .con("protocolo", d.protocolo.to_string())
+        .con("target", d.target.clone())
+        .con("sha256", d.sha256.clone())
+}
+
+/// Un rango de protocolo `A-B` (o `A`, que es `A-A`), con `A <= B`.
+pub fn leer_rango(texto: &str) -> Option<(u32, u32)> {
+    let numero = |t: &str| {
+        (!t.is_empty() && t.len() <= 9 && t.bytes().all(|b| b.is_ascii_digit()))
+            .then(|| t.parse::<u32>().ok())
+            .flatten()
+    };
+    let (a, b) = match texto.split_once('-') {
+        Some((a, b)) => (numero(a)?, numero(b)?),
+        None => {
+            let a = numero(texto)?;
+            (a, a)
+        }
+    };
+    (a <= b).then_some((a, b))
+}
+
+/// `compatible --protocolo A-B`: `ok` si [`PROTOCOLO`] cae en el rango;
+/// si no, `rechazada` (3) con `motivo=protocolo-incompatible` y el arreglo.
+pub fn compatible(args: &[String]) -> Informe {
+    let rango = match args {
+        [p, r] if p == "--protocolo" => leer_rango(r).map(|x| (x, r.as_str())),
+        _ => None,
+    };
+    let Some(((a, b), texto)) = rango else {
+        return Informe::nuevo(COMPATIBLE, Resultado::Uso)
+            .con("motivo", "argumentos")
+            .diagnostico(&txt::USO_COMPATIBLE);
+    };
+    let base = |r| {
+        Informe::nuevo(COMPATIBLE, r)
+            .con("protocolo", PROTOCOLO.to_string())
+            .con("rango", texto)
+    };
+    if (a..=b).contains(&PROTOCOLO) {
+        return base(Resultado::Ok);
+    }
+    base(Resultado::Rechazada)
+        .con("motivo", "protocolo-incompatible")
+        .con("arreglo", "tc2 actualizar --cli")
+        .diagnostico(
+            Mensaje::nuevo(&txt::PROTOCOLO_INCOMPATIBLE)
+                .con("protocolo", PROTOCOLO.to_string())
+                .con("rango", texto),
+        )
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn rangos_de_protocolo() {
+        assert_eq!(leer_rango("1-1"), Some((1, 1)));
+        assert_eq!(leer_rango("1-3"), Some((1, 3)));
+        assert_eq!(leer_rango("2"), Some((2, 2)));
+        for malo in ["", "-", "1-", "-1", "3-1", "a-b", "1-2-3", "+1-2", "1 - 2"] {
+            assert_eq!(leer_rango(malo), None, "{malo}");
+        }
+    }
+
+    #[test]
+    fn json_con_las_cuatro_claves() {
+        let d = DatosVersion {
+            version: "2.0.0".into(),
+            protocolo: 1,
+            target: "win-x64".into(),
+            sha256: "ab".into(),
+        };
+        assert_eq!(
+            d.json(),
+            "{\"version\":\"2.0.0\",\"protocolo\":1,\"target\":\"win-x64\",\"sha256\":\"ab\"}"
+        );
+    }
+}
diff --git a/xtask/Cargo.toml b/xtask/Cargo.toml
index d33a2fb..9997a1a 100644
--- a/xtask/Cargo.toml
+++ b/xtask/Cargo.toml
@@ -9,3 +9,8 @@ publish.workspace = true
 
 [lints]
 workspace = true
+
+# F2-12: la tabla de targets, el sha256 y SHA256SUMS viven en la caja comun
+# con tc2.
+[dependencies]
+distribucion = { path = "../crates/distribucion" }
diff --git a/xtask/src/dist.rs b/xtask/src/dist.rs
deleted file mode 100644
index 1de58b5..0000000
--- a/xtask/src/dist.rs
+++ /dev/null
@@ -1,234 +0,0 @@
-//! Distribucion (F2-10, ADR F1-14): los 6 targets de P12, el nombre de cada
-//! binario publicado, el formato de `SHA256SUMS` y la regla tag↔version.
-//!
-//! Es logica pura, sin E/S, para probarla en local: el empaquetado y la
-//! release de `.github/workflows/release.yml` llaman a `cargo xtask`, que
-//! usa este modulo.
-
-/// Un target de P12: el alias publico (contrato de `cli.lock`) y el triple de
-/// Rust con el que se compila.
-#[derive(Debug, Clone, Copy, PartialEq, Eq)]
-pub struct Target {
-    pub alias: &'static str,
-    pub triple: &'static str,
-}
-
-/// Los 6 targets de P12 (ADR F1-14, punto 3). En Linux, musl estatico
-/// (decision de la persona en TASK-025): el binario no depende de la glibc
-/// del sistema.
-pub const TARGETS: [Target; 6] = [
-    Target {
-        alias: "win-x64",
-        triple: "x86_64-pc-windows-msvc",
-    },
-    Target {
-        alias: "win-arm64",
-        triple: "aarch64-pc-windows-msvc",
-    },
-    Target {
-        alias: "darwin-x64",
-        triple: "x86_64-apple-darwin",
-    },
-    Target {
-        alias: "darwin-arm64",
-        triple: "aarch64-apple-darwin",
-    },
-    Target {
-        alias: "linux-x64",
-        triple: "x86_64-unknown-linux-musl",
-    },
-    Target {
-        alias: "linux-arm64",
-        triple: "aarch64-unknown-linux-musl",
-    },
-];
-
-/// Nombre del fichero de suma de un solo binario: `<fichero>.sha256`.
-pub const SUFIJO_SUMA: &str = ".sha256";
-/// Fichero con las sumas de todos los binarios de la release.
-pub const SUMAS: &str = "SHA256SUMS";
-
-impl Target {
-    pub fn de_triple(triple: &str) -> Option<Target> {
-        TARGETS.iter().copied().find(|t| t.triple == triple)
-    }
-
-    pub fn es_windows(self) -> bool {
-        self.alias.starts_with("win-")
-    }
-
-    pub fn es_macos(self) -> bool {
-        self.alias.starts_with("darwin-")
-    }
-
-    /// Binario que deja `cargo build --target`.
-    pub fn binario_cargo(self) -> &'static str {
-        if self.es_windows() { "tc2.exe" } else { "tc2" }
-    }
-
-    /// Nombre publicado (ADR F1-14, punto 4): `tc2-<version>-<alias>[.exe]`.
-    pub fn fichero(self, version: &str) -> String {
-        let ext = if self.es_windows() { ".exe" } else { "" };
-        format!("tc2-{version}-{}{ext}", self.alias)
-    }
-}
-
-/// Una linea con el formato de `sha256sum` (modo texto): `<hex>  <fichero>`.
-pub fn linea_suma(hash: &str, fichero: &str) -> String {
-    format!("{hash}  {fichero}\n")
-}
-
-/// `SHA256SUMS` a partir de pares (fichero, hash), ordenado por fichero para
-/// que el resultado no dependa del orden en que se listan los ficheros. Falla
-/// si un hash no son 64 hex en minusculas, o si un fichero se repite o trae
-/// un separador o un salto de linea (romperia el formato).
-pub fn sumas(entradas: &[(String, String)]) -> Result<String, String> {
-    let mut ordenadas: Vec<&(String, String)> = entradas.iter().collect();
-    ordenadas.sort_by(|a, b| a.0.cmp(&b.0));
-    let mut texto = String::new();
-    let mut anterior: Option<&str> = None;
-    for (fichero, hash) in ordenadas {
-        if hash.len() != 64 || !hash.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
-            return Err(format!("hash no valido para {fichero}: «{hash}»"));
-        }
-        if fichero.is_empty() || fichero.contains(['/', '\\', '\n', '\r']) {
-            return Err(format!("nombre de fichero no valido: «{fichero}»"));
-        }
-        if anterior == Some(fichero.as_str()) {
-            return Err(format!("fichero repetido: {fichero}"));
-        }
-        anterior = Some(fichero);
-        texto.push_str(&linea_suma(hash, fichero));
-    }
-    if texto.is_empty() {
-        return Err("no hay ficheros que sumar".to_owned());
-    }
-    Ok(texto)
-}
-
-/// Clase de la release segun su tag.
-#[derive(Debug, Clone, Copy, PartialEq, Eq)]
-pub enum Clase {
-    Estable,
-    Prerelease,
-}
-
-/// El tag tiene que ser `v<version>` (estable) o `v<version>-<sufijo>`
-/// (prerelease), con `<version>` la del workspace: asi el `tc2 --version` del
-/// binario publicado coincide con el tag.
-pub fn clase_de_tag(tag: &str, version: &str) -> Result<Clase, String> {
-    let resto = tag
-        .strip_prefix('v')
-        .and_then(|r| r.strip_prefix(version))
-        .ok_or_else(|| format!("el tag «{tag}» no es v{version} ni v{version}-<sufijo>"))?;
-    if resto.is_empty() {
-        return Ok(Clase::Estable);
-    }
-    match resto.strip_prefix('-') {
-        Some(sufijo)
-            if !sufijo.is_empty()
-                && sufijo
-                    .bytes()
-                    .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-') =>
-        {
-            Ok(Clase::Prerelease)
-        }
-        _ => Err(format!(
-            "el tag «{tag}» no es v{version} ni v{version}-<sufijo>"
-        )),
-    }
-}
-
-#[cfg(test)]
-mod tests {
-    use super::*;
-
-    #[test]
-    fn seis_targets_de_p12_con_exe_solo_en_windows() {
-        let alias: Vec<&str> = TARGETS.iter().map(|t| t.alias).collect();
-        assert_eq!(
-            alias,
-            [
-                "win-x64",
-                "win-arm64",
-                "darwin-x64",
-                "darwin-arm64",
-                "linux-x64",
-                "linux-arm64"
-            ]
-        );
-        for t in TARGETS {
-            assert_eq!(Target::de_triple(t.triple), Some(t));
-            let f = t.fichero("2.0.0");
-            assert!(f.starts_with(&format!("tc2-2.0.0-{}", t.alias)));
-            assert_eq!(f.ends_with(".exe"), t.es_windows(), "{f}");
-            assert_eq!(t.binario_cargo().ends_with(".exe"), t.es_windows());
-        }
-        // Los triples, literales: son los que pide rustup en la CI.
-        let triples: Vec<&str> = TARGETS.iter().map(|t| t.triple).collect();
-        assert_eq!(
-            triples,
-            [
-                "x86_64-pc-windows-msvc",
-                "aarch64-pc-windows-msvc",
-                "x86_64-apple-darwin",
-                "aarch64-apple-darwin",
-                "x86_64-unknown-linux-musl",
-                "aarch64-unknown-linux-musl"
-            ]
-        );
-        let macos: Vec<&str> = TARGETS
-            .iter()
-            .filter(|t| t.es_macos())
-            .map(|t| t.alias)
-            .collect();
-        assert_eq!(macos, ["darwin-x64", "darwin-arm64"]);
-        assert_eq!(TARGETS[0].fichero("2.0.0"), "tc2-2.0.0-win-x64.exe");
-        assert_eq!(TARGETS[4].fichero("2.0.0"), "tc2-2.0.0-linux-x64");
-        assert_eq!(Target::de_triple("x86_64-unknown-linux-gnu"), None);
-    }
-
-    #[test]
-    fn sumas_con_formato_de_sha256sum_y_orden_estable() {
-        let vacio = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
-        let abc = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
-        let a = (String::from("tc2-0.0.0-win-x64.exe"), abc.to_owned());
-        let b = (String::from("tc2-0.0.0-linux-x64"), vacio.to_owned());
-        let esperado = format!("{vacio}  tc2-0.0.0-linux-x64\n{abc}  tc2-0.0.0-win-x64.exe\n");
-        assert_eq!(sumas(&[a.clone(), b.clone()]), Ok(esperado.clone()));
-        assert_eq!(sumas(&[b.clone(), a.clone()]), Ok(esperado));
-        assert!(sumas(&[a.clone(), a.clone()]).is_err());
-        assert!(sumas(&[]).is_err());
-        assert!(sumas(&[(String::from("x"), abc.to_uppercase())]).is_err());
-        assert!(sumas(&[(String::from("x"), String::from("ab"))]).is_err());
-        // 64 caracteres, pero con una «g» que no es hex.
-        let no_hex = format!("{}g", &abc[..63]);
-        assert!(sumas(&[(String::from("x"), no_hex)]).is_err());
-        for malo in ["", "dir/x", "dir\\x", "x\ny", "x\ry"] {
-            assert!(
-                sumas(&[(malo.to_owned(), abc.to_owned())]).is_err(),
-                "{malo:?}"
-            );
-        }
-    }
-
-    #[test]
-    fn tag_y_version_tienen_que_casar() {
-        assert_eq!(clase_de_tag("v0.0.0", "0.0.0"), Ok(Clase::Estable));
-        assert_eq!(
-            clase_de_tag("v0.0.0-prueba.1", "0.0.0"),
-            Ok(Clase::Prerelease)
-        );
-        for malo in [
-            "v0.0.1",
-            "0.0.0",
-            "v0.0.0-",
-            "v0.0.00",
-            "v0.0.0.1",
-            "v0.0.0-a b",
-            "v0.0.0-a/b",
-        ] {
-            assert!(clase_de_tag(malo, "0.0.0").is_err(), "{malo}");
-        }
-    }
-}
diff --git a/xtask/src/instaladores.rs b/xtask/src/instaladores.rs
index 791349d..f7a445b 100644
--- a/xtask/src/instaladores.rs
+++ b/xtask/src/instaladores.rs
@@ -10,7 +10,7 @@
 use std::collections::BTreeSet;
 use std::path::PathBuf;
 
-use crate::dist::TARGETS;
+use distribucion::TARGETS;
 
 fn leer(nombre: &str) -> String {
     let ruta = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
diff --git a/xtask/src/main.rs b/xtask/src/main.rs
index c0ee217..f8f51e7 100644
--- a/xtask/src/main.rs
+++ b/xtask/src/main.rs
@@ -16,18 +16,24 @@
 //! - `comprobar-tag <tag>`: escribe `estable` o `prerelease`, o falla si el
 //!   tag no casa con la version del workspace.
 //! - `version`: escribe la version del workspace.
+//! - `cli-lock <SHA256SUMS> [--tag <tag>]` (F2-12): escribe en stdout el
+//!   `cli.lock` del plugin (ADR F1-14, punto 4) para la release `<tag>` (por
+//!   defecto `v<version>`), con los 6 targets y sus sha256 tomados del
+//!   `SHA256SUMS` de esa release, nunca de un build local.
 //!
 //! El resultado va en el codigo de salida: 0 verde, 1 rojo, 2 mal invocado.
 
-mod dist;
 #[cfg(test)]
 mod instaladores;
-mod sha256;
 
 use std::path::{Path, PathBuf};
 use std::process::{Command, ExitCode};
 
-use dist::{Clase, Target};
+// La tabla de targets, el nombre publicado y el sha256 son de la caja comun
+// con tc2 (F2-12).
+use distribucion as dist;
+use distribucion::sha256;
+use distribucion::{Clase, Target};
 
 /// Version del workspace (`version.workspace = true`): la misma que `tc2`.
 const VERSION: &str = env!("CARGO_PKG_VERSION");
@@ -36,7 +42,8 @@ const USO: &str = "Uso: cargo xtask verificar --perfil rapida|completa
      cargo xtask empaquetar --target <triple> --salida <dir>
      cargo xtask sumas <dir>
      cargo xtask comprobar-tag <tag>
-     cargo xtask version";
+     cargo xtask version
+     cargo xtask cli-lock <SHA256SUMS> [--tag <tag>]";
 
 #[derive(Debug, Clone, Copy, PartialEq, Eq)]
 enum Perfil {
@@ -51,6 +58,7 @@ enum Orden {
     Sumas(PathBuf),
     ComprobarTag(String),
     Version,
+    CliLock { sumas: PathBuf, tag: String },
 }
 
 fn main() -> ExitCode {
@@ -77,6 +85,10 @@ fn main() -> ExitCode {
             println!("{VERSION}");
             Ok(())
         }
+        Orden::CliLock { sumas, tag } => std::fs::read_to_string(&sumas)
+            .map_err(|e| format!("{}: {e}", sumas.display()))
+            .and_then(|texto| dist::cli_lock(&tag, VERSION, &texto))
+            .map(|lock| print!("{lock}")),
     };
     match resultado {
         Ok(()) => ExitCode::SUCCESS,
@@ -106,6 +118,14 @@ fn leer_orden(args: &[String]) -> Result<Orden, String> {
         ["sumas", dir] => Ok(Orden::Sumas(PathBuf::from(dir))),
         ["comprobar-tag", tag] => Ok(Orden::ComprobarTag((*tag).to_owned())),
         ["version"] => Ok(Orden::Version),
+        ["cli-lock", sumas] => Ok(Orden::CliLock {
+            sumas: PathBuf::from(sumas),
+            tag: format!("v{VERSION}"),
+        }),
+        ["cli-lock", sumas, "--tag", tag] => Ok(Orden::CliLock {
+            sumas: PathBuf::from(sumas),
+            tag: (*tag).to_owned(),
+        }),
         _ => Err("argumentos no validos".to_owned()),
     }
 }
@@ -433,6 +453,22 @@ mod tests {
             Ok(Orden::ComprobarTag("v1.0.0".to_owned()))
         );
         assert_eq!(leer_orden(&v(&["version"])), Ok(Orden::Version));
+        assert_eq!(
+            leer_orden(&v(&["cli-lock", "dist/SHA256SUMS"])),
+            Ok(Orden::CliLock {
+                sumas: PathBuf::from("dist/SHA256SUMS"),
+                tag: format!("v{VERSION}")
+            })
+        );
+        assert_eq!(
+            leer_orden(&v(&["cli-lock", "S", "--tag", "v1.0.0-rc.1"])),
+            Ok(Orden::CliLock {
+                sumas: PathBuf::from("S"),
+                tag: "v1.0.0-rc.1".to_owned()
+            })
+        );
+        assert!(leer_orden(&v(&["cli-lock"])).is_err());
+        assert!(leer_orden(&v(&["cli-lock", "S", "--tag"])).is_err());
         assert!(leer_orden(&v(&["version", "x"])).is_err());
         assert!(leer_orden(&v(&["sumas", "a", "b"])).is_err());
         assert!(leer_orden(&v(&["sumas"])).is_err());
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
Cargo.lock                                                                                |  8 ++++++++
 tareas/{01-en-diseno => 02-en-curso}/TASK-027/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-027/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-027/tarea.md                                    | 16 +++++++++++++++-
 4 files changed, 23 insertions(+), 1 deletion(-)
````

## Tests del diff (2 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/actualizar_cli.rs" ":(literal)crates/tc2/tests/cli.rs"
````

````
crates/tc2/tests/actualizar_cli.rs | 355 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/cli.rs            |  19 +++++++++
 2 files changed, 374 insertions(+)
````
