# Peticion de revision — TASK-076 (ronda 1)

- Tarea: TASK-076 — F2-4b Nombres unicos de tarea y rama
- Rama revisada: feature/task-076-f2-4b-nombres-unicos-de-tarea-y-rama
- Rama base: develop
- Commit revisado (HEAD): ebc7fa5647f8386bf0266e16eab63a8a34d7e8a8
- Fecha: 2026-11-16
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-076 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-076/tarea.md
- tareas/03-en-revision/TASK-076/planificacion/
- crates/tc2/src/alta.rs
- crates/tc2/src/estado/lectura.rs
- crates/tc2/src/git/estado.rs
- crates/tc2/src/git/mod.rs
- crates/tc2/src/git/puerto.rs
- crates/tc2/src/git/rama.rs
- crates/tc2/src/i18n/catalogo/estado.rs
- crates/tc2/src/i18n/catalogo/git.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/tarea/mod.rs
- crates/tc2/src/tarea/modelo.rs
- crates/tc2/src/tarea/texto.rs
- crates/tc2/tests/alta.rs
- crates/tc2/tests/comun/mod.rs
- crates/tc2/tests/estado_casos.rs
- crates/tc2/tests/tarea.rs

## Commits a revisar (git log develop..HEAD)

````
ebc7fa5 docs(TASK-076): autorrevision con rust-wasm-reviewer y code-quality-reviewer
0f11a56 fix(TASK-076): el alta fija tipo y rama desde la misma constante
40eeeb6 test(TASK-076): aviso de parecidas en espanol e ingles
da479a2 feat(TASK-076): aviso de titulo duplicado o parecido a una tarea abierta
bada16c feat(TASK-076): leer_varios con un solo git cat-file --batch
7bd6455 feat(TASK-076): el ID nuevo salta las ramas de tarea locales y remotas
24d4c81 feat(TASK-076): plegado, slug y rama derivada del ID en tarea.md
d66e30f chore(TASK-076): tarea en curso
````

## Diff de tu dominio (16 fichero(s) de 17; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/alta.rs b/crates/tc2/src/alta.rs
index 3e1d439..a5a7155 100644
--- a/crates/tc2/src/alta.rs
+++ b/crates/tc2/src/alta.rs
@@ -7,11 +7,15 @@
 //! operacion sobre lo nuevo y sale el numero siguiente. La carpeta publicada
 //! es el compare-and-swap: reservar un numero sin crearla no reservaria nada.
 
-use std::cell::Cell;
+use std::cell::{Cell, RefCell};
+use std::collections::BTreeSet;
 use std::time::{SystemTime, UNIX_EPOCH};
 
 use crate::estado::{Cambio, Estado, Fallo, Operacion};
-use crate::tarea::{Cuerpo, Tarea};
+use crate::i18n::catalogo::estado as txt;
+use crate::i18n::{Idioma, Mensaje};
+use crate::tarea::texto::{Parecido, parecido, slug};
+use crate::tarea::{Cuerpo, Documento, Tarea};
 
 /// Nombre de la operacion y de la orden del binario.
 pub const NOMBRE: &str = "new";
@@ -24,7 +28,9 @@ const PREFIJO: &str = "TASK-";
 pub struct Alta {
     titulo: String,
     fecha: String,
+    ocupados: BTreeSet<u32>,
     id: Cell<Option<u32>>,
+    parecidas: RefCell<Vec<Parecida>>,
 }
 
 impl Alta {
@@ -38,15 +44,35 @@ impl Alta {
         Self {
             titulo: titulo.to_owned(),
             fecha: fecha.to_owned(),
+            ocupados: BTreeSet::new(),
             id: Cell::new(None),
+            parecidas: RefCell::new(Vec::new()),
         }
     }
 
+    /// IDs que ya tienen rama de tarea (`git::ids_con_rama`, leidos una vez
+    /// antes de la transaccion): el ID nuevo queda por encima (F2-4b).
+    #[must_use]
+    pub fn con_ocupados(mut self, ocupados: BTreeSet<u32>) -> Self {
+        self.ocupados = ocupados;
+        self
+    }
+
+    /// Rama derivada del ID del ultimo intento (F2-4b).
+    pub fn rama(&self) -> Option<String> {
+        self.id().map(|id| rama_de(&id, &self.titulo))
+    }
+
     /// ID calculado en el ultimo intento (el que publico, si la transaccion
     /// salio bien).
     pub fn id(&self) -> Option<String> {
         self.id.get().map(formato_id)
     }
+
+    /// Tareas abiertas parecidas vistas en el ultimo intento.
+    pub fn parecidas(&self) -> Vec<Parecida> {
+        self.parecidas.borrow().clone()
+    }
 }
 
 impl Operacion for Alta {
@@ -61,9 +87,14 @@ impl Operacion for Alta {
         true
     }
     fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
-        let n = siguiente_id(&estado.rutas())?;
+        let rutas = estado.rutas();
+        let n = siguiente_id(&rutas, &self.ocupados)?;
         self.id.set(Some(n));
         let id = formato_id(n);
+        comprobar_rama(&id, &self.titulo)?;
+        // Sobre el mismo estado fresco del intento: dos altas a la vez se ven
+        // (la segunda en publicar avisa de la primera, TASK-076 C1).
+        *self.parecidas.borrow_mut() = parecidas(estado, &rutas, &self.titulo)?;
         Ok(vec![Cambio::nuevo(
             format!("{id}/tarea.md"),
             tarea_md(&id, &self.titulo, &self.fecha),
@@ -71,6 +102,75 @@ impl Operacion for Alta {
     }
 }
 
+/// Estados en los que una tarea esta abierta: contra ellas se avisa.
+const ABIERTOS: [&str; 4] = ["planificada", "en-diseno", "en-curso", "en-revision"];
+
+/// Una tarea abierta con un titulo igual o casi igual al del alta (RR-12).
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Parecida {
+    /// ID de la tarea parecida.
+    pub id: String,
+    /// Su titulo.
+    pub titulo: String,
+    /// Como se parece.
+    pub parecido: Parecido,
+}
+
+/// `TASK-<cifras>/tarea.md`.
+fn es_tarea_md(ruta: &str) -> bool {
+    numero(ruta).is_some() && ruta.split_once('/').is_some_and(|(_, r)| r == "tarea.md")
+}
+
+/// Tareas abiertas cuyo titulo se parece a `titulo`, en orden de ID. Los
+/// `tarea.md` que no se leen o son de formato 1 se saltan sin fallar.
+fn parecidas(estado: &dyn Estado, rutas: &[String], titulo: &str) -> Result<Vec<Parecida>, Fallo> {
+    let mds: Vec<String> = rutas.iter().filter(|r| es_tarea_md(r)).cloned().collect();
+    let mut v = Vec::new();
+    for md in estado.leer_varios(&mds)?.into_iter().flatten() {
+        let Ok(Documento::Formato2(t, _)) = crate::tarea::leer(&md) else {
+            continue;
+        };
+        if !ABIERTOS.contains(&t.estado.as_str()) {
+            continue;
+        }
+        if let Some(p) = parecido(titulo, &t.titulo) {
+            v.push(Parecida {
+                id: t.id,
+                titulo: t.titulo,
+                parecido: p,
+            });
+        }
+    }
+    Ok(v)
+}
+
+/// Una linea de aviso con las tareas parecidas, en `idioma`; `None` si no
+/// hay. Va a stderr aunque el alta salga bien: es solo un aviso (D1).
+pub fn aviso_parecidas(p: &[Parecida], idioma: Idioma) -> Option<String> {
+    if p.is_empty() {
+        return None;
+    }
+    let lista: Vec<String> = p
+        .iter()
+        .map(|x| {
+            let m = match x.parecido {
+                Parecido::Duplicado => Mensaje::nuevo(&txt::PARECIDA_DUPLICADO),
+                Parecido::Casi(c) => Mensaje::nuevo(&txt::PARECIDA_CASI)
+                    .con("unidad", (c / 100).to_string())
+                    .con("centesimas", format!("{:02}", c % 100)),
+            };
+            m.con("id", x.id.as_str())
+                .con("titulo", x.titulo.as_str())
+                .texto(idioma)
+        })
+        .collect();
+    Some(
+        Mensaje::nuevo(&txt::AVISO_PARECIDAS)
+            .con("lista", lista.join("; "))
+            .texto(idioma),
+    )
+}
+
 /// Reconstruye un [`Alta`] desde su anotacion (para el catalogo).
 pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
     let titulo = a.strip_prefix(NOMBRE)?.strip_prefix('|')?;
@@ -100,19 +200,52 @@ fn numero(ruta: &str) -> Option<u32> {
     cifras.parse().ok()
 }
 
-/// Siguiente ID: el maximo de las carpetas `TASK-<cifras>` mas uno (cuentan
-/// terminadas y descartadas, que no se borran; los huecos no se rellenan).
-pub fn siguiente_id(rutas: &[String]) -> Result<u32, Fallo> {
-    let maximo = rutas.iter().filter_map(|r| numero(r)).max().unwrap_or(0);
+/// Siguiente ID: el maximo de las carpetas `TASK-<cifras>` y de los IDs con
+/// rama (`ocupados`) mas uno (cuentan terminadas y descartadas, que no se
+/// borran; los huecos no se rellenan). Adenda de F2-4b al ADR F1-2 punto 3.
+pub fn siguiente_id(rutas: &[String], ocupados: &BTreeSet<u32>) -> Result<u32, Fallo> {
+    let maximo = rutas
+        .iter()
+        .filter_map(|r| numero(r))
+        .chain(ocupados.iter().copied())
+        .max()
+        .unwrap_or(0);
     maximo
         .checked_add(1)
         .ok_or_else(|| Fallo::Error((&crate::i18n::catalogo::estado::SIN_IDS).into()))
 }
 
+/// Tipo con el que nace una tarea (plantilla de F1-9, punto 6).
+const TIPO: &str = "feature";
+
+/// Rama de la tarea `id` (ADR F1-3 y F1-9 punto 6): `feature/task-nnn-<slug>`
+/// con el slug del titulo (F2-4b).
+pub fn rama_de(id: &str, titulo: &str) -> String {
+    crate::git::nombre_de_rama(id, TIPO, &slug(titulo))
+}
+
+/// La rama derivada pasa la validacion de las operaciones de rama: si no,
+/// `start` no podria crearla.
+fn comprobar_rama(id: &str, titulo: &str) -> Result<(), Fallo> {
+    crate::git::rama::rama_valida(NOMBRE, id, TIPO, &slug(titulo))
+        .map(drop)
+        .map_err(|i| {
+            Fallo::Error(
+                crate::i18n::Mensaje::nuevo(&crate::i18n::catalogo::estado::RAMA_DERIVADA_INVALIDA)
+                    .con("id", id)
+                    .con("motivo", i.valor("motivo").unwrap_or_default()),
+            )
+        })
+}
+
 /// `tarea.md` de una tarea recien creada: el modelo de F2-5 con la
-/// plantilla de F1-9 (punto 6).
+/// plantilla de F1-9 (punto 6) y la rama derivada del ID (F2-4b).
 pub fn tarea_md(id: &str, titulo: &str, fecha: &str) -> String {
-    Tarea::nueva(id, titulo, fecha).escribir(&Cuerpo::plantilla())
+    let mut t = Tarea::nueva(id, titulo, fecha);
+    // El tipo y la rama salen de la misma constante: no pueden divergir.
+    TIPO.clone_into(&mut t.tipo);
+    t.rama = Some(rama_de(id, titulo));
+    t.escribir(&Cuerpo::plantilla())
 }
 
 /// Fecha de hoy en UTC, `AAAA-MM-DD`, sin dependencias.
@@ -161,19 +294,27 @@ mod tests {
             "TASK-/tarea.md",
             "TASK-+9/tarea.md",
         ]);
-        assert_eq!(siguiente_id(&r), Ok(8));
+        let sin = BTreeSet::new();
+        assert_eq!(siguiente_id(&r, &sin), Ok(8));
         // Una carpeta en minusculas puesta a mano tambien ocupa su numero.
         assert_eq!(
-            siguiente_id(&rutas(&["task-013/tarea.md", "TASK-002/tarea.md"])),
+            siguiente_id(&rutas(&["task-013/tarea.md", "TASK-002/tarea.md"]), &sin),
             Ok(14)
         );
-        assert_eq!(siguiente_id(&rutas(&["ñ/x", "TAS"])), Ok(1));
-        assert_eq!(siguiente_id(&[]), Ok(1));
-        assert_eq!(siguiente_id(&rutas(&["TASK-999/tarea.md"])), Ok(1000));
+        assert_eq!(siguiente_id(&rutas(&["ñ/x", "TAS"]), &sin), Ok(1));
+        assert_eq!(siguiente_id(&[], &sin), Ok(1));
+        assert_eq!(siguiente_id(&rutas(&["TASK-999/tarea.md"]), &sin), Ok(1000));
         assert_eq!(formato_id(1000), "TASK-1000");
         assert_eq!(formato_id(8), "TASK-008");
         let tope = rutas(&[&format!("TASK-{}/tarea.md", u32::MAX)]);
-        assert!(matches!(siguiente_id(&tope), Err(Fallo::Error(_))));
+        assert!(matches!(siguiente_id(&tope, &sin), Err(Fallo::Error(_))));
+        // F2-4b: una rama sin carpeta tambien ocupa su numero (adenda F1-2).
+        assert_eq!(siguiente_id(&r, &BTreeSet::from([5, 12])), Ok(13));
+        assert_eq!(siguiente_id(&r, &BTreeSet::from([3])), Ok(8));
+        assert!(matches!(
+            siguiente_id(&[], &BTreeSet::from([u32::MAX])),
+            Err(Fallo::Error(_))
+        ));
     }
 
     #[test]
@@ -218,7 +359,82 @@ mod tests {
         );
         assert!(t.contains("id: \"TASK-008\"\n"));
         assert!(t.contains("creado: \"2026-11-15\"\nactualizado: \"2026-11-15\"\n"));
-        assert!(t.contains("rama: null\n"));
+        assert!(
+            t.contains("rama: \"feature/task-008-c-ruta-x-fecha\"\n"),
+            "{t}"
+        );
+    }
+
+    /// C7: el `tarea.md` lleva `rama: feature/task-nnn-<slug>` y esa rama
+    /// pasa `rama_valida`; un titulo sin letras latinas da el slug `tarea`.
+    #[test]
+    fn rama_derivada_del_id_pasa_rama_valida() {
+        for (titulo, rama) in [
+            (
+                "Nombres únicos de tarea",
+                "feature/task-076-nombres-unicos-de-tarea",
+            ),
+            ("Задача без латиницы", "feature/task-076-tarea"),
+            ("漢字", "feature/task-076-tarea"),
+        ] {
+            assert_eq!(rama_de("TASK-076", titulo), rama);
+            let md = tarea_md("TASK-076", titulo, "2026-11-16");
+            let leida = match crate::tarea::leer(&md) {
+                Ok(crate::tarea::Documento::Formato2(t, _)) => t.rama,
+                _ => None,
+            };
+            assert_eq!(leida.as_deref(), Some(rama));
+            let slug = rama.trim_start_matches("feature/task-076-");
+            assert_eq!(
+                crate::git::rama::rama_valida("t", "TASK-076", TIPO, slug).ok(),
+                Some(rama.to_owned())
+            );
+            assert_eq!(comprobar_rama("TASK-076", titulo), Ok(()));
+        }
+        assert_eq!(rama_de("TASK-1000", "x"), "feature/task-1000-x");
+    }
+
+    #[test]
+    fn aviso_de_parecidas_en_los_dos_idiomas() {
+        assert_eq!(aviso_parecidas(&[], Idioma::Es), None);
+        let p = [
+            Parecida {
+                id: "TASK-007".into(),
+                titulo: "Exportar informes".into(),
+                parecido: Parecido::Duplicado,
+            },
+            Parecida {
+                id: "TASK-009".into(),
+                titulo: "Exprotar informes".into(),
+                parecido: Parecido::Casi(86),
+            },
+            Parecida {
+                id: "TASK-010".into(),
+                titulo: "x".into(),
+                parecido: Parecido::Casi(100),
+            },
+        ];
+        assert_eq!(
+            aviso_parecidas(&p, Idioma::Es).as_deref(),
+            Some(
+                "aviso: ya hay tareas abiertas con un titulo igual o parecido: TASK-007 \"Exportar informes\" (duplicado); TASK-009 \"Exprotar informes\" (parecido 0,86); TASK-010 \"x\" (parecido 1,00)"
+            )
+        );
+        assert_eq!(
+            aviso_parecidas(&p[1..2], Idioma::En).as_deref(),
+            Some(
+                "warning: there are already open tasks with the same or a similar title: TASK-009 \"Exprotar informes\" (similarity 0.86)"
+            )
+        );
+        assert!(es_tarea_md("TASK-007/tarea.md") && es_tarea_md("task-7/tarea.md"));
+        for r in [
+            "TASK-007/plan.md",
+            "TASK-007/x/tarea.md",
+            "notas/tarea.md",
+            "TASK-x/tarea.md",
+        ] {
+            assert!(!es_tarea_md(r), "{r}");
+        }
     }
 
     #[test]
diff --git a/crates/tc2/src/estado/lectura.rs b/crates/tc2/src/estado/lectura.rs
index b5d8449..e1b5fde 100644
--- a/crates/tc2/src/estado/lectura.rs
+++ b/crates/tc2/src/estado/lectura.rs
@@ -9,7 +9,7 @@ use std::path::{Path, PathBuf};
 use super::transaccion::Fallo;
 use super::{CARPETA, OP, RAMA};
 use crate::contrato::{Informe, Resultado};
-use crate::git::estado::{hay_remoto, leer_blob, listar_arbol};
+use crate::git::estado::{hay_remoto, leer_blob, leer_blobs, listar_arbol};
 use crate::git::{Cambio, Git, REMOTO, localizar_principal};
 
 /// Lo que una operacion puede leer del estado.
@@ -18,6 +18,21 @@ pub trait Estado {
     fn rutas(&self) -> Vec<String>;
     /// Contenido de `ruta`, o `None` si no existe.
     fn leer(&self, ruta: &str) -> Result<Option<String>, Fallo>;
+    /// Contenido de varias rutas, en el mismo orden (F2-4b). Por defecto,
+    /// una a una con [`Estado::leer`]; [`Vista`] lo hace en un solo proceso.
+    fn leer_varios(&self, rutas: &[String]) -> Result<Vec<Option<String>>, Fallo> {
+        rutas.iter().map(|r| self.leer(r)).collect()
+    }
+}
+
+/// Un fallo de lectura de git como [`Fallo::Error`]. Sin diagnostico queda
+/// la linea de contrato, que no se traduce (ADR F1-4).
+fn fallo_de_lectura(i: &Informe) -> Fallo {
+    Fallo::Error(
+        i.diagnostico
+            .clone()
+            .unwrap_or_else(|| crate::i18n::Mensaje::externo(i.linea())),
+    )
 }
 
 /// Vista de un commit de la rama de estado (o del estado vacio si la rama no
@@ -82,18 +97,34 @@ impl Estado for Vista<'_> {
         match &self.commit {
             Some(c) if self.rutas.contains(ruta) => leer_blob(self.git, &self.dir, OP, c, ruta)
                 .map(Some)
-                .map_err(|i| {
-                    // Sin diagnostico queda la linea de contrato, que no se
-                    // traduce (ADR F1-4).
-                    Fallo::Error(
-                        i.diagnostico
-                            .clone()
-                            .unwrap_or_else(|| crate::i18n::Mensaje::externo(i.linea())),
-                    )
-                }),
+                .map_err(|i| fallo_de_lectura(&i)),
             _ => Ok(None),
         }
     }
+
+    /// Las rutas del commit se leen con un solo `git cat-file --batch`; las
+    /// superpuestas y las que no existen no lanzan nada.
+    fn leer_varios(&self, rutas: &[String]) -> Result<Vec<Option<String>>, Fallo> {
+        let del_commit: Vec<&str> = rutas
+            .iter()
+            .filter(|r| !self.encima.contains_key(*r) && self.rutas.contains(*r))
+            .map(String::as_str)
+            .collect();
+        let mut leidos = match (&self.commit, del_commit.is_empty()) {
+            (Some(c), false) => leer_blobs(self.git, &self.dir, OP, c, &del_commit)
+                .map_err(|i| fallo_de_lectura(&i))?
+                .into_iter(),
+            _ => Vec::new().into_iter(),
+        };
+        Ok(rutas
+            .iter()
+            .map(|r| match self.encima.get(r) {
+                Some(c) => Some(c.clone()),
+                None if self.commit.is_some() && self.rutas.contains(r) => leidos.next().flatten(),
+                None => None,
+            })
+            .collect())
+    }
 }
 
 /// Vista del estado tal como lo conoce este clon: lo ultimo traido de
diff --git a/crates/tc2/src/git/estado.rs b/crates/tc2/src/git/estado.rs
index 6e997d4..5b8ce3e 100644
--- a/crates/tc2/src/git/estado.rs
+++ b/crates/tc2/src/git/estado.rs
@@ -114,6 +114,81 @@ pub(crate) fn leer_blob(git: &Git, dir: &Path, op: &str, commit: &str, ruta: &st
     )
 }
 
+/// Contenido de varias `rutas` de `commit` con un solo `git cat-file
+/// --batch` (F2-4b): en el mismo orden, `None` si no existe o no es un
+/// fichero. Una ruta con salto de linea no cabe en la lista de stdin: esa se
+/// lee sola con [`leer_blob`].
+pub(crate) fn leer_blobs(
+    git: &Git,
+    dir: &Path,
+    op: &str,
+    commit: &str,
+    rutas: &[&str],
+) -> Paso<Vec<Option<String>>> {
+    let (en_lote, sueltas): (Vec<&str>, Vec<&str>) =
+        rutas.iter().partition(|r| !r.contains(['\n', '\r']));
+    let mut leidos: std::collections::HashMap<&str, Option<String>> =
+        std::collections::HashMap::new();
+    if !en_lote.is_empty() {
+        let entrada: String = en_lote.iter().map(|r| format!("{commit}:{r}\n")).collect();
+        let s = git
+            .ejecutar_con_entrada(dir, Subcomando::CatFile, &["--batch"], entrada.as_bytes())
+            .map_err(|e| {
+                Informe::nuevo(op, Resultado::Error)
+                    .con("git", Subcomando::CatFile.nombre())
+                    .diagnostico(e.mensaje())
+            })?;
+        if s.codigo != 0 {
+            return Err(Informe::nuevo(op, Resultado::Error)
+                .con("git", Subcomando::CatFile.nombre())
+                .diagnostico(
+                    Mensaje::nuevo(&txt::GIT_FALLO)
+                        .con("subcomando", Subcomando::CatFile.nombre())
+                        .con("error", s.stderr.lines().next().unwrap_or_default()),
+                ));
+        }
+        let contenidos = partir_batch(&s.stdout, en_lote.len()).ok_or_else(|| {
+            Informe::nuevo(op, Resultado::Error).diagnostico(&txt::CAT_FILE_INCOMPLETO)
+        })?;
+        leidos.extend(en_lote.iter().copied().zip(contenidos));
+    }
+    for r in sueltas {
+        leidos.insert(r, Some(leer_blob(git, dir, op, commit, r)?));
+    }
+    Ok(rutas
+        .iter()
+        .map(|r| leidos.get(r).cloned().flatten())
+        .collect())
+}
+
+/// Parte la salida de `cat-file --batch` en `n` objetos: por cada uno,
+/// `<sha> <tipo> <bytes>\n<contenido>\n`, o `<nombre> missing\n`. Solo los
+/// `blob` dan contenido. `None` si la salida no tiene esa forma.
+fn partir_batch(salida: &[u8], n: usize) -> Option<Vec<Option<String>>> {
+    let mut v = Vec::with_capacity(n);
+    let mut resto = salida;
+    for _ in 0..n {
+        let fin = resto.iter().position(|b| *b == b'\n')?;
+        let cabecera = std::str::from_utf8(resto.get(..fin)?).ok()?;
+        resto = resto.get(fin + 1..)?;
+        let mut campos = cabecera.rsplitn(3, ' ');
+        let ultimo = campos.next()?;
+        if ultimo == "missing" || ultimo == "ambiguous" {
+            v.push(None);
+            continue;
+        }
+        let bytes: usize = ultimo.parse().ok()?;
+        let tipo = campos.next()?;
+        let contenido = resto.get(..bytes)?;
+        if resto.get(bytes) != Some(&b'\n') {
+            return None;
+        }
+        resto = resto.get(bytes + 1..)?;
+        v.push((tipo == "blob").then(|| String::from_utf8_lossy(contenido).into_owned()));
+    }
+    resto.is_empty().then_some(v)
+}
+
 /// Mensaje completo de un commit.
 pub(crate) fn mensaje_de(git: &Git, dir: &Path, op: &str, commit: &str) -> Paso<String> {
     exigir(
@@ -280,7 +355,33 @@ pub(crate) fn hay_remoto(git: &Git, dir: &Path, op: &str) -> Paso<bool> {
 
 #[cfg(test)]
 mod tests {
-    use super::ruta_valida;
+    use super::{partir_batch, ruta_valida};
+
+    #[test]
+    fn salida_de_cat_file_batch() {
+        let s = b"abc blob 5\nhola\n\nx:no missing\ndef tree 3\nabc\nghi blob 0\n\n";
+        assert_eq!(
+            partir_batch(s, 4),
+            Some(vec![
+                Some("hola\n".to_owned()),
+                None,
+                None,
+                Some(String::new())
+            ])
+        );
+        // Bytes que no casan con el tamano, de mas o de menos: no se entiende.
+        assert_eq!(partir_batch(b"abc blob 9\nhola\n", 1), None);
+        assert_eq!(partir_batch(b"abc blob 2\nhola\n", 1), None);
+        assert_eq!(partir_batch(s, 3), None);
+        assert_eq!(partir_batch(s, 5), None);
+        assert_eq!(partir_batch(b"", 0), Some(vec![]));
+        // Contenido multibyte: el tamano es en bytes.
+        let t = "a ñ".as_bytes();
+        let mut m = format!("abc blob {}\n", t.len()).into_bytes();
+        m.extend_from_slice(t);
+        m.push(b'\n');
+        assert_eq!(partir_batch(&m, 1), Some(vec![Some("a ñ".to_owned())]));
+    }
 
     #[test]
     fn rutas_de_la_rama_de_estado() {
diff --git a/crates/tc2/src/git/mod.rs b/crates/tc2/src/git/mod.rs
index 0b96295..5a54a15 100644
--- a/crates/tc2/src/git/mod.rs
+++ b/crates/tc2/src/git/mod.rs
@@ -21,8 +21,10 @@ mod worktree;
 pub use diff::diff_tarea;
 pub use estado::Cambio;
 pub use guardar::guardar;
-pub use puerto::{ErrorGit, Git, PREFIJO_REFS_PROPIAS, SalidaGit, Subcomando};
-pub use rama::{comprobar_id, crear_rama_y_worktree, localizar_principal, reanudar, validar_base};
+pub use puerto::{ErrorGit, Git, PREFIJO_REFS_PROPIAS, SalidaCruda, SalidaGit, Subcomando};
+pub use rama::{
+    comprobar_id, crear_rama_y_worktree, ids_con_rama, localizar_principal, reanudar, validar_base,
+};
 pub use remoto::{
     ESPERAS_PUSH, FalloRemoto, actualizar_principal, actualizar_rama, clasificar_fallo_remoto,
     fetch, push,
diff --git a/crates/tc2/src/git/puerto.rs b/crates/tc2/src/git/puerto.rs
index 92d8790..14a7735 100644
--- a/crates/tc2/src/git/puerto.rs
+++ b/crates/tc2/src/git/puerto.rs
@@ -8,8 +8,9 @@
 
 use std::cell::RefCell;
 use std::ffi::OsString;
+use std::io::Write as _;
 use std::path::Path;
-use std::process::Command;
+use std::process::{Command, Stdio};
 
 use crate::i18n::Mensaje;
 use crate::i18n::catalogo::git as txt;
@@ -23,6 +24,8 @@ pub enum Subcomando {
     Add,
     /// `git branch`
     Branch,
+    /// `git cat-file` (lectura; `--batch` con la lista por stdin, F2-4b)
+    CatFile,
     /// `git commit`
     Commit,
     /// `git commit-tree` (crea un objeto commit; no mueve ninguna rama)
@@ -75,6 +78,7 @@ impl Subcomando {
         match self {
             Self::Add => "add",
             Self::Branch => "branch",
+            Self::CatFile => "cat-file",
             Self::Commit => "commit",
             Self::CommitTree => "commit-tree",
             Self::Diff => "diff",
@@ -139,6 +143,18 @@ impl SalidaGit {
     }
 }
 
+/// Lo que devolvio una llamada con entrada por stdin: stdout en bytes, sin
+/// convertir (los tamanos de `cat-file --batch` son en bytes).
+#[derive(Debug, Clone)]
+pub struct SalidaCruda {
+    /// Codigo de salida (`-1` si el proceso murio por una senal).
+    pub codigo: i32,
+    /// Salida estandar, en bytes.
+    pub stdout: Vec<u8>,
+    /// Salida de error, tal cual.
+    pub stderr: String,
+}
+
 /// La llamada no llego a ejecutarse.
 #[derive(Debug, Clone, PartialEq, Eq)]
 pub enum ErrorGit {
@@ -263,14 +279,56 @@ impl Git {
         self.lanzar(dir, sub, args, Some(indice))
     }
 
-    fn lanzar(
+    /// Lanza `git cat-file <args>` en `dir` con `entrada` por stdin (lectura
+    /// en lote, F2-4b). Solo `cat-file`: es el unico que lo necesita y no
+    /// escribe nada.
+    pub fn ejecutar_con_entrada(
         &self,
         dir: &Path,
         sub: Subcomando,
         args: &[&str],
-        indice: Option<&Path>,
-    ) -> Result<SalidaGit, ErrorGit> {
+        entrada: &[u8],
+    ) -> Result<SalidaCruda, ErrorGit> {
+        if sub != Subcomando::CatFile {
+            return Err(ErrorGit::Prohibido(
+                Mensaje::nuevo(&txt::SIN_ENTRADA).con("sub", sub.nombre()),
+            ));
+        }
         comprobar_argumentos(sub, args)?;
+        let mut cmd = self.comando(dir, sub, args, None);
+        cmd.stdin(Stdio::piped())
+            .stdout(Stdio::piped())
+            .stderr(Stdio::piped());
+        let mut hijo = cmd.spawn().map_err(|e| ErrorGit::Lanzar(e.to_string()))?;
+        let stdin = hijo.stdin.take();
+        // La entrada se escribe en otro hilo mientras se lee la salida: si
+        // no, con muchas rutas se llenarian las dos tuberias y se bloquearia.
+        // Un fallo al escribir (git ya termino) se ve en su codigo y stderr.
+        let salida = std::thread::scope(|s| {
+            s.spawn(move || {
+                if let Some(mut e) = stdin {
+                    let _ = e.write_all(entrada);
+                }
+            });
+            hijo.wait_with_output()
+        })
+        .map_err(|e| ErrorGit::Lanzar(e.to_string()))?;
+        Ok(SalidaCruda {
+            codigo: salida.status.code().unwrap_or(-1),
+            stdout: salida.stdout,
+            stderr: String::from_utf8_lossy(&salida.stderr).into_owned(),
+        })
+    }
+
+    /// `git -c ... <sub> <args>` en `dir` con el entorno fijo, y su linea en
+    /// el registro.
+    fn comando(
+        &self,
+        dir: &Path,
+        sub: Subcomando,
+        args: &[&str],
+        indice: Option<&Path>,
+    ) -> Command {
         let mut cmd = Command::new(&self.binario);
         cmd.current_dir(dir)
             .args(["-c", "color.ui=never", "-c", "core.quotepath=false"])
@@ -291,7 +349,21 @@ impl Git {
         self.registro
             .borrow_mut()
             .push(format!("git {} {}", sub.nombre(), args.join(" ")));
-        let salida = cmd.output().map_err(|e| ErrorGit::Lanzar(e.to_string()))?;
+        cmd
+    }
+
+    fn lanzar(
+        &self,
+        dir: &Path,
+        sub: Subcomando,
+        args: &[&str],
+        indice: Option<&Path>,
+    ) -> Result<SalidaGit, ErrorGit> {
+        comprobar_argumentos(sub, args)?;
+        let salida = self
+            .comando(dir, sub, args, indice)
+            .output()
+            .map_err(|e| ErrorGit::Lanzar(e.to_string()))?;
         Ok(SalidaGit {
             codigo: salida.status.code().unwrap_or(-1),
             stdout: String::from_utf8_lossy(&salida.stdout).into_owned(),
@@ -518,6 +590,13 @@ mod tests {
             let r = git.ejecutar(Path::new("."), Subcomando::UpdateRef, ok);
             assert!(matches!(r, Err(ErrorGit::Lanzar(_))), "{ok:?}: {r:?}");
         }
+        // Entrada por stdin: solo `cat-file`.
+        for sub in [Subcomando::UpdateRef, Subcomando::Push, Subcomando::Show] {
+            let r = git.ejecutar_con_entrada(Path::new("."), sub, &["--stdin"], b"x\n");
+            assert!(matches!(r, Err(ErrorGit::Prohibido(_))), "{sub:?}: {r:?}");
+        }
+        let r = git.ejecutar_con_entrada(Path::new("."), Subcomando::CatFile, &["--batch"], b"");
+        assert!(matches!(r, Err(ErrorGit::Lanzar(_))), "{r:?}");
         // Tras `--` en push no hay rutas: se sigue mirando.
         let r = git.ejecutar(Path::new("."), Subcomando::Push, &["origin", "--", "+r:r"]);
         assert!(matches!(r, Err(ErrorGit::Prohibido(_))));
diff --git a/crates/tc2/src/git/rama.rs b/crates/tc2/src/git/rama.rs
index d07cfea..4805866 100644
--- a/crates/tc2/src/git/rama.rs
+++ b/crates/tc2/src/git/rama.rs
@@ -1,8 +1,11 @@
 //! Operaciones de rama y worktree de tarea: `localizar-principal`,
 //! `validar-base`, `comprobar-id`, `crear-rama-y-worktree` y `reanudar`.
 
+use std::collections::BTreeSet;
 use std::path::{Path, PathBuf};
 
+use super::estado::hay_remoto;
+use super::remoto::{FalloRemoto, clasificar_fallo_remoto};
 use super::{
     CARPETA_WORKTREES, Git, Paso, REMOTO, Subcomando, commit_de, correr, es_id, exigir, misma_ruta,
     nombre_de_rama, worktree_relativo, worktrees,
@@ -87,8 +90,13 @@ fn ramas_del_id(git: &Git, dir: &Path, op: &str, id: &str) -> Paso<Vec<String>>
         Subcomando::Branch,
         &["--list", "--format=%(refname:short)", &patron],
     )?;
+    Ok(nombres(&remoto.stdout, &local))
+}
+
+/// Nombres de rama de la salida de `ls-remote --heads` y de `branch --list
+/// --format=%(refname:short)`, ordenados y sin repetir.
+fn nombres(remoto: &str, local: &str) -> Vec<String> {
     let mut ramas: Vec<String> = remoto
-        .stdout
         .lines()
         .filter_map(|l| l.split('\t').nth(1))
         .filter_map(|r| r.strip_prefix("refs/heads/"))
@@ -97,11 +105,74 @@ fn ramas_del_id(git: &Git, dir: &Path, op: &str, id: &str) -> Paso<Vec<String>>
         .collect();
     ramas.sort();
     ramas.dedup();
-    Ok(ramas)
+    ramas
+}
+
+/// Patron de las ramas de tarea (ADR F1-3: `<tipo>/task-nnn-<slug>`).
+/// `ls-remote` lo casa por sufijo (tambien `refs/heads/task-1-x`, sin tipo) y
+/// `branch --list` contra el nombre entero: lo que cuenta lo decide
+/// [`id_de_rama`], igual para los dos.
+const PATRON_TAREAS: &str = "*/task-*";
+
+/// Numero de tarea de una rama: el primer componente, salvo el primero (el
+/// tipo), de la forma `task-<cifras>` o `task-<cifras>-<resto>`.
+fn id_de_rama(rama: &str) -> Option<u32> {
+    rama.split('/').skip(1).find_map(|c| {
+        let resto = c.strip_prefix("task-")?;
+        let fin = resto
+            .find(|ch: char| !ch.is_ascii_digit())
+            .unwrap_or(resto.len());
+        let (cifras, tras) = resto.split_at(fin);
+        (!cifras.is_empty() && (tras.is_empty() || tras.starts_with('-')))
+            .then(|| cifras.parse().ok())
+            .flatten()
+    })
+}
+
+/// IDs que ya tienen rama de tarea (`*/task-nnn-*`) en local o, si hay
+/// `origin`, en el remoto (F2-4b): el alta no nombra una rama que ya existe.
+/// Un fallo del remoto se clasifica como en `fetch`: `sinred` (4),
+/// `denegada` (8) o `error` local.
+pub fn ids_con_rama(git: &Git, dir: &Path, op: &str) -> Paso<BTreeSet<u32>> {
+    let principal = localizar_principal(git, dir)?;
+    let remoto = if hay_remoto(git, &principal, op)? {
+        let s = correr(
+            git,
+            &principal,
+            op,
+            Subcomando::LsRemote,
+            &["--heads", REMOTO, PATRON_TAREAS],
+        )?;
+        if !s.exito() {
+            return Err(match clasificar_fallo_remoto(&s.stderr) {
+                FalloRemoto::SinRed | FalloRemoto::RamaInexistente => {
+                    Informe::nuevo(op, Resultado::Sinred)
+                        .con("remoto", REMOTO)
+                        .diagnostico(super::remoto::sin_red_remoto(op))
+                }
+                FalloRemoto::Local => super::remoto::error_local(op, &s.primera_linea_error()),
+                otro => super::remoto::denegada(op, otro, &s.primera_linea_error()),
+            });
+        }
+        s.stdout
+    } else {
+        String::new()
+    };
+    let local = exigir(
+        git,
+        &principal,
+        op,
+        Subcomando::Branch,
+        &["--list", "--format=%(refname:short)", PATRON_TAREAS],
+    )?;
+    Ok(nombres(&remoto, &local)
+        .iter()
+        .filter_map(|r| id_de_rama(r))
+        .collect())
 }
 
 /// Valida `id`, `tipo` y `slug` y devuelve el nombre de rama.
-fn rama_valida(op: &str, id: &str, tipo: &str, slug: &str) -> Paso<String> {
+pub(crate) fn rama_valida(op: &str, id: &str, tipo: &str, slug: &str) -> Paso<String> {
     let tipo_ok = !tipo.is_empty() && tipo.bytes().all(|b| b.is_ascii_lowercase());
     let slug_ok = !slug.is_empty()
         && !slug.starts_with('-')
@@ -333,3 +404,35 @@ pub(crate) fn renombrar(mut i: Informe, op: &str) -> Informe {
     op.clone_into(&mut i.comando);
     i
 }
+
+#[cfg(test)]
+mod tests {
+    use super::{id_de_rama, nombres};
+
+    /// Lo que devuelve `ls-remote` por sufijo (comprobado con git 2.56):
+    /// cuenta un componente `task-<cifras>` que no sea el primero.
+    #[test]
+    fn id_de_las_ramas_de_tarea() {
+        for (rama, id) in [
+            ("feature/task-005-y", Some(5)),
+            ("a/b/task-1-x", Some(1)),
+            ("feature/task-10", Some(10)),
+            ("feature/task-12-s/sub", Some(12)),
+            ("fix/task-0042-x", Some(42)),
+            ("task-7-raiz", None),
+            ("feature/xtask-9-z", None),
+            ("feature/task-x", None),
+            ("feature/task-9x", None),
+            ("feature/task--1", None),
+            ("fix/TASK-11-m", None),
+            ("feature/task-99999999999-x", None),
+        ] {
+            assert_eq!(id_de_rama(rama), id, "{rama}");
+        }
+        let n = nombres(
+            "abc\trefs/heads/feature/task-005-y\nabc\trefs/heads/task-7-raiz\n",
+            "feature/task-005-y\n  \nfix/task-3-z\n",
+        );
+        assert_eq!(n, ["feature/task-005-y", "fix/task-3-z", "task-7-raiz"]);
+    }
+}
diff --git a/crates/tc2/src/i18n/catalogo/estado.rs b/crates/tc2/src/i18n/catalogo/estado.rs
index 2689b53..6b808ef 100644
--- a/crates/tc2/src/i18n/catalogo/estado.rs
+++ b/crates/tc2/src/i18n/catalogo/estado.rs
@@ -89,4 +89,20 @@ crate::i18n::textos! {
         es: "no quedan IDs de tarea",
         en: "no task IDs left",
     }
+    AVISO_PARECIDAS {
+        es: "aviso: ya hay tareas abiertas con un titulo igual o parecido: {lista}",
+        en: "warning: there are already open tasks with the same or a similar title: {lista}",
+    }
+    PARECIDA_DUPLICADO {
+        es: "{id} \"{titulo}\" (duplicado)",
+        en: "{id} \"{titulo}\" (duplicate)",
+    }
+    PARECIDA_CASI {
+        es: "{id} \"{titulo}\" (parecido {unidad},{centesimas})",
+        en: "{id} \"{titulo}\" (similarity {unidad}.{centesimas})",
+    }
+    RAMA_DERIVADA_INVALIDA {
+        es: "la rama derivada de {id} no es valida ({motivo})",
+        en: "the branch derived from {id} is not valid ({motivo})",
+    }
 }
diff --git a/crates/tc2/src/i18n/catalogo/git.rs b/crates/tc2/src/i18n/catalogo/git.rs
index b2ef392..21ab19f 100644
--- a/crates/tc2/src/i18n/catalogo/git.rs
+++ b/crates/tc2/src/i18n/catalogo/git.rs
@@ -6,6 +6,14 @@ crate::i18n::textos! {
         es: "argumento prohibido para la capa: {argumento}",
         en: "argument not allowed for the layer: {argumento}",
     }
+    SIN_ENTRADA {
+        es: "argumento prohibido para la capa: {sub} con entrada por stdin (solo cat-file)",
+        en: "argument not allowed for the layer: {sub} with stdin input (only cat-file)",
+    }
+    CAT_FILE_INCOMPLETO {
+        es: "git cat-file --batch devolvio una salida que no se entiende",
+        en: "git cat-file --batch returned output that cannot be parsed",
+    }
     SIN_INDICE_TEMPORAL {
         es: "argumento prohibido para la capa: {sub} sin indice temporal",
         en: "argument not allowed for the layer: {sub} without a temporary index",
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 7ff52e7..14b8c7e 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -85,7 +85,17 @@ where
                 [j, r @ ..] if j == "--json" => (true, r),
                 r => (false, r),
             };
-            orden_new(resto).salida_en(json, idioma)
+            let mut parecidas = Vec::new();
+            let mut s = orden_new(resto, &mut parecidas).salida_en(json, idioma);
+            // El aviso de titulos parecidos va a stderr con codigo 0: es la
+            // unica excepcion a «diagnostico solo si falla» (F2-4b, D1).
+            if s.codigo == 0
+                && let Some(a) = alta::aviso_parecidas(&parecidas, idioma)
+            {
+                s.stderr.push_str(&a);
+                s.stderr.push('\n');
+            }
+            s
         }
         [orden, resto @ ..] if orden == version::VERSION => {
             let (json, resto) = con_json(resto);
@@ -197,8 +207,9 @@ fn catalogo(anotacion: &str) -> Option<Box<dyn estado::Operacion>> {
 }
 
 /// `new "<titulo>"`: alta de una tarea con su ID reservado por el push de la
-/// rama de estado (F2-4). Sale `new ok id=TASK-NNN ...`.
-fn orden_new(args: &[String]) -> Informe {
+/// rama de estado (F2-4). Sale `new ok id=TASK-NNN rama=... [parecidas=...]`;
+/// las tareas abiertas parecidas quedan ademas en `parecidas` para el aviso.
+fn orden_new(args: &[String], parecidas: &mut Vec<alta::Parecida>) -> Informe {
     let titulo = match args {
         [t] if alta::titulo_valido(t) => t,
         [_] => {
@@ -225,15 +236,35 @@ fn orden_new(args: &[String]) -> Informe {
         }
     };
     let g = git::Git::nuevo();
-    let op = alta::Alta::nueva(titulo);
+    // Las ramas `*/task-*` se leen una vez, antes de la transaccion: el ID
+    // nuevo queda por encima de todas (F2-4b). Sin red no se reserva nada.
+    let ocupados = match git::ids_con_rama(&g, &dir, alta::NOMBRE) {
+        Ok(o) => o,
+        Err(mut i) => {
+            alta::NOMBRE.clone_into(&mut i.comando);
+            if i.resultado == Resultado::Sinred {
+                i = i.con("pendiente", "no");
+            }
+            return i;
+        }
+    };
+    let op = alta::Alta::nueva(titulo).con_ocupados(ocupados);
     let c = estado::Contexto::nuevo(&g, &dir, &catalogo);
     let mut i = estado::transaccion(&c, &op);
     alta::NOMBRE.clone_into(&mut i.comando);
     if i.resultado == Resultado::Ok
-        && let Some(id) = op.id()
+        && let (Some(id), Some(rama)) = (op.id(), op.rama())
     {
         i.claves
             .insert(0, ("id".to_owned(), contrato::Valor::Texto(id)));
+        i.claves
+            .insert(1, ("rama".to_owned(), contrato::Valor::Texto(rama)));
+        *parecidas = op.parecidas();
+        if !parecidas.is_empty() {
+            let ids = parecidas.iter().map(|p| p.id.clone()).collect();
+            i.claves
+                .insert(2, ("parecidas".to_owned(), contrato::Valor::Lista(ids)));
+        }
     }
     i
 }
diff --git a/crates/tc2/src/tarea/mod.rs b/crates/tc2/src/tarea/mod.rs
index 4c32c41..39a1402 100644
--- a/crates/tc2/src/tarea/mod.rs
+++ b/crates/tc2/src/tarea/mod.rs
@@ -12,6 +12,7 @@
 
 mod cuerpo;
 mod modelo;
+pub mod texto;
 mod yaml;
 
 use std::fmt;
diff --git a/crates/tc2/src/tarea/modelo.rs b/crates/tc2/src/tarea/modelo.rs
index 7c2f2f0..db129a0 100644
--- a/crates/tc2/src/tarea/modelo.rs
+++ b/crates/tc2/src/tarea/modelo.rs
@@ -317,7 +317,7 @@ impl Tarea {
             estado: "planificada".to_owned(),
             motivo: None,
             plan_aprobado: false,
-            // La deriva del ID F2-4b; hasta entonces, `null`.
+            // La pone el alta (F2-4b) derivada del ID y del titulo.
             rama: None,
             asignado_a: None,
             roles: Roles {
diff --git a/crates/tc2/src/tarea/texto.rs b/crates/tc2/src/tarea/texto.rs
new file mode 100644
index 0000000..76f1a91
--- /dev/null
+++ b/crates/tc2/src/tarea/texto.rs
@@ -0,0 +1,267 @@
+//! Texto de los titulos (F2-4b, RR-12): plegado, slug de la rama y parecido
+//! entre titulos. Hoja pura, sin crates: un solo plegado sirve al slug y a la
+//! comparacion.
+//!
+//! - [`plegar`]: minusculas, sin tildes (tabla fija de Latin-1 y Latin
+//!   Extended-A), lo no alfanumerico a espacio y espacios colapsados. Otros
+//!   alfabetos quedan en minuscula: se comparan, pero no entran en el slug.
+//! - [`slug`]: ASCII `[a-z0-9-]`, como mucho [`SLUG_MAX`] bytes cortando en
+//!   guion; vacio → `tarea`.
+//! - [`parecido`]: plegados iguales → duplicado; si no, coeficiente de Dice
+//!   sobre los bigramas de caracteres del plegado sin espacios, en
+//!   centesimas; desde [`UMBRAL`] cuenta como parecido.
+
+/// Centesimas de Dice desde las que dos titulos son «casi iguales» (D2 de
+/// TASK-076: constante hasta que F9-4 mida el ruido).
+pub const UMBRAL: u32 = 80;
+
+/// Longitud maxima del slug, en bytes.
+pub const SLUG_MAX: usize = 40;
+
+/// Slug de un titulo sin nada que valga en ASCII.
+pub const SLUG_VACIO: &str = "tarea";
+
+/// Como se parecen dos titulos.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Parecido {
+    /// Plegados iguales.
+    Duplicado,
+    /// Dice >= [`UMBRAL`], en centesimas (truncadas: 0,866 → 86).
+    Casi(u32),
+}
+
+/// Equivalente sin tilde de una letra minuscula de Latin-1 o Latin
+/// Extended-A; `None` si no es una de ellas.
+fn sin_tilde(c: char) -> Option<&'static str> {
+    Some(match c {
+        'ª' | 'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => "a",
+        'æ' => "ae",
+        'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' => "c",
+        'ð' | 'ď' | 'đ' => "d",
+        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' => "e",
+        'ĝ' | 'ğ' | 'ġ' | 'ģ' => "g",
+        'ĥ' | 'ħ' => "h",
+        'ì' | 'í' | 'î' | 'ï' | 'ĩ' | 'ī' | 'ĭ' | 'į' | 'ı' => "i",
+        'ĳ' => "ij",
+        'ĵ' => "j",
+        'ķ' | 'ĸ' => "k",
+        'ĺ' | 'ļ' | 'ľ' | 'ŀ' | 'ł' => "l",
+        'ñ' | 'ń' | 'ņ' | 'ň' | 'ŉ' | 'ŋ' => "n",
+        'º' | 'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ŏ' | 'ő' => "o",
+        'œ' => "oe",
+        'ŕ' | 'ŗ' | 'ř' => "r",
+        'ś' | 'ŝ' | 'ş' | 'š' | 'ſ' => "s",
+        'ß' => "ss",
+        'ţ' | 'ť' | 'ŧ' => "t",
+        'þ' => "th",
+        'ù' | 'ú' | 'û' | 'ü' | 'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' => "u",
+        'ŵ' => "w",
+        'ý' | 'ÿ' | 'ŷ' => "y",
+        'ź' | 'ż' | 'ž' => "z",
+        _ => return None,
+    })
+}
+
+/// Marcas combinantes (U+0300..U+036F): una tilde escrita aparte (`e` +
+/// U+0301) o la que deja `İ` al pasar a minuscula se quitan.
+fn combinante(c: char) -> bool {
+    ('\u{300}'..='\u{36f}').contains(&c)
+}
+
+/// Titulo plegado: minusculas, sin tildes, lo no alfanumerico a un espacio,
+/// sin espacios al principio ni al final ni repetidos.
+pub fn plegar(titulo: &str) -> String {
+    let mut s = String::with_capacity(titulo.len());
+    let mut espacio = false;
+    for c in titulo.chars().flat_map(char::to_lowercase) {
+        if combinante(c) {
+            continue;
+        }
+        if let Some(t) = sin_tilde(c) {
+            if espacio && !s.is_empty() {
+                s.push(' ');
+            }
+            espacio = false;
+            s.push_str(t);
+        } else if c.is_alphanumeric() {
+            if espacio && !s.is_empty() {
+                s.push(' ');
+            }
+            espacio = false;
+            s.push(c);
+        } else {
+            espacio = true;
+        }
+    }
+    s
+}
+
+/// Slug de la rama (ADR F1-3): el plegado con solo `[a-z0-9]` y guiones
+/// entre medias, como mucho [`SLUG_MAX`] bytes cortando en el ultimo guion
+/// que quepa; si no queda nada, [`SLUG_VACIO`].
+pub fn slug(titulo: &str) -> String {
+    let mut s = String::new();
+    let mut guion = false;
+    for c in plegar(titulo).chars() {
+        if c.is_ascii_lowercase() || c.is_ascii_digit() {
+            if guion && !s.is_empty() {
+                s.push('-');
+            }
+            guion = false;
+            s.push(c);
+        } else {
+            guion = true;
+        }
+    }
+    if s.len() > SLUG_MAX {
+        // Solo hay ASCII: cortar por bytes no parte ningun caracter.
+        let corte = match s.get(..=SLUG_MAX).and_then(|p| p.rfind('-')) {
+            Some(i) if i > 0 => i,
+            _ => SLUG_MAX,
+        };
+        s.truncate(corte);
+        while s.ends_with('-') {
+            s.pop();
+        }
+    }
+    if s.is_empty() {
+        SLUG_VACIO.to_owned()
+    } else {
+        s
+    }
+}
+
+/// Bigramas de caracteres de `plegado` sin espacios, ordenados (multiconjunto).
+fn bigramas(plegado: &str) -> Vec<(char, char)> {
+    let c: Vec<char> = plegado.chars().filter(|c| *c != ' ').collect();
+    let mut v: Vec<(char, char)> = c.windows(2).map(|w| (w[0], w[1])).collect();
+    v.sort_unstable();
+    v
+}
+
+/// Coeficiente de Dice entre dos plegados, en centesimas truncadas:
+/// `2·|A∩B| / (|A|+|B|)` sobre los multiconjuntos de bigramas. Truncar hace
+/// que `centesimas >= UMBRAL` sea exactamente `Dice >= 0,80`.
+fn dice(a: &str, b: &str) -> u32 {
+    let (x, y) = (bigramas(a), bigramas(b));
+    let total = x.len() + y.len();
+    if total == 0 {
+        return 0;
+    }
+    let (mut i, mut j, mut comunes) = (0, 0, 0usize);
+    while let (Some(p), Some(q)) = (x.get(i), y.get(j)) {
+        match p.cmp(q) {
+            std::cmp::Ordering::Less => i += 1,
+            std::cmp::Ordering::Greater => j += 1,
+            std::cmp::Ordering::Equal => {
+                comunes += 1;
+                i += 1;
+                j += 1;
+            }
+        }
+    }
+    u32::try_from(200 * comunes / total).unwrap_or(0)
+}
+
+/// Si dos titulos son el mismo o casi: `None` si no se parecen.
+pub fn parecido(a: &str, b: &str) -> Option<Parecido> {
+    let (pa, pb) = (plegar(a), plegar(b));
+    if pa == pb {
+        return Some(Parecido::Duplicado);
+    }
+    let d = dice(&pa, &pb);
+    (d >= UMBRAL).then_some(Parecido::Casi(d))
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn plegado_sin_tildes_ni_signos() {
+        for (t, p) in [
+            ("  Exportar   INFORMES!! ", "exportar informes"),
+            ("Añadir «búsqueda» — rápida", "anadir busqueda rapida"),
+            ("Ça, Æsir; Œuvre; Straße", "ca aesir oeuvre strasse"),
+            ("Łódź Đakovo İstanbul", "lodz dakovo istanbul"),
+            ("cafe\u{301} con e\u{301}", "cafe con e"),
+            ("Задача №5", "задача 5"),
+            ("漢字", "漢字"),
+            ("---", ""),
+        ] {
+            assert_eq!(plegar(t), p, "{t}");
+        }
+    }
+
+    /// C7: el slug es ASCII `[a-z0-9-]`, corta en guion y un titulo sin
+    /// letras latinas da `tarea`.
+    #[test]
+    fn slug_ascii_cortado_en_guion_y_vacio_es_tarea() {
+        for (t, s) in [
+            (
+                "Nombres únicos de tarea y rama",
+                "nombres-unicos-de-tarea-y-rama",
+            ),
+            ("C:\\ruta\\ \"x\" {fecha}", "c-ruta-x-fecha"),
+            ("Задача", "tarea"),
+            ("漢字 — !!!", "tarea"),
+            ("", "tarea"),
+            ("Café 日本 v2", "cafe-v2"),
+            (
+                "uno dos tres cuatro cinco seis siete ocho nueve diez",
+                "uno-dos-tres-cuatro-cinco-seis-siete",
+            ),
+            (&"a".repeat(50), &"a".repeat(40)),
+            (
+                "abcdefghijklmnopqrstuvwxyzabcdefghijklmn-z",
+                "abcdefghijklmnopqrstuvwxyzabcdefghijklmn",
+            ),
+        ] {
+            let r = slug(t);
+            assert_eq!(r, s, "{t}");
+            assert!(r.len() <= SLUG_MAX);
+            assert!(!r.starts_with('-') && !r.ends_with('-'), "{r}");
+            assert!(
+                r.bytes()
+                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'),
+                "{r}"
+            );
+        }
+    }
+
+    /// C5: tabla del umbral de «casi igual» (Dice >= 0,80 sobre bigramas).
+    #[test]
+    fn umbral_de_casi_igual() {
+        for (a, b, esperado) in [
+            (
+                "Exportar informes",
+                "Exprotar informes",
+                Some(Parecido::Casi(80)),
+            ),
+            ("Login con Google", "Login con GitHub", None),
+            (
+                "Exportar informes",
+                "EXPORTAR  informes.",
+                Some(Parecido::Duplicado),
+            ),
+            (
+                "Añadir búsqueda",
+                "anadir busqueda",
+                Some(Parecido::Duplicado),
+            ),
+            (
+                "Exportar informes",
+                "Exportar informes PDF",
+                Some(Parecido::Casi(90)),
+            ),
+            ("Exportar informes", "Importar facturas", None),
+            ("a", "b", None),
+            ("", "", Some(Parecido::Duplicado)),
+        ] {
+            assert_eq!(parecido(a, b), esperado, "{a} / {b}");
+            assert_eq!(parecido(b, a), esperado, "simetrico: {a} / {b}");
+        }
+        assert_eq!(dice("exportar informes", "exprotar informes"), 80);
+        assert_eq!(dice("login con google", "login con github"), 61);
+    }
+}
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-076/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-076/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-076/tarea.md                                    | 14 +++++++++++++-
 3 files changed, 13 insertions(+), 1 deletion(-)
````

## Tests del diff (4 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/alta.rs" ":(literal)crates/tc2/tests/comun/mod.rs" ":(literal)crates/tc2/tests/estado_casos.rs" ":(literal)crates/tc2/tests/tarea.rs"
````

````
crates/tc2/tests/alta.rs         | 284 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-
 crates/tc2/tests/comun/mod.rs    |   3 +-
 crates/tc2/tests/estado_casos.rs |  44 +++++++++++++++++++++++++
 crates/tc2/tests/tarea.rs        |   6 +++-
 4 files changed, 334 insertions(+), 3 deletions(-)
````
