//! Tests del soporte comun de los tests de integracion (`tests/comun`).
//! Van en un solo binario para no repetirse en cada fichero que declara
//! `mod comun;`.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::collections::BTreeMap;
use std::ffi::OsStr;

/// A8-02: el lanzador comun quita las credenciales de la maquina (tokens de
/// los tres roles y GitHub Apps) y fija el idioma y el locale.
#[test]
fn el_lanzador_quita_credenciales_y_fija_el_idioma() {
    let c = comun::orden_tc2();
    let envs: BTreeMap<&OsStr, Option<&OsStr>> = c.get_envs().collect();
    for quitada in [
        "TC2_TOKEN_PERSONA",
        "TC2_TOKEN_AGENTE",
        "TC2_TOKEN_REVISOR",
        "TC2_APP_AGENTE_ID",
        "TC2_APP_AGENTE_CLAVE",
        "TC2_APP_REVISOR_ID",
        "TC2_APP_REVISOR_CLAVE",
    ] {
        assert_eq!(
            envs.get(OsStr::new(quitada)),
            Some(&None),
            "{quitada} tiene que quitarse del entorno"
        );
    }
    for (clave, valor) in [
        ("TC2_IDIOMA", "es"),
        ("LC_ALL", "C"),
        ("GIT_TERMINAL_PROMPT", "0"),
    ] {
        assert_eq!(
            envs.get(OsStr::new(clave)),
            Some(&Some(OsStr::new(valor))),
            "{clave}"
        );
    }
    assert_eq!(c.get_program(), OsStr::new(comun::BINARIO));
}
