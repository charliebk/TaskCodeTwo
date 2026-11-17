//! Los `tarea.md` de la rama de estado, de los dos formatos, leidos en un
//! solo proceso (F2-4b). Quien los usa decide que hacer con los que no se
//! leen: `next` los cuenta en `ilegibles=` (TASK-030 C4), el alta los salta.

use super::lectura::Estado;
use super::transaccion::Fallo;
use crate::grafo::{DESCARTADA, TERMINADA};
use crate::i18n::catalogo::comun as txt;
use crate::i18n::{Mensaje, Texto};
use crate::tarea::{Asignado, Cuerpo, Documento, ErrorTarea, Tarea};

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

/// El rechazo de la tarea `id` si su `asignado_a` es otra persona que
/// `quien`, comun a las operaciones de una tarea: `None` si no la lleva
/// nadie o la lleva el. La de una persona de la lista se rechaza con `ajena`
/// (motivo y texto de la orden: `tomada` en `start`, `de-otra-persona` en
/// `descartar`, `pausa` y `reanudar`) y devuelve su login; un texto heredado
/// de la 0.16, `asignado-heredado` en todas.
pub fn de_otra_persona(
    id: &str,
    asignado_a: Option<&Asignado>,
    quien: &Asignado,
    orden: &str,
    ajena: (&str, &'static Texto),
) -> Option<(Fallo, Option<String>)> {
    let a = asignado_a.filter(|a| !a.misma_persona(quien))?;
    let rechazo = |motivo: &str, m: Mensaje| Fallo::rechazada(motivo, m.con("id", id));
    Some(match (a.login(), a.heredado()) {
        (Some(login), _) => (
            rechazo(ajena.0, Mensaje::nuevo(ajena.1).con("login", login)),
            Some(login.to_owned()),
        ),
        (None, texto) => (
            rechazo(
                "asignado-heredado",
                Mensaje::nuevo(&txt::TAREA_ASIGNADO_HEREDADO)
                    .con("texto", texto.unwrap_or_default())
                    .con("orden", orden),
            ),
            None,
        ),
    })
}

/// El rechazo de la tarea `id` en un `estado` que no admite la `orden`
/// (no es de `admitidos`): `terminada`, `descartada` o `estado-no-valido`.
pub fn por_estado(id: &str, estado: &str, orden: &str, admitidos: &[&str]) -> Fallo {
    let (motivo, m) = match estado {
        TERMINADA => ("terminada", Mensaje::nuevo(&txt::TAREA_TERMINADA)),
        DESCARTADA => ("descartada", Mensaje::nuevo(&txt::TAREA_DESCARTADA)),
        otro => (
            "estado-no-valido",
            Mensaje::nuevo(&txt::TAREA_ESTADO_NO_VALIDO)
                .con("estado", otro)
                .con("estados", admitidos.join(", ")),
        ),
    };
    Fallo::rechazada(motivo, m.con("id", id).con("orden", orden))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Idioma;

    /// Motivo y diagnostico (en los dos idiomas) de un rechazo.
    fn partes(f: &Fallo) -> (String, Vec<String>) {
        match f {
            Fallo::Rechazada {
                motivo,
                diagnostico,
            } => (
                motivo.clone(),
                [Idioma::Es, Idioma::En]
                    .into_iter()
                    .map(|i| diagnostico.texto(i))
                    .collect(),
            ),
            Fallo::Error(_) => (String::new(), Vec::new()),
        }
    }

    /// Los rechazos comunes de `start` y de `descartar`/`pausa`/`reanudar`
    /// llevan el ID y la orden; el de estado, los estados que la admiten.
    #[test]
    fn rechazos_comunes_por_estado_y_por_asignado() {
        for (estado, motivo) in [
            (TERMINADA, "terminada"),
            (DESCARTADA, "descartada"),
            ("en-curso", "estado-no-valido"),
        ] {
            let (m, textos) = partes(&por_estado(
                "TASK-007",
                estado,
                "start",
                &["planificada", "en-diseno"],
            ));
            assert_eq!(m, motivo);
            for t in textos {
                assert!(t.contains("TASK-007") && t.contains("tc2 start"), "{t}");
                if motivo == "estado-no-valido" {
                    assert!(
                        t.contains("en-curso") && t.contains("planificada, en-diseno"),
                        "{t}"
                    );
                }
            }
        }
        let ana = Asignado::persona(7, "ana");
        let beto = Asignado::persona(9, "beto");
        let yo = ana.clone().unwrap_or_else(|| unreachable!());
        let ajena = (
            "de-otra-persona",
            &crate::i18n::catalogo::ciclo::DE_OTRA_PERSONA,
        );
        assert!(de_otra_persona("TASK-007", None, &yo, "pausa", ajena).is_none());
        assert!(de_otra_persona("TASK-007", ana.as_ref(), &yo, "pausa", ajena).is_none());
        let (f, dueno) = de_otra_persona("TASK-007", beto.as_ref(), &yo, "pausa", ajena)
            .unwrap_or_else(|| unreachable!());
        assert_eq!(dueno.as_deref(), Some("beto"));
        let (m, textos) = partes(&f);
        assert_eq!(m, "de-otra-persona");
        assert!(
            textos
                .iter()
                .all(|t| t.contains("beto") && t.contains("TASK-007")),
            "{textos:?}"
        );
    }

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
