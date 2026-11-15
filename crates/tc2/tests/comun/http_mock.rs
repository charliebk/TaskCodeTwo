//! Servidor HTTP simulado en `127.0.0.1` para la bateria del puerto
//! Plataforma (F1-13, PoC M): sirve respuestas fijas por `METODO ruta` y
//! registra cada peticion. Solo la biblioteca estandar.

use std::collections::{BTreeMap, VecDeque};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

/// Una respuesta simulada.
#[derive(Debug, Clone)]
pub struct Simulada {
    pub estado: u16,
    pub cuerpo: String,
    pub cabeceras: Vec<(String, String)>,
}

impl Simulada {
    pub fn json(estado: u16, cuerpo: &str) -> Self {
        Self {
            estado,
            cuerpo: cuerpo.to_owned(),
            cabeceras: Vec::new(),
        }
    }

    pub fn con(mut self, k: &str, v: &str) -> Self {
        self.cabeceras.push((k.to_owned(), v.to_owned()));
        self
    }
}

/// Peticion recibida.
#[derive(Debug, Clone)]
pub struct Recibida {
    /// `METODO ruta?consulta`.
    pub linea: String,
    pub autorizacion: Option<String>,
    pub cuerpo: String,
    /// Cabeceras, con el nombre en minusculas (sin la autorizacion).
    pub cabeceras: Vec<(String, String)>,
}

type Rutas = BTreeMap<String, VecDeque<Simulada>>;

pub struct Mock {
    pub puerto: u16,
    rutas: Arc<Mutex<Rutas>>,
    recibidas: Arc<Mutex<Vec<Recibida>>>,
}

impl Mock {
    /// Arranca el servidor en un puerto libre de 127.0.0.1.
    pub fn nuevo() -> Self {
        let l = TcpListener::bind("127.0.0.1:0").expect("bind");
        let puerto = l.local_addr().expect("addr").port();
        let rutas: Arc<Mutex<Rutas>> = Arc::default();
        let recibidas: Arc<Mutex<Vec<Recibida>>> = Arc::default();
        let (r, v) = (rutas.clone(), recibidas.clone());
        thread::spawn(move || {
            for s in l.incoming().flatten() {
                atender(s, &r, &v);
            }
        });
        Self {
            puerto,
            rutas,
            recibidas,
        }
    }

    pub fn base(&self) -> String {
        format!("http://127.0.0.1:{}", self.puerto)
    }

    /// Anade una respuesta para `METODO ruta` (sin consulta, o con ella si se
    /// quiere distinguir). Varias para la misma clave se sirven en orden y la
    /// ultima se repite.
    pub fn responder(&self, metodo: &str, ruta: &str, r: Simulada) {
        self.rutas
            .lock()
            .expect("lock")
            .entry(format!("{metodo} {ruta}"))
            .or_default()
            .push_back(r);
    }

    pub fn recibidas(&self) -> Vec<Recibida> {
        self.recibidas.lock().expect("lock").clone()
    }
}

fn atender(s: TcpStream, rutas: &Mutex<Rutas>, recibidas: &Mutex<Vec<Recibida>>) {
    let mut lector = BufReader::new(match s.try_clone() {
        Ok(c) => c,
        Err(_) => return,
    });
    let mut primera = String::new();
    if lector.read_line(&mut primera).is_err() {
        return;
    }
    let mut partes = primera.split_whitespace();
    let metodo = partes.next().unwrap_or_default().to_owned();
    let ruta = partes.next().unwrap_or_default().to_owned();
    let mut largo = 0usize;
    let mut autorizacion = None;
    let mut cabeceras = Vec::new();
    loop {
        let mut l = String::new();
        if lector.read_line(&mut l).is_err() || l.trim().is_empty() {
            break;
        }
        if let Some((k, v)) = l.split_once(':') {
            let k = k.trim().to_ascii_lowercase();
            if k == "content-length" {
                largo = v.trim().parse().unwrap_or(0);
            } else if k == "authorization" {
                autorizacion = Some(v.trim().to_owned());
                continue;
            }
            cabeceras.push((k, v.trim().to_owned()));
        }
    }
    let mut cuerpo = vec![0u8; largo];
    let _ = lector.read_exact(&mut cuerpo);
    recibidas.lock().expect("lock").push(Recibida {
        linea: format!("{metodo} {ruta}"),
        autorizacion,
        cuerpo: String::from_utf8_lossy(&cuerpo).into_owned(),
        cabeceras,
    });
    let sin_consulta = ruta.split('?').next().unwrap_or_default().to_owned();
    let respuesta = {
        let mut r = rutas.lock().expect("lock");
        let cola = if r.contains_key(&format!("{metodo} {ruta}")) {
            r.get_mut(&format!("{metodo} {ruta}"))
        } else {
            r.get_mut(&format!("{metodo} {sin_consulta}"))
        };
        match cola {
            Some(c) if c.len() > 1 => c.pop_front(),
            Some(c) => c.front().cloned(),
            None => None,
        }
    }
    .unwrap_or_else(|| Simulada::json(404, "{\"message\":\"Not Found\"}"));
    let mut cab = String::new();
    for (k, v) in &respuesta.cabeceras {
        cab.push_str(&format!("{k}: {v}\r\n"));
    }
    let texto = format!(
        "HTTP/1.1 {} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{cab}\r\n",
        respuesta.estado,
        respuesta.cuerpo.len()
    );
    let mut s = s;
    let _ = s.write_all(texto.as_bytes());
    let _ = s.write_all(respuesta.cuerpo.as_bytes());
    let _ = s.flush();
}
