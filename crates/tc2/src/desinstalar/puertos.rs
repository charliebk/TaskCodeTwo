//! Puerto de lo que `desinstalar --maquina` toca fuera del sistema de
//! ficheros: el PATH de usuario del registro de Windows, si otro proceso usa
//! un binario y el borrado del binario en marcha tras salir (F2-14, D4 y D5).
//!
//! [`MaquinaReal`] lo hace con un hijo `powershell.exe` (solo Windows; en
//! Unix no hay registro y un binario en marcha se puede borrar). Los tests
//! usan [`MaquinaEnMemoria`]: nunca tocan el registro ni lanzan nada.

use std::cell::RefCell;
use std::io;
use std::path::{Path, PathBuf};

/// Lo que `desinstalar` necesita de la maquina.
pub trait Maquina {
    /// Valor crudo (sin expandir `%VARIABLES%`) del PATH de usuario, o
    /// `None` si no hay (o no es Windows).
    fn leer_path_usuario(&self) -> io::Result<Option<String>>;
    /// Escribe `nuevo` en el PATH de usuario si sigue valiendo `viejo` (si
    /// no, error: otro lo cambio entre medias), con el mismo tipo de valor.
    fn escribir_path_usuario(&self, viejo: &str, nuevo: &str) -> io::Result<()>;
    /// Si otro proceso distinto de `pid` ejecuta `ruta`.
    fn en_uso_por_otro(&self, ruta: &Path, pid: u32) -> io::Result<bool>;
    /// Deja un proceso auxiliar que, cuando `pid` acabe (como mucho 10 s),
    /// borra `ficheros` y despues las carpetas de `dirs` que queden vacias.
    fn programar_borrado(&self, pid: u32, ficheros: &[PathBuf], dirs: &[PathBuf])
    -> io::Result<()>;
}

/// Una peticion de autoborrado registrada por [`MaquinaEnMemoria`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Borrado {
    /// Proceso al que se espera.
    pub pid: u32,
    /// Ficheros que se borran.
    pub ficheros: Vec<PathBuf>,
    /// Carpetas que se borran si quedan vacias.
    pub dirs: Vec<PathBuf>,
}

/// Maquina falsa para los tests: un PATH de usuario en memoria, una lista
/// de binarios «en uso por otro» y el registro de los autoborrados.
#[derive(Debug, Default)]
pub struct MaquinaEnMemoria {
    /// PATH de usuario.
    pub path: RefCell<Option<String>>,
    /// Binarios que otro proceso esta ejecutando.
    pub en_uso: Vec<PathBuf>,
    /// Autoborrados pedidos.
    pub borrados: RefCell<Vec<Borrado>>,
}

impl Maquina for MaquinaEnMemoria {
    fn leer_path_usuario(&self) -> io::Result<Option<String>> {
        Ok(self.path.borrow().clone())
    }

    fn escribir_path_usuario(&self, viejo: &str, nuevo: &str) -> io::Result<()> {
        let mut p = self.path.borrow_mut();
        if p.as_deref() != Some(viejo) {
            return Err(io::Error::other("el PATH cambio entre medias"));
        }
        *p = Some(nuevo.to_owned());
        Ok(())
    }

    fn en_uso_por_otro(&self, ruta: &Path, _pid: u32) -> io::Result<bool> {
        Ok(self.en_uso.iter().any(|r| r == ruta))
    }

    fn programar_borrado(
        &self,
        pid: u32,
        ficheros: &[PathBuf],
        dirs: &[PathBuf],
    ) -> io::Result<()> {
        self.borrados.borrow_mut().push(Borrado {
            pid,
            ficheros: ficheros.to_vec(),
            dirs: dirs.to_vec(),
        });
        Ok(())
    }
}

/// La maquina de verdad.
#[derive(Debug, Default)]
pub struct MaquinaReal;

/// Inversa de `Add-PathUsuario` de `install.ps1`: lee el valor crudo, sin
/// expandir las `%VARIABLES%`.
#[cfg(windows)]
const PS_LEER_PATH: &str = "[Console]::OutputEncoding = [Text.Encoding]::UTF8; $k = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment'); if ($null -eq $k) { exit 3 }; $v = $k.GetValue('Path', $null, [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames); $k.Close(); if ($null -eq $v) { exit 3 }; [Console]::Out.Write([string]$v)";

/// Escribe el PATH nuevo con el mismo tipo de valor si sigue siendo el
/// viejo, y avisa a las ventanas abiertas como el instalador.
#[cfg(windows)]
const PS_ESCRIBIR_PATH: &str = "$ErrorActionPreference = 'Stop'; $k = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment', $true); try { $actual = [string]$k.GetValue('Path', '', [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames); if ($actual -cne $env:TC2_PATH_VIEJO) { exit 4 }; $tipo = [Microsoft.Win32.RegistryValueKind]::ExpandString; if ($k.GetValueNames() -contains 'Path') { $tipo = $k.GetValueKind('Path') }; $k.SetValue('Path', $env:TC2_PATH_NUEVO, $tipo) } finally { $k.Close() }; [Environment]::SetEnvironmentVariable('TC2_INSTALAR_AVISO', '1', 'User'); [Environment]::SetEnvironmentVariable('TC2_INSTALAR_AVISO', $null, 'User')";

/// Cuenta los procesos distintos de `TC2_PID` que ejecutan `TC2_RUTA`.
#[cfg(windows)]
const PS_EN_USO: &str = "$n = @(Get-CimInstance Win32_Process | Where-Object { $_.ExecutablePath -and $_.ExecutablePath -ieq $env:TC2_RUTA -and $_.ProcessId -ne [int]$env:TC2_PID }).Count; [Console]::Out.Write($n)";

/// Espera al proceso (como mucho 10 s) y borra; reintenta unos segundos si
/// un antivirus tiene el fichero abierto un momento.
#[cfg(windows)]
const PS_AUTOBORRADO: &str = "try { Wait-Process -Id ([int]$env:TC2_PID) -Timeout 10 -ErrorAction Stop } catch {}; foreach ($f in ($env:TC2_FICHEROS -split \"`n\")) { if (-not $f) { continue }; for ($i = 0; $i -lt 20; $i++) { if (-not (Test-Path -LiteralPath $f)) { break }; try { Remove-Item -LiteralPath $f -Force -ErrorAction Stop; break } catch { Start-Sleep -Milliseconds 250 } } }; foreach ($d in ($env:TC2_DIRS -split \"`n\")) { if ($d -and (Test-Path -LiteralPath $d) -and -not (Get-ChildItem -LiteralPath $d -Force)) { Remove-Item -LiteralPath $d -Force -ErrorAction SilentlyContinue } }";

#[cfg(windows)]
fn powershell(script: &str, vars: &[(&str, &std::ffi::OsStr)]) -> io::Result<std::process::Output> {
    let mut c = std::process::Command::new("powershell.exe");
    c.args(["-NoProfile", "-NonInteractive", "-Command", script])
        .stdin(std::process::Stdio::null());
    for (k, v) in vars {
        c.env(k, v);
    }
    c.output()
}

#[cfg(windows)]
impl Maquina for MaquinaReal {
    fn leer_path_usuario(&self) -> io::Result<Option<String>> {
        let o = powershell(PS_LEER_PATH, &[])?;
        match o.status.code() {
            Some(0) => Ok(Some(String::from_utf8_lossy(&o.stdout).into_owned())),
            Some(3) => Ok(None),
            _ => Err(io::Error::other(
                String::from_utf8_lossy(&o.stderr).trim().to_owned(),
            )),
        }
    }

    fn escribir_path_usuario(&self, viejo: &str, nuevo: &str) -> io::Result<()> {
        let o = powershell(
            PS_ESCRIBIR_PATH,
            &[
                ("TC2_PATH_VIEJO", viejo.as_ref()),
                ("TC2_PATH_NUEVO", nuevo.as_ref()),
            ],
        )?;
        match o.status.code() {
            Some(0) => Ok(()),
            Some(4) => Err(io::Error::other("el PATH de usuario cambio entre medias")),
            _ => Err(io::Error::other(
                String::from_utf8_lossy(&o.stderr).trim().to_owned(),
            )),
        }
    }

    fn en_uso_por_otro(&self, ruta: &Path, pid: u32) -> io::Result<bool> {
        let pid = pid.to_string();
        let o = powershell(
            PS_EN_USO,
            &[("TC2_RUTA", ruta.as_os_str()), ("TC2_PID", pid.as_ref())],
        )?;
        if !o.status.success() {
            return Err(io::Error::other(
                String::from_utf8_lossy(&o.stderr).trim().to_owned(),
            ));
        }
        Ok(String::from_utf8_lossy(&o.stdout).trim() != "0")
    }

    fn programar_borrado(
        &self,
        pid: u32,
        ficheros: &[PathBuf],
        dirs: &[PathBuf],
    ) -> io::Result<()> {
        use std::os::windows::process::CommandExt;
        // DETACHED_PROCESS | CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP: el
        // auxiliar sobrevive a este proceso y a su consola.
        const FLAGS: u32 = 0x0000_0008 | 0x0800_0000 | 0x0000_0200;
        let unir = |v: &[PathBuf]| {
            let mut s = std::ffi::OsString::new();
            for (i, p) in v.iter().enumerate() {
                if i > 0 {
                    s.push("\n");
                }
                s.push(p.as_os_str());
            }
            s
        };
        std::process::Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-WindowStyle",
                "Hidden",
                "-Command",
                PS_AUTOBORRADO,
            ])
            .env("TC2_PID", pid.to_string())
            .env("TC2_FICHEROS", unir(ficheros))
            .env("TC2_DIRS", unir(dirs))
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .creation_flags(FLAGS)
            .spawn()
            .map(|_| ())
    }
}

#[cfg(not(windows))]
impl Maquina for MaquinaReal {
    fn leer_path_usuario(&self) -> io::Result<Option<String>> {
        Ok(None)
    }

    fn escribir_path_usuario(&self, _viejo: &str, _nuevo: &str) -> io::Result<()> {
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }

    fn en_uso_por_otro(&self, _ruta: &Path, _pid: u32) -> io::Result<bool> {
        // En Unix un binario en marcha se puede borrar: nunca bloquea.
        Ok(false)
    }

    fn programar_borrado(
        &self,
        _pid: u32,
        _ficheros: &[PathBuf],
        _dirs: &[PathBuf],
    ) -> io::Result<()> {
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }
}
