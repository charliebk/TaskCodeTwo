//! `tc2 import` (F3-2, RF-02): alta en bloque desde un fichero de
//! planificacion, con dependencias e hito (el sprint, D2 de TASK-029).
//!
//! - [`fichero`] lee el fichero (puro: limites, formato, claves y ciclos).
//! - [`Importar`] es **una sola** [`Operacion`] sobre la transaccion de F2-3:
//!   un commit, un push y N IDs consecutivos (ADR F1-2, punto 3). Comparte
//!   con `new` el nucleo [`crate::alta`] (reserva, rama y parecidas).
//!
//! Idempotencia (D1): cada tarea importada guarda `lote: "<lote>/<clave>"`.
//! Al repetir el import se crean solo las entradas cuya clave no esta; las
//! que ya estan no se actualizan (un import no sincroniza) y salen en
//! `existentes=`. Si no falta ninguna, `sin-cambios` sin commit.
//!
//! Dependencias: una clave del fichero se resuelve al ID de su entrada (la
//! nueva o la ya importada); un `TASK-NNN` tiene que existir en el estado
//! fresco del intento. Una referencia que no existe da `uso` (2): el fallo
//! es del fichero, y no se escribe nada.

pub mod fichero;

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::alta::{self, Parecida};
use crate::contrato::{Informe, Resultado, Valor};
use crate::estado::{self, Cambio, Catalogo, Estado, Fallo, Operacion};
use crate::git;
use crate::i18n::Mensaje;
use crate::i18n::catalogo::cli as txt_cli;
use crate::i18n::catalogo::importar as txt;
use fichero::{EntradaLote, ErrorLote, Limites};

/// Nombre de la operacion y de la orden.
pub const NOMBRE: &str = "import";

/// Lo que hace (o haria) un import sobre un estado.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Plan {
    /// IDs que se crean, en el orden del fichero.
    pub creadas: Vec<String>,
    /// IDs de las entradas que ya estaban importadas, en el orden del
    /// fichero.
    pub existentes: Vec<String>,
    /// Tareas abiertas (de fuera del lote) con un titulo parecido.
    pub parecidas: Vec<Parecida>,
}

/// Alta en bloque de las entradas de un lote.
#[derive(Debug)]
pub struct Importar {
    lote: String,
    sprint: u32,
    entradas: Vec<EntradaLote>,
    fecha: String,
    ocupados: BTreeSet<u32>,
    plan: RefCell<Option<Plan>>,
}

impl Importar {
    /// Import de `entradas` con el nombre de lote `lote`, en `sprint`.
    pub fn nuevo(lote: &str, sprint: u32, entradas: Vec<EntradaLote>) -> Self {
        Self {
            lote: lote.to_owned(),
            sprint,
            entradas,
            fecha: alta::hoy(),
            ocupados: BTreeSet::new(),
            plan: RefCell::new(None),
        }
    }

    /// IDs con rama (`git::ids_con_rama`), como en `new` (F2-4b).
    #[must_use]
    pub fn con_ocupados(mut self, ocupados: BTreeSet<u32>) -> Self {
        self.ocupados = ocupados;
        self
    }

    /// El plan del ultimo intento.
    pub fn plan(&self) -> Option<Plan> {
        self.plan.borrow().clone()
    }

    fn lote_de(&self, e: &EntradaLote) -> String {
        format!("{}/{}", self.lote, e.clave)
    }
}

impl Operacion for Importar {
    fn nombre(&self) -> &str {
        NOMBRE
    }
    fn anotacion(&self) -> String {
        format!("{NOMBRE}|{}|{}", self.lote, self.entradas.len())
    }
    fn exige_red(&self) -> bool {
        // Reserva IDs: sin red falla, nunca queda pendiente (F1-2 D2b).
        true
    }
    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        let rutas = estado.rutas();
        let tareas = alta::tareas(estado, &rutas)?;
        let prefijo = format!("{}/", self.lote);
        let del_lote = |t: &crate::tarea::Tarea| {
            t.lote
                .as_deref()
                .and_then(|l| l.strip_prefix(&prefijo))
                .map(str::to_owned)
        };
        // Clave → ID de lo ya importado de este lote (la primera si hubiera
        // dos con la misma clave, puestas a mano).
        let mut mapa: BTreeMap<String, String> = BTreeMap::new();
        for t in &tareas {
            if let Some(c) = del_lote(t) {
                mapa.entry(c).or_insert_with(|| t.id.clone());
            }
        }
        let (existentes, faltan): (Vec<&EntradaLote>, Vec<&EntradaLote>) = self
            .entradas
            .iter()
            .partition(|e| mapa.contains_key(&e.clave));
        let existentes: Vec<String> = existentes
            .iter()
            .filter_map(|e| mapa.get(&e.clave).cloned())
            .collect();
        let ids = alta::reservar(&rutas, &self.ocupados, faltan.len())?;
        for (e, id) in faltan.iter().zip(&ids) {
            mapa.insert(e.clave.clone(), alta::formato_id(*id));
        }
        let mut entradas = Vec::with_capacity(faltan.len());
        for e in &faltan {
            let mut en = e.entrada(self.sprint);
            en.lote = Some(self.lote_de(e));
            for d in &e.dependencias {
                let id = match mapa.get(d) {
                    Some(id) => id.clone(),
                    None if rutas.iter().any(|r| *r == format!("{d}/tarea.md")) => d.clone(),
                    None => {
                        return Err(Fallo::rechazada(
                            "referencia-inexistente",
                            Mensaje::nuevo(&txt::EXTERNA_INEXISTENTE)
                                .con("referencia", d.as_str())
                                .con("clave", e.clave.as_str()),
                        ));
                    }
                };
                en.dependencias.push(id);
            }
            entradas.push(en);
        }
        // Las del propio lote no cuentan como parecidas: son el mismo plan.
        let fuera: Vec<crate::tarea::Tarea> = tareas
            .into_iter()
            .filter(|t| del_lote(t).is_none())
            .collect();
        let a = alta::altas(&fuera, &ids, &entradas, &self.fecha)?;
        *self.plan.borrow_mut() = Some(Plan {
            creadas: ids.iter().map(|n| alta::formato_id(*n)).collect(),
            existentes,
            parecidas: a.parecidas,
        });
        Ok(a.cambios)
    }
}

/// Opciones de la orden (sin `--json`).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Opciones {
    sprint: Option<u32>,
    fichero: String,
}

fn uso(motivo: &str) -> Informe {
    Informe::nuevo(NOMBRE, Resultado::Uso)
        .con("motivo", motivo)
        .diagnostico(&txt::USO_IMPORT)
}

fn opciones(args: &[String]) -> Result<Opciones, Informe> {
    let mut sprint = None;
    let mut fichero = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--sprint" if sprint.is_none() => {
                let v = it.next().ok_or_else(|| uso("sprint-invalido"))?;
                let n = v
                    .bytes()
                    .all(|b| b.is_ascii_digit())
                    .then(|| v.parse::<u32>().ok())
                    .flatten()
                    .ok_or_else(|| uso("sprint-invalido"))?;
                sprint = Some(n);
            }
            f if !f.starts_with("--") && fichero.is_none() => fichero = Some(f.to_owned()),
            _ => return Err(uso("argumentos")),
        }
    }
    let fichero = fichero.ok_or_else(|| uso("sin-fichero"))?;
    Ok(Opciones { sprint, fichero })
}

fn de_error(e: ErrorLote) -> Informe {
    let mut i = Informe::nuevo(NOMBRE, Resultado::Uso).con("motivo", e.motivo);
    for (k, v) in e.claves {
        i = i.con(k, v);
    }
    i.diagnostico(e.mensaje)
}

/// Lee el fichero con el tope de bytes antes de cargarlo entero.
fn leer_fichero(ruta: &Path, limites: Limites) -> Result<String, Informe> {
    let no_se_lee = |e: std::io::Error| {
        Informe::nuevo(NOMBRE, Resultado::Uso)
            .con("motivo", "no-se-lee")
            .diagnostico(
                Mensaje::nuevo(&txt::NO_SE_LEE)
                    .con("ruta", ruta.display().to_string())
                    .con("error", e.to_string()),
            )
    };
    let m = std::fs::metadata(ruta).map_err(no_se_lee)?;
    if usize::try_from(m.len()).map_or(true, |n| n > limites.bytes) {
        return Err(de_error(fichero::grande(limites)));
    }
    let bytes = std::fs::read(ruta).map_err(no_se_lee)?;
    String::from_utf8(bytes).map_err(|_| {
        Informe::nuevo(NOMBRE, Resultado::Uso)
            .con("motivo", "no-utf8")
            .diagnostico(Mensaje::nuevo(&txt::NO_UTF8).con("ruta", ruta.display().to_string()))
    })
}

/// `import [--simular] [--sprint N] <fichero>` en la carpeta actual. Las
/// tareas abiertas parecidas quedan en `parecidas` para el aviso.
pub fn orden(args: &[String], catalogo: Catalogo<'_>, parecidas: &mut Vec<Parecida>) -> Informe {
    let o = match opciones(args) {
        Ok(o) => o,
        Err(i) => return i,
    };
    let dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            return Informe::nuevo(NOMBRE, Resultado::Error)
                .diagnostico(Mensaje::nuevo(&txt_cli::CARPETA_ACTUAL).con("error", e.to_string()));
        }
    };
    let g = git::Git::nuevo();
    importar_en(&g, &dir, &o, catalogo, parecidas)
}

fn importar_en(
    g: &git::Git,
    dir: &Path,
    o: &Opciones,
    catalogo: Catalogo<'_>,
    parecidas: &mut Vec<Parecida>,
) -> Informe {
    let limites = Limites::default();
    let ruta = dir.join(&o.fichero);
    let texto = match leer_fichero(&ruta, limites) {
        Ok(t) => t,
        Err(i) => return i,
    };
    let lote = match fichero::leer(&texto, limites) {
        Ok(l) => l,
        Err(e) => return de_error(e),
    };
    let sprint = match (lote.sprint, o.sprint) {
        (Some(c), Some(f)) if c != f => {
            return Informe::nuevo(NOMBRE, Resultado::Uso)
                .con("motivo", "sprint-en-conflicto")
                .diagnostico(
                    Mensaje::nuevo(&txt::SPRINT_EN_CONFLICTO)
                        .con("cabecera", c.to_string())
                        .con("opcion", f.to_string()),
                );
        }
        (c, f) => c.or(f).unwrap_or(0),
    };
    // Sin `lote:` en la cabecera, el nombre del fichero (en slug).
    let nombre = lote.nombre.clone().unwrap_or_else(|| {
        let tallo = ruta
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        crate::tarea::texto::slug(&tallo)
    });
    // Las ramas `*/task-*` una vez, antes de la transaccion (F2-4b).
    let ocupados = match git::ids_con_rama(g, dir, NOMBRE) {
        Ok(x) => x,
        Err(mut i) => {
            NOMBRE.clone_into(&mut i.comando);
            if i.resultado == Resultado::Sinred {
                i = i.con("pendiente", "no");
            }
            return i;
        }
    };
    let op = Importar::nuevo(&nombre, sprint, lote.entradas).con_ocupados(ocupados);
    let c = estado::Contexto::nuevo(g, dir, catalogo);
    let mut i = estado::transaccion(&c, &op);
    NOMBRE.clone_into(&mut i.comando);
    if i.resultado == Resultado::Rechazada && i.valor("motivo") == Some("referencia-inexistente") {
        // El fallo es del fichero, no del estado: uso (C2).
        i.resultado = Resultado::Uso;
    }
    if i.resultado.es_exito()
        && let Some(p) = op.plan()
    {
        i = con_plan(i, &p, parecidas);
    }
    i
}

/// Pone `creadas=`, `existentes=`, `ids=` (y `parecidas=`) al principio.
fn con_plan(mut i: Informe, p: &Plan, parecidas: &mut Vec<Parecida>) -> Informe {
    let mut delante = vec![
        (
            "creadas".to_owned(),
            Valor::Texto(p.creadas.len().to_string()),
        ),
        (
            "existentes".to_owned(),
            Valor::Texto(p.existentes.len().to_string()),
        ),
    ];
    if !p.creadas.is_empty() {
        delante.push(("ids".to_owned(), Valor::Lista(p.creadas.clone())));
    }
    if !p.parecidas.is_empty() {
        let ids = p.parecidas.iter().map(|x| x.id.clone()).collect();
        delante.push(("parecidas".to_owned(), Valor::Lista(ids)));
    }
    parecidas.clone_from(&p.parecidas);
    delante.append(&mut i.claves);
    i.claves = delante;
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn opciones_de_la_orden() {
        assert_eq!(
            opciones(&args(&["f.md", "--sprint", "3"])),
            Ok(Opciones {
                sprint: Some(3),
                fichero: "f.md".to_owned()
            })
        );
        for (a, m) in [
            (vec![], "sin-fichero"),
            (vec!["a.md", "b.md"], "argumentos"),
            (vec!["a.md", "--sprint", "1", "--sprint", "1"], "argumentos"),
            (vec!["a.md", "--otra"], "argumentos"),
            (vec!["a.md", "--sprint"], "sprint-invalido"),
            (vec!["a.md", "--sprint", "-1"], "sprint-invalido"),
            (vec!["a.md", "--sprint", "x"], "sprint-invalido"),
        ] {
            let r = opciones(&args(&a));
            assert_eq!(
                r.err().and_then(|i| i.valor("motivo").map(str::to_owned)),
                Some(m.to_owned()),
                "{a:?}"
            );
        }
    }

    #[test]
    fn anotacion_en_una_linea() {
        let op = Importar::nuevo("fase-3", 3, Vec::new());
        assert_eq!(op.anotacion(), "import|fase-3|0");
        assert!(op.exige_red());
    }
}
