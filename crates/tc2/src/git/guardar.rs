//! `guardar "<mensaje>" <rutas...>`: commit de las rutas dadas y solo de
//! ellas, nunca `add -A` (RF-23, ADR F1-4 D2a). Con un merge abierto (lo deja
//! `actualizar-rama` tras un conflicto) cierra el merge si las rutas resuelven
//! todos los conflictos y ya no tienen marcas.

use std::path::{Component, Path};

use super::remoto::{merge_abierto, sin_resolver};
use super::{Git, Paso, Subcomando, commit_de, entradas_status, exigir, registros_status};
use crate::contrato::{Informe, Resultado, corto};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::git as txt;

const OP: &str = "guardar";

/// Guarda `rutas` (relativas a `dir`) con `mensaje`.
pub fn guardar(git: &Git, dir: &Path, mensaje: &str, rutas: &[&str]) -> Informe {
    guardar_paso(git, dir, mensaje, rutas).unwrap_or_else(|i| i)
}

fn uso(motivo: &str) -> Informe {
    Informe::nuevo(OP, Resultado::Uso)
        .con("motivo", motivo)
        .diagnostico(&crate::i18n::catalogo::cli::USO_GUARDAR)
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
                .diagnostico(
                    Mensaje::nuevo(&txt::RESUELVE_Y_GUARDA_CON_EL_RESTO)
                        .con("ficheros", pendientes.join(" ")),
                ));
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
                .diagnostico(
                    Mensaje::nuevo(&txt::QUITA_LAS_MARCAS).con("ficheros", con_marcas.join(" ")),
                ));
        }
        let estado = exigir(
            git,
            dir,
            OP,
            Subcomando::Status,
            &["--porcelain", "-z", "--untracked-files=all"],
        )?;
        let (add, _) = repartir(dir, rutas, &desde_raiz, &entradas_con_origen(&estado));
        if !add.is_empty() {
            let mut a = vec!["--"];
            a.extend_from_slice(&add);
            exigir(git, dir, OP, Subcomando::Add, &a)?;
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
    let salida = exigir(git, dir, OP, Subcomando::Status, &status)?;
    let cambios = entradas_status(&salida).len();
    if cambios == 0 {
        return Ok(Informe::nuevo(OP, Resultado::SinCambios)
            .con("sin-guardar", sin_guardar(git, dir)?.to_string()));
    }
    let (add, commit) = repartir(dir, rutas, &desde_raiz, &entradas_con_origen(&salida));
    if !add.is_empty() {
        let mut a = vec!["--"];
        a.extend_from_slice(&add);
        exigir(git, dir, OP, Subcomando::Add, &a)?;
    }
    // `--only`: el commit lleva solo estas rutas aunque haya otras en el
    // indice (RF-23).
    let mut c = vec!["-q", "-m", mensaje, "--only", "--"];
    c.extend_from_slice(&commit);
    exigir(git, dir, OP, Subcomando::Commit, &c)?;
    let head = commit_de(git, dir, OP, "HEAD")?.unwrap_or_default();
    Ok(Informe::nuevo(OP, Resultado::Ok)
        .con("commit", corto(&head))
        .con("ficheros", cambios.to_string())
        .con("sin-guardar", sin_guardar(git, dir)?.to_string()))
}

/// Reparte las rutas pedidas en las que hay que pasar al indice (`add`) y
/// las que entran en el commit:
/// - en disco: `add` y commit;
/// - borrada en el worktree sin pasar al indice (` D`): `add` y commit;
/// - borrada ya en el indice (`git rm`, origen de un `git mv`): solo commit,
///   porque `add` de una ruta que no esta ni en disco ni en el indice falla;
/// - desconocida para git y ausente del disco: se ignora (nada que guardar).
fn repartir<'a>(
    dir: &Path,
    rutas: &[&'a str],
    desde_raiz: &[String],
    status: &[(String, String)],
) -> (Vec<&'a str>, Vec<&'a str>) {
    let (mut add, mut commit) = (Vec::new(), Vec::new());
    for (r, raiz) in rutas.iter().zip(desde_raiz) {
        let estados: Vec<&str> = status
            .iter()
            .filter(|(_, s)| {
                s == raiz
                    || s.strip_prefix(raiz.as_str())
                        .is_some_and(|resto| resto.starts_with('/'))
            })
            .map(|(xy, _)| xy.as_str())
            .collect();
        if dir.join(r).symlink_metadata().is_ok() || estados.iter().any(|xy| xy.ends_with('D')) {
            add.push(*r);
            commit.push(*r);
        } else if !estados.is_empty() {
            commit.push(*r);
        }
    }
    (add, commit)
}

/// Como [`entradas_status`], pero un renombrado tambien aporta su ruta de
/// origen, como borrada en el indice (`D `). La copia (`C`) tambien trae el
/// origen, pero el origen sigue ahi.
fn entradas_con_origen(salida: &str) -> Vec<(String, String)> {
    let mut v = Vec::new();
    for (xy, ruta, origen) in registros_status(salida) {
        if let (Some(o), true) = (origen, xy.starts_with('R')) {
            v.push(("D ".to_owned(), o));
        }
        v.push((xy, ruta));
    }
    v
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
    use super::{entradas_con_origen, relativa_a_la_raiz};

    /// Un renombrado (`R`) aporta su origen como `D `; una copia (`C`), no.
    #[test]
    fn el_origen_cuenta_solo_en_los_renombrados() {
        let s = "R  nuevo.txt\0viejo.txt\0C  copia.txt\0orig.txt\0 M a.txt\0";
        let par = |xy: &str, r: &str| (xy.to_owned(), r.to_owned());
        assert_eq!(
            entradas_con_origen(s),
            vec![
                par("D ", "viejo.txt"),
                par("R ", "nuevo.txt"),
                par("C ", "copia.txt"),
                par(" M", "a.txt"),
            ]
        );
    }

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
