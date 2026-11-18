# Peticion de revision — TASK-083 (ronda 1)

- Tarea: TASK-083 — F3-13 Estimacion por fase de cada tarea al planificar
- Rama revisada: feature/task-083-f3-13-estimacion-por-fase-de-cada-tarea
- Rama base: develop
- Commit revisado (HEAD): 2792c7e1f3c33f6fd9ab426df543f15c6fee3c6f
- Fecha: 2026-11-18
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-083 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-083/tarea.md
- tareas/03-en-revision/TASK-083/planificacion/
- crates/tc2/src/alta.rs
- crates/tc2/src/doctor/estado.rs
- crates/tc2/src/estimacion/estimar.rs
- crates/tc2/src/estimacion/forma.rs
- crates/tc2/src/estimacion/historico.rs
- crates/tc2/src/estimacion/mod.rs
- crates/tc2/src/estimacion/real.rs
- crates/tc2/src/git/mod.rs
- crates/tc2/src/git/puerto.rs
- crates/tc2/src/i18n/catalogo/cli.rs
- crates/tc2/src/i18n/catalogo/estimacion.rs
- crates/tc2/src/i18n/catalogo/mod.rs
- crates/tc2/src/importar/mod.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/tarea/modelo.rs
- crates/tc2/src/toma/mod.rs
- crates/tc2/src/toma/operacion.rs
- crates/tc2/tests/comun/mod.rs
- crates/tc2/tests/estimacion.rs
- crates/tc2/tests/toma.rs

## Commits a revisar (git log develop..HEAD)

````
2792c7e docs(TASK-083): autorrevision y correccion del criterio C1
e9a6879 docs(TASK-083): adendas de F1-17 y F1-4 por la estimacion
a1c4841 test(TASK-083): paridad con poc-f1-17.py y E2E de C1 a C7
bc21575 feat(TASK-083): recalcular en new, import y start, tc2 estimar y doctor
1a5e8fb feat(TASK-083): lector del historico y git log de develop
9387345 feat(TASK-083): forma del campo estimacion, su lectura y su linea
c21b687 feat(TASK-083): estimar, restante y error como funciones puras
3ee6b69 feat(TASK-083): real por fase de una tarea terminada (estimacion::real)
b92ff20 chore(TASK-083): tarea en curso
````

## Diff de tu dominio (20 fichero(s) de 23; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/alta.rs b/crates/tc2/src/alta.rs
index 088e8ca..34288ee 100644
--- a/crates/tc2/src/alta.rs
+++ b/crates/tc2/src/alta.rs
@@ -11,7 +11,8 @@ use std::cell::{Cell, RefCell};
 use std::collections::BTreeSet;
 use std::time::{SystemTime, UNIX_EPOCH};
 
-use crate::estado::{Cambio, Estado, Fallo, Operacion};
+use crate::estado::{Cambio, Estado, Fallo, Leida, Operacion};
+use crate::estimacion::{Commits, Estimacion, Historico, historico};
 use crate::i18n::catalogo::estado as txt;
 use crate::i18n::{Idioma, Mensaje};
 use crate::tarea::texto::{Parecido, parecido, slug};
@@ -265,7 +266,8 @@ impl NoListo {
     }
 }
 
-/// Alta de una tarea: reserva el siguiente ID y escribe su `tarea.md`.
+/// Alta de una tarea: reserva el siguiente ID y escribe su `tarea.md`, con
+/// su estimacion (F3-13).
 #[derive(Debug)]
 pub struct Alta {
     entrada: Entrada,
@@ -273,6 +275,9 @@ pub struct Alta {
     ocupados: BTreeSet<u32>,
     id: Cell<Option<u32>>,
     parecidas: RefCell<Vec<Parecida>>,
+    ahora: u64,
+    commits: Commits,
+    estimacion: RefCell<Option<Estimacion>>,
 }
 
 impl Alta {
@@ -298,6 +303,9 @@ impl Alta {
             ocupados: BTreeSet::new(),
             id: Cell::new(None),
             parecidas: RefCell::new(Vec::new()),
+            ahora: crate::identidad::ahora(),
+            commits: Commits::vacios(),
+            estimacion: RefCell::new(None),
         }
     }
 
@@ -309,6 +317,19 @@ impl Alta {
         self
     }
 
+    /// Los commits de `develop` para el historico de la estimacion (un
+    /// `git log` leido antes de la transaccion, F3-13).
+    #[must_use]
+    pub fn con_commits(mut self, commits: Commits) -> Self {
+        self.commits = commits;
+        self
+    }
+
+    /// La estimacion escrita en el ultimo intento.
+    pub fn estimacion(&self) -> Option<Estimacion> {
+        self.estimacion.borrow().clone()
+    }
+
     /// Rama derivada del ID del ultimo intento (F2-4b).
     pub fn rama(&self) -> Option<String> {
         self.id()
@@ -356,15 +377,22 @@ impl Operacion for Alta {
         let ids = reservar(&rutas, &self.ocupados, 1)?;
         self.id.set(ids.first().copied());
         // Sobre el mismo estado fresco del intento: dos altas a la vez se ven
-        // (la segunda en publicar avisa de la primera, TASK-076 C1).
-        let tareas = tareas(estado, &rutas)?;
+        // (la segunda en publicar avisa de la primera, TASK-076 C1). Una sola
+        // lectura de los `tarea.md` para las parecidas y el historico.
+        let leidas = leidas(estado, &rutas)?;
+        let h = historico(&leidas, &self.commits);
         let a = altas(
-            &tareas,
+            &de_formato2(&leidas),
             &ids,
             std::slice::from_ref(&self.entrada),
             &self.fecha,
+            &Calculo {
+                historico: &h,
+                ahora: self.ahora,
+            },
         )?;
         *self.parecidas.borrow_mut() = a.parecidas;
+        *self.estimacion.borrow_mut() = a.estimaciones.into_iter().next();
         Ok(a.cambios)
     }
 }
@@ -377,18 +405,33 @@ pub(crate) struct Altas {
     pub cambios: Vec<Cambio>,
     /// Tareas abiertas parecidas a alguna entrada, sin repetir, por ID.
     pub parecidas: Vec<Parecida>,
+    /// La estimacion de cada entrada, en su orden (F3-13).
+    pub estimaciones: Vec<Estimacion>,
+}
+
+/// Con que se estima cada tarea nueva (F3-13): el historico, calculado una
+/// vez por intento aunque el lote traiga muchas, y la hora del calculo.
+#[derive(Debug)]
+pub(crate) struct Calculo<'a> {
+    /// Muestras de las terminadas.
+    pub historico: &'a Historico,
+    /// Segundos desde 1970 (UTC).
+    pub ahora: u64,
 }
 
 /// Nucleo puro del alta (F2-4 y F3-2): `entradas[i]` nace con el ID
-/// `ids[i]`. Comprueba la rama derivada de cada una y busca las tareas
-/// abiertas de `tareas` con un titulo parecido.
+/// `ids[i]`. Comprueba la rama derivada de cada una, busca las tareas
+/// abiertas de `tareas` con un titulo parecido y le pone su estimacion
+/// (F3-13, aunque sea `sin-datos`).
 pub(crate) fn altas(
     tareas: &[Tarea],
     ids: &[u32],
     entradas: &[Entrada],
     fecha: &str,
+    calculo: &Calculo<'_>,
 ) -> Result<Altas, Fallo> {
     let mut cambios = Vec::with_capacity(entradas.len());
+    let mut estimaciones = Vec::with_capacity(entradas.len());
     let mut vistas = BTreeSet::new();
     let mut parecidas = Vec::new();
     for (i, e) in entradas.iter().enumerate() {
@@ -398,10 +441,14 @@ pub(crate) fn altas(
                 .ok_or_else(|| Fallo::Error((&crate::i18n::catalogo::estado::SIN_IDS).into()))?,
         );
         comprobar_rama(&id, &e.tipo, &e.titulo)?;
-        cambios.push(Cambio::nuevo(
-            format!("{id}/tarea.md"),
-            tarea_de(&id, e, fecha),
+        let (mut t, cuerpo) = tarea_y_cuerpo(&id, e, fecha);
+        estimaciones.push(crate::estimacion::recalcular(
+            &mut t,
+            &cuerpo,
+            calculo.historico,
+            calculo.ahora,
         ));
+        cambios.push(Cambio::nuevo(format!("{id}/tarea.md"), t.escribir(&cuerpo)));
         for p in parecidas_a(tareas, &e.titulo) {
             if vistas.insert(p.id.clone()) {
                 parecidas.push(p);
@@ -409,7 +456,11 @@ pub(crate) fn altas(
         }
     }
     parecidas.sort_by(|a: &Parecida, b| a.id.cmp(&b.id));
-    Ok(Altas { cambios, parecidas })
+    Ok(Altas {
+        cambios,
+        parecidas,
+        estimaciones,
+    })
 }
 
 /// `n` IDs consecutivos desde [`siguiente_id`]: los de un lote no dejan
@@ -449,19 +500,26 @@ fn es_tarea_md(ruta: &str) -> bool {
     numero(ruta).is_some() && ruta.split_once('/').is_some_and(|(_, r)| r == "tarea.md")
 }
 
-/// Las tareas de formato 2 del estado, en orden de ruta, leidas en un solo
-/// proceso (F2-4b) con [`crate::estado::tareas`]. Los `tarea.md` que no se
-/// leen o son de formato 1 se saltan sin fallar, y las carpetas con sufijo
-/// no cuentan aqui (filtro de siempre del alta).
-pub(crate) fn tareas(estado: &dyn Estado, rutas: &[String]) -> Result<Vec<Tarea>, Fallo> {
-    let mds: Vec<String> = rutas.iter().filter(|r| es_tarea_md(r)).cloned().collect();
-    Ok(crate::estado::tareas(estado, &mds)?
-        .into_iter()
+/// Todos los `tarea.md` del estado, de los dos formatos, en orden de ruta y
+/// leidos en un solo proceso (F2-4b) con [`crate::estado::tareas`]: los
+/// mismos sirven para las parecidas ([`de_formato2`]) y para el historico de
+/// la estimacion (F3-13).
+pub(crate) fn leidas(estado: &dyn Estado, rutas: &[String]) -> Result<Vec<Leida>, Fallo> {
+    crate::estado::tareas(estado, rutas)
+}
+
+/// Las tareas de formato 2 de `leidas`. Los `tarea.md` que no se leen o son
+/// de formato 1 se saltan sin fallar, y las carpetas con sufijo no cuentan
+/// aqui (filtro de siempre del alta).
+pub(crate) fn de_formato2(leidas: &[Leida]) -> Vec<Tarea> {
+    leidas
+        .iter()
+        .filter(|(r, _)| es_tarea_md(r))
         .filter_map(|(_, d)| match d {
-            Ok(Documento::Formato2(t, _)) => Some(t),
+            Ok(Documento::Formato2(t, _)) => Some(t.clone()),
             _ => None,
         })
-        .collect())
+        .collect()
 }
 
 /// Tareas abiertas de `tareas` cuyo titulo se parece a `titulo`.
@@ -583,6 +641,13 @@ pub fn tarea_md(id: &str, titulo: &str, fecha: &str) -> String {
 /// resto en el frontmatter. El tipo y la rama salen del mismo campo: no
 /// pueden divergir.
 fn tarea_de(id: &str, e: &Entrada, fecha: &str) -> String {
+    let (t, cuerpo) = tarea_y_cuerpo(id, e, fecha);
+    t.escribir(&cuerpo)
+}
+
+/// El modelo y el cuerpo de [`tarea_de`], sin escribir (el alta les pone la
+/// estimacion antes, F3-13).
+fn tarea_y_cuerpo(id: &str, e: &Entrada, fecha: &str) -> (Tarea, Cuerpo) {
     let mut t = Tarea::nueva(id, &e.titulo, fecha);
     e.tipo.clone_into(&mut t.tipo);
     t.sprint = e.sprint;
@@ -591,7 +656,7 @@ fn tarea_de(id: &str, e: &Entrada, fecha: &str) -> String {
     t.dependencias.clone_from(&e.dependencias);
     t.lote.clone_from(&e.lote);
     t.rama = Some(rama_de(id, &e.tipo, &e.titulo));
-    t.escribir(&Cuerpo::con_enunciado(&e.objetivo, &e.criterios))
+    (t, Cuerpo::con_enunciado(&e.objetivo, &e.criterios))
 }
 
 /// Fecha de hoy en UTC, `AAAA-MM-DD`, sin dependencias.
diff --git a/crates/tc2/src/doctor/estado.rs b/crates/tc2/src/doctor/estado.rs
index a3f2418..0bc17e0 100644
--- a/crates/tc2/src/doctor/estado.rs
+++ b/crates/tc2/src/doctor/estado.rs
@@ -4,7 +4,9 @@
 //!   `fetch` (lo ultimo traido; D4a de F3-5b: `doctor` solo lee):
 //!   ausente, `falla motivo=sin-rama-de-estado`; una ref que existe y no es
 //!   un commit, `falla motivo=ref-ilegible ref=...`; un `tarea.md` que no se
-//!   entiende, `falla motivo=tarea-ilegible ids=...`.
+//!   entiende, `falla motivo=tarea-ilegible ids=...`. Con el estado leido,
+//!   `estado historico ok usables=N descartadas=M` (F3-13: las terminadas
+//!   que dan muestra a la estimacion y las que no).
 //! - La instalacion es el inventario de `tc2 init` en simulacion, sin
 //!   credenciales (no llama a la API: los rulesets salen `pendiente
 //!   motivo=sin-token` y los mira [`super::plataforma`]). Cada accion que
@@ -70,6 +72,16 @@ pub(super) fn rama(g: &Git, dir: &Path) -> (Vec<Hallazgo>, Option<String>) {
             .con("ids", s.ilegibles.join(",")),
         Err(_) => t(Estado::Falla).motivo("estado-ilegible").con("ref", r),
     });
+    // F3-13: el historico de la estimacion, sin `git log` (doctor solo lee
+    // el estado; los descartes no dependen de los commits).
+    if let Ok(leidas) = estado::tareas(&vista, &estado::Estado::rutas(&vista)) {
+        let h = crate::estimacion::historico(&leidas, &crate::estimacion::Commits::vacios());
+        v.push(
+            Hallazgo::nuevo("estado", "historico", Estado::Ok)
+                .con("usables", h.usables.to_string())
+                .con("descartadas", h.descartadas().to_string()),
+        );
+    }
     (v, commit)
 }
 
diff --git a/crates/tc2/src/estimacion/estimar.rs b/crates/tc2/src/estimacion/estimar.rs
new file mode 100644
index 0000000..25f0e6c
--- /dev/null
+++ b/crates/tc2/src/estimacion/estimar.rs
@@ -0,0 +1,522 @@
+//! Nucleo puro de la estimacion (ADR F1-17, decisiones 3, 4 y 5):
+//! [`estimar`] (cascada de tareas parecidas con mediana y p80 por fase y en
+//! total), [`restante`] (lo que falta desde la fase en curso) y [`error`]
+//! (estimado frente a real).
+//!
+//! - Cascada con al menos [`N_MIN`] muestras por nivel: carril, complejidad
+//!   y tamano; carril y complejidad; carril; y repo. Sin semillas de otros
+//!   proyectos (D3a de TASK-083): si ningun nivel llega, `sin-datos`, nunca
+//!   un numero inventado.
+//! - Carril declarado (D5a): una tarea sin carril solo se compara en `repo`.
+//! - La tarea estimada nunca es muestra de si misma (C7).
+//! - Mediana y p80 como `mediana` y `cuantil` de `poc-f1-17.py` (rango mas
+//!   cercano, el de F1-18).
+
+use super::historico::Muestra;
+use super::real::{FASES, Real};
+use crate::carril::Carril;
+
+/// Muestras minimas de un nivel de la cascada (fijado antes de medir).
+pub const N_MIN: usize = 5;
+
+/// Cuantil que se ensena ademas de la mediana (D2a).
+pub const P80: f64 = 0.8;
+
+/// Nivel de una estimacion sin ningun nivel con [`N_MIN`] muestras.
+pub const SIN_DATOS: &str = "sin-datos";
+
+/// Niveles de la cascada, en orden.
+pub const NIVELES: [&str; 4] = [
+    "carril+complejidad+tamano",
+    "carril+complejidad",
+    "carril",
+    "repo",
+];
+
+/// Lo que se sabe de la tarea al estimarla.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Objetivo {
+    /// Su ID (nunca es muestra de si misma).
+    pub id: String,
+    /// Carril declarado.
+    pub carril: Option<Carril>,
+    /// Complejidad declarada.
+    pub complejidad: Option<String>,
+    /// Casillas de criterios de aceptacion.
+    pub criterios: usize,
+}
+
+/// Mediana y p80, en minutos.
+#[derive(Debug, Clone, Copy, PartialEq)]
+pub struct Rango {
+    /// Mediana.
+    pub mediana: f64,
+    /// Percentil 80 (rango mas cercano).
+    pub p80: f64,
+}
+
+/// Rango por fase; una fase que no se conoce, `None`.
+#[derive(Debug, Clone, Copy, PartialEq)]
+pub struct PorFase {
+    /// Implementacion.
+    pub implementacion: Option<Rango>,
+    /// Rondas.
+    pub rondas: Option<Rango>,
+    /// Cierre.
+    pub cierre: Option<Rango>,
+}
+
+impl PorFase {
+    /// La fase por su nombre.
+    pub fn fase(&self, f: &str) -> Option<Rango> {
+        match f {
+            "implementacion" => self.implementacion,
+            "rondas" => self.rondas,
+            "cierre" => self.cierre,
+            _ => None,
+        }
+    }
+}
+
+/// La estimacion de una tarea (forma del punto 7 de F1-17).
+#[derive(Debug, Clone, PartialEq)]
+pub struct Estimacion {
+    /// Instante del calculo, `AAAA-MM-DDTHH:MM:SSZ`.
+    pub calculada: String,
+    /// Nivel de la cascada usado, o [`SIN_DATOS`].
+    pub nivel: String,
+    /// Muestras del nivel (con `sin-datos`, las disponibles).
+    pub n: usize,
+    /// Rango por fase; `None` con `sin-datos`.
+    pub por_fase: Option<PorFase>,
+    /// Rango del total; `None` con `sin-datos`.
+    pub total: Option<Rango>,
+    /// Estimacion del agente en el plan (D1a); `None` hasta que exista
+    /// `plan` (F4-1).
+    pub agente: Option<u64>,
+}
+
+impl Estimacion {
+    /// Si no hubo tareas parecidas suficientes.
+    pub fn sin_datos(&self) -> bool {
+        self.nivel == SIN_DATOS
+    }
+}
+
+/// Mediana de `xs` (media de las dos centrales si son pares).
+pub fn mediana(xs: &[f64]) -> Option<f64> {
+    let mut v = xs.to_vec();
+    v.sort_by(f64::total_cmp);
+    let n = v.len();
+    match n {
+        0 => None,
+        _ if n % 2 == 1 => v.get(n / 2).copied(),
+        _ => Some((v[n / 2 - 1] + v[n / 2]) / 2.0),
+    }
+}
+
+/// Cuantil `q` por rango mas cercano: el valor en la posicion
+/// `ceil(q * n)` de la lista ordenada (con la misma tolerancia que la PoC).
+pub fn cuantil(xs: &[f64], q: f64) -> Option<f64> {
+    let mut v = xs.to_vec();
+    v.sort_by(f64::total_cmp);
+    if v.is_empty() {
+        return None;
+    }
+    let k = ((q * v.len() as f64) - 1e-9).ceil().max(1.0) as usize;
+    v.get(k.min(v.len()) - 1).copied()
+}
+
+/// Mediana y p80 de una lista no vacia.
+fn rango(xs: &[f64]) -> Option<Rango> {
+    Some(Rango {
+        mediana: mediana(xs)?,
+        p80: cuantil(xs, P80)?,
+    })
+}
+
+/// Tamano por criterios (1-3, 4-6, 7 o mas), el de la PoC.
+fn tamano(criterios: usize) -> u8 {
+    match criterios {
+        0..=3 => 1,
+        4..=6 => 2,
+        _ => 3,
+    }
+}
+
+/// Si `m` es parecida a `obj` en `nivel`. Sin carril declarado solo vale
+/// `repo` (D5a); la complejidad se compara tal cual (`null` con `null`,
+/// como la PoC).
+fn parecida(obj: &Objetivo, m: &Muestra, nivel: &str) -> bool {
+    let carril = obj.carril.is_some() && obj.carril == m.carril;
+    match nivel {
+        "carril+complejidad+tamano" => {
+            carril
+                && obj.complejidad == m.complejidad
+                && tamano(obj.criterios) == tamano(m.criterios)
+        }
+        "carril+complejidad" => carril && obj.complejidad == m.complejidad,
+        "carril" => carril,
+        _ => true,
+    }
+}
+
+/// Las muestras que puede usar `obj`: todas menos las que la contienen
+/// (C7: nunca es muestra de si misma).
+fn disponibles<'a>(obj: &Objetivo, muestras: &'a [Muestra]) -> Vec<&'a Muestra> {
+    muestras
+        .iter()
+        .filter(|m| !m.miembros.contains(&obj.id) && m.id != obj.id)
+        .collect()
+}
+
+/// El primer nivel de la cascada con [`N_MIN`] muestras parecidas y esas
+/// muestras; `None` si ninguno llega.
+pub fn parecidas<'a>(
+    obj: &Objetivo,
+    muestras: &'a [Muestra],
+) -> Option<(&'static str, Vec<&'a Muestra>)> {
+    let d = disponibles(obj, muestras);
+    NIVELES.into_iter().find_map(|nivel| {
+        let ms: Vec<&Muestra> = d
+            .iter()
+            .copied()
+            .filter(|m| parecida(obj, m, nivel))
+            .collect();
+        (ms.len() >= N_MIN).then_some((nivel, ms))
+    })
+}
+
+/// `estimar(tarea, muestras) -> Estimacion` (funcion pura): mediana y p80
+/// por fase y en total del primer nivel con [`N_MIN`] muestras; si ninguno
+/// llega, `sin-datos` con las muestras disponibles en `n`.
+pub fn estimar(obj: &Objetivo, muestras: &[Muestra], calculada: &str) -> Estimacion {
+    let Some((nivel, ms)) = parecidas(obj, muestras) else {
+        return Estimacion {
+            calculada: calculada.to_owned(),
+            nivel: SIN_DATOS.to_owned(),
+            n: disponibles(obj, muestras).len(),
+            por_fase: None,
+            total: None,
+            agente: None,
+        };
+    };
+    let de = |f: fn(&Real) -> f64| rango(&ms.iter().map(|m| f(&m.real)).collect::<Vec<_>>());
+    Estimacion {
+        calculada: calculada.to_owned(),
+        nivel: nivel.to_owned(),
+        n: ms.len(),
+        por_fase: Some(PorFase {
+            implementacion: de(|r| r.implementacion),
+            rondas: de(|r| r.rondas),
+            cierre: de(|r| r.cierre),
+        }),
+        total: de(Real::total),
+        agente: None,
+    }
+}
+
+/// Fase en curso de una tarea abierta (para [`restante`]).
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Fase {
+    /// Antes de `start`: faltan las tres fases medibles.
+    Diseno,
+    /// De `start` a la primera `review`.
+    Implementacion,
+    /// De la primera `review` a `finish`.
+    Rondas,
+}
+
+/// Lo que falta de una tarea (punto 5 de F1-17).
+#[derive(Debug, Clone, Copy, PartialEq)]
+pub struct Restante {
+    /// Mediana de lo que falta (con `desbordada`, cota inferior).
+    pub mediana: f64,
+    /// p80 de lo que falta (sin sentido con `desbordada`: no se ensena).
+    pub p80: f64,
+    /// Muestras usadas.
+    pub n: usize,
+    /// Si se condiciono a las parecidas que ya pasaron de lo transcurrido.
+    pub condicionada: bool,
+    /// Si la fase en curso va por encima de sus parecidas: lo que se da es
+    /// solo una cota inferior (las fases siguientes).
+    pub desbordada: bool,
+}
+
+/// `restante(parecidas, fase, transcurrido)`: la distribucion de lo que
+/// falta entre las parecidas. Si al menos [`N_MIN`] ya pasaron de los
+/// minutos `transcurrido` en la fase en curso, se condiciona a esas; si no,
+/// la fase va desbordada y la cifra es una cota inferior. Sin parecidas,
+/// `None`.
+pub fn restante(parecidas: &[&Muestra], fase: Fase, transcurrido: f64) -> Option<Restante> {
+    if parecidas.is_empty() {
+        return None;
+    }
+    let (siguientes, actual): (&[&str], Option<&str>) = match fase {
+        Fase::Diseno => (&FASES, None),
+        Fase::Implementacion => (&FASES, Some("implementacion")),
+        Fase::Rondas => (&FASES[1..], Some("rondas")),
+    };
+    let real = |m: &Muestra, f: &str| m.real.fase(f).unwrap_or_default();
+    let vivas: Vec<&Muestra> = parecidas
+        .iter()
+        .copied()
+        .filter(|m| actual.is_none_or(|a| real(m, a) > transcurrido))
+        .collect();
+    let condicionada = vivas.len() >= N_MIN && actual.is_some();
+    let base: Vec<&Muestra> = if vivas.len() >= N_MIN {
+        vivas.clone()
+    } else {
+        parecidas.to_vec()
+    };
+    let xs: Vec<f64> = base
+        .iter()
+        .map(|m| {
+            let mut r: f64 = siguientes.iter().map(|f| real(m, f)).sum();
+            if let Some(a) = actual {
+                r -= transcurrido.min(real(m, a));
+            }
+            r.max(0.0)
+        })
+        .collect();
+    Some(Restante {
+        mediana: mediana(&xs)?,
+        p80: cuantil(&xs, P80)?,
+        n: base.len(),
+        condicionada,
+        desbordada: actual.is_some() && vivas.len() < N_MIN,
+    })
+}
+
+/// Error de una fase (o del total): estimado (la mediana guardada) frente a
+/// real.
+#[derive(Debug, Clone, Copy, PartialEq)]
+pub struct ErrorFase {
+    /// `implementacion`, `rondas`, `cierre` o `total`.
+    pub fase: &'static str,
+    /// Mediana estimada.
+    pub estimado: f64,
+    /// Real.
+    pub real: f64,
+    /// |estimado - real|, en minutos.
+    pub absoluto: f64,
+    /// |estimado - real| / real (real de al menos 1 min, como la PoC).
+    pub relativo: f64,
+    /// estimado / real (real de al menos 1 min).
+    pub razon: f64,
+}
+
+/// El error de `e` frente a `r` por fase y en total, de las que `e` tiene
+/// cifra (D4a de TASK-083: funcion pura; la escritura al cerrar es de F8-7).
+pub fn error(e: &Estimacion, r: &Real) -> Vec<ErrorFase> {
+    let mut v = Vec::new();
+    let mut uno = |fase: &'static str, est: Option<Rango>, real: f64| {
+        if let Some(est) = est {
+            let base = real.max(1.0);
+            let absoluto = (est.mediana - real).abs();
+            v.push(ErrorFase {
+                fase,
+                estimado: est.mediana,
+                real,
+                absoluto,
+                relativo: absoluto / base,
+                razon: est.mediana / base,
+            });
+        }
+    };
+    if let Some(p) = &e.por_fase {
+        for f in FASES {
+            uno(f, p.fase(f), r.fase(f).unwrap_or_default());
+        }
+    }
+    uno("total", e.total, r.total());
+    v
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn muestra(
+        id: &str,
+        carril: Option<Carril>,
+        compl: Option<&str>,
+        crit: usize,
+        imp: f64,
+    ) -> Muestra {
+        Muestra {
+            id: id.to_owned(),
+            miembros: vec![id.to_owned()],
+            carril,
+            complejidad: compl.map(str::to_owned),
+            criterios: crit,
+            real: Real {
+                implementacion: imp,
+                rondas: 10.0,
+                cierre: 5.0,
+            },
+            start: 0,
+            finish: 0,
+        }
+    }
+
+    fn obj(carril: Option<Carril>) -> Objetivo {
+        Objetivo {
+            id: "TASK-100".to_owned(),
+            carril,
+            complejidad: Some("media".to_owned()),
+            criterios: 2,
+        }
+    }
+
+    const F: Option<Carril> = Some(Carril::Funcionalidad);
+
+    #[test]
+    fn mediana_y_p80_por_rango_mas_cercano() {
+        assert_eq!(mediana(&[3.0, 1.0, 2.0]), Some(2.0));
+        assert_eq!(mediana(&[4.0, 1.0, 2.0, 3.0]), Some(2.5));
+        assert_eq!(mediana(&[]), None);
+        let seis = [10.0, 20.0, 30.0, 40.0, 50.0, 60.0];
+        assert_eq!(cuantil(&seis, P80), Some(50.0));
+        assert_eq!(cuantil(&seis[..5], P80), Some(40.0));
+        assert_eq!(cuantil(&[7.0], P80), Some(7.0));
+        assert_eq!(cuantil(&[], P80), None);
+    }
+
+    /// C2 y C3: la cascada baja de nivel con menos de 5 y, sin ninguno,
+    /// `sin-datos` sin cifras.
+    #[test]
+    fn cascada_y_sin_datos() {
+        let mut ms: Vec<Muestra> = (0..4)
+            .map(|i| {
+                muestra(
+                    &format!("TASK-{i:03}"),
+                    F,
+                    Some("media"),
+                    2,
+                    10.0 * f64::from(i + 1),
+                )
+            })
+            .collect();
+        let e = estimar(&obj(F), &ms, "t");
+        assert_eq!((e.nivel.as_str(), e.n, e.total), (SIN_DATOS, 4, None));
+        assert!(e.por_fase.is_none() && e.sin_datos());
+        ms.push(muestra("TASK-010", F, Some("alta"), 2, 50.0));
+        let e = estimar(&obj(F), &ms, "t");
+        assert_eq!((e.nivel.as_str(), e.n), ("carril", 5));
+        assert_eq!(
+            e.por_fase.and_then(|p| p.implementacion),
+            Some(Rango {
+                mediana: 30.0,
+                p80: 40.0
+            })
+        );
+        ms.push(muestra("TASK-011", F, Some("media"), 9, 60.0));
+        let e = estimar(&obj(F), &ms, "t");
+        assert_eq!((e.nivel.as_str(), e.n), ("carril+complejidad", 5));
+        ms.push(muestra("TASK-012", F, Some("media"), 1, 60.0));
+        let e = estimar(&obj(F), &ms, "t");
+        assert_eq!((e.nivel.as_str(), e.n), ("carril+complejidad+tamano", 5));
+        // Sin carril declarado (D5a), solo `repo`.
+        let e = estimar(&obj(None), &ms, "t");
+        assert_eq!((e.nivel.as_str(), e.n), ("repo", 7));
+        // Las muestras sin carril no cuentan en los niveles de carril.
+        let sin: Vec<Muestra> = (0..6)
+            .map(|i| muestra(&format!("TASK-{i:03}"), None, Some("media"), 2, 5.0))
+            .collect();
+        assert_eq!(estimar(&obj(None), &sin, "t").nivel, "repo");
+    }
+
+    /// C7: la tarea estimada nunca es muestra de si misma, ni sola ni dentro
+    /// de un grupo de troceo.
+    #[test]
+    fn c7_nunca_es_muestra_de_si_misma() {
+        let mut ms: Vec<Muestra> = (0..4)
+            .map(|i| muestra(&format!("TASK-{i:03}"), F, Some("media"), 2, 10.0))
+            .collect();
+        ms.push(muestra("TASK-100", F, Some("media"), 2, 99.0));
+        let e = estimar(&obj(F), &ms, "t");
+        assert_eq!((e.nivel.as_str(), e.n), (SIN_DATOS, 4));
+        let mut grupo = muestra("TASK-050+TASK-100", F, Some("media"), 2, 99.0);
+        grupo.miembros = vec!["TASK-050".into(), "TASK-100".into()];
+        ms.pop();
+        ms.push(grupo);
+        assert_eq!(estimar(&obj(F), &ms, "t").n, 4);
+    }
+
+    /// Punto 5 de F1-17: condicionada a las que pasaron de lo transcurrido;
+    /// con menos de 5, desbordada y cota inferior.
+    #[test]
+    fn restante_condicionado_y_desbordado() {
+        let ms: Vec<Muestra> = [10.0, 20.0, 30.0, 40.0, 50.0, 60.0]
+            .iter()
+            .enumerate()
+            .map(|(i, x)| muestra(&format!("TASK-{i:03}"), F, None, 2, *x))
+            .collect();
+        let p: Vec<&Muestra> = ms.iter().collect();
+        // Diseno: todo lo que falta (imp + 15).
+        let r = restante(&p, Fase::Diseno, 0.0);
+        assert_eq!(
+            r.map(|r| (r.mediana, r.p80, r.condicionada, r.desbordada)),
+            Some((50.0, 65.0, false, false))
+        );
+        // 5 minutos en implementacion: pasan las 6, condicionada.
+        let r = restante(&p, Fase::Implementacion, 5.0);
+        assert_eq!(
+            r.map(|r| (r.n, r.condicionada, r.desbordada)),
+            Some((6, true, false))
+        );
+        // 45 minutos: solo 2 por encima, desbordada; la cota sale de todas.
+        let r = restante(&p, Fase::Implementacion, 45.0);
+        assert_eq!(r.map(|r| (r.n, r.desbordada)), Some((6, true)));
+        // En rondas, lo que falta son rondas y cierre.
+        let r = restante(&p, Fase::Rondas, 0.0);
+        assert_eq!(r.map(|r| r.mediana), Some(15.0));
+        assert_eq!(restante(&[], Fase::Rondas, 0.0), None);
+    }
+
+    /// C6: [30, 52] guardado frente a 75 de real: 45 min, 0,60 y 0,40.
+    #[test]
+    fn c6_error_absoluto_relativo_y_razon() {
+        let e = Estimacion {
+            calculada: "t".into(),
+            nivel: "carril".into(),
+            n: 5,
+            por_fase: Some(PorFase {
+                implementacion: Some(Rango {
+                    mediana: 30.0,
+                    p80: 52.0,
+                }),
+                rondas: None,
+                cierre: None,
+            }),
+            total: Some(Rango {
+                mediana: 69.0,
+                p80: 90.0,
+            }),
+            agente: None,
+        };
+        let r = Real {
+            implementacion: 75.0,
+            rondas: 30.0,
+            cierre: 10.0,
+        };
+        let v = error(&e, &r);
+        assert_eq!(v.len(), 2);
+        assert_eq!(v[0].fase, "implementacion");
+        assert_eq!(v[0].absoluto, 45.0);
+        assert!((v[0].relativo - 0.6).abs() < 1e-12);
+        assert!((v[0].razon - 0.4).abs() < 1e-12);
+        assert_eq!(
+            (v[1].fase, v[1].absoluto, v[1].real),
+            ("total", 46.0, 115.0)
+        );
+        // Real de menos de 1 min cuenta 1.
+        let r0 = Real {
+            implementacion: 0.0,
+            rondas: 0.0,
+            cierre: 0.0,
+        };
+        assert_eq!(error(&e, &r0)[0].relativo, 30.0);
+    }
+}
diff --git a/crates/tc2/src/estimacion/forma.rs b/crates/tc2/src/estimacion/forma.rs
new file mode 100644
index 0000000..8f994cd
--- /dev/null
+++ b/crates/tc2/src/estimacion/forma.rs
@@ -0,0 +1,256 @@
+//! La forma del campo `estimacion` de `tarea.md` (ADR F1-17, punto 7), su
+//! lectura, la linea para la persona y las claves de la linea de contrato.
+//!
+//! ```yaml
+//! estimacion:
+//!   calculada: "2026-11-15T12:00:00Z"
+//!   nivel: "carril+complejidad+tamano"
+//!   n: 17
+//!   por_fase: {implementacion: [21, 52], rondas: [16, 34], cierre: [6, 12]}
+//!   total: [50, 96]
+//!   agente: null
+//! ```
+//!
+//! Los pares son [mediana, p80] en minutos enteros (redondeo al mas
+//! cercano). Con `nivel: "sin-datos"`, `por_fase: null` y `total: null`.
+
+use serde::Deserialize;
+
+use super::estimar::{Estimacion, N_MIN, PorFase, Rango};
+use super::real::FASES;
+use crate::contrato::Valor;
+use crate::tarea::yaml;
+
+/// Minutos enteros de una cifra (redondeo al mas cercano).
+pub fn minutos(x: f64) -> u64 {
+    let r = x.round();
+    if r.is_finite() && r > 0.0 {
+        r as u64
+    } else {
+        0
+    }
+}
+
+/// `[mediana, p80]` en minutos.
+fn par(r: Rango) -> String {
+    format!("[{}, {}]", minutos(r.mediana), minutos(r.p80))
+}
+
+impl Estimacion {
+    /// La entrada `estimacion:` del frontmatter, con su salto final: lo que
+    /// guarda [`crate::tarea::Tarea::poner_estimacion`].
+    pub fn yaml(&self) -> String {
+        let por_fase = match &self.por_fase {
+            None => "null".to_owned(),
+            Some(p) => {
+                let v: Vec<String> = FASES
+                    .iter()
+                    .filter_map(|f| p.fase(f).map(|r| format!("{f}: {}", par(r))))
+                    .collect();
+                format!("{{{}}}", v.join(", "))
+            }
+        };
+        format!(
+            "estimacion:\n  calculada: {}\n  nivel: {}\n  n: {}\n  por_fase: {por_fase}\n  total: {}\n  agente: {}\n",
+            yaml::cadena(&self.calculada),
+            yaml::cadena(&self.nivel),
+            self.n,
+            self.total.map_or_else(|| "null".to_owned(), par),
+            yaml::numero(self.agente),
+        )
+    }
+
+    /// Lee la entrada cruda `estimacion: ...` (la de
+    /// [`crate::tarea::Tarea::estimacion`]). Las claves que faltan valen
+    /// `null` (o `0`/vacio); una clave desconocida o un tipo que no es el
+    /// suyo es error.
+    pub fn leer(crudo: &str) -> Result<Self, String> {
+        #[derive(Deserialize)]
+        #[serde(deny_unknown_fields)]
+        struct Fases {
+            implementacion: Option<(f64, f64)>,
+            rondas: Option<(f64, f64)>,
+            cierre: Option<(f64, f64)>,
+        }
+        #[derive(Deserialize)]
+        #[serde(deny_unknown_fields)]
+        struct Crudo {
+            calculada: Option<String>,
+            nivel: Option<String>,
+            n: Option<u64>,
+            por_fase: Option<Fases>,
+            total: Option<(f64, f64)>,
+            agente: Option<u64>,
+        }
+        #[derive(Deserialize)]
+        struct Entrada {
+            estimacion: Option<Crudo>,
+        }
+        let r = |p: Option<(f64, f64)>| p.map(|(mediana, p80)| Rango { mediana, p80 });
+        let e: Entrada = yaml::decodificar(crudo).map_err(|e| e.to_string())?;
+        let c = e.estimacion.ok_or_else(|| "estimacion: null".to_owned())?;
+        Ok(Self {
+            calculada: c.calculada.unwrap_or_default(),
+            nivel: c.nivel.unwrap_or_default(),
+            n: usize::try_from(c.n.unwrap_or_default()).unwrap_or(usize::MAX),
+            por_fase: c.por_fase.map(|f| PorFase {
+                implementacion: r(f.implementacion),
+                rondas: r(f.rondas),
+                cierre: r(f.cierre),
+            }),
+            total: r(c.total),
+            agente: c.agente,
+        })
+    }
+
+    /// La linea para la persona: `estimacion: ≈M min (p80 P), n=K,
+    /// nivel=X` o, sin datos, `estimacion: sin tareas parecidas (n=k<5)`.
+    /// Es un rango de tareas parecidas, no una estimacion calibrada (D8a).
+    pub fn linea(&self) -> String {
+        match self.total {
+            Some(t) if !self.sin_datos() => format!(
+                "estimacion: ≈{} min (p80 {}), n={}, nivel={}",
+                minutos(t.mediana),
+                minutos(t.p80),
+                self.n,
+                self.nivel
+            ),
+            _ => format!("estimacion: sin tareas parecidas (n={}<{N_MIN})", self.n),
+        }
+    }
+
+    /// Las claves de la linea de contrato: `estimacion=<mediana>|sin-datos`,
+    /// `estimacion-p80=<p80>` (con datos), `estimacion-n=<n>` y
+    /// `estimacion-nivel=<nivel>`.
+    pub fn claves(&self) -> Vec<(String, Valor)> {
+        let t = |s: String| Valor::Texto(s);
+        let mut v = Vec::new();
+        match self.total {
+            Some(r) if !self.sin_datos() => {
+                v.push(("estimacion".to_owned(), t(minutos(r.mediana).to_string())));
+                v.push(("estimacion-p80".to_owned(), t(minutos(r.p80).to_string())));
+            }
+            _ => v.push((
+                "estimacion".to_owned(),
+                t(super::estimar::SIN_DATOS.to_owned()),
+            )),
+        }
+        v.push(("estimacion-n".to_owned(), t(self.n.to_string())));
+        v.push(("estimacion-nivel".to_owned(), t(self.nivel.clone())));
+        v
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn con_datos() -> Estimacion {
+        Estimacion {
+            calculada: "2026-11-15T12:00:00Z".into(),
+            nivel: "carril+complejidad+tamano".into(),
+            n: 17,
+            por_fase: Some(PorFase {
+                implementacion: Some(Rango {
+                    mediana: 21.4,
+                    p80: 52.0,
+                }),
+                rondas: Some(Rango {
+                    mediana: 15.5,
+                    p80: 34.0,
+                }),
+                cierre: Some(Rango {
+                    mediana: 6.0,
+                    p80: 12.0,
+                }),
+            }),
+            total: Some(Rango {
+                mediana: 49.5,
+                p80: 96.0,
+            }),
+            agente: None,
+        }
+    }
+
+    /// La forma del punto 7 de F1-17, byte a byte, y su lectura (enteros).
+    #[test]
+    fn forma_de_f1_17_ida_y_vuelta() {
+        let e = con_datos();
+        let y = e.yaml();
+        assert_eq!(
+            y,
+            "estimacion:\n  calculada: \"2026-11-15T12:00:00Z\"\n  nivel: \"carril+complejidad+tamano\"\n  n: 17\n  por_fase: {implementacion: [21, 52], rondas: [16, 34], cierre: [6, 12]}\n  total: [50, 96]\n  agente: null\n"
+        );
+        let l = Estimacion::leer(&y);
+        let redondeada = |x: f64| x.round();
+        assert_eq!(
+            l.as_ref().map(|l| l.total.map(|t| t.mediana)),
+            Ok(Some(redondeada(49.5)))
+        );
+        assert_eq!(l.as_ref().map(|l| l.n), Ok(17));
+        assert_eq!(l.map(|l| l.yaml()), Ok(y));
+    }
+
+    /// C3: sin datos, `por_fase: null`, `total: null` y la linea lo dice
+    /// sin cifras.
+    #[test]
+    fn c3_sin_datos_sin_cifras() {
+        let e = Estimacion {
+            calculada: "2026-11-15T12:00:00Z".into(),
+            nivel: "sin-datos".into(),
+            n: 3,
+            por_fase: None,
+            total: None,
+            agente: None,
+        };
+        assert_eq!(
+            e.yaml(),
+            "estimacion:\n  calculada: \"2026-11-15T12:00:00Z\"\n  nivel: \"sin-datos\"\n  n: 3\n  por_fase: null\n  total: null\n  agente: null\n"
+        );
+        assert_eq!(e.linea(), "estimacion: sin tareas parecidas (n=3<5)");
+        let claves: Vec<String> = e
+            .claves()
+            .iter()
+            .map(|(k, v)| match v {
+                Valor::Texto(t) => format!("{k}={t}"),
+                Valor::Lista(_) => String::new(),
+            })
+            .collect();
+        assert_eq!(
+            claves,
+            [
+                "estimacion=sin-datos",
+                "estimacion-n=3",
+                "estimacion-nivel=sin-datos"
+            ]
+        );
+        assert_eq!(Estimacion::leer(&e.yaml()), Ok(e));
+        assert_eq!(
+            con_datos().linea(),
+            "estimacion: ≈50 min (p80 96), n=17, nivel=carril+complejidad+tamano"
+        );
+    }
+
+    #[test]
+    fn lectura_tolerante_pero_estricta_en_tipos() {
+        let parcial = "estimacion:\n  calculada: \"x\"\n  por_fase: {implementacion: [21, 52]}\n  total: [50, 96]\n";
+        let e = Estimacion::leer(parcial);
+        assert_eq!(
+            e.as_ref()
+                .map(|e| (e.nivel.as_str(), e.por_fase.and_then(|p| p.rondas))),
+            Ok(("", None))
+        );
+        for malo in [
+            "estimacion: null\n",
+            "estimacion:\n  otra: 1\n",
+            "estimacion:\n  total: [1]\n",
+            "estimacion:\n  n: \"x\"\n",
+            "estimacion: [\n",
+        ] {
+            assert!(Estimacion::leer(malo).is_err(), "{malo}");
+        }
+        assert_eq!(minutos(-3.0), 0);
+        assert_eq!(minutos(f64::NAN), 0);
+        assert_eq!(minutos(29.5), 30);
+    }
+}
diff --git a/crates/tc2/src/estimacion/historico.rs b/crates/tc2/src/estimacion/historico.rs
new file mode 100644
index 0000000..8ff94cf
--- /dev/null
+++ b/crates/tc2/src/estimacion/historico.rs
@@ -0,0 +1,542 @@
+//! Lector unico del historico (ADR F1-17, decision 1): de los `tarea.md`
+//! terminados de la rama de estado (los dos formatos) y de un `git log` de
+//! `develop`, las muestras con su real por fase y los descartes por motivo.
+//!
+//! - Los commits de una tarea son los que la nombran en el asunto
+//!   (`TASK-NNN`, sin mayusculas) y los de su rama, `M^1..M^2` del merge
+//!   `merge(feature|fix): feature|fix/task-nnn-...` de la 0.16 (la regla de
+//!   `poc-f1-17.py`).
+//! - Las partes de una tarea troceada son una sola muestra (la regla textual
+//!   sobre `## Objetivo` de la PoC).
+//! - Carril: el declarado (D5a de TASK-083); una tarea de la 0.16 no lo
+//!   tiene y solo cuenta en el nivel `repo`.
+//! - Nunca escribe nada: lo derivado va aparte.
+
+use std::collections::{BTreeMap, HashMap, HashSet};
+
+use super::real::{self, Real};
+use crate::carril::Carril;
+use crate::estado::Leida;
+use crate::tarea::{Cuerpo, Documento};
+
+/// Estado de una tarea terminada.
+const TERMINADA: &str = "terminada";
+
+/// Los commits de `develop` por tarea, de un solo `git log`.
+#[derive(Debug, Clone, Default)]
+pub struct Commits {
+    /// Instantes de los commits cuyo asunto nombra cada tarea.
+    asuntos: HashMap<String, Vec<i64>>,
+    /// Padres (primero, segundo) del merge de la rama de cada tarea.
+    merges: HashMap<String, (String, String)>,
+    /// Padres e instante de cada commit.
+    grafo: HashMap<String, (Vec<String>, i64)>,
+}
+
+/// Si `c` es de palabra (`\w` de Python).
+fn de_palabra(c: char) -> bool {
+    c.is_alphanumeric() || c == '_'
+}
+
+/// `TASK-NNN` (con tres cifras al menos) de los `\bTASK-(\d+)\b` de `texto`,
+/// sin mayusculas.
+fn ids_del_asunto(texto: &str) -> Vec<String> {
+    let mut v = Vec::new();
+    let bajo = texto.to_ascii_lowercase();
+    let mut desde = 0;
+    while let Some(i) = bajo.get(desde..).and_then(|r| r.find("task-")) {
+        let ini = desde + i;
+        desde = ini + 5;
+        if bajo[..ini].chars().next_back().is_some_and(de_palabra) {
+            continue;
+        }
+        let cifras: String = bajo[ini + 5..]
+            .chars()
+            .take_while(char::is_ascii_digit)
+            .collect();
+        if cifras.is_empty() {
+            continue;
+        }
+        let tras = &bajo[ini + 5 + cifras.len()..];
+        if tras.chars().next().is_some_and(de_palabra) {
+            continue;
+        }
+        if let Ok(n) = cifras.parse::<u64>() {
+            let id = format!("TASK-{n:03}");
+            if !v.contains(&id) {
+                v.push(id);
+            }
+        }
+    }
+    v
+}
+
+/// El ID del asunto de un merge de rama de tarea de la 0.16:
+/// `merge(feature|fix): feature|fix/task-<cifras>-...`.
+fn id_de_merge(asunto: &str) -> Option<String> {
+    let resto = ["merge(feature): ", "merge(fix): "]
+        .iter()
+        .find_map(|p| asunto.strip_prefix(p))?;
+    let resto = ["feature/task-", "fix/task-"]
+        .iter()
+        .find_map(|p| resto.strip_prefix(p))?;
+    let cifras: String = resto.chars().take_while(char::is_ascii_digit).collect();
+    if cifras.is_empty() || !resto[cifras.len()..].starts_with('-') {
+        return None;
+    }
+    cifras.parse::<u64>().ok().map(|n| format!("TASK-{n:03}"))
+}
+
+impl Commits {
+    /// Sin commits (sin `develop` o sin `git log`).
+    pub fn vacios() -> Self {
+        Self::default()
+    }
+
+    /// Lee la salida de `git log --format=%H%x1f%P%x1f%ct%x1f%s` (de la mas
+    /// nueva a la mas vieja). Las lineas que no tienen esa forma se saltan.
+    pub fn de_log(texto: &str) -> Self {
+        let mut c = Self::default();
+        for l in texto.lines() {
+            let mut p = l.splitn(4, '\u{1f}');
+            let (Some(sha), Some(padres), Some(ct), Some(asunto)) =
+                (p.next(), p.next(), p.next(), p.next())
+            else {
+                continue;
+            };
+            let Ok(ts) = ct.trim().parse::<i64>() else {
+                continue;
+            };
+            let padres: Vec<String> = padres.split_whitespace().map(str::to_owned).collect();
+            for id in ids_del_asunto(asunto) {
+                c.asuntos.entry(id).or_default().push(ts);
+            }
+            if let (Some(id), [p1, p2]) = (id_de_merge(asunto), padres.as_slice()) {
+                c.merges.entry(id).or_insert((p1.clone(), p2.clone()));
+            }
+            c.grafo.insert(sha.to_owned(), (padres, ts));
+        }
+        c
+    }
+
+    /// Antepasados de `sha` (incluido) en el grafo leido.
+    fn antepasados(&self, sha: &str, fuera: &HashSet<String>) -> Vec<String> {
+        let mut vistos: HashSet<String> = HashSet::new();
+        let mut pila = vec![sha.to_owned()];
+        let mut orden = Vec::new();
+        while let Some(s) = pila.pop() {
+            if fuera.contains(&s) || !vistos.insert(s.clone()) {
+                continue;
+            }
+            if let Some((padres, _)) = self.grafo.get(&s) {
+                pila.extend(padres.iter().cloned());
+                orden.push(s);
+            }
+        }
+        orden
+    }
+
+    /// Instantes de los commits de la tarea `id`: los que la nombran en el
+    /// asunto y los de su rama (`M^1..M^2`).
+    pub fn de_tarea(&self, id: &str) -> Vec<i64> {
+        let mut v = self.asuntos.get(id).cloned().unwrap_or_default();
+        if let Some((p1, p2)) = self.merges.get(id) {
+            let base: HashSet<String> = self.antepasados(p1, &HashSet::new()).into_iter().collect();
+            v.extend(
+                self.antepasados(p2, &base)
+                    .iter()
+                    .filter_map(|s| self.grafo.get(s).map(|(_, ts)| *ts)),
+            );
+        }
+        v.sort_unstable();
+        v
+    }
+}
+
+/// Una muestra: una tarea terminada (o un grupo de troceo) con su real.
+#[derive(Debug, Clone, PartialEq)]
+pub struct Muestra {
+    /// ID, o los IDs del grupo unidos por `+` (por orden de `start`).
+    pub id: String,
+    /// Las tareas de la muestra.
+    pub miembros: Vec<String>,
+    /// Carril declarado (el mayor del grupo); `None` en la 0.16.
+    pub carril: Option<Carril>,
+    /// Complejidad (la de la primera del grupo).
+    pub complejidad: Option<String>,
+    /// Casillas de `## Criterios de aceptacion` (sumadas en un grupo).
+    pub criterios: usize,
+    /// Real por fase (sumado en un grupo).
+    pub real: Real,
+    /// `start` de la primera.
+    pub start: i64,
+    /// `finish` de la ultima.
+    pub finish: i64,
+}
+
+/// Lo que sale del lector: muestras y descartes.
+#[derive(Debug, Clone, Default, PartialEq)]
+pub struct Historico {
+    /// Muestras, por orden de `start`.
+    pub muestras: Vec<Muestra>,
+    /// Tareas terminadas que dan muestra (antes de agrupar).
+    pub usables: usize,
+    /// IDs descartados por motivo.
+    pub descartes: BTreeMap<&'static str, Vec<String>>,
+    /// Filas de las tablas que no se entendieron.
+    pub filas_malas: usize,
+}
+
+impl Historico {
+    /// Tareas terminadas descartadas.
+    pub fn descartadas(&self) -> usize {
+        self.descartes.values().map(Vec::len).sum()
+    }
+}
+
+/// Casillas `- [ ]` o `- [x]` de `## Criterios de aceptacion` (el tamano de
+/// F1-17, divergencia 7).
+pub fn criterios(cuerpo: &Cuerpo) -> usize {
+    cuerpo.seccion("Criterios de aceptacion").map_or(0, |s| {
+        s.texto
+            .lines()
+            .filter(|l| ["- [ ]", "- [x]", "- [X]"].iter().any(|p| l.starts_with(p)))
+            .count()
+    })
+}
+
+/// `TASK-<cifras>` al principio de `s`, tal cual (sin normalizar, como la
+/// PoC).
+fn id_al_principio(s: &str) -> Option<&str> {
+    let resto = s.strip_prefix("TASK-")?;
+    let n = resto.bytes().take_while(u8::is_ascii_digit).count();
+    (n > 0).then(|| &s[..5 + n])
+}
+
+/// Posiciones (en bytes) donde empieza `aguja` en `texto`.
+fn posiciones<'a>(texto: &'a str, aguja: &'a str) -> impl Iterator<Item = usize> + 'a {
+    texto.match_indices(aguja).map(|(i, _)| i)
+}
+
+/// Si en `i` hay limite de palabra antes (como `\b` delante de una letra).
+fn limite_antes(texto: &str, i: usize) -> bool {
+    !texto[..i].chars().next_back().is_some_and(de_palabra)
+}
+
+/// IDs con los que se agrupa una tarea por lo que dice su objetivo (la
+/// regla textual de `troceo()` en la PoC, sin regex).
+pub fn troceo(id: &str, objetivo: &str) -> HashSet<String> {
+    let mut ids: HashSet<String> = HashSet::new();
+    let mut tras = |i: usize, prefijo: &str| {
+        if let Some(t) = objetivo.get(i + prefijo.len()..).and_then(id_al_principio) {
+            ids.insert(t.to_owned());
+        }
+    };
+    let con_limite = [
+        "Primera parte de ",
+        "Segunda parte de ",
+        "Tercera parte de ",
+        "Cuarta parte de ",
+        "Quinta parte de ",
+        "Sexta parte de ",
+        "Parte de ",
+        "Troceo de ",
+        "continuación de ",
+        "continuacion de ",
+        "Sigue a ",
+    ];
+    for p in con_limite {
+        for i in posiciones(objetivo, p) {
+            if limite_antes(objetivo, i) {
+                tras(i, p);
+            }
+        }
+    }
+    for p in ["pasa a ", "pasan a "] {
+        for i in posiciones(objetivo, p) {
+            let resto = &objetivo[i + p.len()..];
+            if let Some(t) = id_al_principio(resto)
+                && resto[t.len()..].starts_with(" (troceo")
+            {
+                ids.insert(t.to_owned());
+            }
+        }
+    }
+    // `parte del [^.]{0,80}? troceado en (TASK-\d+)`: el hueco mas corto.
+    for i in posiciones(objetivo, "parte del ") {
+        let resto = &objetivo[i + "parte del ".len()..];
+        for (j, _) in resto.match_indices(" troceado en ") {
+            let hueco = &resto[..j];
+            if hueco.contains('.') || hueco.chars().count() > 80 {
+                break;
+            }
+            if let Some(t) = id_al_principio(&resto[j + " troceado en ".len()..]) {
+                ids.insert(t.to_owned());
+                break;
+            }
+        }
+    }
+    // `(TASK-\d+) dej[oó] resuelto`.
+    for i in posiciones(objetivo, "TASK-") {
+        if let Some(t) = id_al_principio(&objetivo[i..]) {
+            let resto = &objetivo[i + t.len()..];
+            if resto.starts_with(" dejo resuelto") || resto.starts_with(" dejó resuelto") {
+                ids.insert(t.to_owned());
+            }
+        }
+    }
+    // `Troceada en el dise[nñ]o[^.]*\.?[^.]*`: los IDs de esa frase y la
+    // siguiente.
+    let inicio = ["Troceada en el diseno", "Troceada en el diseño"]
+        .iter()
+        .filter_map(|p| objetivo.find(p))
+        .min();
+    if let Some(i) = inicio {
+        let resto = &objetivo[i..];
+        let mut fin = resto.find('.').unwrap_or(resto.len());
+        if resto[fin..].starts_with('.') {
+            fin += 1;
+            fin += resto[fin..].find('.').unwrap_or(resto.len() - fin);
+        }
+        let frase = &resto[..fin];
+        for j in posiciones(frase, "TASK-") {
+            if let Some(t) = id_al_principio(&frase[j..]) {
+                ids.insert(t.to_owned());
+            }
+        }
+    }
+    ids.remove(id);
+    ids
+}
+
+/// Una tarea terminada leida, antes de agrupar.
+struct Leida1 {
+    id: String,
+    carril: Option<Carril>,
+    complejidad: Option<String>,
+    criterios: usize,
+    real: Real,
+    start: i64,
+    finish: i64,
+    troceo: HashSet<String>,
+}
+
+/// Lo comun de los dos formatos que necesita el lector.
+fn datos(doc: &Documento) -> (&str, &str, Option<Carril>, Option<String>, &Cuerpo) {
+    match doc {
+        Documento::Formato1(t, c) => (
+            t.id.as_str(),
+            t.estado.as_str(),
+            None,
+            t.complejidad.clone(),
+            c,
+        ),
+        Documento::Formato2(t, c) => (
+            t.id.as_str(),
+            t.estado.as_str(),
+            t.carril.as_deref().and_then(Carril::desde),
+            t.complejidad.clone(),
+            c,
+        ),
+    }
+}
+
+/// `historico(tareas) -> (muestras, descartes)` sobre los `tarea.md` leidos
+/// de la rama de estado (los de [`crate::estado::tareas`]) y los commits.
+pub fn historico(leidas: &[Leida], commits: &Commits) -> Historico {
+    let mut h = Historico::default();
+    let mut sueltas: Vec<Leida1> = Vec::new();
+    for (_, doc) in leidas {
+        let Ok(doc) = doc else { continue };
+        let (id, estado, carril, complejidad, cuerpo) = datos(doc);
+        if estado != TERMINADA {
+            continue;
+        }
+        let ev = real::eventos(cuerpo, commits.de_tarea(id));
+        h.filas_malas += ev.filas_malas;
+        match real::real(&ev) {
+            Ok(r) => {
+                let de = |f: &'static str| ev.transiciones.iter().filter(move |p| p.fase == f);
+                let objetivo = cuerpo.seccion("Objetivo").map_or("", |s| s.texto.as_str());
+                sueltas.push(Leida1 {
+                    id: id.to_owned(),
+                    carril,
+                    complejidad,
+                    criterios: criterios(cuerpo),
+                    real: r,
+                    start: de("start").map(|p| p.ts).min().unwrap_or_default(),
+                    finish: de("finish").map(|p| p.ts).max().unwrap_or_default(),
+                    troceo: troceo(id, objetivo),
+                });
+            }
+            Err(d) => h
+                .descartes
+                .entry(d.motivo())
+                .or_default()
+                .push(id.to_owned()),
+        }
+    }
+    h.usables = sueltas.len();
+    h.muestras = agrupar(sueltas);
+    h
+}
+
+/// Raiz de `x` (union-find con compresion).
+fn raiz(padre: &mut HashMap<String, String>, x: &str) -> String {
+    let mut r = x.to_owned();
+    while let Some(p) = padre.get(&r).filter(|p| **p != r).cloned() {
+        r = p;
+    }
+    let mut y = x.to_owned();
+    while let Some(p) = padre.get(&y).filter(|p| **p != y).cloned() {
+        padre.insert(y, r.clone());
+        y = p;
+    }
+    r
+}
+
+/// Los grupos de troceo como una muestra cada uno (la raiz es el ID menor),
+/// por orden de `start`.
+fn agrupar(sueltas: Vec<Leida1>) -> Vec<Muestra> {
+    let mut padre: HashMap<String, String> = sueltas
+        .iter()
+        .map(|m| (m.id.clone(), m.id.clone()))
+        .collect();
+    for m in &sueltas {
+        for o in &m.troceo {
+            if padre.contains_key(o) {
+                let (a, b) = (raiz(&mut padre, &m.id), raiz(&mut padre, o));
+                if a != b {
+                    let (menor, mayor) = if a < b { (a, b) } else { (b, a) };
+                    padre.insert(mayor, menor);
+                }
+            }
+        }
+    }
+    let mut grupos: Vec<(String, Vec<Leida1>)> = Vec::new();
+    for m in sueltas {
+        let r = raiz(&mut padre, &m.id);
+        match grupos.iter_mut().find(|(g, _)| *g == r) {
+            Some((_, v)) => v.push(m),
+            None => grupos.push((r, vec![m])),
+        }
+    }
+    let mut v: Vec<Muestra> = grupos.into_iter().map(|(_, ms)| de_grupo(ms)).collect();
+    v.sort_by_key(|m| m.start);
+    v
+}
+
+/// Un grupo como muestra: real sumado por fase (en orden de `start`),
+/// criterios sumados, el carril mayor, la complejidad de la primera.
+fn de_grupo(mut ms: Vec<Leida1>) -> Muestra {
+    ms.sort_by_key(|m| m.start);
+    let suma = |f: fn(&Real) -> f64| ms.iter().fold(0.0, |acc, m| acc + f(&m.real));
+    let real = Real {
+        implementacion: suma(|r| r.implementacion),
+        rondas: suma(|r| r.rondas),
+        cierre: suma(|r| r.cierre),
+    };
+    let miembros: Vec<String> = ms.iter().map(|m| m.id.clone()).collect();
+    Muestra {
+        id: miembros.join("+"),
+        carril: ms.iter().filter_map(|m| m.carril).max(),
+        complejidad: ms.first().and_then(|m| m.complejidad.clone()),
+        criterios: ms.iter().map(|m| m.criterios).sum(),
+        real,
+        start: ms.first().map_or(0, |m| m.start),
+        finish: ms.iter().map(|m| m.finish).max().unwrap_or_default(),
+        miembros,
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn ids_del_asunto_como_la_poc() {
+        assert_eq!(ids_del_asunto("feat(TASK-083): x"), ["TASK-083"]);
+        assert_eq!(ids_del_asunto("docs(task-3): nota"), ["TASK-003"]);
+        assert_eq!(
+            ids_del_asunto("merge(feature): feature/task-001-x -> develop"),
+            ["TASK-001"]
+        );
+        assert!(ids_del_asunto("TASK-083a y xTASK-1 y TASK- y TASK_1").is_empty());
+        assert_eq!(
+            ids_del_asunto("TASK-0012, TASK-12 y TASK-1000"),
+            ["TASK-012", "TASK-1000"]
+        );
+        assert_eq!(
+            id_de_merge("merge(fix): fix/task-12-x -> develop").as_deref(),
+            Some("TASK-012")
+        );
+        for no in [
+            "Merge(feature): feature/task-1-x",
+            "merge(hotfix): hotfix/task-1-x",
+            "merge(feature): feature/task-x",
+            "merge(feature): feature/task-1",
+        ] {
+            assert_eq!(id_de_merge(no), None, "{no}");
+        }
+    }
+
+    /// `M^1..M^2`: los commits de la rama, no los de `develop` de antes.
+    #[test]
+    fn commits_de_la_rama_y_del_asunto() {
+        let l = |sha: &str, padres: &str, ts: i64, s: &str| {
+            format!("{sha}\u{1f}{padres}\u{1f}{ts}\u{1f}{s}\n")
+        };
+        let log = [
+            l(
+                "m",
+                "d1 b2",
+                50,
+                "merge(feature): feature/task-007-x -> develop",
+            ),
+            l("b2", "b1", 40, "dos"),
+            l("d1", "d0", 35, "chore(TASK-007): en develop"),
+            l("b1", "d0", 30, "uno"),
+            l("d0", "", 10, "inicial"),
+            "basura\n".to_owned(),
+        ]
+        .concat();
+        let c = Commits::de_log(&log);
+        assert_eq!(c.de_tarea("TASK-007"), vec![30, 35, 40, 50]);
+        assert!(c.de_tarea("TASK-008").is_empty());
+        assert!(Commits::vacios().de_tarea("TASK-007").is_empty());
+    }
+
+    #[test]
+    fn troceo_por_el_objetivo() {
+        let t = |o: &str| {
+            let mut v: Vec<String> = troceo("TASK-010", o).into_iter().collect();
+            v.sort();
+            v
+        };
+        assert_eq!(t("Segunda parte de TASK-005: lo que falto."), ["TASK-005"]);
+        assert_eq!(
+            t("Parte de TASK-4 y continuación de TASK-3."),
+            ["TASK-3", "TASK-4"]
+        );
+        assert_eq!(
+            t("Sigue a TASK-002. Troceo de TASK-001"),
+            ["TASK-001", "TASK-002"]
+        );
+        assert_eq!(t("los criterios pasan a TASK-020 (troceo)"), ["TASK-020"]);
+        assert_eq!(t("la parte del cierre troceado en TASK-030"), ["TASK-030"]);
+        assert_eq!(
+            t("la parte del cierre. Luego troceado en TASK-030"),
+            Vec::<String>::new()
+        );
+        assert_eq!(t("TASK-040 dejó resuelto el resto"), ["TASK-040"]);
+        assert_eq!(
+            t("Troceada en el diseño con TASK-050. Y TASK-051. No TASK-052"),
+            ["TASK-050", "TASK-051"]
+        );
+        // Sin limite de palabra delante, no; y la propia no cuenta.
+        assert_eq!(
+            t("xParte de TASK-1 y Sigue a TASK-010"),
+            Vec::<String>::new()
+        );
+    }
+}
diff --git a/crates/tc2/src/estimacion/mod.rs b/crates/tc2/src/estimacion/mod.rs
new file mode 100644
index 0000000..2c9461a
--- /dev/null
+++ b/crates/tc2/src/estimacion/mod.rs
@@ -0,0 +1,335 @@
+//! Estimacion por fase de cada tarea (F3-13; ADR F1-17, «Consecuencias»):
+//! el lector del historico, la estimacion, lo que falta y el error, y la
+//! escritura del campo `estimacion`.
+//!
+//! - [`real`]: el real por fase de una tarea terminada (D6a de F1-17).
+//! - [`historico`]: las muestras de las terminadas de la rama de estado y de
+//!   un `git log` de `develop`, con los descartes por motivo.
+//! - [`estimar`](mod@estimar): la cascada de tareas parecidas, [`restante`] y
+//!   [`error`]. Todo puro: ni git, ni red, ni modelos (C7).
+//! - [`recalcular`]: el unico punto por el que las ordenes ponen el campo
+//!   (`new`, `import`, `start` y `tc2 estimar`; D2a de TASK-083). `pausa`,
+//!   `reanudar`, `liberar` y `reasignar` no lo tocan.
+//! - `tc2 estimar [--json]` rellena las abiertas con `estimacion: null` en
+//!   un solo commit de estado; `tc2 estimar TASK-NNN` la ensena recalculada
+//!   sin escribir; `tc2 estimar --error TASK-NNN` mide la guardada frente al
+//!   real (solo lectura, D4a).
+
+pub mod estimar;
+mod forma;
+pub mod historico;
+pub mod real;
+
+use std::cell::RefCell;
+use std::path::Path;
+
+use crate::contrato::{Informe, Resultado, Valor};
+use crate::estado::{self, Cambio, Catalogo, Estado, Fallo, Operacion};
+use crate::git::{self, Git};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::estimacion as txt;
+use crate::tarea::{Cuerpo, Documento, Tarea};
+
+pub use estimar::{
+    ErrorFase, Estimacion, Fase, N_MIN, NIVELES, Objetivo, P80, PorFase, Rango, Restante,
+    SIN_DATOS, cuantil, error, estimar, mediana, parecidas, restante,
+};
+pub use forma::minutos;
+pub use historico::{Commits, Historico, Muestra, criterios, historico};
+pub use real::{Descarte, Eventos, FASES, Real, UMBRAL_MIN, eventos};
+
+/// Nombre de la orden y de la operacion de la rama de estado.
+pub const NOMBRE: &str = "estimar";
+
+/// Estados en los que una tarea ya no se estima.
+const CERRADOS: [&str; 2] = ["terminada", "descartada"];
+
+/// Los commits de `develop` de un solo `git log` ([`git::historia`]). Si no
+/// se pueden leer, ninguno: la estimacion sale igual, con los eventos de los
+/// `tarea.md` (la fila de transiciones y de coste).
+pub fn commits_de(g: &Git, dir: &Path) -> Commits {
+    git::historia(g, dir, NOMBRE)
+        .map(|t| Commits::de_log(&t))
+        .unwrap_or_default()
+}
+
+/// Lo que se sabe de una tarea para estimarla.
+pub fn objetivo(t: &Tarea, cuerpo: &Cuerpo) -> Objetivo {
+    Objetivo {
+        id: t.id.clone(),
+        carril: t.carril.as_deref().and_then(crate::carril::Carril::desde),
+        complejidad: t.complejidad.clone(),
+        criterios: criterios(cuerpo),
+    }
+}
+
+/// Recalcula la estimacion de `t` con las muestras de `h` y la pone en su
+/// campo `estimacion` (el resto del `tarea.md`, byte a byte). `ahora`, en
+/// segundos desde 1970, es la hora `calculada`.
+pub fn recalcular(t: &mut Tarea, cuerpo: &Cuerpo, h: &Historico, ahora: u64) -> Estimacion {
+    let e = estimar(
+        &objetivo(t, cuerpo),
+        &h.muestras,
+        &crate::alta::instante_utc(ahora),
+    );
+    t.poner_estimacion(&e);
+    e
+}
+
+/// Pone las claves de la estimacion (`estimacion=`, `estimacion-p80=`,
+/// `estimacion-n=`, `estimacion-nivel=`) al final de `i`.
+pub fn con_claves(mut i: Informe, e: Option<&Estimacion>) -> Informe {
+    if let Some(e) = e {
+        i.claves.extend(e.claves());
+    }
+    i
+}
+
+/// `tc2 estimar` sin argumentos: estima las tareas abiertas de formato 2
+/// con `estimacion: null` (una sola lectura del estado y del historico, un
+/// solo commit). Las terminadas y descartadas no se tocan nunca.
+#[derive(Debug)]
+pub struct Rellenar {
+    ahora: u64,
+    commits: Commits,
+    ids: RefCell<Vec<String>>,
+}
+
+impl Rellenar {
+    /// Con los commits de `develop` y la hora del calculo.
+    pub fn nuevo(commits: Commits, ahora: u64) -> Self {
+        Self {
+            ahora,
+            commits,
+            ids: RefCell::new(Vec::new()),
+        }
+    }
+
+    /// IDs estimados en el ultimo intento.
+    pub fn ids(&self) -> Vec<String> {
+        self.ids.borrow().clone()
+    }
+}
+
+impl Operacion for Rellenar {
+    fn nombre(&self) -> &str {
+        NOMBRE
+    }
+    fn anotacion(&self) -> String {
+        NOMBRE.to_owned()
+    }
+    fn exige_red(&self) -> bool {
+        // Se recalcula sobre el estado fresco: sin red falla, no queda
+        // pendiente.
+        true
+    }
+    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
+        let leidas = estado::tareas(estado, &estado.rutas())?;
+        let h = historico(&leidas, &self.commits);
+        let mut cambios = Vec::new();
+        let mut ids = Vec::new();
+        for (ruta, doc) in &leidas {
+            let Ok(Documento::Formato2(t, c)) = doc else {
+                continue;
+            };
+            if CERRADOS.contains(&t.estado.as_str()) || t.estimacion().is_some() {
+                continue;
+            }
+            let mut t = t.clone();
+            recalcular(&mut t, c, &h, self.ahora);
+            cambios.push(Cambio::nuevo(ruta.clone(), t.escribir(c)));
+            ids.push(t.id.clone());
+        }
+        *self.ids.borrow_mut() = ids;
+        Ok(cambios)
+    }
+}
+
+/// Lo que necesita `tc2 estimar` del entorno.
+pub struct Entorno<'a> {
+    /// Carpeta desde la que se lanza.
+    pub dir: &'a Path,
+    /// Puerto Git.
+    pub git: &'a Git,
+    /// Para re-ejecutar las pendientes de la transaccion.
+    pub catalogo: Catalogo<'a>,
+    /// Hora del calculo, en segundos desde 1970 (UTC).
+    pub ahora: u64,
+}
+
+/// `estimar [TASK-NNN | --error TASK-NNN]` (sin `--json`, que quita
+/// `lib.rs`): el informe y, si sale bien, la linea para la persona (que va
+/// detras de la de contrato y no en JSON).
+pub fn orden(args: &[String], e: &Entorno<'_>) -> (Informe, Option<String>) {
+    let uso = |motivo: &str| {
+        (
+            Informe::nuevo(NOMBRE, Resultado::Uso)
+                .con("motivo", motivo)
+                .diagnostico(&txt::USO_ESTIMAR),
+            None,
+        )
+    };
+    let con_id = |id: &str, f: &dyn Fn(&str) -> (Informe, Option<String>)| {
+        if git::es_id(id) {
+            f(id)
+        } else {
+            uso("id-invalido")
+        }
+    };
+    match args {
+        [] => (rellenar(e), None),
+        [a] if a == "--error" => uso("sin-id"),
+        [a, id] if a == "--error" => con_id(id, &|id| medir(e, id)),
+        [id] if !id.starts_with("--") => con_id(id, &|id| mostrar(e, id)),
+        _ => uso("argumentos-de-mas"),
+    }
+}
+
+/// El informe de una transaccion como de `estimar`.
+fn de_estimar(mut i: Informe) -> Informe {
+    NOMBRE.clone_into(&mut i.comando);
+    i.claves.retain(|(k, _)| k != "operacion");
+    i
+}
+
+fn rellenar(e: &Entorno<'_>) -> Informe {
+    let op = Rellenar::nuevo(commits_de(e.git, e.dir), e.ahora);
+    let ctx = estado::Contexto::nuevo(e.git, e.dir, e.catalogo, None);
+    let mut i = de_estimar(estado::transaccion(&ctx, &op));
+    if i.resultado.es_exito() {
+        let ids = if i.resultado == Resultado::Ok {
+            op.ids()
+        } else {
+            Vec::new()
+        };
+        let mut delante = vec![(
+            "actualizadas".to_owned(),
+            Valor::Texto(ids.len().to_string()),
+        )];
+        if !ids.is_empty() {
+            delante.push(("ids".to_owned(), Valor::Lista(ids)));
+        }
+        delante.append(&mut i.claves);
+        i.claves = delante;
+    }
+    i
+}
+
+/// La tarea `id` leida del estado conocido (sin `fetch`: solo lectura), con
+/// todos los `tarea.md` y los commits.
+fn leer(e: &Entorno<'_>, id: &str) -> Result<(Tarea, Cuerpo, Historico, Commits), Informe> {
+    let vista = estado::leer(e.git, e.dir).map_err(de_estimar)?;
+    let rechazo = |f: Fallo| match f {
+        Fallo::Rechazada {
+            motivo,
+            diagnostico,
+        } => Informe::nuevo(NOMBRE, Resultado::Rechazada)
+            .con("id", id)
+            .con("motivo", motivo)
+            .diagnostico(diagnostico),
+        Fallo::Error(d) => Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d),
+    };
+    let (t, c) = estado::tarea_de(&vista, id).map_err(rechazo)?;
+    let leidas = estado::tareas(&vista, &vista.rutas()).map_err(rechazo)?;
+    let commits = commits_de(e.git, e.dir);
+    let h = historico(&leidas, &commits);
+    Ok((t, c, h, commits))
+}
+
+/// `estimar TASK-NNN`: la estimacion recalculada ahora, sin escribir nada.
+fn mostrar(e: &Entorno<'_>, id: &str) -> (Informe, Option<String>) {
+    match leer(e, id) {
+        Err(i) => (i, None),
+        Ok((t, c, h, _)) => {
+            let est = estimar(
+                &objetivo(&t, &c),
+                &h.muestras,
+                &crate::alta::instante_utc(e.ahora),
+            );
+            let i = con_claves(
+                Informe::nuevo(NOMBRE, Resultado::Ok).con("id", id),
+                Some(&est),
+            );
+            (i, Some(est.linea()))
+        }
+    }
+}
+
+/// Dos decimales con punto (claves) y con coma (la linea).
+fn decimales(x: f64) -> (String, String) {
+    let p = format!("{x:.2}");
+    let c = p.replace('.', ",");
+    (p, c)
+}
+
+/// `estimar --error TASK-NNN`: la estimacion guardada frente al real por
+/// fase y en total (D4a: solo lectura; guardarlo al cerrar es de F8-7).
+fn medir(e: &Entorno<'_>, id: &str) -> (Informe, Option<String>) {
+    let rechazada = |motivo: &str, m: Mensaje| {
+        (
+            Informe::nuevo(NOMBRE, Resultado::Rechazada)
+                .con("id", id)
+                .con("motivo", motivo)
+                .diagnostico(m.con("id", id)),
+            None,
+        )
+    };
+    let (t, c, _, commits) = match leer(e, id) {
+        Ok(x) => x,
+        Err(i) => return (i, None),
+    };
+    let Some(crudo) = t.estimacion() else {
+        return rechazada("sin-estimacion", Mensaje::nuevo(&txt::SIN_ESTIMACION));
+    };
+    let est = match Estimacion::leer(crudo) {
+        Ok(x) => x,
+        Err(err) => {
+            return rechazada(
+                "estimacion-ilegible",
+                Mensaje::nuevo(&txt::ESTIMACION_ILEGIBLE).con("error", err),
+            );
+        }
+    };
+    let real = match real::real(&eventos(&c, commits.de_tarea(id))) {
+        Ok(r) => r,
+        Err(d) => {
+            let (mut i, _) = rechazada(
+                "sin-real",
+                Mensaje::nuevo(&txt::SIN_REAL).con("motivo", d.motivo()),
+            );
+            i = i.con("causa", d.motivo());
+            return (i, None);
+        }
+    };
+    let errores = error(&est, &real);
+    if errores.is_empty() {
+        return rechazada("sin-cifras", Mensaje::nuevo(&txt::SIN_CIFRAS));
+    }
+    let mut i = Informe::nuevo(NOMBRE, Resultado::Ok).con("id", id);
+    let mut partes = Vec::new();
+    for x in &errores {
+        let (rel, rel_c) = decimales(x.relativo);
+        let (razon, razon_c) = decimales(x.razon);
+        let abs = minutos(x.absoluto).to_string();
+        i = i
+            .con(&format!("error-{}", x.fase), abs.clone())
+            .con(&format!("relativo-{}", x.fase), rel)
+            .con(&format!("razon-{}", x.fase), razon);
+        partes.push(format!(
+            "{} {abs} min (relativo {rel_c}; estimado/real {razon_c})",
+            x.fase
+        ));
+    }
+    (i, Some(format!("error: {}", partes.join("; "))))
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn decimales_con_punto_y_con_coma() {
+        assert_eq!(decimales(0.6), ("0.60".to_owned(), "0,60".to_owned()));
+        assert_eq!(decimales(1.0 / 3.0).0, "0.33");
+    }
+}
diff --git a/crates/tc2/src/estimacion/real.rs b/crates/tc2/src/estimacion/real.rs
new file mode 100644
index 0000000..94b27e3
--- /dev/null
+++ b/crates/tc2/src/estimacion/real.rs
@@ -0,0 +1,420 @@
+//! El real por fase de una tarea terminada (ADR F1-17, decision 1 y D6a),
+//! recalculado desde sus eventos: las filas de `## Transiciones`, las de
+//! `## Coste` y los commits que la nombran o estan en su rama.
+//!
+//! - Fases medibles (D7a): implementacion (de `start` a la primera
+//!   `review`), rondas (de la primera a la ultima `review`) y cierre (de la
+//!   ultima `review` a `finish`). El diseno queda sin calibrar.
+//! - El real de una fase es la suma de los huecos entre eventos
+//!   consecutivos, sin las ventanas de `pausa` (desde la `pausa` hasta la
+//!   transicion siguiente que no sea otra `pausa`, como la 0.16), y con cada
+//!   hueco de mas de [`UMBRAL_MIN`] minutos contado como [`UMBRAL_MIN`]
+//!   (divergencia 6 de F1-17: tope, no eliminacion).
+//! - Mismas reglas, en el mismo orden, que `real`, `tramo` y
+//!   `ventanas_pausa` de `poc-f1-17.py` (paridad en `tests/estimacion.rs`).
+//!
+//! Puro: no lee git ni ficheros.
+
+use crate::tarea::Cuerpo;
+
+/// Minutos a partir de los cuales un hueco sin eventos cuenta solo eso
+/// (D6a de F1-17).
+pub const UMBRAL_MIN: f64 = 45.0;
+
+/// Las fases medibles, en su orden (D7a).
+pub const FASES: [&str; 3] = ["implementacion", "rondas", "cierre"];
+
+/// Una fila de `## Transiciones` que se entiende: instante (segundos desde
+/// 1970, UTC) y fase.
+#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
+pub struct Paso {
+    /// Instante de la fila.
+    pub ts: i64,
+    /// Fase (`start`, `review`, `pausa`...).
+    pub fase: String,
+}
+
+/// Los eventos de una tarea.
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct Eventos {
+    /// Filas de `## Transiciones` con fecha valida.
+    pub transiciones: Vec<Paso>,
+    /// Si la tabla de transiciones tenia alguna fila (aunque ilegible).
+    pub con_filas: bool,
+    /// Instantes de las filas de `## Coste`.
+    pub coste: Vec<i64>,
+    /// Instantes de los commits de la tarea.
+    pub commits: Vec<i64>,
+    /// Filas que no se entendieron (fecha o total ilegibles).
+    pub filas_malas: usize,
+}
+
+/// Minutos reales por fase.
+#[derive(Debug, Clone, Copy, PartialEq)]
+pub struct Real {
+    /// De `start` a la primera `review`.
+    pub implementacion: f64,
+    /// De la primera a la ultima `review`.
+    pub rondas: f64,
+    /// De la ultima `review` a `finish`.
+    pub cierre: f64,
+}
+
+impl Real {
+    /// Suma de las tres fases (en el orden de la PoC).
+    pub fn total(&self) -> f64 {
+        self.implementacion + self.rondas + self.cierre
+    }
+
+    /// La fase por su nombre (`total` incluido).
+    pub fn fase(&self, f: &str) -> Option<f64> {
+        match f {
+            "implementacion" => Some(self.implementacion),
+            "rondas" => Some(self.rondas),
+            "cierre" => Some(self.cierre),
+            "total" => Some(self.total()),
+            _ => None,
+        }
+    }
+}
+
+/// Por que una tarea terminada no da muestra (los motivos de la PoC).
+#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
+pub enum Descarte {
+    /// `## Transiciones` sin ninguna fila.
+    SinTransiciones,
+    /// Ninguna fila `start` con fecha.
+    SinStart,
+    /// Ninguna fila `finish` con fecha.
+    SinFinish,
+    /// Ninguna fila `review` con fecha.
+    SinReview,
+    /// Una `review` antes del `start` o despues del `finish`: una fase
+    /// tendria duracion negativa (dato invalido).
+    DuracionNegativa,
+}
+
+impl Descarte {
+    /// El motivo en la salida (`duracion-negativa`...).
+    pub fn motivo(self) -> &'static str {
+        match self {
+            Self::SinTransiciones => "sin-transiciones",
+            Self::SinStart => "sin-start",
+            Self::SinFinish => "sin-finish",
+            Self::SinReview => "sin-review",
+            Self::DuracionNegativa => "duracion-negativa",
+        }
+    }
+}
+
+/// Segundos desde 1970 de `AAAA-MM-DDTHH:MM:SSZ` exacto (la fecha que
+/// escriben la 0.16 y la 2.0); cualquier otra forma, `None` (fila ilegible,
+/// como `fecha()` de la PoC).
+pub fn segundos_de(fecha: &str) -> Option<i64> {
+    let b = fecha.as_bytes();
+    let forma = b.len() == 20
+        && b.iter().enumerate().all(|(i, c)| match i {
+            4 | 7 => *c == b'-',
+            10 => *c == b'T',
+            13 | 16 => *c == b':',
+            19 => *c == b'Z',
+            _ => c.is_ascii_digit(),
+        });
+    if !forma {
+        return None;
+    }
+    crate::inventario::ms_de_iso(fecha).map(|ms| ms.div_euclid(1000))
+}
+
+/// Las filas de la primera tabla Markdown de `texto` como mapas cabecera →
+/// celda (como `tabla_md` de la PoC: la primera fila es la cabecera y la
+/// segunda, el separador). Sin cabecera y separador, ninguna.
+pub fn tabla(texto: &str) -> Vec<Vec<(String, String)>> {
+    let filas: Vec<Vec<String>> = texto
+        .lines()
+        .map(str::trim)
+        .filter(|l| l.starts_with('|'))
+        .map(|l| {
+            l.trim_matches('|')
+                .split('|')
+                .map(|c| c.trim().to_owned())
+                .collect()
+        })
+        .collect();
+    let Some(cabecera) = filas.first() else {
+        return Vec::new();
+    };
+    if filas.len() < 2 {
+        return Vec::new();
+    }
+    filas
+        .iter()
+        .skip(2)
+        .map(|f| cabecera.iter().cloned().zip(f.iter().cloned()).collect())
+        .collect()
+}
+
+/// Celda `clave` de una fila de [`tabla`].
+fn celda<'a>(fila: &'a [(String, String)], clave: &str) -> Option<&'a str> {
+    fila.iter()
+        .find(|(k, _)| k == clave)
+        .map(|(_, v)| v.as_str())
+}
+
+/// Los eventos de `cuerpo` mas los commits de la tarea.
+pub fn eventos(cuerpo: &Cuerpo, commits: Vec<i64>) -> Eventos {
+    let mut ev = Eventos {
+        commits,
+        ..Eventos::default()
+    };
+    let texto = |t: &str| cuerpo.seccion(t).map_or("", |s| s.texto.as_str());
+    let filas = tabla(texto("Transiciones"));
+    ev.con_filas = !filas.is_empty();
+    for f in &filas {
+        match segundos_de(celda(f, "fecha").unwrap_or_default()) {
+            Some(ts) => ev.transiciones.push(Paso {
+                ts,
+                fase: celda(f, "fase").unwrap_or_default().to_owned(),
+            }),
+            None => ev.filas_malas += 1,
+        }
+    }
+    for f in tabla(texto("Coste")) {
+        let total = celda(&f, "total").unwrap_or_default();
+        let total_ok = total.is_empty() || total.parse::<i64>().is_ok();
+        match celda(&f, "fecha").and_then(segundos_de) {
+            Some(ts) if total_ok => ev.coste.push(ts),
+            _ => ev.filas_malas += 1,
+        }
+    }
+    ev
+}
+
+/// Ventanas de pausa: de cada `pausa` a la transicion siguiente que no sea
+/// otra `pausa` (en el orden por instante y fase), si es posterior.
+pub fn ventanas_pausa(transiciones: &[Paso]) -> Vec<(i64, i64)> {
+    let mut ts: Vec<&Paso> = transiciones.iter().collect();
+    ts.sort();
+    let mut v = Vec::new();
+    for (i, p) in ts.iter().enumerate() {
+        if p.fase != crate::tarea::PAUSA {
+            continue;
+        }
+        let sig = ts
+            .get(i + 1..)
+            .unwrap_or_default()
+            .iter()
+            .find(|q| q.fase != crate::tarea::PAUSA)
+            .map(|q| q.ts);
+        if let Some(s) = sig
+            && s > p.ts
+        {
+            v.push((p.ts, s));
+        }
+    }
+    v
+}
+
+/// Minutos activos entre `ini` y `fin`: los huecos entre los eventos de
+/// dentro (y los bordes de las pausas), sin los que caen en una pausa, y
+/// con cada hueco de mas de [`UMBRAL_MIN`] contado como [`UMBRAL_MIN`].
+pub fn tramo(ini: i64, fin: i64, eventos: &[i64], pausas: &[(i64, i64)]) -> f64 {
+    let mut pts: Vec<i64> = vec![ini, fin];
+    pts.extend(eventos.iter().copied().filter(|e| ini < *e && *e < fin));
+    for (a, b) in pausas {
+        pts.extend([*a, *b].into_iter().filter(|x| ini < *x && *x < fin));
+    }
+    pts.sort_unstable();
+    pts.dedup();
+    let mut suma = 0.0;
+    for par in pts.windows(2) {
+        let (a, b) = (par[0], par[1]);
+        // El punto medio, como la PoC, en coma flotante.
+        let medio = (a as f64 + b as f64) / 2.0;
+        if pausas
+            .iter()
+            .any(|(pa, pb)| *pa as f64 <= medio && medio <= *pb as f64)
+        {
+            continue;
+        }
+        let h = (b - a) as f64 / 60.0;
+        suma += h.min(UMBRAL_MIN);
+    }
+    suma
+}
+
+/// El real por fase, o por que la tarea no da muestra.
+pub fn real(ev: &Eventos) -> Result<Real, Descarte> {
+    if !ev.con_filas {
+        return Err(Descarte::SinTransiciones);
+    }
+    let de = |f: &'static str| ev.transiciones.iter().filter(move |p| p.fase == f);
+    let start = de("start").map(|p| p.ts).min().ok_or(Descarte::SinStart)?;
+    let finish = de("finish")
+        .map(|p| p.ts)
+        .max()
+        .ok_or(Descarte::SinFinish)?;
+    let mut reviews: Vec<i64> = de("review").map(|p| p.ts).collect();
+    reviews.sort_unstable();
+    let (Some(&r1), Some(&rn)) = (reviews.first(), reviews.last()) else {
+        return Err(Descarte::SinReview);
+    };
+    if !(start <= r1 && rn <= finish) {
+        return Err(Descarte::DuracionNegativa);
+    }
+    let mut todos: Vec<i64> = ev.commits.clone();
+    todos.extend(&ev.coste);
+    todos.extend(ev.transiciones.iter().map(|p| p.ts));
+    todos.sort_unstable();
+    todos.dedup();
+    let pausas = ventanas_pausa(&ev.transiciones);
+    Ok(Real {
+        implementacion: tramo(start, r1, &todos, &pausas),
+        rondas: tramo(r1, rn, &todos, &pausas),
+        cierre: tramo(rn, finish, &todos, &pausas),
+    })
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    /// Segundos de `HH:MM` del 2026-11-17 (UTC).
+    fn h(hh: i64, mm: i64) -> i64 {
+        1_794_873_600 + hh * 3600 + mm * 60
+    }
+
+    fn paso(ts: i64, fase: &str) -> Paso {
+        Paso {
+            ts,
+            fase: fase.to_owned(),
+        }
+    }
+
+    /// Los eventos de C1: start 10:00, commit 10:20, pausa 10:30, reanudar
+    /// 11:30, commit 11:40 (si `con_1140`), review 13:00 y 13:30, finish
+    /// 13:40.
+    fn c1(con_1140: bool) -> Eventos {
+        let mut commits = vec![h(10, 20)];
+        if con_1140 {
+            commits.push(h(11, 40));
+        }
+        Eventos {
+            transiciones: vec![
+                paso(h(10, 0), "start"),
+                paso(h(10, 30), "pausa"),
+                paso(h(11, 30), "reanudar"),
+                paso(h(13, 0), "review"),
+                paso(h(13, 30), "review"),
+                paso(h(13, 40), "finish"),
+            ],
+            con_filas: true,
+            commits,
+            ..Eventos::default()
+        }
+    }
+
+    #[test]
+    fn fechas_exactas_y_tablas() {
+        assert_eq!(segundos_de("2026-11-17T10:00:00Z"), Some(h(10, 0)));
+        for malo in [
+            "2026-11-17",
+            "2026-11-17T10:00:00",
+            "2026-11-17 10:00:00Z",
+            "2026-11-17T10:00:00+00:00",
+            "2026-13-17T10:00:00Z",
+            "x",
+        ] {
+            assert_eq!(segundos_de(malo), None, "{malo}");
+        }
+        let t = tabla("## X\n| a | b |\n|---|---|\n| 1 | 2 |\n|| 3 |\ntexto\n| 4 |\n");
+        assert_eq!(
+            t,
+            vec![
+                vec![("a".into(), "1".into()), ("b".into(), "2".into())],
+                vec![("a".into(), "3".into())],
+                vec![("a".into(), "4".into())],
+            ]
+        );
+        assert!(tabla("| solo cabecera |\n").is_empty());
+    }
+
+    /// C1: el real por fase con la regla de F1-17 (paridad con la PoC en
+    /// `tests/estimacion.rs`): con el commit de las 11:40, 85/30/10; sin el,
+    /// 75/30/10 (el 75 del enunciado de C1 es el de este segundo caso).
+    #[test]
+    fn c1_real_por_fase_con_pausa_y_tope() {
+        let r = real(&c1(true));
+        assert_eq!(
+            r,
+            Ok(Real {
+                implementacion: 85.0,
+                rondas: 30.0,
+                cierre: 10.0
+            })
+        );
+        let r = real(&c1(false));
+        assert_eq!(
+            r.map(|r| (r.implementacion, r.rondas, r.cierre, r.total())),
+            Ok((75.0, 30.0, 10.0, 115.0))
+        );
+    }
+
+    /// C1: una `review` antes del `start` es duracion negativa; y los demas
+    /// descartes, por su orden.
+    #[test]
+    fn c1_duracion_negativa_y_descartes() {
+        let mut e = c1(true);
+        e.transiciones[3].ts = h(9, 0);
+        assert_eq!(real(&e), Err(Descarte::DuracionNegativa));
+        assert_eq!(Descarte::DuracionNegativa.motivo(), "duracion-negativa");
+        let mut e = c1(true);
+        e.transiciones.retain(|p| p.fase != "review");
+        assert_eq!(real(&e), Err(Descarte::SinReview));
+        let mut e = c1(true);
+        e.transiciones.retain(|p| p.fase != "finish");
+        assert_eq!(real(&e), Err(Descarte::SinFinish));
+        let mut e = c1(true);
+        e.transiciones.retain(|p| p.fase != "start");
+        assert_eq!(real(&e), Err(Descarte::SinStart));
+        assert_eq!(real(&Eventos::default()), Err(Descarte::SinTransiciones));
+    }
+
+    /// Una pausa sin `reanudar` dura hasta la transicion siguiente (aqui,
+    /// `finish`); dos `pausa` seguidas no cierran la ventana.
+    #[test]
+    fn pausa_sin_reanudar_y_pausas_seguidas() {
+        let t = vec![
+            paso(h(10, 0), "start"),
+            paso(h(10, 10), "review"),
+            paso(h(10, 20), "pausa"),
+            paso(h(10, 25), "pausa"),
+            paso(h(12, 0), "finish"),
+        ];
+        assert_eq!(
+            ventanas_pausa(&t),
+            vec![(h(10, 20), h(12, 0)), (h(10, 25), h(12, 0))]
+        );
+        let e = Eventos {
+            transiciones: t,
+            con_filas: true,
+            ..Eventos::default()
+        };
+        assert_eq!(real(&e).map(|r| r.cierre), Ok(10.0));
+    }
+
+    /// Las filas de Coste y de Transiciones del cuerpo, con las ilegibles
+    /// contadas.
+    #[test]
+    fn eventos_del_cuerpo() {
+        let c = Cuerpo::leer(
+            "## Transiciones\n\n| fecha | fase | modo |\n|---|---|---|\n| 2026-11-17T10:00:00Z | start | m |\n| ayer | review | m |\n\n## Coste\n\n| fecha | fase | total |\n|---|---|---|\n| 2026-11-17T10:30:00Z | implementacion | 5 |\n| 2026-11-17T10:40:00Z | x | mucho |\n| 2026-11-17T10:50:00Z | x | |\n",
+        );
+        let e = eventos(&c, vec![7]);
+        assert_eq!(e.transiciones, vec![paso(h(10, 0), "start")]);
+        assert_eq!(e.coste, vec![h(10, 30), h(10, 50)]);
+        assert_eq!(e.commits, vec![7]);
+        assert_eq!(e.filas_malas, 2);
+        assert!(e.con_filas);
+    }
+}
diff --git a/crates/tc2/src/git/mod.rs b/crates/tc2/src/git/mod.rs
index e172593..58bffca 100644
--- a/crates/tc2/src/git/mod.rs
+++ b/crates/tc2/src/git/mod.rs
@@ -132,6 +132,23 @@ pub(crate) fn rama_actual(git: &Git, dir: &Path, op: &str) -> Paso<Option<String
     Ok(s.trim().strip_prefix("refs/heads/").map(str::to_owned))
 }
 
+/// Formato de [`historia`]: SHA, padres, instante del commit y asunto,
+/// separados por U+001F.
+pub const FORMATO_HISTORIA: &str = "--format=%H%x1f%P%x1f%ct%x1f%s";
+
+/// La historia de `develop` para la estimacion (F3-13; ADR F1-17, punto 1):
+/// un solo `git log` de `origin/develop` o, sin el, de `develop`, con
+/// [`FORMATO_HISTORIA`]. Sin ninguna de las dos, vacia (no hay commits que
+/// contar).
+pub fn historia(git: &Git, dir: &Path, op: &str) -> Paso<String> {
+    for r in ["refs/remotes/origin/develop", "refs/heads/develop"] {
+        if commit_de(git, dir, op, r)?.is_some() {
+            return exigir(git, dir, op, Subcomando::Log, &[FORMATO_HISTORIA, r, "--"]);
+        }
+    }
+    Ok(String::new())
+}
+
 /// `develop` y `main` no se tocan desde una operacion de tarea.
 pub(crate) fn es_protegida(rama: &str) -> bool {
     rama == "develop" || rama == "main"
diff --git a/crates/tc2/src/git/puerto.rs b/crates/tc2/src/git/puerto.rs
index 69a93c8..f031aa7 100644
--- a/crates/tc2/src/git/puerto.rs
+++ b/crates/tc2/src/git/puerto.rs
@@ -40,6 +40,8 @@ pub enum Subcomando {
     HashObject,
     /// `git ls-remote`
     LsRemote,
+    /// `git log` (lectura; el historico de la estimacion, F3-13)
+    Log,
     /// `git ls-tree`
     LsTree,
     /// `git merge`
@@ -88,6 +90,7 @@ impl Subcomando {
             Self::ForEachRef => "for-each-ref",
             Self::HashObject => "hash-object",
             Self::LsRemote => "ls-remote",
+            Self::Log => "log",
             Self::LsTree => "ls-tree",
             Self::Merge => "merge",
             Self::MergeBase => "merge-base",
diff --git a/crates/tc2/src/i18n/catalogo/cli.rs b/crates/tc2/src/i18n/catalogo/cli.rs
index b2fcb66..628cab6 100644
--- a/crates/tc2/src/i18n/catalogo/cli.rs
+++ b/crates/tc2/src/i18n/catalogo/cli.rs
@@ -10,8 +10,8 @@ crate::i18n::textos! {
         en: "tc2: could not write the output: {error}",
     }
     USO {
-        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<motivo>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
-        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<reason>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
+        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<motivo>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
+        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<reason>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
     }
     FALTA_LA_ORDEN {
         es: "tc2: falta la orden. {uso}",
diff --git a/crates/tc2/src/i18n/catalogo/estimacion.rs b/crates/tc2/src/i18n/catalogo/estimacion.rs
new file mode 100644
index 0000000..e4f2228
--- /dev/null
+++ b/crates/tc2/src/i18n/catalogo/estimacion.rs
@@ -0,0 +1,24 @@
+//! Mensajes de `tc2 estimar` (F3-13).
+
+crate::i18n::textos! {
+    USO_ESTIMAR {
+        es: "uso: estimar [--json] [TASK-NNN | --error TASK-NNN]",
+        en: "usage: estimar [--json] [TASK-NNN | --error TASK-NNN]",
+    }
+    SIN_ESTIMACION {
+        es: "sin estimacion guardada en {id}: la pone tc2 estimar (o start); sin ella no hay error que medir",
+        en: "no saved estimate in {id}: tc2 estimar (or start) sets it; without it there is no error to measure",
+    }
+    SIN_CIFRAS {
+        es: "la estimacion guardada de {id} es sin-datos (no habia tareas parecidas): no hay cifras que comparar con el real",
+        en: "the saved estimate of {id} is sin-datos (there were no similar tasks): there are no figures to compare with the actual time",
+    }
+    ESTIMACION_ILEGIBLE {
+        es: "la estimacion guardada de {id} no tiene la forma de F1-17: {error}",
+        en: "the saved estimate of {id} does not have the F1-17 shape: {error}",
+    }
+    SIN_REAL {
+        es: "la tarea {id} no tiene real que medir ({motivo}): hacen falta start, review y finish en orden en sus transiciones",
+        en: "task {id} has no actual time to measure ({motivo}): it needs start, review and finish in order in its transitions",
+    }
+}
diff --git a/crates/tc2/src/i18n/catalogo/mod.rs b/crates/tc2/src/i18n/catalogo/mod.rs
index c716f93..bb90661 100644
--- a/crates/tc2/src/i18n/catalogo/mod.rs
+++ b/crates/tc2/src/i18n/catalogo/mod.rs
@@ -9,6 +9,7 @@ pub mod config;
 pub mod desinstalar;
 pub mod doctor;
 pub mod estado;
+pub mod estimacion;
 pub mod git;
 pub mod identidad;
 pub mod importar;
@@ -32,6 +33,7 @@ pub fn todos() -> impl Iterator<Item = &'static Texto> {
         desinstalar::TODOS,
         doctor::TODOS,
         estado::TODOS,
+        estimacion::TODOS,
         git::TODOS,
         identidad::TODOS,
         importar::TODOS,
diff --git a/crates/tc2/src/importar/mod.rs b/crates/tc2/src/importar/mod.rs
index 1e5c715..4369816 100644
--- a/crates/tc2/src/importar/mod.rs
+++ b/crates/tc2/src/importar/mod.rs
@@ -61,6 +61,8 @@ pub struct Importar {
     fecha: String,
     ocupados: BTreeSet<u32>,
     plan: RefCell<Option<Plan>>,
+    ahora: u64,
+    commits: crate::estimacion::Commits,
 }
 
 impl Importar {
@@ -73,6 +75,8 @@ impl Importar {
             fecha: alta::hoy(),
             ocupados: BTreeSet::new(),
             plan: RefCell::new(None),
+            ahora: crate::identidad::ahora(),
+            commits: crate::estimacion::Commits::vacios(),
         }
     }
 
@@ -83,6 +87,13 @@ impl Importar {
         self
     }
 
+    /// Los commits de `develop` para la estimacion de cada tarea (F3-13).
+    #[must_use]
+    pub fn con_commits(mut self, commits: crate::estimacion::Commits) -> Self {
+        self.commits = commits;
+        self
+    }
+
     /// El plan del ultimo intento.
     pub fn plan(&self) -> Option<Plan> {
         self.plan.borrow().clone()
@@ -106,7 +117,11 @@ impl Operacion for Importar {
     }
     fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
         let rutas = estado.rutas();
-        let tareas = alta::tareas(estado, &rutas)?;
+        // Una sola lectura de los `tarea.md` y un solo historico para todo
+        // el lote (F3-13).
+        let leidas = alta::leidas(estado, &rutas)?;
+        let historico = crate::estimacion::historico(&leidas, &self.commits);
+        let tareas = alta::de_formato2(&leidas);
         let prefijo = format!("{}/", self.lote);
         let del_lote = |t: &crate::tarea::Tarea| {
             t.lote
@@ -160,7 +175,16 @@ impl Operacion for Importar {
             .into_iter()
             .filter(|t| del_lote(t).is_none())
             .collect();
-        let a = alta::altas(&fuera, &ids, &entradas, &self.fecha)?;
+        let a = alta::altas(
+            &fuera,
+            &ids,
+            &entradas,
+            &self.fecha,
+            &alta::Calculo {
+                historico: &historico,
+                ahora: self.ahora,
+            },
+        )?;
         *self.plan.borrow_mut() = Some(Plan {
             creadas: ids.iter().map(|n| alta::formato_id(*n)).collect(),
             existentes,
@@ -312,7 +336,10 @@ fn importar_en(
             return i;
         }
     };
-    let op = Importar::nuevo(&nombre, sprint, lote.entradas).con_ocupados(ocupados);
+    // La historia de `develop` para la estimacion (F3-13), una vez.
+    let op = Importar::nuevo(&nombre, sprint, lote.entradas)
+        .con_ocupados(ocupados)
+        .con_commits(crate::estimacion::commits_de(g, dir));
     if o.simular {
         return simular(g, dir, &op, parecidas);
     }
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 566de32..48633c5 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -31,6 +31,7 @@ pub mod descarga;
 pub mod desinstalar;
 pub mod doctor;
 pub mod estado;
+pub mod estimacion;
 pub mod git;
 pub mod grafo;
 pub mod huella;
@@ -181,6 +182,10 @@ where
             de_una_tarea(ciclo::REASIGNAR, |e| ciclo::orden_reasignar(&resto, e))
                 .salida_en(json, idioma)
         }
+        [orden, resto @ ..] if orden == estimacion::NOMBRE => {
+            let (json, resto) = con_json(resto);
+            orden_estimar(&resto, json, idioma)
+        }
         [orden, resto @ ..] if orden == identidad::NOMBRE => {
             let (json, resto) = con_json(resto);
             orden_identidad(&resto).salida_en(json, idioma)
@@ -346,6 +351,34 @@ fn de_una_tarea(nombre: &str, orden: impl FnOnce(&toma::Entorno<'_>) -> Informe)
     orden(&e)
 }
 
+/// `estimar [--json] [TASK-NNN | --error TASK-NNN]` (F3-13) en la carpeta
+/// actual. Sin `--json` y con exito, la linea para la persona va detras de
+/// la de contrato (como mucho 5 lineas, F1-4 decision 4).
+fn orden_estimar(args: &[String], json: bool, idioma: Idioma) -> Salida {
+    let dir = match std::env::current_dir() {
+        Ok(d) => d,
+        Err(e) => {
+            return Informe::nuevo(estimacion::NOMBRE, Resultado::Error)
+                .diagnostico(Mensaje::nuevo(&txt::CARPETA_ACTUAL).con("error", e.to_string()))
+                .salida_en(json, idioma);
+        }
+    };
+    let g = git::Git::nuevo();
+    let e = estimacion::Entorno {
+        dir: &dir,
+        git: &g,
+        catalogo: &catalogo,
+        ahora: identidad::ahora(),
+    };
+    let (i, linea) = estimacion::orden(args, &e);
+    let mut s = i.salida_en(json, idioma);
+    if let (false, true, Some(l)) = (json, i.resultado.es_exito(), linea) {
+        s.stdout.push_str(&l);
+        s.stdout.push('\n');
+    }
+    s
+}
+
 /// `status` o `board` (F3-5) en la carpeta actual.
 fn orden_tablero(orden: &str, args: &[String], json: bool, idioma: Idioma) -> Salida {
     let dir = match tablero::carpeta_actual(orden) {
@@ -432,7 +465,10 @@ fn orden_new(args: &[String], parecidas: &mut Vec<alta::Parecida>) -> Informe {
             return i;
         }
     };
-    let op = alta::Alta::con_entrada(entrada).con_ocupados(ocupados);
+    // La historia de `develop` para la estimacion (F3-13), tambien antes.
+    let op = alta::Alta::con_entrada(entrada)
+        .con_ocupados(ocupados)
+        .con_commits(estimacion::commits_de(&g, &dir));
     let personas = roadmap::personas(&g, &dir);
     let generar = roadmap::generador(&personas);
     let c = estado::Contexto::nuevo(&g, &dir, &catalogo, Some(&generar));
@@ -452,6 +488,7 @@ fn orden_new(args: &[String], parecidas: &mut Vec<alta::Parecida>) -> Informe {
             i.claves
                 .insert(2, ("parecidas".to_owned(), contrato::Valor::Lista(ids)));
         }
+        i = estimacion::con_claves(i, op.estimacion().as_ref());
     }
     i
 }
diff --git a/crates/tc2/src/tarea/modelo.rs b/crates/tc2/src/tarea/modelo.rs
index 38b32b6..25c28bf 100644
--- a/crates/tc2/src/tarea/modelo.rs
+++ b/crates/tc2/src/tarea/modelo.rs
@@ -322,7 +322,8 @@ pub struct Tarea {
     pub asignado_a: Option<Asignado>,
     pub roles: Roles,
     /// Entrada `estimacion:` cruda (su forma la fija F1-17); `None` es `null`.
-    /// Privada: solo llega de una lectura, que la ha validado como YAML.
+    /// Privada: solo llega de una lectura, que la ha validado como YAML, o de
+    /// [`Tarea::poner_estimacion`].
     estimacion: Option<String>,
     pub tokens_diseno: Option<u64>,
     pub tokens_implementacion: Option<u64>,
@@ -520,6 +521,13 @@ impl Tarea {
         self.estimacion.as_deref()
     }
 
+    /// Pone la estimacion con la forma del punto 7 de F1-17 (F3-13): el
+    /// unico camino de escritura del campo. Lo escrito se vuelve a leer igual
+    /// (`estimacion` en bloque, YAML valido).
+    pub fn poner_estimacion(&mut self, e: &crate::estimacion::Estimacion) {
+        self.estimacion = Some(e.yaml());
+    }
+
     /// Lee el frontmatter de formato 2 con el esquema estricto.
     pub(crate) fn leer(partes: &Partes<'_>) -> Result<Self, ErrorTarea> {
         faltan(partes, &CLAVES)?;
diff --git a/crates/tc2/src/toma/mod.rs b/crates/tc2/src/toma/mod.rs
index 72f6bba..77f82c3 100644
--- a/crates/tc2/src/toma/mod.rs
+++ b/crates/tc2/src/toma/mod.rs
@@ -133,7 +133,10 @@ fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
     // 2. Quien la toma.
     let quien = (e.quien)()?;
     // 3. La toma en seco sobre lo leido: los rechazos, sin esperar al push.
-    let op = Tomar::nueva(id, &quien, e.ahora);
+    // La historia de `develop` para la estimacion (F3-13), una vez y antes
+    // de la transaccion.
+    let op =
+        Tomar::nueva(id, &quien, e.ahora).con_commits(crate::estimacion::commits_de(g, &principal));
     let empezada = match op.aplicar(&vista) {
         Ok(c) => c.is_empty(),
         Err(Fallo::Rechazada {
@@ -189,7 +192,9 @@ fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
     match t.resultado {
         Resultado::Ok => {
             let commit = t.valor("commit").map(str::to_owned);
-            Ok(destino.crear(commit, &quien))
+            let i = destino.crear(commit, &quien);
+            // Las claves de la estimacion publicada (F3-13).
+            Ok(crate::estimacion::con_claves(i, op.estimacion().as_ref()))
         }
         // Entre la lectura y el push paso a ser suya y empezada.
         Resultado::SinCambios => Ok(destino.crear(None, &quien)),
diff --git a/crates/tc2/src/toma/operacion.rs b/crates/tc2/src/toma/operacion.rs
index 75d8e98..8de9035 100644
--- a/crates/tc2/src/toma/operacion.rs
+++ b/crates/tc2/src/toma/operacion.rs
@@ -3,7 +3,8 @@
 //!
 //! [`Tomar`] pasa por el punto unico de escritura ([`crate::estado::transaccion`])
 //! y se re-ejecuta en cada intento: decide solo con el estado que recibe.
-//! Lee unicamente el `tarea.md` de la tarea y los de sus dependencias.
+//! Lee el `tarea.md` de la tarea y los de sus dependencias para decidir y,
+//! si la toma procede, todos para el historico de la estimacion (F3-13).
 //!
 //! - Ya la lleva quien la toma y esta en curso o en revision: no cambia
 //!   nada (`sin-cambios`, sin commit).
@@ -14,8 +15,9 @@
 //! - Dependencias (D3): con la regla de [`crate::grafo::dependencia_cumplida`];
 //!   alguna sin cumplir, `rechazada motivo=dependencias-pendientes`.
 //! - Si procede escribe `estado: en-curso`, `asignado_a`, `actualizado`,
-//!   `rama` si faltaba y una fila nueva en `## Transiciones`; el resto del
-//!   `tarea.md`, byte a byte.
+//!   `rama` si faltaba, la `estimacion` recalculada (F3-13: lee entonces
+//!   todos los `tarea.md` para el historico) y una fila nueva en
+//!   `## Transiciones`; el resto del `tarea.md`, byte a byte.
 //!
 //! Crear la rama y el worktree no es cosa de la operacion: lo hace `start`
 //! fuera de la transaccion y solo con la toma publicada.
@@ -23,6 +25,7 @@
 use std::cell::RefCell;
 
 use crate::estado::{Cambio, Estado, Fallo, Operacion, de_otra_persona, por_estado, tarea_de};
+use crate::estimacion::{Commits, Estimacion};
 use crate::grafo::{Nodo, dependencia_cumplida};
 use crate::i18n::Mensaje;
 use crate::i18n::catalogo::toma as txt;
@@ -52,9 +55,11 @@ pub struct Tomar {
     asignado: Asignado,
     decidido_por: &'static str,
     ahora: u64,
+    commits: Commits,
     dueno: RefCell<Option<String>>,
     pendientes: RefCell<Vec<String>>,
     rama: RefCell<Option<String>>,
+    estimacion: RefCell<Option<Estimacion>>,
 }
 
 impl Tomar {
@@ -70,12 +75,27 @@ impl Tomar {
             asignado,
             decidido_por,
             ahora,
+            commits: Commits::vacios(),
             dueno: RefCell::new(None),
             pendientes: RefCell::new(Vec::new()),
             rama: RefCell::new(None),
+            estimacion: RefCell::new(None),
         }
     }
 
+    /// Los commits de `develop` para el historico de la estimacion (un
+    /// `git log` leido antes de la transaccion, F3-13).
+    #[must_use]
+    pub fn con_commits(mut self, commits: Commits) -> Self {
+        self.commits = commits;
+        self
+    }
+
+    /// La estimacion escrita en el ultimo intento (F3-13).
+    pub fn estimacion(&self) -> Option<Estimacion> {
+        self.estimacion.borrow().clone()
+    }
+
     /// Login de quien la llevaba si el ultimo intento la rechazo por
     /// `tomada`.
     pub fn dueno(&self) -> Option<String> {
@@ -121,6 +141,7 @@ impl Operacion for Tomar {
         *self.dueno.borrow_mut() = None;
         self.pendientes.borrow_mut().clear();
         *self.rama.borrow_mut() = None;
+        *self.estimacion.borrow_mut() = None;
         let ruta = format!("{}/tarea.md", self.id);
         let (mut t, mut cuerpo) = tarea_de(estado, &self.id)?;
         let rama = t
@@ -176,6 +197,12 @@ impl Operacion for Tomar {
             *self.pendientes.borrow_mut() = pendientes;
             return Err(self.rechazo("dependencias-pendientes", m));
         }
+        // La estimacion, recalculada con las terminadas del estado fresco
+        // (F3-13, D2a): `calculada` es la hora del `start`.
+        let leidas = crate::estado::tareas(estado, &estado.rutas())?;
+        let h = crate::estimacion::historico(&leidas, &self.commits);
+        let e = crate::estimacion::recalcular(&mut t, &cuerpo, &h, self.ahora);
+        *self.estimacion.borrow_mut() = Some(e);
         let instante = instante(self.ahora);
         EN_CURSO.clone_into(&mut t.estado);
         t.asignado_a = Some(self.asignado.clone());
@@ -240,6 +267,10 @@ mod tests {
     /// 2026-11-17T10:20:30Z.
     const AHORA: u64 = 1_794_910_830;
 
+    /// La estimacion que pone la toma sin ninguna terminada en el estado
+    /// (F3-13): `sin-datos`, calculada a la hora del `start`.
+    const SIN_DATOS: &str = "estimacion:\n  calculada: \"2026-11-17T10:20:30Z\"\n  nivel: \"sin-datos\"\n  n: 0\n  por_fase: null\n  total: null\n  agente: null\n";
+
     fn ana() -> Quien {
         Quien::Agente(Cuenta {
             proveedor: Proveedor::GitHub,
@@ -300,6 +331,7 @@ mod tests {
             .replace("estado: \"planificada\"", "estado: \"en-curso\"")
             .replace("asignado_a: null", "asignado_a: {id: 7, login: \"ana\"}")
             .replace("actualizado: \"2026-11-01\"", "actualizado: \"2026-11-17\"")
+            .replace("estimacion: null\n", SIN_DATOS)
             + &format!(
                 "| 2026-11-17T10:20:30Z | start | manual | agente | {} |\n",
                 crate::VERSION
@@ -344,6 +376,7 @@ mod tests {
             .replace("estado: \"en-diseno\"", "estado: \"en-curso\"")
             .replace("asignado_a: null", "asignado_a: {id: 7, login: \"ana\"}")
             .replace("actualizado: \"2026-11-01\"", "actualizado: \"2026-11-17\"")
+            .replace("estimacion: null\n", SIN_DATOS)
             + &format!(
                 "| 2026-11-17T10:20:30Z | start | manual | agente | {} |\n",
                 crate::VERSION
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-083/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-083/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-083/tarea.md                                    | 35 ++++++++++++++++++++++++++++++++---
 3 files changed, 32 insertions(+), 3 deletions(-)
````

## Tests del diff (3 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/comun/mod.rs" ":(literal)crates/tc2/tests/estimacion.rs" ":(literal)crates/tc2/tests/toma.rs"
````

````
crates/tc2/tests/comun/mod.rs  |   3 +-
 crates/tc2/tests/estimacion.rs | 784 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/toma.rs       |  16 +++-
 3 files changed, 801 insertions(+), 2 deletions(-)
````
