//! Inventario de `tc2 init`: de solo lectura (ficheros, refs, worktrees,
//! `ls-remote` y lecturas `GET` de la plataforma), calcula el plan.

use std::path::Path;

use super::{ARREGLO_016, Accion, Contexto, Decision, Efecto, NOMBRE, ORDEN_PUERTAS, leer, t};
use crate::config::{self, Motivo};
use crate::contrato::{Informe, Resultado};
use crate::estado::{self, CARPETA, Previsto};
use crate::git::estado::{dir_comun, hay_remoto, url_origin};
use crate::git::rama::renombrar;
use crate::git::{
    FalloRemoto, REMOTO, Subcomando, clasificar_fallo_remoto, commit_de, correr, exigir,
    localizar_principal, worktrees,
};
use crate::huella::{LINEAS_EXCLUDE, LINEAS_GITIGNORE, RUTA_WORKFLOW};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::init as txt;
use crate::plataforma::personas::{self, Personas};
use crate::plataforma::puertas::{modelo, ruleset_github};
use crate::plataforma::{GitHub, GitLab, Plataforma, Proveedor, Rol, detectar};

/// El adaptador de la plataforma y su proveedor, o el motivo por el que no
/// lo hay (`sin-remoto`, `sin-token` o el motivo de `detectar`, como
/// `plataforma-contradice`).
type Conexion = Result<(Box<dyn Plataforma>, Proveedor), String>;

/// Ramas de codigo que `init` deja (F1-3).
const RAMAS: [&str; 2] = ["main", "develop"];

/// Lineas de `lineas` que no estan en `texto`. Se comparan sin espacios y
/// sin la barra final: `/tareas` ya ignora lo mismo que `/tareas/`.
pub(super) fn faltan(texto: &str, lineas: &[&str]) -> Vec<String> {
    lineas
        .iter()
        .filter(|l| !texto.lines().any(|x| misma_linea(x, l)))
        .map(|l| (*l).to_owned())
        .collect()
}

/// Si dos lineas de `.gitignore` o `exclude` son la misma regla.
pub(super) fn misma_linea(a: &str, b: &str) -> bool {
    let norm = |s: &str| s.trim().trim_end_matches('/').to_owned();
    norm(a) == norm(b)
}

/// Argumento entrecomillado para el shell de la persona: una ruta con
/// espacios, `;` o `'` no parte la orden ni ejecuta nada. Comillas simples
/// en los dos, que no expanden nada; dentro, la `'` se escribe `''` en
/// PowerShell (`windows`) y `'\''` en `sh`, cuya forma PowerShell trocearia.
pub(super) fn comillas(s: &str, windows: bool) -> String {
    if windows {
        format!("'{}'", s.replace('\'', "''"))
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

fn sinred(detalle: &str) -> Informe {
    Informe::nuevo(NOMBRE, Resultado::Sinred)
        .con("pendiente", "no")
        .diagnostico(Mensaje::nuevo(&txt::SIN_RED).con("error", detalle))
}

fn denegada(f: FalloRemoto, detalle: &str) -> Informe {
    Informe::nuevo(NOMBRE, Resultado::Denegada)
        .con("motivo", f.motivo())
        .diagnostico(
            Mensaje::nuevo(&txt::DENEGADA)
                .con("motivo", f.motivo())
                .con("error", detalle),
        )
}

pub(super) fn plan(c: &Contexto<'_>) -> Result<Vec<Accion>, Informe> {
    let g = c.git;
    let principal = localizar_principal(g, c.dir).map_err(|i| renombrar(i, NOMBRE))?;
    let mut v = Vec::new();

    // Configuracion: la 0.16 o una que no vale no se pisan (C4).
    let ruta_config = principal.join(config::RUTA);
    let mut clave: Option<(&'static str, String)> = None;
    match leer(&ruta_config)? {
        None => v.push(Accion::crea(
            t(&ruta_config),
            "config",
            Efecto::Escribir {
                ruta: ruta_config.clone(),
                contenido: config::texto_minimo().to_owned(),
            },
        )),
        Some(bytes) => match config::leer(&String::from_utf8_lossy(&bytes)) {
            Ok(cfg) => {
                clave = cfg.plataforma.map(|p| (p.proveedor.nombre(), p.api));
                v.push(Accion::nueva(
                    t(&ruta_config),
                    "config",
                    "taskcodetwo",
                    Decision::Existe,
                ));
            }
            Err(e) if e.motivo == Motivo::Formato => {
                v.push(rechazo_016(t(&ruta_config), "config"));
            }
            Err(e) => {
                let mut a = Accion::nueva(t(&ruta_config), "config", "ajeno", Decision::Rechaza)
                    .motivo(e.motivo.nombre());
                a.diagnostico = Some(
                    Mensaje::nuevo(&txt::CONFIG_NO_VALE)
                        .con("ruta", config::RUTA)
                        .con("motivo", e.motivo.nombre()),
                );
                v.push(a);
            }
        },
    }

    // `.gitignore`: solo se anaden lineas.
    let gitignore = principal.join(".gitignore");
    let texto = leer(&gitignore)?
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .unwrap_or_default();
    let lineas = faltan(&texto, &LINEAS_GITIGNORE);
    v.push(if lineas.is_empty() {
        Accion::nueva(t(&gitignore), "gitignore", "taskcodetwo", Decision::Existe)
    } else {
        Accion::crea(
            t(&gitignore),
            "gitignore",
            Efecto::Anadir {
                ruta: gitignore.clone(),
                lineas,
            },
        )
    });

    // La rama de estado y el remoto: lo que se lee aqui decide si hay red.
    let previsto = estado::previsto(g, &principal).map_err(|i| renombrar(i, NOMBRE))?;
    match &previsto {
        Previsto::SinRed(d) => return Err(sinred(d)),
        Previsto::Denegada(f, d) => return Err(denegada(*f, d)),
        _ => {}
    }
    let remoto = hay_remoto(g, &principal, NOMBRE)?;
    let remotas = if remoto {
        ramas_remotas(c, &principal)?
    } else {
        Vec::new()
    };

    // `main` y `develop` (F1-3): con un commit raiz solo en un repo vacio.
    if !ramas(c, &principal, remoto, &remotas, &mut v)?
        && let Some(a) = versionar(c, &principal, &v)?
    {
        v.push(a);
    }

    // `info/exclude`.
    let comun = dir_comun(g, &principal, NOMBRE)?;
    let exclude = comun.join("info").join("exclude");
    let texto = leer(&exclude)?
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .unwrap_or_default();
    v.push(if faltan(&texto, &LINEAS_EXCLUDE).is_empty() {
        Accion::nueva(t(&exclude), "exclude", "taskcodetwo", Decision::Existe)
    } else {
        Accion::crea(
            t(&exclude),
            "exclude",
            Efecto::Excluir {
                principal: principal.clone(),
            },
        )
    });

    // Rama de estado montada en `tareas/`.
    let carpeta = principal.join(CARPETA);
    v.push(match previsto {
        Previsto::Montada => Accion::nueva(
            t(&carpeta),
            "worktree-estado",
            "taskcodetwo",
            Decision::Existe,
        ),
        Previsto::Local | Previsto::DelRemoto | Previsto::Crear => Accion::crea(
            t(&carpeta),
            "worktree-estado",
            Efecto::MontarEstado {
                principal: principal.clone(),
            },
        ),
        Previsto::Ocupada => rechazo_016(t(&carpeta), "worktree-estado"),
        Previsto::OtraRuta(r) => Accion::nueva(
            t(&carpeta),
            "worktree-estado",
            "taskcodetwo",
            Decision::Rechaza,
        )
        .motivo("montada-en-otra-ruta")
        .orden(format!(
            "git worktree move {} {}",
            comillas(&t(&r), cfg!(windows)),
            CARPETA
        )),
        Previsto::SinRed(_) | Previsto::Denegada(..) => {
            // Ya devuelto arriba.
            Accion::nueva(
                t(&carpeta),
                "worktree-estado",
                "taskcodetwo",
                Decision::Pendiente,
            )
        }
    });

    // Personas y puertas: necesitan la plataforma y el token de la persona.
    let plataforma = if remoto {
        plataforma(c, &principal, clave.as_ref())?
    } else {
        Err("sin-remoto".to_owned())
    };
    v.push(personas_yml(&principal, &plataforma)?);

    // Workflow de CI: ranura hasta que F5-1 entregue el reutilizable (D2).
    let workflow = principal.join(RUTA_WORKFLOW);
    v.push(if workflow.is_file() {
        Accion::nueva(t(&workflow), "workflow", "taskcodetwo", Decision::Existe)
    } else {
        Accion::nueva(t(&workflow), "workflow", "taskcodetwo", Decision::Pendiente)
            .motivo("sin-plantilla")
    });

    // Rulesets: solo propuesta (D1); los aplica TASK-118.
    puertas(&plataforma, &mut v);
    Ok(v)
}

/// Linea `rechaza motivo=proyecto-016` (C4).
fn rechazo_016(ruta: String, clase: &'static str) -> Accion {
    let mut a = Accion::nueva(ruta.clone(), clase, "taskcode-016", Decision::Rechaza)
        .motivo("proyecto-016")
        .orden(ARREGLO_016);
    a.diagnostico = Some(
        Mensaje::nuevo(&txt::PROYECTO_016)
            .con("ruta", ruta)
            .con("arreglo", ARREGLO_016),
    );
    a
}

/// Todas las ramas que tiene `origin`, `(nombre, sha)` (para saber si tiene
/// historia, no solo `main` y `develop`). Sin red, `sinred`.
fn ramas_remotas(c: &Contexto<'_>, principal: &Path) -> Result<Vec<(String, String)>, Informe> {
    let s = correr(
        c.git,
        principal,
        NOMBRE,
        Subcomando::LsRemote,
        &["--heads", REMOTO],
    )?;
    if !s.exito() {
        return Err(match clasificar_fallo_remoto(&s.stderr) {
            FalloRemoto::SinRed | FalloRemoto::RamaInexistente => sinred(&s.primera_linea_error()),
            FalloRemoto::Local => crate::git::remoto::error_local(NOMBRE, &s.primera_linea_error()),
            otro => denegada(otro, &s.primera_linea_error()),
        });
    }
    Ok(s.stdout
        .lines()
        .filter_map(|l| l.split_once(char::is_whitespace))
        .filter_map(|(sha, r)| {
            r.trim()
                .strip_prefix("refs/heads/")
                .map(|n| (n.to_owned(), sha.to_owned()))
        })
        .collect())
}

/// Como esta la historia local frente a la de `origin`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Historia {
    /// Tienen algun ancestro comun.
    Relacionada,
    /// Se comprobo cada rama de `origin` y ninguna tiene nada en comun.
    SinRelacion,
    /// Falta en el clon algun commit de `origin` con el que comparar (y con
    /// los que hay, ninguno en comun): hace falta `git fetch origin`.
    NoComprobable,
}

impl Historia {
    /// Motivo de las lineas cuando no se propone publicar.
    fn motivo(self) -> Option<&'static str> {
        match self {
            Self::Relacionada => None,
            Self::SinRelacion => Some(MOTIVO_SIN_RELACION),
            Self::NoComprobable => Some(MOTIVO_NO_COMPROBADO),
        }
    }
}

/// Motivo de una rama cuya historia no tiene nada en comun con `origin`.
pub(super) const MOTIVO_SIN_RELACION: &str = "historia-sin-relacion";

/// Motivo de una rama cuya relacion con `origin` no se pudo comprobar.
pub(super) const MOTIVO_NO_COMPROBADO: &str = "no-comprobado";

/// Si la historia local (HEAD, o la primera rama local) tiene algun ancestro
/// comun con alguna rama de `origin`. Solo con lo que ya hay en el clon (sin
/// `fetch`: `--simular` no escribe): por cada rama de `origin`, la rama
/// remota que se trajo y el sha de `ls-remote` si ese commit ya esta (aunque
/// falte la ref remota, p. ej. traido por sha). Si alguna rama no tiene
/// commit con el que comparar, no se afirma que no haya relacion.
fn historia_relacionada(
    c: &Contexto<'_>,
    principal: &Path,
    remotas: &[(&str, &str)],
) -> Result<Historia, Informe> {
    let g = c.git;
    let primera_local = exigir(
        g,
        principal,
        NOMBRE,
        Subcomando::ForEachRef,
        &["--count=1", "--format=%(objectname)", "refs/heads/"],
    )?;
    let Some(local) = commit_de(g, principal, NOMBRE, "HEAD")?
        .or_else(|| Some(primera_local.trim().to_owned()).filter(|s| !s.is_empty()))
    else {
        return Ok(Historia::NoComprobable);
    };
    let mut falta_alguna = false;
    for (nombre, sha) in remotas {
        let traida = commit_de(
            g,
            principal,
            NOMBRE,
            &format!("refs/remotes/{REMOTO}/{nombre}"),
        )?;
        let conocida = commit_de(g, principal, NOMBRE, sha)?;
        if traida.is_none() && conocida.is_none() {
            falta_alguna = true;
        }
        for candidato in [traida, conocida].into_iter().flatten() {
            let s = correr(
                g,
                principal,
                NOMBRE,
                Subcomando::MergeBase,
                &[&local, &candidato],
            )?;
            if s.exito() && !s.stdout.trim().is_empty() {
                return Ok(Historia::Relacionada);
            }
        }
    }
    Ok(if falta_alguna {
        Historia::NoComprobable
    } else {
        Historia::SinRelacion
    })
}

/// Lineas de `main` y `develop` (D3). El repo cuenta como vacio solo si no
/// tiene commits y `origin` no tiene ninguna rama (la de estado no cuenta:
/// es huerfana). Vacio: un commit raiz con la configuracion y `.gitignore` y
/// las dos ramas en el (publicadas si hay remoto). Con historia, local o en
/// `origin`, la rama que falta la crea la persona desde la que ya hay: un
/// commit raiz propio dejaria en el remoto dos historias sin relacion. Si
/// `origin` tiene historia, se parte siempre de ella (`origin/<rama>`), y
/// si la historia local no tiene relacion con la remota no se propone
/// publicar nada (`motivo=historia-sin-relacion`, con aviso).
/// Devuelve si el repo estaba vacio.
fn ramas(
    c: &Contexto<'_>,
    principal: &Path,
    remoto: bool,
    remotas: &[(String, String)],
    v: &mut Vec<Accion>,
) -> Result<bool, Informe> {
    let g = c.git;
    let hay_commits = !exigir(
        g,
        principal,
        NOMBRE,
        Subcomando::ForEachRef,
        &["--count=1", "--format=%(refname)", "refs/heads/"],
    )?
    .trim()
    .is_empty();
    let head = worktrees(g, principal, NOMBRE)?
        .first()
        .and_then(|w| w.rama.clone());
    let mut locales = Vec::new();
    for r in RAMAS {
        if commit_de(g, principal, NOMBRE, &format!("refs/heads/{r}"))?.is_some() {
            locales.push(r);
        }
    }
    // Ramas de `origin` con historia del proyecto (sin la de estado).
    let con_sha: Vec<(&str, &str)> = remotas
        .iter()
        .map(|(n, s)| (n.as_str(), s.as_str()))
        .filter(|(n, _)| *n != crate::huella::RAMA_ESTADO)
        .collect();
    let historia_remota: Vec<&str> = con_sha.iter().map(|(n, _)| *n).collect();
    let vacio = !hay_commits && historia_remota.is_empty();
    // Con historia en los dos lados, que tengan algo en comun. Si no la
    // tienen, o no se pudo comprobar, no se propone publicar nada.
    let relacion = if hay_commits && !historia_remota.is_empty() {
        historia_relacionada(c, principal, &con_sha)?
    } else {
        Historia::Relacionada
    };
    let sin_publicar = relacion.motivo();
    // Rama de `origin` desde la que partir (la otra rama de codigo, o la
    // primera que haya).
    let otra_remota = RAMAS
        .iter()
        .copied()
        .find(|x| historia_remota.contains(x))
        .or_else(|| historia_remota.first().copied());
    let mut nuevas: Vec<String> = Vec::new();
    let mut lineas = Vec::new();
    for r in RAMAS {
        let ruta = format!("refs/heads/{r}");
        let local = locales.contains(&r);
        let en_remoto = historia_remota.contains(&r);
        let a = Accion::nueva(ruta, "rama", "proyecto", Decision::Existe);
        lineas.push(if local {
            if let (true, false, Some(m)) = (remoto, en_remoto, sin_publicar) {
                // Publicarla podria subir a `origin` una historia ajena a la
                // suya: la orden trae el remoto y comprueba si tienen algo en
                // comun (`merge-base` vacio: nada).
                let comprobar = match otra_remota {
                    Some(b) => format!("git fetch {REMOTO} && git merge-base {r} {REMOTO}/{b}"),
                    None => format!("git fetch {REMOTO}"),
                };
                persona(a, m, comprobar)
            } else if remoto && !en_remoto {
                persona(a, "sin-publicar", format!("git push {REMOTO} {r}"))
            } else {
                a
            }
        } else if en_remoto {
            persona(
                a,
                "en-el-remoto",
                format!("git fetch {REMOTO} {r} && git branch {r} {REMOTO}/{r}"),
            )
        } else if !vacio {
            // Si `origin` tiene historia, desde ella; si no, desde la otra
            // rama local (HEAD puede estar sin nacer en esta, tras un `init`
            // cortado a medias) o desde HEAD.
            let mut o = match (otra_remota, locales.first()) {
                (Some(b), _) => format!("git fetch {REMOTO} {b} && git branch {r} {REMOTO}/{b}"),
                (None, Some(l)) => format!("git branch {r} {l}"),
                (None, None) => format!("git branch {r} HEAD"),
            };
            if remoto && sin_publicar.is_none() {
                o.push_str(&format!(" && git push {REMOTO} {r}"));
            }
            let motivo = if let Some(m) = sin_publicar {
                m
            } else if hay_commits {
                "con-historia"
            } else {
                "remoto-con-historia"
            };
            persona(a, motivo, o)
        } else {
            nuevas.push(r.to_owned());
            let mut a = a;
            a.decision = Decision::Crea;
            a
        });
    }
    if let Some(primera) = lineas.iter_mut().find(|a| a.decision == Decision::Crea) {
        primera.efecto = Efecto::RamasRaiz {
            principal: principal.to_path_buf(),
            ficheros: vec![config::RUTA.to_owned(), ".gitignore".to_owned()],
            ramas: nuevas.clone(),
            head: head.as_ref().is_some_and(|h| nuevas.contains(h)),
            publicar: remoto,
        };
    }
    v.extend(lineas);
    // HEAD sin nacer en otra rama (`master`): se queda en ella; la persona
    // la pasa a `develop` (con los ficheros ya en el disco, `git switch` se
    // negaria).
    if !hay_commits
        && !nuevas.is_empty()
        && let Some(h) = head.filter(|h| !nuevas.contains(h))
    {
        v.push(persona(
            Accion::nueva("HEAD", "head", "proyecto", Decision::Persona),
            &format!("head-en-{h}"),
            "git symbolic-ref HEAD refs/heads/develop && git reset -q".to_owned(),
        ));
    }
    Ok(vacio)
}

/// Con historia, `init` deja la configuracion y `.gitignore` en el disco: la
/// persona tiene que versionarlos (por PR) o quien clone no los tendra. Se
/// dice si `init` los crea o cambia, o si siguen sin versionar.
fn versionar(c: &Contexto<'_>, principal: &Path, v: &[Accion]) -> Result<Option<Accion>, Informe> {
    const FICHEROS: [&str; 2] = [config::RUTA, ".gitignore"];
    let los_toca = v
        .iter()
        .any(|a| matches!(a.clase, "config" | "gitignore") && a.decision == Decision::Crea);
    let mut args = vec!["--porcelain", "--"];
    args.extend(FICHEROS);
    let sin_versionar = !exigir(c.git, principal, NOMBRE, Subcomando::Status, &args)?
        .trim()
        .is_empty();
    Ok((los_toca || sin_versionar).then(|| {
        persona(
            Accion::nueva(
                FICHEROS.join(","),
                "versionar",
                "proyecto",
                Decision::Persona,
            ),
            "sin-versionar",
            format!(
                "git add -- {} && git commit -m \"chore: configura TaskCodeTwo\"",
                FICHEROS.join(" ")
            ),
        )
    }))
}

fn persona(mut a: Accion, motivo: &str, orden: String) -> Accion {
    a.decision = Decision::Persona;
    a.motivo(motivo).orden(orden)
}

/// El adaptador de la plataforma con el token de la persona, o el motivo
/// por el que no lo hay (que queda en las lineas `pendiente`).
fn plataforma(
    c: &Contexto<'_>,
    principal: &Path,
    clave: Option<&(&'static str, String)>,
) -> Result<Conexion, Informe> {
    let Some(url) = url_origin(c.git, principal, NOMBRE)? else {
        return Ok(Err("sin-remoto".to_owned()));
    };
    let destino = match detectar(&url, clave.map(|(p, a)| (*p, a.as_str()))) {
        Ok(d) => d,
        Err(i) => return Ok(Err(motivo_de_deteccion(&i))),
    };
    let credenciales = (c.credenciales)();
    if credenciales.token(Rol::Persona).is_none() {
        return Ok(Err("sin-token".to_owned()));
    }
    let p: Result<Box<dyn Plataforma>, Informe> = match destino.proveedor {
        Proveedor::GitHub => GitHub::nuevo(
            destino.api,
            &destino.proyecto,
            credenciales,
            Personas::default(),
        )
        .map(|x| Box::new(x) as Box<dyn Plataforma>),
        Proveedor::GitLab => GitLab::nuevo(
            destino.api,
            &destino.proyecto,
            credenciales,
            Personas::default(),
        )
        .map(|x| Box::new(x) as Box<dyn Plataforma>),
    };
    Ok(p.map(|p| (p, destino.proveedor)).map_err(|i| motivo_de(&i)))
}

/// Motivo concreto de un rechazo de `detectar` (su linea solo dice
/// `motivo=plataforma` u `origin`; el porque esta en el mensaje): la clave
/// del mensaje en minusculas y con guiones, p. ej. `plataforma-contradice`
/// o `api-no-cambiable`. No lleva datos de la persona ni el token.
fn motivo_de_deteccion(i: &Informe) -> String {
    i.diagnostico
        .as_ref()
        .and_then(|m| m.clave().rsplit("::").next())
        .map_or_else(
            || "sin-plataforma".to_owned(),
            |c| c.to_lowercase().replace('_', "-"),
        )
}

/// Motivo de un informe de la plataforma para la linea `pendiente`.
fn motivo_de(i: &Informe) -> String {
    i.valor("motivo")
        .map_or_else(|| i.resultado.texto().to_owned(), str::to_owned)
}

/// `personas.yml`: la persona de `GET /user` (F1-6), solo si falta.
fn personas_yml(principal: &Path, plataforma: &Conexion) -> Result<Accion, Informe> {
    let ruta = principal.join(personas::RUTA);
    let nueva = |d: Decision| Accion::nueva(t(&ruta), "personas", "taskcodetwo", d);
    if let Some(bytes) = leer(&ruta)? {
        return Ok(match Personas::leer(&String::from_utf8_lossy(&bytes)) {
            Ok(_) => nueva(Decision::Existe),
            Err(_) => {
                let mut a = nueva(Decision::Rechaza).motivo("personas");
                a.dueno = "ajeno";
                a.diagnostico =
                    Some(Mensaje::nuevo(&txt::PERSONAS_NO_VALE).con("ruta", personas::RUTA));
                a
            }
        });
    }
    let (p, _) = match plataforma {
        Ok(p) => p,
        Err(m) => return Ok(nueva(Decision::Pendiente).motivo(m.clone())),
    };
    let u = match p.identidad(Rol::Persona) {
        Ok(u) => u,
        Err(i) => return Ok(nueva(Decision::Pendiente).motivo(motivo_de(&i))),
    };
    if u.tipo != "User" {
        return Ok(nueva(Decision::Pendiente).motivo("no-es-persona"));
    }
    let texto = personas::texto_de(&u);
    if Personas::leer(&texto).is_err() {
        return Ok(nueva(Decision::Pendiente).motivo("login-no-valido"));
    }
    Ok(Accion::crea(
        t(&ruta),
        "personas",
        Efecto::Escribir {
            ruta: ruta.clone(),
            contenido: texto,
        },
    ))
}

/// Una linea por ruleset del modelo de F1-5: `existe` si la plataforma ya
/// lo tiene; si no, `persona` con la orden que lo aplicara (TASK-118) y la
/// propuesta en JSON. Solo lecturas: ninguna escritura en la plataforma.
fn puertas(plataforma: &Conexion, v: &mut Vec<Accion>) {
    let leidas = match plataforma {
        Ok((p, prov)) => Ok((p.leer_puertas(Rol::Persona, "develop"), *prov)),
        Err(m) => Err(m.clone()),
    };
    for puerta in modelo() {
        let nombre = puerta.nombre_completo();
        let nueva = |d: Decision| Accion::nueva(nombre.clone(), "ruleset", "taskcodetwo", d);
        v.push(match &leidas {
            Err(m) => nueva(Decision::Pendiente).motivo(m.clone()),
            Ok((Err(i), _)) => nueva(Decision::Pendiente).motivo(motivo_de(i)),
            Ok((Ok(r), _)) if !r.plan_da_reglas => nueva(Decision::Pendiente).motivo("plan"),
            Ok((Ok(r), _)) if r.rulesets.contains(&nombre) => nueva(Decision::Existe),
            Ok((Ok(_), prov)) => {
                let mut a = nueva(Decision::Persona)
                    .motivo("aplicar-con-task-118")
                    .orden(ORDEN_PUERTAS);
                if *prov == Proveedor::GitHub {
                    a.propuesta = Some(ruleset_github(&puerta));
                }
                a
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Revision 2, MEN-5: en `sh`, la `'` se cierra, se escapa y se reabre;
    /// el `;` queda dentro de las comillas.
    #[test]
    fn comillas_para_sh() {
        assert_eq!(comillas("/r/a b", false), "'/r/a b'");
        assert_eq!(
            comillas("/r/a'; echo X; 'b", false),
            "'/r/a'\\''; echo X; '\\''b'"
        );
    }

    /// Revision 2, MEN-5: en PowerShell la `'` se dobla (`''`) dentro de
    /// comillas simples, que no expanden `$` ni `` ` ``; el `;` queda
    /// dentro.
    #[test]
    fn comillas_para_powershell() {
        assert_eq!(comillas("C:/r/a b", true), "'C:/r/a b'");
        assert_eq!(
            comillas("C:/r/a'; calc; 'b $x `y", true),
            "'C:/r/a''; calc; ''b $x `y'"
        );
    }

    #[test]
    fn misma_regla_con_o_sin_barra_final() {
        assert!(misma_linea("/tareas", "/tareas/"));
        assert!(misma_linea("  /tareas/ ", "/tareas/"));
        assert!(!misma_linea("/tareas-viejas/", "/tareas/"));
    }
}
