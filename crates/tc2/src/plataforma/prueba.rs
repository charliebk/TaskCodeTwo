//! Soporte comun de los dobles de [`Plataforma`] de los tests unitarios
//! (A8-05): cada doble implementa [`Parcial`] solo con lo que usa y se pasa
//! envuelto en [`Doble`]. Una operacion que el test no espera hace `panic!`
//! con su nombre del contrato, en vez de devolver un error en silencio. Los
//! tests de integracion tienen el suyo en `tests/comun/plataforma.rs` (otro
//! crate).

#![allow(clippy::panic)]

use std::ffi::OsString;

use super::{
    Firma, ModeloPuertas, Paso, Plataforma, PrAbierto, ReglasLeidas, RevisionPublicada, Rol,
    Usuario,
};
use crate::contrato::Informe;

/// Las operaciones de [`Plataforma`] que un doble puede dar. Por defecto,
/// todas hacen `panic!` con el nombre de la operacion.
pub(crate) trait Parcial {
    fn prs_abiertos(&self, _rol: Rol) -> Paso<Vec<PrAbierto>> {
        inesperada("prs-abiertos")
    }
    fn abrir_pr(&self, _rama: &str, _base: &str, _titulo: &str, _cuerpo: &str) -> Informe {
        inesperada("abrir-pr")
    }
    fn entorno_push(&self, _url_remoto: &str) -> Paso<Vec<(OsString, OsString)>> {
        inesperada("entorno-push")
    }
    fn leer_revision(&self, _rol: Rol, _pr: u64, _origen: u64) -> Paso<RevisionPublicada> {
        inesperada("leer-revision")
    }
    fn estado_puertas(&self, _rol: Rol, _pr: u64, _modelo: &ModeloPuertas) -> Informe {
        inesperada("estado-puertas")
    }
    fn merge_pr(&self, _pr: u64, _sha: &str) -> Informe {
        inesperada("merge-pr")
    }
    fn leer_puertas(&self, _rol: Rol, _rama: &str) -> Paso<ReglasLeidas> {
        inesperada("leer-puertas")
    }
    fn leer_permisos_ci(&self) -> Informe {
        inesperada("leer-permisos-ci")
    }
    fn firma_commit(&self, _sha: &str) -> Paso<Firma> {
        inesperada("firma-commit")
    }
    fn identidad(&self, _rol: Rol) -> Paso<Usuario> {
        inesperada("identidad")
    }
}

fn inesperada(op: &str) -> ! {
    panic!("doble de Plataforma: {op} no se esperaba")
}

/// Envoltorio que da [`Plataforma`] a un [`Parcial`]. Las operaciones que
/// [`Parcial`] no recoge siguen con el defecto del trait.
pub(crate) struct Doble<P>(pub P);

impl<P: Parcial> Plataforma for Doble<P> {
    fn prs_abiertos(&self, rol: Rol) -> Paso<Vec<PrAbierto>> {
        self.0.prs_abiertos(rol)
    }
    fn abrir_pr(&self, rama: &str, base: &str, titulo: &str, cuerpo: &str) -> Informe {
        self.0.abrir_pr(rama, base, titulo, cuerpo)
    }
    fn entorno_push(&self, url_remoto: &str) -> Paso<Vec<(OsString, OsString)>> {
        self.0.entorno_push(url_remoto)
    }
    fn leer_revision(&self, rol: Rol, pr: u64, origen: u64) -> Paso<RevisionPublicada> {
        self.0.leer_revision(rol, pr, origen)
    }
    fn estado_puertas(&self, rol: Rol, pr: u64, modelo: &ModeloPuertas) -> Informe {
        self.0.estado_puertas(rol, pr, modelo)
    }
    fn merge_pr(&self, pr: u64, sha: &str) -> Informe {
        self.0.merge_pr(pr, sha)
    }
    fn leer_puertas(&self, rol: Rol, rama: &str) -> Paso<ReglasLeidas> {
        self.0.leer_puertas(rol, rama)
    }
    fn leer_permisos_ci(&self) -> Informe {
        self.0.leer_permisos_ci()
    }
    fn firma_commit(&self, sha: &str) -> Paso<Firma> {
        self.0.firma_commit(sha)
    }
    fn identidad(&self, rol: Rol) -> Paso<Usuario> {
        self.0.identidad(rol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Vacio;
    impl Parcial for Vacio {}

    /// A8-05: una operacion que el doble no espera hace `panic!` con su
    /// nombre del contrato.
    #[test]
    #[should_panic(expected = "merge-pr")]
    fn un_doble_vacio_no_admite_merge_pr() {
        let _ = Doble(Vacio).merge_pr(1, "a");
    }
}
