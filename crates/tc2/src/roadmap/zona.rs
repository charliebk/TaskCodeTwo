//! Zona manual de `ROADMAP.md` (`<!-- manual:inicio ... -->` hasta
//! `<!-- manual:fin -->`, la de la 0.16; ADR F1-1, decision 3).
//!
//! [`extraer`] es estricta: una marca de inicio sin fin, una de fin suelta o
//! una segunda zona son un ROADMAP roto, y se dice en vez de adivinar donde
//! acaba (adivinar mal perderia texto de la persona). La zona se devuelve
//! como un trozo del texto, byte a byte, con sus saltos de linea (LF o CRLF).
//! [`interior_tolerante`] es la lectura que usa el orden de los hitos: no
//! falla, para que `next` siga dando su orden con un ROADMAP roto.
//!
//! Hoja pura.

/// Marca de inicio (la linea empieza asi, tras espacios).
pub const INICIO: &str = "<!-- manual:inicio";
/// Marca de fin.
pub const FIN: &str = "<!-- manual:fin";

/// Zona de un ROADMAP recien creado: la de la 0.16, con `tc2` en lugar de
/// `taskctl`.
pub const POR_DEFECTO: &str = "<!-- manual:inicio (se edita a mano; tc2 la conserva) -->\n\
## Hitos\n\
\n\
Un `### Hn — título` por hito (n es el campo `sprint` de sus tareas), en el orden en que se hacen y con su motivo debajo. tc2 conserva esta zona tal cual y toma de aquí los títulos y el orden de los hitos.\n\
<!-- manual:fin -->\n";

/// Por que la zona manual esta rota.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rota {
    /// Abre la zona y no la cierra.
    SinFin,
    /// Cierra una zona que no se abrio.
    FinSuelto,
    /// Hay mas de una zona (o un inicio dentro de otra).
    Repetida,
}

impl Rota {
    /// Valor de `roadmap-roto=` y de `motivo=`.
    pub fn motivo(self) -> &'static str {
        match self {
            Self::SinFin => "sin-fin",
            Self::FinSuelto => "fin-suelto",
            Self::Repetida => "repetida",
        }
    }
}

fn es_marca(linea: &str, marca: &str) -> bool {
    linea.trim_start().starts_with(marca)
}

/// La zona manual de `texto`, desde el principio de la linea de inicio hasta
/// el final de la de fin (con su salto de linea, si lo tiene), tal cual.
/// Sin zona, `None`.
pub fn extraer(texto: &str) -> Result<Option<&str>, Rota> {
    let mut inicio: Option<usize> = None;
    let mut fin: Option<usize> = None;
    let mut pos = 0;
    for linea in texto.split_inclusive('\n') {
        if es_marca(linea, INICIO) {
            if inicio.is_some() {
                return Err(Rota::Repetida);
            }
            inicio = Some(pos);
        } else if es_marca(linea, FIN) {
            match (inicio, fin) {
                (None, _) => return Err(Rota::FinSuelto),
                (Some(_), Some(_)) => return Err(Rota::Repetida),
                (Some(_), None) => fin = Some(pos + linea.len()),
            }
        }
        pos += linea.len();
    }
    match (inicio, fin) {
        (None, _) => Ok(None),
        (Some(_), None) => Err(Rota::SinFin),
        (Some(i), Some(f)) => Ok(texto.get(i..f)),
    }
}

/// Las lineas de dentro de la zona (sin las marcas), con LF. Tolerante: un
/// inicio sin fin llega hasta el final y una segunda marca de inicio se
/// salta; sin inicio, nada.
pub fn interior_tolerante(texto: &str) -> String {
    let mut dentro = false;
    let mut zona = String::new();
    for l in texto.lines() {
        if es_marca(l, INICIO) {
            dentro = true;
        } else if es_marca(l, FIN) {
            break;
        } else if dentro {
            zona.push_str(l);
            zona.push('\n');
        }
    }
    zona
}

#[cfg(test)]
mod tests {
    use super::*;

    /// C2: la zona sale byte a byte, con LF y con CRLF, y con texto delante
    /// y detras.
    #[test]
    fn extrae_byte_a_byte_con_lf_y_crlf() {
        for salto in ["\n", "\r\n"] {
            let zona = format!(
                "<!-- manual:inicio (a mano) -->{salto}## Hitos{salto}{salto}### H1 — Uno  {salto}\t texto\u{a0}raro{salto}<!-- manual:fin -->{salto}"
            );
            let texto = format!("# Roadmap{salto}{salto}{zona}{salto}## Generado{salto}");
            assert_eq!(extraer(&texto), Ok(Some(zona.as_str())));
        }
        // Sin salto tras la marca de fin.
        let t = "x\n<!-- manual:inicio -->\ny\n<!-- manual:fin -->";
        assert_eq!(
            extraer(t),
            Ok(Some("<!-- manual:inicio -->\ny\n<!-- manual:fin -->"))
        );
        assert_eq!(extraer("# Sin zona\n"), Ok(None));
        assert_eq!(extraer(""), Ok(None));
    }

    /// D3: una zona rota se dice, no se adivina.
    #[test]
    fn zona_rota() {
        assert_eq!(extraer("<!-- manual:inicio -->\nx\n"), Err(Rota::SinFin));
        assert_eq!(extraer("x\n<!-- manual:fin -->\n"), Err(Rota::FinSuelto));
        assert_eq!(
            extraer("<!-- manual:inicio -->\n<!-- manual:inicio -->\n<!-- manual:fin -->\n"),
            Err(Rota::Repetida)
        );
        assert_eq!(
            extraer("<!-- manual:inicio -->\n<!-- manual:fin -->\n<!-- manual:fin -->\n"),
            Err(Rota::Repetida)
        );
        assert_eq!(
            extraer(
                "<!-- manual:inicio -->\n<!-- manual:fin -->\n<!-- manual:inicio -->\n<!-- manual:fin -->\n"
            ),
            Err(Rota::Repetida)
        );
        assert_eq!(Rota::SinFin.motivo(), "sin-fin");
    }

    #[test]
    fn la_zona_por_defecto_es_valida() {
        assert_eq!(extraer(POR_DEFECTO), Ok(Some(POR_DEFECTO)));
    }

    #[test]
    fn interior_tolerante_como_antes() {
        let r = "### H9 — Fuera\n<!-- manual:inicio x -->\n### H2 — Dos\r\n<!-- manual:fin -->\n### H8\n";
        assert_eq!(interior_tolerante(r), "### H2 — Dos\n");
        assert_eq!(
            interior_tolerante("<!-- manual:inicio -->\na\n"),
            "a\n",
            "sin fin llega al final"
        );
        assert_eq!(interior_tolerante("a\n<!-- manual:fin -->\n"), "");
    }
}
