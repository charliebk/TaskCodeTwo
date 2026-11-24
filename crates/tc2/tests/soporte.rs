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

/// A8-20 (MEN-1 de la revision 1): un git de escenario que falla para el
/// test, con los argumentos en el mensaje.
#[test]
#[should_panic(expected = "no-existe")]
fn git_en_con_un_git_que_falla_hace_panic() {
    let f = comun::Fixture::nueva("soporte-git-falla");
    comun::git::git_en(&f.raiz, &f.entorno(), &["rev-parse", "no-existe"]);
}

/// A8-20 (MEN-1 de la revision 1): git se lanza con `LC_ALL=C`,
/// `GIT_TERMINAL_PROMPT=0` y `GIT_CONFIG_NOSYSTEM=1`, aunque el entorno que
/// se le pasa diga otra cosa.
#[test]
fn git_en_fija_locale_y_prompt() {
    let f = comun::Fixture::nueva("soporte-git-entorno");
    // Solo el gitconfig de la fixture: `GIT_CONFIG_NOSYSTEM` lo pone git_en.
    let mut e: Vec<_> = f
        .entorno()
        .into_iter()
        .filter(|(k, _)| k != "GIT_CONFIG_NOSYSTEM")
        .collect();
    e.push(("LC_ALL".into(), "es_ES.UTF-8".into()));
    e.push(("GIT_TERMINAL_PROMPT".into(), "1".into()));
    let visto = comun::git::git_en(
        &f.raiz,
        &e,
        &[
            "-c",
            "alias.entorno=!printenv LC_ALL GIT_TERMINAL_PROMPT GIT_CONFIG_NOSYSTEM",
            "entorno",
        ],
    );
    assert_eq!(visto, "C\n0\n1\n");
}

/// A8-16 (MEN-2 de la revision 1): `salida` no da por bueno un exito que
/// escribe en stderr.
#[test]
#[should_panic(expected = "stderr en un exito")]
fn salida_rechaza_stderr_en_un_exito() {
    // El stderr lo escribe un alias de git, que trae su propio `sh`: el test
    // no depende de que haya un `sh` en el PATH (IMP-2 de la revision 2).
    let o = std::process::Command::new("git")
        .args([
            "-c",
            "alias.dos-salidas=!echo ok; echo aviso >&2",
            "dos-salidas",
        ])
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", comun::volcado(&o));
    comun::salida(&o);
}

/// A8-16 (MEN-3 de la revision 1): `con_fila` exige que la fila escrita sea
/// la de la fase, el modo, el autor y la version esperados.
#[test]
#[should_panic(expected = "| start | manual | agente |")]
fn con_fila_rechaza_una_fila_de_otra_fase() {
    let hoy = tc2::toma::instante(tc2::identidad::ahora());
    let antes = "actualizado: \"2020-01-01\"\n";
    let despues = format!(
        "actualizado: \"{}\"\n| {hoy} | start | manual | agente | {} |\n",
        &hoy[..10],
        tc2::VERSION
    );
    comun::remoto::con_fila(antes, &despues, "liberar", "agente");
}

/// A3-06: una sola `valor`, sobre la linea de contrato (la primera) y sin el
/// final de linea; no confunde una clave con otra que empieza igual.
#[test]
fn valor_recorta_y_lee_la_linea_de_contrato() {
    assert_eq!(comun::valor("k=v\n", "k"), Some("v"));
    assert_eq!(comun::valor("a k=1\nb k=2\n", "k"), Some("1"));
    assert_eq!(comun::valor("x kk=1 k=2", "k"), Some("2"));
}
