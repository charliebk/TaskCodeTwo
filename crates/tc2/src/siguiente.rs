//! `tc2 next [--json]` (F3-3): lo listo para empezar por dependencias y
//! prioridad.
//!
//! Trae antes la rama de estado ([`estado::leer_fresco`], F3-5b: solo
//! escribe `origin/taskcodetwo/tareas` y la marca de sincronizacion) y
//! calcula con [`crate::situacion`], lo mismo que `roadmap` (TASK-031 C4).
//! El orden de los hitos sale de los titulos `### Hn — titulo` de la zona
//! manual del `ROADMAP.md` de la rama de estado (D2 de TASK-030).
//!
//! Salida: `next ok listas=... en-marcha=... commit=<sha> [avisos]
//! remoto=... sincronizado=...`, con `commit=` la foto del estado usada; sin
//! red, lo ultimo traido con un aviso en stderr y codigo 0. Avisos, solo si
//! los hay:
//! `ilegibles=` (`tarea.md` que no se entienden: no se saltan en silencio),
//! `ciclos=` (las atascadas), `dependencias-inexistentes=`,
//! `depende-de-descartada=` (`A>B`), `hitos-sin-titulo=`, `duplicadas=`
//! (IDs repetidos: vale el primero) y `roadmap-roto=` (la zona manual del
//! ROADMAP, rota: D3 de TASK-031). Sin rama de estado, `rechazada`.

use std::path::Path;

use crate::Salida;
use crate::contrato::{Informe, Resultado, corto};
use crate::estado::{self, Estado, Fallo};
use crate::git::Git;
use crate::i18n::catalogo::cli as txt_cli;
use crate::i18n::catalogo::siguiente as txt;
use crate::i18n::{Idioma, Mensaje};
use crate::situacion::{self, Situacion};

/// Nombre de la orden.
pub const NOMBRE: &str = "next";

/// `next [--json]` en la carpeta actual (sin `--json`, que quita `lib.rs`).
/// Si no se pudo traer la rama de estado, el aviso va a stderr con codigo 0.
pub fn orden(args: &[String], json: bool, idioma: Idioma) -> Salida {
    if !args.is_empty() {
        return Informe::nuevo(NOMBRE, Resultado::Uso)
            .con("motivo", "argumentos-de-mas")
            .diagnostico(&txt::USO_NEXT)
            .salida_en(json, idioma);
    }
    match std::env::current_dir() {
        Ok(dir) => {
            let (i, aviso) = siguiente(&Git::nuevo(), &dir, crate::identidad::ahora());
            let mut s = i.salida_en(json, idioma);
            if let (0, Some(a)) = (s.codigo, aviso) {
                s.stderr.push_str(&a.texto(idioma));
                s.stderr.push('\n');
            }
            s
        }
        Err(e) => Informe::nuevo(NOMBRE, Resultado::Error)
            .diagnostico(Mensaje::nuevo(&txt_cli::CARPETA_ACTUAL).con("error", e.to_string()))
            .salida_en(json, idioma),
    }
}

/// `next` sobre el estado recien traido al clon de `dir` (F3-5b).
pub fn siguiente_en(g: &Git, dir: &Path) -> Informe {
    siguiente(g, dir, crate::identidad::ahora()).0
}

/// `next` tras traer la rama de estado: el informe (con `remoto=` y
/// `sincronizado=`) y el aviso de frescura, si no se pudo traer.
fn siguiente(g: &Git, dir: &Path, ahora: u64) -> (Informe, Option<Mensaje>) {
    let (vista, frescura) = match estado::leer_fresco(g, dir, ahora) {
        Ok(v) => v,
        Err(mut i) => {
            NOMBRE.clone_into(&mut i.comando);
            return (i, None);
        }
    };
    let aviso = frescura.aviso(ahora);
    let commit = vista.commit().map(corto).unwrap_or_default();
    let i = match calcular(&vista) {
        Ok(c) => frescura.anotar(c.informe(&commit)),
        Err(f) => f.informe(NOMBRE),
    };
    (i, aviso)
}

/// Lo que se calcula sobre el estado: la [`Situacion`] comun con `roadmap`.
#[derive(Debug, Default)]
struct Calculo(Situacion);

impl Calculo {
    fn informe(self, commit: &str) -> Informe {
        let roto = self.0.roadmap_roto();
        let s = self.0;
        let a = s.orden.avisos;
        let mut i = Informe::nuevo(NOMBRE, Resultado::Ok)
            .con_lista("listas", s.listas)
            .con_lista("en-marcha", s.en_marcha)
            .con("commit", commit);
        let hitos: Vec<String> = a.hitos_sin_titulo.iter().map(|h| format!("H{h}")).collect();
        for (clave, v) in [
            ("ilegibles", s.ilegibles),
            ("ciclos", s.orden.atascadas),
            ("dependencias-inexistentes", a.dependencias_inexistentes),
            ("depende-de-descartada", a.depende_de_descartada),
            ("hitos-sin-titulo", hitos),
            ("duplicadas", a.duplicadas),
        ] {
            if !v.is_empty() {
                i = i.con_lista(clave, v);
            }
        }
        // D3 de TASK-031: la zona manual rota se avisa.
        if let Some(r) = roto {
            i = i.con("roadmap-roto", r.motivo());
        }
        i
    }
}

/// El informe de `next` sobre `estado` con la foto `commit` (para comparar
/// con `board` y `roadmap`, TASK-032 C3a).
pub fn informe_de(estado: &dyn Estado, commit: &str) -> Result<Informe, Fallo> {
    calcular(estado).map(|c| c.informe(commit))
}

/// Listas, en marcha e ilegibles de `estado`.
fn calcular(estado: &dyn Estado) -> Result<Calculo, Fallo> {
    situacion::calcular(estado).map(Calculo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::estado::prueba::EnMemoria;
    use std::collections::BTreeMap;

    fn tarea(id: &str, sprint: u32, estado: &str, deps: &[&str]) -> String {
        let mut t = crate::tarea::Tarea::nueva(id, "t", "2026-01-01");
        t.sprint = sprint;
        t.estado = estado.to_owned();
        t.dependencias = deps.iter().map(|d| (*d).to_owned()).collect();
        t.escribir(&crate::tarea::Cuerpo::leer(""))
    }

    /// C4: un `tarea.md` ilegible no se salta en silencio.
    #[test]
    fn ilegible_sale_en_ilegibles() {
        let mut m = BTreeMap::new();
        m.insert(
            "TASK-001/tarea.md".into(),
            tarea("TASK-001", 1, "planificada", &[]),
        );
        m.insert("TASK-002/tarea.md".into(), "---\nid: roto\n".into());
        m.insert("TASK-003/plan.md".into(), "no es un tarea.md".into());
        let c = calcular(&EnMemoria(m));
        assert!(c.is_ok(), "{c:?}");
        let Ok(c) = c else { return };
        assert_eq!(
            c.informe("0123456789ab").linea(),
            // Sin ROADMAP.md vale la zona por defecto, que solo declara H0:
            // H1 va por su numero y se avisa.
            "next ok listas=TASK-001 en-marcha= commit=0123456789ab ilegibles=TASK-002 hitos-sin-titulo=H1"
        );
    }

    /// El orden de los hitos sale de la zona manual del ROADMAP.md.
    #[test]
    fn hitos_del_roadmap_de_la_rama_de_estado() {
        let mut m = BTreeMap::new();
        m.insert(
            "TASK-001/tarea.md".into(),
            tarea("TASK-001", 1, "planificada", &[]),
        );
        m.insert(
            "TASK-002/tarea.md".into(),
            tarea("TASK-002", 2, "planificada", &[]),
        );
        m.insert(
            "TASK-003/tarea.md".into(),
            tarea("TASK-003", 3, "en-curso", &[]),
        );
        m.insert(
            situacion::ROADMAP.into(),
            "<!-- manual:inicio -->\n### H2 — Dos\n### H1 — Uno\n<!-- manual:fin -->\n".into(),
        );
        let c = calcular(&EnMemoria(m.clone()));
        let Ok(c) = c else { return };
        assert_eq!(
            c.informe("0123456789ab").linea(),
            "next ok listas=TASK-002,TASK-001 en-marcha=TASK-003 commit=0123456789ab hitos-sin-titulo=H3"
        );
        // D3 de TASK-031: sin la marca de fin, el orden sigue (lectura
        // tolerante) y se avisa.
        m.insert(
            situacion::ROADMAP.into(),
            "<!-- manual:inicio -->\n### H2 — Dos\n### H1 — Uno\n".into(),
        );
        let c = calcular(&EnMemoria(m));
        let Ok(c) = c else { return };
        assert_eq!(
            c.informe("0123456789ab").linea(),
            "next ok listas=TASK-002,TASK-001 en-marcha=TASK-003 commit=0123456789ab hitos-sin-titulo=H3 roadmap-roto=sin-fin"
        );
    }

    #[test]
    fn argumentos_de_mas_es_uso() {
        let s = orden(&["x".to_owned()], false, Idioma::Es);
        assert_eq!(
            (s.codigo, s.stdout.as_str()),
            (2, "next uso motivo=argumentos-de-mas\n")
        );
    }
}
