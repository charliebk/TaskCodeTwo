//! `tc2 revisoras [--json] --rutas <fichero|-> [--tarea TASK-NNN]
//! [--carril <c>] [--complejidad <x>] [--plugin <nombre>=<dir>]...
//! [--informe <fichero> --seccion <salida> [--artefacto <n>]]
//! [--detalle <salida.json>]` (F6-3).
//!
//! Las rutas son dato (una por linea; los dos nombres de cada renombrado):
//! en CI salen de `git diff --name-only --no-renames base...head` sobre el
//! checkout de la base; en local, de [`crate::git::ficheros_de_rama`]. La
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
    /// Lee la entrada estandar (para `--rutas -`).
    pub stdin: &'a dyn Fn() -> std::io::Result<String>,
}

/// Inventario y catalogo con los que se selecciona, y los avisos de
/// leerlos.
#[derive(Debug, Clone, Default)]
pub struct Fuentes {
    /// Lo instalado.
    pub inventario: Inventario,
    /// Las revisoras con metadatos.
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
    let indice = |n: &str| heuristica::NIVELES.iter().position(|x| *x == n);
    let cx = r
        .declarado
        .as_deref()
        .and_then(indice)
        .into_iter()
        .chain(indice(r.heuristico))
        .max()
        .and_then(|i| heuristica::NIVELES.get(i).copied())
        .unwrap_or(r.heuristico);
    (carril, cx)
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
    tarea: Option<String>,
    carril: Option<Carril>,
    complejidad: Option<String>,
    plugins: Vec<(String, PathBuf)>,
    informe: Option<PathBuf>,
    artefacto: Option<String>,
    seccion: Option<PathBuf>,
    detalle: Option<PathBuf>,
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
        Some("-") => (e.stdin)().map_err(|x| fallo_lectura("-", x.to_string()))?,
        Some(f) => leer_con_tope(Path::new(f), TOPE_RUTAS)
            .map_err(|x| fallo_lectura(f, x.to_string()))?
            .ok_or_else(|| fallo_lectura(f, format!("mas de {TOPE_RUTAS} bytes")))?,
        None => return Err(uso("sin-rutas")),
    };
    let rutas: Vec<String> = texto
        .lines()
        .map(|l| l.trim_end_matches('\r'))
        .filter(|l| !l.trim().is_empty())
        .map(str::to_owned)
        .collect();
    let f = fuentes(e.git, e.dir, e.claude, &a.plugins);
    let mut avisos = f.avisos.clone();
    let mut carril = Carril::Funcionalidad;
    let mut cx: Option<String> = None;
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
    let config = crate::config::de_base(e.git, e.dir).unwrap_or_else(|_| {
        avisos.push("config-de-base-no-leida".to_owned());
        crate::config::Config::suelo()
    });
    let ctx = Contexto {
        carril,
        complejidad: cx.as_deref(),
        modelos_permitidos: &config.seleccion.modelos_permitidos,
        ronda: 1,
    };
    let mut s = super::revision(&rutas, &f.catalogo, &f.inventario, &ctx).map_err(|x| {
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
