//! Confianza local (F1-8 decision 5): la unica interfaz que decide si `tc2`
//! puede ejecutar una orden de la configuracion del repo.
//!
//! `estado(repo, clave, orden, incluye) -> confiado | cambiado | nunca`, con
//! `clave` la ruta de la clave en la configuracion (`verificacion.comando`) y
//! la orden leida del arbol de trabajo. Generica por clave: no sabe de hooks
//! ni de perfiles.
//!
//! - **Huella**: sha256 de la orden, de los globs de `confianza_incluye` y
//!   del contenido de su cierre ([`cierre`]).
//! - **Almacen fuera del arbol**: `<cache>/taskcodetwo/confianza/<sha>.json`
//!   (`%LOCALAPPDATA%` en Windows, `$XDG_CACHE_HOME` o `~/.cache` en el
//!   resto), un fichero por repo con nombre el sha256 de la ruta canonica del
//!   directorio comun de Git: los worktrees comparten confianza y otro clon
//!   no. Nada del repo cuenta. Si el almacen se borra o no se lee, todo
//!   vuelve a `nunca` (falla cerrado).
//! - **Estados**: `nunca` (sin entrada), `confiado` (misma huella),
//!   `cambiado` (otra huella). Por contenido: volver al contenido confiado
//!   vuelve a dar `confiado`.

pub mod cierre;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::contrato::{Informe, Resultado};
use crate::git::{self, Git, Subcomando};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::verificacion as t;
use crate::instalacion;

pub use cierre::Fichero;

/// Nombre de la orden `tc2 confiar`.
pub const NOMBRE: &str = "confiar";

/// Claves de la configuracion cuya orden se puede confiar hoy.
pub const CLAVES: [&str; 1] = ["verificacion.comando"];

/// Version del fichero del almacen.
/// Version del fichero del almacen. La 2 (ronda 1 de TASK-046) cambia los
/// `candidatos` (comillas y no ASCII): un almacen de la 1 se lee vacio y todo
/// vuelve a `nunca` (falla cerrado).
const VERSION_ALMACEN: u32 = 2;

/// Estado de confianza de una orden.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Estado {
    /// Misma huella que la confiada.
    Confiado,
    /// Hay confianza para la clave, pero con otra huella.
    Cambiado,
    /// Sin confianza para la clave en este repo.
    Nunca,
}

impl Estado {
    /// Nombre en la linea de contrato.
    pub fn nombre(self) -> &'static str {
        match self {
            Self::Confiado => "confiado",
            Self::Cambiado => "cambiado",
            Self::Nunca => "nunca",
        }
    }
}

/// Lo que se sabe de una orden tras calcular su huella.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluacion {
    /// Estado.
    pub estado: Estado,
    /// Huella actual.
    pub huella: String,
    /// Ficheros del cierre.
    pub ficheros: usize,
    /// Bytes leidos para hashear (0 en caliente y sin cambios).
    pub hasheados: u64,
    /// Globs de `confianza_incluye` que no casan con ningun fichero.
    pub globs_vacios: Vec<String>,
}

/// Un repo abierto para la confianza: su arbol de trabajo, el nombre de su
/// fichero en el almacen y sus ficheros versionados.
pub struct Repo {
    /// Raiz del arbol de trabajo (`--show-toplevel`).
    pub raiz: PathBuf,
    /// sha256 de la ruta canonica del directorio comun de Git.
    pub id: String,
    indice: cierre::Indice,
}

/// Abre el repo de `dir`.
pub fn abrir(git: &Git, dir: &Path, op: &str) -> Result<Repo, Informe> {
    let raiz = git::exigir(
        git,
        dir,
        op,
        Subcomando::RevParse,
        &["--path-format=absolute", "--show-toplevel"],
    )?;
    let raiz = PathBuf::from(raiz.trim());
    let comun = git::estado::dir_comun(git, dir, op)?;
    let comun = std::fs::canonicalize(&comun).unwrap_or(comun);
    let ls = git::exigir(git, &raiz, op, Subcomando::LsFiles, &["-z"])?;
    Ok(Repo {
        raiz,
        id: cierre::sha_hex(comun.to_string_lossy().as_bytes()),
        indice: cierre::Indice::de_ls_files(&ls),
    })
}

/// Carpeta del almacen de confianza de esta maquina: la subcarpeta
/// `confianza` de la carpeta `taskcodetwo` de datos locales
/// (`%LOCALAPPDATA%\taskcodetwo` o `$XDG_CACHE_HOME/taskcodetwo`, la misma
/// que la cache del lanzador, F1-14).
pub fn almacen(e: &instalacion::Entorno) -> Option<PathBuf> {
    e.raices_de_cache().pop().map(|r| r.join("confianza"))
}

/// La orden de `clave` en `config`, tal como se hashea. En
/// `verificacion.comando` entra la shell: cambiarla cambia lo que se ejecuta.
pub fn orden_configurada(config: &Config, clave: &str) -> Option<String> {
    match clave {
        "verificacion.comando" => config
            .verificacion
            .as_ref()
            .map(|v| format!("{}\n{}", v.shell, v.comando)),
        _ => None,
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Almacen {
    version: u32,
    claves: BTreeMap<String, Entrada>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Entrada {
    huella: String,
    ficheros: Vec<Fichero>,
}

fn ruta_almacen(almacen: &Path, repo: &Repo) -> PathBuf {
    almacen.join(format!("{}.json", repo.id))
}

/// El almacen del repo; vacio si no existe o no se lee (falla cerrado).
fn leer_almacen(ruta: &Path) -> Almacen {
    std::fs::read_to_string(ruta)
        .ok()
        .and_then(|s| serde_json::from_str::<Almacen>(&s).ok())
        .filter(|a| a.version == VERSION_ALMACEN)
        .unwrap_or_default()
}

fn escribir_almacen(ruta: &Path, a: &Almacen, op: &str) -> Result<(), Informe> {
    let error = |e: &dyn std::fmt::Display| {
        Informe::nuevo(op, Resultado::Error).diagnostico(
            Mensaje::nuevo(&t::ALMACEN_NO_ESCRITO)
                .con("ruta", ruta.display().to_string())
                .con("error", e.to_string()),
        )
    };
    if let Some(padre) = ruta.parent() {
        std::fs::create_dir_all(padre).map_err(|e| error(&e))?;
    }
    let texto = serde_json::to_string_pretty(a).map_err(|e| error(&e))?;
    let tmp = ruta.with_extension(format!("json.{}.tmp", std::process::id()));
    std::fs::write(&tmp, texto).map_err(|e| error(&e))?;
    std::fs::rename(&tmp, ruta).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        error(&e)
    })
}

fn error_de_cierre(op: &str, e: &cierre::ErrorCierre) -> Informe {
    match e {
        cierre::ErrorCierre::Lectura(e) => Informe::nuevo(op, Resultado::Error)
            .diagnostico(Mensaje::nuevo(&t::CIERRE_NO_LEIDO).con("error", e.to_string())),
        cierre::ErrorCierre::Glob { glob, error } => Informe::nuevo(op, Resultado::Rechazada)
            .con("motivo", "glob-invalido")
            .diagnostico(
                Mensaje::nuevo(&t::GLOB_INVALIDO)
                    .con("glob", glob.as_str())
                    .con("error", error.as_str())
                    .con("ruta", crate::config::RUTA),
            ),
    }
}

/// Estado de confianza de `orden` (la de `clave`) en `repo`. En caliente
/// reutiliza el sha de los ficheros con el mismo `(tamano, mtime)`; si sigue
/// confiada y algun fichero se volvio a leer, refresca la cache.
pub fn estado(
    repo: &Repo,
    almacen: &Path,
    clave: &str,
    orden: &str,
    incluye: &[String],
    op: &str,
) -> Result<Evaluacion, Informe> {
    let ruta = ruta_almacen(almacen, repo);
    let mut a = leer_almacen(&ruta);
    let entrada = a.claves.get(clave).cloned();
    let c = cierre::calcular(
        &repo.raiz,
        &repo.indice,
        orden,
        incluye,
        entrada.as_ref().map(|e| e.ficheros.as_slice()),
    )
    .map_err(|e| error_de_cierre(op, &e))?;
    let huella = cierre::huella(orden, incluye, &c.ficheros);
    let estado = match &entrada {
        None => Estado::Nunca,
        Some(e) if e.huella == huella => Estado::Confiado,
        Some(_) => Estado::Cambiado,
    };
    if estado == Estado::Confiado && c.hasheados > 0 {
        // Solo la cache: misma huella. Si no se puede escribir, la proxima
        // vez se vuelve a leer, nada mas.
        a.claves.insert(
            clave.to_owned(),
            Entrada {
                huella: huella.clone(),
                ficheros: c.ficheros.clone(),
            },
        );
        let _ = escribir_almacen(&ruta, &a, op);
    }
    Ok(Evaluacion {
        estado,
        huella,
        ficheros: c.ficheros.len(),
        hasheados: c.hasheados,
        globs_vacios: c.globs_vacios.clone(),
    })
}

/// Confia en `orden` tal como esta ahora en el arbol: rehashea todo el
/// cierre, sin cache, y lo guarda en el almacen.
pub fn confiar(
    repo: &Repo,
    almacen: &Path,
    clave: &str,
    orden: &str,
    incluye: &[String],
    op: &str,
) -> Result<Evaluacion, Informe> {
    let ruta = ruta_almacen(almacen, repo);
    let mut a = leer_almacen(&ruta);
    a.version = VERSION_ALMACEN;
    let c = cierre::calcular(&repo.raiz, &repo.indice, orden, incluye, None)
        .map_err(|e| error_de_cierre(op, &e))?;
    let huella = cierre::huella(orden, incluye, &c.ficheros);
    a.claves.insert(
        clave.to_owned(),
        Entrada {
            huella: huella.clone(),
            ficheros: c.ficheros.clone(),
        },
    );
    escribir_almacen(&ruta, &a, op)?;
    Ok(Evaluacion {
        estado: Estado::Confiado,
        huella,
        ficheros: c.ficheros.len(),
        hasheados: c.hasheados,
        globs_vacios: c.globs_vacios.clone(),
    })
}

/// `tc2 confiar <clave>` en la carpeta actual: confia en la orden de `clave`
/// tal como esta en el arbol de trabajo y en lo que ejecuta.
pub fn orden(args: &[String]) -> Informe {
    let uso = || {
        Informe::nuevo(NOMBRE, Resultado::Uso)
            .diagnostico(Mensaje::nuevo(&t::USO_CONFIAR).con("claves", CLAVES.join(", ")))
    };
    let [clave] = args else {
        return uso().con("motivo", "argumentos");
    };
    if clave.starts_with('-') {
        return uso().con("motivo", "argumentos");
    }
    if !CLAVES.contains(&clave.as_str()) {
        return Informe::nuevo(NOMBRE, Resultado::Uso)
            .con("motivo", "clave-desconocida")
            .diagnostico(
                Mensaje::nuevo(&t::CLAVE_DESCONOCIDA)
                    .con("clave", clave.as_str())
                    .con("claves", CLAVES.join(", ")),
            );
    }
    match confiar_en(clave) {
        Ok(i) | Err(i) => i,
    }
}

fn confiar_en(clave: &str) -> Result<Informe, Informe> {
    let dir = std::env::current_dir().map_err(|e| {
        Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(
            Mensaje::nuevo(&crate::i18n::catalogo::cli::CARPETA_ACTUAL).con("error", e.to_string()),
        )
    })?;
    let almacen = almacen_del_proceso(NOMBRE)?;
    let repo = abrir(&Git::nuevo(), &dir, NOMBRE)?;
    let config = crate::config::del_arbol(&repo.raiz).map_err(|mut i| {
        i.comando = NOMBRE.to_owned();
        i
    })?;
    let Some(orden) = orden_configurada(&config, clave) else {
        return Err(Informe::nuevo(NOMBRE, Resultado::Rechazada)
            .con("motivo", "sin-orden")
            .con("clave", clave)
            .diagnostico(
                Mensaje::nuevo(&t::SIN_ORDEN)
                    .con("ruta", crate::config::RUTA)
                    .con("clave", clave),
            ));
    };
    let ev = confiar(
        &repo,
        &almacen,
        clave,
        &orden,
        &config.confianza_incluye,
        NOMBRE,
    )?;
    let i = Informe::nuevo(NOMBRE, Resultado::Ok)
        .con("clave", clave)
        .con("huella", crate::contrato::corto(&ev.huella))
        .con("ficheros", ev.ficheros.to_string())
        .con("hasheados", ev.hasheados.to_string());
    Ok(if ev.globs_vacios.is_empty() {
        i
    } else {
        i.con_lista("globs-sin-ficheros", ev.globs_vacios)
    })
}

/// Aviso para la persona (stderr, codigo 0) si `tc2 confiar` salio bien con
/// globs de `confianza_incluye` que no casan con nada: lo que declaro no
/// entra en la confianza.
pub fn aviso_globs(i: &Informe, idioma: crate::i18n::Idioma) -> Option<String> {
    let globs = i.claves.iter().find_map(|(k, v)| match v {
        crate::contrato::Valor::Lista(l) if k == "globs-sin-ficheros" => Some(l.join(", ")),
        _ => None,
    })?;
    Some(
        Mensaje::nuevo(&t::GLOBS_SIN_FICHEROS)
            .con("globs", globs)
            .con("ruta", crate::config::RUTA)
            .texto(idioma),
    )
}

/// El almacen del entorno de este proceso, o el informe que dice que hacer.
pub fn almacen_del_proceso(op: &str) -> Result<PathBuf, Informe> {
    instalacion::Entorno::del_proceso()
        .ok()
        .as_ref()
        .and_then(almacen)
        .ok_or_else(|| {
            Informe::nuevo(op, Resultado::Error)
                .con("motivo", "sin-almacen")
                .diagnostico(&t::SIN_ALMACEN)
        })
}
