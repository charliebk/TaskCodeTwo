//! F3-2: `tc2 import` en bloque con dependencias e hito. E2E con el binario
//! real contra un remoto bare. Los ficheros de import se escriben en la
//! carpeta temporal de la fixture, fuera del repo.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::sync::{Arc, Barrier};

use comun::{Fixture, texto, valor};
use tc2::estado;

const RAMA: &str = estado::RAMA;

fn tc2(f: &Fixture, dir: &Path, args: &[&str]) -> Output {
    comun::lanzar(dir, f.entorno(), args)
}

/// Escribe el fichero de import en la carpeta temporal de la fixture.
fn fichero(f: &Fixture, nombre: &str, contenido: &str) -> PathBuf {
    let p = f.raiz.join(nombre);
    std::fs::write(&p, contenido).unwrap();
    p
}

/// `import <fichero> [args]`: (codigo, stdout, stderr).
fn importar(f: &Fixture, dir: &Path, ruta: &Path, mas: &[&str]) -> (i32, String, String) {
    let mut args = vec!["import", ruta.to_str().unwrap()];
    args.extend_from_slice(mas);
    let o = tc2(f, dir, &args);
    (
        o.status.code().unwrap_or(-1),
        texto(&o.stdout),
        texto(&o.stderr),
    )
}

/// Un lote de `n` entradas `H<i>` con cabecera; `deps(i)` da la linea de
/// dependencias de la entrada `i` (desde 1).
fn lote(nombre: &str, sprint: u32, n: usize, deps: impl Fn(usize) -> Option<String>) -> String {
    let mut t = format!("---\nlote: {nombre}\nsprint: {sprint}\n---\n# Plan {nombre}\n\n");
    for i in 1..=n {
        write!(
            t,
            "### H{i} Tarea {nombre} numero {i}\n> Objetivo de la {i}\n- Criterio {i}a\n- Criterio {i}b\n"
        )
        .unwrap();
        if let Some(d) = deps(i) {
            writeln!(t, "Dependencias: {d}").unwrap();
        }
        t.push('\n');
    }
    t
}

/// Carpetas `TASK-*` publicadas.
fn carpetas(f: &Fixture, bare: &Path) -> Vec<String> {
    f.crudo(bare, &["ls-tree", "--name-only", RAMA])
        .lines()
        .filter(|l| l.starts_with("TASK-"))
        .map(str::to_owned)
        .collect()
}

fn ids(desde: u32, hasta: u32) -> String {
    (desde..=hasta)
        .map(|n| format!("TASK-{n:03}"))
        .collect::<Vec<_>>()
        .join(",")
}

/// Refs del remoto (la rama de estado incluida): si no cambian, no se
/// publico nada.
fn refs(f: &Fixture, bare: &Path) -> String {
    f.crudo(bare, &["for-each-ref", "--format=%(refname) %(objectname)"])
}

/// C1 y C3: 37 entradas dan un commit, 37 carpetas consecutivas con su
/// sprint, objetivo, criterios y dependencias; repetirlo no crea nada.
#[test]
fn c1_c3_importa_37_en_un_commit_y_repetir_no_crea_nada() {
    let f = Fixture::nueva("import-37");
    let (bare, a) = f.remoto_y_clon();
    let deps = |i: usize| match i {
        2 => Some("H1".to_owned()),
        37 => Some("H36, H1".to_owned()),
        _ => None,
    };
    let ruta = fichero(&f, "hito37.md", &lote("hito37", 3, 37, deps));
    let (codigo, out, err) = importar(&f, &a, &ruta, &[]);
    assert_eq!(codigo, 0, "{out} {err}");
    assert!(
        out.starts_with(&format!(
            "import ok creadas=37 existentes=0 ids={} operacion=import commit=",
            ids(1, 37)
        )),
        "{out}"
    );
    // Un solo commit en la rama de estado (era nueva) y 37 carpetas.
    let commits = f.crudo(&bare, &["rev-list", "--count", RAMA]);
    assert_eq!(commits.trim(), "1");
    let esperadas: Vec<String> = (1..=37).map(|n| format!("TASK-{n:03}")).collect();
    assert_eq!(carpetas(&f, &bare), esperadas);
    for i in 1..=37usize {
        let md = f.publicado(&bare, &format!("TASK-{i:03}/tarea.md"));
        assert!(
            md.contains(&format!("\ntitulo: \"H{i} Tarea hito37 numero {i}\"\n")),
            "{md}"
        );
        assert!(md.contains("\nsprint: 3\n"), "{md}");
        assert!(md.contains(&format!("\nlote: \"hito37/H{i}\"\n")), "{md}");
        assert!(
            md.contains(&format!("## Objetivo\n\nObjetivo de la {i}\n\n")),
            "{md}"
        );
        assert!(
            md.contains(&format!("- [ ] Criterio {i}a\n- [ ] Criterio {i}b\n")),
            "{md}"
        );
        assert!(
            md.contains(&format!(
                "\nrama: \"feature/task-{i:03}-h{i}-tarea-hito37-numero-{i}\"\n"
            )),
            "{md}"
        );
        let leida = tc2::tarea::leer(&md).unwrap();
        assert!(matches!(leida, tc2::tarea::Documento::Formato2(..)));
    }
    // C2: las claves locales quedan resueltas a TASK-NNN.
    assert!(
        f.publicado(&bare, "TASK-002/tarea.md")
            .contains("\ndependencias: [\"TASK-001\"]\n")
    );
    assert!(
        f.publicado(&bare, "TASK-037/tarea.md")
            .contains("\ndependencias: [\"TASK-036\", \"TASK-001\"]\n")
    );
    assert!(
        f.publicado(&bare, "TASK-003/tarea.md")
            .contains("\ndependencias: []\n")
    );

    // C3: repetirlo no crea nada ni publica nada.
    let antes = f.huella(&bare, &a);
    let (codigo, out, err) = importar(&f, &a, &ruta, &[]);
    assert_eq!(codigo, 0, "{out} {err}");
    assert!(
        out.starts_with("import sin-cambios creadas=0 existentes=37 operacion=import"),
        "{out}"
    );
    assert_eq!(valor(&out, "ids"), None, "{out}");
    assert_eq!(antes, f.huella(&bare, &a), "la 2.a vez cambio el estado");
    assert_eq!(f.crudo(&bare, &["rev-list", "--count", RAMA]).trim(), "1");
}

/// C2: las externas se validan contra el estado; una inexistente, un ciclo
/// o una clave que no esta en el fichero dan `uso` (2) sin commit.
#[test]
fn c2_dependencias_externas_validadas_y_errores_sin_commit() {
    let f = Fixture::nueva("import-deps");
    let (bare, a) = f.remoto_y_clon();
    let o = tc2(&f, &a, &["new", "Existente"]);
    assert_eq!(o.status.code(), Some(0), "{}", texto(&o.stdout));
    let ruta = fichero(
        &f,
        "externas.md",
        "### E1 Usa la existente\n- c\nDependencias: TASK-001\n### E2 Y la local\n- c\nDependencias: E1, TASK-001\n",
    );
    let (codigo, out, err) = importar(&f, &a, &ruta, &["--sprint", "2"]);
    assert_eq!(codigo, 0, "{out} {err}");
    assert_eq!(valor(&out, "ids"), Some("TASK-002,TASK-003"), "{out}");
    assert!(
        f.publicado(&bare, "TASK-002/tarea.md")
            .contains("\ndependencias: [\"TASK-001\"]\n")
    );
    assert!(
        f.publicado(&bare, "TASK-003/tarea.md")
            .contains("\ndependencias: [\"TASK-002\", \"TASK-001\"]\n")
    );
    assert!(
        f.publicado(&bare, "TASK-003/tarea.md")
            .contains("\nsprint: 2\n")
    );
    // Sin cabecera, el lote es el nombre del fichero.
    assert!(
        f.publicado(&bare, "TASK-002/tarea.md")
            .contains("\nlote: \"externas/E1\"\n")
    );

    let sha = refs(&f, &bare);
    for (nombre, contenido, motivo) in [
        (
            "inexistente.md",
            "### X1 a\n- c\nDependencias: TASK-099\n",
            "referencia-inexistente",
        ),
        (
            "local.md",
            "### X1 a\n- c\nDependencias: X9\n",
            "referencia-inexistente",
        ),
        (
            "ciclo.md",
            "### X1 a\n- c\nDependencias: X2\n### X2 b\n- c\nDependencias: X1\n",
            "ciclo",
        ),
    ] {
        let ruta = fichero(&f, nombre, contenido);
        let (codigo, out, err) = importar(&f, &a, &ruta, &[]);
        assert_eq!(codigo, 2, "{nombre}: {out} {err}");
        assert!(out.starts_with("import uso "), "{out}");
        assert_eq!(valor(&out, "motivo"), Some(motivo), "{out}");
        assert_eq!(err.lines().count(), 1, "{err}");
        assert_eq!(sha, refs(&f, &bare), "{nombre}: la rama de estado cambio");
    }
}

/// C4: el mismo fichero con 2 entradas mas crea solo esas 2, con sus
/// dependencias hacia las ya importadas resueltas.
#[test]
fn c4_dos_entradas_mas_crean_solo_esas_dos() {
    let f = Fixture::nueva("import-ampliado");
    let (bare, a) = f.remoto_y_clon();
    let base = lote("hito", 4, 5, |_| None);
    let ruta = fichero(&f, "hito.md", &base);
    let (codigo, out, _) = importar(&f, &a, &ruta, &[]);
    assert_eq!((codigo, valor(&out, "creadas")), (0, Some("5")), "{out}");
    // Otra tarea entre medias: los nuevos IDs van detras.
    assert_eq!(tc2(&f, &a, &["new", "Entre medias"]).status.code(), Some(0));
    let ampliado = format!(
        "{base}### H6 Nueva seis\n- c6\nDependencias: H5, TASK-006\n\n### H7 Nueva siete\n- c7\nDependencias: H1, H6\n"
    );
    let ruta = fichero(&f, "hito.md", &ampliado);
    let (codigo, out, err) = importar(&f, &a, &ruta, &[]);
    assert_eq!(codigo, 0, "{out} {err}");
    assert!(
        out.starts_with("import ok creadas=2 existentes=5 ids=TASK-007,TASK-008 "),
        "{out}"
    );
    assert!(
        f.publicado(&bare, "TASK-007/tarea.md")
            .contains("\ndependencias: [\"TASK-005\", \"TASK-006\"]\n")
    );
    assert!(
        f.publicado(&bare, "TASK-008/tarea.md")
            .contains("\ndependencias: [\"TASK-001\", \"TASK-007\"]\n")
    );
    assert!(
        f.publicado(&bare, "TASK-008/tarea.md")
            .contains("\nlote: \"hito/H7\"\n")
    );
    assert_eq!(carpetas(&f, &bare).len(), 8);
}

/// C5 (i): una entrada invalida en la posicion 30 no escribe nada.
#[test]
fn c5_entrada_invalida_en_la_30_no_escribe_nada() {
    let f = Fixture::nueva("import-invalida");
    let (bare, a) = f.remoto_y_clon();
    let bueno = lote("malo", 1, 40, |_| None);
    let malo = bueno.replacen("- Criterio 30a\n- Criterio 30b\n", "", 1);
    assert_ne!(bueno, malo);
    let ruta = fichero(&f, "malo.md", &malo);
    let antes = f.huella(&bare, &a);
    let (codigo, out, err) = importar(&f, &a, &ruta, &[]);
    assert_eq!(codigo, 2, "{out} {err}");
    assert!(
        out.starts_with("import uso motivo=enunciado-no-listo linea="),
        "{out}"
    );
    assert_eq!(valor(&out, "causa"), Some("sin-criterios"), "{out}");
    assert!(err.contains("H30 Tarea malo numero 30"), "{err}");
    assert_eq!(
        antes,
        f.huella(&bare, &a),
        "un import invalido toco el estado"
    );
}

/// C5 (ii): dos clones importando a la vez dan IDs sin solape y sin huecos
/// dentro de cada lote.
#[test]
fn c5_dos_clones_importando_a_la_vez() {
    const N: usize = 12;
    let f = Arc::new(Fixture::nueva("import-paralelo"));
    let (bare, a) = f.remoto_y_clon();
    let b = f.otro_clon(&bare, "clon-b");
    let barrera = Arc::new(Barrier::new(2));
    let hilos: Vec<_> = [("lote-a", a), ("lote-b", b)]
        .into_iter()
        .map(|(nombre, dir)| {
            let f = Arc::clone(&f);
            let barrera = Arc::clone(&barrera);
            let ruta = fichero(&f, &format!("{nombre}.md"), &lote(nombre, 1, N, |_| None));
            std::thread::spawn(move || {
                barrera.wait();
                for _ in 0..10 {
                    let (codigo, out, err) = importar(&f, &dir, &ruta, &[]);
                    // `agotada` (5) es «reintenta, nada escrito».
                    if codigo == 5 {
                        continue;
                    }
                    assert_eq!(codigo, 0, "{out} {err}");
                    return valor(&out, "ids").unwrap().to_owned();
                }
                panic!("agotada 10 veces");
            })
        })
        .collect();
    let listas: Vec<Vec<u32>> = hilos
        .into_iter()
        .map(|h| {
            h.join()
                .unwrap()
                .split(',')
                .map(|id| id.trim_start_matches("TASK-").parse().unwrap())
                .collect()
        })
        .collect();
    for l in &listas {
        assert_eq!(l.len(), N);
        assert!(l.windows(2).all(|w| w[1] == w[0] + 1), "hueco en {l:?}");
    }
    let todos: BTreeSet<u32> = listas.iter().flatten().copied().collect();
    assert_eq!(todos.len(), 2 * N, "solape: {listas:?}");
    assert_eq!(todos, (1..=u32::try_from(2 * N).unwrap()).collect());
    assert_eq!(carpetas(&f, &bare).len(), 2 * N);
}

/// C7: pasar los limites da `uso` con su motivo; sin red, `sinred
/// pendiente=no` sin dejar nada.
#[test]
fn c7_limites_y_sin_red() {
    let f = Fixture::nueva("import-limites");
    let (bare, a) = f.remoto_y_clon();
    let antes = f.huella(&bare, &a);
    let grande = fichero(&f, "grande.md", &"x".repeat((1 << 20) + 1));
    let (codigo, out, _) = importar(&f, &a, &grande, &[]);
    assert_eq!(codigo, 2, "{out}");
    assert_eq!(out, "import uso motivo=fichero-grande limite=1048576\n");
    let muchas = fichero(&f, "muchas.md", &lote("muchas", 1, 201, |_| None));
    let (codigo, out, _) = importar(&f, &a, &muchas, &[]);
    assert_eq!(codigo, 2, "{out}");
    assert_eq!(out, "import uso motivo=demasiadas-entradas limite=200\n");
    // Vacio o solo espacios: se rechaza al leer, como en `plan` (A3-03).
    let vacio = fichero(&f, "vacio.md", " \n\t\n");
    let (codigo, out, _) = importar(&f, &a, &vacio, &[]);
    assert_eq!(codigo, 2, "{out}");
    assert_eq!(out, "import uso motivo=vacio\n");
    assert_eq!(antes, f.huella(&bare, &a));
    // 200 justas si valen.
    let justas = fichero(&f, "justas.md", &lote("justas", 1, 200, |_| None));
    let (codigo, out, err) = importar(&f, &a, &justas, &[]);
    assert_eq!(
        (codigo, valor(&out, "creadas")),
        (0, Some("200")),
        "{out} {err}"
    );

    f.crudo(
        &a,
        &[
            "remote",
            "set-url",
            "origin",
            "http://127.0.0.1:1/remoto.git",
        ],
    );
    let refs_antes = f.crudo(&a, &["for-each-ref", "--format=%(refname) %(objectname)"]);
    let otro = fichero(&f, "otro.md", &lote("otro", 1, 2, |_| None));
    let (codigo, out, _) = importar(&f, &a, &otro, &[]);
    assert_eq!(codigo, 4, "{out}");
    assert!(out.starts_with("import sinred "), "{out}");
    assert_eq!(valor(&out, "pendiente"), Some("no"), "{out}");
    let refs_despues = f.crudo(&a, &["for-each-ref", "--format=%(refname) %(objectname)"]);
    assert_eq!(refs_antes, refs_despues, "import sin red dejo refs");
}

/// C6: `--simular` imprime los IDs previstos y no cambia ni un byte en
/// local ni en el remoto; el import real da esos mismos IDs.
#[test]
fn c6_simular_no_cambia_ni_un_byte() {
    let f = Fixture::nueva("import-simular");
    let (bare, a) = f.remoto_y_clon();
    // Sin rama de estado todavia: preve desde el 1.
    let ruta = fichero(&f, "sim.md", &lote("sim", 5, 3, |_| None));
    let (antes_a, antes_r) = (comun::instantanea(&a, &[]), comun::instantanea(&bare, &[]));
    let (codigo, out, err) = importar(&f, &a, &ruta, &["--simular"]);
    assert_eq!(codigo, 0, "{out} {err}");
    assert_eq!(
        out,
        "import ok simular=si creadas=3 existentes=0 ids=TASK-001,TASK-002,TASK-003\n"
    );
    assert!(
        antes_a == comun::instantanea(&a, &[]),
        "--simular cambio el clon"
    );
    assert!(
        antes_r == comun::instantanea(&bare, &[]),
        "--simular cambio el remoto"
    );

    let (codigo, out, _) = importar(&f, &a, &ruta, &[]);
    assert_eq!(
        valor(&out, "ids"),
        Some("TASK-001,TASK-002,TASK-003"),
        "{codigo} {out}"
    );
    // Con estado: dos entradas mas, previstas sobre el commit conocido.
    let ampliado = format!(
        "{}### H4 Cuarta\n- c\nDependencias: H1\n### H5 Quinta\n- c\n",
        lote("sim", 5, 3, |_| None)
    );
    let ruta = fichero(&f, "sim.md", &ampliado);
    let (antes_a, antes_r) = (comun::instantanea(&a, &[]), comun::instantanea(&bare, &[]));
    let (codigo, out, err) = importar(&f, &a, &ruta, &["--simular", "--json"]);
    assert_eq!(codigo, 0, "{out} {err}");
    assert!(
        out.contains("\"claves\":{\"simular\":\"si\",\"creadas\":\"2\",\"existentes\":\"3\",\"ids\":[\"TASK-004\",\"TASK-005\"],\"base\":\""),
        "{out}"
    );
    assert!(
        antes_a == comun::instantanea(&a, &[]),
        "--simular cambio el clon"
    );
    assert!(
        antes_r == comun::instantanea(&bare, &[]),
        "--simular cambio el remoto"
    );
    let (codigo, out, _) = importar(&f, &a, &ruta, &[]);
    assert_eq!(
        valor(&out, "ids"),
        Some("TASK-004,TASK-005"),
        "{codigo} {out}"
    );
    // Todo importado: la simulacion dice sin-cambios.
    let (codigo, out, _) = importar(&f, &a, &ruta, &["--simular"]);
    assert_eq!(codigo, 0, "{out}");
    assert!(
        out.starts_with("import sin-cambios simular=si creadas=0 existentes=5 base="),
        "{out}"
    );
}

/// El bloque `markdown` de §5 de `AMPLIACION-2026-11-16.md`, tal cual.
fn fichero_de_la_ampliacion() -> String {
    let doc = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/plan/AMPLIACION-2026-11-16.md"),
    )
    .unwrap();
    // Byte a byte, con los fines de linea que tenga el checkout (CRLF o LF).
    let seccion = &doc[doc.find("## 5. Fichero de import").unwrap()..];
    let apertura = seccion.find("```markdown").unwrap();
    let desde = apertura + seccion[apertura..].find('\n').unwrap() + 1;
    let hasta = seccion[desde..].find("\n```").unwrap() + 1;
    seccion[desde..desde + hasta].to_owned()
}

/// C8: el fichero de import de formato 0.16 de la ampliacion se importa sin
/// tocarlo (sin cabecera: el sprint llega por `--sprint`).
#[test]
fn c8_fichero_016_de_la_ampliacion_se_importa_sin_tocarlo() {
    let f = Fixture::nueva("import-016");
    let (bare, a) = f.remoto_y_clon();
    let texto = fichero_de_la_ampliacion();
    let n = texto.lines().filter(|l| l.starts_with("### ")).count();
    assert!(n > 10, "{texto}");
    let ruta = fichero(&f, "ampliacion.md", &texto);
    let (codigo, out, err) = importar(&f, &a, &ruta, &["--sprint", "10"]);
    assert_eq!(codigo, 0, "{out} {err}");
    assert_eq!(
        valor(&out, "creadas"),
        Some(n.to_string().as_str()),
        "{out}"
    );
    assert_eq!(
        std::fs::read_to_string(&ruta).unwrap(),
        texto,
        "se toco el fichero"
    );
    let md = f.publicado(&bare, "TASK-001/tarea.md");
    assert!(
        md.contains("\ntitulo: \"F1-19 ADR memoria del proyecto: indice de contexto, etiquetas y busqueda\"\n"),
        "{md}"
    );
    assert!(md.contains("\nsprint: 10\n"), "{md}");
    assert!(md.contains("\nlote: \"ampliacion/F1-19\"\n"), "{md}");
    assert!(
        md.contains("- [ ] Cumple lo que pide la tarea F1-19 del plan"),
        "{md}"
    );
}
