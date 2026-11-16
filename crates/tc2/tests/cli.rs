//! Pruebas del binario compilado: lo lanzan como lo lanzaria una persona.

#![allow(clippy::panic)] // en un test, panic! es la forma de fallar

use std::process::Command;

fn tc2(args: &[&str]) -> std::process::Output {
    tc2_en("es", args)
}

/// Lanza `tc2` con `TC2_IDIOMA` fijado, para no depender del locale de la
/// maquina que corre los tests.
fn tc2_en(idioma: &str, args: &[&str]) -> std::process::Output {
    match Command::new(env!("CARGO_BIN_EXE_tc2"))
        .args(args)
        .env("TC2_IDIOMA", idioma)
        .output()
    {
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

#[test]
fn tc2_idioma_en_da_el_diagnostico_en_ingles_con_el_mismo_codigo() {
    let es = tc2_en("es", &["no-existe"]);
    let en = tc2_en("en", &["no-existe"]);
    assert_eq!(es.status.code(), Some(2));
    assert_eq!(en.status.code(), Some(2));
    assert_eq!(es.stdout, en.stdout);
    let (es, en) = (
        String::from_utf8_lossy(&es.stderr),
        String::from_utf8_lossy(&en.stderr),
    );
    assert!(
        es.starts_with("tc2: orden desconocida «no-existe». Uso:"),
        "{es}"
    );
    assert!(
        en.starts_with("tc2: unknown command «no-existe». Usage:"),
        "{en}"
    );
    // Una orden con informe: stdout igual, solo cambia el diagnostico.
    let es = tc2_en("es", &["guardar"]);
    let en = tc2_en("en", &["guardar"]);
    assert_eq!(es.status.code(), en.status.code());
    assert_eq!(es.stdout, en.stdout);
    assert_eq!(
        String::from_utf8_lossy(&en.stderr),
        "usage: guardar \"<message>\" <paths...>\n"
    );
}

/// TASK-027, criterio 4: el binario real da en `version --json` el sha256 de
/// su propio fichero, calculado aqui por otro proceso.
#[test]
fn version_json_del_binario_con_su_sha256() {
    let o = tc2(&["version", "--json"]);
    assert_eq!(o.status.code(), Some(0));
    let v: serde_json::Value = match serde_json::from_slice(&o.stdout) {
        Ok(v) => v,
        Err(e) => panic!("version --json no es JSON: {e}"),
    };
    let bytes = match std::fs::read(env!("CARGO_BIN_EXE_tc2")) {
        Ok(b) => b,
        Err(e) => panic!("no se pudo leer tc2: {e}"),
    };
    assert_eq!(v["sha256"], distribucion::sha256::hex(&bytes));
    assert_eq!(v["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(v.as_object().map(serde_json::Map::len), Some(4));
}
