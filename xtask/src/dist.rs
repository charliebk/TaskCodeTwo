//! Distribucion (F2-10, ADR F1-14): los 6 targets de P12, el nombre de cada
//! binario publicado, el formato de `SHA256SUMS` y la regla tag↔version.
//!
//! Es logica pura, sin E/S, para probarla en local: el empaquetado y la
//! release de `.github/workflows/release.yml` llaman a `cargo xtask`, que
//! usa este modulo.

/// Un target de P12: el alias publico (contrato de `cli.lock`) y el triple de
/// Rust con el que se compila.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target {
    pub alias: &'static str,
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
    pub fn de_triple(triple: &str) -> Option<Target> {
        TARGETS.iter().copied().find(|t| t.triple == triple)
    }

    pub fn es_windows(self) -> bool {
        self.alias.starts_with("win-")
    }

    pub fn es_macos(self) -> bool {
        self.alias.starts_with("darwin-")
    }

    /// Binario que deja `cargo build --target`.
    pub fn binario_cargo(self) -> &'static str {
        if self.es_windows() { "tc2.exe" } else { "tc2" }
    }

    /// Nombre publicado (ADR F1-14, punto 4): `tc2-<version>-<alias>[.exe]`.
    pub fn fichero(self, version: &str) -> String {
        let ext = if self.es_windows() { ".exe" } else { "" };
        format!("tc2-{version}-{}{ext}", self.alias)
    }
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
        if hash.len() != 64 || !hash.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
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

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(TARGETS[0].fichero("2.0.0"), "tc2-2.0.0-win-x64.exe");
        assert_eq!(TARGETS[4].fichero("2.0.0"), "tc2-2.0.0-linux-x64");
        assert!(TARGETS[4].triple.ends_with("-musl"));
        assert!(TARGETS[5].triple.ends_with("-musl"));
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
        assert!(sumas(&[(String::from("dir/x"), abc.to_owned())]).is_err());
        assert!(sumas(&[(String::from("x\ny"), abc.to_owned())]).is_err());
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
