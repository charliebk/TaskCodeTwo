//! F3-5 (TASK-032): `tc2 board` y `tc2 status` sobre clones con remoto bare.
//! C1 (dos clones dan lo mismo byte a byte), C2 (no escriben nada) y C3c
//! (sin plataforma o sin red, `board` sale 0 con el estado entero; `status`
//! no construye adaptador), y la plataforma de produccion contra el mock.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::cell::Cell;
use std::path::{Path, PathBuf};

use comun::http_mock::{Mock, Simulada, serie};
use comun::{Fixture, escribir};
use tc2::contrato::{Informe, Resultado};
use tc2::estado;
use tc2::i18n::Idioma;
use tc2::plataforma::{
    Firma, ModeloPuertas, Paso, Personas, Plataforma, PrAbierto, ReglasLeidas, Rol, Usuario,
};
use tc2::tablero::{self, Conectada, Consulta, Entorno};
use tc2::tarea::{Asignado, Cuerpo, Tarea};

const RAMA: &str = estado::RAMA;
const SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PERSONAS: &str = "personas:\n  - {id: 7, login: \"ana\", correos: [\"ana@x.com\"]}\n";

/// (codigo, stdout, stderr) de `tc2 <args>` en `dir`, sin tokens.
fn tc2(
    f: &Fixture,
    dir: &Path,
    idioma: &str,
    args: &[&str],
    env: &[(&str, &str)],
) -> (i32, String, String) {
    let o = comun::orden_tc2()
        .current_dir(dir)
        .args(args)
        .envs(f.entorno())
        .env("TC2_IDIOMA", idioma)
        .envs(env.iter().copied())
        .output()
        .expect("lanzar tc2");
    (
        o.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&o.stdout).into_owned(),
        String::from_utf8_lossy(&o.stderr).into_owned(),
    )
}

fn tarea(id: &str, estado: &str, rama: Option<&str>, quien: Option<&str>) -> String {
    let mut t = Tarea::nueva(id, &format!("Titulo de {id}"), "2026-01-01");
    t.sprint = 1;
    t.estado = estado.to_owned();
    t.rama = rama.map(str::to_owned);
    if let Some(l) = quien {
        t.asignado_a = Asignado::persona(7, l);
    }
    t.escribir(&Cuerpo::leer(""))
}

/// Remoto con personas en `develop` y la rama de estado publicada desde un
/// repo aparte; devuelve (bare, clon, commit de la rama de estado).
fn con_estado(f: &Fixture) -> (PathBuf, PathBuf, String) {
    let (bare, clon) = f.remoto_y_clon();
    escribir(&clon, ".taskcode/personas.yml", PERSONAS);
    f.crudo(&clon, &["add", ".taskcode/personas.yml"]);
    f.crudo(&clon, &["commit", "-q", "-m", "personas"]);
    f.crudo(&clon, &["push", "-q", "origin", "develop"]);
    let dir = f.raiz.join("estado");
    f.crudo(&f.raiz, &["init", "-q", "-b", RAMA, "estado"]);
    for (id, md) in [
        ("TASK-001", tarea("TASK-001", "terminada", None, None)),
        (
            "TASK-002",
            tarea(
                "TASK-002",
                "en-curso",
                Some("feature/task-002-b"),
                Some("ana"),
            ),
        ),
        (
            "TASK-003",
            tarea(
                "TASK-003",
                "en-revision",
                Some("feature/task-003-c"),
                Some("ana"),
            ),
        ),
        (
            "TASK-004",
            tarea("TASK-004", "en-revision", Some("feature/task-004-d"), None),
        ),
        ("TASK-005", tarea("TASK-005", "planificada", None, None)),
        (
            "TASK-006",
            tarea("TASK-006", "en-diseno", Some("feature/task-006-f"), None),
        ),
    ] {
        escribir(&dir, &format!("{id}/tarea.md"), &md);
    }
    f.crudo(&dir, &["add", "-A"]);
    f.crudo(&dir, &["commit", "-q", "-m", "estado"]);
    f.crudo(&dir, &["push", "-q", bare.to_str().unwrap(), RAMA]);
    let commit = f.crudo(&dir, &["rev-parse", "HEAD"]).trim().to_owned();
    f.crudo(&clon, &["fetch", "-q", "origin"]);
    (bare, clon, commit)
}

/// Plataforma falsa: solo `prs-abiertos`, con la respuesta dada.
struct Falsa(Paso<Vec<PrAbierto>>);

impl Plataforma for Falsa {
    fn prs_abiertos(&self, _: Rol) -> Paso<Vec<PrAbierto>> {
        self.0.clone()
    }
    fn abrir_pr(&self, _: &str, _: &str, _: &str, _: &str) -> Informe {
        panic!("board no abre PR")
    }
    fn estado_puertas(&self, _: Rol, _: u64, _: &ModeloPuertas) -> Informe {
        panic!("board no lee puertas")
    }
    fn merge_pr(&self, _: u64, _: &str) -> Informe {
        panic!("board no mergea")
    }
    fn leer_puertas(&self, _: Rol, _: &str) -> Paso<ReglasLeidas> {
        panic!("board no lee puertas")
    }
    fn leer_permisos_ci(&self) -> Informe {
        panic!("board no lee permisos")
    }
    fn firma_commit(&self, _: &str) -> Paso<Firma> {
        panic!("board no lee firmas")
    }
    fn identidad(&self, _: Rol) -> Paso<Usuario> {
        panic!("board no pide identidad")
    }
}

fn prs() -> Vec<PrAbierto> {
    vec![
        PrAbierto {
            numero: 12,
            rama: "feature/task-003-c".into(),
            sha: SHA.into(),
            borrador: false,
        },
        PrAbierto {
            numero: 14,
            rama: "feature/task-002-b".into(),
            sha: SHA.into(),
            borrador: false,
        },
    ]
}

/// `s` con cada hora UTC `AAAA-MM-DDTHH:MM:SSZ` cambiada por `HORA`: la de
/// la ultima sincronizacion (F3-5b) es de cada clon.
fn sin_hora(s: &str) -> String {
    let b = s.as_bytes();
    let es_hora = |v: &[u8]| {
        v.len() == 20
            && v.iter().enumerate().all(|(i, c)| match i {
                4 | 7 => *c == b'-',
                10 => *c == b'T',
                13 | 16 => *c == b':',
                19 => *c == b'Z',
                _ => c.is_ascii_digit(),
            })
    };
    let mut o = String::new();
    let mut i = 0;
    while i < b.len() {
        if i + 20 <= b.len() && es_hora(&b[i..i + 20]) {
            o.push_str("HORA");
            i += 20;
        } else {
            let c = s[i..].chars().next().unwrap();
            o.push(c);
            i += c.len_utf8();
        }
    }
    o
}

/// C1: dos clones con el mismo `commit=` y la misma respuesta (falsa) de la
/// plataforma dan `board --json` y `status --json` iguales byte a byte salvo
/// la hora de su ultima sincronizacion (F3-5b), aunque uno tenga cache de
/// identidad, `tareas/` montada con cambios sin publicar y otro idioma. El
/// `commit=` es el de `origin/taskcodetwo/tareas` y el mismo que dan `next`
/// y `roadmap`.
#[test]
fn dos_clones_dan_lo_mismo() {
    let f = Fixture::nueva("tablero-c1");
    let (bare, a, commit) = con_estado(&f);
    let b = f.otro_clon(&bare, "b");
    f.crudo(&b, &["fetch", "-q", "origin"]);
    // En A: cache de identidad, tareas/ montada y un tarea.md cambiado sin
    // publicar.
    escribir(
        &a,
        ".git/taskcodetwo/identidad.yml",
        "proveedor: github\nid: 7\nlogin: ana\nguardada: 1800000000\n",
    );
    assert!(estado::montar(&f.git(), &a).resultado.es_exito());
    escribir(
        &a.join("tareas"),
        "TASK-005/tarea.md",
        &tarea("TASK-005", "en-curso", Some("feature/x"), Some("ana")),
    );
    let ga = f.git();
    let gb = f.git();
    let conectar = |_: &Personas| {
        Ok(Conectada {
            plataforma: Box::new(Falsa(Ok(prs()))),
            rol: Rol::Agente,
        })
    };
    let ea = Entorno {
        dir: &a,
        git: &ga,
        conectar: &conectar,
    };
    let eb = Entorno {
        dir: &b,
        git: &gb,
        conectar: &conectar,
    };
    let board_a = tablero::orden_board(&[], true, Idioma::Es, &ea);
    let board_b = tablero::orden_board(&[], true, Idioma::En, &eb);
    assert_eq!(board_a.codigo, 0);
    assert_eq!(sin_hora(&board_a.stdout), sin_hora(&board_b.stdout));
    let status_a = tablero::orden_status(&[], true, Idioma::Es, &ea);
    let status_b = tablero::orden_status(&[], true, Idioma::En, &eb);
    assert_eq!(status_a.codigo, 0);
    assert_eq!(sin_hora(&status_a.stdout), sin_hora(&status_b.stdout));
    // El idioma solo cambia stderr (los avisos).
    assert!(
        board_a.stderr.starts_with("aviso: en revision sin PR"),
        "{}",
        board_a.stderr
    );
    assert!(
        board_b.stderr.starts_with("warning: in review"),
        "{}",
        board_b.stderr
    );
    let v: serde_json::Value = serde_json::from_str(&board_a.stdout).unwrap();
    let k = &v["claves"];
    assert_eq!(k["commit"], &commit[..12]);
    assert_eq!(k["plataforma"], "ok");
    assert_eq!(k["en-curso"], serde_json::json!(["TASK-002"]));
    assert_eq!(
        k["en-revision"],
        serde_json::json!(["TASK-003", "TASK-004"])
    );
    assert_eq!(k["listas"], serde_json::json!(["TASK-005"]));
    assert_eq!(k["prs"], serde_json::json!(["TASK-002#14", "TASK-003#12"]));
    assert_eq!(k["revision-sin-pr"], serde_json::json!(["TASK-004"]));
    assert_eq!(k["pr-sin-revision"], serde_json::json!(["TASK-002"]));
    assert_eq!(
        k["quien"],
        serde_json::json!(["TASK-002:@ana", "TASK-003:@ana"])
    );
    // Con el binario: `board --json --sin-plataforma` y `status --json` iguales
    // en los dos clones y los dos idiomas; `commit=` igual al de next y
    // roadmap.
    for args in [
        vec!["board", "--json", "--sin-plataforma"],
        vec!["status", "--json"],
    ] {
        let (ca, oa, _) = tc2(&f, &a, "es", &args, &[]);
        let (cb, ob, _) = tc2(&f, &b, "en", &args, &[]);
        assert_eq!((ca, cb), (0, 0), "{args:?}: {oa}");
        assert_eq!(sin_hora(&oa), sin_hora(&ob), "{args:?}");
        assert!(
            oa.contains(&format!("\"commit\":\"{}\"", &commit[..12])),
            "{oa}"
        );
    }
    for args in [vec!["next", "--json"], vec!["roadmap", "--json"]] {
        let (c, o, _) = tc2(&f, &a, "es", &args, &[]);
        assert_eq!(c, 0, "{args:?}: {o}");
        assert!(
            o.contains(&format!("\"commit\":\"{}\"", &commit[..12])),
            "{o}"
        );
    }
}

/// Lo que no puede cambiar: status (con ignorados) del clon y de `tareas/`,
/// refs, arbol de `develop` y `.git/taskcodetwo/` (rutas y contenido) salvo
/// la marca de sincronizacion, que F3-5b renueva en cada lectura con red (y
/// la comprueba `tests/frescura.rs`).
fn foto(f: &Fixture, clon: &Path) -> String {
    let mut s = f.crudo(clon, &["status", "--porcelain", "--ignored"]);
    s += &f.crudo(
        &clon.join("tareas"),
        &["status", "--porcelain", "--ignored"],
    );
    s += &f.crudo(clon, &["for-each-ref", "--format=%(refname) %(objectname)"]);
    s += &f.crudo(clon, &["rev-parse", "develop^{tree}"]);
    let mut pila = vec![clon.join(".git").join("taskcodetwo")];
    let mut ficheros = Vec::new();
    while let Some(d) = pila.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                pila.push(p);
            } else if p.file_name().and_then(|n| n.to_str())
                != Some(tc2::huella::FICHERO_SINCRONIZACION)
            {
                ficheros.push(p);
            }
        }
    }
    ficheros.sort();
    for p in ficheros {
        s += &format!("{}:{:?}\n", p.display(), std::fs::read(&p).unwrap());
    }
    s
}

/// C2: tras `board` y `status` (texto y JSON, con y sin plataforma) no cambia
/// nada; `board --escribir` es uso (2) y no existe `docs/BOARD.md`.
#[test]
fn board_y_status_no_escriben_nada() {
    let f = Fixture::nueva("tablero-c2");
    let (_bare, clon, _) = con_estado(&f);
    assert!(estado::montar(&f.git(), &clon).resultado.es_exito());
    escribir(&clon, "ignorado.log", "x\n");
    escribir(&clon, ".gitignore", "*.log\n");
    escribir(&clon.join("tareas"), "TASK-005/notas.txt", "sin publicar\n");
    escribir(&clon, ".git/taskcodetwo/identidad.yml", "x: 1\n");
    let antes = foto(&f, &clon);
    for args in [
        vec!["status"],
        vec!["status", "--json"],
        vec!["board"],
        vec!["board", "--json"],
        vec!["board", "--sin-plataforma"],
        vec!["board", "--json", "--sin-plataforma"],
    ] {
        let (c, out, err) = tc2(&f, &clon, "es", &args, &[]);
        assert_eq!(c, 0, "{args:?}: {out} {err}");
        assert_eq!(foto(&f, &clon), antes, "{args:?} cambio algo");
    }
    let (c, out, err) = tc2(&f, &clon, "es", &["board", "--escribir"], &[]);
    assert_eq!(
        (c, out.as_str(), err.as_str()),
        (
            2,
            "board uso motivo=argumentos\n",
            "uso: board [--sin-plataforma] [--json]\n"
        )
    );
    let (c, out, _) = tc2(&f, &clon, "en", &["status", "x"], &[]);
    assert_eq!(
        (c, out.as_str()),
        (2, "status uso motivo=argumentos-de-mas\n")
    );
    assert_eq!(foto(&f, &clon), antes);
    assert!(!clon.join("docs").join("BOARD.md").exists());
    assert!(!clon.join("tareas").join("docs").join("BOARD.md").exists());
    // Sin rama de estado, como `next`.
    let otra = Fixture::nueva("tablero-c2-sin");
    let (_, sin) = otra.remoto_y_clon();
    for orden in ["board", "status"] {
        let (c, out, _) = tc2(&otra, &sin, "es", &[orden], &[]);
        assert_eq!(
            (c, out),
            (3, format!("{orden} rechazada motivo=sin-rama-de-estado\n"))
        );
    }
}

/// C3c: `status` no llama nunca a `conectar` (no construye adaptador);
/// `board` si, una vez, salvo con `--sin-plataforma`. Con la plataforma
/// fallando, `board` sale 0 con el estado entero, `plataforma=` y sin
/// pendientes.
#[test]
fn status_no_construye_adaptador_y_board_aguanta_los_fallos() {
    let f = Fixture::nueva("tablero-c3c");
    let (_bare, clon, _) = con_estado(&f);
    let g = f.git();
    let llamadas = Cell::new(0);
    let respuesta: Cell<Option<Result<Paso<Vec<PrAbierto>>, Consulta>>> = Cell::new(None);
    let conectar = |_: &Personas| {
        llamadas.set(llamadas.get() + 1);
        match respuesta.take().expect("respuesta preparada") {
            Ok(r) => Ok(Conectada {
                plataforma: Box::new(Falsa(r)),
                rol: Rol::Agente,
            }),
            Err(c) => Err(c),
        }
    };
    let e = Entorno {
        dir: &clon,
        git: &g,
        conectar: &conectar,
    };
    for json in [false, true] {
        let s = tablero::orden_status(&[], json, Idioma::Es, &e);
        assert_eq!((s.codigo, s.stderr.as_str()), (0, ""));
    }
    let s = tablero::orden_status(&[], false, Idioma::Es, &e);
    assert_eq!(
        s.stdout.split(" · commit ").next(),
        Some(
            "en diseño: TASK-006 · en curso: TASK-002 @ana · en revisión: TASK-003 @ana, TASK-004 · listas para empezar: TASK-005"
        )
    );
    assert_eq!(llamadas.get(), 0, "status construyo un adaptador");
    let s = tablero::orden_board(&["--sin-plataforma".into()], true, Idioma::Es, &e);
    assert_eq!(s.codigo, 0);
    assert_eq!(llamadas.get(), 0, "--sin-plataforma construyo un adaptador");
    let estado_entero = "\"en-diseno\":[\"TASK-006\"],\"en-curso\":[\"TASK-002\"],\"en-revision\":[\"TASK-003\",\"TASK-004\"],\"listas\":[\"TASK-005\"]";
    for (r, nombre) in [
        (Err(Consulta::NoConfigurada), "no-configurada"),
        (Err(Consulta::SinCredenciales), "sin-credenciales"),
        (
            Ok(Err(Informe::nuevo("prs-abiertos", Resultado::Sinred))),
            "sin-red",
        ),
        (
            Ok(Err(Informe::nuevo("prs-abiertos", Resultado::Error)
                .con("motivo", "demasiadas-paginas"))),
            "error",
        ),
    ] {
        let antes = llamadas.get();
        respuesta.set(Some(r));
        let s = tablero::orden_board(&[], true, Idioma::Es, &e);
        assert_eq!(llamadas.get(), antes + 1);
        assert_eq!(s.codigo, 0, "{nombre}: {}", s.stdout);
        assert!(
            s.stdout.contains(&format!("\"plataforma\":\"{nombre}\"")),
            "{}",
            s.stdout
        );
        assert!(s.stdout.contains(estado_entero), "{nombre}: {}", s.stdout);
        for clave in ["\"prs\"", "revision-sin-pr", "pr-sin-revision"] {
            assert!(!s.stdout.contains(clave), "{nombre}: {}", s.stdout);
        }
        assert_eq!(s.stderr.lines().count(), 1, "{nombre}: {}", s.stderr);
    }
}

/// La plataforma de produccion de `board` (deteccion por `origin` y la
/// clave `plataforma` de la base, token de agente) contra el mock; y sin
/// red (nadie escucha), `plataforma=sin-red` con codigo 0. Sin token,
/// `sin-credenciales` sin llamar. `status` no llama nunca.
#[test]
fn board_con_la_plataforma_de_produccion() {
    let _serie = serie();
    let f = Fixture::nueva("tablero-e2e");
    let (bare, clon, _) = con_estado(&f);
    let m = Mock::nuevo();
    let pr = |n: u64, rama: &str, draft: bool| {
        format!(
            r#"{{"number":{n},"draft":{draft},"head":{{"ref":"{rama}","sha":"{SHA}","repo":{{"id":1}}}},"base":{{"ref":"develop","sha":"{SHA}","repo":{{"id":1}}}}}}"#
        )
    };
    m.responder(
        "GET",
        "/repos/o/r/pulls",
        Simulada::json(
            200,
            &format!(
                "[{},{}]",
                pr(12, "feature/task-003-c", false),
                pr(15, "feature/task-004-d", true)
            ),
        ),
    );
    let config = |api: &str| {
        escribir(
            &clon,
            ".taskcode/config.yml",
            &format!("---\nformato: 2\nplataforma:\n  proveedor: \"github\"\n  api: \"{api}\"\n"),
        );
        f.crudo(&clon, &["add", ".taskcode/config.yml"]);
        f.crudo(&clon, &["commit", "-q", "-m", "config"]);
        f.crudo(&clon, &["push", "-q", "origin", "develop"]);
        f.crudo(&clon, &["fetch", "-q", "origin"]);
    };
    config(&m.base());
    let url = f.crudo(&clon, &["remote", "get-url", "origin"]);
    f.crudo(
        &clon,
        &[
            "remote",
            "set-url",
            "origin",
            "ssh://git@git.ejemplo.test/o/r.git",
        ],
    );
    // `origin` va por ssh a un host inventado (para detectar la plataforma):
    // el «ssh» sirve el remoto bare, como en `tests/toma.rs`, para que el
    // `fetch` de F3-5b no salga sin red.
    let ssh = format!(
        "sh -c 'exec git upload-pack \"$0\"' '{}'",
        bare.to_string_lossy().replace('\\', "/")
    );
    let con_ssh = [
        ("GIT_SSH_COMMAND", ssh.as_str()),
        ("GIT_SSH_VARIANT", "simple"),
    ];
    let token = [
        ("TC2_TOKEN_AGENTE", "token-de-prueba"),
        con_ssh[0],
        con_ssh[1],
    ];
    // H5 de la revision 1 de TASK-079: una vez (1 de 4, con la maquina
    // cargada) salio `plataforma` distinto de `ok` aqui sin que se viera cual.
    // No se reprodujo (18 pasadas del binario, 6 a la vez). Si vuelve, el
    // mensaje dice el valor, el paso, lo que tardo (una parada del antivirus
    // de TASK-115 da `sin-red` pasados los 10 s de conexion), el aviso de
    // stderr y lo que recibio el mock.
    let t0 = std::time::Instant::now();
    let (c, out, err) = tc2(&f, &clon, "es", &["board", "--json"], &token);
    let tardo = t0.elapsed();
    assert_eq!(c, 0, "{out} {err}");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let k = &v["claves"];
    assert_eq!(
        k["plataforma"],
        "ok",
        "paso 1 (board --json con token) tardo {tardo:?}; stderr: {err}; recibidas: {}; stdout: {out}",
        m.recibidas().len()
    );
    assert_eq!(k["prs"], serde_json::json!(["TASK-003#12", "TASK-004#15"]));
    assert_eq!(k["borradores"], serde_json::json!(["TASK-004"]));
    assert_eq!(k["revision-sin-pr"], serde_json::json!(["TASK-004"]));
    let recibidas = m.recibidas();
    assert_eq!(recibidas.len(), 1);
    assert_eq!(
        recibidas[0].linea,
        "GET /repos/o/r/pulls?state=open&per_page=100"
    );
    assert_eq!(
        recibidas[0].autorizacion.as_deref(),
        Some("Bearer token-de-prueba")
    );
    // Texto: la columna de PR y los avisos en stderr con codigo 0.
    let (c, out, err) = tc2(&f, &clon, "es", &["board"], &token);
    assert_eq!(c, 0);
    assert!(
        out.contains("  TASK-004  Titulo de TASK-004  —     #15 (borrador)\n"),
        "{out}"
    );
    assert_eq!(err, "aviso: en revision sin PR abierto listo: TASK-004\n");
    // `status` no llama; sin token, `sin-credenciales` sin llamar.
    let (c, _, _) = tc2(&f, &clon, "es", &["status"], &token);
    assert_eq!(c, 0);
    let (c, out, err) = tc2(&f, &clon, "en", &["board", "--json"], &con_ssh);
    assert_eq!(c, 0);
    assert!(out.contains("\"plataforma\":\"sin-credenciales\""), "{out}");
    assert!(
        err.starts_with("warning: pull requests were not read"),
        "{err}"
    );
    // Las dos de `board` con token (JSON y texto); ninguna mas.
    assert_eq!(m.recibidas().len(), 2, "status o sin token llamaron");
    // Sin red: nadie escucha en el puerto 1.
    f.crudo(&clon, &["remote", "set-url", "origin", url.trim()]);
    config("http://127.0.0.1:1");
    f.crudo(
        &clon,
        &[
            "remote",
            "set-url",
            "origin",
            "ssh://git@git.ejemplo.test/o/r.git",
        ],
    );
    let (c, out, err) = tc2(&f, &clon, "es", &["board", "--json"], &token);
    assert_eq!(c, 0, "{out} {err}");
    assert!(out.contains("\"plataforma\":\"sin-red\""), "{out}");
    assert!(
        out.contains("\"en-revision\":[\"TASK-003\",\"TASK-004\"]"),
        "{out}"
    );
    assert!(
        err.starts_with("aviso: no se consultaron los PR: sin red"),
        "{err}"
    );
}

/// TASK-081 C9 y C10: con filas de transicion de 2020, la tarea en curso
/// cuenta en `inactivas=1` y `board` marca su fila; la en revision con un
/// commit reciente en `origin/<rama>` no, aunque su ultima fila sea igual
/// de antigua. En un clon con el worktree de una tarea que ahora lleva otra
/// persona, `status` y `board` avisan `ya-no-tuyas=` con quien la lleva,
/// codigo 0; sin cache de identidad, no.
#[test]
fn c9_c10_inactivas_y_ya_no_tuyas() {
    let f = Fixture::nueva("tablero-f3-10");
    let (bare, clon) = f.remoto_y_clon();
    escribir(&clon, ".taskcode/personas.yml", PERSONAS);
    f.crudo(&clon, &["add", ".taskcode/personas.yml"]);
    f.crudo(&clon, &["commit", "-q", "-m", "personas"]);
    f.crudo(&clon, &["push", "-q", "origin", "develop"]);
    let vieja = "## Transiciones\n\n| fecha | fase | modo | decidido_por | version |\n|---|---|---|---|---|\n| 2020-01-01T00:00:00Z | start | manual | persona | 0.1 |\n";
    let md = |id: &str, estado: &str, rama: &str, quien: Option<Asignado>| {
        let mut t = Tarea::nueva(id, &format!("Titulo de {id}"), "2026-01-01");
        t.estado = estado.to_owned();
        t.rama = Some(rama.to_owned());
        t.asignado_a = quien;
        t.escribir(&Cuerpo::leer(vieja))
    };
    let dir = f.raiz.join("estado");
    f.crudo(&f.raiz, &["init", "-q", "-b", RAMA, "estado"]);
    for (id, texto) in [
        (
            "TASK-002",
            md(
                "TASK-002",
                "en-curso",
                "feature/task-002-b",
                Asignado::persona(8, "beto"),
            ),
        ),
        (
            "TASK-003",
            md(
                "TASK-003",
                "en-revision",
                "feature/task-003-c",
                Asignado::persona(7, "ana"),
            ),
        ),
    ] {
        escribir(&dir, &format!("{id}/tarea.md"), &texto);
    }
    f.crudo(&dir, &["add", "-A"]);
    f.crudo(&dir, &["commit", "-q", "-m", "estado"]);
    f.crudo(&dir, &["push", "-q", bare.to_str().unwrap(), RAMA]);
    // La rama de TASK-003 con un commit de ahora; la de TASK-002 no esta.
    f.crudo(
        &clon,
        &[
            "push",
            "-q",
            "origin",
            "develop:refs/heads/feature/task-003-c",
        ],
    );
    f.crudo(&clon, &["fetch", "-q", "origin"]);
    let (c, out, err) = tc2(&f, &clon, "es", &["status", "--json"], &[]);
    assert_eq!((c, err.as_str()), (0, ""), "{out}");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["claves"]["inactivas"], "1", "{out}");
    assert!(v["claves"].get("ya-no-tuyas").is_none(), "{out}");
    let (_, texto, _) = tc2(&f, &clon, "es", &["status"], &[]);
    assert!(texto.trim_end().ends_with(" · sin actividad 1"), "{texto}");
    let (c, texto, _) = tc2(&f, &clon, "es", &["board", "--sin-plataforma"], &[]);
    assert_eq!(c, 0);
    let fila = |id: &str| {
        texto
            .lines()
            .find(|l| l.starts_with(&format!("  {id}  ")))
            .unwrap_or_default()
            .to_owned()
    };
    assert!(
        fila("TASK-002").contains("· sin actividad hace ") && fila("TASK-002").ends_with(" días"),
        "{texto}"
    );
    assert!(!fila("TASK-003").contains("sin actividad"), "{texto}");
    // C10: ana tiene el worktree de TASK-002, que ahora lleva beto.
    f.crudo(
        &clon,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "feature/task-002-b",
            ".taskcodetwo/worktrees/TASK-002",
        ],
    );
    let (c, out, err) = tc2(&f, &clon, "es", &["status", "--json"], &[]);
    assert_eq!((c, err.as_str()), (0, ""), "sin cache no se avisa: {out}");
    tc2::identidad::Cache::en(&clon.join(".git"))
        .guardar(
            &tc2::identidad::Cuenta {
                proveedor: tc2::plataforma::Proveedor::GitHub,
                id: 7,
                login: "ana".to_owned(),
            },
            tc2::identidad::ahora(),
        )
        .unwrap();
    for args in [
        vec!["status", "--json"],
        vec!["board", "--json", "--sin-plataforma"],
    ] {
        let (c, out, err) = tc2(&f, &clon, "es", &args, &[]);
        assert_eq!(c, 0, "{args:?}: {out}");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(
            v["claves"]["ya-no-tuyas"],
            serde_json::json!(["TASK-002"]),
            "{out}"
        );
        assert_eq!(
            err,
            "aviso: TASK-002 ya no es tuya: la lleva @beto; tu worktree sigue en .taskcodetwo/worktrees/TASK-002 y la rama no se ha tocado\n",
            "{args:?}"
        );
    }
    let (_, _, en) = tc2(&f, &clon, "en", &["status"], &[]);
    assert!(
        en.starts_with("warning: TASK-002 is no longer yours: @beto has it"),
        "{en}"
    );
}
