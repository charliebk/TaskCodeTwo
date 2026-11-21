//! `tc2 verificar --perfil <p> [--hook]` (F5-6): lanza el comando de
//! verificacion del proyecto, solo si su orden esta confiada.
//!
//! El nucleo es un comando determinista (F1-11 decision 10): lee
//! `verificacion` de `.taskcode/config.yml` **del arbol de trabajo**
//! (F1-8 decision 5), pregunta a [`crate::confianza::estado`] por la clave
//! `verificacion.comando` y, si esta confiada, lanza `verificacion.shell` con
//! `<comando> --perfil <p>`, `TC2_PERFIL=<p>` y el `timeout` del perfil.
//! Codigos (F1-11 decision 8): cualquier codigo distinto de 0 es rojo, y un
//! `timeout` agotado tambien, salvo en `rapida`, donde es «excedido».
//!
//! `--hook` es la misma orden colgada del evento `Stop` de Claude Code: la
//! decision es la misma y solo cambia como se devuelve. Con
//! `stop_hook_active: true` en el JSON de stdin sale con 0 sin ejecutar
//! nada; si no se ejecuta (sin confianza, sin el perfil, configuracion
//! invalida) sale con 0 y una linea en stderr que dice que hacer, nunca
//! bloquea por eso; en rojo sale con 2 y un resumen corto en stderr, que
//! Claude Code le pasa al agente.

use std::io::{IsTerminal, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::Salida;
use crate::confianza::{self, Estado};
use crate::config::{PerfilVerificacion, Verificacion};
use crate::contrato::{Informe, Resultado};
use crate::git::Git;
use crate::i18n::catalogo::verificacion as t;
use crate::i18n::{Idioma, Mensaje};

/// Nombre de la orden.
pub const NOMBRE: &str = "verificar";

/// Clave de la configuracion que se confia y se ejecuta.
pub const CLAVE: &str = "verificacion.comando";

/// Perfiles de F1-7 (decision 4).
pub const PERFILES: [&str; 4] = ["rapida", "impacto", "completa", "pesada"];

/// Lineas de la salida del comando que `--hook` pasa al agente en rojo.
const LINEAS_RESUMEN: usize = 15;

/// Tope de lo que se lee de stdin con `--hook`.
const LIMITE_STDIN: u64 = 1 << 20;

/// Cada cuanto se mira si el comando termino.
const SONDEO: Duration = Duration::from_millis(10);

/// `tc2 verificar` con los argumentos ya sin `--json`.
pub fn orden(args: &[String], json: bool, idioma: Idioma) -> Salida {
    // Con `--hook`, nada que no sea el rojo de la rapida bloquea la parada:
    // `stop_hook_active` corta antes de validar nada mas, y un error de uso
    // (la entrada de settings.json mal escrita) sale con 0 y un aviso.
    let hook = args.iter().any(|a| a == "--hook");
    if hook && stop_hook_activo() {
        return Informe::nuevo(NOMBRE, Resultado::SinCambios)
            .con("motivo", "stop-hook-activo")
            .salida_en(json, idioma);
    }
    let perfil = match leer_argumentos(args) {
        Ok(p) => p,
        Err(i) if hook => {
            let mut s = i.salida_en(json, idioma);
            s.codigo = 0;
            s.stderr = aviso_de_hook(
                Mensaje::nuevo(&t::HOOK_MAL_ESCRITO)
                    .con("motivo", i.diagnostico_en(idioma).unwrap_or_default()),
                idioma,
            );
            return s;
        }
        Err(i) => return i.salida_en(json, idioma),
    };
    let captura = if hook { crear_captura() } else { None };
    let informe = match std::env::current_dir() {
        Ok(dir) => {
            let almacen = confianza::almacen_del_proceso(NOMBRE);
            match almacen {
                Ok(a) => verificar(&Git::nuevo(), &dir, &a, &perfil, captura.as_deref()),
                Err(i) => i,
            }
        }
        Err(e) => Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(
            Mensaje::nuevo(&crate::i18n::catalogo::cli::CARPETA_ACTUAL).con("error", e.to_string()),
        ),
    };
    let salida = if hook {
        como_hook(&informe, &perfil, captura.as_deref(), json, idioma)
    } else {
        informe.salida_en(json, idioma)
    };
    if let Some(c) = captura {
        let _ = std::fs::remove_file(c);
    }
    salida
}

fn uso() -> Informe {
    Informe::nuevo(NOMBRE, Resultado::Uso)
        .con("motivo", "argumentos")
        .diagnostico(&t::USO_VERIFICAR)
}

/// El perfil de `--perfil <p> [--hook]`, o el informe de uso.
fn leer_argumentos(args: &[String]) -> Result<String, Informe> {
    let mut perfil = None;
    let mut hook = false;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--perfil" => match it.next() {
                Some(p) if perfil.is_none() => perfil = Some(p.clone()),
                _ => return Err(uso()),
            },
            "--hook" if !hook => hook = true,
            _ => return Err(uso()),
        }
    }
    perfil
        .filter(|p| PERFILES.contains(&p.as_str()))
        .ok_or_else(uso)
}

/// Una linea de stderr para la persona con `--hook` (el diagnostico de una
/// config con YAML invalido trae varias).
fn aviso_de_hook(m: Mensaje, idioma: Idioma) -> String {
    format!("{}\n", crate::contrato::una_linea(&m.texto(idioma)))
}

/// `stop_hook_active` del JSON de stdin del evento `Stop`. Sin JSON (stdin
/// en una terminal, vacio o mal formado) no se sabe y se sigue: el tope de
/// continuaciones de Claude Code corta un bucle.
fn stop_hook_activo() -> bool {
    let stdin = std::io::stdin();
    if stdin.is_terminal() {
        return false;
    }
    let mut entrada = String::new();
    let _ = stdin.lock().take(LIMITE_STDIN).read_to_string(&mut entrada);
    serde_json::from_str::<serde_json::Value>(&entrada)
        .ok()
        .and_then(|v| {
            v.get("stop_hook_active")
                .and_then(serde_json::Value::as_bool)
        })
        .unwrap_or(false)
}

/// Fichero donde `--hook` recoge la salida del comando: nombre aleatorio y
/// creado en exclusiva (`create_new`, no sigue un enlace ni pisa un fichero
/// que ya exista); se borra al terminar. Si no se puede crear, la salida va a
/// stderr como sin `--hook`.
fn crear_captura() -> Option<PathBuf> {
    use ring::rand::SecureRandom;
    let mut azar = [0u8; 16];
    ring::rand::SystemRandom::new().fill(&mut azar).ok()?;
    let nombre = crate::confianza::cierre::sha_hex(&azar);
    let ruta = std::env::temp_dir().join(format!("tc2-verificar-{}.log", &nombre[..32]));
    std::fs::File::options()
        .write(true)
        .create_new(true)
        .open(&ruta)
        .ok()
        .map(|_| ruta)
}

/// Tope de lo que se lee del final de la captura para el resumen.
const COLA_BYTES: u64 = 64 * 1024;

/// Las ultimas [`LINEAS_RESUMEN`] lineas de la captura, leyendo solo su
/// final.
fn cola(ruta: &Path) -> Vec<String> {
    use std::io::{Seek, SeekFrom};
    let Ok(mut f) = std::fs::File::open(ruta) else {
        return Vec::new();
    };
    let largo = f.metadata().map(|m| m.len()).unwrap_or(0);
    let _ = f.seek(SeekFrom::Start(largo.saturating_sub(COLA_BYTES)));
    let mut datos = Vec::new();
    let _ = f.take(COLA_BYTES).read_to_end(&mut datos);
    let texto = String::from_utf8_lossy(&datos);
    let lineas: Vec<&str> = texto.lines().collect();
    lineas[lineas.len().saturating_sub(LINEAS_RESUMEN)..]
        .iter()
        .map(|l| (*l).to_owned())
        .collect()
}

/// La decision y la ejecucion, comunes a la orden y al hook. Con `captura`,
/// la salida del comando va a ese fichero; sin ella, a stderr.
pub fn verificar(
    git: &Git,
    dir: &Path,
    almacen: &Path,
    perfil: &str,
    captura: Option<&Path>,
) -> Informe {
    match decidir_y_lanzar(git, dir, almacen, perfil, captura) {
        Ok(i) | Err(i) => i,
    }
}

fn decidir_y_lanzar(
    git: &Git,
    dir: &Path,
    almacen: &Path,
    perfil: &str,
    captura: Option<&Path>,
) -> Result<Informe, Informe> {
    let repo = confianza::abrir(git, dir, NOMBRE)?;
    let config = crate::config::del_arbol(&repo.raiz).map_err(|mut i| {
        i.comando = NOMBRE.to_owned();
        i
    })?;
    let rechazo = |motivo: &str, m: Mensaje| {
        Informe::nuevo(NOMBRE, Resultado::Rechazada)
            .con("motivo", motivo)
            .con("perfil", perfil)
            .diagnostico(m)
    };
    let Some(v) = config.verificacion.as_ref() else {
        return Err(rechazo(
            "sin-verificacion",
            Mensaje::nuevo(&t::SIN_VERIFICACION).con("ruta", crate::config::RUTA),
        ));
    };
    let Some(p) = perfil_de(v, perfil) else {
        return Err(rechazo(
            "sin-perfil",
            Mensaje::nuevo(&t::SIN_PERFIL)
                .con("ruta", crate::config::RUTA)
                .con("perfil", perfil),
        ));
    };
    let orden = confianza::orden_configurada(&config, CLAVE).unwrap_or_default();
    let ev = confianza::estado(
        &repo,
        almacen,
        CLAVE,
        &orden,
        &config.confianza_incluye,
        NOMBRE,
    )?;
    let base = |r: Resultado| {
        Informe::nuevo(NOMBRE, r)
            .con("perfil", perfil)
            .con("clave", CLAVE)
            .con("confianza", ev.estado.nombre())
            .con("hasheados", ev.hasheados.to_string())
    };
    match ev.estado {
        Estado::Confiado => {}
        Estado::Nunca | Estado::Cambiado => {
            let que = if ev.estado == Estado::Nunca {
                &t::NUNCA_CONFIADA
            } else {
                &t::CAMBIADA
            };
            return Err(base(Resultado::Rechazada)
                .con("motivo", "sin-confianza")
                .diagnostico(Mensaje::nuevo(que).con("clave", CLAVE)));
        }
    }
    let inicio = Instant::now();
    let fin = lanzar(&repo.raiz, v, perfil, p.timeout, captura).map_err(|e| {
        base(Resultado::Error)
            .con("motivo", "no-se-lanza")
            .diagnostico(
                Mensaje::nuevo(&t::NO_SE_LANZA)
                    .con("shell", v.shell.as_str())
                    .con("error", e.to_string()),
            )
    })?;
    let segundos = inicio.elapsed().as_secs().to_string();
    Ok(match fin {
        Fin::Codigo(0) => base(Resultado::Ok)
            .con("resultado", "verde")
            .con("segundos", segundos),
        // F1-11 decision 8: en la rapida, agotar el presupuesto no es rojo.
        Fin::Timeout if perfil == "rapida" => base(Resultado::Ok)
            .con("resultado", "excedido")
            .con("segundos", segundos),
        Fin::Timeout => base(Resultado::Rechazada)
            .con("motivo", "timeout")
            .con("resultado", "rojo")
            .con("segundos", segundos)
            .diagnostico(
                Mensaje::nuevo(&t::TIMEOUT)
                    .con("perfil", perfil)
                    .con("timeout", p.timeout.to_string()),
            ),
        Fin::Codigo(c) => base(Resultado::Rechazada)
            .con("motivo", "rojo")
            .con("resultado", "rojo")
            .con("codigo", c.to_string())
            .con("segundos", segundos)
            .diagnostico(
                Mensaje::nuevo(&t::ROJO)
                    .con("perfil", perfil)
                    .con("codigo", c.to_string()),
            ),
    })
}

fn perfil_de<'a>(v: &'a Verificacion, perfil: &str) -> Option<&'a PerfilVerificacion> {
    match perfil {
        "rapida" => v.perfiles.rapida.as_ref(),
        "impacto" => v.perfiles.impacto.as_ref(),
        "completa" => Some(&v.perfiles.completa),
        "pesada" => v.perfiles.pesada.as_ref(),
        _ => None,
    }
}

/// Como termino el comando.
enum Fin {
    /// Con este codigo (`-1` si murio por una senal).
    Codigo(i32),
    /// Se agoto el `timeout` y se mato.
    Timeout,
}

/// La linea de la shell: `<shell> <opciones> "<comando> --perfil <p>"`.
fn orden_de_shell(shell: &str, linea: &str) -> Command {
    match shell {
        "cmd" => {
            let mut c = Command::new("cmd");
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                c.raw_arg(format!("/d /s /c \"{linea}\""));
            }
            #[cfg(not(windows))]
            c.args(["/d", "/s", "/c", linea]);
            c
        }
        "pwsh" => {
            let mut c = Command::new("pwsh");
            c.args(["-NoProfile", "-NonInteractive", "-Command", linea]);
            c
        }
        otra => {
            let mut c = Command::new(otra);
            c.args(["-c", linea]);
            c
        }
    }
}

fn lanzar(
    raiz: &Path,
    v: &Verificacion,
    perfil: &str,
    timeout: u32,
    captura: Option<&Path>,
) -> std::io::Result<Fin> {
    let linea = format!("{} --perfil {perfil}", v.comando);
    let mut c = orden_de_shell(&v.shell, &linea);
    c.current_dir(raiz)
        .env("TC2_PERFIL", perfil)
        .stdin(Stdio::null());
    match captura {
        Some(ruta) => {
            // Ya creado en exclusiva por `crear_captura`: no se crea de nuevo.
            let f = std::fs::File::options().write(true).open(ruta)?;
            c.stdout(f.try_clone()?).stderr(f);
        }
        // La salida del comando no se mezcla con la linea de contrato.
        None => {
            c.stdout(std::io::stderr()).stderr(Stdio::inherit());
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        c.process_group(0);
    }
    let mut hijo = c.spawn()?;
    let limite = Instant::now() + Duration::from_secs(u64::from(timeout));
    loop {
        if let Some(s) = hijo.try_wait()? {
            return Ok(Fin::Codigo(s.code().unwrap_or(-1)));
        }
        if Instant::now() >= limite {
            matar_arbol(hijo.id());
            let _ = hijo.kill();
            let _ = hijo.wait();
            return Ok(Fin::Timeout);
        }
        std::thread::sleep(SONDEO);
    }
}

/// Mata el comando y lo que haya lanzado (la shell sola dejaria vivos a sus
/// hijos).
fn matar_arbol(pid: u32) {
    let pid = pid.to_string();
    #[cfg(windows)]
    let mut c = {
        let mut c = Command::new("taskkill");
        c.args(["/T", "/F", "/PID", &pid]);
        c
    };
    #[cfg(not(windows))]
    let mut c = {
        let mut c = Command::new("kill");
        c.args(["-KILL", "--", &format!("-{pid}")]);
        c
    };
    let _ = c
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

/// Si el informe es un rojo de la verificacion (y no un «no se ejecuta»).
fn es_rojo(i: &Informe) -> bool {
    matches!(i.valor("motivo"), Some("rojo" | "timeout"))
}

/// El mismo informe, devuelto como lo espera el evento `Stop`: 2 en rojo
/// (bloquea y el resumen va al agente), 0 en todo lo demas.
fn como_hook(
    i: &Informe,
    perfil: &str,
    captura: Option<&Path>,
    json: bool,
    idioma: Idioma,
) -> Salida {
    let mut s = i.salida_en(json, idioma);
    if i.resultado.es_exito() {
        return s;
    }
    let motivo = i.diagnostico_en(idioma).unwrap_or_default();
    if es_rojo(i) {
        let mut resumen = Mensaje::nuevo(&t::HOOK_ROJO)
            .con("motivo", motivo)
            .texto(idioma);
        resumen.push('\n');
        for l in captura.map(cola).unwrap_or_default() {
            resumen.push_str(&l);
            resumen.push('\n');
        }
        s.codigo = 2;
        s.stderr = resumen;
    } else {
        s.codigo = 0;
        s.stderr = aviso_de_hook(
            Mensaje::nuevo(&t::HOOK_NO_SE_EJECUTA)
                .con("perfil", perfil)
                .con("motivo", motivo),
            idioma,
        );
    }
    s
}
