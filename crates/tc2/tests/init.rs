//! `tc2 init` (TASK-028, F3-1): un test (o mas) por criterio, con git real
//! en carpetas temporales y GitHub simulado en 127.0.0.1. Nada contra la
//! red real: `origin` apunta a un host inventado y su transporte ssh se
//! sustituye con `GIT_SSH_COMMAND` (el remoto bare de la fixture, o un
//! «ssh» que no conecta).
//!
//! El servidor simulado no cierra la conexion tras responder: con
//! `Content-Length` cierra el cliente (en esta maquina un FIN del servidor
//! detras de los datos colgaba a veces la conexion, TASK-027).

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use comun::{Fixture, escribir};
use tc2::contrato::Resultado;
use tc2::git::Git;
use tc2::i18n::Idioma;
use tc2::init::{Contexto, Decision, Hecho, Opciones, init};
use tc2::plataforma::{Credenciales, CredencialesFijas, Rol, Secreto};

/// GitHub simulado: `GET <ruta>` da la respuesta fija o 404, y registra
/// cada peticion (`METODO ruta`).
struct Servidor {
    puerto: u16,
    peticiones: Arc<Mutex<Vec<String>>>,
}

impl Servidor {
    fn nuevo(rutas: &[(&str, &str)]) -> Self {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let puerto = l.local_addr().unwrap().port();
        let peticiones: Arc<Mutex<Vec<String>>> = Arc::default();
        let rutas: Arc<BTreeMap<String, String>> = Arc::new(
            rutas
                .iter()
                .map(|(r, c)| ((*r).to_owned(), (*c).to_owned()))
                .collect(),
        );
        let p = peticiones.clone();
        std::thread::spawn(move || {
            for s in l.incoming().flatten() {
                let (p, rutas) = (p.clone(), rutas.clone());
                std::thread::spawn(move || atender(s, &p, &rutas));
            }
        });
        Self { puerto, peticiones }
    }

    fn base(&self) -> String {
        format!("http://127.0.0.1:{}", self.puerto)
    }

    fn peticiones(&self) -> Vec<String> {
        self.peticiones.lock().unwrap().clone()
    }
}

fn atender(s: std::net::TcpStream, p: &Mutex<Vec<String>>, rutas: &BTreeMap<String, String>) {
    let mut lector = BufReader::new(s.try_clone().unwrap());
    let mut primera = String::new();
    if lector.read_line(&mut primera).is_err() {
        return;
    }
    let mut largo = 0usize;
    loop {
        let mut l = String::new();
        if lector.read_line(&mut l).is_err() || l.trim().is_empty() {
            break;
        }
        if let Some((k, v)) = l.split_once(':')
            && k.trim().eq_ignore_ascii_case("content-length")
        {
            largo = v.trim().parse().unwrap_or(0);
        }
    }
    let mut cuerpo = vec![0u8; largo];
    let _ = std::io::Read::read_exact(&mut lector, &mut cuerpo);
    let mut partes = primera.split_whitespace();
    let metodo = partes.next().unwrap_or_default().to_owned();
    let ruta = partes.next().unwrap_or_default().to_owned();
    p.lock().unwrap().push(format!("{metodo} {ruta}"));
    let sin_consulta = ruta.split('?').next().unwrap_or_default();
    let (estado, cuerpo) = match rutas.get(sin_consulta).filter(|_| metodo == "GET") {
        Some(c) => ("200 OK", c.clone()),
        None => ("404 Not Found", "{\"message\":\"Not Found\"}".to_owned()),
    };
    let mut s = s;
    let _ = write!(
        s,
        "HTTP/1.1 {estado}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{cuerpo}",
        cuerpo.len()
    );
    let _ = s.flush();
    // Sin FIN del servidor: cierra el cliente al leer el `Content-Length`.
    let _ = s.set_read_timeout(Some(std::time::Duration::from_secs(30)));
    let _ = std::io::Read::read_to_end(&mut lector, &mut Vec::new());
}

/// Rutas de GitHub que lee `init`: la persona y los rulesets (ninguno).
fn github() -> Servidor {
    Servidor::nuevo(&[
        (
            "/user",
            "{\"login\":\"carlos\",\"id\":1001,\"type\":\"User\"}",
        ),
        ("/repos/o/r/rulesets", "[]"),
        ("/repos/o/r/rules/branches/develop", "[]"),
    ])
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
    init(Opciones { simular }, &c)
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

/// Instantanea de una carpeta (con `.git`): ruta relativa → contenido.
fn instantanea(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    fn recorrer(base: &Path, d: &Path, m: &mut BTreeMap<String, Vec<u8>>) {
        let Ok(it) = std::fs::read_dir(d) else { return };
        for e in it.flatten() {
            let p = e.path();
            let rel = p
                .strip_prefix(base)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if p.is_dir() {
                m.insert(format!("{rel}/"), Vec::new());
                recorrer(base, &p, m);
            } else {
                m.insert(rel, std::fs::read(&p).unwrap());
            }
        }
    }
    let mut m = BTreeMap::new();
    recorrer(dir, dir, &mut m);
    m
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
    let ficheros = instantanea(&repo);
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
    assert_eq!(ficheros, instantanea(&repo));
}

/// C3: `--simular` (y `--simular --json`) imprime el plan y `simular=si`
/// sin cambiar ni un byte (ni crea `.git/taskcodetwo/`), y el plan simulado
/// es igual a las acciones de la ejecucion real de despues.
#[test]
fn simular_no_toca_nada_y_coincide() {
    let f = Fixture::nueva("init-c3");
    let repo = repo_vacio(&f);
    let g = f.git();
    let antes = instantanea(&repo);
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
    assert_eq!(antes, instantanea(&repo), "--simular cambio algo");
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
        let antes = instantanea(&repo);
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
        assert_eq!(antes, instantanea(&repo), "{caso}: escribio algo");
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
        let antes = instantanea(&repo);
        let h = correr(&repo, &f.git(), false, &sin_token);
        assert_eq!(h.informe.resultado, Resultado::Rechazada, "{}", stdout(&h));
        assert_eq!(h.informe.valor("motivo"), Some(motivo));
        assert_eq!(antes, instantanea(&repo));
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
    assert!(s.peticiones().is_empty(), "{:?}", s.peticiones());

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
                "accion ruta=taskcodetwo-{n} clase=ruleset dueno=taskcodetwo decision=persona motivo=sin-checks orden=tc2%20init%20--aplicar-puertas"
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
    let peticiones = s.peticiones();
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
    let antes = instantanea(&repo);
    for simular in [false, true] {
        let h = correr(&repo, &git_sin_red(&f), simular, &con_token);
        let s = h.salida_en(false, Idioma::Es);
        assert_eq!(s.codigo, 4, "{}", s.stdout);
        assert_eq!(s.stdout, "init sinred pendiente=no\n");
        assert!(s.stderr.contains("Connection refused"), "{}", s.stderr);
        assert_eq!(antes, instantanea(&repo));
    }
}

/// En un repo vacio con remoto, el commit raiz y las tres ramas se
/// publican sin force, y la rama de estado queda en el remoto.
#[test]
fn repo_vacio_con_remoto_publica() {
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
