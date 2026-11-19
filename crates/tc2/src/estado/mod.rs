//! Rama de estado `taskcodetwo/tareas` (F2-3; ADR F1-1 y F1-2; nombres en
//! [`crate::huella`]).
//!
//! - [`montar`] crea la rama huerfana si no existe y la monta como worktree
//!   en `tareas/` del worktree principal (idempotente; la vuelve a montar si
//!   se desmonto o se borro la carpeta).
//! - [`transaccion`] es el **unico punto de escritura**: cerrojo local,
//!   `fetch`, la operacion sobre el estado fresco, commit construido sin
//!   tocar ningun worktree y `push` sin force; si el push se rechaza por
//!   non-fast-forward, re-ejecuta la operacion sobre lo nuevo (nunca rebase).
//!   Sin red, las operaciones que no reservan nada compartido quedan como
//!   pendientes y se re-ejecutan en la siguiente transaccion con red.
//! - [`leer`] da una vista del estado (lo ultimo traido del remoto, o la
//!   rama local en un repo sin remoto) y [`tareas`], sus `tarea.md` de los
//!   dos formatos con su lectura.
//! - [`leer_fresco`] (F3-5b) trae antes la rama de estado con el mismo
//!   `fetch` que la transaccion y dice su [`Frescura`]: la hora del ultimo
//!   `fetch` con exito ([`Marca`]) y por que no se pudo traer, si no se pudo.
//!   La usan `status`, `board`, `next` y `roadmap`.
//!
//! La correccion no depende de `tareas/`: se lee y se escribe sobre commits;
//! la carpeta montada solo se adelanta (fast-forward) tras publicar.
//!
//! Salida con el contrato de F1-4 (`estado <resultado> clave=valor`) y los
//! codigos de la tabla de `contrato.rs`.

mod cerrojo;
mod frescura;
mod lectura;
mod montaje;
mod pendientes;
mod tareas;
mod transaccion;

pub use crate::git::{Cambio, Sello};
pub use cerrojo::Cerrojo;
pub use frescura::{Frescura, Marca, Remoto, hace, sincronizado_texto};
pub(crate) use lectura::referencia_de_lectura;
pub use lectura::{Estado, Local, Vista, leer, leer_fresco, local_de, sincronizado_en};
pub use montaje::{Previsto, montar, montar_con, previsto};
pub use tareas::{Leida, de_otra_persona, es_tarea_md, por_estado, tarea_de, tareas};
pub use transaccion::{Catalogo, Contexto, Fallo, Operacion, Politica, Roadmap, transaccion};

/// Rama huerfana del estado (ADR F1-1; nombre en [`crate::huella`]).
pub const RAMA: &str = crate::huella::RAMA_ESTADO;

/// Carpeta del worktree principal donde se monta (ADR F1-1).
pub const CARPETA: &str = "tareas";

/// `ROADMAP.md` en la raiz de la rama de estado (ADR F1-1, decision 3): lo
/// escribe el generador de la transaccion y lo leen `next` y `roadmap`.
pub const ROADMAP: &str = "ROADMAP.md";

/// Refs de las operaciones pendientes de publicar (sin red, D2b).
pub const REFS_PENDIENTES: &str = crate::huella::REFS_PENDIENTES;

/// Refs de las pendientes que al re-ejecutarse ya no procedian: su contenido
/// queda en un objeto de git y no se pierde (pregunta abierta de F1-2).
pub const REFS_RECHAZADAS: &str = crate::huella::REFS_RECHAZADAS;

/// Nombre de la operacion en la linea de contrato.
pub(crate) const OP: &str = "estado";
