//! `tc2 roadmap` (F3-4): la vista para todos, en `ROADMAP.md` de la rama de
//! estado (ADR F1-1, decision 3).
//!
//! - [`de_estado`] genera el `ROADMAP.md` con el mismo calculo que `next`
//!   ([`crate::situacion`]) y la zona manual del que ya hay, byte a byte; si
//!   esa zona esta rota, devuelve el anterior tal cual (D3: no se bloquea la
//!   operacion; `next` y `--comparar` lo avisan). Es el generador que recibe
//!   cada transaccion de estado ([`crate::estado::Contexto::nuevo`]): se
//!   regenera, nunca se mergea (ADR F1-2, D2.4).

pub mod generar;
pub mod mermaid;
pub mod zona;

use std::path::Path;

use crate::estado::{Estado, Fallo};
use crate::git::Git;
use crate::plataforma::Personas;
use crate::situacion;

/// Lista de personas de la base para convertir los `asignado_a` heredados
/// (D6); sin lista, ninguna: se muestran tal cual.
pub fn personas(g: &Git, dir: &Path) -> Personas {
    crate::identidad::personas_de_base(g, dir).unwrap_or_default()
}

/// El generador que recibe cada transaccion de estado.
pub fn generador(personas: &Personas) -> impl Fn(&dyn Estado) -> Result<String, Fallo> + '_ {
    move |e: &dyn Estado| de_estado(e, personas)
}

/// El `ROADMAP.md` de `estado`: generado alrededor de su zona manual (o la
/// de por defecto), o el que hay tal cual si su zona esta rota (D3).
pub fn de_estado(estado: &dyn Estado, personas: &Personas) -> Result<String, Fallo> {
    let s = situacion::calcular(estado)?;
    let zona = match s.roadmap.as_deref().map(zona::extraer) {
        None | Some(Ok(None)) => zona::POR_DEFECTO,
        Some(Ok(Some(z))) => z,
        Some(Err(_)) => return Ok(s.roadmap.clone().unwrap_or_default()),
    };
    Ok(generar::generar(&s, zona, personas))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use crate::tarea::{Asignado, Cuerpo, Tarea};

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

    fn v2(
        id: &str,
        titulo: &str,
        sprint: u32,
        estado: &str,
        deps: &[&str],
        asignado: Option<&str>,
    ) -> String {
        let mut t = Tarea::nueva(id, titulo, "2026-01-01");
        t.sprint = sprint;
        t.estado = estado.to_owned();
        if estado == "descartada" {
            t.motivo = Some("ya no hace falta".to_owned());
        }
        t.dependencias = deps.iter().map(|d| (*d).to_owned()).collect();
        if let Some(login) = asignado {
            t.asignado_a = Asignado::persona(7, login);
        }
        t.escribir(&Cuerpo::leer(""))
    }

    /// Formato 1 (0.16), con `asignado_a` de texto libre.
    fn v1(id: &str, titulo: &str, sprint: u32, estado: &str, asignado: &str) -> String {
        format!(
            "---\nid: {id}\ntitulo: \"{titulo}\"\ntipo: feature\nsprint: {sprint}\netiquetas: []\n\
             complejidad: null\nmodelo_sugerido: sonnet\nestado: {estado}\nplan_aprobado: false\n\
             rama: feature/x\nasignado_a: {asignado}\nagente_revisor: general-purpose\n\
             skills_recomendados: []\nregla_seleccion_skill: null\nultimo_commit_revisado: null\n\
             revision_codex: false\ntokens_diseno: null\ntokens_implementacion: null\n\
             tokens_revision: null\ncreado: 2026-01-01\nactualizado: 2026-01-01\n\
             dependencias: []\n---\n"
        )
    }

    const ZONA: &str = "<!-- manual:inicio (a mano) -->\r\n## Hitos\r\n\r\n### H2 — Dos\r\nEl motivo.\r\n### H1 — Uno\r\n### H3 — Tres\r\n<!-- manual:fin -->\r\n";

    /// Un estado que pasa por cada seccion.
    fn estado_de_referencia() -> EnMemoria {
        let mut m = BTreeMap::new();
        for (id, md) in [
            (
                "TASK-001",
                v2("TASK-001", "Base", 1, "terminada", &[], None),
            ),
            (
                "TASK-002",
                v2(
                    "TASK-002",
                    "Motor",
                    1,
                    "en-curso",
                    &["TASK-001"],
                    Some("ana"),
                ),
            ),
            (
                "TASK-003",
                v1("TASK-003", "Revisar", 1, "en-revision", "ana@x.com"),
            ),
            (
                "TASK-004",
                v2(
                    "TASK-004",
                    "Usa el motor",
                    2,
                    "planificada",
                    &["TASK-002"],
                    None,
                ),
            ),
            (
                "TASK-005",
                v2("TASK-005", "Lista", 2, "planificada", &["TASK-001"], None),
            ),
            (
                "TASK-006",
                v2("TASK-006", "Ya no", 2, "descartada", &[], None),
            ),
            (
                "TASK-007",
                v2(
                    "TASK-007",
                    "Tras la descartada",
                    2,
                    "planificada",
                    &["TASK-006", "TASK-099"],
                    None,
                ),
            ),
            (
                "TASK-008",
                v1(
                    "TASK-008",
                    "Con a | b <!-- y \\\"comillas\\\" #1",
                    2,
                    "en-diseno",
                    "alguien de fuera",
                ),
            ),
            (
                "TASK-009",
                v2("TASK-009", "Cerrada", 3, "terminada", &[], None),
            ),
            ("TASK-010", "---\nid: roto\n".to_owned()),
            (
                "TASK-011",
                v2("TASK-011", "Ciclo a", 2, "planificada", &["TASK-012"], None),
            ),
            (
                "TASK-012",
                v2(
                    "TASK-012",
                    "Ciclo b",
                    2,
                    "planificada",
                    &["TASK-011", "TASK-010"],
                    None,
                ),
            ),
        ] {
            m.insert(format!("{id}/tarea.md"), md);
        }
        m.insert(
            situacion::ROADMAP.to_owned(),
            format!("# Viejo\n\n{ZONA}\n## Lo generado antes\n"),
        );
        EnMemoria(m)
    }

    fn personas() -> Personas {
        Personas::leer("personas:\n  - {id: 9, login: \"ana-x\", correos: [\"ana@x.com\"]}\n")
            .unwrap_or_default()
    }

    const REFERENCIA: &str = include_str!("../../tests/fixtures/roadmap/referencia.md");

    /// C5: la salida de referencia cubre cada seccion (con quien, revision,
    /// hito con dependencias y Mermaid, terminadas y descartadas); C2: la
    /// zona manual, con CRLF, byte a byte. Para rehacer la referencia:
    /// `TC2_REESCRIBIR=1 cargo test -p tc2 --lib referencia`.
    #[test]
    fn salida_de_referencia() {
        let r = de_estado(&estado_de_referencia(), &personas());
        assert!(r.is_ok(), "{r:?}");
        let Ok(r) = r else { return };
        if std::env::var_os("TC2_REESCRIBIR").is_some() {
            let ruta = concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/roadmap/referencia.md"
            );
            assert!(std::fs::write(ruta, &r).is_ok());
            return;
        }
        assert_eq!(r, REFERENCIA);
        assert_eq!(zona::extraer(&r), Ok(Some(ZONA)));
        for seccion in [
            "\n## Estado del trabajo\n",
            "\n## En curso\n",
            "\n## En revisión\n",
            "\n## H2 — Dos\n",
            "\n## H1 — Uno\n",
            "\n## Terminadas\n",
            "\n## Descartadas\n",
            "```mermaid\n",
            "| @ana |",
            "| @ana-x |",
            "<details>",
        ] {
            assert!(r.contains(seccion), "falta {seccion:?}");
        }
        assert!(!r.contains("\n## H3"), "un hito cerrado no lleva apartado");
        // Mismos datos, mismos bytes (lo que da `sin-cambios`).
        let mut otra = estado_de_referencia();
        otra.0.insert(situacion::ROADMAP.to_owned(), r.clone());
        assert_eq!(de_estado(&otra, &personas()).ok(), Some(r));
    }

    /// D3: con la zona rota se devuelve el anterior tal cual.
    #[test]
    fn zona_rota_deja_el_anterior() {
        let mut e = estado_de_referencia();
        let roto = "# Roadmap\n<!-- manual:inicio -->\n### H1 — Uno\n".to_owned();
        e.0.insert(situacion::ROADMAP.to_owned(), roto.clone());
        assert_eq!(de_estado(&e, &Personas::default()).ok(), Some(roto));
    }

    /// Sin ROADMAP.md (o sin zona), la zona de por defecto.
    #[test]
    fn sin_roadmap_la_zona_por_defecto() {
        let mut e = estado_de_referencia();
        e.0.remove(situacion::ROADMAP);
        let r = de_estado(&e, &Personas::default()).unwrap_or_default();
        assert_eq!(zona::extraer(&r), Ok(Some(zona::POR_DEFECTO)));
        assert!(r.starts_with(&format!("# Roadmap de tareas\n\n{}\n\n", generar::MARCA)));
    }
}
