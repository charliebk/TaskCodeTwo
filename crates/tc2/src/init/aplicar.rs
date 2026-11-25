//! Aplicacion de las acciones `crea` del plan de `tc2 init`. Cada efecto
//! vuelve a comprobar lo que lee: si algo cambio desde el plan, falla sin
//! pisarlo (las puertas, con `aplicar-puertas`, igual: solo crean lo que
//! falta en ese momento).

use std::io::Write;
use std::path::Path;

use super::inventario::Conexion;
use super::{Contexto, Efecto, NOMBRE};
use crate::contrato::{Informe, Resultado};
use crate::git::estado::construir_commit;
use crate::git::remoto::publicar;
use crate::git::{Cambio, Git, Subcomando, exigir};
use crate::huella::LINEAS_EXCLUDE;
use crate::i18n::Mensaje;
use crate::i18n::catalogo::comun::CAMBIO_DESDE_EL_PLAN;
use crate::i18n::catalogo::init as txt;
use crate::plan_dato::externo;
use crate::plataforma::puertas::{APLICAR_PUERTAS, Origenes, modelo};

/// Mensaje del commit raiz de `main` y `develop`.
const MENSAJE_RAIZ: &str = "chore: inicia TaskCodeTwo (tc2 init)";

/// Aplica un efecto. `Err` con el mensaje del fallo, para el hueco
/// `{error}` de `FALLO_AL_APLICAR` (se traduce al escribir la salida).
pub(super) fn aplicar(e: &Efecto, c: &Contexto<'_>) -> Result<(), Mensaje> {
    match e {
        Efecto::Ninguno => Ok(()),
        Efecto::Escribir { ruta, contenido } => escribir_nuevo(ruta, contenido),
        Efecto::Anadir { ruta, lineas } => anadir(ruta, lineas),
        Efecto::Excluir { principal } => {
            for l in LINEAS_EXCLUDE {
                crate::git::rama::excluir(c.git, principal, NOMBRE, l).map_err(|i| i.mensaje())?;
            }
            Ok(())
        }
        Efecto::RamasRaiz {
            principal,
            ficheros,
            ramas,
            head,
            publicar,
        } => {
            ramas_raiz(c.git, principal, ficheros, ramas, *head, *publicar).map_err(|i| i.mensaje())
        }
        Efecto::MontarEstado { principal } => {
            let i = crate::estado::montar(c.git, principal);
            if i.resultado.es_exito() {
                Ok(())
            } else {
                Err(i.mensaje())
            }
        }
        // Necesita la plataforma: lo aplica [`puertas`] desde `init`.
        Efecto::AplicarPuertas { .. } => Err(Mensaje::nuevo(&txt::EFECTO_PUERTAS_SIN_PLATAFORMA)),
    }
}

/// `aplicar-puertas` con el adaptador del plan: el adaptador vuelve a
/// comprobar todo antes de escribir y solo crea lo que falta.
pub(super) fn puertas(conexion: &Conexion, origenes: &Origenes) -> Informe {
    match conexion {
        Ok((p, _)) => p.aplicar_puertas(&modelo(), origenes),
        Err(m) => Informe::nuevo(APLICAR_PUERTAS, Resultado::Rechazada).con("motivo", m.clone()),
    }
}

/// Crea `ruta` con `contenido`; si ya existe (la creo alguien despues del
/// plan), falla sin tocarla.
fn escribir_nuevo(ruta: &Path, contenido: &str) -> Result<(), Mensaje> {
    if let Some(padre) = ruta.parent() {
        std::fs::create_dir_all(padre).map_err(externo)?;
    }
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(ruta)
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                Mensaje::nuevo(&CAMBIO_DESDE_EL_PLAN).con("ruta", ruta.display().to_string())
            } else {
                externo(e)
            }
        })?;
    f.write_all(contenido.as_bytes()).map_err(externo)
}

/// Anade a `ruta` las `lineas` que le falten, sin tocar las demas (y con
/// el fin de linea que ya use el fichero).
fn anadir(ruta: &Path, lineas: &[String]) -> Result<(), Mensaje> {
    let actual = match std::fs::read_to_string(ruta) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(externo(e)),
    };
    let fin = if actual.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut nuevo = actual.clone();
    for l in lineas {
        if actual.lines().any(|x| super::inventario::misma_linea(x, l)) {
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
    std::fs::write(ruta, nuevo).map_err(externo)
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
        let bytes =
            std::fs::read(&ruta).map_err(|e| crate::lectura::no_se_lee(NOMBRE, &ruta, &e))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Idioma;

    /// A7-07: un fichero que alguien creo despues del plan no se pisa, y el
    /// diagnostico sale entero en ingles con `en`.
    #[test]
    fn escribir_nuevo_sobre_uno_existente_en_ingles() -> std::io::Result<()> {
        let dir = std::env::temp_dir()
            .join("tc2-tests")
            .join(format!("init-escribir-{}", std::process::id()));
        std::fs::create_dir_all(&dir)?;
        let ruta = dir.join("config.yml");
        std::fs::write(&ruta, "de otra persona\n")?;
        let r = escribir_nuevo(&ruta, "de tc2\n");
        let ahora = std::fs::read_to_string(&ruta);
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(ahora?, "de otra persona\n");
        let r_txt = ruta.display().to_string();
        let e = r.err().map(|m| {
            Mensaje::nuevo(&txt::FALLO_AL_APLICAR)
                .con("ruta", r_txt.clone())
                .con_mensaje("error", m)
                .texto(Idioma::En)
        });
        assert_eq!(
            e,
            Some(format!(
                "could not create {r_txt}: {r_txt} changed after the plan was made. What the previous lines list is already done: fix the cause and run tc2 init again"
            ))
        );
        Ok(())
    }

    /// A7-07: el informe de un paso que falla entra en `{error}` como
    /// mensaje (se traduce); sin diagnostico, la linea de contrato tal cual.
    #[test]
    fn informe_como_mensaje() {
        let con = Informe::nuevo("estado", Resultado::Error)
            .diagnostico(Mensaje::nuevo(&txt::EFECTO_PUERTAS_SIN_PLATAFORMA));
        let sin = Informe::nuevo("estado", Resultado::Error).con("motivo", "x");
        assert_eq!(
            con.mensaje().texto(Idioma::En),
            "aplicar-puertas without a platform"
        );
        assert_eq!(sin.mensaje().texto(Idioma::En), sin.linea());
        let fallo = Mensaje::nuevo(&txt::FALLO_AL_APLICAR)
            .con("ruta", "taskcodetwo-main")
            .con_mensaje("error", con.mensaje())
            .texto(Idioma::En);
        assert!(
            fallo.starts_with(
                "could not create taskcodetwo-main: aplicar-puertas without a platform. "
            ),
            "{fallo}"
        );
    }
}
