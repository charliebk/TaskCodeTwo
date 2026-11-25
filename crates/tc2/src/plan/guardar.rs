//! `plan guardar` como operacion de la rama de estado (F4-1, D3): publica en
//! `TASK-NNN/planificacion/` un fichero de fuera del repo.
//!
//! Ningun agente puede escribir sus salidas en `tareas/` como en la 0.16:
//! con `tareas/` sucio toda transaccion sale `rechazada
//! motivo=tareas-con-cambios`. Por eso el plan y las salidas de los roles
//! entran por aqui, por el punto unico de escritura.
//!
//! - Destinos: solo `plan-final.md` o `brainstorm/salida-<rol>-<n>.md`, con
//!   `<rol>` uno de los roles de [`super::roles`] ([`destino_valido`]). Una
//!   salida exige su peticion (`peticion-<rol>-<n>.md`) en el estado: no se
//!   guarda la de un rol que `plan` no lanzo.
//! - Solo con la tarea `en-diseno` y sin el plan aprobado; la de otra
//!   persona se rechaza como en `plan`.
//! - El mismo contenido que ya esta publicado: `sin-cambios`. No toca
//!   `tarea.md` (no es una fase).

use std::cell::RefCell;

use super::plantillas::{self as p, BRAINSTORM, PLAN_FINAL};
use super::roles;
use crate::estado::{
    AnotacionConDueno, Cambio, Estado, Fallo, Operacion, por_estado, tarea_propia,
};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::plan as txt;
use crate::identidad::Quien;
use crate::tarea::Asignado;

use super::operacion::EN_DISENO;

/// Nombre de la operacion (en el commit y en la anotacion).
pub const NOMBRE_OP: &str = "plan-guardar";

/// Si `destino` es uno de la lista: `plan-final.md` o
/// `brainstorm/salida-<rol>-<n>.md` con un rol conocido y `n` un entero
/// positivo sin ceros delante. Devuelve, para una salida, el nombre de su
/// peticion.
pub fn destino_valido(destino: &str) -> Option<Option<String>> {
    if destino == PLAN_FINAL {
        return Some(None);
    }
    let resto = destino
        .strip_prefix(BRAINSTORM)?
        .strip_prefix("/salida-")?
        .strip_suffix(".md")?;
    let (rol, n) = resto.rsplit_once('-')?;
    let numero = !n.is_empty() && !n.starts_with('0') && n.bytes().all(|b| b.is_ascii_digit());
    let conocido = roles::todos().any(|r| r.id == rol);
    (numero && conocido && n.parse::<u32>().is_ok()).then(|| Some(format!("peticion-{rol}-{n}.md")))
}

/// Publicar `contenido` en `TASK-NNN/planificacion/<destino>`.
#[derive(Debug)]
pub struct Guardar {
    id: String,
    asignado: Asignado,
    destino: String,
    peticion: Option<String>,
    contenido: String,
    dueno: RefCell<Option<String>>,
}

impl Guardar {
    /// `plan guardar` de `contenido` en `destino` (ya validado con
    /// [`destino_valido`]) de la tarea `id`, por `quien`. Los finales de
    /// linea CRLF pasan a LF.
    pub fn nueva(id: &str, quien: &Quien, destino: &str, contenido: &str) -> Option<Self> {
        let peticion = destino_valido(destino)?;
        Some(Self {
            id: id.to_owned(),
            asignado: quien.asignado(),
            destino: destino.to_owned(),
            peticion,
            contenido: contenido.replace("\r\n", "\n"),
            dueno: RefCell::new(None),
        })
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

impl Operacion for Guardar {
    fn nombre(&self) -> &str {
        NOMBRE_OP
    }

    fn anotacion(&self) -> String {
        AnotacionConDueno {
            orden: NOMBRE_OP,
            id: &self.id,
            asignado: self.asignado.clone(),
            resto: Some(&self.destino),
        }
        .escribir()
    }

    fn exige_red(&self) -> bool {
        // El contenido no cabe en la anotacion: sin red falla, nunca queda
        // pendiente.
        true
    }

    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        let (t, _) = tarea_propia(
            estado,
            &self.id,
            &self.asignado,
            super::NOMBRE,
            ("de-otra-persona", &txt::DE_OTRA_PERSONA),
            &self.dueno,
        )?;
        if t.estado != EN_DISENO {
            return Err(por_estado(&self.id, &t.estado, super::NOMBRE, &[EN_DISENO]));
        }
        if t.plan_aprobado {
            return Err(self.rechazo("plan-aprobado", Mensaje::nuevo(&txt::PLAN_APROBADO)));
        }
        if let Some(pet) = &self.peticion {
            let ruta = p::ruta_brainstorm(&self.id, pet);
            if estado.leer(&ruta)?.is_none() {
                return Err(self.rechazo(
                    "sin-peticion",
                    Mensaje::nuevo(&txt::SIN_PETICION)
                        .con("destino", self.destino.as_str())
                        .con("peticion", pet.as_str()),
                ));
            }
        }
        let ruta = format!("{}/{}/{}", self.id, p::PLANIFICACION, self.destino);
        if estado.leer(&ruta)?.as_deref() == Some(self.contenido.as_str()) {
            return Ok(Vec::new());
        }
        Ok(vec![Cambio::nuevo(ruta, self.contenido.clone())])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::estado::prueba::EnMemoria;
    use crate::identidad::Cuenta;
    use crate::plataforma::Proveedor;
    use crate::tarea::{Cuerpo, Tarea};
    use std::collections::BTreeMap;

    fn ana() -> Quien {
        Quien::Agente(Cuenta {
            proveedor: Proveedor::GitHub,
            id: 7,
            login: "ana".to_owned(),
        })
    }

    /// Criterio 3: la lista de destinos; lo que no esta en ella no vale.
    #[test]
    fn destinos_de_la_lista() {
        assert_eq!(destino_valido("plan-final.md"), Some(None));
        assert_eq!(
            destino_valido("brainstorm/salida-brainstorm-riesgos-2.md"),
            Some(Some("peticion-brainstorm-riesgos-2.md".to_owned()))
        );
        assert_eq!(
            destino_valido("brainstorm/salida-brainstorm-rendimiento-10.md"),
            Some(Some("peticion-brainstorm-rendimiento-10.md".to_owned()))
        );
        for malo in [
            "tarea.md",
            "../tarea.md",
            "plan-final.md.bak",
            "./plan-final.md",
            "brainstorm/peticion-plan-1.md",
            "brainstorm/salida-brainstorm-unificador-1.md",
            "brainstorm/salida-brainstorm-otro-1.md",
            "brainstorm/salida-brainstorm-riesgos-0.md",
            "brainstorm/salida-brainstorm-riesgos-01.md",
            "brainstorm/salida-brainstorm-riesgos-.md",
            "brainstorm/salida-brainstorm-riesgos-1.txt",
            "brainstorm/x/salida-brainstorm-riesgos-1.md",
            "brainstorm\\salida-brainstorm-riesgos-1.md",
            "brainstorm/salida-brainstorm-riesgos-99999999999.md",
        ] {
            assert_eq!(destino_valido(malo), None, "{malo}");
        }
    }

    #[test]
    fn guarda_solo_en_diseno_con_peticion() {
        let md = |estado: &str| {
            let mut t = Tarea::nueva("TASK-001", "Uno", "2026-11-01");
            t.estado = estado.to_owned();
            t.escribir(&Cuerpo::plantilla())
        };
        let pet = "TASK-001/planificacion/brainstorm/peticion-brainstorm-riesgos-1.md";
        let e = EnMemoria(BTreeMap::from([
            ("TASK-001/tarea.md".to_owned(), md("en-diseno")),
            (pet.to_owned(), "p".to_owned()),
            (
                "TASK-001/planificacion/plan-final.md".to_owned(),
                "viejo\n".to_owned(),
            ),
            (
                "TASK-002/tarea.md".to_owned(),
                md("planificada").replace("TASK-001", "TASK-002"),
            ),
        ]));
        let g =
            |id: &str, d: &str, c: &str| Guardar::nueva(id, &ana(), d, c).map(|o| o.aplicar(&e));
        let ok = g("TASK-001", "plan-final.md", "a\r\nb\n");
        assert_eq!(
            ok,
            Some(Ok(vec![Cambio::nuevo(
                "TASK-001/planificacion/plan-final.md",
                "a\nb\n"
            )]))
        );
        let s = g("TASK-001", "brainstorm/salida-brainstorm-riesgos-1.md", "x");
        assert_eq!(s.map(|r| r.map(|c| c.len())), Some(Ok(1)));
        let motivo = |r: Option<Result<Vec<Cambio>, Fallo>>| match r {
            Some(Err(Fallo::Rechazada { motivo, .. })) => motivo,
            otro => format!("{otro:?}"),
        };
        assert_eq!(
            motivo(g(
                "TASK-001",
                "brainstorm/salida-brainstorm-testing-1.md",
                "x"
            )),
            "sin-peticion"
        );
        assert_eq!(
            motivo(g("TASK-002", "plan-final.md", "x")),
            "estado-no-valido"
        );
        assert!(g("TASK-001", "tarea.md", "x").is_none());
        // Lo mismo que ya esta: nada que escribir.
        assert_eq!(
            g("TASK-001", "plan-final.md", "viejo\r\n"),
            Some(Ok(Vec::new()))
        );
        assert!(
            g(
                "TASK-001",
                "brainstorm/peticion-brainstorm-riesgos-1.md",
                "p"
            )
            .is_none()
        );
    }
}
