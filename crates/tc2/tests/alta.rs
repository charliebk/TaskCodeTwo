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
use tc2::alta::Alta;
use tc2::contrato::Resultado;
use tc2::estado::{self, Cambio, Contexto, Estado, Fallo, Operacion};
use tc2::tarea::texto::Parecido;

/// La rama de estado sale de `huella.rs` (C1 de TASK-116).
const RAMA: &str = estado::RAMA;

#[test]
fn la_rama_de_estado_es_la_de_la_2_0() {
    assert_eq!(RAMA, "taskcodetwo/tareas");
    assert_eq!(RAMA, tc2::huella::RAMA_ESTADO);
}

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
        let refs = f.crudo(&dir, &["for-each-ref", "refs/taskcodetwo/"]);
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

/// TASK-029 C9: `new` acepta los campos de RF-01 (titulo, tipo, objetivo,
/// criterios, etiquetas, complejidad, sprint y dependencias) y los deja en
/// el `tarea.md`; un enunciado que no esta listo, un valor fuera de su
/// lista o una dependencia que no existe dan `uso` (2) sin escribir nada.
#[test]
fn c9_new_con_los_campos_de_rf_01_y_rechazo_de_un_enunciado_no_listo() {
    let f = Fixture::nueva("alta-rf01");
    let (bare, a) = f.remoto_y_clon();
    nueva(&f, &a, "Base");
    let o = tc2(
        &f,
        &a,
        &[
            "new",
            "Arreglar el login",
            "--tipo",
            "fix",
            "--objetivo",
            "Que la persona entre\ncon su cuenta",
            "--criterio",
            "Entra con usuario y clave",
            "--criterio",
            "Con clave mala da error",
            "--etiqueta",
            "seguridad",
            "--complejidad",
            "media",
            "--sprint",
            "3",
            "--dependencia",
            "TASK-001",
        ],
    );
    let out = texto(&o.stdout);
    assert_eq!(o.status.code(), Some(0), "{out} {}", texto(&o.stderr));
    assert_eq!(valor(&out, "id"), Some("TASK-002"), "{out}");
    assert_eq!(
        valor(&out, "rama"),
        Some("fix/task-002-arreglar-el-login"),
        "{out}"
    );
    let md = f.crudo(&bare, &["show", &format!("{RAMA}:TASK-002/tarea.md")]);
    for linea in [
        "\ntipo: \"fix\"\n",
        "\nsprint: 3\n",
        "\netiquetas: [\"seguridad\"]\n",
        "\ncomplejidad: \"media\"\n",
        "\nrama: \"fix/task-002-arreglar-el-login\"\n",
        "\ndependencias: [\"TASK-001\"]\n",
        "## Objetivo\n\nQue la persona entre\ncon su cuenta\n\n",
        "- [ ] Entra con usuario y clave\n- [ ] Con clave mala da error\n",
    ] {
        assert!(md.contains(linea), "falta {linea:?} en {md}");
    }
    assert!(!md.contains("\nlote:"), "{md}");

    let antes = f.crudo(
        &bare,
        &["for-each-ref", "--format=%(refname) %(objectname)"],
    );
    for (args, motivo) in [
        (
            vec!["new", "Sin criterios", "--objetivo", "Algo"],
            "enunciado-no-listo",
        ),
        (
            vec!["new", "Hueco", "--criterio", "<criterio comprobable>"],
            "enunciado-no-listo",
        ),
        (
            vec!["new", "Tipo malo", "--tipo", "epica"],
            "valor-invalido",
        ),
        (
            vec!["new", "Dep mala", "--dependencia", "TASK-099"],
            "referencia-inexistente",
        ),
    ] {
        let o = tc2(&f, &a, &args);
        let out = texto(&o.stdout);
        let err = texto(&o.stderr);
        assert_eq!(o.status.code(), Some(2), "{args:?}: {out} {err}");
        assert_eq!(valor(out.trim_end(), "motivo"), Some(motivo), "{out}");
        assert_eq!(err.lines().count(), 1, "{err}");
    }
    let despues = f.crudo(
        &bare,
        &["for-each-ref", "--format=%(refname) %(objectname)"],
    );
    assert_eq!(antes, despues, "un new rechazado publico algo");
    let o = tc2(&f, &a, &["new", "Sin criterios", "--objetivo", "Algo"]);
    assert!(
        texto(&o.stderr).contains("no esta listo: no tiene ningun criterio de aceptacion"),
        "{}",
        texto(&o.stderr)
    );
}

/// Pasa una tarea a `terminada` en la rama de estado (por la transaccion,
/// como lo haria `finish`).
struct Terminar(&'static str);

impl Operacion for Terminar {
    fn nombre(&self) -> &str {
        "terminar"
    }
    fn anotacion(&self) -> String {
        format!("terminar|{}", self.0)
    }
    fn exige_red(&self) -> bool {
        true
    }
    fn aplicar(&self, e: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        let ruta = format!("{}/tarea.md", self.0);
        let md = e.leer(&ruta)?.unwrap_or_default();
        let nuevo = md.replacen("estado: \"planificada\"", "estado: \"terminada\"", 1);
        assert_ne!(md, nuevo, "{md}");
        Ok(vec![Cambio::nuevo(ruta, nuevo)])
    }
}

fn sin_catalogo(_: &str) -> Option<Box<dyn Operacion>> {
    None
}

fn nueva(f: &Fixture, dir: &Path, titulo: &str) -> (String, String) {
    let o = tc2(f, dir, &["new", titulo]);
    let out = texto(&o.stdout);
    let err = texto(&o.stderr);
    assert_eq!(o.status.code(), Some(0), "{out} {err}");
    (out, err)
}

/// TASK-076 C4: un titulo que plegado es igual al de una tarea abierta sale
/// con `parecidas=` y el aviso `(duplicado)`, con codigo 0; contra una
/// terminada no avisa.
#[test]
fn titulo_duplicado_de_una_abierta_avisa_y_de_una_terminada_no() {
    let f = Fixture::nueva("alta-duplicado");
    let (_bare, a) = f.remoto_y_clon();
    let (out, err) = nueva(&f, &a, "Exportar informes");
    assert_eq!(valor(&out, "id"), Some("TASK-001"), "{out}");
    assert_eq!(valor(&out, "parecidas"), None, "{out}");
    assert!(err.is_empty(), "{err}");

    let (out, err) = nueva(&f, &a, "  EXPORTAR informes!");
    assert_eq!(valor(&out, "id"), Some("TASK-002"), "{out}");
    assert_eq!(valor(&out, "parecidas"), Some("TASK-001"), "{out}");
    assert_eq!(
        err,
        "aviso: ya hay tareas abiertas con un titulo igual o parecido: TASK-001 \"Exportar informes\" (duplicado)\n"
    );
    // Casi igual: avisa de las dos, con el coeficiente.
    let (out, err) = nueva(&f, &a, "Exprotar informes");
    assert_eq!(valor(&out, "parecidas"), Some("TASK-001,TASK-002"), "{out}");
    assert!(
        err.contains("TASK-001 \"Exportar informes\" (parecido 0,80)"),
        "{err}"
    );
    let o = tc2(&f, &a, &["new", "--json", "Exportar informes"]);
    let j = texto(&o.stdout);
    assert!(
        j.contains("\"parecidas\":[\"TASK-001\",\"TASK-002\",\"TASK-003\"]"),
        "{j}"
    );

    // Terminadas TASK-001..004, nada abierto con ese titulo: no avisa.
    let g = f.git();
    let c = Contexto::nuevo(&g, &a, &sin_catalogo);
    for id in ["TASK-001", "TASK-002", "TASK-003", "TASK-004"] {
        let i = estado::transaccion(&c, &Terminar(id));
        assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    }
    let (out, err) = nueva(&f, &a, "Exportar informes");
    assert_eq!(valor(&out, "id"), Some("TASK-005"), "{out}");
    assert_eq!(valor(&out, "parecidas"), None, "{out}");
    assert!(err.is_empty(), "{err}");
}

/// TASK-076, paso 3 del plan: con varias tareas, el alta lee todos los
/// `tarea.md` en un solo `git cat-file --batch` por intento, sin `git show`.
#[test]
fn el_alta_lee_las_tareas_en_un_solo_proceso_por_intento() {
    let f = Fixture::nueva("alta-lote");
    let (_bare, a) = f.remoto_y_clon();
    for t in ["Uno", "Dos", "Tres"] {
        nueva(&f, &a, t);
    }
    let g = f.git();
    let c = Contexto::nuevo(&g, &a, &sin_catalogo);
    let op = Alta::nueva("Tres");
    let i = estado::transaccion(&c, &op);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert_eq!(i.valor("intentos"), Some("1"), "{}", i.linea());
    let registro = g.registro();
    let lotes = registro
        .iter()
        .filter(|l| l.starts_with("git cat-file"))
        .count();
    let shows = registro
        .iter()
        .filter(|l| l.starts_with("git show "))
        .count();
    assert_eq!((lotes, shows), (1, 0), "{registro:#?}");
    assert_eq!(op.id().as_deref(), Some("TASK-004"));
    let p = op.parecidas();
    assert_eq!(p.len(), 1, "{p:?}");
    assert_eq!(
        (p[0].id.as_str(), p[0].parecido),
        ("TASK-003", Parecido::Duplicado)
    );
}

/// TASK-076 C1: dos `new "X"` a la vez desde dos clones dan IDs y ramas
/// distintos en los `tarea.md` publicados, y la segunda en publicar sale con
/// `parecidas=<ID de la otra>`.
#[test]
fn mismo_titulo_a_la_vez_desde_dos_clones() {
    let f = Arc::new(Fixture::nueva("alta-mismo-titulo"));
    let (bare, a) = f.remoto_y_clon();
    let b = f.otro_clon(&bare, "clon-b");
    let barrera = Arc::new(Barrier::new(2));
    let hilos: Vec<_> = [a, b]
        .into_iter()
        .map(|dir| {
            let f = Arc::clone(&f);
            let barrera = Arc::clone(&barrera);
            std::thread::spawn(move || {
                barrera.wait();
                for _ in 0..10 {
                    let o = tc2(&f, &dir, &["new", "Mismo titulo"]);
                    let out = texto(&o.stdout);
                    // `agotada` (5) es «reintenta, nada escrito».
                    if o.status.code() == Some(5) {
                        continue;
                    }
                    assert_eq!(o.status.code(), Some(0), "{out} {}", texto(&o.stderr));
                    return (out, texto(&o.stderr));
                }
                panic!("agotada 10 veces");
            })
        })
        .collect();
    let mut salidas: Vec<(String, String)> = hilos.into_iter().map(|h| h.join().unwrap()).collect();
    salidas.sort_by(|x, y| valor(&x.0, "id").cmp(&valor(&y.0, "id")));
    let (primera, segunda) = (&salidas[0], &salidas[1]);
    assert_eq!(valor(&primera.0, "id"), Some("TASK-001"), "{primera:?}");
    assert_eq!(valor(&segunda.0, "id"), Some("TASK-002"), "{segunda:?}");
    assert_eq!(valor(&primera.0, "parecidas"), None, "{primera:?}");
    assert!(primera.1.is_empty(), "{primera:?}");
    assert_eq!(
        valor(&segunda.0, "parecidas"),
        Some("TASK-001"),
        "{segunda:?}"
    );
    assert!(
        segunda.1.contains("TASK-001 \"Mismo titulo\" (duplicado)"),
        "{segunda:?}"
    );
    // Ramas distintas en lo publicado, y la misma que dijo cada `new`.
    for (n, (out, _)) in [1, 2].iter().zip(&salidas) {
        let rama = format!("feature/task-{n:03}-mismo-titulo");
        assert_eq!(valor(out, "rama"), Some(rama.as_str()), "{out}");
        let md = f.crudo(&bare, &["show", &format!("{RAMA}:TASK-{n:03}/tarea.md")]);
        assert!(md.contains(&format!("\nrama: \"{rama}\"\n")), "{md}");
    }
}
