//! C1c: `push` reintenta un fallo transitorio y no un non-fast-forward;
//! `tag-anotado` sobre otro commit no publica nada; sin remoto, `sinred`.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::time::Duration;

use comun::{Fixture, contrato, escribir, registro_permitido};
use tc2::contrato::Resultado;
use tc2::git;

const SIN_ESPERA: [Duration; 2] = [Duration::ZERO, Duration::ZERO];

/// Hook del remoto que rechaza el primer push y acepta los siguientes.
fn hook_que_falla_una_vez(bare: &std::path::Path) {
    let hook = bare.join("hooks").join("pre-receive");
    std::fs::write(
        &hook,
        "#!/bin/sh\nif [ -f fallado ]; then exit 0; fi\ntouch fallado\necho fallo transitorio >&2\nexit 1\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}

#[test]
fn push_reintenta_lo_transitorio_y_rechaza_lo_que_no_procede() {
    let f = Fixture::nueva("push");
    let (bare, clon) = f.remoto_y_clon();
    let g = f.git();
    let wt = clon.join(".taskcode/worktrees/TASK-007");
    let rama = "feature/task-007-login";

    git::crear_rama_y_worktree(&g, &clon, "TASK-007", "feature", "login", "origin/develop");
    escribir(&wt, "g.txt", "login\n");
    git::guardar(&g, &wt, "feat(TASK-007): login", &["g.txt"]);

    // Fallo transitorio: el segundo intento entra.
    hook_que_falla_una_vez(&bare);
    let i = git::push(&g, &wt, &SIN_ESPERA);
    contrato(&i);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert_eq!(i.valor("intentos"), Some("2"));

    // Otra persona publica en la misma rama; el agente guarda y publica encima.
    let otro = f.otro_clon(&bare, "otro");
    f.crudo(&otro, &["checkout", "-q", rama]);
    escribir(&otro, "h.txt", "de otro\n");
    f.crudo(&otro, &["add", "h.txt"]);
    f.crudo(&otro, &["commit", "-q", "-m", "otro"]);
    f.crudo(&otro, &["push", "-q", "origin", rama]);
    escribir(&wt, "i.txt", "mio\n");
    git::guardar(&g, &wt, "feat(TASK-007): mas", &["i.txt"]);
    let antes = g
        .registro()
        .iter()
        .filter(|l| l.starts_with("git push"))
        .count();
    let i = git::push(&g, &wt, &SIN_ESPERA);
    contrato(&i);
    assert_eq!(
        i.linea(),
        format!("push rechazada rama={rama} motivo=no-fast-forward")
    );
    assert_eq!(i.salida(false).codigo, 3);
    assert_eq!(i.salida(false).stderr, "lanza actualizar-rama\n");
    let despues = g
        .registro()
        .iter()
        .filter(|l| l.starts_with("git push"))
        .count();
    assert_eq!(despues - antes, 1, "un non-fast-forward no se reintenta");

    // Tags sobre otro commit: en remoto (sin traerlo) y solo en local.
    f.crudo(
        &otro,
        &["tag", "-a", "-m", "v0.2.0", "v0.2.0", "origin/develop"],
    );
    f.crudo(&otro, &["push", "-q", "origin", "v0.2.0"]);
    let i = git::tag_anotado(&g, &clon, "v0.2.0", rama, &SIN_ESPERA);
    contrato(&i);
    assert_eq!(
        i.linea(),
        "tag-anotado rechazada tag=v0.2.0 motivo=existe-en-otro-commit donde=remoto"
    );
    assert_eq!(i.salida(false).codigo, 3);
    f.crudo(&clon, &["tag", "v0.0.9", "origin/develop"]);
    let i = git::tag_anotado(&g, &clon, "v0.0.9", rama, &SIN_ESPERA);
    contrato(&i);
    assert_eq!(
        i.linea(),
        "tag-anotado rechazada tag=v0.0.9 motivo=existe-en-otro-commit donde=local"
    );
    assert_eq!(
        f.crudo(&bare, &["tag", "--list", "v0.0.9"]),
        "",
        "no se publico"
    );

    // Sin remoto alcanzable.
    f.crudo(
        &clon,
        &[
            "remote",
            "set-url",
            "origin",
            f.raiz.join("no-existe").to_str().unwrap(),
        ],
    );
    let i = git::fetch(&g, &clon);
    contrato(&i);
    assert_eq!(i.linea(), "fetch sinred remoto=origin");
    assert_eq!(i.salida(false).codigo, 4);
    let i = git::push(&g, &wt, &SIN_ESPERA);
    contrato(&i);
    assert_eq!(i.resultado, Resultado::Sinred, "{}", i.linea());
    assert_eq!(i.valor("intentos"), Some("3"));

    registro_permitido(&g.registro());
}
