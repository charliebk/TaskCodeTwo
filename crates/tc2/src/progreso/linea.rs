//! La linea de progreso (puro): lo que pintan la status line y
//! `tc2 status --seguir` (L5 de F1-17).
//!
//! ```text
//! TASK-075 · implementacion · fases que faltan: rondas, cierre · faltan ≈50 min (p80 96) · n=17 carril+complejidad+tamano
//! TASK-075 · implementacion · fases que faltan: rondas, cierre · faltan ≥31 min (fase en curso por encima de sus parecidas) · n=17 carril · sin actividad registrada hace 127 min
//! TASK-075 · pausada en implementacion · fases que faltan: rondas, cierre · faltan: sin estimacion (n=3)
//! tc2 · sin tarea en curso
//! ```
//!
//! Lo transcurrido es el del calculo mas el reloj desde el ultimo evento
//! (o desde la ultima actividad que se ve sin lanzar git: el `logs/HEAD` del
//! worktree), cada hueco con el tope de 45 min de F1-17. Pausada, el reloj
//! no corre. Todo lo que viene del fichero se sanea (F1-17, punto 8).

use super::fichero::{PASO_MIN, Progreso, TOPE_MIN};
use crate::estimacion::{UMBRAL_MIN, minutos};
use crate::tablero::sanear;

/// La linea cuando no hay tarea en curso (o fuera de un repo).
pub const SIN_TAREA: &str = "tc2 · sin tarea en curso";

/// Separador de las partes de la linea.
const SEP: &str = " · ";

/// Longitud maxima del nivel en la linea.
const MAX_NIVEL: usize = 40;

/// Longitud maxima de cada aviso en la linea.
pub const MAX_AVISO_LINEA: usize = 80;

/// Avisos como mucho en la linea.
const AVISOS_EN_LINEA: usize = 2;

/// Minutos (con decimales) de `desde` a `hasta`; 0 si `hasta` no es
/// posterior.
fn min_entre(desde: u64, hasta: u64) -> f64 {
    hasta.saturating_sub(desde) as f64 / 60.0
}

/// Lo transcurrido en la fase a las `ahora` y los minutos sin actividad
/// (el hueco abierto, sin tope).
pub fn transcurrido(p: &Progreso, ahora: u64, actividad: Option<u64>) -> (f64, f64) {
    if p.pausada {
        return (p.transcurrido, 0.0);
    }
    // Una actividad posterior al ultimo evento (un commit despues del
    // calculo) parte el hueco en dos; una del futuro no cuenta.
    let act = actividad
        .filter(|a| *a <= ahora)
        .unwrap_or(0)
        .max(p.ultimo_evento);
    let antes = min_entre(p.ultimo_evento, act).min(UMBRAL_MIN);
    let abierto = min_entre(act, ahora);
    (p.transcurrido + antes + abierto.min(UMBRAL_MIN), abierto)
}

/// Aviso de una tabla que ya no llega a lo transcurrido (revision 1 de
/// TASK-084, R1-05).
pub const SATURADA: &str = "estimacion saturada: mas de 480 min en la fase";

/// La parte de lo que falta. Pasado el ultimo punto de la tabla
/// ([`TOPE_MIN`]), el dato es el de ese punto y la linea lo avisa: la tabla
/// no se alarga porque una fase de mas de 8 h activas ya no se parece a sus
/// parecidas y el fichero creceria sin aportar nada.
fn lo_que_falta(p: &Progreso, t: f64) -> String {
    let i = ((t / PASO_MIN as f64).floor().max(0.0) as usize).min(p.tabla.len().saturating_sub(1));
    let nivel = sanear(&p.nivel, MAX_NIVEL);
    let saturada = if t > TOPE_MIN as f64 {
        format!("{SEP}{SATURADA}")
    } else {
        String::new()
    };
    let parte = match p.tabla.get(i) {
        None => return format!("faltan: sin estimacion (n={})", p.n),
        Some(x) if x.desbordada => format!(
            "faltan ≥{} min (fase en curso por encima de sus parecidas){SEP}n={} {nivel}",
            minutos(x.mediana),
            p.n
        ),
        Some(x) => format!(
            "faltan ≈{} min (p80 {}){SEP}n={} {nivel}",
            minutos(x.mediana),
            minutos(x.p80),
            p.n
        ),
    };
    format!("{parte}{saturada}")
}

/// `linea(progreso, ahora, ultima_actividad)`: la linea de la tarea.
pub fn linea(p: &Progreso, ahora: u64, actividad: Option<u64>) -> String {
    let (t, abierto) = transcurrido(p, ahora, actividad);
    let id = sanear(&p.id, 16);
    let fase = sanear(&p.fase, 20);
    let mut partes = vec![id];
    partes.push(if p.pausada {
        format!("pausada en {fase}")
    } else {
        fase
    });
    if !p.faltan.is_empty() {
        let faltan: Vec<String> = p.faltan.iter().map(|f| sanear(f, 20)).collect();
        partes.push(format!("fases que faltan: {}", faltan.join(", ")));
    }
    partes.push(lo_que_falta(p, t));
    if abierto > UMBRAL_MIN {
        partes.push(format!(
            "sin actividad registrada hace {} min",
            abierto.floor() as u64
        ));
    }
    for a in p.avisos.iter().take(AVISOS_EN_LINEA) {
        let a = sanear(a, MAX_AVISO_LINEA);
        if !a.is_empty() {
            partes.push(a);
        }
    }
    partes.join(SEP)
}

/// La linea de un dato que no es el ultimo (el `.anterior` de un fichero
/// ilegible): con su antiguedad.
pub fn con_antiguedad(l: &str, p: &Progreso, ahora: u64) -> String {
    format!(
        "{l}{SEP}(dato de hace {} min)",
        min_entre(p.calculada, ahora).floor() as u64
    )
}

/// La linea de una tarea cuyo progreso no se lee.
pub fn ilegible(id: &str) -> String {
    format!("{}{SEP}progreso ilegible", sanear(id, 16))
}

/// La linea del principal con varias en curso: la mas reciente y `+N en
/// curso`.
pub fn con_mas(l: &str, otras: usize) -> String {
    if otras == 0 {
        l.to_owned()
    } else {
        format!("{l}{SEP}+{otras} en curso")
    }
}

#[cfg(test)]
mod tests {
    use super::super::calculo::tests::{T0, historico, tarea};
    use super::super::calculo::{EN_CURSO, calcular};
    use super::super::fichero::tests::ejemplo;
    use super::*;
    use crate::estimacion::{Commits, Historico};

    /// C2: muestras conocidas y 20 min transcurridos dan exactamente la
    /// linea (fase, fases que faltan, ≈mediana, p80, n y nivel).
    #[test]
    fn c2_linea_exacta_a_los_20_min() {
        let (t, c) = tarea(EN_CURSO, &[(0, "start")]);
        let p = calcular(&t, &c, &historico(), &Commits::vacios(), T0);
        let p = p.unwrap_or_else(|| unreachable!());
        assert_eq!(
            linea(&p, T0 + 20 * 60, None),
            "TASK-007 · implementacion · fases que faltan: rondas, cierre · faltan ≈50 min (p80 65) · n=6 repo"
        );
        // Recien empezada: todo lo de las parecidas (imp + 15).
        assert_eq!(
            linea(&p, T0, None),
            "TASK-007 · implementacion · fases que faltan: rondas, cierre · faltan ≈70 min (p80 85) · n=6 repo"
        );
    }

    /// C5: sin parecidas, `sin estimacion (n=N)`.
    #[test]
    fn c5_sin_parecidas() {
        let (t, c) = tarea(EN_CURSO, &[(0, "start")]);
        let h = Historico {
            muestras: historico().muestras[..3].to_vec(),
            ..Historico::default()
        };
        let p = calcular(&t, &c, &h, &Commits::vacios(), T0);
        let p = p.unwrap_or_else(|| unreachable!());
        assert_eq!(
            linea(&p, T0 + 60, None),
            "TASK-007 · implementacion · fases que faltan: rondas, cierre · faltan: sin estimacion (n=3)"
        );
    }

    /// C6: pasado lo de las parecidas, la cota `≥X` con el aviso; mas de 45
    /// min sin eventos, «sin actividad registrada» y el reloj con el tope.
    #[test]
    fn c6_desbordada_y_sin_actividad() {
        let (t, c) = tarea(
            EN_CURSO,
            &[(0, "start"), (40 * 60, "pausa"), (41 * 60, "reanudar")],
        );
        let p = calcular(&t, &c, &historico(), &Commits::vacios(), T0 + 41 * 60);
        let p = p.unwrap_or_else(|| unreachable!());
        assert_eq!(p.transcurrido, 40.0);
        // 40 + 30 = 70: solo una parecida (80) pasa de 70: desbordada; la
        // cota es lo de las fases siguientes (10 + 5).
        assert_eq!(
            linea(&p, T0 + 71 * 60, None),
            "TASK-007 · implementacion · fases que faltan: rondas, cierre · faltan ≥15 min (fase en curso por encima de sus parecidas) · n=6 repo"
        );
        // 127 min sin nada: el reloj suma 45 y lo dice.
        let l = linea(&p, T0 + (41 + 127) * 60, None);
        assert!(
            l.ends_with("(fase en curso por encima de sus parecidas) · n=6 repo · sin actividad registrada hace 127 min"),
            "{l}"
        );
        assert_eq!(transcurrido(&p, T0 + (41 + 127) * 60, None).0, 85.0);
        // Una actividad de `logs/HEAD` a los 100 min parte el hueco: 45 + 27.
        let (t2, abierto) = transcurrido(&p, T0 + (41 + 127) * 60, Some(T0 + (41 + 100) * 60));
        assert_eq!((t2, abierto), (40.0 + 45.0 + 27.0, 27.0));
        // Una actividad del futuro no cuenta.
        assert_eq!(
            transcurrido(&p, T0 + 50 * 60, Some(T0 + 99_999)),
            transcurrido(&p, T0 + 50 * 60, None)
        );
    }

    /// Pausada: el reloj no corre y la linea lo dice.
    #[test]
    fn pausada_congela() {
        let (t, c) = tarea(EN_CURSO, &[(0, "start"), (20 * 60, "pausa")]);
        let p = calcular(&t, &c, &historico(), &Commits::vacios(), T0 + 20 * 60);
        let p = p.unwrap_or_else(|| unreachable!());
        let a = linea(&p, T0 + 20 * 60, None);
        assert_eq!(a, linea(&p, T0 + 500 * 60, None));
        assert_eq!(
            a,
            "TASK-007 · pausada en implementacion · fases que faltan: rondas, cierre · faltan ≈50 min (p80 65) · n=6 repo"
        );
    }

    /// C10: avisos con control, ANSI o bidi salen saneados y acotados; el
    /// nivel tambien.
    #[test]
    fn c10_avisos_saneados_y_acotados() {
        let mut p = ejemplo("TASK-001");
        p.avisos = vec![
            format!(
                "rojo \u{1b}[31mpeligro\u{1b}[0m \u{202e}al reves\u{2066} {}",
                "x".repeat(500)
            ),
            "linea\nnueva\r\ttab".to_owned(),
            "tercero no sale".to_owned(),
        ];
        p.nivel = format!("repo\u{200b}\u{7}{}", "n".repeat(100));
        let l = linea(&p, 1_000, None);
        assert!(
            !l.chars().any(|c| c.is_control()
                || matches!(c, '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{FEFF}')),
            "{l:?}"
        );
        assert!(!l.contains('\u{1b}') && !l.contains("tercero"), "{l}");
        assert!(l.contains("linea nueva tab"), "{l}");
        assert!(l.chars().count() <= 400, "{}", l.chars().count());
        let partes: Vec<&str> = l.split(SEP).collect();
        assert!(
            partes
                .iter()
                .all(|x| x.chars().count() <= MAX_AVISO_LINEA.max(MAX_NIVEL + 10)),
            "{l}"
        );
    }

    /// Revision 1 de TASK-084 (M1): a mitad de tramo (23 min) vale el punto
    /// de los 20, no el de los 25 (`floor`, no `round`).
    #[test]
    fn m1_a_mitad_de_tramo_vale_el_punto_anterior() {
        let (t, c) = tarea(EN_CURSO, &[(0, "start")]);
        let p = calcular(&t, &c, &historico(), &Commits::vacios(), T0);
        let p = p.unwrap_or_else(|| unreachable!());
        let a_los_20 = linea(&p, T0 + 20 * 60, None);
        let a_los_25 = linea(&p, T0 + 25 * 60, None);
        assert_ne!(a_los_20, a_los_25);
        assert_eq!(linea(&p, T0 + 23 * 60, None), a_los_20);
        assert_eq!(linea(&p, T0 + 24 * 60 + 59, None), a_los_20);
    }

    /// Revision 1 de TASK-084 (M2): la frontera de «sin actividad
    /// registrada»: a los 45 min no, a los 46 si.
    #[test]
    fn m2_frontera_de_sin_actividad() {
        let p = ejemplo("TASK-001");
        let a_los_45 = linea(&p, p.ultimo_evento + 45 * 60, None);
        assert!(!a_los_45.contains("sin actividad"), "{a_los_45}");
        let a_los_46 = linea(&p, p.ultimo_evento + 46 * 60, None);
        assert!(
            a_los_46.ends_with(" · sin actividad registrada hace 46 min"),
            "{a_los_46}"
        );
    }

    /// Revision 1 de TASK-084 (R1-05): pasado el ultimo punto de la tabla,
    /// la linea avisa de que el dato esta saturado; en el punto, no.
    #[test]
    fn r1_05_pasado_el_tope_avisa_saturada() {
        assert!(SATURADA.contains(&format!("{TOPE_MIN} min")));
        let mut p = ejemplo("TASK-001");
        p.transcurrido = TOPE_MIN as f64;
        let en_el_tope = linea(&p, p.ultimo_evento, None);
        assert!(!en_el_tope.contains("saturada"), "{en_el_tope}");
        p.transcurrido = TOPE_MIN as f64 + 1.0;
        let pasado = linea(&p, p.ultimo_evento, None);
        assert!(
            pasado.ends_with(&format!("faltan ≈50 min (p80 65) · n=6 repo · {SATURADA}")),
            "{pasado}"
        );
        // Sin tabla no hay dato que saturar.
        p.tabla.clear();
        assert!(!linea(&p, p.ultimo_evento, None).contains("saturada"));
    }

    #[test]
    fn otras_formas() {
        let p = ejemplo("TASK-001");
        assert_eq!(
            con_antiguedad("L", &p, p.calculada + 7 * 60 + 59),
            "L · (dato de hace 7 min)"
        );
        assert_eq!(ilegible("TASK-001"), "TASK-001 · progreso ilegible");
        assert_eq!(con_mas("L", 0), "L");
        assert_eq!(con_mas("L", 2), "L · +2 en curso");
        assert_eq!(SIN_TAREA, "tc2 · sin tarea en curso");
    }
}
