//! Situacion del trabajo sobre un estado (F3-3 y F3-4): las tareas leidas,
//! el orden de trabajo, las listas para empezar y las en marcha. La usan
//! `next` y `roadmap`, que coinciden por construccion (TASK-031 C4).
//!
//! Lee todos los `tarea.md` y el `ROADMAP.md` de la rama de estado con un
//! solo [`Estado::leer_varios`] (un `git cat-file --batch` sobre una
//! [`crate::estado::Vista`]) y calcula el orden con [`grafo`].

use crate::estado::{Estado, Fallo, es_tarea_md};
use crate::grafo::{self, Hitos, Nodo};
use crate::roadmap::zona::{self, Rota};
use crate::tarea::{Asignado, Documento};

pub use crate::estado::ROADMAP;

/// Lo que se muestra de una tarea legible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ficha {
    /// Lo que cuenta para el orden.
    pub nodo: Nodo,
    /// Titulo.
    pub titulo: String,
    /// Quien la lleva (en formato 1, siempre texto heredado).
    pub asignado: Option<Asignado>,
}

impl Ficha {
    /// Ficha de un `tarea.md` leido.
    pub fn de(doc: &Documento) -> Self {
        let (titulo, asignado) = match doc {
            Documento::Formato1(t, _) => (t.titulo.clone(), t.asignado_a.clone()),
            Documento::Formato2(t, _) => (t.titulo.clone(), t.asignado_a.clone()),
        };
        Self {
            nodo: Nodo::de(doc),
            titulo,
            asignado,
        }
    }
}

/// Todo lo que se calcula sobre un estado.
#[derive(Debug, Clone, Default)]
pub struct Situacion {
    /// Las tareas legibles, en orden de ruta.
    pub fichas: Vec<Ficha>,
    /// Los nodos del orden: los de las fichas y, detras, los ilegibles.
    pub nodos: Vec<Nodo>,
    /// Carpetas de los `tarea.md` que no se entienden.
    pub ilegibles: Vec<String>,
    /// Orden de los hitos (zona manual del ROADMAP, lectura tolerante).
    pub hitos: Hitos,
    /// Orden de trabajo, atascadas y avisos.
    pub orden: grafo::Orden,
    /// Listas para empezar, en el orden de trabajo.
    pub listas: Vec<String>,
    /// En diseno, en curso o en revision, en el orden de trabajo.
    pub en_marcha: Vec<String>,
    /// El `ROADMAP.md` de la rama de estado, si lo hay.
    pub roadmap: Option<String>,
}

impl Situacion {
    /// Si la zona manual del ROADMAP esta rota (D3 de TASK-031).
    pub fn roadmap_roto(&self) -> Option<Rota> {
        self.roadmap.as_deref().and_then(|t| zona::extraer(t).err())
    }
}

/// La situacion de `estado`.
pub fn calcular(estado: &dyn Estado) -> Result<Situacion, Fallo> {
    let mut rutas: Vec<String> = estado
        .rutas()
        .into_iter()
        .filter(|r| es_tarea_md(r))
        .collect();
    rutas.push(ROADMAP.to_owned());
    let mut textos = estado.leer_varios(&rutas)?;
    let roadmap = textos.pop().flatten();
    rutas.pop();
    let mut fichas = Vec::new();
    let mut ilegibles = Vec::new();
    for (ruta, texto) in rutas.iter().zip(textos) {
        let Some(texto) = texto else { continue };
        match crate::tarea::leer(&texto) {
            Ok(d) => fichas.push(Ficha::de(&d)),
            Err(_) => ilegibles.push(ruta.split('/').next().unwrap_or(ruta).to_owned()),
        }
    }
    let mut nodos: Vec<Nodo> = fichas.iter().map(|f| f.nodo.clone()).collect();
    // Detras de las legibles: si un ID se repite, vale la legible. Una
    // ilegible bloquea a quien dependa de ella.
    nodos.extend(ilegibles.iter().map(|id| Nodo::ilegible(id)));
    let hitos = roadmap
        .as_deref()
        .map(Hitos::del_roadmap)
        .unwrap_or_default();
    let orden = grafo::orden_de_trabajo(&nodos, &hitos);
    Ok(Situacion {
        listas: grafo::listas(&orden, &nodos),
        en_marcha: grafo::en_marcha(&orden, &nodos),
        fichas,
        nodos,
        ilegibles,
        hitos,
        orden,
        roadmap,
    })
}
