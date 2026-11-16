//! Texto de los titulos (F2-4b, RR-12): plegado, slug de la rama y parecido
//! entre titulos. Hoja pura, sin crates: un solo plegado sirve al slug y a la
//! comparacion.
//!
//! - [`plegar`]: minusculas, sin tildes (tabla fija de Latin-1 y Latin
//!   Extended-A), lo no alfanumerico a espacio y espacios colapsados. Otros
//!   alfabetos quedan en minuscula: se comparan, pero no entran en el slug.
//! - [`slug`]: ASCII `[a-z0-9-]`, como mucho [`SLUG_MAX`] bytes cortando en
//!   guion; vacio → `tarea`.
//! - [`parecido`]: plegados iguales → duplicado; si no, coeficiente de Dice
//!   sobre los bigramas de caracteres del plegado sin espacios, en
//!   centesimas; desde [`UMBRAL`] cuenta como parecido.

/// Centesimas de Dice desde las que dos titulos son «casi iguales» (D2 de
/// TASK-076: constante hasta que F9-4 mida el ruido).
pub const UMBRAL: u32 = 80;

/// Longitud maxima del slug, en bytes.
pub const SLUG_MAX: usize = 40;

/// Slug de un titulo sin nada que valga en ASCII.
pub const SLUG_VACIO: &str = "tarea";

/// Como se parecen dos titulos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parecido {
    /// Plegados iguales.
    Duplicado,
    /// Dice >= [`UMBRAL`], en centesimas (truncadas: 0,866 → 86).
    Casi(u32),
}

/// Equivalente sin tilde de una letra minuscula de Latin-1 o Latin
/// Extended-A; `None` si no es una de ellas.
fn sin_tilde(c: char) -> Option<&'static str> {
    Some(match c {
        'ª' | 'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => "a",
        'æ' => "ae",
        'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' => "c",
        'ð' | 'ď' | 'đ' => "d",
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' => "e",
        'ĝ' | 'ğ' | 'ġ' | 'ģ' => "g",
        'ĥ' | 'ħ' => "h",
        'ì' | 'í' | 'î' | 'ï' | 'ĩ' | 'ī' | 'ĭ' | 'į' | 'ı' => "i",
        'ĳ' => "ij",
        'ĵ' => "j",
        'ķ' | 'ĸ' => "k",
        'ĺ' | 'ļ' | 'ľ' | 'ŀ' | 'ł' => "l",
        'ñ' | 'ń' | 'ņ' | 'ň' | 'ŉ' | 'ŋ' => "n",
        'º' | 'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ŏ' | 'ő' => "o",
        'œ' => "oe",
        'ŕ' | 'ŗ' | 'ř' => "r",
        'ś' | 'ŝ' | 'ş' | 'š' | 'ſ' => "s",
        'ß' => "ss",
        'ţ' | 'ť' | 'ŧ' => "t",
        'þ' => "th",
        'ù' | 'ú' | 'û' | 'ü' | 'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' => "u",
        'ŵ' => "w",
        'ý' | 'ÿ' | 'ŷ' => "y",
        'ź' | 'ż' | 'ž' => "z",
        _ => return None,
    })
}

/// Marcas combinantes (U+0300..U+036F): una tilde escrita aparte (`e` +
/// U+0301) o la que deja `İ` al pasar a minuscula se quitan.
fn combinante(c: char) -> bool {
    ('\u{300}'..='\u{36f}').contains(&c)
}

/// Titulo plegado: minusculas, sin tildes, lo no alfanumerico a un espacio,
/// sin espacios al principio ni al final ni repetidos.
pub fn plegar(titulo: &str) -> String {
    let mut s = String::with_capacity(titulo.len());
    let mut espacio = false;
    for c in titulo.chars().flat_map(char::to_lowercase) {
        if combinante(c) {
            continue;
        }
        if let Some(t) = sin_tilde(c) {
            if espacio && !s.is_empty() {
                s.push(' ');
            }
            espacio = false;
            s.push_str(t);
        } else if c.is_alphanumeric() {
            if espacio && !s.is_empty() {
                s.push(' ');
            }
            espacio = false;
            s.push(c);
        } else {
            espacio = true;
        }
    }
    s
}

/// Slug de la rama (ADR F1-3): el plegado con solo `[a-z0-9]` y guiones
/// entre medias, como mucho [`SLUG_MAX`] bytes cortando en el ultimo guion
/// que quepa; si no queda nada, [`SLUG_VACIO`].
pub fn slug(titulo: &str) -> String {
    let mut s = String::new();
    let mut guion = false;
    for c in plegar(titulo).chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            if guion && !s.is_empty() {
                s.push('-');
            }
            guion = false;
            s.push(c);
        } else {
            guion = true;
        }
    }
    if s.len() > SLUG_MAX {
        // Solo hay ASCII: cortar por bytes no parte ningun caracter.
        let corte = match s.get(..=SLUG_MAX).and_then(|p| p.rfind('-')) {
            Some(i) if i > 0 => i,
            _ => SLUG_MAX,
        };
        s.truncate(corte);
        while s.ends_with('-') {
            s.pop();
        }
    }
    if s.is_empty() {
        SLUG_VACIO.to_owned()
    } else {
        s
    }
}

/// Bigramas de caracteres de `plegado` sin espacios, ordenados (multiconjunto).
fn bigramas(plegado: &str) -> Vec<(char, char)> {
    let c: Vec<char> = plegado.chars().filter(|c| *c != ' ').collect();
    let mut v: Vec<(char, char)> = c.windows(2).map(|w| (w[0], w[1])).collect();
    v.sort_unstable();
    v
}

/// Coeficiente de Dice entre dos plegados, en centesimas truncadas:
/// `2·|A∩B| / (|A|+|B|)` sobre los multiconjuntos de bigramas. Truncar hace
/// que `centesimas >= UMBRAL` sea exactamente `Dice >= 0,80`.
fn dice(a: &str, b: &str) -> u32 {
    let (x, y) = (bigramas(a), bigramas(b));
    let total = x.len() + y.len();
    if total == 0 {
        return 0;
    }
    let (mut i, mut j, mut comunes) = (0, 0, 0usize);
    while let (Some(p), Some(q)) = (x.get(i), y.get(j)) {
        match p.cmp(q) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                comunes += 1;
                i += 1;
                j += 1;
            }
        }
    }
    u32::try_from(200 * comunes / total).unwrap_or(0)
}

/// Si dos titulos son el mismo o casi: `None` si no se parecen.
pub fn parecido(a: &str, b: &str) -> Option<Parecido> {
    let (pa, pb) = (plegar(a), plegar(b));
    if pa == pb {
        return Some(Parecido::Duplicado);
    }
    let d = dice(&pa, &pb);
    (d >= UMBRAL).then_some(Parecido::Casi(d))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plegado_sin_tildes_ni_signos() {
        for (t, p) in [
            ("  Exportar   INFORMES!! ", "exportar informes"),
            ("Añadir «búsqueda» — rápida", "anadir busqueda rapida"),
            ("Ça, Æsir; Œuvre; Straße", "ca aesir oeuvre strasse"),
            ("Łódź Đakovo İstanbul", "lodz dakovo istanbul"),
            ("cafe\u{301} con e\u{301}", "cafe con e"),
            ("Задача №5", "задача 5"),
            ("漢字", "漢字"),
            ("---", ""),
        ] {
            assert_eq!(plegar(t), p, "{t}");
        }
    }

    /// C7: el slug es ASCII `[a-z0-9-]`, corta en guion y un titulo sin
    /// letras latinas da `tarea`.
    #[test]
    fn slug_ascii_cortado_en_guion_y_vacio_es_tarea() {
        for (t, s) in [
            (
                "Nombres únicos de tarea y rama",
                "nombres-unicos-de-tarea-y-rama",
            ),
            ("C:\\ruta\\ \"x\" {fecha}", "c-ruta-x-fecha"),
            ("Задача", "tarea"),
            ("漢字 — !!!", "tarea"),
            ("", "tarea"),
            ("Café 日本 v2", "cafe-v2"),
            (
                "uno dos tres cuatro cinco seis siete ocho nueve diez",
                "uno-dos-tres-cuatro-cinco-seis-siete",
            ),
            (&"a".repeat(50), &"a".repeat(40)),
            (
                "abcdefghijklmnopqrstuvwxyzabcdefghijklmn-z",
                "abcdefghijklmnopqrstuvwxyzabcdefghijklmn",
            ),
        ] {
            let r = slug(t);
            assert_eq!(r, s, "{t}");
            assert!(r.len() <= SLUG_MAX);
            assert!(!r.starts_with('-') && !r.ends_with('-'), "{r}");
            assert!(
                r.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'),
                "{r}"
            );
        }
    }

    /// C5: tabla del umbral de «casi igual» (Dice >= 0,80 sobre bigramas).
    #[test]
    fn umbral_de_casi_igual() {
        for (a, b, esperado) in [
            (
                "Exportar informes",
                "Exprotar informes",
                Some(Parecido::Casi(80)),
            ),
            ("Login con Google", "Login con GitHub", None),
            (
                "Exportar informes",
                "EXPORTAR  informes.",
                Some(Parecido::Duplicado),
            ),
            (
                "Añadir búsqueda",
                "anadir busqueda",
                Some(Parecido::Duplicado),
            ),
            (
                "Exportar informes",
                "Exportar informes PDF",
                Some(Parecido::Casi(90)),
            ),
            ("Exportar informes", "Importar facturas", None),
            ("a", "b", None),
            ("", "", Some(Parecido::Duplicado)),
        ] {
            assert_eq!(parecido(a, b), esperado, "{a} / {b}");
            assert_eq!(parecido(b, a), esperado, "simetrico: {a} / {b}");
        }
        assert_eq!(dice("exportar informes", "exprotar informes"), 80);
        assert_eq!(dice("login con google", "login con github"), 61);
    }
}
