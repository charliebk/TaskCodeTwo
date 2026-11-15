//! Punto de entrada de `tc2`: delega en la biblioteca y solo escribe.

use std::io::Write;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = Vec::new();
    for a in std::env::args_os().skip(1) {
        match a.into_string() {
            Ok(s) => args.push(s),
            Err(_) => {
                let _ = writeln!(std::io::stderr(), "tc2: argumento que no es UTF-8");
                return ExitCode::from(2);
            }
        }
    }
    let salida = tc2::ejecutar(args);
    let _ = std::io::stdout().write_all(salida.stdout.as_bytes());
    let _ = std::io::stderr().write_all(salida.stderr.as_bytes());
    ExitCode::from(salida.codigo)
}
