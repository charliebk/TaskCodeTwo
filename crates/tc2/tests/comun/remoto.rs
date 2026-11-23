//! Escenario «remoto bare servido por un ssh falso» de los tests de
//! integracion (A8-01, A8-16): personas, identidad y operaciones de la
//! biblioteca lanzadas por una persona.

use std::ffi::OsString;
use std::path::Path;

use tc2::contrato::Informe;
use tc2::git::Git;
use tc2::identidad::{Cuenta, Quien};
use tc2::toma::Entorno;

use super::ops::sin_catalogo;

/// `op` con la biblioteca, lanzada por la persona `cuenta` en un terminal
/// (`Quien::Persona`, que el binario solo da con un token real): la linea de
/// contrato.
pub fn como_persona(
    dir: &Path,
    entorno: Vec<(OsString, OsString)>,
    cuenta: Cuenta,
    op: impl FnOnce(&Entorno<'_>) -> Informe,
) -> String {
    let g = Git::nuevo().con_entorno(entorno);
    let q = move || -> Result<Quien, Informe> { Ok(Quien::Persona(cuenta.clone())) };
    let e = Entorno {
        dir,
        git: &g,
        quien: &q,
        catalogo: &sin_catalogo,
        ahora: tc2::identidad::ahora(),
    };
    op(&e).linea()
}
