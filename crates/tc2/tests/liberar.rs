//! F3-10 (TASK-081): `tc2 liberar` y `tc2 reasignar` con el binario real,
//! como los lanza el agente (sin terminal: quien los lanza sale de la cache
//! de identidad), y con la biblioteca cuando hace falta una persona en un
//! terminal (`Quien::Persona`, que el binario solo da con un token real).
//!
//! Como en `tests/ciclo.rs`: `origin` apunta a `ssh://git@github.com/o/r.git`
//! y el transporte ssh se sustituye con `GIT_SSH_COMMAND`, que sirve el
//! remoto bare de la fixture; sin red, un «ssh» que responde «Connection
//! refused».

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use comun::{Fixture, escribir};
use tc2::contrato::{Informe, Resultado};
use tc2::estado::{self, Cambio, Contexto, Estado, Fallo, Operacion};
use tc2::identidad::{Cache, Cuenta, Quien};
use tc2::plataforma::Proveedor;

const RAMA_ESTADO: &str = estado::RAMA;
const ORIGIN: &str = "ssh://git@github.com/o/r.git";
const PERSONAS: &str =
    "---\npersonas:\n  - {id: 1001, login: \"ana\"}\n  - {id: 1002, login: \"beto\"}\n";

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
    Command::new(env!("CARGO_BIN_EXE_tc2"))
        .current_dir(dir)
        .args(args)
        .envs(entorno)
        .env("LC_ALL", "C")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env_remove("TC2_TOKEN_PERSONA")
        .env_remove("TC2_TOKEN_AGENTE")
        .output()
        .expect("lanzar tc2")
}

fn texto(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

/// `tc2 <args>` en `dir` con el remoto: (codigo, la unica linea de stdout).
fn tc2(f: &Fixture, dir: &Path, args: &[&str]) -> (Option<i32>, String) {
    salida(&lanzar(dir, entorno_ssh(f), args))
}

/// Un exito no escribe en stderr; un fallo, una linea.
fn salida(o: &Output) -> (Option<i32>, String) {
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

fn git(f: &Fixture, dir: &Path, args: &[&str]) -> String {
    git_con(f, dir, args, &[])
}

fn git_con(f: &Fixture, dir: &Path, args: &[&str], env: &[(&str, &str)]) -> String {
    let o = Command::new("git")
        .current_dir(dir)
        .args(args)
        .envs(entorno_ssh(f))
        .envs(env.iter().copied())
        .env("LC_ALL", "C")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("lanzar git");
    assert!(o.status.success(), "git {args:?}: {}", texto(&o.stderr));
    texto(&o.stdout)
}

fn valor<'a>(linea: &'a str, clave: &str) -> Option<&'a str> {
    linea
        .split(' ')
        .find_map(|kv| kv.strip_prefix(clave)?.strip_prefix('='))
}

fn cuenta(id: u64, login: &str) -> Cuenta {
    Cuenta {
        proveedor: Proveedor::GitHub,
        id,
        login: login.to_owned(),
    }
}

fn identificar(dir: &Path, id: u64, login: &str) {
    Cache::en(&dir.join(".git"))
        .guardar(&cuenta(id, login), tc2::identidad::ahora())
        .unwrap();
}

/// Remoto con `personas.yml` (y `config`, si se da) y las tareas de
/// `titulos`; el clon `a` tiene `origin` en GitHub y la identidad de ana.
fn preparar(f: &Fixture, titulos: &[&str], config: Option<&str>) -> (PathBuf, PathBuf) {
    let (bare, prep) = f.remoto_y_clon();
    escribir(&prep, ".taskcode/personas.yml", PERSONAS);
    f.crudo(&prep, &["add", ".taskcode/personas.yml"]);
    if let Some(c) = config {
        escribir(&prep, ".taskcode/config.yml", c);
        f.crudo(&prep, &["add", ".taskcode/config.yml"]);
    }
    f.crudo(&prep, &["commit", "-q", "-m", "personas"]);
    f.crudo(&prep, &["push", "-q", "origin", "develop"]);
    for t in titulos {
        let o = lanzar(&prep, f.entorno(), &["new", t]);
        assert_eq!(o.status.code(), Some(0), "{}", texto(&o.stdout));
    }
    let a = otro_clon(f, &bare, "a");
    identificar(&a, 1001, "ana");
    (bare, a)
}

fn otro_clon(f: &Fixture, bare: &Path, nombre: &str) -> PathBuf {
    let d = f.otro_clon(bare, nombre);
    f.crudo(&d, &["remote", "set-url", "origin", ORIGIN]);
    d
}

/// El clon de beto, con su identidad.
fn clon_de_beto(f: &Fixture, bare: &Path) -> PathBuf {
    let b = otro_clon(f, bare, "b");
    identificar(&b, 1002, "beto");
    b
}

fn tarea(f: &Fixture, bare: &Path, id: &str) -> String {
    f.crudo(bare, &["show", &format!("{RAMA_ESTADO}:{id}/tarea.md")])
}

fn punta(f: &Fixture, bare: &Path) -> String {
    f.crudo(bare, &["rev-parse", RAMA_ESTADO])
}

fn worktree(dir: &Path, id: &str) -> PathBuf {
    dir.join(".taskcodetwo").join("worktrees").join(id)
}

/// Un commit en el worktree de la tarea y su rama publicada; con `fecha`,
/// la del commit (autor y committer).
fn trabajar(f: &Fixture, dir: &Path, id: &str, rama: &str, fecha: Option<&str>) -> String {
    let wt = worktree(dir, id);
    let n = std::fs::read_dir(&wt).unwrap().count();
    escribir(&wt, &format!("trabajo-{n}.txt"), "hecho\n");
    git(f, &wt, &["add", "."]);
    let env: Vec<(&str, &str)> = fecha
        .map(|d| vec![("GIT_COMMITTER_DATE", d), ("GIT_AUTHOR_DATE", d)])
        .unwrap_or_default();
    git_con(f, &wt, &["commit", "-q", "-m", "trabajo"], &env);
    git(f, &wt, &["push", "-q", "origin", rama]);
    git(f, &wt, &["rev-parse", "HEAD"]).trim().to_owned()
}

/// Lo que `tarea.md` gana con una fila: `antes` con `actualizado` de hoy y
/// la fila `fase` de `por` (la ultima linea de `despues`).
fn con_fila(antes: &str, despues: &str, fase: &str, por: &str) -> String {
    let hoy = tc2::toma::instante(tc2::identidad::ahora());
    let actualizado = antes
        .lines()
        .find_map(|l| l.strip_prefix("actualizado: "))
        .unwrap()
        .to_owned();
    let fila = despues.lines().last().unwrap();
    assert!(fila.starts_with(&format!("| {}T", &hoy[..10])), "{fila}");
    assert!(
        fila.ends_with(&format!(" | {fase} | manual | {por} | {} |", tc2::VERSION)),
        "{fila}"
    );
    format!(
        "{}{fila}\n",
        antes.replacen(
            &format!("actualizado: {actualizado}"),
            &format!("actualizado: \"{}\"", &hoy[..10]),
            1
        )
    )
}

/// Escribe en la rama de estado por la transaccion (como otra fase): con
/// `de`, lo cambia por `a`; sin el, envejece la fecha de cada fila de
/// `## Transiciones` a 2020-01-01.
struct Escribir {
    ruta: String,
    de: Option<String>,
    a: String,
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
        let nuevo = match &self.de {
            Some(de) => {
                assert!(t.contains(de.as_str()), "{t}");
                t.replacen(de.as_str(), &self.a, 1)
            }
            None => t
                .split_inclusive('\n')
                .map(|l| match l.strip_prefix("| 20") {
                    Some(resto) if resto.get(18..20) == Some(" |") => {
                        format!("| 2020-01-01T00:00:00Z{}", &resto[18..])
                    }
                    _ => l.to_owned(),
                })
                .collect(),
        };
        Ok(vec![Cambio::nuevo(self.ruta.clone(), nuevo)])
    }
}

fn sin_catalogo(_: &str) -> Option<Box<dyn Operacion>> {
    None
}

fn transaccion(f: &Fixture, dir: &Path, op: &Escribir) {
    let g = tc2::git::Git::nuevo().con_entorno(entorno_ssh(f));
    let i = estado::transaccion(&Contexto::nuevo(&g, dir, &sin_catalogo, None), op);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
}

/// Cambia `de` por `a` en el `tarea.md` publicado de `id`.
fn editar(f: &Fixture, dir: &Path, id: &str, de: &str, a: &str) {
    let op = Escribir {
        ruta: format!("{id}/tarea.md"),
        de: Some(de.to_owned()),
        a: a.to_owned(),
    };
    transaccion(f, dir, &op);
}

/// Envejece las filas de transicion publicadas de `id`.
fn envejecer(f: &Fixture, dir: &Path, id: &str) {
    let op = Escribir {
        ruta: format!("{id}/tarea.md"),
        de: None,
        a: String::new(),
    };
    transaccion(f, dir, &op);
    assert!(
        tarea(f, &f.raiz.join("remoto.git"), id).contains("| 2020-01-01T00:00:00Z | start |"),
        "no se envejecio"
    );
}

/// `liberar` (sin `destino`) o `reasignar` con la biblioteca, lanzado por
/// una persona en un terminal: la linea de contrato.
fn como_persona(f: &Fixture, dir: &Path, quien: Cuenta, id: &str, destino: Option<&str>) -> String {
    let g = tc2::git::Git::nuevo().con_entorno(entorno_ssh(f));
    let q = move || -> Result<Quien, Informe> { Ok(Quien::Persona(quien.clone())) };
    let e = tc2::toma::Entorno {
        dir,
        git: &g,
        quien: &q,
        catalogo: &sin_catalogo,
        ahora: tc2::identidad::ahora(),
    };
    tc2::ciclo::dejar_en(&e, id, destino).linea()
}

/// C1 y C2: el dueno libera su tarea en curso (publicada): `planificada`,
/// `asignado_a: null` y la fila, con rama, `plan_aprobado` y el resto byte a
/// byte; el ROADMAP en el mismo commit; `asignado-anterior=ana`; las ramas
/// local y remota, en el mismo commit. Otra persona hace `start` en otro
/// clon y sigue en esa rama con los commits de la primera.
#[test]
fn c1_c2_el_dueno_libera_y_otra_persona_la_retoma() {
    let f = Fixture::nueva("liberar-c1");
    let (bare, a) = preparar(&f, &["Uno"], None);
    let (c, out) = tc2(&f, &a, &["start", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    let rama = "feature/task-001-uno";
    let de_ana = trabajar(&f, &a, "TASK-001", rama, None);
    let antes = tarea(&f, &bare, "TASK-001");
    let (c, out) = tc2(&f, &a, &["liberar", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    assert!(out.starts_with("liberar ok id=TASK-001 commit="), "{out}");
    assert_eq!(valor(&out, "asignado-anterior"), Some("ana"), "{out}");
    assert_eq!(valor(&out, "worktree"), Some("quitado"), "{out}");
    let despues = tarea(&f, &bare, "TASK-001");
    let esperado = con_fila(
        &antes
            .replacen("estado: \"en-curso\"", "estado: \"planificada\"", 1)
            .replacen(
                "asignado_a: {id: 1001, login: \"ana\"}",
                "asignado_a: null",
                1,
            ),
        &despues,
        "liberar",
        "agente",
    );
    assert_eq!(despues, esperado);
    assert!(
        despues.contains(&format!("\nrama: \"{rama}\"\n")),
        "{despues}"
    );
    let cambiados = f.crudo(&bare, &["show", "--name-only", "--format=", RAMA_ESTADO]);
    let mut cambiados: Vec<&str> = cambiados.lines().collect();
    cambiados.sort_unstable();
    assert_eq!(cambiados, ["ROADMAP.md", "TASK-001/tarea.md"]);
    let local = f.crudo(&a, &["rev-parse", &format!("refs/heads/{rama}")]);
    let remota = f.crudo(&bare, &["rev-parse", &format!("refs/heads/{rama}")]);
    assert_eq!(
        (local.trim(), remota.trim()),
        (de_ana.as_str(), de_ana.as_str())
    );
    // C2: beto la empieza en su clon y sigue en la misma rama.
    let b = clon_de_beto(&f, &bare);
    let (c, out) = tc2(&f, &b, &["start", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    assert!(out.starts_with("start ok id=TASK-001 "), "{out}");
    assert_eq!(valor(&out, "asignado"), Some("beto"), "{out}");
    let suya = f.crudo(&b, &["rev-parse", &format!("refs/heads/{rama}")]);
    assert_eq!(suya.trim(), de_ana);
    f.crudo(&b, &["merge-base", "--is-ancestor", &de_ana, rama]);
    let wt = worktree(&b, "TASK-001");
    assert_eq!(git(&f, &wt, &["rev-parse", "HEAD"]).trim(), de_ana);
    assert!(tarea(&f, &bare, "TASK-001").contains("\nasignado_a: {id: 1002, login: \"beto\"}\n"));
}

/// Paso 7 del plan: aceptar la rama propia no debilita `ocupada`; con la
/// propia y otra `*/task-001-*` ajena, `start` sigue dando 3 `ocupada` y
/// solo nombra la ajena.
#[test]
fn la_rama_propia_no_debilita_ocupada() {
    let f = Fixture::nueva("liberar-ocupada");
    let (bare, a) = preparar(&f, &["Uno"], None);
    let (c, out) = tc2(&f, &a, &["start", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    trabajar(&f, &a, "TASK-001", "feature/task-001-uno", None);
    let (c, out) = tc2(&f, &a, &["liberar", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
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
    let b = clon_de_beto(&f, &bare);
    let antes = f.huella(&bare, &b);
    let (c, out) = tc2(&f, &b, &["start", "TASK-001"]);
    assert_eq!(c, Some(3), "{out}");
    assert_eq!(valor(&out, "motivo"), Some("ocupada"), "{out}");
    assert_eq!(valor(&out, "ramas"), Some("fix/task-001-a-mano"), "{out}");
    assert_eq!(f.huella(&bare, &b), antes);
}

/// C3: reasignar a beto (escrito con mayusculas) cambia solo `asignado_a`
/// al objeto de la lista y anade la fila, sin cambiar el estado; beto hace
/// `start` y obtiene la rama existente sin perder commits.
#[test]
fn c3_reasignar_y_el_nuevo_sigue_en_la_rama() {
    let f = Fixture::nueva("liberar-c3");
    let (bare, a) = preparar(&f, &["Uno"], None);
    let (c, out) = tc2(&f, &a, &["start", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    let rama = "feature/task-001-uno";
    let de_ana = trabajar(&f, &a, "TASK-001", rama, None);
    let antes = tarea(&f, &bare, "TASK-001");
    let (c, out) = tc2(&f, &a, &["reasignar", "TASK-001", "BETO"]);
    assert_eq!(c, Some(0), "{out}");
    assert!(out.starts_with("reasignar ok id=TASK-001 commit="), "{out}");
    assert_eq!(valor(&out, "asignado-anterior"), Some("ana"), "{out}");
    assert_eq!(valor(&out, "asignado"), Some("beto"), "{out}");
    let despues = tarea(&f, &bare, "TASK-001");
    let esperado = con_fila(
        &antes.replacen(
            "asignado_a: {id: 1001, login: \"ana\"}",
            "asignado_a: {id: 1002, login: \"beto\"}",
            1,
        ),
        &despues,
        "reasignar",
        "agente",
    );
    assert_eq!(despues, esperado);
    assert!(despues.contains("\nestado: \"en-curso\"\n"));
    let b = clon_de_beto(&f, &bare);
    let (c, out) = tc2(&f, &b, &["start", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    assert_eq!(
        f.crudo(&b, &["rev-parse", &format!("refs/heads/{rama}")])
            .trim(),
        de_ana
    );
    assert!(worktree(&b, "TASK-001").is_dir());
}

/// C4: un login que no esta en la lista, mal escrito o un bot da 3
/// `fuera-de-lista` sin commit en la rama de estado.
#[test]
fn c4_fuera_de_lista_sin_commit() {
    let f = Fixture::nueva("liberar-c4");
    let (bare, a) = preparar(&f, &["Uno"], None);
    let p = punta(&f, &bare);
    for login in ["carla", "bet0", "dependabot[bot]"] {
        let (c, out) = tc2(&f, &a, &["reasignar", "TASK-001", login]);
        assert_eq!(c, Some(3), "{out}");
        assert!(out.starts_with("reasignar rechazada id=TASK-001 "), "{out}");
        assert_eq!(valor(&out, "motivo"), Some("fuera-de-lista"), "{out}");
        assert_eq!(punta(&f, &bare), p, "{login}");
    }
}

/// C5, C6 y C10: la tarea de beto con actividad reciente no la libera ni
/// la reasigna nadie mas (3 `activa dias=0 umbral=3`, el umbral de la
/// configuracion), tampoco con las filas viejas si su rama tiene un commit
/// reciente; con filas y rama envejecidas, el agente de ana sigue sin poder
/// (`requiere-persona`) y ana en un terminal si. En el clon de beto, `status`
/// avisa `ya-no-tuyas=` con quien la lleva ahora.
#[test]
fn c5_c6_c10_tarea_de_otra_persona_por_actividad() {
    let f = Fixture::nueva("liberar-c5");
    let (bare, a) = preparar(
        &f,
        &["Uno", "Dos"],
        Some("---\nformato: 2\ndias_sin_actividad: 3\n"),
    );
    let b = clon_de_beto(&f, &bare);
    for (id, rama) in [
        ("TASK-001", "feature/task-001-uno"),
        ("TASK-002", "feature/task-002-dos"),
    ] {
        // F3-11 y TASK-128: las dos son de beto, asi que el `start` de
        // TASK-002 no avisa de TASK-001 (ya publicada) ni saca la `nota:`:
        // nadie se avisa contra si mismo. stderr vacio en los dos.
        let o = lanzar(&b, entorno_ssh(&f), &["start", id]);
        let out = texto(&o.stdout);
        assert_eq!(o.status.code(), Some(0), "{out}");
        assert_eq!(out.lines().count(), 1, "{out}");
        assert!(out.starts_with(&format!("start ok id={id} ")), "{out}");
        assert!(
            !out.contains(" solape") && !out.contains(" en-curso-modulos="),
            "{out}"
        );
        let err = texto(&o.stderr);
        assert!(err.is_empty(), "stderr en un exito: {err}");
        trabajar(&f, &b, id, rama, None);
    }
    // La orden trae antes de decidir: lo que traiga no es una escritura.
    git(&f, &a, &["fetch", "-q", "origin"]);
    let antes = f.huella(&bare, &a);
    for args in [
        vec!["liberar", "TASK-001"],
        vec!["reasignar", "TASK-001", "ana"],
    ] {
        let (c, out) = tc2(&f, &a, &args);
        assert_eq!(c, Some(3), "{out}");
        assert_eq!(valor(&out, "motivo"), Some("activa"), "{out}");
        assert_eq!(valor(&out, "dias"), Some("0"), "{out}");
        assert_eq!(valor(&out, "umbral"), Some("3"), "{out}");
        assert_eq!(valor(&out, "asignado"), Some("beto"), "{out}");
        assert_eq!(f.huella(&bare, &a), antes, "{args:?}");
    }
    // Filas viejas, rama con un commit reciente: sigue activa.
    for id in ["TASK-001", "TASK-002"] {
        envejecer(&f, &a, id);
    }
    let out = como_persona(&f, &a, cuenta(1001, "ana"), "TASK-001", None);
    assert_eq!(valor(&out, "motivo"), Some("activa"), "{out}");
    // La rama tambien envejecida: el agente no puede, ana en persona si.
    let viejo = Some("2020-01-02T00:00:00Z");
    trabajar(&f, &b, "TASK-001", "feature/task-001-uno", viejo);
    trabajar(&f, &b, "TASK-002", "feature/task-002-dos", viejo);
    let p = punta(&f, &bare);
    let (c, out) = tc2(&f, &a, &["liberar", "TASK-001"]);
    assert_eq!(c, Some(3), "{out}");
    assert_eq!(valor(&out, "motivo"), Some("requiere-persona"), "{out}");
    assert_eq!(valor(&out, "asignado"), Some("beto"), "{out}");
    assert_eq!(punta(&f, &bare), p);
    let out = como_persona(&f, &a, cuenta(1001, "ana"), "TASK-001", Some("ana"));
    assert!(out.starts_with("reasignar ok id=TASK-001 commit="), "{out}");
    assert_eq!(valor(&out, "asignado-anterior"), Some("beto"), "{out}");
    assert_eq!(valor(&out, "asignado"), Some("ana"), "{out}");
    let md = tarea(&f, &bare, "TASK-001");
    assert!(md.ends_with(&format!(
        " | reasignar | manual | persona | {} |\n",
        tc2::VERSION
    )));
    let out = como_persona(&f, &a, cuenta(1001, "ana"), "TASK-002", None);
    assert!(out.starts_with("liberar ok id=TASK-002 commit="), "{out}");
    assert_eq!(valor(&out, "asignado-anterior"), Some("beto"), "{out}");
    assert!(tarea(&f, &bare, "TASK-002").contains("\nestado: \"planificada\"\n"));
    // C10: beto conserva sus worktrees; `status` le avisa con codigo 0.
    let o = lanzar(&b, entorno_ssh(&f), &["status", "--json"]);
    assert_eq!(o.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_str(&texto(&o.stdout)).unwrap();
    assert_eq!(
        v["claves"]["ya-no-tuyas"],
        serde_json::json!(["TASK-001", "TASK-002"])
    );
    let err = texto(&o.stderr);
    assert!(
        err.contains("aviso: TASK-001 ya no es tuya: la lleva @ana;")
            && err.contains("aviso: TASK-002 ya no es tuya: esta libre"),
        "{err}"
    );
    assert!(worktree(&b, "TASK-001").is_dir());
    // En el clon de ana no hay aviso: no tiene esos worktrees.
    let o = lanzar(&a, entorno_ssh(&f), &["status", "--json"]);
    assert!(
        !texto(&o.stdout).contains("ya-no-tuyas"),
        "{}",
        texto(&o.stdout)
    );
}

/// C6 (la propia): sin terminal, sobre una tarea propia procede (tambien
/// `reasignar`; `liberar` lo prueba C1).
#[test]
fn c6_el_agente_reasigna_la_propia() {
    let f = Fixture::nueva("liberar-c6");
    let (_bare, a) = preparar(&f, &["Uno"], None);
    let (c, out) = tc2(&f, &a, &["start", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    trabajar(&f, &a, "TASK-001", "feature/task-001-uno", None);
    let (c, out) = tc2(&f, &a, &["reasignar", "TASK-001", "beto"]);
    assert_eq!(c, Some(0), "{out}");
    assert_eq!(valor(&out, "asignado"), Some("beto"), "{out}");
}

/// C7: sin red `liberar` y `reasignar` dan 4 `sinred pendiente=no` sin
/// commit ni ref pendiente.
#[test]
fn c7_sin_red_no_deja_nada() {
    let f = Fixture::nueva("liberar-c7");
    let (bare, a) = preparar(&f, &["Uno"], None);
    let (c, out) = tc2(&f, &a, &["start", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    let antes = f.huella(&bare, &a);
    for args in [
        vec!["liberar", "TASK-001"],
        vec!["reasignar", "TASK-001", "beto"],
    ] {
        let (c, out) = salida(&lanzar(&a, entorno_sin_red(&f), &args));
        assert_eq!(c, Some(4), "{out}");
        assert!(
            out.starts_with(&format!("{} sinred id=TASK-001", args[0])),
            "{out}"
        );
        assert_eq!(valor(&out, "pendiente"), Some("no"), "{out}");
        assert_eq!(f.huella(&bare, &a), antes, "{args:?}");
        assert!(
            f.crudo(&a, &["for-each-ref", estado::REFS_PENDIENTES])
                .trim()
                .is_empty()
        );
    }
}

/// C8: con cambios sin guardar o commits sin publicar en el worktree, 3
/// `sin-publicar` antes de la transaccion con el estado intacto; publicado
/// y limpio, `worktree=quitado` y la rama sigue.
#[test]
fn c8_trabajo_sin_publicar_no_se_suelta() {
    let f = Fixture::nueva("liberar-c8");
    let (bare, a) = preparar(&f, &["Uno"], None);
    let (c, out) = tc2(&f, &a, &["start", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    let rama = "feature/task-001-uno";
    let wt = worktree(&a, "TASK-001");
    escribir(&wt, "a-medias.txt", "sin guardar\n");
    let p = punta(&f, &bare);
    let (c, out) = tc2(&f, &a, &["liberar", "TASK-001"]);
    assert_eq!(c, Some(3), "{out}");
    assert_eq!(valor(&out, "motivo"), Some("sin-publicar"), "{out}");
    assert_eq!(
        valor(&out, "worktree-motivo"),
        Some("cambios-sin-guardar"),
        "{out}"
    );
    assert_eq!(punta(&f, &bare), p);
    git(&f, &wt, &["add", "a-medias.txt"]);
    git(&f, &wt, &["commit", "-q", "-m", "local"]);
    let (c, out) = tc2(&f, &a, &["reasignar", "TASK-001", "beto"]);
    assert_eq!(c, Some(3), "{out}");
    assert_eq!(
        valor(&out, "worktree-motivo"),
        Some("sin-publicar"),
        "{out}"
    );
    assert_eq!(punta(&f, &bare), p);
    assert!(tarea(&f, &bare, "TASK-001").contains("\nestado: \"en-curso\"\n"));
    git(&f, &wt, &["push", "-q", "origin", rama]);
    let (c, out) = tc2(&f, &a, &["liberar", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    assert_eq!(valor(&out, "worktree"), Some("quitado"), "{out}");
    assert!(!wt.exists());
    f.crudo(
        &a,
        &["rev-parse", "--verify", &format!("refs/heads/{rama}")],
    );
    f.crudo(
        &bare,
        &["rev-parse", "--verify", &format!("refs/heads/{rama}")],
    );
}

/// C11: liberar una terminada, descartada o en revision da su rechazo por
/// estado; repetir sobre una ya libre, `sin-cambios` sin commit.
#[test]
fn c11_rechazos_por_estado_y_repetir() {
    let f = Fixture::nueva("liberar-c11");
    let (bare, a) = preparar(&f, &["Uno", "Dos", "Tres", "Cuatro"], None);
    // Antes de `editar`, que publica sin regenerar el ROADMAP (la siguiente
    // transaccion lo regeneraria y no seria `sin-cambios`).
    let p = punta(&f, &bare);
    for _ in 0..2 {
        let (c, out) = tc2(&f, &a, &["liberar", "TASK-004"]);
        assert_eq!(c, Some(0), "{out}");
        assert!(out.starts_with("liberar sin-cambios id=TASK-004"), "{out}");
        assert_eq!(valor(&out, "asignado-anterior"), None, "{out}");
    }
    assert_eq!(punta(&f, &bare), p);
    editar(
        &f,
        &a,
        "TASK-001",
        "estado: \"planificada\"",
        "estado: \"terminada\"",
    );
    let (c, out) = tc2(&f, &a, &["descartar", "TASK-002", "no"]);
    assert_eq!(c, Some(0), "{out}");
    let (c, out) = tc2(&f, &a, &["start", "TASK-003"]);
    assert_eq!(c, Some(0), "{out}");
    editar(
        &f,
        &a,
        "TASK-003",
        "estado: \"en-curso\"",
        "estado: \"en-revision\"",
    );
    let p = punta(&f, &bare);
    for (id, motivo) in [
        ("TASK-001", "terminada"),
        ("TASK-002", "descartada"),
        ("TASK-003", "estado-no-valido"),
    ] {
        let (c, out) = tc2(&f, &a, &["liberar", id]);
        assert_eq!((c, valor(&out, "motivo")), (Some(3), Some(motivo)), "{out}");
    }
    assert_eq!(punta(&f, &bare), p);
}

/// C12: un `asignado_a` heredado de la 0.16 se reasigna (lo decide una
/// persona, sin esperar N dias) y queda como objeto `{id, login}`; el
/// agente da `requiere-persona`.
#[test]
fn c12_heredado_queda_como_objeto() {
    let f = Fixture::nueva("liberar-c12");
    let (bare, a) = preparar(&f, &["Uno"], None);
    editar(
        &f,
        &a,
        "TASK-001",
        "asignado_a: null",
        "asignado_a: \"ana@x.com\"",
    );
    let (c, out) = tc2(&f, &a, &["reasignar", "TASK-001", "beto"]);
    assert_eq!(c, Some(3), "{out}");
    assert_eq!(valor(&out, "motivo"), Some("requiere-persona"), "{out}");
    let antes = tarea(&f, &bare, "TASK-001");
    let out = como_persona(&f, &a, cuenta(1002, "beto"), "TASK-001", Some("beto"));
    assert!(out.starts_with("reasignar ok id=TASK-001 "), "{out}");
    assert_eq!(valor(&out, "asignado-anterior"), Some("ana@x.com"), "{out}");
    let despues = tarea(&f, &bare, "TASK-001");
    let esperado = con_fila(
        &antes.replacen(
            "asignado_a: \"ana@x.com\"",
            "asignado_a: {id: 1002, login: \"beto\"}",
            1,
        ),
        &despues,
        "reasignar",
        "persona",
    );
    assert_eq!(despues, esperado);
}

/// Uso y `--json`: sin ID, ID mal formado, sin login o con argumentos de
/// mas es 2 sin tocar nada; `--json` da el mismo contenido en una linea.
#[test]
fn uso_y_json() {
    let f = Fixture::nueva("liberar-uso");
    let (bare, a) = preparar(&f, &["Uno"], None);
    let antes = f.huella(&bare, &a);
    for (args, linea) in [
        (vec!["liberar"], "liberar uso motivo=sin-id"),
        (vec!["liberar", "TASK-1"], "liberar uso motivo=id-invalido"),
        (
            vec!["liberar", "TASK-001", "x"],
            "liberar uso motivo=argumentos-de-mas",
        ),
        (vec!["reasignar"], "reasignar uso motivo=sin-id"),
        (
            vec!["reasignar", "TASK-001"],
            "reasignar uso motivo=sin-login",
        ),
        (
            vec!["reasignar", "TASK-01", "beto"],
            "reasignar uso motivo=id-invalido",
        ),
        (
            vec!["reasignar", "TASK-001", "beto", "x"],
            "reasignar uso motivo=argumentos-de-mas",
        ),
    ] {
        let (c, out) = tc2(&f, &a, &args);
        assert_eq!((c, out.as_str()), (Some(2), linea));
    }
    assert_eq!(f.huella(&bare, &a), antes);
    let (c, out) = tc2(&f, &a, &["reasignar", "--json", "TASK-001", "beto"]);
    assert_eq!(c, Some(0), "{out}");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["comando"], "reasignar");
    assert_eq!(v["claves"]["id"], "TASK-001");
    assert_eq!(v["claves"]["asignado"], "beto");
    assert_eq!(v["claves"]["worktree"], "ninguno");
}
