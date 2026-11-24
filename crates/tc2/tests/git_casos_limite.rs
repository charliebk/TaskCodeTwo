//! Casos limite de la revision de la ronda 1 (IMPORTANTE-1, IMPORTANTE-2,
//! MENOR-1 a MENOR-4), contra repos reales. Desde F2-3 un rechazo del
//! remoto es `denegada` (8) y no se reintenta.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::path::{Path, PathBuf};

use comun::{Fixture, contrato, escribir};
use tc2::contrato::Resultado;
use tc2::git;

/// Tarea TASK-007 abierta con su worktree; devuelve (bare, clon, worktree).
fn con_tarea(f: &Fixture) -> (PathBuf, PathBuf, PathBuf) {
    let (bare, clon) = f.remoto_y_clon();
    let i = git::crear_rama_y_worktree(
        &f.git(),
        &clon,
        "TASK-007",
        "feature",
        "login",
        "origin/develop",
    );
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    let wt = clon.join(".taskcodetwo/worktrees/TASK-007");
    (bare, clon, wt)
}

fn en_head(f: &Fixture, wt: &Path) -> String {
    f.crudo(
        wt,
        &["show", "--no-renames", "--name-status", "--format=", "HEAD"],
    )
}

#[test]
fn guardar_renombrados_y_borrados_ya_en_el_indice() {
    let f = Fixture::nueva("limite-guardar");
    let (_bare, _clon, wt) = con_tarea(&f);
    let g = f.git();
    escribir(&wt, "m.txt", "m\n");
    escribir(&wt, "z.txt", "z\n");
    f.crudo(&wt, &["add", "m.txt", "z.txt"]);
    f.crudo(&wt, &["commit", "-q", "-m", "base"]);

    // git mv: el commit lleva el renombrado entero, no medio.
    f.crudo(&wt, &["mv", "m.txt", "n.txt"]);
    let i = git::guardar(&g, &wt, "mv", &["m.txt", "n.txt"]);
    contrato(&i);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert_eq!(i.valor("sin-guardar"), Some("0"), "{}", i.linea());
    let h = en_head(&f, &wt);
    assert!(h.contains("D\tm.txt") && h.contains("A\tn.txt"), "{h}");

    // git rm: ya borrado en el indice y en disco.
    f.crudo(&wt, &["rm", "-q", "z.txt"]);
    let i = git::guardar(&g, &wt, "rm z", &["z.txt"]);
    contrato(&i);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert_eq!(en_head(&f, &wt).trim(), "D\tz.txt");
    assert_eq!(f.crudo(&wt, &["status", "--porcelain"]), "");
}

#[test]
fn guardar_respeta_el_indice_ajeno_y_acepta_mensajes_con_guiones() {
    let f = Fixture::nueva("limite-indice");
    let (_bare, _clon, wt) = con_tarea(&f);
    let g = f.git();
    escribir(&wt, "otro.txt", "ajeno\n");
    f.crudo(&wt, &["add", "otro.txt"]);
    escribir(&wt, "a.txt", "a\n");
    let i = git::guardar(&g, &wt, "--force no se usa", &["a.txt"]);
    contrato(&i);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert_eq!(en_head(&f, &wt).trim(), "A\ta.txt");
    assert_eq!(f.crudo(&wt, &["status", "--porcelain"]), "A  otro.txt\n");
    let mensaje = f.crudo(&wt, &["log", "-1", "--format=%s"]);
    assert_eq!(mensaje.trim(), "--force no se usa");

    escribir(&wt, "--force-x.txt", "x\n");
    let i = git::guardar(&g, &wt, "fichero raro", &["--force-x.txt"]);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
}

#[test]
fn quitar_worktree_con_head_suelto_no_pierde_commits() {
    let f = Fixture::nueva("limite-suelto");
    let (_bare, clon, wt) = con_tarea(&f);
    f.crudo(&wt, &["checkout", "-q", "--detach"]);
    escribir(&wt, "perdido.txt", "no publicar\n");
    f.crudo(&wt, &["add", "perdido.txt"]);
    f.crudo(&wt, &["commit", "-q", "-m", "suelto"]);
    let i = git::quitar_worktree(&f.git(), &clon, "TASK-007");
    contrato(&i);
    assert_eq!(
        i.linea(),
        "quitar-worktree rechazada id=TASK-007 motivo=sin-rama"
    );
    assert!(wt.join("perdido.txt").exists());
}

#[test]
fn rechazo_del_remoto_es_denegada_sin_reintento() {
    let f = Fixture::nueva("limite-hook");
    let (bare, _clon, wt) = con_tarea(&f);
    let g = f.git();
    escribir(&wt, "g.txt", "g\n");
    git::guardar(&g, &wt, "g", &["g.txt"]);
    let hook = bare.join("hooks").join("pre-receive");
    std::fs::write(&hook, "#!/bin/sh\necho protegida >&2\nexit 1\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let i = git::push(&g, &wt, &[std::time::Duration::ZERO; 2]);
    contrato(&i);
    assert_eq!(
        i.linea(),
        "push denegada rama=feature/task-007-login motivo=remoto-rechaza intentos=1"
    );
    assert_eq!(i.salida(false).codigo, 8);

    // Lo mismo en las otras operaciones que publican (adenda de F1-4).
    let i = git::tag_anotado(&g, &wt, "v9.9.9", "HEAD", &[std::time::Duration::ZERO; 2]);
    contrato(&i);
    assert_eq!(
        i.linea(),
        "tag-anotado denegada tag=v9.9.9 motivo=remoto-rechaza intentos=1"
    );
    std::fs::remove_file(&hook).unwrap();
    assert_eq!(git::push(&g, &wt, &[]).resultado, Resultado::Ok);
    std::fs::write(
        &hook,
        "#!/bin/sh
echo protegida >&2
exit 1
",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    // develop avanza en el remoto (sin pasar por el hook: escribe el bare).
    let otro = f.otro_clon(&bare, "otro");
    escribir(
        &otro, "base.txt", "base
",
    );
    f.crudo(&otro, &["add", "base.txt"]);
    f.crudo(&otro, &["commit", "-q", "-m", "base"]);
    let develop = f.crudo(&otro, &["rev-parse", "HEAD"]);
    f.crudo(
        &bare,
        &["fetch", "-q", otro.to_str().unwrap(), "develop:develop"],
    );
    assert_eq!(f.crudo(&bare, &["rev-parse", "develop"]), develop);
    let i = git::actualizar_rama(&g, &wt, &[std::time::Duration::ZERO; 2]);
    contrato(&i);
    // La linea y el diagnostico enteros (TASK-142, A4-04: el informe de un
    // publicar fallido es uno solo para push, tag-anotado y actualizar-rama).
    assert_eq!(
        i.linea(),
        "actualizar-rama denegada rama=feature/task-007-login motivo=remoto-rechaza intentos=1"
    );
    let stderr = i.salida(false).stderr;
    assert!(
        stderr.starts_with("push: [remote rejected] ")
            && stderr.contains("; la rama esta actualizada en local; "),
        "{stderr}"
    );
}

#[test]
fn actualizar_rama_trae_lo_publicado_en_la_rama_por_fast_forward() {
    let f = Fixture::nueva("limite-ff");
    let (bare, _clon, wt) = con_tarea(&f);
    let g = f.git();
    escribir(&wt, "g.txt", "g\n");
    git::guardar(&g, &wt, "g", &["g.txt"]);
    assert_eq!(git::push(&g, &wt, &[]).resultado, Resultado::Ok);
    let otro = f.otro_clon(&bare, "otro");
    f.crudo(&otro, &["checkout", "-q", "feature/task-007-login"]);
    escribir(&otro, "h.txt", "h\n");
    f.crudo(&otro, &["add", "h.txt"]);
    f.crudo(&otro, &["commit", "-q", "-m", "de otro"]);
    f.crudo(&otro, &["push", "-q", "origin", "feature/task-007-login"]);

    let i = git::actualizar_rama(&g, &wt, &[]);
    contrato(&i);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    let padres = f.crudo(&wt, &["rev-list", "--parents", "-n", "1", "HEAD"]);
    assert_eq!(
        padres.split_whitespace().count(),
        2,
        "fast-forward, sin merge: {padres}"
    );
    assert_eq!(
        f.crudo(&wt, &["rev-parse", "HEAD"]),
        f.crudo(&otro, &["rev-parse", "HEAD"])
    );
}

/// TASK-142 (A4-06): `crear-rama-y-worktree` y `reanudar` rechazan una
/// carpeta de worktree ya ocupada sin crear nada, con la ruta relativa en el
/// diagnostico.
#[test]
fn carpeta_de_worktree_ocupada_en_crear_y_reanudar() {
    let f = Fixture::nueva("limite-carpeta");
    let (bare, clon) = f.remoto_y_clon();
    let g = f.git();
    escribir(&clon, ".taskcodetwo/worktrees/TASK-009/a.txt", "a\n");
    escribir(&clon, ".taskcodetwo/worktrees/TASK-008/a.txt", "a\n");
    f.crudo(&clon, &["branch", "feature/task-008-x"]);
    let antes = f.huella(&bare, &clon);

    let i = git::crear_rama_y_worktree(&g, &clon, "TASK-009", "feature", "y", "origin/develop");
    contrato(&i);
    assert_eq!(
        i.linea(),
        "crear-rama-y-worktree rechazada rama=feature/task-009-y motivo=carpeta-ocupada"
    );
    let stderr = i.salida(false).stderr;
    assert!(
        stderr.contains(".taskcodetwo/worktrees/TASK-009"),
        "{stderr}"
    );

    let i = git::reanudar(&g, &clon, "TASK-008", "feature", "x");
    contrato(&i);
    assert_eq!(
        i.linea(),
        "reanudar rechazada rama=feature/task-008-x motivo=carpeta-ocupada"
    );
    let stderr = i.salida(false).stderr;
    assert!(
        stderr.contains(".taskcodetwo/worktrees/TASK-008"),
        "{stderr}"
    );
    assert_eq!(antes, f.huella(&bare, &clon), "no se creo nada");
}
