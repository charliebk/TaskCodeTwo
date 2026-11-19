//! `plan` como operacion de la rama de estado (F4-1). Como
//! [`crate::toma::Tomar`], pasa por el punto unico de escritura y decide
//! solo con el estado que recibe (se re-ejecuta en cada intento).
//!
//! - Quien: la tarea de otra persona se rechaza (`de-otra-persona`, o
//!   `asignado-heredado`); estados de origen `planificada` y `en-diseno`; en
//!   pausa, `pausada`; con el plan ya aprobado, `plan-aprobado` (como
//!   `plan guardar`).
//! - El enunciado tiene que valer ([`Enunciado::validar`]): si no, se
//!   rechaza con su motivo sin escribir nada.
//! - Carril: el mayor de [`Carril::declarado`] por el tipo, el ya congelado
//!   en la tarea y el pedido con `--carril` (D4); pedir uno por debajo se
//!   rechaza (`carril-baja`). Queda congelado en `carril:`.
//! - Modo: el de la ultima fila `plan` de `## Transiciones` si la hay (ya
//!   congelado); si no, `modo_flujo` de la configuracion de la base. Va en
//!   la columna `modo` de la fila `plan` (F1-10, decision 5).
//! - Roles: [`Carril::roles_diseno`] con la complejidad
//!   ([`heuristica::resolver`]) y [`roles::seleccionar`]; un rol es la forma
//!   `redaccion` (`peticion-plan-<n>.md`), el resto `unificador` (una
//!   peticion por rol y `peticion-unificador-<n>.md`).
//! - Ronda: la mayor con testigo (`peticion-unificador-<n>.md` o
//!   `peticion-plan-<n>.md`) en `TASK-NNN/planificacion/brainstorm/`. Si sus
//!   peticiones son exactamente las que tocan ahora y no tiene cambios
//!   pedidos (`cambios-pedidos-<n>.md`, de `approve --pedir-cambios`), la
//!   ronda sigue abierta y no se reescribe; si no (no hay ninguna, cambio el
//!   juego de roles o la persona pidio cambios), se abre la siguiente. Las
//!   rondas solo se anaden: la rama de estado no borra ficheros.
//! - `plan-final.md`: el esqueleto de F1-9 solo si no existe; uno que ya
//!   existe no se pisa.
//! - `tarea.md`: `estado: en-diseno`, `carril:`, `actualizado` y la fila
//!   `plan`, solo si algo cambia. Repetir `plan` sobre una ronda abierta con
//!   el mismo carril no cambia nada (`sin-cambios`).

use std::cell::RefCell;

use super::NOMBRE;
use super::heuristica::{self, Enunciado, MAX_CRITERIOS};
use super::plantillas::{self as p, Datos};
use super::roles::{self, Rol};
use crate::carril::Carril;
use crate::config::Config;
use crate::estado::{Cambio, Estado, Fallo, Operacion, de_otra_persona, por_estado, tarea_de};
use crate::i18n::catalogo::plan as txt;
use crate::i18n::{Mensaje, Texto};
use crate::identidad::Quien;
use crate::tarea::{Asignado, Cuerpo, Transicion};
use crate::toma::instante;

/// Estados desde los que se planifica.
pub const PLANIFICABLES: [&str; 2] = ["planificada", "en-diseno"];

/// Estado en el que queda la tarea.
pub const EN_DISENO: &str = "en-diseno";

/// Modos validos de la columna `modo` (los de `modo_flujo`).
const MODOS: [&str; 3] = ["manual", "semiautomatico", "automatico"];

/// Como se cierra la ronda.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Forma {
    /// Un solo rol, sin unificador: su respuesta es el plan.
    Redaccion,
    /// Cero, dos o mas roles: consolida el unificador.
    Unificador,
}

impl Forma {
    /// Nombre en la linea.
    pub fn nombre(self) -> &'static str {
        match self {
            Self::Redaccion => "redaccion",
            Self::Unificador => "unificador",
        }
    }
}

/// Lo que decidio el ultimo intento, para la linea.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resumen {
    /// Carril congelado.
    pub carril: Carril,
    /// Modo congelado.
    pub modo: String,
    /// Ronda abierta (la escrita o la que ya estaba).
    pub ronda: u32,
    /// Titulos de los roles lanzados.
    pub roles: Vec<&'static str>,
    /// Forma de la ronda.
    pub forma: Forma,
    /// Peticiones de la ronda (nombres de fichero de `brainstorm/`).
    pub peticiones: Vec<String>,
}

/// Planificar la tarea `id`.
#[derive(Debug)]
pub struct Planificar {
    id: String,
    asignado: Asignado,
    decidido_por: &'static str,
    ahora: u64,
    carril: Option<Carril>,
    modo: &'static str,
    config: Config,
    resumen: RefCell<Option<Resumen>>,
    dueno: RefCell<Option<String>>,
}

impl Planificar {
    /// `plan` de `id` por `quien` en el instante `ahora`, con el carril
    /// pedido (si lo hay) y la configuracion de la base.
    pub fn nueva(
        id: &str,
        quien: &Quien,
        carril: Option<Carril>,
        config: &Config,
        ahora: u64,
    ) -> Self {
        Self {
            id: id.to_owned(),
            asignado: quien.asignado(),
            decidido_por: quien.decidido_por(),
            ahora,
            carril,
            modo: config.modo_flujo.nombre(),
            config: config.clone(),
            resumen: RefCell::new(None),
            dueno: RefCell::new(None),
        }
    }

    /// Lo que decidio el ultimo intento que llego a decidir.
    pub fn resumen(&self) -> Option<Resumen> {
        self.resumen.borrow().clone()
    }

    /// Login de quien lleva la tarea si el ultimo intento la rechazo por
    /// `de-otra-persona`.
    pub fn dueno(&self) -> Option<String> {
        self.dueno.borrow().clone()
    }

    fn rechazo(&self, motivo: &str, m: Mensaje) -> Fallo {
        Fallo::rechazada(motivo, m.con("id", self.id.as_str()))
    }

    /// El rechazo de un enunciado que no vale.
    fn enunciado_invalido(&self, motivo: &'static str, n: usize) -> Fallo {
        let t: &'static Texto = match motivo {
            "objetivo-vacio" => &txt::OBJETIVO_VACIO,
            "sin-criterios" => &txt::SIN_CRITERIOS,
            "criterio-vacio" => &txt::CRITERIO_VACIO,
            _ => &txt::DEMASIADOS_CRITERIOS,
        };
        let m = Mensaje::nuevo(t)
            .con("n", n.to_string())
            .con("max", MAX_CRITERIOS.to_string());
        self.rechazo(motivo, m)
    }
}

/// El modo de la ultima fila `plan` de `## Transiciones`, si es uno valido.
pub fn modo_congelado(cuerpo: &Cuerpo) -> Option<String> {
    let s = cuerpo.seccion("Transiciones")?;
    s.texto
        .lines()
        .rev()
        .filter_map(|l| {
            let mut c = l.trim().strip_prefix('|')?.split('|').map(str::trim);
            let (_fecha, fase, modo) = (c.next()?, c.next()?, c.next()?);
            (fase == NOMBRE).then_some(modo)
        })
        .find(|m| MODOS.contains(m))
        .map(str::to_owned)
}

/// La mayor ronda con testigo de `TASK-NNN/planificacion/brainstorm/`, sus
/// peticiones (nombres, en orden) y si esta cerrada por
/// `approve --pedir-cambios` (`cambios-pedidos-<n>.md`, D1a de TASK-037).
pub(super) fn ronda_mayor(rutas: &[String], id: &str) -> (u32, Vec<String>, bool) {
    let prefijo = p::ruta_brainstorm(id, "");
    let nombres: Vec<&str> = rutas
        .iter()
        .filter_map(|r| r.strip_prefix(&prefijo))
        .filter(|n| !n.contains('/'))
        .collect();
    let numero = |n: &str| -> Option<u32> {
        let resto = n
            .strip_prefix("peticion-unificador-")
            .or_else(|| n.strip_prefix("peticion-plan-"))?
            .strip_suffix(".md")?;
        (!resto.is_empty() && resto.bytes().all(|b| b.is_ascii_digit()))
            .then(|| resto.parse().ok())
            .flatten()
    };
    let r = nombres.iter().filter_map(|n| numero(n)).max().unwrap_or(0);
    let sufijo = format!("-{r}.md");
    let mut de_r: Vec<String> = nombres
        .iter()
        .filter(|n| n.starts_with("peticion-") && n.ends_with(&sufijo))
        .map(|n| (*n).to_owned())
        .collect();
    de_r.sort();
    let cerrada = r >= 1 && nombres.contains(&p::nombre_cambios_pedidos(r).as_str());
    (r, de_r, cerrada)
}

/// Las peticiones de una ronda: nombre y contenido, en orden de nombre.
fn peticiones(d: &Datos<'_>, roles: &[Rol], forma: Forma) -> Vec<(String, String)> {
    let mut v = match (forma, roles) {
        (Forma::Redaccion, [rol]) => vec![(
            p::nombre_peticion_redaccion(d.ronda),
            p::peticion_redaccion(d, rol),
        )],
        _ => {
            let mut v: Vec<(String, String)> = roles
                .iter()
                .map(|rol| {
                    let otros: Vec<Rol> =
                        roles.iter().filter(|r| r.id != rol.id).copied().collect();
                    (
                        p::nombre_peticion_rol(rol, d.ronda),
                        p::peticion_rol(d, rol, &otros),
                    )
                })
                .collect();
            v.push((
                p::nombre_peticion_unificador(d.ronda),
                p::peticion_unificador(d, roles),
            ));
            v
        }
    };
    v.sort_by(|a, b| a.0.cmp(&b.0));
    v
}

impl Operacion for Planificar {
    fn nombre(&self) -> &str {
        NOMBRE
    }

    fn anotacion(&self) -> String {
        format!(
            "{NOMBRE}|{}|{}|{}|{}",
            self.id,
            self.asignado.id().unwrap_or_default(),
            self.asignado.login().unwrap_or_default(),
            self.carril.map_or("-", Carril::nombre)
        )
    }

    fn exige_red(&self) -> bool {
        // Quien orquesta necesita las peticiones ahora: sin red falla, nunca
        // queda pendiente (plan de TASK-036). Por eso no tiene catalogo.
        true
    }

    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        *self.resumen.borrow_mut() = None;
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
        if !PLANIFICABLES.contains(&t.estado.as_str()) {
            return Err(por_estado(&self.id, &t.estado, NOMBRE, &PLANIFICABLES));
        }
        // Como `plan guardar`: un plan aprobado no se replanifica.
        if t.plan_aprobado {
            return Err(self.rechazo("plan-aprobado", Mensaje::nuevo(&txt::PLAN_APROBADO)));
        }
        if cuerpo.en_pausa() {
            return Err(self.rechazo("pausada", Mensaje::nuevo(&txt::PAUSADA)));
        }
        let enunciado = Enunciado::de(&cuerpo);
        if let Err((motivo, n)) = enunciado.validar() {
            return Err(self.enunciado_invalido(motivo, n));
        }
        // El carril: solo sube (D4; lo escrito en la tarea no baja nada).
        let declarado = Carril::declarado(&t.tipo);
        let minimo = t
            .carril
            .as_deref()
            .and_then(Carril::desde)
            .map_or(declarado, |c| c.max(declarado));
        let carril = match self.carril {
            Some(c) if c < minimo => {
                return Err(self.rechazo(
                    "carril-baja",
                    Mensaje::nuevo(&txt::CARRIL_BAJA)
                        .con("pedido", c.nombre())
                        .con("minimo", minimo.nombre()),
                ));
            }
            Some(c) => c,
            None => minimo,
        };
        let resolucion = heuristica::resolver(
            t.complejidad.as_deref(),
            &t.etiquetas,
            &t.tipo,
            &t.dependencias,
            &enunciado,
        );
        let n = carril.roles_diseno(resolucion.roles, &t.tipo, &self.config);
        let roles = roles::seleccionar(n, &t.etiquetas);
        let forma = if roles.len() == 1 {
            Forma::Redaccion
        } else {
            Forma::Unificador
        };
        let modo = modo_congelado(&cuerpo).unwrap_or_else(|| self.modo.to_owned());
        let instante = instante(self.ahora);
        let fecha = instante.get(..10).unwrap_or_default().to_owned();
        let rutas = estado.rutas();
        let (mayor, de_mayor, cerrada) = ronda_mayor(&rutas, &self.id);
        let datos = |ronda: u32| Datos {
            id: &self.id,
            titulo: &t.titulo,
            tipo: &t.tipo,
            complejidad: t.complejidad.as_deref(),
            carril,
            ronda,
            fecha: &fecha,
            enunciado: &enunciado,
            resolucion: &resolucion,
        };
        let nombres = |ronda: u32| -> Vec<String> {
            peticiones(&datos(ronda), &roles, forma)
                .into_iter()
                .map(|(n, _)| n)
                .collect()
        };
        // Con cambios pedidos, la ronda esta cerrada aunque los roles sean
        // los mismos: se abre la siguiente (D1a de TASK-037).
        let abierta = mayor >= 1 && !cerrada && nombres(mayor) == de_mayor;
        let ronda = if abierta { mayor } else { mayor + 1 };
        let mut cambios = Vec::new();
        if !abierta {
            for (nombre, texto) in peticiones(&datos(ronda), &roles, forma) {
                cambios.push(Cambio::nuevo(p::ruta_brainstorm(&self.id, &nombre), texto));
            }
        }
        let plan = p::ruta_plan_final(&self.id);
        if !rutas.contains(&plan) {
            cambios.push(Cambio::nuevo(
                plan,
                p::plan_final(&self.id, &t.titulo, &roles),
            ));
        }
        *self.resumen.borrow_mut() = Some(Resumen {
            carril,
            modo: modo.clone(),
            ronda,
            roles: roles.iter().map(|r| r.titulo).collect(),
            forma,
            peticiones: nombres(ronda),
        });
        let cambia_tarea = t.estado != EN_DISENO
            || t.carril.as_deref() != Some(carril.nombre())
            || !cambios.is_empty();
        if cambia_tarea {
            EN_DISENO.clone_into(&mut t.estado);
            t.carril = Some(carril.nombre().to_owned());
            fecha.clone_into(&mut t.actualizado);
            cuerpo.anadir_transicion(&Transicion {
                fecha: &instante,
                fase: NOMBRE,
                modo: &modo,
                decidido_por: self.decidido_por,
                version: crate::VERSION,
            });
            cambios.push(Cambio::nuevo(
                format!("{}/tarea.md", self.id),
                t.escribir(&cuerpo),
            ));
        }
        Ok(cambios)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ModoFlujo;
    use crate::identidad::Cuenta;
    use crate::plataforma::Proveedor;
    use crate::roadmap::tests::EnMemoria;
    use crate::tarea::Tarea;
    use std::collections::BTreeMap;

    /// 2026-11-17T10:20:30Z.
    const AHORA: u64 = 1_794_910_830;

    fn ana() -> Quien {
        Quien::Agente(Cuenta {
            proveedor: Proveedor::GitHub,
            id: 7,
            login: "ana".to_owned(),
        })
    }

    fn md(tipo: &str, estado: &str, criterios: &[&str]) -> String {
        let mut t = Tarea::nueva("TASK-001", "Uno", "2026-11-01");
        t.tipo = tipo.to_owned();
        t.estado = estado.to_owned();
        let c: Vec<String> = criterios.iter().map(|x| (*x).to_owned()).collect();
        t.escribir(&Cuerpo::con_enunciado("Que haga algo", &c))
    }

    fn estado(ficheros: &[(&str, String)]) -> EnMemoria {
        EnMemoria(
            ficheros
                .iter()
                .map(|(r, t)| ((*r).to_owned(), t.clone()))
                .collect::<BTreeMap<_, _>>(),
        )
    }

    fn rutas(r: &Result<Vec<Cambio>, Fallo>) -> Vec<String> {
        r.as_ref()
            .map(|c| c.iter().map(|x| x.ruta.clone()).collect())
            .unwrap_or_default()
    }

    fn motivo(r: &Result<Vec<Cambio>, Fallo>) -> Option<&str> {
        match r {
            Err(Fallo::Rechazada { motivo, .. }) => Some(motivo),
            _ => None,
        }
    }

    /// Criterio 2 (unitario): uno en fix (solo la peticion de redaccion),
    /// los cuatro y el unificador con `--carril riesgo-alto`; el carril y
    /// el modo de la configuracion quedan en la tarea.
    #[test]
    fn uno_en_fix_y_todos_en_riesgo_alto() {
        let e = estado(&[("TASK-001/tarea.md", md("fix", "planificada", &["a"]))]);
        let mut c = Config::suelo();
        c.modo_flujo = ModoFlujo::Automatico;
        let op = Planificar::nueva("TASK-001", &ana(), None, &c, AHORA);
        let r = op.aplicar(&e);
        assert_eq!(
            rutas(&r),
            [
                "TASK-001/planificacion/brainstorm/peticion-plan-1.md",
                "TASK-001/planificacion/plan-final.md",
                "TASK-001/tarea.md",
            ]
        );
        let s = op.resumen();
        assert_eq!(
            s.as_ref()
                .map(|s| (s.carril, s.modo.as_str(), s.forma, s.roles.clone())),
            Some((
                Carril::Fix,
                "automatico",
                Forma::Redaccion,
                vec!["arquitectura"]
            ))
        );
        let tarea = r
            .as_ref()
            .ok()
            .and_then(|c| c.last())
            .map(|c| c.contenido.clone())
            .unwrap_or_default();
        assert!(
            tarea.contains("\ncarril: \"fix\"\nestado: \"en-diseno\"\n"),
            "{tarea}"
        );
        assert!(tarea.ends_with(&format!(
            "| 2026-11-17T10:20:30Z | plan | automatico | agente | {} |\n",
            crate::VERSION
        )));
        let op = Planificar::nueva("TASK-001", &ana(), Some(Carril::RiesgoAlto), &c, AHORA);
        let r = op.aplicar(&e);
        let b = "TASK-001/planificacion/brainstorm/";
        assert_eq!(
            rutas(&r),
            [
                format!("{b}peticion-brainstorm-arquitectura-1.md"),
                format!("{b}peticion-brainstorm-dominio-1.md"),
                format!("{b}peticion-brainstorm-riesgos-1.md"),
                format!("{b}peticion-brainstorm-testing-1.md"),
                format!("{b}peticion-unificador-1.md"),
                "TASK-001/planificacion/plan-final.md".to_owned(),
                "TASK-001/tarea.md".to_owned(),
            ]
        );
    }

    /// Repetir sobre la ronda abierta no cambia nada; subir el carril abre
    /// la ronda 2 sin pisar el plan; el modo congelado gana a la
    /// configuracion; bajar el carril se rechaza.
    #[test]
    fn rondas_y_carril() {
        let c = Config::suelo();
        let mut e = estado(&[("TASK-001/tarea.md", md("fix", "planificada", &["a"]))]);
        let aplicar = |e: &mut EnMemoria, carril: Option<Carril>, c: &Config| {
            let op = Planificar::nueva("TASK-001", &ana(), carril, c, AHORA);
            let r = op.aplicar(e);
            for x in r.as_ref().map(Vec::as_slice).unwrap_or_default() {
                e.0.insert(x.ruta.clone(), x.contenido.clone());
            }
            (r, op.resumen())
        };
        let (r, _) = aplicar(&mut e, None, &c);
        assert_eq!(rutas(&r).len(), 3);
        let (r, s) = aplicar(&mut e, None, &c);
        assert_eq!(r, Ok(Vec::new()));
        assert_eq!(
            s.map(|s| (s.ronda, s.peticiones)),
            Some((1, vec!["peticion-plan-1.md".to_owned()]))
        );
        // La configuracion cambia de modo: el congelado manda.
        let mut auto = Config::suelo();
        auto.modo_flujo = ModoFlujo::Automatico;
        let (r, s) = aplicar(&mut e, Some(Carril::Funcionalidad), &auto);
        // Mismo juego de roles (uno): sigue la ronda 1, cambia el carril.
        assert_eq!(rutas(&r), ["TASK-001/tarea.md"]);
        assert_eq!(s.map(|s| (s.ronda, s.modo)), Some((1, "manual".to_owned())));
        e.0.insert(
            "TASK-001/planificacion/plan-final.md".into(),
            "mi plan\n".into(),
        );
        let (r, s) = aplicar(&mut e, Some(Carril::RiesgoAlto), &c);
        let r = rutas(&r);
        assert_eq!(r.len(), 6, "{r:?}");
        assert!(
            r.contains(&"TASK-001/planificacion/brainstorm/peticion-unificador-2.md".to_owned())
        );
        assert!(!r.contains(&"TASK-001/planificacion/plan-final.md".to_owned()));
        assert_eq!(s.map(|s| s.ronda), Some(2));
        let (r, _) = aplicar(&mut e, Some(Carril::Fix), &c);
        assert_eq!(motivo(&r), Some("carril-baja"));
        assert_eq!(
            e.0.get("TASK-001/planificacion/plan-final.md")
                .map(String::as_str),
            Some("mi plan\n")
        );
    }

    /// D1a de TASK-037: con `cambios-pedidos-1.md` la ronda 1 esta cerrada
    /// y `plan` con los mismos roles abre la ronda 2 (sin pisar el plan);
    /// sin el testigo, repetir no cambia nada. Un testigo de otra ronda no
    /// cierra la mayor.
    #[test]
    fn cambios_pedidos_cierran_la_ronda() {
        let c = Config::suelo();
        let mut e = estado(&[("TASK-001/tarea.md", md("fix", "planificada", &["a"]))]);
        let aplicar = |e: &mut EnMemoria| {
            let op = Planificar::nueva("TASK-001", &ana(), None, &c, AHORA);
            let r = op.aplicar(e);
            for x in r.as_ref().map(Vec::as_slice).unwrap_or_default() {
                e.0.insert(x.ruta.clone(), x.contenido.clone());
            }
            (r, op.resumen())
        };
        assert_eq!(rutas(&aplicar(&mut e).0).len(), 3);
        assert_eq!(aplicar(&mut e).0, Ok(Vec::new()));
        let b = "TASK-001/planificacion/brainstorm/";
        e.0.insert(format!("{b}cambios-pedidos-0.md"), "x\n".into());
        assert_eq!(aplicar(&mut e).0, Ok(Vec::new()));
        e.0.insert(format!("{b}cambios-pedidos-1.md"), "cambia esto\n".into());
        let (r, s) = aplicar(&mut e);
        assert_eq!(
            rutas(&r),
            [
                format!("{b}peticion-plan-2.md"),
                "TASK-001/tarea.md".to_owned()
            ]
        );
        assert_eq!(
            s.map(|s| (s.ronda, s.peticiones)),
            Some((2, vec!["peticion-plan-2.md".to_owned()]))
        );
        // La ronda 2 sigue abierta mientras nadie pida cambios en ella.
        assert_eq!(aplicar(&mut e).0, Ok(Vec::new()));
    }

    /// Lo que no se planifica: enunciado que no vale, estado, otra persona.
    #[test]
    fn rechazos() {
        let c = Config::suelo();
        let mut sin_objetivo = Tarea::nueva("TASK-002", "Dos", "2026-11-01");
        sin_objetivo.estado = "planificada".into();
        let otra = md("feature", "planificada", &["a"])
            .replace("asignado_a: null", "asignado_a: {id: 9, login: \"beto\"}");
        let e = estado(&[
            ("TASK-001/tarea.md", md("feature", "en-curso", &["a"])),
            (
                "TASK-002/tarea.md",
                sin_objetivo.escribir(&Cuerpo::plantilla()),
            ),
            ("TASK-003/tarea.md", otra.replace("TASK-001", "TASK-003")),
            (
                "TASK-004/tarea.md",
                md("feature", "planificada", &[]).replace("TASK-001", "TASK-004"),
            ),
            (
                "TASK-005/tarea.md",
                md("feature", "en-diseno", &["a"])
                    .replace("TASK-001", "TASK-005")
                    .replace("plan_aprobado: false", "plan_aprobado: true"),
            ),
            (
                "TASK-006/tarea.md",
                md("feature", "planificada", &["a"]).replace("TASK-001", "TASK-006")
                    + "| 2026-11-17T09:00:00Z | pausa | manual | agente | 0.1 |\n",
            ),
        ]);
        for (id, esperado) in [
            ("TASK-001", "estado-no-valido"),
            ("TASK-002", "objetivo-vacio"),
            ("TASK-003", "de-otra-persona"),
            ("TASK-004", "criterio-vacio"),
            ("TASK-005", "plan-aprobado"),
            ("TASK-006", "pausada"),
            ("TASK-009", "no-existe"),
        ] {
            let op = Planificar::nueva(id, &ana(), None, &c, AHORA);
            assert_eq!(motivo(&op.aplicar(&e)), Some(esperado), "{id}");
        }
    }

    /// El tope de 1 rol de un `hotfix` va antes de
    /// `politica_carriles.<carril>.roles_diseno`: la configuracion solo
    /// endurece y sube un hotfix de funcionalidad a sus 3 roles.
    #[test]
    fn tope_de_hotfix_antes_de_la_politica() {
        let mut c = Config::suelo();
        c.politica_carriles.insert(
            Carril::Funcionalidad,
            crate::config::PoliticaCarril {
                roles_diseno: Some(3),
                plan_aprobado: None,
                paradas_extra: Vec::new(),
                revisoras_extra: Vec::new(),
                lote: None,
                perfil: None,
            },
        );
        for (por_complejidad, tipo, config, esperado) in [
            (4, "hotfix", &c, 3),
            (4, "hotfix", &Config::suelo(), 1),
            (4, "feature", &c, 4),
            (0, "hotfix", &c, 3),
        ] {
            assert_eq!(
                Carril::Funcionalidad.roles_diseno(por_complejidad, tipo, config),
                esperado,
                "{por_complejidad} {tipo}"
            );
        }
        let md = md("hotfix", "planificada", &["a"])
            .replace("complejidad: null", "complejidad: \"critica\"");
        let e = estado(&[("TASK-001/tarea.md", md)]);
        let op = Planificar::nueva("TASK-001", &ana(), None, &c, AHORA);
        assert!(op.aplicar(&e).is_ok());
        assert_eq!(op.resumen().map(|s| s.roles.len()), Some(3));
    }

    #[test]
    fn modo_de_la_ultima_fila_plan() {
        let mut c = Cuerpo::plantilla();
        assert_eq!(modo_congelado(&c), None);
        for (fase, modo) in [
            ("plan", "automatico"),
            ("start", "manual"),
            ("plan", "raro"),
        ] {
            c.anadir_transicion(&Transicion {
                fecha: "2026-11-17T10:20:30Z",
                fase,
                modo,
                decidido_por: "agente",
                version: "0.1",
            });
        }
        assert_eq!(modo_congelado(&c).as_deref(), Some("automatico"));
    }
}
