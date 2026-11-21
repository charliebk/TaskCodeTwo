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

#[derive(Default)]
struct Args {
    respuesta: Option<PathBuf>,
    diff: Option<PathBuf>,
    revisora: Option<String>,
    informe: Option<PathBuf>,
    salida: Option<PathBuf>,
    resumen: Option<PathBuf>,
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

fn args(a: &[String]) -> Result<Args, Informe> {
    let mut r = Args::default();
    let mut it = a.iter();
    while let Some(x) = it.next() {
        let mut valor = || it.next().cloned().ok_or_else(|| uso("falta-valor"));
        match x.as_str() {
            "--respuesta" if r.respuesta.is_none() => r.respuesta = Some(valor()?.into()),
            "--diff" if r.diff.is_none() => r.diff = Some(valor()?.into()),
            "--revisora" if r.revisora.is_none() => {
                let v = valor()?;
                if !es_revisora(&v) {
                    return Err(uso("revisora-invalida"));
                }
                r.revisora = Some(v);
            }
            "--informe" if r.informe.is_none() => r.informe = Some(valor()?.into()),
            "--salida" if r.salida.is_none() => r.salida = Some(valor()?.into()),
            "--resumen" if r.resumen.is_none() => r.resumen = Some(valor()?.into()),
            _ => return Err(uso("argumento-desconocido")),
        }
    }
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
    let v = super::validar(revisora, &respuesta, &diff, inf.as_ref());
    if let Some(p) = &a.salida {
        let texto = serde_json::to_string_pretty(&v).map_err(|e| fallo_io(p, e.to_string()))?;
        std::fs::write(p, texto + "\n").map_err(|e| fallo_io(p, e.to_string()))?;
    }
    if let Some(p) = &a.resumen {
        std::fs::write(p, super::resumen_markdown(&v)).map_err(|e| fallo_io(p, e.to_string()))?;
    }
    let mut i = Informe::nuevo(NOMBRE, Resultado::Ok)
        .con("revisora", revisora.as_str())
        .con("aceptados", v.aceptados.len().to_string())
        .con("rechazados", v.rechazados.len().to_string())
        .con_lista("motivos", v.motivos());
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
    let mut salida: Option<PathBuf> = None;
    let mut extra: Option<PathBuf> = None;
    let mut entradas: Vec<PathBuf> = Vec::new();
    let mut it = a.iter();
    while let Some(x) = it.next() {
        let mut valor = || it.next().cloned().ok_or_else(|| uso("falta-valor"));
        match x.as_str() {
            "--hallazgos" => entradas.push(valor()?.into()),
            "--salida" if salida.is_none() => salida = Some(valor()?.into()),
            "--extra" if extra.is_none() => extra = Some(valor()?.into()),
            _ => return Err(uso("argumento-desconocido")),
        }
    }
    let Some(dir) = salida else {
        return Err(uso("sin-salida"));
    };
    let extra = match &extra {
        Some(p) => leer_entrada(p, TOPE_EXTRA)?,
        None => String::new(),
    };
    let revisoras: Vec<Option<super::Validados>> =
        entradas.iter().map(|p| leer_validados(p)).collect();
    let v = super::veredicto(&revisoras);
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
        .con("lotes", lotes.len().max(1).to_string()))
}
