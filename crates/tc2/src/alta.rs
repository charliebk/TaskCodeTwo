//! Alta de tareas con reserva de ID (F2-4; ADR F1-2, punto 3, y F1-1).
//!
//! El ID se reserva con el push de la rama de estado: [`Alta`] calcula el
//! siguiente ID sobre el estado fresco de cada intento (maximo + 1, nunca el
//! recuento) y crea la carpeta fija `TASK-NNN/` con su `tarea.md`. Si otro
//! clon publico antes, el push se rechaza, la transaccion re-ejecuta la
//! operacion sobre lo nuevo y sale el numero siguiente. La carpeta publicada
//! es el compare-and-swap: reservar un numero sin crearla no reservaria nada.

use std::cell::Cell;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::estado::{Cambio, Estado, Fallo, Operacion};
use crate::tarea::{Cuerpo, Tarea};

/// Nombre de la operacion y de la orden del binario.
pub const NOMBRE: &str = "new";

/// Prefijo de las carpetas de tarea en la rama de estado.
const PREFIJO: &str = "TASK-";

/// Alta de una tarea: reserva el siguiente ID y escribe su `tarea.md`.
#[derive(Debug)]
pub struct Alta {
    titulo: String,
    fecha: String,
    id: Cell<Option<u32>>,
}

impl Alta {
    /// Alta con `titulo` y la fecha de hoy (UTC).
    pub fn nueva(titulo: &str) -> Self {
        Self::con_fecha(titulo, &hoy())
    }

    /// Alta con una fecha `AAAA-MM-DD` dada.
    pub fn con_fecha(titulo: &str, fecha: &str) -> Self {
        Self {
            titulo: titulo.to_owned(),
            fecha: fecha.to_owned(),
            id: Cell::new(None),
        }
    }

    /// ID calculado en el ultimo intento (el que publico, si la transaccion
    /// salio bien).
    pub fn id(&self) -> Option<String> {
        self.id.get().map(formato_id)
    }
}

impl Operacion for Alta {
    fn nombre(&self) -> &str {
        NOMBRE
    }
    fn anotacion(&self) -> String {
        format!("{NOMBRE}|{}", self.titulo)
    }
    fn exige_red(&self) -> bool {
        // Reserva algo compartido: sin red falla, nunca queda pendiente (D2b).
        true
    }
    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        let n = siguiente_id(&estado.rutas())?;
        self.id.set(Some(n));
        let id = formato_id(n);
        Ok(vec![Cambio::nuevo(
            format!("{id}/tarea.md"),
            tarea_md(&id, &self.titulo, &self.fecha),
        )])
    }
}

/// Reconstruye un [`Alta`] desde su anotacion (para el catalogo).
pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
    let titulo = a.strip_prefix(NOMBRE)?.strip_prefix('|')?;
    titulo_valido(titulo).then(|| Box::new(Alta::nueva(titulo)) as Box<dyn Operacion>)
}

pub use crate::tarea::titulo_valido;

/// `TASK-NNN` con al menos tres cifras (`TASK-1000` sin renumerar).
pub fn formato_id(n: u32) -> String {
    format!("{PREFIJO}{n:03}")
}

/// El numero de la carpeta de tarea del primer componente de `ruta`. El
/// prefijo se compara sin mayusculas, como compara las rutas la transaccion:
/// una `task-013/` puesta a mano tambien ocupa el 13.
fn numero(ruta: &str) -> Option<u32> {
    let primero = ruta.split('/').next()?;
    let prefijo = primero.get(..PREFIJO.len())?;
    if !prefijo.eq_ignore_ascii_case(PREFIJO) {
        return None;
    }
    let cifras = primero.get(PREFIJO.len()..)?;
    if cifras.is_empty() || !cifras.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    cifras.parse().ok()
}

/// Siguiente ID: el maximo de las carpetas `TASK-<cifras>` mas uno (cuentan
/// terminadas y descartadas, que no se borran; los huecos no se rellenan).
pub fn siguiente_id(rutas: &[String]) -> Result<u32, Fallo> {
    let maximo = rutas.iter().filter_map(|r| numero(r)).max().unwrap_or(0);
    maximo
        .checked_add(1)
        .ok_or_else(|| Fallo::Error((&crate::i18n::catalogo::estado::SIN_IDS).into()))
}

/// `tarea.md` de una tarea recien creada: el modelo de F2-5 con la
/// plantilla de F1-9 (punto 6).
pub fn tarea_md(id: &str, titulo: &str, fecha: &str) -> String {
    Tarea::nueva(id, titulo, fecha).escribir(&Cuerpo::plantilla())
}

/// Fecha de hoy en UTC, `AAAA-MM-DD`, sin dependencias.
fn hoy() -> String {
    let segundos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    fecha_de_dias(i64::try_from(segundos / 86_400).unwrap_or(0))
}

/// Fecha civil de un numero de dias desde 1970-01-01 (algoritmo de Howard
/// Hinnant, `civil_from_days`).
fn fecha_de_dias(dias: i64) -> String {
    let z = dias + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let a = yoe + era * 400 + i64::from(m <= 2);
    format!("{a:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rutas(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn siguiente_id_es_el_maximo_mas_uno() {
        // TASK-007 es una descartada: cuenta igual; el hueco no se rellena.
        let r = rutas(&[
            "LEEME.md",
            "ROADMAP.md",
            "TASK-001/tarea.md",
            "TASK-007/tarea.md",
            "TASK-007/planificacion/plan-final.md",
            "notas/TASK-050.md",
            "TASK-1a/tarea.md",
            "TASK-/tarea.md",
            "TASK-+9/tarea.md",
        ]);
        assert_eq!(siguiente_id(&r), Ok(8));
        // Una carpeta en minusculas puesta a mano tambien ocupa su numero.
        assert_eq!(
            siguiente_id(&rutas(&["task-013/tarea.md", "TASK-002/tarea.md"])),
            Ok(14)
        );
        assert_eq!(siguiente_id(&rutas(&["ñ/x", "TAS"])), Ok(1));
        assert_eq!(siguiente_id(&[]), Ok(1));
        assert_eq!(siguiente_id(&rutas(&["TASK-999/tarea.md"])), Ok(1000));
        assert_eq!(formato_id(1000), "TASK-1000");
        assert_eq!(formato_id(8), "TASK-008");
        let tope = rutas(&[&format!("TASK-{}/tarea.md", u32::MAX)]);
        assert!(matches!(siguiente_id(&tope), Err(Fallo::Error(_))));
    }

    #[test]
    fn plantilla_escapa_el_titulo_y_sigue_f1_9() {
        let t = tarea_md("TASK-008", "C:\\ruta\\ \"x\" {fecha}", "2026-11-15");
        assert!(
            t.contains("titulo: \"C:\\\\ruta\\\\ \\\"x\\\" {fecha}\"\n"),
            "{t}"
        );
        let claves: Vec<&str> = t
            .lines()
            .skip(1)
            .take_while(|l| *l != "---")
            .filter(|l| !l.starts_with(' '))
            .filter_map(|l| l.split(':').next())
            .collect();
        assert_eq!(
            claves,
            [
                "formato",
                "id",
                "titulo",
                "tipo",
                "sprint",
                "etiquetas",
                "complejidad",
                "carril",
                "estado",
                "motivo",
                "plan_aprobado",
                "rama",
                "asignado_a",
                "roles",
                "estimacion",
                "tokens_diseno",
                "tokens_implementacion",
                "tokens_revision",
                "creado",
                "actualizado",
                "dependencias",
            ]
        );
        assert!(t.contains("id: \"TASK-008\"\n"));
        assert!(t.contains("creado: \"2026-11-15\"\nactualizado: \"2026-11-15\"\n"));
        assert!(t.contains("rama: null\n"));
    }

    #[test]
    fn fechas_civiles() {
        assert_eq!(fecha_de_dias(0), "1970-01-01");
        assert_eq!(fecha_de_dias(11_016), "2000-02-29");
        assert_eq!(fecha_de_dias(20_772), "2026-11-15");
        assert_eq!(fecha_de_dias(-1), "1969-12-31");
    }

    #[test]
    fn anotacion_ida_y_vuelta() {
        let a = Alta::con_fecha("con | barra", "2026-11-15");
        assert_eq!(a.anotacion(), "new|con | barra");
        assert!(desde_anotacion(&a.anotacion()).is_some());
        assert!(desde_anotacion("new|").is_none());
        assert!(desde_anotacion("otra|x").is_none());
        assert!(desde_anotacion("newx|t").is_none());
        assert!(!titulo_valido("a\nb"));
        assert!(!titulo_valido("a\tb"));
        assert!(!titulo_valido("a\u{2028}b"));
        assert!(!titulo_valido("a\u{2029}b"));
        assert!(!titulo_valido("a\u{85}b"));
        assert!(!titulo_valido("   "));
        assert!(titulo_valido("Título con ñ"));
    }
}
