//! Situacion del trabajo sobre un estado (F3-3 y F3-4): las tareas leidas,
//! el orden de trabajo, las listas para empezar y las en marcha. La usan
//! `next` y `roadmap`, que coinciden por construccion (TASK-031 C4).
//!
//! Lee todos los `tarea.md` y el `ROADMAP.md` de la rama de estado con un
//! solo [`Estado::leer_varios`] (un `git cat-file --batch` sobre una
//! [`crate::estado::Vista`]) y calcula el orden con [`grafo`].

use crate::estado::{Estado, Fallo, es_tarea_md};
use crate::grafo::{self, Hitos, Nodo};
use crate::identidad::convertir_heredado;
use crate::plataforma::Personas;
use crate::roadmap::zona::{self, Rota};
use crate::tarea::{Asignado, Documento};

pub use crate::estado::ROADMAP;

/// Lo que se muestra de una tarea legible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ficha {
    /// Lo que cuenta para el orden.
    pub nodo: Nodo,
    /// Titulo.
    pub titulo: String,
    /// Quien la lleva (en formato 1, siempre texto heredado).
    pub asignado: Option<Asignado>,
    /// Rama de la tarea, si la tiene (la que casa con su PR, F3-5).
    pub rama: Option<String>,
    /// Por que se descarto (formato 2; el formato 1 no lo lleva).
    pub motivo: Option<String>,
    /// En pausa: la ultima fila `pausa`/`reanudar` de sus transiciones es
    /// `pausa` ([`crate::tarea::Cuerpo::en_pausa`], TASK-034).
    pub en_pausa: bool,
    /// Instante (segundos desde 1970) de la ultima fila de sus transiciones
    /// ([`crate::tarea::Cuerpo::ultima_fecha`], F3-10).
    pub ultima_transicion: Option<u64>,
}

/// Segundos de un dia.
const DIA: u64 = 86_400;

/// Dias completos sin actividad desde `ultima` hasta `ahora`. `ultima` es la
/// actividad ya filtrada por [`crate::ciclo::liberar::actividad_conocida`]
/// (F3-10, D2a): solo puede ir por delante de `ahora` dentro de su
/// `MARGEN_FUTURO`, y entonces la resta satura y da 0.
pub fn dias_sin_actividad(ultima: u64, ahora: u64) -> u64 {
    ahora.saturating_sub(ultima) / DIA
}

impl Ficha {
    /// Ficha de un `tarea.md` leido.
    pub fn de(doc: &Documento) -> Self {
        let (titulo, asignado, rama, motivo, cuerpo) = match doc {
            Documento::Formato1(t, c) => (
                t.titulo.clone(),
                t.asignado_a.clone(),
                Some(t.rama.clone()),
                None,
                c,
            ),
            Documento::Formato2(t, c) => (
                t.titulo.clone(),
                t.asignado_a.clone(),
                t.rama.clone(),
                t.motivo.clone(),
                c,
            ),
        };
        Self {
            nodo: Nodo::de(doc),
            titulo,
            asignado,
            rama: rama.filter(|r| !r.trim().is_empty()),
            motivo,
            en_pausa: cuerpo.en_pausa(),
            ultima_transicion: cuerpo.ultima_fecha(),
        }
    }
}

/// Quien lleva una tarea, como lo dicen `roadmap`, `board` y `status`
/// (TASK-032 C3a): un solo calculo para las tres vistas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Quien {
    /// Nadie.
    Nadie,
    /// Una cuenta de la plataforma, por su login.
    Persona(String),
    /// Texto heredado de la 0.16 que no casa con ninguna persona de la lista.
    Heredado(String),
}

impl Quien {
    /// El texto de la vista: `@login`, el heredado tal cual o `—`.
    pub fn texto(&self) -> String {
        match self {
            Self::Nadie => "—".to_owned(),
            Self::Persona(l) => format!("@{l}"),
            Self::Heredado(h) => h.clone(),
        }
    }
}

/// Quien lleva la tarea de `a`, con los heredados convertidos por la lista
/// de personas de la base (D6 de TASK-031).
pub fn quien(a: Option<&Asignado>, personas: &Personas) -> Quien {
    let Some(a) = a else {
        return Quien::Nadie;
    };
    let a = convertir_heredado(a.clone(), personas);
    match (a.login(), a.heredado()) {
        (Some(l), _) => Quien::Persona(l.to_owned()),
        (None, Some(h)) => Quien::Heredado(h.to_owned()),
        (None, None) => Quien::Nadie,
    }
}

/// Todo lo que se calcula sobre un estado.
#[derive(Debug, Clone, Default)]
pub struct Situacion {
    /// Las tareas legibles, en orden de ruta.
    pub fichas: Vec<Ficha>,
    /// Los nodos del orden: los de las fichas y, detras, los ilegibles.
    pub nodos: Vec<Nodo>,
    /// Carpetas de los `tarea.md` que no se entienden.
    pub ilegibles: Vec<String>,
    /// Orden de los hitos (zona manual del ROADMAP, lectura tolerante).
    pub hitos: Hitos,
    /// Orden de trabajo, atascadas y avisos.
    pub orden: grafo::Orden,
    /// Listas para empezar, en el orden de trabajo (sin las en pausa).
    pub listas: Vec<String>,
    /// En diseno, en curso o en revision, en el orden de trabajo.
    pub en_marcha: Vec<String>,
    /// El `ROADMAP.md` de la rama de estado, si lo hay.
    pub roadmap: Option<String>,
}

impl Situacion {
    /// Si la zona manual del ROADMAP esta rota (D3 de TASK-031).
    pub fn roadmap_roto(&self) -> Option<Rota> {
        self.roadmap.as_deref().and_then(|t| zona::extraer(t).err())
    }
}

/// La situacion de `estado`.
pub fn calcular(estado: &dyn Estado) -> Result<Situacion, Fallo> {
    let mut rutas: Vec<String> = estado
        .rutas()
        .into_iter()
        .filter(|r| es_tarea_md(r))
        .collect();
    rutas.push(ROADMAP.to_owned());
    let mut textos = estado.leer_varios(&rutas)?;
    let roadmap = textos.pop().flatten();
    rutas.pop();
    let mut fichas = Vec::new();
    let mut ilegibles = Vec::new();
    for (ruta, texto) in rutas.iter().zip(textos) {
        let Some(texto) = texto else { continue };
        match crate::tarea::leer(&texto) {
            Ok(d) => fichas.push(Ficha::de(&d)),
            Err(_) => ilegibles.push(ruta.split('/').next().unwrap_or(ruta).to_owned()),
        }
    }
    let mut nodos: Vec<Nodo> = fichas.iter().map(|f| f.nodo.clone()).collect();
    // Detras de las legibles: si un ID se repite, vale la legible. Una
    // ilegible bloquea a quien dependa de ella.
    nodos.extend(ilegibles.iter().map(|id| Nodo::ilegible(id)));
    // Sin ROADMAP o sin zona, la de por defecto: es la que llevara el que se
    // genere, y asi el primero ya esta al dia (`--comparar`).
    let hitos = match roadmap.as_deref() {
        Some(t) if zona::extraer(t) != Ok(None) => Hitos::del_roadmap(t),
        _ => Hitos::del_roadmap(zona::POR_DEFECTO),
    };
    let orden = grafo::orden_de_trabajo(&nodos, &hitos);
    // Una en pausa no se ofrece para empezar (TASK-034 C9); sigue en el
    // orden de trabajo. Con un ID repetido vale su primera ficha, como en el
    // ROADMAP.
    let mut listas = grafo::listas(&orden, &nodos);
    listas.retain(|id| {
        !fichas
            .iter()
            .find(|f| f.nodo.id == *id)
            .is_some_and(|f| f.en_pausa)
    });
    Ok(Situacion {
        listas,
        en_marcha: grafo::en_marcha(&orden, &nodos),
        fichas,
        nodos,
        ilegibles,
        hitos,
        orden,
        roadmap,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tarea::{Cuerpo, Tarea};

    fn ficha(cuerpo: &str) -> Ficha {
        let t = Tarea::nueva("TASK-001", "Uno", "2026-11-01");
        Ficha::de(&Documento::Formato2(t, Cuerpo::leer(cuerpo)))
    }

    /// TASK-081 (D2a): la ultima fila cuenta tambien si es `pausa`. Los dias
    /// son completos; la regla de la actividad (y su margen para un reloj
    /// adelantado) vive en `ciclo::liberar::actividad_conocida` (TASK-125).
    #[test]
    fn dias_sin_actividad_completos() {
        let f = ficha(
            "## Transiciones\n| fecha | fase |\n|---|---|\n| 2026-11-17T10:20:30Z | pausa |\n",
        );
        let fila = 1_794_910_830;
        assert_eq!(f.ultima_transicion, Some(fila));
        assert_eq!(dias_sin_actividad(fila, fila + 7 * DIA - 1), 6);
        assert_eq!(dias_sin_actividad(fila, fila + 7 * DIA), 7);
        assert_eq!(dias_sin_actividad(fila + 100, fila), 0);
    }
}
