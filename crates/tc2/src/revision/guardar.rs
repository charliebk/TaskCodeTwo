//! El informe de la revision IA en la carpeta de la tarea (F6-5), como
//! operacion de la rama de estado (precedente: `plan guardar`, F4-1).
//!
//! `tc2 review` lee el check `revision-ia` ya terminado
//! ([`crate::plataforma::Plataforma::leer_revision`], con el agente) y lo
//! publica en `TASK-NNN/revision/revision-ia-<sha12>.json`
//! (`$defs/informe-tarea` de `hallazgos.schema.json`) y `.md` (generado por
//! tc2 con el veredicto y la tabla de aceptados por `plano`, sin la prosa del
//! modelo). La CI no escribe en la rama de estado: no tiene la identidad del
//! agente (F1-6, F1-8).
//!
//! - Solo con la tarea `en-revision` y de quien la lleva (la regla de
//!   `review`).
//! - El mismo contenido que ya esta publicado: `sin-cambios`. No toca
//!   `tarea.md`.
//! - Exige red: el contenido no cabe en la anotacion.

use std::cell::RefCell;

use crate::estado::{Cambio, Estado, Fallo, Operacion, de_otra_persona, por_estado, tarea_de};
use crate::hallazgos::InformeTarea;
use crate::i18n::catalogo::revision as txt;
use crate::identidad::Quien;
use crate::tarea::Asignado;

use super::{EN_REVISION, NOMBRE};

/// Nombre de la operacion (en el commit y en la anotacion).
pub const NOMBRE_OP: &str = "revision-guardar";

/// Carpeta de la tarea en la que se guarda.
pub const CARPETA: &str = "revision";

/// Publicar el informe de la revision IA de una tarea.
#[derive(Debug)]
pub struct GuardarRevision {
    id: String,
    asignado: Asignado,
    sha12: String,
    json: String,
    md: String,
    dueno: RefCell<Option<String>>,
}

impl GuardarRevision {
    /// Guardar `informe` en la carpeta de la tarea `id`, por `quien`.
    pub fn nueva(id: &str, quien: &Quien, informe: &InformeTarea) -> Self {
        let json = serde_json::to_string_pretty(informe).unwrap_or_default() + "\n";
        Self {
            id: id.to_owned(),
            asignado: quien.asignado(),
            sha12: informe.sha.chars().take(12).collect(),
            json,
            md: informe.markdown(),
            dueno: RefCell::new(None),
        }
    }

    /// Ruta del JSON en la rama de estado.
    pub fn ruta_json(&self) -> String {
        format!("{}/{CARPETA}/revision-ia-{}.json", self.id, self.sha12)
    }

    fn ruta_md(&self) -> String {
        format!("{}/{CARPETA}/revision-ia-{}.md", self.id, self.sha12)
    }

    /// Login de quien lleva la tarea si el ultimo intento la rechazo por
    /// `de-otra-persona`.
    pub fn dueno(&self) -> Option<String> {
        self.dueno.borrow().clone()
    }
}

impl Operacion for GuardarRevision {
    fn nombre(&self) -> &str {
        NOMBRE_OP
    }

    fn anotacion(&self) -> String {
        format!(
            "{NOMBRE_OP}|{}|{}|{}|{}",
            self.id,
            self.asignado.id().unwrap_or_default(),
            self.asignado.login().unwrap_or_default(),
            self.sha12
        )
    }

    fn exige_red(&self) -> bool {
        // El informe no cabe en la anotacion: sin red falla, nunca queda
        // pendiente.
        true
    }

    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        *self.dueno.borrow_mut() = None;
        let (t, _) = tarea_de(estado, &self.id)?;
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
        if t.estado != EN_REVISION {
            return Err(por_estado(&self.id, &t.estado, NOMBRE, &[EN_REVISION]));
        }
        let mut cambios = Vec::new();
        for (ruta, contenido) in [(self.ruta_json(), &self.json), (self.ruta_md(), &self.md)] {
            if estado.leer(&ruta)?.as_deref() != Some(contenido.as_str()) {
                cambios.push(Cambio::nuevo(ruta, contenido.clone()));
            }
        }
        Ok(cambios)
    }
}
