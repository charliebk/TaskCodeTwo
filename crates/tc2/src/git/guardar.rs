//! `guardar "<mensaje>" <rutas...>`: commit de las rutas dadas y solo de
//! ellas, nunca `add -A` (RF-23, ADR F1-4 D2a). Con un merge abierto (lo deja
//! `actualizar-rama` tras un conflicto) cierra el merge si las rutas resuelven
//! todos los conflictos y ya no tienen marcas.

use std::path::{Component, Path};

use super::remoto::{merge_abierto, sin_resolver};
use super::{Git, Paso, Subcomando, commit_de, entradas_status, exigir};
use crate::contrato::{Informe, Resultado, corto};

const OP: &str = "guardar";

/// Guarda `rutas` (relativas a `dir`) con `mensaje`.
pub fn guardar(git: &Git, dir: &Path, mensaje: &str, rutas: &[&str]) -> Informe {
    guardar_paso(git, dir, mensaje, rutas).unwrap_or_else(|i| i)
}

fn uso(motivo: &str) -> Informe {
    Informe::nuevo(OP, Resultado::Uso)
        .con("motivo", motivo)
        .diagnostico("uso: guardar \"<mensaje>\" <rutas...>")
}

fn guardar_paso(git: &Git, dir: &Path, mensaje: &str, rutas: &[&str]) -> Paso<Informe> {
    if mensaje.trim().is_empty() {
        return Err(uso("mensaje-vacio"));
    }
    if rutas.is_empty() {
        return Err(uso("sin-rutas"));
    }
    let prefijo = exigir(git, dir, OP, Subcomando::RevParse, &["--show-prefix"])?;
    let mut desde_raiz = Vec::with_capacity(rutas.len());
    for r in rutas {
        match relativa_a_la_raiz(prefijo.trim(), r) {
            Some(p) => desde_raiz.push(p),
            None => return Err(uso("ruta-fuera-del-repo")),
        }
    }
    let mut args = vec!["--"];
    args.extend_from_slice(rutas);

    if merge_abierto(git, dir, OP)? {
        let pendientes: Vec<String> = sin_resolver(git, dir, OP)?
            .into_iter()
            .filter(|f| !desde_raiz.contains(f))
            .collect();
        if !pendientes.is_empty() {
            return Ok(Informe::nuevo(OP, Resultado::Rechazada)
                .con("motivo", "conflicto-sin-resolver")
                .con_lista("ficheros", pendientes.clone())
                .diagnostico(format!(
                    "resuelve {} y guardalo con el resto",
                    pendientes.join(" ")
                )));
        }
        let con_marcas: Vec<String> = rutas
            .iter()
            .filter(|r| tiene_marcas(&dir.join(r)))
            .map(|r| (*r).to_owned())
            .collect();
        if !con_marcas.is_empty() {
            return Ok(Informe::nuevo(OP, Resultado::Rechazada)
                .con("motivo", "marcas-de-conflicto")
                .con_lista("ficheros", con_marcas.clone())
                .diagnostico(format!(
                    "quita las marcas de conflicto de {}",
                    con_marcas.join(" ")
                )));
        }
        let estado = exigir(
            git,
            dir,
            OP,
            Subcomando::Status,
            &["--porcelain", "-z", "--untracked-files=all"],
        )?;
        let utiles = conocidas(dir, rutas, &desde_raiz, &entradas_status(&estado));
        if !utiles.is_empty() {
            let mut add = vec!["--"];
            add.extend_from_slice(&utiles);
            exigir(git, dir, OP, Subcomando::Add, &add)?;
        }
        // Durante un merge el commit es del indice entero (el merge), no
        // parcial: git no admite rutas aqui.
        exigir(git, dir, OP, Subcomando::Commit, &["-q", "-m", mensaje])?;
        let head = commit_de(git, dir, OP, "HEAD")?.unwrap_or_default();
        return Ok(Informe::nuevo(OP, Resultado::Ok)
            .con("commit", corto(&head))
            .con("tipo", "merge")
            .con("ficheros", rutas.len().to_string())
            .con("sin-guardar", sin_guardar(git, dir)?.to_string()));
    }

    let mut status = vec!["--porcelain", "-z", "--untracked-files=all"];
    status.extend_from_slice(&args);
    let entradas = entradas_status(&exigir(git, dir, OP, Subcomando::Status, &status)?);
    let cambios = entradas.len();
    if cambios == 0 {
        return Ok(Informe::nuevo(OP, Resultado::SinCambios)
            .con("sin-guardar", sin_guardar(git, dir)?.to_string()));
    }
    // Solo las rutas que git conoce o que existen: una ruta sin cambios ni
    // fichero haria fallar el `add` entero. `add -- <rutas>` recoge tambien
    // los borrados (git >= 2.0) y nunca toca otras rutas.
    let utiles = conocidas(dir, rutas, &desde_raiz, &entradas);
    let mut add = vec!["--"];
    add.extend_from_slice(&utiles);
    exigir(git, dir, OP, Subcomando::Add, &add)?;
    let mut commit = vec!["-q", "-m", mensaje, "--only", "--"];
    commit.extend_from_slice(&utiles);
    exigir(git, dir, OP, Subcomando::Commit, &commit)?;
    let head = commit_de(git, dir, OP, "HEAD")?.unwrap_or_default();
    Ok(Informe::nuevo(OP, Resultado::Ok)
        .con("commit", corto(&head))
        .con("ficheros", cambios.to_string())
        .con("sin-guardar", sin_guardar(git, dir)?.to_string()))
}

/// Rutas que existen en disco o que aparecen en el status (borradas).
fn conocidas<'a>(
    dir: &Path,
    rutas: &[&'a str],
    desde_raiz: &[String],
    status: &[(String, String)],
) -> Vec<&'a str> {
    rutas
        .iter()
        .zip(desde_raiz)
        .filter(|(r, raiz)| {
            dir.join(r).symlink_metadata().is_ok()
                || status.iter().any(|(_, s)| {
                    s == *raiz
                        || s.strip_prefix(raiz.as_str())
                            .is_some_and(|resto| resto.starts_with('/'))
                })
        })
        .map(|(r, _)| *r)
        .collect()
}

/// Cambios que quedan en el worktree sin guardar (se informan, no se tocan).
fn sin_guardar(git: &Git, dir: &Path) -> Paso<usize> {
    let s = exigir(
        git,
        dir,
        OP,
        Subcomando::Status,
        &["--porcelain", "-z", "--untracked-files=all"],
    )?;
    Ok(entradas_status(&s).len())
}

/// Lineas de marca de conflicto que deja git (`<<<<<<< ` y `>>>>>>> `).
fn tiene_marcas(fichero: &Path) -> bool {
    std::fs::read(fichero).is_ok_and(|b| {
        String::from_utf8_lossy(&b).lines().any(|l| {
            l.starts_with("<<<<<<< ")
                || l.starts_with(">>>>>>> ")
                || l == "<<<<<<<"
                || l == ">>>>>>>"
        })
    })
}

/// Ruta relativa a la raiz del worktree, con `/`, resolviendo `.` y `..`.
/// `None` si sale del worktree o es absoluta.
fn relativa_a_la_raiz(prefijo: &str, ruta: &str) -> Option<String> {
    let mut partes: Vec<String> = prefijo
        .split('/')
        .filter(|p| !p.is_empty())
        .map(str::to_owned)
        .collect();
    let ruta = ruta.replace('\\', "/");
    for c in Path::new(&ruta).components() {
        match c {
            Component::Normal(p) => partes.push(p.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir => {
                partes.pop()?;
            }
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(partes.join("/"))
}

#[cfg(test)]
mod tests {
    use super::relativa_a_la_raiz;

    #[test]
    fn rutas_relativas_a_la_raiz() {
        assert_eq!(
            relativa_a_la_raiz("", "a/b.txt").as_deref(),
            Some("a/b.txt")
        );
        assert_eq!(
            relativa_a_la_raiz("sub/", "./x.txt").as_deref(),
            Some("sub/x.txt")
        );
        assert_eq!(
            relativa_a_la_raiz("sub/", "..\\y.txt").as_deref(),
            Some("y.txt")
        );
        assert_eq!(relativa_a_la_raiz("", "../fuera.txt"), None);
        assert_eq!(relativa_a_la_raiz("", "/abs.txt"), None);
    }
}
