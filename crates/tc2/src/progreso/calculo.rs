//! El calculo del progreso de una tarea abierta (puro: ni git ni ficheros).
//!
//! Sin segunda formula: reutiliza [`crate::estimacion`] (`parecidas`,
//! `restante`, `eventos`, `ventanas_pausa` y `tramo`, de TASK-083).
//!
//! - `en-curso` es la fase de implementacion; `en-revision`, la de rondas
//!   (con solo el cierre por delante).
//! - Lo transcurrido en la fase es el real de F1-17 desde su inicio (`start`
//!   o la primera `review`) hasta el ultimo evento: huecos de mas de 45 min
//!   contados como 45 y sin las ventanas de pausa. Lo que pase despues lo
//!   suma la vista con el reloj ([`super::linea`]).
//! - La tabla guarda lo que falta para cada transcurrido de 0 a 480 min cada
//!   5: la vista solo busca, no recalcula.

use super::fichero::{MAX_AVISOS, PASO_MIN, PUNTOS, Progreso, Punto, VERSION};
use crate::estimacion::real::{tramo, ventanas_pausa};
use crate::estimacion::{self, Commits, Fase, Historico, eventos, parecidas, restante};
use crate::tarea::{Cuerpo, Tarea};

/// Estado de una tarea en la fase de implementacion.
pub const EN_CURSO: &str = "en-curso";

/// Estado de una tarea en la fase de rondas.
pub const EN_REVISION: &str = "en-revision";

/// Longitud maxima de un aviso.
pub const MAX_AVISO: usize = 120;

/// Aviso cuando el `git log` de `develop` no se pudo leer.
pub const AVISO_SIN_LOG: &str = "historia sin git log: lo transcurrido no cuenta los commits";

/// La fase en curso de una tarea en `estado`, su nombre y las que faltan.
pub fn fase_de(estado: &str) -> Option<(Fase, &'static str, &'static [&'static str])> {
    match estado {
        EN_CURSO => Some((
            Fase::Implementacion,
            "implementacion",
            &["rondas", "cierre"],
        )),
        EN_REVISION => Some((Fase::Rondas, "rondas", &["cierre"])),
        _ => None,
    }
}

/// `calcular(tarea, cuerpo, historico, commits, ahora)`: el progreso de `t`
/// si esta en curso o en revision; si no, `None`. `ahora` en segundos desde
/// 1970.
pub fn calcular(
    t: &Tarea,
    c: &Cuerpo,
    h: &Historico,
    commits: &Commits,
    ahora: u64,
) -> Option<Progreso> {
    let (fase, nombre, faltan) = fase_de(&t.estado)?;
    let ahora_i = i64::try_from(ahora).unwrap_or(i64::MAX);
    let ev = eventos(c, commits.de_tarea(&t.id));
    let marca = if fase == Fase::Rondas {
        "review"
    } else {
        "start"
    };
    let inicio = ev
        .transiciones
        .iter()
        .filter(|p| p.fase == marca && p.ts <= ahora_i)
        .map(|p| p.ts)
        .min()
        .unwrap_or(ahora_i);
    let mut todos: Vec<i64> = ev.commits.clone();
    todos.extend(&ev.coste);
    todos.extend(ev.transiciones.iter().map(|p| p.ts));
    todos.retain(|x| *x <= ahora_i);
    todos.sort_unstable();
    todos.dedup();
    let ultimo = todos.last().copied().unwrap_or(inicio).max(inicio);
    let pausas = ventanas_pausa(&ev.transiciones);
    let transcurrido = tramo(inicio, ultimo, &todos, &pausas);
    let obj = estimacion::objetivo(t, c);
    let est = estimacion::estimar(&obj, &h.muestras, "");
    let tabla = match parecidas(&obj, &h.muestras) {
        Some((_, ms)) => (0..PUNTOS as u64)
            .filter_map(|i| restante(&ms, fase, (i * PASO_MIN) as f64))
            .map(|r| Punto {
                mediana: r.mediana,
                p80: r.p80,
                desbordada: r.desbordada,
            })
            .collect(),
        None => Vec::new(),
    };
    let mut avisos = Vec::new();
    if !commits.leido() {
        avisos.push(AVISO_SIN_LOG.to_owned());
    }
    avisos.truncate(MAX_AVISOS);
    Some(Progreso {
        version: VERSION,
        id: t.id.clone(),
        fase: nombre.to_owned(),
        pausada: c.en_pausa(),
        faltan: faltan.iter().map(|f| (*f).to_owned()).collect(),
        transcurrido,
        ultimo_evento: u64::try_from(ultimo).unwrap_or_default(),
        calculada: ahora,
        nivel: est.nivel,
        n: est.n,
        tabla,
        avisos: avisos
            .iter()
            .map(|a| crate::tablero::sanear(a, MAX_AVISO))
            .collect(),
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::estimacion::{Muestra, Real};

    /// 2026-11-17T10:00:00Z.
    pub(crate) const T0: u64 = 1_794_909_600;

    /// Seis muestras de repo: implementacion 30..80, rondas 10 y cierre 5.
    pub(crate) fn historico() -> Historico {
        let muestras = [30.0, 40.0, 50.0, 60.0, 70.0, 80.0]
            .iter()
            .enumerate()
            .map(|(i, imp)| Muestra {
                id: format!("TASK-{:03}", i + 1),
                miembros: vec![format!("TASK-{:03}", i + 1)],
                carril: None,
                complejidad: None,
                criterios: 1,
                real: Real {
                    implementacion: *imp,
                    rondas: 10.0,
                    cierre: 5.0,
                },
                start: 0,
                finish: 0,
            })
            .collect();
        Historico {
            muestras,
            ..Historico::default()
        }
    }

    /// Una tarea `TASK-007` en `estado` con las filas de `filas` ((segundos
    /// desde [`T0`], fase)).
    pub(crate) fn tarea(estado: &str, filas: &[(u64, &str)]) -> (Tarea, Cuerpo) {
        let mut t = Tarea::nueva("TASK-007", "Abierta", "2026-11-17");
        estado.clone_into(&mut t.estado);
        let mut texto = "## Objetivo\n\nX.\n\n## Criterios de aceptacion\n\n- [ ] a\n\n## Transiciones\n\n| fecha | fase | modo | decidido_por | version |\n|---|---|---|---|---|\n".to_owned();
        for (s, f) in filas {
            texto.push_str(&format!(
                "| {} | {f} | manual | persona | 0.1 |\n",
                crate::fecha::instante(T0 + s)
            ));
        }
        (t, Cuerpo::leer(&texto))
    }

    /// C2 (calculo): `en-curso` es implementacion con rondas y cierre por
    /// delante; la tabla coincide con `restante` en cada punto.
    #[test]
    fn en_curso_es_implementacion_y_la_tabla_es_restante() {
        let h = historico();
        let (t, c) = tarea(EN_CURSO, &[(0, "start")]);
        let p = calcular(&t, &c, &h, &Commits::vacios(), T0 + 1200);
        let p = p.unwrap_or_else(|| unreachable!());
        assert_eq!(p.fase, "implementacion");
        assert_eq!(p.faltan, ["rondas", "cierre"]);
        assert_eq!((p.nivel.as_str(), p.n), ("repo", 6));
        assert_eq!((p.transcurrido, p.ultimo_evento), (0.0, T0));
        assert_eq!(p.calculada, T0 + 1200);
        assert_eq!(p.tabla.len(), PUNTOS);
        let ms: Vec<&Muestra> = h.muestras.iter().collect();
        for (i, punto) in p.tabla.iter().enumerate() {
            let r = restante(&ms, Fase::Implementacion, (i as u64 * PASO_MIN) as f64);
            assert_eq!(
                r.map(|r| (r.mediana, r.p80, r.desbordada)),
                Some((punto.mediana, punto.p80, punto.desbordada)),
                "{i}"
            );
        }
        // A los 20 minutos: ≈50 (p80 65), sin desbordar.
        assert_eq!(
            p.tabla[4],
            Punto {
                mediana: 50.0,
                p80: 65.0,
                desbordada: false
            }
        );
        assert!(!p.pausada && p.avisos.is_empty());
    }

    /// C4: `en-revision` es la fase de rondas con solo el cierre por delante,
    /// y lo transcurrido cuenta desde la primera `review`.
    #[test]
    fn c4_en_revision_es_rondas_con_solo_cierre() {
        let (t, c) = tarea(
            EN_REVISION,
            &[(0, "start"), (1800, "review"), (2400, "review")],
        );
        let p = calcular(&t, &c, &historico(), &Commits::vacios(), T0 + 3000);
        let p = p.unwrap_or_else(|| unreachable!());
        assert_eq!(p.fase, "rondas");
        assert_eq!(p.faltan, ["cierre"]);
        assert_eq!(p.transcurrido, 10.0);
        assert_eq!(p.ultimo_evento, T0 + 2400);
        // En rondas lo que falta son rondas y cierre (15 a los 0 min).
        assert_eq!(p.tabla[0].mediana, 15.0);
        // Otros estados no tienen progreso.
        for e in ["planificada", "terminada", "descartada", "en-diseno"] {
            let (t, c) = tarea(e, &[(0, "start")]);
            assert!(
                calcular(&t, &c, &historico(), &Commits::vacios(), T0).is_none(),
                "{e}"
            );
        }
    }

    /// La pausa congela: la ventana no cuenta, y con la ultima fila `pausa`
    /// la tarea sale pausada. Los huecos de mas de 45 min cuentan 45.
    #[test]
    fn pausa_y_tope_de_45() {
        let (t, c) = tarea(
            EN_CURSO,
            &[(0, "start"), (600, "pausa"), (4200, "reanudar")],
        );
        let p = calcular(&t, &c, &historico(), &Commits::vacios(), T0 + 4800);
        let p = p.unwrap_or_else(|| unreachable!());
        assert_eq!((p.transcurrido, p.pausada), (10.0, false));
        let (t, c) = tarea(EN_CURSO, &[(0, "start"), (600, "pausa")]);
        let p = calcular(&t, &c, &historico(), &Commits::vacios(), T0 + 9000);
        assert_eq!(p.map(|p| (p.transcurrido, p.pausada)), Some((10.0, true)));
        // Un hueco de 2 h entre start y un commit cuenta 45.
        let (t, c) = tarea(EN_CURSO, &[(0, "start")]);
        let log = format!("a\u{1f}\u{1f}{}\u{1f}TASK-007 algo\n", T0 + 7200);
        let p = calcular(&t, &c, &historico(), &Commits::de_log(&log), T0 + 7300);
        assert_eq!(p.map(|p| p.transcurrido), Some(45.0));
    }

    /// Sin parecidas, la tabla va vacia con `sin-datos`; sin `git log`, el
    /// aviso.
    #[test]
    fn sin_parecidas_y_sin_log() {
        let (t, c) = tarea(EN_CURSO, &[(0, "start")]);
        let p = calcular(&t, &c, &Historico::default(), &Commits::sin_log(), T0);
        let p = p.unwrap_or_else(|| unreachable!());
        assert!(p.tabla.is_empty());
        assert_eq!((p.nivel.as_str(), p.n), ("sin-datos", 0));
        assert_eq!(p.avisos, [AVISO_SIN_LOG]);
    }
}
