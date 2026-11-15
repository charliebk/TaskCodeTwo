//! Nucleo del CLI `tc2` de TaskCodeTwo.
//!
//! Modulos: [`contrato`] (salida de toda operacion, ADR F1-4), [`git`]
//! (capa de operaciones Git deterministas, F2-2) y [`estado`] (rama de
//! estado `taskcode/tareas`, F2-3). Las capas que vienen (modelo de tarea de
//! F2-5...) entran como modulos de esta biblioteca.

pub mod contrato;
pub mod estado;
pub mod git;

use contrato::{Informe, Resultado};

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

/// Uso del binario, para los mensajes de error.
const USO: &str = "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json]";

/// Interpreta los argumentos (sin el nombre del programa) y ejecuta la orden
/// en la carpeta actual.
pub fn ejecutar<I, S>(args: I) -> Salida
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args: Vec<String> = args.into_iter().map(|a| a.as_ref().to_owned()).collect();
    match args.as_slice() {
        [a] if a == "--version" || a == "-V" => Salida {
            codigo: 0,
            stdout: format!(
                "tc2 {VERSION}
"
            ),
            stderr: String::new(),
        },
        [orden, resto @ ..] if orden == "guardar" || orden == "actualizar-rama" => {
            let (json, resto) = match resto {
                [j, r @ ..] if j == "--json" => (true, r),
                r => (false, r),
            };
            orden_git(orden, resto).salida(json)
        }
        [] => Salida {
            codigo: 2,
            stdout: String::new(),
            stderr: format!(
                "tc2: falta la orden. {USO}
"
            ),
        },
        [otro, ..] => Salida {
            codigo: 2,
            stdout: String::new(),
            stderr: format!(
                "tc2: orden desconocida «{otro}». {USO}
"
            ),
        },
    }
}

/// Las dos operaciones Git publicas (ADR F1-4 D1a), en la carpeta actual.
fn orden_git(orden: &str, args: &[String]) -> Informe {
    let dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            return Informe::nuevo(orden, Resultado::Error)
                .diagnostico(format!("no se puede leer la carpeta actual: {e}"));
        }
    };
    let g = git::Git::nuevo();
    if orden == "guardar" {
        let Some((mensaje, rutas)) = args.split_first() else {
            return Informe::nuevo(orden, Resultado::Uso)
                .con("motivo", "sin-mensaje")
                .diagnostico("uso: guardar \"<mensaje>\" <rutas...>");
        };
        let rutas: Vec<&str> = rutas.iter().map(String::as_str).collect();
        git::guardar(&g, &dir, mensaje, &rutas)
    } else if args.is_empty() {
        git::actualizar_rama(&g, &dir, &git::ESPERAS_PUSH)
    } else {
        Informe::nuevo(orden, Resultado::Uso)
            .con("motivo", "argumentos-de-mas")
            .diagnostico("uso: actualizar-rama [--json]")
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

    #[test]
    fn guardar_sin_mensaje_o_sin_rutas_es_uso() {
        let s = ejecutar(["guardar"]);
        assert_eq!(
            (s.codigo, s.stdout.as_str()),
            (
                2,
                "guardar uso motivo=sin-mensaje
"
            )
        );
        let s = ejecutar(["guardar", "--json", "mensaje"]);
        assert_eq!(s.codigo, 2);
        assert!(
            s.stdout
                .starts_with("{\"comando\":\"guardar\",\"resultado\":\"uso\"")
        );
        let s = ejecutar(["actualizar-rama", "de-mas"]);
        assert_eq!(
            s.stdout,
            "actualizar-rama uso motivo=argumentos-de-mas
"
        );
    }
}
