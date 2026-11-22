//! Detector de dominio de F1-15 (decision 3; F4-6): funcion pura sobre el
//! texto de la tarea, sus etiquetas y las rutas del cambio, con la
//! `tabla-dominios` del ADR ([`super::tabla::dominios`]) y lo que la
//! configuracion anade (`seleccion.dominios_extra`, que solo anade senales a
//! un dominio de la lista cerrada).
//!
//! - Etiqueta: igualdad, plegada (sin tildes ni mayusculas, como la PoC T).
//! - Palabra: al principio de una palabra del objetivo o de los criterios,
//!   plegados con [`crate::tarea::texto::plegar`].
//! - Ruta: glob sin distinguir mayusculas sobre cada ruta (los dos nombres
//!   de cada renombrado llegan como dos rutas).
//!
//! El texto de la tarea no es confiable y solo sube: [`union`] suma las
//! senales congeladas con el plan aprobado (`roles.diseno.senales`) a las
//! del texto vivo, de modo que reescribir la tarea no quita un dominio.

use std::collections::{BTreeMap, BTreeSet};

use super::tabla::{DOMINIOS, FilaDominio, Senal};
use crate::config::Senales;
use crate::tarea::texto::plegar;

/// Lo que se mira de una tarea.
#[derive(Debug, Clone, Copy)]
pub struct Texto<'a> {
    /// Objetivo.
    pub objetivo: &'a str,
    /// Criterios de aceptacion.
    pub criterios: &'a [String],
    /// Etiquetas.
    pub etiquetas: &'a [String],
}

/// Dominios detectados, en el orden de la lista cerrada, cada uno con sus
/// senales `<senal>:<valor>` ordenadas y sin repetir.
pub type Dominios = Vec<(String, Vec<String>)>;

fn casa_palabra(texto: &str, valor: &str) -> bool {
    let v = plegar(valor);
    !v.is_empty() && format!(" {texto}").contains(&format!(" {v}"))
}

/// Los dominios de `t` y `rutas` con `tabla` y `extra`.
pub fn dominios(
    t: &Texto<'_>,
    rutas: &[String],
    tabla: &[FilaDominio],
    extra: &BTreeMap<String, Senales>,
) -> Dominios {
    let texto = plegar(&format!("{}\n{}", t.objetivo, t.criterios.join("\n")));
    let etiquetas: BTreeSet<String> = t.etiquetas.iter().map(|e| plegar(e)).collect();
    let mut filas: Vec<(String, Senal, String)> = tabla
        .iter()
        .map(|f| (f.dominio.clone(), f.senal, f.valor.clone()))
        .collect();
    for (d, s) in extra {
        for (lista, senal) in [
            (&s.etiquetas, Senal::Etiqueta),
            (&s.palabras, Senal::Palabra),
            (&s.rutas, Senal::Ruta),
        ] {
            filas.extend(lista.iter().map(|v| (d.clone(), senal, v.clone())));
        }
    }
    let mut vistos: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    // Un dominio fuera de la lista (no lo admite la configuracion) se cae
    // en [`ordenar`].
    for (d, senal, valor) in filas {
        let ok = match senal {
            Senal::Etiqueta => etiquetas.contains(&plegar(&valor)),
            Senal::Palabra => casa_palabra(&texto, &valor),
            Senal::Ruta => {
                let g = super::globs(std::slice::from_ref(&valor));
                rutas.iter().any(|r| g.is_match(super::normalizar(r)))
            }
        };
        if ok {
            vistos
                .entry(d)
                .or_default()
                .insert(format!("{}:{valor}", senal.nombre()));
        }
    }
    ordenar(vistos)
}

fn ordenar(vistos: BTreeMap<String, BTreeSet<String>>) -> Dominios {
    DOMINIOS
        .iter()
        .filter_map(|d| {
            vistos
                .get(*d)
                .map(|s| ((*d).to_owned(), s.iter().cloned().collect()))
        })
        .collect()
}

/// Las senales como se anotan en `roles.<rol>.senales`:
/// `<dominio>:<senal>:<valor>`.
pub fn anotar(d: &Dominios) -> Vec<String> {
    d.iter()
        .flat_map(|(dom, ss)| ss.iter().map(move |s| format!("{dom}:{s}")))
        .collect()
}

/// `d` mas las senales `congeladas` (`<dominio>:<senal>:<valor>`): solo
/// anade. Una senal congelada de un dominio fuera de la lista no cuenta.
pub fn union(d: &Dominios, congeladas: &[String]) -> Dominios {
    let mut vistos: BTreeMap<String, BTreeSet<String>> = d
        .iter()
        .map(|(k, v)| (k.clone(), v.iter().cloned().collect()))
        .collect();
    for c in congeladas {
        if let Some((dom, s)) = c.split_once(':')
            && DOMINIOS.contains(&dom)
            && !s.is_empty()
        {
            vistos
                .entry(dom.to_owned())
                .or_default()
                .insert(s.to_owned());
        }
    }
    ordenar(vistos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seleccion::tabla;

    fn tabla_adr() -> Vec<FilaDominio> {
        tabla::dominios(tabla::ADR).unwrap_or_default()
    }

    fn v(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|x| (*x).to_owned()).collect()
    }

    fn det(obj: &str, etq: &[&str], rutas: &[&str]) -> Dominios {
        let criterios = v(&["criterio"]);
        let etiquetas = v(etq);
        let t = Texto {
            objetivo: obj,
            criterios: &criterios,
            etiquetas: &etiquetas,
        };
        dominios(&t, &v(rutas), &tabla_adr(), &BTreeMap::new())
    }

    /// Criterio 3 (tabla adversaria): tildes y mayusculas, palabra a mitad
    /// de otra, etiqueta, los dos nombres de un renombrado,
    /// `.Claude/settings.json`, `dominios_extra` que solo anade y la union
    /// con las senales congeladas que no deja quitar un dominio.
    #[test]
    fn tabla_adversaria() {
        let p = |s: &[&str]| vec![("puertas-seguridad".to_owned(), v(s))];
        // Tildes y mayusculas: «APROBACIÓN» casa con `aprobacion`.
        assert_eq!(
            det("Pide la APROBACIÓN", &[], &[]),
            p(&["palabra:aprobacion"])
        );
        // Al principio de palabra: «checks» si, «recheck» y «matoken» no.
        assert_eq!(det("los checks", &[], &[]), p(&["palabra:check"]));
        assert!(det("un recheck y un matoken", &[], &[]).is_empty());
        // Etiqueta exacta (plegada); una parecida no.
        assert_eq!(det("nada", &["Seguridad"], &[]), p(&["etiqueta:seguridad"]));
        assert!(det("nada", &["seguridades"], &[]).is_empty());
        // Rendimiento por etiqueta.
        assert_eq!(
            det("nada", &["wasm"], &[]),
            vec![("rendimiento".to_owned(), v(&["etiqueta:wasm"]))]
        );
        // Renombrado: basta con uno de los dos nombres.
        assert_eq!(
            det("nada", &[], &["src/a.rs", ".github/workflows/x.yml"]),
            p(&["ruta:.github/**"])
        );
        // Sin distinguir mayusculas ni barras de Windows.
        assert_eq!(
            det("nada", &[], &[".Claude\\settings.json"]),
            p(&["ruta:.claude/**"])
        );
        assert_eq!(
            det("nada", &[], &["docs/CODEOWNERS"]),
            p(&["ruta:**/CODEOWNERS"])
        );
        assert!(det("nada", &[], &["src/claude/x.rs"]).is_empty());
        // `dominios_extra` solo anade (un dominio nuevo se ignora).
        let extra = BTreeMap::from([
            (
                "puertas-seguridad".to_owned(),
                Senales {
                    etiquetas: Vec::new(),
                    palabras: v(&["credencial"]),
                    rutas: v(&["src/auth/**"]),
                },
            ),
            (
                "datos".to_owned(),
                Senales {
                    etiquetas: Vec::new(),
                    palabras: v(&["tabla"]),
                    rutas: Vec::new(),
                },
            ),
        ]);
        let criterios = v(&["la tabla"]);
        let t = Texto {
            objetivo: "Una credencial nueva",
            criterios: &criterios,
            etiquetas: &[],
        };
        let d = dominios(&t, &v(&["src/auth/x.rs"]), &tabla_adr(), &extra);
        assert_eq!(d, p(&["palabra:credencial", "ruta:src/auth/**"]));
        // Union: el texto vivo ya no dice nada y lo congelado sigue.
        let vivo = det("otro vocabulario", &[], &[]);
        assert!(vivo.is_empty());
        let congeladas = v(&["puertas-seguridad:palabra:check", "datos:palabra:tabla"]);
        assert_eq!(union(&vivo, &congeladas), p(&["palabra:check"]));
        assert_eq!(
            anotar(&union(&det("token", &[], &[]), &congeladas)),
            v(&[
                "puertas-seguridad:palabra:check",
                "puertas-seguridad:palabra:token"
            ])
        );
    }
}
