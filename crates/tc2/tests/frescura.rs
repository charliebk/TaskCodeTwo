//! F3-5b (TASK-079): `status`, `board`, `next` y `roadmap` traen la rama de
//! estado antes de mostrar y dicen la hora de la ultima sincronizacion.
//!
//! Con red, `origin` es el remoto bare de la fixture (ruta local). Sin red
//! no se simula con una ruta inexistente (eso es `denegado`): `origin` va
//! por ssh y el «ssh» de `GIT_SSH_COMMAND` responde «Connection refused»,
//! como en `tests/toma.rs`.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use comun::{Fixture, escribir, ops::sin_catalogo, remoto::ORIGIN, valor};
use tc2::contrato::Resultado;
use tc2::estado::{self, Cambio, Contexto, Estado, Fallo, Operacion};
use tc2::i18n::Idioma;
use tc2::plataforma::Personas;
use tc2::tablero::{self, Conectada, Consulta, Entorno};

const RAMA: &str = estado::RAMA;

/// Entorno de git cuyo «ssh» llega y el servidor no acepta la clave.
fn entorno_denegado(f: &Fixture) -> Vec<(OsString, OsString)> {
    let mut e = f.entorno();
    e.push((
        "GIT_SSH_COMMAND".into(),
        "sh -c 'echo \"git@github.com: Permission denied (publickey).\" >&2; exit 255'".into(),
    ));
    e.push(("GIT_SSH_VARIANT".into(), "simple".into()));
    e
}

/// (codigo, stdout, stderr) de `tc2 <args>` en `dir` con `entorno`.
fn lanzar(
    dir: &Path,
    entorno: Vec<(OsString, OsString)>,
    idioma: &str,
    args: &[&str],
) -> (i32, String, String) {
    let o = comun::orden_tc2()
        .current_dir(dir)
        .args(args)
        .envs(entorno)
        .env("TC2_IDIOMA", idioma)
        .output()
        .expect("lanzar tc2");
    (
        o.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&o.stdout).into_owned(),
        String::from_utf8_lossy(&o.stderr).into_owned(),
    )
}

/// `tc2 <args>` con el remoto de la fixture (ruta local o ssh servido).
fn tc2(f: &Fixture, dir: &Path, args: &[&str]) -> (i32, String, String) {
    lanzar(dir, f.entorno_ssh(), "es", args)
}

/// Las claves del JSON de una orden.
fn claves(json: &str) -> serde_json::Value {
    let v: serde_json::Value = serde_json::from_str(json).unwrap_or_else(|e| panic!("{e}: {json}"));
    v["claves"].clone()
}

/// Si `h` es una hora `AAAA-MM-DDTHH:MM:SSZ`.
fn es_hora(h: &str) -> bool {
    let b = h.as_bytes();
    b.len() == 20
        && b.iter().enumerate().all(|(i, c)| match i {
            4 | 7 => *c == b'-',
            10 => *c == b'T',
            13 | 16 => *c == b':',
            19 => *c == b'Z',
            _ => c.is_ascii_digit(),
        })
}

/// `tc2 new <titulo>` con exito; devuelve su linea.
fn nueva(f: &Fixture, dir: &Path, titulo: &str) -> String {
    let (c, out, err) = tc2(f, dir, &["new", titulo]);
    assert_eq!(c, 0, "{out} {err}");
    out
}

/// Remoto bare con una tarea publicada por `tc2 new` desde el clon `a`, y
/// `origin` de `a` por ssh (GitHub por el host).
fn con_tarea(f: &Fixture) -> (PathBuf, PathBuf) {
    let (bare, a) = f.remoto_y_clon();
    nueva(f, &a, "Primera");
    f.crudo(&a, &["remote", "set-url", "origin", ORIGIN]);
    (bare, a)
}

fn sin_plataforma(_: &Personas) -> Result<Conectada, Consulta> {
    panic!("no se consulta la plataforma")
}

/// C1: A publica con `tc2 new` y B, sin nada antes, ve la tarea nueva con el
/// `commit=` que A publico en `status`, `next`, `board` y `roadmap` (JSON).
#[test]
fn c1_b_ve_lo_que_publica_a_sin_fetch_previo() {
    let f = Fixture::nueva("frescura-c1");
    let (bare, a) = con_tarea(&f);
    let b = f.clon_github(&bare, "b");
    let linea = nueva(&f, &a, "Segunda");
    assert!(linea.starts_with("new ok id=TASK-002 "), "{linea}");
    let commit = valor(&linea, "commit").unwrap().to_owned();
    assert_eq!(commit.len(), 12, "{linea}");
    // B no ha traido nada desde que se clono.
    let antes = f.crudo(&b, &["rev-parse", &format!("origin/{RAMA}")]);
    assert!(!antes.starts_with(&commit), "B ya lo tenia");
    for args in [
        vec!["status", "--json"],
        vec!["next", "--json"],
        vec!["board", "--sin-plataforma", "--json"],
        vec!["roadmap", "--json"],
    ] {
        let (c, out, err) = tc2(&f, &b, &args);
        assert_eq!((c, err.as_str()), (0, ""), "{args:?}: {out}");
        let k = claves(&out);
        assert_eq!(k["commit"], commit.as_str(), "{args:?}: {out}");
        assert_eq!(k["remoto"], "consultado", "{args:?}: {out}");
        assert!(out.contains("TASK-002"), "{args:?}: {out}");
    }
}

/// C2: sin red (tras un `status` con red), `status` sale 0 con la ultima foto,
/// `remoto=sin-red`, `sincronizado=` igual a la hora del ultimo `fetch` con
/// exito y un aviso con esa hora y el «hace N»; sin sincronizar nunca,
/// `sincronizado=nunca` con aviso.
#[test]
fn c2_sin_red_la_ultima_foto_con_su_hora() {
    let f = Fixture::nueva("frescura-c2");
    let (bare, a) = con_tarea(&f);
    // La marca la dejo el `new`; se borra para que la que se lee despues sea
    // la que escribe este `status` (H3 de la revision 1).
    let marca = estado::Marca::en(&a.join(".git"));
    std::fs::remove_file(marca.ruta()).unwrap();
    let (c, out, err) = tc2(&f, &a, &["status", "--json"]);
    assert_eq!((c, err.as_str()), (0, ""), "{out}");
    let k = claves(&out);
    let hora = k["sincronizado"].as_str().unwrap().to_owned();
    assert!(es_hora(&hora), "la lectura no recreo la marca: {out}");
    assert_eq!(estado::sincronizado_texto(marca.leer()), hora);
    let commit = k["commit"].as_str().unwrap().to_owned();
    // Lo que se publique despues no se ve sin red.
    let b = f.clon_github(&bare, "b");
    nueva(&f, &b, "Segunda");
    for json in [true, false] {
        let args: &[&str] = if json {
            &["status", "--json"]
        } else {
            &["status"]
        };
        let (c, out, err) = lanzar(&a, f.entorno_sin_red(), "es", args);
        assert_eq!(c, 0, "{out} {err}");
        if json {
            let k = claves(&out);
            assert_eq!(k["remoto"], "sin-red", "{out}");
            assert_eq!(k["sincronizado"], hora.as_str(), "{out}");
            assert_eq!(k["commit"], commit.as_str(), "{out}");
        } else {
            assert!(
                out.trim_end()
                    .ends_with(&format!("commit {commit} · sincronizado {hora}")),
                "{out}"
            );
        }
        assert_eq!(err.lines().count(), 1, "{err}");
        assert!(
            err.starts_with("aviso: sin red, no se pudo traer la rama de estado")
                && err.contains(&format!(" el {hora} (hace ")),
            "{err}"
        );
    }
    let (_, _, en) = lanzar(&a, f.entorno_sin_red(), "en", &["status"]);
    assert!(
        en.starts_with("warning: no network") && en.contains(" ago)"),
        "{en}"
    );
    // Un clon que nunca sincronizo con `tc2`.
    let nuevo = f.clon_github(&bare, "nuevo");
    let (c, out, err) = lanzar(&nuevo, f.entorno_sin_red(), "es", &["status", "--json"]);
    assert_eq!(c, 0, "{out} {err}");
    let k = claves(&out);
    assert_eq!(
        (k["remoto"].as_str(), k["sincronizado"].as_str()),
        (Some("sin-red"), Some("nunca")),
        "{out}"
    );
    assert!(
        err.contains("no la ha sincronizado nunca") && err.starts_with("aviso: sin red"),
        "{err}"
    );
}

/// C3: la hora sale en texto (linea de `status`, cabecera de `board` y, en
/// `roadmap`, stderr) y en el JSON de las cuatro con `sincronizado` y
/// `remoto`.
#[test]
fn c3_la_hora_en_texto_y_en_json() {
    let f = Fixture::nueva("frescura-c3");
    let (_bare, a) = con_tarea(&f);
    let marca = estado::Marca::en(&a.join(".git"));
    std::fs::remove_file(marca.ruta()).unwrap();
    let (_, out, _) = tc2(&f, &a, &["status", "--json"]);
    let hora = claves(&out)["sincronizado"].as_str().unwrap().to_owned();
    assert!(es_hora(&hora), "la lectura no recreo la marca: {out}");
    for orden in ["status", "next", "board", "roadmap"] {
        // Cada orden de lectura recrea la marca por su cuenta (H3).
        std::fs::remove_file(marca.ruta()).unwrap();
        let mut args = vec![orden, "--json"];
        if orden == "board" {
            args.push("--sin-plataforma");
        }
        let (c, out, err) = tc2(&f, &a, &args);
        assert_eq!((c, err.as_str()), (0, ""), "{orden}: {out}");
        let k = claves(&out);
        assert_eq!(k["remoto"], "consultado", "{orden}: {out}");
        assert!(
            k["sincronizado"].as_str().is_some_and(es_hora),
            "{orden}: {out}"
        );
        assert!(marca.leer().is_some(), "{orden} no recreo la marca");
    }
    let (c, out, err) = tc2(&f, &a, &["status"]);
    assert_eq!((c, err.as_str()), (0, ""));
    let linea = out.trim_end();
    let h = linea.rsplit(" · sincronizado ").next().unwrap();
    assert!(es_hora(h) && h >= hora.as_str(), "{out}");
    let (c, out, _) = tc2(&f, &a, &["board", "--sin-plataforma"]);
    assert_eq!(c, 0);
    let cabecera = out.lines().next().unwrap();
    assert!(
        cabecera.starts_with("Tablero · commit ") && cabecera.contains(" · sincronizado "),
        "{cabecera}"
    );
    assert!(es_hora(cabecera.rsplit(' ').next().unwrap()), "{cabecera}");
    let (c, out, _) = tc2(&f, &a, &["next"]);
    assert_eq!(c, 0);
    assert!(
        valor(&out, "remoto") == Some("consultado")
            && valor(&out, "sincronizado").is_some_and(es_hora),
        "{out}"
    );
    let (c, out, err) = tc2(&f, &a, &["roadmap"]);
    assert_eq!(c, 0);
    assert!(!out.contains("sincronizado"), "la hora entro en el ROADMAP");
    assert_eq!(err.lines().count(), 1, "{err}");
    assert!(
        err.starts_with("estado sincronizado con origin el ")
            && err.trim_end().ends_with("(hace menos de 1 min)"),
        "{err}"
    );
    let (_, _, en) = lanzar(&a, f.entorno_ssh(), "en", &["roadmap"]);
    assert!(en.starts_with("state synchronised with origin at "), "{en}");
}

/// Lo que puede cambiar una lectura: todo menos la ref remota de la rama de
/// estado y la marca. Refs del clon y del bare, status del principal y de
/// `tareas/`, `.git/taskcodetwo/` sin la marca y commits del bare.
fn foto(f: &Fixture, bare: &Path, clon: &Path) -> String {
    let remota = format!("refs/remotes/origin/{RAMA} ");
    let mut s: String = f
        .crudo(clon, &["for-each-ref", "--format=%(refname) %(objectname)"])
        .lines()
        .filter(|l| !l.starts_with(&remota))
        .map(|l| format!("{l}\n"))
        .collect();
    s += &f.crudo(bare, &["for-each-ref", "--format=%(refname) %(objectname)"]);
    s += &f.crudo(bare, &["rev-list", "--all", "--count"]);
    s += &f.crudo(clon, &["status", "--porcelain", "--ignored"]);
    s += &f.crudo(
        &clon.join("tareas"),
        &["status", "--porcelain", "--ignored"],
    );
    let propia = clon.join(".git").join("taskcodetwo");
    let mut ficheros: Vec<PathBuf> = std::fs::read_dir(&propia)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name().and_then(|n| n.to_str()) != Some(tc2::huella::FICHERO_SINCRONIZACION)
        })
        .collect();
    ficheros.sort();
    for p in ficheros {
        s += &format!("{}:{:?}\n", p.display(), std::fs::read(&p).ok());
    }
    s
}

/// C4: tras las cuatro ordenes con red no cambian
/// `refs/heads/taskcodetwo/tareas`, el status del principal ni de `tareas/`,
/// el bare no gana commits y el registro del puerto no tiene `push`,
/// `commit-tree` ni `update-ref`; solo cambian
/// `refs/remotes/origin/taskcodetwo/tareas` y la marca.
#[test]
fn c4_las_cuatro_solo_escriben_la_ref_remota_y_la_marca() {
    let f = Fixture::nueva("frescura-c4");
    let (bare, a) = f.remoto_y_clon();
    nueva(&f, &a, "Primera");
    let b = f.otro_clon(&bare, "b");
    assert!(estado::montar(&f.git(), &b).resultado.es_exito());
    escribir(&b.join("tareas"), "TASK-001/notas.txt", "sin publicar\n");
    escribir(&b, "suelto.txt", "x\n");
    // A publica algo que B aun no tiene.
    let linea = nueva(&f, &a, "Segunda");
    let nuevo = valor(&linea, "commit").unwrap().to_owned();
    // `montar` trajo la rama con el mismo `fetch` y dejo la marca (M3 de la
    // revision 1); se borra para que la escriban las lecturas.
    let marca = estado::Marca::en(&b.join(".git"));
    assert!(marca.leer().is_some(), "montar no dejo la marca");
    std::fs::remove_file(marca.ruta()).unwrap();
    let local = f.crudo(&b, &["rev-parse", &format!("refs/heads/{RAMA}")]);
    let antes = foto(&f, &bare, &b);
    let g = f.git();
    let conectar = sin_plataforma;
    let e = Entorno {
        dir: &b,
        git: &g,
        conectar: &conectar,
    };
    let s = tablero::orden_status(&[], true, Idioma::Es, &e);
    assert_eq!(s.codigo, 0, "{}", s.stdout);
    let s = tablero::orden_board(&["--sin-plataforma".into()], true, Idioma::Es, &e);
    assert_eq!(s.codigo, 0, "{}", s.stdout);
    let i = tc2::siguiente::siguiente_en(&g, &b);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    let r = tc2::roadmap::imprimir_en(&g, &b);
    assert!(r.is_ok(), "{r:?}");
    let registro = g.registro();
    comun::registro_permitido(&registro);
    let fetch = registro
        .iter()
        .filter(|l| l.starts_with("git fetch "))
        .count();
    assert_eq!(fetch, 4, "un fetch por orden: {registro:#?}");
    for l in &registro {
        for prohibido in ["git push", "git commit-tree", "git update-ref"] {
            assert!(!l.starts_with(prohibido), "{l}");
        }
    }
    assert_eq!(foto(&f, &bare, &b), antes);
    assert_eq!(
        f.crudo(&b, &["rev-parse", &format!("refs/heads/{RAMA}")]),
        local,
        "la rama local de estado se movio"
    );
    let remota = f.crudo(&b, &["rev-parse", &format!("refs/remotes/origin/{RAMA}")]);
    assert!(remota.starts_with(&nuevo), "{remota} frente a {nuevo}");
    assert!(marca.leer().is_some(), "la marca no se escribio");
}

/// C5: tras un `tc2 new` con exito, `status` sin red da `sincronizado=` igual
/// o posterior a la hora de ese `new` (la transaccion renueva la marca).
#[test]
fn c5_tras_new_la_hora_es_la_de_ese_new() {
    let f = Fixture::nueva("frescura-c5");
    let (_bare, a) = con_tarea(&f);
    // El primer `new` ya dejo la marca; se cambia por una vieja: el `new`
    // siguiente tiene que renovarla.
    assert!(estado::Marca::en(&a.join(".git")).leer().is_some());
    estado::Marca::en(&a.join(".git"))
        .registrar(1_000_000_000)
        .unwrap();
    let antes = estado::sincronizado_texto(Some(tc2::identidad::ahora()));
    nueva(&f, &a, "Segunda");
    let (c, out, err) = lanzar(&a, f.entorno_sin_red(), "es", &["status", "--json"]);
    assert_eq!(c, 0, "{out} {err}");
    let k = claves(&out);
    assert_eq!(k["remoto"], "sin-red", "{out}");
    let h = k["sincronizado"].as_str().unwrap();
    assert!(es_hora(h) && h >= antes.as_str(), "{h} < {antes}");
    assert!(out.contains("TASK-002"), "{out}");
}

/// Escribe un fichero en la rama de estado; no exige red (sin red queda
/// pendiente).
struct Nota;

impl Operacion for Nota {
    fn nombre(&self) -> &str {
        "nota"
    }
    fn anotacion(&self) -> String {
        "nota|TASK-001".to_owned()
    }
    fn exige_red(&self) -> bool {
        false
    }
    fn aplicar(&self, _: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        Ok(vec![Cambio::nuevo("TASK-001/nota.md", "nota\n")])
    }
}

/// C6: con credenciales denegadas sale 0 con `remoto=denegado` y aviso; sin
/// remoto no hace `fetch` y sale `remoto=sin-remoto`.
#[test]
fn c6_denegado_y_sin_remoto() {
    let f = Fixture::nueva("frescura-c6");
    let (_bare, a) = con_tarea(&f);
    for orden in ["status", "next", "roadmap"] {
        let (c, out, err) = lanzar(&a, entorno_denegado(&f), "es", &[orden, "--json"]);
        assert_eq!(c, 0, "{orden}: {out} {err}");
        assert_eq!(claves(&out)["remoto"], "denegado", "{orden}: {out}");
        assert!(
            err.starts_with("aviso: origin no deja leer la rama de estado (credenciales)"),
            "{orden}: {err}"
        );
    }
    let (c, out, err) = lanzar(
        &a,
        entorno_denegado(&f),
        "es",
        &["board", "--sin-plataforma", "--json"],
    );
    assert_eq!(c, 0, "{out} {err}");
    assert_eq!(claves(&out)["remoto"], "denegado", "{out}");
    // Sin remoto: la rama local es el estado.
    let solo = f.raiz.join("solo");
    f.crudo(&f.raiz, &["init", "-q", "-b", "develop", "solo"]);
    escribir(&solo, "README.md", "x\n");
    f.crudo(&solo, &["add", "README.md"]);
    f.crudo(&solo, &["commit", "-q", "-m", "inicial"]);
    let (c, out, err) = lanzar(&solo, f.entorno(), "es", &["new", "Sola"]);
    assert_eq!(c, 0, "{out} {err}");
    let g = f.git();
    let conectar = sin_plataforma;
    let e = Entorno {
        dir: &solo,
        git: &g,
        conectar: &conectar,
    };
    let s = tablero::orden_status(&[], false, Idioma::Es, &e);
    assert_eq!((s.codigo, s.stderr.as_str()), (0, ""), "{}", s.stdout);
    assert!(!s.stdout.contains("sincronizado"), "{}", s.stdout);
    let s = tablero::orden_status(&[], true, Idioma::Es, &e);
    let k = claves(&s.stdout);
    assert_eq!(
        (k["remoto"].as_str(), k["sincronizado"].as_str()),
        (Some("sin-remoto"), Some("nunca")),
        "{}",
        s.stdout
    );
    assert!(s.stdout.contains("TASK-001"), "{}", s.stdout);
    let i = tc2::siguiente::siguiente_en(&g, &solo);
    assert_eq!(valor(&i.linea(), "remoto"), Some("sin-remoto"));
    assert!(
        !g.registro().iter().any(|l| l.starts_with("git fetch")),
        "{:#?}",
        g.registro()
    );
}

/// C7: `status` dice `pendientes=N` con las refs de
/// `refs/taskcodetwo/pendientes/` y avisa `tareas=con-cambios|diverge` con
/// codigo 0.
#[test]
fn c7_pendientes_y_avisos_de_tareas() {
    let f = Fixture::nueva("frescura-c7");
    let (bare, a) = con_tarea(&f);
    let (c, out, _) = tc2(&f, &a, &["status", "--json"]);
    assert_eq!(c, 0);
    assert_eq!(claves(&out)["pendientes"], "0", "{out}");
    assert!(claves(&out).get("tareas").is_none(), "{out}");
    // Una operacion sin red queda pendiente.
    let g = tc2::git::Git::nuevo().con_entorno(f.entorno_sin_red());
    let i = estado::transaccion(&Contexto::nuevo(&g, &a, &sin_catalogo, None), &Nota);
    assert_eq!(i.resultado, Resultado::Sinred, "{}", i.linea());
    assert_eq!(i.valor("pendiente"), Some("si"));
    let (c, out, err) = lanzar(&a, f.entorno_sin_red(), "es", &["status", "--json"]);
    assert_eq!(c, 0, "{out} {err}");
    assert_eq!(claves(&out)["pendientes"], "1", "{out}");
    let refs = f.crudo(&a, &["for-each-ref", "refs/taskcodetwo/pendientes/"]);
    assert_eq!(refs.lines().count(), 1, "{refs}");
    // En texto tambien, al final, solo con alguna (M2 de la revision 1; con
    // cero no sale, lo comprueba C2). `status` solo lee: no la publica.
    let (c, out, _) = lanzar(&a, f.entorno_sin_red(), "es", &["status"]);
    assert_eq!(c, 0, "{out}");
    let linea = out.trim_end();
    assert!(
        linea.ends_with(" · pendientes 1") && linea.contains(" · sincronizado "),
        "{out}"
    );
    let (c, out, _) = tc2(&f, &a, &["status"]);
    assert_eq!(c, 0, "{out}");
    assert!(
        out.trim_end().ends_with(" · pendientes 1"),
        "con red: {out}"
    );

    // con-cambios: `tareas/` montada, atrasada y con cambios sin publicar.
    let b = f.clon_github(&bare, "b");
    let gb = tc2::git::Git::nuevo().con_entorno(f.entorno_ssh());
    assert!(estado::montar(&gb, &b).resultado.es_exito());
    escribir(&b.join("tareas"), "TASK-001/notas.txt", "mio\n");
    let (c, _, _) = tc2(&f, &a, &["new", "Segunda"]);
    assert_eq!(c, 0);
    let (c, out, err) = tc2(&f, &b, &["status", "--json"]);
    assert_eq!(c, 0, "{out} {err}");
    assert_eq!(claves(&out)["tareas"], "con-cambios", "{out}");
    assert!(
        err.starts_with("aviso: tareas/ tiene cambios sin publicar"),
        "{err}"
    );
    let (c, out, err) = tc2(&f, &b, &["status"]);
    assert_eq!(c, 0, "{out}");
    assert!(err.starts_with("aviso: tareas/"), "{err}");

    // diverge: la rama local tiene un commit que no esta en lo publicado.
    let tareas = b.join("tareas");
    f.crudo(&tareas, &["add", "TASK-001/notas.txt"]);
    f.crudo(&tareas, &["commit", "-q", "-m", "a mano"]);
    let (c, out, err) = tc2(&f, &b, &["status", "--json"]);
    assert_eq!(c, 0, "{out} {err}");
    assert_eq!(claves(&out)["tareas"], "diverge", "{out}");
    assert!(
        err.starts_with("aviso: la rama local taskcodetwo/tareas tiene commits"),
        "{err}"
    );
    let (_, _, en) = lanzar(&b, f.entorno_ssh(), "en", &["status"]);
    assert!(en.starts_with("warning: the local branch"), "{en}");
}

/// C8: `doctor` no hace `fetch` (no aparece en su registro) y muestra
/// `sincronizado=`.
#[test]
fn c8_doctor_sin_fetch_y_con_sincronizado() {
    let f = Fixture::nueva("frescura-c8");
    let (_bare, a) = con_tarea(&f);
    let conectar = |_: &Personas| Err(Consulta::NoConfigurada);
    let g = tc2::git::Git::nuevo().con_entorno(f.entorno_ssh());
    // La marca la dejo el `new`: su hora.
    let hora = estado::sincronizado_texto(estado::Marca::en(&a.join(".git")).leer());
    assert!(es_hora(&hora), "{hora}");
    let e = tc2::doctor::Entorno {
        dir: &a,
        git: &g,
        conectar: &conectar,
        claude: None,
    };
    let s = tc2::doctor::orden(&[], false, Idioma::Es, &e);
    let primera = s.stdout.lines().next().unwrap();
    assert_eq!(
        valor(primera, "sincronizado"),
        Some(hora.as_str()),
        "{primera}"
    );
    let registro = g.registro();
    assert!(
        !registro.iter().any(|l| l.starts_with("git fetch")),
        "doctor hizo fetch: {registro:#?}"
    );
    // Sin marca, `nunca`.
    std::fs::remove_file(estado::Marca::en(&a.join(".git")).ruta()).unwrap();
    let s = tc2::doctor::orden(&[], true, Idioma::Es, &e);
    assert_eq!(claves(&s.stdout)["sincronizado"], "nunca", "{}", s.stdout);
}

/// C9: el stdout de `roadmap` es byte a byte el ROADMAP que `--escribir`
/// publicaria.
#[test]
fn c9_el_stdout_de_roadmap_es_lo_que_publicaria_escribir() {
    let f = Fixture::nueva("frescura-c9");
    let (bare, a) = con_tarea(&f);
    // Un ROADMAP.md desfasado publicado a mano (sin `tc2`), con su zona.
    let dir = f.raiz.join("a-mano");
    f.crudo(
        &f.raiz,
        &["clone", "-q", "-b", RAMA, bare.to_str().unwrap(), "a-mano"],
    );
    escribir(
        &dir,
        "ROADMAP.md",
        "# Viejo\n\n<!-- manual:inicio -->\n### H1 — Uno\n<!-- manual:fin -->\n",
    );
    f.crudo(&dir, &["commit", "-q", "-am", "roadmap viejo"]);
    f.crudo(&dir, &["push", "-q", "origin", RAMA]);
    let (c, impreso, err) = tc2(&f, &a, &["roadmap"]);
    assert_eq!(c, 0, "{impreso} {err}");
    assert!(!impreso.contains("sincronizado"), "{impreso}");
    let (c, out, _) = tc2(&f, &a, &["roadmap", "--comparar"]);
    assert_eq!((c, valor(&out, "motivo")), (3, Some("desfasado")), "{out}");
    // `--comparar` tambien dice la frescura (M1 de la revision 1).
    assert_eq!(valor(&out, "remoto"), Some("consultado"), "{out}");
    assert!(valor(&out, "sincronizado").is_some_and(es_hora), "{out}");
    let (c, out, err) = lanzar(&a, f.entorno_sin_red(), "es", &["roadmap", "--comparar"]);
    assert_eq!(c, 3, "{out} {err}");
    assert_eq!(valor(&out, "remoto"), Some("sin-red"), "{out}");
    assert!(valor(&out, "sincronizado").is_some_and(es_hora), "{out}");
    // El JSON lleva el mismo texto.
    let (_, json, _) = tc2(&f, &a, &["roadmap", "--json"]);
    assert_eq!(claves(&json)["texto"], impreso.as_str());
    let (c, out, err) = tc2(&f, &a, &["roadmap", "--escribir"]);
    assert_eq!(c, 0, "{out} {err}");
    let publicado = f.crudo(&bare, &["show", &format!("{RAMA}:ROADMAP.md")]);
    assert_eq!(publicado, impreso);
    let (c, out, _) = tc2(&f, &a, &["roadmap", "--comparar"]);
    assert_eq!(c, 0, "{out}");
}

/// Refs de pendientes del clon.
fn pendientes(f: &Fixture, clon: &Path) -> String {
    f.crudo(clon, &["for-each-ref", "refs/taskcodetwo/pendientes/"])
}

/// H1 de la revision 1: la rama de estado reescrita en `origin` (el `fetch`
/// sin `+` la rechaza non-fast-forward) no es `sin-red`. La lectura sale 0
/// con `remoto=rechazado` y su aviso; `new` sale `error` (1) con
/// `motivo=rama-reescrita`, sin dejar pendiente ni tocar el remoto.
#[test]
fn c10_rama_reescrita_en_origin_no_es_sin_red() {
    let f = Fixture::nueva("frescura-c10");
    let (bare, a) = con_tarea(&f);
    let commit = claves(&tc2(&f, &a, &["status", "--json"]).1)["commit"]
        .as_str()
        .unwrap()
        .to_owned();
    // Un commit huerfano (arbol vacio) en lugar de la rama de estado.
    let huerfano = f
        .crudo(
            &bare,
            &[
                "commit-tree",
                "4b825dc642cb6eb9a060e54bf8d69288fbee4904",
                "-m",
                "reescrita",
            ],
        )
        .trim()
        .to_owned();
    f.crudo(
        &bare,
        &["update-ref", &format!("refs/heads/{RAMA}"), &huerfano],
    );
    let (c, out, err) = tc2(&f, &a, &["status", "--json"]);
    assert_eq!(c, 0, "{out} {err}");
    let k = claves(&out);
    assert_eq!(k["remoto"], "rechazado", "{out}");
    assert_eq!(k["commit"], commit.as_str(), "lo ultimo traido: {out}");
    assert!(
        err.starts_with("aviso: origin tiene la rama de estado reescrita"),
        "{err}"
    );
    let (_, _, en) = lanzar(&a, f.entorno_ssh(), "en", &["status"]);
    assert!(
        en.starts_with("warning: origin has the state branch rewritten"),
        "{en}"
    );
    let (c, out, err) = tc2(&f, &a, &["new", "Tras reescribir"]);
    assert_eq!(c, 1, "{out} {err}");
    assert!(
        format!("{out}{err}").contains("motivo=rama-reescrita"),
        "{out} {err}"
    );
    assert!(err.contains("[rejected]"), "con la linea de git: {err}");
    assert_eq!(pendientes(&f, &a), "", "no queda pendiente");
    let en_bare = f.crudo(&bare, &["rev-parse", &format!("refs/heads/{RAMA}")]);
    assert_eq!(en_bare.trim(), huerfano, "el remoto no cambia");
}

/// H1 y H4 de la revision 1: un `.lock` huerfano en la ref remota de la
/// rama de estado no es `sin-red`. La lectura reintenta, sigue bloqueada y
/// sale 0 con `remoto=error-local` y un aviso que dice el `.lock`; `new`
/// sale `error` (1) con `motivo=error-local` sin dejar pendiente. Quitado
/// el `.lock`, todo vuelve a `consultado`.
#[test]
fn c11_ref_remota_bloqueada_no_es_sin_red() {
    let f = Fixture::nueva("frescura-c11");
    let (bare, a) = con_tarea(&f);
    let (c, _, _) = tc2(&f, &a, &["status"]);
    assert_eq!(c, 0);
    // Algo nuevo en `origin`, para que el `fetch` tenga que mover la ref.
    let b = f.clon_github(&bare, "b");
    nueva(&f, &b, "Segunda");
    let lock = a
        .join(".git")
        .join("refs")
        .join("remotes")
        .join("origin")
        .join(format!("{RAMA}.lock"));
    std::fs::create_dir_all(lock.parent().unwrap()).unwrap();
    std::fs::write(&lock, "").unwrap();
    let (c, out, err) = tc2(&f, &a, &["status", "--json"]);
    assert_eq!(c, 0, "{out} {err}");
    let k = claves(&out);
    assert_eq!(k["remoto"], "error-local", "{out}");
    assert!(!out.contains("TASK-002"), "no se trajo: {out}");
    assert_eq!(err.lines().count(), 1, "{err}");
    assert!(
        err.starts_with("aviso: la ref de la rama de estado de este clon esta bloqueada")
            && err.contains("cannot lock ref"),
        "{err}"
    );
    let (c, out, err) = tc2(&f, &a, &["new", "Con el lock"]);
    assert_eq!(c, 1, "{out} {err}");
    assert!(
        format!("{out}{err}").contains("motivo=error-local"),
        "{out} {err}"
    );
    assert_eq!(pendientes(&f, &a), "", "no queda pendiente");
    std::fs::remove_file(&lock).unwrap();
    let (c, out, err) = tc2(&f, &a, &["status", "--json"]);
    assert_eq!((c, err.as_str()), (0, ""), "{out}");
    assert_eq!(claves(&out)["remoto"], "consultado", "{out}");
    assert!(out.contains("TASK-002"), "{out}");
}
