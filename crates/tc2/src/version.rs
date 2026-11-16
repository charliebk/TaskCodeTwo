//! `tc2 version [--json]` y `tc2 compatible --protocolo A-B` (F2-12, ADR
//! F1-14 punto 5): lo que el lanzador del plugin pregunta a un `tc2`
//! instalado aparte para decidir si lo usa.
//!
//! `version --json` devuelve exactamente `{"version", "protocolo", "target",
//! "sha256"}`. El sha256 es el del fichero del propio ejecutable: sirve para
//! compararlo con `cli.lock`, pero quien comprueba es siempre otro proceso
//! (punto 4): un binario manipulado podria mentir sobre si mismo.

use std::path::Path;

use distribucion::Target;

use crate::contrato::{Informe, Resultado, cadena_json};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::actualizar as txt;

/// Protocolo entre el plugin y el CLI: sube cuando cambia algo que el plugin
/// usa (subcomandos, `--json`, codigos de salida). ADR F1-14, punto 5. Es
/// el de la caja `distribucion`, el mismo que escribe `cargo xtask cli-lock`.
pub const PROTOCOLO: u32 = distribucion::PROTOCOLO;

/// Nombre de la orden `version`.
pub const VERSION: &str = "version";
/// Nombre de la orden `compatible`.
pub const COMPATIBLE: &str = "compatible";

/// Lo que devuelve `version`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatosVersion {
    /// Version del CLI.
    pub version: String,
    /// [`PROTOCOLO`].
    pub protocolo: u32,
    /// Alias de P12 (`win-x64`...) o `<os>-<arch>` fuera de P12.
    pub target: String,
    /// sha256 del fichero del ejecutable.
    pub sha256: String,
}

impl DatosVersion {
    /// El JSON de una linea con exactamente las cuatro claves, en este orden.
    pub fn json(&self) -> String {
        format!(
            "{{\"version\":{},\"protocolo\":{},\"target\":{},\"sha256\":{}}}",
            cadena_json(&self.version),
            self.protocolo,
            cadena_json(&self.target),
            cadena_json(&self.sha256)
        )
    }
}

/// Alias de P12 de este binario o, fuera de P12, `<os>-<arch>`.
pub fn target_actual() -> String {
    Target::actual().map_or_else(
        || format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
        |t| t.alias.to_owned(),
    )
}

/// Datos de `version` para el ejecutable de `ruta`.
pub fn datos(ruta: &Path) -> Result<DatosVersion, Informe> {
    let bytes = std::fs::read(ruta).map_err(|e| {
        Informe::nuevo(VERSION, Resultado::Error).diagnostico(
            Mensaje::nuevo(&txt::NO_SE_LEE_EL_EJECUTABLE)
                .con("ruta", ruta.display().to_string())
                .con("error", e.to_string()),
        )
    })?;
    Ok(DatosVersion {
        version: crate::VERSION.to_owned(),
        protocolo: PROTOCOLO,
        target: target_actual(),
        sha256: distribucion::sha256::hex(&bytes),
    })
}

/// `version` sin `--json`: la linea de contrato.
pub fn informe(d: &DatosVersion) -> Informe {
    Informe::nuevo(VERSION, Resultado::Ok)
        .con("version", d.version.clone())
        .con("protocolo", d.protocolo.to_string())
        .con("target", d.target.clone())
        .con("sha256", d.sha256.clone())
}

/// Un rango de protocolo `A-B` (o `A`, que es `A-A`), con `A <= B`.
pub fn leer_rango(texto: &str) -> Option<(u32, u32)> {
    let numero = |t: &str| {
        (!t.is_empty() && t.len() <= 9 && t.bytes().all(|b| b.is_ascii_digit()))
            .then(|| t.parse::<u32>().ok())
            .flatten()
    };
    let (a, b) = match texto.split_once('-') {
        Some((a, b)) => (numero(a)?, numero(b)?),
        None => {
            let a = numero(texto)?;
            (a, a)
        }
    };
    (a <= b).then_some((a, b))
}

/// `compatible --protocolo A-B`: `ok` si [`PROTOCOLO`] cae en el rango;
/// si no, `rechazada` (3) con `motivo=protocolo-incompatible` y el arreglo.
pub fn compatible(args: &[String]) -> Informe {
    let rango = match args {
        [p, r] if p == "--protocolo" => leer_rango(r).map(|x| (x, r.as_str())),
        _ => None,
    };
    let Some(((a, b), texto)) = rango else {
        return Informe::nuevo(COMPATIBLE, Resultado::Uso)
            .con("motivo", "argumentos")
            .diagnostico(&txt::USO_COMPATIBLE);
    };
    let base = |r| {
        Informe::nuevo(COMPATIBLE, r)
            .con("protocolo", PROTOCOLO.to_string())
            .con("rango", texto)
    };
    if (a..=b).contains(&PROTOCOLO) {
        return base(Resultado::Ok);
    }
    base(Resultado::Rechazada)
        .con("motivo", "protocolo-incompatible")
        .con("arreglo", "tc2 actualizar --cli")
        .diagnostico(
            Mensaje::nuevo(&txt::PROTOCOLO_INCOMPATIBLE)
                .con("protocolo", PROTOCOLO.to_string())
                .con("rango", texto),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rangos_de_protocolo() {
        assert_eq!(leer_rango("1-1"), Some((1, 1)));
        assert_eq!(leer_rango("1-3"), Some((1, 3)));
        assert_eq!(leer_rango("2"), Some((2, 2)));
        for malo in ["", "-", "1-", "-1", "3-1", "a-b", "1-2-3", "+1-2", "1 - 2"] {
            assert_eq!(leer_rango(malo), None, "{malo}");
        }
    }

    #[test]
    fn json_con_las_cuatro_claves() {
        let d = DatosVersion {
            version: "2.0.0".into(),
            protocolo: 1,
            target: "win-x64".into(),
            sha256: "ab".into(),
        };
        assert_eq!(
            d.json(),
            "{\"version\":\"2.0.0\",\"protocolo\":1,\"target\":\"win-x64\",\"sha256\":\"ab\"}"
        );
    }
}
