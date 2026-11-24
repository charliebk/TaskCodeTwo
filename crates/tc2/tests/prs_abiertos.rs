//! F3-5 (TASK-032): `prs-abiertos` del puerto Plataforma en GitHub y GitLab
//! contra el servidor simulado en 127.0.0.1. Los dos adaptadores dan la misma
//! lista (RR-15): solo los PR del propio repo, con su borrador, por numero.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use comun::adaptadores;
use comun::http_mock::{Mock, Simulada, serie};
use tc2::contrato::Resultado;
use tc2::plataforma::conexion::MAX_PAGINAS;
use tc2::plataforma::{Personas, Plataforma, PrAbierto, Rol};

const SHA_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SHA_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const SHA_C: &str = "cccccccccccccccccccccccccccccccccccccccc";
const GH: &str = "/repos/o/r/pulls";
const GL: &str = "/api/v4/projects/7/merge_requests";
const TOKEN: &str = "token-de-prueba";

fn github(base: &str, roles: &[Rol]) -> Box<dyn Plataforma> {
    Box::new(adaptadores::github(base, TOKEN, roles, Personas::default()))
}

fn gitlab(base: &str, roles: &[Rol]) -> Box<dyn Plataforma> {
    Box::new(adaptadores::gitlab(base, TOKEN, roles, Personas::default()))
}

/// Un PR de GitHub; `fork` pone la cabeza en otro repo.
fn pr_gh(n: u64, rama: &str, sha: &str, draft: bool, fork: bool) -> String {
    let cabeza = if fork { 2 } else { 1 };
    format!(
        r#"{{"number":{n},"draft":{draft},"title":"t","head":{{"ref":"{rama}","sha":"{sha}","repo":{{"id":{cabeza}}}}},"base":{{"ref":"develop","sha":"{SHA_C}","repo":{{"id":1}}}}}}"#
    )
}

/// Un PR o MR de la lista de abiertos: [`pr_gh`] o [`mr_gl`].
type Uno = fn(u64, &str, &str, bool, bool) -> String;

/// Un MR de GitLab; `fork` pone el origen en otro proyecto.
fn mr_gl(n: u64, rama: &str, sha: &str, draft: bool, fork: bool) -> String {
    let origen = if fork { 8 } else { 7 };
    format!(
        r#"{{"iid":{n},"state":"opened","source_branch":"{rama}","target_branch":"develop","sha":"{sha}","draft":{draft},"work_in_progress":{draft},"source_project_id":{origen},"target_project_id":7}}"#
    )
}

fn esperados() -> Vec<PrAbierto> {
    vec![
        PrAbierto {
            numero: 3,
            rama: "feature/task-003-x".into(),
            sha: SHA_B.into(),
            borrador: false,
        },
        PrAbierto {
            numero: 5,
            rama: "feature/task-002-y".into(),
            sha: SHA_A.into(),
            borrador: true,
        },
    ]
}

/// Dos paginas (por `Link rel=next`), un PR de fork con el mismo nombre de
/// rama que no cuenta, un borrador, un sha en mayusculas que sale en
/// minusculas y orden por numero: la misma lista en GitHub y en GitLab, con
/// el rol `agente` y una llamada por pagina.
#[test]
fn misma_lista_en_github_y_gitlab_sin_forks_y_paginada() {
    let _serie = serie();
    for plataforma in ["github", "gitlab"] {
        let m = Mock::nuevo();
        let (ruta, p, pagina1, pagina2) = if plataforma == "github" {
            (
                GH,
                github(&m.base(), &[Rol::Agente]),
                format!(
                    "[{},{}]",
                    pr_gh(5, "feature/task-002-y", SHA_A, true, false),
                    pr_gh(9, "feature/task-003-x", SHA_C, false, true)
                ),
                format!(
                    "[{}]",
                    pr_gh(
                        3,
                        "feature/task-003-x",
                        &SHA_B.to_ascii_uppercase(),
                        false,
                        false
                    )
                ),
            )
        } else {
            (
                GL,
                gitlab(&m.base(), &[Rol::Agente]),
                format!(
                    "[{},{}]",
                    mr_gl(5, "feature/task-002-y", SHA_A, true, false),
                    mr_gl(9, "feature/task-003-x", SHA_C, false, true)
                ),
                format!(
                    "[{}]",
                    mr_gl(
                        3,
                        "feature/task-003-x",
                        &SHA_B.to_ascii_uppercase(),
                        false,
                        false
                    )
                ),
            )
        };
        // La segunda pagina, como la pide el cliente (con la consulta).
        let estado = if plataforma == "github" {
            "open"
        } else {
            "opened"
        };
        let siguiente = format!("{ruta}?state={estado}&page=2");
        m.responder("GET", &siguiente, Simulada::json(200, &pagina2));
        m.responder(
            "GET",
            ruta,
            Simulada::json(200, &pagina1)
                .con("link", &format!("<{}{siguiente}>; rel=\"next\"", m.base())),
        );
        let r = p.prs_abiertos(Rol::Agente);
        assert_eq!(r, Ok(esperados()), "{plataforma}");
        let lineas: Vec<String> = m.recibidas().into_iter().map(|x| x.linea).collect();
        assert_eq!(lineas.len(), 2, "{plataforma}: {lineas:?}");
        assert!(lineas.iter().all(|l| l.starts_with("GET ")), "{lineas:?}");
        // El revisor no lista PR (F1-6: no es su operacion).
        let r = p.prs_abiertos(Rol::Revisor);
        assert_eq!(
            r.err().map(|i| i.linea()).as_deref(),
            Some("prs-abiertos rechazada motivo=identidad rol=revisor")
        );
    }
}

/// Sin token del rol, `rechazada motivo=identidad` sin llamar; con un token
/// que no vale, `denegada motivo=credenciales`; sin red, `sinred`; mas de
/// `MAX_PAGINAS` paginas, `error motivo=demasiadas-paginas`.
#[test]
fn fallos_tipados() {
    let _serie = serie();
    for plataforma in ["github", "gitlab"] {
        let m = Mock::nuevo();
        let hacer = |roles: &[Rol]| {
            if plataforma == "github" {
                github(&m.base(), roles)
            } else {
                gitlab(&m.base(), roles)
            }
        };
        let ruta = if plataforma == "github" { GH } else { GL };
        let r = hacer(&[]).prs_abiertos(Rol::Agente);
        assert_eq!(
            r.err().map(|i| i.linea()).as_deref(),
            Some("prs-abiertos rechazada motivo=identidad falta=agente")
        );
        assert!(m.recibidas().is_empty());
        m.responder("GET", ruta, Simulada::json(401, "{\"message\":\"Bad\"}"));
        let r = hacer(&[Rol::Agente]).prs_abiertos(Rol::Agente);
        assert_eq!(
            r.err()
                .map(|i| (i.resultado, i.valor("motivo").map(str::to_owned))),
            Some((Resultado::Denegada, Some("credenciales".to_owned())))
        );
        // Sin red: nadie escucha en el puerto 1.
        let sin_red = if plataforma == "github" {
            github("http://127.0.0.1:1", &[Rol::Agente])
        } else {
            gitlab("http://127.0.0.1:1", &[Rol::Agente])
        };
        let r = sin_red.prs_abiertos(Rol::Agente);
        assert_eq!(r.err().map(|i| i.resultado), Some(Resultado::Sinred));
    }
    // Paginas sin fin: cada una apunta a si misma.
    let m = Mock::nuevo();
    m.responder(
        "GET",
        GH,
        Simulada::json(200, "[]").con(
            "link",
            &format!("<{}{GH}?state=open&page=2>; rel=\"next\"", m.base()),
        ),
    );
    let r = github(&m.base(), &[Rol::Agente]).prs_abiertos(Rol::Agente);
    assert_eq!(
        r.err().map(|i| i.linea()).as_deref(),
        Some("prs-abiertos error motivo=demasiadas-paginas")
    );
    assert_eq!(m.recibidas().len(), MAX_PAGINAS);
}

/// Un sha que no es de 40 hex en un PR propio es respuesta invalida (falla
/// cerrado); una rama que tc2 no crearia se salta. En los dos adaptadores.
#[test]
fn sha_invalido_falla_y_rama_rara_se_salta() {
    let _serie = serie();
    for plataforma in ["github", "gitlab"] {
        let (ruta, uno): (&str, Uno) = if plataforma == "github" {
            (GH, pr_gh)
        } else {
            (GL, mr_gl)
        };
        let hacer = |base: &str| {
            if plataforma == "github" {
                github(base, &[Rol::Agente])
            } else {
                gitlab(base, &[Rol::Agente])
            }
        };
        let m = Mock::nuevo();
        m.responder(
            "GET",
            ruta,
            Simulada::json(
                200,
                &format!("[{}]", uno(1, "rama con espacio", "x", false, false)),
            ),
        );
        let p = hacer(&m.base());
        assert_eq!(p.prs_abiertos(Rol::Agente), Ok(Vec::new()), "{plataforma}");
        let m = Mock::nuevo();
        m.responder(
            "GET",
            ruta,
            Simulada::json(
                200,
                &format!("[{}]", uno(1, "feature/x", "x", false, false)),
            ),
        );
        let p = hacer(&m.base());
        let r = p.prs_abiertos(Rol::Agente);
        assert_eq!(
            r.err().map(|i| i.linea()).as_deref(),
            Some("prs-abiertos error motivo=respuesta-invalida"),
            "{plataforma}"
        );
    }
}
