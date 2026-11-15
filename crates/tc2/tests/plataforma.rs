//! F2-6: puerto Plataforma y adaptador GitHub contra un servidor simulado en
//! 127.0.0.1 (sin red). La bateria de los 6 casos es la de la PoC M de F1-13
//! con sus fixtures (`tests/fixtures/plataforma/github`); F2-9 pasara la
//! misma con GitLab.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use comun::http_mock::{Mock, Simulada};
use tc2::contrato::Resultado;
use tc2::plataforma::http::{Base, Cliente, FalloHttp, Metodo};
use tc2::plataforma::{
    CredencialesFijas, GitHub, ModeloPuertas, Personas, Plataforma, Rol, Secreto,
};

const TOKEN: &str = "ghp_token_de_prueba_que_no_debe_salir";
const SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PERSONA: u64 = 1001;

fn modelo(exigir: &[&str]) -> ModeloPuertas {
    ModeloPuertas {
        checks: vec![
            ("verificacion".into(), 15368),
            ("carril".into(), 15368),
            ("revision-ia".into(), 99001),
        ],
        exigir_impuestas: exigir.iter().map(|g| (*g).to_owned()).collect(),
    }
}

fn adaptador(mock: &Mock, roles: &[Rol]) -> GitHub {
    let mut c = CredencialesFijas::nuevas();
    for r in roles {
        c = c.con(*r, Secreto::nuevo(TOKEN));
    }
    GitHub::nuevo(
        Base::nueva(&mock.base()).unwrap(),
        "o/r",
        Box::new(c),
        Personas::nuevas([PERSONA]),
    )
    .unwrap()
}

fn persona_en(mock: &Mock, id: u64) {
    mock.responder(
        "GET",
        "/user",
        Simulada::json(
            200,
            &format!("{{\"login\":\"carlos\",\"id\":{id},\"type\":\"User\"}}"),
        ),
    );
}

fn pr(numero: u64, merged: bool, state: &str) -> String {
    format!(
        "{{\"number\":{numero},\"state\":\"{state}\",\"merged\":{merged},\"mergeable_state\":\"clean\",\"head\":{{\"ref\":\"feature/x\",\"sha\":\"{SHA}\"}},\"base\":{{\"ref\":\"develop\",\"sha\":\"{SHA}\"}}}}"
    )
}

/// Criterio 2: la bateria de los 6 casos de la PoC M (misma linea y mismo
/// numero de llamadas que en F1-13), y `abrir-pr` y `merge-pr` idempotentes.
#[test]
fn bateria_de_la_poc_m_e_idempotencia() {
    let carpeta = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/plataforma/github"
    );
    let casos = [
        (
            "verde",
            "estado-puertas ok pr=12 aprobaciones=1 ignoradas=0",
            0,
            4,
        ),
        (
            "esperando-checks",
            "estado-puertas esperando pr=12 aprobaciones=1 ignoradas=0 falta=revision-ia",
            7,
            4,
        ),
        (
            "sin-aprobacion",
            "estado-puertas esperando pr=12 aprobaciones=0 ignoradas=0 falta=aprobacion",
            7,
            4,
        ),
        (
            "aprobacion-bot",
            "estado-puertas esperando pr=12 aprobaciones=0 ignoradas=1 falta=aprobacion",
            7,
            4,
        ),
        (
            "plan",
            "estado-puertas rechazada pr=12 motivo=plan garantia=G03",
            3,
            2,
        ),
        (
            "sin-cambios",
            "estado-puertas sin-cambios pr=12 estado=mergeado",
            0,
            1,
        ),
    ];
    for (caso, linea, codigo, llamadas) in casos {
        let f: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(format!("{carpeta}/{caso}.json")).unwrap(),
        )
        .unwrap();
        let mock = Mock::nuevo();
        for (ruta, r) in f["respuestas"].as_object().unwrap() {
            mock.responder(
                "GET",
                ruta,
                Simulada::json(
                    r["estado"].as_u64().unwrap() as u16,
                    &r["cuerpo"].to_string(),
                ),
            );
        }
        let exigir: Vec<&str> = f["politica"]["exigir_impuestas"]
            .as_array()
            .unwrap()
            .iter()
            .map(|g| g.as_str().unwrap())
            .collect();
        let gh = adaptador(&mock, &[Rol::Agente]);
        let i = gh.estado_puertas(Rol::Agente, 12, &modelo(&exigir));
        assert_eq!(i.linea(), linea, "{caso}");
        assert_eq!(i.resultado.codigo(), codigo, "{caso}");
        assert_eq!(gh.registro().len(), llamadas, "{caso}: {:?}", gh.registro());
        let recibidas = mock.recibidas();
        assert_eq!(recibidas.len(), llamadas, "{caso}");
        assert!(
            recibidas
                .iter()
                .all(|r| r.autorizacion.as_deref() == Some(&format!("Bearer {TOKEN}")[..])),
            "{caso}: cada llamada lleva el token del rol"
        );
    }

    // abrir-pr: abre y, la segunda vez, devuelve el mismo PR sin crear otro.
    let mock = Mock::nuevo();
    let lista = "GET /repos/o/r/pulls?head=o%3Afeature%2Fx&base=develop&state=open&per_page=100";
    let (m, r) = lista.split_once(' ').unwrap();
    mock.responder(m, r, Simulada::json(200, "[]"));
    mock.responder(
        m,
        r,
        Simulada::json(200, &format!("[{}]", pr(5, false, "open"))),
    );
    mock.responder(
        "POST",
        "/repos/o/r/pulls",
        Simulada::json(201, &pr(5, false, "open")),
    );
    let gh = adaptador(&mock, &[Rol::Agente]);
    assert_eq!(
        gh.abrir_pr("feature/x", "develop", "T", "c").linea(),
        "abrir-pr ok pr=5"
    );
    assert_eq!(
        gh.abrir_pr("feature/x", "develop", "T", "c").linea(),
        "abrir-pr sin-cambios pr=5"
    );
    let posts: Vec<_> = mock
        .recibidas()
        .into_iter()
        .filter(|r| r.linea.starts_with("POST"))
        .collect();
    assert_eq!(posts.len(), 1);
    let cuerpo: serde_json::Value = serde_json::from_str(&posts[0].cuerpo).unwrap();
    assert_eq!(cuerpo["head"], "feature/x");
    assert_eq!(cuerpo["base"], "develop");

    // abrir-pr: un 422 «already exists» (carrera con otro proceso) relee.
    let mock = Mock::nuevo();
    mock.responder(m, r, Simulada::json(200, "[]"));
    mock.responder(
        m,
        r,
        Simulada::json(200, &format!("[{}]", pr(6, false, "open"))),
    );
    mock.responder(
        "POST",
        "/repos/o/r/pulls",
        Simulada::json(
            422,
            "{\"message\":\"Validation Failed\",\"errors\":[{\"message\":\"A pull request already exists for o:feature/x.\"}]}",
        ),
    );
    let gh = adaptador(&mock, &[Rol::Agente]);
    assert_eq!(
        gh.abrir_pr("feature/x", "develop", "T", "c").linea(),
        "abrir-pr sin-cambios pr=6"
    );

    // merge-pr: mergea con el sha evaluado y, ya mergeado, sin-cambios.
    let mock = Mock::nuevo();
    persona_en(&mock, PERSONA);
    mock.responder(
        "GET",
        "/repos/o/r/pulls/5",
        Simulada::json(200, &pr(5, false, "open")),
    );
    mock.responder(
        "GET",
        "/repos/o/r/pulls/5",
        Simulada::json(200, &pr(5, true, "closed")),
    );
    mock.responder(
        "PUT",
        "/repos/o/r/pulls/5/merge",
        Simulada::json(
            200,
            &format!("{{\"merged\":true,\"sha\":\"{}\"}}", "b".repeat(40)),
        ),
    );
    let gh = adaptador(&mock, &[Rol::Persona]);
    assert_eq!(
        gh.merge_pr(5, SHA).linea(),
        "merge-pr ok pr=5 commit=bbbbbbbbbbbb"
    );
    assert_eq!(
        gh.merge_pr(5, SHA).linea(),
        "merge-pr sin-cambios pr=5 estado=mergeado"
    );
    let put = mock
        .recibidas()
        .into_iter()
        .find(|r| r.linea.starts_with("PUT"))
        .unwrap();
    let cuerpo: serde_json::Value = serde_json::from_str(&put.cuerpo).unwrap();
    assert_eq!(
        (cuerpo["merge_method"].as_str(), cuerpo["sha"].as_str()),
        (Some("merge"), Some(SHA))
    );
    // `GET /user` una sola vez por rol y proceso.
    assert_eq!(
        mock.recibidas()
            .iter()
            .filter(|r| r.linea == "GET /user")
            .count(),
        1
    );

    // merge-pr: si el head ya no es el evaluado, no hay PUT.
    let mock = Mock::nuevo();
    persona_en(&mock, PERSONA);
    mock.responder(
        "GET",
        "/repos/o/r/pulls/5",
        Simulada::json(200, &pr(5, false, "open")),
    );
    let gh = adaptador(&mock, &[Rol::Persona]);
    let i = gh.merge_pr(5, &"c".repeat(40));
    assert_eq!(i.linea(), "merge-pr rechazada pr=5 motivo=head-cambiado");
    assert!(!mock.recibidas().iter().any(|r| r.linea.starts_with("PUT")));
}

/// Criterio 2: el envoltorio rechaza antes de la red lo que no es su destino
/// (el metodo es una lista cerrada por tipo: no hay forma de pedir otro), no
/// sigue redirecciones y no lee mas de 1 MiB.
#[test]
fn el_envoltorio_rechaza_antes_de_la_red() {
    for b in [
        "http://api.github.com",
        "http://localhost:1",
        "http://10.0.0.1:80",
        "https://u:p@h",
    ] {
        assert!(Base::nueva(b).is_err(), "{b}");
    }
    let mock = Mock::nuevo();
    let c = Cliente::nuevo(Base::nueva(&mock.base()).unwrap());
    let token = Secreto::nuevo(TOKEN);
    for ruta in [
        "/repos/o/../x",
        "/repos/o/%2e%2e/x",
        "/repos/o/%252e%252e/x",
        "/repos/o/./x",
        "//otro.example/x",
        "https://otro.example/x",
        "/repos/o\\x",
        "/repos/o/r?q=<script>",
    ] {
        let r = c.llamar(Metodo::Get, ruta, Some(&token), None);
        assert!(matches!(r, Err(FalloHttp::Rechazada(_))), "{ruta}: {r:?}");
    }
    assert!(
        mock.recibidas().is_empty(),
        "ninguna peticion llego al servidor"
    );
    assert!(c.registro().is_empty());

    // Redireccion: no se sigue (ni siquiera a la misma base).
    mock.responder(
        "GET",
        "/user",
        Simulada::json(302, "").con("Location", &format!("{}/otro", mock.base())),
    );
    let gh = adaptador(&mock, &[Rol::Agente]);
    let i = gh.identidad(Rol::Agente).unwrap_err();
    assert_eq!(i.linea(), "identidad error motivo=redireccion");
    assert_eq!(mock.recibidas().len(), 1);

    // Cuerpo de mas de 1 MiB.
    let mock = Mock::nuevo();
    let grande = format!(
        "{{\"login\":\"{}\",\"id\":1,\"type\":\"User\"}}",
        "x".repeat(1_100_000)
    );
    mock.responder("GET", "/user", Simulada::json(200, &grande));
    let gh = adaptador(&mock, &[Rol::Agente]);
    let i = gh.identidad(Rol::Agente).unwrap_err();
    assert_eq!(i.linea(), "identidad error motivo=respuesta-invalida");

    // Paginacion hacia otro host: se rechaza sin seguirla.
    let mock = Mock::nuevo();
    mock.responder(
        "GET",
        "/repos/o/r/pulls/12",
        Simulada::json(200, &pr(12, false, "open")),
    );
    mock.responder(
        "GET",
        "/repos/o/r/rules/branches/develop",
        Simulada::json(200, "[]"),
    );
    mock.responder(
        "GET",
        &format!("/repos/o/r/commits/{SHA}/check-runs"),
        Simulada::json(200, "{\"check_runs\":[]}").con(
            "Link",
            "<https://evil.example/repos/o/r/x?page=2>; rel=\"next\"",
        ),
    );
    let gh = adaptador(&mock, &[Rol::Agente]);
    let i = gh.estado_puertas(Rol::Agente, 12, &modelo(&[]));
    assert_eq!(i.linea(), "estado-puertas rechazada motivo=destino");
    assert_eq!(mock.recibidas().len(), 3);
}

/// Criterio 2: cada fallo HTTP da su linea, su codigo y un arreglo para la
/// persona; un rol sin token falla cerrado; el token no sale en ningun sitio.
#[test]
fn errores_dirigidos_a_la_persona() {
    let plan = "{\"message\":\"Upgrade to GitHub Pro or make this repository public to enable this feature.\"}";
    let casos: Vec<(Simulada, &str, u8, &str)> = vec![
        (
            Simulada::json(401, "{\"message\":\"Bad credentials\"}"),
            "identidad denegada motivo=credenciales rol=agente",
            8,
            "TC2_TOKEN_AGENTE",
        ),
        (
            Simulada::json(403, plan),
            "identidad rechazada motivo=plan",
            3,
            "publico",
        ),
        (
            Simulada::json(403, "{\"message\":\"API rate limit exceeded\"}")
                .con("x-ratelimit-remaining", "0")
                .con("retry-after", "42"),
            "identidad agotada motivo=limite-peticiones reintenta-en=42",
            5,
            "espera 42",
        ),
        (
            Simulada::json(429, "{}").con("retry-after", "7"),
            "identidad agotada motivo=limite-peticiones reintenta-en=7",
            5,
            "relanza",
        ),
        (
            Simulada::json(
                403,
                "{\"message\":\"Resource not accessible by integration\"}",
            ),
            "identidad denegada motivo=permisos rol=agente",
            8,
            "necesita",
        ),
        (
            Simulada::json(404, "{\"message\":\"Not Found\"}"),
            "identidad denegada motivo=no-existe-o-sin-acceso rol=agente",
            8,
            "alcance del token",
        ),
        (
            Simulada::json(500, "{\"message\":\"boom\\u0007\"}"),
            "identidad error motivo=http estado=500",
            1,
            "reintenta",
        ),
        (
            Simulada::json(200, "{\"login\":\"x\"}"),
            "identidad error motivo=respuesta-invalida",
            1,
            "forma esperada",
        ),
    ];
    for (respuesta, linea, codigo, arreglo) in casos {
        let mock = Mock::nuevo();
        mock.responder("GET", "/user", respuesta);
        let gh = adaptador(&mock, &[Rol::Agente]);
        let i = gh.identidad(Rol::Agente).unwrap_err();
        let s = i.salida(false);
        assert_eq!((s.stdout.trim_end(), s.codigo), (linea, codigo));
        assert!(s.stderr.contains(arreglo), "{linea}: {}", s.stderr);
        assert!(!s.stderr.contains('\u{7}'), "sin controles de la API");
        for texto in [
            s.stdout,
            s.stderr,
            i.salida(true).stdout,
            format!("{gh:?} {i:?}"),
        ] {
            assert!(!texto.contains(TOKEN), "el token no sale: {texto}");
        }
    }

    // Merge: 405 (no mergeable) y 409 (head cambiado en la plataforma).
    for (estado, linea) in [
        (405, "merge-pr rechazada pr=5 motivo=no-mergeable"),
        (409, "merge-pr rechazada pr=5 motivo=head-cambiado"),
    ] {
        let mock = Mock::nuevo();
        persona_en(&mock, PERSONA);
        mock.responder(
            "GET",
            "/repos/o/r/pulls/5",
            Simulada::json(200, &pr(5, false, "open")),
        );
        mock.responder(
            "PUT",
            "/repos/o/r/pulls/5/merge",
            Simulada::json(estado, "{\"message\":\"x\"}"),
        );
        let gh = adaptador(&mock, &[Rol::Persona]);
        assert_eq!(gh.merge_pr(5, SHA).linea(), linea);
    }

    // abrir-pr: 422 sin commits nuevos.
    let mock = Mock::nuevo();
    mock.responder("GET", "/repos/o/r/pulls", Simulada::json(200, "[]"));
    mock.responder(
        "POST",
        "/repos/o/r/pulls",
        Simulada::json(422, "{\"message\":\"Validation Failed\",\"errors\":[{\"message\":\"No commits between develop and feature/x\"}]}"),
    );
    let gh = adaptador(&mock, &[Rol::Agente]);
    assert_eq!(
        gh.abrir_pr("feature/x", "develop", "T", "").linea(),
        "abrir-pr rechazada motivo=sin-commits"
    );

    // Sin red: un puerto cerrado.
    let cerrado = {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };
    let gh = GitHub::nuevo(
        Base::nueva(&format!("http://127.0.0.1:{cerrado}")).unwrap(),
        "o/r",
        Box::new(CredencialesFijas::nuevas().con(Rol::Agente, Secreto::nuevo(TOKEN))),
        Personas::default(),
    )
    .unwrap();
    let s = gh.identidad(Rol::Agente).unwrap_err().salida(false);
    assert_eq!((s.stdout.as_str(), s.codigo), ("identidad sinred\n", 4));

    // Identidad: falta el token del rol (nunca se prueba con otro) y persona
    // que no esta en la lista.
    let mock = Mock::nuevo();
    let gh = adaptador(&mock, &[Rol::Agente]);
    assert_eq!(
        gh.merge_pr(5, SHA).linea(),
        "merge-pr rechazada motivo=identidad falta=persona"
    );
    assert_eq!(
        gh.leer_permisos_ci().linea(),
        "leer-permisos-ci rechazada motivo=identidad falta=persona"
    );
    assert!(mock.recibidas().is_empty(), "sin token no hay peticion");
    let mock = Mock::nuevo();
    persona_en(&mock, 4242);
    let gh = adaptador(&mock, &[Rol::Persona]);
    assert_eq!(
        gh.merge_pr(5, SHA).linea(),
        "merge-pr rechazada motivo=identidad rol=persona"
    );
    assert_eq!(
        gh.estado_puertas(Rol::Persona, 5, &modelo(&[])).linea(),
        "estado-puertas rechazada motivo=identidad rol=persona"
    );
    assert_eq!(mock.recibidas().len(), 1, "solo GET /user, una vez");
    let s = gh
        .estado_puertas(Rol::Revisor, 5, &modelo(&[]))
        .salida(false);
    assert_eq!(s.codigo, Resultado::Rechazada.codigo());
}
