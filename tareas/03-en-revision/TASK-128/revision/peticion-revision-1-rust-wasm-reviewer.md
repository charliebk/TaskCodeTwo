# Peticion de revision — TASK-128 (ronda 1)

- Tarea: TASK-128 — Seguimiento de F3-11 y F3-12: aviso de solape contra uno mismo, textos al catalogo y test intermitente de progreso
- Rama revisada: fix/task-128-seguimiento-de-f3-11-y-f3-12-aviso-de-so
- Rama base: develop
- Commit revisado (HEAD): 6fdb49100d3066da1c76a255b171cb8ed232d052
- Fecha: 2026-11-18
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-128 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-128/tarea.md
- tareas/03-en-revision/TASK-128/planificacion/
- crates/tc2/src/i18n/catalogo/toma.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/progreso/fichero.rs
- crates/tc2/src/progreso/mod.rs
- crates/tc2/src/solape.rs
- crates/tc2/src/toma/mod.rs
- crates/tc2/tests/liberar.rs
- crates/tc2/tests/progreso.rs
- crates/tc2/tests/solape.rs

## Commits a revisar (git log develop..HEAD)

````
6fdb491 chore(TASK-128): autorrevision
78f965e fix(TASK-128): el anterior del progreso es el fichero que habia, byte a byte
9df6b42 fix(TASK-128): sin aviso de solape contra uno mismo y textos al catalogo
06a1d7d chore(TASK-128): tarea en curso
````

## Diff de tu dominio (9 fichero(s) de 10; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/i18n/catalogo/toma.rs b/crates/tc2/src/i18n/catalogo/toma.rs
index 1a50719..e4688cc 100644
--- a/crates/tc2/src/i18n/catalogo/toma.rs
+++ b/crates/tc2/src/i18n/catalogo/toma.rs
@@ -45,4 +45,12 @@ crate::i18n::textos! {
         es: "nota: tu rama aun no cambia nada; en curso con rama publicada: {tareas}",
         en: "note: your branch changes nothing yet; in progress with a published branch: {tareas}",
     }
+    AVISO_Y_MAS {
+        es: "aviso: y {n} tareas mas tocan lo mismo (todas en la clave solape)",
+        en: "warning: and {n} more tasks touch the same (all of them in the solape key)",
+    }
+    NOTA_Y_MAS {
+        es: "{tareas}; y {n} mas",
+        en: "{tareas}; and {n} more",
+    }
 }
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 914275b..1c31009 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -437,6 +437,10 @@ fn con_progreso(
         };
     }
     match refrescar(&id) {
+        Ok(progreso::Refresco::EscritoSinAnterior) => ConProgreso {
+            informe: i.con(progreso::CLAVE, progreso::SIN_ANTERIOR),
+            aviso: None,
+        },
         Ok(_) => ConProgreso {
             informe: i,
             aviso: None,
@@ -915,11 +919,19 @@ mod tests {
             "{}",
             s.stderr
         );
-        let s = con_progreso(ok, |_| Ok(progreso::Refresco::Escrito)).salida(false, Idioma::Es);
+        let s =
+            con_progreso(ok.clone(), |_| Ok(progreso::Refresco::Escrito)).salida(false, Idioma::Es);
         assert_eq!(
             (s.codigo, s.stdout.as_str(), s.stderr.as_str()),
             (0, "start ok id=TASK-001\n", "")
         );
+        // TASK-128: sin rotar el anterior, la linea lo dice y stderr no.
+        let s = con_progreso(ok, |_| Ok(progreso::Refresco::EscritoSinAnterior))
+            .salida(false, Idioma::Es);
+        assert_eq!(
+            (s.codigo, s.stdout.as_str(), s.stderr.as_str()),
+            (0, "start ok id=TASK-001 progreso=sin-anterior\n", "")
+        );
         let mut llamado = false;
         let mal = Informe::nuevo("pausa", Resultado::Rechazada).con("id", "TASK-001");
         let s = con_progreso(mal, |_| {
diff --git a/crates/tc2/src/progreso/fichero.rs b/crates/tc2/src/progreso/fichero.rs
index 44ad69d..7ba8937 100644
--- a/crates/tc2/src/progreso/fichero.rs
+++ b/crates/tc2/src/progreso/fichero.rs
@@ -134,10 +134,19 @@ fn bytes(ruta: &Path) -> Option<Vec<u8>> {
 
 /// Un progreso valido de `ruta`, si lo hay.
 fn de(ruta: &Path, id: &str) -> Option<Progreso> {
-    let p: Progreso = serde_json::from_slice(&bytes(ruta)?).ok()?;
+    de_bytes(&bytes(ruta)?, id)
+}
+
+fn de_bytes(b: &[u8], id: &str) -> Option<Progreso> {
+    let p: Progreso = serde_json::from_slice(b).ok()?;
     (p.valido() && p.id == id).then_some(p)
 }
 
+/// Si `b` es un progreso valido de `id`.
+fn valido(b: &[u8], id: &str) -> bool {
+    de_bytes(b, id).is_some()
+}
+
 /// Lo que da la lectura de una tarea.
 #[derive(Debug, Clone, PartialEq)]
 pub enum Lectura {
@@ -164,19 +173,47 @@ pub fn leer(carpeta: &Path, id: &str) -> Lectura {
     de(&ruta_anterior(carpeta, id), id).map_or(Lectura::Ilegible, Lectura::Anterior)
 }
 
+/// Que paso con el `.anterior.json` al escribir un progreso.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Anterior {
+    /// El que habia paso a ser el anterior.
+    Rotado,
+    /// No habia progreso: no hay anterior que escribir.
+    SinPrevio,
+    /// Habia progreso pero no paso a ser el anterior: no se pudo leer
+    /// entero (a medias, o abierto por otro proceso) o no se pudo escribir.
+    /// El nuevo se escribio igual (TASK-128: ya no se calla).
+    NoEscrito,
+}
+
 /// Escribe `p` en `carpeta`: antes, el que habia pasa a `.anterior.json`.
 /// Cada paso, de una vez ([`crate::huella::escribir_local`]). Si el anterior
 /// no se puede escribir, el progreso se escribe igual (revision 1 de
 /// TASK-084, R1-04): el anterior solo es el respaldo de un fichero a medias,
-/// y sin el nuevo la linea se quedaria con el dato viejo.
-pub fn escribir(carpeta: &Path, p: &Progreso) -> std::io::Result<()> {
+/// y sin el nuevo la linea se quedaria con el dato viejo. Lo que paso con el
+/// anterior se devuelve (TASK-128) para que la transicion lo diga.
+pub fn escribir(carpeta: &Path, p: &Progreso) -> std::io::Result<Anterior> {
     let r = ruta(carpeta, &p.id);
     // Solo un anterior valido: si el actual quedo a medias, el anterior que
     // habia sigue siendo el ultimo bueno.
-    if let Some(previo) = de(&r, &p.id) {
-        let _ = crate::huella::escribir_local(&ruta_anterior(carpeta, &p.id), &previo.json());
-    }
-    crate::huella::escribir_local(&r, &p.json())
+    let anterior = if !r.exists() {
+        Anterior::SinPrevio
+    } else {
+        // Los bytes tal cual, no el progreso vuelto a serializar: un `f64`
+        // leido y reescrito por `serde_json` puede cambiar en la ultima cifra
+        // (TASK-128), y el anterior dejaria de ser el fichero que habia.
+        match bytes(&r).filter(|b| valido(b, &p.id)) {
+            Some(previo) => {
+                match crate::huella::escribir_local(&ruta_anterior(carpeta, &p.id), &previo) {
+                    Ok(()) => Anterior::Rotado,
+                    Err(_) => Anterior::NoEscrito,
+                }
+            }
+            None => Anterior::NoEscrito,
+        }
+    };
+    crate::huella::escribir_local(&r, &p.json())?;
+    Ok(anterior)
 }
 
 /// Borra el progreso de `id` (y su anterior). Que no exista no es un fallo.
@@ -243,6 +280,52 @@ pub(crate) mod tests {
         d
     }
 
+    /// TASK-128: `escribir` dice que paso con el anterior. Sin progreso,
+    /// `SinPrevio`; con uno bueno, `Rotado`; si el anterior no se puede
+    /// escribir (aqui, una carpeta ocupa su ruta), `NoEscrito` y el nuevo
+    /// queda escrito igual; sobre uno a medias, `NoEscrito`.
+    #[test]
+    fn escribir_dice_que_paso_con_el_anterior() -> std::io::Result<()> {
+        let d = dir("rotacion");
+        let mut p = ejemplo("TASK-001");
+        assert_eq!(escribir(&d, &p)?, Anterior::SinPrevio);
+        p.calculada = 2_000;
+        assert_eq!(escribir(&d, &p)?, Anterior::Rotado);
+        std::fs::remove_file(ruta_anterior(&d, "TASK-001"))?;
+        std::fs::create_dir_all(ruta_anterior(&d, "TASK-001").join("ocupada"))?;
+        p.calculada = 3_000;
+        assert_eq!(escribir(&d, &p)?, Anterior::NoEscrito);
+        assert_eq!(leer(&d, "TASK-001"), Lectura::Bueno(p.clone()));
+        std::fs::remove_dir_all(ruta_anterior(&d, "TASK-001"))?;
+        let json = p.json();
+        std::fs::write(ruta(&d, "TASK-001"), &json[..json.len() / 2])?;
+        p.calculada = 4_000;
+        assert_eq!(escribir(&d, &p)?, Anterior::NoEscrito);
+        assert_eq!(leer(&d, "TASK-001"), Lectura::Bueno(p));
+        let _ = std::fs::remove_dir_all(&d);
+        Ok(())
+    }
+
+    /// TASK-128, causa del `progreso::c3` intermitente: el anterior es el
+    /// fichero que habia byte a byte, tambien con un `transcurrido` que
+    /// `serde_json` no devuelve igual al leerlo y reescribirlo (este valor
+    /// cambia en la ultima cifra; con el progreso re-serializado, falla).
+    #[test]
+    fn el_anterior_es_el_fichero_que_habia_byte_a_byte() -> std::io::Result<()> {
+        let d = dir("bytes");
+        let mut p = ejemplo("TASK-001");
+        p.transcurrido = 0.009_027_842_777_777_777;
+        escribir(&d, &p)?;
+        let habia = std::fs::read(ruta(&d, "TASK-001"))?;
+        assert_eq!(habia, p.json());
+        let mut q = p.clone();
+        q.calculada = 2_000;
+        assert_eq!(escribir(&d, &q)?, Anterior::Rotado);
+        assert_eq!(std::fs::read(ruta_anterior(&d, "TASK-001"))?, habia);
+        let _ = std::fs::remove_dir_all(&d);
+        Ok(())
+    }
+
     /// C3 (en pequeno) y C8: al reescribir, el bueno pasa a `.anterior`; un
     /// fichero a medias da el anterior; sin fichero, ninguno.
     #[test]
diff --git a/crates/tc2/src/progreso/mod.rs b/crates/tc2/src/progreso/mod.rs
index ea4c5cd..370bc40 100644
--- a/crates/tc2/src/progreso/mod.rs
+++ b/crates/tc2/src/progreso/mod.rs
@@ -214,6 +214,10 @@ pub fn orden_estado(stdin: &str, cwd: Option<&Path>, ahora: u64) -> String {
 pub enum Refresco {
     /// Escribio el progreso de la tarea.
     Escrito,
+    /// Escribio el progreso, pero el que habia no paso a `.anterior.json`
+    /// ([`fichero::Anterior::NoEscrito`]): la linea lo dice con
+    /// `progreso=sin-anterior` (TASK-128).
+    EscritoSinAnterior,
     /// La tarea no esta en curso ni en revision (o la deja quien la lanza):
     /// borro su progreso.
     Borrado,
@@ -258,7 +262,10 @@ pub fn refrescar(g: &Git, dir: &Path, id: &str, ahora: u64) -> Result<Refresco,
     let h = crate::estimacion::historico(&leidas, &commits);
     match calcular(&t, &c, &h, &commits, ahora) {
         Some(p) => fichero::escribir(&carpeta, &p)
-            .map(|()| Refresco::Escrito)
+            .map(|a| match a {
+                fichero::Anterior::NoEscrito => Refresco::EscritoSinAnterior,
+                fichero::Anterior::Rotado | fichero::Anterior::SinPrevio => Refresco::Escrito,
+            })
             .map_err(|e| e.to_string()),
         None => quitar_en(&carpeta, id),
     }
@@ -276,9 +283,14 @@ pub const CLAVE: &str = "progreso";
 /// Valor de [`CLAVE`].
 pub const NO_ESCRITO: &str = "no-escrito";
 
+/// Valor de [`CLAVE`] cuando el progreso se escribio pero el que habia no
+/// paso a `.anterior.json` (TASK-128). No cambia el resultado ni sale por
+/// stderr: el progreso esta al dia, solo falta su respaldo.
+pub const SIN_ANTERIOR: &str = "sin-anterior";
+
 /// Escribe `p`, ya calculado, en el clon de `dir` (lo usa `start`, que lo
 /// calcula dentro de la toma con el historico que ya leyo).
-pub fn escribir_en(g: &Git, dir: &Path, p: &Progreso) -> Result<(), String> {
+pub fn escribir_en(g: &Git, dir: &Path, p: &Progreso) -> Result<fichero::Anterior, String> {
     let (_, carpeta) = carpeta_de(g, dir)?;
     fichero::escribir(&carpeta, p).map_err(|e| e.to_string())
 }
diff --git a/crates/tc2/src/solape.rs b/crates/tc2/src/solape.rs
index 7285b9c..602ec68 100644
--- a/crates/tc2/src/solape.rs
+++ b/crates/tc2/src/solape.rs
@@ -22,11 +22,11 @@ use std::path::Path;
 
 use crate::contrato::Informe;
 use crate::git::{self, Git};
+use crate::i18n::Mensaje;
 use crate::i18n::catalogo::toma as txt;
-use crate::i18n::{Mensaje, Texto};
 use crate::plataforma::Personas;
 use crate::situacion::{self, Ficha};
-use crate::tarea::Documento;
+use crate::tarea::{Asignado, Documento};
 
 /// Estados que cuentan para el solape (D2a de TASK-082): las tareas que
 /// alguien esta cambiando o que esperan revision con sus cambios en la rama.
@@ -43,6 +43,9 @@ pub struct EnCurso {
     pub quien: String,
     /// Su rama, si la tiene.
     pub rama: Option<String>,
+    /// Quien la lleva, para no avisar a una persona contra si misma
+    /// (TASK-128); `None` sin asignar.
+    pub asignado: Option<Asignado>,
 }
 
 /// Las tareas en curso o en revision de unos `tarea.md` leidos, por ID y
@@ -57,6 +60,7 @@ pub fn en_curso<'a>(docs: impl IntoIterator<Item = &'a Documento>) -> Vec<EnCurs
             quien: situacion::quien(f.asignado.as_ref(), &sin_lista).texto(),
             id: f.nodo.id,
             rama: f.rama,
+            asignado: f.asignado,
         })
         .collect();
     v.sort_by(|a, b| a.id.cmp(&b.id));
@@ -151,23 +155,6 @@ pub const MAX_AVISOS: usize = 5;
 /// Tareas como mucho en la nota del primer `start`; el resto, «y N mas».
 pub const MAX_TAREAS_NOTA: usize = 5;
 
-/// La linea final de los avisos que no caben en [`MAX_AVISOS`]. Vive aqui
-/// y no en el catalogo de `toma` porque la correccion de H3 solo podia
-/// tocar este modulo.
-pub static AVISO_Y_MAS: Texto = Texto {
-    clave: "tc2::solape::AVISO_Y_MAS",
-    es: "aviso: y {n} tareas mas tocan lo mismo (todas en la clave solape)",
-    en: "warning: and {n} more tasks touch the same (all of them in the solape key)",
-};
-
-/// La cola de la nota cuando no caben todas las tareas
-/// ([`MAX_TAREAS_NOTA`]).
-pub static NOTA_Y_MAS: Texto = Texto {
-    clave: "tc2::solape::NOTA_Y_MAS",
-    es: "{tareas}; y {n} mas",
-    en: "{tareas}; and {n} more",
-};
-
 /// Lo que sale del solape de una tarea.
 #[derive(Debug, Clone, Default, PartialEq, Eq)]
 pub struct Solape {
@@ -189,15 +176,34 @@ pub struct Solape {
     pub truncado: bool,
 }
 
+/// Las tareas de `en_curso` que cuentan para el solape de `id`: ni la propia
+/// ni las que lleva la misma persona que `propio` (por id de cuenta, como
+/// decide la toma si una tarea es suya), que no se avisa a nadie contra si
+/// mismo (TASK-128). Las heredadas y las sin asignar si cuentan.
+fn otras<'a>(en_curso: &'a [EnCurso], id: &str, propio: &Asignado) -> Vec<&'a EnCurso> {
+    en_curso
+        .iter()
+        .filter(|t| t.id != id && !t.asignado.as_ref().is_some_and(|a| a.misma_persona(propio)))
+        .collect()
+}
+
 /// El solape de la tarea `id` (rama `rama`, ya creada en local) con las
-/// tareas `en_curso`, desde la carpeta `dir` del repo. Sin red: lo que
+/// tareas `en_curso`, desde la carpeta `dir` del repo. Las de la persona
+/// `propio` no cuentan. Sin red: lo que
 /// sabe el clon tras el `fetch` de `start`.
 ///
 /// `None` (ninguna clave) si no hay otra tarea en curso o el clon no conoce
 /// ninguna rama de `origin` (sin remoto o sin haberlo traido nunca). Un
 /// `diff` que falla no rompe nada: la tarea queda en `incompletas`.
-pub fn calcular(g: &Git, dir: &Path, id: &str, rama: &str, en_curso: &[EnCurso]) -> Option<Solape> {
-    let otras: Vec<&EnCurso> = en_curso.iter().filter(|t| t.id != id).collect();
+pub fn calcular(
+    g: &Git,
+    dir: &Path,
+    id: &str,
+    rama: &str,
+    propio: &Asignado,
+    en_curso: &[EnCurso],
+) -> Option<Solape> {
+    let otras = otras(en_curso, id, propio);
     if otras.is_empty() {
         return None;
     }
@@ -354,7 +360,7 @@ impl Solape {
             .collect();
         match self.cruces.len().saturating_sub(MAX_AVISOS) {
             0 => {}
-            n => v.push(Mensaje::nuevo(&AVISO_Y_MAS).con("n", n.to_string())),
+            n => v.push(Mensaje::nuevo(&txt::AVISO_Y_MAS).con("n", n.to_string())),
         }
         if self.propia_vacia && !self.publicadas.is_empty() {
             let tareas: Vec<String> = self
@@ -377,7 +383,7 @@ impl Solape {
                     0 => nota.con("tareas", tareas),
                     n => nota.con_mensaje(
                         "tareas",
-                        Mensaje::nuevo(&NOTA_Y_MAS)
+                        Mensaje::nuevo(&txt::NOTA_Y_MAS)
                             .con("tareas", tareas)
                             .con("n", n.to_string()),
                     ),
@@ -418,6 +424,24 @@ fn resumen<'a>(v: impl Iterator<Item = &'a str>) -> String {
 #[cfg(test)]
 mod tests {
     use super::*;
+
+    /// TASK-128 C2: los textos del solape viven en el catalogo de `toma`, y
+    /// asi los recorre `catalogo_completo` (traducidos, mismos huecos).
+    #[test]
+    fn textos_del_solape_en_el_catalogo_de_toma() {
+        for t in [&txt::AVISO_Y_MAS, &txt::NOTA_Y_MAS] {
+            assert!(
+                txt::TODOS.iter().any(|c| c.clave == t.clave),
+                "{} no esta en catalogo::toma::TODOS",
+                t.clave
+            );
+            assert!(
+                t.clave.starts_with("tc2::i18n::catalogo::toma::"),
+                "{}",
+                t.clave
+            );
+        }
+    }
     use crate::tarea::{Asignado, Cuerpo, Tarea};
 
     fn conjunto(v: &[&str]) -> BTreeSet<String> {
@@ -733,13 +757,42 @@ mod tests {
                     id: "TASK-001".to_owned(),
                     quien: "@ana".to_owned(),
                     rama: Some("feature/task-001-x".to_owned()),
+                    asignado: Asignado::persona(7, "ana"),
                 },
                 EnCurso {
                     id: "TASK-003".to_owned(),
                     quien: "—".to_owned(),
                     rama: Some("feature/task-003-x".to_owned()),
+                    asignado: None,
                 },
             ]
         );
     }
+
+    /// TASK-128 C1: no cuentan la propia ni las de la misma persona (por id
+    /// de cuenta, aunque el login haya cambiado); si las de otra persona y
+    /// las sin asignar.
+    #[test]
+    #[allow(clippy::unwrap_used)]
+    fn otras_quita_la_propia_y_las_de_la_misma_persona() {
+        let t = |id: &str, asignado: Option<Asignado>| EnCurso {
+            id: id.to_owned(),
+            quien: String::new(),
+            rama: None,
+            asignado,
+        };
+        let lista = [
+            t("TASK-001", Asignado::persona(8, "beto")),
+            t("TASK-002", Asignado::persona(7, "ana")),
+            t("TASK-003", Asignado::persona(7, "ana-nueva")),
+            t("TASK-004", None),
+            t("TASK-005", Asignado::persona(9, "ana")),
+        ];
+        let propio = Asignado::persona(7, "ana").unwrap();
+        let ids: Vec<&str> = otras(&lista, "TASK-002", &propio)
+            .iter()
+            .map(|t| t.id.as_str())
+            .collect();
+        assert_eq!(ids, ["TASK-001", "TASK-004", "TASK-005"]);
+    }
 }
diff --git a/crates/tc2/src/toma/mod.rs b/crates/tc2/src/toma/mod.rs
index 3b7367f..a6ef595 100644
--- a/crates/tc2/src/toma/mod.rs
+++ b/crates/tc2/src/toma/mod.rs
@@ -190,7 +190,7 @@ fn paso(e: &Entorno<'_>, id: &str, s: &mut Option<Solape>) -> Result<Informe, In
     if empezada {
         // 5c. Ya es suya y esta empezada: crear solo lo que falte.
         let i = destino.crear(None, &quien);
-        return Ok(destino.con_solape(i, || en_curso_de(&vista), s));
+        return Ok(destino.con_solape(i, &quien, || en_curso_de(&vista), s));
     }
     // 5. El ID no tiene rama de nadie mas. La suya propia (la de `rama:`)
     // no cuenta: tras `liberar` o `reasignar` (F3-10) la siguiente persona
@@ -224,20 +224,26 @@ fn paso(e: &Entorno<'_>, id: &str, s: &mut Option<Solape>) -> Result<Informe, In
             }
             // El progreso en vivo (F3-12), con lo que ya calculo la toma.
             // Un fallo no cambia el resultado: `progreso=no-escrito`, y
-            // `lib.rs` lo avisa por stderr.
+            // `lib.rs` lo avisa por stderr; sin rotar el anterior,
+            // `progreso=sin-anterior` sin aviso (TASK-128).
             if i.resultado.es_exito()
                 && let Some(p) = op.progreso()
-                && crate::progreso::escribir_en(g, &principal, &p).is_err()
             {
-                i = i.con(crate::progreso::CLAVE, crate::progreso::NO_ESCRITO);
+                match crate::progreso::escribir_en(g, &principal, &p) {
+                    Err(_) => i = i.con(crate::progreso::CLAVE, crate::progreso::NO_ESCRITO),
+                    Ok(crate::progreso::fichero::Anterior::NoEscrito) => {
+                        i = i.con(crate::progreso::CLAVE, crate::progreso::SIN_ANTERIOR);
+                    }
+                    Ok(_) => {}
+                }
             }
             // El solape (F3-11) con las tareas en curso que leyo la toma.
-            Ok(destino.con_solape(i, || op.en_curso(), s))
+            Ok(destino.con_solape(i, &quien, || op.en_curso(), s))
         }
         // Entre la lectura y el push paso a ser suya y empezada.
         Resultado::SinCambios => {
             let i = destino.crear(None, &quien);
-            Ok(destino.con_solape(i, || en_curso_de(&vista), s))
+            Ok(destino.con_solape(i, &quien, || en_curso_de(&vista), s))
         }
         Resultado::Rechazada => Err(rechazo_de_toma(t, &op)),
         _ => Err(t),
@@ -370,10 +376,12 @@ impl Destino<'_> {
 
     /// Anade a `i` las claves del solape (F3-11) y lo deja en `salida`, solo
     /// si `i` es un exito: la rama propia ya existe en local. `en_curso` se
-    /// evalua solo entonces. Nunca cambia el resultado de `i`.
+    /// evalua solo entonces. Las tareas de `quien` no cuentan (TASK-128).
+    /// Nunca cambia el resultado de `i`.
     fn con_solape(
         &self,
         i: Informe,
+        quien: &Quien,
         en_curso: impl FnOnce() -> Option<Vec<EnCurso>>,
         salida: &mut Option<Solape>,
     ) -> Informe {
@@ -383,7 +391,9 @@ impl Destino<'_> {
         let Some(lista) = en_curso() else {
             return i;
         };
-        let Some(s) = solape::calcular(self.g, self.principal, self.id, self.rama, &lista) else {
+        let propio = quien.asignado();
+        let Some(s) = solape::calcular(self.g, self.principal, self.id, self.rama, &propio, &lista)
+        else {
             return i;
         };
         let i = s.con_claves(i);
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-128/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-128/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-128/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````

## Tests del diff (3 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/liberar.rs" ":(literal)crates/tc2/tests/progreso.rs" ":(literal)crates/tc2/tests/solape.rs"
````

````
crates/tc2/tests/liberar.rs  | 18 ++++++++----------
 crates/tc2/tests/progreso.rs | 15 +++++++++++++--
 crates/tc2/tests/solape.rs   | 66 +++++++++++++++++++++++++++++++++++++++++++++++-------------------
 3 files changed, 68 insertions(+), 31 deletions(-)
````
