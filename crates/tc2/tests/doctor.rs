//! F3-8 (TASK-035): `tc2 doctor` sobre repos reales en carpetas temporales
//! y GitHub y GitLab simulados en 127.0.0.1. Un test (o mas) por criterio;
//! los de plataforma corren las mismas aserciones en las dos (RR-15).

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::path::{Path, PathBuf};

use comun::http_mock::{Mock, Simulada, serie};
use comun::tareas::tarea;
use comun::{Fixture, escribir};
use tc2::doctor::{self, Entorno, Estado, Hallazgo, Resumen};
use tc2::estado;
use tc2::i18n::Idioma;
use tc2::plataforma::http::Base;
use tc2::plataforma::{CredencialesFijas, GitHub, GitLab, Personas, Plataforma, Rol, Secreto};
use tc2::tablero::{Conectada, Consulta};

const PERSONAS: &str = "personas:\n  - {id: 1001, login: \"ana\"}\n";

/// Remoto bare con personas en `develop` y la rama de estado publicada
/// desde un repo aparte (con `extra` de mas); devuelve (bare, clon).
fn con_estado(f: &Fixture, extra: &[(&str, &str)]) -> (PathBuf, PathBuf) {
    let (bare, clon) = f.remoto_y_clon();
    escribir(&clon, ".taskcode/personas.yml", PERSONAS);
    f.crudo(&clon, &["add", ".taskcode/personas.yml"]);
    f.crudo(&clon, &["commit", "-q", "-m", "personas"]);
    f.crudo(&clon, &["push", "-q", "origin", "develop"]);
    let mut ficheros = vec![(
        "TASK-001/tarea.md".to_owned(),
        tarea("TASK-001", 1, "planificada", &[]),
    )];
    ficheros.extend(
        extra
            .iter()
            .map(|(ruta, texto)| ((*ruta).to_owned(), (*texto).to_owned())),
    );
    f.publicar_estado(&bare, &ficheros);
    f.crudo(&clon, &["fetch", "-q", "origin"]);
    (bare, clon)
}

/// Plataforma sin configurar (para lo que no es de plataforma).
fn sin_plataforma(_: &Personas) -> Result<Conectada, Consulta> {
    Err(Consulta::NoConfigurada)
}

/// Los hallazgos de `doctor` en `dir` con `conectar`.
fn hallazgos(
    g: &tc2::git::Git,
    dir: &Path,
    conectar: &tc2::tablero::Conectar<'_>,
) -> (Vec<Hallazgo>, Resumen) {
    doctor::comprobar(&Entorno {
        dir,
        git: g,
        conectar,
        claude: None,
    })
    .unwrap_or_else(|i| panic!("{}", i.linea()))
}

fn de(hs: &[Hallazgo], area: &str, id: &str) -> Vec<String> {
    hs.iter()
        .filter(|h| h.area == area && h.id == id)
        .map(Hallazgo::linea)
        .collect()
}

/// C3: sin rama de estado, `falla motivo=sin-rama-de-estado
/// arreglo=tc2-init`; con un `tarea.md` ilegible, `falla` con su ID; con la
/// ref de la rama apuntando a algo que no es un commit, `falla` con la ref.
/// Codigo 3 en los tres.
#[test]
fn rama_de_estado_ausente_o_corrupta_falla() {
    let f = Fixture::nueva("doctor-c3");
    let (_, sin) = f.remoto_y_clon();
    let g = f.git();
    let (hs, r) = hallazgos(&g, &sin, &sin_plataforma);
    assert_eq!(
        de(&hs, "estado", "rama"),
        ["estado rama falla motivo=sin-rama-de-estado arreglo=tc2-init"]
    );
    assert_eq!(r.commit, None);
    assert_eq!(
        doctor::salida(&hs, &r, false, tc2::i18n::Idioma::Es).codigo,
        3
    );

    let f = Fixture::nueva("doctor-c3-ilegible");
    let (_, clon) = con_estado(&f, &[("TASK-009/tarea.md", "---\nesto no es\n")]);
    let g = f.git();
    let (hs, r) = hallazgos(&g, &clon, &sin_plataforma);
    assert_eq!(
        de(&hs, "estado", "rama"),
        ["estado rama ok ref=refs/remotes/origin/taskcodetwo/tareas"]
    );
    assert_eq!(
        de(&hs, "estado", "tareas"),
        ["estado tareas falla motivo=tarea-ilegible ids=TASK-009"]
    );
    assert_eq!(r.commit.as_ref().map(String::len), Some(12));
    assert_eq!(
        doctor::salida(&hs, &r, true, tc2::i18n::Idioma::Es).codigo,
        3
    );

    // La ref remota apunta a un blob: existe y no es un commit.
    let blob = f
        .crudo(&clon, &["rev-parse", "develop:README.md"])
        .trim()
        .to_owned();
    f.crudo(
        &clon,
        &[
            "update-ref",
            "refs/remotes/origin/taskcodetwo/tareas",
            &blob,
        ],
    );
    let (hs, _) = hallazgos(&g, &clon, &sin_plataforma);
    assert_eq!(
        de(&hs, "estado", "rama"),
        ["estado rama falla motivo=ref-ilegible ref=refs/remotes/origin/taskcodetwo/tareas"]
    );
    assert!(hs.iter().all(|h| h.area != "estado" || h.id != "tareas"));
}

/// (codigo, stdout, stderr) de `tc2 <args>` en `dir`, sin tokens.
fn tc2(f: &Fixture, dir: &Path, args: &[&str], env: &[(&str, &str)]) -> (i32, String, String) {
    let o = comun::orden_tc2()
        .current_dir(dir)
        .args(args)
        .envs(f.entorno())
        .env("GIT_CEILING_DIRECTORIES", &f.raiz)
        // El inventario (F3-9) lee una carpeta de Claude Code propia, nunca
        // la de la maquina.
        .env("CLAUDE_CONFIG_DIR", f.raiz.join("claude"))
        .envs(env.iter().copied())
        .output()
        .expect("lanzar tc2");
    (
        o.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&o.stdout).into_owned(),
        String::from_utf8_lossy(&o.stderr).into_owned(),
    )
}

/// C6: sin `git` en el PATH, `falla motivo=sin-git` (codigo 3) y lo demas
/// `no-comprobado`, sin suponer nada; fuera de un repo, `rechazada
/// motivo=no-es-repo` con el diagnostico en stderr.
#[test]
fn sin_git_falla_y_fuera_de_un_repo_rechaza() {
    let f = Fixture::nueva("doctor-c6");
    let vacia = f.raiz.join("vacia");
    std::fs::create_dir_all(&vacia).unwrap();
    let (c, out, err) = tc2(
        &f,
        &vacia,
        &["doctor"],
        &[("PATH", vacia.to_str().unwrap())],
    );
    assert_eq!(c, 3, "{out} {err}");
    let primera = out.lines().next().unwrap();
    assert!(
        primera.starts_with("doctor rechazada fallos=1 avisos=0 no-comprobado=4"),
        "{out}"
    );
    assert!(
        out.contains("entorno git falla motivo=sin-git arreglo=instalar-git"),
        "{out}"
    );
    assert!(
        out.contains("estado rama no-comprobado motivo=sin-git"),
        "{out}"
    );
    assert!(err.contains("entorno git falla"), "{err}");
    let (c, out, err) = tc2(&f, &vacia, &["doctor"], &[]);
    assert_eq!(
        (c, out.as_str()),
        (3, "doctor rechazada motivo=no-es-repo\n"),
        "{err}"
    );
    assert_eq!(err.lines().count(), 1, "{err}");
    // C11 de TASK-080: `--recursos` con algo de mas es uso.
    let (c, out, _) = tc2(&f, &vacia, &["doctor", "--recursos", "x"], &[]);
    assert_eq!((c, out.as_str()), (2, "doctor uso motivo=argumentos\n"));
    // Fuera de un repo, `--recursos` rechaza como `doctor`.
    let (c, out, _) = tc2(&f, &vacia, &["doctor", "--recursos"], &[]);
    assert_eq!(
        (c, out.as_str()),
        (3, "doctor rechazada motivo=no-es-repo\n")
    );
}

// ---------------------------------------------------------------------
// Plataforma: el mismo caso neutral servido como GitHub y como GitLab.
// ---------------------------------------------------------------------

const SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const MODELO: [&str; 3] = ["verificacion", "revision-ia", "carril"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Prov {
    GitHub,
    GitLab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ci {
    Bien,
    PuedeAprobar,
    Prohibido,
}

/// Plan que sirve el mock: GitHub (`privado`, `org`, rulesets con 403 de
/// plan) o el `plan` del namespace de GitLab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Plan {
    GitHub {
        privado: bool,
        org: bool,
        sin_rulesets: bool,
    },
    GitLab(&'static str),
    /// La lectura del plan da 404 (no se sabe).
    Ilegible,
}

/// Un caso neutral de plataforma.
#[derive(Debug, Clone)]
struct Caso {
    /// Ramas protegidas (con su ruleset y `pull_request`, o push a nadie).
    protegidas: Vec<&'static str>,
    /// 403 de permisos al leer las puertas.
    puertas_403: bool,
    ci: Ci,
    /// Checks requeridos en `develop` (GitLab: «Pipelines must succeed» si
    /// no esta vacio).
    requeridos: Vec<&'static str>,
    /// Checks publicados en la cabeza de `develop`.
    publicados: Vec<&'static str>,
    plan: Plan,
    /// `null` donde la API da una lista o un numero (H5): GitHub,
    /// `required_status_checks: null`; GitLab, `approvals_required: null`.
    nulos: bool,
    /// GitLab: 403 en las reglas de aprobacion (H4).
    aprobacion_403: bool,
    /// 403 en los checks publicados (H3): check-runs o estados.
    checks_403: bool,
}

impl Caso {
    /// Todo en orden: las dos ramas protegidas, los tres checks del modelo
    /// requeridos y publicados.
    fn bien(prov: Prov) -> Self {
        Self {
            protegidas: vec!["develop", "main"],
            puertas_403: false,
            ci: Ci::Bien,
            requeridos: MODELO.to_vec(),
            publicados: MODELO.to_vec(),
            plan: match prov {
                Prov::GitHub => Plan::GitHub {
                    privado: false,
                    org: false,
                    sin_rulesets: false,
                },
                Prov::GitLab => Plan::GitLab("premium"),
            },
            nulos: false,
            aprobacion_403: false,
            checks_403: false,
        }
    }
}

fn json(estado: u16, v: &serde_json::Value) -> Simulada {
    Simulada::json(estado, &v.to_string())
}

/// Sirve `caso` como lo daria `prov`.
fn servir(prov: Prov, caso: &Caso) -> Mock {
    use serde_json::json as j;
    let m = Mock::nuevo();
    let prohibido = j!({"message": "Resource not accessible by integration"});
    match prov {
        Prov::GitHub => {
            let r = "/repos/o/r";
            m.responder(
                "GET",
                "/user",
                json(200, &j!({"login": "ana", "id": 1001, "type": "User"})),
            );
            if caso.puertas_403 {
                m.responder("GET", &format!("{r}/rulesets"), json(403, &prohibido));
            } else {
                let nombres: Vec<serde_json::Value> = caso
                    .protegidas
                    .iter()
                    .map(|p| j!({"id": 1, "name": format!("taskcodetwo-{p}")}))
                    .collect();
                if let Plan::GitHub {
                    sin_rulesets: true, ..
                } = caso.plan
                {
                    m.responder(
                        "GET",
                        &format!("{r}/rulesets"),
                        json(403, &j!({"message": "Upgrade to GitHub Pro or make this repository public to enable this feature."})),
                    );
                } else {
                    m.responder("GET", &format!("{r}/rulesets"), json(200, &j!(nombres)));
                }
            }
            for rama in ["develop", "main"] {
                let mut reglas = Vec::new();
                if caso.protegidas.contains(&rama) {
                    reglas.push(j!({"type": "deletion"}));
                    reglas.push(j!({"type": "non_fast_forward"}));
                    reglas.push(j!({"type": "pull_request", "parameters": {
                        "required_approving_review_count": 1,
                        "dismiss_stale_reviews_on_push": true,
                        "require_code_owner_review": true,
                        "require_last_push_approval": true,
                        "required_review_thread_resolution": false}}));
                }
                if rama == "develop" && caso.nulos {
                    reglas.push(j!({"type": "required_status_checks", "parameters": {
                        "required_status_checks": null,
                        "strict_required_status_checks_policy": true}}));
                } else if rama == "develop" && !caso.requeridos.is_empty() {
                    let checks: Vec<serde_json::Value> =
                        caso.requeridos.iter().map(|c| j!({"context": c})).collect();
                    reglas.push(j!({"type": "required_status_checks", "parameters": {
                        "required_status_checks": checks,
                        "strict_required_status_checks_policy": true}}));
                }
                m.responder(
                    "GET",
                    &format!("{r}/rules/branches/{rama}"),
                    json(200, &j!(reglas)),
                );
            }
            let runs: Vec<serde_json::Value> = caso
                .publicados
                .iter()
                .map(|n| j!({"name": n, "status": "completed", "conclusion": "success", "app": {"id": 1}}))
                .collect();
            m.responder(
                "GET",
                &format!("{r}/commits/heads/develop/check-runs"),
                if caso.checks_403 {
                    json(403, &prohibido)
                } else {
                    json(200, &j!({"total_count": runs.len(), "check_runs": runs}))
                },
            );
            let ci = match caso.ci {
                Ci::Bien => json(
                    200,
                    &j!({"can_approve_pull_request_reviews": false, "default_workflow_permissions": "read"}),
                ),
                Ci::PuedeAprobar => json(
                    200,
                    &j!({"can_approve_pull_request_reviews": true, "default_workflow_permissions": "write"}),
                ),
                Ci::Prohibido => json(403, &prohibido),
            };
            m.responder("GET", &format!("{r}/actions/permissions/workflow"), ci);
            if let Plan::GitHub { privado, org, .. } = caso.plan {
                m.responder(
                    "GET",
                    r,
                    json(
                        200,
                        &j!({"id": 1, "private": privado,
                             "visibility": if privado { "private" } else { "public" },
                             "owner": {"login": "o", "type": if org { "Organization" } else { "User" }}}),
                    ),
                );
            }
        }
        Prov::GitLab => {
            let p = "/api/v4/projects/7";
            m.responder(
                "GET",
                "/api/v4/user",
                json(200, &j!({"username": "ana", "id": 1001, "bot": false})),
            );
            if caso.puertas_403 {
                m.responder(
                    "GET",
                    &format!("{p}/protected_branches"),
                    json(403, &j!({"message": "403 Forbidden"})),
                );
            } else {
                let ramas: Vec<serde_json::Value> = caso
                    .protegidas
                    .iter()
                    .map(|r| {
                        j!({"name": r, "merge_access_levels": [{"access_level": 40}],
                            "push_access_levels": [{"access_level": 0}],
                            "allow_force_push": false, "code_owner_approval_required": false})
                    })
                    .collect();
                m.responder(
                    "GET",
                    &format!("{p}/protected_branches"),
                    json(200, &j!(ramas)),
                );
            }
            let premium = matches!(caso.plan, Plan::GitLab("premium" | "ultimate"));
            m.responder(
                "GET",
                &format!("{p}/approval_rules"),
                if caso.aprobacion_403 {
                    json(403, &j!({"message": "403 Forbidden"}))
                } else if premium && caso.nulos {
                    json(200, &j!([{"name": "persona", "approvals_required": null}]))
                } else if premium {
                    json(200, &j!([{"name": "persona", "approvals_required": 1}]))
                } else {
                    json(200, &j!([]))
                },
            );
            m.responder(
                "GET",
                p,
                json(
                    200,
                    &j!({"id": 7, "visibility": "private",
                         "only_allow_merge_if_pipeline_succeeds": !caso.requeridos.is_empty(),
                         "namespace": {"id": 3, "kind": "group", "full_path": "g"}}),
                ),
            );
            m.responder(
                "GET",
                &format!("{p}/repository/branches/develop"),
                json(200, &j!({"name": "develop", "commit": {"id": SHA}})),
            );
            let estados: Vec<serde_json::Value> = caso
                .publicados
                .iter()
                .enumerate()
                .map(
                    |(i, n)| j!({"id": i + 1, "name": n, "status": "success", "author": {"id": 2}}),
                )
                .collect();
            m.responder(
                "GET",
                &format!("{p}/repository/commits/{SHA}/statuses"),
                if caso.checks_403 {
                    json(403, &j!({"message": "403 Forbidden"}))
                } else {
                    json(200, &j!(estados))
                },
            );
            if let Plan::GitLab(plan) = caso.plan {
                m.responder(
                    "GET",
                    "/api/v4/namespaces/3",
                    json(200, &j!({"id": 3, "plan": plan})),
                );
            }
        }
    }
    m
}

/// El adaptador de `prov` contra `base`, con token de agente y de persona.
fn adaptador(prov: Prov, base: &str, personas: &Personas) -> Box<dyn Plataforma> {
    let c = CredencialesFijas::nuevas()
        .con(Rol::Agente, Secreto::nuevo("tok-agente"))
        .con(Rol::Persona, Secreto::nuevo("tok-persona"));
    match prov {
        Prov::GitHub => Box::new(
            GitHub::nuevo(
                Base::nueva(base).unwrap(),
                "o/r",
                Box::new(c),
                personas.clone(),
            )
            .unwrap(),
        ),
        Prov::GitLab => Box::new(
            GitLab::nuevo(
                Base::nueva(&format!("{base}/api/v4")).unwrap(),
                "7",
                Box::new(c),
                personas.clone(),
            )
            .unwrap(),
        ),
    }
}

/// Conecta como persona con el adaptador de `prov` contra `base`.
fn conectar(prov: Prov, base: String) -> impl Fn(&Personas) -> Result<Conectada, Consulta> {
    move |p: &Personas| {
        Ok(Conectada {
            plataforma: adaptador(prov, &base, p),
            rol: Rol::Persona,
        })
    }
}

/// `doctor` en `clon` (de `f`) contra `caso` servido como `prov`:
/// hallazgos, resumen, peticiones recibidas y el `Git` con su registro.
fn doctor_contra_en(
    f: &Fixture,
    clon: &Path,
    prov: Prov,
    caso: &Caso,
) -> (Vec<Hallazgo>, Resumen, Vec<String>, tc2::git::Git) {
    let m = servir(prov, caso);
    let g = f.git();
    let (hs, r) = hallazgos(&g, clon, &conectar(prov, m.base()));
    let recibidas = m.recibidas().into_iter().map(|r| r.linea).collect();
    (hs, r, recibidas, g)
}

/// `doctor` en un clon con estado contra `caso` servido como `prov`:
/// hallazgos, resumen y peticiones recibidas.
fn doctor_contra(prov: Prov, caso: &Caso) -> (Vec<Hallazgo>, Resumen, Vec<String>) {
    let f = Fixture::nueva("doctor-plataforma");
    let (_, clon) = con_estado(&f, &[]);
    let (hs, r, recibidas, _) = doctor_contra_en(&f, &clon, prov, caso);
    (hs, r, recibidas)
}

fn lineas_de(hs: &[Hallazgo], area: &str) -> Vec<String> {
    hs.iter()
        .filter(|h| h.area == area)
        .map(Hallazgo::linea)
        .collect()
}

fn codigo(hs: &[Hallazgo], r: &Resumen) -> u8 {
    doctor::salida(hs, r, false, Idioma::Es).codigo
}

/// C1: sin el ruleset `taskcodetwo-develop` (GitHub) o sin `develop`
/// protegida (GitLab), `doctor rechazada` con `proteccion develop falla
/// motivo=proteccion-ausente arreglo=tc2-init` y codigo 3; misma linea en
/// las dos.
#[test]
fn develop_sin_proteger_falla() {
    let _serie = serie();
    for prov in [Prov::GitHub, Prov::GitLab] {
        let mut caso = Caso::bien(prov);
        caso.protegidas = vec!["main"];
        let (hs, r, _) = doctor_contra(prov, &caso);
        assert_eq!(
            lineas_de(&hs, "proteccion"),
            [
                "proteccion develop falla motivo=proteccion-ausente arreglo=tc2-init",
                "proteccion main ok"
            ],
            "{prov:?}"
        );
        let s = doctor::salida(&hs, &r, false, Idioma::Es);
        assert_eq!(s.codigo, 3, "{prov:?}");
        assert!(
            s.stdout.starts_with("doctor rechazada fallos="),
            "{}",
            s.stdout
        );
        assert!(
            s.stdout.contains(
                "\nproteccion develop falla motivo=proteccion-ausente arreglo=tc2-init\n"
            ),
            "{prov:?}: {}",
            s.stdout
        );
        assert_eq!(r.plataforma, "ok");
    }
}

/// C2: con `develop` y `main` protegidas no hay hallazgo de proteccion que
/// no sea `ok` y, sin otros fallos, el codigo es 0. Solo `GET`.
#[test]
fn ramas_protegidas_sin_fallos_dan_cero() {
    let _serie = serie();
    for prov in [Prov::GitHub, Prov::GitLab] {
        let (hs, r, recibidas) = doctor_contra(prov, &Caso::bien(prov));
        assert_eq!(
            lineas_de(&hs, "proteccion"),
            ["proteccion develop ok", "proteccion main ok"],
            "{prov:?}"
        );
        let fallos: Vec<String> = hs
            .iter()
            .filter(|h| h.estado == Estado::Falla)
            .map(Hallazgo::linea)
            .collect();
        assert!(fallos.is_empty(), "{prov:?}: {fallos:?}");
        assert_eq!(codigo(&hs, &r), 0, "{prov:?}");
        assert!(!recibidas.is_empty());
        assert!(
            recibidas.iter().all(|l| l.starts_with("GET ")),
            "{recibidas:?}"
        );
        // R1-06 de TASK-035: lo comun al repo se lee una vez y los checks
        // publicados solo en `develop`.
        let veces = |trozo: &str| recibidas.iter().filter(|l| l.contains(trozo)).count();
        let una_vez: &[&str] = match prov {
            Prov::GitHub => &[
                "/rulesets",
                "/rules/branches/develop",
                "/rules/branches/main",
                "/check-runs",
            ],
            Prov::GitLab => &[
                "/protected_branches",
                "/approval_rules",
                "/repository/branches/",
                "/statuses",
            ],
        };
        for t in una_vez {
            assert_eq!(veces(t), 1, "{prov:?} {t}: {recibidas:?}");
        }
        assert_eq!(veces("heads/main/check-runs"), 0, "{recibidas:?}");
    }
}

/// H1 (con las respuestas `null` del informe), H3, H4 y H5 de TASK-035:
/// `required_status_checks: null` y `approvals_required: null` se leen
/// (como lista vacia y 0) y lo que no esta configurado no sale impuesto; un
/// 403 en los checks publicados dice su permiso; un 403 en las reglas de
/// aprobacion de GitLab con plan Premium no es config ausente.
#[test]
fn nulos_y_403_parciales_no_dan_nada_por_bueno() {
    let _serie = serie();
    let primera = |hs: &[Hallazgo], r: &Resumen| doctor::lineas(hs, r)[0].clone();
    for prov in [Prov::GitHub, Prov::GitLab] {
        let mut caso = Caso::bien(prov);
        caso.nulos = true;
        let (hs, r, _) = doctor_contra(prov, &caso);
        assert_eq!(r.plataforma, "ok", "{prov:?}");
        let (cuentas, id) = match prov {
            Prov::GitHub => (
                " plan=github-free-publico-usuario impuestas=18 sustituidas=1 no-disponibles=1 ",
                "G09",
            ),
            // G06 ya falta en el caso (sin propietarios de codigo).
            Prov::GitLab => (
                " plan=gitlab-premium impuestas=15 sustituidas=3 no-disponibles=2 ",
                "G03",
            ),
        };
        assert!(
            primera(&hs, &r).contains(cuentas),
            "{prov:?}: {}",
            primera(&hs, &r)
        );
        assert!(
            lineas_de(&hs, "garantia").contains(&format!(
                "garantia {id} aviso motivo=config capacidad=no-disponible arreglo=tc2-init"
            )),
            "{prov:?}: {:?}",
            lineas_de(&hs, "garantia")
        );
        // H3: 403 en los checks publicados.
        let mut caso = Caso::bien(prov);
        caso.checks_403 = true;
        let (hs, _, _) = doctor_contra(prov, &caso);
        let permiso = match prov {
            Prov::GitHub => "checks:read",
            Prov::GitLab => "read_api",
        };
        assert_eq!(
            lineas_de(&hs, "check"),
            MODELO
                .iter()
                .map(|c| format!(
                    "check {c} no-comprobado motivo=permiso-insuficiente rama=develop permiso={permiso}"
                ))
                .collect::<Vec<_>>(),
            "{prov:?}"
        );
    }
    // H4: GitLab Premium legible y 403 en las reglas de aprobacion.
    let mut caso = Caso::bien(Prov::GitLab);
    caso.aprobacion_403 = true;
    let (hs, r, _) = doctor_contra(Prov::GitLab, &caso);
    let g = lineas_de(&hs, "garantia");
    assert!(
        g.contains(&"garantia G03 no-comprobado motivo=permiso-insuficiente".to_owned()),
        "{g:?}"
    );
    assert!(
        primera(&hs, &r).contains(
            " plan=gitlab-premium impuestas=15 sustituidas=3 no-disponibles=1 no-comprobadas=1 "
        ),
        "{}",
        primera(&hs, &r)
    );
}

/// C4: un 403 al leer las puertas da `falla motivo=permiso-insuficiente
/// permiso=...` (y la proteccion `no-comprobado`, no `ok`); en GitHub, un
/// 403 en los permisos de la CI da su permiso y una CI que puede aprobar,
/// `falla motivo=ci-puede-aprobar`. GitLab no tiene esa lectura (el job
/// token es fijo, G16): `ok`.
#[test]
fn permisos_insuficientes_y_ci_que_aprueba_fallan() {
    let _serie = serie();
    for prov in [Prov::GitHub, Prov::GitLab] {
        let mut caso = Caso::bien(prov);
        caso.puertas_403 = true;
        let (hs, r, _) = doctor_contra(prov, &caso);
        let permiso = match prov {
            Prov::GitHub => "metadata:read",
            Prov::GitLab => "read_api",
        };
        assert!(
            lineas_de(&hs, "permisos").contains(&format!(
                "permisos leer-puertas falla motivo=permiso-insuficiente permiso={permiso} rol=persona"
            )),
            "{prov:?}: {:?}",
            lineas_de(&hs, "permisos")
        );
        assert_eq!(
            lineas_de(&hs, "proteccion"),
            [
                "proteccion develop no-comprobado motivo=permiso-insuficiente",
                "proteccion main no-comprobado motivo=permiso-insuficiente"
            ],
            "{prov:?}"
        );
        assert_eq!(codigo(&hs, &r), 3);
        // H1 de TASK-035: sin puertas leidas, las garantias cuya
        // configuracion se lee de ellas no salen impuestas.
        let (cuentas, ids) = match prov {
            Prov::GitHub => (
                " plan=github-free-publico-usuario impuestas=13 sustituidas=1 no-disponibles=0 no-comprobadas=6 ",
                vec!["G01", "G02", "G03", "G04", "G05", "G09"],
            ),
            Prov::GitLab => (
                " plan=gitlab-premium impuestas=14 sustituidas=3 no-disponibles=0 no-comprobadas=3 ",
                vec!["G02", "G03", "G06"],
            ),
        };
        let primera = doctor::lineas(&hs, &r)[0].clone();
        assert!(primera.contains(cuentas), "{prov:?}: {primera}");
        let g = lineas_de(&hs, "garantia");
        for id in ids {
            assert!(
                g.contains(&format!(
                    "garantia {id} no-comprobado motivo=puertas-no-leidas"
                )),
                "{prov:?} {id}: {g:?}"
            );
        }
        let v: serde_json::Value =
            serde_json::from_str(&doctor::salida(&hs, &r, true, Idioma::Es).stdout).unwrap();
        let g02 = v["hallazgos"]
            .as_array()
            .unwrap()
            .iter()
            .find(|h| h["area"] == "garantia" && h["id"] == "G02")
            .cloned()
            .unwrap_or_default();
        assert_eq!(g02["estado"], "no-comprobado", "{prov:?}: {g02}");
        assert!(g02.get("capacidad").is_none(), "{prov:?}: {g02}");
    }
    let mut caso = Caso::bien(Prov::GitHub);
    caso.ci = Ci::Prohibido;
    let (hs, _, _) = doctor_contra(Prov::GitHub, &caso);
    assert_eq!(
        lineas_de(&hs, "permisos"),
        ["permisos ci falla motivo=permiso-insuficiente permiso=administration:read rol=persona"]
    );
    caso.ci = Ci::PuedeAprobar;
    let (hs, r, _) = doctor_contra(Prov::GitHub, &caso);
    assert_eq!(
        lineas_de(&hs, "permisos"),
        ["permisos ci falla motivo=ci-puede-aprobar arreglo=desactivar-aprobacion-por-actions"]
    );
    assert_eq!(codigo(&hs, &r), 3);
    let (hs, _, _) = doctor_contra(Prov::GitLab, &Caso::bien(Prov::GitLab));
    assert_eq!(lineas_de(&hs, "permisos"), ["permisos ci ok"]);
}

/// C5: un check requerido que no esta entre los publicados en la cabeza de
/// `develop` da `falla motivo=check-sin-publicar` (o `aviso` si no hay
/// ninguno publicado, D4a); un check del modelo que la rama no exige da
/// `falla motivo=check-no-obligatorio`. Mismas lineas en las dos.
#[test]
fn checks_sin_publicar_o_no_obligatorios() {
    let _serie = serie();
    for prov in [Prov::GitHub, Prov::GitLab] {
        let mut caso = Caso::bien(prov);
        let (hs, _, recibidas) = doctor_contra(prov, &caso);
        assert_eq!(
            lineas_de(&hs, "check"),
            [
                "check verificacion ok rama=develop",
                "check revision-ia ok rama=develop",
                "check carril ok rama=develop"
            ],
            "{prov:?}: {recibidas:?}"
        );
        caso.publicados = vec!["verificacion", "carril", "otro"];
        let (hs, r, _) = doctor_contra(prov, &caso);
        assert_eq!(
            lineas_de(&hs, "check"),
            [
                "check verificacion ok rama=develop",
                "check revision-ia falla motivo=check-sin-publicar rama=develop",
                "check carril ok rama=develop"
            ],
            "{prov:?}"
        );
        assert_eq!(codigo(&hs, &r), 3);
        caso.publicados = Vec::new();
        let (hs, r, _) = doctor_contra(prov, &caso);
        assert!(
            lineas_de(&hs, "check")
                .iter()
                .all(|l| l.contains(" aviso motivo=check-sin-publicar rama=develop")),
            "{prov:?}: {:?}",
            lineas_de(&hs, "check")
        );
        assert_eq!(codigo(&hs, &r), 0, "{prov:?}");
        // GitHub: el ruleset no exige `revision-ia`; GitLab: sin «Pipelines
        // must succeed» no exige ninguno.
        caso.publicados = MODELO.to_vec();
        caso.requeridos = match prov {
            Prov::GitHub => vec!["verificacion", "carril"],
            Prov::GitLab => Vec::new(),
        };
        let (hs, r, _) = doctor_contra(prov, &caso);
        assert!(
            lineas_de(&hs, "check").contains(
                &"check revision-ia falla motivo=check-no-obligatorio rama=develop arreglo=tc2-init"
                    .to_owned()
            ),
            "{prov:?}: {:?}",
            lineas_de(&hs, "check")
        );
        assert_eq!(codigo(&hs, &r), 3);
    }
}

/// C11: `capacidades` con la tabla G01..G20 y el plan observado: con
/// distintos planes en GitHub y GitLab, `doctor` dice las garantias
/// sustituidas o no disponibles con motivo y arreglo, y las cuentas en la
/// primera linea; el mismo caso da la misma linea en las dos plataformas.
#[test]
fn capacidades_por_plan_en_las_dos_plataformas() {
    let _serie = serie();
    let primera = |hs: &[Hallazgo], r: &Resumen| doctor::lineas(hs, r)[0].clone();
    // GitHub, repo privado de una cuenta gratuita: rulesets con 403 de plan.
    let mut caso = Caso::bien(Prov::GitHub);
    caso.plan = Plan::GitHub {
        privado: true,
        org: false,
        sin_rulesets: true,
    };
    let (hs, r, _) = doctor_contra(Prov::GitHub, &caso);
    assert!(
        primera(&hs, &r).contains(
            " plan=github-free-privado-usuario impuestas=9 sustituidas=1 no-disponibles=10 "
        ),
        "{}",
        primera(&hs, &r)
    );
    let g = lineas_de(&hs, "garantia");
    assert!(g.contains(&"garantia G03 aviso motivo=plan capacidad=no-disponible arreglo=repo-publico-o-github-pro".to_owned()), "{g:?}");
    assert!(g.contains(&"garantia G10 aviso motivo=plan capacidad=sustituida por=al-dia-por-cli arreglo=org-publica-o-enterprise-cloud".to_owned()), "{g:?}");
    // GitLab Free y Premium (sin propietarios de codigo: G06 por config).
    let mut caso = Caso::bien(Prov::GitLab);
    caso.plan = Plan::GitLab("free");
    let (hs, r, _) = doctor_contra(Prov::GitLab, &caso);
    assert!(
        primera(&hs, &r).contains(" plan=gitlab-free impuestas=11 sustituidas=9 no-disponibles=0 "),
        "{}",
        primera(&hs, &r)
    );
    let g = lineas_de(&hs, "garantia");
    assert!(g.contains(&"garantia G03 aviso motivo=plan capacidad=sustituida por=merge-solo-persona arreglo=gitlab-premium".to_owned()), "{g:?}");
    assert!(g.contains(&"garantia G01 aviso motivo=plataforma capacidad=sustituida por=maintainer-solo-persona arreglo=ninguno-limite-de-la-plataforma".to_owned()), "{g:?}");
    caso.plan = Plan::GitLab("premium");
    let (hs, r, _) = doctor_contra(Prov::GitLab, &caso);
    assert!(
        primera(&hs, &r)
            .contains(" plan=gitlab-premium impuestas=16 sustituidas=3 no-disponibles=1 "),
        "{}",
        primera(&hs, &r)
    );
    assert!(lineas_de(&hs, "garantia").contains(
        &"garantia G06 aviso motivo=config capacidad=no-disponible arreglo=tc2-init".to_owned()
    ));
    // El mismo caso, la misma linea: `develop` sin solo-PR en un plan que
    // lo da; y el plan que no se puede leer.
    let mismo = |preparar: &dyn Fn(&mut Caso, Prov), buscar: &str| -> Vec<String> {
        [Prov::GitHub, Prov::GitLab]
            .into_iter()
            .map(|prov| {
                let mut caso = Caso::bien(prov);
                preparar(&mut caso, prov);
                let (hs, r, _) = doctor_contra(prov, &caso);
                let l: Vec<String> = lineas_de(&hs, "garantia")
                    .into_iter()
                    .filter(|l| l.starts_with(buscar))
                    .collect();
                format!(
                    "{l:?} {}",
                    primera(&hs, &r).contains(" plan=no-comprobado ")
                )
            })
            .collect()
    };
    let v = mismo(
        &|c, prov| {
            c.protegidas = vec!["main"];
            if prov == Prov::GitLab {
                c.plan = Plan::GitLab("free");
            }
        },
        "garantia G02 ",
    );
    assert_eq!(v[0], v[1]);
    assert_eq!(
        v[0],
        "[\"garantia G02 aviso motivo=config capacidad=no-disponible arreglo=tc2-init\"] false"
    );
    let v = mismo(&|c, _| c.plan = Plan::Ilegible, "garantia plan ");
    assert_eq!(v[0], v[1]);
    assert_eq!(
        v[0],
        "[\"garantia plan no-comprobado motivo=plan-no-comprobado\"] true"
    );
}

/// C7: con el binario, como mucho 5 lineas en stdout, la primera fija y
/// `y N mas: tc2 doctor --json` si no caben; `--json` en una linea con
/// todos los hallazgos y sin ANSI.
#[test]
fn salida_acotada_y_json_con_todo() {
    let f = Fixture::nueva("doctor-c7");
    let (_, clon) = con_estado(&f, &[]);
    let (c, out, err) = tc2(&f, &clon, &["doctor"], &[]);
    assert!(c == 0 || c == 3, "{out} {err}");
    let lineas: Vec<&str> = out.lines().collect();
    assert!(lineas.len() <= 5, "{out}");
    assert!(lineas[0].starts_with("doctor "), "{out}");
    assert!(!out.contains('\u{1b}') && !err.contains('\u{1b}'));
    let (cj, json, _) = tc2(&f, &clon, &["doctor", "--json"], &[]);
    assert_eq!(cj, c);
    comun::salida_acotada(&lineas, &json, "tc2 doctor --json");
}

/// Lo que `doctor` no puede cambiar: refs, `git worktree list`, el arbol de
/// trabajo de cada worktree (con ignorados), el indice, `config`,
/// `info/exclude` y `<git-common-dir>/taskcodetwo/` (rutas y contenido).
fn foto(f: &Fixture, clon: &Path) -> String {
    let mut s = f.crudo(clon, &["for-each-ref", "--format=%(refname) %(objectname)"]);
    let wts = f.crudo(clon, &["worktree", "list", "--porcelain"]);
    s += &wts;
    for l in wts.lines() {
        if let Some(r) = l.strip_prefix("worktree ") {
            s += &f.crudo(Path::new(r), &["status", "--porcelain", "--ignored"]);
        }
    }
    let git = clon.join(".git");
    for fichero in ["index", "config", "info/exclude"] {
        s += &format!("{fichero}:{:?}\n", std::fs::read(git.join(fichero)).ok());
    }
    s += &format!(
        "{:?}
",
        comun::instantanea(&git.join("taskcodetwo"), &[])
    );
    s
}

/// C10: `doctor` no escribe nada: la foto del clon es la misma antes y
/// despues (biblioteca contra la plataforma simulada y binario, texto y
/// JSON); el mock solo recibe `GET` y git solo lanza lecturas.
#[test]
fn doctor_no_escribe_nada() {
    let _serie = serie();
    let f = Fixture::nueva("doctor-c10");
    let (_, clon) = con_estado(&f, &[]);
    assert!(estado::montar(&f.git(), &clon).resultado.es_exito());
    escribir(&clon, ".gitignore", "*.log\n");
    escribir(&clon, "ignorado.log", "x\n");
    escribir(&clon.join("tareas"), "TASK-001/notas.txt", "sin publicar\n");
    escribir(&clon, ".git/taskcodetwo/identidad.yml", "x: 1\n");
    let antes = foto(&f, &clon);
    for prov in [Prov::GitHub, Prov::GitLab] {
        // Solo importa el efecto: `hallazgos` ya falla si `comprobar` da `Err`.
        let (_, _, recibidas, g) = doctor_contra_en(&f, &clon, prov, &Caso::bien(prov));
        assert_eq!(foto(&f, &clon), antes, "{prov:?} cambio algo");
        assert!(!recibidas.is_empty());
        assert!(
            recibidas.iter().all(|r| r.starts_with("GET ")),
            "{prov:?}: {recibidas:?}"
        );
        const LECTURAS: [&str; 11] = [
            "version",
            "worktree",
            "rev-parse",
            "ls-remote",
            "for-each-ref",
            "ls-tree",
            "cat-file",
            "show",
            "status",
            "merge-base",
            "show-ref",
        ];
        for l in g.registro() {
            let mut t = l.split(' ').skip(1);
            let sub = t.next().unwrap_or_default();
            assert!(LECTURAS.contains(&sub), "git que no es de lectura: {l}");
            if sub == "worktree" {
                assert_eq!(t.next(), Some("list"), "{l}");
            }
        }
    }
    for args in [vec!["doctor"], vec!["doctor", "--json"]] {
        let (c, out, err) = tc2(&f, &clon, &args, &[]);
        assert!(c == 0 || c == 3, "{args:?}: {out} {err}");
        assert_eq!(foto(&f, &clon), antes, "{args:?} cambio algo");
    }
}

/// C9: sin plataforma, sin credenciales o sin red, `plataforma=` lo dice y
/// los hallazgos de plataforma salen `no-comprobado` (ninguno `ok`); el
/// codigo no cambia por eso.
#[test]
fn sin_red_sin_credenciales_o_sin_plataforma_no_comprueba() {
    let _serie = serie();
    let f = Fixture::nueva("doctor-c9");
    let (_, clon) = con_estado(&f, &[]);
    let g = f.git();
    let fallo = |c: Consulta| move |_: &Personas| -> Result<Conectada, Consulta> { Err(c.clone()) };
    for (consulta, nombre) in [
        (Consulta::NoConfigurada, "no-configurada"),
        (Consulta::SinCredenciales, "sin-credenciales"),
    ] {
        let conectar = fallo(consulta);
        let (hs, r) = hallazgos(&g, &clon, &conectar);
        assert_eq!(r.plataforma, nombre);
        let de_plataforma: Vec<&Hallazgo> = hs
            .iter()
            .filter(|h| ["proteccion", "permisos", "check", "garantia"].contains(&h.area))
            .collect();
        assert!(!de_plataforma.is_empty());
        assert!(
            de_plataforma
                .iter()
                .all(|h| h.estado == Estado::NoComprobado),
            "{de_plataforma:?}"
        );
        assert!(
            doctor::lineas(&hs, &r)[0].contains(&format!(" plataforma={nombre}")),
            "{:?}",
            doctor::lineas(&hs, &r)
        );
    }
    // Sin red: nadie escucha en el puerto 1, en las dos plataformas.
    for prov in [Prov::GitHub, Prov::GitLab] {
        let sin_red = conectar(prov, "http://127.0.0.1:1".to_owned());
        let (hs, r) = hallazgos(&g, &clon, &sin_red);
        assert_eq!(r.plataforma, "sin-red", "{prov:?}");
        assert_eq!(
            lineas_de(&hs, "proteccion"),
            [
                "proteccion develop no-comprobado motivo=sin-red",
                "proteccion main no-comprobado motivo=sin-red"
            ]
        );
        // R1-05 de TASK-035: tambien `check` y `garantia`.
        for area in ["proteccion", "permisos", "check", "garantia"] {
            let de_area: Vec<&Hallazgo> = hs.iter().filter(|h| h.area == area).collect();
            assert!(!de_area.is_empty(), "{prov:?} {area}");
            assert!(
                de_area
                    .iter()
                    .all(|h| h.estado == Estado::NoComprobado
                        && h.motivo.as_deref() == Some("sin-red")),
                "{prov:?} {area}: {de_area:?}"
            );
        }
    }
    // El binario, con la plataforma de produccion: `origin` es una carpeta,
    // asi que no hay plataforma reconocida.
    let (c, out, err) = tc2(&f, &clon, &["doctor"], &[]);
    assert!(c == 0 || c == 3, "{out} {err}");
    assert!(
        out.lines()
            .next()
            .unwrap()
            .contains(" plataforma=no-configurada"),
        "{out}"
    );
}
