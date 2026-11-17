//! `tc2 init` (F3-1): deja un repo listo para TaskCodeTwo.
//!
//! Sigue el patron de `tc2 desinstalar` («plan como dato», F1-12 PL1): un
//! inventario de solo lectura calcula el plan (una [`Accion`] por artefacto,
//! con su decision) y la ejecucion aplica ese mismo plan; `--simular` lo
//! imprime y para. Lo que se crea sale de los nombres de [`crate::huella`],
//! los mismos que lee `desinstalar`, asi que se reconocen sin mas.
//!
//! Que crea, solo si falta: la configuracion minima, las lineas de
//! `.gitignore` y de `info/exclude`, `main` y `develop` con un commit raiz
//! (solo en un repo sin commits; con historia lo hace la persona), la rama
//! de estado montada en `tareas/` y `personas.yml` con la persona del token.
//! El workflow de CI queda como ranura `pendiente` hasta F5-1 y los rulesets
//! `taskcodetwo-*` solo se proponen (`persona`): aplicarlos es de TASK-118.
//!
//! Nunca pisa la 0.16 ni una configuracion que no vale: `rechazada` sin
//! escribir nada. Con `origin` y sin red, `sinred pendiente=no` sin escribir.
//!
//! Salida: una linea `accion ruta=... clase=... dueno=... decision=...` por
//! accion y al final `init <resultado> [simular=si] crea=N existe=N
//! persona=N pendiente=N`.

mod aplicar;
mod inventario;

use std::path::{Path, PathBuf};

use crate::Salida;
use crate::contrato::{Informe, LineaAccion, Resultado};
use crate::git::Git;
use crate::i18n::catalogo::init as txt;
use crate::i18n::{Idioma, Mensaje};
use crate::plataforma::Credenciales;

/// Nombre de la orden.
pub const NOMBRE: &str = "init";

/// Orden que aplicara las puertas (TASK-118).
pub const ORDEN_PUERTAS: &str = "tc2 init --aplicar-puertas";

/// Arreglo para un proyecto de la 0.16.
pub const ARREGLO_016: &str = "tc2 actualizar --simular";

/// Opciones de la orden.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Opciones {
    /// Solo imprimir el plan.
    pub simular: bool,
}

/// Lee las opciones (sin `--json`).
pub fn leer_opciones(args: &[String]) -> Result<Opciones, Informe> {
    let mut op = Opciones::default();
    for a in args {
        match a.as_str() {
            "--simular" if !op.simular => op.simular = true,
            _ => {
                return Err(Informe::nuevo(NOMBRE, Resultado::Uso)
                    .con("motivo", "argumentos")
                    .diagnostico(&txt::USO_INIT));
            }
        }
    }
    Ok(op)
}

/// Que se decide hacer con un artefacto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Falta y lo crea `init`.
    Crea,
    /// Ya esta: no se toca.
    Existe,
    /// Lo hace la persona con la orden de la linea.
    Persona,
    /// No se puede hacer ahora (sin remoto, sin token, sin plantilla...).
    Pendiente,
    /// Impide aplicar el plan: `rechazada` sin tocar nada.
    Rechaza,
}

impl Decision {
    /// Valor de `decision=`.
    pub fn texto(self) -> &'static str {
        match self {
            Self::Crea => "crea",
            Self::Existe => "existe",
            Self::Persona => "persona",
            Self::Pendiente => "pendiente",
            Self::Rechaza => "rechaza",
        }
    }
}

/// Lo que hace una accion `crea` al aplicarse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Efecto {
    /// Nada (la hace otra linea, o es informativa).
    Ninguno,
    /// Crear un fichero que no existe.
    Escribir {
        /// El fichero.
        ruta: PathBuf,
        /// Su contenido.
        contenido: String,
    },
    /// Anadir a un fichero las lineas que le falten.
    Anadir {
        /// El fichero.
        ruta: PathBuf,
        /// Las lineas.
        lineas: Vec<String>,
    },
    /// Anadir lineas a `info/exclude` (`git::rama::excluir`).
    Excluir {
        /// Worktree principal.
        principal: PathBuf,
    },
    /// Commit raiz con `ficheros` y las ramas `ramas` en el (repo vacio).
    RamasRaiz {
        /// Worktree principal.
        principal: PathBuf,
        /// Rutas relativas que lleva el commit, leidas del disco al aplicar.
        ficheros: Vec<String>,
        /// Ramas que se crean.
        ramas: Vec<String>,
        /// HEAD esta sin nacer en una de ellas: se pone el indice al dia.
        head: bool,
        /// Se publican en `origin`.
        publicar: bool,
    },
    /// `estado::montar`.
    MontarEstado {
        /// Worktree principal.
        principal: PathBuf,
    },
}

/// Una linea del plan.
#[derive(Debug, Clone, PartialEq)]
pub struct Accion {
    /// Ruta, ref o nombre.
    pub ruta: String,
    /// Que es: `config`, `gitignore`, `rama`, `worktree-estado`...
    pub clase: &'static str,
    /// De quien es: `taskcodetwo` o `proyecto`.
    pub dueno: &'static str,
    /// Que se hace.
    pub decision: Decision,
    /// Por que (si no es evidente).
    pub motivo: Option<String>,
    /// La orden de la persona.
    pub orden: Option<String>,
    /// Propuesta para la plataforma (el ruleset), solo en JSON.
    pub propuesta: Option<serde_json::Value>,
    /// Que decir a la persona si bloquea.
    pub(crate) diagnostico: Option<Mensaje>,
    /// Lo que se hace al aplicar.
    pub(crate) efecto: Efecto,
}

impl Accion {
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
            propuesta: None,
            diagnostico: None,
            efecto: Efecto::Ninguno,
        }
    }

    /// Accion `crea` de la 2.0 con su efecto.
    pub(crate) fn crea(ruta: impl Into<String>, clase: &'static str, efecto: Efecto) -> Self {
        let mut a = Self::nueva(ruta, clase, "taskcodetwo", Decision::Crea);
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

    fn comun(&self) -> LineaAccion<'_> {
        LineaAccion {
            ruta: &self.ruta,
            clase: self.clase,
            dueno: self.dueno,
            decision: self.decision.texto(),
            extra: &[],
        }
    }

    /// Linea `accion ruta=... clase=... dueno=... decision=...`.
    pub fn linea(&self) -> String {
        LineaAccion {
            extra: &[
                ("motivo", self.motivo.as_deref()),
                ("orden", self.orden.as_deref()),
            ],
            ..self.comun()
        }
        .texto()
    }

    /// La misma linea en JSON, con la propuesta si la hay.
    pub fn json(&self) -> String {
        let mut s = LineaAccion {
            extra: &[
                ("motivo", self.motivo.as_deref()),
                ("orden", self.orden.as_deref()),
            ],
            ..self.comun()
        }
        .json();
        if let Some(p) = &self.propuesta
            && let Some(sin_cierre) = s.strip_suffix("}}")
        {
            s = format!("{sin_cierre},\"propuesta\":{p}}}}}");
        }
        s
    }
}

/// Lo que necesita `init`, inyectable en los tests.
pub struct Contexto<'a> {
    /// Carpeta del proyecto.
    pub dir: &'a Path,
    /// Puerto Git.
    pub git: &'a Git,
    /// Tokens por rol (se piden al construir el adaptador de la plataforma).
    pub credenciales: &'a dyn Fn() -> Box<dyn Credenciales>,
}

/// Resultado: las acciones (ya aplicadas, salvo con `--simular`) y la linea
/// final.
#[derive(Debug, Clone)]
pub struct Hecho {
    /// Una por artefacto.
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
    /// diagnostico si fallo y un aviso si hay acciones de la persona.
    pub fn salida_en(&self, json: bool, idioma: Idioma) -> Salida {
        let fin = self.informe.salida_en(json, idioma);
        let mut stdout = String::new();
        for a in &self.acciones {
            stdout.push_str(&if json { a.json() } else { a.linea() });
            stdout.push('\n');
        }
        stdout.push_str(&fin.stdout);
        let mut stderr = String::new();
        let personas = self
            .acciones
            .iter()
            .filter(|a| a.decision == Decision::Persona)
            .count();
        if personas > 0 && self.informe.resultado.es_exito() {
            stderr.push_str(
                &Mensaje::nuevo(&txt::AVISO_PERSONA)
                    .con("n", personas.to_string())
                    .texto(idioma),
            );
            stderr.push('\n');
        }
        stderr.push_str(&fin.stderr);
        Salida {
            codigo: fin.codigo,
            stdout,
            stderr,
        }
    }
}

/// Ruta como texto para la linea.
pub(crate) fn t(p: &Path) -> String {
    p.display().to_string()
}

/// Informe de error al leer `ruta`.
pub(crate) fn no_se_lee(ruta: &Path, e: &std::io::Error) -> Informe {
    Informe::nuevo(NOMBRE, Resultado::Error)
        .con("ruta", t(ruta))
        .diagnostico(
            Mensaje::nuevo(&txt::NO_SE_LEE)
                .con("ruta", t(ruta))
                .con("error", e.to_string()),
        )
}

/// Lee un fichero; `None` si no existe.
pub(crate) fn leer(p: &Path) -> Result<Option<Vec<u8>>, Informe> {
    match std::fs::read(p) {
        Ok(b) => Ok(Some(b)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(no_se_lee(p, &e)),
    }
}

/// El plan, sin aplicar nada.
pub fn plan(c: &Contexto<'_>) -> Result<Vec<Accion>, Informe> {
    inventario::plan(c)
}

/// `init` con las opciones ya leidas.
pub fn init(op: Opciones, c: &Contexto<'_>) -> Hecho {
    let mut acciones = match plan(c) {
        Ok(v) => v,
        Err(i) => return Hecho::solo(i),
    };
    let cuenta = |r: Resultado, acciones: &[Accion], cabeza: &[(&str, String)]| {
        let n = |d: Decision| acciones.iter().filter(|a| a.decision == d).count();
        let mut i = Informe::nuevo(NOMBRE, r);
        for (k, v) in cabeza {
            i = i.con(k, v.clone());
        }
        if op.simular {
            i = i.con("simular", "si");
        }
        i.con("crea", n(Decision::Crea).to_string())
            .con("existe", n(Decision::Existe).to_string())
            .con("persona", n(Decision::Persona).to_string())
            .con("pendiente", n(Decision::Pendiente).to_string())
    };
    if let Some(b) = acciones
        .iter()
        .find(|a| a.decision == Decision::Rechaza)
        .cloned()
    {
        // No se toca nada: lo que se habria creado queda pendiente.
        for a in acciones.iter_mut().filter(|a| a.decision == Decision::Crea) {
            a.decision = Decision::Pendiente;
            a.motivo = Some("no-aplicada".to_owned());
        }
        let motivo = b.motivo.clone().unwrap_or_default();
        let mut cabeza = vec![("motivo", motivo.clone())];
        if let Some(o) = &b.orden {
            cabeza.push(("arreglo", o.clone()));
        }
        cabeza.push(("ruta", b.ruta.clone()));
        let diag = b.diagnostico.clone().unwrap_or_else(|| {
            Mensaje::nuevo(&txt::BLOQUEADA)
                .con("ruta", b.ruta.clone())
                .con("motivo", motivo)
        });
        let informe = cuenta(Resultado::Rechazada, &acciones, &cabeza).diagnostico(diag);
        return Hecho { acciones, informe };
    }
    if !op.simular {
        // (ruta, error) del primer fallo; lo que venia detras no se aplica.
        let mut fallo: Option<(String, String)> = None;
        for a in acciones.iter_mut() {
            if a.decision != Decision::Crea {
                continue;
            }
            if fallo.is_some() {
                a.decision = Decision::Pendiente;
                a.motivo = Some("no-aplicada".to_owned());
                continue;
            }
            if let Err(e) = aplicar::aplicar(&a.efecto, c) {
                a.decision = Decision::Pendiente;
                a.motivo = Some("error".to_owned());
                fallo = Some((a.ruta.clone(), e));
            }
        }
        if let Some((ruta, e)) = fallo {
            let informe = cuenta(Resultado::Error, &acciones, &[("ruta", ruta.clone())])
                .diagnostico(
                    Mensaje::nuevo(&txt::FALLO_AL_APLICAR)
                        .con("ruta", ruta)
                        .con("error", e),
                );
            return Hecho { acciones, informe };
        }
    }
    let r = if acciones.iter().any(|a| a.decision == Decision::Crea) {
        Resultado::Ok
    } else {
        Resultado::SinCambios
    };
    let informe = cuenta(r, &acciones, &[]);
    Hecho { acciones, informe }
}

/// La orden completa desde la linea de ordenes, en la carpeta actual y con
/// los tokens del proceso.
pub fn orden(args: &[String], json: bool, idioma: Idioma) -> Salida {
    let op = match leer_opciones(args) {
        Ok(o) => o,
        Err(i) => return i.salida_en(json, idioma),
    };
    let dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            return Informe::nuevo(NOMBRE, Resultado::Error)
                .diagnostico(
                    Mensaje::nuevo(&crate::i18n::catalogo::cli::CARPETA_ACTUAL)
                        .con("error", e.to_string()),
                )
                .salida_en(json, idioma);
        }
    };
    let git = Git::nuevo();
    let credenciales =
        || -> Box<dyn Credenciales> { Box::new(crate::plataforma::CredencialesEntorno) };
    let c = Contexto {
        dir: &dir,
        git: &git,
        credenciales: &credenciales,
    };
    init(op, &c).salida_en(json, idioma)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opciones() {
        let v = |a: &[&str]| a.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        assert_eq!(leer_opciones(&v(&[])), Ok(Opciones { simular: false }));
        assert_eq!(
            leer_opciones(&v(&["--simular"])),
            Ok(Opciones { simular: true })
        );
        for args in [v(&["--simular", "--simular"]), v(&["--dry-run"]), v(&["x"])] {
            let i = leer_opciones(&args).err();
            assert_eq!(
                i.as_ref().map(|i| (i.resultado, i.valor("motivo"))),
                Some((Resultado::Uso, Some("argumentos"))),
                "{args:?}"
            );
        }
    }

    #[test]
    fn linea_y_json_con_propuesta() {
        let mut a = Accion::nueva(
            "taskcodetwo-main",
            "ruleset",
            "taskcodetwo",
            Decision::Persona,
        )
        .motivo("aplicar-con-task-118")
        .orden(ORDEN_PUERTAS);
        assert_eq!(
            a.linea(),
            "accion ruta=taskcodetwo-main clase=ruleset dueno=taskcodetwo decision=persona motivo=aplicar-con-task-118 orden=tc2%20init%20--aplicar-puertas"
        );
        a.propuesta = Some(serde_json::json!({"name": "taskcodetwo-main"}));
        let v: serde_json::Value = serde_json::from_str(&a.json()).unwrap_or_default();
        assert_eq!(v["accion"]["propuesta"]["name"], "taskcodetwo-main");
        assert_eq!(v["accion"]["orden"], ORDEN_PUERTAS);
    }
}
