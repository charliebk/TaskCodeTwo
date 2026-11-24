//! La toma de una tarea como operacion de la rama de estado (F3-6; ADR
//! F1-2, decision 4).
//!
//! [`Tomar`] pasa por el punto unico de escritura ([`crate::estado::transaccion`])
//! y se re-ejecuta en cada intento: decide solo con el estado que recibe.
//! Lee el `tarea.md` de la tarea y los de sus dependencias para decidir y,
//! si la toma procede, todos para el historico de la estimacion (F3-13).
//!
//! - Ya la lleva quien la toma y esta en curso o en revision: no cambia
//!   nada (`sin-cambios`, sin commit).
//! - La lleva otra persona: `rechazada motivo=tomada` (o
//!   `asignado-heredado` si es un texto de la 0.16).
//! - Estado de origen (D1 de TASK-033): `planificada` o `en-diseno`; una
//!   terminada, descartada o en otro estado se rechaza con su motivo.
//! - Dependencias (D3): con la regla de [`crate::grafo::dependencia_cumplida`];
//!   alguna sin cumplir, `rechazada motivo=dependencias-pendientes`.
//! - Si procede escribe `estado: en-curso`, `asignado_a`, `actualizado`,
//!   `rama` si faltaba, la `estimacion` recalculada (F3-13: lee entonces
//!   todos los `tarea.md` para el historico) y una fila nueva en
//!   `## Transiciones`; el resto del `tarea.md`, byte a byte.
//!
//! Crear la rama y el worktree no es cosa de la operacion: lo hace `start`
//! fuera de la transaccion y solo con la toma publicada.

use std::cell::RefCell;

use crate::estado::{Cambio, Estado, Fallo, Operacion, de_otra_persona, por_estado, tarea_de};
use crate::estimacion::{Commits, Estimacion};
use crate::grafo::{Nodo, dependencia_cumplida};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::toma as txt;
use crate::identidad::Quien;
use crate::tarea::{Asignado, Transicion};

use super::NOMBRE;

/// Estados en los que la tarea de quien la toma ya esta empezada: `start`
/// la reanuda sin escribir estado (F1-3, decision 2, caso c).
pub const EMPEZADA: [&str; 2] = ["en-curso", "en-revision"];

/// Estados desde los que `start` empieza una tarea (D1 de TASK-033).
pub const EMPEZABLES: [&str; 2] = ["planificada", "en-diseno"];

/// Estado en el que queda la tarea tomada.
pub const EN_CURSO: &str = "en-curso";

/// Modo de la fila de transicion: la 2.0 aun no congela el modo en la
/// tarea (F1-10 lo hara en `plan`); hasta entonces, `manual`.
pub const MODO: &str = "manual";

/// Tomar la tarea `id` para una persona.
#[derive(Debug)]
pub struct Tomar {
    id: String,
    asignado: Asignado,
    decidido_por: &'static str,
    ahora: u64,
    commits: Commits,
    dueno: RefCell<Option<String>>,
    pendientes: RefCell<Vec<String>>,
    rama: RefCell<Option<String>>,
    estimacion: RefCell<Option<Estimacion>>,
    progreso: RefCell<Option<crate::progreso::Progreso>>,
    en_curso: RefCell<Option<Vec<crate::solape::EnCurso>>>,
}

impl Tomar {
    /// La toma de `id` por `quien` en el instante `ahora` (segundos desde
    /// 1970, UTC).
    pub fn nueva(id: &str, quien: &Quien, ahora: u64) -> Self {
        Self::con(id, quien.asignado(), quien.decidido_por(), ahora)
    }

    fn con(id: &str, asignado: Asignado, decidido_por: &'static str, ahora: u64) -> Self {
        Self {
            id: id.to_owned(),
            asignado,
            decidido_por,
            ahora,
            commits: Commits::vacios(),
            dueno: RefCell::new(None),
            pendientes: RefCell::new(Vec::new()),
            rama: RefCell::new(None),
            estimacion: RefCell::new(None),
            progreso: RefCell::new(None),
            en_curso: RefCell::new(None),
        }
    }

    /// Las tareas en curso o en revision del estado del ultimo intento que
    /// escribio la toma, de la misma lectura de todos los `tarea.md` que el
    /// historico (F3-11): el solape de `start` no vuelve a leer el estado.
    pub fn en_curso(&self) -> Option<Vec<crate::solape::EnCurso>> {
        self.en_curso.borrow().clone()
    }

    /// El progreso en vivo de la tarea tomada en el ultimo intento (F3-12),
    /// calculado con el mismo historico y los mismos commits que la
    /// estimacion: `start` lo escribe sin volver a leer el estado ni la
    /// historia (R1-05 y R1-06 de TASK-083).
    pub fn progreso(&self) -> Option<crate::progreso::Progreso> {
        self.progreso.borrow().clone()
    }

    /// Los commits de `develop` para el historico de la estimacion (un
    /// `git log` leido antes de la transaccion, F3-13).
    #[must_use]
    pub fn con_commits(mut self, commits: Commits) -> Self {
        self.commits = commits;
        self
    }

    /// La estimacion escrita en el ultimo intento (F3-13).
    pub fn estimacion(&self) -> Option<Estimacion> {
        self.estimacion.borrow().clone()
    }

    /// Login de quien la llevaba si el ultimo intento la rechazo por
    /// `tomada`.
    pub fn dueno(&self) -> Option<String> {
        self.dueno.borrow().clone()
    }

    /// Dependencias sin cumplir del ultimo intento.
    pub fn pendientes(&self) -> Vec<String> {
        self.pendientes.borrow().clone()
    }

    /// Rama de la tarea vista en el ultimo intento: la de su `rama:` o, si
    /// faltaba, la derivada del ID, el tipo y el titulo (la que se escribe).
    pub fn rama(&self) -> Option<String> {
        self.rama.borrow().clone()
    }

    fn rechazo(&self, motivo: &str, m: Mensaje) -> Fallo {
        Fallo::rechazada(motivo, m.con("id", self.id.as_str()))
    }
}

impl Operacion for Tomar {
    fn nombre(&self) -> &str {
        NOMBRE
    }

    fn anotacion(&self) -> String {
        format!(
            "{NOMBRE}|{}|{}|{}",
            self.id,
            self.asignado.id().unwrap_or_default(),
            self.asignado.login().unwrap_or_default()
        )
    }

    fn exige_red(&self) -> bool {
        // Reserva algo compartido: sin red falla, nunca queda pendiente (D2b).
        true
    }

    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        *self.dueno.borrow_mut() = None;
        self.pendientes.borrow_mut().clear();
        *self.rama.borrow_mut() = None;
        *self.estimacion.borrow_mut() = None;
        *self.progreso.borrow_mut() = None;
        *self.en_curso.borrow_mut() = None;
        let ruta = format!("{}/tarea.md", self.id);
        let (mut t, mut cuerpo) = tarea_de(estado, &self.id)?;
        let rama = t
            .rama
            .clone()
            .unwrap_or_else(|| crate::alta::rama_de(&t.id, &t.tipo, &t.titulo));
        *self.rama.borrow_mut() = Some(rama.clone());
        let mia = t
            .asignado_a
            .as_ref()
            .is_some_and(|a| a.misma_persona(&self.asignado));
        if mia && EMPEZADA.contains(&t.estado.as_str()) {
            return Ok(Vec::new());
        }
        if let Some((fallo, dueno)) = de_otra_persona(
            &self.id,
            t.asignado_a.as_ref(),
            &self.asignado,
            NOMBRE,
            ("tomada", &txt::TOMADA),
        ) {
            *self.dueno.borrow_mut() = dueno;
            return Err(fallo);
        }
        if !EMPEZABLES.contains(&t.estado.as_str()) {
            return Err(por_estado(&self.id, &t.estado, NOMBRE, &EMPEZABLES));
        }
        // En pausa (TASK-034): no se empieza hasta reanudarla.
        if cuerpo.en_pausa() {
            return Err(self.rechazo("pausada", Mensaje::nuevo(&txt::PAUSADA)));
        }
        let rutas: Vec<String> = t
            .dependencias
            .iter()
            .map(|d| format!("{d}/tarea.md"))
            .collect();
        let textos = estado.leer_varios(&rutas)?;
        let pendientes: Vec<String> = t
            .dependencias
            .iter()
            .zip(textos)
            .filter(|(d, texto)| {
                let nodo = texto.as_ref().map(|x| match crate::tarea::leer(x) {
                    Ok(doc) => Nodo::de(&doc),
                    Err(_) => Nodo::ilegible(d),
                });
                !dependencia_cumplida(nodo.as_ref())
            })
            .map(|(d, _)| d.clone())
            .collect();
        if !pendientes.is_empty() {
            let m = Mensaje::nuevo(&txt::DEPENDENCIAS_PENDIENTES).con("ids", pendientes.join(", "));
            *self.pendientes.borrow_mut() = pendientes;
            return Err(self.rechazo("dependencias-pendientes", m));
        }
        // La estimacion, recalculada con las terminadas del estado fresco
        // (F3-13, D2a): `calculada` es la hora del `start`.
        let leidas = crate::estado::tareas(estado, &estado.rutas())?;
        *self.en_curso.borrow_mut() = Some(crate::solape::en_curso(
            leidas.iter().filter_map(|(_, d)| d.as_ref().ok()),
        ));
        let h = crate::estimacion::historico(&leidas, &self.commits);
        let e = crate::estimacion::recalcular(&mut t, &cuerpo, &h, self.ahora);
        *self.estimacion.borrow_mut() = Some(e);
        let instante = instante(self.ahora);
        EN_CURSO.clone_into(&mut t.estado);
        t.asignado_a = Some(self.asignado.clone());
        t.actualizado = instante.get(..10).unwrap_or_default().to_owned();
        if t.rama.is_none() {
            t.rama = Some(rama);
        }
        cuerpo.anadir_transicion(&Transicion {
            fecha: &instante,
            fase: NOMBRE,
            modo: MODO,
            decidido_por: self.decidido_por,
            version: crate::VERSION,
        });
        *self.progreso.borrow_mut() =
            crate::progreso::calcular(&t, &cuerpo, &h, &self.commits, self.ahora);
        Ok(vec![Cambio::nuevo(ruta, t.escribir(&cuerpo))])
    }
}

/// `AAAA-MM-DDTHH:MM:SSZ` (UTC) de `segundos` desde 1970.
pub fn instante(segundos: u64) -> String {
    let dias = i64::try_from(segundos / 86_400).unwrap_or(0);
    let resto = segundos % 86_400;
    format!(
        "{}T{:02}:{:02}:{:02}Z",
        crate::alta::fecha_de_dias(dias),
        resto / 3600,
        resto % 3600 / 60,
        resto % 60
    )
}

/// Reconstruye un [`Tomar`] desde su anotacion `start|TASK-NNN|<id>|<login>`
/// (para el catalogo). `start` exige red y nunca queda pendiente; si se
/// re-ejecutara, lo haria el CLI sin persona delante: `decidido_por: agente`.
pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
    let mut partes = a.splitn(4, '|');
    if partes.next()? != NOMBRE {
        return None;
    }
    let id = partes.next()?;
    let cuenta: u64 = partes.next()?.parse().ok()?;
    let asignado = Asignado::persona(cuenta, partes.next()?)?;
    crate::git::es_id(id).then(|| {
        Box::new(Tomar::con(
            id,
            asignado,
            "agente",
            crate::identidad::ahora(),
        )) as Box<dyn Operacion>
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::estado::prueba::{estado, motivo};
    use crate::identidad::Cuenta;
    use crate::plataforma::Proveedor;
    use crate::tarea::{Cuerpo, Documento, Tarea};

    /// 2026-11-17T10:20:30Z.
    const AHORA: u64 = 1_794_910_830;

    /// La estimacion que pone la toma sin ninguna terminada en el estado
    /// (F3-13): `sin-datos`, calculada a la hora del `start`.
    const SIN_DATOS: &str = "estimacion:\n  calculada: \"2026-11-17T10:20:30Z\"\n  nivel: \"sin-datos\"\n  n: 0\n  por_fase: null\n  total: null\n  agente: null\n";

    fn ana() -> Quien {
        Quien::Agente(Cuenta {
            proveedor: Proveedor::GitHub,
            id: 7,
            login: "ana".to_owned(),
        })
    }

    fn md(id: &str, estado: &str, deps: &[&str], asignado: Option<Asignado>) -> String {
        let mut t = Tarea::nueva(id, &format!("Tarea {id}"), "2026-11-01");
        t.estado = estado.to_owned();
        if estado == "descartada" {
            t.motivo = Some("no hace falta".to_owned());
        }
        t.dependencias = deps.iter().map(|d| (*d).to_owned()).collect();
        t.asignado_a = asignado;
        t.rama = Some(crate::alta::rama_de(id, &t.tipo, &t.titulo));
        t.escribir(&Cuerpo::plantilla())
    }

    #[test]
    fn instante_en_utc() {
        assert_eq!(instante(0), "1970-01-01T00:00:00Z");
        assert_eq!(instante(AHORA), "2026-11-17T10:20:30Z");
        assert_eq!(instante(86_399), "1970-01-01T23:59:59Z");
    }

    /// C7 (unitario): la toma escribe estado, asignado, actualizado y una
    /// fila de transicion; el resto del `tarea.md`, byte a byte.
    #[test]
    fn la_toma_escribe_solo_lo_suyo() {
        let antes = md("TASK-001", "planificada", &[], None);
        let e = estado(&[("TASK-001", antes.clone())]);
        let op = Tomar::nueva("TASK-001", &ana(), AHORA);
        let cambios = op.aplicar(&e);
        assert_eq!(cambios.as_ref().map(Vec::len), Ok(1), "{cambios:?}");
        let Ok([c]) = cambios.as_deref() else {
            return;
        };
        assert_eq!(c.ruta, "TASK-001/tarea.md");
        let esperado = antes
            .replace("estado: \"planificada\"", "estado: \"en-curso\"")
            .replace("asignado_a: null", "asignado_a: {id: 7, login: \"ana\"}")
            .replace("actualizado: \"2026-11-01\"", "actualizado: \"2026-11-17\"")
            .replace("estimacion: null\n", SIN_DATOS)
            + &format!(
                "| 2026-11-17T10:20:30Z | start | manual | agente | {} |\n",
                crate::VERSION
            );
        assert_eq!(c.contenido, esperado);
        assert_eq!(
            op.rama().as_deref(),
            Some("feature/task-001-tarea-task-001")
        );
        // Lo escrito se vuelve a leer como formato 2 sin claves desconocidas.
        let leida = match crate::tarea::leer(&c.contenido) {
            Ok(Documento::Formato2(t, _)) => Some(t.claves_desconocidas().len()),
            _ => None,
        };
        assert_eq!(leida, Some(0));
        assert_eq!(op.anotacion(), "start|TASK-001|7|ana");
        assert!(op.exige_red());
    }

    /// Suposicion 1 del plan: una clave desconocida y el cuerpo se conservan
    /// byte a byte al reescribir; suposicion 4: sin `rama:`, la toma la
    /// escribe derivada del ID.
    #[test]
    fn conserva_claves_desconocidas_y_escribe_la_rama_si_falta() {
        let base = md("TASK-002", "en-diseno", &[], None).replace(
            "dependencias: []\n",
            "dependencias: []\nzonas: [ui, \"api\"] # comentario\n",
        );
        let sin_rama = base.replace("rama: \"feature/task-002-tarea-task-002\"", "rama: null");
        assert_ne!(base, sin_rama);
        let e = estado(&[("TASK-002", sin_rama.clone())]);
        let op = Tomar::nueva("TASK-002", &ana(), AHORA);
        let r = op.aplicar(&e);
        let texto = r
            .as_ref()
            .ok()
            .and_then(|c| c.first())
            .map(|c| c.contenido.clone())
            .unwrap_or_default();
        // Todo igual que en `base` (que lleva la rama) salvo lo de la toma.
        let esperado = base
            .replace("estado: \"en-diseno\"", "estado: \"en-curso\"")
            .replace("asignado_a: null", "asignado_a: {id: 7, login: \"ana\"}")
            .replace("actualizado: \"2026-11-01\"", "actualizado: \"2026-11-17\"")
            .replace("estimacion: null\n", SIN_DATOS)
            + &format!(
                "| 2026-11-17T10:20:30Z | start | manual | agente | {} |\n",
                crate::VERSION
            );
        assert_eq!(texto, esperado, "{r:?}");
        assert!(
            texto.contains("\ndependencias: []\nzonas: [ui, \"api\"] # comentario\n---\n"),
            "{texto}"
        );
        assert_eq!(
            op.rama().as_deref(),
            Some("feature/task-002-tarea-task-002")
        );
    }

    /// C2 (unitario): siendo el dueno de una tarea en curso o en revision,
    /// nada que escribir; el dueno de una planificada si la empieza.
    #[test]
    fn dueno_no_reescribe_una_empezada() {
        let yo = ana().asignado();
        for e in EMPEZADA {
            let s = estado(&[("TASK-003", md("TASK-003", e, &[], Some(yo.clone())))]);
            let op = Tomar::nueva("TASK-003", &ana(), AHORA);
            assert_eq!(op.aplicar(&s), Ok(Vec::new()), "{e}");
            assert!(op.rama().is_some());
        }
        let s = estado(&[("TASK-003", md("TASK-003", "planificada", &[], Some(yo)))]);
        let r = Tomar::nueva("TASK-003", &ana(), AHORA).aplicar(&s);
        assert_eq!(r.map(|c| c.len()), Ok(1));
    }

    /// C1 (unitario): la lleva otra persona: `tomada` con su login; un
    /// texto heredado de la 0.16, `asignado-heredado`.
    #[test]
    fn otra_persona_la_lleva() {
        let beto = Asignado::persona(9, "beto");
        for estado_tarea in ["planificada", "en-curso"] {
            let s = estado(&[("TASK-004", md("TASK-004", estado_tarea, &[], beto.clone()))]);
            let op = Tomar::nueva("TASK-004", &ana(), AHORA);
            let r = op.aplicar(&s);
            assert_eq!(motivo(&r), Some("tomada"));
            assert_eq!(op.dueno().as_deref(), Some("beto"));
        }
        let heredado = md("TASK-005", "planificada", &[], None)
            .replace("asignado_a: null", "asignado_a: \"ana@x.com\"");
        let s = estado(&[("TASK-005", heredado)]);
        let op = Tomar::nueva("TASK-005", &ana(), AHORA);
        assert_eq!(motivo(&op.aplicar(&s)), Some("asignado-heredado"));
        assert_eq!(op.dueno(), None);
    }

    /// C4 y C5 (unitarios): dependencias sin cumplir, estados que no se
    /// empiezan, formato 1, ilegible e inexistente.
    #[test]
    fn rechazos_por_estado_dependencias_y_formato() {
        let s = estado(&[
            ("TASK-010", md("TASK-010", "terminada", &[], None)),
            ("TASK-011", md("TASK-011", "en-curso", &[], None)),
            ("TASK-012", md("TASK-012", "descartada", &[], None)),
            (
                "TASK-013",
                md(
                    "TASK-013",
                    "planificada",
                    &["TASK-010", "TASK-011", "TASK-012", "TASK-099", "TASK-014"],
                    None,
                ),
            ),
            ("TASK-014", "---\nformato: 2\nid: x\n---\n".to_owned()),
            ("TASK-015", md("TASK-015", "en-revision", &[], None)),
        ]);
        let op = Tomar::nueva("TASK-013", &ana(), AHORA);
        assert_eq!(motivo(&op.aplicar(&s)), Some("dependencias-pendientes"));
        // La inexistente no bloquea (regla de `next`); la ilegible si.
        assert_eq!(op.pendientes(), ["TASK-011", "TASK-012", "TASK-014"]);
        for (id, esperado) in [
            ("TASK-010", "terminada"),
            ("TASK-012", "descartada"),
            ("TASK-011", "estado-no-valido"),
            ("TASK-015", "estado-no-valido"),
            ("TASK-014", "ilegible"),
            ("TASK-020", "no-existe"),
        ] {
            let r = Tomar::nueva(id, &ana(), AHORA).aplicar(&s);
            assert_eq!(motivo(&r), Some(esperado), "{id}");
        }
        let v1 = "---\nid: TASK-030\ntitulo: \"t\"\ntipo: feature\nsprint: 0\netiquetas: []\n\
                  complejidad: null\nmodelo_sugerido: sonnet\nestado: planificada\nplan_aprobado: false\n\
                  rama: feature/task-030-t\nasignado_a: null\nagente_revisor: general-purpose\n\
                  skills_recomendados: []\nregla_seleccion_skill: null\nultimo_commit_revisado: null\n\
                  revision_codex: false\ntokens_diseno: null\ntokens_implementacion: null\n\
                  tokens_revision: null\ncreado: 2026-01-01\nactualizado: 2026-01-01\n\
                  dependencias: []\n---\n";
        let s = estado(&[("TASK-030", v1.to_owned())]);
        let r = Tomar::nueva("TASK-030", &ana(), AHORA).aplicar(&s);
        assert_eq!(motivo(&r), Some("formato-1"));
        // El diagnostico lleva el ID y se traduce.
        if let Err(Fallo::Rechazada { diagnostico, .. }) = r {
            for idioma in [crate::i18n::Idioma::Es, crate::i18n::Idioma::En] {
                let d = diagnostico.texto(idioma);
                assert!(
                    d.contains("TASK-030") && d.contains("tc2 actualizar"),
                    "{d}"
                );
            }
        }
    }

    /// TASK-034 C9 (unitario): una planificada en pausa no se empieza
    /// (`pausada`); reanudada, si.
    #[test]
    fn en_pausa_no_se_empieza() {
        let fila =
            |fase: &str| format!("| 2026-11-17T09:00:00Z | {fase} | manual | agente | 0.1 |\n");
        let pausada = md("TASK-001", "planificada", &[], None) + &fila("pausa");
        let s = estado(&[("TASK-001", pausada.clone())]);
        let r = Tomar::nueva("TASK-001", &ana(), AHORA).aplicar(&s);
        assert_eq!(motivo(&r), Some("pausada"));
        let s = estado(&[("TASK-001", pausada + &fila("reanudar"))]);
        let r = Tomar::nueva("TASK-001", &ana(), AHORA).aplicar(&s);
        assert_eq!(r.map(|c| c.len()), Ok(1));
    }

    #[test]
    fn anotacion_ida_y_vuelta() {
        let op = Tomar::nueva("TASK-001", &ana(), AHORA);
        let r = desde_anotacion(&op.anotacion());
        assert_eq!(r.map(|o| o.anotacion()), Some(op.anotacion()));
        for mala in [
            "start|TASK-1|7|ana",
            "start|TASK-001|x|ana",
            "start|TASK-001|0|ana",
            "start|TASK-001|7|",
            "start|TASK-001|7",
            "new|TASK-001|7|ana",
        ] {
            assert!(desde_anotacion(mala).is_none(), "{mala}");
        }
    }
}
