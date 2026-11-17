//! Los `tarea.md` de la rama de estado, de los dos formatos, leidos en un
//! solo proceso (F2-4b). Quien los usa decide que hacer con los que no se
//! leen: `next` los cuenta en `ilegibles=` (TASK-030 C4), el alta los salta.

use super::lectura::Estado;
use super::transaccion::Fallo;
use crate::tarea::{Documento, ErrorTarea};

/// Prefijo de las carpetas de tarea.
const PREFIJO: &str = "TASK-";

/// `TASK-<cifras>[a-z]/tarea.md` en la raiz (la rama de estado es plana,
/// ADR F1-1). El prefijo se compara sin mayusculas, como el alta.
pub fn es_tarea_md(ruta: &str) -> bool {
    let Some((carpeta, "tarea.md")) = ruta.split_once('/') else {
        return false;
    };
    let (Some(prefijo), Some(resto)) = (carpeta.get(..PREFIJO.len()), carpeta.get(PREFIJO.len()..))
    else {
        return false;
    };
    let cifras = resto
        .strip_suffix(|c: char| c.is_ascii_lowercase())
        .unwrap_or(resto);
    prefijo.eq_ignore_ascii_case(PREFIJO)
        && !cifras.is_empty()
        && cifras.bytes().all(|b| b.is_ascii_digit())
}

/// Un `tarea.md` con su ruta y su lectura.
pub type Leida = (String, Result<Documento, ErrorTarea>);

/// Cada `tarea.md` de `rutas` con su lectura, en orden de ruta. Un fallo de
/// git es [`Fallo`]; un `tarea.md` que no se entiende va con su error.
pub fn tareas(estado: &dyn Estado, rutas: &[String]) -> Result<Vec<Leida>, Fallo> {
    let mds: Vec<String> = rutas.iter().filter(|r| es_tarea_md(r)).cloned().collect();
    let textos = estado.leer_varios(&mds)?;
    Ok(mds
        .into_iter()
        .zip(textos)
        .filter_map(|(r, t)| t.map(|t| (r, crate::tarea::leer(&t))))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rutas_de_tarea_md() {
        for r in [
            "TASK-001/tarea.md",
            "TASK-1000/tarea.md",
            "TASK-012a/tarea.md",
            "task-013/tarea.md",
        ] {
            assert!(es_tarea_md(r), "{r}");
        }
        for r in [
            "ROADMAP.md",
            "TASK-/tarea.md",
            "TASK-01ab/tarea.md",
            "TASK-001/plan.md",
            "TASK-001/x/tarea.md",
            "x/TASK-001/tarea.md",
            "TASK-00A/tarea.md",
            "TÁSK-001/tarea.md",
        ] {
            assert!(!es_tarea_md(r), "{r}");
        }
    }
}
