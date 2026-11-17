//! Punto de entrada de `tc2`: delega en la biblioteca y solo escribe.
//!
//! Es el unico sitio que lee el idioma del entorno (F2-8): `TC2_IDIOMA`,
//! luego `LC_ALL`, `LC_MESSAGES` y `LANG`, y si no, espanol.

use std::io::Write;
use std::process::ExitCode;

use tc2::i18n::catalogo::cli as txt;
use tc2::i18n::{Idioma, Mensaje};

fn main() -> ExitCode {
    // F2-12: en Windows, `actualizar --cli` deja `tc2.exe.old` (un ejecutable
    // en marcha no se puede borrar); lo borra la siguiente ejecucion. Una
    // llamada al sistema si no existe.
    // `desinstalar` no: su `--simular` no toca nada (revision 1, MEN-6 de
    // TASK-078), y el `.old` lo quita su propio plan.
    #[cfg(windows)]
    if std::env::args_os()
        .nth(1)
        .is_none_or(|a| a != "desinstalar")
        && let Ok(exe) = std::env::current_exe()
    {
        tc2::actualizar_cli::limpiar_old(&exe);
    }
    let idioma = Idioma::de_entorno(|k| std::env::var(k).ok());
    let mut args = Vec::new();
    for a in std::env::args_os().skip(1) {
        match a.into_string() {
            Ok(s) => args.push(s),
            Err(_) => {
                let m = Mensaje::nuevo(&txt::ARGUMENTO_NO_UTF8);
                let _ = writeln!(std::io::stderr(), "{}", m.texto(idioma));
                return ExitCode::from(2);
            }
        }
    }
    let salida = tc2::ejecutar_en(idioma, args);
    // Si no se puede escribir la salida (stdout cerrado), no se da por bueno:
    // quien lee el codigo 0 creeria que recibio la respuesta.
    let mut stdout = std::io::stdout();
    if let Err(e) = stdout
        .write_all(salida.stdout.as_bytes())
        .and_then(|()| stdout.flush())
    {
        let m = Mensaje::nuevo(&txt::NO_SE_PUDO_ESCRIBIR).con("error", e.to_string());
        let _ = writeln!(std::io::stderr(), "{}", m.texto(idioma));
        return ExitCode::from(1);
    }
    // Un fallo al escribir en stderr no tiene a donde avisarse.
    let _ = std::io::stderr().write_all(salida.stderr.as_bytes());
    ExitCode::from(salida.codigo)
}
