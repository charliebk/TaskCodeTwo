//! Nucleo del CLI `tc2` de TaskCodeTwo.
//!
//! El esqueleto (F2-1) solo sabe decir su version. Las capas que vienen
//! (operaciones Git de F2-2, modelo de tarea de F2-5...) entran como modulos
//! de esta biblioteca.

/// Version del CLI, la de `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Resultado de interpretar la linea de ordenes: que escribir y con que
/// codigo salir. Sin efectos, para poder probarlo sin lanzar procesos.
#[derive(Debug, PartialEq, Eq)]
pub struct Salida {
    /// Codigo de salida del proceso.
    pub codigo: u8,
    /// Texto para la salida estandar (vacio si no hay).
    pub stdout: String,
    /// Texto para la salida de error (vacio si no hay).
    pub stderr: String,
}

/// Interpreta los argumentos (sin el nombre del programa).
pub fn ejecutar<I, S>(args: I) -> Salida
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args: Vec<String> = args.into_iter().map(|a| a.as_ref().to_owned()).collect();
    match args.as_slice() {
        [a] if a == "--version" || a == "-V" => Salida {
            codigo: 0,
            stdout: format!("tc2 {VERSION}\n"),
            stderr: String::new(),
        },
        [] => Salida {
            codigo: 2,
            stdout: String::new(),
            stderr: "tc2: falta la orden. Uso: tc2 --version\n".to_owned(),
        },
        [otro, ..] => Salida {
            codigo: 2,
            stdout: String::new(),
            stderr: format!("tc2: orden desconocida «{otro}». Uso: tc2 --version\n"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_corta_y_larga() {
        for a in ["--version", "-V"] {
            let s = ejecutar([a]);
            assert_eq!(s.codigo, 0);
            assert_eq!(s.stdout, format!("tc2 {VERSION}\n"));
        }
    }

    #[test]
    fn sin_orden_falla() {
        assert_eq!(ejecutar(Vec::<String>::new()).codigo, 2);
    }
}
