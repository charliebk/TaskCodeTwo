//! Cliente HTTP de los arneses E2E contra GitHub y GitLab reales (TASK-139,
//! A1-06): las llamadas que el puerto no tiene, solo para preparar y
//! limpiar. Lo incluyen solo `plataforma_e2e` y `plataforma_gitlab_e2e`
//! (fuera de `comun` para no compilarlo en todos los binarios); cada E2E
//! conserva su `Arnes` con sus rutas (F2-9: la bateria es la misma, cada una
//! contra la API de su plataforma).

use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;
use tc2::contrato::Informe;

/// Agente HTTP del arnes, con la cabecera `Accept` de su plataforma.
pub struct Cliente {
    agente: ureq::Agent,
    accept: &'static str,
}

impl Cliente {
    /// Cliente con TLS del almacen del sistema; un estado HTTP de error
    /// llega como respuesta, no como fallo.
    pub fn nuevo(accept: &'static str) -> Self {
        let agente: ureq::Agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .tls_config(
                ureq::tls::TlsConfig::builder()
                    .root_certs(ureq::tls::RootCerts::PlatformVerifier)
                    .build(),
            )
            .build()
            .into();
        Self { agente, accept }
    }

    /// `metodo` a la URL completa `url` con `token`: `(estado, cuerpo)`, o
    /// `None` sin red o con un metodo (o un cuerpo que falta) que no se
    /// admite. No entra en panico: sirve para limpiar desde `Drop`.
    pub fn intentar_con(
        &self,
        token: &str,
        metodo: &str,
        url: &str,
        cuerpo: Option<Value>,
    ) -> Option<(u16, Value)> {
        let auth = format!("Bearer {token}");
        fn cab<B>(p: ureq::RequestBuilder<B>, auth: &str, accept: &str) -> ureq::RequestBuilder<B> {
            p.header("Authorization", auth)
                .header("Accept", accept)
                .header("Content-Type", "application/json")
                .header("User-Agent", "taskcodetwo-e2e")
        }
        let (a, acc) = (&self.agente, self.accept);
        let r = match (metodo, cuerpo) {
            ("GET", _) => cab(a.get(url), &auth, acc).call(),
            ("DELETE", _) => cab(a.delete(url), &auth, acc).call(),
            ("POST", Some(c)) => cab(a.post(url), &auth, acc).send(c.to_string()),
            ("PUT", Some(c)) => cab(a.put(url), &auth, acc).send(c.to_string()),
            ("PATCH", Some(c)) => cab(a.patch(url), &auth, acc).send(c.to_string()),
            _ => return None,
        };
        let mut r = r.ok()?;
        let estado = r.status().as_u16();
        let texto = r.body_mut().read_to_string().unwrap_or_default();
        Some((estado, serde_json::from_str(&texto).unwrap_or(Value::Null)))
    }
}

/// Sufijo unico para ramas y nombres del E2E.
pub fn sufijo() -> String {
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("{}-{}", t.as_secs(), std::process::id())
}

/// La linea del informe, tambien en stderr para seguir el E2E.
pub fn ver(i: &Informe) -> String {
    let l = i.linea();
    eprintln!("  {l}");
    l
}
