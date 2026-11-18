//! Estimacion por fase de cada tarea (F3-13; ADR F1-17, «Consecuencias»):
//! el lector del historico, la estimacion, lo que falta y el error, y la
//! escritura del campo `estimacion`.
//!
//! - [`real`]: el real por fase de una tarea terminada (D6a de F1-17).
//! - [`historico`]: las muestras de las terminadas de la rama de estado y de
//!   un `git log` de `develop`, con los descartes por motivo.
//! - [`estimar`](mod@estimar): la cascada de tareas parecidas, [`restante`] y
//!   [`error`]. Todo puro: ni git, ni red, ni modelos (C7).
//! - [`recalcular`]: el unico punto por el que las ordenes ponen el campo
//!   (`new`, `import`, `start` y `tc2 estimar`; D2a de TASK-083). `pausa`,
//!   `reanudar`, `liberar` y `reasignar` no lo tocan.
//! - `tc2 estimar [--json]` rellena las abiertas con `estimacion: null` en
//!   un solo commit de estado; `tc2 estimar TASK-NNN` la ensena recalculada
//!   sin escribir; `tc2 estimar --error TASK-NNN` mide la guardada frente al
//!   real (solo lectura, D4a).

pub mod estimar;
mod forma;
pub mod historico;
pub mod real;

use std::cell::RefCell;
use std::path::Path;

use crate::contrato::{Informe, Resultado, Valor};
use crate::estado::{self, Cambio, Catalogo, Estado, Fallo, Operacion};
use crate::git::{self, Git};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::estimacion as txt;
use crate::tarea::{Cuerpo, Documento, Tarea};

pub use estimar::{
    ErrorFase, Estimacion, Fase, N_MIN, NIVELES, Objetivo, P80, PorFase, Rango, Restante,
    SIN_DATOS, cuantil, error, estimar, mediana, parecidas, restante,
};
pub use forma::minutos;
pub use historico::{Commits, Historico, Muestra, criterios, historico};
pub use real::{Descarte, Eventos, FASES, Real, UMBRAL_MIN, eventos};

/// Nombre de la orden y de la operacion de la rama de estado.
pub const NOMBRE: &str = "estimar";

/// Estados en los que una tarea ya no se estima.
const CERRADOS: [&str; 2] = ["terminada", "descartada"];

/// Los commits de `develop` de un solo `git log` ([`git::historia`]). Si no
/// se pueden leer, ninguno: la estimacion sale igual, con los eventos de los
/// `tarea.md` (la fila de transiciones y de coste).
pub fn commits_de(g: &Git, dir: &Path) -> Commits {
    git::historia(g, dir, NOMBRE)
        .map(|t| Commits::de_log(&t))
        .unwrap_or_default()
}

/// Lo que se sabe de una tarea para estimarla.
pub fn objetivo(t: &Tarea, cuerpo: &Cuerpo) -> Objetivo {
    Objetivo {
        id: t.id.clone(),
        carril: t.carril.as_deref().and_then(crate::carril::Carril::desde),
        complejidad: t.complejidad.clone(),
        criterios: criterios(cuerpo),
    }
}

/// Recalcula la estimacion de `t` con las muestras de `h` y la pone en su
/// campo `estimacion` (el resto del `tarea.md`, byte a byte). `ahora`, en
/// segundos desde 1970, es la hora `calculada`.
pub fn recalcular(t: &mut Tarea, cuerpo: &Cuerpo, h: &Historico, ahora: u64) -> Estimacion {
    let e = estimar(
        &objetivo(t, cuerpo),
        &h.muestras,
        &crate::alta::instante_utc(ahora),
    );
    t.poner_estimacion(&e);
    e
}

/// Pone las claves de la estimacion (`estimacion=`, `estimacion-p80=`,
/// `estimacion-n=`, `estimacion-nivel=`) al final de `i`.
pub fn con_claves(mut i: Informe, e: Option<&Estimacion>) -> Informe {
    if let Some(e) = e {
        i.claves.extend(e.claves());
    }
    i
}

/// `tc2 estimar` sin argumentos: estima las tareas abiertas de formato 2
/// con `estimacion: null` (una sola lectura del estado y del historico, un
/// solo commit). Las terminadas y descartadas no se tocan nunca.
#[derive(Debug)]
pub struct Rellenar {
    ahora: u64,
    commits: Commits,
    ids: RefCell<Vec<String>>,
}

impl Rellenar {
    /// Con los commits de `develop` y la hora del calculo.
    pub fn nuevo(commits: Commits, ahora: u64) -> Self {
        Self {
            ahora,
            commits,
            ids: RefCell::new(Vec::new()),
        }
    }

    /// IDs estimados en el ultimo intento.
    pub fn ids(&self) -> Vec<String> {
        self.ids.borrow().clone()
    }
}

impl Operacion for Rellenar {
    fn nombre(&self) -> &str {
        NOMBRE
    }
    fn anotacion(&self) -> String {
        NOMBRE.to_owned()
    }
    fn exige_red(&self) -> bool {
        // Se recalcula sobre el estado fresco: sin red falla, no queda
        // pendiente.
        true
    }
    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        let leidas = estado::tareas(estado, &estado.rutas())?;
        let h = historico(&leidas, &self.commits);
        let mut cambios = Vec::new();
        let mut ids = Vec::new();
        for (ruta, doc) in &leidas {
            let Ok(Documento::Formato2(t, c)) = doc else {
                continue;
            };
            if CERRADOS.contains(&t.estado.as_str()) || t.estimacion().is_some() {
                continue;
            }
            let mut t = t.clone();
            recalcular(&mut t, c, &h, self.ahora);
            cambios.push(Cambio::nuevo(ruta.clone(), t.escribir(c)));
            ids.push(t.id.clone());
        }
        *self.ids.borrow_mut() = ids;
        Ok(cambios)
    }
}

/// Lo que necesita `tc2 estimar` del entorno.
pub struct Entorno<'a> {
    /// Carpeta desde la que se lanza.
    pub dir: &'a Path,
    /// Puerto Git.
    pub git: &'a Git,
    /// Para re-ejecutar las pendientes de la transaccion.
    pub catalogo: Catalogo<'a>,
    /// Hora del calculo, en segundos desde 1970 (UTC).
    pub ahora: u64,
}

/// `estimar [TASK-NNN | --error TASK-NNN]` (sin `--json`, que quita
/// `lib.rs`): el informe y, si sale bien, la linea para la persona (que va
/// detras de la de contrato y no en JSON).
pub fn orden(args: &[String], e: &Entorno<'_>) -> (Informe, Option<String>) {
    let uso = |motivo: &str| {
        (
            Informe::nuevo(NOMBRE, Resultado::Uso)
                .con("motivo", motivo)
                .diagnostico(&txt::USO_ESTIMAR),
            None,
        )
    };
    let con_id = |id: &str, f: &dyn Fn(&str) -> (Informe, Option<String>)| {
        if git::es_id(id) {
            f(id)
        } else {
            uso("id-invalido")
        }
    };
    match args {
        [] => (rellenar(e), None),
        [a] if a == "--error" => uso("sin-id"),
        [a, id] if a == "--error" => con_id(id, &|id| medir(e, id)),
        [id] if !id.starts_with("--") => con_id(id, &|id| mostrar(e, id)),
        _ => uso("argumentos-de-mas"),
    }
}

/// El informe de una transaccion como de `estimar`.
fn de_estimar(mut i: Informe) -> Informe {
    NOMBRE.clone_into(&mut i.comando);
    i.claves.retain(|(k, _)| k != "operacion");
    i
}

fn rellenar(e: &Entorno<'_>) -> Informe {
    let op = Rellenar::nuevo(commits_de(e.git, e.dir), e.ahora);
    let ctx = estado::Contexto::nuevo(e.git, e.dir, e.catalogo, None);
    let mut i = de_estimar(estado::transaccion(&ctx, &op));
    if i.resultado.es_exito() {
        let ids = if i.resultado == Resultado::Ok {
            op.ids()
        } else {
            Vec::new()
        };
        let mut delante = vec![(
            "actualizadas".to_owned(),
            Valor::Texto(ids.len().to_string()),
        )];
        if !ids.is_empty() {
            delante.push(("ids".to_owned(), Valor::Lista(ids)));
        }
        delante.append(&mut i.claves);
        i.claves = delante;
    }
    i
}

/// La tarea `id` leida del estado conocido (sin `fetch`: solo lectura), con
/// todos los `tarea.md` y los commits.
fn leer(e: &Entorno<'_>, id: &str) -> Result<(Tarea, Cuerpo, Historico, Commits), Informe> {
    let vista = estado::leer(e.git, e.dir).map_err(de_estimar)?;
    let rechazo = |f: Fallo| match f {
        Fallo::Rechazada {
            motivo,
            diagnostico,
        } => Informe::nuevo(NOMBRE, Resultado::Rechazada)
            .con("id", id)
            .con("motivo", motivo)
            .diagnostico(diagnostico),
        Fallo::Error(d) => Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d),
    };
    let (t, c) = estado::tarea_de(&vista, id).map_err(rechazo)?;
    let leidas = estado::tareas(&vista, &vista.rutas()).map_err(rechazo)?;
    let commits = commits_de(e.git, e.dir);
    let h = historico(&leidas, &commits);
    Ok((t, c, h, commits))
}

/// `estimar TASK-NNN`: la estimacion recalculada ahora, sin escribir nada.
fn mostrar(e: &Entorno<'_>, id: &str) -> (Informe, Option<String>) {
    match leer(e, id) {
        Err(i) => (i, None),
        Ok((t, c, h, _)) => {
            let est = estimar(
                &objetivo(&t, &c),
                &h.muestras,
                &crate::alta::instante_utc(e.ahora),
            );
            let i = con_claves(
                Informe::nuevo(NOMBRE, Resultado::Ok).con("id", id),
                Some(&est),
            );
            (i, Some(est.linea()))
        }
    }
}

/// Dos decimales con punto (claves) y con coma (la linea).
fn decimales(x: f64) -> (String, String) {
    let p = format!("{x:.2}");
    let c = p.replace('.', ",");
    (p, c)
}

/// `estimar --error TASK-NNN`: la estimacion guardada frente al real por
/// fase y en total (D4a: solo lectura; guardarlo al cerrar es de F8-7).
fn medir(e: &Entorno<'_>, id: &str) -> (Informe, Option<String>) {
    let rechazada = |motivo: &str, m: Mensaje| {
        (
            Informe::nuevo(NOMBRE, Resultado::Rechazada)
                .con("id", id)
                .con("motivo", motivo)
                .diagnostico(m.con("id", id)),
            None,
        )
    };
    let (t, c, _, commits) = match leer(e, id) {
        Ok(x) => x,
        Err(i) => return (i, None),
    };
    let Some(crudo) = t.estimacion() else {
        return rechazada("sin-estimacion", Mensaje::nuevo(&txt::SIN_ESTIMACION));
    };
    let est = match Estimacion::leer(crudo) {
        Ok(x) => x,
        Err(err) => {
            return rechazada(
                "estimacion-ilegible",
                Mensaje::nuevo(&txt::ESTIMACION_ILEGIBLE).con("error", err),
            );
        }
    };
    let real = match real::real(&eventos(&c, commits.de_tarea(id))) {
        Ok(r) => r,
        Err(d) => {
            let (mut i, _) = rechazada(
                "sin-real",
                Mensaje::nuevo(&txt::SIN_REAL).con("motivo", d.motivo()),
            );
            i = i.con("causa", d.motivo());
            return (i, None);
        }
    };
    let errores = error(&est, &real);
    if errores.is_empty() {
        return rechazada("sin-cifras", Mensaje::nuevo(&txt::SIN_CIFRAS));
    }
    let mut i = Informe::nuevo(NOMBRE, Resultado::Ok).con("id", id);
    let mut partes = Vec::new();
    for x in &errores {
        let (rel, rel_c) = decimales(x.relativo);
        let (razon, razon_c) = decimales(x.razon);
        let abs = minutos(x.absoluto).to_string();
        i = i
            .con(&format!("error-{}", x.fase), abs.clone())
            .con(&format!("relativo-{}", x.fase), rel)
            .con(&format!("razon-{}", x.fase), razon);
        partes.push(format!(
            "{} {abs} min (relativo {rel_c}; estimado/real {razon_c})",
            x.fase
        ));
    }
    (i, Some(format!("error: {}", partes.join("; "))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimales_con_punto_y_con_coma() {
        assert_eq!(decimales(0.6), ("0.60".to_owned(), "0,60".to_owned()));
        assert_eq!(decimales(1.0 / 3.0).0, "0.33");
    }
}
