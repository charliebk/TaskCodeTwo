//! Lectura acotada de un fichero externo (A3-03): la de `plan guardar`,
//! `approve --pedir-cambios` e `import`, en un solo sitio y con un solo
//! tope. Cada orden traduce el fallo a su propio informe.
//!
//! El fichero lo pasa quien lanza la orden: es un dato no confiable (F1-8),
//! asi que la ruta y el error del sistema salen ya saneados
//! ([`sanear`]: controles a espacio, 200 caracteres como mucho).
//!
//! Tambien la lectura de un fichero propio del proyecto o de la maquina
//! (la de `init`, `desinstalar` y la configuracion, A7-01 y A7-09):
//! [`opcional`] distingue «no existe» de «no se lee», y [`no_se_lee`] da el
//! informe `error` de lo segundo a nombre de la orden. Esas rutas las
//! calcula `tc2`, no las pasa la persona: no se sanean.
//!
//! Y el «leer con tope» de todos (A8-17): [`acotado`] lee como mucho un
//! byte mas del tope y rechaza si llega a leerlo; [`con_tope`] y
//! [`texto_con_tope`] lo aplican a un fichero. El texto es UTF-8 estricto:
//! unos bytes que no lo son dan `InvalidData`, nunca `U+FFFD`.

use std::io::{self, Read as _};
use std::path::Path;

use crate::contrato::{Informe, Resultado};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::comun::NO_SE_LEE;
use crate::plataforma::errores::sanear;

/// Informe `{op} error ruta=...` de un fichero que existe y no se lee.
pub(crate) fn no_se_lee(op: &str, ruta: &Path, e: &io::Error) -> Informe {
    let r = ruta.display().to_string();
    Informe::nuevo(op, Resultado::Error)
        .con("ruta", r.clone())
        .diagnostico(
            Mensaje::nuevo(&NO_SE_LEE)
                .con("ruta", r)
                .con("error", e.to_string()),
        )
}

/// Lee un fichero; `None` si no existe y [`no_se_lee`] si no se lee.
pub(crate) fn opcional(op: &str, ruta: &Path) -> Result<Option<Vec<u8>>, Informe> {
    match std::fs::read(ruta) {
        Ok(b) => Ok(Some(b)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(no_se_lee(op, ruta, &e)),
    }
}

/// Lee de `r` como mucho `tope` bytes: se lee un byte de mas, y si llega a
/// leerse, `None` (sin carrera entre mirar el tamano y leer).
pub(crate) fn acotado(r: impl io::Read, tope: u64) -> io::Result<Option<Vec<u8>>> {
    let mut b = Vec::new();
    r.take(tope.saturating_add(1)).read_to_end(&mut b)?;
    Ok((b.len() as u64 <= tope).then_some(b))
}

/// Los bytes de `ruta` con [`acotado`]: `None` si pasa de `tope`.
pub(crate) fn con_tope(ruta: &Path, tope: u64) -> io::Result<Option<Vec<u8>>> {
    acotado(std::fs::File::open(ruta)?, tope)
}

/// El texto de `ruta` con [`con_tope`]: `None` si pasa de `tope` e
/// `InvalidData` si no es UTF-8.
pub(crate) fn texto_con_tope(ruta: &Path, tope: u64) -> io::Result<Option<String>> {
    con_tope(ruta, tope)?
        .map(|b| String::from_utf8(b).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
        .transpose()
}

/// Tope de un fichero externo (D4 de TASK-029).
pub const MAX_BYTES: usize = 1 << 20;

/// Por que no se leyo el fichero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motivo {
    /// No se abre o falla la lectura (`error` dice por que).
    NoSeLee,
    /// Pasa del tope.
    Grande,
    /// No es UTF-8.
    NoUtf8,
    /// Vacio o solo con espacios.
    Vacio,
}

/// Un fichero que no se leyo: el motivo, y la ruta y el error saneados
/// (`error` vacio salvo en [`Motivo::NoSeLee`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoSeLeyo {
    /// Por que.
    pub motivo: Motivo,
    /// La ruta, saneada.
    pub ruta: String,
    /// El error del sistema, saneado.
    pub error: String,
}

/// Lee `ruta` sin cargar mas de `tope` bytes: se lee como mucho un byte de
/// mas, y si llega a leerse el fichero es demasiado grande (sin carrera
/// entre mirar el tamano y leer). UTF-8 estricto y no vacio.
pub fn fichero(ruta: &Path, tope: usize) -> Result<String, NoSeLeyo> {
    let mal = |motivo: Motivo, error: String| NoSeLeyo {
        motivo,
        ruta: sanear(&ruta.display().to_string()),
        error,
    };
    let no_se_lee = |e: io::Error| mal(Motivo::NoSeLee, sanear(&e.to_string()));
    let f = std::fs::File::open(ruta).map_err(no_se_lee)?;
    let bytes = acotado(f, u64::try_from(tope).unwrap_or(u64::MAX))
        .map_err(no_se_lee)?
        .ok_or_else(|| mal(Motivo::Grande, String::new()))?;
    let texto = String::from_utf8(bytes).map_err(|_| mal(Motivo::NoUtf8, String::new()))?;
    if texto.trim().is_empty() {
        return Err(mal(Motivo::Vacio, String::new()));
    }
    Ok(texto)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Bytes que no son UTF-8: `NoUtf8`, sin reemplazarlos por `\u{FFFD}`;
    /// el mismo fichero en UTF-8, se lee.
    #[test]
    fn no_utf8_se_rechaza() -> std::io::Result<()> {
        let dir = std::env::temp_dir()
            .join("tc2-tests")
            .join(format!("lectura-{}", std::process::id()));
        std::fs::create_dir_all(&dir)?;
        let malo = dir.join("binario.md");
        std::fs::write(&malo, [b'a', 0xff, 0xfe, b'b'])?;
        let r = fichero(&malo, MAX_BYTES).map_err(|n| n.motivo);
        let bueno = dir.join("texto.md");
        std::fs::write(&bueno, "a\u{f1}b")?;
        let b = fichero(&bueno, MAX_BYTES).map_err(|n| n.motivo);
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(r, Err(Motivo::NoUtf8));
        assert_eq!(b, Ok("a\u{f1}b".to_owned()));
        Ok(())
    }

    /// Frontera del tope: `tope` bytes se leen enteros; `tope + 1`, `None`.
    #[test]
    fn acotado_en_la_frontera() -> io::Result<()> {
        assert_eq!(acotado(&b"abcd"[..], 4)?, Some(b"abcd".to_vec()));
        assert_eq!(acotado(&b"abcde"[..], 4)?, None);
        assert_eq!(acotado(&b""[..], 0)?, Some(Vec::new()));
        assert_eq!(acotado(&b"a"[..], 0)?, None);
        Ok(())
    }

    /// Decision 3 de TASK-150: `texto_con_tope` rechaza lo que no es UTF-8
    /// con `InvalidData` (sin `U+FFFD`); `con_tope` da los bytes tal cual.
    #[test]
    fn texto_con_tope_es_utf8_estricto() -> io::Result<()> {
        let dir = std::env::temp_dir()
            .join("tc2-tests")
            .join(format!("lectura-tope-{}", std::process::id()));
        std::fs::create_dir_all(&dir)?;
        let malo = dir.join("binario");
        std::fs::write(&malo, [b'a', 0xff])?;
        let texto = texto_con_tope(&malo, 16).map_err(|e| e.kind());
        let bytes = con_tope(&malo, 16)?;
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(texto, Err(io::ErrorKind::InvalidData));
        assert_eq!(bytes, Some(vec![b'a', 0xff]));
        Ok(())
    }
}
