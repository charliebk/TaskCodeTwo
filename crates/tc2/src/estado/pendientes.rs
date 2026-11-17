//! Operaciones pendientes de publicar (sin red, ADR F1-2 D2b).
//!
//! Cada pendiente es un commit (hecho con `commit-tree`, como los demas) con
//! la operacion anotada en el mensaje, apuntado por
//! `refs/taskcodetwo/pendientes/NNNNNN`. La siguiente transaccion con red no
//! publica ese commit: reconstruye la operacion desde su anotacion y la
//! re-ejecuta sobre el estado nuevo. Publicada, la ref se borra; si ya no
//! procede, pasa a `refs/taskcodetwo/rechazadas/` y su contenido sigue en git.

use std::path::Path;

use super::{OP, REFS_PENDIENTES, REFS_RECHAZADAS};
use crate::git::estado::mensaje_de;
use crate::git::{Git, Subcomando};

type Paso<T> = Result<T, crate::contrato::Informe>;

/// Prefijo de la linea del mensaje que anota la operacion.
pub(crate) const ANOTACION: &str = "Operacion: ";

/// Valor "no existe" de `update-ref` (compare-and-swap de creacion).
const CERO: &str = "0000000000000000000000000000000000000000";

/// Una operacion pendiente.
#[derive(Debug, Clone)]
pub(crate) struct Pendiente {
    pub referencia: String,
    pub sha: String,
    /// `None` si el mensaje no trae la anotacion.
    pub anotacion: Option<String>,
}

/// Que hacer con una pendiente tras la transaccion.
#[derive(Debug, Clone)]
pub(crate) enum Destino {
    /// Se publico: se borra su ref.
    Publicada,
    /// Ya no procede: pasa a rechazadas.
    Rechazada,
}

fn refs(git: &Git, dir: &Path, prefijo: &str) -> Paso<Vec<(String, String)>> {
    let s = crate::git::exigir(
        git,
        dir,
        OP,
        Subcomando::ForEachRef,
        &["--format=%(refname) %(objectname)", prefijo],
    )?;
    let mut v: Vec<(String, String)> = s
        .lines()
        .filter_map(|l| l.split_once(' '))
        .map(|(r, o)| (r.to_owned(), o.to_owned()))
        .collect();
    v.sort();
    Ok(v)
}

/// Ultima linea `Operacion: ...` del mensaje.
pub(crate) fn anotacion_de(mensaje: &str) -> Option<String> {
    mensaje
        .lines()
        .rev()
        .find_map(|l| l.strip_prefix(ANOTACION))
        .map(|a| a.trim().to_owned())
        .filter(|a| !a.is_empty())
}

/// Cuantas pendientes hay (las refs de [`REFS_PENDIENTES`]), sin leer sus
/// mensajes: lo que dice `status` (F3-5b, F1-2 D5).
pub(crate) fn contar(git: &Git, dir: &Path) -> Paso<usize> {
    refs(git, dir, REFS_PENDIENTES).map(|v| v.len())
}

/// Pendientes en orden de llegada.
pub(crate) fn listar(git: &Git, dir: &Path) -> Paso<Vec<Pendiente>> {
    let mut v = Vec::new();
    for (referencia, sha) in refs(git, dir, REFS_PENDIENTES)? {
        let anotacion = anotacion_de(&mensaje_de(git, dir, OP, &sha)?);
        v.push(Pendiente {
            referencia,
            sha,
            anotacion,
        });
    }
    Ok(v)
}

fn numero(referencia: &str) -> u64 {
    referencia
        .rsplit('/')
        .next()
        .and_then(|n| n.parse().ok())
        .unwrap_or(0)
}

/// Guarda `sha` como la siguiente pendiente y devuelve su ref. Los numeros
/// no se reutilizan mientras quede una rechazada con ese numero.
pub(crate) fn guardar(git: &Git, dir: &Path, sha: &str) -> Paso<String> {
    let mut maximo = 0;
    for prefijo in [REFS_PENDIENTES, REFS_RECHAZADAS] {
        for (r, _) in refs(git, dir, prefijo)? {
            maximo = maximo.max(numero(&r));
        }
    }
    let referencia = format!("{REFS_PENDIENTES}{:06}", maximo + 1);
    crate::git::exigir(
        git,
        dir,
        OP,
        Subcomando::UpdateRef,
        &[&referencia, sha, CERO],
    )?;
    Ok(referencia)
}

/// Aplica los destinos. Devuelve (publicadas, rechazadas).
pub(crate) fn resolver(
    git: &Git,
    dir: &Path,
    destinos: &[(Pendiente, Destino)],
) -> Paso<(usize, usize)> {
    let (mut publicadas, mut rechazadas) = (0, 0);
    for (p, d) in destinos {
        if let Destino::Rechazada = d {
            let nombre = p.referencia.trim_start_matches(REFS_PENDIENTES);
            let destino = format!("{REFS_RECHAZADAS}{nombre}");
            crate::git::exigir(
                git,
                dir,
                OP,
                Subcomando::UpdateRef,
                &[&destino, &p.sha, CERO],
            )?;
            rechazadas += 1;
        } else {
            publicadas += 1;
        }
        crate::git::exigir(
            git,
            dir,
            OP,
            Subcomando::UpdateRef,
            &["-d", &p.referencia, &p.sha],
        )?;
    }
    Ok((publicadas, rechazadas))
}

#[cfg(test)]
mod tests {
    use super::{anotacion_de, numero};

    #[test]
    fn anotacion_y_numero() {
        assert_eq!(
            anotacion_de("estado: anotar\n\nOperacion: anotar|a.md\n").as_deref(),
            Some("anotar|a.md")
        );
        assert_eq!(anotacion_de("sin anotacion\n"), None);
        assert_eq!(numero("refs/taskcodetwo/pendientes/000012"), 12);
        assert_eq!(numero("refs/taskcodetwo/pendientes/x"), 0);
    }
}
