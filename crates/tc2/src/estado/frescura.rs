//! Frescura de la lectura (F3-5b, RR-12): la hora del ultimo `fetch` con
//! exito de la rama de estado y por que la ultima lectura no pudo traerla.
//!
//! - [`Marca`] guarda la hora en `<git-common-dir>/taskcodetwo/sincronizacion`
//!   (D3a), como la cache de identidad: temporal y `rename`, de modo que quien
//!   lee no ve nunca un fichero a medias. Un fichero que no se entiende, o
//!   demasiado grande, es «nunca»: la hora nunca sale mas fresca de lo real.
//! - [`Frescura`] es la unica fuente del formato: las claves `remoto=` y
//!   `sincronizado=` ([`Frescura::anotar`]) y el aviso de stderr
//!   ([`Frescura::aviso`]). La hora va en UTC, `AAAA-MM-DDTHH:MM:SSZ`, mas
//!   «hace N» en los textos para la persona (D6a).

use std::path::{Path, PathBuf};

use super::{OP, RAMA};
use crate::contrato::Informe;
use crate::git::{FalloRemoto, Git, REMOTO, Subcomando, clasificar_fallo_remoto, correr};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::estado as txt;

/// Como fue el `fetch` de la rama de estado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Traida {
    /// Salio bien: `origin/taskcodetwo/tareas` es lo ultimo publicado.
    Hecha,
    /// `origin` no tiene la rama de estado.
    SinRama,
    /// `origin` tiene una rama de estado que no desciende de la que trajo
    /// este clon (reescrita con force): el `fetch` sin `+` la rechaza
    /// (`! [rejected] ... (non-fast-forward)`). No es falta de red.
    Reescrita {
        /// La linea `[rejected]` de git.
        detalle: String,
    },
    /// La ref remota de este clon esta bloqueada (`cannot lock ref ...: File
    /// exists`): otro `fetch` del mismo clon a la vez o un `.lock` huerfano.
    Bloqueada {
        /// La linea `cannot lock ref` de git.
        detalle: String,
    },
    /// No salio: la clasificacion y el stderr de git.
    Fallo {
        /// Por que.
        fallo: FalloRemoto,
        /// stderr entero.
        stderr: String,
        /// Primera linea de error.
        primera: String,
    },
}

impl Traida {
    /// Lo que dice la lectura de este intento.
    pub(crate) fn remoto(&self) -> Remoto {
        match self {
            Self::Hecha => Remoto::Consultado,
            Self::SinRama => Remoto::SinRama,
            Self::Reescrita { .. } => Remoto::Rechazado,
            Self::Bloqueada { detalle } => Remoto::RefBloqueada(detalle.clone()),
            // `RamaInexistente` no llega aqui: `traer_estado` la da como
            // `SinRama`.
            Self::Fallo { fallo, .. } => match fallo {
                FalloRemoto::SinRed => Remoto::SinRed,
                FalloRemoto::Local => Remoto::ErrorLocal,
                otro => Remoto::Denegado(otro.motivo().to_owned()),
            },
        }
    }
}

/// Trae solo la rama de estado (`fetch` de `refs/heads/taskcodetwo/tareas` a
/// `refs/remotes/origin/taskcodetwo/tareas`, sin `+`) y, si sale bien,
/// registra `ahora` en la marca. Es el `fetch` de cada intento de la
/// transaccion, el de la lectura fresca y el de `montar`: uno solo, sin
/// copias. Un fallo al escribir la marca no se propaga: la hora se queda la
/// anterior, nunca mas fresca de lo real. `Err` solo si no se pudo lanzar
/// git.
///
/// Sin `-q`: con `-q`, git calla el rechazo non-fast-forward (stderr vacio,
/// comprobado con git 2.56) y no se podria distinguir de otros fallos. En
/// exito el stderr no se mira.
pub(crate) fn traer_estado(
    git: &Git,
    principal: &Path,
    marca: &Marca,
    ahora: u64,
) -> Result<Traida, Informe> {
    let refspec = format!("refs/heads/{RAMA}:refs/remotes/{REMOTO}/{RAMA}");
    let s = correr(git, principal, OP, Subcomando::Fetch, &[REMOTO, &refspec])?;
    if s.exito() {
        let _ = marca.registrar(ahora);
        return Ok(Traida::Hecha);
    }
    let primera = s.primera_linea_error();
    Ok(clasificar_traida(s.stderr, primera))
}

/// `error` (1) de una escritura (transaccion o `montar`) que encuentra la
/// rama de estado reescrita en `origin`: no se escribe nada ni queda
/// pendiente (reintentar no lo arregla, y sin red no es).
pub(crate) fn rama_reescrita(op: &str, detalle: &str) -> Informe {
    Informe::nuevo(op, crate::contrato::Resultado::Error)
        .con("motivo", "rama-reescrita")
        .diagnostico(Mensaje::nuevo(&txt::RAMA_REESCRITA).con("detalle", detalle))
}

/// Clasifica el stderr de un `fetch` de la rama de estado que fallo. Antes
/// que [`clasificar_fallo_remoto`] (que da como `sin-red` lo que no
/// reconoce), dos fallos que no son de red: el rechazo non-fast-forward (la
/// rama reescrita en `origin`; manda aunque haya ademas un `.lock`, porque
/// es la causa) y la ref remota bloqueada por un `.lock`.
fn clasificar_traida(stderr: String, primera: String) -> Traida {
    let linea = |pred: &dyn Fn(&str) -> bool| {
        stderr
            .lines()
            .map(str::trim)
            .find(|l| pred(&l.to_ascii_lowercase()))
            .map(str::to_owned)
    };
    if let Some(detalle) = linea(&|l| l.contains("[rejected]") && l.contains("non-fast-forward")) {
        return Traida::Reescrita { detalle };
    }
    if let Some(detalle) = linea(&|l| l.contains("cannot lock ref") && l.contains("file exists")) {
        return Traida::Bloqueada { detalle };
    }
    match clasificar_fallo_remoto(&stderr) {
        FalloRemoto::RamaInexistente => Traida::SinRama,
        fallo => Traida::Fallo {
            fallo,
            primera,
            stderr,
        },
    }
}

/// Bytes como mucho del fichero de la marca (un numero y un salto de linea).
const LIMITE_MARCA: u64 = 64;

/// Marca de la ultima sincronizacion de la rama de estado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Marca {
    ruta: PathBuf,
}

impl Marca {
    /// La marca del repo cuya carpeta comun es `comun`.
    pub fn en(comun: &Path) -> Self {
        Self {
            ruta: crate::huella::en_comun(comun).join(crate::huella::FICHERO_SINCRONIZACION),
        }
    }

    /// Ruta del fichero.
    pub fn ruta(&self) -> &Path {
        &self.ruta
    }

    /// Segundos desde 1970 del ultimo `fetch` con exito; `None` si no hay
    /// marca, no se entiende o es posterior a ahora (se trata como «nunca»).
    pub fn leer(&self) -> Option<u64> {
        self.leer_en(crate::identidad::ahora())
    }

    /// Como [`Marca::leer`] con `ahora` dado. Una hora posterior a `ahora`
    /// (un reloj que iba adelantado al escribirla o un fichero tocado a
    /// mano) es «nunca»: mostrarla haria la lectura mas fresca de lo real.
    pub fn leer_en(&self, ahora: u64) -> Option<u64> {
        self.leer_crudo().filter(|s| *s <= ahora)
    }

    fn leer_crudo(&self) -> Option<u64> {
        use std::io::Read;
        let mut texto = String::new();
        std::fs::File::open(&self.ruta)
            .ok()?
            .take(LIMITE_MARCA + 1)
            .read_to_string(&mut texto)
            .ok()?;
        if texto.len() as u64 > LIMITE_MARCA {
            return None;
        }
        let t = texto.strip_suffix('\n').unwrap_or(&texto);
        if t.is_empty() || !t.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        t.parse().ok()
    }

    /// Guarda `ahora`. Escribe un temporal y lo renombra
    /// ([`crate::huella::escribir_local`]).
    pub fn registrar(&self, ahora: u64) -> std::io::Result<()> {
        crate::huella::escribir_local(&self.ruta, format!("{ahora}\n").as_bytes())
    }
}

/// Que paso al intentar traer la rama de estado antes de mostrar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Remoto {
    /// El `fetch` salio bien: se muestra lo ultimo publicado.
    Consultado,
    /// Sin red (o un fallo que no se reconoce): se muestra lo ultimo traido.
    SinRed,
    /// `origin` tiene la rama de estado reescrita (no desciende de la que
    /// trajo este clon) y el `fetch` sin `+` no la trae.
    Rechazado,
    /// La ref remota de este clon esta bloqueada por un `.lock` (otro git a
    /// la vez o uno huerfano); lleva la linea de git. Sale como
    /// `error-local`, con su propio aviso.
    RefBloqueada(String),
    /// `origin` responde pero no deja leer; el motivo es el de
    /// [`crate::git::FalloRemoto::motivo`] (`credenciales`,
    /// `remoto-no-existe`...).
    Denegado(String),
    /// Fallo del propio clon al traer (p. ej. sin permiso en `.git`).
    ErrorLocal,
    /// `origin` responde pero no tiene la rama de estado.
    SinRama,
    /// Sin `origin`: la rama local es el estado y no se trae nada.
    SinRemoto,
}

impl Remoto {
    /// Valor de `remoto=`.
    pub fn texto(&self) -> &'static str {
        match self {
            Self::Consultado => "consultado",
            Self::SinRed => "sin-red",
            Self::Rechazado => "rechazado",
            Self::Denegado(_) => "denegado",
            Self::ErrorLocal | Self::RefBloqueada(_) => "error-local",
            Self::SinRama => "sin-rama",
            Self::SinRemoto => "sin-remoto",
        }
    }

    /// Por que no es fresca, para el aviso; `None` si no hay nada que avisar.
    fn porque(&self) -> Option<Mensaje> {
        Some(match self {
            Self::Consultado | Self::SinRemoto => return None,
            Self::SinRed => Mensaje::nuevo(&txt::PORQUE_SIN_RED),
            Self::Rechazado => Mensaje::nuevo(&txt::PORQUE_RECHAZADO),
            Self::Denegado(m) => Mensaje::nuevo(&txt::PORQUE_DENEGADO).con("motivo", m.clone()),
            Self::ErrorLocal => Mensaje::nuevo(&txt::PORQUE_ERROR_LOCAL),
            Self::RefBloqueada(d) => {
                Mensaje::nuevo(&txt::PORQUE_REF_BLOQUEADA).con("detalle", d.clone())
            }
            Self::SinRama => Mensaje::nuevo(&txt::PORQUE_SIN_RAMA),
        })
    }
}

/// Lo que se sabe de la frescura de una lectura.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frescura {
    /// Como fue el intento de traer la rama de estado.
    pub remoto: Remoto,
    /// Ultimo `fetch` con exito (segundos desde 1970), si lo hubo.
    pub sincronizado: Option<u64>,
}

/// Valor de `sincronizado=`: la hora UTC o `nunca`.
pub fn sincronizado_texto(sincronizado: Option<u64>) -> String {
    sincronizado.map_or_else(|| "nunca".to_owned(), crate::alta::instante_utc)
}

/// «hace N» entre `entonces` y `ahora` (un `entonces` del futuro, por un
/// reloj desajustado, cuenta como «hace menos de 1 min»).
pub fn hace(entonces: u64, ahora: u64) -> Mensaje {
    let s = ahora.saturating_sub(entonces);
    match s {
        0..60 => Mensaje::nuevo(&txt::HACE_MENOS_DE_UN_MINUTO),
        60..3600 => Mensaje::nuevo(&txt::HACE_MINUTOS).con("n", (s / 60).to_string()),
        3600..172_800 => Mensaje::nuevo(&txt::HACE_HORAS).con("n", (s / 3600).to_string()),
        _ => Mensaje::nuevo(&txt::HACE_DIAS).con("n", (s / 86_400).to_string()),
    }
}

impl Frescura {
    /// Anade `remoto=` y `sincronizado=` al informe.
    #[must_use]
    pub fn anotar(&self, i: Informe) -> Informe {
        i.con("remoto", self.remoto.texto())
            .con("sincronizado", self.texto())
    }

    /// Valor de `sincronizado=`.
    pub fn texto(&self) -> String {
        sincronizado_texto(self.sincronizado)
    }

    /// Lo que se ensena en el texto para la persona (linea de `status` y
    /// cabecera de `board`): la hora, o `None` sin `origin` (no hay nada que
    /// sincronizar).
    pub fn en_texto(&self) -> Option<String> {
        (self.remoto != Remoto::SinRemoto).then(|| self.texto())
    }

    /// Aviso para stderr si la lectura no trajo lo ultimo publicado: por
    /// que, y desde cuando es lo que se muestra (o que no se sincronizo
    /// nunca). `None` si se consulto o no hay `origin`.
    pub fn aviso(&self, ahora: u64) -> Option<Mensaje> {
        let porque = self.remoto.porque()?;
        Some(match self.sincronizado {
            Some(s) => Mensaje::nuevo(&txt::AVISO_NO_FRESCO)
                .con_mensaje("porque", porque)
                .con("hora", crate::alta::instante_utc(s))
                .con_mensaje("hace", hace(s, ahora)),
            None => Mensaje::nuevo(&txt::AVISO_NUNCA_SINCRONIZADO).con_mensaje("porque", porque),
        })
    }

    /// Una linea para stderr con la hora de la ultima sincronizacion (la de
    /// `roadmap`, cuyo stdout es el ROADMAP byte a byte): el aviso si lo
    /// hay; si no, la hora; nada sin `origin`.
    pub fn linea(&self, ahora: u64) -> Option<Mensaje> {
        if let Some(a) = self.aviso(ahora) {
            return Some(a);
        }
        match (&self.remoto, self.sincronizado) {
            (Remoto::Consultado, Some(s)) => Some(
                Mensaje::nuevo(&txt::SINCRONIZADO)
                    .con("hora", crate::alta::instante_utc(s))
                    .con_mensaje("hace", hace(s, ahora)),
            ),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contrato::Resultado;
    use crate::i18n::Idioma;

    /// 2026-11-17T16:04:12Z.
    const T: u64 = 1_794_931_452;

    fn carpeta(nombre: &str) -> PathBuf {
        let d = std::env::temp_dir()
            .join("tc2-tests")
            .join(format!("frescura-{nombre}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn marca_ida_y_vuelta() -> std::io::Result<()> {
        let comun = carpeta("ida");
        let m = Marca::en(&comun);
        assert_eq!(m.ruta(), comun.join("taskcodetwo").join("sincronizacion"));
        assert_eq!(m.leer(), None, "sin fichero es nunca");
        m.registrar(T)?;
        assert_eq!(m.leer(), Some(T));
        m.registrar(T + 5)?;
        assert_eq!(m.leer(), Some(T + 5));
        // Sin temporales sueltos.
        let sueltos: Vec<_> = std::fs::read_dir(comun.join("taskcodetwo"))?
            .flatten()
            .map(|e| e.file_name())
            .collect();
        assert_eq!(sueltos.len(), 1, "{sueltos:?}");
        let _ = std::fs::remove_dir_all(&comun);
        Ok(())
    }

    #[test]
    fn marca_corrupta_o_enorme_es_nunca() -> std::io::Result<()> {
        let comun = carpeta("corrupta");
        let m = Marca::en(&comun);
        std::fs::create_dir_all(comun.join("taskcodetwo"))?;
        for malo in [
            "".to_owned(),
            "\n".to_owned(),
            "abc\n".to_owned(),
            "-5\n".to_owned(),
            "12 34\n".to_owned(),
            "99999999999999999999999\n".to_owned(),
            format!("{}\n", "1".repeat(100)),
        ] {
            std::fs::write(m.ruta(), &malo)?;
            assert_eq!(m.leer(), None, "{malo:?}");
        }
        std::fs::write(m.ruta(), "1794931452")?;
        assert_eq!(m.leer(), Some(T), "sin salto final tambien vale");
        let _ = std::fs::remove_dir_all(&comun);
        Ok(())
    }

    /// H2 (revision 1 de TASK-079): una marca con hora posterior a `ahora`
    /// es «nunca», tambien sin red: nunca mas fresca de lo real.
    #[test]
    fn marca_del_futuro_es_nunca() -> std::io::Result<()> {
        let comun = carpeta("futuro");
        let m = Marca::en(&comun);
        m.registrar(T + 1)?;
        assert_eq!(m.leer_en(T), None, "un segundo despues de ahora");
        assert_eq!(m.leer_en(T + 1), Some(T + 1), "igual a ahora vale");
        assert_eq!(m.leer_en(T + 100), Some(T + 1));
        m.registrar(99_999_999_999)?;
        assert_eq!(m.leer(), None, "el caso del informe: ano 5138");
        let f = Frescura {
            remoto: Remoto::SinRed,
            sincronizado: m.leer(),
        };
        assert_eq!(f.texto(), "nunca");
        let _ = std::fs::remove_dir_all(&comun);
        Ok(())
    }

    /// H1 (revision 1 de TASK-079): el stderr real de git 2.56 (sin `-q`)
    /// del rechazo non-fast-forward y del `.lock`, y que lo demas sigue con
    /// la clasificacion de siempre.
    #[test]
    fn clasifica_rechazo_y_ref_bloqueada() {
        let rechazo = "From ssh://git@github.com/o/r\n ! [rejected]        taskcodetwo/tareas -> origin/taskcodetwo/tareas  (non-fast-forward)\n";
        let lock = "error: cannot lock ref 'refs/remotes/origin/taskcodetwo/tareas': Unable to create 'C:/x/.git/refs/remotes/origin/taskcodetwo/tareas.lock': File exists.\n\nAnother git process seems to be running in this repository, or the lock file may be stale\nFrom ../remoto\n ! a5d3ff0..95116cd  taskcodetwo/tareas -> origin/taskcodetwo/tareas  (unable to update local ref)\n";
        let ambos = format!(
            "{rechazo}error: update_ref failed for ref 'refs/remotes/origin/taskcodetwo/tareas': cannot lock ref 'refs/remotes/origin/taskcodetwo/tareas': Unable to create 'C:/x/tareas.lock': File exists.\n"
        );
        let c = |s: &str| clasificar_traida(s.to_owned(), "p".to_owned());
        assert_eq!(
            c(rechazo),
            Traida::Reescrita {
                detalle: "! [rejected]        taskcodetwo/tareas -> origin/taskcodetwo/tareas  (non-fast-forward)".to_owned()
            }
        );
        assert!(
            matches!(c(&ambos), Traida::Reescrita { .. }),
            "manda el rechazo"
        );
        let bloqueada = c(lock);
        assert!(
            matches!(&bloqueada, Traida::Bloqueada { detalle }
                if detalle.starts_with("error: cannot lock ref 'refs/remotes/origin/")),
            "{bloqueada:?}"
        );
        assert_eq!(c(lock).remoto().texto(), "error-local");
        assert_eq!(c(rechazo).remoto().texto(), "rechazado");
        // Un `cannot lock ref` por permisos no es un `.lock`: sigue `Local`.
        let permiso = "error: cannot lock ref 'refs/heads/x': Permission denied";
        assert_eq!(c(permiso).remoto(), Remoto::ErrorLocal);
        assert_eq!(c("").remoto(), Remoto::SinRed);
        assert_eq!(
            c("fatal: couldn't find remote ref refs/heads/taskcodetwo/tareas"),
            Traida::SinRama
        );
        assert_eq!(
            c("git@github.com: Permission denied (publickey).\nfatal: Could not read from remote repository.").remoto(),
            Remoto::Denegado("credenciales".to_owned())
        );
    }

    /// Los avisos de los dos motivos nuevos, en los dos idiomas.
    #[test]
    fn aviso_de_rechazado_y_ref_bloqueada() {
        let f = |remoto| Frescura {
            remoto,
            sincronizado: Some(T),
        };
        let textos = |remoto, ahora| {
            let a = f(remoto).aviso(ahora);
            (
                a.as_ref().map(|m| m.texto(Idioma::Es)).unwrap_or_default(),
                a.map(|m| m.texto(Idioma::En)).unwrap_or_default(),
            )
        };
        let (es, en) = textos(Remoto::Rechazado, T + 180);
        assert_eq!(
            es,
            "aviso: origin tiene la rama de estado reescrita (no sigue a la que trajo este clon) y git no la trae sin forzar; se muestra lo que este clon trajo el 2026-11-17T16:04:12Z (hace 3 min)"
        );
        assert!(
            en.starts_with("warning: origin has the state branch rewritten"),
            "{en}"
        );
        let (es, en) = textos(
            Remoto::RefBloqueada("error: cannot lock ref 'x'".to_owned()),
            T,
        );
        assert_eq!(
            es,
            "aviso: la ref de la rama de estado de este clon esta bloqueada por otro git a la vez o un .lock huerfano (error: cannot lock ref 'x'); se muestra lo que este clon trajo el 2026-11-17T16:04:12Z (hace menos de 1 min)"
        );
        assert!(
            en.starts_with("warning: this clone's state branch ref is locked"),
            "{en}"
        );
        let base = Informe::nuevo("status", Resultado::Ok);
        assert_eq!(
            f(Remoto::Rechazado).anotar(base).linea(),
            "status ok remoto=rechazado sincronizado=2026-11-17T16:04:12Z"
        );
    }

    #[test]
    fn hace_en_minutos_horas_y_dias() {
        let es = |a: u64, b: u64| hace(a, b).texto(Idioma::Es);
        assert_eq!(es(T, T), "hace menos de 1 min");
        assert_eq!(es(T + 10, T), "hace menos de 1 min", "reloj desajustado");
        assert_eq!(es(T, T + 59), "hace menos de 1 min");
        assert_eq!(es(T, T + 60), "hace 1 min");
        assert_eq!(es(T, T + 3599), "hace 59 min");
        assert_eq!(es(T, T + 3600), "hace 1 h");
        assert_eq!(es(T, T + 172_799), "hace 47 h");
        assert_eq!(es(T, T + 3 * 86_400), "hace 3 dias");
        assert_eq!(hace(T, T + 120).texto(Idioma::En), "2 min ago");
    }

    /// `anotar` y `aviso` por cada `Remoto`, en los dos idiomas.
    #[test]
    fn anotar_y_aviso_por_cada_remoto() {
        let base = Informe::nuevo("status", Resultado::Ok);
        let casos = [
            (Remoto::Consultado, "consultado", None, None),
            (
                Remoto::SinRed,
                "sin-red",
                Some(
                    "aviso: sin red, no se pudo traer la rama de estado de origin; se muestra lo que este clon trajo el 2026-11-17T16:04:12Z (hace 3 min)",
                ),
                Some(
                    "warning: no network, the state branch could not be fetched from origin; showing what this clone fetched at 2026-11-17T16:04:12Z (3 min ago)",
                ),
            ),
            (
                Remoto::Denegado("credenciales".to_owned()),
                "denegado",
                Some(
                    "aviso: origin no deja leer la rama de estado (credenciales); se muestra lo que este clon trajo el 2026-11-17T16:04:12Z (hace 3 min)",
                ),
                Some(
                    "warning: origin does not allow reading the state branch (credenciales); showing what this clone fetched at 2026-11-17T16:04:12Z (3 min ago)",
                ),
            ),
            (
                Remoto::ErrorLocal,
                "error-local",
                Some(
                    "aviso: no se pudo traer la rama de estado por un fallo de este clon; se muestra lo que este clon trajo el 2026-11-17T16:04:12Z (hace 3 min)",
                ),
                Some(
                    "warning: the state branch could not be fetched because of a failure in this clone; showing what this clone fetched at 2026-11-17T16:04:12Z (3 min ago)",
                ),
            ),
            (
                Remoto::SinRama,
                "sin-rama",
                Some(
                    "aviso: origin no tiene la rama de estado; se muestra lo que este clon trajo el 2026-11-17T16:04:12Z (hace 3 min)",
                ),
                Some(
                    "warning: origin does not have the state branch; showing what this clone fetched at 2026-11-17T16:04:12Z (3 min ago)",
                ),
            ),
            (Remoto::SinRemoto, "sin-remoto", None, None),
        ];
        for (remoto, valor, es, en) in casos {
            let f = Frescura {
                remoto: remoto.clone(),
                sincronizado: Some(T),
            };
            assert_eq!(
                f.anotar(base.clone()).linea(),
                format!("status ok remoto={valor} sincronizado=2026-11-17T16:04:12Z")
            );
            let a = f.aviso(T + 180);
            assert_eq!(a.as_ref().map(|m| m.texto(Idioma::Es)).as_deref(), es);
            assert_eq!(a.as_ref().map(|m| m.texto(Idioma::En)).as_deref(), en);
            // Nunca sincronizado: `nunca` y, si habia que avisar, el aviso
            // de nunca.
            let nunca = Frescura {
                remoto,
                sincronizado: None,
            };
            assert_eq!(
                nunca.anotar(base.clone()).linea(),
                format!("status ok remoto={valor} sincronizado=nunca")
            );
            let a = nunca.aviso(T).map(|m| m.texto(Idioma::Es));
            assert_eq!(a.is_some(), es.is_some());
            if let Some(a) = a {
                assert!(
                    a.ends_with(
                        " y este clon no la ha sincronizado nunca; lo que se muestra puede no estar al dia"
                    ),
                    "{a}"
                );
            }
        }
    }

    #[test]
    fn en_texto_y_linea() {
        let f = |remoto, sincronizado| Frescura {
            remoto,
            sincronizado,
        };
        assert_eq!(
            f(Remoto::Consultado, Some(T)).en_texto().as_deref(),
            Some("2026-11-17T16:04:12Z")
        );
        assert_eq!(f(Remoto::SinRed, None).en_texto().as_deref(), Some("nunca"));
        assert_eq!(f(Remoto::SinRemoto, Some(T)).en_texto(), None);
        assert_eq!(
            f(Remoto::Consultado, Some(T))
                .linea(T + 30)
                .map(|m| m.texto(Idioma::Es))
                .as_deref(),
            Some("estado sincronizado con origin el 2026-11-17T16:04:12Z (hace menos de 1 min)")
        );
        assert!(
            f(Remoto::SinRed, Some(T))
                .linea(T)
                .is_some_and(|m| m.texto(Idioma::Es).starts_with("aviso: sin red"))
        );
        assert_eq!(f(Remoto::SinRemoto, None).linea(T), None);
    }
}
