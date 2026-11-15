//! Test de guarda de F2-8: ningun texto dirigido a la persona se escribe
//! fuera del catalogo. Lee el codigo de `src/` (sin los modulos de tests) y
//! busca las formas de colar un literal en un diagnostico.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Usos permitidos de `Mensaje::externo(` por fichero: texto que viene de
/// fuera (git, el sistema, HTTP) sin nada que traducir alrededor. Lista
/// cerrada: un uso nuevo tiene que anadirse aqui a proposito.
const EXTERNOS: &[(&str, usize)] = &[
    // Fallo al leer un blob sin diagnostico: queda la linea de contrato, que
    // no se traduce (ADR F1-4).
    ("estado/lectura.rs", 1),
];

/// Formas de meter un literal en un diagnostico sin pasar por el catalogo.
const PROHIBIDOS: &[&str] = &[
    ".diagnostico(\"",
    ".diagnostico(format!",
    "Mensaje::externo(\"",
    "Mensaje::externo(format!",
];

fn fuentes(dir: &Path, base: &Path, v: &mut Vec<(String, String)>) {
    let Ok(entradas) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entradas.flatten() {
        let p = e.path();
        if p.is_dir() {
            fuentes(&p, base, v);
        } else if p.extension().is_some_and(|x| x == "rs") {
            let rel = p
                .strip_prefix(base)
                .unwrap_or(&p)
                .to_string_lossy()
                .replace('\\', "/");
            if rel.starts_with("i18n/") {
                continue;
            }
            let texto = std::fs::read_to_string(&p).unwrap_or_default();
            // Fuera el modulo de tests del final del fichero.
            let codigo = match texto.find("#[cfg(test)]\nmod tests") {
                Some(i) => texto[..i].to_owned(),
                None => texto,
            };
            v.push((rel, codigo));
        }
    }
}

#[test]
fn ningun_literal_fuera_del_catalogo() {
    let base: PathBuf = [env!("CARGO_MANIFEST_DIR"), "src"].iter().collect();
    let mut v = Vec::new();
    fuentes(&base, &base, &mut v);
    assert!(
        v.len() > 10,
        "no se encontro el codigo en {}",
        base.display()
    );
    let mut fallos = Vec::new();
    let mut externos: BTreeMap<&str, usize> = BTreeMap::new();
    for (rel, codigo) in &v {
        for p in PROHIBIDOS {
            if codigo.contains(p) {
                fallos.push(format!("{rel}: usa `{p}`: pasa el texto al catalogo"));
            }
        }
        let n = codigo.matches("Mensaje::externo(").count();
        if n > 0 {
            externos.insert(rel.as_str(), n);
        }
    }
    let permitidos: BTreeMap<&str, usize> = EXTERNOS.iter().copied().collect();
    if externos != permitidos {
        fallos.push(format!(
            "usos de Mensaje::externo distintos de EXTERNOS: hay {externos:?}, se permiten {permitidos:?}"
        ));
    }
    assert!(fallos.is_empty(), "{}", fallos.join("\n"));
}
