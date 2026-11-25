//! Las operaciones con dueno de la rama de estado (grupo «operacion con
//! dueno» de la auditoria 4D): lo que comparten `review`, `plan`, `approve`,
//! `start`, `descartar`, `pausa`, `reanudar`, `liberar` y `reasignar` sobre
//! una tarea que lleva una persona.

use crate::contrato::{Informe, Resultado};

/// Un rechazo con lo que sabe la operacion (A8-09): el login de quien lleva
/// la tarea (`asignado=`) o el arreglo de una tarea de la 0.16. Cualquier
/// otro resultado queda como esta.
pub fn con_dueno(mut i: Informe, dueno: Option<String>) -> Informe {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A8-09: `asignado=` solo en un rechazo `de-otra-persona` con dueno; el
    /// arreglo de `formato-1`; lo demas, sin tocar.
    #[test]
    fn rechazo_con_lo_que_sabe_la_operacion() {
        let rechazo = |m: &str| Informe::nuevo("plan", Resultado::Rechazada).con("motivo", m);
        let ana = || Some("ana".to_owned());
        assert_eq!(
            con_dueno(rechazo("de-otra-persona"), ana()).linea(),
            "plan rechazada motivo=de-otra-persona asignado=ana"
        );
        assert_eq!(
            con_dueno(rechazo("de-otra-persona"), None).linea(),
            "plan rechazada motivo=de-otra-persona"
        );
        assert_eq!(
            con_dueno(rechazo("formato-1"), ana()).linea(),
            "plan rechazada motivo=formato-1 arreglo=tc2%20actualizar"
        );
        assert_eq!(
            con_dueno(rechazo("no-existe"), ana()).linea(),
            "plan rechazada motivo=no-existe"
        );
        let error = Informe::nuevo("plan", Resultado::Error).con("motivo", "de-otra-persona");
        assert_eq!(
            con_dueno(error, ana()).linea(),
            "plan error motivo=de-otra-persona"
        );
    }
}
