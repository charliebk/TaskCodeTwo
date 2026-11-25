//! Soporte comun de los tests unitarios de las operaciones de `ciclo`
//! (A7-04): el instante fijo, la persona de los escenarios, el `tarea.md` de
//! partida y la fila de transicion que anade una operacion. Lo comun a toda
//! operacion sobre la rama de estado esta en `estado::prueba`.

use crate::grafo::DESCARTADA;
use crate::identidad::{Cuenta, Quien};
use crate::plataforma::Proveedor;
use crate::tarea::{Asignado, Cuerpo, Tarea};

/// 2026-11-17T10:20:30Z.
pub(crate) const AHORA: u64 = 1_794_910_830;

/// Cuenta de GitHub con ese id y login.
pub(crate) fn cuenta(id: u64, login: &str) -> Cuenta {
    Cuenta {
        proveedor: Proveedor::GitHub,
        id,
        login: login.to_owned(),
    }
}

/// Ana desde su agente (sin terminal).
pub(crate) fn ana() -> Quien {
    Quien::Agente(cuenta(7, "ana"))
}

/// Lo que cambia entre los `tarea.md` de partida de cada modulo.
#[derive(Default)]
pub(crate) struct Extra {
    /// `plan_aprobado: true`.
    pub(crate) plan_aprobado: bool,
    /// Una fila de transicion `start` de hace tantos segundos (respecto a
    /// [`AHORA`]); sin ella, la tabla de transiciones queda vacia.
    pub(crate) hace: Option<u64>,
}

/// Un `tarea.md` con ese estado y asignado, su rama y lo que diga `extra`.
/// Una descartada lleva el motivo `el de entonces`.
pub(crate) fn md(id: &str, estado: &str, asignado: Option<Asignado>, extra: Extra) -> String {
    let mut t = Tarea::nueva(id, &format!("Tarea {id}"), "2026-11-01");
    t.estado = estado.to_owned();
    if estado == DESCARTADA {
        t.motivo = Some("el de entonces".to_owned());
    }
    t.plan_aprobado = extra.plan_aprobado;
    t.asignado_a = asignado;
    t.rama = Some(crate::alta::rama_de(id, &t.tipo, &t.titulo));
    let fila = extra.hace.map_or_else(String::new, |hace| {
        format!(
            "| {} | start | manual | persona | 0.1 |\n",
            crate::toma::instante(AHORA - hace)
        )
    });
    t.escribir(&Cuerpo::plantilla()) + &fila
}

/// La fila de transicion de `fase` que anade `por` en [`AHORA`].
pub(crate) fn fila(fase: &str, por: &str) -> String {
    format!(
        "| 2026-11-17T10:20:30Z | {fase} | manual | {por} | {} |\n",
        crate::VERSION
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// El `tarea.md` de partida: una descartada lleva siempre el motivo
    /// `el de entonces` (un solo literal en los dos modulos, A7-04); `extra`
    /// solo cambia `plan_aprobado` y la fila de transicion.
    #[test]
    fn md_de_partida() {
        let liberar = Extra {
            plan_aprobado: true,
            hace: Some(0),
        };
        for extra in [Extra::default(), liberar] {
            let (aprobado, con_fila) = (extra.plan_aprobado, extra.hace.is_some());
            let t = md("TASK-001", DESCARTADA, None, extra);
            assert!(t.contains("\nmotivo: \"el de entonces\"\n"), "{t}");
            assert!(t.contains(&format!("\nplan_aprobado: {aprobado}\n")), "{t}");
            assert_eq!(
                t.ends_with("| start | manual | persona | 0.1 |\n"),
                con_fila
            );
        }
        let t = md("TASK-001", "planificada", None, Extra::default());
        assert!(t.contains("\nmotivo: null\n"), "{t}");
    }
}
