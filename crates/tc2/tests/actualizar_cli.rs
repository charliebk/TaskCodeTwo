//! TASK-027 (F2-12): `tc2 actualizar --cli` contra una release servida en
//! 127.0.0.1 (`TC2_BASE_URL`), sin red. Cada prueba copia el `tc2` compilado
//! a una carpeta temporal (la «instalacion vX») y lo lanza desde ahi, porque
//! la orden sustituye el ejecutable que se esta ejecutando.
//!
//! La «vY» es el ejemplo `tc2_falso` (`examples/tc2_falso.rs`): responde a
//! `--version` y `version --json` como un `tc2` de version [`VERSION_Y`].
//! `cargo test` compila los ejemplos antes de lanzar los tests.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};

use distribucion::Target;

/// Carpeta temporal de una prueba, bajo `target/tmp` del workspace
/// (`CARGO_TARGET_TMPDIR`) y no en `%TEMP%`: en Windows, un antivirus retiene
/// a veces el trafico de un `.exe` recien copiado a `%TEMP%` (conexiones
/// repetidas y una descarga cortada a los ~19 s, visto en TASK-027). Se borra
/// al terminar.
struct Carpeta {
    raiz: PathBuf,
}

impl Carpeta {
    fn nueva(nombre: &str) -> Self {
        let raiz = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("tc2-actualizar")
            .join(format!("{nombre}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&raiz);
        std::fs::create_dir_all(&raiz).unwrap();
        Self { raiz }
    }
}

impl Drop for Carpeta {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.raiz);
    }
}

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
        let ficheros = Arc::new(ficheros);
        std::thread::spawn(move || {
            for s in l.incoming().flatten() {
                // Un hilo por conexion: el cierre ordenado de una no frena
                // las demas.
                let (p, ficheros) = (p.clone(), ficheros.clone());
                std::thread::spawn(move || atender(s, &p, &ficheros));
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

/// Sirve una peticion: `GET /<nombre>` da el contenido o 404.
fn atender(s: std::net::TcpStream, p: &Mutex<Vec<String>>, ficheros: &BTreeMap<String, Vec<u8>>) {
    let mut lector = BufReader::new(s.try_clone().unwrap());
    let mut primera = String::new();
    if lector.read_line(&mut primera).is_err() {
        return;
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
    // Cierre ordenado: en Windows, cerrar con datos sin leer en
    // el socket manda un RST que puede tirar la respuesta aun no
    // entregada (el cliente ve «Peer disconnected» a media
    // descarga del binario). Se cierra la escritura y se espera a
    // que el cliente cierre la suya.
    let _ = s.shutdown(std::net::Shutdown::Write);
    let _ = s.set_read_timeout(Some(std::time::Duration::from_secs(30)));
    let _ = std::io::Read::read_to_end(&mut lector, &mut Vec::new());
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

/// Ruta de un ejemplo (`tc2_falso`, `gh_falso`), que `cargo test` deja en
/// `examples/`.
fn ejemplo(nombre: &str) -> PathBuf {
    let ruta = Path::new(env!("CARGO_BIN_EXE_tc2"))
        .parent()
        .unwrap()
        .join("examples")
        .join(format!("{nombre}{}", std::env::consts::EXE_SUFFIX));
    assert!(
        ruta.is_file(),
        "falta {}: lanza `cargo test -p tc2`, que compila los ejemplos",
        ruta.display()
    );
    ruta
}

/// El binario del doble `tc2_falso`.
fn binario_vy() -> Vec<u8> {
    std::fs::read(ejemplo("tc2_falso")).unwrap()
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
    lanzar_con(exe, base, idioma, args, &[])
}

/// Como [`lanzar`], con variables de mas. Sin `TC2_GH` ni `GH_FALSO*`
/// heredadas de quien corre los tests.
fn lanzar_con(
    exe: &Path,
    base: &str,
    idioma: &str,
    args: &[&str],
    entorno: &[(&str, &std::ffi::OsStr)],
) -> Output {
    let mut c = Command::new(exe);
    c.args(args)
        .env("TC2_BASE_URL", base)
        .env("TC2_IDIOMA", idioma)
        .env_remove("TC2_GH")
        .env_remove("GH_FALSO")
        .env_remove("GH_FALSO_REGISTRO");
    for (k, v) in entorno {
        c.env(k, v);
    }
    c.output().unwrap()
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
    let f = Carpeta::nueva("actualizar-comprobar");
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

    // Revision 1, MEN-6: con --version de una anterior, `ok` lo dice
    // (`cambio=anterior`) y no se confunde con una mas nueva.
    let anterior = format!("{VERSION_X}-anterior");
    let srv = Servidor::nuevo(release(&anterior, b"binario-anterior", None));
    let o = lanzar(
        &exe,
        &srv.base(),
        "es",
        &["actualizar", "--cli", "--comprobar", "--version", &anterior],
    );
    assert_eq!(o.status.code(), Some(0), "{}", texto(&o.stderr));
    assert_eq!(
        texto(&o.stdout),
        format!("actualizar ok actual={VERSION_X} disponible={anterior} cambio=anterior\n")
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
    let f = Carpeta::nueva("actualizar-plugin");
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

/// Criterio 1: la instalacion vX pasa a vY sin reinstalar; despues
/// `tc2 --version` da vY.
#[test]
fn actualizar_sustituye_vx_por_vy_servida_en_127_0_0_1() {
    let f = Carpeta::nueva("actualizar-vx-vy");
    let dir = f.raiz.join("bin");
    let exe = instalar_vx(&dir);
    let vy = binario_vy();
    let srv = Servidor::nuevo(release(VERSION_Y, &vy, None));

    let antes = Command::new(&exe).arg("--version").output().unwrap();
    assert_eq!(texto(&antes.stdout), format!("tc2 {VERSION_X}\n"));

    let o = lanzar(&exe, &srv.base(), "es", &["actualizar", "--cli"]);
    assert_eq!(o.status.code(), Some(0), "{}", texto(&o.stderr));
    let hash_vy = distribucion::sha256::hex(&vy);
    assert_eq!(
        texto(&o.stdout),
        format!(
            "actualizar ok anterior={VERSION_X} version={VERSION_Y} sha256={hash_vy} attestation=ensayo\n"
        )
    );

    let despues = Command::new(&exe).arg("--version").output().unwrap();
    assert_eq!(texto(&despues.stdout), format!("tc2 {VERSION_Y}\n"));
    assert_eq!(sha256(&exe), hash_vy);
    // No queda la descarga. En Windows queda `tc2.exe.old` (el proceso que
    // actualiza no puede borrarse a si mismo) hasta la siguiente ejecucion;
    // ya terminado, se puede borrar.
    let quedan = restos(&dir, &exe);
    assert!(quedan.iter().all(|r| r == "tc2.exe.old"), "{quedan:?}");
    tc2::actualizar_cli::limpiar_old(&exe);
    assert!(restos(&dir, &exe).is_empty());
    let pedidos = srv.peticiones();
    assert!(pedidos.contains(&format!("/{}", target().fichero(VERSION_Y))));
}

/// Criterio 2: si el sha256 no casa con `SHA256SUMS`, `rechazada` (3), el
/// binario instalado no cambia y no quedan `.descarga` ni `.old`.
#[test]
fn sha256_que_no_casa_es_rechazada_y_no_toca_nada() {
    let f = Carpeta::nueva("actualizar-hash");
    let dir = f.raiz.join("bin");
    let exe = instalar_vx(&dir);
    let antes = sha256(&exe);
    let vy = binario_vy();
    let otro = "0".repeat(64);
    let srv = Servidor::nuevo(release(VERSION_Y, &vy, Some(&otro)));
    for idioma in ["es", "en"] {
        let o = lanzar(&exe, &srv.base(), idioma, &["actualizar", "--cli"]);
        assert_eq!(o.status.code(), Some(3), "{}", texto(&o.stderr));
        let linea = texto(&o.stdout);
        assert!(
            linea.starts_with("actualizar rechazada motivo=sha256-no-casa fichero="),
            "{linea}"
        );
        assert!(linea.contains(&format!("esperado={otro}")), "{linea}");
        assert_eq!(sha256(&exe), antes);
        let quedan = restos(&dir, &exe);
        assert!(quedan.is_empty(), "{quedan:?}");
    }
    // Y sigue siendo la vX.
    let v = Command::new(&exe).arg("--version").output().unwrap();
    assert_eq!(texto(&v.stdout), format!("tc2 {VERSION_X}\n"));
}

/// Revision 1, IMP-1: con `gh` disponible se comprueba la attestation, como
/// en los instaladores. Tres casos con el doble `gh_falso` (`TC2_GH`): sin
/// `gh` sigue con el sha256 (`attestation=sin-gh`); con `gh` que la da por
/// buena, `attestation=comprobada`; con `gh` que la rechaza, `rechazada` (3)
/// y el binario instalado no cambia.
#[test]
fn attestation_con_gh_ausente_valido_y_que_falla() {
    let vy = binario_vy();
    let hash_vy = distribucion::sha256::hex(&vy);
    let gh = ejemplo("gh_falso");
    let srv = Servidor::nuevo(release(VERSION_Y, &vy, None));

    // gh ausente.
    let f = Carpeta::nueva("attestation-sin-gh");
    let exe = instalar_vx(&f.raiz.join("bin"));
    let no_hay = f
        .raiz
        .join(format!("gh-no-existe{}", std::env::consts::EXE_SUFFIX));
    let o = lanzar_con(
        &exe,
        &srv.base(),
        "es",
        &["actualizar", "--cli"],
        &[("TC2_GH", no_hay.as_os_str())],
    );
    assert_eq!(o.status.code(), Some(0), "{}", texto(&o.stderr));
    assert!(
        texto(&o.stdout).ends_with(" attestation=sin-gh\n"),
        "{}",
        texto(&o.stdout)
    );
    assert_eq!(sha256(&exe), hash_vy);

    // gh que da la attestation por buena, con los argumentos del instalador.
    let f = Carpeta::nueva("attestation-ok");
    let exe = instalar_vx(&f.raiz.join("bin"));
    let registro = f.raiz.join("gh.log");
    let o = lanzar_con(
        &exe,
        &srv.base(),
        "es",
        &["actualizar", "--cli"],
        &[
            ("TC2_GH", gh.as_os_str()),
            ("GH_FALSO", std::ffi::OsStr::new("ok")),
            ("GH_FALSO_REGISTRO", registro.as_os_str()),
        ],
    );
    assert_eq!(o.status.code(), Some(0), "{}", texto(&o.stderr));
    assert_eq!(
        texto(&o.stdout),
        format!(
            "actualizar ok anterior={VERSION_X} version={VERSION_Y} sha256={hash_vy} attestation=comprobada\n"
        )
    );
    assert_eq!(sha256(&exe), hash_vy);
    let llamadas = std::fs::read_to_string(&registro).unwrap();
    let verify: Vec<&str> = llamadas
        .lines()
        .find(|l| l.starts_with("attestation\tverify\t"))
        .unwrap_or_default()
        .split('\t')
        .collect();
    assert_eq!(
        verify.get(3..),
        Some(
            &[
                "--repo",
                "charliebk/TaskCodeTwo",
                "--signer-workflow",
                "charliebk/TaskCodeTwo/.github/workflows/release.yml"
            ][..]
        ),
        "{llamadas}"
    );
    assert!(
        verify.get(2).is_some_and(|f| f.ends_with(".descarga")),
        "{llamadas}"
    );

    // gh que la rechaza: rechazada y nada cambia.
    let f = Carpeta::nueva("attestation-falla");
    let dir = f.raiz.join("bin");
    let exe = instalar_vx(&dir);
    let antes = sha256(&exe);
    for idioma in ["es", "en"] {
        let o = lanzar_con(
            &exe,
            &srv.base(),
            idioma,
            &["actualizar", "--cli"],
            &[
                ("TC2_GH", gh.as_os_str()),
                ("GH_FALSO", std::ffi::OsStr::new("falla")),
            ],
        );
        assert_eq!(o.status.code(), Some(3), "{}", texto(&o.stderr));
        assert_eq!(
            texto(&o.stdout),
            format!(
                "actualizar rechazada motivo=attestation-no-valida fichero={}\n",
                target().fichero(VERSION_Y)
            )
        );
        assert!(
            texto(&o.stderr).contains("no attestations found"),
            "{}",
            texto(&o.stderr)
        );
        assert_eq!(sha256(&exe), antes);
        let quedan = restos(&dir, &exe);
        assert!(quedan.is_empty(), "{quedan:?}");
    }
}

/// Revision 1, MEN-3: en Windows, el `tc2.exe.old` que deja una
/// actualizacion lo borra la siguiente ejecucion de `tc2` (`main.rs`).
#[cfg(windows)]
#[test]
fn la_siguiente_ejecucion_borra_el_old() {
    let f = Carpeta::nueva("old-al-arrancar");
    let dir = f.raiz.join("bin");
    let exe = instalar_vx(&dir);
    let old = tc2::actualizar_cli::ruta_old(&exe);
    std::fs::write(&old, b"binario anterior").unwrap();
    let o = Command::new(&exe).arg("--version").output().unwrap();
    assert_eq!(texto(&o.stdout), format!("tc2 {VERSION_X}\n"));
    assert!(!old.exists(), "{} sigue ahi", old.display());
}
