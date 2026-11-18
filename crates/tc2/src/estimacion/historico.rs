//! Lector unico del historico (ADR F1-17, decision 1): de los `tarea.md`
//! terminados de la rama de estado (los dos formatos) y de un `git log` de
//! `develop`, las muestras con su real por fase y los descartes por motivo.
//!
//! - Los commits de una tarea son los que la nombran en el asunto
//!   (`TASK-NNN`, sin mayusculas) y los de su rama, `M^1..M^2` del merge
//!   `merge(feature|fix): feature|fix/task-nnn-...` de la 0.16 (la regla de
//!   `poc-f1-17.py`).
//! - Las partes de una tarea troceada son una sola muestra (la regla textual
//!   sobre `## Objetivo` de la PoC).
//! - Carril: el declarado (D5a de TASK-083); una tarea de la 0.16 no lo
//!   tiene y solo cuenta en el nivel `repo`.
//! - Nunca escribe nada: lo derivado va aparte.

use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet};

use super::real::{self, Real};
use crate::carril::Carril;
use crate::estado::Leida;
use crate::tarea::{Cuerpo, Documento};

/// Estado de una tarea terminada.
const TERMINADA: &str = "terminada";

/// Los commits de `develop` por tarea, de un solo `git log`.
#[derive(Debug, Clone, Default)]
pub struct Commits {
    /// Instantes de los commits cuyo asunto nombra cada tarea.
    asuntos: HashMap<String, Vec<i64>>,
    /// Padres (primero, segundo) del merge de la rama de cada tarea.
    merges: HashMap<String, (String, String)>,
    /// Padres e instante de cada commit.
    grafo: HashMap<String, (Vec<String>, i64)>,
    /// Generacion de cada commit del grafo (1 + la mayor de sus padres
    /// leidos), calculada una vez en [`Commits::de_log`]: ordena el
    /// recorrido de [`Commits::solo_de`] (H1 de la ronda 1).
    generacion: HashMap<String, u32>,
    /// Si el `git log` fallo (R1-04 de la ronda 1): ninguno, y la salida lo
    /// dice con `historia=sin-log`.
    sin_log: bool,
}

/// Marca de [`Commits::solo_de`]: alcanzado desde la base (`M^1`).
const DESDE_BASE: u8 = 1;
/// Marca de [`Commits::solo_de`]: alcanzado desde la rama (`M^2`).
const DESDE_RAMA: u8 = 2;

/// Generacion de cada commit de `grafo` (sin recursion; un ciclo, que git no
/// produce, no lo cuelga).
fn generaciones(grafo: &HashMap<String, (Vec<String>, i64)>) -> HashMap<String, u32> {
    let mut gen_: HashMap<String, u32> = HashMap::with_capacity(grafo.len());
    let mut abiertos: HashSet<&str> = HashSet::new();
    for inicio in grafo.keys() {
        let mut pila: Vec<(&str, bool)> = vec![(inicio.as_str(), false)];
        while let Some((s, cerrar)) = pila.pop() {
            if gen_.contains_key(s) {
                continue;
            }
            let padres = grafo.get(s).map_or(&[][..], |(p, _)| p.as_slice());
            if cerrar {
                let g = padres
                    .iter()
                    .filter_map(|p| gen_.get(p.as_str()))
                    .max()
                    .map_or(1, |m| m.saturating_add(1));
                gen_.insert(s.to_owned(), g);
            } else if abiertos.insert(s) {
                pila.push((s, true));
                pila.extend(
                    padres
                        .iter()
                        .filter(|p| grafo.contains_key(p.as_str()))
                        .map(|p| (p.as_str(), false)),
                );
            }
        }
    }
    gen_
}

/// Si `c` es de palabra (`\w` de Python).
fn de_palabra(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// `TASK-NNN` (con tres cifras al menos) de los `\bTASK-(\d+)\b` de `texto`,
/// sin mayusculas.
fn ids_del_asunto(texto: &str) -> Vec<String> {
    let mut v = Vec::new();
    let bajo = texto.to_ascii_lowercase();
    let mut desde = 0;
    while let Some(i) = bajo.get(desde..).and_then(|r| r.find("task-")) {
        let ini = desde + i;
        desde = ini + 5;
        if bajo[..ini].chars().next_back().is_some_and(de_palabra) {
            continue;
        }
        let cifras: String = bajo[ini + 5..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        if cifras.is_empty() {
            continue;
        }
        let tras = &bajo[ini + 5 + cifras.len()..];
        if tras.chars().next().is_some_and(de_palabra) {
            continue;
        }
        if let Ok(n) = cifras.parse::<u64>() {
            let id = format!("TASK-{n:03}");
            if !v.contains(&id) {
                v.push(id);
            }
        }
    }
    v
}

/// El ID del asunto de un merge de rama de tarea de la 0.16:
/// `merge(feature|fix): feature|fix/task-<cifras>-...`.
fn id_de_merge(asunto: &str) -> Option<String> {
    let resto = ["merge(feature): ", "merge(fix): "]
        .iter()
        .find_map(|p| asunto.strip_prefix(p))?;
    let resto = ["feature/task-", "fix/task-"]
        .iter()
        .find_map(|p| resto.strip_prefix(p))?;
    let cifras: String = resto.chars().take_while(char::is_ascii_digit).collect();
    if cifras.is_empty() || !resto[cifras.len()..].starts_with('-') {
        return None;
    }
    cifras.parse::<u64>().ok().map(|n| format!("TASK-{n:03}"))
}

impl Commits {
    /// Sin commits (sin `develop` o sin `git log`).
    pub fn vacios() -> Self {
        Self::default()
    }

    /// Ninguno porque el `git log` fallo (no porque no haya `develop`).
    pub fn sin_log() -> Self {
        Self {
            sin_log: true,
            ..Self::default()
        }
    }

    /// Si el `git log` se pudo leer (o no habia `develop` que leer).
    pub fn leido(&self) -> bool {
        !self.sin_log
    }

    /// Lee la salida de `git log --format=%H%x1f%P%x1f%ct%x1f%s` (de la mas
    /// nueva a la mas vieja). Las lineas que no tienen esa forma se saltan.
    pub fn de_log(texto: &str) -> Self {
        let mut c = Self::default();
        for l in texto.lines() {
            let mut p = l.splitn(4, '\u{1f}');
            let (Some(sha), Some(padres), Some(ct), Some(asunto)) =
                (p.next(), p.next(), p.next(), p.next())
            else {
                continue;
            };
            let Ok(ts) = ct.trim().parse::<i64>() else {
                continue;
            };
            let padres: Vec<String> = padres.split_whitespace().map(str::to_owned).collect();
            for id in ids_del_asunto(asunto) {
                c.asuntos.entry(id).or_default().push(ts);
            }
            if let (Some(id), [p1, p2]) = (id_de_merge(asunto), padres.as_slice()) {
                c.merges.entry(id).or_insert((p1.clone(), p2.clone()));
            }
            c.grafo.insert(sha.to_owned(), (padres, ts));
        }
        c.generacion = generaciones(&c.grafo);
        c
    }

    /// Los commits del grafo leido que son antepasados de `rama` (incluido)
    /// y no de `base` (`base..rama`). Pinta hacia abajo por generacion desde
    /// los dos a la vez y para en cuanto no queda en la cola ningun commit
    /// alcanzado solo desde `rama`: recorre lo de la rama y lo de `develop`
    /// hasta el merge-base, no todo `develop` por cada merge (H1).
    fn solo_de<'a>(&'a self, rama: &'a str, base: &'a str) -> Vec<&'a str> {
        let mut p = Pintura::default();
        p.marcar(self, base, DESDE_BASE);
        p.marcar(self, rama, DESDE_RAMA);
        let mut fuera = Vec::new();
        while p.pendientes > 0 {
            let Some((_, s)) = p.cola.pop() else { break };
            p.hechos.insert(s);
            let m = p.marcas.get(s).copied().unwrap_or_default();
            if m == DESDE_RAMA {
                p.pendientes -= 1;
                fuera.push(s);
            }
            if let Some((padres, _)) = self.grafo.get(s) {
                for padre in padres {
                    p.marcar(self, padre, m);
                }
            }
        }
        fuera
    }

    /// Instantes de los commits de la tarea `id`: los que la nombran en el
    /// asunto y los de su rama (`M^1..M^2`).
    pub fn de_tarea(&self, id: &str) -> Vec<i64> {
        let mut v = self.asuntos.get(id).cloned().unwrap_or_default();
        if let Some((p1, p2)) = self.merges.get(id) {
            v.extend(
                self.solo_de(p2, p1)
                    .into_iter()
                    .filter_map(|s| self.grafo.get(s).map(|(_, ts)| *ts)),
            );
        }
        v.sort_unstable();
        v
    }
}

/// El estado del recorrido de [`Commits::solo_de`].
#[derive(Default)]
struct Pintura<'a> {
    /// Marcas de cada commit alcanzado.
    marcas: HashMap<&'a str, u8>,
    /// Por recorrer, la generacion mayor primero (un commit sale despues de
    /// todos sus hijos: sus marcas ya son las finales).
    cola: BinaryHeap<(u32, &'a str)>,
    /// Ya sacados de la cola.
    hechos: HashSet<&'a str>,
    /// En la cola con solo la marca de la rama.
    pendientes: usize,
}

impl<'a> Pintura<'a> {
    /// Anade la marca `m` a `s` (si es del grafo leido y no ha salido ya).
    fn marcar(&mut self, c: &Commits, s: &'a str, m: u8) {
        let Some(&g) = c.generacion.get(s) else {
            return;
        };
        if self.hechos.contains(s) {
            return;
        }
        let antes = self.marcas.get(s).copied().unwrap_or_default();
        let ahora = antes | m;
        if ahora == antes {
            return;
        }
        self.marcas.insert(s, ahora);
        if antes == 0 {
            self.cola.push((g, s));
        }
        if ahora == DESDE_RAMA {
            self.pendientes += 1;
        } else if antes == DESDE_RAMA {
            self.pendientes -= 1;
        }
    }
}

/// Una muestra: una tarea terminada (o un grupo de troceo) con su real.
#[derive(Debug, Clone, PartialEq)]
pub struct Muestra {
    /// ID, o los IDs del grupo unidos por `+` (por orden de `start`).
    pub id: String,
    /// Las tareas de la muestra.
    pub miembros: Vec<String>,
    /// Carril declarado (el mayor del grupo); `None` en la 0.16.
    pub carril: Option<Carril>,
    /// Complejidad (la de la primera del grupo).
    pub complejidad: Option<String>,
    /// Casillas de `## Criterios de aceptacion` (sumadas en un grupo).
    pub criterios: usize,
    /// Real por fase (sumado en un grupo).
    pub real: Real,
    /// `start` de la primera.
    pub start: i64,
    /// `finish` de la ultima.
    pub finish: i64,
}

/// Lo que sale del lector: muestras y descartes.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Historico {
    /// Muestras, por orden de `start`.
    pub muestras: Vec<Muestra>,
    /// Tareas terminadas que dan muestra (antes de agrupar).
    pub usables: usize,
    /// IDs descartados por motivo.
    pub descartes: BTreeMap<&'static str, Vec<String>>,
    /// Filas de las tablas que no se entendieron.
    pub filas_malas: usize,
}

impl Historico {
    /// Tareas terminadas descartadas.
    pub fn descartadas(&self) -> usize {
        self.descartes.values().map(Vec::len).sum()
    }
}

/// Casillas `- [ ]` o `- [x]` de `## Criterios de aceptacion` (el tamano de
/// F1-17, divergencia 7).
pub fn criterios(cuerpo: &Cuerpo) -> usize {
    cuerpo.seccion("Criterios de aceptacion").map_or(0, |s| {
        s.texto
            .lines()
            .filter(|l| ["- [ ]", "- [x]", "- [X]"].iter().any(|p| l.starts_with(p)))
            .count()
    })
}

/// `TASK-<cifras>` al principio de `s`, tal cual (sin normalizar, como la
/// PoC).
fn id_al_principio(s: &str) -> Option<&str> {
    let resto = s.strip_prefix("TASK-")?;
    let n = resto.bytes().take_while(u8::is_ascii_digit).count();
    (n > 0).then(|| &s[..5 + n])
}

/// Posiciones (en bytes) donde empieza `aguja` en `texto`.
fn posiciones<'a>(texto: &'a str, aguja: &'a str) -> impl Iterator<Item = usize> + 'a {
    texto.match_indices(aguja).map(|(i, _)| i)
}

/// Si en `i` hay limite de palabra antes (como `\b` delante de una letra).
fn limite_antes(texto: &str, i: usize) -> bool {
    !texto[..i].chars().next_back().is_some_and(de_palabra)
}

/// IDs con los que se agrupa una tarea por lo que dice su objetivo (la
/// regla textual de `troceo()` en la PoC, sin regex).
pub fn troceo(id: &str, objetivo: &str) -> HashSet<String> {
    let mut ids: HashSet<String> = HashSet::new();
    let mut tras = |i: usize, prefijo: &str| {
        if let Some(t) = objetivo.get(i + prefijo.len()..).and_then(id_al_principio) {
            ids.insert(t.to_owned());
        }
    };
    let con_limite = [
        "Primera parte de ",
        "Segunda parte de ",
        "Tercera parte de ",
        "Cuarta parte de ",
        "Quinta parte de ",
        "Sexta parte de ",
        "Parte de ",
        "Troceo de ",
        "continuación de ",
        "continuacion de ",
        "Sigue a ",
    ];
    for p in con_limite {
        for i in posiciones(objetivo, p) {
            if limite_antes(objetivo, i) {
                tras(i, p);
            }
        }
    }
    for p in ["pasa a ", "pasan a "] {
        for i in posiciones(objetivo, p) {
            let resto = &objetivo[i + p.len()..];
            if let Some(t) = id_al_principio(resto)
                && resto[t.len()..].starts_with(" (troceo")
            {
                ids.insert(t.to_owned());
            }
        }
    }
    // `parte del [^.]{0,80}? troceado en (TASK-\d+)`: el hueco mas corto.
    for i in posiciones(objetivo, "parte del ") {
        let resto = &objetivo[i + "parte del ".len()..];
        for (j, _) in resto.match_indices(" troceado en ") {
            let hueco = &resto[..j];
            if hueco.contains('.') || hueco.chars().count() > 80 {
                break;
            }
            if let Some(t) = id_al_principio(&resto[j + " troceado en ".len()..]) {
                ids.insert(t.to_owned());
                break;
            }
        }
    }
    // `(TASK-\d+) dej[oó] resuelto`.
    for i in posiciones(objetivo, "TASK-") {
        if let Some(t) = id_al_principio(&objetivo[i..]) {
            let resto = &objetivo[i + t.len()..];
            if resto.starts_with(" dejo resuelto") || resto.starts_with(" dejó resuelto") {
                ids.insert(t.to_owned());
            }
        }
    }
    // `Troceada en el dise[nñ]o[^.]*\.?[^.]*`: los IDs de esa frase y la
    // siguiente.
    let inicio = ["Troceada en el diseno", "Troceada en el diseño"]
        .iter()
        .filter_map(|p| objetivo.find(p))
        .min();
    if let Some(i) = inicio {
        let resto = &objetivo[i..];
        let mut fin = resto.find('.').unwrap_or(resto.len());
        if resto[fin..].starts_with('.') {
            fin += 1;
            fin += resto[fin..].find('.').unwrap_or(resto.len() - fin);
        }
        let frase = &resto[..fin];
        for j in posiciones(frase, "TASK-") {
            if let Some(t) = id_al_principio(&frase[j..]) {
                ids.insert(t.to_owned());
            }
        }
    }
    ids.remove(id);
    ids
}

/// Una tarea terminada leida, antes de agrupar.
struct Leida1 {
    id: String,
    carril: Option<Carril>,
    complejidad: Option<String>,
    criterios: usize,
    real: Real,
    start: i64,
    finish: i64,
    troceo: HashSet<String>,
}

/// Lo comun de los dos formatos que necesita el lector.
fn datos(doc: &Documento) -> (&str, &str, Option<Carril>, Option<String>, &Cuerpo) {
    match doc {
        Documento::Formato1(t, c) => (
            t.id.as_str(),
            t.estado.as_str(),
            None,
            t.complejidad.clone(),
            c,
        ),
        Documento::Formato2(t, c) => (
            t.id.as_str(),
            t.estado.as_str(),
            t.carril.as_deref().and_then(Carril::desde),
            t.complejidad.clone(),
            c,
        ),
    }
}

/// `historico(tareas) -> (muestras, descartes)` sobre los `tarea.md` leidos
/// de la rama de estado (los de [`crate::estado::tareas`]) y los commits.
pub fn historico(leidas: &[Leida], commits: &Commits) -> Historico {
    let mut h = Historico::default();
    let mut sueltas: Vec<Leida1> = Vec::new();
    for (_, doc) in leidas {
        let Ok(doc) = doc else { continue };
        let (id, estado, carril, complejidad, cuerpo) = datos(doc);
        if estado != TERMINADA {
            continue;
        }
        let ev = real::eventos(cuerpo, commits.de_tarea(id));
        h.filas_malas += ev.filas_malas;
        match real::real(&ev) {
            Ok(r) => {
                let de = |f: &'static str| ev.transiciones.iter().filter(move |p| p.fase == f);
                let objetivo = cuerpo.seccion("Objetivo").map_or("", |s| s.texto.as_str());
                sueltas.push(Leida1 {
                    id: id.to_owned(),
                    carril,
                    complejidad,
                    criterios: criterios(cuerpo),
                    real: r,
                    start: de("start").map(|p| p.ts).min().unwrap_or_default(),
                    finish: de("finish").map(|p| p.ts).max().unwrap_or_default(),
                    troceo: troceo(id, objetivo),
                });
            }
            Err(d) => h
                .descartes
                .entry(d.motivo())
                .or_default()
                .push(id.to_owned()),
        }
    }
    h.usables = sueltas.len();
    h.muestras = agrupar(sueltas);
    h
}

/// Raiz de `x` (union-find con compresion).
fn raiz(padre: &mut HashMap<String, String>, x: &str) -> String {
    let mut r = x.to_owned();
    while let Some(p) = padre.get(&r).filter(|p| **p != r).cloned() {
        r = p;
    }
    let mut y = x.to_owned();
    while let Some(p) = padre.get(&y).filter(|p| **p != y).cloned() {
        padre.insert(y, r.clone());
        y = p;
    }
    r
}

/// Los grupos de troceo como una muestra cada uno (la raiz es el ID menor),
/// por orden de `start`.
fn agrupar(sueltas: Vec<Leida1>) -> Vec<Muestra> {
    let mut padre: HashMap<String, String> = sueltas
        .iter()
        .map(|m| (m.id.clone(), m.id.clone()))
        .collect();
    for m in &sueltas {
        for o in &m.troceo {
            if padre.contains_key(o) {
                let (a, b) = (raiz(&mut padre, &m.id), raiz(&mut padre, o));
                if a != b {
                    let (menor, mayor) = if a < b { (a, b) } else { (b, a) };
                    padre.insert(mayor, menor);
                }
            }
        }
    }
    let mut grupos: Vec<(String, Vec<Leida1>)> = Vec::new();
    for m in sueltas {
        let r = raiz(&mut padre, &m.id);
        match grupos.iter_mut().find(|(g, _)| *g == r) {
            Some((_, v)) => v.push(m),
            None => grupos.push((r, vec![m])),
        }
    }
    let mut v: Vec<Muestra> = grupos.into_iter().map(|(_, ms)| de_grupo(ms)).collect();
    v.sort_by_key(|m| m.start);
    v
}

/// Un grupo como muestra: real sumado por fase (en orden de `start`),
/// criterios sumados, el carril mayor, la complejidad de la primera.
fn de_grupo(mut ms: Vec<Leida1>) -> Muestra {
    ms.sort_by_key(|m| m.start);
    let suma = |f: fn(&Real) -> f64| ms.iter().fold(0.0, |acc, m| acc + f(&m.real));
    let real = Real {
        implementacion: suma(|r| r.implementacion),
        rondas: suma(|r| r.rondas),
        cierre: suma(|r| r.cierre),
    };
    let miembros: Vec<String> = ms.iter().map(|m| m.id.clone()).collect();
    Muestra {
        id: miembros.join("+"),
        carril: ms.iter().filter_map(|m| m.carril).max(),
        complejidad: ms.first().and_then(|m| m.complejidad.clone()),
        criterios: ms.iter().map(|m| m.criterios).sum(),
        real,
        start: ms.first().map_or(0, |m| m.start),
        finish: ms.iter().map(|m| m.finish).max().unwrap_or_default(),
        miembros,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_del_asunto_como_la_poc() {
        assert_eq!(ids_del_asunto("feat(TASK-083): x"), ["TASK-083"]);
        assert_eq!(ids_del_asunto("docs(task-3): nota"), ["TASK-003"]);
        assert_eq!(
            ids_del_asunto("merge(feature): feature/task-001-x -> develop"),
            ["TASK-001"]
        );
        assert!(ids_del_asunto("TASK-083a y xTASK-1 y TASK- y TASK_1").is_empty());
        assert_eq!(
            ids_del_asunto("TASK-0012, TASK-12 y TASK-1000"),
            ["TASK-012", "TASK-1000"]
        );
        assert_eq!(
            id_de_merge("merge(fix): fix/task-12-x -> develop").as_deref(),
            Some("TASK-012")
        );
        for no in [
            "Merge(feature): feature/task-1-x",
            "merge(hotfix): hotfix/task-1-x",
            "merge(feature): feature/task-x",
            "merge(feature): feature/task-1",
        ] {
            assert_eq!(id_de_merge(no), None, "{no}");
        }
    }

    /// `M^1..M^2`: los commits de la rama, no los de `develop` de antes.
    #[test]
    fn commits_de_la_rama_y_del_asunto() {
        let l = |sha: &str, padres: &str, ts: i64, s: &str| {
            format!("{sha}\u{1f}{padres}\u{1f}{ts}\u{1f}{s}\n")
        };
        let log = [
            l(
                "m",
                "d1 b2",
                50,
                "merge(feature): feature/task-007-x -> develop",
            ),
            l("b2", "b1", 40, "dos"),
            l("d1", "d0", 35, "chore(TASK-007): en develop"),
            l("b1", "d0", 30, "uno"),
            l("d0", "", 10, "inicial"),
            "basura\n".to_owned(),
        ]
        .concat();
        let c = Commits::de_log(&log);
        assert_eq!(c.de_tarea("TASK-007"), vec![30, 35, 40, 50]);
        assert!(c.de_tarea("TASK-008").is_empty());
        assert!(Commits::vacios().de_tarea("TASK-007").is_empty());
    }

    /// El calculo de antes de H1 (antepasados completos de `M^1` por cada
    /// merge), como referencia.
    fn antepasados(c: &Commits, sha: &str, fuera: &HashSet<String>) -> Vec<String> {
        let mut vistos: HashSet<String> = HashSet::new();
        let mut pila = vec![sha.to_owned()];
        let mut orden = Vec::new();
        while let Some(s) = pila.pop() {
            if fuera.contains(&s) || !vistos.insert(s.clone()) {
                continue;
            }
            if let Some((padres, _)) = c.grafo.get(&s) {
                pila.extend(padres.iter().cloned());
                orden.push(s);
            }
        }
        orden
    }

    fn de_tarea_ingenuo(c: &Commits, id: &str) -> Vec<i64> {
        let mut v = c.asuntos.get(id).cloned().unwrap_or_default();
        if let Some((p1, p2)) = c.merges.get(id) {
            let base: HashSet<String> = antepasados(c, p1, &HashSet::new()).into_iter().collect();
            v.extend(
                antepasados(c, p2, &base)
                    .iter()
                    .filter_map(|s| c.grafo.get(s).map(|(_, ts)| *ts)),
            );
        }
        v.sort_unstable();
        v
    }

    /// Un `develop` sintetico con `tareas` merges de rama: ramas que salen
    /// de puntos viejos de `develop` o de otra rama, que traen `develop` a
    /// medias y con relojes desordenados.
    fn log_sintetico(tareas: usize, entre: usize, semilla: u64) -> String {
        let mut r = semilla;
        let mut azar = move || {
            r = r
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            usize::try_from(r >> 33).unwrap_or_default()
        };
        let mut lineas: Vec<String> = Vec::new();
        let mut t: i64 = 1_000_000;
        let mut n = 0usize;
        let mut commit = |padres: &[String], asunto: &str, azar: &mut dyn FnMut() -> usize| {
            n += 1;
            t += 60;
            let sha = format!("c{n:07}");
            let ts = t + (i64::try_from(azar() % 7).unwrap_or_default() - 3) * 1000;
            lineas.push(format!(
                "{sha}\u{1f}{}\u{1f}{ts}\u{1f}{asunto}",
                padres.join(" ")
            ));
            sha
        };
        let mut develop = vec![commit(&[], "inicial", &mut azar)];
        let mut puntas: Vec<String> = Vec::new();
        for k in 0..tareas {
            for _ in 0..entre {
                let asunto = if azar() % 9 == 0 {
                    format!("docs(TASK-{:03}): nota", azar() % (tareas + 1))
                } else {
                    "chore: algo".to_owned()
                };
                let d = commit(&[develop[develop.len() - 1].clone()], &asunto, &mut azar);
                develop.push(d);
            }
            let mut punta = if !puntas.is_empty() && azar() % 5 == 0 {
                puntas[azar() % puntas.len()].clone()
            } else {
                develop[develop.len() - 1 - azar() % develop.len().min(50)].clone()
            };
            for j in 0..1 + azar() % 4 {
                punta = if j == 1 && azar() % 3 == 0 {
                    commit(
                        &[punta, develop[develop.len() - 1].clone()],
                        "merge develop en la rama",
                        &mut azar,
                    )
                } else {
                    commit(&[punta], "trabajo", &mut azar)
                };
            }
            puntas.push(punta.clone());
            let m = commit(
                &[develop[develop.len() - 1].clone(), punta],
                &format!("merge(feature): feature/task-{k:03}-x -> develop"),
                &mut azar,
            );
            develop.push(m);
        }
        lineas.reverse();
        lineas.join("\n")
    }

    /// H1: recorrer solo hasta el merge-base da lo mismo que recorrer todos
    /// los antepasados de `M^1` por cada merge.
    #[test]
    fn h1_rama_igual_que_todos_los_antepasados() {
        let mut con_rama = 0;
        for semilla in 1..=6 {
            let c = Commits::de_log(&log_sintetico(60, 8, semilla));
            for k in 0..61 {
                let id = format!("TASK-{k:03}");
                let v = c.de_tarea(&id);
                assert_eq!(v, de_tarea_ingenuo(&c, &id), "{semilla} {id}");
                con_rama += usize::from(c.merges.contains_key(&id) && !v.is_empty());
            }
        }
        assert!(con_rama >= 300, "{con_rama}");
        // Un ciclo (que git no produce) ni cuelga ni hace panic.
        let ciclo = "a\u{1f}b\u{1f}1\u{1f}x\nb\u{1f}a\u{1f}2\u{1f}y\nm\u{1f}a b\u{1f}3\u{1f}merge(feature): feature/task-001-x -> develop\n";
        let _ = Commits::de_log(ciclo).de_tarea("TASK-001");
    }

    /// H1, medida: 300 merges sobre unos 30 000 commits; con
    /// `cargo test --release -p tc2 --lib h1_medida -- --ignored --nocapture`.
    #[test]
    #[ignore = "medida de tiempo, a mano"]
    fn h1_medida() {
        let log = log_sintetico(300, 100, 7);
        let ids: Vec<String> = (0..300).map(|k| format!("TASK-{k:03}")).collect();
        let t = std::time::Instant::now();
        let c = Commits::de_log(&log);
        let nuevo: Vec<Vec<i64>> = ids.iter().map(|id| c.de_tarea(id)).collect();
        let d_nuevo = t.elapsed();
        let t = std::time::Instant::now();
        let viejo: Vec<Vec<i64>> = ids.iter().map(|id| de_tarea_ingenuo(&c, id)).collect();
        let d_viejo = t.elapsed();
        assert_eq!(nuevo, viejo);
        eprintln!(
            "commits={} de_log+de_tarea nuevo={d_nuevo:?} antes(de_tarea)={d_viejo:?}",
            c.grafo.len()
        );
    }

    #[test]
    fn troceo_por_el_objetivo() {
        let t = |o: &str| {
            let mut v: Vec<String> = troceo("TASK-010", o).into_iter().collect();
            v.sort();
            v
        };
        assert_eq!(t("Segunda parte de TASK-005: lo que falto."), ["TASK-005"]);
        assert_eq!(
            t("Parte de TASK-4 y continuación de TASK-3."),
            ["TASK-3", "TASK-4"]
        );
        assert_eq!(
            t("Sigue a TASK-002. Troceo de TASK-001"),
            ["TASK-001", "TASK-002"]
        );
        assert_eq!(t("los criterios pasan a TASK-020 (troceo)"), ["TASK-020"]);
        assert_eq!(t("la parte del cierre troceado en TASK-030"), ["TASK-030"]);
        assert_eq!(
            t("la parte del cierre. Luego troceado en TASK-030"),
            Vec::<String>::new()
        );
        assert_eq!(t("TASK-040 dejó resuelto el resto"), ["TASK-040"]);
        assert_eq!(
            t("Troceada en el diseño con TASK-050. Y TASK-051. No TASK-052"),
            ["TASK-050", "TASK-051"]
        );
        // Sin limite de palabra delante, no; y la propia no cuenta.
        assert_eq!(
            t("xParte de TASK-1 y Sigue a TASK-010"),
            Vec::<String>::new()
        );
    }
}
