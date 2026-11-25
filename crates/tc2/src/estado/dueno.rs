//! Las operaciones con dueno de la rama de estado (grupo «operacion con
//! dueno» de la auditoria 4D): lo que comparten `review`, `plan`, `approve`,
//! `start`, `descartar`, `pausa`, `reanudar`, `liberar` y `reasignar` sobre
//! una tarea que lleva una persona.

use std::cell::RefCell;

use super::lectura::Estado;
use super::tareas::{de_otra_persona, tarea_de};
use super::transaccion::Fallo;
use crate::contrato::{Informe, Resultado};
use crate::i18n::Texto;
use crate::tarea::{Asignado, Cuerpo, Tarea};

/// La tarea `id` de `estado`, si `quien` puede cambiarla con `orden` (A8-07,
/// D2 de TASK-034): la lleva el o nadie. El preambulo comun de
/// [`super::Operacion::aplicar`] en las operaciones con dueno: limpia
/// `dueno` (la operacion se re-ejecuta en cada intento), lee la tarea y, si
/// la lleva otra persona, guarda su login en `dueno` y la rechaza con `ajena`
/// (motivo y texto de la orden), o con `asignado-heredado` si es un texto de
/// la 0.16.
pub fn tarea_propia(
    estado: &dyn Estado,
    id: &str,
    quien: &Asignado,
    orden: &str,
    ajena: (&str, &'static Texto),
    dueno: &RefCell<Option<String>>,
) -> Result<(Tarea, Cuerpo), Fallo> {
    *dueno.borrow_mut() = None;
    let (t, c) = tarea_de(estado, id)?;
    if let Some((fallo, login)) = de_otra_persona(id, t.asignado_a.as_ref(), quien, orden, ajena) {
        *dueno.borrow_mut() = login;
        return Err(fallo);
    }
    Ok((t, c))
}

/// La anotacion de una operacion con dueno (A8-19):
/// `<orden>|<id>|<cuenta>|<login>[|<resto>]`. Se escribe y se lee solo
/// aqui; cada operacion interpreta su `resto` (el instante, el motivo, el
/// carril...), que puede llevar `|`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnotacionConDueno<'a> {
    /// Nombre de la operacion (`start`, `review`, `descartar`...).
    pub orden: &'a str,
    /// La tarea (en `approve-hito`, el numero del hito).
    pub id: &'a str,
    /// Quien la lanza.
    pub asignado: Asignado,
    /// Lo que anade la operacion, si anade algo.
    pub resto: Option<&'a str>,
}

impl<'a> AnotacionConDueno<'a> {
    /// La linea de la anotacion.
    pub fn escribir(&self) -> String {
        let mut s = format!(
            "{}|{}|{}|{}",
            self.orden,
            self.id,
            self.asignado.id().unwrap_or_default(),
            self.asignado.login().unwrap_or_default()
        );
        if let Some(r) = self.resto {
            s.push('|');
            s.push_str(r);
        }
        s
    }

    /// La anotacion de `a`, si tiene la forma: un ID de tarea, una cuenta
    /// numerica y un login que valen ([`Asignado::persona`]). `None` si no.
    pub fn leer(a: &'a str) -> Option<Self> {
        let mut partes = a.splitn(5, '|');
        let orden = partes.next()?;
        let id = partes.next()?;
        let cuenta: u64 = partes.next()?.parse().ok()?;
        let asignado = Asignado::persona(cuenta, partes.next()?)?;
        crate::git::es_id(id).then_some(Self {
            orden,
            id,
            asignado,
            resto: partes.next(),
        })
    }
}

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
    use crate::ciclo::prueba::{Extra, md};
    use crate::estado::prueba::estado;

    /// A8-19: ida y vuelta de la operacion entera, sin `resto`, con uno y con
    /// uno que lleva `|`; las anotaciones malas no se leen.
    #[test]
    fn anotacion_con_dueno_ida_y_vuelta() {
        let ana = Asignado::persona(7, "ana").unwrap_or_else(|| unreachable!());
        for resto in [None, Some("1794910830"), Some(""), Some("por | esto")] {
            let a = AnotacionConDueno {
                orden: "descartar",
                id: "TASK-001",
                asignado: ana.clone(),
                resto,
            };
            let texto = a.escribir();
            assert_eq!(AnotacionConDueno::leer(&texto), Some(a), "{texto}");
        }
        let a = AnotacionConDueno {
            orden: "start",
            id: "TASK-012",
            asignado: ana,
            resto: Some("x|y"),
        };
        assert_eq!(a.escribir(), "start|TASK-012|7|ana|x|y");
        for mala in [
            "start|TASK-1|7|ana",
            "start|TASK-001|x|ana",
            "start|TASK-001|0|ana",
            "start|TASK-001|7|",
            "start|TASK-001|7",
            "start",
        ] {
            assert_eq!(AnotacionConDueno::leer(mala), None, "{mala}");
        }
    }

    /// A8-07: la propia o sin dueno se lee; la ajena guarda el login y se
    /// rechaza con el motivo de la orden; la heredada y la inexistente no
    /// dejan dueno, tampoco el de un intento anterior.
    #[test]
    fn tarea_propia_ajena_heredada_e_inexistente() {
        let ana = Asignado::persona(7, "ana").unwrap_or_else(|| unreachable!());
        let beto = Asignado::persona(9, "beto");
        let heredado = md("TASK-003", "en-curso", None, Extra::default())
            .replace("asignado_a: null", "asignado_a: \"ana@x.com\"");
        let s = estado(&[
            (
                "TASK-001",
                md("TASK-001", "en-curso", Some(ana.clone()), Extra::default()),
            ),
            (
                "TASK-002",
                md("TASK-002", "en-curso", beto, Extra::default()),
            ),
            ("TASK-003", heredado),
            (
                "TASK-004",
                md("TASK-004", "en-curso", None, Extra::default()),
            ),
        ]);
        let ajena = ("de-otra", &crate::i18n::catalogo::plan::DE_OTRA_PERSONA);
        let dueno = RefCell::new(None);
        let leer = |id: &str| tarea_propia(&s, id, &ana, "plan", ajena, &dueno);
        let motivo = |r: Result<(Tarea, Cuerpo), Fallo>| match r {
            Err(Fallo::Rechazada { motivo, .. }) => motivo,
            otro => format!("{otro:?}"),
        };
        assert_eq!(
            leer("TASK-001").map(|(t, _)| t.id).ok().as_deref(),
            Some("TASK-001")
        );
        assert_eq!(*dueno.borrow(), None);
        assert_eq!(motivo(leer("TASK-002")), "de-otra");
        assert_eq!(dueno.borrow().as_deref(), Some("beto"));
        assert_eq!(motivo(leer("TASK-003")), "asignado-heredado");
        assert_eq!(*dueno.borrow(), None);
        *dueno.borrow_mut() = Some("beto".to_owned());
        assert_eq!(motivo(leer("TASK-009")), "no-existe");
        assert_eq!(*dueno.borrow(), None);
        *dueno.borrow_mut() = Some("beto".to_owned());
        assert!(leer("TASK-004").is_ok());
        assert_eq!(*dueno.borrow(), None);
    }

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
