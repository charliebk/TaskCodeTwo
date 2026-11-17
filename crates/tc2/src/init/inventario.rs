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
/// lo hay (`sin-remoto`, `sin-plataforma`, `sin-token`).
type Conexion = Result<(Box<dyn Plataforma>, Proveedor), &'static str>;

/// Ramas de codigo que `init` deja (F1-3).
const RAMAS: [&str; 2] = ["main", "develop"];

/// Lineas de `lineas` que no estan en `texto` (comparadas sin espacios).
fn faltan(texto: &str, lineas: &[&str]) -> Vec<String> {
    lineas
        .iter()
        .filter(|l| !texto.lines().any(|x| x.trim() == **l))
        .map(|l| (*l).to_owned())
        .collect()
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
    ramas(c, &principal, remoto, &remotas, &mut v)?;

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
        .orden(format!("git worktree move {} {}", t(&r), CARPETA)),
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
        Err("sin-remoto")
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

/// Ramas de codigo que tiene `origin` (de [`RAMAS`]). Sin red, `sinred`.
fn ramas_remotas(c: &Contexto<'_>, principal: &Path) -> Result<Vec<String>, Informe> {
    let mut args = vec!["--heads".to_owned(), REMOTO.to_owned()];
    args.extend(RAMAS.iter().map(|r| format!("refs/heads/{r}")));
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let s = correr(c.git, principal, NOMBRE, Subcomando::LsRemote, &args)?;
    if !s.exito() {
        return Err(match clasificar_fallo_remoto(&s.stderr) {
            FalloRemoto::SinRed | FalloRemoto::RamaInexistente => sinred(&s.primera_linea_error()),
            FalloRemoto::Local => crate::git::remoto::error_local(NOMBRE, &s.primera_linea_error()),
            otro => denegada(otro, &s.primera_linea_error()),
        });
    }
    Ok(s.stdout
        .lines()
        .filter_map(|l| l.split_whitespace().nth(1))
        .filter_map(|r| r.strip_prefix("refs/heads/"))
        .map(str::to_owned)
        .collect())
}

/// Lineas de `main` y `develop` (D3): en un repo sin commits, un commit raiz
/// con la configuracion y `.gitignore` y las dos ramas en el (publicadas si
/// hay remoto); con historia, lo hace la persona.
fn ramas(
    c: &Contexto<'_>,
    principal: &Path,
    remoto: bool,
    remotas: &[String],
    v: &mut Vec<Accion>,
) -> Result<(), Informe> {
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
    let mut nuevas: Vec<String> = Vec::new();
    let mut lineas = Vec::new();
    for r in RAMAS {
        let ruta = format!("refs/heads/{r}");
        let local = commit_de(g, principal, NOMBRE, &ruta)?.is_some();
        let en_remoto = remotas.iter().any(|x| x == r);
        let a = Accion::nueva(ruta, "rama", "proyecto", Decision::Existe);
        lineas.push(if local {
            if remoto && !en_remoto {
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
        } else if hay_commits {
            let mut o = format!("git branch {r} HEAD");
            if remoto {
                o.push_str(&format!(" && git push {REMOTO} {r}"));
            }
            persona(a, "con-historia", o)
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
    Ok(())
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
        return Ok(Err("sin-remoto"));
    };
    let Ok(destino) = detectar(&url, clave.map(|(p, a)| (*p, a.as_str()))) else {
        return Ok(Err("sin-plataforma"));
    };
    let credenciales = (c.credenciales)();
    if credenciales.token(Rol::Persona).is_none() {
        return Ok(Err("sin-token"));
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
    Ok(p.map(|p| (p, destino.proveedor))
        .map_err(|_| "sin-plataforma"))
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
        Err(m) => return Ok(nueva(Decision::Pendiente).motivo(*m)),
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
        Err(m) => Err((*m).to_owned()),
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
                    .motivo("sin-checks")
                    .orden(ORDEN_PUERTAS);
                if *prov == Proveedor::GitHub {
                    a.propuesta = Some(ruleset_github(&puerta));
                }
                a
            }
        });
    }
}
