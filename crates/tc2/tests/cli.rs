//! Pruebas del binario compilado: lo lanzan como lo lanzaria una persona.

#![allow(clippy::panic)] // en un test, panic! es la forma de fallar

use std::process::Command;

fn tc2(args: &[&str]) -> std::process::Output {
    match Command::new(env!("CARGO_BIN_EXE_tc2")).args(args).output() {
        Ok(o) => o,
        Err(e) => panic!("no se pudo lanzar tc2: {e}"),
    }
}

#[test]
fn version_da_codigo_0_y_la_version_de_cargo_toml() {
    let o = tc2(&["--version"]);
    assert_eq!(o.status.code(), Some(0));
    let esperada = format!("tc2 {}\n", env!("CARGO_PKG_VERSION"));
    assert_eq!(String::from_utf8_lossy(&o.stdout), esperada);
    assert!(o.stderr.is_empty());
}

#[test]
fn orden_desconocida_da_codigo_distinto_de_0() {
    let o = tc2(&["no-existe"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(o.stdout.is_empty());
    assert!(String::from_utf8_lossy(&o.stderr).contains("no-existe"));
}
