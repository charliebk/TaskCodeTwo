//! `tc2 board` y `tc2 status` (F3-5): el estado del equipo, quien lleva que
//! y que espera revision.
//!
//! Las dos ordenes solo leen. Salen de la misma [`situacion::calcular`] sobre
//! [`estado::leer_fresco`] que `next` y `roadmap` (misma foto y mismo
//! `commit=`): traen antes la rama de estado (F3-5b) y dicen la hora de la
//! ultima sincronizacion (`remoto=` y `sincronizado=`; sin red, lo ultimo
//! traido con un aviso en stderr y codigo 0). No escriben nada mas que
//! `origin/taskcodetwo/tareas` y la marca de sincronizacion.
//!
//! - `status [--json]`: una linea de equipo; no consulta la plataforma. Con
//!   `--json`, `status ok en-diseno=... en-curso=... en-revision=... listas=...
//!   commit=... remoto=... sincronizado=... pendientes=N [tareas=...]`. En
//!   texto, `· pendientes N` al final solo si N > 0.
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
use crate::estado::{self, Fallo, Frescura, Local};
use crate::git::Git;
use crate::grafo::{Fase, TERMINADA};
use crate::i18n::catalogo::cli as txt_cli;
use crate::i18n::catalogo::roadmap as txt_roadmap;
use crate::i18n::catalogo::tablero as txt;
use crate::i18n::{Idioma, Mensaje};
use crate::plataforma::{Personas, PrAbierto};
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
    /// Hora de la ultima sincronizacion para el texto (`None`: no se
    /// ensena, sin `origin`).
    pub sincronizado: Option<String>,
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
    if let Some(s) = &t.sincronizado {
        partes.push(format!("sincronizado {s}"));
    }
    partes.join(" · ")
}

/// El texto de `board`: un apartado por grupo con ID, titulo, quien y PR, y
/// el recuento de terminadas y descartadas. `pr` da la celda del PR de cada
/// fila (`None`: sin columna).
pub fn texto_board(t: &Tablero, plataforma: &str, pr: Option<&dyn Fn(&Fila) -> String>) -> String {
    let mut o = format!("Tablero · commit {} · plataforma {plataforma}", t.commit);
    if let Some(s) = &t.sincronizado {
        let _ = write!(o, " · sincronizado {s}");
    }
    o.push('\n');
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

/// Lo leido para `status` y `board`.
struct Leido {
    /// El tablero, con la hora para el texto.
    tablero: Tablero,
    /// Avisos de personas.
    avisos: Vec<&'static str>,
    /// La lista de personas de la base.
    personas: Personas,
    /// La frescura de la lectura.
    frescura: Frescura,
    /// Lo del propio clon, si se pidio (solo `status`).
    local: Option<Local>,
    /// Cuando se leyo, para el «hace N».
    ahora: u64,
}

/// El tablero del estado que conoce el clon de `dir` tras traer la rama de
/// estado (F3-5b), con los avisos de personas. Sin rama de estado,
/// `rechazada motivo=sin-rama-de-estado`.
fn tablero_en(g: &Git, dir: &Path, op: &str, con_local: bool) -> Result<Leido, Informe> {
    let renombrar = |mut i: Informe| {
        op.clone_into(&mut i.comando);
        i
    };
    let ahora = crate::identidad::ahora();
    let (vista, frescura) = estado::leer_fresco(g, dir, ahora).map_err(renombrar)?;
    let local = if con_local {
        Some(estado::local_de(g, &vista).map_err(renombrar)?)
    } else {
        None
    };
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
    let mut t = tablero(&s, &personas, &commit);
    t.sincronizado = frescura.en_texto();
    Ok(Leido {
        tablero: t,
        avisos,
        personas,
        frescura,
        local,
        ahora,
    })
}

// La conexion con la plataforma vive en `identidad` desde F3-8 (la usan
// `board` y `doctor`); se reexporta para no cambiar la API de `board`.
pub use crate::identidad::{Conectada, Conectar, Consulta, conectar_produccion};

/// Lo que necesitan las dos ordenes del entorno.
pub struct Entorno<'a> {
    /// Carpeta desde la que se lanza.
    pub dir: &'a Path,
    /// Puerto Git.
    pub git: &'a Git,
    /// Construye el adaptador; solo lo llama `board` (C3c: `status` no).
    pub conectar: &'a Conectar<'a>,
}

/// El PR de cada tarea y los avisos (C3b).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Cruce {
    /// `ID -> (numero, borrador)` de las filas con PR abierto.
    pub pr: BTreeMap<String, (u64, bool)>,
    /// En revision sin un PR abierto listo (sin PR o solo en borrador).
    pub revision_sin_pr: Vec<String>,
    /// En diseno, en curso o lista con un PR abierto listo (no borrador).
    pub pr_sin_revision: Vec<String>,
}

/// Casa cada fila con el PR abierto de su rama (`head.ref == rama`; con
/// varios, el de numero menor). Una tarea sin rama no casa.
pub fn cruzar(t: &Tablero, prs: &[PrAbierto]) -> Cruce {
    let mut c = Cruce::default();
    for f in t.filas() {
        let pr = f
            .rama
            .as_deref()
            .and_then(|r| prs.iter().filter(|p| p.rama == r).min_by_key(|p| p.numero));
        if let Some(p) = pr {
            c.pr.insert(f.id.clone(), (p.numero, p.borrador));
        }
    }
    let listo = |id: &str| c.pr.get(id).is_some_and(|(_, borrador)| !borrador);
    c.revision_sin_pr = t
        .en_revision
        .iter()
        .filter(|f| !listo(&f.id))
        .map(|f| f.id.clone())
        .collect();
    c.pr_sin_revision = t
        .en_diseno
        .iter()
        .chain(&t.en_curso)
        .chain(&t.listas)
        .filter(|f| listo(&f.id))
        .map(|f| f.id.clone())
        .collect();
    c
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
        Ok((linea, i, avisos)) => {
            let mut s = if json {
                i.salida_en(true, idioma)
            } else {
                Salida {
                    codigo: 0,
                    stdout: format!("{linea}\n"),
                    stderr: String::new(),
                }
            };
            s.stderr.push_str(&lineas(&avisos, idioma));
            s
        }
        Err(i) => i.salida_en(json, idioma),
    }
}

/// Mensajes en lineas para stderr.
pub fn lineas(avisos: &[Mensaje], idioma: Idioma) -> String {
    avisos
        .iter()
        .map(|m| format!("{}\n", m.texto(idioma)))
        .collect()
}

/// `status` sobre el estado recien traido: la linea de texto, el informe y
/// los avisos para stderr (la frescura si no se pudo traer y `tareas/`).
pub fn status_en(e: &Entorno<'_>) -> Result<(String, Informe, Vec<Mensaje>), Informe> {
    let l = tablero_en(e.git, e.dir, STATUS, true)?;
    let t = &l.tablero;
    let mut i = Informe::nuevo(STATUS, Resultado::Ok);
    for (clave, _, filas) in t.grupos() {
        i = i.con_lista(clave, ids(filas));
    }
    i = i.con("commit", t.commit.clone());
    if !t.ilegibles.is_empty() {
        i = i.con_lista("ilegibles", t.ilegibles.clone());
    }
    i = l.frescura.anotar(i);
    let mut avisos: Vec<Mensaje> = l.frescura.aviso(l.ahora).into_iter().collect();
    let mut linea = linea_status(t);
    if let Some(local) = l.local {
        i = i.con("pendientes", local.pendientes.to_string());
        // En texto solo si hay alguna: es lo que falta por publicar.
        if local.pendientes > 0 {
            let _ = write!(linea, " · pendientes {}", local.pendientes);
        }
        if let Some(tareas) = local.tareas {
            i = i.con("tareas", tareas);
            let m = if tareas == "diverge" {
                Mensaje::nuevo(&txt::AVISO_TAREAS_DIVERGE).con("rama", estado::RAMA)
            } else {
                Mensaje::nuevo(&txt::AVISO_TAREAS_CON_CAMBIOS)
            };
            avisos.push(m.con("carpeta", estado::CARPETA));
        }
    }
    Ok((linea, i, avisos))
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
        Ok((texto, i, de_frescura)) => {
            let mut avisos = texto_de_avisos(&i, idioma);
            avisos.push_str(&lineas(&de_frescura, idioma));
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

/// `board` sobre el estado recien traido: el texto, el informe (sin
/// `texto=`, con `remoto=` y `sincronizado=`) y el aviso de frescura para
/// stderr, si lo hay.
pub fn board_en(e: &Entorno<'_>, modo: Modo) -> Result<(String, Informe, Vec<Mensaje>), Informe> {
    // El estado primero: si no hay rama de estado, no se habla con nadie.
    let l = tablero_en(e.git, e.dir, BOARD, false)?;
    let consulta = match modo {
        Modo::SinPlataforma => Consulta::NoConsultada,
        Modo::ConPlataforma => match (e.conectar)(&l.personas) {
            Ok(c) => match c.plataforma.prs_abiertos(c.rol) {
                Ok(prs) => Consulta::Hecha(prs),
                Err(i) => Consulta::de_fallo(&i),
            },
            Err(c) => c,
        },
    };
    let (texto, i) = board_de(&l.tablero, &consulta, &l.avisos);
    Ok((
        texto,
        l.frescura.anotar(i),
        l.frescura.aviso(l.ahora).into_iter().collect(),
    ))
}

/// Texto e informe de `board` para un tablero y lo que se supo de la
/// plataforma (funcion pura, C3b y C3c).
pub fn board_de(t: &Tablero, consulta: &Consulta, avisos: &[&str]) -> (String, Informe) {
    let cruce = match consulta {
        Consulta::Hecha(prs) => Some(cruzar(t, prs)),
        _ => None,
    };
    let celda = |f: &Fila| -> String {
        match cruce.as_ref().and_then(|c| c.pr.get(&f.id)) {
            Some((n, true)) => format!("#{n} (borrador)"),
            Some((n, false)) => format!("#{n}"),
            None => "—".to_owned(),
        }
    };
    let texto = texto_board(
        t,
        consulta.nombre(),
        cruce.as_ref().map(|_| &celda as &dyn Fn(&Fila) -> String),
    );
    (
        texto,
        informe_board(t, consulta.nombre(), cruce.as_ref(), avisos),
    )
}

fn informe_board(t: &Tablero, plataforma: &str, cruce: Option<&Cruce>, avisos: &[&str]) -> Informe {
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
    i = i.con_lista("quien", quien);
    // Sin consulta hecha no hay pendientes que avisar (C3c).
    if let Some(c) = cruce {
        let con_pr: Vec<&Fila> = t.filas().filter(|f| c.pr.contains_key(&f.id)).collect();
        let prs = con_pr
            .iter()
            .filter_map(|f| c.pr.get(&f.id).map(|(n, _)| format!("{}#{n}", f.id)))
            .collect();
        i = i.con_lista("prs", prs);
        let borradores: Vec<String> = con_pr
            .iter()
            .filter(|f| c.pr.get(&f.id).is_some_and(|(_, b)| *b))
            .map(|f| f.id.clone())
            .collect();
        for (clave, v) in [
            ("borradores", borradores),
            ("revision-sin-pr", c.revision_sin_pr.clone()),
            ("pr-sin-revision", c.pr_sin_revision.clone()),
        ] {
            if !v.is_empty() {
                i = i.con_lista(clave, v);
            }
        }
    }
    i = i
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
    let plataforma = match i.valor("plataforma") {
        Some("no-configurada") => Some(&txt::AVISO_PLATAFORMA_NO_CONFIGURADA),
        Some("sin-credenciales") => Some(&txt::AVISO_PLATAFORMA_SIN_CREDENCIALES),
        Some("sin-red") => Some(&txt::AVISO_PLATAFORMA_SIN_RED),
        Some("error") => Some(&txt::AVISO_PLATAFORMA_ERROR),
        _ => None,
    };
    if let Some(m) = plataforma {
        linea(Mensaje::nuevo(m));
    }
    for (clave, m) in [
        ("revision-sin-pr", &txt::AVISO_REVISION_SIN_PR),
        ("pr-sin-revision", &txt::AVISO_PR_SIN_REVISION),
    ] {
        let v = lista(clave);
        if !v.is_empty() {
            linea(Mensaje::nuevo(m).con("ids", v.join(", ")));
        }
    }
    if lista("avisos").iter().any(|a| a == PERSONAS_ILEGIBLES) {
        linea(Mensaje::nuevo(&txt_roadmap::AVISO_PERSONAS_ILEGIBLES));
    }
    s
}

/// La carpeta actual, o el error de no poder leerla.
pub fn carpeta_actual(op: &str) -> Result<std::path::PathBuf, Informe> {
    std::env::current_dir().map_err(|e| {
        Informe::nuevo(op, Resultado::Error)
            .diagnostico(Mensaje::nuevo(&txt_cli::CARPETA_ACTUAL).con("error", e.to_string()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plataforma::{Firma, ModeloPuertas, Paso, Plataforma, ReglasLeidas, Rol, Usuario};
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
        let board = board_de(&t, &Consulta::NoConsultada, &[]).1.linea();
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

    /// Plataforma falsa: solo `prs-abiertos`, con la respuesta dada.
    struct Falsa(Paso<Vec<PrAbierto>>);

    impl Plataforma for Falsa {
        fn prs_abiertos(&self, _: Rol) -> Paso<Vec<PrAbierto>> {
            self.0.clone()
        }
        fn abrir_pr(&self, _: &str, _: &str, _: &str, _: &str) -> Informe {
            Informe::nuevo("abrir-pr", Resultado::Error)
        }
        fn estado_puertas(&self, _: Rol, _: u64, _: &ModeloPuertas) -> Informe {
            Informe::nuevo("estado-puertas", Resultado::Error)
        }
        fn merge_pr(&self, _: u64, _: &str) -> Informe {
            Informe::nuevo("merge-pr", Resultado::Error)
        }
        fn leer_puertas(&self, _: Rol, _: &str) -> Paso<ReglasLeidas> {
            Err(Informe::nuevo("leer-puertas", Resultado::Error))
        }
        fn leer_permisos_ci(&self) -> Informe {
            Informe::nuevo("leer-permisos-ci", Resultado::Error)
        }
        fn firma_commit(&self, _: &str) -> Paso<Firma> {
            Err(Informe::nuevo("firma-commit", Resultado::Error))
        }
        fn identidad(&self, _: Rol) -> Paso<Usuario> {
            Err(Informe::nuevo("identidad", Resultado::Error))
        }
    }

    fn pr(numero: u64, rama: &str, borrador: bool) -> PrAbierto {
        PrAbierto {
            numero,
            rama: rama.to_owned(),
            sha: "a".repeat(40),
            borrador,
        }
    }

    fn fila(id: &str, rama: Option<&str>, quien: Quien) -> Fila {
        Fila {
            id: id.to_owned(),
            titulo: format!("Titulo de {id}"),
            quien,
            rama: rama.map(str::to_owned),
        }
    }

    /// Un tablero con cada caso de cruce.
    fn de_prueba() -> Tablero {
        Tablero {
            commit: COMMIT.to_owned(),
            en_diseno: vec![fila("TASK-001", Some("feature/a"), Quien::Nadie)],
            en_curso: vec![fila(
                "TASK-002",
                Some("feature/b"),
                Quien::Persona("ana".into()),
            )],
            en_revision: vec![
                fila("TASK-003", Some("feature/c"), Quien::Persona("luis".into())),
                fila("TASK-004", Some("feature/d"), Quien::Nadie),
                fila("TASK-005", Some("feature/e"), Quien::Nadie),
            ],
            listas: vec![fila("TASK-006", None, Quien::Nadie)],
            terminadas: 4,
            descartadas: 1,
            ilegibles: Vec::new(),
            sincronizado: None,
        }
    }

    /// C3 (parte pura): la hora va al final de la linea de `status` y de la
    /// cabecera de `board`; sin hora (sin `origin`), no sale.
    #[test]
    fn la_hora_en_la_linea_y_la_cabecera() {
        let t = Tablero {
            sincronizado: Some("2026-11-17T16:04:12Z".to_owned()),
            ..de_prueba()
        };
        assert!(
            linea_status(&t).ends_with(&format!(
                "commit {COMMIT} · sincronizado 2026-11-17T16:04:12Z"
            )),
            "{}",
            linea_status(&t)
        );
        assert!(texto_board(&t, "ok", None).starts_with(
            "Tablero · commit 0123456789ab · plataforma ok · sincronizado 2026-11-17T16:04:12Z\n"
        ));
        assert!(!linea_status(&de_prueba()).contains("sincronizado"));
        assert!(!texto_board(&de_prueba(), "ok", None).contains("sincronizado"));
    }

    /// C3b: `board` lista los cuatro grupos con ID, titulo, quien y PR, y
    /// avisa `revision-sin-pr=` (sin PR o solo en borrador) y
    /// `pr-sin-revision=` (PR listo de una tarea que no esta en revision).
    #[test]
    fn board_cruza_los_pr_y_avisa() {
        let falsa = Falsa(Ok(vec![
            pr(3, "feature/c", false),
            pr(5, "feature/b", true),
            pr(7, "feature/a", false),
            pr(9, "feature/e", true),
            pr(11, "feature/c", false),
            pr(12, "feature/otra", false),
        ]));
        let consulta = match falsa.prs_abiertos(Rol::Agente) {
            Ok(v) => Consulta::Hecha(v),
            Err(i) => Consulta::de_fallo(&i),
        };
        let t = de_prueba();
        let (texto, i) = board_de(&t, &consulta, &[]);
        assert_eq!(
            i.linea(),
            "board ok commit=0123456789ab plataforma=ok en-diseno=TASK-001 en-curso=TASK-002 \
             en-revision=TASK-003,TASK-004,TASK-005 listas=TASK-006 \
             quien=TASK-002:@ana,TASK-003:@luis \
             prs=TASK-001#7,TASK-002#5,TASK-003#3,TASK-005#9 borradores=TASK-002,TASK-005 \
             revision-sin-pr=TASK-004,TASK-005 pr-sin-revision=TASK-001 terminadas=4 descartadas=1"
        );
        for esperado in [
            "Tablero · commit 0123456789ab · plataforma ok\n",
            "\nEn diseño (1)\n  TASK-001  Titulo de TASK-001  —  #7\n",
            "\nEn curso (1)\n  TASK-002  Titulo de TASK-002  @ana  #5 (borrador)\n",
            "  TASK-003  Titulo de TASK-003  @luis  #3\n",
            "  TASK-004  Titulo de TASK-004  —      —\n",
            "\nListas para empezar (1)\n  TASK-006  Titulo de TASK-006  —  —\n",
            "\nTerminadas: 4 · Descartadas: 1\n",
        ] {
            assert!(texto.contains(esperado), "falta {esperado:?} en:\n{texto}");
        }
        // Los avisos van a stderr, en los dos idiomas.
        let es = texto_de_avisos(&i, Idioma::Es);
        assert_eq!(
            es,
            "aviso: en revision sin PR abierto listo: TASK-004, TASK-005\n\
             aviso: con PR abierto listo y sin pasar a revision: TASK-001\n"
        );
        assert!(texto_de_avisos(&i, Idioma::En).starts_with("warning: in review"));
    }

    /// C3c (parte pura): con la plataforma fallando, la parte de estado sale
    /// entera, `plataforma=` dice por que y no hay pendientes (ni `prs=` ni
    /// avisos de revision); sin columna de PR en el texto.
    #[test]
    fn plataforma_que_falla_deja_el_estado_entero() {
        let t = de_prueba();
        let (base, _) = board_de(&t, &Consulta::NoConsultada, &[]);
        for (fallo, nombre) in [
            (Informe::nuevo("prs-abiertos", Resultado::Sinred), "sin-red"),
            (
                Informe::nuevo("prs-abiertos", Resultado::Rechazada).con("motivo", "identidad"),
                "sin-credenciales",
            ),
            (
                Informe::nuevo("prs-abiertos", Resultado::Denegada).con("motivo", "credenciales"),
                "sin-credenciales",
            ),
            (
                Informe::nuevo("prs-abiertos", Resultado::Error)
                    .con("motivo", "demasiadas-paginas"),
                "error",
            ),
            (
                Informe::nuevo("prs-abiertos", Resultado::Denegada).con("motivo", "permisos"),
                "error",
            ),
        ] {
            let falsa = Falsa(Err(fallo));
            let consulta = match falsa.prs_abiertos(Rol::Agente) {
                Ok(v) => Consulta::Hecha(v),
                Err(i) => Consulta::de_fallo(&i),
            };
            let (texto, i) = board_de(&t, &consulta, &[]);
            assert_eq!(i.resultado, Resultado::Ok);
            assert_eq!(i.valor("plataforma"), Some(nombre));
            let linea = i.linea();
            for clave in [
                "prs=",
                "borradores=",
                "revision-sin-pr=",
                "pr-sin-revision=",
            ] {
                assert!(!linea.contains(clave), "{linea}");
            }
            assert!(linea.contains(
                "en-diseno=TASK-001 en-curso=TASK-002 en-revision=TASK-003,TASK-004,TASK-005 listas=TASK-006"
            ));
            // El mismo texto que sin consultar, salvo la cabecera.
            assert_eq!(
                texto.replace(&format!("plataforma {nombre}"), "plataforma no-consultada"),
                base
            );
            let avisos = texto_de_avisos(&i, Idioma::Es);
            assert_eq!(avisos.lines().count(), 1, "{avisos}");
            assert!(avisos.starts_with("aviso: no se"), "{avisos}");
        }
    }
}
