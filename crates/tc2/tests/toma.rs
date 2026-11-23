//! F3-6 (TASK-033): `tc2 start` con el binario real, como lo lanza el
//! agente (sin terminal: quien la toma sale de la cache de identidad).
//!
//! `origin` apunta a `ssh://git@github.com/o/r.git` (GitHub por el host, sin
//! clave `plataforma`) y el transporte ssh se sustituye con
//! `GIT_SSH_COMMAND`, que sirve el remoto bare de la fixture: `ls-remote
//! --get-url` aplica `insteadOf`, asi que el remoto no se puede redirigir por
//! configuracion sin cambiar el host que se detecta (como en
//! `tests/identidad.rs`). Sin terminal no se habla con la API.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Arc, Barrier};

use comun::{Fixture, escribir};
use tc2::contrato::Resultado;
use tc2::estado::{self, Cambio, Contexto, Estado, Fallo, Operacion};
use tc2::identidad::{Cache, Cuenta};
use tc2::plataforma::Proveedor;

const RAMA_ESTADO: &str = estado::RAMA;
const ORIGIN: &str = "ssh://git@github.com/o/r.git";
const PERSONAS: &str =
    "---\npersonas:\n  - {id: 1001, login: \"ana\"}\n  - {id: 1002, login: \"beto\"}\n";

/// Entorno de git con el «ssh» que sirve el remoto bare de la fixture
/// (`upload-pack` para leer, `receive-pack` para publicar).
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

/// Entorno de git cuyo «ssh» no llega al host, como sin red.
fn entorno_sin_red(f: &Fixture) -> Vec<(OsString, OsString)> {
    let mut e = f.entorno();
    e.push((
        "GIT_SSH_COMMAND".into(),
        "sh -c 'echo \"ssh: connect to host github.com port 22: Connection refused\" >&2; exit 255'"
            .into(),
    ));
    e.push(("GIT_SSH_VARIANT".into(), "simple".into()));
    e
}

fn lanzar(dir: &Path, entorno: Vec<(OsString, OsString)>, args: &[&str]) -> Output {
    comun::orden_tc2()
        .current_dir(dir)
        .args(args)
        .envs(entorno)
        .output()
        .expect("lanzar tc2")
}

/// `tc2 <args>` en `dir` con el remoto de la fixture.
fn tc2(f: &Fixture, dir: &Path, args: &[&str]) -> Output {
    lanzar(dir, entorno_ssh(f), args)
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

/// Identifica el clon `dir` como la persona `id`/`login` (cache vigente).
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

/// Remoto con `personas.yml` en `develop` y las tareas de `altas` (cada una,
/// los argumentos de `tc2 new`) creadas desde un clon de preparacion; el
/// clon `a` que devuelve tiene `origin` en GitHub y la identidad de ana.
fn preparar(f: &Fixture, altas: &[&[&str]]) -> (PathBuf, PathBuf) {
    let (bare, prep) = f.remoto_y_clon();
    escribir(&prep, ".taskcode/personas.yml", PERSONAS);
    f.crudo(&prep, &["add", ".taskcode/personas.yml"]);
    f.crudo(&prep, &["commit", "-q", "-m", "personas"]);
    f.crudo(&prep, &["push", "-q", "origin", "develop"]);
    for a in altas {
        let mut args = vec!["new"];
        args.extend_from_slice(a);
        let o = lanzar(&prep, f.entorno(), &args);
        assert_eq!(o.status.code(), Some(0), "{}", texto(&o.stdout));
    }
    let a = otro_clon(f, &bare, "a");
    identificar(&a, 1001, "ana");
    (bare, a)
}

/// Otro clon con `origin` en GitHub.
fn otro_clon(f: &Fixture, bare: &Path, nombre: &str) -> PathBuf {
    let d = f.otro_clon(bare, nombre);
    f.crudo(&d, &["remote", "set-url", "origin", ORIGIN]);
    d
}

/// `tarea.md` publicado de `id`.
fn publicado(f: &Fixture, bare: &Path, id: &str) -> String {
    f.crudo(bare, &["show", &format!("{RAMA_ESTADO}:{id}/tarea.md")])
}

fn punta_estado(f: &Fixture, bare: &Path) -> String {
    f.crudo(bare, &["rev-parse", RAMA_ESTADO])
}

/// Ramas `*/task-nnn-*` del clon.
fn ramas_del_id(f: &Fixture, dir: &Path, id: &str) -> String {
    f.crudo(
        dir,
        &[
            "branch",
            "--list",
            "--format=%(refname:short)",
            &format!("*/{}-*", id.to_lowercase()),
        ],
    )
}

fn worktree(dir: &Path, id: &str) -> PathBuf {
    dir.join(".taskcodetwo").join("worktrees").join(id)
}

/// Comprueba la salida de `start`: codigo, primera linea y una sola linea.
fn start(f: &Fixture, dir: &Path, id: &str) -> (Option<i32>, String) {
    let o = tc2(f, dir, &["start", id]);
    let out = texto(&o.stdout);
    assert_eq!(out.lines().count(), 1, "{out}");
    let err = texto(&o.stderr);
    if o.status.code() == Some(0) {
        assert!(err.is_empty(), "stderr en un exito: {err}");
    } else {
        assert_eq!(err.lines().count(), 1, "{err}");
    }
    (o.status.code(), out.trim_end().to_owned())
}

/// Escribe en la rama de estado por la transaccion (como lo haria otra
/// fase): `cambiar` recibe el texto de `ruta` (si existe) y da el nuevo.
struct Escribir {
    ruta: String,
    cambiar: Box<dyn Fn(Option<String>) -> String>,
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
        let t = e.leer(&self.ruta)?;
        Ok(vec![Cambio::nuevo(self.ruta.clone(), (self.cambiar)(t))])
    }
}

fn sin_catalogo(_: &str) -> Option<Box<dyn Operacion>> {
    None
}

fn escribir_estado(
    f: &Fixture,
    dir: &Path,
    ruta: &str,
    cambiar: impl Fn(Option<String>) -> String + 'static,
) {
    let g = tc2::git::Git::nuevo().con_entorno(entorno_ssh(f));
    let op = Escribir {
        ruta: ruta.to_owned(),
        cambiar: Box::new(cambiar),
    };
    let i = estado::transaccion(&Contexto::nuevo(&g, dir, &sin_catalogo, None), &op);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
}

/// Cambia `de` por `a` en el `tarea.md` publicado de `id`.
fn editar(f: &Fixture, dir: &Path, id: &str, de: &'static str, a: &'static str) {
    escribir_estado(f, dir, &format!("{id}/tarea.md"), move |t| {
        let t = t.unwrap_or_default();
        assert!(t.contains(de), "{t}");
        t.replacen(de, a, 1)
    });
}

/// C1: dos clones lanzan `start TASK-001` a la vez: uno sale `ok` y el otro
/// `rechazada motivo=tomada` con el login del ganador; en el remoto la lleva
/// el ganador y solo el tiene rama y worktree (el perdedor, ni commit de
/// estado local).
#[test]
fn c1_dos_clones_a_la_vez_solo_uno_la_toma() {
    let f = Arc::new(Fixture::nueva("toma-c1"));
    let (bare, a) = preparar(&f, &[&["Primera"]]);
    let b = otro_clon(&f, &bare, "b");
    identificar(&b, 1002, "beto");
    let barrera = Arc::new(Barrier::new(2));
    let hilos: Vec<_> = [("ana", a.clone()), ("beto", b.clone())]
        .into_iter()
        .map(|(login, dir)| {
            let (f, barrera) = (Arc::clone(&f), Arc::clone(&barrera));
            std::thread::spawn(move || {
                barrera.wait();
                let o = tc2(&f, &dir, &["start", "TASK-001"]);
                (
                    login,
                    dir,
                    o.status.code(),
                    texto(&o.stdout).trim_end().to_owned(),
                )
            })
        })
        .collect();
    let mut r: Vec<_> = hilos.into_iter().map(|h| h.join().unwrap()).collect();
    r.sort_by_key(|(_, _, codigo, _)| *codigo);
    let [(ganador, dir_g, c_g, out_g), (_, dir_p, c_p, out_p)] = r.as_slice() else {
        panic!("{r:?}");
    };
    assert_eq!(*c_g, Some(0), "{out_g}");
    assert!(out_g.starts_with("start ok id=TASK-001 "), "{out_g}");
    assert_eq!(*c_p, Some(3), "{out_p}");
    assert!(out_p.starts_with("start rechazada id=TASK-001 "), "{out_p}");
    assert_eq!(valor(out_p, "motivo"), Some("tomada"), "{out_p}");
    assert_eq!(valor(out_p, "asignado"), Some(*ganador), "{out_p}");
    let md = publicado(&f, &bare, "TASK-001");
    let id = if *ganador == "ana" { 1001 } else { 1002 };
    assert!(
        md.contains(&format!(
            "\nasignado_a: {{id: {id}, login: \"{ganador}\"}}\n"
        )),
        "{md}"
    );
    let rama = "feature/task-001-primera";
    assert_eq!(ramas_del_id(&f, dir_g, "TASK-001").trim(), rama);
    assert!(worktree(dir_g, "TASK-001").is_dir());
    assert_eq!(ramas_del_id(&f, dir_p, "TASK-001").trim(), "");
    assert!(!worktree(dir_p, "TASK-001").exists());
    // El perdedor no tiene rama de estado local ni refs propias.
    let refs = f.crudo(
        dir_p,
        &[
            "for-each-ref",
            "refs/heads/taskcodetwo/",
            "refs/taskcodetwo/",
        ],
    );
    assert!(refs.trim().is_empty(), "{refs}");
}

/// C2: repetir `start` siendo el dueno sale 0 `sin-cambios` sin commit nuevo
/// en el estado remoto; sin worktree, o sin worktree ni rama local, recrea
/// solo lo que falta; una tarea `en-revision` suya se reanuda desde la rama
/// publicada sin reescribir el estado.
#[test]
fn c2_repetir_y_reanudar_siendo_el_dueno() {
    let f = Fixture::nueva("toma-c2");
    let (bare, a) = preparar(&f, &[&["Primera"]]);
    let rama = "feature/task-001-primera";
    let (c, out) = start(&f, &a, "TASK-001");
    assert_eq!(c, Some(0), "{out}");
    let punta = punta_estado(&f, &bare);
    let (c, out) = start(&f, &a, "TASK-001");
    assert_eq!(
        (c, out.as_str()),
        (
            Some(0),
            format!(
                "start sin-cambios id=TASK-001 rama={rama} worktree=.taskcodetwo/worktrees/TASK-001 asignado=ana"
            )
            .as_str()
        )
    );
    assert_eq!(punta_estado(&f, &bare), punta, "sin-cambios publico algo");
    // Sin el worktree (borrado a mano): solo el worktree.
    std::fs::remove_dir_all(worktree(&a, "TASK-001")).unwrap();
    let (c, out) = start(&f, &a, "TASK-001");
    assert_eq!(c, Some(0), "{out}");
    assert_eq!(valor(&out, "creado"), Some("worktree"), "{out}");
    assert!(worktree(&a, "TASK-001").join("README.md").is_file());
    // Sin worktree ni rama local (y sin rama en el remoto): las dos, desde
    // la base.
    std::fs::remove_dir_all(worktree(&a, "TASK-001")).unwrap();
    f.crudo(&a, &["worktree", "prune"]);
    f.crudo(&a, &["branch", "-D", rama]);
    let (c, out) = start(&f, &a, "TASK-001");
    assert_eq!(c, Some(0), "{out}");
    assert_eq!(valor(&out, "creado"), Some("rama,worktree"), "{out}");
    assert_eq!(punta_estado(&f, &bare), punta);
    // En revision, con la rama publicada y perdidos rama local y worktree:
    // se reanuda desde `origin/<rama>` sin tocar el estado.
    escribir(&worktree(&a, "TASK-001"), "hecho.txt", "hecho\n");
    let wt = worktree(&a, "TASK-001");
    f.crudo(&wt, &["add", "hecho.txt"]);
    f.crudo(&wt, &["commit", "-q", "-m", "trabajo"]);
    git(&f, &wt, &["push", "-q", "origin", rama]);
    let publicada = f.crudo(&a, &["rev-parse", rama]);
    editar(
        &f,
        &a,
        "TASK-001",
        "estado: \"en-curso\"",
        "estado: \"en-revision\"",
    );
    let punta = punta_estado(&f, &bare);
    std::fs::remove_dir_all(&wt).unwrap();
    f.crudo(&a, &["worktree", "prune"]);
    f.crudo(&a, &["branch", "-D", rama]);
    let (c, out) = start(&f, &a, "TASK-001");
    assert_eq!(c, Some(0), "{out}");
    assert_eq!(valor(&out, "creado"), Some("rama,worktree"), "{out}");
    assert_eq!(
        punta_estado(&f, &bare),
        punta,
        "en revision reescribio el estado"
    );
    assert!(publicado(&f, &bare, "TASK-001").contains("estado: \"en-revision\""));
    assert_eq!(f.crudo(&wt, &["rev-parse", "HEAD"]), publicada);
    assert!(wt.join("hecho.txt").is_file());
}

/// C3: con el remoto inaccesible sale 4 `sinred pendiente=no` y no cambia
/// nada: ni la rama de estado ni pendientes, ni rama ni worktree.
#[test]
fn c3_sin_red_no_deja_nada() {
    let f = Fixture::nueva("toma-c3");
    let (bare, a) = preparar(&f, &[&["Primera"]]);
    // Con la rama de estado local, como tras cualquier escritura.
    git(
        &f,
        &a,
        &[
            "branch",
            "--no-track",
            RAMA_ESTADO,
            &format!("origin/{RAMA_ESTADO}"),
        ],
    );
    let antes = f.huella(&bare, &a);
    let o = lanzar(&a, entorno_sin_red(&f), &["start", "TASK-001"]);
    let out = texto(&o.stdout);
    assert_eq!(o.status.code(), Some(4), "{out}");
    assert!(out.starts_with("start sinred id=TASK-001 "), "{out}");
    assert_eq!(valor(out.trim_end(), "pendiente"), Some("no"), "{out}");
    assert_eq!(f.huella(&bare, &a), antes);
    let pendientes = f.crudo(&a, &["for-each-ref", "refs/taskcodetwo/"]);
    assert!(pendientes.trim().is_empty(), "{pendientes}");
    assert_eq!(ramas_del_id(&f, &a, "TASK-001").trim(), "");
    assert!(!worktree(&a, "TASK-001").exists());
}

/// C4: con una dependencia que no esta terminada (o esta descartada) sale 3
/// `motivo=dependencias-pendientes` con los IDs y nada escrito; terminada,
/// la toma sale.
#[test]
fn c4_dependencias_sin_terminar_no_dejan_empezar() {
    let f = Fixture::nueva("toma-c4");
    let (bare, a) = preparar(
        &f,
        &[
            &["Base"],
            &["Otra base"],
            &[
                "Encima",
                "--dependencia",
                "TASK-001",
                "--dependencia",
                "TASK-002",
            ],
        ],
    );
    editar(
        &f,
        &a,
        "TASK-002",
        "estado: \"planificada\"",
        "estado: \"terminada\"",
    );
    let antes = f.huella(&bare, &a);
    let (c, out) = start(&f, &a, "TASK-003");
    assert_eq!(c, Some(3), "{out}");
    assert_eq!(valor(&out, "motivo"), Some("dependencias-pendientes"));
    assert_eq!(valor(&out, "dependencias"), Some("TASK-001"));
    assert_eq!(f.huella(&bare, &a), antes);
    // Descartada tambien bloquea.
    editar(
        &f,
        &a,
        "TASK-001",
        "estado: \"planificada\"\nmotivo: null",
        "estado: \"descartada\"\nmotivo: \"no hace falta\"",
    );
    let antes = f.huella(&bare, &a);
    let (c, out) = start(&f, &a, "TASK-003");
    assert_eq!(
        (c, valor(&out, "dependencias")),
        (Some(3), Some("TASK-001"))
    );
    assert_eq!(f.huella(&bare, &a), antes);
    editar(
        &f,
        &a,
        "TASK-001",
        "estado: \"descartada\"",
        "estado: \"terminada\"",
    );
    let (c, out) = start(&f, &a, "TASK-003");
    assert_eq!(c, Some(0), "{out}");
}

/// C5: terminada, descartada, formato 1 o inexistente dan 3 con su motivo;
/// un ID mal formado, uso (2); nada escrito.
#[test]
fn c5_tareas_que_no_se_empiezan() {
    let f = Fixture::nueva("toma-c5");
    let (bare, a) = preparar(&f, &[&["Hecha"], &["Tirada"]]);
    editar(
        &f,
        &a,
        "TASK-001",
        "estado: \"planificada\"",
        "estado: \"terminada\"",
    );
    editar(
        &f,
        &a,
        "TASK-002",
        "estado: \"planificada\"\nmotivo: null",
        "estado: \"descartada\"\nmotivo: \"no hace falta\"",
    );
    escribir_estado(&f, &a, "TASK-003/tarea.md", |_| {
        "---\nid: TASK-003\ntitulo: \"Vieja\"\ntipo: feature\nsprint: 0\netiquetas: []\n\
         complejidad: null\nmodelo_sugerido: sonnet\nestado: planificada\nplan_aprobado: false\n\
         rama: feature/task-003-vieja\nasignado_a: null\nagente_revisor: general-purpose\n\
         skills_recomendados: []\nregla_seleccion_skill: null\nultimo_commit_revisado: null\n\
         revision_codex: false\ntokens_diseno: null\ntokens_implementacion: null\n\
         tokens_revision: null\ncreado: 2026-01-01\nactualizado: 2026-01-01\n\
         dependencias: []\n---\n"
            .to_owned()
    });
    let antes = f.huella(&bare, &a);
    for (id, motivo) in [
        ("TASK-001", "terminada"),
        ("TASK-002", "descartada"),
        ("TASK-003", "formato-1"),
        ("TASK-050", "no-existe"),
    ] {
        let (c, out) = start(&f, &a, id);
        assert_eq!(c, Some(3), "{out}");
        assert_eq!(valor(&out, "motivo"), Some(motivo), "{out}");
        assert_eq!(f.huella(&bare, &a), antes, "{id}");
    }
    let (_, out) = start(&f, &a, "TASK-003");
    assert_eq!(valor(&out, "arreglo"), Some("tc2%20actualizar"), "{out}");
    let o = tc2(&f, &a, &["start", "TASK-3"]);
    assert_eq!(o.status.code(), Some(2));
    assert_eq!(texto(&o.stdout), "start uso motivo=id-invalido\n");
    assert_eq!(f.huella(&bare, &a), antes);
}

/// C6: si el ID ya tiene rama `*/task-nnn-*` en el remoto y no la lleva
/// nadie, 3 `motivo=ocupada` sin escribir nada.
#[test]
fn c6_rama_del_id_en_el_remoto_sin_dueno_es_ocupada() {
    let f = Fixture::nueva("toma-c6");
    let (bare, a) = preparar(&f, &[&["Primera"]]);
    git(
        &f,
        &a,
        &[
            "push",
            "-q",
            "origin",
            "develop:refs/heads/fix/task-001-a-mano",
        ],
    );
    let antes = f.huella(&bare, &a);
    let (c, out) = start(&f, &a, "TASK-001");
    assert_eq!(c, Some(3), "{out}");
    assert_eq!(valor(&out, "motivo"), Some("ocupada"), "{out}");
    assert_eq!(valor(&out, "ramas"), Some("fix/task-001-a-mano"), "{out}");
    assert_eq!(f.huella(&bare, &a), antes);
    assert!(!worktree(&a, "TASK-001").exists());
}

/// MENOR-1 de la ronda 1: con la carpeta del worktree ocupada por otra cosa,
/// 3 `motivo=carpeta-ocupada` antes de publicar la toma: sin `estado=`, la
/// tarea sigue sin asignar en el remoto y no hay rama.
#[test]
fn carpeta_del_worktree_ocupada_no_publica_la_toma() {
    let f = Fixture::nueva("toma-carpeta");
    let (bare, a) = preparar(&f, &[&["Primera"]]);
    escribir(&worktree(&a, "TASK-001"), "basura.txt", "basura\n");
    let antes = f.huella(&bare, &a);
    let punta = punta_estado(&f, &bare);
    let (c, out) = start(&f, &a, "TASK-001");
    assert_eq!(c, Some(3), "{out}");
    assert!(out.starts_with("start rechazada id=TASK-001 "), "{out}");
    assert_eq!(valor(&out, "motivo"), Some("carpeta-ocupada"), "{out}");
    assert_eq!(valor(&out, "estado"), None, "{out}");
    assert_eq!(f.huella(&bare, &a), antes);
    assert_eq!(punta_estado(&f, &bare), punta);
    let md = publicado(&f, &bare, "TASK-001");
    assert!(md.contains("\nasignado_a: null\n"), "{md}");
    assert!(md.contains("\nestado: \"planificada\"\n"), "{md}");
    assert_eq!(ramas_del_id(&f, &a, "TASK-001").trim(), "");
    assert!(worktree(&a, "TASK-001").join("basura.txt").is_file());
    // Libre la carpeta, la toma sale.
    std::fs::remove_dir_all(worktree(&a, "TASK-001")).unwrap();
    let (c, out) = start(&f, &a, "TASK-001");
    assert_eq!(c, Some(0), "{out}");
}

/// M2 de la ronda 1: una `rama:` publicada que no es de la tarea da 3
/// `motivo=rama-invalida` sin escribir nada.
#[test]
fn rama_mal_formada_es_rama_invalida() {
    let f = Fixture::nueva("toma-rama");
    let (bare, a) = preparar(&f, &[&["Primera"]]);
    editar(
        &f,
        &a,
        "TASK-001",
        "rama: \"feature/task-001-primera\"",
        "rama: \"feature/task-002-de-otra\"",
    );
    let antes = f.huella(&bare, &a);
    let (c, out) = start(&f, &a, "TASK-001");
    assert_eq!(c, Some(3), "{out}");
    assert_eq!(valor(&out, "motivo"), Some("rama-invalida"), "{out}");
    assert_eq!(valor(&out, "estado"), None, "{out}");
    assert_eq!(f.huella(&bare, &a), antes);
    assert!(publicado(&f, &bare, "TASK-001").contains("\nasignado_a: null\n"));
    assert_eq!(ramas_del_id(&f, &a, "TASK-001").trim(), "");
    assert!(!worktree(&a, "TASK-001").exists());
}

/// C7: tras `ok`, el `tarea.md` publicado cambia solo en estado, asignado,
/// actualizado y una fila de transicion; `ROADMAP.md` la pone en marcha; la
/// rama sale de la punta de `origin/develop` sin upstream; el worktree en
/// `.taskcodetwo/worktrees/TASK-001`; el principal, limpio.
#[test]
fn c7_lo_publicado_la_rama_y_el_worktree() {
    let f = Fixture::nueva("toma-c7");
    let (bare, a) = preparar(&f, &[&["Primera", "--criterio", "uno"]]);
    let antes = publicado(&f, &bare, "TASK-001");
    let (c, out) = start(&f, &a, "TASK-001");
    assert_eq!(c, Some(0), "{out}");
    // La primera toma crea rama y worktree, pero no lo dice con `creado=`
    // (eso es de reanudar).
    assert!(valor(&out, "creado").is_none(), "{out}");
    let rama = "feature/task-001-primera";
    let despues = publicado(&f, &bare, "TASK-001");
    let hoy = tc2::toma::instante(tc2::identidad::ahora());
    let creado = antes
        .lines()
        .find_map(|l| l.strip_prefix("actualizado: "))
        .unwrap()
        .to_owned();
    let esperado = antes
        .replacen("estado: \"planificada\"", "estado: \"en-curso\"", 1)
        .replacen(
            "asignado_a: null",
            "asignado_a: {id: 1001, login: \"ana\"}",
            1,
        )
        .replacen(
            &format!("actualizado: {creado}"),
            &format!("actualizado: \"{}\"", &hoy[..10]),
            1,
        )
        // F3-13: la toma recalcula la estimacion (sin terminadas, sin-datos);
        // el resto, byte a byte.
        .replacen(&estimacion(&antes), &estimacion(&despues), 1);
    assert!(
        estimacion(&despues).contains("\n  nivel: \"sin-datos\"\n  n: 0\n"),
        "{despues}"
    );
    let fila = despues.lines().last().unwrap();
    assert_eq!(despues, format!("{esperado}{fila}\n"));
    assert!(fila.starts_with(&format!("| {}T", &hoy[..10])), "{fila}");
    assert!(
        fila.ends_with(&format!(" | start | manual | agente | {} |", tc2::VERSION)),
        "{fila}"
    );
    let roadmap = f.crudo(&bare, &["show", &format!("{RAMA_ESTADO}:ROADMAP.md")]);
    assert!(
        roadmap.contains("- **En marcha:** TASK-001 (en curso)\n"),
        "{roadmap}"
    );
    assert_eq!(
        f.crudo(&a, &["rev-parse", rama]),
        f.crudo(&a, &["rev-parse", "origin/develop"])
    );
    let upstream = f.crudo(
        &a,
        &[
            "for-each-ref",
            "--format=%(upstream)",
            &format!("refs/heads/{rama}"),
        ],
    );
    assert_eq!(upstream.trim(), "", "la rama tiene upstream");
    let wt = worktree(&a, "TASK-001");
    assert_eq!(
        f.crudo(&wt, &["rev-parse", "--abbrev-ref", "HEAD"]).trim(),
        rama
    );
    assert_eq!(
        valor(&out, "worktree"),
        Some(".taskcodetwo/worktrees/TASK-001")
    );
    assert_eq!(f.crudo(&a, &["status", "--porcelain"]), "");
    assert_eq!(f.crudo(&wt, &["status", "--porcelain"]), "");
}

/// El bloque `estimacion:` de un `tarea.md` (F3-13).
fn estimacion(md: &str) -> String {
    let i = md.find("\nestimacion:").unwrap() + 1;
    let fin = md[i..].find("\ntokens_diseno:").unwrap() + i + 1;
    md[i..fin].to_owned()
}

/// C8: sin terminal y sin cache de identidad que valga (no hay, o caducada)
/// sale 3 `motivo=identidad` y nada escrito.
#[test]
fn c8_sin_identidad_no_se_toma() {
    let f = Fixture::nueva("toma-c8");
    let (bare, a) = preparar(&f, &[&["Primera"]]);
    let cache = Cache::en(&a.join(".git"));
    std::fs::remove_file(cache.ruta()).unwrap();
    let antes = f.huella(&bare, &a);
    let (c, out) = start(&f, &a, "TASK-001");
    assert_eq!(c, Some(3), "{out}");
    assert_eq!(valor(&out, "motivo"), Some("identidad"), "{out}");
    assert_eq!(valor(&out, "cache"), Some("falta"), "{out}");
    assert_eq!(f.huella(&bare, &a), antes);
    let o = tc2(&f, &a, &["start", "TASK-001"]);
    assert!(
        texto(&o.stderr).contains("tc2 identidad"),
        "{}",
        texto(&o.stderr)
    );
    // Caducada: igual.
    let vieja = Cuenta {
        proveedor: Proveedor::GitHub,
        id: 1001,
        login: "ana".to_owned(),
    };
    cache
        .guardar(
            &vieja,
            tc2::identidad::ahora() - tc2::identidad::TTL_SEGUNDOS - 10,
        )
        .unwrap();
    let (c, out) = start(&f, &a, "TASK-001");
    assert_eq!(
        (c, valor(&out, "cache")),
        (Some(3), Some("caducada")),
        "{out}"
    );
    assert_eq!(f.huella(&bare, &a), antes);
}

/// C9: lanzado desde el worktree de otra tarea escribe el estado en el
/// principal y crea el worktree junto al principal; con `--json`, una linea
/// de JSON con las mismas claves.
#[test]
fn c9_desde_un_worktree_de_tarea_y_en_json() {
    let f = Fixture::nueva("toma-c9");
    let (bare, a) = preparar(&f, &[&["Primera"], &["Segunda"]]);
    let (c, out) = start(&f, &a, "TASK-001");
    assert_eq!(c, Some(0), "{out}");
    let wt1 = worktree(&a, "TASK-001");
    let o = tc2(&f, &wt1, &["start", "--json", "TASK-002"]);
    let out = texto(&o.stdout);
    assert_eq!(o.status.code(), Some(0), "{out} {}", texto(&o.stderr));
    assert_eq!(out.lines().count(), 1, "{out}");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["comando"], "start");
    assert_eq!(v["resultado"], "ok");
    assert_eq!(v["codigo"], 0);
    let claves = &v["claves"];
    assert_eq!(claves["id"], "TASK-002");
    assert_eq!(claves["rama"], "feature/task-002-segunda");
    assert_eq!(claves["worktree"], ".taskcodetwo/worktrees/TASK-002");
    assert_eq!(claves["asignado"], "ana");
    assert!(publicado(&f, &bare, "TASK-002").contains("estado: \"en-curso\""));
    assert!(worktree(&a, "TASK-002").is_dir());
    assert!(!worktree(&wt1, "TASK-002").exists());
    assert!(!wt1.join("tareas").exists());
    assert_eq!(f.crudo(&wt1, &["status", "--porcelain"]), "");
    assert_eq!(f.crudo(&a, &["status", "--porcelain"]), "");
    // Un rechazo en JSON tambien es una linea, con el motivo.
    let o = tc2(&f, &wt1, &["start", "--json", "TASK-050"]);
    let out = texto(&o.stdout);
    assert_eq!(
        (o.status.code(), out.lines().count()),
        (Some(3), 1),
        "{out}"
    );
    assert!(out.contains("\"motivo\":\"no-existe\""), "{out}");
}

/// D4 de TASK-033: un `hotfix` sale del ultimo tag de `origin/main` (o de su
/// punta sin tags) y un `release` no se empieza con `start`.
#[test]
fn d4_hotfix_desde_el_ultimo_tag_y_release_rechazada() {
    let f = Fixture::nueva("toma-d4");
    let (bare, a) = preparar(
        &f,
        &[
            &["Parche", "--tipo", "hotfix"],
            &["Version", "--tipo", "release"],
            &["Sin tags", "--tipo", "hotfix"],
        ],
    );
    // Sin tags: la punta de origin/main.
    let (c, out) = start(&f, &a, "TASK-003");
    assert_eq!(c, Some(0), "{out}");
    assert_eq!(
        f.crudo(&a, &["rev-parse", "hotfix/task-003-sin-tags"]),
        f.crudo(&a, &["rev-parse", "origin/main"])
    );
    // Con tags en main y main por delante: el ultimo tag por version.
    let prep = f.raiz.join("clon");
    f.crudo(&prep, &["checkout", "-q", "main"]);
    f.crudo(&prep, &["tag", "-a", "v0.9.0", "-m", "v0.9.0"]);
    f.crudo(&prep, &["commit", "-q", "--allow-empty", "-m", "uno"]);
    f.crudo(&prep, &["tag", "-a", "v0.10.0", "-m", "v0.10.0"]);
    f.crudo(&prep, &["commit", "-q", "--allow-empty", "-m", "dos"]);
    f.crudo(&prep, &["push", "-q", "--follow-tags", "origin", "main"]);
    // Un tag mas nuevo en una rama que no esta fusionada en main no cuenta
    // (I1 de la ronda 1): gana el ultimo tag contenido en origin/main.
    f.crudo(&prep, &["checkout", "-q", "-b", "sin-fusionar"]);
    f.crudo(&prep, &["commit", "-q", "--allow-empty", "-m", "fuera"]);
    f.crudo(&prep, &["tag", "-a", "v1.0.0", "-m", "v1.0.0"]);
    f.crudo(&prep, &["push", "-q", "origin", "sin-fusionar", "v1.0.0"]);
    f.crudo(&prep, &["checkout", "-q", "main"]);
    let (c, out) = start(&f, &a, "TASK-001");
    assert_eq!(c, Some(0), "{out}");
    // El tag de fuera llego al clon: si no, la prueba no distinguiria nada.
    assert_ne!(
        f.crudo(&a, &["rev-parse", "v1.0.0^{commit}"]),
        f.crudo(&a, &["rev-parse", "v0.10.0^{commit}"])
    );
    assert_eq!(
        f.crudo(&a, &["rev-parse", "hotfix/task-001-parche"]),
        f.crudo(&a, &["rev-parse", "v0.10.0^{commit}"])
    );
    let antes = f.huella(&bare, &a);
    let (c, out) = start(&f, &a, "TASK-002");
    assert_eq!(
        (c, valor(&out, "motivo")),
        (Some(3), Some("tipo-release")),
        "{out}"
    );
    assert_eq!(f.huella(&bare, &a), antes);
}
