# Peticion de revision — TASK-030 (ronda 1)

- Tarea: TASK-030 — F3-3 tc2 next: lo listo para empezar por dependencias y prioridad
- Rama revisada: feature/task-030-f3-3-next-lo-listo-para-empezar-por-depe
- Rama base: develop
- Commit revisado (HEAD): fc1f1887ff81feae206dda283f77e319fdfb12de
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-030 (criterios de aceptacion y plan)
- Alcance: completa (motivo: primera ronda de revision)

## Instrucciones para el agente revisor

Eres un revisor INDEPENDIENTE del agente que implemento. Tu trabajo es
reproducir empiricamente, no leer el diff y opinar: clona el repo a un
directorio temporal, corre la suite tu mismo (salvo que esta
peticion ya traiga la verificacion de la ronda en verde para el commit revisado;
entonces vale ese resultado) y construye el caso que
rompe el codigo antes de reportarlo. Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. Vuelca tu salida en el
informe de esta ronda (informe-revision-1-rust-wasm-reviewer.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-030/tarea.md
- tareas/03-en-revision/TASK-030/planificacion/
- crates/tc2/src/alta.rs
- crates/tc2/src/estado/mod.rs
- crates/tc2/src/estado/tareas.rs
- crates/tc2/src/grafo.rs
- crates/tc2/src/i18n/catalogo/cli.rs
- crates/tc2/src/i18n/catalogo/mod.rs
- crates/tc2/src/i18n/catalogo/siguiente.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/siguiente.rs
- crates/tc2/tests/siguiente.rs

## Commits a revisar (git log develop..HEAD)

````
fc1f188 chore(TASK-030): C3 con las claves del contrato F1-4
d8dde8d chore(TASK-030): autorrevision y notas de implementacion
484e64a fix(TASK-030): aviso duplicadas= para IDs repetidos
38224ca feat(TASK-030): orden next con --json, i18n y E2E
ffbbb1c refactor(TASK-030): estado::tareas lee los tarea.md de los dos formatos
02a03b3 test(TASK-030): fixture de ToolsaWeb 1fc94425 y test de C1
3cc103a feat(TASK-030): nucleo grafo con orden de trabajo y listas
d783d91 chore(TASK-030): tarea en curso
````

## Diff de tu dominio (10 fichero(s) de 13; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/alta.rs b/crates/tc2/src/alta.rs
index 0197bb5..1edfab0 100644
--- a/crates/tc2/src/alta.rs
+++ b/crates/tc2/src/alta.rs
@@ -450,15 +450,14 @@ fn es_tarea_md(ruta: &str) -> bool {
 }
 
 /// Las tareas de formato 2 del estado, en orden de ruta, leidas en un solo
-/// proceso (F2-4b). Los `tarea.md` que no se leen o son de formato 1 se
-/// saltan sin fallar.
+/// proceso (F2-4b) con [`crate::estado::tareas`]. Los `tarea.md` que no se
+/// leen o son de formato 1 se saltan sin fallar, y las carpetas con sufijo
+/// no cuentan aqui (filtro de siempre del alta).
 pub(crate) fn tareas(estado: &dyn Estado, rutas: &[String]) -> Result<Vec<Tarea>, Fallo> {
     let mds: Vec<String> = rutas.iter().filter(|r| es_tarea_md(r)).cloned().collect();
-    Ok(estado
-        .leer_varios(&mds)?
+    Ok(crate::estado::tareas(estado, &mds)?
         .into_iter()
-        .flatten()
-        .filter_map(|md| match crate::tarea::leer(&md) {
+        .filter_map(|(_, d)| match d {
             Ok(Documento::Formato2(t, _)) => Some(t),
             _ => None,
         })
diff --git a/crates/tc2/src/estado/mod.rs b/crates/tc2/src/estado/mod.rs
index c9d501b..4f88038 100644
--- a/crates/tc2/src/estado/mod.rs
+++ b/crates/tc2/src/estado/mod.rs
@@ -11,7 +11,8 @@
 //!   Sin red, las operaciones que no reservan nada compartido quedan como
 //!   pendientes y se re-ejecutan en la siguiente transaccion con red.
 //! - [`leer`] da una vista del estado (lo ultimo traido del remoto, o la
-//!   rama local en un repo sin remoto).
+//!   rama local en un repo sin remoto) y [`tareas`], sus `tarea.md` de los
+//!   dos formatos con su lectura.
 //!
 //! La correccion no depende de `tareas/`: se lee y se escribe sobre commits;
 //! la carpeta montada solo se adelanta (fast-forward) tras publicar.
@@ -23,12 +24,14 @@ mod cerrojo;
 mod lectura;
 mod montaje;
 mod pendientes;
+mod tareas;
 mod transaccion;
 
 pub use crate::git::Cambio;
 pub use cerrojo::Cerrojo;
 pub use lectura::{Estado, Vista, leer};
 pub use montaje::{Previsto, montar, montar_con, previsto};
+pub use tareas::{Leida, es_tarea_md, tareas};
 pub use transaccion::{Catalogo, Contexto, Fallo, Operacion, Politica, Roadmap, transaccion};
 
 /// Rama huerfana del estado (ADR F1-1; nombre en [`crate::huella`]).
diff --git a/crates/tc2/src/estado/tareas.rs b/crates/tc2/src/estado/tareas.rs
new file mode 100644
index 0000000..293fcfb
--- /dev/null
+++ b/crates/tc2/src/estado/tareas.rs
@@ -0,0 +1,72 @@
+//! Los `tarea.md` de la rama de estado, de los dos formatos, leidos en un
+//! solo proceso (F2-4b). Quien los usa decide que hacer con los que no se
+//! leen: `next` los cuenta en `ilegibles=` (TASK-030 C4), el alta los salta.
+
+use super::lectura::Estado;
+use super::transaccion::Fallo;
+use crate::tarea::{Documento, ErrorTarea};
+
+/// Prefijo de las carpetas de tarea.
+const PREFIJO: &str = "TASK-";
+
+/// `TASK-<cifras>[a-z]/tarea.md` en la raiz (la rama de estado es plana,
+/// ADR F1-1). El prefijo se compara sin mayusculas, como el alta.
+pub fn es_tarea_md(ruta: &str) -> bool {
+    let Some((carpeta, "tarea.md")) = ruta.split_once('/') else {
+        return false;
+    };
+    let (Some(prefijo), Some(resto)) = (carpeta.get(..PREFIJO.len()), carpeta.get(PREFIJO.len()..))
+    else {
+        return false;
+    };
+    let cifras = resto
+        .strip_suffix(|c: char| c.is_ascii_lowercase())
+        .unwrap_or(resto);
+    prefijo.eq_ignore_ascii_case(PREFIJO)
+        && !cifras.is_empty()
+        && cifras.bytes().all(|b| b.is_ascii_digit())
+}
+
+/// Un `tarea.md` con su ruta y su lectura.
+pub type Leida = (String, Result<Documento, ErrorTarea>);
+
+/// Cada `tarea.md` de `rutas` con su lectura, en orden de ruta. Un fallo de
+/// git es [`Fallo`]; un `tarea.md` que no se entiende va con su error.
+pub fn tareas(estado: &dyn Estado, rutas: &[String]) -> Result<Vec<Leida>, Fallo> {
+    let mds: Vec<String> = rutas.iter().filter(|r| es_tarea_md(r)).cloned().collect();
+    let textos = estado.leer_varios(&mds)?;
+    Ok(mds
+        .into_iter()
+        .zip(textos)
+        .filter_map(|(r, t)| t.map(|t| (r, crate::tarea::leer(&t))))
+        .collect())
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn rutas_de_tarea_md() {
+        for r in [
+            "TASK-001/tarea.md",
+            "TASK-1000/tarea.md",
+            "TASK-012a/tarea.md",
+            "task-013/tarea.md",
+        ] {
+            assert!(es_tarea_md(r), "{r}");
+        }
+        for r in [
+            "ROADMAP.md",
+            "TASK-/tarea.md",
+            "TASK-01ab/tarea.md",
+            "TASK-001/plan.md",
+            "TASK-001/x/tarea.md",
+            "x/TASK-001/tarea.md",
+            "TASK-00A/tarea.md",
+            "TÁSK-001/tarea.md",
+        ] {
+            assert!(!es_tarea_md(r), "{r}");
+        }
+    }
+}
diff --git a/crates/tc2/src/grafo.rs b/crates/tc2/src/grafo.rs
new file mode 100644
index 0000000..a98f8e4
--- /dev/null
+++ b/crates/tc2/src/grafo.rs
@@ -0,0 +1,683 @@
+//! Nucleo puro del orden de trabajo (F3-3; lo reutiliza `roadmap`, F3-4).
+//!
+//! El oraculo es `orden_de_trabajo` de `scripts/grafo-tareas.py` de ToolsaWeb
+//! (D1 de TASK-030): Kahn con prioridad sobre todas las tareas no
+//! descartadas, terminadas incluidas, de modo que una dependencia terminada
+//! de un hito posterior retrasa a quien depende de ella. A igualdad, primero
+//! la del hito anterior (por la posicion de su titulo `### Hn — titulo`, no
+//! por su numero; D2) y luego la de numero y sufijo menores (D4: no hay
+//! campo `prioridad`).
+//!
+//! - Lista para empezar: `planificada` con todas sus dependencias existentes
+//!   `terminada`. Una dependencia inexistente no bloquea (aviso); una
+//!   descartada sale del orden pero bloquea (D3, aviso); una ilegible
+//!   tambien bloquea (no se sabe si esta terminada).
+//! - Ciclo: sus tareas, y las que dependen de ellas, quedan `atascadas`,
+//!   fuera del orden y de las listas.
+//!
+//! Hoja pura: no lee git ni la rama de estado.
+
+use std::cmp::Reverse;
+use std::collections::{BTreeMap, BTreeSet, BinaryHeap};
+
+use crate::tarea::Documento;
+
+/// Estado de una tarea `planificada` (la unica que puede estar lista).
+pub const PLANIFICADA: &str = "planificada";
+/// Estado de una tarea terminada.
+pub const TERMINADA: &str = "terminada";
+/// Estado (formato 2) o etiqueta (formato 1) de una tarea descartada.
+pub const DESCARTADA: &str = "descartada";
+/// Estados de las tareas que alguien lleva entre manos.
+pub const EN_MARCHA: [&str; 3] = ["en-diseno", "en-curso", "en-revision"];
+
+/// Si una tarea cuenta para el orden, esta descartada o no se pudo leer.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum Fase {
+    /// No descartada, con su `estado`.
+    Viva(String),
+    /// Descartada: fuera del orden, pero bloquea a quien dependa de ella.
+    Descartada,
+    /// Su `tarea.md` no se entiende: fuera del orden y, como no se sabe si
+    /// esta terminada, bloquea a quien dependa de ella (quien la lee la
+    /// avisa aparte).
+    Ilegible,
+}
+
+/// Lo minimo de una tarea para ordenarla.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Nodo {
+    /// `TASK-NNN` o `TASK-NNNa`.
+    pub id: String,
+    /// Hito (`sprint` del frontmatter).
+    pub sprint: u32,
+    /// Viva con su estado, o descartada.
+    pub fase: Fase,
+    /// IDs de los que depende, tal como estan en el frontmatter.
+    pub dependencias: Vec<String>,
+}
+
+impl Nodo {
+    /// Proyecta un `tarea.md` leido. En formato 1 (0.16) se descarta con la
+    /// etiqueta `descartada`; en formato 2, con el estado `descartada`.
+    pub fn de(doc: &Documento) -> Self {
+        match doc {
+            Documento::Formato1(t, _) => Self {
+                id: t.id.clone(),
+                sprint: t.sprint,
+                fase: if t.etiquetas.iter().any(|e| e == DESCARTADA) {
+                    Fase::Descartada
+                } else {
+                    Fase::Viva(t.estado.clone())
+                },
+                dependencias: t.dependencias.clone(),
+            },
+            Documento::Formato2(t, _) => Self {
+                id: t.id.clone(),
+                sprint: t.sprint,
+                fase: if t.estado == DESCARTADA {
+                    Fase::Descartada
+                } else {
+                    Fase::Viva(t.estado.clone())
+                },
+                dependencias: t.dependencias.clone(),
+            },
+        }
+    }
+
+    /// Nodo de un `tarea.md` ilegible de la carpeta `id`: sin hito ni
+    /// dependencias conocidos.
+    pub fn ilegible(id: &str) -> Self {
+        Self {
+            id: id.to_owned(),
+            sprint: 0,
+            fase: Fase::Ilegible,
+            dependencias: Vec::new(),
+        }
+    }
+
+    /// Estado si esta viva.
+    pub fn estado(&self) -> Option<&str> {
+        match &self.fase {
+            Fase::Viva(e) => Some(e),
+            Fase::Descartada | Fase::Ilegible => None,
+        }
+    }
+}
+
+/// Orden de los hitos: la posicion de su titulo `### Hn — titulo` (D2).
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct Hitos {
+    posicion: BTreeMap<u32, usize>,
+}
+
+/// Marca de inicio de la zona manual de `ROADMAP.md` (la de la 0.16).
+const MANUAL_INICIO: &str = "<!-- manual:inicio";
+/// Marca de fin de la zona manual.
+const MANUAL_FIN: &str = "<!-- manual:fin";
+
+impl Hitos {
+    /// Hitos de las lineas `### Hn — titulo` de `texto`, en su orden; si un
+    /// hito sale dos veces vale su primera posicion (como el oraculo).
+    pub fn de_titulos(texto: &str) -> Self {
+        let mut posicion = BTreeMap::new();
+        for h in texto.lines().filter_map(hito_de_linea) {
+            let n = posicion.len();
+            posicion.entry(h).or_insert(n);
+        }
+        Self { posicion }
+    }
+
+    /// Hitos de la zona manual (`<!-- manual:inicio ... -->` hasta
+    /// `<!-- manual:fin -->`) de un `ROADMAP.md`. Sin zona, ninguno.
+    pub fn del_roadmap(texto: &str) -> Self {
+        let mut dentro = false;
+        let mut zona = String::new();
+        for l in texto.lines() {
+            let t = l.trim_start();
+            if t.starts_with(MANUAL_INICIO) {
+                dentro = true;
+            } else if t.starts_with(MANUAL_FIN) {
+                break;
+            } else if dentro {
+                zona.push_str(l);
+                zona.push('\n');
+            }
+        }
+        Self::de_titulos(&zona)
+    }
+
+    /// Si no hay ningun titulo.
+    pub fn es_vacio(&self) -> bool {
+        self.posicion.is_empty()
+    }
+
+    /// Si el hito `h` tiene titulo.
+    pub fn tiene(&self, h: u32) -> bool {
+        self.posicion.contains_key(&h)
+    }
+
+    /// Rango del hito `h`: su posicion si tiene titulo; si no, detras de
+    /// todos por su numero; sin titulos, su numero.
+    pub fn rango(&self, h: u32) -> u64 {
+        match self.posicion.get(&h) {
+            Some(p) => u64::try_from(*p).unwrap_or(u64::MAX),
+            None if self.posicion.is_empty() => u64::from(h),
+            None => 1_000_000 + u64::from(h),
+        }
+    }
+}
+
+/// `### H<n> — <titulo>` (raya U+2014 entre espacios, titulo no vacio).
+fn hito_de_linea(l: &str) -> Option<u32> {
+    let resto = l.strip_prefix("### H")?;
+    let fin = resto
+        .find(|c: char| !c.is_ascii_digit())
+        .unwrap_or(resto.len());
+    let (cifras, titulo) = resto.split_at(fin);
+    let titulo = titulo.strip_prefix(" \u{2014} ")?;
+    if cifras.is_empty() || titulo.trim_end_matches('\r').is_empty() {
+        return None;
+    }
+    cifras.parse().ok()
+}
+
+/// Avisos del calculo: no frenan, pero se dicen.
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct Avisos {
+    /// `TASK-A>TASK-B`: A depende de B, que no existe (no bloquea).
+    pub dependencias_inexistentes: Vec<String>,
+    /// `TASK-A>TASK-B`: A depende de B, descartada (bloquea, D3).
+    pub depende_de_descartada: Vec<String>,
+    /// Hitos con tareas en el orden y sin titulo `### Hn — titulo`.
+    pub hitos_sin_titulo: Vec<u32>,
+    /// IDs que salen en mas de un nodo: cuenta el primero y el resto no se
+    /// ignora en silencio.
+    pub duplicadas: Vec<String>,
+}
+
+/// Resultado del orden de trabajo.
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct Orden {
+    /// Tareas no descartadas en el orden en que se hacen.
+    pub orden: Vec<String>,
+    /// Las que un ciclo deja sin orden (las del ciclo y las que dependen de
+    /// ellas), por prioridad.
+    pub atascadas: Vec<String>,
+    /// Avisos.
+    pub avisos: Avisos,
+}
+
+/// Clave de prioridad: (rango del hito, numero, sufijo, ID).
+type Clave = (u64, u64, String, String);
+
+/// Numero y sufijo de `TASK-NNNa`; sin cifras, al final.
+fn numero_y_sufijo(id: &str) -> (u64, String) {
+    let resto = id.strip_prefix("TASK-").unwrap_or(id);
+    let fin = resto
+        .find(|c: char| !c.is_ascii_digit())
+        .unwrap_or(resto.len());
+    let (cifras, sufijo) = resto.split_at(fin);
+    (cifras.parse().unwrap_or(u64::MAX), sufijo.to_owned())
+}
+
+/// Indice por ID (si un ID se repite vale el primero).
+fn indice(nodos: &[Nodo]) -> BTreeMap<&str, &Nodo> {
+    let mut idx = BTreeMap::new();
+    for n in nodos {
+        idx.entry(n.id.as_str()).or_insert(n);
+    }
+    idx
+}
+
+/// Orden de trabajo de `nodos` con el orden de `hitos` (Kahn con montículo,
+/// O((n+e) log n)).
+pub fn orden_de_trabajo(nodos: &[Nodo], hitos: &Hitos) -> Orden {
+    let idx = indice(nodos);
+    let clave = |n: &Nodo| -> Clave {
+        let (num, suf) = numero_y_sufijo(&n.id);
+        (hitos.rango(n.sprint), num, suf, n.id.clone())
+    };
+    let vivas: BTreeMap<&str, &Nodo> = idx
+        .iter()
+        .filter(|(_, n)| matches!(n.fase, Fase::Viva(_)))
+        .map(|(k, n)| (*k, *n))
+        .collect();
+    let mut avisos = Avisos::default();
+    let mut vistos = BTreeSet::new();
+    let duplicadas: BTreeSet<&str> = nodos
+        .iter()
+        .map(|n| n.id.as_str())
+        .filter(|id| !vistos.insert(*id))
+        .collect();
+    avisos.duplicadas = duplicadas.into_iter().map(str::to_owned).collect();
+    let mut faltan: BTreeMap<&str, usize> = BTreeMap::new();
+    let mut dependientes: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
+    for (id, n) in &vivas {
+        let deps: BTreeSet<&str> = n.dependencias.iter().map(String::as_str).collect();
+        let mut cuenta = 0;
+        for d in deps {
+            match idx.get(d).map(|m| &m.fase) {
+                None => avisos.dependencias_inexistentes.push(format!("{id}>{d}")),
+                Some(Fase::Descartada) => {
+                    avisos.depende_de_descartada.push(format!("{id}>{d}"));
+                }
+                // Ya avisada como ilegible por quien la leyo; bloquea en
+                // `listas`.
+                Some(Fase::Ilegible) => {}
+                Some(Fase::Viva(_)) => {
+                    cuenta += 1;
+                    dependientes.entry(d).or_default().push(id);
+                }
+            }
+        }
+        faltan.insert(id, cuenta);
+    }
+    let mut monticulo: BinaryHeap<Reverse<Clave>> = vivas
+        .values()
+        .filter(|n| faltan.get(n.id.as_str()) == Some(&0))
+        .map(|n| Reverse(clave(n)))
+        .collect();
+    let mut orden = Vec::with_capacity(vivas.len());
+    while let Some(Reverse((_, _, _, id))) = monticulo.pop() {
+        if let Some(us) = dependientes.get(id.as_str()) {
+            for u in us {
+                if let Some(c) = faltan.get_mut(u) {
+                    *c = c.saturating_sub(1);
+                    if *c == 0
+                        && let Some(n) = vivas.get(u)
+                    {
+                        monticulo.push(Reverse(clave(n)));
+                    }
+                }
+            }
+        }
+        orden.push(id);
+    }
+    let puestas: BTreeSet<&str> = orden.iter().map(String::as_str).collect();
+    let mut atascadas: Vec<&Nodo> = vivas
+        .values()
+        .filter(|n| !puestas.contains(n.id.as_str()))
+        .copied()
+        .collect();
+    atascadas.sort_by_key(|n| clave(n));
+    let sin_titulo: BTreeSet<u32> = orden
+        .iter()
+        .filter_map(|id| idx.get(id.as_str()))
+        .map(|n| n.sprint)
+        .filter(|h| !hitos.tiene(*h))
+        .collect();
+    avisos.hitos_sin_titulo = sin_titulo.into_iter().collect();
+    Orden {
+        orden,
+        atascadas: atascadas.into_iter().map(|n| n.id.clone()).collect(),
+        avisos,
+    }
+}
+
+/// Las listas para empezar, en el orden de trabajo: `planificada` y con
+/// todas sus dependencias existentes terminadas (una descartada bloquea).
+pub fn listas(o: &Orden, nodos: &[Nodo]) -> Vec<String> {
+    let idx = indice(nodos);
+    o.orden
+        .iter()
+        .filter(|id| {
+            idx.get(id.as_str()).is_some_and(|n| {
+                n.estado() == Some(PLANIFICADA)
+                    && n.dependencias.iter().all(|d| match idx.get(d.as_str()) {
+                        None => true,
+                        Some(m) => m.estado() == Some(TERMINADA),
+                    })
+            })
+        })
+        .cloned()
+        .collect()
+}
+
+/// Las que estan en diseno, en curso o en revision, en el orden de trabajo.
+pub fn en_marcha(o: &Orden, nodos: &[Nodo]) -> Vec<String> {
+    let idx = indice(nodos);
+    o.orden
+        .iter()
+        .filter(|id| {
+            idx.get(id.as_str())
+                .and_then(|n| n.estado())
+                .is_some_and(|e| EN_MARCHA.contains(&e))
+        })
+        .cloned()
+        .collect()
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn n(id: &str, sprint: u32, estado: &str, deps: &[&str]) -> Nodo {
+        Nodo {
+            id: id.to_owned(),
+            sprint,
+            fase: if estado == DESCARTADA {
+                Fase::Descartada
+            } else {
+                Fase::Viva(estado.to_owned())
+            },
+            dependencias: deps.iter().map(|d| (*d).to_owned()).collect(),
+        }
+    }
+
+    fn ids(v: &[&str]) -> Vec<String> {
+        v.iter().map(|s| (*s).to_owned()).collect()
+    }
+
+    /// El caso de las autopruebas del oraculo (`probar`, «Roadmap»): la
+    /// dependencia manda sobre el numero; a igualdad, hito y numero menores.
+    #[test]
+    fn orden_como_el_oraculo() {
+        let nodos = [
+            n("TASK-020", 1, "planificada", &[]),
+            n("TASK-016", 0, "planificada", &["TASK-030"]),
+            n("TASK-030", 0, "planificada", &[]),
+            n("TASK-001", 0, "terminada", &[]),
+            n("TASK-014", 0, "en-diseno", &["TASK-001"]),
+            n("TASK-006", 0, "descartada", &[]),
+        ];
+        let o = orden_de_trabajo(&nodos, &Hitos::de_titulos("### H0 — Base\n"));
+        assert_eq!(
+            o.orden,
+            ids(&["TASK-001", "TASK-014", "TASK-030", "TASK-016", "TASK-020"])
+        );
+        assert_eq!(listas(&o, &nodos), ids(&["TASK-030", "TASK-020"]));
+        assert_eq!(en_marcha(&o, &nodos), ids(&["TASK-014"]));
+        assert_eq!(o.avisos.hitos_sin_titulo, [1]);
+    }
+
+    /// C2: una dependencia inexistente no bloquea y se avisa.
+    #[test]
+    fn dependencia_inexistente_no_bloquea_y_avisa() {
+        let nodos = [n("TASK-002", 0, "planificada", &["TASK-099"])];
+        let o = orden_de_trabajo(&nodos, &Hitos::default());
+        assert_eq!(listas(&o, &nodos), ids(&["TASK-002"]));
+        assert_eq!(
+            o.avisos.dependencias_inexistentes,
+            ids(&["TASK-002>TASK-099"])
+        );
+        assert!(o.atascadas.is_empty());
+    }
+
+    /// C2: una dependencia descartada sale del orden pero bloquea (D3), y
+    /// se avisa; la descartada no sale en ninguna lista.
+    #[test]
+    fn dependencia_descartada_bloquea_y_avisa() {
+        let nodos = [
+            n("TASK-001", 0, "descartada", &[]),
+            n("TASK-002", 0, "planificada", &["TASK-001"]),
+            n("TASK-003", 0, "planificada", &[]),
+        ];
+        let o = orden_de_trabajo(&nodos, &Hitos::default());
+        assert_eq!(o.orden, ids(&["TASK-002", "TASK-003"]));
+        assert_eq!(listas(&o, &nodos), ids(&["TASK-003"]));
+        assert_eq!(o.avisos.depende_de_descartada, ids(&["TASK-002>TASK-001"]));
+    }
+
+    /// Un ID repetido no se ignora en silencio: vale el primero y se avisa.
+    #[test]
+    fn id_repetido_avisa() {
+        let nodos = [
+            n("TASK-001", 0, "planificada", &[]),
+            n("TASK-001", 0, "terminada", &[]),
+            n("TASK-002", 0, "planificada", &["TASK-001"]),
+        ];
+        let o = orden_de_trabajo(&nodos, &Hitos::default());
+        assert_eq!(o.avisos.duplicadas, ids(&["TASK-001"]));
+        assert_eq!(listas(&o, &nodos), ids(&["TASK-001"]));
+    }
+
+    /// Una dependencia ilegible no es inexistente: bloquea, sin aviso de
+    /// inexistente ni de descartada (el aviso es `ilegibles=`).
+    #[test]
+    fn dependencia_ilegible_bloquea() {
+        let nodos = [
+            Nodo::ilegible("TASK-001"),
+            n("TASK-002", 0, "planificada", &["TASK-001"]),
+        ];
+        let o = orden_de_trabajo(&nodos, &Hitos::default());
+        assert_eq!(o.orden, ids(&["TASK-002"]));
+        assert!(listas(&o, &nodos).is_empty());
+        assert!(o.avisos.dependencias_inexistentes.is_empty());
+        assert!(o.avisos.depende_de_descartada.is_empty());
+    }
+
+    /// C2: un ciclo deja atascadas sus tareas y las que dependen de ellas,
+    /// fuera de las listas; el resto sigue.
+    #[test]
+    fn ciclo_deja_atascadas() {
+        let nodos = [
+            n("TASK-001", 0, "planificada", &["TASK-002"]),
+            n("TASK-002", 0, "planificada", &["TASK-001"]),
+            n("TASK-003", 0, "planificada", &["TASK-002"]),
+            n("TASK-004", 0, "planificada", &[]),
+            n("TASK-005", 0, "planificada", &["TASK-005"]),
+        ];
+        let o = orden_de_trabajo(&nodos, &Hitos::default());
+        assert_eq!(o.orden, ids(&["TASK-004"]));
+        assert_eq!(
+            o.atascadas,
+            ids(&["TASK-001", "TASK-002", "TASK-003", "TASK-005"])
+        );
+        assert_eq!(listas(&o, &nodos), ids(&["TASK-004"]));
+    }
+
+    /// C2: una dependencia terminada de un hito posterior retrasa a quien
+    /// depende de ella (el Kahn incluye las terminadas, como el oraculo).
+    #[test]
+    fn dependencia_terminada_de_un_hito_posterior_retrasa() {
+        let nodos = [
+            n("TASK-001", 0, "planificada", &["TASK-009"]),
+            n("TASK-002", 1, "planificada", &[]),
+            n("TASK-009", 2, "terminada", &[]),
+        ];
+        let o = orden_de_trabajo(&nodos, &Hitos::default());
+        assert_eq!(o.orden, ids(&["TASK-002", "TASK-009", "TASK-001"]));
+        assert_eq!(listas(&o, &nodos), ids(&["TASK-002", "TASK-001"]));
+        // Por numero de hito y sin titulos, sin el retraso iria primera.
+        let sola = [n("TASK-001", 0, "planificada", &[]), nodos[1].clone()];
+        let o = orden_de_trabajo(&sola, &Hitos::default());
+        assert_eq!(o.orden, ids(&["TASK-001", "TASK-002"]));
+    }
+
+    /// C2: los IDs con sufijo van tras su numero y antes del siguiente; los
+    /// de cuatro cifras, por numero (no como texto).
+    #[test]
+    fn ids_con_sufijo_y_de_cuatro_cifras() {
+        let nodos = [
+            n("TASK-1000", 0, "planificada", &[]),
+            n("TASK-013", 0, "planificada", &[]),
+            n("TASK-012b", 0, "planificada", &[]),
+            n("TASK-012", 0, "planificada", &[]),
+            n("TASK-012a", 0, "planificada", &["TASK-012b"]),
+            n("TASK-999", 0, "planificada", &[]),
+        ];
+        let o = orden_de_trabajo(&nodos, &Hitos::default());
+        assert_eq!(
+            o.orden,
+            ids(&[
+                "TASK-012",
+                "TASK-012b",
+                "TASK-012a",
+                "TASK-013",
+                "TASK-999",
+                "TASK-1000"
+            ])
+        );
+        // TASK-012a espera a TASK-012b, que no esta terminada.
+        assert!(!listas(&o, &nodos).contains(&"TASK-012a".to_owned()));
+    }
+
+    /// C2: un hito sin titulo va detras de todos los que lo tienen, por su
+    /// numero, y se avisa; el orden de los titulados es el de sus titulos.
+    #[test]
+    fn hito_sin_titulo_va_detras_y_avisa() {
+        let hitos = Hitos::de_titulos("### H0 — Base\nx\n### H14 — Motor\n### H1 — IDE\n");
+        let nodos = [
+            n("TASK-001", 7, "planificada", &[]),
+            n("TASK-002", 1, "planificada", &[]),
+            n("TASK-003", 14, "planificada", &[]),
+            n("TASK-004", 3, "planificada", &[]),
+        ];
+        let o = orden_de_trabajo(&nodos, &hitos);
+        assert_eq!(
+            o.orden,
+            ids(&["TASK-003", "TASK-002", "TASK-004", "TASK-001"])
+        );
+        assert_eq!(o.avisos.hitos_sin_titulo, [3, 7]);
+        assert_eq!([hitos.rango(0), hitos.rango(14), hitos.rango(1)], [0, 1, 2]);
+        assert_eq!(hitos.rango(7), 1_000_000 + 7);
+        // Sin ningun titulo, el numero.
+        assert_eq!(Hitos::default().rango(7), 7);
+    }
+
+    #[test]
+    fn hitos_de_la_zona_manual_del_roadmap() {
+        let r = "# Roadmap\n### H9 — Fuera\n<!-- manual:inicio (se edita a mano) -->\n## Hitos\n\
+                 ### H2 — Dos\n### H1 — Uno\n### H3 —\n### Hx — Mal\n### H2 — Otra vez\n\
+                 <!-- manual:fin -->\n### H8 — Fuera\n";
+        let h = Hitos::del_roadmap(r);
+        assert_eq!([h.rango(2), h.rango(1)], [0, 1]);
+        assert!(!h.tiene(9) && !h.tiene(8) && !h.tiene(3));
+        assert!(Hitos::del_roadmap("### H1 — Sin zona\n").es_vacio());
+    }
+
+    #[test]
+    fn nodo_de_los_dos_formatos() {
+        let v1 = "---\nid: TASK-005\ntitulo: \"t\"\ntipo: feature\nsprint: 2\netiquetas: [x, descartada]\n\
+                  complejidad: null\nmodelo_sugerido: sonnet\nestado: planificada\nplan_aprobado: false\n\
+                  rama: feature/task-005-t\nasignado_a: null\nagente_revisor: general-purpose\n\
+                  skills_recomendados: []\nregla_seleccion_skill: null\nultimo_commit_revisado: null\n\
+                  revision_codex: false\ntokens_diseno: null\ntokens_implementacion: null\n\
+                  tokens_revision: null\ncreado: 2026-01-01\nactualizado: 2026-01-01\n\
+                  dependencias: [TASK-001]\n---\n";
+        let d = crate::tarea::leer(v1);
+        assert!(d.is_ok(), "{d:?}");
+        let Ok(d) = d else { return };
+        let nodo = Nodo::de(&d);
+        assert_eq!(
+            (nodo.id.as_str(), nodo.sprint, &nodo.fase),
+            ("TASK-005", 2, &Fase::Descartada)
+        );
+        assert_eq!(nodo.dependencias, ids(&["TASK-001"]));
+        let mut t = crate::tarea::Tarea::nueva("TASK-006", "t", "2026-01-01");
+        t.estado = DESCARTADA.to_owned();
+        let d2 = Documento::Formato2(t.clone(), crate::tarea::Cuerpo::leer(""));
+        assert_eq!(Nodo::de(&d2).fase, Fase::Descartada);
+        t.estado = "en-curso".to_owned();
+        let d2 = Documento::Formato2(t, crate::tarea::Cuerpo::leer(""));
+        assert_eq!(Nodo::de(&d2).estado(), Some("en-curso"));
+    }
+
+    /// Instantanea de ToolsaWeb (SHA en la cabecera de cada fichero; D1):
+    /// `tareas.tsv` y `hitos.txt` salen de `cargar` y `titulos_de_hitos` de
+    /// `scripts/grafo-tareas.py`, y `esperado.txt` son las lineas «En
+    /// marcha» y «Listas para empezar» del `ROADMAP.md` de ese commit, en el
+    /// que `grafo-tareas.py --comprobar` sale 0.
+    const TW_TAREAS: &str = include_str!("../tests/fixtures/toolsaweb-1fc94425/tareas.tsv");
+    const TW_HITOS: &str = include_str!("../tests/fixtures/toolsaweb-1fc94425/hitos.txt");
+    const TW_ESPERADO: &str = include_str!("../tests/fixtures/toolsaweb-1fc94425/esperado.txt");
+
+    /// `id\tsprint\testado\tdescartada(si|no)\tdependencias(,)`.
+    fn nodos_tsv(tsv: &str) -> Vec<Nodo> {
+        tsv.lines()
+            .map(|l| l.trim_end_matches('\r'))
+            .filter(|l| !l.is_empty() && !l.starts_with('#'))
+            .map(|l| {
+                let c: Vec<&str> = l.split('\t').collect();
+                let campo = |i: usize| c.get(i).copied().unwrap_or_default();
+                Nodo {
+                    id: campo(0).to_owned(),
+                    sprint: campo(1).parse().unwrap_or(u32::MAX),
+                    fase: if campo(3) == "si" {
+                        Fase::Descartada
+                    } else {
+                        Fase::Viva(campo(2).to_owned())
+                    },
+                    dependencias: campo(4)
+                        .split(',')
+                        .filter(|d| !d.is_empty())
+                        .map(str::to_owned)
+                        .collect(),
+                }
+            })
+            .collect()
+    }
+
+    /// IDs de la linea `- **<titulo>:** TASK-1 (x), TASK-2` del ROADMAP.
+    fn linea_esperada(titulo: &str) -> Vec<String> {
+        let prefijo = format!("- **{titulo}:** ");
+        let resto = TW_ESPERADO
+            .lines()
+            .find_map(|l| l.trim_end_matches('\r').strip_prefix(prefijo.as_str()))
+            .unwrap_or_default();
+        if resto == "ninguna" {
+            return Vec::new();
+        }
+        resto
+            .split(", ")
+            .filter_map(|p| p.split(' ').next())
+            .map(str::to_owned)
+            .collect()
+    }
+
+    /// C1: el nucleo da exactamente las listas del ROADMAP de ToolsaWeb,
+    /// en el mismo orden, y las en marcha.
+    #[test]
+    fn toolsaweb_listas_y_en_marcha_como_su_roadmap() {
+        assert!(TW_TAREAS.starts_with("# ToolsaWeb 1fc94425964a5d33b5e2c3c4c39691dbd580c641"));
+        let nodos = nodos_tsv(TW_TAREAS);
+        assert_eq!(nodos.len(), 404);
+        let hitos = Hitos::de_titulos(TW_HITOS);
+        let o = orden_de_trabajo(&nodos, &hitos);
+        let esperadas = linea_esperada("Listas para empezar");
+        assert_eq!(esperadas.len(), 54);
+        assert_eq!(listas(&o, &nodos), esperadas);
+        assert_eq!(en_marcha(&o, &nodos), linea_esperada("En marcha"));
+        assert!(o.atascadas.is_empty());
+        assert!(o.avisos.hitos_sin_titulo.is_empty());
+        assert!(o.avisos.dependencias_inexistentes.is_empty());
+        // Ordenar los hitos por su numero no lo da (H14 va tras H0).
+        let por_numero = orden_de_trabajo(&nodos, &Hitos::default());
+        assert_ne!(listas(&por_numero, &nodos), esperadas);
+    }
+
+    /// Los `tarea.md` reales de ToolsaWeb en el SHA de la fixture pasan por
+    /// el lector estricto de formato 1 y se proyectan igual que la fixture.
+    /// Necesita un clon en ese SHA: `TC2_TOOLSAWEB=<clon> cargo test -p tc2
+    /// --lib toolsaweb_reales -- --ignored`.
+    #[test]
+    #[ignore = "necesita un clon de ToolsaWeb en TC2_TOOLSAWEB"]
+    fn toolsaweb_reales_se_leen_y_proyectan_como_la_fixture() {
+        let raiz = std::env::var_os("TC2_TOOLSAWEB");
+        assert!(raiz.is_some(), "falta TC2_TOOLSAWEB");
+        let Some(raiz) = raiz else { return };
+        let tareas = std::path::Path::new(&raiz).join("tareas");
+        let mut leidos = Vec::new();
+        let mut fallos = Vec::new();
+        for fase in std::fs::read_dir(&tareas).into_iter().flatten().flatten() {
+            for t in std::fs::read_dir(fase.path())
+                .into_iter()
+                .flatten()
+                .flatten()
+            {
+                let md = t.path().join("tarea.md");
+                let Ok(texto) = std::fs::read_to_string(&md) else {
+                    continue;
+                };
+                match crate::tarea::leer(&texto.replace("\r\n", "\n")) {
+                    Ok(d) => leidos.push(Nodo::de(&d)),
+                    Err(e) => fallos.push(format!("{}: {e}", md.display())),
+                }
+            }
+        }
+        assert!(fallos.is_empty(), "{}", fallos.join("\n"));
+        leidos.sort_by(|a, b| a.id.cmp(&b.id));
+        assert_eq!(leidos, nodos_tsv(TW_TAREAS));
+    }
+}
diff --git a/crates/tc2/src/i18n/catalogo/cli.rs b/crates/tc2/src/i18n/catalogo/cli.rs
index 86f8a72..ffc7f5c 100644
--- a/crates/tc2/src/i18n/catalogo/cli.rs
+++ b/crates/tc2/src/i18n/catalogo/cli.rs
@@ -10,8 +10,8 @@ crate::i18n::textos! {
         en: "tc2: could not write the output: {error}",
     }
     USO {
-        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
-        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
+        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
+        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
     }
     FALTA_LA_ORDEN {
         es: "tc2: falta la orden. {uso}",
diff --git a/crates/tc2/src/i18n/catalogo/mod.rs b/crates/tc2/src/i18n/catalogo/mod.rs
index cdacf1a..6ec4d46 100644
--- a/crates/tc2/src/i18n/catalogo/mod.rs
+++ b/crates/tc2/src/i18n/catalogo/mod.rs
@@ -12,6 +12,7 @@ pub mod identidad;
 pub mod importar;
 pub mod init;
 pub mod plataforma;
+pub mod siguiente;
 
 use super::Texto;
 
@@ -29,6 +30,7 @@ pub fn todos() -> impl Iterator<Item = &'static Texto> {
         importar::TODOS,
         init::TODOS,
         plataforma::TODOS,
+        siguiente::TODOS,
     ]
     .into_iter()
     .flatten()
diff --git a/crates/tc2/src/i18n/catalogo/siguiente.rs b/crates/tc2/src/i18n/catalogo/siguiente.rs
new file mode 100644
index 0000000..4c74b92
--- /dev/null
+++ b/crates/tc2/src/i18n/catalogo/siguiente.rs
@@ -0,0 +1,8 @@
+//! Mensajes de `tc2 next` (F3-3).
+
+crate::i18n::textos! {
+    USO_NEXT {
+        es: "uso: next [--json]",
+        en: "usage: next [--json]",
+    }
+}
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 156a393..9d5da4a 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -13,7 +13,8 @@
 //! deja en un repo, F1-12 D1a) e [`init`] (deja un repo listo, F3-1) e
 //! [`instalacion`] (donde vive el `tc2` de la
 //! maquina, F2-12 y F2-14) y [`desinstalar`] (quitar TaskCodeTwo, F2-14) e
-//! [`importar`] (alta en bloque con dependencias e hito, F3-2). Las capas
+//! [`importar`] (alta en bloque con dependencias e hito, F3-2) y [`grafo`] y
+//! [`siguiente`] (orden de trabajo y `next`, F3-3). Las capas
 //! que vienen entran como modulos de esta biblioteca.
 
 pub mod actualizar_cli;
@@ -25,6 +26,7 @@ pub mod descarga;
 pub mod desinstalar;
 pub mod estado;
 pub mod git;
+pub mod grafo;
 pub mod huella;
 pub mod i18n;
 pub mod identidad;
@@ -32,6 +34,7 @@ pub mod importar;
 pub mod init;
 pub mod instalacion;
 pub mod plataforma;
+pub mod siguiente;
 pub mod tarea;
 pub mod version;
 mod yaml;
@@ -106,6 +109,10 @@ where
             let s = importar::orden(&resto, &catalogo, &mut parecidas).salida_en(json, idioma);
             con_aviso(s, &parecidas, idioma)
         }
+        [orden, resto @ ..] if orden == siguiente::NOMBRE => {
+            let (json, resto) = con_json(resto);
+            siguiente::orden(&resto).salida_en(json, idioma)
+        }
         [orden, resto @ ..] if orden == version::VERSION => {
             let (json, resto) = con_json(resto);
             orden_version(json, &resto, idioma)
diff --git a/crates/tc2/src/siguiente.rs b/crates/tc2/src/siguiente.rs
new file mode 100644
index 0000000..74802f4
--- /dev/null
+++ b/crates/tc2/src/siguiente.rs
@@ -0,0 +1,207 @@
+//! `tc2 next [--json]` (F3-3): lo listo para empezar por dependencias y
+//! prioridad.
+//!
+//! Lee el estado tal como lo conoce este clon ([`estado::leer`]: sin
+//! `fetch` y sin escribir; la lectura fresca es de F3-5b), proyecta cada
+//! `tarea.md` a un [`Nodo`] y calcula el orden con [`grafo`]. El orden de los
+//! hitos sale de los titulos `### Hn — titulo` de la zona manual del
+//! `ROADMAP.md` de la rama de estado (D2 de TASK-030).
+//!
+//! Salida: `next ok listas=... en-marcha=... commit=<sha> [avisos]`, con
+//! `commit=` la foto del estado usada. Avisos, solo si los hay:
+//! `ilegibles=` (`tarea.md` que no se entienden: no se saltan en silencio),
+//! `ciclos=` (las atascadas), `dependencias-inexistentes=`,
+//! `depende-de-descartada=` (`A>B`), `hitos-sin-titulo=` y `duplicadas=`
+//! (IDs repetidos: vale el primero). Sin rama de estado, `rechazada`.
+
+use std::path::Path;
+
+use crate::contrato::{Informe, Resultado, corto};
+use crate::estado::{self, Estado, Fallo};
+use crate::git::Git;
+use crate::grafo::{self, Hitos, Nodo};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::cli as txt_cli;
+use crate::i18n::catalogo::siguiente as txt;
+
+/// Nombre de la orden.
+pub const NOMBRE: &str = "next";
+
+/// `ROADMAP.md` en la raiz de la rama de estado (ADR F1-1, decision 3).
+const ROADMAP: &str = "ROADMAP.md";
+
+/// `next [--json]` en la carpeta actual (sin `--json`, que quita `lib.rs`).
+pub fn orden(args: &[String]) -> Informe {
+    if !args.is_empty() {
+        return Informe::nuevo(NOMBRE, Resultado::Uso)
+            .con("motivo", "argumentos-de-mas")
+            .diagnostico(&txt::USO_NEXT);
+    }
+    match std::env::current_dir() {
+        Ok(dir) => siguiente_en(&Git::nuevo(), &dir),
+        Err(e) => Informe::nuevo(NOMBRE, Resultado::Error)
+            .diagnostico(Mensaje::nuevo(&txt_cli::CARPETA_ACTUAL).con("error", e.to_string())),
+    }
+}
+
+/// `next` sobre el estado que conoce el clon de `dir`.
+pub fn siguiente_en(g: &Git, dir: &Path) -> Informe {
+    let vista = match estado::leer(g, dir) {
+        Ok(v) => v,
+        Err(mut i) => {
+            NOMBRE.clone_into(&mut i.comando);
+            return i;
+        }
+    };
+    let commit = vista.commit().map(corto).unwrap_or_default();
+    match calcular(&vista) {
+        Ok(c) => c.informe(&commit),
+        Err(Fallo::Rechazada {
+            motivo,
+            diagnostico,
+        }) => Informe::nuevo(NOMBRE, Resultado::Rechazada)
+            .con("motivo", motivo)
+            .diagnostico(diagnostico),
+        Err(Fallo::Error(d)) => Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d),
+    }
+}
+
+/// Lo que se calcula sobre el estado.
+#[derive(Debug, Default)]
+struct Calculo {
+    listas: Vec<String>,
+    en_marcha: Vec<String>,
+    ilegibles: Vec<String>,
+    orden: grafo::Orden,
+}
+
+impl Calculo {
+    fn informe(self, commit: &str) -> Informe {
+        let a = self.orden.avisos;
+        let mut i = Informe::nuevo(NOMBRE, Resultado::Ok)
+            .con_lista("listas", self.listas)
+            .con_lista("en-marcha", self.en_marcha)
+            .con("commit", commit);
+        let hitos: Vec<String> = a.hitos_sin_titulo.iter().map(|h| format!("H{h}")).collect();
+        for (clave, v) in [
+            ("ilegibles", self.ilegibles),
+            ("ciclos", self.orden.atascadas),
+            ("dependencias-inexistentes", a.dependencias_inexistentes),
+            ("depende-de-descartada", a.depende_de_descartada),
+            ("hitos-sin-titulo", hitos),
+            ("duplicadas", a.duplicadas),
+        ] {
+            if !v.is_empty() {
+                i = i.con_lista(clave, v);
+            }
+        }
+        i
+    }
+}
+
+/// Listas, en marcha e ilegibles de `estado`.
+fn calcular(estado: &dyn Estado) -> Result<Calculo, Fallo> {
+    let rutas = estado.rutas();
+    let mut nodos = Vec::new();
+    let mut ilegibles = Vec::new();
+    for (ruta, leida) in estado::tareas(estado, &rutas)? {
+        match leida {
+            Ok(d) => nodos.push(Nodo::de(&d)),
+            Err(_) => ilegibles.push(ruta.split('/').next().unwrap_or(&ruta).to_owned()),
+        }
+    }
+    // Detras de las legibles: si un ID se repite, vale la legible. Una
+    // ilegible bloquea a quien dependa de ella.
+    nodos.extend(ilegibles.iter().map(|id| Nodo::ilegible(id)));
+    let hitos = match estado.leer(ROADMAP)? {
+        Some(t) => Hitos::del_roadmap(&t),
+        None => Hitos::default(),
+    };
+    let orden = grafo::orden_de_trabajo(&nodos, &hitos);
+    Ok(Calculo {
+        listas: grafo::listas(&orden, &nodos),
+        en_marcha: grafo::en_marcha(&orden, &nodos),
+        ilegibles,
+        orden,
+    })
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use std::collections::BTreeMap;
+
+    /// Estado en memoria.
+    struct EnMemoria(BTreeMap<String, String>);
+
+    impl Estado for EnMemoria {
+        fn rutas(&self) -> Vec<String> {
+            self.0.keys().cloned().collect()
+        }
+        fn leer(&self, ruta: &str) -> Result<Option<String>, Fallo> {
+            Ok(self.0.get(ruta).cloned())
+        }
+    }
+
+    fn tarea(id: &str, sprint: u32, estado: &str, deps: &[&str]) -> String {
+        let mut t = crate::tarea::Tarea::nueva(id, "t", "2026-01-01");
+        t.sprint = sprint;
+        t.estado = estado.to_owned();
+        t.dependencias = deps.iter().map(|d| (*d).to_owned()).collect();
+        t.escribir(&crate::tarea::Cuerpo::leer(""))
+    }
+
+    /// C4: un `tarea.md` ilegible no se salta en silencio.
+    #[test]
+    fn ilegible_sale_en_ilegibles() {
+        let mut m = BTreeMap::new();
+        m.insert(
+            "TASK-001/tarea.md".into(),
+            tarea("TASK-001", 0, "planificada", &[]),
+        );
+        m.insert("TASK-002/tarea.md".into(), "---\nid: roto\n".into());
+        m.insert("TASK-003/plan.md".into(), "no es un tarea.md".into());
+        let c = calcular(&EnMemoria(m));
+        assert!(c.is_ok(), "{c:?}");
+        let Ok(c) = c else { return };
+        assert_eq!(
+            c.informe("0123456789ab").linea(),
+            // Sin ROADMAP.md, el hito va por su numero y se avisa.
+            "next ok listas=TASK-001 en-marcha= commit=0123456789ab ilegibles=TASK-002 hitos-sin-titulo=H0"
+        );
+    }
+
+    /// El orden de los hitos sale de la zona manual del ROADMAP.md.
+    #[test]
+    fn hitos_del_roadmap_de_la_rama_de_estado() {
+        let mut m = BTreeMap::new();
+        m.insert(
+            "TASK-001/tarea.md".into(),
+            tarea("TASK-001", 1, "planificada", &[]),
+        );
+        m.insert(
+            "TASK-002/tarea.md".into(),
+            tarea("TASK-002", 2, "planificada", &[]),
+        );
+        m.insert(
+            "TASK-003/tarea.md".into(),
+            tarea("TASK-003", 3, "en-curso", &[]),
+        );
+        m.insert(
+            ROADMAP.into(),
+            "<!-- manual:inicio -->\n### H2 — Dos\n### H1 — Uno\n<!-- manual:fin -->\n".into(),
+        );
+        let c = calcular(&EnMemoria(m));
+        let Ok(c) = c else { return };
+        assert_eq!(
+            c.informe("0123456789ab").linea(),
+            "next ok listas=TASK-002,TASK-001 en-marcha=TASK-003 commit=0123456789ab hitos-sin-titulo=H3"
+        );
+    }
+
+    #[test]
+    fn argumentos_de_mas_es_uso() {
+        let i = orden(&["x".to_owned()]);
+        assert_eq!(i.linea(), "next uso motivo=argumentos-de-mas");
+    }
+}
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/01-en-diseno/TASK-030/tarea.md                                                     | 42 ------------------------------------------
 tareas/{01-en-diseno => 02-en-curso}/TASK-030/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-030/planificacion/plan-final.md                 |  0
 tareas/02-en-curso/TASK-030/tarea.md                                                      | 57 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 4 files changed, 57 insertions(+), 42 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/siguiente.rs"
````

````
crates/tc2/tests/siguiente.rs | 208 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 208 insertions(+)
````
