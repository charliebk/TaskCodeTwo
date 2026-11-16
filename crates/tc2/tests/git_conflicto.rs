//! C1b: `actualizar-rama` con un conflicto provocado. Codigo 6 sin rebase ni
//! force; repetir no vuelve a mergear; `guardar` no cierra el merge con otra
//! ruta ni con marcas; al resolver, merge commit con dos padres y publicado.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use comun::{Fixture, contrato, dos_veces, escribir, registro_permitido};
use tc2::contrato::Resultado;
use tc2::git;

#[test]
fn actualizar_rama_con_conflicto_se_resuelve_con_guardar() {
    let f = Fixture::nueva("conflicto");
    let (bare, clon) = f.remoto_y_clon();
    let g = f.git();
    let wt = clon.join(".taskcodetwo/worktrees/TASK-008");

    let i = git::crear_rama_y_worktree(&g, &clon, "TASK-008", "feature", "texto", "origin/develop");
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    escribir(&wt, "README.md", "linea 1 de TASK-008\n");
    assert_eq!(
        git::guardar(&g, &wt, "feat(TASK-008): texto", &["README.md"]).resultado,
        Resultado::Ok
    );
    assert_eq!(git::push(&g, &wt, &[]).resultado, Resultado::Ok);

    // Otra persona integra en develop un cambio en la misma linea.
    let otro = f.otro_clon(&bare, "otro");
    escribir(&otro, "README.md", "linea 1 de otro PR\n");
    f.crudo(&otro, &["commit", "-q", "-am", "otro PR"]);
    f.crudo(&otro, &["push", "-q", "origin", "develop"]);

    let i = dos_veces(&f, &bare, &clon, || git::actualizar_rama(&g, &wt, &[]));
    assert_eq!(
        i.linea(),
        "actualizar-rama conflicto base=origin/develop ficheros=README.md"
    );
    assert_eq!(i.salida(false).codigo, 6);
    assert_eq!(
        i.salida(false).stderr,
        "resuelve README.md y lanza guardar\n"
    );

    escribir(&wt, "nota.txt", "otra cosa\n");
    let i = git::guardar(&g, &wt, "otra cosa", &["nota.txt"]);
    contrato(&i);
    assert_eq!(
        i.linea(),
        "guardar rechazada motivo=conflicto-sin-resolver ficheros=README.md"
    );

    let i = git::guardar(&g, &wt, "resolver", &["README.md"]);
    contrato(&i);
    assert_eq!(
        i.linea(),
        "guardar rechazada motivo=marcas-de-conflicto ficheros=README.md"
    );

    escribir(&wt, "README.md", "linea 1 de TASK-008 y de otro PR\n");
    let i = git::guardar(
        &g,
        &wt,
        "merge(TASK-008): develop en la rama",
        &["README.md"],
    );
    contrato(&i);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert_eq!(i.valor("tipo"), Some("merge"));
    // nota.txt no entra: solo las rutas dadas.
    assert_eq!(i.valor("sin-guardar"), Some("1"));
    let padres = f.crudo(&wt, &["rev-list", "--parents", "-n", "1", "HEAD"]);
    assert_eq!(
        padres.split_whitespace().count(),
        3,
        "merge con dos padres: {padres}"
    );

    // Ya resuelto: la base esta contenida y publica la rama.
    std::fs::remove_file(wt.join("nota.txt")).unwrap();
    let i = git::actualizar_rama(&g, &wt, &[]);
    contrato(&i);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert_eq!(i.valor("publicada"), Some("si"));
    let i = dos_veces(&f, &bare, &clon, || git::actualizar_rama(&g, &wt, &[]));
    assert_eq!(i.linea(), "actualizar-rama sin-cambios base=origin/develop");

    let local = f.crudo(&wt, &["rev-parse", "HEAD"]);
    let remota = f.crudo(&bare, &["rev-parse", "feature/task-008-texto"]);
    assert_eq!(local, remota);
    registro_permitido(&g.registro());
}
