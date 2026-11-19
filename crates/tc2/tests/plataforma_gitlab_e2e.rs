//! F2-9, criterio 1: E2E del adaptador GitLab contra un proyecto de pruebas
//! real en gitlab.com.
//!
//! Ignorados por defecto (escriben en GitLab y necesitan red). Se lanzan a
//! mano con:
//!
//! ```text
//! TC2_E2E_GITLAB_PROYECTO=grupo/proyecto TC2_E2E_GITLAB_TOKEN=... cargo test -p tc2 --test plataforma_gitlab_e2e -- --ignored --nocapture --test-threads=1
//! ```
//!
//! El proyecto tiene `main` y `develop`. Una sola cuenta hace de agente y de
//! persona (decision 2 del plan). El arnes crea ramas, commits, estados de
//! commit y la proteccion de `develop` con su propio cliente (no es parte
//! del puerto: `aplicar-puertas` es de F3-1) y deja el proyecto como estaba:
//! cierra los MR que no se mergean, borra las ramas y quita la proteccion.
//! Los commits que llegan a `develop` se quedan (como en el E2E de F2-6).
//!
//! F6-1 (TASK-047), `persona_aprueba_mr_del_agente`: el agente es una cuenta
//! de servicio (o un token de proyecto) con rol Developer y la persona, la
//! cuenta de `TC2_E2E_GITLAB_TOKEN`. Variables:
//!
//! - `TC2_E2E_GITLAB_PROYECTO`: `grupo/proyecto` (puede cambiar; no se fija
//!   en el codigo).
//! - `TC2_E2E_GITLAB_TOKEN`: token de la persona (aprueba, cierra y limpia).
//! - `TC2_E2E_GITLAB_TOKEN_AGENTE`: token de la cuenta de servicio del
//!   agente.
//!
//! ```text
//! TC2_E2E_GITLAB_PROYECTO=grupo/proyecto TC2_E2E_GITLAB_TOKEN=... TC2_E2E_GITLAB_TOKEN_AGENTE=... cargo test -p tc2 --test plataforma_gitlab_e2e persona_aprueba_mr_del_agente -- --ignored --nocapture --test-threads=1
//! ```

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod agente_git;

use std::cell::{Cell, RefCell};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};
use tc2::contrato::{Informe, Resultado};
use tc2::plataforma::credenciales::de_entorno_con;
use tc2::plataforma::http::Base;
use tc2::plataforma::{
    AutorGit, CredencialesFijas, GitLab, ModeloPuertas, Personas, Plataforma, Proveedor, Rol,
    Secreto,
};

const API: &str = "https://gitlab.com/api/v4";

struct Entorno {
    proyecto: String,
    token: String,
}

fn entorno() -> Entorno {
    let proyecto = std::env::var("TC2_E2E_GITLAB_PROYECTO")
        .expect("define TC2_E2E_GITLAB_PROYECTO=grupo/proyecto");
    let token = std::env::var("TC2_E2E_GITLAB_TOKEN").expect("define TC2_E2E_GITLAB_TOKEN");
    Entorno { proyecto, token }
}

fn cod(s: &str) -> String {
    s.replace('/', "%2F")
}

/// Cliente del arnes: llamadas que el puerto no tiene (ramas, commits,
/// estados, proteccion, aprobar, cerrar), solo para preparar y limpiar.
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
        let url = format!("{API}/projects/{}{ruta}", cod(&self.e.proyecto));
        let auth = format!("Bearer {}", self.e.token);
        fn cab<B>(p: ureq::RequestBuilder<B>, auth: &str) -> ureq::RequestBuilder<B> {
            p.header("Authorization", auth)
                .header("Accept", "application/json")
                .header("Content-Type", "application/json")
                .header("User-Agent", "taskcodetwo-e2e")
        }
        let r = match (metodo, cuerpo) {
            ("GET", _) => cab(self.agente.get(&url), &auth).call(),
            ("DELETE", _) => cab(self.agente.delete(&url), &auth).call(),
            ("POST", Some(c)) => cab(self.agente.post(&url), &auth).send(c.to_string()),
            ("PUT", Some(c)) => cab(self.agente.put(&url), &auth).send(c.to_string()),
            _ => return None,
        };
        let mut r = r.ok()?;
        let estado = r.status().as_u16();
        let texto = r.body_mut().read_to_string().unwrap_or_default();
        Some((estado, serde_json::from_str(&texto).unwrap_or(Value::Null)))
    }

    /// Commit en `rama` (creandola desde `desde` si se da) que escribe
    /// `fichero`; devuelve su sha.
    fn commit(&self, rama: &str, desde: Option<&str>, fichero: &str, contenido: &str) -> String {
        let existe =
            self.api(
                "GET",
                &format!(
                    "/repository/files/{}?ref={}",
                    cod(fichero),
                    cod(desde.unwrap_or(rama))
                ),
                None,
            )
            .0 == 200;
        let mut c = json!({
            "branch": rama,
            "commit_message": "e2e: commit de prueba de TASK-022",
            "actions": [{
                "action": if existe { "update" } else { "create" },
                "file_path": fichero,
                "content": contenido,
            }],
        });
        if let Some(d) = desde {
            c["start_branch"] = d.into();
        }
        let (e, v) = self.api("POST", "/repository/commits", Some(c));
        assert_eq!(e, 201, "{v}");
        v["id"].as_str().unwrap().to_owned()
    }

    fn rama_sin_commits(&self, rama: &str, desde: &str) {
        let (e, v) = self.api(
            "POST",
            &format!(
                "/repository/branches?branch={}&ref={}",
                cod(rama),
                cod(desde)
            ),
            Some(json!({})),
        );
        assert_eq!(e, 201, "{v}");
    }

    fn estado(&self, sha: &str, nombre: &str, estado: &str) {
        let (e, v) = self.api(
            "POST",
            &format!("/statuses/{sha}"),
            Some(json!({"state": estado, "name": nombre})),
        );
        assert_eq!(e, 201, "{v}");
    }

    fn proteger_develop(&self) {
        let _ = self.intentar("DELETE", "/protected_branches/develop", None);
        let (e, v) = self.api(
            "POST",
            "/protected_branches",
            Some(json!({"name": "develop", "push_access_level": 40, "merge_access_level": 40})),
        );
        assert_eq!(e, 201, "{v}");
    }

    fn desproteger_develop(&self) {
        let _ = self.intentar("DELETE", "/protected_branches/develop", None);
    }

    fn borrar_rama(&self, rama: &str) {
        let _ = self.intentar(
            "DELETE",
            &format!("/repository/branches/{}", cod(rama)),
            None,
        );
    }

    fn cerrar_mr(&self, n: &str) {
        let _ = self.intentar(
            "PUT",
            &format!("/merge_requests/{n}"),
            Some(json!({"state_event": "close"})),
        );
    }

    /// Llamada con otro token (el del agente en F6-1) a una ruta de la API.
    fn intentar_con(
        &self,
        token: &str,
        metodo: &str,
        ruta: &str,
        cuerpo: Option<Value>,
    ) -> Option<(u16, Value)> {
        let url = format!("{API}{ruta}");
        let auth = format!("Bearer {token}");
        let p = match metodo {
            "GET" => self.agente.get(&url).header("Authorization", &auth).call(),
            "POST" => self
                .agente
                .post(&url)
                .header("Authorization", &auth)
                .header("Content-Type", "application/json")
                .send(cuerpo.unwrap_or_else(|| json!({})).to_string()),
            _ => return None,
        };
        let mut r = p.ok()?;
        let estado = r.status().as_u16();
        let texto = r.body_mut().read_to_string().unwrap_or_default();
        Some((estado, serde_json::from_str(&texto).unwrap_or(Value::Null)))
    }

    fn usuario(&self) -> u64 {
        let auth = format!("Bearer {}", self.e.token);
        let mut r = self
            .agente
            .get(format!("{API}/user"))
            .header("Authorization", &auth)
            .call()
            .unwrap();
        let v: Value = serde_json::from_str(&r.body_mut().read_to_string().unwrap()).unwrap();
        v["id"].as_u64().unwrap()
    }
}

/// Deja el proyecto de pruebas como estaba aunque una asercion falle a
/// medias.
struct Limpieza<'a> {
    arnes: &'a Arnes,
    mrs: RefCell<Vec<String>>,
    ramas: RefCell<Vec<String>>,
    protegida: Cell<bool>,
}

impl<'a> Limpieza<'a> {
    fn nueva(arnes: &'a Arnes) -> Self {
        Self {
            arnes,
            mrs: RefCell::default(),
            ramas: RefCell::default(),
            protegida: Cell::new(false),
        }
    }

    fn rama(&self, r: &str) {
        self.ramas.borrow_mut().push(r.to_owned());
    }

    fn mr(&self, n: &str) {
        self.mrs.borrow_mut().push(n.to_owned());
    }

    fn proteger(&self) {
        self.arnes.proteger_develop();
        self.protegida.set(true);
    }
}

impl Drop for Limpieza<'_> {
    fn drop(&mut self) {
        for n in self.mrs.borrow().iter() {
            self.arnes.cerrar_mr(n);
        }
        for r in self.ramas.borrow().iter() {
            self.arnes.borrar_rama(r);
        }
        if self.protegida.get() {
            self.arnes.desproteger_develop();
        }
    }
}

fn adaptador(proyecto: &str, token: &str, personas: Personas) -> GitLab {
    let s = Secreto::nuevo(token);
    let c = CredencialesFijas::nuevas()
        .con(Rol::Agente, s.clone())
        .con(Rol::Persona, s);
    GitLab::nuevo(Base::nueva(API).unwrap(), proyecto, Box::new(c), personas).unwrap()
}

fn sufijo() -> String {
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("{}-{}", t.as_secs(), std::process::id())
}

fn ver(i: &Informe) -> String {
    let l = i.linea();
    eprintln!("  {l}");
    l
}

fn modelo(checks: &[(&str, u64)]) -> ModeloPuertas {
    ModeloPuertas {
        checks: checks.iter().map(|(n, o)| ((*n).to_owned(), *o)).collect(),
        exigir_impuestas: Vec::new(),
    }
}

/// GitLab calcula la mezclabilidad en segundo plano: se repite la lectura
/// hasta que la linea cumple `listo` (o se agota la espera).
fn esperar(mut leer: impl FnMut() -> Informe, listo: impl Fn(&str) -> bool) -> String {
    let mut l = String::new();
    for _ in 0..15 {
        l = leer().linea();
        if listo(&l) {
            break;
        }
        std::thread::sleep(Duration::from_secs(2));
    }
    eprintln!("  {l}");
    l
}

/// (a) Recorrido: abrir-pr idempotente, estado-puertas (G01, checks por
/// autor, aprobacion), merge-pr idempotente, firma del merge, identidad,
/// permisos de la CI, lectura de reglas, y errores 401, 404 y sin commits.
#[test]
#[ignore = "E2E: escribe en el proyecto de pruebas de GitLab (TC2_E2E_GITLAB_PROYECTO, TC2_E2E_GITLAB_TOKEN)"]
fn recorrido_contra_gitlab() {
    let arnes = Arnes::nuevo();
    let (proyecto, token) = (arnes.e.proyecto.clone(), arnes.e.token.clone());
    let yo = arnes.usuario();
    let gl = adaptador(&proyecto, &token, Personas::nuevas([yo]));
    let limpieza = Limpieza::nueva(&arnes);
    let s = sufijo();
    let rama = format!("e2e/a-{s}");
    limpieza.rama(&rama);
    let sha = arnes.commit(
        &rama,
        Some("develop"),
        &format!("e2e/a-{s}.txt"),
        "prueba E2E de TASK-022\n",
    );

    eprintln!("abrir-pr:");
    let l = ver(&gl.abrir_pr(&rama, "develop", "E2E TASK-022", "MR de prueba"));
    assert!(l.starts_with("abrir-pr ok pr="), "{l}");
    let iid = l.trim_start_matches("abrir-pr ok pr=").to_owned();
    limpieza.mr(&iid);
    let n: u64 = iid.parse().unwrap();
    assert_eq!(
        ver(&gl.abrir_pr(&rama, "develop", "E2E TASK-022", "")),
        format!("abrir-pr sin-cambios pr={iid}")
    );

    eprintln!("estado-puertas:");
    let m = modelo(&[("verificacion", yo)]);
    arnes.desproteger_develop();
    assert_eq!(
        ver(&gl.estado_puertas(Rol::Agente, n, &m)),
        format!("estado-puertas rechazada pr={n} motivo=config garantia=G01")
    );
    limpieza.proteger();
    let l = esperar(
        || gl.estado_puertas(Rol::Agente, n, &m),
        |l| l.contains("falta=verificacion,aprobacion"),
    );
    assert_eq!(
        l,
        format!(
            "estado-puertas esperando pr={n} aprobaciones=0 ignoradas=0 falta=verificacion,aprobacion"
        )
    );
    arnes.estado(&sha, "verificacion", "success");
    assert_eq!(
        ver(&gl.estado_puertas(Rol::Agente, n, &m)),
        format!("estado-puertas esperando pr={n} aprobaciones=0 ignoradas=0 falta=aprobacion")
    );
    // Un estado con el nombre del revisor publicado por una cuenta que no es
    // su origen no cuenta (G09, `author.id`).
    arnes.estado(&sha, "revision-ia", "success");
    let con_revisor = modelo(&[("verificacion", yo), ("revision-ia", 999_999_999)]);
    assert!(
        ver(&gl.estado_puertas(Rol::Agente, n, &con_revisor))
            .contains("falta=revision-ia,aprobacion")
    );

    eprintln!("autoaprobacion (se observa):");
    let (e, v) = arnes.api(
        "POST",
        &format!("/merge_requests/{n}/approve"),
        Some(json!({})),
    );
    eprintln!(
        "  POST approve -> {e} {}",
        v.get("message").unwrap_or(&Value::Null)
    );
    let tras = ver(&gl.estado_puertas(Rol::Agente, n, &m));
    // GitLab Free deja aprobar al autor; la aprobacion no cuenta (RR-15).
    let esperado = if e == 201 {
        format!("estado-puertas esperando pr={n} aprobaciones=0 ignoradas=1 falta=aprobacion")
    } else {
        format!("estado-puertas esperando pr={n} aprobaciones=0 ignoradas=0 falta=aprobacion")
    };
    assert_eq!(tras, esperado);

    eprintln!("merge-pr:");
    assert_eq!(
        ver(&gl.merge_pr(n, &"0".repeat(40))),
        format!("merge-pr rechazada pr={n} motivo=head-cambiado")
    );
    let l = ver(&gl.merge_pr(n, &sha));
    assert!(l.starts_with(&format!("merge-pr ok pr={n}")), "{l}");
    assert_eq!(
        ver(&gl.merge_pr(n, &sha)),
        format!("merge-pr sin-cambios pr={n} estado=mergeado")
    );
    let (_, mr) = arnes.api("GET", &format!("/merge_requests/{n}"), None);
    let merge_sha = mr["merge_commit_sha"].as_str().unwrap_or(&sha).to_owned();

    eprintln!("firma del merge, identidad, permisos de la CI, reglas:");
    let f = gl.firma_commit(&merge_sha).unwrap();
    eprintln!(
        "  firma verificada={} motivo={} clave={:?}",
        f.verificada, f.motivo, f.clave
    );
    assert!(!f.cuenta_como_aprobacion(&Personas::nuevas([yo])));
    let u = gl.identidad(Rol::Agente).unwrap();
    eprintln!("  identidad login={} id={} tipo={}", u.login, u.id, u.tipo);
    assert_eq!((u.id, u.tipo.as_str()), (yo, "User"));
    assert_eq!(
        ver(&gl.leer_permisos_ci()),
        "leer-permisos-ci ok ci-aprueba=false fuente=plataforma"
    );
    let r = gl.leer_puertas(Rol::Agente, "develop").unwrap();
    eprintln!(
        "  leer-puertas plan_da_reglas={} reglas={:?} rama={:?}",
        r.plan_da_reglas, r.rulesets, r.reglas_rama
    );
    assert!(r.reglas_rama.contains(&"merge=40".to_owned()), "{r:?}");

    eprintln!("errores:");
    let vacia = format!("e2e/vacia-{s}");
    limpieza.rama(&vacia);
    arnes.rama_sin_commits(&vacia, "develop");
    assert_eq!(
        ver(&gl.abrir_pr(&vacia, "develop", "T", "")),
        "abrir-pr rechazada motivo=sin-commits"
    );
    let falso = adaptador(&proyecto, "glpat-token-falso", Personas::default());
    let i = falso.identidad(Rol::Agente).unwrap_err();
    assert_eq!(ver(&i), "identidad denegada motivo=credenciales rol=agente");
    assert!(!i.salida(false).stderr.contains("glpat-token-falso"));
    let otro = adaptador(
        &format!("{proyecto}-no-existe-{s}"),
        &token,
        Personas::default(),
    );
    let i = otro.abrir_pr("x", "develop", "T", "");
    assert_eq!(
        ver(&i),
        "abrir-pr denegada motivo=no-existe-o-sin-acceso rol=agente"
    );
    eprintln!("llamadas del adaptador: {}", gl.registro().len());
    assert_eq!(i.resultado, Resultado::Denegada);
}

/// (b) Puertas reales: rama por detras (`al-dia`), conflicto
/// (`sin-conflictos`) y merge rechazado por GitLab (`no-mergeable`).
#[test]
#[ignore = "E2E: escribe en el proyecto de pruebas de GitLab (TC2_E2E_GITLAB_PROYECTO, TC2_E2E_GITLAB_TOKEN)"]
fn puertas_reales_contra_gitlab() {
    let arnes = Arnes::nuevo();
    let (proyecto, token) = (arnes.e.proyecto.clone(), arnes.e.token.clone());
    let yo = arnes.usuario();
    let gl = adaptador(&proyecto, &token, Personas::nuevas([yo]));
    let limpieza = Limpieza::nueva(&arnes);
    let s = sufijo();
    let comun = format!("e2e/comun-{s}.txt");
    // Fichero comun en develop, y dos ramas desde ahi.
    arnes.commit("develop", None, &comun, "base\n");
    let conflicto = format!("e2e/conflicto-{s}");
    let detras = format!("e2e/detras-{s}");
    limpieza.rama(&conflicto);
    limpieza.rama(&detras);
    let sha_c = arnes.commit(&conflicto, Some("develop"), &comun, "desde la rama\n");
    let sha_d = arnes.commit(
        &detras,
        Some("develop"),
        &format!("e2e/detras-{s}.txt"),
        "x\n",
    );
    let mut iids = Vec::new();
    for rama in [&conflicto, &detras] {
        let l = ver(&gl.abrir_pr(rama, "develop", "E2E TASK-022 puertas", ""));
        let iid = l.trim_start_matches("abrir-pr ok pr=").to_owned();
        limpieza.mr(&iid);
        iids.push(iid.parse::<u64>().unwrap());
    }
    // develop avanza y pisa el fichero comun.
    arnes.commit("develop", None, &comun, "desde develop\n");
    limpieza.proteger();
    arnes.estado(&sha_c, "verificacion", "success");
    arnes.estado(&sha_d, "verificacion", "success");
    let m = modelo(&[("verificacion", yo)]);

    eprintln!("conflicto:");
    let n = iids[0];
    let l = esperar(
        || gl.estado_puertas(Rol::Agente, n, &m),
        |l| l.contains("sin-conflictos"),
    );
    assert!(l.contains("al-dia") && l.contains("sin-conflictos"), "{l}");
    let i = gl.merge_pr(n, &sha_c);
    let l = ver(&i);
    assert!(
        l == format!("merge-pr rechazada pr={n} motivo=no-mergeable")
            || l.starts_with(&format!("merge-pr error pr={n}")),
        "{l}"
    );
    eprintln!("  diagnostico: {:?}", i.diagnostico);

    eprintln!("detras:");
    let n = iids[1];
    let l = esperar(
        || gl.estado_puertas(Rol::Agente, n, &m),
        |l| l.contains("al-dia"),
    );
    assert_eq!(
        l,
        format!(
            "estado-puertas esperando pr={n} aprobaciones=0 ignoradas=0 falta=aprobacion,al-dia"
        )
    );
}

/// (c) F6-1: el agente, con su cuenta de servicio, empuja una rama por git
/// con su token (fuera la configuracion de git del usuario) y abre el MR;
/// si GitLab le deja aprobarlo, esa aprobacion no cuenta; la persona lo
/// aprueba y `estado-puertas` con la persona cuenta la suya.
#[test]
#[ignore = "E2E: escribe en el proyecto de pruebas de GitLab (TC2_E2E_GITLAB_PROYECTO, TC2_E2E_GITLAB_TOKEN, TC2_E2E_GITLAB_TOKEN_AGENTE)"]
fn persona_aprueba_mr_del_agente() {
    let arnes = Arnes::nuevo();
    let (proyecto, token) = (arnes.e.proyecto.clone(), arnes.e.token.clone());
    let token_agente =
        std::env::var("TC2_E2E_GITLAB_TOKEN_AGENTE").expect("define TC2_E2E_GITLAB_TOKEN_AGENTE");
    let yo = arnes.usuario();
    let vars = |v: &str| match v {
        "TC2_TOKEN_AGENTE" => Some(token_agente.clone()),
        "TC2_TOKEN_PERSONA" => Some(token.clone()),
        _ => None,
    };
    let gl = GitLab::nuevo(
        Base::nueva(API).unwrap(),
        &proyecto,
        Box::new(de_entorno_con(vars, true)),
        Personas::nuevas([yo]),
    )
    .unwrap();

    // La identidad del agente es una cuenta `bot: true`, distinta de la
    // persona.
    let agente = gl.identidad(Rol::Agente).unwrap();
    eprintln!(
        "  agente: login={} id={} tipo={}",
        agente.login, agente.id, agente.tipo
    );
    assert_ne!(agente.id, yo, "el agente tiene que ser otra cuenta");
    assert_eq!(agente.tipo, "Bot", "la cuenta del agente no es bot");
    let (e, u) = arnes
        .intentar_con(&token_agente, "GET", "/user", None)
        .expect("red");
    assert_eq!(e, 200, "{u}");
    let correo = ["commit_email", "email"]
        .iter()
        .find_map(|k| u[*k].as_str().filter(|c| !c.is_empty()))
        .map_or_else(
            || format!("{}@noreply.gitlab.com", agente.login),
            str::to_owned,
        );
    let autor = AutorGit {
        nombre: agente.login.clone(),
        correo,
    };

    let limpieza = Limpieza::nueva(&arnes);
    let rama = format!("e2e/agente-{}", sufijo());
    limpieza.rama(&rama);
    let (_clon, sha) = agente_git::publicar_rama(
        Proveedor::GitLab,
        &format!("https://gitlab.com/{proyecto}.git"),
        "https://gitlab.com/",
        &Secreto::nuevo(token_agente.clone()),
        &autor,
        ("main", &rama),
    );
    let (e, c) = arnes.api("GET", &format!("/repository/commits/{sha}"), None);
    assert_eq!(e, 200, "{c}");
    eprintln!("  commit {sha}: autor={}", c["author_name"]);
    assert_eq!(c["author_name"], agente.login.as_str());

    let i = gl.abrir_pr(&rama, "main", "E2E TASK-047", "MR del agente (F6-1)");
    assert_eq!(i.resultado, Resultado::Ok, "{}", ver(&i));
    ver(&i);
    let n = i.valor("pr").unwrap().to_owned();
    limpieza.mr(&n);
    let num: u64 = n.parse().unwrap();
    let (e, mr) = arnes.api("GET", &format!("/merge_requests/{n}"), None);
    assert_eq!(e, 200, "{mr}");
    assert_eq!(mr["author"]["id"], agente.id, "el MR lo abre el agente");

    // El agente intenta aprobar su propio MR: si GitLab lo deja (Free deja
    // aprobar al autor), no cuenta.
    let m = modelo(&[]);
    let (e, v) = arnes
        .intentar_con(
            &token_agente,
            "POST",
            &format!(
                "/projects/{}/merge_requests/{n}/approve",
                proyecto.replace('/', "%2F")
            ),
            None,
        )
        .expect("red");
    eprintln!(
        "  auto-aprobacion del agente: {e} {}",
        v.get("message").unwrap_or(&Value::Null)
    );
    let i = esperar(
        || gl.estado_puertas(Rol::Persona, num, &m),
        |l| l.contains("aprobaciones="),
    );
    assert!(i.contains("aprobaciones=0"), "{i}");
    if e == 201 {
        assert!(i.contains("ignoradas=1"), "{i}");
    }

    // La persona aprueba.
    let (e, v) = arnes.api(
        "POST",
        &format!("/merge_requests/{n}/approve"),
        Some(json!({})),
    );
    assert_eq!(e, 201, "{v}");
    let i = gl.estado_puertas(Rol::Persona, num, &m);
    assert_eq!(i.valor("aprobaciones"), Some("1"), "{}", ver(&i));
    ver(&i);
    let i = gl.estado_puertas(Rol::Agente, num, &m);
    assert_eq!(i.valor("aprobaciones"), Some("1"), "{}", ver(&i));

    drop(limpieza);
}
