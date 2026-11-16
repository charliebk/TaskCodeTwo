//! `tc2 actualizar --cli [--comprobar] [--version X]` (F2-12, ADR F1-14
//! puntos 4 y 5): sustituye el `tc2` instalado por la version de la release
//! sin reinstalar.
//!
//! - La version disponible sale de `SHA256SUMS` de la release (la ultima
//!   estable, o la de `--version`): una peticion, la misma fuente que los
//!   instaladores de F2-11.
//! - `--comprobar` solo lee: `actualizar ok actual=X disponible=Y` si hay una
//!   mas nueva, `sin-cambios` si no. Es el aviso de version nueva: lo usan
//!   `doctor` y un hook opcional, nunca una consulta pasiva.
//! - Un `tc2` de la cache del plugin no se toca (lo gestiona el plugin con su
//!   `cli.lock`), ni uno de un gestor de paquetes.
//! - El binario nuevo se baja junto al actual (`.tc2-<v>-<pid>.descarga`),
//!   su sha256 lo calcula este proceso, que no es el comprobado (punto 4), y
//!   solo si casa con `SHA256SUMS` se pregunta al nuevo `version --json` y se
//!   sustituye. En Windows, por renombrados (actual → `<exe>.old`, nuevo → su
//!   sitio) con vuelta atras; el `.old` lo borra la siguiente ejecucion.

use std::cmp::Ordering;
use std::path::{Path, PathBuf};

use distribucion::Target;

use crate::contrato::{Informe, Resultado};
use crate::descarga::{self, Descargador, Fallo, Origen};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::actualizar as txt;

/// Nombre de la orden.
pub const NOMBRE: &str = "actualizar";

/// Quien gestiona el `tc2` que se esta ejecutando.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modo {
    /// Instalado con la linea de instalacion (o copiado a mano): se actualiza.
    Instalador,
    /// La cache del plugin (`.../taskcodetwo/cli/<version>/tc2[.exe]`): lo
    /// gestiona el plugin con `cli.lock`.
    Plugin,
    /// Un gestor de paquetes (npm, Homebrew, winget, Scoop, cargo): se
    /// actualiza con el.
    Gestor(&'static str),
}

/// Modo de un ejecutable por su ruta (sin tocar el disco). Reconoce las
/// rutas de Windows, macOS y Linux sin depender del sistema en que corre.
pub fn detectar_modo(ruta: &str) -> Modo {
    let partes: Vec<String> = ruta
        .split(['/', '\\'])
        .filter(|p| !p.is_empty())
        .map(str::to_ascii_lowercase)
        .collect();
    let n = partes.len();
    let es_tc2 = partes.last().is_some_and(|f| f == "tc2" || f == "tc2.exe");
    if es_tc2
        && n >= 4
        && partes.get(n - 4).is_some_and(|p| p == "taskcodetwo")
        && partes.get(n - 3).is_some_and(|p| p == "cli")
    {
        return Modo::Plugin;
    }
    for p in &partes {
        let gestor = match p.as_str() {
            "node_modules" => "npm",
            "cellar" | "homebrew" | ".linuxbrew" | "linuxbrew" => "homebrew",
            "winget" => "winget",
            "scoop" => "scoop",
            ".cargo" => "cargo",
            _ => continue,
        };
        return Modo::Gestor(gestor);
    }
    Modo::Instalador
}

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
    actualizar(&exe, &op, &origen, &Descargador::nuevo())
}

/// Lo que publica la release para un target.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Publicado {
    version: String,
    fichero: String,
    sha256: String,
}

/// El binario de `target` en `SHA256SUMS`: el de `version` si se pide, y si
/// no, el de version mas alta.
fn elegir(pares: &[(String, String)], target: Target, version: Option<&str>) -> Option<Publicado> {
    let mut candidatos = pares.iter().filter_map(|(f, h)| {
        target.version_de_fichero(f).map(|v| Publicado {
            version: v,
            fichero: f.clone(),
            sha256: h.clone(),
        })
    });
    match version {
        Some(v) => candidatos.find(|p| p.version == v),
        None => {
            candidatos.max_by(|a, b| comparar(&a.version, &b.version).unwrap_or(Ordering::Equal))
        }
    }
}

/// `actualizar --cli` sobre el ejecutable `exe`.
pub fn actualizar(exe: &Path, op: &Opciones, origen: &Origen, d: &Descargador) -> Informe {
    limpiar_old(exe);
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
        return base(Resultado::Ok);
    }
    instalar(exe, target, &publicado, origen, d)
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
) -> Informe {
    let Some(dir) = exe.parent() else {
        return sin_permiso(exe, "sin carpeta");
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
    let url = origen.url_binario(&publicado.version, &publicado.fichero);
    let bajada = d
        .a_escritor(&url, &mut fichero, descarga::LIMITE_BINARIO)
        .map_err(|f| match f {
            Fallo::Escritura { error } => sin_permiso(&temporal, &error),
            otro => otro.informe(NOMBRE),
        })
        .and_then(|_| {
            fichero
                .sync_all()
                .map_err(|e| sin_permiso(&temporal, &e.to_string()))
        });
    drop(fichero);
    if let Err(i) = bajada {
        return i;
    }
    let obtenido = match std::fs::read(&temporal) {
        Ok(b) => distribucion::sha256::hex(&b),
        Err(e) => return sin_permiso(&temporal, &e.to_string()),
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
    if let Err(e) = hacer_ejecutable(&temporal) {
        return sin_permiso(&temporal, &e.to_string());
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

/// Lanza `nuevo version --json` y exige su version, su target y el sha256
/// que ya calculo este proceso. `Err` con el detalle si no.
fn comprobar_nuevo(nuevo: &Path, version: &str, alias: &str, sha256: &str) -> Result<(), String> {
    let salida = std::process::Command::new(nuevo)
        .args(["version", "--json"])
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|e| e.to_string())?;
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

/// `<exe>.old`: donde queda el binario anterior en Windows hasta la
/// siguiente ejecucion.
pub fn ruta_old(exe: &Path) -> PathBuf {
    let mut s = exe.as_os_str().to_owned();
    s.push(".old");
    PathBuf::from(s)
}

/// Borra `<exe>.old` si lo hay (lo deja una actualizacion en Windows, donde
/// un ejecutable en marcha no se puede borrar). Si sigue en uso, se queda.
pub fn limpiar_old(exe: &Path) {
    let _ = std::fs::remove_file(ruta_old(exe));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// TASK-027, criterio 6: la cache del plugin se reconoce en los tres
    /// sistemas, y no se confunde con una instalacion normal.
    #[test]
    fn detectar_modo_con_rutas_de_los_tres_sistemas() {
        for plugin in [
            r"C:\Users\ana\AppData\Local\taskcodetwo\cli\2.0.0\tc2.exe",
            r"c:\users\ana\appdata\local\TaskCodeTwo\CLI\2.0.0-rc.1\TC2.EXE",
            "/Users/ana/.cache/taskcodetwo/cli/2.0.0/tc2",
            "/Users/ana/Library/Caches/taskcodetwo/cli/2.0.0/tc2",
            "/home/ana/.cache/taskcodetwo/cli/2.0.0/tc2",
        ] {
            assert_eq!(detectar_modo(plugin), Modo::Plugin, "{plugin}");
        }
        for instalador in [
            r"C:\Users\ana\AppData\Local\taskcodetwo\bin\tc2.exe",
            "/Users/ana/.local/bin/tc2",
            "/home/ana/.local/bin/tc2",
            "/home/ana/taskcodetwo/cli/tc2",
            "/home/ana/taskcodetwo/cli/2.0.0/otro",
            "/opt/herramientas/tc2",
        ] {
            assert_eq!(detectar_modo(instalador), Modo::Instalador, "{instalador}");
        }
        for (ruta, gestor) in [
            (
                r"C:\Users\ana\AppData\Roaming\npm\node_modules\taskcodetwo\bin\tc2.exe",
                "npm",
            ),
            ("/opt/homebrew/Cellar/tc2/2.0.0/bin/tc2", "homebrew"),
            (
                r"C:\Users\ana\AppData\Local\Microsoft\WinGet\Packages\x\tc2.exe",
                "winget",
            ),
            (r"C:\Users\ana\scoop\apps\tc2\current\tc2.exe", "scoop"),
            ("/home/ana/.cargo/bin/tc2", "cargo"),
        ] {
            assert_eq!(detectar_modo(ruta), Modo::Gestor(gestor), "{ruta}");
        }
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
    }
}
