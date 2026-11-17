//! Texto del `ROADMAP.md` 2.0 desde una [`Situacion`] (ADR F1-1, decision
//! 3): cabecera con la marca de formato, la zona manual byte a byte, el
//! estado del trabajo (las mismas listas que `next`), en curso con quien la
//! lleva, en revision, un apartado por hito no cerrado con su tabla y su
//! Mermaid, y las terminadas y descartadas plegadas.
//!
//! Hoja pura: LF y sin fecha, para que los mismos datos den los mismos bytes
//! (`roadmap --escribir` dos veces seguidas da `sin-cambios`).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use super::mermaid;
use crate::grafo::{self, Fase, PLANIFICADA, TERMINADA};
use crate::plataforma::Personas;
use crate::situacion::{self, Ficha, Quien, Situacion};
use crate::tarea::Asignado;

/// Tercera linea del fichero: lo generado y su formato.
pub const MARCA: &str = "<!-- generado por tc2; formato 2; no editar fuera de la zona manual -->";

/// Lo que se consulta al escribir.
struct Datos<'a> {
    s: &'a Situacion,
    personas: &'a Personas,
    fichas: BTreeMap<&'a str, &'a Ficha>,
    paso: BTreeMap<&'a str, usize>,
    listas: BTreeSet<&'a str>,
    atascadas: BTreeSet<&'a str>,
    ilegibles: BTreeSet<&'a str>,
}

impl<'a> Datos<'a> {
    fn nuevos(s: &'a Situacion, personas: &'a Personas) -> Self {
        let mut fichas = BTreeMap::new();
        for f in &s.fichas {
            fichas.entry(f.nodo.id.as_str()).or_insert(f);
        }
        Self {
            s,
            personas,
            fichas,
            paso: s
                .orden
                .orden
                .iter()
                .enumerate()
                .map(|(i, id)| (id.as_str(), i + 1))
                .collect(),
            listas: s.listas.iter().map(String::as_str).collect(),
            atascadas: s.orden.atascadas.iter().map(String::as_str).collect(),
            ilegibles: s.ilegibles.iter().map(String::as_str).collect(),
        }
    }

    /// Fichas vivas y no terminadas, en el orden de trabajo (las atascadas
    /// detras, por numero).
    fn abiertas(&self, filtro: impl Fn(&Ficha) -> bool) -> Vec<&'a Ficha> {
        let mut v: Vec<&Ficha> = self
            .fichas
            .values()
            .copied()
            .filter(|f| matches!(f.nodo.estado(), Some(e) if e != TERMINADA))
            .filter(|f| filtro(f))
            .collect();
        v.sort_by_key(|f| {
            (
                self.paso
                    .get(f.nodo.id.as_str())
                    .copied()
                    .unwrap_or(usize::MAX),
                grafo::numero_y_sufijo(&f.nodo.id),
            )
        });
        v
    }

    /// Texto del estado y clase Mermaid de una tarea viva.
    fn estado(&self, f: &Ficha) -> (String, &'static str) {
        let id = f.nodo.id.as_str();
        let e = f.nodo.estado().unwrap_or_default();
        if self.atascadas.contains(id) {
            return ("atascada (ciclo)".to_owned(), "atascada");
        }
        match e {
            TERMINADA => ("terminada".to_owned(), "terminada"),
            "en-diseno" => ("en diseño".to_owned(), "marcha"),
            "en-curso" => ("en curso".to_owned(), "marcha"),
            "en-revision" => ("en revisión".to_owned(), "marcha"),
            PLANIFICADA if self.listas.contains(id) => ("lista para empezar".to_owned(), "lista"),
            PLANIFICADA => {
                let faltan: Vec<String> = deps(f)
                    .into_iter()
                    .filter_map(|d| self.pendiente(d))
                    .collect();
                if faltan.is_empty() {
                    ("planificada".to_owned(), "espera")
                } else {
                    (format!("espera a {}", faltan.join(", ")), "espera")
                }
            }
            otro => (otro.to_owned(), "espera"),
        }
    }

    /// `d` si bloquea (existe y no esta terminada), con lo que es si no
    /// esta viva.
    fn pendiente(&self, d: &str) -> Option<String> {
        if self.ilegibles.contains(d) && !self.fichas.contains_key(d) {
            return Some(format!("{d} (ilegible)"));
        }
        let f = self.fichas.get(d)?;
        match &f.nodo.fase {
            Fase::Viva(e) if e == TERMINADA => None,
            Fase::Viva(_) => Some(d.to_owned()),
            Fase::Descartada => Some(format!("{d} (descartada)")),
            Fase::Ilegible => Some(format!("{d} (ilegible)")),
        }
    }

    /// El «quien» comun de [`situacion::quien`], con el heredado como celda.
    fn con_quien(&self, a: Option<&Asignado>) -> String {
        match situacion::quien(a, self.personas) {
            Quien::Heredado(h) => celda(&h),
            q => q.texto(),
        }
    }
}

/// Dependencias sin repetir, en su orden.
fn deps(f: &Ficha) -> Vec<&str> {
    let mut vistas = BTreeSet::new();
    f.nodo
        .dependencias
        .iter()
        .map(String::as_str)
        .filter(|d| vistas.insert(*d))
        .collect()
}

/// Texto de una celda de tabla o de una linea de lista: sin `|` que parta
/// la fila ni `<` que abra una etiqueta o un comentario HTML.
fn celda(t: &str) -> String {
    t.replace('|', "\\|").replace('<', "&lt;")
}

fn lista_o_ninguna(v: &[String]) -> String {
    if v.is_empty() {
        "ninguna".to_owned()
    } else {
        v.join(", ")
    }
}

fn depende_de(f: &Ficha) -> String {
    let d = deps(f);
    if d.is_empty() {
        "—".to_owned()
    } else {
        d.join(", ")
    }
}

/// El `ROADMAP.md` de `s` con la zona manual `zona` (byte a byte).
pub fn generar(s: &Situacion, zona: &str, personas: &Personas) -> String {
    let d = Datos::nuevos(s, personas);
    let mut o = String::from("# Roadmap de tareas\n\n");
    o.push_str(MARCA);
    o.push_str("\n\n");
    o.push_str(zona);
    if !zona.ends_with('\n') {
        o.push('\n');
    }
    estado_del_trabajo(&d, &mut o);
    en_marcha(&d, &mut o, "En curso", &["en-diseno", "en-curso"]);
    en_marcha(&d, &mut o, "En revisión", &["en-revision"]);
    hitos(&d, &mut o);
    plegadas(&d, &mut o, "Terminadas", |f| {
        f.nodo.estado() == Some(TERMINADA)
    });
    plegadas(&d, &mut o, "Descartadas", |f| {
        f.nodo.fase == Fase::Descartada
    });
    o
}

fn estado_del_trabajo(d: &Datos<'_>, o: &mut String) {
    let s = d.s;
    let marcha: Vec<String> = s
        .en_marcha
        .iter()
        .map(|id| match d.fichas.get(id.as_str()) {
            Some(f) => format!("{id} ({})", d.estado(f).0),
            None => id.clone(),
        })
        .collect();
    o.push_str("\n## Estado del trabajo\n\n");
    o.push_str(
        "Orden en que se hacen las tareas: ninguna va antes que aquellas de las que depende y, a igualdad, va primero la del hito anterior (en el orden de los `### Hn — título` de la zona manual) y luego la de número menor. tc2 lo regenera en cada cambio de la rama de estado; solo la zona manual se edita a mano.\n\n",
    );
    let _ = writeln!(o, "- **En marcha:** {}", lista_o_ninguna(&marcha));
    let _ = writeln!(
        o,
        "- **Listas para empezar:** {}",
        lista_o_ninguna(&s.listas)
    );
    let a = &s.orden.avisos;
    let hitos: Vec<String> = a.hitos_sin_titulo.iter().map(|h| format!("H{h}")).collect();
    for (titulo, v) in [
        ("Atascadas (ciclo)", &s.orden.atascadas),
        ("Ilegibles", &s.ilegibles),
        ("Dependencias inexistentes", &a.dependencias_inexistentes),
        ("Dependen de una descartada", &a.depende_de_descartada),
        ("Hitos sin título", &hitos),
        ("Duplicadas", &a.duplicadas),
    ] {
        if !v.is_empty() {
            let _ = writeln!(o, "- **{titulo}:** {}", v.join(", "));
        }
    }
}

fn en_marcha(d: &Datos<'_>, o: &mut String, titulo: &str, estados: &[&str]) {
    let _ = writeln!(o, "\n## {titulo}\n");
    let v = d.abiertas(|f| f.nodo.estado().is_some_and(|e| estados.contains(&e)));
    if v.is_empty() {
        o.push_str("Ninguna.\n");
        return;
    }
    o.push_str("| Tarea | Título | Con quién | Hito | Depende de |\n|---|---|---|---|---|\n");
    for f in v {
        let _ = writeln!(
            o,
            "| {} | {} | {} | H{} | {} |",
            f.nodo.id,
            celda(&f.titulo),
            d.con_quien(f.asignado.as_ref()),
            f.nodo.sprint,
            depende_de(f)
        );
    }
}

fn hitos(d: &Datos<'_>, o: &mut String) {
    let abiertos: BTreeSet<u32> = d.abiertas(|_| true).iter().map(|f| f.nodo.sprint).collect();
    let mut orden: Vec<u32> = abiertos.into_iter().collect();
    orden.sort_by_key(|h| (d.s.hitos.rango(*h), *h));
    for h in orden {
        hito(d, o, h);
    }
}

fn hito(d: &Datos<'_>, o: &mut String, h: u32) {
    match d.s.hitos.titulo(h) {
        Some(t) => {
            let _ = writeln!(o, "\n## H{h} — {t}\n");
        }
        None => {
            let _ = writeln!(o, "\n## H{h}\n");
        }
    }
    let vivas: Vec<&Ficha> = d
        .fichas
        .values()
        .copied()
        .filter(|f| f.nodo.sprint == h && f.nodo.estado().is_some())
        .collect();
    let hechas = vivas
        .iter()
        .filter(|f| f.nodo.estado() == Some(TERMINADA))
        .count();
    let _ = writeln!(o, "{hechas} de {} terminadas.\n", vivas.len());
    o.push_str(
        "| Paso | Tarea | Título | Depende de | Estado | Con quién |\n|---|---|---|---|---|---|\n",
    );
    let abiertas = d.abiertas(|f| f.nodo.sprint == h);
    for f in &abiertas {
        let paso = d
            .paso
            .get(f.nodo.id.as_str())
            .map_or_else(|| "—".to_owned(), ToString::to_string);
        let _ = writeln!(
            o,
            "| {paso} | {} | {} | {} | {} | {} |",
            f.nodo.id,
            celda(&f.titulo),
            depende_de(f),
            celda(&d.estado(f).0),
            d.con_quien(f.asignado.as_ref())
        );
    }
    o.push('\n');
    o.push_str(&grafo_del_hito(d, &abiertas));
}

/// Nodos: las abiertas del hito y, de borde, las dependencias que no lo
/// son (de otro hito, terminadas, descartadas o ilegibles).
fn grafo_del_hito(d: &Datos<'_>, abiertas: &[&Ficha]) -> String {
    let dentro: BTreeSet<&str> = abiertas.iter().map(|f| f.nodo.id.as_str()).collect();
    let mut nodos: Vec<mermaid::Nodo> = abiertas
        .iter()
        .map(|f| mermaid::Nodo {
            id: f.nodo.id.clone(),
            titulo: f.titulo.clone(),
            clase: d.estado(f).1,
        })
        .collect();
    let mut borde: BTreeMap<&str, mermaid::Nodo> = BTreeMap::new();
    let mut aristas = Vec::new();
    for f in abiertas {
        for dep in deps(f) {
            if !dentro.contains(dep) && !borde.contains_key(dep) {
                let nodo = match d.fichas.get(dep) {
                    // Sin titulo: sale en su hito o en las plegadas, y asi
                    // el fichero cabe (C6).
                    Some(g) => mermaid::Nodo {
                        id: dep.to_owned(),
                        titulo: String::new(),
                        clase: match &g.nodo.fase {
                            Fase::Descartada => "descartada",
                            Fase::Viva(e) if e == TERMINADA => "terminada",
                            _ => "borde",
                        },
                    },
                    None if d.ilegibles.contains(dep) => mermaid::Nodo {
                        id: dep.to_owned(),
                        titulo: "(ilegible)".to_owned(),
                        clase: "borde",
                    },
                    // Inexistente: ya se avisa arriba; no se dibuja.
                    None => continue,
                };
                borde.insert(dep, nodo);
            }
            aristas.push((dep.to_owned(), f.nodo.id.clone()));
        }
    }
    nodos.extend(borde.into_values());
    mermaid::bloque(&nodos, &aristas)
}

fn plegadas(d: &Datos<'_>, o: &mut String, titulo: &str, filtro: impl Fn(&Ficha) -> bool) {
    let mut v: Vec<&Ficha> = d.fichas.values().copied().filter(|f| filtro(f)).collect();
    v.sort_by_key(|f| grafo::numero_y_sufijo(&f.nodo.id));
    let _ = writeln!(o, "\n## {titulo}\n");
    if v.is_empty() {
        o.push_str("Ninguna.\n");
        return;
    }
    let _ = writeln!(
        o,
        "Total: {}.\n\n<details>\n<summary>Ver la lista</summary>\n",
        v.len()
    );
    for f in v {
        let _ = writeln!(
            o,
            "- {} — {} (H{})",
            f.nodo.id,
            celda(&f.titulo),
            f.nodo.sprint
        );
    }
    o.push_str("\n</details>\n");
}

/// Las tareas de una linea `- **<titulo>:** TASK-1 (x), TASK-2` de un
/// ROADMAP (`ninguna`: vacia). Para comparar con `next` (C4).
pub fn ids_de_la_linea(roadmap: &str, titulo: &str) -> Option<Vec<String>> {
    let prefijo = format!("- **{titulo}:** ");
    let resto = roadmap
        .lines()
        .find_map(|l| l.trim_end_matches('\r').strip_prefix(prefijo.as_str()))?;
    if resto == "ninguna" {
        return Some(Vec::new());
    }
    Some(
        resto
            .split(", ")
            .filter_map(|p| p.split(' ').next())
            .map(str::to_owned)
            .collect(),
    )
}
