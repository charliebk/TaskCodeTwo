//! F3-8 (TASK-035): `tc2 doctor` sobre repos reales en carpetas temporales
//! y GitHub y GitLab simulados en 127.0.0.1. Un test (o mas) por criterio;
//! los de plataforma corren las mismas aserciones en las dos (RR-15).

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::path::Path;
use std::process::Command;

use comun::Fixture;

/// (codigo, stdout, stderr) de `tc2 <args>` en `dir`, sin tokens.
fn tc2(f: &Fixture, dir: &Path, args: &[&str], env: &[(&str, &str)]) -> (i32, String, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_tc2"))
        .current_dir(dir)
        .args(args)
        .envs(f.entorno())
        .env("LC_ALL", "C")
        .env("TC2_IDIOMA", "es")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CEILING_DIRECTORIES", &f.raiz)
        .env_remove("TC2_TOKEN_AGENTE")
        .env_remove("TC2_TOKEN_PERSONA")
        .env_remove("TC2_TOKEN_REVISOR")
        .envs(env.iter().copied())
        .output()
        .expect("lanzar tc2");
    (
        o.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&o.stdout).into_owned(),
        String::from_utf8_lossy(&o.stderr).into_owned(),
    )
}

/// C6: sin `git` en el PATH, `falla motivo=sin-git` (codigo 3) y lo demas
/// `no-comprobado`, sin suponer nada; fuera de un repo, `rechazada
/// motivo=no-es-repo` con el diagnostico en stderr.
#[test]
fn sin_git_falla_y_fuera_de_un_repo_rechaza() {
    let f = Fixture::nueva("doctor-c6");
    let vacia = f.raiz.join("vacia");
    std::fs::create_dir_all(&vacia).unwrap();
    let (c, out, err) = tc2(&f, &vacia, &["doctor"], &[("PATH", vacia.to_str().unwrap())]);
    assert_eq!(c, 3, "{out} {err}");
    let primera = out.lines().next().unwrap();
    assert!(
        primera.starts_with("doctor rechazada fallos=1 avisos=0 no-comprobado=4"),
        "{out}"
    );
    assert!(
        out.contains("entorno git falla motivo=sin-git arreglo=instalar-git"),
        "{out}"
    );
    assert!(out.contains("estado rama no-comprobado motivo=sin-git"), "{out}");
    assert!(err.contains("entorno git falla"), "{err}");
    let (c, out, err) = tc2(&f, &vacia, &["doctor"], &[]);
    assert_eq!(
        (c, out.as_str()),
        (3, "doctor rechazada motivo=no-es-repo\n"),
        "{err}"
    );
    assert_eq!(err.lines().count(), 1, "{err}");
    let (c, out, _) = tc2(&f, &vacia, &["doctor", "--recursos"], &[]);
    assert_eq!((c, out.as_str()), (2, "doctor uso motivo=argumentos\n"));
}
