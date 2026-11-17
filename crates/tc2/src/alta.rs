//! Alta de tareas con reserva de ID (F2-4; ADR F1-2, punto 3, y F1-1).
//!
//! El ID se reserva con el push de la rama de estado: [`Alta`] calcula el
//! siguiente ID sobre el estado fresco de cada intento (maximo + 1, nunca el
//! recuento) y crea la carpeta fija `TASK-NNN/` con su `tarea.md`. Si otro
//! clon publico antes, el push se rechaza, la transaccion re-ejecuta la
//! operacion sobre lo nuevo y sale el numero siguiente. La carpeta publicada
//! es el compare-and-swap: reservar un numero sin crearla no reservaria nada.

use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::estado::{Cambio, Estado, Fallo, Operacion};
use crate::i18n::catalogo::estado as txt;
use crate::i18n::{Idioma, Mensaje};
use crate::tarea::texto::{Parecido, parecido, slug};
use crate::tarea::{Cuerpo, Documento, Tarea};

/// Nombre de la operacion y de la orden del binario.
pub const NOMBRE: &str = "new";

/// Prefijo de las carpetas de tarea en la rama de estado.
const PREFIJO: &str = "TASK-";

/// Una tarea por crear (RF-01): lo que pone quien la da de alta, con las
/// dependencias ya resueltas a `TASK-NNN`. La comparten `new` (un lote de
/// una) e `import` (F3-2): un solo camino para reservar el ID, derivar la
/// rama y avisar de los titulos parecidos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entrada {
    /// Titulo ([`titulo_valido`]).
    pub titulo: String,
    /// Tipo (`tarea::TIPOS`); decide el prefijo de la rama.
    pub tipo: String,
    /// Sprint (el hito, D2 de TASK-029).
    pub sprint: u32,
    /// Objetivo; vacio deja el de la plantilla.
    pub objetivo: String,
    /// Criterios de aceptacion; sin ninguno queda el de la plantilla.
    pub criterios: Vec<String>,
    /// Etiquetas.
    pub etiquetas: Vec<String>,
    /// Complejidad (`tarea::COMPLEJIDADES`).
    pub complejidad: Option<String>,
    /// Dependencias, `TASK-NNN`.
    pub dependencias: Vec<String>,
    /// `lote/clave` si llega de un import (D1 de TASK-029).
    pub lote: Option<String>,
}

impl Entrada {
    /// Entrada con solo el titulo y los valores de la plantilla de F1-9.
    pub fn nueva(titulo: &str) -> Self {
        Self {
            titulo: titulo.to_owned(),
            tipo: TIPO.to_owned(),
            sprint: 0,
            objetivo: String::new(),
            criterios: Vec::new(),
            etiquetas: Vec::new(),
            complejidad: None,
            dependencias: Vec::new(),
            lote: None,
        }
    }

    /// Si el enunciado esta listo (RF-01). Sin objetivo ni criterios es el
    /// borrador de la plantilla, que `new` admite como hasta F2-4 salvo que
    /// `exige_criterios` (el import, como la 0.16, pide al menos uno). Con
    /// algo escrito, no esta listo si no hay criterios, si uno esta vacio o
    /// repetido, si queda un hueco de la plantilla (`<...>`) o si hay
    /// caracteres de control (el objetivo admite saltos de linea).
    pub fn listo(&self, exige_criterios: bool) -> Result<(), NoListo> {
        let objetivo = self.objetivo.trim();
        if objetivo.is_empty() && self.criterios.is_empty() && !exige_criterios {
            return Ok(());
        }
        if self.criterios.is_empty() {
            return Err(NoListo::SinCriterios);
        }
        let control = |s: &str, salto: bool| {
            s.chars().any(|c| {
                (c.is_control() && !(salto && c == '\n')) || matches!(c, '\u{2028}' | '\u{2029}')
            })
        };
        if control(objetivo, true) || self.criterios.iter().any(|c| control(c, false)) {
            return Err(NoListo::Control);
        }
        let hueco = |s: &str| s.starts_with('<') && s.ends_with('>');
        if hueco(objetivo) {
            return Err(NoListo::Plantilla);
        }
        let mut vistos = BTreeSet::new();
        for c in self.criterios.iter().map(|c| c.trim()) {
            if c.is_empty() {
                return Err(NoListo::CriterioVacio);
            }
            if hueco(c) {
                return Err(NoListo::Plantilla);
            }
            if !vistos.insert(crate::tarea::texto::plegar(c)) {
                return Err(NoListo::Repetido);
            }
        }
        Ok(())
    }
}

/// `motivo=` de una dependencia que no existe en el estado.
pub(crate) const REFERENCIA_INEXISTENTE: &str = "referencia-inexistente";

/// Una dependencia que no existe es un fallo de la llamada (el fichero o
/// los argumentos), no del estado: `uso` (2) en vez de `rechazada` (C2).
pub(crate) fn referencia_a_uso(i: &mut crate::contrato::Informe) {
    if i.resultado == crate::contrato::Resultado::Rechazada
        && i.valor("motivo") == Some(REFERENCIA_INEXISTENTE)
    {
        i.resultado = crate::contrato::Resultado::Uso;
    }
}

/// Un sprint: entero sin signo, sin espacios ni `+` (`--sprint`, `sprint:`).
pub(crate) fn sprint_de(v: &str) -> Option<u32> {
    v.bytes()
        .all(|b| b.is_ascii_digit())
        .then(|| v.parse().ok())
        .flatten()
}

/// `uso` de `new` con `motivo` y `diagnostico`.
fn uso(motivo: &str, diagnostico: impl Into<Mensaje>) -> crate::contrato::Informe {
    crate::contrato::Informe::nuevo(NOMBRE, crate::contrato::Resultado::Uso)
        .con("motivo", motivo)
        .diagnostico(diagnostico)
}

/// Lee los argumentos de `new` (sin `--json`): el titulo y los campos de
/// RF-01 (`--tipo`, `--objetivo`, `--criterio`..., `--etiqueta`...,
/// `--complejidad`, `--sprint`, `--dependencia`...). Rechaza con `uso` un
/// valor fuera de su lista y un enunciado que no esta listo
/// ([`Entrada::listo`]).
pub fn entrada_de_args(args: &[String]) -> Result<Entrada, crate::contrato::Informe> {
    use crate::i18n::catalogo::cli as tc;
    use crate::i18n::catalogo::importar as ti;
    let invalido = |opcion: &str, valor: &str, permitidos: &str| {
        uso(
            "valor-invalido",
            Mensaje::nuevo(&ti::VALOR_INVALIDO)
                .con("opcion", opcion)
                .con("valor", valor)
                .con("permitidos", permitidos),
        )
        .con("opcion", opcion.trim_start_matches('-'))
    };
    let mut titulos: Vec<&String> = Vec::new();
    let mut e = Entrada::nueva("");
    let mut vistas: BTreeSet<&str> = BTreeSet::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let opcion = a.as_str();
        if !opcion.starts_with("--") {
            titulos.push(a);
            continue;
        }
        let repetible = matches!(opcion, "--criterio" | "--etiqueta" | "--dependencia");
        let conocida = repetible
            || matches!(
                opcion,
                "--tipo" | "--objetivo" | "--complejidad" | "--sprint"
            );
        if !conocida || (!repetible && !vistas.insert(opcion)) {
            return Err(uso("argumentos-de-mas", &ti::USO_NEW_CAMPOS));
        }
        let Some(v) = it.next() else {
            return Err(uso("falta-valor", &ti::USO_NEW_CAMPOS).con("opcion", &opcion[2..]));
        };
        match opcion {
            "--tipo" if crate::tarea::TIPOS.contains(&v.as_str()) => v.clone_into(&mut e.tipo),
            "--tipo" => return Err(invalido(opcion, v, &crate::tarea::TIPOS.join(", "))),
            "--complejidad" if crate::tarea::COMPLEJIDADES.contains(&v.as_str()) => {
                e.complejidad = Some(v.clone());
            }
            "--complejidad" => {
                return Err(invalido(opcion, v, &crate::tarea::COMPLEJIDADES.join(", ")));
            }
            "--sprint" => {
                e.sprint = sprint_de(v).ok_or_else(|| invalido(opcion, v, "0, 1, 2..."))?;
            }
            // Un argumento con CRLF (Windows) cuenta como saltos de linea.
            "--objetivo" => e.objetivo = v.replace("\r\n", "\n"),
            "--criterio" => e.criterios.push(v.clone()),
            "--etiqueta" if crate::tarea::login_valido(v) => e.etiquetas.push(v.clone()),
            "--etiqueta" => {
                return Err(uso(
                    "valor-invalido",
                    Mensaje::nuevo(&ti::ETIQUETA_INVALIDA).con("valor", v.as_str()),
                )
                .con("opcion", "etiqueta"));
            }
            _ if crate::tarea::id_valido(v) => {
                if !e.dependencias.contains(v) {
                    e.dependencias.push(v.clone());
                }
            }
            _ => return Err(invalido(opcion, v, "TASK-NNN")),
        }
    }
    let titulo = match titulos.as_slice() {
        [t] if titulo_valido(t) => (*t).clone(),
        [_] => return Err(uso("titulo-invalido", &tc::TITULO_INVALIDO)),
        [] => return Err(uso("sin-titulo", &tc::USO_NEW)),
        _ => return Err(uso("argumentos-de-mas", &tc::TITULO_ENTRE_COMILLAS)),
    };
    e.titulo = titulo;
    if let Err(c) = e.listo(false) {
        return Err(uso(
            "enunciado-no-listo",
            Mensaje::nuevo(&ti::NO_LISTO)
                .con("titulo", e.titulo.as_str())
                .con_mensaje("causa", c.mensaje()),
        )
        .con("causa", c.causa()));
    }
    Ok(e)
}

/// Por que un enunciado no esta listo (RF-01).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoListo {
    /// Hay enunciado pero ningun criterio.
    SinCriterios,
    /// Un criterio vacio.
    CriterioVacio,
    /// Queda un hueco de la plantilla.
    Plantilla,
    /// Un criterio repetido.
    Repetido,
    /// Caracteres de control.
    Control,
}

impl NoListo {
    /// Valor de `causa=` en la linea de contrato.
    pub fn causa(self) -> &'static str {
        match self {
            Self::SinCriterios => "sin-criterios",
            Self::CriterioVacio => "criterio-vacio",
            Self::Plantilla => "plantilla",
            Self::Repetido => "criterio-repetido",
            Self::Control => "caracter-de-control",
        }
    }

    /// La causa explicada, para el diagnostico.
    pub fn mensaje(self) -> Mensaje {
        use crate::i18n::catalogo::importar as t;
        Mensaje::nuevo(match self {
            Self::SinCriterios => &t::CAUSA_SIN_CRITERIOS,
            Self::CriterioVacio => &t::CAUSA_CRITERIO_VACIO,
            Self::Plantilla => &t::CAUSA_PLANTILLA,
            Self::Repetido => &t::CAUSA_REPETIDO,
            Self::Control => &t::CAUSA_CONTROL,
        })
    }
}

/// Alta de una tarea: reserva el siguiente ID y escribe su `tarea.md`.
#[derive(Debug)]
pub struct Alta {
    entrada: Entrada,
    fecha: String,
    ocupados: BTreeSet<u32>,
    id: Cell<Option<u32>>,
    parecidas: RefCell<Vec<Parecida>>,
}

impl Alta {
    /// Alta con `titulo` y la fecha de hoy (UTC).
    pub fn nueva(titulo: &str) -> Self {
        Self::con_fecha(titulo, &hoy())
    }

    /// Alta con una fecha `AAAA-MM-DD` dada.
    pub fn con_fecha(titulo: &str, fecha: &str) -> Self {
        Self::de_entrada(Entrada::nueva(titulo), fecha)
    }

    /// Alta de `entrada` con la fecha de hoy (UTC).
    pub fn con_entrada(entrada: Entrada) -> Self {
        Self::de_entrada(entrada, &hoy())
    }

    fn de_entrada(entrada: Entrada, fecha: &str) -> Self {
        Self {
            entrada,
            fecha: fecha.to_owned(),
            ocupados: BTreeSet::new(),
            id: Cell::new(None),
            parecidas: RefCell::new(Vec::new()),
        }
    }

    /// IDs que ya tienen rama de tarea (`git::ids_con_rama`, leidos una vez
    /// antes de la transaccion): el ID nuevo queda por encima (F2-4b).
    #[must_use]
    pub fn con_ocupados(mut self, ocupados: BTreeSet<u32>) -> Self {
        self.ocupados = ocupados;
        self
    }

    /// Rama derivada del ID del ultimo intento (F2-4b).
    pub fn rama(&self) -> Option<String> {
        self.id()
            .map(|id| rama_de(&id, &self.entrada.tipo, &self.entrada.titulo))
    }

    /// ID calculado en el ultimo intento (el que publico, si la transaccion
    /// salio bien).
    pub fn id(&self) -> Option<String> {
        self.id.get().map(formato_id)
    }

    /// Tareas abiertas parecidas vistas en el ultimo intento.
    pub fn parecidas(&self) -> Vec<Parecida> {
        self.parecidas.borrow().clone()
    }
}

impl Operacion for Alta {
    fn nombre(&self) -> &str {
        NOMBRE
    }
    fn anotacion(&self) -> String {
        format!("{NOMBRE}|{}", self.entrada.titulo)
    }
    fn exige_red(&self) -> bool {
        // Reserva algo compartido: sin red falla, nunca queda pendiente (D2b).
        true
    }
    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        let rutas = estado.rutas();
        // Las dependencias tienen que existir en el estado fresco (RF-03).
        if let Some(d) = self
            .entrada
            .dependencias
            .iter()
            .find(|d| !rutas.iter().any(|r| *r == format!("{d}/tarea.md")))
        {
            return Err(Fallo::rechazada(
                REFERENCIA_INEXISTENTE,
                Mensaje::nuevo(&crate::i18n::catalogo::importar::DEPENDENCIA_INEXISTENTE)
                    .con("referencia", d.as_str()),
            ));
        }
        let ids = reservar(&rutas, &self.ocupados, 1)?;
        self.id.set(ids.first().copied());
        // Sobre el mismo estado fresco del intento: dos altas a la vez se ven
        // (la segunda en publicar avisa de la primera, TASK-076 C1).
        let tareas = tareas(estado, &rutas)?;
        let a = altas(
            &tareas,
            &ids,
            std::slice::from_ref(&self.entrada),
            &self.fecha,
        )?;
        *self.parecidas.borrow_mut() = a.parecidas;
        Ok(a.cambios)
    }
}

/// Lo que sale de [`altas`]: un `tarea.md` por entrada y las tareas abiertas
/// que se le parecen.
#[derive(Debug)]
pub(crate) struct Altas {
    /// Un cambio por entrada, en su orden.
    pub cambios: Vec<Cambio>,
    /// Tareas abiertas parecidas a alguna entrada, sin repetir, por ID.
    pub parecidas: Vec<Parecida>,
}

/// Nucleo puro del alta (F2-4 y F3-2): `entradas[i]` nace con el ID
/// `ids[i]`. Comprueba la rama derivada de cada una y busca las tareas
/// abiertas de `tareas` con un titulo parecido.
pub(crate) fn altas(
    tareas: &[Tarea],
    ids: &[u32],
    entradas: &[Entrada],
    fecha: &str,
) -> Result<Altas, Fallo> {
    let mut cambios = Vec::with_capacity(entradas.len());
    let mut vistas = BTreeSet::new();
    let mut parecidas = Vec::new();
    for (i, e) in entradas.iter().enumerate() {
        let id = formato_id(
            ids.get(i)
                .copied()
                .ok_or_else(|| Fallo::Error((&crate::i18n::catalogo::estado::SIN_IDS).into()))?,
        );
        comprobar_rama(&id, &e.tipo, &e.titulo)?;
        cambios.push(Cambio::nuevo(
            format!("{id}/tarea.md"),
            tarea_de(&id, e, fecha),
        ));
        for p in parecidas_a(tareas, &e.titulo) {
            if vistas.insert(p.id.clone()) {
                parecidas.push(p);
            }
        }
    }
    parecidas.sort_by(|a: &Parecida, b| a.id.cmp(&b.id));
    Ok(Altas { cambios, parecidas })
}

/// `n` IDs consecutivos desde [`siguiente_id`]: los de un lote no dejan
/// huecos entre si (ADR F1-2, punto 3).
pub(crate) fn reservar(
    rutas: &[String],
    ocupados: &BTreeSet<u32>,
    n: usize,
) -> Result<Vec<u32>, Fallo> {
    let primero = siguiente_id(rutas, ocupados)?;
    (0..n)
        .map(|i| {
            u32::try_from(i)
                .ok()
                .and_then(|i| primero.checked_add(i))
                .ok_or_else(|| Fallo::Error((&crate::i18n::catalogo::estado::SIN_IDS).into()))
        })
        .collect()
}

/// Estados en los que una tarea esta abierta: contra ellas se avisa.
const ABIERTOS: [&str; 4] = ["planificada", "en-diseno", "en-curso", "en-revision"];

/// Una tarea abierta con un titulo igual o casi igual al del alta (RR-12).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parecida {
    /// ID de la tarea parecida.
    pub id: String,
    /// Su titulo.
    pub titulo: String,
    /// Como se parece.
    pub parecido: Parecido,
}

/// `TASK-<cifras>/tarea.md`.
fn es_tarea_md(ruta: &str) -> bool {
    numero(ruta).is_some() && ruta.split_once('/').is_some_and(|(_, r)| r == "tarea.md")
}

/// Las tareas de formato 2 del estado, en orden de ruta, leidas en un solo
/// proceso (F2-4b) con [`crate::estado::tareas`]. Los `tarea.md` que no se
/// leen o son de formato 1 se saltan sin fallar, y las carpetas con sufijo
/// no cuentan aqui (filtro de siempre del alta).
pub(crate) fn tareas(estado: &dyn Estado, rutas: &[String]) -> Result<Vec<Tarea>, Fallo> {
    let mds: Vec<String> = rutas.iter().filter(|r| es_tarea_md(r)).cloned().collect();
    Ok(crate::estado::tareas(estado, &mds)?
        .into_iter()
        .filter_map(|(_, d)| match d {
            Ok(Documento::Formato2(t, _)) => Some(t),
            _ => None,
        })
        .collect())
}

/// Tareas abiertas de `tareas` cuyo titulo se parece a `titulo`.
fn parecidas_a(tareas: &[Tarea], titulo: &str) -> Vec<Parecida> {
    tareas
        .iter()
        .filter(|t| ABIERTOS.contains(&t.estado.as_str()))
        .filter_map(|t| {
            parecido(titulo, &t.titulo).map(|p| Parecida {
                id: t.id.clone(),
                titulo: t.titulo.clone(),
                parecido: p,
            })
        })
        .collect()
}

/// Una linea de aviso con las tareas parecidas, en `idioma`; `None` si no
/// hay. Va a stderr aunque el alta salga bien: es solo un aviso (D1).
pub fn aviso_parecidas(p: &[Parecida], idioma: Idioma) -> Option<String> {
    if p.is_empty() {
        return None;
    }
    let lista: Vec<String> = p
        .iter()
        .map(|x| {
            let m = match x.parecido {
                Parecido::Duplicado => Mensaje::nuevo(&txt::PARECIDA_DUPLICADO),
                Parecido::Casi(c) => Mensaje::nuevo(&txt::PARECIDA_CASI)
                    .con("unidad", (c / 100).to_string())
                    .con("centesimas", format!("{:02}", c % 100)),
            };
            m.con("id", x.id.as_str())
                .con("titulo", x.titulo.as_str())
                .texto(idioma)
        })
        .collect();
    Some(
        Mensaje::nuevo(&txt::AVISO_PARECIDAS)
            .con("lista", lista.join("; "))
            .texto(idioma),
    )
}

/// Reconstruye un [`Alta`] desde su anotacion (para el catalogo).
pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
    let titulo = a.strip_prefix(NOMBRE)?.strip_prefix('|')?;
    titulo_valido(titulo).then(|| Box::new(Alta::nueva(titulo)) as Box<dyn Operacion>)
}

pub use crate::tarea::titulo_valido;

/// `TASK-NNN` con al menos tres cifras (`TASK-1000` sin renumerar).
pub fn formato_id(n: u32) -> String {
    format!("{PREFIJO}{n:03}")
}

/// El numero de la carpeta de tarea del primer componente de `ruta`. El
/// prefijo se compara sin mayusculas, como compara las rutas la transaccion:
/// una `task-013/` puesta a mano tambien ocupa el 13.
fn numero(ruta: &str) -> Option<u32> {
    let primero = ruta.split('/').next()?;
    let prefijo = primero.get(..PREFIJO.len())?;
    if !prefijo.eq_ignore_ascii_case(PREFIJO) {
        return None;
    }
    let cifras = primero.get(PREFIJO.len()..)?;
    if cifras.is_empty() || !cifras.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    cifras.parse().ok()
}

/// Siguiente ID: el maximo de las carpetas `TASK-<cifras>` y de los IDs con
/// rama (`ocupados`) mas uno (cuentan terminadas y descartadas, que no se
/// borran; los huecos no se rellenan). Adenda de F2-4b al ADR F1-2 punto 3.
pub fn siguiente_id(rutas: &[String], ocupados: &BTreeSet<u32>) -> Result<u32, Fallo> {
    let maximo = rutas
        .iter()
        .filter_map(|r| numero(r))
        .chain(ocupados.iter().copied())
        .max()
        .unwrap_or(0);
    maximo
        .checked_add(1)
        .ok_or_else(|| Fallo::Error((&crate::i18n::catalogo::estado::SIN_IDS).into()))
}

/// Tipo con el que nace una tarea (plantilla de F1-9, punto 6).
const TIPO: &str = "feature";

/// Rama de la tarea `id` (ADR F1-3 y F1-9 punto 6): `<tipo>/task-nnn-<slug>`
/// con el slug del titulo (F2-4b).
pub fn rama_de(id: &str, tipo: &str, titulo: &str) -> String {
    crate::git::nombre_de_rama(id, tipo, &slug(titulo))
}

/// La rama derivada pasa la validacion de las operaciones de rama: si no,
/// `start` no podria crearla.
fn comprobar_rama(id: &str, tipo: &str, titulo: &str) -> Result<(), Fallo> {
    crate::git::rama::rama_valida(NOMBRE, id, tipo, &slug(titulo))
        .map(drop)
        .map_err(|i| {
            Fallo::Error(
                crate::i18n::Mensaje::nuevo(&crate::i18n::catalogo::estado::RAMA_DERIVADA_INVALIDA)
                    .con("id", id)
                    .con("motivo", i.valor("motivo").unwrap_or_default()),
            )
        })
}

/// `tarea.md` de una tarea recien creada con solo el titulo: el modelo de
/// F2-5 con la plantilla de F1-9 (punto 6) y la rama derivada del ID (F2-4b).
pub fn tarea_md(id: &str, titulo: &str, fecha: &str) -> String {
    tarea_de(id, &Entrada::nueva(titulo), fecha)
}

/// `tarea.md` de `entrada` con el ID `id`: el enunciado en el cuerpo y el
/// resto en el frontmatter. El tipo y la rama salen del mismo campo: no
/// pueden divergir.
fn tarea_de(id: &str, e: &Entrada, fecha: &str) -> String {
    let mut t = Tarea::nueva(id, &e.titulo, fecha);
    e.tipo.clone_into(&mut t.tipo);
    t.sprint = e.sprint;
    t.etiquetas.clone_from(&e.etiquetas);
    t.complejidad.clone_from(&e.complejidad);
    t.dependencias.clone_from(&e.dependencias);
    t.lote.clone_from(&e.lote);
    t.rama = Some(rama_de(id, &e.tipo, &e.titulo));
    t.escribir(&Cuerpo::con_enunciado(&e.objetivo, &e.criterios))
}

/// Fecha de hoy en UTC, `AAAA-MM-DD`, sin dependencias.
pub(crate) fn hoy() -> String {
    let segundos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    fecha_de_dias(i64::try_from(segundos / 86_400).unwrap_or(0))
}

/// Fecha civil de un numero de dias desde 1970-01-01 (algoritmo de Howard
/// Hinnant, `civil_from_days`).
pub(crate) fn fecha_de_dias(dias: i64) -> String {
    let z = dias + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let a = yoe + era * 400 + i64::from(m <= 2);
    format!("{a:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rutas(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn siguiente_id_es_el_maximo_mas_uno() {
        // TASK-007 es una descartada: cuenta igual; el hueco no se rellena.
        let r = rutas(&[
            "LEEME.md",
            "ROADMAP.md",
            "TASK-001/tarea.md",
            "TASK-007/tarea.md",
            "TASK-007/planificacion/plan-final.md",
            "notas/TASK-050.md",
            "TASK-1a/tarea.md",
            "TASK-/tarea.md",
            "TASK-+9/tarea.md",
        ]);
        let sin = BTreeSet::new();
        assert_eq!(siguiente_id(&r, &sin), Ok(8));
        // Una carpeta en minusculas puesta a mano tambien ocupa su numero.
        assert_eq!(
            siguiente_id(&rutas(&["task-013/tarea.md", "TASK-002/tarea.md"]), &sin),
            Ok(14)
        );
        assert_eq!(siguiente_id(&rutas(&["ñ/x", "TAS"]), &sin), Ok(1));
        assert_eq!(siguiente_id(&[], &sin), Ok(1));
        assert_eq!(siguiente_id(&rutas(&["TASK-999/tarea.md"]), &sin), Ok(1000));
        assert_eq!(formato_id(1000), "TASK-1000");
        assert_eq!(formato_id(8), "TASK-008");
        let tope = rutas(&[&format!("TASK-{}/tarea.md", u32::MAX)]);
        assert!(matches!(siguiente_id(&tope, &sin), Err(Fallo::Error(_))));
        // F2-4b: una rama sin carpeta tambien ocupa su numero (adenda F1-2).
        assert_eq!(siguiente_id(&r, &BTreeSet::from([5, 12])), Ok(13));
        assert_eq!(siguiente_id(&r, &BTreeSet::from([3])), Ok(8));
        assert!(matches!(
            siguiente_id(&[], &BTreeSet::from([u32::MAX])),
            Err(Fallo::Error(_))
        ));
    }

    #[test]
    fn plantilla_escapa_el_titulo_y_sigue_f1_9() {
        let t = tarea_md("TASK-008", "C:\\ruta\\ \"x\" {fecha}", "2026-11-15");
        assert!(
            t.contains("titulo: \"C:\\\\ruta\\\\ \\\"x\\\" {fecha}\"\n"),
            "{t}"
        );
        let claves: Vec<&str> = t
            .lines()
            .skip(1)
            .take_while(|l| *l != "---")
            .filter(|l| !l.starts_with(' '))
            .filter_map(|l| l.split(':').next())
            .collect();
        assert_eq!(
            claves,
            [
                "formato",
                "id",
                "titulo",
                "tipo",
                "sprint",
                "etiquetas",
                "complejidad",
                "carril",
                "estado",
                "motivo",
                "plan_aprobado",
                "rama",
                "asignado_a",
                "roles",
                "estimacion",
                "tokens_diseno",
                "tokens_implementacion",
                "tokens_revision",
                "creado",
                "actualizado",
                "dependencias",
            ]
        );
        // Adenda de TASK-029 a F1-9: una tarea importada lleva ademas `lote`,
        // la ultima clave conocida; sin lote no se escribe.
        let mut e = Entrada::nueva("x");
        e.lote = Some("hito37/F3-2".to_owned());
        let importada = tarea_de("TASK-009", &e, "2026-11-17");
        let ultimas: Vec<&str> = importada
            .lines()
            .skip(1)
            .take_while(|l| *l != "---")
            .filter_map(|l| l.split(':').next())
            .collect();
        assert_eq!(ultimas.len(), claves.len() + 4); // + las 3 de `roles` y `lote`
        assert_eq!(ultimas.last(), Some(&"lote"));
        assert!(importada.contains("\ndependencias: []\nlote: \"hito37/F3-2\"\n---\n"));
        let leida = match crate::tarea::leer(&importada) {
            Ok(Documento::Formato2(t, _)) => Some((t.claves_desconocidas().len(), t.lote)),
            _ => None,
        };
        assert_eq!(leida, Some((0, Some("hito37/F3-2".to_owned()))));
        for malo in ["sin-barra", "/clave", "lote/"] {
            let t = importada.replace("hito37/F3-2", malo);
            assert!(crate::tarea::leer(&t).is_err(), "{malo}");
        }
        assert!(!t.contains("\nlote:"));
        assert!(t.contains("id: \"TASK-008\"\n"));
        assert!(t.contains("creado: \"2026-11-15\"\nactualizado: \"2026-11-15\"\n"));
        assert!(
            t.contains("rama: \"feature/task-008-c-ruta-x-fecha\"\n"),
            "{t}"
        );
    }

    /// C7: el `tarea.md` lleva `rama: feature/task-nnn-<slug>` y esa rama
    /// pasa `rama_valida`; un titulo sin letras latinas da el slug `tarea`.
    #[test]
    fn rama_derivada_del_id_pasa_rama_valida() {
        for (titulo, rama) in [
            (
                "Nombres únicos de tarea",
                "feature/task-076-nombres-unicos-de-tarea",
            ),
            ("Задача без латиницы", "feature/task-076-tarea"),
            ("漢字", "feature/task-076-tarea"),
        ] {
            assert_eq!(rama_de("TASK-076", TIPO, titulo), rama);
            let md = tarea_md("TASK-076", titulo, "2026-11-16");
            let leida = match crate::tarea::leer(&md) {
                Ok(crate::tarea::Documento::Formato2(t, _)) => t.rama,
                _ => None,
            };
            assert_eq!(leida.as_deref(), Some(rama));
            let slug = rama.trim_start_matches("feature/task-076-");
            assert_eq!(
                crate::git::rama::rama_valida("t", "TASK-076", TIPO, slug).ok(),
                Some(rama.to_owned())
            );
            assert_eq!(comprobar_rama("TASK-076", TIPO, titulo), Ok(()));
        }
        assert_eq!(rama_de("TASK-1000", TIPO, "x"), "feature/task-1000-x");
    }

    /// TASK-029: con `tipo` como parametro, cada tipo del modelo da una rama
    /// que pasa `rama_valida`, y el `tarea.md` lleva ese tipo y esa rama.
    #[test]
    fn rama_derivada_por_tipo_pasa_rama_valida() {
        for tipo in crate::tarea::TIPOS {
            let mut e = Entrada::nueva("Arreglar el login");
            tipo.clone_into(&mut e.tipo);
            let rama = rama_de("TASK-029", tipo, &e.titulo);
            assert_eq!(rama, format!("{tipo}/task-029-arreglar-el-login"));
            assert_eq!(comprobar_rama("TASK-029", tipo, &e.titulo), Ok(()));
            let leida = match crate::tarea::leer(&tarea_de("TASK-029", &e, "2026-11-17")) {
                Ok(Documento::Formato2(t, _)) => Some((t.tipo, t.rama)),
                _ => None,
            };
            assert_eq!(leida, Some((tipo.to_owned(), Some(rama))));
        }
    }

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_owned()).collect()
    }

    /// C9 (unitario): `new` acepta los campos de RF-01 y rechaza con `uso`
    /// un valor fuera de su lista o un enunciado que no esta listo.
    #[test]
    fn new_acepta_los_campos_de_rf_01() {
        let e = entrada_de_args(&args(&[
            "--tipo",
            "fix",
            "Arreglar el login",
            "--objetivo",
            "Que entre",
            "--criterio",
            "a",
            "--criterio",
            "b",
            "--etiqueta",
            "rust",
            "--complejidad",
            "media",
            "--sprint",
            "3",
            "--dependencia",
            "TASK-001",
            "--dependencia",
            "TASK-001",
        ]));
        let mut esperada = Entrada::nueva("Arreglar el login");
        "fix".clone_into(&mut esperada.tipo);
        "Que entre".clone_into(&mut esperada.objetivo);
        esperada.criterios = vec!["a".into(), "b".into()];
        esperada.etiquetas = vec!["rust".into()];
        esperada.complejidad = Some("media".into());
        esperada.sprint = 3;
        esperada.dependencias = vec!["TASK-001".into()];
        assert_eq!(e, Ok(esperada));
        // Solo el titulo sigue valiendo (borrador de la plantilla, F2-4).
        assert_eq!(entrada_de_args(&args(&["x"])), Ok(Entrada::nueva("x")));
        let motivo = |a: &[&str]| {
            entrada_de_args(&args(a)).err().map(|i| {
                (
                    i.valor("motivo").unwrap_or_default().to_owned(),
                    i.valor("causa").unwrap_or_default().to_owned(),
                )
            })
        };
        let m = |a: &str, b: &str| Some((a.to_owned(), b.to_owned()));
        for (a, esperado) in [
            (
                vec!["x", "--objetivo", "solo"],
                m("enunciado-no-listo", "sin-criterios"),
            ),
            (
                vec!["x", "--criterio", "<criterio comprobable>"],
                m("enunciado-no-listo", "plantilla"),
            ),
            (
                vec![
                    "x",
                    "--objetivo",
                    "<que se quiere y por que>",
                    "--criterio",
                    "c",
                ],
                m("enunciado-no-listo", "plantilla"),
            ),
            (
                vec!["x", "--criterio", "  "],
                m("enunciado-no-listo", "criterio-vacio"),
            ),
            (
                vec!["x", "--criterio", "Uno", "--criterio", "uno"],
                m("enunciado-no-listo", "criterio-repetido"),
            ),
            (
                vec!["x", "--criterio", "a\nb"],
                m("enunciado-no-listo", "caracter-de-control"),
            ),
            (vec!["x", "--tipo", "epica"], m("valor-invalido", "")),
            (
                vec!["x", "--complejidad", "enorme"],
                m("valor-invalido", ""),
            ),
            (vec!["x", "--sprint", "-2"], m("valor-invalido", "")),
            (
                vec!["x", "--etiqueta", "con espacio"],
                m("valor-invalido", ""),
            ),
            (vec!["x", "--dependencia", "F2-4"], m("valor-invalido", "")),
            (vec!["x", "--tipo"], m("falta-valor", "")),
            (
                vec!["x", "--tipo", "fix", "--tipo", "fix"],
                m("argumentos-de-mas", ""),
            ),
            (vec!["x", "--otra", "v"], m("argumentos-de-mas", "")),
            (vec!["--criterio", "c"], m("sin-titulo", "")),
        ] {
            assert_eq!(motivo(&a), esperado, "{a:?}");
        }
        // El objetivo admite varias lineas.
        assert!(entrada_de_args(&args(&["x", "--objetivo", "a\nb", "--criterio", "c"])).is_ok());
    }

    #[test]
    fn reservar_da_ids_consecutivos_sin_huecos() {
        let r = rutas(&["TASK-004/tarea.md"]);
        assert_eq!(reservar(&r, &BTreeSet::from([6]), 3), Ok(vec![7, 8, 9]));
        assert_eq!(reservar(&r, &BTreeSet::new(), 0), Ok(vec![]));
        let tope = rutas(&[&format!("TASK-{}/tarea.md", u32::MAX - 2)]);
        assert_eq!(reservar(&tope, &BTreeSet::new(), 2).map(|v| v.len()), Ok(2));
        assert!(matches!(
            reservar(&tope, &BTreeSet::new(), 3),
            Err(Fallo::Error(_))
        ));
    }

    #[test]
    fn aviso_de_parecidas_en_los_dos_idiomas() {
        assert_eq!(aviso_parecidas(&[], Idioma::Es), None);
        let p = [
            Parecida {
                id: "TASK-007".into(),
                titulo: "Exportar informes".into(),
                parecido: Parecido::Duplicado,
            },
            Parecida {
                id: "TASK-009".into(),
                titulo: "Exprotar informes".into(),
                parecido: Parecido::Casi(86),
            },
            Parecida {
                id: "TASK-010".into(),
                titulo: "x".into(),
                parecido: Parecido::Casi(100),
            },
        ];
        assert_eq!(
            aviso_parecidas(&p, Idioma::Es).as_deref(),
            Some(
                "aviso: ya hay tareas abiertas con un titulo igual o parecido: TASK-007 \"Exportar informes\" (duplicado); TASK-009 \"Exprotar informes\" (parecido 0,86); TASK-010 \"x\" (parecido 1,00)"
            )
        );
        assert_eq!(
            aviso_parecidas(&p[1..2], Idioma::En).as_deref(),
            Some(
                "warning: there are already open tasks with the same or a similar title: TASK-009 \"Exprotar informes\" (similarity 0.86)"
            )
        );
        assert!(es_tarea_md("TASK-007/tarea.md") && es_tarea_md("task-7/tarea.md"));
        for r in [
            "TASK-007/plan.md",
            "TASK-007/x/tarea.md",
            "notas/tarea.md",
            "TASK-x/tarea.md",
        ] {
            assert!(!es_tarea_md(r), "{r}");
        }
    }

    /// `ABIERTOS` es `tarea::ESTADOS` menos los dos de cierre: si el modelo
    /// gana o renombra un estado, este test obliga a decidir si esta abierto.
    #[test]
    fn abiertos_son_los_estados_del_modelo_menos_los_de_cierre() {
        for e in ABIERTOS {
            assert!(crate::tarea::ESTADOS.contains(&e), "{e} no es un estado");
        }
        let cerrados: Vec<&str> = crate::tarea::ESTADOS
            .iter()
            .copied()
            .filter(|e| !ABIERTOS.contains(e))
            .collect();
        assert_eq!(cerrados, ["terminada", "descartada"]);
    }

    #[test]
    fn fechas_civiles() {
        assert_eq!(fecha_de_dias(0), "1970-01-01");
        assert_eq!(fecha_de_dias(11_016), "2000-02-29");
        assert_eq!(fecha_de_dias(20_772), "2026-11-15");
        assert_eq!(fecha_de_dias(-1), "1969-12-31");
    }

    #[test]
    fn anotacion_ida_y_vuelta() {
        let a = Alta::con_fecha("con | barra", "2026-11-15");
        assert_eq!(a.anotacion(), "new|con | barra");
        assert!(desde_anotacion(&a.anotacion()).is_some());
        assert!(desde_anotacion("new|").is_none());
        assert!(desde_anotacion("otra|x").is_none());
        assert!(desde_anotacion("newx|t").is_none());
        assert!(!titulo_valido("a\nb"));
        assert!(!titulo_valido("a\tb"));
        assert!(!titulo_valido("a\u{2028}b"));
        assert!(!titulo_valido("a\u{2029}b"));
        assert!(!titulo_valido("a\u{85}b"));
        assert!(!titulo_valido("   "));
        assert!(titulo_valido("Título con ñ"));
    }
}
