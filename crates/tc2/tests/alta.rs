//! F2-4: `tc2 new` reserva el ID con el push de la rama de estado. E2E con
//! el binario real, como lo lanzaria la persona o el agente (pesados: lanzan
//! procesos y esperan con la politica por defecto).

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::collections::BTreeSet;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::{Arc, Barrier};

use comun::Fixture;

const RAMA: &str = "taskcode/tareas";

fn tc2(f: &Fixture, dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tc2"))
        .current_dir(dir)
        .args(args)
        .envs(f.entorno())
        .env("LC_ALL", "C")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("lanzar tc2")
}

fn texto(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

/// Carpetas `TASK-*` publicadas en el remoto y el titulo de cada una.
fn publicadas(f: &Fixture, bare: &Path) -> Vec<(String, String)> {
    let arbol = f.crudo(bare, &["ls-tree", "--name-only", RAMA]);
    arbol
        .lines()
        .filter(|l| l.starts_with("TASK-"))
        .map(|carpeta| {
            let md = f.crudo(bare, &["show", &format!("{RAMA}:{carpeta}/tarea.md")]);
            assert!(
                md.contains(&format!("\nid: \"{carpeta}\"\n")),
                "id distinto de su carpeta en {carpeta}: {md}"
            );
            let titulo = md
                .lines()
                .find_map(|l| l.strip_prefix("titulo: \""))
                .and_then(|t| t.strip_suffix('"'))
                .unwrap()
                .to_owned();
            (carpeta.to_owned(), titulo)
        })
        .collect()
}

/// C1: dos clones crean tareas a la vez y todos los IDs salen distintos.
#[test]
fn dos_clones_crean_a_la_vez_con_ids_distintos() {
    const POR_CLON: usize = 3;
    let f = Arc::new(Fixture::nueva("alta-paralelo"));
    let (bare, a) = f.remoto_y_clon();
    let b = f.otro_clon(&bare, "clon-b");
    let barrera = Arc::new(Barrier::new(2));
    let hilos: Vec<_> = [("a", a), ("b", b)]
        .into_iter()
        .map(|(etiqueta, dir)| {
            let f = Arc::clone(&f);
            let barrera = Arc::clone(&barrera);
            std::thread::spawn(move || {
                barrera.wait();
                let mut ids = Vec::new();
                for i in 0..POR_CLON {
                    let titulo = format!("t-{etiqueta}-{i}");
                    // `agotada` (5) es «reintenta, nada escrito»: se relanza.
                    let mut intentos = 0;
                    loop {
                        intentos += 1;
                        let o = tc2(&f, &dir, &["new", &titulo]);
                        let out = texto(&o.stdout);
                        if o.status.code() == Some(5) && intentos < 10 {
                            continue;
                        }
                        assert_eq!(o.status.code(), Some(0), "{out} {}", texto(&o.stderr));
                        let id = out
                            .split(' ')
                            .find_map(|kv| kv.strip_prefix("id="))
                            .unwrap()
                            .to_owned();
                        ids.push((id, titulo));
                        break;
                    }
                }
                ids
            })
        })
        .collect();
    let mut dichos: Vec<(String, String)> =
        hilos.into_iter().flat_map(|h| h.join().unwrap()).collect();
    dichos.sort();

    let total = 2 * POR_CLON;
    let mut en_remoto = publicadas(&f, &bare);
    en_remoto.sort();
    let esperadas: Vec<String> = (1..=total).map(|n| format!("TASK-{n:03}")).collect();
    let carpetas: Vec<String> = en_remoto.iter().map(|(c, _)| c.clone()).collect();
    assert_eq!(carpetas, esperadas, "IDs no consecutivos o repetidos");
    // Lo que dijo cada `new` es lo que quedo publicado.
    assert_eq!(dichos, en_remoto);
    let titulos: BTreeSet<&String> = en_remoto.iter().map(|(_, t)| t).collect();
    assert_eq!(titulos.len(), total, "un titulo perdido o duplicado");
    // Historia lineal y nada pendiente ni apartado.
    let merges = f.crudo(&bare, &["rev-list", "--merges", RAMA]);
    assert!(merges.trim().is_empty(), "merges en {RAMA}: {merges}");
    for dir in [f.raiz.join("clon"), f.raiz.join("clon-b")] {
        let refs = f.crudo(&dir, &["for-each-ref", "refs/taskcode/"]);
        assert!(refs.trim().is_empty(), "refs propias: {refs}");
    }
}

/// C2a: el ID se calcula sobre el estado fresco del remoto, no sobre lo que
/// el clon tenia: B no ha hecho `fetch` y aun asi recibe el siguiente.
#[test]
fn el_segundo_clon_recibe_el_siguiente_id_sin_fetch_previo() {
    let f = Fixture::nueva("alta-secuencial");
    let (bare, a) = f.remoto_y_clon();
    let b = f.otro_clon(&bare, "clon-b");
    let o = tc2(&f, &a, &["new", "Primera"]);
    let out = texto(&o.stdout);
    assert_eq!(o.status.code(), Some(0), "{out} {}", texto(&o.stderr));
    assert!(out.starts_with("new ok id=TASK-001 "), "{out}");
    assert!(o.stderr.is_empty());

    let o = tc2(&f, &b, &["new", "--json", "Segunda \"con comillas\""]);
    let out = texto(&o.stdout);
    assert_eq!(o.status.code(), Some(0), "{out} {}", texto(&o.stderr));
    assert!(
        out.starts_with("{\"comando\":\"new\",\"resultado\":\"ok\""),
        "{out}"
    );
    assert!(out.contains("\"id\":\"TASK-002\""), "{out}");
    assert_eq!(
        publicadas(&f, &bare),
        [
            ("TASK-001".to_owned(), "Primera".to_owned()),
            (
                "TASK-002".to_owned(),
                "Segunda \\\"con comillas\\\"".to_owned()
            ),
        ]
    );
}

/// Valor de `clave=` en una linea de contrato.
fn valor<'a>(linea: &'a str, clave: &str) -> Option<&'a str> {
    linea
        .split(' ')
        .find_map(|kv| kv.strip_prefix(clave)?.strip_prefix('='))
}

/// TASK-076 C2: una rama de tarea que solo esta en el remoto (sin carpeta)
/// ocupa su numero: el siguiente `new` da un ID mayor.
#[test]
fn rama_solo_en_el_remoto_ocupa_su_id() {
    let f = Fixture::nueva("alta-rama-remota");
    let (bare, a) = f.remoto_y_clon();
    let b = f.otro_clon(&bare, "clon-b");
    // Otra persona publica la rama sin pasar por `new` (ni carpeta).
    f.crudo(&b, &["branch", "feature/task-005-x"]);
    f.crudo(&b, &["push", "-q", "origin", "feature/task-005-x"]);
    let o = tc2(&f, &a, &["new", "Despues de la rama"]);
    let out = texto(&o.stdout);
    assert_eq!(o.status.code(), Some(0), "{out} {}", texto(&o.stderr));
    assert_eq!(valor(&out, "id"), Some("TASK-006"), "{out}");
    assert_eq!(
        valor(&out, "rama"),
        Some("feature/task-006-despues-de-la-rama"),
        "{out}"
    );
    let md = f.crudo(&bare, &["show", &format!("{RAMA}:TASK-006/tarea.md")]);
    assert!(
        md.contains("\nrama: \"feature/task-006-despues-de-la-rama\"\n"),
        "{md}"
    );
    assert!(
        !f.crudo(&bare, &["ls-tree", "--name-only", RAMA])
            .contains("TASK-005")
    );
}

/// TASK-076 C3: lo mismo con una rama `*/task-NNN-*` que solo existe en
/// local (tambien en un repo sin remoto, que solo mira la local).
#[test]
fn rama_solo_en_local_ocupa_su_id() {
    let f = Fixture::nueva("alta-rama-local");
    let (_bare, a) = f.remoto_y_clon();
    f.crudo(&a, &["branch", "fix/task-009-y"]);
    let o = tc2(&f, &a, &["new", "Tras la local"]);
    let out = texto(&o.stdout);
    assert_eq!(o.status.code(), Some(0), "{out} {}", texto(&o.stderr));
    assert_eq!(valor(&out, "id"), Some("TASK-010"), "{out}");

    // Sin remoto: la rama local tambien cuenta.
    let solo = f.raiz.join("solo");
    f.crudo(&f.raiz, &["init", "-q", "-b", "develop", "solo"]);
    f.crudo(&solo, &["commit", "-q", "--allow-empty", "-m", "inicial"]);
    f.crudo(&solo, &["branch", "a/b/task-020-z"]);
    let o = tc2(&f, &solo, &["new", "Sin remoto"]);
    let out = texto(&o.stdout);
    assert_eq!(o.status.code(), Some(0), "{out} {}", texto(&o.stderr));
    assert_eq!(valor(&out, "id"), Some("TASK-021"), "{out}");
}

/// TASK-076 C6: si el remoto no responde, el `ls-remote` previo sale
/// `sinred` (4) antes de la transaccion (sin `fetch` ni `push`) y no deja
/// refs; un remoto que no existe es `denegada` (8), como en `fetch`.
#[test]
fn ls_remote_previo_sin_red_o_denegado_no_deja_nada() {
    let f = Fixture::nueva("alta-lsremote");
    let (_bare, a) = f.remoto_y_clon();
    let antes = f.crudo(&a, &["for-each-ref", "--format=%(refname) %(objectname)"]);
    f.crudo(
        &a,
        &[
            "remote",
            "set-url",
            "origin",
            "http://127.0.0.1:1/remoto.git",
        ],
    );
    let g = f.git();
    let i = tc2::git::ids_con_rama(&g, &a, "new").expect_err("sin red");
    assert_eq!(
        i.resultado,
        tc2::contrato::Resultado::Sinred,
        "{}",
        i.linea()
    );
    let remotas: Vec<String> = g
        .registro()
        .into_iter()
        .filter(|l| l.starts_with("git fetch") || l.starts_with("git push"))
        .collect();
    assert!(remotas.is_empty(), "{remotas:?}");
    let no_existe = f.raiz.join("no-existe.git");
    f.crudo(
        &a,
        &["remote", "set-url", "origin", no_existe.to_str().unwrap()],
    );
    let o = tc2(&f, &a, &["new", "Remoto que no existe"]);
    let out = texto(&o.stdout);
    assert_eq!(o.status.code(), Some(8), "{out} {}", texto(&o.stderr));
    assert!(out.starts_with("new denegada "), "{out}");
    assert_eq!(valor(&out, "operacion"), None, "{out}");
    let despues = f.crudo(&a, &["for-each-ref", "--format=%(refname) %(objectname)"]);
    assert_eq!(antes, despues, "new dejo refs");
}

/// C2b: sin red, `new` no reserva nada: `sinred pendiente=no` (4), sin
/// commit local ni operacion pendiente (D2b de F1-2). Desde F2-4b lo corta
/// el `ls-remote` previo: la linea no lleva `operacion=` (TASK-076 C6).
#[test]
fn sin_red_falla_sin_dejar_nada() {
    let f = Fixture::nueva("alta-sinred");
    let (_bare, a) = f.remoto_y_clon();
    f.crudo(
        &a,
        &[
            "remote",
            "set-url",
            "origin",
            "http://127.0.0.1:1/remoto.git",
        ],
    );
    let antes = f.crudo(&a, &["for-each-ref", "--format=%(refname) %(objectname)"]);
    let o = tc2(&f, &a, &["new", "Sin red"]);
    let out = texto(&o.stdout);
    assert_eq!(o.status.code(), Some(4), "{out} {}", texto(&o.stderr));
    assert!(out.starts_with("new sinred "), "{out}");
    assert!(out.contains(" pendiente=no"), "{out}");
    assert_eq!(valor(&out, "operacion"), None, "{out}");
    assert_eq!(valor(&out, "remoto"), Some("origin"), "{out}");
    let despues = f.crudo(&a, &["for-each-ref", "--format=%(refname) %(objectname)"]);
    assert_eq!(antes, despues, "new sin red dejo refs");
}
