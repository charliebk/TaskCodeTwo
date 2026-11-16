//! Mensajes en espanol e ingles (F2-8, RF-73).
//!
//! El idioma se decide en un solo sitio, el borde del proceso (`main.rs`,
//! con [`Idioma::de_entorno`]). El resto del codigo no escribe textos: arma
//! un [`Mensaje`] neutro (un [`Texto`] del catalogo mas sus argumentos) y el
//! texto se genera al escribir la salida, en el idioma que toque.
//!
//! Lo que no cambia con el idioma (ADR F1-4): la linea de stdout, las claves
//! del JSON, los valores de `motivo` y los codigos de salida. Solo se traduce
//! el diagnostico.
//!
//! El catalogo ([`catalogo`]) se escribe con la macro `textos!`, que exige
//! las dos traducciones de cada entrada: si falta una, no compila. El test
//! `catalogo_completo` comprueba ademas que no estan vacias y que llevan los
//! mismos huecos `{nombre}`.

pub mod catalogo;
#[cfg(test)]
mod guarda;

use std::fmt::Write as _;

/// Idioma de los mensajes dirigidos a la persona.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Idioma {
    /// Espanol: el de por defecto (RF-73).
    #[default]
    Es,
    /// Ingles.
    En,
}

/// Variable que fija el idioma de `tc2` por encima del locale del sistema.
pub const VARIABLE: &str = "TC2_IDIOMA";

impl Idioma {
    /// Resuelve el idioma desde el entorno: `TC2_IDIOMA` (`es` o `en`), luego
    /// `LC_ALL`, `LC_MESSAGES` y `LANG` (por su prefijo `es` o `en`; `C`,
    /// `POSIX` y lo que no se reconoce se saltan) y, si no hay nada, espanol.
    ///
    /// El entorno llega como funcion de lectura para poder probarlo sin
    /// tocar las variables del proceso (`set_var` exige `unsafe` en edition
    /// 2024 y el workspace lo prohibe).
    pub fn de_entorno(leer: impl Fn(&str) -> Option<String>) -> Self {
        [VARIABLE, "LC_ALL", "LC_MESSAGES", "LANG"]
            .iter()
            .find_map(|v| leer(v).and_then(|valor| Self::de_valor(&valor)))
            .unwrap_or_default()
    }

    /// Idioma de un valor tipo locale (`en`, `en_US.UTF-8`, `es-419`,
    /// `EN`...). `None` si no es espanol ni ingles.
    fn de_valor(valor: &str) -> Option<Self> {
        let base = valor
            .trim()
            .split(['_', '-', '.', '@'])
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        match base.as_str() {
            "es" => Some(Self::Es),
            "en" => Some(Self::En),
            _ => None,
        }
    }
}

/// Una entrada del catalogo: el mismo mensaje en los dos idiomas. Los huecos
/// se escriben `{nombre}` y se rellenan con [`Mensaje::con`].
#[derive(Debug, PartialEq, Eq)]
pub struct Texto {
    /// Identificador unico: `modulo::NOMBRE`.
    pub clave: &'static str,
    /// Texto en espanol.
    pub es: &'static str,
    /// Texto en ingles.
    pub en: &'static str,
}

impl Texto {
    /// La plantilla en un idioma.
    pub fn en_idioma(&self, idioma: Idioma) -> &'static str {
        match idioma {
            Idioma::Es => self.es,
            Idioma::En => self.en,
        }
    }
}

/// Valor de un hueco: texto tal cual o otro mensaje, que se traduce en el
/// mismo idioma que el que lo contiene.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Arg {
    Texto(String),
    Mensaje(Mensaje),
}

/// Mensaje neutro: una entrada del catalogo mas sus argumentos. El texto se
/// genera con [`Mensaje::texto`] en el idioma que se pida.
///
/// No hay `From<String>`: un texto solo entra como argumento de un hueco o,
/// si viene de fuera (stderr de git, error del sistema, HTTP) y no hay nada
/// que traducir alrededor, con [`Mensaje::externo`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mensaje {
    texto: &'static Texto,
    args: Vec<(&'static str, Arg)>,
}

impl Mensaje {
    /// Mensaje sin argumentos todavia.
    pub fn nuevo(texto: &'static Texto) -> Self {
        Self {
            texto,
            args: Vec::new(),
        }
    }

    /// Texto que viene de fuera de `tc2` y no se traduce: el mensaje entero
    /// es ese texto.
    pub fn externo(texto: impl Into<String>) -> Self {
        Self::nuevo(&catalogo::comun::EXTERNO).con("texto", texto)
    }

    /// Rellena el hueco `{nombre}` con un valor tal cual (una ruta, un hash,
    /// un mensaje de git...).
    #[must_use]
    pub fn con(mut self, nombre: &'static str, valor: impl Into<String>) -> Self {
        self.args.push((nombre, Arg::Texto(valor.into())));
        self
    }

    /// Rellena el hueco `{nombre}` con otro mensaje, traducido en el mismo
    /// idioma.
    #[must_use]
    pub fn con_mensaje(mut self, nombre: &'static str, valor: impl Into<Mensaje>) -> Self {
        self.args.push((nombre, Arg::Mensaje(valor.into())));
        self
    }

    /// Clave del catalogo de la que sale.
    pub fn clave(&self) -> &'static str {
        self.texto.clave
    }

    /// El texto en un idioma, con los huecos rellenos. Un hueco sin valor se
    /// deja tal cual (`{nombre}`), y lo que traen los valores no se vuelve a
    /// sustituir.
    pub fn texto(&self, idioma: Idioma) -> String {
        let plantilla = self.texto.en_idioma(idioma);
        let mut s = String::with_capacity(plantilla.len() + 16);
        let mut resto = plantilla;
        while let Some(ini) = resto.find('{') {
            s.push_str(&resto[..ini]);
            let tras = &resto[ini + 1..];
            match tras.find('}') {
                Some(fin) if es_nombre(&tras[..fin]) => {
                    let nombre = &tras[..fin];
                    match self.args.iter().find(|(n, _)| *n == nombre) {
                        Some((_, Arg::Texto(t))) => s.push_str(t),
                        Some((_, Arg::Mensaje(m))) => s.push_str(&m.texto(idioma)),
                        None => {
                            // Un hueco sin valor es un fallo de quien arma el
                            // mensaje: en los tests (debug) revienta; en
                            // release se deja visible, sin perder el resto.
                            debug_assert!(
                                false,
                                "{}: falta el valor del hueco {{{nombre}}}",
                                self.texto.clave
                            );
                            let _ = write!(s, "{{{nombre}}}");
                        }
                    }
                    resto = &tras[fin + 1..];
                }
                _ => {
                    s.push('{');
                    resto = tras;
                }
            }
        }
        s.push_str(resto);
        s
    }
}

impl From<&'static Texto> for Mensaje {
    fn from(texto: &'static Texto) -> Self {
        Self::nuevo(texto)
    }
}

/// Nombre valido de hueco: letras ASCII minusculas, digitos y `_`.
fn es_nombre(t: &str) -> bool {
    !t.is_empty()
        && t.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Huecos `{nombre}` de una plantilla, ordenados y sin repetir.
pub fn huecos(plantilla: &str) -> Vec<&str> {
    let mut v = Vec::new();
    let mut resto = plantilla;
    while let Some(ini) = resto.find('{') {
        let tras = &resto[ini + 1..];
        match tras.find('}') {
            Some(fin) if es_nombre(&tras[..fin]) => {
                v.push(&tras[..fin]);
                resto = &tras[fin + 1..];
            }
            _ => resto = tras,
        }
    }
    v.sort_unstable();
    v.dedup();
    v
}

/// Define entradas del catalogo. Cada una exige `es` y `en`: si falta una,
/// la macro no casa y no compila. Genera ademas `TODOS`, la lista de las
/// entradas del modulo, que recorre el test del catalogo.
macro_rules! textos {
    ($( $(#[$meta:meta])* $nombre:ident { es: $es:literal, en: $en:literal $(,)? } )*) => {
        $(
            $(#[$meta])*
            pub static $nombre: $crate::i18n::Texto = $crate::i18n::Texto {
                clave: concat!(module_path!(), "::", stringify!($nombre)),
                es: $es,
                en: $en,
            };
        )*
        /// Todas las entradas de este modulo del catalogo.
        pub static TODOS: &[&$crate::i18n::Texto] = &[$(&$nombre),*];
    };
}
pub(crate) use textos;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn entorno<'a>(pares: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |k| {
            pares
                .iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| (*v).to_owned())
        }
    }

    #[test]
    fn resolucion_de_idioma() {
        let casos: &[(&[(&str, &str)], Idioma)] = &[
            (&[], Idioma::Es),
            (&[("LANG", "en_US.UTF-8")], Idioma::En),
            (&[("LANG", "es_ES.UTF-8")], Idioma::Es),
            (&[("LANG", "C")], Idioma::Es),
            (&[("LANG", "POSIX")], Idioma::Es),
            (&[("LANG", "fr_FR.UTF-8")], Idioma::Es),
            (&[("LANG", "")], Idioma::Es),
            (&[("LANG", "en_US@euro")], Idioma::En),
            (&[("LANG", "es-419")], Idioma::Es),
            // Prioridad: TC2_IDIOMA > LC_ALL > LC_MESSAGES > LANG.
            (&[("TC2_IDIOMA", "es"), ("LC_ALL", "en_US")], Idioma::Es),
            (&[("TC2_IDIOMA", "EN"), ("LANG", "es_ES")], Idioma::En),
            (&[("LC_ALL", "en_GB"), ("LANG", "es_ES")], Idioma::En),
            (&[("LC_MESSAGES", "es_ES"), ("LANG", "en_US")], Idioma::Es),
            // Un valor que no se reconoce se salta y manda el siguiente.
            (&[("TC2_IDIOMA", "klingon"), ("LANG", "en_US")], Idioma::En),
            (&[("LC_ALL", "C"), ("LANG", "en_US")], Idioma::En),
        ];
        for (pares, esperado) in casos {
            assert_eq!(Idioma::de_entorno(entorno(pares)), *esperado, "{pares:?}");
        }
    }

    #[test]
    fn huecos_y_sustitucion() {
        static T: Texto = Texto {
            clave: "prueba::T",
            es: "{op}: no existe {ruta} {x-y} {",
            en: "{op}: {ruta} does not exist {x-y} {",
        };
        let m = Mensaje::nuevo(&T).con("op", "push").con("ruta", "{op}");
        assert_eq!(m.texto(Idioma::Es), "push: no existe {op} {x-y} {");
        assert_eq!(m.texto(Idioma::En), "push: {op} does not exist {x-y} {");
        assert_eq!(huecos(T.es), vec!["op", "ruta"]);
        let anidado = Mensaje::nuevo(&catalogo::comun::EXTERNO).con_mensaje("texto", m);
        assert_eq!(
            anidado.texto(Idioma::En),
            "push: {op} does not exist {x-y} {"
        );
    }

    /// Criterio de F2-8: falla si falta una traduccion. Que falte una de las
    /// dos ya no compila (macro `textos!`); aqui se comprueba que ninguna esta
    /// vacia, que el ingles no es el espanol copiado (salvo una lista
    /// cerrada), que llevan los mismos huecos y que las claves no se repiten.
    #[test]
    fn catalogo_completo() {
        // Entradas que pueden ser iguales en los dos idiomas: solo un hueco o
        // solo nombres de ordenes.
        const IGUALES: &[&str] = &["tc2::i18n::catalogo::comun::EXTERNO"];
        let mut claves = HashSet::new();
        let mut total = 0;
        for t in catalogo::todos() {
            total += 1;
            assert!(claves.insert(t.clave), "clave repetida: {}", t.clave);
            assert!(
                !t.es.trim().is_empty(),
                "{}: falta el texto en espanol",
                t.clave
            );
            assert!(
                !t.en.trim().is_empty(),
                "{}: falta el texto en ingles",
                t.clave
            );
            assert!(
                t.es != t.en || IGUALES.contains(&t.clave),
                "{}: el texto en ingles es el mismo que en espanol (sin traducir)",
                t.clave
            );
            assert_eq!(
                huecos(t.es),
                huecos(t.en),
                "{}: los huecos no coinciden entre idiomas",
                t.clave
            );
        }
        assert!(total > 1, "el catalogo esta vacio");
    }
}
