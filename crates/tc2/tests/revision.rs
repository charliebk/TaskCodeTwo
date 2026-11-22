//! F6-2 (TASK-048): `tc2 review` con repos git reales (remoto bare y el
//! worktree de la tarea que deja `start`) y una plataforma doble que apunta
//! lo que le piden.
//!
//! - Criterio «PR enlazado»: el push llega al remoto, `abrir-pr` recibe la
//!   rama, `develop`, el titulo y el cuerpo con las rutas de la tarea, la
//!   tarea queda `en-revision` en la rama de estado y repetir sale
//!   `sin-cambios`.
//! - Requisito de F6-2 (identidad del agente): el push de `review` lleva la
//!   cabecera `Authorization: Basic` del entorno que da la plataforma
//!   (`entorno-push`); el remoto de push es el servidor simulado de
//!   `comun::http_mock` (`remote.origin.pushurl`), el de fetch el bare.
//!
//! F6-5 (TASK-051), criterio 2: con el check `revision-ia` terminado,
//! `review` guarda su informe en `TASK-NNN/revision/` de la rama de estado
//! (repetir no escribe nada); con el check pendiente, nada.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::cell::RefCell;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use comun::http_mock::{Mock, serie};
use comun::{Fixture, escribir};
use tc2::contrato::{Informe, Resultado};
use tc2::estado::{self, Operacion};
use tc2::git::Git;
use tc2::identidad::{Cuenta, Quien};
use tc2::plataforma::{
    Firma, ModeloPuertas, Paso, Plataforma, PrAbierto, Proveedor, ReglasLeidas, RevisionPublicada,
    Rol, Secreto, Usuario, entorno_git,
};
use tc2::{revision, toma};

type Entorno = Vec<(OsString, OsString)>;

/// Plataforma doble: da `extra` como entorno de push y abre un PR (el 12)
/// la primera vez; apunta las URL y los PR pedidos.
#[derive(Default)]
struct Doble {
    extra: Entorno,
    urls: RefCell<Vec<String>>,
    prs: RefCell<Vec<(String, String, String, String)>>,
    /// Lo que da `estado-puertas` (sin nada, un error).
    puertas: RefCell<Option<Informe>>,
    /// Lo que da `leer-revision` y cuantas veces se pidio.
    revision: RefCell<Option<RevisionPublicada>>,
    lecturas: RefCell<Vec<(u64, u64)>>,
}

struct Compartida(Rc<Doble>);

fn sin_uso(op: &str) -> Informe {
    Informe::nuevo(op, Resultado::Error)
}

impl Plataforma for Compartida {
    fn prs_abiertos(&self, _: Rol) -> Paso<Vec<PrAbierto>> {
        Err(sin_uso("prs-abiertos"))
    }
    fn entorno_push(&self, url: &str) -> Paso<Entorno> {
        self.0.urls.borrow_mut().push(url.to_owned());
        Ok(self.0.extra.clone())
    }
    fn abrir_pr(&self, rama: &str, base: &str, titulo: &str, cuerpo: &str) -> Informe {
        let mut prs = self.0.prs.borrow_mut();
        let r = if prs.is_empty() {
            Resultado::Ok
        } else {
            Resultado::SinCambios
        };
        prs.push((rama.into(), base.into(), titulo.into(), cuerpo.into()));
        Informe::nuevo("abrir-pr", r).con("pr", "12")
    }
    fn estado_puertas(&self, _: Rol, _: u64, _: &ModeloPuertas) -> Informe {
        self.0
            .puertas
            .borrow()
            .clone()
            .unwrap_or_else(|| sin_uso("estado-puertas"))
    }
    fn leer_revision(&self, rol: Rol, pr: u64, origen: u64) -> Paso<RevisionPublicada> {
        assert_eq!(rol, Rol::Agente);
        self.0.lecturas.borrow_mut().push((pr, origen));
        self.0
            .revision
            .borrow()
            .clone()
            .ok_or_else(|| sin_uso("leer-revision"))
    }
    fn merge_pr(&self, _: u64, _: &str) -> Informe {
        sin_uso("merge-pr")
    }
    fn leer_puertas(&self, _: Rol, _: &str) -> Paso<ReglasLeidas> {
        Err(sin_uso("leer-puertas"))
    }
    fn leer_permisos_ci(&self) -> Informe {
        sin_uso("leer-permisos-ci")
    }
    fn firma_commit(&self, _: &str) -> Paso<Firma> {
        Err(sin_uso("firma-commit"))
    }
    fn identidad(&self, _: Rol) -> Paso<Usuario> {
        Err(sin_uso("identidad"))
    }
}

fn catalogo(a: &str) -> Option<Box<dyn Operacion>> {
    toma::desde_anotacion(a).or_else(|| revision::desde_anotacion(a))
}

fn ana() -> Result<Quien, Informe> {
    Ok(Quien::Agente(Cuenta {
        proveedor: Proveedor::GitHub,
        id: 7,
        login: "ana".to_owned(),
    }))
}

const RAMA: &str = "feature/task-001-uno";

/// La seleccion de F6-3 en este repo: `a.txt` suelto va a la generica (S09),
/// que sin plugin instalado se quita con aviso (S14); sin configuracion en la
/// base, el aviso y la escala completa.
const SELECCION: &str = "revisoras= modelo=sonnet reglas=M04,S09,S12,S14 \
     avisos=config-de-base-no-leida,no-instalado:taskcode-plugin:code-quality-reviewer";

/// Remoto bare y clon con TASK-001 dada de alta, empezada por ana (rama y
/// worktree de `start`) y con un commit en su worktree.
fn preparar(f: &Fixture, g: &Git) -> (PathBuf, PathBuf, PathBuf) {
    let (bare, clon) = f.remoto_y_clon();
    let entrada = tc2::alta::entrada_de_args(&["Uno".to_owned()]).unwrap();
    let alta = tc2::alta::Alta::con_entrada(entrada);
    let ctx = estado::Contexto::nuevo(g, &clon, &catalogo, None);
    let i = estado::transaccion(&ctx, &alta);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    let e = toma::Entorno {
        dir: &clon,
        git: g,
        quien: &ana,
        catalogo: &catalogo,
        ahora: tc2::identidad::ahora(),
    };
    let i = toma::start_en(&e, "TASK-001");
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    let wt = clon.join(".taskcodetwo/worktrees/TASK-001");
    escribir(&wt, "a.txt", "trabajo de la tarea\n");
    f.crudo(&wt, &["add", "a.txt"]);
    f.crudo(&wt, &["commit", "-q", "-m", "feat(TASK-001): a"]);
    (bare, clon, wt)
}

fn review(g: &Git, dir: &Path, doble: &Rc<Doble>, esperas: &[Duration]) -> Informe {
    let base = toma::Entorno {
        dir,
        git: g,
        quien: &ana,
        catalogo: &catalogo,
        ahora: tc2::identidad::ahora(),
    };
    let d = Rc::clone(doble);
    let conectar = move || -> Paso<Box<dyn Plataforma>> { Ok(Box::new(Compartida(Rc::clone(&d)))) };
    let e = revision::Entorno {
        base: &base,
        conectar: &conectar,
        esperas,
        claude: None,
    };
    let i = revision::review_en(&e, "TASK-001");
    eprintln!("  {}", i.linea());
    i
}

fn publicado(f: &Fixture, bare: &Path) -> String {
    f.crudo(
        bare,
        &["show", &format!("{}:TASK-001/tarea.md", estado::RAMA)],
    )
}

/// Criterio «PR enlazado a la tarea» (integracion).
#[test]
fn review_empuja_abre_el_pr_y_pasa_a_en_revision() {
    let f = Fixture::nueva("review");
    let g = f.git();
    let (bare, clon, wt) = preparar(&f, &g);
    let doble = Rc::new(Doble::default());

    let i = review(&g, &clon, &doble, &[]);
    assert_eq!(
        i.linea(),
        format!(
            "review ok id=TASK-001 rama={RAMA} pr=12 estado=en-revision revision-ia=sin-revisor {SELECCION}"
        )
    );
    // El push llego: la rama del remoto es la cabeza del worktree.
    let cabeza = f.crudo(&wt, &["rev-parse", "HEAD"]);
    assert_eq!(f.crudo(&bare, &["rev-parse", RAMA]), cabeza);
    // El entorno se pidio para la URL de origin (el bare).
    assert_eq!(doble.urls.borrow().len(), 1);
    assert!(
        doble.urls.borrow()[0].ends_with("remoto.git"),
        "{:?}",
        doble.urls.borrow()
    );
    // `abrir-pr`: rama, develop, titulo y cuerpo con las rutas de la tarea.
    let (rama, base, titulo, cuerpo) = doble.prs.borrow()[0].clone();
    assert_eq!((rama.as_str(), base.as_str()), (RAMA, "develop"));
    assert_eq!(titulo, "TASK-001: Uno");
    assert!(
        cuerpo.contains("`taskcodetwo/tareas:TASK-001/tarea.md`"),
        "{cuerpo}"
    );
    assert!(
        cuerpo.contains("TASK-001/planificacion/plan-final.md"),
        "{cuerpo}"
    );
    // La tarea, en revision en la rama de estado, con su fila.
    let t = publicado(&f, &bare);
    assert!(t.contains("estado: \"en-revision\""), "{t}");
    assert!(t.contains("| review | manual | agente |"), "{t}");
    // F6-3: la seleccion, anotada en `roles.revision`.
    assert!(
        t.contains("revision: {agente: \"general-purpose\", modelo: \"sonnet\", skills: []"),
        "{t}"
    );

    // Repetir sin nada nuevo converge.
    let antes = f.crudo(&bare, &["rev-parse", estado::RAMA]);
    let i = review(&g, &clon, &doble, &[]);
    assert_eq!(
        i.linea(),
        format!(
            "review sin-cambios id=TASK-001 rama={RAMA} pr=12 estado=en-revision revision-ia=sin-revisor {SELECCION}"
        )
    );
    assert_eq!(f.crudo(&bare, &["rev-parse", estado::RAMA]), antes);
    // Desde el worktree de la tarea, lo mismo.
    let i = review(&g, &wt, &doble, &[]);
    assert_eq!(i.resultado, Resultado::SinCambios, "{}", i.linea());
}

/// Requisito de F6-2: el push de `review` va con la identidad del agente
/// (la cabecera del entorno de la plataforma), no con la del puerto de la
/// transaccion; si falla, ni PR ni cambio de estado.
#[test]
fn el_push_de_review_lleva_la_cabecera_del_agente() {
    const TOKEN: &str = "ghs_token_del_agente_secreto";
    let _serie = serie();
    let f = Fixture::nueva("review-cabecera");
    let g = f.git();
    let (bare, clon, _wt) = preparar(&f, &g);
    let mock = Mock::nuevo();
    f.crudo(
        &clon,
        &[
            "config",
            "remote.origin.pushurl",
            &format!("{}/o/r.git", mock.base()),
        ],
    );
    let doble = Rc::new(Doble {
        extra: entorno_git(
            Proveedor::GitHub,
            &format!("{}/", mock.base()),
            &Secreto::nuevo(TOKEN),
            None,
        ),
        ..Doble::default()
    });

    let i = review(&g, &clon, &doble, &[Duration::ZERO, Duration::ZERO]);
    assert!(!i.resultado.es_exito(), "{}", i.linea());
    assert_eq!(i.comando, "review");
    assert!(doble.prs.borrow().is_empty(), "PR abierto sin push");
    assert!(publicado(&f, &bare).contains("estado: \"en-curso\""));

    let basica = STANDARD.encode(format!("x-access-token:{TOKEN}"));
    let recibidas = mock.recibidas();
    let refs = recibidas
        .iter()
        .find(|r| r.linea.starts_with("GET /o/r.git/info/refs"))
        .unwrap_or_else(|| panic!("git no llego al remoto: {recibidas:?}"));
    assert_eq!(
        refs.autorizacion.as_deref(),
        Some(format!("Basic {basica}").as_str())
    );
    // El puerto de la transaccion no lleva el token.
    assert!(!g.registro().join("\n").contains(TOKEN));
}

/// F6-5, criterio 2 (integracion con Git real): `review` guarda el informe
/// del check `revision-ia` ya terminado en la rama de estado; repetido sale
/// `sin-cambios`; con el check pendiente no escribe nada.
#[test]
fn review_guarda_el_informe_de_la_revision_ia() {
    let f = Fixture::nueva("review-informe");
    let g = f.git();
    let (bare, clon, wt) = preparar(&f, &g);
    // El origen del check en la configuracion de la base.
    escribir(
        &clon,
        ".taskcode/config.yml",
        "---\nformato: 2\nplataforma:\n  proveedor: \"github\"\n  api: \"https://api.github.com\"\n  revisor_id: 99001\n",
    );
    f.crudo(&clon, &["add", ".taskcode/config.yml"]);
    f.crudo(&clon, &["commit", "-q", "-m", "config"]);
    f.crudo(&clon, &["push", "-q", "origin", "develop"]);
    let cabeza = f.crudo(&wt, &["rev-parse", "HEAD"]).trim().to_owned();
    let sha12 = &cabeza[..12];
    let doble = Rc::new(Doble::default());
    let rama_estado = || f.crudo(&bare, &["rev-parse", estado::RAMA]);
    let en_estado = |ruta: &str| {
        f.crudo(
            &bare,
            &["ls-tree", "-r", "--name-only", estado::RAMA, "--", ruta],
        )
        .trim()
        .to_owned()
    };

    // Check pendiente: ni se lee la revision ni se escribe nada.
    *doble.puertas.borrow_mut() = Some(
        Informe::nuevo("estado-puertas", Resultado::Esperando)
            .con_lista("falta", vec!["revision-ia".to_owned()]),
    );
    let i = review(&g, &clon, &doble, &[]);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert!(
        i.linea().contains(" revision-ia=pendiente revisoras="),
        "{}",
        i.linea()
    );
    assert!(!i.linea().contains("hallazgos="), "{}", i.linea());
    assert!(doble.lecturas.borrow().is_empty());
    assert_eq!(en_estado("TASK-001/revision"), "");

    // Terminado en rojo: se lee con el agente y se guarda.
    *doble.puertas.borrow_mut() = Some(
        Informe::nuevo("estado-puertas", Resultado::Rechazada)
            .con("motivo", "check-rojo")
            .con_lista("checks", vec!["revision-ia".to_owned()]),
    );
    let aceptado: tc2::hallazgos::AceptadoDe = serde_json::from_str(
        r#"{"revisora":"rev:a","hallazgo":{"id":"I1","severidad":"IMPORTANTE","clase":"contrato","fichero":"a.txt","linea":1,"descripcion":"falta | algo","evidencia":{"tipo":"borde","entrada":"x","esperado":"1","obtenido":"2"}}}"#,
    )
    .unwrap();
    *doble.revision.borrow_mut() = Some(RevisionPublicada {
        sha: cabeza.clone(),
        conclusion: "failure".to_owned(),
        titulo: "revision-ia: rojo (criticos=0 importantes=1 menores=0)".to_owned(),
        resumen: "**Veredicto: rojo**".to_owned(),
        aceptados: vec![aceptado.clone()],
    });
    let antes = rama_estado();
    let i = review(&g, &clon, &doble, &[]);
    let ruta = format!("TASK-001/revision/revision-ia-{sha12}.json");
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert!(
        i.linea().contains(&format!(
            " revision-ia=fallo hallazgos=rojo informe={ruta} revisoras="
        )),
        "{}",
        i.linea()
    );
    assert_eq!(*doble.lecturas.borrow(), [(12, 99001)]);
    assert_ne!(rama_estado(), antes);
    assert_eq!(
        en_estado("TASK-001/revision"),
        format!("{ruta}\nTASK-001/revision/revision-ia-{sha12}.md")
    );
    let json = f.crudo(&bare, &["show", &format!("{}:{ruta}", estado::RAMA)]);
    let leido: tc2::hallazgos::InformeTarea = serde_json::from_str(&json).unwrap();
    assert_eq!(
        (
            leido.tarea.as_str(),
            leido.pr,
            leido.sha.as_str(),
            leido.importantes
        ),
        ("TASK-001", 12, cabeza.as_str(), 1)
    );
    assert_eq!(leido.veredicto, tc2::hallazgos::Dictamen::Rojo);
    assert_eq!(leido.aceptados, [aceptado]);
    let md = f.crudo(
        &bare,
        &[
            "show",
            &format!("{}:TASK-001/revision/revision-ia-{sha12}.md", estado::RAMA),
        ],
    );
    assert!(md.contains("Veredicto: **rojo**"), "{md}");
    assert!(
        md.contains(r"| rev:a | 1 | abierto | I1 | IMPORTANTE | contrato | a.txt | 1 | borde | falta \| algo |"),
        "{md}"
    );

    // Repetir: el mismo informe, nada que escribir.
    let antes = rama_estado();
    let i = review(&g, &clon, &doble, &[]);
    assert_eq!(i.resultado, Resultado::SinCambios, "{}", i.linea());
    assert!(
        i.linea()
            .contains(&format!("hallazgos=rojo informe={ruta}")),
        "{}",
        i.linea()
    );
    assert_eq!(rama_estado(), antes);

    // Un titulo que no es de tc2 (o una lectura que falla): sin-leer, sin
    // escribir.
    doble.revision.borrow_mut().as_mut().unwrap().titulo = "revision-ia: neutral (F6-4)".to_owned();
    let i = review(&g, &clon, &doble, &[]);
    assert!(
        i.linea()
            .contains(" revision-ia=fallo hallazgos=sin-leer revisoras="),
        "{}",
        i.linea()
    );
    assert_eq!(rama_estado(), antes);
}
