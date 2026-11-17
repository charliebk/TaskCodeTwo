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

use distribucion::TARGETS;

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

/// Lo que detecta cada script (`uname -s/-m` en sh, PROCESSOR_ARCHITECTURE
/// del registro en ps1) y el alias que tiene que salir en esa misma rama.
const DETECCION: [(&str, &str, &str); 6] = [
    ("install.sh", "Linux/x86_64", "linux-x64"),
    ("install.sh", "Linux/aarch64", "linux-arm64"),
    ("install.sh", "Darwin/x86_64", "darwin-x64"),
    ("install.sh", "Darwin/arm64", "darwin-arm64"),
    ("install.ps1", "'AMD64'", "win-x64"),
    ("install.ps1", "'ARM64'", "win-arm64"),
];

#[test]
fn cada_sistema_detectado_da_su_alias() {
    let cubiertos: BTreeSet<String> = DETECCION.iter().map(|d| d.2.to_owned()).collect();
    assert_eq!(
        cubiertos,
        alias_de_p12(false)
            .union(&alias_de_p12(true))
            .cloned()
            .collect()
    );
    for (nombre, clave, alias) in DETECCION {
        let texto = leer(nombre);
        let ramas: Vec<&str> = texto.lines().filter(|l| l.contains(clave)).collect();
        assert_eq!(ramas.len(), 1, "{nombre}: una sola rama con {clave}");
        let esperado: BTreeSet<String> = [alias.to_owned()].into_iter().collect();
        assert_eq!(
            alias_en(ramas[0]),
            esperado,
            "{nombre}: {clave} tiene que dar {alias}"
        );
    }
}

/// `palabra` como palabra entera (sin letra, digito ni _ a los lados), sin
/// distinguir mayusculas.
fn tiene_palabra(linea: &str, palabra: &str) -> bool {
    let linea = linea.to_ascii_lowercase();
    let es_de_palabra = |c: char| c.is_ascii_alphanumeric() || c == '_';
    linea.match_indices(palabra).any(|(i, _)| {
        let antes = linea[..i].chars().next_back().is_some_and(es_de_palabra);
        let despues = linea[i + palabra.len()..]
            .chars()
            .next()
            .is_some_and(es_de_palabra);
        !antes && !despues
    })
}

#[test]
fn tiene_palabra_solo_cuenta_palabras_enteras() {
    assert!(tiene_palabra("exit 1", "exit"));
    assert!(tiene_palabra("if ($x) { EXIT }", "exit"));
    assert!(tiene_palabra("[Environment]::Exit(1)", "exit"));
    assert!(!tiene_palabra("$exito = 1", "exit"));
    assert!(!tiene_palabra("Exit_Code", "exit"));
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
    // Con `irm | iex`, un `exit` (o `[Environment]::Exit`) en cualquier
    // sitio cerraria la ventana de la persona. Los comentarios no cuentan.
    let con_exit: Vec<String> = leer("install.ps1")
        .lines()
        .filter(|l| !l.trim_start().starts_with('#') && tiene_palabra(l, "exit"))
        .map(str::to_owned)
        .collect();
    assert!(con_exit.is_empty(), "install.ps1 usa exit: {con_exit:?}");
}

/// TASK-078 (F2-14): `install.sh` escribe el bloque del perfil con las
/// marcas y la linea que `tc2 desinstalar` reconoce (constantes de
/// `distribucion`), y `install.ps1` anade el directorio al PATH del usuario
/// con `Add-PathUsuario`, cuya inversa hace `tc2 desinstalar`.
#[test]
fn install_sh_usa_las_marcas_que_quita_desinstalar() {
    let sh = leer("install.sh");
    assert!(
        sh.contains(&format!(
            "MARCA_INICIO=\"{}\"",
            distribucion::MARCA_INICIO_PERFIL
        )),
        "install.sh no usa MARCA_INICIO_PERFIL"
    );
    assert!(
        sh.contains(&format!("MARCA_FIN=\"{}\"", distribucion::MARCA_FIN_PERFIL)),
        "install.sh no usa MARCA_FIN_PERFIL"
    );
    assert!(
        sh.contains(&format!("printf '{}", distribucion::LINEA_PERFIL)),
        "install.sh no escribe la linea del bloque"
    );
    let ps1 = leer("install.ps1");
    assert!(ps1.contains("function Add-PathUsuario"));
    assert!(ps1.contains(r"taskcodetwo\bin"));
}
