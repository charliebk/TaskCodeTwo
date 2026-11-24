//! `tc2 init` (TASK-028, F3-1): un test (o mas) por criterio, con git real
//! en carpetas temporales y GitHub simulado en 127.0.0.1. Nada contra la
//! red real: `origin` apunta a un host inventado y su transporte ssh se
//! sustituye con `GIT_SSH_COMMAND` (el remoto bare de la fixture, o un
//! «ssh» que no conecta). El GitHub simulado es `comun::http_mock`.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use comun::http_mock::{Mock, Recibida, Simulada, serie};
use comun::{Fixture, escribir};
use tc2::contrato::Resultado;
use tc2::git::Git;
use tc2::i18n::Idioma;
use tc2::init::{Contexto, Decision, Hecho, Opciones, init};
use tc2::plataforma::{Credenciales, CredencialesFijas, Rol, Secreto};

/// Rutas de GitHub que lee `init`: la persona y los rulesets (ninguno).
fn github() -> Mock {
    let m = Mock::nuevo();
    m.responder(
        "GET",
        "/user",
        Simulada::json(200, "{\"login\":\"carlos\",\"id\":1001,\"type\":\"User\"}"),
    );
    m.responder("GET", "/repos/o/r/rulesets", Simulada::json(200, "[]"));
    m.responder(
        "GET",
        "/repos/o/r/rules/branches/develop",
        Simulada::json(200, "[]"),
    );
    m
}

/// `origin` en un host que no es github.com: la API sale de la clave
/// `plataforma` de la configuracion (el servidor simulado).
const ORIGIN: &str = "ssh://git@git.ejemplo.test/o/r.git";

fn config_con_plataforma(api: &str) -> String {
    format!("---\nformato: 2\nplataforma:\n  proveedor: \"github\"\n  api: \"{api}\"\n")
}

/// Puerto Git de la fixture cuyo «ssh» es `ssh_command`.
fn git_ssh(f: &Fixture, ssh_command: String) -> Git {
    let mut e = f.entorno();
    e.push(("GIT_SSH_COMMAND".into(), ssh_command.into()));
    e.push(("GIT_SSH_VARIANT".into(), "simple".into()));
    Git::nuevo().con_entorno(e)
}

/// El «ssh» sirve el remoto bare `bare` (lectura y escritura).
fn git_con_ssh(f: &Fixture, bare: &Path) -> Git {
    let bare = bare.to_string_lossy().replace('\\', "/");
    git_ssh(
        f,
        format!(
            "sh -c 'case \"$2\" in *receive-pack*) exec git receive-pack \"$0\" ;; *) exec git upload-pack \"$0\" ;; esac' '{bare}'"
        ),
    )
}

/// El «ssh» no llega al host, como sin red.
fn git_sin_red(f: &Fixture) -> Git {
    git_ssh(
        f,
        "sh -c 'echo \"ssh: connect to host git.ejemplo.test port 22: Connection refused\" >&2; exit 255'"
            .to_owned(),
    )
}

/// `git init` sin remoto ni commits (HEAD sin nacer en `develop`).
fn repo_vacio(f: &Fixture) -> PathBuf {
    f.crudo(&f.raiz, &["init", "-q", "-b", "develop", "repo"]);
    f.raiz.join("repo")
}

/// Remoto bare vacio, sin rechazar nada que no sea force.
fn bare(f: &Fixture) -> PathBuf {
    f.crudo(&f.raiz, &["init", "-q", "--bare", "remoto.git"]);
    let b = f.raiz.join("remoto.git");
    f.crudo(&b, &["config", "receive.denyNonFastForwards", "true"]);
    b
}

fn sin_token() -> Box<dyn Credenciales> {
    Box::new(CredencialesFijas::nuevas())
}

fn con_token() -> Box<dyn Credenciales> {
    Box::new(CredencialesFijas::nuevas().con(Rol::Persona, Secreto::nuevo("tok-persona")))
}

fn correr(
    dir: &Path,
    g: &Git,
    simular: bool,
    credenciales: &dyn Fn() -> Box<dyn Credenciales>,
) -> Hecho {
    let c = Contexto {
        dir,
        git: g,
        credenciales,
    };
    init(
        Opciones {
            simular,
            aplicar_puertas: false,
        },
        &c,
    )
}

fn stdout(h: &Hecho) -> String {
    h.salida_en(false, Idioma::Es).stdout
}

/// `(ruta, clase, decision, motivo)` de las acciones de `clase`.
fn de_clase(h: &Hecho, clase: &str) -> Vec<(String, &'static str, Option<String>)> {
    h.acciones
        .iter()
        .filter(|a| a.clase == clase)
        .map(|a| (a.ruta.clone(), a.decision.texto(), a.motivo.clone()))
        .collect()
}

/// Lo que no debe cambiar en una segunda ejecucion: refs, worktrees,
/// `exclude` y el arbol (hashes) de cada worktree.
fn huella_repo(f: &Fixture, dir: &Path) -> String {
    let mut h = f.crudo(dir, &["for-each-ref", "--format=%(refname) %(objectname)"]);
    let wts = f.crudo(dir, &["worktree", "list", "--porcelain"]);
    h += &wts;
    h += &std::fs::read_to_string(dir.join(".git").join("info").join("exclude")).unwrap();
    for l in wts.lines() {
        if let Some(r) = l.strip_prefix("worktree ") {
            h += &f.crudo(Path::new(r), &["status", "--porcelain", "--ignored"]);
            h += &f.crudo(Path::new(r), &["ls-files", "-s"]);
        }
    }
    h
}

/// C1: en un repo vacio y sin remoto, `init ok`: la rama de estado montada
/// en `tareas/`, `main` y `develop` con un commit raiz que lleva la
/// configuracion minima (`config::leer` = `Config::suelo()`) y `.gitignore`,
/// y las dos lineas en `info/exclude`.
#[test]
fn repo_vacio_sin_remoto() {
    let f = Fixture::nueva("init-c1");
    let repo = repo_vacio(&f);
    let g = f.git();
    let h = correr(&repo, &g, false, &sin_token);
    let s = h.salida_en(false, Idioma::Es);
    assert_eq!(s.codigo, 0, "{}{}", s.stdout, s.stderr);
    assert!(
        s.stdout.lines().last().unwrap().starts_with("init ok "),
        "{}",
        s.stdout
    );
    // Rama de estado montada en `tareas/`.
    let wts = f.crudo(&repo, &["worktree", "list", "--porcelain"]);
    assert!(
        wts.contains("/repo/tareas\n") && wts.contains("branch refs/heads/taskcodetwo/tareas"),
        "{wts}"
    );
    assert!(repo.join("tareas").join("LEEME.md").is_file());
    // `main` y `develop` en el mismo commit raiz, con los dos ficheros.
    let main = f.crudo(&repo, &["rev-parse", "main"]);
    assert_eq!(main, f.crudo(&repo, &["rev-parse", "develop"]));
    assert_eq!(
        f.crudo(&repo, &["rev-list", "--count", "develop"]).trim(),
        "1"
    );
    assert_eq!(
        f.crudo(&repo, &["ls-tree", "-r", "--name-only", "develop"]),
        ".gitignore\n.taskcode/config.yml\n"
    );
    let cfg = f.crudo(&repo, &["show", "develop:.taskcode/config.yml"]);
    assert_eq!(
        tc2::config::leer(&cfg),
        Ok(tc2::config::Config::suelo()),
        "{cfg}"
    );
    let gi = f.crudo(&repo, &["show", "develop:.gitignore"]);
    assert!(
        gi.lines().any(|l| l == "/tareas/") && gi.lines().any(|l| l == "/.taskcodetwo/"),
        "{gi}"
    );
    let ex = std::fs::read_to_string(repo.join(".git").join("info").join("exclude")).unwrap();
    for l in ["/tareas/", "/.taskcodetwo/worktrees/"] {
        assert!(ex.lines().any(|x| x == l), "{l}: {ex}");
    }
    // HEAD en `develop` y nada pendiente en el worktree.
    assert_eq!(
        f.crudo(&repo, &["symbolic-ref", "HEAD"]).trim(),
        "refs/heads/develop"
    );
    assert_eq!(f.crudo(&repo, &["status", "--porcelain"]), "");
    // Sin remoto: personas y puertas pendientes; el workflow, ranura.
    assert_eq!(de_clase(&h, "personas")[0].2.as_deref(), Some("sin-remoto"));
    assert_eq!(de_clase(&h, "workflow")[0].1, "pendiente", "{}", stdout(&h));
    comun::registro_permitido(&g.registro());
}

/// C2: la segunda ejecucion sale `init sin-cambios`: lo que la primera creo
/// es ahora `existe` (lo demas, igual), y refs, worktrees, `exclude` y el
/// arbol no cambian.
#[test]
fn init_dos_veces_sin_cambios() {
    let f = Fixture::nueva("init-c2");
    let repo = repo_vacio(&f);
    let g = f.git();
    let primera = correr(&repo, &g, false, &sin_token);
    assert_eq!(
        primera.informe.resultado,
        Resultado::Ok,
        "{}",
        stdout(&primera)
    );
    let antes = huella_repo(&f, &repo);
    let ficheros = comun::instantanea(&repo, &[]);
    let segunda = correr(&repo, &g, false, &sin_token);
    assert_eq!(
        segunda.informe.resultado,
        Resultado::SinCambios,
        "{}",
        stdout(&segunda)
    );
    assert!(
        stdout(&segunda)
            .lines()
            .last()
            .unwrap()
            .starts_with("init sin-cambios crea=0 "),
        "{}",
        stdout(&segunda)
    );
    assert_eq!(primera.acciones.len(), segunda.acciones.len());
    for (a, b) in primera.acciones.iter().zip(&segunda.acciones) {
        assert_eq!((&a.ruta, a.clase), (&b.ruta, b.clase));
        if a.decision == Decision::Crea {
            assert_eq!(b.decision, Decision::Existe, "{}", b.linea());
        } else {
            assert_eq!(a.linea(), b.linea());
        }
    }
    assert!(
        segunda
            .acciones
            .iter()
            .all(|a| matches!(a.decision, Decision::Existe | Decision::Pendiente)),
        "{}",
        stdout(&segunda)
    );
    assert_eq!(antes, huella_repo(&f, &repo));
    assert_eq!(ficheros, comun::instantanea(&repo, &[]));
}

/// C3: `--simular` (y `--simular --json`) imprime el plan y `simular=si`
/// sin cambiar ni un byte (ni crea `.git/taskcodetwo/`), y el plan simulado
/// es igual a las acciones de la ejecucion real de despues.
#[test]
fn simular_no_toca_nada_y_coincide() {
    let f = Fixture::nueva("init-c3");
    let repo = repo_vacio(&f);
    let g = f.git();
    let antes = comun::instantanea(&repo, &[]);
    let sim = correr(&repo, &g, true, &sin_token);
    let texto = stdout(&sim);
    let fin = texto.lines().last().unwrap();
    assert!(fin.starts_with("init ok simular=si crea="), "{texto}");
    let json = sim.salida_en(true, Idioma::Es).stdout;
    for l in json.lines() {
        let v: serde_json::Value = serde_json::from_str(l).expect(l);
        assert!(v.get("accion").is_some() || v["comando"] == "init", "{l}");
    }
    assert!(json.contains("\"simular\":\"si\""), "{json}");
    assert_eq!(
        antes,
        comun::instantanea(&repo, &[]),
        "--simular cambio algo"
    );
    assert!(!repo.join(".git").join("taskcodetwo").exists());
    let real = correr(&repo, &g, false, &sin_token);
    assert_eq!(real.informe.resultado, Resultado::Ok, "{}", stdout(&real));
    let lineas = |h: &Hecho| h.acciones.iter().map(|a| a.linea()).collect::<Vec<_>>();
    assert_eq!(lineas(&sim), lineas(&real));
}

/// C4: un proyecto de la 0.16 (config sin `formato: 2`, o `tareas/` como
/// carpeta normal) sale `init rechazada motivo=proyecto-016` con el arreglo,
/// codigo 3, sin escribir nada.
#[test]
fn init_en_repo_016_rechaza() {
    for caso in ["config", "tareas"] {
        let f = Fixture::nueva("init-c4-016");
        let repo = repo_vacio(&f);
        if caso == "config" {
            escribir(&repo, ".taskcode/config.yml", "modo: automatico\n");
        } else {
            escribir(&repo, "tareas/00-planificadas/TASK-001/tarea.md", "x\n");
        }
        let antes = comun::instantanea(&repo, &[]);
        let h = correr(&repo, &f.git(), false, &sin_token);
        let s = h.salida_en(false, Idioma::Es);
        assert_eq!(s.codigo, 3, "{caso}: {}", s.stdout);
        let fin = s.stdout.lines().last().unwrap();
        assert!(
            fin.starts_with(
                "init rechazada motivo=proyecto-016 arreglo=tc2%20actualizar%20--simular "
            ),
            "{caso}: {fin}"
        );
        assert!(
            s.stderr.contains("tc2 actualizar --simular"),
            "{}",
            s.stderr
        );
        assert!(
            h.acciones.iter().all(|a| a.decision != Decision::Crea),
            "{caso}: {}",
            s.stdout
        );
        assert_eq!(
            antes,
            comun::instantanea(&repo, &[]),
            "{caso}: escribio algo"
        );
        let en = h.salida_en(false, Idioma::En);
        assert_eq!(en.stdout, s.stdout);
        assert_ne!(en.stderr, s.stderr);
    }
}

/// C4: una configuracion de un formato mas nuevo o que no vale tampoco se
/// pisa: `rechazada` con su motivo y nada escrito.
#[test]
fn init_con_config_invalida_no_la_pisa() {
    for (texto, motivo) in [
        ("---\nformato: 3\n", "formato-nuevo"),
        ("---\nformato: 2\nno_existe: 1\n", "config-invalida"),
    ] {
        let f = Fixture::nueva("init-c4-invalida");
        let repo = repo_vacio(&f);
        escribir(&repo, ".taskcode/config.yml", texto);
        let antes = comun::instantanea(&repo, &[]);
        let h = correr(&repo, &f.git(), false, &sin_token);
        assert_eq!(h.informe.resultado, Resultado::Rechazada, "{}", stdout(&h));
        assert_eq!(h.informe.valor("motivo"), Some(motivo));
        assert_eq!(antes, comun::instantanea(&repo, &[]));
        assert_eq!(
            std::fs::read_to_string(repo.join(".taskcode").join("config.yml")).unwrap(),
            texto
        );
    }
}

/// C5 y C6: sin remoto, `init ok` con `personas.yml` y las puertas
/// `pendiente motivo=sin-remoto`; tras `git remote add origin`, un segundo
/// `init` escribe `personas.yml` con la persona del token y propone una
/// linea por ruleset `taskcodetwo-*` (`persona`, con la orden que lo
/// aplica) sin ninguna escritura en la plataforma.
#[test]
fn sin_remoto_completa_al_anadirlo() {
    let _serie = serie();
    let s = github();
    let f = Fixture::nueva("init-c5");
    let repo = repo_vacio(&f);
    escribir(
        &repo,
        ".taskcode/config.yml",
        &config_con_plataforma(&s.base()),
    );
    let b = bare(&f);
    let g = git_con_ssh(&f, &b);
    let primera = correr(&repo, &g, false, &con_token);
    assert_eq!(
        primera.informe.resultado,
        Resultado::Ok,
        "{}",
        stdout(&primera)
    );
    let pendientes = |h: &Hecho, clase: &str| {
        de_clase(h, clase)
            .into_iter()
            .map(|(_, d, m)| (d, m.unwrap_or_default()))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        pendientes(&primera, "personas"),
        [("pendiente", "sin-remoto".to_owned())]
    );
    assert_eq!(
        pendientes(&primera, "ruleset"),
        vec![("pendiente", "sin-remoto".to_owned()); 5]
    );
    assert!(s.recibidas().is_empty(), "{:?}", s.recibidas());

    f.crudo(&repo, &["remote", "add", "origin", ORIGIN]);
    let segunda = correr(&repo, &g, false, &con_token);
    let texto = stdout(&segunda);
    assert_eq!(segunda.informe.resultado, Resultado::Ok, "{texto}");
    assert_eq!(pendientes(&segunda, "personas"), [("crea", String::new())]);
    let personas = std::fs::read_to_string(repo.join(".taskcode").join("personas.yml")).unwrap();
    let lista = tc2::plataforma::Personas::leer(&personas).unwrap();
    assert_eq!(
        lista
            .entradas()
            .iter()
            .map(|p| (p.id, p.login.as_str()))
            .collect::<Vec<_>>(),
        [(1001, "carlos")]
    );
    // C6: una linea por ruleset, de la persona, con la orden; ninguna
    // escritura en la plataforma.
    let rulesets: Vec<&str> = texto
        .lines()
        .filter(|l| l.contains(" clase=ruleset "))
        .collect();
    assert_eq!(rulesets.len(), 5, "{texto}");
    for (l, n) in rulesets.iter().zip([
        "develop",
        "main",
        "ramas-de-tarea",
        "tareas",
        "tags-version",
    ]) {
        assert_eq!(
            *l,
            format!(
                "accion ruta=taskcodetwo-{n} clase=ruleset dueno=taskcodetwo decision=persona motivo=aplicar-con-task-118 orden=tc2%20init%20--aplicar-puertas"
            )
        );
    }
    let json = segunda.salida_en(true, Idioma::Es).stdout;
    let develop: serde_json::Value = serde_json::from_str(
        json.lines()
            .find(|l| l.contains("\"taskcodetwo-develop\""))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        develop["accion"]["propuesta"]["name"],
        "taskcodetwo-develop"
    );
    let peticiones: Vec<String> = s.recibidas().into_iter().map(|r| r.linea).collect();
    assert!(!peticiones.is_empty());
    assert!(
        peticiones.iter().all(|p| p.starts_with("GET ")),
        "escrituras en la plataforma: {peticiones:?}"
    );
    assert!(
        peticiones.contains(&"GET /user".to_owned()),
        "{peticiones:?}"
    );
    // Las ramas de codigo ya tienen historia: publicarlas es de la persona.
    assert_eq!(
        pendientes(&segunda, "rama"),
        vec![("persona", "sin-publicar".to_owned()); 2]
    );
    // Y un tercer `init` ya no crea nada.
    let tercera = correr(&repo, &g, false, &con_token);
    assert_eq!(
        tercera.informe.resultado,
        Resultado::SinCambios,
        "{}",
        stdout(&tercera)
    );
}

/// C5: con `origin` y sin red, `init sinred pendiente=no` (codigo 4) sin
/// escribir nada.
#[test]
fn sin_red_no_escribe() {
    let f = Fixture::nueva("init-sinred");
    let repo = repo_vacio(&f);
    f.crudo(&repo, &["remote", "add", "origin", ORIGIN]);
    let antes = comun::instantanea(&repo, &[]);
    for simular in [false, true] {
        let h = correr(&repo, &git_sin_red(&f), simular, &con_token);
        let s = h.salida_en(false, Idioma::Es);
        assert_eq!(s.codigo, 4, "{}", s.stdout);
        assert_eq!(s.stdout, "init sinred pendiente=no\n");
        assert!(s.stderr.contains("Connection refused"), "{}", s.stderr);
        assert_eq!(antes, comun::instantanea(&repo, &[]));
    }
}

/// En un repo vacio con remoto, el commit raiz y las tres ramas se
/// publican sin force, y la rama de estado queda en el remoto.
#[test]
fn repo_vacio_con_remoto_publica() {
    let _serie = serie();
    let s = github();
    let f = Fixture::nueva("init-remoto");
    let repo = repo_vacio(&f);
    escribir(
        &repo,
        ".taskcode/config.yml",
        &config_con_plataforma(&s.base()),
    );
    let b = bare(&f);
    f.crudo(&repo, &["remote", "add", "origin", ORIGIN]);
    let g = git_con_ssh(&f, &b);
    let h = correr(&repo, &g, false, &con_token);
    assert_eq!(h.informe.resultado, Resultado::Ok, "{}", stdout(&h));
    let remoto = f.crudo(&b, &["for-each-ref", "--format=%(refname)"]);
    assert_eq!(
        remoto,
        "refs/heads/develop\nrefs/heads/main\nrefs/heads/taskcodetwo/tareas\n"
    );
    assert_eq!(
        f.crudo(&b, &["rev-parse", "develop"]),
        f.crudo(&repo, &["rev-parse", "develop"])
    );
    comun::registro_permitido(&g.registro());
}

/// HEAD sin nacer en otra rama (`master`): `main` y `develop` se crean, HEAD
/// se queda donde estaba y la linea `HEAD` da a la persona la orden que la
/// pasa a `develop` (`git switch` se negaria con los ficheros en el disco).
#[test]
fn head_en_master_queda_para_la_persona() {
    let f = Fixture::nueva("init-master");
    f.crudo(&f.raiz, &["init", "-q", "-b", "master", "repo"]);
    let repo = f.raiz.join("repo");
    let h = correr(&repo, &f.git(), false, &sin_token);
    assert_eq!(h.informe.resultado, Resultado::Ok, "{}", stdout(&h));
    let head = de_clase(&h, "head");
    assert_eq!(
        head,
        [(
            "HEAD".to_owned(),
            "persona",
            Some("head-en-master".to_owned())
        )]
    );
    let orden = h
        .acciones
        .iter()
        .find(|a| a.clase == "head")
        .and_then(|a| a.orden.clone())
        .unwrap();
    assert_eq!(
        f.crudo(&repo, &["symbolic-ref", "HEAD"]).trim(),
        "refs/heads/master"
    );
    // La orden de la linea deja HEAD en `develop` y el worktree limpio.
    for parte in orden.split(" && ") {
        let args: Vec<&str> = parte.split(' ').skip(1).collect();
        f.crudo(&repo, &args);
    }
    assert_eq!(f.crudo(&repo, &["status", "--porcelain"]), "");
    assert_eq!(
        f.crudo(&repo, &["rev-parse", "HEAD"]),
        f.crudo(&repo, &["rev-parse", "main"])
    );
}

/// Con historia, `main` o `develop` que faltan los crea la persona (D3),
/// desde la otra rama si existe.
#[test]
fn con_historia_las_ramas_que_faltan_son_de_la_persona() {
    let f = Fixture::nueva("init-historia");
    let repo = repo_vacio(&f);
    escribir(&repo, "README.md", "x\n");
    f.crudo(&repo, &["add", "README.md"]);
    f.crudo(&repo, &["commit", "-q", "-m", "inicial"]);
    let h = correr(&repo, &f.git(), false, &sin_token);
    assert_eq!(h.informe.resultado, Resultado::Ok, "{}", stdout(&h));
    let main = h
        .acciones
        .iter()
        .find(|a| a.ruta == "refs/heads/main")
        .unwrap();
    assert_eq!(
        main.linea(),
        "accion ruta=refs/heads/main clase=rama dueno=proyecto decision=persona motivo=con-historia orden=git%20branch%20main%20develop"
    );
    assert!(
        f.crudo(&repo, &["branch", "--list", "main"]).is_empty(),
        "init creo main con historia"
    );
    assert_eq!(
        f.crudo(&repo, &["rev-list", "--count", "develop"]).trim(),
        "1"
    );
}

/// Un fallo al aplicar (commit sin identidad) sale `init error` con la ruta;
/// lo de detras queda `pendiente motivo=no-aplicada` y, arreglada la causa,
/// repetir `init` termina el trabajo.
#[test]
fn fallo_al_aplicar_y_repetir_termina() {
    let f = Fixture::nueva("init-fallo");
    let repo = repo_vacio(&f);
    let sin_usuario = f.raiz.join("gitconfig-sin-usuario");
    std::fs::write(
        &sin_usuario,
        "[user]\n\tuseConfigOnly = true\n[core]\n\tautocrlf = false\n",
    )
    .unwrap();
    let g = Git::nuevo().con_entorno(vec![
        ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
        ("GIT_CONFIG_GLOBAL".into(), sin_usuario.into_os_string()),
    ]);
    let h = correr(&repo, &g, false, &sin_token);
    let s = h.salida_en(false, Idioma::Es);
    assert_eq!(s.codigo, 1, "{}", s.stdout);
    assert_eq!(h.informe.valor("ruta"), Some("refs/heads/main"));
    assert!(
        s.stderr.starts_with("no se pudo crear refs/heads/main"),
        "{}",
        s.stderr
    );
    let decision = |ruta_acaba: &str| {
        h.acciones
            .iter()
            .find(|a| a.ruta.replace('\\', "/").ends_with(ruta_acaba))
            .map(|a| (a.decision.texto(), a.motivo.clone().unwrap_or_default()))
            .unwrap()
    };
    assert_eq!(
        decision("refs/heads/main"),
        ("pendiente", "error".to_owned())
    );
    assert_eq!(decision("/tareas"), ("pendiente", "no-aplicada".to_owned()));
    assert!(!repo.join("tareas").exists());
    // Con identidad, el segundo `init` termina sin pisar lo ya creado.
    let h = correr(&repo, &f.git(), false, &sin_token);
    assert_eq!(h.informe.resultado, Resultado::Ok, "{}", stdout(&h));
    assert_eq!(de_clase(&h, "config")[0].1, "existe");
    assert_eq!(de_clase(&h, "rama")[0].1, "crea");
    assert_eq!(f.crudo(&repo, &["status", "--porcelain"]), "");
}

/// Remoto bare con un commit en `rama` (y nada mas): un proyecto que ya
/// existe en la plataforma.
fn bare_con_rama(f: &Fixture, rama: &str) -> PathBuf {
    let b = bare(f);
    f.crudo(&f.raiz, &["init", "-q", "-b", rama, "semilla"]);
    let semilla = f.raiz.join("semilla");
    escribir(&semilla, "README.md", "proyecto\n");
    f.crudo(&semilla, &["add", "README.md"]);
    f.crudo(&semilla, &["commit", "-q", "-m", "inicial"]);
    f.crudo(&semilla, &["push", "-q", b.to_str().unwrap(), rama]);
    b
}

/// Ramas de codigo del remoto (sin la de estado, que es huerfana).
fn ramas_de(f: &Fixture, b: &Path) -> String {
    f.crudo(b, &["for-each-ref", "--format=%(refname)", "refs/heads/"])
        .lines()
        .filter(|l| !l.contains("taskcodetwo/"))
        .map(|l| format!("{l}\n"))
        .collect()
}

/// Revision 1, IMP-1: repo local sin commits y `origin` que ya tiene
/// historia (`main` sin `develop`, o solo otra rama): no cuenta como vacio.
/// No se crea ningun commit raiz ni se publica nada en las ramas de codigo;
/// la rama que falta la crea la persona desde la que ya hay.
#[test]
fn remoto_con_historia_no_es_repo_vacio() {
    for (rama, ordenes) in [
        (
            "main",
            [
                "git fetch origin main && git branch main origin/main",
                "git fetch origin main && git branch develop origin/main && git push origin develop",
            ],
        ),
        (
            "trunk",
            [
                "git fetch origin trunk && git branch main origin/trunk && git push origin main",
                "git fetch origin trunk && git branch develop origin/trunk && git push origin develop",
            ],
        ),
    ] {
        let f = Fixture::nueva("init-imp1");
        let repo = repo_vacio(&f);
        let b = bare_con_rama(&f, rama);
        let antes = ramas_de(&f, &b);
        f.crudo(&repo, &["remote", "add", "origin", ORIGIN]);
        let g = git_con_ssh(&f, &b);
        let h = correr(&repo, &g, false, &sin_token);
        assert_eq!(h.informe.resultado, Resultado::Ok, "{rama}: {}", stdout(&h));
        let lineas: Vec<(String, Option<String>)> = h
            .acciones
            .iter()
            .filter(|a| a.clase == "rama")
            .map(|a| (a.decision.texto().to_owned(), a.orden.clone()))
            .collect();
        assert_eq!(
            lineas,
            ordenes
                .iter()
                .map(|o| ("persona".to_owned(), Some((*o).to_owned())))
                .collect::<Vec<_>>(),
            "{rama}: {}",
            stdout(&h)
        );
        // Ni ramas locales nuevas ni historia ajena publicada.
        assert_eq!(f.crudo(&repo, &["branch", "--list", "main", "develop"]), "");
        assert_eq!(ramas_de(&f, &b), antes, "{rama}");
        // La configuracion queda en el disco, y la linea dice que versionarla.
        let v = de_clase(&h, "versionar");
        assert_eq!(v.len(), 1, "{}", stdout(&h));
        assert_eq!(v[0].1, "persona");
    }
}

/// Clon con un commit propio (sin relacion con nada) y `origin` apuntando
/// a `b` por el «ssh» de la fixture.
fn clon_con_historia_propia(f: &Fixture, nombre: &str) -> PathBuf {
    f.crudo(&f.raiz, &["init", "-q", "-b", "master", nombre]);
    let repo = f.raiz.join(nombre);
    escribir(&repo, "propio.txt", "historia local\n");
    f.crudo(&repo, &["add", "propio.txt"]);
    f.crudo(&repo, &["commit", "-q", "-m", "historia local"]);
    f.crudo(&repo, &["remote", "add", "origin", ORIGIN]);
    repo
}

/// La rama `develop` del plan.
fn develop(h: &Hecho) -> &tc2::init::Accion {
    h.acciones
        .iter()
        .find(|a| a.ruta == "refs/heads/develop")
        .unwrap()
}

/// Ninguna orden del plan publica una rama de codigo.
fn sin_push(h: &Hecho) -> bool {
    !h.acciones
        .iter()
        .filter_map(|a| a.orden.as_deref())
        .any(|o| o.contains("git push"))
}

/// Revisiones 2 y 3, IMP-1, IMP-2 e IMP-3: clon con commits propios y el
/// `main` de `origin`.
/// - Sin traer `origin`, la relacion no se puede comprobar: `no-comprobado`,
///   sin push, y un aviso que pide `git fetch origin` (no afirma que no la
///   haya).
/// - Traido `origin/main`, sin nada en comun: `historia-sin-relacion`, sin
///   push. El aviso dice lo cierto: no se publica ninguna rama de codigo,
///   y la de estado si (y en el remoto esta).
/// - Con un clon de verdad (historia relacionada), la rama si se propone
///   publicar.
#[test]
fn historia_local_sin_relacion_no_propone_publicar() {
    let f = Fixture::nueva("init-imp1-r2");
    let b = bare_con_rama(&f, "main");
    let antes = ramas_de(&f, &b);

    let sin_traer = clon_con_historia_propia(&f, "sin-traer");
    let h = correr(&sin_traer, &git_con_ssh(&f, &b), true, &sin_token);
    let s = h.salida_en(false, Idioma::Es);
    assert_eq!(s.codigo, 0, "{}", s.stdout);
    assert_eq!(develop(&h).motivo.as_deref(), Some("no-comprobado"));
    assert_eq!(
        develop(&h).orden.as_deref(),
        Some("git fetch origin main && git branch develop origin/main")
    );
    assert!(sin_push(&h), "{}", s.stdout);
    assert!(
        s.stderr.contains("no se pudo comprobar")
            && s.stderr
                .contains("ejecuta git fetch origin y repite tc2 init"),
        "{}",
        s.stderr
    );
    assert!(!s.stderr.contains("nada en comun"), "{}", s.stderr);

    let repo = clon_con_historia_propia(&f, "repo");
    f.crudo(
        &repo,
        &[
            "fetch",
            "-q",
            b.to_str().unwrap(),
            "main:refs/remotes/origin/main",
        ],
    );
    let h = correr(&repo, &git_con_ssh(&f, &b), false, &sin_token);
    let s = h.salida_en(false, Idioma::Es);
    assert_eq!(s.codigo, 0, "{}", s.stdout);
    assert_eq!(develop(&h).decision, Decision::Persona);
    assert_eq!(develop(&h).motivo.as_deref(), Some("historia-sin-relacion"));
    assert_eq!(
        develop(&h).orden.as_deref(),
        Some("git fetch origin main && git branch develop origin/main")
    );
    assert!(sin_push(&h), "{}", s.stdout);
    assert!(
        s.stderr.contains("nada en comun con la de origin")
            && s.stderr.contains("ninguna rama de codigo")
            && s.stderr
                .contains("la rama de estado taskcodetwo/tareas, huerfana, si se publica"),
        "{}",
        s.stderr
    );
    let en = h.salida_en(false, Idioma::En).stderr;
    assert!(
        en.contains("any code branch") && en.contains("the state branch taskcodetwo/tareas"),
        "{en}"
    );
    // Lo que dice el aviso es lo que paso: las ramas de codigo del remoto no
    // cambian y la de estado si se publico.
    assert_eq!(ramas_de(&f, &b), antes);
    assert!(
        f.crudo(&b, &["for-each-ref", "--format=%(refname)"])
            .contains("refs/heads/taskcodetwo/tareas\n")
    );

    // Un clon de verdad: la misma rama, desde `origin/main` y publicada.
    f.crudo(&f.raiz, &["clone", "-q", b.to_str().unwrap(), "clon"]);
    let clon = f.raiz.join("clon");
    // El HEAD del bare apunta a `develop`, que no existe: sin checkout.
    f.crudo(&clon, &["checkout", "-q", "main"]);
    let h = correr(&clon, &f.git(), false, &sin_token);
    let develop = h
        .acciones
        .iter()
        .find(|a| a.ruta == "refs/heads/develop")
        .unwrap();
    assert_eq!(
        develop.motivo.as_deref(),
        Some("con-historia"),
        "{}",
        stdout(&h)
    );
    assert_eq!(
        develop.orden.as_deref(),
        Some("git fetch origin main && git branch develop origin/main && git push origin develop")
    );
    assert!(
        !h.salida_en(false, Idioma::Es)
            .stderr
            .contains("nada en comun")
    );
}

/// Revision 3, IMP-3: el commit de `origin/main` esta en el clon (traido por
/// sha) aunque falte `refs/remotes/origin/main`, y la historia local es su
/// ancestro: esta relacionada, y la rama que falta se propone publicar.
#[test]
fn ancestro_traido_por_sha_cuenta_como_relacionada() {
    let f = Fixture::nueva("init-imp3");
    let b = bare_con_rama(&f, "main");
    let a = f.crudo(&f.raiz.join("semilla"), &["rev-parse", "main"]);
    let a = a.trim();
    // `main` avanza en el remoto: B, hijo de A.
    let semilla = f.raiz.join("semilla");
    escribir(&semilla, "README.md", "proyecto 2\n");
    f.crudo(&semilla, &["commit", "-q", "-am", "B"]);
    f.crudo(&semilla, &["push", "-q", b.to_str().unwrap(), "main"]);
    let b_sha = f.crudo(&semilla, &["rev-parse", "main"]);
    let b_sha = b_sha.trim();

    f.crudo(&f.raiz, &["init", "-q", "-b", "master", "repo"]);
    let repo = f.raiz.join("repo");
    f.crudo(&repo, &["fetch", "-q", b.to_str().unwrap(), b_sha]);
    f.crudo(&repo, &["reset", "-q", "--hard", a]);
    f.crudo(&repo, &["remote", "add", "origin", ORIGIN]);
    assert!(
        f.crudo(&repo, &["for-each-ref", "refs/remotes/"])
            .is_empty(),
        "no debe haber refs remotas"
    );
    let h = correr(&repo, &git_con_ssh(&f, &b), true, &sin_token);
    let s = h.salida_en(false, Idioma::Es);
    assert_eq!(
        develop(&h).motivo.as_deref(),
        Some("con-historia"),
        "{}",
        s.stdout
    );
    assert_eq!(
        develop(&h).orden.as_deref(),
        Some("git fetch origin main && git branch develop origin/main && git push origin develop")
    );
    assert!(!s.stderr.contains("aviso: "), "{}", s.stderr);
}

/// Revision 3, MEN-7 y MEN-8: una rama local que no esta en `origin`, con
/// historia sin relacion, sale `persona` con la orden que trae el remoto y
/// comprueba si tienen algo en comun; ninguna linea `persona` va sin orden.
#[test]
fn rama_local_sin_relacion_lleva_su_orden() {
    let f = Fixture::nueva("init-men7");
    let b = bare_con_rama(&f, "main");
    let repo = clon_con_historia_propia(&f, "repo");
    f.crudo(&repo, &["branch", "develop"]);
    f.crudo(
        &repo,
        &[
            "fetch",
            "-q",
            b.to_str().unwrap(),
            "main:refs/remotes/origin/main",
        ],
    );
    let h = correr(&repo, &git_con_ssh(&f, &b), true, &sin_token);
    assert_eq!(develop(&h).motivo.as_deref(), Some("historia-sin-relacion"));
    assert_eq!(
        develop(&h).orden.as_deref(),
        Some("git fetch origin && git merge-base develop origin/main")
    );
    assert!(
        h.acciones
            .iter()
            .filter(|a| a.decision == Decision::Persona)
            .all(|a| a.orden.is_some()),
        "{}",
        stdout(&h)
    );
    assert!(sin_push(&h), "{}", stdout(&h));
}

/// Revision 1, MEN-1 de calidad: con historia, una linea dice que hay que
/// versionar la configuracion y `.gitignore`; deja de salir cuando estan
/// en un commit.
#[test]
fn con_historia_pide_versionar_la_configuracion() {
    let f = Fixture::nueva("init-versionar");
    let repo = repo_vacio(&f);
    escribir(&repo, "README.md", "x\n");
    f.crudo(&repo, &["add", "README.md"]);
    f.crudo(&repo, &["commit", "-q", "-m", "inicial"]);
    let g = f.git();
    let h = correr(&repo, &g, false, &sin_token);
    let v: Vec<&tc2::init::Accion> = h
        .acciones
        .iter()
        .filter(|a| a.clase == "versionar")
        .collect();
    assert_eq!(v.len(), 1, "{}", stdout(&h));
    assert_eq!(
        v[0].linea(),
        "accion ruta=.taskcode/config.yml,.gitignore clase=versionar dueno=proyecto decision=persona motivo=sin-versionar orden=git%20add%20--%20.taskcode/config.yml%20.gitignore%20&&%20git%20commit%20-m%20\"chore:%20configura%20TaskCodeTwo\""
    );
    f.crudo(&repo, &["add", "--", ".taskcode/config.yml", ".gitignore"]);
    f.crudo(&repo, &["commit", "-q", "-m", "configura"]);
    let h = correr(&repo, &g, false, &sin_token);
    assert!(de_clase(&h, "versionar").is_empty(), "{}", stdout(&h));
}

/// Revision 1, MEN-1 de rust: la orden de una rama de estado montada en otra
/// ruta va entrecomillada (espacios y `;` no parten la orden).
#[test]
fn montada_en_otra_ruta_da_la_orden_entrecomillada() {
    let f = Fixture::nueva("init-otra-ruta");
    let repo = repo_vacio(&f);
    let g = f.git();
    assert_eq!(
        correr(&repo, &g, false, &sin_token).informe.resultado,
        Resultado::Ok
    );
    let otra = f.raiz.join("con espacios; echo X");
    f.crudo(
        &repo,
        &["worktree", "move", "tareas", otra.to_str().unwrap()],
    );
    let h = correr(&repo, &g, false, &sin_token);
    assert_eq!(h.informe.resultado, Resultado::Rechazada, "{}", stdout(&h));
    assert_eq!(h.informe.valor("motivo"), Some("montada-en-otra-ruta"));
    let ruta = f
        .crudo(&repo, &["worktree", "list", "--porcelain"])
        .lines()
        .filter_map(|l| l.strip_prefix("worktree "))
        .find(|l| l.contains("con espacios"))
        .unwrap()
        .to_owned();
    let orden = h
        .acciones
        .iter()
        .find(|a| a.clase == "worktree-estado")
        .and_then(|a| a.orden.clone())
        .unwrap();
    assert_eq!(orden, format!("git worktree move '{ruta}' tareas"));
}

/// Revision 1, MEN-4 de calidad: `/tareas` sin barra final ya cuenta; no se
/// anade `/tareas/` repetido.
#[test]
fn gitignore_sin_barra_final_no_se_duplica() {
    let f = Fixture::nueva("init-gitignore");
    let repo = repo_vacio(&f);
    escribir(&repo, ".gitignore", "/tareas\n");
    let h = correr(&repo, &f.git(), false, &sin_token);
    assert_eq!(h.informe.resultado, Resultado::Ok, "{}", stdout(&h));
    assert_eq!(
        std::fs::read_to_string(repo.join(".gitignore")).unwrap(),
        "/tareas\n/.taskcodetwo/\n"
    );
}

/// Revision 1, MEN-4 de rust: si `detectar` rechaza la plataforma, las
/// lineas `pendiente` dicen por que (aqui la clave contradice el host), y
/// no se habla con ninguna API.
#[test]
fn plataforma_rechazada_conserva_el_motivo() {
    let f = Fixture::nueva("init-detectar");
    let repo = repo_vacio(&f);
    escribir(
        &repo,
        ".taskcode/config.yml",
        "---\nformato: 2\nplataforma:\n  proveedor: \"gitlab\"\n  api: \"https://gitlab.com/api/v4\"\n",
    );
    let b = bare(&f);
    f.crudo(
        &repo,
        &["remote", "add", "origin", "ssh://git@github.com/o/r.git"],
    );
    let h = correr(&repo, &git_con_ssh(&f, &b), false, &con_token);
    assert_eq!(h.informe.resultado, Resultado::Ok, "{}", stdout(&h));
    let motivos: Vec<Option<String>> = h
        .acciones
        .iter()
        .filter(|a| matches!(a.clase, "personas" | "ruleset"))
        .map(|a| a.motivo.clone())
        .collect();
    assert_eq!(motivos.len(), 6);
    assert!(
        motivos
            .iter()
            .all(|m| m.as_deref() == Some("plataforma-contradice")),
        "{}",
        stdout(&h)
    );
}

/// C7: tras `init`, `desinstalar --proyecto --simular` lista, con
/// `dueno=taskcodetwo`, todo lo de la 2.0 que creo `init`.
#[test]
fn init_y_desinstalar_se_reconocen() {
    let f = Fixture::nueva("init-c7");
    let repo = repo_vacio(&f);
    let g = f.git();
    let h = correr(&repo, &g, false, &sin_token);
    assert_eq!(h.informe.resultado, Resultado::Ok, "{}", stdout(&h));
    let home = f.raiz.join("home");
    let entorno = tc2::instalacion::Entorno {
        sistema: tc2::instalacion::Sistema::Linux,
        home: Some(home.clone()),
        local_app_data: Some(home.join("AppData").join("Local")),
        xdg_cache_home: None,
        install_dir: None,
        claude_config_dir: None,
        exe: f.raiz.join("tc2"),
        pid: 4242,
    };
    let maquina = tc2::desinstalar::MaquinaEnMemoria::default();
    let c = tc2::desinstalar::Contexto {
        entorno: &entorno,
        dir: &repo,
        git: &g,
        maquina: &maquina,
    };
    let op = tc2::desinstalar::leer_opciones(&["--proyecto".to_owned(), "--simular".to_owned()])
        .unwrap();
    let d = tc2::desinstalar::desinstalar(&op, &c);
    let lista = d.salida_en(false, Idioma::Es).stdout;
    let norm = |r: &str| r.replace('\\', "/").to_lowercase();
    let suyas: Vec<String> = d
        .acciones
        .iter()
        .filter(|a| a.dueno == "taskcodetwo")
        .map(|a| norm(&a.ruta))
        .collect();
    let creadas: Vec<_> = h
        .acciones
        .iter()
        .filter(|a| a.decision == Decision::Crea && a.dueno == "taskcodetwo")
        .collect();
    assert!(creadas.len() >= 4, "{}", stdout(&h));
    for a in creadas {
        assert!(
            suyas.contains(&norm(&a.ruta)),
            "{} ({}) no esta en desinstalar:\n{lista}",
            a.ruta,
            a.clase
        );
    }
    assert!(
        lista.contains(" clase=gitignore dueno=taskcodetwo decision=persona"),
        "{lista}"
    );
}

// --- TASK-118: `tc2 init --aplicar-puertas` ---------------------------------
//
// Contra el doble HTTP comun (`comun::http_mock`), que acepta escrituras y
// registra cada peticion con su cuerpo. El repo se deja como lo dejaria un
// primer `init` sin la opcion: `develop` y `main` publicadas en el remoto
// bare y `personas.yml` con la persona del token.

/// Id del revisor IA (`plataforma.revisor_id`) en los tests.
const REVISOR: u64 = 99001;

/// `init` con `--aplicar-puertas`.
fn aplicar(dir: &Path, g: &Git, simular: bool) -> Hecho {
    let c = Contexto {
        dir,
        git: g,
        credenciales: &con_token,
    };
    init(
        Opciones {
            simular,
            aplicar_puertas: true,
        },
        &c,
    )
}

/// Configuracion con la clave `plataforma` y, si se da, `revisor_id`.
fn config_puertas(proveedor: &str, api: &str, revisor: Option<u64>) -> String {
    let mut s =
        format!("---\nformato: 2\nplataforma:\n  proveedor: \"{proveedor}\"\n  api: \"{api}\"\n");
    if let Some(id) = revisor {
        s.push_str(&format!("  revisor_id: {id}\n"));
    }
    s
}

/// Repo con `develop` y `main` ya en el remoto y `personas.yml` escrito por
/// un primer `init` sin la opcion contra `mock`.
fn repo_listo(f: &Fixture, config: &str) -> (PathBuf, Git) {
    let repo = repo_vacio(f);
    escribir(&repo, ".taskcode/config.yml", config);
    let b = bare(f);
    f.crudo(&repo, &["remote", "add", "origin", ORIGIN]);
    let g = git_con_ssh(f, &b);
    let h = correr(&repo, &g, false, &con_token);
    assert_eq!(h.informe.resultado, Resultado::Ok, "{}", stdout(&h));
    assert!(
        repo.join(".taskcode").join("personas.yml").is_file(),
        "{}",
        stdout(&h)
    );
    (repo, g)
}

/// Escrituras (`POST`, `PUT`, `PATCH`, `DELETE`) que recibio el doble desde
/// la peticion `desde`.
fn escrituras(m: &Mock, desde: usize) -> Vec<Recibida> {
    m.recibidas()
        .into_iter()
        .skip(desde)
        .filter(|r| !r.linea.starts_with("GET "))
        .collect()
}

/// GitHub con la persona 1001, los rulesets `rulesets` y los check-runs de
/// la cabeza de `develop` publicados por `(nombre, app.id)`.
fn github_mock(rulesets: &[&str], checks: &[(&str, u64)]) -> Mock {
    let m = Mock::nuevo();
    m.responder(
        "GET",
        "/user",
        Simulada::json(200, r#"{"login":"carlos","id":1001,"type":"User"}"#),
    );
    let rs: Vec<serde_json::Value> = rulesets
        .iter()
        .enumerate()
        .map(|(i, n)| serde_json::json!({"id": i + 1, "name": n}))
        .collect();
    m.responder(
        "GET",
        "/repos/o/r/rulesets",
        Simulada::json(200, &serde_json::Value::from(rs).to_string()),
    );
    m.responder(
        "GET",
        "/repos/o/r/rules/branches/develop",
        Simulada::json(200, "[]"),
    );
    let runs: Vec<serde_json::Value> = checks
        .iter()
        .map(|(n, app)| {
            serde_json::json!({"name": n, "status": "completed", "conclusion": "success", "app": {"id": app}})
        })
        .collect();
    m.responder(
        "GET",
        "/repos/o/r/commits/heads/develop/check-runs",
        Simulada::json(
            200,
            &serde_json::json!({"total_count": runs.len(), "check_runs": runs}).to_string(),
        ),
    );
    m.responder(
        "POST",
        "/repos/o/r/rulesets",
        Simulada::json(201, r#"{"id":77}"#),
    );
    m
}

/// Los tres checks del modelo publicados por su origen en GitHub.
const CHECKS_GITHUB: [(&str, u64); 3] = [
    ("verificacion", 15368),
    ("revision-ia", REVISOR),
    ("carril", 15368),
];

/// `(ruta, decision, motivo)` de las lineas `ruleset`.
fn rulesets_de(h: &Hecho) -> Vec<(String, &'static str, Option<String>)> {
    de_clase(h, "ruleset")
}

/// Criterio 1 (GitHub): con los checks publicados por sus origenes, `init
/// --aplicar-puertas` hace `POST /rulesets` de las que faltan, con el
/// `integration_id` de cada check y sin `_nota`, y no toca la que ya
/// existe. Una segunda ejecucion con las cinco ya creadas no escribe nada.
#[test]
fn aplicar_puertas_en_github_crea_las_que_faltan() {
    let _serie = serie();
    let m = github_mock(&["taskcodetwo-tareas", "de-la-persona"], &CHECKS_GITHUB);
    let f = Fixture::nueva("init-puertas-gh");
    let (repo, g) = repo_listo(&f, &config_puertas("github", &m.base(), Some(REVISOR)));
    let antes = m.recibidas().len();

    // `--simular` dice lo mismo sin escribir.
    let sim = aplicar(&repo, &g, true);
    assert_eq!(sim.informe.resultado, Resultado::Ok, "{}", stdout(&sim));
    assert!(escrituras(&m, antes).is_empty());

    let h = aplicar(&repo, &g, false);
    let texto = stdout(&h);
    assert_eq!(h.informe.resultado, Resultado::Ok, "{texto}");
    let crea = |n: &str| (format!("taskcodetwo-{n}"), "crea", None);
    assert_eq!(
        rulesets_de(&h),
        [
            crea("develop"),
            crea("main"),
            crea("ramas-de-tarea"),
            ("taskcodetwo-tareas".to_owned(), "existe", None),
            crea("tags-version"),
        ],
        "{texto}"
    );
    assert_eq!(rulesets_de(&sim), rulesets_de(&h));
    // Lo unico que crea esta ejecucion son los cuatro rulesets.
    assert_eq!(h.informe.valor("crea"), Some("4"), "{texto}");
    let posts = escrituras(&m, antes);
    assert_eq!(posts.len(), 4, "{posts:?}");
    let mut nombres = Vec::new();
    for p in &posts {
        assert_eq!(p.linea, "POST /repos/o/r/rulesets");
        let v: serde_json::Value = serde_json::from_str(&p.cuerpo).unwrap();
        assert!(v.get("_nota").is_none(), "{v}");
        assert_eq!(v["bypass_actors"], serde_json::json!([]));
        nombres.push(v["name"].as_str().unwrap().to_owned());
        for r in v["rules"].as_array().unwrap() {
            if r["type"] == "required_status_checks" {
                for c in r["parameters"]["required_status_checks"]
                    .as_array()
                    .unwrap()
                {
                    let esperado = if c["context"] == "revision-ia" {
                        REVISOR
                    } else {
                        15368
                    };
                    assert_eq!(c["integration_id"], esperado, "{v}");
                }
            }
        }
    }
    assert_eq!(
        nombres,
        [
            "taskcodetwo-develop",
            "taskcodetwo-main",
            "taskcodetwo-ramas-de-tarea",
            "taskcodetwo-tags-version"
        ]
    );
    // El ruleset ajeno y el existente no se tocan: ni PUT ni DELETE.
    assert!(posts.iter().all(|p| p.linea.starts_with("POST ")));

    // Segunda ejecucion, con las cinco ya en GitHub: ninguna escritura.
    let m2 = github_mock(
        &[
            "taskcodetwo-develop",
            "taskcodetwo-main",
            "taskcodetwo-ramas-de-tarea",
            "taskcodetwo-tareas",
            "taskcodetwo-tags-version",
        ],
        &CHECKS_GITHUB,
    );
    escribir(
        &repo,
        ".taskcode/config.yml",
        &config_puertas("github", &m2.base(), Some(REVISOR)),
    );
    let h2 = aplicar(&repo, &g, false);
    assert_eq!(
        h2.informe.resultado,
        Resultado::SinCambios,
        "{}",
        stdout(&h2)
    );
    assert!(
        rulesets_de(&h2).iter().all(|(_, d, _)| *d == "existe"),
        "{}",
        stdout(&h2)
    );
    assert!(escrituras(&m2, 0).is_empty(), "{:?}", escrituras(&m2, 0));
}

/// GitLab con la persona 1001 en Free, `main` ya protegida con los valores
/// de GitLab al crear el proyecto (push y merge de Maintainer), los estados
/// de la cabeza de `develop` publicados por `(nombre, author.id)` y las
/// ramas y tags protegidos `protegidas` y `tags` (JSON).
fn gitlab_mock(protegidas: &str, tags: &str, estados: &[(&str, u64)]) -> Mock {
    gitlab_mock_con(protegidas, tags, estados, AJUSTES_FF)
}

/// Ajustes del proyecto recien creado: merge por fast-forward y sin exigir
/// el pipeline.
const AJUSTES_FF: &str =
    r#"{"only_allow_merge_if_pipeline_succeeds":false,"merge_method":"ff","namespace":{"id":9}}"#;

/// Ajustes del proyecto como los deja `aplicar-puertas`.
const AJUSTES_BIEN: &str =
    r#"{"only_allow_merge_if_pipeline_succeeds":true,"merge_method":"merge","namespace":{"id":9}}"#;

/// Como [`gitlab_mock`], con los ajustes del proyecto `proyecto` (JSON).
fn gitlab_mock_con(protegidas: &str, tags: &str, estados: &[(&str, u64)], proyecto: &str) -> Mock {
    let m = Mock::nuevo();
    let p = "/api/v4/projects/o%2Fr";
    m.responder(
        "GET",
        "/api/v4/user",
        Simulada::json(200, r#"{"username":"carlos","id":1001,"bot":false}"#),
    );
    m.responder(
        "GET",
        &format!("{p}/protected_branches"),
        Simulada::json(200, protegidas),
    );
    m.responder(
        "GET",
        &format!("{p}/protected_tags"),
        Simulada::json(200, tags),
    );
    m.responder(
        "GET",
        &format!("{p}/approval_rules"),
        Simulada::json(200, "[]"),
    );
    m.responder("GET", p, Simulada::json(200, proyecto));
    m.responder(
        "GET",
        "/api/v4/namespaces/9",
        Simulada::json(200, r#"{"plan":"free"}"#),
    );
    let sha = "c".repeat(40);
    m.responder(
        "GET",
        &format!("{p}/repository/branches/develop"),
        Simulada::json(200, &format!(r#"{{"commit":{{"id":"{sha}"}}}}"#)),
    );
    let estados: Vec<serde_json::Value> = estados
        .iter()
        .enumerate()
        .map(|(i, (n, a))| {
            serde_json::json!({"id": i + 1, "name": n, "status": "success", "author": {"id": a}})
        })
        .collect();
    m.responder(
        "GET",
        &format!("{p}/repository/commits/{sha}/statuses"),
        Simulada::json(200, &serde_json::Value::from(estados).to_string()),
    );
    m.responder(
        "POST",
        &format!("{p}/protected_branches"),
        Simulada::json(201, "{}"),
    );
    m.responder(
        "POST",
        &format!("{p}/protected_tags"),
        Simulada::json(201, "{}"),
    );
    m.responder("PUT", p, Simulada::json(200, "{}"));
    m
}

/// `main` como la protege GitLab al crear el proyecto.
const MAIN_POR_DEFECTO: &str = r#"[{"name":"main","push_access_levels":[{"access_level":40}],"merge_access_levels":[{"access_level":40}],"allow_force_push":false}]"#;

/// Criterio 1 (GitLab): ramas y tags protegidos con niveles explicitos
/// (push de Developer y sin force-push en la rama de estado y las de tarea;
/// `develop` sin push y merge de Maintainer), los ajustes de merge del
/// proyecto, ninguna peticion `DELETE`, y `main`, ya protegida distinta,
/// sale `existe motivo=difiere` sin tocarse. En Free, `sin-aprobaciones=si`.
#[test]
fn aplicar_puertas_en_gitlab_protege_con_niveles_explicitos() {
    let _serie = serie();
    let estados = [
        ("verificacion", 3001),
        ("carril", 3001),
        ("revision-ia", REVISOR),
    ];
    let m = gitlab_mock(MAIN_POR_DEFECTO, "[]", &estados);
    let f = Fixture::nueva("init-puertas-gl");
    let api = format!("{}/api/v4", m.base());
    let (repo, g) = repo_listo(&f, &config_puertas("gitlab", &api, Some(REVISOR)));
    let antes = m.recibidas().len();
    let h = aplicar(&repo, &g, false);
    let texto = stdout(&h);
    assert_eq!(h.informe.resultado, Resultado::Ok, "{texto}");
    assert_eq!(h.informe.valor("sin-aprobaciones"), Some("si"), "{texto}");
    let crea = |n: &str| (format!("taskcodetwo-{n}"), "crea", None);
    assert_eq!(
        rulesets_de(&h),
        [
            crea("develop"),
            (
                "taskcodetwo-main".to_owned(),
                "existe",
                Some("difiere".to_owned())
            ),
            crea("ramas-de-tarea"),
            crea("tareas"),
            crea("tags-version"),
        ],
        "{texto}"
    );
    let es = escrituras(&m, antes);
    assert!(
        es.iter()
            .all(|r| !r.linea.starts_with("DELETE ") && !r.linea.starts_with("PATCH ")),
        "{es:?}"
    );
    let ramas: BTreeMap<String, serde_json::Value> = es
        .iter()
        .filter(|r| r.linea == "POST /api/v4/projects/o%2Fr/protected_branches")
        .map(|r| {
            let v: serde_json::Value = serde_json::from_str(&r.cuerpo).unwrap();
            (v["name"].as_str().unwrap().to_owned(), v)
        })
        .collect();
    assert_eq!(
        ramas.keys().map(String::as_str).collect::<Vec<_>>(),
        [
            "develop",
            "feature/*",
            "fix/*",
            "hotfix/*",
            "taskcodetwo/tareas"
        ],
        "{es:?}"
    );
    for (n, v) in &ramas {
        assert_eq!(v["allow_force_push"], false, "{n}");
        let (push, merge) = if n == "develop" { (0, 40) } else { (30, 30) };
        assert_eq!(v["push_access_level"], push, "{n}: {v}");
        assert_eq!(v["merge_access_level"], merge, "{n}: {v}");
    }
    let tags: Vec<serde_json::Value> = es
        .iter()
        .filter(|r| r.linea == "POST /api/v4/projects/o%2Fr/protected_tags")
        .map(|r| serde_json::from_str(&r.cuerpo).unwrap())
        .collect();
    assert_eq!(
        tags,
        [serde_json::json!({"name": "v*", "create_access_level": 30})]
    );
    let ajustes: Vec<&Recibida> = es
        .iter()
        .filter(|r| r.linea == "PUT /api/v4/projects/o%2Fr")
        .collect();
    assert_eq!(ajustes.len(), 1, "{es:?}");
    let v: serde_json::Value = serde_json::from_str(&ajustes[0].cuerpo).unwrap();
    assert_eq!(
        v,
        serde_json::json!({"merge_method": "merge", "only_allow_merge_if_pipeline_succeeds": true})
    );
    // Nada sobre `main` ni sobre reglas de aprobacion (Free).
    assert!(
        es.iter()
            .all(|r| !r.cuerpo.contains("\"main\"") && !r.linea.contains("approval_rules")),
        "{es:?}"
    );
    assert_eq!(es.len(), 7, "{es:?}");

    // Segunda ejecucion con todo ya protegido y los ajustes de merge ya
    // puestos (sin ellos, `develop` se completaria): ninguna escritura.
    let todo = r#"[
      {"name":"main","push_access_levels":[{"access_level":40}],"merge_access_levels":[{"access_level":40}],"allow_force_push":false},
      {"name":"develop","push_access_levels":[{"access_level":0}],"merge_access_levels":[{"access_level":40}],"allow_force_push":false},
      {"name":"feature/*","push_access_levels":[{"access_level":30}],"merge_access_levels":[{"access_level":30}],"allow_force_push":false},
      {"name":"fix/*","push_access_levels":[{"access_level":30}],"merge_access_levels":[{"access_level":30}],"allow_force_push":false},
      {"name":"hotfix/*","push_access_levels":[{"access_level":30}],"merge_access_levels":[{"access_level":30}],"allow_force_push":false},
      {"name":"taskcodetwo/tareas","push_access_levels":[{"access_level":30}],"merge_access_levels":[{"access_level":30}],"allow_force_push":false}
    ]"#;
    let m2 = gitlab_mock_con(
        todo,
        r#"[{"name":"v*","create_access_levels":[{"access_level":30}]}]"#,
        &estados,
        AJUSTES_BIEN,
    );
    escribir(
        &repo,
        ".taskcode/config.yml",
        &config_puertas("gitlab", &format!("{}/api/v4", m2.base()), Some(REVISOR)),
    );
    let h2 = aplicar(&repo, &g, false);
    assert_eq!(
        h2.informe.resultado,
        Resultado::SinCambios,
        "{}",
        stdout(&h2)
    );
    assert!(escrituras(&m2, 0).is_empty(), "{:?}", escrituras(&m2, 0));
}

/// Criterio 2: sin el origen de un check (`revisor_id` ausente o 0) o sin
/// los checks ya publicados por su origen en `develop` (un `revision-ia`
/// de otra App, `verificacion` sin publicar), la opcion se rechaza con el
/// motivo y lo que falta, sin ninguna escritura en la plataforma ni en el
/// disco.
#[test]
fn aplicar_puertas_sin_origen_se_rechaza_sin_escribir() {
    let _serie = serie();
    /// `(revisor_id, checks publicados, faltan=)`.
    type Caso<'a> = (Option<u64>, Vec<(&'a str, u64)>, &'a str);
    let casos: [Caso<'_>; 4] = [
        (None, CHECKS_GITHUB.to_vec(), "revision-ia"),
        (Some(0), CHECKS_GITHUB.to_vec(), "revision-ia"),
        (
            Some(REVISOR),
            vec![
                ("verificacion", 15368),
                ("revision-ia", 4242),
                ("carril", 15368),
            ],
            "revision-ia",
        ),
        (
            Some(REVISOR),
            vec![("verificacion", 4242), ("revision-ia", REVISOR)],
            "verificacion,carril",
        ),
    ];
    for (revisor, checks, faltan) in casos {
        let m = github_mock(&[], &checks);
        let f = Fixture::nueva("init-puertas-sin-origen");
        let (repo, g) = repo_listo(&f, &config_puertas("github", &m.base(), revisor));
        let antes = m.recibidas().len();
        let disco = comun::instantanea(&repo, &[]);
        for simular in [true, false] {
            let h = aplicar(&repo, &g, simular);
            let s = h.salida_en(false, Idioma::Es);
            assert_eq!(h.informe.resultado, Resultado::Rechazada, "{}", s.stdout);
            let fin = s.stdout.lines().last().unwrap_or_default().to_owned();
            assert!(
                fin.starts_with(&format!(
                    "init rechazada motivo=puertas-sin-origen faltan={faltan} ruta=taskcodetwo-develop "
                )),
                "{fin}"
            );
            assert!(
                rulesets_de(&h)
                    .iter()
                    .all(|(_, d, m)| *d == "rechaza" && m.as_deref() == Some("puertas-sin-origen")),
                "{}",
                s.stdout
            );
            // El diagnostico dice que hacer.
            assert!(
                s.stderr.contains("tc2 init --aplicar-puertas"),
                "{}",
                s.stderr
            );
        }
        assert!(
            escrituras(&m, antes).is_empty(),
            "{:?}",
            escrituras(&m, antes)
        );
        assert_eq!(disco, comun::instantanea(&repo, &[]));
    }
}

/// Criterio 2 (resto de la precondicion): sin `personas.yml`, con `develop`
/// sin publicar o con la persona fuera de la lista, la opcion se rechaza
/// con su motivo y sin escribir en la plataforma.
#[test]
fn aplicar_puertas_exige_personas_ramas_y_persona() {
    let _serie = serie();
    let m = github_mock(&[], &CHECKS_GITHUB);
    let config = config_puertas("github", &m.base(), Some(REVISOR));

    // Sin `personas.yml` (repo con remoto pero sin el primer `init`).
    let f = Fixture::nueva("init-puertas-personas");
    let (repo, g) = repo_listo(&f, &config);
    std::fs::remove_file(repo.join(".taskcode").join("personas.yml")).unwrap();
    let h = aplicar(&repo, &g, false);
    assert_eq!(
        h.informe.valor("motivo"),
        Some("sin-personas"),
        "{}",
        stdout(&h)
    );
    // `personas.yml` no se crea en esta ejecucion: el plan se rechaza entero.
    assert!(!repo.join(".taskcode").join("personas.yml").exists());

    // La persona del token no esta en la lista.
    let f = Fixture::nueva("init-puertas-lista");
    let (repo, g) = repo_listo(&f, &config);
    escribir(
        &repo,
        ".taskcode/personas.yml",
        "---\npersonas:\n  - {id: 7, login: \"otra\"}\n",
    );
    let h = aplicar(&repo, &g, false);
    assert_eq!(
        h.informe.valor("motivo"),
        Some("sin-persona"),
        "{}",
        stdout(&h)
    );

    // `develop` con un commit local que `origin` no tiene.
    let f = Fixture::nueva("init-puertas-delante");
    let (repo, g) = repo_listo(&f, &config);
    f.crudo(
        &repo,
        &[
            "commit",
            "-q",
            "--allow-empty",
            "-m",
            "local",
            "--no-verify",
        ],
    );
    let h = aplicar(&repo, &g, false);
    assert_eq!(
        h.informe.valor("motivo"),
        Some("ramas-por-delante"),
        "{}",
        stdout(&h)
    );
    assert!(escrituras(&m, 0).is_empty(), "{:?}", escrituras(&m, 0));
}
