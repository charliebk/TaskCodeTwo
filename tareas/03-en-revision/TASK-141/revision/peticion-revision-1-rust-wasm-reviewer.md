# Peticion de revision — TASK-141 (ronda 1)

- Tarea: TASK-141 — Auditoria 4D: A3 plan, importar y alta (5 IMPORTANTE)
- Rama revisada: fix/task-141-auditoria-4d-a3-plan-importar-y-alta-5-i
- Rama base: develop
- Commit revisado (HEAD): 65a9b47af269e1b7ec3bd6ee34daace322bf0d0d
- Fecha: 2026-11-24
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-141 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-141/tarea.md
- tareas/03-en-revision/TASK-141/planificacion/
- crates/tc2/src/alta.rs
- crates/tc2/src/i18n/catalogo/importar.rs
- crates/tc2/src/importar/fichero.rs
- crates/tc2/src/importar/mod.rs
- crates/tc2/src/lectura.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/plan/aprobar.rs
- crates/tc2/src/plan/guardar.rs
- crates/tc2/src/plan/mod.rs
- crates/tc2/tests/alta.rs
- crates/tc2/tests/estimacion.rs
- crates/tc2/tests/importar.rs
- crates/tc2/tests/progreso.rs
- crates/tc2/tests/toma.rs

## Commits a revisar (git log develop..HEAD)

````
65a9b47 docs(TASK-141): resultado, criterios y autorrevision
ab11d28 fix(TASK-141): A3-02 test de es_esqueleto con los roles de una ronda
324b2f8 fix(TASK-141): A3-01 anotacion_ida_y_vuelta compara la operacion reconstruida
345ccd6 fix(TASK-141): A3-08 new e import aplican la regla de listo de plan
cacc96b fix(TASK-141): A3-05 approve y approve --hito comparten persona_y_config
1fcc11b fix(TASK-141): A3-03 una sola lectura acotada para plan e import
88a78a3 chore(TASK-141): tarea en curso
````

## Diff de tu dominio (14 fichero(s) de 14; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/alta.rs b/crates/tc2/src/alta.rs
index 34288ee..60299c2 100644
--- a/crates/tc2/src/alta.rs
+++ b/crates/tc2/src/alta.rs
@@ -104,7 +104,19 @@ impl Entrada {
                 return Err(NoListo::Repetido);
             }
         }
-        Ok(())
+        // Y la misma regla que `plan` (A3-08): lo que `plan` rechazaria no
+        // se da de alta.
+        let enunciado = crate::plan::heuristica::Enunciado {
+            objetivo: objetivo.to_owned(),
+            criterios: self.criterios.clone(),
+        };
+        match enunciado.validar() {
+            Ok(()) => Ok(()),
+            Err(("objetivo-vacio", _)) => Err(NoListo::SinObjetivo),
+            Err(("demasiados-criterios", _)) => Err(NoListo::DemasiadosCriterios),
+            Err(("sin-criterios", _)) => Err(NoListo::SinCriterios),
+            Err(_) => Err(NoListo::CriterioVacio),
+        }
     }
 }
 
@@ -239,6 +251,10 @@ pub enum NoListo {
     Repetido,
     /// Caracteres de control.
     Control,
+    /// Hay criterios pero no objetivo (la regla de `plan`).
+    SinObjetivo,
+    /// Mas criterios de los que admite `plan`.
+    DemasiadosCriterios,
 }
 
 impl NoListo {
@@ -250,6 +266,8 @@ impl NoListo {
             Self::Plantilla => "plantilla",
             Self::Repetido => "criterio-repetido",
             Self::Control => "caracter-de-control",
+            Self::SinObjetivo => "objetivo-vacio",
+            Self::DemasiadosCriterios => "demasiados-criterios",
         }
     }
 
@@ -262,6 +280,11 @@ impl NoListo {
             Self::Plantilla => &t::CAUSA_PLANTILLA,
             Self::Repetido => &t::CAUSA_REPETIDO,
             Self::Control => &t::CAUSA_CONTROL,
+            Self::SinObjetivo => &t::CAUSA_SIN_OBJETIVO,
+            Self::DemasiadosCriterios => {
+                return Mensaje::nuevo(&t::CAUSA_DEMASIADOS_CRITERIOS)
+                    .con("max", crate::plan::heuristica::MAX_CRITERIOS.to_string());
+            }
         })
     }
 }
@@ -565,9 +588,22 @@ pub fn aviso_parecidas(p: &[Parecida], idioma: Idioma) -> Option<String> {
 }
 
 /// Reconstruye un [`Alta`] desde su anotacion (para el catalogo).
+///
+/// La anotacion solo lleva el titulo, y la reconstruida es un alta con ese
+/// titulo y lo demas de la plantilla (tipo, objetivo, criterios,
+/// dependencias y lote se pierden). No hace falta mas: un `new` exige red
+/// ([`Operacion::exige_red`], F1-2 D2b), asi que sin red falla y nunca
+/// queda pendiente de reintento; el catalogo no llega a reconstruir uno con
+/// datos. Si `new` dejara de exigir red, la anotacion tendria que llevar la
+/// entrada entera (lo vigila `anotacion_ida_y_vuelta`).
 pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
+    alta_de_anotacion(a).map(|x| Box::new(x) as Box<dyn Operacion>)
+}
+
+/// La mitad sin `Box` de [`desde_anotacion`].
+fn alta_de_anotacion(a: &str) -> Option<Alta> {
     let titulo = a.strip_prefix(NOMBRE)?.strip_prefix('|')?;
-    titulo_valido(titulo).then(|| Box::new(Alta::nueva(titulo)) as Box<dyn Operacion>)
+    titulo_valido(titulo).then(|| Alta::nueva(titulo))
 }
 
 pub use crate::tarea::titulo_valido;
@@ -1052,7 +1088,20 @@ mod tests {
     fn anotacion_ida_y_vuelta() {
         let a = Alta::con_fecha("con | barra", "2026-11-15");
         assert_eq!(a.anotacion(), "new|con | barra");
-        assert!(desde_anotacion(&a.anotacion()).is_some());
+        // La reconstruida es la misma operacion (como en `ciclo`).
+        let r = desde_anotacion(&a.anotacion());
+        assert_eq!(
+            r.as_ref()
+                .map(|x| (x.nombre().to_owned(), x.anotacion(), x.exige_red())),
+            Some((a.nombre().to_owned(), a.anotacion(), a.exige_red()))
+        );
+        // Solo el titulo: basta porque un `new` exige red y nunca queda
+        // pendiente (F1-2 D2b). Si dejara de exigirla, esto sale en rojo.
+        assert!(a.exige_red());
+        assert_eq!(
+            alta_de_anotacion(&a.anotacion()).map(|x| x.entrada),
+            Some(Entrada::nueva("con | barra"))
+        );
         assert!(desde_anotacion("new|").is_none());
         assert!(desde_anotacion("otra|x").is_none());
         assert!(desde_anotacion("newx|t").is_none());
diff --git a/crates/tc2/src/i18n/catalogo/importar.rs b/crates/tc2/src/i18n/catalogo/importar.rs
index 6b7e755..43e0004 100644
--- a/crates/tc2/src/i18n/catalogo/importar.rs
+++ b/crates/tc2/src/i18n/catalogo/importar.rs
@@ -13,6 +13,10 @@ crate::i18n::textos! {
         es: "{ruta} no es texto UTF-8. No se ha importado nada",
         en: "{ruta} is not UTF-8 text. Nothing was imported",
     }
+    VACIO {
+        es: "{ruta} esta vacio. No se ha importado nada",
+        en: "{ruta} is empty. Nothing was imported",
+    }
     FICHERO_GRANDE {
         es: "el fichero pasa de {limite} bytes: partelo en varios lotes. No se ha importado nada",
         en: "the file is larger than {limite} bytes: split it into several batches. Nothing was imported",
@@ -77,6 +81,14 @@ crate::i18n::textos! {
         es: "lleva caracteres de control",
         en: "it contains control characters",
     }
+    CAUSA_SIN_OBJETIVO {
+        es: "tiene criterios pero no objetivo",
+        en: "it has criteria but no goal",
+    }
+    CAUSA_DEMASIADOS_CRITERIOS {
+        es: "tiene mas de {max} criterios de aceptacion: partela en varias tareas",
+        en: "it has more than {max} acceptance criteria: split it into several tasks",
+    }
     CLAVE_DUPLICADA {
         es: "linea {linea}: la clave «{clave}» ya la usa otra entrada; empieza el «###» por una clave distinta (p. ej. «### F3-2 ...»). No se ha importado nada",
         en: "line {linea}: the key «{clave}» is already used by another entry; start the «###» with a different key (e.g. «### F3-2 ...»). Nothing was imported",
diff --git a/crates/tc2/src/importar/fichero.rs b/crates/tc2/src/importar/fichero.rs
index 15eb179..41bebf4 100644
--- a/crates/tc2/src/importar/fichero.rs
+++ b/crates/tc2/src/importar/fichero.rs
@@ -29,8 +29,9 @@ use crate::i18n::Mensaje;
 use crate::i18n::catalogo::importar as txt;
 use crate::tarea::{id_valido, titulo_valido};
 
-/// Tamano maximo del fichero (D4 de TASK-029).
-pub const MAX_BYTES: usize = 1 << 20;
+/// Tamano maximo del fichero (D4 de TASK-029): el tope unico de la lectura
+/// acotada (modulo `lectura`), el mismo de `plan guardar`.
+pub use crate::lectura::MAX_BYTES;
 /// Entradas maximas por lote (D4 de TASK-029).
 pub const MAX_ENTRADAS: usize = 200;
 /// Longitud maxima de una clave de entrada o de un nombre de lote.
@@ -520,7 +521,7 @@ mod tests {
     #[test]
     fn formato_016_sin_cabecera() {
         let l = ok(
-            "# Plan\n\nprosa ignorada\n\n### F1-19 ADR memoria\n> Objetivo uno\n>\n> dos\n- Criterio a\n  sigue a\n* Criterio b\n\n### Sin clave aqui\n- c\n## Otra cosa\ntexto suelto tras cerrar\n",
+            "# Plan\n\nprosa ignorada\n\n### F1-19 ADR memoria\n> Objetivo uno\n>\n> dos\n- Criterio a\n  sigue a\n* Criterio b\n\n### Sin clave aqui\n> o\n- c\n## Otra cosa\ntexto suelto tras cerrar\n",
         );
         assert_eq!((l.nombre, l.sprint), (None, None));
         let e = &l.entradas;
@@ -530,17 +531,17 @@ mod tests {
         assert_eq!(e[0].objetivo, "Objetivo uno\n\ndos");
         assert_eq!(e[0].criterios, ["Criterio a sigue a", "Criterio b"]);
         assert_eq!(e[1].clave, "sin-clave-aqui");
-        assert_eq!(e[1].objetivo, "");
+        assert_eq!(e[1].objetivo, "o");
     }
 
     #[test]
     fn cabecera_y_dependencias() {
         let l = ok(
-            "\u{feff}---\r\nlote: fase-3\r\nsprint: \"3\"\r\n---\r\n### A1 Uno\r\n- c\r\n### A2 Dos\r\nDependencias: A1, TASK-019, A1\r\n- c\r\n",
+            "\u{feff}---\r\nlote: fase-3\r\nsprint: \"3\"\r\n---\r\n### A1 Uno\r\n> o\r\n- c\r\n### A2 Dos\r\n> o\r\nDependencias: A1, TASK-019, A1\r\n- c\r\n",
         );
         assert_eq!((l.nombre.as_deref(), l.sprint), (Some("fase-3"), Some(3)));
         assert_eq!(l.entradas[1].dependencias, ["A1", "TASK-019"]);
-        assert_eq!(l.entradas[1].linea, 7);
+        assert_eq!(l.entradas[1].linea, 8);
         for (t, m) in [
             ("---\nlote: x\n", "cabecera-sin-cerrar"),
             ("---\nhito: 3\n---\n### A1 x\n- c\n", "cabecera-invalida"),
@@ -573,14 +574,17 @@ mod tests {
             ("### A1 x\nprosa\n- a\n", "linea-suelta"),
             ("### A1 x\n- a\n> objetivo tarde\n", "linea-suelta"),
             ("### A1 x\n- a\n\n  sangrada tras blanco\n", "linea-suelta"),
-            ("### A1 x\n- a\n### A1 y\n- b\n", "clave-duplicada"),
             (
-                "### A1 x\n- a\nDependencias: B9\n",
+                "### A1 x\n> o\n- a\n### A1 y\n> o\n- b\n",
+                "clave-duplicada",
+            ),
+            (
+                "### A1 x\n> o\n- a\nDependencias: B9\n",
                 "referencia-inexistente",
             ),
-            ("### A1 x\n- a\nDependencias: A1\n", "ciclo"),
+            ("### A1 x\n> o\n- a\nDependencias: A1\n", "ciclo"),
             (
-                "### A1 x\n- a\nDependencias: A3\n### A2 y\n- b\nDependencias: A1\n### A3 z\n- c\nDependencias: A2\n",
+                "### A1 x\n> o\n- a\nDependencias: A3\n### A2 y\n> o\n- b\nDependencias: A1\n### A3 z\n> o\n- c\nDependencias: A2\n",
                 "ciclo",
             ),
         ] {
@@ -589,20 +593,37 @@ mod tests {
         // La linea del error es la del `###` de la entrada mala.
         let mut t = String::new();
         for i in 1..=40 {
-            t.push_str(&format!("### T{i} tarea {i}\n- criterio {i}\n"));
+            t.push_str(&format!("### T{i} tarea {i}\n> o\n- criterio {i}\n"));
         }
-        let malo = t.replacen("### T30 tarea 30\n- criterio 30\n", "### T30 tarea 30\n", 1);
+        let malo = t.replacen(
+            "### T30 tarea 30\n> o\n- criterio 30\n",
+            "### T30 tarea 30\n",
+            1,
+        );
         let e = leer(&malo, Limites::default()).err();
         assert_eq!(
             e.map(|e| (e.motivo, e.claves)),
             Some((
                 "enunciado-no-listo",
                 vec![
-                    ("linea", "59".to_owned()),
+                    ("linea", "88".to_owned()),
                     ("causa", "sin-criterios".to_owned())
                 ]
             ))
         );
+        // Una entrada sin lineas `>` no esta lista, como la rechazaria
+        // `plan` (A3-08).
+        let e = leer("### A1 x\n> o\n- a\n### A2 y\n- b\n", Limites::default()).err();
+        assert_eq!(
+            e.map(|e| (e.motivo, e.claves)),
+            Some((
+                "enunciado-no-listo",
+                vec![
+                    ("linea", "4".to_owned()),
+                    ("causa", "objetivo-vacio".to_owned())
+                ]
+            ))
+        );
         let e = leer(&t.replace("criterio 3\n", "x\nprosa\n"), Limites::default()).err();
         assert_eq!(e.map(|e| e.motivo), Some("linea-suelta"));
     }
@@ -617,13 +638,13 @@ mod tests {
             leer(&"x".repeat(65), l).err().map(|e| e.motivo),
             Some("fichero-grande")
         );
-        let tres = "### A1 a\n- c\n### A2 b\n- c\n### A3 c\n- c\n";
-        assert_eq!(tres.len(), 39);
+        let tres = "### A1 a\n> o\n- c\n### A2 b\n> o\n- c\n### A3 c\n> o\n- c\n";
+        assert_eq!(tres.len(), 51);
         assert_eq!(
             leer(tres, l).err().map(|e| e.motivo),
             Some("demasiadas-entradas")
         );
-        assert_eq!(leer(&tres[..26], l).map(|x| x.entradas.len()), Ok(2));
+        assert_eq!(leer(&tres[..34], l).map(|x| x.entradas.len()), Ok(2));
     }
 
     #[test]
diff --git a/crates/tc2/src/importar/mod.rs b/crates/tc2/src/importar/mod.rs
index 4369816..975b69f 100644
--- a/crates/tc2/src/importar/mod.rs
+++ b/crates/tc2/src/importar/mod.rs
@@ -240,33 +240,23 @@ fn de_error(e: ErrorLote) -> Informe {
     i.diagnostico(e.mensaje)
 }
 
-/// Lee el fichero sin cargar mas del tope de bytes: se lee como mucho un
-/// byte de mas, y si llega a leerse el fichero es demasiado grande (sin
-/// carrera entre mirar el tamano y leer).
+/// Lee el fichero con [`crate::lectura::fichero`] (tope de `limites`,
+/// UTF-8, no vacio; ruta y error saneados, F1-8). Un fallo es `uso`.
 fn leer_fichero(ruta: &Path, limites: Limites) -> Result<String, Informe> {
-    use std::io::Read as _;
-    let no_se_lee = |e: std::io::Error| {
-        Informe::nuevo(NOMBRE, Resultado::Uso)
-            .con("motivo", "no-se-lee")
-            .diagnostico(
-                Mensaje::nuevo(&txt::NO_SE_LEE)
-                    .con("ruta", ruta.display().to_string())
-                    .con("error", e.to_string()),
-            )
-    };
-    let f = std::fs::File::open(ruta).map_err(no_se_lee)?;
-    let tope = u64::try_from(limites.bytes).unwrap_or(u64::MAX);
-    let mut bytes = Vec::new();
-    f.take(tope.saturating_add(1))
-        .read_to_end(&mut bytes)
-        .map_err(no_se_lee)?;
-    if bytes.len() > limites.bytes {
-        return Err(de_error(fichero::grande(limites)));
-    }
-    String::from_utf8(bytes).map_err(|_| {
+    use crate::lectura::{self, Motivo};
+    lectura::fichero(ruta, limites.bytes).map_err(|n| {
+        let (motivo, m) = match n.motivo {
+            Motivo::Grande => return de_error(fichero::grande(limites)),
+            Motivo::NoSeLee => (
+                "no-se-lee",
+                Mensaje::nuevo(&txt::NO_SE_LEE).con("error", n.error),
+            ),
+            Motivo::NoUtf8 => ("no-utf8", Mensaje::nuevo(&txt::NO_UTF8)),
+            Motivo::Vacio => ("vacio", Mensaje::nuevo(&txt::VACIO)),
+        };
         Informe::nuevo(NOMBRE, Resultado::Uso)
-            .con("motivo", "no-utf8")
-            .diagnostico(Mensaje::nuevo(&txt::NO_UTF8).con("ruta", ruta.display().to_string()))
+            .con("motivo", motivo)
+            .diagnostico(m.con("ruta", n.ruta))
     })
 }
 
@@ -467,6 +457,27 @@ mod tests {
         }
     }
 
+    /// La ruta de `import` es un dato no confiable (F1-8): el diagnostico de
+    /// un fichero que no se lee sale saneado, sin saltos de linea ni
+    /// secuencias de escape (A3-03).
+    #[test]
+    fn ruta_que_no_se_lee_saneada() {
+        let ruta = std::env::temp_dir()
+            .join("tc2-tests")
+            .join("no\nexiste\u{1b}[31m.md");
+        let i = leer_fichero(&ruta, Limites::default()).err();
+        assert_eq!(
+            i.as_ref().map(|i| (i.resultado, i.valor("motivo"))),
+            Some((Resultado::Uso, Some("no-se-lee")))
+        );
+        let d = i
+            .and_then(|i| i.diagnostico)
+            .map(|m| m.texto(crate::i18n::Idioma::Es))
+            .unwrap_or_default();
+        assert!(d.contains("no existe [31m.md"), "{d}");
+        assert!(!d.contains('\n') && !d.contains('\u{1b}'), "{d:?}");
+    }
+
     #[test]
     fn anotacion_en_una_linea() {
         let op = Importar::nuevo("fase-3", 3, Vec::new());
diff --git a/crates/tc2/src/lectura.rs b/crates/tc2/src/lectura.rs
new file mode 100644
index 0000000..340a16e
--- /dev/null
+++ b/crates/tc2/src/lectura.rs
@@ -0,0 +1,65 @@
+//! Lectura acotada de un fichero externo (A3-03): la de `plan guardar`,
+//! `approve --pedir-cambios` e `import`, en un solo sitio y con un solo
+//! tope. Cada orden traduce el fallo a su propio informe.
+//!
+//! El fichero lo pasa quien lanza la orden: es un dato no confiable (F1-8),
+//! asi que la ruta y el error del sistema salen ya saneados
+//! ([`sanear`]: controles a espacio, 200 caracteres como mucho).
+
+use std::io::Read as _;
+use std::path::Path;
+
+use crate::plataforma::errores::sanear;
+
+/// Tope de un fichero externo (D4 de TASK-029).
+pub const MAX_BYTES: usize = 1 << 20;
+
+/// Por que no se leyo el fichero.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Motivo {
+    /// No se abre o falla la lectura (`error` dice por que).
+    NoSeLee,
+    /// Pasa del tope.
+    Grande,
+    /// No es UTF-8.
+    NoUtf8,
+    /// Vacio o solo con espacios.
+    Vacio,
+}
+
+/// Un fichero que no se leyo: el motivo, y la ruta y el error saneados
+/// (`error` vacio salvo en [`Motivo::NoSeLee`]).
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct NoSeLeyo {
+    /// Por que.
+    pub motivo: Motivo,
+    /// La ruta, saneada.
+    pub ruta: String,
+    /// El error del sistema, saneado.
+    pub error: String,
+}
+
+/// Lee `ruta` sin cargar mas de `tope` bytes: se lee como mucho un byte de
+/// mas, y si llega a leerse el fichero es demasiado grande (sin carrera
+/// entre mirar el tamano y leer). UTF-8 estricto y no vacio.
+pub fn fichero(ruta: &Path, tope: usize) -> Result<String, NoSeLeyo> {
+    let mal = |motivo: Motivo, error: String| NoSeLeyo {
+        motivo,
+        ruta: sanear(&ruta.display().to_string()),
+        error,
+    };
+    let no_se_lee = |e: std::io::Error| mal(Motivo::NoSeLee, sanear(&e.to_string()));
+    let f = std::fs::File::open(ruta).map_err(no_se_lee)?;
+    let mut bytes = Vec::new();
+    f.take(u64::try_from(tope).unwrap_or(u64::MAX).saturating_add(1))
+        .read_to_end(&mut bytes)
+        .map_err(no_se_lee)?;
+    if bytes.len() > tope {
+        return Err(mal(Motivo::Grande, String::new()));
+    }
+    let texto = String::from_utf8(bytes).map_err(|_| mal(Motivo::NoUtf8, String::new()))?;
+    if texto.trim().is_empty() {
+        return Err(mal(Motivo::Vacio, String::new()));
+    }
+    Ok(texto)
+}
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 68abc7e..ce02215 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -48,6 +48,7 @@ pub mod importar;
 pub mod init;
 pub mod instalacion;
 pub mod inventario;
+mod lectura;
 pub mod plan;
 pub mod plataforma;
 pub mod progreso;
diff --git a/crates/tc2/src/plan/aprobar.rs b/crates/tc2/src/plan/aprobar.rs
index 78319bb..f95b16e 100644
--- a/crates/tc2/src/plan/aprobar.rs
+++ b/crates/tc2/src/plan/aprobar.rs
@@ -701,6 +701,30 @@ mod tests {
         assert_eq!(juegos_de_rondas(&rutas, "TASK-001"), vec![dos]);
     }
 
+    /// Los roles elegidos por dominio (F4-6): carril `fix` con la etiqueta
+    /// `wasm` deja el esqueleto de arquitectura y rendimiento, un juego que
+    /// `roles::seleccionar` no da. Solo lo reconoce la ronda de sus
+    /// peticiones (A3-02); sin las rutas, no.
+    #[test]
+    fn esqueleto_de_los_roles_de_la_ronda() {
+        let mut t = Tarea::nueva("TASK-001", "Uno", "2026-11-01");
+        t.etiquetas = vec!["wasm".to_owned()];
+        let juego: Vec<Rol> = roles::todos()
+            .filter(|r| ["brainstorm-arquitectura", "brainstorm-rendimiento"].contains(&r.id))
+            .copied()
+            .collect();
+        assert_eq!(juego.len(), 2);
+        let b = "TASK-001/planificacion/brainstorm/";
+        let rutas = vec![
+            format!("{b}peticion-brainstorm-arquitectura-1.md"),
+            format!("{b}peticion-brainstorm-rendimiento-1.md"),
+            format!("{b}peticion-unificador-1.md"),
+        ];
+        let plan = p::plan_final("TASK-001", "Uno", &juego);
+        assert!(es_esqueleto(&plan, &t, &rutas));
+        assert!(!es_esqueleto(&plan, &t, &[]));
+    }
+
     /// `exige_red` y el sello firmado con `Plan-Blob` de `plan-final.md`.
     #[test]
     fn exige_red_y_firma_con_plan_blob() {
diff --git a/crates/tc2/src/plan/guardar.rs b/crates/tc2/src/plan/guardar.rs
index 5e8e142..d6c7f13 100644
--- a/crates/tc2/src/plan/guardar.rs
+++ b/crates/tc2/src/plan/guardar.rs
@@ -30,9 +30,6 @@ use super::operacion::EN_DISENO;
 /// Nombre de la operacion (en el commit y en la anotacion).
 pub const NOMBRE_OP: &str = "plan-guardar";
 
-/// Tope del fichero que se publica (el del import, D4 de TASK-029).
-pub const MAX_BYTES: usize = 1 << 20;
-
 /// Si `destino` es uno de la lista: `plan-final.md` o
 /// `brainstorm/salida-<rol>-<n>.md` con un rol conocido y `n` un entero
 /// positivo sin ceros delante. Devuelve, para una salida, el nombre de su
diff --git a/crates/tc2/src/plan/mod.rs b/crates/tc2/src/plan/mod.rs
index 446ebc3..ef0da3f 100644
--- a/crates/tc2/src/plan/mod.rs
+++ b/crates/tc2/src/plan/mod.rs
@@ -58,6 +58,7 @@ use crate::estado::{self, Operacion};
 use crate::git::{self, Git};
 use crate::i18n::Mensaje;
 use crate::i18n::catalogo::plan as txt;
+use crate::identidad::Quien;
 use crate::plataforma::errores::sanear;
 use crate::toma::Entorno;
 
@@ -133,40 +134,28 @@ fn orden_guardar(args: &[String], e: &Entorno<'_>) -> Informe {
     guardar_en(e, id, destino, &contenido)
 }
 
-/// Lee el fichero a publicar sin cargar mas del tope: UTF-8, no vacio y de
-/// [`guardar::MAX_BYTES`] como mucho. Un fallo es `uso` (como el fichero de
-/// `import`).
+/// Lee el fichero a publicar con [`crate::lectura::fichero`]: UTF-8, no
+/// vacio y de [`crate::lectura::MAX_BYTES`] como mucho. Un fallo es `uso`
+/// (como el fichero de `import`).
 fn leer_fichero(ruta: &Path) -> Result<String, Informe> {
-    use std::io::Read as _;
-    let mal = |motivo: &str, m: Mensaje| {
+    use crate::lectura::{self, Motivo};
+    lectura::fichero(ruta, lectura::MAX_BYTES).map_err(|n| {
+        let (motivo, m) = match n.motivo {
+            Motivo::NoSeLee => (
+                "no-se-lee",
+                Mensaje::nuevo(&txt::NO_SE_LEE).con("error", n.error),
+            ),
+            Motivo::Grande => (
+                "grande",
+                Mensaje::nuevo(&txt::GRANDE).con("max", lectura::MAX_BYTES.to_string()),
+            ),
+            Motivo::NoUtf8 => ("no-utf8", Mensaje::nuevo(&txt::NO_UTF8)),
+            Motivo::Vacio => ("vacio", Mensaje::nuevo(&txt::VACIO)),
+        };
         Informe::nuevo(NOMBRE, Resultado::Uso)
             .con("motivo", motivo)
-            .diagnostico(m.con("ruta", sanear(&ruta.display().to_string())))
-    };
-    let no_se_lee = |e: std::io::Error| {
-        mal(
-            "no-se-lee",
-            Mensaje::nuevo(&txt::NO_SE_LEE).con("error", sanear(&e.to_string())),
-        )
-    };
-    let f = std::fs::File::open(ruta).map_err(no_se_lee)?;
-    let tope = u64::try_from(guardar::MAX_BYTES).unwrap_or(u64::MAX);
-    let mut bytes = Vec::new();
-    f.take(tope.saturating_add(1))
-        .read_to_end(&mut bytes)
-        .map_err(no_se_lee)?;
-    if bytes.len() > guardar::MAX_BYTES {
-        return Err(mal(
-            "grande",
-            Mensaje::nuevo(&txt::GRANDE).con("max", guardar::MAX_BYTES.to_string()),
-        ));
-    }
-    let texto =
-        String::from_utf8(bytes).map_err(|_| mal("no-utf8", Mensaje::nuevo(&txt::NO_UTF8)))?;
-    if texto.trim().is_empty() {
-        return Err(mal("vacio", Mensaje::nuevo(&txt::VACIO)));
-    }
-    Ok(texto)
+            .diagnostico(m.con("ruta", n.ruta))
+    })
 }
 
 /// `plan` de la tarea `id` con el carril pedido en el repo de `e`.
@@ -174,7 +163,7 @@ pub fn plan_en(e: &Entorno<'_>, id: &str, carril: Option<Carril>) -> Informe {
     let i = match paso_plan(e, id, carril) {
         Ok(i) | Err(i) => i,
     };
-    de_la_orden(NOMBRE, i, id)
+    de_la_orden(NOMBRE, i, "id", id)
 }
 
 fn uso_approve(motivo: &str) -> Informe {
@@ -266,32 +255,15 @@ fn orden_approve_hito(args: &[String], e: &Entorno<'_>) -> Informe {
 /// `hito=` primero, `aprobadas=` con el lote y, si alguna tarea no esta
 /// lista, `tareas=ID:motivo,...`.
 pub fn aprobar_hito_en(e: &Entorno<'_>, hito: u32) -> Informe {
-    let mut i = match paso_aprobar_hito(e, hito) {
+    let i = match paso_aprobar_hito(e, hito) {
         Ok(i) | Err(i) => i,
     };
-    aprobar::NOMBRE.clone_into(&mut i.comando);
-    i.claves
-        .retain(|(k, _)| k != "operacion" && k != "id" && k != "hito");
-    i.claves
-        .insert(0, ("hito".to_owned(), Valor::Texto(hito.to_string())));
-    if i.resultado == Resultado::Sinred && i.valor("pendiente").is_none() {
-        i = i.con("pendiente", "no");
-    }
-    i
+    de_la_orden(aprobar::NOMBRE, i, "hito", &hito.to_string())
 }
 
 fn paso_aprobar_hito(e: &Entorno<'_>, hito: u32) -> Result<Informe, Informe> {
-    // Como `approve`: quien, lo primero.
-    let quien = match (e.quien)() {
-        Ok(q) => q,
-        Err(i) if i.resultado == Resultado::Rechazada => return Err(sin_persona(Some(i))),
-        Err(i) => return Err(i),
-    };
-    let Some(op) = aprobar::AprobarHito::nueva(hito, &quien, e.ahora) else {
-        return Err(sin_persona(None));
-    };
-    traer(e.git, e.dir)?;
-    let op = op.con_config(&config_de_base(e.git, e.dir)?);
+    let (op, config) = persona_y_config(e, |q| aprobar::AprobarHito::nueva(hito, q, e.ahora))?;
+    let op = op.con_config(&config);
     let t = escribir(e, &op);
     match t.resultado {
         Resultado::Ok => Ok(t.con_lista("aprobadas", op.aprobadas())),
@@ -319,7 +291,7 @@ pub fn aprobar_en(e: &Entorno<'_>, id: &str) -> Informe {
     let i = match paso_aprobar(e, id) {
         Ok(i) | Err(i) => i,
     };
-    de_la_orden(aprobar::NOMBRE, i, id)
+    de_la_orden(aprobar::NOMBRE, i, "id", id)
 }
 
 /// `approve --pedir-cambios` de `contenido` en la tarea `id` del repo de
@@ -328,7 +300,7 @@ pub fn pedir_cambios_en(e: &Entorno<'_>, id: &str, contenido: &str) -> Informe {
     let i = match paso_pedir_cambios(e, id, contenido) {
         Ok(i) | Err(i) => i,
     };
-    de_la_orden(aprobar::NOMBRE, i, id)
+    de_la_orden(aprobar::NOMBRE, i, "id", id)
 }
 
 /// `rechazada motivo=identidad rol=persona` (RR-04), con lo que diga el
@@ -346,19 +318,29 @@ fn sin_persona(rechazo: Option<Informe>) -> Informe {
     i
 }
 
-fn paso_aprobar(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
-    // Lo primero, quien: el agente (o sin identidad) no llega a leer ni a
-    // escribir nada.
+/// El arranque comun de `approve` y `approve --hito`: lo primero, quien (el
+/// agente, o sin identidad, no llega a leer ni a escribir nada:
+/// `rechazada motivo=identidad rol=persona`); con la operacion que da
+/// `nueva` para esa persona, `fetch` y la configuracion de la base.
+fn persona_y_config<O>(
+    e: &Entorno<'_>,
+    nueva: impl FnOnce(&Quien) -> Option<O>,
+) -> Result<(O, Config), Informe> {
     let quien = match (e.quien)() {
         Ok(q) => q,
         Err(i) if i.resultado == Resultado::Rechazada => return Err(sin_persona(Some(i))),
         Err(i) => return Err(i),
     };
-    let Some(op) = Aprobar::nueva(id, &quien, e.ahora) else {
+    let Some(op) = nueva(&quien) else {
         return Err(sin_persona(None));
     };
     traer(e.git, e.dir)?;
-    let op = op.con_config(&config_de_base(e.git, e.dir)?);
+    Ok((op, config_de_base(e.git, e.dir)?))
+}
+
+fn paso_aprobar(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
+    let (op, config) = persona_y_config(e, |q| Aprobar::nueva(id, q, e.ahora))?;
+    let op = op.con_config(&config);
     let t = escribir(e, &op);
     match t.resultado {
         Resultado::Ok | Resultado::SinCambios => Ok(con_seleccion(t, op.roles())),
@@ -413,7 +395,7 @@ pub fn guardar_en(e: &Entorno<'_>, id: &str, destino: &str, contenido: &str) ->
     let i = match paso_guardar(e, id, destino, contenido) {
         Ok(i) | Err(i) => i,
     };
-    let mut i = de_la_orden(NOMBRE, i, id);
+    let mut i = de_la_orden(NOMBRE, i, "id", id);
     i.claves
         .insert(1, ("destino".to_owned(), Valor::Texto(destino.to_owned())));
     i
@@ -536,14 +518,16 @@ fn con_dueno(mut i: Informe, dueno: Option<String>) -> Informe {
     i
 }
 
-/// El informe como de la orden `nombre`: el comando, `id=` el primero y sin
-/// la clave `operacion` de la transaccion. Un `sinred` sin `pendiente` dice
-/// `pendiente=no` (exige red).
-fn de_la_orden(nombre: &str, mut i: Informe, id: &str) -> Informe {
+/// El informe como de la orden `nombre`: el comando, `clave=valor` el
+/// primero (`id=` de una tarea o `hito=` de `approve --hito`) y sin la
+/// clave `operacion` ni el `id=` de la transaccion. Un `sinred` sin
+/// `pendiente` dice `pendiente=no` (exige red).
+fn de_la_orden(nombre: &str, mut i: Informe, clave: &str, valor: &str) -> Informe {
     nombre.clone_into(&mut i.comando);
-    i.claves.retain(|(k, _)| k != "operacion" && k != "id");
     i.claves
-        .insert(0, ("id".to_owned(), Valor::Texto(id.to_owned())));
+        .retain(|(k, _)| k != "operacion" && k != "id" && k != clave);
+    i.claves
+        .insert(0, (clave.to_owned(), Valor::Texto(valor.to_owned())));
     if i.resultado == Resultado::Sinred && i.valor("pendiente").is_none() {
         i = i.con("pendiente", "no");
     }
@@ -554,7 +538,6 @@ fn de_la_orden(nombre: &str, mut i: Informe, id: &str) -> Informe {
 mod tests {
     use super::*;
     use crate::i18n::Idioma;
-    use crate::identidad::Quien;
 
     /// Carpeta temporal propia del test: se borra al acabar.
     struct Carpeta(std::path::PathBuf);
@@ -695,7 +678,7 @@ mod tests {
         let _ = std::fs::remove_dir_all(&raiz);
         std::fs::create_dir_all(&raiz)?;
         let _borrar = Carpeta(raiz.clone());
-        let tope = guardar::MAX_BYTES;
+        let tope = crate::lectura::MAX_BYTES;
         for (nombre, bytes, motivo) in [
             ("vacio.md", b" \n\t\n".to_vec(), Some("vacio")),
             ("binario.md", vec![b'a', 0xff, 0xfe], Some("no-utf8")),
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-141/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-141/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-141/tarea.md                                    | 45 ++++++++++++++++++++++++++++++++++++++-------
 3 files changed, 38 insertions(+), 7 deletions(-)
````

## Tests del diff (5 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/alta.rs" ":(literal)crates/tc2/tests/estimacion.rs" ":(literal)crates/tc2/tests/importar.rs" ":(literal)crates/tc2/tests/progreso.rs" ":(literal)crates/tc2/tests/toma.rs"
````

````
crates/tc2/tests/alta.rs       | 24 +++++++++++++++++++++++-
 crates/tc2/tests/estimacion.rs | 29 +++++++++++++++++++++++++----
 crates/tc2/tests/importar.rs   | 17 +++++++++++------
 crates/tc2/tests/progreso.rs   | 14 +++++++++++---
 crates/tc2/tests/toma.rs       |  2 +-
 5 files changed, 71 insertions(+), 15 deletions(-)
````
## Candidatos 4D

Pistas generadas desde el diff, no hallazgos: comprueba cada una (Regla 4D de `revision.md`).

### Posible duplicacion con lo que ya existia

Definiciones nuevas con nombre igual o parecido a otra de la rama base. Mira si hacen lo mismo con pequenas diferencias (duplicacion real = IMPORTANTE).

- `Motivo` (crates/tc2/src/lectura.rs:19) ~ `Motivo` (crates/tc2/src/config/mod.rs:321), `motivo` (crates/tc2/src/desinstalar/mod.rs:240), `motivo` (crates/tc2/src/desinstalar/texto.rs:40) y 11 mas
- `fichero` (crates/tc2/src/lectura.rs:45) ~ `fichero` (crates/distribucion/src/lib.rs:140), `Fichero` (crates/tc2/src/confianza/cierre.rs:34), `fichero` (crates/tc2/tests/importar.rs:25) y 1 mas
- `de_la_orden` (crates/tc2/src/plan/mod.rs:525) ~ `de_la_orden` (crates/tc2/src/ciclo/mod.rs:423)

