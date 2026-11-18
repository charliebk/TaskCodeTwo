//! El real por fase de una tarea terminada (ADR F1-17, decision 1 y D6a),
//! recalculado desde sus eventos: las filas de `## Transiciones`, las de
//! `## Coste` y los commits que la nombran o estan en su rama.
//!
//! - Fases medibles (D7a): implementacion (de `start` a la primera
//!   `review`), rondas (de la primera a la ultima `review`) y cierre (de la
//!   ultima `review` a `finish`). El diseno queda sin calibrar.
//! - El real de una fase es la suma de los huecos entre eventos
//!   consecutivos, sin las ventanas de `pausa` (desde la `pausa` hasta la
//!   transicion siguiente que no sea otra `pausa`, como la 0.16), y con cada
//!   hueco de mas de [`UMBRAL_MIN`] minutos contado como [`UMBRAL_MIN`]
//!   (divergencia 6 de F1-17: tope, no eliminacion).
//! - Mismas reglas, en el mismo orden, que `real`, `tramo` y
//!   `ventanas_pausa` de `poc-f1-17.py` (paridad en `tests/estimacion.rs`).
//!
//! Puro: no lee git ni ficheros.

use crate::tarea::Cuerpo;

/// Minutos a partir de los cuales un hueco sin eventos cuenta solo eso
/// (D6a de F1-17).
pub const UMBRAL_MIN: f64 = 45.0;

/// Las fases medibles, en su orden (D7a).
pub const FASES: [&str; 3] = ["implementacion", "rondas", "cierre"];

/// Una fila de `## Transiciones` que se entiende: instante (segundos desde
/// 1970, UTC) y fase.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Paso {
    /// Instante de la fila.
    pub ts: i64,
    /// Fase (`start`, `review`, `pausa`...).
    pub fase: String,
}

/// Los eventos de una tarea.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Eventos {
    /// Filas de `## Transiciones` con fecha valida.
    pub transiciones: Vec<Paso>,
    /// Si la tabla de transiciones tenia alguna fila (aunque ilegible).
    pub con_filas: bool,
    /// Instantes de las filas de `## Coste`.
    pub coste: Vec<i64>,
    /// Instantes de los commits de la tarea.
    pub commits: Vec<i64>,
    /// Filas que no se entendieron (fecha o total ilegibles).
    pub filas_malas: usize,
}

/// Minutos reales por fase.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Real {
    /// De `start` a la primera `review`.
    pub implementacion: f64,
    /// De la primera a la ultima `review`.
    pub rondas: f64,
    /// De la ultima `review` a `finish`.
    pub cierre: f64,
}

impl Real {
    /// Suma de las tres fases (en el orden de la PoC).
    pub fn total(&self) -> f64 {
        self.implementacion + self.rondas + self.cierre
    }

    /// La fase por su nombre (`total` incluido).
    pub fn fase(&self, f: &str) -> Option<f64> {
        match f {
            "implementacion" => Some(self.implementacion),
            "rondas" => Some(self.rondas),
            "cierre" => Some(self.cierre),
            "total" => Some(self.total()),
            _ => None,
        }
    }
}

/// Por que una tarea terminada no da muestra (los motivos de la PoC).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Descarte {
    /// `## Transiciones` sin ninguna fila.
    SinTransiciones,
    /// Ninguna fila `start` con fecha.
    SinStart,
    /// Ninguna fila `finish` con fecha.
    SinFinish,
    /// Ninguna fila `review` con fecha.
    SinReview,
    /// Una `review` antes del `start` o despues del `finish`: una fase
    /// tendria duracion negativa (dato invalido).
    DuracionNegativa,
}

impl Descarte {
    /// El motivo en la salida (`duracion-negativa`...).
    pub fn motivo(self) -> &'static str {
        match self {
            Self::SinTransiciones => "sin-transiciones",
            Self::SinStart => "sin-start",
            Self::SinFinish => "sin-finish",
            Self::SinReview => "sin-review",
            Self::DuracionNegativa => "duracion-negativa",
        }
    }
}

/// Segundos desde 1970 de `AAAA-MM-DDTHH:MM:SSZ` exacto (la fecha que
/// escriben la 0.16 y la 2.0); cualquier otra forma, `None` (fila ilegible,
/// como `fecha()` de la PoC).
pub fn segundos_de(fecha: &str) -> Option<i64> {
    let b = fecha.as_bytes();
    let forma = b.len() == 20
        && b.iter().enumerate().all(|(i, c)| match i {
            4 | 7 => *c == b'-',
            10 => *c == b'T',
            13 | 16 => *c == b':',
            19 => *c == b'Z',
            _ => c.is_ascii_digit(),
        });
    if !forma {
        return None;
    }
    crate::inventario::ms_de_iso(fecha).map(|ms| ms.div_euclid(1000))
}

/// Las filas de la primera tabla Markdown de `texto` como mapas cabecera →
/// celda (como `tabla_md` de la PoC: la primera fila es la cabecera y la
/// segunda, el separador). Sin cabecera y separador, ninguna.
pub fn tabla(texto: &str) -> Vec<Vec<(String, String)>> {
    let filas: Vec<Vec<String>> = texto
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with('|'))
        .map(|l| {
            l.trim_matches('|')
                .split('|')
                .map(|c| c.trim().to_owned())
                .collect()
        })
        .collect();
    let Some(cabecera) = filas.first() else {
        return Vec::new();
    };
    if filas.len() < 2 {
        return Vec::new();
    }
    filas
        .iter()
        .skip(2)
        .map(|f| cabecera.iter().cloned().zip(f.iter().cloned()).collect())
        .collect()
}

/// Celda `clave` de una fila de [`tabla`].
fn celda<'a>(fila: &'a [(String, String)], clave: &str) -> Option<&'a str> {
    fila.iter()
        .find(|(k, _)| k == clave)
        .map(|(_, v)| v.as_str())
}

/// Los eventos de `cuerpo` mas los commits de la tarea.
pub fn eventos(cuerpo: &Cuerpo, commits: Vec<i64>) -> Eventos {
    let mut ev = Eventos {
        commits,
        ..Eventos::default()
    };
    let texto = |t: &str| cuerpo.seccion(t).map_or("", |s| s.texto.as_str());
    let filas = tabla(texto("Transiciones"));
    ev.con_filas = !filas.is_empty();
    for f in &filas {
        match segundos_de(celda(f, "fecha").unwrap_or_default()) {
            Some(ts) => ev.transiciones.push(Paso {
                ts,
                fase: celda(f, "fase").unwrap_or_default().to_owned(),
            }),
            None => ev.filas_malas += 1,
        }
    }
    for f in tabla(texto("Coste")) {
        let total = celda(&f, "total").unwrap_or_default();
        let total_ok = total.is_empty() || total.parse::<i64>().is_ok();
        match celda(&f, "fecha").and_then(segundos_de) {
            Some(ts) if total_ok => ev.coste.push(ts),
            _ => ev.filas_malas += 1,
        }
    }
    ev
}

/// Ventanas de pausa: de cada `pausa` a la transicion siguiente que no sea
/// otra `pausa` (en el orden por instante y fase), si es posterior.
pub fn ventanas_pausa(transiciones: &[Paso]) -> Vec<(i64, i64)> {
    let mut ts: Vec<&Paso> = transiciones.iter().collect();
    ts.sort();
    let mut v = Vec::new();
    for (i, p) in ts.iter().enumerate() {
        if p.fase != crate::tarea::PAUSA {
            continue;
        }
        let sig = ts
            .get(i + 1..)
            .unwrap_or_default()
            .iter()
            .find(|q| q.fase != crate::tarea::PAUSA)
            .map(|q| q.ts);
        if let Some(s) = sig
            && s > p.ts
        {
            v.push((p.ts, s));
        }
    }
    v
}

/// Minutos activos entre `ini` y `fin`: los huecos entre los eventos de
/// dentro (y los bordes de las pausas), sin los que caen en una pausa, y
/// con cada hueco de mas de [`UMBRAL_MIN`] contado como [`UMBRAL_MIN`].
pub fn tramo(ini: i64, fin: i64, eventos: &[i64], pausas: &[(i64, i64)]) -> f64 {
    let mut pts: Vec<i64> = vec![ini, fin];
    pts.extend(eventos.iter().copied().filter(|e| ini < *e && *e < fin));
    for (a, b) in pausas {
        pts.extend([*a, *b].into_iter().filter(|x| ini < *x && *x < fin));
    }
    pts.sort_unstable();
    pts.dedup();
    let mut suma = 0.0;
    for par in pts.windows(2) {
        let (a, b) = (par[0], par[1]);
        // El punto medio, como la PoC, en coma flotante.
        let medio = (a as f64 + b as f64) / 2.0;
        if pausas
            .iter()
            .any(|(pa, pb)| *pa as f64 <= medio && medio <= *pb as f64)
        {
            continue;
        }
        let h = (b - a) as f64 / 60.0;
        suma += h.min(UMBRAL_MIN);
    }
    suma
}

/// El real por fase, o por que la tarea no da muestra.
pub fn real(ev: &Eventos) -> Result<Real, Descarte> {
    if !ev.con_filas {
        return Err(Descarte::SinTransiciones);
    }
    let de = |f: &'static str| ev.transiciones.iter().filter(move |p| p.fase == f);
    let start = de("start").map(|p| p.ts).min().ok_or(Descarte::SinStart)?;
    let finish = de("finish")
        .map(|p| p.ts)
        .max()
        .ok_or(Descarte::SinFinish)?;
    let mut reviews: Vec<i64> = de("review").map(|p| p.ts).collect();
    reviews.sort_unstable();
    let (Some(&r1), Some(&rn)) = (reviews.first(), reviews.last()) else {
        return Err(Descarte::SinReview);
    };
    if !(start <= r1 && rn <= finish) {
        return Err(Descarte::DuracionNegativa);
    }
    let mut todos: Vec<i64> = ev.commits.clone();
    todos.extend(&ev.coste);
    todos.extend(ev.transiciones.iter().map(|p| p.ts));
    todos.sort_unstable();
    todos.dedup();
    let pausas = ventanas_pausa(&ev.transiciones);
    Ok(Real {
        implementacion: tramo(start, r1, &todos, &pausas),
        rondas: tramo(r1, rn, &todos, &pausas),
        cierre: tramo(rn, finish, &todos, &pausas),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Segundos de `HH:MM` del 2026-11-17 (UTC).
    fn h(hh: i64, mm: i64) -> i64 {
        1_794_873_600 + hh * 3600 + mm * 60
    }

    fn paso(ts: i64, fase: &str) -> Paso {
        Paso {
            ts,
            fase: fase.to_owned(),
        }
    }

    /// Los eventos de C1: start 10:00, commit 10:20, pausa 10:30, reanudar
    /// 11:30, commit 11:40 (si `con_1140`), review 13:00 y 13:30, finish
    /// 13:40.
    fn c1(con_1140: bool) -> Eventos {
        let mut commits = vec![h(10, 20)];
        if con_1140 {
            commits.push(h(11, 40));
        }
        Eventos {
            transiciones: vec![
                paso(h(10, 0), "start"),
                paso(h(10, 30), "pausa"),
                paso(h(11, 30), "reanudar"),
                paso(h(13, 0), "review"),
                paso(h(13, 30), "review"),
                paso(h(13, 40), "finish"),
            ],
            con_filas: true,
            commits,
            ..Eventos::default()
        }
    }

    #[test]
    fn fechas_exactas_y_tablas() {
        assert_eq!(segundos_de("2026-11-17T10:00:00Z"), Some(h(10, 0)));
        for malo in [
            "2026-11-17",
            "2026-11-17T10:00:00",
            "2026-11-17 10:00:00Z",
            "2026-11-17T10:00:00+00:00",
            "2026-13-17T10:00:00Z",
            "x",
        ] {
            assert_eq!(segundos_de(malo), None, "{malo}");
        }
        let t = tabla("## X\n| a | b |\n|---|---|\n| 1 | 2 |\n|| 3 |\ntexto\n| 4 |\n");
        assert_eq!(
            t,
            vec![
                vec![("a".into(), "1".into()), ("b".into(), "2".into())],
                vec![("a".into(), "3".into())],
                vec![("a".into(), "4".into())],
            ]
        );
        assert!(tabla("| solo cabecera |\n").is_empty());
    }

    /// C1: el real por fase con la regla de F1-17 (paridad con la PoC en
    /// `tests/estimacion.rs`): con el commit de las 11:40, 85/30/10; sin el,
    /// 75/30/10 (el 75 del enunciado de C1 es el de este segundo caso).
    #[test]
    fn c1_real_por_fase_con_pausa_y_tope() {
        let r = real(&c1(true));
        assert_eq!(
            r,
            Ok(Real {
                implementacion: 85.0,
                rondas: 30.0,
                cierre: 10.0
            })
        );
        let r = real(&c1(false));
        assert_eq!(
            r.map(|r| (r.implementacion, r.rondas, r.cierre, r.total())),
            Ok((75.0, 30.0, 10.0, 115.0))
        );
    }

    /// C1: una `review` antes del `start` es duracion negativa; y los demas
    /// descartes, por su orden.
    #[test]
    fn c1_duracion_negativa_y_descartes() {
        let mut e = c1(true);
        e.transiciones[3].ts = h(9, 0);
        assert_eq!(real(&e), Err(Descarte::DuracionNegativa));
        assert_eq!(Descarte::DuracionNegativa.motivo(), "duracion-negativa");
        let mut e = c1(true);
        e.transiciones.retain(|p| p.fase != "review");
        assert_eq!(real(&e), Err(Descarte::SinReview));
        let mut e = c1(true);
        e.transiciones.retain(|p| p.fase != "finish");
        assert_eq!(real(&e), Err(Descarte::SinFinish));
        let mut e = c1(true);
        e.transiciones.retain(|p| p.fase != "start");
        assert_eq!(real(&e), Err(Descarte::SinStart));
        assert_eq!(real(&Eventos::default()), Err(Descarte::SinTransiciones));
    }

    /// Una pausa sin `reanudar` dura hasta la transicion siguiente (aqui,
    /// `finish`); dos `pausa` seguidas no cierran la ventana.
    #[test]
    fn pausa_sin_reanudar_y_pausas_seguidas() {
        let t = vec![
            paso(h(10, 0), "start"),
            paso(h(10, 10), "review"),
            paso(h(10, 20), "pausa"),
            paso(h(10, 25), "pausa"),
            paso(h(12, 0), "finish"),
        ];
        assert_eq!(
            ventanas_pausa(&t),
            vec![(h(10, 20), h(12, 0)), (h(10, 25), h(12, 0))]
        );
        let e = Eventos {
            transiciones: t,
            con_filas: true,
            ..Eventos::default()
        };
        assert_eq!(real(&e).map(|r| r.cierre), Ok(10.0));
    }

    /// Las filas de Coste y de Transiciones del cuerpo, con las ilegibles
    /// contadas.
    #[test]
    fn eventos_del_cuerpo() {
        let c = Cuerpo::leer(
            "## Transiciones\n\n| fecha | fase | modo |\n|---|---|---|\n| 2026-11-17T10:00:00Z | start | m |\n| ayer | review | m |\n\n## Coste\n\n| fecha | fase | total |\n|---|---|---|\n| 2026-11-17T10:30:00Z | implementacion | 5 |\n| 2026-11-17T10:40:00Z | x | mucho |\n| 2026-11-17T10:50:00Z | x | |\n",
        );
        let e = eventos(&c, vec![7]);
        assert_eq!(e.transiciones, vec![paso(h(10, 0), "start")]);
        assert_eq!(e.coste, vec![h(10, 30), h(10, 50)]);
        assert_eq!(e.commits, vec![7]);
        assert_eq!(e.filas_malas, 2);
        assert!(e.con_filas);
    }
}
