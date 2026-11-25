//! Esqueleto «plan como dato» (F1-12 PL1) comun a `tc2 init` y `tc2
//! desinstalar` (A7-01): un inventario de solo lectura calcula el plan y la
//! ejecucion aplica ese mismo plan. Aqui va lo que no depende de la orden:
//! el rechazo previo, que no aplica nada, y el bucle que aplica en orden y
//! corta en el primer fallo. Cada orden pone su [`Paso`] (que se aplica y
//! como se aplaza) y su efecto.

use std::path::Path;

/// Una linea del plan, vista desde el bucle de aplicacion.
pub(crate) trait Paso {
    /// Se aplica al ejecutar el plan (`crea` en init, `borra` en
    /// desinstalar).
    fn aplicable(&self) -> bool;
    /// Queda `pendiente` con ese motivo.
    fn aplazar(&mut self, motivo: &str);
    /// Ruta (o ref, o nombre) de la linea.
    fn ruta(&self) -> &str;
}

/// El plan no se aplica: lo que se habria hecho queda `pendiente
/// motivo=no-aplicada`, y el resumen no cuenta lo que no se hizo.
pub(crate) fn rechazar<P: Paso>(acciones: &mut [P]) {
    for a in acciones.iter_mut().filter(|a| a.aplicable()) {
        a.aplazar("no-aplicada");
    }
}

/// Aplica en orden las lineas aplicables con `aplicar`. En el primer fallo
/// esa linea queda `pendiente motivo=error` y las de detras `pendiente
/// motivo=no-aplicada`, sin intentarlas. Devuelve la ruta y el error del
/// fallo, si lo hubo.
pub(crate) fn aplicar_en_orden<P: Paso, E>(
    acciones: &mut [P],
    mut aplicar: impl FnMut(&P) -> Result<(), E>,
) -> Option<(String, E)> {
    let mut fallo = None;
    for a in acciones.iter_mut().filter(|a| a.aplicable()) {
        if fallo.is_some() {
            a.aplazar("no-aplicada");
            continue;
        }
        if let Err(e) = aplicar(a) {
            a.aplazar("error");
            fallo = Some((a.ruta().to_owned(), e));
        }
    }
    fallo
}

/// Ruta como texto para la linea.
pub(crate) fn t(p: &Path) -> String {
    p.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Linea de prueba: (ruta, aplicable, motivo).
    #[derive(Debug, PartialEq)]
    struct L(&'static str, bool, Option<String>);

    impl Paso for L {
        fn aplicable(&self) -> bool {
            self.1
        }
        fn aplazar(&mut self, motivo: &str) {
            self.1 = false;
            self.2 = Some(motivo.to_owned());
        }
        fn ruta(&self) -> &str {
            self.0
        }
    }

    fn plan() -> Vec<L> {
        vec![
            L("a", true, None),
            L("b", false, None),
            L("c", true, None),
            L("d", true, None),
        ]
    }

    /// Tras el primer fallo, la que falla queda `error` y lo de detras
    /// `no-aplicada` sin intentarse; lo no aplicable no se toca.
    #[test]
    fn corta_en_el_primer_fallo() {
        let mut v = plan();
        let mut intentadas = Vec::new();
        let fallo = aplicar_en_orden(&mut v, |l| {
            intentadas.push(l.0);
            if l.0 == "c" { Err("roto") } else { Ok(()) }
        });
        assert_eq!(fallo, Some(("c".to_owned(), "roto")));
        assert_eq!(intentadas, ["a", "c"]);
        assert_eq!(
            v,
            [
                L("a", true, None),
                L("b", false, None),
                L("c", false, Some("error".to_owned())),
                L("d", false, Some("no-aplicada".to_owned())),
            ]
        );
        let mut v = plan();
        assert_eq!(aplicar_en_orden(&mut v, |_| Ok::<(), ()>(())), None);
        assert_eq!(v, plan());
    }

    /// `rechazar` aplaza solo las aplicables, sin aplicar nada.
    #[test]
    fn rechazar_aplaza_las_aplicables() {
        let mut v = plan();
        rechazar(&mut v);
        let no = || Some("no-aplicada".to_owned());
        assert_eq!(
            v,
            [
                L("a", false, no()),
                L("b", false, None),
                L("c", false, no()),
                L("d", false, no()),
            ]
        );
    }
}
