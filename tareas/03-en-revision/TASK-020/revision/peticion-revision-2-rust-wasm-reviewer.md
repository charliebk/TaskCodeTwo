# Peticion de revision — TASK-020 (ronda 2)

- Tarea: TASK-020 — F2-5 Modelo de tarea y parser/serializador de frontmatter YAML valido
- Rama revisada: feature/task-020-f2-5-modelo-de-tarea-y-parser-serializad
- Rama base: develop
- Commit revisado (HEAD): 28ac126257f93838179e03a8e9905745575499a1
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-020 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde f2fdb154fbe7f9cef83906ff34b4c79fd56b290d (el commit revisado en la ronda anterior)
- Alcance: completa (motivo: un informe de la ronda anterior no tiene tabla de hallazgos legible)

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
informe de esta ronda (informe-revision-2-rust-wasm-reviewer.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-020/tarea.md
- tareas/03-en-revision/TASK-020/planificacion/
- crates/tc2/src/tarea/cuerpo.rs
- crates/tc2/src/tarea/modelo.rs
- crates/tc2/src/tarea/yaml.rs
- crates/tc2/tests/tarea.rs

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.


Tabla de hallazgos ausente o ilegible en: informe-revision-1-rust-wasm-reviewer.md. Lee esos informes enteros.

## Commits a revisar (git log f2fdb154fbe7f9cef83906ff34b4c79fd56b290d..HEAD)

````
28ac126 chore(TASK-020): autorrevision ronda 2
53c942f fix(TASK-020): hallazgos de la ronda 1 (tipos canonicos, U+FFFE, cercas)
29fcca4 chore(TASK-020): peticion de revision ronda 1
````

## Diff de tu dominio (4 fichero(s) de 4; git diff f2fdb154fbe7f9cef83906ff34b4c79fd56b290d..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/tarea/cuerpo.rs b/crates/tc2/src/tarea/cuerpo.rs
index 6c827cf..eae57ff 100644
--- a/crates/tc2/src/tarea/cuerpo.rs
+++ b/crates/tc2/src/tarea/cuerpo.rs
@@ -43,10 +43,34 @@ pub struct Cuerpo {
     pub secciones: Vec<Seccion>,
 }
 
-/// Si `linea` abre o cierra un bloque de codigo cercado.
-fn es_cerca(linea: &str) -> bool {
-    let l = linea.trim_start();
-    l.starts_with("```") || l.starts_with("~~~")
+/// Cerca de codigo de `linea` (CommonMark 4.5): hasta 3 espacios de
+/// sangria y 3 o mas `` ` `` o `~` seguidos. Devuelve el caracter, cuantos
+/// lleva y el resto de la linea.
+fn cerca(linea: &str) -> Option<(char, usize, &str)> {
+    let sin_sangria = linea.trim_start_matches(' ');
+    if linea.len() - sin_sangria.len() > 3 {
+        return None;
+    }
+    let c = sin_sangria
+        .chars()
+        .next()
+        .filter(|c| matches!(c, '`' | '~'))?;
+    let n = sin_sangria.chars().take_while(|x| *x == c).count();
+    (n >= 3).then(|| (c, n, &sin_sangria[n..]))
+}
+
+/// Si `linea` abre un bloque: en una cerca de `` ` `` la informacion que
+/// sigue no puede llevar `` ` ``.
+fn abre(linea: &str) -> Option<(char, usize)> {
+    let (c, n, resto) = cerca(linea)?;
+    (c == '~' || !resto.contains('`')).then_some((c, n))
+}
+
+/// Si `linea` cierra el bloque abierto con `abierta`: el mismo caracter, al
+/// menos tantos y nada mas que espacios detras.
+fn cierra(linea: &str, abierta: (char, usize)) -> bool {
+    cerca(linea)
+        .is_some_and(|(c, n, resto)| c == abierta.0 && n >= abierta.1 && resto.trim().is_empty())
 }
 
 /// Titulo de `linea` si es un encabezado de seccion `## ` (no `###`).
@@ -60,12 +84,20 @@ impl Cuerpo {
     /// cercado no abre seccion.
     pub fn leer(texto: &str) -> Self {
         let mut cuerpo = Self::default();
-        let mut en_codigo = false;
+        let mut abierta: Option<(char, usize)> = None;
         for linea in texto.split_inclusive('\n') {
-            let encabezado = if en_codigo { None } else { titulo(linea) };
-            if es_cerca(linea) {
-                en_codigo = !en_codigo;
-            }
+            let encabezado = match abierta {
+                Some(a) => {
+                    if cierra(linea, a) {
+                        abierta = None;
+                    }
+                    None
+                }
+                None => {
+                    abierta = abre(linea);
+                    titulo(linea)
+                }
+            };
             match (encabezado, cuerpo.secciones.last_mut()) {
                 (Some(t), _) => cuerpo.secciones.push(Seccion {
                     titulo: t,
@@ -119,4 +151,26 @@ mod tests {
         assert_eq!(p.escribir(), PLANTILLA);
         assert_eq!(p.secciones.len(), 5);
     }
+
+    #[test]
+    fn cercas_como_commonmark() {
+        let titulos = |t: &str| -> Vec<String> {
+            Cuerpo::leer(t)
+                .secciones
+                .into_iter()
+                .map(|s| s.titulo)
+                .collect()
+        };
+        // Cuatro acentos no se cierran con tres; `~~~` no cierra `` ``` ``.
+        assert_eq!(titulos("## A\n````\n```\n## B\n````\n## C\n"), ["A", "C"]);
+        assert_eq!(titulos("## A\n```\n~~~\n## B\n```\n## C\n"), ["A", "C"]);
+        // `` ```x``` `` no abre (lleva acentos en la informacion), una
+        // sangria de 4 no es cerca, y una cierre con texto detras no cierra.
+        assert_eq!(titulos("## A\n```x```\n## B\n"), ["A", "B"]);
+        assert_eq!(titulos("## A\n    ```\n## B\n"), ["A", "B"]);
+        assert_eq!(
+            titulos("## A\n~~~ md\n~~~ no\n## B\n~~~\n## C\n"),
+            ["A", "C"]
+        );
+    }
 }
diff --git a/crates/tc2/src/tarea/modelo.rs b/crates/tc2/src/tarea/modelo.rs
index 18ad743..7c2f2f0 100644
--- a/crates/tc2/src/tarea/modelo.rs
+++ b/crates/tc2/src/tarea/modelo.rs
@@ -177,7 +177,8 @@ pub struct Tarea {
     pub asignado_a: Option<String>,
     pub roles: Roles,
     /// Entrada `estimacion:` cruda (su forma la fija F1-17); `None` es `null`.
-    pub estimacion: Option<String>,
+    /// Privada: solo llega de una lectura, que la ha validado como YAML.
+    estimacion: Option<String>,
     pub tokens_diseno: Option<u64>,
     pub tokens_implementacion: Option<u64>,
     pub tokens_revision: Option<u64>,
@@ -185,8 +186,9 @@ pub struct Tarea {
     pub actualizado: String,
     pub dependencias: Vec<String>,
     /// Claves desconocidas con su texto crudo, en su orden (F1-9 D4a: se
-    /// conservan en la ida y vuelta; `doctor` avisa).
-    pub desconocidas: Vec<(String, String)>,
+    /// conservan en la ida y vuelta; `doctor` avisa). Privadas: solo llegan
+    /// de una lectura, que las ha validado y comprobado sin duplicadas.
+    desconocidas: Vec<(String, String)>,
 }
 
 #[derive(Deserialize)]
@@ -345,9 +347,25 @@ impl Tarea {
         &self.desconocidas
     }
 
+    /// Entrada `estimacion:` cruda tal como se leyo; `None` si es `null`.
+    pub fn estimacion(&self) -> Option<&str> {
+        self.estimacion.as_deref()
+    }
+
     /// Lee el frontmatter de formato 2 con el esquema estricto.
     pub(crate) fn leer(partes: &Partes<'_>) -> Result<Self, ErrorTarea> {
         faltan(partes, &CLAVES)?;
+        canonicos(
+            partes,
+            &[
+                ("formato", Canon::Entero),
+                ("sprint", Canon::Entero),
+                ("plan_aprobado", Canon::Bool),
+                ("tokens_diseno", Canon::EnteroONull),
+                ("tokens_implementacion", Canon::EnteroONull),
+                ("tokens_revision", Canon::EnteroONull),
+            ],
+        )?;
         let c: CrudoV2 = yaml::decodificar(&partes.yaml())?;
         let formato = exigir("formato", c.formato)?;
         if formato != 2 {
@@ -422,8 +440,9 @@ impl Tarea {
     /// Frontmatter en la forma fija de F1-9 seguido de `cuerpo`. Las claves
     /// desconocidas van al final, en su orden y con su texto original.
     /// No valida: un texto con caracteres de control sale escapado (YAML
-    /// valido) pero [`super::leer`] lo rechaza, y `estimacion` y las
-    /// desconocidas se copian tal cual (tienen que venir de una lectura).
+    /// valido) pero [`super::leer`] lo rechaza. Lo que no conserva: los
+    /// comentarios del frontmatter, el BOM y los `\r\n` del frontmatter (sale
+    /// con `\n`; el cuerpo, byte a byte como se leyo).
     pub fn escribir(&self, cuerpo: &Cuerpo) -> String {
         let r = &self.roles;
         let rol = |x: &Rol| {
@@ -556,6 +575,9 @@ struct CrudoV1 {
 /// La 0.16 escapa `"` pero no `\` (H11): un `\` que no va seguido de `"`
 /// lo leeria mal un parser estandar (`C:\ruta` daria un retorno de carro).
 /// Mejor un error claro que un valor cambiado en silencio.
+/// Solo mira dentro de cadenas entre comillas dobles: en una cadena plana,
+/// de comilla simple o en un comentario la barra es literal tambien para un
+/// parser estandar.
 fn escape_ambiguo(partes: &Partes<'_>) -> Result<(), ErrorTarea> {
     for e in partes
         .entradas
@@ -563,13 +585,69 @@ fn escape_ambiguo(partes: &Partes<'_>) -> Result<(), ErrorTarea> {
         .filter(|e| CLAVES_V1.contains(&e.clave.as_str()))
     {
         let mut cs = e.crudo.chars().peekable();
+        let mut dobles = false;
         while let Some(c) = cs.next() {
-            if c == '\\' && cs.next_if_eq(&'"').is_none() {
-                return Err(ErrorTarea::en(
-                    &e.clave,
-                    "lleva `\\` con el escape de la 0.16, que un parser YAML estandar lee distinto (H11)",
-                ));
+            match (dobles, c) {
+                (true, '\\') if cs.next_if_eq(&'"').is_none() => {
+                    return Err(ErrorTarea::en(
+                        &e.clave,
+                        "lleva `\\` con el escape de la 0.16, que un parser YAML estandar lee distinto (H11)",
+                    ));
+                }
+                (true, '"') | (false, '"') => dobles = !dobles,
+                (false, '\'') => {
+                    // Comilla simple: hasta la siguiente (`''` es una comilla).
+                    while cs.next().is_some_and(|c| c != '\'') {}
+                }
+                (false, '#') => {
+                    // Comentario: hasta el final de la linea.
+                    while cs.next().is_some_and(|c| c != '\n') {}
+                }
+                _ => {}
+            }
+        }
+    }
+    Ok(())
+}
+
+/// Grafia que se exige al texto crudo de un campo no textual.
+#[derive(Clone, Copy)]
+enum Canon {
+    /// `true` o `false`.
+    Bool,
+    /// `0` o `[1-9][0-9]*`.
+    Entero,
+    /// Un [`Canon::Entero`] o `null`.
+    EnteroONull,
+}
+
+/// Los campos bool y enteros llevan su grafia canonica, sin comillas: un
+/// `'7'` o un `"false"` serian cadenas para un parser estandar y el crate los
+/// convertiria (F1-9 punto 5: tipo incorrecto da error).
+fn canonicos(partes: &Partes<'_>, campos: &[(&str, Canon)]) -> Result<(), ErrorTarea> {
+    let entero = |v: &str| {
+        v == "0" || (!v.starts_with('0') && !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()))
+    };
+    for (clave, canon) in campos {
+        let Some(v) = partes.valor_plano(clave) else {
+            // Falta (lo dice `faltan`/`exigir`) o ocupa varias lineas.
+            if partes.tiene(clave) {
+                return Err(ErrorTarea::en(clave, "tiene que ir en una sola linea"));
             }
+            continue;
+        };
+        let vale = match canon {
+            Canon::Bool => v == "true" || v == "false",
+            Canon::Entero => entero(v),
+            Canon::EnteroONull => v == "null" || entero(v),
+        };
+        if !vale {
+            let esperado = match canon {
+                Canon::Bool => "`true` o `false`",
+                Canon::Entero => "un entero sin comillas",
+                Canon::EnteroONull => "un entero sin comillas o `null`",
+            };
+            return Err(ErrorTarea::en(clave, format!("«{v}» no es {esperado}")));
         }
     }
     Ok(())
@@ -580,6 +658,17 @@ impl TareaV1 {
     /// (`validateTask`): obligatorios los que la 0.16 exige.
     pub(crate) fn leer(partes: &Partes<'_>) -> Result<Self, ErrorTarea> {
         escape_ambiguo(partes)?;
+        canonicos(
+            partes,
+            &[
+                ("sprint", Canon::Entero),
+                ("plan_aprobado", Canon::Bool),
+                ("revision_codex", Canon::Bool),
+                ("tokens_diseno", Canon::EnteroONull),
+                ("tokens_implementacion", Canon::EnteroONull),
+                ("tokens_revision", Canon::EnteroONull),
+            ],
+        )?;
         let c: CrudoV1 = yaml::decodificar(&partes.yaml())?;
         Ok(Self {
             id: id("id", exigir("id", c.id)?)?,
diff --git a/crates/tc2/src/tarea/yaml.rs b/crates/tc2/src/tarea/yaml.rs
index bd12415..b18cd5a 100644
--- a/crates/tc2/src/tarea/yaml.rs
+++ b/crates/tc2/src/tarea/yaml.rs
@@ -10,6 +10,8 @@
 //!   forma byte a byte (comillas dobles siempre, orden de claves, `roles` en
 //!   flujo) y un serializador generico no la da.
 
+use std::collections::HashSet;
+
 use serde::de::DeserializeOwned;
 use serde_saphyr::{DuplicateKeyPolicy, MergeKeyPolicy, Options};
 
@@ -39,6 +41,20 @@ impl Partes<'_> {
         self.entradas.iter().any(|e| e.clave == clave)
     }
 
+    /// Valor de una entrada de una sola linea, sin la clave, sin un
+    /// comentario ` #` final y sin espacios; `None` si falta o si ocupa
+    /// varias lineas.
+    pub fn valor_plano(&self, clave: &str) -> Option<&str> {
+        let e = self.entradas.iter().find(|e| e.clave == clave)?;
+        let linea = e.crudo.strip_suffix('\n')?;
+        if linea.contains('\n') {
+            return None;
+        }
+        let (_, valor) = linea.split_once(':')?;
+        let valor = valor.split_once(" #").map_or(valor, |(v, _)| v);
+        Some(valor.trim())
+    }
+
     /// El texto YAML del frontmatter (sin delimitadores), normalizado a `\n`.
     pub fn yaml(&self) -> String {
         self.entradas.iter().map(|e| e.crudo.as_str()).collect()
@@ -67,6 +83,8 @@ pub(crate) fn partir(texto: &str) -> Result<Partes<'_>, ErrorTarea> {
     }
     let mut consumido = texto.split_inclusive('\n').next().map_or(0, str::len);
     let mut entradas: Vec<Entrada> = Vec::new();
+    // Para detectar duplicadas en tiempo lineal (dato no confiable, F1-8).
+    let mut vistas: HashSet<String> = HashSet::new();
     let mut cerrado = false;
     for l in lineas {
         consumido += l.len();
@@ -83,7 +101,9 @@ pub(crate) fn partir(texto: &str) -> Result<Partes<'_>, ErrorTarea> {
                     e.crudo.push_str(&linea);
                     e.crudo.push('\n');
                 }
-                None if linea.trim().is_empty() => {}
+                // Lineas en blanco y comentarios antes de la primera clave
+                // son YAML valido; como todo comentario, no se conservan.
+                None if linea.trim().is_empty() || linea.starts_with('#') => {}
                 None => {
                     return Err(ErrorTarea::general(
                         "texto antes de la primera clave del frontmatter",
@@ -102,7 +122,7 @@ pub(crate) fn partir(texto: &str) -> Result<Partes<'_>, ErrorTarea> {
                 "clave no admitida en el frontmatter: «{clave}»"
             )));
         }
-        if entradas.iter().any(|e| e.clave == clave) {
+        if !vistas.insert(clave.to_owned()) {
             return Err(ErrorTarea::en(clave, "clave duplicada"));
         }
         entradas.push(Entrada {
@@ -124,6 +144,11 @@ pub(crate) fn partir(texto: &str) -> Result<Partes<'_>, ErrorTarea> {
 /// denegacion de servicio, F1-8).
 fn opciones() -> Options {
     let mut o = Options::default();
+    // Booleanos y numeros solo con la grafia de YAML 1.2 (`yes`, `1_0` o `010`
+    // no son bool ni entero, como para un parser estandar).
+    o.strict_booleans = true;
+    o.underscored_numbers_as_strings = true;
+    o.leading_zero_numbers_as_strings = true;
     o.duplicate_keys = DuplicateKeyPolicy::Error;
     o.merge_keys = MergeKeyPolicy::Error;
     let mut presupuesto = o.budget.unwrap_or_default();
@@ -150,7 +175,14 @@ pub(crate) fn cadena(texto: &str) -> String {
         match c {
             '\\' => s.push_str("\\\\"),
             '"' => s.push_str("\\\""),
-            c if c.is_control() || matches!(c, '\u{2028}' | '\u{2029}' | '\u{feff}') => {
+            // U+FFFE y U+FFFF no son imprimibles en YAML: un parser estandar
+            // los rechaza si van tal cual.
+            c if c.is_control()
+                || matches!(
+                    c,
+                    '\u{2028}' | '\u{2029}' | '\u{feff}' | '\u{fffe}' | '\u{ffff}'
+                ) =>
+            {
                 s.push_str(&format!("\\u{:04X}", u32::from(c)));
             }
             c => s.push(c),
@@ -182,7 +214,7 @@ mod tests {
 
     #[test]
     fn partir_separa_entradas_y_cuerpo_intacto() -> Result<(), ErrorTarea> {
-        let t = "\u{feff}---\r\nid: \"TASK-001\"\r\nroles:\r\n  a: {b: 1}\r\n# nota\r\nx: 1\r\n--- \r\n## Objetivo\r\n";
+        let t = "\u{feff}---\r\n# inicial\r\nid: \"TASK-001\"\r\nroles:\r\n  a: {b: 1}\r\n# nota\r\nx: 1\r\n--- \r\n## Objetivo\r\n";
         let p = partir(t)?;
         let claves: Vec<&str> = p.entradas.iter().map(|e| e.clave.as_str()).collect();
         assert_eq!(claves, ["id", "roles", "x"]);
````

## Excluido del diff (5 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff f2fdb154fbe7f9cef83906ff34b4c79fd56b290d..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-020/planificacion/brainstorm/peticion-plan-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-020/planificacion/plan-final.md                 |    0
 tareas/03-en-revision/TASK-020/revision/informe-revision-1-rust-wasm-reviewer.md            |   21 ++
 tareas/03-en-revision/TASK-020/revision/peticion-revision-1-rust-wasm-reviewer.md           | 1343 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-020/tarea.md                                    |   10 +-
 5 files changed, 1371 insertions(+), 3 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff f2fdb154fbe7f9cef83906ff34b4c79fd56b290d..HEAD -- ":(literal)crates/tc2/tests/tarea.rs"
````

````
crates/tc2/tests/tarea.rs | 90 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-----
 1 file changed, 85 insertions(+), 5 deletions(-)
````
