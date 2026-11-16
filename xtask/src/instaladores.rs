//! Pruebas de los instaladores de una linea de `instalar/` (TASK-026, F2-11).
//!
//! Los scripts repiten la tabla de alias de P12 (no se generan): estas
//! pruebas exigen que sea la de `dist::TARGETS`. Ademas, que sean ASCII (con
//! `irm | iex`, Windows PowerShell 5.1 puede decodificar mal una tilde), que
//! `install.sh` tenga finales LF y que todo vaya dentro de una funcion que se
//! llama en la ultima linea (una descarga cortada no ejecuta medio script).
#![allow(clippy::panic)] // modulo solo de test: panic! es la forma de fallar

use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::dist::TARGETS;

fn leer(nombre: &str) -> String {
    let ruta = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("instalar")
        .join(nombre);
    std::fs::read_to_string(&ruta).unwrap_or_else(|e| panic!("{}: {e}", ruta.display()))
}

/// Alias de la forma de P12 que aparecen en `texto`: `win-`, `darwin-` o
/// `linux-` al principio de una palabra, seguido de minusculas y digitos.
fn alias_en(texto: &str) -> BTreeSet<String> {
    let mut encontrados = BTreeSet::new();
    for prefijo in ["win-", "darwin-", "linux-"] {
        for (i, _) in texto.match_indices(prefijo) {
            let a_mitad_de_palabra = texto[..i]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_ascii_alphanumeric());
            if a_mitad_de_palabra {
                continue;
            }
            let resto = &texto[i + prefijo.len()..];
            let fin = resto
                .find(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit()))
                .unwrap_or(resto.len());
            encontrados.insert(format!("{prefijo}{}", &resto[..fin]));
        }
    }
    encontrados
}

fn alias_de_p12(windows: bool) -> BTreeSet<String> {
    TARGETS
        .iter()
        .filter(|t| t.es_windows() == windows)
        .map(|t| t.alias.to_owned())
        .collect()
}

#[test]
fn alias_en_solo_cuenta_palabras_enteras() {
    let encontrados = alias_en("x linux-x64) twin-x win-arm64.exe 'darwin-' ");
    let esperado: BTreeSet<String> = ["darwin-", "linux-x64", "win-arm64"]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    assert_eq!(encontrados, esperado);
}

#[test]
fn la_tabla_de_alias_de_los_instaladores_es_la_de_p12() {
    assert_eq!(alias_en(&leer("install.sh")), alias_de_p12(false));
    assert_eq!(alias_en(&leer("install.ps1")), alias_de_p12(true));
}

#[test]
fn instaladores_ascii_y_con_la_llamada_en_la_ultima_linea() {
    for (nombre, llamada) in [
        ("install.sh", "main \"$@\""),
        ("install.ps1", "Install-Tc2"),
    ] {
        let texto = leer(nombre);
        assert!(texto.is_ascii(), "{nombre} tiene caracteres no ASCII");
        let ultima = texto.lines().rev().find(|l| !l.trim().is_empty());
        assert_eq!(ultima.map(str::trim), Some(llamada), "{nombre}");
    }
    assert!(
        !leer("install.sh").contains('\r'),
        "install.sh tiene finales CRLF"
    );
    // Con `irm | iex`, un `exit` cerraria la ventana de la persona.
    let sin_exit = leer("install.ps1")
        .lines()
        .all(|l| !l.trim_start().to_ascii_lowercase().starts_with("exit"));
    assert!(sin_exit, "install.ps1 usa exit");
}
