//! Comprobaciones del entorno: `git` y su version (D3a: `aviso` por debajo de
//! 2.56, `falla` solo sin git o con una version que no se entiende), el
//! `tc2` que corre, estar en un repo y la configuracion de la base.

use std::path::Path;

use super::{ARREGLO_INIT, Estado, Hallazgo, NOMBRE};
use crate::config;
use crate::contrato::Informe;
use crate::git::{ErrorGit, Git, SalidaGit, Subcomando, localizar_principal};

/// Motivo sin `git`.
pub(super) const SIN_GIT: &str = "sin-git";

/// La version de git con la que se ha verificado todo (F1-1, adenda de
/// F2-3; F1-4 «salida de git entre versiones»).
pub const GIT_VERIFICADO: (u32, u32, u32) = (2, 56, 0);

/// `(mayor, menor, parche)` de la salida de `git version` (`git version
/// 2.56.0.windows.4`); `None` si no se entiende.
pub fn version_de_git(salida: &str) -> Option<(u32, u32, u32)> {
    let resto = salida.trim().strip_prefix("git version ")?;
    let mut partes = resto.split(['.', ' ', '-']);
    let mut num = || partes.next().and_then(|p| p.parse::<u32>().ok());
    let mayor = num()?;
    let menor = num()?;
    Some((mayor, menor, num().unwrap_or(0)))
}

/// El hallazgo de `git` con lo que dio `git version` (funcion pura).
pub fn hallazgo_git(r: &Result<SalidaGit, ErrorGit>) -> Hallazgo {
    let h = Hallazgo::nuevo("entorno", "git", Estado::Ok);
    let s = match r {
        Err(_) => {
            return Hallazgo {
                estado: Estado::Falla,
                ..h
            }
            .motivo(SIN_GIT)
            .arreglo("instalar-git");
        }
        Ok(s) => s,
    };
    let version = s.exito().then(|| version_de_git(&s.stdout)).flatten();
    match version {
        None => Hallazgo {
            estado: Estado::Falla,
            ..h
        }
        .motivo("git-irreconocible")
        .arreglo("instalar-git"),
        Some(v) => {
            let texto = format!("{}.{}.{}", v.0, v.1, v.2);
            if v < GIT_VERIFICADO {
                Hallazgo {
                    estado: Estado::Aviso,
                    ..h
                }
                .motivo("git-no-verificado")
                .con("version", texto)
                .arreglo("git-2.56-o-posterior")
            } else {
                h.con("version", texto)
            }
        }
    }
}

/// `git version` con el puerto.
pub(super) fn git(g: &Git, dir: &Path) -> Hallazgo {
    hallazgo_git(&g.ejecutar(dir, Subcomando::Version, &[]))
}

/// El `tc2` que corre y su protocolo (siempre `ok`: es informativo).
pub(super) fn tc2() -> Hallazgo {
    Hallazgo::nuevo("entorno", "tc2", Estado::Ok)
        .con("version", crate::VERSION)
        .con("protocolo", crate::PROTOCOLO.to_string())
}

/// Fuera de un repo, `doctor rechazada motivo=no-es-repo` con el
/// diagnostico de la capa git.
pub(super) fn repo(g: &Git, dir: &Path) -> Result<(), Informe> {
    localizar_principal(g, dir).map(|_| ()).map_err(|mut i| {
        NOMBRE.clone_into(&mut i.comando);
        i
    })
}

/// La configuracion de la base (`origin/develop`, F1-10 decision 3).
pub(super) fn config(g: &Git, dir: &Path) -> Hallazgo {
    let h = |e: Estado| Hallazgo::nuevo("entorno", "config", e);
    match config::de_base(g, dir) {
        Ok(_) => h(Estado::Ok),
        Err(i) => match i.valor("motivo") {
            Some("sin-base") => h(Estado::Aviso)
                .motivo("sin-base")
                .arreglo("git fetch origin develop"),
            Some("sin-config") => h(Estado::Aviso).motivo("sin-config").arreglo(ARREGLO_INIT),
            Some("formato") => h(Estado::Falla)
                .motivo("proyecto-016")
                .arreglo(crate::init::ARREGLO_016),
            Some(m) => h(Estado::Falla).motivo(m.to_owned()),
            None => h(Estado::NoComprobado).motivo(i.resultado.texto()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn salida(codigo: i32, stdout: &str) -> Result<SalidaGit, ErrorGit> {
        Ok(SalidaGit {
            codigo,
            stdout: stdout.to_owned(),
            stderr: String::new(),
        })
    }

    #[test]
    fn version_de_git_se_lee() {
        assert_eq!(
            version_de_git("git version 2.56.0.windows.4\n"),
            Some((2, 56, 0))
        );
        assert_eq!(version_de_git("git version 2.43.0"), Some((2, 43, 0)));
        assert_eq!(version_de_git("git version 3.0"), Some((3, 0, 0)));
        for malo in ["", "git version", "git version x.y", "hg version 2.56.0"] {
            assert_eq!(version_de_git(malo), None, "{malo}");
        }
    }

    /// C6 (parte pura): sin git o irreconocible, `falla`; por debajo de
    /// 2.56, `aviso motivo=git-no-verificado`; 2.56 o mas, `ok`.
    #[test]
    fn git_falla_sin_binario_y_avisa_por_debajo_de_2_56() {
        let l = |r: Result<SalidaGit, ErrorGit>| hallazgo_git(&r).linea();
        assert_eq!(
            l(Err(ErrorGit::Lanzar("no existe".into()))),
            "entorno git falla motivo=sin-git arreglo=instalar-git"
        );
        assert_eq!(
            l(salida(0, "otra cosa\n")),
            "entorno git falla motivo=git-irreconocible arreglo=instalar-git"
        );
        assert_eq!(
            l(salida(1, "git version 2.56.0\n")),
            "entorno git falla motivo=git-irreconocible arreglo=instalar-git"
        );
        assert_eq!(
            l(salida(0, "git version 2.55.9\n")),
            "entorno git aviso motivo=git-no-verificado version=2.55.9 arreglo=git-2.56-o-posterior"
        );
        assert_eq!(
            l(salida(0, "git version 2.56.0.windows.4\n")),
            "entorno git ok version=2.56.0"
        );
    }
}
