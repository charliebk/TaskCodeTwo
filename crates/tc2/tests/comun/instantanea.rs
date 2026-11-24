//! Instantanea de una carpeta para las comprobaciones de «la orden no toca
//! nada» (A8-12): un solo recorrido para todos los tests.

use std::collections::BTreeMap;
use std::path::Path;

/// Foto recursiva de `dir`: ruta relativa (con `/`) → `"dir"` para cada
/// carpeta y `"<len> <sha256>"` para cada fichero. Las entradas cuyo nombre
/// esta en `excluir` no entran, a cualquier profundidad; una carpeta
/// excluida se salta entera. Una carpeta inexistente da el mapa vacio; un
/// fichero que no se puede leer hace fallar el test.
pub fn instantanea(dir: &Path, excluir: &[&str]) -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    recorrer(dir, dir, excluir, &mut m);
    m
}

fn recorrer(base: &Path, d: &Path, excluir: &[&str], m: &mut BTreeMap<String, String>) {
    let Ok(it) = std::fs::read_dir(d) else { return };
    for e in it.flatten() {
        if excluir.iter().any(|x| e.file_name() == *x) {
            continue;
        }
        let p = e.path();
        let rel = p
            .strip_prefix(base)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if p.is_dir() {
            m.insert(rel, "dir".to_owned());
            recorrer(base, &p, excluir, m);
        } else {
            let b = std::fs::read(&p)
                .unwrap_or_else(|err| panic!("leer {} para la instantanea: {err}", p.display()));
            m.insert(
                rel,
                format!("{} {}", b.len(), distribucion::sha256::hex(&b)),
            );
        }
    }
}
