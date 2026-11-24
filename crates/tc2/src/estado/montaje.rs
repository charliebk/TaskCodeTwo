//! Montaje de la rama de estado en `tareas/` (ADR F1-1, decision 2).
//!
//! La rama huerfana se crea con un commit raiz hecho con `commit-tree` y
//! publicado por la misma transaccion que cualquier escritura: no hace falta
//! `worktree add --orphan` (Git 2.42), y si dos clones la crean a la vez el
//! segundo recibe non-fast-forward, re-ejecuta, la ve creada y solo la monta.
//!
//! [`previsto`] dice, sin escribir nada, que haria [`montar`]; `montar` parte
//! de esa misma clasificacion, asi que `tc2 init --simular` (F3-1) y la
//! ejecucion no se pueden separar.

use std::path::{Path, PathBuf};

use super::cerrojo::{Cerrojo, ErrorCerrojo};
use super::frescura::{Marca, Traida, rama_reescrita, traer_estado};
use super::lectura::Estado;
use super::transaccion::{Contexto, Fallo, Operacion, Politica, transaccion};
use super::{CARPETA, RAMA};
use crate::contrato::{Informe, Resultado, corto};
use crate::git::estado::{dir_comun, hay_remoto};
use crate::git::rama::excluir;
use crate::git::{
    Cambio, FalloRemoto, Git, REMOTO, Subcomando, clasificar_fallo_remoto, commit_de, correr,
    es_ancestro, exigir, localizar_principal, misma_ruta, worktrees,
};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::estado as txt;

const OP: &str = "montar-estado";

/// Fichero que deja el commit raiz.
pub(crate) const LEEME: &str = "LEEME.md";

/// Texto del [`LEEME`], compuesto desde el nombre de la rama.
use crate::huella::TEXTO_LEEME;

/// Operacion que crea la rama: solo escribe [`LEEME`] si no esta.
struct Inicializar;

impl Operacion for Inicializar {
    fn nombre(&self) -> &str {
        "inicializar"
    }
    fn anotacion(&self) -> String {
        "inicializar".to_owned()
    }
    fn exige_red(&self) -> bool {
        true
    }
    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        if estado.rutas().iter().any(|r| r == LEEME) {
            Ok(Vec::new())
        } else {
            Ok(vec![Cambio::nuevo(LEEME, TEXTO_LEEME)])
        }
    }
}

fn ninguna(_: &str) -> Option<Box<dyn Operacion>> {
    None
}

fn renombrado(mut i: Informe) -> Informe {
    OP.clone_into(&mut i.comando);
    i
}

/// Lo que haria [`montar`] ahora mismo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Previsto {
    /// Ya montada en `tareas/`: nada que hacer.
    Montada,
    /// La rama local existe y falta montarla.
    Local,
    /// La rama solo esta en el remoto: se trae y se monta.
    DelRemoto,
    /// No esta en ningun sitio: se crea (y se publica si hay remoto).
    Crear,
    /// `tareas/` existe con contenido que no es la rama (un repo de la 0.16).
    Ocupada,
    /// La rama esta montada en otra carpeta.
    OtraRuta(PathBuf),
    /// No se pudo hablar con el remoto (el detalle de git).
    SinRed(String),
    /// El remoto deniega la lectura (credenciales, no existe...).
    Denegada(FalloRemoto, String),
}

/// Que haria [`montar`] en el repo de `dir`, sin escribir nada: lee los
/// worktrees, la carpeta, las ramas y, si hace falta, el remoto con
/// `ls-remote` (que no escribe refs, a diferencia de `fetch`).
pub fn previsto(git: &Git, dir: &Path) -> Result<Previsto, Informe> {
    let principal = localizar_principal(git, dir)?;
    previsto_en(git, &principal)
}

fn previsto_en(git: &Git, principal: &Path) -> Result<Previsto, Informe> {
    let carpeta = principal.join(CARPETA);
    for w in worktrees(git, principal, OP)?
        .iter()
        .filter(|w| w.rama.as_deref() == Some(RAMA))
    {
        // Un registro cuya carpeta ya no existe no cuenta: `montar` lo poda.
        if !w.ruta.exists() {
            continue;
        }
        return Ok(if misma_ruta(&w.ruta, &carpeta) {
            Previsto::Montada
        } else {
            Previsto::OtraRuta(w.ruta.clone())
        });
    }
    let ocupada = std::fs::read_dir(&carpeta)
        .map(|mut d| d.next().is_some())
        .unwrap_or(carpeta.exists());
    if ocupada {
        return Ok(Previsto::Ocupada);
    }
    if commit_de(git, principal, OP, &format!("refs/heads/{RAMA}"))?.is_some() {
        return Ok(Previsto::Local);
    }
    if !hay_remoto(git, principal, OP)? {
        return Ok(Previsto::Crear);
    }
    let s = correr(
        git,
        principal,
        OP,
        Subcomando::LsRemote,
        &["--heads", REMOTO, &format!("refs/heads/{RAMA}")],
    )?;
    if s.exito() {
        return Ok(if s.stdout.trim().is_empty() {
            Previsto::Crear
        } else {
            Previsto::DelRemoto
        });
    }
    match clasificar_fallo_remoto(&s.stderr) {
        FalloRemoto::RamaInexistente => Ok(Previsto::Crear),
        FalloRemoto::SinRed => Ok(Previsto::SinRed(s.primera_linea_error())),
        FalloRemoto::Local => Err(crate::git::remoto::error_local(
            OP,
            &s.primera_linea_error(),
        )),
        otro => Ok(Previsto::Denegada(otro, s.primera_linea_error())),
    }
}

/// Crea la rama de estado si no existe (local, en el remoto o en ningun
/// sitio) y la monta en `tareas/` del worktree principal. Idempotente: ya
/// montada → `sin-cambios`. Si la carpeta se borro (tambien con
/// `git clean -ffdx`), la vuelve a montar. `tareas/` ocupada por otra cosa
/// (un repo de la 0.16) → `rechazada motivo=tareas-ocupada`.
pub fn montar(git: &Git, dir: &Path) -> Informe {
    montar_con(git, dir, Politica::default())
}

/// Como [`montar`], con otra politica de reintentos para crear la rama.
pub fn montar_con(git: &Git, dir: &Path, politica: Politica) -> Informe {
    paso(git, dir, politica, true).unwrap_or_else(renombrado)
}

/// `puede_crear`: si puede crear la rama con una transaccion; tras crearla
/// se vuelve a entrar una vez, ya sin poder crear, para montarla.
fn paso(git: &Git, dir: &Path, politica: Politica, puede_crear: bool) -> Result<Informe, Informe> {
    let principal = localizar_principal(git, dir)?;
    // El mismo cerrojo que la transaccion: la persona y sus agentes del mismo
    // clon no montan a la vez (dos `worktree add` en la misma carpeta).
    let comun = dir_comun(git, &principal, OP)?;
    let cerrojo = match Cerrojo::tomar(&comun, politica.espera_cerrojo) {
        Ok(c) => c,
        Err(ErrorCerrojo::Ocupado) => {
            return Ok(Informe::nuevo(OP, Resultado::Agotada)
                .con("motivo", "cerrojo")
                .diagnostico(&txt::CERROJO_OCUPADO_AL_MONTAR));
        }
        Err(ErrorCerrojo::Io(e)) => {
            return Err(Informe::nuevo(OP, Resultado::Error)
                .diagnostico(Mensaje::nuevo(&txt::CERROJO_ERROR).con("error", e.to_string())));
        }
    };
    let carpeta = principal.join(CARPETA);
    let ok = |r: Resultado, commit: Option<String>| {
        let i = Informe::nuevo(OP, r).con("rama", RAMA).con("ruta", CARPETA);
        match commit {
            Some(c) => i.con("commit", corto(&c)),
            None => i,
        }
    };

    let previsto = previsto_en(git, &principal)?;
    match &previsto {
        Previsto::Montada => {
            excluir(git, &principal, OP, &format!("/{CARPETA}/"))?;
            return Ok(ok(Resultado::SinCambios, None));
        }
        Previsto::OtraRuta(ruta) => {
            return Ok(Informe::nuevo(OP, Resultado::Rechazada)
                .con("rama", RAMA)
                .con("motivo", "montada-en-otra-ruta")
                .diagnostico(
                    Mensaje::nuevo(&txt::MONTADA_EN_OTRA_RUTA)
                        .con("rama", RAMA)
                        .con("ruta", ruta.display().to_string()),
                ));
        }
        Previsto::Ocupada => {
            return Ok(Informe::nuevo(OP, Resultado::Rechazada)
                .con("ruta", CARPETA)
                .con("motivo", "tareas-ocupada")
                .diagnostico(Mensaje::nuevo(&txt::CARPETA_OCUPADA).con("carpeta", CARPETA)));
        }
        Previsto::SinRed(detalle) => return Ok(sin_red(detalle)),
        Previsto::Denegada(f, detalle) => return Ok(denegada(*f, detalle)),
        Previsto::Local | Previsto::DelRemoto | Previsto::Crear => {}
    }
    // Un registro de la rama cuya carpeta se borro impide volver a montarla.
    if worktrees(git, &principal, OP)?
        .iter()
        .any(|w| w.rama.as_deref() == Some(RAMA) && !w.ruta.exists())
    {
        exigir(git, &principal, OP, Subcomando::Worktree, &["prune"])?;
    }

    let remoto = hay_remoto(git, &principal, OP)?;
    let local = format!("refs/heads/{RAMA}");
    let remota = format!("refs/remotes/{REMOTO}/{RAMA}");

    let mut crear = previsto == Previsto::Crear;
    if previsto == Previsto::DelRemoto {
        // El mismo `fetch` que la transaccion y la lectura fresca: con
        // exito renueva la marca de la ultima sincronizacion.
        let marca = Marca::en(&comun);
        match traer_estado(git, &principal, &marca, crate::identidad::ahora())? {
            Traida::Hecha => {
                exigir(git, &principal, OP, Subcomando::Branch, &[RAMA, &remota])?;
            }
            // Desaparecio del remoto entre la lectura y el fetch.
            Traida::SinRama => crear = true,
            Traida::Reescrita { detalle } => return Err(rama_reescrita(OP, &detalle)),
            Traida::Bloqueada { detalle } => {
                return Err(crate::git::remoto::error_local(OP, &detalle));
            }
            Traida::Fallo { fallo, primera, .. } => match fallo {
                FalloRemoto::SinRed => return Ok(sin_red(&primera)),
                FalloRemoto::Local => {
                    return Err(crate::git::remoto::error_local(OP, &primera));
                }
                otro => return Ok(denegada(otro, &primera)),
            },
        }
    }
    if crear {
        if !puede_crear {
            return Err(Informe::nuevo(OP, Resultado::Error)
                .diagnostico(Mensaje::nuevo(&txt::RAMA_SIGUE_SIN_EXISTIR).con("rama", RAMA)));
        }
        // La transaccion toma el cerrojo ella misma: se suelta antes y,
        // creada la rama, se vuelve a empezar para montarla.
        drop(cerrojo);
        // Inicializar no lleva ROADMAP: el primero lo escribe la primera
        // operacion (con la zona de por defecto).
        let mut c = Contexto::nuevo(git, &principal, &ninguna, None);
        c.politica = politica;
        let i = transaccion(&c, &Inicializar);
        if !i.resultado.es_exito() {
            return Ok(renombrado(i));
        }
        return paso(git, dir, politica, false);
    }

    let ruta = carpeta.to_str().ok_or_else(|| {
        Informe::nuevo(OP, Resultado::Error).diagnostico(&txt::RUTA_TAREAS_NO_UTF8)
    })?;
    exigir(
        git,
        &principal,
        OP,
        Subcomando::Worktree,
        &["add", "-q", ruta, RAMA],
    )?;
    excluir(git, &principal, OP, &format!("/{CARPETA}/"))?;
    if remoto
        && let Some(r) = commit_de(git, &principal, OP, &remota)?
        && let Some(l) = commit_de(git, &principal, OP, &local)?
        && l != r
        && es_ancestro(git, &principal, OP, &l, &r)?
    {
        // Lo publicado va por delante de la rama local: fast-forward.
        let _ = correr(
            git,
            &carpeta,
            OP,
            Subcomando::Merge,
            &["-q", "--ff-only", &r],
        );
    }
    let head = commit_de(git, &carpeta, OP, "HEAD")?;
    Ok(ok(Resultado::Ok, head))
}

fn sin_red(detalle: &str) -> Informe {
    Informe::nuevo(OP, Resultado::Sinred)
        .con("pendiente", "no")
        .diagnostico(Mensaje::nuevo(&txt::MONTAR_EXIGE_RED).con("error", detalle))
}

fn denegada(f: FalloRemoto, detalle: &str) -> Informe {
    Informe::nuevo(OP, Resultado::Denegada)
        .con("motivo", f.motivo())
        .diagnostico(Mensaje::nuevo(&txt::MONTAR_DENEGADA).con("error", detalle))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::prueba::{R, aislar, sh};
    use std::process::{Command, Stdio};

    fn utf8(p: &Path) -> R<&str> {
        Ok(p.to_str().ok_or("ruta no UTF-8")?)
    }

    /// Carpeta temporal propia de cada test (se borra al empezar y al acabar).
    fn carpeta(nombre: &str) -> PathBuf {
        let d = std::env::temp_dir()
            .join("tc2-tests")
            .join(format!("montaje-{nombre}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    /// Commit en el bare con un fichero `f` de contenido `texto` y los padres
    /// dados (ninguno: huerfano).
    fn commit_en_bare(raiz: &Path, bare: &Path, texto: &str, padres: &[&str]) -> R<String> {
        let tmp = bare.join("blob-temporal");
        std::fs::write(&tmp, texto)?;
        let blob = sh(raiz, bare, &["hash-object", "-w", utf8(&tmp)?])?;
        std::fs::remove_file(&tmp)?;
        let mut c = Command::new("git")
            .current_dir(bare)
            .arg("mktree")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        {
            use std::io::Write as _;
            let mut e = c.stdin.take().ok_or("sin stdin")?;
            writeln!(e, "100644 blob {blob}\tf")?;
        }
        let s = c.wait_with_output()?;
        if !s.status.success() {
            return Err("mktree".into());
        }
        let arbol = String::from_utf8_lossy(&s.stdout).trim().to_owned();
        let mut args = vec!["commit-tree", arbol.as_str(), "-m", texto];
        for p in padres {
            args.push("-p");
            args.push(p);
        }
        sh(raiz, bare, &args)
    }

    /// Un bare con `main` y la rama de estado, y un clon que solo tiene
    /// `refs/remotes/origin/taskcodetwo/tareas` (sin rama local). Devuelve
    /// (raiz, bare, clon, commit de la rama de estado).
    fn escenario(nombre: &str) -> R<(PathBuf, PathBuf, PathBuf, String)> {
        let raiz = carpeta(nombre);
        let bare = raiz.join("remoto.git");
        let clon = raiz.join("clon");
        std::fs::create_dir_all(&bare)?;
        aislar(&raiz)?;
        sh(&raiz, &bare, &["init", "-q", "--bare"])?;
        let c_main = commit_en_bare(&raiz, &bare, "main", &[])?;
        sh(&raiz, &bare, &["update-ref", "refs/heads/main", &c_main])?;
        sh(&raiz, &bare, &["symbolic-ref", "HEAD", "refs/heads/main"])?;
        let c_estado = commit_en_bare(&raiz, &bare, "estado", &[])?;
        sh(
            &raiz,
            &bare,
            &["update-ref", &format!("refs/heads/{RAMA}"), &c_estado],
        )?;
        sh(&raiz, &raiz, &["clone", "-q", utf8(&bare)?, utf8(&clon)?])?;
        assert_eq!(
            sh(
                &raiz,
                &clon,
                &["rev-parse", &format!("refs/remotes/{REMOTO}/{RAMA}")]
            )?,
            c_estado
        );
        assert!(!rama_local(&clon)?, "el clon no tiene la rama local");
        assert_eq!(
            previsto(&Git::nuevo(), &clon).map_err(|i| i.linea())?,
            Previsto::DelRemoto
        );
        Ok((raiz, bare, clon, c_estado))
    }

    fn rama_local(clon: &Path) -> R<bool> {
        Ok(Command::new("git")
            .current_dir(clon)
            .args(["rev-parse", "--verify", "-q", &format!("refs/heads/{RAMA}")])
            .output()?
            .status
            .success())
    }

    fn politica_rapida() -> Politica {
        Politica {
            espera_cerrojo: std::time::Duration::from_millis(200),
            ..Politica::default()
        }
    }

    /// `montar` sale con `error motivo=<motivo>` sin dejar pendiente, sin
    /// crear la rama local y sin montar `tareas/`. Dos pasadas:
    ///
    /// - con `intentos: 0`, la transaccion de crear la rama responderia `uso
    ///   motivo=sin-intentos` sin hacer `fetch`: asi el error tiene que
    ///   salir del brazo de `montar` y no del mismo brazo de la transaccion
    ///   (con la politica normal, si `montar` pasara a crear, la transaccion
    ///   repetiria el `fetch` y daria el mismo informe: la mutacion
    ///   `Reescrita/Bloqueada => crear = true` seguiria en verde);
    /// - con la politica normal, de punta a punta.
    fn montar_falla(clon: &Path, motivo: &str) -> R<()> {
        let sin_transaccion = Politica {
            intentos: 0,
            ..politica_rapida()
        };
        for politica in [sin_transaccion, politica_rapida()] {
            let i = montar_con(&Git::nuevo(), clon, politica);
            assert_eq!(i.comando, OP);
            assert_eq!(i.resultado, Resultado::Error, "{}", i.linea());
            assert_eq!(i.valor("motivo"), Some(motivo), "{}", i.linea());
            assert_eq!(i.valor("pendiente"), None, "{}", i.linea());
            assert!(!rama_local(clon)?, "no crea la rama local");
            assert!(!clon.join(CARPETA).exists(), "no monta tareas/");
        }
        Ok(())
    }

    /// M5 (revision 2 de TASK-079): la rama de estado reescrita en `origin`
    /// (un commit huerfano) entre la lectura y el `fetch`: `error
    /// motivo=rama-reescrita`, sin crear la rama local ni dejar pendiente.
    #[test]
    fn montar_con_rama_reescrita_es_error() -> R<()> {
        let (raiz, bare, clon, _) = escenario("reescrita")?;
        let huerfano = commit_en_bare(&raiz, &bare, "reescrita", &[])?;
        sh(
            &raiz,
            &bare,
            &["update-ref", &format!("refs/heads/{RAMA}"), &huerfano],
        )?;

        montar_falla(&clon, "rama-reescrita")?;
        // El bare sigue con el huerfano: no se ha publicado nada.
        assert_eq!(
            sh(&raiz, &bare, &["rev-parse", &format!("refs/heads/{RAMA}")])?,
            huerfano
        );
        let _ = std::fs::remove_dir_all(&raiz);
        Ok(())
    }

    /// M5 (revision 2 de TASK-079): un `.lock` huerfano en la ref remota del
    /// clon: `error motivo=error-local`, sin crear la rama local.
    #[test]
    fn montar_con_ref_bloqueada_es_error_local() -> R<()> {
        let (raiz, bare, clon, c_estado) = escenario("bloqueada")?;
        // El remoto avanza (fast-forward) para que el `fetch` tenga que
        // escribir la ref remota.
        let siguiente = commit_en_bare(&raiz, &bare, "siguiente", &[&c_estado])?;
        sh(
            &raiz,
            &bare,
            &["update-ref", &format!("refs/heads/{RAMA}"), &siguiente],
        )?;
        let lock = clon
            .join(".git")
            .join("refs")
            .join("remotes")
            .join(REMOTO)
            .join(format!("{RAMA}.lock"));
        std::fs::create_dir_all(lock.parent().ok_or("sin padre")?)?;
        std::fs::write(&lock, "")?;

        montar_falla(&clon, "error-local")?;
        assert!(lock.exists(), "el lock ajeno no se toca");
        assert_eq!(
            sh(&raiz, &bare, &["rev-parse", &format!("refs/heads/{RAMA}")])?,
            siguiente
        );
        let _ = std::fs::remove_dir_all(&raiz);
        Ok(())
    }
}
