//! F6-1 (TASK-047): identidad propia del agente.
//!
//! - C1.1: la GitHub App firma un JWT RS256 que se verifica con su clave
//!   publica, pide el token de instalacion reducido al repo y a los permisos
//!   del rol, lo reutiliza y lo renueva al caducar; por la fabrica de
//!   produccion, el adaptador usa ese token y un fallo al acunarlo no es
//!   «falta identidad».
//! - C2.3: el entorno git del agente lleva la cabecera y el autor `[bot]`, y
//!   el token no aparece en los argumentos de git ni de sus hijos.
//!
//! Recursos reales: la clave RSA la genera `ssh-keygen -t rsa -m PEM` en la
//! carpeta temporal del test (ninguna clave en el repo), los repos son git
//! reales y la API es el servidor simulado de `comun::http_mock`.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use comun::http_mock::{Mock, Recibida, Simulada, serie};
use comun::{Fixture, escribir};
use serde_json::{Value, json};
use tc2::contrato::Resultado;
use tc2::git;
use tc2::plataforma::credenciales::de_entorno_con;
use tc2::plataforma::http::Base;
use tc2::plataforma::{
    AutorGit, Credenciales, CredencialesApp, FalloCredencial, GitHub, GitLab, Personas, Plataforma,
    Proveedor, Rol, Secreto, entorno_git,
};

const APP: &str = "123456";

/// Clave RSA desechable en `dir`: `(ruta del PEM privado, DER PKCS#1 de la
/// publica)`. La publica la saca `ssh-keygen -e -m PEM`, no el codigo que
/// se prueba.
fn clave_rsa(dir: &Path) -> (PathBuf, Vec<u8>) {
    let ruta = dir.join("app.pem");
    let o = Command::new("ssh-keygen")
        .args(["-q", "-t", "rsa", "-b", "2048", "-m", "PEM", "-N", "", "-f"])
        .arg(&ruta)
        .output()
        .expect("lanzar ssh-keygen");
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let pem = std::fs::read_to_string(&ruta).unwrap();
    // Suposicion del plan: GitHub da (y ssh-keygen -m PEM genera) PKCS#1.
    assert!(pem.starts_with("-----BEGIN RSA PRIVATE KEY-----"), "{pem}");
    let o = Command::new("ssh-keygen")
        .args(["-e", "-m", "PEM", "-f"])
        .arg(dir.join("app.pem.pub"))
        .output()
        .expect("lanzar ssh-keygen -e");
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let publica = String::from_utf8(o.stdout).unwrap();
    let b64: String = publica
        .lines()
        .filter(|l| !l.starts_with("-----"))
        .map(str::trim)
        .collect();
    (ruta, STANDARD.decode(b64).unwrap())
}

fn ahora() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Comprueba el JWT de una peticion contra la clave publica y devuelve su
/// carga.
fn jwt_verificado(r: &Recibida, publica: &[u8]) -> Value {
    let jwt = r
        .autorizacion
        .as_deref()
        .and_then(|a| a.strip_prefix("Bearer "))
        .unwrap_or_else(|| panic!("sin Bearer en {}", r.linea));
    let partes: Vec<&str> = jwt.split('.').collect();
    assert_eq!(partes.len(), 3, "{jwt}");
    let firmado = format!("{}.{}", partes[0], partes[1]);
    let firma = URL_SAFE_NO_PAD.decode(partes[2]).unwrap();
    ring::signature::UnparsedPublicKey::new(&ring::signature::RSA_PKCS1_2048_8192_SHA256, publica)
        .verify(firmado.as_bytes(), &firma)
        .expect("la firma del JWT no se verifica con la clave publica de la App");
    let cab: Value = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(partes[0]).unwrap()).unwrap();
    assert_eq!(cab, json!({"alg": "RS256", "typ": "JWT"}));
    serde_json::from_slice(&URL_SAFE_NO_PAD.decode(partes[1]).unwrap()).unwrap()
}

fn permisos_json(rol: Rol) -> Value {
    let m: serde_json::Map<String, Value> = tc2::plataforma::app_github::permisos(rol)
        .unwrap()
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).into()))
        .collect();
    Value::Object(m)
}

/// C1.1: JWT verificable, token reducido al repo y a los permisos exactos
/// del rol, cache y renovacion.
#[test]
fn la_app_pide_un_token_reducido_lo_guarda_y_lo_renueva() {
    let _serie = serie();
    let f = Fixture::nueva("app-github");
    let (ruta, publica) = clave_rsa(&f.raiz);
    let pem = std::fs::read_to_string(&ruta).unwrap();

    let mock = Mock::nuevo();
    mock.responder(
        "GET",
        "/repos/o/r/installation",
        Simulada::json(200, r#"{"id":77}"#),
    );
    // El primero ya caducado: la siguiente llamada tiene que renovarlo. El
    // segundo vale hasta 2099: la tercera lo reutiliza.
    mock.responder(
        "POST",
        "/app/installations/77/access_tokens",
        Simulada::json(
            201,
            r#"{"token":"ghs_uno","expires_at":"2000-01-01T00:00:00Z"}"#,
        ),
    );
    mock.responder(
        "POST",
        "/app/installations/77/access_tokens",
        Simulada::json(
            201,
            r#"{"token":"ghs_dos","expires_at":"2099-01-01T00:00:00Z"}"#,
        ),
    );
    let base = Base::nueva(&mock.base()).unwrap();
    let app = CredencialesApp::nueva(APP, &pem, base.clone(), "o/r", Rol::Agente).unwrap();
    let antes = ahora();
    assert_eq!(app.obtener(Rol::Agente), Ok(Secreto::nuevo("ghs_uno")));
    assert_eq!(app.obtener(Rol::Agente), Ok(Secreto::nuevo("ghs_dos")));
    assert_eq!(app.obtener(Rol::Agente), Ok(Secreto::nuevo("ghs_dos")));
    // Otro rol no sale de esta App.
    assert_eq!(app.obtener(Rol::Revisor), Err(FalloCredencial::Falta));

    let r = mock.recibidas();
    let lineas: Vec<&str> = r.iter().map(|x| x.linea.as_str()).collect();
    assert_eq!(
        lineas,
        [
            "GET /repos/o/r/installation",
            "POST /app/installations/77/access_tokens",
            "POST /app/installations/77/access_tokens",
        ],
        "la instalacion se busca una vez y el token se pide dos (renovacion), no tres"
    );
    for x in &r {
        let carga = jwt_verificado(x, &publica);
        assert_eq!(carga["iss"], json!(123_456), "{carga}");
        let (iat, exp) = (
            carga["iat"].as_u64().unwrap(),
            carga["exp"].as_u64().unwrap(),
        );
        assert!(iat + 50 <= antes + 1 && iat + 70 >= antes, "{carga}");
        assert!(exp > ahora() && exp - iat <= 600, "{carga}");
    }
    for x in &r[1..] {
        let cuerpo: Value = serde_json::from_str(&x.cuerpo).unwrap();
        assert_eq!(
            cuerpo,
            json!({"repositories": ["r"], "permissions": permisos_json(Rol::Agente)})
        );
    }

    // El revisor pide sus permisos (checks:write, el resto lectura).
    let mock = Mock::nuevo();
    mock.responder(
        "GET",
        "/repos/o/r/installation",
        Simulada::json(200, r#"{"id":78}"#),
    );
    mock.responder(
        "POST",
        "/app/installations/78/access_tokens",
        Simulada::json(
            201,
            r#"{"token":"ghs_rev","expires_at":"2099-01-01T00:00:00Z"}"#,
        ),
    );
    let base = Base::nueva(&mock.base()).unwrap();
    let rev = CredencialesApp::nueva(APP, &pem, base, "o/r", Rol::Revisor).unwrap();
    assert_eq!(rev.obtener(Rol::Revisor), Ok(Secreto::nuevo("ghs_rev")));
    let cuerpo: Value = serde_json::from_str(&mock.recibidas()[1].cuerpo).unwrap();
    assert_eq!(
        cuerpo,
        json!({"repositories": ["r"], "permissions": {
            "checks": "write", "contents": "read", "pull_requests": "read", "metadata": "read"
        }})
    );
}

/// C1.1, por la fabrica de produccion: el adaptador de GitHub con
/// `TC2_APP_AGENTE_*` usa el token de instalacion (no el de la variable) y
/// un fallo al acunar sale `denegada motivo=credenciales causa=...`.
#[test]
fn el_adaptador_usa_la_app_de_la_fabrica_y_su_fallo_no_es_falta() {
    let _serie = serie();
    let f = Fixture::nueva("app-fabrica");
    let (ruta, _) = clave_rsa(&f.raiz);
    let clave = ruta.to_str().unwrap().to_owned();
    let vars = move |v: &str| match v {
        "TC2_APP_AGENTE_ID" => Some(APP.to_owned()),
        "TC2_APP_AGENTE_CLAVE" => Some(clave.clone()),
        "TC2_TOKEN_AGENTE" => Some("ghp_de_la_variable".to_owned()),
        _ => None,
    };

    let mock = Mock::nuevo();
    mock.responder(
        "GET",
        "/repos/o/r/installation",
        Simulada::json(200, r#"{"id":9}"#),
    );
    mock.responder(
        "POST",
        "/app/installations/9/access_tokens",
        Simulada::json(
            201,
            r#"{"token":"ghs_instalacion","expires_at":"2099-01-01T00:00:00Z"}"#,
        ),
    );
    mock.responder("GET", "/repos/o/r/pulls", Simulada::json(200, "[]"));
    let gh = GitHub::nuevo(
        Base::nueva(&mock.base()).unwrap(),
        "o/r",
        Box::new(de_entorno_con(&vars, false)),
        Personas::default(),
    )
    .unwrap();
    assert_eq!(gh.prs_abiertos(Rol::Agente).unwrap(), Vec::new());
    let pulls = mock
        .recibidas()
        .into_iter()
        .find(|r| r.linea.starts_with("GET /repos/o/r/pulls"))
        .expect("prs-abiertos no llego a la API");
    assert_eq!(
        pulls.autorizacion.as_deref(),
        Some("Bearer ghs_instalacion")
    );

    // App sin instalar en el repo: no es «falta identidad».
    let mock = Mock::nuevo();
    mock.responder(
        "GET",
        "/repos/o/r/installation",
        Simulada::json(404, r#"{"message":"Not Found"}"#),
    );
    let gh = GitHub::nuevo(
        Base::nueva(&mock.base()).unwrap(),
        "o/r",
        Box::new(de_entorno_con(&vars, false)),
        Personas::default(),
    )
    .unwrap();
    let s = gh.prs_abiertos(Rol::Agente).unwrap_err().salida(false);
    assert_eq!(
        (s.stdout.as_str(), s.codigo),
        (
            "prs-abiertos denegada motivo=credenciales rol=agente causa=sin-instalacion\n",
            8
        )
    );
    assert!(s.stderr.contains("TC2_APP_AGENTE_ID"), "{}", s.stderr);
    assert!(!s.stderr.contains("ghp_de_la_variable"), "{}", s.stderr);
    // Nunca se probo con el token de la variable ni con otro rol.
    assert!(
        mock.recibidas()
            .iter()
            .all(|r| r.linea.contains("installation"))
    );
}

/// C2.3: entorno git del agente con un repo real: el commit sale con el
/// autor `[bot]`, el push manda la cabecera al remoto HTTP y el token no
/// esta en el registro del puerto ni en la linea de ordenes de git ni de
/// sus hijos (`GIT_TRACE`).
#[test]
fn el_entorno_git_del_agente_lleva_cabecera_y_autor_sin_token_en_argv() {
    const TOKEN: &str = "ghs_token_de_instalacion_secreto";
    let _serie = serie();
    let f = Fixture::nueva("git-agente");
    let (_bare, clon) = f.remoto_y_clon();
    let mock = Mock::nuevo();
    let traza = f.raiz.join("traza.txt");
    let autor = AutorGit::bot_github("taskcode-agente", Some(4242));
    let mut extra = entorno_git(
        Proveedor::GitHub,
        &format!("{}/", mock.base()),
        &Secreto::nuevo(TOKEN),
        Some(&autor),
    );
    extra.push((
        "GIT_TRACE".into(),
        traza.to_str().unwrap().replace('\\', "/").into(),
    ));
    let g = f.git().con_entorno_extra(extra);

    let i =
        git::crear_rama_y_worktree(&g, &clon, "TASK-047", "feature", "agente", "origin/develop");
    assert!(i.resultado.es_exito(), "{}", i.linea());
    let wt = clon.join(".taskcodetwo/worktrees/TASK-047");
    escribir(&wt, "a.txt", "del agente\n");
    let i = git::guardar(&g, &wt, "feat(TASK-047): commit del agente", &["a.txt"]);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    let quien = f.crudo(&wt, &["log", "-1", "--format=%an <%ae>|%cn <%ce>"]);
    assert_eq!(
        quien.trim(),
        "taskcode-agente[bot] <4242+taskcode-agente[bot]@users.noreply.github.com>|\
         taskcode-agente[bot] <4242+taskcode-agente[bot]@users.noreply.github.com>"
    );

    // El remoto pasa a ser HTTP (el servidor simulado, que responde 404):
    // lo que importa es lo que git le manda.
    f.crudo(
        &clon,
        &[
            "remote",
            "set-url",
            "origin",
            &format!("{}/o/r.git", mock.base()),
        ],
    );
    let i = git::push(&g, &wt, &[Duration::ZERO, Duration::ZERO]);
    eprintln!("  {}", i.linea());
    assert!(!i.resultado.es_exito(), "{}", i.linea());

    let basica = STANDARD.encode(format!("x-access-token:{TOKEN}"));
    let recibidas = mock.recibidas();
    let refs = recibidas
        .iter()
        .find(|r| r.linea.starts_with("GET /o/r.git/info/refs"))
        .unwrap_or_else(|| panic!("git no llego al remoto: {recibidas:?}"));
    assert_eq!(
        refs.autorizacion.as_deref(),
        Some(format!("Basic {basica}").as_str())
    );

    let registro = g.registro().join("\n");
    let traza = std::fs::read_to_string(&traza).unwrap();
    assert!(
        traza.contains("remote-http") || traza.contains("remote-https"),
        "la traza no recoge los hijos de git: {traza}"
    );
    for (donde, texto) in [("registro", &registro), ("GIT_TRACE", &traza)] {
        assert!(!texto.contains(TOKEN), "token en {donde}: {texto}");
        assert!(!texto.contains(&basica), "cabecera en {donde}: {texto}");
    }
}

// --- Ronda 1 de revision de TASK-047 (IMP-1, MEN-1, MEN-2, MEN-4,
// MENOR-1, MENOR-3, MENOR-4) ---

/// `2026-11-19T10:00:00Z` de unos segundos desde 1970 (algoritmo de dias
/// civiles de H. Hinnant; el `expires_at` que da GitHub).
fn iso(segundos: u64) -> String {
    let dias = i64::try_from(segundos / 86_400).unwrap();
    let resto = segundos % 86_400;
    let z = dias + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let dia = doy - (153 * mp + 2) / 5 + 1;
    let mes = if mp < 10 { mp + 3 } else { mp - 9 };
    let anio = yoe + era * 400 + i64::from(mes <= 2);
    format!(
        "{anio:04}-{mes:02}-{dia:02}T{:02}:{:02}:{:02}Z",
        resto / 3600,
        resto % 3600 / 60,
        resto % 60
    )
}

#[test]
fn iso_formatea_como_github() {
    assert_eq!(iso(0), "1970-01-01T00:00:00Z");
    assert_eq!(iso(951_782_400), "2000-02-29T00:00:00Z");
    assert_eq!(iso(4_070_908_800), "2099-01-01T00:00:00Z");
}

/// La causa de un fallo al acunar, o panico si no lo es.
fn causa(r: Result<Secreto, FalloCredencial>) -> (&'static str, String) {
    match r {
        Err(FalloCredencial::Acunar { causa, detalle }) => (causa, detalle),
        otro => panic!("se esperaba un fallo al acunar: {otro:?}"),
    }
}

fn app_del_agente(pem: &str, base: &str) -> CredencialesApp {
    CredencialesApp::nueva(APP, pem, Base::nueva(base).unwrap(), "o/r", Rol::Agente).unwrap()
}

/// Un puerto local en el que no escucha nadie.
fn puerto_cerrado() -> u16 {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    l.local_addr().unwrap().port()
}

/// IMP-1 / MENOR-3: el token se renueva cuando le quedan menos de
/// [`tc2::plataforma::app_github::MARGEN`] (5 min) y se reutiliza si le
/// quedan mas.
#[test]
fn el_token_que_vence_dentro_del_margen_se_renueva_y_el_de_fuera_no() {
    let _serie = serie();
    let f = Fixture::nueva("app-margen");
    let (ruta, _) = clave_rsa(&f.raiz);
    let pem = std::fs::read_to_string(&ruta).unwrap();
    assert_eq!(tc2::plataforma::app_github::MARGEN, 300);
    let t0 = ahora();
    let mock = Mock::nuevo();
    mock.responder(
        "GET",
        "/repos/o/r/installation",
        Simulada::json(200, r#"{"id":77}"#),
    );
    // Vence en 4 min (dentro del margen): la segunda llamada lo renueva. El
    // segundo vence en 10 min (fuera): la tercera lo reutiliza.
    for (token, vida) in [("ghs_cuatro", 240), ("ghs_diez", 600)] {
        mock.responder(
            "POST",
            "/app/installations/77/access_tokens",
            Simulada::json(
                201,
                &format!(r#"{{"token":"{token}","expires_at":"{}"}}"#, iso(t0 + vida)),
            ),
        );
    }
    let app = app_del_agente(&pem, &mock.base());
    assert_eq!(app.obtener(Rol::Agente), Ok(Secreto::nuevo("ghs_cuatro")));
    assert_eq!(app.obtener(Rol::Agente), Ok(Secreto::nuevo("ghs_diez")));
    assert_eq!(app.obtener(Rol::Agente), Ok(Secreto::nuevo("ghs_diez")));
    let posts = mock
        .recibidas()
        .iter()
        .filter(|r| r.linea.starts_with("POST"))
        .count();
    assert_eq!(posts, 2, "renueva el de 4 min y reutiliza el de 10");
}

/// IMP-1 / MENOR-3: un 404 de `access_tokens` (App reinstalada con otro id)
/// olvida la instalacion y la siguiente llamada la vuelve a buscar.
#[test]
fn un_404_al_pedir_el_token_olvida_la_instalacion() {
    let _serie = serie();
    let f = Fixture::nueva("app-reinstalada");
    let (ruta, _) = clave_rsa(&f.raiz);
    let pem = std::fs::read_to_string(&ruta).unwrap();
    let mock = Mock::nuevo();
    mock.responder(
        "GET",
        "/repos/o/r/installation",
        Simulada::json(200, r#"{"id":5}"#),
    );
    mock.responder(
        "GET",
        "/repos/o/r/installation",
        Simulada::json(200, r#"{"id":6}"#),
    );
    mock.responder(
        "POST",
        "/app/installations/5/access_tokens",
        Simulada::json(404, r#"{"message":"Not Found"}"#),
    );
    mock.responder(
        "POST",
        "/app/installations/6/access_tokens",
        Simulada::json(
            201,
            r#"{"token":"ghs_nueva","expires_at":"2099-01-01T00:00:00Z"}"#,
        ),
    );
    let app = app_del_agente(&pem, &mock.base());
    assert_eq!(causa(app.obtener(Rol::Agente)).0, "sin-instalacion");
    assert_eq!(app.obtener(Rol::Agente), Ok(Secreto::nuevo("ghs_nueva")));
    let lineas: Vec<String> = mock.recibidas().into_iter().map(|r| r.linea).collect();
    assert_eq!(
        lineas,
        [
            "GET /repos/o/r/installation",
            "POST /app/installations/5/access_tokens",
            "GET /repos/o/r/installation",
            "POST /app/installations/6/access_tokens",
        ]
    );
}

/// IMP-1 / MENOR-3: cada fallo al acunar sale con su causa del contrato de
/// la adenda F6-1.
#[test]
fn cada_fallo_al_acunar_tiene_su_causa() {
    let _serie = serie();
    let f = Fixture::nueva("app-causas");
    let (ruta, _) = clave_rsa(&f.raiz);
    let pem = std::fs::read_to_string(&ruta).unwrap();
    let instalada = || {
        let mock = Mock::nuevo();
        mock.responder(
            "GET",
            "/repos/o/r/installation",
            Simulada::json(200, r#"{"id":3}"#),
        );
        mock
    };
    let token = |cuerpo: &str| {
        let mock = instalada();
        mock.responder(
            "POST",
            "/app/installations/3/access_tokens",
            Simulada::json(201, cuerpo),
        );
        mock
    };

    // 401 del JWT.
    let mock = Mock::nuevo();
    mock.responder(
        "GET",
        "/repos/o/r/installation",
        Simulada::json(401, r#"{"message":"Bad credentials"}"#),
    );
    let (c, d) = causa(app_del_agente(&pem, &mock.base()).obtener(Rol::Agente));
    assert_eq!(
        (c, d.as_str()),
        ("jwt-rechazado", "HTTP 401 Bad credentials")
    );

    // 422: la instalacion no concede los permisos pedidos.
    let mock = instalada();
    mock.responder(
        "POST",
        "/app/installations/3/access_tokens",
        Simulada::json(422, r#"{"message":"permissions"}"#),
    );
    let (c, _) = causa(app_del_agente(&pem, &mock.base()).obtener(Rol::Agente));
    assert_eq!(c, "permisos-no-concedidos");

    // 201, pero con menos permisos de los pedidos: tampoco vale.
    let mock = token(
        r#"{"token":"ghs_x","expires_at":"2099-01-01T00:00:00Z","permissions":{
            "contents":"read","pull_requests":"write","checks":"read",
            "statuses":"read","metadata":"read"}}"#,
    );
    let (c, d) = causa(app_del_agente(&pem, &mock.base()).obtener(Rol::Agente));
    assert_eq!(
        (c, d.as_str()),
        ("permisos-no-concedidos", "contents:write")
    );
    // Con todos (write cubre read), si.
    let mock = token(
        r#"{"token":"ghs_x","expires_at":"2099-01-01T00:00:00Z","permissions":{
            "contents":"write","pull_requests":"write","checks":"write",
            "statuses":"read","metadata":"read"}}"#,
    );
    assert_eq!(
        app_del_agente(&pem, &mock.base()).obtener(Rol::Agente),
        Ok(Secreto::nuevo("ghs_x"))
    );

    // Otro estado, o un 201 sin la forma esperada.
    let mock = Mock::nuevo();
    mock.responder(
        "GET",
        "/repos/o/r/installation",
        Simulada::json(500, r#"{"message":"boom"}"#),
    );
    let (c, _) = causa(app_del_agente(&pem, &mock.base()).obtener(Rol::Agente));
    assert_eq!(c, "respuesta-inesperada");
    let mock = token(r#"{"sin":"token"}"#);
    let (c, _) = causa(app_del_agente(&pem, &mock.base()).obtener(Rol::Agente));
    assert_eq!(c, "respuesta-inesperada");

    // Sin red.
    let base = format!("http://127.0.0.1:{}", puerto_cerrado());
    let (c, _) = causa(app_del_agente(&pem, &base).obtener(Rol::Agente));
    assert_eq!(c, "sinred");
}

/// IMP-1 / MENOR-3, por el adaptador: sin red al acunar sale `sinred` (no
/// `denegada motivo=credenciales`).
#[test]
fn el_adaptador_traduce_sinred_al_acunar() {
    let _serie = serie();
    let f = Fixture::nueva("app-sinred");
    let (ruta, _) = clave_rsa(&f.raiz);
    let clave = ruta.to_str().unwrap().to_owned();
    let vars = move |v: &str| match v {
        "TC2_APP_AGENTE_ID" => Some(APP.to_owned()),
        "TC2_APP_AGENTE_CLAVE" => Some(clave.clone()),
        _ => None,
    };
    let salida = |base: String| {
        let gh = GitHub::nuevo(
            Base::nueva(&base).unwrap(),
            "o/r",
            Box::new(de_entorno_con(&vars, false)),
            Personas::default(),
        )
        .unwrap();
        gh.prs_abiertos(Rol::Agente).unwrap_err().salida(false)
    };

    let s = salida(format!("http://127.0.0.1:{}", puerto_cerrado()));
    assert!(s.stdout.starts_with("prs-abiertos sinred "), "{}", s.stdout);
    assert!(s.stdout.contains("causa=sinred"), "{}", s.stdout);
}

/// MEN-1 / MENOR-1: con la App del agente configurada y un destino GitLab,
/// se usa `TC2_TOKEN_AGENTE` (nunca el de otro rol); sin el, el diagnostico
/// dice que en GitLab la App no sirve y que hay que definirlo.
#[test]
fn en_gitlab_la_app_cede_al_token_del_mismo_rol() {
    let _serie = serie();
    let app = [
        ("TC2_APP_AGENTE_ID", APP),
        ("TC2_APP_AGENTE_CLAVE", "/no/existe/app.pem"),
    ];
    let con_token = |extra: &[(&'static str, &'static str)]| {
        let vars: Vec<(&str, &str)> = app.iter().chain(extra).copied().collect();
        move |v: &str| {
            vars.iter()
                .find(|(k, _)| *k == v)
                .map(|(_, x)| (*x).to_owned())
        }
    };
    let gitlab = |mock: &Mock, vars| {
        GitLab::nuevo(
            Base::nueva(&format!("{}/api/v4", mock.base())).unwrap(),
            "o/r",
            Box::new(de_entorno_con(vars, true)),
            Personas::default(),
        )
        .unwrap()
    };

    let mock = Mock::nuevo();
    let gl = gitlab(
        &mock,
        con_token(&[
            ("TC2_TOKEN_AGENTE", "glpat-agente"),
            ("TC2_TOKEN_PERSONA", "glpat-persona"),
        ]),
    );
    let _ = gl.prs_abiertos(Rol::Agente);
    let r = mock.recibidas();
    assert!(!r.is_empty(), "no llego a la API");
    assert!(
        r.iter()
            .all(|x| x.autorizacion.as_deref() == Some("Bearer glpat-agente")),
        "{r:?}"
    );

    // Sin token del agente: no se prueba con el de persona.
    let mock = Mock::nuevo();
    let gl = gitlab(&mock, con_token(&[("TC2_TOKEN_PERSONA", "glpat-persona")]));
    let s = gl.prs_abiertos(Rol::Agente).unwrap_err().salida(false);
    assert_eq!(
        (s.stdout.as_str(), s.codigo),
        (
            "prs-abiertos denegada motivo=credenciales rol=agente causa=app-solo-github\n",
            8
        )
    );
    assert!(s.stderr.contains("en GitLab"), "{}", s.stderr);
    assert!(s.stderr.contains("define TC2_TOKEN_AGENTE"), "{}", s.stderr);
    assert!(mock.recibidas().is_empty());
}

/// MEN-2: el PEM se lee aunque traiga BOM UTF-8 o texto antes de
/// `-----BEGIN`; sin `-----BEGIN`, el detalle lo dice.
#[test]
fn el_pem_admite_bom_y_texto_antes_del_begin() {
    let f = Fixture::nueva("app-pem-bom");
    let (ruta, _) = clave_rsa(&f.raiz);
    let pem = std::fs::read_to_string(&ruta).unwrap();
    let base = Base::nueva("http://127.0.0.1:9").unwrap();
    for variante in [
        format!("\u{feff}{pem}"),
        format!("# clave de la App del agente\r\n\r\n{pem}"),
        format!("\u{feff}Bag Attributes\n{}", pem.replace('\n', "\r\n")),
    ] {
        CredencialesApp::nueva(APP, &variante, base.clone(), "o/r", Rol::Agente)
            .unwrap_or_else(|e| panic!("{e:?}: {variante:.40}"));
    }
    let e = CredencialesApp::nueva(APP, "\u{feff}sin cabecera\n", base, "o/r", Rol::Agente)
        .unwrap_err();
    assert_eq!(
        e,
        FalloCredencial::Acunar {
            causa: "clave-ilegible",
            detalle: "pem sin -----BEGIN".to_owned(),
        }
    );
}

/// MEN-4 / MENOR-5: la cabecera del agente no pisa entradas `GIT_CONFIG_*`
/// anteriores del puerto, ni `con_entorno` la pisa a ella, en ningun orden.
#[test]
fn el_entorno_git_del_agente_se_acumula_con_otras_entradas_de_config() {
    const TOKEN: &str = "ghs_acumulado";
    let _serie = serie();
    let f = Fixture::nueva("git-acumula");
    let (_bare, clon) = f.remoto_y_clon();
    let mock = Mock::nuevo();
    let autor = AutorGit::bot_github("taskcode-agente", None);
    let previa: Vec<(std::ffi::OsString, std::ffi::OsString)> = vec![
        ("GIT_CONFIG_COUNT".into(), "1".into()),
        ("GIT_CONFIG_KEY_0".into(), "i18n.commitEncoding".into()),
        ("GIT_CONFIG_VALUE_0".into(), "ISO-8859-1".into()),
    ];
    // Previa, la del agente y, despues, `con_entorno` con otra variable.
    let g = f
        .git()
        .con_entorno_extra(previa)
        .con_entorno_extra(entorno_git(
            Proveedor::GitHub,
            &format!("{}/", mock.base()),
            &Secreto::nuevo(TOKEN),
            Some(&autor),
        ))
        .con_entorno(vec![("TC2_OTRA".into(), "x".into())]);

    let i =
        git::crear_rama_y_worktree(&g, &clon, "TASK-047", "feature", "agente", "origin/develop");
    assert!(i.resultado.es_exito(), "{}", i.linea());
    let wt = clon.join(".taskcodetwo/worktrees/TASK-047");
    escribir(&wt, "a.txt", "del agente\n");
    let i = git::guardar(&g, &wt, "feat(TASK-047): commit del agente", &["a.txt"]);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    // La entrada previa sigue viva: el commit lleva su codificacion.
    let objeto = f.crudo(&wt, &["cat-file", "-p", "HEAD"]);
    assert!(objeto.contains("\nencoding ISO-8859-1\n"), "{objeto}");

    // Y la cabecera del agente tambien.
    f.crudo(
        &clon,
        &[
            "remote",
            "set-url",
            "origin",
            &format!("{}/o/r.git", mock.base()),
        ],
    );
    let _ = git::push(&g, &wt, &[Duration::ZERO, Duration::ZERO]);
    let basica = STANDARD.encode(format!("x-access-token:{TOKEN}"));
    let refs = mock
        .recibidas()
        .into_iter()
        .find(|r| r.linea.starts_with("GET /o/r.git/info/refs"))
        .expect("git no llego al remoto");
    assert_eq!(
        refs.autorizacion.as_deref(),
        Some(format!("Basic {basica}").as_str())
    );
}
