//! Nucleo puro del orden de trabajo (F3-3; lo reutiliza `roadmap`, F3-4).
//!
//! El oraculo es `orden_de_trabajo` de `scripts/grafo-tareas.py` de ToolsaWeb
//! (D1 de TASK-030): Kahn con prioridad sobre todas las tareas no
//! descartadas, terminadas incluidas, de modo que una dependencia terminada
//! de un hito posterior retrasa a quien depende de ella. A igualdad, primero
//! la del hito anterior (por la posicion de su titulo `### Hn — titulo`, no
//! por su numero; D2) y luego la de numero y sufijo menores (D4: no hay
//! campo `prioridad`).
//!
//! - Lista para empezar: `planificada` con todas sus dependencias existentes
//!   `terminada`. Una dependencia inexistente no bloquea (aviso); una
//!   descartada sale del orden pero bloquea (D3, aviso).
//! - Ciclo: sus tareas, y las que dependen de ellas, quedan `atascadas`,
//!   fuera del orden y de las listas.
//!
//! Hoja pura: no lee git ni la rama de estado.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap};

use crate::tarea::Documento;

/// Estado de una tarea `planificada` (la unica que puede estar lista).
pub const PLANIFICADA: &str = "planificada";
/// Estado de una tarea terminada.
pub const TERMINADA: &str = "terminada";
/// Estado (formato 2) o etiqueta (formato 1) de una tarea descartada.
pub const DESCARTADA: &str = "descartada";
/// Estados de las tareas que alguien lleva entre manos.
pub const EN_MARCHA: [&str; 3] = ["en-diseno", "en-curso", "en-revision"];

/// Si una tarea cuenta para el orden o esta descartada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fase {
    /// No descartada, con su `estado`.
    Viva(String),
    /// Descartada: fuera del orden, pero bloquea a quien dependa de ella.
    Descartada,
}

/// Lo minimo de una tarea para ordenarla.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nodo {
    /// `TASK-NNN` o `TASK-NNNa`.
    pub id: String,
    /// Hito (`sprint` del frontmatter).
    pub sprint: u32,
    /// Viva con su estado, o descartada.
    pub fase: Fase,
    /// IDs de los que depende, tal como estan en el frontmatter.
    pub dependencias: Vec<String>,
}

impl Nodo {
    /// Proyecta un `tarea.md` leido. En formato 1 (0.16) se descarta con la
    /// etiqueta `descartada`; en formato 2, con el estado `descartada`.
    pub fn de(doc: &Documento) -> Self {
        match doc {
            Documento::Formato1(t, _) => Self {
                id: t.id.clone(),
                sprint: t.sprint,
                fase: if t.etiquetas.iter().any(|e| e == DESCARTADA) {
                    Fase::Descartada
                } else {
                    Fase::Viva(t.estado.clone())
                },
                dependencias: t.dependencias.clone(),
            },
            Documento::Formato2(t, _) => Self {
                id: t.id.clone(),
                sprint: t.sprint,
                fase: if t.estado == DESCARTADA {
                    Fase::Descartada
                } else {
                    Fase::Viva(t.estado.clone())
                },
                dependencias: t.dependencias.clone(),
            },
        }
    }

    /// Estado si esta viva.
    pub fn estado(&self) -> Option<&str> {
        match &self.fase {
            Fase::Viva(e) => Some(e),
            Fase::Descartada => None,
        }
    }
}

/// Orden de los hitos: la posicion de su titulo `### Hn — titulo` (D2).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Hitos {
    posicion: BTreeMap<u32, usize>,
}

/// Marca de inicio de la zona manual de `ROADMAP.md` (la de la 0.16).
const MANUAL_INICIO: &str = "<!-- manual:inicio";
/// Marca de fin de la zona manual.
const MANUAL_FIN: &str = "<!-- manual:fin";

impl Hitos {
    /// Hitos de las lineas `### Hn — titulo` de `texto`, en su orden; si un
    /// hito sale dos veces vale su primera posicion (como el oraculo).
    pub fn de_titulos(texto: &str) -> Self {
        let mut posicion = BTreeMap::new();
        for h in texto.lines().filter_map(hito_de_linea) {
            let n = posicion.len();
            posicion.entry(h).or_insert(n);
        }
        Self { posicion }
    }

    /// Hitos de la zona manual (`<!-- manual:inicio ... -->` hasta
    /// `<!-- manual:fin -->`) de un `ROADMAP.md`. Sin zona, ninguno.
    pub fn del_roadmap(texto: &str) -> Self {
        let mut dentro = false;
        let mut zona = String::new();
        for l in texto.lines() {
            let t = l.trim_start();
            if t.starts_with(MANUAL_INICIO) {
                dentro = true;
            } else if t.starts_with(MANUAL_FIN) {
                break;
            } else if dentro {
                zona.push_str(l);
                zona.push('\n');
            }
        }
        Self::de_titulos(&zona)
    }

    /// Si no hay ningun titulo.
    pub fn es_vacio(&self) -> bool {
        self.posicion.is_empty()
    }

    /// Si el hito `h` tiene titulo.
    pub fn tiene(&self, h: u32) -> bool {
        self.posicion.contains_key(&h)
    }

    /// Rango del hito `h`: su posicion si tiene titulo; si no, detras de
    /// todos por su numero; sin titulos, su numero.
    pub fn rango(&self, h: u32) -> u64 {
        match self.posicion.get(&h) {
            Some(p) => u64::try_from(*p).unwrap_or(u64::MAX),
            None if self.posicion.is_empty() => u64::from(h),
            None => 1_000_000 + u64::from(h),
        }
    }
}

/// `### H<n> — <titulo>` (raya U+2014 entre espacios, titulo no vacio).
fn hito_de_linea(l: &str) -> Option<u32> {
    let resto = l.strip_prefix("### H")?;
    let fin = resto
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(resto.len());
    let (cifras, titulo) = resto.split_at(fin);
    let titulo = titulo.strip_prefix(" \u{2014} ")?;
    if cifras.is_empty() || titulo.trim_end_matches('\r').is_empty() {
        return None;
    }
    cifras.parse().ok()
}

/// Avisos del calculo: no frenan, pero se dicen.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Avisos {
    /// `TASK-A>TASK-B`: A depende de B, que no existe (no bloquea).
    pub dependencias_inexistentes: Vec<String>,
    /// `TASK-A>TASK-B`: A depende de B, descartada (bloquea, D3).
    pub depende_de_descartada: Vec<String>,
    /// Hitos con tareas en el orden y sin titulo `### Hn — titulo`.
    pub hitos_sin_titulo: Vec<u32>,
}

/// Resultado del orden de trabajo.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Orden {
    /// Tareas no descartadas en el orden en que se hacen.
    pub orden: Vec<String>,
    /// Las que un ciclo deja sin orden (las del ciclo y las que dependen de
    /// ellas), por prioridad.
    pub atascadas: Vec<String>,
    /// Avisos.
    pub avisos: Avisos,
}

/// Clave de prioridad: (rango del hito, numero, sufijo, ID).
type Clave = (u64, u64, String, String);

/// Numero y sufijo de `TASK-NNNa`; sin cifras, al final.
fn numero_y_sufijo(id: &str) -> (u64, String) {
    let resto = id.strip_prefix("TASK-").unwrap_or(id);
    let fin = resto
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(resto.len());
    let (cifras, sufijo) = resto.split_at(fin);
    (cifras.parse().unwrap_or(u64::MAX), sufijo.to_owned())
}

/// Indice por ID (si un ID se repite vale el primero).
fn indice(nodos: &[Nodo]) -> BTreeMap<&str, &Nodo> {
    let mut idx = BTreeMap::new();
    for n in nodos {
        idx.entry(n.id.as_str()).or_insert(n);
    }
    idx
}

/// Orden de trabajo de `nodos` con el orden de `hitos` (Kahn con montículo,
/// O((n+e) log n)).
pub fn orden_de_trabajo(nodos: &[Nodo], hitos: &Hitos) -> Orden {
    let idx = indice(nodos);
    let clave = |n: &Nodo| -> Clave {
        let (num, suf) = numero_y_sufijo(&n.id);
        (hitos.rango(n.sprint), num, suf, n.id.clone())
    };
    let vivas: BTreeMap<&str, &Nodo> = idx
        .iter()
        .filter(|(_, n)| n.fase != Fase::Descartada)
        .map(|(k, n)| (*k, *n))
        .collect();
    let mut avisos = Avisos::default();
    let mut faltan: BTreeMap<&str, usize> = BTreeMap::new();
    let mut dependientes: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (id, n) in &vivas {
        let deps: BTreeSet<&str> = n.dependencias.iter().map(String::as_str).collect();
        let mut cuenta = 0;
        for d in deps {
            match idx.get(d) {
                None => avisos.dependencias_inexistentes.push(format!("{id}>{d}")),
                Some(m) if m.fase == Fase::Descartada => {
                    avisos.depende_de_descartada.push(format!("{id}>{d}"));
                }
                Some(_) => {
                    cuenta += 1;
                    dependientes.entry(d).or_default().push(id);
                }
            }
        }
        faltan.insert(id, cuenta);
    }
    let mut monticulo: BinaryHeap<Reverse<Clave>> = vivas
        .values()
        .filter(|n| faltan.get(n.id.as_str()) == Some(&0))
        .map(|n| Reverse(clave(n)))
        .collect();
    let mut orden = Vec::with_capacity(vivas.len());
    while let Some(Reverse((_, _, _, id))) = monticulo.pop() {
        if let Some(us) = dependientes.get(id.as_str()) {
            for u in us {
                if let Some(c) = faltan.get_mut(u) {
                    *c = c.saturating_sub(1);
                    if *c == 0
                        && let Some(n) = vivas.get(u)
                    {
                        monticulo.push(Reverse(clave(n)));
                    }
                }
            }
        }
        orden.push(id);
    }
    let puestas: BTreeSet<&str> = orden.iter().map(String::as_str).collect();
    let mut atascadas: Vec<&Nodo> = vivas
        .values()
        .filter(|n| !puestas.contains(n.id.as_str()))
        .copied()
        .collect();
    atascadas.sort_by_key(|n| clave(n));
    let sin_titulo: BTreeSet<u32> = orden
        .iter()
        .filter_map(|id| idx.get(id.as_str()))
        .map(|n| n.sprint)
        .filter(|h| !hitos.tiene(*h))
        .collect();
    avisos.hitos_sin_titulo = sin_titulo.into_iter().collect();
    Orden {
        orden,
        atascadas: atascadas.into_iter().map(|n| n.id.clone()).collect(),
        avisos,
    }
}

/// Las listas para empezar, en el orden de trabajo: `planificada` y con
/// todas sus dependencias existentes terminadas (una descartada bloquea).
pub fn listas(o: &Orden, nodos: &[Nodo]) -> Vec<String> {
    let idx = indice(nodos);
    o.orden
        .iter()
        .filter(|id| {
            idx.get(id.as_str()).is_some_and(|n| {
                n.estado() == Some(PLANIFICADA)
                    && n.dependencias.iter().all(|d| match idx.get(d.as_str()) {
                        None => true,
                        Some(m) => m.estado() == Some(TERMINADA),
                    })
            })
        })
        .cloned()
        .collect()
}

/// Las que estan en diseno, en curso o en revision, en el orden de trabajo.
pub fn en_marcha(o: &Orden, nodos: &[Nodo]) -> Vec<String> {
    let idx = indice(nodos);
    o.orden
        .iter()
        .filter(|id| {
            idx.get(id.as_str())
                .and_then(|n| n.estado())
                .is_some_and(|e| EN_MARCHA.contains(&e))
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(id: &str, sprint: u32, estado: &str, deps: &[&str]) -> Nodo {
        Nodo {
            id: id.to_owned(),
            sprint,
            fase: if estado == DESCARTADA {
                Fase::Descartada
            } else {
                Fase::Viva(estado.to_owned())
            },
            dependencias: deps.iter().map(|d| (*d).to_owned()).collect(),
        }
    }

    fn ids(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_owned()).collect()
    }

    /// El caso de las autopruebas del oraculo (`probar`, «Roadmap»): la
    /// dependencia manda sobre el numero; a igualdad, hito y numero menores.
    #[test]
    fn orden_como_el_oraculo() {
        let nodos = [
            n("TASK-020", 1, "planificada", &[]),
            n("TASK-016", 0, "planificada", &["TASK-030"]),
            n("TASK-030", 0, "planificada", &[]),
            n("TASK-001", 0, "terminada", &[]),
            n("TASK-014", 0, "en-diseno", &["TASK-001"]),
            n("TASK-006", 0, "descartada", &[]),
        ];
        let o = orden_de_trabajo(&nodos, &Hitos::de_titulos("### H0 — Base\n"));
        assert_eq!(
            o.orden,
            ids(&["TASK-001", "TASK-014", "TASK-030", "TASK-016", "TASK-020"])
        );
        assert_eq!(listas(&o, &nodos), ids(&["TASK-030", "TASK-020"]));
        assert_eq!(en_marcha(&o, &nodos), ids(&["TASK-014"]));
        assert_eq!(o.avisos.hitos_sin_titulo, [1]);
    }

    /// C2: una dependencia inexistente no bloquea y se avisa.
    #[test]
    fn dependencia_inexistente_no_bloquea_y_avisa() {
        let nodos = [n("TASK-002", 0, "planificada", &["TASK-099"])];
        let o = orden_de_trabajo(&nodos, &Hitos::default());
        assert_eq!(listas(&o, &nodos), ids(&["TASK-002"]));
        assert_eq!(
            o.avisos.dependencias_inexistentes,
            ids(&["TASK-002>TASK-099"])
        );
        assert!(o.atascadas.is_empty());
    }

    /// C2: una dependencia descartada sale del orden pero bloquea (D3), y
    /// se avisa; la descartada no sale en ninguna lista.
    #[test]
    fn dependencia_descartada_bloquea_y_avisa() {
        let nodos = [
            n("TASK-001", 0, "descartada", &[]),
            n("TASK-002", 0, "planificada", &["TASK-001"]),
            n("TASK-003", 0, "planificada", &[]),
        ];
        let o = orden_de_trabajo(&nodos, &Hitos::default());
        assert_eq!(o.orden, ids(&["TASK-002", "TASK-003"]));
        assert_eq!(listas(&o, &nodos), ids(&["TASK-003"]));
        assert_eq!(o.avisos.depende_de_descartada, ids(&["TASK-002>TASK-001"]));
    }

    /// C2: un ciclo deja atascadas sus tareas y las que dependen de ellas,
    /// fuera de las listas; el resto sigue.
    #[test]
    fn ciclo_deja_atascadas() {
        let nodos = [
            n("TASK-001", 0, "planificada", &["TASK-002"]),
            n("TASK-002", 0, "planificada", &["TASK-001"]),
            n("TASK-003", 0, "planificada", &["TASK-002"]),
            n("TASK-004", 0, "planificada", &[]),
            n("TASK-005", 0, "planificada", &["TASK-005"]),
        ];
        let o = orden_de_trabajo(&nodos, &Hitos::default());
        assert_eq!(o.orden, ids(&["TASK-004"]));
        assert_eq!(
            o.atascadas,
            ids(&["TASK-001", "TASK-002", "TASK-003", "TASK-005"])
        );
        assert_eq!(listas(&o, &nodos), ids(&["TASK-004"]));
    }

    /// C2: una dependencia terminada de un hito posterior retrasa a quien
    /// depende de ella (el Kahn incluye las terminadas, como el oraculo).
    #[test]
    fn dependencia_terminada_de_un_hito_posterior_retrasa() {
        let nodos = [
            n("TASK-001", 0, "planificada", &["TASK-009"]),
            n("TASK-002", 1, "planificada", &[]),
            n("TASK-009", 2, "terminada", &[]),
        ];
        let o = orden_de_trabajo(&nodos, &Hitos::default());
        assert_eq!(o.orden, ids(&["TASK-002", "TASK-009", "TASK-001"]));
        assert_eq!(listas(&o, &nodos), ids(&["TASK-002", "TASK-001"]));
        // Por numero de hito y sin titulos, sin el retraso iria primera.
        let sola = [n("TASK-001", 0, "planificada", &[]), nodos[1].clone()];
        let o = orden_de_trabajo(&sola, &Hitos::default());
        assert_eq!(o.orden, ids(&["TASK-001", "TASK-002"]));
    }

    /// C2: los IDs con sufijo van tras su numero y antes del siguiente; los
    /// de cuatro cifras, por numero (no como texto).
    #[test]
    fn ids_con_sufijo_y_de_cuatro_cifras() {
        let nodos = [
            n("TASK-1000", 0, "planificada", &[]),
            n("TASK-013", 0, "planificada", &[]),
            n("TASK-012b", 0, "planificada", &[]),
            n("TASK-012", 0, "planificada", &[]),
            n("TASK-012a", 0, "planificada", &["TASK-012b"]),
            n("TASK-999", 0, "planificada", &[]),
        ];
        let o = orden_de_trabajo(&nodos, &Hitos::default());
        assert_eq!(
            o.orden,
            ids(&[
                "TASK-012",
                "TASK-012b",
                "TASK-012a",
                "TASK-013",
                "TASK-999",
                "TASK-1000"
            ])
        );
        // TASK-012a espera a TASK-012b, que no esta terminada.
        assert!(!listas(&o, &nodos).contains(&"TASK-012a".to_owned()));
    }

    /// C2: un hito sin titulo va detras de todos los que lo tienen, por su
    /// numero, y se avisa; el orden de los titulados es el de sus titulos.
    #[test]
    fn hito_sin_titulo_va_detras_y_avisa() {
        let hitos = Hitos::de_titulos("### H0 — Base\nx\n### H14 — Motor\n### H1 — IDE\n");
        let nodos = [
            n("TASK-001", 7, "planificada", &[]),
            n("TASK-002", 1, "planificada", &[]),
            n("TASK-003", 14, "planificada", &[]),
            n("TASK-004", 3, "planificada", &[]),
        ];
        let o = orden_de_trabajo(&nodos, &hitos);
        assert_eq!(
            o.orden,
            ids(&["TASK-003", "TASK-002", "TASK-004", "TASK-001"])
        );
        assert_eq!(o.avisos.hitos_sin_titulo, [3, 7]);
        assert_eq!([hitos.rango(0), hitos.rango(14), hitos.rango(1)], [0, 1, 2]);
        assert_eq!(hitos.rango(7), 1_000_000 + 7);
        // Sin ningun titulo, el numero.
        assert_eq!(Hitos::default().rango(7), 7);
    }

    #[test]
    fn hitos_de_la_zona_manual_del_roadmap() {
        let r = "# Roadmap\n### H9 — Fuera\n<!-- manual:inicio (se edita a mano) -->\n## Hitos\n\
                 ### H2 — Dos\n### H1 — Uno\n### H3 —\n### Hx — Mal\n### H2 — Otra vez\n\
                 <!-- manual:fin -->\n### H8 — Fuera\n";
        let h = Hitos::del_roadmap(r);
        assert_eq!([h.rango(2), h.rango(1)], [0, 1]);
        assert!(!h.tiene(9) && !h.tiene(8) && !h.tiene(3));
        assert!(Hitos::del_roadmap("### H1 — Sin zona\n").es_vacio());
    }

    #[test]
    fn nodo_de_los_dos_formatos() {
        let v1 = "---\nid: TASK-005\ntitulo: \"t\"\ntipo: feature\nsprint: 2\netiquetas: [x, descartada]\n\
                  complejidad: null\nmodelo_sugerido: sonnet\nestado: planificada\nplan_aprobado: false\n\
                  rama: feature/task-005-t\nasignado_a: null\nagente_revisor: general-purpose\n\
                  skills_recomendados: []\nregla_seleccion_skill: null\nultimo_commit_revisado: null\n\
                  revision_codex: false\ntokens_diseno: null\ntokens_implementacion: null\n\
                  tokens_revision: null\ncreado: 2026-01-01\nactualizado: 2026-01-01\n\
                  dependencias: [TASK-001]\n---\n";
        let d = crate::tarea::leer(v1);
        assert!(d.is_ok(), "{d:?}");
        let Ok(d) = d else { return };
        let nodo = Nodo::de(&d);
        assert_eq!(
            (nodo.id.as_str(), nodo.sprint, &nodo.fase),
            ("TASK-005", 2, &Fase::Descartada)
        );
        assert_eq!(nodo.dependencias, ids(&["TASK-001"]));
        let mut t = crate::tarea::Tarea::nueva("TASK-006", "t", "2026-01-01");
        t.estado = DESCARTADA.to_owned();
        let d2 = Documento::Formato2(t.clone(), crate::tarea::Cuerpo::leer(""));
        assert_eq!(Nodo::de(&d2).fase, Fase::Descartada);
        t.estado = "en-curso".to_owned();
        let d2 = Documento::Formato2(t, crate::tarea::Cuerpo::leer(""));
        assert_eq!(Nodo::de(&d2).estado(), Some("en-curso"));
    }

    /// Instantanea de ToolsaWeb (SHA en la cabecera de cada fichero; D1):
    /// `tareas.tsv` y `hitos.txt` salen de `cargar` y `titulos_de_hitos` de
    /// `scripts/grafo-tareas.py`, y `esperado.txt` son las lineas «En
    /// marcha» y «Listas para empezar» del `ROADMAP.md` de ese commit, en el
    /// que `grafo-tareas.py --comprobar` sale 0.
    const TW_TAREAS: &str = include_str!("../tests/fixtures/toolsaweb-1fc94425/tareas.tsv");
    const TW_HITOS: &str = include_str!("../tests/fixtures/toolsaweb-1fc94425/hitos.txt");
    const TW_ESPERADO: &str = include_str!("../tests/fixtures/toolsaweb-1fc94425/esperado.txt");

    /// `id\tsprint\testado\tdescartada(si|no)\tdependencias(,)`.
    fn nodos_tsv(tsv: &str) -> Vec<Nodo> {
        tsv.lines()
            .map(|l| l.trim_end_matches('\r'))
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(|l| {
                let c: Vec<&str> = l.split('\t').collect();
                let campo = |i: usize| c.get(i).copied().unwrap_or_default();
                Nodo {
                    id: campo(0).to_owned(),
                    sprint: campo(1).parse().unwrap_or(u32::MAX),
                    fase: if campo(3) == "si" {
                        Fase::Descartada
                    } else {
                        Fase::Viva(campo(2).to_owned())
                    },
                    dependencias: campo(4)
                        .split(',')
                        .filter(|d| !d.is_empty())
                        .map(str::to_owned)
                        .collect(),
                }
            })
            .collect()
    }

    /// IDs de la linea `- **<titulo>:** TASK-1 (x), TASK-2` del ROADMAP.
    fn linea_esperada(titulo: &str) -> Vec<String> {
        let prefijo = format!("- **{titulo}:** ");
        let resto = TW_ESPERADO
            .lines()
            .find_map(|l| l.trim_end_matches('\r').strip_prefix(prefijo.as_str()))
            .unwrap_or_default();
        if resto == "ninguna" {
            return Vec::new();
        }
        resto
            .split(", ")
            .filter_map(|p| p.split(' ').next())
            .map(str::to_owned)
            .collect()
    }

    /// C1: el nucleo da exactamente las listas del ROADMAP de ToolsaWeb,
    /// en el mismo orden, y las en marcha.
    #[test]
    fn toolsaweb_listas_y_en_marcha_como_su_roadmap() {
        assert!(TW_TAREAS.starts_with("# ToolsaWeb 1fc94425964a5d33b5e2c3c4c39691dbd580c641"));
        let nodos = nodos_tsv(TW_TAREAS);
        assert_eq!(nodos.len(), 404);
        let hitos = Hitos::de_titulos(TW_HITOS);
        let o = orden_de_trabajo(&nodos, &hitos);
        let esperadas = linea_esperada("Listas para empezar");
        assert_eq!(esperadas.len(), 54);
        assert_eq!(listas(&o, &nodos), esperadas);
        assert_eq!(en_marcha(&o, &nodos), linea_esperada("En marcha"));
        assert!(o.atascadas.is_empty());
        assert!(o.avisos.hitos_sin_titulo.is_empty());
        assert!(o.avisos.dependencias_inexistentes.is_empty());
        // Ordenar los hitos por su numero no lo da (H14 va tras H0).
        let por_numero = orden_de_trabajo(&nodos, &Hitos::default());
        assert_ne!(listas(&por_numero, &nodos), esperadas);
    }

    /// Los `tarea.md` reales de ToolsaWeb en el SHA de la fixture pasan por
    /// el lector estricto de formato 1 y se proyectan igual que la fixture.
    /// Necesita un clon en ese SHA: `TC2_TOOLSAWEB=<clon> cargo test -p tc2
    /// --lib toolsaweb_reales -- --ignored`.
    #[test]
    #[ignore = "necesita un clon de ToolsaWeb en TC2_TOOLSAWEB"]
    fn toolsaweb_reales_se_leen_y_proyectan_como_la_fixture() {
        let raiz = std::env::var_os("TC2_TOOLSAWEB");
        assert!(raiz.is_some(), "falta TC2_TOOLSAWEB");
        let Some(raiz) = raiz else { return };
        let tareas = std::path::Path::new(&raiz).join("tareas");
        let mut leidos = Vec::new();
        let mut fallos = Vec::new();
        for fase in std::fs::read_dir(&tareas).into_iter().flatten().flatten() {
            for t in std::fs::read_dir(fase.path())
                .into_iter()
                .flatten()
                .flatten()
            {
                let md = t.path().join("tarea.md");
                let Ok(texto) = std::fs::read_to_string(&md) else {
                    continue;
                };
                match crate::tarea::leer(&texto.replace("\r\n", "\n")) {
                    Ok(d) => leidos.push(Nodo::de(&d)),
                    Err(e) => fallos.push(format!("{}: {e}", md.display())),
                }
            }
        }
        assert!(fallos.is_empty(), "{}", fallos.join("\n"));
        leidos.sort_by(|a, b| a.id.cmp(&b.id));
        assert_eq!(leidos, nodos_tsv(TW_TAREAS));
    }
}
