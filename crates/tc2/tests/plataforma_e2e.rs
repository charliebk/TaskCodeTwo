//! F2-6, criterio 1: E2E del adaptador GitHub contra un repo de pruebas real.
//!
//! Ignorados por defecto (escriben en GitHub y necesitan red). Se lanzan a
//! mano con:
//!
//! ```text
//! TC2_E2E_REPO=dueno/repo TC2_E2E_TOKEN=... cargo test -p tc2 --test plataforma_e2e -- --ignored --nocapture --test-threads=1
//! ```
//!
//! El repo tiene que ser publico (los rulesets de GitHub Free solo funcionan
//! en publicos), con `main` y `develop`. Una sola cuenta hace de agente y de
//! persona (decision 2 del plan). El arnes crea ramas, commits y el ruleset
//! con su propio cliente (no es parte del puerto: `aplicar-puertas` es de
//! F3-1) y deja el repo como estaba: cierra los PR que no se mergean, borra
//! las ramas y el ruleset.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

use std::cell::{Cell, RefCell};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};
use tc2::contrato::Resultado;
use tc2::plataforma::http::Base;
use tc2::plataforma::{
    CredencialesFijas, GitHub, ModeloPuertas, Personas, Plataforma, Rol, Secreto,
};

struct Entorno {
    repo: String,
    token: String,
}

fn entorno() -> Entorno {
    let repo = std::env::var("TC2_E2E_REPO").expect("define TC2_E2E_REPO=dueno/repo");
    let token = std::env::var("TC2_E2E_TOKEN").expect("define TC2_E2E_TOKEN");
    Entorno { repo, token }
}

/// Cliente del arnes: llamadas que el puerto no tiene (crear ramas,
/// commits, rulesets, reviews; borrar), solo para preparar y limpiar.
struct Arnes {
    agente: ureq::Agent,
    e: Entorno,
}

impl Arnes {
    fn nuevo() -> Self {
        let agente: ureq::Agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .tls_config(
                ureq::tls::TlsConfig::builder()
                    .root_certs(ureq::tls::RootCerts::PlatformVerifier)
                    .build(),
            )
            .build()
            .into();
        Self {
            agente,
            e: entorno(),
        }
    }

    fn api(&self, metodo: &str, ruta: &str, cuerpo: Option<Value>) -> (u16, Value) {
        self.intentar(metodo, ruta, cuerpo).expect("red")
    }

    /// Como `api`, sin entrar en panico: para limpiar desde `Drop`.
    fn intentar(&self, metodo: &str, ruta: &str, cuerpo: Option<Value>) -> Option<(u16, Value)> {
        let url = format!("https://api.github.com/repos/{}{ruta}", self.e.repo);
        let auth = format!("Bearer {}", self.e.token);
        fn cab<B>(p: ureq::RequestBuilder<B>, auth: &str) -> ureq::RequestBuilder<B> {
            p.header("Authorization", auth)
                .header("Accept", "application/vnd.github+json")
                .header("Content-Type", "application/json")
                .header("User-Agent", "taskcodetwo-e2e")
        }
        let r = match (metodo, cuerpo) {
            ("GET", _) => cab(self.agente.get(&url), &auth).call(),
            ("DELETE", _) => cab(self.agente.delete(&url), &auth).call(),
            ("POST", Some(c)) => cab(self.agente.post(&url), &auth).send(c.to_string()),
            ("PUT", Some(c)) => cab(self.agente.put(&url), &auth).send(c.to_string()),
            ("PATCH", Some(c)) => cab(self.agente.patch(&url), &auth).send(c.to_string()),
            _ => return None,
        };
        let mut r = r.ok()?;
        let estado = r.status().as_u16();
        let texto = r.body_mut().read_to_string().unwrap_or_default();
        Some((estado, serde_json::from_str(&texto).unwrap_or(Value::Null)))
    }

    fn sha_de(&self, rama: &str) -> String {
        let (e, v) = self.api("GET", &format!("/commits/{rama}"), None);
        assert_eq!(e, 200, "{v}");
        v["sha"].as_str().unwrap().to_owned()
    }

    /// Rama nueva desde `base`, con un commit; devuelve el sha del commit.
    fn rama_con_commit(&self, rama: &str, base: &str) -> String {
        let sha = self.sha_de(base);
        let (e, v) = self.api(
            "POST",
            "/git/refs",
            Some(json!({"ref": format!("refs/heads/{rama}"), "sha": sha})),
        );
        assert_eq!(e, 201, "{v}");
        let contenido = "cHJ1ZWJhIEUyRSBkZSBUQVNLLTAyMQo="; // «prueba E2E de TASK-021»
        let (e, v) = self.api(
            "PUT",
            &format!("/contents/e2e/{rama}.txt"),
            Some(json!({"message": "e2e: commit de prueba", "content": contenido, "branch": rama})),
        );
        assert_eq!(e, 201, "{v}");
        v["commit"]["sha"].as_str().unwrap().to_owned()
    }

    fn borrar_rama(&self, rama: &str) {
        let _ = self.intentar("DELETE", &format!("/git/refs/heads/{rama}"), None);
    }

    fn cerrar_pr(&self, n: &str) {
        let _ = self.intentar(
            "PATCH",
            &format!("/pulls/{n}"),
            Some(json!({"state": "closed"})),
        );
    }
}

/// Deja el repo de pruebas como estaba aunque una asercion falle a medias:
/// cierra los PR, borra las ramas y el ruleset que se hayan apuntado.
struct Limpieza<'a> {
    arnes: &'a Arnes,
    prs: RefCell<Vec<String>>,
    ramas: RefCell<Vec<String>>,
    ruleset: Cell<Option<u64>>,
}

impl<'a> Limpieza<'a> {
    fn nueva(arnes: &'a Arnes) -> Self {
        Self {
            arnes,
            prs: RefCell::default(),
            ramas: RefCell::default(),
            ruleset: Cell::new(None),
        }
    }

    fn rama(&self, r: &str) {
        self.ramas.borrow_mut().push(r.to_owned());
    }

    fn pr(&self, n: &str) {
        self.prs.borrow_mut().push(n.to_owned());
    }
}

impl Drop for Limpieza<'_> {
    fn drop(&mut self) {
        for n in self.prs.borrow().iter() {
            self.arnes.cerrar_pr(n);
        }
        for r in self.ramas.borrow().iter() {
            self.arnes.borrar_rama(r);
        }
        if let Some(id) = self.ruleset.get() {
            let _ = self
                .arnes
                .intentar("DELETE", &format!("/rulesets/{id}"), None);
        }
    }
}

fn adaptador(repo: &str, token: &str, personas: Personas) -> GitHub {
    let s = Secreto::nuevo(token);
    let c = CredencialesFijas::nuevas()
        .con(Rol::Agente, s.clone())
        .con(Rol::Persona, s);
    GitHub::nuevo(Base::github(), repo, Box::new(c), personas).unwrap()
}

fn sufijo() -> String {
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("{}-{}", t.as_secs(), std::process::id())
}

fn ver(i: &tc2::contrato::Informe) -> String {
    let l = i.linea();
    eprintln!("  {l}");
    l
}

/// (a) Recorrido: abrir-pr idempotente, estado-puertas, merge-pr
/// idempotente, firma del merge (web-flow no cuenta), permisos de la CI,
/// lectura de reglas, y errores 401, 404 y sin commits dirigidos.
#[test]
#[ignore = "E2E: escribe en el repo de pruebas de GitHub (TC2_E2E_REPO, TC2_E2E_TOKEN)"]
fn recorrido_contra_github() {
    let arnes = Arnes::nuevo();
    let (repo, token) = (arnes.e.repo.clone(), arnes.e.token.clone());
    let yo = adaptador(&repo, &token, Personas::default())
        .identidad(Rol::Agente)
        .unwrap();
    eprintln!("  identidad: login={} type={}", yo.login, yo.tipo);
    assert_eq!(yo.tipo, "User");
    let gh = adaptador(&repo, &token, Personas::nuevas([yo.id]));
    let rama = format!("e2e/a-{}", sufijo());
    let limpieza = Limpieza::nueva(&arnes);
    limpieza.rama(&rama);
    let sha = arnes.rama_con_commit(&rama, "main");

    let i = gh.abrir_pr(
        &rama,
        "main",
        "E2E TASK-021 (a)",
        "PR de prueba del adaptador",
    );
    assert_eq!(i.resultado, Resultado::Ok, "{}", ver(&i));
    ver(&i);
    let n = i.valor("pr").unwrap().to_owned();
    let i = gh.abrir_pr(&rama, "main", "E2E TASK-021 (a)", "otra vez");
    assert_eq!(ver(&i), format!("abrir-pr sin-cambios pr={n}"));

    let num: u64 = n.parse().unwrap();
    let modelo = ModeloPuertas::default();
    let i = gh.estado_puertas(Rol::Agente, num, &modelo);
    assert_eq!(
        ver(&i),
        format!("estado-puertas esperando pr={num} aprobaciones=0 ignoradas=0 falta=aprobacion")
    );
    let i = gh.estado_puertas(Rol::Persona, num, &modelo);
    assert_eq!(i.resultado, Resultado::Esperando, "{}", ver(&i));

    // Head distinto del evaluado: ni siquiera se intenta.
    let i = gh.merge_pr(num, &"0".repeat(40));
    assert_eq!(
        ver(&i),
        format!("merge-pr rechazada pr={num} motivo=head-cambiado")
    );
    // main no tiene reglas: la persona puede mergear.
    let i = gh.merge_pr(num, &sha);
    assert_eq!(i.resultado, Resultado::Ok, "{}", ver(&i));
    ver(&i);
    let i = gh.merge_pr(num, &sha);
    assert_eq!(
        ver(&i),
        format!("merge-pr sin-cambios pr={num} estado=mergeado")
    );
    let i = gh.estado_puertas(Rol::Agente, num, &modelo);
    assert_eq!(
        ver(&i),
        format!("estado-puertas sin-cambios pr={num} estado=mergeado")
    );

    // El merge lo firma GitHub (web-flow): verificado, pero no es una
    // aprobacion de persona (F1-6, regla 4).
    let merge = arnes.sha_de("main");
    let f = gh.firma_commit(&merge).unwrap();
    eprintln!(
        "  firma del merge: verificada={} motivo={} committer={:?}",
        f.verificada,
        f.motivo,
        f.committer.as_ref().map(|c| (&c.login, &c.tipo))
    );
    assert!(f.verificada);
    assert_eq!(
        f.committer.as_ref().map(|c| c.login.as_str()),
        Some("web-flow")
    );
    assert!(!f.cuenta_como_aprobacion(&Personas::nuevas([yo.id])));

    let i = gh.leer_permisos_ci();
    assert_eq!(i.resultado, Resultado::Ok, "{}", ver(&i));
    assert_eq!(i.valor("ci-aprueba"), Some("false"));
    ver(&i);
    let reglas = gh.leer_puertas(Rol::Agente, "main").unwrap();
    eprintln!("  leer-puertas main: {reglas:?}");
    assert!(reglas.plan_da_reglas);

    // Sin commits nuevos respecto a la base.
    let vacia = format!("e2e/vacia-{}", sufijo());
    limpieza.rama(&vacia);
    let base_sha = arnes.sha_de("main");
    let (e, _) = arnes.api(
        "POST",
        "/git/refs",
        Some(json!({"ref": format!("refs/heads/{vacia}"), "sha": base_sha})),
    );
    assert_eq!(e, 201);
    let i = gh.abrir_pr(&vacia, "main", "E2E vacia", "");
    assert_eq!(ver(&i), "abrir-pr rechazada motivo=sin-commits");

    // Errores dirigidos: token que no vale y repo inexistente.
    let malo = adaptador(
        &repo,
        "ghp_token_que_no_existe_00000000000000",
        Personas::default(),
    );
    let s = malo.identidad(Rol::Agente).unwrap_err().salida(false);
    eprintln!(
        "  {}  [{}] {}",
        s.stdout.trim_end(),
        s.codigo,
        s.stderr.trim_end()
    );
    assert_eq!(
        (s.stdout.as_str(), s.codigo),
        ("identidad denegada motivo=credenciales rol=agente\n", 8)
    );
    assert!(s.stderr.contains("TC2_TOKEN_AGENTE"));
    let dueno = repo.split('/').next().unwrap();
    let otro = adaptador(
        &format!("{dueno}/no-existe-tc2-e2e"),
        &token,
        Personas::default(),
    );
    let s = otro.estado_puertas(Rol::Agente, 1, &modelo).salida(false);
    eprintln!(
        "  {}  [{}] {}",
        s.stdout.trim_end(),
        s.codigo,
        s.stderr.trim_end()
    );
    assert_eq!(
        (s.stdout.as_str(), s.codigo),
        (
            "estado-puertas denegada motivo=no-existe-o-sin-acceso rol=agente\n",
            8
        )
    );
    eprintln!("  llamadas del adaptador: {}", gh.registro().len());

    drop(limpieza);
}

/// (b) Puertas reales: con un ruleset en `develop` que exige 1 aprobacion,
/// GitHub bloquea el merge, el autor no puede aprobar su PR y la review
/// propia no cuenta.
#[test]
#[ignore = "E2E: escribe en el repo de pruebas de GitHub (TC2_E2E_REPO, TC2_E2E_TOKEN)"]
fn puertas_reales_contra_github() {
    let arnes = Arnes::nuevo();
    let (repo, token) = (arnes.e.repo.clone(), arnes.e.token.clone());
    let yo = adaptador(&repo, &token, Personas::default())
        .identidad(Rol::Agente)
        .unwrap();
    let gh = adaptador(&repo, &token, Personas::nuevas([yo.id]));
    let nombre = format!("tc2-e2e-{}", sufijo());
    let (e, v) = arnes.api(
        "POST",
        "/rulesets",
        Some(json!({
            "name": nombre, "target": "branch", "enforcement": "active",
            "conditions": {"ref_name": {"include": ["refs/heads/develop"], "exclude": []}},
            "rules": [{"type": "pull_request", "parameters": {
                "required_approving_review_count": 1,
                "dismiss_stale_reviews_on_push": false,
                "require_code_owner_review": false,
                "require_last_push_approval": false,
                "required_review_thread_resolution": false}}]
        })),
    );
    assert_eq!(e, 201, "{v}");
    let limpieza = Limpieza::nueva(&arnes);
    limpieza.ruleset.set(v["id"].as_u64());
    let rama = format!("e2e/b-{}", sufijo());
    limpieza.rama(&rama);
    let sha = arnes.rama_con_commit(&rama, "develop");

    let reglas = gh.leer_puertas(Rol::Agente, "develop").unwrap();
    eprintln!("  leer-puertas develop: {reglas:?}");
    assert!(reglas.rulesets.contains(&nombre));
    assert!(reglas.reglas_rama.iter().any(|t| t == "pull_request"));

    let i = gh.abrir_pr(&rama, "develop", "E2E TASK-021 (b)", "PR con puertas");
    assert_eq!(i.resultado, Resultado::Ok, "{}", ver(&i));
    ver(&i);
    let n = i.valor("pr").unwrap().to_owned();
    limpieza.pr(&n);
    let num: u64 = n.parse().unwrap();
    let modelo = ModeloPuertas {
        checks: Vec::new(),
        exigir_impuestas: vec!["G03".into()],
    };
    let i = gh.estado_puertas(Rol::Agente, num, &modelo);
    assert_eq!(
        ver(&i),
        format!("estado-puertas esperando pr={num} aprobaciones=0 ignoradas=0 falta=aprobacion")
    );

    // El autor no puede aprobar su propio PR (documental en F1-6).
    let (e, v) = arnes.api(
        "POST",
        &format!("/pulls/{n}/reviews"),
        Some(json!({"event": "APPROVE", "body": "auto-aprobacion (tiene que fallar)"})),
    );
    eprintln!("  auto-aprobacion: {e} {}", v["errors"]);
    assert_eq!(e, 422, "{v}");
    // Una review de comentario: forma real de GET /reviews, y no cuenta.
    let (e, v) = arnes.api(
        "POST",
        &format!("/pulls/{n}/reviews"),
        Some(json!({"event": "COMMENT", "body": "comentario de prueba"})),
    );
    assert_eq!(e, 200, "{v}");
    let (_, revs) = arnes.api("GET", &format!("/pulls/{n}/reviews"), None);
    let r0 = &revs[0];
    eprintln!(
        "  forma de una review: state={} user.type={} user.id presente={} claves={:?}",
        r0["state"],
        r0["user"]["type"],
        r0["user"]["id"].is_u64(),
        r0.as_object()
            .map(|o| o.keys().cloned().collect::<Vec<_>>())
    );
    assert_eq!(r0["state"], "COMMENTED");
    let i = gh.estado_puertas(Rol::Agente, num, &modelo);
    assert_eq!(i.resultado, Resultado::Esperando, "{}", ver(&i));

    // La plataforma bloquea el merge sin aprobacion (documental en F1-13).
    let i = gh.merge_pr(num, &sha);
    let s = i.salida(false);
    eprintln!(
        "  {}  [{}] {}",
        s.stdout.trim_end(),
        s.codigo,
        s.stderr.trim_end()
    );
    assert_eq!(
        s.stdout,
        format!("merge-pr rechazada pr={num} motivo=no-mergeable\n")
    );
    assert_eq!(s.codigo, 3);

    drop(limpieza);
    let (_, quedan) = arnes.api("GET", "/rulesets", None);
    assert!(
        !quedan.to_string().contains(&nombre),
        "el ruleset del E2E se ha borrado"
    );
}
