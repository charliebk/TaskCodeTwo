//! Aplicacion de las acciones `crea` del plan de `tc2 init`. Cada efecto
//! vuelve a comprobar lo que lee: si algo cambio desde el plan, falla sin
//! pisarlo.

use std::io::Write;
use std::path::Path;

use super::{Contexto, Efecto, NOMBRE};
use crate::contrato::{Informe, Resultado};
use crate::git::estado::construir_commit;
use crate::git::remoto::publicar;
use crate::git::{Cambio, Git, Subcomando, exigir};
use crate::huella::LINEAS_EXCLUDE;
use crate::i18n::Idioma;
use crate::i18n::Mensaje;
use crate::i18n::catalogo::init as txt;

/// Mensaje del commit raiz de `main` y `develop`.
const MENSAJE_RAIZ: &str = "chore: inicia TaskCodeTwo (tc2 init)";

/// Texto de un informe para la linea de fallo.
fn texto(i: &Informe) -> String {
    i.diagnostico_en(Idioma::Es).unwrap_or_else(|| i.linea())
}

/// Aplica un efecto. `Err` con el texto del fallo.
pub(super) fn aplicar(e: &Efecto, c: &Contexto<'_>) -> Result<(), String> {
    match e {
        Efecto::Ninguno => Ok(()),
        Efecto::Escribir { ruta, contenido } => escribir_nuevo(ruta, contenido),
        Efecto::Anadir { ruta, lineas } => anadir(ruta, lineas),
        Efecto::Excluir { principal } => {
            for l in LINEAS_EXCLUDE {
                crate::git::rama::excluir(c.git, principal, NOMBRE, l).map_err(|i| texto(&i))?;
            }
            Ok(())
        }
        Efecto::RamasRaiz {
            principal,
            ficheros,
            ramas,
            head,
            publicar,
        } => ramas_raiz(c.git, principal, ficheros, ramas, *head, *publicar).map_err(|i| texto(&i)),
        Efecto::MontarEstado { principal } => {
            let i = crate::estado::montar(c.git, principal);
            if i.resultado.es_exito() {
                Ok(())
            } else {
                Err(texto(&i))
            }
        }
    }
}

/// Crea `ruta` con `contenido`; si ya existe (la creo alguien despues del
/// plan), falla sin tocarla.
fn escribir_nuevo(ruta: &Path, contenido: &str) -> Result<(), String> {
    if let Some(padre) = ruta.parent() {
        std::fs::create_dir_all(padre).map_err(|e| e.to_string())?;
    }
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(ruta)
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                Mensaje::nuevo(&txt::CAMBIO_DESDE_EL_PLAN)
                    .con("ruta", ruta.display().to_string())
                    .texto(Idioma::Es)
            } else {
                e.to_string()
            }
        })?;
    f.write_all(contenido.as_bytes()).map_err(|e| e.to_string())
}

/// Anade a `ruta` las `lineas` que le falten, sin tocar las demas (y con
/// el fin de linea que ya use el fichero).
fn anadir(ruta: &Path, lineas: &[String]) -> Result<(), String> {
    let actual = match std::fs::read_to_string(ruta) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e.to_string()),
    };
    let fin = if actual.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut nuevo = actual.clone();
    for l in lineas {
        if actual.lines().any(|x| x.trim() == l) {
            continue;
        }
        if !nuevo.is_empty() && !nuevo.ends_with('\n') {
            nuevo.push_str(fin);
        }
        nuevo.push_str(l);
        nuevo.push_str(fin);
    }
    if nuevo == actual {
        return Ok(());
    }
    std::fs::write(ruta, nuevo).map_err(|e| e.to_string())
}

/// Commit raiz con `ficheros` (tal como estan en el disco) y `ramas` en el.
/// Si HEAD estaba sin nacer en una de ellas, el indice se pone al dia con
/// esos ficheros (si no, `git status` los daria por borrados). Con remoto,
/// se publican sin force.
fn ramas_raiz(
    git: &Git,
    principal: &Path,
    ficheros: &[String],
    ramas: &[String],
    head: bool,
    con_remoto: bool,
) -> Result<(), Informe> {
    let mut cambios = Vec::with_capacity(ficheros.len());
    for f in ficheros {
        let ruta = principal.join(f);
        let bytes = std::fs::read(&ruta).map_err(|e| super::no_se_lee(&ruta, &e))?;
        cambios.push(Cambio::nuevo(
            f.as_str(),
            String::from_utf8_lossy(&bytes).into_owned(),
        ));
    }
    let sha = construir_commit(git, principal, NOMBRE, None, &cambios, MENSAJE_RAIZ)?
        .ok_or_else(|| Informe::nuevo(NOMBRE, Resultado::Error))?;
    for r in ramas {
        exigir(git, principal, NOMBRE, Subcomando::Branch, &[r, &sha])?;
    }
    if head {
        let mut args = vec!["--"];
        args.extend(ficheros.iter().map(String::as_str));
        exigir(git, principal, NOMBRE, Subcomando::Add, &args)?;
    }
    if con_remoto {
        for r in ramas {
            let p = publicar(
                git,
                principal,
                NOMBRE,
                &format!("refs/heads/{r}:refs/heads/{r}"),
                &[],
            )?;
            if !p.resultado.es_exito() {
                return Err(Informe::nuevo(NOMBRE, p.resultado).diagnostico(
                    Mensaje::nuevo(&txt::FALLO_AL_APLICAR)
                        .con("ruta", format!("refs/heads/{r}"))
                        .con("error", p.ultimo),
                ));
            }
        }
    }
    Ok(())
}
