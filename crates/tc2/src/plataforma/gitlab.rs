//! Adaptador de GitLab del puerto `Plataforma` (F2-9, ADR F1-13 decisiones
//! 1, 4 y 6; F1-6 decision 1 para la identidad de cada operacion).
//!
//! Mismo trait y misma linea que el de GitHub (RR-15): lo que cambia es de
//! donde sale cada lectura. Cada endpoint es una variante de [`Endpoint`]
//! sobre la API REST v4, sin `glab`. El proyecto va en la ruta como id
//! numerico o como `grupo%2Fproyecto`, codificado una sola vez. Las
//! respuestas son dato no confiable (F1-8, Z2): structs con los campos que
//! se usan, y un campo ausente da `error motivo=respuesta-invalida`.

use std::collections::BTreeMap;

use serde::Deserialize;

use super::conexion::{Conexion, head_cambiado, rama_valida, rol_no_valido, sha_valido, uso};
use super::credenciales::{Credenciales, Rol};
use super::errores;
use super::http::{Base, Metodo, codificar};
use super::puertas::{self, EstadoCheck, Lecturas, Objetivo, Origenes, Puerta};
use super::{
    Clave, Creada, EstadoPuerta, FalloLectura, Firma, ModeloPuertas, Paso, Personas, PlanObservado,
    Plataforma, PrAbierto, Proveedor, PuertasLeidas, ReglasLeidas, TierGitLab, Usuario,
    aplicar_en_lote,
};
use crate::contrato::{Informe, Resultado, Valor, corto};
use crate::i18n::catalogo::plataforma as t;
use crate::i18n::{Mensaje, Texto};

const ABRIR_PR: &str = "abrir-pr";
const MERGE_PR: &str = "merge-pr";
const LEER_PUERTAS: &str = "leer-puertas";
const LEER_PERMISOS_CI: &str = "leer-permisos-ci";
const FIRMA_COMMIT: &str = "firma-commit";
const IDENTIDAD: &str = "identidad";
const PRS_ABIERTOS: &str = "prs-abiertos";
const LEER_PLAN: &str = "leer-plan";

/// El scope de lectura en una clave (`doctor`, `permiso=`); el texto
/// completo, con el rol, es [`PERMISO_LECTURA`].
pub const PERMISO_LECTURA_CORTO: &str = "read_api";

/// Permiso de lectura que necesita un token de GitLab.
static PERMISO_LECTURA: &Texto = &t::PERMISO_LECTURA_GITLAB;

/// Permiso para abrir un MR.
static PERMISO_ESCRITURA: &Texto = &t::PERMISO_ESCRITURA_GITLAB;

/// Permiso para mergear en una rama protegida.
static PERMISO_MERGE: &Texto = &t::PERMISO_MERGE_GITLAB;

/// Garantias que en GitLab solo da Premium o Ultimate (tabla de F1-13): sin
/// reglas de aprobacion (403 en Free), la politica no puede exigirlas
/// impuestas.
const GARANTIAS_PREMIUM: [&str; 6] = ["G03", "G04", "G05", "G06", "G10", "G12"];

/// Nivel de acceso de Maintainer.
const MAINTAINER: u64 = 40;

/// Nivel de acceso de Developer (el agente y el revisor, F1-13 D6).
const DEVELOPER: u64 = 30;

/// Nivel «nadie» (`No one`).
const NADIE: u64 = 0;

/// Regla de aprobacion que crea `aplicar-puertas` en Premium.
const REGLA_APROBACION: &str = "taskcodetwo-aprobacion";

/// Niveles que `aplicar-puertas` pone a una rama protegida, siempre
/// explicitos (la API pone Maintainer si no se dan, y el agente, Developer,
/// se quedaria sin push; riesgo 1 del plan de TASK-118).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Niveles {
    push: u64,
    merge: u64,
}

/// Los niveles de una puerta: `develop` y `main` (solo por MR) sin push y
/// con merge solo de Maintainer (G01: el merge es de la persona); las ramas
/// de tarea y la de estado, push y merge de Developer. Nunca force-push.
fn niveles_de(p: &Puerta) -> Niveles {
    if p.solo_pr.is_some() {
        Niveles {
            push: NADIE,
            merge: MAINTAINER,
        }
    } else {
        Niveles {
            push: DEVELOPER,
            merge: DEVELOPER,
        }
    }
}

/// El nombre de GitLab de un patron del modelo: sin `refs/heads/` ni
/// `refs/tags/`, y `**` como `*` (el comodin de GitLab).
fn nombre_gitlab(patron: &str) -> String {
    patron
        .trim_start_matches("refs/heads/")
        .trim_start_matches("refs/tags/")
        .replace("**", "*")
}

/// Si una lista de niveles es exactamente `nivel`, sin usuarios ni grupos.
fn solo_nivel(v: &[NivelApi], nivel: u64) -> bool {
    v.len() == 1
        && v.iter()
            .all(|n| n.access_level == nivel && n.user_id.is_none() && n.group_id.is_none())
}

/// Endpoints de la API que usa el adaptador (lista cerrada).
enum Endpoint<'a> {
    Usuario,
    Mr(u64),
    MrConDivergencia(u64),
    MrsAbiertos { rama: &'a str, base: &'a str },
    TodosLosMrsAbiertos,
    CommitsEntre { base: &'a str, rama: &'a str },
    CrearMr,
    Merge(u64),
    ReglasAprobacion,
    RamaProtegida(&'a str),
    RamasProtegidas,
    Estados(&'a str),
    Aprobaciones(u64),
    Firma(&'a str),
    Commit(&'a str),
    Proyecto,
    Rama(&'a str),
    Namespace(u64),
    CrearRamaProtegida,
    TagsProtegidos,
    CrearTagProtegido,
    AjustesProyecto,
    CrearReglaAprobacion,
}

impl Endpoint<'_> {
    fn metodo_y_ruta(&self, proyecto: &str) -> (Metodo, String) {
        let p = format!("/projects/{proyecto}");
        match self {
            Self::Usuario => (Metodo::Get, "/user".into()),
            Self::Mr(n) => (Metodo::Get, format!("{p}/merge_requests/{n}")),
            Self::MrConDivergencia(n) => (
                Metodo::Get,
                format!("{p}/merge_requests/{n}?include_diverged_commits_count=true"),
            ),
            Self::MrsAbiertos { rama, base } => (
                Metodo::Get,
                format!(
                    "{p}/merge_requests?source_branch={}&target_branch={}&state=opened&per_page=100",
                    codificar(rama),
                    codificar(base)
                ),
            ),
            Self::CommitsEntre { base, rama } => (
                Metodo::Get,
                format!(
                    "{p}/repository/commits?ref_name={}&per_page=1",
                    codificar(&format!("{base}..{rama}"))
                ),
            ),
            Self::TodosLosMrsAbiertos => (
                Metodo::Get,
                format!("{p}/merge_requests?state=opened&per_page=100"),
            ),
            Self::CrearMr => (Metodo::Post, format!("{p}/merge_requests")),
            Self::Merge(n) => (Metodo::Put, format!("{p}/merge_requests/{n}/merge")),
            Self::ReglasAprobacion => (Metodo::Get, format!("{p}/approval_rules?per_page=100")),
            Self::RamaProtegida(rama) => (
                Metodo::Get,
                format!("{p}/protected_branches/{}", codificar(rama)),
            ),
            Self::RamasProtegidas => (Metodo::Get, format!("{p}/protected_branches?per_page=100")),
            Self::Estados(sha) => (
                Metodo::Get,
                format!("{p}/repository/commits/{sha}/statuses?all=true&per_page=100"),
            ),
            Self::Aprobaciones(n) => (Metodo::Get, format!("{p}/merge_requests/{n}/approvals")),
            Self::Firma(sha) => (
                Metodo::Get,
                format!("{p}/repository/commits/{sha}/signature"),
            ),
            Self::Commit(sha) => (Metodo::Get, format!("{p}/repository/commits/{sha}")),
            Self::Proyecto => (Metodo::Get, p),
            // Fuera del proyecto: la ruta la construye el adaptador con el
            // id numerico que da el propio proyecto.
            Self::Namespace(id) => (Metodo::Get, format!("/namespaces/{id}")),
            Self::Rama(rama) => (
                Metodo::Get,
                format!("{p}/repository/branches/{}", codificar(rama)),
            ),
            // Escrituras de `aplicar-puertas` (TASK-118): solo crean; los
            // ajustes de merge del proyecto, con `PUT` idempotente. Ningun
            // `DELETE` ni `PATCH`.
            Self::CrearRamaProtegida => (Metodo::Post, format!("{p}/protected_branches")),
            Self::TagsProtegidos => (Metodo::Get, format!("{p}/protected_tags?per_page=100")),
            Self::CrearTagProtegido => (Metodo::Post, format!("{p}/protected_tags")),
            Self::AjustesProyecto => (Metodo::Put, p),
            Self::CrearReglaAprobacion => (Metodo::Post, format!("{p}/approval_rules")),
        }
    }
}

#[derive(Deserialize)]
struct UsuarioApi {
    #[serde(default)]
    username: String,
    id: u64,
    #[serde(default)]
    bot: bool,
}

impl From<UsuarioApi> for Usuario {
    fn from(u: UsuarioApi) -> Self {
        Self {
            login: u.username,
            id: u.id,
            tipo: if u.bot { "Bot" } else { "User" }.to_owned(),
        }
    }
}

#[derive(Deserialize)]
struct MrApi {
    iid: u64,
    state: String,
    #[serde(default)]
    sha: Option<String>,
    target_branch: String,
    source_branch: String,
}

/// Un MR de la lista de abiertos. `draft` desde GitLab 14; antes,
/// `work_in_progress` (que sigue saliendo, obsoleto).
#[derive(Deserialize)]
struct MrListaApi {
    iid: u64,
    source_branch: String,
    #[serde(default)]
    sha: Option<String>,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    work_in_progress: bool,
    source_project_id: u64,
    target_project_id: u64,
}

#[derive(Deserialize)]
struct MrPuertasApi {
    state: String,
    sha: String,
    author: AutorApi,
    target_branch: String,
    has_conflicts: bool,
    diverged_commits_count: u64,
    #[serde(default)]
    detailed_merge_status: Option<String>,
}

#[derive(Deserialize)]
struct NivelApi {
    access_level: u64,
    #[serde(default)]
    user_id: Option<u64>,
    #[serde(default)]
    group_id: Option<u64>,
}

#[derive(Deserialize)]
struct RamaProtegidaApi {
    name: String,
    merge_access_levels: Vec<NivelApi>,
    #[serde(default, deserialize_with = "super::nulo_por_defecto")]
    push_access_levels: Vec<NivelApi>,
    allow_force_push: bool,
    #[serde(default, deserialize_with = "super::nulo_por_defecto")]
    code_owner_approval_required: bool,
}

#[derive(Deserialize)]
struct AutorApi {
    id: u64,
}

#[derive(Deserialize)]
struct EstadoApi {
    id: u64,
    name: String,
    status: String,
    author: AutorApi,
}

#[derive(Deserialize)]
struct AprobacionesApi {
    approved_by: Vec<AprobadorApi>,
}

#[derive(Deserialize)]
struct AprobadorApi {
    user: UsuarioApi,
}

#[derive(Deserialize)]
struct ReglaAprobacionApi {
    name: String,
    /// `null` cuenta como 0 (no pide ninguna): lo conservador, `doctor` la
    /// da por no configurada (H5 de TASK-035).
    #[serde(default, deserialize_with = "super::nulo_por_defecto")]
    approvals_required: u64,
}

#[derive(Deserialize)]
struct MergeApi {
    state: String,
    #[serde(default)]
    merge_commit_sha: Option<String>,
}

#[derive(Deserialize)]
struct ClaveApi {
    id: u64,
}

#[derive(Deserialize)]
struct FirmaApi {
    #[serde(default)]
    signature_type: String,
    verification_status: String,
    #[serde(default)]
    gpg_key_id: Option<u64>,
    #[serde(default)]
    key: Option<ClaveApi>,
}

#[derive(Deserialize)]
struct CommitApi {
    id: String,
}

/// Lo que se lee del proyecto. Los dos campos solo salen con sesion
/// (GL-01 de F1-13): ausentes, `None`.
#[derive(Deserialize)]
struct ProyectoApi {
    #[serde(default)]
    only_allow_merge_if_pipeline_succeeds: Option<bool>,
    /// `merge`, `rebase_merge` o `ff` (solo con sesion).
    #[serde(default)]
    merge_method: Option<String>,
    #[serde(default)]
    namespace: Option<NamespaceRefApi>,
}

#[derive(Deserialize)]
struct NamespaceRefApi {
    id: u64,
}

#[derive(Deserialize)]
struct NamespaceApi {
    #[serde(default)]
    plan: Option<String>,
}

#[derive(Deserialize)]
struct RamaApi {
    commit: CommitApi,
}

#[derive(Deserialize)]
struct TagProtegidoApi {
    name: String,
    #[serde(default, deserialize_with = "super::nulo_por_defecto")]
    create_access_levels: Vec<NivelApi>,
}

/// Adaptador de GitLab para un proyecto.
pub struct GitLab {
    con: Conexion,
    /// Id del proyecto ya codificado para la ruta.
    proyecto: String,
}

impl std::fmt::Debug for GitLab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitLab")
            .field("con", &self.con)
            .field("proyecto", &self.proyecto)
            .finish_non_exhaustive()
    }
}

impl GitLab {
    /// Adaptador para el proyecto `proyecto` (id numerico o
    /// `grupo[/subgrupo...]/proyecto`) en la API `base` (con su `/api/v4`).
    pub fn nuevo(
        base: Base,
        proyecto: &str,
        credenciales: Box<dyn Credenciales>,
        personas: Personas,
    ) -> Result<Self, Informe> {
        let Some(codificado) = codificar_proyecto(proyecto) else {
            return Err(Informe::nuevo("plataforma", Resultado::Uso)
                .con("motivo", "proyecto-invalido")
                .diagnostico(&t::PROYECTO_GITLAB));
        };
        Ok(Self {
            con: Conexion::nueva(base, Proveedor::GitLab, credenciales, personas),
            proyecto: codificado,
        })
    }

    /// Llamadas hechas, `METODO ruta` (para medir el coste por operacion).
    pub fn registro(&self) -> Vec<String> {
        self.con.registro()
    }

    fn ep(&self, ep: &Endpoint<'_>) -> (Metodo, String) {
        ep.metodo_y_ruta(&self.proyecto)
    }

    fn exigir_persona(&self, op: &str) -> Paso<()> {
        self.con.exigir_persona(op, self.identidad(Rol::Persona))
    }

    fn mr_abierto(&self, rama: &str, base: &str) -> Paso<Option<u64>> {
        let (_, ruta) = self.ep(&Endpoint::MrsAbiertos { rama, base });
        let mrs: Vec<MrApi> =
            self.con
                .leer_paginas(ABRIR_PR, Rol::Agente, PERMISO_LECTURA, ruta, |c| {
                    self.con.parsear(ABRIR_PR, c)
                })?;
        Ok(mrs
            .into_iter()
            .find(|m| m.source_branch == rama && m.target_branch == base && m.state == "opened")
            .map(|m| m.iid))
    }

    /// Checks requeridos del proyecto (F3-8): GitLab no exige checks por
    /// nombre; con «Pipelines must succeed» cuenta el pipeline entero
    /// (`todos`). `None` si el proyecto no se pudo leer o no trae el campo
    /// (sin sesion no sale, GL-01 de F1-13).
    fn checks_requeridos(&self, rol: Rol) -> (Option<Vec<String>>, bool) {
        let p = self.con.leer::<ProyectoApi>(
            LEER_PUERTAS,
            rol,
            PERMISO_LECTURA,
            self.ep(&Endpoint::Proyecto),
        );
        match p.ok().and_then(|p| p.only_allow_merge_if_pipeline_succeeds) {
            Some(todos) => (Some(Vec::new()), todos),
            None => (None, false),
        }
    }

    /// Nombres de los estados de la cabeza de `rama` (la API de estados
    /// pide el sha: se lee antes la rama). Si algo no se lee, el motivo
    /// (un 403, con su permiso: H3 de TASK-035).
    fn checks_publicados(&self, rol: Rol, rama: &str) -> Result<Vec<String>, FalloLectura> {
        let fallo = |i: Informe| FalloLectura::de(&i, PERMISO_LECTURA_CORTO);
        let r: RamaApi = self
            .con
            .leer(
                LEER_PUERTAS,
                rol,
                PERMISO_LECTURA,
                self.ep(&Endpoint::Rama(rama)),
            )
            .map_err(fallo)?;
        if !sha_valido(&r.commit.id) {
            return Err(FalloLectura::motivo("respuesta-invalida"));
        }
        let (_, ruta) = self.ep(&Endpoint::Estados(&r.commit.id));
        let estados: Vec<EstadoApi> = self
            .con
            .leer_paginas(LEER_PUERTAS, rol, PERMISO_LECTURA, ruta, |c| {
                self.con.parsear(LEER_PUERTAS, c)
            })
            .map_err(fallo)?;
        Ok(estados.into_iter().map(|e| e.name).collect())
    }

    /// Si `rama` tiene commits que no estan en `base`.
    fn tiene_commits(&self, rama: &str, base: &str) -> Paso<bool> {
        let commits: Vec<CommitApi> = self.con.leer(
            ABRIR_PR,
            Rol::Agente,
            PERMISO_LECTURA,
            self.ep(&Endpoint::CommitsEntre { base, rama }),
        )?;
        Ok(commits.iter().any(|c| !c.id.is_empty()))
    }
}

/// Id del proyecto listo para la ruta: el numero tal cual, o los segmentos
/// validados unidos con `%2F`. `None` si no vale.
fn codificar_proyecto(p: &str) -> Option<String> {
    if !p.is_empty() && p.len() <= 20 && p.bytes().all(|b| b.is_ascii_digit()) {
        return (p.parse::<u64>().ok()? > 0).then(|| p.to_owned());
    }
    let segmentos: Vec<&str> = p.split('/').collect();
    let valido = segmentos.len() >= 2
        && p.len() <= 255
        && segmentos.iter().all(|s| {
            !s.is_empty()
                && *s != "."
                && *s != ".."
                && !s.starts_with(['-', '.'])
                && !s.ends_with('.')
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-._".contains(&b))
        });
    valido.then(|| segmentos.join("%2F"))
}

/// Estado de un estado de commit de GitLab. Solo `success` es verde; un
/// estado saltado o manual no da la puerta por cumplida (falla cerrado,
/// como la PoC M).
fn estado_check(s: &str) -> EstadoCheck {
    match s {
        "success" => EstadoCheck::Ok,
        "failed" | "canceled" => EstadoCheck::Fallo,
        _ => EstadoCheck::Pendiente,
    }
}

impl Plataforma for GitLab {
    fn prs_abiertos(&self, rol: Rol) -> Paso<Vec<PrAbierto>> {
        if rol == Rol::Revisor {
            return Err(rol_no_valido(PRS_ABIERTOS, rol));
        }
        let (_, ruta) = self.ep(&Endpoint::TodosLosMrsAbiertos);
        let mrs: Vec<MrListaApi> =
            self.con
                .leer_paginas(PRS_ABIERTOS, rol, PERMISO_LECTURA, ruta, |c| {
                    self.con.parsear(PRS_ABIERTOS, c)
                })?;
        let mut v = Vec::new();
        for m in mrs {
            // Los MR desde un fork no cuentan; una rama que tc2 no habria
            // creado no casa con ninguna tarea.
            if m.source_project_id != m.target_project_id || !rama_valida(&m.source_branch) {
                continue;
            }
            let sha = m.sha.unwrap_or_default();
            if !sha_valido(&sha) {
                return Err(self.con.invalida_por(PRS_ABIERTOS, &t::RAMA_DESTINO_O_SHA));
            }
            v.push(PrAbierto {
                numero: m.iid,
                rama: m.source_branch,
                sha: sha.to_ascii_lowercase(),
                borrador: m.draft || m.work_in_progress,
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
        match self.mr_abierto(rama, base) {
            Ok(Some(n)) => {
                return Informe::nuevo(ABRIR_PR, Resultado::SinCambios).con("pr", n.to_string());
            }
            Ok(None) => {}
            Err(i) => return i,
        }
        // GitLab abre un MR sin commits: se comprueba antes para dar la
        // misma linea que GitHub.
        match self.tiene_commits(rama, base) {
            Ok(true) => {}
            Ok(false) => {
                return Informe::nuevo(ABRIR_PR, Resultado::Rechazada)
                    .con("motivo", "sin-commits")
                    .diagnostico(Mensaje::nuevo(&t::SIN_COMMITS).con("cita", ""));
            }
            Err(i) => return i,
        }
        let datos = serde_json::json!({
            "source_branch": rama, "target_branch": base,
            "title": titulo, "description": cuerpo,
        })
        .to_string();
        let r = match self.con.pedir(
            ABRIR_PR,
            Rol::Agente,
            self.ep(&Endpoint::CrearMr),
            Some(&datos),
        ) {
            Ok(r) => r,
            Err(i) => return i,
        };
        if r.estado == 409 {
            // Otro proceso lo abrio entre la lectura y la escritura.
            return match self.mr_abierto(rama, base) {
                Ok(Some(n)) => {
                    Informe::nuevo(ABRIR_PR, Resultado::SinCambios).con("pr", n.to_string())
                }
                Ok(None) => Informe::nuevo(ABRIR_PR, Resultado::Conflicto)
                    .con("motivo", "mr-existente")
                    .diagnostico(
                        Mensaje::nuevo(&t::MR_EXISTENTE).con("api", errores::mensaje_api(&r)),
                    ),
                Err(i) => i,
            };
        }
        if !r.exito() {
            return self
                .con
                .traducir(ABRIR_PR, Rol::Agente, PERMISO_ESCRITURA, &r);
        }
        match self.con.parsear::<MrApi>(ABRIR_PR, &r.cuerpo) {
            Ok(m) => Informe::nuevo(ABRIR_PR, Resultado::Ok).con("pr", m.iid.to_string()),
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
        let m: MrPuertasApi = match self.con.leer(
            op,
            rol,
            PERMISO_LECTURA,
            self.ep(&Endpoint::MrConDivergencia(pr)),
        ) {
            Ok(m) => m,
            Err(i) => return i,
        };
        if m.state == "merged" {
            return puertas::mergeado(pr);
        }
        if m.state != "opened" {
            return puertas::pr_cerrado(pr);
        }
        if !rama_valida(&m.target_branch) || !sha_valido(&m.sha) {
            return self.con.invalida_por(op, &t::RAMA_DESTINO_O_SHA);
        }
        // Las reglas de aprobacion son de Premium. Observado en el E2E: en
        // Free el endpoint responde 200 con la lista vacia (no el 403 que
        // suponia F1-13), asi que lo que se comprueba es que haya una regla
        // que pida al menos una aprobacion; el 403 sigue contando como plan.
        let r = match self
            .con
            .pedir(op, rol, self.ep(&Endpoint::ReglasAprobacion), None)
        {
            Ok(r) => r,
            Err(i) => return i,
        };
        let impone_aprobacion = if r.exito() {
            match self.con.parsear::<Vec<ReglaAprobacionApi>>(op, &r.cuerpo) {
                Ok(v) => v.iter().any(|x| x.approvals_required >= 1),
                Err(i) => return i,
            }
        } else if r.estado == 403 {
            false
        } else {
            return self.con.traducir(op, rol, PERMISO_LECTURA, &r);
        };
        if !impone_aprobacion
            && let Some(g) = modelo
                .exigir_impuestas
                .iter()
                .find(|g| GARANTIAS_PREMIUM.contains(&g.as_str()))
        {
            return if r.estado == 403 {
                puertas::rechazada_por_plan(pr, g)
            } else {
                puertas::rechazada_por_config(pr, g, &t::ARREGLO_APROBACIONES_GITLAB)
            };
        }
        // G01: la rama destino solo la mergean Maintainers y no admite
        // force-push.
        let r = match self.con.pedir(
            op,
            rol,
            self.ep(&Endpoint::RamaProtegida(&m.target_branch)),
            None,
        ) {
            Ok(r) => r,
            Err(i) => return i,
        };
        let protegida = match r.estado {
            404 => None,
            _ if r.exito() => match self.con.parsear::<RamaProtegidaApi>(op, &r.cuerpo) {
                Ok(p) => Some(p),
                Err(i) => return i,
            },
            _ => return self.con.traducir(op, rol, PERMISO_LECTURA, &r),
        };
        let g01 = protegida.is_some_and(|p| {
            !p.allow_force_push
                && p.merge_access_levels.len() == 1
                && p.merge_access_levels.iter().all(|n| {
                    n.access_level == MAINTAINER && n.user_id.is_none() && n.group_id.is_none()
                })
        });
        if !g01 {
            return puertas::rechazada_por_config(pr, "G01", &t::ARREGLO_RAMA_PROTEGIDA_GITLAB);
        }
        let (_, ruta) = self.ep(&Endpoint::Estados(&m.sha));
        let mut estados: Vec<EstadoApi> =
            match self
                .con
                .leer_paginas(op, rol, PERMISO_LECTURA, ruta, |c| self.con.parsear(op, c))
            {
                Ok(v) => v,
                Err(i) => return i,
            };
        // De cada nombre cuenta el ultimo estado (por id) que publico su
        // origen: `author.id` es el sustituto de G09 (F1-13, decision 6).
        estados.sort_by_key(|e| e.id);
        let checks = modelo
            .checks
            .iter()
            .map(|(nombre, origen)| {
                let estado = estados
                    .iter()
                    .rev()
                    .find(|e| &e.name == nombre && e.author.id == *origen)
                    .map_or(EstadoCheck::Pendiente, |e| estado_check(&e.status));
                (nombre.clone(), estado)
            })
            .collect();
        let a: AprobacionesApi = match self.con.leer(
            op,
            rol,
            PERMISO_LECTURA,
            self.ep(&Endpoint::Aprobaciones(pr)),
        ) {
            Ok(a) => a,
            Err(i) => return i,
        };
        // Una aprobacion por cuenta, por id (F1-6, regla 6). La del autor del
        // MR no cuenta: GitLab Free la admite (observado en el E2E) y GitHub
        // no, y la linea tiene que ser la misma (RR-15).
        let aprobadores: BTreeMap<u64, Usuario> = a
            .approved_by
            .into_iter()
            .map(|x| (x.user.id, Usuario::from(x.user)))
            .collect();
        let aprobaciones = aprobadores
            .values()
            .filter(|u| u.id != m.author.id && u.es_persona(self.con.personas()))
            .count();
        puertas::evaluar(&Lecturas {
            pr,
            checks,
            aprobaciones,
            ignoradas: aprobadores.len() - aprobaciones,
            al_dia: m.diverged_commits_count == 0,
            // `has_conflicts` se calcula en segundo plano: mientras GitLab
            // comprueba el MR no se da por libre de conflictos (falla
            // cerrado; la siguiente lectura lo resuelve).
            sin_conflictos: !m.has_conflicts
                && !matches!(
                    m.detailed_merge_status.as_deref(),
                    Some("checking" | "unchecked" | "preparing" | "conflict")
                ),
        })
    }

    fn merge_pr(&self, pr: u64, sha: &str) -> Informe {
        if !sha_valido(sha) {
            return uso(MERGE_PR, "sha-invalido", &t::SHA_ESPERADO_COMPLETO);
        }
        if let Err(i) = self.exigir_persona(MERGE_PR) {
            return i;
        }
        let m: MrApi = match self.con.leer(
            MERGE_PR,
            Rol::Persona,
            PERMISO_MERGE,
            self.ep(&Endpoint::Mr(pr)),
        ) {
            Ok(m) => m,
            Err(i) => return i,
        };
        if m.state == "merged" {
            return Informe::nuevo(MERGE_PR, Resultado::SinCambios)
                .con("pr", pr.to_string())
                .con("estado", "mergeado");
        }
        if m.state != "opened" {
            return Informe::nuevo(MERGE_PR, Resultado::Rechazada)
                .con("pr", pr.to_string())
                .con("motivo", "pr-cerrado")
                .diagnostico(&t::MR_CERRADO);
        }
        if !m
            .sha
            .as_deref()
            .is_some_and(|s| s.eq_ignore_ascii_case(sha))
        {
            return head_cambiado(pr);
        }
        let datos = serde_json::json!({
            "sha": sha, "squash": false, "should_remove_source_branch": false,
        })
        .to_string();
        let r = match self.con.pedir(
            MERGE_PR,
            Rol::Persona,
            self.ep(&Endpoint::Merge(pr)),
            Some(&datos),
        ) {
            Ok(r) => r,
            Err(i) => return i,
        };
        if r.estado == 401 {
            // El token de persona ya se valido con `GET /user`: GitLab da
            // 401 en el merge cuando la cuenta no puede mergear.
            return Informe::nuevo(MERGE_PR, Resultado::Denegada)
                .con("pr", pr.to_string())
                .con("motivo", "permisos")
                .con("rol", Rol::Persona.nombre())
                .diagnostico(
                    Mensaje::nuevo(&t::MERGE_NO_PERMITIDO_GITLAB)
                        .con_mensaje("permiso", PERMISO_MERGE)
                        .con("api", errores::mensaje_api(&r)),
                );
        }
        if !r.exito() {
            let mut i = self.con.traducir(MERGE_PR, Rol::Persona, PERMISO_MERGE, &r);
            i.claves
                .insert(0, ("pr".to_owned(), Valor::Texto(pr.to_string())));
            return i;
        }
        match self.con.parsear::<MergeApi>(MERGE_PR, &r.cuerpo) {
            Ok(MergeApi {
                state,
                merge_commit_sha: Some(s),
            }) if state == "merged" && sha_valido(&s) => Informe::nuevo(MERGE_PR, Resultado::Ok)
                .con("pr", pr.to_string())
                .con("commit", corto(&s)),
            // Mergeado aunque la respuesta no traiga un sha legible.
            Ok(m) if m.state == "merged" => {
                Informe::nuevo(MERGE_PR, Resultado::Ok).con("pr", pr.to_string())
            }
            Ok(_) => self.con.invalida(MERGE_PR, "state"),
            Err(i) => i,
        }
    }

    fn leer_puertas(&self, rol: Rol, rama: &str) -> Paso<ReglasLeidas> {
        validar_puertas(rol, rama)?;
        let comun = self.puertas_comunes(rol)?;
        Ok(self.puertas_de_rama(rol, rama, &comun, true))
    }

    fn leer_puertas_de(
        &self,
        rol: Rol,
        ramas: &[&str],
        checks_de: &str,
    ) -> Vec<Paso<ReglasLeidas>> {
        // Ramas protegidas, reglas de aprobacion y proyecto son del
        // proyecto: se leen una vez (R1-06 de TASK-035).
        let validas: Vec<Paso<()>> = ramas.iter().map(|r| validar_puertas(rol, r)).collect();
        let comun = if validas.iter().any(Result::is_ok) {
            Some(self.puertas_comunes(rol))
        } else {
            None
        };
        ramas
            .iter()
            .zip(validas)
            .map(|(rama, v)| {
                v?;
                match comun.as_ref() {
                    Some(Ok(c)) => Ok(self.puertas_de_rama(rol, rama, c, *rama == checks_de)),
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
        // El job token de GitLab no puede aprobar MR: es fijo de la
        // plataforma, no un ajuste del proyecto (G16, tabla de F1-13).
        Informe::nuevo(LEER_PERMISOS_CI, Resultado::Ok)
            .con("ci-aprueba", "false")
            .con("fuente", "plataforma")
    }

    fn firma_commit(&self, sha: &str) -> Paso<Firma> {
        if !sha_valido(sha) {
            return Err(uso(FIRMA_COMMIT, "sha-invalido", &t::SHA_COMPLETO));
        }
        let r = self.con.pedir(
            FIRMA_COMMIT,
            Rol::Agente,
            self.ep(&Endpoint::Firma(sha)),
            None,
        )?;
        if r.estado == 404 {
            // Sin firma o sin commit: se distingue leyendo el commit.
            let _: CommitApi = self.con.leer(
                FIRMA_COMMIT,
                Rol::Agente,
                PERMISO_LECTURA,
                self.ep(&Endpoint::Commit(sha)),
            )?;
            return Ok(Firma {
                verificada: false,
                motivo: "unsigned".to_owned(),
                committer: None,
                clave: None,
            });
        }
        if !r.exito() {
            return Err(self
                .con
                .traducir(FIRMA_COMMIT, Rol::Agente, PERMISO_LECTURA, &r));
        }
        let f: FirmaApi = self.con.parsear(FIRMA_COMMIT, &r.cuerpo)?;
        // `verified_system` es la firma de la propia GitLab (commits web):
        // no es de una persona.
        Ok(Firma {
            verificada: f.verification_status == "verified",
            motivo: errores::sanear(&f.verification_status),
            committer: None,
            clave: match f.signature_type.as_str() {
                "SSH" => f.key.map(|k| Clave::Ssh(k.id)),
                "PGP" => f.gpg_key_id.map(Clave::Gpg),
                _ => None,
            },
        })
    }

    fn identidad(&self, rol: Rol) -> Paso<Usuario> {
        self.con.identidad(rol, || {
            self.con
                .leer::<UsuarioApi>(IDENTIDAD, rol, PERMISO_LECTURA, self.ep(&Endpoint::Usuario))
                .map(Usuario::from)
        })
    }

    fn proveedor(&self) -> Option<Proveedor> {
        Some(Proveedor::GitLab)
    }

    fn leer_plan(&self, rol: Rol) -> Paso<PlanObservado> {
        if rol == Rol::Revisor {
            return Err(rol_no_valido(LEER_PLAN, rol));
        }
        let p: ProyectoApi = self.con.leer(
            LEER_PLAN,
            rol,
            PERMISO_LECTURA,
            self.ep(&Endpoint::Proyecto),
        )?;
        let Some(ns) = p.namespace else {
            return Ok(PlanObservado::GitLab(None));
        };
        // El `plan` del namespace (API de namespaces; la pagina no dice
        // quien lo ve): si no se lee o no se reconoce, no se supone ninguno.
        let r = self
            .con
            .pedir(LEER_PLAN, rol, self.ep(&Endpoint::Namespace(ns.id)), None)?;
        if !r.exito() {
            return Ok(PlanObservado::GitLab(None));
        }
        let n: NamespaceApi = self.con.parsear(LEER_PLAN, &r.cuerpo)?;
        Ok(PlanObservado::GitLab(
            n.plan.as_deref().and_then(TierGitLab::de_plan),
        ))
    }

    fn preparar_puertas(&self, ps: &[Puerta], origenes: &Origenes) -> Paso<PuertasLeidas> {
        self.leer_para_aplicar(ps, origenes).map(|(l, _)| l)
    }

    fn aplicar_puertas(&self, ps: &[Puerta], origenes: &Origenes) -> Informe {
        let op = puertas::APLICAR_PUERTAS;
        let rol = Rol::Persona;
        let (leidas, pa) = match self.leer_para_aplicar(ps, origenes) {
            Ok(x) => x,
            Err(i) => return i,
        };
        let mut ajustes = pa.ajustes_bien;
        let mut aprobacion = pa.con_aprobacion;
        // Una escritura; 409 es «ya existe» (otro llego antes).
        let crear = |ep: Endpoint<'_>, datos: serde_json::Value| -> Paso<bool> {
            let r = self
                .con
                .pedir(op, rol, self.ep(&ep), Some(&datos.to_string()))?;
            if r.exito() {
                Ok(true)
            } else if r.estado == 409 {
                Ok(false)
            } else {
                Err(self.con.traducir(op, rol, PERMISO_MERGE, &r))
            }
        };
        aplicar_en_lote(ps, &leidas, |p| {
            let mut nueva = false;
            match &p.objetivo {
                Objetivo::Ramas(r) => {
                    let n = niveles_de(p);
                    for nombre in r.iter().map(|x| nombre_gitlab(x)) {
                        if pa.ramas.contains(&nombre) {
                            continue;
                        }
                        nueva |= crear(
                            Endpoint::CrearRamaProtegida,
                            serde_json::json!({
                                "name": nombre,
                                "push_access_level": n.push,
                                "merge_access_level": n.merge,
                                "allow_force_push": false,
                            }),
                        )?;
                    }
                }
                Objetivo::Tags(t) => {
                    for nombre in t.iter().map(|x| nombre_gitlab(x)) {
                        if pa.tags.contains(&nombre) {
                            continue;
                        }
                        nueva |= crear(
                            Endpoint::CrearTagProtegido,
                            serde_json::json!({
                                "name": nombre,
                                "create_access_level": DEVELOPER,
                            }),
                        )?;
                    }
                }
            }
            if let Some(pr) = p.solo_pr {
                if !ajustes {
                    let r = self.con.pedir(
                        op,
                        rol,
                        self.ep(&Endpoint::AjustesProyecto),
                        Some(
                            &serde_json::json!({
                                "merge_method": "merge",
                                "only_allow_merge_if_pipeline_succeeds": true,
                            })
                            .to_string(),
                        ),
                    )?;
                    if !r.exito() {
                        return Err(self.con.traducir(op, rol, PERMISO_MERGE, &r));
                    }
                    ajustes = true;
                    nueva = true;
                }
                if pa.premium && !aprobacion {
                    nueva |= crear(
                        Endpoint::CrearReglaAprobacion,
                        serde_json::json!({
                            "name": REGLA_APROBACION,
                            "approvals_required": pr.aprobaciones,
                            "rule_type": "any_approver",
                        }),
                    )?;
                    aprobacion = true;
                }
            }
            Ok(if nueva {
                Creada::Nueva
            } else {
                Creada::YaExistia
            })
        })
    }
}

/// Lo leido por `aplicar-puertas` que la escritura necesita.
struct ParaAplicar {
    /// Nombres de las ramas protegidas.
    ramas: Vec<String>,
    /// Nombres de los tags protegidos.
    tags: Vec<String>,
    /// El proyecto ya mergea con merge commit y exige el pipeline.
    ajustes_bien: bool,
    /// Premium o Ultimate: hay reglas de aprobacion.
    premium: bool,
    /// Ya hay una regla de aprobacion que pide al menos una.
    con_aprobacion: bool,
}

impl GitLab {
    /// Las lecturas de `aplicar-puertas`: la persona de la lista, lo que ya
    /// esta protegido, D2 (cada check ya publicado en la cabeza de
    /// `develop`: `revision-ia` por el revisor, los del pipeline por
    /// cualquiera), los ajustes del proyecto y el tier. Lo existente se
    /// reconoce por la rama o el tag protegido, no por el nombre de una
    /// regla de aprobacion.
    fn leer_para_aplicar(
        &self,
        ps: &[Puerta],
        origenes: &Origenes,
    ) -> Paso<(PuertasLeidas, ParaAplicar)> {
        let op = puertas::APLICAR_PUERTAS;
        let rol = Rol::Persona;
        self.exigir_persona(op)?;
        let (_, ruta) = self.ep(&Endpoint::RamasProtegidas);
        let ramas: Vec<RamaProtegidaApi> =
            self.con
                .leer_paginas(op, rol, PERMISO_MERGE, ruta, |c| self.con.parsear(op, c))?;
        let (_, ruta) = self.ep(&Endpoint::TagsProtegidos);
        let tags: Vec<TagProtegidoApi> =
            self.con
                .leer_paginas(op, rol, PERMISO_MERGE, ruta, |c| self.con.parsear(op, c))?;
        let develop: RamaApi = self.con.leer(
            op,
            rol,
            PERMISO_LECTURA,
            self.ep(&Endpoint::Rama("develop")),
        )?;
        if !sha_valido(&develop.commit.id) {
            return Err(self.con.invalida_por(op, &t::RAMA_DESTINO_O_SHA));
        }
        let (_, ruta) = self.ep(&Endpoint::Estados(&develop.commit.id));
        let estados: Vec<EstadoApi> =
            self.con
                .leer_paginas(op, rol, PERMISO_LECTURA, ruta, |c| self.con.parsear(op, c))?;
        let vistos: Vec<(String, u64)> =
            estados.into_iter().map(|e| (e.name, e.author.id)).collect();
        let faltan = puertas::no_vistos(origenes, &vistos);
        if !faltan.is_empty() {
            return Err(puertas::sin_origen_visto(faltan));
        }
        let proyecto: ProyectoApi =
            self.con
                .leer(op, rol, PERMISO_LECTURA, self.ep(&Endpoint::Proyecto))?;
        let ajustes_bien = proyecto.merge_method.as_deref() == Some("merge")
            && proyecto.only_allow_merge_if_pipeline_succeeds == Some(true);
        let premium = matches!(
            self.leer_plan(rol).map_err(|mut i| {
                op.clone_into(&mut i.comando);
                i
            })?,
            PlanObservado::GitLab(Some(t)) if t >= TierGitLab::Premium
        );
        let con_aprobacion = premium && {
            let reglas: Vec<ReglaAprobacionApi> = self.con.leer(
                op,
                rol,
                PERMISO_LECTURA,
                self.ep(&Endpoint::ReglasAprobacion),
            )?;
            reglas.iter().any(|r| r.approvals_required >= 1)
        };
        let mut leidas = PuertasLeidas::default();
        for p in ps {
            let estado = match &p.objetivo {
                Objetivo::Ramas(r) => {
                    let n = niveles_de(p);
                    let mut e = EstadoPuerta::Existe;
                    for nombre in r.iter().map(|x| nombre_gitlab(x)) {
                        match ramas.iter().find(|x| x.name == nombre) {
                            None => e = EstadoPuerta::Falta,
                            Some(x)
                                if x.allow_force_push
                                    || !solo_nivel(&x.push_access_levels, n.push)
                                    || !solo_nivel(&x.merge_access_levels, n.merge) =>
                            {
                                if e != EstadoPuerta::Falta {
                                    e = EstadoPuerta::Difiere;
                                }
                            }
                            Some(_) => {}
                        }
                    }
                    if e == EstadoPuerta::Existe
                        && p.solo_pr.is_some()
                        && (!ajustes_bien || (premium && !con_aprobacion))
                    {
                        e = EstadoPuerta::Difiere;
                    }
                    e
                }
                Objetivo::Tags(t) => {
                    let mut e = EstadoPuerta::Existe;
                    for nombre in t.iter().map(|x| nombre_gitlab(x)) {
                        match tags.iter().find(|x| x.name == nombre) {
                            None => e = EstadoPuerta::Falta,
                            Some(x) if !solo_nivel(&x.create_access_levels, DEVELOPER) => {
                                if e != EstadoPuerta::Falta {
                                    e = EstadoPuerta::Difiere;
                                }
                            }
                            Some(_) => {}
                        }
                    }
                    e
                }
            };
            leidas.puertas.push((p.nombre_completo(), estado));
        }
        if !premium {
            leidas.notas.push(("sin-aprobaciones", "si".to_owned()));
        }
        Ok((
            leidas,
            ParaAplicar {
                ramas: ramas.into_iter().map(|r| r.name).collect(),
                tags: tags.into_iter().map(|t| t.name).collect(),
                ajustes_bien,
                premium,
                con_aprobacion,
            },
        ))
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

/// Lo que `leer-puertas` lee del proyecto, comun a todas las ramas.
struct PuertasComunes {
    ramas: Vec<RamaProtegidaApi>,
    plan_da_reglas: bool,
    reglas: Vec<ReglaAprobacionApi>,
    requeridos: Option<Vec<String>>,
    todos: bool,
}

impl GitLab {
    /// Ramas protegidas, reglas de aprobacion (un 403: `plan_da_reglas`
    /// falso) y checks requeridos del proyecto.
    fn puertas_comunes(&self, rol: Rol) -> Paso<PuertasComunes> {
        let (_, ruta) = self.ep(&Endpoint::RamasProtegidas);
        let ramas: Vec<RamaProtegidaApi> =
            self.con
                .leer_paginas(LEER_PUERTAS, rol, PERMISO_LECTURA, ruta, |c| {
                    self.con.parsear(LEER_PUERTAS, c)
                })?;
        let r = self.con.pedir(
            LEER_PUERTAS,
            rol,
            self.ep(&Endpoint::ReglasAprobacion),
            None,
        )?;
        let (plan_da_reglas, reglas) = if r.estado == 403 {
            (false, Vec::new())
        } else if r.exito() {
            let v: Vec<ReglaAprobacionApi> = self.con.parsear(LEER_PUERTAS, &r.cuerpo)?;
            (true, v)
        } else {
            return Err(self.con.traducir(LEER_PUERTAS, rol, PERMISO_LECTURA, &r));
        };
        let (requeridos, todos) = self.checks_requeridos(rol);
        Ok(PuertasComunes {
            ramas,
            plan_da_reglas,
            reglas,
            requeridos,
            todos,
        })
    }

    /// Las puertas de `rama` con lo comun ya leido; los estados de su
    /// cabeza solo `con_checks`.
    fn puertas_de_rama(
        &self,
        rol: Rol,
        rama: &str,
        c: &PuertasComunes,
        con_checks: bool,
    ) -> ReglasLeidas {
        let mut reglas_rama = Vec::new();
        if let Some(p) = c.ramas.iter().find(|p| p.name == rama) {
            reglas_rama.extend(
                p.push_access_levels
                    .iter()
                    .map(|n| format!("push={}", n.access_level)),
            );
            reglas_rama.extend(
                p.merge_access_levels
                    .iter()
                    .map(|n| format!("merge={}", n.access_level)),
            );
            if p.allow_force_push {
                reglas_rama.push("force-push".to_owned());
            }
            if p.code_owner_approval_required {
                reglas_rama.push("code-owner".to_owned());
            }
        }
        let (checks_publicados, fallo_checks) = if con_checks {
            match self.checks_publicados(rol, rama) {
                Ok(v) => (Some(v), None),
                Err(f) => (None, Some(f)),
            }
        } else {
            (None, None)
        };
        ReglasLeidas {
            plan_da_reglas: c.plan_da_reglas,
            aprobaciones: c
                .reglas
                .iter()
                .map(|r| r.approvals_required)
                .max()
                .unwrap_or(0),
            rulesets: c.reglas.iter().map(|r| r.name.clone()).collect(),
            reglas_rama,
            checks_requeridos: c.requeridos.clone(),
            todos_los_checks: c.todos,
            checks_publicados,
            fallo_checks,
            descartar_viejas: false,
            ultimo_push: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proyecto_codificado_una_vez() {
        assert_eq!(codificar_proyecto("7").as_deref(), Some("7"));
        assert_eq!(codificar_proyecto("g/p").as_deref(), Some("g%2Fp"));
        assert_eq!(
            codificar_proyecto("g/s.b/p-1_x").as_deref(),
            Some("g%2Fs.b%2Fp-1_x")
        );
        for p in [
            "", "0", "p", "g/", "/p", "g//p", "g/..", "g/./p", "g/p?x", "g/p%2F", "g/-p", "g/.p",
            "g/p.", "g p/x",
        ] {
            assert_eq!(codificar_proyecto(p), None, "{p}");
        }
    }

    #[test]
    fn solo_success_es_verde() {
        assert_eq!(estado_check("success"), EstadoCheck::Ok);
        assert_eq!(estado_check("failed"), EstadoCheck::Fallo);
        assert_eq!(estado_check("canceled"), EstadoCheck::Fallo);
        for s in ["pending", "running", "skipped", "manual", "created", "otro"] {
            assert_eq!(estado_check(s), EstadoCheck::Pendiente, "{s}");
        }
    }
}
