//! Operaciones de prueba sobre la rama de estado.

use std::path::Path;
use std::time::Duration;

use tc2::contrato::Informe;
use tc2::estado::{self, Cambio, Contexto, Estado, Fallo, Operacion, Politica};
use tc2::git::Git;
// Texto de prueba, fuera del catalogo: el test de guarda solo mira `src/`.
use tc2::i18n::Mensaje;

/// Politica de reintentos sin esperas largas para los tests.
pub fn rapida() -> Politica {
    Politica {
        intentos: 3,
        base: Duration::from_millis(1),
        espera_cerrojo: Duration::from_secs(30),
    }
}

/// Escribe `op` por la transaccion con [`catalogo`] y [`rapida`], y
/// comprueba el contrato de su informe.
pub fn escribir_estado(g: &Git, dir: &Path, op: &dyn Operacion) -> Informe {
    let mut c = Contexto::nuevo(g, dir, &catalogo, None);
    c.politica = rapida();
    let i = estado::transaccion(&c, op);
    super::contrato(&i);
    i
}

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

/// Escribe en la rama de estado por la transaccion (como lo haria otra
/// fase): `cambiar` recibe el texto de `ruta` (si existe) y da el nuevo.
pub struct Escribir {
    pub ruta: String,
    pub cambiar: Box<dyn Fn(Option<String>) -> String>,
}

impl Operacion for Escribir {
    fn nombre(&self) -> &str {
        "escribir"
    }
    fn anotacion(&self) -> String {
        format!("escribir|{}", self.ruta)
    }
    fn exige_red(&self) -> bool {
        true
    }
    fn aplicar(&self, e: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        let t = e.leer(&self.ruta)?;
        Ok(vec![Cambio::nuevo(self.ruta.clone(), (self.cambiar)(t))])
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
