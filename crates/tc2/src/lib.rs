//! Nucleo del CLI `tc2` de TaskCodeTwo.
//!
//! Modulos: [`contrato`] (salida de toda operacion, ADR F1-4), [`git`]
//! (capa de operaciones Git deterministas, F2-2) y [`estado`] (rama de
//! estado `taskcode/tareas`, F2-3), [`alta`] (alta de tareas con reserva de
//! ID, F2-4) [`tarea`] (modelo de tarea y frontmatter YAML, F2-5) y
//! [`plataforma`] (puerto Plataforma y adaptadores GitHub y GitLab, F2-6 y
//! F2-9), [`carril`] (tabla de suelo de F1-10) y [`config`] (configuracion
//! con esquema validado, F2-7) e [`i18n`] (mensajes en espanol e ingles,
//! F2-8), [`version`] (`version --json` y `compatible`, F2-12) y
//! [`actualizar_cli`] (`actualizar --cli`, F2-12). Las capas
//! que vienen entran como modulos de esta biblioteca.

pub mod actualizar_cli;
pub mod alta;
pub mod carril;
pub mod config;
pub mod contrato;
pub mod descarga;
pub mod estado;
pub mod git;
pub mod i18n;
pub mod identidad;
pub mod plataforma;
pub mod tarea;
pub mod version;
mod yaml;

pub use version::PROTOCOLO;

use contrato::{Informe, Resultado};
use i18n::catalogo::cli as txt;
use i18n::{Idioma, Mensaje};

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

/// Interpreta los argumentos (sin el nombre del programa) y ejecuta la orden
/// en la carpeta actual, con los mensajes en espanol.
pub fn ejecutar<I, S>(args: I) -> Salida
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    ejecutar_en(Idioma::Es, args)
}

/// Como [`ejecutar`], con los mensajes dirigidos a la persona en `idioma`
/// (F2-8). stdout y el codigo de salida no cambian con el idioma.
pub fn ejecutar_en<I, S>(idioma: Idioma, args: I) -> Salida
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
            orden_git(orden, resto).salida_en(json, idioma)
        }
        [orden, resto @ ..] if orden == alta::NOMBRE => {
            let (json, resto) = match resto {
                [j, r @ ..] if j == "--json" => (true, r),
                r => (false, r),
            };
            let mut parecidas = Vec::new();
            let mut s = orden_new(resto, &mut parecidas).salida_en(json, idioma);
            // El aviso de titulos parecidos va a stderr con codigo 0: es la
            // unica excepcion a «diagnostico solo si falla» (F2-4b, D1).
            if s.codigo == 0
                && let Some(a) = alta::aviso_parecidas(&parecidas, idioma)
            {
                s.stderr.push_str(&a);
                s.stderr.push('\n');
            }
            s
        }
        [orden, resto @ ..] if orden == version::VERSION => {
            let (json, resto) = con_json(resto);
            orden_version(json, &resto, idioma)
        }
        [orden, resto @ ..] if orden == version::COMPATIBLE => {
            let (json, resto) = con_json(resto);
            version::compatible(&resto).salida_en(json, idioma)
        }
        [orden, resto @ ..] if orden == actualizar_cli::NOMBRE => {
            let (json, resto) = con_json(resto);
            actualizar_cli::orden(&resto).salida_en(json, idioma)
        }
        [] => Salida {
            codigo: 2,
            stdout: String::new(),
            stderr: format!(
                "{}
",
                Mensaje::nuevo(&txt::FALTA_LA_ORDEN)
                    .con_mensaje("uso", &txt::USO)
                    .texto(idioma)
            ),
        },
        [otro, ..] => Salida {
            codigo: 2,
            stdout: String::new(),
            stderr: format!(
                "{}
",
                Mensaje::nuevo(&txt::ORDEN_DESCONOCIDA)
                    .con("orden", otro.as_str())
                    .con_mensaje("uso", &txt::USO)
                    .texto(idioma)
            ),
        },
    }
}

/// Quita `--json` de donde este y dice si estaba.
fn con_json(args: &[String]) -> (bool, Vec<String>) {
    let json = args.iter().any(|a| a == "--json");
    (
        json,
        args.iter().filter(|a| *a != "--json").cloned().collect(),
    )
}

/// `version [--json]`: con `--json`, exactamente `{version, protocolo,
/// target, sha256}` (ADR F1-14, punto 5); sin el, la linea de contrato.
fn orden_version(json: bool, args: &[String], idioma: Idioma) -> Salida {
    if !args.is_empty() {
        return Informe::nuevo(version::VERSION, Resultado::Uso)
            .con("motivo", "argumentos-de-mas")
            .diagnostico(&i18n::catalogo::actualizar::USO_VERSION)
            .salida_en(json, idioma);
    }
    let datos = std::env::current_exe()
        .map_err(|e| {
            Informe::nuevo(version::VERSION, Resultado::Error).diagnostico(
                Mensaje::nuevo(&i18n::catalogo::actualizar::NO_SE_LEE_EL_EJECUTABLE)
                    .con("ruta", "current_exe")
                    .con("error", e.to_string()),
            )
        })
        .and_then(|exe| version::datos(&exe));
    match datos {
        Ok(d) if json => Salida {
            codigo: 0,
            stdout: format!("{}\n", d.json()),
            stderr: String::new(),
        },
        Ok(d) => version::informe(&d).salida_en(false, idioma),
        Err(i) => i.salida_en(json, idioma),
    }
}

/// Las dos operaciones Git publicas (ADR F1-4 D1a), en la carpeta actual.
fn orden_git(orden: &str, args: &[String]) -> Informe {
    let dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            return Informe::nuevo(orden, Resultado::Error)
                .diagnostico(Mensaje::nuevo(&txt::CARPETA_ACTUAL).con("error", e.to_string()));
        }
    };
    let g = git::Git::nuevo();
    if orden == "guardar" {
        let Some((mensaje, rutas)) = args.split_first() else {
            return Informe::nuevo(orden, Resultado::Uso)
                .con("motivo", "sin-mensaje")
                .diagnostico(&txt::USO_GUARDAR);
        };
        let rutas: Vec<&str> = rutas.iter().map(String::as_str).collect();
        git::guardar(&g, &dir, mensaje, &rutas)
    } else if args.is_empty() {
        git::actualizar_rama(&g, &dir, &git::ESPERAS_PUSH)
    } else {
        Informe::nuevo(orden, Resultado::Uso)
            .con("motivo", "argumentos-de-mas")
            .diagnostico(&txt::USO_ACTUALIZAR_RAMA)
    }
}

/// Catalogo de produccion: reconstruye las operaciones de la rama de estado
/// desde su anotacion. Las operaciones nuevas se anaden aqui.
fn catalogo(anotacion: &str) -> Option<Box<dyn estado::Operacion>> {
    alta::desde_anotacion(anotacion)
}

/// `new "<titulo>"`: alta de una tarea con su ID reservado por el push de la
/// rama de estado (F2-4). Sale `new ok id=TASK-NNN rama=... [parecidas=...]`;
/// las tareas abiertas parecidas quedan ademas en `parecidas` para el aviso.
fn orden_new(args: &[String], parecidas: &mut Vec<alta::Parecida>) -> Informe {
    let titulo = match args {
        [t] if alta::titulo_valido(t) => t,
        [_] => {
            return Informe::nuevo(alta::NOMBRE, Resultado::Uso)
                .con("motivo", "titulo-invalido")
                .diagnostico(&txt::TITULO_INVALIDO);
        }
        [] => {
            return Informe::nuevo(alta::NOMBRE, Resultado::Uso)
                .con("motivo", "sin-titulo")
                .diagnostico(&txt::USO_NEW);
        }
        _ => {
            return Informe::nuevo(alta::NOMBRE, Resultado::Uso)
                .con("motivo", "argumentos-de-mas")
                .diagnostico(&txt::TITULO_ENTRE_COMILLAS);
        }
    };
    let dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            return Informe::nuevo(alta::NOMBRE, Resultado::Error)
                .diagnostico(Mensaje::nuevo(&txt::CARPETA_ACTUAL).con("error", e.to_string()));
        }
    };
    let g = git::Git::nuevo();
    // Las ramas `*/task-*` se leen una vez, antes de la transaccion: el ID
    // nuevo queda por encima de todas (F2-4b). Sin red no se reserva nada.
    let ocupados = match git::ids_con_rama(&g, &dir, alta::NOMBRE) {
        Ok(o) => o,
        Err(mut i) => {
            alta::NOMBRE.clone_into(&mut i.comando);
            if i.resultado == Resultado::Sinred {
                i = i.con("pendiente", "no");
            }
            return i;
        }
    };
    let op = alta::Alta::nueva(titulo).con_ocupados(ocupados);
    let c = estado::Contexto::nuevo(&g, &dir, &catalogo);
    let mut i = estado::transaccion(&c, &op);
    alta::NOMBRE.clone_into(&mut i.comando);
    if i.resultado == Resultado::Ok
        && let (Some(id), Some(rama)) = (op.id(), op.rama())
    {
        i.claves
            .insert(0, ("id".to_owned(), contrato::Valor::Texto(id)));
        i.claves
            .insert(1, ("rama".to_owned(), contrato::Valor::Texto(rama)));
        *parecidas = op.parecidas();
        if !parecidas.is_empty() {
            let ids = parecidas.iter().map(|p| p.id.clone()).collect();
            i.claves
                .insert(2, ("parecidas".to_owned(), contrato::Valor::Lista(ids)));
        }
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

    /// TASK-027, criterio 4: `version --json` da exactamente las cuatro
    /// claves del ADR F1-14 (punto 5), con el sha256 del ejecutable.
    #[test]
    fn version_json_exactamente_version_protocolo_target_sha256() {
        let s = ejecutar(["version", "--json"]);
        assert_eq!((s.codigo, s.stderr.as_str()), (0, ""));
        let v: serde_json::Value = serde_json::from_str(&s.stdout).unwrap_or_default();
        let claves: Vec<&str> = v
            .as_object()
            .map(|o| o.keys().map(String::as_str).collect())
            .unwrap_or_default();
        let mut ordenadas = claves.clone();
        ordenadas.sort_unstable();
        assert_eq!(ordenadas, ["protocolo", "sha256", "target", "version"]);
        assert_eq!(v["version"], VERSION);
        assert_eq!(v["protocolo"], PROTOCOLO);
        let alias = distribucion::Target::actual().map(|t| t.alias);
        assert_eq!(v["target"].as_str(), alias);
        let exe = std::env::current_exe().unwrap_or_default();
        let bytes = std::fs::read(exe).unwrap_or_default();
        assert_eq!(v["sha256"], distribucion::sha256::hex(&bytes));
        assert_eq!(s.stdout.lines().count(), 1);
        // Sin --json, la linea de contrato con las mismas claves.
        let l = ejecutar(["version"]);
        assert_eq!(l.codigo, 0);
        assert!(l.stdout.starts_with(&format!(
            "version ok version={VERSION} protocolo={PROTOCOLO} target="
        )));
        assert_eq!(ejecutar(["version", "de-mas"]).codigo, 2);
    }

    /// TASK-027, criterio 5: un rango que no incluye el protocolo sale
    /// `rechazada` (3) con el motivo y el arreglo, en los dos idiomas.
    #[test]
    fn compatible_fuera_de_rango_es_rechazada_con_el_arreglo() {
        let fuera = format!("{}-{}", PROTOCOLO + 1, PROTOCOLO + 2);
        let es = ejecutar_en(Idioma::Es, ["compatible", "--protocolo", &fuera]);
        let en = ejecutar_en(Idioma::En, ["compatible", "--protocolo", &fuera]);
        for s in [&es, &en] {
            assert_eq!(s.codigo, 3);
            assert_eq!(
                s.stdout,
                format!(
                    "compatible rechazada protocolo={PROTOCOLO} rango={fuera} motivo=protocolo-incompatible arreglo=tc2%20actualizar%20--cli\n"
                )
            );
            assert!(
                s.stderr
                    .contains("motivo=protocolo-incompatible arreglo=\"tc2 actualizar --cli\""),
                "{}",
                s.stderr
            );
            assert_eq!(s.stderr.lines().count(), 1);
        }
        assert!(es.stderr.starts_with("este tc2 habla el protocolo"));
        assert!(en.stderr.starts_with("this tc2 speaks protocol"));
        // En JSON el arreglo va tal cual.
        let j = ejecutar_en(Idioma::En, ["compatible", "--json", "--protocolo", &fuera]);
        assert_eq!(j.codigo, 3);
        assert!(j.stdout.contains("\"arreglo\":\"tc2 actualizar --cli\""));
        // Dentro del rango, ok; mal escrito, uso.
        let dentro = format!("1-{PROTOCOLO}");
        let ok = ejecutar(["compatible", "--protocolo", &dentro]);
        assert_eq!(
            (ok.codigo, ok.stdout),
            (
                0,
                format!("compatible ok protocolo={PROTOCOLO} rango={dentro}\n")
            )
        );
        for args in [
            vec!["compatible"],
            vec!["compatible", "--protocolo"],
            vec!["compatible", "--protocolo", "3-1"],
            vec!["compatible", "--protocolo", "x"],
        ] {
            assert_eq!(ejecutar(&args).codigo, 2, "{args:?}");
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

    #[test]
    fn new_y_actualizar_rama_en_ingles_mismo_stdout_y_codigo() {
        for args in [
            vec!["new"],
            vec!["new", "a\tb"],
            vec!["new", "--json", "a", "b"],
            vec!["actualizar-rama", "de-mas"],
            vec!["guardar"],
        ] {
            let es = ejecutar_en(Idioma::Es, &args);
            let en = ejecutar_en(Idioma::En, &args);
            assert_eq!(es.codigo, en.codigo, "{args:?}");
            assert!(!en.stderr.is_empty(), "{args:?}");
            assert_ne!(es.stderr, en.stderr, "{args:?}: stderr sin traducir");
            if args.contains(&"--json") {
                assert!(en.stdout.contains("\"diagnostico\":"), "{args:?}");
            } else {
                assert_eq!(es.stdout, en.stdout, "{args:?}");
            }
        }
        assert_eq!(
            ejecutar_en(Idioma::En, ["new"]).stderr,
            "usage: new [--json] \"<title>\"\n"
        );
    }
}
