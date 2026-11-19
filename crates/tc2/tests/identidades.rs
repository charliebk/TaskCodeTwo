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
    AutorGit, Credenciales, CredencialesApp, FalloCredencial, GitHub, Personas, Plataforma,
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
        &autor,
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
