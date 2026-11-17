//! `tc2 desinstalar` (TASK-078, F2-14): un test (o mas) por criterio.
//!
//! Seguridad: nada toca el HOME, el `~/.claude`, el registro ni el PATH de
//! esta maquina. El HOME, `LOCALAPPDATA` y el ejecutable son falsos, dentro
//! de la carpeta de la fixture (bajo `temp_dir`, que se borra al acabar), y
//! el PATH de usuario, los procesos y el autoborrado van por
//! `MaquinaEnMemoria`. El PATH real de Windows y el autoborrado de verdad
//! solo se prueban en CI (`.github/scripts/probar-instalador.sh`).

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use comun::{Fixture, escribir};
use tc2::contrato::Resultado;
use tc2::desinstalar::{Contexto, Hecho, MaquinaEnMemoria, Opciones, desinstalar};
use tc2::i18n::Idioma;
use tc2::instalacion::{Entorno, Sistema};
use tc2::{estado, git};

/// Bloque que escribe `install.sh` (su `printf`, con las constantes que el
/// test de `xtask` exige a `install.sh`).
fn bloque(dir: &str) -> String {
    format!(
        "\n{}\n{}*\":{dir}:\"*) ;; *) export PATH=\"{dir}:$PATH\" ;; esac\n{}\n",
        distribucion::MARCA_INICIO_PERFIL,
        distribucion::LINEA_PERFIL,
        distribucion::MARCA_FIN_PERFIL
    )
}

/// Instantanea de una carpeta: ruta relativa → (tamano, sha256), y las
/// carpetas con su marca.
fn instantanea(dir: &Path) -> BTreeMap<String, String> {
    fn recorrer(base: &Path, d: &Path, m: &mut BTreeMap<String, String>) {
        let Ok(it) = std::fs::read_dir(d) else { return };
        for e in it.flatten() {
            let p = e.path();
            let rel = p
                .strip_prefix(base)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if p.is_dir() {
                m.insert(rel, "dir".to_owned());
                recorrer(base, &p, m);
            } else {
                let b = std::fs::read(&p).unwrap();
                m.insert(
                    rel,
                    format!("{} {}", b.len(), distribucion::sha256::hex(&b)),
                );
            }
        }
    }
    let mut m = BTreeMap::new();
    recorrer(dir, dir, &mut m);
    m
}

fn entorno(sistema: Sistema, home: &Path, exe: PathBuf) -> Entorno {
    Entorno {
        sistema,
        home: Some(home.to_path_buf()),
        local_app_data: Some(home.join("AppData").join("Local")),
        xdg_cache_home: None,
        install_dir: None,
        claude_config_dir: None,
        exe,
        pid: 4242,
    }
}

fn op(args: &[&str]) -> Opciones {
    let v: Vec<String> = args.iter().map(|s| (*s).to_owned()).collect();
    tc2::desinstalar::leer_opciones(&v).unwrap()
}

fn correr(e: &Entorno, dir: &Path, g: &git::Git, m: &MaquinaEnMemoria, args: &[&str]) -> Hecho {
    let c = Contexto {
        entorno: e,
        dir,
        git: g,
        maquina: m,
    };
    desinstalar(&op(args), &c)
}

fn stdout(h: &Hecho) -> String {
    h.salida_en(false, Idioma::Es).stdout
}

/// Las lineas de accion con `clase` (y su `decision`).
fn decisiones(h: &Hecho, clase: &str) -> Vec<&'static str> {
    h.acciones
        .iter()
        .filter(|a| a.clase == clase)
        .map(|a| a.decision.texto())
        .collect()
}

/// Lo que deja `install.sh` (Unix) en un HOME: el binario, un temporal de
/// cada clase, el bloque en `.profile` y `.bashrc`, y la cache del lanzador.
fn instalacion_unix(home: &Path) -> (PathBuf, String) {
    let bin = home.join(".local").join("bin");
    escribir(&bin, "tc2", "binario");
    escribir(&bin, ".tc2-2.0.0-77.descarga", "a medias");
    escribir(&bin, ".tc2.nuevo.88", "a medias");
    escribir(&bin, "otra-herramienta", "de la persona");
    let perfil = "# mio\nalias ll='ls -l'\n";
    let dir = bin.to_string_lossy().replace('\\', "/");
    escribir(home, ".profile", &format!("{perfil}{}", bloque(&dir)));
    escribir(home, ".bashrc", &format!("{perfil}{}", bloque(&dir)));
    escribir(home, ".zshrc", perfil);
    escribir(
        &home
            .join(".cache")
            .join("taskcodetwo")
            .join("cli")
            .join("2.0.0"),
        "tc2",
        "del lanzador",
    );
    (bin.join("tc2"), perfil.to_owned())
}

/// Lo que deja `install.ps1` (Windows) en un `LOCALAPPDATA`: el binario, el
/// `.old` de `actualizar --cli`, temporales y la cache del lanzador.
fn instalacion_windows(home: &Path) -> PathBuf {
    let raiz = home.join("AppData").join("Local").join("taskcodetwo");
    let bin = raiz.join("bin");
    escribir(&bin, "tc2.exe", "binario");
    escribir(&bin, "tc2.exe.old", "anterior");
    escribir(&bin, ".tc2-2.0.0-77.descarga", "a medias");
    escribir(&bin, ".tc2.nuevo.88.exe", "a medias");
    escribir(&raiz.join("cli").join("2.0.0"), "tc2.exe", "del lanzador");
    bin.join("tc2.exe")
}

/// Repo con la rama de estado montada y un worktree de tarea publicado.
fn repo_con_estado(f: &Fixture) -> (PathBuf, PathBuf) {
    let (bare, clon) = f.remoto_y_clon();
    let g = f.git();
    assert!(estado::montar(&g, &clon).resultado.es_exito());
    let i = git::crear_rama_y_worktree(&g, &clon, "TASK-001", "feature", "x", "origin/develop");
    assert!(i.resultado.es_exito(), "{}", i.linea());
    let wt = clon.join(".taskcodetwo").join("worktrees").join("TASK-001");
    f.crudo(&wt, &["push", "-q", "origin", "feature/task-001-x"]);
    // Una linea de la persona en exclude, que debe quedar.
    let ex = clon.join(".git").join("info").join("exclude");
    let mut t = std::fs::read_to_string(&ex).unwrap();
    t.push_str("/mio/\n");
    std::fs::write(&ex, t).unwrap();
    (bare, clon)
}

/// C1: `--simular` en los dos ambitos imprime una linea por accion con
/// ruta, clase, dueno y decision, sale 0 y no cambia ni un byte.
#[test]
fn simular_no_toca_nada() {
    let f = Fixture::nueva("desinstalar-simular");
    let (bare, clon) = repo_con_estado(&f);
    let home = f.raiz.join("home");
    let (exe, _) = instalacion_unix(&home);
    let _ = instalacion_windows(&home);
    escribir(
        &home.join(".claude"),
        "settings.json",
        "{\"statusLine\": {\"type\": \"command\", \"command\": \"tc2 estado --stdin\"}}\n",
    );
    let g = f.git();
    let antes = [instantanea(&home), instantanea(&clon), instantanea(&bare)];
    for sistema in [Sistema::Linux, Sistema::Windows] {
        let exe = if sistema == Sistema::Windows {
            home.join("AppData/Local/taskcodetwo/bin/tc2.exe")
        } else {
            exe.clone()
        };
        let e = entorno(sistema, &home, exe);
        let m = MaquinaEnMemoria {
            path: Some(format!(
                "C:\\otro;{}",
                home.join("AppData/Local/taskcodetwo/bin").display()
            ))
            .into(),
            ..MaquinaEnMemoria::default()
        };
        for args in [
            &["--maquina", "--simular"][..],
            &["--proyecto", "--simular"][..],
            &[
                "--proyecto",
                "--maquina",
                "--simular",
                "--config",
                "--borrar-tareas",
                "taskcodetwo/tareas",
            ][..],
        ] {
            let h = correr(&e, &clon, &g, &m, args);
            let s = h.salida_en(false, Idioma::Es);
            assert_eq!(s.codigo, 0, "{args:?}: {}{}", s.stdout, s.stderr);
            let lineas: Vec<&str> = s.stdout.lines().collect();
            assert_eq!(lineas.len(), h.acciones.len() + 1, "{}", s.stdout);
            assert!(
                h.acciones.iter().any(|a| a.decision.texto() == "borra"),
                "{}",
                s.stdout
            );
            for l in &lineas[..lineas.len() - 1] {
                assert!(l.starts_with("accion ruta="), "{l}");
                for k in [" clase=", " dueno=", " decision="] {
                    assert!(l.contains(k), "{l}");
                }
            }
            assert!(
                lineas.last().unwrap().starts_with("desinstalar ok "),
                "{}",
                s.stdout
            );
            assert!(lineas.last().unwrap().contains(" simular=si"));
            assert!(m.borrados.borrow().is_empty());
        }
        assert!(m.path.borrow().as_deref().unwrap().contains("taskcodetwo"));
    }
    let despues = [instantanea(&home), instantanea(&clon), instantanea(&bare)];
    assert_eq!(antes, despues);
}

/// C2 (Unix): quita el binario, los temporales, el bloque de cada perfil
/// (el resto, byte a byte) y la cache del lanzador; lo de la persona queda.
#[test]
fn maquina_unix_quita_lo_suyo_y_deja_el_perfil_igual() {
    let f = Fixture::nueva("desinstalar-unix");
    let home = f.raiz.join("home");
    let (exe, perfil) = instalacion_unix(&home);
    let e = entorno(Sistema::Linux, &home, exe.clone());
    let m = MaquinaEnMemoria::default();
    let h = correr(&e, &f.raiz, &f.git(), &m, &["--maquina"]);
    assert_eq!(h.informe.resultado, Resultado::Ok, "{}", stdout(&h));
    let bin = home.join(".local").join("bin");
    assert!(!exe.exists());
    assert!(!bin.join(".tc2-2.0.0-77.descarga").exists());
    assert!(!bin.join(".tc2.nuevo.88").exists());
    assert!(
        bin.join("otra-herramienta").is_file(),
        "lo de la persona se queda"
    );
    for p in [".profile", ".bashrc", ".zshrc"] {
        assert_eq!(
            std::fs::read_to_string(home.join(p)).unwrap(),
            perfil,
            "{p}"
        );
    }
    assert!(!home.join(".cache").join("taskcodetwo").exists());
    assert!(home.join(".cache").is_dir());
    assert!(
        m.borrados.borrow().is_empty(),
        "en Unix se borra en el momento"
    );
}

/// C2 y C8 (Windows, con la maquina en memoria): quita `.old`, temporales,
/// la entrada del PATH de usuario (las demas intactas) y la cache; el
/// binario en marcha lo borra un auxiliar al salir, con su carpeta.
#[test]
fn maquina_windows_path_y_autoborrado() {
    let f = Fixture::nueva("desinstalar-windows");
    let home = f.raiz.join("home");
    let exe = instalacion_windows(&home);
    let bin = exe.parent().unwrap().to_path_buf();
    let raiz = bin.parent().unwrap().to_path_buf();
    let e = entorno(Sistema::Windows, &home, exe.clone());
    let path = format!(
        "C:\\otro;%LOCALAPPDATA%\\taskcodetwo\\bin;;%SYSTEMROOT%\\x;{}\\",
        bin.display()
    );
    let m = MaquinaEnMemoria {
        path: Some(path).into(),
        ..MaquinaEnMemoria::default()
    };
    let h = correr(&e, &f.raiz, &f.git(), &m, &["--maquina"]);
    assert_eq!(h.informe.resultado, Resultado::Ok, "{}", stdout(&h));
    assert_eq!(
        m.path.borrow().as_deref(),
        Some("C:\\otro;;%SYSTEMROOT%\\x")
    );
    assert!(!bin.join("tc2.exe.old").exists());
    assert!(!bin.join(".tc2-2.0.0-77.descarga").exists());
    assert!(!bin.join(".tc2.nuevo.88.exe").exists());
    assert!(!raiz.join("cli").exists());
    // El binario en marcha: auxiliar que espera al PID y borra.
    let b = m.borrados.borrow();
    assert_eq!(b.len(), 1);
    assert_eq!(b[0].pid, 4242);
    assert_eq!(b[0].ficheros, vec![exe.clone()]);
    assert_eq!(b[0].dirs, vec![bin.clone(), raiz.clone()]);
    assert!(stdout(&h).contains("clase=binario dueno=taskcodetwo decision=borra motivo=al-salir"));
}

/// C8: si otro proceso ejecuta el binario, `pendiente motivo=en-uso` y ni el
/// binario, ni su `.old`, ni su carpeta, ni su entrada del PATH se tocan.
#[test]
fn binario_en_uso_por_otro_queda_pendiente_sin_nada_a_medias() {
    let f = Fixture::nueva("desinstalar-en-uso");
    let home = f.raiz.join("home");
    let exe = instalacion_windows(&home);
    let bin = exe.parent().unwrap().to_path_buf();
    let e = entorno(Sistema::Windows, &home, exe.clone());
    let path = format!("C:\\otro;{}", bin.display());
    let m = MaquinaEnMemoria {
        path: Some(path.clone()).into(),
        en_uso: vec![exe.clone()],
        ..MaquinaEnMemoria::default()
    };
    let antes = instantanea(&bin);
    let h = correr(&e, &f.raiz, &f.git(), &m, &["--maquina"]);
    let s = h.salida_en(false, Idioma::Es);
    assert_eq!(s.codigo, 3, "{}", s.stdout);
    assert!(s.stdout.lines().last().unwrap().contains("motivo=en-uso"));
    assert!(s.stdout.contains("decision=pendiente motivo=en-uso"));
    assert_eq!(instantanea(&bin), antes);
    assert_eq!(m.path.borrow().as_deref(), Some(path.as_str()));
    assert!(m.borrados.borrow().is_empty());
    // Lo demas si se quito.
    assert!(!bin.parent().unwrap().join("cli").exists());
}

/// C3: la status line se quita solo si su `command` es el de la 2.0; la de
/// la 0.16 y una ajena quedan identicas byte a byte.
#[test]
fn status_line_solo_la_de_la_2() {
    let f = Fixture::nueva("desinstalar-status");
    for (texto, se_quita, dueno) in [
        (
            "{\n  \"model\": \"x\",\n  \"statusLine\": {\n    \"type\": \"command\",\n    \"command\": \"tc2 estado --stdin\"\n  },\n  \"env\": {}\n}\n",
            true,
            "taskcodetwo",
        ),
        (
            "{\n  \"statusLine\": {\"type\": \"command\", \"command\": \"taskctl estado --stdin\"}\n}\n",
            false,
            "taskcode-016",
        ),
        (
            "{\"statusLine\":{\"type\":\"command\",\"command\":\"npx ccstatusline tc2 estado --stdin\"}}",
            false,
            "ajeno",
        ),
    ] {
        let home = f.raiz.join(format!("home-{dueno}"));
        escribir(&home.join(".claude"), "settings.json", texto);
        let e = entorno(Sistema::Linux, &home, home.join("x").join("tc2"));
        let h = correr(
            &e,
            &f.raiz,
            &f.git(),
            &MaquinaEnMemoria::default(),
            &["--maquina"],
        );
        let ahora = std::fs::read_to_string(home.join(".claude").join("settings.json")).unwrap();
        let a = h
            .acciones
            .iter()
            .find(|a| a.clase == "status-line")
            .unwrap();
        assert_eq!(a.dueno, dueno);
        if se_quita {
            assert_eq!(ahora, "{\n  \"model\": \"x\",\n  \"env\": {}\n}\n");
            assert_eq!(a.decision.texto(), "borra");
        } else {
            assert_eq!(ahora, texto);
            assert_eq!(a.decision.texto(), "conserva");
            assert_eq!(h.informe.resultado, Resultado::SinCambios);
        }
    }
}

/// C4: los hooks nunca se editan; uno que nombra `tc2` se lista como aviso.
#[test]
fn hooks_nunca_se_editan_y_se_avisa() {
    let f = Fixture::nueva("desinstalar-hooks");
    let home = f.raiz.join("home");
    let texto = "{\"hooks\":{\"Stop\":[{\"hooks\":[{\"type\":\"command\",\"command\":\"tc2 guardar x\"},{\"type\":\"command\",\"command\":\"otro\"}]}]},\"statusLine\":{\"type\":\"command\",\"command\":\"tc2 estado --stdin\"}}";
    escribir(&home.join(".claude"), "settings.json", texto);
    let e = entorno(Sistema::Linux, &home, home.join("x").join("tc2"));
    let h = correr(
        &e,
        &f.raiz,
        &f.git(),
        &MaquinaEnMemoria::default(),
        &["--maquina"],
    );
    let ahora = std::fs::read_to_string(home.join(".claude").join("settings.json")).unwrap();
    // Solo se va la status line; la clave hooks queda igual.
    assert_eq!(
        ahora,
        "{\"hooks\":{\"Stop\":[{\"hooks\":[{\"type\":\"command\",\"command\":\"tc2 guardar x\"},{\"type\":\"command\",\"command\":\"otro\"}]}]}}"
    );
    assert_eq!(decisiones(&h, "hook"), vec!["conserva"]);
    let s = h.salida_en(false, Idioma::Es);
    assert!(
        s.stdout
            .contains("clase=hook dueno=ajeno decision=conserva motivo=nombra-tc2")
    );
    assert!(s.stderr.contains("nunca edita los hooks"), "{}", s.stderr);
    // Sin status line: el settings con hooks ajenos queda identico.
    let solo_hooks = "{\"hooks\":{\"Stop\":[{\"hooks\":[{\"type\":\"command\",\"command\":\"/x/tc2.exe guardar\"}]}]}}\n";
    escribir(&home.join(".claude"), "settings.json", solo_hooks);
    let h = correr(
        &e,
        &f.raiz,
        &f.git(),
        &MaquinaEnMemoria::default(),
        &["--maquina"],
    );
    assert_eq!(
        std::fs::read_to_string(home.join(".claude").join("settings.json")).unwrap(),
        solo_hooks
    );
    assert_eq!(h.informe.resultado, Resultado::SinCambios);
}

/// C5: quita el worktree de estado y el de tarea (limpio y publicado), sus
/// registros, `.taskcodetwo/`, `.git/taskcodetwo/` y las lineas propias de
/// `exclude`; el remoto queda igual.
#[test]
fn proyecto_quita_worktrees_comun_y_exclude_sin_tocar_el_remoto() {
    let f = Fixture::nueva("desinstalar-proyecto");
    let (bare, clon) = repo_con_estado(&f);
    let remoto_antes = f.crudo(&clon, &["ls-remote", "origin"]);
    let e = entorno(Sistema::Linux, &f.raiz.join("home"), f.raiz.join("tc2"));
    let h = correr(
        &e,
        &clon,
        &f.git(),
        &MaquinaEnMemoria::default(),
        &["--proyecto"],
    );
    assert_eq!(h.informe.resultado, Resultado::Ok, "{}", stdout(&h));
    assert_eq!(decisiones(&h, "worktree-estado"), vec!["borra"]);
    assert_eq!(decisiones(&h, "worktree-tarea"), vec!["borra"]);
    let wts = f.crudo(&clon, &["worktree", "list", "--porcelain"]);
    assert_eq!(
        wts.lines().filter(|l| l.starts_with("worktree ")).count(),
        1,
        "{wts}"
    );
    assert!(!clon.join("tareas").exists());
    assert!(!clon.join(".taskcodetwo").exists());
    assert!(!clon.join(".git").join("taskcodetwo").exists());
    let wt_reg = clon.join(".git").join("worktrees");
    assert!(
        std::fs::read_dir(&wt_reg)
            .map(|mut d| d.next().is_none())
            .unwrap_or(true)
    );
    let ex = std::fs::read_to_string(clon.join(".git").join("info").join("exclude")).unwrap();
    assert!(
        !ex.contains("/.taskcodetwo/worktrees/") && !ex.contains("/tareas/"),
        "{ex}"
    );
    assert!(ex.ends_with("/mio/\n"), "{ex}");
    assert_eq!(f.crudo(&clon, &["ls-remote", "origin"]), remoto_antes);
    assert!(!f.crudo(&bare, &["ls-remote", "."]).is_empty());
    // La rama de estado local y la de la tarea se conservan.
    f.crudo(
        &clon,
        &["rev-parse", "--verify", "refs/heads/taskcodetwo/tareas"],
    );
    f.crudo(
        &clon,
        &["rev-parse", "--verify", "refs/heads/feature/task-001-x"],
    );
}

/// C5: un worktree de tarea con commits sin publicar (o cambios) deja todo
/// `rechazada` sin tocar nada.
#[test]
fn worktree_de_tarea_sin_publicar_rechaza_sin_tocar_nada() {
    let f = Fixture::nueva("desinstalar-rechazada");
    let (bare, clon) = repo_con_estado(&f);
    let wt = clon.join(".taskcodetwo").join("worktrees").join("TASK-001");
    escribir(&wt, "nuevo.txt", "x\n");
    f.crudo(&wt, &["add", "nuevo.txt"]);
    f.crudo(&wt, &["commit", "-q", "-m", "sin publicar"]);
    let antes = [instantanea(&clon), instantanea(&bare)];
    let e = entorno(Sistema::Linux, &f.raiz.join("home"), f.raiz.join("tc2"));
    let h = correr(
        &e,
        &clon,
        &f.git(),
        &MaquinaEnMemoria::default(),
        &["--proyecto", "--config"],
    );
    let s = h.salida_en(false, Idioma::Es);
    assert_eq!(s.codigo, 3, "{}", s.stdout);
    assert!(
        s.stdout
            .lines()
            .last()
            .unwrap()
            .contains("motivo=sin-publicar")
    );
    assert_eq!([instantanea(&clon), instantanea(&bare)], antes);
    // Con cambios sin guardar, igual.
    escribir(&wt, "otro.txt", "y\n");
    f.crudo(&wt, &["push", "-q", "origin", "feature/task-001-x"]);
    let h = correr(
        &e,
        &clon,
        &f.git(),
        &MaquinaEnMemoria::default(),
        &["--proyecto"],
    );
    assert_eq!(h.informe.valor("motivo"), Some("cambios-sin-guardar"));
}

/// C6: la config de la 2.0 y `personas.yml` solo se borran con `--config`;
/// la de la 0.16 nunca.
#[test]
fn config_solo_con_la_opcion() {
    let f = Fixture::nueva("desinstalar-config");
    let (_bare, clon) = f.remoto_y_clon();
    escribir(&clon, ".taskcode/config.yml", "---\nformato: 2\n");
    escribir(&clon, ".taskcode/personas.yml", "---\npersonas: []\n");
    let e = entorno(Sistema::Linux, &f.raiz.join("home"), f.raiz.join("tc2"));
    let g = f.git();
    let m = MaquinaEnMemoria::default();
    let h = correr(&e, &clon, &g, &m, &["--proyecto"]);
    assert_eq!(decisiones(&h, "config"), vec!["conserva", "conserva"]);
    assert!(stdout(&h).contains("decision=conserva motivo=sin-config"));
    assert!(clon.join(".taskcode/config.yml").is_file());
    let h = correr(&e, &clon, &g, &m, &["--proyecto", "--config"]);
    assert_eq!(decisiones(&h, "config"), vec!["borra", "borra"]);
    assert!(!clon.join(".taskcode/config.yml").exists());
    assert!(!clon.join(".taskcode/personas.yml").exists());
    // La de la 0.16 (sin formato 2) se queda aun con --config.
    escribir(&clon, ".taskcode/config.yml", "modo_flujo: automatico\n");
    let h = correr(&e, &clon, &g, &m, &["--proyecto", "--config"]);
    assert_eq!(h.acciones[0].dueno, "taskcode-016");
    assert_eq!(
        std::fs::read_to_string(clon.join(".taskcode/config.yml")).unwrap(),
        "modo_flujo: automatico\n"
    );
}

/// C7: la rama local de estado solo con `--borrar-tareas <nombre exacto>`;
/// la remota nunca: se imprime el `git push --delete` para la persona.
#[test]
fn rama_local_solo_con_borrar_tareas_y_la_remota_nunca() {
    let f = Fixture::nueva("desinstalar-rama");
    let (_bare, clon) = repo_con_estado(&f);
    let remoto_antes = f.crudo(&clon, &["ls-remote", "origin"]);
    let e = entorno(Sistema::Linux, &f.raiz.join("home"), f.raiz.join("tc2"));
    let g = f.git();
    let m = MaquinaEnMemoria::default();
    let h = correr(&e, &clon, &g, &m, &["--proyecto"]);
    assert_eq!(decisiones(&h, "rama-estado"), vec!["conserva"]);
    assert_eq!(decisiones(&h, "rama-remota"), vec!["conserva"]);
    f.crudo(
        &clon,
        &["rev-parse", "--verify", "refs/heads/taskcodetwo/tareas"],
    );
    // Otro nombre: uso, sin tocar nada.
    let s = tc2::ejecutar(["desinstalar", "--proyecto", "--borrar-tareas", "tareas"]);
    assert_eq!(s.codigo, 2);
    assert!(s.stdout.contains("motivo=rama-no-coincide"));
    let h = correr(
        &e,
        &clon,
        &g,
        &m,
        &["--proyecto", "--borrar-tareas", "taskcodetwo/tareas"],
    );
    assert_eq!(h.informe.resultado, Resultado::Ok, "{}", stdout(&h));
    assert_eq!(decisiones(&h, "rama-estado"), vec!["borra"]);
    let s = h.salida_en(false, Idioma::Es);
    assert!(s.stdout.contains(
        "clase=rama-remota dueno=taskcodetwo decision=persona orden=git%20push%20origin%20--delete%20taskcodetwo/tareas"
    ));
    assert!(
        s.stderr
            .contains("git push origin --delete taskcodetwo/tareas")
    );
    let local = f.crudo(&clon, &["for-each-ref", "refs/heads/taskcodetwo/"]);
    assert!(local.is_empty(), "{local}");
    assert_eq!(f.crudo(&clon, &["ls-remote", "origin"]), remoto_antes);
}

/// C9: en modo plugin o gestor el binario no se borra y la linea lleva la
/// orden exacta que lo quita.
#[test]
fn plugin_y_gestor_no_borran_el_binario() {
    let f = Fixture::nueva("desinstalar-modos");
    let home = f.raiz.join("home");
    let cli = home.join(".cache").join("taskcodetwo").join("cli");
    escribir(&cli.join("2.0.0"), "tc2", "del plugin");
    escribir(&cli.join("1.9.0"), "tc2", "vieja");
    escribir(
        &home.join(".claude").join("plugins"),
        "installed_plugins.json",
        "{\"version\":2,\"plugins\":{\"taskcodetwo@taskcodetwo\":[{\"scope\":\"user\"}]}}",
    );
    let exe = cli.join("2.0.0").join("tc2");
    let e = entorno(Sistema::Linux, &home, exe.clone());
    let h = correr(
        &e,
        &f.raiz,
        &f.git(),
        &MaquinaEnMemoria::default(),
        &["--maquina"],
    );
    let s = h.salida_en(false, Idioma::Es);
    assert!(exe.is_file(), "el binario del plugin no se borra");
    assert!(
        !cli.join("1.9.0").exists(),
        "las otras versiones de la cache si"
    );
    assert!(s.stdout.contains(
        "clase=binario dueno=plugin decision=persona orden=claude%20plugin%20uninstall%20taskcodetwo@taskcodetwo%20--scope%20user"
    ), "{}", s.stdout);
    assert!(s.stdout.contains("clase=cache-cli dueno=taskcodetwo decision=persona motivo=en-uso-por-el-plugin orden=rm%20-rf"));
    assert!(
        s.stderr
            .contains("claude plugin uninstall taskcodetwo@taskcodetwo --scope user")
    );
    // Gestor: cargo.
    let exe = home.join(".cargo").join("bin").join("tc2");
    escribir(exe.parent().unwrap(), "tc2", "de cargo");
    let e = entorno(Sistema::Linux, &home, exe.clone());
    let h = correr(
        &e,
        &f.raiz,
        &f.git(),
        &MaquinaEnMemoria::default(),
        &["--maquina"],
    );
    assert!(exe.is_file());
    assert!(stdout(&h).contains(
        "clase=binario dueno=gestor decision=persona motivo=cargo orden=cargo%20uninstall%20tc2"
    ));
}

/// C10: con la huella de la 0.16 en el HOME falso y en el repo, todo eso
/// queda identico byte a byte, aun con `--config` y `--borrar-tareas`.
#[test]
fn huella_de_la_016_intacta() {
    let f = Fixture::nueva("desinstalar-016");
    let (_bare, clon) = f.remoto_y_clon();
    escribir(&clon, ".taskcode/config.yml", "modo_flujo: automatico\n");
    escribir(&clon, ".taskcode/doctor-deploy.yml", "desactivados: []\n");
    escribir(&clon, "tareas/ROADMAP.md", "# Roadmap\n");
    f.crudo(&clon, &["add", "."]);
    f.crudo(&clon, &["commit", "-q", "-m", "0.16"]);
    let c016 = clon.join(".git").join("taskcode");
    escribir(&c016, "puertas.log", "verde\n");
    escribir(&c016, "gitflow/gitflow.log", "start\n");
    escribir(
        &clon.join(".git").join("taskcodetwo"),
        "identidad.yml",
        "---\n",
    );
    let home = f.raiz.join("home");
    let claude = home.join(".claude");
    escribir(
        &claude,
        "settings.json",
        "{\"statusLine\":{\"type\":\"command\",\"command\":\"taskctl estado --stdin\"}}",
    );
    escribir(
        &claude.join("plugins"),
        "installed_plugins.json",
        "{\"version\":2,\"plugins\":{\"taskcode-plugin@taskcode-marketplace\":[{\"scope\":\"user\"}]}}",
    );
    escribir(
        &claude.join("plugins/cache/taskcode-marketplace/taskcode-plugin/0.16.1/bin"),
        "taskctl",
        "0.16",
    );
    let (exe, _) = instalacion_unix(&home);
    let huella = || {
        [
            instantanea(&c016),
            instantanea(&clon.join(".taskcode")),
            instantanea(&clon.join("tareas")),
            instantanea(&claude),
        ]
    };
    let antes = huella();
    let e = entorno(Sistema::Linux, &home, exe);
    let h = correr(
        &e,
        &clon,
        &f.git(),
        &MaquinaEnMemoria::default(),
        &[
            "--proyecto",
            "--maquina",
            "--config",
            "--borrar-tareas",
            "taskcodetwo/tareas",
        ],
    );
    assert_eq!(h.informe.resultado, Resultado::Ok, "{}", stdout(&h));
    assert_eq!(huella(), antes);
    assert!(!clon.join(".git").join("taskcodetwo").exists());
    assert!(
        stdout(&h).contains("dueno=taskcode-016 decision=conserva motivo=desinstalacion-f9-10")
    );
}

/// C11: una segunda ejecucion responde `sin-cambios`.
#[test]
fn segunda_ejecucion_sin_cambios() {
    let f = Fixture::nueva("desinstalar-dos-veces");
    let (_bare, clon) = repo_con_estado(&f);
    let home = f.raiz.join("home");
    let (exe, _) = instalacion_unix(&home);
    escribir(
        &home.join(".claude"),
        "settings.json",
        "{\"statusLine\":{\"type\":\"command\",\"command\":\"tc2 estado --stdin\"}}",
    );
    let e = entorno(Sistema::Linux, &home, exe);
    let g = f.git();
    let m = MaquinaEnMemoria::default();
    let todo = [
        "--proyecto",
        "--maquina",
        "--config",
        "--borrar-tareas",
        "taskcodetwo/tareas",
    ];
    let h = correr(&e, &clon, &g, &m, &todo);
    assert_eq!(h.informe.resultado, Resultado::Ok, "{}", stdout(&h));
    let h = correr(&e, &clon, &g, &m, &todo);
    let s = h.salida_en(false, Idioma::Es);
    assert_eq!(s.codigo, 0);
    assert!(
        s.stdout
            .lines()
            .last()
            .unwrap()
            .starts_with("desinstalar sin-cambios "),
        "{}",
        s.stdout
    );
    // Y en Windows (maquina en memoria), tras el autoborrado.
    let home = f.raiz.join("home-w");
    let exe = instalacion_windows(&home);
    let e = entorno(Sistema::Windows, &home, exe.clone());
    let m = MaquinaEnMemoria {
        path: Some(format!("{}", exe.parent().unwrap().display())).into(),
        ..MaquinaEnMemoria::default()
    };
    assert_eq!(
        correr(&e, &clon, &g, &m, &["--maquina"]).informe.resultado,
        Resultado::Ok
    );
    // Lo que haria el auxiliar.
    std::fs::remove_dir_all(home.join("AppData/Local/taskcodetwo")).unwrap();
    assert_eq!(
        correr(&e, &clon, &g, &m, &["--maquina"]).informe.resultado,
        Resultado::SinCambios
    );
}

/// Sin ambito, `uso`; y la orden esta en la linea de ordenes.
#[test]
fn sin_ambito_es_uso() {
    let s = tc2::ejecutar(["desinstalar"]);
    assert_eq!(s.codigo, 2);
    assert_eq!(s.stdout, "desinstalar uso motivo=sin-ambito\n");
    let s = tc2::ejecutar_en(
        Idioma::En,
        ["desinstalar", "--json", "--maquina", "--config"],
    );
    assert_eq!(s.codigo, 2);
    assert!(s.stdout.contains("\"motivo\":\"opcion-de-proyecto\""));
}
