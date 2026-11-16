//! Doble de prueba de TASK-027 (F2-12): un «tc2 9.9.9» para
//! `tests/actualizar_cli.rs`, que lo sirve como la version nueva de una
//! release de ensayo. Responde como el `tc2` real a lo unico que
//! `actualizar --cli` y el test le preguntan:
//!
//! - `--version` → `tc2 9.9.9`;
//! - `version --json` → `{version, protocolo, target, sha256}`, con el
//!   sha256 de su propio fichero.
//!
//! No es parte de la distribucion: `cargo xtask empaquetar` solo copia el
//! binario `tc2`.

use std::io::Write;
use std::process::ExitCode;

use tc2::version::{DatosVersion, PROTOCOLO, target_actual};

/// Version que dice ser.
const VERSION: &str = "9.9.9";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let texto = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["--version"] => format!("tc2 {VERSION}\n"),
        ["version", "--json"] => {
            let sha256 = std::env::current_exe()
                .and_then(std::fs::read)
                .map(|b| distribucion::sha256::hex(&b))
                .unwrap_or_default();
            let d = DatosVersion {
                version: VERSION.to_owned(),
                protocolo: PROTOCOLO,
                target: target_actual(),
                sha256,
            };
            format!("{}\n", d.json())
        }
        _ => return ExitCode::from(2),
    };
    match std::io::stdout().write_all(texto.as_bytes()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::from(1),
    }
}
