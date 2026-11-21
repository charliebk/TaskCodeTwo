//! Cierre de una orden (F1-8 decision 5): los ficheros **versionados** que
//! se ejecutan como script y que la orden nombra y, a su vez, los que nombran
//! estos (`%~dp0` resuelto), mas los de los globs de `confianza_incluye`.
//!
//! Solo se siguen tokens con forma de ruta (llevan `.` o `/`), como el
//! prototipo de la PoC A1 de F1-8 (`tareas/04-terminadas/TASK-009/poc/
//! poc-f1-8.sh`): lo que se carga por nombre de modulo no entra, y se declara
//! en `confianza_incluye`.
//!
//! La cache de D2a: un fichero cuyo `(tamano, mtime)` coincide con el de la
//! entrada confiada no se vuelve a leer; se reutilizan su `sha` y sus
//! candidatos. Los candidatos son rutas normalizadas, no resueltas: un
//! fichero versionado despues de confiar entra en el cierre aunque el script
//! que lo nombra no haya cambiado.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::time::UNIX_EPOCH;

use globset::{GlobBuilder, GlobSetBuilder};
use serde::{Deserialize, Serialize};

/// Extensiones de lo que se ejecuta como script (`""`: sin extension).
const SCRIPT: [&str; 11] = [
    "bat", "cmd", "sh", "ps1", "psm1", "py", "js", "mjs", "cjs", "ts", "",
];

/// Por encima de este tamano un fichero se hashea pero no se buscan rutas en
/// el (como la PoC).
const MAX_ESCANEO: u64 = 2_000_000;

/// Un fichero del cierre, como se guarda en el almacen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fichero {
    /// Ruta relativa a la raiz del arbol, con `/`, como la da `git ls-files`.
    pub ruta: String,
    /// Bytes.
    pub tamano: u64,
    /// `mtime` en nanosegundos desde la epoca Unix.
    pub mtime: u64,
    /// sha256 del contenido, en hexadecimal.
    pub sha: String,
    /// Rutas normalizadas que nombra (sin resolver).
    pub candidatos: Vec<String>,
}

/// Ficheros versionados del arbol, con la clave de busqueda del sistema: sin
/// distinguir mayusculas en Windows y macOS, donde `Scripts\V.bat` ejecuta
/// `scripts/v.bat`.
pub struct Indice {
    rutas: HashMap<String, String>,
    todas: Vec<String>,
}

fn clave_busqueda(ruta: &str) -> String {
    if cfg!(any(windows, target_os = "macos")) {
        ruta.to_lowercase()
    } else {
        ruta.to_owned()
    }
}

impl Indice {
    /// Indice de la salida de `git ls-files -z`.
    pub fn de_ls_files(salida: &str) -> Self {
        let todas: Vec<String> = salida
            .split('\0')
            .filter(|r| !r.is_empty())
            .map(str::to_owned)
            .collect();
        let rutas = todas
            .iter()
            .map(|r| (clave_busqueda(r), r.clone()))
            .collect();
        Self { rutas, todas }
    }

    /// La ruta versionada que es `candidato`, si se ejecuta como script.
    fn script(&self, candidato: &str) -> Option<&str> {
        let real = self.rutas.get(&clave_busqueda(candidato))?;
        es_script(real).then_some(real.as_str())
    }
}

fn es_script(ruta: &str) -> bool {
    let nombre = ruta.rsplit('/').next().unwrap_or(ruta);
    let ext = match nombre.rfind('.') {
        Some(i) if i > 0 => nombre[i + 1..].to_ascii_lowercase(),
        _ => String::new(),
    };
    SCRIPT.contains(&ext.as_str())
}

/// Une `base` y `rel` y quita `.` y `..`. `None` si sale del arbol o es
/// absoluta.
fn normalizar(base: &str, rel: &str) -> Option<String> {
    if rel.starts_with('/') {
        return None;
    }
    let mut partes: Vec<&str> = Vec::new();
    for p in base.split('/').chain(rel.split('/')) {
        match p {
            "" | "." => {}
            ".." => {
                partes.pop()?;
            }
            otro => partes.push(otro),
        }
    }
    (!partes.is_empty()).then(|| partes.join("/"))
}

fn es_de_token(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '%' | '~' | '-' | '\\' | '/')
}

/// Rutas normalizadas que nombra `texto`, escrito en la carpeta `dir` (vacia
/// en la raiz). Cada token con forma de ruta da hasta dos: relativo a `dir`
/// y relativo a la raiz.
pub fn candidatos(texto: &str, dir: &str) -> Vec<String> {
    let dp0 = if dir.is_empty() {
        String::new()
    } else {
        format!("{dir}/")
    };
    let mut v: BTreeSet<String> = BTreeSet::new();
    for tok in texto.split(|c: char| !es_de_token(c)) {
        if tok.is_empty() {
            continue;
        }
        let t = tok.replace("%~dp0", &dp0).replace('\\', "/");
        if !t.contains(['.', '/']) || t == "." || t == ".." {
            continue;
        }
        let bases: &[&str] = if dir.is_empty() { &[""] } else { &[dir, ""] };
        for b in bases {
            if let Some(n) = normalizar(b, &t) {
                v.insert(n);
            }
        }
    }
    v.into_iter().collect()
}

fn mtime_ns(m: &std::fs::Metadata) -> u64 {
    m.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX))
}

/// sha256 en hexadecimal.
pub fn sha_hex(datos: &[u8]) -> String {
    use std::fmt::Write;
    let d = ring::digest::digest(&ring::digest::SHA256, datos);
    let mut s = String::with_capacity(64);
    for b in d.as_ref() {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Resultado de calcular un cierre.
pub struct Cierre {
    /// Ficheros, por ruta.
    pub ficheros: Vec<Fichero>,
    /// Bytes leidos para hashear (0 si todo salio de la cache).
    pub hasheados: u64,
}

/// Calcula el cierre de `orden` en `raiz`. Con `cache`, un fichero con el
/// mismo `(tamano, mtime)` que su entrada no se vuelve a leer.
pub fn calcular(
    raiz: &Path,
    indice: &Indice,
    orden: &str,
    incluye: &[String],
    cache: Option<&[Fichero]>,
) -> std::io::Result<Cierre> {
    let cache: HashMap<&str, &Fichero> = cache
        .unwrap_or_default()
        .iter()
        .map(|f| (f.ruta.as_str(), f))
        .collect();
    let mut pendientes: Vec<String> = candidatos(orden, "")
        .iter()
        .filter_map(|c| indice.script(c).map(str::to_owned))
        .collect();
    let mut globs = GlobSetBuilder::new();
    for g in incluye {
        if let Ok(g) = GlobBuilder::new(g).literal_separator(true).build() {
            globs.add(g);
        }
    }
    if let Ok(set) = globs.build()
        && !incluye.is_empty()
    {
        pendientes.extend(indice.todas.iter().filter(|r| set.is_match(r)).cloned());
    }
    let mut vistos: BTreeMap<String, Fichero> = BTreeMap::new();
    let mut hasheados = 0u64;
    while let Some(rel) = pendientes.pop() {
        if vistos.contains_key(&rel) {
            continue;
        }
        let ruta = raiz.join(&rel);
        // Versionado pero borrado del arbol: no se ejecuta; que falte cambia
        // la huella.
        let Ok(meta) = std::fs::metadata(&ruta) else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        let (tamano, mtime) = (meta.len(), mtime_ns(&meta));
        let f = match cache.get(rel.as_str()) {
            Some(c) if c.tamano == tamano && c.mtime == mtime => (*c).clone(),
            _ => {
                let datos = std::fs::read(&ruta)?;
                hasheados += datos.len() as u64;
                let dir = rel.rsplit_once('/').map_or("", |(d, _)| d);
                let candidatos = if es_script(&rel) && tamano <= MAX_ESCANEO {
                    candidatos(&String::from_utf8_lossy(&datos), dir)
                } else {
                    Vec::new()
                };
                Fichero {
                    ruta: rel.clone(),
                    tamano,
                    mtime,
                    sha: sha_hex(&datos),
                    candidatos,
                }
            }
        };
        for c in &f.candidatos {
            if let Some(r) = indice.script(c)
                && !vistos.contains_key(r)
            {
                pendientes.push(r.to_owned());
            }
        }
        vistos.insert(rel, f);
    }
    Ok(Cierre {
        ficheros: vistos.into_values().collect(),
        hasheados,
    })
}

/// Huella de la orden, de los globs y del contenido del cierre.
pub fn huella(orden: &str, incluye: &[String], ficheros: &[Fichero]) -> String {
    let mut globs: Vec<&str> = incluye.iter().map(String::as_str).collect();
    globs.sort_unstable();
    let mut s = format!("orden\0{orden}\0incluye\0{}\0", globs.join("\0"));
    for f in ficheros {
        s.push_str(&f.ruta);
        s.push('\0');
        s.push_str(&f.sha);
        s.push('\n');
    }
    sha_hex(s.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidatos_resuelven_dp0_barras_y_puntos() {
        let c = candidatos(
            r#"call "%~dp0lib\paso.bat" && python -I ../x.py"#,
            "scripts",
        );
        assert!(c.contains(&"scripts/lib/paso.bat".to_owned()), "{c:?}");
        assert!(c.contains(&"x.py".to_owned()), "{c:?}");
        // Tokens sin forma de ruta no cuentan.
        assert!(!c.iter().any(|x| x == "call" || x == "python"), "{c:?}");
        assert_eq!(normalizar("", "../fuera.sh"), None);
        assert_eq!(normalizar("", "/abs.sh"), None);
    }

    #[test]
    fn script_por_extension() {
        assert!(es_script("scripts/v.BAT"));
        assert!(es_script("scripts/verificar"));
        assert!(!es_script("README.md"));
    }
}
