//! Distribucion (F2-10, F2-12, ADR F1-14): los 6 targets de P12, el nombre de
//! cada binario publicado, el sha256, el formato de `SHA256SUMS` (escribirlo
//! y leerlo) y la regla tag↔version.
//!
//! Es logica pura, sin E/S, para probarla en local. La comparten `xtask`
//! (empaquetado y release de `.github/workflows/release.yml`) y `tc2`
//! (`tc2 version --json` y `tc2 actualizar --cli`): asi el nombre que se
//! publica y el que se busca al actualizar salen de la misma tabla.

pub mod sha256;

/// Protocolo entre el plugin y el CLI (ADR F1-14, punto 5): sube cuando
/// cambia algo que el plugin usa (subcomandos, `--json`, codigos de salida).
/// Vive aqui para que `tc2 version --json` y `cargo xtask cli-lock` den el
/// mismo.
pub const PROTOCOLO: u32 = 1;

/// Releases del repositorio: `cli.lock` apunta a `<RELEASES>/download/<tag>/`.
pub const RELEASES: &str = "https://github.com/charliebk/TaskCodeTwo/releases";

/// Un target de P12: el alias publico (contrato de `cli.lock`) y el triple de
/// Rust con el que se compila.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target {
    /// Alias publico: `win-x64`, `linux-arm64`...
    pub alias: &'static str,
    /// Triple de Rust con el que se compila.
    pub triple: &'static str,
}

/// Los 6 targets de P12 (ADR F1-14, punto 3). En Linux, musl estatico
/// (decision de la persona en TASK-025): el binario no depende de la glibc
/// del sistema.
pub const TARGETS: [Target; 6] = [
    Target {
        alias: "win-x64",
        triple: "x86_64-pc-windows-msvc",
    },
    Target {
        alias: "win-arm64",
        triple: "aarch64-pc-windows-msvc",
    },
    Target {
        alias: "darwin-x64",
        triple: "x86_64-apple-darwin",
    },
    Target {
        alias: "darwin-arm64",
        triple: "aarch64-apple-darwin",
    },
    Target {
        alias: "linux-x64",
        triple: "x86_64-unknown-linux-musl",
    },
    Target {
        alias: "linux-arm64",
        triple: "aarch64-unknown-linux-musl",
    },
];

/// Nombre del fichero de suma de un solo binario: `<fichero>.sha256`.
pub const SUFIJO_SUMA: &str = ".sha256";
/// Fichero con las sumas de todos los binarios de la release.
pub const SUMAS: &str = "SHA256SUMS";

impl Target {
    /// El target de un triple de P12.
    pub fn de_triple(triple: &str) -> Option<Target> {
        TARGETS.iter().copied().find(|t| t.triple == triple)
    }

    /// El target de un alias de P12.
    pub fn de_alias(alias: &str) -> Option<Target> {
        TARGETS.iter().copied().find(|t| t.alias == alias)
    }

    /// El target para el que se compilo este binario, por sistema y
    /// arquitectura (un build local en Linux con glibc cuenta como el alias
    /// de Linux de su arquitectura). `None` fuera de P12.
    pub fn actual() -> Option<Target> {
        Self::de_sistema(std::env::consts::OS, std::env::consts::ARCH)
    }

    /// El target de un sistema y una arquitectura con los nombres de
    /// `std::env::consts`.
    pub fn de_sistema(os: &str, arch: &str) -> Option<Target> {
        let alias = match (os, arch) {
            ("windows", "x86_64") => "win-x64",
            ("windows", "aarch64") => "win-arm64",
            ("macos", "x86_64") => "darwin-x64",
            ("macos", "aarch64") => "darwin-arm64",
            ("linux", "x86_64") => "linux-x64",
            ("linux", "aarch64") => "linux-arm64",
            _ => return None,
        };
        Self::de_alias(alias)
    }

    /// Si es un target de Windows.
    pub fn es_windows(self) -> bool {
        self.alias.starts_with("win-")
    }

    /// Si es un target de macOS.
    pub fn es_macos(self) -> bool {
        self.alias.starts_with("darwin-")
    }

    /// Binario que deja `cargo build --target`.
    pub fn binario_cargo(self) -> &'static str {
        if self.es_windows() { "tc2.exe" } else { "tc2" }
    }

    /// Nombre publicado (ADR F1-14, punto 4): `tc2-<version>-<alias>[.exe]`.
    pub fn fichero(self, version: &str) -> String {
        format!("tc2-{version}-{}{}", self.alias, self.extension())
    }

    /// La version de un nombre publicado de este target, si lo es:
    /// `tc2-2.0.0-win-x64.exe` → `2.0.0`. La version no puede estar vacia ni
    /// llevar separadores de ruta ni espacios.
    pub fn version_de_fichero(self, fichero: &str) -> Option<String> {
        let sufijo = format!("-{}{}", self.alias, self.extension());
        let version = fichero.strip_prefix("tc2-")?.strip_suffix(&sufijo)?;
        version_valida(version).then(|| version.to_owned())
    }

    fn extension(self) -> &'static str {
        if self.es_windows() { ".exe" } else { "" }
    }
}

/// Una version con la forma de las publicadas: `1.2.3` o `1.2.3-sufijo`,
/// con el sufijo de alfanumericos, `.` y `-`.
pub fn version_valida(v: &str) -> bool {
    let (nucleo, sufijo) = match v.split_once('-') {
        Some((n, s)) => (n, Some(s)),
        None => (v, None),
    };
    let partes: Vec<&str> = nucleo.split('.').collect();
    partes.len() == 3
        && partes
            .iter()
            .all(|p| !p.is_empty() && p.len() <= 9 && p.bytes().all(|b| b.is_ascii_digit()))
        && sufijo.is_none_or(|s| {
            !s.is_empty()
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
        })
}

/// Lee un `SHA256SUMS` con el formato de [`sumas`] (tambien el modo binario
/// de `sha256sum`, `<hex> *<fichero>`, y finales CRLF). Devuelve los pares
/// (fichero, hash) en el orden del texto. Falla con la linea si una no tiene
/// el formato o si un fichero se repite.
pub fn leer_sumas(texto: &str) -> Result<Vec<(String, String)>, String> {
    let mut pares: Vec<(String, String)> = Vec::new();
    for (n, linea) in texto.lines().enumerate() {
        let linea = linea.strip_suffix('\r').unwrap_or(linea);
        if linea.is_empty() {
            continue;
        }
        let formato = || {
            format!(
                "linea {} de {SUMAS} sin el formato `<sha256>  <fichero>`",
                n + 1
            )
        };
        let (hash, resto) = linea.split_once(' ').ok_or_else(formato)?;
        let fichero = resto
            .strip_prefix(' ')
            .or_else(|| resto.strip_prefix('*'))
            .ok_or_else(formato)?;
        if !hash_valido(hash) || fichero.is_empty() || fichero.contains(['/', '\\']) {
            return Err(formato());
        }
        if pares.iter().any(|(f, _)| f == fichero) {
            return Err(format!("fichero repetido en {SUMAS}: {fichero}"));
        }
        pares.push((fichero.to_owned(), hash.to_owned()));
    }
    Ok(pares)
}

/// 64 hex en minusculas.
pub fn hash_valido(hash: &str) -> bool {
    hash.len() == 64 && hash.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Una linea con el formato de `sha256sum` (modo texto): `<hex>  <fichero>`.
pub fn linea_suma(hash: &str, fichero: &str) -> String {
    format!("{hash}  {fichero}\n")
}

/// `SHA256SUMS` a partir de pares (fichero, hash), ordenado por fichero para
/// que el resultado no dependa del orden en que se listan los ficheros. Falla
/// si un hash no son 64 hex en minusculas, o si un fichero se repite o trae
/// un separador o un salto de linea (romperia el formato).
pub fn sumas(entradas: &[(String, String)]) -> Result<String, String> {
    let mut ordenadas: Vec<&(String, String)> = entradas.iter().collect();
    ordenadas.sort_by(|a, b| a.0.cmp(&b.0));
    let mut texto = String::new();
    let mut anterior: Option<&str> = None;
    for (fichero, hash) in ordenadas {
        if !hash_valido(hash) {
            return Err(format!("hash no valido para {fichero}: «{hash}»"));
        }
        if fichero.is_empty() || fichero.contains(['/', '\\', '\n', '\r']) {
            return Err(format!("nombre de fichero no valido: «{fichero}»"));
        }
        if anterior == Some(fichero.as_str()) {
            return Err(format!("fichero repetido: {fichero}"));
        }
        anterior = Some(fichero);
        texto.push_str(&linea_suma(hash, fichero));
    }
    if texto.is_empty() {
        return Err("no hay ficheros que sumar".to_owned());
    }
    Ok(texto)
}

/// Clase de la release segun su tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clase {
    Estable,
    Prerelease,
}

/// El tag tiene que ser `v<version>` (estable) o `v<version>-<sufijo>`
/// (prerelease), con `<version>` la del workspace: asi el `tc2 --version` del
/// binario publicado coincide con el tag.
pub fn clase_de_tag(tag: &str, version: &str) -> Result<Clase, String> {
    let resto = tag
        .strip_prefix('v')
        .and_then(|r| r.strip_prefix(version))
        .ok_or_else(|| format!("el tag «{tag}» no es v{version} ni v{version}-<sufijo>"))?;
    if resto.is_empty() {
        return Ok(Clase::Estable);
    }
    match resto.strip_prefix('-') {
        Some(sufijo)
            if !sufijo.is_empty()
                && sufijo
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-') =>
        {
            Ok(Clase::Prerelease)
        }
        _ => Err(format!(
            "el tag «{tag}» no es v{version} ni v{version}-<sufijo>"
        )),
    }
}

/// `cli.lock` del plugin (ADR F1-14, punto 4) para la release `tag` de
/// `version`, desde su `SHA256SUMS`: los 6 targets, cada uno con su fichero y
/// su sha256, y el rango de protocolo `[PROTOCOLO, PROTOCOLO]`. Falla si el
/// tag no casa con la version, si `SHA256SUMS` no se lee o si le falta un
/// target. Sale ordenado por la tabla de targets, asi que no depende del
/// orden de `SHA256SUMS`.
pub fn cli_lock(tag: &str, version: &str, texto_sumas: &str) -> Result<String, String> {
    clase_de_tag(tag, version)?;
    let pares = leer_sumas(texto_sumas)?;
    let mut s = format!(
        "# cli.lock de TaskCodeTwo (ADR F1-14, punto 4). Lo genera `cargo xtask cli-lock`\n\
         # desde el SHA256SUMS de la release {tag}; no se edita a mano.\n\
         formato: 1\n\
         version: \"{version}\"\n\
         protocolo: [{PROTOCOLO}, {PROTOCOLO}]\n\
         base_url: \"{RELEASES}/download/{tag}/\"\n\
         targets:\n"
    );
    for t in TARGETS {
        let fichero = t.fichero(version);
        let hash = pares
            .iter()
            .find(|(f, _)| *f == fichero)
            .map(|(_, h)| h)
            .ok_or_else(|| format!("{SUMAS} no trae {fichero}"))?;
        s.push_str(&format!(
            "  {}: {{fichero: \"{fichero}\", sha256: \"{hash}\"}}\n",
            t.alias
        ));
    }
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_lock_con_los_6_targets_desde_sha256sums() {
        let entradas: Vec<(String, String)> = TARGETS
            .iter()
            .enumerate()
            .map(|(i, t)| (t.fichero("2.0.0"), format!("{i}").repeat(64)))
            .collect();
        let sumas_texto = sumas(&entradas).unwrap_or_default();
        let lock = cli_lock("v2.0.0", "2.0.0", &sumas_texto).unwrap_or_default();
        assert!(lock.contains("\nformato: 1\nversion: \"2.0.0\"\n"));
        assert!(lock.contains(&format!("\nprotocolo: [{PROTOCOLO}, {PROTOCOLO}]\n")));
        assert!(lock.contains(
            "\nbase_url: \"https://github.com/charliebk/TaskCodeTwo/releases/download/v2.0.0/\"\n"
        ));
        assert!(lock.contains(&format!(
            "\n  win-x64: {{fichero: \"tc2-2.0.0-win-x64.exe\", sha256: \"{}\"}}\n",
            "0".repeat(64)
        )));
        assert_eq!(lock.lines().filter(|l| l.starts_with("  ")).count(), 6);
        // Prerelease: el tag lleva el sufijo y los ficheros, la version.
        assert!(
            cli_lock("v2.0.0-rc.1", "2.0.0", &sumas_texto)
                .is_ok_and(|l| l.contains("/download/v2.0.0-rc.1/\""))
        );
        // Falta un target, o el tag no casa.
        let cinco = sumas(&entradas[..5]).unwrap_or_default();
        assert!(cli_lock("v2.0.0", "2.0.0", &cinco).is_err());
        assert!(cli_lock("v2.0.1", "2.0.0", &sumas_texto).is_err());
    }

    #[test]
    fn seis_targets_de_p12_con_exe_solo_en_windows() {
        let alias: Vec<&str> = TARGETS.iter().map(|t| t.alias).collect();
        assert_eq!(
            alias,
            [
                "win-x64",
                "win-arm64",
                "darwin-x64",
                "darwin-arm64",
                "linux-x64",
                "linux-arm64"
            ]
        );
        for t in TARGETS {
            assert_eq!(Target::de_triple(t.triple), Some(t));
            let f = t.fichero("2.0.0");
            assert!(f.starts_with(&format!("tc2-2.0.0-{}", t.alias)));
            assert_eq!(f.ends_with(".exe"), t.es_windows(), "{f}");
            assert_eq!(t.binario_cargo().ends_with(".exe"), t.es_windows());
        }
        // Los triples, literales: son los que pide rustup en la CI.
        let triples: Vec<&str> = TARGETS.iter().map(|t| t.triple).collect();
        assert_eq!(
            triples,
            [
                "x86_64-pc-windows-msvc",
                "aarch64-pc-windows-msvc",
                "x86_64-apple-darwin",
                "aarch64-apple-darwin",
                "x86_64-unknown-linux-musl",
                "aarch64-unknown-linux-musl"
            ]
        );
        let macos: Vec<&str> = TARGETS
            .iter()
            .filter(|t| t.es_macos())
            .map(|t| t.alias)
            .collect();
        assert_eq!(macos, ["darwin-x64", "darwin-arm64"]);
        assert_eq!(TARGETS[0].fichero("2.0.0"), "tc2-2.0.0-win-x64.exe");
        assert_eq!(TARGETS[4].fichero("2.0.0"), "tc2-2.0.0-linux-x64");
        assert_eq!(Target::de_triple("x86_64-unknown-linux-gnu"), None);
    }

    #[test]
    fn sumas_con_formato_de_sha256sum_y_orden_estable() {
        let vacio = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let abc = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        let a = (String::from("tc2-0.0.0-win-x64.exe"), abc.to_owned());
        let b = (String::from("tc2-0.0.0-linux-x64"), vacio.to_owned());
        let esperado = format!("{vacio}  tc2-0.0.0-linux-x64\n{abc}  tc2-0.0.0-win-x64.exe\n");
        assert_eq!(sumas(&[a.clone(), b.clone()]), Ok(esperado.clone()));
        assert_eq!(sumas(&[b.clone(), a.clone()]), Ok(esperado));
        assert!(sumas(&[a.clone(), a.clone()]).is_err());
        assert!(sumas(&[]).is_err());
        assert!(sumas(&[(String::from("x"), abc.to_uppercase())]).is_err());
        assert!(sumas(&[(String::from("x"), String::from("ab"))]).is_err());
        // 64 caracteres, pero con una «g» que no es hex.
        let no_hex = format!("{}g", &abc[..63]);
        assert!(sumas(&[(String::from("x"), no_hex)]).is_err());
        for malo in ["", "dir/x", "dir\\x", "x\ny", "x\ry"] {
            assert!(
                sumas(&[(malo.to_owned(), abc.to_owned())]).is_err(),
                "{malo:?}"
            );
        }
    }

    #[test]
    fn alias_sistema_y_version_de_fichero() {
        for t in TARGETS {
            assert_eq!(Target::de_alias(t.alias), Some(t));
            assert_eq!(
                t.version_de_fichero(&t.fichero("2.0.0-rc.1")).as_deref(),
                Some("2.0.0-rc.1")
            );
        }
        assert_eq!(Target::de_sistema("windows", "x86_64"), Some(TARGETS[0]));
        assert_eq!(Target::de_sistema("macos", "aarch64"), Some(TARGETS[3]));
        assert_eq!(Target::de_sistema("linux", "aarch64"), Some(TARGETS[5]));
        assert_eq!(Target::de_sistema("freebsd", "x86_64"), None);
        assert!(Target::actual().is_some());
        let win = TARGETS[0];
        for malo in [
            "tc2-2.0.0-linux-x64",
            "tc2-2.0.0-win-x64",
            "tc2--win-x64.exe",
            "tc2-2.0-win-x64.exe",
            "tc2-../x-win-x64.exe",
            "tc2-2.0.0-a b-win-x64.exe",
            "x-2.0.0-win-x64.exe",
        ] {
            assert_eq!(win.version_de_fichero(malo), None, "{malo}");
        }
        assert!(version_valida("10.20.30"));
        assert!(!version_valida("1.2.3-"));
        assert!(!version_valida("1.2.x"));
    }

    #[test]
    fn leer_sumas_es_la_inversa_de_sumas() {
        let abc = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        let vacio = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let entradas = vec![
            (String::from("tc2-0.0.0-linux-x64"), vacio.to_owned()),
            (String::from("tc2-0.0.0-win-x64.exe"), abc.to_owned()),
        ];
        let texto = sumas(&entradas).unwrap_or_default();
        assert_eq!(leer_sumas(&texto), Ok(entradas.clone()));
        // CRLF y modo binario de sha256sum.
        let crlf = format!("{vacio} *tc2-0.0.0-linux-x64\r\n{abc}  tc2-0.0.0-win-x64.exe\r\n");
        assert_eq!(leer_sumas(&crlf), Ok(entradas));
        for malo in [
            format!("{abc} x"),
            format!("{abc}  dir/x"),
            format!("{}  x", abc.to_uppercase()),
            format!("{abc}  x\n{vacio}  x"),
            String::from("nada"),
        ] {
            assert!(leer_sumas(&malo).is_err(), "{malo}");
        }
    }

    #[test]
    fn tag_y_version_tienen_que_casar() {
        assert_eq!(clase_de_tag("v0.0.0", "0.0.0"), Ok(Clase::Estable));
        assert_eq!(
            clase_de_tag("v0.0.0-prueba.1", "0.0.0"),
            Ok(Clase::Prerelease)
        );
        for malo in [
            "v0.0.1",
            "0.0.0",
            "v0.0.0-",
            "v0.0.00",
            "v0.0.0.1",
            "v0.0.0-a b",
            "v0.0.0-a/b",
        ] {
            assert!(clase_de_tag(malo, "0.0.0").is_err(), "{malo}");
        }
    }
}
