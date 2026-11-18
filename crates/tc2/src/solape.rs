//! Aviso de solape entre tareas en curso (F3-11; ADR F1-17, punto 11 y su
//! adenda de TASK-082).
//!
//! Sin zonas declaradas (llegan con el plan de F4-1), el solape sale de lo
//! que ya tocan las ramas: el diff real de la rama propia frente al de cada
//! otra tarea en curso o en revision con la rama publicada en `origin`.
//! Dos tareas solapan si tocan el mismo fichero o el mismo **modulo**, que
//! es la carpeta padre del fichero (los de la raiz no tienen modulo).
//!
//! Es siempre un aviso: nunca cambia el resultado de `start` (F1-17, D4a).
//!
//! Punto de extension de F4-1: [`Toca`] ganara las `zonas` (globs) de la
//! tarea y [`cruzar`] la regla de prefijo de F1-17 (punto 11).

use std::collections::BTreeSet;

use crate::plataforma::Personas;
use crate::situacion::{self, Ficha};
use crate::tarea::Documento;

/// Estados que cuentan para el solape (D2a de TASK-082): las tareas que
/// alguien esta cambiando o que esperan revision con sus cambios en la rama.
pub const ESTADOS: [&str; 2] = ["en-curso", "en-revision"];

/// Una tarea en curso o en revision, como la proyecta [`Ficha::de`] (la
/// misma lectura de `board`, `status` y `roadmap`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnCurso {
    /// `TASK-NNN`.
    pub id: String,
    /// Quien la lleva, como lo ensenan las vistas (`@login`, el texto
    /// heredado o `—`).
    pub quien: String,
    /// Su rama, si la tiene.
    pub rama: Option<String>,
}

/// Las tareas en curso o en revision de unos `tarea.md` leidos, por ID y
/// sin repetir (con un ID repetido vale la primera, como en el ROADMAP).
pub fn en_curso<'a>(docs: impl IntoIterator<Item = &'a Documento>) -> Vec<EnCurso> {
    let sin_lista = Personas::default();
    let mut v: Vec<EnCurso> = docs
        .into_iter()
        .map(Ficha::de)
        .filter(|f| f.nodo.estado().is_some_and(|e| ESTADOS.contains(&e)))
        .map(|f| EnCurso {
            quien: situacion::quien(f.asignado.as_ref(), &sin_lista).texto(),
            id: f.nodo.id,
            rama: f.rama,
        })
        .collect();
    v.sort_by(|a, b| a.id.cmp(&b.id));
    v.dedup_by(|b, a| a.id == b.id);
    v
}

/// Lo que toca otra tarea: su ID, quien la lleva y los ficheros de su rama.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toca {
    /// `TASK-NNN`.
    pub id: String,
    /// Quien la lleva (texto de las vistas).
    pub quien: String,
    /// Ficheros que cambia su rama desde su base.
    pub ficheros: BTreeSet<String>,
}

/// Modulo de un fichero (D1a de TASK-082): su carpeta padre. Un fichero de
/// la raiz no tiene modulo. Las rutas son las de `git diff`, siempre con
/// `/`: una `\` es parte del nombre, no un separador.
pub fn modulo(ruta: &str) -> Option<&str> {
    let (padre, _) = ruta.rsplit_once('/')?;
    (!padre.is_empty()).then_some(padre)
}

fn modulos(ficheros: &BTreeSet<String>) -> BTreeSet<&str> {
    ficheros.iter().filter_map(|f| modulo(f)).collect()
}

/// El solape de la tarea propia con otra.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cruce {
    /// `TASK-NNN` de la otra.
    pub id: String,
    /// Quien la lleva (texto de las vistas).
    pub quien: String,
    /// Ficheros que tocan las dos, en orden.
    pub ficheros: Vec<String>,
    /// Modulos que tocan las dos sin compartir fichero en ellos, en orden.
    pub modulos: Vec<String>,
}

/// Los cruces de los ficheros `propia` con cada una de `otras`, por ID. Un
/// fichero comun es un cruce de fichero; un modulo comun en el que no hay
/// fichero comun, un cruce de modulo (asi se distingue «tocamos lo mismo» de
/// «tocamos la misma carpeta»). Sin nada comun, la otra no sale.
pub fn cruzar(propia: &BTreeSet<String>, otras: &[Toca]) -> Vec<Cruce> {
    let mios = modulos(propia);
    let mut v: Vec<Cruce> = otras
        .iter()
        .filter_map(|o| {
            let ficheros: Vec<String> = propia.intersection(&o.ficheros).cloned().collect();
            let ya: BTreeSet<&str> = ficheros.iter().filter_map(|f| modulo(f)).collect();
            let modulos: Vec<String> = mios
                .intersection(&modulos(&o.ficheros))
                .filter(|m| !ya.contains(*m))
                .map(|m| (*m).to_owned())
                .collect();
            (!ficheros.is_empty() || !modulos.is_empty()).then(|| Cruce {
                id: o.id.clone(),
                quien: o.quien.clone(),
                ficheros,
                modulos,
            })
        })
        .collect();
    v.sort_by(|a, b| a.id.cmp(&b.id));
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tarea::{Asignado, Cuerpo, Tarea};

    fn conjunto(v: &[&str]) -> BTreeSet<String> {
        v.iter().map(|s| (*s).to_owned()).collect()
    }

    fn toca(id: &str, ficheros: &[&str]) -> Toca {
        Toca {
            id: id.to_owned(),
            quien: format!("@{}", id.to_lowercase()),
            ficheros: conjunto(ficheros),
        }
    }

    #[test]
    fn modulo_es_la_carpeta_padre() {
        assert_eq!(modulo("src/x/a.rs"), Some("src/x"));
        assert_eq!(modulo("a/b/c/d.txt"), Some("a/b/c"));
        assert_eq!(modulo("crates/tc2/src/lib.rs"), Some("crates/tc2/src"));
        // La raiz no tiene modulo.
        assert_eq!(modulo("Cargo.toml"), None);
        assert_eq!(modulo("README.md"), None);
        assert_eq!(modulo("/raro"), None);
        // `\` no separa: git escribe siempre `/`.
        assert_eq!(modulo("src\\x\\a.rs"), None);
        assert_eq!(modulo("src/x\\a.rs"), Some("src"));
    }

    /// C1, C2, C3 y C8 (unitarios).
    #[test]
    fn cruzar_por_fichero_por_modulo_o_nada() {
        let propia = conjunto(&["src/x/a.rs"]);
        // C1: el mismo fichero; su modulo no se repite como cruce de modulo.
        let c = cruzar(&propia, &[toca("TASK-001", &["src/x/a.rs", "src/x/c.rs"])]);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].ficheros, ["src/x/a.rs"]);
        assert!(c[0].modulos.is_empty(), "{c:?}");
        // C2: otro fichero en la misma carpeta.
        let c = cruzar(&propia, &[toca("TASK-001", &["src/x/b.rs"])]);
        assert_eq!(c[0].ficheros, Vec::<String>::new());
        assert_eq!(c[0].modulos, ["src/x"]);
        // C3: nada en comun.
        assert!(cruzar(&propia, &[toca("TASK-001", &["web/y/c.ts"])]).is_empty());
        // Una carpeta hija no es el mismo modulo.
        assert!(cruzar(&propia, &[toca("TASK-001", &["src/x/z/b.rs"])]).is_empty());
        // C8: en la raiz no hay modulo; el mismo fichero si cruza.
        let raiz = conjunto(&["Cargo.toml"]);
        assert!(cruzar(&raiz, &[toca("TASK-001", &["README.md"])]).is_empty());
        let c = cruzar(&raiz, &[toca("TASK-001", &["Cargo.toml"])]);
        assert_eq!(c[0].ficheros, ["Cargo.toml"]);
        assert!(c[0].modulos.is_empty());
        // Sin ficheros propios, nada.
        assert!(cruzar(&BTreeSet::new(), &[toca("TASK-001", &["src/x/a.rs"])]).is_empty());
    }

    /// Varias tareas: salen por ID, solo las que cruzan, cada una con lo
    /// suyo; un modulo con fichero comun no sale como modulo, otro si.
    #[test]
    fn cruzar_varias_en_orden_estable() {
        let propia = conjunto(&["src/x/a.rs", "src/y/b.rs", "docs/c.md"]);
        let otras = [
            toca("TASK-009", &["src/y/otro.rs"]),
            toca("TASK-002", &["web/z.ts"]),
            toca("TASK-004", &["src/x/a.rs", "src/y/b.rs", "docs/d.md"]),
        ];
        let c = cruzar(&propia, &otras);
        let ids: Vec<&str> = c.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, ["TASK-004", "TASK-009"]);
        assert_eq!(c[0].ficheros, ["src/x/a.rs", "src/y/b.rs"]);
        assert_eq!(c[0].modulos, ["docs"]);
        assert_eq!(c[0].quien, "@task-004");
        assert_eq!(c[1].modulos, ["src/y"]);
        // Mismo resultado con las otras en otro orden.
        let mut al_reves = otras.to_vec();
        al_reves.reverse();
        assert_eq!(cruzar(&propia, &al_reves), c);
    }

    fn doc(id: &str, estado: &str, asignado: Option<Asignado>, rama: Option<&str>) -> Documento {
        let mut t = Tarea::nueva(id, "Una", "2026-11-01");
        t.estado = estado.to_owned();
        t.asignado_a = asignado;
        t.rama = rama.map(str::to_owned);
        Documento::Formato2(t, Cuerpo::plantilla())
    }

    /// D2a: cuentan en curso y en revision (C7: una terminada no), con quien
    /// las lleva como en las vistas y por ID.
    #[test]
    fn en_curso_filtra_por_estado() {
        let docs = [
            doc("TASK-003", "en-revision", None, Some("feature/task-003-x")),
            doc(
                "TASK-001",
                "en-curso",
                Asignado::persona(7, "ana"),
                Some("feature/task-001-x"),
            ),
            doc(
                "TASK-002",
                "terminada",
                Asignado::persona(7, "ana"),
                Some("feature/task-002-x"),
            ),
            doc("TASK-004", "planificada", None, None),
            doc("TASK-005", "en-diseno", None, None),
            doc("TASK-001", "en-curso", Asignado::persona(8, "beto"), None),
        ];
        let v = en_curso(docs.iter());
        assert_eq!(
            v,
            [
                EnCurso {
                    id: "TASK-001".to_owned(),
                    quien: "@ana".to_owned(),
                    rama: Some("feature/task-001-x".to_owned()),
                },
                EnCurso {
                    id: "TASK-003".to_owned(),
                    quien: "—".to_owned(),
                    rama: Some("feature/task-003-x".to_owned()),
                },
            ]
        );
    }
}
