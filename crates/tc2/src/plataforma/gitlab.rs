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
use super::puertas::{self, EstadoCheck, Lecturas};
use super::{
    Clave, Firma, ModeloPuertas, Paso, Personas, Plataforma, Proveedor, ReglasLeidas, Usuario,
};
use crate::contrato::{Informe, Resultado, Valor, corto};

const ABRIR_PR: &str = "abrir-pr";
const MERGE_PR: &str = "merge-pr";
const LEER_PUERTAS: &str = "leer-puertas";
const LEER_PERMISOS_CI: &str = "leer-permisos-ci";
const FIRMA_COMMIT: &str = "firma-commit";
const IDENTIDAD: &str = "identidad";

/// Permiso de lectura que necesita un token de GitLab.
const PERMISO_LECTURA: &str = "el scope api (o read_api) y el rol Developer en el proyecto";

/// Permiso para abrir un MR.
const PERMISO_ESCRITURA: &str = "el scope api y el rol Developer en el proyecto";

/// Permiso para mergear en una rama protegida.
const PERMISO_MERGE: &str = "el scope api y el rol Maintainer en el proyecto";

/// Garantias que en GitLab solo da Premium o Ultimate (tabla de F1-13): sin
/// reglas de aprobacion (403 en Free), la politica no puede exigirlas
/// impuestas.
const GARANTIAS_PREMIUM: [&str; 6] = ["G03", "G04", "G05", "G06", "G10", "G12"];

/// Nivel de acceso de Maintainer.
const MAINTAINER: u64 = 40;

/// Endpoints de la API que usa el adaptador (lista cerrada).
enum Endpoint<'a> {
    Usuario,
    Mr(u64),
    MrConDivergencia(u64),
    MrsAbiertos { rama: &'a str, base: &'a str },
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
    #[serde(default)]
    push_access_levels: Vec<NivelApi>,
    allow_force_push: bool,
    #[serde(default)]
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
    #[serde(default)]
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
                .diagnostico(
                    "el proyecto de GitLab tiene que ser su id numerico o grupo/proyecto",
                ));
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
                    .diagnostico(
                        "la rama no tiene commits nuevos respecto a la base: no hay nada que revisar",
                    );
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
                    .diagnostico(format!(
                        "GitLab dice que ya hay un MR para la rama pero no se encuentra abierto: reintenta (GitLab: «{}»)",
                        errores::mensaje_api(&r)
                    )),
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
            return self.con.invalida(op, "rama destino o sha del MR");
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
                puertas::rechazada_por_config(
                    pr,
                    g,
                    "GitLab no impone aprobaciones en este proyecto: crea una regla de aprobacion con al menos una aprobacion (requiere Premium o Ultimate) o quita la garantia de la politica",
                )
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
            return puertas::rechazada_por_config(
                pr,
                "G01",
                "la rama destino no esta protegida con merge solo para Maintainers y sin force-push: protegela (taskctl setup lo propone)",
            );
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
            return uso(
                MERGE_PR,
                "sha-invalido",
                "el sha esperado tiene que ser completo (40 hex)",
            );
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
                .diagnostico("el MR esta cerrado sin mergear");
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
                .diagnostico(format!(
                    "GitLab no deja mergear a esta cuenta: necesita {PERMISO_MERGE} (GitLab: «{}»)",
                    errores::mensaje_api(&r)
                ));
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
        let (_, ruta) = self.ep(&Endpoint::RamasProtegidas);
        let ramas: Vec<RamaProtegidaApi> =
            self.con
                .leer_paginas(LEER_PUERTAS, rol, PERMISO_LECTURA, ruta, |c| {
                    self.con.parsear(LEER_PUERTAS, c)
                })?;
        let mut reglas_rama = Vec::new();
        if let Some(p) = ramas.iter().find(|p| p.name == rama) {
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
        let r = self.con.pedir(
            LEER_PUERTAS,
            rol,
            self.ep(&Endpoint::ReglasAprobacion),
            None,
        )?;
        if r.estado == 403 {
            return Ok(ReglasLeidas {
                plan_da_reglas: false,
                rulesets: Vec::new(),
                reglas_rama,
            });
        }
        if !r.exito() {
            return Err(self.con.traducir(LEER_PUERTAS, rol, PERMISO_LECTURA, &r));
        }
        let reglas: Vec<ReglaAprobacionApi> = self.con.parsear(LEER_PUERTAS, &r.cuerpo)?;
        Ok(ReglasLeidas {
            plan_da_reglas: true,
            rulesets: reglas.into_iter().map(|r| r.name).collect(),
            reglas_rama,
        })
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
            return Err(uso(
                FIRMA_COMMIT,
                "sha-invalido",
                "el sha tiene que ser completo (40 hex)",
            ));
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
