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
//!
//! F6-1 (TASK-047), `persona_aprueba_pr_del_agente`: el agente es una GitHub
//! App instalada en el repo de pruebas y la persona, la cuenta de
//! `TC2_E2E_TOKEN`. Variables:
//!
//! - `TC2_E2E_REPO`: `dueno/repo` (el mismo de arriba).
//! - `TC2_E2E_TOKEN`: token de la persona (aprueba, cierra y limpia).
//! - `TC2_E2E_APP_ID`: id de la App del agente.
//! - `TC2_E2E_APP_CLAVE`: ruta al PEM de la clave privada de esa App.
//!
//! ```text
//! TC2_E2E_REPO=dueno/repo TC2_E2E_TOKEN=... TC2_E2E_APP_ID=... TC2_E2E_APP_CLAVE=/ruta/app.pem cargo test -p tc2 --test plataforma_e2e persona_aprueba_pr_del_agente -- --ignored --nocapture --test-threads=1
//! ```
//!
//! F6-2 (TASK-048), `review_abre_el_pr_del_agente_contra_develop`: las
//! mismas variables (la persona solo prepara y limpia; el push y el PR los
//! hace la App con `tc2::revision::publicar`). El repo necesita `develop`.
//! Cierra el PR y borra la rama aunque falle a medias.
//!
//! ```text
//! TC2_E2E_REPO=dueno/repo TC2_E2E_TOKEN=... TC2_E2E_APP_ID=... TC2_E2E_APP_CLAVE=/ruta/app.pem cargo test -p tc2 --test plataforma_e2e review_abre_el_pr_del_agente_contra_develop -- --ignored --nocapture --test-threads=1
//! ```

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod agente_git;

use std::cell::{Cell, RefCell};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};
use tc2::contrato::Resultado;
use tc2::plataforma::credenciales::de_entorno_con;
use tc2::plataforma::http::{Base, Cliente, Metodo};
use tc2::plataforma::{
    AutorGit, Credenciales, CredencialesApp, CredencialesFijas, GitHub, ModeloPuertas, Personas,
    Plataforma, Proveedor, Rol, Secreto,
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
        self.intentar_con(&self.e.token, metodo, &url, cuerpo)
    }

    /// Llamada a una URL completa con otro token (el de la App del agente
    /// en F6-1).
    fn intentar_con(
        &self,
        token: &str,
        metodo: &str,
        url: &str,
        cuerpo: Option<Value>,
    ) -> Option<(u16, Value)> {
        let auth = format!("Bearer {token}");
        fn cab<B>(p: ureq::RequestBuilder<B>, auth: &str) -> ureq::RequestBuilder<B> {
            p.header("Authorization", auth)
                .header("Accept", "application/vnd.github+json")
                .header("Content-Type", "application/json")
                .header("User-Agent", "taskcodetwo-e2e")
        }
        let r = match (metodo, cuerpo) {
            ("GET", _) => cab(self.agente.get(url), &auth).call(),
            ("DELETE", _) => cab(self.agente.delete(url), &auth).call(),
            ("POST", Some(c)) => cab(self.agente.post(url), &auth).send(c.to_string()),
            ("PUT", Some(c)) => cab(self.agente.put(url), &auth).send(c.to_string()),
            ("PATCH", Some(c)) => cab(self.agente.patch(url), &auth).send(c.to_string()),
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

/// (c) F6-1: el agente, como GitHub App, empuja una rama por git con su
/// token de instalacion (fuera la configuracion de git del usuario) y abre
/// el PR; la App no puede aprobar su propio PR; la persona lo aprueba y
/// `estado-puertas` con la persona cuenta esa aprobacion.
#[test]
#[ignore = "E2E: escribe en el repo de pruebas de GitHub (TC2_E2E_REPO, TC2_E2E_TOKEN, TC2_E2E_APP_ID, TC2_E2E_APP_CLAVE)"]
fn persona_aprueba_pr_del_agente() {
    let arnes = Arnes::nuevo();
    let (repo, token) = (arnes.e.repo.clone(), arnes.e.token.clone());
    let app_id = std::env::var("TC2_E2E_APP_ID").expect("define TC2_E2E_APP_ID");
    let clave = std::env::var("TC2_E2E_APP_CLAVE").expect("define TC2_E2E_APP_CLAVE");
    let yo = adaptador(&repo, &token, Personas::default())
        .identidad(Rol::Persona)
        .unwrap();

    // La App del agente: slug, cuenta [bot] y token de instalacion.
    let app = CredencialesApp::desde_fichero(
        &app_id,
        std::path::Path::new(&clave),
        &Base::github(),
        &repo,
        Rol::Agente,
    )
    .unwrap();
    let slug = app.slug().unwrap();
    let bot = format!("{slug}[bot]");
    let (e, cuenta) = arnes
        .intentar_con(
            &token,
            "GET",
            &format!("https://api.github.com/users/{bot}"),
            None,
        )
        .expect("red");
    assert_eq!(e, 200, "{cuenta}");
    assert_eq!(cuenta["type"], "Bot", "{cuenta}");
    let token_app = app.obtener(Rol::Agente).unwrap();
    let autor = AutorGit::bot_github(&slug, cuenta["id"].as_u64());

    let limpieza = Limpieza::nueva(&arnes);
    let rama = format!("e2e/agente-{}", sufijo());
    limpieza.rama(&rama);
    let (_clon, sha) = agente_git::publicar_rama(
        Proveedor::GitHub,
        &format!("https://github.com/{repo}.git"),
        "https://github.com/",
        &token_app,
        &autor,
        ("main", &rama),
    );
    let (e, c) = arnes.api("GET", &format!("/commits/{sha}"), None);
    assert_eq!(e, 200, "{c}");
    eprintln!(
        "  commit {sha}: autor={} login={}",
        c["commit"]["author"]["name"], c["author"]["login"]
    );
    assert_eq!(c["commit"]["author"]["name"], bot.as_str());

    // El adaptador con la fabrica de produccion: App del agente y token de
    // persona (con terminal).
    let vars = |v: &str| match v {
        "TC2_APP_AGENTE_ID" => Some(app_id.clone()),
        "TC2_APP_AGENTE_CLAVE" => Some(clave.clone()),
        "TC2_TOKEN_PERSONA" => Some(token.clone()),
        _ => None,
    };
    let gh = GitHub::nuevo(
        Base::github(),
        &repo,
        Box::new(de_entorno_con(vars, true)),
        Personas::nuevas([yo.id]),
    )
    .unwrap();
    let i = gh.abrir_pr(&rama, "main", "E2E TASK-047", "PR del agente (F6-1)");
    assert_eq!(i.resultado, Resultado::Ok, "{}", ver(&i));
    ver(&i);
    let n = i.valor("pr").unwrap().to_owned();
    limpieza.pr(&n);
    let num: u64 = n.parse().unwrap();
    let (e, pr) = arnes.api("GET", &format!("/pulls/{n}"), None);
    assert_eq!(e, 200, "{pr}");
    assert_eq!(pr["user"]["login"], bot.as_str(), "el PR lo abre la App");
    assert_eq!(pr["user"]["type"], "Bot");

    // La App no puede aprobar su propio PR (no verificado en F1-6).
    // Con el `Cliente` de la biblioteca, que recibe el `Secreto` sin
    // exponerlo (MEN-5 de la ronda 1 de TASK-047).
    let cuerpo =
        json!({"event": "APPROVE", "body": "auto-aprobacion de la App (tiene que fallar)"})
            .to_string();
    let r = Cliente::nuevo(Base::github())
        .llamar(
            Metodo::Post,
            &format!("/repos/{repo}/pulls/{n}/reviews"),
            Some(&token_app),
            Some(&cuerpo),
        )
        .expect("red");
    let (e, v) = (r.estado, r.cuerpo);
    eprintln!("  auto-aprobacion de la App: {e} {v}");
    assert!(
        (400..500).contains(&e),
        "la App aprobo su propio PR: {e} {v}"
    );

    let modelo = ModeloPuertas::default();
    let i = gh.estado_puertas(Rol::Persona, num, &modelo);
    assert_eq!(i.valor("aprobaciones"), Some("0"), "{}", ver(&i));

    // La persona aprueba.
    let (e, v) = arnes.api(
        "POST",
        &format!("/pulls/{n}/reviews"),
        Some(json!({"event": "APPROVE", "body": "aprobado por la persona (E2E TASK-047)"})),
    );
    assert_eq!(e, 200, "{v}");
    let i = gh.estado_puertas(Rol::Persona, num, &modelo);
    assert_eq!(i.valor("aprobaciones"), Some("1"), "{}", ver(&i));
    ver(&i);
    // Y el agente ve lo mismo con su token de instalacion.
    let i = gh.estado_puertas(Rol::Agente, num, &modelo);
    assert_eq!(i.valor("aprobaciones"), Some("1"), "{}", ver(&i));

    drop(limpieza);
}

/// (d) F6-2 (TASK-048): lo que hace `review` con la plataforma, con la
/// fabrica de produccion y solo la App del agente: empuja la rama con el
/// entorno de `entorno-push` (sin la configuracion de git del usuario ni la
/// cabecera de F6-1), abre el PR contra `develop` con el titulo y el cuerpo
/// de la tarea (lo abre la App), repetir converge y `revision-ia` sale
/// `pendiente` (nadie lo publica con el origen dado). No escribe en la rama
/// de estado del repo de pruebas: ese paso lo cubre `tests/revision.rs`.
/// Limpia el PR y la rama.
#[test]
#[ignore = "E2E: escribe en el repo de pruebas de GitHub (TC2_E2E_REPO, TC2_E2E_TOKEN, TC2_E2E_APP_ID, TC2_E2E_APP_CLAVE)"]
fn review_abre_el_pr_del_agente_contra_develop() {
    use tc2::git::{self, Git};
    use tc2::revision::{self, Publicado};

    let arnes = Arnes::nuevo();
    let repo = arnes.e.repo.clone();
    let app_id = std::env::var("TC2_E2E_APP_ID").expect("define TC2_E2E_APP_ID");
    let clave = std::env::var("TC2_E2E_APP_CLAVE").expect("define TC2_E2E_APP_CLAVE");
    let app = CredencialesApp::desde_fichero(
        &app_id,
        std::path::Path::new(&clave),
        &Base::github(),
        &repo,
        Rol::Agente,
    )
    .unwrap();
    let slug = app.slug().unwrap();
    let bot = format!("{slug}[bot]");
    let token_app = app.obtener(Rol::Agente).unwrap();

    let limpieza = Limpieza::nueva(&arnes);
    let rama = format!("e2e/review-{}", sufijo());
    limpieza.rama(&rama);
    let url = format!("https://github.com/{repo}.git");
    let (clon, sha) = agente_git::preparar_rama(
        Proveedor::GitHub,
        &url,
        "https://github.com/",
        &token_app,
        &AutorGit::bot_github(&slug, None),
        ("develop", &rama),
    );
    let vars = |v: &str| match v {
        "TC2_APP_AGENTE_ID" => Some(app_id.clone()),
        "TC2_APP_AGENTE_CLAVE" => Some(clave.clone()),
        _ => None,
    };
    let gh = GitHub::nuevo(
        Base::github(),
        &repo,
        Box::new(de_entorno_con(vars, false)),
        Personas::default(),
    )
    .unwrap();
    let g = Git::nuevo().con_entorno(clon.aislado.clone());
    let t = tc2::tarea::Tarea::nueva("TASK-999", "E2E de review (TASK-048)", "2026-11-19");
    let (titulo, cuerpo) = (revision::titulo_pr(&t), revision::cuerpo_pr(&t));
    let datos = (rama.as_str(), titulo.as_str(), cuerpo.as_str());

    let p = revision::publicar(&gh, &g, &clon.dir, &url, datos, &git::ESPERAS_PUSH)
        .unwrap_or_else(|i| panic!("{}", ver(&i)));
    limpieza.pr(&p.pr.to_string());
    eprintln!("  publicado: {p:?}");
    assert!(p.empujado && p.abierto, "{p:?}");
    assert_eq!(arnes.sha_de(&rama), sha, "el push no llego");
    let (e, pr) = arnes.api("GET", &format!("/pulls/{}", p.pr), None);
    assert_eq!(e, 200, "{pr}");
    assert_eq!(pr["user"]["login"], bot.as_str(), "el PR lo abre la App");
    assert_eq!(pr["user"]["type"], "Bot");
    assert_eq!(pr["base"]["ref"], "develop");
    assert_eq!(pr["title"], titulo.as_str());
    assert_eq!(pr["body"], cuerpo.as_str());

    // Repetir converge: nada que empujar y el PR ya abierto.
    let q = revision::publicar(&gh, &g, &clon.dir, &url, datos, &git::ESPERAS_PUSH)
        .unwrap_or_else(|i| panic!("{}", ver(&i)));
    assert_eq!(
        q,
        Publicado {
            pr: p.pr,
            empujado: false,
            abierto: false
        }
    );

    // `revision-ia` con un origen que nadie usa: pendiente (o sin-revisor
    // sin origen).
    assert_eq!(revision::revision_ia(&gh, p.pr, None), "sin-revisor");
    let mut ria = "";
    for _ in 0..10 {
        ria = revision::revision_ia(&gh, p.pr, Some(1));
        if ria == "pendiente" {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
    eprintln!("  revision-ia={ria}");
    assert_eq!(ria, "pendiente");

    drop(clon);
    drop(limpieza);
}
