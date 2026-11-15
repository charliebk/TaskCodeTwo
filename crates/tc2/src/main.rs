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
    // Si no se puede escribir la salida (stdout cerrado), no se da por bueno:
    // quien lee el codigo 0 creeria que recibio la respuesta.
    let mut stdout = std::io::stdout();
    if let Err(e) = stdout
        .write_all(salida.stdout.as_bytes())
        .and_then(|()| stdout.flush())
    {
        let _ = writeln!(std::io::stderr(), "tc2: no se pudo escribir la salida: {e}");
        return ExitCode::from(1);
    }
    // Un fallo al escribir en stderr no tiene a donde avisarse.
    let _ = std::io::stderr().write_all(salida.stderr.as_bytes());
    ExitCode::from(salida.codigo)
}
