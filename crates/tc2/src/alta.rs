//! Alta de tareas con reserva de ID (F2-4; ADR F1-2, punto 3, y F1-1).
//!
//! El ID se reserva con el push de la rama de estado: [`Alta`] calcula el
//! siguiente ID sobre el estado fresco de cada intento (maximo + 1, nunca el
//! recuento) y crea la carpeta fija `TASK-NNN/` con su `tarea.md`. Si otro
//! clon publico antes, el push se rechaza, la transaccion re-ejecuta la
//! operacion sobre lo nuevo y sale el numero siguiente. La carpeta publicada
//! es el compare-and-swap: reservar un numero sin crearla no reservaria nada.

use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::estado::{Cambio, Estado, Fallo, Operacion};
use crate::i18n::catalogo::estado as txt;
use crate::i18n::{Idioma, Mensaje};
use crate::tarea::texto::{Parecido, parecido, slug};
use crate::tarea::{Cuerpo, Documento, Tarea};

/// Nombre de la operacion y de la orden del binario.
pub const NOMBRE: &str = "new";

/// Prefijo de las carpetas de tarea en la rama de estado.
const PREFIJO: &str = "TASK-";

/// Alta de una tarea: reserva el siguiente ID y escribe su `tarea.md`.
#[derive(Debug)]
pub struct Alta {
    titulo: String,
    fecha: String,
    ocupados: BTreeSet<u32>,
    id: Cell<Option<u32>>,
    parecidas: RefCell<Vec<Parecida>>,
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
            ocupados: BTreeSet::new(),
            id: Cell::new(None),
            parecidas: RefCell::new(Vec::new()),
        }
    }

    /// IDs que ya tienen rama de tarea (`git::ids_con_rama`, leidos una vez
    /// antes de la transaccion): el ID nuevo queda por encima (F2-4b).
    #[must_use]
    pub fn con_ocupados(mut self, ocupados: BTreeSet<u32>) -> Self {
        self.ocupados = ocupados;
        self
    }

    /// Rama derivada del ID del ultimo intento (F2-4b).
    pub fn rama(&self) -> Option<String> {
        self.id().map(|id| rama_de(&id, &self.titulo))
    }

    /// ID calculado en el ultimo intento (el que publico, si la transaccion
    /// salio bien).
    pub fn id(&self) -> Option<String> {
        self.id.get().map(formato_id)
    }

    /// Tareas abiertas parecidas vistas en el ultimo intento.
    pub fn parecidas(&self) -> Vec<Parecida> {
        self.parecidas.borrow().clone()
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
        let rutas = estado.rutas();
        let n = siguiente_id(&rutas, &self.ocupados)?;
        self.id.set(Some(n));
        let id = formato_id(n);
        comprobar_rama(&id, &self.titulo)?;
        // Sobre el mismo estado fresco del intento: dos altas a la vez se ven
        // (la segunda en publicar avisa de la primera, TASK-076 C1).
        *self.parecidas.borrow_mut() = parecidas(estado, &rutas, &self.titulo)?;
        Ok(vec![Cambio::nuevo(
            format!("{id}/tarea.md"),
            tarea_md(&id, &self.titulo, &self.fecha),
        )])
    }
}

/// Estados en los que una tarea esta abierta: contra ellas se avisa.
const ABIERTOS: [&str; 4] = ["planificada", "en-diseno", "en-curso", "en-revision"];

/// Una tarea abierta con un titulo igual o casi igual al del alta (RR-12).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parecida {
    /// ID de la tarea parecida.
    pub id: String,
    /// Su titulo.
    pub titulo: String,
    /// Como se parece.
    pub parecido: Parecido,
}

/// `TASK-<cifras>/tarea.md`.
fn es_tarea_md(ruta: &str) -> bool {
    numero(ruta).is_some() && ruta.split_once('/').is_some_and(|(_, r)| r == "tarea.md")
}

/// Tareas abiertas cuyo titulo se parece a `titulo`, en orden de ID. Los
/// `tarea.md` que no se leen o son de formato 1 se saltan sin fallar.
fn parecidas(estado: &dyn Estado, rutas: &[String], titulo: &str) -> Result<Vec<Parecida>, Fallo> {
    let mds: Vec<String> = rutas.iter().filter(|r| es_tarea_md(r)).cloned().collect();
    let mut v = Vec::new();
    for md in estado.leer_varios(&mds)?.into_iter().flatten() {
        let Ok(Documento::Formato2(t, _)) = crate::tarea::leer(&md) else {
            continue;
        };
        if !ABIERTOS.contains(&t.estado.as_str()) {
            continue;
        }
        if let Some(p) = parecido(titulo, &t.titulo) {
            v.push(Parecida {
                id: t.id,
                titulo: t.titulo,
                parecido: p,
            });
        }
    }
    Ok(v)
}

/// Una linea de aviso con las tareas parecidas, en `idioma`; `None` si no
/// hay. Va a stderr aunque el alta salga bien: es solo un aviso (D1).
pub fn aviso_parecidas(p: &[Parecida], idioma: Idioma) -> Option<String> {
    if p.is_empty() {
        return None;
    }
    let lista: Vec<String> = p
        .iter()
        .map(|x| {
            let m = match x.parecido {
                Parecido::Duplicado => Mensaje::nuevo(&txt::PARECIDA_DUPLICADO),
                Parecido::Casi(c) => Mensaje::nuevo(&txt::PARECIDA_CASI)
                    .con("unidad", (c / 100).to_string())
                    .con("centesimas", format!("{:02}", c % 100)),
            };
            m.con("id", x.id.as_str())
                .con("titulo", x.titulo.as_str())
                .texto(idioma)
        })
        .collect();
    Some(
        Mensaje::nuevo(&txt::AVISO_PARECIDAS)
            .con("lista", lista.join("; "))
            .texto(idioma),
    )
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

/// Siguiente ID: el maximo de las carpetas `TASK-<cifras>` y de los IDs con
/// rama (`ocupados`) mas uno (cuentan terminadas y descartadas, que no se
/// borran; los huecos no se rellenan). Adenda de F2-4b al ADR F1-2 punto 3.
pub fn siguiente_id(rutas: &[String], ocupados: &BTreeSet<u32>) -> Result<u32, Fallo> {
    let maximo = rutas
        .iter()
        .filter_map(|r| numero(r))
        .chain(ocupados.iter().copied())
        .max()
        .unwrap_or(0);
    maximo
        .checked_add(1)
        .ok_or_else(|| Fallo::Error((&crate::i18n::catalogo::estado::SIN_IDS).into()))
}

/// Tipo con el que nace una tarea (plantilla de F1-9, punto 6).
const TIPO: &str = "feature";

/// Rama de la tarea `id` (ADR F1-3 y F1-9 punto 6): `feature/task-nnn-<slug>`
/// con el slug del titulo (F2-4b).
pub fn rama_de(id: &str, titulo: &str) -> String {
    crate::git::nombre_de_rama(id, TIPO, &slug(titulo))
}

/// La rama derivada pasa la validacion de las operaciones de rama: si no,
/// `start` no podria crearla.
fn comprobar_rama(id: &str, titulo: &str) -> Result<(), Fallo> {
    crate::git::rama::rama_valida(NOMBRE, id, TIPO, &slug(titulo))
        .map(drop)
        .map_err(|i| {
            Fallo::Error(
                crate::i18n::Mensaje::nuevo(&crate::i18n::catalogo::estado::RAMA_DERIVADA_INVALIDA)
                    .con("id", id)
                    .con("motivo", i.valor("motivo").unwrap_or_default()),
            )
        })
}

/// `tarea.md` de una tarea recien creada: el modelo de F2-5 con la
/// plantilla de F1-9 (punto 6) y la rama derivada del ID (F2-4b).
pub fn tarea_md(id: &str, titulo: &str, fecha: &str) -> String {
    let mut t = Tarea::nueva(id, titulo, fecha);
    t.rama = Some(rama_de(id, titulo));
    t.escribir(&Cuerpo::plantilla())
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
        let sin = BTreeSet::new();
        assert_eq!(siguiente_id(&r, &sin), Ok(8));
        // Una carpeta en minusculas puesta a mano tambien ocupa su numero.
        assert_eq!(
            siguiente_id(&rutas(&["task-013/tarea.md", "TASK-002/tarea.md"]), &sin),
            Ok(14)
        );
        assert_eq!(siguiente_id(&rutas(&["ñ/x", "TAS"]), &sin), Ok(1));
        assert_eq!(siguiente_id(&[], &sin), Ok(1));
        assert_eq!(siguiente_id(&rutas(&["TASK-999/tarea.md"]), &sin), Ok(1000));
        assert_eq!(formato_id(1000), "TASK-1000");
        assert_eq!(formato_id(8), "TASK-008");
        let tope = rutas(&[&format!("TASK-{}/tarea.md", u32::MAX)]);
        assert!(matches!(siguiente_id(&tope, &sin), Err(Fallo::Error(_))));
        // F2-4b: una rama sin carpeta tambien ocupa su numero (adenda F1-2).
        assert_eq!(siguiente_id(&r, &BTreeSet::from([5, 12])), Ok(13));
        assert_eq!(siguiente_id(&r, &BTreeSet::from([3])), Ok(8));
        assert!(matches!(
            siguiente_id(&[], &BTreeSet::from([u32::MAX])),
            Err(Fallo::Error(_))
        ));
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
        assert!(
            t.contains("rama: \"feature/task-008-c-ruta-x-fecha\"\n"),
            "{t}"
        );
    }

    /// C7: el `tarea.md` lleva `rama: feature/task-nnn-<slug>` y esa rama
    /// pasa `rama_valida`; un titulo sin letras latinas da el slug `tarea`.
    #[test]
    fn rama_derivada_del_id_pasa_rama_valida() {
        for (titulo, rama) in [
            (
                "Nombres únicos de tarea",
                "feature/task-076-nombres-unicos-de-tarea",
            ),
            ("Задача без латиницы", "feature/task-076-tarea"),
            ("漢字", "feature/task-076-tarea"),
        ] {
            assert_eq!(rama_de("TASK-076", titulo), rama);
            let md = tarea_md("TASK-076", titulo, "2026-11-16");
            let leida = match crate::tarea::leer(&md) {
                Ok(crate::tarea::Documento::Formato2(t, _)) => t.rama,
                _ => None,
            };
            assert_eq!(leida.as_deref(), Some(rama));
            let slug = rama.trim_start_matches("feature/task-076-");
            assert_eq!(
                crate::git::rama::rama_valida("t", "TASK-076", TIPO, slug).ok(),
                Some(rama.to_owned())
            );
            assert_eq!(comprobar_rama("TASK-076", titulo), Ok(()));
        }
        assert_eq!(rama_de("TASK-1000", "x"), "feature/task-1000-x");
    }

    #[test]
    fn aviso_de_parecidas_en_los_dos_idiomas() {
        assert_eq!(aviso_parecidas(&[], Idioma::Es), None);
        let p = [
            Parecida {
                id: "TASK-007".into(),
                titulo: "Exportar informes".into(),
                parecido: Parecido::Duplicado,
            },
            Parecida {
                id: "TASK-009".into(),
                titulo: "Exprotar informes".into(),
                parecido: Parecido::Casi(86),
            },
            Parecida {
                id: "TASK-010".into(),
                titulo: "x".into(),
                parecido: Parecido::Casi(100),
            },
        ];
        assert_eq!(
            aviso_parecidas(&p, Idioma::Es).as_deref(),
            Some(
                "aviso: ya hay tareas abiertas con un titulo igual o parecido: TASK-007 \"Exportar informes\" (duplicado); TASK-009 \"Exprotar informes\" (parecido 0,86); TASK-010 \"x\" (parecido 1,00)"
            )
        );
        assert_eq!(
            aviso_parecidas(&p[1..2], Idioma::En).as_deref(),
            Some(
                "warning: there are already open tasks with the same or a similar title: TASK-009 \"Exprotar informes\" (similarity 0.86)"
            )
        );
        assert!(es_tarea_md("TASK-007/tarea.md") && es_tarea_md("task-7/tarea.md"));
        for r in [
            "TASK-007/plan.md",
            "TASK-007/x/tarea.md",
            "notas/tarea.md",
            "TASK-x/tarea.md",
        ] {
            assert!(!es_tarea_md(r), "{r}");
        }
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
