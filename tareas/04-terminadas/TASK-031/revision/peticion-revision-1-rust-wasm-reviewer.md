# Peticion de revision — TASK-031 (ronda 1)

- Tarea: TASK-031 — F3-4 tc2 roadmap: la vista para todos
- Rama revisada: feature/task-031-f3-4-roadmap-la-vista-para-todos
- Rama base: develop
- Commit revisado (HEAD): 8a54049c717b71525d0a180c8e00676686271db0
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-031 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-031/tarea.md
- tareas/03-en-revision/TASK-031/planificacion/
- crates/tc2/src/estado/mod.rs
- crates/tc2/src/estado/montaje.rs
- crates/tc2/src/estado/transaccion.rs
- crates/tc2/src/grafo.rs
- crates/tc2/src/i18n/catalogo/cli.rs
- crates/tc2/src/i18n/catalogo/mod.rs
- crates/tc2/src/i18n/catalogo/roadmap.rs
- crates/tc2/src/importar/mod.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/roadmap/generar.rs
- crates/tc2/src/roadmap/mermaid.rs
- crates/tc2/src/roadmap/mod.rs
- crates/tc2/src/roadmap/zona.rs
- crates/tc2/src/siguiente.rs
- crates/tc2/src/situacion.rs
- crates/tc2/tests/alta.rs
- crates/tc2/tests/convivencia_016.rs
- crates/tc2/tests/estado_bordes.rs
- crates/tc2/tests/estado_casos.rs
- crates/tc2/tests/estado_concurrencia.rs
- crates/tc2/tests/roadmap.rs

## Commits a revisar (git log develop..HEAD)

````
8a54049 chore(TASK-031): resultado y autorrevision
3a1dd0f refactor(TASK-031): un solo nombre para ROADMAP.md y ajustes de revision
84954b4 test(TASK-031): integracion de roadmap para C1 a C6
5802680 feat(TASK-031): orden tc2 roadmap con --comparar y --escribir
1527267 feat(TASK-031): new e import regeneran el ROADMAP en la transaccion
0b143f4 feat(TASK-031): generador puro del ROADMAP con Mermaid por hito
a17896a refactor(TASK-031): situacion comun y zona manual estricta
7f20b12 chore(TASK-031): tarea en curso
````

## Diff de tu dominio (21 fichero(s) de 23; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/estado/mod.rs b/crates/tc2/src/estado/mod.rs
index 4f88038..0008894 100644
--- a/crates/tc2/src/estado/mod.rs
+++ b/crates/tc2/src/estado/mod.rs
@@ -40,6 +40,10 @@ pub const RAMA: &str = crate::huella::RAMA_ESTADO;
 /// Carpeta del worktree principal donde se monta (ADR F1-1).
 pub const CARPETA: &str = "tareas";
 
+/// `ROADMAP.md` en la raiz de la rama de estado (ADR F1-1, decision 3): lo
+/// escribe el generador de la transaccion y lo leen `next` y `roadmap`.
+pub const ROADMAP: &str = "ROADMAP.md";
+
 /// Refs de las operaciones pendientes de publicar (sin red, D2b).
 pub const REFS_PENDIENTES: &str = crate::huella::REFS_PENDIENTES;
 
diff --git a/crates/tc2/src/estado/montaje.rs b/crates/tc2/src/estado/montaje.rs
index f9b767b..6f63724 100644
--- a/crates/tc2/src/estado/montaje.rs
+++ b/crates/tc2/src/estado/montaje.rs
@@ -261,7 +261,9 @@ fn paso(git: &Git, dir: &Path, politica: Politica, puede_crear: bool) -> Result<
         // La transaccion toma el cerrojo ella misma: se suelta antes y,
         // creada la rama, se vuelve a empezar para montarla.
         drop(cerrojo);
-        let mut c = Contexto::nuevo(git, &principal, &ninguna);
+        // Inicializar no lleva ROADMAP: el primero lo escribe la primera
+        // operacion (con la zona de por defecto).
+        let mut c = Contexto::nuevo(git, &principal, &ninguna, None);
         c.politica = politica;
         let i = transaccion(&c, &Inicializar);
         if !i.resultado.es_exito() {
diff --git a/crates/tc2/src/estado/transaccion.rs b/crates/tc2/src/estado/transaccion.rs
index 416fc3a..dd6eeca 100644
--- a/crates/tc2/src/estado/transaccion.rs
+++ b/crates/tc2/src/estado/transaccion.rs
@@ -70,7 +70,8 @@ pub trait Operacion {
 pub type Catalogo<'a> = &'a dyn Fn(&str) -> Option<Box<dyn Operacion>>;
 
 /// Genera `ROADMAP.md` desde el estado ya cambiado (se regenera, nunca se
-/// mergea: ADR F1-1). Lo pone F3-4; hasta entonces, `None`.
+/// mergea: ADR F1-1 y F1-2 D2.4). El de produccion es
+/// [`crate::roadmap::generador`]; el montaje no lleva.
 pub type Roadmap<'a> = &'a dyn Fn(&dyn Estado) -> Result<String, Fallo>;
 
 /// Reintentos y esperas (ADR F1-2, D3a: 5 intentos, base 2 s).
@@ -109,14 +110,20 @@ pub struct Contexto<'a> {
 }
 
 impl<'a> Contexto<'a> {
-    /// Contexto con la politica por defecto y sin generador de roadmap.
-    pub fn nuevo(git: &'a Git, dir: &'a Path, catalogo: Catalogo<'a>) -> Self {
+    /// Contexto con la politica por defecto. El generador de `ROADMAP.md`
+    /// va explicito: quien escribe decide si lo regenera (TASK-031).
+    pub fn nuevo(
+        git: &'a Git,
+        dir: &'a Path,
+        catalogo: Catalogo<'a>,
+        roadmap: Option<Roadmap<'a>>,
+    ) -> Self {
         Self {
             git,
             dir,
             politica: Politica::default(),
             catalogo,
-            roadmap: None,
+            roadmap,
         }
     }
 }
@@ -347,7 +354,7 @@ fn intento(
     anotaciones.push(anotacion.to_owned());
     if let Some(generar) = c.roadmap {
         let texto = generar(&vista).map_err(de_fallo)?;
-        let roadmap = vec![Cambio::nuevo("ROADMAP.md", texto)];
+        let roadmap = vec![Cambio::nuevo(super::ROADMAP, texto)];
         if let Some(i) = duplicada(&vista.rutas(), &roadmap, op) {
             return Ok(Intento::Hecho(i));
         }
diff --git a/crates/tc2/src/grafo.rs b/crates/tc2/src/grafo.rs
index a98f8e4..bfef091 100644
--- a/crates/tc2/src/grafo.rs
+++ b/crates/tc2/src/grafo.rs
@@ -20,6 +20,7 @@
 use std::cmp::Reverse;
 use std::collections::{BTreeMap, BTreeSet, BinaryHeap};
 
+use crate::roadmap::zona;
 use crate::tarea::Documento;
 
 /// Estado de una tarea `planificada` (la unica que puede estar lista).
@@ -109,42 +110,33 @@ impl Nodo {
 #[derive(Debug, Clone, Default, PartialEq, Eq)]
 pub struct Hitos {
     posicion: BTreeMap<u32, usize>,
+    titulos: BTreeMap<u32, String>,
 }
 
-/// Marca de inicio de la zona manual de `ROADMAP.md` (la de la 0.16).
-const MANUAL_INICIO: &str = "<!-- manual:inicio";
-/// Marca de fin de la zona manual.
-const MANUAL_FIN: &str = "<!-- manual:fin";
-
 impl Hitos {
     /// Hitos de las lineas `### Hn — titulo` de `texto`, en su orden; si un
     /// hito sale dos veces vale su primera posicion (como el oraculo).
     pub fn de_titulos(texto: &str) -> Self {
         let mut posicion = BTreeMap::new();
-        for h in texto.lines().filter_map(hito_de_linea) {
+        let mut titulos = BTreeMap::new();
+        for (h, titulo) in texto.lines().filter_map(hito_de_linea) {
             let n = posicion.len();
             posicion.entry(h).or_insert(n);
+            titulos.entry(h).or_insert_with(|| titulo.to_owned());
         }
-        Self { posicion }
+        Self { posicion, titulos }
     }
 
     /// Hitos de la zona manual (`<!-- manual:inicio ... -->` hasta
-    /// `<!-- manual:fin -->`) de un `ROADMAP.md`. Sin zona, ninguno.
+    /// `<!-- manual:fin -->`) de un `ROADMAP.md`, con la lectura tolerante de
+    /// [`zona::interior_tolerante`]. Sin zona, ninguno.
     pub fn del_roadmap(texto: &str) -> Self {
-        let mut dentro = false;
-        let mut zona = String::new();
-        for l in texto.lines() {
-            let t = l.trim_start();
-            if t.starts_with(MANUAL_INICIO) {
-                dentro = true;
-            } else if t.starts_with(MANUAL_FIN) {
-                break;
-            } else if dentro {
-                zona.push_str(l);
-                zona.push('\n');
-            }
-        }
-        Self::de_titulos(&zona)
+        Self::de_titulos(&zona::interior_tolerante(texto))
+    }
+
+    /// Titulo del hito `h`, si lo tiene (el de su primera linea).
+    pub fn titulo(&self, h: u32) -> Option<&str> {
+        self.titulos.get(&h).map(String::as_str)
     }
 
     /// Si no hay ningun titulo.
@@ -168,8 +160,9 @@ impl Hitos {
     }
 }
 
-/// `### H<n> — <titulo>` (raya U+2014 entre espacios, titulo no vacio).
-fn hito_de_linea(l: &str) -> Option<u32> {
+/// `### H<n> — <titulo>` (raya U+2014 entre espacios, titulo no vacio):
+/// el hito y su titulo sin espacios al final.
+fn hito_de_linea(l: &str) -> Option<(u32, &str)> {
     let resto = l.strip_prefix("### H")?;
     let fin = resto
         .find(|c: char| !c.is_ascii_digit())
@@ -179,7 +172,7 @@ fn hito_de_linea(l: &str) -> Option<u32> {
     if cifras.is_empty() || titulo.trim_end_matches('\r').is_empty() {
         return None;
     }
-    cifras.parse().ok()
+    Some((cifras.parse().ok()?, titulo.trim_end()))
 }
 
 /// Avisos del calculo: no frenan, pero se dicen.
@@ -212,7 +205,7 @@ pub struct Orden {
 type Clave = (u64, u64, String, String);
 
 /// Numero y sufijo de `TASK-NNNa`; sin cifras, al final.
-fn numero_y_sufijo(id: &str) -> (u64, String) {
+pub(crate) fn numero_y_sufijo(id: &str) -> (u64, String) {
     let resto = id.strip_prefix("TASK-").unwrap_or(id);
     let fin = resto
         .find(|c: char| !c.is_ascii_digit())
@@ -545,6 +538,10 @@ mod tests {
         assert_eq!([h.rango(2), h.rango(1)], [0, 1]);
         assert!(!h.tiene(9) && !h.tiene(8) && !h.tiene(3));
         assert!(Hitos::del_roadmap("### H1 — Sin zona\n").es_vacio());
+        assert_eq!(
+            (h.titulo(2), h.titulo(1), h.titulo(9)),
+            (Some("Dos"), Some("Uno"), None)
+        );
     }
 
     #[test]
diff --git a/crates/tc2/src/i18n/catalogo/cli.rs b/crates/tc2/src/i18n/catalogo/cli.rs
index ffc7f5c..bf908a1 100644
--- a/crates/tc2/src/i18n/catalogo/cli.rs
+++ b/crates/tc2/src/i18n/catalogo/cli.rs
@@ -10,8 +10,8 @@ crate::i18n::textos! {
         en: "tc2: could not write the output: {error}",
     }
     USO {
-        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
-        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
+        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
+        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
     }
     FALTA_LA_ORDEN {
         es: "tc2: falta la orden. {uso}",
diff --git a/crates/tc2/src/i18n/catalogo/mod.rs b/crates/tc2/src/i18n/catalogo/mod.rs
index 6ec4d46..adba285 100644
--- a/crates/tc2/src/i18n/catalogo/mod.rs
+++ b/crates/tc2/src/i18n/catalogo/mod.rs
@@ -12,6 +12,7 @@ pub mod identidad;
 pub mod importar;
 pub mod init;
 pub mod plataforma;
+pub mod roadmap;
 pub mod siguiente;
 
 use super::Texto;
@@ -30,6 +31,7 @@ pub fn todos() -> impl Iterator<Item = &'static Texto> {
         importar::TODOS,
         init::TODOS,
         plataforma::TODOS,
+        roadmap::TODOS,
         siguiente::TODOS,
     ]
     .into_iter()
diff --git a/crates/tc2/src/i18n/catalogo/roadmap.rs b/crates/tc2/src/i18n/catalogo/roadmap.rs
new file mode 100644
index 0000000..fb3ceff
--- /dev/null
+++ b/crates/tc2/src/i18n/catalogo/roadmap.rs
@@ -0,0 +1,16 @@
+//! Mensajes de `tc2 roadmap` (F3-4).
+
+crate::i18n::textos! {
+    USO_ROADMAP {
+        es: "uso: roadmap [--comparar | --escribir] [--json]",
+        en: "usage: roadmap [--comparar | --escribir] [--json]",
+    }
+    DESFASADO {
+        es: "el ROADMAP.md publicado no esta al dia: ejecuta tc2 roadmap --escribir",
+        en: "the published ROADMAP.md is out of date: run tc2 roadmap --escribir",
+    }
+    ZONA_ROTA {
+        es: "la zona manual de {donde} esta rota ({motivo}): deja una sola zona que abra con «<!-- manual:inicio -->» y cierre con «<!-- manual:fin -->», y publicala con tc2 roadmap --escribir",
+        en: "the manual zone in {donde} is broken ({motivo}): keep a single zone that opens with «<!-- manual:inicio -->» and closes with «<!-- manual:fin -->», and publish it with tc2 roadmap --escribir",
+    }
+}
diff --git a/crates/tc2/src/importar/mod.rs b/crates/tc2/src/importar/mod.rs
index d961e7c..1e5c715 100644
--- a/crates/tc2/src/importar/mod.rs
+++ b/crates/tc2/src/importar/mod.rs
@@ -316,7 +316,9 @@ fn importar_en(
     if o.simular {
         return simular(g, dir, &op, parecidas);
     }
-    let c = estado::Contexto::nuevo(g, dir, catalogo);
+    let personas = crate::roadmap::personas(g, dir);
+    let generar = crate::roadmap::generador(&personas);
+    let c = estado::Contexto::nuevo(g, dir, catalogo, Some(&generar));
     let mut i = estado::transaccion(&c, &op);
     NOMBRE.clone_into(&mut i.comando);
     alta::referencia_a_uso(&mut i);
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 9d5da4a..35e01be 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -14,7 +14,8 @@
 //! [`instalacion`] (donde vive el `tc2` de la
 //! maquina, F2-12 y F2-14) y [`desinstalar`] (quitar TaskCodeTwo, F2-14) e
 //! [`importar`] (alta en bloque con dependencias e hito, F3-2) y [`grafo`] y
-//! [`siguiente`] (orden de trabajo y `next`, F3-3). Las capas
+//! [`siguiente`] (orden de trabajo y `next`, F3-3) y [`situacion`] y
+//! [`roadmap`] (el calculo comun y `roadmap`, F3-4). Las capas
 //! que vienen entran como modulos de esta biblioteca.
 
 pub mod actualizar_cli;
@@ -34,7 +35,9 @@ pub mod importar;
 pub mod init;
 pub mod instalacion;
 pub mod plataforma;
+pub mod roadmap;
 pub mod siguiente;
+pub mod situacion;
 pub mod tarea;
 pub mod version;
 mod yaml;
@@ -113,6 +116,10 @@ where
             let (json, resto) = con_json(resto);
             siguiente::orden(&resto).salida_en(json, idioma)
         }
+        [orden, resto @ ..] if orden == roadmap::NOMBRE => {
+            let (json, resto) = con_json(resto);
+            roadmap::orden(&resto, json, idioma, &catalogo)
+        }
         [orden, resto @ ..] if orden == version::VERSION => {
             let (json, resto) = con_json(resto);
             orden_version(json, &resto, idioma)
@@ -267,7 +274,7 @@ fn orden_identidad(args: &[String]) -> Informe {
 /// Catalogo de produccion: reconstruye las operaciones de la rama de estado
 /// desde su anotacion. Las operaciones nuevas se anaden aqui.
 fn catalogo(anotacion: &str) -> Option<Box<dyn estado::Operacion>> {
-    alta::desde_anotacion(anotacion)
+    alta::desde_anotacion(anotacion).or_else(|| roadmap::desde_anotacion(anotacion))
 }
 
 /// `new "<titulo>" [campos de RF-01]`: alta de una tarea con su ID reservado
@@ -301,7 +308,9 @@ fn orden_new(args: &[String], parecidas: &mut Vec<alta::Parecida>) -> Informe {
         }
     };
     let op = alta::Alta::con_entrada(entrada).con_ocupados(ocupados);
-    let c = estado::Contexto::nuevo(&g, &dir, &catalogo);
+    let personas = roadmap::personas(&g, &dir);
+    let generar = roadmap::generador(&personas);
+    let c = estado::Contexto::nuevo(&g, &dir, &catalogo, Some(&generar));
     let mut i = estado::transaccion(&c, &op);
     alta::NOMBRE.clone_into(&mut i.comando);
     alta::referencia_a_uso(&mut i);
diff --git a/crates/tc2/src/roadmap/generar.rs b/crates/tc2/src/roadmap/generar.rs
new file mode 100644
index 0000000..a0a845f
--- /dev/null
+++ b/crates/tc2/src/roadmap/generar.rs
@@ -0,0 +1,389 @@
+//! Texto del `ROADMAP.md` 2.0 desde una [`Situacion`] (ADR F1-1, decision
+//! 3): cabecera con la marca de formato, la zona manual byte a byte, el
+//! estado del trabajo (las mismas listas que `next`), en curso con quien la
+//! lleva, en revision, un apartado por hito no cerrado con su tabla y su
+//! Mermaid, y las terminadas y descartadas plegadas.
+//!
+//! Hoja pura: LF y sin fecha, para que los mismos datos den los mismos bytes
+//! (`roadmap --escribir` dos veces seguidas da `sin-cambios`).
+
+use std::collections::{BTreeMap, BTreeSet};
+use std::fmt::Write as _;
+
+use super::mermaid;
+use crate::grafo::{self, Fase, PLANIFICADA, TERMINADA};
+use crate::identidad::convertir_heredado;
+use crate::plataforma::Personas;
+use crate::situacion::{Ficha, Situacion};
+use crate::tarea::Asignado;
+
+/// Tercera linea del fichero: lo generado y su formato.
+pub const MARCA: &str = "<!-- generado por tc2; formato 2; no editar fuera de la zona manual -->";
+
+/// Lo que se consulta al escribir.
+struct Datos<'a> {
+    s: &'a Situacion,
+    personas: &'a Personas,
+    fichas: BTreeMap<&'a str, &'a Ficha>,
+    paso: BTreeMap<&'a str, usize>,
+    listas: BTreeSet<&'a str>,
+    atascadas: BTreeSet<&'a str>,
+    ilegibles: BTreeSet<&'a str>,
+}
+
+impl<'a> Datos<'a> {
+    fn nuevos(s: &'a Situacion, personas: &'a Personas) -> Self {
+        let mut fichas = BTreeMap::new();
+        for f in &s.fichas {
+            fichas.entry(f.nodo.id.as_str()).or_insert(f);
+        }
+        Self {
+            s,
+            personas,
+            fichas,
+            paso: s
+                .orden
+                .orden
+                .iter()
+                .enumerate()
+                .map(|(i, id)| (id.as_str(), i + 1))
+                .collect(),
+            listas: s.listas.iter().map(String::as_str).collect(),
+            atascadas: s.orden.atascadas.iter().map(String::as_str).collect(),
+            ilegibles: s.ilegibles.iter().map(String::as_str).collect(),
+        }
+    }
+
+    /// Fichas vivas y no terminadas, en el orden de trabajo (las atascadas
+    /// detras, por numero).
+    fn abiertas(&self, filtro: impl Fn(&Ficha) -> bool) -> Vec<&'a Ficha> {
+        let mut v: Vec<&Ficha> = self
+            .fichas
+            .values()
+            .copied()
+            .filter(|f| matches!(f.nodo.estado(), Some(e) if e != TERMINADA))
+            .filter(|f| filtro(f))
+            .collect();
+        v.sort_by_key(|f| {
+            (
+                self.paso
+                    .get(f.nodo.id.as_str())
+                    .copied()
+                    .unwrap_or(usize::MAX),
+                grafo::numero_y_sufijo(&f.nodo.id),
+            )
+        });
+        v
+    }
+
+    /// Texto del estado y clase Mermaid de una tarea viva.
+    fn estado(&self, f: &Ficha) -> (String, &'static str) {
+        let id = f.nodo.id.as_str();
+        let e = f.nodo.estado().unwrap_or_default();
+        if self.atascadas.contains(id) {
+            return ("atascada (ciclo)".to_owned(), "atascada");
+        }
+        match e {
+            TERMINADA => ("terminada".to_owned(), "terminada"),
+            "en-diseno" => ("en diseño".to_owned(), "marcha"),
+            "en-curso" => ("en curso".to_owned(), "marcha"),
+            "en-revision" => ("en revisión".to_owned(), "marcha"),
+            PLANIFICADA if self.listas.contains(id) => ("lista para empezar".to_owned(), "lista"),
+            PLANIFICADA => {
+                let faltan: Vec<String> = deps(f)
+                    .into_iter()
+                    .filter_map(|d| self.pendiente(d))
+                    .collect();
+                if faltan.is_empty() {
+                    ("planificada".to_owned(), "espera")
+                } else {
+                    (format!("espera a {}", faltan.join(", ")), "espera")
+                }
+            }
+            otro => (otro.to_owned(), "espera"),
+        }
+    }
+
+    /// `d` si bloquea (existe y no esta terminada), con lo que es si no
+    /// esta viva.
+    fn pendiente(&self, d: &str) -> Option<String> {
+        if self.ilegibles.contains(d) && !self.fichas.contains_key(d) {
+            return Some(format!("{d} (ilegible)"));
+        }
+        let f = self.fichas.get(d)?;
+        match &f.nodo.fase {
+            Fase::Viva(e) if e == TERMINADA => None,
+            Fase::Viva(_) => Some(d.to_owned()),
+            Fase::Descartada => Some(format!("{d} (descartada)")),
+            Fase::Ilegible => Some(format!("{d} (ilegible)")),
+        }
+    }
+
+    fn con_quien(&self, a: Option<&Asignado>) -> String {
+        let Some(a) = a else {
+            return "—".to_owned();
+        };
+        let a = convertir_heredado(a.clone(), self.personas);
+        match (a.login(), a.heredado()) {
+            (Some(l), _) => format!("@{l}"),
+            (None, Some(h)) => celda(h),
+            (None, None) => "—".to_owned(),
+        }
+    }
+}
+
+/// Dependencias sin repetir, en su orden.
+fn deps(f: &Ficha) -> Vec<&str> {
+    let mut vistas = BTreeSet::new();
+    f.nodo
+        .dependencias
+        .iter()
+        .map(String::as_str)
+        .filter(|d| vistas.insert(*d))
+        .collect()
+}
+
+/// Texto de una celda de tabla o de una linea de lista: sin `|` que parta
+/// la fila ni `<` que abra una etiqueta o un comentario HTML.
+fn celda(t: &str) -> String {
+    t.replace('|', "\\|").replace('<', "&lt;")
+}
+
+fn lista_o_ninguna(v: &[String]) -> String {
+    if v.is_empty() {
+        "ninguna".to_owned()
+    } else {
+        v.join(", ")
+    }
+}
+
+fn depende_de(f: &Ficha) -> String {
+    let d = deps(f);
+    if d.is_empty() {
+        "—".to_owned()
+    } else {
+        d.join(", ")
+    }
+}
+
+/// El `ROADMAP.md` de `s` con la zona manual `zona` (byte a byte).
+pub fn generar(s: &Situacion, zona: &str, personas: &Personas) -> String {
+    let d = Datos::nuevos(s, personas);
+    let mut o = String::from("# Roadmap de tareas\n\n");
+    o.push_str(MARCA);
+    o.push_str("\n\n");
+    o.push_str(zona);
+    if !zona.ends_with('\n') {
+        o.push('\n');
+    }
+    estado_del_trabajo(&d, &mut o);
+    en_marcha(&d, &mut o, "En curso", &["en-diseno", "en-curso"]);
+    en_marcha(&d, &mut o, "En revisión", &["en-revision"]);
+    hitos(&d, &mut o);
+    plegadas(&d, &mut o, "Terminadas", |f| {
+        f.nodo.estado() == Some(TERMINADA)
+    });
+    plegadas(&d, &mut o, "Descartadas", |f| {
+        f.nodo.fase == Fase::Descartada
+    });
+    o
+}
+
+fn estado_del_trabajo(d: &Datos<'_>, o: &mut String) {
+    let s = d.s;
+    let marcha: Vec<String> = s
+        .en_marcha
+        .iter()
+        .map(|id| match d.fichas.get(id.as_str()) {
+            Some(f) => format!("{id} ({})", d.estado(f).0),
+            None => id.clone(),
+        })
+        .collect();
+    o.push_str("\n## Estado del trabajo\n\n");
+    o.push_str(
+        "Orden en que se hacen las tareas: ninguna va antes que aquellas de las que depende y, a igualdad, va primero la del hito anterior (en el orden de los `### Hn — título` de la zona manual) y luego la de número menor. tc2 lo regenera en cada cambio de la rama de estado; solo la zona manual se edita a mano.\n\n",
+    );
+    let _ = writeln!(o, "- **En marcha:** {}", lista_o_ninguna(&marcha));
+    let _ = writeln!(
+        o,
+        "- **Listas para empezar:** {}",
+        lista_o_ninguna(&s.listas)
+    );
+    let a = &s.orden.avisos;
+    let hitos: Vec<String> = a.hitos_sin_titulo.iter().map(|h| format!("H{h}")).collect();
+    for (titulo, v) in [
+        ("Atascadas (ciclo)", &s.orden.atascadas),
+        ("Ilegibles", &s.ilegibles),
+        ("Dependencias inexistentes", &a.dependencias_inexistentes),
+        ("Dependen de una descartada", &a.depende_de_descartada),
+        ("Hitos sin título", &hitos),
+        ("Duplicadas", &a.duplicadas),
+    ] {
+        if !v.is_empty() {
+            let _ = writeln!(o, "- **{titulo}:** {}", v.join(", "));
+        }
+    }
+}
+
+fn en_marcha(d: &Datos<'_>, o: &mut String, titulo: &str, estados: &[&str]) {
+    let _ = writeln!(o, "\n## {titulo}\n");
+    let v = d.abiertas(|f| f.nodo.estado().is_some_and(|e| estados.contains(&e)));
+    if v.is_empty() {
+        o.push_str("Ninguna.\n");
+        return;
+    }
+    o.push_str("| Tarea | Título | Con quién | Hito | Depende de |\n|---|---|---|---|---|\n");
+    for f in v {
+        let _ = writeln!(
+            o,
+            "| {} | {} | {} | H{} | {} |",
+            f.nodo.id,
+            celda(&f.titulo),
+            d.con_quien(f.asignado.as_ref()),
+            f.nodo.sprint,
+            depende_de(f)
+        );
+    }
+}
+
+fn hitos(d: &Datos<'_>, o: &mut String) {
+    let abiertos: BTreeSet<u32> = d.abiertas(|_| true).iter().map(|f| f.nodo.sprint).collect();
+    let mut orden: Vec<u32> = abiertos.into_iter().collect();
+    orden.sort_by_key(|h| (d.s.hitos.rango(*h), *h));
+    for h in orden {
+        hito(d, o, h);
+    }
+}
+
+fn hito(d: &Datos<'_>, o: &mut String, h: u32) {
+    match d.s.hitos.titulo(h) {
+        Some(t) => {
+            let _ = writeln!(o, "\n## H{h} — {t}\n");
+        }
+        None => {
+            let _ = writeln!(o, "\n## H{h}\n");
+        }
+    }
+    let vivas: Vec<&Ficha> = d
+        .fichas
+        .values()
+        .copied()
+        .filter(|f| f.nodo.sprint == h && f.nodo.estado().is_some())
+        .collect();
+    let hechas = vivas
+        .iter()
+        .filter(|f| f.nodo.estado() == Some(TERMINADA))
+        .count();
+    let _ = writeln!(o, "{hechas} de {} terminadas.\n", vivas.len());
+    o.push_str(
+        "| Paso | Tarea | Título | Depende de | Estado | Con quién |\n|---|---|---|---|---|---|\n",
+    );
+    let abiertas = d.abiertas(|f| f.nodo.sprint == h);
+    for f in &abiertas {
+        let paso = d
+            .paso
+            .get(f.nodo.id.as_str())
+            .map_or_else(|| "—".to_owned(), ToString::to_string);
+        let _ = writeln!(
+            o,
+            "| {paso} | {} | {} | {} | {} | {} |",
+            f.nodo.id,
+            celda(&f.titulo),
+            depende_de(f),
+            celda(&d.estado(f).0),
+            d.con_quien(f.asignado.as_ref())
+        );
+    }
+    o.push('\n');
+    o.push_str(&grafo_del_hito(d, &abiertas));
+}
+
+/// Nodos: las abiertas del hito y, de borde, las dependencias que no lo
+/// son (de otro hito, terminadas, descartadas o ilegibles).
+fn grafo_del_hito(d: &Datos<'_>, abiertas: &[&Ficha]) -> String {
+    let dentro: BTreeSet<&str> = abiertas.iter().map(|f| f.nodo.id.as_str()).collect();
+    let mut nodos: Vec<mermaid::Nodo> = abiertas
+        .iter()
+        .map(|f| mermaid::Nodo {
+            id: f.nodo.id.clone(),
+            titulo: f.titulo.clone(),
+            clase: d.estado(f).1,
+        })
+        .collect();
+    let mut borde: BTreeMap<&str, mermaid::Nodo> = BTreeMap::new();
+    let mut aristas = Vec::new();
+    for f in abiertas {
+        for dep in deps(f) {
+            if !dentro.contains(dep) && !borde.contains_key(dep) {
+                let nodo = match d.fichas.get(dep) {
+                    // Sin titulo: sale en su hito o en las plegadas, y asi
+                    // el fichero cabe (C6).
+                    Some(g) => mermaid::Nodo {
+                        id: dep.to_owned(),
+                        titulo: String::new(),
+                        clase: match &g.nodo.fase {
+                            Fase::Descartada => "descartada",
+                            Fase::Viva(e) if e == TERMINADA => "terminada",
+                            _ => "borde",
+                        },
+                    },
+                    None if d.ilegibles.contains(dep) => mermaid::Nodo {
+                        id: dep.to_owned(),
+                        titulo: "(ilegible)".to_owned(),
+                        clase: "borde",
+                    },
+                    // Inexistente: ya se avisa arriba; no se dibuja.
+                    None => continue,
+                };
+                borde.insert(dep, nodo);
+            }
+            aristas.push((dep.to_owned(), f.nodo.id.clone()));
+        }
+    }
+    nodos.extend(borde.into_values());
+    mermaid::bloque(&nodos, &aristas)
+}
+
+fn plegadas(d: &Datos<'_>, o: &mut String, titulo: &str, filtro: impl Fn(&Ficha) -> bool) {
+    let mut v: Vec<&Ficha> = d.fichas.values().copied().filter(|f| filtro(f)).collect();
+    v.sort_by_key(|f| grafo::numero_y_sufijo(&f.nodo.id));
+    let _ = writeln!(o, "\n## {titulo}\n");
+    if v.is_empty() {
+        o.push_str("Ninguna.\n");
+        return;
+    }
+    let _ = writeln!(
+        o,
+        "Total: {}.\n\n<details>\n<summary>Ver la lista</summary>\n",
+        v.len()
+    );
+    for f in v {
+        let _ = writeln!(
+            o,
+            "- {} — {} (H{})",
+            f.nodo.id,
+            celda(&f.titulo),
+            f.nodo.sprint
+        );
+    }
+    o.push_str("\n</details>\n");
+}
+
+/// Las tareas de una linea `- **<titulo>:** TASK-1 (x), TASK-2` de un
+/// ROADMAP (`ninguna`: vacia). Para comparar con `next` (C4).
+pub fn ids_de_la_linea(roadmap: &str, titulo: &str) -> Option<Vec<String>> {
+    let prefijo = format!("- **{titulo}:** ");
+    let resto = roadmap
+        .lines()
+        .find_map(|l| l.trim_end_matches('\r').strip_prefix(prefijo.as_str()))?;
+    if resto == "ninguna" {
+        return Some(Vec::new());
+    }
+    Some(
+        resto
+            .split(", ")
+            .filter_map(|p| p.split(' ').next())
+            .map(str::to_owned)
+            .collect(),
+    )
+}
diff --git a/crates/tc2/src/roadmap/mermaid.rs b/crates/tc2/src/roadmap/mermaid.rs
new file mode 100644
index 0000000..b261956
--- /dev/null
+++ b/crates/tc2/src/roadmap/mermaid.rs
@@ -0,0 +1,168 @@
+//! Bloque Mermaid de un hito (ADR F1-1, decision 3: un grafo por hito no
+//! cerrado). Acotado a lo que GitHub y GitLab pintan: menos de
+//! [`MAX_ARISTAS`] aristas y de [`MAX_CARACTERES`] caracteres (C6 de
+//! TASK-031); si no cabe, una nota en su lugar.
+//!
+//! Hoja pura.
+
+use std::fmt::Write as _;
+
+/// Aristas por bloque por debajo de las que se dibuja (`maxEdges` de Mermaid
+/// es 500).
+pub const MAX_ARISTAS: usize = 500;
+/// Caracteres por bloque por debajo de los que se dibuja (`maxTextSize` de
+/// Mermaid es 50 000).
+pub const MAX_CARACTERES: usize = 50_000;
+
+/// Clases de los nodos, por estado; en cada bloque, solo las que usa.
+const CLASES: [(&str, &str); 7] = [
+    ("lista", "fill:#d4f7d4,stroke:#2e7d32"),
+    ("marcha", "fill:#d6e9ff,stroke:#1565c0"),
+    ("espera", "fill:#f3f3f3,stroke:#888888"),
+    ("atascada", "fill:#ffd6d6,stroke:#c62828"),
+    ("terminada", "fill:#e8e8e8,stroke:#9e9e9e,color:#555555"),
+    (
+        "descartada",
+        "fill:#ffffff,stroke:#bbbbbb,stroke-dasharray:4 3,color:#888888",
+    ),
+    ("borde", "fill:#ffffff,stroke:#999999,stroke-dasharray:2 2"),
+];
+
+/// Un nodo del grafo.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Nodo {
+    /// `TASK-NNN`.
+    pub id: String,
+    /// Titulo (se escapa aqui); vacio, la etiqueta es solo el ID.
+    pub titulo: String,
+    /// Clase: `lista`, `marcha`, `espera`, `atascada`, `terminada`,
+    /// `descartada` o `borde`.
+    pub clase: &'static str,
+}
+
+/// Identificador Mermaid de `TASK-NNN[a]`: `TNNN[a]`; cualquier otro
+/// caracter, `_`.
+pub fn id_de(id: &str) -> String {
+    let (prefijo, resto) = match id.strip_prefix("TASK-") {
+        Some(r) => ("T", r),
+        None => ("N_", id),
+    };
+    let limpio: String = resto
+        .chars()
+        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
+        .collect();
+    format!("{prefijo}{limpio}")
+}
+
+/// Texto de una etiqueta entre comillas, con los caracteres que Mermaid
+/// interpreta como codigos de entidad.
+pub fn escapar(texto: &str) -> String {
+    let mut s = String::with_capacity(texto.len());
+    for c in texto.chars() {
+        match c {
+            '#' => s.push_str("#35;"),
+            '"' => s.push_str("#quot;"),
+            '<' => s.push_str("#lt;"),
+            '>' => s.push_str("#gt;"),
+            '`' => s.push_str("#96;"),
+            c if c.is_control() => s.push(' '),
+            c => s.push(c),
+        }
+    }
+    s
+}
+
+/// El bloque ```` ```mermaid ```` con `nodos` y `aristas` (`(de, a)`: `a`
+/// depende de `de`), o una nota si pasa de los limites.
+pub fn bloque(nodos: &[Nodo], aristas: &[(String, String)]) -> String {
+    let mut s = String::from("```mermaid\nflowchart TD\n");
+    for n in nodos {
+        let _ = if n.titulo.is_empty() {
+            writeln!(s, "  {}[\"{}\"]:::{}", id_de(&n.id), n.id, n.clase)
+        } else {
+            writeln!(
+                s,
+                "  {}[\"{} · {}\"]:::{}",
+                id_de(&n.id),
+                n.id,
+                escapar(&n.titulo),
+                n.clase
+            )
+        };
+    }
+    for (de, a) in aristas {
+        let _ = writeln!(s, "  {} --> {}", id_de(de), id_de(a));
+    }
+    for (clase, estilo) in CLASES {
+        if nodos.iter().any(|n| n.clase == clase) {
+            let _ = writeln!(s, "  classDef {clase} {estilo}");
+        }
+    }
+    s.push_str("```\n");
+    if aristas.len() >= MAX_ARISTAS || s.chars().count() >= MAX_CARACTERES {
+        return format!(
+            "_Grafo de este hito demasiado grande para dibujarlo ({} nodos, {} aristas)._\n",
+            nodos.len(),
+            aristas.len()
+        );
+    }
+    s
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn ids_y_etiquetas_escapadas() {
+        assert_eq!(id_de("TASK-031"), "T031");
+        assert_eq!(id_de("TASK-012a"), "T012a");
+        assert_eq!(id_de("raro-1"), "N_raro_1");
+        assert_eq!(
+            escapar("a \"b\" <c> #1 `d`\te"),
+            "a #quot;b#quot; #lt;c#gt; #35;1 #96;d#96; e"
+        );
+    }
+
+    #[test]
+    fn bloque_con_nodos_aristas_y_clases() {
+        let n = |id: &str, clase| Nodo {
+            id: id.to_owned(),
+            titulo: "t".to_owned(),
+            clase,
+        };
+        let b = bloque(
+            &[n("TASK-001", "borde"), n("TASK-002", "lista")],
+            &[("TASK-001".to_owned(), "TASK-002".to_owned())],
+        );
+        assert!(b.starts_with("```mermaid\nflowchart TD\n  T001[\"TASK-001 · t\"]:::borde\n"));
+        assert!(b.contains("  T002[\"TASK-002 · t\"]:::lista\n  T001 --> T002\n"));
+        assert!(b.ends_with(
+            "  classDef lista fill:#d4f7d4,stroke:#2e7d32\n  classDef borde fill:#ffffff,stroke:#999999,stroke-dasharray:2 2\n```\n"
+        ));
+        let sin_titulo = Nodo {
+            titulo: String::new(),
+            ..n("TASK-003", "borde")
+        };
+        assert!(bloque(&[sin_titulo], &[]).contains("  T003[\"TASK-003\"]:::borde\n"));
+    }
+
+    /// C6: si no cabe, una nota en vez de un grafo que no se pinta.
+    #[test]
+    fn grafo_demasiado_grande_es_una_nota() {
+        let nodos: Vec<Nodo> = (0..=MAX_ARISTAS)
+            .map(|i| Nodo {
+                id: format!("TASK-{i:04}"),
+                titulo: "t".to_owned(),
+                clase: "lista",
+            })
+            .collect();
+        let aristas: Vec<(String, String)> = (0..MAX_ARISTAS)
+            .map(|i| (format!("TASK-{i:04}"), format!("TASK-{:04}", i + 1)))
+            .collect();
+        let b = bloque(&nodos, &aristas);
+        assert!(!b.contains("```"), "{b}");
+        assert!(b.contains("500 aristas"));
+        assert!(bloque(&nodos, &aristas[..MAX_ARISTAS - 1]).starts_with("```mermaid"));
+    }
+}
diff --git a/crates/tc2/src/roadmap/mod.rs b/crates/tc2/src/roadmap/mod.rs
new file mode 100644
index 0000000..a1731bd
--- /dev/null
+++ b/crates/tc2/src/roadmap/mod.rs
@@ -0,0 +1,539 @@
+//! `tc2 roadmap` (F3-4): la vista para todos, en `ROADMAP.md` de la rama de
+//! estado (ADR F1-1, decision 3).
+//!
+//! - [`de_estado`] genera el `ROADMAP.md` con el mismo calculo que `next`
+//!   ([`crate::situacion`]) y la zona manual del que ya hay, byte a byte; si
+//!   esa zona esta rota, devuelve el anterior tal cual (D3: no se bloquea la
+//!   operacion; `next` y `--comparar` lo avisan). Es el generador que recibe
+//!   cada transaccion de estado ([`crate::estado::Contexto::nuevo`]): se
+//!   regenera, nunca se mergea (ADR F1-2, D2.4).
+//! - La orden `roadmap [--comparar | --escribir] [--json]` (D2): sin
+//!   opciones imprime el ROADMAP que toca sobre el estado que conoce el clon
+//!   (solo lee); `--comparar` dice si lo publicado esta al dia (`ok`) o no
+//!   (`rechazada motivo=desfasado|roadmap-roto`), sin escribir; `--escribir`
+//!   lo publica como una transaccion mas ([`Regenerar`]).
+//! - La zona manual se edita en `tareas/ROADMAP.md` y se publica con
+//!   `--escribir` (D4): si el fichero de la carpeta montada tiene cambios, se
+//!   prepara (`git add`) y su zona es la que se publica, byte a byte; tras
+//!   publicar, la carpeta se adelanta a lo publicado.
+
+pub mod generar;
+pub mod mermaid;
+pub mod zona;
+
+use std::path::{Path, PathBuf};
+
+use crate::Salida;
+use crate::contrato::{Informe, Resultado, Valor, corto};
+use crate::estado::{self, CARPETA, Cambio, Catalogo, Estado, Fallo, Operacion, RAMA};
+use crate::git::{Git, Subcomando, correr, exigir, localizar_principal, worktrees};
+use crate::i18n::catalogo::cli as txt_cli;
+use crate::i18n::catalogo::roadmap as txt;
+use crate::i18n::{Idioma, Mensaje};
+use crate::plataforma::Personas;
+use crate::situacion::{self, ROADMAP, Situacion};
+
+/// Nombre de la orden.
+pub const NOMBRE: &str = "roadmap";
+
+/// Lista de personas de la base para convertir los `asignado_a` heredados
+/// (D6); sin lista, ninguna: se muestran tal cual.
+pub fn personas(g: &Git, dir: &Path) -> Personas {
+    crate::identidad::personas_de_base(g, dir).unwrap_or_default()
+}
+
+/// El generador que recibe cada transaccion de estado.
+pub fn generador(personas: &Personas) -> impl Fn(&dyn Estado) -> Result<String, Fallo> + '_ {
+    move |e: &dyn Estado| de_estado(e, personas)
+}
+
+/// El `ROADMAP.md` de `estado`: generado alrededor de su zona manual (o la
+/// de por defecto), o el que hay tal cual si su zona esta rota (D3).
+pub fn de_estado(estado: &dyn Estado, personas: &Personas) -> Result<String, Fallo> {
+    let s = situacion::calcular(estado)?;
+    Ok(de_situacion(&s, personas).unwrap_or_else(|| s.roadmap.clone().unwrap_or_default()))
+}
+
+/// El ROADMAP que toca para `s`, o `None` si su zona manual esta rota.
+fn de_situacion(s: &Situacion, personas: &Personas) -> Option<String> {
+    let zona = match s.roadmap.as_deref().map(zona::extraer) {
+        None | Some(Ok(None)) => zona::POR_DEFECTO,
+        Some(Ok(Some(z))) => z,
+        Some(Err(_)) => return None,
+    };
+    Some(generar::generar(s, zona, personas))
+}
+
+fn roto(donde: &str, motivo: &str) -> Fallo {
+    Fallo::rechazada(
+        "roadmap-roto",
+        Mensaje::nuevo(&txt::ZONA_ROTA)
+            .con("donde", donde)
+            .con("motivo", motivo),
+    )
+}
+
+/// `roadmap --escribir` como operacion de la rama de estado: no cambia nada
+/// salvo, si la persona edito la zona manual en `tareas/`, el `ROADMAP.md`
+/// con esa zona; el generador de la transaccion hace el resto. Exige red
+/// (D2b de F1-2): sin red no queda pendiente.
+#[derive(Debug, Clone, Default)]
+pub struct Regenerar {
+    /// `tareas/ROADMAP.md` preparado por la persona, con su zona valida.
+    editado: Option<String>,
+}
+
+impl Regenerar {
+    /// Sin edicion de la persona.
+    pub fn nueva() -> Self {
+        Self::default()
+    }
+
+    /// Con el `ROADMAP.md` editado por la persona (su zona ya validada).
+    pub fn con_editado(texto: String) -> Self {
+        Self {
+            editado: Some(texto),
+        }
+    }
+}
+
+impl Operacion for Regenerar {
+    fn nombre(&self) -> &str {
+        NOMBRE
+    }
+    fn anotacion(&self) -> String {
+        format!("{NOMBRE}|escribir")
+    }
+    fn exige_red(&self) -> bool {
+        true
+    }
+    fn aplicar(&self, e: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
+        let actual = e.leer(ROADMAP)?;
+        match &self.editado {
+            Some(t) if actual.as_deref() == Some(t.as_str()) => Ok(Vec::new()),
+            Some(t) => Ok(vec![Cambio::nuevo(ROADMAP, t.clone())]),
+            None => match actual.as_deref().map(zona::extraer) {
+                Some(Err(r)) => Err(roto(ROADMAP, r.motivo())),
+                _ => Ok(Vec::new()),
+            },
+        }
+    }
+}
+
+/// Reconstruye [`Regenerar`] desde su anotacion (nunca queda pendiente:
+/// exige red).
+pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
+    (a == format!("{NOMBRE}|escribir")).then(|| Box::new(Regenerar::nueva()) as Box<dyn Operacion>)
+}
+
+/// Que hace la orden.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Modo {
+    /// Imprimir el ROADMAP que toca (solo lee).
+    Imprimir,
+    /// Decir si lo publicado esta al dia (solo lee).
+    Comparar,
+    /// Publicarlo.
+    Escribir,
+}
+
+/// `roadmap [--comparar | --escribir]` (sin `--json`, que quita `lib.rs`)
+/// en la carpeta actual; `catalogo` re-ejecuta las pendientes.
+pub fn orden(args: &[String], json: bool, idioma: Idioma, catalogo: Catalogo<'_>) -> Salida {
+    let modo = match args {
+        [] => Modo::Imprimir,
+        [a] if a == "--comparar" => Modo::Comparar,
+        [a] if a == "--escribir" => Modo::Escribir,
+        _ => {
+            return Informe::nuevo(NOMBRE, Resultado::Uso)
+                .con("motivo", "argumentos")
+                .diagnostico(&txt::USO_ROADMAP)
+                .salida_en(json, idioma);
+        }
+    };
+    let dir = match std::env::current_dir() {
+        Ok(d) => d,
+        Err(e) => {
+            return Informe::nuevo(NOMBRE, Resultado::Error)
+                .diagnostico(Mensaje::nuevo(&txt_cli::CARPETA_ACTUAL).con("error", e.to_string()))
+                .salida_en(json, idioma);
+        }
+    };
+    let g = Git::nuevo();
+    match modo {
+        Modo::Imprimir => match imprimir_en(&g, &dir) {
+            Ok((texto, _)) if !json => Salida {
+                codigo: 0,
+                stdout: texto,
+                stderr: String::new(),
+            },
+            Ok((texto, commit)) => Informe::nuevo(NOMBRE, Resultado::Ok)
+                .con("commit", commit)
+                .con("texto", texto)
+                .salida_en(true, idioma),
+            Err(i) => i.salida_en(json, idioma),
+        },
+        Modo::Comparar => comparar_en(&g, &dir).salida_en(json, idioma),
+        Modo::Escribir => escribir_en(&g, &dir, catalogo).salida_en(json, idioma),
+    }
+}
+
+fn renombrado(mut i: Informe) -> Informe {
+    NOMBRE.clone_into(&mut i.comando);
+    i
+}
+
+fn de_fallo(f: Fallo) -> Informe {
+    match f {
+        Fallo::Rechazada {
+            motivo,
+            diagnostico,
+        } => Informe::nuevo(NOMBRE, Resultado::Rechazada)
+            .con("motivo", motivo)
+            .diagnostico(diagnostico),
+        Fallo::Error(d) => Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d),
+    }
+}
+
+/// La situacion del estado que conoce el clon (sin `fetch`) y su commit.
+fn situacion_en(g: &Git, dir: &Path) -> Result<(Situacion, String), Informe> {
+    let vista = estado::leer(g, dir).map_err(renombrado)?;
+    let commit = vista.commit().map(corto).unwrap_or_default();
+    let s = situacion::calcular(&vista).map_err(de_fallo)?;
+    Ok((s, commit))
+}
+
+/// El ROADMAP que toca sobre el estado que conoce el clon, y el commit de
+/// ese estado. Con la zona rota, `rechazada motivo=roadmap-roto`.
+pub fn imprimir_en(g: &Git, dir: &Path) -> Result<(String, String), Informe> {
+    let (s, commit) = situacion_en(g, dir)?;
+    match de_situacion(&s, &personas(g, dir)) {
+        Some(t) => Ok((t, commit)),
+        None => Err(zona_rota(&s, &commit)),
+    }
+}
+
+fn zona_rota(s: &Situacion, commit: &str) -> Informe {
+    let motivo = s.roadmap_roto().map_or("", zona::Rota::motivo);
+    de_fallo(roto(ROADMAP, motivo))
+        .con("roto", motivo)
+        .con("commit", commit)
+}
+
+/// `--comparar`: `ok` si el `ROADMAP.md` publicado es el que toca;
+/// `rechazada motivo=desfasado` si no; `rechazada motivo=roadmap-roto` si
+/// su zona esta rota (D3). No escribe nada.
+pub fn comparar_en(g: &Git, dir: &Path) -> Informe {
+    let (s, commit) = match situacion_en(g, dir) {
+        Ok(x) => x,
+        Err(i) => return i,
+    };
+    match de_situacion(&s, &personas(g, dir)) {
+        None => zona_rota(&s, &commit),
+        Some(t) if s.roadmap.as_deref() == Some(t.as_str()) => {
+            Informe::nuevo(NOMBRE, Resultado::Ok).con("commit", commit)
+        }
+        Some(_) => Informe::nuevo(NOMBRE, Resultado::Rechazada)
+            .con("motivo", "desfasado")
+            .con("commit", commit)
+            .diagnostico(&txt::DESFASADO),
+    }
+}
+
+/// `tareas/` montada (la carpeta del worktree de la rama de estado).
+fn carpeta_montada(g: &Git, dir: &Path) -> Result<Option<PathBuf>, Informe> {
+    let principal = localizar_principal(g, dir).map_err(renombrado)?;
+    Ok(worktrees(g, &principal, NOMBRE)?
+        .into_iter()
+        .find(|w| w.rama.as_deref() == Some(RAMA) && w.ruta.exists())
+        .map(|w| w.ruta))
+}
+
+/// Si `tareas/ROADMAP.md` tiene cambios, los prepara y devuelve lo
+/// preparado (normalizado como lo guardaria git), con su zona validada.
+fn editado_en(g: &Git, carpeta: &Path) -> Result<Option<String>, Informe> {
+    let s = exigir(
+        g,
+        carpeta,
+        NOMBRE,
+        Subcomando::Status,
+        &["--porcelain", "--", ROADMAP],
+    )?;
+    if s.trim().is_empty() {
+        return Ok(None);
+    }
+    exigir(g, carpeta, NOMBRE, Subcomando::Add, &["--", ROADMAP])?;
+    let texto = crate::git::estado::leer_blob(g, carpeta, NOMBRE, "", ROADMAP)?;
+    let donde = format!("{CARPETA}/{ROADMAP}");
+    match zona::extraer(&texto) {
+        Ok(Some(_)) => Ok(Some(texto)),
+        Ok(None) => Err(de_fallo(roto(&donde, "sin-zona")).con("roto", "sin-zona")),
+        Err(r) => Err(de_fallo(roto(&donde, r.motivo())).con("roto", r.motivo())),
+    }
+}
+
+/// `--escribir`: publica el ROADMAP que toca como una transaccion (D1).
+pub fn escribir_en(g: &Git, dir: &Path, catalogo: Catalogo<'_>) -> Informe {
+    let carpeta = match carpeta_montada(g, dir) {
+        Ok(c) => c,
+        Err(i) => return renombrado(i),
+    };
+    let editado = match carpeta.as_deref().map(|c| editado_en(g, c)).transpose() {
+        Ok(e) => e.flatten(),
+        Err(i) => return renombrado(i),
+    };
+    let op = match &editado {
+        Some(t) => Regenerar::con_editado(t.clone()),
+        None => Regenerar::nueva(),
+    };
+    let personas = personas(g, dir);
+    let generar = generador(&personas);
+    let c = estado::Contexto::nuevo(g, dir, catalogo, Some(&generar));
+    let mut i = renombrado(estado::transaccion(&c, &op));
+    if editado.is_some()
+        && i.resultado == Resultado::Ok
+        && i.valor("tareas") == Some("con-cambios")
+        && let (Some(carpeta), Some(commit)) = (carpeta, i.valor("commit").map(str::to_owned))
+        && adelantar_editada(g, &carpeta, &commit)
+        && let Some(v) = i.claves.iter_mut().find(|(k, _)| k == "tareas")
+    {
+        v.1 = Valor::Texto("al-dia".to_owned());
+    }
+    i
+}
+
+/// Tras publicar la zona editada, deja `tareas/ROADMAP.md` como lo
+/// publicado (que lleva esa zona byte a byte), lo prepara y adelanta la
+/// carpeta: sin esto, el cambio ya publicado la dejaria atascada. Si algo
+/// falla, la carpeta queda como estaba (con `tareas=con-cambios`).
+fn adelantar_editada(g: &Git, carpeta: &Path, commit: &str) -> bool {
+    let Ok(publicado) = crate::git::estado::leer_blob(g, carpeta, NOMBRE, commit, ROADMAP) else {
+        return false;
+    };
+    std::fs::write(carpeta.join(ROADMAP), publicado.as_bytes()).is_ok()
+        && exigir(g, carpeta, NOMBRE, Subcomando::Add, &["--", ROADMAP]).is_ok()
+        && correr(
+            g,
+            carpeta,
+            NOMBRE,
+            Subcomando::Merge,
+            &["-q", "--ff-only", commit],
+        )
+        .is_ok_and(|s| s.exito())
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use std::collections::BTreeMap;
+
+    use crate::tarea::{Asignado, Cuerpo, Tarea};
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
+    fn v2(
+        id: &str,
+        titulo: &str,
+        sprint: u32,
+        estado: &str,
+        deps: &[&str],
+        asignado: Option<&str>,
+    ) -> String {
+        let mut t = Tarea::nueva(id, titulo, "2026-01-01");
+        t.sprint = sprint;
+        t.estado = estado.to_owned();
+        if estado == "descartada" {
+            t.motivo = Some("ya no hace falta".to_owned());
+        }
+        t.dependencias = deps.iter().map(|d| (*d).to_owned()).collect();
+        if let Some(login) = asignado {
+            t.asignado_a = Asignado::persona(7, login);
+        }
+        t.escribir(&Cuerpo::leer(""))
+    }
+
+    /// Formato 1 (0.16), con `asignado_a` de texto libre.
+    fn v1(id: &str, titulo: &str, sprint: u32, estado: &str, asignado: &str) -> String {
+        format!(
+            "---\nid: {id}\ntitulo: \"{titulo}\"\ntipo: feature\nsprint: {sprint}\netiquetas: []\n\
+             complejidad: null\nmodelo_sugerido: sonnet\nestado: {estado}\nplan_aprobado: false\n\
+             rama: feature/x\nasignado_a: {asignado}\nagente_revisor: general-purpose\n\
+             skills_recomendados: []\nregla_seleccion_skill: null\nultimo_commit_revisado: null\n\
+             revision_codex: false\ntokens_diseno: null\ntokens_implementacion: null\n\
+             tokens_revision: null\ncreado: 2026-01-01\nactualizado: 2026-01-01\n\
+             dependencias: []\n---\n"
+        )
+    }
+
+    const ZONA: &str = "<!-- manual:inicio (a mano) -->\r\n## Hitos\r\n\r\n### H2 — Dos\r\nEl motivo.\r\n### H1 — Uno\r\n### H3 — Tres\r\n<!-- manual:fin -->\r\n";
+
+    /// Un estado que pasa por cada seccion.
+    fn estado_de_referencia() -> EnMemoria {
+        let mut m = BTreeMap::new();
+        for (id, md) in [
+            (
+                "TASK-001",
+                v2("TASK-001", "Base", 1, "terminada", &[], None),
+            ),
+            (
+                "TASK-002",
+                v2(
+                    "TASK-002",
+                    "Motor",
+                    1,
+                    "en-curso",
+                    &["TASK-001"],
+                    Some("ana"),
+                ),
+            ),
+            (
+                "TASK-003",
+                v1("TASK-003", "Revisar", 1, "en-revision", "ana@x.com"),
+            ),
+            (
+                "TASK-004",
+                v2(
+                    "TASK-004",
+                    "Usa el motor",
+                    2,
+                    "planificada",
+                    &["TASK-002"],
+                    None,
+                ),
+            ),
+            (
+                "TASK-005",
+                v2("TASK-005", "Lista", 2, "planificada", &["TASK-001"], None),
+            ),
+            (
+                "TASK-006",
+                v2("TASK-006", "Ya no", 2, "descartada", &[], None),
+            ),
+            (
+                "TASK-007",
+                v2(
+                    "TASK-007",
+                    "Tras la descartada",
+                    2,
+                    "planificada",
+                    &["TASK-006", "TASK-099"],
+                    None,
+                ),
+            ),
+            (
+                "TASK-008",
+                v1(
+                    "TASK-008",
+                    "Con a | b <!-- y \\\"comillas\\\" #1",
+                    2,
+                    "en-diseno",
+                    "alguien de fuera",
+                ),
+            ),
+            (
+                "TASK-009",
+                v2("TASK-009", "Cerrada", 3, "terminada", &[], None),
+            ),
+            ("TASK-010", "---\nid: roto\n".to_owned()),
+            (
+                "TASK-011",
+                v2("TASK-011", "Ciclo a", 2, "planificada", &["TASK-012"], None),
+            ),
+            (
+                "TASK-012",
+                v2(
+                    "TASK-012",
+                    "Ciclo b",
+                    2,
+                    "planificada",
+                    &["TASK-011", "TASK-010"],
+                    None,
+                ),
+            ),
+        ] {
+            m.insert(format!("{id}/tarea.md"), md);
+        }
+        m.insert(
+            situacion::ROADMAP.to_owned(),
+            format!("# Viejo\n\n{ZONA}\n## Lo generado antes\n"),
+        );
+        EnMemoria(m)
+    }
+
+    fn personas() -> Personas {
+        Personas::leer("personas:\n  - {id: 9, login: \"ana-x\", correos: [\"ana@x.com\"]}\n")
+            .unwrap_or_default()
+    }
+
+    const REFERENCIA: &str = include_str!("../../tests/fixtures/roadmap/referencia.md");
+
+    /// C5: la salida de referencia cubre cada seccion (con quien, revision,
+    /// hito con dependencias y Mermaid, terminadas y descartadas); C2: la
+    /// zona manual, con CRLF, byte a byte. Para rehacer la referencia:
+    /// `TC2_REESCRIBIR=1 cargo test -p tc2 --lib referencia`.
+    #[test]
+    fn salida_de_referencia() {
+        let r = de_estado(&estado_de_referencia(), &personas());
+        assert!(r.is_ok(), "{r:?}");
+        let Ok(r) = r else { return };
+        if std::env::var_os("TC2_REESCRIBIR").is_some() {
+            let ruta = concat!(
+                env!("CARGO_MANIFEST_DIR"),
+                "/tests/fixtures/roadmap/referencia.md"
+            );
+            assert!(std::fs::write(ruta, &r).is_ok());
+            return;
+        }
+        assert_eq!(r, REFERENCIA);
+        assert_eq!(zona::extraer(&r), Ok(Some(ZONA)));
+        for seccion in [
+            "\n## Estado del trabajo\n",
+            "\n## En curso\n",
+            "\n## En revisión\n",
+            "\n## H2 — Dos\n",
+            "\n## H1 — Uno\n",
+            "\n## Terminadas\n",
+            "\n## Descartadas\n",
+            "```mermaid\n",
+            "| @ana |",
+            "| @ana-x |",
+            "<details>",
+        ] {
+            assert!(r.contains(seccion), "falta {seccion:?}");
+        }
+        assert!(!r.contains("\n## H3"), "un hito cerrado no lleva apartado");
+        // Mismos datos, mismos bytes (lo que da `sin-cambios`).
+        let mut otra = estado_de_referencia();
+        otra.0.insert(situacion::ROADMAP.to_owned(), r.clone());
+        assert_eq!(de_estado(&otra, &personas()).ok(), Some(r));
+    }
+
+    /// D3: con la zona rota se devuelve el anterior tal cual.
+    #[test]
+    fn zona_rota_deja_el_anterior() {
+        let mut e = estado_de_referencia();
+        let roto = "# Roadmap\n<!-- manual:inicio -->\n### H1 — Uno\n".to_owned();
+        e.0.insert(situacion::ROADMAP.to_owned(), roto.clone());
+        assert_eq!(de_estado(&e, &Personas::default()).ok(), Some(roto));
+    }
+
+    /// Sin ROADMAP.md (o sin zona), la zona de por defecto.
+    #[test]
+    fn sin_roadmap_la_zona_por_defecto() {
+        let mut e = estado_de_referencia();
+        e.0.remove(situacion::ROADMAP);
+        let r = de_estado(&e, &Personas::default()).unwrap_or_default();
+        assert_eq!(zona::extraer(&r), Ok(Some(zona::POR_DEFECTO)));
+        assert!(r.starts_with(&format!("# Roadmap de tareas\n\n{}\n\n", generar::MARCA)));
+    }
+}
diff --git a/crates/tc2/src/roadmap/zona.rs b/crates/tc2/src/roadmap/zona.rs
new file mode 100644
index 0000000..cf3e901
--- /dev/null
+++ b/crates/tc2/src/roadmap/zona.rs
@@ -0,0 +1,163 @@
+//! Zona manual de `ROADMAP.md` (`<!-- manual:inicio ... -->` hasta
+//! `<!-- manual:fin -->`, la de la 0.16; ADR F1-1, decision 3).
+//!
+//! [`extraer`] es estricta: una marca de inicio sin fin, una de fin suelta o
+//! una segunda zona son un ROADMAP roto, y se dice en vez de adivinar donde
+//! acaba (adivinar mal perderia texto de la persona). La zona se devuelve
+//! como un trozo del texto, byte a byte, con sus saltos de linea (LF o CRLF).
+//! [`interior_tolerante`] es la lectura que usa el orden de los hitos: no
+//! falla, para que `next` siga dando su orden con un ROADMAP roto.
+//!
+//! Hoja pura.
+
+/// Marca de inicio (la linea empieza asi, tras espacios).
+pub const INICIO: &str = "<!-- manual:inicio";
+/// Marca de fin.
+pub const FIN: &str = "<!-- manual:fin";
+
+/// Zona de un ROADMAP recien creado: la de la 0.16, con `tc2` en lugar de
+/// `taskctl`.
+pub const POR_DEFECTO: &str = "<!-- manual:inicio (se edita a mano; tc2 la conserva) -->\n\
+## Hitos\n\
+\n\
+Un `### Hn — título` por hito (n es el campo `sprint` de sus tareas), en el orden en que se hacen y con su motivo debajo. tc2 conserva esta zona tal cual y toma de aquí los títulos y el orden de los hitos.\n\
+<!-- manual:fin -->\n";
+
+/// Por que la zona manual esta rota.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Rota {
+    /// Abre la zona y no la cierra.
+    SinFin,
+    /// Cierra una zona que no se abrio.
+    FinSuelto,
+    /// Hay mas de una zona (o un inicio dentro de otra).
+    Repetida,
+}
+
+impl Rota {
+    /// Valor de `roadmap-roto=` y de `motivo=`.
+    pub fn motivo(self) -> &'static str {
+        match self {
+            Self::SinFin => "sin-fin",
+            Self::FinSuelto => "fin-suelto",
+            Self::Repetida => "repetida",
+        }
+    }
+}
+
+fn es_marca(linea: &str, marca: &str) -> bool {
+    linea.trim_start().starts_with(marca)
+}
+
+/// La zona manual de `texto`, desde el principio de la linea de inicio hasta
+/// el final de la de fin (con su salto de linea, si lo tiene), tal cual.
+/// Sin zona, `None`.
+pub fn extraer(texto: &str) -> Result<Option<&str>, Rota> {
+    let mut inicio: Option<usize> = None;
+    let mut fin: Option<usize> = None;
+    let mut pos = 0;
+    for linea in texto.split_inclusive('\n') {
+        if es_marca(linea, INICIO) {
+            if inicio.is_some() {
+                return Err(Rota::Repetida);
+            }
+            inicio = Some(pos);
+        } else if es_marca(linea, FIN) {
+            match (inicio, fin) {
+                (None, _) => return Err(Rota::FinSuelto),
+                (Some(_), Some(_)) => return Err(Rota::Repetida),
+                (Some(_), None) => fin = Some(pos + linea.len()),
+            }
+        }
+        pos += linea.len();
+    }
+    match (inicio, fin) {
+        (None, _) => Ok(None),
+        (Some(_), None) => Err(Rota::SinFin),
+        (Some(i), Some(f)) => Ok(texto.get(i..f)),
+    }
+}
+
+/// Las lineas de dentro de la zona (sin las marcas), con LF. Tolerante: un
+/// inicio sin fin llega hasta el final y una segunda marca de inicio se
+/// salta; sin inicio, nada.
+pub fn interior_tolerante(texto: &str) -> String {
+    let mut dentro = false;
+    let mut zona = String::new();
+    for l in texto.lines() {
+        if es_marca(l, INICIO) {
+            dentro = true;
+        } else if es_marca(l, FIN) {
+            break;
+        } else if dentro {
+            zona.push_str(l);
+            zona.push('\n');
+        }
+    }
+    zona
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    /// C2: la zona sale byte a byte, con LF y con CRLF, y con texto delante
+    /// y detras.
+    #[test]
+    fn extrae_byte_a_byte_con_lf_y_crlf() {
+        for salto in ["\n", "\r\n"] {
+            let zona = format!(
+                "<!-- manual:inicio (a mano) -->{salto}## Hitos{salto}{salto}### H1 — Uno  {salto}\t texto\u{a0}raro{salto}<!-- manual:fin -->{salto}"
+            );
+            let texto = format!("# Roadmap{salto}{salto}{zona}{salto}## Generado{salto}");
+            assert_eq!(extraer(&texto), Ok(Some(zona.as_str())));
+        }
+        // Sin salto tras la marca de fin.
+        let t = "x\n<!-- manual:inicio -->\ny\n<!-- manual:fin -->";
+        assert_eq!(
+            extraer(t),
+            Ok(Some("<!-- manual:inicio -->\ny\n<!-- manual:fin -->"))
+        );
+        assert_eq!(extraer("# Sin zona\n"), Ok(None));
+        assert_eq!(extraer(""), Ok(None));
+    }
+
+    /// D3: una zona rota se dice, no se adivina.
+    #[test]
+    fn zona_rota() {
+        assert_eq!(extraer("<!-- manual:inicio -->\nx\n"), Err(Rota::SinFin));
+        assert_eq!(extraer("x\n<!-- manual:fin -->\n"), Err(Rota::FinSuelto));
+        assert_eq!(
+            extraer("<!-- manual:inicio -->\n<!-- manual:inicio -->\n<!-- manual:fin -->\n"),
+            Err(Rota::Repetida)
+        );
+        assert_eq!(
+            extraer("<!-- manual:inicio -->\n<!-- manual:fin -->\n<!-- manual:fin -->\n"),
+            Err(Rota::Repetida)
+        );
+        assert_eq!(
+            extraer(
+                "<!-- manual:inicio -->\n<!-- manual:fin -->\n<!-- manual:inicio -->\n<!-- manual:fin -->\n"
+            ),
+            Err(Rota::Repetida)
+        );
+        assert_eq!(Rota::SinFin.motivo(), "sin-fin");
+    }
+
+    #[test]
+    fn la_zona_por_defecto_es_valida() {
+        assert_eq!(extraer(POR_DEFECTO), Ok(Some(POR_DEFECTO)));
+    }
+
+    #[test]
+    fn interior_tolerante_como_antes() {
+        let r = "### H9 — Fuera\n<!-- manual:inicio x -->\n### H2 — Dos\r\n<!-- manual:fin -->\n### H8\n";
+        assert_eq!(interior_tolerante(r), "### H2 — Dos\n");
+        assert_eq!(
+            interior_tolerante("<!-- manual:inicio -->\na\n"),
+            "a\n",
+            "sin fin llega al final"
+        );
+        assert_eq!(interior_tolerante("a\n<!-- manual:fin -->\n"), "");
+    }
+}
diff --git a/crates/tc2/src/siguiente.rs b/crates/tc2/src/siguiente.rs
index 74802f4..06fc87a 100644
--- a/crates/tc2/src/siguiente.rs
+++ b/crates/tc2/src/siguiente.rs
@@ -2,34 +2,32 @@
 //! prioridad.
 //!
 //! Lee el estado tal como lo conoce este clon ([`estado::leer`]: sin
-//! `fetch` y sin escribir; la lectura fresca es de F3-5b), proyecta cada
-//! `tarea.md` a un [`Nodo`] y calcula el orden con [`grafo`]. El orden de los
-//! hitos sale de los titulos `### Hn — titulo` de la zona manual del
+//! `fetch` y sin escribir; la lectura fresca es de F3-5b) y calcula con
+//! [`crate::situacion`], lo mismo que `roadmap` (TASK-031 C4). El orden de
+//! los hitos sale de los titulos `### Hn — titulo` de la zona manual del
 //! `ROADMAP.md` de la rama de estado (D2 de TASK-030).
 //!
 //! Salida: `next ok listas=... en-marcha=... commit=<sha> [avisos]`, con
 //! `commit=` la foto del estado usada. Avisos, solo si los hay:
 //! `ilegibles=` (`tarea.md` que no se entienden: no se saltan en silencio),
 //! `ciclos=` (las atascadas), `dependencias-inexistentes=`,
-//! `depende-de-descartada=` (`A>B`), `hitos-sin-titulo=` y `duplicadas=`
-//! (IDs repetidos: vale el primero). Sin rama de estado, `rechazada`.
+//! `depende-de-descartada=` (`A>B`), `hitos-sin-titulo=`, `duplicadas=`
+//! (IDs repetidos: vale el primero) y `roadmap-roto=` (la zona manual del
+//! ROADMAP, rota: D3 de TASK-031). Sin rama de estado, `rechazada`.
 
 use std::path::Path;
 
 use crate::contrato::{Informe, Resultado, corto};
 use crate::estado::{self, Estado, Fallo};
 use crate::git::Git;
-use crate::grafo::{self, Hitos, Nodo};
 use crate::i18n::Mensaje;
 use crate::i18n::catalogo::cli as txt_cli;
 use crate::i18n::catalogo::siguiente as txt;
+use crate::situacion::{self, Situacion};
 
 /// Nombre de la orden.
 pub const NOMBRE: &str = "next";
 
-/// `ROADMAP.md` en la raiz de la rama de estado (ADR F1-1, decision 3).
-const ROADMAP: &str = "ROADMAP.md";
-
 /// `next [--json]` en la carpeta actual (sin `--json`, que quita `lib.rs`).
 pub fn orden(args: &[String]) -> Informe {
     if !args.is_empty() {
@@ -66,26 +64,23 @@ pub fn siguiente_en(g: &Git, dir: &Path) -> Informe {
     }
 }
 
-/// Lo que se calcula sobre el estado.
+/// Lo que se calcula sobre el estado: la [`Situacion`] comun con `roadmap`.
 #[derive(Debug, Default)]
-struct Calculo {
-    listas: Vec<String>,
-    en_marcha: Vec<String>,
-    ilegibles: Vec<String>,
-    orden: grafo::Orden,
-}
+struct Calculo(Situacion);
 
 impl Calculo {
     fn informe(self, commit: &str) -> Informe {
-        let a = self.orden.avisos;
+        let roto = self.0.roadmap_roto();
+        let s = self.0;
+        let a = s.orden.avisos;
         let mut i = Informe::nuevo(NOMBRE, Resultado::Ok)
-            .con_lista("listas", self.listas)
-            .con_lista("en-marcha", self.en_marcha)
+            .con_lista("listas", s.listas)
+            .con_lista("en-marcha", s.en_marcha)
             .con("commit", commit);
         let hitos: Vec<String> = a.hitos_sin_titulo.iter().map(|h| format!("H{h}")).collect();
         for (clave, v) in [
-            ("ilegibles", self.ilegibles),
-            ("ciclos", self.orden.atascadas),
+            ("ilegibles", s.ilegibles),
+            ("ciclos", s.orden.atascadas),
             ("dependencias-inexistentes", a.dependencias_inexistentes),
             ("depende-de-descartada", a.depende_de_descartada),
             ("hitos-sin-titulo", hitos),
@@ -95,35 +90,17 @@ impl Calculo {
                 i = i.con_lista(clave, v);
             }
         }
+        // D3 de TASK-031: la zona manual rota se avisa.
+        if let Some(r) = roto {
+            i = i.con("roadmap-roto", r.motivo());
+        }
         i
     }
 }
 
 /// Listas, en marcha e ilegibles de `estado`.
 fn calcular(estado: &dyn Estado) -> Result<Calculo, Fallo> {
-    let rutas = estado.rutas();
-    let mut nodos = Vec::new();
-    let mut ilegibles = Vec::new();
-    for (ruta, leida) in estado::tareas(estado, &rutas)? {
-        match leida {
-            Ok(d) => nodos.push(Nodo::de(&d)),
-            Err(_) => ilegibles.push(ruta.split('/').next().unwrap_or(&ruta).to_owned()),
-        }
-    }
-    // Detras de las legibles: si un ID se repite, vale la legible. Una
-    // ilegible bloquea a quien dependa de ella.
-    nodos.extend(ilegibles.iter().map(|id| Nodo::ilegible(id)));
-    let hitos = match estado.leer(ROADMAP)? {
-        Some(t) => Hitos::del_roadmap(&t),
-        None => Hitos::default(),
-    };
-    let orden = grafo::orden_de_trabajo(&nodos, &hitos);
-    Ok(Calculo {
-        listas: grafo::listas(&orden, &nodos),
-        en_marcha: grafo::en_marcha(&orden, &nodos),
-        ilegibles,
-        orden,
-    })
+    situacion::calcular(estado).map(Calculo)
 }
 
 #[cfg(test)]
@@ -188,15 +165,27 @@ mod tests {
             tarea("TASK-003", 3, "en-curso", &[]),
         );
         m.insert(
-            ROADMAP.into(),
+            situacion::ROADMAP.into(),
             "<!-- manual:inicio -->\n### H2 — Dos\n### H1 — Uno\n<!-- manual:fin -->\n".into(),
         );
-        let c = calcular(&EnMemoria(m));
+        let c = calcular(&EnMemoria(m.clone()));
         let Ok(c) = c else { return };
         assert_eq!(
             c.informe("0123456789ab").linea(),
             "next ok listas=TASK-002,TASK-001 en-marcha=TASK-003 commit=0123456789ab hitos-sin-titulo=H3"
         );
+        // D3 de TASK-031: sin la marca de fin, el orden sigue (lectura
+        // tolerante) y se avisa.
+        m.insert(
+            situacion::ROADMAP.into(),
+            "<!-- manual:inicio -->\n### H2 — Dos\n### H1 — Uno\n".into(),
+        );
+        let c = calcular(&EnMemoria(m));
+        let Ok(c) = c else { return };
+        assert_eq!(
+            c.informe("0123456789ab").linea(),
+            "next ok listas=TASK-002,TASK-001 en-marcha=TASK-003 commit=0123456789ab hitos-sin-titulo=H3 roadmap-roto=sin-fin"
+        );
     }
 
     #[test]
diff --git a/crates/tc2/src/situacion.rs b/crates/tc2/src/situacion.rs
new file mode 100644
index 0000000..23b65c9
--- /dev/null
+++ b/crates/tc2/src/situacion.rs
@@ -0,0 +1,109 @@
+//! Situacion del trabajo sobre un estado (F3-3 y F3-4): las tareas leidas,
+//! el orden de trabajo, las listas para empezar y las en marcha. La usan
+//! `next` y `roadmap`, que coinciden por construccion (TASK-031 C4).
+//!
+//! Lee todos los `tarea.md` y el `ROADMAP.md` de la rama de estado con un
+//! solo [`Estado::leer_varios`] (un `git cat-file --batch` sobre una
+//! [`crate::estado::Vista`]) y calcula el orden con [`grafo`].
+
+use crate::estado::{Estado, Fallo, es_tarea_md};
+use crate::grafo::{self, Hitos, Nodo};
+use crate::roadmap::zona::{self, Rota};
+use crate::tarea::{Asignado, Documento};
+
+pub use crate::estado::ROADMAP;
+
+/// Lo que se muestra de una tarea legible.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Ficha {
+    /// Lo que cuenta para el orden.
+    pub nodo: Nodo,
+    /// Titulo.
+    pub titulo: String,
+    /// Quien la lleva (en formato 1, siempre texto heredado).
+    pub asignado: Option<Asignado>,
+}
+
+impl Ficha {
+    /// Ficha de un `tarea.md` leido.
+    pub fn de(doc: &Documento) -> Self {
+        let (titulo, asignado) = match doc {
+            Documento::Formato1(t, _) => (t.titulo.clone(), t.asignado_a.clone()),
+            Documento::Formato2(t, _) => (t.titulo.clone(), t.asignado_a.clone()),
+        };
+        Self {
+            nodo: Nodo::de(doc),
+            titulo,
+            asignado,
+        }
+    }
+}
+
+/// Todo lo que se calcula sobre un estado.
+#[derive(Debug, Clone, Default)]
+pub struct Situacion {
+    /// Las tareas legibles, en orden de ruta.
+    pub fichas: Vec<Ficha>,
+    /// Los nodos del orden: los de las fichas y, detras, los ilegibles.
+    pub nodos: Vec<Nodo>,
+    /// Carpetas de los `tarea.md` que no se entienden.
+    pub ilegibles: Vec<String>,
+    /// Orden de los hitos (zona manual del ROADMAP, lectura tolerante).
+    pub hitos: Hitos,
+    /// Orden de trabajo, atascadas y avisos.
+    pub orden: grafo::Orden,
+    /// Listas para empezar, en el orden de trabajo.
+    pub listas: Vec<String>,
+    /// En diseno, en curso o en revision, en el orden de trabajo.
+    pub en_marcha: Vec<String>,
+    /// El `ROADMAP.md` de la rama de estado, si lo hay.
+    pub roadmap: Option<String>,
+}
+
+impl Situacion {
+    /// Si la zona manual del ROADMAP esta rota (D3 de TASK-031).
+    pub fn roadmap_roto(&self) -> Option<Rota> {
+        self.roadmap.as_deref().and_then(|t| zona::extraer(t).err())
+    }
+}
+
+/// La situacion de `estado`.
+pub fn calcular(estado: &dyn Estado) -> Result<Situacion, Fallo> {
+    let mut rutas: Vec<String> = estado
+        .rutas()
+        .into_iter()
+        .filter(|r| es_tarea_md(r))
+        .collect();
+    rutas.push(ROADMAP.to_owned());
+    let mut textos = estado.leer_varios(&rutas)?;
+    let roadmap = textos.pop().flatten();
+    rutas.pop();
+    let mut fichas = Vec::new();
+    let mut ilegibles = Vec::new();
+    for (ruta, texto) in rutas.iter().zip(textos) {
+        let Some(texto) = texto else { continue };
+        match crate::tarea::leer(&texto) {
+            Ok(d) => fichas.push(Ficha::de(&d)),
+            Err(_) => ilegibles.push(ruta.split('/').next().unwrap_or(ruta).to_owned()),
+        }
+    }
+    let mut nodos: Vec<Nodo> = fichas.iter().map(|f| f.nodo.clone()).collect();
+    // Detras de las legibles: si un ID se repite, vale la legible. Una
+    // ilegible bloquea a quien dependa de ella.
+    nodos.extend(ilegibles.iter().map(|id| Nodo::ilegible(id)));
+    let hitos = roadmap
+        .as_deref()
+        .map(Hitos::del_roadmap)
+        .unwrap_or_default();
+    let orden = grafo::orden_de_trabajo(&nodos, &hitos);
+    Ok(Situacion {
+        listas: grafo::listas(&orden, &nodos),
+        en_marcha: grafo::en_marcha(&orden, &nodos),
+        fichas,
+        nodos,
+        ilegibles,
+        hitos,
+        orden,
+        roadmap,
+    })
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
tareas/01-en-diseno/TASK-031/tarea.md                                                     | 44 --------------------------------------------
 tareas/{01-en-diseno => 02-en-curso}/TASK-031/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-031/planificacion/plan-final.md                 |  0
 tareas/02-en-curso/TASK-031/tarea.md                                                      | 86 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 4 files changed, 86 insertions(+), 44 deletions(-)
````

## Tests del diff (6 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/alta.rs" ":(literal)crates/tc2/tests/convivencia_016.rs" ":(literal)crates/tc2/tests/estado_bordes.rs" ":(literal)crates/tc2/tests/estado_casos.rs" ":(literal)crates/tc2/tests/estado_concurrencia.rs" ":(literal)crates/tc2/tests/roadmap.rs"
````

````
crates/tc2/tests/alta.rs                |  18 +++++-
 crates/tc2/tests/convivencia_016.rs     |   2 +-
 crates/tc2/tests/estado_bordes.rs       |   6 +-
 crates/tc2/tests/estado_casos.rs        |   2 +-
 crates/tc2/tests/estado_concurrencia.rs |  58 +++++++++++++++---
 crates/tc2/tests/roadmap.rs             | 513 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 6 files changed, 585 insertions(+), 14 deletions(-)
````
