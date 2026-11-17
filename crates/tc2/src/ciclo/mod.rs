//! `tc2 descartar`, `tc2 pausa` y `tc2 reanudar` (F3-7, RF-07 y RF-08).
//!
//! Las operaciones de la rama de estado estan en [`operacion`]; aqui, las
//! ordenes del CLI:
//!
//! - `descartar [--json] TASK-NNN "<motivo>"`: `fetch` (exige red: sin ella,
//!   `sinred pendiente=no` sin escribir nada), quien la lanza (identidad), el
//!   descarte por el punto unico de escritura ([`estado::transaccion`], con
//!   el `ROADMAP.md` regenerado en el mismo commit) y, publicado (o ya
//!   descartada), el worktree de la tarea con [`git::quitar_worktree`] (D3):
//!   se quita si esta limpio y publicado; si no, se conserva y la linea lo
//!   dice (`worktree=conservado worktree-motivo=...`). La rama no se toca
//!   nunca. Lo unico posterior a publicar es quitar el worktree, que nunca
//!   hace fallar la orden: no hay nada que comprobar antes que pueda dejar
//!   el descarte publicado y la orden rechazada.
//! - `pausa [--json] TASK-NNN` y `reanudar [--json] TASK-NNN`: quien la
//!   lanza y la fila por la transaccion; sin red quedan pendientes y la
//!   siguiente transaccion con red las publica (D4, ADR F1-2 punto 5).
//!
//! Lanzadas desde un worktree de tarea escriben el estado en el principal
//! (lo localiza la transaccion). El entorno es el de `start`
//! ([`crate::toma::Entorno`]).

pub mod operacion;

use std::path::Path;

use crate::contrato::{Informe, Resultado, Valor};
use crate::estado::{self, Operacion};
use crate::git::{self, Git};
use crate::i18n::catalogo::ciclo as txt;
use crate::i18n::{Mensaje, Texto};
use crate::toma::Entorno;

pub use operacion::{DESCARTAR, Descartar, MOTIVO_MAX, Pausar, desde_anotacion, motivo_valido};

/// Nombre de la orden `pausa` (y de su fila de transicion).
pub const PAUSA: &str = crate::tarea::PAUSA;

/// Nombre de la orden `reanudar` (y de su fila de transicion).
pub const REANUDAR: &str = crate::tarea::REANUDAR;

fn uso(nombre: &str, motivo: &str, m: impl Into<Mensaje>) -> Informe {
    Informe::nuevo(nombre, Resultado::Uso)
        .con("motivo", motivo)
        .diagnostico(m)
}

/// `descartar TASK-NNN "<motivo>"` (sin `--json`, que quita `lib.rs`). Sin
/// motivo, `uso motivo=sin-motivo`; vacio, solo espacios, con saltos de
/// linea o caracteres de control o de mas de [`MOTIVO_MAX`] caracteres,
/// `uso motivo=motivo-invalido`. Nada se lee ni se escribe antes.
pub fn orden_descartar(args: &[String], e: &Entorno<'_>) -> Informe {
    let u = |motivo: &str| uso(DESCARTAR, motivo, &txt::USO_DESCARTAR);
    match args {
        [] => u("sin-id"),
        [id, ..] if !git::es_id(id) => u("id-invalido"),
        [_] => u("sin-motivo"),
        [id, m] if motivo_valido(m) => descartar_en(e, id, m),
        [_, _] => uso(
            DESCARTAR,
            "motivo-invalido",
            Mensaje::nuevo(&txt::MOTIVO_INVALIDO).con("max", MOTIVO_MAX.to_string()),
        ),
        _ => u("argumentos-de-mas"),
    }
}

/// `pausa TASK-NNN` o, con `reanudar`, `reanudar TASK-NNN`.
pub fn orden_pausa(args: &[String], reanudar: bool, e: &Entorno<'_>) -> Informe {
    let (nombre, m): (&str, &'static Texto) = if reanudar {
        (REANUDAR, &txt::USO_REANUDAR)
    } else {
        (PAUSA, &txt::USO_PAUSA)
    };
    match args {
        [id] if git::es_id(id) => pausa_en(e, id, reanudar),
        [_] => uso(nombre, "id-invalido", m),
        [] => uso(nombre, "sin-id", m),
        _ => uso(nombre, "argumentos-de-mas", m),
    }
}

/// `descartar` de la tarea `id` con `motivo` (ya validado) en el repo de
/// `e`.
pub fn descartar_en(e: &Entorno<'_>, id: &str, motivo: &str) -> Informe {
    let i = match paso_descartar(e, id, motivo) {
        Ok(i) | Err(i) => i,
    };
    de_la_orden(i, DESCARTAR, id)
}

/// `pausa` (o `reanudar`) de la tarea `id` en el repo de `e`.
pub fn pausa_en(e: &Entorno<'_>, id: &str, reanudar: bool) -> Informe {
    let op = match (e.quien)() {
        Ok(q) => Pausar::nueva(id, &q, reanudar, e.ahora),
        Err(i) => return de_la_orden(i, if reanudar { REANUDAR } else { PAUSA }, id),
    };
    let t = escribir(e, &op);
    let i = con_dueno(t, op.dueno());
    de_la_orden(i, op.nombre(), id)
}

fn paso_descartar(e: &Entorno<'_>, id: &str, motivo: &str) -> Result<Informe, Informe> {
    let g = e.git;
    let principal = git::localizar_principal(g, e.dir)?;
    // Exige red (D4): lo de ahora del remoto, tambien las ramas, para saber
    // si la de la tarea esta publicada antes de quitar su worktree.
    if git::estado::hay_remoto(g, &principal, DESCARTAR)? {
        let f = git::fetch(g, &principal);
        if !f.resultado.es_exito() {
            return Err(f);
        }
    }
    let quien = (e.quien)()?;
    let op = Descartar::nueva(id, &quien, motivo, e.ahora);
    let t = escribir(e, &op);
    match t.resultado {
        // Publicado, o ya descartada (repetir termina de quitar el worktree).
        Resultado::Ok | Resultado::SinCambios => Ok(con_worktree(t, g, &principal, e.dir, id)),
        _ => Err(con_dueno(t, op.dueno())),
    }
}

/// La operacion por el punto unico de escritura, con el `ROADMAP.md`
/// regenerado en el mismo commit.
fn escribir(e: &Entorno<'_>, op: &dyn Operacion) -> Informe {
    let personas = crate::roadmap::personas(e.git, e.dir);
    let generar = crate::roadmap::generador(&personas);
    let ctx = estado::Contexto::nuevo(e.git, e.dir, e.catalogo, Some(&generar));
    estado::transaccion(&ctx, op)
}

/// Un rechazo con lo que sabe la operacion: el login de quien la lleva o
/// el arreglo de una tarea de la 0.16.
fn con_dueno(mut i: Informe, dueno: Option<String>) -> Informe {
    if i.resultado != Resultado::Rechazada {
        return i;
    }
    match (i.valor("motivo"), dueno) {
        (Some("de-otra-persona"), Some(l)) => i = i.con("asignado", l),
        (Some("formato-1"), _) => i = i.con("arreglo", "tc2 actualizar"),
        _ => {}
    }
    i
}

/// Tras el descarte, el worktree de la tarea (D3): `worktree=quitado`,
/// `ninguno` (no habia) o `conservado` con `worktree-motivo=` (cambios sin
/// guardar, commits sin publicar, HEAD suelto, lanzado desde dentro o un
/// fallo de git). La rama, local y remota, sigue.
fn con_worktree(i: Informe, g: &Git, principal: &Path, dir: &Path, id: &str) -> Informe {
    let wt = principal.join(git::CARPETA_WORKTREES).join(id);
    if dentro_de(dir, &wt) {
        return i
            .con("worktree", "conservado")
            .con("worktree-motivo", "en-uso");
    }
    let q = git::quitar_worktree(g, principal, id);
    match q.resultado {
        Resultado::Ok => i.con("worktree", "quitado"),
        Resultado::SinCambios => i.con("worktree", "ninguno"),
        _ => {
            let motivo = q.valor("motivo").unwrap_or("error").to_owned();
            i.con("worktree", "conservado")
                .con("worktree-motivo", motivo)
        }
    }
}

/// Si `dir` esta dentro de `wt` (o es el): quitarlo dejaria la orden sin
/// carpeta.
fn dentro_de(dir: &Path, wt: &Path) -> bool {
    match (std::fs::canonicalize(dir), std::fs::canonicalize(wt)) {
        (Ok(d), Ok(w)) => d.starts_with(w),
        _ => false,
    }
}

/// El informe como de la orden: el comando, `id=` el primero y sin la clave
/// `operacion` de la transaccion. Un `sinred` sin `pendiente` (el `fetch`
/// previo de `descartar`) dice `pendiente=no`.
fn de_la_orden(mut i: Informe, nombre: &str, id: &str) -> Informe {
    nombre.clone_into(&mut i.comando);
    i.claves.retain(|(k, _)| k != "operacion" && k != "id");
    i.claves
        .insert(0, ("id".to_owned(), Valor::Texto(id.to_owned())));
    if i.resultado == Resultado::Sinred && i.valor("pendiente").is_none() {
        i = i.con("pendiente", "no");
    }
    i
}
