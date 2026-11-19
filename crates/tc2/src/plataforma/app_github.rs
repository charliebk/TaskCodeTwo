//! Credenciales de una GitHub App (F6-1; ADR F1-6, decisiones 2 y 5).
//!
//! Una App no tiene token fijo: el CLI firma un JWT RS256 con la clave
//! privada de la App (`iss` = id de la App, `iat` 60 s en el pasado, `exp` a
//! 9 minutos), busca la instalacion del repo (`GET
//! /repos/{o}/{r}/installation`) y pide un token de instalacion (`POST
//! /app/installations/{id}/access_tokens`) **reducido a ese repo y a los
//! permisos de la tabla 2 del ADR para el rol** ([`permisos`]). El token se
//! guarda hasta `expires_at` menos [`MARGEN`] y luego se renueva.
//!
//! Habla con la API por el mismo [`Cliente`] (y la misma [`Base`]) que el
//! adaptador: ruta validada, sin redirecciones y sin el token en errores ni
//! registro. La clave se lee con `ring` (ya en el arbol por rustls): PEM
//! PKCS#1 (`BEGIN RSA PRIVATE KEY`, el que descarga GitHub) o PKCS#8
//! (`BEGIN PRIVATE KEY`).

use std::cell::{Cell, RefCell};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use ring::rand::SystemRandom;
use ring::signature::{RSA_PKCS1_SHA256, RsaKeyPair};
use serde::Deserialize;

use super::credenciales::{Credenciales, FalloCredencial, Rol, Secreto, causas, id_valido};
use super::errores::sanear;
use super::github::repo_valido;
use super::http::{Base, Cliente, FalloHttp, Metodo, Respuesta};

/// Antes de `expires_at` se deja de usar un token y se pide otro (los
/// tokens de instalacion duran 1 h).
pub const MARGEN: u64 = 5 * 60;

/// El JWT se firma con `iat` este numero de segundos en el pasado (deriva de
/// reloj, como pide GitHub).
const ATRAS: u64 = 60;

/// Vida del JWT (GitHub admite hasta 10 minutos).
const VIDA_JWT: u64 = 9 * 60;

/// Permisos del token de instalacion de cada rol: la tabla 2 del ADR F1-6.
/// La persona no tiene App.
pub fn permisos(rol: Rol) -> Option<&'static [(&'static str, &'static str)]> {
    match rol {
        Rol::Agente => Some(&[
            ("contents", "write"),
            ("pull_requests", "write"),
            ("checks", "read"),
            ("statuses", "read"),
            ("metadata", "read"),
        ]),
        Rol::Revisor => Some(&[
            ("checks", "write"),
            ("contents", "read"),
            ("pull_requests", "read"),
            ("metadata", "read"),
        ]),
        Rol::Persona => None,
    }
}

#[derive(Deserialize)]
struct InstalacionApi {
    id: u64,
}

#[derive(Deserialize)]
struct TokenApi {
    token: String,
    expires_at: String,
    /// Permisos que concede de verdad (GitHub los devuelve; si faltan en la
    /// respuesta no se puede comprobar y se acepta).
    #[serde(default)]
    permissions: Option<std::collections::BTreeMap<String, String>>,
}

#[derive(Deserialize)]
struct AppApi {
    slug: String,
}

/// Credenciales de la App de un rol para un repo.
pub struct CredencialesApp {
    id: String,
    rol: Rol,
    clave: RsaKeyPair,
    cliente: Cliente,
    repo: String,
    instalacion: Cell<Option<u64>>,
    cache: RefCell<Option<(Secreto, u64)>>,
}

impl std::fmt::Debug for CredencialesApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CredencialesApp")
            .field("id", &self.id)
            .field("rol", &self.rol)
            .field("repo", &self.repo)
            .finish_non_exhaustive()
    }
}

impl CredencialesApp {
    /// App `id` del `rol` con su clave en PEM, para `repo` (`dueno/repo`) en
    /// la API `base`.
    pub fn nueva(
        id: &str,
        pem: &str,
        base: Base,
        repo: &str,
        rol: Rol,
    ) -> Result<Self, FalloCredencial> {
        let incompleta = |detalle: &str| FalloCredencial::Acunar {
            causa: causas::APP_INCOMPLETA,
            detalle: detalle.to_owned(),
        };
        if !id_valido(id) {
            return Err(incompleta("id"));
        }
        if permisos(rol).is_none() {
            return Err(incompleta(rol.nombre()));
        }
        if !repo_valido(repo) {
            return Err(incompleta("repo"));
        }
        Ok(Self {
            id: id.to_owned(),
            rol,
            clave: leer_pem(pem)?,
            cliente: Cliente::nuevo(base),
            repo: repo.to_owned(),
            instalacion: Cell::new(None),
            cache: RefCell::new(None),
        })
    }

    /// Como [`CredencialesApp::nueva`], con la clave en el fichero `ruta`.
    pub fn desde_fichero(
        id: &str,
        ruta: &Path,
        base: &Base,
        repo: &str,
        rol: Rol,
    ) -> Result<Self, FalloCredencial> {
        let pem = std::fs::read_to_string(ruta).map_err(|e| FalloCredencial::Acunar {
            causa: causas::CLAVE_ILEGIBLE,
            detalle: sanear(&e.kind().to_string()),
        })?;
        Self::nueva(id, &pem, base.clone(), repo, rol)
    }

    /// Llamadas hechas, `METODO ruta`.
    pub fn registro(&self) -> Vec<String> {
        self.cliente.registro()
    }

    /// El `slug` de la App (`GET /app` con el JWT): su cuenta es
    /// `<slug>[bot]`.
    pub fn slug(&self) -> Result<String, FalloCredencial> {
        let jwt = self.firmar(ahora())?;
        let r = self.llamar(Metodo::Get, "/app", &jwt, None)?;
        if r.estado != 200 {
            return Err(fallo_estado(&r, causas::RESPUESTA_INESPERADA));
        }
        let a: AppApi = leer(&r)?;
        Ok(a.slug)
    }

    /// JWT RS256 de la App para el instante `ahora`.
    fn firmar(&self, ahora: u64) -> Result<Secreto, FalloCredencial> {
        let cabecera = URL_SAFE_NO_PAD.encode(r#"{"alg":"RS256","typ":"JWT"}"#);
        // El id numerico va como numero; un client id (`Iv1.…`), como texto
        // (`id_valido` ya excluye comillas y barras).
        let iss = if self.id.bytes().all(|b| b.is_ascii_digit()) {
            self.id.clone()
        } else {
            format!("\"{}\"", self.id)
        };
        let carga = format!(
            r#"{{"iat":{},"exp":{},"iss":{iss}}}"#,
            ahora.saturating_sub(ATRAS),
            ahora + VIDA_JWT
        );
        let entrada = format!("{cabecera}.{}", URL_SAFE_NO_PAD.encode(carga));
        let mut firma = vec![0u8; self.clave.public().modulus_len()];
        self.clave
            .sign(
                &RSA_PKCS1_SHA256,
                &SystemRandom::new(),
                entrada.as_bytes(),
                &mut firma,
            )
            .map_err(|_| FalloCredencial::Acunar {
                causa: causas::CLAVE_ILEGIBLE,
                detalle: "firma".to_owned(),
            })?;
        Ok(Secreto::nuevo(format!(
            "{entrada}.{}",
            URL_SAFE_NO_PAD.encode(firma)
        )))
    }

    fn llamar(
        &self,
        metodo: Metodo,
        ruta: &str,
        jwt: &Secreto,
        cuerpo: Option<&str>,
    ) -> Result<Respuesta, FalloCredencial> {
        self.cliente
            .llamar(metodo, ruta, Some(jwt), cuerpo)
            .map_err(|f| fallo_transporte(&f))
    }

    /// Id de la instalacion de la App en el repo (una vez por proceso).
    fn instalacion(&self, jwt: &Secreto) -> Result<u64, FalloCredencial> {
        if let Some(id) = self.instalacion.get() {
            return Ok(id);
        }
        let ruta = format!("/repos/{}/installation", self.repo);
        let r = self.llamar(Metodo::Get, &ruta, jwt, None)?;
        if r.estado != 200 {
            return Err(fallo_estado(&r, causas::RESPUESTA_INESPERADA));
        }
        let i: InstalacionApi = leer(&r)?;
        self.instalacion.set(Some(i.id));
        Ok(i.id)
    }

    /// Pide un token nuevo, reducido al repo y a los permisos del rol.
    fn acunar(&self, ahora: u64) -> Result<(Secreto, u64), FalloCredencial> {
        let jwt = self.firmar(ahora)?;
        let instalacion = self.instalacion(&jwt)?;
        let nombre = self.repo.split('/').nth(1).unwrap_or_default();
        let permisos: serde_json::Map<String, serde_json::Value> = permisos(self.rol)
            .unwrap_or_default()
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).into()))
            .collect();
        let cuerpo = serde_json::json!({
            "repositories": [nombre],
            "permissions": permisos,
        })
        .to_string();
        let ruta = format!("/app/installations/{instalacion}/access_tokens");
        let r = self.llamar(Metodo::Post, &ruta, &jwt, Some(&cuerpo))?;
        if r.estado != 201 {
            if r.estado == 404 {
                // La App se desinstalo o se reinstalo con otro id.
                self.instalacion.set(None);
            }
            return Err(fallo_estado(&r, causas::PERMISOS_NO_CONCEDIDOS));
        }
        let t: TokenApi = leer(&r)?;
        let expira = crate::inventario::ms_de_iso(&t.expires_at)
            .and_then(|ms| u64::try_from(ms / 1000).ok())
            .ok_or_else(|| FalloCredencial::Acunar {
                causa: causas::RESPUESTA_INESPERADA,
                detalle: "expires_at".to_owned(),
            })?;
        if t.token.trim().is_empty() || t.token.chars().any(char::is_control) {
            return Err(FalloCredencial::Acunar {
                causa: causas::RESPUESTA_INESPERADA,
                detalle: "token".to_owned(),
            });
        }
        if let Some(concedidos) = &t.permissions
            && let Some(falta) = permiso_que_falta(self.rol, concedidos)
        {
            return Err(FalloCredencial::Acunar {
                causa: causas::PERMISOS_NO_CONCEDIDOS,
                detalle: falta,
            });
        }
        Ok((Secreto::nuevo(t.token), expira))
    }
}

impl Credenciales for CredencialesApp {
    fn token(&self, rol: Rol) -> Option<Secreto> {
        self.obtener(rol).ok()
    }

    fn obtener(&self, rol: Rol) -> Result<Secreto, FalloCredencial> {
        if rol != self.rol {
            return Err(FalloCredencial::Falta);
        }
        let ahora = ahora();
        if let Some((s, expira)) = self.cache.borrow().as_ref()
            && expira.saturating_sub(MARGEN) > ahora
        {
            return Ok(s.clone());
        }
        let (s, expira) = self.acunar(ahora)?;
        *self.cache.borrow_mut() = Some((s.clone(), expira));
        Ok(s)
    }

    fn tiene(&self, rol: Rol) -> bool {
        rol == self.rol
    }
}

/// Segundos desde 1970.
fn ahora() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Primer permiso del rol que el token no concede con el nivel pedido
/// (`write` cubre `read`), como `permiso:nivel`.
fn permiso_que_falta(
    rol: Rol,
    concedidos: &std::collections::BTreeMap<String, String>,
) -> Option<String> {
    permisos(rol)
        .unwrap_or_default()
        .iter()
        .find(|(k, v)| {
            let dado = concedidos.get(*k).map(String::as_str);
            !(dado == Some(*v) || (*v == "read" && dado == Some("write")))
        })
        .map(|(k, v)| format!("{k}:{v}"))
}

/// Fallo del transporte al pedir el token: sin red, `sinred`; TLS, `tls`
/// (el diagnostico dice que revisar); el resto, `respuesta-inesperada` con
/// un detalle legible (nunca el `Debug` de la variante).
fn fallo_transporte(f: &FalloHttp) -> FalloCredencial {
    let (causa, detalle) = match f {
        FalloHttp::Sinred(m) => (causas::SINRED, sanear(m)),
        FalloHttp::Tls(m) => (causas::TLS, sanear(m)),
        FalloHttp::RespuestaInvalida(m) => (
            causas::RESPUESTA_INESPERADA,
            sanear(&format!("respuesta ilegible: {m}")),
        ),
        FalloHttp::CuerpoGrande => (
            causas::RESPUESTA_INESPERADA,
            "respuesta demasiado grande".to_owned(),
        ),
        FalloHttp::Redireccion(e) => (
            causas::RESPUESTA_INESPERADA,
            format!("redireccion {e}: revisa la URL de la API"),
        ),
        FalloHttp::Rechazada(_) => (
            causas::RESPUESTA_INESPERADA,
            "ruta rechazada antes de la red".to_owned(),
        ),
        FalloHttp::Otro(m) => (
            causas::RESPUESTA_INESPERADA,
            sanear(&format!("cliente HTTP: {m}")),
        ),
    };
    FalloCredencial::Acunar { causa, detalle }
}

/// Fallo por el estado HTTP: 401 es el JWT, 404 la instalacion, 422 los
/// permisos (`en_422`) y el resto, inesperado.
fn fallo_estado(r: &Respuesta, en_422: &'static str) -> FalloCredencial {
    let causa = match r.estado {
        401 => causas::JWT_RECHAZADO,
        404 => causas::SIN_INSTALACION,
        422 => en_422,
        _ => causas::RESPUESTA_INESPERADA,
    };
    let mensaje = serde_json::from_str::<serde_json::Value>(&r.cuerpo)
        .ok()
        .and_then(|v| v.get("message").and_then(|m| m.as_str()).map(sanear))
        .unwrap_or_default();
    FalloCredencial::Acunar {
        causa,
        detalle: sanear(&format!("HTTP {} {mensaje}", r.estado)),
    }
}

fn leer<T: serde::de::DeserializeOwned>(r: &Respuesta) -> Result<T, FalloCredencial> {
    serde_json::from_str(&r.cuerpo).map_err(|e| FalloCredencial::Acunar {
        causa: causas::RESPUESTA_INESPERADA,
        detalle: sanear(&e.to_string()),
    })
}

/// Clave RSA de un PEM PKCS#1 o PKCS#8 sin cifrar.
fn leer_pem(pem: &str) -> Result<RsaKeyPair, FalloCredencial> {
    let ilegible = |detalle: &str| FalloCredencial::Acunar {
        causa: causas::CLAVE_ILEGIBLE,
        detalle: detalle.to_owned(),
    };
    // Un BOM UTF-8 (`Set-Content -Encoding utf8` de Windows PowerShell 5.1)
    // y el texto antes de `-----BEGIN` (comentarios, como admite RFC 7468)
    // se ignoran.
    let mut lineas = pem
        .trim_start_matches('\u{feff}')
        .lines()
        .map(str::trim)
        .skip_while(|l| !l.starts_with("-----BEGIN "));
    let etiqueta = lineas
        .next()
        .ok_or_else(|| ilegible("pem sin -----BEGIN"))?
        .strip_prefix("-----BEGIN ")
        .and_then(|l| l.strip_suffix("-----"))
        .ok_or_else(|| ilegible("pem"))?
        .to_owned();
    let fin = format!("-----END {etiqueta}-----");
    let mut b64 = String::new();
    let mut cerrado = false;
    for l in lineas {
        if l == fin {
            cerrado = true;
            break;
        }
        // Cabeceras (`Proc-Type: 4,ENCRYPTED`): clave cifrada, no se admite.
        if l.contains(':') {
            return Err(ilegible("pem cifrado"));
        }
        b64.push_str(l);
    }
    if !cerrado {
        return Err(ilegible("pem"));
    }
    let der = STANDARD.decode(b64).map_err(|_| ilegible("pem base64"))?;
    let clave = match etiqueta.as_str() {
        "RSA PRIVATE KEY" => RsaKeyPair::from_der(&der),
        "PRIVATE KEY" => RsaKeyPair::from_pkcs8(&der),
        _ => return Err(ilegible("pem no rsa")),
    };
    clave.map_err(|e| ilegible(&e.to_string()))
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;

    /// C2.2 del plan de TASK-047: los permisos por rol son los de la tabla
    /// 2 del ADR F1-6; el revisor no pide escritura de PR ni de contents.
    #[test]
    fn los_permisos_por_rol_son_los_de_la_tabla_del_adr() {
        let agente: Vec<_> = permisos(Rol::Agente).unwrap_or_default().to_vec();
        assert_eq!(
            agente,
            vec![
                ("contents", "write"),
                ("pull_requests", "write"),
                ("checks", "read"),
                ("statuses", "read"),
                ("metadata", "read"),
            ]
        );
        let revisor: Vec<_> = permisos(Rol::Revisor).unwrap_or_default().to_vec();
        assert_eq!(
            revisor,
            vec![
                ("checks", "write"),
                ("contents", "read"),
                ("pull_requests", "read"),
                ("metadata", "read"),
            ]
        );
        for (k, v) in &revisor {
            assert!(
                !(*v == "write" && (*k == "pull_requests" || *k == "contents")),
                "el revisor no puede aprobar ni empujar: {k}:{v}"
            );
        }
        // Ninguna App administra, toca workflows ni escribe statuses.
        for p in [agente, revisor] {
            for (k, v) in p {
                assert!(
                    !["administration", "workflows", "secrets", "issues"].contains(&k),
                    "{k}"
                );
                assert!(!(k == "statuses" && v == "write"));
            }
        }
        assert!(permisos(Rol::Persona).is_none());
    }

    #[test]
    fn pem_que_no_vale() {
        for pem in [
            "",
            "hola",
            "-----BEGIN RSA PRIVATE KEY-----\nAAAA\n",
            "-----BEGIN RSA PRIVATE KEY-----\n%%%%\n-----END RSA PRIVATE KEY-----\n",
            "-----BEGIN RSA PRIVATE KEY-----\nProc-Type: 4,ENCRYPTED\nAAAA\n-----END RSA PRIVATE KEY-----\n",
            "-----BEGIN EC PRIVATE KEY-----\nAAAA\n-----END EC PRIVATE KEY-----\n",
            "-----BEGIN RSA PRIVATE KEY-----\nAAAA\n-----END RSA PRIVATE KEY-----\n",
        ] {
            match leer_pem(pem) {
                Err(FalloCredencial::Acunar { causa, .. }) => {
                    assert_eq!(causa, causas::CLAVE_ILEGIBLE, "{pem}");
                }
                otro => panic!("{pem}: {:?}", otro.map(|_| ())),
            }
        }
    }
}
