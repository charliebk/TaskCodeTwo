//! PoC de F1-14 (TASK-014): comando `tablero <raiz-del-repo> [--sin-red]`.
//! Cumple `../especificacion-comando.md`.

use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use serde::Deserialize;

#[derive(Deserialize)]
struct Frontmatter {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    titulo: String,
    estado: Option<String>,
}

enum Resultado {
    Valida(String),
    Error(&'static str),
}

fn main() -> ExitCode {
    match ejecutar() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("tablero: {e}");
            ExitCode::from(1)
        }
    }
}

fn ejecutar() -> Result<(), String> {
    let mut raiz: Option<PathBuf> = None;
    let mut sin_red = false;
    for arg in std::env::args().skip(1) {
        if arg == "--sin-red" {
            sin_red = true;
        } else if raiz.is_none() {
            raiz = Some(PathBuf::from(arg));
        } else {
            return Err(format!("argumento inesperado: {arg}"));
        }
    }
    let raiz = raiz.ok_or("uso: tablero <raiz-del-repo> [--sin-red]")?;

    // Paso 1: recorrer tareas/**/tarea.md y ordenar por ruta relativa (bytes UTF-8).
    let mut rutas: Vec<String> = Vec::new();
    let dir_tareas = raiz.join("tareas");
    if dir_tareas.is_dir() {
        recorrer(&dir_tareas, "tareas", &mut rutas)?;
    }
    rutas.sort_unstable_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

    // Pasos 2 y 3.
    let mut validas = 0usize;
    let mut por_estado: BTreeMap<String, usize> = BTreeMap::new();
    let mut errores: Vec<(String, &'static str)> = Vec::new();
    for rel in &rutas {
        let texto = fs::read(raiz.join(rel)).map_err(|e| format!("no se pudo leer {rel}: {e}"))?;
        match analizar(&String::from_utf8_lossy(&texto)) {
            Resultado::Valida(estado) => {
                validas += 1;
                *por_estado.entry(estado).or_insert(0) += 1;
            }
            Resultado::Error(motivo) => errores.push((rel.clone(), motivo)),
        }
    }

    // Paso 4: git como proceso hijo.
    let salida_git = Command::new("git")
        .arg("-C")
        .arg(&raiz)
        .args(["log", "-1", "--format=%H"])
        .output()
        .map_err(|e| format!("no se pudo ejecutar git: {e}"))?;
    if !salida_git.status.success() {
        return Err(format!("git log fallo: {}", String::from_utf8_lossy(&salida_git.stderr).trim()));
    }
    let hash = String::from_utf8_lossy(&salida_git.stdout).lines().next().unwrap_or("").trim().to_string();

    // Paso 5: red.
    let red = if sin_red { "omitida" } else if consultar_red() { "si" } else { "no" };

    // Paso 6: salida (BTreeMap<String> ordena por bytes).
    let mut s = String::new();
    s.push_str(&format!("tareas: {validas}\nerrores: {}\n", errores.len()));
    for (estado, n) in &por_estado {
        s.push_str(&format!("{estado}: {n}\n"));
    }
    s.push_str(&format!("git: {hash}\nred: {red}\n"));
    for (rel, motivo) in &errores {
        s.push_str(&format!("error {rel}: {motivo}\n"));
    }
    let mut out = io::stdout().lock();
    out.write_all(s.as_bytes()).and_then(|_| out.flush()).map_err(|e| format!("no se pudo escribir stdout: {e}"))
}

/// Recorre `dir` y acumula rutas relativas con `/` de cada `tarea.md`.
fn recorrer(dir: &Path, rel: &str, rutas: &mut Vec<String>) -> Result<(), String> {
    let entradas = fs::read_dir(dir).map_err(|e| format!("no se pudo leer {rel}: {e}"))?;
    for entrada in entradas {
        let entrada = entrada.map_err(|e| format!("no se pudo leer {rel}: {e}"))?;
        let nombre = entrada.file_name().to_string_lossy().into_owned();
        let rel_hijo = format!("{rel}/{nombre}");
        let tipo = entrada.file_type().map_err(|e| format!("no se pudo leer {rel_hijo}: {e}"))?;
        if tipo.is_dir() {
            recorrer(&entrada.path(), &rel_hijo, rutas)?;
        } else if tipo.is_file() && nombre == "tarea.md" {
            rutas.push(rel_hijo);
        }
    }
    Ok(())
}

/// Extrae el frontmatter (primera linea `---` hasta la siguiente `---`) y lo valida.
fn analizar(texto: &str) -> Resultado {
    let texto = texto.strip_prefix('\u{feff}').unwrap_or(texto);
    let mut lineas = texto.split('\n').map(|l| l.strip_suffix('\r').unwrap_or(l));
    if lineas.next() != Some("---") {
        return Resultado::Error("sin frontmatter");
    }
    let mut yaml = Vec::new();
    let mut cerrado = false;
    for l in lineas {
        if l == "---" {
            cerrado = true;
            break;
        }
        yaml.push(l);
    }
    if !cerrado {
        return Resultado::Error("sin frontmatter");
    }
    let yaml = yaml.join("\n");
    // Politica por defecto de serde-saphyr: DuplicateKeyPolicy::Error (se fija explicita).
    let opciones = serde_saphyr::options! { duplicate_keys: serde_saphyr::DuplicateKeyPolicy::Error };
    match serde_saphyr::from_str_with_options::<Frontmatter>(&yaml, opciones) {
        Ok(fm) => Resultado::Valida(fm.estado.unwrap_or_else(|| "desconocido".to_string())),
        Err(e) if es_duplicada(&e) => Resultado::Error("clave duplicada"),
        Err(_) => Resultado::Error("yaml invalido"),
    }
}

fn es_duplicada(e: &serde_saphyr::Error) -> bool {
    let mut actual = e;
    loop {
        match actual {
            serde_saphyr::Error::WithSnippet { error, .. } | serde_saphyr::Error::AliasError { error, .. } => {
                actual = error
            }
            serde_saphyr::Error::DuplicateMappingKey { .. } => return true,
            _ => return false,
        }
    }
}

/// GET anonimo a https://api.github.com/zen; true solo si 2xx (cualquier fallo cuenta como `no`).
fn consultar_red() -> bool {
    let agente: ureq::Agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        // Raices del sistema operativo (almacen de Windows), no las de Mozilla embebidas:
        // asi se respetan las CA corporativas o de inspeccion TLS que instale el equipo.
        .tls_config(ureq::tls::TlsConfig::builder().root_certs(ureq::tls::RootCerts::PlatformVerifier).build())
        .timeout_global(Some(std::time::Duration::from_secs(10)))
        .build()
        .into();
    match agente.get("https://api.github.com/zen").header("User-Agent", "taskcodetwo-poc").call() {
        Ok(r) => r.status().is_success(),
        Err(_) => false,
    }
}
