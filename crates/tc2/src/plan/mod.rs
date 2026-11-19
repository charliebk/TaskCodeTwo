//! `tc2 plan` y `tc2 plan guardar` (F4-1, RR-02): la fase de diseno con
//! brainstorm de roles y unificador, proporcional al carril.
//!
//! - `plan [--json] TASK-NNN [--carril <c>]`: `fetch` (exige red), quien lo
//!   lanza (identidad), la configuracion de la base (`modo_flujo` y
//!   `politica_carriles`; sin base o sin configuracion, el suelo) y la
//!   operacion [`Planificar`] por el punto unico de escritura
//!   ([`estado::transaccion`], con el `ROADMAP.md` del mismo commit).
//!   Publica en la carpeta fija de la tarea, `TASK-NNN/planificacion/`: las
//!   peticiones de la ronda en `brainstorm/` y el esqueleto de
//!   `plan-final.md` si no existe; la tarea queda `en-diseno` con el carril
//!   y el modo congelados. Linea: `plan ok id= commit= ... carril= modo=
//!   ronda= roles= forma=redaccion|unificador peticiones=`. El CLI no llama
//!   a ningun modelo (como la 0.16): quien orquesta lanza los agentes.
//! - `plan guardar [--json] TASK-NNN <destino> <fichero>`: publica un
//!   fichero (de fuera del repo) como `plan-final.md` o como la salida de un
//!   rol ([`guardar`]). Destino y fichero se comprueban antes de leer el
//!   estado (`uso`).
//!
//! Lanzadas desde un worktree de tarea escriben el estado en el principal
//! (lo localiza la transaccion). El entorno es el de `start`
//! ([`crate::toma::Entorno`]).

pub mod guardar;
pub mod heuristica;
pub mod operacion;
pub mod plantillas;
pub mod roles;

use std::path::Path;

use crate::carril::Carril;
use crate::config::Config;
use crate::contrato::{Informe, Resultado, Valor};
use crate::estado::{self, Operacion};
use crate::git::{self, Git};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::plan as txt;
use crate::plataforma::errores::sanear;
use crate::toma::Entorno;

pub use guardar::{Guardar, destino_valido};
pub use operacion::{Forma, Planificar, Resumen};

/// Nombre de la orden y de la operacion de `plan` (y de su fila).
pub const NOMBRE: &str = "plan";

/// Subcomando que publica un fichero en la planificacion.
pub const GUARDAR: &str = "guardar";

fn uso(motivo: &str) -> Informe {
    Informe::nuevo(NOMBRE, Resultado::Uso)
        .con("motivo", motivo)
        .diagnostico(&txt::USO_PLAN)
}

/// `plan ...` o `plan guardar ...` (sin `--json`, que quita `lib.rs`). Nada
/// se lee ni se escribe antes de comprobar los argumentos.
pub fn orden(args: &[String], e: &Entorno<'_>) -> Informe {
    match args.split_first() {
        Some((g, resto)) if g == GUARDAR => orden_guardar(resto, e),
        _ => orden_plan(args, e),
    }
}

fn orden_plan(args: &[String], e: &Entorno<'_>) -> Informe {
    let Some((id, resto)) = args.split_first() else {
        return uso("sin-id");
    };
    if !git::es_id(id) {
        return uso("id-invalido");
    }
    let mut carril = None;
    let mut it = resto.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--carril" if carril.is_none() => match it.next().and_then(|c| Carril::desde(c)) {
                Some(c) => carril = Some(c),
                None => return uso("carril-invalido"),
            },
            _ => return uso("argumentos-de-mas"),
        }
    }
    plan_en(e, id, carril)
}

fn orden_guardar(args: &[String], e: &Entorno<'_>) -> Informe {
    let (id, destino, fichero) = match args {
        [] => return uso("sin-id"),
        [id, ..] if !git::es_id(id) => return uso("id-invalido"),
        [_] => return uso("sin-destino"),
        [_, _] => return uso("sin-fichero"),
        [id, d, f] => (id, d, f),
        _ => return uso("argumentos-de-mas"),
    };
    if destino_valido(destino).is_none() {
        let nombres: Vec<&str> = roles::todos().map(|r| r.id).collect();
        return Informe::nuevo(NOMBRE, Resultado::Uso)
            .con("motivo", "destino-invalido")
            .diagnostico(
                Mensaje::nuevo(&txt::DESTINO_INVALIDO)
                    .con("destino", sanear(destino))
                    .con("roles", nombres.join(", ")),
            );
    }
    let contenido = match leer_fichero(&e.dir.join(fichero)) {
        Ok(c) => c,
        Err(i) => return i,
    };
    guardar_en(e, id, destino, &contenido)
}

/// Lee el fichero a publicar sin cargar mas del tope: UTF-8, no vacio y de
/// [`guardar::MAX_BYTES`] como mucho. Un fallo es `uso` (como el fichero de
/// `import`).
fn leer_fichero(ruta: &Path) -> Result<String, Informe> {
    use std::io::Read as _;
    let mal = |motivo: &str, m: Mensaje| {
        Informe::nuevo(NOMBRE, Resultado::Uso)
            .con("motivo", motivo)
            .diagnostico(m.con("ruta", sanear(&ruta.display().to_string())))
    };
    let no_se_lee = |e: std::io::Error| {
        mal(
            "no-se-lee",
            Mensaje::nuevo(&txt::NO_SE_LEE).con("error", sanear(&e.to_string())),
        )
    };
    let f = std::fs::File::open(ruta).map_err(no_se_lee)?;
    let tope = u64::try_from(guardar::MAX_BYTES).unwrap_or(u64::MAX);
    let mut bytes = Vec::new();
    f.take(tope.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(no_se_lee)?;
    if bytes.len() > guardar::MAX_BYTES {
        return Err(mal(
            "grande",
            Mensaje::nuevo(&txt::GRANDE).con("max", guardar::MAX_BYTES.to_string()),
        ));
    }
    let texto =
        String::from_utf8(bytes).map_err(|_| mal("no-utf8", Mensaje::nuevo(&txt::NO_UTF8)))?;
    if texto.trim().is_empty() {
        return Err(mal("vacio", Mensaje::nuevo(&txt::VACIO)));
    }
    Ok(texto)
}

/// `plan` de la tarea `id` con el carril pedido en el repo de `e`.
pub fn plan_en(e: &Entorno<'_>, id: &str, carril: Option<Carril>) -> Informe {
    let i = match paso_plan(e, id, carril) {
        Ok(i) | Err(i) => i,
    };
    de_la_orden(i, id)
}

/// `plan guardar` de `contenido` en `destino` (ya validado) de la tarea
/// `id` en el repo de `e`.
pub fn guardar_en(e: &Entorno<'_>, id: &str, destino: &str, contenido: &str) -> Informe {
    let i = match paso_guardar(e, id, destino, contenido) {
        Ok(i) | Err(i) => i,
    };
    let mut i = de_la_orden(i, id);
    i.claves
        .insert(1, ("destino".to_owned(), Valor::Texto(destino.to_owned())));
    i
}

/// `fetch` si hay remoto: `plan` y `plan guardar` exigen red.
fn traer(g: &Git, dir: &Path) -> Result<(), Informe> {
    let principal = git::localizar_principal(g, dir)?;
    if git::estado::hay_remoto(g, &principal, NOMBRE)? {
        let f = git::fetch(g, &principal);
        if !f.resultado.es_exito() {
            return Err(f);
        }
    }
    Ok(())
}

/// La configuracion de la base (F1-10 decision 3). Sin base o sin
/// configuracion, el suelo; una que no vale falla con su informe a nombre
/// de `plan` (como `ciclo::umbral_de_inactividad`).
pub fn config_de_base(g: &Git, dir: &Path) -> Result<Config, Informe> {
    match crate::config::de_base(g, dir) {
        Ok(c) => Ok(c),
        Err(i) if matches!(i.valor("motivo"), Some("sin-base" | "sin-config")) => {
            Ok(Config::suelo())
        }
        Err(mut i) => {
            NOMBRE.clone_into(&mut i.comando);
            Err(i)
        }
    }
}

fn paso_plan(e: &Entorno<'_>, id: &str, carril: Option<Carril>) -> Result<Informe, Informe> {
    traer(e.git, e.dir)?;
    let quien = (e.quien)()?;
    let config = config_de_base(e.git, e.dir)?;
    let op = Planificar::nueva(id, &quien, carril, &config, e.ahora);
    let t = escribir(e, &op);
    match t.resultado {
        Resultado::Ok | Resultado::SinCambios => Ok(con_resumen(t, op.resumen())),
        _ => Err(con_dueno(t, op.dueno())),
    }
}

fn paso_guardar(
    e: &Entorno<'_>,
    id: &str,
    destino: &str,
    contenido: &str,
) -> Result<Informe, Informe> {
    traer(e.git, e.dir)?;
    let quien = (e.quien)()?;
    let Some(op) = Guardar::nueva(id, &quien, destino, contenido) else {
        return Err(uso("destino-invalido"));
    };
    let t = escribir(e, &op);
    match t.resultado {
        Resultado::Ok | Resultado::SinCambios => Ok(t),
        _ => Err(con_dueno(t, op.dueno())),
    }
}

/// La operacion por el punto unico de escritura, con el `ROADMAP.md`
/// regenerado en el mismo commit.
fn escribir(e: &Entorno<'_>, op: &dyn Operacion) -> Informe {
    let personas = crate::roadmap::personas(e.git, e.dir);
    let generar = crate::roadmap::generador(&personas);
    let ctx = estado::Contexto::nuevo(e.git, e.dir, e.catalogo, Some(&generar));
    estado::transaccion(&ctx, op)
}

/// Las claves de lo que decidio `plan`.
fn con_resumen(i: Informe, r: Option<Resumen>) -> Informe {
    let Some(r) = r else {
        return i;
    };
    i.con("carril", r.carril.nombre())
        .con("modo", r.modo)
        .con("ronda", r.ronda.to_string())
        .con_lista("roles", r.roles.iter().map(|x| (*x).to_owned()).collect())
        .con("forma", r.forma.nombre())
        .con_lista("peticiones", r.peticiones)
}

/// Un rechazo con lo que sabe la operacion: quien la lleva o el arreglo de
/// una tarea de la 0.16.
fn con_dueno(mut i: Informe, dueno: Option<String>) -> Informe {
    if i.resultado != Resultado::Rechazada {
        return i;
    }
    match (i.valor("motivo"), dueno) {
        (Some("de-otra-persona"), Some(l)) => i = i.con("asignado", l),
        (Some("formato-1"), _) => i = i.con("arreglo", "tc2 actualizar"),
        _ => {}
    }
    i
}

/// El informe como de la orden: el comando, `id=` el primero y sin la clave
/// `operacion` de la transaccion. Un `sinred` sin `pendiente` dice
/// `pendiente=no` (exige red).
fn de_la_orden(mut i: Informe, id: &str) -> Informe {
    NOMBRE.clone_into(&mut i.comando);
    i.claves.retain(|(k, _)| k != "operacion" && k != "id");
    i.claves
        .insert(0, ("id".to_owned(), Valor::Texto(id.to_owned())));
    if i.resultado == Resultado::Sinred && i.valor("pendiente").is_none() {
        i = i.con("pendiente", "no");
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Idioma;

    /// Carpeta temporal propia del test: se borra al acabar.
    struct Carpeta(std::path::PathBuf);

    impl Drop for Carpeta {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// El uso no anuncia `--carril errata`: `plan` nunca la acepta (todo
    /// tipo declara fix o funcionalidad y el carril solo sube).
    #[test]
    fn el_uso_no_anuncia_errata() {
        for t in [txt::USO_PLAN.es, txt::USO_PLAN.en] {
            assert!(!t.contains("errata"), "{t}");
            assert!(t.contains("--carril fix|funcionalidad|riesgo-alto"), "{t}");
        }
    }

    /// `plan guardar` no publica un fichero vacio, que no es UTF-8 o de mas
    /// de 1 MiB (`uso`); justo en el tope, si. El diagnostico va saneado:
    /// una ruta con un salto de linea queda en una sola linea.
    #[test]
    fn fichero_vacio_no_utf8_grande_y_ruta_saneada() -> Result<(), std::io::Error> {
        let raiz = std::env::temp_dir()
            .join("tc2-tests")
            .join(format!("plan-fichero-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&raiz);
        std::fs::create_dir_all(&raiz)?;
        let _borrar = Carpeta(raiz.clone());
        let tope = guardar::MAX_BYTES;
        for (nombre, bytes, motivo) in [
            ("vacio.md", b" \n\t\n".to_vec(), Some("vacio")),
            ("binario.md", vec![b'a', 0xff, 0xfe], Some("no-utf8")),
            ("grande.md", vec![b'a'; tope + 1], Some("grande")),
            ("justo.md", vec![b'a'; tope], None),
        ] {
            let ruta = raiz.join(nombre);
            std::fs::write(&ruta, bytes)?;
            let r = leer_fichero(&ruta);
            assert_eq!(
                r.as_ref().err().and_then(|i| i.valor("motivo")),
                motivo,
                "{nombre}"
            );
            if let Err(i) = r {
                assert_eq!(i.resultado, Resultado::Uso, "{nombre}");
            }
        }
        let i = leer_fichero(&raiz.join("no\nexiste.md")).err();
        assert_eq!(
            i.as_ref().and_then(|i| i.valor("motivo")),
            Some("no-se-lee")
        );
        let d = i
            .and_then(|i| i.diagnostico)
            .map(|m| m.texto(Idioma::Es))
            .unwrap_or_default();
        assert!(d.contains("no existe.md") && !d.contains('\n'), "{d}");
        Ok(())
    }
}
