//! Doble de `gh` para `tests/actualizar_cli.rs` (TASK-027, revision 1,
//! IMP-1): simula la attestation sin red. `tc2 actualizar --cli` lo usa si
//! `TC2_GH` apunta a el.
//!
//! - `auth status` → 0 (hay sesion).
//! - `attestation verify <fichero> --repo <r> --signer-workflow <w>` → 0 si
//!   `GH_FALSO=ok`; si no, 1 con una linea en stderr, como `gh` cuando la
//!   attestation no vale.
//! - Con `GH_FALSO_REGISTRO=<fichero>`, anade al fichero una linea con los
//!   argumentos de cada llamada, separados por tabuladores.
//!
//! Solo usa `std`. No es parte de la distribucion.

use std::io::Write;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Ok(registro) = std::env::var("GH_FALSO_REGISTRO") {
        let linea = format!("{}\n", args.join("\t"));
        let escrito = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(registro)
            .and_then(|mut f| f.write_all(linea.as_bytes()));
        if escrito.is_err() {
            return ExitCode::from(2);
        }
    }
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["auth", "status"] => ExitCode::SUCCESS,
        [
            "attestation",
            "verify",
            _fichero,
            "--repo",
            _,
            "--signer-workflow",
            _,
        ] => {
            if std::env::var("GH_FALSO").as_deref() == Ok("ok") {
                ExitCode::SUCCESS
            } else {
                let _ = writeln!(
                    std::io::stderr(),
                    "X Verification failed: no attestations found (gh falso)"
                );
                ExitCode::from(1)
            }
        }
        _ => ExitCode::from(2),
    }
}
