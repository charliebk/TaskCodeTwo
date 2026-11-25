//! `tc2 plan` y `tc2 plan guardar` (F4-1, RR-02): la fase de diseno con
//! brainstorm de roles y unificador, proporcional al carril.
//!
//! - `plan [--json] TASK-NNN [--carril <c>]`: `fetch` (exige red), quien lo
//!   lanza (identidad), la configuracion de la base (`modo_flujo` y
//!   `politica_carriles`; sin base o sin configuracion, el suelo) y la
//!   operacion [`Planificar`] por el punto unico de escritura
//!   ([`crate::estado::transaccion`], con el `ROADMAP.md` del mismo commit).
//!   Publica en la carpeta fija de la tarea, `TASK-NNN/planificacion/`: las
//!   peticiones de la ronda en `brainstorm/` y el esqueleto de
//!   `plan-final.md` si no existe; la tarea queda `en-diseno` con el carril
//!   y el modo congelados. Linea: `plan ok id= commit= ... carril= modo=
//!   ronda= roles= forma=redaccion|unificador peticiones= dominios=
//!   modelo-diseno= modelo-implementacion= avisos=` (F4-6: la seleccion de
//!   diseno e implementacion, anotada en `roles.diseno` y
//!   `roles.implementacion`). El CLI no llama
//!   a ningun modelo (como la 0.16): quien orquesta lanza los agentes.
//! - `plan guardar [--json] TASK-NNN <destino> <fichero>`: publica un
//!   fichero (de fuera del repo) como `plan-final.md` o como la salida de un
//!   rol ([`guardar`]). Destino y fichero se comprueban antes de leer el
//!   estado (`uso`).
//!
//! - `approve [--json] TASK-NNN` (F4-2, RR-04): solo una persona (con
//!   terminal y token verificado: [`crate::identidad::resolver`]); con el
//!   agente o sin identidad, `rechazada motivo=identidad rol=persona` antes
//!   de leer nada. Luego `fetch`, la configuracion y la operacion
//!   [`Aprobar`], cuyo commit va firmado con la clave de la persona y con el
//!   trailer `Plan-Blob`; si la firma falla, `rechazada motivo=firma` sin
//!   publicar nada. `--decidido-por` no existe (F1-10 decision 6): `uso`.
//! - `approve [--json] TASK-NNN --pedir-cambios <fichero>`: publica
//!   `brainstorm/cambios-pedidos-<n>.md` en la ronda mayor y la cierra
//!   ([`PedirCambios`], D1a); la puede lanzar tambien el agente.
//! - `approve [--json] --hito <n>` (F4-3): como `approve` (persona primero,
//!   `fetch`, configuracion), con [`aprobar::AprobarHito`]: los planes de
//!   las tareas abiertas del sprint `n` (`4` o `H4`) en un unico commit
//!   firmado, todo o nada. Linea: `approve ok hito= commit= ...
//!   aprobadas=`; si alguna no esta lista, `rechazada motivo=no-listas
//!   tareas=ID:motivo,...`; tambien `hito-vacio` y `demasiadas max=20`. Con
//!   un ID, con `--pedir-cambios` o con un hito que no es un numero, `uso`.
//!
//! Lanzadas desde un worktree de tarea escriben el estado en el principal
//! (lo localiza la transaccion). El entorno es el de `start`
//! ([`crate::toma::Entorno`]).

pub mod aprobar;
pub mod guardar;
pub mod heuristica;
pub mod operacion;
pub mod plantillas;
pub mod roles;

use std::path::Path;

use crate::carril::Carril;
use crate::config::Config;
use crate::contrato::{Informe, Resultado, Valor};
use crate::estado::{como_orden, con_dueno};
use crate::git::{self, Git};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::plan as txt;
use crate::identidad::Quien;
use crate::plataforma::errores::sanear;
use crate::toma::Entorno;

pub use aprobar::{Aprobar, AprobarHito, PedirCambios};
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

/// Lee el fichero a publicar con [`crate::lectura::fichero`]: UTF-8, no
/// vacio y de [`crate::lectura::MAX_BYTES`] como mucho. Un fallo es `uso`
/// (como el fichero de `import`).
fn leer_fichero(ruta: &Path) -> Result<String, Informe> {
    use crate::lectura::{self, Motivo};
    lectura::fichero(ruta, lectura::MAX_BYTES).map_err(|n| {
        let (motivo, m) = match n.motivo {
            Motivo::NoSeLee => (
                "no-se-lee",
                Mensaje::nuevo(&txt::NO_SE_LEE).con("error", n.error),
            ),
            Motivo::Grande => (
                "grande",
                Mensaje::nuevo(&txt::GRANDE).con("max", lectura::MAX_BYTES.to_string()),
            ),
            Motivo::NoUtf8 => ("no-utf8", Mensaje::nuevo(&txt::NO_UTF8)),
            Motivo::Vacio => ("vacio", Mensaje::nuevo(&txt::VACIO)),
        };
        Informe::nuevo(NOMBRE, Resultado::Uso)
            .con("motivo", motivo)
            .diagnostico(m.con("ruta", n.ruta))
    })
}

/// `plan` de la tarea `id` con el carril pedido en el repo de `e`.
pub fn plan_en(e: &Entorno<'_>, id: &str, carril: Option<Carril>) -> Informe {
    let i = match paso_plan(e, id, carril) {
        Ok(i) | Err(i) => i,
    };
    como_orden(i, NOMBRE, "id", id)
}

fn uso_approve(motivo: &str) -> Informe {
    Informe::nuevo(aprobar::NOMBRE, Resultado::Uso)
        .con("motivo", motivo)
        .diagnostico(&txt::USO_APPROVE)
}

/// `approve TASK-NNN [--pedir-cambios <fichero>]` (sin `--json`, que quita
/// `lib.rs`). Nada se lee ni se escribe antes de comprobar los argumentos.
pub fn orden_approve(args: &[String], e: &Entorno<'_>) -> Informe {
    let Some((id, resto)) = args.split_first() else {
        return uso_approve("sin-id");
    };
    if id == HITO {
        return orden_approve_hito(resto, e);
    }
    if !git::es_id(id) {
        return uso_approve("id-invalido");
    }
    let mut fichero = None;
    let mut it = resto.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            // F1-10 decision 6: la parada `approve` firmado es fija.
            x if es_decidido_por(x) => return decidido_por(),
            // Un hito o una tarea, no las dos cosas.
            x if x == HITO || x.starts_with("--hito=") => return uso_approve("hito-con-id"),
            "--pedir-cambios" if fichero.is_none() => match it.next() {
                Some(f) => fichero = Some(f),
                None => return uso_approve("sin-fichero"),
            },
            _ => return uso_approve("argumentos-de-mas"),
        }
    }
    match fichero {
        None => aprobar_en(e, id),
        Some(f) => match leer_fichero(&e.dir.join(f)) {
            Ok(c) => pedir_cambios_en(e, id, &c),
            Err(mut i) => {
                aprobar::NOMBRE.clone_into(&mut i.comando);
                i
            }
        },
    }
}

/// Opcion de `approve` que aprueba un hito entero (F4-3).
const HITO: &str = "--hito";

fn es_decidido_por(a: &str) -> bool {
    a == "--decidido-por" || a.starts_with("--decidido-por=")
}

fn decidido_por() -> Informe {
    Informe::nuevo(aprobar::NOMBRE, Resultado::Uso)
        .con("motivo", "decidido-por")
        .diagnostico(&txt::DECIDIDO_POR)
}

/// El numero de hito de `--hito`: `4` o `H4` (como lo escribe el
/// roadmap).
fn numero_de_hito(a: &str) -> Option<u32> {
    let cifras = a.strip_prefix(['H', 'h']).unwrap_or(a);
    if cifras.is_empty() || !cifras.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    cifras.parse().ok()
}

/// `approve --hito <n>`: sin ID ni `--pedir-cambios` (`uso`).
fn orden_approve_hito(args: &[String], e: &Entorno<'_>) -> Informe {
    let Some((n, resto)) = args.split_first() else {
        return uso_approve("sin-hito");
    };
    let Some(hito) = numero_de_hito(n) else {
        return uso_approve("hito-invalido");
    };
    match resto.first().map(String::as_str) {
        None => aprobar_hito_en(e, hito),
        Some(x) if es_decidido_por(x) => decidido_por(),
        Some(x) if git::es_id(x) => uso_approve("hito-con-id"),
        Some("--pedir-cambios") => uso_approve("hito-con-pedir-cambios"),
        Some(_) => uso_approve("argumentos-de-mas"),
    }
}

/// `approve --hito <hito>` en el repo de `e` (F4-3): la linea lleva
/// `hito=` primero, `aprobadas=` con el lote y, si alguna tarea no esta
/// lista, `tareas=ID:motivo,...`.
pub fn aprobar_hito_en(e: &Entorno<'_>, hito: u32) -> Informe {
    let i = match paso_aprobar_hito(e, hito) {
        Ok(i) | Err(i) => i,
    };
    como_orden(i, aprobar::NOMBRE, "hito", &hito.to_string())
}

fn paso_aprobar_hito(e: &Entorno<'_>, hito: u32) -> Result<Informe, Informe> {
    let (op, config) = persona_y_config(e, |q| aprobar::AprobarHito::nueva(hito, q, e.ahora))?;
    let op = op.con_config(&config);
    let t = e.escribir(&op);
    match t.resultado {
        Resultado::Ok => Ok(t.con_lista("aprobadas", op.aprobadas())),
        Resultado::SinCambios => Ok(t),
        Resultado::Rechazada => {
            let t = match t.valor("motivo") {
                Some("no-listas") => t.con_lista(
                    "tareas",
                    op.fallos()
                        .into_iter()
                        .map(|(id, m)| format!("{id}:{m}"))
                        .collect(),
                ),
                Some("demasiadas") => t.con("max", aprobar::MAX_HITO.to_string()),
                _ => t,
            };
            Err(t)
        }
        _ => Err(t),
    }
}

/// `approve` de la tarea `id` en el repo de `e`.
pub fn aprobar_en(e: &Entorno<'_>, id: &str) -> Informe {
    let i = match paso_aprobar(e, id) {
        Ok(i) | Err(i) => i,
    };
    como_orden(i, aprobar::NOMBRE, "id", id)
}

/// `approve --pedir-cambios` de `contenido` en la tarea `id` del repo de
/// `e`.
pub fn pedir_cambios_en(e: &Entorno<'_>, id: &str, contenido: &str) -> Informe {
    let i = match paso_pedir_cambios(e, id, contenido) {
        Ok(i) | Err(i) => i,
    };
    como_orden(i, aprobar::NOMBRE, "id", id)
}

/// `rechazada motivo=identidad rol=persona` (RR-04), con lo que diga el
/// rechazo de la identidad si lo hubo (`cache=`, `cuenta=`...).
fn sin_persona(rechazo: Option<Informe>) -> Informe {
    let mut i = Informe::nuevo(aprobar::NOMBRE, Resultado::Rechazada)
        .con("motivo", "identidad")
        .con("rol", "persona")
        .diagnostico(&txt::APROBAR_SIN_PERSONA);
    for (k, v) in rechazo.map(|r| r.claves).unwrap_or_default() {
        if k != "motivo" && k != "rol" {
            i.claves.push((k, v));
        }
    }
    i
}

/// El arranque comun de `approve` y `approve --hito`: lo primero, quien (el
/// agente, o sin identidad, no llega a leer ni a escribir nada:
/// `rechazada motivo=identidad rol=persona`); con la operacion que da
/// `nueva` para esa persona, `fetch` y la configuracion de la base.
fn persona_y_config<O>(
    e: &Entorno<'_>,
    nueva: impl FnOnce(&Quien) -> Option<O>,
) -> Result<(O, Config), Informe> {
    let quien = match (e.quien)() {
        Ok(q) => q,
        Err(i) if i.resultado == Resultado::Rechazada => return Err(sin_persona(Some(i))),
        Err(i) => return Err(i),
    };
    let Some(op) = nueva(&quien) else {
        return Err(sin_persona(None));
    };
    traer(e.git, e.dir)?;
    Ok((op, config_de_base(e.git, e.dir)?))
}

fn paso_aprobar(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
    let (op, config) = persona_y_config(e, |q| Aprobar::nueva(id, q, e.ahora))?;
    let op = op.con_config(&config);
    let t = e.escribir(&op);
    match t.resultado {
        Resultado::Ok | Resultado::SinCambios => Ok(con_seleccion(t, op.roles())),
        _ => Err(con_dueno(t, op.dueno())),
    }
}

/// La seleccion que firma la persona (F4-6; F1-15 decision 3): dominios
/// con su senal (`ninguno` tambien), roles de diseno y modelos, leidos de
/// `roles.diseno` y `roles.implementacion` sin recalcular. Una tarea sin la
/// seleccion de F4-6 (planificada antes) no la muestra.
fn con_seleccion(i: Informe, roles: Option<crate::tarea::Roles>) -> Informe {
    let Some(r) = roles.filter(|r| !r.diseno.reglas.is_empty()) else {
        return i;
    };
    let n = r
        .diseno
        .agentes
        .iter()
        .filter(|a| !a.ends_with(roles::UNIFICADOR))
        .count();
    let senales: Vec<String> = r.diseno.senales.clone();
    i.con("dominios", lista_o_ninguno(&senales))
        .con("roles", n.to_string())
        .con("modelo-diseno", r.diseno.modelo.as_deref().unwrap_or("-"))
        .con(
            "modelo-implementacion",
            r.implementacion.modelo.as_deref().unwrap_or("-"),
        )
}

fn paso_pedir_cambios(e: &Entorno<'_>, id: &str, contenido: &str) -> Result<Informe, Informe> {
    let quien = (e.quien)()?;
    traer(e.git, e.dir)?;
    let op = PedirCambios::nueva(id, &quien, contenido);
    let t = e.escribir(&op);
    match t.resultado {
        Resultado::Ok | Resultado::SinCambios => {
            let n = op.ronda().unwrap_or_default();
            Ok(
                t.con("cambios-pedidos", plantillas::nombre_cambios_pedidos(n))
                    .con("ronda", n.to_string()),
            )
        }
        _ => Err(con_dueno(t, op.dueno())),
    }
}

/// `plan guardar` de `contenido` en `destino` (ya validado) de la tarea
/// `id` en el repo de `e`.
pub fn guardar_en(e: &Entorno<'_>, id: &str, destino: &str, contenido: &str) -> Informe {
    let i = match paso_guardar(e, id, destino, contenido) {
        Ok(i) | Err(i) => i,
    };
    let mut i = como_orden(i, NOMBRE, "id", id);
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
    // Inventario y catalogo para la seleccion (F4-6): se leen antes de la
    // operacion, que no lee el disco.
    let claude = crate::dir_claude();
    let f = crate::seleccion::orden::fuentes(e.git, e.dir, claude.as_deref(), &[]);
    let op = Planificar::nueva(id, &quien, carril, &config, e.ahora)
        .con_fuentes(f.inventario, f.catalogo);
    let t = e.escribir(&op);
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
    let t = e.escribir(&op);
    match t.resultado {
        Resultado::Ok | Resultado::SinCambios => Ok(t),
        _ => Err(con_dueno(t, op.dueno())),
    }
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
        .con("dominios", lista_o_ninguno(&r.dominios))
        .con("modelo-diseno", o_guion(&r.modelos.0))
        .con("modelo-implementacion", o_guion(&r.modelos.1))
        .con_lista("avisos", r.avisos)
}

/// `a,b` o `ninguno` (F1-15: «dominios: ninguno» tambien se muestra).
pub(crate) fn lista_o_ninguno(v: &[String]) -> String {
    if v.is_empty() {
        "ninguno".to_owned()
    } else {
        v.join(",")
    }
}

/// El valor o `-` si esta vacio.
pub(crate) fn o_guion(s: &str) -> &str {
    if s.is_empty() { "-" } else { s }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::estado::Operacion;
    use crate::i18n::Idioma;

    /// Carpeta temporal propia del test: se borra al acabar.
    struct Carpeta(std::path::PathBuf);

    impl Drop for Carpeta {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn sin_catalogo(_: &str) -> Option<Box<dyn Operacion>> {
        None
    }

    /// Criterio 1 de TASK-037 (unitario): con el agente o sin identidad,
    /// `approve rechazada motivo=identidad rol=persona` y ni una llamada a
    /// git (ni transaccion ni `commit-tree`); el rechazo de la identidad
    /// conserva su `cache=`.
    #[test]
    fn approve_sin_persona_no_toca_nada() {
        use crate::identidad::Cuenta;
        use crate::plataforma::Proveedor;
        let agente = || -> Result<Quien, Informe> {
            Ok(Quien::Agente(Cuenta {
                proveedor: Proveedor::GitHub,
                id: 7,
                login: "ana".to_owned(),
            }))
        };
        let sin_cache = || -> Result<Quien, Informe> {
            Err(Informe::nuevo("identidad", Resultado::Rechazada)
                .con("motivo", "identidad")
                .con("cache", "falta"))
        };
        let quienes: [&dyn Fn() -> Result<Quien, Informe>; 2] = [&agente, &sin_cache];
        for (quien, esperado) in quienes.into_iter().zip([
            "approve rechazada id=TASK-001 motivo=identidad rol=persona",
            "approve rechazada id=TASK-001 motivo=identidad rol=persona cache=falta",
        ]) {
            let g = Git::nuevo();
            let dir = std::env::temp_dir();
            let e = Entorno {
                dir: &dir,
                git: &g,
                quien,
                catalogo: &sin_catalogo,
                ahora: 0,
            };
            let i = orden_approve(&["TASK-001".to_owned()], &e);
            assert_eq!(i.linea(), esperado);
            assert_eq!(i.resultado.codigo(), 3);
            assert!(g.registro().is_empty(), "{:?}", g.registro());
            let d = i.diagnostico_en(Idioma::Es).unwrap_or_default();
            assert!(d.contains("RR-04"), "{d}");
        }
    }

    /// C2 de TASK-038 (unitario): `approve --hito` con el agente es
    /// `rechazada motivo=identidad rol=persona` sin una llamada a git; mal
    /// llamado, `uso` sin preguntar siquiera quien es.
    #[test]
    fn approve_hito_sin_persona_o_mal_llamado_no_toca_nada() {
        use crate::identidad::Cuenta;
        use crate::plataforma::Proveedor;
        let agente = || -> Result<Quien, Informe> {
            Ok(Quien::Agente(Cuenta {
                proveedor: Proveedor::GitHub,
                id: 7,
                login: "ana".to_owned(),
            }))
        };
        type Quienes<'a> = &'a dyn Fn() -> Result<Quien, Informe>;
        let nadie = || -> Result<Quien, Informe> { unreachable!() };
        let casos: [(&[&str], Quienes<'_>, &str); 7] = [
            (
                &["--hito", "4"],
                &agente,
                "approve rechazada hito=4 motivo=identidad rol=persona",
            ),
            (
                &["--hito", "H4"],
                &agente,
                "approve rechazada hito=4 motivo=identidad rol=persona",
            ),
            (&["--hito", "x"], &nadie, "approve uso motivo=hito-invalido"),
            (
                &["--hito", "-1"],
                &nadie,
                "approve uso motivo=hito-invalido",
            ),
            (&["--hito"], &nadie, "approve uso motivo=sin-hito"),
            (
                &["TASK-001", "--hito=4"],
                &nadie,
                "approve uso motivo=hito-con-id",
            ),
            (
                &["--hito", "4", "--decidido-por", "automatico"],
                &nadie,
                "approve uso motivo=decidido-por",
            ),
        ];
        for (args, quien, esperado) in casos {
            let g = Git::nuevo();
            let dir = std::env::temp_dir();
            let e = Entorno {
                dir: &dir,
                git: &g,
                quien,
                catalogo: &sin_catalogo,
                ahora: 0,
            };
            let args: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
            let i = orden_approve(&args, &e);
            assert_eq!(i.linea(), esperado, "{args:?}");
            assert!(g.registro().is_empty(), "{:?}", g.registro());
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
        let tope = crate::lectura::MAX_BYTES;
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
