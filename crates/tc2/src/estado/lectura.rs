//! Lectura del estado: una vista sobre un commit de la rama de estado, con
//! los cambios que la transaccion va acumulando encima. La misma vista la
//! usan la transaccion (para que la operacion decida sobre el estado fresco)
//! y la lectura publica.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::frescura::{Frescura, Marca, Remoto, Traida, traer_estado};
use super::transaccion::Fallo;
use super::{CARPETA, OP, RAMA};
use crate::contrato::{Informe, Resultado};
use crate::git::estado::{dir_comun, hay_remoto, leer_blob, leer_blobs, listar_arbol};
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

    /// Vista del estado vacio (la rama de estado aun no existe), para
    /// calcular un plan sin escribir (`import --simular`).
    pub fn vacia(git: &'a Git, dir: &Path) -> Self {
        Self {
            git,
            dir: dir.to_path_buf(),
            commit: None,
            rutas: BTreeSet::new(),
            encima: BTreeMap::new(),
        }
    }

    /// Commit sobre el que se lee (`None`: la rama no existe).
    pub fn commit(&self) -> Option<&str> {
        self.commit.as_deref()
    }

    /// Carpeta desde la que se lee (la principal del repo).
    pub(crate) fn dir(&self) -> &Path {
        &self.dir
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

/// La carpeta principal, la ref de la rama de estado que se lee y si hay
/// `origin`.
fn lectura_de(git: &Git, dir: &Path) -> Result<(PathBuf, String, bool), Informe> {
    let principal = localizar_principal(git, dir)?;
    let remoto = hay_remoto(git, &principal, OP)?;
    let referencia = if remoto {
        format!("refs/remotes/{REMOTO}/{RAMA}")
    } else {
        format!("refs/heads/{RAMA}")
    };
    Ok((principal, referencia, remoto))
}

/// La carpeta principal y la ref de la rama de estado que lee [`leer`]: la
/// remota si hay `origin`, la local si no. `doctor` (F3-8) decide sobre la
/// misma ref.
pub(crate) fn referencia_de_lectura(git: &Git, dir: &Path) -> Result<(PathBuf, String), Informe> {
    lectura_de(git, dir).map(|(p, r, _)| (p, r))
}

/// Vista del estado tal como lo conoce este clon: lo ultimo traido de
/// `origin/taskcodetwo/tareas` (sin `fetch`; la lectura fresca es
/// [`leer_fresco`]) o, en un repo sin remoto, la rama local. Sin rama de
/// estado → `rechazada`. La usan `doctor`, `import` y `start`.
pub fn leer<'a>(git: &'a Git, dir: &Path) -> Result<Vista<'a>, Informe> {
    let (principal, referencia) = referencia_de_lectura(git, dir)?;
    vista_de(git, &principal, &referencia)
}

/// Lectura fresca (F3-5b): trae la rama de estado de `origin` (el mismo
/// `fetch` que la transaccion), renueva la marca si sale bien y da la vista
/// de la misma ref que [`leer`] con su [`Frescura`]. Un fallo del remoto no
/// es error: se lee lo ultimo traido y la frescura dice por que. Sin
/// `origin` no hay `fetch`. No toma el cerrojo de estado ni escribe nada mas
/// que `refs/remotes/origin/taskcodetwo/tareas` y la marca.
pub fn leer_fresco<'a>(
    git: &'a Git,
    dir: &Path,
    ahora: u64,
) -> Result<(Vista<'a>, Frescura), Informe> {
    let (principal, referencia, remoto) = lectura_de(git, dir)?;
    let marca = Marca::en(&dir_comun(git, &principal, OP)?);
    let remoto = if remoto {
        con_reintento(&ESPERAS_REF_BLOQUEADA, || {
            traer_estado(git, &principal, &marca, ahora)
        })?
        .remoto()
    } else {
        Remoto::SinRemoto
    };
    let frescura = Frescura {
        remoto,
        sincronizado: marca.leer_en(ahora),
    };
    Ok((vista_de(git, &principal, &referencia)?, frescura))
}

/// Esperas antes de repetir el `fetch` de la lectura si la ref remota esta
/// bloqueada (revision 1 de TASK-079, H4): la lectura no toma el cerrojo de
/// estado, asi que su `fetch` puede coincidir con el de una transaccion u
/// otra lectura del mismo clon, que sueltan el `.lock` en milisegundos. Un
/// `.lock` huerfano sigue ahi tras las esperas y sale `remoto=error-local`.
/// La transaccion no reintenta: su ref bloqueada es `error` (no hay
/// reintento de `fetch` que heredar alli).
const ESPERAS_REF_BLOQUEADA: [std::time::Duration; 2] = [
    std::time::Duration::from_millis(200),
    std::time::Duration::from_millis(500),
];

/// `traer` y, mientras salga [`Traida::Bloqueada`], otra vez tras cada
/// espera de `esperas`.
fn con_reintento(
    esperas: &[std::time::Duration],
    mut traer: impl FnMut() -> Result<Traida, Informe>,
) -> Result<Traida, Informe> {
    let mut t = traer()?;
    for e in esperas {
        if !matches!(t, Traida::Bloqueada { .. }) {
            break;
        }
        std::thread::sleep(*e);
        t = traer()?;
    }
    Ok(t)
}

/// Hora de la ultima sincronizacion del clon de `dir`, sin `fetch` (la de
/// `doctor`, D4a). `None` si no se sincronizo nunca o no se puede saber.
pub fn sincronizado_en(git: &Git, dir: &Path) -> Option<u64> {
    let principal = localizar_principal(git, dir).ok()?;
    Marca::en(&dir_comun(git, &principal, OP).ok()?).leer()
}

/// Lo que `status` dice del propio clon (F3-5b; F1-2 D5 lo encarga).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Local {
    /// Operaciones pendientes de publicar.
    pub pendientes: usize,
    /// `con-cambios` (la carpeta montada no esta en lo leido y tiene cambios
    /// sin publicar, que impiden adelantarla) o `diverge` (la rama local tiene
    /// commits que no estan en lo leido); `None` si no hay nada que avisar.
    pub tareas: Option<&'static str>,
}

/// El [`Local`] del clon de `vista`, comparando la rama local con el commit
/// que se leyo. Solo lee: `status --porcelain` de `tareas/` solo se lanza si
/// la rama local no esta en lo leido.
pub fn local_de(git: &Git, vista: &Vista<'_>) -> Result<Local, Informe> {
    let dir = vista.dir();
    let pendientes = super::pendientes::contar(git, dir)?;
    let rama = format!("refs/heads/{RAMA}");
    let tareas = match (crate::git::commit_de(git, dir, OP, &rama)?, vista.commit()) {
        (Some(local), Some(leido)) if local != leido => {
            if !crate::git::es_ancestro(git, dir, OP, &local, leido)? {
                Some("diverge")
            } else {
                let montada = crate::git::worktrees(git, dir, OP)?
                    .into_iter()
                    .find(|w| w.rama.as_deref() == Some(RAMA) && w.ruta.exists());
                match montada {
                    Some(w) => {
                        let s = crate::git::exigir(
                            git,
                            &w.ruta,
                            OP,
                            crate::git::Subcomando::Status,
                            &["--porcelain"],
                        )?;
                        (!s.trim().is_empty()).then_some("con-cambios")
                    }
                    None => None,
                }
            }
        }
        _ => None,
    };
    Ok(Local { pendientes, tareas })
}

/// Vista de `referencia`; sin rama de estado → `rechazada`.
fn vista_de<'a>(git: &'a Git, principal: &Path, referencia: &str) -> Result<Vista<'a>, Informe> {
    match crate::git::commit_de(git, principal, OP, referencia)? {
        Some(c) => Vista::nueva(git, principal, Some(c)),
        None => Err(Informe::nuevo(OP, Resultado::Rechazada)
            .con("motivo", "sin-rama-de-estado")
            .diagnostico(
                crate::i18n::Mensaje::nuevo(&crate::i18n::catalogo::estado::MONTA_PRIMERO)
                    .con("carpeta", CARPETA),
            )),
    }
}

#[cfg(test)]
mod tests {
    use super::{Traida, con_reintento};
    use std::time::Duration;

    fn bloqueada() -> Traida {
        Traida::Bloqueada {
            detalle: "error: cannot lock ref 'x'".to_owned(),
        }
    }

    /// H4 (revision 1 de TASK-079): la lectura repite el `fetch` mientras la
    /// ref este bloqueada, como mucho una vez por espera, y para en cuanto
    /// sale otra cosa.
    #[test]
    fn reintenta_solo_la_ref_bloqueada() {
        let esperas = [Duration::ZERO, Duration::ZERO];
        // Se suelta el `.lock` al segundo intento.
        let mut n = 0;
        let t = con_reintento(&esperas, || {
            n += 1;
            Ok(if n < 2 { bloqueada() } else { Traida::Hecha })
        });
        assert_eq!((t.ok(), n), (Some(Traida::Hecha), 2));
        // Un `.lock` huerfano: tres intentos y se queda bloqueada.
        let mut n = 0;
        let t = con_reintento(&esperas, || {
            n += 1;
            Ok(bloqueada())
        });
        assert_eq!((t.ok(), n), (Some(bloqueada()), 3));
        // Lo demas no se repite.
        let mut n = 0;
        let t = con_reintento(&esperas, || {
            n += 1;
            Ok(Traida::SinRama)
        });
        assert_eq!((t.ok(), n), (Some(Traida::SinRama), 1));
    }
}
