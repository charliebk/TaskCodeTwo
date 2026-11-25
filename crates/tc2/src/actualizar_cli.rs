//! `tc2 actualizar --cli [--comprobar] [--version X]` (F2-12, ADR F1-14
//! puntos 4 y 5): sustituye el `tc2` instalado por la version de la release
//! sin reinstalar.
//!
//! - La version disponible sale de `SHA256SUMS` de la release (la ultima
//!   estable, o la de `--version`): una peticion, la misma fuente que los
//!   instaladores de F2-11.
//! - `--comprobar` solo lee: `actualizar ok actual=X disponible=Y` si hay una
//!   mas nueva, `sin-cambios` si no (con `--version` de una anterior, `ok`
//!   con `cambio=anterior`). Es el aviso de version nueva: es una orden
//!   explicita, nunca una consulta pasiva; la llamara `doctor` cuando exista
//!   (F3-8) y, si se quiere, un hook de inicio de sesion.
//! - Si hay `gh` con sesion, comprueba la attestation de procedencia del
//!   binario nuevo como los instaladores de F2-11; sin `gh`, sigue con el
//!   sha256 y lo dice en la linea (`attestation=sin-gh`).
//! - Un `tc2` de la cache del plugin no se toca (lo gestiona el plugin con su
//!   `cli.lock`), ni uno de un gestor de paquetes.
//! - El binario nuevo se baja junto al actual (`.tc2-<v>-<pid>.descarga`),
//!   su sha256 lo calcula este proceso, que no es el comprobado (punto 4), y
//!   solo si casa con `SHA256SUMS` se pregunta al nuevo `version --json` y se
//!   sustituye. En Windows, por renombrados (actual → `<exe>.old`, nuevo → su
//!   sitio) con vuelta atras; el `.old` lo borra la siguiente ejecucion.

use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use distribucion::Target;

use crate::contrato::{Informe, Resultado};
use crate::descarga::{self, Descargador, Fallo, Origen};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::actualizar as txt;

/// Nombre de la orden.
pub const NOMBRE: &str = "actualizar";

pub use crate::instalacion::{Modo, detectar_modo, limpiar_old, ruta_old};

/// Compara dos versiones `X.Y.Z[-pre]` con las reglas de semver: el nucleo
/// por numeros y, a igual nucleo, una prerelease es anterior a la estable;
/// entre prereleases, identificador a identificador (los numericos por valor
/// y antes que los alfanumericos). `None` si alguna no es una version.
pub fn comparar(a: &str, b: &str) -> Option<Ordering> {
    let (na, pa) = partes_version(a)?;
    let (nb, pb) = partes_version(b)?;
    Some(na.cmp(&nb).then_with(|| match (pa, pb) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(x), Some(y)) => comparar_pre(x, y),
    }))
}

fn partes_version(v: &str) -> Option<([u64; 3], Option<&str>)> {
    if !distribucion::version_valida(v) {
        return None;
    }
    let (nucleo, pre) = match v.split_once('-') {
        Some((n, p)) => (n, Some(p)),
        None => (v, None),
    };
    let mut n = [0u64; 3];
    for (i, p) in nucleo.split('.').enumerate() {
        *n.get_mut(i)? = p.parse().ok()?;
    }
    Some((n, pre))
}

fn comparar_pre(a: &str, b: &str) -> Ordering {
    let mut ia = a.split('.');
    let mut ib = b.split('.');
    loop {
        match (ia.next(), ib.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) => {
                let o = match (x.parse::<u64>(), y.parse::<u64>()) {
                    (Ok(m), Ok(n)) => m.cmp(&n),
                    (Ok(_), Err(_)) => Ordering::Less,
                    (Err(_), Ok(_)) => Ordering::Greater,
                    (Err(_), Err(_)) => x.cmp(y),
                };
                if o != Ordering::Equal {
                    return o;
                }
            }
        }
    }
}

/// Opciones de la orden.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Opciones {
    /// Solo comprobar, sin tocar nada.
    pub comprobar: bool,
    /// Version concreta en lugar de la ultima estable.
    pub version: Option<String>,
}

/// Lee `--cli [--comprobar] [--version X]` (sin `--json`). Sin `--cli`, uso:
/// actualizar un proyecto es de F9-2.
pub fn leer_opciones(args: &[String]) -> Result<Opciones, Informe> {
    let uso = |motivo: &str| {
        Informe::nuevo(NOMBRE, Resultado::Uso)
            .con("motivo", motivo)
            .diagnostico(&txt::USO_ACTUALIZAR)
    };
    let mut cli = false;
    let mut op = Opciones::default();
    let mut i = args.iter();
    while let Some(a) = i.next() {
        match a.as_str() {
            "--cli" if !cli => cli = true,
            "--comprobar" if !op.comprobar => op.comprobar = true,
            "--version" if op.version.is_none() => {
                let Some(v) = i.next() else {
                    return Err(uso("sin-version"));
                };
                let v = v.strip_prefix('v').unwrap_or(v);
                if !distribucion::version_valida(v) {
                    return Err(Informe::nuevo(NOMBRE, Resultado::Uso)
                        .con("motivo", "version-invalida")
                        .diagnostico(Mensaje::nuevo(&txt::VERSION_INVALIDA).con("version", v)));
                }
                op.version = Some(v.to_owned());
            }
            _ => return Err(uso("argumentos")),
        }
    }
    if cli { Ok(op) } else { Err(uso("sin-cli")) }
}

/// La orden completa para el ejecutable actual, con `TC2_BASE_URL` del
/// entorno.
pub fn orden(args: &[String]) -> Informe {
    let op = match leer_opciones(args) {
        Ok(o) => o,
        Err(i) => return i,
    };
    let origen = match Origen::de_variable(std::env::var(descarga::VARIABLE_BASE).ok().as_deref()) {
        Ok(o) => o,
        Err(m) => {
            return Informe::nuevo(NOMBRE, Resultado::Uso)
                .con("motivo", "base-url")
                .diagnostico(m);
        }
    };
    let exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(e) => {
            return Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(
                Mensaje::nuevo(&txt::NO_SE_LEE_EL_EJECUTABLE)
                    .con("ruta", "current_exe")
                    .con("error", e.to_string()),
            );
        }
    };
    let gh = gh_de_variable(std::env::var(VARIABLE_GH).ok().as_deref(), &origen);
    actualizar(&exe, &op, &origen, &Descargador::nuevo(), gh.as_deref())
}

/// Variable con el `gh` que se usa para la attestation (por defecto, el del
/// PATH).
pub const VARIABLE_GH: &str = "TC2_GH";

/// El `gh` para la attestation: el de `TC2_GH` si esta definida y, si no,
/// `gh` del PATH. En un ensayo (`TC2_BASE_URL`) los binarios no tienen
/// attestation, asi que solo se comprueba si `TC2_GH` lo pide, como en los
/// instaladores, que no la comprueban con `TC2_BASE_URL`.
pub fn gh_de_variable(valor: Option<&str>, origen: &Origen) -> Option<String> {
    match (valor.filter(|v| !v.is_empty()), origen) {
        (Some(v), _) => Some(v.to_owned()),
        (None, Origen::GitHub) => Some("gh".to_owned()),
        (None, Origen::Ensayo(_)) => None,
    }
}

/// Lo que publica la release para un target.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Publicado {
    /// Version del binario (la de su nombre).
    version: String,
    /// Version del tag de la release (`v<tag>`): la del binario, o con un
    /// sufijo en una prerelease (`v0.0.0-prueba.1` publica `tc2-0.0.0-*`,
    /// regla de `distribucion::clase_de_tag`).
    tag: String,
    fichero: String,
    sha256: String,
}

/// El binario de `target` en `SHA256SUMS`. Con `tag` (la de `--version`),
/// el de esa release: su version es la del tag o la del tag sin el sufijo de
/// prerelease. Sin `tag`, el de version mas alta, y el tag es su version (la
/// ultima estable).
fn elegir(pares: &[(String, String)], target: Target, tag: Option<&str>) -> Option<Publicado> {
    let mut candidatos = pares.iter().filter_map(|(f, h)| {
        target.version_de_fichero(f).map(|v| Publicado {
            tag: tag.map_or_else(|| v.clone(), str::to_owned),
            version: v,
            fichero: f.clone(),
            sha256: h.clone(),
        })
    });
    match tag {
        Some(t) => candidatos.find(|p| {
            t == p.version
                || t.strip_prefix(p.version.as_str())
                    .is_some_and(|r| r.starts_with('-'))
        }),
        None => {
            candidatos.max_by(|a, b| comparar(&a.version, &b.version).unwrap_or(Ordering::Equal))
        }
    }
}

/// `actualizar --cli` sobre el ejecutable `exe`, con `gh` para la
/// attestation (`None`: no se comprueba, en un ensayo).
pub fn actualizar(
    exe: &Path,
    op: &Opciones,
    origen: &Origen,
    d: &Descargador,
    gh: Option<&str>,
) -> Informe {
    match detectar_modo(&exe.to_string_lossy()) {
        Modo::Instalador => {}
        Modo::Plugin => {
            return Informe::nuevo(NOMBRE, Resultado::Rechazada)
                .con("motivo", "gestionado-por-plugin")
                .con("arreglo", "actualizar-el-plugin")
                .diagnostico(
                    Mensaje::nuevo(&txt::GESTIONADO_POR_PLUGIN)
                        .con("ruta", exe.display().to_string()),
                );
        }
        Modo::Gestor(g) => {
            return Informe::nuevo(NOMBRE, Resultado::Rechazada)
                .con("motivo", "gestionado-por-gestor")
                .con("gestor", g)
                .diagnostico(
                    Mensaje::nuevo(&txt::GESTIONADO_POR_GESTOR)
                        .con("gestor", g)
                        .con("ruta", exe.display().to_string()),
                );
        }
    }
    let Some(target) = Target::actual() else {
        return Informe::nuevo(NOMBRE, Resultado::Rechazada)
            .con("motivo", "target-fuera-de-p12")
            .diagnostico(
                Mensaje::nuevo(&txt::TARGET_FUERA_DE_P12)
                    .con("sistema", crate::version::target_actual()),
            );
    };
    let url = origen.url_sumas(op.version.as_deref());
    let texto = match d.texto(&url, descarga::LIMITE_SUMAS) {
        Ok(t) => t,
        Err(f) => return f.informe(NOMBRE),
    };
    let pares = match distribucion::leer_sumas(&texto) {
        Ok(p) => p,
        Err(e) => {
            return Informe::nuevo(NOMBRE, Resultado::Rechazada)
                .con("motivo", "sumas-invalidas")
                .con("url", url.clone())
                .diagnostico(
                    Mensaje::nuevo(&txt::SUMAS_INVALIDAS)
                        .con("url", url)
                        .con("error", e),
                );
        }
    };
    let Some(publicado) = elegir(&pares, target, op.version.as_deref()) else {
        let version = op
            .version
            .as_deref()
            .map(|v| format!(" {v}"))
            .unwrap_or_default();
        return Informe::nuevo(NOMBRE, Resultado::Rechazada)
            .con("motivo", "sin-binario")
            .con("target", target.alias)
            .con("url", url.clone())
            .diagnostico(
                Mensaje::nuevo(&txt::SIN_BINARIO)
                    .con("url", url)
                    .con("target", target.alias)
                    .con("version", version),
            );
    };
    let actual = crate::VERSION;
    let orden = comparar(actual, &publicado.version);
    // Sin --version solo se sube; con --version se instala esa, salvo si ya
    // es la que hay.
    let nada_que_hacer = match op.version {
        Some(_) => orden == Some(Ordering::Equal),
        None => orden != Some(Ordering::Less),
    };
    let base = |r| {
        Informe::nuevo(NOMBRE, r)
            .con("actual", actual)
            .con("disponible", publicado.version.clone())
    };
    if nada_que_hacer {
        return base(Resultado::SinCambios);
    }
    if op.comprobar {
        // Con --version de una anterior, `ok` no quiere decir que haya una
        // mas nueva: se dice.
        return if orden == Some(Ordering::Greater) {
            base(Resultado::Ok).con("cambio", "anterior")
        } else {
            base(Resultado::Ok)
        };
    }
    instalar(exe, target, &publicado, origen, d, gh)
}

/// Baja el binario junto a `exe`, comprueba su sha256 contra `SHA256SUMS`
/// (lo calcula este proceso, no el binario nuevo), le pide `version --json`
/// y lo pone en el sitio de `exe`. Si algo falla antes del ultimo paso, no
/// queda nada nuevo en la carpeta.
fn instalar(
    exe: &Path,
    target: Target,
    publicado: &Publicado,
    origen: &Origen,
    d: &Descargador,
    gh: Option<&str>,
) -> Informe {
    // Un `.old` de una actualizacion anterior (Windows). Aqui y no antes:
    // `--comprobar` no toca nada.
    limpiar_old(exe);
    let Some(dir) = exe.parent() else {
        return sin_permiso(
            exe,
            &std::io::Error::from(std::io::ErrorKind::NotFound).to_string(),
        );
    };
    let temporal = dir.join(format!(
        ".tc2-{}-{}.descarga",
        publicado.version,
        std::process::id()
    ));
    // Abrirlo es tambien la comprobacion de permisos: antes de bajar nada.
    let mut fichero = match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporal)
    {
        Ok(f) => f,
        Err(e) => return sin_permiso(dir, &e.to_string()),
    };
    let guarda = Borrar(Some(temporal.clone()));
    let url = origen.url_binario(&publicado.tag, &publicado.fichero);
    let bajada = bajar_con_un_reintento(d, &url, &mut fichero)
        .map_err(|f| match f {
            Fallo::Escritura { error, tipo } => fallo_de_escritura(&temporal, tipo, &error),
            otro => otro.informe(NOMBRE),
        })
        .and_then(|_| {
            fichero
                .sync_all()
                .map_err(|e| fallo_de_escritura(&temporal, e.kind(), &e.to_string()))
        });
    drop(fichero);
    if let Err(i) = bajada {
        return i;
    }
    let obtenido = match std::fs::read(&temporal) {
        Ok(b) => distribucion::sha256::hex(&b),
        Err(e) => return fallo_de_escritura(&temporal, e.kind(), &e.to_string()),
    };
    if obtenido != publicado.sha256 {
        return Informe::nuevo(NOMBRE, Resultado::Rechazada)
            .con("motivo", "sha256-no-casa")
            .con("fichero", publicado.fichero.clone())
            .con("esperado", publicado.sha256.clone())
            .con("obtenido", obtenido.clone())
            .diagnostico(
                Mensaje::nuevo(&txt::SHA256_NO_CASA)
                    .con("fichero", publicado.fichero.clone())
                    .con("obtenido", obtenido)
                    .con("esperado", publicado.sha256.clone()),
            );
    }
    // La procedencia, antes de ejecutar nada del binario nuevo.
    let attestation = match comprobar_attestation(gh, &temporal) {
        Ok(a) => a,
        Err(detalle) => {
            return Informe::nuevo(NOMBRE, Resultado::Rechazada)
                .con("motivo", "attestation-no-valida")
                .con("fichero", publicado.fichero.clone())
                .diagnostico(
                    Mensaje::nuevo(&txt::ATTESTATION_NO_VALIDA)
                        .con("fichero", publicado.fichero.clone())
                        .con("detalle", detalle),
                );
        }
    };
    if let Err(e) = hacer_ejecutable(&temporal) {
        return fallo_de_escritura(&temporal, e.kind(), &e.to_string());
    }
    if let Err(detalle) = comprobar_nuevo(&temporal, &publicado.version, target.alias, &obtenido) {
        return Informe::nuevo(NOMBRE, Resultado::Rechazada)
            .con("motivo", "binario-no-arranca")
            .diagnostico(
                Mensaje::nuevo(&txt::NO_ARRANCA)
                    .con("version", publicado.version.clone())
                    .con("target", target.alias)
                    .con("detalle", detalle),
            );
    }
    if let Err(e) = sustituir_con(exe, &temporal, cfg!(windows), |a, b| std::fs::rename(a, b)) {
        return Informe::nuevo(NOMBRE, Resultado::Error)
            .con("motivo", "no-se-sustituye")
            .con("ruta", exe.display().to_string())
            .diagnostico(
                Mensaje::nuevo(&txt::NO_SE_SUSTITUYE)
                    .con("ruta", exe.display().to_string())
                    .con("error", e.to_string()),
            );
    }
    guarda.desarmar();
    // En Unix no hay `.old`; en Windows sigue en uso hasta que este proceso
    // acabe y lo borra la siguiente ejecucion.
    limpiar_old(exe);
    Informe::nuevo(NOMBRE, Resultado::Ok)
        .con("anterior", crate::VERSION)
        .con("version", publicado.version.clone())
        .con("sha256", obtenido)
        .con("attestation", attestation.texto())
}

/// Como acabo la comprobacion de la attestation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attestation {
    /// `gh attestation verify` la dio por buena.
    Comprobada,
    /// No hay `gh`, o no tiene sesion: solo el sha256 (como el instalador).
    SinGh,
    /// Ensayo con `TC2_BASE_URL`: sus binarios no tienen attestation.
    Ensayo,
}

impl Attestation {
    /// Valor de la clave `attestation=`.
    pub fn texto(self) -> &'static str {
        match self {
            Self::Comprobada => "comprobada",
            Self::SinGh => "sin-gh",
            Self::Ensayo => "ensayo",
        }
    }
}

/// Tiempo maximo de cada llamada a `gh`.
const ESPERA_GH: Duration = Duration::from_secs(60);

/// Comprueba la attestation de `fichero` con `gh` (paridad con los
/// instaladores de F2-11): `gh auth status` y, si hay sesion,
/// `gh attestation verify <fichero> --repo <REPO> --signer-workflow
/// <WORKFLOW_RELEASE>`. Sin `gh` o sin sesion, `SinGh`. `Err` con el detalle
/// si `gh` la rechaza o no responde.
pub fn comprobar_attestation(gh: Option<&str>, fichero: &Path) -> Result<Attestation, String> {
    let Some(gh) = gh else {
        return Ok(Attestation::Ensayo);
    };
    let mut sesion = Command::new(gh);
    sesion.args(["auth", "status"]);
    match ejecutar_con_limite(sesion, ESPERA_GH) {
        Ok(Some(s)) if s.status.success() => {}
        // Sin gh, gh sin sesion, o un gh que no responde a `auth status`.
        _ => return Ok(Attestation::SinGh),
    }
    let mut verificar = Command::new(gh);
    verificar
        .args(["attestation", "verify"])
        .arg(fichero)
        .args(["--repo", distribucion::REPO])
        .args(["--signer-workflow", distribucion::WORKFLOW_RELEASE]);
    match ejecutar_con_limite(verificar, ESPERA_GH) {
        Ok(Some(s)) if s.status.success() => Ok(Attestation::Comprobada),
        Ok(Some(s)) => Err(ultima_linea(&s.stderr)
            .unwrap_or_else(|| format!("gh attestation verify salio con {}", s.status))),
        Ok(None) => Err(format!(
            "gh attestation verify no respondio en {} s",
            ESPERA_GH.as_secs()
        )),
        Err(e) => Err(e.to_string()),
    }
}

/// La ultima linea no vacia de una salida, para el diagnostico.
fn ultima_linea(bytes: &[u8]) -> Option<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .rev()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map(|l| l.chars().take(300).collect())
}

/// Lanza `cmd` sin entrada, en un grupo propio, y espera como mucho `limite`
/// ([`crate::proceso::esperar_con_plazo`]). `Ok(None)` si no termino a
/// tiempo: entonces se mata el arbol entero (el hijo y lo que haya lanzado,
/// con `taskkill /T` o `kill` al grupo), y con el se cierran las tuberias.
/// stdout y stderr se leen en hilos aparte, hasta 1 MiB cada uno, para que
/// un hijo que escribe mucho no se bloquee.
///
/// El plazo vale para todo, tambien para leer la salida: si el hijo acaba a
/// tiempo pero deja un nieto en segundo plano con las tuberias heredadas (un
/// `.bat` que lanza otro proceso), las tuberias no se cierran y los hilos
/// lectores siguen bloqueados. Entonces se espera al cierre hasta que se
/// cumple el plazo (no mas) y se devuelve lo leido hasta ese momento: la
/// salida del hijo, que acabo, no se pierde, pero la llamada tarda el plazo
/// entero. A ese nieto no se le mata (el hijo acabo bien); los hilos acaban
/// solos cuando cierre las tuberias (o con el proceso).
pub(crate) fn ejecutar_con_limite(
    mut cmd: Command,
    limite: Duration,
) -> std::io::Result<Option<Output>> {
    use std::io::Read;
    use std::sync::{Arc, Mutex, PoisonError, mpsc};
    const MAXIMO: usize = 1024 * 1024;
    let mut hijo = crate::proceso::grupo_propio(&mut cmd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    // Cada hilo va dejando lo leido en un buffer compartido y avisa al ver
    // el final de la tuberia; quien espera se lleva lo que haya, acabe o no.
    let leer = |p: Option<Box<dyn Read + Send>>| {
        let leido: Arc<Mutex<Vec<u8>>> = Arc::default();
        let (tx, rx) = mpsc::channel::<()>();
        let destino = leido.clone();
        std::thread::spawn(move || {
            if let Some(mut p) = p {
                let mut trozo = [0u8; 8192];
                loop {
                    let n = match p.read(&mut trozo) {
                        Ok(0) => break,
                        Ok(n) => n,
                        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                        Err(_) => break,
                    };
                    let mut v = destino.lock().unwrap_or_else(PoisonError::into_inner);
                    let cabe = MAXIMO.saturating_sub(v.len()).min(n);
                    v.extend_from_slice(trozo.get(..cabe).unwrap_or_default());
                    if v.len() >= MAXIMO {
                        break;
                    }
                }
            }
            // Si nadie espera ya (plazo cumplido), se descarta.
            let _ = tx.send(());
        });
        (leido, rx)
    };
    let recoger = |(leido, rx): (Arc<Mutex<Vec<u8>>>, mpsc::Receiver<()>), espera: Duration| {
        let _ = rx.recv_timeout(espera);
        std::mem::take(&mut *leido.lock().unwrap_or_else(PoisonError::into_inner))
    };
    let out = leer(
        hijo.stdout
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
    );
    let err = leer(
        hijo.stderr
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
    );
    let fin = Instant::now() + limite;
    let status = crate::proceso::esperar_con_plazo(&mut hijo, limite)?;
    // Lo que quede del plazo, con un margen corto para recoger la salida de
    // un hijo que acabo justo al limite.
    let restante = || {
        fin.saturating_duration_since(Instant::now())
            .max(Duration::from_millis(100))
    };
    let stdout = recoger(out, restante());
    let stderr = recoger(err, restante());
    Ok(status.map(|status| Output {
        status,
        stdout,
        stderr,
    }))
}

/// Un fallo de E/S sobre `ruta`: `sin-permiso` si es un permiso denegado y,
/// si no (disco lleno, por ejemplo), `error motivo=escritura`.
fn fallo_de_escritura(ruta: &Path, tipo: std::io::ErrorKind, error: &str) -> Informe {
    if tipo == std::io::ErrorKind::PermissionDenied {
        return sin_permiso(ruta, error);
    }
    Informe::nuevo(NOMBRE, Resultado::Error)
        .con("motivo", "escritura")
        .con("ruta", ruta.display().to_string())
        .diagnostico(
            Mensaje::nuevo(&txt::ESCRITURA)
                .con("ruta", ruta.display().to_string())
                .con("error", error),
        )
}

/// Pausa antes del reintento de la descarga.
const PAUSA_REINTENTO: Duration = Duration::from_secs(1);

/// Baja el binario a `fichero` y, ante un fallo de transporte transitorio
/// (no conecta, la conexion se corta antes o despues de las cabeceras, plazo
/// de conexion agotado; ver [`descarga::es_transitorio`]), lo vacia y lo
/// intenta una vez mas. Son unos MiB por una red que puede fallar a medias;
/// y en una maquina de desarrollo con Windows se ha visto (TASK-027) que la
/// red local corta a veces a los ~19 s las descargas de mas de ~450 KB. No
/// se reintentan los fallos que se repetirian igual: 4xx, 5xx, TLS, una
/// redireccion no permitida, el limite de tamano y una respuesta invalida;
/// ni el plazo global agotado, que doblaria la espera.
pub(crate) fn bajar_con_un_reintento(
    d: &Descargador,
    url: &str,
    fichero: &mut std::fs::File,
) -> Result<u64, Fallo> {
    use std::io::{Seek, SeekFrom};
    match d.a_escritor(url, fichero, descarga::LIMITE_BINARIO) {
        Err(Fallo::Sinred {
            transitorio: true, ..
        }) => {
            let vaciar = fichero
                .set_len(0)
                .and_then(|()| fichero.seek(SeekFrom::Start(0)).map(|_| ()));
            if let Err(e) = vaciar {
                return Err(Fallo::Escritura {
                    error: e.to_string(),
                    tipo: e.kind(),
                });
            }
            // Una pausa corta antes de repetir, como cualquier reintento:
            // un corte suele venir de algo que tarda un momento en pasar.
            std::thread::sleep(PAUSA_REINTENTO);
            d.a_escritor(url, fichero, descarga::LIMITE_BINARIO)
        }
        otro => otro,
    }
}

/// `rechazada motivo=sin-permiso` con la ruta.
fn sin_permiso(ruta: &Path, error: &str) -> Informe {
    Informe::nuevo(NOMBRE, Resultado::Rechazada)
        .con("motivo", "sin-permiso")
        .con("ruta", ruta.display().to_string())
        .diagnostico(
            Mensaje::nuevo(&txt::SIN_PERMISO)
                .con("ruta", ruta.display().to_string())
                .con("error", error),
        )
}

/// Borra el fichero al salir de su ambito, salvo que se desarme.
struct Borrar(Option<PathBuf>);

impl Borrar {
    fn desarmar(mut self) {
        self.0 = None;
    }
}

impl Drop for Borrar {
    fn drop(&mut self) {
        if let Some(r) = self.0.take() {
            let _ = std::fs::remove_file(r);
        }
    }
}

#[cfg(unix)]
fn hacer_ejecutable(ruta: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(ruta, std::fs::Permissions::from_mode(0o755))
}

#[cfg(not(unix))]
fn hacer_ejecutable(_ruta: &Path) -> std::io::Result<()> {
    Ok(())
}

/// Tiempo maximo del `version --json` del binario nuevo.
const ESPERA_NUEVO: Duration = Duration::from_secs(30);

/// Lanza `nuevo version --json` (como mucho [`ESPERA_NUEVO`]) y exige su
/// version, su target y el sha256 que ya calculo este proceso. `Err` con el
/// detalle si no.
fn comprobar_nuevo(nuevo: &Path, version: &str, alias: &str, sha256: &str) -> Result<(), String> {
    let mut cmd = Command::new(nuevo);
    cmd.args(["version", "--json"]);
    let salida = ejecutar_con_limite(cmd, ESPERA_NUEVO)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| {
            format!(
                "version --json no respondio en {} s",
                ESPERA_NUEVO.as_secs()
            )
        })?;
    if !salida.status.success() {
        return Err(format!("version --json salio con {}", salida.status));
    }
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout)
        .map_err(|e| format!("version --json no es JSON: {e}"))?;
    let campo = |k: &str| {
        v.get(k)
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };
    if campo("version") != version {
        return Err(format!("dice ser la version {}", campo("version")));
    }
    if campo("target") != alias {
        return Err(format!("dice ser del target {}", campo("target")));
    }
    if campo("sha256") != sha256 {
        return Err(format!("da otro sha256 ({})", campo("sha256")));
    }
    if !v.get("protocolo").is_some_and(serde_json::Value::is_u64) {
        return Err("version --json sin protocolo".to_owned());
    }
    Ok(())
}

/// Pone `nuevo` en el sitio de `viejo` con `renombrar`.
///
/// - `por_old` (Windows, donde un ejecutable en marcha se puede renombrar
///   pero no sobrescribir): `viejo` → `<viejo>.old` y `nuevo` → `viejo`; si
///   el segundo falla, `<viejo>.old` vuelve a `viejo` (vuelta atras) y el
///   error lo dice.
/// - si no (Unix): un solo renombrado atomico de `nuevo` sobre `viejo`.
pub fn sustituir_con(
    viejo: &Path,
    nuevo: &Path,
    por_old: bool,
    mut renombrar: impl FnMut(&Path, &Path) -> std::io::Result<()>,
) -> std::io::Result<()> {
    if !por_old {
        return renombrar(nuevo, viejo);
    }
    let old = ruta_old(viejo);
    // Uno de una actualizacion anterior que no se pudo borrar.
    let _ = std::fs::remove_file(&old);
    renombrar(viejo, &old)?;
    if let Err(e) = renombrar(nuevo, viejo) {
        return match renombrar(&old, viejo) {
            Ok(()) => Err(e),
            Err(e2) => Err(std::io::Error::other(format!(
                "{e}; y al volver atras: {e2}; el binario anterior esta en {}",
                old.display()
            ))),
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Una orden de shell segun el sistema (`cmd /c` en Windows, `sh -c` en
    /// los demas), con `TC2_PRUEBA=1`.
    fn shell(windows: &str, unix: &str) -> Command {
        let mut c = if cfg!(windows) {
            let mut c = Command::new("cmd");
            c.args(["/c", windows]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(["-c", unix]);
            c
        };
        c.env("TC2_PRUEBA", "1");
        c
    }

    /// Revision 2, MEN-6: el plazo se cumple aunque el hijo deje un nieto
    /// con las tuberias heredadas (aqui, un shell que lanza una espera larga
    /// y no la sustituye con `exec`). Antes, la llamada esperaba al nieto.
    #[test]
    fn ejecutar_con_limite_cumple_el_plazo_con_un_nieto() {
        let cmd = shell("ping -n 30 127.0.0.1 >nul & echo fin", "sleep 30; echo fin");
        let inicio = Instant::now();
        let r = ejecutar_con_limite(cmd, Duration::from_secs(1));
        let tardo = inicio.elapsed();
        assert!(matches!(r, Ok(None)), "{r:?}");
        assert!(tardo < Duration::from_secs(10), "tardo {tardo:?}");
        // Un hijo que acaba a tiempo da su salida y su codigo.
        let eco = shell("echo hola", "echo hola");
        let s = ejecutar_con_limite(eco, Duration::from_secs(20));
        let s = s.ok().flatten();
        assert_eq!(s.as_ref().map(|o| o.status.success()), Some(true));
        assert_eq!(
            s.map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned()),
            Some("hola".to_owned())
        );
    }

    /// TASK-115 (MEN-7 de TASK-027): si el hijo acaba bien pero deja un
    /// nieto con stdout heredado, la llamada vuelve al cumplirse el plazo con
    /// la salida que el hijo ya escribio. Antes se perdia (stdout vacio). El
    /// nieto dura poco (unos 5 s) para no dejarlo huerfano mucho tiempo.
    #[test]
    fn ejecutar_con_limite_devuelve_lo_leido_si_un_nieto_retiene_stdout() {
        let cmd = shell(
            "start /b ping -n 6 127.0.0.1 & echo hola",
            "sleep 5 & echo hola",
        );
        let inicio = Instant::now();
        let r = ejecutar_con_limite(cmd, Duration::from_secs(2));
        let tardo = inicio.elapsed();
        let s = r.ok().flatten();
        assert_eq!(s.as_ref().map(|o| o.status.success()), Some(true));
        let stdout = s
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
            .unwrap_or_default();
        assert!(
            stdout.lines().any(|l| l.trim() == "hola"),
            "stdout: {stdout:?}"
        );
        assert!(tardo < Duration::from_secs(4), "tardo {tardo:?}");
    }

    /// TASK-115: el doc de `bajar_con_un_reintento` esta pegado a su funcion
    /// (no detras de una `const`, que se lo quedaria) y sin lineas repetidas.
    #[test]
    fn el_doc_de_bajar_con_un_reintento_esta_sobre_su_funcion() {
        let fuente = include_str!("actualizar_cli.rs");
        let firma = concat!("fn ", "bajar_con_un_reintento(");
        let lineas: Vec<&str> = fuente.lines().collect();
        let pos = lineas.iter().position(|l| {
            let l = l.trim_start();
            l.strip_prefix("pub(crate) ")
                .unwrap_or(l)
                .starts_with(firma)
        });
        assert!(pos.is_some(), "no esta la funcion");
        let pos = pos.unwrap_or_default();
        let doc: Vec<&str> = lineas
            .get(..pos)
            .unwrap_or_default()
            .iter()
            .rev()
            .take_while(|l| l.trim_start().starts_with("///"))
            .map(|l| l.trim())
            .collect();
        assert!(
            doc.iter()
                .any(|l| !l.trim_start_matches('/').trim().is_empty()),
            "sin doc justo encima de la funcion"
        );
        let mut vistas = std::collections::BTreeSet::new();
        for l in &doc {
            assert!(
                l.trim_start_matches('/').trim().is_empty() || vistas.insert(*l),
                "linea repetida: {l}"
            );
        }
        // Lo de encima del bloque no es una `const` con el doc colgando.
        let encima = lineas
            .get(..pos.saturating_sub(doc.len()))
            .and_then(|a| a.last())
            .map(|l| l.trim())
            .unwrap_or_default();
        assert!(
            !encima.starts_with("const ") && !encima.starts_with("pub const "),
            "el doc viene detras de: {encima}"
        );
    }

    /// Revision 1, IMP-1: que `gh` se usa, segun `TC2_GH` y el origen.
    #[test]
    fn gh_para_la_attestation() {
        let ensayo = Origen::Ensayo("http://127.0.0.1:1".into());
        assert_eq!(gh_de_variable(None, &Origen::GitHub).as_deref(), Some("gh"));
        assert_eq!(
            gh_de_variable(Some(""), &Origen::GitHub).as_deref(),
            Some("gh")
        );
        assert_eq!(gh_de_variable(None, &ensayo), None);
        assert_eq!(
            gh_de_variable(Some("/x/gh"), &ensayo).as_deref(),
            Some("/x/gh")
        );
        // Sin gh: solo el sha256; en ensayo: no se comprueba.
        let f = Path::new("no-importa");
        assert_eq!(comprobar_attestation(None, f), Ok(Attestation::Ensayo));
        assert_eq!(
            comprobar_attestation(Some("tc2-gh-que-no-existe-en-ningun-path"), f),
            Ok(Attestation::SinGh)
        );
    }

    /// TASK-027, criterio 3: orden de versiones con prereleases.
    #[test]
    fn comparar_versiones_con_prerelease() {
        use Ordering::{Equal, Greater, Less};
        for (a, b, o) in [
            ("1.0.0", "1.0.0", Equal),
            ("1.0.0", "1.0.1", Less),
            ("1.10.0", "1.9.0", Greater),
            ("2.0.0-rc.1", "2.0.0", Less),
            ("2.0.0", "2.0.0-rc.1", Greater),
            ("2.0.0-rc.2", "2.0.0-rc.10", Less),
            ("2.0.0-alpha", "2.0.0-alpha.1", Less),
            ("2.0.0-1", "2.0.0-alpha", Less),
            ("2.0.0-beta", "2.0.0-alpha", Greater),
            ("1.9.9", "2.0.0-rc.1", Less),
        ] {
            assert_eq!(comparar(a, b), Some(o), "{a} {b}");
        }
        assert_eq!(comparar("1.0", "1.0.0"), None);
        assert_eq!(comparar("1.0.0", "x"), None);
    }

    #[test]
    fn opciones_de_la_orden() {
        let v = |a: &[&str]| a.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        assert_eq!(leer_opciones(&v(&["--cli"])), Ok(Opciones::default()));
        assert_eq!(
            leer_opciones(&v(&["--comprobar", "--cli", "--version", "v2.0.0-rc.1"])),
            Ok(Opciones {
                comprobar: true,
                version: Some("2.0.0-rc.1".into())
            })
        );
        for (args, motivo) in [
            (v(&[]), "sin-cli"),
            (v(&["--comprobar"]), "sin-cli"),
            (v(&["--cli", "--cli"]), "argumentos"),
            (v(&["--cli", "--version"]), "sin-version"),
            (v(&["--cli", "--version", "2.0"]), "version-invalida"),
            (v(&["--cli", "x"]), "argumentos"),
        ] {
            let i = leer_opciones(&args).err();
            assert_eq!(
                i.as_ref().map(|i| (i.resultado, i.valor("motivo"))),
                Some((Resultado::Uso, Some(motivo))),
                "{args:?}"
            );
        }
    }

    /// Vuelta atras (plan, plan de pruebas): si falla el segundo renombrado,
    /// el original vuelve a su sitio intacto y no queda `.old`.
    #[test]
    fn fallo_en_el_segundo_renombrado_deja_el_original() {
        let dir = std::env::temp_dir().join(format!("tc2-sustituir-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(std::fs::create_dir_all(&dir).is_ok());
        let viejo = dir.join("tc2.exe");
        let nuevo = dir.join(".tc2-9.9.9-1.descarga");
        assert!(std::fs::write(&viejo, b"viejo").is_ok());
        assert!(std::fs::write(&nuevo, b"nuevo").is_ok());
        let mut llamadas = 0;
        let r = sustituir_con(&viejo, &nuevo, true, |a, b| {
            llamadas += 1;
            if llamadas == 2 {
                Err(std::io::Error::other("inyectado"))
            } else {
                std::fs::rename(a, b)
            }
        });
        assert_eq!(r.map_err(|e| e.to_string()), Err("inyectado".to_owned()));
        assert_eq!(llamadas, 3);
        assert_eq!(std::fs::read(&viejo).ok().as_deref(), Some(&b"viejo"[..]));
        assert!(!ruta_old(&viejo).exists());
        // Sin fallos: el nuevo en su sitio y el viejo en `.old`.
        assert!(sustituir_con(&viejo, &nuevo, true, |a, b| std::fs::rename(a, b)).is_ok());
        assert_eq!(std::fs::read(&viejo).ok().as_deref(), Some(&b"nuevo"[..]));
        assert_eq!(
            std::fs::read(ruta_old(&viejo)).ok().as_deref(),
            Some(&b"viejo"[..])
        );
        limpiar_old(&viejo);
        assert!(!ruta_old(&viejo).exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn elegir_el_binario_del_target() {
        let h = "a".repeat(64);
        let pares: Vec<(String, String)> = [
            "tc2-2.0.0-win-x64.exe",
            "tc2-2.1.0-win-x64.exe",
            "tc2-2.1.0-linux-x64",
            "install.sh",
        ]
        .iter()
        .map(|f| ((*f).to_owned(), h.clone()))
        .collect();
        let win = distribucion::TARGETS[0];
        let linux = distribucion::TARGETS[4];
        assert_eq!(
            elegir(&pares, win, None).map(|p| p.version),
            Some("2.1.0".into())
        );
        assert_eq!(
            elegir(&pares, win, Some("2.0.0")).map(|p| p.fichero),
            Some("tc2-2.0.0-win-x64.exe".into())
        );
        assert_eq!(elegir(&pares, linux, Some("2.0.0")), None);
        assert_eq!(elegir(&pares, distribucion::TARGETS[1], None), None);
        // Sin --version, el tag es la version del binario.
        assert_eq!(
            elegir(&pares, win, None).map(|p| p.tag),
            Some("2.1.0".into())
        );
        // Prerelease: el tag v2.1.0-rc.1 publica tc2-2.1.0-*.
        assert_eq!(
            elegir(&pares, linux, Some("2.1.0-rc.1")).map(|p| (p.version, p.tag)),
            Some(("2.1.0".into(), "2.1.0-rc.1".into()))
        );
        assert_eq!(elegir(&pares, linux, Some("2.1.00")), None);
    }
}
