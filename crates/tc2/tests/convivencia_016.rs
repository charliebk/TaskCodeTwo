//! C4 de TASK-116 (F1-12 D1a): un repo con la huella de la 0.16 queda
//! intacto tras `tc2 new`, `crear_rama_y_worktree` (el «start» de la 2.0)
//! y una escritura de estado sin red que deja una pendiente, y todo lo que la 2.0 deja nuevo vive bajo los nombres de `huella.rs`.
//!
//! La huella de la 0.16 es la que deja la 0.16.1 (su codigo en la cache del
//! plugin y OpenDockTrading y ToolsaWeb, medido el 2026-11-16):
//! `.git/taskcode/` (`puertas.log`, `gitflow/`, `cadena.lock`,
//! `progreso.json`, `progreso-contador`, `freno-ultimo-verde.json`),
//! `.taskcode/config.yml`, `.taskcode/doctor-deploy.yml` y `tareas/` en la
//! historia de `develop`.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use std::time::Duration;

use comun::ops::{Anotar, catalogo};
use comun::{Fixture, escribir};
use tc2::contrato::Resultado;
use tc2::estado::{self, Contexto, Politica};
use tc2::{git, huella};

/// Ficheros de la 0.16 en `.git/taskcode/`.
const COMUN_016: [(&str, &str); 6] = [
    ("puertas.log", "2026-11-16T08:00:00Z TASK-001 verde\n"),
    ("gitflow/gitflow-2026-11-16.log", "start TASK-001\n"),
    ("cadena.lock", ""),
    (
        "progreso.json",
        "{\"tarea\":\"TASK-001\",\"fase\":\"en-curso\"}\n",
    ),
    ("progreso-contador", ""),
    ("freno-ultimo-verde.json", "{\"commit\":\"abc\"}\n"),
];

/// Ficheros versionados de la 0.16 en `develop`.
const ARBOL_016: [(&str, &str); 4] = [
    (
        ".taskcode/config.yml",
        "modo_flujo: automatico\ncomando_verificacion: scripts\\verificar.bat\ntimeout_verificacion: 1200\n",
    ),
    (".taskcode/doctor-deploy.yml", "desactivados: []\n"),
    ("tareas/ROADMAP.md", "# Roadmap\n\n- TASK-001\n"),
    (
        "tareas/04-terminadas/TASK-001/tarea.md",
        "---\nid: TASK-001\nestado: terminada\n---\n",
    ),
];

/// Todo lo de la 0.16 que no debe cambiar: ficheros de `.git/taskcode/` y
/// `.taskcode/` (con su contenido), refs de `develop` y `main` (local y
/// remoto), el espacio `refs/taskcode/` (vacio: la 0.16 no lo usa, y la 2.0
/// no debe escribir en el) y el arbol versionado de `develop`.
fn huella_016(f: &Fixture, bare: &Path, clon: &Path) -> BTreeMap<String, String> {
    let mut h = BTreeMap::new();
    for raiz in [clon.join(".git").join("taskcode"), clon.join(".taskcode")] {
        recorrer(&raiz, &raiz, &mut h);
    }
    for (dir, refs) in [
        (
            clon,
            "refs/heads/develop refs/heads/main refs/remotes/origin/develop refs/remotes/origin/main refs/taskcode/",
        ),
        (bare, "refs/heads/develop refs/heads/main refs/taskcode/"),
    ] {
        let mut args = vec!["for-each-ref", "--format=%(refname) %(objectname)"];
        args.extend(refs.split(' '));
        h.insert(format!("refs@{}", dir.display()), f.crudo(dir, &args));
    }
    h.insert(
        "arbol-develop".to_owned(),
        f.crudo(bare, &["ls-tree", "-r", "develop"]),
    );
    h
}

fn recorrer(base: &Path, dir: &Path, h: &mut BTreeMap<String, String>) {
    let Ok(entradas) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entradas.flatten() {
        let p = e.path();
        if p.is_dir() {
            recorrer(base, &p, h);
        } else {
            let rel = p.strip_prefix(base).unwrap().display().to_string();
            h.insert(
                format!("{}:{rel}", base.display()),
                String::from_utf8_lossy(&std::fs::read(&p).unwrap()).into_owned(),
            );
        }
    }
}

/// Nombres de primer nivel de una carpeta.
fn nombres(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().into_string().unwrap())
        .collect();
    v.sort();
    v
}

/// Todas las refs del repo.
fn refs(f: &Fixture, dir: &Path) -> Vec<String> {
    f.crudo(dir, &["for-each-ref", "--format=%(refname)"])
        .lines()
        .map(str::to_owned)
        .collect()
}

#[test]
fn la_huella_de_la_016_queda_intacta_tras_new_y_start() {
    let f = Fixture::nueva("convivencia-016");
    let (bare, clon) = f.remoto_y_clon();
    for (rel, txt) in ARBOL_016 {
        escribir(&clon, rel, txt);
    }
    f.crudo(&clon, &["add", "."]);
    f.crudo(&clon, &["commit", "-q", "-m", "chore: tareas de la 0.16"]);
    f.crudo(&clon, &["push", "-q", "origin", "develop"]);
    f.crudo(&clon, &["fetch", "-q", "origin"]);
    let comun_016 = clon.join(".git").join("taskcode");
    for (rel, txt) in COMUN_016 {
        escribir(&comun_016, rel, txt);
    }

    let antes = huella_016(&f, &bare, &clon);
    let refs_antes = refs(&f, &clon);
    let git_antes = nombres(&clon.join(".git"));
    let raiz_antes = nombres(&clon);

    // `tc2 new` por el binario, como lo lanza la persona.
    let o = Command::new(env!("CARGO_BIN_EXE_tc2"))
        .current_dir(&clon)
        .args(["new", "Convivir con la 0.16"])
        .envs(f.entorno())
        .env("LC_ALL", "C")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("lanzar tc2");
    let out = String::from_utf8_lossy(&o.stdout);
    assert_eq!(
        o.status.code(),
        Some(0),
        "{out} {}",
        String::from_utf8_lossy(&o.stderr)
    );
    // El «start» de la 2.0: rama y worktree de tarea.
    let g = f.git();
    let i = git::crear_rama_y_worktree(
        &g,
        &clon,
        "TASK-001",
        "feature",
        "convivir",
        "origin/develop",
    );
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());

    // Una escritura de estado sin red deja su pendiente en las refs propias.
    let url = bare.to_str().unwrap().to_owned();
    f.crudo(
        &clon,
        &["remote", "set-url", "origin", "http://127.0.0.1:1/x.git"],
    );
    let mut c = Contexto::nuevo(&g, &clon, &catalogo);
    c.politica = Politica {
        intentos: 3,
        base: Duration::from_millis(1),
        espera_cerrojo: Duration::from_secs(30),
    };
    let i = estado::transaccion(&c, &Anotar::nueva("notas/a.md", false));
    assert_eq!(
        i.linea(),
        "estado sinred operacion=anotar pendiente=si pendientes=1"
    );
    f.crudo(&clon, &["remote", "set-url", "origin", &url]);
    assert_eq!(
        f.crudo(
            &clon,
            &["for-each-ref", "--format=%(refname)", "refs/taskcodetwo/"]
        ),
        "refs/taskcodetwo/pendientes/000001\n"
    );

    // Nada de la 0.16 ha cambiado.
    assert_eq!(huella_016(&f, &bare, &clon), antes);

    // Todo lo nuevo de la 2.0 lleva sus nombres.
    let rama_tarea = "refs/heads/feature/task-001-convivir";
    for r in refs(&f, &clon) {
        if refs_antes.contains(&r) {
            continue;
        }
        let propia = r.starts_with(huella::PREFIJO_REFS)
            || r == format!("refs/heads/{}", huella::RAMA_ESTADO)
            || r == format!("refs/remotes/origin/{}", huella::RAMA_ESTADO)
            || r == rama_tarea;
        assert!(propia, "ref nueva sin nombre de la 2.0: {r}");
    }
    let nuevas_git: Vec<String> = nombres(&clon.join(".git"))
        .into_iter()
        .filter(|n| !git_antes.contains(n))
        .collect();
    for n in &nuevas_git {
        // `worktrees` es de git (la crea `git worktree add`); lo demas, de la 2.0.
        assert!(
            n == huella::CARPETA_COMUN || n == "worktrees",
            "nuevo en .git sin nombre de la 2.0: {n} ({nuevas_git:?})"
        );
    }
    assert!(
        nombres(&clon.join(".git").join(huella::CARPETA_COMUN)).contains(&"estado.lock".to_owned()),
        "el cerrojo de la 2.0 vive en .git/taskcodetwo/"
    );
    let nuevas_raiz: Vec<String> = nombres(&clon)
        .into_iter()
        .filter(|n| !raiz_antes.contains(n))
        .collect();
    let carpeta_wt = huella::CARPETA_WORKTREES.split('/').next().unwrap();
    assert_eq!(nuevas_raiz, [carpeta_wt], "nuevo en la raiz del proyecto");
    assert!(
        clon.join(huella::CARPETA_WORKTREES)
            .join("TASK-001")
            .is_dir()
    );
    // `.taskcode/` sigue teniendo solo lo de la 0.16.
    assert_eq!(
        nombres(&clon.join(".taskcode")),
        ["config.yml", "doctor-deploy.yml"]
    );
}
