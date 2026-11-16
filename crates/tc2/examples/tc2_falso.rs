//! Doble de prueba de TASK-027 (F2-12): un «tc2 9.9.9» para
//! `tests/actualizar_cli.rs`, que lo sirve como la version nueva de una
//! release de ensayo. Responde como el `tc2` real a lo unico que
//! `actualizar --cli` y el test le preguntan:
//!
//! - `--version` → `tc2 9.9.9`;
//! - `version --json` → `{version, protocolo, target, sha256}`, con el
//!   sha256 de su propio fichero.
//!
//! Solo usa `std` y la caja `distribucion`, no la biblioteca `tc2`, para que
//! el binario sea pequeno: en Windows se ha visto (TASK-027) que la pila de
//! red local de una maquina con filtros corta a veces a los ~19 s las
//! descargas de mas de ~450 KB por 127.0.0.1, tambien con curl y
//! `python -m http.server`.
//!
//! No es parte de la distribucion: `cargo xtask empaquetar` solo copia el
//! binario `tc2`.

use std::io::Write;
use std::process::ExitCode;

use distribucion::{PROTOCOLO, Target};

/// Version que dice ser.
const VERSION: &str = "9.9.9";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let texto = match args.as_slice() {
        ["--version"] => format!("tc2 {VERSION}\n"),
        ["version", "--json"] => {
            let sha256 = std::env::current_exe()
                .and_then(std::fs::read)
                .map(|b| distribucion::sha256::hex(&b))
                .unwrap_or_default();
            let target = Target::actual().map_or("desconocido", |t| t.alias);
            // Los valores no llevan comillas ni barras: alias, version y hex.
            format!(
                "{{\"version\":\"{VERSION}\",\"protocolo\":{PROTOCOLO},\"target\":\"{target}\",\"sha256\":\"{sha256}\"}}\n"
            )
        }
        _ => return ExitCode::from(2),
    };
    match std::io::stdout().write_all(texto.as_bytes()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::from(1),
    }
}
