//! Liberar y reasignar una tarea como operaciones de la rama de estado
//! (F3-10, RR-12; ADR F1-2 D2b: exigen red). Como [`super::Descartar`],
//! pasan por el punto unico de escritura, deciden solo con el estado que
//! reciben y reescriben unicamente el `tarea.md` de su tarea: frontmatter,
//! una fila nueva en `## Transiciones` y el resto byte a byte.
//!
//! - Quien (D3 de TASK-081): el dueno siempre (tambien su agente); una
//!   tarea sin dueno, cualquiera. La de otra persona de la lista, solo con
//!   [`Dejar::umbral`] dias o mas sin actividad (si no, `rechazada
//!   motivo=activa`) y lanzada por una persona en un terminal (el agente no
//!   quita a nadie una tarea: `requiere-persona`). Un `asignado_a` heredado
//!   de la 0.16 no espera los dias, pero tambien lo decide una persona.
//! - Actividad (D2a): la mas reciente de la ultima fila de `## Transiciones`
//!   (la pausa no exime) y del ultimo commit de `origin/<rama>`, que llega
//!   de fuera ya traido (la rama no esta en la rama de estado). Sin ninguna
//!   de las dos no se sabe y no frena.
//! - [`Liberar`] (D6a): `en-curso` pasa a `planificada`; `en-diseno` y
//!   `planificada` se quedan; `asignado_a: null`; `rama` y `plan_aprobado`,
//!   igual. `en-revision` se rechaza (tiene PR: se reasigna). Una ya libre,
//!   `sin-cambios`.
//! - [`Reasignar`]: solo `asignado_a`, a la persona de la lista que llega
//!   ya validada; conserva el estado. Ya suya, `sin-cambios`.

use std::cell::RefCell;

use super::operacion::{ABIERTAS, Lanzada};
use crate::estado::{Cambio, Estado, Fallo, Operacion, por_estado, tarea_de};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::ciclo as txt;
use crate::identidad::Quien;
use crate::situacion::dias_sin_actividad;
use crate::tarea::{Asignado, Cuerpo, Tarea};
use crate::toma::operacion::EN_CURSO;

/// Nombre de `liberar`, de su operacion y de su fila de transicion.
pub const LIBERAR: &str = "liberar";

/// Nombre de `reasignar`, de su operacion y de su fila de transicion.
pub const REASIGNAR: &str = "reasignar";

/// Dias sin actividad por defecto (clave `dias_sin_actividad`, D1a).
pub const DIAS_POR_DEFECTO: u64 = 7;

/// Estado al que vuelve una tarea en curso liberada.
const PLANIFICADA: &str = "planificada";

/// Estados desde los que se libera (D6a).
pub const LIBERABLES: [&str; 3] = [PLANIFICADA, "en-diseno", EN_CURSO];

/// Lo comun de las dos: la tarea, quien la lanza y cuando, la actividad de
/// su rama publicada y el umbral de dias; y lo que vio el ultimo intento.
#[derive(Debug)]
pub struct Dejar {
    base: Lanzada,
    actividad_rama: Option<u64>,
    dias: u64,
    anterior: RefCell<Option<String>>,
    activa: RefCell<Option<u64>>,
}

impl Dejar {
    fn nueva(
        id: &str,
        asignado: Asignado,
        decidido_por: &'static str,
        ahora: u64,
        actividad_rama: Option<u64>,
        dias: u64,
    ) -> Self {
        Self {
            base: Lanzada::nueva(id, asignado, decidido_por, ahora),
            actividad_rama,
            dias,
            anterior: RefCell::new(None),
            activa: RefCell::new(None),
        }
    }

    /// Login de quien la llevaba si el ultimo intento la rechazo por
    /// `activa` o `requiere-persona`.
    pub fn dueno(&self) -> Option<String> {
        self.base.dueno.borrow().clone()
    }

    /// Quien la llevaba (login o texto heredado) si el ultimo intento la
    /// cambio.
    pub fn anterior(&self) -> Option<String> {
        self.anterior.borrow().clone()
    }

    /// Dias sin actividad de la tarea si el ultimo intento la rechazo por
    /// `activa`.
    pub fn activa(&self) -> Option<u64> {
        *self.activa.borrow()
    }

    /// Dias sin actividad a partir de los cuales otra persona puede dejarla.
    pub fn umbral(&self) -> u64 {
        self.dias
    }

    /// Si quien la lanza puede dejar la tarea `t` (D3).
    fn permiso(&self, t: &Tarea, cuerpo: &Cuerpo, orden: &str) -> Result<(), Fallo> {
        *self.base.dueno.borrow_mut() = None;
        *self.activa.borrow_mut() = None;
        let id = self.base.id.as_str();
        let Some(a) = t.asignado_a.as_ref() else {
            return Ok(());
        };
        if a.misma_persona(&self.base.asignado) {
            return Ok(());
        }
        *self.base.dueno.borrow_mut() = a.login().map(str::to_owned);
        if let Some(login) = a.login() {
            let actividad = cuerpo.ultima_fecha().max(self.actividad_rama);
            if let Some(k) = actividad.map(|x| dias_sin_actividad(x, self.base.ahora))
                && k < self.dias
            {
                *self.activa.borrow_mut() = Some(k);
                return Err(Fallo::rechazada(
                    "activa",
                    Mensaje::nuevo(&txt::ACTIVA)
                        .con("id", id)
                        .con("login", login)
                        .con("dias", k.to_string())
                        .con("umbral", self.dias.to_string())
                        .con("orden", orden),
                ));
            }
        }
        if self.base.decidido_por != "persona" {
            let quien = a
                .login()
                .map(|l| format!("@{l}"))
                .or_else(|| a.heredado().map(str::to_owned))
                .unwrap_or_default();
            return Err(Fallo::rechazada(
                "requiere-persona",
                Mensaje::nuevo(&txt::REQUIERE_PERSONA)
                    .con("id", id)
                    .con("quien", quien)
                    .con("orden", orden),
            ));
        }
        Ok(())
    }

    /// Anota quien la llevaba antes de cambiarla.
    fn anotar_anterior(&self, t: &Tarea) {
        *self.anterior.borrow_mut() = t
            .asignado_a
            .as_ref()
            .and_then(|a| a.login().or_else(|| a.heredado()))
            .map(str::to_owned);
    }

    /// `|<actividad o ->|<dias>|<ahora>` de la anotacion.
    fn cola(&self) -> String {
        format!(
            "{}|{}|{}",
            self.actividad_rama
                .map_or_else(|| "-".to_owned(), |x| x.to_string()),
            self.dias,
            self.base.ahora
        )
    }
}

/// Liberar la tarea `id`: queda sin asignar y lista para empezar.
#[derive(Debug)]
pub struct Liberar {
    d: Dejar,
}

impl Liberar {
    /// La liberacion de `id` por `quien` en `ahora` (segundos desde 1970,
    /// UTC), con la actividad de su rama publicada (`actividad_rama`) y el
    /// umbral de `dias`.
    pub fn nueva(
        id: &str,
        quien: &Quien,
        ahora: u64,
        actividad_rama: Option<u64>,
        dias: u64,
    ) -> Self {
        Self {
            d: Dejar::nueva(
                id,
                quien.asignado(),
                quien.decidido_por(),
                ahora,
                actividad_rama,
                dias,
            ),
        }
    }

    /// Lo comun que vio el ultimo intento.
    pub fn dejar(&self) -> &Dejar {
        &self.d
    }
}

impl Operacion for Liberar {
    fn nombre(&self) -> &str {
        LIBERAR
    }

    fn anotacion(&self) -> String {
        format!("{}|{}", self.d.base.anotacion(LIBERAR), self.d.cola())
    }

    fn exige_red(&self) -> bool {
        // Cambia quien lleva una tarea compartida (D2b de F1-2).
        true
    }

    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        *self.d.anterior.borrow_mut() = None;
        let (mut t, cuerpo) = tarea_de(estado, &self.d.base.id)?;
        if !LIBERABLES.contains(&t.estado.as_str()) {
            return Err(por_estado(&self.d.base.id, &t.estado, LIBERAR, &LIBERABLES));
        }
        if t.asignado_a.is_none() && t.estado != EN_CURSO {
            // Ya libre: nada que escribir.
            return Ok(Vec::new());
        }
        self.d.permiso(&t, &cuerpo, LIBERAR)?;
        self.d.anotar_anterior(&t);
        if t.estado == EN_CURSO {
            PLANIFICADA.clone_into(&mut t.estado);
        }
        t.asignado_a = None;
        Ok(self.d.base.escribir(t, cuerpo, LIBERAR))
    }
}

/// Reasignar la tarea `id` a otra persona de la lista.
#[derive(Debug)]
pub struct Reasignar {
    d: Dejar,
    a: Asignado,
}

impl Reasignar {
    /// La reasignacion de `id` a `a` (una persona de la lista, ya validada)
    /// por `quien` en `ahora`, con la actividad de su rama publicada y el
    /// umbral de `dias`.
    pub fn nueva(
        id: &str,
        quien: &Quien,
        a: Asignado,
        ahora: u64,
        actividad_rama: Option<u64>,
        dias: u64,
    ) -> Self {
        Self {
            d: Dejar::nueva(
                id,
                quien.asignado(),
                quien.decidido_por(),
                ahora,
                actividad_rama,
                dias,
            ),
            a,
        }
    }

    /// Lo comun que vio el ultimo intento.
    pub fn dejar(&self) -> &Dejar {
        &self.d
    }

    /// La persona a la que se reasigna.
    pub fn destino(&self) -> &Asignado {
        &self.a
    }
}

impl Operacion for Reasignar {
    fn nombre(&self) -> &str {
        REASIGNAR
    }

    fn anotacion(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.d.base.anotacion(REASIGNAR),
            self.a.id().unwrap_or_default(),
            self.a.login().unwrap_or_default(),
            self.d.cola()
        )
    }

    fn exige_red(&self) -> bool {
        // Cambia quien lleva una tarea compartida (D2b de F1-2).
        true
    }

    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        *self.d.anterior.borrow_mut() = None;
        let (mut t, cuerpo) = tarea_de(estado, &self.d.base.id)?;
        if !ABIERTAS.contains(&t.estado.as_str()) {
            return Err(por_estado(&self.d.base.id, &t.estado, REASIGNAR, &ABIERTAS));
        }
        if t.asignado_a
            .as_ref()
            .is_some_and(|x| x.misma_persona(&self.a))
        {
            // Ya la lleva: nada que escribir.
            return Ok(Vec::new());
        }
        self.d.permiso(&t, &cuerpo, REASIGNAR)?;
        self.d.anotar_anterior(&t);
        t.asignado_a = Some(self.a.clone());
        Ok(self.d.base.escribir(t, cuerpo, REASIGNAR))
    }
}

/// Reconstruye una operacion de este modulo desde su anotacion (para el
/// catalogo): `liberar|TASK-NNN|<id>|<login>|<actividad o ->|<dias>|<ahora>`
/// o `reasignar|TASK-NNN|<id>|<login>|<id destino>|<login destino>|<actividad
/// o ->|<dias>|<ahora>`. Exigen red y no quedan pendientes; si se
/// re-ejecutaran, sin persona delante: `decidido_por: agente`.
pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
    let partes: Vec<&str> = a.split('|').collect();
    let (nombre, id) = (*partes.first()?, *partes.get(1)?);
    if !crate::git::es_id(id) {
        return None;
    }
    let asignado = Asignado::persona(partes.get(2)?.parse().ok()?, partes.get(3)?)?;
    let cola = |p: &[&str]| -> Option<(Option<u64>, u64, u64)> {
        let [actividad, dias, ahora] = p else {
            return None;
        };
        let actividad = match *actividad {
            "-" => None,
            x => Some(x.parse().ok()?),
        };
        Some((actividad, dias.parse().ok()?, ahora.parse().ok()?))
    };
    match nombre {
        LIBERAR => {
            let (actividad, dias, ahora) = cola(partes.get(4..)?)?;
            Some(Box::new(Liberar {
                d: Dejar::nueva(id, asignado, "agente", ahora, actividad, dias),
            }))
        }
        REASIGNAR => {
            let destino = Asignado::persona(partes.get(4)?.parse().ok()?, partes.get(5)?)?;
            let (actividad, dias, ahora) = cola(partes.get(6..)?)?;
            Some(Box::new(Reasignar {
                d: Dejar::nueva(id, asignado, "agente", ahora, actividad, dias),
                a: destino,
            }))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grafo::DESCARTADA;
    use crate::identidad::Cuenta;
    use crate::plataforma::Proveedor;
    use crate::roadmap::tests::EnMemoria;
    use std::collections::BTreeMap;

    /// 2026-11-17T10:20:30Z.
    const AHORA: u64 = 1_794_910_830;
    const DIA: u64 = 86_400;

    fn cuenta(id: u64, login: &str) -> Cuenta {
        Cuenta {
            proveedor: Proveedor::GitHub,
            id,
            login: login.to_owned(),
        }
    }

    /// Ana desde su agente (sin terminal).
    fn ana() -> Quien {
        Quien::Agente(cuenta(7, "ana"))
    }

    /// Beto en un terminal.
    fn beto() -> Quien {
        Quien::Persona(cuenta(9, "beto"))
    }

    fn persona(id: u64, login: &str) -> Asignado {
        Asignado::persona(id, login).unwrap_or_else(|| unreachable!())
    }

    /// Un `tarea.md` con una fila de transicion de hace `hace` segundos.
    fn md(id: &str, estado: &str, asignado: Option<Asignado>, hace: u64) -> String {
        let mut t = Tarea::nueva(id, &format!("Tarea {id}"), "2026-11-01");
        t.estado = estado.to_owned();
        if estado == DESCARTADA {
            t.motivo = Some("no".to_owned());
        }
        t.plan_aprobado = true;
        t.asignado_a = asignado;
        t.rama = Some(crate::alta::rama_de(id, &t.tipo, &t.titulo));
        let fila = format!(
            "| {} | start | manual | persona | 0.1 |\n",
            crate::toma::instante(AHORA - hace)
        );
        t.escribir(&Cuerpo::plantilla()) + &fila
    }

    fn estado(tareas: &[(&str, String)]) -> EnMemoria {
        EnMemoria(
            tareas
                .iter()
                .map(|(id, t)| (format!("{id}/tarea.md"), t.clone()))
                .collect::<BTreeMap<_, _>>(),
        )
    }

    fn motivo(r: &Result<Vec<Cambio>, Fallo>) -> Option<&str> {
        match r {
            Err(Fallo::Rechazada { motivo, .. }) => Some(motivo),
            _ => None,
        }
    }

    fn texto(r: &Result<Vec<Cambio>, Fallo>) -> String {
        match r.as_deref() {
            Ok([c]) => c.contenido.clone(),
            otro => format!("{otro:?}"),
        }
    }

    fn fila(fase: &str, por: &str) -> String {
        format!(
            "| 2026-11-17T10:20:30Z | {fase} | manual | {por} | {} |\n",
            crate::VERSION
        )
    }

    /// C1 (unitario): el dueno (su agente) libera una en curso: `planificada`,
    /// `asignado_a: null`, `actualizado` y la fila; `rama` y `plan_aprobado`
    /// y el resto, igual. En diseno se queda en diseno.
    #[test]
    fn liberar_escribe_solo_lo_suyo() {
        let yo = ana().asignado();
        for (de, a) in [("en-curso", "planificada"), ("en-diseno", "en-diseno")] {
            let antes = md("TASK-001", de, Some(yo.clone()), 0);
            let s = estado(&[("TASK-001", antes.clone())]);
            let op = Liberar::nueva("TASK-001", &ana(), AHORA, None, 7);
            let r = op.aplicar(&s);
            let esperado = antes
                .replace(&format!("estado: \"{de}\""), &format!("estado: \"{a}\""))
                .replace("asignado_a: {id: 7, login: \"ana\"}", "asignado_a: null")
                .replace("actualizado: \"2026-11-01\"", "actualizado: \"2026-11-17\"")
                + &fila("liberar", "agente");
            assert_eq!(texto(&r), esperado, "{de}");
            assert!(esperado.contains("\nplan_aprobado: true\n"));
            assert!(esperado.contains("\nrama: \"feature/task-001-tarea-task-001\"\n"));
            assert_eq!(op.dejar().anterior().as_deref(), Some("ana"));
            assert!(op.exige_red());
        }
    }

    /// C3 (unitario): reasignar cambia solo `asignado_a` (al objeto de la
    /// lista) y anade la fila; el estado se conserva. Ya suya, nada.
    #[test]
    fn reasignar_escribe_solo_el_asignado() {
        let yo = ana().asignado();
        for e in ABIERTAS {
            let antes = md("TASK-001", e, Some(yo.clone()), 0);
            let s = estado(&[("TASK-001", antes.clone())]);
            let op = Reasignar::nueva("TASK-001", &ana(), persona(9, "beto"), AHORA, None, 7);
            let esperado = antes
                .replace(
                    "asignado_a: {id: 7, login: \"ana\"}",
                    "asignado_a: {id: 9, login: \"beto\"}",
                )
                .replace("actualizado: \"2026-11-01\"", "actualizado: \"2026-11-17\"")
                + &fila("reasignar", "agente");
            assert_eq!(texto(&op.aplicar(&s)), esperado, "{e}");
            assert_eq!(op.dejar().anterior().as_deref(), Some("ana"));
            // A quien ya la lleva: sin cambios.
            let op = Reasignar::nueva("TASK-001", &ana(), persona(7, "ana"), AHORA, None, 7);
            assert_eq!(op.aplicar(&s), Ok(Vec::new()), "{e}");
        }
    }

    /// C5 (unitario): la de otra persona con actividad de hace menos de N
    /// dias (por la fila o por el commit de la rama) se rechaza `activa`
    /// con los dias; con N o mas, procede (una persona en un terminal).
    #[test]
    fn otra_persona_activa_o_inactiva() {
        let de_ana = Some(persona(7, "ana"));
        let reciente = estado(&[("TASK-001", md("TASK-001", "en-curso", de_ana.clone(), DIA))]);
        let vieja = estado(&[("TASK-001", md("TASK-001", "en-curso", de_ana, 7 * DIA))]);
        // (motivo del rechazo o «ok», dias de `activa`, dueno) de liberar o
        // reasignar con la actividad de la rama `rama`.
        let probar = |s: &EnMemoria, reasignar: bool, rama: Option<u64>| {
            let (r, d) = if reasignar {
                let op = Reasignar::nueva("TASK-001", &beto(), persona(9, "beto"), AHORA, rama, 7);
                (op.aplicar(s), (op.dejar().activa(), op.dejar().dueno()))
            } else {
                let op = Liberar::nueva("TASK-001", &beto(), AHORA, rama, 7);
                (op.aplicar(s), (op.dejar().activa(), op.dejar().dueno()))
            };
            let m = match &r {
                Ok(c) if c.len() == 1 => "ok".to_owned(),
                otro => motivo(otro).unwrap_or("?").to_owned(),
            };
            (m, d.0, d.1)
        };
        for reasignar in [false, true] {
            let (m, dias, dueno) = probar(&reciente, reasignar, None);
            assert_eq!(
                (m.as_str(), dias, dueno.as_deref()),
                ("activa", Some(1), Some("ana"))
            );
            let (m, dias, _) = probar(&vieja, reasignar, None);
            assert_eq!((m.as_str(), dias), ("ok", None));
            // Un commit reciente en la rama la mantiene activa aunque la
            // fila sea vieja.
            let (m, dias, _) = probar(&vieja, reasignar, Some(AHORA - 2 * DIA));
            assert_eq!((m.as_str(), dias), ("activa", Some(2)));
            // Una actividad del futuro cuenta como de hoy.
            let (m, dias, _) = probar(&vieja, reasignar, Some(AHORA + 30 * DIA));
            assert_eq!((m.as_str(), dias), ("activa", Some(0)));
        }
    }

    /// C6 (unitario): el agente no quita a nadie una tarea aunque haya
    /// pasado N (`requiere-persona`); sobre la propia procede; una sin dueno,
    /// cualquiera.
    #[test]
    fn el_agente_no_quita_tareas() {
        let vieja = |a: Option<Asignado>| md("TASK-001", "en-curso", a, 30 * DIA);
        let s = estado(&[("TASK-001", vieja(Some(persona(9, "beto"))))]);
        let op = Liberar::nueva("TASK-001", &ana(), AHORA, None, 7);
        assert_eq!(motivo(&op.aplicar(&s)), Some("requiere-persona"));
        assert_eq!(op.dejar().dueno().as_deref(), Some("beto"));
        let op = Reasignar::nueva("TASK-001", &ana(), persona(7, "ana"), AHORA, None, 7);
        assert_eq!(motivo(&op.aplicar(&s)), Some("requiere-persona"));
        // Propia (de ana) o sin dueno: el agente si.
        for a in [Some(persona(7, "ana")), None] {
            let s = estado(&[("TASK-001", vieja(a))]);
            let r = Liberar::nueva("TASK-001", &ana(), AHORA, None, 7).aplicar(&s);
            assert!(r.is_ok_and(|c| c.len() == 1));
        }
    }

    /// C11 (unitario): liberar una terminada, descartada o en revision se
    /// rechaza por estado; una ya libre, sin cambios. Reasignar una
    /// terminada o descartada, tambien por estado.
    #[test]
    fn rechazos_por_estado_y_ya_libre() {
        let yo = Some(ana().asignado());
        let s = estado(&[
            ("TASK-001", md("TASK-001", "terminada", yo.clone(), 0)),
            ("TASK-002", md("TASK-002", DESCARTADA, yo.clone(), 0)),
            ("TASK-003", md("TASK-003", "en-revision", yo, 0)),
            ("TASK-004", md("TASK-004", "planificada", None, 0)),
            ("TASK-005", md("TASK-005", "en-diseno", None, 0)),
        ]);
        for (id, esperado) in [
            ("TASK-001", "terminada"),
            ("TASK-002", "descartada"),
            ("TASK-003", "estado-no-valido"),
            ("TASK-050", "no-existe"),
        ] {
            let r = Liberar::nueva(id, &ana(), AHORA, None, 7).aplicar(&s);
            assert_eq!(motivo(&r), Some(esperado), "{id}");
        }
        for id in ["TASK-004", "TASK-005"] {
            let op = Liberar::nueva(id, &ana(), AHORA, None, 7);
            assert_eq!(op.aplicar(&s), Ok(Vec::new()), "{id}");
            assert_eq!(op.dejar().anterior(), None);
        }
        for (id, esperado) in [("TASK-001", "terminada"), ("TASK-002", "descartada")] {
            let r = Reasignar::nueva(id, &ana(), persona(9, "beto"), AHORA, None, 7).aplicar(&s);
            assert_eq!(motivo(&r), Some(esperado), "{id}");
        }
        // En revision si se reasigna (D6a).
        let r =
            Reasignar::nueva("TASK-003", &ana(), persona(9, "beto"), AHORA, None, 7).aplicar(&s);
        assert!(r.is_ok_and(|c| c.len() == 1));
    }

    /// C12 (unitario): un `asignado_a` heredado de la 0.16 se reasigna sin
    /// esperar N dias (lo decide una persona) y queda como objeto; el
    /// agente, `requiere-persona`.
    #[test]
    fn heredado_se_reasigna_a_un_objeto() {
        let heredado = md("TASK-001", "en-curso", None, 0)
            .replace("asignado_a: null", "asignado_a: \"ana@x.com\"");
        let s = estado(&[("TASK-001", heredado.clone())]);
        let op = Reasignar::nueva("TASK-001", &beto(), persona(9, "beto"), AHORA, None, 7);
        let esperado = heredado
            .replace(
                "asignado_a: \"ana@x.com\"",
                "asignado_a: {id: 9, login: \"beto\"}",
            )
            .replace("actualizado: \"2026-11-01\"", "actualizado: \"2026-11-17\"")
            + &fila("reasignar", "persona");
        assert_eq!(texto(&op.aplicar(&s)), esperado);
        assert_eq!(op.dejar().anterior().as_deref(), Some("ana@x.com"));
        let op = Reasignar::nueva("TASK-001", &ana(), persona(9, "beto"), AHORA, None, 7);
        assert_eq!(motivo(&op.aplicar(&s)), Some("requiere-persona"));
        assert_eq!(op.dejar().dueno(), None);
    }

    /// Las anotaciones se reconstruyen igual y las malas no.
    #[test]
    fn anotacion_ida_y_vuelta() {
        let ops: [Box<dyn Operacion>; 3] = [
            Box::new(Liberar::nueva("TASK-001", &ana(), AHORA, None, 7)),
            Box::new(Liberar::nueva("TASK-001", &ana(), AHORA, Some(5), 30)),
            Box::new(Reasignar::nueva(
                "TASK-001",
                &ana(),
                persona(9, "beto"),
                AHORA,
                Some(5),
                7,
            )),
        ];
        let anotaciones: Vec<String> = ops.iter().map(|o| o.anotacion()).collect();
        assert_eq!(
            anotaciones,
            [
                "liberar|TASK-001|7|ana|-|7|1794910830",
                "liberar|TASK-001|7|ana|5|30|1794910830",
                "reasignar|TASK-001|7|ana|9|beto|5|7|1794910830",
            ]
        );
        for (o, a) in ops.iter().zip(&anotaciones) {
            let r = desde_anotacion(a);
            assert_eq!(
                r.as_ref()
                    .map(|x| (x.nombre().to_owned(), x.anotacion(), x.exige_red())),
                Some((o.nombre().to_owned(), a.clone(), true))
            );
        }
        for mala in [
            "liberar|TASK-001|7|ana|-|7",
            "liberar|TASK-001|7|ana|x|7|1",
            "liberar|TASK-001|7|ana|-|7|1|2",
            "liberar|TASK-1|7|ana|-|7|1",
            "liberar|TASK-001|0|ana|-|7|1",
            "reasignar|TASK-001|7|ana|0|beto|-|7|1",
            "reasignar|TASK-001|7|ana|9||-|7|1",
            "reasignar|TASK-001|7|ana|9|beto|-|7",
            "descartar|TASK-001|7|ana|m",
        ] {
            assert!(desde_anotacion(mala).is_none(), "{mala}");
        }
    }
}
