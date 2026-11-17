//! Bloque Mermaid de un hito (ADR F1-1, decision 3: un grafo por hito no
//! cerrado). Acotado a lo que GitHub y GitLab pintan: menos de
//! [`MAX_ARISTAS`] aristas y de [`MAX_CARACTERES`] caracteres (C6 de
//! TASK-031); si no cabe, una nota en su lugar.
//!
//! Hoja pura.

use std::fmt::Write as _;

/// Aristas por bloque por debajo de las que se dibuja (`maxEdges` de Mermaid
/// es 500).
pub const MAX_ARISTAS: usize = 500;
/// Caracteres por bloque por debajo de los que se dibuja (`maxTextSize` de
/// Mermaid es 50 000).
pub const MAX_CARACTERES: usize = 50_000;

/// Clases de los nodos, por estado; en cada bloque, solo las que usa.
const CLASES: [(&str, &str); 7] = [
    ("lista", "fill:#d4f7d4,stroke:#2e7d32"),
    ("marcha", "fill:#d6e9ff,stroke:#1565c0"),
    ("espera", "fill:#f3f3f3,stroke:#888888"),
    ("atascada", "fill:#ffd6d6,stroke:#c62828"),
    ("terminada", "fill:#e8e8e8,stroke:#9e9e9e,color:#555555"),
    (
        "descartada",
        "fill:#ffffff,stroke:#bbbbbb,stroke-dasharray:4 3,color:#888888",
    ),
    ("borde", "fill:#ffffff,stroke:#999999,stroke-dasharray:2 2"),
];

/// Un nodo del grafo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nodo {
    /// `TASK-NNN`.
    pub id: String,
    /// Titulo (se escapa aqui); vacio, la etiqueta es solo el ID.
    pub titulo: String,
    /// Clase: `lista`, `marcha`, `espera`, `atascada`, `terminada`,
    /// `descartada` o `borde`.
    pub clase: &'static str,
}

/// Identificador Mermaid de `TASK-NNN[a]`: `TNNN[a]`; cualquier otro
/// caracter, `_`.
pub fn id_de(id: &str) -> String {
    let (prefijo, resto) = match id.strip_prefix("TASK-") {
        Some(r) => ("T", r),
        None => ("N_", id),
    };
    let limpio: String = resto
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    format!("{prefijo}{limpio}")
}

/// Texto de una etiqueta entre comillas, con los caracteres que Mermaid
/// interpreta como codigos de entidad.
pub fn escapar(texto: &str) -> String {
    let mut s = String::with_capacity(texto.len());
    for c in texto.chars() {
        match c {
            '#' => s.push_str("#35;"),
            '"' => s.push_str("#quot;"),
            '<' => s.push_str("#lt;"),
            '>' => s.push_str("#gt;"),
            '`' => s.push_str("#96;"),
            c if c.is_control() => s.push(' '),
            c => s.push(c),
        }
    }
    s
}

/// El bloque ```` ```mermaid ```` con `nodos` y `aristas` (`(de, a)`: `a`
/// depende de `de`), o una nota si pasa de los limites.
pub fn bloque(nodos: &[Nodo], aristas: &[(String, String)]) -> String {
    let mut s = String::from("```mermaid\nflowchart TD\n");
    for n in nodos {
        let _ = if n.titulo.is_empty() {
            writeln!(s, "  {}[\"{}\"]:::{}", id_de(&n.id), n.id, n.clase)
        } else {
            writeln!(
                s,
                "  {}[\"{} · {}\"]:::{}",
                id_de(&n.id),
                n.id,
                escapar(&n.titulo),
                n.clase
            )
        };
    }
    for (de, a) in aristas {
        let _ = writeln!(s, "  {} --> {}", id_de(de), id_de(a));
    }
    for (clase, estilo) in CLASES {
        if nodos.iter().any(|n| n.clase == clase) {
            let _ = writeln!(s, "  classDef {clase} {estilo}");
        }
    }
    s.push_str("```\n");
    if aristas.len() >= MAX_ARISTAS || s.chars().count() >= MAX_CARACTERES {
        return format!(
            "_Grafo de este hito demasiado grande para dibujarlo ({} nodos, {} aristas)._\n",
            nodos.len(),
            aristas.len()
        );
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_y_etiquetas_escapadas() {
        assert_eq!(id_de("TASK-031"), "T031");
        assert_eq!(id_de("TASK-012a"), "T012a");
        assert_eq!(id_de("raro-1"), "N_raro_1");
        assert_eq!(
            escapar("a \"b\" <c> #1 `d`\te"),
            "a #quot;b#quot; #lt;c#gt; #35;1 #96;d#96; e"
        );
    }

    #[test]
    fn bloque_con_nodos_aristas_y_clases() {
        let n = |id: &str, clase| Nodo {
            id: id.to_owned(),
            titulo: "t".to_owned(),
            clase,
        };
        let b = bloque(
            &[n("TASK-001", "borde"), n("TASK-002", "lista")],
            &[("TASK-001".to_owned(), "TASK-002".to_owned())],
        );
        assert!(b.starts_with("```mermaid\nflowchart TD\n  T001[\"TASK-001 · t\"]:::borde\n"));
        assert!(b.contains("  T002[\"TASK-002 · t\"]:::lista\n  T001 --> T002\n"));
        assert!(b.ends_with(
            "  classDef lista fill:#d4f7d4,stroke:#2e7d32\n  classDef borde fill:#ffffff,stroke:#999999,stroke-dasharray:2 2\n```\n"
        ));
        let sin_titulo = Nodo {
            titulo: String::new(),
            ..n("TASK-003", "borde")
        };
        assert!(bloque(&[sin_titulo], &[]).contains("  T003[\"TASK-003\"]:::borde\n"));
    }

    /// C6: si no cabe, una nota en vez de un grafo que no se pinta.
    #[test]
    fn grafo_demasiado_grande_es_una_nota() {
        let nodos: Vec<Nodo> = (0..=MAX_ARISTAS)
            .map(|i| Nodo {
                id: format!("TASK-{i:04}"),
                titulo: "t".to_owned(),
                clase: "lista",
            })
            .collect();
        let aristas: Vec<(String, String)> = (0..MAX_ARISTAS)
            .map(|i| (format!("TASK-{i:04}"), format!("TASK-{:04}", i + 1)))
            .collect();
        let b = bloque(&nodos, &aristas);
        assert!(!b.contains("```"), "{b}");
        assert!(b.contains("500 aristas"));
        assert!(bloque(&nodos, &aristas[..MAX_ARISTAS - 1]).starts_with("```mermaid"));
    }
}
