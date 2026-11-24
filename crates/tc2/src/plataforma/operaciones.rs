//! Flujo neutral de las operaciones del puerto (TASK-139, grupo A1 de la
//! auditoria 4D): lo que no depende de la API de la plataforma (validar la
//! entrada, la idempotencia, la carrera de «ya existe» y las lineas de
//! resultado). Cada adaptador solo lee y escribe su API y traduce lo leido a
//! los tipos de aqui (F1-13, D1a), y asi los dos dan la misma linea byte a
//! byte (RR-15). El transporte es de [`super::conexion`] y la evaluacion de
//! las puertas, de [`super::puertas`].

use super::Paso;
use super::conexion::{rama_valida, sha_valido, uso};
use super::puertas;
use crate::contrato::{Informe, Resultado, Valor};
use crate::i18n::catalogo::plataforma as t;
use crate::i18n::{Mensaje, Texto};

/// Nombre de la operacion `abrir-pr`.
pub(crate) const ABRIR_PR: &str = "abrir-pr";

/// Nombre de la operacion `merge-pr`.
pub(crate) const MERGE_PR: &str = "merge-pr";

/// Lo que dio la escritura que crea el PR.
pub(crate) enum Creado {
    /// Creado, con su numero.
    Pr(u64),
    /// La plataforma dice que ya existe (GitHub: 422 `already exists`;
    /// GitLab: 409). Lleva el diagnostico de la plataforma por si al releer
    /// no se encuentra abierto.
    YaExiste(Mensaje),
    /// Cualquier otro fallo, ya traducido.
    Fallo(Informe),
}

/// Las lecturas y la escritura de `abrir-pr` de un adaptador.
pub(crate) trait AbrePr {
    /// El PR abierto de `rama` contra `base`, si lo hay.
    fn abierto(&self, rama: &str, base: &str) -> Paso<Option<u64>>;

    /// Comprobacion antes de crear (GitLab abre un MR sin commits: lo
    /// comprueba aqui para dar la misma linea que GitHub). Por defecto,
    /// ninguna.
    fn antes_de_crear(&self, _rama: &str, _base: &str) -> Paso<()> {
        Ok(())
    }

    /// Crea el PR de `rama` contra `base`.
    fn crear(&self, rama: &str, base: &str, titulo: &str, cuerpo: &str) -> Creado;
}

/// `abrir-pr`: valida la entrada, devuelve el PR que ya este abierto
/// (`sin-cambios`) o lo crea. Si al crear la plataforma dice que ya existe
/// (otro proceso lo abrio entre la lectura y la escritura), se relee: el
/// abierto es `sin-cambios`; si sigue sin verse, `conflicto
/// motivo=pr-existente`, la misma linea en las dos plataformas.
pub(crate) fn abrir_pr(
    a: &impl AbrePr,
    rama: &str,
    base: &str,
    titulo: &str,
    cuerpo: &str,
) -> Informe {
    if !rama_valida(rama) || !rama_valida(base) {
        return uso(ABRIR_PR, "rama-invalida", &t::RAMA_NO_VALIDA);
    }
    if titulo.trim().is_empty() || titulo.chars().any(char::is_control) {
        return uso(ABRIR_PR, "titulo-invalido", &t::TITULO_NO_VALIDO);
    }
    let sin_cambios =
        |n: u64| Informe::nuevo(ABRIR_PR, Resultado::SinCambios).con("pr", n.to_string());
    match a.abierto(rama, base) {
        Ok(Some(n)) => return sin_cambios(n),
        Ok(None) => {}
        Err(i) => return i,
    }
    if let Err(i) = a.antes_de_crear(rama, base) {
        return i;
    }
    match a.crear(rama, base, titulo, cuerpo) {
        Creado::Pr(n) => Informe::nuevo(ABRIR_PR, Resultado::Ok).con("pr", n.to_string()),
        Creado::YaExiste(diag) => match a.abierto(rama, base) {
            Ok(Some(n)) => sin_cambios(n),
            Ok(None) => Informe::nuevo(ABRIR_PR, Resultado::Conflicto)
                .con("motivo", "pr-existente")
                .diagnostico(diag),
            Err(i) => i,
        },
        Creado::Fallo(i) => i,
    }
}

/// Entrada de `merge-pr`: el sha completo (sin llamar si no vale) y despues
/// la persona de la lista.
pub(crate) fn entrada_merge(sha: &str, exigir_persona: impl FnOnce() -> Paso<()>) -> Paso<()> {
    if !sha_valido(sha) {
        return Err(uso(MERGE_PR, "sha-invalido", &t::SHA_ESPERADO_COMPLETO));
    }
    exigir_persona()
}

/// El estado del PR leido antes de evaluarlo (`estado-puertas`) o
/// mergearlo (`merge-pr`): mergeado es `sin-cambios estado=mergeado`, y
/// cerrado sin mergear, `rechazada motivo=pr-cerrado` con el diagnostico
/// `cerrado`.
pub(crate) fn segun_estado(
    op: &str,
    pr: u64,
    mergeado: bool,
    abierto: bool,
    cerrado: &'static Texto,
) -> Paso<()> {
    if mergeado {
        return Err(puertas::mergeado_en(op, pr));
    }
    if !abierto {
        return Err(puertas::pr_cerrado_en(op, pr, cerrado));
    }
    Ok(())
}

/// Un fallo de la escritura del merge, con `pr=` delante.
pub(crate) fn merge_fallido(pr: u64, mut i: Informe) -> Informe {
    i.claves
        .insert(0, ("pr".to_owned(), Valor::Texto(pr.to_string())));
    i
}
