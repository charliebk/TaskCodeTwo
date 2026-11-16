//! Operaciones con el remoto: `fetch`, `push` con reintento,
//! `actualizar-rama` y `actualizar-principal`.

use std::path::Path;
use std::time::Duration;

use super::rama::renombrar;
use super::{
    Git, Paso, REMOTO, Subcomando, base_de_rama, commit_de, correr, entradas_status, es_ancestro,
    es_protegida, exigir, git_fallo, localizar_principal, rama_actual,
};
use crate::contrato::{Informe, Resultado, corto};
use crate::i18n::catalogo::git as txt;
use crate::i18n::{Mensaje, Texto};

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
        .diagnostico(sin_red_remoto(op))
}

/// «`op`: no se pudo leer origin; reintenta con red».
pub(crate) fn sin_red_remoto(op: &str) -> Mensaje {
    Mensaje::nuevo(&txt::SIN_RED_REMOTO)
        .con("op", op)
        .con("remoto", REMOTO)
}

/// Por que fallo una llamada que habla con el remoto (F2-3: responde las
/// preguntas abiertas de F1-2 sobre credenciales y rechazos del remoto).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FalloRemoto {
    /// No hubo respuesta: red caida, DNS, servidor parado. Se reintenta.
    SinRed,
    /// El remoto respondio pero no acepta estas credenciales o permisos.
    Credenciales,
    /// La URL de `origin` no es un repositorio.
    RemotoNoExiste,
    /// El remoto existe pero no tiene la rama pedida.
    RamaInexistente,
    /// Fallo del propio clon, no del remoto (p. ej. sin permiso para
    /// escribir en `.git`): `error`, ni red ni credenciales.
    Local,
}

impl FalloRemoto {
    /// Valor de `motivo=` en la linea de contrato.
    pub fn motivo(self) -> &'static str {
        match self {
            Self::SinRed => "sin-red",
            Self::Credenciales => "credenciales",
            Self::RemotoNoExiste => "remoto-no-existe",
            Self::RamaInexistente => "rama-no-existe",
            Self::Local => "error-local",
        }
    }
}

/// Clasifica el stderr de `fetch`, `push` o `ls-remote` (en `LC_ALL=C`, que
/// fija el puerto). Lo que no se reconoce es `SinRed`: manda reintentar, que
/// es lo prudente ante un texto desconocido.
pub fn clasificar_fallo_remoto(stderr: &str) -> FalloRemoto {
    let t = stderr.to_ascii_lowercase();
    let alguno = |patrones: &[&str]| patrones.iter().any(|p| t.contains(p));
    // Marcas del transporte: curl o ssh que no llegan a conectar. Con ellas,
    // un `permission denied` es EACCES de la red (firewall, SELinux).
    let transporte = alguno(&[
        "unable to access",
        "failed to connect",
        "connect to host",
        "could not resolve",
        "couldn't connect",
    ]);
    if alguno(&["couldn't find remote ref", "could not find remote ref"]) {
        FalloRemoto::RamaInexistente
    } else if alguno(&[
        "authentication failed",
        "permission denied (publickey",
        "could not read username",
        "could not read password",
        "terminal prompts disabled",
        "returned error: 401",
        "returned error: 403",
        "remote: permission to ",
        "access denied",
    ]) || (t.contains("permission denied")
        && t.contains("could not read from remote repository")
        && !transporte)
    {
        // `permission denied` suelto tambien lo da el sistema de ficheros
        // local (no poder escribir en `.git`): solo cuenta como credenciales
        // si git dice ademas que no pudo leer del remoto.
        FalloRemoto::Credenciales
    } else if alguno(&[
        "does not appear to be a git repository",
        "repository not found",
        "not a git repository",
        "returned error: 404",
    ]) {
        FalloRemoto::RemotoNoExiste
    } else if t.contains("permission denied") && !transporte {
        // Sin las marcas del remoto de arriba ni las del transporte (curl o
        // ssh con EACCES, p. ej. un firewall, son la red), es el sistema de
        // ficheros del clon (`cannot open .git/FETCH_HEAD: Permission denied`).
        FalloRemoto::Local
    } else {
        FalloRemoto::SinRed
    }
}

/// Informe de un fallo del remoto que no es de red: `denegada` (8).
pub(crate) fn denegada(op: &str, fallo: FalloRemoto, detalle: &str) -> Informe {
    let texto = match fallo {
        FalloRemoto::Credenciales => &txt::DENEGADA_CREDENCIALES,
        FalloRemoto::RemotoNoExiste => &txt::DENEGADA_URL,
        _ => &txt::DENEGADA_REGLAS,
    };
    Informe::nuevo(op, Resultado::Denegada)
        .con("remoto", REMOTO)
        .con("motivo", fallo.motivo())
        .diagnostico(Mensaje::nuevo(texto).con("op", op).con("detalle", detalle))
}

/// Fallo del propio clon al hablar con el remoto: `error` (1).
pub(crate) fn error_local(op: &str, detalle: &str) -> Informe {
    Informe::nuevo(op, Resultado::Error)
        .con("motivo", FalloRemoto::Local.motivo())
        .diagnostico(
            Mensaje::nuevo(&txt::ERROR_LOCAL)
                .con("op", op)
                .con("detalle", detalle),
        )
}

/// Que hacer tras un push que no salio, segun el resultado.
pub(crate) fn que_hacer_al_publicar(r: Resultado) -> &'static Texto {
    match r {
        Resultado::Denegada => &txt::QUE_HACER_DENEGADA,
        Resultado::Error => &txt::QUE_HACER_ERROR_LOCAL,
        _ => &txt::QUE_HACER_SIN_RED,
    }
}

/// «push: `ultimo`; que hacer».
fn push_fallo(ultimo: String, r: Resultado) -> Mensaje {
    Mensaje::nuevo(&txt::PUSH_FALLO)
        .con("ultimo", ultimo)
        .con_mensaje("que_hacer", que_hacer_al_publicar(r))
}

/// Trae el remoto. Nada nuevo → `sin-cambios`; sin red → `sinred`; remoto
/// inexistente o sin permiso → `denegada`.
pub fn fetch(git: &Git, dir: &Path) -> Informe {
    traer(git, dir, "fetch").unwrap_or_else(|i| i)
}

fn traer(git: &Git, dir: &Path, op: &str) -> Paso<Informe> {
    let antes = refs_remotas(git, dir, op)?;
    let s = correr(git, dir, op, Subcomando::Fetch, &["-q", REMOTO])?;
    if !s.exito() {
        return Err(match clasificar_fallo_remoto(&s.stderr) {
            FalloRemoto::SinRed | FalloRemoto::RamaInexistente => sinred(op),
            FalloRemoto::Local => error_local(op, &s.primera_linea_error()),
            otro => denegada(op, otro, &s.primera_linea_error()),
        });
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
    /// Fallo con su resumen y, si el remoto respondio denegandolo
    /// (`[remote rejected]` por un hook o una regla, credenciales, remoto
    /// inexistente), el motivo: eso no se arregla reintentando.
    Fallo(String, Option<&'static str>),
    /// Fallo del propio clon (permisos en `.git`): no es del remoto.
    Local(String),
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
        _ if resumen.starts_with("[remote rejected]") && es_contencion(resumen) => {
            Intento::NoFastForward
        }
        _ if resumen.starts_with("[remote rejected]") => {
            Intento::Fallo(resumen.to_owned(), Some("remoto-rechaza"))
        }
        _ => {
            let detalle = if resumen.is_empty() {
                s.primera_linea_error()
            } else {
                resumen.to_owned()
            };
            match clasificar_fallo_remoto(&s.stderr) {
                FalloRemoto::SinRed | FalloRemoto::RamaInexistente => Intento::Fallo(detalle, None),
                FalloRemoto::Local => Intento::Local(detalle),
                otro => Intento::Fallo(detalle, Some(otro.motivo())),
            }
        }
    })
}

/// `[remote rejected]` por contencion: otro push actualizo la misma ref a la
/// vez y el remoto no pudo tomar su cerrojo (visto en F2-3 con dos procesos
/// contra el mismo remoto). Es lo mismo que un non-fast-forward: la ref se
/// movio; no es una denegacion. `reference already exists`: dos clones
/// crean a la vez una ref que no existia (la rama de estado en el primer
/// `new`, visto en F2-4) y el remoto rechaza al segundo.
fn es_contencion(resumen: &str) -> bool {
    let t = resumen.to_ascii_lowercase();
    [
        "reference already exists",
        "cannot lock ref",
        "failed to lock",
        "unable to lock",
        "failed to update ref",
        "but expected",
        "incorrect old value",
    ]
    .iter()
    .any(|p| t.contains(p))
}

/// Como termino [`publicar`].
pub(crate) struct Publicacion {
    /// `Ok`, `SinCambios`, `Rechazada` (non-fast-forward), `Denegada`,
    /// `Error` (fallo local) o `Sinred`.
    pub resultado: Resultado,
    /// Intentos hechos.
    pub intentos: usize,
    /// Ultimo fallo, para el diagnostico.
    pub ultimo: String,
    /// Motivo de una `Denegada` o un `Error` local.
    pub motivo: &'static str,
}

/// Publica `refspec` con reintentos ante fallos transitorios (sin red). Un
/// non-fast-forward (`rechazada`) y una denegacion del remoto (`denegada`:
/// hook, regla, credenciales; reintentar no la arregla) no se reintentan.
/// Agotados los intentos sin red: `sinred`.
pub(crate) fn publicar(
    git: &Git,
    dir: &Path,
    op: &str,
    refspec: &str,
    esperas: &[Duration],
) -> Paso<Publicacion> {
    let fin = |resultado, intentos, ultimo: String, motivo| Publicacion {
        resultado,
        intentos,
        ultimo,
        motivo,
    };
    let mut ultimo = String::new();
    for intento in 0..=esperas.len() {
        match intento_push(git, dir, op, refspec)? {
            Intento::AlDia => return Ok(fin(Resultado::SinCambios, intento + 1, ultimo, "")),
            Intento::Publicado => return Ok(fin(Resultado::Ok, intento + 1, ultimo, "")),
            Intento::NoFastForward => {
                return Ok(fin(Resultado::Rechazada, intento + 1, ultimo, ""));
            }
            Intento::Fallo(m, Some(motivo)) => {
                return Ok(fin(Resultado::Denegada, intento + 1, m, motivo));
            }
            Intento::Local(m) => {
                return Ok(fin(
                    Resultado::Error,
                    intento + 1,
                    m,
                    FalloRemoto::Local.motivo(),
                ));
            }
            Intento::Fallo(m, None) => ultimo = m,
        }
        if let Some(e) = esperas.get(intento) {
            std::thread::sleep(*e);
        }
    }
    Ok(fin(Resultado::Sinred, esperas.len() + 1, ultimo, ""))
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
    let p = publicar(git, dir, op, &refspec, esperas)?;
    let (r, intentos, ultimo) = (p.resultado, p.intentos, p.ultimo);
    let i = Informe::nuevo(op, r).con("rama", rama.as_str());
    Ok(match r {
        Resultado::Ok => i
            .con("commit", corto(&head))
            .con("intentos", intentos.to_string()),
        Resultado::SinCambios => i.con("commit", corto(&head)),
        Resultado::Rechazada => i
            .con("motivo", "no-fast-forward")
            .diagnostico(&txt::LANZA_ACTUALIZAR_RAMA),
        Resultado::Denegada | Resultado::Error => i
            .con("motivo", p.motivo)
            .con("intentos", intentos.to_string())
            .diagnostico(push_fallo(ultimo, r)),
        _ => i
            .con("intentos", intentos.to_string())
            .diagnostico(push_fallo(ultimo, r)),
    })
}

/// La rama actual, si es de tarea (ni HEAD suelto ni `develop`/`main`).
fn rama_de_tarea(git: &Git, dir: &Path, op: &str) -> Paso<String> {
    match rama_actual(git, dir, op)? {
        None => Err(Informe::nuevo(op, Resultado::Rechazada)
            .con("motivo", "sin-rama")
            .diagnostico(&txt::LANZALO_EN_EL_WORKTREE)),
        Some(r) if es_protegida(&r) => Err(Informe::nuevo(op, Resultado::Rechazada)
            .con("rama", r.as_str())
            .con("motivo", "rama-protegida")
            .diagnostico(&txt::LANZALO_FUERA_DE_PROTEGIDA)),
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
            Mensaje::nuevo(&txt::LANZA_GUARDAR_CON_RESUELTOS)
        } else {
            Mensaje::nuevo(&txt::RESUELVE_Y_LANZA_GUARDAR).con("ficheros", ficheros.join(" "))
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
            .diagnostico(&txt::GUARDA_ANTES_DE_ACTUALIZAR));
    }
    traer(git, dir, OP)?;
    if commit_de(git, dir, OP, base)?.is_none() {
        return Ok(Informe::nuevo(OP, Resultado::Rechazada)
            .con("base", base)
            .con("motivo", "base-no-existe")
            .diagnostico(Mensaje::nuevo(&txt::BASE_NO_EXISTE_EN_REMOTO).con("base", base)));
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
        // La base entra siempre con merge commit (queda constancia de la
        // actualizacion); lo publicado en la propia rama, por fast-forward si
        // se puede, para no ensuciar el historial.
        let modo = if fuente == base { "--no-ff" } else { "--ff" };
        let s = correr(
            git,
            dir,
            OP,
            Subcomando::Merge,
            &["-q", "--no-edit", modo, fuente],
        )?;
        if !s.exito() {
            if merge_abierto(git, dir, OP)? {
                return Ok(conflicto(sin_resolver(git, dir, OP)?));
            }
            return Ok(Informe::nuevo(OP, Resultado::Error)
                .con("base", base)
                .diagnostico(git_fallo(Subcomando::Merge, &s)));
        }
        mergeado = true;
    }
    // Publica si la rama ya estaba publicada y va por delante (merge recien
    // hecho, o un conflicto resuelto con guardar).
    let mut publicada = false;
    if hay_remota && !es_ancestro(git, dir, OP, "HEAD", &remota)? {
        let refspec = format!("refs/heads/{rama}:refs/heads/{rama}");
        let p = publicar(git, dir, OP, &refspec, esperas)?;
        let (r, intentos, ultimo) = (p.resultado, p.intentos, p.ultimo);
        match r {
            Resultado::Ok | Resultado::SinCambios => publicada = true,
            Resultado::Rechazada => {
                return Ok(Informe::nuevo(OP, Resultado::Rechazada)
                    .con("rama", rama.as_str())
                    .con("motivo", "no-fast-forward")
                    .diagnostico(&txt::PUBLICARON_ENTRE_MEDIAS));
            }
            otro => {
                let que_hacer = que_hacer_al_publicar(otro);
                let mut i = Informe::nuevo(OP, otro).con("rama", rama.as_str());
                if matches!(otro, Resultado::Denegada | Resultado::Error) {
                    i = i.con("motivo", p.motivo);
                }
                return Ok(i.con("intentos", intentos.to_string()).diagnostico(
                    Mensaje::nuevo(&txt::PUSH_FALLO_ACTUALIZADA_EN_LOCAL)
                        .con("ultimo", ultimo)
                        .con_mensaje("que_hacer", que_hacer),
                ));
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
            .diagnostico(Mensaje::nuevo(&txt::PRINCIPAL_EN_OTRA_RAMA).con("rama", rama)));
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
            .diagnostico(
                Mensaje::nuevo(&txt::PRINCIPAL_DIVERGENTE)
                    .con("rama", rama)
                    .con("remota", remota.as_str()),
            ));
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

#[cfg(test)]
mod tests {
    use super::{FalloRemoto, clasificar_fallo_remoto, es_contencion};

    #[test]
    fn clasifica_los_fallos_del_remoto() {
        let casos = [
            (
                "fatal: couldn't find remote ref refs/heads/taskcode/tareas",
                FalloRemoto::RamaInexistente,
            ),
            (
                "remote: Invalid username or password.\nfatal: Authentication failed for 'https://github.com/a/b.git/'",
                FalloRemoto::Credenciales,
            ),
            (
                "fatal: could not read Username for 'https://github.com': terminal prompts disabled",
                FalloRemoto::Credenciales,
            ),
            (
                "git@github.com: Permission denied (publickey).\nfatal: Could not read from remote repository.",
                FalloRemoto::Credenciales,
            ),
            (
                "remote: Permission to a/b.git denied to c.\nfatal: unable to access 'https://github.com/a/b.git/': The requested URL returned error: 403",
                FalloRemoto::Credenciales,
            ),
            (
                "fatal: '/x/no-existe' does not appear to be a git repository\nfatal: Could not read from remote repository.",
                FalloRemoto::RemotoNoExiste,
            ),
            (
                "remote: Repository not found.\nfatal: repository 'https://github.com/a/b.git/' not found",
                FalloRemoto::RemotoNoExiste,
            ),
            (
                "fatal: unable to access 'http://127.0.0.1:1/x.git/': Failed to connect to 127.0.0.1 port 1",
                FalloRemoto::SinRed,
            ),
            (
                "ssh: Could not resolve hostname github.com: Name or service not known\nfatal: Could not read from remote repository.",
                FalloRemoto::SinRed,
            ),
            ("algo que no se reconoce", FalloRemoto::SinRed),
        ];
        for (stderr, esperado) in casos {
            assert_eq!(clasificar_fallo_remoto(stderr), esperado, "{stderr}");
        }
    }

    #[test]
    fn permiso_local_no_es_de_credenciales() {
        let local = "error: cannot open .git/FETCH_HEAD: Permission denied";
        assert_eq!(clasificar_fallo_remoto(local), FalloRemoto::Local);
        // Una denegacion del remoto sigue siendo de credenciales.
        for remoto in [
            "remote: access denied",
            "fatal: unable to access 'https://host/a.git/': access denied",
            "remote: Access denied.\nfatal: Could not read from remote repository.",
        ] {
            assert_eq!(
                clasificar_fallo_remoto(remoto),
                FalloRemoto::Credenciales,
                "{remoto}"
            );
        }
    }

    #[test]
    fn red_bloqueada_con_permiso_denegado_es_sin_red() {
        // curl con EACCES (firewall, SELinux): es la red, no el clon.
        for t in [
            "fatal: unable to access 'https://host/a.git/': Failed to connect to host port 443 after 3 ms: Permission denied",
            "ssh: connect to host github.com port 22: Permission denied\nfatal: Could not read from remote repository.",
        ] {
            assert_eq!(clasificar_fallo_remoto(t), FalloRemoto::SinRed, "{t}");
        }
        // Un `Connection closed` tras rechazar la contrasena no es la red.
        let t = "git@h: Permission denied, please try again.\ngit@h: Permission denied (password).\nConnection closed by 1.2.3.4 port 22\nfatal: Could not read from remote repository.";
        assert_eq!(clasificar_fallo_remoto(t), FalloRemoto::Credenciales);
        let t = "error: cannot lock ref 'refs/heads/x': Permission denied\nhint: connection lost";
        assert_eq!(clasificar_fallo_remoto(t), FalloRemoto::Local);
    }

    #[test]
    fn contencion_frente_a_denegacion() {
        assert!(es_contencion(
            "[remote rejected] (cannot lock ref 'refs/heads/taskcode/tareas': is at 1a but expected 2b)"
        ));
        assert!(es_contencion("[remote rejected] (failed to update ref)"));
        assert!(es_contencion(
            "[remote rejected] (reference already exists)"
        ));
        assert!(!es_contencion(
            "[remote rejected] (pre-receive hook declined)"
        ));
        assert!(!es_contencion(
            "[remote rejected] (protected branch hook declined)"
        ));
    }
}
