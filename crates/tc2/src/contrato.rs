//! Contrato de salida de las operaciones (ADR F1-4, decisiones 4 y 5).
//!
//! Toda operacion devuelve un [`Informe`]. En el caso normal se ve como una
//! sola linea `<comando> <resultado> clave=valor...` en stdout, sin ANSI; el
//! diagnostico va a stderr y solo si la operacion no salio bien. Con `--json`
//! la misma informacion sale como un objeto JSON en una linea.
//!
//! La tabla de codigos de salida vive aqui y solo aqui: la propuso F1-4 y la
//! fija F2-3 (0 ok y sin-cambios, 1 error, 2 uso, 3 rechazada, 4 sinred,
//! 5 agotada, 6 conflicto, 7 esperando, 8 denegada).

use crate::Salida;
use crate::i18n::{Idioma, Mensaje};

/// Resultado de una operacion. Cada uno lleva su codigo de salida.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resultado {
    /// Hecho.
    Ok,
    /// Ya estaba hecho: no se ha cambiado nada.
    SinCambios,
    /// La operacion no procede; el motivo va en la linea. No se reintenta.
    Rechazada,
    /// No se pudo hablar con el remoto.
    Sinred,
    /// Se agotaron los reintentos sin escribir nada.
    Agotada,
    /// Hay un conflicto que resolver a mano.
    Conflicto,
    /// Puertas pendientes en la plataforma (lo emite el puerto Plataforma).
    Esperando,
    /// El remoto deniega la operacion (credenciales, permisos, un hook o una
    /// regla, o no existe): no se reintenta, se avisa a la persona (F2-3).
    Denegada,
    /// La llamada esta mal formada.
    Uso,
    /// Fallo interno: no se reintenta, se avisa a la persona.
    Error,
}

impl Resultado {
    /// Palabra del resultado en la linea de contrato.
    pub fn texto(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::SinCambios => "sin-cambios",
            Self::Rechazada => "rechazada",
            Self::Sinred => "sinred",
            Self::Agotada => "agotada",
            Self::Conflicto => "conflicto",
            Self::Esperando => "esperando",
            Self::Denegada => "denegada",
            Self::Uso => "uso",
            Self::Error => "error",
        }
    }

    /// Codigo de salida del proceso (tabla de F1-4, decision 5, fijada en F2-3).
    pub fn codigo(self) -> u8 {
        match self {
            Self::Ok | Self::SinCambios => 0,
            Self::Error => 1,
            Self::Uso => 2,
            Self::Rechazada => 3,
            Self::Sinred => 4,
            Self::Agotada => 5,
            Self::Conflicto => 6,
            Self::Esperando => 7,
            Self::Denegada => 8,
        }
    }

    /// Si quien lo recibe puede seguir sin hacer nada mas.
    pub fn es_exito(self) -> bool {
        self.codigo() == 0
    }
}

/// Valor de una clave: texto suelto o lista (se escribe separada por comas).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Valor {
    /// Un valor.
    Texto(String),
    /// Varios valores; en la linea, separados por comas.
    Lista(Vec<String>),
}

/// Lo que devuelve una operacion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Informe {
    /// Nombre de la operacion o del comando (`guardar`, `push`...).
    pub comando: String,
    /// Como termino.
    pub resultado: Resultado,
    /// Claves en el orden en que se escriben.
    pub claves: Vec<(String, Valor)>,
    /// Una linea que dice que hacer; solo se muestra si no fue un exito.
    /// Es un [`Mensaje`] neutro: se traduce al escribir la salida (F2-8).
    pub diagnostico: Option<Mensaje>,
}

impl Informe {
    /// Informe sin claves.
    pub fn nuevo(comando: &str, resultado: Resultado) -> Self {
        Self {
            comando: comando.to_owned(),
            resultado,
            claves: Vec::new(),
            diagnostico: None,
        }
    }

    /// Anade una clave de texto.
    #[must_use]
    pub fn con(mut self, clave: &str, valor: impl Into<String>) -> Self {
        self.claves
            .push((clave.to_owned(), Valor::Texto(valor.into())));
        self
    }

    /// Anade una clave de lista.
    #[must_use]
    pub fn con_lista(mut self, clave: &str, valores: Vec<String>) -> Self {
        self.claves.push((clave.to_owned(), Valor::Lista(valores)));
        self
    }

    /// Fija el diagnostico (una linea), como mensaje del catalogo.
    #[must_use]
    pub fn diagnostico(mut self, mensaje: impl Into<Mensaje>) -> Self {
        self.diagnostico = Some(mensaje.into());
        self
    }

    /// Valor de texto de una clave, si existe.
    pub fn valor(&self, clave: &str) -> Option<&str> {
        self.claves.iter().find_map(|(k, v)| match v {
            Valor::Texto(t) if k == clave => Some(t.as_str()),
            _ => None,
        })
    }

    /// Linea de contrato, sin salto final.
    pub fn linea(&self) -> String {
        let mut s = format!("{} {}", self.comando, self.resultado.texto());
        for (k, v) in &self.claves {
            s.push(' ');
            s.push_str(k);
            s.push('=');
            match v {
                Valor::Texto(t) => s.push_str(&escapar_valor(t, false)),
                Valor::Lista(l) => {
                    let partes: Vec<String> = l.iter().map(|e| escapar_valor(e, true)).collect();
                    s.push_str(&partes.join(","));
                }
            }
        }
        s
    }

    /// La misma informacion en JSON, en una linea, con el diagnostico en
    /// espanol.
    pub fn json(&self) -> String {
        self.json_en(Idioma::Es)
    }

    /// La misma informacion en JSON, en una linea, con el diagnostico en el
    /// idioma pedido. Las claves y sus valores no cambian con el idioma.
    pub fn json_en(&self, idioma: Idioma) -> String {
        let mut s = format!(
            "{{\"comando\":{},\"resultado\":{},\"codigo\":{},\"claves\":{{",
            cadena_json(&self.comando),
            cadena_json(self.resultado.texto()),
            self.resultado.codigo()
        );
        for (i, (k, v)) in self.claves.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push_str(&cadena_json(k));
            s.push(':');
            match v {
                Valor::Texto(t) => s.push_str(&cadena_json(t)),
                Valor::Lista(l) => {
                    let partes: Vec<String> = l.iter().map(|e| cadena_json(e)).collect();
                    s.push('[');
                    s.push_str(&partes.join(","));
                    s.push(']');
                }
            }
        }
        s.push('}');
        if let Some(d) = self.diagnostico_visible() {
            s.push_str(",\"diagnostico\":");
            s.push_str(&cadena_json(&d.texto(idioma)));
        }
        s.push('}');
        s
    }

    fn diagnostico_visible(&self) -> Option<&Mensaje> {
        if self.resultado.es_exito() {
            None
        } else {
            self.diagnostico.as_ref()
        }
    }

    /// El diagnostico en un idioma, si lo hay (aunque fuera un exito).
    pub fn diagnostico_en(&self, idioma: Idioma) -> Option<String> {
        self.diagnostico.as_ref().map(|d| d.texto(idioma))
    }

    /// Lo convierte en la salida del proceso, con el diagnostico en espanol.
    pub fn salida(&self, json: bool) -> Salida {
        self.salida_en(json, Idioma::Es)
    }

    /// Lo convierte en la salida del proceso, con el diagnostico en el idioma
    /// pedido. stdout y el codigo no cambian con el idioma.
    pub fn salida_en(&self, json: bool, idioma: Idioma) -> Salida {
        let stdout = if json {
            self.json_en(idioma)
        } else {
            self.linea()
        };
        let stderr = match self.diagnostico_visible() {
            Some(d) => format!("{}\n", una_linea(&d.texto(idioma))),
            None => String::new(),
        };
        Salida {
            codigo: self.resultado.codigo(),
            stdout: format!("{stdout}\n"),
            stderr,
        }
    }
}

impl From<Informe> for Salida {
    fn from(i: Informe) -> Self {
        i.salida(false)
    }
}

/// Hash abreviado a 12 caracteres, como fija el contrato.
pub fn corto(hash: &str) -> String {
    hash.chars().take(12).collect()
}

/// Los valores no llevan espacios: espacio, `%`, controles (y `,` dentro de
/// una lista) se escriben como `%XX`, de forma reversible.
fn escapar_valor(v: &str, en_lista: bool) -> String {
    let mut s = String::with_capacity(v.len());
    for c in v.chars() {
        if c == '%' || c.is_whitespace() || c.is_control() || (en_lista && c == ',') {
            let mut buf = [0u8; 4];
            for b in c.encode_utf8(&mut buf).bytes() {
                s.push_str(&format!("%{b:02X}"));
            }
        } else {
            s.push(c);
        }
    }
    s
}

fn una_linea(t: &str) -> String {
    t.lines().map(str::trim).collect::<Vec<_>>().join(" ")
}

pub(crate) fn cadena_json(t: &str) -> String {
    let mut s = String::with_capacity(t.len() + 2);
    s.push('"');
    for c in t.chars() {
        match c {
            '"' => s.push_str("\\\""),
            '\\' => s.push_str("\\\\"),
            '\n' => s.push_str("\\n"),
            '\r' => s.push_str("\\r"),
            '\t' => s.push_str("\\t"),
            c if (c as u32) < 0x20 => s.push_str(&format!("\\u{:04x}", c as u32)),
            c => s.push(c),
        }
    }
    s.push('"');
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codigos_de_la_tabla_fijada_en_f2_3() {
        let tabla = [
            (Resultado::Ok, "ok", 0),
            (Resultado::SinCambios, "sin-cambios", 0),
            (Resultado::Error, "error", 1),
            (Resultado::Uso, "uso", 2),
            (Resultado::Rechazada, "rechazada", 3),
            (Resultado::Sinred, "sinred", 4),
            (Resultado::Agotada, "agotada", 5),
            (Resultado::Conflicto, "conflicto", 6),
            (Resultado::Esperando, "esperando", 7),
            (Resultado::Denegada, "denegada", 8),
        ];
        for (r, t, c) in tabla {
            assert_eq!((r.texto(), r.codigo()), (t, c));
        }
    }

    #[test]
    fn linea_de_contrato_y_json_con_las_mismas_claves() {
        let i = Informe::nuevo("guardar", Resultado::Ok)
            .con("commit", corto("48d72d1e513bffffffffffffffffffffffffffff"))
            .con("nota", "con espacio y 100%")
            .con_lista("ficheros", vec!["a,b.txt".into(), "c.txt".into()])
            .diagnostico(Mensaje::externo("no se ve en un exito"));
        let s = i.salida(false);
        assert_eq!(s.codigo, 0);
        assert_eq!(
            s.stdout,
            "guardar ok commit=48d72d1e513b nota=con%20espacio%20y%20100%25 ficheros=a%2Cb.txt,c.txt\n"
        );
        assert!(s.stderr.is_empty());
        assert_eq!(s.stdout.lines().count(), 1);
        assert!(!s.stdout.contains('\u{1b}'));
        let j = i.salida(true);
        assert_eq!(
            j.stdout,
            "{\"comando\":\"guardar\",\"resultado\":\"ok\",\"codigo\":0,\"claves\":{\"commit\":\"48d72d1e513b\",\"nota\":\"con espacio y 100%\",\"ficheros\":[\"a,b.txt\",\"c.txt\"]}}\n"
        );
    }

    #[test]
    fn el_diagnostico_va_a_stderr_solo_si_falla() {
        let s = Informe::nuevo("push", Resultado::Rechazada)
            .con("motivo", "no-fast-forward")
            .diagnostico(Mensaje::externo("lanza\nactualizar-rama"))
            .salida(false);
        assert_eq!(s.codigo, 3);
        assert_eq!(s.stdout, "push rechazada motivo=no-fast-forward\n");
        assert_eq!(s.stderr, "lanza actualizar-rama\n");
        let j = Informe::nuevo("x", Resultado::Uso)
            .diagnostico(Mensaje::externo("di \"algo\""))
            .salida(true);
        assert!(
            j.stdout
                .ends_with(",\"diagnostico\":\"di \\\"algo\\\"\"}\n")
        );
    }

    #[test]
    fn mismo_informe_dos_idiomas() {
        let i = Informe::nuevo("guardar", Resultado::Uso)
            .con("motivo", "sin-mensaje")
            .diagnostico(&crate::i18n::catalogo::cli::USO_GUARDAR);
        for json in [false, true] {
            let es = i.salida_en(json, Idioma::Es);
            let en = i.salida_en(json, Idioma::En);
            assert_eq!(es.codigo, en.codigo);
            assert_eq!(es.codigo, 2);
            // stderr cambia con el idioma y no con --json.
            assert_eq!(es.stderr, "uso: guardar \"<mensaje>\" <rutas...>\n");
            assert_eq!(en.stderr, "usage: guardar \"<message>\" <paths...>\n");
            if json {
                assert!(es.stdout.contains("\"diagnostico\":\"uso: guardar"));
                assert!(en.stdout.contains("\"diagnostico\":\"usage: guardar"));
                // Lo demas del JSON es igual en los dos idiomas.
                let sin_diag = |t: &str| {
                    t.split(",\"diagnostico\"")
                        .next()
                        .unwrap_or_default()
                        .to_owned()
                };
                assert_eq!(sin_diag(&es.stdout), sin_diag(&en.stdout));
            } else {
                assert_eq!(es.stdout, en.stdout);
                assert_eq!(es.stdout, "guardar uso motivo=sin-mensaje\n");
            }
        }
    }
}
