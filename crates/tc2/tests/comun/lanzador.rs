//! Lanzador unico del binario `tc2` para los tests de integracion (A8-02).
//!
//! Todos los tests lanzan el binario con la misma base: sin depender del
//! locale de la maquina (`LC_ALL=C`, `TC2_IDIOMA=es`), sin que git pregunte
//! credenciales (`GIT_TERMINAL_PROMPT=0`) y sin las credenciales de la
//! maquina (tokens de los tres roles y GitHub Apps). Cada test anade despues
//! lo suyo: en `Command` gana la ultima operacion sobre cada variable, asi
//! que un `.env(..)` o `.env_remove(..)` posterior cambia la base.

use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, Output};

use tc2::plataforma::Rol;

/// Ruta del binario compilado por `cargo test`.
pub const BINARIO: &str = env!("CARGO_BIN_EXE_tc2");

/// `tc2` listo para lanzar con la base comun; el test pone carpeta,
/// argumentos y lo que le haga falta.
pub fn orden_tc2() -> Command {
    let mut c = Command::new(BINARIO);
    c.env("LC_ALL", "C")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env(tc2::i18n::VARIABLE, "es");
    for rol in [Rol::Agente, Rol::Revisor, Rol::Persona] {
        c.env_remove(rol.variable());
        if let Some((id, clave)) = rol.variables_app() {
            c.env_remove(id).env_remove(clave);
        }
    }
    c
}

/// `tc2 <args>` en `dir` con la base comun y `entorno`.
pub fn lanzar(dir: &Path, entorno: Vec<(OsString, OsString)>, args: &[&str]) -> Output {
    orden_tc2()
        .current_dir(dir)
        .args(args)
        .envs(entorno)
        .output()
        .expect("lanzar tc2")
}

/// Bytes de una salida como texto.
pub fn texto(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

/// Valor de `clave=` en la linea de contrato: la primera linea de `salida`,
/// sin el final de linea.
pub fn valor<'a>(salida: &'a str, clave: &str) -> Option<&'a str> {
    salida
        .lines()
        .next()?
        .trim_end()
        .split(' ')
        .find_map(|kv| kv.strip_prefix(clave)?.strip_prefix('='))
}

/// Contrato de una salida de `tc2`: una sola linea en stdout; un exito no
/// escribe en stderr y un fallo, una linea. Da el codigo y esa linea.
pub fn salida(o: &Output) -> (Option<i32>, String) {
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

/// Salida acotada de `doctor` (F1-4 decision 4) en un caso que no cabe:
/// `json` (la de `--json`) en una linea, sin ANSI y con todos los
/// hallazgos; mas de 4 que no son `ok`, y `lineas` (las del texto) son 5 con
/// la ultima `y N mas: <detalle>`. Da el JSON leido.
pub fn salida_acotada(lineas: &[&str], json: &str, detalle: &str) -> serde_json::Value {
    assert_eq!(json.lines().count(), 1, "{json}");
    assert!(!json.contains('\u{1b}'), "{json}");
    let v: serde_json::Value = serde_json::from_str(json).unwrap_or_else(|e| panic!("{e}: {json}"));
    let n = |k: &str| {
        v["claves"][k]
            .as_str()
            .and_then(|x| x.parse::<usize>().ok())
            .unwrap_or_else(|| panic!("sin {k}: {json}"))
    };
    let malos = n("fallos") + n("avisos") + n("no-comprobado");
    let todos = v["hallazgos"].as_array().expect("hallazgos");
    let oks = todos.iter().filter(|h| h["estado"] == "ok").count();
    assert_eq!(todos.len(), malos + oks, "{json}");
    assert!(malos > 4, "el caso tiene que no caber: {json}");
    assert_eq!(lineas.len(), 5, "{lineas:?}");
    assert_eq!(lineas[4], format!("y {} mas: {detalle}", malos - 3));
    v
}

/// Codigo, stdout y stderr de una salida, para el mensaje de un assert.
pub fn volcado(o: &Output) -> String {
    format!(
        "codigo {:?}\nstdout: {}\nstderr: {}",
        o.status.code(),
        texto(&o.stdout),
        texto(&o.stderr)
    )
}
