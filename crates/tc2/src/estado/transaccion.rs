//! Punto unico de escritura de la rama de estado (ADR F1-2, decision 1 y 2).
//!
//! Por intento: cerrojo local, `fetch` solo de la rama de estado, las
//! pendientes y la operacion sobre ese estado fresco, `ROADMAP.md`
//! regenerado si hay generador, commit con `commit-tree` y `push` sin force.
//! El rechazo non-fast-forward es el compare-and-swap: no queda nada local
//! que descartar y se vuelve a empezar re-ejecutando la operacion, con
//! espera `base * 2^(n-1) * U(0,5; 1)`. Agotados los intentos, `agotada` sin
//! nada escrito.

use std::hash::{BuildHasher, Hasher, RandomState};
use std::path::Path;
use std::time::Duration;

use super::cerrojo::{Cerrojo, ErrorCerrojo};
use super::lectura::{Estado, Vista};
use super::pendientes::{self, ANOTACION, Destino};
use super::{CARPETA, OP, RAMA};
use crate::contrato::{Informe, Resultado, corto};
use crate::git::estado::{construir_commit, dir_comun, hay_remoto};
use crate::git::remoto::publicar;
use crate::git::{
    Cambio, FalloRemoto, Git, REMOTO, Subcomando, clasificar_fallo_remoto, commit_de, correr,
    es_ancestro, exigir, localizar_principal, worktrees,
};

/// Por que una operacion no produce cambios.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fallo {
    /// La operacion no procede sobre este estado (la tarea la lleva otro, no
    /// existe...): `rechazada` (3), no se reintenta.
    Rechazada {
        /// Valor de `motivo=`.
        motivo: String,
        /// Que hacer, en una linea.
        diagnostico: String,
    },
    /// Fallo interno: `error` (1).
    Error(String),
}

impl Fallo {
    /// Atajo para [`Fallo::Rechazada`].
    pub fn rechazada(motivo: impl Into<String>, diagnostico: impl Into<String>) -> Self {
        Self::Rechazada {
            motivo: motivo.into(),
            diagnostico: diagnostico.into(),
        }
    }
}

/// Una operacion sobre el estado. Se re-ejecuta en cada intento: tiene que
/// decidir solo a partir del estado que recibe.
pub trait Operacion {
    /// Nombre corto (`alta`, `start`...), para la linea y el commit.
    fn nombre(&self) -> &str;
    /// Una linea que permite reconstruirla con el [`Catalogo`] (las
    /// pendientes se re-ejecutan desde ella).
    fn anotacion(&self) -> String;
    /// Si reserva algo compartido (`new`, `import`, `start`, `liberar`,
    /// `reasignar`): sin red falla en vez de quedar pendiente (D2b).
    fn exige_red(&self) -> bool;
    /// Los cambios sobre `estado`, o por que no procede.
    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo>;
}

/// Reconstruye una operacion desde su anotacion (`None`: desconocida).
pub type Catalogo<'a> = &'a dyn Fn(&str) -> Option<Box<dyn Operacion>>;

/// Genera `ROADMAP.md` desde el estado ya cambiado (se regenera, nunca se
/// mergea: ADR F1-1). Lo pone F3-4; hasta entonces, `None`.
pub type Roadmap<'a> = &'a dyn Fn(&dyn Estado) -> Result<String, Fallo>;

/// Reintentos y esperas (ADR F1-2, D3a: 5 intentos, base 2 s).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Politica {
    /// Intentos de publicar antes de `agotada`.
    pub intentos: u32,
    /// Base de la espera exponencial con jitter.
    pub base: Duration,
    /// Espera maxima por el cerrojo local.
    pub espera_cerrojo: Duration,
}

impl Default for Politica {
    fn default() -> Self {
        Self {
            intentos: 5,
            base: Duration::from_secs(2),
            espera_cerrojo: Duration::from_secs(60),
        }
    }
}

/// Todo lo que necesita una transaccion.
pub struct Contexto<'a> {
    /// Puerto Git.
    pub git: &'a Git,
    /// Carpeta desde la que se lanza (cualquier worktree del repo).
    pub dir: &'a Path,
    /// Reintentos y esperas.
    pub politica: Politica,
    /// Para re-ejecutar las pendientes.
    pub catalogo: Catalogo<'a>,
    /// Generador de `ROADMAP.md`, si lo hay.
    pub roadmap: Option<Roadmap<'a>>,
}

impl<'a> Contexto<'a> {
    /// Contexto con la politica por defecto y sin generador de roadmap.
    pub fn nuevo(git: &'a Git, dir: &'a Path, catalogo: Catalogo<'a>) -> Self {
        Self {
            git,
            dir,
            politica: Politica::default(),
            catalogo,
            roadmap: None,
        }
    }
}

enum Intento {
    Hecho(Informe),
    Reintentar,
}

/// Como quedo la rama local y `tareas/` tras publicar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tareas {
    /// `tareas/` montada y adelantada.
    AlDia,
    /// Sin montar: se adelanto (o creo) la rama local.
    SinMontar,
    /// Montada, pero con cambios de la persona que impiden adelantarla.
    ConCambios,
    /// La rama local tiene commits que no estan en lo publicado.
    Diverge,
}

impl Tareas {
    fn texto(self) -> &'static str {
        match self {
            Self::AlDia => "al-dia",
            Self::SinMontar => "sin-montar",
            Self::ConCambios => "con-cambios",
            Self::Diverge => "diverge",
        }
    }
}

fn informe(r: Resultado, op: &dyn Operacion) -> Informe {
    Informe::nuevo(OP, r).con("operacion", op.nombre())
}

fn error(diagnostico: impl Into<String>) -> Informe {
    Informe::nuevo(OP, Resultado::Error).diagnostico(diagnostico)
}

fn renombrado(mut i: Informe) -> Informe {
    OP.clone_into(&mut i.comando);
    i
}

/// Ejecuta `op` sobre la rama de estado con el protocolo de F1-2.
///
/// Resultados: `ok` (publicada; con `tareas=` dice si la carpeta montada se
/// adelanto), `sin-cambios`, `rechazada` (3, la operacion no procede),
/// `sinred` (4, `pendiente=si|no`), `agotada` (5, nada escrito), `denegada`
/// (8, credenciales, hook o regla del remoto) o `error`.
pub fn transaccion(c: &Contexto<'_>, op: &dyn Operacion) -> Informe {
    ejecutar(c, op).unwrap_or_else(renombrado)
}

fn ejecutar(c: &Contexto<'_>, op: &dyn Operacion) -> Result<Informe, Informe> {
    let anotacion = op.anotacion();
    if anotacion.trim().is_empty() || anotacion.contains(['\n', '\r']) {
        return Ok(informe(Resultado::Uso, op).con("motivo", "anotacion-invalida"));
    }
    if c.politica.intentos == 0 {
        return Ok(informe(Resultado::Uso, op).con("motivo", "sin-intentos"));
    }
    let principal = localizar_principal(c.git, c.dir)?;
    let comun = dir_comun(c.git, &principal, OP)?;
    let remoto = hay_remoto(c.git, &principal, OP)?;
    let mut n = 0;
    loop {
        n += 1;
        let cerrojo = match Cerrojo::tomar(&comun, c.politica.espera_cerrojo) {
            Ok(x) => x,
            Err(ErrorCerrojo::Ocupado) => {
                return Ok(informe(Resultado::Agotada, op)
                    .con("motivo", "cerrojo")
                    .diagnostico(
                        "otro proceso de este clon esta escribiendo la rama de estado: reintenta",
                    ));
            }
            Err(ErrorCerrojo::Io(e)) => return Err(error(format!("cerrojo: {e}"))),
        };
        let r = intento(c, op, &anotacion, &principal, remoto, n);
        drop(cerrojo);
        match r? {
            Intento::Hecho(i) => return Ok(i),
            Intento::Reintentar if n >= c.politica.intentos => {
                return Ok(informe(Resultado::Agotada, op)
                    .con("intentos", n.to_string())
                    .diagnostico(
                        "la rama de estado cambio en cada intento: no se ha escrito nada; reintenta",
                    ));
            }
            Intento::Reintentar => std::thread::sleep(espera(c.politica.base, n)),
        }
    }
}

/// `base * 2^(n-1) * U(0,5; 1)`. El azar sale de `RandomState` (claves
/// aleatorias por proceso), sin dependencias.
fn espera(base: Duration, n: u32) -> Duration {
    let azar = RandomState::new().build_hasher().finish() % 1000;
    let factor = 0.5 + f64::from(u32::try_from(azar).unwrap_or(0)) / 2000.0;
    let exp = 2u32.saturating_pow(n.saturating_sub(1)).min(1024);
    base.mul_f64(f64::from(exp) * factor)
}

fn mensaje(nombre: &str, anotaciones: &[String]) -> String {
    let mut m = format!("estado: {nombre}\n\n");
    for a in anotaciones {
        m.push_str(ANOTACION);
        m.push_str(a);
        m.push('\n');
    }
    m
}

fn rama_remota() -> String {
    format!("refs/remotes/{REMOTO}/{RAMA}")
}

fn rama_local() -> String {
    format!("refs/heads/{RAMA}")
}

fn de_fallo(f: Fallo) -> Informe {
    match f {
        Fallo::Error(d) => error(d),
        Fallo::Rechazada {
            motivo,
            diagnostico,
        } => error(format!("{motivo}: {diagnostico}")),
    }
}

fn intento(
    c: &Contexto<'_>,
    op: &dyn Operacion,
    anotacion: &str,
    principal: &Path,
    remoto: bool,
    n: u32,
) -> Result<Intento, Informe> {
    let git = c.git;
    let base = if remoto {
        let refspec = format!("refs/heads/{RAMA}:{}", rama_remota());
        let s = correr(
            git,
            principal,
            OP,
            Subcomando::Fetch,
            &["-q", REMOTO, &refspec],
        )?;
        if s.exito() {
            commit_de(git, principal, OP, &rama_remota())?
        } else {
            match clasificar_fallo_remoto(&s.stderr) {
                // El remoto aun no tiene la rama. Si existe la local (un repo
                // que se monto sin remoto y al que luego se anadio `origin`),
                // se parte de ella para publicar su historia, no de cero.
                FalloRemoto::RamaInexistente => commit_de(git, principal, OP, &rama_local())?,
                FalloRemoto::SinRed => {
                    return sin_red(c, op, anotacion, principal, &s.primera_linea_error());
                }
                FalloRemoto::Local => {
                    return Err(crate::git::remoto::error_local(
                        OP,
                        &s.primera_linea_error(),
                    ));
                }
                otro => return Ok(Intento::Hecho(denegada(op, otro.motivo(), &s.stderr))),
            }
        }
    } else {
        commit_de(git, principal, OP, &rama_local())?
    };

    let mut vista = Vista::nueva(git, principal, base.clone())?;
    let mut anotaciones = Vec::new();
    let mut destinos = Vec::new();
    if remoto {
        for p in pendientes::listar(git, principal)? {
            let reconstruida = p.anotacion.as_deref().and_then(|a| (c.catalogo)(a));
            let Some(o) = reconstruida else {
                destinos.push((p, Destino::Rechazada));
                continue;
            };
            match o.aplicar(&vista) {
                // Una pendiente que choca por mayusculas con lo publicado ya
                // no procede: si se quedara, bloquearia cada operacion.
                Ok(cambios) if choque(&vista.rutas(), &cambios).is_some() => {
                    destinos.push((p, Destino::Rechazada));
                }
                Ok(cambios) => {
                    vista.superponer(cambios);
                    anotaciones.push(o.anotacion());
                    destinos.push((p, Destino::Publicada));
                }
                // Tambien un error al re-ejecutarla: si abortara, bloquearia
                // todas las escrituras siguientes mientras la ref siga ahi.
                // Rechazada, su contenido queda en git y la linea lo dice.
                Err(_) => destinos.push((p, Destino::Rechazada)),
            }
        }
    }
    match op.aplicar(&vista) {
        Ok(cambios) => {
            if let Some(i) = duplicada(&vista.rutas(), &cambios, op) {
                return Ok(Intento::Hecho(i));
            }
            vista.superponer(cambios);
        }
        Err(Fallo::Rechazada {
            motivo,
            diagnostico,
        }) => {
            return Ok(Intento::Hecho(
                informe(Resultado::Rechazada, op)
                    .con("motivo", motivo)
                    .diagnostico(diagnostico),
            ));
        }
        Err(Fallo::Error(d)) => return Err(error(d)),
    }
    anotaciones.push(anotacion.to_owned());
    if let Some(generar) = c.roadmap {
        let texto = generar(&vista).map_err(de_fallo)?;
        vista.superponer(vec![Cambio::nuevo("ROADMAP.md", texto)]);
    }

    let mensaje = mensaje(op.nombre(), &anotaciones);
    let Some(sha) = construir_commit(
        git,
        principal,
        OP,
        base.as_deref(),
        &vista.cambios(),
        &mensaje,
    )?
    else {
        let (publicadas, rechazadas) = pendientes::resolver(git, principal, &destinos)?;
        return Ok(Intento::Hecho(con_pendientes(
            informe(Resultado::SinCambios, op),
            publicadas,
            rechazadas,
        )));
    };

    if !remoto {
        // Sin remoto el commit local es la transaccion (RN-06); el cerrojo
        // serializa y `adelantar` comprueba que la rama no se movio.
        return Ok(match adelantar(git, principal, &sha)? {
            Tareas::Diverge => Intento::Reintentar,
            Tareas::ConCambios => Intento::Hecho(
                informe(Resultado::Rechazada, op)
                    .con("motivo", "tareas-con-cambios")
                    .diagnostico(format!(
                        "{CARPETA}/ tiene cambios que chocan con la operacion: guardalos o deshazlos y reintenta"
                    )),
            ),
            t => Intento::Hecho(ok(op, &sha, n, t, 0, 0)),
        });
    }

    let p = publicar(git, principal, OP, &format!("{sha}:refs/heads/{RAMA}"), &[])?;
    match p.resultado {
        Resultado::Ok | Resultado::SinCambios => {
            let (publicadas, rechazadas) = pendientes::resolver(git, principal, &destinos)?;
            let t = adelantar(git, principal, &sha)?;
            Ok(Intento::Hecho(ok(op, &sha, n, t, publicadas, rechazadas)))
        }
        Resultado::Rechazada => Ok(Intento::Reintentar),
        Resultado::Denegada => Ok(Intento::Hecho(denegada(op, p.motivo, &p.ultimo))),
        // Fallo del propio clon al publicar: ni sin red ni pendiente.
        Resultado::Error => Err(crate::git::remoto::error_local(OP, &p.ultimo)),
        _ => sin_red(c, op, anotacion, principal, &p.ultimo),
    }
}

/// Primera ruta de `cambios` que solo difiere en mayusculas de otra ya
/// presente en `rutas` (en Windows o macOS las dos chocarian al sacarlas al
/// worktree): (la nueva, la que estaba).
fn choque(rutas: &[String], cambios: &[Cambio]) -> Option<(String, String)> {
    cambios.iter().find_map(|c| {
        let minus = c.ruta.to_lowercase();
        rutas
            .iter()
            .find(|r| **r != c.ruta && r.to_lowercase() == minus)
            .map(|otra| (c.ruta.clone(), otra.clone()))
    })
}

/// `rechazada motivo=ruta-duplicada` si los cambios de la operacion chocan.
fn duplicada(rutas: &[String], cambios: &[Cambio], op: &dyn Operacion) -> Option<Informe> {
    choque(rutas, cambios).map(|(nueva, otra)| {
        informe(Resultado::Rechazada, op)
            .con("motivo", "ruta-duplicada")
            .diagnostico(format!(
                "{nueva} solo difiere en mayusculas de {otra}: chocarian en Windows y macOS"
            ))
    })
}

fn denegada(op: &dyn Operacion, motivo: &str, detalle: &str) -> Informe {
    let primera = detalle
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("sin mensaje");
    informe(Resultado::Denegada, op)
        .con("motivo", motivo)
        .diagnostico(format!(
            "{primera}; el remoto deniega la escritura: revisa credenciales, permisos o reglas, no se reintenta"
        ))
}

fn con_pendientes(mut i: Informe, publicadas: usize, rechazadas: usize) -> Informe {
    if publicadas > 0 {
        i = i.con("pendientes", publicadas.to_string());
    }
    if rechazadas > 0 {
        i = i.con("rechazadas", rechazadas.to_string());
    }
    i
}

fn ok(
    op: &dyn Operacion,
    sha: &str,
    n: u32,
    t: Tareas,
    publicadas: usize,
    rechazadas: usize,
) -> Informe {
    con_pendientes(
        informe(Resultado::Ok, op)
            .con("commit", corto(sha))
            .con("intentos", n.to_string())
            .con("tareas", t.texto()),
        publicadas,
        rechazadas,
    )
}

/// Sin red: si la operacion exige red, `sinred pendiente=no` sin escribir
/// nada; si no, se comprueba contra lo ultimo conocido (una tarea ajena se
/// rechaza tambien sin red) y queda como pendiente.
fn sin_red(
    c: &Contexto<'_>,
    op: &dyn Operacion,
    anotacion: &str,
    principal: &Path,
    detalle: &str,
) -> Result<Intento, Informe> {
    if op.exige_red() {
        return Ok(Intento::Hecho(
            informe(Resultado::Sinred, op)
                .con("pendiente", "no")
                .diagnostico(format!(
                    "{detalle}; esta operacion exige red: no se ha escrito nada, reintenta con red"
                )),
        ));
    }
    let git = c.git;
    let previas = pendientes::listar(git, principal)?;
    let conocida = match previas.last() {
        Some(p) => Some(p.sha.clone()),
        None => commit_de(git, principal, OP, &rama_remota())?,
    };
    let mut vista = Vista::nueva(git, principal, conocida.clone())?;
    match op.aplicar(&vista) {
        Ok(cambios) => {
            if let Some(i) = duplicada(&vista.rutas(), &cambios, op) {
                return Ok(Intento::Hecho(i));
            }
            vista.superponer(cambios);
        }
        Err(Fallo::Rechazada {
            motivo,
            diagnostico,
        }) => {
            return Ok(Intento::Hecho(
                informe(Resultado::Rechazada, op)
                    .con("motivo", motivo)
                    .diagnostico(diagnostico),
            ));
        }
        Err(Fallo::Error(d)) => return Err(error(d)),
    }
    let mensaje = mensaje(op.nombre(), &[anotacion.to_owned()]);
    let Some(sha) = construir_commit(
        git,
        principal,
        OP,
        conocida.as_deref(),
        &vista.cambios(),
        &mensaje,
    )?
    else {
        return Ok(Intento::Hecho(informe(Resultado::SinCambios, op)));
    };
    pendientes::guardar(git, principal, &sha)?;
    Ok(Intento::Hecho(
        informe(Resultado::Sinred, op)
            .con("pendiente", "si")
            .con("pendientes", (previas.len() + 1).to_string())
            .diagnostico(format!(
                "{detalle}; queda pendiente de publicar: se publicara en la siguiente operacion con red"
            )),
    ))
}

/// Lleva la rama local `taskcode/tareas` (y `tareas/` si esta montada) a
/// `sha`, solo por fast-forward.
fn adelantar(git: &Git, principal: &Path, sha: &str) -> Result<Tareas, Informe> {
    let local = commit_de(git, principal, OP, &rama_local())?;
    if let Some(l) = &local
        && !es_ancestro(git, principal, OP, l, sha)?
    {
        return Ok(Tareas::Diverge);
    }
    let wts = worktrees(git, principal, OP)?;
    let mut huerfano = false;
    for w in wts.iter().filter(|w| w.rama.as_deref() == Some(RAMA)) {
        if !w.ruta.exists() {
            huerfano = true;
            continue;
        }
        let s = correr(
            git,
            &w.ruta,
            OP,
            Subcomando::Merge,
            &["-q", "--ff-only", sha],
        )?;
        return Ok(if s.exito() {
            Tareas::AlDia
        } else {
            Tareas::ConCambios
        });
    }
    if huerfano {
        // La carpeta se borro a mano: git aun cree la rama en uso.
        exigir(git, principal, OP, Subcomando::Worktree, &["prune"])?;
    }
    match local {
        None => {
            exigir(git, principal, OP, Subcomando::Branch, &[RAMA, sha])?;
        }
        Some(_) => {
            // La rama no esta en ningun worktree: se adelanta con un fetch
            // local sin `+` (solo fast-forward) desde una ref temporal propia.
            let tmp = format!(
                "{}tmp/{}",
                crate::git::PREFIJO_REFS_PROPIAS,
                std::process::id()
            );
            exigir(git, principal, OP, Subcomando::UpdateRef, &[&tmp, sha])?;
            let s = correr(
                git,
                principal,
                OP,
                Subcomando::Fetch,
                &["-q", ".", &format!("{tmp}:{}", rama_local())],
            );
            // La ref temporal se borra tambien si el fetch fallo; si no se
            // puede borrar, se dice (no se deja basura sin avisar).
            let borrado = exigir(
                git,
                principal,
                OP,
                Subcomando::UpdateRef,
                &["-d", &tmp, sha],
            );
            let s = s?;
            if !s.exito() {
                return Err(error(format!(
                    "no se pudo adelantar {RAMA}: {}",
                    s.primera_linea_error()
                )));
            }
            borrado?;
        }
    }
    Ok(Tareas::SinMontar)
}

#[cfg(test)]
mod tests {
    use super::espera;
    use std::time::Duration;

    #[test]
    fn espera_exponencial_con_jitter_entre_la_mitad_y_el_total() {
        let base = Duration::from_millis(100);
        for n in 1..=4u32 {
            let tope = base * 2u32.pow(n - 1);
            for _ in 0..50 {
                let e = espera(base, n);
                assert!(
                    e >= tope / 2 && e <= tope,
                    "n={n}: {e:?} fuera de [{:?}, {tope:?}]",
                    tope / 2
                );
            }
        }
        let distintas: std::collections::BTreeSet<Duration> =
            (0..20).map(|_| espera(base, 1)).collect();
        assert!(distintas.len() > 1, "sin jitter");
    }
}
