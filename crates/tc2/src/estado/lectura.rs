//! Lectura del estado: una vista sobre un commit de la rama de estado, con
//! los cambios que la transaccion va acumulando encima. La misma vista la
//! usan la transaccion (para que la operacion decida sobre el estado fresco)
//! y la lectura publica.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::transaccion::Fallo;
use super::{CARPETA, OP, RAMA};
use crate::contrato::{Informe, Resultado};
use crate::git::estado::{hay_remoto, leer_blob, leer_blobs, listar_arbol};
use crate::git::{Cambio, Git, REMOTO, localizar_principal};

/// Lo que una operacion puede leer del estado.
pub trait Estado {
    /// Rutas de todos los ficheros, ordenadas.
    fn rutas(&self) -> Vec<String>;
    /// Contenido de `ruta`, o `None` si no existe.
    fn leer(&self, ruta: &str) -> Result<Option<String>, Fallo>;
    /// Contenido de varias rutas, en el mismo orden (F2-4b). Por defecto,
    /// una a una con [`Estado::leer`]; [`Vista`] lo hace en un solo proceso.
    fn leer_varios(&self, rutas: &[String]) -> Result<Vec<Option<String>>, Fallo> {
        rutas.iter().map(|r| self.leer(r)).collect()
    }
}

/// Un fallo de lectura de git como [`Fallo::Error`]. Sin diagnostico queda
/// la linea de contrato, que no se traduce (ADR F1-4).
fn fallo_de_lectura(i: &Informe) -> Fallo {
    Fallo::Error(
        i.diagnostico
            .clone()
            .unwrap_or_else(|| crate::i18n::Mensaje::externo(i.linea())),
    )
}

/// Vista de un commit de la rama de estado (o del estado vacio si la rama no
/// existe todavia), con cambios superpuestos.
#[derive(Debug)]
pub struct Vista<'a> {
    git: &'a Git,
    dir: PathBuf,
    commit: Option<String>,
    rutas: BTreeSet<String>,
    encima: BTreeMap<String, String>,
}

impl<'a> Vista<'a> {
    /// Vista de `commit`, leyendo con `git` desde `dir`.
    pub(crate) fn nueva(git: &'a Git, dir: &Path, commit: Option<String>) -> Result<Self, Informe> {
        let rutas = match &commit {
            Some(c) => listar_arbol(git, dir, OP, c)?.into_iter().collect(),
            None => BTreeSet::new(),
        };
        Ok(Self {
            git,
            dir: dir.to_path_buf(),
            commit,
            rutas,
            encima: BTreeMap::new(),
        })
    }

    /// Commit sobre el que se lee (`None`: la rama no existe).
    pub fn commit(&self) -> Option<&str> {
        self.commit.as_deref()
    }

    /// Superpone cambios (los posteriores ganan).
    pub(crate) fn superponer(&mut self, cambios: Vec<Cambio>) {
        for c in cambios {
            self.encima.insert(c.ruta, c.contenido);
        }
    }

    /// Cambios superpuestos, en orden de ruta.
    pub(crate) fn cambios(&self) -> Vec<Cambio> {
        self.encima
            .iter()
            .map(|(r, c)| Cambio::nuevo(r.clone(), c.clone()))
            .collect()
    }
}

impl Estado for Vista<'_> {
    fn rutas(&self) -> Vec<String> {
        let mut v: BTreeSet<String> = self.rutas.clone();
        v.extend(self.encima.keys().cloned());
        v.into_iter().collect()
    }

    fn leer(&self, ruta: &str) -> Result<Option<String>, Fallo> {
        if let Some(c) = self.encima.get(ruta) {
            return Ok(Some(c.clone()));
        }
        match &self.commit {
            Some(c) if self.rutas.contains(ruta) => leer_blob(self.git, &self.dir, OP, c, ruta)
                .map(Some)
                .map_err(|i| fallo_de_lectura(&i)),
            _ => Ok(None),
        }
    }

    /// Las rutas del commit se leen con un solo `git cat-file --batch`; las
    /// superpuestas y las que no existen no lanzan nada.
    fn leer_varios(&self, rutas: &[String]) -> Result<Vec<Option<String>>, Fallo> {
        let del_commit: Vec<&str> = rutas
            .iter()
            .filter(|r| !self.encima.contains_key(*r) && self.rutas.contains(*r))
            .map(String::as_str)
            .collect();
        let mut leidos = match (&self.commit, del_commit.is_empty()) {
            (Some(c), false) => leer_blobs(self.git, &self.dir, OP, c, &del_commit)
                .map_err(|i| fallo_de_lectura(&i))?
                .into_iter(),
            _ => Vec::new().into_iter(),
        };
        Ok(rutas
            .iter()
            .map(|r| match self.encima.get(r) {
                Some(c) => Some(c.clone()),
                None if self.commit.is_some() && self.rutas.contains(r) => leidos.next().flatten(),
                None => None,
            })
            .collect())
    }
}

/// Vista del estado tal como lo conoce este clon: lo ultimo traido de
/// `origin/taskcodetwo/tareas` (sin `fetch`: la lectura fresca es de F3-5b) o,
/// en un repo sin remoto, la rama local. Sin rama de estado → `rechazada`.
pub fn leer<'a>(git: &'a Git, dir: &Path) -> Result<Vista<'a>, Informe> {
    let principal = localizar_principal(git, dir)?;
    let referencia = if hay_remoto(git, &principal, OP)? {
        format!("refs/remotes/{REMOTO}/{RAMA}")
    } else {
        format!("refs/heads/{RAMA}")
    };
    match crate::git::commit_de(git, &principal, OP, &referencia)? {
        Some(c) => Vista::nueva(git, &principal, Some(c)),
        None => Err(Informe::nuevo(OP, Resultado::Rechazada)
            .con("motivo", "sin-rama-de-estado")
            .diagnostico(
                crate::i18n::Mensaje::nuevo(&crate::i18n::catalogo::estado::MONTA_PRIMERO)
                    .con("carpeta", CARPETA),
            )),
    }
}
