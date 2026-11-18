//! F3-12 (TASK-084): progreso en vivo con el binario real: `tc2 estado
//! --stdin` (la status line de Claude Code) y `tc2 status --seguir`, sobre
//! repos en una carpeta temporal.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use comun::Fixture;

fn texto(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

/// `tc2 estado --stdin` en `dir` con `json` por stdin y el entorno `vars`.
fn estado_stdin(dir: &Path, json: &str, vars: &[(&str, &std::ffi::OsStr)]) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_tc2"));
    c.current_dir(dir)
        .args(["estado", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (k, v) in vars {
        c.env(k, v);
    }
    let mut hijo = c.spawn().expect("lanzar tc2");
    hijo.stdin
        .take()
        .unwrap()
        .write_all(json.as_bytes())
        .unwrap();
    hijo.wait_with_output().unwrap()
}

/// El JSON de la status line con `workspace.current_dir`.
fn json_de(dir: &Path) -> String {
    serde_json::json!({
        "model": {"display_name": "Opus"},
        "workspace": {"current_dir": dir.display().to_string(), "project_dir": "x"},
        "session_id": "s"
    })
    .to_string()
}

/// C7: sin tarea en curso, fuera de un repo o con un JSON que no vale, sale
/// `tc2 · sin tarea en curso` con codigo 0 y stderr vacio.
#[test]
fn c7_sin_tarea_o_fuera_de_un_repo() {
    let f = Fixture::nueva("progreso-c7");
    let (_bare, clon) = f.remoto_y_clon();
    let fuera = f.raiz.join("fuera");
    std::fs::create_dir_all(&fuera).unwrap();
    for (dir, json) in [
        (&clon, json_de(&clon)),
        (&clon, "no es json".to_owned()),
        (&clon, String::new()),
        (&fuera, json_de(&fuera)),
        (&fuera, json_de(&f.raiz.join("no-existe"))),
    ] {
        let o = estado_stdin(dir, &json, &[]);
        assert_eq!(o.status.code(), Some(0), "{json}");
        assert_eq!(texto(&o.stdout), "tc2 · sin tarea en curso\n", "{json}");
        assert!(o.stderr.is_empty(), "{}", texto(&o.stderr));
    }
}
