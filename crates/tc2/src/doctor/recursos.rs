//! `tc2 doctor --recursos` (F3-9; ADR F1-14 punto 8): el inventario de
//! skills y agentes de la maquina y del proyecto y el manifiesto de recursos,
//! como hallazgos del area `recursos`.
//!
//! Lee, sin escribir nada: la carpeta de Claude Code (la que da
//! `CLAUDE_CONFIG_DIR` o `~/.claude`), `.claude/` del proyecto, `recursos:`
//! de la configuracion de la base (`origin/develop`) y `roles.*` de las
//! tareas no terminadas de la rama de estado (lo ultimo traido, sin
//! `fetch`). Los ejecutables se buscan en el PATH.
//!
//! [`hallazgo`] adapta un [`Recurso`] de [`crate::recursos`] a un
//! [`Hallazgo`]: `recursos <id> <estado> [motivo=] tipo= [tarea=] [nivel=]
//! [version=] [arreglo=]`. `nivel=` solo se escribe si no es `necesario`
//! (el de lo que declara el proyecto o la tarea).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::{Entorno, Estado, Hallazgo, NOMBRE, entorno};
use crate::contrato::Informe;
use crate::estado::Estado as _;
use crate::git::{Subcomando, correr};
use crate::grafo::{DESCARTADA, TERMINADA};
use crate::inventario::{self, Inventario};
use crate::recursos::{
    self, Fuente, Manifiesto, Momento, NO_INSTALADO, Nivel, Recomendado, Recurso, TipoRecurso,
};
use crate::tarea::Documento;

/// Area de los hallazgos.
pub const AREA: &str = "recursos";

/// Arreglo del hallazgo agregado del `doctor` normal.
pub const ARREGLO: &str = "tc2 doctor --recursos";

/// Ejecutables convenientes que se buscan siempre (F1-14, punto 8).
const CONVENIENTES: [&str; 1] = ["rtk"];

/// El hallazgo de un recurso evaluado (funcion pura).
pub fn hallazgo(x: &Recurso) -> Hallazgo {
    let estado = match x.estado {
        recursos::Estado::Falla => Estado::Falla,
        recursos::Estado::Aviso => Estado::Aviso,
        recursos::Estado::NoComprobado => Estado::NoComprobado,
        recursos::Estado::Ok => Estado::Ok,
    };
    let mut h = Hallazgo::nuevo(AREA, x.id.clone(), estado);
    if let Some(c) = x.comprobacion {
        h = h.motivo(c);
    }
    h = h.con("tipo", x.tipo);
    if let Some(t) = &x.tarea {
        h = h.con("tarea", t.clone());
    }
    if x.nivel != Nivel::Necesario {
        h = h.con("nivel", x.nivel.texto());
    }
    if let Some(v) = &x.version {
        h = h.con("version", v.clone());
    }
    if let Some(a) = &x.arreglo {
        h = h.arreglo(a.clone());
    }
    h
}

/// La raiz del worktree desde el que se lanza (donde esta `.claude/` del
/// proyecto). Fuera de un repo, el rechazo de `doctor`.
fn raiz(e: &Entorno<'_>) -> Result<PathBuf, Informe> {
    entorno::repo(e.git, e.dir)?;
    let s = correr(
        e.git,
        e.dir,
        NOMBRE,
        Subcomando::RevParse,
        &["--show-toplevel"],
    )?;
    let t = s.stdout.trim();
    Ok(if s.exito() && !t.is_empty() {
        PathBuf::from(t)
    } else {
        e.dir.to_path_buf()
    })
}

/// Un hallazgo `no-comprobado` de una fuente de recomendaciones.
fn sin_leer(id: &str, i: &Informe) -> Hallazgo {
    Hallazgo::nuevo(AREA, id, Estado::NoComprobado).motivo(
        i.valor("motivo")
            .map_or_else(|| i.resultado.texto().to_owned(), str::to_owned),
    )
}

/// Lo que recomiendan `recursos:` de la configuracion y `roles.*` de las
/// tareas no terminadas (D1a), con lo que no se pudo leer.
fn recomendados(e: &Entorno<'_>) -> (Vec<Recomendado>, Vec<Hallazgo>) {
    let mut v = Vec::new();
    let mut hs = Vec::new();
    match crate::config::de_base(e.git, e.dir) {
        Ok(c) => {
            for r in c.recursos {
                if let Some(tipo) = TipoRecurso::de_texto(&r.tipo) {
                    v.push(Recomendado {
                        id: r.id,
                        tipo,
                        fuente: Fuente::Config,
                    });
                }
            }
        }
        // Sin configuracion no hay `recursos:` que comprobar.
        Err(i) if i.valor("motivo") == Some("sin-config") => {}
        Err(i) => hs.push(sin_leer("config", &i)),
    }
    let vista = match crate::estado::leer(e.git, e.dir) {
        Ok(v) => v,
        Err(i) => {
            hs.push(sin_leer("tareas", &i));
            return (v, hs);
        }
    };
    let leidas = match crate::estado::tareas(&vista, &vista.rutas()) {
        Ok(l) => l,
        Err(_) => {
            hs.push(
                Hallazgo::nuevo(AREA, "tareas", Estado::NoComprobado).motivo("estado-ilegible"),
            );
            return (v, hs);
        }
    };
    for (_, l) in leidas {
        // Las ilegibles ya las cuenta `doctor` en `estado tareas`; las de la
        // 0.16 no tienen `roles`.
        let Ok(Documento::Formato2(t, _)) = l else {
            continue;
        };
        if t.estado == TERMINADA || t.estado == DESCARTADA {
            continue;
        }
        let fuente = Fuente::Tarea(t.id.clone());
        let r = &t.roles;
        for s in r
            .diseno
            .skills
            .iter()
            .chain(&r.implementacion.skills)
            .chain(&r.revision.skills)
        {
            v.push(Recomendado {
                id: s.clone(),
                tipo: TipoRecurso::Skill,
                fuente: fuente.clone(),
            });
        }
        if let Some(a) = &r.revision.agente {
            v.push(Recomendado {
                id: a.clone(),
                tipo: TipoRecurso::Agente,
                fuente,
            });
        }
    }
    (v, hs)
}

/// Si hay un ejecutable `nombre` en alguna carpeta del PATH (en Windows,
/// con las extensiones de `PATHEXT`).
fn en_path(nombre: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    let exts: Vec<String> = if cfg!(windows) {
        std::env::var("PATHEXT")
            .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_owned())
            .split(';')
            .filter(|x| !x.is_empty())
            .map(str::to_owned)
            .collect()
    } else {
        vec![String::new()]
    };
    std::env::split_paths(&path).any(|d| {
        exts.iter()
            .any(|x| d.join(format!("{nombre}{x}")).is_file())
    })
}

/// Todos los hallazgos de recursos y el inventario. `Err` fuera de un repo.
pub fn comprobar(e: &Entorno<'_>) -> Result<(Vec<Hallazgo>, Inventario), Informe> {
    let raiz = raiz(e)?;
    comprobar_en(e, &raiz)
}

fn comprobar_en(e: &Entorno<'_>, raiz: &Path) -> Result<(Vec<Hallazgo>, Inventario), Informe> {
    let mut hs = Vec::new();
    let lectura = inventario::leer(e.claude, raiz);
    if e.claude.is_none() {
        hs.push(
            Hallazgo::nuevo(AREA, "maquina", Estado::NoComprobado)
                .motivo("sin-carpeta-de-claude")
                .arreglo("definir-CLAUDE_CONFIG_DIR-o-HOME"),
        );
    }
    for i in &lectura.ilegibles {
        hs.push(
            Hallazgo::nuevo(AREA, i.id.clone(), Estado::NoComprobado)
                .motivo(i.motivo)
                .con("ruta", i.ruta.clone()),
        );
    }
    for i in &lectura.avisos {
        hs.push(
            Hallazgo::nuevo(AREA, i.id.clone(), Estado::Aviso)
                .motivo(i.motivo)
                .con("ruta", i.ruta.clone()),
        );
    }
    let (rec, h) = recomendados(e);
    hs.extend(h);
    let buscar: BTreeSet<String> = rec
        .iter()
        .filter(|r| r.tipo == TipoRecurso::Ejecutable)
        .map(|r| r.id.clone())
        .chain(CONVENIENTES.iter().map(|x| (*x).to_owned()))
        .collect();
    let m = Manifiesto {
        recomendados: rec,
        ejecutables: buscar.into_iter().filter(|n| en_path(n)).collect(),
        mcp: lectura.mcp.clone(),
        completo: e.claude.is_some() && lectura.ilegibles.is_empty(),
    };
    let informe = recursos::evaluar_recursos(&lectura.inventario, &m, &Momento::Doctor);
    hs.extend(informe.recursos.iter().map(hallazgo));
    Ok((hs, lectura.inventario))
}

/// El hallazgo agregado del `doctor` normal: `falla ids=N` con lo que no
/// esta instalado, `no-comprobado` si algo no se pudo leer, `ok` si no.
pub fn agregado(e: &Entorno<'_>) -> Hallazgo {
    let h = |x: Estado| Hallazgo::nuevo(AREA, "no-instalados", x);
    match comprobar(e) {
        Ok((hs, _)) => {
            let n = hs
                .iter()
                .filter(|h| h.estado == Estado::Falla && h.motivo.as_deref() == Some(NO_INSTALADO))
                .count();
            if n > 0 {
                h(Estado::Falla).con("ids", n.to_string()).arreglo(ARREGLO)
            } else if hs.iter().any(|h| h.estado == Estado::NoComprobado) {
                h(Estado::NoComprobado)
                    .motivo("recursos-sin-comprobar")
                    .arreglo(ARREGLO)
            } else {
                h(Estado::Ok)
            }
        }
        Err(i) => sin_leer("no-instalados", &i),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// C2 (parte pura): la linea de un recomendado que falta.
    #[test]
    fn la_linea_de_un_recurso() {
        let r = Recurso {
            id: "fantasma".to_owned(),
            tipo: "skill",
            nivel: Nivel::Necesario,
            estado: recursos::Estado::Falla,
            comprobacion: Some(NO_INSTALADO),
            tarea: None,
            version: None,
            arreglo: Some("instalar-skill-o-quitarla-de-recursos".to_owned()),
        };
        assert_eq!(
            hallazgo(&r).linea(),
            "recursos fantasma falla motivo=no-instalado tipo=skill arreglo=instalar-skill-o-quitarla-de-recursos"
        );
        let r = Recurso {
            nivel: Nivel::Estorba,
            estado: recursos::Estado::Aviso,
            tarea: Some("TASK-001".to_owned()),
            version: Some("0.16.1".to_owned()),
            ..r
        };
        assert_eq!(
            hallazgo(&r).linea(),
            "recursos fantasma aviso motivo=no-instalado tipo=skill tarea=TASK-001 nivel=estorba version=0.16.1 arreglo=instalar-skill-o-quitarla-de-recursos"
        );
    }
}
