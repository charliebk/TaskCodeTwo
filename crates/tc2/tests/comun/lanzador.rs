//! Lanzador unico del binario `tc2` para los tests de integracion (A8-02).
//!
//! Todos los tests lanzan el binario con la misma base: sin depender del
//! locale de la maquina (`LC_ALL=C`, `TC2_IDIOMA=es`), sin que git pregunte
//! credenciales (`GIT_TERMINAL_PROMPT=0`) y sin las credenciales de la
//! maquina (tokens de los tres roles y GitHub Apps). Cada test anade despues
//! lo suyo: en `Command` gana la ultima operacion sobre cada variable, asi
//! que un `.env(..)` o `.env_remove(..)` posterior cambia la base.

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

/// Codigo, stdout y stderr de una salida, para el mensaje de un assert.
pub fn volcado(o: &Output) -> String {
    format!(
        "codigo {:?}\nstdout: {}\nstderr: {}",
        o.status.code(),
        texto(&o.stdout),
        texto(&o.stderr)
    )
}
