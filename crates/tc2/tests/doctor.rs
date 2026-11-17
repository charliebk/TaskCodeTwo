//! F3-8 (TASK-035): `tc2 doctor` sobre repos reales en carpetas temporales
//! y GitHub y GitLab simulados en 127.0.0.1. Un test (o mas) por criterio;
//! los de plataforma corren las mismas aserciones en las dos (RR-15).

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::path::{Path, PathBuf};
use std::process::Command;

use comun::{Fixture, escribir};
use tc2::doctor::{self, Entorno, Estado, Hallazgo, Resumen};
use tc2::estado;
use tc2::plataforma::Personas;
use tc2::tablero::{Conectada, Consulta};
use tc2::tarea::{Cuerpo, Tarea};

const RAMA: &str = estado::RAMA;
const PERSONAS: &str = "personas:\n  - {id: 1001, login: \"ana\"}\n";

fn tarea(id: &str, estado: &str) -> String {
    let mut t = Tarea::nueva(id, &format!("Titulo de {id}"), "2026-01-01");
    t.sprint = 1;
    t.estado = estado.to_owned();
    t.escribir(&Cuerpo::leer(""))
}

/// Remoto bare con personas en `develop` y la rama de estado publicada
/// desde un repo aparte (con `extra` de mas); devuelve (bare, clon).
fn con_estado(f: &Fixture, extra: &[(&str, &str)]) -> (PathBuf, PathBuf) {
    let (bare, clon) = f.remoto_y_clon();
    escribir(&clon, ".taskcode/personas.yml", PERSONAS);
    f.crudo(&clon, &["add", ".taskcode/personas.yml"]);
    f.crudo(&clon, &["commit", "-q", "-m", "personas"]);
    f.crudo(&clon, &["push", "-q", "origin", "develop"]);
    let dir = f.raiz.join("estado");
    f.crudo(&f.raiz, &["init", "-q", "-b", RAMA, "estado"]);
    escribir(&dir, "TASK-001/tarea.md", &tarea("TASK-001", "planificada"));
    for (ruta, texto) in extra {
        escribir(&dir, ruta, texto);
    }
    f.crudo(&dir, &["add", "-A"]);
    f.crudo(&dir, &["commit", "-q", "-m", "estado"]);
    f.crudo(&dir, &["push", "-q", bare.to_str().unwrap(), RAMA]);
    f.crudo(&clon, &["fetch", "-q", "origin"]);
    (bare, clon)
}

/// Plataforma sin configurar (para lo que no es de plataforma).
fn sin_plataforma(_: &Personas) -> Result<Conectada, Consulta> {
    Err(Consulta::NoConfigurada)
}

/// Los hallazgos de `doctor` en `dir` con `conectar`.
fn hallazgos(
    g: &tc2::git::Git,
    dir: &Path,
    conectar: &tc2::tablero::Conectar<'_>,
) -> (Vec<Hallazgo>, Resumen) {
    doctor::comprobar(&Entorno {
        dir,
        git: g,
        conectar,
    })
    .unwrap_or_else(|i| panic!("{}", i.linea()))
}

fn de(hs: &[Hallazgo], area: &str, id: &str) -> Vec<String> {
    hs.iter()
        .filter(|h| h.area == area && h.id == id)
        .map(Hallazgo::linea)
        .collect()
}

/// C3: sin rama de estado, `falla motivo=sin-rama-de-estado
/// arreglo=tc2-init`; con un `tarea.md` ilegible, `falla` con su ID; con la
/// ref de la rama apuntando a algo que no es un commit, `falla` con la ref.
/// Codigo 3 en los tres.
#[test]
fn rama_de_estado_ausente_o_corrupta_falla() {
    let f = Fixture::nueva("doctor-c3");
    let (_, sin) = f.remoto_y_clon();
    let g = f.git();
    let (hs, r) = hallazgos(&g, &sin, &sin_plataforma);
    assert_eq!(
        de(&hs, "estado", "rama"),
        ["estado rama falla motivo=sin-rama-de-estado arreglo=tc2-init"]
    );
    assert_eq!(r.commit, None);
    assert_eq!(
        doctor::salida(&hs, &r, false, tc2::i18n::Idioma::Es).codigo,
        3
    );

    let f = Fixture::nueva("doctor-c3-ilegible");
    let (_, clon) = con_estado(&f, &[("TASK-009/tarea.md", "---\nesto no es\n")]);
    let g = f.git();
    let (hs, r) = hallazgos(&g, &clon, &sin_plataforma);
    assert_eq!(
        de(&hs, "estado", "rama"),
        ["estado rama ok ref=refs/remotes/origin/taskcodetwo/tareas"]
    );
    assert_eq!(
        de(&hs, "estado", "tareas"),
        ["estado tareas falla motivo=tarea-ilegible ids=TASK-009"]
    );
    assert_eq!(r.commit.as_ref().map(String::len), Some(12));
    assert_eq!(
        doctor::salida(&hs, &r, true, tc2::i18n::Idioma::Es).codigo,
        3
    );

    // La ref remota apunta a un blob: existe y no es un commit.
    let blob = f
        .crudo(&clon, &["rev-parse", "develop:README.md"])
        .trim()
        .to_owned();
    f.crudo(
        &clon,
        &[
            "update-ref",
            "refs/remotes/origin/taskcodetwo/tareas",
            &blob,
        ],
    );
    let (hs, _) = hallazgos(&g, &clon, &sin_plataforma);
    assert_eq!(
        de(&hs, "estado", "rama"),
        ["estado rama falla motivo=ref-ilegible ref=refs/remotes/origin/taskcodetwo/tareas"]
    );
    assert!(hs.iter().all(|h| h.area != "estado" || h.id != "tareas"));
    let _ = Estado::Ok;
}

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
    let (c, out, err) = tc2(
        &f,
        &vacia,
        &["doctor"],
        &[("PATH", vacia.to_str().unwrap())],
    );
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
    assert!(
        out.contains("estado rama no-comprobado motivo=sin-git"),
        "{out}"
    );
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
