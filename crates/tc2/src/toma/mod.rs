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
//! `comprobar-id` (una rama `*/task-nnn-*` que no es la de su `rama:`,
//! `ocupada`; la propia la retoma, F3-10); la carpeta del worktree libre
//! (`carpeta-ocupada`); la
//! transaccion; la rama y el worktree.
//!
//! Lanzada desde un worktree de tarea escribe el estado en el principal (lo
//! localiza la transaccion) y crea el worktree junto al principal.
//!
//! Con la rama y el worktree listos, el aviso de solape (F3-11,
//! [`crate::solape`]): que otras tareas en curso tocan lo mismo. Solo anade
//! claves a la linea (y el aviso de stderr en `lib.rs`); nunca cambia el
//! resultado ni el codigo.

pub mod operacion;

use std::path::Path;

use crate::contrato::{Informe, Resultado};
use crate::estado::{self, Catalogo, Estado, Fallo, Operacion, como_orden};
use crate::git::{self, Git, Subcomando};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::toma as txt;
use crate::identidad::Quien;
use crate::solape::{self, EnCurso, Solape};

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

impl Entorno<'_> {
    /// La operacion por el punto unico de escritura, con el `ROADMAP.md`
    /// regenerado en el mismo commit (A8-09).
    pub(crate) fn escribir(&self, op: &dyn Operacion) -> Informe {
        let personas = crate::roadmap::personas(self.git, self.dir);
        let generar = crate::roadmap::generador(&personas);
        let ctx = estado::Contexto::nuevo(self.git, self.dir, self.catalogo, Some(&generar));
        estado::transaccion(&ctx, op)
    }
}

/// `start TASK-NNN` (sin `--json`, que quita `lib.rs`).
pub fn orden(args: &[String], e: &Entorno<'_>) -> Informe {
    orden_con_solape(args, e).0
}

/// Como [`orden`], y el solape calculado si la toma salio bien, para el
/// aviso de stderr (F3-11).
pub fn orden_con_solape(args: &[String], e: &Entorno<'_>) -> (Informe, Option<Solape>) {
    let uso = |motivo: &str| {
        Informe::nuevo(NOMBRE, Resultado::Uso)
            .con("motivo", motivo)
            .diagnostico(&txt::USO_START)
    };
    match args {
        [id] if git::es_id(id) => start_con_solape(e, id),
        [_] => (uso("id-invalido"), None),
        [] => (uso("sin-id"), None),
        _ => (uso("argumentos-de-mas"), None),
    }
}

/// `start` de la tarea `id` en el repo de `e`.
pub fn start_en(e: &Entorno<'_>, id: &str) -> Informe {
    start_con_solape(e, id).0
}

/// Como [`start_en`], y el solape calculado si la toma salio bien.
pub fn start_con_solape(e: &Entorno<'_>, id: &str) -> (Informe, Option<Solape>) {
    let mut s = None;
    let i = match paso(e, id, &mut s) {
        Ok(i) | Err(i) => i,
    };
    (como_orden(i, NOMBRE, "id", id), s)
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
        Some("pausada") => i = i.con("arreglo", "tc2 reanudar"),
        _ => {}
    }
    i
}

fn paso(e: &Entorno<'_>, id: &str, s: &mut Option<Solape>) -> Result<Informe, Informe> {
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
    // Sin el historico de la estimacion (R1-06): se calcula una sola vez, en
    // la transaccion.
    let sondeo = Tomar::nueva(id, &quien, e.ahora);
    let empezada = match sondeo.aplicar(&SinHistorico(&vista)) {
        Ok(c) => c.is_empty(),
        Err(f) => return Err(rechazo_de_toma(f.informe(NOMBRE), &sondeo)),
    };
    let rama = sondeo.rama().unwrap_or_default();
    let Some((tipo, slug)) = git::partes_de_rama(id, &rama) else {
        return Err(Fallo::rechazada(
            "rama-invalida",
            Mensaje::nuevo(&txt::RAMA_INVALIDA)
                .con("id", id)
                .con("rama", rama.as_str()),
        )
        .informe(NOMBRE)
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
        let i = destino.crear(None, &quien);
        return Ok(destino.con_solape(i, &quien, || en_curso_de(&vista), s));
    }
    // 5. El ID no tiene rama de nadie mas. La suya propia (la de `rama:`)
    // no cuenta: tras `liberar` o `reasignar` (F3-10) la siguiente persona
    // sigue en ella y `crear` la recupera con `git::reanudar`.
    let c = git::comprobar_id_salvo(g, &principal, id, Some(&rama));
    if !c.resultado.es_exito() {
        return Err(c);
    }
    // 5b. La carpeta del worktree, libre: si no, la toma quedaria publicada
    // sin poder crear el worktree.
    carpeta_libre(g, &principal, id, &rama)?;
    // 6. La toma, por el punto unico de escritura. La historia de `develop`
    // para la estimacion (F3-13), un solo `git log` antes de la transaccion;
    // cada intento recalcula con el estado fresco.
    let commits = crate::estimacion::commits_de(g, &principal);
    let leido = commits.leido();
    let op = Tomar::nueva(id, &quien, e.ahora).con_commits(commits);
    let t = e.escribir(&op);
    match t.resultado {
        Resultado::Ok => {
            let commit = t.valor("commit").map(str::to_owned);
            let mut i = destino.crear(commit, &quien);
            // Las claves de la estimacion publicada (F3-13) y, si el `git
            // log` fallo, `historia=sin-log` (R1-04).
            i = crate::estimacion::con_claves(i, op.estimacion().as_ref());
            if !leido {
                i = i.con("historia", crate::estimacion::SIN_LOG);
            }
            // El progreso en vivo (F3-12), con lo que ya calculo la toma.
            // Un fallo no cambia el resultado: `progreso=no-escrito`, y
            // `lib.rs` lo avisa por stderr; sin rotar el anterior,
            // `progreso=sin-anterior` sin aviso (TASK-128).
            if i.resultado.es_exito()
                && let Some(p) = op.progreso()
            {
                match crate::progreso::escribir_en(g, &principal, &p) {
                    Err(_) => i = i.con(crate::progreso::CLAVE, crate::progreso::NO_ESCRITO),
                    Ok(crate::progreso::fichero::Anterior::NoEscrito) => {
                        i = i.con(crate::progreso::CLAVE, crate::progreso::SIN_ANTERIOR);
                    }
                    Ok(_) => {}
                }
            }
            // El solape (F3-11) con las tareas en curso que leyo la toma.
            Ok(destino.con_solape(i, &quien, || op.en_curso(), s))
        }
        // Entre la lectura y el push paso a ser suya y empezada.
        Resultado::SinCambios => {
            let i = destino.crear(None, &quien);
            Ok(destino.con_solape(i, &quien, || en_curso_de(&vista), s))
        }
        Resultado::Rechazada => Err(rechazo_de_toma(t, &op)),
        _ => Err(t),
    }
}

/// El estado leido sin su lista de rutas, para el sondeo de [`paso`]: la
/// toma decide igual (lee su `tarea.md` y los de sus dependencias por ruta)
/// pero no lee todos los `tarea.md` ni calcula el historico de la
/// estimacion, que se calcula una sola vez en la transaccion (R1-06).
struct SinHistorico<'a>(&'a dyn Estado);

impl Estado for SinHistorico<'_> {
    fn rutas(&self) -> Vec<String> {
        Vec::new()
    }
    fn leer(&self, ruta: &str) -> Result<Option<String>, Fallo> {
        self.0.leer(ruta)
    }
    fn leer_varios(&self, rutas: &[String]) -> Result<Vec<Option<String>>, Fallo> {
        self.0.leer_varios(rutas)
    }
}

/// Las tareas en curso o en revision de un estado leido (un solo `git
/// cat-file --batch` con todos los `tarea.md`). Si la lectura falla, `None`:
/// sin solape, nunca un fallo de `start`.
fn en_curso_de(e: &dyn Estado) -> Option<Vec<EnCurso>> {
    let leidas = estado::tareas(e, &e.rutas()).ok()?;
    Some(solape::en_curso(
        leidas.iter().filter_map(|(_, d)| d.as_ref().ok()),
    ))
}

/// Base de la rama (ADR F1-3, decision 1; D4 de TASK-033): `origin/develop`;
/// para `hotfix/`, el ultimo tag (por version) contenido en `origin/main` o,
/// sin tags, su punta; `release/` no se empieza con `start`.
fn base_de(g: &Git, principal: &Path, id: &str, rama: &str) -> Result<String, Informe> {
    if rama.starts_with("release/") {
        return Err(Fallo::rechazada(
            "tipo-release",
            Mensaje::nuevo(&txt::TIPO_RELEASE).con("id", id),
        )
        .informe(NOMBRE));
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
    Err(Fallo::rechazada(
        "carpeta-ocupada",
        Mensaje::nuevo(&crate::i18n::catalogo::git::CARPETA_OCUPADA)
            .con("ruta", git::worktree_relativo(id)),
    )
    .informe(NOMBRE)
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

    /// Anade a `i` las claves del solape (F3-11) y lo deja en `salida`, solo
    /// si `i` es un exito: la rama propia ya existe en local. `en_curso` se
    /// evalua solo entonces. Las tareas de `quien` no cuentan (TASK-128).
    /// Nunca cambia el resultado de `i`.
    fn con_solape(
        &self,
        i: Informe,
        quien: &Quien,
        en_curso: impl FnOnce() -> Option<Vec<EnCurso>>,
        salida: &mut Option<Solape>,
    ) -> Informe {
        if !i.resultado.es_exito() {
            return i;
        }
        let Some(lista) = en_curso() else {
            return i;
        };
        let propio = quien.asignado();
        let Some(s) = solape::calcular(self.g, self.principal, self.id, self.rama, &propio, &lista)
        else {
            return i;
        };
        let i = s.con_claves(i);
        *salida = Some(s);
        i
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
