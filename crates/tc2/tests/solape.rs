//! F3-11 (TASK-082): aviso de solape en `tc2 start` con el binario real y
//! dos clones (ana y luis) de un remoto bare, como `tests/toma.rs`: `origin`
//! apunta a GitHub y el transporte ssh lo sirve `GIT_SSH_COMMAND` desde el
//! bare de la fixture.
//!
//! Escena comun ([`escena`]): ana empieza TASK-001, toca unos ficheros y
//! publica su rama; luis empieza TASK-002 (primer `start`, rama vacia: la
//! nota), toca los suyos, publica y vuelve a lanzar `start` (reanudar): el
//! aviso.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use comun::{Fixture, escribir};
use tc2::contrato::Resultado;
use tc2::estado::{self, Cambio, Contexto, Estado, Fallo, Operacion};
use tc2::identidad::{Cache, Cuenta, Quien};
use tc2::plataforma::Proveedor;
use tc2::solape::{self, EnCurso};

const ORIGIN: &str = "ssh://git@github.com/o/r.git";
const PERSONAS: &str =
    "---\npersonas:\n  - {id: 1001, login: \"ana\"}\n  - {id: 1002, login: \"luis\"}\n";
const RAMA_1: &str = "feature/task-001-uno";
const RAMA_2: &str = "feature/task-002-dos";
const RAMA_3: &str = "feature/task-003-tres";

/// Entorno de git con el «ssh» que sirve el remoto bare de la fixture.
fn entorno_ssh(f: &Fixture) -> Vec<(OsString, OsString)> {
    let bare = f
        .raiz
        .join("remoto.git")
        .to_string_lossy()
        .replace('\\', "/");
    let mut e = f.entorno();
    e.push((
        "GIT_SSH_COMMAND".into(),
        format!(
            "sh -c 'case \"$2\" in *receive-pack*) exec git receive-pack \"$0\";; *) exec git upload-pack \"$0\";; esac' '{bare}'"
        )
        .into(),
    ));
    e.push(("GIT_SSH_VARIANT".into(), "simple".into()));
    e
}

fn tc2(f: &Fixture, dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tc2"))
        .current_dir(dir)
        .args(args)
        .envs(entorno_ssh(f))
        .env("LC_ALL", "C")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("TC2_IDIOMA", "es")
        .env_remove("TC2_TOKEN_PERSONA")
        .env_remove("TC2_TOKEN_AGENTE")
        .output()
        .expect("lanzar tc2")
}

fn texto(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

/// `git <args>` en `dir` con el remoto de la fixture.
fn git(f: &Fixture, dir: &Path, args: &[&str]) -> String {
    let o = Command::new("git")
        .current_dir(dir)
        .args(args)
        .envs(entorno_ssh(f))
        .env("LC_ALL", "C")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("lanzar git");
    assert!(o.status.success(), "git {args:?}: {}", texto(&o.stderr));
    texto(&o.stdout)
}

/// Valor de `clave=` en una linea de contrato.
fn valor<'a>(linea: &'a str, clave: &str) -> Option<&'a str> {
    linea
        .split(' ')
        .find_map(|kv| kv.strip_prefix(clave)?.strip_prefix('='))
}

/// Las claves `solape*` y `en-curso-modulos` de una linea.
fn claves_solape(linea: &str) -> Vec<&str> {
    linea
        .split(' ')
        .filter(|kv| kv.starts_with("solape") || kv.starts_with("en-curso-modulos"))
        .collect()
}

/// La linea sin las claves del solape.
fn sin_solape(linea: &str) -> String {
    linea
        .split(' ')
        .filter(|kv| !kv.starts_with("solape") && !kv.starts_with("en-curso-modulos"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn identificar(dir: &Path, id: u64, login: &str) {
    let cuenta = Cuenta {
        proveedor: Proveedor::GitHub,
        id,
        login: login.to_owned(),
    };
    Cache::en(&dir.join(".git"))
        .guardar(&cuenta, tc2::identidad::ahora())
        .unwrap();
}

fn otro_clon(f: &Fixture, bare: &Path, nombre: &str) -> PathBuf {
    let d = f.otro_clon(bare, nombre);
    f.crudo(&d, &["remote", "set-url", "origin", ORIGIN]);
    d
}

fn worktree(dir: &Path, id: &str) -> PathBuf {
    dir.join(".taskcodetwo").join("worktrees").join(id)
}

/// Escribe en la rama de estado por la transaccion (como otra fase).
struct Escribir {
    ruta: String,
    de: &'static str,
    a: &'static str,
}

impl Operacion for Escribir {
    fn nombre(&self) -> &str {
        "escribir"
    }
    fn anotacion(&self) -> String {
        format!("escribir|{}", self.ruta)
    }
    fn exige_red(&self) -> bool {
        true
    }
    fn aplicar(&self, e: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        let t = e.leer(&self.ruta)?.unwrap_or_default();
        assert!(t.contains(self.de), "{t}");
        Ok(vec![Cambio::nuevo(
            self.ruta.clone(),
            t.replacen(self.de, self.a, 1),
        )])
    }
}

fn sin_catalogo(_: &str) -> Option<Box<dyn Operacion>> {
    None
}

/// Cambia `de` por `a` en el `tarea.md` publicado de `id`.
fn editar(f: &Fixture, dir: &Path, id: &str, de: &'static str, a: &'static str) {
    let g = tc2::git::Git::nuevo().con_entorno(entorno_ssh(f));
    let op = Escribir {
        ruta: format!("{id}/tarea.md"),
        de,
        a,
    };
    let i = estado::transaccion(&Contexto::nuevo(&g, dir, &sin_catalogo, None), &op);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
}

/// `tc2 start id` en `dir`: tiene que salir 0.
fn start(f: &Fixture, dir: &Path, id: &str) -> (String, String) {
    let o = tc2(f, dir, &["start", id]);
    let (out, err) = (texto(&o.stdout), texto(&o.stderr));
    assert_eq!(o.status.code(), Some(0), "{out}{err}");
    assert_eq!(out.lines().count(), 1, "{out}");
    (out.trim_end().to_owned(), err)
}

/// Escribe `ficheros` en el worktree de `id`, los guarda en un commit y, si
/// `publicar`, empuja la rama.
fn tocar(f: &Fixture, dir: &Path, id: &str, rama: &str, ficheros: &[&str], publicar: bool) {
    let wt = worktree(dir, id);
    for r in ficheros {
        escribir(&wt, r, &format!("de {id}\n"));
        f.crudo(&wt, &["add", "--", r]);
    }
    f.crudo(&wt, &["commit", "-q", "-m", &format!("trabajo de {id}")]);
    if publicar {
        git(f, &wt, &["push", "-q", "origin", rama]);
    }
}

/// Lo que deja la escena comun.
struct Escena {
    f: Fixture,
    bare: PathBuf,
    a: PathBuf,
    b: PathBuf,
    /// Primer `start TASK-002` de luis (rama vacia): linea y stderr.
    primera: (String, String),
    /// `start TASK-002` de luis con sus ficheros publicados.
    ultima: (String, String),
}

/// Remoto con TASK-001..003; ana (clon `a`) empieza TASK-001, toca
/// `de_ana` y publica; `antes` prepara lo demas; luis (clon `b`) empieza
/// TASK-002, toca `de_luis`, publica y lanza `start TASK-002` otra vez.
fn escena(
    nombre: &str,
    de_ana: &[&str],
    de_luis: &[&str],
    antes: impl FnOnce(&Fixture, &Path),
) -> Escena {
    let f = Fixture::nueva(nombre);
    let (bare, prep) = f.remoto_y_clon();
    escribir(&prep, ".taskcode/personas.yml", PERSONAS);
    f.crudo(&prep, &["add", ".taskcode/personas.yml"]);
    f.crudo(&prep, &["commit", "-q", "-m", "personas"]);
    f.crudo(&prep, &["push", "-q", "origin", "develop"]);
    for t in ["Uno", "Dos", "Tres"] {
        let o = Command::new(env!("CARGO_BIN_EXE_tc2"))
            .current_dir(&prep)
            .args(["new", t])
            .envs(f.entorno())
            .env("LC_ALL", "C")
            .output()
            .expect("tc2 new");
        assert_eq!(o.status.code(), Some(0), "{}", texto(&o.stdout));
    }
    let a = otro_clon(&f, &bare, "a");
    identificar(&a, 1001, "ana");
    let b = otro_clon(&f, &bare, "b");
    identificar(&b, 1002, "luis");
    start(&f, &a, "TASK-001");
    tocar(&f, &a, "TASK-001", RAMA_1, de_ana, true);
    antes(&f, &a);
    let primera = start(&f, &b, "TASK-002");
    tocar(&f, &b, "TASK-002", RAMA_2, de_luis, true);
    let ultima = start(&f, &b, "TASK-002");
    Escena {
        f,
        bare,
        a,
        b,
        primera,
        ultima,
    }
}

fn nada(_: &Fixture, _: &Path) {}

/// Lineas de stderr que empiezan por `prefijo`.
fn lineas<'a>(err: &'a str, prefijo: &str) -> Vec<&'a str> {
    err.lines().filter(|l| l.starts_with(prefijo)).collect()
}

/// C1: las dos ramas publicadas tocan `src/x/a.rs`: `start TASK-002` de
/// luis sale 0 con una linea `aviso:` que nombra TASK-001, @ana y el
/// fichero, y las claves `solape`, `solape-ficheros` y `solape-asignados`
/// en la linea y en `--json`.
#[test]
fn c1_mismo_fichero_avisa_con_tarea_persona_y_fichero() {
    let e = escena("solape-c1", &["src/x/a.rs"], &["src/x/a.rs"], nada);
    let (out, err) = &e.ultima;
    assert!(out.starts_with("start sin-cambios id=TASK-002 "), "{out}");
    let avisos = lineas(err, "aviso:");
    assert_eq!(avisos.len(), 1, "{err}");
    for parte in ["TASK-001", "@ana", "src/x/a.rs"] {
        assert!(avisos[0].contains(parte), "{err}");
    }
    assert_eq!(err.lines().count(), 1, "{err}");
    assert_eq!(valor(out, "solape"), Some("TASK-001"), "{out}");
    assert_eq!(
        valor(out, "solape-ficheros"),
        Some("TASK-001:src/x/a.rs"),
        "{out}"
    );
    assert_eq!(
        valor(out, "solape-asignados"),
        Some("TASK-001:@ana"),
        "{out}"
    );
    assert_eq!(valor(out, "solape-modulos"), None, "{out}");
    // Con `--json`, las mismas claves como listas; stderr fuera del JSON.
    let o = tc2(&e.f, &e.b, &["start", "--json", "TASK-002"]);
    assert_eq!(o.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_str(&texto(&o.stdout)).unwrap();
    let c = &v["claves"];
    assert_eq!(c["solape"], serde_json::json!(["TASK-001"]));
    assert_eq!(
        c["solape-ficheros"],
        serde_json::json!(["TASK-001:src/x/a.rs"])
    );
    assert_eq!(c["solape-asignados"], serde_json::json!(["TASK-001:@ana"]));
    assert!(texto(&o.stderr).starts_with("aviso: TASK-001 (@ana) "));
}

/// C2: `src/x/a.rs` frente a `src/x/b.rs`: aviso de modulo
/// (`solape-modulos=TASK-001:src/x`) y sin `solape-ficheros`.
#[test]
fn c2_misma_carpeta_avisa_de_modulo() {
    let e = escena("solape-c2", &["src/x/a.rs"], &["src/x/b.rs"], nada);
    let (out, err) = &e.ultima;
    assert_eq!(valor(out, "solape"), Some("TASK-001"), "{out}");
    assert_eq!(
        valor(out, "solape-modulos"),
        Some("TASK-001:src/x"),
        "{out}"
    );
    assert_eq!(valor(out, "solape-ficheros"), None, "{out}");
    let avisos = lineas(err, "aviso:");
    assert_eq!(avisos.len(), 1, "{err}");
    assert!(avisos[0].contains("TASK-001") && avisos[0].contains("src/x"));
}

/// C3 y C4: sin nada en comun no hay aviso ni claves `solape*`; con solape
/// y sin el, `start` da el mismo resultado, codigo y linea (salvo las
/// claves del solape), no publica nada al reanudar y deja la misma rama y
/// el mismo worktree.
#[test]
fn c3_c4_sin_solape_nada_y_el_aviso_no_cambia_start() {
    let con = escena("solape-c4-con", &["src/x/a.rs"], &["src/x/a.rs"], nada);
    let sin = escena("solape-c4-sin", &["src/x/a.rs"], &["web/y/c.ts"], nada);
    // C3.
    let (out, err) = &sin.ultima;
    assert!(claves_solape(out).is_empty(), "{out}");
    assert!(lineas(err, "aviso:").is_empty(), "{err}");
    assert!(err.is_empty(), "{err}");
    // C4: misma linea salvo el solape, en la reanudacion y en la toma.
    assert!(!claves_solape(&con.ultima.0).is_empty());
    assert_eq!(sin_solape(&con.ultima.0), sin_solape(&sin.ultima.0));
    let nombres = |l: &str| -> Vec<String> {
        sin_solape(l)
            .split(' ')
            .map(|kv| kv.split('=').next().unwrap_or_default().to_owned())
            .collect()
    };
    assert_eq!(nombres(&con.primera.0), nombres(&sin.primera.0));
    for e in [&con, &sin] {
        assert!(e.primera.0.starts_with("start ok id=TASK-002 "));
        assert_eq!(valor(&e.primera.0, "asignado"), Some("luis"));
        // La toma publicada: TASK-002 en curso y de luis.
        let md = e.f.crudo(
            &e.bare,
            &["show", &format!("{}:TASK-002/tarea.md", estado::RAMA)],
        );
        assert!(md.contains("estado: \"en-curso\""), "{md}");
        assert!(md.contains("login: \"luis\""), "{md}");
        // La rama y el worktree.
        let wt = worktree(&e.b, "TASK-002");
        assert_eq!(
            e.f.crudo(&wt, &["rev-parse", "--abbrev-ref", "HEAD"])
                .trim(),
            RAMA_2
        );
        assert_eq!(
            e.f.crudo(&e.b, &["rev-parse", RAMA_2]),
            e.f.crudo(&e.b, &["rev-parse", &format!("origin/{RAMA_2}")])
        );
        // Reanudar con o sin solape no publica nada.
        let punta = e.f.crudo(&e.bare, &["rev-parse", estado::RAMA]);
        let (out, _) = start(&e.f, &e.b, "TASK-002");
        assert_eq!(out, e.ultima.0);
        assert_eq!(e.f.crudo(&e.bare, &["rev-parse", estado::RAMA]), punta);
    }
}

/// C6: una tarea en curso sin `origin/<rama>` (ana empieza TASK-003 y no la
/// publica, aunque en su clon toque el mismo fichero) no da aviso y sale en
/// `solape-sin-rama`.
#[test]
fn c6_en_curso_sin_rama_publicada_no_avisa() {
    let e = escena("solape-c6", &["web/z.ts"], &["src/x/a.rs"], |f, a| {
        start(f, a, "TASK-003");
        tocar(f, a, "TASK-003", RAMA_3, &["src/x/a.rs"], false);
    });
    let (out, err) = &e.ultima;
    assert_eq!(valor(out, "solape-sin-rama"), Some("TASK-003"), "{out}");
    assert_eq!(valor(out, "solape"), None, "{out}");
    assert!(lineas(err, "aviso:").is_empty(), "{err}");
    // En la primera tambien: no se sabe que toca.
    assert_eq!(
        valor(&e.primera.0, "solape-sin-rama"),
        Some("TASK-003"),
        "{}",
        e.primera.0
    );
    assert!(e.a.join(".taskcodetwo").exists());
}

/// C7: una tarea `terminada` cuya rama conservada toca el mismo fichero no
/// da aviso ni claves.
#[test]
fn c7_terminada_con_rama_conservada_no_avisa() {
    let e = escena("solape-c7", &["src/x/a.rs"], &["src/x/a.rs"], |f, a| {
        editar(
            f,
            a,
            "TASK-001",
            "estado: \"en-curso\"",
            "estado: \"terminada\"",
        );
    });
    let (out, err) = &e.ultima;
    // La rama de TASK-001 sigue en el remoto.
    assert!(!e.f.crudo(&e.bare, &["rev-parse", RAMA_1]).trim().is_empty());
    assert!(claves_solape(out).is_empty(), "{out}");
    assert!(err.is_empty(), "{err}");
    assert!(claves_solape(&e.primera.0).is_empty(), "{}", e.primera.0);
    assert!(e.primera.1.is_empty(), "{}", e.primera.1);
}

/// C8: `Cargo.toml` frente a `README.md`, los dos en la raiz, no avisa (sin
/// modulo); el mismo `Cargo.toml` en las dos, aviso de fichero.
#[test]
fn c8_ficheros_de_la_raiz_sin_modulo() {
    let e = escena("solape-c8-no", &["Cargo.toml"], &["README.md"], nada);
    let (out, err) = &e.ultima;
    assert!(claves_solape(out).is_empty(), "{out}");
    assert!(lineas(err, "aviso:").is_empty(), "{err}");
    let e = escena("solape-c8-si", &["Cargo.toml"], &["Cargo.toml"], nada);
    let (out, err) = &e.ultima;
    assert_eq!(
        valor(out, "solape-ficheros"),
        Some("TASK-001:Cargo.toml"),
        "{out}"
    );
    assert_eq!(valor(out, "solape-modulos"), None, "{out}");
    assert_eq!(lineas(err, "aviso:").len(), 1, "{err}");
}

/// C9: en el primer `start` (rama propia sin commits) sale una linea `nota:`
/// (no `aviso:`) con las tareas en curso con rama publicada, su @ y como
/// mucho 5 de sus modulos, y `en-curso-modulos=` en la linea.
#[test]
fn c9_primer_start_nota_con_lo_que_tocan_las_demas() {
    let de_ana = [
        "m1/a.rs", "m2/a.rs", "m3/a.rs", "m4/a.rs", "m5/a.rs", "m6/a.rs", "raiz.md",
    ];
    let e = escena("solape-c9", &de_ana, &["src/x/a.rs"], nada);
    let (out, err) = &e.primera;
    assert!(out.starts_with("start ok id=TASK-002 "), "{out}");
    assert!(lineas(err, "aviso:").is_empty(), "{err}");
    let notas = lineas(err, "nota:");
    assert_eq!(notas.len(), 1, "{err}");
    assert_eq!(err.lines().count(), 1, "{err}");
    assert!(
        notas[0].ends_with("TASK-001 (@ana: m1, m2, m3, m4, m5)"),
        "{err}"
    );
    assert_eq!(
        valor(out, "en-curso-modulos"),
        Some("TASK-001:m1,TASK-001:m2,TASK-001:m3,TASK-001:m4,TASK-001:m5"),
        "{out}"
    );
    assert_eq!(valor(out, "solape"), None, "{out}");
    // Ya con cambios propios, no hay nota ni `en-curso-modulos`.
    let (out, err) = &e.ultima;
    assert!(lineas(err, "nota:").is_empty(), "{err}");
    assert_eq!(valor(out, "en-curso-modulos"), None, "{out}");
}

/// Las tareas de la escena: TASK-001 (ana, publicada), TASK-003 (ana,
/// publicada en C10) y una en curso sin rama en el remoto.
fn en_curso() -> Vec<EnCurso> {
    let t = |id: &str, quien: &str, rama: &str| EnCurso {
        id: id.to_owned(),
        quien: quien.to_owned(),
        rama: Some(rama.to_owned()),
    };
    vec![
        t("TASK-001", "@ana", RAMA_1),
        t("TASK-002", "@luis", RAMA_2),
        t("TASK-003", "@ana", RAMA_3),
        t("TASK-009", "@beto", "feature/task-009-no-publicada"),
    ]
}

/// C10: el solape lanza exactamente 1 + k + 1 procesos de git (k = otras
/// tareas en curso con rama publicada): un `for-each-ref` de las ramas de
/// `origin`, el `diff` de la propia y uno por cada publicada. Sin otra
/// tarea en curso, ninguno.
#[test]
fn c10_uno_mas_k_mas_uno_procesos() {
    let e = escena("solape-c10", &["src/x/a.rs"], &["src/x/a.rs"], |f, a| {
        start(f, a, "TASK-003");
        tocar(f, a, "TASK-003", RAMA_3, &["web/z.ts"], true);
    });
    let g = tc2::git::Git::nuevo().con_entorno(entorno_ssh(&e.f));
    let s = solape::calcular(&g, &e.b, "TASK-002", RAMA_2, &en_curso()).unwrap();
    let r = g.registro();
    let k = 2;
    assert_eq!(r.len(), 1 + k + 1, "{r:#?}");
    assert!(
        r[0].starts_with("git for-each-ref ") && r[0].ends_with(" refs/remotes/origin/"),
        "{r:#?}"
    );
    for l in &r[1..] {
        assert!(
            l.starts_with("git diff --name-only -z --no-renames origin/develop..."),
            "{r:#?}"
        );
    }
    assert!(
        r[1].ends_with(&format!("...refs/heads/{RAMA_2} --")),
        "{r:#?}"
    );
    assert_eq!(
        s.cruces.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
        ["TASK-001"]
    );
    assert_eq!(s.sin_rama, ["TASK-009"]);
    // Sin otra tarea en curso, ningun proceso.
    let g = tc2::git::Git::nuevo().con_entorno(entorno_ssh(&e.f));
    let solo: Vec<EnCurso> = en_curso()
        .into_iter()
        .filter(|t| t.id == "TASK-002")
        .collect();
    assert_eq!(solape::calcular(&g, &e.b, "TASK-002", RAMA_2, &solo), None);
    assert!(g.registro().is_empty(), "{:#?}", g.registro());
}

/// C5: lanzado por la biblioteca, tras el `fetch` de `start` el solape solo
/// lanza `cat-file`, `for-each-ref` y `diff` (nunca `fetch`, `ls-remote` ni
/// `push`); un repo sin `origin` no da claves `solape*` (un solo
/// `for-each-ref`, sin `diff`).
#[test]
fn c5_sin_red_y_sin_origin_sin_claves() {
    let e = escena("solape-c5", &["src/x/a.rs"], &["src/x/a.rs"], nada);
    let g = tc2::git::Git::nuevo().con_entorno(entorno_ssh(&e.f));
    let luis = || -> Result<Quien, tc2::contrato::Informe> {
        Ok(Quien::Persona(Cuenta {
            proveedor: Proveedor::GitHub,
            id: 1002,
            login: "luis".to_owned(),
        }))
    };
    let entorno = tc2::toma::Entorno {
        dir: &e.b,
        git: &g,
        quien: &luis,
        catalogo: &sin_catalogo,
        ahora: tc2::identidad::ahora(),
    };
    let (i, s) = tc2::toma::start_con_solape(&entorno, "TASK-002");
    assert_eq!(i.resultado, Resultado::SinCambios, "{}", i.linea());
    assert_eq!(i.valor("asignado"), Some("luis"));
    assert!(s.is_some_and(|s| !s.cruces.is_empty()));
    let r = g.registro();
    let fetch = r.iter().position(|l| l.starts_with("git fetch ")).unwrap();
    // Desde la lectura de los `tarea.md` del solape hasta el final.
    let desde = r
        .iter()
        .rposition(|l| l.starts_with("git cat-file "))
        .unwrap();
    assert!(fetch < desde, "{r:#?}");
    let solape = &r[desde..];
    assert!(solape.len() >= 3, "{r:#?}");
    for l in solape {
        let sub = l.split(' ').nth(1).unwrap_or_default();
        assert!(
            ["cat-file", "for-each-ref", "diff"].contains(&sub),
            "{l}: {r:#?}"
        );
    }
    comun::registro_permitido(&r);
    // Sin `origin`: sin claves y sin `diff`.
    let c = e.f.otro_clon(&e.bare, "sin-origin");
    e.f.crudo(&c, &["branch", "-q", RAMA_2, "origin/develop"]);
    e.f.crudo(&c, &["remote", "remove", "origin"]);
    let g = tc2::git::Git::nuevo().con_entorno(entorno_ssh(&e.f));
    assert_eq!(
        solape::calcular(&g, &c, "TASK-002", RAMA_2, &en_curso()),
        None
    );
    let r = g.registro();
    assert_eq!(r.len(), 1, "{r:#?}");
    assert!(r[0].starts_with("git for-each-ref "), "{r:#?}");
}
