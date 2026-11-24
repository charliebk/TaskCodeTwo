//! `tc2 hallazgos [--json] validar --respuesta <md> --diff <txt>
//! --revisora <id> [--informe <json>] [--salida <json>] [--resumen <md>]` y
//! `tc2 hallazgos [--json] instrucciones` (F6-4).
//!
//! `validar` lee la respuesta de una revisora (con un tope de
//! [`TOPE_RESPUESTA`] bytes), el diff que vio (tope [`TOPE_DIFF`]) y, si se
//! da, el informe de verificacion de F5-4 (tope [`TOPE_INFORME`]; si no se
//! lee, `paso_rojo` no se acepta), y escribe la linea `hallazgos ok
//! revisora=<id> aceptados=N rechazados=M motivos=<quien:motivo,...>
//! [informe=leido|sin-informe|grande|invalido]`. Con `--salida`, el JSON del
//! esquema `hallazgos.schema.json`; con `--resumen`, la seccion en Markdown
//! para el check `revision-ia`. Que haya rechazos no es un fallo de la
//! orden: sale `ok` y el rojo lo decide F6-5.
//!
//! `instrucciones` escribe en stdout el texto que va al prompt
//! ([`super::instrucciones`]); con `--json`, en la clave `instrucciones` del
//! contrato.
//!
//! `veredicto --salida <dir> [--hallazgos <json>]... [--extra <txt>]` (F6-5)
//! junta las salidas de `validar` de todas las revisoras ([`super::veredicto`];
//! una que no se lee, pasa de [`TOPE_VALIDADOS`] o no es del contrato cuenta
//! como revisora sin respuesta valida) y escribe en `<dir>` el `check.json`
//! que publica el workflow (`conclusion`, `output.title`, `output.summary` y
//! las primeras 50 anotaciones) y un `anotaciones-<k>.json` por cada lote
//! siguiente. `--extra` son lineas que van al final del resumen (la
//! seleccion y el estado de cada revisora). Linea: `hallazgos ok
//! veredicto=<rojo|verde|incompleta> conclusion=<success|failure>
//! criticos=N importantes=M menores=K revisoras=R incompletas=I lotes=L`.
//! Un veredicto rojo no es un fallo de la orden: sale `ok`.
//!
//! F6-6 (rondas incrementales):
//! - `anterior --check-runs <json>... [--origen <id>]`: de los volcados de
//!   check-runs de los commits del PR (del mas nuevo al mas viejo, sin el
//!   head), el check que sirve de ronda anterior; la App del revisor sale de
//!   `--origen` o de `plataforma.revisor_id` de la configuracion de la base.
//!   Linea: `hallazgos ok anterior=<check-id>|ninguno [sha=<sha>]
//!   avisos=<...>`.
//! - `ronda ... --salida <dir>`: con las anotaciones de ese check y los
//!   ficheros cambiados desde su commit, escribe `ronda.json`
//!   (`$defs/ronda`). Linea: `hallazgos ok ronda=R modo=<m>
//!   anterior=<sha12|ninguno> abiertos=N tope=T avisos=<...>`.
//! - `abiertos --ronda <json> --detalle <json> --salida <dir>`: los abiertos
//!   CRITICO e IMPORTANTE de cada revisora (`abiertos-<i>.json` y `.md`).
//! - `desde --anterior <sha> --head <sha> --base <ref>`: desde donde se
//!   saca el diff incremental, sin lo que entro de la base por un merge
//!   ([`desde_incremental`]). Linea: `hallazgos ok desde=<oid>|base
//!   motivo=<...>`.
//! - `validar --abiertos <json>` valida tambien los cierres; `veredicto
//!   --ronda <json>` cuenta nuevos, persisten y cerrados (la linea gana
//!   `ronda=R modo=<m> nuevos=N persisten=P cerrados=C`).
//!
//! F6-7 (prompt injection):
//! - `validar --prosa <md>`: la prosa del modelo para el check, inerte
//!   ([`super::prosa`]); el workflow la concatena en vez de la respuesta.
//! - `prompt --revisora <id> --contexto <txt> [--skill <md>] [--seccion <md>]
//!   [--abiertos <md>] --diff <txt> --bytes <n> --tope <n> --salida <md>`:
//!   escribe el prompt de la revisora ([`super::prompt::componer`]); el
//!   workflow ya no lo arma. Linea: `hallazgos ok revisora=<id> bytes=<n>
//!   cortado=<si|no> abiertos=<si|no>`.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::Salida;
use crate::contrato::{Informe, Resultado};
use crate::i18n::catalogo::hallazgos as txt;
use crate::i18n::{Idioma, Mensaje};
use crate::verificacion::informe;

/// Nombre de la orden.
pub const NOMBRE: &str = "hallazgos";

/// Bytes que se leen como mucho de la respuesta del modelo (el workflow la
/// corta antes a 64 KiB).
pub const TOPE_RESPUESTA: u64 = 256 * 1024;

/// Bytes que se leen como mucho del diff.
pub const TOPE_DIFF: u64 = 4 * 1024 * 1024;

/// Bytes que se leen como mucho del informe de verificacion (el mismo tope
/// que `tc2 revisoras`).
pub const TOPE_INFORME: u64 = crate::seleccion::orden::TOPE_INFORME;

/// Bytes que se leen como mucho de cada salida de `validar` (F6-5).
pub const TOPE_VALIDADOS: u64 = 4 * 1024 * 1024;

/// Bytes que se leen como mucho de `--extra`.
pub const TOPE_EXTRA: u64 = 64 * 1024;

struct Args {
    respuesta: Option<PathBuf>,
    diff: Option<PathBuf>,
    revisora: Option<String>,
    informe: Option<PathBuf>,
    salida: Option<PathBuf>,
    resumen: Option<PathBuf>,
    abiertos: Option<PathBuf>,
    prosa: Option<PathBuf>,
}

fn uso(motivo: &str) -> Informe {
    Informe::nuevo(NOMBRE, Resultado::Uso)
        .con("motivo", motivo)
        .diagnostico(&txt::USO_HALLAZGOS)
}

/// Un id de revisora: el de `tc2 revisoras` (`plugin:skill`), sin espacios
/// ni comas, para que la linea no cambie de forma.
fn es_revisora(t: &str) -> bool {
    !t.is_empty()
        && t.len() <= 200
        && t.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '.' | '_' | '-' | '/'))
}

/// Las opciones de una suborden, por nombre, con sus valores en orden.
struct Opciones(BTreeMap<&'static str, Vec<String>>);

impl Opciones {
    /// El valor de una opcion que va como mucho una vez.
    fn una(&self, nombre: &str) -> Option<String> {
        self.0.get(nombre).and_then(|v| v.first()).cloned()
    }

    /// El valor de una opcion unica, como ruta.
    fn ruta(&self, nombre: &str) -> Option<PathBuf> {
        self.una(nombre).map(PathBuf::from)
    }

    /// Los valores de una opcion repetible, como rutas, en el orden dado.
    fn rutas(&self, nombre: &str) -> Vec<PathBuf> {
        self.0
            .get(nombre)
            .map(|v| v.iter().map(PathBuf::from).collect())
            .unwrap_or_default()
    }
}

/// Analiza `--opcion valor` de una suborden: cada opcion lleva un valor; las
/// de `unicas` van como mucho una vez y las de `repetibles`, las que se
/// quiera. Una opcion que no es de ninguna, o una unica repetida (antes de
/// leer su valor), es `argumento-desconocido`; una sin valor al final,
/// `falta-valor`. Que el valor valga lo mira cada suborden despues.
fn opciones(
    a: &[String],
    unicas: &[&'static str],
    repetibles: &[&'static str],
) -> Result<Opciones, Informe> {
    let mut o: BTreeMap<&'static str, Vec<String>> = BTreeMap::new();
    let mut it = a.iter();
    while let Some(x) = it.next() {
        let Some(nombre) = unicas.iter().chain(repetibles).find(|n| **n == x.as_str()) else {
            return Err(uso("argumento-desconocido"));
        };
        let valores = o.entry(nombre).or_default();
        if unicas.contains(nombre) && !valores.is_empty() {
            return Err(uso("argumento-desconocido"));
        }
        valores.push(it.next().cloned().ok_or_else(|| uso("falta-valor"))?);
    }
    Ok(Opciones(o))
}

/// `Err(motivo)` de uso si el valor de `nombre` esta y no cumple `vale`.
fn comprobar(
    o: &Opciones,
    nombre: &str,
    vale: fn(&str) -> bool,
    motivo: &str,
) -> Result<(), Informe> {
    match o.una(nombre) {
        Some(v) if !vale(&v) => Err(uso(motivo)),
        _ => Ok(()),
    }
}

fn args(a: &[String]) -> Result<Args, Informe> {
    let o = opciones(
        a,
        &[
            "--respuesta",
            "--diff",
            "--revisora",
            "--informe",
            "--salida",
            "--resumen",
            "--abiertos",
            "--prosa",
        ],
        &[],
    )?;
    comprobar(&o, "--revisora", es_revisora, "revisora-invalida")?;
    let r = Args {
        respuesta: o.ruta("--respuesta"),
        diff: o.ruta("--diff"),
        revisora: o.una("--revisora"),
        informe: o.ruta("--informe"),
        salida: o.ruta("--salida"),
        resumen: o.ruta("--resumen"),
        abiertos: o.ruta("--abiertos"),
        prosa: o.ruta("--prosa"),
    };
    for (falta, motivo) in [
        (r.respuesta.is_none(), "sin-respuesta"),
        (r.diff.is_none(), "sin-diff"),
        (r.revisora.is_none(), "sin-revisora"),
    ] {
        if falta {
            return Err(uso(motivo));
        }
    }
    Ok(r)
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

fn fallo_io(p: &Path, error: String) -> Informe {
    Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(
        Mensaje::nuevo(&txt::NO_SE_LEE)
            .con("ruta", p.display().to_string())
            .con("error", error),
    )
}

fn leer_entrada(p: &Path, tope: u64) -> Result<String, Informe> {
    leer_con_tope(p, tope)
        .map_err(|e| fallo_io(p, e.to_string()))?
        .ok_or_else(|| {
            Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(
                Mensaje::nuevo(&txt::DEMASIADO_GRANDE)
                    .con("ruta", p.display().to_string())
                    .con("tope", tope.to_string()),
            )
        })
}

/// `hallazgos ...` (sin `--json`, que quita `lib.rs`).
pub fn orden(a: &[String], json: bool, idioma: Idioma) -> Salida {
    match a {
        [s] if s == "instrucciones" => {
            let texto = super::instrucciones();
            if json {
                Informe::nuevo(NOMBRE, Resultado::Ok)
                    .con("instrucciones", texto)
                    .salida_en(true, idioma)
            } else {
                Salida {
                    codigo: 0,
                    stdout: texto,
                    stderr: String::new(),
                }
            }
        }
        [s, ..] if s == "instrucciones" => uso("argumentos-de-mas").salida_en(json, idioma),
        [s, resto @ ..] if s == "validar" => match validar(resto) {
            Ok(i) | Err(i) => i.salida_en(json, idioma),
        },
        [s, resto @ ..] if s == "veredicto" => match veredicto(resto) {
            Ok(i) | Err(i) => i.salida_en(json, idioma),
        },
        [s, resto @ ..] if s == "anterior" => match anterior(resto) {
            Ok(i) | Err(i) => i.salida_en(json, idioma),
        },
        [s, resto @ ..] if s == "ronda" => match ronda(resto) {
            Ok(i) | Err(i) => i.salida_en(json, idioma),
        },
        [s, resto @ ..] if s == "abiertos" => match abiertos(resto) {
            Ok(i) | Err(i) => i.salida_en(json, idioma),
        },
        [s, resto @ ..] if s == "desde" => match desde(resto) {
            Ok(i) | Err(i) => i.salida_en(json, idioma),
        },
        [s, resto @ ..] if s == "prompt" => match prompt(resto) {
            Ok(i) | Err(i) => i.salida_en(json, idioma),
        },
        [] => uso("sin-suborden").salida_en(json, idioma),
        _ => uso("suborden-desconocida").salida_en(json, idioma),
    }
}

fn validar(a: &[String]) -> Result<Informe, Informe> {
    let a = args(a)?;
    let (Some(pr), Some(pd), Some(revisora)) = (&a.respuesta, &a.diff, &a.revisora) else {
        return Err(uso("sin-respuesta"));
    };
    let respuesta = leer_entrada(pr, TOPE_RESPUESTA)?;
    let diff = super::Diff::leer(&leer_entrada(pd, TOPE_DIFF)?);
    let (estado, inf) = match &a.informe {
        None => (None, None),
        Some(p) => match leer_con_tope(p, TOPE_INFORME) {
            Err(_) => (Some("sin-informe"), None),
            Ok(None) => (Some("grande"), None),
            Ok(Some(t)) => match informe::leer(&t) {
                Ok(i) => (Some("leido"), Some(i)),
                Err(_) => (Some("invalido"), None),
            },
        },
    };
    let abiertos: Vec<super::AceptadoDe> = match &a.abiertos {
        Some(p) => leer_json(p, "una lista de abiertos de tc2 hallazgos abiertos")?,
        None => Vec::new(),
    };
    let v = super::validar_con(revisora, &respuesta, &diff, inf.as_ref(), &abiertos);
    if let Some(p) = &a.salida {
        let texto = serde_json::to_string_pretty(&v).map_err(|e| fallo_io(p, e.to_string()))?;
        std::fs::write(p, texto + "\n").map_err(|e| fallo_io(p, e.to_string()))?;
    }
    if let Some(p) = &a.resumen {
        std::fs::write(p, super::resumen_markdown(&v)).map_err(|e| fallo_io(p, e.to_string()))?;
    }
    if let Some(p) = &a.prosa {
        std::fs::write(p, super::prosa(revisora, &respuesta))
            .map_err(|e| fallo_io(p, e.to_string()))?;
    }
    let mut i = Informe::nuevo(NOMBRE, Resultado::Ok)
        .con("revisora", revisora.as_str())
        .con("aceptados", v.aceptados.len().to_string())
        .con("rechazados", v.rechazados.len().to_string())
        .con_lista("motivos", v.motivos());
    if a.abiertos.is_some() {
        i = i.con("cierres", v.cierres.len().to_string());
    }
    if let Some(e) = estado {
        i = i.con("informe", e);
    }
    Ok(i)
}

/// Una salida de `validar` del contrato vigente, o `None`.
fn leer_validados(p: &Path) -> Option<super::Validados> {
    let t = leer_con_tope(p, TOPE_VALIDADOS).ok()??;
    serde_json::from_str::<super::Validados>(&t)
        .ok()
        .filter(|v| v.version == super::VERSION)
}

fn escribir_json(p: &Path, v: &serde_json::Value) -> Result<(), Informe> {
    let texto = serde_json::to_string_pretty(v).map_err(|e| fallo_io(p, e.to_string()))?;
    std::fs::write(p, texto + "\n").map_err(|e| fallo_io(p, e.to_string()))
}

fn veredicto(a: &[String]) -> Result<Informe, Informe> {
    let o = opciones(a, &["--salida", "--extra", "--ronda"], &["--hallazgos"])?;
    let (extra, decision, entradas) =
        (o.ruta("--extra"), o.ruta("--ronda"), o.rutas("--hallazgos"));
    let Some(dir) = o.ruta("--salida") else {
        return Err(uso("sin-salida"));
    };
    let extra = match &extra {
        Some(p) => leer_entrada(p, TOPE_EXTRA)?,
        None => String::new(),
    };
    let revisoras: Vec<Option<super::Validados>> =
        entradas.iter().map(|p| leer_validados(p)).collect();
    let decision = decision.as_deref().map(leer_decision).transpose()?;
    let v = super::veredicto_ronda(&revisoras, decision.as_ref());
    let lotes = super::anotaciones(&v);
    let (titulo, resumen) = (v.titulo(), super::resumen_veredicto(&v, &extra));
    std::fs::create_dir_all(&dir).map_err(|e| fallo_io(&dir, e.to_string()))?;
    let salida_de = |anotaciones: &[super::Anotacion]| {
        serde_json::json!({
            "title": titulo,
            "summary": resumen,
            "annotations": anotaciones,
        })
    };
    let primero = lotes.first().map(Vec::as_slice).unwrap_or_default();
    escribir_json(
        &dir.join("check.json"),
        &serde_json::json!({
            "conclusion": v.dictamen.conclusion(),
            "output": salida_de(primero),
        }),
    )?;
    for (k, lote) in lotes.iter().enumerate().skip(1) {
        escribir_json(
            &dir.join(format!("anotaciones-{k}.json")),
            &serde_json::json!({ "output": salida_de(lote) }),
        )?;
    }
    Ok(Informe::nuevo(NOMBRE, Resultado::Ok)
        .con("veredicto", v.dictamen.texto())
        .con("conclusion", v.dictamen.conclusion())
        .con("criticos", v.criticos.to_string())
        .con("importantes", v.importantes.to_string())
        .con("menores", v.menores.to_string())
        .con("revisoras", v.filas.len().to_string())
        .con("incompletas", v.incompletas().to_string())
        .con("lotes", lotes.len().max(1).to_string())
        .con("ronda", v.ronda.to_string())
        .con("modo", v.modo.texto())
        .con("nuevos", v.nuevos.to_string())
        .con("persisten", v.persisten.to_string())
        .con("cerrados", v.cerrados.to_string()))
}

// --- Rondas (F6-6) ------------------------------------------------------------

/// Bytes que se leen como mucho de cada volcado de check-runs, pagina de
/// anotaciones, `ronda.json`, abiertos o detalle (F6-6).
pub const TOPE_JSON: u64 = 4 * 1024 * 1024;

fn no_es(p: &Path, que: &str, error: String) -> Informe {
    Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(
        Mensaje::nuevo(&txt::NO_ES_DEL_CONTRATO)
            .con("ruta", p.display().to_string())
            .con("que", que)
            .con("error", error),
    )
}

/// Lee `p` (con tope) como `T`; si no, el error dice que se esperaba `que`.
fn leer_json<T: serde::de::DeserializeOwned>(p: &Path, que: &str) -> Result<T, Informe> {
    let texto = leer_entrada(p, TOPE_JSON)?;
    serde_json::from_str(&texto).map_err(|e| no_es(p, que, e.to_string()))
}

/// Un volcado de `GET .../commits/<sha>/check-runs`.
#[derive(serde::Deserialize)]
struct PaginaChecks {
    check_runs: Vec<super::ronda::CheckRun>,
}

/// Una anotacion de `GET .../check-runs/<id>/annotations`: su `raw_details`.
#[derive(serde::Deserialize)]
struct AnotacionLeida {
    #[serde(default)]
    raw_details: Option<String>,
}

/// La App del revisor: `--origen` o `plataforma.revisor_id` de la
/// configuracion de la base; con la configuracion (para el tope) y los
/// avisos.
fn origen_y_config(
    origen: Option<u64>,
    g: &crate::git::Git,
    dir: &Path,
) -> (Option<u64>, crate::config::Config, Vec<String>) {
    let mut avisos = Vec::new();
    let config = crate::config::de_base(g, dir).unwrap_or_else(|_| {
        avisos.push("config-de-base-no-leida".to_owned());
        crate::config::Config::suelo()
    });
    let origen = origen
        .or_else(|| config.plataforma.as_ref().and_then(|p| p.revisor_id))
        .filter(|id| *id != 0);
    if origen.is_none() {
        avisos.push("sin-origen".to_owned());
    }
    (origen, config, avisos)
}

fn carpeta_actual() -> Result<PathBuf, Informe> {
    std::env::current_dir().map_err(|e| fallo_io(Path::new("."), e.to_string()))
}

fn id_de_app(v: &str) -> Result<u64, Informe> {
    v.parse::<u64>()
        .ok()
        .filter(|n| *n != 0)
        .ok_or_else(|| uso("origen-invalido"))
}

/// Los check-runs de cada `--check-runs`, en el orden dado (del commit mas
/// nuevo al mas viejo).
fn leer_checks(ficheros: &[PathBuf]) -> Result<Vec<Vec<super::ronda::CheckRun>>, Informe> {
    ficheros
        .iter()
        .map(|p| {
            leer_json::<PaginaChecks>(p, "un volcado de check-runs de la API").map(|x| x.check_runs)
        })
        .collect()
}

/// `anterior --check-runs <json>... [--origen <id>]`: el check que sirve de
/// ronda anterior ([`super::ronda::elegir`]). Linea: `hallazgos ok
/// anterior=<id>|ninguno [sha=<sha>] [avisos=...]`.
fn anterior(a: &[String]) -> Result<Informe, Informe> {
    let o = opciones(a, &["--origen"], &["--check-runs"])?;
    let origen = o.una("--origen").as_deref().map(id_de_app).transpose()?;
    let checks = o.rutas("--check-runs");
    let dir = carpeta_actual()?;
    let (origen, _, avisos) = origen_y_config(origen, &crate::git::Git::nuevo(), &dir);
    let commits = leer_checks(&checks)?;
    let elegido = origen.and_then(|o| super::ronda::elegir(commits, o).check);
    let mut i = Informe::nuevo(NOMBRE, Resultado::Ok);
    i = match &elegido {
        Some(c) => i
            .con("anterior", c.id.to_string())
            .con("sha", c.head_sha.as_str()),
        None => i.con("anterior", "ninguno"),
    };
    Ok(i.con_lista("avisos", avisos))
}

/// Rutas separadas por NUL (la salida de `git diff --name-only -z`).
fn rutas_nul(p: &Path) -> Result<Vec<String>, Informe> {
    let texto = leer_entrada(p, crate::seleccion::orden::TOPE_RUTAS)?;
    Ok(texto
        .split('\0')
        .filter(|r| !r.trim().is_empty())
        .map(str::to_owned)
        .collect())
}

/// Las rondas extra de la tarea `id` en la rama de estado, o el aviso.
fn extra_de_tarea(g: &crate::git::Git, dir: &Path, id: &str) -> Result<u32, String> {
    let v = crate::estado::leer(g, dir)
        .map_err(|i| format!("tarea-no-leida:{}", i.valor("motivo").unwrap_or("estado")))?;
    let (_, cuerpo) = crate::estado::tarea_de(&v, id).map_err(|f| match f {
        crate::estado::Fallo::Rechazada { motivo, .. } => format!("tarea-no-leida:{motivo}"),
        crate::estado::Fallo::Error(_) => "tarea-no-leida:error".to_owned(),
    })?;
    Ok(super::ronda::rondas_extra(&cuerpo))
}

/// `ronda --check-runs <json>... [--anotaciones <json>]... --rutas <nul>
/// [--tarea TASK-NNN] [--origen <id>] --salida <dir>`: escribe
/// `<dir>/ronda.json` ([`super::ronda::decidir`]). El tope efectivo es
/// `tope_rondas` de la base mas los elementos de `## Rondas extra` de la
/// tarea. Linea: `hallazgos ok ronda=R modo=<m> anterior=<sha12|ninguno>
/// abiertos=N tope=T avisos=<...>`.
fn ronda(a: &[String]) -> Result<Informe, Informe> {
    let o = opciones(
        a,
        &["--rutas", "--tarea", "--origen", "--salida"],
        &["--check-runs", "--anotaciones"],
    )?;
    comprobar(&o, "--tarea", crate::git::es_id, "id-invalido")?;
    let origen = o.una("--origen").as_deref().map(id_de_app).transpose()?;
    let (checks, anotaciones, tarea) = (
        o.rutas("--check-runs"),
        o.rutas("--anotaciones"),
        o.una("--tarea"),
    );
    let Some(rutas) = o.ruta("--rutas") else {
        return Err(uso("sin-rutas"));
    };
    let Some(dir_salida) = o.ruta("--salida") else {
        return Err(uso("sin-salida"));
    };
    let cambiados = rutas_nul(&rutas)?;
    let dir = carpeta_actual()?;
    let g = crate::git::Git::nuevo();
    let (origen, config, mut avisos) = origen_y_config(origen, &g, &dir);
    let mut tope = u32::from(config.tope_rondas);
    if let Some(id) = &tarea {
        match extra_de_tarea(&g, &dir, id) {
            Ok(n) => tope = tope.saturating_add(n),
            Err(aviso) => avisos.push(aviso),
        }
    }
    let commits = leer_checks(&checks)?;
    let eleccion = origen
        .map(|o| super::ronda::elegir(commits, o))
        .unwrap_or_default();
    let previa = match &eleccion.check {
        None => None,
        Some(c) => {
            let mut aceptados = Vec::new();
            for p in &anotaciones {
                let pagina: Vec<AnotacionLeida> =
                    leer_json(p, "una pagina de anotaciones de la API")?;
                for n in pagina {
                    let a = n
                        .raw_details
                        .as_deref()
                        .and_then(|r| serde_json::from_str::<super::AceptadoDe>(r).ok())
                        .ok_or_else(|| {
                            no_es(
                                p,
                                "una pagina de anotaciones de tc2 hallazgos veredicto",
                                "una anotacion sin raw_details de tc2".to_owned(),
                            )
                        })?;
                    aceptados.push(a);
                }
            }
            let previa = super::ronda::anterior(c, aceptados).ok_or_else(|| {
                Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(
                    Mensaje::nuevo(&txt::ANTERIOR_INCOHERENTE)
                        .con("check", c.id.to_string())
                        .con("sha", crate::contrato::corto(&c.head_sha)),
                )
            })?;
            Some(previa)
        }
    };
    let d = super::ronda::decidir(previa.as_ref(), &cambiados, tope, eleccion.ligera_pendiente);
    std::fs::create_dir_all(&dir_salida).map_err(|e| fallo_io(&dir_salida, e.to_string()))?;
    let v = serde_json::to_value(&d).map_err(|e| fallo_io(&dir_salida, e.to_string()))?;
    escribir_json(&dir_salida.join("ronda.json"), &v)?;
    Ok(Informe::nuevo(NOMBRE, Resultado::Ok)
        .con("ronda", d.ronda.to_string())
        .con("modo", d.modo.texto())
        .con(
            "anterior",
            d.sha_anterior
                .as_deref()
                .map_or("ninguno".to_owned(), |s| s.chars().take(12).collect()),
        )
        .con("abiertos", d.abiertos.len().to_string())
        .con("tope", d.tope.to_string())
        .con_lista("avisos", avisos))
}

/// Un SHA de commit (SHA-1 o SHA-256), en minusculas.
fn es_sha(t: &str) -> bool {
    matches!(t.len(), 40 | 64) && t.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Una ref de la base (`origin/develop`): sin opciones ni rangos.
fn es_ref(t: &str) -> bool {
    !t.is_empty()
        && t.len() <= 200
        && !t.starts_with('-')
        && !t.contains("..")
        && t.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '.' | '_' | '-'))
}

/// Desde donde se saca el diff incremental de `anterior` a `head` (F6-6):
/// sin lo que entro de la base en la rama (un merge de `base` o «Update
/// branch») despues de la ronda anterior, que no es del PR.
///
/// - `anterior` no es ancestro de `head` (rebase): `None`, el diff vuelve a
///   `base...head`;
/// - el `merge-base` de `base` y `head` ya estaba en `anterior` (no entro
///   nada de la base desde entonces): `anterior`;
/// - si no, el arbol de `anterior` con lo que trajo la base
///   (`git merge-tree --write-tree anterior <merge-base>`): el diff de ese
///   arbol a `head` es solo lo del PR. Si esa mezcla tiene conflictos, o git
///   falla, `None` (falla en cerrado: se revisa todo el PR).
///
/// Da tambien el motivo para la linea.
pub fn desde_incremental(
    g: &crate::git::Git,
    dir: &Path,
    base: &str,
    anterior: &str,
    head: &str,
) -> (Option<String>, &'static str) {
    use crate::git::Subcomando;
    match crate::git::es_ancestro(g, dir, NOMBRE, anterior, head) {
        Ok(true) => {}
        Ok(false) => return (None, "no-ancestro"),
        Err(_) => return (None, "git"),
    }
    let Ok(mb) = crate::git::exigir(g, dir, NOMBRE, Subcomando::MergeBase, &[base, head]) else {
        return (None, "sin-merge-base");
    };
    let mb = mb.trim().to_owned();
    if !es_sha(&mb) {
        return (None, "sin-merge-base");
    }
    match crate::git::es_ancestro(g, dir, NOMBRE, &mb, anterior) {
        Ok(true) => return (Some(anterior.to_owned()), "directo"),
        Ok(false) => {}
        Err(_) => return (None, "git"),
    }
    let Ok(s) = crate::git::correr(
        g,
        dir,
        NOMBRE,
        Subcomando::MergeTree,
        &["--write-tree", "--no-messages", anterior, &mb],
    ) else {
        return (None, "git");
    };
    let arbol = s
        .stdout
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_owned();
    match s.codigo {
        0 if es_sha(&arbol) => (Some(arbol), "merge"),
        1 => (None, "conflicto"),
        _ => (None, "git"),
    }
}

/// `desde --anterior <sha> --head <sha> --base <ref>`
/// ([`desde_incremental`]). Linea: `hallazgos ok desde=<oid>|base
/// motivo=<directo|merge|no-ancestro|sin-merge-base|conflicto|git>`.
fn desde(a: &[String]) -> Result<Informe, Informe> {
    let o = opciones(a, &["--anterior", "--head", "--base"], &[])?;
    let (Some(anterior), Some(head), Some(base)) =
        (o.una("--anterior"), o.una("--head"), o.una("--base"))
    else {
        return Err(uso("faltan-argumentos"));
    };
    if !es_sha(&anterior) || !es_sha(&head) {
        return Err(uso("sha-invalido"));
    }
    if !es_ref(&base) {
        return Err(uso("base-invalida"));
    }
    let dir = carpeta_actual()?;
    let (desde, motivo) =
        desde_incremental(&crate::git::Git::nuevo(), &dir, &base, &anterior, &head);
    Ok(Informe::nuevo(NOMBRE, Resultado::Ok)
        .con("desde", desde.unwrap_or_else(|| "base".to_owned()))
        .con("motivo", motivo))
}

/// Una `ronda.json` del contrato vigente.
fn leer_decision(p: &Path) -> Result<super::ronda::Decision, Informe> {
    let d: super::ronda::Decision = leer_json(p, "una ronda.json de tc2 hallazgos ronda")?;
    if d.version != super::VERSION {
        return Err(no_es(
            p,
            "una ronda.json de tc2 hallazgos ronda",
            format!("version {} y no {}", d.version, super::VERSION),
        ));
    }
    Ok(d)
}

/// El detalle de `tc2 revisoras --detalle`.
#[derive(serde::Deserialize)]
struct Detalle {
    revisoras: Vec<super::ronda::RevisoraDetalle>,
}

/// `abiertos --ronda <json> --detalle <json> --salida <dir>`: por revisora
/// del detalle (en su orden), `abiertos-<i>.json` y `abiertos-<i>.md`
/// ([`super::ronda::repartir`]). Linea: `hallazgos ok revisoras=N
/// abiertos=M sin-revisora=K`.
fn abiertos(a: &[String]) -> Result<Informe, Informe> {
    let o = opciones(a, &["--ronda", "--detalle", "--salida"], &[])?;
    let (Some(r), Some(detalle), Some(dir)) =
        (o.ruta("--ronda"), o.ruta("--detalle"), o.ruta("--salida"))
    else {
        return Err(uso("faltan-argumentos"));
    };
    let d = leer_decision(&r)?;
    let det: Detalle = leer_json(&detalle, "un detalle de tc2 revisoras")?;
    let reparto = super::ronda::repartir(&d.abiertos, &det.revisoras);
    std::fs::create_dir_all(&dir).map_err(|e| fallo_io(&dir, e.to_string()))?;
    let mut dados = 0;
    for (i, lista) in reparto.iter().enumerate() {
        dados += lista.len();
        let v = serde_json::to_value(lista).map_err(|e| fallo_io(&dir, e.to_string()))?;
        escribir_json(&dir.join(format!("abiertos-{i}.json")), &v)?;
        let md = dir.join(format!("abiertos-{i}.md"));
        std::fs::write(&md, super::ronda::abiertos_markdown(lista))
            .map_err(|e| fallo_io(&md, e.to_string()))?;
    }
    let graves = d.abiertos.iter().filter(|a| a.grave()).count();
    Ok(Informe::nuevo(NOMBRE, Resultado::Ok)
        .con("revisoras", reparto.len().to_string())
        .con("abiertos", dados.to_string())
        .con("sin-revisora", (graves - dados.min(graves)).to_string()))
}

// --- Prompt (F6-7) ------------------------------------------------------------

/// Bytes que se leen como mucho del `SKILL.md` (al prompt van
/// [`super::prompt::TOPE_SKILL`]).
pub const TOPE_SKILL: u64 = 4 * 1024 * 1024;

/// `prompt ...`: el prompt de una revisora ([`super::prompt::componer`]).
fn prompt(a: &[String]) -> Result<Informe, Informe> {
    let o = opciones(
        a,
        &[
            "--contexto",
            "--skill",
            "--seccion",
            "--abiertos",
            "--diff",
            "--salida",
            "--revisora",
            "--bytes",
            "--tope",
        ],
        &[],
    )?;
    comprobar(&o, "--revisora", es_revisora, "revisora-invalida")?;
    let numero = |n: &str| {
        o.una(n)
            .map(|v| v.parse::<u64>().map_err(|_| uso("numero-invalido")))
            .transpose()
    };
    let (bytes, tope) = (numero("--bytes")?, numero("--tope")?);
    let (skill, seccion, abiertos) = (o.ruta("--skill"), o.ruta("--seccion"), o.ruta("--abiertos"));
    let (Some(revisora), Some(contexto), Some(diff), Some(bytes), Some(tope), Some(salida)) = (
        o.una("--revisora"),
        o.ruta("--contexto"),
        o.ruta("--diff"),
        bytes,
        tope,
        o.ruta("--salida"),
    ) else {
        return Err(uso("faltan-argumentos"));
    };
    let opcional =
        |p: &Option<PathBuf>, t: u64| p.as_deref().map(|p| leer_entrada(p, t)).transpose();
    let contexto = leer_entrada(&contexto, TOPE_EXTRA)?;
    let skill = opcional(&skill, TOPE_SKILL)?;
    let seccion = opcional(&seccion, TOPE_JSON)?;
    let abiertos = opcional(&abiertos, TOPE_JSON)?;
    let diff = leer_entrada(&diff, TOPE_DIFF)?;
    let partes = super::prompt::Partes {
        revisora: &revisora,
        contexto: &contexto,
        skill: skill.as_deref(),
        seccion: seccion.as_deref(),
        abiertos: abiertos.as_deref(),
        diff: &diff,
        bytes,
        tope,
    };
    let texto = super::prompt::componer(&partes);
    std::fs::write(&salida, &texto).map_err(|e| fallo_io(&salida, e.to_string()))?;
    let si = |b: bool| if b { "si" } else { "no" };
    Ok(Informe::nuevo(NOMBRE, Resultado::Ok)
        .con("revisora", revisora.as_str())
        .con("bytes", texto.len().to_string())
        .con("cortado", si(partes.cortado()))
        .con("abiertos", si(partes.abiertos.is_some())))
}
