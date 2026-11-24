//! C2b, C3a y la parte de locale de C3b: `tc2 guardar` y
//! `tc2 actualizar-rama` lanzados como proceso, como los lanzaria el agente.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::path::Path;
use std::process::Output;

use comun::{Fixture, escribir, texto};
use tc2::contrato::Resultado;
use tc2::git;

fn tc2(f: &Fixture, dir: &Path, args: &[&str], locale: Option<&str>) -> Output {
    let mut c = comun::orden_tc2();
    c.current_dir(dir).args(args).envs(f.entorno());
    // Sin locale fijado, el idioma no puede venir del runner (macOS trae
    // LANG=en_US): se quitan las variables que lee `main.rs` y queda espanol.
    for var in ["TC2_IDIOMA", "LC_ALL", "LC_MESSAGES", "LANG"] {
        c.env_remove(var);
    }
    if let Some(l) = locale {
        c.env("LANG", l).env("LC_ALL", l);
    }
    c.output().expect("lanzar tc2")
}

#[test]
fn guardar_y_actualizar_rama_por_el_binario_cumplen_el_contrato() {
    let f = Fixture::nueva("binario");
    let (_bare, clon) = f.remoto_y_clon();
    let g = f.git();
    let i = git::crear_rama_y_worktree(&g, &clon, "TASK-010", "feature", "bin", "origin/develop");
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    let wt = clon.join(".taskcodetwo/worktrees/TASK-010");

    // RF-23: con otros cambios en el worktree, solo se guardan las rutas dadas.
    escribir(&wt, "a.txt", "a\n");
    escribir(&wt, "b.txt", "b\n");
    escribir(&wt, "README.md", "cambiado\n");
    let o = tc2(&f, &wt, &["guardar", "feat(TASK-010): a", "a.txt"], None);
    let out = texto(&o.stdout);
    assert_eq!(o.status.code(), Some(0), "{out} {}", texto(&o.stderr));
    assert!(o.stderr.is_empty());
    assert_eq!(out.lines().count(), 1);
    assert!(out.starts_with("guardar ok commit="), "{out}");
    assert!(out.ends_with(" ficheros=1 sin-guardar=2\n"), "{out}");
    let en_commit = f.crudo(&wt, &["show", "--name-only", "--format=", "HEAD"]);
    assert_eq!(en_commit.trim(), "a.txt");

    // --json con la misma informacion.
    let o = tc2(
        &f,
        &wt,
        &["guardar", "--json", "feat(TASK-010): b", "b.txt"],
        None,
    );
    let out = texto(&o.stdout);
    assert_eq!(o.status.code(), Some(0));
    assert!(
        out.starts_with("{\"comando\":\"guardar\",\"resultado\":\"ok\",\"codigo\":0,"),
        "{out}"
    );
    assert!(
        out.contains("\"ficheros\":\"1\",\"sin-guardar\":\"1\""),
        "{out}"
    );

    // Salida identica con un locale traducido en el entorno padre.
    f.crudo(&wt, &["checkout", "-q", "--", "README.md"]);
    let c = tc2(&f, &wt, &["actualizar-rama"], None);
    let es = tc2(&f, &wt, &["actualizar-rama"], Some("es_ES.UTF-8"));
    assert_eq!(c.status.code(), Some(0), "{}", texto(&c.stderr));
    assert_eq!(
        texto(&c.stdout),
        "actualizar-rama sin-cambios base=origin/develop\n"
    );
    assert_eq!(c.stdout, es.stdout);
    assert_eq!(c.status.code(), es.status.code());
    assert!(c.stderr.is_empty() && es.stderr.is_empty());

    // Uso: sin rutas, codigo 2 y la forma de llamarlo en stderr.
    let o = tc2(&f, &wt, &["guardar", "mensaje"], None);
    assert_eq!(o.status.code(), Some(2));
    assert_eq!(texto(&o.stdout), "guardar uso motivo=sin-rutas\n");
    assert_eq!(texto(&o.stderr), "uso: guardar \"<mensaje>\" <rutas...>\n");
}
