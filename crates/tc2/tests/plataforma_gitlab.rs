//! F2-9: adaptador GitLab del puerto Plataforma contra un servidor simulado
//! en 127.0.0.1 (sin red). La bateria es la misma de F2-6 (PoC M de F1-13):
//! los fixtures de GitHub y de GitLab tienen que dar la misma linea byte a
//! byte (RR-15).

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use comun::adaptadores::{self, servir};
use comun::http_mock::{Mock, Simulada, serie};
use tc2::contrato::Resultado;
use tc2::plataforma::http::{Base, Cliente, FalloHttp, Metodo};
use tc2::plataforma::{
    Clave, CredencialesFijas, GitHub, GitLab, ModeloPuertas, Personas, Plataforma, Proveedor, Rol,
    Secreto,
};

const TOKEN: &str = "glpat_token_de_prueba_que_no_debe_salir";
const SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SHA_MERGE: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const PERSONA: u64 = 1001;
const CLAVE: u64 = 555;
const P: &str = "/api/v4/projects/7";

fn base(mock: &Mock) -> Base {
    Base::nueva(&format!("{}/api/v4", mock.base())).unwrap()
}

fn gitlab(mock: &Mock, roles: &[Rol]) -> GitLab {
    adaptadores::gitlab(
        &mock.base(),
        TOKEN,
        roles,
        Personas::nuevas([PERSONA]).con_claves([Clave::Ssh(CLAVE)]),
    )
}

fn github(mock: &Mock) -> GitHub {
    adaptadores::github(
        &mock.base(),
        TOKEN,
        &[Rol::Agente],
        Personas::nuevas([PERSONA]),
    )
}

/// El modelo de cada plataforma: el origen de un check es el
/// `integration_id` (GitHub) o el `author.id` del estado (GitLab).
fn modelo(p: Proveedor, exigir: &[&str]) -> ModeloPuertas {
    let (ci, revisor) = match p {
        Proveedor::GitHub => (15368, 99001),
        Proveedor::GitLab => (2001, 2002),
    };
    ModeloPuertas {
        checks: vec![
            ("verificacion".into(), ci),
            ("carril".into(), ci),
            ("revision-ia".into(), revisor),
        ],
        exigir_impuestas: exigir.iter().map(|g| (*g).to_owned()).collect(),
    }
}

fn fixture(plataforma: &str, caso: &str) -> serde_json::Value {
    let ruta = format!(
        "{}/tests/fixtures/plataforma/{plataforma}/{caso}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(ruta).unwrap()).unwrap()
}

fn exigidas(f: &serde_json::Value) -> Vec<&str> {
    f["politica"]["exigir_impuestas"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g.as_str().unwrap())
        .collect()
}

/// Corre `estado-puertas` con los fixtures de las dos plataformas, quizas
/// retocados, y devuelve `(linea, codigo, llamadas)` de cada una.
fn las_dos(
    caso: &str,
    retoque_gh: impl Fn(&mut serde_json::Value),
    retoque_gl: impl Fn(&mut serde_json::Value),
) -> [(String, u8, usize); 2] {
    let mut fh = fixture("github", caso);
    retoque_gh(&mut fh);
    let mock = servir(&fh);
    let gh = github(&mock);
    let i = gh.estado_puertas(Rol::Agente, 12, &modelo(Proveedor::GitHub, &exigidas(&fh)));
    let a = (i.linea(), i.resultado.codigo(), gh.registro().len());
    let mut fl = fixture("gitlab", caso);
    retoque_gl(&mut fl);
    let mock = servir(&fl);
    let gl = gitlab(&mock, &[Rol::Agente]);
    let i = gl.estado_puertas(Rol::Agente, 12, &modelo(Proveedor::GitLab, &exigidas(&fl)));
    assert_eq!(
        gl.registro().len(),
        mock.recibidas().len(),
        "{caso}: lo registrado es lo que llego"
    );
    [a, (i.linea(), i.resultado.codigo(), gl.registro().len())]
}

fn mr_de(f: &mut serde_json::Value) -> &mut serde_json::Value {
    &mut f["respuestas"][format!("{P}/merge_requests/12?include_diverged_commits_count=true")]["cuerpo"]
}

/// Criterio 2 (RR-15): la misma bateria que F2-6 da la misma linea y el mismo
/// codigo en GitHub y en GitLab, con 4 y 5 lecturas; tambien `al-dia` y
/// `sin-conflictos`. Y `abrir-pr` y `merge-pr` de GitLab son idempotentes.
#[test]
fn bateria_compartida_con_github_e_idempotencia() {
    let _serie = serie();
    let casos = [
        (
            "verde",
            "estado-puertas ok pr=12 aprobaciones=1 ignoradas=0",
            0,
            4,
            5,
        ),
        (
            "esperando-checks",
            "estado-puertas esperando pr=12 aprobaciones=1 ignoradas=0 falta=revision-ia",
            7,
            4,
            5,
        ),
        (
            "sin-aprobacion",
            "estado-puertas esperando pr=12 aprobaciones=0 ignoradas=0 falta=aprobacion",
            7,
            4,
            5,
        ),
        (
            "aprobacion-bot",
            "estado-puertas esperando pr=12 aprobaciones=0 ignoradas=1 falta=aprobacion",
            7,
            4,
            5,
        ),
        (
            "plan",
            "estado-puertas rechazada pr=12 motivo=plan garantia=G03",
            3,
            2,
            2,
        ),
        (
            "sin-cambios",
            "estado-puertas sin-cambios pr=12 estado=mergeado",
            0,
            1,
            1,
        ),
    ];
    for (caso, linea, codigo, n_gh, n_gl) in casos {
        let [gh, gl] = las_dos(caso, |_| {}, |_| {});
        assert_eq!(gh, (linea.to_owned(), codigo, n_gh), "{caso} en GitHub");
        assert_eq!(gl, (linea.to_owned(), codigo, n_gl), "{caso} en GitLab");
    }

    // Rama por detras y con conflictos, en las dos.
    let pr_gh = |estado: &'static str| {
        move |f: &mut serde_json::Value| {
            f["respuestas"]["/repos/o/r/pulls/12"]["cuerpo"]["mergeable_state"] = estado.into();
        }
    };
    let [gh, gl] = las_dos("verde", pr_gh("behind"), |f| {
        mr_de(f)["diverged_commits_count"] = 3.into();
    });
    let linea = "estado-puertas esperando pr=12 aprobaciones=1 ignoradas=0 falta=al-dia";
    assert_eq!((gh.0.as_str(), gh.1), (linea, 7));
    assert_eq!((gl.0.as_str(), gl.1), (linea, 7));
    let [gh, gl] = las_dos("verde", pr_gh("dirty"), |f| {
        mr_de(f)["has_conflicts"] = true.into();
    });
    let linea = "estado-puertas esperando pr=12 aprobaciones=1 ignoradas=0 falta=sin-conflictos";
    assert_eq!((gh.0.as_str(), gh.1), (linea, 7));
    assert_eq!((gl.0.as_str(), gl.1), (linea, 7));

    // abrir-pr: el MR ya abierto no se duplica.
    let mock = Mock::nuevo();
    mock.responder(
        "GET",
        &format!("{P}/merge_requests"),
        Simulada::json(
            200,
            "[{\"iid\":12,\"state\":\"opened\",\"source_branch\":\"feature/x\",\"target_branch\":\"develop\"}]",
        ),
    );
    let gl = gitlab(&mock, &[Rol::Agente]);
    assert_eq!(
        gl.abrir_pr("feature/x", "develop", "T", "").linea(),
        "abrir-pr sin-cambios pr=12"
    );
    assert_eq!(
        mock.recibidas()[0].linea,
        format!(
            "GET {P}/merge_requests?source_branch=feature%2Fx&target_branch=develop&state=opened&per_page=100"
        )
    );
    assert!(
        mock.recibidas()
            .iter()
            .all(|r| !r.linea.starts_with("POST"))
    );

    // abrir-pr: sin MR, con commits nuevos, lo crea; con 409 de carrera,
    // relee y devuelve el que abrio el otro.
    for (estado_post, linea) in [
        (201, "abrir-pr ok pr=13"),
        (409, "abrir-pr sin-cambios pr=13"),
    ] {
        let mock = Mock::nuevo();
        let ruta = format!("{P}/merge_requests");
        mock.responder("GET", &ruta, Simulada::json(200, "[]"));
        mock.responder(
            "GET",
            &ruta,
            Simulada::json(
                200,
                "[{\"iid\":13,\"state\":\"opened\",\"source_branch\":\"feature/x\",\"target_branch\":\"develop\"}]",
            ),
        );
        mock.responder(
            "GET",
            &format!("{P}/repository/commits"),
            Simulada::json(200, &format!("[{{\"id\":\"{SHA}\"}}]")),
        );
        mock.responder(
            "POST",
            &ruta,
            Simulada::json(
                estado_post,
                "{\"iid\":13,\"state\":\"opened\",\"source_branch\":\"feature/x\",\"target_branch\":\"develop\",\"message\":[\"Another open merge request already exists for this source branch: !13\"]}",
            ),
        );
        let gl = gitlab(&mock, &[Rol::Agente]);
        assert_eq!(gl.abrir_pr("feature/x", "develop", "T", "").linea(), linea);
        let post = mock
            .recibidas()
            .into_iter()
            .find(|r| r.linea.starts_with("POST"))
            .unwrap();
        let datos: serde_json::Value = serde_json::from_str(&post.cuerpo).unwrap();
        assert_eq!(datos["source_branch"], "feature/x");
        assert_eq!(datos["target_branch"], "develop");
        assert!(
            mock.recibidas()
                .iter()
                .any(|r| r.linea.contains("ref_name=develop..feature%2Fx")),
            "comprueba los commits nuevos antes de crear"
        );
    }

    // merge-pr: mergea con el sha evaluado; ya mergeado, sin-cambios.
    for (estado_mr, linea, puts) in [
        ("opened", "merge-pr ok pr=12 commit=bbbbbbbbbbbb", 1),
        ("merged", "merge-pr sin-cambios pr=12 estado=mergeado", 0),
    ] {
        let mock = Mock::nuevo();
        persona(&mock, PERSONA, false);
        mock.responder(
            "GET",
            &format!("{P}/merge_requests/12"),
            Simulada::json(200, &mr(12, estado_mr, SHA)),
        );
        mock.responder(
            "PUT",
            &format!("{P}/merge_requests/12/merge"),
            Simulada::json(
                200,
                &format!("{{\"state\":\"merged\",\"merge_commit_sha\":\"{SHA_MERGE}\"}}"),
            ),
        );
        let gl = gitlab(&mock, &[Rol::Persona]);
        assert_eq!(gl.merge_pr(12, SHA).linea(), linea);
        let put: Vec<_> = mock
            .recibidas()
            .into_iter()
            .filter(|r| r.linea.starts_with("PUT"))
            .collect();
        assert_eq!(put.len(), puts);
        if let Some(p) = put.first() {
            let d: serde_json::Value = serde_json::from_str(&p.cuerpo).unwrap();
            assert_eq!(d["sha"], SHA);
            assert_eq!(d["squash"], false);
            assert_eq!(d["should_remove_source_branch"], false);
        }
    }
}

fn persona(mock: &Mock, id: u64, bot: bool) {
    mock.responder(
        "GET",
        "/api/v4/user",
        Simulada::json(
            200,
            &format!("{{\"username\":\"carlos\",\"id\":{id},\"bot\":{bot}}}"),
        ),
    );
}

fn mr(iid: u64, estado: &str, sha: &str) -> String {
    format!(
        "{{\"iid\":{iid},\"state\":\"{estado}\",\"sha\":\"{sha}\",\"source_branch\":\"feature/x\",\"target_branch\":\"develop\"}}"
    )
}

/// Criterio 2: el envoltorio habla GitLab (cabeceras, `%2F` exacto, prefijo
/// `/api/v4`) y los errores llegan dirigidos a la persona, sin el token.
#[test]
fn envoltorio_y_errores_de_gitlab() {
    let _serie = serie();
    // Cabeceras y proyecto con nombre, codificado una vez.
    let mock = Mock::nuevo();
    mock.responder(
        "GET",
        "/api/v4/projects/grupo%2Fsub%2Fproy/merge_requests",
        Simulada::json(200, "[]"),
    );
    mock.responder(
        "GET",
        "/api/v4/projects/grupo%2Fsub%2Fproy/repository/commits",
        Simulada::json(200, "[]"),
    );
    let gl = GitLab::nuevo(
        base(&mock),
        "grupo/sub/proy",
        adaptadores::credenciales(TOKEN, &[Rol::Agente]),
        Personas::default(),
    )
    .unwrap();
    assert_eq!(
        gl.abrir_pr("feature/x", "develop", "T", "").linea(),
        "abrir-pr rechazada motivo=sin-commits"
    );
    let r = &mock.recibidas()[0];
    assert!(
        r.linea
            .starts_with("GET /api/v4/projects/grupo%2Fsub%2Fproy/")
    );
    assert_eq!(
        r.autorizacion.as_deref(),
        Some(format!("Bearer {TOKEN}").as_str())
    );
    let cab = |k: &str| {
        r.cabeceras
            .iter()
            .find(|(n, _)| n == k)
            .map(|(_, v)| v.as_str())
    };
    assert_eq!(cab("accept"), Some("application/json"));
    assert_eq!(cab("x-github-api-version"), None);
    for p in ["g", "g/../p", "g/p?x", "g/p%2F"] {
        let r = GitLab::nuevo(
            base(&mock),
            p,
            Box::new(CredencialesFijas::nuevas()),
            Personas::default(),
        );
        assert!(r.is_err_and(|i| i.resultado == Resultado::Uso), "{p}");
    }

    // `%2F` solo con GitLab y solo exacto.
    let token = Secreto::nuevo(TOKEN);
    let gitlab_c = Cliente::para(base(&mock), Proveedor::GitLab);
    let github_c = Cliente::para(base(&mock), Proveedor::GitHub);
    assert!(
        gitlab_c
            .llamar(Metodo::Get, "/projects/g%2Fp", Some(&token), None)
            .is_ok()
    );
    for ruta in [
        "/projects/g%2fp",
        "/projects/g%252Fp",
        "/projects/g%2F..",
        "/projects/%2Fp",
    ] {
        assert!(
            matches!(
                gitlab_c.llamar(Metodo::Get, ruta, Some(&token), None),
                Err(FalloHttp::Rechazada(_))
            ),
            "{ruta}"
        );
    }
    assert!(matches!(
        github_c.llamar(Metodo::Get, "/projects/g%2Fp", Some(&token), None),
        Err(FalloHttp::Rechazada(_))
    ));

    // Tabla de errores de GitLab, sobre `identidad`.
    let casos = [
        (
            Simulada::json(401, "{\"message\":\"401 Unauthorized\"}"),
            "identidad denegada motivo=credenciales rol=agente",
            8,
            "TC2_TOKEN_AGENTE",
        ),
        (
            Simulada::json(403, "{\"message\":\"403 Forbidden\"}"),
            "identidad denegada motivo=permisos rol=agente",
            8,
            "scope api",
        ),
        (
            Simulada::json(404, "{\"message\":\"404 Project Not Found\"}"),
            "identidad denegada motivo=no-existe-o-sin-acceso rol=agente",
            8,
            "el proyecto, el MR o el commit",
        ),
        (
            Simulada::json(429, "{\"message\":\"Retry later\"}").con("Retry-After", "30"),
            "identidad agotada motivo=limite-peticiones reintenta-en=30",
            5,
            "espera 30 s",
        ),
        (
            Simulada::json(403, "{\"message\":\"x\"}")
                .con("RateLimit-Remaining", "0")
                .con("RateLimit-Reset", "1"),
            "identidad agotada motivo=limite-peticiones reintenta-en=0",
            5,
            "relanza",
        ),
        (
            Simulada::json(400, "{\"message\":{\"base\":[\"mal\\u0007\"]}}"),
            "identidad error motivo=validacion",
            1,
            "GitLab rechazo",
        ),
        (
            Simulada::json(500, "{\"error\":\"boom\"}"),
            "identidad error motivo=http estado=500",
            1,
            "GitLab: «boom»",
        ),
        (
            Simulada::json(200, "{\"username\":\"x\"}"),
            "identidad error motivo=respuesta-invalida",
            1,
            "respuesta de GitLab",
        ),
    ];
    adaptadores::tabla_de_errores("/api/v4/user", TOKEN, casos.into(), |m| {
        gitlab(m, &[Rol::Agente])
    });

    // Merge: 405 y 406 (no mergeable) y 409 (sha distinto en la plataforma).
    for (estado, linea) in [
        (401, "merge-pr denegada pr=5 motivo=permisos rol=persona"),
        (405, "merge-pr rechazada pr=5 motivo=no-mergeable"),
        (406, "merge-pr rechazada pr=5 motivo=no-mergeable"),
        (409, "merge-pr rechazada pr=5 motivo=head-cambiado"),
    ] {
        let mock = Mock::nuevo();
        persona(&mock, PERSONA, false);
        mock.responder(
            "GET",
            &format!("{P}/merge_requests/5"),
            Simulada::json(200, &mr(5, "opened", SHA)),
        );
        mock.responder(
            "PUT",
            &format!("{P}/merge_requests/5/merge"),
            Simulada::json(estado, "{\"message\":\"Branch cannot be merged\"}"),
        );
        let gl = gitlab(&mock, &[Rol::Persona]);
        assert_eq!(gl.merge_pr(5, SHA).linea(), linea);
    }
}

/// A1-02 (RR-15): en la carrera de `abrir-pr` (crear dice «ya existe»: 422
/// `already exists` en GitHub, 409 en GitLab; y la relectura de abiertos
/// sigue vacia) los dos adaptadores dan la misma linea y el mismo codigo,
/// byte a byte; el diagnostico es de cada plataforma y cita a la API.
#[test]
fn carrera_de_abrir_pr_da_la_misma_linea() {
    let _serie = serie();
    let mock = Mock::nuevo();
    mock.responder("GET", "/repos/o/r/pulls", Simulada::json(200, "[]"));
    mock.responder(
        "POST",
        "/repos/o/r/pulls",
        Simulada::json(
            422,
            "{\"message\":\"Validation Failed\",\"errors\":[{\"message\":\"A pull request already exists for o:feature/x.\"}]}",
        ),
    );
    let gh = github(&mock)
        .abrir_pr("feature/x", "develop", "T", "")
        .salida(false);
    let mock = Mock::nuevo();
    mock.responder(
        "GET",
        &format!("{P}/merge_requests"),
        Simulada::json(200, "[]"),
    );
    mock.responder(
        "GET",
        &format!("{P}/repository/commits"),
        Simulada::json(200, &format!("[{{\"id\":\"{SHA}\"}}]")),
    );
    mock.responder(
        "POST",
        &format!("{P}/merge_requests"),
        Simulada::json(
            409,
            "{\"message\":[\"Another open merge request already exists\"]}",
        ),
    );
    let gl = gitlab(&mock, &[Rol::Agente])
        .abrir_pr("feature/x", "develop", "T", "")
        .salida(false);
    assert_eq!(
        (gh.stdout.as_str(), gh.codigo),
        ("abrir-pr conflicto motivo=pr-existente\n", 6)
    );
    assert_eq!(
        (gl.stdout.as_str(), gl.codigo),
        (gh.stdout.as_str(), gh.codigo)
    );
    assert!(gh.stderr.contains("GitHub"), "{}", gh.stderr);
    assert!(
        gl.stderr.contains("Another open merge request"),
        "{}",
        gl.stderr
    );
}

fn verde_con(retoque: impl Fn(&mut serde_json::Value)) -> (GitLab, Mock) {
    let mut f = fixture("gitlab", "verde");
    retoque(&mut f);
    let mock = servir(&f);
    (gitlab(&mock, &[Rol::Agente, Rol::Persona]), mock)
}

fn estados(f: &mut serde_json::Value) -> &mut serde_json::Value {
    &mut f["respuestas"][format!("{P}/repository/commits/{SHA}/statuses")]["cuerpo"]
}

/// Criterio 2: las protecciones del adaptador GitLab, cada una con un caso
/// que se pone rojo si se quita.
#[test]
fn protecciones_de_gitlab() {
    let _serie = serie();
    let m = modelo(Proveedor::GitLab, &[]);
    let puertas = |retoque: &dyn Fn(&mut serde_json::Value)| {
        let (gl, _mock) = verde_con(retoque);
        gl.estado_puertas(Rol::Agente, 12, &m).linea()
    };

    // G09 (autor del estado): el revisor publicado por otra cuenta no cuenta.
    assert_eq!(
        puertas(&|f| estados(f)[2]["author"]["id"] = 9999.into()),
        "estado-puertas esperando pr=12 aprobaciones=1 ignoradas=0 falta=revision-ia"
    );
    // El ultimo estado de cada nombre manda, por id, aunque llegue antes.
    assert_eq!(
        puertas(&|f| {
            let s = estados(f).as_array_mut().unwrap();
            s.insert(
                0,
                serde_json::json!({"author":{"id":2001},"id":9,"name":"carril","status":"failed"}),
            );
        }),
        "estado-puertas rechazada pr=12 motivo=check-rojo checks=carril"
    );
    // Un estado saltado no cumple la puerta.
    assert_eq!(
        puertas(&|f| estados(f)[0]["status"] = "skipped".into()),
        "estado-puertas esperando pr=12 aprobaciones=1 ignoradas=0 falta=verificacion"
    );
    // Aprobacion de una cuenta bot, aunque su id este en la lista.
    assert_eq!(
        puertas(&|f| {
            f["respuestas"][format!("{P}/merge_requests/12/approvals")]["cuerpo"]["approved_by"]
                [0]["user"]["bot"] = true.into();
        }),
        "estado-puertas esperando pr=12 aprobaciones=0 ignoradas=1 falta=aprobacion"
    );
    // G01: rama destino sin proteger, con force-push o con merge para
    // Developers.
    let rama = format!("{P}/protected_branches/develop");
    for retoque in [
        Box::new(|f: &mut serde_json::Value| {
            f["respuestas"][&rama]["estado"] = 404.into();
        }) as Box<dyn Fn(&mut serde_json::Value) + '_>,
        Box::new(|f: &mut serde_json::Value| {
            f["respuestas"][&rama]["cuerpo"]["allow_force_push"] = true.into();
        }),
        Box::new(|f: &mut serde_json::Value| {
            f["respuestas"][&rama]["cuerpo"]["merge_access_levels"][0]["access_level"] = 30.into();
        }),
    ] {
        assert_eq!(
            puertas(&*retoque),
            "estado-puertas rechazada pr=12 motivo=config garantia=G01"
        );
    }
    // Plan: el 403 de las reglas de aprobacion sin garantia Premium exigida
    // sigue; un 401 ahi no se toma por plan.
    let reglas = format!("{P}/approval_rules");
    assert_eq!(
        puertas(&|f| f["respuestas"][&reglas]["estado"] = 403.into()),
        "estado-puertas ok pr=12 aprobaciones=1 ignoradas=0"
    );
    assert_eq!(
        puertas(&|f| f["respuestas"][&reglas]["estado"] = 401.into()),
        "estado-puertas denegada motivo=credenciales rol=agente"
    );
    // Free responde 200 con la lista vacia (observado en el E2E): con una
    // garantia Premium exigida, ninguna regla que pida aprobacion es un
    // rechazo, no un verde.
    let premium = modelo(Proveedor::GitLab, &["G03"]);
    for reglas_vistas in ["[]", "[{\"id\":1,\"name\":\"r\",\"approvals_required\":0}]"] {
        let (gl, _mock) = verde_con(|f| {
            f["respuestas"][&reglas]["cuerpo"] = serde_json::from_str(reglas_vistas).unwrap();
        });
        assert_eq!(
            gl.estado_puertas(Rol::Agente, 12, &premium).linea(),
            "estado-puertas rechazada pr=12 motivo=config garantia=G03",
            "{reglas_vistas}"
        );
    }
    let (gl, _mock) = verde_con(|_| {});
    assert_eq!(
        gl.estado_puertas(Rol::Agente, 12, &premium).linea(),
        "estado-puertas ok pr=12 aprobaciones=1 ignoradas=0"
    );
    // La aprobacion del autor del MR no cuenta (GitLab Free la admite).
    assert_eq!(
        puertas(&|f| mr_de(f)["author"]["id"] = PERSONA.into()),
        "estado-puertas esperando pr=12 aprobaciones=0 ignoradas=1 falta=aprobacion"
    );
    // Mientras GitLab comprueba el MR, no se da por libre de conflictos.
    assert_eq!(
        puertas(&|f| mr_de(f)["detailed_merge_status"] = "checking".into()),
        "estado-puertas esperando pr=12 aprobaciones=1 ignoradas=0 falta=sin-conflictos"
    );
    // G01 con un segundo nivel de merge (Developers ademas de Maintainers).
    assert_eq!(
        puertas(&|f| {
            f["respuestas"][format!("{P}/protected_branches/develop")]["cuerpo"]
                ["merge_access_levels"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({"access_level": 30}));
        }),
        "estado-puertas rechazada pr=12 motivo=config garantia=G01"
    );
    // Campo ausente: respuesta invalida, no un verde por defecto.
    assert_eq!(
        puertas(&|f| {
            mr_de(f).as_object_mut().unwrap().remove("has_conflicts");
        }),
        "estado-puertas error motivo=respuesta-invalida"
    );
    // MR cerrado.
    assert_eq!(
        puertas(&|f| mr_de(f)["state"] = "closed".into()),
        "estado-puertas rechazada pr=12 motivo=pr-cerrado"
    );
    // Como persona, la cuenta tiene que estar en la lista.
    let (gl, mock) = verde_con(|_| {});
    persona(&mock, 4242, false);
    assert_eq!(
        gl.estado_puertas(Rol::Persona, 12, &m).linea(),
        "estado-puertas rechazada motivo=identidad rol=persona"
    );

    // G01: el nivel de merge es el rol Maintainer, no un usuario concreto.
    assert_eq!(
        puertas(&|f| {
            f["respuestas"][format!("{P}/protected_branches/develop")]["cuerpo"]["merge_access_levels"]
                [0]["user_id"] = 5.into();
        }),
        "estado-puertas rechazada pr=12 motivo=config garantia=G01"
    );

    // merge-pr con un token de persona fuera de la lista: sin PUT.
    let mock = Mock::nuevo();
    persona(&mock, 4242, false);
    mock.responder(
        "GET",
        &format!("{P}/merge_requests/12"),
        Simulada::json(200, &mr(12, "opened", SHA)),
    );
    let gl = gitlab(&mock, &[Rol::Persona]);
    assert_eq!(
        gl.merge_pr(12, SHA).linea(),
        "merge-pr rechazada motivo=identidad rol=persona"
    );
    assert!(mock.recibidas().iter().all(|r| !r.linea.starts_with("PUT")));

    // merge-pr con otro head: rechazado sin PUT.
    let mock = Mock::nuevo();
    persona(&mock, PERSONA, false);
    mock.responder(
        "GET",
        &format!("{P}/merge_requests/12"),
        Simulada::json(200, &mr(12, "opened", SHA_MERGE)),
    );
    let gl = gitlab(&mock, &[Rol::Persona]);
    assert_eq!(
        gl.merge_pr(12, SHA).linea(),
        "merge-pr rechazada pr=12 motivo=head-cambiado"
    );
    assert!(mock.recibidas().iter().all(|r| !r.linea.starts_with("PUT")));

    // abrir-pr sin commits nuevos: no se crea.
    let mock = Mock::nuevo();
    mock.responder(
        "GET",
        &format!("{P}/merge_requests"),
        Simulada::json(200, "[]"),
    );
    mock.responder(
        "GET",
        &format!("{P}/repository/commits"),
        Simulada::json(200, "[]"),
    );
    let gl = gitlab(&mock, &[Rol::Agente]);
    assert_eq!(
        gl.abrir_pr("feature/x", "develop", "T", "").linea(),
        "abrir-pr rechazada motivo=sin-commits"
    );
    assert!(
        mock.recibidas()
            .iter()
            .all(|r| !r.linea.starts_with("POST"))
    );

    // Contrato del firmante por clave (F1-13, decision 6).
    let personas = Personas::nuevas([PERSONA]).con_claves([Clave::Ssh(CLAVE)]);
    let firma = |respuesta: Simulada, commit: Simulada| {
        let mock = Mock::nuevo();
        mock.responder(
            "GET",
            &format!("{P}/repository/commits/{SHA}/signature"),
            respuesta,
        );
        mock.responder("GET", &format!("{P}/repository/commits/{SHA}"), commit);
        gitlab(&mock, &[Rol::Agente]).firma_commit(SHA)
    };
    let commit = || Simulada::json(200, &format!("{{\"id\":\"{SHA}\"}}"));
    let f = firma(
        Simulada::json(
            200,
            &format!("{{\"signature_type\":\"SSH\",\"verification_status\":\"verified\",\"key\":{{\"id\":{CLAVE}}}}}"),
        ),
        commit(),
    )
    .unwrap();
    assert!(f.cuenta_como_aprobacion(&personas));
    let f = firma(
        Simulada::json(
            200,
            "{\"signature_type\":\"PGP\",\"verification_status\":\"verified\",\"gpg_key_id\":777}",
        ),
        commit(),
    )
    .unwrap();
    assert_eq!(f.clave, Some(Clave::Gpg(777)));
    assert!(
        !f.cuenta_como_aprobacion(&personas),
        "clave fuera de la lista"
    );
    let f = firma(
        Simulada::json(
            200,
            &format!("{{\"signature_type\":\"SSH\",\"verification_status\":\"verified_system\",\"key\":{{\"id\":{CLAVE}}}}}"),
        ),
        commit(),
    )
    .unwrap();
    assert!(
        !f.cuenta_como_aprobacion(&personas),
        "la firma de GitLab no es de una persona"
    );
    // El id de una clave GPG no se confunde con el de una SSH de la lista.
    let f = firma(
        Simulada::json(
            200,
            &format!("{{\"signature_type\":\"PGP\",\"verification_status\":\"verified\",\"gpg_key_id\":{CLAVE}}}"),
        ),
        commit(),
    )
    .unwrap();
    assert!(
        !f.cuenta_como_aprobacion(&personas),
        "tipo de clave distinto"
    );
    let f = firma(
        Simulada::json(404, "{\"message\":\"404 Signature Not Found\"}"),
        commit(),
    )
    .unwrap();
    assert_eq!((f.verificada, f.motivo.as_str()), (false, "unsigned"));
    let i = firma(
        Simulada::json(404, "{\"message\":\"404 Not Found\"}"),
        Simulada::json(404, "{\"message\":\"404 Commit Not Found\"}"),
    )
    .unwrap_err();
    assert_eq!(
        i.linea(),
        "firma-commit denegada motivo=no-existe-o-sin-acceso rol=agente"
    );

    // leer-puertas: la proteccion de la rama y el plan sin reglas.
    let mock = Mock::nuevo();
    mock.responder(
        "GET",
        &format!("{P}/protected_branches"),
        Simulada::json(
            200,
            "[{\"name\":\"main\",\"merge_access_levels\":[{\"access_level\":30}],\"allow_force_push\":true},{\"name\":\"develop\",\"merge_access_levels\":[{\"access_level\":40}],\"push_access_levels\":[{\"access_level\":0}],\"allow_force_push\":false,\"code_owner_approval_required\":true}]",
        ),
    );
    mock.responder(
        "GET",
        &reglas,
        Simulada::json(403, "{\"message\":\"403 Forbidden\"}"),
    );
    // Con token de revisor (el caso real): lo rechaza el rol, no la falta de
    // token, y sin llamar.
    let gl = gitlab(&mock, &[Rol::Agente, Rol::Revisor]);
    let l = gl.leer_puertas(Rol::Agente, "develop").unwrap();
    assert!(!l.plan_da_reglas);
    assert_eq!(l.reglas_rama, ["push=0", "merge=40", "code-owner"]);
    let antes = mock.recibidas().len();
    assert_eq!(
        gl.leer_puertas(Rol::Revisor, "develop")
            .unwrap_err()
            .linea(),
        "leer-puertas rechazada motivo=identidad rol=revisor"
    );
    assert_eq!(mock.recibidas().len(), antes, "el revisor no llama");

    // leer-permisos-ci e identidad: persona de la lista, bot como Bot.
    let mock = Mock::nuevo();
    persona(&mock, PERSONA, false);
    assert_eq!(
        gitlab(&mock, &[Rol::Agente]).leer_permisos_ci().linea(),
        "leer-permisos-ci rechazada motivo=identidad falta=persona"
    );
    let gl = gitlab(&mock, &[Rol::Persona]);
    assert_eq!(
        gl.leer_permisos_ci().linea(),
        "leer-permisos-ci ok ci-aprueba=false fuente=plataforma"
    );
    let mock = Mock::nuevo();
    persona(&mock, PERSONA, true);
    let u = gitlab(&mock, &[Rol::Agente])
        .identidad(Rol::Agente)
        .unwrap();
    assert_eq!((u.tipo.as_str(), u.id), ("Bot", PERSONA));
    assert!(!u.es_persona(&personas));
}

/// GitLab para `aplicar-puertas` en Premium: persona 1001, nada protegido,
/// los checks ya publicados en `develop`, y las reglas de aprobacion y los
/// ajustes del proyecto que se den.
fn gitlab_premium(reglas: &str, proyecto: &str) -> Mock {
    gitlab_premium_con(reglas, proyecto, "[]", "[]")
}

/// Como [`gitlab_premium`], con las ramas y los tags protegidos que se den.
fn gitlab_premium_con(reglas: &str, proyecto: &str, protegidas: &str, tags: &str) -> Mock {
    let sha = "c".repeat(40);
    let mock = Mock::nuevo();
    mock.responder(
        "GET",
        "/api/v4/user",
        Simulada::json(200, r#"{"username":"carlos","id":1001,"bot":false}"#),
    );
    for (ruta, cuerpo) in [
        ("/protected_branches", protegidas.to_owned()),
        ("/protected_tags", tags.to_owned()),
        ("/approval_rules", reglas.to_owned()),
        ("", proyecto.to_owned()),
        (
            "/repository/branches/develop",
            format!(r#"{{"commit":{{"id":"{sha}"}}}}"#),
        ),
        (
            &format!("/repository/commits/{sha}/statuses"),
            r#"[{"id":1,"name":"verificacion","status":"success","author":{"id":3001}},
               {"id":2,"name":"carril","status":"success","author":{"id":3001}},
               {"id":3,"name":"revision-ia","status":"success","author":{"id":2002}}]"#
                .to_owned(),
        ),
    ] {
        mock.responder("GET", &format!("{P}{ruta}"), Simulada::json(200, &cuerpo));
    }
    mock.responder(
        "GET",
        "/api/v4/namespaces/9",
        Simulada::json(200, r#"{"plan":"premium"}"#),
    );
    mock
}

/// TASK-118: en Premium, la regla de aprobacion se crea si no hay ninguna
/// que pida al menos una (lo existente se reconoce por lo que pide, no por
/// su nombre); un 409 al proteger una rama es «otro llego antes». Ninguna
/// escritura es `DELETE` ni `PATCH`.
#[test]
fn aplicar_puertas_gitlab_premium_y_carrera() {
    use tc2::plataforma::puertas::{modelo as puertas, origenes};
    let _serie = serie();
    let o = origenes(Proveedor::GitLab, Some(2002)).unwrap();
    let bien = r#"{"only_allow_merge_if_pipeline_succeeds":true,"merge_method":"merge","namespace":{"id":9}}"#;

    // Regla ya puesta (con otro nombre) y ajustes bien; `develop` la
    // protegio otro entre la lectura y la escritura (409).
    let mock = gitlab_premium(r#"[{"name":"de-la-persona","approvals_required":1}]"#, bien);
    mock.responder(
        "POST",
        &format!("{P}/protected_branches"),
        Simulada::json(
            409,
            r#"{"message":"Protected branch 'develop' already exists"}"#,
        ),
    );
    mock.responder(
        "POST",
        &format!("{P}/protected_branches"),
        Simulada::json(201, "{}"),
    );
    mock.responder(
        "POST",
        &format!("{P}/protected_tags"),
        Simulada::json(201, "{}"),
    );
    let gl = gitlab(&mock, &[Rol::Persona]);
    let i = gl.aplicar_puertas(&puertas(), &o);
    assert_eq!(
        i.linea(),
        "aplicar-puertas ok aplicadas=taskcodetwo-main,taskcodetwo-ramas-de-tarea,taskcodetwo-tareas,taskcodetwo-tags-version existentes=taskcodetwo-develop"
    );
    let escrituras: Vec<String> = gl
        .registro()
        .into_iter()
        .filter(|l| !l.starts_with("GET "))
        .collect();
    assert!(
        escrituras
            .iter()
            .all(|l| l.starts_with("POST ") && !l.contains("approval_rules")),
        "{escrituras:?}"
    );

    // Sin regla que pida aprobaciones: se crea una, una sola vez.
    let mock = gitlab_premium(r#"[{"name":"All Members","approvals_required":0}]"#, bien);
    for ruta in ["/protected_branches", "/protected_tags", "/approval_rules"] {
        mock.responder("POST", &format!("{P}{ruta}"), Simulada::json(201, "{}"));
    }
    let gl = gitlab(&mock, &[Rol::Persona]);
    let i = gl.aplicar_puertas(&puertas(), &o);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert_eq!(i.valor("sin-aprobaciones"), None);
    let reglas: Vec<serde_json::Value> = mock
        .recibidas()
        .into_iter()
        .filter(|r| r.linea == format!("POST {P}/approval_rules"))
        .map(|r| serde_json::from_str(&r.cuerpo).unwrap())
        .collect();
    assert_eq!(
        reglas,
        [
            serde_json::json!({"name": "taskcodetwo-aprobacion", "approvals_required": 1, "rule_type": "any_approver"})
        ]
    );
    assert!(
        mock.recibidas()
            .iter()
            .all(|r| !r.linea.starts_with("DELETE ") && !r.linea.starts_with("PATCH "))
    );
}

/// Revision 1, IMP-1: un corte entre proteger `develop` y el `PUT` de los
/// ajustes de merge deja `develop` a medias; la repeticion ve la proteccion
/// igual que el modelo y completa los ajustes y la regla de aprobacion
/// (Premium) en vez de darla por distinta, y una tercera no escribe nada.
#[test]
fn aplicar_puertas_gitlab_corte_tras_proteger_converge() {
    use tc2::plataforma::puertas::{modelo as puertas, origenes};
    let _serie = serie();
    let o = origenes(Proveedor::GitLab, Some(2002)).unwrap();
    let ff = r#"{"only_allow_merge_if_pipeline_succeeds":false,"merge_method":"ff","namespace":{"id":9}}"#;
    let bien = r#"{"only_allow_merge_if_pipeline_succeeds":true,"merge_method":"merge","namespace":{"id":9}}"#;
    let sin_regla = r#"[{"name":"All Members","approvals_required":0}]"#;
    let escrituras = |m: &Mock| -> Vec<String> {
        m.recibidas()
            .into_iter()
            .filter(|r| !r.linea.starts_with("GET "))
            .map(|r| r.linea)
            .collect()
    };
    let rama = |n: &str, push: u64, merge: u64| {
        format!(
            r#"{{"name":"{n}","push_access_levels":[{{"access_level":{push}}}],"merge_access_levels":[{{"access_level":{merge}}}],"allow_force_push":false}}"#
        )
    };

    // Ronda 1: `develop` queda protegida y el PUT da 500.
    let m = gitlab_premium_con(sin_regla, ff, "[]", "[]");
    m.responder(
        "POST",
        &format!("{P}/protected_branches"),
        Simulada::json(201, "{}"),
    );
    m.responder("PUT", P, Simulada::json(500, r#"{"message":"x"}"#));
    let i = gitlab(&m, &[Rol::Persona]).aplicar_puertas(&puertas(), &o);
    assert_eq!(
        i.linea(),
        "aplicar-puertas error motivo=http estado=500 aplicadas= a-medias=taskcodetwo-develop faltan=taskcodetwo-develop,taskcodetwo-main,taskcodetwo-ramas-de-tarea,taskcodetwo-tareas,taskcodetwo-tags-version"
    );
    assert_eq!(
        escrituras(&m),
        [format!("POST {P}/protected_branches"), format!("PUT {P}")]
    );

    // Ronda 2: `develop` ya protegida como el modelo; faltan los ajustes,
    // la regla y las demas puertas.
    let m = gitlab_premium_con(
        sin_regla,
        ff,
        &format!("[{}]", rama("develop", 0, 40)),
        "[]",
    );
    for ruta in ["/protected_branches", "/protected_tags", "/approval_rules"] {
        m.responder("POST", &format!("{P}{ruta}"), Simulada::json(201, "{}"));
    }
    m.responder("PUT", P, Simulada::json(200, "{}"));
    let gl = gitlab(&m, &[Rol::Persona]);
    let leidas = gl.preparar_puertas(&puertas(), &o).unwrap();
    assert!(
        leidas
            .con_estado(tc2::plataforma::EstadoPuerta::Difiere)
            .is_empty(),
        "{leidas:?}"
    );
    let i = gl.aplicar_puertas(&puertas(), &o);
    assert_eq!(
        i.linea(),
        "aplicar-puertas ok aplicadas=taskcodetwo-develop,taskcodetwo-main,taskcodetwo-ramas-de-tarea,taskcodetwo-tareas,taskcodetwo-tags-version existentes="
    );
    let es = escrituras(&m);
    assert_eq!(
        es.iter().filter(|l| **l == format!("PUT {P}")).count(),
        1,
        "{es:?}"
    );
    assert_eq!(
        es.iter()
            .filter(|l| **l == format!("POST {P}/approval_rules"))
            .count(),
        1,
        "{es:?}"
    );
    let ramas: Vec<String> = m
        .recibidas()
        .into_iter()
        .filter(|r| r.linea == format!("POST {P}/protected_branches"))
        .map(|r| r.cuerpo)
        .collect();
    assert!(
        ramas.iter().all(|c| !c.contains("\"develop\"")),
        "{ramas:?}"
    );
    assert!(
        es.iter()
            .all(|l| l.starts_with("POST ") || l.starts_with("PUT "))
    );

    // Ronda 3: todo completo, ninguna escritura.
    let todas = [
        rama("develop", 0, 40),
        rama("main", 0, 40),
        rama("feature/*", 30, 30),
        rama("fix/*", 30, 30),
        rama("hotfix/*", 30, 30),
        rama("taskcodetwo/tareas", 30, 30),
    ]
    .join(",");
    let m = gitlab_premium_con(
        r#"[{"name":"taskcodetwo-aprobacion","approvals_required":1}]"#,
        bien,
        &format!("[{todas}]"),
        r#"[{"name":"v*","create_access_levels":[{"access_level":30}]}]"#,
    );
    let i = gitlab(&m, &[Rol::Persona]).aplicar_puertas(&puertas(), &o);
    assert_eq!(i.resultado, Resultado::SinCambios, "{}", i.linea());
    assert!(escrituras(&m).is_empty(), "{:?}", escrituras(&m));
}
