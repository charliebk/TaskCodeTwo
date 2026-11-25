//! `tc2 descartar`, `tc2 pausa` y `tc2 reanudar` (F3-7, RF-07 y RF-08).
//!
//! Las operaciones de la rama de estado estan en [`operacion`]; aqui, las
//! ordenes del CLI:
//!
//! - `descartar [--json] TASK-NNN "<motivo>"`: `fetch` (exige red: sin ella,
//!   `sinred pendiente=no` sin escribir nada), quien la lanza (identidad), el
//!   descarte por el punto unico de escritura ([`estado::transaccion`], con
//!   el `ROADMAP.md` regenerado en el mismo commit) y, publicado (o ya
//!   descartada), el worktree de la tarea con [`git::quitar_worktree`] (D3):
//!   se quita si esta limpio y publicado; si no, se conserva y la linea lo
//!   dice (`worktree=conservado worktree-motivo=...`). La rama no se toca
//!   nunca. Lo unico posterior a publicar es quitar el worktree, que nunca
//!   hace fallar la orden: no hay nada que comprobar antes que pueda dejar
//!   el descarte publicado y la orden rechazada.
//! - `pausa [--json] TASK-NNN` y `reanudar [--json] TASK-NNN`: quien la
//!   lanza y la fila por la transaccion; sin red quedan pendientes y la
//!   siguiente transaccion con red las publica (D4, ADR F1-2 punto 5).
//!
//! - `liberar [--json] TASK-NNN` y `reasignar [--json] TASK-NNN <login>`
//!   (F3-10, RR-12): como `descartar`, `fetch` completo (exigen red), quien
//!   la lanza y la operacion ([`liberar`]) por la transaccion; antes, el
//!   destino de `reasignar` en la lista de personas de la base
//!   (`fuera-de-lista`), la actividad de la rama publicada y el umbral de
//!   dias (clave `dias_sin_actividad`), la operacion en seco (sus rechazos
//!   salen sin esperar al push) y el worktree de la tarea en este clon: con
//!   trabajo sin publicar, `rechazada motivo=sin-publicar` sin escribir
//!   nada (D4). Publicada, el worktree se quita como en `descartar`. La rama
//!   no se toca nunca: la siguiente persona la retoma con `start`.
//!
//! Lanzadas desde un worktree de tarea escriben el estado en el principal
//! (lo localiza la transaccion). El entorno es el de `start`
//! ([`crate::toma::Entorno`]).

pub mod liberar;
pub mod operacion;
#[cfg(test)]
pub(crate) mod prueba;

use std::path::Path;

use crate::contrato::{Informe, Resultado};
use crate::estado::{self, Operacion, como_orden, con_dueno};
use crate::git::{self, Git};
use crate::i18n::catalogo::ciclo as txt;
use crate::i18n::{Mensaje, Texto};
use crate::tarea::Asignado;
use crate::toma::Entorno;

pub use liberar::{DIAS_POR_DEFECTO, Dejar, LIBERAR, Liberar, REASIGNAR, Reasignar};
pub use operacion::{DESCARTAR, Descartar, MOTIVO_MAX, Pausar, motivo_valido};

/// Reconstruye una operacion de este modulo desde su anotacion (para el
/// catalogo): las de [`operacion`] y las de [`liberar`].
pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
    operacion::desde_anotacion(a).or_else(|| liberar::desde_anotacion(a))
}

/// Nombre de la orden `pausa` (y de su fila de transicion).
pub const PAUSA: &str = crate::tarea::PAUSA;

/// Nombre de la orden `reanudar` (y de su fila de transicion).
pub const REANUDAR: &str = crate::tarea::REANUDAR;

fn uso(nombre: &str, motivo: &str, m: impl Into<Mensaje>) -> Informe {
    Informe::nuevo(nombre, Resultado::Uso)
        .con("motivo", motivo)
        .diagnostico(m)
}

/// `descartar TASK-NNN "<motivo>"` (sin `--json`, que quita `lib.rs`). Sin
/// motivo, `uso motivo=sin-motivo`; vacio, solo espacios, con saltos de
/// linea o caracteres de control o de mas de [`MOTIVO_MAX`] caracteres,
/// `uso motivo=motivo-invalido`. Nada se lee ni se escribe antes.
pub fn orden_descartar(args: &[String], e: &Entorno<'_>) -> Informe {
    let u = |motivo: &str| uso(DESCARTAR, motivo, &txt::USO_DESCARTAR);
    match args {
        [] => u("sin-id"),
        [id, ..] if !git::es_id(id) => u("id-invalido"),
        [_] => u("sin-motivo"),
        [id, m] if motivo_valido(m) => descartar_en(e, id, m),
        [_, _] => uso(
            DESCARTAR,
            "motivo-invalido",
            Mensaje::nuevo(&txt::MOTIVO_INVALIDO).con("max", MOTIVO_MAX.to_string()),
        ),
        _ => u("argumentos-de-mas"),
    }
}

/// `pausa TASK-NNN` o, con `reanudar`, `reanudar TASK-NNN`.
pub fn orden_pausa(args: &[String], reanudar: bool, e: &Entorno<'_>) -> Informe {
    let (nombre, m): (&str, &'static Texto) = if reanudar {
        (REANUDAR, &txt::USO_REANUDAR)
    } else {
        (PAUSA, &txt::USO_PAUSA)
    };
    match args {
        [id] if git::es_id(id) => pausa_en(e, id, reanudar),
        [_] => uso(nombre, "id-invalido", m),
        [] => uso(nombre, "sin-id", m),
        _ => uso(nombre, "argumentos-de-mas", m),
    }
}

/// `liberar TASK-NNN` (sin `--json`, que quita `lib.rs`).
pub fn orden_liberar(args: &[String], e: &Entorno<'_>) -> Informe {
    let u = |motivo: &str| uso(LIBERAR, motivo, &txt::USO_LIBERAR);
    match args {
        [id] if git::es_id(id) => dejar_en(e, id, None),
        [_] => u("id-invalido"),
        [] => u("sin-id"),
        _ => u("argumentos-de-mas"),
    }
}

/// `reasignar TASK-NNN <login>` (sin `--json`, que quita `lib.rs`). Sin
/// login, `uso motivo=sin-login`. Nada se lee ni se escribe antes.
pub fn orden_reasignar(args: &[String], e: &Entorno<'_>) -> Informe {
    let u = |motivo: &str| uso(REASIGNAR, motivo, &txt::USO_REASIGNAR);
    match args {
        [] => u("sin-id"),
        [id, ..] if !git::es_id(id) => u("id-invalido"),
        [_] => u("sin-login"),
        [id, login] => dejar_en(e, id, Some(login)),
        _ => u("argumentos-de-mas"),
    }
}

/// `liberar` (sin `destino`) o `reasignar` a `destino` de la tarea `id` en
/// el repo de `e`.
pub fn dejar_en(e: &Entorno<'_>, id: &str, destino: Option<&str>) -> Informe {
    let nombre = if destino.is_some() {
        REASIGNAR
    } else {
        LIBERAR
    };
    let i = match paso_dejar(e, id, destino) {
        Ok(i) | Err(i) => i,
    };
    como_orden(i, nombre, "id", id)
}

/// La persona de la lista de la base con ese login (sin distinguir
/// mayusculas); si no esta, `rechazada motivo=fuera-de-lista`.
fn destino_de(e: &Entorno<'_>, login: &str) -> Result<Asignado, Informe> {
    let personas = crate::identidad::personas_de_base(e.git, e.dir)?;
    personas
        .por_login(login)
        .and_then(|p| Asignado::persona(p.id, &p.login))
        .ok_or_else(|| {
            Informe::nuevo(REASIGNAR, Resultado::Rechazada)
                .con("motivo", "fuera-de-lista")
                .diagnostico(
                    Mensaje::nuevo(&txt::FUERA_DE_LISTA)
                        .con("login", crate::plataforma::errores::sanear(login))
                        .con("ruta", crate::plataforma::personas::RUTA)
                        .con("base", crate::config::BASE),
                )
        })
}

/// Umbral de dias sin actividad (clave `dias_sin_actividad` de la
/// configuracion de la base, D1a), con la politica de
/// [`crate::config::de_base_o_suelo`]: sin base o sin configuracion, el del
/// suelo (el de por defecto, [`DIAS_POR_DEFECTO`]); una configuracion que no
/// vale falla con su informe a nombre de `op` (M-4 de la ronda 1).
pub fn umbral_de_inactividad(g: &Git, dir: &Path, op: &str) -> Result<u64, Informe> {
    crate::config::de_base_o_suelo(g, dir, op).map(|c| u64::from(c.dias_sin_actividad))
}

/// Las dos operaciones, para tratarlas igual.
enum Dejada {
    Liberar(Liberar),
    Reasignar(Reasignar),
}

impl Dejada {
    fn op(&self) -> &dyn Operacion {
        match self {
            Self::Liberar(o) => o,
            Self::Reasignar(o) => o,
        }
    }

    fn dejar(&self) -> &Dejar {
        match self {
            Self::Liberar(o) => o.dejar(),
            Self::Reasignar(o) => o.dejar(),
        }
    }
}

fn paso_dejar(e: &Entorno<'_>, id: &str, destino: Option<&str>) -> Result<Informe, Informe> {
    let g = e.git;
    let nombre = if destino.is_some() {
        REASIGNAR
    } else {
        LIBERAR
    };
    let principal = git::localizar_principal(g, e.dir)?;
    // Exige red: lo de ahora del remoto, tambien las ramas (la actividad de
    // la de la tarea y si el worktree esta publicado).
    if git::estado::hay_remoto(g, &principal, nombre)? {
        let f = git::fetch(g, &principal);
        if !f.resultado.es_exito() {
            return Err(f);
        }
    }
    let quien = (e.quien)()?;
    let a = destino.map(|l| destino_de(e, l)).transpose()?;
    let vista = estado::leer(g, e.dir)?;
    let actividad_rama = match estado::tarea_de(&vista, id).ok().and_then(|(t, _)| t.rama) {
        Some(rama) => git::fechas_de_ramas(g, &principal, nombre)?
            .get(&rama)
            .copied(),
        None => None,
    };
    let dias = umbral_de_inactividad(g, e.dir, nombre)?;
    // Si se la queda quien la lanza, su worktree sigue siendo suyo.
    let para_mi = a
        .as_ref()
        .is_some_and(|x| x.misma_persona(&quien.asignado()));
    let op = match a {
        Some(a) => Dejada::Reasignar(Reasignar::nueva(
            id,
            &quien,
            a,
            e.ahora,
            actividad_rama,
            dias,
        )),
        None => Dejada::Liberar(Liberar::nueva(id, &quien, e.ahora, actividad_rama, dias)),
    };
    // En seco sobre lo leido: los rechazos, sin esperar al push.
    let cambia = match op.op().aplicar(&vista) {
        Ok(c) => !c.is_empty(),
        Err(f) => return Err(con_rechazo(f.informe(nombre), op.dejar())),
    };
    // El trabajo sin publicar del worktree de la tarea en este clon (D4):
    // antes de escribir nada.
    if cambia
        && !para_mi
        && let Some(m) = git::worktree_publicado(g, e.dir, nombre, id)?
    {
        return Err(Informe::nuevo(nombre, Resultado::Rechazada)
            .con("motivo", "sin-publicar")
            .con("worktree-motivo", m)
            .diagnostico(
                Mensaje::nuevo(&txt::SIN_PUBLICAR)
                    .con("id", id)
                    .con("que", m),
            ));
    }
    let t = e.escribir(op.op());
    match t.resultado {
        Resultado::Ok | Resultado::SinCambios => {
            let mut i = t;
            if let Some(x) = op.dejar().anterior() {
                i = i.con("asignado-anterior", x);
            }
            if let Some(l) = destino_de_op(&op) {
                i = i.con("asignado", l);
            }
            Ok(if para_mi {
                i
            } else {
                con_worktree(i, g, &principal, e.dir, id)
            })
        }
        _ => Err(con_rechazo(t, op.dejar())),
    }
}

/// Login de la persona a la que se reasigna.
fn destino_de_op(op: &Dejada) -> Option<String> {
    match op {
        Dejada::Reasignar(r) => r.destino().login().map(str::to_owned),
        Dejada::Liberar(_) => None,
    }
}

/// Un rechazo de `liberar` o `reasignar` con lo que sabe la operacion: quien
/// la lleva, los dias sin actividad y el umbral, o el arreglo de una tarea
/// de la 0.16.
fn con_rechazo(mut i: Informe, d: &Dejar) -> Informe {
    if i.resultado != Resultado::Rechazada {
        return i;
    }
    match i.valor("motivo") {
        Some("activa") => {
            if let Some(k) = d.activa() {
                i = i.con("dias", k.to_string());
            }
            i = i.con("umbral", d.umbral().to_string());
        }
        Some("formato-1") => i = i.con("arreglo", "tc2 actualizar"),
        _ => {}
    }
    if matches!(i.valor("motivo"), Some("activa" | "requiere-persona"))
        && let Some(l) = d.dueno()
    {
        i = i.con("asignado", l);
    }
    i
}

/// `descartar` de la tarea `id` con `motivo` (ya validado) en el repo de
/// `e`.
pub fn descartar_en(e: &Entorno<'_>, id: &str, motivo: &str) -> Informe {
    let i = match paso_descartar(e, id, motivo) {
        Ok(i) | Err(i) => i,
    };
    como_orden(i, DESCARTAR, "id", id)
}

/// `pausa` (o `reanudar`) de la tarea `id` en el repo de `e`.
pub fn pausa_en(e: &Entorno<'_>, id: &str, reanudar: bool) -> Informe {
    let op = match (e.quien)() {
        Ok(q) => Pausar::nueva(id, &q, reanudar, e.ahora),
        Err(i) => return como_orden(i, if reanudar { REANUDAR } else { PAUSA }, "id", id),
    };
    let t = e.escribir(&op);
    let i = con_dueno(t, op.dueno());
    como_orden(i, op.nombre(), "id", id)
}

fn paso_descartar(e: &Entorno<'_>, id: &str, motivo: &str) -> Result<Informe, Informe> {
    let g = e.git;
    let principal = git::localizar_principal(g, e.dir)?;
    // Exige red (D4): lo de ahora del remoto, tambien las ramas, para saber
    // si la de la tarea esta publicada antes de quitar su worktree.
    if git::estado::hay_remoto(g, &principal, DESCARTAR)? {
        let f = git::fetch(g, &principal);
        if !f.resultado.es_exito() {
            return Err(f);
        }
    }
    let quien = (e.quien)()?;
    let op = Descartar::nueva(id, &quien, motivo, e.ahora);
    let t = e.escribir(&op);
    match t.resultado {
        // Publicado, o ya descartada (repetir termina de quitar el worktree).
        Resultado::Ok | Resultado::SinCambios => Ok(con_worktree(t, g, &principal, e.dir, id)),
        _ => Err(con_dueno(t, op.dueno())),
    }
}

/// Tras el descarte, el worktree de la tarea (D3): `worktree=quitado`,
/// `ninguno` (no habia) o `conservado` con `worktree-motivo=` (cambios sin
/// guardar, commits sin publicar, HEAD suelto, lanzado desde dentro o un
/// fallo de git). La rama, local y remota, sigue.
fn con_worktree(i: Informe, g: &Git, principal: &Path, dir: &Path, id: &str) -> Informe {
    let wt = principal.join(git::CARPETA_WORKTREES).join(id);
    if dentro_de(dir, &wt) {
        return i
            .con("worktree", "conservado")
            .con("worktree-motivo", "en-uso");
    }
    let q = git::quitar_worktree(g, principal, id);
    match q.resultado {
        Resultado::Ok => i.con("worktree", "quitado"),
        Resultado::SinCambios => i.con("worktree", "ninguno"),
        _ => {
            let motivo = q.valor("motivo").unwrap_or("error").to_owned();
            i.con("worktree", "conservado")
                .con("worktree-motivo", motivo)
        }
    }
}

/// Si `dir` esta dentro de `wt` (o es el): quitarlo dejaria la orden sin
/// carpeta.
fn dentro_de(dir: &Path, wt: &Path) -> bool {
    match (std::fs::canonicalize(dir), std::fs::canonicalize(wt)) {
        (Ok(d), Ok(w)) => d.starts_with(w),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::prueba::{R, aislar, entorno, sh};
    use crate::identidad::{Cuenta, Quien};
    use crate::plataforma::Proveedor;
    use std::path::PathBuf;

    const PERSONAS: &str =
        "---\npersonas:\n  - {id: 1001, login: \"ana\"}\n  - {id: 1002, login: \"beto\"}\n";

    const DIA: u64 = 86_400;

    /// Carpeta temporal propia del test: se borra al acabar.
    struct Carpeta(PathBuf);

    impl Drop for Carpeta {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn persona(id: u64, login: &str) -> Quien {
        Quien::Persona(Cuenta {
            proveedor: Proveedor::GitHub,
            id,
            login: login.to_owned(),
        })
    }

    fn sin_catalogo(_: &str) -> Option<Box<dyn Operacion>> {
        None
    }

    /// Escenario git de los dos tests: un bare `remoto.git` y su clon.
    struct Repo {
        _borrar: Carpeta,
        raiz: PathBuf,
        clon: PathBuf,
        git: Git,
    }

    /// Monta en `tc2-tests/{nombre}-{pid}` un clon de `develop` con un
    /// primer commit de `ficheros` (ruta relativa, contenido), crea `ramas`
    /// en ese commit y publica `develop` y `ramas` en `origin` (el bare).
    fn repo_con_remoto(nombre: &str, ficheros: &[(&str, &str)], ramas: &[&str]) -> R<Repo> {
        let raiz = std::env::temp_dir()
            .join("tc2-tests")
            .join(format!("{nombre}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&raiz);
        std::fs::create_dir_all(&raiz)?;
        let borrar = Carpeta(raiz.clone());
        aislar(&raiz)?;
        sh(
            &raiz,
            &raiz,
            &["init", "-q", "--bare", "-b", "develop", "remoto.git"],
        )?;
        sh(&raiz, &raiz, &["init", "-q", "-b", "develop", "clon"])?;
        let (bare, clon) = (raiz.join("remoto.git"), raiz.join("clon"));
        for (ruta, contenido) in ficheros {
            let ruta = clon.join(ruta);
            if let Some(padre) = ruta.parent() {
                std::fs::create_dir_all(padre)?;
            }
            std::fs::write(ruta, contenido)?;
        }
        sh(&raiz, &clon, &["add", "."])?;
        sh(&raiz, &clon, &["commit", "-q", "-m", "inicial"])?;
        for rama in ramas {
            sh(&raiz, &clon, &["branch", rama])?;
        }
        let url = bare.to_str().ok_or("ruta no UTF-8")?;
        sh(&raiz, &clon, &["remote", "add", "origin", url])?;
        let mut push = vec!["push", "-q", "origin", "develop"];
        push.extend_from_slice(ramas);
        sh(&raiz, &clon, &push)?;
        sh(&raiz, &clon, &["fetch", "-q", "origin"])?;
        let git = Git::nuevo().con_entorno(entorno(&raiz));
        Ok(Repo {
            _borrar: borrar,
            raiz,
            clon,
            git,
        })
    }

    /// M-2 de la ronda 1: quien se reasigna a si misma la tarea de otra
    /// persona (inactiva, en un terminal) se la queda con su worktree de
    /// este clon: no se rechaza `sin-publicar` aunque tenga trabajo sin
    /// publicar (`&& !para_mi`) ni se intenta quitar (`if para_mi`).
    #[test]
    fn reasignarse_a_si_misma_no_toca_su_worktree() -> R<()> {
        let Repo {
            _borrar,
            clon,
            git: g,
            ..
        } = repo_con_remoto(
            "ciclo-para-mi",
            &[(crate::plataforma::personas::RUTA, PERSONAS)],
            &["main"],
        )?;
        // TASK-001, de beto y empezada en este clon.
        let entrada = crate::alta::entrada_de_args(&["Uno".to_owned()]).map_err(|i| i.linea())?;
        let alta = crate::alta::Alta::con_entrada(entrada);
        let c = estado::Contexto::nuevo(&g, &clon, &sin_catalogo, None);
        let i = estado::transaccion(&c, &alta);
        assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
        let ahora = crate::identidad::ahora();
        let beto = || -> Result<Quien, Informe> { Ok(persona(1002, "beto")) };
        let e = Entorno {
            dir: &clon,
            git: &g,
            quien: &beto,
            catalogo: &sin_catalogo,
            ahora,
        };
        let i = crate::toma::start_en(&e, "TASK-001");
        assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
        // Trabajo sin publicar en el worktree.
        let wt = clon.join(git::CARPETA_WORKTREES).join("TASK-001");
        std::fs::write(wt.join("a-medias.txt"), "a medias\n")?;
        // Ana, en un terminal y pasado el umbral, se la reasigna a si misma.
        let ana = || -> Result<Quien, Informe> { Ok(persona(1001, "ana")) };
        let e = Entorno {
            quien: &ana,
            ahora: ahora + 30 * DIA,
            ..e
        };
        let i = dejar_en(&e, "TASK-001", Some("ana"));
        assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
        assert_eq!(i.valor("asignado"), Some("ana"), "{}", i.linea());
        assert_eq!(i.valor("asignado-anterior"), Some("beto"), "{}", i.linea());
        assert_eq!(i.valor("worktree"), None, "{}", i.linea());
        assert!(wt.join("a-medias.txt").is_file());
        Ok(())
    }

    /// A8-10: la politica comun de `plan` y de `liberar`: sin configuracion
    /// en la base, el suelo; con una que no vale, su rechazo a nombre de la
    /// orden que la pidio.
    #[test]
    fn config_de_la_base_o_suelo_a_nombre_de_la_orden() -> R<()> {
        let Repo {
            _borrar,
            raiz,
            clon,
            git: g,
        } = repo_con_remoto(
            "ciclo-suelo",
            &[(
                "README.md",
                "linea 1
",
            )],
            &[],
        )?;
        assert_eq!(
            crate::config::de_base_o_suelo(&g, &clon, "plan").map_err(|i| i.linea()),
            Ok(crate::config::Config::suelo())
        );
        std::fs::create_dir_all(clon.join(".taskcode"))?;
        std::fs::write(
            clon.join(crate::config::RUTA),
            "---
formato: 9
",
        )?;
        sh(&raiz, &clon, &["add", "."])?;
        sh(&raiz, &clon, &["commit", "-q", "-m", "config"])?;
        sh(&raiz, &clon, &["push", "-q", "origin", "develop"])?;
        sh(&raiz, &clon, &["fetch", "-q", "origin"])?;
        let i = crate::config::de_base_o_suelo(&g, &clon, "plan")
            .err()
            .ok_or("acepto formato 9")?;
        assert_eq!(i.comando, "plan", "{}", i.linea());
        assert_eq!(i.resultado, Resultado::Rechazada, "{}", i.linea());
        Ok(())
    }

    /// M-4 de la ronda 1: sin configuracion, el umbral de por defecto; con
    /// una que no vale, el rechazo de la configuracion a nombre de la orden
    /// (no el de por defecto en silencio); con una valida, su valor.
    #[test]
    fn umbral_sin_config_invalida_o_valida() -> R<()> {
        let Repo {
            _borrar,
            raiz,
            clon,
            git: g,
        } = repo_con_remoto("ciclo-umbral", &[("README.md", "linea 1\n")], &[])?;
        assert_eq!(
            umbral_de_inactividad(&g, &clon, LIBERAR).map_err(|i| i.linea()),
            Ok(DIAS_POR_DEFECTO)
        );
        let publicar = |texto: &str| -> R<()> {
            std::fs::create_dir_all(clon.join(".taskcode"))?;
            std::fs::write(clon.join(crate::config::RUTA), texto)?;
            sh(&raiz, &clon, &["add", "."])?;
            sh(&raiz, &clon, &["commit", "-q", "-m", "config"])?;
            sh(&raiz, &clon, &["push", "-q", "origin", "develop"])?;
            sh(&raiz, &clon, &["fetch", "-q", "origin"])?;
            Ok(())
        };
        publicar("---\nformato: 2\ndias_sin_actividad: 0\n")?;
        let i = umbral_de_inactividad(&g, &clon, REASIGNAR)
            .err()
            .ok_or("acepto 0")?;
        assert_eq!(i.comando, REASIGNAR, "{}", i.linea());
        assert_eq!(i.resultado, Resultado::Rechazada, "{}", i.linea());
        assert!(
            !matches!(i.valor("motivo"), None | Some("sin-base" | "sin-config")),
            "{}",
            i.linea()
        );
        publicar("---\nformato: 2\ndias_sin_actividad: 3\n")?;
        assert_eq!(
            umbral_de_inactividad(&g, &clon, LIBERAR).map_err(|i| i.linea()),
            Ok(3)
        );
        Ok(())
    }
}
