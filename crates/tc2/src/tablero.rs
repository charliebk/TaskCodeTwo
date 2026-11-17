//! `tc2 board` y `tc2 status` (F3-5): el estado del equipo, quien lleva que
//! y que espera revision.
//!
//! Las dos ordenes solo leen. Salen de la misma [`situacion::calcular`] sobre
//! [`estado::leer`] que `next` y `roadmap` (misma foto y mismo `commit=`), sin
//! `fetch` (la frescura es de F3-5b) y sin escribir nada.
//!
//! - `status [--json]`: una linea de equipo, sin red. Con `--json`, `status
//!   ok en-diseno=... en-curso=... en-revision=... listas=... commit=...`.
//! - `board [--sin-plataforma] [--json]`: la vista completa por grupos (en
//!   diseno, en curso, en revision y listas para empezar), con ID, titulo,
//!   quien y PR; terminadas y descartadas solo como recuento.
//!
//! El texto ajeno (titulos, logins, asignados heredados) se sanea al
//! imprimirlo (F1-17, decision 8): sin caracteres de control ni de formato y
//! con longitud acotada.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use crate::Salida;
use crate::contrato::{Informe, Resultado, corto};
use crate::estado::{self, Fallo};
use crate::git::Git;
use crate::grafo::{Fase, TERMINADA};
use crate::i18n::catalogo::cli as txt_cli;
use crate::i18n::catalogo::roadmap as txt_roadmap;
use crate::i18n::catalogo::tablero as txt;
use crate::i18n::{Idioma, Mensaje};
use crate::plataforma::Personas;
use crate::situacion::{self, Ficha, Quien, Situacion};

/// Nombre de `board`.
pub const BOARD: &str = "board";
/// Nombre de `status`.
pub const STATUS: &str = "status";

/// Tareas por grupo en la linea de `status` (las demas, `+N`).
pub const K_STATUS: usize = 3;
/// Caracteres como mucho de un titulo impreso.
const MAX_TITULO: usize = 60;
/// Caracteres como mucho de un «quien» impreso.
const MAX_QUIEN: usize = 40;

/// Una tarea de un grupo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fila {
    /// ID.
    pub id: String,
    /// Titulo, tal cual (se sanea al imprimir).
    pub titulo: String,
    /// Quien la lleva.
    pub quien: Quien,
    /// Su rama, si la tiene.
    pub rama: Option<String>,
}

/// Lo que se ve del equipo sobre un estado.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tablero {
    /// Commit de la foto del estado (12 caracteres).
    pub commit: String,
    /// En diseno, en el orden de trabajo.
    pub en_diseno: Vec<Fila>,
    /// En curso, en el orden de trabajo.
    pub en_curso: Vec<Fila>,
    /// En revision, en el orden de trabajo.
    pub en_revision: Vec<Fila>,
    /// Listas para empezar, en el orden de `next`.
    pub listas: Vec<Fila>,
    /// Cuantas terminadas.
    pub terminadas: usize,
    /// Cuantas descartadas.
    pub descartadas: usize,
    /// Carpetas con un `tarea.md` que no se entiende.
    pub ilegibles: Vec<String>,
}

impl Tablero {
    /// Los cuatro grupos con su clave de contrato y su titulo.
    pub fn grupos(&self) -> [(&'static str, &'static str, &[Fila]); 4] {
        [
            ("en-diseno", "En diseño", &self.en_diseno),
            ("en-curso", "En curso", &self.en_curso),
            ("en-revision", "En revisión", &self.en_revision),
            ("listas", "Listas para empezar", &self.listas),
        ]
    }

    /// Todas las filas, grupo tras grupo.
    pub fn filas(&self) -> impl Iterator<Item = &Fila> {
        self.en_diseno
            .iter()
            .chain(&self.en_curso)
            .chain(&self.en_revision)
            .chain(&self.listas)
    }
}

/// El tablero de `s` (funcion pura): las listas de `next` repartidas por
/// estado, con el «quien» de `roadmap` ([`situacion::quien`]).
pub fn tablero(s: &Situacion, personas: &Personas, commit: &str) -> Tablero {
    // Si un ID se repite, vale el primero (como en `roadmap`).
    let mut fichas: BTreeMap<&str, &Ficha> = BTreeMap::new();
    for f in &s.fichas {
        fichas.entry(f.nodo.id.as_str()).or_insert(f);
    }
    let fila = |id: &String| -> Option<Fila> {
        let f = fichas.get(id.as_str())?;
        Some(Fila {
            id: id.clone(),
            titulo: f.titulo.clone(),
            quien: situacion::quien(f.asignado.as_ref(), personas),
            rama: f.rama.clone(),
        })
    };
    let mut t = Tablero {
        commit: commit.to_owned(),
        ilegibles: s.ilegibles.clone(),
        ..Tablero::default()
    };
    for id in &s.en_marcha {
        let estado = fichas.get(id.as_str()).and_then(|f| f.nodo.estado());
        let grupo = match estado {
            Some("en-diseno") => &mut t.en_diseno,
            Some("en-curso") => &mut t.en_curso,
            Some("en-revision") => &mut t.en_revision,
            _ => continue,
        };
        grupo.extend(fila(id));
    }
    t.listas = s.listas.iter().filter_map(fila).collect();
    for f in fichas.values() {
        match &f.nodo.fase {
            Fase::Viva(e) if e == TERMINADA => t.terminadas += 1,
            Fase::Descartada => t.descartadas += 1,
            _ => {}
        }
    }
    t
}

/// Texto ajeno listo para imprimir (F1-17, decision 8): sin caracteres de
/// control ni de formato (bidi, anchura cero), espacios juntados y como
/// mucho `max` caracteres (con `…` si se corta).
pub fn sanear(t: &str, max: usize) -> String {
    let formato = |c: char| matches!(c, '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{FEFF}');
    let limpio: String = t
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .filter(|c| !formato(*c))
        .collect();
    let limpio = limpio.split_whitespace().collect::<Vec<_>>().join(" ");
    if limpio.chars().count() <= max {
        return limpio;
    }
    let mut corto: String = limpio.chars().take(max.saturating_sub(1)).collect();
    corto.push('…');
    corto
}

/// El «quien» impreso de una fila.
fn quien_impreso(q: &Quien) -> String {
    sanear(&q.texto(), MAX_QUIEN)
}

/// IDs de unas filas.
fn ids(v: &[Fila]) -> Vec<String> {
    v.iter().map(|f| f.id.clone()).collect()
}

/// La linea de `status` sin `--json`: los grupos con tareas, `K_STATUS`
/// por grupo como mucho y `+N` con las demas, y el commit.
pub fn linea_status(t: &Tablero) -> String {
    let mut partes = Vec::new();
    for (_, titulo, filas) in t.grupos() {
        if filas.is_empty() {
            continue;
        }
        let mut v: Vec<String> = filas
            .iter()
            .take(K_STATUS)
            .map(|f| match f.quien {
                Quien::Nadie => f.id.clone(),
                ref q => format!("{} {}", f.id, quien_impreso(q)),
            })
            .collect();
        if filas.len() > K_STATUS {
            v.push(format!("+{}", filas.len() - K_STATUS));
        }
        partes.push(format!("{}: {}", titulo.to_lowercase(), v.join(", ")));
    }
    if partes.is_empty() {
        partes.push("nada en marcha ni listo para empezar".to_owned());
    }
    partes.push(format!("commit {}", t.commit));
    partes.join(" · ")
}

/// El texto de `board`: un apartado por grupo con ID, titulo, quien y PR, y
/// el recuento de terminadas y descartadas. `pr` da la celda del PR de cada
/// fila (`None`: sin columna).
pub fn texto_board(t: &Tablero, plataforma: &str, pr: Option<&dyn Fn(&Fila) -> String>) -> String {
    let mut o = format!("Tablero · commit {} · plataforma {plataforma}\n", t.commit);
    for (_, titulo, filas) in t.grupos() {
        let _ = writeln!(o, "\n{titulo} ({})", filas.len());
        if filas.is_empty() {
            o.push_str("  ninguna\n");
            continue;
        }
        let celdas: Vec<[String; 4]> = filas
            .iter()
            .map(|f| {
                [
                    f.id.clone(),
                    sanear(&f.titulo, MAX_TITULO),
                    quien_impreso(&f.quien),
                    pr.map(|p| p(f)).unwrap_or_default(),
                ]
            })
            .collect();
        let ancho = |i: usize| {
            celdas
                .iter()
                .map(|c| c[i].chars().count())
                .max()
                .unwrap_or(0)
        };
        let (a0, a1, a2) = (ancho(0), ancho(1), ancho(2));
        for c in &celdas {
            let mut l = format!(
                "  {}  {}  {}",
                rellenar(&c[0], a0),
                rellenar(&c[1], a1),
                rellenar(&c[2], a2)
            );
            if pr.is_some() {
                l.push_str("  ");
                l.push_str(&c[3]);
            }
            o.push_str(l.trim_end());
            o.push('\n');
        }
    }
    let _ = writeln!(
        o,
        "\nTerminadas: {} · Descartadas: {}",
        t.terminadas, t.descartadas
    );
    if !t.ilegibles.is_empty() {
        let _ = writeln!(o, "Ilegibles: {}", t.ilegibles.join(", "));
    }
    o
}

fn rellenar(t: &str, ancho: usize) -> String {
    let n = t.chars().count();
    format!("{t}{}", " ".repeat(ancho.saturating_sub(n)))
}

/// Aviso: la lista de personas existe y no se pudo leer.
const PERSONAS_ILEGIBLES: &str = "personas-ilegibles";

/// El tablero del estado que conoce el clon de `dir`, con los avisos de
/// personas. Sin rama de estado, `rechazada motivo=sin-rama-de-estado`.
fn tablero_en(g: &Git, dir: &Path, op: &str) -> Result<(Tablero, Vec<&'static str>), Informe> {
    let vista = estado::leer(g, dir).map_err(|mut i| {
        op.clone_into(&mut i.comando);
        i
    })?;
    let commit = vista.commit().map(corto).unwrap_or_default();
    let s = situacion::calcular(&vista).map_err(|f| match f {
        Fallo::Rechazada {
            motivo,
            diagnostico,
        } => Informe::nuevo(op, Resultado::Rechazada)
            .con("motivo", motivo)
            .diagnostico(diagnostico),
        Fallo::Error(d) => Informe::nuevo(op, Resultado::Error).diagnostico(d),
    })?;
    let (personas, ilegibles) = crate::roadmap::personas_y_aviso(g, dir);
    let avisos = if ilegibles {
        vec![PERSONAS_ILEGIBLES]
    } else {
        Vec::new()
    };
    Ok((tablero(&s, &personas, &commit), avisos))
}

/// Lo que necesitan las dos ordenes del entorno.
pub struct Entorno<'a> {
    /// Carpeta desde la que se lanza.
    pub dir: &'a Path,
    /// Puerto Git.
    pub git: &'a Git,
}

/// `status [--json]` (sin `--json`, que quita `lib.rs`). No consulta nunca
/// la plataforma.
pub fn orden_status(args: &[String], json: bool, idioma: Idioma, e: &Entorno<'_>) -> Salida {
    if !args.is_empty() {
        return Informe::nuevo(STATUS, Resultado::Uso)
            .con("motivo", "argumentos-de-mas")
            .diagnostico(&txt::USO_STATUS)
            .salida_en(json, idioma);
    }
    match status_en(e) {
        Ok((linea, _)) if !json => Salida {
            codigo: 0,
            stdout: format!("{linea}\n"),
            stderr: String::new(),
        },
        Ok((_, i)) | Err(i) => i.salida_en(json, idioma),
    }
}

/// `status` sobre el estado que conoce el clon: la linea de texto y el
/// informe.
pub fn status_en(e: &Entorno<'_>) -> Result<(String, Informe), Informe> {
    let (t, _) = tablero_en(e.git, e.dir, STATUS)?;
    let mut i = Informe::nuevo(STATUS, Resultado::Ok);
    for (clave, _, filas) in t.grupos() {
        i = i.con_lista(clave, ids(filas));
    }
    i = i.con("commit", t.commit.clone());
    if !t.ilegibles.is_empty() {
        i = i.con_lista("ilegibles", t.ilegibles.clone());
    }
    Ok((linea_status(&t), i))
}

/// Que hace `board` con la plataforma.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modo {
    /// La consulta (por defecto).
    ConPlataforma,
    /// `--sin-plataforma`: no la consulta.
    SinPlataforma,
}

/// `board [--sin-plataforma] [--json]` (sin `--json`, que quita `lib.rs`).
/// El texto va a stdout y los avisos a stderr con codigo 0.
pub fn orden_board(args: &[String], json: bool, idioma: Idioma, e: &Entorno<'_>) -> Salida {
    let modo = match args {
        [] => Modo::ConPlataforma,
        [a] if a == "--sin-plataforma" => Modo::SinPlataforma,
        _ => {
            return Informe::nuevo(BOARD, Resultado::Uso)
                .con("motivo", "argumentos")
                .diagnostico(&txt::USO_BOARD)
                .salida_en(json, idioma);
        }
    };
    match board_en(e, modo) {
        Ok((texto, i)) => {
            let avisos = texto_de_avisos(&i, idioma);
            let mut s = if json {
                i.con("texto", texto).salida_en(true, idioma)
            } else {
                Salida {
                    codigo: 0,
                    stdout: texto,
                    stderr: String::new(),
                }
            };
            s.stderr.push_str(&avisos);
            s
        }
        Err(i) => i.salida_en(json, idioma),
    }
}

/// `board` sobre el estado que conoce el clon: el texto y el informe (sin
/// `texto=`).
pub fn board_en(e: &Entorno<'_>, _modo: Modo) -> Result<(String, Informe), Informe> {
    let (t, avisos) = tablero_en(e.git, e.dir, BOARD)?;
    let plataforma = "no-consultada";
    let texto = texto_board(&t, plataforma, None);
    Ok((texto, informe_board(&t, plataforma, &avisos)))
}

fn informe_board(t: &Tablero, plataforma: &str, avisos: &[&str]) -> Informe {
    let mut i = Informe::nuevo(BOARD, Resultado::Ok)
        .con("commit", t.commit.clone())
        .con("plataforma", plataforma);
    for (clave, _, filas) in t.grupos() {
        i = i.con_lista(clave, ids(filas));
    }
    let quien: Vec<String> = t
        .filas()
        .filter(|f| f.quien != Quien::Nadie)
        .map(|f| format!("{}:{}", f.id, quien_impreso(&f.quien)))
        .collect();
    i = i
        .con_lista("quien", quien)
        .con("terminadas", t.terminadas.to_string())
        .con("descartadas", t.descartadas.to_string());
    if !t.ilegibles.is_empty() {
        i = i.con_lista("ilegibles", t.ilegibles.clone());
    }
    if !avisos.is_empty() {
        i = i.con_lista("avisos", avisos.iter().map(|a| (*a).to_owned()).collect());
    }
    i
}

/// Los avisos de un informe de `board`, una linea cada uno, para stderr.
fn texto_de_avisos(i: &Informe, idioma: Idioma) -> String {
    let mut s = String::new();
    let mut linea = |m: Mensaje| {
        s.push_str(&m.texto(idioma));
        s.push('\n');
    };
    let lista = |clave: &str| -> Vec<String> {
        i.claves
            .iter()
            .find_map(|(k, v)| match v {
                crate::contrato::Valor::Lista(l) if k == clave => Some(l.clone()),
                _ => None,
            })
            .unwrap_or_default()
    };
    if lista("avisos").iter().any(|a| a == PERSONAS_ILEGIBLES) {
        linea(Mensaje::nuevo(&txt_roadmap::AVISO_PERSONAS_ILEGIBLES));
    }
    s
}

/// `Entorno` de la carpeta actual, o el error de no poder leerla.
pub fn carpeta_actual(op: &str) -> Result<std::path::PathBuf, Informe> {
    std::env::current_dir().map_err(|e| {
        Informe::nuevo(op, Resultado::Error)
            .diagnostico(Mensaje::nuevo(&txt_cli::CARPETA_ACTUAL).con("error", e.to_string()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::roadmap::tests::{estado_de_referencia, personas};

    const COMMIT: &str = "0123456789ab";

    /// Valor de lista de `clave=` en una linea de contrato.
    fn lista(linea: &str, clave: &str) -> Vec<String> {
        linea
            .split(' ')
            .find_map(|kv| kv.strip_prefix(clave)?.strip_prefix('='))
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect()
    }

    /// La celda «Con quien» de `id` en las tablas de en curso y en revision
    /// de un ROADMAP.
    fn quien_del_roadmap(roadmap: &str, id: &str) -> Option<String> {
        let prefijo = format!("| {id} | ");
        roadmap
            .lines()
            .filter(|l| l.starts_with(&prefijo))
            // Las tablas de en marcha: | Tarea | Titulo | Con quien | Hito | Depende de |
            .find(|l| l.split(" | ").nth(3).is_some_and(|h| h.starts_with('H')))
            .and_then(|l| l.split(" | ").nth(2).map(str::to_owned))
    }

    /// C3a: sobre el mismo estado, las listas de `board` son las de `next`
    /// (en diseno + en curso + en revision = `en-marcha`, en su orden;
    /// `listas` igual), el «quien» es el texto de `roadmap` y `commit=` es el
    /// mismo.
    #[test]
    fn board_next_y_roadmap_dicen_lo_mismo() {
        let e = estado_de_referencia();
        let s = situacion::calcular(&e);
        assert!(s.is_ok(), "{s:?}");
        let Ok(s) = s else { return };
        let t = tablero(&s, &personas(), COMMIT);
        let next = crate::siguiente::informe_de(&e, COMMIT).map(|i| i.linea());
        assert!(next.is_ok(), "{next:?}");
        let Ok(next) = next else { return };
        let board = informe_board(&t, "no-consultada", &[]).linea();
        let mut en_marcha: Vec<(usize, String)> = Vec::new();
        for g in ["en-diseno", "en-curso", "en-revision"] {
            for id in lista(&board, g) {
                let pos = lista(&next, "en-marcha").iter().position(|x| *x == id);
                assert!(pos.is_some(), "{id} de {g} no esta en next: {next}");
                en_marcha.push((pos.unwrap_or_default(), id));
            }
        }
        // Cada grupo conserva el orden de `next`.
        for g in ["en-diseno", "en-curso", "en-revision"] {
            let v = lista(&board, g);
            let pos: Vec<usize> = v
                .iter()
                .filter_map(|id| lista(&next, "en-marcha").iter().position(|x| x == id))
                .collect();
            assert!(pos.windows(2).all(|w| w[0] < w[1]), "{g}: {board}");
        }
        en_marcha.sort();
        let ids: Vec<String> = en_marcha.into_iter().map(|(_, id)| id).collect();
        assert_eq!(ids, lista(&next, "en-marcha"));
        assert_eq!(lista(&board, "listas"), lista(&next, "listas"));
        assert!(!lista(&board, "listas").is_empty());
        assert_eq!(lista(&board, "commit"), lista(&next, "commit"));
        // El «quien»: el texto de la tabla del ROADMAP.
        let roadmap = crate::roadmap::de_estado(&e, &personas()).unwrap_or_default();
        let mut vistos = 0;
        for f in t.en_diseno.iter().chain(&t.en_curso).chain(&t.en_revision) {
            assert_eq!(
                Some(f.quien.texto()),
                quien_del_roadmap(&roadmap, &f.id),
                "{}",
                f.id
            );
            vistos += 1;
        }
        assert_eq!(vistos, 3, "{board}");
        // Los tres casos: login, heredado convertido por la lista y heredado
        // tal cual.
        assert_eq!(
            lista(&board, "quien"),
            [
                "TASK-008:alguien%20de%20fuera",
                "TASK-002:@ana",
                "TASK-003:@ana-x"
            ]
        );
        assert_eq!(
            (t.terminadas, t.descartadas, t.ilegibles.clone()),
            (2, 1, vec!["TASK-010".to_owned()])
        );
        // `status` da las mismas listas.
        assert!(linea_status(&t).ends_with(&format!("commit {COMMIT}")));
    }

    /// Titulos y logins ajenos se sanean al imprimir.
    #[test]
    fn el_texto_ajeno_se_sanea() {
        let fila = |id: &str, titulo: &str, quien: Quien| Fila {
            id: id.to_owned(),
            titulo: titulo.to_owned(),
            quien,
            rama: None,
        };
        let t = Tablero {
            commit: COMMIT.to_owned(),
            en_curso: vec![fila(
                "TASK-001",
                &format!("rojo \u{1b}[31m{}", "x".repeat(100)),
                Quien::Heredado("ana\u{202E}\nluis".to_owned()),
            )],
            ..Tablero::default()
        };
        let texto = texto_board(&t, "no-consultada", None);
        assert!(!texto.contains('\u{1b}') && !texto.contains('\u{202E}'));
        assert!(texto.contains("ana luis"), "{texto}");
        assert!(texto.lines().all(|l| l.chars().count() < 120), "{texto}");
        let s = linea_status(&t);
        assert!(!s.contains('\u{1b}') && !s.contains('\n'), "{s}");
    }

    /// `status` corta en K tareas por grupo con `+N`.
    #[test]
    fn status_acota_con_mas_n() {
        let filas: Vec<Fila> = (1..=5)
            .map(|n| Fila {
                id: format!("TASK-00{n}"),
                titulo: "t".to_owned(),
                quien: Quien::Nadie,
                rama: None,
            })
            .collect();
        let t = Tablero {
            commit: COMMIT.to_owned(),
            listas: filas,
            ..Tablero::default()
        };
        assert_eq!(
            linea_status(&t),
            format!("listas para empezar: TASK-001, TASK-002, TASK-003, +2 · commit {COMMIT}")
        );
        assert_eq!(
            linea_status(&Tablero {
                commit: COMMIT.to_owned(),
                ..Tablero::default()
            }),
            format!("nada en marcha ni listo para empezar · commit {COMMIT}")
        );
    }

    #[test]
    fn sanear_quita_controles_y_formato_y_acota() {
        assert_eq!(sanear("a\u{1b}[31mb\tc\n", 60), "a [31mb c");
        assert_eq!(sanear("x\u{202E}y\u{200B}z", 60), "xyz");
        assert_eq!(sanear("abcdef", 4), "abc…");
        assert_eq!(sanear("  dos   espacios ", 60), "dos espacios");
    }
}
