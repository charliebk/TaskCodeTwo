//! `tc2 next [--json]` (F3-3): lo listo para empezar por dependencias y
//! prioridad.
//!
//! Lee el estado tal como lo conoce este clon ([`estado::leer`]: sin
//! `fetch` y sin escribir; la lectura fresca es de F3-5b) y calcula con
//! [`crate::situacion`], lo mismo que `roadmap` (TASK-031 C4). El orden de
//! los hitos sale de los titulos `### Hn — titulo` de la zona manual del
//! `ROADMAP.md` de la rama de estado (D2 de TASK-030).
//!
//! Salida: `next ok listas=... en-marcha=... commit=<sha> [avisos]`, con
//! `commit=` la foto del estado usada. Avisos, solo si los hay:
//! `ilegibles=` (`tarea.md` que no se entienden: no se saltan en silencio),
//! `ciclos=` (las atascadas), `dependencias-inexistentes=`,
//! `depende-de-descartada=` (`A>B`), `hitos-sin-titulo=`, `duplicadas=`
//! (IDs repetidos: vale el primero) y `roadmap-roto=` (la zona manual del
//! ROADMAP, rota: D3 de TASK-031). Sin rama de estado, `rechazada`.

use std::path::Path;

use crate::contrato::{Informe, Resultado, corto};
use crate::estado::{self, Estado, Fallo};
use crate::git::Git;
use crate::i18n::Mensaje;
use crate::i18n::catalogo::cli as txt_cli;
use crate::i18n::catalogo::siguiente as txt;
use crate::situacion::{self, Situacion};

/// Nombre de la orden.
pub const NOMBRE: &str = "next";

/// `next [--json]` en la carpeta actual (sin `--json`, que quita `lib.rs`).
pub fn orden(args: &[String]) -> Informe {
    if !args.is_empty() {
        return Informe::nuevo(NOMBRE, Resultado::Uso)
            .con("motivo", "argumentos-de-mas")
            .diagnostico(&txt::USO_NEXT);
    }
    match std::env::current_dir() {
        Ok(dir) => siguiente_en(&Git::nuevo(), &dir),
        Err(e) => Informe::nuevo(NOMBRE, Resultado::Error)
            .diagnostico(Mensaje::nuevo(&txt_cli::CARPETA_ACTUAL).con("error", e.to_string())),
    }
}

/// `next` sobre el estado que conoce el clon de `dir`.
pub fn siguiente_en(g: &Git, dir: &Path) -> Informe {
    let vista = match estado::leer(g, dir) {
        Ok(v) => v,
        Err(mut i) => {
            NOMBRE.clone_into(&mut i.comando);
            return i;
        }
    };
    let commit = vista.commit().map(corto).unwrap_or_default();
    match calcular(&vista) {
        Ok(c) => c.informe(&commit),
        Err(Fallo::Rechazada {
            motivo,
            diagnostico,
        }) => Informe::nuevo(NOMBRE, Resultado::Rechazada)
            .con("motivo", motivo)
            .diagnostico(diagnostico),
        Err(Fallo::Error(d)) => Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d),
    }
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

/// Listas, en marcha e ilegibles de `estado`.
fn calcular(estado: &dyn Estado) -> Result<Calculo, Fallo> {
    situacion::calcular(estado).map(Calculo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    /// Estado en memoria.
    struct EnMemoria(BTreeMap<String, String>);

    impl Estado for EnMemoria {
        fn rutas(&self) -> Vec<String> {
            self.0.keys().cloned().collect()
        }
        fn leer(&self, ruta: &str) -> Result<Option<String>, Fallo> {
            Ok(self.0.get(ruta).cloned())
        }
    }

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
            tarea("TASK-001", 0, "planificada", &[]),
        );
        m.insert("TASK-002/tarea.md".into(), "---\nid: roto\n".into());
        m.insert("TASK-003/plan.md".into(), "no es un tarea.md".into());
        let c = calcular(&EnMemoria(m));
        assert!(c.is_ok(), "{c:?}");
        let Ok(c) = c else { return };
        assert_eq!(
            c.informe("0123456789ab").linea(),
            // Sin ROADMAP.md, el hito va por su numero y se avisa.
            "next ok listas=TASK-001 en-marcha= commit=0123456789ab ilegibles=TASK-002 hitos-sin-titulo=H0"
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
        let i = orden(&["x".to_owned()]);
        assert_eq!(i.linea(), "next uso motivo=argumentos-de-mas");
    }
}
