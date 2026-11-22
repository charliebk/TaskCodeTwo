//! `tc2 seleccion` (F4-6, [`orden_seleccion`]): los tres roles de una tarea
//! sin escribir nada.
//!
//! `tc2 revisoras [--json] --rutas <fichero|-> [--nul] [--tarea TASK-NNN]
//! [--carril <c>] [--complejidad <x>] [--plugin <nombre>=<dir>]...
//! [--informe <fichero> --seccion <salida> [--artefacto <n>]]
//! [--ronda <n>] [--detalle <salida.json>]` (F6-3).
//!
//! Las rutas son dato (una por linea, o separadas por NUL con `--nul`; los
//! dos nombres de cada renombrado), con un tope de [`TOPE_RUTAS`] bytes
//! tanto por fichero como por stdin: en CI salen de
//! `git -c core.quotepath=false diff --name-only -z --no-renames base...head`
//! sobre el checkout de la base (con `-z` las rutas no ASCII llegan literales,
//! sin comillas ni escapes); en local, de [`crate::git::ficheros_de_rama`]. La
//! configuracion y las skills del proyecto se leen de la base
//! (`origin/develop`), nunca del worktree del PR (F1-15 decision 2).
//!
//! Inventario y catalogo:
//! - con `--plugin <nombre>=<dir>` (el revisor en CI, con el plugin a una
//!   version fijada), solo esos plugins (`<dir>/skills/<n>/SKILL.md`), las
//!   skills del proyecto en la base y los agentes integrados: no se lee la
//!   carpeta de Claude Code de la maquina, y el resultado no depende de lo
//!   que tenga instalado el runner;
//! - sin `--plugin`, el inventario de la maquina ([`crate::inventario::leer`])
//!   con las skills del proyecto sustituidas por las de la base.
//!
//! Carril y complejidad: con `--tarea`, los de la tarea en la rama de estado
//! (carril: el mayor del declarado por el tipo y el congelado; complejidad:
//! la mayor entre la declarada y la heuristica de la 0.16). `--carril` solo
//! sube; `--complejidad` manda sobre la de la tarea. Sin nada: funcionalidad
//! y complejidad desconocida (media, con aviso).
//!
//! `--informe` (seguimiento de TASK-044): el informe de verificacion de F5-4
//! se lee con un tope de [`TOPE_INFORME`] bytes; si vale, `--seccion` recibe
//! [`seccion_revisor`] y la linea dice `informe=verde|rojo`; si no,
//! `informe=grande|invalido|sin-informe` y no se escribe seccion.
//!
//! `--ronda` (F6-6): la ronda de la revision (por defecto 1), para las
//! reglas con `ronda>=N` (S13). En CI sale de `ronda.json` de `tc2 hallazgos
//! ronda`.
//!
//! Linea: `revisoras ok revisoras=<ids> agentes=<ids> modelo=<m>
//! modelo-api=<id> reglas=<ids> avisos=<...> carril=<c> complejidad=<x>
//! [informe=<...>]`. Lo de cada revisora (su `SKILL.md` y las rutas que
//! recibe) no cabe en claves del contrato F1-4: va, con `--detalle`, a un
//! JSON `{"revisoras":[{"id","skill","rutas"}]}` que lee el workflow.

use std::io::Read;
use std::path::{Path, PathBuf};

use super::catalogo::{self, Catalogo};
use super::{Contexto, Seleccion};
use crate::carril::Carril;
use crate::contrato::{Informe, Resultado};
use crate::git::Git;
use crate::i18n::Mensaje;
use crate::i18n::catalogo::seleccion as txt;
use crate::inventario::{self, Inventario, Origen};
use crate::plan::heuristica;
use crate::tarea::{Cuerpo, Tarea};
use crate::verificacion::informe::{self, seccion_revisor};

/// Nombre de la orden.
pub const NOMBRE: &str = "revisoras";

/// Bytes que se leen como mucho del informe de verificacion: un informe v1
/// real ocupa menos de 2 KiB; lo que pase de aqui no se cita.
pub const TOPE_INFORME: u64 = 64 * 1024;

/// Bytes que se leen como mucho del fichero de rutas.
pub const TOPE_RUTAS: u64 = 4 * 1024 * 1024;

/// Lo que necesita la orden del entorno.
pub struct Entorno<'a> {
    /// Carpeta del repo (el checkout de la base en CI).
    pub dir: &'a Path,
    /// Puerto Git.
    pub git: &'a Git,
    /// Carpeta de Claude Code, si se sabe.
    pub claude: Option<&'a Path>,
    /// Lee la entrada estandar (para `--rutas -`): como mucho
    /// `TOPE_RUTAS + 1` bytes, para que el exceso se vea y falle igual que
    /// con fichero.
    pub stdin: &'a dyn Fn() -> std::io::Result<Vec<u8>>,
}

/// Inventario y catalogo con los que se selecciona, y los avisos de
/// leerlos.
#[derive(Debug, Clone, Default)]
pub struct Fuentes {
    /// Lo instalado.
    pub inventario: Inventario,
    /// Las entradas con metadatos (revisoras y skills de diseno o
    /// ejecucion).
    pub catalogo: Catalogo,
    /// Avisos (`sin-base`).
    pub avisos: Vec<String>,
}

/// Inventario y catalogo para `dir` (ver el comentario del modulo).
pub fn fuentes(
    g: &Git,
    dir: &Path,
    claude: Option<&Path>,
    plugins: &[(String, PathBuf)],
) -> Fuentes {
    let mut avisos = Vec::new();
    let base = match crate::git::commit_de(g, dir, NOMBRE, crate::config::BASE) {
        Ok(Some(_)) => catalogo::de_base(g, dir, crate::config::BASE).unwrap_or_else(|_| {
            avisos.push("sin-skills-de-base".to_owned());
            catalogo::Lectura::default()
        }),
        _ => {
            avisos.push("sin-base".to_owned());
            catalogo::Lectura::default()
        }
    };
    let mut entradas = base.entradas;
    let mut revisoras = base.revisoras;
    let mut lista_plugins = Vec::new();
    if plugins.is_empty() {
        let l = inventario::leer(claude, dir);
        if let Some(c) = claude {
            revisoras.extend(catalogo::de_instalados(c, &l.inventario));
        }
        entradas.extend(
            l.inventario
                .entradas()
                .iter()
                .filter(|e| e.origen != Origen::Proyecto)
                .cloned(),
        );
        lista_plugins.extend(l.inventario.plugins().iter().cloned());
    } else {
        for (nombre, d) in plugins {
            let origen = Origen::Plugin {
                nombre: nombre.clone(),
                version: None,
            };
            let l = catalogo::de_carpeta(&d.join("skills"), &format!("{nombre}:"), &origen);
            entradas.extend(l.entradas);
            revisoras.extend(l.revisoras);
        }
        for id in inventario::INTEGRADOS {
            entradas.push(inventario::Entrada {
                tipo: inventario::Tipo::Agente,
                id: id.to_owned(),
                origen: Origen::Integrado,
            });
        }
    }
    // Lo que declara `recursos:` de la configuracion de la base (F4-6).
    if let Ok(cfg) = crate::config::de_base(g, dir) {
        revisoras.extend(catalogo::de_config(&cfg.recursos));
    }
    Fuentes {
        inventario: Inventario::nuevo(entradas, lista_plugins),
        catalogo: Catalogo::nuevo(revisoras),
        avisos,
    }
}

/// Carril efectivo y complejidad de una tarea: el carril mayor entre el
/// declarado por el tipo y el congelado; la complejidad mayor entre la
/// declarada y la heuristica (F1-15 decision 1).
pub fn de_tarea(t: &Tarea, c: &Cuerpo) -> (Carril, &'static str) {
    let declarado = Carril::declarado(&t.tipo);
    let carril = t
        .carril
        .as_deref()
        .and_then(Carril::desde)
        .map_or(declarado, |x| x.max(declarado));
    let e = heuristica::Enunciado::de(c);
    let r = heuristica::resolver(
        t.complejidad.as_deref(),
        &t.etiquetas,
        &t.tipo,
        &t.dependencias,
        &e,
    );
    (carril, complejidad(&r))
}

/// Los dominios de una tarea para la revision (F4-6; F1-15 decision 3):
/// los del texto vivo y las `rutas` del diff con `dominios_extra` de la
/// configuracion de la base, mas las senales congeladas con el plan
/// aprobado (`roles.diseno.senales`): reescribir la tarea no quita un
/// dominio. Sin tarea, solo las rutas.
pub fn dominios_de(
    tarea: Option<(&Tarea, &Cuerpo)>,
    rutas: &[String],
    cfg: &crate::config::Config,
) -> super::dominios::Dominios {
    let tabla = super::tabla::dominios(super::tabla::ADR).unwrap_or_default();
    let extra = &cfg.seleccion.dominios_extra;
    match tarea {
        Some((t, c)) => {
            let e = heuristica::Enunciado::de(c);
            let texto = super::dominios::Texto {
                objetivo: &e.objetivo,
                criterios: &e.criterios,
                etiquetas: &t.etiquetas,
            };
            let d = super::dominios::dominios(&texto, rutas, &tabla, extra);
            super::dominios::union(&d, &t.roles.diseno.senales)
        }
        None => {
            let texto = super::dominios::Texto {
                objetivo: "",
                criterios: &[],
                etiquetas: &[],
            };
            super::dominios::dominios(&texto, rutas, &tabla, extra)
        }
    }
}

/// La complejidad de la seleccion: la mayor entre la declarada y la
/// heuristica (F1-15 decision 1).
pub fn complejidad(r: &heuristica::Resolucion) -> &'static str {
    let indice = |n: &str| heuristica::NIVELES.iter().position(|x| *x == n);
    r.declarado
        .as_deref()
        .and_then(indice)
        .into_iter()
        .chain(indice(r.heuristico))
        .max()
        .and_then(|i| heuristica::NIVELES.get(i).copied())
        .unwrap_or(r.heuristico)
}

/// Las claves de la seleccion en un informe.
pub fn con_seleccion(i: Informe, s: &Seleccion) -> Informe {
    i.con_lista("revisoras", s.revisoras.clone())
        .con_lista("agentes", s.agentes.clone())
        .con("modelo", s.modelo.as_str())
        .con("modelo-api", super::modelo_api(&s.modelo))
        .con_lista("reglas", s.reglas.clone())
        .con_lista("avisos", s.avisos.clone())
}

/// El detalle por revisora: `{"revisoras":[{"id","skill","rutas"}]}`, en el
/// orden de la seleccion.
pub fn detalle(s: &Seleccion, cat: &Catalogo) -> String {
    let v: Vec<serde_json::Value> = s
        .revisoras
        .iter()
        .map(|id| {
            serde_json::json!({
                "id": id,
                "skill": cat.revisora(id).map(|r| r.ruta.clone()),
                "rutas": s.rutas.get(id).cloned().unwrap_or_default(),
            })
        })
        .collect();
    serde_json::json!({ "revisoras": v }).to_string()
}

/// Lee como mucho `tope` bytes de `p`: `Ok(None)` si pasa del tope.
fn leer_con_tope(p: &Path, tope: u64) -> std::io::Result<Option<String>> {
    let mut b = Vec::new();
    std::fs::File::open(p)?.take(tope + 1).read_to_end(&mut b)?;
    if b.len() as u64 > tope {
        return Ok(None);
    }
    Ok(Some(String::from_utf8_lossy(&b).into_owned()))
}

/// Estado del informe de verificacion y, si vale, su seccion para el
/// revisor (artefacto `artefacto`).
pub fn informe_revisor(p: &Path, artefacto: &str) -> (&'static str, Option<String>) {
    match leer_con_tope(p, TOPE_INFORME) {
        Err(_) => ("sin-informe", None),
        Ok(None) => ("grande", None),
        Ok(Some(t)) => match informe::leer(&t) {
            Ok(i) => {
                let estado = match i.resultado {
                    informe::Resultado::Verde => "verde",
                    informe::Resultado::Rojo => "rojo",
                };
                (estado, Some(seccion_revisor(&i, artefacto)))
            }
            Err(_) => ("invalido", None),
        },
    }
}

/// Argumentos de la orden.
#[derive(Default)]
struct Args {
    rutas: Option<String>,
    nul: bool,
    tarea: Option<String>,
    carril: Option<Carril>,
    complejidad: Option<String>,
    plugins: Vec<(String, PathBuf)>,
    informe: Option<PathBuf>,
    artefacto: Option<String>,
    seccion: Option<PathBuf>,
    detalle: Option<PathBuf>,
    ronda: Option<u32>,
}

/// Las rutas del texto: con `nul`, separadas por NUL y literales (la salida
/// de `git diff --name-only -z`, sin las comillas ni los escapes que git
/// pone a lo no ASCII); sin `nul`, una por linea (sin el `\r` final).
fn separar_rutas(texto: &str, nul: bool) -> Vec<String> {
    let trozos: Vec<&str> = if nul {
        texto.split('\0').collect()
    } else {
        texto.lines().map(|l| l.trim_end_matches('\r')).collect()
    };
    trozos
        .into_iter()
        .filter(|l| !l.trim().is_empty())
        .map(str::to_owned)
        .collect()
}

fn uso(motivo: &str) -> Informe {
    Informe::nuevo(NOMBRE, Resultado::Uso)
        .con("motivo", motivo)
        .diagnostico(&txt::USO_REVISORAS)
}

fn args(a: &[String]) -> Result<Args, Informe> {
    let mut r = Args::default();
    let mut it = a.iter();
    while let Some(x) = it.next() {
        let mut valor = || it.next().cloned().ok_or_else(|| uso("falta-valor"));
        match x.as_str() {
            "--rutas" if r.rutas.is_none() => r.rutas = Some(valor()?),
            "--nul" if !r.nul => r.nul = true,
            "--tarea" if r.tarea.is_none() => {
                let v = valor()?;
                if !crate::git::es_id(&v) {
                    return Err(uso("id-invalido"));
                }
                r.tarea = Some(v);
            }
            "--carril" if r.carril.is_none() => {
                r.carril = Some(Carril::desde(&valor()?).ok_or_else(|| uso("carril-invalido"))?);
            }
            "--complejidad" if r.complejidad.is_none() => {
                let v = valor()?;
                if !crate::tarea::COMPLEJIDADES.contains(&v.as_str()) {
                    return Err(uso("complejidad-invalida"));
                }
                r.complejidad = Some(v);
            }
            "--plugin" => {
                let v = valor()?;
                let Some((n, d)) = v.split_once('=').filter(|(n, d)| {
                    !n.is_empty()
                        && !d.is_empty()
                        && n.chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                }) else {
                    return Err(uso("plugin-invalido"));
                };
                r.plugins.push((n.to_owned(), PathBuf::from(d)));
            }
            "--informe" if r.informe.is_none() => r.informe = Some(PathBuf::from(valor()?)),
            "--artefacto" if r.artefacto.is_none() => r.artefacto = Some(valor()?),
            "--seccion" if r.seccion.is_none() => r.seccion = Some(PathBuf::from(valor()?)),
            "--detalle" if r.detalle.is_none() => r.detalle = Some(PathBuf::from(valor()?)),
            "--ronda" if r.ronda.is_none() => {
                let n = valor()?
                    .parse::<u32>()
                    .ok()
                    .filter(|n| *n >= 1)
                    .ok_or_else(|| uso("ronda-invalida"))?;
                r.ronda = Some(n);
            }
            _ => return Err(uso("argumento-desconocido")),
        }
    }
    if r.rutas.is_none() {
        return Err(uso("sin-rutas"));
    }
    if r.informe.is_some() != r.seccion.is_some() {
        return Err(uso("informe-sin-seccion"));
    }
    Ok(r)
}

/// `revisoras ...` (sin `--json`, que quita `lib.rs`).
pub fn orden(a: &[String], e: &Entorno<'_>) -> Informe {
    match paso(a, e) {
        Ok(i) | Err(i) => i,
    }
}

fn paso(a: &[String], e: &Entorno<'_>) -> Result<Informe, Informe> {
    let a = args(a)?;
    let fallo_lectura = |ruta: &str, error: String| {
        Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(
            Mensaje::nuevo(&txt::NO_SE_LEE)
                .con("ruta", ruta)
                .con("error", error),
        )
    };
    let texto = match a.rutas.as_deref() {
        Some("-") => {
            let b = (e.stdin)().map_err(|x| fallo_lectura("-", x.to_string()))?;
            if b.len() as u64 > TOPE_RUTAS {
                return Err(fallo_lectura("-", format!("mas de {TOPE_RUTAS} bytes")));
            }
            String::from_utf8(b).map_err(|x| fallo_lectura("-", x.to_string()))?
        }
        Some(f) => leer_con_tope(Path::new(f), TOPE_RUTAS)
            .map_err(|x| fallo_lectura(f, x.to_string()))?
            .ok_or_else(|| fallo_lectura(f, format!("mas de {TOPE_RUTAS} bytes")))?,
        None => return Err(uso("sin-rutas")),
    };
    let rutas = separar_rutas(&texto, a.nul);
    let f = fuentes(e.git, e.dir, e.claude, &a.plugins);
    let mut avisos = f.avisos.clone();
    let mut carril = Carril::Funcionalidad;
    let mut cx: Option<String> = None;
    let config = crate::config::de_base(e.git, e.dir).unwrap_or_else(|_| {
        avisos.push("config-de-base-no-leida".to_owned());
        crate::config::Config::suelo()
    });
    let mut doms = dominios_de(None, &rutas, &config);
    if let Some(id) = &a.tarea {
        let leida = crate::estado::leer(e.git, e.dir)
            .map_err(|i| i.valor("motivo").unwrap_or("estado").to_owned())
            .and_then(|v| {
                crate::estado::tarea_de(&v, id).map_err(|f| match f {
                    crate::estado::Fallo::Rechazada { motivo, .. } => motivo,
                    crate::estado::Fallo::Error(_) => "error".to_owned(),
                })
            });
        match leida {
            Ok((t, c)) => {
                let (k, x) = de_tarea(&t, &c);
                carril = k;
                cx = Some(x.to_owned());
                doms = dominios_de(Some((&t, &c)), &rutas, &config);
            }
            Err(m) => avisos.push(format!("tarea-no-leida:{m}")),
        }
    }
    if let Some(c) = a.carril {
        carril = carril.max(c);
    }
    if a.complejidad.is_some() {
        cx = a.complejidad.clone();
    }
    let ctx = Contexto {
        carril,
        complejidad: cx.as_deref(),
        modelos_permitidos: &config.seleccion.modelos_permitidos,
        ronda: a.ronda.unwrap_or(1),
    };
    let entrada = super::Entrada {
        rutas: &rutas,
        dominios: &doms,
        etiquetas: &[],
        roles_base: 0,
    };
    let mut s = super::seleccionar(
        super::Rol::Revision,
        &entrada,
        &f.catalogo,
        &f.inventario,
        &ctx,
    )
    .map_err(|x| {
        Informe::nuevo(NOMBRE, Resultado::Error)
            .diagnostico(Mensaje::nuevo(&txt::TABLA_ILEGIBLE).con("error", x))
    })?;
    avisos.append(&mut s.avisos);
    s.avisos = avisos;
    let mut i = con_seleccion(Informe::nuevo(NOMBRE, Resultado::Ok), &s)
        .con("carril", carril.nombre())
        .con("complejidad", cx.as_deref().unwrap_or("desconocida"));
    if let (Some(inf), Some(sec)) = (&a.informe, &a.seccion) {
        let artefacto = a.artefacto.as_deref().unwrap_or("informe-verificacion");
        let (estado, seccion) = informe_revisor(inf, artefacto);
        if let Some(texto) = seccion {
            std::fs::write(sec, texto)
                .map_err(|x| fallo_lectura(&sec.display().to_string(), x.to_string()))?;
        }
        i = i.con("informe", estado);
    }
    if let Some(d) = &a.detalle {
        std::fs::write(d, detalle(&s, &f.catalogo))
            .map_err(|x| fallo_lectura(&d.display().to_string(), x.to_string()))?;
    }
    Ok(i)
}

/// Nombre de la orden que da la seleccion de los tres roles (F4-6).
pub const SELECCION: &str = "seleccion";

fn uso_seleccion(motivo: &str) -> Informe {
    Informe::nuevo(SELECCION, Resultado::Uso)
        .con("motivo", motivo)
        .diagnostico(&txt::USO_SELECCION)
}

/// Una tarea leida de un fichero en formato 1 (0.16) o 2: la de la 0.16 se
/// pasa al modelo de formato 2 con lo que la seleccion mira (tipo,
/// etiquetas, complejidad y dependencias).
fn tarea_de_fichero(texto: &str) -> Result<(Tarea, Cuerpo), String> {
    match crate::tarea::leer(texto).map_err(|e| e.to_string())? {
        crate::tarea::Documento::Formato2(t, c) => Ok((t, c)),
        crate::tarea::Documento::Formato1(v, c) => {
            let mut t = Tarea::nueva(&v.id, &v.titulo, &v.creado);
            t.tipo = v.tipo;
            t.etiquetas = v.etiquetas;
            t.complejidad = v.complejidad;
            t.dependencias = v.dependencias;
            Ok((t, c))
        }
    }
}

/// `seleccion [--json] [TASK-NNN] [--tarea-fichero <tarea.md>] [--rutas
/// <fichero>]` (F4-6): la seleccion de diseno, implementacion y revision de
/// una tarea sin escribir nada (contrato para el orquestador, F4-4, y
/// `lote`, F1-16). La tarea sale de la rama de estado o, con
/// `--tarea-fichero`, de un `tarea.md` en formato 1 o 2 (para medir sobre
/// tareas reales de la 0.16). Diseno e implementacion se calculan como en
/// `plan` (el texto, sin rutas, y las senales congeladas); revision, con
/// las `--rutas` del diff. Inventario, catalogo y configuracion, los de
/// [`fuentes`] y la base.
pub fn orden_seleccion(a: &[String], e: &Entorno<'_>) -> Informe {
    match paso_seleccion(a, e) {
        Ok(i) | Err(i) => i,
    }
}

fn paso_seleccion(a: &[String], e: &Entorno<'_>) -> Result<Informe, Informe> {
    let (mut id, mut fichero, mut rutas_f) = (None, None, None);
    let mut it = a.iter();
    while let Some(x) = it.next() {
        let mut valor = || {
            it.next()
                .cloned()
                .ok_or_else(|| uso_seleccion("falta-valor"))
        };
        match x.as_str() {
            "--tarea-fichero" if fichero.is_none() => fichero = Some(valor()?),
            "--rutas" if rutas_f.is_none() => rutas_f = Some(valor()?),
            v if id.is_none() && crate::git::es_id(v) => id = Some(v.to_owned()),
            _ => return Err(uso_seleccion("argumento-desconocido")),
        }
    }
    let error = |ruta: &str, err: String| {
        Informe::nuevo(SELECCION, Resultado::Error).diagnostico(
            Mensaje::nuevo(&txt::NO_SE_LEE)
                .con("ruta", ruta)
                .con("error", err),
        )
    };
    let (t, c) = match (&fichero, &id) {
        (Some(f), _) => {
            let texto = leer_con_tope(Path::new(f), TOPE_RUTAS)
                .map_err(|x| error(f, x.to_string()))?
                .ok_or_else(|| error(f, format!("mas de {TOPE_RUTAS} bytes")))?;
            tarea_de_fichero(&texto).map_err(|x| error(f, x))?
        }
        (None, Some(id)) => {
            let v = crate::estado::leer(e.git, e.dir).map_err(|mut i| {
                SELECCION.clone_into(&mut i.comando);
                i
            })?;
            crate::estado::tarea_de(&v, id).map_err(|f| match f {
                crate::estado::Fallo::Rechazada {
                    motivo,
                    diagnostico,
                } => Informe::nuevo(SELECCION, Resultado::Rechazada)
                    .con("motivo", motivo)
                    .diagnostico(diagnostico),
                crate::estado::Fallo::Error(m) => {
                    Informe::nuevo(SELECCION, Resultado::Error).diagnostico(m)
                }
            })?
        }
        (None, None) => return Err(uso_seleccion("sin-tarea")),
    };
    let rutas = match &rutas_f {
        Some(f) => separar_rutas(
            &leer_con_tope(Path::new(f), TOPE_RUTAS)
                .map_err(|x| error(f, x.to_string()))?
                .ok_or_else(|| error(f, format!("mas de {TOPE_RUTAS} bytes")))?,
            false,
        ),
        None => Vec::new(),
    };
    let f = fuentes(e.git, e.dir, e.claude, &[]);
    let mut avisos = f.avisos.clone();
    let config = crate::config::de_base(e.git, e.dir).unwrap_or_else(|_| {
        avisos.push("config-de-base-no-leida".to_owned());
        crate::config::Config::suelo()
    });
    let (carril, _) = de_tarea(&t, &c);
    let enunciado = heuristica::Enunciado::de(&c);
    let r = heuristica::resolver(
        t.complejidad.as_deref(),
        &t.etiquetas,
        &t.tipo,
        &t.dependencias,
        &enunciado,
    );
    let cx = complejidad(&r);
    let n = carril.roles_diseno(r.roles, &t.tipo, &config);
    let ctx = Contexto {
        carril,
        complejidad: Some(cx),
        modelos_permitidos: &config.seleccion.modelos_permitidos,
        ronda: 1,
    };
    let doms_plan = dominios_de(Some((&t, &c)), &[], &config);
    let doms_rev = dominios_de(Some((&t, &c)), &rutas, &config);
    let sel = |rol, doms: &super::dominios::Dominios| {
        let entrada = super::Entrada {
            rutas: &rutas,
            dominios: doms,
            etiquetas: &t.etiquetas,
            roles_base: n,
        };
        super::seleccionar(rol, &entrada, &f.catalogo, &f.inventario, &ctx).map_err(|x| {
            Informe::nuevo(SELECCION, Resultado::Error)
                .diagnostico(Mensaje::nuevo(&txt::TABLA_ILEGIBLE).con("error", x))
        })
    };
    let d = sel(super::Rol::Diseno, &doms_plan)?;
    let im = sel(super::Rol::Implementacion, &doms_plan)?;
    let rv = sel(super::Rol::Revision, &doms_rev)?;
    for (rol, s) in [("diseno", &d), ("implementacion", &im), ("revision", &rv)] {
        avisos.extend(s.avisos.iter().map(|a| format!("{rol}:{a}")));
    }
    let nombres = |d: &super::dominios::Dominios| {
        let v: Vec<String> = d.iter().map(|(x, _)| x.clone()).collect();
        if v.is_empty() {
            "ninguno".to_owned()
        } else {
            v.join(",")
        }
    };
    let modelo = |s: &Seleccion| {
        if s.modelo.is_empty() {
            "-".to_owned()
        } else {
            s.modelo.clone()
        }
    };
    Ok(Informe::nuevo(SELECCION, Resultado::Ok)
        .con("id", t.id.as_str())
        .con("carril", carril.nombre())
        .con("complejidad", cx)
        .con("dominios", nombres(&doms_rev))
        .con("dominios-diseno", nombres(&doms_plan))
        .con_lista("senales", rv.senales.clone())
        .con_lista("diseno-agentes", d.roles.clone())
        .con("diseno-modelo", modelo(&d))
        .con_lista("diseno-reglas", d.reglas.clone())
        .con_lista("implementacion-skills", im.skills.clone())
        .con("implementacion-modelo", modelo(&im))
        .con_lista("implementacion-reglas", im.reglas.clone())
        .con_lista("revision-revisoras", rv.revisoras.clone())
        .con_lista("revision-agentes", rv.agentes.clone())
        .con("revision-modelo", modelo(&rv))
        .con_lista("revision-reglas", rv.reglas.clone())
        .con_lista("avisos", avisos))
}
