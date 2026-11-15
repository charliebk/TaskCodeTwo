//! Adaptador de GitHub del puerto `Plataforma` (F2-6, ADR F1-13 decisiones
//! 1 y 4; F1-6 decision 1 para la identidad de cada operacion).
//!
//! Cada endpoint es una variante de [`Endpoint`], que construye el metodo y
//! la ruta: no hay llamadas con una URL libre. Las respuestas son dato no
//! confiable (F1-8, Z2): se leen con structs que solo tienen los campos que
//! se usan y un campo ausente da `error motivo=respuesta-invalida`. El
//! transporte es el comun de [`super::conexion`].

use std::collections::BTreeMap;

use serde::Deserialize;

use super::conexion::{Conexion, rama_valida, rol_no_valido, sha_valido, uso};
use super::credenciales::{Credenciales, Rol};
use super::errores::{self, es_aviso_de_plan};
use super::http::{Base, Metodo, codificar};
use super::puertas::{self, EstadoCheck, Lecturas};
use super::{
    Firma, ModeloPuertas, Paso, Personas, Plataforma, Proveedor, ReglasLeidas, Usuario,
};
use crate::contrato::{Informe, Resultado, Valor, corto};

const ABRIR_PR: &str = "abrir-pr";
const MERGE_PR: &str = "merge-pr";
const LEER_PUERTAS: &str = "leer-puertas";
const LEER_PERMISOS_CI: &str = "leer-permisos-ci";
const FIRMA_COMMIT: &str = "firma-commit";
const IDENTIDAD: &str = "identidad";

/// Garantias que en GitHub dan los rulesets (tier «Free publico;
/// Pro/Team/Enterprise» de la tabla de F1-13): sin ellos, el 403 de plan.
const GARANTIAS_DE_RULESETS: [&str; 10] = [
    "G01", "G02", "G03", "G04", "G05", "G06", "G07", "G08", "G09", "G11",
];

/// Endpoints de la API que usa el adaptador (lista cerrada).
enum Endpoint<'a> {
    Usuario,
    Pr(u64),
    PrsAbiertos { cabeza: &'a str, base: &'a str },
    CrearPr,
    Merge(u64),
    ReglasRama(&'a str),
    Rulesets,
    CheckRuns(&'a str),
    Reviews(u64),
    PermisosWorkflow,
    Commit(&'a str),
}

impl Endpoint<'_> {
    fn metodo_y_ruta(&self, repo: &str) -> (Metodo, String) {
        let r = format!("/repos/{repo}");
        match self {
            Self::Usuario => (Metodo::Get, "/user".into()),
            Self::Pr(n) => (Metodo::Get, format!("{r}/pulls/{n}")),
            Self::PrsAbiertos { cabeza, base } => (
                Metodo::Get,
                format!(
                    "{r}/pulls?head={}&base={}&state=open&per_page=100",
                    codificar(cabeza),
                    codificar(base)
                ),
            ),
            Self::CrearPr => (Metodo::Post, format!("{r}/pulls")),
            Self::Merge(n) => (Metodo::Put, format!("{r}/pulls/{n}/merge")),
            Self::ReglasRama(rama) => (Metodo::Get, format!("{r}/rules/branches/{rama}")),
            Self::Rulesets => (Metodo::Get, format!("{r}/rulesets?per_page=100")),
            Self::CheckRuns(sha) => (
                Metodo::Get,
                format!("{r}/commits/{sha}/check-runs?per_page=100"),
            ),
            Self::Reviews(n) => (Metodo::Get, format!("{r}/pulls/{n}/reviews?per_page=100")),
            Self::PermisosWorkflow => (Metodo::Get, format!("{r}/actions/permissions/workflow")),
            Self::Commit(sha) => (Metodo::Get, format!("{r}/commits/{sha}")),
        }
    }
}

#[derive(Deserialize)]
struct UsuarioApi {
    #[serde(default)]
    login: String,
    id: u64,
    #[serde(rename = "type")]
    tipo: String,
}

impl From<UsuarioApi> for Usuario {
    fn from(u: UsuarioApi) -> Self {
        Self {
            login: u.login,
            id: u.id,
            tipo: u.tipo,
        }
    }
}

#[derive(Deserialize)]
struct RefApi {
    #[serde(rename = "ref")]
    nombre: String,
    #[serde(default)]
    sha: String,
}

#[derive(Deserialize)]
struct PrApi {
    number: u64,
    state: String,
    #[serde(default)]
    merged: bool,
    #[serde(default)]
    mergeable_state: Option<String>,
    head: RefApi,
    base: RefApi,
}

#[derive(Deserialize)]
struct CheckRunsApi {
    check_runs: Vec<CheckRunApi>,
}

#[derive(Deserialize)]
struct CheckRunApi {
    name: String,
    status: String,
    #[serde(default)]
    conclusion: Option<String>,
    #[serde(default)]
    app: Option<AppApi>,
}

#[derive(Deserialize)]
struct AppApi {
    id: u64,
}

#[derive(Deserialize)]
struct ReviewApi {
    state: String,
    #[serde(default)]
    user: Option<UsuarioApi>,
}

#[derive(Deserialize)]
struct MergeApi {
    merged: bool,
    #[serde(default)]
    sha: Option<String>,
}

#[derive(Deserialize)]
struct PermisosApi {
    can_approve_pull_request_reviews: bool,
    default_workflow_permissions: String,
}

#[derive(Deserialize)]
struct CommitApi {
    commit: CommitInternoApi,
    #[serde(default)]
    committer: Option<UsuarioApi>,
}

#[derive(Deserialize)]
struct CommitInternoApi {
    verification: VerificacionApi,
}

#[derive(Deserialize)]
struct VerificacionApi {
    verified: bool,
    #[serde(default)]
    reason: String,
}

#[derive(Deserialize)]
struct RulesetApi {
    name: String,
}

#[derive(Deserialize)]
struct ReglaApi {
    #[serde(rename = "type")]
    tipo: String,
}

/// Adaptador de GitHub para un repo.
pub struct GitHub {
    con: Conexion,
    repo: String,
}

impl std::fmt::Debug for GitHub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitHub")
            .field("con", &self.con)
            .field("repo", &self.repo)
            .finish_non_exhaustive()
    }
}

impl GitHub {
    /// Adaptador para `dueno/repo` en la API `base`.
    pub fn nuevo(
        base: Base,
        repo: &str,
        credenciales: Box<dyn Credenciales>,
        personas: Personas,
    ) -> Result<Self, Informe> {
        let partes: Vec<&str> = repo.split('/').collect();
        let valido = partes.len() == 2
            && partes.iter().all(|p| {
                !p.is_empty()
                    && *p != "."
                    && *p != ".."
                    && p.bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"-._".contains(&b))
            });
        if !valido {
            return Err(Informe::nuevo("plataforma", Resultado::Uso)
                .con("motivo", "repo-invalido")
                .diagnostico("el repo de GitHub tiene que ser dueno/repo"));
        }
        Ok(Self {
            con: Conexion::nueva(base, Proveedor::GitHub, credenciales, personas),
            repo: repo.to_owned(),
        })
    }

    /// Llamadas hechas, `METODO ruta` (para medir el coste por operacion).
    pub fn registro(&self) -> Vec<String> {
        self.con.registro()
    }

    fn ep(&self, ep: &Endpoint<'_>) -> (Metodo, String) {
        ep.metodo_y_ruta(&self.repo)
    }

    /// La identidad de persona tiene que estar en la lista de personas.
    fn exigir_persona(&self, op: &str) -> Paso<()> {
        self.con.exigir_persona(op, self.identidad(Rol::Persona))
    }

    fn pr_abierto(&self, rama: &str, base: &str) -> Paso<Option<u64>> {
        let cabeza = format!("{}:{rama}", self.dueno());
        let (_, ruta) = self.ep(&Endpoint::PrsAbiertos {
            cabeza: &cabeza,
            base,
        });
        let prs: Vec<PrApi> =
            self.con
                .leer_paginas(ABRIR_PR, Rol::Agente, "pull_requests:read", ruta, |c| {
                    self.con.parsear(ABRIR_PR, c)
                })?;
        Ok(prs
            .into_iter()
            .find(|p| p.head.nombre == rama && p.base.nombre == base)
            .map(|p| p.number))
    }

    fn dueno(&self) -> &str {
        self.repo.split('/').next().unwrap_or_default()
    }
}

impl Plataforma for GitHub {
    fn abrir_pr(&self, rama: &str, base: &str, titulo: &str, cuerpo: &str) -> Informe {
        if !rama_valida(rama) || !rama_valida(base) {
            return uso(ABRIR_PR, "rama-invalida", "nombre de rama no valido");
        }
        if titulo.trim().is_empty() || titulo.chars().any(char::is_control) {
            return uso(
                ABRIR_PR,
                "titulo-invalido",
                "el titulo no puede estar vacio ni llevar controles",
            );
        }
        match self.pr_abierto(rama, base) {
            Ok(Some(n)) => {
                return Informe::nuevo(ABRIR_PR, Resultado::SinCambios).con("pr", n.to_string());
            }
            Ok(None) => {}
            Err(i) => return i,
        }
        let datos = serde_json::json!({
            "title": titulo, "head": rama, "base": base, "body": cuerpo,
        })
        .to_string();
        let r = match self.con.pedir(
            ABRIR_PR,
            Rol::Agente,
            self.ep(&Endpoint::CrearPr),
            Some(&datos),
        ) {
            Ok(r) => r,
            Err(i) => return i,
        };
        if r.estado == 422 && r.cuerpo.contains("already exists") {
            // Otro proceso lo abrio entre la lectura y la escritura.
            return match self.pr_abierto(rama, base) {
                Ok(Some(n)) => {
                    Informe::nuevo(ABRIR_PR, Resultado::SinCambios).con("pr", n.to_string())
                }
                Ok(None) => self
                    .con
                    .traducir(ABRIR_PR, Rol::Agente, "pull_requests:write", &r),
                Err(i) => i,
            };
        }
        if !r.exito() {
            return self
                .con
                .traducir(ABRIR_PR, Rol::Agente, "pull_requests:write", &r);
        }
        match self.con.parsear::<PrApi>(ABRIR_PR, &r.cuerpo) {
            Ok(p) => Informe::nuevo(ABRIR_PR, Resultado::Ok).con("pr", p.number.to_string()),
            Err(i) => i,
        }
    }

    fn estado_puertas(&self, rol: Rol, pr: u64, modelo: &ModeloPuertas) -> Informe {
        let op = puertas::ESTADO_PUERTAS;
        if rol == Rol::Revisor {
            return rol_no_valido(op, rol);
        }
        if rol == Rol::Persona
            && let Err(i) = self.exigir_persona(op)
        {
            return i;
        }
        let permiso = "pull_requests:read y checks:read";
        let p: PrApi = match self.con.leer(op, rol, permiso, self.ep(&Endpoint::Pr(pr))) {
            Ok(p) => p,
            Err(i) => return i,
        };
        if p.merged {
            return puertas::mergeado(pr);
        }
        if p.state != "open" {
            return puertas::pr_cerrado(pr);
        }
        if !rama_valida(&p.base.nombre) || !sha_valido(&p.head.sha) {
            return self.con.invalida(op, "rama base o sha del head");
        }
        let r = match self
            .con
            .pedir(op, rol, self.ep(&Endpoint::ReglasRama(&p.base.nombre)), None)
        {
            Ok(r) => r,
            Err(i) => return i,
        };
        let plan_da_reglas = !es_aviso_de_plan(&r);
        if plan_da_reglas && !r.exito() {
            return self.con.traducir(op, rol, permiso, &r);
        }
        if !plan_da_reglas
            && let Some(g) = modelo
                .exigir_impuestas
                .iter()
                .find(|g| GARANTIAS_DE_RULESETS.contains(&g.as_str()))
        {
            return puertas::rechazada_por_plan(pr, g);
        }
        let (_, ruta) = self.ep(&Endpoint::CheckRuns(&p.head.sha));
        let runs: Vec<CheckRunApi> = match self.con.leer_paginas(op, rol, permiso, ruta, |c| {
            self.con
                .parsear::<CheckRunsApi>(op, c)
                .map(|x| x.check_runs)
        }) {
            Ok(v) => v,
            Err(i) => return i,
        };
        // La API da los check-runs del mas nuevo al mas viejo: cuenta el
        // primero de cada nombre que publico su origen (integration_id).
        let checks = modelo
            .checks
            .iter()
            .map(|(nombre, origen)| {
                let estado = runs
                    .iter()
                    .find(|r| &r.name == nombre && r.app.as_ref().is_some_and(|a| a.id == *origen))
                    .map_or(EstadoCheck::Pendiente, |r| {
                        if r.status != "completed" {
                            EstadoCheck::Pendiente
                        } else if matches!(
                            r.conclusion.as_deref(),
                            Some("success" | "neutral" | "skipped")
                        ) {
                            // GitHub da por cumplido un check obligatorio
                            // que acaba neutral o saltado.
                            EstadoCheck::Ok
                        } else {
                            EstadoCheck::Fallo
                        }
                    });
                (nombre.clone(), estado)
            })
            .collect();
        let (_, ruta) = self.ep(&Endpoint::Reviews(pr));
        let reviews: Vec<ReviewApi> =
            match self
                .con
                .leer_paginas(op, rol, permiso, ruta, |c| self.con.parsear(op, c))
            {
                Ok(v) => v,
                Err(i) => return i,
            };
        // Las reviews vienen de la mas vieja a la mas nueva: cuenta la ultima
        // de cada cuenta, por id (F1-6, regla 6). Solo cambian el estado las
        // que deciden: un comentario o una review pendiente no retiran una
        // aprobacion anterior (como en GitHub).
        let mut ultima: BTreeMap<u64, (String, Usuario)> = BTreeMap::new();
        for rv in reviews {
            let decide = matches!(
                rv.state.as_str(),
                "APPROVED" | "CHANGES_REQUESTED" | "DISMISSED"
            );
            if let Some(u) = rv.user.filter(|_| decide) {
                ultima.insert(u.id, (rv.state, u.into()));
            }
        }
        let aprobadas: Vec<&Usuario> = ultima
            .values()
            .filter(|(s, _)| s == "APPROVED")
            .map(|(_, u)| u)
            .collect();
        let aprobaciones = aprobadas
            .iter()
            .filter(|u| u.es_persona(self.con.personas()))
            .count();
        puertas::evaluar(&Lecturas {
            pr,
            checks,
            aprobaciones,
            ignoradas: aprobadas.len() - aprobaciones,
            al_dia: p.mergeable_state.as_deref() != Some("behind"),
            // `unknown` (GitHub aun calculando) no se da por conflicto:
            // `merge-pr` lo vuelve a comprobar en la plataforma (405).
            sin_conflictos: p.mergeable_state.as_deref() != Some("dirty"),
        })
    }

    fn merge_pr(&self, pr: u64, sha: &str) -> Informe {
        if !sha_valido(sha) {
            return uso(
                MERGE_PR,
                "sha-invalido",
                "el sha esperado tiene que ser completo (40 hex)",
            );
        }
        if let Err(i) = self.exigir_persona(MERGE_PR) {
            return i;
        }
        let permiso = "contents:write";
        let p: PrApi = match self
            .con
            .leer(MERGE_PR, Rol::Persona, permiso, self.ep(&Endpoint::Pr(pr)))
        {
            Ok(p) => p,
            Err(i) => return i,
        };
        if p.merged {
            return Informe::nuevo(MERGE_PR, Resultado::SinCambios)
                .con("pr", pr.to_string())
                .con("estado", "mergeado");
        }
        if p.state != "open" {
            return Informe::nuevo(MERGE_PR, Resultado::Rechazada)
                .con("pr", pr.to_string())
                .con("motivo", "pr-cerrado")
                .diagnostico("el PR esta cerrado sin mergear");
        }
        if !p.head.sha.eq_ignore_ascii_case(sha) {
            return head_cambiado(pr);
        }
        let datos = serde_json::json!({ "merge_method": "merge", "sha": sha }).to_string();
        let r = match self.con.pedir(
            MERGE_PR,
            Rol::Persona,
            self.ep(&Endpoint::Merge(pr)),
            Some(&datos),
        ) {
            Ok(r) => r,
            Err(i) => return i,
        };
        if !r.exito() {
            let mut i = self.con.traducir(MERGE_PR, Rol::Persona, permiso, &r);
            i.claves
                .insert(0, ("pr".to_owned(), Valor::Texto(pr.to_string())));
            return i;
        }
        match self.con.parsear::<MergeApi>(MERGE_PR, &r.cuerpo) {
            Ok(MergeApi {
                merged: true,
                sha: Some(s),
            }) if sha_valido(&s) => Informe::nuevo(MERGE_PR, Resultado::Ok)
                .con("pr", pr.to_string())
                .con("commit", corto(&s)),
            // Mergeado aunque la respuesta no traiga un sha legible: se dice
            // que esta hecho, sin inventar el commit.
            Ok(m) if m.merged => Informe::nuevo(MERGE_PR, Resultado::Ok).con("pr", pr.to_string()),
            Ok(_) => self.con.invalida(MERGE_PR, "merged"),
            Err(i) => i,
        }
    }

    fn leer_puertas(&self, rol: Rol, rama: &str) -> Paso<ReglasLeidas> {
        if !rama_valida(rama) {
            return Err(uso(
                LEER_PUERTAS,
                "rama-invalida",
                "nombre de rama no valido",
            ));
        }
        if rol == Rol::Revisor {
            return Err(rol_no_valido(LEER_PUERTAS, rol));
        }
        let permiso = "metadata:read";
        let r = self
            .con
            .pedir(LEER_PUERTAS, rol, self.ep(&Endpoint::Rulesets), None)?;
        if es_aviso_de_plan(&r) {
            return Ok(ReglasLeidas {
                plan_da_reglas: false,
                ..ReglasLeidas::default()
            });
        }
        if !r.exito() {
            return Err(self.con.traducir(LEER_PUERTAS, rol, permiso, &r));
        }
        let rulesets: Vec<RulesetApi> = self.con.parsear(LEER_PUERTAS, &r.cuerpo)?;
        let reglas: Vec<ReglaApi> = self.con.leer(
            LEER_PUERTAS,
            rol,
            permiso,
            self.ep(&Endpoint::ReglasRama(rama)),
        )?;
        Ok(ReglasLeidas {
            plan_da_reglas: true,
            rulesets: rulesets.into_iter().map(|r| r.name).collect(),
            reglas_rama: reglas.into_iter().map(|r| r.tipo).collect(),
        })
    }

    fn leer_permisos_ci(&self) -> Informe {
        if let Err(i) = self.exigir_persona(LEER_PERMISOS_CI) {
            return i;
        }
        let p: PermisosApi = match self.con.leer(
            LEER_PERMISOS_CI,
            Rol::Persona,
            "administration:read",
            self.ep(&Endpoint::PermisosWorkflow),
        ) {
            Ok(p) => p,
            Err(i) => return i,
        };
        let permisos = errores::sanear(&p.default_workflow_permissions);
        if p.can_approve_pull_request_reviews {
            Informe::nuevo(LEER_PERMISOS_CI, Resultado::Rechazada)
                .con("motivo", "ci-puede-aprobar")
                .con("permisos", permisos)
                .diagnostico("el token de Actions puede aprobar PR: desactiva «Allow GitHub Actions to create and approve pull requests» en Settings > Actions > General (G16)")
        } else {
            Informe::nuevo(LEER_PERMISOS_CI, Resultado::Ok)
                .con("ci-aprueba", "false")
                .con("permisos", permisos)
        }
    }

    fn firma_commit(&self, sha: &str) -> Paso<Firma> {
        if !sha_valido(sha) {
            return Err(uso(
                FIRMA_COMMIT,
                "sha-invalido",
                "el sha tiene que ser completo (40 hex)",
            ));
        }
        let c: CommitApi = self.con.leer(
            FIRMA_COMMIT,
            Rol::Agente,
            "contents:read",
            self.ep(&Endpoint::Commit(sha)),
        )?;
        Ok(Firma {
            verificada: c.commit.verification.verified,
            motivo: errores::sanear(&c.commit.verification.reason),
            committer: c.committer.map(Usuario::from),
            clave: None,
        })
    }

    fn identidad(&self, rol: Rol) -> Paso<Usuario> {
        self.con.identidad(rol, || {
            self.con
                .leer::<UsuarioApi>(IDENTIDAD, rol, "(ninguno)", self.ep(&Endpoint::Usuario))
                .map(Usuario::from)
        })
    }
}

/// `merge-pr` rechazado porque el head no es el evaluado.
pub(super) fn head_cambiado(pr: u64) -> Informe {
    Informe::nuevo(MERGE_PR, Resultado::Rechazada)
        .con("pr", pr.to_string())
        .con("motivo", "head-cambiado")
        .diagnostico(
            "la rama ha cambiado desde que se evaluaron las puertas: vuelve a lanzar estado-puertas",
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_invalido_es_uso() {
        for repo in ["o", "o/r/s", "o/..", "o/r?x", "/r"] {
            let r = GitHub::nuevo(
                Base::github(),
                repo,
                Box::new(super::super::CredencialesFijas::nuevas()),
                Personas::default(),
            );
            assert!(r.is_err_and(|i| i.resultado == Resultado::Uso), "{repo}");
        }
    }
}
