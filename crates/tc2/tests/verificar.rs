//! F5-6 (TASK-046): `tc2 confiar` y `tc2 verificar --perfil <p> [--hook]`
//! contra repos git temporales reales, sin mocks, con el binario compilado y
//! un `LOCALAPPDATA`/`XDG_CACHE_HOME` propio de cada test (el almacen de
//! confianza nunca es el de la maquina).
//!
//! Las ordenes del proyecto son scripts triviales (nunca `cargo`): `cmd` y
//! `.bat` en Windows, `sh` y `.sh` en el resto. Escriben en el fichero de
//! `TESTIGO` (fuera del repo) el perfil que reciben: si existe, se ejecuto
//! algo del repo.
//!
//! El test pesado (P1, presupuesto de 1 s) esta ignorado; se lanza a mano:
//!
//! ```text
//! cargo test -p tc2 --release --test verificar -- --ignored --nocapture
//! ```

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant, SystemTime};

use comun::{Fixture, escribir};

const CLAVE: &str = "verificacion.comando";

/// Un repo de juguete con su carpeta de datos propia.
struct Caso {
    fx: Fixture,
    repo: PathBuf,
    datos: PathBuf,
    testigo: PathBuf,
}

/// Lo que hace el script `paso` que llama el principal: salir bien, en rojo
/// con un codigo, o tardar mas que cualquier timeout de los tests.
#[derive(Clone, Copy)]
enum Paso {
    Verde,
    Rojo(u8),
    Lento,
    /// Rojo (codigo 6) tras escribir `LINEA-1` .. `LINEA-30`.
    Lineas,
}

fn cuerpo_paso(p: Paso) -> String {
    if cfg!(windows) {
        match p {
            Paso::Verde => "@exit /b 0\r\n".into(),
            Paso::Rojo(c) => format!("@echo FALLO-DEL-PASO\r\n@exit /b {c}\r\n"),
            Paso::Lento => "@ping -n 8 127.0.0.1 > nul\r\n@exit /b 0\r\n".into(),
            Paso::Lineas => "@for /L %%i in (1,1,30) do @echo LINEA-%%i\r\n@exit /b 6\r\n".into(),
        }
    } else {
        match p {
            Paso::Verde => "true\n".into(),
            Paso::Rojo(c) => format!("echo FALLO-DEL-PASO\nexit {c}\n"),
            Paso::Lento => "sleep 7\n".into(),
            Paso::Lineas => "for i in $(seq 1 30); do echo LINEA-$i; done\nexit 6\n".into(),
        }
    }
}

/// Ruta del script `paso`, que solo se alcanza por `%~dp0` desde el
/// principal.
fn ruta_paso() -> &'static str {
    if cfg!(windows) {
        "scripts/lib/paso.bat"
    } else {
        "scripts/lib/paso.sh"
    }
}

/// Config del proyecto: la orden nombra solo el script principal.
fn config(extra_orden: &str, rapida: Option<u32>, completa: u32) -> String {
    config_con(extra_orden, rapida, completa, None, "'incluido/*.cfg'")
}

/// Como [`config`], con otra shell y otros globs de `confianza_incluye`.
fn config_con(
    extra_orden: &str,
    rapida: Option<u32>,
    completa: u32,
    shell: Option<&str>,
    incluye: &str,
) -> String {
    let (comando, por_defecto) = if cfg!(windows) {
        (format!(r"scripts\verificar.bat{extra_orden}"), "cmd")
    } else {
        (format!("sh scripts/verificar.sh{extra_orden}"), "sh")
    };
    let shell = shell.unwrap_or(por_defecto);
    let rapida = rapida
        .map(|t| format!("    rapida: {{timeout: {t}}}\n"))
        .unwrap_or_default();
    format!(
        "---\nformato: 2\nverificacion:\n  comando: '{comando}'\n  shell: '{shell}'\n  perfiles:\n{rapida}    completa: {{timeout: {completa}, runner: {{github: linux}}}}\nconfianza_incluye: [{incluye}]\n"
    )
}

impl Caso {
    fn nuevo(nombre: &str) -> Self {
        let fx = Fixture::nueva(nombre);
        let repo = fx.raiz.join("repo");
        let datos = fx.raiz.join("datos");
        let testigo = fx.raiz.join("testigo");
        fx.crudo(&fx.raiz, &["init", "-q", "-b", "develop", "repo"]);
        Self {
            fx,
            repo,
            datos,
            testigo,
        }
    }

    /// Escribe y commitea la config, el principal, el `paso` y el fichero de
    /// `confianza_incluye`.
    fn proyecto(&self, paso: Paso) {
        if cfg!(windows) {
            escribir(
                &self.repo,
                "scripts/verificar.bat",
                "@echo off\r\necho %TC2_PERFIL%> \"%TESTIGO%\"\r\ncall \"%~dp0lib\\paso.bat\"\r\nexit /b %ERRORLEVEL%\r\n",
            );
        } else {
            // En sh, `%~dp0` solo aparece en el comentario: el cierre lo
            // resuelve igual (la PoC A1 de F1-8 sigue tokens, no la shell).
            escribir(
                &self.repo,
                "scripts/verificar.sh",
                "echo \"$TC2_PERFIL\" > \"$TESTIGO\"\n# %~dp0lib/paso.sh\n. \"$(dirname \"$0\")/lib/paso.sh\"\n",
            );
        }
        escribir(&self.repo, ruta_paso(), &cuerpo_paso(paso));
        escribir(&self.repo, "incluido/a.cfg", "valor=1\n");
        escribir(
            &self.repo,
            ".taskcode/config.yml",
            &config("", Some(60), 600),
        );
        self.fx.crudo(&self.repo, &["add", "-A"]);
        self.fx
            .crudo(&self.repo, &["commit", "-q", "-m", "proyecto"]);
    }

    fn tc2(&self, dir: &Path, args: &[&str], stdin: Option<&str>) -> Output {
        let mut c = Command::new(env!("CARGO_BIN_EXE_tc2"));
        c.args(args)
            .current_dir(dir)
            .envs(self.fx.entorno())
            .env("TC2_IDIOMA", "es")
            .env("LOCALAPPDATA", &self.datos)
            .env("XDG_CACHE_HOME", &self.datos)
            .env("TESTIGO", &self.testigo)
            .stdin(if stdin.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut hijo = c.spawn().expect("lanzar tc2");
        if let Some(s) = stdin {
            hijo.stdin.take().unwrap().write_all(s.as_bytes()).unwrap();
        }
        hijo.wait_with_output().expect("esperar a tc2")
    }

    fn verificar(&self, perfil: &str) -> Output {
        self.tc2(&self.repo, &["verificar", "--perfil", perfil], None)
    }

    fn hook(&self, activo: bool) -> Output {
        let json = format!(
            "{{\"session_id\":\"abc\",\"cwd\":\"x\",\"hook_event_name\":\"Stop\",\"stop_hook_active\":{activo}}}"
        );
        self.tc2(
            &self.repo,
            &["verificar", "--perfil", "rapida", "--hook"],
            Some(&json),
        )
    }

    fn confiar(&self) {
        let o = self.tc2(&self.repo, &["confiar", CLAVE], None);
        assert_eq!(o.status.code(), Some(0), "{}", texto(&o));
        assert!(salida(&o).starts_with("confiar ok clave=verificacion.comando"));
    }

    /// Si se ejecuto el comando del proyecto (y lo deja listo para la
    /// siguiente comprobacion).
    fn se_ejecuto(&self) -> bool {
        let si = self.testigo.exists();
        let _ = std::fs::remove_file(&self.testigo);
        si
    }
}

fn salida(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn error(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn texto(o: &Output) -> String {
    format!(
        "codigo {:?}\nstdout: {}\nstderr: {}",
        o.status.code(),
        salida(o),
        error(o)
    )
}

/// Valor de una clave de la linea de contrato.
fn clave(o: &Output, k: &str) -> Option<String> {
    salida(o).split_whitespace().find_map(|kv| {
        kv.split_once('=')
            .filter(|(c, _)| *c == k)
            .map(|(_, v)| v.to_owned())
    })
}

/// Cambia el `mtime` sin tocar el contenido.
fn tocar(ruta: &Path, segundos: u64) {
    let f = std::fs::File::options().write(true).open(ruta).unwrap();
    f.set_modified(SystemTime::now() + Duration::from_secs(segundos))
        .unwrap();
}

/// P3 — criterio 2: sin confianza no se ejecuta nada del repo y el aviso
/// dice que hacer; tras `tc2 confiar`, si.
#[test]
fn sin_confianza_no_se_ejecuta_nada_y_tras_confiar_si() {
    let c = Caso::nuevo("verificar-p3");
    c.proyecto(Paso::Verde);
    let o = c.verificar("rapida");
    assert_eq!(o.status.code(), Some(3), "{}", texto(&o));
    assert_eq!(clave(&o, "motivo").as_deref(), Some("sin-confianza"));
    assert_eq!(clave(&o, "confianza").as_deref(), Some("nunca"));
    assert!(
        error(&o).contains("`tc2 confiar verificacion.comando`"),
        "{}",
        texto(&o)
    );
    assert!(!c.se_ejecuto(), "se ejecuto la orden sin confianza");

    c.confiar();
    let o = c.verificar("rapida");
    assert_eq!(o.status.code(), Some(0), "{}", texto(&o));
    assert_eq!(clave(&o, "resultado").as_deref(), Some("verde"));
    assert_eq!(clave(&o, "confianza").as_deref(), Some("confiado"));
    assert_eq!(
        std::fs::read_to_string(&c.testigo).unwrap().trim(),
        "rapida",
        "TC2_PERFIL"
    );
    assert!(c.se_ejecuto());

    // En rojo, codigo distinto de 0 y el codigo del comando en la linea.
    escribir(&c.repo, ruta_paso(), &cuerpo_paso(Paso::Rojo(3)));
    c.confiar();
    let o = c.verificar("rapida");
    assert_eq!(o.status.code(), Some(3), "{}", texto(&o));
    assert_eq!(clave(&o, "motivo").as_deref(), Some("rojo"));
    assert_eq!(clave(&o, "codigo").as_deref(), Some("3"));
}

/// P4 — criterio 2: con la orden confiada, cambiar un script del cierre
/// (alcanzado por `%~dp0`), un fichero de `confianza_incluye` o la orden da
/// `cambiado` y no se ejecuta; volver al contenido original da `confiado`.
#[test]
fn cambiar_cierre_globs_u_orden_da_cambiado_y_volver_da_confiado() {
    let c = Caso::nuevo("verificar-p4");
    c.proyecto(Paso::Verde);
    c.confiar();
    let config_original = config("", Some(60), 600);
    // Otra shell con la misma orden tambien cambia lo que se ejecuta (la
    // shell entra en la huella; no hace falta que este instalada: no se lanza).
    let otra_shell = if cfg!(windows) { "pwsh" } else { "bash" };
    let cambios: [(&str, String, String); 4] = [
        (
            ruta_paso(),
            cuerpo_paso(Paso::Verde),
            cuerpo_paso(Paso::Rojo(0)),
        ),
        ("incluido/a.cfg", "valor=1\n".into(), "valor=2\n".into()),
        (
            ".taskcode/config.yml",
            config_original.clone(),
            config(" --otra", Some(60), 600),
        ),
        (
            ".taskcode/config.yml",
            config_original.clone(),
            config_con("", Some(60), 600, Some(otra_shell), "'incluido/*.cfg'"),
        ),
    ];
    for (ruta, original, cambiado) in &cambios {
        escribir(&c.repo, ruta, cambiado);
        let o = c.verificar("rapida");
        assert_eq!(o.status.code(), Some(3), "{ruta}: {}", texto(&o));
        assert_eq!(
            clave(&o, "confianza").as_deref(),
            Some("cambiado"),
            "{ruta}: {}",
            texto(&o)
        );
        assert!(error(&o).contains("`tc2 confiar verificacion.comando`"));
        assert!(!c.se_ejecuto(), "{ruta}: se ejecuto con el cierre cambiado");

        escribir(&c.repo, ruta, original);
        let o = c.verificar("rapida");
        assert_eq!(o.status.code(), Some(0), "{ruta}: {}", texto(&o));
        assert_eq!(clave(&o, "confianza").as_deref(), Some("confiado"));
        assert!(c.se_ejecuto());
    }
}

/// P5 — criterio 2: un fichero del repo que diga «confiado» no cuenta; un
/// worktree comparte la confianza y otro clon no; el almacen esta en la
/// carpeta de datos y no deja nada en el arbol.
#[test]
fn la_confianza_es_del_repo_comun_y_vive_fuera_del_arbol() {
    let c = Caso::nuevo("verificar-p5");
    c.proyecto(Paso::Verde);
    escribir(
        &c.repo,
        ".taskcode/confianza.json",
        "{\"version\":1,\"claves\":{\"verificacion.comando\":\"confiado\"}}\n",
    );
    c.fx.crudo(&c.repo, &["add", "-A"]);
    c.fx.crudo(&c.repo, &["commit", "-q", "-m", "falsa confianza"]);
    let o = c.verificar("rapida");
    assert_eq!(clave(&o, "confianza").as_deref(), Some("nunca"));
    assert!(!c.se_ejecuto());

    c.confiar();
    assert_eq!(c.fx.crudo(&c.repo, &["status", "--porcelain"]), "");
    let almacen = c.datos.join("taskcodetwo").join("confianza");
    let ficheros: Vec<_> = std::fs::read_dir(&almacen)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(ficheros.len(), 1, "{ficheros:?}");
    assert!(ficheros[0].ends_with(".json"), "{ficheros:?}");

    let wt = c.fx.raiz.join("wt");
    c.fx.crudo(
        &c.repo,
        &["worktree", "add", "-q", "-b", "otra", wt.to_str().unwrap()],
    );
    let o = c.tc2(&wt, &["verificar", "--perfil", "rapida"], None);
    assert_eq!(o.status.code(), Some(0), "worktree: {}", texto(&o));
    assert_eq!(clave(&o, "confianza").as_deref(), Some("confiado"));
    assert!(c.se_ejecuto());

    c.fx.crudo(&c.fx.raiz, &["clone", "-q", "repo", "clon2"]);
    let o = c.tc2(
        &c.fx.raiz.join("clon2"),
        &["verificar", "--perfil", "rapida"],
        None,
    );
    assert_eq!(o.status.code(), Some(3), "otro clon: {}", texto(&o));
    assert_eq!(clave(&o, "confianza").as_deref(), Some("nunca"));
    assert!(!c.se_ejecuto(), "otro clon heredo la confianza");
}

/// P2 — criterio 1 (D2a): en caliente y sin cambios no se hashea nada; un
/// script tocado (mismo contenido, otro `mtime`) se rehashea entero y sigue
/// confiado.
#[test]
fn en_caliente_no_se_rehashea_y_un_script_tocado_si() {
    let c = Caso::nuevo("verificar-p2");
    c.proyecto(Paso::Verde);
    c.confiar();
    let o = c.verificar("rapida");
    assert_eq!(o.status.code(), Some(0), "{}", texto(&o));
    assert_eq!(
        clave(&o, "hasheados").as_deref(),
        Some("0"),
        "{}",
        texto(&o)
    );

    let paso = c.repo.join(ruta_paso());
    tocar(&paso, 10);
    let tamano = std::fs::metadata(&paso).unwrap().len().to_string();
    let o = c.verificar("rapida");
    assert_eq!(o.status.code(), Some(0), "{}", texto(&o));
    assert_eq!(clave(&o, "confianza").as_deref(), Some("confiado"));
    assert_eq!(clave(&o, "hasheados"), Some(tamano), "{}", texto(&o));

    let o = c.verificar("rapida");
    assert_eq!(
        clave(&o, "hasheados").as_deref(),
        Some("0"),
        "{}",
        texto(&o)
    );
}

/// P6 — criterio 3: `--hook` sale con 0 sin ejecutar con
/// `stop_hook_active: true`; sin confianza o sin `rapida`, 0 con aviso y sin
/// ejecutar; con la rapida en rojo, 2 con un resumen corto.
#[test]
fn hook_no_bloquea_sin_confianza_y_bloquea_en_rojo() {
    let c = Caso::nuevo("verificar-p6");
    c.proyecto(Paso::Rojo(4));

    let o = c.hook(false);
    assert_eq!(o.status.code(), Some(0), "sin confianza: {}", texto(&o));
    assert!(
        error(&o).contains("`tc2 confiar verificacion.comando`"),
        "{}",
        texto(&o)
    );
    assert!(!c.se_ejecuto());

    c.confiar();
    let o = c.hook(true);
    assert_eq!(o.status.code(), Some(0), "activo: {}", texto(&o));
    assert_eq!(clave(&o, "motivo").as_deref(), Some("stop-hook-activo"));
    assert!(!c.se_ejecuto(), "con stop_hook_active se ejecuto");

    let o = c.hook(false);
    assert_eq!(o.status.code(), Some(2), "rojo: {}", texto(&o));
    assert!(error(&o).contains("FALLO-DEL-PASO"), "{}", texto(&o));
    assert!(error(&o).lines().count() <= 16, "{}", texto(&o));
    assert!(c.se_ejecuto());

    // Sin perfil `rapida`: 0 con aviso, sin ejecutar.
    escribir(&c.repo, ".taskcode/config.yml", &config("", None, 600));
    let o = c.hook(false);
    assert_eq!(o.status.code(), Some(0), "sin rapida: {}", texto(&o));
    assert_eq!(clave(&o, "motivo").as_deref(), Some("sin-perfil"));
    assert!(error(&o).contains("rapida"), "{}", texto(&o));
    assert!(!c.se_ejecuto());

    // Config invalida (la de la 0.16, sin `formato: 2`): 0 con aviso.
    escribir(&c.repo, ".taskcode/config.yml", "modo_flujo: automatico\n");
    let o = c.hook(false);
    assert_eq!(o.status.code(), Some(0), "config 0.16: {}", texto(&o));
    assert!(!error(&o).is_empty());
    assert!(!c.se_ejecuto());

    // YAML mal formado: el diagnostico del parser trae varias lineas; el
    // aviso del hook es una sola.
    escribir(
        &c.repo,
        ".taskcode/config.yml",
        "---\nformato: 2\nverificacion: [\n  - a: {\n",
    );
    let o = c.hook(false);
    assert_eq!(o.status.code(), Some(0), "yaml: {}", texto(&o));
    assert_eq!(
        clave(&o, "motivo").as_deref(),
        Some("yaml"),
        "{}",
        texto(&o)
    );
    assert_eq!(error(&o).lines().count(), 1, "{}", texto(&o));
    assert!(!c.se_ejecuto());
}

/// Ronda 1, IMP-1 (calidad): con `--hook`, una entrada de settings.json mal
/// escrita (perfil inexistente, opcion desconocida) no bloquea la parada:
/// sale con 0 y una linea que dice como arreglarla; y `stop_hook_active`
/// corta antes de validar los argumentos.
#[test]
fn hook_mal_escrito_no_bloquea_la_parada() {
    let c = Caso::nuevo("verificar-hook-mal");
    c.proyecto(Paso::Rojo(4));
    c.confiar();
    let falso = "{\"hook_event_name\":\"Stop\",\"stop_hook_active\":false}";
    let activo = "{\"hook_event_name\":\"Stop\",\"stop_hook_active\":true}";
    for (args, stdin) in [
        (&["verificar", "--perfil", "rapid", "--hook"][..], falso),
        (
            &["verificar", "--perfil", "rapida", "--hook", "--foo"][..],
            falso,
        ),
        (&["verificar", "--hook"][..], falso),
        (&["verificar", "--perfil", "rapid", "--hook"][..], activo),
    ] {
        let o = c.tc2(&c.repo, args, Some(stdin));
        assert_eq!(o.status.code(), Some(0), "{args:?} {stdin}: {}", texto(&o));
        assert!(error(&o).lines().count() <= 1, "{}", texto(&o));
        if stdin == falso {
            assert!(error(&o).contains("settings.json"), "{}", texto(&o));
        } else {
            assert_eq!(clave(&o, "motivo").as_deref(), Some("stop-hook-activo"));
        }
        assert!(!c.se_ejecuto(), "{args:?}: se ejecuto");
    }
    // Sin `--hook` el error de uso sigue siendo uso (2).
    let o = c.tc2(&c.repo, &["verificar", "--perfil", "rapid"], None);
    assert_eq!(o.status.code(), Some(2), "{}", texto(&o));
}

/// Ronda 1, MEN-1 (calidad): con `--hook` y un perfil que no es `rapida`,
/// un timeout agotado es rojo y bloquea (2); el resumen trae solo las
/// ultimas 15 lineas de la salida.
#[test]
fn hook_bloquea_con_timeout_fuera_de_rapida_y_resume_15_lineas() {
    let c = Caso::nuevo("verificar-hook-cola");
    c.proyecto(Paso::Lineas);
    c.confiar();
    let o = c.hook(false);
    assert_eq!(o.status.code(), Some(2), "{}", texto(&o));
    let err = error(&o);
    let lineas: Vec<&str> = err.lines().map(str::trim).collect();
    assert!(lineas.contains(&"LINEA-30"), "{err}");
    assert!(lineas.contains(&"LINEA-16"), "{err}");
    assert!(!lineas.contains(&"LINEA-15"), "{err}");
    assert!(!lineas.contains(&"LINEA-1"), "{err}");
    let _ = c.se_ejecuto();

    escribir(&c.repo, ruta_paso(), &cuerpo_paso(Paso::Lento));
    escribir(&c.repo, ".taskcode/config.yml", &config("", Some(60), 1));
    c.confiar();
    let o = c.tc2(
        &c.repo,
        &["verificar", "--perfil", "completa", "--hook"],
        Some("{\"stop_hook_active\":false}"),
    );
    assert_eq!(o.status.code(), Some(2), "completa: {}", texto(&o));
    assert_eq!(clave(&o, "motivo").as_deref(), Some("timeout"));
}

/// Ronda 1, IMP-1 (rust): el cierre sigue scripts con espacios en el nombre
/// (entre comillas) y con caracteres no ASCII: cambiarlos da `cambiado`.
#[test]
fn el_cierre_sigue_nombres_con_espacios_y_no_ascii() {
    let c = Caso::nuevo("verificar-nombres");
    c.proyecto(Paso::Verde);
    let (espacio, acento) = if cfg!(windows) {
        escribir(
            &c.repo,
            "scripts/verificar.bat",
            "@echo off\r\necho %TC2_PERFIL%> \"%TESTIGO%\"\r\ncall \"%~dp0lib\\mi paso.bat\"\r\nrem %~dp0lib\\verificación.bat\r\nexit /b %ERRORLEVEL%\r\n",
        );
        ("scripts/lib/mi paso.bat", "scripts/lib/verificación.bat")
    } else {
        escribir(
            &c.repo,
            "scripts/verificar.sh",
            "echo \"$TC2_PERFIL\" > \"$TESTIGO\"\n# \"%~dp0lib/mi paso.sh\"\n. \"$(dirname \"$0\")/lib/mi paso.sh\"\n# %~dp0lib/verificación.sh\n",
        );
        ("scripts/lib/mi paso.sh", "scripts/lib/verificación.sh")
    };
    escribir(&c.repo, espacio, &cuerpo_paso(Paso::Verde));
    escribir(&c.repo, acento, &cuerpo_paso(Paso::Verde));
    c.fx.crudo(&c.repo, &["add", "-A"]);
    c.fx.crudo(&c.repo, &["commit", "-q", "-m", "nombres raros"]);
    c.confiar();
    for ruta in [espacio, acento] {
        let o = c.verificar("rapida");
        assert_eq!(
            clave(&o, "confianza").as_deref(),
            Some("confiado"),
            "{}",
            texto(&o)
        );
        assert!(c.se_ejecuto());
        escribir(&c.repo, ruta, &cuerpo_paso(Paso::Rojo(0)));
        let o = c.verificar("rapida");
        assert_eq!(
            clave(&o, "confianza").as_deref(),
            Some("cambiado"),
            "{ruta}: {}",
            texto(&o)
        );
        assert!(!c.se_ejecuto(), "{ruta}: se ejecuto cambiado");
        escribir(&c.repo, ruta, &cuerpo_paso(Paso::Verde));
    }
}

/// Ronda 2, CRITICO-1: `linea` nombra `raro` (nombre con espacio) de forma
/// que las comillas no se emparejan. Va detras de la salida del script
/// principal (no se ejecuta en ninguna plataforma): lo que se prueba es el
/// cierre, que es el mismo para cualquier shell. `raro` tiene que entrar en
/// la confianza: cambiarlo da `cambiado` y no se ejecuta nada.
fn raro_entra_en_el_cierre(nombre: &str, linea: &str, raro: &str) {
    let c = Caso::nuevo(nombre);
    c.proyecto(Paso::Verde);
    let principal = if cfg!(windows) {
        "scripts/verificar.bat"
    } else {
        "scripts/verificar.sh"
    };
    let mut contenido = std::fs::read_to_string(c.repo.join(principal)).unwrap();
    if !cfg!(windows) {
        contenido.push_str("exit 0\n");
    }
    contenido.push_str(linea);
    contenido.push('\n');
    escribir(&c.repo, principal, &contenido);
    c.fx.crudo(&c.repo, &["add", "-A"]);
    c.fx.crudo(&c.repo, &["commit", "-q", "-m", "linea con el nombre"]);
    // Confiado antes de que exista `raro`, con la cache caliente: versionarlo
    // despues cambia el cierre aunque el principal no haya cambiado.
    c.confiar();
    let o = c.verificar("rapida");
    assert_eq!(
        clave(&o, "hasheados").as_deref(),
        Some("0"),
        "{}",
        texto(&o)
    );
    let _ = c.se_ejecuto();
    escribir(&c.repo, raro, "echo bueno\n");
    c.fx.crudo(&c.repo, &["add", "-A"]);
    c.fx.crudo(&c.repo, &["commit", "-q", "-m", "nombre raro"]);
    let o = c.verificar("rapida");
    assert_eq!(
        clave(&o, "confianza").as_deref(),
        Some("cambiado"),
        "{}",
        texto(&o)
    );
    assert!(!c.se_ejecuto(), "se ejecuto con {raro} nuevo sin confiar");
    let o = c.tc2(&c.repo, &["confiar", CLAVE], None);
    assert_eq!(o.status.code(), Some(0), "{}", texto(&o));
    // principal, paso, `incluido/a.cfg` (glob) y el raro.
    assert_eq!(clave(&o, "ficheros").as_deref(), Some("4"), "{}", texto(&o));
    let o = c.verificar("rapida");
    assert_eq!(
        clave(&o, "confianza").as_deref(),
        Some("confiado"),
        "{}",
        texto(&o)
    );
    assert!(c.se_ejecuto());
    escribir(&c.repo, raro, "echo malo\n");
    let o = c.verificar("rapida");
    assert_eq!(o.status.code(), Some(3), "{}", texto(&o));
    assert_eq!(
        clave(&o, "confianza").as_deref(),
        Some("cambiado"),
        "{}",
        texto(&o)
    );
    assert!(!c.se_ejecuto(), "se ejecuto con {raro} cambiado");
}

/// Ronda 2, CRITICO-1 (cmd): un apostrofo suelto antes de una ruta entre
/// comillas con espacios en la misma linea.
#[test]
fn apostrofo_suelto_no_deja_fuera_un_script_con_espacios() {
    raro_entra_en_el_cierre(
        "verificar-apostrofo",
        r#"echo Don't panic & call "%~dp0lib\mi paso.bat""#,
        "scripts/lib/mi paso.bat",
    );
}

/// Ronda 2, CRITICO-1 (sh): comillas anidadas alrededor de una ruta con
/// espacios.
#[test]
fn comillas_anidadas_no_dejan_fuera_un_script_con_espacios() {
    raro_entra_en_el_cierre(
        "verificar-anidadas",
        r#". "$(dirname "$0")/lib/mi paso.sh""#,
        "scripts/lib/mi paso.sh",
    );
}

/// Ronda 1, IMP-2 (rust): `./` se normaliza en `confianza_incluye`, un glob
/// invalido falla cerrado (config rechazada en `confiar` y en `verificar`)
/// y un glob que no casa con nada se avisa en `tc2 confiar`.
#[test]
fn globs_de_confianza_incluye_normalizados_invalidos_y_vacios() {
    let c = Caso::nuevo("verificar-globs");
    c.proyecto(Paso::Verde);
    let mut globs = vec!["'./incluido/*.cfg'"];
    if cfg!(any(windows, target_os = "macos")) {
        globs.push("'INCLUIDO/*.cfg'");
    }
    for g in globs {
        escribir(
            &c.repo,
            ".taskcode/config.yml",
            &config_con("", Some(60), 600, None, g),
        );
        c.confiar();
        escribir(&c.repo, "incluido/a.cfg", "valor=2\n");
        let o = c.verificar("rapida");
        assert_eq!(
            clave(&o, "confianza").as_deref(),
            Some("cambiado"),
            "{g}: {}",
            texto(&o)
        );
        assert!(!c.se_ejecuto(), "{g}: se ejecuto");
        escribir(&c.repo, "incluido/a.cfg", "valor=1\n");
    }

    escribir(
        &c.repo,
        ".taskcode/config.yml",
        &config_con("", Some(60), 600, None, "'incluido/['"),
    );
    let o = c.tc2(&c.repo, &["confiar", CLAVE], None);
    assert_eq!(o.status.code(), Some(3), "confiar: {}", texto(&o));
    assert!(error(&o).contains("incluido/["), "{}", texto(&o));
    let o = c.verificar("rapida");
    assert_eq!(o.status.code(), Some(3), "verificar: {}", texto(&o));
    assert!(!c.se_ejecuto());

    escribir(
        &c.repo,
        ".taskcode/config.yml",
        &config_con("", Some(60), 600, None, "'incluido/*.cfg', 'nada/*.cfg'"),
    );
    let o = c.tc2(&c.repo, &["confiar", CLAVE], None);
    assert_eq!(o.status.code(), Some(0), "{}", texto(&o));
    assert_eq!(
        clave(&o, "globs-sin-ficheros").as_deref(),
        Some("nada/*.cfg")
    );
    assert!(error(&o).contains("nada/*.cfg"), "{}", texto(&o));
}

/// Ronda 1, MENOR-1 (rust): en Windows y macOS un script nombrado con otras
/// mayusculas (`%~dp0LIB\PASO.BAT`) es el mismo fichero y entra en el cierre.
#[cfg(any(windows, target_os = "macos"))]
#[test]
fn el_cierre_no_distingue_mayusculas_en_windows_y_macos() {
    let c = Caso::nuevo("verificar-mayusculas");
    c.proyecto(Paso::Verde);
    if cfg!(windows) {
        escribir(
            &c.repo,
            "scripts/verificar.bat",
            "@echo off\r\necho %TC2_PERFIL%> \"%TESTIGO%\"\r\ncall \"%~dp0LIB\\PASO.BAT\"\r\nexit /b %ERRORLEVEL%\r\n",
        );
    } else {
        escribir(
            &c.repo,
            "scripts/verificar.sh",
            "echo \"$TC2_PERFIL\" > \"$TESTIGO\"\n# %~dp0LIB/PASO.SH\n. \"$(dirname \"$0\")/LIB/PASO.SH\"\n",
        );
    }
    c.fx.crudo(&c.repo, &["add", "-A"]);
    c.fx.crudo(&c.repo, &["commit", "-q", "-m", "mayusculas"]);
    c.confiar();
    escribir(&c.repo, ruta_paso(), &cuerpo_paso(Paso::Rojo(0)));
    let o = c.verificar("rapida");
    assert_eq!(
        clave(&o, "confianza").as_deref(),
        Some("cambiado"),
        "{}",
        texto(&o)
    );
    assert!(!c.se_ejecuto());
}

/// P7 — criterio 3: `--hook` y la orden sin hook toman la misma decision
/// con el mismo motivo; en `rapida` un timeout agotado no es rojo y en
/// `completa` si.
#[test]
fn hook_y_orden_deciden_igual_y_timeout_en_rapida_no_es_rojo() {
    let c = Caso::nuevo("verificar-p7");
    c.proyecto(Paso::Rojo(5));
    let decision = |o: &Output| {
        ["resultado", "motivo", "confianza", "codigo"]
            .map(|k| clave(o, k))
            .to_vec()
    };
    let comparar = |que: &str| {
        let sin = c.verificar("rapida");
        let _ = c.se_ejecuto();
        let con = c.hook(false);
        let _ = c.se_ejecuto();
        assert_eq!(
            decision(&sin),
            decision(&con),
            "{que}\n{}\n{}",
            texto(&sin),
            texto(&con)
        );
        assert_eq!(
            sin.stdout.split(|b| *b == b' ').next(),
            Some(&b"verificar"[..])
        );
    };
    comparar("nunca");
    c.confiar();
    comparar("confiado en rojo");
    escribir(&c.repo, ruta_paso(), &cuerpo_paso(Paso::Verde));
    c.confiar();
    comparar("confiado en verde");

    escribir(&c.repo, ruta_paso(), &cuerpo_paso(Paso::Lento));
    escribir(&c.repo, ".taskcode/config.yml", &config("", Some(1), 1));
    c.confiar();
    let t = Instant::now();
    let o = c.verificar("rapida");
    assert_eq!(o.status.code(), Some(0), "rapida: {}", texto(&o));
    assert_eq!(clave(&o, "resultado").as_deref(), Some("excedido"));
    assert!(
        t.elapsed() < Duration::from_secs(6),
        "no se corto: {:?}",
        t.elapsed()
    );
    let o = c.hook(false);
    assert_eq!(o.status.code(), Some(0), "hook: {}", texto(&o));
    assert_eq!(clave(&o, "resultado").as_deref(), Some("excedido"));
    let o = c.verificar("completa");
    assert_eq!(o.status.code(), Some(3), "completa: {}", texto(&o));
    assert_eq!(clave(&o, "motivo").as_deref(), Some("timeout"));
    assert_eq!(clave(&o, "resultado").as_deref(), Some("rojo"));
}

/// P1 — criterio 1 (D1a), pesado: un cierre como el de ToolsaWeb (51
/// scripts versionados, unos 1,2 MB) con una orden confiada que sale
/// enseguida. `tc2 verificar --perfil rapida` entero (lanzar `tc2`, leer la
/// config, cierre, huella, almacen y lanzar la shell) en 1 s o menos en frio
/// (todos los `mtime` cambiados: se rehashea todo) y en caliente; sin
/// confiar, 1 s o menos en total. Imprime las cifras con `--nocapture`.
#[test]
#[ignore = "pesado: mide tiempo; a mano con --release -- --ignored --nocapture"]
fn presupuesto_de_un_segundo_con_un_cierre_como_el_de_toolsaweb() {
    let c = Caso::nuevo("verificar-p1");
    let n = 50;
    let relleno: String = (0..410)
        .map(|i| format!("# linea de relleno {i:04} para llegar al tamano de ToolsaWeb\n"))
        .collect();
    let mut principal = if cfg!(windows) {
        String::from("@exit /b 0\r\n")
    } else {
        String::from("exit 0\n")
    };
    for i in 0..n {
        let rel = format!("scripts/s{i:02}.py");
        escribir(&c.repo, &rel, &relleno);
        principal.push_str(&format!("rem {}\n", rel.replace('/', "\\")));
    }
    let (ruta, comando, shell) = if cfg!(windows) {
        ("scripts/verificar.bat", r"scripts\verificar.bat", "cmd")
    } else {
        ("scripts/verificar.sh", "sh scripts/verificar.sh", "sh")
    };
    escribir(&c.repo, ruta, &principal);
    escribir(
        &c.repo,
        ".taskcode/config.yml",
        &format!(
            "---\nformato: 2\nverificacion:\n  comando: '{comando}'\n  shell: '{shell}'\n  perfiles:\n    rapida: {{timeout: 60}}\n    completa: {{timeout: 600, runner: {{github: linux}}}}\n"
        ),
    );
    c.fx.crudo(&c.repo, &["add", "-A"]);
    c.fx.crudo(&c.repo, &["commit", "-q", "-m", "cierre grande"]);

    let medir = |que: &str| {
        let t = Instant::now();
        let o = c.verificar("rapida");
        let e = t.elapsed();
        println!("P1 {que}: {} ms; {}", e.as_millis(), salida(&o).trim());
        (o, e)
    };
    let (o, sin) = medir("sin confiar");
    assert_eq!(clave(&o, "confianza").as_deref(), Some("nunca"));
    c.confiar();
    let o = c.tc2(&c.repo, &["confiar", CLAVE], None);
    println!("P1 confiar: {}", salida(&o).trim());
    assert_eq!(clave(&o, "ficheros").as_deref(), Some("51"));
    let bytes: u64 = clave(&o, "hasheados").unwrap().parse().unwrap();
    assert!(bytes > 1_000_000, "cierre de {bytes} bytes");
    for i in 0..n {
        tocar(&c.repo.join(format!("scripts/s{i:02}.py")), 30);
    }
    tocar(&c.repo.join(ruta), 30);
    let (o, frio) = medir("en frio");
    assert_eq!(clave(&o, "hasheados"), Some(bytes.to_string()));
    assert_eq!(o.status.code(), Some(0), "{}", texto(&o));
    let (o, caliente) = medir("en caliente");
    assert_eq!(clave(&o, "hasheados").as_deref(), Some("0"));
    let limite = Duration::from_secs(1);
    assert!(sin <= limite, "sin confiar: {sin:?}");
    assert!(frio <= limite, "en frio: {frio:?}");
    assert!(caliente <= limite, "en caliente: {caliente:?}");
}
