//! Esperar a un proceso hijo con plazo y matarlo si no acaba (A9-01): la
//! espera de `tc2 verificar` y la de `actualizar_cli::ejecutar_con_limite`,
//! en un solo sitio. Al cumplirse el plazo se mata el arbol entero (el hijo
//! y lo que haya lanzado), sin dependencias: `taskkill /T /F` en Windows y
//! `kill -KILL -- -<pgid>` en Unix, con el hijo en un grupo propio
//! ([`grupo_propio`]).

use std::io;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

/// Cada cuanto se mira si el hijo termino.
const SONDEO: Duration = Duration::from_millis(10);

/// Pone `c` en un grupo de procesos propio (`process_group(0)` en Unix),
/// para que [`esperar_con_plazo`] pueda matar el arbol entero. En Windows no
/// hace nada: `taskkill /T` sigue el arbol por el proceso padre.
pub(crate) fn grupo_propio(c: &mut Command) -> &mut Command {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        c.process_group(0);
    }
    c
}

/// Espera a `hijo` como mucho `plazo`: `Ok(Some(estado))` si acabo y
/// `Ok(None)` si se cumplio el plazo. Al cumplirse, o si falla `try_wait`
/// (sin saber si sigue vivo), se mata el arbol entero ([`matar_arbol`]) y el
/// hijo, y se espera a que muera. En Unix el arbol entero solo cae si el
/// hijo se lanzo con [`grupo_propio`].
pub(crate) fn esperar_con_plazo(
    hijo: &mut Child,
    plazo: Duration,
) -> io::Result<Option<ExitStatus>> {
    let fin = Instant::now() + plazo;
    loop {
        match hijo.try_wait() {
            Ok(Some(s)) => return Ok(Some(s)),
            Ok(None) if Instant::now() >= fin => {
                matar(hijo);
                return Ok(None);
            }
            Ok(None) => std::thread::sleep(SONDEO),
            Err(e) => {
                matar(hijo);
                return Err(e);
            }
        }
    }
}

/// Mata el arbol de `hijo` y al propio hijo, y lo recoge.
fn matar(hijo: &mut Child) {
    matar_arbol(hijo.id());
    let _ = hijo.kill();
    let _ = hijo.wait();
}

/// Mata el proceso `pid` y lo que haya lanzado (una shell sola dejaria vivos
/// a sus hijos, con las tuberias heredadas abiertas).
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Un hijo que no acaba y deja un nieto en segundo plano con stdout
    /// heredado: al cumplirse el plazo cae el arbol entero y la tuberia llega
    /// al final enseguida. Matando solo al hijo, el nieto la retendria 30 s.
    #[test]
    fn al_plazo_mata_el_arbol_entero() -> io::Result<()> {
        use std::io::Read as _;
        let mut c = if cfg!(windows) {
            let mut c = Command::new("cmd");
            c.args([
                "/c",
                "start /b ping -n 30 127.0.0.1 & ping -n 30 127.0.0.1 >nul",
            ]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(["-c", "sleep 30 & sleep 30"]);
            c
        };
        grupo_propio(&mut c)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let mut hijo = c.spawn()?;
        let mut salida = hijo.stdout.take();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut v = Vec::new();
            if let Some(s) = salida.as_mut() {
                let _ = s.read_to_end(&mut v);
            }
            let _ = tx.send(());
        });
        let inicio = Instant::now();
        let r = esperar_con_plazo(&mut hijo, Duration::from_secs(1))?;
        assert!(r.is_none(), "{r:?}");
        let eof = rx.recv_timeout(Duration::from_secs(5));
        assert!(
            eof.is_ok(),
            "la tuberia sigue abierta tras {:?}",
            inicio.elapsed()
        );
        Ok(())
    }
}
