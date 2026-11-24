//! Nucleo del CLI `tc2` de TaskCodeTwo.
//!
//! Modulos: [`contrato`] (salida de toda operacion, ADR F1-4), [`git`]
//! (capa de operaciones Git deterministas, F2-2) y [`estado`] (rama de
//! estado `taskcodetwo/tareas`, F2-3), [`alta`] (alta de tareas con reserva de
//! ID, F2-4) [`tarea`] (modelo de tarea y frontmatter YAML, F2-5) y
//! [`plataforma`] (puerto Plataforma y adaptadores GitHub y GitLab, F2-6 y
//! F2-9), [`carril`] (tabla de suelo de F1-10) y [`config`] (configuracion
//! con esquema validado, F2-7) e [`i18n`] (mensajes en espanol e ingles,
//! F2-8), [`version`] (`version --json` y `compatible`, F2-12) y
//! [`actualizar_cli`] (`actualizar --cli`, F2-12) e [`identidad`] (quien
//! ejecuta y `tc2 identidad`, F2-13) y [`huella`] (los nombres que la 2.0
//! deja en un repo, F1-12 D1a) e [`init`] (deja un repo listo, F3-1) e
//! [`instalacion`] (donde vive el `tc2` de la
//! maquina, F2-12 y F2-14) y [`desinstalar`] (quitar TaskCodeTwo, F2-14) e
//! [`importar`] (alta en bloque con dependencias e hito, F3-2) y [`grafo`] y
//! [`siguiente`] (orden de trabajo y `next`, F3-3) y [`situacion`] y
//! [`roadmap`] (el calculo comun y `roadmap`, F3-4) y [`tablero`]
//! (`board` y `status`, F3-5) y [`toma`] (`start`, F3-6) y [`solape`]
//! (aviso de solape de `start`, F3-11) y [`ciclo`]
//! (`descartar`, `pausa` y `reanudar`, F3-7; `liberar` y `reasignar`,
//! F3-10) y [`doctor`] (F3-8) y [`plan`] (`plan` y `plan guardar`, F4-1;
//! `approve`, F4-2) y [`revision`] (`review`, F6-2) y [`seleccion`]
//! (`revisoras`, revisoras y modelo de F1-15, F6-3) y [`hallazgos`]
//! (`hallazgos`, hallazgos con evidencia, F6-4). Las capas
//! que vienen entran como modulos de esta biblioteca.

pub mod actualizar_cli;
pub mod alta;
pub mod carril;
pub mod ciclo;
pub mod confianza;
pub mod config;
pub mod contrato;
pub mod dato;
pub mod descarga;
pub mod desinstalar;
pub mod doctor;
pub mod estado;
pub mod estimacion;
pub mod git;
pub mod grafo;
pub mod hallazgos;
pub mod huella;
pub mod i18n;
pub mod identidad;
pub mod importar;
pub mod init;
pub mod instalacion;
pub mod inventario;
mod lectura;
pub mod plan;
pub mod plataforma;
pub mod progreso;
pub mod recursos;
pub mod revision;
pub mod roadmap;
pub mod seleccion;
pub mod siguiente;
pub mod situacion;
pub mod solape;
pub mod tablero;
pub mod tarea;
pub mod toma;
pub mod verificacion;
pub mod version;
mod yaml;

pub use version::PROTOCOLO;

use contrato::{Informe, Resultado};
use i18n::catalogo::cli as txt;
use i18n::{Idioma, Mensaje};

/// Version del CLI, la de `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Resultado de interpretar la linea de ordenes: que escribir y con que
/// codigo salir. Sin efectos, para poder probarlo sin lanzar procesos.
#[derive(Debug, PartialEq, Eq)]
pub struct Salida {
    /// Codigo de salida del proceso.
    pub codigo: u8,
    /// Texto para la salida estandar (vacio si no hay).
    pub stdout: String,
    /// Texto para la salida de error (vacio si no hay).
    pub stderr: String,
}

/// Interpreta los argumentos (sin el nombre del programa) y ejecuta la orden
/// en la carpeta actual, con los mensajes en espanol.
pub fn ejecutar<I, S>(args: I) -> Salida
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    ejecutar_en(Idioma::Es, args)
}

/// Como [`ejecutar`], con los mensajes dirigidos a la persona en `idioma`
/// (F2-8). stdout y el codigo de salida no cambian con el idioma.
pub fn ejecutar_en<I, S>(idioma: Idioma, args: I) -> Salida
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args: Vec<String> = args.into_iter().map(|a| a.as_ref().to_owned()).collect();
    match args.as_slice() {
        // F3-12: la status line, lo primero y sin identidad, catalogo ni git.
        [a, b] if a == progreso::ESTADO && b == "--stdin" => progreso::estado_del_proceso(),
        [a] if a == "--version" || a == "-V" => Salida {
            codigo: 0,
            stdout: format!(
                "tc2 {VERSION}
"
            ),
            stderr: String::new(),
        },
        [orden, resto @ ..] if orden == "guardar" || orden == "actualizar-rama" => {
            let (json, resto) = match resto {
                [j, r @ ..] if j == "--json" => (true, r),
                r => (false, r),
            };
            orden_git(orden, resto).salida_en(json, idioma)
        }
        [orden, resto @ ..] if orden == alta::NOMBRE => {
            let (json, resto) = match resto {
                [j, r @ ..] if j == "--json" => (true, r),
                r => (false, r),
            };
            let mut parecidas = Vec::new();
            let s = orden_new(resto, &mut parecidas).salida_en(json, idioma);
            con_aviso(s, &parecidas, idioma)
        }
        [orden, resto @ ..] if orden == importar::NOMBRE => {
            let (json, resto) = con_json(resto);
            let mut parecidas = Vec::new();
            let s = importar::orden(&resto, &catalogo, &mut parecidas).salida_en(json, idioma);
            con_aviso(s, &parecidas, idioma)
        }
        [orden, resto @ ..] if orden == siguiente::NOMBRE => {
            let (json, resto) = con_json(resto);
            siguiente::orden(&resto, json, idioma)
        }
        [orden, resto @ ..] if orden == roadmap::NOMBRE => {
            let (json, resto) = con_json(resto);
            roadmap::orden(&resto, json, idioma, &catalogo)
        }
        [orden, resto @ ..] if orden == tablero::STATUS || orden == tablero::BOARD => {
            let (json, resto) = con_json(resto);
            orden_tablero(orden, &resto, json, idioma)
        }
        [orden, resto @ ..] if orden == version::VERSION => {
            let (json, resto) = con_json(resto);
            orden_version(json, &resto, idioma)
        }
        [orden, resto @ ..] if orden == version::COMPATIBLE => {
            let (json, resto) = con_json(resto);
            version::compatible(&resto).salida_en(json, idioma)
        }
        [orden, resto @ ..] if orden == actualizar_cli::NOMBRE => {
            let (json, resto) = con_json(resto);
            actualizar_cli::orden(&resto).salida_en(json, idioma)
        }
        [orden, resto @ ..] if orden == desinstalar::NOMBRE => {
            let (json, resto) = con_json(resto);
            desinstalar::orden(&resto, json, idioma)
        }
        [orden, resto @ ..] if orden == init::NOMBRE => {
            let (json, resto) = con_json(resto);
            init::orden(&resto, json, idioma)
        }
        [orden, resto @ ..] if orden == doctor::NOMBRE => {
            let (json, resto) = con_json(resto);
            orden_doctor(&resto, json, idioma)
        }
        [orden, resto @ ..] if orden == toma::NOMBRE => {
            let (json, resto) = con_json(resto);
            let mut solape = None;
            let s = de_una_tarea(toma::NOMBRE, (json, idioma), |e| {
                let (i, s) = toma::orden_con_solape(&resto, e);
                solape = s;
                i
            });
            con_aviso_de_solape(s, solape.as_ref(), idioma)
        }
        [orden, resto @ ..] if orden == ciclo::DESCARTAR => {
            let (json, resto) = con_json(resto);
            de_una_tarea(ciclo::DESCARTAR, (json, idioma), |e| {
                ciclo::orden_descartar(&resto, e)
            })
        }
        [orden, resto @ ..] if orden == ciclo::PAUSA || orden == ciclo::REANUDAR => {
            let (json, resto) = con_json(resto);
            let reanudar = orden == ciclo::REANUDAR;
            de_una_tarea(orden, (json, idioma), |e| {
                ciclo::orden_pausa(&resto, reanudar, e)
            })
        }
        [orden, resto @ ..] if orden == ciclo::LIBERAR => {
            let (json, resto) = con_json(resto);
            de_una_tarea(ciclo::LIBERAR, (json, idioma), |e| {
                ciclo::orden_liberar(&resto, e)
            })
        }
        [orden, resto @ ..] if orden == ciclo::REASIGNAR => {
            let (json, resto) = con_json(resto);
            de_una_tarea(ciclo::REASIGNAR, (json, idioma), |e| {
                ciclo::orden_reasignar(&resto, e)
            })
        }
        [orden, resto @ ..] if orden == plan::NOMBRE => {
            let (json, resto) = con_json(resto);
            de_una_tarea(plan::NOMBRE, (json, idioma), |e| plan::orden(&resto, e))
        }
        [orden, resto @ ..] if orden == plan::aprobar::NOMBRE => {
            let (json, resto) = con_json(resto);
            de_una_tarea(plan::aprobar::NOMBRE, (json, idioma), |e| {
                plan::orden_approve(&resto, e)
            })
        }
        [orden, resto @ ..] if orden == revision::NOMBRE => {
            let (json, resto) = con_json(resto);
            de_una_tarea(revision::NOMBRE, (json, idioma), |e| {
                orden_review(&resto, e)
            })
        }
        [orden, resto @ ..] if orden == seleccion::orden::NOMBRE => {
            let (json, resto) = con_json(resto);
            orden_revisoras(&resto).salida_en(json, idioma)
        }
        [orden, resto @ ..] if orden == seleccion::orden::SELECCION => {
            let (json, resto) = con_json(resto);
            orden_seleccion(&resto).salida_en(json, idioma)
        }
        [orden, resto @ ..] if orden == hallazgos::orden::NOMBRE => {
            let (json, resto) = con_json(resto);
            hallazgos::orden::orden(&resto, json, idioma)
        }
        [orden, resto @ ..] if orden == estimacion::NOMBRE => {
            let (json, resto) = con_json(resto);
            orden_estimar(&resto, json, idioma)
        }
        [orden, resto @ ..] if orden == verificacion::ejecutar::NOMBRE => {
            let (json, resto) = con_json(resto);
            verificacion::ejecutar::orden(&resto, json, idioma)
        }
        [orden, resto @ ..] if orden == confianza::NOMBRE => {
            let (json, resto) = con_json(resto);
            let i = confianza::orden(&resto);
            let mut s = i.salida_en(json, idioma);
            if s.codigo == 0
                && let Some(a) = confianza::aviso_globs(&i, idioma)
            {
                s.stderr.push_str(&a);
                s.stderr.push('\n');
            }
            s
        }
        [orden, resto @ ..] if orden == identidad::NOMBRE => {
            let (json, resto) = con_json(resto);
            orden_identidad(&resto).salida_en(json, idioma)
        }
        [] => Salida {
            codigo: 2,
            stdout: String::new(),
            stderr: format!(
                "{}
",
                Mensaje::nuevo(&txt::FALTA_LA_ORDEN)
                    .con_mensaje("uso", &txt::USO)
                    .texto(idioma)
            ),
        },
        [otro, ..] => Salida {
            codigo: 2,
            stdout: String::new(),
            stderr: format!(
                "{}
",
                Mensaje::nuevo(&txt::ORDEN_DESCONOCIDA)
                    .con("orden", otro.as_str())
                    .con_mensaje("uso", &txt::USO)
                    .texto(idioma)
            ),
        },
    }
}

/// El aviso de titulos parecidos va a stderr con codigo 0: es la unica
/// excepcion a «diagnostico solo si falla» (F2-4b, D1).
fn con_aviso(mut s: Salida, parecidas: &[alta::Parecida], idioma: Idioma) -> Salida {
    if s.codigo == 0
        && let Some(a) = alta::aviso_parecidas(parecidas, idioma)
    {
        s.stderr.push_str(&a);
        s.stderr.push('\n');
    }
    s
}

/// El aviso de solape de `start` (F3-11) va a stderr con codigo 0, como el
/// de titulos parecidos: solo para la persona; el canal fiable son las
/// claves `solape*` de la linea y del JSON. Nunca cambia codigo ni stdout.
fn con_aviso_de_solape(mut s: Salida, solape: Option<&solape::Solape>, idioma: Idioma) -> Salida {
    if s.codigo != 0 {
        return s;
    }
    for a in solape.map(solape::Solape::avisos).unwrap_or_default() {
        s.stderr.push_str(&a.texto(idioma));
        s.stderr.push('\n');
    }
    s
}

/// Quita `--json` de donde este y dice si estaba.
fn con_json(args: &[String]) -> (bool, Vec<String>) {
    let json = args.iter().any(|a| a == "--json");
    (
        json,
        args.iter().filter(|a| *a != "--json").cloned().collect(),
    )
}

/// `version [--json]`: con `--json`, exactamente `{version, protocolo,
/// target, sha256}` (ADR F1-14, punto 5); sin el, la linea de contrato.
fn orden_version(json: bool, args: &[String], idioma: Idioma) -> Salida {
    if !args.is_empty() {
        return Informe::nuevo(version::VERSION, Resultado::Uso)
            .con("motivo", "argumentos-de-mas")
            .diagnostico(&i18n::catalogo::actualizar::USO_VERSION)
            .salida_en(json, idioma);
    }
    let datos = std::env::current_exe()
        .map_err(|e| {
            Informe::nuevo(version::VERSION, Resultado::Error).diagnostico(
                Mensaje::nuevo(&i18n::catalogo::actualizar::NO_SE_LEE_EL_EJECUTABLE)
                    .con("ruta", "current_exe")
                    .con("error", e.to_string()),
            )
        })
        .and_then(|exe| version::datos(&exe));
    match datos {
        Ok(d) if json => Salida {
            codigo: 0,
            stdout: format!("{}\n", d.json()),
            stderr: String::new(),
        },
        Ok(d) => version::informe(&d).salida_en(false, idioma),
        Err(i) => i.salida_en(json, idioma),
    }
}

/// Las dos operaciones Git publicas (ADR F1-4 D1a), en la carpeta actual.
fn orden_git(orden: &str, args: &[String]) -> Informe {
    let dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            return Informe::nuevo(orden, Resultado::Error)
                .diagnostico(Mensaje::nuevo(&txt::CARPETA_ACTUAL).con("error", e.to_string()));
        }
    };
    let g = git::Git::nuevo();
    if orden == "guardar" {
        let Some((mensaje, rutas)) = args.split_first() else {
            return Informe::nuevo(orden, Resultado::Uso)
                .con("motivo", "sin-mensaje")
                .diagnostico(&txt::USO_GUARDAR);
        };
        let rutas: Vec<&str> = rutas.iter().map(String::as_str).collect();
        git::guardar(&g, &dir, mensaje, &rutas)
    } else if args.is_empty() {
        git::actualizar_rama(&g, &dir, &git::ESPERAS_PUSH)
    } else {
        Informe::nuevo(orden, Resultado::Uso)
            .con("motivo", "argumentos-de-mas")
            .diagnostico(&txt::USO_ACTUALIZAR_RAMA)
    }
}

/// `identidad [--json]` (F2-13) en la carpeta actual, con el terminal y los
/// tokens del proceso.
fn orden_identidad(args: &[String]) -> Informe {
    use std::io::IsTerminal;
    let dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            return Informe::nuevo(identidad::NOMBRE, Resultado::Error)
                .diagnostico(Mensaje::nuevo(&txt::CARPETA_ACTUAL).con("error", e.to_string()));
        }
    };
    let g = git::Git::nuevo();
    identidad::orden(
        args,
        identidad::Entorno {
            dir: &dir,
            git: &g,
            tty: std::io::stdin().is_terminal(),
            credenciales: plataforma::de_entorno(),
            ahora: identidad::ahora(),
        },
    )
}

/// Una orden sobre una tarea (`start`, F3-6; `descartar`, `pausa` y
/// `reanudar`, F3-7; `liberar` y `reasignar`, F3-10; `plan`, F4-1;
/// `approve`, F4-2) en la carpeta actual:
/// quien la lanza sale de
/// [`identidad::resolver`] con el terminal y los tokens del proceso.
///
/// Si la transicion sale bien, refresca el progreso en vivo de la tarea
/// (F3-12): [`progreso::refrescar`], o [`progreso::quitar`] tras
/// `descartar`, `liberar` y `reasignar`. Un fallo ahi no cambia la
/// transicion: la linea gana `progreso=no-escrito` y stderr un aviso, con el
/// mismo codigo.
fn de_una_tarea(
    nombre: &str,
    (json, idioma): (bool, Idioma),
    orden: impl FnOnce(&toma::Entorno<'_>) -> Informe,
) -> Salida {
    let dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            return Informe::nuevo(nombre, Resultado::Error)
                .diagnostico(Mensaje::nuevo(&txt::CARPETA_ACTUAL).con("error", e.to_string()))
                .salida_en(json, idioma);
        }
    };
    let g = git::Git::nuevo();
    let ahora = identidad::ahora();
    let i = transicion(&dir, &g, ahora, orden);
    // `start` que publico la toma (`estado=`) ya escribio el progreso con lo
    // que calculo dentro (un solo `git log` y una sola lectura de todos los
    // `tarea.md`, R1-05 y R1-06 de TASK-083).
    let hecho = nombre == toma::NOMBRE && i.valor("estado").is_some();
    con_progreso(i, |id| {
        if hecho {
            Ok(progreso::Refresco::Escrito)
        } else if [ciclo::DESCARTAR, ciclo::LIBERAR, ciclo::REASIGNAR].contains(&nombre) {
            progreso::quitar(&g, &dir, id)
        } else {
            progreso::refrescar(&g, &dir, id, ahora)
        }
    })
    .salida(json, idioma)
}

/// El informe de una transicion y, si fallo el refresco del progreso, el
/// aviso para stderr.
struct ConProgreso {
    informe: Informe,
    aviso: Option<Mensaje>,
}

impl ConProgreso {
    fn salida(self, json: bool, idioma: Idioma) -> Salida {
        let mut s = self.informe.salida_en(json, idioma);
        if let Some(a) = self.aviso {
            s.stderr.push_str(&a.texto(idioma));
            s.stderr.push('\n');
        }
        s
    }
}

/// Refresca el progreso de la tarea (`id=` de la linea) si la transicion
/// salio bien; un fallo de `refrescar` no toca el resultado. Si la propia
/// transicion ya dice `progreso=no-escrito` (`start`), solo el aviso.
fn con_progreso(
    i: Informe,
    refrescar: impl FnOnce(&str) -> Result<progreso::Refresco, String>,
) -> ConProgreso {
    let aviso = |id: &str, e: &str| {
        Some(
            Mensaje::nuevo(&i18n::catalogo::progreso::AVISO_NO_ESCRITO)
                .con("id", id)
                .con("error", tablero::sanear(e, 200)),
        )
    };
    let id = match i.valor("id") {
        Some(id) if i.resultado.es_exito() => id.to_owned(),
        _ => {
            return ConProgreso {
                informe: i,
                aviso: None,
            };
        }
    };
    if i.valor(progreso::CLAVE) == Some(progreso::NO_ESCRITO) {
        return ConProgreso {
            aviso: aviso(&id, "escritura"),
            informe: i,
        };
    }
    match refrescar(&id) {
        Ok(progreso::Refresco::EscritoSinAnterior) => ConProgreso {
            informe: i.con(progreso::CLAVE, progreso::SIN_ANTERIOR),
            aviso: None,
        },
        Ok(_) => ConProgreso {
            informe: i,
            aviso: None,
        },
        Err(e) => ConProgreso {
            informe: i.con(progreso::CLAVE, progreso::NO_ESCRITO),
            aviso: aviso(&id, &e),
        },
    }
}

/// La transicion en `dir`, con quien la lanza resuelto por
/// [`identidad::resolver`].
fn transicion(
    dir: &std::path::Path,
    g: &git::Git,
    ahora: u64,
    orden: impl FnOnce(&toma::Entorno<'_>) -> Informe,
) -> Informe {
    use std::io::IsTerminal;
    let quien = || {
        identidad::resolver(identidad::Entorno {
            dir,
            git: g,
            tty: std::io::stdin().is_terminal(),
            credenciales: plataforma::de_entorno(),
            ahora,
        })
    };
    let e = toma::Entorno {
        dir,
        git: g,
        quien: &quien,
        catalogo: &catalogo,
        ahora,
    };
    orden(&e)
}

/// `review [--json] TASK-NNN` (F6-2): el entorno de `start` mas el
/// adaptador de la plataforma de `origin` (la deteccion de `start`, la lista
/// de personas de la base y los tokens o la App del agente del proceso);
/// `review` le pide siempre con el rol de agente.
fn orden_review(args: &[String], e: &toma::Entorno<'_>) -> Informe {
    let conectar = || -> plataforma::Paso<Box<dyn plataforma::Plataforma>> {
        let destino = identidad::destino_de(e.git, e.dir)?;
        let personas = identidad::personas_de_base(e.git, e.dir)?;
        identidad::adaptador(destino, plataforma::de_entorno(), personas)
    };
    let claude = dir_claude();
    revision::orden(
        args,
        &revision::Entorno {
            base: e,
            conectar: &conectar,
            esperas: &git::ESPERAS_PUSH,
            claude: claude.as_deref(),
        },
    )
}

/// La carpeta de Claude Code (`CLAUDE_CONFIG_DIR` o `~/.claude`), solo para
/// leer el inventario (F3-9, F6-3).
pub(crate) fn dir_claude() -> Option<std::path::PathBuf> {
    instalacion::Entorno::de_variables(
        instalacion::Sistema::actual(),
        std::path::PathBuf::new(),
        std::process::id(),
        |k| std::env::var(k).ok(),
    )
    .dir_claude()
}

/// `revisoras [--json] ...` (F6-3) en la carpeta actual: la entrada
/// estandar para `--rutas -`.
fn orden_revisoras(args: &[String]) -> Informe {
    let dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            return Informe::nuevo(seleccion::orden::NOMBRE, Resultado::Error)
                .diagnostico(Mensaje::nuevo(&txt::CARPETA_ACTUAL).con("error", e.to_string()));
        }
    };
    let g = git::Git::nuevo();
    let claude = dir_claude();
    // Un byte de mas que el tope: la orden falla si llega, como con fichero.
    let stdin = || {
        use std::io::Read;
        let mut b = Vec::new();
        std::io::stdin()
            .take(seleccion::orden::TOPE_RUTAS + 1)
            .read_to_end(&mut b)
            .map(|_| b)
    };
    seleccion::orden::orden(
        args,
        &seleccion::orden::Entorno {
            dir: &dir,
            git: &g,
            claude: claude.as_deref(),
            stdin: &stdin,
        },
    )
}

/// `seleccion [--json] ...` (F4-6) en la carpeta actual.
fn orden_seleccion(args: &[String]) -> Informe {
    let dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            return Informe::nuevo(seleccion::orden::SELECCION, Resultado::Error)
                .diagnostico(Mensaje::nuevo(&txt::CARPETA_ACTUAL).con("error", e.to_string()));
        }
    };
    let g = git::Git::nuevo();
    let claude = dir_claude();
    let stdin = || Ok(Vec::new());
    seleccion::orden::orden_seleccion(
        args,
        &seleccion::orden::Entorno {
            dir: &dir,
            git: &g,
            claude: claude.as_deref(),
            stdin: &stdin,
        },
    )
}

/// `estimar [--json] [TASK-NNN | --error TASK-NNN]` (F3-13) en la carpeta
/// actual. Sin `--json` y con exito, la linea para la persona va detras de
/// la de contrato (como mucho 5 lineas, F1-4 decision 4).
fn orden_estimar(args: &[String], json: bool, idioma: Idioma) -> Salida {
    let dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            return Informe::nuevo(estimacion::NOMBRE, Resultado::Error)
                .diagnostico(Mensaje::nuevo(&txt::CARPETA_ACTUAL).con("error", e.to_string()))
                .salida_en(json, idioma);
        }
    };
    let g = git::Git::nuevo();
    let e = estimacion::Entorno {
        dir: &dir,
        git: &g,
        catalogo: &catalogo,
        ahora: identidad::ahora(),
    };
    let (i, linea) = estimacion::orden(args, &e);
    let mut s = i.salida_en(json, idioma);
    if let (false, true, Some(l)) = (json, i.resultado.es_exito(), linea) {
        s.stdout.push_str(&l);
        s.stdout.push('\n');
    }
    s
}

/// `status` o `board` (F3-5) en la carpeta actual.
fn orden_tablero(orden: &str, args: &[String], json: bool, idioma: Idioma) -> Salida {
    let dir = match tablero::carpeta_actual(orden) {
        Ok(d) => d,
        Err(i) => return i.salida_en(json, idioma),
    };
    let g = git::Git::nuevo();
    let conectar = tablero::conectar_produccion(&g, &dir);
    let e = tablero::Entorno {
        dir: &dir,
        git: &g,
        conectar: &conectar,
    };
    if orden == tablero::STATUS {
        tablero::orden_status(args, json, idioma, &e)
    } else {
        tablero::orden_board(args, json, idioma, &e)
    }
}

/// `doctor [--json]` (F3-8) en la carpeta actual, con la plataforma de
/// produccion (la misma deteccion y los mismos tokens que `board`).
fn orden_doctor(args: &[String], json: bool, idioma: Idioma) -> Salida {
    let dir = match tablero::carpeta_actual(doctor::NOMBRE) {
        Ok(d) => d,
        Err(i) => return i.salida_en(json, idioma),
    };
    let g = git::Git::nuevo();
    let conectar = tablero::conectar_produccion(&g, &dir);
    let claude = dir_claude();
    let e = doctor::Entorno {
        dir: &dir,
        git: &g,
        conectar: &conectar,
        claude: claude.as_deref(),
    };
    doctor::orden(args, json, idioma, &e)
}

/// Catalogo de produccion: reconstruye las operaciones de la rama de estado
/// desde su anotacion. Las operaciones nuevas se anaden aqui.
fn catalogo(anotacion: &str) -> Option<Box<dyn estado::Operacion>> {
    alta::desde_anotacion(anotacion)
        .or_else(|| roadmap::desde_anotacion(anotacion))
        .or_else(|| toma::desde_anotacion(anotacion))
        .or_else(|| ciclo::desde_anotacion(anotacion))
        .or_else(|| revision::desde_anotacion(anotacion))
}

/// `new "<titulo>" [campos de RF-01]`: alta de una tarea con su ID reservado
/// por el push de la rama de estado (F2-4). Sale `new ok id=TASK-NNN
/// rama=... [parecidas=...]`; las tareas abiertas parecidas quedan ademas en
/// `parecidas` para el aviso. Un enunciado que no esta listo o una
/// dependencia que no existe es `uso` sin escribir nada (TASK-029).
fn orden_new(args: &[String], parecidas: &mut Vec<alta::Parecida>) -> Informe {
    let entrada = match alta::entrada_de_args(args) {
        Ok(e) => e,
        Err(i) => return i,
    };
    let dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            return Informe::nuevo(alta::NOMBRE, Resultado::Error)
                .diagnostico(Mensaje::nuevo(&txt::CARPETA_ACTUAL).con("error", e.to_string()));
        }
    };
    let g = git::Git::nuevo();
    // Las ramas `*/task-*` se leen una vez, antes de la transaccion: el ID
    // nuevo queda por encima de todas (F2-4b). Sin red no se reserva nada.
    let ocupados = match git::ids_con_rama(&g, &dir, alta::NOMBRE) {
        Ok(o) => o,
        Err(mut i) => {
            alta::NOMBRE.clone_into(&mut i.comando);
            if i.resultado == Resultado::Sinred {
                i = i.con("pendiente", "no");
            }
            return i;
        }
    };
    // La historia de `develop` para la estimacion (F3-13), tambien antes.
    let op = alta::Alta::con_entrada(entrada)
        .con_ocupados(ocupados)
        .con_commits(estimacion::commits_de(&g, &dir));
    let personas = roadmap::personas(&g, &dir);
    let generar = roadmap::generador(&personas);
    let c = estado::Contexto::nuevo(&g, &dir, &catalogo, Some(&generar));
    let mut i = estado::transaccion(&c, &op);
    alta::NOMBRE.clone_into(&mut i.comando);
    alta::referencia_a_uso(&mut i);
    if i.resultado == Resultado::Ok
        && let (Some(id), Some(rama)) = (op.id(), op.rama())
    {
        i.claves
            .insert(0, ("id".to_owned(), contrato::Valor::Texto(id)));
        i.claves
            .insert(1, ("rama".to_owned(), contrato::Valor::Texto(rama)));
        *parecidas = op.parecidas();
        if !parecidas.is_empty() {
            let ids = parecidas.iter().map(|p| p.id.clone()).collect();
            i.claves
                .insert(2, ("parecidas".to_owned(), contrato::Valor::Lista(ids)));
        }
        i = estimacion::con_claves(i, op.estimacion().as_ref());
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sin_titulo_o_con_salto_de_linea_es_uso() {
        for (args, motivo) in [
            (vec!["new"], "sin-titulo"),
            (vec!["new", "--json"], "sin-titulo"),
            (vec!["new", "a", "b"], "argumentos-de-mas"),
            (vec!["new", "a\tb"], "titulo-invalido"),
            (vec!["new", "linea\notra"], "titulo-invalido"),
            (vec!["new", "  "], "titulo-invalido"),
        ] {
            let s = ejecutar(&args);
            assert_eq!(s.codigo, 2, "{args:?}");
            assert!(s.stdout.contains(motivo), "{args:?}: {}", s.stdout);
        }
    }

    #[test]
    fn version_corta_y_larga() {
        for a in ["--version", "-V"] {
            let s = ejecutar([a]);
            assert_eq!(s.codigo, 0);
            assert_eq!(s.stdout, format!("tc2 {VERSION}\n"));
        }
    }

    /// TASK-027, criterio 4: `version --json` da exactamente las cuatro
    /// claves del ADR F1-14 (punto 5), con el sha256 del ejecutable.
    #[test]
    fn version_json_exactamente_version_protocolo_target_sha256() {
        let s = ejecutar(["version", "--json"]);
        assert_eq!((s.codigo, s.stderr.as_str()), (0, ""));
        let v: serde_json::Value = serde_json::from_str(&s.stdout).unwrap_or_default();
        let claves: Vec<&str> = v
            .as_object()
            .map(|o| o.keys().map(String::as_str).collect())
            .unwrap_or_default();
        let mut ordenadas = claves.clone();
        ordenadas.sort_unstable();
        assert_eq!(ordenadas, ["protocolo", "sha256", "target", "version"]);
        assert_eq!(v["version"], VERSION);
        assert_eq!(v["protocolo"], PROTOCOLO);
        let alias = distribucion::Target::actual().map(|t| t.alias);
        assert_eq!(v["target"].as_str(), alias);
        let exe = std::env::current_exe().unwrap_or_default();
        let bytes = std::fs::read(exe).unwrap_or_default();
        assert_eq!(v["sha256"], distribucion::sha256::hex(&bytes));
        assert_eq!(s.stdout.lines().count(), 1);
        // Sin --json, la linea de contrato con las mismas claves.
        let l = ejecutar(["version"]);
        assert_eq!(l.codigo, 0);
        assert!(l.stdout.starts_with(&format!(
            "version ok version={VERSION} protocolo={PROTOCOLO} target="
        )));
        assert_eq!(ejecutar(["version", "de-mas"]).codigo, 2);
    }

    /// TASK-027, criterio 5: un rango que no incluye el protocolo sale
    /// `rechazada` (3) con el motivo y el arreglo, en los dos idiomas.
    #[test]
    fn compatible_fuera_de_rango_es_rechazada_con_el_arreglo() {
        let fuera = format!("{}-{}", PROTOCOLO + 1, PROTOCOLO + 2);
        let es = ejecutar_en(Idioma::Es, ["compatible", "--protocolo", &fuera]);
        let en = ejecutar_en(Idioma::En, ["compatible", "--protocolo", &fuera]);
        for s in [&es, &en] {
            assert_eq!(s.codigo, 3);
            assert_eq!(
                s.stdout,
                format!(
                    "compatible rechazada protocolo={PROTOCOLO} rango={fuera} motivo=protocolo-incompatible arreglo=tc2%20actualizar%20--cli\n"
                )
            );
            assert!(
                s.stderr
                    .contains("motivo=protocolo-incompatible arreglo=\"tc2 actualizar --cli\""),
                "{}",
                s.stderr
            );
            assert_eq!(s.stderr.lines().count(), 1);
        }
        assert!(es.stderr.starts_with("este tc2 habla el protocolo"));
        assert!(en.stderr.starts_with("this tc2 speaks protocol"));
        // En JSON el arreglo va tal cual.
        let j = ejecutar_en(Idioma::En, ["compatible", "--json", "--protocolo", &fuera]);
        assert_eq!(j.codigo, 3);
        assert!(j.stdout.contains("\"arreglo\":\"tc2 actualizar --cli\""));
        // Dentro del rango, ok; mal escrito, uso.
        let dentro = format!("1-{PROTOCOLO}");
        let ok = ejecutar(["compatible", "--protocolo", &dentro]);
        assert_eq!(
            (ok.codigo, ok.stdout),
            (
                0,
                format!("compatible ok protocolo={PROTOCOLO} rango={dentro}\n")
            )
        );
        for args in [
            vec!["compatible"],
            vec!["compatible", "--protocolo"],
            vec!["compatible", "--protocolo", "3-1"],
            vec!["compatible", "--protocolo", "x"],
        ] {
            assert_eq!(ejecutar(&args).codigo, 2, "{args:?}");
        }
    }

    /// F3-6: el catalogo de produccion reconstruye la toma desde su
    /// anotacion, igual que el alta y el roadmap.
    #[test]
    fn el_catalogo_reconstruye_la_toma() {
        let op = catalogo("start|TASK-001|7|ana");
        assert_eq!(
            op.map(|o| (o.nombre().to_owned(), o.anotacion())),
            Some(("start".to_owned(), "start|TASK-001|7|ana".to_owned()))
        );
        assert!(catalogo("start|TASK-001|7").is_none());
    }

    /// F3-7: el catalogo reconstruye descartar, pausa y reanudar (una pausa
    /// pendiente sin red se re-ejecuta desde aqui, TASK-034 C8).
    #[test]
    fn el_catalogo_reconstruye_descartar_pausa_y_reanudar() {
        for a in [
            "descartar|TASK-001|7|ana|ya no | hace falta",
            "pausa|TASK-001|7|ana|1794910830",
            "reanudar|TASK-001|7|ana|1794910830",
        ] {
            let op = catalogo(a);
            assert_eq!(op.map(|o| o.anotacion()), Some(a.to_owned()), "{a}");
        }
        assert!(catalogo("pausa|TASK-001|7|ana").is_none());
    }

    /// TASK-033 C5: un ID mal formado (o sin ID, o de mas) es `uso` (2) sin
    /// tocar nada, con el diagnostico en los dos idiomas.
    #[test]
    fn start_con_id_mal_formado_es_uso() {
        for (args, motivo) in [
            (vec!["start"], "sin-id"),
            (vec!["start", "TASK-1"], "id-invalido"),
            (vec!["start", "task-001"], "id-invalido"),
            (vec!["start", "TASK-001a"], "id-invalido"),
            (
                vec!["start", "--json", "TASK-001", "x"],
                "argumentos-de-mas",
            ),
        ] {
            let es = ejecutar_en(Idioma::Es, &args);
            let en = ejecutar_en(Idioma::En, &args);
            assert_eq!((es.codigo, en.codigo), (2, 2), "{args:?}");
            assert!(es.stdout.contains(motivo), "{args:?}: {}", es.stdout);
            assert_eq!(es.stderr, "uso: start [--json] TASK-NNN\n");
            assert_eq!(en.stderr, "usage: start [--json] TASK-NNN\n");
        }
        assert_eq!(
            ejecutar(["start", "TASK-1"]).stdout,
            "start uso motivo=id-invalido\n"
        );
    }

    /// TASK-034 C11 y C2: `descartar`, `pausa` y `reanudar` con un ID mal
    /// formado, sin ID o con argumentos de mas son `uso` (2) como `start`;
    /// `descartar` sin motivo o con uno invalido, tambien. Nada se lee: no
    /// hay repo en la carpeta de los tests de la biblioteca que importe.
    #[test]
    fn descartar_pausa_y_reanudar_mal_llamados_son_uso() {
        let largo = "m".repeat(ciclo::MOTIVO_MAX + 1);
        for (args, motivo, es, en) in [
            (
                vec!["descartar"],
                "sin-id",
                "uso: descartar",
                "usage: descartar",
            ),
            (
                vec!["descartar", "TASK-1", "m"],
                "id-invalido",
                "uso: descartar",
                "usage: descartar",
            ),
            (
                vec!["descartar", "TASK-001"],
                "sin-motivo",
                "uso: descartar",
                "usage: descartar",
            ),
            (
                vec!["descartar", "TASK-001", ""],
                "motivo-invalido",
                "el motivo",
                "the reason",
            ),
            (
                vec!["descartar", "TASK-001", "  "],
                "motivo-invalido",
                "el motivo",
                "the reason",
            ),
            (
                vec!["descartar", "TASK-001", "a\nb"],
                "motivo-invalido",
                "el motivo",
                "the reason",
            ),
            (
                vec!["descartar", "TASK-001", "a\tb"],
                "motivo-invalido",
                "el motivo",
                "the reason",
            ),
            (
                vec!["descartar", "TASK-001", &largo],
                "motivo-invalido",
                "el motivo",
                "the reason",
            ),
            (
                vec!["descartar", "--json", "TASK-001", "m", "x"],
                "argumentos-de-mas",
                "uso: descartar",
                "usage: descartar",
            ),
            (vec!["pausa"], "sin-id", "uso: pausa", "usage: pausa"),
            (
                vec!["pausa", "task-001"],
                "id-invalido",
                "uso: pausa",
                "usage: pausa",
            ),
            (
                vec!["pausa", "TASK-001", "x"],
                "argumentos-de-mas",
                "uso: pausa",
                "usage: pausa",
            ),
            (
                vec!["reanudar"],
                "sin-id",
                "uso: reanudar",
                "usage: reanudar",
            ),
            (
                vec!["reanudar", "TASK-001a1"],
                "id-invalido",
                "uso: reanudar",
                "usage: reanudar",
            ),
            (
                vec!["reanudar", "TASK-001", "x"],
                "argumentos-de-mas",
                "uso: reanudar",
                "usage: reanudar",
            ),
        ] {
            let s_es = ejecutar_en(Idioma::Es, &args);
            let s_en = ejecutar_en(Idioma::En, &args);
            assert_eq!((s_es.codigo, s_en.codigo), (2, 2), "{args:?}");
            let json = args.contains(&"--json");
            if !json {
                assert_eq!(
                    s_es.stdout,
                    format!("{} uso motivo={motivo}\n", args[0]),
                    "{args:?}"
                );
                assert_eq!(s_es.stdout, s_en.stdout);
                assert!(s_es.stderr.starts_with(es), "{args:?}: {}", s_es.stderr);
                assert!(s_en.stderr.starts_with(en), "{args:?}: {}", s_en.stderr);
            } else {
                assert_eq!(s_es.stdout.lines().count(), 1);
                assert!(
                    s_es.stdout.contains(&format!("\"motivo\":\"{motivo}\"")),
                    "{}",
                    s_es.stdout
                );
            }
        }
    }

    /// F3-12: un fallo al refrescar el progreso nunca cambia el resultado de
    /// la transicion (mismo codigo; la linea gana `progreso=no-escrito` y
    /// stderr un aviso); una transicion que no sale bien no lo refresca.
    #[test]
    fn el_progreso_no_cambia_la_transicion() {
        let ok = Informe::nuevo("start", Resultado::Ok).con("id", "TASK-001");
        let s =
            con_progreso(ok.clone(), |_| Err("disco lleno".to_owned())).salida(false, Idioma::Es);
        assert_eq!(s.codigo, 0);
        assert_eq!(s.stdout, "start ok id=TASK-001 progreso=no-escrito\n");
        assert!(
            s.stderr
                .starts_with("aviso: no se pudo escribir el progreso de TASK-001 (disco lleno)"),
            "{}",
            s.stderr
        );
        let s =
            con_progreso(ok.clone(), |_| Ok(progreso::Refresco::Escrito)).salida(false, Idioma::Es);
        assert_eq!(
            (s.codigo, s.stdout.as_str(), s.stderr.as_str()),
            (0, "start ok id=TASK-001\n", "")
        );
        // TASK-128: sin rotar el anterior, la linea lo dice y stderr no.
        let s = con_progreso(ok, |_| Ok(progreso::Refresco::EscritoSinAnterior))
            .salida(false, Idioma::Es);
        assert_eq!(
            (s.codigo, s.stdout.as_str(), s.stderr.as_str()),
            (0, "start ok id=TASK-001 progreso=sin-anterior\n", "")
        );
        let mut llamado = false;
        let mal = Informe::nuevo("pausa", Resultado::Rechazada).con("id", "TASK-001");
        let s = con_progreso(mal, |_| {
            llamado = true;
            Ok(progreso::Refresco::Escrito)
        })
        .salida(false, Idioma::Es);
        assert_eq!((s.codigo, llamado), (3, false));
        // `start` que ya lo intento: solo el aviso, sin volver a refrescar.
        let ya = Informe::nuevo("start", Resultado::Ok)
            .con("id", "TASK-001")
            .con("progreso", "no-escrito");
        let mut llamado = false;
        let s = con_progreso(ya, |_| {
            llamado = true;
            Ok(progreso::Refresco::Escrito)
        })
        .salida(false, Idioma::Es);
        assert_eq!((s.codigo, llamado), (0, false));
        assert_eq!(s.stdout, "start ok id=TASK-001 progreso=no-escrito\n");
        assert_eq!(s.stderr.lines().count(), 1, "{}", s.stderr);
        let sin_cambios = Informe::nuevo("pausa", Resultado::SinCambios).con("id", "TASK-001");
        let s = con_progreso(sin_cambios, |_| Err("x".to_owned())).salida(true, Idioma::En);
        assert_eq!(s.codigo, 0);
        assert!(
            s.stdout.contains("\"progreso\":\"no-escrito\""),
            "{}",
            s.stdout
        );
        assert!(s.stderr.starts_with("warning: "), "{}", s.stderr);
    }

    /// TASK-036: `plan` y `plan guardar` mal llamados son `uso` (2) sin
    /// leer nada, con el diagnostico en los dos idiomas.
    #[test]
    fn plan_mal_llamado_es_uso() {
        for (args, motivo) in [
            (vec!["plan"], "sin-id"),
            (vec!["plan", "TASK-1"], "id-invalido"),
            (vec!["plan", "TASK-001", "--carril"], "carril-invalido"),
            (
                vec!["plan", "TASK-001", "--carril", "release"],
                "carril-invalido",
            ),
            (
                vec!["plan", "TASK-001", "--carril", "fix", "--carril", "fix"],
                "argumentos-de-mas",
            ),
            (vec!["plan", "TASK-001", "x"], "argumentos-de-mas"),
            (vec!["plan", "guardar"], "sin-id"),
            (vec!["plan", "guardar", "TASK-001"], "sin-destino"),
            (
                vec!["plan", "guardar", "TASK-001", "plan-final.md"],
                "sin-fichero",
            ),
            (
                vec!["plan", "guardar", "TASK-001", "tarea.md", "f"],
                "destino-invalido",
            ),
            (
                vec!["plan", "guardar", "TASK-001", "plan-final.md", "f", "x"],
                "argumentos-de-mas",
            ),
        ] {
            let es = ejecutar_en(Idioma::Es, &args);
            let en = ejecutar_en(Idioma::En, &args);
            assert_eq!((es.codigo, en.codigo), (2, 2), "{args:?}");
            assert_eq!(es.stdout, format!("plan uso motivo={motivo}\n"), "{args:?}");
            assert_eq!(es.stdout, en.stdout);
            assert_ne!(es.stderr, en.stderr, "{args:?}");
            assert_eq!(es.stderr.lines().count(), 1, "{args:?}");
        }
    }

    /// TASK-037: `approve` mal llamado es `uso` (2) sin leer nada;
    /// `--decidido-por automatico` no existe en la 2.0 (F1-10 decision 6).
    #[test]
    fn approve_mal_llamado_es_uso() {
        for (args, motivo) in [
            (vec!["approve"], "sin-id"),
            (vec!["approve", "TASK-1"], "id-invalido"),
            (vec!["approve", "TASK-001", "x"], "argumentos-de-mas"),
            (
                vec!["approve", "TASK-001", "--pedir-cambios"],
                "sin-fichero",
            ),
            (
                vec![
                    "approve",
                    "TASK-001",
                    "--pedir-cambios",
                    "a",
                    "--pedir-cambios",
                    "b",
                ],
                "argumentos-de-mas",
            ),
            (
                vec!["approve", "TASK-001", "--decidido-por", "automatico"],
                "decidido-por",
            ),
            (
                vec!["approve", "TASK-001", "--decidido-por=automatico"],
                "decidido-por",
            ),
        ] {
            let es = ejecutar_en(Idioma::Es, &args);
            let en = ejecutar_en(Idioma::En, &args);
            assert_eq!((es.codigo, en.codigo), (2, 2), "{args:?}");
            assert_eq!(
                es.stdout,
                format!("approve uso motivo={motivo}\n"),
                "{args:?}"
            );
            assert_eq!(es.stdout, en.stdout);
            assert_ne!(es.stderr, en.stderr, "{args:?}");
            assert_eq!(es.stderr.lines().count(), 1, "{args:?}");
        }
        let s = ejecutar_en(
            Idioma::Es,
            ["approve", "TASK-001", "--decidido-por", "automatico"],
        );
        assert!(s.stderr.contains("decision 6"), "{}", s.stderr);
    }

    #[test]
    fn sin_orden_falla() {
        assert_eq!(ejecutar(Vec::<String>::new()).codigo, 2);
    }

    #[test]
    fn guardar_sin_mensaje_o_sin_rutas_es_uso() {
        let s = ejecutar(["guardar"]);
        assert_eq!(
            (s.codigo, s.stdout.as_str()),
            (
                2,
                "guardar uso motivo=sin-mensaje
"
            )
        );
        let s = ejecutar(["guardar", "--json", "mensaje"]);
        assert_eq!(s.codigo, 2);
        assert!(
            s.stdout
                .starts_with("{\"comando\":\"guardar\",\"resultado\":\"uso\"")
        );
        let s = ejecutar(["actualizar-rama", "de-mas"]);
        assert_eq!(
            s.stdout,
            "actualizar-rama uso motivo=argumentos-de-mas
"
        );
    }

    #[test]
    fn new_y_actualizar_rama_en_ingles_mismo_stdout_y_codigo() {
        for args in [
            vec!["new"],
            vec!["new", "a\tb"],
            vec!["new", "--json", "a", "b"],
            vec!["actualizar-rama", "de-mas"],
            vec!["guardar"],
        ] {
            let es = ejecutar_en(Idioma::Es, &args);
            let en = ejecutar_en(Idioma::En, &args);
            assert_eq!(es.codigo, en.codigo, "{args:?}");
            assert!(!en.stderr.is_empty(), "{args:?}");
            assert_ne!(es.stderr, en.stderr, "{args:?}: stderr sin traducir");
            if args.contains(&"--json") {
                assert!(en.stdout.contains("\"diagnostico\":"), "{args:?}");
            } else {
                assert_eq!(es.stdout, en.stdout, "{args:?}");
            }
        }
        assert_eq!(
            ejecutar_en(Idioma::En, ["new"]).stderr,
            "usage: new [--json] \"<title>\"\n"
        );
    }
}
