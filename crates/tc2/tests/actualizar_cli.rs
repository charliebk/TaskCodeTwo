//! TASK-027 (F2-12): `tc2 actualizar --cli` contra una release servida en
//! 127.0.0.1 (`TC2_BASE_URL`), sin red. Cada prueba copia el `tc2` compilado
//! a una carpeta temporal (la «instalacion vX») y lo lanza desde ahi, porque
//! la orden sustituye el ejecutable que se esta ejecutando.
//!
//! La «vY» es el ejemplo `tc2_falso` (`examples/tc2_falso.rs`): responde a
//! `--version` y `version --json` como un `tc2` de version [`VERSION_Y`].
//! `cargo test` compila los ejemplos antes de lanzar los tests.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};

use comun::Fixture;
use distribucion::Target;

/// Version del doble `tc2_falso`.
const VERSION_Y: &str = "9.9.9";
/// Version del `tc2` compilado (la «vX»).
const VERSION_X: &str = env!("CARGO_PKG_VERSION");

/// Servidor de ficheros minimo: `GET /<nombre>` da el contenido o 404, y
/// cuenta las peticiones.
struct Servidor {
    puerto: u16,
    peticiones: Arc<Mutex<Vec<String>>>,
}

impl Servidor {
    fn nuevo(ficheros: BTreeMap<String, Vec<u8>>) -> Self {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let puerto = l.local_addr().unwrap().port();
        let peticiones: Arc<Mutex<Vec<String>>> = Arc::default();
        let p = peticiones.clone();
        std::thread::spawn(move || {
            for s in l.incoming().flatten() {
                let mut lector = BufReader::new(s.try_clone().unwrap());
                let mut primera = String::new();
                if lector.read_line(&mut primera).is_err() {
                    continue;
                }
                loop {
                    let mut l = String::new();
                    if lector.read_line(&mut l).is_err() || l.trim().is_empty() {
                        break;
                    }
                }
                let ruta = primera.split_whitespace().nth(1).unwrap_or("").to_owned();
                p.lock().unwrap().push(ruta.clone());
                let (estado, cuerpo) = match ficheros.get(ruta.trim_start_matches('/')) {
                    Some(c) => ("200 OK", c.clone()),
                    None => ("404 Not Found", b"no".to_vec()),
                };
                let mut s = s;
                let _ = write!(
                    s,
                    "HTTP/1.1 {estado}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    cuerpo.len()
                );
                let _ = s.write_all(&cuerpo);
                let _ = s.flush();
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

fn target() -> Target {
    Target::actual().expect("los tests corren en un sistema de P12")
}

fn sha256(ruta: &Path) -> String {
    distribucion::sha256::hex(&std::fs::read(ruta).unwrap())
}

/// Copia el `tc2` compilado a `<dir>/tc2[.exe]`.
fn instalar_vx(dir: &Path) -> PathBuf {
    std::fs::create_dir_all(dir).unwrap();
    let destino = dir.join(format!("tc2{}", std::env::consts::EXE_SUFFIX));
    std::fs::copy(env!("CARGO_BIN_EXE_tc2"), &destino).unwrap();
    destino
}

/// Release de ensayo con `binario` como version `version` del target actual,
/// y `hash` en `SHA256SUMS` (el suyo si es `None`).
fn release(version: &str, binario: &[u8], hash: Option<&str>) -> BTreeMap<String, Vec<u8>> {
    let fichero = target().fichero(version);
    let hash = hash.map_or_else(|| distribucion::sha256::hex(binario), str::to_owned);
    let sumas = distribucion::sumas(&[(fichero.clone(), hash)]).unwrap();
    BTreeMap::from([
        (distribucion::SUMAS.to_owned(), sumas.into_bytes()),
        (fichero, binario.to_vec()),
    ])
}

fn lanzar(exe: &Path, base: &str, idioma: &str, args: &[&str]) -> Output {
    Command::new(exe)
        .args(args)
        .env("TC2_BASE_URL", base)
        .env("TC2_IDIOMA", idioma)
        .output()
        .unwrap()
}

fn texto(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

/// Ficheros de `dir` que no son el ejecutable.
fn restos(dir: &Path, exe: &Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p != exe)
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect()
}

/// Criterio 3: `--comprobar` dice si hay version nueva sin tocar nada, y sin
/// red sale `sinred` (4) con la URL.
#[test]
fn comprobar_no_modifica_nada_y_sin_red_da_sinred_con_la_url() {
    let f = Fixture::nueva("actualizar-comprobar");
    let dir = f.raiz.join("bin");
    let exe = instalar_vx(&dir);
    let antes = sha256(&exe);

    // Hay una mas nueva (el binario no se baja: basta SHA256SUMS).
    let srv = Servidor::nuevo(release(VERSION_Y, b"binario-y", None));
    let o = lanzar(
        &exe,
        &srv.base(),
        "es",
        &["actualizar", "--cli", "--comprobar"],
    );
    assert_eq!(o.status.code(), Some(0), "{}", texto(&o.stderr));
    assert_eq!(
        texto(&o.stdout),
        format!("actualizar ok actual={VERSION_X} disponible={VERSION_Y}\n")
    );
    // Solo SHA256SUMS: el binario no se baja. (No se cuenta cuantas: en
    // Windows, un antivirus puede repetir la primera conexion de un .exe
    // recien copiado a %TEMP%; se vio en TASK-027.)
    let p = srv.peticiones();
    assert!(
        !p.is_empty() && p.iter().all(|r| r == "/SHA256SUMS"),
        "{p:?}"
    );

    // La misma version: sin-cambios.
    let srv = Servidor::nuevo(release(VERSION_X, b"binario-x", None));
    let o = lanzar(
        &exe,
        &srv.base(),
        "es",
        &["actualizar", "--cli", "--comprobar"],
    );
    assert_eq!(o.status.code(), Some(0), "{}", texto(&o.stderr));
    assert_eq!(
        texto(&o.stdout),
        format!("actualizar sin-cambios actual={VERSION_X} disponible={VERSION_X}\n")
    );

    // Sin red: un puerto en el que ya no escucha nadie.
    let muerto = {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };
    let base = format!("http://127.0.0.1:{muerto}");
    for idioma in ["es", "en"] {
        let o = lanzar(&exe, &base, idioma, &["actualizar", "--cli", "--comprobar"]);
        assert_eq!(o.status.code(), Some(4), "{}", texto(&o.stderr));
        assert_eq!(
            texto(&o.stdout),
            format!("actualizar sinred url={base}/SHA256SUMS\n")
        );
        assert!(texto(&o.stderr).contains(&format!("{base}/SHA256SUMS")));
    }

    assert_eq!(sha256(&exe), antes);
    assert!(restos(&dir, &exe).is_empty(), "{:?}", restos(&dir, &exe));
}

/// Criterio 6: un `tc2` en la cache del plugin no se sustituye, y ni
/// siquiera se pregunta a la release.
#[test]
fn el_tc2_de_la_cache_del_plugin_no_se_sustituye() {
    let f = Fixture::nueva("actualizar-plugin");
    let dir = f.raiz.join("taskcodetwo").join("cli").join(VERSION_X);
    let exe = instalar_vx(&dir);
    let antes = sha256(&exe);
    let srv = Servidor::nuevo(release(VERSION_Y, b"binario-y", None));
    for idioma in ["es", "en"] {
        let o = lanzar(&exe, &srv.base(), idioma, &["actualizar", "--cli"]);
        assert_eq!(o.status.code(), Some(3));
        assert_eq!(
            texto(&o.stdout),
            "actualizar rechazada motivo=gestionado-por-plugin arreglo=actualizar-el-plugin\n"
        );
        assert!(
            texto(&o.stderr).contains("motivo=gestionado-por-plugin arreglo="),
            "{}",
            texto(&o.stderr)
        );
    }
    assert!(srv.peticiones().is_empty());
    assert_eq!(sha256(&exe), antes);
}

/// Sin `--cli`, uso: actualizar un proyecto es de F9-2.
#[test]
fn actualizar_sin_cli_es_uso() {
    let o = Command::new(env!("CARGO_BIN_EXE_tc2"))
        .args(["actualizar"])
        .env("TC2_IDIOMA", "es")
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(2));
    assert_eq!(texto(&o.stdout), "actualizar uso motivo=sin-cli\n");
}
