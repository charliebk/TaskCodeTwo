//! Donde vive el `tc2` de esta maquina (F2-12 y F2-14): quien lo gestiona
//! ([`Modo`]), las rutas que dejan los instaladores de F2-11 y el lanzador
//! del plugin, y el `.old` de una actualizacion en Windows.
//!
//! Todo lo que depende del entorno del proceso (HOME, `LOCALAPPDATA`,
//! `TC2_INSTALL_DIR`...) entra por un [`Entorno`] inyectado: los tests de
//! `tc2 desinstalar` usan un HOME falso y nunca el real de la maquina.

use std::path::{Path, PathBuf};

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
    // Cada gestor por su propia estructura de carpetas, no por un nombre
    // suelto: una carpeta `Scoop\bin` o `~/.cargo/otra` de la persona no es
    // de un gestor. `dir` son las carpetas, sin el fichero.
    let dir = partes.get(..n.saturating_sub(1)).unwrap_or_default();
    let seguidas = |a: &str, b: &str| dir.windows(2).any(|w| w[0] == a && w[1] == b);
    // `<raiz>/node_modules/<paquete>/...`: el binario dentro de un paquete.
    let npm = dir
        .iter()
        .position(|p| p == "node_modules")
        .is_some_and(|i| i + 1 < dir.len());
    // `Cellar/<formula>/<version>/...`, o el prefijo de Homebrew.
    let homebrew = dir
        .iter()
        .position(|p| p == "cellar")
        .is_some_and(|i| i + 2 < dir.len())
        || dir.starts_with(&["opt".to_owned(), "homebrew".to_owned()])
        || seguidas("linuxbrew", ".linuxbrew");
    let gestor = if npm {
        Some("npm")
    } else if homebrew {
        Some("homebrew")
    } else if seguidas("winget", "packages") || seguidas("winget", "links") {
        Some("winget")
    } else if seguidas("scoop", "apps") || seguidas("scoop", "shims") {
        Some("scoop")
    } else if dir.len() >= 2 && dir[dir.len() - 2] == ".cargo" && dir[dir.len() - 1] == "bin" {
        Some("cargo")
    } else {
        None
    };
    gestor.map_or(Modo::Instalador, Modo::Gestor)
}

/// La orden exacta que quita un `tc2` instalado con un gestor de paquetes
/// (nombres de F1-12 D1a: el paquete es `taskcodetwo`; en cargo, el crate
/// `tc2`).
pub fn orden_del_gestor(gestor: &str) -> String {
    match gestor {
        "npm" => "npm uninstall -g taskcodetwo",
        "homebrew" => "brew uninstall taskcodetwo",
        "winget" => "winget uninstall taskcodetwo",
        "scoop" => "scoop uninstall taskcodetwo",
        "cargo" => "cargo uninstall tc2",
        _ => "",
    }
    .to_owned()
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

/// Sistema para el que se calculan las rutas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sistema {
    /// Windows.
    Windows,
    /// macOS.
    Mac,
    /// Linux y el resto de Unix.
    Linux,
}

impl Sistema {
    /// El sistema en que corre este proceso.
    pub fn actual() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::Mac
        } else {
            Self::Linux
        }
    }

    /// Nombre del binario: `tc2.exe` o `tc2`.
    pub fn binario(self) -> &'static str {
        match self {
            Self::Windows => "tc2.exe",
            Self::Mac | Self::Linux => "tc2",
        }
    }
}

/// Lo que se lee del entorno del proceso, inyectable en los tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entorno {
    /// Sistema para el que se calculan las rutas.
    pub sistema: Sistema,
    /// HOME (en Windows, `USERPROFILE`).
    pub home: Option<PathBuf>,
    /// `LOCALAPPDATA` (Windows).
    pub local_app_data: Option<PathBuf>,
    /// `XDG_CACHE_HOME` (Unix).
    pub xdg_cache_home: Option<PathBuf>,
    /// `TC2_INSTALL_DIR`, si se instalo en otro sitio.
    pub install_dir: Option<PathBuf>,
    /// `CLAUDE_CONFIG_DIR` (por defecto `~/.claude`).
    pub claude_config_dir: Option<PathBuf>,
    /// El ejecutable de este `tc2`.
    pub exe: PathBuf,
    /// PID de este proceso.
    pub pid: u32,
}

/// Una variable de entorno no vacia como ruta.
fn ruta_de(valor: Option<String>) -> Option<PathBuf> {
    valor.filter(|v| !v.is_empty()).map(PathBuf::from)
}

impl Entorno {
    /// El entorno del proceso, leido con `leer` (para no tocar las variables
    /// del proceso en los tests).
    pub fn de_variables(
        sistema: Sistema,
        exe: PathBuf,
        pid: u32,
        leer: impl Fn(&str) -> Option<String>,
    ) -> Self {
        let home = match sistema {
            Sistema::Windows => ruta_de(leer("USERPROFILE")).or_else(|| ruta_de(leer("HOME"))),
            Sistema::Mac | Sistema::Linux => ruta_de(leer("HOME")),
        };
        Self {
            sistema,
            home,
            local_app_data: ruta_de(leer("LOCALAPPDATA")),
            xdg_cache_home: ruta_de(leer("XDG_CACHE_HOME")),
            install_dir: ruta_de(leer("TC2_INSTALL_DIR")),
            claude_config_dir: ruta_de(leer("CLAUDE_CONFIG_DIR")),
            exe,
            pid,
        }
    }

    /// El entorno real de este proceso.
    pub fn del_proceso() -> std::io::Result<Self> {
        Ok(Self::de_variables(
            Sistema::actual(),
            std::env::current_exe()?,
            std::process::id(),
            |k| std::env::var(k).ok(),
        ))
    }

    /// Modo del `tc2` que se esta ejecutando.
    pub fn modo(&self) -> Modo {
        detectar_modo(&self.exe.to_string_lossy())
    }

    /// Directorio por defecto de los instaladores de F2-11:
    /// `%LOCALAPPDATA%\taskcodetwo\bin` o `~/.local/bin`.
    pub fn dir_por_defecto(&self) -> Option<PathBuf> {
        match self.sistema {
            Sistema::Windows => self
                .local_app_data
                .as_ref()
                .map(|l| l.join("taskcodetwo").join("bin")),
            Sistema::Mac | Sistema::Linux => {
                self.home.as_ref().map(|h| h.join(".local").join("bin"))
            }
        }
    }

    /// Si `dir` es una carpeta solo de `tc2` (la de Windows por defecto):
    /// vacia, se puede borrar. `~/.local/bin` o un `TC2_INSTALL_DIR` no.
    pub fn dir_propio(&self, dir: &Path) -> bool {
        self.sistema == Sistema::Windows && self.dir_por_defecto().as_deref() == Some(dir)
    }

    /// Directorios donde pudo dejar `tc2` un instalador: `TC2_INSTALL_DIR` y
    /// el de por defecto, sin repetir.
    pub fn dirs_de_instalacion(&self) -> Vec<PathBuf> {
        let mut v: Vec<PathBuf> = Vec::new();
        for d in [self.install_dir.clone(), self.dir_por_defecto()]
            .into_iter()
            .flatten()
        {
            if !v.contains(&d) {
                v.push(d);
            }
        }
        v
    }

    /// Carpetas `taskcodetwo` de la cache del lanzador del plugin (ADR F1-14,
    /// punto 4), sin repetir: `%LOCALAPPDATA%\taskcodetwo`,
    /// `$XDG_CACHE_HOME/taskcodetwo` o `~/.cache/taskcodetwo` y, en macOS,
    /// `~/Library/Caches/taskcodetwo`. La cache es su subcarpeta `cli`.
    pub fn raices_de_cache(&self) -> Vec<PathBuf> {
        let mut v: Vec<PathBuf> = Vec::new();
        let mut poner = |p: Option<PathBuf>| {
            if let Some(p) = p.map(|p| p.join("taskcodetwo"))
                && !v.contains(&p)
            {
                v.push(p);
            }
        };
        match self.sistema {
            Sistema::Windows => poner(self.local_app_data.clone()),
            Sistema::Mac | Sistema::Linux => {
                if self.sistema == Sistema::Mac {
                    poner(self.home.as_ref().map(|h| h.join("Library").join("Caches")));
                }
                poner(
                    self.xdg_cache_home
                        .clone()
                        .or_else(|| self.home.as_ref().map(|h| h.join(".cache"))),
                );
            }
        }
        v
    }

    /// Carpeta de configuracion de Claude Code (`CLAUDE_CONFIG_DIR` o
    /// `~/.claude`).
    pub fn dir_claude(&self) -> Option<PathBuf> {
        self.claude_config_dir
            .clone()
            .or_else(|| self.home.as_ref().map(|h| h.join(".claude")))
    }
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
        // Cada fila fija el texto exacto de la orden que desinstala su gestor.
        let npm = "npm uninstall -g taskcodetwo";
        let brew = "brew uninstall taskcodetwo";
        let winget = "winget uninstall taskcodetwo";
        let scoop = "scoop uninstall taskcodetwo";
        let cargo = "cargo uninstall tc2";
        for (ruta, gestor, orden) in [
            (
                r"C:\Users\ana\AppData\Roaming\npm\node_modules\taskcodetwo\bin\tc2.exe",
                "npm",
                npm,
            ),
            ("/opt/homebrew/Cellar/tc2/2.0.0/bin/tc2", "homebrew", brew),
            // Solo la rama del prefijo `/opt/homebrew`, sin `Cellar`.
            ("/opt/homebrew/bin/tc2", "homebrew", brew),
            (
                r"C:\Users\ana\AppData\Local\Microsoft\WinGet\Packages\x\tc2.exe",
                "winget",
                winget,
            ),
            // Solo la rama de `WinGet\Links`, sin `Packages`.
            (
                r"C:\Users\ana\AppData\Local\Microsoft\WinGet\Links\tc2.exe",
                "winget",
                winget,
            ),
            (
                r"C:\Users\ana\scoop\apps\tc2\current\tc2.exe",
                "scoop",
                scoop,
            ),
            ("/home/ana/.cargo/bin/tc2", "cargo", cargo),
            ("/home/linuxbrew/.linuxbrew/bin/tc2", "homebrew", brew),
            (r"C:\Users\ana\scoop\shims\tc2.exe", "scoop", scoop),
        ] {
            assert_eq!(detectar_modo(ruta), Modo::Gestor(gestor), "{ruta}");
            assert_eq!(orden_del_gestor(gestor), orden, "{gestor}");
        }
        // Revision 1, MEN-1: un nombre de gestor suelto en la ruta no basta.
        for propia in [
            r"C:\Users\ana\Scoop\bin\tc2.exe",
            r"C:\herramientas\scoop\tc2.exe",
            "/home/ana/.cargo/otra/tc2",
            "/home/ana/.cargo/bin/sub/tc2",
            "/home/ana/node_modules/tc2",
            "/home/ana/cellar/tc2",
            "/home/ana/homebrew/tc2",
            r"C:\Users\ana\winget\tc2.exe",
        ] {
            assert_eq!(detectar_modo(propia), Modo::Instalador, "{propia}");
        }
    }

    #[test]
    fn rutas_del_entorno_por_sistema() {
        let vars = |k: &str| match k {
            "HOME" => Some("/h".to_owned()),
            "USERPROFILE" => Some("C:/u".to_owned()),
            "LOCALAPPDATA" => Some("C:/u/L".to_owned()),
            "TC2_INSTALL_DIR" => Some(String::new()),
            _ => None,
        };
        let w = Entorno::de_variables(Sistema::Windows, "C:/x/tc2.exe".into(), 1, vars);
        assert_eq!(w.home, Some(PathBuf::from("C:/u")));
        assert_eq!(w.install_dir, None);
        assert_eq!(
            w.dirs_de_instalacion(),
            vec![PathBuf::from("C:/u/L/taskcodetwo/bin")]
        );
        assert!(w.dir_propio(Path::new("C:/u/L/taskcodetwo/bin")));
        assert_eq!(
            w.raices_de_cache(),
            vec![PathBuf::from("C:/u/L/taskcodetwo")]
        );
        assert_eq!(w.dir_claude(), Some(PathBuf::from("C:/u/.claude")));
        // `TC2_INSTALL_DIR` igual a la carpeta por defecto: sale una sola vez.
        let repetida = |k: &str| match k {
            "TC2_INSTALL_DIR" => Some("C:/u/L/taskcodetwo/bin".to_owned()),
            otra => vars(otra),
        };
        let wr = Entorno::de_variables(Sistema::Windows, "C:/x/tc2.exe".into(), 1, repetida);
        assert_eq!(
            wr.dirs_de_instalacion(),
            vec![PathBuf::from("C:/u/L/taskcodetwo/bin")]
        );
        let l = Entorno::de_variables(Sistema::Linux, "/h/.local/bin/tc2".into(), 1, vars);
        assert_eq!(
            l.dirs_de_instalacion(),
            vec![PathBuf::from("/h/.local/bin")]
        );
        assert!(!l.dir_propio(Path::new("/h/.local/bin")));
        assert_eq!(
            l.raices_de_cache(),
            vec![PathBuf::from("/h/.cache/taskcodetwo")]
        );
        let m = Entorno::de_variables(Sistema::Mac, "/h/.local/bin/tc2".into(), 1, vars);
        assert_eq!(
            m.raices_de_cache(),
            vec![
                PathBuf::from("/h/Library/Caches/taskcodetwo"),
                PathBuf::from("/h/.cache/taskcodetwo")
            ]
        );
    }
}
