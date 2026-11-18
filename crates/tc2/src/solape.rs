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
//!
//! [`calcular`] corre tras el `fetch` de `start` y sin red: un
//! `for-each-ref` de `refs/remotes/origin/`, el `diff` de la rama propia y
//! uno por cada otra tarea con rama publicada (1 + k + 1 procesos de git,
//! con k como mucho [`MAX_DIFFS`]).

use std::collections::BTreeSet;
use std::path::Path;

use crate::contrato::Informe;
use crate::git::{self, Git};
use crate::i18n::catalogo::toma as txt;
use crate::i18n::{Mensaje, Texto};
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

/// Nombre de la operacion en los informes de error de git (no se ven: un
/// fallo del solape solo marca `solape-incompleto`).
const OP: &str = "solape";

/// Elementos como mucho de cada lista de la linea (D5a de TASK-082); si se
/// corta alguna, `solape-truncado=si`.
pub const MAX_LISTA: usize = 20;

/// Modulos como mucho por tarea en `en-curso-modulos` y en la nota (D3b).
pub const MAX_MODULOS_NOTA: usize = 5;

/// Otras tareas como mucho cuyo `diff` se lee (H3 de la ronda 1): las
/// publicadas de mas (por ID) no lanzan `git diff` y la linea lleva
/// `solape-truncado=si`. Asi el solape son como mucho 1 + 20 + 1 procesos.
pub const MAX_DIFFS: usize = 20;

/// Lineas `aviso:` como mucho en stderr; el resto, una linea final «y N
/// mas» (H3). La linea de `start` las lleva todas, hasta [`MAX_LISTA`].
pub const MAX_AVISOS: usize = 5;

/// Tareas como mucho en la nota del primer `start`; el resto, «y N mas».
pub const MAX_TAREAS_NOTA: usize = 5;

/// La linea final de los avisos que no caben en [`MAX_AVISOS`]. Vive aqui
/// y no en el catalogo de `toma` porque la correccion de H3 solo podia
/// tocar este modulo.
pub static AVISO_Y_MAS: Texto = Texto {
    clave: "tc2::solape::AVISO_Y_MAS",
    es: "aviso: y {n} tareas mas tocan lo mismo (todas en la clave solape)",
    en: "warning: and {n} more tasks touch the same (all of them in the solape key)",
};

/// La cola de la nota cuando no caben todas las tareas
/// ([`MAX_TAREAS_NOTA`]).
pub static NOTA_Y_MAS: Texto = Texto {
    clave: "tc2::solape::NOTA_Y_MAS",
    es: "{tareas}; y {n} mas",
    en: "{tareas}; and {n} more",
};

/// Lo que sale del solape de una tarea.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Solape {
    /// Las otras tareas que tocan lo mismo, por ID.
    pub cruces: Vec<Cruce>,
    /// Las otras tareas con la rama publicada y su diff leido, por ID.
    pub publicadas: Vec<Toca>,
    /// Las otras tareas sin `origin/<rama>` (o sin `rama:`): no se sabe que
    /// tocan.
    pub sin_rama: Vec<String>,
    /// Las tareas cuyo `diff` fallo (la propia incluida): su solape no se
    /// sabe.
    pub incompletas: Vec<String>,
    /// La rama propia aun no cambia nada (primer `start`, D3b): en vez de
    /// cruces, la nota con lo que tocan las demas.
    pub propia_vacia: bool,
    /// Habia mas de [`MAX_DIFFS`] tareas con la rama publicada: las de mas
    /// no se leyeron (sin `diff`) y la linea lleva `solape-truncado=si`.
    pub truncado: bool,
}

/// El solape de la tarea `id` (rama `rama`, ya creada en local) con las
/// tareas `en_curso`, desde la carpeta `dir` del repo. Sin red: lo que
/// sabe el clon tras el `fetch` de `start`.
///
/// `None` (ninguna clave) si no hay otra tarea en curso o el clon no conoce
/// ninguna rama de `origin` (sin remoto o sin haberlo traido nunca). Un
/// `diff` que falla no rompe nada: la tarea queda en `incompletas`.
pub fn calcular(g: &Git, dir: &Path, id: &str, rama: &str, en_curso: &[EnCurso]) -> Option<Solape> {
    let otras: Vec<&EnCurso> = en_curso.iter().filter(|t| t.id != id).collect();
    if otras.is_empty() {
        return None;
    }
    let mut s = Solape::default();
    let remotas = match git::fechas_de_ramas(g, dir, OP) {
        Ok(r) if r.is_empty() => return None,
        Ok(r) => r,
        Err(_) => {
            s.incompletas = otras.iter().map(|t| t.id.clone()).collect();
            return Some(s);
        }
    };
    let propia = git::ficheros_de_rama(
        g,
        dir,
        OP,
        git::base_de_rama(rama),
        &format!("refs/heads/{rama}"),
    )
    .ok();
    if propia.is_none() {
        s.incompletas.push(id.to_owned());
    }
    let mut diffs = 0;
    for t in otras {
        let Some(r) = t.rama.as_deref().filter(|r| remotas.contains_key(*r)) else {
            s.sin_rama.push(t.id.clone());
            continue;
        };
        if diffs == MAX_DIFFS {
            s.truncado = true;
            continue;
        }
        diffs += 1;
        let remota = format!("refs/remotes/{}/{r}", git::REMOTO);
        match git::ficheros_de_rama(g, dir, OP, git::base_de_rama(r), &remota) {
            Ok(ficheros) => s.publicadas.push(Toca {
                id: t.id.clone(),
                quien: t.quien.clone(),
                ficheros,
            }),
            Err(_) => s.incompletas.push(t.id.clone()),
        }
    }
    s.incompletas.sort();
    match propia {
        Some(p) if p.is_empty() => s.propia_vacia = true,
        Some(p) => s.cruces = cruzar(&p, &s.publicadas),
        None => {}
    }
    Some(s)
}

/// `TASK-NNN:<valor>`: un elemento de una lista de la linea. `:` no se
/// escapa en un valor de lista (F1-4, decision 4: solo espacio, `%`,
/// controles y `,`) y el ID no lo lleva, asi que el primer `:` separa.
fn de(id: &str, valor: &str) -> String {
    format!("{id}:{valor}")
}

/// Anade la lista `clave` si no esta vacia, cortada a [`MAX_LISTA`].
fn lista(i: Informe, clave: &str, mut v: Vec<String>, truncado: &mut bool) -> Informe {
    if v.is_empty() {
        return i;
    }
    if v.len() > MAX_LISTA {
        v.truncate(MAX_LISTA);
        *truncado = true;
    }
    i.con_lista(clave, v)
}

/// Los modulos de una tarea para la nota, como mucho [`MAX_MODULOS_NOTA`].
fn modulos_nota(t: &Toca) -> Vec<&str> {
    modulos(&t.ficheros)
        .into_iter()
        .take(MAX_MODULOS_NOTA)
        .collect()
}

impl Solape {
    /// Las claves del solape en la linea de `start` (D5a de TASK-082):
    /// `solape`, `solape-ficheros`, `solape-modulos` y `solape-asignados`
    /// si cruza con alguna; `en-curso-modulos` en el primer `start` (D3b);
    /// `solape-sin-rama` y `solape-incompleto` si las hay; y
    /// `solape-truncado=si` si se corto alguna lista. Una lista vacia no
    /// sale.
    pub fn con_claves(&self, mut i: Informe) -> Informe {
        let mut t = false;
        let c = &self.cruces;
        i = lista(
            i,
            "solape",
            c.iter().map(|c| c.id.clone()).collect(),
            &mut t,
        );
        let ficheros = c
            .iter()
            .flat_map(|c| c.ficheros.iter().map(|f| de(&c.id, f)))
            .collect();
        i = lista(i, "solape-ficheros", ficheros, &mut t);
        let modulos = c
            .iter()
            .flat_map(|c| c.modulos.iter().map(|m| de(&c.id, m)))
            .collect();
        i = lista(i, "solape-modulos", modulos, &mut t);
        let asignados = c.iter().map(|c| de(&c.id, &c.quien)).collect();
        i = lista(i, "solape-asignados", asignados, &mut t);
        if self.propia_vacia {
            let en_curso = self
                .publicadas
                .iter()
                .flat_map(|p| modulos_nota(p).into_iter().map(|m| de(&p.id, m)))
                .collect();
            i = lista(i, "en-curso-modulos", en_curso, &mut t);
        }
        i = lista(i, "solape-sin-rama", self.sin_rama.clone(), &mut t);
        i = lista(i, "solape-incompleto", self.incompletas.clone(), &mut t);
        if t || self.truncado {
            i.con("solape-truncado", "si")
        } else {
            i
        }
    }

    /// Lo que se dice a la persona por stderr (D3b y D4a de TASK-082): una
    /// linea `aviso:` por tarea que cruza (como mucho [`MAX_AVISOS`] y una
    /// final «y N mas») y, en el primer `start`, una `nota:` con las tareas
    /// en curso con rama publicada (como mucho [`MAX_TAREAS_NOTA`] y «y N
    /// mas»), su @ y sus modulos. Login, rutas e IDs son texto ajeno: van
    /// saneados (F1-17, punto 8). Sin nada que decir, vacio.
    pub fn avisos(&self) -> Vec<Mensaje> {
        let mut v: Vec<Mensaje> = self
            .cruces
            .iter()
            .take(MAX_AVISOS)
            .map(|c| {
                let ficheros = resumen(c.ficheros.iter().map(String::as_str));
                let modulos = resumen(c.modulos.iter().map(String::as_str));
                let que = match (c.ficheros.is_empty(), c.modulos.is_empty()) {
                    (false, true) => {
                        Mensaje::nuevo(&txt::SOLAPE_FICHEROS).con("ficheros", ficheros)
                    }
                    (true, _) => Mensaje::nuevo(&txt::SOLAPE_MODULOS).con("modulos", modulos),
                    (false, false) => Mensaje::nuevo(&txt::SOLAPE_FICHEROS_Y_MODULOS)
                        .con("ficheros", ficheros)
                        .con("modulos", modulos),
                };
                Mensaje::nuevo(&txt::AVISO_SOLAPE)
                    .con("id", sanear(&c.id, MAX_ID))
                    .con("quien", sanear(&c.quien, MAX_QUIEN))
                    .con_mensaje("que", que)
            })
            .collect();
        match self.cruces.len().saturating_sub(MAX_AVISOS) {
            0 => {}
            n => v.push(Mensaje::nuevo(&AVISO_Y_MAS).con("n", n.to_string())),
        }
        if self.propia_vacia && !self.publicadas.is_empty() {
            let tareas: Vec<String> = self
                .publicadas
                .iter()
                .take(MAX_TAREAS_NOTA)
                .map(|p| {
                    let id = sanear(&p.id, MAX_ID);
                    let quien = sanear(&p.quien, MAX_QUIEN);
                    match modulos_nota(p) {
                        m if m.is_empty() => format!("{id} ({quien})"),
                        m => format!("{id} ({quien}: {})", resumen(m.into_iter())),
                    }
                })
                .collect();
            let tareas = tareas.join("; ");
            let nota = Mensaje::nuevo(&txt::NOTA_EN_CURSO);
            v.push(
                match self.publicadas.len().saturating_sub(MAX_TAREAS_NOTA) {
                    0 => nota.con("tareas", tareas),
                    n => nota.con_mensaje(
                        "tareas",
                        Mensaje::nuevo(&NOTA_Y_MAS)
                            .con("tareas", tareas)
                            .con("n", n.to_string()),
                    ),
                },
            );
        }
        v
    }
}

/// Caracteres como mucho de un ID, un quien y una ruta en el aviso.
const MAX_ID: usize = 20;
const MAX_QUIEN: usize = 40;
const MAX_RUTA: usize = 120;

/// Elementos como mucho de una lista en el aviso; el resto, `+N`.
const MAX_EN_AVISO: usize = 5;

fn sanear(t: &str, max: usize) -> String {
    crate::tablero::sanear(t, max)
}

/// Las rutas de un aviso: saneadas, separadas por comas y, pasadas
/// [`MAX_EN_AVISO`], `+N`.
fn resumen<'a>(v: impl Iterator<Item = &'a str>) -> String {
    let todas: Vec<&str> = v.collect();
    let mut s: Vec<String> = todas
        .iter()
        .take(MAX_EN_AVISO)
        .map(|r| sanear(r, MAX_RUTA))
        .collect();
    if todas.len() > MAX_EN_AVISO {
        s.push(format!("+{}", todas.len() - MAX_EN_AVISO));
    }
    s.join(", ")
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

    fn cruce(id: &str, quien: &str, ficheros: &[&str], modulos: &[&str]) -> Cruce {
        Cruce {
            id: id.to_owned(),
            quien: quien.to_owned(),
            ficheros: ficheros.iter().map(|s| (*s).to_owned()).collect(),
            modulos: modulos.iter().map(|s| (*s).to_owned()).collect(),
        }
    }

    fn linea(s: &Solape) -> String {
        s.con_claves(Informe::nuevo("start", crate::contrato::Resultado::Ok))
            .linea()
    }

    /// C1 y C2 (unitarios): las claves planas `TASK-NNN:valor`, solo las
    /// listas con algo; sin cruces ni nada mas, ninguna clave (C3).
    #[test]
    fn claves_del_solape() {
        let s = Solape {
            cruces: vec![
                cruce("TASK-001", "@ana", &["src/x/a.rs"], &[]),
                cruce("TASK-003", "@beto", &[], &["web/y"]),
            ],
            sin_rama: vec!["TASK-004".to_owned()],
            ..Solape::default()
        };
        assert_eq!(
            linea(&s),
            "start ok solape=TASK-001,TASK-003 solape-ficheros=TASK-001:src/x/a.rs \
             solape-modulos=TASK-003:web/y solape-asignados=TASK-001:@ana,TASK-003:@beto \
             solape-sin-rama=TASK-004"
        );
        assert_eq!(linea(&Solape::default()), "start ok");
        // `en-curso-modulos` solo con la rama propia vacia (D3b), como
        // mucho 5 por tarea.
        let muchos: Vec<String> = (0..7).map(|n| format!("m{n}/f.rs")).collect();
        let p = Toca {
            id: "TASK-002".to_owned(),
            quien: "@luis".to_owned(),
            ficheros: muchos.into_iter().chain(["raiz.md".to_owned()]).collect(),
        };
        let mut s = Solape {
            publicadas: vec![p],
            ..Solape::default()
        };
        assert_eq!(linea(&s), "start ok");
        s.propia_vacia = true;
        assert_eq!(
            linea(&s),
            "start ok en-curso-modulos=TASK-002:m0,TASK-002:m1,TASK-002:m2,TASK-002:m3,TASK-002:m4"
        );
        let n = s.avisos();
        assert_eq!(n.len(), 1);
        assert_eq!(
            n[0].texto(crate::i18n::Idioma::Es),
            "nota: tu rama aun no cambia nada; en curso con rama publicada: \
             TASK-002 (@luis: m0, m1, m2, m3, m4)"
        );
    }

    /// D5a: cada lista, como mucho 20, y `solape-truncado=si` si se corta.
    #[test]
    fn listas_cortadas_a_veinte() {
        let ficheros: Vec<String> = (0..25).map(|n| format!("f{n:02}.rs")).collect();
        let refs: Vec<&str> = ficheros.iter().map(String::as_str).collect();
        let s = Solape {
            cruces: vec![cruce("TASK-001", "@ana", &refs, &[])],
            ..Solape::default()
        };
        let i = s.con_claves(Informe::nuevo("start", crate::contrato::Resultado::Ok));
        let l = i.linea();
        let lista = l
            .split(' ')
            .find_map(|kv| kv.strip_prefix("solape-ficheros="))
            .unwrap_or_default();
        assert_eq!(lista.split(',').count(), MAX_LISTA, "{l}");
        assert_eq!(i.valor("solape-truncado"), Some("si"), "{l}");
        let corta = Solape {
            cruces: vec![cruce("TASK-001", "@ana", &refs[..MAX_LISTA], &[])],
            ..Solape::default()
        };
        assert_eq!(linea(&corta).find("solape-truncado"), None);
        // En el aviso, 5 y `+N`.
        let a = s.avisos()[0].texto(crate::i18n::Idioma::Es);
        assert!(
            a.contains("(f00.rs, f01.rs, f02.rs, f03.rs, f04.rs, +20)"),
            "{a}"
        );
    }

    /// H3: con muchas tareas, stderr no crece sin limite: 5 lineas `aviso:`
    /// y una final «y N mas»; la nota, 5 tareas y «y N mas». La linea sigue
    /// cortada a 20 y `truncado` (diffs no lanzados) da `solape-truncado`.
    #[test]
    fn avisos_y_nota_con_tope() {
        let ids: Vec<String> = (1..=25).map(|n| format!("TASK-{n:03}")).collect();
        let s = Solape {
            cruces: ids
                .iter()
                .map(|id| cruce(id, "@ana", &["src/x/a.rs"], &[]))
                .collect(),
            ..Solape::default()
        };
        let a = s.avisos();
        assert_eq!(a.len(), MAX_AVISOS + 1);
        assert!(
            a[..MAX_AVISOS]
                .iter()
                .all(|m| m.clave().ends_with("AVISO_SOLAPE"))
        );
        assert_eq!(
            a[MAX_AVISOS].texto(crate::i18n::Idioma::Es),
            "aviso: y 20 tareas mas tocan lo mismo (todas en la clave solape)"
        );
        assert_eq!(
            a[MAX_AVISOS].texto(crate::i18n::Idioma::En),
            "warning: and 20 more tasks touch the same (all of them in the solape key)"
        );
        // Justo en el tope, sin la linea final.
        let justo = Solape {
            cruces: s.cruces[..MAX_AVISOS].to_vec(),
            ..Solape::default()
        };
        assert_eq!(justo.avisos().len(), MAX_AVISOS);
        // La nota del primer `start`.
        let nota = Solape {
            publicadas: ids.iter().map(|id| toca(id, &["m/a.rs"])).collect(),
            propia_vacia: true,
            ..Solape::default()
        };
        let n = nota.avisos();
        assert_eq!(n.len(), 1);
        assert_eq!(
            n[0].texto(crate::i18n::Idioma::Es),
            "nota: tu rama aun no cambia nada; en curso con rama publicada: \
             TASK-001 (@task-001: m); TASK-002 (@task-002: m); TASK-003 (@task-003: m); \
             TASK-004 (@task-004: m); TASK-005 (@task-005: m); y 20 mas"
        );
        assert!(
            n[0].texto(crate::i18n::Idioma::En)
                .ends_with("; and 20 more")
        );
        let pocas = Solape {
            publicadas: nota.publicadas[..MAX_TAREAS_NOTA].to_vec(),
            propia_vacia: true,
            ..Solape::default()
        };
        assert!(
            pocas.avisos()[0]
                .texto(crate::i18n::Idioma::Es)
                .ends_with("(@task-005: m)")
        );
        // Diffs no lanzados: `solape-truncado=si` aunque ninguna lista se corte.
        let t = Solape {
            truncado: true,
            ..Solape::default()
        };
        assert_eq!(linea(&t), "start ok solape-truncado=si");
    }

    /// C1 (unitario) y C11: el aviso nombra la tarea, quien la lleva y lo
    /// que toca; un login, una ruta o un ID con caracteres de control (o de
    /// formato) salen saneados, en los dos idiomas. En la linea, el contrato
    /// los escapa `%XX`.
    #[test]
    fn avisos_saneados() {
        let s = Solape {
            cruces: vec![
                cruce("TASK-001", "@ana", &["src/x/a.rs"], &[]),
                cruce(
                    "TASK-00\u{1b}[31m2",
                    "@lu\u{7}is\u{202e}",
                    &["src/\u{1b}]8;;x\u{7}b.rs"],
                    &["web/\r\ny"],
                ),
            ],
            ..Solape::default()
        };
        let a = s.avisos();
        assert_eq!(a.len(), 2);
        let es = a[0].texto(crate::i18n::Idioma::Es);
        assert_eq!(
            es,
            "aviso: TASK-001 (@ana) toca los mismos ficheros (src/x/a.rs): hablad antes de \
             seguir (solo es un aviso; start ya se hizo)"
        );
        for idioma in [crate::i18n::Idioma::Es, crate::i18n::Idioma::En] {
            let t = a[1].texto(idioma);
            assert!(
                !t.chars().any(|c| c.is_control() || c == '\u{202e}'),
                "{t:?}"
            );
            assert!(t.contains("@lu is") && t.contains("web/ y"), "{t:?}");
        }
        let l = linea(&s);
        assert!(!l.chars().any(char::is_control), "{l:?}");
        assert!(l.contains("TASK-00%1B[31m2:@lu%07is"), "{l}");
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
