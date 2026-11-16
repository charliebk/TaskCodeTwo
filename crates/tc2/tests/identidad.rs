//! F2-13 (TASK-077): `tc2 identidad` y `quien_ejecuta` contra GitHub y
//! GitLab simulados en 127.0.0.1, con repos reales en una carpeta temporal.
//!
//! El servidor simulado es propio y no cierra la conexion tras una
//! respuesta completa: con `Content-Length` el cliente sabe cuando acaba y
//! cierra el (en esta maquina un FIN del servidor detras de los datos hacia
//! que la conexion se colgara a veces ~19 s, TASK-027).

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use comun::{Fixture, escribir};
use tc2::contrato::{Informe, Resultado};
use tc2::identidad::{self, Cache, Cuenta, Entorno, Quien, TTL_SEGUNDOS, quien_ejecuta};
use tc2::plataforma::http::Base;
use tc2::plataforma::{
    CredencialesFijas, GitHub, GitLab, Personas, Plataforma, Proveedor, Rol, Secreto,
};

const AHORA: u64 = 1_800_000_000;
const PERSONAS: &str = "---\npersonas:\n  - {id: 1001, login: \"carlos\", nombre: \"Carlos\"}\n  - {id: 1002, login: \"web-flow\"}\n  - {id: 1003, login: \"ci-bot\"}\n";

/// Servidor HTTP simulado: `GET <ruta>` da la respuesta fija o 404, y
/// registra cada peticion.
struct Servidor {
    puerto: u16,
    peticiones: Arc<Mutex<Vec<String>>>,
}

impl Servidor {
    fn nuevo(rutas: &[(&str, &str)]) -> Self {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let puerto = l.local_addr().unwrap().port();
        let peticiones: Arc<Mutex<Vec<String>>> = Arc::default();
        let rutas: Arc<BTreeMap<String, String>> = Arc::new(
            rutas
                .iter()
                .map(|(r, c)| ((*r).to_owned(), (*c).to_owned()))
                .collect(),
        );
        let p = peticiones.clone();
        std::thread::spawn(move || {
            for s in l.incoming().flatten() {
                let (p, rutas) = (p.clone(), rutas.clone());
                std::thread::spawn(move || atender(s, &p, &rutas));
            }
        });
        Self { puerto, peticiones }
    }

    fn base(&self) -> String {
        format!("http://127.0.0.1:{}", self.puerto)
    }

    fn peticiones(&self) -> Vec<String> {
        self.peticiones.lock().unwrap().clone()
    }
}

fn atender(s: std::net::TcpStream, p: &Mutex<Vec<String>>, rutas: &BTreeMap<String, String>) {
    let mut lector = BufReader::new(s.try_clone().unwrap());
    let mut primera = String::new();
    if lector.read_line(&mut primera).is_err() {
        return;
    }
    loop {
        let mut l = String::new();
        if lector.read_line(&mut l).is_err() || l.trim().is_empty() {
            break;
        }
    }
    let mut partes = primera.split_whitespace();
    let metodo = partes.next().unwrap_or_default();
    let ruta = partes.next().unwrap_or_default().to_owned();
    p.lock().unwrap().push(format!("{metodo} {ruta}"));
    let (estado, cuerpo) = match rutas.get(&ruta) {
        Some(c) => ("200 OK", c.clone()),
        None => ("404 Not Found", "{\"message\":\"Not Found\"}".to_owned()),
    };
    let mut s = s;
    let _ = write!(
        s,
        "HTTP/1.1 {estado}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{cuerpo}",
        cuerpo.len()
    );
    let _ = s.flush();
    // Sin FIN del servidor: cierra el cliente al leer el `Content-Length`.
    let _ = s.set_read_timeout(Some(std::time::Duration::from_secs(30)));
    let _ = std::io::Read::read_to_end(&mut lector, &mut Vec::new());
}

/// Plataforma simulada: GitHub o GitLab, con la forma de `GET /user` de
/// cada una.
#[derive(Clone, Copy, Debug)]
enum Prov {
    GitHub,
    GitLab,
}

impl Prov {
    fn nombre(self) -> &'static str {
        match self {
            Self::GitHub => "github",
            Self::GitLab => "gitlab",
        }
    }

    fn proveedor(self) -> Proveedor {
        match self {
            Self::GitHub => Proveedor::GitHub,
            Self::GitLab => Proveedor::GitLab,
        }
    }

    /// Cuerpo de `GET /user` para una cuenta; `bot` la marca como bot.
    fn usuario(self, login: &str, id: u64, bot: bool) -> String {
        match self {
            Self::GitHub => format!(
                "{{\"login\":\"{login}\",\"id\":{id},\"type\":\"{}\"}}",
                if bot { "Bot" } else { "User" }
            ),
            Self::GitLab => {
                format!("{{\"username\":\"{login}\",\"id\":{id},\"bot\":{bot}}}")
            }
        }
    }

    fn origin(self) -> &'static str {
        match self {
            Self::GitHub => "https://git.ejemplo.test/o/r.git",
            Self::GitLab => "https://git.ejemplo.test/g/sub/p.git",
        }
    }

    fn adaptador(self, api: &str) -> Box<dyn Plataforma> {
        let c = Box::new(CredencialesFijas::nuevas().con(Rol::Persona, Secreto::nuevo("tok")));
        let personas = Personas::leer(PERSONAS).unwrap();
        let base = Base::nueva(api).unwrap();
        match self {
            Self::GitHub => Box::new(GitHub::nuevo(base, "o/r", c, personas).unwrap()),
            Self::GitLab => Box::new(GitLab::nuevo(base, "g/sub/p", c, personas).unwrap()),
        }
    }
}

/// Clon con `origin/develop` que lleva `.taskcode/config.yml` (con la clave
/// `plataforma` si se da `api`) y `.taskcode/personas.yml` (si se da), y
/// `origin` apuntando a un host que no es github.com ni gitlab.com.
fn repo(f: &Fixture, prov: Prov, api: Option<&str>, personas: Option<&str>) -> PathBuf {
    let (_, clon) = f.remoto_y_clon();
    let config = match api {
        Some(api) => format!(
            "---\nformato: 2\nplataforma:\n  proveedor: \"{}\"\n  api: \"{api}\"\n",
            prov.nombre()
        ),
        None => "---\nformato: 2\n".to_owned(),
    };
    escribir(&clon, ".taskcode/config.yml", &config);
    let mut rutas = vec![".taskcode/config.yml"];
    if let Some(p) = personas {
        escribir(&clon, ".taskcode/personas.yml", p);
        rutas.push(".taskcode/personas.yml");
    }
    let mut add = vec!["add"];
    add.extend(rutas);
    f.crudo(&clon, &add);
    f.crudo(&clon, &["commit", "-q", "-m", "taskcode"]);
    f.crudo(&clon, &["push", "-q", "origin", "develop"]);
    f.crudo(&clon, &["fetch", "-q", "origin"]);
    f.crudo(&clon, &["remote", "set-url", "origin", prov.origin()]);
    clon
}

fn entorno<'a>(dir: &'a Path, git: &'a tc2::git::Git, tty: bool) -> Entorno<'a> {
    Entorno {
        dir,
        git,
        tty,
        credenciales: Box::new(
            CredencialesFijas::nuevas().con(Rol::Persona, Secreto::nuevo("tok-persona")),
        ),
        ahora: AHORA,
    }
}

fn cache_de(clon: &Path) -> Cache {
    Cache::en(&clon.join(".git"))
}

fn lanzar(f: &Fixture, clon: &Path, tty: bool) -> Informe {
    let g = f.git();
    let i = identidad::orden(&[], entorno(clon, &g, tty));
    comun::contrato(&i);
    i
}

/// C1: con token de persona y terminal, `tc2 identidad` imprime `identidad
/// ok rol=persona id=<N> login=<L>` y guarda la cache local. En GitHub y en
/// GitLab (cuyo `login` es el `username`), con una sola peticion. C7: con la
/// cache vigente, `quien_ejecuta` sin terminal no pide nada a la red.
#[test]
fn identidad_con_terminal_guarda_la_cache_en_github_y_gitlab() {
    for prov in [Prov::GitHub, Prov::GitLab] {
        let s = Servidor::nuevo(&[("/user", &prov.usuario("carlos", 1001, false))]);
        let f = Fixture::nueva("identidad-c1");
        let clon = repo(&f, prov, Some(&s.base()), Some(PERSONAS));
        let i = lanzar(&f, &clon, true);
        assert_eq!(
            i.linea(),
            "identidad ok rol=persona id=1001 login=carlos",
            "{prov:?}"
        );
        assert_eq!(s.peticiones(), ["GET /user"], "{prov:?}");
        let g = cache_de(&clon).leer().expect("cache guardada");
        assert_eq!(
            (g.cuenta.proveedor, g.cuenta.id, g.cuenta.login.as_str()),
            (prov.proveedor(), 1001, "carlos")
        );
        assert_eq!(g.verificado, AHORA);
        // Sin terminal: el agente en nombre de carlos, 0 peticiones.
        let p = prov.adaptador(&s.base());
        let personas = Personas::leer(PERSONAS).unwrap();
        let q = quien_ejecuta(
            p.as_ref(),
            prov.proveedor(),
            &personas,
            &cache_de(&clon),
            false,
            AHORA + 3600,
        )
        .unwrap();
        assert_eq!(q.decidido_por(), "agente");
        assert!(matches!(&q, Quien::Agente(c) if c.id == 1001 && c.login == "carlos"));
        assert_eq!(s.peticiones().len(), 1, "{prov:?}: con cache no hay red");
        // JSON: las mismas claves.
        let g2 = f.git();
        let j = identidad::orden(&[], entorno(&clon, &g2, true)).json();
        assert!(
            j.contains("\"resultado\":\"ok\"") && j.contains("\"login\":\"carlos\""),
            "{j}"
        );
    }
}

/// C2: una cuenta fuera de `personas.yml`, `Bot` o `web-flow` da `rechazada
/// motivo=identidad` y la cache no se toca.
#[test]
fn cuenta_fuera_de_lista_bot_o_web_flow_no_toca_la_cache() {
    for prov in [Prov::GitHub, Prov::GitLab] {
        for (login, id, bot, cuenta) in [
            ("otra", 4242, false, "fuera-de-lista"),
            ("ci-bot", 1003, true, "bot"),
            ("web-flow", 1002, false, "web-flow"),
        ] {
            let s = Servidor::nuevo(&[("/user", &prov.usuario(login, id, bot))]);
            let f = Fixture::nueva("identidad-c2");
            let clon = repo(&f, prov, Some(&s.base()), Some(PERSONAS));
            let cache = cache_de(&clon);
            let previa = Cuenta {
                proveedor: prov.proveedor(),
                id: 1001,
                login: "carlos".to_owned(),
            };
            cache.guardar(&previa, AHORA - 10).unwrap();
            let antes = std::fs::read(cache.ruta()).unwrap();
            let i = lanzar(&f, &clon, true);
            assert_eq!(
                i.linea(),
                format!("identidad rechazada motivo=identidad rol=persona cuenta={cuenta}"),
                "{prov:?} {login}"
            );
            assert_eq!(i.resultado.codigo(), 3);
            assert_eq!(std::fs::read(cache.ruta()).unwrap(), antes, "{login}");
        }
    }
}

/// C7: sin red, `tc2 identidad` falla (`sinred`, codigo 4) sin tocar la
/// cache; sin cache, o caducada, y sin terminal, `quien_ejecuta` rechaza
/// pidiendo `tc2 identidad`, tambien sin red.
#[test]
fn sin_red_no_toca_la_cache_y_sin_cache_se_pide_tc2_identidad() {
    let cerrado = {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };
    let api = format!("http://127.0.0.1:{cerrado}");
    let f = Fixture::nueva("identidad-c7");
    let clon = repo(&f, Prov::GitHub, Some(&api), Some(PERSONAS));
    let cache = cache_de(&clon);
    let i = lanzar(&f, &clon, true);
    assert_eq!(
        (i.linea().as_str(), i.resultado),
        ("identidad sinred", Resultado::Sinred)
    );
    assert!(!cache.ruta().exists(), "sin red no se escribe la cache");
    let p = Prov::GitHub.adaptador(&api);
    let personas = Personas::leer(PERSONAS).unwrap();
    let gh = Proveedor::GitHub;
    let r = quien_ejecuta(p.as_ref(), gh, &personas, &cache, false, AHORA).unwrap_err();
    assert_eq!(
        r.linea(),
        "identidad rechazada motivo=identidad cache=falta"
    );
    assert!(
        r.diagnostico_en(tc2::i18n::Idioma::Es)
            .unwrap()
            .contains("tc2 identidad")
    );
    assert!(
        r.diagnostico_en(tc2::i18n::Idioma::En)
            .unwrap()
            .contains("tc2 identidad")
    );
    let c = Cuenta {
        proveedor: gh,
        id: 1001,
        login: "carlos".to_owned(),
    };
    cache.guardar(&c, AHORA).unwrap();
    let r = quien_ejecuta(
        p.as_ref(),
        gh,
        &personas,
        &cache,
        false,
        AHORA + TTL_SEGUNDOS,
    )
    .unwrap_err();
    assert_eq!(
        r.linea(),
        "identidad rechazada motivo=identidad cache=caducada"
    );
    // Vigente, sin terminal y sin red: vale (no habla con la plataforma).
    let q = quien_ejecuta(p.as_ref(), gh, &personas, &cache, false, AHORA + 1).unwrap();
    assert_eq!(q.decidido_por(), "agente");
}

/// C8: sin plataforma reconocida no hay identidad (ni vuelta a texto libre):
/// host desconocido sin la clave `plataforma`, o sin `origin`. Tampoco sin
/// terminal ni sin la lista de personas.
#[test]
fn sin_plataforma_reconocida_se_rechaza() {
    let f = Fixture::nueva("identidad-c8");
    let clon = repo(&f, Prov::GitHub, None, Some(PERSONAS));
    let i = lanzar(&f, &clon, true);
    assert_eq!(i.linea(), "identidad rechazada motivo=plataforma");
    f.crudo(&clon, &["remote", "remove", "origin"]);
    let i = lanzar(&f, &clon, true);
    assert_eq!(i.linea(), "identidad rechazada motivo=plataforma");
    assert!(!cache_de(&clon).ruta().exists());
    // Sin terminal no se verifica nada.
    let i = lanzar(&f, &clon, false);
    assert_eq!(
        i.linea(),
        "identidad rechazada motivo=identidad rol=persona"
    );
    // Sin `personas.yml` en la base.
    let s = Servidor::nuevo(&[("/user", &Prov::GitHub.usuario("carlos", 1001, false))]);
    let f = Fixture::nueva("identidad-c8b");
    let clon = repo(&f, Prov::GitHub, Some(&s.base()), None);
    let i = lanzar(&f, &clon, true);
    assert_eq!(i.linea(), "identidad rechazada motivo=sin-personas");
    assert!(s.peticiones().is_empty());
    // Argumentos de mas: uso.
    let s = tc2::ejecutar(["identidad", "de-mas"]);
    assert_eq!(s.codigo, 2);
    assert_eq!(s.stdout, "identidad uso motivo=argumentos-de-mas\n");
}
