//! `tc2 review [--json] TASK-NNN` (F6-2): empuja la rama de la tarea y abre
//! su PR contra `develop` con la identidad del agente (ADR F1-6, decision 1;
//! adenda F6-2), pasa la tarea a `en-revision` y dice como esta el check
//! `revision-ia`.
//!
//! Une piezas que ya existen, sin capacidades de plataforma nuevas salvo
//! `entorno-push`:
//!
//! 1. `push` de la rama desde el worktree de la tarea con el token del
//!    agente: [`crate::git::push`] con el puerto
//!    [`Git::derivado`] del entorno que da
//!    [`Plataforma::entorno_push`] (cabecera limitada a la URL de `origin`;
//!    el puerto de la transaccion no se toca).
//! 2. [`Plataforma::abrir_pr`] de la rama contra `develop`, idempotente.
//! 3. `en-curso` -> `en-revision` por el punto unico de escritura
//!    ([`Revisar`]), **despues** de tener el PR: el estado nunca dice «en
//!    revision» sin PR. Si falla, la linea lleva el `pr=` y repetir converge.
//! 4. [`Plataforma::estado_puertas`] con el agente para leer `revision-ia`
//!    (D1: se lee y se informa; lo publica el revisor en F6-3).
//!
//! «PR enlazado a la tarea» sin tocar el esquema (F1-9): titulo
//! `TASK-NNN: <titulo>` y cuerpo de [`cuerpo_pr`] con las rutas de la tarea
//! y de su plan en la rama de estado; `board` ya enlaza PR y tarea por la
//! rama.
//!
//! Linea: `review ok id=TASK-NNN rama=<r> pr=<n> estado=en-revision
//! revision-ia=<ok|pendiente|fallo|sin-revisor|sin-leer> revisoras=<ids>
//! modelo=<m> reglas=<ids> avisos=<...>` (la seleccion de F6-3, anotada en
//! `roles.revision`). Repetirla sin
//! nada nuevo sale `sin-cambios`.

pub mod operacion;

use std::path::Path;
use std::time::Duration;

use crate::config;
use crate::contrato::{Informe, Resultado, Valor};
use crate::estado::{self, Fallo, Operacion};
use crate::git::{self, Git};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::revision as txt;
use crate::plataforma::{ModeloPuertas, Paso, Plataforma, Rol};
use crate::seleccion;
use crate::tarea::Tarea;
use crate::toma;

pub use operacion::{AnotacionRevision, EN_REVISION, Revisar, desde_anotacion};

/// Nombre de la orden y de la operacion de la rama de estado.
pub const NOMBRE: &str = "review";

/// Rama contra la que se abre el PR (D3: los `hotfix` llegan con F7).
pub const BASE: &str = "develop";

/// Nombre del check del revisor IA.
pub const REVISION_IA: &str = "revision-ia";

/// Construye el adaptador de la plataforma de `origin` con las credenciales
/// del proceso (en produccion, la deteccion de `start` y los tokens
/// `TC2_TOKEN_*` o la App del agente).
pub type Conectar<'a> = dyn Fn() -> Paso<Box<dyn Plataforma>> + 'a;

/// Lo que necesita `review` del entorno, para probarlo sin terminal ni
/// tokens: lo de `start` mas la plataforma y las esperas del push.
pub struct Entorno<'a> {
    /// Carpeta, puerto Git, quien la lanza, catalogo e instante.
    pub base: &'a toma::Entorno<'a>,
    /// El adaptador; `review` le pide siempre con el rol de agente.
    pub conectar: &'a Conectar<'a>,
    /// Reintentos del push (en produccion, [`git::ESPERAS_PUSH`]).
    pub esperas: &'a [Duration],
    /// Carpeta de Claude Code para el inventario de la seleccion (F6-3);
    /// `None`, solo lo del proyecto en la base y lo integrado.
    pub claude: Option<&'a Path>,
}

/// `review TASK-NNN` (sin `--json`, que quita `lib.rs`).
pub fn orden(args: &[String], e: &Entorno<'_>) -> Informe {
    let uso = |motivo: &str| {
        Informe::nuevo(NOMBRE, Resultado::Uso)
            .con("motivo", motivo)
            .diagnostico(&txt::USO_REVIEW)
    };
    match args {
        [id] if git::es_id(id) => review_en(e, id),
        [_] => uso("id-invalido"),
        [] => uso("sin-id"),
        _ => uso("argumentos-de-mas"),
    }
}

/// `review` de la tarea `id` en el repo de `e`.
pub fn review_en(e: &Entorno<'_>, id: &str) -> Informe {
    let i = match paso(e, id) {
        Ok(i) | Err(i) => i,
    };
    de_review(i, id)
}

/// Titulo del PR: `TASK-NNN: <titulo>`.
pub fn titulo_pr(t: &Tarea) -> String {
    format!("{}: {}", t.id, t.titulo)
}

/// Cuerpo del PR: el ID y las rutas de la tarea y de su plan en la rama de
/// estado (el enlace PR -> tarea; el de tarea -> PR lo da `board` por la
/// rama).
pub fn cuerpo_pr(t: &Tarea) -> String {
    let (id, rama) = (&t.id, estado::RAMA);
    format!(
        "Tarea {id}: {}\n\n\
         - Tarea: `{rama}:{id}/tarea.md`\n\
         - Plan: `{rama}:{id}/planificacion/plan-final.md`\n\n\
         Abierto por `tc2 review {id}` con la identidad del agente (F6-2).\n",
        t.titulo
    )
}

/// Lo que dejo [`publicar`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Publicado {
    /// Numero del PR (en GitLab, el `iid` del MR).
    pub pr: u64,
    /// Si el push publico algo (`push ok`; `sin-cambios`, no).
    pub empujado: bool,
    /// Si el PR se acaba de abrir (`abrir-pr ok`; ya abierto, no).
    pub abierto: bool,
}

/// Empuja la rama actual del worktree `wt` a `url_remoto` (la de `origin`)
/// con el entorno del agente que da `p` y abre su PR contra [`BASE`] con
/// `titulo` y `cuerpo`. El puerto `g` no cambia: el push corre con un
/// [`Git::derivado`]. Lo usan `review` y su E2E.
pub fn publicar(
    p: &dyn Plataforma,
    g: &Git,
    wt: &Path,
    url_remoto: &str,
    (rama, titulo, cuerpo): (&str, &str, &str),
    esperas: &[Duration],
) -> Result<Publicado, Informe> {
    let agente = g.derivado(p.entorno_push(url_remoto)?);
    let push = git::push(&agente, wt, esperas);
    if !push.resultado.es_exito() {
        return Err(push);
    }
    let pr = p.abrir_pr(rama, BASE, titulo, cuerpo);
    let numero = pr.valor("pr").and_then(|n| n.parse::<u64>().ok());
    match (pr.resultado, numero) {
        (Resultado::Ok | Resultado::SinCambios, Some(n)) => Ok(Publicado {
            pr: n,
            empujado: push.resultado == Resultado::Ok,
            abierto: pr.resultado == Resultado::Ok,
        }),
        _ => Err(pr),
    }
}

/// Como esta `revision-ia` en el PR `pr`, leido con el agente: `sin-revisor`
/// si la configuracion no da su origen (`plataforma.revisor_id`, TASK-118),
/// y si no `ok`, `pendiente` o `fallo` segun `estado-puertas` con solo ese
/// check en el modelo; `sin-leer` si la lectura falla (no cambia el
/// resultado de `review`).
pub fn revision_ia(p: &dyn Plataforma, pr: u64, revisor_id: Option<u64>) -> &'static str {
    let Some(id) = revisor_id.filter(|id| *id != 0) else {
        return "sin-revisor";
    };
    let modelo = ModeloPuertas {
        checks: vec![(REVISION_IA.to_owned(), id)],
        exigir_impuestas: Vec::new(),
    };
    let i = p.estado_puertas(Rol::Agente, pr, &modelo);
    let en = |clave: &str| {
        i.claves.iter().any(|(k, v)| {
            k == clave && matches!(v, Valor::Lista(l) if l.iter().any(|c| c == REVISION_IA))
        })
    };
    match i.resultado {
        Resultado::Rechazada if i.valor("motivo") == Some("check-rojo") && en("checks") => "fallo",
        Resultado::Esperando if en("falta") => "pendiente",
        Resultado::Ok | Resultado::Esperando => "ok",
        _ => "sin-leer",
    }
}

/// El informe como de `review`: el comando, `id=` el primero y sin la clave
/// `operacion` de la transaccion. Un `sinred` dice `pendiente=no` (`review`
/// exige red).
fn de_review(mut i: Informe, id: &str) -> Informe {
    NOMBRE.clone_into(&mut i.comando);
    i.claves.retain(|(k, _)| k != "operacion" && k != "id");
    i.claves
        .insert(0, ("id".to_owned(), Valor::Texto(id.to_owned())));
    if i.resultado == Resultado::Sinred && i.valor("pendiente").is_none() {
        i = i.con("pendiente", "no");
    }
    i
}

fn rechazada(motivo: &str, m: Mensaje) -> Informe {
    Informe::nuevo(NOMBRE, Resultado::Rechazada)
        .con("motivo", motivo)
        .diagnostico(m)
}

fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
    let (g, dir, ahora) = (e.base.git, e.base.dir, e.base.ahora);
    let principal = git::localizar_principal(g, dir)?;
    // 1. Lo de ahora del remoto: el estado y la rama.
    if git::estado::hay_remoto(g, &principal, NOMBRE)? {
        let f = git::fetch(g, &principal);
        if !f.resultado.es_exito() {
            return Err(f);
        }
    }
    let vista = estado::leer(g, dir)?;
    let quien = (e.base.quien)()?;
    // 2. El paso a revision en seco: los rechazos, antes de empujar nada.
    let sondeo = Revisar::nueva(id, &quien, ahora);
    if let Err(f) = sondeo.aplicar(&vista) {
        return Err(match f {
            Fallo::Rechazada {
                motivo,
                diagnostico,
            } => {
                let i = rechazada(&motivo, diagnostico);
                match sondeo.dueno() {
                    Some(l) => i.con("asignado", l),
                    None => i,
                }
            }
            Fallo::Error(d) => Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d),
        });
    }
    let (t, cuerpo) = match estado::tarea_de(&vista, id) {
        Ok(tc) => tc,
        Err(Fallo::Rechazada {
            motivo,
            diagnostico,
        }) => return Err(rechazada(&motivo, diagnostico)),
        Err(Fallo::Error(d)) => return Err(Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d)),
    };
    // 3. La rama, contra `develop` (D3) y con su worktree.
    let Some(rama) = t.rama.clone() else {
        return Err(rechazada(
            "sin-rama",
            Mensaje::nuevo(&txt::SIN_RAMA).con("id", id),
        ));
    };
    if git::base_de_rama(&rama) != config::BASE {
        return Err(rechazada(
            "carril-no-soportado",
            Mensaje::nuevo(&txt::CARRIL_NO_SOPORTADO)
                .con("id", id)
                .con("rama", rama.as_str()),
        )
        .con("rama", rama.as_str()));
    }
    let wt = principal.join(git::CARPETA_WORKTREES).join(id);
    let registrado = wt.exists()
        && git::worktrees(g, &principal, NOMBRE)?
            .iter()
            .any(|w| w.rama.as_deref() == Some(rama.as_str()) && git::misma_ruta(&w.ruta, &wt));
    if !registrado {
        return Err(rechazada(
            "sin-worktree",
            Mensaje::nuevo(&txt::SIN_WORKTREE)
                .con("id", id)
                .con("ruta", git::worktree_relativo(id))
                .con("rama", rama.as_str()),
        )
        .con("rama", rama.as_str())
        .con("arreglo", format!("tc2 start {id}")));
    }
    let Some(url) = git::estado::url_origin(g, &wt, NOMBRE)? else {
        return Err(rechazada("plataforma", Mensaje::nuevo(&txt::SIN_ORIGIN)));
    };
    // 4. La seleccion de revision (F6-3), con el diff de la rama y lo de la
    // base; no cambia nada si falla: lo dice en `avisos`.
    let sel = seleccion_de(g, &principal, &rama, (&t, &cuerpo), e.claude);
    // 5. Push y PR con el agente.
    let p = (e.conectar)()?;
    let publicado = publicar(
        p.as_ref(),
        g,
        &wt,
        &url,
        (&rama, &titulo_pr(&t), &cuerpo_pr(&t)),
        e.esperas,
    )?;
    let pr = publicado.pr.to_string();
    // 6. El estado, con el PR ya abierto (y la seleccion en `roles.revision`).
    let personas = crate::roadmap::personas(g, dir);
    let generar = crate::roadmap::generador(&personas);
    let ctx = estado::Contexto::nuevo(g, dir, e.base.catalogo, Some(&generar));
    let op = Revisar::nueva(id, &quien, ahora).con_seleccion(AnotacionRevision {
        agente: sel.agentes.first().cloned(),
        modelo: sel.modelo.clone(),
        skills: sel.revisoras.clone(),
    });
    let tr = estado::transaccion(&ctx, &op);
    match tr.resultado {
        Resultado::Ok | Resultado::SinCambios => {}
        _ => {
            let mut tr = tr.con("rama", rama.as_str()).con("pr", pr);
            if let Some(l) = op.dueno() {
                tr = tr.con("asignado", l);
            }
            return Err(tr);
        }
    }
    // 7. `revision-ia`, leido con el agente.
    let revisor = config::de_base(g, &principal)
        .ok()
        .and_then(|c| c.plataforma)
        .and_then(|p| p.revisor_id);
    let ria = revision_ia(p.as_ref(), publicado.pr, revisor);
    let nada_nuevo =
        !publicado.empujado && !publicado.abierto && tr.resultado == Resultado::SinCambios;
    let resultado = if nada_nuevo {
        Resultado::SinCambios
    } else {
        Resultado::Ok
    };
    Ok(Informe::nuevo(NOMBRE, resultado)
        .con("rama", rama.as_str())
        .con("pr", pr)
        .con("estado", EN_REVISION)
        .con(REVISION_IA, ria)
        .con_lista("revisoras", sel.revisoras)
        .con("modelo", sel.modelo)
        .con_lista("reglas", sel.reglas)
        .con_lista("avisos", sel.avisos))
}

/// La seleccion de revision de la tarea `t` en la rama `rama` (F6-3): rutas
/// de `git diff --no-renames origin/develop...<rama>`, inventario y catalogo
/// de [`seleccion::orden::fuentes`], carril y complejidad de
/// [`seleccion::orden::de_tarea`] y `modelos_permitidos` de la base. Lo que
/// no se puede leer sale en `avisos` (`sin-diff`, `sin-base`...).
pub fn seleccion_de(
    g: &Git,
    principal: &Path,
    rama: &str,
    (t, cuerpo): (&Tarea, &crate::tarea::Cuerpo),
    claude: Option<&Path>,
) -> seleccion::Seleccion {
    let mut avisos = Vec::new();
    let rutas: Vec<String> = git::ficheros_de_rama(g, principal, NOMBRE, config::BASE, rama)
        .map(|r| r.into_iter().collect())
        .unwrap_or_else(|_| {
            avisos.push("sin-diff".to_owned());
            Vec::new()
        });
    let f = seleccion::orden::fuentes(g, principal, claude, &[]);
    avisos.extend(f.avisos.iter().cloned());
    let (carril, cx) = seleccion::orden::de_tarea(t, cuerpo);
    let cfg = config::de_base(g, principal).unwrap_or_else(|_| {
        avisos.push("config-de-base-no-leida".to_owned());
        config::Config::suelo()
    });
    let ctx = seleccion::Contexto {
        carril,
        complejidad: Some(cx),
        modelos_permitidos: &cfg.seleccion.modelos_permitidos,
        ronda: 1,
    };
    let mut s = seleccion::revision(&rutas, &f.catalogo, &f.inventario, &ctx).unwrap_or_default();
    avisos.append(&mut s.avisos);
    s.avisos = avisos;
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plataforma::{Firma, PrAbierto, ReglasLeidas, Usuario};
    use std::cell::RefCell;

    /// Plataforma de prueba: solo `estado-puertas`, con el informe fijo y
    /// el modelo recibido.
    struct Puertas {
        informe: Informe,
        modelo: RefCell<Option<ModeloPuertas>>,
    }

    fn sin_uso(op: &str) -> Informe {
        Informe::nuevo(op, Resultado::Error)
    }

    impl Plataforma for Puertas {
        fn prs_abiertos(&self, _: Rol) -> Paso<Vec<PrAbierto>> {
            Err(sin_uso("prs-abiertos"))
        }
        fn abrir_pr(&self, _: &str, _: &str, _: &str, _: &str) -> Informe {
            sin_uso("abrir-pr")
        }
        fn estado_puertas(&self, rol: Rol, _: u64, modelo: &ModeloPuertas) -> Informe {
            assert_eq!(rol, Rol::Agente);
            *self.modelo.borrow_mut() = Some(modelo.clone());
            self.informe.clone()
        }
        fn merge_pr(&self, _: u64, _: &str) -> Informe {
            sin_uso("merge-pr")
        }
        fn leer_puertas(&self, _: Rol, _: &str) -> Paso<ReglasLeidas> {
            Err(sin_uso("leer-puertas"))
        }
        fn leer_permisos_ci(&self) -> Informe {
            sin_uso("leer-permisos-ci")
        }
        fn firma_commit(&self, _: &str) -> Paso<Firma> {
            Err(sin_uso("firma-commit"))
        }
        fn identidad(&self, _: Rol) -> Paso<Usuario> {
            Err(sin_uso("identidad"))
        }
    }

    fn con(informe: Informe) -> Puertas {
        Puertas {
            informe,
            modelo: RefCell::new(None),
        }
    }

    /// Criterio «check `revision-ia`» (unitario): sin `revisor_id` no se
    /// pregunta (`sin-revisor`); con el, el modelo lleva solo ese check con
    /// su origen y la linea de `estado-puertas` se traduce.
    #[test]
    fn revision_ia_de_estado_puertas() {
        let p = con(Informe::nuevo("estado-puertas", Resultado::Ok));
        assert_eq!(revision_ia(&p, 7, None), "sin-revisor");
        assert_eq!(revision_ia(&p, 7, Some(0)), "sin-revisor");
        assert!(p.modelo.borrow().is_none());
        assert_eq!(revision_ia(&p, 7, Some(99)), "ok");
        assert_eq!(
            p.modelo.borrow().as_ref().map(|m| m.checks.clone()),
            Some(vec![("revision-ia".to_owned(), 99)])
        );
        let esperando = |falta: &[&str]| {
            Informe::nuevo("estado-puertas", Resultado::Esperando)
                .con_lista("falta", falta.iter().map(|s| (*s).to_owned()).collect())
        };
        let p = con(esperando(&["revision-ia", "aprobacion"]));
        assert_eq!(revision_ia(&p, 7, Some(99)), "pendiente");
        let p = con(esperando(&["aprobacion"]));
        assert_eq!(revision_ia(&p, 7, Some(99)), "ok");
        let rojo = Informe::nuevo("estado-puertas", Resultado::Rechazada)
            .con("motivo", "check-rojo")
            .con_lista("checks", vec!["revision-ia".to_owned()]);
        assert_eq!(revision_ia(&con(rojo), 7, Some(99)), "fallo");
        let p = con(Informe::nuevo("estado-puertas", Resultado::Sinred));
        assert_eq!(revision_ia(&p, 7, Some(99)), "sin-leer");
    }

    /// Criterio «PR enlazado» (unitario): titulo con el ID y cuerpo con las
    /// rutas de la tarea y del plan en la rama de estado.
    #[test]
    fn titulo_y_cuerpo_del_pr() {
        let t = Tarea::nueva("TASK-048", "Review abre el PR", "2026-11-19");
        assert_eq!(titulo_pr(&t), "TASK-048: Review abre el PR");
        assert_eq!(
            cuerpo_pr(&t),
            "Tarea TASK-048: Review abre el PR\n\n\
             - Tarea: `taskcodetwo/tareas:TASK-048/tarea.md`\n\
             - Plan: `taskcodetwo/tareas:TASK-048/planificacion/plan-final.md`\n\n\
             Abierto por `tc2 review TASK-048` con la identidad del agente (F6-2).\n"
        );
    }
}
