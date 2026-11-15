//! Montaje de la rama de estado en `tareas/` (ADR F1-1, decision 2).
//!
//! La rama huerfana se crea con un commit raiz hecho con `commit-tree` y
//! publicado por la misma transaccion que cualquier escritura: no hace falta
//! `worktree add --orphan` (Git 2.42), y si dos clones la crean a la vez el
//! segundo recibe non-fast-forward, re-ejecuta, la ve creada y solo la monta.

use std::path::Path;

use super::cerrojo::{Cerrojo, ErrorCerrojo};
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

const OP: &str = "montar-estado";

/// Fichero que deja el commit raiz.
pub(crate) const LEEME: &str = "LEEME.md";

const TEXTO_LEEME: &str = "# Rama de estado de TaskCodeTwo\n\nEsta rama (`taskcode/tareas`) guarda las tareas y el roadmap (ADR F1-1).\nNo tiene historia comun con las ramas de codigo: no la mezcles con ellas.\nLa escribe el CLI; se monta como worktree en `tareas/`.\n";

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
                .diagnostico_sin_traducir("otro proceso de este clon esta montando o escribiendo la rama de estado: reintenta"));
        }
        Err(ErrorCerrojo::Io(e)) => {
            return Err(Informe::nuevo(OP, Resultado::Error)
                .diagnostico_sin_traducir(format!("cerrojo: {e}")));
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

    for w in worktrees(git, &principal, OP)?
        .iter()
        .filter(|w| w.rama.as_deref() == Some(RAMA))
    {
        if !w.ruta.exists() {
            exigir(git, &principal, OP, Subcomando::Worktree, &["prune"])?;
        } else if misma_ruta(&w.ruta, &carpeta) {
            excluir(git, &principal, OP, &format!("/{CARPETA}/"))?;
            return Ok(ok(Resultado::SinCambios, None));
        } else {
            return Ok(Informe::nuevo(OP, Resultado::Rechazada)
                .con("rama", RAMA)
                .con("motivo", "montada-en-otra-ruta")
                .diagnostico_sin_traducir(format!(
                    "{RAMA} ya esta montada en {}: quita ese worktree",
                    w.ruta.display()
                )));
        }
    }
    let ocupada = std::fs::read_dir(&carpeta)
        .map(|mut d| d.next().is_some())
        .unwrap_or(carpeta.exists());
    if ocupada {
        return Ok(Informe::nuevo(OP, Resultado::Rechazada)
            .con("ruta", CARPETA)
            .con("motivo", "tareas-ocupada")
            .diagnostico_sin_traducir(format!(
                "{CARPETA}/ ya existe y no es la rama de estado (¿un repo de la 0.16? migralo con actualizar)"
            )));
    }

    let remoto = hay_remoto(git, &principal, OP)?;
    let local = format!("refs/heads/{RAMA}");
    let remota = format!("refs/remotes/{REMOTO}/{RAMA}");

    if commit_de(git, &principal, OP, &local)?.is_none() {
        let mut en_remoto = false;
        if remoto {
            let refspec = format!("refs/heads/{RAMA}:{remota}");
            let s = correr(
                git,
                &principal,
                OP,
                Subcomando::Fetch,
                &["-q", REMOTO, &refspec],
            )?;
            if s.exito() {
                en_remoto = true;
            } else {
                match clasificar_fallo_remoto(&s.stderr) {
                    FalloRemoto::RamaInexistente => {}
                    FalloRemoto::SinRed => {
                        return Ok(Informe::nuevo(OP, Resultado::Sinred)
                            .con("pendiente", "no")
                            .diagnostico_sin_traducir(format!(
                                "{}; montar la rama de estado exige red la primera vez",
                                s.primera_linea_error()
                            )));
                    }
                    FalloRemoto::Local => {
                        return Err(crate::git::remoto::error_local(
                            OP,
                            &s.primera_linea_error(),
                        ));
                    }
                    otro => {
                        return Ok(Informe::nuevo(OP, Resultado::Denegada)
                            .con("motivo", otro.motivo())
                            .diagnostico_sin_traducir(format!(
                                "{}; revisa credenciales, permisos o la URL de origin",
                                s.primera_linea_error()
                            )));
                    }
                }
            }
        }
        if en_remoto {
            exigir(git, &principal, OP, Subcomando::Branch, &[RAMA, &remota])?;
        } else {
            if !puede_crear {
                return Err(Informe::nuevo(OP, Resultado::Error)
                    .diagnostico_sin_traducir(format!("{RAMA} sigue sin existir tras crearla")));
            }
            // La transaccion toma el cerrojo ella misma: se suelta antes y,
            // creada la rama, se vuelve a empezar para montarla.
            drop(cerrojo);
            let mut c = Contexto::nuevo(git, &principal, &ninguna);
            c.politica = politica;
            let i = transaccion(&c, &Inicializar);
            if !i.resultado.es_exito() {
                return Ok(renombrado(i));
            }

            return paso(git, dir, politica, false);
        }
    }

    let ruta = carpeta.to_str().ok_or_else(|| {
        Informe::nuevo(OP, Resultado::Error)
            .diagnostico_sin_traducir("la ruta de tareas/ no es UTF-8")
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
