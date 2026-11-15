//! `tag-anotado`: tag anotado sobre un commit y publicado, idempotente.

use std::path::Path;
use std::time::Duration;

use super::remoto::publicar;
use super::{Git, Paso, REMOTO, Subcomando, commit_de, correr, exigir};
use crate::contrato::{Informe, Resultado, corto};

/// Pone `tag` sobre `referencia` y lo publica. Compara con el tag local
/// (aunque no este publicado) y con el del remoto: en otro commit →
/// `rechazada` sin publicar nada; en el mismo y publicado → `sin-cambios`.
pub fn tag_anotado(
    git: &Git,
    dir: &Path,
    tag: &str,
    referencia: &str,
    esperas: &[Duration],
) -> Informe {
    tag_paso(git, dir, tag, referencia, esperas).unwrap_or_else(|i| i)
}

fn tag_paso(
    git: &Git,
    dir: &Path,
    tag: &str,
    referencia: &str,
    esperas: &[Duration],
) -> Paso<Informe> {
    const OP: &str = "tag-anotado";
    let valido = |t: &str| {
        !t.is_empty()
            && !t.starts_with('-')
            && !t.chars().any(|c| c.is_whitespace() || c.is_control())
    };
    if !valido(tag) || !valido(referencia) {
        return Err(Informe::nuevo(OP, Resultado::Uso).con("motivo", "argumento-invalido"));
    }
    let Some(commit) = commit_de(git, dir, OP, referencia)? else {
        return Ok(Informe::nuevo(OP, Resultado::Rechazada)
            .con("tag", tag)
            .con("motivo", "ref-no-existe")
            .diagnostico(format!("{referencia} no existe")));
    };
    let otro_commit = |donde: &str| {
        Informe::nuevo(OP, Resultado::Rechazada)
            .con("tag", tag)
            .con("motivo", "existe-en-otro-commit")
            .con("donde", donde)
            .diagnostico(format!(
                "el tag {tag} ya existe sobre otro commit: elige otro nombre"
            ))
    };
    let local = commit_de(git, dir, OP, &format!("refs/tags/{tag}"))?;
    if local.as_ref().is_some_and(|l| *l != commit) {
        return Ok(otro_commit("local"));
    }
    let remoto = commit_remoto(git, dir, OP, tag)?;
    if remoto.as_ref().is_some_and(|r| *r != commit) {
        return Ok(otro_commit("remoto"));
    }
    if local.is_some() && remoto.is_some() {
        return Ok(Informe::nuevo(OP, Resultado::SinCambios)
            .con("tag", tag)
            .con("commit", corto(&commit)));
    }
    if local.is_none() {
        exigir(
            git,
            dir,
            OP,
            Subcomando::Tag,
            &["-a", "-m", tag, "--", tag, &commit],
        )?;
    }
    if remoto.is_none() {
        let refspec = format!("refs/tags/{tag}:refs/tags/{tag}");
        let (r, intentos, ultimo) = publicar(git, dir, OP, &refspec, esperas)?;
        if !r.es_exito() {
            return Ok(Informe::nuevo(
                OP,
                if r == Resultado::Rechazada {
                    Resultado::Rechazada
                } else {
                    Resultado::Sinred
                },
            )
            .con("tag", tag)
            .con("intentos", intentos.to_string())
            .diagnostico(format!(
                "no se pudo publicar el tag: {ultimo}; el tag queda en local, relanza con red"
            )));
        }
    }
    Ok(Informe::nuevo(OP, Resultado::Ok)
        .con("tag", tag)
        .con("commit", corto(&commit)))
}

/// Commit del tag en el remoto (anotado: su `^{}`; ligero: el propio).
fn commit_remoto(git: &Git, dir: &Path, op: &str, tag: &str) -> Paso<Option<String>> {
    let r = format!("refs/tags/{tag}");
    let pelado = format!("{r}^{{}}");
    let s = correr(
        git,
        dir,
        op,
        Subcomando::LsRemote,
        &["--tags", REMOTO, &r, &pelado],
    )?;
    if !s.exito() {
        return Err(Informe::nuevo(op, Resultado::Sinred)
            .con("remoto", REMOTO)
            .diagnostico(format!("{op}: no se pudo leer {REMOTO}; reintenta con red")));
    }
    let mut directo = None;
    for l in s.stdout.lines() {
        let mut c = l.split('\t');
        let (Some(sha), Some(nombre)) = (c.next(), c.next()) else {
            continue;
        };
        if nombre == pelado {
            return Ok(Some(sha.to_owned()));
        }
        if nombre == r {
            directo = Some(sha.to_owned());
        }
    }
    Ok(directo)
}
