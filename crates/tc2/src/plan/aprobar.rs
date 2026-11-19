//! `tc2 approve` como operaciones de la rama de estado (F4-2, RR-04).
//!
//! - [`Aprobar`]: solo existe para una persona ([`Aprobar::nueva`] con
//!   [`Quien::Agente`] es `None`). Exige la tarea `en-diseno`, sin pausa, con
//!   un `plan-final.md` que no sea el esqueleto; la de otra persona se
//!   rechaza (`de-otra-persona`). Escribe `plan_aprobado: true`,
//!   `actualizado` y la fila `approve | <modo congelado> | persona`. Con el
//!   plan ya aprobado no cambia nada (`sin-cambios`, sin firmar otra vez).
//!   Su commit va firmado y con el trailer `Plan-Blob` del `plan-final.md`
//!   de ese mismo commit ([`Sello::Firmado`], F1-6 decision 3).
//! - [`PedirCambios`]: publica `brainstorm/cambios-pedidos-<n>.md` en la
//!   ronda mayor, que asi queda cerrada: el siguiente `plan` abre la `n+1`
//!   (D1a). No concede nada: la puede lanzar tambien el agente. No toca
//!   `tarea.md` (como `plan guardar`, no es una fase).
//!
//! Las dos exigen red y nunca quedan pendientes (no tienen catalogo).

use std::cell::RefCell;

use super::operacion::{EN_DISENO, modo_congelado, ronda_mayor};
use super::plantillas as p;
use super::roles::{self, Rol};
use crate::config::Config;
use crate::estado::{
    Cambio, Estado, Fallo, Operacion, Sello, de_otra_persona, por_estado, tarea_de,
};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::plan as txt;
use crate::identidad::Quien;
use crate::tarea::{Asignado, Tarea, Transicion};
use crate::toma::instante;

/// Nombre de la orden, de la operacion y de su fila.
pub const NOMBRE: &str = "approve";

/// Nombre de la operacion de `--pedir-cambios` (en el commit).
pub const NOMBRE_CAMBIOS: &str = "approve-pedir-cambios";

/// Clave del trailer que ata la aprobacion al plan (F1-5, F1-6).
pub const PLAN_BLOB: &str = "Plan-Blob";

/// Fase de la fila que se escribe: la orden.
const FASE: &str = NOMBRE;

/// Decidido por: solo una persona aprueba.
const PERSONA: &str = "persona";

/// Aprobar el plan de la tarea `id`.
#[derive(Debug)]
pub struct Aprobar {
    id: String,
    asignado: Asignado,
    ahora: u64,
    modo: &'static str,
    dueno: RefCell<Option<String>>,
}

impl Aprobar {
    /// `approve` de `id` por `quien` en `ahora`; `None` si no es una
    /// persona (RR-04: el agente no aprueba nunca). El modo de la fila es el
    /// congelado por `plan`; sin el, el `modo_flujo` del suelo hasta que se
    /// de la configuracion con [`Aprobar::con_config`].
    pub fn nueva(id: &str, quien: &Quien, ahora: u64) -> Option<Self> {
        let Quien::Persona(_) = quien else {
            return None;
        };
        Some(Self {
            id: id.to_owned(),
            asignado: quien.asignado(),
            ahora,
            modo: Config::suelo().modo_flujo.nombre(),
            dueno: RefCell::new(None),
        })
    }

    /// El `modo_flujo` de la configuracion de la base, para la fila si la
    /// tarea no tiene ninguna `plan`.
    #[must_use]
    pub fn con_config(mut self, config: &Config) -> Self {
        self.modo = config.modo_flujo.nombre();
        self
    }

    /// Login de quien lleva la tarea si el ultimo intento la rechazo por
    /// `de-otra-persona`.
    pub fn dueno(&self) -> Option<String> {
        self.dueno.borrow().clone()
    }

    fn rechazo(&self, motivo: &str, m: Mensaje) -> Fallo {
        Fallo::rechazada(motivo, m.con("id", self.id.as_str()))
    }
}

/// Si `plan` es el esqueleto de `plan-final.md` que dejo `plan`: se compara
/// con el de cada juego de roles posible (de 0 a 5 roles, con las etiquetas
/// de la tarea y sin ellas) y con el de los roles de cada ronda con
/// unificador, como `planEsPlantilla` de la 0.16.
fn es_esqueleto(plan: &str, t: &Tarea, rutas: &[String]) -> bool {
    let plan = plan.replace("\r\n", "\n");
    let mut juegos: Vec<Vec<Rol>> = Vec::new();
    for n in 0..=5u8 {
        juegos.push(roles::seleccionar(n, &t.etiquetas));
        juegos.push(roles::seleccionar(n, &[]));
    }
    juegos.extend(juegos_de_rondas(rutas, &t.id));
    juegos
        .iter()
        .any(|r| p::plan_final(&t.id, &t.titulo, r) == plan)
}

/// Los roles de cada ronda, por los nombres de sus peticiones
/// (`peticion-brainstorm-<rol>-<n>.md`), en el orden de [`roles::todos`].
fn juegos_de_rondas(rutas: &[String], id: &str) -> Vec<Vec<Rol>> {
    let prefijo = p::ruta_brainstorm(id, "");
    let mut por_ronda: std::collections::BTreeMap<&str, Vec<&str>> =
        std::collections::BTreeMap::new();
    for nombre in rutas.iter().filter_map(|r| r.strip_prefix(&prefijo)) {
        let Some(resto) = nombre
            .strip_prefix("peticion-")
            .and_then(|n| n.strip_suffix(".md"))
        else {
            continue;
        };
        if let Some((rol, n)) = resto.rsplit_once('-') {
            por_ronda.entry(n).or_default().push(rol);
        }
    }
    por_ronda
        .values()
        .map(|ids| {
            roles::todos()
                .filter(|r| ids.contains(&r.id))
                .copied()
                .collect()
        })
        .collect()
}

impl Operacion for Aprobar {
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
        // La firma es de ahora y de la persona: nunca queda pendiente.
        true
    }

    fn sello(&self) -> Sello {
        Sello::Firmado {
            trailers: vec![(PLAN_BLOB, p::ruta_plan_final(&self.id))],
        }
    }

    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        *self.dueno.borrow_mut() = None;
        let (mut t, mut cuerpo) = tarea_de(estado, &self.id)?;
        if let Some((fallo, dueno)) = de_otra_persona(
            &self.id,
            t.asignado_a.as_ref(),
            &self.asignado,
            NOMBRE,
            ("de-otra-persona", &txt::APROBAR_DE_OTRA_PERSONA),
        ) {
            *self.dueno.borrow_mut() = dueno;
            return Err(fallo);
        }
        // Ya aprobado: nada que escribir ni que firmar otra vez.
        if t.plan_aprobado {
            return Ok(Vec::new());
        }
        if t.estado != EN_DISENO {
            return Err(por_estado(&self.id, &t.estado, NOMBRE, &[EN_DISENO]));
        }
        if cuerpo.en_pausa() {
            return Err(self.rechazo("pausada", Mensaje::nuevo(&txt::APROBAR_PAUSADA)));
        }
        let ruta = p::ruta_plan_final(&self.id);
        let Some(plan) = estado.leer(&ruta)? else {
            return Err(self.rechazo("sin-plan", Mensaje::nuevo(&txt::SIN_PLAN)));
        };
        if es_esqueleto(&plan, &t, &estado.rutas()) {
            return Err(self.rechazo("plan-esqueleto", Mensaje::nuevo(&txt::PLAN_ESQUELETO)));
        }
        let modo = modo_congelado(&cuerpo).unwrap_or_else(|| self.modo.to_owned());
        let instante = instante(self.ahora);
        t.plan_aprobado = true;
        instante
            .get(..10)
            .unwrap_or_default()
            .clone_into(&mut t.actualizado);
        cuerpo.anadir_transicion(&Transicion {
            fecha: &instante,
            fase: FASE,
            modo: &modo,
            decidido_por: PERSONA,
            version: crate::VERSION,
        });
        Ok(vec![Cambio::nuevo(
            format!("{}/tarea.md", self.id),
            t.escribir(&cuerpo),
        )])
    }
}

/// `approve --pedir-cambios`: cerrar la ronda mayor con lo que hay que
/// cambiar del plan.
#[derive(Debug)]
pub struct PedirCambios {
    id: String,
    asignado: Asignado,
    contenido: String,
    ronda: RefCell<Option<u32>>,
    dueno: RefCell<Option<String>>,
}

impl PedirCambios {
    /// Los cambios de `contenido` (CRLF pasa a LF) para la tarea `id`, por
    /// `quien` (persona o agente).
    pub fn nueva(id: &str, quien: &Quien, contenido: &str) -> Self {
        Self {
            id: id.to_owned(),
            asignado: quien.asignado(),
            contenido: contenido.replace("\r\n", "\n"),
            ronda: RefCell::new(None),
            dueno: RefCell::new(None),
        }
    }

    /// La ronda que cerro (o ya estaba cerrada con lo mismo) el ultimo
    /// intento.
    pub fn ronda(&self) -> Option<u32> {
        *self.ronda.borrow()
    }

    /// Login de quien lleva la tarea si el ultimo intento la rechazo por
    /// `de-otra-persona`.
    pub fn dueno(&self) -> Option<String> {
        self.dueno.borrow().clone()
    }

    fn rechazo(&self, motivo: &str, m: Mensaje) -> Fallo {
        Fallo::rechazada(motivo, m.con("id", self.id.as_str()))
    }
}

impl Operacion for PedirCambios {
    fn nombre(&self) -> &str {
        NOMBRE_CAMBIOS
    }

    fn anotacion(&self) -> String {
        format!(
            "{NOMBRE_CAMBIOS}|{}|{}|{}",
            self.id,
            self.asignado.id().unwrap_or_default(),
            self.asignado.login().unwrap_or_default()
        )
    }

    fn exige_red(&self) -> bool {
        // El contenido no cabe en la anotacion (como `plan guardar`).
        true
    }

    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        *self.ronda.borrow_mut() = None;
        *self.dueno.borrow_mut() = None;
        let (t, _) = tarea_de(estado, &self.id)?;
        if let Some((fallo, dueno)) = de_otra_persona(
            &self.id,
            t.asignado_a.as_ref(),
            &self.asignado,
            NOMBRE,
            ("de-otra-persona", &txt::APROBAR_DE_OTRA_PERSONA),
        ) {
            *self.dueno.borrow_mut() = dueno;
            return Err(fallo);
        }
        if t.estado != EN_DISENO {
            return Err(por_estado(&self.id, &t.estado, NOMBRE, &[EN_DISENO]));
        }
        if t.plan_aprobado {
            return Err(self.rechazo("plan-aprobado", Mensaje::nuevo(&txt::PLAN_APROBADO)));
        }
        let (ronda, _, cerrada) = ronda_mayor(&estado.rutas(), &self.id);
        if ronda == 0 {
            return Err(self.rechazo("sin-ronda", Mensaje::nuevo(&txt::SIN_RONDA)));
        }
        let ruta = p::ruta_brainstorm(&self.id, &p::nombre_cambios_pedidos(ronda));
        if cerrada {
            // Lo mismo otra vez: nada que escribir; otra cosa, a la
            // siguiente ronda.
            if estado.leer(&ruta)?.as_deref() == Some(self.contenido.as_str()) {
                *self.ronda.borrow_mut() = Some(ronda);
                return Ok(Vec::new());
            }
            return Err(self.rechazo(
                "ronda-cerrada",
                Mensaje::nuevo(&txt::RONDA_CERRADA).con("ronda", ronda.to_string()),
            ));
        }
        *self.ronda.borrow_mut() = Some(ronda);
        Ok(vec![Cambio::nuevo(ruta, self.contenido.clone())])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identidad::Cuenta;
    use crate::plataforma::Proveedor;
    use crate::roadmap::tests::EnMemoria;
    use crate::tarea::Cuerpo;
    use std::collections::BTreeMap;

    /// 2026-11-19T10:20:30Z.
    const AHORA: u64 = 1_795_083_630;

    fn cuenta() -> Cuenta {
        Cuenta {
            proveedor: Proveedor::GitHub,
            id: 7,
            login: "ana".to_owned(),
        }
    }

    fn persona() -> Quien {
        Quien::Persona(cuenta())
    }

    fn md(id: &str, estado: &str) -> String {
        let mut t = Tarea::nueva(id, "Uno", "2026-11-01");
        t.estado = estado.to_owned();
        let mut c = Cuerpo::con_enunciado("Que haga algo", &["a".to_owned()]);
        c.anadir_transicion(&Transicion {
            fecha: "2026-11-18T09:00:00Z",
            fase: "plan",
            modo: "semiautomatico",
            decidido_por: "agente",
            version: "0.1",
        });
        t.escribir(&c)
    }

    fn motivo(r: &Result<Vec<Cambio>, Fallo>) -> Option<&str> {
        match r {
            Err(Fallo::Rechazada { motivo, .. }) => Some(motivo),
            _ => None,
        }
    }

    /// Criterio 1: el agente no puede construir un `Aprobar` (RR-04).
    #[test]
    fn solo_una_persona_aprueba() {
        assert!(Aprobar::nueva("TASK-001", &Quien::Agente(cuenta()), AHORA).is_none());
        assert!(Aprobar::nueva("TASK-001", &persona(), AHORA).is_some());
    }

    /// Criterio 3 (unitario): aprueba un plan redactado con la fila de la
    /// persona y el modo congelado; el esqueleto, otra persona, el estado,
    /// la pausa y la falta de plan se rechazan; ya aprobado no cambia nada.
    #[test]
    fn aplicar_aprueba_y_rechaza() {
        let roles = roles::seleccionar(1, &[]);
        let esqueleto = p::plan_final("TASK-002", "Uno", &roles);
        let pf = |id: &str| format!("{id}/planificacion/plan-final.md");
        let otra = md("TASK-003", "en-diseno")
            .replace("asignado_a: null", "asignado_a: {id: 9, login: \"beto\"}");
        let e = EnMemoria(BTreeMap::from([
            ("TASK-001/tarea.md".to_owned(), md("TASK-001", "en-diseno")),
            (pf("TASK-001"), "# Mi plan\n".to_owned()),
            ("TASK-002/tarea.md".to_owned(), md("TASK-002", "en-diseno")),
            (pf("TASK-002"), esqueleto.replace('\n', "\r\n")),
            ("TASK-003/tarea.md".to_owned(), otra),
            (pf("TASK-003"), "# Plan\n".to_owned()),
            (
                "TASK-004/tarea.md".to_owned(),
                md("TASK-004", "planificada"),
            ),
            (pf("TASK-004"), "# Plan\n".to_owned()),
            (
                "TASK-005/tarea.md".to_owned(),
                md("TASK-005", "en-diseno").replace("plan_aprobado: false", "plan_aprobado: true"),
            ),
            ("TASK-006/tarea.md".to_owned(), md("TASK-006", "en-diseno")),
            (
                "TASK-007/tarea.md".to_owned(),
                md("TASK-007", "en-diseno")
                    + "| 2026-11-18T10:00:00Z | pausa | manual | agente | 0.1 |\n",
            ),
            (pf("TASK-007"), "# Plan\n".to_owned()),
        ]));
        let aplicar = |id: &str| {
            let op = Aprobar::nueva(id, &persona(), AHORA)
                .unwrap_or_else(|| unreachable!())
                .con_config(&Config::suelo());
            (op.aplicar(&e), op.dueno())
        };
        let (r, _) = aplicar("TASK-001");
        let rutas: Vec<String> = r
            .as_ref()
            .map(|c| c.iter().map(|x| x.ruta.clone()).collect())
            .unwrap_or_default();
        assert_eq!(rutas, ["TASK-001/tarea.md"], "{r:?}");
        let tarea = r
            .ok()
            .and_then(|c| c.first().map(|x| x.contenido.clone()))
            .unwrap_or_default();
        assert!(tarea.contains("\nplan_aprobado: true\n"), "{tarea}");
        assert!(tarea.contains("\nactualizado: \"2026-11-19\"\n"), "{tarea}");
        assert!(
            tarea.ends_with(&format!(
                "| 2026-11-19T10:20:30Z | approve | semiautomatico | persona | {} |\n",
                crate::VERSION
            )),
            "{tarea}"
        );
        assert_eq!(motivo(&aplicar("TASK-002").0), Some("plan-esqueleto"));
        let (r, dueno) = aplicar("TASK-003");
        assert_eq!(
            (motivo(&r), dueno.as_deref()),
            (Some("de-otra-persona"), Some("beto"))
        );
        assert_eq!(motivo(&aplicar("TASK-004").0), Some("estado-no-valido"));
        assert_eq!(aplicar("TASK-005").0, Ok(Vec::new()));
        assert_eq!(motivo(&aplicar("TASK-006").0), Some("sin-plan"));
        assert_eq!(motivo(&aplicar("TASK-007").0), Some("pausada"));
        assert_eq!(motivo(&aplicar("TASK-009").0), Some("no-existe"));
    }

    /// El esqueleto de una ronda con unificador (cuatro roles) tambien se
    /// reconoce; un plan redactado, no.
    #[test]
    fn esqueleto_de_cualquier_juego_de_roles() {
        let mut t = Tarea::nueva("TASK-001", "Uno", "2026-11-01");
        t.etiquetas = vec!["wasm".to_owned()];
        let cinco = roles::seleccionar(4, &t.etiquetas);
        let cuatro = roles::seleccionar(4, &[]);
        for r in [&cinco, &cuatro, &Vec::new()] {
            assert!(es_esqueleto(&p::plan_final("TASK-001", "Uno", r), &t, &[]));
        }
        assert!(!es_esqueleto("# Plan\n", &t, &[]));
        let b = "TASK-001/planificacion/brainstorm/";
        let rutas = vec![
            format!("{b}peticion-brainstorm-riesgos-1.md"),
            format!("{b}peticion-brainstorm-arquitectura-1.md"),
            format!("{b}peticion-unificador-1.md"),
        ];
        let dos: Vec<Rol> = roles::todos()
            .filter(|r| ["brainstorm-arquitectura", "brainstorm-riesgos"].contains(&r.id))
            .copied()
            .collect();
        assert_eq!(juegos_de_rondas(&rutas, "TASK-001"), vec![dos]);
    }

    /// `exige_red` y el sello firmado con `Plan-Blob` de `plan-final.md`.
    #[test]
    fn exige_red_y_firma_con_plan_blob() {
        let op = Aprobar::nueva("TASK-001", &persona(), AHORA).unwrap_or_else(|| unreachable!());
        assert!(op.exige_red());
        assert_eq!(
            op.sello(),
            Sello::Firmado {
                trailers: vec![(
                    "Plan-Blob",
                    "TASK-001/planificacion/plan-final.md".to_owned()
                )]
            }
        );
        assert_eq!(op.anotacion(), "approve|TASK-001|7|ana");
        let c = PedirCambios::nueva("TASK-001", &persona(), "x");
        assert!(c.exige_red());
        assert_eq!(c.sello(), Sello::Normal);
    }

    /// D1a: los cambios pedidos van a la ronda mayor; repetirlos no cambia
    /// nada; otros, con la ronda ya cerrada, se rechazan; sin ronda o con el
    /// plan aprobado, tambien. El agente puede pedirlos.
    #[test]
    fn pedir_cambios_cierra_la_ronda_mayor() {
        let b = "TASK-001/planificacion/brainstorm/";
        let mut e = EnMemoria(BTreeMap::from([
            ("TASK-001/tarea.md".to_owned(), md("TASK-001", "en-diseno")),
            (format!("{b}peticion-plan-1.md"), "p".to_owned()),
            (format!("{b}peticion-unificador-2.md"), "u".to_owned()),
            ("TASK-002/tarea.md".to_owned(), md("TASK-002", "en-diseno")),
            (
                "TASK-003/tarea.md".to_owned(),
                md("TASK-003", "en-diseno").replace("plan_aprobado: false", "plan_aprobado: true"),
            ),
        ]));
        let agente = Quien::Agente(cuenta());
        let op = PedirCambios::nueva("TASK-001", &agente, "cambia a\r\n");
        let r = op.aplicar(&e);
        assert_eq!(
            r,
            Ok(vec![Cambio::nuevo(
                format!("{b}cambios-pedidos-2.md"),
                "cambia a\n"
            )])
        );
        assert_eq!(op.ronda(), Some(2));
        e.0.insert(format!("{b}cambios-pedidos-2.md"), "cambia a\n".into());
        assert_eq!(op.aplicar(&e), Ok(Vec::new()));
        let otro = PedirCambios::nueva("TASK-001", &agente, "otra cosa\n");
        assert_eq!(motivo(&otro.aplicar(&e)), Some("ronda-cerrada"));
        let sin = PedirCambios::nueva("TASK-002", &agente, "x\n");
        assert_eq!(motivo(&sin.aplicar(&e)), Some("sin-ronda"));
        let ya = PedirCambios::nueva("TASK-003", &agente, "x\n");
        assert_eq!(motivo(&ya.aplicar(&e)), Some("plan-aprobado"));
    }
}
