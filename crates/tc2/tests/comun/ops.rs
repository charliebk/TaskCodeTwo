//! Operaciones de prueba sobre la rama de estado.

use tc2::estado::{Cambio, Estado, Fallo, Operacion};
// Texto de prueba, fuera del catalogo: el test de guarda solo mira `src/`.
use tc2::i18n::Mensaje;

/// Escribe `ruta` si no existe (idempotente).
pub struct Anotar {
    pub ruta: String,
    pub red: bool,
}

impl Anotar {
    pub fn nueva(ruta: &str, red: bool) -> Self {
        Self {
            ruta: ruta.to_owned(),
            red,
        }
    }
}

impl Operacion for Anotar {
    fn nombre(&self) -> &str {
        "anotar"
    }
    fn anotacion(&self) -> String {
        format!(
            "anotar|{}|{}",
            self.ruta,
            if self.red { "red" } else { "local" }
        )
    }
    fn exige_red(&self) -> bool {
        self.red
    }
    fn aplicar(&self, e: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        if e.leer(&self.ruta)?.is_some() {
            return Ok(Vec::new());
        }
        Ok(vec![Cambio::nuevo(
            self.ruta.clone(),
            format!("{}\n", self.ruta),
        )])
    }
}

/// Crea `ruta` y se rechaza si ya existe (como una toma: solo uno gana).
pub struct CrearUnico {
    pub ruta: String,
}

impl Operacion for CrearUnico {
    fn nombre(&self) -> &str {
        "crear-unico"
    }
    fn anotacion(&self) -> String {
        format!("crear-unico|{}", self.ruta)
    }
    fn exige_red(&self) -> bool {
        false
    }
    fn aplicar(&self, e: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        if e.leer(&self.ruta)?.is_some() {
            return Err(Fallo::rechazada(
                "ya-existe",
                Mensaje::externo(format!("{} ya existe", self.ruta)),
            ));
        }
        Ok(vec![Cambio::nuevo(self.ruta.clone(), "unico\n")])
    }
}

/// Escribe `ruta`, pero da un error interno si el estado ya tiene `veneno`
/// (una pendiente que falla al re-ejecutarse).
pub struct FallaSi {
    pub ruta: String,
    pub veneno: String,
}

impl Operacion for FallaSi {
    fn nombre(&self) -> &str {
        "falla-si"
    }
    fn anotacion(&self) -> String {
        format!("falla-si|{}|{}", self.ruta, self.veneno)
    }
    fn exige_red(&self) -> bool {
        false
    }
    fn aplicar(&self, e: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        if e.leer(&self.veneno)?.is_some() {
            return Err(Fallo::Error(Mensaje::externo(format!(
                "{} envenena {}",
                self.veneno, self.ruta
            ))));
        }
        Ok(vec![Cambio::nuevo(
            self.ruta.clone(),
            "x
",
        )])
    }
}

/// Reconstruye las operaciones de prueba desde su anotacion.
pub fn catalogo(a: &str) -> Option<Box<dyn Operacion>> {
    let partes: Vec<&str> = a.split('|').collect();
    match partes.as_slice() {
        ["anotar", ruta, modo] => Some(Box::new(Anotar::nueva(ruta, *modo == "red"))),
        ["falla-si", ruta, veneno] => Some(Box::new(FallaSi {
            ruta: (*ruta).to_owned(),
            veneno: (*veneno).to_owned(),
        })),
        ["crear-unico", ruta] => Some(Box::new(CrearUnico {
            ruta: (*ruta).to_owned(),
        })),
        _ => None,
    }
}

/// Catalogo vacio: ninguna pendiente que re-ejecutar.
pub fn sin_catalogo(_: &str) -> Option<Box<dyn Operacion>> {
    None
}

/// Generador minimo de `ROADMAP.md`: una linea por fichero de `notas/`.
pub fn roadmap(e: &dyn Estado) -> Result<String, Fallo> {
    let mut s = String::from("# Roadmap de prueba\n");
    for r in e.rutas().iter().filter(|r| r.starts_with("notas/")) {
        s.push_str("- ");
        s.push_str(r);
        s.push('\n');
    }
    Ok(s)
}
