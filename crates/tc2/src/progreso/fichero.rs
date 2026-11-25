//! El fichero de progreso de una tarea (F3-12): `progreso/TASK-NNN.json` en
//! la carpeta propia de la carpeta comun de git ([`crate::huella`]), con el
//! anterior en `TASK-NNN.anterior.json`.
//!
//! Lleva lo que la vista necesita para pintar sin recorrer el historico
//! (L1 de F1-17): la fase, las que faltan, lo transcurrido al calcular, el
//! ultimo evento, el nivel, `n` y la tabla de lo que falta de 0 a
//! [`TOPE_MIN`] minutos cada [`PASO_MIN`] (sacada de
//! [`crate::estimacion::restante`]). Su tamano no depende del numero de
//! tareas. Lleva `version` para poder anadir el lote (F4-9).
//!
//! Lo que se lee es texto no confiable (otro proceso o una persona puede
//! tocarlo): se valida al leer y se sanea al pintar.

use std::io::Read;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Version del formato del fichero.
pub const VERSION: u32 = 1;

/// Minutos entre dos puntos de la tabla.
pub const PASO_MIN: u64 = 5;

/// Minutos del ultimo punto de la tabla (8 h).
pub const TOPE_MIN: u64 = 480;

/// Puntos de la tabla: de 0 a [`TOPE_MIN`] cada [`PASO_MIN`].
pub const PUNTOS: usize = (TOPE_MIN / PASO_MIN) as usize + 1;

/// Bytes como mucho de un fichero de progreso: uno mayor no es de `tc2`.
pub const LIMITE: u64 = 64 * 1024;

/// Avisos como mucho en un fichero.
pub const MAX_AVISOS: usize = 4;

/// Lo que falta con un transcurrido dado.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Punto {
    /// Mediana de lo que falta (con `desbordada`, cota inferior).
    pub mediana: f64,
    /// p80 de lo que falta.
    pub p80: f64,
    /// La fase en curso va por encima de sus parecidas.
    pub desbordada: bool,
}

/// El progreso de una tarea en curso, tal como lo deja la ultima transicion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Progreso {
    /// [`VERSION`].
    pub version: u32,
    /// `TASK-NNN`.
    pub id: String,
    /// `implementacion` o `rondas`.
    pub fase: String,
    /// Si la ultima de sus filas `pausa`/`reanudar` es `pausa`: el reloj no
    /// corre.
    pub pausada: bool,
    /// Fases que faltan despues de la actual, en orden.
    pub faltan: Vec<String>,
    /// Minutos activos de la fase en curso hasta [`Progreso::ultimo_evento`]
    /// (regla del real de F1-17: huecos a 45 min, sin pausas).
    pub transcurrido: f64,
    /// Instante del ultimo evento de la tarea (segundos desde 1970).
    pub ultimo_evento: u64,
    /// Instante del calculo (segundos desde 1970).
    pub calculada: u64,
    /// Nivel de la cascada o `sin-datos`.
    pub nivel: String,
    /// Muestras del nivel (con `sin-datos`, las disponibles).
    pub n: usize,
    /// Lo que falta cada [`PASO_MIN`] minutos transcurridos; vacia sin
    /// estimacion.
    pub tabla: Vec<Punto>,
    /// Avisos para la vista (se sanean al escribir y al pintar).
    pub avisos: Vec<String>,
}

impl Progreso {
    /// Si lo leido tiene la forma que escribe este `tc2`.
    pub fn valido(&self) -> bool {
        self.version == VERSION
            && crate::git::es_id(&self.id)
            && matches!(self.fase.as_str(), "implementacion" | "rondas")
            && self
                .faltan
                .iter()
                .all(|f| crate::estimacion::FASES.contains(&f.as_str()))
            && self.tabla.len() <= PUNTOS
            && self.avisos.len() <= MAX_AVISOS
            && self.transcurrido.is_finite()
            && self.transcurrido >= 0.0
            && self
                .tabla
                .iter()
                .all(|p| p.mediana.is_finite() && p.p80.is_finite())
    }

    /// El JSON del fichero.
    pub fn json(&self) -> Vec<u8> {
        let mut v = serde_json::to_vec(self).unwrap_or_default();
        v.push(b'\n');
        v
    }
}

/// Carpeta del progreso del repo cuya carpeta comun de git es `comun`.
pub fn carpeta(comun: &Path) -> PathBuf {
    crate::huella::en_comun(comun).join(crate::huella::CARPETA_PROGRESO)
}

/// `TASK-NNN.json` en `carpeta`.
pub fn ruta(carpeta: &Path, id: &str) -> PathBuf {
    carpeta.join(format!("{id}.json"))
}

/// `TASK-NNN.anterior.json` en `carpeta`.
pub fn ruta_anterior(carpeta: &Path, id: &str) -> PathBuf {
    carpeta.join(format!("{id}.anterior.json"))
}

/// Bytes de `ruta`, con tope; `None` si no se lee o pasa de [`LIMITE`].
fn bytes(ruta: &Path) -> Option<Vec<u8>> {
    let mut v = Vec::new();
    std::fs::File::open(ruta)
        .ok()?
        .take(LIMITE + 1)
        .read_to_end(&mut v)
        .ok()?;
    (v.len() as u64 <= LIMITE).then_some(v)
}

/// Un progreso valido de `ruta`, si lo hay.
fn de(ruta: &Path, id: &str) -> Option<Progreso> {
    de_bytes(&bytes(ruta)?, id)
}

fn de_bytes(b: &[u8], id: &str) -> Option<Progreso> {
    let p: Progreso = serde_json::from_slice(b).ok()?;
    (p.valido() && p.id == id).then_some(p)
}

/// Si `b` es un progreso valido de `id`.
fn valido(b: &[u8], id: &str) -> bool {
    de_bytes(b, id).is_some()
}

/// Lo que da la lectura de una tarea.
#[derive(Debug, Clone, PartialEq)]
pub enum Lectura {
    /// No hay fichero: la tarea no esta en curso en este clon.
    Ninguno,
    /// El fichero, valido.
    Bueno(Progreso),
    /// El fichero no se entiende (a medias o tocado): el anterior, que si.
    Anterior(Progreso),
    /// Ni el fichero ni el anterior se entienden.
    Ilegible,
}

/// Lee el progreso de `id` en `carpeta`. Sin fichero, [`Lectura::Ninguno`]
/// (el anterior no cuenta: la tarea dejo de estar en curso).
pub fn leer(carpeta: &Path, id: &str) -> Lectura {
    let r = ruta(carpeta, id);
    if !r.exists() {
        return Lectura::Ninguno;
    }
    if let Some(p) = de(&r, id) {
        return Lectura::Bueno(p);
    }
    de(&ruta_anterior(carpeta, id), id).map_or(Lectura::Ilegible, Lectura::Anterior)
}

/// Que paso con el `.anterior.json` al escribir un progreso.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anterior {
    /// El que habia paso a ser el anterior.
    Rotado,
    /// No habia progreso: no hay anterior que escribir.
    SinPrevio,
    /// Habia progreso pero no paso a ser el anterior: no se pudo leer
    /// entero (a medias, o abierto por otro proceso) o no se pudo escribir.
    /// El nuevo se escribio igual (TASK-128: ya no se calla).
    NoEscrito,
}

/// Escribe `p` en `carpeta`: antes, el que habia pasa a `.anterior.json`.
/// Cada paso, de una vez (`crate::escritura::atomica`). Si el anterior
/// no se puede escribir, el progreso se escribe igual (revision 1 de
/// TASK-084, R1-04): el anterior solo es el respaldo de un fichero a medias,
/// y sin el nuevo la linea se quedaria con el dato viejo. Lo que paso con el
/// anterior se devuelve (TASK-128) para que la transicion lo diga.
pub fn escribir(carpeta: &Path, p: &Progreso) -> std::io::Result<Anterior> {
    let r = ruta(carpeta, &p.id);
    // Solo un anterior valido: si el actual quedo a medias, el anterior que
    // habia sigue siendo el ultimo bueno.
    let anterior = if !r.exists() {
        Anterior::SinPrevio
    } else {
        // Los bytes tal cual, no el progreso vuelto a serializar: un `f64`
        // leido y reescrito por `serde_json` puede cambiar en la ultima cifra
        // (TASK-128), y el anterior dejaria de ser el fichero que habia.
        match bytes(&r).filter(|b| valido(b, &p.id)) {
            Some(previo) => {
                match crate::escritura::atomica(&ruta_anterior(carpeta, &p.id), &previo) {
                    Ok(()) => Anterior::Rotado,
                    Err(_) => Anterior::NoEscrito,
                }
            }
            None => Anterior::NoEscrito,
        }
    };
    crate::escritura::atomica(&r, &p.json())?;
    Ok(anterior)
}

/// Borra el progreso de `id` (y su anterior). Que no exista no es un fallo.
pub fn borrar(carpeta: &Path, id: &str) -> std::io::Result<()> {
    for r in [ruta(carpeta, id), ruta_anterior(carpeta, id)] {
        match std::fs::remove_file(&r) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e),
            _ => {}
        }
    }
    Ok(())
}

/// IDs con fichero de progreso en `carpeta` (sin los anteriores), en orden.
pub fn ids(carpeta: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(carpeta)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let n = e.file_name().into_string().ok()?;
            let id = n.strip_suffix(".json")?;
            crate::git::es_id(id).then(|| id.to_owned())
        })
        .collect();
    v.sort();
    v
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Un progreso de ejemplo.
    pub(crate) fn ejemplo(id: &str) -> Progreso {
        Progreso {
            version: VERSION,
            id: id.to_owned(),
            fase: "implementacion".to_owned(),
            pausada: false,
            faltan: vec!["rondas".to_owned(), "cierre".to_owned()],
            transcurrido: 0.0,
            ultimo_evento: 1_000,
            calculada: 1_000,
            nivel: "repo".to_owned(),
            n: 6,
            tabla: vec![
                Punto {
                    mediana: 50.0,
                    p80: 65.0,
                    desbordada: false,
                };
                PUNTOS
            ],
            avisos: Vec::new(),
        }
    }

    fn dir(nombre: &str) -> PathBuf {
        let d = std::env::temp_dir()
            .join("tc2-tests")
            .join(format!("progreso-fichero-{nombre}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    /// TASK-128: `escribir` dice que paso con el anterior. Sin progreso,
    /// `SinPrevio`; con uno bueno, `Rotado`; si el anterior no se puede
    /// escribir (aqui, una carpeta ocupa su ruta), `NoEscrito` y el nuevo
    /// queda escrito igual; sobre uno a medias, `NoEscrito`.
    #[test]
    fn escribir_dice_que_paso_con_el_anterior() -> std::io::Result<()> {
        let d = dir("rotacion");
        let mut p = ejemplo("TASK-001");
        assert_eq!(escribir(&d, &p)?, Anterior::SinPrevio);
        p.calculada = 2_000;
        assert_eq!(escribir(&d, &p)?, Anterior::Rotado);
        std::fs::remove_file(ruta_anterior(&d, "TASK-001"))?;
        std::fs::create_dir_all(ruta_anterior(&d, "TASK-001").join("ocupada"))?;
        p.calculada = 3_000;
        assert_eq!(escribir(&d, &p)?, Anterior::NoEscrito);
        assert_eq!(leer(&d, "TASK-001"), Lectura::Bueno(p.clone()));
        std::fs::remove_dir_all(ruta_anterior(&d, "TASK-001"))?;
        let json = p.json();
        std::fs::write(ruta(&d, "TASK-001"), &json[..json.len() / 2])?;
        p.calculada = 4_000;
        assert_eq!(escribir(&d, &p)?, Anterior::NoEscrito);
        assert_eq!(leer(&d, "TASK-001"), Lectura::Bueno(p));
        let _ = std::fs::remove_dir_all(&d);
        Ok(())
    }

    /// TASK-128, causa del `progreso::c3` intermitente: el anterior es el
    /// fichero que habia byte a byte, tambien con un `transcurrido` que
    /// `serde_json` no devuelve igual al leerlo y reescribirlo (este valor
    /// cambia en la ultima cifra; con el progreso re-serializado, falla).
    #[test]
    fn el_anterior_es_el_fichero_que_habia_byte_a_byte() -> std::io::Result<()> {
        let d = dir("bytes");
        let mut p = ejemplo("TASK-001");
        p.transcurrido = 0.009_027_842_777_777_777;
        escribir(&d, &p)?;
        let habia = std::fs::read(ruta(&d, "TASK-001"))?;
        assert_eq!(habia, p.json());
        let mut q = p.clone();
        q.calculada = 2_000;
        assert_eq!(escribir(&d, &q)?, Anterior::Rotado);
        assert_eq!(std::fs::read(ruta_anterior(&d, "TASK-001"))?, habia);
        let _ = std::fs::remove_dir_all(&d);
        Ok(())
    }

    /// C3 (en pequeno) y C8: al reescribir, el bueno pasa a `.anterior`; un
    /// fichero a medias da el anterior; sin fichero, ninguno.
    #[test]
    fn anterior_y_fichero_a_medias() -> std::io::Result<()> {
        let d = dir("anterior");
        assert_eq!(leer(&d, "TASK-001"), Lectura::Ninguno);
        let mut p = ejemplo("TASK-001");
        escribir(&d, &p)?;
        assert_eq!(leer(&d, "TASK-001"), Lectura::Bueno(p.clone()));
        assert!(!ruta_anterior(&d, "TASK-001").exists());
        let primero = p.clone();
        p.calculada = 2_000;
        escribir(&d, &p)?;
        assert_eq!(leer(&d, "TASK-001"), Lectura::Bueno(p.clone()));
        assert_eq!(
            std::fs::read(ruta_anterior(&d, "TASK-001"))?,
            primero.json()
        );
        // A medias: el anterior.
        let json = p.json();
        std::fs::write(ruta(&d, "TASK-001"), &json[..json.len() / 2])?;
        assert_eq!(leer(&d, "TASK-001"), Lectura::Anterior(primero.clone()));
        // Escribir sobre uno a medias conserva el anterior bueno.
        p.calculada = 3_000;
        escribir(&d, &p)?;
        assert_eq!(
            std::fs::read(ruta_anterior(&d, "TASK-001"))?,
            primero.json()
        );
        // Los dos mal: ilegible.
        std::fs::write(ruta(&d, "TASK-001"), b"{")?;
        std::fs::write(ruta_anterior(&d, "TASK-001"), b"x")?;
        assert_eq!(leer(&d, "TASK-001"), Lectura::Ilegible);
        // Otro ID dentro, una version que no es o enorme: no valen.
        let mut otro = ejemplo("TASK-002");
        std::fs::write(ruta(&d, "TASK-001"), otro.json())?;
        assert_eq!(leer(&d, "TASK-001"), Lectura::Ilegible);
        otro.id = "TASK-001".to_owned();
        otro.version = 2;
        std::fs::write(ruta(&d, "TASK-001"), otro.json())?;
        assert_eq!(leer(&d, "TASK-001"), Lectura::Ilegible);
        std::fs::write(ruta(&d, "TASK-001"), vec![b' '; LIMITE as usize + 1])?;
        assert_eq!(leer(&d, "TASK-001"), Lectura::Ilegible);
        assert_eq!(ids(&d), ["TASK-001"]);
        borrar(&d, "TASK-001")?;
        borrar(&d, "TASK-001")?;
        assert_eq!(leer(&d, "TASK-001"), Lectura::Ninguno);
        assert!(ids(&d).is_empty());
        let _ = std::fs::remove_dir_all(&d);
        Ok(())
    }

    /// Revision 1 de TASK-084 (R1-04): si el anterior no se puede escribir
    /// (aqui, una carpeta con su nombre), el progreso se escribe igual.
    #[test]
    fn sin_anterior_escribe_el_progreso() -> std::io::Result<()> {
        let d = dir("anterior-roto");
        let mut p = ejemplo("TASK-001");
        escribir(&d, &p)?;
        std::fs::create_dir_all(ruta_anterior(&d, "TASK-001").join("ocupado"))?;
        p.calculada = 2_000;
        escribir(&d, &p)?;
        assert_eq!(leer(&d, "TASK-001"), Lectura::Bueno(p));
        assert!(ruta_anterior(&d, "TASK-001").is_dir());
        let _ = std::fs::remove_dir_all(&d);
        Ok(())
    }
}
