//! C1 de F2-3: dos procesos escriben la rama de estado a la vez sin perder
//! nada (pesados: lanzan procesos reales).
//!
//! Los procesos hijos son este mismo ejecutable de tests reinvocado con
//! `hijo_escritor` y unas variables de entorno: sin dependencias y sin
//! ampliar el binario `tc2` (D1a de F1-4). Arrancan a la vez con una barrera
//! de fichero.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use comun::Fixture;
use comun::ops::{Anotar, catalogo, roadmap};
use tc2::alta::Alta;
use tc2::contrato::Resultado;
use tc2::estado::{self, Contexto, Operacion, Politica, Roadmap};
use tc2::git::Git;
use tc2::plataforma::Personas;

const DIR: &str = "TC2_HIJO_DIR";
const MODO: &str = "TC2_HIJO_MODO";
const ETIQUETA: &str = "TC2_HIJO_ETIQUETA";
const K: &str = "TC2_HIJO_K";
const BARRERA: &str = "TC2_HIJO_BARRERA";

/// Escrituras por proceso.
const ESCRITURAS: usize = 5;

/// Proceso hijo: no hace nada salvo que lo lance un test de este fichero.
#[test]
fn hijo_escritor() {
    let Ok(dir) = std::env::var(DIR) else {
        return;
    };
    let etiqueta = std::env::var(ETIQUETA).unwrap();
    let k: usize = std::env::var(K).unwrap().parse().unwrap();
    let barrera = std::env::var(BARRERA).unwrap();
    let limite = Instant::now() + Duration::from_secs(60);
    while !Path::new(&barrera).exists() {
        assert!(Instant::now() < limite, "la barrera no llego");
        std::thread::sleep(Duration::from_millis(5));
    }
    // El entorno (configuracion aislada) lo hereda del padre.
    let git = Git::nuevo();
    let dir = Path::new(&dir);
    // `alta`: altas con el generador de produccion (C1 de TASK-031).
    let alta = std::env::var(MODO).is_ok_and(|m| m == "alta");
    let personas = Personas::default();
    let real = tc2::roadmap::generador(&personas);
    let generar: Roadmap<'_> = if alta { &real } else { &roadmap };
    let mut c = Contexto::nuevo(&git, dir, &catalogo, Some(generar));
    c.politica = Politica {
        intentos: 40,
        base: Duration::from_millis(15),
        espera_cerrojo: Duration::from_secs(120),
    };
    for i in 0..k {
        let op: Box<dyn Operacion> = if alta {
            Box::new(Alta::nueva(&format!("t-{etiqueta}-{i}")))
        } else {
            Box::new(Anotar::nueva(&format!("notas/{etiqueta}-{i}.md"), true))
        };
        let inf = estado::transaccion(&c, op.as_ref());
        println!("RESULTADO {} {:?}", inf.linea(), inf.diagnostico);
        assert_eq!(inf.resultado, Resultado::Ok, "{}", inf.linea());
    }
}

fn lanzar(f: &Fixture, dir: &Path, etiqueta: &str, barrera: &Path, modo: &str) -> Child {
    Command::new(std::env::current_exe().unwrap())
        .args([
            "hijo_escritor",
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ])
        .envs(f.entorno())
        .env(DIR, dir)
        .env(ETIQUETA, etiqueta)
        .env(K, ESCRITURAS.to_string())
        .env(BARRERA, barrera)
        .env(MODO, modo)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

/// Lanza un hijo por carpeta, los suelta a la vez y comprueba que cada uno
/// publico sus escrituras.
fn a_la_vez(f: &Fixture, dirs: &[(&Path, &str)]) {
    a_la_vez_en_modo(f, dirs, "anotar");
}

fn a_la_vez_en_modo(f: &Fixture, dirs: &[(&Path, &str)], modo: &str) {
    let barrera = f.raiz.join("barrera");
    let hijos: Vec<(String, Child)> = dirs
        .iter()
        .map(|(d, e)| ((*e).to_owned(), lanzar(f, d, e, &barrera, modo)))
        .collect();
    std::fs::write(&barrera, "ya").unwrap();
    for (e, h) in hijos {
        let o = h.wait_with_output().unwrap();
        let out = String::from_utf8_lossy(&o.stdout);
        assert!(
            o.status.success(),
            "hijo {e}: {out}\n{}",
            String::from_utf8_lossy(&o.stderr)
        );
        // La primera linea lleva delante el nombre del test del arnes.
        let oks = out.matches("RESULTADO estado ok ").count();
        assert_eq!(oks, ESCRITURAS, "hijo {e}: {out}");
    }
}

/// Lo que hay en `rama` de `repo`: notas, lineas del roadmap, commits y
/// merges.
fn contenido(f: &Fixture, repo: &Path, rama: &str) -> (Vec<String>, usize, usize, usize) {
    let notas: Vec<String> = f
        .crudo(repo, &["ls-tree", "-r", "--name-only", rama, "notas/"])
        .lines()
        .map(str::to_owned)
        .collect();
    let roadmap = f.crudo(repo, &["show", &format!("{rama}:ROADMAP.md")]);
    let entradas = roadmap
        .lines()
        .filter(|l| l.starts_with("- notas/"))
        .count();
    let commits = f
        .crudo(repo, &["rev-list", "--count", rama])
        .trim()
        .parse()
        .unwrap();
    let merges = f
        .crudo(repo, &["rev-list", "--count", "--merges", rama])
        .trim()
        .parse()
        .unwrap();
    (notas, entradas, commits, merges)
}

fn esperadas(etiquetas: &[&str]) -> Vec<String> {
    let mut v: Vec<String> = etiquetas
        .iter()
        .flat_map(|e| (0..ESCRITURAS).map(move |i| format!("notas/{e}-{i}.md")))
        .collect();
    v.sort();
    v
}

#[test]
fn dos_clones_escriben_a_la_vez_sin_perder_nada() {
    let f = Fixture::nueva("estado-dos-clones");
    let (bare, clon) = f.remoto_y_clon();
    let otro = f.otro_clon(&bare, "otro");
    let g = f.git();
    assert!(estado::montar(&g, &clon).resultado.es_exito());
    assert!(estado::montar(&g, &otro).resultado.es_exito());

    a_la_vez(&f, &[(&clon, "uno"), (&otro, "dos")]);

    let (notas, entradas, commits, merges) = contenido(&f, &bare, estado::RAMA);
    assert_eq!(notas, esperadas(&["dos", "uno"]));
    assert_eq!(entradas, 2 * ESCRITURAS, "ROADMAP.md regenerado con todas");
    assert_eq!(commits, 1 + 2 * ESCRITURAS, "raiz + una por escritura");
    assert_eq!(merges, 0, "historia lineal");
    for c in [&clon, &otro] {
        assert_eq!(
            f.crudo(c, &["for-each-ref", "refs/taskcodetwo/"]),
            "",
            "nada pendiente ni temporal"
        );
        assert_eq!(f.crudo(&c.join("tareas"), &["status", "--porcelain"]), "");
        assert_eq!(f.crudo(c, &["status", "--porcelain"]), "");
    }
}

/// C1 de TASK-031: dos clones dan de alta a la vez con el generador de
/// produccion; los dos acaban `ok`, el ROADMAP.md publicado tiene todas las
/// tareas, la historia no tiene merges y `--comparar` lo da al dia.
#[test]
fn dos_clones_dan_de_alta_a_la_vez_y_el_roadmap_queda_al_dia() {
    let f = Fixture::nueva("estado-roadmap-real");
    let (bare, clon) = f.remoto_y_clon();
    let otro = f.otro_clon(&bare, "otro");

    a_la_vez_en_modo(&f, &[(&clon, "uno"), (&otro, "dos")], "alta");

    let roadmap = f.crudo(&bare, &["show", &format!("{}:ROADMAP.md", estado::RAMA)]);
    for n in 1..=2 * ESCRITURAS {
        assert!(
            roadmap.contains(&format!("| TASK-{n:03} | t-")),
            "falta TASK-{n:03}:\n{roadmap}"
        );
    }
    let merges = f.crudo(&bare, &["rev-list", "--merges", estado::RAMA]);
    assert_eq!(merges, "", "historia lineal");
    let g = f.git();
    for c in [&clon, &otro] {
        f.crudo(c, &["fetch", "-q", "origin"]);
        let i = tc2::roadmap::comparar_en(&g, c);
        assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    }
}

#[test]
fn dos_procesos_del_mismo_clon_sin_remoto_los_serializa_el_cerrojo() {
    // Sin remoto no hay compare-and-swap en un servidor: lo unico que impide
    // que dos procesos del mismo clon se pisen es el cerrojo local.
    let f = Fixture::nueva("estado-cerrojo");
    let solo = f.raiz.join("solo");
    f.crudo(&f.raiz, &["init", "-q", "-b", "develop", "solo"]);
    comun::escribir(&solo, "README.md", "x\n");
    f.crudo(&solo, &["add", "README.md"]);
    f.crudo(&solo, &["commit", "-q", "-m", "inicial"]);
    let g = f.git();
    assert!(estado::montar(&g, &solo).resultado.es_exito());

    a_la_vez(&f, &[(&solo, "a"), (&solo, "b")]);

    let (notas, entradas, commits, merges) = contenido(&f, &solo, estado::RAMA);
    assert_eq!(notas, esperadas(&["a", "b"]));
    assert_eq!(entradas, 2 * ESCRITURAS);
    assert_eq!(commits, 1 + 2 * ESCRITURAS);
    assert_eq!(merges, 0);
    assert_eq!(
        f.crudo(&solo.join("tareas"), &["status", "--porcelain"]),
        ""
    );
    assert_eq!(f.crudo(&solo, &["for-each-ref", "refs/taskcodetwo/"]), "");
}
