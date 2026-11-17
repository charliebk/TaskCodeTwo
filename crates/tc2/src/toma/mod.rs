//! `tc2 start [--json] TASK-NNN` (F3-6): toma atomica de la tarea, rama y
//! worktree (ADR F1-2, decision 4; F1-3, decision 2, fila `start`).
//!
//! Dos mitades separadas por el push:
//!
//! 1. La toma ([`Tomar`]) pasa por el punto unico de escritura
//!    ([`estado::transaccion`]): dos clones a la vez, uno gana y el otro sale
//!    `rechazada motivo=tomada` con el login del ganador.
//! 2. Fuera de la transaccion y solo con la toma publicada, la rama (sin
//!    upstream, desde la punta de la base) y el worktree en
//!    `.taskcodetwo/worktrees/TASK-NNN` ([`git::crear_rama_y_worktree`]). Si
//!    la tarea ya es de quien la lanza y esta empezada, no se escribe estado:
//!    se crea solo lo que falte ([`git::reanudar`]).
//!
//! Pasos de [`start_en`]: `fetch`; lectura del estado; quien lo lanza
//! (identidad, D6); la toma en seco sobre lo leido (los rechazos salen sin
//! esperar al push); la base valida (`origin/develop`, o para `hotfix` el
//! ultimo tag de `origin/main` o su punta, D4; `release` se rechaza);
//! `comprobar-id` (una rama `*/task-nnn-*` que no es de quien la lanza,
//! `ocupada`); la carpeta del worktree libre (`carpeta-ocupada`); la
//! transaccion; la rama y el worktree.
//!
//! Lanzada desde un worktree de tarea escribe el estado en el principal (lo
//! localiza la transaccion) y crea el worktree junto al principal.

pub mod operacion;

use std::path::Path;

use crate::contrato::{Informe, Resultado, Valor};
use crate::estado::{self, Catalogo, Fallo, Operacion};
use crate::git::{self, Git, Subcomando};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::toma as txt;
use crate::identidad::Quien;

pub use operacion::{Tomar, desde_anotacion, instante};

/// Nombre de la orden y de la operacion de la rama de estado.
pub const NOMBRE: &str = "start";

/// Base de un `hotfix` sin tags en `main` (ADR F1-3, decision 1).
const MAIN: &str = "origin/main";

/// Lo que necesita `start` del entorno, para poder probarlo sin terminal ni
/// tokens (como [`crate::tablero::Entorno`]). El orquestador de F4-4 lo usa
/// directamente.
pub struct Entorno<'a> {
    /// Carpeta desde la que se lanza (el principal o un worktree de tarea).
    pub dir: &'a Path,
    /// Puerto Git.
    pub git: &'a Git,
    /// Quien lanza `start`; en produccion, [`crate::identidad::resolver`].
    pub quien: &'a dyn Fn() -> Result<Quien, Informe>,
    /// Para re-ejecutar las pendientes de la transaccion.
    pub catalogo: Catalogo<'a>,
    /// Instante de la toma, en segundos desde 1970 (UTC).
    pub ahora: u64,
}

/// `start TASK-NNN` (sin `--json`, que quita `lib.rs`).
pub fn orden(args: &[String], e: &Entorno<'_>) -> Informe {
    let uso = |motivo: &str| {
        Informe::nuevo(NOMBRE, Resultado::Uso)
            .con("motivo", motivo)
            .diagnostico(&txt::USO_START)
    };
    match args {
        [id] if git::es_id(id) => start_en(e, id),
        [_] => uso("id-invalido"),
        [] => uso("sin-id"),
        _ => uso("argumentos-de-mas"),
    }
}

/// `start` de la tarea `id` en el repo de `e`.
pub fn start_en(e: &Entorno<'_>, id: &str) -> Informe {
    let i = match paso(e, id) {
        Ok(i) | Err(i) => i,
    };
    de_start(i, id)
}

/// El informe como de `start`: el comando, `id=` el primero y sin la clave
/// `operacion` de la transaccion. Un `sinred` dice siempre `pendiente=no`
/// (`start` exige red, F1-2 D2b).
fn de_start(mut i: Informe, id: &str) -> Informe {
    NOMBRE.clone_into(&mut i.comando);
    i.claves.retain(|(k, _)| k != "operacion" && k != "id");
    i.claves
        .insert(0, ("id".to_owned(), Valor::Texto(id.to_owned())));
    if i.resultado == Resultado::Sinred && i.valor("pendiente").is_none() {
        i = i.con("pendiente", "no");
    }
    i
}

fn rechazada(motivo: &str, m: Mensaje) -> Informe {
    Informe::nuevo(NOMBRE, Resultado::Rechazada)
        .con("motivo", motivo)
        .diagnostico(m)
}

/// Un rechazo de la toma con lo que sabe la operacion: el login de quien la
/// lleva o las dependencias sin cumplir.
fn rechazo_de_toma(mut i: Informe, op: &Tomar) -> Informe {
    match i.valor("motivo") {
        Some("tomada") => {
            if let Some(l) = op.dueno() {
                i = i.con("asignado", l);
            }
        }
        Some("dependencias-pendientes") => i = i.con_lista("dependencias", op.pendientes()),
        Some("formato-1") => i = i.con("arreglo", "tc2 actualizar"),
        _ => {}
    }
    i
}

fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
    let g = e.git;
    let principal = git::localizar_principal(g, e.dir)?;
    // 1. Lo de ahora del remoto: el estado, la base y las ramas.
    if git::estado::hay_remoto(g, &principal, NOMBRE)? {
        let f = git::fetch(g, &principal);
        if !f.resultado.es_exito() {
            return Err(f);
        }
    }
    let vista = estado::leer(g, e.dir)?;
    // 2. Quien la toma.
    let quien = (e.quien)()?;
    // 3. La toma en seco sobre lo leido: los rechazos, sin esperar al push.
    let op = Tomar::nueva(id, &quien, e.ahora);
    let empezada = match op.aplicar(&vista) {
        Ok(c) => c.is_empty(),
        Err(Fallo::Rechazada {
            motivo,
            diagnostico,
        }) => return Err(rechazo_de_toma(rechazada(&motivo, diagnostico), &op)),
        Err(Fallo::Error(d)) => return Err(Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d)),
    };
    let rama = op.rama().unwrap_or_default();
    let Some((tipo, slug)) = git::partes_de_rama(id, &rama) else {
        return Err(rechazada(
            "rama-invalida",
            Mensaje::nuevo(&txt::RAMA_INVALIDA)
                .con("id", id)
                .con("rama", rama.as_str()),
        )
        .con("rama", rama.as_str()));
    };
    // 4. La base, valida antes de escribir nada (F1-3).
    let base = base_de(g, &principal, id, &rama)?;
    let v = git::validar_base(g, &principal, &base);
    if !v.resultado.es_exito() {
        return Err(v);
    }
    let destino = Destino {
        g,
        principal: &principal,
        id,
        rama: &rama,
        tipo: &tipo,
        slug: &slug,
        base: &base,
    };
    if empezada {
        // 5c. Ya es suya y esta empezada: crear solo lo que falte.
        return Ok(destino.crear(None, &quien));
    }
    // 5. El ID no tiene rama de nadie mas.
    let c = git::comprobar_id(g, &principal, id);
    if !c.resultado.es_exito() {
        return Err(c);
    }
    // 5b. La carpeta del worktree, libre: si no, la toma quedaria publicada
    // sin poder crear el worktree.
    carpeta_libre(g, &principal, id, &rama)?;
    // 6. La toma, por el punto unico de escritura.
    let personas = crate::roadmap::personas(g, e.dir);
    let generar = crate::roadmap::generador(&personas);
    let ctx = estado::Contexto::nuevo(g, e.dir, e.catalogo, Some(&generar));
    let t = estado::transaccion(&ctx, &op);
    match t.resultado {
        Resultado::Ok => {
            let commit = t.valor("commit").map(str::to_owned);
            Ok(destino.crear(commit, &quien))
        }
        // Entre la lectura y el push paso a ser suya y empezada.
        Resultado::SinCambios => Ok(destino.crear(None, &quien)),
        Resultado::Rechazada => Err(rechazo_de_toma(t, &op)),
        _ => Err(t),
    }
}

/// Base de la rama (ADR F1-3, decision 1; D4 de TASK-033): `origin/develop`;
/// para `hotfix/`, el ultimo tag (por version) contenido en `origin/main` o,
/// sin tags, su punta; `release/` no se empieza con `start`.
fn base_de(g: &Git, principal: &Path, id: &str, rama: &str) -> Result<String, Informe> {
    if rama.starts_with("release/") {
        return Err(rechazada(
            "tipo-release",
            Mensaje::nuevo(&txt::TIPO_RELEASE).con("id", id),
        ));
    }
    if !rama.starts_with("hotfix/") {
        return Ok(git::base_de_rama(rama).to_owned());
    }
    if git::commit_de(g, principal, NOMBRE, MAIN)?.is_none() {
        // `validar-base` lo rechaza con su motivo.
        return Ok(MAIN.to_owned());
    }
    let tags = git::exigir(
        g,
        principal,
        NOMBRE,
        Subcomando::ForEachRef,
        &[
            "--merged",
            MAIN,
            "--sort=-v:refname",
            "--count=1",
            "--format=%(refname)",
            "refs/tags/",
        ],
    )?;
    Ok(tags
        .lines()
        .map(str::trim)
        .find(|t| !t.is_empty())
        .map_or_else(|| MAIN.to_owned(), str::to_owned))
}

/// La carpeta `.taskcodetwo/worktrees/TASK-NNN` no existe o es el worktree
/// registrado de `rama`; si la ocupa otra cosa, `rechazada
/// motivo=carpeta-ocupada` antes de escribir estado.
fn carpeta_libre(g: &Git, principal: &Path, id: &str, rama: &str) -> Result<(), Informe> {
    let wt = principal.join(git::CARPETA_WORKTREES).join(id);
    if !wt.exists() {
        return Ok(());
    }
    let registrado = git::worktrees(g, principal, NOMBRE)?
        .iter()
        .any(|w| w.rama.as_deref() == Some(rama) && git::misma_ruta(&w.ruta, &wt));
    if registrado {
        return Ok(());
    }
    Err(rechazada(
        "carpeta-ocupada",
        Mensaje::nuevo(&crate::i18n::catalogo::git::CARPETA_OCUPADA)
            .con("ruta", git::worktree_relativo(id)),
    )
    .con("rama", rama))
}

/// La rama y el worktree de la tarea.
struct Destino<'a> {
    g: &'a Git,
    principal: &'a Path,
    id: &'a str,
    rama: &'a str,
    tipo: &'a str,
    slug: &'a str,
    base: &'a str,
}

impl Destino<'_> {
    /// Crea lo que falte (F1-3, casos b y c): sin rama en local ni en el
    /// remoto, rama nueva sin upstream desde la base y su worktree; si la
    /// rama existe, [`git::reanudar`]. `commit` es el de la toma publicada,
    /// si se acaba de escribir.
    fn crear(&self, commit: Option<String>, quien: &Quien) -> Informe {
        let r = self.crear_paso(commit.is_some());
        let mut i = match r {
            Ok(i) | Err(i) => i,
        };
        let ok = i.resultado.es_exito();
        let mut s = Informe::nuevo(NOMBRE, i.resultado);
        s.claves = std::mem::take(&mut i.claves);
        s.diagnostico = i.diagnostico.take();
        if let Some(c) = commit {
            // La toma quedo publicada aunque la rama no se pudiera crear:
            // repetir `start` crea lo que falte (C2).
            s = s.con("estado", c);
        }
        if ok {
            s = s.con("asignado", quien.cuenta().login.clone());
        }
        s
    }

    fn crear_paso(&self, nueva: bool) -> Result<Informe, Informe> {
        let (g, p) = (self.g, self.principal);
        let local = git::commit_de(g, p, NOMBRE, &format!("refs/heads/{}", self.rama))?;
        let remota = git::commit_de(
            g,
            p,
            NOMBRE,
            &format!("refs/remotes/{}/{}", git::REMOTO, self.rama),
        )?;
        if local.is_none() && remota.is_none() {
            let mut i = git::crear_rama_y_worktree(g, p, self.id, self.tipo, self.slug, self.base);
            if i.resultado == Resultado::Ok && !nueva {
                i = i.con_lista("creado", vec!["rama".to_owned(), "worktree".to_owned()]);
            }
            return Ok(i);
        }
        Ok(git::reanudar(g, p, self.id, self.tipo, self.slug))
    }
}
