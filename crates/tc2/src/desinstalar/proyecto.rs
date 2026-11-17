//! Inventario y plan de `desinstalar --proyecto`: los worktrees de la 2.0
//! (estado y tareas), su carpeta, la carpeta propia en la comun de git, las
//! lineas de `info/exclude`, las refs propias, la rama local de estado y la
//! configuracion. Los nombres salen de [`crate::huella`].

use std::path::{Path, PathBuf};
use std::time::Duration;

use super::texto::hooks_con_tc2;
use super::{Accion, Contexto, Decision, Efecto, NOMBRE, Opciones, leer, t};
use crate::contrato::{Informe, Resultado};
use crate::estado::Cerrojo;
use crate::git::estado::dir_comun;
use crate::git::rama::renombrar;
use crate::git::{
    CARPETA_WORKTREES, Git, REMOTO, Subcomando, Worktree, commit_de, es_ancestro, exigir,
    localizar_principal, misma_ruta, worktrees,
};
use crate::huella::{self, RAMA_ESTADO};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::desinstalar as txt;

/// Si la carpeta `dir` esta dentro de `ruta` (o es ella).
fn dentro(dir: &Path, ruta: &Path) -> bool {
    match (std::fs::canonicalize(dir), std::fs::canonicalize(ruta)) {
        (Ok(d), Ok(r)) => d.starts_with(r),
        _ => dir.starts_with(ruta),
    }
}

/// Por que un worktree no se puede quitar sin perder nada (o sin que falle
/// `git worktree remove` a mitad del plan), o `None`.
fn bloqueo(
    c: &Contexto<'_>,
    principal: &Path,
    w: &Worktree,
    de_tarea: bool,
) -> Result<Option<&'static str>, Informe> {
    let (git, ruta, rama) = (c.git, w.ruta.as_path(), w.rama.as_deref());
    if w.bloqueado {
        return Ok(Some("worktree-bloqueado"));
    }
    // Desde dentro, `git worktree remove` no puede borrar la carpeta actual.
    if dentro(c.dir, ruta) {
        return Ok(Some("carpeta-actual-dentro"));
    }
    let status = exigir(
        git,
        ruta,
        NOMBRE,
        Subcomando::Status,
        &["--porcelain", "-z"],
    )?;
    if !status.is_empty() {
        return Ok(Some("cambios-sin-guardar"));
    }
    if !de_tarea {
        // La rama de estado se conserva (salvo --borrar-tareas): sus commits
        // no se pierden al quitar el worktree.
        return Ok(None);
    }
    let Some(rama) = rama else {
        return Ok(Some("sin-rama"));
    };
    let publicada = match commit_de(
        git,
        principal,
        NOMBRE,
        &format!("refs/remotes/{REMOTO}/{rama}"),
    )? {
        Some(remota) => es_ancestro(git, ruta, NOMBRE, "HEAD", &remota)?,
        None => false,
    };
    Ok((!publicada).then_some("sin-publicar"))
}

pub(super) fn plan(op: &Opciones, c: &Contexto<'_>, v: &mut Vec<Accion>) -> Result<(), Informe> {
    let g = c.git;
    let principal = localizar_principal(g, c.dir).map_err(|i| renombrar(i, NOMBRE))?;
    let comun = dir_comun(g, &principal, NOMBRE)?;
    let carpeta_wt = principal.join(CARPETA_WORKTREES);
    let mut podar = false;
    let mut quitados: Vec<PathBuf> = Vec::new();
    for w in worktrees(g, &principal, NOMBRE)?.iter().skip(1) {
        let de_estado = w.rama.as_deref() == Some(RAMA_ESTADO);
        let de_tarea = w.ruta.parent().is_some_and(|p| misma_ruta(p, &carpeta_wt));
        if !de_estado && !de_tarea {
            continue;
        }
        if !w.ruta.exists() {
            podar = true;
            continue;
        }
        let clase = if de_estado {
            "worktree-estado"
        } else {
            "worktree-tarea"
        };
        match bloqueo(c, &principal, w, de_tarea)? {
            Some(m) => {
                let mut a =
                    Accion::nueva(t(&w.ruta), clase, "taskcodetwo", Decision::Conserva).motivo(m);
                a.bloquea = true;
                v.push(a);
            }
            None => {
                quitados.push(w.ruta.clone());
                v.push(Accion::borra(
                    t(&w.ruta),
                    clase,
                    Efecto::QuitarWorktree {
                        principal: principal.clone(),
                        ruta: w.ruta.clone(),
                    },
                ));
            }
        }
    }
    if podar {
        v.push(Accion::borra(
            t(&comun.join("worktrees")),
            "registro-worktree",
            Efecto::PodarWorktrees {
                principal: principal.clone(),
            },
        ));
    }
    // `.taskcodetwo/worktrees/` y `.taskcodetwo/`, si se quedan vacias.
    let raiz_wt = carpeta_wt.parent().map(Path::to_path_buf);
    for d in [Some(carpeta_wt.clone()), raiz_wt].into_iter().flatten() {
        if !d.is_dir() {
            continue;
        }
        let todo_se_va = std::fs::read_dir(&d)
            .map(|it| {
                it.flatten()
                    .all(|h| quitados.iter().any(|q| misma_ruta(q, &h.path())))
            })
            .unwrap_or(false);
        if todo_se_va {
            quitados.push(d.clone());
            v.push(Accion::borra(
                t(&d),
                "carpeta",
                Efecto::CarpetaSiVacia(d.clone()),
            ));
        } else {
            v.push(
                Accion::nueva(t(&d), "carpeta", "taskcodetwo", Decision::Conserva)
                    .motivo("no-vacia"),
            );
        }
    }
    // Lineas de `info/exclude` (inversa de `git::rama::excluir`).
    let exclude = comun.join("info").join("exclude");
    if let Some(bytes) = leer(&exclude)?
        && let Ok(texto) = String::from_utf8(bytes.clone())
    {
        let propias = huella::LINEAS_EXCLUDE;
        let nuevo: String = texto
            .split_inclusive('\n')
            .filter(|l| !propias.iter().any(|p| l.trim() == *p))
            .collect();
        if nuevo != texto {
            v.push(Accion::borra(
                t(&exclude),
                "exclude",
                Efecto::Reescribir {
                    ruta: exclude.clone(),
                    antes: bytes,
                    despues: nuevo.into_bytes(),
                },
            ));
        }
    }
    // La carpeta propia en la comun: cerrojo, cache de identidad, indices.
    let propia = huella::en_comun(&comun);
    if propia.is_dir() {
        v.push(Accion::borra(
            t(&propia),
            "carpeta-comun",
            Efecto::Carpeta(propia.clone()),
        ));
    }
    // Refs propias (operaciones pendientes y rechazadas): son tareas sin
    // publicar, asi que van con la rama (C7).
    let refs = exigir(
        g,
        &principal,
        NOMBRE,
        Subcomando::ForEachRef,
        &["--format=%(refname) %(objectname)", huella::PREFIJO_REFS],
    )?;
    for l in refs.lines() {
        let Some((r, sha)) = l.split_once(' ') else {
            continue;
        };
        if op.borrar_tareas.is_some() {
            v.push(Accion::borra(
                r,
                "ref-propia",
                Efecto::BorrarRef {
                    principal: principal.clone(),
                    referencia: r.to_owned(),
                    sha: sha.to_owned(),
                },
            ));
        } else {
            v.push(
                Accion::nueva(r, "ref-propia", "taskcodetwo", Decision::Conserva)
                    .motivo("sin-borrar-tareas"),
            );
        }
    }
    // La rama local de estado: solo con --borrar-tareas <nombre exacto>.
    let local = format!("refs/heads/{RAMA_ESTADO}");
    let remota = format!("refs/remotes/{REMOTO}/{RAMA_ESTADO}");
    let sha_remota = commit_de(g, &principal, NOMBRE, &remota)?;
    if let Some(sha) = commit_de(g, &principal, NOMBRE, &local)? {
        if op.borrar_tareas.is_some() {
            // Con commits que el remoto no tiene, se avisa y la linea lleva
            // el sha para recuperarlos (`git branch <rama> <sha>`).
            let publicada = match &sha_remota {
                Some(r) => es_ancestro(g, &principal, NOMBRE, &sha, r)?,
                None => false,
            };
            let mut a = Accion::borra(
                local,
                "rama-estado",
                Efecto::BorrarRamaEstado {
                    principal: principal.clone(),
                    sha: sha.clone(),
                },
            );
            if !publicada {
                a = a.motivo("adelantada-del-remoto");
                a.sha = Some(sha);
            }
            v.push(a);
        } else {
            v.push(
                Accion::nueva(local, "rama-estado", "taskcodetwo", Decision::Conserva)
                    .motivo("sin-borrar-tareas"),
            );
        }
    }
    // La remota nunca (D2): con --borrar-tareas, la orden para la persona.
    if sha_remota.is_some() {
        let a = Accion::nueva(remota, "rama-remota", "taskcodetwo", Decision::Conserva);
        v.push(if op.borrar_tareas.is_some() {
            let mut a = a.orden(format!("git push {REMOTO} --delete {RAMA_ESTADO}"));
            a.decision = Decision::Persona;
            a
        } else {
            a.motivo("remota")
        });
    }
    // Configuracion del proyecto: solo con --config (C6). Es de la 2.0 si
    // `config::leer` la reconoce (formato 2); la de la 0.16 no es suya y se
    // queda siempre, y con ella `personas.yml`, compartido (F1-12 D4a, D6).
    let config = principal.join(crate::config::RUTA);
    let mut de_la_016 = false;
    if let Some(bytes) = leer(&config)? {
        let de_la_2 = match crate::config::leer(&String::from_utf8_lossy(&bytes)) {
            Ok(_) => true,
            Err(e) => matches!(
                e.motivo,
                crate::config::Motivo::ConfigInvalida | crate::config::Motivo::FormatoNuevo
            ),
        };
        de_la_016 = !de_la_2;
        v.push(match (de_la_2, op.config) {
            (false, _) => Accion::nueva(t(&config), "config", "taskcode-016", Decision::Conserva),
            (true, true) => Accion::borra(t(&config), "config", Efecto::Fichero(config.clone())),
            (true, false) => Accion::nueva(t(&config), "config", "taskcodetwo", Decision::Conserva)
                .motivo("sin-config"),
        });
    }
    let personas = principal.join(crate::plataforma::personas::RUTA);
    if personas.is_file() {
        v.push(if de_la_016 {
            Accion::nueva(t(&personas), "config", "taskcode-016", Decision::Conserva)
        } else if op.config {
            Accion::borra(t(&personas), "config", Efecto::Fichero(personas.clone()))
        } else {
            Accion::nueva(t(&personas), "config", "taskcodetwo", Decision::Conserva)
                .motivo("sin-config")
        });
    }
    // Hooks del proyecto que nombran tc2: aviso, nunca se editan (C4).
    for nombre in ["settings.json", "settings.local.json"] {
        let p = principal.join(".claude").join(nombre);
        if let Some(bytes) = leer(&p)?
            && let Ok(valor) = serde_json::from_slice::<serde_json::Value>(&bytes)
        {
            for _ in hooks_con_tc2(&valor) {
                v.push(
                    Accion::nueva(
                        format!("{}#hooks", t(&p)),
                        "hook",
                        "ajeno",
                        Decision::Conserva,
                    )
                    .motivo("nombra-tc2"),
                );
            }
        }
    }
    Ok(())
}

/// Antes de aplicar: que ningun otro `tc2` este escribiendo el estado (el
/// mismo cerrojo que la transaccion). Solo si la carpeta propia existe: si
/// no, nadie lo tiene y no se crea.
pub(super) fn cerrojo_libre(c: &Contexto<'_>) -> Result<(), Informe> {
    let principal = localizar_principal(c.git, c.dir).map_err(|i| renombrar(i, NOMBRE))?;
    let comun = dir_comun(c.git, &principal, NOMBRE)?;
    if !huella::en_comun(&comun)
        .join(huella::FICHERO_CERROJO)
        .is_file()
    {
        return Ok(());
    }
    match Cerrojo::tomar(&comun, Duration::ZERO) {
        Ok(_) => Ok(()),
        Err(_) => Err(Informe::nuevo(NOMBRE, Resultado::Rechazada)
            .con("motivo", "cerrojo")
            .diagnostico(Mensaje::nuevo(&txt::CERROJO_OCUPADO))),
    }
}

/// Texto de un informe de error de git, para la linea de fallo.
fn texto(i: &Informe) -> String {
    i.diagnostico_en(crate::i18n::Idioma::Es)
        .unwrap_or_else(|| i.linea())
}

pub(super) fn quitar_worktree(git: &Git, principal: &Path, ruta: &Path) -> Result<(), String> {
    let r = ruta.to_str().ok_or_else(|| t(ruta))?;
    exigir(git, principal, NOMBRE, Subcomando::Worktree, &["remove", r])
        .map(|_| ())
        .map_err(|i| texto(&i))
}

pub(super) fn podar(git: &Git, principal: &Path) -> Result<(), String> {
    exigir(git, principal, NOMBRE, Subcomando::Worktree, &["prune"])
        .map(|_| ())
        .map_err(|i| texto(&i))
}

pub(super) fn borrar_ref(git: &Git, principal: &Path, r: &str, sha: &str) -> Result<(), String> {
    exigir(
        git,
        principal,
        NOMBRE,
        Subcomando::UpdateRef,
        &["-d", r, sha],
    )
    .map(|_| ())
    .map_err(|i| texto(&i))
}

pub(super) fn borrar_rama(git: &Git, principal: &Path, sha: &str) -> Result<(), String> {
    let s = git
        .borrar_rama_de_estado(principal, sha)
        .map_err(|e| e.to_string())?;
    if s.exito() {
        Ok(())
    } else {
        Err(s.primera_linea_error())
    }
}
