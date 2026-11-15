//! Operaciones con el remoto: `fetch`, `push` con reintento,
//! `actualizar-rama` y `actualizar-principal`.

use std::path::Path;
use std::time::Duration;

use super::rama::renombrar;
use super::{
    Git, Paso, REMOTO, Subcomando, base_de_rama, commit_de, correr, entradas_status, es_ancestro,
    es_protegida, exigir, localizar_principal, rama_actual,
};
use crate::contrato::{Informe, Resultado, corto};

/// Esperas entre intentos de `push`: tres intentos, a 1 s y 2 s.
pub const ESPERAS_PUSH: [Duration; 2] = [Duration::from_secs(1), Duration::from_secs(2)];

/// Huella de las ramas remotas, para saber si el `fetch` trajo algo.
fn refs_remotas(git: &Git, dir: &Path, op: &str) -> Paso<String> {
    exigir(
        git,
        dir,
        op,
        Subcomando::ForEachRef,
        &[
            "--format=%(refname) %(objectname)",
            &format!("refs/remotes/{REMOTO}"),
        ],
    )
}

fn sinred(op: &str) -> Informe {
    Informe::nuevo(op, Resultado::Sinred)
        .con("remoto", REMOTO)
        .diagnostico(format!("{op}: no se pudo leer {REMOTO}; reintenta con red"))
}

/// Trae el remoto. Nada nuevo → `sin-cambios`; sin remoto → `sinred`.
pub fn fetch(git: &Git, dir: &Path) -> Informe {
    traer(git, dir, "fetch").unwrap_or_else(|i| i)
}

fn traer(git: &Git, dir: &Path, op: &str) -> Paso<Informe> {
    let antes = refs_remotas(git, dir, op)?;
    let s = correr(git, dir, op, Subcomando::Fetch, &["-q", REMOTO])?;
    if !s.exito() {
        return Err(sinred(op));
    }
    let despues = refs_remotas(git, dir, op)?;
    let r = if antes == despues {
        Resultado::SinCambios
    } else {
        Resultado::Ok
    };
    Ok(Informe::nuevo(op, r).con("remoto", REMOTO))
}

/// Como termino un intento de push, leido de `git push --porcelain`.
enum Intento {
    Publicado,
    AlDia,
    NoFastForward,
    Fallo(String),
}

fn intento_push(git: &Git, dir: &Path, op: &str, refspec: &str) -> Paso<Intento> {
    let s = correr(
        git,
        dir,
        op,
        Subcomando::Push,
        &["--porcelain", REMOTO, refspec],
    )?;
    // Linea de porcelana: `<flag>\t<origen>:<destino>\t<resumen>`.
    let linea = s
        .stdout
        .lines()
        .find(|l| l.contains('\t') && l.split('\t').nth(1).is_some_and(|r| r.contains(':')));
    let flag = linea.and_then(|l| l.chars().next());
    let resumen = linea.and_then(|l| l.split('\t').nth(2)).unwrap_or("");
    Ok(match flag {
        Some('=') if s.exito() => Intento::AlDia,
        Some(' ' | '*') if s.exito() => Intento::Publicado,
        Some('!') if resumen.starts_with("[rejected]") => Intento::NoFastForward,
        _ => Intento::Fallo(if resumen.is_empty() {
            s.primera_linea_error()
        } else {
            resumen.to_owned()
        }),
    })
}

/// Publica `refspec` con reintentos ante fallos transitorios. Un
/// non-fast-forward no se reintenta. Devuelve el resultado y los intentos.
pub(crate) fn publicar(
    git: &Git,
    dir: &Path,
    op: &str,
    refspec: &str,
    esperas: &[Duration],
) -> Paso<(Resultado, usize, String)> {
    let mut ultimo = String::new();
    for intento in 0..=esperas.len() {
        match intento_push(git, dir, op, refspec)? {
            Intento::AlDia => return Ok((Resultado::SinCambios, intento + 1, String::new())),
            Intento::Publicado => return Ok((Resultado::Ok, intento + 1, String::new())),
            Intento::NoFastForward => {
                return Ok((Resultado::Rechazada, intento + 1, String::new()));
            }
            Intento::Fallo(m) => ultimo = m,
        }
        if let Some(e) = esperas.get(intento) {
            std::thread::sleep(*e);
        }
    }
    Ok((Resultado::Sinred, esperas.len() + 1, ultimo))
}

/// Publica la rama actual en `origin` con el mismo nombre, sin force.
/// `esperas` fija los reintentos (en produccion, [`ESPERAS_PUSH`]).
pub fn push(git: &Git, dir: &Path, esperas: &[Duration]) -> Informe {
    push_paso(git, dir, "push", esperas).unwrap_or_else(|i| i)
}

fn push_paso(git: &Git, dir: &Path, op: &str, esperas: &[Duration]) -> Paso<Informe> {
    let rama = rama_de_tarea(git, dir, op)?;
    let head = commit_de(git, dir, op, "HEAD")?.unwrap_or_default();
    let refspec = format!("refs/heads/{rama}:refs/heads/{rama}");
    let (r, intentos, ultimo) = publicar(git, dir, op, &refspec, esperas)?;
    let i = Informe::nuevo(op, r).con("rama", rama.as_str());
    Ok(match r {
        Resultado::Ok => i
            .con("commit", corto(&head))
            .con("intentos", intentos.to_string()),
        Resultado::SinCambios => i.con("commit", corto(&head)),
        Resultado::Rechazada => i
            .con("motivo", "no-fast-forward")
            .diagnostico("lanza actualizar-rama"),
        _ => i
            .con("intentos", intentos.to_string())
            .diagnostico(format!("push: {ultimo}; reintenta con red")),
    })
}

/// La rama actual, si es de tarea (ni HEAD suelto ni `develop`/`main`).
fn rama_de_tarea(git: &Git, dir: &Path, op: &str) -> Paso<String> {
    match rama_actual(git, dir, op)? {
        None => Err(Informe::nuevo(op, Resultado::Rechazada)
            .con("motivo", "sin-rama")
            .diagnostico("lanzalo en el worktree de la tarea")),
        Some(r) if es_protegida(&r) => Err(Informe::nuevo(op, Resultado::Rechazada)
            .con("rama", r.as_str())
            .con("motivo", "rama-protegida")
            .diagnostico("lanzalo en el worktree de la tarea, no en develop ni en main")),
        Some(r) => Ok(r),
    }
}

/// Ficheros con conflicto sin resolver.
pub(crate) fn sin_resolver(git: &Git, dir: &Path, op: &str) -> Paso<Vec<String>> {
    let s = exigir(
        git,
        dir,
        op,
        Subcomando::Diff,
        &["--name-only", "--diff-filter=U", "-z"],
    )?;
    Ok(s.split('\0')
        .filter(|f| !f.is_empty())
        .map(str::to_owned)
        .collect())
}

/// Si hay un merge abierto (`MERGE_HEAD`).
pub(crate) fn merge_abierto(git: &Git, dir: &Path, op: &str) -> Paso<bool> {
    Ok(commit_de(git, dir, op, "MERGE_HEAD")?.is_some())
}

/// Lleva a la rama de la tarea lo nuevo de su base (y lo que otra persona
/// haya publicado en la misma rama) con merge, **sin rebase**, y publica.
/// Conflicto → `conflicto` (6) con los ficheros; repetirlo no vuelve a
/// mergear. Se resuelve y se lanza `guardar`.
pub fn actualizar_rama(git: &Git, dir: &Path, esperas: &[Duration]) -> Informe {
    actualizar(git, dir, esperas).unwrap_or_else(|i| i)
}

fn actualizar(git: &Git, dir: &Path, esperas: &[Duration]) -> Paso<Informe> {
    const OP: &str = "actualizar-rama";
    let rama = rama_de_tarea(git, dir, OP)?;
    let base = base_de_rama(&rama);
    let conflicto = |ficheros: Vec<String>| {
        let diag = if ficheros.is_empty() {
            "lanza guardar con los ficheros resueltos".to_owned()
        } else {
            format!("resuelve {} y lanza guardar", ficheros.join(" "))
        };
        Informe::nuevo(OP, Resultado::Conflicto)
            .con("base", base)
            .con_lista("ficheros", ficheros)
            .diagnostico(diag)
    };
    if merge_abierto(git, dir, OP)? {
        return Ok(conflicto(sin_resolver(git, dir, OP)?));
    }
    let pendientes = exigir(
        git,
        dir,
        OP,
        Subcomando::Status,
        &["--porcelain", "-z", "--untracked-files=no"],
    )?;
    if !entradas_status(&pendientes).is_empty() {
        return Ok(Informe::nuevo(OP, Resultado::Rechazada)
            .con("motivo", "cambios-sin-guardar")
            .diagnostico("guarda los cambios con guardar antes de actualizar la rama"));
    }
    traer(git, dir, OP)?;
    if commit_de(git, dir, OP, base)?.is_none() {
        return Ok(Informe::nuevo(OP, Resultado::Rechazada)
            .con("base", base)
            .con("motivo", "base-no-existe")
            .diagnostico(format!("{base} no existe en el remoto")));
    }
    let remota = format!("{REMOTO}/{rama}");
    let hay_remota = commit_de(git, dir, OP, &remota)?.is_some();
    let mut mergeado = false;
    let fuentes: Vec<&str> = if hay_remota {
        vec![remota.as_str(), base]
    } else {
        vec![base]
    };
    for fuente in fuentes {
        if es_ancestro(git, dir, OP, fuente, "HEAD")? {
            continue;
        }
        let s = correr(
            git,
            dir,
            OP,
            Subcomando::Merge,
            &["-q", "--no-edit", "--no-ff", fuente],
        )?;
        if !s.exito() {
            if merge_abierto(git, dir, OP)? {
                return Ok(conflicto(sin_resolver(git, dir, OP)?));
            }
            return Ok(Informe::nuevo(OP, Resultado::Error)
                .con("base", base)
                .diagnostico(format!("git merge fallo: {}", s.primera_linea_error())));
        }
        mergeado = true;
    }
    // Publica si la rama ya estaba publicada y va por delante (merge recien
    // hecho, o un conflicto resuelto con guardar).
    let mut publicada = false;
    if hay_remota && !es_ancestro(git, dir, OP, "HEAD", &remota)? {
        let refspec = format!("refs/heads/{rama}:refs/heads/{rama}");
        let (r, intentos, ultimo) = publicar(git, dir, OP, &refspec, esperas)?;
        match r {
            Resultado::Ok | Resultado::SinCambios => publicada = true,
            Resultado::Rechazada => {
                return Ok(Informe::nuevo(OP, Resultado::Rechazada)
                    .con("rama", rama.as_str())
                    .con("motivo", "no-fast-forward")
                    .diagnostico(
                        "alguien publico en la rama entre medias: vuelve a lanzar actualizar-rama",
                    ));
            }
            _ => {
                return Ok(Informe::nuevo(OP, Resultado::Sinred)
                    .con("rama", rama.as_str())
                    .con("intentos", intentos.to_string())
                    .diagnostico(format!(
                        "push: {ultimo}; la rama esta actualizada en local, reintenta con red"
                    )));
            }
        }
    }
    let head = commit_de(git, dir, OP, "HEAD")?.unwrap_or_default();
    if !mergeado && !publicada {
        return Ok(Informe::nuevo(OP, Resultado::SinCambios).con("base", base));
    }
    Ok(Informe::nuevo(OP, Resultado::Ok)
        .con("base", base)
        .con("commit", corto(&head))
        .con("publicada", if publicada { "si" } else { "no" }))
}

/// Adelanta la rama del worktree principal (`develop` o `main`) con
/// `pull --ff-only`. Al dia → `sin-cambios`; divergente → `rechazada`.
pub fn actualizar_principal(git: &Git, dir: &Path, rama: &str) -> Informe {
    actualizar_principal_paso(git, dir, rama).unwrap_or_else(|i| i)
}

fn actualizar_principal_paso(git: &Git, dir: &Path, rama: &str) -> Paso<Informe> {
    const OP: &str = "actualizar-principal";
    if !es_protegida(rama) {
        return Err(Informe::nuevo(OP, Resultado::Uso).con("motivo", "rama-invalida"));
    }
    let principal = localizar_principal(git, dir).map_err(|i| renombrar(i, OP))?;
    let actual = rama_actual(git, &principal, OP)?;
    if actual.as_deref() != Some(rama) {
        return Ok(Informe::nuevo(OP, Resultado::Rechazada)
            .con("rama", rama)
            .con("motivo", "principal-en-otra-rama")
            .diagnostico(format!("el worktree principal no esta en {rama}")));
    }
    traer(git, &principal, OP)?;
    let remota = format!("{REMOTO}/{rama}");
    let Some(r) = commit_de(git, &principal, OP, &remota)? else {
        return Ok(Informe::nuevo(OP, Resultado::Rechazada)
            .con("rama", rama)
            .con("motivo", "sin-rama-remota"));
    };
    if es_ancestro(git, &principal, OP, &r, "HEAD")? {
        return Ok(Informe::nuevo(OP, Resultado::SinCambios).con("rama", rama));
    }
    if !es_ancestro(git, &principal, OP, "HEAD", &r)? {
        return Ok(Informe::nuevo(OP, Resultado::Rechazada)
            .con("rama", rama)
            .con("motivo", "divergente")
            .diagnostico(format!(
                "{rama} local tiene commits que no estan en {remota}"
            )));
    }
    exigir(
        git,
        &principal,
        OP,
        Subcomando::Pull,
        &["-q", "--ff-only", REMOTO, rama],
    )?;
    Ok(Informe::nuevo(OP, Resultado::Ok)
        .con("rama", rama)
        .con("commit", corto(&r)))
}
