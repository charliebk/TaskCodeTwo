# Peticion de revision — TASK-031 (ronda 2)

- Tarea: TASK-031 — F3-4 tc2 roadmap: la vista para todos
- Rama revisada: feature/task-031-f3-4-roadmap-la-vista-para-todos
- Rama base: develop
- Commit revisado (HEAD): 48d4390b5a72a8a4311ad4007f5b77284bb0300e
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-031 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 8a54049c717b71525d0a180c8e00676686271db0 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (10 hallazgo(s)))

## Instrucciones para el agente revisor (verificacion acotada)

Eres un revisor INDEPENDIENTE del agente que implemento. La rama ya paso una
revision completa: esta ronda verifica las correcciones de la ronda anterior,
no vuelve a revisar la rama entera ni los MENORES de la ronda anterior (tu
tabla solo trae CRITICOS e IMPORTANTES). Haz esto, y nada mas:

1. Comprueba, uno por uno, que cada hallazgo pendiente de tu tabla de abajo
   queda resuelto por el diff de esta ronda (los marcados `corregido` tambien:
   verificalo, no lo des por bueno). Para un CRITICO o IMPORTANTE, reproduce el
   fallo original y comprueba que ya no ocurre; puedes leer el repo entero si la
   causa no esta en el diff.
2. Ejecuta la suite (o el comando de verificacion del proyecto) sobre el commit
   revisado y di el resultado: verificar sin ejecutar no vale. Si esta peticion
   ya trae la verificacion de la ronda en verde para ese mismo commit, no la
   repitas: cita ese resultado y corre solo los tests de lo que verificas.
3. Busca en el diff de esta ronda el fallo que la correccion abre: es justo
   donde se cuelan los fallos nuevos. Si encuentras uno, construye el caso que
   lo rompe antes de reportarlo.

Solo abre un hallazgo NUEVO si es CRITICO o si lo introduce la propia correccion
(algo que el diff de esta ronda rompe). Lo demas que veas fuera de eso va como
nota al final del informe, no en la tabla: no abre otra ronda.

Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. En la tabla, cada hallazgo pendiente que ya este
resuelto va como `corregido`; el que no, `abierto`. Vuelca tu salida en el
informe de esta ronda (informe-revision-2-rust-wasm-reviewer.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-031/tarea.md
- tareas/03-en-revision/TASK-031/planificacion/
- crates/tc2/src/i18n/catalogo/roadmap.rs
- crates/tc2/src/roadmap/mod.rs
- crates/tc2/src/roadmap/zona.rs
- crates/tc2/src/siguiente.rs
- crates/tc2/src/situacion.rs
- crates/tc2/tests/roadmap.rs

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 2 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | abierto | crates/tc2/src/roadmap/mod.rs:112 | informe-revision-1-rust-wasm-reviewer.md |

## Commits a revisar (git log 8a54049c717b71525d0a180c8e00676686271db0..HEAD)

````
48d4390 fix(TASK-031): escribir no pisa una zona manual publicada por otro
e0cc487 fix(TASK-031): sin zona publicada los hitos salen de la zona por defecto
154c1e5 fix(TASK-031): marcas completas de la zona manual y H0 por defecto
7d70f1f chore(TASK-031): veredictos y coste de la revision ronda 1
74257cd chore(TASK-031): informes de revision ronda 1
9401218 chore(TASK-031): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 8a54049c717b71525d0a180c8e00676686271db0..HEAD)

````diff
diff --git a/crates/tc2/src/i18n/catalogo/roadmap.rs b/crates/tc2/src/i18n/catalogo/roadmap.rs
index fb3ceff..2355473 100644
--- a/crates/tc2/src/i18n/catalogo/roadmap.rs
+++ b/crates/tc2/src/i18n/catalogo/roadmap.rs
@@ -9,6 +9,22 @@ crate::i18n::textos! {
         es: "el ROADMAP.md publicado no esta al dia: ejecuta tc2 roadmap --escribir",
         en: "the published ROADMAP.md is out of date: run tc2 roadmap --escribir",
     }
+    ZONA_DIVERGENTE {
+        es: "la zona manual publicada cambio desde que editaste {donde}: no se ha publicado nada y tu edicion sigue en la carpeta; guarda una copia, descarta el cambio, trae lo publicado con tc2 roadmap --escribir y vuelve a aplicar tu edicion a mano",
+        en: "the published manual zone changed since you edited {donde}: nothing was published and your edit is still in the folder; save a copy, discard the change, bring in what was published with tc2 roadmap --escribir and reapply your edit by hand",
+    }
+    NO_SE_RESTAURO {
+        es: "no se pudo dejar {donde} como estaba en el indice: {error}; revisa git status en tareas/ antes de seguir",
+        en: "could not put {donde} back as it was in the index: {error}; check git status in tareas/ before going on",
+    }
+    AVISO_FUERA_DE_ZONA {
+        es: "aviso: {donde} tiene cambios fuera de la zona manual; esa parte la genera tc2 y no se publica: descartalos",
+        en: "warning: {donde} has changes outside the manual zone; tc2 generates that part and it is not published: discard them",
+    }
+    AVISO_PERSONAS_ILEGIBLES {
+        es: "aviso: no se pudo leer la lista de personas de la base; los asignados heredados se muestran tal cual",
+        en: "warning: the list of people in the base could not be read; inherited assignees are shown as they are",
+    }
     ZONA_ROTA {
         es: "la zona manual de {donde} esta rota ({motivo}): deja una sola zona que abra con «<!-- manual:inicio -->» y cierre con «<!-- manual:fin -->», y publicala con tc2 roadmap --escribir",
         en: "the manual zone in {donde} is broken ({motivo}): keep a single zone that opens with «<!-- manual:inicio -->» and closes with «<!-- manual:fin -->», and publish it with tc2 roadmap --escribir",
diff --git a/crates/tc2/src/roadmap/mod.rs b/crates/tc2/src/roadmap/mod.rs
index a1731bd..0500e3e 100644
--- a/crates/tc2/src/roadmap/mod.rs
+++ b/crates/tc2/src/roadmap/mod.rs
@@ -13,9 +13,12 @@
 //!   (`rechazada motivo=desfasado|roadmap-roto`), sin escribir; `--escribir`
 //!   lo publica como una transaccion mas ([`Regenerar`]).
 //! - La zona manual se edita en `tareas/ROADMAP.md` y se publica con
-//!   `--escribir` (D4): si el fichero de la carpeta montada tiene cambios, se
-//!   prepara (`git add`) y su zona es la que se publica, byte a byte; tras
-//!   publicar, la carpeta se adelanta a lo publicado.
+//!   `--escribir` (D4): si la zona de la carpeta montada cambio, se valida, se
+//!   prepara (`git add`) y es la que se publica, byte a byte, salvo que la
+//!   publicada haya cambiado desde la de partida (`zona-divergente`); tras
+//!   publicar, la carpeta se adelanta a lo publicado. Si no se publica, el
+//!   indice vuelve a como estaba. Los cambios fuera de la zona no se publican
+//!   y se avisan.
 
 pub mod generar;
 pub mod mermaid;
@@ -39,7 +42,56 @@ pub const NOMBRE: &str = "roadmap";
 /// Lista de personas de la base para convertir los `asignado_a` heredados
 /// (D6); sin lista, ninguna: se muestran tal cual.
 pub fn personas(g: &Git, dir: &Path) -> Personas {
-    crate::identidad::personas_de_base(g, dir).unwrap_or_default()
+    personas_y_aviso(g, dir).0
+}
+
+/// Como [`personas`], y si hay que avisar: la lista existe pero no se pudo
+/// leer (no avisa si simplemente no hay lista).
+pub fn personas_y_aviso(g: &Git, dir: &Path) -> (Personas, bool) {
+    match crate::identidad::personas_de_base(g, dir) {
+        Ok(p) => (p, false),
+        Err(i) => (
+            Personas::default(),
+            i.valor("motivo") != Some("sin-personas"),
+        ),
+    }
+}
+
+/// Aviso: `tareas/ROADMAP.md` tiene cambios solo fuera de la zona manual.
+const FUERA_DE_ZONA: &str = "fuera-de-zona";
+/// Aviso: la lista de personas existe y no se pudo leer.
+const PERSONAS_ILEGIBLES: &str = "personas-ilegibles";
+
+/// `avisos=` con los avisos que haya (ninguno: sin la clave).
+fn con_avisos(i: Informe, avisos: &[&str]) -> Informe {
+    if avisos.is_empty() {
+        return i;
+    }
+    i.con_lista("avisos", avisos.iter().map(|a| (*a).to_owned()).collect())
+}
+
+/// Los avisos de un informe, en una linea por aviso para stderr.
+fn texto_de_avisos(i: &Informe, idioma: Idioma) -> String {
+    let avisos = i.claves.iter().find_map(|(k, v)| match v {
+        Valor::Lista(l) if k == "avisos" => Some(l.as_slice()),
+        _ => None,
+    });
+    let mut s = String::new();
+    for a in avisos.unwrap_or_default() {
+        let m = match a.as_str() {
+            FUERA_DE_ZONA => Mensaje::nuevo(&txt::AVISO_FUERA_DE_ZONA).con("donde", donde()),
+            PERSONAS_ILEGIBLES => Mensaje::nuevo(&txt::AVISO_PERSONAS_ILEGIBLES),
+            _ => continue,
+        };
+        s.push_str(&m.texto(idioma));
+        s.push('\n');
+    }
+    s
+}
+
+/// `tareas/ROADMAP.md`, para los mensajes.
+fn donde() -> String {
+    format!("{CARPETA}/{ROADMAP}")
 }
 
 /// El generador que recibe cada transaccion de estado.
@@ -73,14 +125,36 @@ fn roto(donde: &str, motivo: &str) -> Fallo {
     )
 }
 
+/// La zona manual de un ROADMAP para comparar ediciones: la zona; sin zona,
+/// ninguna; rota, el texto entero (cualquier cambio cuenta).
+fn zona_de(texto: &str) -> Option<String> {
+    match zona::extraer(texto) {
+        Ok(z) => z.map(str::to_owned),
+        Err(_) => Some(texto.to_owned()),
+    }
+}
+
+/// Lo que la persona edito en `tareas/ROADMAP.md`.
+#[derive(Debug, Clone, PartialEq, Eq)]
+struct Edicion {
+    /// El fichero editado (normalizado como lo guardaria git), con su zona
+    /// valida.
+    texto: String,
+    /// La zona de la que partio: la del `HEAD` de la carpeta montada.
+    base: Option<String>,
+}
+
 /// `roadmap --escribir` como operacion de la rama de estado: no cambia nada
 /// salvo, si la persona edito la zona manual en `tareas/`, el `ROADMAP.md`
 /// con esa zona; el generador de la transaccion hace el resto. Exige red
 /// (D2b de F1-2): sin red no queda pendiente.
+///
+/// Si la zona publicada ya no es la de la que partio la edicion (otra
+/// persona publico la suya entre medias) y tampoco es la editada, se
+/// rechaza con `zona-divergente` en vez de pisarla (IMP-1 de la revision 1).
 #[derive(Debug, Clone, Default)]
 pub struct Regenerar {
-    /// `tareas/ROADMAP.md` preparado por la persona, con su zona valida.
-    editado: Option<String>,
+    editado: Option<Edicion>,
 }
 
 impl Regenerar {
@@ -89,10 +163,11 @@ impl Regenerar {
         Self::default()
     }
 
-    /// Con el `ROADMAP.md` editado por la persona (su zona ya validada).
-    pub fn con_editado(texto: String) -> Self {
+    /// Con el `ROADMAP.md` editado por la persona (su zona ya validada) y la
+    /// zona de la que partio la edicion.
+    pub fn con_editado(texto: String, base: Option<String>) -> Self {
         Self {
-            editado: Some(texto),
+            editado: Some(Edicion { texto, base }),
         }
     }
 }
@@ -110,8 +185,20 @@ impl Operacion for Regenerar {
     fn aplicar(&self, e: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
         let actual = e.leer(ROADMAP)?;
         match &self.editado {
-            Some(t) if actual.as_deref() == Some(t.as_str()) => Ok(Vec::new()),
-            Some(t) => Ok(vec![Cambio::nuevo(ROADMAP, t.clone())]),
+            Some(ed) => {
+                let publicada = actual.as_deref().and_then(zona_de);
+                if publicada != ed.base && publicada != zona_de(&ed.texto) {
+                    return Err(Fallo::rechazada(
+                        "zona-divergente",
+                        Mensaje::nuevo(&txt::ZONA_DIVERGENTE).con("donde", donde()),
+                    ));
+                }
+                if actual.as_deref() == Some(ed.texto.as_str()) {
+                    Ok(Vec::new())
+                } else {
+                    Ok(vec![Cambio::nuevo(ROADMAP, ed.texto.clone())])
+                }
+            }
             None => match actual.as_deref().map(zona::extraer) {
                 Some(Err(r)) => Err(roto(ROADMAP, r.motivo())),
                 _ => Ok(Vec::new()),
@@ -160,22 +247,28 @@ pub fn orden(args: &[String], json: bool, idioma: Idioma, catalogo: Catalogo<'_>
         }
     };
     let g = Git::nuevo();
-    match modo {
+    let i = match modo {
         Modo::Imprimir => match imprimir_en(&g, &dir) {
-            Ok((texto, _)) if !json => Salida {
-                codigo: 0,
-                stdout: texto,
-                stderr: String::new(),
-            },
-            Ok((texto, commit)) => Informe::nuevo(NOMBRE, Resultado::Ok)
-                .con("commit", commit)
-                .con("texto", texto)
-                .salida_en(true, idioma),
-            Err(i) => i.salida_en(json, idioma),
+            Ok((texto, i)) if !json => {
+                return Salida {
+                    codigo: 0,
+                    stdout: texto,
+                    stderr: texto_de_avisos(&i, idioma),
+                };
+            }
+            Ok((texto, i)) => i.con("texto", texto),
+            Err(i) => i,
         },
-        Modo::Comparar => comparar_en(&g, &dir).salida_en(json, idioma),
-        Modo::Escribir => escribir_en(&g, &dir, catalogo).salida_en(json, idioma),
+        Modo::Comparar => comparar_en(&g, &dir),
+        Modo::Escribir => escribir_en(&g, &dir, catalogo),
+    };
+    let mut s = i.salida_en(json, idioma);
+    // Los avisos van a stderr con codigo 0 (como el de titulos parecidos);
+    // si falla, stderr es el diagnostico.
+    if s.codigo == 0 {
+        s.stderr.push_str(&texto_de_avisos(&i, idioma));
     }
+    s
 }
 
 fn renombrado(mut i: Informe) -> Informe {
@@ -203,12 +296,33 @@ fn situacion_en(g: &Git, dir: &Path) -> Result<(Situacion, String), Informe> {
     Ok((s, commit))
 }
 
-/// El ROADMAP que toca sobre el estado que conoce el clon, y el commit de
-/// ese estado. Con la zona rota, `rechazada motivo=roadmap-roto`.
-pub fn imprimir_en(g: &Git, dir: &Path) -> Result<(String, String), Informe> {
+/// Personas de la base y los avisos que tocan.
+fn personas_con_avisos(g: &Git, dir: &Path) -> (Personas, Vec<&'static str>) {
+    let (p, ilegibles) = personas_y_aviso(g, dir);
+    (
+        p,
+        if ilegibles {
+            vec![PERSONAS_ILEGIBLES]
+        } else {
+            Vec::new()
+        },
+    )
+}
+
+/// El ROADMAP que toca sobre el estado que conoce el clon, con el informe
+/// `ok` (su `commit=` y sus `avisos=`). Con la zona rota, `rechazada
+/// motivo=roadmap-roto`.
+pub fn imprimir_en(g: &Git, dir: &Path) -> Result<(String, Informe), Informe> {
     let (s, commit) = situacion_en(g, dir)?;
-    match de_situacion(&s, &personas(g, dir)) {
-        Some(t) => Ok((t, commit)),
+    let (personas, avisos) = personas_con_avisos(g, dir);
+    match de_situacion(&s, &personas) {
+        Some(t) => Ok((
+            t,
+            con_avisos(
+                Informe::nuevo(NOMBRE, Resultado::Ok).con("commit", commit),
+                &avisos,
+            ),
+        )),
         None => Err(zona_rota(&s, &commit)),
     }
 }
@@ -228,11 +342,13 @@ pub fn comparar_en(g: &Git, dir: &Path) -> Informe {
         Ok(x) => x,
         Err(i) => return i,
     };
-    match de_situacion(&s, &personas(g, dir)) {
+    let (personas, avisos) = personas_con_avisos(g, dir);
+    match de_situacion(&s, &personas) {
         None => zona_rota(&s, &commit),
-        Some(t) if s.roadmap.as_deref() == Some(t.as_str()) => {
-            Informe::nuevo(NOMBRE, Resultado::Ok).con("commit", commit)
-        }
+        Some(t) if s.roadmap.as_deref() == Some(t.as_str()) => con_avisos(
+            Informe::nuevo(NOMBRE, Resultado::Ok).con("commit", commit),
+            &avisos,
+        ),
         Some(_) => Informe::nuevo(NOMBRE, Resultado::Rechazada)
             .con("motivo", "desfasado")
             .con("commit", commit)
@@ -249,9 +365,22 @@ fn carpeta_montada(g: &Git, dir: &Path) -> Result<Option<PathBuf>, Informe> {
         .map(|w| w.ruta))
 }
 
-/// Si `tareas/ROADMAP.md` tiene cambios, los prepara y devuelve lo
-/// preparado (normalizado como lo guardaria git), con su zona validada.
-fn editado_en(g: &Git, carpeta: &Path) -> Result<Option<String>, Informe> {
+/// Que hay en `tareas/ROADMAP.md` respecto a su `HEAD`.
+#[derive(Debug, Clone, PartialEq, Eq)]
+enum Lectura {
+    /// Sin cambios.
+    Limpia,
+    /// Cambios solo fuera de la zona manual: no se publican y se avisa.
+    FueraDeZona,
+    /// La zona manual editada.
+    Zona(Edicion),
+}
+
+/// Lee `tareas/ROADMAP.md` sin tocar el indice: el texto como lo guardaria
+/// git (`hash-object` aplica los filtros de la ruta), con la zona validada
+/// antes de nada (MEN-1 de la revision 1) y comparada con la del `HEAD` de la
+/// carpeta, de la que partio la edicion.
+fn editado_en(g: &Git, carpeta: &Path) -> Result<Lectura, Informe> {
     let s = exigir(
         g,
         carpeta,
@@ -260,46 +389,126 @@ fn editado_en(g: &Git, carpeta: &Path) -> Result<Option<String>, Informe> {
         &["--porcelain", "--", ROADMAP],
     )?;
     if s.trim().is_empty() {
-        return Ok(None);
+        return Ok(Lectura::Limpia);
     }
-    exigir(g, carpeta, NOMBRE, Subcomando::Add, &["--", ROADMAP])?;
-    let texto = crate::git::estado::leer_blob(g, carpeta, NOMBRE, "", ROADMAP)?;
-    let donde = format!("{CARPETA}/{ROADMAP}");
-    match zona::extraer(&texto) {
-        Ok(Some(_)) => Ok(Some(texto)),
-        Ok(None) => Err(de_fallo(roto(&donde, "sin-zona")).con("roto", "sin-zona")),
-        Err(r) => Err(de_fallo(roto(&donde, r.motivo())).con("roto", r.motivo())),
+    let sha = exigir(
+        g,
+        carpeta,
+        NOMBRE,
+        Subcomando::HashObject,
+        &["-w", "--", ROADMAP],
+    )?;
+    let texto = exigir(
+        g,
+        carpeta,
+        NOMBRE,
+        Subcomando::CatFile,
+        &["blob", sha.trim()],
+    )?;
+    let mia = match zona::extraer(&texto) {
+        Ok(Some(z)) => z.to_owned(),
+        Ok(None) => return Err(de_fallo(roto(&donde(), "sin-zona")).con("roto", "sin-zona")),
+        Err(r) => return Err(de_fallo(roto(&donde(), r.motivo())).con("roto", r.motivo())),
+    };
+    let base = crate::git::estado::leer_blob(g, carpeta, NOMBRE, "HEAD", ROADMAP)
+        .ok()
+        .and_then(|t| zona_de(&t));
+    if base.as_deref() == Some(mia.as_str()) {
+        return Ok(Lectura::FueraDeZona);
     }
+    Ok(Lectura::Zona(Edicion { texto, base }))
+}
+
+/// Lo que hay que deshacer si `--escribir` no publica: el contenido que el
+/// indice tenia para `ROADMAP.md` antes de preparar la edicion.
+struct Preparado {
+    previo: String,
+}
+
+/// Prepara la edicion (sin remoto, el adelanto de la carpeta va dentro de la
+/// transaccion y la necesita preparada) recordando lo que habia en el indice.
+fn preparar(g: &Git, carpeta: &Path) -> Result<Preparado, Informe> {
+    let previo = crate::git::estado::leer_blob(g, carpeta, NOMBRE, "", ROADMAP)?;
+    exigir(g, carpeta, NOMBRE, Subcomando::Add, &["--", ROADMAP])?;
+    Ok(Preparado { previo })
 }
 
-/// `--escribir`: publica el ROADMAP que toca como una transaccion (D1).
+/// Deja el indice como estaba (MEN-2 de la revision 1) con la unica
+/// escritura de indice que permite la capa Git (`git add`): se pone el
+/// contenido previo, se prepara y se devuelve el de la persona, que se
+/// tenia en memoria. Si no se puede devolver, se dice.
+fn restaurar_indice(g: &Git, carpeta: &Path, p: &Preparado) -> Result<(), Informe> {
+    let ruta = carpeta.join(ROADMAP);
+    let error = |e: std::io::Error| {
+        Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(
+            Mensaje::nuevo(&txt::NO_SE_RESTAURO)
+                .con("donde", donde())
+                .con("error", e.to_string()),
+        )
+    };
+    let mio = std::fs::read(&ruta).map_err(error)?;
+    std::fs::write(&ruta, p.previo.as_bytes()).map_err(error)?;
+    let preparado = exigir(g, carpeta, NOMBRE, Subcomando::Add, &["--", ROADMAP]);
+    std::fs::write(&ruta, &mio).map_err(error)?;
+    preparado.map(|_| ())
+}
+
+/// `--escribir`: publica el ROADMAP que toca como una transaccion (D1). La
+/// zona editada en `tareas/` se prepara solo sin remoto; si no se publica, el
+/// indice vuelve a como estaba y la edicion sigue en la carpeta.
 pub fn escribir_en(g: &Git, dir: &Path, catalogo: Catalogo<'_>) -> Informe {
     let carpeta = match carpeta_montada(g, dir) {
         Ok(c) => c,
         Err(i) => return renombrado(i),
     };
-    let editado = match carpeta.as_deref().map(|c| editado_en(g, c)).transpose() {
-        Ok(e) => e.flatten(),
+    let lectura = match carpeta.as_deref().map(|c| editado_en(g, c)).transpose() {
+        Ok(l) => l.unwrap_or(Lectura::Limpia),
         Err(i) => return renombrado(i),
     };
-    let op = match &editado {
-        Some(t) => Regenerar::con_editado(t.clone()),
-        None => Regenerar::nueva(),
+    let remoto = match crate::git::estado::hay_remoto(g, dir, NOMBRE) {
+        Ok(r) => r,
+        Err(i) => return renombrado(i),
+    };
+    let (personas, mut avisos) = personas_con_avisos(g, dir);
+    let mut preparado = None;
+    let mut editada = false;
+    let op = match (lectura, carpeta.as_deref()) {
+        (Lectura::Zona(ed), Some(c)) => {
+            if !remoto {
+                match preparar(g, c) {
+                    Ok(p) => preparado = Some(p),
+                    Err(i) => return renombrado(i),
+                }
+            }
+            editada = true;
+            Regenerar::con_editado(ed.texto, ed.base)
+        }
+        (Lectura::FueraDeZona, _) => {
+            avisos.push(FUERA_DE_ZONA);
+            Regenerar::nueva()
+        }
+        _ => Regenerar::nueva(),
     };
-    let personas = personas(g, dir);
     let generar = generador(&personas);
     let c = estado::Contexto::nuevo(g, dir, catalogo, Some(&generar));
     let mut i = renombrado(estado::transaccion(&c, &op));
-    if editado.is_some()
+    if let (Some(p), Some(carpeta)) = (&preparado, carpeta.as_deref())
+        && i.resultado != Resultado::Ok
+        && let Err(e) = restaurar_indice(g, carpeta, p)
+    {
+        return renombrado(e);
+    }
+    if editada
         && i.resultado == Resultado::Ok
         && i.valor("tareas") == Some("con-cambios")
-        && let (Some(carpeta), Some(commit)) = (carpeta, i.valor("commit").map(str::to_owned))
-        && adelantar_editada(g, &carpeta, &commit)
+        && let (Some(carpeta), Some(commit)) =
+            (carpeta.as_deref(), i.valor("commit").map(str::to_owned))
+        && adelantar_editada(g, carpeta, &commit)
         && let Some(v) = i.claves.iter_mut().find(|(k, _)| k == "tareas")
     {
         v.1 = Valor::Texto("al-dia".to_owned());
     }
-    i
+    con_avisos(i, &avisos)
 }
 
 /// Tras publicar la zona editada, deja `tareas/ROADMAP.md` como lo
diff --git a/crates/tc2/src/roadmap/zona.rs b/crates/tc2/src/roadmap/zona.rs
index cf3e901..7460975 100644
--- a/crates/tc2/src/roadmap/zona.rs
+++ b/crates/tc2/src/roadmap/zona.rs
@@ -10,17 +10,21 @@
 //!
 //! Hoja pura.
 
-/// Marca de inicio (la linea empieza asi, tras espacios).
+/// Marca de inicio: la linea (sin espacios alrededor) es
+/// `<!-- manual:inicio -->` o `<!-- manual:inicio <texto> -->`.
 pub const INICIO: &str = "<!-- manual:inicio";
-/// Marca de fin.
-pub const FIN: &str = "<!-- manual:fin";
+/// Marca de fin: la linea (sin espacios alrededor) es exactamente esta.
+pub const FIN: &str = "<!-- manual:fin -->";
 
 /// Zona de un ROADMAP recien creado: la de la 0.16, con `tc2` en lugar de
-/// `taskctl`.
+/// `taskctl` y un `### H0` para que las tareas nuevas (hito 0) no avisen de
+/// un hito sin titulo.
 pub const POR_DEFECTO: &str = "<!-- manual:inicio (se edita a mano; tc2 la conserva) -->\n\
 ## Hitos\n\
 \n\
 Un `### Hn — título` por hito (n es el campo `sprint` de sus tareas), en el orden en que se hacen y con su motivo debajo. tc2 conserva esta zona tal cual y toma de aquí los títulos y el orden de los hitos.\n\
+\n\
+### H0 — Primer hito\n\
 <!-- manual:fin -->\n";
 
 /// Por que la zona manual esta rota.
@@ -45,8 +49,18 @@ impl Rota {
     }
 }
 
-fn es_marca(linea: &str, marca: &str) -> bool {
-    linea.trim_start().starts_with(marca)
+/// Si `linea` es una marca de inicio: la marca completa, no un prefijo
+/// (`<!-- manual:inicio2 -->` no lo es).
+fn es_inicio(linea: &str) -> bool {
+    linea
+        .trim()
+        .strip_prefix(INICIO)
+        .is_some_and(|resto| resto == " -->" || resto.starts_with(' ') && resto.ends_with(" -->"))
+}
+
+/// Si `linea` es la marca de fin, exacta salvo espacios alrededor.
+fn es_fin(linea: &str) -> bool {
+    linea.trim() == FIN
 }
 
 /// La zona manual de `texto`, desde el principio de la linea de inicio hasta
@@ -57,12 +71,12 @@ pub fn extraer(texto: &str) -> Result<Option<&str>, Rota> {
     let mut fin: Option<usize> = None;
     let mut pos = 0;
     for linea in texto.split_inclusive('\n') {
-        if es_marca(linea, INICIO) {
+        if es_inicio(linea) {
             if inicio.is_some() {
                 return Err(Rota::Repetida);
             }
             inicio = Some(pos);
-        } else if es_marca(linea, FIN) {
+        } else if es_fin(linea) {
             match (inicio, fin) {
                 (None, _) => return Err(Rota::FinSuelto),
                 (Some(_), Some(_)) => return Err(Rota::Repetida),
@@ -85,9 +99,9 @@ pub fn interior_tolerante(texto: &str) -> String {
     let mut dentro = false;
     let mut zona = String::new();
     for l in texto.lines() {
-        if es_marca(l, INICIO) {
+        if es_inicio(l) {
             dentro = true;
-        } else if es_marca(l, FIN) {
+        } else if es_fin(l) {
             break;
         } else if dentro {
             zona.push_str(l);
@@ -144,6 +158,20 @@ mod tests {
         assert_eq!(Rota::SinFin.motivo(), "sin-fin");
     }
 
+    /// MEN-2 de la revision 1: la marca es la completa, no un prefijo.
+    #[test]
+    fn marcas_completas_no_prefijos() {
+        let t = "<!-- manual:inicio -->\n<!-- manual:inicio2 -->\n<!-- manual:finalizado -->\n  <!-- manual:fin -->  \r\n";
+        assert_eq!(extraer(t), Ok(Some(t)));
+        assert_eq!(extraer("<!-- manual:iniciox -->\nx\n"), Ok(None));
+        assert_eq!(
+            extraer("<!-- manual:inicio (a mano) -->\n<!-- manual:fin -->\n"),
+            Ok(Some(
+                "<!-- manual:inicio (a mano) -->\n<!-- manual:fin -->\n"
+            ))
+        );
+    }
+
     #[test]
     fn la_zona_por_defecto_es_valida() {
         assert_eq!(extraer(POR_DEFECTO), Ok(Some(POR_DEFECTO)));
diff --git a/crates/tc2/src/siguiente.rs b/crates/tc2/src/siguiente.rs
index 06fc87a..0cdd6f7 100644
--- a/crates/tc2/src/siguiente.rs
+++ b/crates/tc2/src/siguiente.rs
@@ -134,7 +134,7 @@ mod tests {
         let mut m = BTreeMap::new();
         m.insert(
             "TASK-001/tarea.md".into(),
-            tarea("TASK-001", 0, "planificada", &[]),
+            tarea("TASK-001", 1, "planificada", &[]),
         );
         m.insert("TASK-002/tarea.md".into(), "---\nid: roto\n".into());
         m.insert("TASK-003/plan.md".into(), "no es un tarea.md".into());
@@ -143,8 +143,9 @@ mod tests {
         let Ok(c) = c else { return };
         assert_eq!(
             c.informe("0123456789ab").linea(),
-            // Sin ROADMAP.md, el hito va por su numero y se avisa.
-            "next ok listas=TASK-001 en-marcha= commit=0123456789ab ilegibles=TASK-002 hitos-sin-titulo=H0"
+            // Sin ROADMAP.md vale la zona por defecto, que solo declara H0:
+            // H1 va por su numero y se avisa.
+            "next ok listas=TASK-001 en-marcha= commit=0123456789ab ilegibles=TASK-002 hitos-sin-titulo=H1"
         );
     }
 
diff --git a/crates/tc2/src/situacion.rs b/crates/tc2/src/situacion.rs
index 23b65c9..b4667b1 100644
--- a/crates/tc2/src/situacion.rs
+++ b/crates/tc2/src/situacion.rs
@@ -91,10 +91,12 @@ pub fn calcular(estado: &dyn Estado) -> Result<Situacion, Fallo> {
     // Detras de las legibles: si un ID se repite, vale la legible. Una
     // ilegible bloquea a quien dependa de ella.
     nodos.extend(ilegibles.iter().map(|id| Nodo::ilegible(id)));
-    let hitos = roadmap
-        .as_deref()
-        .map(Hitos::del_roadmap)
-        .unwrap_or_default();
+    // Sin ROADMAP o sin zona, la de por defecto: es la que llevara el que se
+    // genere, y asi el primero ya esta al dia (`--comparar`).
+    let hitos = match roadmap.as_deref() {
+        Some(t) if zona::extraer(t) != Ok(None) => Hitos::del_roadmap(t),
+        _ => Hitos::del_roadmap(zona::POR_DEFECTO),
+    };
     let orden = grafo::orden_de_trabajo(&nodos, &hitos);
     Ok(Situacion {
         listas: grafo::listas(&orden, &nodos),
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 8a54049c717b71525d0a180c8e00676686271db0..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-031/planificacion/brainstorm/peticion-plan-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-031/planificacion/plan-final.md                 |    0
 tareas/03-en-revision/TASK-031/revision/informe-revision-1-code-quality-reviewer.md         |  182 ++++++++++
 tareas/03-en-revision/TASK-031/revision/informe-revision-1-rust-wasm-reviewer.md            |   72 ++++
 tareas/03-en-revision/TASK-031/revision/peticion-revision-1-code-quality-reviewer.md        |   98 +++++
 tareas/03-en-revision/TASK-031/revision/peticion-revision-1-rust-wasm-reviewer.md           | 1975 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-031/tarea.md                                    |   16 +-
 7 files changed, 2340 insertions(+), 3 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff 8a54049c717b71525d0a180c8e00676686271db0..HEAD -- ":(literal)crates/tc2/tests/roadmap.rs"
````

````
crates/tc2/tests/roadmap.rs | 178 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-----
 1 file changed, 173 insertions(+), 5 deletions(-)
````
