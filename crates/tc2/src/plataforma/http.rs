//! Envoltorio unico de llamadas HTTP del puerto Plataforma (F1-13, D6a y
//! decision 1; F1-14 punto 2).
//!
//! Toda llamada a la API sale por [`Cliente::llamar`], que comprueba antes de
//! abrir la red:
//! - el metodo: solo los de [`Metodo`] (lista cerrada por tipo);
//! - el destino: la ruta es relativa a una [`Base`] fija (la API de GitHub o
//!   GitLab, la que diga la configuracion o, solo para tests,
//!   `http://127.0.0.1`);
//! - la ruta: empieza por `/`, sin segmentos `.` ni `..`, sin `\`, sin
//!   `%2e` ni `%25` (tampoco codificados dos veces) y sin caracteres fuera
//!   de los no reservados. Con GitLab admite ademas `%2F` (el id de un
//!   proyecto con nombre), y los segmentos se comprueban con la barra ya
//!   decodificada.
//!
//! No sigue redirecciones, limita el tamano de la respuesta y los tiempos de
//! espera, y lleva un registro `METODO ruta` sin cabeceras. El token nunca
//! sale en un error ni en el registro.

use std::cell::RefCell;
use std::time::Duration;

use super::Proveedor;
use super::credenciales::Secreto;
use crate::i18n::Texto;
use crate::i18n::catalogo::plataforma as t;

/// Tamano maximo del cuerpo de una respuesta.
pub const LIMITE_CUERPO: u64 = 1024 * 1024;

/// Tiempo maximo de una llamada completa.
const ESPERA_TOTAL: Duration = Duration::from_secs(30);

/// Tiempo maximo para conectar.
const ESPERA_CONEXION: Duration = Duration::from_secs(10);

/// La API publica de GitHub.
pub const API_GITHUB: &str = "https://api.github.com";

/// Metodos permitidos. No hay `DELETE` ni `PATCH`: ninguna operacion de la
/// lista cerrada los usa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Metodo {
    /// Lectura.
    Get,
    /// Crear (abrir un PR).
    Post,
    /// Mergear un PR.
    Put,
}

impl Metodo {
    /// Nombre HTTP.
    pub fn nombre(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
        }
    }
}

/// Origen de la API: esquema, host y, si acaso, puerto y prefijo, sin barra
/// final. Solo `https`, salvo `http://127.0.0.1:<puerto>` para los tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Base(String);

impl Base {
    /// La API publica de GitHub.
    pub fn github() -> Self {
        Self(API_GITHUB.to_owned())
    }

    /// Valida una base. `Err` con el motivo si no vale.
    pub fn nueva(url: &str) -> Result<Self, &'static Texto> {
        let url = url.trim_end_matches('/');
        let resto = if let Some(r) = url.strip_prefix("https://") {
            r
        } else if let Some(r) = url.strip_prefix("http://127.0.0.1:") {
            let (puerto, ruta) = r.split_once('/').unwrap_or((r, ""));
            let valida = puerto_valido(puerto)
                && !ruta.contains(['?', '#'])
                && (ruta.is_empty() || validar_ruta(&format!("/{ruta}")).is_ok());
            if !valida {
                return Err(&t::HTTP_SOLO_LOCAL);
            }
            return Ok(Self(url.to_owned()));
        } else {
            return Err(&t::URL_HTTPS);
        };
        let (autoridad, ruta) = resto.split_once('/').unwrap_or((resto, ""));
        let (host, puerto) = match autoridad.split_once(':') {
            Some((h, p)) => (h, Some(p)),
            None => (autoridad, None),
        };
        let host_valido = !host.is_empty()
            && host
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
            && !host.starts_with(['.', '-'])
            && puerto.is_none_or(puerto_valido);
        if !host_valido {
            return Err(&t::HOST_NO_VALIDO);
        }
        if ruta.contains('?') || ruta.contains('#') {
            return Err(&t::PREFIJO_CON_CONSULTA);
        }
        if !ruta.is_empty() && validar_ruta(&format!("/{ruta}")).is_err() {
            return Err(&t::PREFIJO_NO_VALIDO);
        }
        Ok(Self(url.to_owned()))
    }

    /// Texto de la base.
    pub fn texto(&self) -> &str {
        &self.0
    }
}

/// Puerto decimal entre 1 y 65535.
fn puerto_valido(p: &str) -> bool {
    !p.is_empty()
        && p.len() <= 5
        && p.bytes().all(|b| b.is_ascii_digit())
        && p.parse::<u32>().is_ok_and(|n| (1..=65535).contains(&n))
}

/// Respuesta ya leida: estado, cuerpo y las cabeceras que se usan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Respuesta {
    /// Codigo HTTP.
    pub estado: u16,
    /// Cuerpo como texto (hasta [`LIMITE_CUERPO`]).
    pub cuerpo: String,
    /// `Link` (paginacion).
    pub link: Option<String>,
    /// `x-ratelimit-remaining` (GitHub) o `ratelimit-remaining` (GitLab).
    pub restantes: Option<String>,
    /// `retry-after`, en segundos.
    pub reintentar: Option<String>,
    /// `x-ratelimit-reset` (GitHub) o `ratelimit-reset` (GitLab), en
    /// segundos desde 1970.
    pub reinicio: Option<String>,
}

impl Respuesta {
    /// Si el estado es 2xx.
    pub fn exito(&self) -> bool {
        (200..300).contains(&self.estado)
    }

    /// Siguiente pagina de `Link` (`rel="next"`), si la hay.
    pub fn siguiente(&self) -> Option<&str> {
        let link = self.link.as_deref()?;
        link.split(',').find_map(|parte| {
            let (url, params) = parte.split_once(';')?;
            let es_next = params.split(';').any(|p| p.trim() == "rel=\"next\"");
            let url = url.trim().strip_prefix('<')?.strip_suffix('>')?;
            es_next.then_some(url)
        })
    }
}

/// Por que no hubo respuesta HTTP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FalloHttp {
    /// El envoltorio la rechazo antes de la red, con el motivo.
    Rechazada(&'static Texto),
    /// No se pudo conectar (DNS, conexion, tiempo agotado).
    Sinred(String),
    /// Fallo de TLS (certificado, almacen del sistema).
    Tls(String),
    /// Respuesta mayor que [`LIMITE_CUERPO`] al leerla, o ilegible.
    RespuestaInvalida(String),
    /// La respuesta declara un tamano mayor que [`LIMITE_CUERPO`].
    CuerpoGrande,
    /// La API respondio con una redireccion (no se siguen).
    Redireccion(u16),
    /// Otro fallo del cliente.
    Otro(String),
}

/// Cliente HTTP con su base fija y la plataforma a la que habla.
pub struct Cliente {
    base: Base,
    proveedor: Proveedor,
    agente: ureq::Agent,
    registro: RefCell<Vec<String>>,
}

impl std::fmt::Debug for Cliente {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cliente")
            .field("base", &self.base)
            .field("proveedor", &self.proveedor)
            .finish()
    }
}

impl Cliente {
    /// Cliente de GitHub para una base ya validada.
    pub fn nuevo(base: Base) -> Self {
        Self::para(base, Proveedor::GitHub)
    }

    /// Cliente para una base ya validada y su plataforma (que decide las
    /// cabeceras y si la ruta admite `%2F`).
    pub fn para(base: Base, proveedor: Proveedor) -> Self {
        Self {
            base,
            proveedor,
            agente: agente(ESPERA_TOTAL),
            registro: RefCell::new(Vec::new()),
        }
    }

    /// La base del cliente.
    pub fn base(&self) -> &Base {
        &self.base
    }

    /// Llamadas hechas, `METODO ruta`, en orden.
    pub fn registro(&self) -> Vec<String> {
        self.registro.borrow().clone()
    }

    /// Convierte una URL absoluta de paginacion en ruta, solo si es de la
    /// misma base.
    pub fn ruta_de(&self, url: &str) -> Result<String, FalloHttp> {
        url.strip_prefix(self.base.texto())
            .filter(|r| r.starts_with('/'))
            .map(str::to_owned)
            .ok_or(FalloHttp::Rechazada(&t::PAGINACION_OTRO_DESTINO))
    }

    /// Hace la llamada. `ruta` incluye la consulta (`?a=b`).
    pub fn llamar(
        &self,
        metodo: Metodo,
        ruta: &str,
        token: Option<&Secreto>,
        cuerpo: Option<&str>,
    ) -> Result<Respuesta, FalloHttp> {
        validar_ruta_con(ruta, self.proveedor == Proveedor::GitLab)
            .map_err(FalloHttp::Rechazada)?;
        self.registro
            .borrow_mut()
            .push(format!("{} {}", metodo.nombre(), ruta));
        let url = format!("{}{}", self.base.texto(), ruta);
        let resultado = match metodo {
            Metodo::Get => self.cabeceras(self.agente.get(&url), token).call(),
            Metodo::Post => self
                .cabeceras(self.agente.post(&url), token)
                .header("Content-Type", "application/json")
                .send(cuerpo.unwrap_or("{}")),
            Metodo::Put => self
                .cabeceras(self.agente.put(&url), token)
                .header("Content-Type", "application/json")
                .send(cuerpo.unwrap_or("{}")),
        };
        let mut r = resultado.map_err(clasificar)?;
        let estado = r.status().as_u16();
        if (300..400).contains(&estado) {
            return Err(FalloHttp::Redireccion(estado));
        }
        let cabecera = |n: &str| {
            r.headers()
                .get(n)
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned)
        };
        let link = cabecera("link");
        let restantes =
            cabecera("x-ratelimit-remaining").or_else(|| cabecera("ratelimit-remaining"));
        let reintentar = cabecera("retry-after");
        let reinicio = cabecera("x-ratelimit-reset").or_else(|| cabecera("ratelimit-reset"));
        // Si la respuesta declara su tamano y pasa del limite, no se lee:
        // cortar la lectura a medias deja la conexion en un estado que el
        // sistema puede notificar como desconexion (y se veria `sinred`).
        let declarado = cabecera("content-length").and_then(|v| v.trim().parse::<u64>().ok());
        if declarado.is_some_and(|n| n > LIMITE_CUERPO) {
            return Err(FalloHttp::CuerpoGrande);
        }
        // Sin tamano declarado (chunked), el limite corta al leer.
        let cuerpo = r
            .body_mut()
            .with_config()
            .limit(LIMITE_CUERPO)
            .read_to_string()
            .map_err(clasificar)?;
        Ok(Respuesta {
            estado,
            cuerpo,
            link,
            restantes,
            reintentar,
            reinicio,
        })
    }

    fn cabeceras<B>(
        &self,
        p: ureq::RequestBuilder<B>,
        token: Option<&Secreto>,
    ) -> ureq::RequestBuilder<B> {
        let p = match self.proveedor {
            Proveedor::GitHub => p
                .header("Accept", "application/vnd.github+json")
                .header("X-GitHub-Api-Version", "2022-11-28"),
            Proveedor::GitLab => p.header("Accept", "application/json"),
        };
        match token {
            Some(t) => p.header("Authorization", format!("Bearer {}", t.exponer())),
            None => p,
        }
    }
}

/// Agente ureq de `tc2`: TLS con el almacen del sistema, user-agent
/// `taskcodetwo-tc2/<version>`, sin seguir redirecciones (quien lo usa decide
/// que hacer con un 3xx), sin convertir los estados HTTP en errores, con
/// `espera_total` por llamada y [`ESPERA_CONEXION`] para conectar. Lo usan el
/// [`Cliente`] de la API y la descarga de `actualizar --cli` (F2-12).
pub(crate) fn agente(espera_total: Duration) -> ureq::Agent {
    let tls = ureq::tls::TlsConfig::builder()
        .root_certs(ureq::tls::RootCerts::PlatformVerifier)
        .build();
    ureq::Agent::config_builder()
        .tls_config(tls)
        .http_status_as_error(false)
        .max_redirects(0)
        .max_redirects_will_error(false)
        .save_redirect_history(false)
        .timeout_global(Some(espera_total))
        .timeout_connect(Some(ESPERA_CONEXION))
        .user_agent(format!("taskcodetwo-tc2/{}", crate::VERSION))
        .build()
        .into()
}

/// Clasifica un error de ureq sin copiar nada que pueda llevar el token
/// (ureq no pone cabeceras en sus errores; aun asi solo se usa el tipo).
pub(crate) fn clasificar(e: ureq::Error) -> FalloHttp {
    match e {
        ureq::Error::HostNotFound
        | ureq::Error::ConnectionFailed
        | ureq::Error::Timeout(_)
        | ureq::Error::Io(_)
        | ureq::Error::BodyStalled => FalloHttp::Sinred(tipo_error(&e)),
        ureq::Error::Tls(_)
        | ureq::Error::Rustls(_)
        | ureq::Error::Pem(_)
        | ureq::Error::TlsRequired => FalloHttp::Tls(tipo_error(&e)),
        ureq::Error::BodyExceedsLimit(_) | ureq::Error::LargeResponseHeader(..) => {
            FalloHttp::RespuestaInvalida(tipo_error(&e))
        }
        ureq::Error::TooManyRedirects | ureq::Error::RedirectFailed => FalloHttp::Redireccion(0),
        _ => FalloHttp::Otro(tipo_error(&e)),
    }
}

/// Texto corto del error de ureq (su `Display` no lleva cabeceras).
fn tipo_error(e: &ureq::Error) -> String {
    let t = e.to_string();
    t.chars().filter(|c| !c.is_control()).take(200).collect()
}

/// Comprueba la ruta (con su consulta) antes de la red, sin admitir `%2F`.
pub fn validar_ruta(ruta: &str) -> Result<(), &'static Texto> {
    validar_ruta_con(ruta, false)
}

/// Comprueba la ruta. Con `admite_barra_codificada` (GitLab) acepta la
/// secuencia exacta `%2F` en el camino y comprueba los segmentos con ella ya
/// decodificada: `a%2F..` es un `..` y `%2f` o `%252F` siguen rechazados.
pub fn validar_ruta_con(ruta: &str, admite_barra_codificada: bool) -> Result<(), &'static Texto> {
    let (camino, consulta) = ruta.split_once('?').unwrap_or((ruta, ""));
    if !camino.starts_with('/') || camino.starts_with("//") {
        return Err(&t::RUTA_NO_RELATIVA);
    }
    let minus = ruta.to_ascii_lowercase();
    if ruta.contains('\\') || minus.contains("%2e") || minus.contains("%25") {
        return Err(&t::RUTA_CODIFICADA);
    }
    let decodificado;
    let camino = if admite_barra_codificada {
        decodificado = camino.replace("%2F", "/");
        decodificado.as_str()
    } else {
        camino
    };
    if camino[1..]
        .split('/')
        .any(|s| s.is_empty() || s == "." || s == "..")
    {
        return Err(&t::RUTA_SEGMENTOS);
    }
    let no_reservado = |b: u8| b.is_ascii_alphanumeric() || b"-._~".contains(&b);
    if !camino.bytes().all(|b| no_reservado(b) || b == b'/') {
        return Err(&t::CARACTER_EN_RUTA);
    }
    if !consulta
        .bytes()
        .all(|b| no_reservado(b) || b"=&:%,".contains(&b))
    {
        return Err(&t::CARACTER_EN_CONSULTA);
    }
    Ok(())
}

/// Codifica un valor de consulta (`%XX` todo lo que no sea no reservado).
pub fn codificar(v: &str) -> String {
    let mut s = String::with_capacity(v.len());
    for b in v.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
            s.push(char::from(b));
        } else {
            s.push_str(&format!("%{b:02X}"));
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rutas_rechazadas_antes_de_la_red() {
        for r in [
            "repos/o/r",
            "//evil.example/x",
            "/repos/o/../r",
            "/repos/o/./r",
            "/repos/o/%2e%2e/r",
            "/repos/o/%252e%252e/r",
            "/repos/o\\r",
            "/repos//r",
            "/repos/o/r?x=<",
            "/repos/o r",
            "/repos/o/r%2Fx",
        ] {
            assert!(validar_ruta(r).is_err(), "{r}");
        }
        for r in [
            "/projects/g%2Fp",
            "/projects/g%2Fs%2Fp/merge_requests?source_branch=a%2Fb",
        ] {
            assert!(validar_ruta_con(r, true).is_ok(), "{r}");
        }
        for r in [
            "/projects/g%2fp",
            "/projects/g%252Fp",
            "/projects/g%2F..",
            "/projects/g%2F.%2Fp",
            "/projects/%2Fp",
            "/projects/g%2F",
            "/projects/g%2F%2Fp",
            "/projects/g%3Fp",
        ] {
            assert!(validar_ruta_con(r, true).is_err(), "{r}");
        }
        for r in [
            "/user",
            "/repos/o/r/pulls?head=o%3Afeature%2Fx&state=open",
            "/repos/o/r/rules/branches/feature/task-1",
        ] {
            assert!(validar_ruta(r).is_ok(), "{r}");
        }
    }

    #[test]
    fn bases_validas_e_invalidas() {
        assert!(Base::nueva("https://api.github.com").is_ok());
        assert!(Base::nueva("https://git.empresa.com/api/v3/").is_ok());
        assert!(Base::nueva("http://127.0.0.1:8080").is_ok());
        assert!(Base::nueva("http://127.0.0.1:8080/api/v4").is_ok());
        for b in [
            "http://api.github.com",
            "http://127.0.0.1:80/../x",
            "http://127.0.0.1:80/x?y",
            "http://127.0.0.1:80/a%2Fb",
            "http://localhost:80",
            "https://user:pw@host",
            "https://",
            "ftp://x",
            "https://host/../x",
            "https://host/api?x=1",
            "https://host:0",
            "https://host:99999",
            "https://host:8a",
            "http://127.0.0.1:70000",
            // Host que empieza por `.` o `-`.
            "https://.evil.example",
            "https://-x.example",
        ] {
            assert!(Base::nueva(b).is_err(), "{b}");
        }
    }

    #[test]
    fn link_next() {
        let r = Respuesta {
            estado: 200,
            cuerpo: String::new(),
            link: Some(
                "<https://api.github.com/x?page=1>; rel=\"prev\", <https://api.github.com/x?page=3>; rel=\"next\""
                    .into(),
            ),
            restantes: None,
            reintentar: None,
            reinicio: None,
        };
        assert_eq!(r.siguiente(), Some("https://api.github.com/x?page=3"));
        let c = Cliente::nuevo(Base::github());
        assert_eq!(
            c.ruta_de("https://api.github.com/x?page=3").ok().as_deref(),
            Some("/x?page=3")
        );
        assert!(c.ruta_de("https://api.github.com.evil/x").is_err());
        assert!(c.ruta_de("https://evil.example/x").is_err());
    }

    #[test]
    fn codificar_valores() {
        assert_eq!(codificar("o:feature/x y"), "o%3Afeature%2Fx%20y");
    }
}
