//! C1a y C3b: ciclo completo de una tarea contra repos reales, cada paso dos
//! veces (idempotencia), con el contrato de salida en cada linea y el
//! registro de llamadas a git contra la lista permitida.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::time::Duration;

use comun::{Fixture, dos_veces, escribir, registro_permitido};
use tc2::contrato::Resultado;
use tc2::git;

const SIN_ESPERA: [Duration; 2] = [Duration::ZERO, Duration::ZERO];

#[test]
fn ciclo_de_una_tarea_es_idempotente_y_solo_usa_la_lista_permitida() {
    let f = Fixture::nueva("ciclo");
    let (bare, clon) = f.remoto_y_clon();
    let g = f.git();
    let wt = clon.join(".taskcodetwo/worktrees/TASK-007");

    let i = dos_veces(&f, &bare, &clon, || git::fetch(&g, &clon));
    assert_eq!(i.linea(), "fetch sin-cambios remoto=origin");

    let i = dos_veces(&f, &bare, &clon, || {
        git::validar_base(&g, &clon, "origin/develop")
    });
    assert_eq!(i.resultado, Resultado::Ok);

    let i = dos_veces(&f, &bare, &clon, || {
        git::comprobar_id(&g, &clon, "TASK-007")
    });
    assert_eq!(i.linea(), "comprobar-id ok id=TASK-007");

    let i = dos_veces(&f, &bare, &clon, || {
        git::crear_rama_y_worktree(&g, &clon, "TASK-007", "feature", "login", "origin/develop")
    });
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert!(i.linea().starts_with(
        "crear-rama-y-worktree ok rama=feature/task-007-login worktree=.taskcodetwo/worktrees/TASK-007 base="
    ));
    // El principal no ve los worktrees de tarea como ficheros sin seguimiento.
    assert_eq!(f.crudo(&clon, &["status", "--porcelain"]), "");

    escribir(&wt, "g.txt", "login\n");
    let i = dos_veces(&f, &bare, &clon, || {
        git::guardar(&g, &wt, "feat(TASK-007): login", &["g.txt"])
    });
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert_eq!(i.valor("ficheros"), Some("1"));
    assert_eq!(i.valor("sin-guardar"), Some("0"));

    let i = dos_veces(&f, &bare, &clon, || git::diff_tarea(&g, &wt));
    assert!(
        i.linea().ends_with("ficheros=1 mas=1 menos=0"),
        "{}",
        i.linea()
    );

    let i = dos_veces(&f, &bare, &clon, || git::push(&g, &wt, &SIN_ESPERA));
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert_eq!(i.valor("intentos"), Some("1"));

    let i = dos_veces(&f, &bare, &clon, || {
        git::comprobar_id(&g, &clon, "TASK-007")
    });
    assert_eq!(
        i.linea(),
        "comprobar-id rechazada id=TASK-007 motivo=ocupada ramas=feature/task-007-login"
    );

    let i = dos_veces(&f, &bare, &clon, || {
        git::tag_anotado(&g, &clon, "v0.1.0", "origin/main", &SIN_ESPERA)
    });
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert!(f.crudo(&bare, &["tag", "--list"]).contains("v0.1.0"));

    let i = dos_veces(&f, &bare, &clon, || {
        git::quitar_worktree(&g, &clon, "TASK-007")
    });
    assert_eq!(
        i.linea(),
        "quitar-worktree ok id=TASK-007 rama=feature/task-007-login"
    );
    assert!(!wt.exists());
    // La rama se conserva.
    assert!(
        f.crudo(&clon, &["branch", "--list"])
            .contains("feature/task-007-login")
    );

    let i = dos_veces(&f, &bare, &clon, || {
        git::reanudar(&g, &clon, "TASK-007", "feature", "login")
    });
    assert_eq!(
        i.linea(),
        "reanudar ok rama=feature/task-007-login worktree=.taskcodetwo/worktrees/TASK-007 creado=worktree"
    );
    assert!(wt.join("g.txt").exists());

    let i = dos_veces(&f, &bare, &clon, || {
        git::actualizar_principal(&g, &clon, "develop")
    });
    assert_eq!(i.linea(), "actualizar-principal sin-cambios rama=develop");

    registro_permitido(&g.registro());
}
