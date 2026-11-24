//! El paso a revision de una tarea como operacion de la rama de estado
//! (F6-2; ADR F1-2). Como [`crate::toma::Tomar`] y
//! [`crate::ciclo::Pausar`], pasa por el punto unico de escritura y decide
//! solo con el estado que recibe; lee unicamente el `tarea.md` de su tarea.
//!
//! - Quien: quien la lleva o cualquiera si no la lleva nadie (la regla de
//!   `start`); la de otra persona, `rechazada motivo=de-otra-persona` (o
//!   `asignado-heredado` si es un texto de la 0.16).
//! - `en-curso` pasa a `en-revision`, con `actualizado` y una fila `review`
//!   en `## Transiciones`; el resto del `tarea.md`, byte a byte.
//! - Ya `en-revision`: nada que escribir (`sin-cambios`), salvo que la
//!   seleccion de revision haya cambiado (F6-3): entonces solo se reescribe
//!   `roles.revision`, sin fila nueva.
//! - Cualquier otro estado se rechaza con su motivo.
//!
//! F6-3 (F1-15 decision 10): con [`Revisar::con_seleccion`] la operacion
//! anota en `roles.revision` el lanzador (`agente`), el modelo y las
//! revisoras (`skills`). `regla` se deja como estaba: el modelo de tarea de
//! F2-5 la limita al vocabulario de la 0.16, asi que los ids de fila van en
//! la linea de `review` y en el check (divergencia de la adenda F6-3). La
//! anotacion de la rama de estado no lleva la seleccion: `review` exige red
//! y nunca queda pendiente.

use std::cell::RefCell;

use crate::estado::{Cambio, Estado, Fallo, Operacion, de_otra_persona, por_estado, tarea_de};
use crate::i18n::catalogo::revision as txt;
use crate::identidad::Quien;
use crate::tarea::{Asignado, Transicion};
use crate::toma::instante;
use crate::toma::operacion::{EN_CURSO, MODO};

use super::NOMBRE;

/// Estado en el que queda la tarea revisada.
pub const EN_REVISION: &str = "en-revision";

/// Lo que la seleccion de revision anota en `roles.revision`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnotacionRevision {
    /// El lanzador (S12), si hay.
    pub agente: Option<String>,
    /// Modelo de la escala.
    pub modelo: String,
    /// Revisoras.
    pub skills: Vec<String>,
    /// Filas aplicadas, senales de dominio y avisos (F4-6).
    pub reglas: Vec<String>,
    /// Senales de dominio.
    pub senales: Vec<String>,
    /// Avisos.
    pub avisos: Vec<String>,
}

/// Pasar la tarea `id` a revision.
#[derive(Debug)]
pub struct Revisar {
    id: String,
    asignado: Asignado,
    decidido_por: &'static str,
    ahora: u64,
    dueno: RefCell<Option<String>>,
    seleccion: Option<AnotacionRevision>,
}

impl Revisar {
    /// El paso a revision de `id` por `quien` en el instante `ahora`
    /// (segundos desde 1970, UTC).
    pub fn nueva(id: &str, quien: &Quien, ahora: u64) -> Self {
        Self::con(id, quien.asignado(), quien.decidido_por(), ahora)
    }

    fn con(id: &str, asignado: Asignado, decidido_por: &'static str, ahora: u64) -> Self {
        Self {
            id: id.to_owned(),
            asignado,
            decidido_por,
            ahora,
            dueno: RefCell::new(None),
            seleccion: None,
        }
    }

    /// La misma operacion, anotando la seleccion en `roles.revision`.
    #[must_use]
    pub fn con_seleccion(mut self, s: AnotacionRevision) -> Self {
        self.seleccion = Some(s);
        self
    }

    /// `roles.revision` de `t` con la seleccion; `None` si no hay seleccion.
    fn anotar(&self, t: &crate::tarea::Tarea) -> Option<crate::tarea::Rol> {
        let s = self.seleccion.as_ref()?;
        let mut rv = t.roles.revision.clone();
        rv.agente.clone_from(&s.agente);
        rv.modelo = Some(s.modelo.clone());
        rv.skills.clone_from(&s.skills);
        rv.reglas.clone_from(&s.reglas);
        rv.senales.clone_from(&s.senales);
        rv.avisos.clone_from(&s.avisos);
        Some(rv)
    }

    /// Login de quien la lleva si el ultimo intento la rechazo por
    /// `de-otra-persona`.
    pub fn dueno(&self) -> Option<String> {
        self.dueno.borrow().clone()
    }
}

impl Operacion for Revisar {
    fn nombre(&self) -> &str {
        NOMBRE
    }

    fn anotacion(&self) -> String {
        format!(
            "{NOMBRE}|{}|{}|{}|{}",
            self.id,
            self.asignado.id().unwrap_or_default(),
            self.asignado.login().unwrap_or_default(),
            self.ahora
        )
    }

    fn exige_red(&self) -> bool {
        // `review` ya empujo la rama y abrio el PR: sin red no hay nada.
        true
    }

    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        *self.dueno.borrow_mut() = None;
        let (mut t, mut cuerpo) = tarea_de(estado, &self.id)?;
        if let Some((fallo, dueno)) = de_otra_persona(
            &self.id,
            t.asignado_a.as_ref(),
            &self.asignado,
            NOMBRE,
            ("de-otra-persona", &txt::DE_OTRA_PERSONA),
        ) {
            *self.dueno.borrow_mut() = dueno;
            return Err(fallo);
        }
        let anotada = self.anotar(&t);
        match t.estado.as_str() {
            EN_REVISION => {
                return Ok(match anotada {
                    Some(rv) if rv != t.roles.revision => {
                        t.roles.revision = rv;
                        vec![Cambio::nuevo(
                            format!("{}/tarea.md", self.id),
                            t.escribir(&cuerpo),
                        )]
                    }
                    _ => Vec::new(),
                });
            }
            EN_CURSO => {}
            otro => return Err(por_estado(&self.id, otro, NOMBRE, &[EN_CURSO])),
        }
        if let Some(rv) = anotada {
            t.roles.revision = rv;
        }
        let instante = instante(self.ahora);
        EN_REVISION.clone_into(&mut t.estado);
        t.actualizado = instante.get(..10).unwrap_or_default().to_owned();
        cuerpo.anadir_transicion(&Transicion {
            fecha: &instante,
            fase: NOMBRE,
            modo: MODO,
            decidido_por: self.decidido_por,
            version: crate::VERSION,
        });
        Ok(vec![Cambio::nuevo(
            format!("{}/tarea.md", self.id),
            t.escribir(&cuerpo),
        )])
    }
}

/// Reconstruye un [`Revisar`] desde su anotacion
/// `review|TASK-NNN|<id>|<login>|<instante>` (para el catalogo). Exige red y
/// nunca queda pendiente; si se re-ejecutara, sin persona delante:
/// `decidido_por: agente`.
pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
    let mut partes = a.splitn(5, '|');
    if partes.next()? != NOMBRE {
        return None;
    }
    let id = partes.next()?;
    let cuenta: u64 = partes.next()?.parse().ok()?;
    let asignado = Asignado::persona(cuenta, partes.next()?)?;
    let ahora: u64 = partes.next()?.parse().ok()?;
    crate::git::es_id(id)
        .then(|| Box::new(Revisar::con(id, asignado, "agente", ahora)) as Box<dyn Operacion>)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::estado::prueba::{estado, motivo};
    use crate::identidad::Cuenta;
    use crate::plataforma::Proveedor;
    use crate::tarea::{Cuerpo, Tarea};

    /// 2026-11-17T10:20:30Z.
    const AHORA: u64 = 1_794_910_830;

    fn ana() -> Quien {
        Quien::Agente(Cuenta {
            proveedor: Proveedor::GitHub,
            id: 7,
            login: "ana".to_owned(),
        })
    }

    fn md(estado: &str, asignado: Option<Asignado>) -> String {
        let mut t = Tarea::nueva("TASK-001", "Uno", "2026-11-01");
        t.estado = estado.to_owned();
        t.asignado_a = asignado;
        t.rama = Some("feature/task-001-uno".to_owned());
        t.escribir(&Cuerpo::plantilla())
    }

    /// `en-curso` pasa a `en-revision` con su fila; ya en revision, nada;
    /// otro estado o de otra persona, rechazada.
    #[test]
    fn de_en_curso_a_en_revision() {
        let antes = md("en-curso", ana().asignado().into());
        let op = Revisar::nueva("TASK-001", &ana(), AHORA);
        let r = op.aplicar(&estado(&[("TASK-001", antes.clone())]));
        let texto = r
            .as_ref()
            .ok()
            .and_then(|c| c.first())
            .map(|c| c.contenido.clone())
            .unwrap_or_default();
        let esperado = antes
            .replace("estado: \"en-curso\"", "estado: \"en-revision\"")
            .replace("actualizado: \"2026-11-01\"", "actualizado: \"2026-11-17\"")
            + &format!(
                "| 2026-11-17T10:20:30Z | review | manual | agente | {} |\n",
                crate::VERSION
            );
        assert_eq!(texto, esperado, "{r:?}");
        let ya = md("en-revision", ana().asignado().into());
        assert_eq!(op.aplicar(&estado(&[("TASK-001", ya)])), Ok(Vec::new()));
        let r = op.aplicar(&estado(&[("TASK-001", md("planificada", None))]));
        assert_eq!(motivo(&r), Some("estado-no-valido"));
        let r = op.aplicar(&estado(&[(
            "TASK-001",
            md("en-curso", Asignado::persona(9, "beto")),
        )]));
        assert_eq!(motivo(&r), Some("de-otra-persona"));
        assert_eq!(op.dueno().as_deref(), Some("beto"));
        assert!(op.exige_red());
    }

    /// F6-3: la seleccion va a `roles.revision` (con `regla` intacta); ya en
    /// revision, solo se reescribe si cambia, sin fila nueva.
    #[test]
    fn la_seleccion_se_anota_en_roles_revision() {
        let sel = |skills: &[&str]| AnotacionRevision {
            agente: Some("general-purpose".to_owned()),
            modelo: "opus".to_owned(),
            skills: skills.iter().map(|s| (*s).to_owned()).collect(),
            reglas: Vec::new(),
            senales: Vec::new(),
            avisos: Vec::new(),
        };
        let antes = md("en-curso", ana().asignado().into());
        let op = Revisar::nueva("TASK-001", &ana(), AHORA).con_seleccion(sel(&["p:rust"]));
        let texto = match op.aplicar(&estado(&[("TASK-001", antes)])) {
            Ok(c) => c.first().map(|c| c.contenido.clone()).unwrap_or_default(),
            Err(_) => String::new(),
        };
        assert!(
            texto.contains("revision: {agente: \"general-purpose\", modelo: \"opus\", skills: [\"p:rust\"], regla: null}"),
            "{texto}"
        );
        let ya = texto.replace("estado: \"en-curso\"", "estado: \"en-revision\"");
        assert_eq!(
            op.aplicar(&estado(&[("TASK-001", ya.clone())])),
            Ok(Vec::new())
        );
        let otra = Revisar::nueva("TASK-001", &ana(), AHORA).con_seleccion(sel(&["p:vue"]));
        let r = otra.aplicar(&estado(&[("TASK-001", ya.clone())]));
        let nuevo = r
            .ok()
            .and_then(|c| c.first().map(|c| c.contenido.clone()))
            .unwrap_or_default();
        assert_eq!(nuevo, ya.replace("[\"p:rust\"]", "[\"p:vue\"]"));
    }

    #[test]
    fn anotacion_ida_y_vuelta() {
        let op = Revisar::nueva("TASK-001", &ana(), AHORA);
        assert_eq!(op.anotacion(), format!("review|TASK-001|7|ana|{AHORA}"));
        let r = desde_anotacion(&op.anotacion());
        assert_eq!(r.map(|o| o.anotacion()), Some(op.anotacion()));
        for mala in [
            "review|TASK-1|7|ana|1",
            "review|TASK-001|x|ana|1",
            "review|TASK-001|7|ana|x",
            "review|TASK-001|7|ana",
            "start|TASK-001|7|ana|1",
        ] {
            assert!(desde_anotacion(mala).is_none(), "{mala}");
        }
    }
}
