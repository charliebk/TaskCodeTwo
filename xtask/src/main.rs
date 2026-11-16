//! `cargo xtask verificar --perfil rapida|completa`: el unico punto de entrada
//! de verificacion del workspace, el mismo en local y en la CI (F2-1).
//!
//! - `rapida`: `cargo fmt --check`, `clippy -D warnings` y `cargo test`, todo
//!   con `--locked`.
//! - `completa`: lo anterior y la comprobacion de build reproducible: copia el
//!   arbol a dos rutas distintas, compila `tc2` en release en limpio en cada
//!   una y exige el mismo sha256 (ADR F1-14). La CI solo la corre en Windows.
//!
//! Distribucion (F2-10), las mismas ordenes en local y en `release.yml`:
//!
//! - `empaquetar --target <triple> --salida <dir>`: compila `tc2` en release
//!   para un target de P12, lo copia como `tc2-<version>-<alias>[.exe]`, lo
//!   firma ad hoc en macOS y deja al lado su `<fichero>.sha256`.
//! - `sumas <dir>`: escribe `<dir>/SHA256SUMS` con los binarios de `<dir>`.
//! - `comprobar-tag <tag>`: escribe `estable` o `prerelease`, o falla si el
//!   tag no casa con la version del workspace.
//! - `version`: escribe la version del workspace.
//!
//! El resultado va en el codigo de salida: 0 verde, 1 rojo, 2 mal invocado.

mod dist;
#[cfg(test)]
mod instaladores;
mod sha256;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use dist::{Clase, Target};

/// Version del workspace (`version.workspace = true`): la misma que `tc2`.
const VERSION: &str = env!("CARGO_PKG_VERSION");

const USO: &str = "Uso: cargo xtask verificar --perfil rapida|completa
     cargo xtask empaquetar --target <triple> --salida <dir>
     cargo xtask sumas <dir>
     cargo xtask comprobar-tag <tag>
     cargo xtask version";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Perfil {
    Rapida,
    Completa,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Orden {
    Verificar(Perfil),
    Empaquetar { target: Target, salida: PathBuf },
    Sumas(PathBuf),
    ComprobarTag(String),
    Version,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let orden = match leer_orden(&args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("xtask: {e}");
            eprintln!("{USO}");
            return ExitCode::from(2);
        }
    };
    let resultado = match orden {
        Orden::Verificar(perfil) => {
            verificar(perfil).map(|()| println!("xtask: verificacion {perfil:?} en verde"))
        }
        Orden::Empaquetar { target, salida } => empaquetar(target, &salida),
        Orden::Sumas(dir) => escribir_sumas(&dir),
        Orden::ComprobarTag(tag) => dist::clase_de_tag(&tag, VERSION).map(|c| match c {
            Clase::Estable => println!("estable"),
            Clase::Prerelease => println!("prerelease"),
        }),
        Orden::Version => {
            println!("{VERSION}");
            Ok(())
        }
    };
    match resultado {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("xtask: ROJO: {e}");
            ExitCode::from(1)
        }
    }
}

fn leer_orden(args: &[String]) -> Result<Orden, String> {
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    match a.as_slice() {
        ["verificar", "--perfil", valor] => match *valor {
            "rapida" => Ok(Orden::Verificar(Perfil::Rapida)),
            "completa" => Ok(Orden::Verificar(Perfil::Completa)),
            otro => Err(format!("perfil desconocido «{otro}»")),
        },
        ["empaquetar", "--target", triple, "--salida", salida] => {
            let target = Target::de_triple(triple)
                .ok_or_else(|| format!("target «{triple}» fuera de los 6 de P12"))?;
            Ok(Orden::Empaquetar {
                target,
                salida: PathBuf::from(salida),
            })
        }
        ["sumas", dir] => Ok(Orden::Sumas(PathBuf::from(dir))),
        ["comprobar-tag", tag] => Ok(Orden::ComprobarTag((*tag).to_owned())),
        ["version"] => Ok(Orden::Version),
        _ => Err("argumentos no validos".to_owned()),
    }
}

/// Un RUSTFLAGS en el entorno anularia en silencio los rustflags de
/// `.cargo/config.toml` (/Brepro).
fn guarda_rustflags() -> Result<(), String> {
    for var in ["RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"] {
        if std::env::var_os(var).is_some() {
            return Err(format!(
                "{var} esta definido y anularia los rustflags de .cargo/config.toml (/Brepro); quitalo"
            ));
        }
    }
    Ok(())
}

/// Compila `tc2` para `target`, lo deja en `salida` con su nombre publicado,
/// lo firma ad hoc en macOS y escribe al lado `<fichero>.sha256`.
fn empaquetar(target: Target, salida: &Path) -> Result<(), String> {
    guarda_rustflags()?;
    let raiz = raiz_workspace()?;
    cargo(
        &raiz,
        &[
            "build",
            "--release",
            "--locked",
            "-p",
            "tc2",
            "--target",
            target.triple,
        ],
        None,
    )?;
    let origen = dir_target(&raiz, std::env::var_os("CARGO_TARGET_DIR"))
        .join(target.triple)
        .join("release")
        .join(target.binario_cargo());
    std::fs::create_dir_all(salida).map_err(|e| format!("{}: {e}", salida.display()))?;
    let fichero = target.fichero(VERSION);
    let destino = salida.join(&fichero);
    std::fs::copy(&origen, &destino).map_err(|e| {
        format!(
            "no se puede copiar {} a {}: {e}",
            origen.display(),
            destino.display()
        )
    })?;
    if target.es_macos() {
        // Firma ad hoc (D4a): sin ella Gatekeeper mata un binario arm64.
        ejecutar("codesign", &["--force", "--sign", "-"], &destino)?;
        ejecutar(
            "codesign",
            &["--verify", "--strict", "--verbose=2"],
            &destino,
        )?;
    }
    let bytes = std::fs::read(&destino).map_err(|e| format!("{}: {e}", destino.display()))?;
    let hash = sha256::hex(&bytes);
    let suma = salida.join(format!("{fichero}{}", dist::SUFIJO_SUMA));
    std::fs::write(&suma, dist::linea_suma(&hash, &fichero))
        .map_err(|e| format!("{}: {e}", suma.display()))?;
    println!(
        "xtask: empaquetado {} ({} bytes)\nxtask: sha256 {hash}  {fichero}",
        destino.display(),
        bytes.len()
    );
    Ok(())
}

/// Directorio de salida de cargo. cargo corre en `raiz`, asi que un
/// `CARGO_TARGET_DIR` relativo es relativo a ella (`join` con una ruta
/// absoluta la devuelve tal cual). No mira `build.target-dir` de
/// `.cargo/config.toml`: el repo no lo usa.
fn dir_target(raiz: &Path, var: Option<std::ffi::OsString>) -> PathBuf {
    var.map_or_else(|| raiz.join("target"), |d| raiz.join(d))
}

/// Lanza `programa args... fichero` y exige codigo 0.
fn ejecutar(programa: &str, args: &[&str], fichero: &Path) -> Result<(), String> {
    println!("xtask: {programa} {} {}", args.join(" "), fichero.display());
    let estado = Command::new(programa)
        .args(args)
        .arg(fichero)
        .status()
        .map_err(|e| format!("no se pudo lanzar {programa}: {e}"))?;
    if estado.success() {
        Ok(())
    } else {
        Err(format!("falla `{programa} {}` ({estado})", args.join(" ")))
    }
}

/// Escribe `<dir>/SHA256SUMS` con los binarios de `<dir>`: todo fichero que
/// no sea el propio `SHA256SUMS` ni un `<fichero>.sha256`.
fn escribir_sumas(dir: &Path) -> Result<(), String> {
    let lectura = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut entradas = Vec::new();
    for entrada in lectura {
        let entrada = entrada.map_err(|e| format!("{}: {e}", dir.display()))?;
        let ruta = entrada.path();
        let nombre = entrada
            .file_name()
            .into_string()
            .map_err(|_| format!("nombre no UTF-8 en {}", dir.display()))?;
        if nombre == dist::SUMAS || nombre.ends_with(dist::SUFIJO_SUMA) {
            continue;
        }
        if !ruta.is_file() {
            return Err(format!("{} no es un fichero", ruta.display()));
        }
        let bytes = std::fs::read(&ruta).map_err(|e| format!("{}: {e}", ruta.display()))?;
        entradas.push((nombre, sha256::hex(&bytes)));
    }
    let texto = dist::sumas(&entradas)?;
    let destino = dir.join(dist::SUMAS);
    std::fs::write(&destino, &texto).map_err(|e| format!("{}: {e}", destino.display()))?;
    print!("{texto}");
    Ok(())
}

fn verificar(perfil: Perfil) -> Result<(), String> {
    guarda_rustflags()?;
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
    match &resultado {
        // En verde no hay nada que mirar: se borra la copia.
        Ok(()) => {
            let _ = std::fs::remove_dir_all(&base);
        }
        // En rojo se conservan los dos binarios para poder compararlos.
        Err(_) => eprintln!(
            "xtask: se conservan las dos copias para inspeccionarlas: {}",
            base.display()
        ),
    }
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
    comparar_hashes(&hashes)?;
    println!("xtask: build reproducible: mismo sha256 en las dos rutas");
    Ok(())
}

/// Verde solo si hay exactamente dos hashes y son iguales.
fn comparar_hashes(hashes: &[String]) -> Result<(), String> {
    match hashes {
        [a, b] if a == b => Ok(()),
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
            leer_orden(&v(&["verificar", "--perfil", "rapida"])),
            Ok(Orden::Verificar(Perfil::Rapida))
        );
        assert_eq!(
            leer_orden(&v(&["verificar", "--perfil", "completa"])),
            Ok(Orden::Verificar(Perfil::Completa))
        );
        assert!(leer_orden(&v(&["verificar", "--perfil", "otra"])).is_err());
        assert!(leer_orden(&v(&["verificar"])).is_err());
    }

    #[test]
    fn ordenes_de_distribucion() {
        assert_eq!(
            leer_orden(&v(&[
                "empaquetar",
                "--target",
                "aarch64-apple-darwin",
                "--salida",
                "dist"
            ])),
            Ok(Orden::Empaquetar {
                target: dist::TARGETS[3],
                salida: PathBuf::from("dist")
            })
        );
        assert!(
            leer_orden(&v(&[
                "empaquetar",
                "--target",
                "x86_64-unknown-linux-gnu",
                "--salida",
                "dist"
            ]))
            .is_err()
        );
        assert_eq!(
            leer_orden(&v(&["sumas", "dist"])),
            Ok(Orden::Sumas(PathBuf::from("dist")))
        );
        assert_eq!(
            leer_orden(&v(&["comprobar-tag", "v1.0.0"])),
            Ok(Orden::ComprobarTag("v1.0.0".to_owned()))
        );
        assert_eq!(leer_orden(&v(&["version"])), Ok(Orden::Version));
        assert!(leer_orden(&v(&["version", "x"])).is_err());
        assert!(leer_orden(&v(&["sumas", "a", "b"])).is_err());
        assert!(leer_orden(&v(&["sumas"])).is_err());
    }

    #[test]
    fn dir_target_relativo_a_la_raiz() {
        let raiz = std::env::temp_dir().join("raiz-ws");
        assert_eq!(dir_target(&raiz, None), raiz.join("target"));
        assert_eq!(dir_target(&raiz, Some("relt".into())), raiz.join("relt"));
        let absoluta = std::env::temp_dir().join("otro-target");
        assert_eq!(
            dir_target(&raiz, Some(absoluta.clone().into_os_string())),
            absoluta
        );
    }

    #[test]
    fn comparar_hashes_exige_dos_iguales() {
        assert!(comparar_hashes(&v(&["aa", "aa"])).is_ok());
        assert!(comparar_hashes(&v(&["aa", "ab"])).is_err());
        assert!(comparar_hashes(&v(&["aa"])).is_err());
        assert!(comparar_hashes(&v(&[])).is_err());
    }
}
