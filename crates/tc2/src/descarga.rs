//! Descarga de la release para `tc2 actualizar --cli` (F2-12).
//!
//! Va aparte del puerto Plataforma: el [`Cliente`](crate::plataforma::http::Cliente)
//! de la API no sigue redirecciones, corta en 1 MiB y solo admite rutas
//! cerradas (contrato de F1-13), y bajar una release necesita las tres cosas
//! al reves. Lo comparten el agente (TLS del sistema, user-agent y tiempos,
//! [`crate::plataforma::http`]).
//!
//! - Origen: las releases de GitHub o, solo para ensayos, `TC2_BASE_URL`
//!   (https, o http a `127.0.0.1:<puerto>`), la carpeta con los ficheros de
//!   una release, como en los instaladores de F2-11.
//! - Redirecciones: se siguen a mano, hasta [`MAX_SALTOS`], y solo a `https`
//!   en los hosts de [`HOSTS_PERMITIDOS`] (github.com y sus hosts de assets).
//! - Limites: [`LIMITE_SUMAS`] para `SHA256SUMS` y [`LIMITE_BINARIO`] para el
//!   binario, que se escribe en un fichero abierto por quien llama.

use std::io::{Read, Write};
use std::time::Duration;

use crate::contrato::{Informe, Resultado};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::actualizar as txt;
use crate::plataforma::http::{self, Base, FalloHttp};

/// Releases del repositorio.
pub const RELEASES: &str = distribucion::RELEASES;

/// Variable de ensayo con la carpeta de una release (la de los instaladores).
pub const VARIABLE_BASE: &str = "TC2_BASE_URL";

/// Tamano maximo de `SHA256SUMS` (6 lineas de unos 100 bytes).
pub const LIMITE_SUMAS: u64 = 64 * 1024;

/// Tamano maximo del binario (el de hoy ronda 0,5 a 3 MiB).
pub const LIMITE_BINARIO: u64 = 64 * 1024 * 1024;

/// Redirecciones que se siguen como mucho.
pub const MAX_SALTOS: usize = 5;

/// Hosts a los que se sigue una redireccion: el de las releases y los de sus
/// assets.
pub const HOSTS_PERMITIDOS: &[&str] = &[
    "github.com",
    "objects.githubusercontent.com",
    "release-assets.githubusercontent.com",
    "github-releases.githubusercontent.com",
];

/// Tiempo maximo de una descarga completa (el binario, por una red lenta).
const ESPERA_DESCARGA: Duration = Duration::from_secs(120);

/// De donde se baja la release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origen {
    /// Las releases de GitHub.
    GitHub,
    /// Ensayo: la carpeta de una release servida en `TC2_BASE_URL`.
    Ensayo(String),
}

impl Origen {
    /// El origen segun el valor de `TC2_BASE_URL` (si esta definida).
    pub fn de_variable(valor: Option<&str>) -> Result<Self, Mensaje> {
        match valor.filter(|v| !v.is_empty()) {
            None => Ok(Self::GitHub),
            Some(v) => Base::nueva(v)
                .map(|b| Self::Ensayo(b.texto().to_owned()))
                .map_err(|_| Mensaje::nuevo(&txt::BASE_URL_INVALIDA).con("url", v)),
        }
    }

    /// URL de `SHA256SUMS`: la de la ultima release estable o la de
    /// `version`.
    pub fn url_sumas(&self, version: Option<&str>) -> String {
        match (self, version) {
            (Self::Ensayo(base), _) => format!("{base}/{}", distribucion::SUMAS),
            (Self::GitHub, None) => format!("{RELEASES}/latest/download/{}", distribucion::SUMAS),
            (Self::GitHub, Some(v)) => {
                format!("{RELEASES}/download/v{v}/{}", distribucion::SUMAS)
            }
        }
    }

    /// URL del binario `fichero` de la release del tag `v<tag>` (por su tag,
    /// no por `latest`: entre leer `SHA256SUMS` y bajar el binario puede
    /// salir otra).
    pub fn url_binario(&self, tag: &str, fichero: &str) -> String {
        match self {
            Self::Ensayo(base) => format!("{base}/{fichero}"),
            Self::GitHub => format!("{RELEASES}/download/v{tag}/{fichero}"),
        }
    }
}

/// Por que no se pudo bajar algo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fallo {
    /// Sin conexion, DNS, tiempo agotado o TLS.
    Sinred {
        /// URL pedida.
        url: String,
        /// Error del cliente.
        error: String,
        /// Si es un fallo de transporte transitorio, antes o despues de las
        /// cabeceras (no resuelve, no conecta, se corta, tiempo de conexion
        /// agotado): lo unico que se reintenta (ver [`es_transitorio`]). TLS,
        /// una respuesta invalida y el plazo global agotado no lo son: se
        /// repetirian igual, y repetir el plazo global doblaria la espera.
        transitorio: bool,
    },
    /// 404: no existe esa release o ese fichero.
    NoPublicada {
        /// URL pedida.
        url: String,
        /// Estado HTTP.
        estado: u16,
    },
    /// Otro estado que no es 2xx.
    Http {
        /// URL pedida.
        url: String,
        /// Estado HTTP.
        estado: u16,
    },
    /// Redireccion a un destino no permitido, o demasiadas.
    Redireccion {
        /// URL pedida.
        url: String,
        /// A donde redirigia.
        destino: String,
    },
    /// Pasa del limite.
    Grande {
        /// URL pedida.
        url: String,
        /// Limite en bytes.
        limite: u64,
    },
    /// La respuesta no es texto UTF-8 (un `SHA256SUMS` roto).
    NoEsTexto {
        /// URL pedida.
        url: String,
        /// Error de la decodificacion.
        error: String,
    },
    /// No se pudo escribir el fichero de destino.
    Escritura {
        /// Error del sistema.
        error: String,
        /// Su tipo: distingue un permiso denegado de un disco lleno.
        tipo: std::io::ErrorKind,
    },
}

impl Fallo {
    /// El informe de `comando` para este fallo.
    pub fn informe(&self, comando: &str) -> Informe {
        match self {
            Self::Sinred { url, error, .. } => Informe::nuevo(comando, Resultado::Sinred)
                .con("url", url.clone())
                .diagnostico(
                    Mensaje::nuevo(&txt::SIN_RED)
                        .con("url", url.clone())
                        .con("error", error.clone()),
                ),
            Self::NoPublicada { url, estado } => Informe::nuevo(comando, Resultado::Rechazada)
                .con("motivo", "no-publicada")
                .con("url", url.clone())
                .diagnostico(
                    Mensaje::nuevo(&txt::NO_PUBLICADA)
                        .con("url", url.clone())
                        .con("estado", estado.to_string()),
                ),
            Self::Http { url, estado } => Informe::nuevo(comando, Resultado::Sinred)
                .con("motivo", "http")
                .con("estado", estado.to_string())
                .con("url", url.clone())
                .diagnostico(
                    Mensaje::nuevo(&txt::HTTP_ESTADO)
                        .con("url", url.clone())
                        .con("estado", estado.to_string()),
                ),
            Self::Redireccion { url, destino } => Informe::nuevo(comando, Resultado::Rechazada)
                .con("motivo", "redireccion-no-permitida")
                .con("url", url.clone())
                .diagnostico(
                    Mensaje::nuevo(&txt::REDIRECCION_NO_PERMITIDA)
                        .con("url", url.clone())
                        .con("destino", destino.clone()),
                ),
            Self::Grande { url, limite } => Informe::nuevo(comando, Resultado::Rechazada)
                .con("motivo", "descarga-grande")
                .con("url", url.clone())
                .diagnostico(
                    Mensaje::nuevo(&txt::DESCARGA_GRANDE)
                        .con("url", url.clone())
                        .con("limite", limite.to_string()),
                ),
            Self::NoEsTexto { url, error } => Informe::nuevo(comando, Resultado::Rechazada)
                .con("motivo", "sumas-invalidas")
                .con("url", url.clone())
                .diagnostico(
                    Mensaje::nuevo(&txt::SUMAS_INVALIDAS)
                        .con("url", url.clone())
                        .con("error", error.clone()),
                ),
            Self::Escritura { error, .. } => Informe::nuevo(comando, Resultado::Error)
                .con("motivo", "escritura")
                .diagnostico(Mensaje::nuevo(&txt::ESCRITURA_DESCARGA).con("error", error.clone())),
        }
    }
}

/// Si se sigue una redireccion a `destino`: `https` a un host de
/// [`HOSTS_PERMITIDOS`], sin usuario ni puerto.
pub fn redireccion_permitida(destino: &str) -> bool {
    let Some(resto) = destino.strip_prefix("https://") else {
        return false;
    };
    let host = resto.split(['/', '?', '#']).next().unwrap_or_default();
    HOSTS_PERMITIDOS.contains(&host)
}

/// Si un error de ureq es un fallo de transporte que puede no repetirse y
/// merece un reintento:
///
/// - si: no resuelve, no conecta, se corta (E/S), el plazo de conexion u
///   otro plazo parcial, cuerpo parado;
/// - no: el plazo global (`ESPERA_DESCARGA`) agotado, porque repetirlo
///   doblaria la espera (TASK-027, MEN-9); TLS, incluido el handshake, que
///   rustls devuelve como E/S de tipo `InvalidData` (MEN-8); y el resto.
///
/// Un cierre de la otra parte a mitad del handshake llega como un fin de
/// datos (`UnexpectedEof`) que no se distingue de un corte, y se reintenta.
pub fn es_transitorio(e: &ureq::Error) -> bool {
    match e {
        ureq::Error::Timeout(ureq::Timeout::Global) => false,
        ureq::Error::Io(io) => es_transitorio_io(io),
        ureq::Error::HostNotFound
        | ureq::Error::ConnectionFailed
        | ureq::Error::Timeout(_)
        | ureq::Error::BodyStalled => true,
        _ => false,
    }
}

/// Como [`es_transitorio`] para un error de E/S: el de la lectura del cuerpo
/// (ureq envuelve en el sus propios errores) o el del transporte.
fn es_transitorio_io(e: &std::io::Error) -> bool {
    if let Some(u) = e.get_ref().and_then(|x| x.downcast_ref::<ureq::Error>()) {
        return es_transitorio(u);
    }
    // `InvalidData`: rustls rechazo lo recibido (certificado, protocolo).
    e.kind() != std::io::ErrorKind::InvalidData
}

/// Quien baja ficheros de la release.
pub struct Descargador {
    agente: ureq::Agent,
}

impl std::fmt::Debug for Descargador {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Descargador").finish_non_exhaustive()
    }
}

impl Default for Descargador {
    fn default() -> Self {
        Self::nuevo()
    }
}

impl Descargador {
    /// Con el agente de `tc2` y el tiempo de una descarga.
    pub fn nuevo() -> Self {
        Self::con_espera(ESPERA_DESCARGA)
    }

    /// Con el agente de `tc2` y `espera` como plazo global de cada descarga
    /// (los tests lo acortan).
    pub(crate) fn con_espera(espera: Duration) -> Self {
        Self {
            agente: http::agente(espera),
        }
    }

    /// Baja `url` como texto, hasta `limite` bytes.
    pub fn texto(&self, url: &str, limite: u64) -> Result<String, Fallo> {
        let mut v = Vec::new();
        self.a_escritor(url, &mut v, limite)?;
        String::from_utf8(v).map_err(|e| Fallo::NoEsTexto {
            url: url.to_owned(),
            error: e.to_string(),
        })
    }

    /// Baja `url` a `destino`, hasta `limite` bytes. Devuelve los bytes
    /// escritos.
    pub fn a_escritor(
        &self,
        url: &str,
        destino: &mut impl Write,
        limite: u64,
    ) -> Result<u64, Fallo> {
        let mut actual = url.to_owned();
        for _ in 0..=MAX_SALTOS {
            let sinred = |e: String, transitorio: bool| Fallo::Sinred {
                url: url.to_owned(),
                error: e,
                transitorio,
            };
            let mut r = match self.agente.get(&actual).call() {
                Ok(r) => r,
                Err(e) => {
                    let transitorio = es_transitorio(&e);
                    return Err(match http::clasificar(e) {
                        // DNS, conexion, tiempo agotado, E/S o cuerpo
                        // parado; transitorio salvo el plazo global y TLS
                        // dentro de la E/S.
                        FalloHttp::Sinred(e) => sinred(e, transitorio),
                        FalloHttp::Tls(e)
                        | FalloHttp::Otro(e)
                        | FalloHttp::RespuestaInvalida(e) => sinred(e, false),
                        FalloHttp::CuerpoGrande => Fallo::Grande {
                            url: url.to_owned(),
                            limite,
                        },
                        FalloHttp::Redireccion(_) | FalloHttp::Rechazada(_) => Fallo::Redireccion {
                            url: url.to_owned(),
                            destino: actual,
                        },
                    });
                }
            };
            let estado = r.status().as_u16();
            if (300..400).contains(&estado) {
                let destino = r
                    .headers()
                    .get("location")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or_default()
                    .to_owned();
                if !redireccion_permitida(&destino) {
                    return Err(Fallo::Redireccion {
                        url: url.to_owned(),
                        destino,
                    });
                }
                actual = destino;
                continue;
            }
            if estado == 404 {
                return Err(Fallo::NoPublicada {
                    url: url.to_owned(),
                    estado,
                });
            }
            if !(200..300).contains(&estado) {
                return Err(Fallo::Http {
                    url: url.to_owned(),
                    estado,
                });
            }
            let grande = || Fallo::Grande {
                url: url.to_owned(),
                limite,
            };
            let declarado = r
                .headers()
                .get("content-length")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.trim().parse::<u64>().ok());
            if declarado.is_some_and(|n| n > limite) {
                return Err(grande());
            }
            let mut lector = r.body_mut().as_reader().take(limite.saturating_add(1));
            let mut total: u64 = 0;
            let mut buf = vec![0u8; 64 * 1024];
            loop {
                let n = match lector.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(sinred(e.to_string(), es_transitorio_io(&e))),
                };
                total = total.saturating_add(n as u64);
                if total > limite {
                    return Err(grande());
                }
                destino
                    .write_all(buf.get(..n).unwrap_or_default())
                    .map_err(|e| Fallo::Escritura {
                        error: e.to_string(),
                        tipo: e.kind(),
                    })?;
            }
            return Ok(total);
        }
        Err(Fallo::Redireccion {
            url: url.to_owned(),
            destino: actual,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{TcpListener, TcpStream};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Instant;

    /// Servidor en 127.0.0.1 que atiende cada conexion con `atender` y las
    /// cuenta.
    fn servidor(atender: fn(TcpStream)) -> Option<(u16, Arc<AtomicUsize>)> {
        let l = TcpListener::bind("127.0.0.1:0").ok()?;
        let puerto = l.local_addr().ok()?.port();
        let conexiones = Arc::new(AtomicUsize::new(0));
        let c = conexiones.clone();
        std::thread::spawn(move || {
            for s in l.incoming().flatten() {
                c.fetch_add(1, Ordering::SeqCst);
                std::thread::spawn(move || atender(s));
            }
        });
        Some((puerto, conexiones))
    }

    /// Espera a que el cliente cierre (como mucho 10 s), sin mandar FIN.
    fn esperar_cierre(mut s: TcpStream) {
        let _ = s.set_read_timeout(Some(Duration::from_secs(10)));
        let _ = s.read_to_end(&mut Vec::new());
    }

    /// Fichero de destino de una prueba en la carpeta temporal.
    struct Temporal(std::path::PathBuf);

    impl Temporal {
        fn nuevo(nombre: &str) -> Option<(Self, std::fs::File)> {
            let ruta = std::env::temp_dir()
                .join(format!("tc2-descarga-{nombre}-{}.tmp", std::process::id()));
            let f = std::fs::File::create(&ruta).ok()?;
            Some((Self(ruta), f))
        }
    }

    impl Drop for Temporal {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    /// TASK-115 (MEN-8 de TASK-027): un fallo de TLS en el handshake no es
    /// transitorio y la descarga del binario no lo reintenta. El servidor
    /// responde en claro a una peticion `https`: rustls lo rechaza.
    #[test]
    fn tls_fallido_no_es_transitorio() {
        let srv = servidor(|mut s| {
            let _ = s.write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n");
            let _ = s.flush();
            esperar_cierre(s);
        });
        let tmp = Temporal::nuevo("tls");
        assert!(srv.is_some() && tmp.is_some());
        let (Some((puerto, conexiones)), Some((_t, mut f))) = (srv, tmp) else {
            return;
        };
        let d = Descargador::con_espera(Duration::from_secs(10));
        let url = format!("https://127.0.0.1:{puerto}/tc2");
        let r = crate::actualizar_cli::bajar_con_un_reintento(&d, &url, &mut f);
        assert!(
            matches!(
                r,
                Err(Fallo::Sinred {
                    transitorio: false,
                    ..
                })
            ),
            "{r:?}"
        );
        assert_eq!(conexiones.load(Ordering::SeqCst), 1, "{r:?}");
    }

    /// TASK-115 (MEN-9 de TASK-027): el plazo global agotado a mitad del
    /// cuerpo no es transitorio: la descarga no se repite y la espera no se
    /// dobla (margen de 1,5 veces el plazo).
    #[test]
    fn timeout_global_no_se_dobla() {
        let srv = servidor(|mut s| {
            let mut buf = [0u8; 4096];
            let _ = s.read(&mut buf);
            // Anuncia 100 bytes, manda 10 y se para.
            let _ = s.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n0123456789");
            let _ = s.flush();
            esperar_cierre(s);
        });
        let tmp = Temporal::nuevo("plazo");
        assert!(srv.is_some() && tmp.is_some());
        let (Some((puerto, conexiones)), Some((_t, mut f))) = (srv, tmp) else {
            return;
        };
        let espera = Duration::from_secs(2);
        let d = Descargador::con_espera(espera);
        let url = format!("http://127.0.0.1:{puerto}/tc2");
        let inicio = Instant::now();
        let r = crate::actualizar_cli::bajar_con_un_reintento(&d, &url, &mut f);
        let tardo = inicio.elapsed();
        assert!(
            matches!(
                r,
                Err(Fallo::Sinred {
                    transitorio: false,
                    ..
                })
            ),
            "{r:?}"
        );
        assert_eq!(conexiones.load(Ordering::SeqCst), 1, "{r:?}");
        assert!(tardo < espera.mul_f32(1.5), "tardo {tardo:?}");
    }

    #[test]
    fn solo_redirecciones_https_a_github() {
        for bien in [
            "https://github.com/charliebk/TaskCodeTwo/releases/download/v2.0.0/SHA256SUMS",
            "https://objects.githubusercontent.com/x?y=1",
            "https://release-assets.githubusercontent.com/x",
        ] {
            assert!(redireccion_permitida(bien), "{bien}");
        }
        for mal in [
            "http://github.com/x",
            "https://github.com.evil.example/x",
            "https://evil.example/github.com",
            "https://user@github.com/x",
            "https://github.com:8443/x",
            "/relativa",
            "",
        ] {
            assert!(!redireccion_permitida(mal), "{mal}");
        }
    }

    #[test]
    fn urls_de_la_release() {
        assert_eq!(
            Origen::GitHub.url_sumas(None),
            "https://github.com/charliebk/TaskCodeTwo/releases/latest/download/SHA256SUMS"
        );
        assert_eq!(
            Origen::GitHub.url_sumas(Some("2.0.0")),
            "https://github.com/charliebk/TaskCodeTwo/releases/download/v2.0.0/SHA256SUMS"
        );
        assert_eq!(
            Origen::GitHub.url_binario("2.0.0", "tc2-2.0.0-linux-x64"),
            "https://github.com/charliebk/TaskCodeTwo/releases/download/v2.0.0/tc2-2.0.0-linux-x64"
        );
        let e = Origen::de_variable(Some("http://127.0.0.1:8765/")).unwrap_or(Origen::GitHub);
        assert_eq!(e, Origen::Ensayo("http://127.0.0.1:8765".into()));
        assert_eq!(
            e.url_sumas(Some("9.9.9")),
            "http://127.0.0.1:8765/SHA256SUMS"
        );
        assert_eq!(Origen::de_variable(None), Ok(Origen::GitHub));
        assert_eq!(Origen::de_variable(Some("")), Ok(Origen::GitHub));
        for mala in ["http://evil.example", "ftp://x", "http://localhost:80"] {
            assert!(Origen::de_variable(Some(mala)).is_err(), "{mala}");
        }
    }
}
