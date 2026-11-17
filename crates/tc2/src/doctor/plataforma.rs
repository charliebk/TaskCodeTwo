//! Lo que se lee de la plataforma: la proteccion de las ramas del modelo con
//! `solo_pr` (`develop` y `main`) comparada con
//! [`crate::plataforma::puertas::modelo`], y los permisos (la lectura de las
//! puertas y los de la CI). Solo lecturas del puerto (`GET`).
//!
//! Sin plataforma, sin credenciales o sin red, todos los hallazgos de aqui
//! salen `no-comprobado`: nunca se da por buena una proteccion sin leerla.

use super::{ARREGLO_INIT, Entorno, Estado, Hallazgo};
use crate::contrato::{Informe, Resultado};
use crate::identidad::Consulta;
use crate::plataforma::puertas::{Objetivo, modelo};
use crate::plataforma::{Paso, Plataforma, Proveedor, ReglasLeidas, github, gitlab};

/// Lo que se comprueba aqui, para decir `no-comprobado` de todo a la vez.
const COMPROBACIONES: [(&str, &str); 3] = [
    ("proteccion", "develop"),
    ("proteccion", "main"),
    ("permisos", "ci"),
];

/// Las ramas del modelo que solo admiten PR (F1-5 decision 2).
pub fn ramas_solo_pr() -> Vec<(&'static str, String)> {
    modelo()
        .into_iter()
        .filter(|p| p.solo_pr.is_some())
        .filter_map(|p| match &p.objetivo {
            Objetivo::Ramas(r) if r.len() == 1 => r[0]
                .strip_prefix("refs/heads/")
                .map(|x| (p.nombre, x.to_owned())),
            _ => None,
        })
        .collect()
}

/// Si la rama solo admite cambios por PR o MR, con lo leido (G02): en
/// GitHub, la regla `pull_request`; en GitLab, protegida y con «Allowed to
/// push» a nadie (`push=0` y ningun otro nivel).
pub fn solo_pr(p: Proveedor, r: &ReglasLeidas) -> bool {
    match p {
        Proveedor::GitHub => r.reglas_rama.iter().any(|x| x == "pull_request"),
        Proveedor::GitLab => {
            let push: Vec<&String> = r
                .reglas_rama
                .iter()
                .filter(|x| x.starts_with("push="))
                .collect();
            !push.is_empty() && push.iter().all(|x| *x == "push=0")
        }
    }
}

/// El permiso de una lectura, para `permiso=`.
fn permiso(p: Option<Proveedor>, op: &str) -> &'static str {
    match (p, op) {
        (Some(Proveedor::GitHub), "leer-permisos-ci") => github::PERMISO_PERMISOS_CI,
        (Some(Proveedor::GitHub), _) => github::PERMISO_LEER_PUERTAS,
        _ => gitlab::PERMISO_LECTURA_CORTO,
    }
}

/// `no-comprobado` de todo lo de la plataforma, con `motivo`.
fn sin_plataforma(motivo: &str) -> Vec<Hallazgo> {
    COMPROBACIONES
        .iter()
        .map(|(a, id)| Hallazgo::nuevo(a, *id, Estado::NoComprobado).motivo(motivo))
        .collect()
}

/// Si un fallo es un 403 de permisos (`denegada motivo=permisos`).
fn es_de_permisos(i: &Informe) -> bool {
    i.resultado == Resultado::Denegada && i.valor("motivo") == Some("permisos")
}

/// El hallazgo de la proteccion de `rama` (funcion pura).
pub fn proteccion(
    p: Option<Proveedor>,
    nombre: &str,
    rama: &str,
    leidas: &Paso<ReglasLeidas>,
) -> Hallazgo {
    let h = |e: Estado| Hallazgo::nuevo("proteccion", rama, e);
    let r = match leidas {
        Ok(r) => r,
        Err(i) if es_de_permisos(i) => {
            return h(Estado::NoComprobado).motivo("permiso-insuficiente");
        }
        Err(i) => {
            return h(Estado::NoComprobado).motivo(
                i.valor("motivo")
                    .unwrap_or_else(|| i.resultado.texto())
                    .to_owned(),
            );
        }
    };
    let Some(p) = p else {
        return h(Estado::NoComprobado).motivo("plataforma-desconocida");
    };
    if p == Proveedor::GitHub && !r.plan_da_reglas {
        // D2a de F1-5: el plan no da rulesets; lo dice `capacidades`.
        return h(Estado::Aviso)
            .motivo("plan")
            .arreglo("repo-publico-o-github-pro");
    }
    let ruleset = format!("{}{nombre}", crate::huella::PREFIJO_RULESETS);
    let falta_ruleset = p == Proveedor::GitHub && !r.rulesets.contains(&ruleset);
    if falta_ruleset || !solo_pr(p, r) {
        return h(Estado::Falla)
            .motivo("proteccion-ausente")
            .arreglo(ARREGLO_INIT);
    }
    h(Estado::Ok)
}

/// El hallazgo de los permisos de la CI (funcion pura).
pub fn permisos_ci(p: Option<Proveedor>, i: &Informe) -> Hallazgo {
    let h = |e: Estado| Hallazgo::nuevo("permisos", "ci", e);
    let motivo = i.valor("motivo");
    match (i.resultado, motivo) {
        (Resultado::Ok, _) => h(Estado::Ok),
        (Resultado::Rechazada, Some("ci-puede-aprobar")) => h(Estado::Falla)
            .motivo("ci-puede-aprobar")
            .arreglo("desactivar-aprobacion-por-actions"),
        (Resultado::Rechazada, Some("identidad")) if i.valor("falta").is_some() => {
            h(Estado::NoComprobado).motivo("sin-token-persona")
        }
        (Resultado::Rechazada, Some("identidad")) => h(Estado::Falla)
            .motivo("persona-fuera-de-lista")
            .arreglo(crate::plataforma::personas::RUTA),
        _ if es_de_permisos(i) => h(Estado::Falla)
            .motivo("permiso-insuficiente")
            .con("permiso", permiso(p, "leer-permisos-ci"))
            .con("rol", "persona"),
        (Resultado::Sinred, _) => h(Estado::NoComprobado).motivo("sin-red"),
        (Resultado::Denegada, Some("credenciales")) => h(Estado::Falla)
            .motivo("credenciales")
            .con("rol", "persona"),
        (r, m) => h(Estado::Falla).motivo(m.unwrap_or_else(|| r.texto()).to_owned()),
    }
}

/// Todo lo de la plataforma: hallazgos, `plataforma=` y las cuentas de
/// `capacidades`.
pub(super) fn comprobar(
    e: &Entorno<'_>,
) -> (Vec<Hallazgo>, String, Option<super::capacidades::Cuentas>) {
    let (personas, _) = crate::roadmap::personas_y_aviso(e.git, e.dir);
    let c = match (e.conectar)(&personas) {
        Ok(c) => c,
        Err(consulta) => {
            let n = consulta.nombre();
            return (sin_plataforma(n), n.to_owned(), None);
        }
    };
    let p: &dyn Plataforma = c.plataforma.as_ref();
    let prov = p.proveedor();
    let ramas = ramas_solo_pr();
    let mut leidas: Vec<Paso<ReglasLeidas>> = Vec::new();
    for (_, rama) in &ramas {
        let r = p.leer_puertas(c.rol, rama);
        // Sin red o sin credenciales en la primera lectura no se sigue: lo
        // de la plataforma queda `no-comprobado`.
        if let Err(i) = &r
            && leidas.is_empty()
            && matches!(
                Consulta::de_fallo(i),
                Consulta::SinRed | Consulta::SinCredenciales
            )
        {
            let n = Consulta::de_fallo(i).nombre();
            return (sin_plataforma(n), n.to_owned(), None);
        }
        leidas.push(r);
    }
    let mut hs = Vec::new();
    let mut plataforma = "ok";
    for ((nombre, rama), r) in ramas.iter().zip(&leidas) {
        hs.push(proteccion(prov, nombre, rama, r));
    }
    // Un 403 al leer las puertas: una sola falla con su permiso.
    if let Some(i) = leidas.iter().filter_map(|r| r.as_ref().err()).next() {
        if es_de_permisos(i) {
            hs.push(
                Hallazgo::nuevo("permisos", "leer-puertas", Estado::Falla)
                    .motivo("permiso-insuficiente")
                    .con("permiso", permiso(prov, "leer-puertas"))
                    .con("rol", c.rol.nombre()),
            );
        } else {
            plataforma = Consulta::de_fallo(i).nombre();
        }
    }
    hs.push(permisos_ci(prov, &p.leer_permisos_ci()));
    (hs, plataforma.to_owned(), None)
}
