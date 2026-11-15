//! Nucleo del CLI `tc2` de TaskCodeTwo.
//!
//! Modulos: [`contrato`] (salida de toda operacion, ADR F1-4), [`git`]
//! (capa de operaciones Git deterministas, F2-2) y [`estado`] (rama de
//! estado `taskcode/tareas`, F2-3), [`alta`] (alta de tareas con reserva de
//! ID, F2-4) [`tarea`] (modelo de tarea y frontmatter YAML, F2-5) y
//! [`plataforma`] (puerto Plataforma y adaptadores GitHub y GitLab, F2-6 y
//! F2-9), [`carril`] (tabla de suelo de F1-10) y [`config`] (configuracion
//! con esquema validado, F2-7). Las capas
//! que vienen entran como modulos de esta biblioteca.

pub mod alta;
pub mod carril;
pub mod config;
pub mod contrato;
pub mod estado;
pub mod git;
pub mod plataforma;
pub mod tarea;
mod yaml;

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
const USO: &str = "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\"";

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
        [orden, resto @ ..] if orden == alta::NOMBRE => {
            let (json, resto) = match resto {
                [j, r @ ..] if j == "--json" => (true, r),
                r => (false, r),
            };
            orden_new(resto).salida(json)
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

/// Catalogo de produccion: reconstruye las operaciones de la rama de estado
/// desde su anotacion. Las operaciones nuevas se anaden aqui.
fn catalogo(anotacion: &str) -> Option<Box<dyn estado::Operacion>> {
    alta::desde_anotacion(anotacion)
}

/// `new "<titulo>"`: alta de una tarea con su ID reservado por el push de la
/// rama de estado (F2-4). Sale `new ok id=TASK-NNN ...`.
fn orden_new(args: &[String]) -> Informe {
    let titulo = match args {
        [t] if alta::titulo_valido(t) => t,
        [_] => {
            return Informe::nuevo(alta::NOMBRE, Resultado::Uso)
                .con("motivo", "titulo-invalido")
                .diagnostico(
                    "el titulo no puede estar vacio ni llevar saltos de linea, tabuladores ni otros caracteres de control",
                );
        }
        [] => {
            return Informe::nuevo(alta::NOMBRE, Resultado::Uso)
                .con("motivo", "sin-titulo")
                .diagnostico("uso: new [--json] \"<titulo>\"");
        }
        _ => {
            return Informe::nuevo(alta::NOMBRE, Resultado::Uso)
                .con("motivo", "argumentos-de-mas")
                .diagnostico("pon el titulo entre comillas: new [--json] \"<titulo>\"");
        }
    };
    let dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            return Informe::nuevo(alta::NOMBRE, Resultado::Error)
                .diagnostico(format!("no se puede leer la carpeta actual: {e}"));
        }
    };
    let g = git::Git::nuevo();
    let op = alta::Alta::nueva(titulo);
    let c = estado::Contexto::nuevo(&g, &dir, &catalogo);
    let mut i = estado::transaccion(&c, &op);
    alta::NOMBRE.clone_into(&mut i.comando);
    if i.resultado == Resultado::Ok
        && let Some(id) = op.id()
    {
        i.claves
            .insert(0, ("id".to_owned(), contrato::Valor::Texto(id)));
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sin_titulo_o_con_salto_de_linea_es_uso() {
        for (args, motivo) in [
            (vec!["new"], "sin-titulo"),
            (vec!["new", "--json"], "sin-titulo"),
            (vec!["new", "a", "b"], "argumentos-de-mas"),
            (vec!["new", "a\tb"], "titulo-invalido"),
            (vec!["new", "linea\notra"], "titulo-invalido"),
            (vec!["new", "  "], "titulo-invalido"),
        ] {
            let s = ejecutar(&args);
            assert_eq!(s.codigo, 2, "{args:?}");
            assert!(s.stdout.contains(motivo), "{args:?}: {}", s.stdout);
        }
    }

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
