//! F3-12 (TASK-084): progreso en vivo con el binario real: `tc2 estado
//! --stdin` (la status line de Claude Code) y `tc2 status --seguir`, sobre
//! repos en una carpeta temporal.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use comun::{Fixture, escribir};
use tc2::contrato::Resultado;
use tc2::estado::{self, Cambio, Contexto, Estado, Fallo, Operacion};
use tc2::identidad::{Cache, Cuenta};
use tc2::plataforma::Proveedor;
use tc2::progreso::{self, Refresco};
use tc2::tarea::{Cuerpo, Tarea};

const ORIGIN: &str = "ssh://git@github.com/o/r.git";
const PERSONAS: &str =
    "---\npersonas:\n  - {id: 1001, login: \"ana\"}\n  - {id: 1002, login: \"beto\"}\n";

fn texto(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

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

/// `tc2 <args>` en `dir` con el remoto de la fixture: (codigo, stdout,
/// stderr).
fn tc2(f: &Fixture, dir: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_tc2"))
        .current_dir(dir)
        .args(args)
        .envs(entorno_ssh(f))
        .env("LC_ALL", "C")
        .env("TC2_IDIOMA", "es")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env_remove("TC2_TOKEN_PERSONA")
        .env_remove("TC2_TOKEN_AGENTE")
        .output()
        .expect("lanzar tc2");
    (o.status.code(), texto(&o.stdout), texto(&o.stderr))
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

/// Remoto con `personas.yml` en `develop` y el clon `a` de ana (`origin` en
/// GitHub, servido por el «ssh» de la fixture).
fn preparar(f: &Fixture) -> (PathBuf, PathBuf) {
    let (bare, prep) = f.remoto_y_clon();
    escribir(&prep, ".taskcode/personas.yml", PERSONAS);
    f.crudo(&prep, &["add", ".taskcode/personas.yml"]);
    f.crudo(&prep, &["commit", "-q", "-m", "personas"]);
    f.crudo(&prep, &["push", "-q", "origin", "develop"]);
    let a = f.otro_clon(&bare, "a");
    f.crudo(&a, &["remote", "set-url", "origin", ORIGIN]);
    identificar(&a, 1001, "ana");
    (bare, a)
}

/// Escribe ficheros en la rama de estado por la transaccion.
struct Poner(Vec<(String, String)>);

impl Operacion for Poner {
    fn nombre(&self) -> &str {
        "poner"
    }
    fn anotacion(&self) -> String {
        "poner".to_owned()
    }
    fn exige_red(&self) -> bool {
        true
    }
    fn aplicar(&self, _: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        Ok(self
            .0
            .iter()
            .map(|(r, t)| Cambio::nuevo(r.clone(), t.clone()))
            .collect())
    }
}

fn sin_catalogo(_: &str) -> Option<Box<dyn Operacion>> {
    None
}

fn poner(f: &Fixture, dir: &Path, ficheros: &[(String, String)]) {
    let g = tc2::git::Git::nuevo().con_entorno(entorno_ssh(f));
    let i = estado::transaccion(
        &Contexto::nuevo(&g, dir, &sin_catalogo, None),
        &Poner(ficheros.to_vec()),
    );
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
}

/// 2026-10-01T09:00:00Z.
const BASE: u64 = 1_790_845_200;

fn fila(ts: u64, fase: &str) -> String {
    format!(
        "| {} | {fase} | manual | persona | 0.1 |\n",
        tc2::toma::instante(ts)
    )
}

const CABECERA: &str =
    "## Transiciones\n\n| fecha | fase | modo | decidido_por | version |\n|---|---|---|---|---|\n";

/// Una terminada con `imp` minutos de implementacion, 10 de rondas y 5 de
/// cierre.
fn terminada(id: &str, dia: u64, imp: u64) -> (String, String) {
    let mut t = Tarea::nueva(id, &format!("Hecha {id}"), "2026-10-01");
    "terminada".clone_into(&mut t.estado);
    t.plan_aprobado = true;
    let s = BASE + dia * 86_400;
    let r1 = s + imp * 60;
    let cuerpo = format!(
        "## Objetivo\n\nHecha.\n\n## Criterios de aceptacion\n\n- [x] a\n\n{CABECERA}{}{}{}{}",
        fila(s, "start"),
        fila(r1, "review"),
        fila(r1 + 600, "review"),
        fila(r1 + 900, "finish"),
    );
    (format!("{id}/tarea.md"), t.escribir(&Cuerpo::leer(&cuerpo)))
}

/// Seis terminadas (implementacion 30..80 min).
fn seis() -> Vec<(String, String)> {
    (1..=6)
        .map(|i| terminada(&format!("TASK-{i:03}"), i, 20 + 10 * i))
        .collect()
}

/// Carpeta del progreso del clon `dir`.
fn carpeta(dir: &Path) -> PathBuf {
    dir.join(".git").join("taskcodetwo").join("progreso")
}

fn worktree(dir: &Path, id: &str) -> PathBuf {
    dir.join(".taskcodetwo").join("worktrees").join(id)
}

/// La linea de `tc2 estado --stdin` para la carpeta `dir` (codigo 0 y
/// stderr vacio comprobados).
fn linea(dir: &Path) -> String {
    let o = estado_stdin(dir, &json_de(dir), &[]);
    assert_eq!(o.status.code(), Some(0));
    assert!(o.stderr.is_empty(), "{}", texto(&o.stderr));
    let s = texto(&o.stdout);
    assert_eq!(s.lines().count(), 1, "{s}");
    s.trim_end().to_owned()
}

/// C3: en un E2E con remoto local, tras `start`, `pausa` y `reanudar` el
/// fichero se reescribe, la linea cambia y el anterior queda en
/// `.anterior.json`; tras `descartar` o `liberar`, «sin tarea en curso». Un
/// fallo al escribir el progreso no cambia la transicion.
#[test]
fn c3_transiciones_reales_reescriben_el_progreso() {
    let f = Fixture::nueva("progreso-c3");
    let (_bare, a) = preparar(&f);
    poner(&f, &a, &seis());
    for t in ["Siete", "Ocho"] {
        let (c, out, err) = tc2(&f, &a, &["new", t, "--criterio", "uno"]);
        assert_eq!(c, Some(0), "{out} {err}");
    }
    let p = carpeta(&a);
    let actual = p.join("TASK-007.json");
    let anterior = p.join("TASK-007.anterior.json");
    assert_eq!(linea(&a), "tc2 · sin tarea en curso");
    // start
    let (c, out, err) = tc2(&f, &a, &["start", "TASK-007"]);
    assert_eq!(c, Some(0), "{out} {err}");
    assert!(err.is_empty() && !out.contains("progreso="), "{out} {err}");
    let tras_start = std::fs::read(&actual).unwrap();
    assert!(!anterior.exists());
    let wt = worktree(&a, "TASK-007");
    let l_start = linea(&wt);
    assert!(
        l_start
            .starts_with("TASK-007 · implementacion · fases que faltan: rondas, cierre · faltan ≈")
            && l_start.ends_with(" · n=6 repo"),
        "{l_start}"
    );
    // Desde una subcarpeta del worktree y desde el principal, la misma.
    std::fs::create_dir_all(wt.join("src")).unwrap();
    assert_eq!(linea(&wt.join("src")), l_start);
    assert_eq!(linea(&a), l_start);
    // pausa
    let (c, out, err) = tc2(&f, &a, &["pausa", "TASK-007"]);
    assert_eq!(c, Some(0), "{out} {err}");
    let tras_pausa = std::fs::read(&actual).unwrap();
    assert_ne!(tras_pausa, tras_start);
    assert_eq!(std::fs::read(&anterior).unwrap(), tras_start);
    let l_pausa = linea(&wt);
    assert!(
        l_pausa.starts_with("TASK-007 · pausada en implementacion · "),
        "{l_pausa}"
    );
    // reanudar
    let (c, out, err) = tc2(&f, &a, &["reanudar", "TASK-007"]);
    assert_eq!(c, Some(0), "{out} {err}");
    assert_ne!(std::fs::read(&actual).unwrap(), tras_pausa);
    assert_eq!(std::fs::read(&anterior).unwrap(), tras_pausa);
    let l_reanudar = linea(&wt);
    assert!(
        l_reanudar.starts_with("TASK-007 · implementacion · "),
        "{l_reanudar}"
    );
    // Otra en curso: el principal ensena la mas reciente y `+1 en curso`.
    let (c, out, err) = tc2(&f, &a, &["start", "TASK-008"]);
    assert_eq!(c, Some(0), "{out} {err}");
    let l = linea(&a);
    assert!(
        l.starts_with("TASK-008 · ") && l.ends_with(" · +1 en curso"),
        "{l}"
    );
    // Un fallo al escribir el progreso no cambia la transicion: mismo
    // codigo y la linea de siempre con `progreso=no-escrito`.
    let ocho = p.join("TASK-008.json");
    std::fs::remove_file(&ocho).unwrap();
    std::fs::create_dir_all(ocho.join("bloqueo")).unwrap();
    let (c, out, err) = tc2(&f, &a, &["pausa", "TASK-008"]);
    assert_eq!(c, Some(0), "{out} {err}");
    assert!(
        out.starts_with("pausa ok id=TASK-008 ")
            && out.trim_end().ends_with(" progreso=no-escrito"),
        "{out}"
    );
    assert!(
        err.starts_with("aviso: no se pudo escribir el progreso de TASK-008"),
        "{err}"
    );
    assert_eq!(err.lines().count(), 1, "{err}");
    std::fs::remove_dir_all(&ocho).unwrap();
    let (c, out, err) = tc2(&f, &a, &["reanudar", "TASK-008"]);
    assert_eq!((c, err.as_str()), (Some(0), ""), "{out}");
    assert!(ocho.is_file());
    // descartar: sin tarea en curso en su worktree (ya quitado) y el
    // principal sigue con la otra.
    let (c, out, err) = tc2(&f, &a, &["descartar", "TASK-007", "ya no hace falta"]);
    assert_eq!(c, Some(0), "{out} {err}");
    assert!(!actual.exists() && !anterior.exists());
    assert_eq!(linea(&wt), "tc2 · sin tarea en curso");
    let l = linea(&a);
    assert!(
        l.starts_with("TASK-008 · ") && !l.contains("en curso ·"),
        "{l}"
    );
    // liberar (con la rama publicada): sin tarea en curso.
    let o = Command::new("git")
        .current_dir(&a)
        .args(["push", "-q", "origin", "feature/task-008-ocho"])
        .envs(entorno_ssh(&f))
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", texto(&o.stderr));
    let (c, out, err) = tc2(&f, &a, &["liberar", "TASK-008"]);
    assert_eq!(c, Some(0), "{out} {err}");
    assert!(!ocho.exists());
    assert_eq!(linea(&a), "tc2 · sin tarea en curso");
}

/// Un repo con `n` tareas en la rama de estado: las seis terminadas, la
/// `TASK-007` en curso desde `inicio` y el resto planificadas. Devuelve el
/// clon.
fn repo_con(f: &Fixture, n: usize, inicio: u64) -> PathBuf {
    let (_bare, a) = preparar(f);
    let mut ficheros = seis();
    let mut t = Tarea::nueva("TASK-007", "En curso", "2026-11-17");
    "en-curso".clone_into(&mut t.estado);
    let cuerpo = format!(
        "## Objetivo\n\nX.\n\n## Criterios de aceptacion\n\n- [ ] a\n\n{CABECERA}{}",
        fila(inicio, "start")
    );
    ficheros.push((
        "TASK-007/tarea.md".to_owned(),
        t.escribir(&Cuerpo::leer(&cuerpo)),
    ));
    for i in 8..=n {
        let id = format!("TASK-{i:03}");
        let t = Tarea::nueva(&id, &format!("Planificada {i}"), "2026-11-17");
        ficheros.push((format!("{id}/tarea.md"), t.escribir(&Cuerpo::plantilla())));
    }
    // Por tandas: `hash-object` recibe las rutas en la linea de ordenes.
    for tanda in ficheros.chunks(100) {
        poner(f, &a, tanda);
    }
    a
}

/// C1: con 10 y con 1000 tareas en la rama de estado, `tc2 estado --stdin`
/// da la misma salida leyendo los mismos ficheros (el de progreso, igual
/// byte a byte), sin lanzar procesos (con un PATH sin git da lo mismo) y en
/// menos de 200 ms (mediana de 7 en caliente).
#[test]
fn c1_misma_salida_con_10_y_1000_tareas_sin_procesos_y_rapida() {
    let ahora = tc2::identidad::ahora();
    let inicio = ahora - 20 * 60;
    let mut salidas = Vec::new();
    let mut ficheros = Vec::new();
    let vacio = std::env::temp_dir()
        .join("tc2-tests")
        .join(format!("progreso-c1-path-{}", std::process::id()));
    std::fs::create_dir_all(&vacio).unwrap();
    for n in [10, 1000] {
        let f = Fixture::nueva(&format!("progreso-c1-{n}"));
        let a = repo_con(&f, n, inicio);
        let g = tc2::git::Git::nuevo().con_entorno(entorno_ssh(&f));
        assert_eq!(
            progreso::refrescar(&g, &a, "TASK-007", ahora),
            Ok(Refresco::Escrito)
        );
        let bytes = std::fs::read(carpeta(&a).join("TASK-007.json")).unwrap();
        // Lo que hay en la carpeta: solo el fichero de la tarea.
        assert_eq!(std::fs::read_dir(carpeta(&a)).unwrap().count(), 1);
        ficheros.push(bytes);
        let l = linea(&a);
        // Sin git en el PATH: la misma linea (no lanza ningun proceso).
        let o = estado_stdin(&a, &json_de(&a), &[("PATH", vacio.as_os_str())]);
        assert_eq!(texto(&o.stdout).trim_end(), l, "{}", texto(&o.stderr));
        // Tiempo en caliente.
        let mut ms: Vec<u128> = (0..7)
            .map(|_| {
                let t0 = std::time::Instant::now();
                let o = estado_stdin(&a, &json_de(&a), &[]);
                let d = t0.elapsed().as_millis();
                assert_eq!(texto(&o.stdout).trim_end(), l);
                d
            })
            .collect();
        ms.sort_unstable();
        println!(
            "estado --stdin con {n} tareas: {ms:?} ms (mediana {})",
            ms[3]
        );
        assert!(ms[3] < 200, "mediana {} ms: {ms:?}", ms[3]);
        salidas.push(l);
    }
    let _ = std::fs::remove_dir_all(&vacio);
    assert_eq!(ficheros[0], ficheros[1]);
    assert_eq!(salidas[0], salidas[1]);
    assert!(
        salidas[0]
            .starts_with("TASK-007 · implementacion · fases que faltan: rondas, cierre · faltan ≈"),
        "{}",
        salidas[0]
    );
}

/// C8: un fichero corrupto o a medias da el ultimo valor bueno con su
/// antiguedad; si tampoco hay anterior, «progreso ilegible» (codigo 0).
#[test]
fn c8_fichero_a_medias_da_el_ultimo_bueno() {
    let f = Fixture::nueva("progreso-c8");
    let ahora = tc2::identidad::ahora();
    let a = repo_con(&f, 8, ahora - 600);
    let g = tc2::git::Git::nuevo().con_entorno(entorno_ssh(&f));
    // Dos calculos: el primero hace 10 min (queda de anterior).
    assert_eq!(
        progreso::refrescar(&g, &a, "TASK-007", ahora - 600),
        Ok(Refresco::Escrito)
    );
    assert_eq!(
        progreso::refrescar(&g, &a, "TASK-007", ahora),
        Ok(Refresco::Escrito)
    );
    let actual = carpeta(&a).join("TASK-007.json");
    let bueno = linea(&a);
    let json = std::fs::read(&actual).unwrap();
    std::fs::write(&actual, &json[..json.len() / 2]).unwrap();
    let l = linea(&a);
    assert!(l.starts_with("TASK-007 · implementacion · "), "{l}");
    assert!(l.ends_with(" · (dato de hace 10 min)"), "{l}");
    assert!(!bueno.contains("dato de hace"), "{bueno}");
    std::fs::write(&actual, b"\x00\xff basura").unwrap();
    assert!(linea(&a).ends_with(" · (dato de hace 10 min)"));
    std::fs::remove_file(carpeta(&a).join("TASK-007.anterior.json")).unwrap();
    assert_eq!(linea(&a), "TASK-007 · progreso ilegible");
}

/// `tc2 estado --stdin` en `dir` con `json` por stdin y el entorno `vars`.
fn estado_stdin(dir: &Path, json: &str, vars: &[(&str, &std::ffi::OsStr)]) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_tc2"));
    c.current_dir(dir)
        .args(["estado", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (k, v) in vars {
        c.env(k, v);
    }
    let mut hijo = c.spawn().expect("lanzar tc2");
    hijo.stdin
        .take()
        .unwrap()
        .write_all(json.as_bytes())
        .unwrap();
    hijo.wait_with_output().unwrap()
}

/// El JSON de la status line con `workspace.current_dir`.
fn json_de(dir: &Path) -> String {
    serde_json::json!({
        "model": {"display_name": "Opus"},
        "workspace": {"current_dir": dir.display().to_string(), "project_dir": "x"},
        "session_id": "s"
    })
    .to_string()
}

/// C7: sin tarea en curso, fuera de un repo o con un JSON que no vale, sale
/// `tc2 · sin tarea en curso` con codigo 0 y stderr vacio.
#[test]
fn c7_sin_tarea_o_fuera_de_un_repo() {
    let f = Fixture::nueva("progreso-c7");
    let (_bare, clon) = f.remoto_y_clon();
    let fuera = f.raiz.join("fuera");
    std::fs::create_dir_all(&fuera).unwrap();
    for (dir, json) in [
        (&clon, json_de(&clon)),
        (&clon, "no es json".to_owned()),
        (&clon, String::new()),
        (&fuera, json_de(&fuera)),
        (&fuera, json_de(&f.raiz.join("no-existe"))),
    ] {
        let o = estado_stdin(dir, &json, &[]);
        assert_eq!(o.status.code(), Some(0), "{json}");
        assert_eq!(texto(&o.stdout), "tc2 · sin tarea en curso\n", "{json}");
        assert!(o.stderr.is_empty(), "{}", texto(&o.stderr));
    }
}
