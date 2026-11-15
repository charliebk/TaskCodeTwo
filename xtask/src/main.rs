//! `cargo xtask verificar --perfil rapida|completa`: el unico punto de entrada
//! de verificacion del workspace, el mismo en local y en la CI (F2-1).
//!
//! - `rapida`: `cargo fmt --check`, `clippy -D warnings` y `cargo test`, todo
//!   con `--locked`.
//! - `completa`: lo anterior y la comprobacion de build reproducible: copia el
//!   arbol a dos rutas distintas, compila `tc2` en release en limpio en cada
//!   una y exige el mismo sha256 (ADR F1-14). La CI solo la corre en Windows.
//!
//! El resultado va en el codigo de salida: 0 verde, 1 rojo, 2 mal invocado.

mod sha256;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Perfil {
    Rapida,
    Completa,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let perfil = match leer_perfil(&args) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("xtask: {e}");
            eprintln!("Uso: cargo xtask verificar --perfil rapida|completa");
            return ExitCode::from(2);
        }
    };
    match verificar(perfil) {
        Ok(()) => {
            println!("xtask: verificacion {perfil:?} en verde");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("xtask: ROJO: {e}");
            ExitCode::from(1)
        }
    }
}

fn leer_perfil(args: &[String]) -> Result<Perfil, String> {
    match args {
        [orden, flag, valor] if orden == "verificar" && flag == "--perfil" => {
            match valor.as_str() {
                "rapida" => Ok(Perfil::Rapida),
                "completa" => Ok(Perfil::Completa),
                otro => Err(format!("perfil desconocido «{otro}»")),
            }
        }
        _ => Err("argumentos no validos".to_owned()),
    }
}

fn verificar(perfil: Perfil) -> Result<(), String> {
    for var in ["RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"] {
        if std::env::var_os(var).is_some() {
            return Err(format!(
                "{var} esta definido y anularia los rustflags de .cargo/config.toml (/Brepro); quitalo"
            ));
        }
    }
    let raiz = raiz_workspace()?;
    cargo(&raiz, &["fmt", "--all", "--check"], None)?;
    cargo(
        &raiz,
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--locked",
            "--",
            "-D",
            "warnings",
        ],
        None,
    )?;
    cargo(&raiz, &["test", "--workspace", "--locked"], None)?;
    if perfil == Perfil::Completa {
        reproducible(&raiz)?;
    }
    Ok(())
}

fn raiz_workspace() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "no se encuentra la raiz del workspace".to_owned())
}

/// Lanza cargo en `dir`; con `target`, fija su `CARGO_TARGET_DIR`.
fn cargo(dir: &Path, args: &[&str], target: Option<&Path>) -> Result<(), String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    println!("xtask: cargo {}", args.join(" "));
    let mut cmd = Command::new(&cargo);
    cmd.args(args).current_dir(dir);
    if let Some(t) = target {
        cmd.env("CARGO_TARGET_DIR", t);
    }
    let estado = cmd
        .status()
        .map_err(|e| format!("no se pudo lanzar {cargo}: {e}"))?;
    if estado.success() {
        Ok(())
    } else {
        Err(format!("falla `cargo {}` ({estado})", args.join(" ")))
    }
}

/// Dos builds en limpio, en dos rutas distintas, con el mismo sha256.
fn reproducible(raiz: &Path) -> Result<(), String> {
    let base = std::env::temp_dir().join(format!("tc2-reproducible-{}", std::process::id()));
    let resultado = reproducible_en(raiz, &base);
    let _ = std::fs::remove_dir_all(&base);
    resultado
}

fn reproducible_en(raiz: &Path, base: &Path) -> Result<(), String> {
    let ficheros = ficheros_del_arbol(raiz)?;
    let mut hashes = Vec::new();
    // Rutas de distinta longitud a proposito: si el binario incrusta la ruta
    // del checkout, los hashes no coinciden.
    for nombre in ["a", "copia-b"] {
        let dir = base.join(nombre);
        copiar(raiz, &dir, &ficheros)?;
        let target = dir.join("target");
        cargo(
            &dir,
            &["build", "--release", "--locked", "-p", "tc2"],
            Some(&target),
        )?;
        let bin = target
            .join("release")
            .join(format!("tc2{}", std::env::consts::EXE_SUFFIX));
        let bytes =
            std::fs::read(&bin).map_err(|e| format!("no se puede leer {}: {e}", bin.display()))?;
        let hash = sha256::hex(&bytes);
        println!(
            "xtask: sha256 {hash}  {} ({} bytes)",
            bin.display(),
            bytes.len()
        );
        hashes.push(hash);
    }
    match hashes.as_slice() {
        [a, b] if a == b => {
            println!("xtask: build reproducible: mismo sha256 en las dos rutas");
            Ok(())
        }
        _ => Err(format!("build NO reproducible: {hashes:?}")),
    }
}

/// Ficheros versionados o nuevos no ignorados, como los ve git, salvo los que
/// no entran en el binario.
fn ficheros_del_arbol(raiz: &Path) -> Result<Vec<PathBuf>, String> {
    let salida = Command::new("git")
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .current_dir(raiz)
        .output()
        .map_err(|e| format!("no se pudo lanzar git: {e}"))?;
    if !salida.status.success() {
        return Err(format!("falla git ls-files ({})", salida.status));
    }
    let texto = String::from_utf8(salida.stdout)
        .map_err(|_| "git ls-files no devolvio UTF-8".to_owned())?;
    Ok(texto
        .split('\0')
        .filter(|s| !s.is_empty() && !s.starts_with("tareas/") && !s.starts_with("docs/"))
        .map(PathBuf::from)
        .collect())
}

fn copiar(raiz: &Path, destino: &Path, ficheros: &[PathBuf]) -> Result<(), String> {
    for f in ficheros {
        let origen = raiz.join(f);
        if !origen.is_file() {
            continue; // borrado en el arbol de trabajo
        }
        let dst = destino.join(f);
        if let Some(padre) = dst.parent() {
            std::fs::create_dir_all(padre).map_err(|e| format!("{}: {e}", padre.display()))?;
        }
        std::fs::copy(&origen, &dst).map_err(|e| format!("{}: {e}", origen.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn perfiles_validos_e_invalidos() {
        assert_eq!(
            leer_perfil(&v(&["verificar", "--perfil", "rapida"])),
            Ok(Perfil::Rapida)
        );
        assert_eq!(
            leer_perfil(&v(&["verificar", "--perfil", "completa"])),
            Ok(Perfil::Completa)
        );
        assert!(leer_perfil(&v(&["verificar", "--perfil", "otra"])).is_err());
        assert!(leer_perfil(&v(&["verificar"])).is_err());
    }
}
