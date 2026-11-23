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

/// A8-20: el git de sistema comun lee la configuracion de la fixture, no la
/// del usuario que corre los tests.
#[test]
fn git_en_aisla_la_configuracion_del_usuario() {
    let f = comun::Fixture::nueva("soporte-git-en");
    let email = comun::git::git_en(&f.raiz, &f.entorno(), &["config", "--global", "user.email"]);
    assert_eq!(email.trim_end(), "prueba@example.com");
}

/// A8-20: sin `GIT_CONFIG_GLOBAL` no se lanza git (entraria la configuracion
/// global de quien corre los tests).
#[test]
#[should_panic(expected = "GIT_CONFIG_GLOBAL")]
fn git_en_sin_gitconfig_propio_no_lanza_git() {
    comun::git::git_en(std::path::Path::new("."), &[], &["--version"]);
}

/// A3-06: una sola `valor`, sobre la linea de contrato (la primera) y sin el
/// final de linea; no confunde una clave con otra que empieza igual.
#[test]
fn valor_recorta_y_lee_la_linea_de_contrato() {
    assert_eq!(comun::valor("k=v\n", "k"), Some("v"));
    assert_eq!(comun::valor("a k=1\nb k=2\n", "k"), Some("1"));
    assert_eq!(comun::valor("x kk=1 k=2", "k"), Some("2"));
}
