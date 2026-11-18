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

use super::conexion::{Conexion, head_cambiado, rama_valida, rol_no_valido, sha_valido, uso};
use super::credenciales::{Credenciales, Rol};
use super::errores::{self, es_aviso_de_plan};
use super::http::{Base, Metodo, codificar};
use super::puertas::{self, EstadoCheck, Lecturas, Origenes, Puerta};
use super::{
    Creada, EstadoPuerta, FalloLectura, Firma, ModeloPuertas, Paso, Personas, PlanObservado,
    Plataforma, PrAbierto, Proveedor, PuertasLeidas, ReglasLeidas, Usuario, aplicar_en_lote,
};
use crate::contrato::{Informe, Resultado, Valor, corto};
use crate::i18n::catalogo::plataforma as t;

const ABRIR_PR: &str = "abrir-pr";
const MERGE_PR: &str = "merge-pr";
const LEER_PUERTAS: &str = "leer-puertas";
const LEER_PERMISOS_CI: &str = "leer-permisos-ci";
const FIRMA_COMMIT: &str = "firma-commit";
const IDENTIDAD: &str = "identidad";
const PRS_ABIERTOS: &str = "prs-abiertos";
const LEER_PLAN: &str = "leer-plan";

/// Permiso de `leer-puertas` (lo nombra `doctor` en `permiso=`).
pub const PERMISO_LEER_PUERTAS: &str = "metadata:read";

/// Permiso de los check-runs que lee `leer-puertas` (GitHub App:
/// «Repository permissions for "Checks"» read,
/// <https://docs.github.com/en/rest/authentication/permissions-required-for-github-apps>,
/// 2026-11-17).
pub const PERMISO_CHECKS: &str = "checks:read";

/// Permiso de `aplicar-puertas`: leer y crear rulesets (GitHub App:
/// «Administration» write).
pub const PERMISO_APLICAR_PUERTAS: &str = "administration:write";

/// Permiso de `leer-permisos-ci`.
pub const PERMISO_PERMISOS_CI: &str = "administration:read";

/// Garantias que en GitHub dan los rulesets (tier «Free publico;
/// Pro/Team/Enterprise» de la tabla de F1-13): sin ellos, el 403 de plan.
const GARANTIAS_DE_RULESETS: [&str; 10] = [
    "G01", "G02", "G03", "G04", "G05", "G06", "G07", "G08", "G09", "G11",
];

/// Endpoints de la API que usa el adaptador (lista cerrada).
enum Endpoint<'a> {
    Usuario,
    Repo,
    Pr(u64),
    PrsAbiertos { cabeza: &'a str, base: &'a str },
    TodosLosPrsAbiertos,
    CrearPr,
    Merge(u64),
    ReglasRama(&'a str),
    Rulesets,
    CrearRuleset,
    CheckRuns(&'a str),
    CheckRunsDeRama(&'a str),
    Reviews(u64),
    PermisosWorkflow,
    Commit(&'a str),
}

impl Endpoint<'_> {
    fn metodo_y_ruta(&self, repo: &str) -> (Metodo, String) {
        let r = format!("/repos/{repo}");
        match self {
            Self::Usuario => (Metodo::Get, "/user".into()),
            Self::Repo => (Metodo::Get, r),
            Self::Pr(n) => (Metodo::Get, format!("{r}/pulls/{n}")),
            Self::PrsAbiertos { cabeza, base } => (
                Metodo::Get,
                format!(
                    "{r}/pulls?head={}&base={}&state=open&per_page=100",
                    codificar(cabeza),
                    codificar(base)
                ),
            ),
            Self::TodosLosPrsAbiertos => {
                (Metodo::Get, format!("{r}/pulls?state=open&per_page=100"))
            }
            Self::CrearPr => (Metodo::Post, format!("{r}/pulls")),
            Self::Merge(n) => (Metodo::Put, format!("{r}/pulls/{n}/merge")),
            Self::ReglasRama(rama) => (Metodo::Get, format!("{r}/rules/branches/{rama}")),
            Self::Rulesets => (Metodo::Get, format!("{r}/rulesets?per_page=100")),
            Self::CrearRuleset => (Metodo::Post, format!("{r}/rulesets")),
            Self::CheckRuns(sha) => (
                Metodo::Get,
                format!("{r}/commits/{sha}/check-runs?per_page=100"),
            ),
            Self::CheckRunsDeRama(rama) => (
                Metodo::Get,
                format!("{r}/commits/heads/{rama}/check-runs?per_page=100"),
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

/// Un PR de la lista de abiertos: la cabeza con su repo, para no contar los
/// de forks.
#[derive(Deserialize)]
struct PrListaApi {
    number: u64,
    #[serde(default)]
    draft: bool,
    head: CabezaApi,
    base: CabezaApi,
}

#[derive(Deserialize)]
struct CabezaApi {
    #[serde(rename = "ref")]
    nombre: String,
    #[serde(default)]
    sha: String,
    /// `null` si el repo de un fork se borro.
    #[serde(default)]
    repo: Option<RepoApi>,
}

#[derive(Deserialize)]
struct RepoApi {
    id: u64,
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

/// Lo que se lee del repo para el plan (GH-01 de F1-13).
#[derive(Deserialize)]
struct RepoPlanApi {
    private: bool,
    #[serde(default)]
    visibility: Option<String>,
    owner: DuenoApi,
}

#[derive(Deserialize)]
struct DuenoApi {
    #[serde(rename = "type")]
    tipo: String,
}

#[derive(Deserialize)]
struct RulesetApi {
    name: String,
}

#[derive(Deserialize)]
struct ReglaApi {
    #[serde(rename = "type")]
    tipo: String,
    /// Los de `pull_request` y `required_status_checks`; los demas tipos
    /// traen otros campos (o ninguno) y se ignoran.
    #[serde(default)]
    parameters: Option<ParametrosApi>,
}

/// Los `parameters` de las reglas `pull_request` y
/// `required_status_checks` (esquema de rulesets,
/// <https://docs.github.com/en/rest/repos/rules>, 2026-11-17: los cinco
/// booleanos y el contador de `pull_request` son obligatorios, pero un
/// `null` no tumba la lectura: H5 de TASK-035).
#[derive(Deserialize, Default)]
struct ParametrosApi {
    #[serde(default)]
    required_approving_review_count: Option<u64>,
    #[serde(default, deserialize_with = "super::nulo_por_defecto")]
    required_status_checks: Vec<CheckRequeridoApi>,
    #[serde(default)]
    dismiss_stale_reviews_on_push: Option<bool>,
    #[serde(default)]
    require_last_push_approval: Option<bool>,
}

#[derive(Deserialize)]
struct CheckRequeridoApi {
    context: String,
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
                .diagnostico(&t::REPO_GITHUB));
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
    fn prs_abiertos(&self, rol: Rol) -> Paso<Vec<PrAbierto>> {
        if rol == Rol::Revisor {
            return Err(rol_no_valido(PRS_ABIERTOS, rol));
        }
        let (_, ruta) = self.ep(&Endpoint::TodosLosPrsAbiertos);
        let prs: Vec<PrListaApi> =
            self.con
                .leer_paginas(PRS_ABIERTOS, rol, "pull_requests:read", ruta, |c| {
                    self.con.parsear(PRS_ABIERTOS, c)
                })?;
        let mut v = Vec::new();
        for p in prs {
            let propio = matches!((&p.head.repo, &p.base.repo), (Some(h), Some(b)) if h.id == b.id);
            // Una rama que tc2 no habria creado no casa con ninguna tarea.
            if !propio || !rama_valida(&p.head.nombre) {
                continue;
            }
            if !sha_valido(&p.head.sha) {
                return Err(self.con.invalida_por(PRS_ABIERTOS, &t::RAMA_BASE_O_SHA));
            }
            v.push(PrAbierto {
                numero: p.number,
                rama: p.head.nombre,
                sha: p.head.sha.to_ascii_lowercase(),
                borrador: p.draft,
            });
        }
        v.sort_by_key(|p| p.numero);
        Ok(v)
    }

    fn abrir_pr(&self, rama: &str, base: &str, titulo: &str, cuerpo: &str) -> Informe {
        if !rama_valida(rama) || !rama_valida(base) {
            return uso(ABRIR_PR, "rama-invalida", &t::RAMA_NO_VALIDA);
        }
        if titulo.trim().is_empty() || titulo.chars().any(char::is_control) {
            return uso(ABRIR_PR, "titulo-invalido", &t::TITULO_NO_VALIDO);
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
        // Dos alcances unidos por una conjuncion: se traduce (IMP-1 de F2-8).
        let permiso = &t::PERMISO_PR_Y_CHECKS;
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
            return self.con.invalida_por(op, &t::RAMA_BASE_O_SHA);
        }
        let r = match self.con.pedir(
            op,
            rol,
            self.ep(&Endpoint::ReglasRama(&p.base.nombre)),
            None,
        ) {
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
        let reviews: Vec<ReviewApi> = match self
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
            return uso(MERGE_PR, "sha-invalido", &t::SHA_ESPERADO_COMPLETO);
        }
        if let Err(i) = self.exigir_persona(MERGE_PR) {
            return i;
        }
        let permiso = "contents:write";
        let p: PrApi =
            match self
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
                .diagnostico(&t::PR_CERRADO);
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
        validar_puertas(rol, rama)?;
        match self.rulesets(rol)? {
            Some(rulesets) => self.puertas_de_rama(rol, rama, &rulesets, true),
            None => Ok(sin_reglas_por_plan()),
        }
    }

    fn leer_puertas_de(
        &self,
        rol: Rol,
        ramas: &[&str],
        checks_de: &str,
    ) -> Vec<Paso<ReglasLeidas>> {
        // Los rulesets son del repo: se leen una vez (solo si alguna rama
        // es valida) y valen para todas.
        let validas: Vec<Paso<()>> = ramas.iter().map(|r| validar_puertas(rol, r)).collect();
        let comun = if validas.iter().any(Result::is_ok) {
            Some(self.rulesets(rol))
        } else {
            None
        };
        ramas
            .iter()
            .zip(validas)
            .map(|(rama, v)| {
                v?;
                match comun.as_ref() {
                    Some(Ok(Some(rs))) => self.puertas_de_rama(rol, rama, rs, *rama == checks_de),
                    Some(Ok(None)) => Ok(sin_reglas_por_plan()),
                    Some(Err(i)) => Err(i.clone()),
                    None => Err(rol_no_valido(LEER_PUERTAS, rol)),
                }
            })
            .collect()
    }

    fn leer_permisos_ci(&self) -> Informe {
        if let Err(i) = self.exigir_persona(LEER_PERMISOS_CI) {
            return i;
        }
        let p: PermisosApi = match self.con.leer(
            LEER_PERMISOS_CI,
            Rol::Persona,
            PERMISO_PERMISOS_CI,
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
                .diagnostico(&t::CI_PUEDE_APROBAR)
        } else {
            Informe::nuevo(LEER_PERMISOS_CI, Resultado::Ok)
                .con("ci-aprueba", "false")
                .con("permisos", permisos)
        }
    }

    fn firma_commit(&self, sha: &str) -> Paso<Firma> {
        if !sha_valido(sha) {
            return Err(uso(FIRMA_COMMIT, "sha-invalido", &t::SHA_COMPLETO));
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
                .leer::<UsuarioApi>(
                    IDENTIDAD,
                    rol,
                    &t::PERMISO_NINGUNO,
                    self.ep(&Endpoint::Usuario),
                )
                .map(Usuario::from)
        })
    }

    fn proveedor(&self) -> Option<Proveedor> {
        Some(Proveedor::GitHub)
    }

    fn leer_plan(&self, rol: Rol) -> Paso<PlanObservado> {
        if rol == Rol::Revisor {
            return Err(rol_no_valido(LEER_PLAN, rol));
        }
        let r: RepoPlanApi = self.con.leer(
            LEER_PLAN,
            rol,
            PERMISO_LEER_PUERTAS,
            self.ep(&Endpoint::Repo),
        )?;
        Ok(PlanObservado::GitHub {
            // `internal` (Enterprise) cuenta como privado.
            publico: !r.private && r.visibility.as_deref().is_none_or(|v| v == "public"),
            organizacion: r.owner.tipo == "Organization",
        })
    }

    fn preparar_puertas(&self, ps: &[Puerta], origenes: &Origenes) -> Paso<PuertasLeidas> {
        let op = puertas::APLICAR_PUERTAS;
        self.exigir_persona(op)?;
        // Sin rulesets en el plan (403 con aviso), `rechazada motivo=plan`.
        let existentes = self.rulesets_de(op, Rol::Persona, PERMISO_APLICAR_PUERTAS)?;
        // D2: cada check, ya publicado por su App en la cabeza de `develop`.
        let (_, ruta) = self.ep(&Endpoint::CheckRunsDeRama("develop"));
        let runs: Vec<CheckRunApi> =
            self.con
                .leer_paginas(op, Rol::Persona, PERMISO_CHECKS, ruta, |c| {
                    self.con
                        .parsear::<CheckRunsApi>(op, c)
                        .map(|x| x.check_runs)
                })?;
        let vistos: Vec<(String, u64)> = runs
            .into_iter()
            .filter_map(|r| r.app.map(|a| (r.name, a.id)))
            .collect();
        let faltan = puertas::no_vistos(origenes, &vistos);
        if !faltan.is_empty() {
            return Err(puertas::sin_origen_visto(faltan));
        }
        // Lo existente se reconoce por el nombre y no se compara: no se
        // toca (D3), asi que da igual si difiere.
        Ok(PuertasLeidas {
            puertas: ps
                .iter()
                .map(|p| {
                    let n = p.nombre_completo();
                    let e = if existentes.contains(&n) {
                        EstadoPuerta::Existe
                    } else {
                        EstadoPuerta::Falta
                    };
                    (n, e)
                })
                .collect(),
            notas: Vec::new(),
        })
    }

    fn aplicar_puertas(&self, ps: &[Puerta], origenes: &Origenes) -> Informe {
        let op = puertas::APLICAR_PUERTAS;
        let leidas = match self.preparar_puertas(ps, origenes) {
            Ok(l) => l,
            Err(i) => return i,
        };
        aplicar_en_lote(ps, &leidas, |p| {
            let cuerpo = puertas::ruleset_github(p, Some(origenes));
            if cuerpo.get("_nota").is_some() {
                // Nunca se envia un check sin su `integration_id`.
                return Err(puertas::sin_origen_visto(
                    p.checks
                        .iter()
                        .filter(|c| origenes.de(c).is_none())
                        .map(|c| (*c).to_owned())
                        .collect(),
                ));
            }
            let r = self.con.pedir(
                op,
                Rol::Persona,
                self.ep(&Endpoint::CrearRuleset),
                Some(&cuerpo.to_string()),
            )?;
            if r.exito() {
                return Ok(Creada::Nueva);
            }
            // 422: el nombre ya existe si otro llego antes; se relee.
            if r.estado == 422
                && self
                    .rulesets_de(op, Rol::Persona, PERMISO_APLICAR_PUERTAS)
                    .is_ok_and(|v| v.contains(&p.nombre_completo()))
            {
                return Ok(Creada::YaExistia);
            }
            Err(self
                .con
                .traducir(op, Rol::Persona, PERMISO_APLICAR_PUERTAS, &r))
        })
    }
}

/// Rama y rol de `leer-puertas`.
fn validar_puertas(rol: Rol, rama: &str) -> Paso<()> {
    if !rama_valida(rama) {
        return Err(uso(LEER_PUERTAS, "rama-invalida", &t::RAMA_NO_VALIDA));
    }
    if rol == Rol::Revisor {
        return Err(rol_no_valido(LEER_PUERTAS, rol));
    }
    Ok(())
}

/// Lo leido cuando el plan no da los rulesets (403 con aviso de plan).
fn sin_reglas_por_plan() -> ReglasLeidas {
    ReglasLeidas {
        plan_da_reglas: false,
        ..ReglasLeidas::default()
    }
}

impl GitHub {
    /// Los nombres de los rulesets del repo; `None` si el plan no los da.
    fn rulesets(&self, rol: Rol) -> Paso<Option<Vec<String>>> {
        let r = self
            .con
            .pedir(LEER_PUERTAS, rol, self.ep(&Endpoint::Rulesets), None)?;
        if es_aviso_de_plan(&r) {
            return Ok(None);
        }
        if !r.exito() {
            return Err(self
                .con
                .traducir(LEER_PUERTAS, rol, PERMISO_LEER_PUERTAS, &r));
        }
        let rulesets: Vec<RulesetApi> = self.con.parsear(LEER_PUERTAS, &r.cuerpo)?;
        Ok(Some(rulesets.into_iter().map(|r| r.name).collect()))
    }

    /// Los nombres de todos los rulesets del repo para `op`, paginados; el
    /// 403 de plan es su `rechazada motivo=plan` (`errores::traducir`).
    fn rulesets_de(&self, op: &str, rol: Rol, permiso: &'static str) -> Paso<Vec<String>> {
        let (_, ruta) = self.ep(&Endpoint::Rulesets);
        let rulesets: Vec<RulesetApi> = self
            .con
            .leer_paginas(op, rol, permiso, ruta, |c| self.con.parsear(op, c))?;
        Ok(rulesets.into_iter().map(|r| r.name).collect())
    }

    /// Las reglas de `rama` con los `rulesets` ya leidos; los check-runs de
    /// su cabeza solo `con_checks`.
    fn puertas_de_rama(
        &self,
        rol: Rol,
        rama: &str,
        rulesets: &[String],
        con_checks: bool,
    ) -> Paso<ReglasLeidas> {
        let reglas: Vec<ReglaApi> = self.con.leer(
            LEER_PUERTAS,
            rol,
            PERMISO_LEER_PUERTAS,
            self.ep(&Endpoint::ReglasRama(rama)),
        )?;
        let parametros = |tipo: &'static str| {
            reglas
                .iter()
                .filter(move |r| r.tipo == tipo)
                .filter_map(|r| r.parameters.as_ref())
        };
        let aprobaciones = parametros("pull_request")
            .filter_map(|p| p.required_approving_review_count)
            .max()
            .unwrap_or(0);
        let descartar_viejas =
            parametros("pull_request").any(|p| p.dismiss_stale_reviews_on_push == Some(true));
        let ultimo_push =
            parametros("pull_request").any(|p| p.require_last_push_approval == Some(true));
        let requeridos: Vec<String> = parametros("required_status_checks")
            .flat_map(|p| p.required_status_checks.iter().map(|c| c.context.clone()))
            .collect();
        // Los check-runs de la cabeza de la rama (`heads/<rama>`, la forma
        // que documenta la API). Si no se pueden leer, no se supone nada y
        // se dice por que (H3 de TASK-035).
        let (publicados, fallo_checks) = if con_checks {
            let (_, ruta) = self.ep(&Endpoint::CheckRunsDeRama(rama));
            match self
                .con
                .leer_paginas(LEER_PUERTAS, rol, PERMISO_CHECKS, ruta, |c| {
                    self.con
                        .parsear::<CheckRunsApi>(LEER_PUERTAS, c)
                        .map(|x| x.check_runs)
                }) {
                Ok(v) => (Some(v.into_iter().map(|r| r.name).collect()), None),
                Err(i) => (None, Some(FalloLectura::de(&i, PERMISO_CHECKS))),
            }
        } else {
            (None, None)
        };
        Ok(ReglasLeidas {
            plan_da_reglas: true,
            rulesets: rulesets.to_vec(),
            reglas_rama: reglas.into_iter().map(|r| r.tipo).collect(),
            aprobaciones,
            checks_requeridos: Some(requeridos),
            todos_los_checks: false,
            checks_publicados: publicados,
            fallo_checks,
            descartar_viejas,
            ultimo_push,
        })
    }
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
