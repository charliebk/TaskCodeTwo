//! F2-13 (TASK-077): `tc2 identidad` y `quien_ejecuta` contra GitHub y
//! GitLab simulados en 127.0.0.1 con `comun::http_mock`, con repos reales
//! en una carpeta temporal.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::net::TcpListener;
use std::path::{Path, PathBuf};

use comun::http_mock::{Mock, Simulada, serie};
use comun::{Fixture, escribir};
use tc2::contrato::{Informe, Resultado};
use tc2::git::Git;
use tc2::identidad::{self, Cache, Cuenta, Entorno, Quien, TTL_SEGUNDOS, quien_ejecuta};
use tc2::plataforma::http::Base;
use tc2::plataforma::{
    CredencialesFijas, GitHub, GitLab, Personas, Plataforma, Proveedor, Rol, Secreto,
};

const AHORA: u64 = 1_800_000_000;
const PERSONAS: &str = "---\npersonas:\n  - {id: 1001, login: \"carlos\", nombre: \"Carlos\"}\n  - {id: 1002, login: \"web-flow\"}\n  - {id: 1003, login: \"ci-bot\"}\n";

/// API simulada con `GET /user` fijo; lo demas da 404.
fn api_user(cuerpo: &str) -> Mock {
    let m = Mock::nuevo();
    m.responder("GET", "/user", Simulada::json(200, cuerpo));
    m
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

    /// `origin` en un host que no es github.com ni gitlab.com, por ssh: con
    /// [`git_con_ssh`] el fetch llega al remoto bare de la fixture.
    fn origin(self) -> &'static str {
        match self {
            Self::GitHub => "ssh://git@git.ejemplo.test/o/r.git",
            Self::GitLab => "ssh://git@git.ejemplo.test/g/sub/p.git",
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
///
/// `ls-remote --get-url` aplica `url.<base>.insteadOf`, asi que el remoto
/// no se puede redirigir por configuracion sin cambiar el host que se
/// detecta: el transporte ssh se sustituye con `GIT_SSH_COMMAND`.
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

fn entorno<'a>(dir: &'a Path, git: &'a Git, tty: bool) -> Entorno<'a> {
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

/// Puerto Git de la fixture cuyo «ssh» es `ssh_command` (variante simple:
/// solo el host y la orden remota, que el guion no mira).
fn git_ssh(f: &Fixture, ssh_command: String) -> Git {
    let mut e = f.entorno();
    e.push(("GIT_SSH_COMMAND".into(), ssh_command.into()));
    e.push(("GIT_SSH_VARIANT".into(), "simple".into()));
    Git::nuevo().con_entorno(e)
}

/// El «ssh» sirve el remoto bare de la fixture (`<raiz>/remoto.git`).
fn git_con_ssh(f: &Fixture) -> Git {
    let bare = f
        .raiz
        .join("remoto.git")
        .to_string_lossy()
        .replace('\\', "/");
    git_ssh(f, format!("sh -c 'exec git upload-pack \"$0\"' '{bare}'"))
}

/// El «ssh» no llega al host, como sin red.
fn git_sin_red(f: &Fixture) -> Git {
    git_ssh(
        f,
        "sh -c 'echo \"ssh: connect to host git.ejemplo.test port 22: Connection refused\" >&2; exit 255'"
            .to_owned(),
    )
}

fn lanzar_con(g: &Git, clon: &Path, tty: bool) -> Informe {
    let i = identidad::orden(&[], entorno(clon, g, tty));
    comun::contrato(&i);
    i
}

fn lanzar(f: &Fixture, clon: &Path, tty: bool) -> Informe {
    lanzar_con(&git_con_ssh(f), clon, tty)
}

/// C1: con token de persona y terminal, `tc2 identidad` imprime `identidad
/// ok rol=persona id=<N> login=<L>` y guarda la cache local. En GitHub y en
/// GitLab (cuyo `login` es el `username`), con una sola peticion. C7: con la
/// cache vigente, `quien_ejecuta` sin terminal no pide nada a la red.
#[test]
fn identidad_con_terminal_guarda_la_cache_en_github_y_gitlab() {
    let _serie = serie();
    for prov in [Prov::GitHub, Prov::GitLab] {
        let s = api_user(&prov.usuario("carlos", 1001, false));
        let f = Fixture::nueva("identidad-c1");
        let clon = repo(&f, prov, Some(&s.base()), Some(PERSONAS));
        let i = lanzar(&f, &clon, true);
        assert_eq!(
            i.linea(),
            "identidad ok rol=persona id=1001 login=carlos",
            "{prov:?}"
        );
        assert_eq!(
            s.recibidas()
                .into_iter()
                .map(|r| r.linea)
                .collect::<Vec<_>>(),
            ["GET /user"],
            "{prov:?}"
        );
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
        assert_eq!(s.recibidas().len(), 1, "{prov:?}: con cache no hay red");
        // JSON: las mismas claves.
        let g2 = git_con_ssh(&f);
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
    let _serie = serie();
    for prov in [Prov::GitHub, Prov::GitLab] {
        for (login, id, bot, cuenta) in [
            ("otra", 4242, false, "fuera-de-lista"),
            ("ci-bot", 1003, true, "bot"),
            ("web-flow", 1002, false, "web-flow"),
        ] {
            let s = api_user(&prov.usuario(login, id, bot));
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
            // Con terminal la base ya se acaba de traer: el arreglo es el PR,
            // no volver a lanzar `tc2 identidad` (revision 2, IMP-2).
            if cuenta == "fuera-de-lista" {
                for idioma in [tc2::i18n::Idioma::Es, tc2::i18n::Idioma::En] {
                    let d = i.diagnostico_en(idioma).unwrap();
                    assert!(
                        d.contains(".taskcode/personas.yml")
                            && d.contains("origin/develop")
                            && d.contains("PR")
                            && !d.contains("tc2 identidad"),
                        "{d}"
                    );
                }
            }
        }
    }
}

/// Revision 2, MEN-2/MEN-4: si `origin` no tiene la rama base, `tc2
/// identidad` no dice `sinred` (reintentar no lo arregla): `rechazada
/// motivo=sin-rama-base rama=develop`, sin hablar con la API ni tocar la
/// cache.
#[test]
fn origin_sin_rama_base_no_es_sin_red() {
    let _serie = serie();
    let s = api_user(&Prov::GitHub.usuario("carlos", 1001, false));
    let f = Fixture::nueva("identidad-sin-base");
    let clon = repo(&f, Prov::GitHub, Some(&s.base()), Some(PERSONAS));
    let bare = f.raiz.join("remoto.git");
    f.crudo(&bare, &["update-ref", "-d", "refs/heads/develop"]);
    let i = lanzar(&f, &clon, true);
    assert_eq!(
        (i.linea().as_str(), i.resultado),
        (
            "identidad rechazada motivo=sin-rama-base rama=develop",
            Resultado::Rechazada
        )
    );
    let es = i.diagnostico_en(tc2::i18n::Idioma::Es).unwrap();
    let en = i.diagnostico_en(tc2::i18n::Idioma::En).unwrap();
    assert!(es.contains("develop") && es.contains("no tiene"), "{es}");
    assert!(en.contains("develop") && en.contains("has no"), "{en}");
    assert!(s.recibidas().is_empty());
    assert!(!cache_de(&clon).ruta().exists());
    // El `fetch` de siempre no cambia: el remoto sin `develop` se trae sin
    // error (solo `traer_rama` pide la rama).
    let g = git_con_ssh(&f);
    assert!(tc2::git::fetch(&g, &clon).resultado.es_exito());
}

/// C7: sin red, `tc2 identidad` falla (`sinred`, codigo 4) sin tocar la
/// cache; sin cache, o caducada, y sin terminal, `quien_ejecuta` rechaza
/// pidiendo `tc2 identidad`, tambien sin red.
#[test]
fn sin_red_no_toca_la_cache_y_sin_cache_se_pide_tc2_identidad() {
    let _serie = serie();
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
    // Sin red ya en el fetch de la rama base (antes de hablar con la API):
    // `sinred` y la cache sin tocar.
    let s = api_user(&Prov::GitHub.usuario("carlos", 1001, false));
    let f2 = Fixture::nueva("identidad-c7b");
    let clon2 = repo(&f2, Prov::GitHub, Some(&s.base()), Some(PERSONAS));
    let cache2 = cache_de(&clon2);
    let previa = Cuenta {
        proveedor: Proveedor::GitHub,
        id: 1001,
        login: "carlos".to_owned(),
    };
    cache2.guardar(&previa, AHORA - 10).unwrap();
    let antes = std::fs::read(cache2.ruta()).unwrap();
    let i = lanzar_con(&git_sin_red(&f2), &clon2, true);
    assert_eq!(
        (i.linea().as_str(), i.resultado),
        ("identidad sinred remoto=origin", Resultado::Sinred)
    );
    assert_eq!(std::fs::read(cache2.ruta()).unwrap(), antes);
    assert!(s.recibidas().is_empty(), "sin fetch no se habla con la API");
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
    let _serie = serie();
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
    let s = api_user(&Prov::GitHub.usuario("carlos", 1001, false));
    let f = Fixture::nueva("identidad-c8b");
    let clon = repo(&f, Prov::GitHub, Some(&s.base()), None);
    let i = lanzar(&f, &clon, true);
    assert_eq!(i.linea(), "identidad rechazada motivo=sin-personas");
    assert!(s.recibidas().is_empty());
    // Argumentos de mas: uso.
    let s = tc2::ejecutar(["identidad", "de-mas"]);
    assert_eq!(s.codigo, 2);
    assert_eq!(s.stdout, "identidad uso motivo=argumentos-de-mas\n");
}

/// Revision 1, IMP-1: quien acaba de entrar en `personas.yml` por un PR
/// fusionado se identifica desde otro clon sin `fetch` previo: con terminal,
/// `tc2 identidad` trae antes la rama base. Sin terminal no se trae nada y
/// el rechazo dice que `tc2 identidad` actualiza la lista.
#[test]
fn persona_recien_anadida_se_identifica_desde_otro_clon_sin_fetch_previo() {
    let _serie = serie();
    let s = api_user(&Prov::GitHub.usuario("nueva", 1004, false));
    let f = Fixture::nueva("identidad-imp1");
    let a = repo(&f, Prov::GitHub, Some(&s.base()), Some(PERSONAS));
    let bare = f.raiz.join("remoto.git");
    // El otro clon, con la lista de antes (sin `nueva`).
    let b = f.otro_clon(&bare, "b");
    f.crudo(&b, &["remote", "set-url", "origin", Prov::GitHub.origin()]);
    // En el primero se anade la persona y se publica.
    escribir(
        &a,
        ".taskcode/personas.yml",
        &format!("{PERSONAS}  - {{id: 1004, login: \"nueva\"}}\n"),
    );
    f.crudo(&a, &["add", ".taskcode/personas.yml"]);
    f.crudo(&a, &["commit", "-q", "-m", "anade a nueva"]);
    f.crudo(&a, &["push", "-q", bare.to_str().unwrap(), "develop"]);
    // Sin terminal (el agente): sin red ni fetch, la lista de su copia no la
    // tiene y el diagnostico manda a `tc2 identidad`.
    let personas_b = identidad::personas_de_base(&git_con_ssh(&f), &b).unwrap();
    assert!(!personas_b.contiene(1004));
    let cache = cache_de(&b);
    cache
        .guardar(
            &Cuenta {
                proveedor: Proveedor::GitHub,
                id: 1004,
                login: "nueva".to_owned(),
            },
            AHORA,
        )
        .unwrap();
    let p = Prov::GitHub.adaptador(&s.base());
    let r = quien_ejecuta(
        p.as_ref(),
        Proveedor::GitHub,
        &personas_b,
        &cache,
        false,
        AHORA,
    )
    .unwrap_err();
    assert!(r.linea().ends_with("cache=fuera-de-lista"));
    for idioma in [tc2::i18n::Idioma::Es, tc2::i18n::Idioma::En] {
        let d = r.diagnostico_en(idioma).unwrap();
        assert!(d.contains("tc2 identidad"), "{d}");
    }
    assert!(s.recibidas().is_empty());
    // Con terminal: trae la base y sale `ok`.
    std::fs::remove_file(cache.ruta()).unwrap();
    let i = lanzar(&f, &b, true);
    assert_eq!(i.linea(), "identidad ok rol=persona id=1004 login=nueva");
    assert_eq!(
        s.recibidas()
            .into_iter()
            .map(|r| r.linea)
            .collect::<Vec<_>>(),
        ["GET /user"]
    );
    assert_eq!(cache.leer().map(|g| g.cuenta.id), Some(1004));
}
