//! Descartar, pausar y reanudar una tarea como operaciones de la rama de
//! estado (F3-7; ADR F1-2 y F1-9). Como [`crate::toma::Tomar`], pasan por el
//! punto unico de escritura y deciden solo con el estado que reciben; leen
//! unicamente el `tarea.md` de su tarea y lo reescriben: frontmatter, una
//! fila nueva en `## Transiciones` y el resto byte a byte.
//!
//! - Quien (D2 de TASK-034): quien lleva la tarea o cualquiera si no la lleva
//!   nadie; la de otra persona, `rechazada motivo=de-otra-persona` (o
//!   `asignado-heredado` si es un texto de la 0.16).
//! - [`Descartar`]: de `planificada`, `en-diseno`, `en-curso` o
//!   `en-revision` a `descartada` con su `motivo`; conserva `asignado_a` y
//!   `rama` (D7). Ya descartada, `sin-cambios` con el motivo de entonces;
//!   terminada, `rechazada motivo=terminada`. Exige red: cambia `next` y las
//!   dependencias de todos (D4).
//! - [`Pausar`]: la pausa no es un estado (D1): una fila `pausa` o
//!   `reanudar` sin tocar estado, asignado ni rama
//!   ([`crate::tarea::Cuerpo::en_pausa`]). Repetirla, o reanudar una que no
//!   esta en pausa, `sin-cambios`. Sin red queda pendiente (D4).

use std::cell::RefCell;

use crate::estado::{Cambio, Estado, Fallo, Operacion, por_estado, tarea_propia};
use crate::grafo::DESCARTADA;
use crate::i18n::catalogo::ciclo as txt;
use crate::identidad::Quien;
use crate::tarea::{Asignado, Cuerpo, PAUSA, REANUDAR, Tarea, Transicion};
use crate::toma::instante;
use crate::toma::operacion::MODO;

/// Nombre de `descartar`, de su operacion y de su fila de transicion.
pub const DESCARTAR: &str = "descartar";

/// Estados de una tarea abierta (ni terminada ni descartada): desde ellos se
/// descarta, se pausa o se reanuda.
pub const ABIERTAS: [&str; 4] = ["planificada", "en-diseno", "en-curso", "en-revision"];

/// Largo maximo del motivo de un descarte, en caracteres: cabe en una
/// linea del ROADMAP y en la anotacion del commit.
pub const MOTIVO_MAX: usize = 500;

/// Si `motivo` vale para un descarte: no vacio, sin saltos de linea ni
/// caracteres de control (la regla de los titulos) y de [`MOTIVO_MAX`]
/// caracteres como mucho.
pub fn motivo_valido(motivo: &str) -> bool {
    crate::tarea::titulo_valido(motivo) && motivo.chars().count() <= MOTIVO_MAX
}

/// Lo comun de las operaciones de una tarea (tambien las de
/// [`super::liberar`]): la tarea, quien la lanza y cuando.
#[derive(Debug)]
pub(super) struct Lanzada {
    pub(super) id: String,
    pub(super) asignado: Asignado,
    pub(super) decidido_por: &'static str,
    pub(super) ahora: u64,
    pub(super) dueno: RefCell<Option<String>>,
}

impl Lanzada {
    pub(super) fn nueva(
        id: &str,
        asignado: Asignado,
        decidido_por: &'static str,
        ahora: u64,
    ) -> Self {
        Self {
            id: id.to_owned(),
            asignado,
            decidido_por,
            ahora,
            dueno: RefCell::new(None),
        }
    }

    /// La tarea de `estado`, si quien lanza `orden` puede cambiarla (D2): la
    /// lleva el o nadie.
    fn tarea(&self, estado: &dyn Estado, orden: &str) -> Result<(Tarea, Cuerpo), Fallo> {
        tarea_propia(
            estado,
            &self.id,
            &self.asignado,
            orden,
            ("de-otra-persona", &txt::DE_OTRA_PERSONA),
            &self.dueno,
        )
    }

    /// `terminada`, `descartada` u otro estado que no es de [`ABIERTAS`].
    fn estado_no_valido(&self, estado: &str, orden: &str) -> Fallo {
        por_estado(&self.id, estado, orden, &ABIERTAS)
    }

    /// El `tarea.md` con `actualizado` de hoy y la fila `fase`.
    pub(super) fn escribir(&self, mut t: Tarea, mut cuerpo: Cuerpo, fase: &str) -> Vec<Cambio> {
        let instante = instante(self.ahora);
        t.actualizado = instante.get(..10).unwrap_or_default().to_owned();
        cuerpo.anadir_transicion(&Transicion {
            fecha: &instante,
            fase,
            modo: MODO,
            decidido_por: self.decidido_por,
            version: crate::VERSION,
        });
        vec![Cambio::nuevo(
            format!("{}/tarea.md", self.id),
            t.escribir(&cuerpo),
        )]
    }

    /// `<nombre>|TASK-NNN|<id>|<login>`.
    pub(super) fn anotacion(&self, nombre: &str) -> String {
        format!(
            "{nombre}|{}|{}|{}",
            self.id,
            self.asignado.id().unwrap_or_default(),
            self.asignado.login().unwrap_or_default()
        )
    }
}

/// Descartar la tarea `id` con un motivo (RF-07).
#[derive(Debug)]
pub struct Descartar {
    base: Lanzada,
    motivo: String,
}

impl Descartar {
    /// El descarte de `id` por `quien` con `motivo` en el instante `ahora`
    /// (segundos desde 1970, UTC). El motivo se valida antes, en la orden
    /// ([`motivo_valido`]).
    pub fn nueva(id: &str, quien: &Quien, motivo: &str, ahora: u64) -> Self {
        Self {
            base: Lanzada::nueva(id, quien.asignado(), quien.decidido_por(), ahora),
            motivo: motivo.to_owned(),
        }
    }

    /// Login de quien lleva la tarea si el ultimo intento la rechazo por
    /// `de-otra-persona`.
    pub fn dueno(&self) -> Option<String> {
        self.base.dueno.borrow().clone()
    }
}

impl Operacion for Descartar {
    fn nombre(&self) -> &str {
        DESCARTAR
    }

    fn anotacion(&self) -> String {
        format!("{}|{}", self.base.anotacion(DESCARTAR), self.motivo)
    }

    fn exige_red(&self) -> bool {
        // Cambia `next` y las dependencias de todos: sin red falla (D4).
        true
    }

    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        let (mut t, cuerpo) = self.base.tarea(estado, DESCARTAR)?;
        match t.estado.as_str() {
            // Ya descartada: nada que escribir; el motivo es el de entonces.
            DESCARTADA => return Ok(Vec::new()),
            e if ABIERTAS.contains(&e) => {}
            otro => return Err(self.base.estado_no_valido(otro, DESCARTAR)),
        }
        DESCARTADA.clone_into(&mut t.estado);
        t.motivo = Some(self.motivo.clone());
        Ok(self.base.escribir(t, cuerpo, DESCARTAR))
    }
}

/// Pausar (o reanudar) la tarea `id` (RF-08).
#[derive(Debug)]
pub struct Pausar {
    base: Lanzada,
    reanudar: bool,
}

impl Pausar {
    /// La pausa de `id` por `quien` en el instante `ahora`; con `reanudar`,
    /// la vuelta.
    pub fn nueva(id: &str, quien: &Quien, reanudar: bool, ahora: u64) -> Self {
        Self {
            base: Lanzada::nueva(id, quien.asignado(), quien.decidido_por(), ahora),
            reanudar,
        }
    }

    /// Login de quien lleva la tarea si el ultimo intento la rechazo por
    /// `de-otra-persona`.
    pub fn dueno(&self) -> Option<String> {
        self.base.dueno.borrow().clone()
    }

    fn fase(&self) -> &'static str {
        if self.reanudar { REANUDAR } else { PAUSA }
    }
}

impl Operacion for Pausar {
    fn nombre(&self) -> &str {
        self.fase()
    }

    fn anotacion(&self) -> String {
        // Con el instante: si queda pendiente sin red, la fila lleva la hora
        // de la pausa y no la de su publicacion (F1-17 mide las pausas).
        format!("{}|{}", self.base.anotacion(self.fase()), self.base.ahora)
    }

    fn exige_red(&self) -> bool {
        // Transicion de una tarea propia: sin red queda pendiente (D4).
        false
    }

    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        let (t, cuerpo) = self.base.tarea(estado, self.fase())?;
        if !ABIERTAS.contains(&t.estado.as_str()) {
            return Err(self.base.estado_no_valido(&t.estado, self.fase()));
        }
        if cuerpo.en_pausa() != self.reanudar {
            // Ya en pausa (o ya sin ella): nada que escribir.
            return Ok(Vec::new());
        }
        Ok(self.base.escribir(t, cuerpo, self.fase()))
    }
}

/// Reconstruye una operacion de este modulo desde su anotacion (para el
/// catalogo): `descartar|TASK-NNN|<id>|<login>|<motivo>`,
/// `pausa|TASK-NNN|<id>|<login>|<instante>` o `reanudar|...`. Se re-ejecuta
/// sin persona delante: `decidido_por: agente`, como en
/// [`crate::toma::desde_anotacion`].
pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
    let mut partes = a.splitn(5, '|');
    let nombre = partes.next()?;
    let id = partes.next()?;
    let cuenta: u64 = partes.next()?.parse().ok()?;
    let asignado = Asignado::persona(cuenta, partes.next()?)?;
    let resto = partes.next()?;
    if !crate::git::es_id(id) {
        return None;
    }
    match nombre {
        DESCARTAR if motivo_valido(resto) => Some(Box::new(Descartar {
            base: Lanzada::nueva(id, asignado, "agente", crate::identidad::ahora()),
            motivo: resto.to_owned(),
        })),
        PAUSA | REANUDAR => {
            let ahora: u64 = resto.parse().ok()?;
            Some(Box::new(Pausar {
                base: Lanzada::nueva(id, asignado, "agente", ahora),
                reanudar: nombre == REANUDAR,
            }))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ciclo::prueba::{AHORA, Extra, ana};
    use crate::estado::prueba::{estado, motivo, texto};

    /// Sin plan aprobado ni filas de transicion.
    fn md(id: &str, estado: &str, asignado: Option<Asignado>) -> String {
        crate::ciclo::prueba::md(id, estado, asignado, Extra::default())
    }

    /// La fila que anade el agente de Ana.
    fn fila(fase: &str) -> String {
        crate::ciclo::prueba::fila(fase, "agente")
    }

    /// C1 (unitario): desde cada estado de origen, `estado: descartada`, el
    /// motivo, `actualizado` y la fila; asignado, rama y el resto, igual.
    #[test]
    fn descartar_escribe_solo_lo_suyo() {
        let yo = ana().asignado();
        for (e, asignado) in [
            ("planificada", None),
            ("en-diseno", None),
            ("en-curso", Some(yo.clone())),
            ("en-revision", Some(yo.clone())),
        ] {
            let antes = md("TASK-001", e, asignado);
            let s = estado(&[("TASK-001", antes.clone())]);
            let op = Descartar::nueva("TASK-001", &ana(), "ya no hace falta | x", AHORA);
            let r = op.aplicar(&s);
            let esperado = antes
                .replace(
                    &format!("estado: \"{e}\"\nmotivo: null"),
                    "estado: \"descartada\"\nmotivo: \"ya no hace falta | x\"",
                )
                .replace("actualizado: \"2026-11-01\"", "actualizado: \"2026-11-17\"")
                + &fila("descartar");
            assert_eq!(texto(&r), esperado, "{e}");
            assert!(op.exige_red());
            assert_eq!(op.dueno(), None);
        }
    }

    /// C6 y C7 (unitarios): de otra persona o heredada se rechaza; ya
    /// descartada, nada que escribir; terminada, rechazada; ilegible o
    /// inexistente, como `start` (la lectura comun, `tarea_de`).
    #[test]
    fn descartar_rechazos_y_sin_cambios() {
        let beto = Asignado::persona(9, "beto");
        let heredado = md("TASK-003", "planificada", None)
            .replace("asignado_a: null", "asignado_a: \"ana@x.com\"");
        let s = estado(&[
            ("TASK-001", md("TASK-001", "en-curso", beto.clone())),
            ("TASK-002", md("TASK-002", DESCARTADA, None)),
            ("TASK-003", heredado),
            ("TASK-004", md("TASK-004", "terminada", None)),
            ("TASK-005", md("TASK-005", DESCARTADA, beto)),
            ("TASK-006", "---\nid: TASK-006\n---\n".to_owned()),
        ]);
        let op = Descartar::nueva("TASK-001", &ana(), "m", AHORA);
        assert_eq!(motivo(&op.aplicar(&s)), Some("de-otra-persona"));
        assert_eq!(op.dueno().as_deref(), Some("beto"));
        let op = Descartar::nueva("TASK-002", &ana(), "otro motivo", AHORA);
        assert_eq!(op.aplicar(&s), Ok(Vec::new()));
        for (id, esperado) in [
            ("TASK-003", "asignado-heredado"),
            ("TASK-004", "terminada"),
            ("TASK-005", "de-otra-persona"),
            ("TASK-006", "ilegible"),
            ("TASK-050", "no-existe"),
        ] {
            let r = Descartar::nueva(id, &ana(), "m", AHORA).aplicar(&s);
            assert_eq!(motivo(&r), Some(esperado), "{id}");
        }
    }

    /// C9 y C10 (unitarios): la pausa anade su fila sin tocar estado,
    /// asignado ni rama; repetirla no escribe; reanudar anade la suya y
    /// reanudar sin pausa no escribe.
    #[test]
    fn pausa_y_reanudar_por_filas() {
        let yo = ana().asignado();
        let antes = md("TASK-001", "en-curso", Some(yo));
        let s = estado(&[("TASK-001", antes.clone())]);
        let reanudar = Pausar::nueva("TASK-001", &ana(), true, AHORA);
        assert_eq!(reanudar.aplicar(&s), Ok(Vec::new()));
        let pausa = Pausar::nueva("TASK-001", &ana(), false, AHORA);
        let pausada = texto(&pausa.aplicar(&s));
        let esperado = antes.replace("actualizado: \"2026-11-01\"", "actualizado: \"2026-11-17\"")
            + &fila("pausa");
        assert_eq!(pausada, esperado);
        assert!(!pausa.exige_red());
        assert_eq!(pausa.nombre(), "pausa");
        let s = estado(&[("TASK-001", pausada.clone())]);
        assert_eq!(pausa.aplicar(&s), Ok(Vec::new()));
        assert_eq!(texto(&reanudar.aplicar(&s)), pausada + &fila("reanudar"));
        assert_eq!(reanudar.nombre(), "reanudar");
    }

    /// C6 y C7 (unitarios): pausar o reanudar la de otra persona, una
    /// terminada o una descartada se rechaza.
    #[test]
    fn pausa_rechazos() {
        let s = estado(&[
            (
                "TASK-001",
                md("TASK-001", "planificada", Asignado::persona(9, "beto")),
            ),
            ("TASK-002", md("TASK-002", "terminada", None)),
            ("TASK-003", md("TASK-003", DESCARTADA, None)),
        ]);
        for reanudar in [false, true] {
            let op = Pausar::nueva("TASK-001", &ana(), reanudar, AHORA);
            assert_eq!(motivo(&op.aplicar(&s)), Some("de-otra-persona"));
            assert_eq!(op.dueno().as_deref(), Some("beto"));
            for (id, esperado) in [("TASK-002", "terminada"), ("TASK-003", "descartada")] {
                let r = Pausar::nueva(id, &ana(), reanudar, AHORA).aplicar(&s);
                assert_eq!(motivo(&r), Some(esperado), "{id} {reanudar}");
            }
        }
    }

    #[test]
    fn motivo_del_descarte() {
        for bueno in [
            "x",
            "ya no hace falta | por TASK-002",
            &"a".repeat(MOTIVO_MAX),
        ] {
            assert!(motivo_valido(bueno), "{bueno}");
        }
        for malo in [
            "",
            "   ",
            "a\nb",
            "a\tb",
            "a\u{2028}b",
            &"a".repeat(MOTIVO_MAX + 1),
        ] {
            assert!(!motivo_valido(malo), "{malo:?}");
        }
    }

    /// Las anotaciones se reconstruyen igual (la pausa, con su instante) y
    /// las malas no.
    #[test]
    fn anotacion_ida_y_vuelta() {
        let ops: [Box<dyn Operacion>; 3] = [
            Box::new(Descartar::nueva("TASK-001", &ana(), "por | esto", AHORA)),
            Box::new(Pausar::nueva("TASK-001", &ana(), false, AHORA)),
            Box::new(Pausar::nueva("TASK-001", &ana(), true, AHORA)),
        ];
        let anotaciones: Vec<String> = ops.iter().map(|o| o.anotacion()).collect();
        assert_eq!(
            anotaciones,
            [
                "descartar|TASK-001|7|ana|por | esto",
                "pausa|TASK-001|7|ana|1794910830",
                "reanudar|TASK-001|7|ana|1794910830",
            ]
        );
        for (o, a) in ops.iter().zip(&anotaciones) {
            let r = desde_anotacion(a);
            assert_eq!(
                r.as_ref()
                    .map(|x| (x.nombre().to_owned(), x.anotacion(), x.exige_red())),
                Some((o.nombre().to_owned(), a.clone(), o.exige_red()))
            );
        }
        for mala in [
            "descartar|TASK-001|7|ana",
            "descartar|TASK-001|7|ana|",
            "descartar|TASK-001|7|ana| ",
            "descartar|TASK-1|7|ana|m",
            "descartar|TASK-001|0|ana|m",
            "pausa|TASK-001|7|ana",
            "pausa|TASK-001|7|ana|x",
            "pausa|TASK-001|x|ana|1",
            "reanudar|TASK-01|7|ana|1",
            "start|TASK-001|7|ana|1",
        ] {
            assert!(desde_anotacion(mala).is_none(), "{mala}");
        }
    }
}
