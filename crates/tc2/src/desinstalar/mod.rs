//! `tc2 desinstalar` (F2-14, RR-16): quita TaskCodeTwo de forma limpia.
//!
//! Dos ambitos explicitos, `--maquina` y `--proyecto` (sin ninguno, `uso`).
//! Cada uno hace un inventario de solo lectura con la huella encontrada y su
//! dueno, calcula el plan (una [`Accion`] por ruta) y lo aplica tal cual.
//! `--simular` imprime el plan y para (patron «plan como dato» de F1-12).
//! Ningun paso usa la red.
//!
//! Solo se borra lo que un reconocedor cerrado da por propio de la 2.0
//! (ruta exacta, marcas exactas o `command` exacto, con los nombres de
//! [`crate::huella`] y de [`distribucion`]). Lo de Claude Code, un gestor de
//! paquetes, otra herramienta o la 0.16 se lista, con la orden que lo quita
//! si la hay, y no se toca (D3, D6). En un proyecto, la config solo con
//! `--config` y la rama de estado local solo con `--borrar-tareas <rama>`;
//! la remota nunca (D1, D2).
//!
//! Salida: una linea `accion ruta=... clase=... dueno=... decision=...` por
//! accion y al final la linea de contrato `desinstalar <resultado> ...`.

mod maquina;
mod proyecto;
pub mod puertos;
pub mod texto;

use std::path::{Path, PathBuf};

use crate::Salida;
use crate::contrato::{Informe, LineaAccion, Resultado};
use crate::git::Git;
use crate::i18n::catalogo::comun::CAMBIO_DESDE_EL_PLAN;
use crate::i18n::catalogo::desinstalar as txt;
use crate::i18n::{Idioma, Mensaje};
use crate::instalacion::{Entorno, Sistema};
use crate::plan_dato::{self, Paso, externo, t};
pub use puertos::{Maquina, MaquinaEnMemoria, MaquinaReal};

/// Nombre de la orden.
pub const NOMBRE: &str = "desinstalar";

/// Opciones de la orden.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Opciones {
    /// Quitar lo de la maquina.
    pub maquina: bool,
    /// Quitar lo del proyecto de la carpeta actual.
    pub proyecto: bool,
    /// Solo imprimir el plan.
    pub simular: bool,
    /// Borrar tambien `.taskcode/config.yml` y `personas.yml`.
    pub config: bool,
    /// Borrar la rama local de estado (y las refs pendientes): el nombre
    /// exacto de la rama, como confirmacion.
    pub borrar_tareas: Option<String>,
}

/// Lee las opciones (sin `--json`).
pub fn leer_opciones(args: &[String]) -> Result<Opciones, Informe> {
    let uso = |motivo: &str| {
        Informe::nuevo(NOMBRE, Resultado::Uso)
            .con("motivo", motivo)
            .diagnostico(&txt::USO_DESINSTALAR)
    };
    let mut op = Opciones::default();
    let mut i = args.iter();
    while let Some(a) = i.next() {
        match a.as_str() {
            "--maquina" if !op.maquina => op.maquina = true,
            "--proyecto" if !op.proyecto => op.proyecto = true,
            "--simular" if !op.simular => op.simular = true,
            "--config" if !op.config => op.config = true,
            "--borrar-tareas" if op.borrar_tareas.is_none() => {
                let Some(r) = i.next() else {
                    return Err(uso("sin-rama"));
                };
                op.borrar_tareas = Some(r.clone());
            }
            _ => return Err(uso("argumentos")),
        }
    }
    if !op.maquina && !op.proyecto {
        return Err(uso("sin-ambito"));
    }
    if (op.config || op.borrar_tareas.is_some()) && !op.proyecto {
        return Err(uso("opcion-de-proyecto"));
    }
    if let Some(r) = &op.borrar_tareas
        && r != crate::huella::RAMA_ESTADO
    {
        return Err(Informe::nuevo(NOMBRE, Resultado::Uso)
            .con("motivo", "rama-no-coincide")
            .diagnostico(
                Mensaje::nuevo(&txt::NOMBRE_NO_COINCIDE)
                    .con("rama", crate::huella::RAMA_ESTADO)
                    .con("nombre", r.as_str()),
            ));
    }
    Ok(op)
}

/// Que se decide hacer con una ruta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Es de la 2.0 y se quita.
    Borra,
    /// Se queda (de otro, o falta el permiso de la persona).
    Conserva,
    /// La quita la persona con la orden de la linea.
    Persona,
    /// Es de la 2.0 pero no se pudo quitar ahora (en uso, o un fallo).
    Pendiente,
}

impl Decision {
    /// Valor de `decision=`.
    pub fn texto(self) -> &'static str {
        match self {
            Self::Borra => "borra",
            Self::Conserva => "conserva",
            Self::Persona => "persona",
            Self::Pendiente => "pendiente",
        }
    }
}

/// Lo que hace una accion `borra` al aplicarse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Efecto {
    /// Nada (lineas informativas).
    Ninguno,
    /// Borrar un fichero.
    Fichero(PathBuf),
    /// Borrar una carpeta con todo lo de dentro.
    Carpeta(PathBuf),
    /// Borrar una carpeta si ha quedado vacia.
    CarpetaSiVacia(PathBuf),
    /// Reescribir un fichero (si sigue siendo `antes`).
    Reescribir {
        /// El fichero.
        ruta: PathBuf,
        /// Lo que tenia al hacer el plan.
        antes: Vec<u8>,
        /// Lo que tendra.
        despues: Vec<u8>,
    },
    /// Cambiar el PATH de usuario de Windows.
    PathUsuario {
        /// Valor al hacer el plan.
        viejo: String,
        /// Valor nuevo.
        nuevo: String,
    },
    /// Borrar el binario en marcha tras salir (Windows).
    Autoborrado {
        /// Ficheros.
        ficheros: Vec<PathBuf>,
        /// Carpetas que se borran si quedan vacias.
        dirs: Vec<PathBuf>,
    },
    /// `git worktree remove <ruta>` (sin forzar).
    QuitarWorktree {
        /// Worktree principal.
        principal: PathBuf,
        /// Ruta del worktree.
        ruta: PathBuf,
    },
    /// `git worktree prune` (registros de worktrees ya borrados).
    PodarWorktrees {
        /// Worktree principal.
        principal: PathBuf,
    },
    /// `git update-ref -d <ref> <sha>` de una ref propia.
    BorrarRef {
        /// Worktree principal.
        principal: PathBuf,
        /// La ref.
        referencia: String,
        /// Su valor al hacer el plan.
        sha: String,
    },
    /// Borrar la rama local de estado (si sigue en `sha`).
    BorrarRamaEstado {
        /// Worktree principal.
        principal: PathBuf,
        /// Su valor al hacer el plan.
        sha: String,
    },
}

/// Una linea del plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Accion {
    /// Ruta (o ref, o entrada del PATH).
    pub ruta: String,
    /// Que es: `binario`, `perfil`, `status-line`, `worktree-estado`...
    pub clase: &'static str,
    /// De quien es: `taskcodetwo`, `claude-code`, `taskcode-016`, `gestor`,
    /// `ajeno`.
    pub dueno: &'static str,
    /// Que se hace.
    pub decision: Decision,
    /// Por que (si no es evidente).
    pub motivo: Option<String>,
    /// La orden que lo quita, cuando la decision es de la persona.
    pub orden: Option<String>,
    /// Commit que se pierde si se aplica (la rama de estado adelantada).
    pub sha: Option<String>,
    /// Si impide aplicar el plan (`rechazada` sin tocar nada).
    pub bloquea: bool,
    /// Lo que se hace al aplicar.
    pub efecto: Efecto,
}

impl Accion {
    /// Linea informativa sin efecto.
    pub(crate) fn nueva(
        ruta: impl Into<String>,
        clase: &'static str,
        dueno: &'static str,
        decision: Decision,
    ) -> Self {
        Self {
            ruta: ruta.into(),
            clase,
            dueno,
            decision,
            motivo: None,
            orden: None,
            sha: None,
            bloquea: false,
            efecto: Efecto::Ninguno,
        }
    }

    /// Accion `borra` de la 2.0 con su efecto.
    pub(crate) fn borra(ruta: impl Into<String>, clase: &'static str, efecto: Efecto) -> Self {
        let mut a = Self::nueva(ruta, clase, "taskcodetwo", Decision::Borra);
        a.efecto = efecto;
        a
    }

    #[must_use]
    pub(crate) fn motivo(mut self, m: impl Into<String>) -> Self {
        self.motivo = Some(m.into());
        self
    }

    #[must_use]
    pub(crate) fn orden(mut self, o: impl Into<String>) -> Self {
        self.orden = Some(o.into());
        self
    }

    /// Linea `accion ruta=... clase=... dueno=... decision=...` (formato
    /// comun de [`LineaAccion`]).
    pub fn linea(&self) -> String {
        self.comun(|l| l.texto())
    }

    /// La misma linea en JSON.
    pub fn json(&self) -> String {
        self.comun(|l| l.json())
    }

    fn comun(&self, f: impl Fn(&LineaAccion<'_>) -> String) -> String {
        f(&LineaAccion {
            ruta: &self.ruta,
            clase: self.clase,
            dueno: self.dueno,
            decision: self.decision.texto(),
            extra: &[
                ("motivo", self.motivo.as_deref()),
                ("orden", self.orden.as_deref()),
                ("sha", self.sha.as_deref()),
            ],
        })
    }

    /// Aviso para la persona por stderr (aunque la orden salga bien): la
    /// orden que tiene que ejecutar, o el hook que nombra `tc2`.
    fn aviso(&self) -> Option<Mensaje> {
        if let (Some("adelantada-del-remoto"), Some(sha)) = (self.motivo.as_deref(), &self.sha) {
            return Some(
                Mensaje::nuevo(&txt::AVISO_ADELANTADA)
                    .con("rama", crate::huella::RAMA_ESTADO)
                    .con("sha", sha.as_str()),
            );
        }
        if self.clase == "hook" {
            return Some(Mensaje::nuevo(&txt::AVISO_HOOK).con("ruta", self.ruta.as_str()));
        }
        match (&self.decision, &self.orden) {
            (Decision::Persona, Some(o)) => Some(
                Mensaje::nuevo(&txt::HAZ_TU)
                    .con("clase", self.clase)
                    .con("ruta", self.ruta.as_str())
                    .con("orden", o.as_str()),
            ),
            _ => None,
        }
    }
}

impl Paso for Accion {
    fn aplicable(&self) -> bool {
        self.decision == Decision::Borra
    }
    fn aplazar(&mut self, motivo: &str) {
        self.decision = Decision::Pendiente;
        self.motivo = Some(motivo.to_owned());
    }
    fn ruta(&self) -> &str {
        &self.ruta
    }
}

/// Lo que se necesita para desinstalar, inyectable en los tests.
pub struct Contexto<'a> {
    /// HOME, `LOCALAPPDATA`, el ejecutable...
    pub entorno: &'a Entorno,
    /// Carpeta del proyecto (para `--proyecto`).
    pub dir: &'a Path,
    /// Puerto Git.
    pub git: &'a Git,
    /// PATH de usuario, procesos y autoborrado.
    pub maquina: &'a dyn Maquina,
}

/// Resultado: las acciones (ya aplicadas, salvo con `--simular`) y la linea
/// final.
#[derive(Debug, Clone)]
pub struct Hecho {
    /// Una por ruta.
    pub acciones: Vec<Accion>,
    /// La linea de contrato final.
    pub informe: Informe,
}

impl Hecho {
    fn solo(informe: Informe) -> Self {
        Self {
            acciones: Vec::new(),
            informe,
        }
    }

    /// La salida del proceso: las lineas de accion y la final; en stderr, el
    /// diagnostico si fallo y los avisos para la persona.
    pub fn salida_en(&self, json: bool, idioma: Idioma) -> Salida {
        let fin = self.informe.salida_en(json, idioma);
        let mut stdout = String::new();
        let mut stderr = String::new();
        for a in &self.acciones {
            stdout.push_str(&if json { a.json() } else { a.linea() });
            stdout.push('\n');
            if let Some(m) = a.aviso() {
                stderr.push_str(&m.texto(idioma));
                stderr.push('\n');
            }
        }
        stdout.push_str(&fin.stdout);
        stderr.push_str(&fin.stderr);
        Salida {
            codigo: fin.codigo,
            stdout,
            stderr,
        }
    }
}

/// Orden `rm` para la persona, segun el sistema.
pub(crate) fn orden_borrar(sistema: Sistema, ruta: &Path) -> String {
    let r = ruta.display().to_string();
    match sistema {
        Sistema::Windows => format!(
            "Remove-Item -LiteralPath '{}' -Recurse -Force",
            r.replace('\'', "''")
        ),
        Sistema::Mac | Sistema::Linux => format!("rm -rf '{}'", r.replace('\'', "'\\''")),
    }
}

/// El plan completo: primero el proyecto y luego la maquina (el orden
/// recomendado).
pub fn plan(op: &Opciones, c: &Contexto<'_>) -> Result<Vec<Accion>, Informe> {
    let mut v = Vec::new();
    if op.proyecto {
        proyecto::plan(op, c, &mut v)?;
    }
    if op.maquina {
        maquina::plan(c, &mut v)?;
    }
    Ok(v)
}

/// `desinstalar` con las opciones ya leidas.
pub fn desinstalar(op: &Opciones, c: &Contexto<'_>) -> Hecho {
    let ambito = match (op.proyecto, op.maquina) {
        (true, true) => "proyecto,maquina",
        (true, false) => "proyecto",
        _ => "maquina",
    };
    let mut acciones = match plan(op, c) {
        Ok(v) => v,
        Err(i) => return Hecho::solo(i),
    };
    let base = |r: Resultado, acciones: &[Accion]| {
        let n = |d: Decision| acciones.iter().filter(|a| a.decision == d).count();
        let mut i = Informe::nuevo(NOMBRE, r).con("ambito", ambito);
        if op.simular {
            i = i.con("simular", "si");
        }
        i.con("borra", n(Decision::Borra).to_string())
            .con("conserva", n(Decision::Conserva).to_string())
            .con("persona", n(Decision::Persona).to_string())
            .con("pendiente", n(Decision::Pendiente).to_string())
    };
    if let Some(b) = acciones.iter().find(|a| a.bloquea).cloned() {
        // No se toca nada: lo que se habria borrado queda pendiente, y el
        // resumen no cuenta borrados que no hay.
        plan_dato::rechazar(&mut acciones);
        let motivo = b.motivo.clone().unwrap_or_default();
        let i = base(Resultado::Rechazada, &acciones)
            .con("motivo", motivo.clone())
            .con("ruta", b.ruta.clone())
            .diagnostico(
                Mensaje::nuevo(&txt::BLOQUEADA)
                    .con("ruta", b.ruta.clone())
                    .con("motivo", motivo),
            );
        return Hecho {
            acciones,
            informe: i,
        };
    }
    if !op.simular {
        if op.proyecto
            && let Err(i) = proyecto::cerrojo_libre(c)
        {
            return Hecho {
                acciones,
                informe: i,
            };
        }
        if let Some((ruta, e)) =
            plan_dato::aplicar_en_orden(&mut acciones, |a| aplicar(&a.efecto, c))
        {
            let informe = base(Resultado::Error, &acciones)
                .con("ruta", ruta.clone())
                .diagnostico(
                    Mensaje::nuevo(&txt::FALLO_AL_APLICAR)
                        .con("ruta", ruta)
                        .con_mensaje("error", e),
                );
            return Hecho { acciones, informe };
        }
    }
    // El que se nombra es el binario del grupo en uso, no su primera linea
    // (un temporal o el `.old`).
    let en_uso_de = |clase: Option<&str>| {
        acciones.iter().find(|a| {
            a.decision == Decision::Pendiente
                && a.motivo.as_deref() == Some("en-uso")
                && clase.is_none_or(|c| a.clase == c)
        })
    };
    let en_uso = en_uso_de(Some("binario")).or_else(|| en_uso_de(None));
    let informe = if let Some(a) = en_uso {
        let ruta = a.ruta.clone();
        base(Resultado::Rechazada, &acciones)
            .con("motivo", "en-uso")
            .con("ruta", ruta.clone())
            .diagnostico(Mensaje::nuevo(&txt::EN_USO).con("ruta", ruta))
    } else if acciones.iter().any(|a| a.decision == Decision::Borra) {
        base(Resultado::Ok, &acciones)
    } else {
        base(Resultado::SinCambios, &acciones)
    };
    Hecho { acciones, informe }
}

/// Aplica un efecto. `Err` con el texto del fallo.
fn aplicar(e: &Efecto, c: &Contexto<'_>) -> Result<(), Mensaje> {
    let no_existe = |r: std::io::Result<()>| match r {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        otro => otro.map_err(externo),
    };
    match e {
        Efecto::Ninguno => Ok(()),
        Efecto::Fichero(p) => no_existe(std::fs::remove_file(p)),
        Efecto::Carpeta(p) => no_existe(std::fs::remove_dir_all(p)),
        Efecto::CarpetaSiVacia(p) => match std::fs::read_dir(p).map(|mut d| d.next().is_none()) {
            Ok(true) => no_existe(std::fs::remove_dir(p)),
            // No ha quedado vacia (algo nuevo de la persona): se queda.
            Ok(false) => Ok(()),
            Err(e) => no_existe(Err(e)),
        },
        Efecto::Reescribir {
            ruta,
            antes,
            despues,
        } => reescribir(ruta, antes, despues),
        Efecto::PathUsuario { viejo, nuevo } => c
            .maquina
            .escribir_path_usuario(viejo, nuevo)
            .map_err(externo),
        Efecto::Autoborrado { ficheros, dirs } => c
            .maquina
            .programar_borrado(c.entorno.pid, ficheros, dirs)
            .map_err(externo),
        Efecto::QuitarWorktree { principal, ruta } => {
            proyecto::quitar_worktree(c.git, principal, ruta)
        }
        Efecto::PodarWorktrees { principal } => proyecto::podar(c.git, principal),
        Efecto::BorrarRef {
            principal,
            referencia,
            sha,
        } => proyecto::borrar_ref(c.git, principal, referencia, sha),
        Efecto::BorrarRamaEstado { principal, sha } => proyecto::borrar_rama(c.git, principal, sha),
    }
}

/// Cambia `ruta` de `antes` a `despues`; si la persona lo cambio despues
/// del plan, no se pisa.
fn reescribir(ruta: &Path, antes: &[u8], despues: &[u8]) -> Result<(), Mensaje> {
    let ahora = std::fs::read(ruta).map_err(externo)?;
    if ahora != antes {
        return Err(Mensaje::nuevo(&CAMBIO_DESDE_EL_PLAN).con("ruta", t(ruta)));
    }
    escribir_atomico(ruta, despues).map_err(externo)
}

/// Escribe `ruta` de una vez: un temporal en la misma carpeta y un
/// renombrado, para que un corte no deje el perfil o el `settings.json` de
/// la persona vacio. Si `ruta` es un enlace simbolico se escribe su destino
/// (como `cat >` de `install.sh`), y se conservan sus permisos.
fn escribir_atomico(ruta: &Path, contenido: &[u8]) -> std::io::Result<()> {
    let destino = if std::fs::symlink_metadata(ruta)?.file_type().is_symlink() {
        std::fs::canonicalize(ruta)?
    } else {
        ruta.to_path_buf()
    };
    let permisos = std::fs::metadata(&destino)?.permissions();
    let mut nombre = destino.file_name().unwrap_or_default().to_os_string();
    nombre.push(format!(".tc2-tmp-{}", std::process::id()));
    let tmp = destino.with_file_name(nombre);
    let r = std::fs::write(&tmp, contenido)
        .and_then(|()| std::fs::set_permissions(&tmp, permisos))
        .and_then(|()| std::fs::rename(&tmp, &destino));
    if r.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    r
}

/// La orden completa desde la linea de ordenes, en la carpeta actual y con
/// el entorno y la maquina de verdad.
pub fn orden(args: &[String], json: bool, idioma: Idioma) -> Salida {
    let op = match leer_opciones(args) {
        Ok(o) => o,
        Err(i) => return i.salida_en(json, idioma),
    };
    let entorno = match Entorno::del_proceso() {
        Ok(e) => e,
        Err(e) => {
            return Informe::nuevo(NOMBRE, Resultado::Error)
                .diagnostico(Mensaje::nuevo(&txt::SIN_ENTORNO).con("error", e.to_string()))
                .salida_en(json, idioma);
        }
    };
    let dir = match crate::tablero::carpeta_actual(NOMBRE) {
        Ok(d) => d,
        Err(i) => return i.salida_en(json, idioma),
    };
    let git = Git::nuevo();
    let maquina = MaquinaReal;
    let c = Contexto {
        entorno: &entorno,
        dir: &dir,
        git: &git,
        maquina: &maquina,
    };
    desinstalar(&op, &c).salida_en(json, idioma)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opciones() {
        let v = |a: &[&str]| a.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        assert_eq!(
            leer_opciones(&v(&["--proyecto", "--borrar-tareas", "taskcodetwo/tareas"])),
            Ok(Opciones {
                proyecto: true,
                borrar_tareas: Some("taskcodetwo/tareas".into()),
                ..Opciones::default()
            })
        );
        for (args, motivo) in [
            (v(&[]), "sin-ambito"),
            (v(&["--simular"]), "sin-ambito"),
            (v(&["--maquina", "--maquina"]), "argumentos"),
            (v(&["--maquina", "--config"]), "opcion-de-proyecto"),
            (v(&["--proyecto", "--borrar-tareas"]), "sin-rama"),
            (
                v(&["--proyecto", "--borrar-tareas", "tareas"]),
                "rama-no-coincide",
            ),
            (v(&["--proyecto", "x"]), "argumentos"),
        ] {
            let i = leer_opciones(&args).err();
            assert_eq!(
                i.as_ref().map(|i| (i.resultado, i.valor("motivo"))),
                Some((Resultado::Uso, Some(motivo))),
                "{args:?}"
            );
        }
    }

    /// A7-07: un fichero que cambio desde el plan no se pisa, y el
    /// diagnostico sale entero en el idioma pedido (con `en`, sin un trozo
    /// en espanol en `{error}`).
    #[test]
    fn cambio_desde_el_plan_en_ingles() -> std::io::Result<()> {
        let dir = std::env::temp_dir()
            .join("tc2-tests")
            .join(format!("desinstalar-reescribir-{}", std::process::id()));
        std::fs::create_dir_all(&dir)?;
        let ruta = dir.join(".profile");
        std::fs::write(&ruta, "cambiado por la persona\n")?;
        let r = reescribir(&ruta, b"original\n", b"");
        let ahora = std::fs::read_to_string(&ruta);
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(ahora?, "cambiado por la persona\n");
        let e = r.err().map(|m| {
            Mensaje::nuevo(&txt::FALLO_AL_APLICAR)
                .con("ruta", t(&ruta))
                .con_mensaje("error", m)
                .texto(Idioma::En)
        });
        let r = t(&ruta);
        assert_eq!(
            e,
            Some(format!(
                "could not remove {r}: {r} changed after the plan was made. What the previous lines list is already removed: fix the cause and run tc2 desinstalar again"
            ))
        );
        Ok(())
    }

    #[test]
    fn linea_y_json_de_una_accion() {
        let a = Accion::nueva("C:/a b", "plugin", "claude-code", Decision::Persona)
            .orden("claude plugin uninstall taskcodetwo@m");
        assert_eq!(
            a.linea(),
            "accion ruta=C:/a%20b clase=plugin dueno=claude-code decision=persona orden=claude%20plugin%20uninstall%20taskcodetwo@m"
        );
        assert_eq!(
            a.json(),
            "{\"accion\":{\"ruta\":\"C:/a b\",\"clase\":\"plugin\",\"dueno\":\"claude-code\",\"decision\":\"persona\",\"orden\":\"claude plugin uninstall taskcodetwo@m\"}}"
        );
        assert_eq!(
            orden_borrar(Sistema::Linux, Path::new("/a'b")),
            "rm -rf '/a'\\''b'"
        );
    }
}
