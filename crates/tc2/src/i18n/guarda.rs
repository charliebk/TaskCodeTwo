//! Test de guarda de F2-8: ningun texto dirigido a la persona se escribe
//! fuera del catalogo. Lee el codigo de `src/` (sin los modulos de tests) y
//! busca las formas de colar un literal en un diagnostico.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Usos permitidos por fichero de `Mensaje::externo(` y de la entrada
/// `EXTERNO` del catalogo (la que hay debajo, A7-10): texto que viene de
/// fuera (git, el sistema, HTTP) sin nada que traducir alrededor. Lista
/// cerrada: un uso nuevo tiene que anadirse aqui a proposito.
const EXTERNOS: &[(&str, usize)] = &[
    // `Informe::mensaje`: un informe sin diagnostico deja la linea de
    // contrato, que no se traduce (ADR F1-4).
    ("contrato.rs", 1),
    // `plan_dato::externo`: el error del sistema o de git en el hueco
    // `{error}` de `FALLO_AL_APLICAR` (A7-07).
    ("plan_dato.rs", 1),
];

/// Formas de meter un literal en un diagnostico sin pasar por el catalogo.
const PROHIBIDOS: &[&str] = &[
    ".diagnostico(\"",
    ".diagnostico(format!",
    "Mensaje::externo(\"",
    "Mensaje::externo(format!",
];

/// `Err("...")` con prosa (un literal con espacios) permitidos por fichero:
/// errores internos que no llegan a la persona como diagnostico propio.
/// Un `Err` con prosa acaba en el hueco `{error}` de otro mensaje y sale en
/// espanol con cualquier idioma (A7-07). Lista cerrada.
const LITERALES_ERR: &[(&str, usize)] = &[
    // Comprobacion del binario descargado: el texto va a la linea de fallo
    // de `actualizar --cli` (fuera de A7, decision 2 de TASK-145).
    ("actualizar_cli.rs", 1),
    // Validacion del informe de verificacion (fuera de A7, decision 2 de
    // TASK-145).
    ("verificacion/informe.rs", 1),
];

/// Cuantas llamadas `{llamada}"..."` de `codigo` llevan como primer
/// argumento un literal con espacios (prosa), aunque el literal empiece en
/// la linea siguiente.
fn con_prosa(codigo: &str, llamada: &str) -> usize {
    codigo
        .match_indices(llamada)
        .filter(|(i, m)| {
            let resto = codigo[i + m.len()..].trim_start();
            resto
                .strip_prefix('"')
                .and_then(|r| r.split('"').next())
                .is_some_and(|l| l.contains(' '))
        })
        .count()
}

/// Cuantos `Err("...")` de `codigo` llevan un literal con espacios.
fn literales_err(codigo: &str) -> usize {
    con_prosa(codigo, "Err(")
}

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
    let mut literales: BTreeMap<&str, usize> = BTreeMap::new();
    for (rel, codigo) in &v {
        for p in PROHIBIDOS {
            if codigo.contains(p) {
                fallos.push(format!("{rel}: usa `{p}`: pasa el texto al catalogo"));
            }
        }
        let n = codigo.matches("Mensaje::externo(").count() + codigo.matches("EXTERNO").count();
        if n > 0 {
            externos.insert(rel.as_str(), n);
        }
        let n = literales_err(codigo);
        if n > 0 {
            literales.insert(rel.as_str(), n);
        }
        // `externo("...")` (el de `Mensaje` o el de `plan_dato`) con prosa:
        // un texto de `tc2` sin catalogo que sale tal cual en cualquier
        // idioma. Sin excepciones.
        if con_prosa(codigo, "externo(") > 0 {
            fallos.push(format!(
                "{rel}: externo(\"...\") con prosa: pasa el texto al catalogo"
            ));
        }
    }
    let permitidos: BTreeMap<&str, usize> = EXTERNOS.iter().copied().collect();
    if externos != permitidos {
        fallos.push(format!(
            "usos de Mensaje::externo o EXTERNO distintos de EXTERNOS: hay {externos:?}, se permiten {permitidos:?}"
        ));
    }
    let permitidos: BTreeMap<&str, usize> = LITERALES_ERR.iter().copied().collect();
    if literales != permitidos {
        fallos.push(format!(
            "Err(\"...\") con prosa distintos de LITERALES_ERR: hay {literales:?}, se permiten {permitidos:?}: pasa el texto al catalogo"
        ));
    }
    assert!(fallos.is_empty(), "{}", fallos.join("\n"));
}
