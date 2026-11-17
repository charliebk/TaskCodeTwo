//! Los `tarea.md` de la rama de estado, de los dos formatos, leidos en un
//! solo proceso (F2-4b). Quien los usa decide que hacer con los que no se
//! leen: `next` los cuenta en `ilegibles=` (TASK-030 C4), el alta los salta.

use super::lectura::Estado;
use super::transaccion::Fallo;
use crate::i18n::Mensaje;
use crate::i18n::catalogo::comun as txt;
use crate::tarea::{Cuerpo, Documento, ErrorTarea, Tarea};

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

/// El `tarea.md` de `id` en formato 2, para cambiarlo: la lectura comun de
/// las operaciones de una tarea (`start`, `descartar`, `pausa`, `reanudar`).
/// Sin el fichero, `rechazada motivo=no-existe`; en formato 1 de la 0.16,
/// `formato-1` (se convierte con `tc2 actualizar`); si no se entiende,
/// `ilegible`. Los diagnosticos llevan el ID.
pub fn tarea_de(estado: &dyn Estado, id: &str) -> Result<(Tarea, Cuerpo), Fallo> {
    let rechazo = |motivo: &str, m: Mensaje| Fallo::rechazada(motivo, m.con("id", id));
    let Some(texto) = estado.leer(&format!("{id}/tarea.md"))? else {
        return Err(rechazo("no-existe", Mensaje::nuevo(&txt::TAREA_NO_EXISTE)));
    };
    match crate::tarea::leer(&texto) {
        Ok(Documento::Formato2(t, c)) => Ok((t, c)),
        Ok(Documento::Formato1(..)) => {
            Err(rechazo("formato-1", Mensaje::nuevo(&txt::TAREA_FORMATO_1)))
        }
        Err(e) => Err(rechazo(
            "ilegible",
            Mensaje::nuevo(&txt::TAREA_ILEGIBLE).con("error", e.to_string()),
        )),
    }
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
