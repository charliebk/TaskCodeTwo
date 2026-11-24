//! F2-6: puerto Plataforma y adaptador GitHub contra un servidor simulado en
//! 127.0.0.1 (sin red). La bateria de los 6 casos es la de la PoC M de F1-13
//! con sus fixtures (`tests/fixtures/plataforma/github`); F2-9 pasara la
//! misma con GitLab.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use comun::http_mock::{Mock, Simulada, serie};
use tc2::contrato::Resultado;
use tc2::plataforma::http::{Base, Cliente, FalloHttp, Metodo};
use tc2::plataforma::{
    CredencialesFijas, GitHub, GitLab, ModeloPuertas, Personas, Plataforma, Rol, Secreto,
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
    let _serie = serie();
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

    // estado-puertas sigue `Link rel=next` en la misma base y cuenta la
    // ultima review de cada persona: la aprobacion de la pagina 1 la anula
    // un cambio pedido en la pagina 2; la de un tercero fuera de la lista se
    // ignora.
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
        Simulada::json(200, "{\"check_runs\":[]}"),
    );
    let review = |estado: &str, id: u64| {
        format!(
            "{{\"state\":\"{estado}\",\"user\":{{\"login\":\"u{id}\",\"id\":{id},\"type\":\"User\"}}}}"
        )
    };
    mock.responder(
        "GET",
        "/repos/o/r/pulls/12/reviews?per_page=100",
        Simulada::json(
            200,
            &format!(
                "[{},{}]",
                review("APPROVED", PERSONA),
                review("APPROVED", 777)
            ),
        )
        .con(
            "Link",
            &format!(
                "<{}/repos/o/r/pulls/12/reviews?per_page=100&page=2>; rel=\"next\"",
                mock.base()
            ),
        ),
    );
    mock.responder(
        "GET",
        "/repos/o/r/pulls/12/reviews?per_page=100&page=2",
        Simulada::json(200, &format!("[{}]", review("CHANGES_REQUESTED", PERSONA))),
    );
    let gh = adaptador(&mock, &[Rol::Agente]);
    let sin_checks = ModeloPuertas::default();
    assert_eq!(
        gh.estado_puertas(Rol::Agente, 12, &sin_checks).linea(),
        "estado-puertas esperando pr=12 aprobaciones=0 ignoradas=1 falta=aprobacion"
    );
    assert_eq!(gh.registro().len(), 5, "{:?}", gh.registro());
}

/// Criterio 2: el envoltorio rechaza antes de la red lo que no es su destino
/// (el metodo es una lista cerrada por tipo: no hay forma de pedir otro), no
/// sigue redirecciones y no lee mas de 1 MiB.
#[test]
fn el_envoltorio_rechaza_antes_de_la_red() {
    let _serie = serie();
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
    let _serie = serie();
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
        // Limite secundario: 403 con `retry-after` y sin
        // `x-ratelimit-remaining` ni «rate limit» en el mensaje.
        (
            Simulada::json(403, "{}").con("retry-after", "30"),
            "identidad agotada motivo=limite-peticiones reintenta-en=30",
            5,
            "espera 30",
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

fn con_personas(mock: &Mock, roles: &[Rol], personas: &[u64]) -> GitHub {
    let mut c = CredencialesFijas::nuevas();
    for r in roles {
        c = c.con(*r, Secreto::nuevo(TOKEN));
    }
    GitHub::nuevo(
        Base::nueva(&mock.base()).unwrap(),
        "o/r",
        Box::new(c),
        Personas::nuevas(personas.iter().copied()),
    )
    .unwrap()
}

fn pr_con_estado(mergeable: &str) -> String {
    pr(12, false, "open").replace("\"clean\"", &format!("\"{mergeable}\""))
}

/// Servidor con un PR 12 abierto, sin reglas, y los check-runs y reviews
/// dados.
fn puertas(pr_json: &str, runs: &str, reviews: &str) -> Mock {
    let mock = Mock::nuevo();
    mock.responder("GET", "/repos/o/r/pulls/12", Simulada::json(200, pr_json));
    mock.responder(
        "GET",
        "/repos/o/r/rules/branches/develop",
        Simulada::json(200, "[]"),
    );
    mock.responder(
        "GET",
        &format!("/repos/o/r/commits/{SHA}/check-runs"),
        Simulada::json(200, &format!("{{\"check_runs\":{runs}}}")),
    );
    mock.responder(
        "GET",
        "/repos/o/r/pulls/12/reviews",
        Simulada::json(200, reviews),
    );
    mock
}

fn run(nombre: &str, app: u64, status: &str, conclusion: &str) -> String {
    format!(
        "{{\"name\":\"{nombre}\",\"app\":{{\"id\":{app}}},\"status\":\"{status}\",\"conclusion\":{conclusion}}}"
    )
}

fn review(estado: &str, login: &str, id: u64, tipo: &str) -> String {
    format!(
        "{{\"state\":\"{estado}\",\"user\":{{\"login\":\"{login}\",\"id\":{id},\"type\":\"{tipo}\"}}}}"
    )
}

/// Protecciones de F1-6 y F1-13 que el E2E no ejerce, con un caso que se
/// pone rojo si se quitan: origen del check (G09), persona = `User` de la
/// lista y no `web-flow` (reglas 3, 4 y 6), contrato del firmante, la CI no
/// aprueba (G16), lectura de reglas con aviso de plan, y los bordes de
/// `estado-puertas` y `merge-pr`.
#[test]
fn protecciones_y_bordes() {
    let _serie = serie();
    let verdes = format!(
        "[{},{}]",
        run("verificacion", 15368, "completed", "\"success\""),
        run("carril", 15368, "completed", "\"neutral\"")
    );
    let modelo_ia = modelo(&[]);

    // G09: un `revision-ia` verde publicado por otra App no cuenta.
    let runs = format!(
        "[{},{},{}]",
        run("verificacion", 15368, "completed", "\"success\""),
        run("carril", 15368, "completed", "\"skipped\""),
        run("revision-ia", 55555, "completed", "\"success\"")
    );
    let aprobada = format!("[{}]", review("APPROVED", "carlos", PERSONA, "User"));
    let mock = puertas(&pr(12, false, "open"), &runs, &aprobada);
    assert_eq!(
        adaptador(&mock, &[Rol::Agente])
            .estado_puertas(Rol::Agente, 12, &modelo_ia)
            .linea(),
        "estado-puertas esperando pr=12 aprobaciones=1 ignoradas=0 falta=revision-ia"
    );
    // ... y uno de su App en rojo deja el check rojo.
    let runs = format!(
        "[{}]",
        run("revision-ia", 99001, "completed", "\"failure\"")
    );
    let mock = puertas(&pr(12, false, "open"), &runs, &aprobada);
    assert_eq!(
        adaptador(&mock, &[Rol::Agente])
            .estado_puertas(Rol::Agente, 12, &modelo_ia)
            .linea(),
        "estado-puertas rechazada pr=12 motivo=check-rojo checks=revision-ia"
    );

    // Reglas 3, 4 y 6: `web-flow` y un Bot no cuentan aunque esten en la
    // lista; un comentario posterior no retira una aprobacion.
    let reviews = format!(
        "[{},{},{},{}]",
        review("APPROVED", "web-flow", 19864447, "User"),
        review("APPROVED", "app[bot]", 2001, "Bot"),
        review("APPROVED", "carlos", PERSONA, "User"),
        review("COMMENTED", "carlos", PERSONA, "User")
    );
    let mock = puertas(&pr(12, false, "open"), &verdes, &reviews);
    let gh = con_personas(&mock, &[Rol::Agente], &[PERSONA, 19864447, 2001]);
    let sin_ia = ModeloPuertas {
        checks: vec![("verificacion".into(), 15368), ("carril".into(), 15368)],
        exigir_impuestas: Vec::new(),
    };
    assert_eq!(
        gh.estado_puertas(Rol::Agente, 12, &sin_ia).linea(),
        "estado-puertas ok pr=12 aprobaciones=1 ignoradas=2"
    );

    // Rama por detras de la base y con conflictos.
    for (estado, falta) in [("behind", "al-dia"), ("dirty", "sin-conflictos")] {
        let mock = puertas(&pr_con_estado(estado), &verdes, &aprobada);
        assert_eq!(
            adaptador(&mock, &[Rol::Agente])
                .estado_puertas(Rol::Agente, 12, &sin_ia)
                .linea(),
            format!("estado-puertas esperando pr=12 aprobaciones=1 ignoradas=0 falta={falta}")
        );
    }

    // PR cerrado sin mergear: estado-puertas y merge-pr lo dicen.
    let mock = Mock::nuevo();
    persona_en(&mock, PERSONA);
    mock.responder(
        "GET",
        "/repos/o/r/pulls/12",
        Simulada::json(200, &pr(12, false, "closed")),
    );
    let gh = adaptador(&mock, &[Rol::Agente, Rol::Persona]);
    assert_eq!(
        gh.estado_puertas(Rol::Agente, 12, &sin_ia).linea(),
        "estado-puertas rechazada pr=12 motivo=pr-cerrado"
    );
    assert_eq!(
        gh.merge_pr(12, SHA).linea(),
        "merge-pr rechazada pr=12 motivo=pr-cerrado"
    );
    // sha incompleto: uso, sin llamar a la API.
    let antes = mock.recibidas().len();
    assert_eq!(
        gh.merge_pr(12, "abc").linea(),
        "merge-pr uso motivo=sha-invalido"
    );
    assert_eq!(mock.recibidas().len(), antes);

    // Merge hecho cuya respuesta no trae sha: ok, sin inventar el commit.
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
        Simulada::json(200, "{\"merged\":true,\"sha\":null}"),
    );
    assert_eq!(
        adaptador(&mock, &[Rol::Persona]).merge_pr(5, SHA).linea(),
        "merge-pr ok pr=5"
    );

    // G16: la CI no aprueba.
    for (puede, linea) in [
        (false, "leer-permisos-ci ok ci-aprueba=false permisos=read"),
        (
            true,
            "leer-permisos-ci rechazada motivo=ci-puede-aprobar permisos=read",
        ),
    ] {
        let mock = Mock::nuevo();
        persona_en(&mock, PERSONA);
        mock.responder(
            "GET",
            "/repos/o/r/actions/permissions/workflow",
            Simulada::json(
                200,
                &format!("{{\"default_workflow_permissions\":\"read\",\"can_approve_pull_request_reviews\":{puede}}}"),
            ),
        );
        assert_eq!(
            adaptador(&mock, &[Rol::Persona]).leer_permisos_ci().linea(),
            linea
        );
    }

    // Contrato del firmante (reglas 1 a 4).
    let commit = |verificado: bool, committer: &str| {
        format!(
            "{{\"commit\":{{\"verification\":{{\"verified\":{verificado},\"reason\":\"valid\"}}}},\"committer\":{committer}}}"
        )
    };
    let persona = "{\"login\":\"carlos\",\"id\":1001,\"type\":\"User\"}";
    let webflow = "{\"login\":\"web-flow\",\"id\":19864447,\"type\":\"User\"}";
    let bot = "{\"login\":\"app[bot]\",\"id\":2001,\"type\":\"Bot\"}";
    for (verificado, committer, cuenta) in [
        (true, persona, true),
        (false, persona, false),
        (true, webflow, false),
        (true, bot, false),
        (true, "null", false),
    ] {
        let mock = Mock::nuevo();
        mock.responder(
            "GET",
            &format!("/repos/o/r/commits/{SHA}"),
            Simulada::json(200, &commit(verificado, committer)),
        );
        let gh = con_personas(&mock, &[Rol::Agente], &[PERSONA, 19864447, 2001]);
        let f = gh.firma_commit(SHA).unwrap();
        assert_eq!(
            f.cuenta_como_aprobacion(&Personas::nuevas([PERSONA, 19864447, 2001])),
            cuenta,
            "{verificado} {committer}"
        );
    }

    // leer-puertas: con aviso de plan no hay reglas y no se sigue leyendo.
    let mock = Mock::nuevo();
    mock.responder(
        "GET",
        "/repos/o/r/rulesets",
        Simulada::json(403, "{\"message\":\"Upgrade to GitHub Pro or make this repository public to enable this feature.\"}"),
    );
    let gh = adaptador(&mock, &[Rol::Agente]);
    let r = gh.leer_puertas(Rol::Agente, "develop").unwrap();
    assert!(!r.plan_da_reglas && r.rulesets.is_empty());
    assert_eq!(mock.recibidas().len(), 1);
    let mock = Mock::nuevo();
    mock.responder(
        "GET",
        "/repos/o/r/rulesets",
        Simulada::json(200, "[{\"id\":1,\"name\":\"taskcodetwo-develop\"}]"),
    );
    mock.responder(
        "GET",
        "/repos/o/r/rules/branches/develop",
        Simulada::json(200, "[{\"type\":\"pull_request\"}]"),
    );
    let gh = adaptador(&mock, &[Rol::Agente, Rol::Revisor]);
    let r = gh.leer_puertas(Rol::Agente, "develop").unwrap();
    assert_eq!(
        (r.plan_da_reglas, r.rulesets, r.reglas_rama),
        (
            true,
            vec!["taskcodetwo-develop".to_owned()],
            vec!["pull_request".to_owned()]
        )
    );
    // Con token de revisor: lo rechaza el rol, no la falta de token, y sin
    // llamar.
    let antes = mock.recibidas().len();
    assert_eq!(
        gh.leer_puertas(Rol::Revisor, "develop")
            .unwrap_err()
            .linea(),
        "leer-puertas rechazada motivo=identidad rol=revisor"
    );
    assert_eq!(mock.recibidas().len(), antes, "el revisor no llama");

    // Limite primario (con reinicio) y secundario (sin cabeceras).
    let reinicio = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 600;
    let mock = Mock::nuevo();
    mock.responder(
        "GET",
        "/user",
        Simulada::json(403, "{\"message\":\"API rate limit exceeded\"}")
            .con("x-ratelimit-remaining", "0")
            .con("x-ratelimit-reset", &reinicio.to_string()),
    );
    let i = adaptador(&mock, &[Rol::Agente])
        .identidad(Rol::Agente)
        .unwrap_err();
    let espera: u64 = i.valor("reintenta-en").unwrap().parse().unwrap();
    assert!((590..=600).contains(&espera), "{}", i.linea());
    let mock = Mock::nuevo();
    mock.responder(
        "GET",
        "/user",
        Simulada::json(
            403,
            "{\"message\":\"You have exceeded a secondary rate limit.\"}",
        ),
    );
    let i = adaptador(&mock, &[Rol::Agente])
        .identidad(Rol::Agente)
        .unwrap_err();
    assert_eq!(
        i.linea(),
        "identidad agotada motivo=limite-peticiones reintenta-en=60"
    );

    // Tope de paginas: una lista que no acaba no se evalua a medias.
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
        Simulada::json(200, "{\"check_runs\":[]}"),
    );
    mock.responder(
        "GET",
        "/repos/o/r/pulls/12/reviews",
        Simulada::json(200, "[]").con(
            "Link",
            &format!(
                "<{}/repos/o/r/pulls/12/reviews?page=2>; rel=\"next\"",
                mock.base()
            ),
        ),
    );
    let gh = adaptador(&mock, &[Rol::Agente]);
    let i = gh.estado_puertas(Rol::Agente, 12, &sin_ia);
    assert_eq!(i.linea(), "estado-puertas error motivo=demasiadas-paginas");
    let paginas = gh
        .registro()
        .iter()
        .filter(|l| l.contains("/reviews"))
        .count();
    assert_eq!(paginas, 10);
}

/// A1-07: `leer-puertas` sigue `Link rel=next` en los rulesets. Con 100 que
/// no son de tc2 en la primera pagina y `taskcodetwo-develop` en la
/// segunda, lo devuelve; y en lote igual.
#[test]
fn leer_puertas_pagina_los_rulesets() {
    let _serie = serie();
    let mock = Mock::nuevo();
    let pagina1: Vec<String> = (1..=100)
        .map(|n| format!("{{\"id\":{n},\"name\":\"otro-{n}\"}}"))
        .collect();
    let siguiente = "/repos/o/r/rulesets?per_page=100&page=2";
    mock.responder(
        "GET",
        siguiente,
        Simulada::json(200, "[{\"id\":101,\"name\":\"taskcodetwo-develop\"}]"),
    );
    mock.responder(
        "GET",
        "/repos/o/r/rulesets",
        Simulada::json(200, &format!("[{}]", pagina1.join(","))).con(
            "link",
            &format!("<{}{siguiente}>; rel=\"next\"", mock.base()),
        ),
    );
    mock.responder(
        "GET",
        "/repos/o/r/rules/branches/develop",
        Simulada::json(200, "[{\"type\":\"pull_request\"}]"),
    );
    let gh = adaptador(&mock, &[Rol::Agente]);
    let r = gh.leer_puertas(Rol::Agente, "develop").unwrap();
    assert_eq!(r.rulesets.len(), 101);
    assert!(
        r.rulesets.iter().any(|n| n == "taskcodetwo-develop"),
        "{:?}",
        r.rulesets
    );
    let lote = gh.leer_puertas_de(Rol::Agente, &["develop"], "");
    assert!(
        lote[0]
            .as_ref()
            .is_ok_and(|r| r.rulesets.iter().any(|n| n == "taskcodetwo-develop"))
    );
}

/// TASK-118: `aplicar-puertas` en el adaptador. Un 422 al crear es «otro
/// llego antes» (se relee y cuenta como existente), un fallo a mitad corta
/// con `aplicadas=` y `faltan=`, y repetirla converge sin reescribir lo ya
/// creado. Sin rulesets en el plan, `rechazada motivo=plan` sin escribir.
#[test]
fn aplicar_puertas_carrera_corte_y_plan() {
    use tc2::plataforma::Proveedor;
    use tc2::plataforma::puertas::{modelo as puertas, origenes};
    let _serie = serie();
    let o = origenes(Proveedor::GitHub, Some(99001)).unwrap();
    let checks = r#"{"total_count":3,"check_runs":[
        {"name":"verificacion","status":"completed","conclusion":"success","app":{"id":15368}},
        {"name":"carril","status":"completed","conclusion":"success","app":{"id":15368}},
        {"name":"revision-ia","status":"completed","conclusion":"success","app":{"id":99001}}]}"#;
    let mock = Mock::nuevo();
    persona_en(&mock, PERSONA);
    mock.responder(
        "GET",
        "/repos/o/r/commits/heads/develop/check-runs",
        Simulada::json(200, checks),
    );
    // Lectura previa (vacia) y la relectura tras el 422 (otro creo develop).
    mock.responder("GET", "/repos/o/r/rulesets", Simulada::json(200, "[]"));
    mock.responder(
        "GET",
        "/repos/o/r/rulesets",
        Simulada::json(200, r#"[{"id":1,"name":"taskcodetwo-develop"}]"#),
    );
    for estado in [422, 201, 500] {
        mock.responder(
            "POST",
            "/repos/o/r/rulesets",
            Simulada::json(estado, r#"{"message":"x"}"#),
        );
    }
    let gh = adaptador(&mock, &[Rol::Persona]);
    let i = gh.aplicar_puertas(&puertas(), &o);
    assert_eq!(
        i.linea(),
        "aplicar-puertas error motivo=http estado=500 aplicadas=taskcodetwo-main faltan=taskcodetwo-ramas-de-tarea,taskcodetwo-tareas,taskcodetwo-tags-version"
    );
    assert!(i.diagnostico.is_some());

    // Repetirla: lo creado sale como existente y solo se crea lo que falta.
    let mock = Mock::nuevo();
    persona_en(&mock, PERSONA);
    mock.responder(
        "GET",
        "/repos/o/r/commits/heads/develop/check-runs",
        Simulada::json(200, checks),
    );
    mock.responder(
        "GET",
        "/repos/o/r/rulesets",
        Simulada::json(
            200,
            r#"[{"id":1,"name":"taskcodetwo-develop"},{"id":2,"name":"taskcodetwo-main"}]"#,
        ),
    );
    mock.responder("POST", "/repos/o/r/rulesets", Simulada::json(201, "{}"));
    let gh = adaptador(&mock, &[Rol::Persona]);
    let i = gh.aplicar_puertas(&puertas(), &o);
    assert_eq!(
        i.linea(),
        "aplicar-puertas ok aplicadas=taskcodetwo-ramas-de-tarea,taskcodetwo-tareas,taskcodetwo-tags-version existentes=taskcodetwo-develop,taskcodetwo-main"
    );
    let escrituras: Vec<String> = gh
        .registro()
        .into_iter()
        .filter(|l| !l.starts_with("GET "))
        .collect();
    assert_eq!(escrituras, vec!["POST /repos/o/r/rulesets".to_owned(); 3]);

    // Sin rulesets en el plan: rechazada antes de escribir.
    let mock = Mock::nuevo();
    persona_en(&mock, PERSONA);
    mock.responder(
        "GET",
        "/repos/o/r/rulesets",
        Simulada::json(403, r#"{"message":"Upgrade to GitHub Pro or make this repository public to enable this feature."}"#),
    );
    let gh = adaptador(&mock, &[Rol::Persona]);
    let i = gh.aplicar_puertas(&puertas(), &o);
    assert_eq!(i.linea(), "aplicar-puertas rechazada motivo=plan");
    assert!(gh.registro().iter().all(|l| l.starts_with("GET ")));
}

/// Un aceptado como lo deja `tc2 hallazgos veredicto` en `raw_details`.
fn crudo(id: &str) -> String {
    serde_json::json!({
        "revisora": "rev:a",
        "hallazgo": {
            "id": id, "severidad": "IMPORTANTE", "clase": "contrato",
            "fichero": "src/a.rs", "linea": 3, "descripcion": "d",
            "evidencia": {"tipo": "borde", "entrada": "x", "esperado": "1", "obtenido": "2"}
        }
    })
    .to_string()
}

fn anotacion(raw: &str) -> String {
    serde_json::json!({"path": "src/a.rs", "start_line": 3, "raw_details": raw}).to_string()
}

/// F6-5, `leer-revision` (TASK-051): el check `revision-ia` de su origen
/// (no el de otra App con el mismo nombre), sus anotaciones en dos paginas,
/// un `raw_details` que no es un aceptado falla cerrado, un check sin
/// terminar espera, y GitLab no lo soporta.
#[test]
fn leer_revision_filtra_por_origen_y_pagina() {
    let _serie = serie();
    let runs = format!(
        "[{},{},{}]",
        "{\"id\":4,\"name\":\"revision-ia\",\"app\":{\"id\":1},\"status\":\"completed\",\"conclusion\":\"success\",\"output\":{\"title\":\"impostor\",\"summary\":\"x\"}}",
        "{\"id\":5,\"name\":\"revision-ia\",\"app\":{\"id\":99001},\"status\":\"completed\",\"conclusion\":\"failure\",\"output\":{\"title\":\"revision-ia: rojo (criticos=0 importantes=2 menores=0)\",\"summary\":\"resumen\"}}",
        run("verificacion", 15368, "completed", "\"success\""),
    );
    let mock = puertas(&pr(12, false, "open"), &runs, "[]");
    let base = mock.base();
    mock.responder(
        "GET",
        "/repos/o/r/check-runs/5/annotations?per_page=100",
        Simulada::json(200, &format!("[{}]", anotacion(&crudo("I1")))).con(
            "Link",
            &format!(
                "<{base}/repos/o/r/check-runs/5/annotations?per_page=100&page=2>; rel=\"next\""
            ),
        ),
    );
    mock.responder(
        "GET",
        "/repos/o/r/check-runs/5/annotations?per_page=100&page=2",
        Simulada::json(200, &format!("[{}]", anotacion(&crudo("I2")))),
    );
    let gh = adaptador(&mock, &[Rol::Agente]);
    let r = gh
        .leer_revision(Rol::Agente, 12, 99001)
        .unwrap_or_else(|i| panic!("{}", i.linea()));
    assert_eq!(r.sha, SHA);
    assert_eq!(r.conclusion, "failure");
    assert_eq!(
        r.titulo,
        "revision-ia: rojo (criticos=0 importantes=2 menores=0)"
    );
    assert_eq!(r.resumen, "resumen");
    let ids: Vec<&str> = r.aceptados.iter().map(|a| a.hallazgo.id.as_str()).collect();
    assert_eq!(ids, ["I1", "I2"]);
    assert_eq!(r.aceptados[0].revisora, "rev:a");
    // Nunca las anotaciones del impostor.
    assert!(
        !mock
            .recibidas()
            .iter()
            .any(|q| q.linea.contains("check-runs/4/")),
        "{:?}",
        mock.recibidas()
    );
    // El revisor no lee con este puerto.
    let i = gh.leer_revision(Rol::Revisor, 12, 99001).unwrap_err();
    assert_eq!(
        i.linea(),
        "leer-revision rechazada motivo=identidad rol=revisor"
    );

    // Un raw_details con una clave de mas, o sin raw_details: respuesta
    // invalida.
    for malo in [
        anotacion(&crudo("I3").replace("\"revisora\"", "\"x\":1,\"revisora\"")),
        "{\"path\":\"src/a.rs\"}".to_owned(),
    ] {
        let mock = puertas(&pr(12, false, "open"), &runs, "[]");
        mock.responder(
            "GET",
            "/repos/o/r/check-runs/5/annotations",
            Simulada::json(200, &format!("[{malo}]")),
        );
        let gh = adaptador(&mock, &[Rol::Agente]);
        let i = gh.leer_revision(Rol::Agente, 12, 99001).unwrap_err();
        assert_eq!(
            i.linea(),
            "leer-revision error motivo=respuesta-invalida",
            "{malo}"
        );
        let d = i.diagnostico_en(tc2::i18n::Idioma::Es).unwrap_or_default();
        assert!(d.contains("raw_details"), "{d}");
    }

    // Sin terminar (o de otro origen): esperando, con que hacer.
    let pendiente = runs.replacen(
        "\"status\":\"completed\",\"conclusion\":\"failure\"",
        "\"status\":\"in_progress\",\"conclusion\":null",
        1,
    );
    assert_ne!(pendiente, runs);
    let mock = puertas(&pr(12, false, "open"), &pendiente, "[]");
    let gh = adaptador(&mock, &[Rol::Agente]);
    let i = gh.leer_revision(Rol::Agente, 12, 99001).unwrap_err();
    assert_eq!(i.linea(), "leer-revision esperando pr=12 motivo=sin-check");
    let d = i.diagnostico_en(tc2::i18n::Idioma::Es).unwrap_or_default();
    assert!(d.contains("tc2 review") && d.contains("aaaaaaa"), "{d}");
    let mock = puertas(&pr(12, false, "open"), &runs, "[]");
    let gh = adaptador(&mock, &[Rol::Agente]);
    let i = gh.leer_revision(Rol::Agente, 12, 424242).unwrap_err();
    assert_eq!(i.linea(), "leer-revision esperando pr=12 motivo=sin-check");

    // GitLab no tiene revision en CI: no soportada.
    let gl = GitLab::nuevo(
        Base::nueva(&mock.base()).unwrap(),
        "o/r",
        Box::new(CredencialesFijas::nuevas().con(Rol::Agente, Secreto::nuevo(TOKEN))),
        Personas::default(),
    )
    .unwrap();
    let i = gl.leer_revision(Rol::Agente, 12, 99001).unwrap_err();
    assert_eq!(i.linea(), "leer-revision rechazada motivo=no-soportada");
}
