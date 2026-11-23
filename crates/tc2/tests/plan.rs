//! F4-1 (TASK-036): `tc2 plan` y `tc2 plan guardar` con el binario real,
//! como los lanza el agente (sin terminal: quien los lanza sale de la cache
//! de identidad), contra un remoto bare de verdad.
//!
//! Como en `tests/ciclo.rs`: `origin` apunta a `ssh://git@github.com/o/r.git`
//! y el transporte ssh se sustituye con `GIT_SSH_COMMAND`, que sirve el
//! remoto bare de la fixture.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Output;

use comun::{Fixture, escribir};
use tc2::contrato::Resultado;
use tc2::estado::{self, Cambio, Contexto, Estado, Fallo, Operacion};
use tc2::identidad::{Cache, Cuenta};
use tc2::plataforma::Proveedor;

const RAMA_ESTADO: &str = estado::RAMA;
const ORIGIN: &str = "ssh://git@github.com/o/r.git";
const PERSONAS: &str =
    "---\npersonas:\n  - {id: 1001, login: \"ana\"}\n  - {id: 1002, login: \"beto\"}\n";
/// Configuracion de la base con el modo que `plan` congela.
const CONFIG: &str = "---\nformato: 2\nmodo_flujo: automatico\n";

fn entorno_ssh(f: &Fixture) -> Vec<(OsString, OsString)> {
    let bare = f
        .raiz
        .join("remoto.git")
        .to_string_lossy()
        .replace('\\', "/");
    let mut e = f.entorno();
    e.push((
        "GIT_SSH_COMMAND".into(),
        format!(
            "sh -c 'case \"$2\" in *receive-pack*) exec git receive-pack \"$0\";; *) exec git upload-pack \"$0\";; esac' '{bare}'"
        )
        .into(),
    ));
    e.push(("GIT_SSH_VARIANT".into(), "simple".into()));
    e
}

fn lanzar(dir: &Path, entorno: Vec<(OsString, OsString)>, args: &[&str]) -> Output {
    comun::orden_tc2()
        .current_dir(dir)
        .args(args)
        .envs(entorno)
        .output()
        .expect("lanzar tc2")
}

fn texto(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

/// `tc2 <args>` en `dir` con el remoto: (codigo, la unica linea de stdout).
/// Un exito no escribe en stderr; un fallo, una linea.
fn tc2(f: &Fixture, dir: &Path, args: &[&str]) -> (Option<i32>, String) {
    let o = lanzar(dir, entorno_ssh(f), args);
    let out = texto(&o.stdout);
    assert_eq!(out.lines().count(), 1, "{out}");
    let err = texto(&o.stderr);
    if o.status.code() == Some(0) {
        assert!(err.is_empty(), "stderr en un exito: {err}");
    } else {
        assert_eq!(err.lines().count(), 1, "{err}");
    }
    (o.status.code(), out.trim_end().to_owned())
}

fn valor<'a>(linea: &'a str, clave: &str) -> Option<&'a str> {
    linea
        .split(' ')
        .find_map(|kv| kv.strip_prefix(clave)?.strip_prefix('='))
}

fn identificar(dir: &Path, id: u64, login: &str) {
    let cuenta = Cuenta {
        proveedor: Proveedor::GitHub,
        id,
        login: login.to_owned(),
    };
    Cache::en(&dir.join(".git"))
        .guardar(&cuenta, tc2::identidad::ahora())
        .unwrap();
}

/// Remoto con `personas.yml`, la configuracion (modo automatico) y las
/// tareas de `altas` (`tc2 new` con sus argumentos); el clon `a` que
/// devuelve tiene `origin` en GitHub y la identidad de ana.
fn preparar(f: &Fixture, altas: &[&[&str]]) -> (PathBuf, PathBuf) {
    let (bare, prep) = f.remoto_y_clon();
    escribir(&prep, ".taskcode/personas.yml", PERSONAS);
    escribir(&prep, ".taskcode/config.yml", CONFIG);
    f.crudo(&prep, &["add", ".taskcode"]);
    f.crudo(&prep, &["commit", "-q", "-m", "personas y config"]);
    f.crudo(&prep, &["push", "-q", "origin", "develop"]);
    for args in altas {
        let mut v = vec!["new"];
        v.extend_from_slice(args);
        let o = lanzar(&prep, f.entorno(), &v);
        assert_eq!(o.status.code(), Some(0), "{}", texto(&o.stdout));
    }
    f.crudo(&f.raiz, &["clone", "-q", bare.to_str().unwrap(), "a"]);
    let a = f.raiz.join("a");
    f.crudo(&a, &["remote", "set-url", "origin", ORIGIN]);
    identificar(&a, 1001, "ana");
    (bare, a)
}

fn publicado(f: &Fixture, bare: &Path, ruta: &str) -> String {
    f.crudo(bare, &["show", &format!("{RAMA_ESTADO}:{ruta}")])
}

/// Los ficheros publicados bajo `TASK-NNN/planificacion/`, relativos a ella.
fn planificacion(f: &Fixture, bare: &Path, id: &str) -> Vec<String> {
    let dentro = format!("{id}/planificacion/");
    f.crudo(bare, &["ls-tree", "-r", "--name-only", RAMA_ESTADO])
        .lines()
        .filter_map(|l| l.strip_prefix(&dentro).map(str::to_owned))
        .collect()
}

fn punta(f: &Fixture, bare: &Path) -> String {
    f.crudo(bare, &["rev-parse", RAMA_ESTADO])
}

const FIX: &[&str] = &[
    "Arreglo",
    "--tipo",
    "fix",
    "--objetivo",
    "Que no falle",
    "--criterio",
    "no falla",
];
const FUNC: &[&str] = &[
    "Funcion",
    "--objetivo",
    "Una funcion nueva",
    "--criterio",
    "hace algo",
    "--criterio",
    "y algo mas",
];
const SIN_ENUNCIADO: &[&str] = &["Sin enunciado"];

/// Criterio 2 (E2E): una tarea fix publica solo `peticion-plan-1.md`
/// (un rol, arquitectura, sin unificador); con `--carril riesgo-alto`, las
/// cuatro peticiones de rol y la del unificador.
#[test]
fn c2_uno_en_fix_y_cuatro_mas_unificador_en_riesgo_alto() {
    let f = Fixture::nueva("plan-c2");
    let (bare, a) = preparar(&f, &[FIX, FUNC]);
    let (c, out) = tc2(&f, &a, &["plan", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    assert!(out.starts_with("plan ok id=TASK-001 commit="), "{out}");
    assert_eq!(valor(&out, "carril"), Some("fix"), "{out}");
    assert_eq!(valor(&out, "roles"), Some("arquitectura"), "{out}");
    assert_eq!(valor(&out, "forma"), Some("redaccion"), "{out}");
    assert_eq!(
        valor(&out, "peticiones"),
        Some("peticion-plan-1.md"),
        "{out}"
    );
    assert_eq!(
        planificacion(&f, &bare, "TASK-001"),
        ["brainstorm/peticion-plan-1.md", "plan-final.md"]
    );
    let p = publicado(
        &f,
        &bare,
        "TASK-001/planificacion/brainstorm/peticion-plan-1.md",
    );
    assert!(p.contains("- Rol: `brainstorm-arquitectura`"), "{p}");
    assert!(p.contains("> Dado lo que ya existe, ¿donde encaja este cambio y que forma tiene?"));

    let (c, out) = tc2(
        &f,
        &a,
        &["plan", "--json", "TASK-002", "--carril", "riesgo-alto"],
    );
    assert_eq!(c, Some(0), "{out}");
    let j: serde_json::Value = serde_json::from_str(&out).unwrap();
    let v = &j["claves"];
    assert_eq!(v["carril"], "riesgo-alto", "{out}");
    assert_eq!(v["forma"], "unificador", "{out}");
    assert_eq!(
        v["roles"],
        serde_json::json!(["arquitectura", "riesgos", "testing", "dominio"])
    );
    let b = "brainstorm/";
    assert_eq!(
        planificacion(&f, &bare, "TASK-002"),
        [
            format!("{b}peticion-brainstorm-arquitectura-1.md"),
            format!("{b}peticion-brainstorm-dominio-1.md"),
            format!("{b}peticion-brainstorm-riesgos-1.md"),
            format!("{b}peticion-brainstorm-testing-1.md"),
            format!("{b}peticion-unificador-1.md"),
            "plan-final.md".to_owned(),
        ]
    );
    let u = publicado(
        &f,
        &bare,
        "TASK-002/planificacion/brainstorm/peticion-unificador-1.md",
    );
    assert!(u.contains("- Rol: `brainstorm-unificador`"), "{u}");
}

/// `tc2 <args>` como [`tc2`], con la carpeta de Claude Code `claude`.
fn tc2_claude(f: &Fixture, dir: &Path, claude: &Path, args: &[&str]) -> (Option<i32>, String) {
    let mut e = entorno_ssh(f);
    e.push(("CLAUDE_CONFIG_DIR".into(), claude.as_os_str().to_owned()));
    let o = lanzar(dir, e, args);
    let out = texto(&o.stdout);
    assert_eq!(out.lines().count(), 1, "{out}");
    (o.status.code(), out.trim_end().to_owned())
}

/// F4-6, criterio 2 (E2E): el modelo cambia con el carril y la complejidad
/// y queda anotado por rol. Una `fix` simple: diseno e implementacion
/// `sonnet` (M02), un rol, su agente del inventario y el modelo en la
/// peticion; con `--carril riesgo-alto`: `opus` en los dos (M07), los
/// cuatro roles y el unificador, cada peticion con su agente y su modelo.
#[test]
fn plan_anota_modelos_distintos() {
    let f = Fixture::nueva("plan-modelos");
    let claude = comun::claude_con_brainstorm(&f.raiz, &[]);
    let (bare, a) = preparar(&f, &[FIX, FUNC]);
    let (c, out) = tc2_claude(&f, &a, &claude, &["plan", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    assert_eq!(valor(&out, "modelo-diseno"), Some("sonnet"), "{out}");
    assert_eq!(
        valor(&out, "modelo-implementacion"),
        Some("sonnet"),
        "{out}"
    );
    assert_eq!(valor(&out, "dominios"), Some("ninguno"), "{out}");
    assert_eq!(valor(&out, "avisos"), Some(""), "{out}");
    let md = publicado(&f, &bare, "TASK-001/tarea.md");
    assert!(
        md.contains("  diseno: {modelo: \"sonnet\", skills: [], regla: null, agentes: [\"taskcode-plugin:brainstorm-arquitectura\"], reglas: [\"M02\", \"S02\"], inventario: \""),
        "{md}"
    );
    assert!(
        md.contains("  implementacion: {modelo: \"sonnet\", skills: [], regla: null, reglas: [\"M02\"], inventario: \""),
        "{md}"
    );
    let p = publicado(
        &f,
        &bare,
        "TASK-001/planificacion/brainstorm/peticion-plan-1.md",
    );
    assert!(
        p.contains("- Rol: `brainstorm-arquitectura` — lanzalo con el agente `taskcode-plugin:brainstorm-arquitectura` y el modelo `sonnet`"),
        "{p}"
    );

    let (c, out) = tc2_claude(
        &f,
        &a,
        &claude,
        &["plan", "TASK-002", "--carril", "riesgo-alto"],
    );
    assert_eq!(c, Some(0), "{out}");
    assert_eq!(valor(&out, "modelo-diseno"), Some("opus"), "{out}");
    assert_eq!(valor(&out, "modelo-implementacion"), Some("opus"), "{out}");
    let md = publicado(&f, &bare, "TASK-002/tarea.md");
    assert!(
        md.contains("  diseno: {modelo: \"opus\", skills: [], regla: null, agentes: [\"taskcode-plugin:brainstorm-arquitectura\", \"taskcode-plugin:brainstorm-riesgos\", \"taskcode-plugin:brainstorm-testing\", \"taskcode-plugin:brainstorm-dominio\", \"taskcode-plugin:brainstorm-unificador\"], reglas: [\"M07\", \"S02\", \"S05\"]"),
        "{md}"
    );
    assert!(md.contains("  implementacion: {modelo: \"opus\""), "{md}");
    let u = publicado(
        &f,
        &bare,
        "TASK-002/planificacion/brainstorm/peticion-unificador-1.md",
    );
    assert!(
        u.contains(
            "lanzalo con el agente `taskcode-plugin:brainstorm-unificador` y el modelo `opus`"
        ),
        "{u}"
    );
    let r = publicado(
        &f,
        &bare,
        "TASK-002/planificacion/brainstorm/peticion-brainstorm-dominio-1.md",
    );
    assert!(
        r.contains("lanzalo con el agente `taskcode-plugin:brainstorm-dominio` y el modelo `opus`"),
        "{r}"
    );
}

/// Criterio 3 (E2E): `plan` publica en `TASK-NNN/planificacion/`, deja la
/// tarea `en-diseno` con el carril y el modo congelados y su fila de
/// transicion; repetir da `sin-cambios` sin tocar el remoto; subir el
/// carril abre la ronda 2 sin pisar el `plan-final.md` ya guardado.
#[test]
fn c3_publica_congela_y_repetir_no_cambia_nada() {
    let f = Fixture::nueva("plan-c3");
    let (bare, a) = preparar(&f, &[FUNC]);
    let antes = publicado(&f, &bare, "TASK-001/tarea.md");
    let (c, out) = tc2(&f, &a, &["plan", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    assert_eq!(valor(&out, "modo"), Some("automatico"), "{out}");
    assert_eq!(valor(&out, "ronda"), Some("1"), "{out}");
    let md = publicado(&f, &bare, "TASK-001/tarea.md");
    assert!(
        md.contains("\ncarril: \"funcionalidad\"\nestado: \"en-diseno\"\n"),
        "{md}"
    );
    assert!(antes.contains("\ncarril: null\nestado: \"planificada\"\n"));
    let fila = md.lines().last().unwrap();
    assert!(
        fila.ends_with(&format!(
            " | plan | automatico | agente | {} |",
            tc2::VERSION
        )),
        "{fila}"
    );
    let esqueleto = publicado(&f, &bare, "TASK-001/planificacion/plan-final.md");
    assert!(
        esqueleto.starts_with("# Plan — TASK-001: Funcion\n"),
        "{esqueleto}"
    );
    assert!(esqueleto.contains("\nDuracion estimada: N min\n"));

    // Repetir: sin cambios y la punta del estado igual.
    let p = punta(&f, &bare);
    let (c, out) = tc2(&f, &a, &["plan", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    assert!(out.starts_with("plan sin-cambios id=TASK-001"), "{out}");
    assert_eq!(valor(&out, "ronda"), Some("1"), "{out}");
    assert_eq!(punta(&f, &bare), p);

    // Un plan guardado no se pisa al volver a planificar con mas roles.
    let fuera = f.raiz.join("plan.md");
    std::fs::write(&fuera, "# Mi plan\n").unwrap();
    let (c, out) = tc2(
        &f,
        &a,
        &[
            "plan",
            "guardar",
            "TASK-001",
            "plan-final.md",
            fuera.to_str().unwrap(),
        ],
    );
    assert_eq!(c, Some(0), "{out}");
    let (c, out) = tc2(&f, &a, &["plan", "TASK-001", "--carril", "riesgo-alto"]);
    assert_eq!(c, Some(0), "{out}");
    assert_eq!(valor(&out, "ronda"), Some("2"), "{out}");
    assert_eq!(valor(&out, "modo"), Some("automatico"), "{out}");
    assert_eq!(
        publicado(&f, &bare, "TASK-001/planificacion/plan-final.md"),
        "# Mi plan\n"
    );
    let r2 = publicado(
        &f,
        &bare,
        "TASK-001/planificacion/brainstorm/peticion-unificador-2.md",
    );
    assert!(r2.contains("re-planificacion"), "{r2}");
    // Bajar el carril se rechaza sin escribir.
    let p = punta(&f, &bare);
    let (c, out) = tc2(&f, &a, &["plan", "TASK-001", "--carril", "fix"]);
    assert_eq!(c, Some(3), "{out}");
    assert_eq!(valor(&out, "motivo"), Some("carril-baja"), "{out}");
    assert_eq!(punta(&f, &bare), p);
}

/// Criterio 3 (E2E): un enunciado que no vale (el de la plantilla) da
/// `rechazada` sin escribir nada.
#[test]
fn c3_enunciado_invalido_rechazada_sin_escribir() {
    let f = Fixture::nueva("plan-enunciado");
    let (bare, a) = preparar(&f, &[SIN_ENUNCIADO]);
    let antes = f.huella(&bare, &a);
    let (c, out) = tc2(&f, &a, &["plan", "TASK-001"]);
    assert_eq!(c, Some(3), "{out}");
    assert_eq!(out, "plan rechazada id=TASK-001 motivo=objetivo-vacio");
    assert_eq!(f.huella(&bare, &a), antes);
    assert!(planificacion(&f, &bare, "TASK-001").is_empty());
    assert!(publicado(&f, &bare, "TASK-001/tarea.md").contains("estado: \"planificada\""));
}

/// Criterio 3 (E2E): `plan guardar` publica `plan-final.md` y la salida de
/// un rol lanzado; repetir da `sin-cambios`; un destino fuera de la lista
/// es `uso` sin tocar nada.
#[test]
fn c3_plan_guardar_publica_y_rechaza_destinos_fuera_de_la_lista() {
    let f = Fixture::nueva("plan-guardar");
    let (bare, a) = preparar(&f, &[FUNC]);
    let fuera = f.raiz.join("salida.md");
    std::fs::write(&fuera, "# Plan\r\n\r\nhecho\r\n").unwrap();
    let ruta = fuera.to_str().unwrap();
    // Antes de planificar no esta en diseno.
    let (c, out) = tc2(
        &f,
        &a,
        &["plan", "guardar", "TASK-001", "plan-final.md", ruta],
    );
    assert_eq!(c, Some(3), "{out}");
    assert_eq!(valor(&out, "motivo"), Some("estado-no-valido"), "{out}");
    let (c, out) = tc2(&f, &a, &["plan", "TASK-001", "--carril", "riesgo-alto"]);
    assert_eq!(c, Some(0), "{out}");
    let (c, out) = tc2(
        &f,
        &a,
        &["plan", "guardar", "TASK-001", "plan-final.md", ruta],
    );
    assert_eq!(c, Some(0), "{out}");
    assert!(
        out.starts_with("plan ok id=TASK-001 destino=plan-final.md commit="),
        "{out}"
    );
    assert_eq!(
        publicado(&f, &bare, "TASK-001/planificacion/plan-final.md"),
        "# Plan\n\nhecho\n"
    );
    let (c, out) = tc2(
        &f,
        &a,
        &["plan", "guardar", "TASK-001", "plan-final.md", ruta],
    );
    assert_eq!(c, Some(0), "{out}");
    assert!(
        out.starts_with("plan sin-cambios id=TASK-001 destino=plan-final.md"),
        "{out}"
    );
    let salida = "brainstorm/salida-brainstorm-riesgos-1.md";
    let (c, out) = tc2(&f, &a, &["plan", "guardar", "TASK-001", salida, ruta]);
    assert_eq!(c, Some(0), "{out}");
    assert!(publicado(&f, &bare, &format!("TASK-001/planificacion/{salida}")).contains("hecho"));
    // La salida de un rol que no se lanzo en esa ronda.
    let (c, out) = tc2(
        &f,
        &a,
        &[
            "plan",
            "guardar",
            "TASK-001",
            "brainstorm/salida-brainstorm-riesgos-2.md",
            ruta,
        ],
    );
    assert_eq!(c, Some(3), "{out}");
    assert_eq!(valor(&out, "motivo"), Some("sin-peticion"), "{out}");
    let antes = f.huella(&bare, &a);
    for destino in [
        "tarea.md",
        "../TASK-002/tarea.md",
        "brainstorm/peticion-plan-1.md",
        "otro.md",
    ] {
        let (c, out) = tc2(&f, &a, &["plan", "guardar", "TASK-001", destino, ruta]);
        assert_eq!(c, Some(2), "{destino}: {out}");
        assert_eq!(out, "plan uso motivo=destino-invalido", "{destino}");
    }
    assert_eq!(f.huella(&bare, &a), antes);
}

/// Escribe en la rama de estado por la transaccion (como otra fase).
struct Escribir {
    ruta: String,
    de: &'static str,
    a: &'static str,
}

impl Operacion for Escribir {
    fn nombre(&self) -> &str {
        "escribir"
    }
    fn anotacion(&self) -> String {
        format!("escribir|{}", self.ruta)
    }
    fn exige_red(&self) -> bool {
        true
    }
    fn aplicar(&self, e: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        let t = e.leer(&self.ruta)?.unwrap_or_default();
        assert!(t.contains(self.de), "{t}");
        Ok(vec![Cambio::nuevo(
            self.ruta.clone(),
            t.replacen(self.de, self.a, 1),
        )])
    }
}

fn sin_catalogo(_: &str) -> Option<Box<dyn Operacion>> {
    None
}

/// Cambia `de` por `a` en el `tarea.md` publicado de `id`.
fn editar(f: &Fixture, dir: &Path, id: &str, de: &'static str, a: &'static str) {
    let g = tc2::git::Git::nuevo().con_entorno(entorno_ssh(f));
    let op = Escribir {
        ruta: format!("{id}/tarea.md"),
        de,
        a,
    };
    let i = estado::transaccion(&Contexto::nuevo(&g, dir, &sin_catalogo, None), &op);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
}

/// MEN-1 y MEN-2 de la revision: la tarea de otra persona no se guarda
/// (`de-otra-persona` con su login); con el plan aprobado, ni `plan` ni
/// `plan guardar` cambian nada (`plan-aprobado`).
#[test]
fn men_otra_persona_y_plan_aprobado() {
    let f = Fixture::nueva("plan-men");
    let (bare, a) = preparar(&f, &[FUNC]);
    let (c, out) = tc2(&f, &a, &["plan", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    let fuera = f.raiz.join("plan.md");
    std::fs::write(&fuera, "# Plan\n").unwrap();
    let ruta = fuera.to_str().unwrap();
    editar(
        &f,
        &a,
        "TASK-001",
        "asignado_a: null",
        "asignado_a: {id: 1002, login: \"beto\"}",
    );
    let p = punta(&f, &bare);
    let (c, out) = tc2(
        &f,
        &a,
        &["plan", "guardar", "TASK-001", "plan-final.md", ruta],
    );
    assert_eq!(c, Some(3), "{out}");
    assert_eq!(valor(&out, "motivo"), Some("de-otra-persona"), "{out}");
    assert_eq!(valor(&out, "asignado"), Some("beto"), "{out}");
    assert_eq!(punta(&f, &bare), p);
    editar(
        &f,
        &a,
        "TASK-001",
        "asignado_a: {id: 1002, login: \"beto\"}",
        "asignado_a: {id: 1001, login: \"ana\"}",
    );
    editar(
        &f,
        &a,
        "TASK-001",
        "plan_aprobado: false",
        "plan_aprobado: true",
    );
    let p = punta(&f, &bare);
    for args in [
        vec!["plan", "guardar", "TASK-001", "plan-final.md", ruta],
        vec!["plan", "TASK-001"],
        vec!["plan", "TASK-001", "--carril", "riesgo-alto"],
    ] {
        let (c, out) = tc2(&f, &a, &args);
        assert_eq!(c, Some(3), "{args:?}: {out}");
        assert_eq!(valor(&out, "motivo"), Some("plan-aprobado"), "{out}");
    }
    assert_eq!(punta(&f, &bare), p);
}
