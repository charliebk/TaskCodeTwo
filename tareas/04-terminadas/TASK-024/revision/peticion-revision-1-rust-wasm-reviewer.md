# Peticion de revision — TASK-024 (ronda 1)

- Tarea: TASK-024 — F2-8 i18n de mensajes
- Rama revisada: feature/task-024-f2-8-i18n-de-mensajes
- Rama base: develop
- Commit revisado (HEAD): a6713930b5f17122ab4d850a1293fc185328c47d
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-024 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-024/tarea.md
- tareas/03-en-revision/TASK-024/planificacion/
- crates/tc2/src/alta.rs
- crates/tc2/src/config/claves.rs
- crates/tc2/src/config/mod.rs
- crates/tc2/src/config/validar.rs
- crates/tc2/src/contrato.rs
- crates/tc2/src/estado/lectura.rs
- crates/tc2/src/estado/montaje.rs
- crates/tc2/src/estado/transaccion.rs
- crates/tc2/src/git/diff.rs
- crates/tc2/src/git/estado.rs
- crates/tc2/src/git/guardar.rs
- crates/tc2/src/git/mod.rs
- crates/tc2/src/git/puerto.rs
- crates/tc2/src/git/rama.rs
- crates/tc2/src/git/remoto.rs
- crates/tc2/src/git/tag.rs
- crates/tc2/src/git/worktree.rs
- crates/tc2/src/i18n/catalogo/cli.rs
- crates/tc2/src/i18n/catalogo/comun.rs
- crates/tc2/src/i18n/catalogo/config.rs
- crates/tc2/src/i18n/catalogo/estado.rs
- crates/tc2/src/i18n/catalogo/git.rs
- crates/tc2/src/i18n/catalogo/mod.rs
- crates/tc2/src/i18n/catalogo/plataforma.rs
- crates/tc2/src/i18n/guarda.rs
- crates/tc2/src/i18n/mod.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/main.rs
- crates/tc2/src/plataforma/conexion.rs
- crates/tc2/src/plataforma/deteccion.rs
- crates/tc2/src/plataforma/errores.rs
- crates/tc2/src/plataforma/github.rs
- crates/tc2/src/plataforma/gitlab.rs
- crates/tc2/src/plataforma/http.rs
- crates/tc2/src/plataforma/puertas.rs
- crates/tc2/tests/cli.rs
- crates/tc2/tests/comun/ops.rs
- crates/tc2/tests/config.rs

## Commits a revisar (git log develop..HEAD)

````
a671393 chore(TASK-024): autorrevision
ce5b396 refactor(TASK-024): Display de ErrorGit desde el catalogo
ef1e309 merge(TASK-024): bloque c, plataforma/ y config/ al catalogo i18n
80ddd50 feat(TASK-024): mensajes de plataforma y config en el catalogo es/en
4a42c6c merge(TASK-024): bloque b, git/ estado/ y alta al catalogo i18n
e4b9b24 feat(TASK-024): migra git/, estado/ y alta al catalogo i18n
ffb61b4 feat(TASK-024): infraestructura i18n es/en y borde del CLI
618d639 chore(TASK-024): tarea en curso
````

## Diff de tu dominio (38 fichero(s) de 38; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/alta.rs b/crates/tc2/src/alta.rs
index a91c1b7..3e1d439 100644
--- a/crates/tc2/src/alta.rs
+++ b/crates/tc2/src/alta.rs
@@ -106,7 +106,7 @@ pub fn siguiente_id(rutas: &[String]) -> Result<u32, Fallo> {
     let maximo = rutas.iter().filter_map(|r| numero(r)).max().unwrap_or(0);
     maximo
         .checked_add(1)
-        .ok_or_else(|| Fallo::Error("no quedan IDs de tarea".to_owned()))
+        .ok_or_else(|| Fallo::Error((&crate::i18n::catalogo::estado::SIN_IDS).into()))
 }
 
 /// `tarea.md` de una tarea recien creada: el modelo de F2-5 con la
diff --git a/crates/tc2/src/config/claves.rs b/crates/tc2/src/config/claves.rs
index 039751f..a4d7107 100644
--- a/crates/tc2/src/config/claves.rs
+++ b/crates/tc2/src/config/claves.rs
@@ -5,6 +5,8 @@
 use serde_json::{Map, Value, json};
 
 use crate::carril::Carril;
+use crate::i18n::catalogo::config as t;
+use crate::i18n::{Idioma, Texto};
 
 /// Tope de claves de primer nivel (plan, F2-7; F1-11, decision 1).
 pub const TOPE_CLAVES: usize = 12;
@@ -88,27 +90,18 @@ pub(crate) enum Regla {
 }
 
 impl Regla {
-    /// Lo que exige, para el esquema y los mensajes.
-    pub(crate) fn texto(self) -> Option<&'static str> {
+    /// Lo que exige, del catalogo. El esquema JSON publicado usa el texto en
+    /// espanol (decision de F2-8: el esquema no se traduce).
+    pub(crate) fn texto(self) -> Option<&'static Texto> {
         match self {
             Self::Valor => None,
-            Self::Nombre => {
-                Some("texto no vacio, sin caracteres de control ni espacios en los extremos")
-            }
-            Self::Orden => Some(
-                "orden no vacia y sin caracteres de control (rutas de Windows entre comillas simples o con /)",
-            ),
-            Self::Glob => Some("glob relativo: sin negacion (!), rutas absolutas ni .."),
-            Self::GlobAcotado => Some(
-                "glob relativo: sin negacion (!), rutas absolutas ni .., y nunca todo el arbol (ningun segmento solo de * y .)",
-            ),
-            Self::RutaTareas => Some(
-                "ruta normalizada bajo tareas/, sin \\, :, comodines, caracteres de control, nombres reservados de Windows, .git, ~ ni punto o espacio al final de un segmento; nunca tareas/ entera, tareas/ROADMAP.md ni tareas/TASK-NNN/ (sin distinguir mayusculas)",
-            ),
-            Self::Cron => Some(
-                "expresion cron de 5 campos con sus rangos (minuto, hora, dia, mes, dia de la semana)",
-            ),
-            Self::Url => Some("URL https de la API (o http://127.0.0.1:<puerto> en pruebas)"),
+            Self::Nombre => Some(&t::REGLA_NOMBRE),
+            Self::Orden => Some(&t::REGLA_ORDEN),
+            Self::Glob => Some(&t::REGLA_GLOB),
+            Self::GlobAcotado => Some(&t::REGLA_GLOB_ACOTADO),
+            Self::RutaTareas => Some(&t::REGLA_RUTA_TAREAS),
+            Self::Cron => Some(&t::REGLA_CRON),
+            Self::Url => Some(&t::REGLA_URL),
         }
     }
 }
@@ -573,8 +566,8 @@ fn a_esquema(n: &Nodo) -> Value {
             } else {
                 m.insert("enum".into(), json!(valores));
             }
-            if let Some(t) = regla.texto() {
-                m.insert("description".into(), json!(t));
+            if let Some(r) = regla.texto() {
+                m.insert("description".into(), json!(r.en_idioma(Idioma::Es)));
             }
             Value::Object(m)
         }
diff --git a/crates/tc2/src/config/mod.rs b/crates/tc2/src/config/mod.rs
index 86189a6..4becc1b 100644
--- a/crates/tc2/src/config/mod.rs
+++ b/crates/tc2/src/config/mod.rs
@@ -28,6 +28,8 @@ pub use claves::{CLAVES_016, DOMINIOS, FASES, MODELOS, TOPE_CLAVES, esquema_json
 use crate::carril::{Carril, Perfil};
 use crate::contrato::{Informe, Resultado};
 use crate::git::{self, Git, Paso, Subcomando};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::config as t;
 use crate::plataforma::Proveedor;
 
 /// Nombre de la operacion en la linea de contrato.
@@ -321,12 +323,12 @@ impl Motivo {
 pub struct ErrorConfig {
     /// Ruta de la clave; vacia si el error es del documento entero.
     pub ruta: String,
-    /// Que esta mal.
-    pub mensaje: String,
+    /// Que esta mal, como mensaje del catalogo (F2-8).
+    pub mensaje: Mensaje,
 }
 
 impl ErrorConfig {
-    pub(crate) fn nuevo(ruta: &str, mensaje: impl Into<String>) -> Self {
+    pub(crate) fn nuevo(ruta: &str, mensaje: impl Into<Mensaje>) -> Self {
         Self {
             ruta: ruta.to_owned(),
             mensaje: mensaje.into(),
@@ -344,7 +346,7 @@ pub struct Invalida {
 }
 
 impl Invalida {
-    fn una(motivo: Motivo, ruta: &str, mensaje: impl Into<String>) -> Self {
+    fn una(motivo: Motivo, ruta: &str, mensaje: impl Into<Mensaje>) -> Self {
         Self {
             motivo,
             errores: vec![ErrorConfig::nuevo(ruta, mensaje)],
@@ -366,39 +368,36 @@ impl Invalida {
         if !rutas.is_empty() {
             i = i.con_lista("claves", rutas);
         }
-        let primero = self
-            .errores
-            .first()
-            .map(|e| {
-                if e.ruta.is_empty() {
-                    e.mensaje.clone()
-                } else {
-                    format!("{}: {}", e.ruta, e.mensaje)
-                }
-            })
-            .unwrap_or_default();
+        let diag = match self.errores.first() {
+            Some(e) if !e.ruta.is_empty() => Mensaje::nuevo(&t::DIAGNOSTICO_CON_CLAVE)
+                .con("clave", e.ruta.clone())
+                .con_mensaje("mensaje", e.mensaje.clone()),
+            Some(e) => Mensaje::nuevo(&t::DIAGNOSTICO).con_mensaje("mensaje", e.mensaje.clone()),
+            None => Mensaje::nuevo(&t::DIAGNOSTICO).con("mensaje", ""),
+        };
         let arreglo = match self.motivo {
-            Motivo::SinConfig => "crea la configuracion con tc2 init".to_owned(),
-            Motivo::Formato => "es un proyecto de la 0.16: migralo con tc2 actualizar".to_owned(),
-            Motivo::FormatoNuevo => "el proyecto es de un tc2 mas nuevo: actualiza tc2".to_owned(),
-            Motivo::Yaml => format!("corrige el YAML de {RUTA}"),
+            Motivo::SinConfig => Mensaje::nuevo(&t::ARREGLO_SIN_CONFIG),
+            Motivo::Formato => Mensaje::nuevo(&t::ARREGLO_FORMATO),
+            Motivo::FormatoNuevo => Mensaje::nuevo(&t::ARREGLO_FORMATO_NUEVO),
+            Motivo::Yaml => Mensaje::nuevo(&t::ARREGLO_YAML).con("ruta", RUTA),
             Motivo::ConfigInvalida
                 if self
                     .errores
                     .iter()
                     .any(|e| CLAVES_016.contains(&e.ruta.as_str())) =>
             {
-                "quedan claves de la 0.16: tc2 actualizar las traduce".to_owned()
-            }
-            Motivo::ConfigInvalida => {
-                format!("corrige {RUTA} (esquema: crates/tc2/esquemas/config.schema.json)")
+                Mensaje::nuevo(&t::ARREGLO_CLAVES_016)
             }
+            Motivo::ConfigInvalida => Mensaje::nuevo(&t::ARREGLO_ESQUEMA).con("ruta", RUTA),
         };
-        let extra = match self.errores.len() {
-            0 | 1 => String::new(),
-            n => format!(" (y {} error(es) mas)", n - 1),
+        let diag = match self.errores.len() {
+            0 | 1 => diag.con("extra", ""),
+            n => diag.con_mensaje(
+                "extra",
+                Mensaje::nuevo(&t::MAS_ERRORES).con("n", (n - 1).to_string()),
+            ),
         };
-        i.diagnostico(format!("{RUTA}: {primero}{extra}. Arreglo: {arreglo}"))
+        i.diagnostico(diag.con("ruta", RUTA).con_mensaje("arreglo", arreglo))
     }
 }
 
@@ -407,23 +406,25 @@ pub fn leer(texto: &str) -> Result<Config, Invalida> {
     let primera = texto.split('\n').next().unwrap_or("");
     if primera.strip_suffix('\r').unwrap_or(primera) != "---" {
         let que = if primera.starts_with('\u{feff}') {
-            "la primera linea tiene que ser --- (sin BOM)"
+            &t::PRIMERA_LINEA_SIN_BOM
         } else {
-            "la primera linea tiene que ser ---"
+            &t::PRIMERA_LINEA
         };
         return Err(Invalida::una(Motivo::Formato, "", que));
     }
-    let raiz: Value = serde_saphyr::from_str_with_options(texto, crate::yaml::opciones())
-        .map_err(|e| Invalida::una(Motivo::Yaml, "", format!("YAML no valido: {e}")))?;
+    let raiz: Value =
+        serde_saphyr::from_str_with_options(texto, crate::yaml::opciones()).map_err(|e| {
+            Invalida::una(
+                Motivo::Yaml,
+                "",
+                Mensaje::nuevo(&t::YAML_NO_VALIDO).con("error", e.to_string()),
+            )
+        })?;
     let formato = match &raiz {
         Value::Object(m) => m.get("formato"),
         Value::Null => None,
         _ => {
-            return Err(Invalida::una(
-                Motivo::ConfigInvalida,
-                "",
-                "la raiz tiene que ser un mapa",
-            ));
+            return Err(Invalida::una(Motivo::ConfigInvalida, "", &t::RAIZ_NO_MAPA));
         }
     };
     match formato {
@@ -432,22 +433,20 @@ pub fn leer(texto: &str) -> Result<Config, Invalida> {
             return Err(Invalida::una(
                 Motivo::FormatoNuevo,
                 "formato",
-                format!("formato {f}: este tc2 solo lee el formato 2"),
+                Mensaje::nuevo(&t::FORMATO_NUEVO).con("formato", f.to_string()),
             ));
         }
         Some(f) => {
             return Err(Invalida::una(
                 Motivo::Formato,
                 "formato",
-                format!("se esperaba 2, no {f}"),
+                Mensaje::nuevo(&t::SE_ESPERABA)
+                    .con("esperado", "2")
+                    .con("valor", f.to_string()),
             ));
         }
         None => {
-            return Err(Invalida::una(
-                Motivo::Formato,
-                "formato",
-                "falta la clave obligatoria",
-            ));
+            return Err(Invalida::una(Motivo::Formato, "formato", &t::FALTA_CLAVE));
         }
     }
     let tabla = claves::Nodo::Objeto(claves::tabla());
@@ -467,7 +466,7 @@ pub fn leer(texto: &str) -> Result<Config, Invalida> {
         Invalida::una(
             Motivo::ConfigInvalida,
             "",
-            format!("error interno al construir la configuracion validada: {e}"),
+            Mensaje::nuevo(&t::ERROR_INTERNO).con("error", e.to_string()),
         )
     })
 }
@@ -501,9 +500,7 @@ fn texto_de_base(git: &Git, dir: &Path) -> Paso<String> {
     if git::commit_de(git, dir, NOMBRE, BASE)?.is_none() {
         return Err(Informe::nuevo(NOMBRE, Resultado::Rechazada)
             .con("motivo", "sin-base")
-            .diagnostico(format!(
-                "no existe {BASE}: haz fetch o crea la rama develop en el remoto"
-            )));
+            .diagnostico(Mensaje::nuevo(&t::SIN_BASE).con("base", BASE)));
     }
     let listado = git::exigir(
         git,
@@ -513,9 +510,14 @@ fn texto_de_base(git: &Git, dir: &Path) -> Paso<String> {
         &["--name-only", "--end-of-options", BASE, "--", RUTA],
     )?;
     if listado.trim() != RUTA {
-        return Err(
-            Invalida::una(Motivo::SinConfig, "", format!("no hay {RUTA} en {BASE}")).informe(),
-        );
+        return Err(Invalida::una(
+            Motivo::SinConfig,
+            "",
+            Mensaje::nuevo(&t::NO_HAY_CONFIG)
+                .con("ruta", RUTA)
+                .con("base", BASE),
+        )
+        .informe());
     }
     git::estado::leer_blob(git, dir, NOMBRE, BASE, RUTA)
 }
diff --git a/crates/tc2/src/config/validar.rs b/crates/tc2/src/config/validar.rs
index 0dfe484..61faf1d 100644
--- a/crates/tc2/src/config/validar.rs
+++ b/crates/tc2/src/config/validar.rs
@@ -6,66 +6,70 @@ use serde_json::Value;
 
 use super::ErrorConfig;
 use super::claves::{CLAVES_016, Campo, Nodo, Regla};
+use crate::i18n::catalogo::config as t;
+use crate::i18n::{Mensaje, Texto};
 use crate::plataforma::http::Base;
 
 /// Errores de `valor` frente a `nodo`, con `ruta` como prefijo.
 pub(crate) fn validar(nodo: &Nodo, valor: &Value, ruta: &str, errores: &mut Vec<ErrorConfig>) {
-    let mut err = |mensaje: String| errores.push(ErrorConfig::nuevo(ruta, mensaje));
+    let mut err = |mensaje: Mensaje| errores.push(ErrorConfig::nuevo(ruta, mensaje));
     match nodo {
         Nodo::Entero { min, max } => {
             let ok = valor
                 .as_u64()
                 .is_some_and(|n| n >= *min && max.is_none_or(|m| n <= m));
             if !ok {
-                err(match max {
-                    Some(m) if m == min => format!("se esperaba {min}, no {}", corto(valor)),
-                    Some(m) => format!("se esperaba un entero de {min} a {m}, no {}", corto(valor)),
-                    None => format!("se esperaba un entero de {min} o mas, no {}", corto(valor)),
-                });
+                let m = match max {
+                    Some(m) if m == min => {
+                        Mensaje::nuevo(&t::SE_ESPERABA).con("esperado", min.to_string())
+                    }
+                    Some(m) => Mensaje::nuevo(&t::ENTERO_EN_RANGO)
+                        .con("min", min.to_string())
+                        .con("max", m.to_string()),
+                    None => Mensaje::nuevo(&t::ENTERO_DESDE).con("min", min.to_string()),
+                };
+                err(con_valor(m, valor));
             }
         }
         Nodo::Booleano { solo } => match (solo, valor.as_bool()) {
             (Some(b), Some(x)) if *b == x => {}
-            (Some(b), _) => err(format!("solo admite {b}")),
+            (Some(b), _) => err(Mensaje::nuevo(&t::SOLO_ADMITE).con("valor", b.to_string())),
             (None, Some(_)) => {}
-            (None, None) => err(format!("se esperaba true o false, no {}", corto(valor))),
+            (None, None) => err(con_valor(Mensaje::nuevo(&t::SE_ESPERABA_BOOLEANO), valor)),
         },
         Nodo::Texto { valores, regla } => {
-            let Some(t) = valor.as_str() else {
-                err(format!("se esperaba un texto, no {}", corto(valor)));
+            let Some(s) = valor.as_str() else {
+                err(con_valor(Mensaje::nuevo(&t::SE_ESPERABA_TEXTO), valor));
                 return;
             };
-            if !valores.is_empty() && !valores.contains(&t) {
-                err(format!(
-                    "valor no admitido «{t}» (admite: {})",
-                    valores.join(", ")
-                ));
-            } else if let Some(m) = comprobar(*regla, t) {
+            if !valores.is_empty() && !valores.contains(&s) {
+                err(Mensaje::nuevo(&t::VALOR_NO_ADMITIDO)
+                    .con("valor", s)
+                    .con("admite", valores.join(", ")));
+            } else if let Some(m) = comprobar(*regla, s) {
                 err(m);
             }
         }
         Nodo::Lista { de, min, primero } => {
             let Some(l) = valor.as_array() else {
-                err(format!("se esperaba una lista, no {}", corto(valor)));
+                err(con_valor(Mensaje::nuevo(&t::SE_ESPERABA_LISTA), valor));
                 return;
             };
             if l.len() < *min {
-                err(format!(
-                    "se esperaba una lista con al menos {min} elemento(s)"
-                ));
+                err(Mensaje::nuevo(&t::LISTA_CORTA).con("min", min.to_string()));
             }
             if let Some(p) = primero
                 && l.first().and_then(Value::as_str) != Some(p)
             {
-                err(format!("el primer elemento tiene que ser «{p}»"));
+                err(Mensaje::nuevo(&t::PRIMER_ELEMENTO).con("valor", *p));
             }
             let textos: Vec<&str> = l.iter().filter_map(Value::as_str).collect();
             if let Some(d) = textos
                 .iter()
                 .enumerate()
-                .find_map(|(i, t)| textos.get(..i).is_some_and(|a| a.contains(t)).then_some(t))
+                .find_map(|(i, x)| textos.get(..i).is_some_and(|a| a.contains(x)).then_some(x))
             {
-                err(format!("«{d}» repetido"));
+                err(Mensaje::nuevo(&t::REPETIDO).con("valor", *d));
             }
             for (i, x) in l.iter().enumerate() {
                 validar(de, x, &format!("{ruta}[{i}]"), errores);
@@ -73,7 +77,7 @@ pub(crate) fn validar(nodo: &Nodo, valor: &Value, ruta: &str, errores: &mut Vec<
         }
         Nodo::Objeto(campos) => {
             let Some(m) = valor.as_object() else {
-                err(format!("se esperaba un mapa, no {}", corto(valor)));
+                err(con_valor(Mensaje::nuevo(&t::SE_ESPERABA_MAPA), valor));
                 return;
             };
             for (k, x) in m {
@@ -87,10 +91,7 @@ pub(crate) fn validar(nodo: &Nodo, valor: &Value, ruta: &str, errores: &mut Vec<
                 .iter()
                 .filter(|c| c.obligatorio && !m.contains_key(c.nombre))
             {
-                errores.push(ErrorConfig::nuevo(
-                    &unir(ruta, c.nombre),
-                    "falta la clave obligatoria",
-                ));
+                errores.push(ErrorConfig::nuevo(&unir(ruta, c.nombre), &t::FALTA_CLAVE));
             }
         }
         Nodo::UnaDe(formas) => {
@@ -100,11 +101,17 @@ pub(crate) fn validar(nodo: &Nodo, valor: &Value, ruta: &str, errores: &mut Vec<
                 e.is_empty()
             });
             if !encaja {
-                let resumen: Vec<String> = formas.iter().map(resumir).collect();
-                err(format!(
-                    "no encaja con ninguna forma admitida: {}",
-                    resumen.join(" o ")
-                ));
+                // «a o b o c»: cada « o » es una entrada del catalogo.
+                let resumen = formas.iter().map(resumir).reduce(|a, b| {
+                    Mensaje::nuevo(&t::FORMA_O_FORMA)
+                        .con_mensaje("a", a)
+                        .con_mensaje("b", b)
+                });
+                let m = Mensaje::nuevo(&t::NO_ENCAJA);
+                err(match resumen {
+                    Some(r) => m.con_mensaje("formas", r),
+                    None => m.con("formas", ""),
+                });
             }
         }
     }
@@ -118,11 +125,11 @@ pub(crate) fn cruzadas(raiz: &Value, errores: &mut Vec<ErrorConfig>) {
             match d.get("tipo").and_then(Value::as_str) {
                 Some("genera") if !escribe => errores.push(ErrorConfig::nuevo(
                     &format!("derivados[{i}].escribe"),
-                    "obligatoria en un derivado de tipo genera",
+                    &t::OBLIGATORIA_EN_GENERA,
                 )),
                 Some("comprueba") if escribe => errores.push(ErrorConfig::nuevo(
                     &format!("derivados[{i}].escribe"),
-                    "prohibida en un derivado de tipo comprueba",
+                    &t::PROHIBIDA_EN_COMPRUEBA,
                 )),
                 _ => {}
             }
@@ -135,7 +142,7 @@ pub(crate) fn cruzadas(raiz: &Value, errores: &mut Vec<ErrorConfig>) {
                 if vistos.contains(&id) {
                     errores.push(ErrorConfig::nuevo(
                         &format!("recursos[{i}].id"),
-                        format!("«{id}» repetido"),
+                        Mensaje::nuevo(&t::REPETIDO).con("valor", id),
                     ));
                 }
                 vistos.push(id);
@@ -168,12 +175,11 @@ pub(crate) fn aplicar_defectos(nodo: &Nodo, valor: &mut Value) {
     }
 }
 
-fn no_admitida(ruta: &str, clave: &str) -> String {
+fn no_admitida(ruta: &str, clave: &str) -> &'static Texto {
     if ruta.is_empty() && CLAVES_016.contains(&clave) {
-        "clave no admitida: es de la configuracion de la 0.16 (tc2 actualizar la traduce)"
-            .to_owned()
+        &t::CLAVE_NO_ADMITIDA_016
     } else {
-        "clave no admitida".to_owned()
+        &t::CLAVE_NO_ADMITIDA
     }
 }
 
@@ -185,75 +191,82 @@ fn unir(ruta: &str, clave: &str) -> String {
     }
 }
 
-/// El valor en el mensaje, corto y sin saltos de linea.
-fn corto(v: &Value) -> String {
+/// Rellena el hueco `{valor}` de `m` con el valor, corto y sin saltos de
+/// linea: un texto tal cual entre «», un numero o booleano tal cual, y lo
+/// demas («nada», «una lista», «un mapa») del catalogo.
+fn con_valor(m: Mensaje, v: &Value) -> Mensaje {
     let s = match v {
-        Value::Null => "nada".to_owned(),
-        Value::String(t) => format!("«{t}»"),
-        Value::Array(_) => "una lista".to_owned(),
-        Value::Object(_) => "un mapa".to_owned(),
+        Value::Null => return m.con_mensaje("valor", &t::NADA),
+        Value::Array(_) => return m.con_mensaje("valor", &t::UNA_LISTA),
+        Value::Object(_) => return m.con_mensaje("valor", &t::UN_MAPA),
+        Value::String(s) => format!("«{s}»"),
         otro => otro.to_string(),
     };
     let s: String = s.chars().filter(|c| !c.is_control()).take(60).collect();
-    s
+    m.con("valor", s)
 }
 
-fn resumir(n: &Nodo) -> String {
+fn resumir(n: &Nodo) -> Mensaje {
     match n {
-        Nodo::Objeto(c) => format!(
-            "un mapa con {}",
+        Nodo::Objeto(c) => Mensaje::nuevo(&t::UN_MAPA_CON).con(
+            "claves",
             c.iter()
                 .map(|x: &Campo| x.nombre)
                 .collect::<Vec<_>>()
-                .join(", ")
+                .join(", "),
         ),
         Nodo::Lista {
             primero: Some(p), ..
-        } => format!("una lista que empieza por {p}"),
-        Nodo::Lista { .. } => "una lista".to_owned(),
-        _ => "un valor".to_owned(),
+        } => Mensaje::nuevo(&t::LISTA_QUE_EMPIEZA).con("primero", *p),
+        Nodo::Lista { .. } => Mensaje::nuevo(&t::UNA_LISTA),
+        _ => Mensaje::nuevo(&t::UN_VALOR),
     }
 }
 
-/// Mensaje de error si `t` no cumple `regla`.
-pub(crate) fn comprobar(regla: Regla, t: &str) -> Option<String> {
-    let control = t.chars().any(char::is_control);
+/// Mensaje de error si `s` no cumple `regla`.
+pub(crate) fn comprobar(regla: Regla, s: &str) -> Option<Mensaje> {
+    let control = s.chars().any(char::is_control);
     match regla {
         Regla::Valor => None,
-        Regla::Nombre => (t.is_empty() || control || t.trim() != t).then(|| {
-            "texto vacio, con espacios en los extremos o con caracteres de control".to_owned()
-        }),
+        Regla::Nombre => {
+            (s.is_empty() || control || s.trim() != s).then(|| Mensaje::nuevo(&t::TEXTO_NO_NOMBRE))
+        }
         Regla::Orden => {
-            if t.trim().is_empty() {
-                Some("orden vacia".to_owned())
-            } else if t.contains(['\n', '\r']) {
-                Some("la orden tiene saltos de linea: escribela en una sola linea".to_owned())
+            if s.trim().is_empty() {
+                Some(Mensaje::nuevo(&t::ORDEN_VACIA))
+            } else if s.contains(['\n', '\r']) {
+                Some(Mensaje::nuevo(&t::ORDEN_CON_SALTOS))
             } else if control {
-                Some("la orden tiene un caracter de control (¿un \\v de una ruta de Windows entre comillas dobles? escribela entre comillas simples o con /)".to_owned())
+                Some(Mensaje::nuevo(&t::ORDEN_CON_CONTROL))
             } else {
                 None
             }
         }
         Regla::Glob | Regla::GlobAcotado => {
-            if let Some(m) = glob_malo(t) {
-                Some(format!("glob no admitido «{t}»: {m}"))
-            } else if regla == Regla::GlobAcotado && todo_el_arbol(t) {
-                Some(format!("glob no admitido «{t}»: nunca todo el arbol"))
+            let motivo = if let Some(m) = glob_malo(s) {
+                Some(m)
+            } else if regla == Regla::GlobAcotado && todo_el_arbol(s) {
+                Some(&t::NUNCA_TODO_EL_ARBOL)
             } else {
                 None
-            }
+            };
+            motivo.map(|m| {
+                Mensaje::nuevo(&t::GLOB_NO_ADMITIDO)
+                    .con("glob", s)
+                    .con_mensaje("motivo", m)
+            })
         }
-        Regla::RutaTareas => ruta_tareas(t)
-            .err()
-            .map(|m| format!("ruta no admitida «{t}»: {m}")),
-        Regla::Cron => (!cron_valido(t)).then(|| {
-            format!(
-                "cron no admitido «{t}»: se esperan 5 campos (minuto 0-59, hora 0-23, dia 1-31, mes 1-12, dia de la semana 0-7) con *, a, a-b, ',' y /n"
-            )
+        Regla::RutaTareas => ruta_tareas(s).err().map(|m| {
+            Mensaje::nuevo(&t::RUTA_NO_ADMITIDA)
+                .con("ruta", s)
+                .con_mensaje("motivo", m)
         }),
-        Regla::Url => Base::nueva(t)
+        Regla::Cron => {
+            (!cron_valido(s)).then(|| Mensaje::nuevo(&t::CRON_NO_ADMITIDO).con("cron", s))
+        }
+        Regla::Url => Base::nueva(s)
             .err()
-            .map(|m| format!("URL no admitida: {m}")),
+            .map(|m| Mensaje::nuevo(&t::URL_NO_ADMITIDA).con_mensaje("motivo", m)),
     }
 }
 
@@ -302,17 +315,17 @@ fn cron_valido(t: &str) -> bool {
         })
 }
 
-fn glob_malo(g: &str) -> Option<&'static str> {
+fn glob_malo(g: &str) -> Option<&'static Texto> {
     if g.is_empty() {
-        Some("vacio")
+        Some(&t::VACIO)
     } else if g.chars().any(char::is_control) {
-        Some("caracter de control")
+        Some(&t::CARACTER_DE_CONTROL)
     } else if g.starts_with('!') {
-        Some("sin negaciones")
+        Some(&t::SIN_NEGACIONES)
     } else if g.starts_with(['/', '\\']) || g.as_bytes().get(1) == Some(&b':') {
-        Some("sin rutas absolutas")
+        Some(&t::SIN_RUTAS_ABSOLUTAS)
     } else if g.split(['/', '\\']).any(|p| p == "..") {
-        Some("sin ..")
+        Some(&t::SIN_PUNTO_PUNTO)
     } else {
         None
     }
@@ -320,32 +333,30 @@ fn glob_malo(g: &str) -> Option<&'static str> {
 
 /// Normaliza una ruta de `derivados[].escribe` (F1-11, decision 1): sin
 /// segmentos `.` ni vacios y con `..` resuelto; bajo `tareas/`.
-pub(crate) fn ruta_tareas(t: &str) -> Result<String, &'static str> {
-    if t.contains(['\\', ':']) {
-        return Err("sin \\ ni :");
+pub(crate) fn ruta_tareas(r: &str) -> Result<String, &'static Texto> {
+    if r.contains(['\\', ':']) {
+        return Err(&t::SIN_BARRA_NI_DOS_PUNTOS);
     }
-    if t.contains(['*', '?', '[', ']', '{', '}']) {
-        return Err("sin comodines");
+    if r.contains(['*', '?', '[', ']', '{', '}']) {
+        return Err(&t::SIN_COMODINES);
     }
-    if t.chars().any(char::is_control) {
-        return Err("caracter de control");
+    if r.chars().any(char::is_control) {
+        return Err(&t::CARACTER_DE_CONTROL);
     }
-    if t.starts_with('/') {
-        return Err("sin rutas absolutas");
+    if r.starts_with('/') {
+        return Err(&t::SIN_RUTAS_ABSOLUTAS);
     }
     let mut partes: Vec<&str> = Vec::new();
-    for p in t.split('/') {
+    for p in r.split('/') {
         match p {
             "" | "." => {}
             ".." => {
                 if partes.pop().is_none() {
-                    return Err("sale de la raiz");
+                    return Err(&t::SALE_DE_LA_RAIZ);
                 }
             }
             p if p.ends_with(['.', ' ']) || nombre_corto(p) => {
-                return Err(
-                    "sin punto ni espacio al final de un segmento ni nombres cortos con ~ (Windows los ignora o los acorta)",
-                );
+                return Err(&t::SEGMENTO_WINDOWS);
             }
             p => partes.push(p),
         }
@@ -354,24 +365,24 @@ pub(crate) fn ruta_tareas(t: &str) -> Result<String, &'static str> {
         .iter()
         .any(|p| crate::git::estado::reservado_en_windows(p))
     {
-        return Err("nombre reservado de Windows");
+        return Err(&t::NOMBRE_RESERVADO_WINDOWS);
     }
     if partes.iter().any(|p| p.eq_ignore_ascii_case(".git")) {
-        return Err("nunca .git");
+        return Err(&t::NUNCA_GIT);
     }
     match partes.as_slice() {
         ["tareas", resto @ ..] if !resto.is_empty() => {
             let segunda = resto.first().copied().unwrap_or("");
             if resto.len() == 1 && segunda.eq_ignore_ascii_case("ROADMAP.md") {
-                Err("tareas/ROADMAP.md es reservada")
+                Err(&t::ROADMAP_RESERVADA)
             } else if es_carpeta_de_tarea(segunda) {
-                Err("las carpetas tareas/TASK-NNN/ son reservadas")
+                Err(&t::CARPETAS_DE_TAREA_RESERVADAS)
             } else {
                 Ok(partes.join("/"))
             }
         }
-        ["tareas"] => Err("nunca tareas/ entera"),
-        _ => Err("tiene que ir bajo tareas/"),
+        ["tareas"] => Err(&t::NUNCA_TAREAS_ENTERA),
+        _ => Err(&t::BAJO_TAREAS),
     }
 }
 
@@ -395,8 +406,9 @@ mod tests {
     #[test]
     fn reglas_de_texto() {
         assert!(
-            comprobar(Regla::Orden, "scripts\u{b}erificar.bat")
-                .is_some_and(|m| m.contains("comillas simples"))
+            comprobar(Regla::Orden, "scripts\u{b}erificar.bat").is_some_and(|m| m
+                .texto(crate::i18n::Idioma::Es)
+                .contains("comillas simples"))
         );
         assert!(comprobar(Regla::Orden, "  ").is_some());
         assert!(comprobar(Regla::Orden, "cargo test").is_none());
@@ -439,7 +451,7 @@ mod tests {
             ("tareas/*.md", "comodines"),
             ("/tareas/x", "absolutas"),
         ] {
-            let e = ruta_tareas(r).err().unwrap_or("ok");
+            let e = ruta_tareas(r).err().map_or("ok", |m| m.es);
             assert!(e.contains(m), "{r}: {e}");
         }
         assert_eq!(
diff --git a/crates/tc2/src/contrato.rs b/crates/tc2/src/contrato.rs
index 0c22af6..cc5c1df 100644
--- a/crates/tc2/src/contrato.rs
+++ b/crates/tc2/src/contrato.rs
@@ -10,6 +10,7 @@
 //! 5 agotada, 6 conflicto, 7 esperando, 8 denegada).
 
 use crate::Salida;
+use crate::i18n::{Idioma, Mensaje};
 
 /// Resultado de una operacion. Cada uno lleva su codigo de salida.
 #[derive(Debug, Clone, Copy, PartialEq, Eq)]
@@ -94,7 +95,8 @@ pub struct Informe {
     /// Claves en el orden en que se escriben.
     pub claves: Vec<(String, Valor)>,
     /// Una linea que dice que hacer; solo se muestra si no fue un exito.
-    pub diagnostico: Option<String>,
+    /// Es un [`Mensaje`] neutro: se traduce al escribir la salida (F2-8).
+    pub diagnostico: Option<Mensaje>,
 }
 
 impl Informe {
@@ -123,10 +125,10 @@ impl Informe {
         self
     }
 
-    /// Fija el diagnostico (una linea).
+    /// Fija el diagnostico (una linea), como mensaje del catalogo.
     #[must_use]
-    pub fn diagnostico(mut self, texto: impl Into<String>) -> Self {
-        self.diagnostico = Some(texto.into());
+    pub fn diagnostico(mut self, mensaje: impl Into<Mensaje>) -> Self {
+        self.diagnostico = Some(mensaje.into());
         self
     }
 
@@ -156,8 +158,15 @@ impl Informe {
         s
     }
 
-    /// La misma informacion en JSON, en una linea.
+    /// La misma informacion en JSON, en una linea, con el diagnostico en
+    /// espanol.
     pub fn json(&self) -> String {
+        self.json_en(Idioma::Es)
+    }
+
+    /// La misma informacion en JSON, en una linea, con el diagnostico en el
+    /// idioma pedido. Las claves y sus valores no cambian con el idioma.
+    pub fn json_en(&self, idioma: Idioma) -> String {
         let mut s = format!(
             "{{\"comando\":{},\"resultado\":{},\"codigo\":{},\"claves\":{{",
             cadena_json(&self.comando),
@@ -183,25 +192,40 @@ impl Informe {
         s.push('}');
         if let Some(d) = self.diagnostico_visible() {
             s.push_str(",\"diagnostico\":");
-            s.push_str(&cadena_json(d));
+            s.push_str(&cadena_json(&d.texto(idioma)));
         }
         s.push('}');
         s
     }
 
-    fn diagnostico_visible(&self) -> Option<&str> {
+    fn diagnostico_visible(&self) -> Option<&Mensaje> {
         if self.resultado.es_exito() {
             None
         } else {
-            self.diagnostico.as_deref()
+            self.diagnostico.as_ref()
         }
     }
 
-    /// Lo convierte en la salida del proceso.
+    /// El diagnostico en un idioma, si lo hay (aunque fuera un exito).
+    pub fn diagnostico_en(&self, idioma: Idioma) -> Option<String> {
+        self.diagnostico.as_ref().map(|d| d.texto(idioma))
+    }
+
+    /// Lo convierte en la salida del proceso, con el diagnostico en espanol.
     pub fn salida(&self, json: bool) -> Salida {
-        let stdout = if json { self.json() } else { self.linea() };
+        self.salida_en(json, Idioma::Es)
+    }
+
+    /// Lo convierte en la salida del proceso, con el diagnostico en el idioma
+    /// pedido. stdout y el codigo no cambian con el idioma.
+    pub fn salida_en(&self, json: bool, idioma: Idioma) -> Salida {
+        let stdout = if json {
+            self.json_en(idioma)
+        } else {
+            self.linea()
+        };
         let stderr = match self.diagnostico_visible() {
-            Some(d) => format!("{}\n", una_linea(d)),
+            Some(d) => format!("{}\n", una_linea(&d.texto(idioma))),
             None => String::new(),
         };
         Salida {
@@ -291,7 +315,7 @@ mod tests {
             .con("commit", corto("48d72d1e513bffffffffffffffffffffffffffff"))
             .con("nota", "con espacio y 100%")
             .con_lista("ficheros", vec!["a,b.txt".into(), "c.txt".into()])
-            .diagnostico("no se ve en un exito");
+            .diagnostico(Mensaje::externo("no se ve en un exito"));
         let s = i.salida(false);
         assert_eq!(s.codigo, 0);
         assert_eq!(
@@ -312,17 +336,48 @@ mod tests {
     fn el_diagnostico_va_a_stderr_solo_si_falla() {
         let s = Informe::nuevo("push", Resultado::Rechazada)
             .con("motivo", "no-fast-forward")
-            .diagnostico("lanza\nactualizar-rama")
+            .diagnostico(Mensaje::externo("lanza\nactualizar-rama"))
             .salida(false);
         assert_eq!(s.codigo, 3);
         assert_eq!(s.stdout, "push rechazada motivo=no-fast-forward\n");
         assert_eq!(s.stderr, "lanza actualizar-rama\n");
         let j = Informe::nuevo("x", Resultado::Uso)
-            .diagnostico("di \"algo\"")
+            .diagnostico(Mensaje::externo("di \"algo\""))
             .salida(true);
         assert!(
             j.stdout
                 .ends_with(",\"diagnostico\":\"di \\\"algo\\\"\"}\n")
         );
     }
+
+    #[test]
+    fn mismo_informe_dos_idiomas() {
+        let i = Informe::nuevo("guardar", Resultado::Uso)
+            .con("motivo", "sin-mensaje")
+            .diagnostico(&crate::i18n::catalogo::cli::USO_GUARDAR);
+        for json in [false, true] {
+            let es = i.salida_en(json, Idioma::Es);
+            let en = i.salida_en(json, Idioma::En);
+            assert_eq!(es.codigo, en.codigo);
+            assert_eq!(es.codigo, 2);
+            // stderr cambia con el idioma y no con --json.
+            assert_eq!(es.stderr, "uso: guardar \"<mensaje>\" <rutas...>\n");
+            assert_eq!(en.stderr, "usage: guardar \"<message>\" <paths...>\n");
+            if json {
+                assert!(es.stdout.contains("\"diagnostico\":\"uso: guardar"));
+                assert!(en.stdout.contains("\"diagnostico\":\"usage: guardar"));
+                // Lo demas del JSON es igual en los dos idiomas.
+                let sin_diag = |t: &str| {
+                    t.split(",\"diagnostico\"")
+                        .next()
+                        .unwrap_or_default()
+                        .to_owned()
+                };
+                assert_eq!(sin_diag(&es.stdout), sin_diag(&en.stdout));
+            } else {
+                assert_eq!(es.stdout, en.stdout);
+                assert_eq!(es.stdout, "guardar uso motivo=sin-mensaje\n");
+            }
+        }
+    }
 }
diff --git a/crates/tc2/src/estado/lectura.rs b/crates/tc2/src/estado/lectura.rs
index 64db458..b5d8449 100644
--- a/crates/tc2/src/estado/lectura.rs
+++ b/crates/tc2/src/estado/lectura.rs
@@ -82,7 +82,15 @@ impl Estado for Vista<'_> {
         match &self.commit {
             Some(c) if self.rutas.contains(ruta) => leer_blob(self.git, &self.dir, OP, c, ruta)
                 .map(Some)
-                .map_err(|i| Fallo::Error(i.diagnostico.clone().unwrap_or_else(|| i.linea()))),
+                .map_err(|i| {
+                    // Sin diagnostico queda la linea de contrato, que no se
+                    // traduce (ADR F1-4).
+                    Fallo::Error(
+                        i.diagnostico
+                            .clone()
+                            .unwrap_or_else(|| crate::i18n::Mensaje::externo(i.linea())),
+                    )
+                }),
             _ => Ok(None),
         }
     }
@@ -102,6 +110,9 @@ pub fn leer<'a>(git: &'a Git, dir: &Path) -> Result<Vista<'a>, Informe> {
         Some(c) => Vista::nueva(git, &principal, Some(c)),
         None => Err(Informe::nuevo(OP, Resultado::Rechazada)
             .con("motivo", "sin-rama-de-estado")
-            .diagnostico(format!("monta la rama de estado en {CARPETA}/ primero"))),
+            .diagnostico(
+                crate::i18n::Mensaje::nuevo(&crate::i18n::catalogo::estado::MONTA_PRIMERO)
+                    .con("carpeta", CARPETA),
+            )),
     }
 }
diff --git a/crates/tc2/src/estado/montaje.rs b/crates/tc2/src/estado/montaje.rs
index 03c77c9..1559b84 100644
--- a/crates/tc2/src/estado/montaje.rs
+++ b/crates/tc2/src/estado/montaje.rs
@@ -18,6 +18,8 @@ use crate::git::{
     Cambio, FalloRemoto, Git, REMOTO, Subcomando, clasificar_fallo_remoto, commit_de, correr,
     es_ancestro, exigir, localizar_principal, misma_ruta, worktrees,
 };
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::estado as txt;
 
 const OP: &str = "montar-estado";
 
@@ -83,10 +85,11 @@ fn paso(git: &Git, dir: &Path, politica: Politica, puede_crear: bool) -> Result<
         Err(ErrorCerrojo::Ocupado) => {
             return Ok(Informe::nuevo(OP, Resultado::Agotada)
                 .con("motivo", "cerrojo")
-                .diagnostico("otro proceso de este clon esta montando o escribiendo la rama de estado: reintenta"));
+                .diagnostico(&txt::CERROJO_OCUPADO_AL_MONTAR));
         }
         Err(ErrorCerrojo::Io(e)) => {
-            return Err(Informe::nuevo(OP, Resultado::Error).diagnostico(format!("cerrojo: {e}")));
+            return Err(Informe::nuevo(OP, Resultado::Error)
+                .diagnostico(Mensaje::nuevo(&txt::CERROJO_ERROR).con("error", e.to_string())));
         }
     };
     let carpeta = principal.join(CARPETA);
@@ -111,10 +114,11 @@ fn paso(git: &Git, dir: &Path, politica: Politica, puede_crear: bool) -> Result<
             return Ok(Informe::nuevo(OP, Resultado::Rechazada)
                 .con("rama", RAMA)
                 .con("motivo", "montada-en-otra-ruta")
-                .diagnostico(format!(
-                    "{RAMA} ya esta montada en {}: quita ese worktree",
-                    w.ruta.display()
-                )));
+                .diagnostico(
+                    Mensaje::nuevo(&txt::MONTADA_EN_OTRA_RUTA)
+                        .con("rama", RAMA)
+                        .con("ruta", w.ruta.display().to_string()),
+                ));
         }
     }
     let ocupada = std::fs::read_dir(&carpeta)
@@ -124,9 +128,7 @@ fn paso(git: &Git, dir: &Path, politica: Politica, puede_crear: bool) -> Result<
         return Ok(Informe::nuevo(OP, Resultado::Rechazada)
             .con("ruta", CARPETA)
             .con("motivo", "tareas-ocupada")
-            .diagnostico(format!(
-                "{CARPETA}/ ya existe y no es la rama de estado (¿un repo de la 0.16? migralo con actualizar)"
-            )));
+            .diagnostico(Mensaje::nuevo(&txt::CARPETA_OCUPADA).con("carpeta", CARPETA)));
     }
 
     let remoto = hay_remoto(git, &principal, OP)?;
@@ -152,10 +154,10 @@ fn paso(git: &Git, dir: &Path, politica: Politica, puede_crear: bool) -> Result<
                     FalloRemoto::SinRed => {
                         return Ok(Informe::nuevo(OP, Resultado::Sinred)
                             .con("pendiente", "no")
-                            .diagnostico(format!(
-                                "{}; montar la rama de estado exige red la primera vez",
-                                s.primera_linea_error()
-                            )));
+                            .diagnostico(
+                                Mensaje::nuevo(&txt::MONTAR_EXIGE_RED)
+                                    .con("error", s.primera_linea_error()),
+                            ));
                     }
                     FalloRemoto::Local => {
                         return Err(crate::git::remoto::error_local(
@@ -166,10 +168,10 @@ fn paso(git: &Git, dir: &Path, politica: Politica, puede_crear: bool) -> Result<
                     otro => {
                         return Ok(Informe::nuevo(OP, Resultado::Denegada)
                             .con("motivo", otro.motivo())
-                            .diagnostico(format!(
-                                "{}; revisa credenciales, permisos o la URL de origin",
-                                s.primera_linea_error()
-                            )));
+                            .diagnostico(
+                                Mensaje::nuevo(&txt::MONTAR_DENEGADA)
+                                    .con("error", s.primera_linea_error()),
+                            ));
                     }
                 }
             }
@@ -179,7 +181,7 @@ fn paso(git: &Git, dir: &Path, politica: Politica, puede_crear: bool) -> Result<
         } else {
             if !puede_crear {
                 return Err(Informe::nuevo(OP, Resultado::Error)
-                    .diagnostico(format!("{RAMA} sigue sin existir tras crearla")));
+                    .diagnostico(Mensaje::nuevo(&txt::RAMA_SIGUE_SIN_EXISTIR).con("rama", RAMA)));
             }
             // La transaccion toma el cerrojo ella misma: se suelta antes y,
             // creada la rama, se vuelve a empezar para montarla.
@@ -196,7 +198,7 @@ fn paso(git: &Git, dir: &Path, politica: Politica, puede_crear: bool) -> Result<
     }
 
     let ruta = carpeta.to_str().ok_or_else(|| {
-        Informe::nuevo(OP, Resultado::Error).diagnostico("la ruta de tareas/ no es UTF-8")
+        Informe::nuevo(OP, Resultado::Error).diagnostico(&txt::RUTA_TAREAS_NO_UTF8)
     })?;
     exigir(
         git,
diff --git a/crates/tc2/src/estado/transaccion.rs b/crates/tc2/src/estado/transaccion.rs
index 01e435c..3439fd0 100644
--- a/crates/tc2/src/estado/transaccion.rs
+++ b/crates/tc2/src/estado/transaccion.rs
@@ -23,6 +23,8 @@ use crate::git::{
     Cambio, FalloRemoto, Git, REMOTO, Subcomando, clasificar_fallo_remoto, commit_de, correr,
     es_ancestro, exigir, localizar_principal, worktrees,
 };
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::estado as txt;
 
 /// Por que una operacion no produce cambios.
 #[derive(Debug, Clone, PartialEq, Eq)]
@@ -32,16 +34,16 @@ pub enum Fallo {
     Rechazada {
         /// Valor de `motivo=`.
         motivo: String,
-        /// Que hacer, en una linea.
-        diagnostico: String,
+        /// Que hacer, en una linea (mensaje del catalogo, F2-8).
+        diagnostico: Mensaje,
     },
     /// Fallo interno: `error` (1).
-    Error(String),
+    Error(Mensaje),
 }
 
 impl Fallo {
     /// Atajo para [`Fallo::Rechazada`].
-    pub fn rechazada(motivo: impl Into<String>, diagnostico: impl Into<String>) -> Self {
+    pub fn rechazada(motivo: impl Into<String>, diagnostico: impl Into<Mensaje>) -> Self {
         Self::Rechazada {
             motivo: motivo.into(),
             diagnostico: diagnostico.into(),
@@ -152,7 +154,7 @@ fn informe(r: Resultado, op: &dyn Operacion) -> Informe {
     Informe::nuevo(OP, r).con("operacion", op.nombre())
 }
 
-fn error(diagnostico: impl Into<String>) -> Informe {
+fn error(diagnostico: impl Into<Mensaje>) -> Informe {
     Informe::nuevo(OP, Resultado::Error).diagnostico(diagnostico)
 }
 
@@ -190,11 +192,13 @@ fn ejecutar(c: &Contexto<'_>, op: &dyn Operacion) -> Result<Informe, Informe> {
             Err(ErrorCerrojo::Ocupado) => {
                 return Ok(informe(Resultado::Agotada, op)
                     .con("motivo", "cerrojo")
-                    .diagnostico(
-                        "otro proceso de este clon esta escribiendo la rama de estado: reintenta",
-                    ));
+                    .diagnostico(&txt::CERROJO_OCUPADO));
+            }
+            Err(ErrorCerrojo::Io(e)) => {
+                return Err(error(
+                    Mensaje::nuevo(&txt::CERROJO_ERROR).con("error", e.to_string()),
+                ));
             }
-            Err(ErrorCerrojo::Io(e)) => return Err(error(format!("cerrojo: {e}"))),
         };
         let r = intento(c, op, &anotacion, &principal, remoto, n);
         drop(cerrojo);
@@ -203,9 +207,7 @@ fn ejecutar(c: &Contexto<'_>, op: &dyn Operacion) -> Result<Informe, Informe> {
             Intento::Reintentar if n >= c.politica.intentos => {
                 return Ok(informe(Resultado::Agotada, op)
                     .con("intentos", n.to_string())
-                    .diagnostico(
-                        "la rama de estado cambio en cada intento: no se ha escrito nada; reintenta",
-                    ));
+                    .diagnostico(&txt::AGOTADA));
             }
             Intento::Reintentar => std::thread::sleep(espera(c.politica.base, n)),
         }
@@ -245,7 +247,11 @@ fn de_fallo(f: Fallo) -> Informe {
         Fallo::Rechazada {
             motivo,
             diagnostico,
-        } => error(format!("{motivo}: {diagnostico}")),
+        } => error(
+            Mensaje::nuevo(&txt::RECHAZADA_COMO_ERROR)
+                .con("motivo", motivo)
+                .con_mensaje("diagnostico", diagnostico),
+        ),
     }
 }
 
@@ -374,9 +380,7 @@ fn intento(
             Tareas::ConCambios => Intento::Hecho(
                 informe(Resultado::Rechazada, op)
                     .con("motivo", "tareas-con-cambios")
-                    .diagnostico(format!(
-                        "{CARPETA}/ tiene cambios que chocan con la operacion: guardalos o deshazlos y reintenta"
-                    )),
+                    .diagnostico(Mensaje::nuevo(&txt::TAREAS_CON_CAMBIOS).con("carpeta", CARPETA)),
             ),
             t => Intento::Hecho(ok(op, &sha, n, t, 0, 0)),
         });
@@ -420,23 +424,23 @@ fn duplicada(rutas: &[String], cambios: &[Cambio], op: &dyn Operacion) -> Option
     choque(rutas, cambios).map(|(nueva, otra)| {
         informe(Resultado::Rechazada, op)
             .con("motivo", "ruta-duplicada")
-            .diagnostico(format!(
-                "{nueva} solo difiere en mayusculas de {otra}: chocarian en Windows y macOS"
-            ))
+            .diagnostico(
+                Mensaje::nuevo(&txt::RUTA_DUPLICADA)
+                    .con("nueva", nueva)
+                    .con("otra", otra),
+            )
     })
 }
 
 fn denegada(op: &dyn Operacion, motivo: &str, detalle: &str) -> Informe {
-    let primera = detalle
-        .lines()
-        .map(str::trim)
-        .find(|l| !l.is_empty())
-        .unwrap_or("sin mensaje");
+    let texto = Mensaje::nuevo(&txt::ESCRITURA_DENEGADA);
+    let texto = match detalle.lines().map(str::trim).find(|l| !l.is_empty()) {
+        Some(primera) => texto.con("detalle", primera),
+        None => texto.con_mensaje("detalle", &txt::SIN_MENSAJE),
+    };
     informe(Resultado::Denegada, op)
         .con("motivo", motivo)
-        .diagnostico(format!(
-            "{primera}; el remoto deniega la escritura: revisa credenciales, permisos o reglas, no se reintenta"
-        ))
+        .diagnostico(texto)
 }
 
 fn con_pendientes(mut i: Informe, publicadas: usize, rechazadas: usize) -> Informe {
@@ -481,9 +485,7 @@ fn sin_red(
         return Ok(Intento::Hecho(
             informe(Resultado::Sinred, op)
                 .con("pendiente", "no")
-                .diagnostico(format!(
-                    "{detalle}; esta operacion exige red: no se ha escrito nada, reintenta con red"
-                )),
+                .diagnostico(Mensaje::nuevo(&txt::EXIGE_RED).con("detalle", detalle)),
         ));
     }
     let git = c.git;
@@ -529,9 +531,7 @@ fn sin_red(
         informe(Resultado::Sinred, op)
             .con("pendiente", "si")
             .con("pendientes", (previas.len() + 1).to_string())
-            .diagnostico(format!(
-                "{detalle}; queda pendiente de publicar: se publicara en la siguiente operacion con red"
-            )),
+            .diagnostico(Mensaje::nuevo(&txt::QUEDA_PENDIENTE).con("detalle", detalle)),
     ))
 }
 
@@ -599,10 +599,11 @@ fn adelantar(git: &Git, principal: &Path, sha: &str) -> Result<Tareas, Informe>
             );
             let s = s?;
             if !s.exito() {
-                return Err(error(format!(
-                    "no se pudo adelantar {RAMA}: {}",
-                    s.primera_linea_error()
-                )));
+                return Err(error(
+                    Mensaje::nuevo(&txt::NO_SE_PUDO_ADELANTAR)
+                        .con("rama", RAMA)
+                        .con("error", s.primera_linea_error()),
+                ));
             }
             borrado?;
         }
diff --git a/crates/tc2/src/git/diff.rs b/crates/tc2/src/git/diff.rs
index 635204f..7ef0dc4 100644
--- a/crates/tc2/src/git/diff.rs
+++ b/crates/tc2/src/git/diff.rs
@@ -5,6 +5,7 @@ use std::path::Path;
 
 use super::{Git, Paso, Subcomando, base_de_rama, commit_de, exigir, rama_actual};
 use crate::contrato::{Informe, Resultado, corto};
+use crate::i18n::catalogo::git as txt;
 
 /// Ficheros y lineas que cambian desde el `merge-base` con la base de la rama
 /// (incluye lo no guardado del worktree; no los ficheros sin seguimiento).
@@ -17,7 +18,7 @@ fn diff_paso(git: &Git, dir: &Path) -> Paso<Informe> {
     let Some(rama) = rama_actual(git, dir, OP)? else {
         return Ok(Informe::nuevo(OP, Resultado::Rechazada)
             .con("motivo", "sin-rama")
-            .diagnostico("lanzalo en el worktree de la tarea"));
+            .diagnostico(&txt::LANZALO_EN_EL_WORKTREE));
     };
     let base = base_de_rama(&rama);
     if commit_de(git, dir, OP, base)?.is_none() {
diff --git a/crates/tc2/src/git/estado.rs b/crates/tc2/src/git/estado.rs
index 95e30da..6e997d4 100644
--- a/crates/tc2/src/git/estado.rs
+++ b/crates/tc2/src/git/estado.rs
@@ -8,8 +8,11 @@
 use std::path::{Component, Path, PathBuf};
 use std::sync::atomic::{AtomicUsize, Ordering};
 
-use super::{Git, Paso, Subcomando, correr, exigir};
+use super::rama::error_de_fichero;
+use super::{Git, Paso, Subcomando, correr, exigir, git_fallo};
 use crate::contrato::{Informe, Resultado};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::git as txt;
 
 /// Un fichero que la operacion anade o cambia en la rama de estado. No hay
 /// borrado: las carpetas de tarea no se borran nunca (ADR F1-1).
@@ -136,7 +139,7 @@ impl Temporal {
             .join(format!("{}-{n}", std::process::id()));
         let _ = std::fs::remove_dir_all(&ruta);
         std::fs::create_dir_all(&ruta).map_err(|e| {
-            Informe::nuevo(op, Resultado::Error).diagnostico(format!("{}: {e}", ruta.display()))
+            Informe::nuevo(op, Resultado::Error).diagnostico(error_de_fichero(&ruta, &e))
         })?;
         Ok(Self(ruta))
     }
@@ -156,9 +159,9 @@ impl Drop for Temporal {
 }
 
 fn texto(op: &str, p: &Path) -> Paso<String> {
-    p.to_str()
-        .map(str::to_owned)
-        .ok_or_else(|| Informe::nuevo(op, Resultado::Error).diagnostico("ruta temporal no UTF-8"))
+    p.to_str().map(str::to_owned).ok_or_else(|| {
+        Informe::nuevo(op, Resultado::Error).diagnostico(&txt::RUTA_TEMPORAL_NO_UTF8)
+    })
 }
 
 fn con_indice(
@@ -174,18 +177,14 @@ fn con_indice(
         .map_err(|e| {
             Informe::nuevo(op, Resultado::Error)
                 .con("git", sub.nombre())
-                .diagnostico(e.to_string())
+                .diagnostico(e.mensaje())
         })?;
     if s.exito() {
         Ok(s.stdout)
     } else {
         Err(Informe::nuevo(op, Resultado::Error)
             .con("git", sub.nombre())
-            .diagnostico(format!(
-                "git {} fallo: {}",
-                sub.nombre(),
-                s.primera_linea_error()
-            )))
+            .diagnostico(git_fallo(sub, &s)))
     }
 }
 
@@ -203,7 +202,9 @@ pub(crate) fn construir_commit(
     if let Some(malo) = cambios.iter().find(|c| !ruta_valida(&c.ruta)) {
         return Err(Informe::nuevo(op, Resultado::Error)
             .con("motivo", "ruta-invalida")
-            .diagnostico(format!("ruta invalida en la rama de estado: {}", malo.ruta)));
+            .diagnostico(
+                Mensaje::nuevo(&txt::RUTA_INVALIDA_EN_ESTADO).con("ruta", malo.ruta.as_str()),
+            ));
     }
     let comun = dir_comun(git, dir, op)?;
     let tmp = Temporal::nueva(&comun, op)?;
@@ -216,7 +217,7 @@ pub(crate) fn construir_commit(
         for (i, c) in cambios.iter().enumerate() {
             let f = tmp.0.join(format!("blob-{i}"));
             std::fs::write(&f, c.contenido.as_bytes()).map_err(|e| {
-                Informe::nuevo(op, Resultado::Error).diagnostico(format!("{}: {e}", f.display()))
+                Informe::nuevo(op, Resultado::Error).diagnostico(error_de_fichero(&f, &e))
             })?;
             ficheros.push(texto(op, &f)?);
         }
@@ -225,8 +226,9 @@ pub(crate) fn construir_commit(
         let shas = exigir(git, dir, op, Subcomando::HashObject, &args)?;
         let shas: Vec<&str> = shas.lines().map(str::trim).collect();
         if shas.len() != cambios.len() {
-            return Err(Informe::nuevo(op, Resultado::Error)
-                .diagnostico("git hash-object no devolvio un hash por fichero"));
+            return Err(
+                Informe::nuevo(op, Resultado::Error).diagnostico(&txt::HASH_OBJECT_INCOMPLETO)
+            );
         }
         let entradas: Vec<String> = cambios
             .iter()
diff --git a/crates/tc2/src/git/guardar.rs b/crates/tc2/src/git/guardar.rs
index de4a337..7d7588d 100644
--- a/crates/tc2/src/git/guardar.rs
+++ b/crates/tc2/src/git/guardar.rs
@@ -8,6 +8,8 @@ use std::path::{Component, Path};
 use super::remoto::{merge_abierto, sin_resolver};
 use super::{Git, Paso, Subcomando, commit_de, entradas_status, exigir};
 use crate::contrato::{Informe, Resultado, corto};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::git as txt;
 
 const OP: &str = "guardar";
 
@@ -19,7 +21,7 @@ pub fn guardar(git: &Git, dir: &Path, mensaje: &str, rutas: &[&str]) -> Informe
 fn uso(motivo: &str) -> Informe {
     Informe::nuevo(OP, Resultado::Uso)
         .con("motivo", motivo)
-        .diagnostico("uso: guardar \"<mensaje>\" <rutas...>")
+        .diagnostico(&crate::i18n::catalogo::cli::USO_GUARDAR)
 }
 
 fn guardar_paso(git: &Git, dir: &Path, mensaje: &str, rutas: &[&str]) -> Paso<Informe> {
@@ -49,10 +51,10 @@ fn guardar_paso(git: &Git, dir: &Path, mensaje: &str, rutas: &[&str]) -> Paso<In
             return Ok(Informe::nuevo(OP, Resultado::Rechazada)
                 .con("motivo", "conflicto-sin-resolver")
                 .con_lista("ficheros", pendientes.clone())
-                .diagnostico(format!(
-                    "resuelve {} y guardalo con el resto",
-                    pendientes.join(" ")
-                )));
+                .diagnostico(
+                    Mensaje::nuevo(&txt::RESUELVE_Y_GUARDA_CON_EL_RESTO)
+                        .con("ficheros", pendientes.join(" ")),
+                ));
         }
         let con_marcas: Vec<String> = rutas
             .iter()
@@ -63,10 +65,9 @@ fn guardar_paso(git: &Git, dir: &Path, mensaje: &str, rutas: &[&str]) -> Paso<In
             return Ok(Informe::nuevo(OP, Resultado::Rechazada)
                 .con("motivo", "marcas-de-conflicto")
                 .con_lista("ficheros", con_marcas.clone())
-                .diagnostico(format!(
-                    "quita las marcas de conflicto de {}",
-                    con_marcas.join(" ")
-                )));
+                .diagnostico(
+                    Mensaje::nuevo(&txt::QUITA_LAS_MARCAS).con("ficheros", con_marcas.join(" ")),
+                ));
         }
         let estado = exigir(
             git,
diff --git a/crates/tc2/src/git/mod.rs b/crates/tc2/src/git/mod.rs
index f800d6a..0b96295 100644
--- a/crates/tc2/src/git/mod.rs
+++ b/crates/tc2/src/git/mod.rs
@@ -33,6 +33,8 @@ pub use worktree::quitar_worktree;
 use std::path::{Component, Path, PathBuf};
 
 use crate::contrato::{Informe, Resultado};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::git as txt;
 
 /// Nombre del remoto (ADR F1-3).
 pub const REMOTO: &str = "origin";
@@ -54,10 +56,17 @@ pub(crate) fn correr(
     git.ejecutar(dir, sub, args).map_err(|e| {
         Informe::nuevo(op, Resultado::Error)
             .con("git", sub.nombre())
-            .diagnostico(e.to_string())
+            .diagnostico(e.mensaje())
     })
 }
 
+/// Diagnostico de un subcomando de git que termino con error.
+pub(crate) fn git_fallo(sub: Subcomando, s: &SalidaGit) -> Mensaje {
+    Mensaje::nuevo(&txt::GIT_FALLO)
+        .con("subcomando", sub.nombre())
+        .con("error", s.primera_linea_error())
+}
+
 /// Como [`correr`], pero un codigo distinto de 0 tambien es `error`.
 pub(crate) fn exigir(
     git: &Git,
@@ -72,11 +81,7 @@ pub(crate) fn exigir(
     } else {
         Err(Informe::nuevo(op, Resultado::Error)
             .con("git", sub.nombre())
-            .diagnostico(format!(
-                "git {} fallo: {}",
-                sub.nombre(),
-                s.primera_linea_error()
-            )))
+            .diagnostico(git_fallo(sub, &s)))
     }
 }
 
@@ -105,8 +110,10 @@ pub(crate) fn es_ancestro(git: &Git, dir: &Path, op: &str, a: &str, b: &str) ->
     match s.codigo {
         0 => Ok(true),
         1 => Ok(false),
-        _ => Err(Informe::nuevo(op, Resultado::Error)
-            .diagnostico(format!("git merge-base fallo: {}", s.primera_linea_error()))),
+        _ => {
+            Err(Informe::nuevo(op, Resultado::Error)
+                .diagnostico(git_fallo(Subcomando::MergeBase, &s)))
+        }
     }
 }
 
diff --git a/crates/tc2/src/git/puerto.rs b/crates/tc2/src/git/puerto.rs
index f14d8a4..ced1111 100644
--- a/crates/tc2/src/git/puerto.rs
+++ b/crates/tc2/src/git/puerto.rs
@@ -11,6 +11,9 @@ use std::ffi::OsString;
 use std::path::Path;
 use std::process::Command;
 
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::git as txt;
+
 /// Subcomandos de `git` que la capa puede lanzar (lista permitida de F1-4 B2b,
 /// reducida a los que usan las operaciones de F2-2 y la rama de estado de
 /// F2-3).
@@ -147,9 +150,18 @@ pub enum ErrorGit {
 
 impl std::fmt::Display for ErrorGit {
     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        // Una sola fuente del texto: el catalogo, en espanol.
+        f.write_str(&self.mensaje().texto(crate::i18n::Idioma::Es))
+    }
+}
+
+impl ErrorGit {
+    /// El error como mensaje del catalogo (F2-8): lo que va al diagnostico
+    /// se traduce; `Display` es su texto en espanol.
+    pub fn mensaje(&self) -> Mensaje {
         match self {
-            Self::Prohibido(a) => write!(f, "argumento prohibido para la capa: {a}"),
-            Self::Lanzar(e) => write!(f, "no se pudo lanzar git: {e}"),
+            Self::Prohibido(a) => Mensaje::nuevo(&txt::ARGUMENTO_PROHIBIDO).con("argumento", a),
+            Self::Lanzar(e) => Mensaje::nuevo(&txt::NO_SE_PUDO_LANZAR_GIT).con("error", e),
         }
     }
 }
diff --git a/crates/tc2/src/git/rama.rs b/crates/tc2/src/git/rama.rs
index 4ad5812..d07cfea 100644
--- a/crates/tc2/src/git/rama.rs
+++ b/crates/tc2/src/git/rama.rs
@@ -8,6 +8,8 @@ use super::{
     nombre_de_rama, worktree_relativo, worktrees,
 };
 use crate::contrato::{Informe, Resultado, corto};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::git as txt;
 
 /// Worktree principal: la primera entrada de `git worktree list` (ADR F1-3).
 /// Funciona lanzado desde el principal o desde cualquier worktree de tarea.
@@ -24,7 +26,7 @@ pub fn localizar_principal(git: &Git, dir: &Path) -> Result<PathBuf, Informe> {
         Some(r) => Ok(PathBuf::from(r)),
         None => Err(Informe::nuevo(OP, Resultado::Rechazada)
             .con("motivo", "no-es-repo")
-            .diagnostico("lanza el comando dentro de un repositorio git")),
+            .diagnostico(&txt::NO_ES_REPO)),
     }
 }
 
@@ -41,9 +43,7 @@ pub fn validar_base(git: &Git, dir: &Path, base: &str) -> Informe {
         Ok(None) => Informe::nuevo(OP, Resultado::Rechazada)
             .con("base", base)
             .con("motivo", "no-existe")
-            .diagnostico(format!(
-                "la base {base} no existe: lanza fetch o revisa el nombre"
-            )),
+            .diagnostico(Mensaje::nuevo(&txt::BASE_NO_EXISTE).con("base", base)),
         Err(i) => i,
     }
 }
@@ -61,7 +61,7 @@ pub fn comprobar_id(git: &Git, dir: &Path, id: &str) -> Informe {
             .con("id", id)
             .con("motivo", "ocupada")
             .con_lista("ramas", r)
-            .diagnostico(format!("{id} ya tiene rama: elige otro ID")),
+            .diagnostico(Mensaje::nuevo(&txt::ID_CON_RAMA).con("id", id)),
         Err(i) => i,
     }
 }
@@ -78,7 +78,7 @@ fn ramas_del_id(git: &Git, dir: &Path, op: &str, id: &str) -> Paso<Vec<String>>
     if !remoto.exito() {
         return Err(Informe::nuevo(op, Resultado::Sinred)
             .con("remoto", REMOTO)
-            .diagnostico(format!("{op}: no se pudo leer {REMOTO}; reintenta con red")));
+            .diagnostico(super::remoto::sin_red_remoto(op)));
     }
     let local = exigir(
         git,
@@ -121,7 +121,7 @@ fn rama_valida(op: &str, id: &str, tipo: &str, slug: &str) -> Paso<String> {
 
 fn ruta_texto(op: &str, p: &Path) -> Paso<String> {
     p.to_str().map(str::to_owned).ok_or_else(|| {
-        Informe::nuevo(op, Resultado::Error).diagnostico("la ruta del worktree no es UTF-8")
+        Informe::nuevo(op, Resultado::Error).diagnostico(&txt::RUTA_WORKTREE_NO_UTF8)
     })
 }
 
@@ -144,7 +144,7 @@ pub(crate) fn excluir(git: &Git, principal: &Path, op: &str, linea: &str) -> Pas
     let info = comun.join("info");
     let fichero = info.join("exclude");
     let error = |e: std::io::Error| {
-        Informe::nuevo(op, Resultado::Error).diagnostico(format!("{}: {e}", fichero.display()))
+        Informe::nuevo(op, Resultado::Error).diagnostico(error_de_fichero(&fichero, &e))
     };
     // Solo un fichero ausente cuenta como vacio: si no se puede leer, no se
     // reescribe (se perderia lo que la persona tenga en el).
@@ -203,11 +203,11 @@ fn crear(git: &Git, dir: &Path, id: &str, tipo: &str, slug: &str, base: &str) ->
                 .con("rama", rama.as_str())
                 .con("worktree", rel)
         } else {
-            rechazo("rama-en-otro-worktree").diagnostico("la rama ya esta abierta en otro worktree")
+            rechazo("rama-en-otro-worktree").diagnostico(&txt::RAMA_EN_OTRO_WORKTREE)
         });
     }
     if commit_de(git, &principal, OP, &format!("refs/heads/{rama}"))?.is_some() {
-        return Ok(rechazo("rama-existe").diagnostico("la rama ya existe: la recupera reanudar"));
+        return Ok(rechazo("rama-existe").diagnostico(&txt::RAMA_YA_EXISTE));
     }
     if commit_de(
         git,
@@ -217,20 +217,17 @@ fn crear(git: &Git, dir: &Path, id: &str, tipo: &str, slug: &str, base: &str) ->
     )?
     .is_some()
     {
-        return Ok(rechazo("rama-en-remoto")
-            .diagnostico("la rama ya esta publicada: la recupera reanudar"));
+        return Ok(rechazo("rama-en-remoto").diagnostico(&txt::RAMA_YA_PUBLICADA));
     }
     if wt.exists() {
         return Ok(rechazo("carpeta-ocupada")
-            .diagnostico(format!("{rel} ya existe y no es el worktree de la tarea")));
+            .diagnostico(Mensaje::nuevo(&txt::CARPETA_OCUPADA).con("ruta", rel.as_str())));
     }
     let Some(commit) = commit_de(git, &principal, OP, base)? else {
         return Ok(Informe::nuevo(OP, Resultado::Rechazada)
             .con("base", base)
             .con("motivo", "base-no-existe")
-            .diagnostico(format!(
-                "la base {base} no existe: lanza fetch o revisa el nombre"
-            )));
+            .diagnostico(Mensaje::nuevo(&txt::BASE_NO_EXISTE).con("base", base)));
     };
     excluir_worktrees(git, &principal, OP)?;
     let wt_txt = ruta_texto(OP, &wt)?;
@@ -270,8 +267,7 @@ fn reanudar_paso(git: &Git, dir: &Path, id: &str, tipo: &str, slug: &str) -> Pas
     if commit_de(git, &principal, OP, &format!("refs/heads/{rama}"))?.is_none() {
         let remota = format!("refs/remotes/{REMOTO}/{rama}");
         if commit_de(git, &principal, OP, &remota)?.is_none() {
-            return Ok(rechazo("rama-no-existe")
-                .diagnostico("la tarea no tiene rama ni en local ni en el remoto"));
+            return Ok(rechazo("rama-no-existe").diagnostico(&txt::TAREA_SIN_RAMA));
         }
         exigir(
             git,
@@ -288,13 +284,12 @@ fn reanudar_paso(git: &Git, dir: &Path, id: &str, tipo: &str, slug: &str) -> Pas
     {
         Some(w) if misma_ruta(&w.ruta, &wt) => {}
         Some(_) => {
-            return Ok(rechazo("rama-en-otro-worktree")
-                .diagnostico("la rama ya esta abierta en otro worktree"));
+            return Ok(rechazo("rama-en-otro-worktree").diagnostico(&txt::RAMA_EN_OTRO_WORKTREE));
         }
         None => {
             if wt.exists() {
                 return Ok(rechazo("carpeta-ocupada")
-                    .diagnostico(format!("{rel} ya existe y no es el worktree de la tarea")));
+                    .diagnostico(Mensaje::nuevo(&txt::CARPETA_OCUPADA).con("ruta", rel.as_str())));
             }
             excluir_worktrees(git, &principal, OP)?;
             let wt_txt = ruta_texto(OP, &wt)?;
@@ -325,6 +320,13 @@ fn reanudar_paso(git: &Git, dir: &Path, id: &str, tipo: &str, slug: &str) -> Pas
     })
 }
 
+/// «`ruta`: error» de un fallo de E/S sobre un fichero.
+pub(crate) fn error_de_fichero(ruta: &Path, e: &std::io::Error) -> Mensaje {
+    Mensaje::nuevo(&txt::ERROR_DE_FICHERO)
+        .con("ruta", ruta.display().to_string())
+        .con("error", e.to_string())
+}
+
 /// Un fallo de `localizar-principal` se informa con el nombre de la operacion
 /// que lo pidio.
 pub(crate) fn renombrar(mut i: Informe, op: &str) -> Informe {
diff --git a/crates/tc2/src/git/remoto.rs b/crates/tc2/src/git/remoto.rs
index 557e7a6..0ac781a 100644
--- a/crates/tc2/src/git/remoto.rs
+++ b/crates/tc2/src/git/remoto.rs
@@ -7,9 +7,11 @@ use std::time::Duration;
 use super::rama::renombrar;
 use super::{
     Git, Paso, REMOTO, Subcomando, base_de_rama, commit_de, correr, entradas_status, es_ancestro,
-    es_protegida, exigir, localizar_principal, rama_actual,
+    es_protegida, exigir, git_fallo, localizar_principal, rama_actual,
 };
 use crate::contrato::{Informe, Resultado, corto};
+use crate::i18n::catalogo::git as txt;
+use crate::i18n::{Mensaje, Texto};
 
 /// Esperas entre intentos de `push`: tres intentos, a 1 s y 2 s.
 pub const ESPERAS_PUSH: [Duration; 2] = [Duration::from_secs(1), Duration::from_secs(2)];
@@ -31,7 +33,14 @@ fn refs_remotas(git: &Git, dir: &Path, op: &str) -> Paso<String> {
 fn sinred(op: &str) -> Informe {
     Informe::nuevo(op, Resultado::Sinred)
         .con("remoto", REMOTO)
-        .diagnostico(format!("{op}: no se pudo leer {REMOTO}; reintenta con red"))
+        .diagnostico(sin_red_remoto(op))
+}
+
+/// «`op`: no se pudo leer origin; reintenta con red».
+pub(crate) fn sin_red_remoto(op: &str) -> Mensaje {
+    Mensaje::nuevo(&txt::SIN_RED_REMOTO)
+        .con("op", op)
+        .con("remoto", REMOTO)
 }
 
 /// Por que fallo una llamada que habla con el remoto (F2-3: responde las
@@ -118,37 +127,44 @@ pub fn clasificar_fallo_remoto(stderr: &str) -> FalloRemoto {
 
 /// Informe de un fallo del remoto que no es de red: `denegada` (8).
 pub(crate) fn denegada(op: &str, fallo: FalloRemoto, detalle: &str) -> Informe {
-    let que_hacer = match fallo {
-        FalloRemoto::Credenciales => "revisa tus credenciales o permisos en el remoto",
-        FalloRemoto::RemotoNoExiste => "revisa la URL de origin",
-        _ => "revisa las reglas del remoto o avisa a la persona",
+    let texto = match fallo {
+        FalloRemoto::Credenciales => &txt::DENEGADA_CREDENCIALES,
+        FalloRemoto::RemotoNoExiste => &txt::DENEGADA_URL,
+        _ => &txt::DENEGADA_REGLAS,
     };
     Informe::nuevo(op, Resultado::Denegada)
         .con("remoto", REMOTO)
         .con("motivo", fallo.motivo())
-        .diagnostico(format!("{op}: {detalle}; {que_hacer}"))
+        .diagnostico(Mensaje::nuevo(texto).con("op", op).con("detalle", detalle))
 }
 
 /// Fallo del propio clon al hablar con el remoto: `error` (1).
 pub(crate) fn error_local(op: &str, detalle: &str) -> Informe {
     Informe::nuevo(op, Resultado::Error)
         .con("motivo", FalloRemoto::Local.motivo())
-        .diagnostico(format!(
-            "{op}: {detalle}; fallo local de git: revisa los permisos de la carpeta del repo"
-        ))
+        .diagnostico(
+            Mensaje::nuevo(&txt::ERROR_LOCAL)
+                .con("op", op)
+                .con("detalle", detalle),
+        )
 }
 
 /// Que hacer tras un push que no salio, segun el resultado.
-pub(crate) fn que_hacer_al_publicar(r: Resultado) -> &'static str {
+pub(crate) fn que_hacer_al_publicar(r: Resultado) -> &'static Texto {
     match r {
-        Resultado::Denegada => {
-            "el remoto deniega el push: revisa sus reglas, tus permisos o avisa a la persona"
-        }
-        Resultado::Error => "fallo local de git: revisa los permisos de la carpeta del repo",
-        _ => "reintenta con red",
+        Resultado::Denegada => &txt::QUE_HACER_DENEGADA,
+        Resultado::Error => &txt::QUE_HACER_ERROR_LOCAL,
+        _ => &txt::QUE_HACER_SIN_RED,
     }
 }
 
+/// «push: `ultimo`; que hacer».
+fn push_fallo(ultimo: String, r: Resultado) -> Mensaje {
+    Mensaje::nuevo(&txt::PUSH_FALLO)
+        .con("ultimo", ultimo)
+        .con_mensaje("que_hacer", que_hacer_al_publicar(r))
+}
+
 /// Trae el remoto. Nada nuevo → `sin-cambios`; sin red → `sinred`; remoto
 /// inexistente o sin permiso → `denegada`.
 pub fn fetch(git: &Git, dir: &Path) -> Informe {
@@ -326,14 +342,14 @@ fn push_paso(git: &Git, dir: &Path, op: &str, esperas: &[Duration]) -> Paso<Info
         Resultado::SinCambios => i.con("commit", corto(&head)),
         Resultado::Rechazada => i
             .con("motivo", "no-fast-forward")
-            .diagnostico("lanza actualizar-rama"),
+            .diagnostico(&txt::LANZA_ACTUALIZAR_RAMA),
         Resultado::Denegada | Resultado::Error => i
             .con("motivo", p.motivo)
             .con("intentos", intentos.to_string())
-            .diagnostico(format!("push: {ultimo}; {}", que_hacer_al_publicar(r))),
+            .diagnostico(push_fallo(ultimo, r)),
         _ => i
             .con("intentos", intentos.to_string())
-            .diagnostico(format!("push: {ultimo}; {}", que_hacer_al_publicar(r))),
+            .diagnostico(push_fallo(ultimo, r)),
     })
 }
 
@@ -342,11 +358,11 @@ fn rama_de_tarea(git: &Git, dir: &Path, op: &str) -> Paso<String> {
     match rama_actual(git, dir, op)? {
         None => Err(Informe::nuevo(op, Resultado::Rechazada)
             .con("motivo", "sin-rama")
-            .diagnostico("lanzalo en el worktree de la tarea")),
+            .diagnostico(&txt::LANZALO_EN_EL_WORKTREE)),
         Some(r) if es_protegida(&r) => Err(Informe::nuevo(op, Resultado::Rechazada)
             .con("rama", r.as_str())
             .con("motivo", "rama-protegida")
-            .diagnostico("lanzalo en el worktree de la tarea, no en develop ni en main")),
+            .diagnostico(&txt::LANZALO_FUERA_DE_PROTEGIDA)),
         Some(r) => Ok(r),
     }
 }
@@ -385,9 +401,9 @@ fn actualizar(git: &Git, dir: &Path, esperas: &[Duration]) -> Paso<Informe> {
     let base = base_de_rama(&rama);
     let conflicto = |ficheros: Vec<String>| {
         let diag = if ficheros.is_empty() {
-            "lanza guardar con los ficheros resueltos".to_owned()
+            Mensaje::nuevo(&txt::LANZA_GUARDAR_CON_RESUELTOS)
         } else {
-            format!("resuelve {} y lanza guardar", ficheros.join(" "))
+            Mensaje::nuevo(&txt::RESUELVE_Y_LANZA_GUARDAR).con("ficheros", ficheros.join(" "))
         };
         Informe::nuevo(OP, Resultado::Conflicto)
             .con("base", base)
@@ -407,14 +423,14 @@ fn actualizar(git: &Git, dir: &Path, esperas: &[Duration]) -> Paso<Informe> {
     if !entradas_status(&pendientes).is_empty() {
         return Ok(Informe::nuevo(OP, Resultado::Rechazada)
             .con("motivo", "cambios-sin-guardar")
-            .diagnostico("guarda los cambios con guardar antes de actualizar la rama"));
+            .diagnostico(&txt::GUARDA_ANTES_DE_ACTUALIZAR));
     }
     traer(git, dir, OP)?;
     if commit_de(git, dir, OP, base)?.is_none() {
         return Ok(Informe::nuevo(OP, Resultado::Rechazada)
             .con("base", base)
             .con("motivo", "base-no-existe")
-            .diagnostico(format!("{base} no existe en el remoto")));
+            .diagnostico(Mensaje::nuevo(&txt::BASE_NO_EXISTE_EN_REMOTO).con("base", base)));
     }
     let remota = format!("{REMOTO}/{rama}");
     let hay_remota = commit_de(git, dir, OP, &remota)?.is_some();
@@ -445,7 +461,7 @@ fn actualizar(git: &Git, dir: &Path, esperas: &[Duration]) -> Paso<Informe> {
             }
             return Ok(Informe::nuevo(OP, Resultado::Error)
                 .con("base", base)
-                .diagnostico(format!("git merge fallo: {}", s.primera_linea_error())));
+                .diagnostico(git_fallo(Subcomando::Merge, &s)));
         }
         mergeado = true;
     }
@@ -462,9 +478,7 @@ fn actualizar(git: &Git, dir: &Path, esperas: &[Duration]) -> Paso<Informe> {
                 return Ok(Informe::nuevo(OP, Resultado::Rechazada)
                     .con("rama", rama.as_str())
                     .con("motivo", "no-fast-forward")
-                    .diagnostico(
-                        "alguien publico en la rama entre medias: vuelve a lanzar actualizar-rama",
-                    ));
+                    .diagnostico(&txt::PUBLICARON_ENTRE_MEDIAS));
             }
             otro => {
                 let que_hacer = que_hacer_al_publicar(otro);
@@ -472,9 +486,11 @@ fn actualizar(git: &Git, dir: &Path, esperas: &[Duration]) -> Paso<Informe> {
                 if matches!(otro, Resultado::Denegada | Resultado::Error) {
                     i = i.con("motivo", p.motivo);
                 }
-                return Ok(i.con("intentos", intentos.to_string()).diagnostico(format!(
-                    "push: {ultimo}; la rama esta actualizada en local; {que_hacer}"
-                )));
+                return Ok(i.con("intentos", intentos.to_string()).diagnostico(
+                    Mensaje::nuevo(&txt::PUSH_FALLO_ACTUALIZADA_EN_LOCAL)
+                        .con("ultimo", ultimo)
+                        .con_mensaje("que_hacer", que_hacer),
+                ));
             }
         }
     }
@@ -505,7 +521,7 @@ fn actualizar_principal_paso(git: &Git, dir: &Path, rama: &str) -> Paso<Informe>
         return Ok(Informe::nuevo(OP, Resultado::Rechazada)
             .con("rama", rama)
             .con("motivo", "principal-en-otra-rama")
-            .diagnostico(format!("el worktree principal no esta en {rama}")));
+            .diagnostico(Mensaje::nuevo(&txt::PRINCIPAL_EN_OTRA_RAMA).con("rama", rama)));
     }
     traer(git, &principal, OP)?;
     let remota = format!("{REMOTO}/{rama}");
@@ -521,9 +537,11 @@ fn actualizar_principal_paso(git: &Git, dir: &Path, rama: &str) -> Paso<Informe>
         return Ok(Informe::nuevo(OP, Resultado::Rechazada)
             .con("rama", rama)
             .con("motivo", "divergente")
-            .diagnostico(format!(
-                "{rama} local tiene commits que no estan en {remota}"
-            )));
+            .diagnostico(
+                Mensaje::nuevo(&txt::PRINCIPAL_DIVERGENTE)
+                    .con("rama", rama)
+                    .con("remota", remota.as_str()),
+            ));
     }
     exigir(
         git,
diff --git a/crates/tc2/src/git/tag.rs b/crates/tc2/src/git/tag.rs
index 12f852d..cb3c6ce 100644
--- a/crates/tc2/src/git/tag.rs
+++ b/crates/tc2/src/git/tag.rs
@@ -6,6 +6,8 @@ use std::time::Duration;
 use super::remoto::publicar;
 use super::{Git, Paso, REMOTO, Subcomando, commit_de, correr, exigir};
 use crate::contrato::{Informe, Resultado, corto};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::git as txt;
 
 /// Pone `tag` sobre `referencia` y lo publica. Compara con el tag local
 /// (aunque no este publicado) y con el del remoto: en otro commit →
@@ -40,16 +42,16 @@ fn tag_paso(
         return Ok(Informe::nuevo(OP, Resultado::Rechazada)
             .con("tag", tag)
             .con("motivo", "ref-no-existe")
-            .diagnostico(format!("{referencia} no existe")));
+            .diagnostico(
+                Mensaje::nuevo(&txt::REFERENCIA_NO_EXISTE).con("referencia", referencia),
+            ));
     };
     let otro_commit = |donde: &str| {
         Informe::nuevo(OP, Resultado::Rechazada)
             .con("tag", tag)
             .con("motivo", "existe-en-otro-commit")
             .con("donde", donde)
-            .diagnostico(format!(
-                "el tag {tag} ya existe sobre otro commit: elige otro nombre"
-            ))
+            .diagnostico(Mensaje::nuevo(&txt::TAG_EN_OTRO_COMMIT).con("tag", tag))
     };
     let local = commit_de(git, dir, OP, &format!("refs/tags/{tag}"))?;
     if local.as_ref().is_some_and(|l| *l != commit) {
@@ -82,12 +84,11 @@ fn tag_paso(
             if matches!(p.resultado, Resultado::Denegada | Resultado::Error) {
                 i = i.con("motivo", p.motivo);
             }
-            return Ok(i
-                .con("intentos", p.intentos.to_string())
-                .diagnostico(format!(
-                    "no se pudo publicar el tag: {}; el tag queda en local, {que_hacer}",
-                    p.ultimo
-                )));
+            return Ok(i.con("intentos", p.intentos.to_string()).diagnostico(
+                Mensaje::nuevo(&txt::TAG_SIN_PUBLICAR)
+                    .con("ultimo", p.ultimo)
+                    .con_mensaje("que_hacer", que_hacer),
+            ));
         }
     }
     Ok(Informe::nuevo(OP, Resultado::Ok)
@@ -109,7 +110,7 @@ fn commit_remoto(git: &Git, dir: &Path, op: &str, tag: &str) -> Paso<Option<Stri
     if !s.exito() {
         return Err(Informe::nuevo(op, Resultado::Sinred)
             .con("remoto", REMOTO)
-            .diagnostico(format!("{op}: no se pudo leer {REMOTO}; reintenta con red")));
+            .diagnostico(super::remoto::sin_red_remoto(op)));
     }
     let mut directo = None;
     for l in s.stdout.lines() {
diff --git a/crates/tc2/src/git/worktree.rs b/crates/tc2/src/git/worktree.rs
index 2af2175..10a91f1 100644
--- a/crates/tc2/src/git/worktree.rs
+++ b/crates/tc2/src/git/worktree.rs
@@ -9,6 +9,8 @@ use super::{
     localizar_principal, misma_ruta, worktree_relativo, worktrees,
 };
 use crate::contrato::{Informe, Resultado};
+use crate::i18n::Texto;
+use crate::i18n::catalogo::git as txt;
 
 /// Quita `.taskcode/worktrees/<id>`. No existe → `sin-cambios`. Con cambios
 /// sin guardar o commits sin publicar o HEAD suelto → `rechazada`, sin tocar nada.
@@ -29,26 +31,20 @@ fn quitar(git: &Git, dir: &Path, id: &str) -> Paso<Informe> {
     else {
         return Ok(Informe::nuevo(OP, Resultado::SinCambios).con("id", id));
     };
-    let rechazo = |motivo: &str, que_hacer: &str| {
+    let rechazo = |motivo: &str, que_hacer: &'static Texto| {
         Informe::nuevo(OP, Resultado::Rechazada)
             .con("id", id)
             .con("motivo", motivo)
-            .diagnostico(que_hacer.to_owned())
+            .diagnostico(que_hacer)
     };
     let status = exigir(git, &w.ruta, OP, Subcomando::Status, &["--porcelain", "-z"])?;
     if !status.is_empty() {
-        return Ok(rechazo(
-            "cambios-sin-guardar",
-            "guarda o descarta los cambios del worktree antes de quitarlo",
-        ));
+        return Ok(rechazo("cambios-sin-guardar", &txt::WORKTREE_CON_CAMBIOS));
     }
     // Con HEAD suelto no hay rama remota con la que comparar: sus commits se
     // perderian al quitar el worktree.
     let Some(rama) = &w.rama else {
-        return Ok(rechazo(
-            "sin-rama",
-            "el worktree tiene HEAD suelto: vuelve a la rama de la tarea",
-        ));
+        return Ok(rechazo("sin-rama", &txt::WORKTREE_HEAD_SUELTO));
     };
     let publicada = match commit_de(
         git,
@@ -60,10 +56,7 @@ fn quitar(git: &Git, dir: &Path, id: &str) -> Paso<Informe> {
         None => false,
     };
     if !publicada {
-        return Ok(rechazo(
-            "sin-publicar",
-            "publica la rama antes de quitar el worktree",
-        ));
+        return Ok(rechazo("sin-publicar", &txt::WORKTREE_SIN_PUBLICAR));
     }
     let rel = worktree_relativo(id);
     exigir(git, &principal, OP, Subcomando::Worktree, &["remove", &rel])?;
diff --git a/crates/tc2/src/i18n/catalogo/cli.rs b/crates/tc2/src/i18n/catalogo/cli.rs
new file mode 100644
index 0000000..8412dbc
--- /dev/null
+++ b/crates/tc2/src/i18n/catalogo/cli.rs
@@ -0,0 +1,48 @@
+//! Mensajes del borde del CLI (`main.rs` y `lib.rs`).
+
+crate::i18n::textos! {
+    ARGUMENTO_NO_UTF8 {
+        es: "tc2: argumento que no es UTF-8",
+        en: "tc2: argument is not valid UTF-8",
+    }
+    NO_SE_PUDO_ESCRIBIR {
+        es: "tc2: no se pudo escribir la salida: {error}",
+        en: "tc2: could not write the output: {error}",
+    }
+    USO {
+        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\"",
+        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\"",
+    }
+    FALTA_LA_ORDEN {
+        es: "tc2: falta la orden. {uso}",
+        en: "tc2: missing command. {uso}",
+    }
+    ORDEN_DESCONOCIDA {
+        es: "tc2: orden desconocida «{orden}». {uso}",
+        en: "tc2: unknown command «{orden}». {uso}",
+    }
+    CARPETA_ACTUAL {
+        es: "no se puede leer la carpeta actual: {error}",
+        en: "cannot read the current folder: {error}",
+    }
+    USO_GUARDAR {
+        es: "uso: guardar \"<mensaje>\" <rutas...>",
+        en: "usage: guardar \"<message>\" <paths...>",
+    }
+    USO_ACTUALIZAR_RAMA {
+        es: "uso: actualizar-rama [--json]",
+        en: "usage: actualizar-rama [--json]",
+    }
+    TITULO_INVALIDO {
+        es: "el titulo no puede estar vacio ni llevar saltos de linea, tabuladores ni otros caracteres de control",
+        en: "the title cannot be empty or contain line breaks, tabs or other control characters",
+    }
+    USO_NEW {
+        es: "uso: new [--json] \"<titulo>\"",
+        en: "usage: new [--json] \"<title>\"",
+    }
+    TITULO_ENTRE_COMILLAS {
+        es: "pon el titulo entre comillas: new [--json] \"<titulo>\"",
+        en: "put the title in quotes: new [--json] \"<title>\"",
+    }
+}
diff --git a/crates/tc2/src/i18n/catalogo/comun.rs b/crates/tc2/src/i18n/catalogo/comun.rs
new file mode 100644
index 0000000..fe75232
--- /dev/null
+++ b/crates/tc2/src/i18n/catalogo/comun.rs
@@ -0,0 +1,7 @@
+//! Entradas que no son de ninguna zona.
+
+crate::i18n::textos! {
+    /// Texto que viene de fuera de `tc2` (git, el sistema, HTTP) y se
+    /// muestra tal cual.
+    EXTERNO { es: "{texto}", en: "{texto}" }
+}
diff --git a/crates/tc2/src/i18n/catalogo/config.rs b/crates/tc2/src/i18n/catalogo/config.rs
new file mode 100644
index 0000000..996e090
--- /dev/null
+++ b/crates/tc2/src/i18n/catalogo/config.rs
@@ -0,0 +1,307 @@
+//! Mensajes de `src/config/`.
+
+crate::i18n::textos! {
+    // --- Diagnostico de `config rechazada` (mod.rs) ---
+    DIAGNOSTICO {
+        es: "{ruta}: {mensaje}{extra}. Arreglo: {arreglo}",
+        en: "{ruta}: {mensaje}{extra}. Fix: {arreglo}",
+    }
+    DIAGNOSTICO_CON_CLAVE {
+        es: "{ruta}: {clave}: {mensaje}{extra}. Arreglo: {arreglo}",
+        en: "{ruta}: {clave}: {mensaje}{extra}. Fix: {arreglo}",
+    }
+    MAS_ERRORES {
+        es: " (y {n} error(es) mas)",
+        en: " (and {n} more error(s))",
+    }
+    ARREGLO_SIN_CONFIG {
+        es: "crea la configuracion con tc2 init",
+        en: "create the configuration with tc2 init",
+    }
+    ARREGLO_FORMATO {
+        es: "es un proyecto de la 0.16: migralo con tc2 actualizar",
+        en: "it is a 0.16 project: migrate it with tc2 actualizar",
+    }
+    ARREGLO_FORMATO_NUEVO {
+        es: "el proyecto es de un tc2 mas nuevo: actualiza tc2",
+        en: "the project is from a newer tc2: update tc2",
+    }
+    ARREGLO_YAML {
+        es: "corrige el YAML de {ruta}",
+        en: "fix the YAML in {ruta}",
+    }
+    ARREGLO_CLAVES_016 {
+        es: "quedan claves de la 0.16: tc2 actualizar las traduce",
+        en: "0.16 keys remain: tc2 actualizar translates them",
+    }
+    ARREGLO_ESQUEMA {
+        es: "corrige {ruta} (esquema: crates/tc2/esquemas/config.schema.json)",
+        en: "fix {ruta} (schema: crates/tc2/esquemas/config.schema.json)",
+    }
+
+    // --- Lectura del documento (mod.rs) ---
+    PRIMERA_LINEA_SIN_BOM {
+        es: "la primera linea tiene que ser --- (sin BOM)",
+        en: "the first line must be --- (without BOM)",
+    }
+    PRIMERA_LINEA {
+        es: "la primera linea tiene que ser ---",
+        en: "the first line must be ---",
+    }
+    YAML_NO_VALIDO {
+        es: "YAML no valido: {error}",
+        en: "invalid YAML: {error}",
+    }
+    RAIZ_NO_MAPA {
+        es: "la raiz tiene que ser un mapa",
+        en: "the root must be a map",
+    }
+    FORMATO_NUEVO {
+        es: "formato {formato}: este tc2 solo lee el formato 2",
+        en: "formato {formato}: this tc2 only reads format 2",
+    }
+    SE_ESPERABA {
+        es: "se esperaba {esperado}, no {valor}",
+        en: "expected {esperado}, not {valor}",
+    }
+    FALTA_CLAVE {
+        es: "falta la clave obligatoria",
+        en: "the required key is missing",
+    }
+    ERROR_INTERNO {
+        es: "error interno al construir la configuracion validada: {error}",
+        en: "internal error building the validated configuration: {error}",
+    }
+    SIN_BASE {
+        es: "no existe {base}: haz fetch o crea la rama develop en el remoto",
+        en: "{base} does not exist: fetch or create the develop branch on the remote",
+    }
+    NO_HAY_CONFIG {
+        es: "no hay {ruta} en {base}",
+        en: "there is no {ruta} in {base}",
+    }
+
+    // --- Validacion contra la tabla (validar.rs) ---
+    ENTERO_EN_RANGO {
+        es: "se esperaba un entero de {min} a {max}, no {valor}",
+        en: "expected an integer from {min} to {max}, not {valor}",
+    }
+    ENTERO_DESDE {
+        es: "se esperaba un entero de {min} o mas, no {valor}",
+        en: "expected an integer of {min} or more, not {valor}",
+    }
+    SOLO_ADMITE {
+        es: "solo admite {valor}",
+        en: "only accepts {valor}",
+    }
+    SE_ESPERABA_BOOLEANO {
+        es: "se esperaba true o false, no {valor}",
+        en: "expected true or false, not {valor}",
+    }
+    SE_ESPERABA_TEXTO {
+        es: "se esperaba un texto, no {valor}",
+        en: "expected a text, not {valor}",
+    }
+    VALOR_NO_ADMITIDO {
+        es: "valor no admitido «{valor}» (admite: {admite})",
+        en: "value not accepted «{valor}» (accepts: {admite})",
+    }
+    SE_ESPERABA_LISTA {
+        es: "se esperaba una lista, no {valor}",
+        en: "expected a list, not {valor}",
+    }
+    LISTA_CORTA {
+        es: "se esperaba una lista con al menos {min} elemento(s)",
+        en: "expected a list with at least {min} element(s)",
+    }
+    PRIMER_ELEMENTO {
+        es: "el primer elemento tiene que ser «{valor}»",
+        en: "the first element must be «{valor}»",
+    }
+    REPETIDO {
+        es: "«{valor}» repetido",
+        en: "«{valor}» repeated",
+    }
+    SE_ESPERABA_MAPA {
+        es: "se esperaba un mapa, no {valor}",
+        en: "expected a map, not {valor}",
+    }
+    NO_ENCAJA {
+        es: "no encaja con ninguna forma admitida: {formas}",
+        en: "does not match any accepted form: {formas}",
+    }
+    FORMA_O_FORMA {
+        es: "{a} o {b}",
+        en: "{a} or {b}",
+    }
+    UN_MAPA_CON {
+        es: "un mapa con {claves}",
+        en: "a map with {claves}",
+    }
+    LISTA_QUE_EMPIEZA {
+        es: "una lista que empieza por {primero}",
+        en: "a list starting with {primero}",
+    }
+    UNA_LISTA {
+        es: "una lista",
+        en: "a list",
+    }
+    UN_MAPA {
+        es: "un mapa",
+        en: "a map",
+    }
+    UN_VALOR {
+        es: "un valor",
+        en: "a value",
+    }
+    NADA {
+        es: "nada",
+        en: "nothing",
+    }
+    OBLIGATORIA_EN_GENERA {
+        es: "obligatoria en un derivado de tipo genera",
+        en: "required in a derivado of tipo genera",
+    }
+    PROHIBIDA_EN_COMPRUEBA {
+        es: "prohibida en un derivado de tipo comprueba",
+        en: "forbidden in a derivado of tipo comprueba",
+    }
+    CLAVE_NO_ADMITIDA_016 {
+        es: "clave no admitida: es de la configuracion de la 0.16 (tc2 actualizar la traduce)",
+        en: "key not accepted: it belongs to the 0.16 configuration (tc2 actualizar translates it)",
+    }
+    CLAVE_NO_ADMITIDA {
+        es: "clave no admitida",
+        en: "key not accepted",
+    }
+
+    // --- Reglas de texto (validar.rs) ---
+    TEXTO_NO_NOMBRE {
+        es: "texto vacio, con espacios en los extremos o con caracteres de control",
+        en: "empty text, with spaces at the ends or with control characters",
+    }
+    ORDEN_VACIA {
+        es: "orden vacia",
+        en: "empty command",
+    }
+    ORDEN_CON_SALTOS {
+        es: "la orden tiene saltos de linea: escribela en una sola linea",
+        en: "the command has line breaks: write it on a single line",
+    }
+    ORDEN_CON_CONTROL {
+        es: "la orden tiene un caracter de control (¿un \\v de una ruta de Windows entre comillas dobles? escribela entre comillas simples o con /)",
+        en: "the command has a control character (a \\v from a Windows path in double quotes? write it in single quotes or with /)",
+    }
+    GLOB_NO_ADMITIDO {
+        es: "glob no admitido «{glob}»: {motivo}",
+        en: "glob not accepted «{glob}»: {motivo}",
+    }
+    NUNCA_TODO_EL_ARBOL {
+        es: "nunca todo el arbol",
+        en: "never the whole tree",
+    }
+    VACIO {
+        es: "vacio",
+        en: "empty",
+    }
+    CARACTER_DE_CONTROL {
+        es: "caracter de control",
+        en: "control character",
+    }
+    SIN_NEGACIONES {
+        es: "sin negaciones",
+        en: "no negations",
+    }
+    SIN_RUTAS_ABSOLUTAS {
+        es: "sin rutas absolutas",
+        en: "no absolute paths",
+    }
+    SIN_PUNTO_PUNTO {
+        es: "sin ..",
+        en: "no ..",
+    }
+    RUTA_NO_ADMITIDA {
+        es: "ruta no admitida «{ruta}»: {motivo}",
+        en: "path not accepted «{ruta}»: {motivo}",
+    }
+    SIN_BARRA_NI_DOS_PUNTOS {
+        es: "sin \\ ni :",
+        en: "no \\ or :",
+    }
+    SIN_COMODINES {
+        es: "sin comodines",
+        en: "no wildcards",
+    }
+    SALE_DE_LA_RAIZ {
+        es: "sale de la raiz",
+        en: "goes outside the root",
+    }
+    SEGMENTO_WINDOWS {
+        es: "sin punto ni espacio al final de un segmento ni nombres cortos con ~ (Windows los ignora o los acorta)",
+        en: "no dot or space at the end of a segment and no short names with ~ (Windows ignores or shortens them)",
+    }
+    NOMBRE_RESERVADO_WINDOWS {
+        es: "nombre reservado de Windows",
+        en: "reserved Windows name",
+    }
+    NUNCA_GIT {
+        es: "nunca .git",
+        en: "never .git",
+    }
+    ROADMAP_RESERVADA {
+        es: "tareas/ROADMAP.md es reservada",
+        en: "tareas/ROADMAP.md is reserved",
+    }
+    CARPETAS_DE_TAREA_RESERVADAS {
+        es: "las carpetas tareas/TASK-NNN/ son reservadas",
+        en: "the tareas/TASK-NNN/ folders are reserved",
+    }
+    NUNCA_TAREAS_ENTERA {
+        es: "nunca tareas/ entera",
+        en: "never the whole tareas/",
+    }
+    BAJO_TAREAS {
+        es: "tiene que ir bajo tareas/",
+        en: "it must be under tareas/",
+    }
+    CRON_NO_ADMITIDO {
+        es: "cron no admitido «{cron}»: se esperan 5 campos (minuto 0-59, hora 0-23, dia 1-31, mes 1-12, dia de la semana 0-7) con *, a, a-b, ',' y /n",
+        en: "cron not accepted «{cron}»: 5 fields are expected (minute 0-59, hour 0-23, day 1-31, month 1-12, day of the week 0-7) with *, a, a-b, ',' and /n",
+    }
+    URL_NO_ADMITIDA {
+        es: "URL no admitida: {motivo}",
+        en: "URL not accepted: {motivo}",
+    }
+
+    // --- Lo que exige cada regla (claves.rs). Solo va al esquema JSON
+    // publicado, que queda en espanol; estan aqui para que no haya textos
+    // de regla fuera del catalogo. ---
+    REGLA_NOMBRE {
+        es: "texto no vacio, sin caracteres de control ni espacios en los extremos",
+        en: "non-empty text, without control characters or spaces at the ends",
+    }
+    REGLA_ORDEN {
+        es: "orden no vacia y sin caracteres de control (rutas de Windows entre comillas simples o con /)",
+        en: "non-empty command without control characters (Windows paths in single quotes or with /)",
+    }
+    REGLA_GLOB {
+        es: "glob relativo: sin negacion (!), rutas absolutas ni ..",
+        en: "relative glob: no negation (!), absolute paths or ..",
+    }
+    REGLA_GLOB_ACOTADO {
+        es: "glob relativo: sin negacion (!), rutas absolutas ni .., y nunca todo el arbol (ningun segmento solo de * y .)",
+        en: "relative glob: no negation (!), absolute paths or .., and never the whole tree (no segment made only of * and .)",
+    }
+    REGLA_RUTA_TAREAS {
+        es: "ruta normalizada bajo tareas/, sin \\, :, comodines, caracteres de control, nombres reservados de Windows, .git, ~ ni punto o espacio al final de un segmento; nunca tareas/ entera, tareas/ROADMAP.md ni tareas/TASK-NNN/ (sin distinguir mayusculas)",
+        en: "normalized path under tareas/, without \\, :, wildcards, control characters, reserved Windows names, .git, ~ or a dot or space at the end of a segment; never the whole tareas/, tareas/ROADMAP.md or tareas/TASK-NNN/ (case-insensitive)",
+    }
+    REGLA_CRON {
+        es: "expresion cron de 5 campos con sus rangos (minuto, hora, dia, mes, dia de la semana)",
+        en: "5-field cron expression with its ranges (minute, hour, day, month, day of the week)",
+    }
+    REGLA_URL {
+        es: "URL https de la API (o http://127.0.0.1:<puerto> en pruebas)",
+        en: "https URL of the API (or http://127.0.0.1:<puerto> in tests)",
+    }
+}
diff --git a/crates/tc2/src/i18n/catalogo/estado.rs b/crates/tc2/src/i18n/catalogo/estado.rs
new file mode 100644
index 0000000..2689b53
--- /dev/null
+++ b/crates/tc2/src/i18n/catalogo/estado.rs
@@ -0,0 +1,92 @@
+//! Mensajes de la rama de estado (`src/estado/`) y del alta (`src/alta.rs`).
+
+crate::i18n::textos! {
+    MONTA_PRIMERO {
+        es: "monta la rama de estado en {carpeta}/ primero",
+        en: "mount the state branch in {carpeta}/ first",
+    }
+
+    // Cerrojo local.
+    CERROJO_OCUPADO_AL_MONTAR {
+        es: "otro proceso de este clon esta montando o escribiendo la rama de estado: reintenta",
+        en: "another process of this clone is mounting or writing the state branch: retry",
+    }
+    CERROJO_OCUPADO {
+        es: "otro proceso de este clon esta escribiendo la rama de estado: reintenta",
+        en: "another process of this clone is writing the state branch: retry",
+    }
+    CERROJO_ERROR {
+        es: "cerrojo: {error}",
+        en: "lock: {error}",
+    }
+
+    // Montaje (`montaje.rs`).
+    MONTADA_EN_OTRA_RUTA {
+        es: "{rama} ya esta montada en {ruta}: quita ese worktree",
+        en: "{rama} is already mounted at {ruta}: remove that worktree",
+    }
+    CARPETA_OCUPADA {
+        es: "{carpeta}/ ya existe y no es la rama de estado (¿un repo de la 0.16? migralo con actualizar)",
+        en: "{carpeta}/ already exists and is not the state branch (a 0.16 repo? migrate it with actualizar)",
+    }
+    MONTAR_EXIGE_RED {
+        es: "{error}; montar la rama de estado exige red la primera vez",
+        en: "{error}; mounting the state branch needs network access the first time",
+    }
+    MONTAR_DENEGADA {
+        es: "{error}; revisa credenciales, permisos o la URL de origin",
+        en: "{error}; check credentials, permissions or the origin URL",
+    }
+    RAMA_SIGUE_SIN_EXISTIR {
+        es: "{rama} sigue sin existir tras crearla",
+        en: "{rama} still does not exist after creating it",
+    }
+    RUTA_TAREAS_NO_UTF8 {
+        es: "la ruta de tareas/ no es UTF-8",
+        en: "the tareas/ path is not valid UTF-8",
+    }
+
+    // Transaccion (`transaccion.rs`).
+    AGOTADA {
+        es: "la rama de estado cambio en cada intento: no se ha escrito nada; reintenta",
+        en: "the state branch changed on every attempt: nothing was written; retry",
+    }
+    RECHAZADA_COMO_ERROR {
+        es: "{motivo}: {diagnostico}",
+        en: "rejected ({motivo}): {diagnostico}",
+    }
+    TAREAS_CON_CAMBIOS {
+        es: "{carpeta}/ tiene cambios que chocan con la operacion: guardalos o deshazlos y reintenta",
+        en: "{carpeta}/ has changes that clash with the operation: save or undo them and retry",
+    }
+    RUTA_DUPLICADA {
+        es: "{nueva} solo difiere en mayusculas de {otra}: chocarian en Windows y macOS",
+        en: "{nueva} differs from {otra} only in case: they would clash on Windows and macOS",
+    }
+    SIN_MENSAJE {
+        es: "sin mensaje",
+        en: "no message",
+    }
+    ESCRITURA_DENEGADA {
+        es: "{detalle}; el remoto deniega la escritura: revisa credenciales, permisos o reglas, no se reintenta",
+        en: "{detalle}; the remote denies the write: check credentials, permissions or rules, it is not retried",
+    }
+    EXIGE_RED {
+        es: "{detalle}; esta operacion exige red: no se ha escrito nada, reintenta con red",
+        en: "{detalle}; this operation needs network access: nothing was written, retry with network access",
+    }
+    QUEDA_PENDIENTE {
+        es: "{detalle}; queda pendiente de publicar: se publicara en la siguiente operacion con red",
+        en: "{detalle}; pending publication: it will be published in the next operation with network access",
+    }
+    NO_SE_PUDO_ADELANTAR {
+        es: "no se pudo adelantar {rama}: {error}",
+        en: "could not fast-forward {rama}: {error}",
+    }
+
+    // Alta (`alta.rs`).
+    SIN_IDS {
+        es: "no quedan IDs de tarea",
+        en: "no task IDs left",
+    }
+}
diff --git a/crates/tc2/src/i18n/catalogo/git.rs b/crates/tc2/src/i18n/catalogo/git.rs
new file mode 100644
index 0000000..ece9886
--- /dev/null
+++ b/crates/tc2/src/i18n/catalogo/git.rs
@@ -0,0 +1,193 @@
+//! Mensajes de `src/git/`.
+
+crate::i18n::textos! {
+    // Puerto y llamadas a git (`mod.rs`, `puerto.rs`, `estado.rs`).
+    ARGUMENTO_PROHIBIDO {
+        es: "argumento prohibido para la capa: {argumento}",
+        en: "argument not allowed for the layer: {argumento}",
+    }
+    NO_SE_PUDO_LANZAR_GIT {
+        es: "no se pudo lanzar git: {error}",
+        en: "could not run git: {error}",
+    }
+    GIT_FALLO {
+        es: "git {subcomando} fallo: {error}",
+        en: "git {subcomando} failed: {error}",
+    }
+    ERROR_DE_FICHERO {
+        es: "{ruta}: {error}",
+        en: "{ruta}: I/O error: {error}",
+    }
+    RUTA_TEMPORAL_NO_UTF8 {
+        es: "ruta temporal no UTF-8",
+        en: "temporary path is not valid UTF-8",
+    }
+    RUTA_INVALIDA_EN_ESTADO {
+        es: "ruta invalida en la rama de estado: {ruta}",
+        en: "invalid path in the state branch: {ruta}",
+    }
+    HASH_OBJECT_INCOMPLETO {
+        es: "git hash-object no devolvio un hash por fichero",
+        en: "git hash-object did not return one hash per file",
+    }
+
+    // Donde se lanza.
+    LANZALO_EN_EL_WORKTREE {
+        es: "lanzalo en el worktree de la tarea",
+        en: "run it in the task worktree",
+    }
+    LANZALO_FUERA_DE_PROTEGIDA {
+        es: "lanzalo en el worktree de la tarea, no en develop ni en main",
+        en: "run it in the task worktree, not on develop or main",
+    }
+    NO_ES_REPO {
+        es: "lanza el comando dentro de un repositorio git",
+        en: "run the command inside a git repository",
+    }
+
+    // Remoto (`remoto.rs`, `rama.rs`, `tag.rs`).
+    SIN_RED_REMOTO {
+        es: "{op}: no se pudo leer {remoto}; reintenta con red",
+        en: "{op}: could not read {remoto}; retry with network access",
+    }
+    DENEGADA_CREDENCIALES {
+        es: "{op}: {detalle}; revisa tus credenciales o permisos en el remoto",
+        en: "{op}: {detalle}; check your credentials or permissions on the remote",
+    }
+    DENEGADA_URL {
+        es: "{op}: {detalle}; revisa la URL de origin",
+        en: "{op}: {detalle}; check the origin URL",
+    }
+    DENEGADA_REGLAS {
+        es: "{op}: {detalle}; revisa las reglas del remoto o avisa a la persona",
+        en: "{op}: {detalle}; check the remote rules or tell the person",
+    }
+    ERROR_LOCAL {
+        es: "{op}: {detalle}; fallo local de git: revisa los permisos de la carpeta del repo",
+        en: "{op}: {detalle}; local git failure: check the permissions of the repo folder",
+    }
+    QUE_HACER_DENEGADA {
+        es: "el remoto deniega el push: revisa sus reglas, tus permisos o avisa a la persona",
+        en: "the remote denies the push: check its rules, your permissions or tell the person",
+    }
+    QUE_HACER_ERROR_LOCAL {
+        es: "fallo local de git: revisa los permisos de la carpeta del repo",
+        en: "local git failure: check the permissions of the repo folder",
+    }
+    QUE_HACER_SIN_RED {
+        es: "reintenta con red",
+        en: "retry with network access",
+    }
+    PUSH_FALLO {
+        es: "push: {ultimo}; {que_hacer}",
+        en: "push failed: {ultimo}; {que_hacer}",
+    }
+    PUSH_FALLO_ACTUALIZADA_EN_LOCAL {
+        es: "push: {ultimo}; la rama esta actualizada en local; {que_hacer}",
+        en: "push failed: {ultimo}; the branch is updated locally; {que_hacer}",
+    }
+    LANZA_ACTUALIZAR_RAMA {
+        es: "lanza actualizar-rama",
+        en: "run actualizar-rama",
+    }
+    LANZA_GUARDAR_CON_RESUELTOS {
+        es: "lanza guardar con los ficheros resueltos",
+        en: "run guardar with the resolved files",
+    }
+    RESUELVE_Y_LANZA_GUARDAR {
+        es: "resuelve {ficheros} y lanza guardar",
+        en: "resolve {ficheros} and run guardar",
+    }
+    GUARDA_ANTES_DE_ACTUALIZAR {
+        es: "guarda los cambios con guardar antes de actualizar la rama",
+        en: "save the changes with guardar before updating the branch",
+    }
+    BASE_NO_EXISTE_EN_REMOTO {
+        es: "{base} no existe en el remoto",
+        en: "{base} does not exist on the remote",
+    }
+    PUBLICARON_ENTRE_MEDIAS {
+        es: "alguien publico en la rama entre medias: vuelve a lanzar actualizar-rama",
+        en: "someone pushed to the branch in the meantime: run actualizar-rama again",
+    }
+    PRINCIPAL_EN_OTRA_RAMA {
+        es: "el worktree principal no esta en {rama}",
+        en: "the main worktree is not on {rama}",
+    }
+    PRINCIPAL_DIVERGENTE {
+        es: "{rama} local tiene commits que no estan en {remota}",
+        en: "local {rama} has commits that are not in {remota}",
+    }
+
+    // `guardar`.
+    RESUELVE_Y_GUARDA_CON_EL_RESTO {
+        es: "resuelve {ficheros} y guardalo con el resto",
+        en: "resolve {ficheros} and save it with the rest",
+    }
+    QUITA_LAS_MARCAS {
+        es: "quita las marcas de conflicto de {ficheros}",
+        en: "remove the conflict markers from {ficheros}",
+    }
+
+    // `tag-anotado`.
+    REFERENCIA_NO_EXISTE {
+        es: "{referencia} no existe",
+        en: "{referencia} does not exist",
+    }
+    TAG_EN_OTRO_COMMIT {
+        es: "el tag {tag} ya existe sobre otro commit: elige otro nombre",
+        en: "tag {tag} already exists on another commit: choose another name",
+    }
+    TAG_SIN_PUBLICAR {
+        es: "no se pudo publicar el tag: {ultimo}; el tag queda en local, {que_hacer}",
+        en: "could not publish the tag: {ultimo}; the tag stays local, {que_hacer}",
+    }
+
+    // `quitar-worktree`.
+    WORKTREE_CON_CAMBIOS {
+        es: "guarda o descarta los cambios del worktree antes de quitarlo",
+        en: "save or discard the worktree changes before removing it",
+    }
+    WORKTREE_HEAD_SUELTO {
+        es: "el worktree tiene HEAD suelto: vuelve a la rama de la tarea",
+        en: "the worktree has a detached HEAD: go back to the task branch",
+    }
+    WORKTREE_SIN_PUBLICAR {
+        es: "publica la rama antes de quitar el worktree",
+        en: "push the branch before removing the worktree",
+    }
+
+    // Rama y worktree de tarea (`rama.rs`).
+    BASE_NO_EXISTE {
+        es: "la base {base} no existe: lanza fetch o revisa el nombre",
+        en: "base {base} does not exist: run fetch or check the name",
+    }
+    ID_CON_RAMA {
+        es: "{id} ya tiene rama: elige otro ID",
+        en: "{id} already has a branch: choose another ID",
+    }
+    RUTA_WORKTREE_NO_UTF8 {
+        es: "la ruta del worktree no es UTF-8",
+        en: "the worktree path is not valid UTF-8",
+    }
+    RAMA_EN_OTRO_WORKTREE {
+        es: "la rama ya esta abierta en otro worktree",
+        en: "the branch is already open in another worktree",
+    }
+    RAMA_YA_EXISTE {
+        es: "la rama ya existe: la recupera reanudar",
+        en: "the branch already exists: reanudar recovers it",
+    }
+    RAMA_YA_PUBLICADA {
+        es: "la rama ya esta publicada: la recupera reanudar",
+        en: "the branch is already published: reanudar recovers it",
+    }
+    CARPETA_OCUPADA {
+        es: "{ruta} ya existe y no es el worktree de la tarea",
+        en: "{ruta} already exists and is not the task worktree",
+    }
+    TAREA_SIN_RAMA {
+        es: "la tarea no tiene rama ni en local ni en el remoto",
+        en: "the task has no branch, neither locally nor on the remote",
+    }
+}
diff --git a/crates/tc2/src/i18n/catalogo/mod.rs b/crates/tc2/src/i18n/catalogo/mod.rs
new file mode 100644
index 0000000..25ded12
--- /dev/null
+++ b/crates/tc2/src/i18n/catalogo/mod.rs
@@ -0,0 +1,26 @@
+//! Catalogo de mensajes, un modulo por zona del codigo: cada carpeta de
+//! `src/` escribe sus entradas en el suyo y no se pisan.
+
+pub mod cli;
+pub mod comun;
+pub mod config;
+pub mod estado;
+pub mod git;
+pub mod plataforma;
+
+use super::Texto;
+
+/// Todas las entradas del catalogo.
+pub fn todos() -> impl Iterator<Item = &'static Texto> {
+    [
+        cli::TODOS,
+        comun::TODOS,
+        config::TODOS,
+        estado::TODOS,
+        git::TODOS,
+        plataforma::TODOS,
+    ]
+    .into_iter()
+    .flatten()
+    .copied()
+}
diff --git a/crates/tc2/src/i18n/catalogo/plataforma.rs b/crates/tc2/src/i18n/catalogo/plataforma.rs
new file mode 100644
index 0000000..612dd15
--- /dev/null
+++ b/crates/tc2/src/i18n/catalogo/plataforma.rs
@@ -0,0 +1,289 @@
+//! Mensajes de `src/plataforma/`.
+//!
+//! `{cita}` es la cita del mensaje de la API (` (GitHub: «...»)`), vacia si
+//! la API no dio mensaje: la compone `plataforma::errores` con el nombre de
+//! la plataforma y el texto saneado, sin nada que traducir.
+
+crate::i18n::textos! {
+    // --- Respuestas HTTP que no son 2xx (errores.rs) ---
+    TOKEN_NO_VALE {
+        es: "el token de {rol} no vale o ha caducado: define o renueva {variable}{cita}",
+        en: "the {rol} token is not valid or has expired: set or renew {variable}{cita}",
+    }
+    LIMITE_AGOTADO {
+        es: "limite de peticiones de la API agotado: espera {segundos} s y relanza{cita}",
+        en: "API rate limit exhausted: wait {segundos} s and run again{cita}",
+    }
+    PLAN_GITHUB {
+        es: "el plan de GitHub no da esta funcion en un repo privado: haz el repo publico o usa GitHub Pro, Team o Enterprise{cita}",
+        en: "the GitHub plan does not offer this feature on a private repo: make the repo public or use GitHub Pro, Team or Enterprise{cita}",
+    }
+    SIN_PERMISO {
+        es: "el token de {rol} no tiene permiso: necesita {permiso}{cita}",
+        en: "the {rol} token lacks permission: it needs {permiso}{cita}",
+    }
+    NO_EXISTE {
+        es: "no existe o el token de {rol} no lo ve: revisa {objeto} y el alcance del token ({plataforma} da 404 en un repo privado sin acceso){cita}",
+        en: "it does not exist or the {rol} token cannot see it: check {objeto} and the token scope ({plataforma} returns 404 on a private repo without access){cita}",
+    }
+    OBJETO_GITHUB {
+        es: "el repo, el PR o el commit",
+        en: "the repo, the PR or the commit",
+    }
+    OBJETO_GITLAB {
+        es: "el proyecto, el MR o el commit",
+        en: "the project, the MR or the commit",
+    }
+    NO_MERGEABLE {
+        es: "{plataforma} no deja mergear: faltan puertas o hay conflictos; lanza estado-puertas{cita}",
+        en: "{plataforma} does not allow the merge: gates are missing or there are conflicts; run estado-puertas{cita}",
+    }
+    HEAD_CAMBIADO {
+        es: "la rama ha cambiado desde que se evaluaron las puertas: vuelve a lanzar estado-puertas{cita}",
+        en: "the branch has changed since the gates were evaluated: run estado-puertas again{cita}",
+    }
+    SIN_COMMITS {
+        es: "la rama no tiene commits nuevos respecto a la base: no hay nada que revisar{cita}",
+        en: "the branch has no new commits relative to the base: there is nothing to review{cita}",
+    }
+    RESPUESTA_INESPERADA {
+        es: "respuesta inesperada de {plataforma} ({estado}): reintenta mas tarde{cita}",
+        en: "unexpected response from {plataforma} ({estado}): try again later{cita}",
+    }
+    VALIDACION {
+        es: "{plataforma} rechazo los datos de la peticion{cita}",
+        en: "{plataforma} rejected the request data{cita}",
+    }
+
+    // --- Fallos sin respuesta (errores.rs) ---
+    LLAMADA_RECHAZADA {
+        es: "llamada rechazada antes de salir a la red: {motivo}",
+        en: "call rejected before reaching the network: {motivo}",
+    }
+    SIN_RED {
+        es: "no se pudo hablar con la API: comprueba la red y reintenta ({detalle})",
+        en: "could not reach the API: check the network and try again ({detalle})",
+    }
+    TLS {
+        es: "fallo de TLS: si la red usa un proxy con su propia CA, instalala en el almacen de certificados del sistema ({detalle})",
+        en: "TLS failure: if the network uses a proxy with its own CA, install it in the system certificate store ({detalle})",
+    }
+    RESPUESTA_ILEGIBLE {
+        es: "respuesta de la API ilegible o demasiado grande ({detalle})",
+        en: "API response unreadable or too large ({detalle})",
+    }
+    DECLARA_MAS_DE {
+        es: "la respuesta declara mas de {limite} bytes",
+        en: "the response declares more than {limite} bytes",
+    }
+    REDIRECCION {
+        es: "la API respondio con una redireccion ({estado}) y no se siguen: revisa la URL de la API o el nombre del repo (renombrado o movido)",
+        en: "the API answered with a redirect ({estado}) and redirects are not followed: check the API URL or the repo name (renamed or moved)",
+    }
+    CLIENTE_HTTP {
+        es: "fallo del cliente HTTP ({detalle})",
+        en: "HTTP client failure ({detalle})",
+    }
+    FORMA_INESPERADA {
+        es: "la respuesta de {plataforma} no tiene la forma esperada ({que})",
+        en: "the {plataforma} response does not have the expected shape ({que})",
+    }
+    RAMA_BASE_O_SHA {
+        es: "rama base o sha del head",
+        en: "base branch or head sha",
+    }
+    RAMA_DESTINO_O_SHA {
+        es: "rama destino o sha del MR",
+        en: "target branch or MR sha",
+    }
+
+    // --- Transporte comun (conexion.rs) ---
+    FALTA_IDENTIDAD {
+        es: "esta operacion corre con la identidad de {rol}: define {variable}{extra}",
+        en: "this operation runs with the {rol} identity: set {variable}{extra}",
+    }
+    SOLO_TERMINAL {
+        es: " (el de persona solo se usa desde un terminal interactivo)",
+        en: " (the persona one is only used from an interactive terminal)",
+    }
+    DEMASIADAS_PAGINAS {
+        es: "la lista tiene mas de {n} elementos: no se evalua a medias",
+        en: "the list has more than {n} elements: it is not evaluated halfway",
+    }
+    PERSONA_FUERA_DE_LISTA {
+        es: "el token de persona es de «{login}», que no esta en la lista de personas del proyecto",
+        en: "the persona token belongs to «{login}», who is not in the project's list of personas",
+    }
+    ROL_NO_VALIDO {
+        es: "{op} corre con la identidad de agente o de persona",
+        en: "{op} runs with the agente or persona identity",
+    }
+
+    // --- Adaptadores (github.rs y gitlab.rs) ---
+    REPO_GITHUB {
+        es: "el repo de GitHub tiene que ser dueno/repo",
+        en: "the GitHub repo must be owner/repo",
+    }
+    PROYECTO_GITLAB {
+        es: "el proyecto de GitLab tiene que ser su id numerico o grupo/proyecto",
+        en: "the GitLab project must be its numeric id or group/project",
+    }
+    RAMA_NO_VALIDA {
+        es: "nombre de rama no valido",
+        en: "invalid branch name",
+    }
+    TITULO_NO_VALIDO {
+        es: "el titulo no puede estar vacio ni llevar controles",
+        en: "the title cannot be empty or contain control characters",
+    }
+    SHA_ESPERADO_COMPLETO {
+        es: "el sha esperado tiene que ser completo (40 hex)",
+        en: "the expected sha must be complete (40 hex)",
+    }
+    SHA_COMPLETO {
+        es: "el sha tiene que ser completo (40 hex)",
+        en: "the sha must be complete (40 hex)",
+    }
+    PR_CERRADO {
+        es: "el PR esta cerrado sin mergear",
+        en: "the PR is closed without being merged",
+    }
+    MR_CERRADO {
+        es: "el MR esta cerrado sin mergear",
+        en: "the MR is closed without being merged",
+    }
+    CI_PUEDE_APROBAR {
+        es: "el token de Actions puede aprobar PR: desactiva «Allow GitHub Actions to create and approve pull requests» en Settings > Actions > General (G16)",
+        en: "the Actions token can approve PRs: turn off «Allow GitHub Actions to create and approve pull requests» in Settings > Actions > General (G16)",
+    }
+    PERMISO_NINGUNO {
+        es: "(ninguno)",
+        en: "(none)",
+    }
+    PERMISO_LECTURA_GITLAB {
+        es: "el scope api (o read_api) y el rol Developer en el proyecto",
+        en: "the api scope (or read_api) and the Developer role in the project",
+    }
+    PERMISO_ESCRITURA_GITLAB {
+        es: "el scope api y el rol Developer en el proyecto",
+        en: "the api scope and the Developer role in the project",
+    }
+    PERMISO_MERGE_GITLAB {
+        es: "el scope api y el rol Maintainer en el proyecto",
+        en: "the api scope and the Maintainer role in the project",
+    }
+    MR_EXISTENTE {
+        es: "GitLab dice que ya hay un MR para la rama pero no se encuentra abierto: reintenta (GitLab: «{api}»)",
+        en: "GitLab says there is already an MR for the branch but no open one is found: try again (GitLab: «{api}»)",
+    }
+    MERGE_NO_PERMITIDO_GITLAB {
+        es: "GitLab no deja mergear a esta cuenta: necesita {permiso} (GitLab: «{api}»)",
+        en: "GitLab does not let this account merge: it needs {permiso} (GitLab: «{api}»)",
+    }
+    ARREGLO_APROBACIONES_GITLAB {
+        es: "GitLab no impone aprobaciones en este proyecto: crea una regla de aprobacion con al menos una aprobacion (requiere Premium o Ultimate) o quita la garantia de la politica",
+        en: "GitLab does not enforce approvals in this project: create an approval rule with at least one approval (requires Premium or Ultimate) or remove the guarantee from the policy",
+    }
+    ARREGLO_RAMA_PROTEGIDA_GITLAB {
+        es: "la rama destino no esta protegida con merge solo para Maintainers y sin force-push: protegela (taskctl setup lo propone)",
+        en: "the target branch is not protected with merge only for Maintainers and no force-push: protect it (taskctl setup proposes it)",
+    }
+
+    // --- Puertas (puertas.rs) ---
+    CHECKS_EN_ROJO {
+        es: "hay checks obligatorios en rojo: corrige y vuelve a empujar",
+        en: "required checks are red: fix and push again",
+    }
+    FALTAN_PUERTAS {
+        es: "faltan puertas: {falta}; espera o atiende lo que falta (al-dia o sin-conflictos: lanza actualizar-rama)",
+        en: "missing gates: {falta}; wait or address what is missing (al-dia or sin-conflictos: run actualizar-rama)",
+    }
+    PR_CERRADO_REABRE {
+        es: "el PR esta cerrado sin mergear: reabrelo o abre otro",
+        en: "the PR is closed without being merged: reopen it or open another one",
+    }
+    RECHAZADA_POR_PLAN {
+        es: "la politica del proyecto exige {garantia} impuesta y el plan de la plataforma no la da: cambia de plan o quita {garantia} de la politica",
+        en: "the project policy requires {garantia} to be enforced and the platform plan does not offer it: change plan or remove {garantia} from the policy",
+    }
+    NO_SE_CUMPLE {
+        es: "{garantia} no se cumple: {arreglo}",
+        en: "{garantia} is not met: {arreglo}",
+    }
+
+    // --- Deteccion (deteccion.rs) ---
+    ORIGIN_NO_ENTENDIDO {
+        es: "no se entiende la URL de origin: tiene que ser https, ssh o scp con dueno y repo",
+        en: "the origin URL cannot be understood: it must be https, ssh or scp with owner and repo",
+    }
+    CLAVE_PLATAFORMA {
+        es: "la clave plataforma vale github o gitlab",
+        en: "the plataforma key must be github or gitlab",
+    }
+    URL_API {
+        es: "URL de la API: {motivo}",
+        en: "API URL: {motivo}",
+    }
+    PLATAFORMA_CONTRADICE {
+        es: "la clave plataforma dice {plataforma} y origin es de {host}",
+        en: "the plataforma key says {plataforma} and origin is on {host}",
+    }
+    API_NO_CAMBIABLE {
+        es: "origin es de {host} y su API es {api}: la clave de la configuracion no puede cambiarla",
+        en: "origin is on {host} and its API is {api}: the configuration key cannot change it",
+    }
+    DECLARA_PLATAFORMA {
+        es: "origin es de {host}: declara plataforma (github o gitlab) y la URL de su API en .taskcode/config.yml",
+        en: "origin is on {host}: declare plataforma (github or gitlab) and its API URL in .taskcode/config.yml",
+    }
+    ORIGIN_GITHUB {
+        es: "en GitHub origin tiene que ser dueno/repo",
+        en: "on GitHub, origin must be owner/repo",
+    }
+
+    // --- Envoltorio HTTP (http.rs) ---
+    HTTP_SOLO_LOCAL {
+        es: "http solo vale para 127.0.0.1 con un puerto y, si acaso, un prefijo valido",
+        en: "http is only valid for 127.0.0.1 with a port and, optionally, a valid prefix",
+    }
+    URL_HTTPS {
+        es: "la URL de la API tiene que ser https",
+        en: "the API URL must be https",
+    }
+    HOST_NO_VALIDO {
+        es: "host de la API no valido (sin usuario ni contrasena)",
+        en: "invalid API host (no user or password)",
+    }
+    PREFIJO_CON_CONSULTA {
+        es: "el prefijo de la API no lleva consulta ni fragmento",
+        en: "the API prefix cannot have a query or fragment",
+    }
+    PREFIJO_NO_VALIDO {
+        es: "prefijo de la API no valido",
+        en: "invalid API prefix",
+    }
+    PAGINACION_OTRO_DESTINO {
+        es: "paginacion hacia otro destino",
+        en: "pagination to another destination",
+    }
+    RUTA_NO_RELATIVA {
+        es: "la ruta tiene que ser relativa a la API",
+        en: "the path must be relative to the API",
+    }
+    RUTA_CODIFICADA {
+        es: "ruta con barra invertida o puntos codificados",
+        en: "path with a backslash or encoded dots",
+    }
+    RUTA_SEGMENTOS {
+        es: "ruta con segmentos vacios, . o ..",
+        en: "path with empty, . or .. segments",
+    }
+    CARACTER_EN_RUTA {
+        es: "caracter no permitido en la ruta",
+        en: "character not allowed in the path",
+    }
+    CARACTER_EN_CONSULTA {
+        es: "caracter no permitido en la consulta",
+        en: "character not allowed in the query",
+    }
+}
diff --git a/crates/tc2/src/i18n/guarda.rs b/crates/tc2/src/i18n/guarda.rs
new file mode 100644
index 0000000..7fb0cb5
--- /dev/null
+++ b/crates/tc2/src/i18n/guarda.rs
@@ -0,0 +1,83 @@
+//! Test de guarda de F2-8: ningun texto dirigido a la persona se escribe
+//! fuera del catalogo. Lee el codigo de `src/` (sin los modulos de tests) y
+//! busca las formas de colar un literal en un diagnostico.
+
+use std::collections::BTreeMap;
+use std::path::{Path, PathBuf};
+
+/// Usos permitidos de `Mensaje::externo(` por fichero: texto que viene de
+/// fuera (git, el sistema, HTTP) sin nada que traducir alrededor. Lista
+/// cerrada: un uso nuevo tiene que anadirse aqui a proposito.
+const EXTERNOS: &[(&str, usize)] = &[
+    // Fallo al leer un blob sin diagnostico: queda la linea de contrato, que
+    // no se traduce (ADR F1-4).
+    ("estado/lectura.rs", 1),
+];
+
+/// Formas de meter un literal en un diagnostico sin pasar por el catalogo.
+const PROHIBIDOS: &[&str] = &[
+    ".diagnostico(\"",
+    ".diagnostico(format!",
+    "Mensaje::externo(\"",
+    "Mensaje::externo(format!",
+];
+
+fn fuentes(dir: &Path, base: &Path, v: &mut Vec<(String, String)>) {
+    let Ok(entradas) = std::fs::read_dir(dir) else {
+        return;
+    };
+    for e in entradas.flatten() {
+        let p = e.path();
+        if p.is_dir() {
+            fuentes(&p, base, v);
+        } else if p.extension().is_some_and(|x| x == "rs") {
+            let rel = p
+                .strip_prefix(base)
+                .unwrap_or(&p)
+                .to_string_lossy()
+                .replace('\\', "/");
+            if rel.starts_with("i18n/") {
+                continue;
+            }
+            let texto = std::fs::read_to_string(&p).unwrap_or_default();
+            // Fuera el modulo de tests del final del fichero.
+            let codigo = match texto.find("#[cfg(test)]\nmod tests") {
+                Some(i) => texto[..i].to_owned(),
+                None => texto,
+            };
+            v.push((rel, codigo));
+        }
+    }
+}
+
+#[test]
+fn ningun_literal_fuera_del_catalogo() {
+    let base: PathBuf = [env!("CARGO_MANIFEST_DIR"), "src"].iter().collect();
+    let mut v = Vec::new();
+    fuentes(&base, &base, &mut v);
+    assert!(
+        v.len() > 10,
+        "no se encontro el codigo en {}",
+        base.display()
+    );
+    let mut fallos = Vec::new();
+    let mut externos: BTreeMap<&str, usize> = BTreeMap::new();
+    for (rel, codigo) in &v {
+        for p in PROHIBIDOS {
+            if codigo.contains(p) {
+                fallos.push(format!("{rel}: usa `{p}`: pasa el texto al catalogo"));
+            }
+        }
+        let n = codigo.matches("Mensaje::externo(").count();
+        if n > 0 {
+            externos.insert(rel.as_str(), n);
+        }
+    }
+    let permitidos: BTreeMap<&str, usize> = EXTERNOS.iter().copied().collect();
+    if externos != permitidos {
+        fallos.push(format!(
+            "usos de Mensaje::externo distintos de EXTERNOS: hay {externos:?}, se permiten {permitidos:?}"
+        ));
+    }
+    assert!(fallos.is_empty(), "{}", fallos.join("\n"));
+}
diff --git a/crates/tc2/src/i18n/mod.rs b/crates/tc2/src/i18n/mod.rs
new file mode 100644
index 0000000..ff78158
--- /dev/null
+++ b/crates/tc2/src/i18n/mod.rs
@@ -0,0 +1,326 @@
+//! Mensajes en espanol e ingles (F2-8, RF-73).
+//!
+//! El idioma se decide en un solo sitio, el borde del proceso (`main.rs`,
+//! con [`Idioma::de_entorno`]). El resto del codigo no escribe textos: arma
+//! un [`Mensaje`] neutro (un [`Texto`] del catalogo mas sus argumentos) y el
+//! texto se genera al escribir la salida, en el idioma que toque.
+//!
+//! Lo que no cambia con el idioma (ADR F1-4): la linea de stdout, las claves
+//! del JSON, los valores de `motivo` y los codigos de salida. Solo se traduce
+//! el diagnostico.
+//!
+//! El catalogo ([`catalogo`]) se escribe con la macro `textos!`, que exige
+//! las dos traducciones de cada entrada: si falta una, no compila. El test
+//! `catalogo_completo` comprueba ademas que no estan vacias y que llevan los
+//! mismos huecos `{nombre}`.
+
+pub mod catalogo;
+#[cfg(test)]
+mod guarda;
+
+use std::fmt::Write as _;
+
+/// Idioma de los mensajes dirigidos a la persona.
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
+pub enum Idioma {
+    /// Espanol: el de por defecto (RF-73).
+    #[default]
+    Es,
+    /// Ingles.
+    En,
+}
+
+/// Variable que fija el idioma de `tc2` por encima del locale del sistema.
+pub const VARIABLE: &str = "TC2_IDIOMA";
+
+impl Idioma {
+    /// Resuelve el idioma desde el entorno: `TC2_IDIOMA` (`es` o `en`), luego
+    /// `LC_ALL`, `LC_MESSAGES` y `LANG` (por su prefijo `es` o `en`; `C`,
+    /// `POSIX` y lo que no se reconoce se saltan) y, si no hay nada, espanol.
+    ///
+    /// El entorno llega como funcion de lectura para poder probarlo sin
+    /// tocar las variables del proceso (`set_var` exige `unsafe` en edition
+    /// 2024 y el workspace lo prohibe).
+    pub fn de_entorno(leer: impl Fn(&str) -> Option<String>) -> Self {
+        [VARIABLE, "LC_ALL", "LC_MESSAGES", "LANG"]
+            .iter()
+            .find_map(|v| leer(v).and_then(|valor| Self::de_valor(&valor)))
+            .unwrap_or_default()
+    }
+
+    /// Idioma de un valor tipo locale (`en`, `en_US.UTF-8`, `es-419`,
+    /// `EN`...). `None` si no es espanol ni ingles.
+    fn de_valor(valor: &str) -> Option<Self> {
+        let base = valor
+            .trim()
+            .split(['_', '-', '.', '@'])
+            .next()
+            .unwrap_or_default()
+            .to_ascii_lowercase();
+        match base.as_str() {
+            "es" => Some(Self::Es),
+            "en" => Some(Self::En),
+            _ => None,
+        }
+    }
+}
+
+/// Una entrada del catalogo: el mismo mensaje en los dos idiomas. Los huecos
+/// se escriben `{nombre}` y se rellenan con [`Mensaje::con`].
+#[derive(Debug, PartialEq, Eq)]
+pub struct Texto {
+    /// Identificador unico: `modulo::NOMBRE`.
+    pub clave: &'static str,
+    /// Texto en espanol.
+    pub es: &'static str,
+    /// Texto en ingles.
+    pub en: &'static str,
+}
+
+impl Texto {
+    /// La plantilla en un idioma.
+    pub fn en_idioma(&self, idioma: Idioma) -> &'static str {
+        match idioma {
+            Idioma::Es => self.es,
+            Idioma::En => self.en,
+        }
+    }
+}
+
+/// Valor de un hueco: texto tal cual o otro mensaje, que se traduce en el
+/// mismo idioma que el que lo contiene.
+#[derive(Debug, Clone, PartialEq, Eq)]
+enum Arg {
+    Texto(String),
+    Mensaje(Mensaje),
+}
+
+/// Mensaje neutro: una entrada del catalogo mas sus argumentos. El texto se
+/// genera con [`Mensaje::texto`] en el idioma que se pida.
+///
+/// No hay `From<String>`: un texto solo entra como argumento de un hueco o,
+/// si viene de fuera (stderr de git, error del sistema, HTTP) y no hay nada
+/// que traducir alrededor, con [`Mensaje::externo`].
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Mensaje {
+    texto: &'static Texto,
+    args: Vec<(&'static str, Arg)>,
+}
+
+impl Mensaje {
+    /// Mensaje sin argumentos todavia.
+    pub fn nuevo(texto: &'static Texto) -> Self {
+        Self {
+            texto,
+            args: Vec::new(),
+        }
+    }
+
+    /// Texto que viene de fuera de `tc2` y no se traduce: el mensaje entero
+    /// es ese texto.
+    pub fn externo(texto: impl Into<String>) -> Self {
+        Self::nuevo(&catalogo::comun::EXTERNO).con("texto", texto)
+    }
+
+    /// Rellena el hueco `{nombre}` con un valor tal cual (una ruta, un hash,
+    /// un mensaje de git...).
+    #[must_use]
+    pub fn con(mut self, nombre: &'static str, valor: impl Into<String>) -> Self {
+        self.args.push((nombre, Arg::Texto(valor.into())));
+        self
+    }
+
+    /// Rellena el hueco `{nombre}` con otro mensaje, traducido en el mismo
+    /// idioma.
+    #[must_use]
+    pub fn con_mensaje(mut self, nombre: &'static str, valor: impl Into<Mensaje>) -> Self {
+        self.args.push((nombre, Arg::Mensaje(valor.into())));
+        self
+    }
+
+    /// Clave del catalogo de la que sale.
+    pub fn clave(&self) -> &'static str {
+        self.texto.clave
+    }
+
+    /// El texto en un idioma, con los huecos rellenos. Un hueco sin valor se
+    /// deja tal cual (`{nombre}`), y lo que traen los valores no se vuelve a
+    /// sustituir.
+    pub fn texto(&self, idioma: Idioma) -> String {
+        let plantilla = self.texto.en_idioma(idioma);
+        let mut s = String::with_capacity(plantilla.len() + 16);
+        let mut resto = plantilla;
+        while let Some(ini) = resto.find('{') {
+            s.push_str(&resto[..ini]);
+            let tras = &resto[ini + 1..];
+            match tras.find('}') {
+                Some(fin) if es_nombre(&tras[..fin]) => {
+                    let nombre = &tras[..fin];
+                    match self.args.iter().find(|(n, _)| *n == nombre) {
+                        Some((_, Arg::Texto(t))) => s.push_str(t),
+                        Some((_, Arg::Mensaje(m))) => s.push_str(&m.texto(idioma)),
+                        None => {
+                            let _ = write!(s, "{{{nombre}}}");
+                        }
+                    }
+                    resto = &tras[fin + 1..];
+                }
+                _ => {
+                    s.push('{');
+                    resto = tras;
+                }
+            }
+        }
+        s.push_str(resto);
+        s
+    }
+}
+
+impl From<&'static Texto> for Mensaje {
+    fn from(texto: &'static Texto) -> Self {
+        Self::nuevo(texto)
+    }
+}
+
+/// Nombre valido de hueco: letras ASCII minusculas, digitos y `_`.
+fn es_nombre(t: &str) -> bool {
+    !t.is_empty()
+        && t.chars()
+            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
+}
+
+/// Huecos `{nombre}` de una plantilla, ordenados y sin repetir.
+pub fn huecos(plantilla: &str) -> Vec<&str> {
+    let mut v = Vec::new();
+    let mut resto = plantilla;
+    while let Some(ini) = resto.find('{') {
+        let tras = &resto[ini + 1..];
+        match tras.find('}') {
+            Some(fin) if es_nombre(&tras[..fin]) => {
+                v.push(&tras[..fin]);
+                resto = &tras[fin + 1..];
+            }
+            _ => resto = tras,
+        }
+    }
+    v.sort_unstable();
+    v.dedup();
+    v
+}
+
+/// Define entradas del catalogo. Cada una exige `es` y `en`: si falta una,
+/// la macro no casa y no compila. Genera ademas `TODOS`, la lista de las
+/// entradas del modulo, que recorre el test del catalogo.
+macro_rules! textos {
+    ($( $(#[$meta:meta])* $nombre:ident { es: $es:literal, en: $en:literal $(,)? } )*) => {
+        $(
+            $(#[$meta])*
+            pub static $nombre: $crate::i18n::Texto = $crate::i18n::Texto {
+                clave: concat!(module_path!(), "::", stringify!($nombre)),
+                es: $es,
+                en: $en,
+            };
+        )*
+        /// Todas las entradas de este modulo del catalogo.
+        pub static TODOS: &[&$crate::i18n::Texto] = &[$(&$nombre),*];
+    };
+}
+pub(crate) use textos;
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use std::collections::HashSet;
+
+    fn entorno<'a>(pares: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
+        move |k| {
+            pares
+                .iter()
+                .find(|(n, _)| *n == k)
+                .map(|(_, v)| (*v).to_owned())
+        }
+    }
+
+    #[test]
+    fn resolucion_de_idioma() {
+        let casos: &[(&[(&str, &str)], Idioma)] = &[
+            (&[], Idioma::Es),
+            (&[("LANG", "en_US.UTF-8")], Idioma::En),
+            (&[("LANG", "es_ES.UTF-8")], Idioma::Es),
+            (&[("LANG", "C")], Idioma::Es),
+            (&[("LANG", "POSIX")], Idioma::Es),
+            (&[("LANG", "fr_FR.UTF-8")], Idioma::Es),
+            (&[("LANG", "")], Idioma::Es),
+            (&[("LANG", "en_US@euro")], Idioma::En),
+            (&[("LANG", "es-419")], Idioma::Es),
+            // Prioridad: TC2_IDIOMA > LC_ALL > LC_MESSAGES > LANG.
+            (&[("TC2_IDIOMA", "es"), ("LC_ALL", "en_US")], Idioma::Es),
+            (&[("TC2_IDIOMA", "EN"), ("LANG", "es_ES")], Idioma::En),
+            (&[("LC_ALL", "en_GB"), ("LANG", "es_ES")], Idioma::En),
+            (&[("LC_MESSAGES", "es_ES"), ("LANG", "en_US")], Idioma::Es),
+            // Un valor que no se reconoce se salta y manda el siguiente.
+            (&[("TC2_IDIOMA", "klingon"), ("LANG", "en_US")], Idioma::En),
+            (&[("LC_ALL", "C"), ("LANG", "en_US")], Idioma::En),
+        ];
+        for (pares, esperado) in casos {
+            assert_eq!(Idioma::de_entorno(entorno(pares)), *esperado, "{pares:?}");
+        }
+    }
+
+    #[test]
+    fn huecos_y_sustitucion() {
+        static T: Texto = Texto {
+            clave: "prueba::T",
+            es: "{op}: no existe {ruta} {x-y} {",
+            en: "{op}: {ruta} does not exist {x-y} {",
+        };
+        let m = Mensaje::nuevo(&T).con("op", "push").con("ruta", "{op}");
+        assert_eq!(m.texto(Idioma::Es), "push: no existe {op} {x-y} {");
+        assert_eq!(m.texto(Idioma::En), "push: {op} does not exist {x-y} {");
+        assert_eq!(huecos(T.es), vec!["op", "ruta"]);
+        let anidado = Mensaje::nuevo(&catalogo::comun::EXTERNO).con_mensaje("texto", m);
+        assert_eq!(
+            anidado.texto(Idioma::En),
+            "push: {op} does not exist {x-y} {"
+        );
+    }
+
+    /// Criterio de F2-8: falla si falta una traduccion. Que falte una de las
+    /// dos ya no compila (macro `textos!`); aqui se comprueba que ninguna esta
+    /// vacia, que el ingles no es el espanol copiado (salvo una lista
+    /// cerrada), que llevan los mismos huecos y que las claves no se repiten.
+    #[test]
+    fn catalogo_completo() {
+        // Entradas que pueden ser iguales en los dos idiomas: solo un hueco o
+        // solo nombres de ordenes.
+        const IGUALES: &[&str] = &["tc2::i18n::catalogo::comun::EXTERNO"];
+        let mut claves = HashSet::new();
+        let mut total = 0;
+        for t in catalogo::todos() {
+            total += 1;
+            assert!(claves.insert(t.clave), "clave repetida: {}", t.clave);
+            assert!(
+                !t.es.trim().is_empty(),
+                "{}: falta el texto en espanol",
+                t.clave
+            );
+            assert!(
+                !t.en.trim().is_empty(),
+                "{}: falta el texto en ingles",
+                t.clave
+            );
+            assert!(
+                t.es != t.en || IGUALES.contains(&t.clave),
+                "{}: el texto en ingles es el mismo que en espanol (sin traducir)",
+                t.clave
+            );
+            assert_eq!(
+                huecos(t.es),
+                huecos(t.en),
+                "{}: los huecos no coinciden entre idiomas",
+                t.clave
+            );
+        }
+        assert!(total > 1, "el catalogo esta vacio");
+    }
+}
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index c873886..7c98b7f 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -6,7 +6,8 @@
 //! ID, F2-4) [`tarea`] (modelo de tarea y frontmatter YAML, F2-5) y
 //! [`plataforma`] (puerto Plataforma y adaptadores GitHub y GitLab, F2-6 y
 //! F2-9), [`carril`] (tabla de suelo de F1-10) y [`config`] (configuracion
-//! con esquema validado, F2-7). Las capas
+//! con esquema validado, F2-7) e [`i18n`] (mensajes en espanol e ingles,
+//! F2-8). Las capas
 //! que vienen entran como modulos de esta biblioteca.
 
 pub mod alta;
@@ -15,11 +16,14 @@ pub mod config;
 pub mod contrato;
 pub mod estado;
 pub mod git;
+pub mod i18n;
 pub mod plataforma;
 pub mod tarea;
 mod yaml;
 
 use contrato::{Informe, Resultado};
+use i18n::catalogo::cli as txt;
+use i18n::{Idioma, Mensaje};
 
 /// Version del CLI, la de `Cargo.toml`.
 pub const VERSION: &str = env!("CARGO_PKG_VERSION");
@@ -36,12 +40,19 @@ pub struct Salida {
     pub stderr: String,
 }
 
-/// Uso del binario, para los mensajes de error.
-const USO: &str = "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\"";
-
 /// Interpreta los argumentos (sin el nombre del programa) y ejecuta la orden
-/// en la carpeta actual.
+/// en la carpeta actual, con los mensajes en espanol.
 pub fn ejecutar<I, S>(args: I) -> Salida
+where
+    I: IntoIterator<Item = S>,
+    S: AsRef<str>,
+{
+    ejecutar_en(Idioma::Es, args)
+}
+
+/// Como [`ejecutar`], con los mensajes dirigidos a la persona en `idioma`
+/// (F2-8). stdout y el codigo de salida no cambian con el idioma.
+pub fn ejecutar_en<I, S>(idioma: Idioma, args: I) -> Salida
 where
     I: IntoIterator<Item = S>,
     S: AsRef<str>,
@@ -61,29 +72,36 @@ where
                 [j, r @ ..] if j == "--json" => (true, r),
                 r => (false, r),
             };
-            orden_git(orden, resto).salida(json)
+            orden_git(orden, resto).salida_en(json, idioma)
         }
         [orden, resto @ ..] if orden == alta::NOMBRE => {
             let (json, resto) = match resto {
                 [j, r @ ..] if j == "--json" => (true, r),
                 r => (false, r),
             };
-            orden_new(resto).salida(json)
+            orden_new(resto).salida_en(json, idioma)
         }
         [] => Salida {
             codigo: 2,
             stdout: String::new(),
             stderr: format!(
-                "tc2: falta la orden. {USO}
-"
+                "{}
+",
+                Mensaje::nuevo(&txt::FALTA_LA_ORDEN)
+                    .con_mensaje("uso", &txt::USO)
+                    .texto(idioma)
             ),
         },
         [otro, ..] => Salida {
             codigo: 2,
             stdout: String::new(),
             stderr: format!(
-                "tc2: orden desconocida «{otro}». {USO}
-"
+                "{}
+",
+                Mensaje::nuevo(&txt::ORDEN_DESCONOCIDA)
+                    .con("orden", otro.as_str())
+                    .con_mensaje("uso", &txt::USO)
+                    .texto(idioma)
             ),
         },
     }
@@ -95,7 +113,7 @@ fn orden_git(orden: &str, args: &[String]) -> Informe {
         Ok(d) => d,
         Err(e) => {
             return Informe::nuevo(orden, Resultado::Error)
-                .diagnostico(format!("no se puede leer la carpeta actual: {e}"));
+                .diagnostico(Mensaje::nuevo(&txt::CARPETA_ACTUAL).con("error", e.to_string()));
         }
     };
     let g = git::Git::nuevo();
@@ -103,7 +121,7 @@ fn orden_git(orden: &str, args: &[String]) -> Informe {
         let Some((mensaje, rutas)) = args.split_first() else {
             return Informe::nuevo(orden, Resultado::Uso)
                 .con("motivo", "sin-mensaje")
-                .diagnostico("uso: guardar \"<mensaje>\" <rutas...>");
+                .diagnostico(&txt::USO_GUARDAR);
         };
         let rutas: Vec<&str> = rutas.iter().map(String::as_str).collect();
         git::guardar(&g, &dir, mensaje, &rutas)
@@ -112,7 +130,7 @@ fn orden_git(orden: &str, args: &[String]) -> Informe {
     } else {
         Informe::nuevo(orden, Resultado::Uso)
             .con("motivo", "argumentos-de-mas")
-            .diagnostico("uso: actualizar-rama [--json]")
+            .diagnostico(&txt::USO_ACTUALIZAR_RAMA)
     }
 }
 
@@ -130,26 +148,24 @@ fn orden_new(args: &[String]) -> Informe {
         [_] => {
             return Informe::nuevo(alta::NOMBRE, Resultado::Uso)
                 .con("motivo", "titulo-invalido")
-                .diagnostico(
-                    "el titulo no puede estar vacio ni llevar saltos de linea, tabuladores ni otros caracteres de control",
-                );
+                .diagnostico(&txt::TITULO_INVALIDO);
         }
         [] => {
             return Informe::nuevo(alta::NOMBRE, Resultado::Uso)
                 .con("motivo", "sin-titulo")
-                .diagnostico("uso: new [--json] \"<titulo>\"");
+                .diagnostico(&txt::USO_NEW);
         }
         _ => {
             return Informe::nuevo(alta::NOMBRE, Resultado::Uso)
                 .con("motivo", "argumentos-de-mas")
-                .diagnostico("pon el titulo entre comillas: new [--json] \"<titulo>\"");
+                .diagnostico(&txt::TITULO_ENTRE_COMILLAS);
         }
     };
     let dir = match std::env::current_dir() {
         Ok(d) => d,
         Err(e) => {
             return Informe::nuevo(alta::NOMBRE, Resultado::Error)
-                .diagnostico(format!("no se puede leer la carpeta actual: {e}"));
+                .diagnostico(Mensaje::nuevo(&txt::CARPETA_ACTUAL).con("error", e.to_string()));
         }
     };
     let g = git::Git::nuevo();
diff --git a/crates/tc2/src/main.rs b/crates/tc2/src/main.rs
index 3625b80..3bbfb15 100644
--- a/crates/tc2/src/main.rs
+++ b/crates/tc2/src/main.rs
@@ -1,20 +1,28 @@
 //! Punto de entrada de `tc2`: delega en la biblioteca y solo escribe.
+//!
+//! Es el unico sitio que lee el idioma del entorno (F2-8): `TC2_IDIOMA`,
+//! luego `LC_ALL`, `LC_MESSAGES` y `LANG`, y si no, espanol.
 
 use std::io::Write;
 use std::process::ExitCode;
 
+use tc2::i18n::catalogo::cli as txt;
+use tc2::i18n::{Idioma, Mensaje};
+
 fn main() -> ExitCode {
+    let idioma = Idioma::de_entorno(|k| std::env::var(k).ok());
     let mut args = Vec::new();
     for a in std::env::args_os().skip(1) {
         match a.into_string() {
             Ok(s) => args.push(s),
             Err(_) => {
-                let _ = writeln!(std::io::stderr(), "tc2: argumento que no es UTF-8");
+                let m = Mensaje::nuevo(&txt::ARGUMENTO_NO_UTF8);
+                let _ = writeln!(std::io::stderr(), "{}", m.texto(idioma));
                 return ExitCode::from(2);
             }
         }
     }
-    let salida = tc2::ejecutar(args);
+    let salida = tc2::ejecutar_en(idioma, args);
     // Si no se puede escribir la salida (stdout cerrado), no se da por bueno:
     // quien lee el codigo 0 creeria que recibio la respuesta.
     let mut stdout = std::io::stdout();
@@ -22,7 +30,8 @@ fn main() -> ExitCode {
         .write_all(salida.stdout.as_bytes())
         .and_then(|()| stdout.flush())
     {
-        let _ = writeln!(std::io::stderr(), "tc2: no se pudo escribir la salida: {e}");
+        let m = Mensaje::nuevo(&txt::NO_SE_PUDO_ESCRIBIR).con("error", e.to_string());
+        let _ = writeln!(std::io::stderr(), "{}", m.texto(idioma));
         return ExitCode::from(1);
     }
     // Un fallo al escribir en stderr no tiene a donde avisarse.
diff --git a/crates/tc2/src/plataforma/conexion.rs b/crates/tc2/src/plataforma/conexion.rs
index 4c6c49c..4a8306b 100644
--- a/crates/tc2/src/plataforma/conexion.rs
+++ b/crates/tc2/src/plataforma/conexion.rs
@@ -12,10 +12,12 @@ use std::collections::BTreeMap;
 use serde::de::DeserializeOwned;
 
 use super::credenciales::{Credenciales, Rol, Secreto};
-use super::errores;
+use super::errores::{self, Permiso};
 use super::http::{Base, Cliente, Metodo, Respuesta};
 use super::{Paso, Personas, Proveedor, Usuario};
 use crate::contrato::{Informe, Resultado};
+use crate::i18n::catalogo::plataforma as t;
+use crate::i18n::{Mensaje, Texto};
 
 /// Paginas como mucho al seguir `Link rel=next` (100 elementos cada una).
 pub const MAX_PAGINAS: usize = 10;
@@ -73,19 +75,18 @@ impl Conexion {
     /// de otro rol).
     pub fn token(&self, op: &str, rol: Rol) -> Paso<Secreto> {
         self.credenciales.token(rol).ok_or_else(|| {
-            let extra = if rol == Rol::Persona {
-                " (el de persona solo se usa desde un terminal interactivo)"
+            let m = Mensaje::nuevo(&t::FALTA_IDENTIDAD)
+                .con("rol", rol.nombre())
+                .con("variable", rol.variable());
+            let m = if rol == Rol::Persona {
+                m.con_mensaje("extra", &t::SOLO_TERMINAL)
             } else {
-                ""
+                m.con("extra", "")
             };
             Informe::nuevo(op, Resultado::Rechazada)
                 .con("motivo", "identidad")
                 .con("falta", rol.nombre())
-                .diagnostico(format!(
-                    "esta operacion corre con la identidad de {}: define {}{extra}",
-                    rol.nombre(),
-                    rol.variable()
-                ))
+                .diagnostico(m)
         })
     }
 
@@ -104,15 +105,27 @@ impl Conexion {
     }
 
     /// Traduce una respuesta que no es 2xx.
-    pub fn traducir(&self, op: &str, rol: Rol, permiso: &str, r: &Respuesta) -> Informe {
-        errores::traducir(self.proveedor, op, rol, permiso, r)
+    pub fn traducir(
+        &self,
+        op: &str,
+        rol: Rol,
+        permiso: impl Into<Permiso>,
+        r: &Respuesta,
+    ) -> Informe {
+        errores::traducir(self.proveedor, op, rol, permiso.into(), r)
     }
 
-    /// Respuesta 2xx con una forma inesperada.
+    /// Respuesta 2xx con una forma inesperada; `que` va tal cual (el error
+    /// del lector JSON o el nombre de un campo).
     pub fn invalida(&self, op: &str, que: &str) -> Informe {
         errores::respuesta_invalida(self.proveedor, op, que)
     }
 
+    /// Respuesta 2xx con una forma inesperada, descrita en el catalogo.
+    pub fn invalida_por(&self, op: &str, que: &'static Texto) -> Informe {
+        errores::respuesta_invalida_por(self.proveedor, op, que)
+    }
+
     /// Lee un cuerpo con la forma `T`.
     pub fn parsear<T: DeserializeOwned>(&self, op: &str, cuerpo: &str) -> Paso<T> {
         serde_json::from_str(cuerpo)
@@ -124,7 +137,7 @@ impl Conexion {
         &self,
         op: &str,
         rol: Rol,
-        permiso: &str,
+        permiso: impl Into<Permiso>,
         ep: (Metodo, String),
     ) -> Paso<T> {
         let r = self.pedir(op, rol, ep, None)?;
@@ -140,13 +153,14 @@ impl Conexion {
         &self,
         op: &str,
         rol: Rol,
-        permiso: &str,
+        permiso: impl Into<Permiso>,
         primera: String,
         mut extraer: F,
     ) -> Paso<Vec<T>>
     where
         F: FnMut(&str) -> Paso<Vec<T>>,
     {
+        let permiso = permiso.into();
         let token = self.token(op, rol)?;
         let mut ruta = primera;
         let mut todo = Vec::new();
@@ -171,10 +185,9 @@ impl Conexion {
         }
         Err(Informe::nuevo(op, Resultado::Error)
             .con("motivo", "demasiadas-paginas")
-            .diagnostico(format!(
-                "la lista tiene mas de {} elementos: no se evalua a medias",
-                MAX_PAGINAS * 100
-            )))
+            .diagnostico(
+                Mensaje::nuevo(&t::DEMASIADAS_PAGINAS).con("n", (MAX_PAGINAS * 100).to_string()),
+            ))
     }
 
     /// Identidad del rol, de la cache o leida con `leer` (una vez por rol y
@@ -200,10 +213,10 @@ impl Conexion {
             Err(Informe::nuevo(op, Resultado::Rechazada)
                 .con("motivo", "identidad")
                 .con("rol", "persona")
-                .diagnostico(format!(
-                    "el token de persona es de «{}», que no esta en la lista de personas del proyecto",
-                    errores::sanear(&u.login)
-                )))
+                .diagnostico(
+                    Mensaje::nuevo(&t::PERSONA_FUERA_DE_LISTA)
+                        .con("login", errores::sanear(&u.login)),
+                ))
         }
     }
 }
@@ -228,10 +241,10 @@ pub fn sha_valido(s: &str) -> bool {
 }
 
 /// Informe de uso (2) con su motivo.
-pub fn uso(op: &str, motivo: &str, diag: &str) -> Informe {
+pub fn uso(op: &str, motivo: &str, diag: &'static Texto) -> Informe {
     Informe::nuevo(op, Resultado::Uso)
         .con("motivo", motivo)
-        .diagnostico(diag.to_owned())
+        .diagnostico(diag)
 }
 
 /// `merge-pr` rechazado porque el head no es el evaluado.
@@ -239,9 +252,7 @@ pub fn head_cambiado(pr: u64) -> Informe {
     Informe::nuevo("merge-pr", Resultado::Rechazada)
         .con("pr", pr.to_string())
         .con("motivo", "head-cambiado")
-        .diagnostico(
-            "la rama ha cambiado desde que se evaluaron las puertas: vuelve a lanzar estado-puertas",
-        )
+        .diagnostico(Mensaje::nuevo(&t::HEAD_CAMBIADO).con("cita", ""))
 }
 
 /// Rechazo de un rol que no corresponde a la operacion.
@@ -249,9 +260,7 @@ pub fn rol_no_valido(op: &str, rol: Rol) -> Informe {
     Informe::nuevo(op, Resultado::Rechazada)
         .con("motivo", "identidad")
         .con("rol", rol.nombre())
-        .diagnostico(format!(
-            "{op} corre con la identidad de agente o de persona"
-        ))
+        .diagnostico(Mensaje::nuevo(&t::ROL_NO_VALIDO).con("op", op))
 }
 
 #[cfg(test)]
diff --git a/crates/tc2/src/plataforma/deteccion.rs b/crates/tc2/src/plataforma/deteccion.rs
index 34366aa..1d14863 100644
--- a/crates/tc2/src/plataforma/deteccion.rs
+++ b/crates/tc2/src/plataforma/deteccion.rs
@@ -8,6 +8,8 @@
 
 use super::http::Base;
 use crate::contrato::{Informe, Resultado};
+use crate::i18n::catalogo::plataforma as t;
+use crate::i18n::{Mensaje, Texto};
 
 /// Nombre de la operacion en los errores.
 const OP: &str = "detectar-plataforma";
@@ -31,9 +33,7 @@ pub fn detectar(url_origin: &str, clave: Option<(&str, &str)>) -> Result<Destino
     let (host, proyecto) = partir_url(url_origin).ok_or_else(|| {
         Informe::nuevo(OP, Resultado::Rechazada)
             .con("motivo", "origin")
-            .diagnostico(
-                "no se entiende la URL de origin: tiene que ser https, ssh o scp con dueno y repo",
-            )
+            .diagnostico(&t::ORIGIN_NO_ENTENDIDO)
     })?;
     let por_host = match host.as_str() {
         "github.com" => Some(Proveedor::GitHub),
@@ -47,43 +47,46 @@ pub fn detectar(url_origin: &str, clave: Option<(&str, &str)>) -> Result<Destino
                 "github" => Proveedor::GitHub,
                 "gitlab" => Proveedor::GitLab,
                 _ => {
-                    return Err(rechazo("la clave plataforma vale github o gitlab"));
+                    return Err(rechazo(&t::CLAVE_PLATAFORMA));
                 }
             };
-            let base = Base::nueva(url).map_err(|m| rechazo(&format!("URL de la API: {m}")))?;
+            let base = Base::nueva(url)
+                .map_err(|m| rechazo(Mensaje::nuevo(&t::URL_API).con_mensaje("motivo", m)))?;
             Some((prov, base))
         }
     };
     let (proveedor, api) = match (por_host, declarado) {
         (Some(h), Some((d, _))) if h != d => {
-            return Err(rechazo(&format!(
-                "la clave plataforma dice {} y origin es de {host}",
-                d.nombre()
-            )));
+            return Err(rechazo(
+                Mensaje::nuevo(&t::PLATAFORMA_CONTRADICE)
+                    .con("plataforma", d.nombre())
+                    .con("host", host),
+            ));
         }
         // Con un host conocido, la API es la oficial: una clave que apunte a
         // otra mandaria los tokens fuera (falla cerrado, decision 7).
         (Some(h), Some((_, base))) => {
-            let oficial = api_oficial(h).map_err(|m| rechazo(&m))?;
+            let oficial = api_oficial(h).map_err(rechazo)?;
             if base != oficial {
-                return Err(rechazo(&format!(
-                    "origin es de {host} y su API es {}: la clave de la configuracion no puede cambiarla",
-                    oficial.texto()
-                )));
+                return Err(rechazo(
+                    Mensaje::nuevo(&t::API_NO_CAMBIABLE)
+                        .con("host", host)
+                        .con("api", oficial.texto()),
+                ));
             }
             (h, base)
         }
-        (Some(h), None) => (h, api_oficial(h).map_err(|m| rechazo(&m))?),
+        (Some(h), None) => (h, api_oficial(h).map_err(rechazo)?),
         (None, Some((d, base))) => (d, base),
         (None, None) => {
-            return Err(rechazo(&format!(
-                "origin es de {host}: declara plataforma (github o gitlab) y la URL de su API en .taskcode/config.yml"
-            )));
+            return Err(rechazo(
+                Mensaje::nuevo(&t::DECLARA_PLATAFORMA).con("host", host),
+            ));
         }
     };
     let segmentos = proyecto.split('/').count();
     if proveedor == Proveedor::GitHub && segmentos != 2 {
-        return Err(rechazo("en GitHub origin tiene que ser dueno/repo"));
+        return Err(rechazo(&t::ORIGIN_GITHUB));
     }
     Ok(Destino {
         proveedor,
@@ -93,17 +96,17 @@ pub fn detectar(url_origin: &str, clave: Option<(&str, &str)>) -> Result<Destino
 }
 
 /// API publica de un host conocido.
-fn api_oficial(p: Proveedor) -> Result<Base, String> {
+fn api_oficial(p: Proveedor) -> Result<Base, &'static Texto> {
     match p {
         Proveedor::GitHub => Ok(Base::github()),
         Proveedor::GitLab => Base::nueva("https://gitlab.com/api/v4"),
     }
 }
 
-fn rechazo(m: &str) -> Informe {
+fn rechazo(m: impl Into<Mensaje>) -> Informe {
     Informe::nuevo(OP, Resultado::Rechazada)
         .con("motivo", "plataforma")
-        .diagnostico(m.to_owned())
+        .diagnostico(m)
 }
 
 /// `(host, proyecto)` de una URL https, ssh o scp, sin `.git` ni barras.
diff --git a/crates/tc2/src/plataforma/errores.rs b/crates/tc2/src/plataforma/errores.rs
index 8a70cb1..50e6cc2 100644
--- a/crates/tc2/src/plataforma/errores.rs
+++ b/crates/tc2/src/plataforma/errores.rs
@@ -9,8 +9,10 @@
 
 use super::Proveedor;
 use super::credenciales::Rol;
-use super::http::{FalloHttp, Respuesta};
+use super::http::{FalloHttp, LIMITE_CUERPO, Respuesta};
 use crate::contrato::{Informe, Resultado};
+use crate::i18n::catalogo::plataforma as t;
+use crate::i18n::{Mensaje, Texto};
 
 /// Aviso de GitHub cuando el plan no da rulesets ni proteccion de rama.
 pub const AVISO_PLAN: &str = "Upgrade to GitHub Pro";
@@ -76,97 +78,132 @@ fn es_numero(s: &str) -> bool {
     !s.is_empty() && s.len() <= 10 && s.bytes().all(|b| b.is_ascii_digit())
 }
 
+/// Permiso que necesita una operacion, para el diagnostico del 403: un
+/// alcance de la API tal cual (`contents:write`) o una descripcion del
+/// catalogo (GitLab: scope y rol).
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Permiso {
+    /// Nombre del alcance, igual en todos los idiomas.
+    Alcance(&'static str),
+    /// Descripcion que se traduce.
+    Descrito(&'static Texto),
+}
+
+impl From<&'static str> for Permiso {
+    fn from(a: &'static str) -> Self {
+        Self::Alcance(a)
+    }
+}
+
+impl From<&'static Texto> for Permiso {
+    fn from(t: &'static Texto) -> Self {
+        Self::Descrito(t)
+    }
+}
+
+impl Permiso {
+    /// Rellena el hueco `{permiso}` de `m`.
+    pub fn en(self, m: Mensaje) -> Mensaje {
+        match self {
+            Self::Alcance(a) => m.con("permiso", a),
+            Self::Descrito(t) => m.con_mensaje("permiso", t),
+        }
+    }
+}
+
 /// Traduce una respuesta que no es 2xx. `op` es la operacion, `rol` con que
 /// identidad corria y `permiso` el que necesita (para el 403 sin plan).
-pub fn traducir(p: Proveedor, op: &str, rol: Rol, permiso: &str, r: &Respuesta) -> Informe {
+pub fn traducir(p: Proveedor, op: &str, rol: Rol, permiso: Permiso, r: &Respuesta) -> Informe {
     let api = mensaje_api(r);
     let nombre = p.titulo();
+    // La cita solo lleva el nombre de la plataforma y el texto de la API:
+    // nada que traducir.
     let cita = if api.is_empty() {
         String::new()
     } else {
         format!(" ({nombre}: «{api}»)")
     };
     let objeto = match p {
-        Proveedor::GitHub => "el repo, el PR o el commit",
-        Proveedor::GitLab => "el proyecto, el MR o el commit",
+        Proveedor::GitHub => &t::OBJETO_GITHUB,
+        Proveedor::GitLab => &t::OBJETO_GITLAB,
     };
+    let con_cita = |texto: &'static Texto| Mensaje::nuevo(texto).con("cita", cita.clone());
     match r.estado {
         401 => Informe::nuevo(op, Resultado::Denegada)
             .con("motivo", "credenciales")
             .con("rol", rol.nombre())
-            .diagnostico(format!(
-                "el token de {} no vale o ha caducado: define o renueva {}{cita}",
-                rol.nombre(),
-                rol.variable()
-            )),
+            .diagnostico(
+                con_cita(&t::TOKEN_NO_VALE)
+                    .con("rol", rol.nombre())
+                    .con("variable", rol.variable()),
+            ),
         _ if es_limite(r) => {
             let s = espera(r);
             Informe::nuevo(op, Resultado::Agotada)
                 .con("motivo", "limite-peticiones")
                 .con("reintenta-en", s.clone())
-                .diagnostico(format!(
-                    "limite de peticiones de la API agotado: espera {s} s y relanza{cita}"
-                ))
+                .diagnostico(con_cita(&t::LIMITE_AGOTADO).con("segundos", s))
         }
         403 if p == Proveedor::GitHub && api.contains(AVISO_PLAN) => {
             Informe::nuevo(op, Resultado::Rechazada)
                 .con("motivo", "plan")
-                .diagnostico(format!(
-                    "el plan de GitHub no da esta funcion en un repo privado: haz el repo publico o usa GitHub Pro, Team o Enterprise{cita}"
-                ))
+                .diagnostico(con_cita(&t::PLAN_GITHUB))
         }
         403 => Informe::nuevo(op, Resultado::Denegada)
             .con("motivo", "permisos")
             .con("rol", rol.nombre())
-            .diagnostico(format!(
-                "el token de {} no tiene permiso: necesita {permiso}{cita}",
-                rol.nombre()
-            )),
+            .diagnostico(permiso.en(con_cita(&t::SIN_PERMISO).con("rol", rol.nombre()))),
         404 => Informe::nuevo(op, Resultado::Denegada)
             .con("motivo", "no-existe-o-sin-acceso")
             .con("rol", rol.nombre())
-            .diagnostico(format!(
-                "no existe o el token de {} no lo ve: revisa {objeto} y el alcance del token ({nombre} da 404 en un repo privado sin acceso){cita}",
-                rol.nombre()
-            )),
+            .diagnostico(
+                con_cita(&t::NO_EXISTE)
+                    .con("rol", rol.nombre())
+                    .con_mensaje("objeto", objeto)
+                    .con("plataforma", nombre),
+            ),
         // GitLab da 405 o 406 cuando el MR no se puede mergear.
         405 => no_mergeable(op, nombre, &cita),
         406 if p == Proveedor::GitLab => no_mergeable(op, nombre, &cita),
         409 => Informe::nuevo(op, Resultado::Rechazada)
             .con("motivo", "head-cambiado")
-            .diagnostico(format!(
-                "la rama ha cambiado desde que se evaluaron las puertas: vuelve a lanzar estado-puertas{cita}"
-            )),
+            .diagnostico(con_cita(&t::HEAD_CAMBIADO)),
         // El detalle del 422 de GitHub viene en `errors[].message`, no en
         // `message`.
         422 if r.cuerpo.contains("No commits between") => Informe::nuevo(op, Resultado::Rechazada)
             .con("motivo", "sin-commits")
-            .diagnostico(format!(
-                "la rama no tiene commits nuevos respecto a la base: no hay nada que revisar{cita}"
-            )),
+            .diagnostico(con_cita(&t::SIN_COMMITS)),
         400 if p == Proveedor::GitLab => validacion(op, nombre, &cita),
         422 => validacion(op, nombre, &cita),
         e => Informe::nuevo(op, Resultado::Error)
             .con("motivo", "http")
             .con("estado", e.to_string())
-            .diagnostico(format!(
-                "respuesta inesperada de {nombre} ({e}): reintenta mas tarde{cita}"
-            )),
+            .diagnostico(
+                con_cita(&t::RESPUESTA_INESPERADA)
+                    .con("plataforma", nombre)
+                    .con("estado", e.to_string()),
+            ),
     }
 }
 
 fn no_mergeable(op: &str, nombre: &str, cita: &str) -> Informe {
     Informe::nuevo(op, Resultado::Rechazada)
         .con("motivo", "no-mergeable")
-        .diagnostico(format!(
-            "{nombre} no deja mergear: faltan puertas o hay conflictos; lanza estado-puertas{cita}"
-        ))
+        .diagnostico(
+            Mensaje::nuevo(&t::NO_MERGEABLE)
+                .con("plataforma", nombre)
+                .con("cita", cita),
+        )
 }
 
 fn validacion(op: &str, nombre: &str, cita: &str) -> Informe {
     Informe::nuevo(op, Resultado::Error)
         .con("motivo", "validacion")
-        .diagnostico(format!("{nombre} rechazo los datos de la peticion{cita}"))
+        .diagnostico(
+            Mensaje::nuevo(&t::VALIDACION)
+                .con("plataforma", nombre)
+                .con("cita", cita),
+        )
 }
 
 /// Traduce un fallo sin respuesta.
@@ -174,43 +211,45 @@ pub fn fallo(op: &str, f: &FalloHttp) -> Informe {
     match f {
         FalloHttp::Rechazada(m) => Informe::nuevo(op, Resultado::Rechazada)
             .con("motivo", "destino")
-            .diagnostico(format!(
-                "llamada rechazada antes de salir a la red: {}",
-                sanear(m)
-            )),
-        FalloHttp::Sinred(m) => Informe::nuevo(op, Resultado::Sinred).diagnostico(format!(
-            "no se pudo hablar con la API: comprueba la red y reintenta ({})",
-            sanear(m)
-        )),
+            .diagnostico(Mensaje::nuevo(&t::LLAMADA_RECHAZADA).con_mensaje("motivo", *m)),
+        FalloHttp::Sinred(m) => Informe::nuevo(op, Resultado::Sinred)
+            .diagnostico(Mensaje::nuevo(&t::SIN_RED).con("detalle", sanear(m))),
         FalloHttp::Tls(m) => Informe::nuevo(op, Resultado::Error)
             .con("motivo", "tls")
-            .diagnostico(format!(
-                "fallo de TLS: si la red usa un proxy con su propia CA, instalala en el almacen de certificados del sistema ({})",
-                sanear(m)
-            )),
+            .diagnostico(Mensaje::nuevo(&t::TLS).con("detalle", sanear(m))),
         FalloHttp::RespuestaInvalida(m) => Informe::nuevo(op, Resultado::Error)
             .con("motivo", "respuesta-invalida")
-            .diagnostico(format!(
-                "respuesta de la API ilegible o demasiado grande ({})",
-                sanear(m)
+            .diagnostico(Mensaje::nuevo(&t::RESPUESTA_ILEGIBLE).con("detalle", sanear(m))),
+        FalloHttp::CuerpoGrande => Informe::nuevo(op, Resultado::Error)
+            .con("motivo", "respuesta-invalida")
+            .diagnostico(Mensaje::nuevo(&t::RESPUESTA_ILEGIBLE).con_mensaje(
+                "detalle",
+                Mensaje::nuevo(&t::DECLARA_MAS_DE).con("limite", LIMITE_CUERPO.to_string()),
             )),
         FalloHttp::Redireccion(e) => Informe::nuevo(op, Resultado::Error)
             .con("motivo", "redireccion")
-            .diagnostico(format!(
-                "la API respondio con una redireccion ({e}) y no se siguen: revisa la URL de la API o el nombre del repo (renombrado o movido)"
-            )),
+            .diagnostico(Mensaje::nuevo(&t::REDIRECCION).con("estado", e.to_string())),
         FalloHttp::Otro(m) => Informe::nuevo(op, Resultado::Error)
             .con("motivo", "cliente")
-            .diagnostico(format!("fallo del cliente HTTP ({})", sanear(m))),
+            .diagnostico(Mensaje::nuevo(&t::CLIENTE_HTTP).con("detalle", sanear(m))),
     }
 }
 
-/// Respuesta 2xx con un cuerpo que no tiene la forma esperada.
+/// Respuesta 2xx con un cuerpo que no tiene la forma esperada. `que` dice
+/// que falta, tal cual: el error del lector JSON o el nombre de un campo.
 pub fn respuesta_invalida(p: Proveedor, op: &str, que: &str) -> Informe {
+    forma_inesperada(op, p, |m| m.con("que", que))
+}
+
+/// Como [`respuesta_invalida`], con `que` descrito en el catalogo.
+pub fn respuesta_invalida_por(p: Proveedor, op: &str, que: &'static Texto) -> Informe {
+    forma_inesperada(op, p, |m| m.con_mensaje("que", que))
+}
+
+fn forma_inesperada(op: &str, p: Proveedor, que: impl FnOnce(Mensaje) -> Mensaje) -> Informe {
     Informe::nuevo(op, Resultado::Error)
         .con("motivo", "respuesta-invalida")
-        .diagnostico(format!(
-            "la respuesta de {} no tiene la forma esperada ({que})",
-            p.titulo()
+        .diagnostico(que(
+            Mensaje::nuevo(&t::FORMA_INESPERADA).con("plataforma", p.titulo())
         ))
 }
diff --git a/crates/tc2/src/plataforma/github.rs b/crates/tc2/src/plataforma/github.rs
index ef77bc3..e65ba70 100644
--- a/crates/tc2/src/plataforma/github.rs
+++ b/crates/tc2/src/plataforma/github.rs
@@ -18,6 +18,7 @@ use super::http::{Base, Metodo, codificar};
 use super::puertas::{self, EstadoCheck, Lecturas};
 use super::{Firma, ModeloPuertas, Paso, Personas, Plataforma, Proveedor, ReglasLeidas, Usuario};
 use crate::contrato::{Informe, Resultado, Valor, corto};
+use crate::i18n::catalogo::plataforma as t;
 
 const ABRIR_PR: &str = "abrir-pr";
 const MERGE_PR: &str = "merge-pr";
@@ -220,7 +221,7 @@ impl GitHub {
         if !valido {
             return Err(Informe::nuevo("plataforma", Resultado::Uso)
                 .con("motivo", "repo-invalido")
-                .diagnostico("el repo de GitHub tiene que ser dueno/repo"));
+                .diagnostico(&t::REPO_GITHUB));
         }
         Ok(Self {
             con: Conexion::nueva(base, Proveedor::GitHub, credenciales, personas),
@@ -267,14 +268,10 @@ impl GitHub {
 impl Plataforma for GitHub {
     fn abrir_pr(&self, rama: &str, base: &str, titulo: &str, cuerpo: &str) -> Informe {
         if !rama_valida(rama) || !rama_valida(base) {
-            return uso(ABRIR_PR, "rama-invalida", "nombre de rama no valido");
+            return uso(ABRIR_PR, "rama-invalida", &t::RAMA_NO_VALIDA);
         }
         if titulo.trim().is_empty() || titulo.chars().any(char::is_control) {
-            return uso(
-                ABRIR_PR,
-                "titulo-invalido",
-                "el titulo no puede estar vacio ni llevar controles",
-            );
+            return uso(ABRIR_PR, "titulo-invalido", &t::TITULO_NO_VALIDO);
         }
         match self.pr_abierto(rama, base) {
             Ok(Some(n)) => {
@@ -341,7 +338,7 @@ impl Plataforma for GitHub {
             return puertas::pr_cerrado(pr);
         }
         if !rama_valida(&p.base.nombre) || !sha_valido(&p.head.sha) {
-            return self.con.invalida(op, "rama base o sha del head");
+            return self.con.invalida_por(op, &t::RAMA_BASE_O_SHA);
         }
         let r = match self.con.pedir(
             op,
@@ -444,11 +441,7 @@ impl Plataforma for GitHub {
 
     fn merge_pr(&self, pr: u64, sha: &str) -> Informe {
         if !sha_valido(sha) {
-            return uso(
-                MERGE_PR,
-                "sha-invalido",
-                "el sha esperado tiene que ser completo (40 hex)",
-            );
+            return uso(MERGE_PR, "sha-invalido", &t::SHA_ESPERADO_COMPLETO);
         }
         if let Err(i) = self.exigir_persona(MERGE_PR) {
             return i;
@@ -471,7 +464,7 @@ impl Plataforma for GitHub {
             return Informe::nuevo(MERGE_PR, Resultado::Rechazada)
                 .con("pr", pr.to_string())
                 .con("motivo", "pr-cerrado")
-                .diagnostico("el PR esta cerrado sin mergear");
+                .diagnostico(&t::PR_CERRADO);
         }
         if !p.head.sha.eq_ignore_ascii_case(sha) {
             return head_cambiado(pr);
@@ -509,11 +502,7 @@ impl Plataforma for GitHub {
 
     fn leer_puertas(&self, rol: Rol, rama: &str) -> Paso<ReglasLeidas> {
         if !rama_valida(rama) {
-            return Err(uso(
-                LEER_PUERTAS,
-                "rama-invalida",
-                "nombre de rama no valido",
-            ));
+            return Err(uso(LEER_PUERTAS, "rama-invalida", &t::RAMA_NO_VALIDA));
         }
         if rol == Rol::Revisor {
             return Err(rol_no_valido(LEER_PUERTAS, rol));
@@ -563,7 +552,7 @@ impl Plataforma for GitHub {
             Informe::nuevo(LEER_PERMISOS_CI, Resultado::Rechazada)
                 .con("motivo", "ci-puede-aprobar")
                 .con("permisos", permisos)
-                .diagnostico("el token de Actions puede aprobar PR: desactiva «Allow GitHub Actions to create and approve pull requests» en Settings > Actions > General (G16)")
+                .diagnostico(&t::CI_PUEDE_APROBAR)
         } else {
             Informe::nuevo(LEER_PERMISOS_CI, Resultado::Ok)
                 .con("ci-aprueba", "false")
@@ -573,11 +562,7 @@ impl Plataforma for GitHub {
 
     fn firma_commit(&self, sha: &str) -> Paso<Firma> {
         if !sha_valido(sha) {
-            return Err(uso(
-                FIRMA_COMMIT,
-                "sha-invalido",
-                "el sha tiene que ser completo (40 hex)",
-            ));
+            return Err(uso(FIRMA_COMMIT, "sha-invalido", &t::SHA_COMPLETO));
         }
         let c: CommitApi = self.con.leer(
             FIRMA_COMMIT,
@@ -596,7 +581,12 @@ impl Plataforma for GitHub {
     fn identidad(&self, rol: Rol) -> Paso<Usuario> {
         self.con.identidad(rol, || {
             self.con
-                .leer::<UsuarioApi>(IDENTIDAD, rol, "(ninguno)", self.ep(&Endpoint::Usuario))
+                .leer::<UsuarioApi>(
+                    IDENTIDAD,
+                    rol,
+                    &t::PERMISO_NINGUNO,
+                    self.ep(&Endpoint::Usuario),
+                )
                 .map(Usuario::from)
         })
     }
diff --git a/crates/tc2/src/plataforma/gitlab.rs b/crates/tc2/src/plataforma/gitlab.rs
index 661d0a8..91f7587 100644
--- a/crates/tc2/src/plataforma/gitlab.rs
+++ b/crates/tc2/src/plataforma/gitlab.rs
@@ -21,6 +21,8 @@ use super::{
     Clave, Firma, ModeloPuertas, Paso, Personas, Plataforma, Proveedor, ReglasLeidas, Usuario,
 };
 use crate::contrato::{Informe, Resultado, Valor, corto};
+use crate::i18n::catalogo::plataforma as t;
+use crate::i18n::{Mensaje, Texto};
 
 const ABRIR_PR: &str = "abrir-pr";
 const MERGE_PR: &str = "merge-pr";
@@ -30,13 +32,13 @@ const FIRMA_COMMIT: &str = "firma-commit";
 const IDENTIDAD: &str = "identidad";
 
 /// Permiso de lectura que necesita un token de GitLab.
-const PERMISO_LECTURA: &str = "el scope api (o read_api) y el rol Developer en el proyecto";
+static PERMISO_LECTURA: &Texto = &t::PERMISO_LECTURA_GITLAB;
 
 /// Permiso para abrir un MR.
-const PERMISO_ESCRITURA: &str = "el scope api y el rol Developer en el proyecto";
+static PERMISO_ESCRITURA: &Texto = &t::PERMISO_ESCRITURA_GITLAB;
 
 /// Permiso para mergear en una rama protegida.
-const PERMISO_MERGE: &str = "el scope api y el rol Maintainer en el proyecto";
+static PERMISO_MERGE: &Texto = &t::PERMISO_MERGE_GITLAB;
 
 /// Garantias que en GitLab solo da Premium o Ultimate (tabla de F1-13): sin
 /// reglas de aprobacion (403 en Free), la politica no puede exigirlas
@@ -258,9 +260,7 @@ impl GitLab {
         let Some(codificado) = codificar_proyecto(proyecto) else {
             return Err(Informe::nuevo("plataforma", Resultado::Uso)
                 .con("motivo", "proyecto-invalido")
-                .diagnostico(
-                    "el proyecto de GitLab tiene que ser su id numerico o grupo/proyecto",
-                ));
+                .diagnostico(&t::PROYECTO_GITLAB));
         };
         Ok(Self {
             con: Conexion::nueva(base, Proveedor::GitLab, credenciales, personas),
@@ -341,14 +341,10 @@ fn estado_check(s: &str) -> EstadoCheck {
 impl Plataforma for GitLab {
     fn abrir_pr(&self, rama: &str, base: &str, titulo: &str, cuerpo: &str) -> Informe {
         if !rama_valida(rama) || !rama_valida(base) {
-            return uso(ABRIR_PR, "rama-invalida", "nombre de rama no valido");
+            return uso(ABRIR_PR, "rama-invalida", &t::RAMA_NO_VALIDA);
         }
         if titulo.trim().is_empty() || titulo.chars().any(char::is_control) {
-            return uso(
-                ABRIR_PR,
-                "titulo-invalido",
-                "el titulo no puede estar vacio ni llevar controles",
-            );
+            return uso(ABRIR_PR, "titulo-invalido", &t::TITULO_NO_VALIDO);
         }
         match self.mr_abierto(rama, base) {
             Ok(Some(n)) => {
@@ -364,9 +360,7 @@ impl Plataforma for GitLab {
             Ok(false) => {
                 return Informe::nuevo(ABRIR_PR, Resultado::Rechazada)
                     .con("motivo", "sin-commits")
-                    .diagnostico(
-                        "la rama no tiene commits nuevos respecto a la base: no hay nada que revisar",
-                    );
+                    .diagnostico(Mensaje::nuevo(&t::SIN_COMMITS).con("cita", ""));
             }
             Err(i) => return i,
         }
@@ -392,10 +386,9 @@ impl Plataforma for GitLab {
                 }
                 Ok(None) => Informe::nuevo(ABRIR_PR, Resultado::Conflicto)
                     .con("motivo", "mr-existente")
-                    .diagnostico(format!(
-                        "GitLab dice que ya hay un MR para la rama pero no se encuentra abierto: reintenta (GitLab: «{}»)",
-                        errores::mensaje_api(&r)
-                    )),
+                    .diagnostico(
+                        Mensaje::nuevo(&t::MR_EXISTENTE).con("api", errores::mensaje_api(&r)),
+                    ),
                 Err(i) => i,
             };
         }
@@ -436,7 +429,7 @@ impl Plataforma for GitLab {
             return puertas::pr_cerrado(pr);
         }
         if !rama_valida(&m.target_branch) || !sha_valido(&m.sha) {
-            return self.con.invalida(op, "rama destino o sha del MR");
+            return self.con.invalida_por(op, &t::RAMA_DESTINO_O_SHA);
         }
         // Las reglas de aprobacion son de Premium. Observado en el E2E: en
         // Free el endpoint responde 200 con la lista vacia (no el 403 que
@@ -468,11 +461,7 @@ impl Plataforma for GitLab {
             return if r.estado == 403 {
                 puertas::rechazada_por_plan(pr, g)
             } else {
-                puertas::rechazada_por_config(
-                    pr,
-                    g,
-                    "GitLab no impone aprobaciones en este proyecto: crea una regla de aprobacion con al menos una aprobacion (requiere Premium o Ultimate) o quita la garantia de la politica",
-                )
+                puertas::rechazada_por_config(pr, g, &t::ARREGLO_APROBACIONES_GITLAB)
             };
         }
         // G01: la rama destino solo la mergean Maintainers y no admite
@@ -502,11 +491,7 @@ impl Plataforma for GitLab {
                 })
         });
         if !g01 {
-            return puertas::rechazada_por_config(
-                pr,
-                "G01",
-                "la rama destino no esta protegida con merge solo para Maintainers y sin force-push: protegela (taskctl setup lo propone)",
-            );
+            return puertas::rechazada_por_config(pr, "G01", &t::ARREGLO_RAMA_PROTEGIDA_GITLAB);
         }
         let (_, ruta) = self.ep(&Endpoint::Estados(&m.sha));
         let mut estados: Vec<EstadoApi> =
@@ -572,11 +557,7 @@ impl Plataforma for GitLab {
 
     fn merge_pr(&self, pr: u64, sha: &str) -> Informe {
         if !sha_valido(sha) {
-            return uso(
-                MERGE_PR,
-                "sha-invalido",
-                "el sha esperado tiene que ser completo (40 hex)",
-            );
+            return uso(MERGE_PR, "sha-invalido", &t::SHA_ESPERADO_COMPLETO);
         }
         if let Err(i) = self.exigir_persona(MERGE_PR) {
             return i;
@@ -599,7 +580,7 @@ impl Plataforma for GitLab {
             return Informe::nuevo(MERGE_PR, Resultado::Rechazada)
                 .con("pr", pr.to_string())
                 .con("motivo", "pr-cerrado")
-                .diagnostico("el MR esta cerrado sin mergear");
+                .diagnostico(&t::MR_CERRADO);
         }
         if !m
             .sha
@@ -628,10 +609,11 @@ impl Plataforma for GitLab {
                 .con("pr", pr.to_string())
                 .con("motivo", "permisos")
                 .con("rol", Rol::Persona.nombre())
-                .diagnostico(format!(
-                    "GitLab no deja mergear a esta cuenta: necesita {PERMISO_MERGE} (GitLab: «{}»)",
-                    errores::mensaje_api(&r)
-                ));
+                .diagnostico(
+                    Mensaje::nuevo(&t::MERGE_NO_PERMITIDO_GITLAB)
+                        .con_mensaje("permiso", PERMISO_MERGE)
+                        .con("api", errores::mensaje_api(&r)),
+                );
         }
         if !r.exito() {
             let mut i = self.con.traducir(MERGE_PR, Rol::Persona, PERMISO_MERGE, &r);
@@ -657,11 +639,7 @@ impl Plataforma for GitLab {
 
     fn leer_puertas(&self, rol: Rol, rama: &str) -> Paso<ReglasLeidas> {
         if !rama_valida(rama) {
-            return Err(uso(
-                LEER_PUERTAS,
-                "rama-invalida",
-                "nombre de rama no valido",
-            ));
+            return Err(uso(LEER_PUERTAS, "rama-invalida", &t::RAMA_NO_VALIDA));
         }
         if rol == Rol::Revisor {
             return Err(rol_no_valido(LEER_PUERTAS, rol));
@@ -728,11 +706,7 @@ impl Plataforma for GitLab {
 
     fn firma_commit(&self, sha: &str) -> Paso<Firma> {
         if !sha_valido(sha) {
-            return Err(uso(
-                FIRMA_COMMIT,
-                "sha-invalido",
-                "el sha tiene que ser completo (40 hex)",
-            ));
+            return Err(uso(FIRMA_COMMIT, "sha-invalido", &t::SHA_COMPLETO));
         }
         let r = self.con.pedir(
             FIRMA_COMMIT,
diff --git a/crates/tc2/src/plataforma/http.rs b/crates/tc2/src/plataforma/http.rs
index 3f9f1d9..fb5dbe2 100644
--- a/crates/tc2/src/plataforma/http.rs
+++ b/crates/tc2/src/plataforma/http.rs
@@ -22,6 +22,8 @@ use std::time::Duration;
 
 use super::Proveedor;
 use super::credenciales::Secreto;
+use crate::i18n::Texto;
+use crate::i18n::catalogo::plataforma as t;
 
 /// Tamano maximo del cuerpo de una respuesta.
 pub const LIMITE_CUERPO: u64 = 1024 * 1024;
@@ -70,7 +72,7 @@ impl Base {
     }
 
     /// Valida una base. `Err` con el motivo si no vale.
-    pub fn nueva(url: &str) -> Result<Self, String> {
+    pub fn nueva(url: &str) -> Result<Self, &'static Texto> {
         let url = url.trim_end_matches('/');
         let resto = if let Some(r) = url.strip_prefix("https://") {
             r
@@ -80,14 +82,11 @@ impl Base {
                 && !ruta.contains(['?', '#'])
                 && (ruta.is_empty() || validar_ruta(&format!("/{ruta}")).is_ok());
             if !valida {
-                return Err(
-                    "http solo vale para 127.0.0.1 con un puerto y, si acaso, un prefijo valido"
-                        .into(),
-                );
+                return Err(&t::HTTP_SOLO_LOCAL);
             }
             return Ok(Self(url.to_owned()));
         } else {
-            return Err("la URL de la API tiene que ser https".into());
+            return Err(&t::URL_HTTPS);
         };
         let (autoridad, ruta) = resto.split_once('/').unwrap_or((resto, ""));
         let (host, puerto) = match autoridad.split_once(':') {
@@ -101,13 +100,13 @@ impl Base {
             && !host.starts_with(['.', '-'])
             && puerto.is_none_or(puerto_valido);
         if !host_valido {
-            return Err("host de la API no valido (sin usuario ni contrasena)".into());
+            return Err(&t::HOST_NO_VALIDO);
         }
         if ruta.contains('?') || ruta.contains('#') {
-            return Err("el prefijo de la API no lleva consulta ni fragmento".into());
+            return Err(&t::PREFIJO_CON_CONSULTA);
         }
         if !ruta.is_empty() && validar_ruta(&format!("/{ruta}")).is_err() {
-            return Err("prefijo de la API no valido".into());
+            return Err(&t::PREFIJO_NO_VALIDO);
         }
         Ok(Self(url.to_owned()))
     }
@@ -165,14 +164,16 @@ impl Respuesta {
 /// Por que no hubo respuesta HTTP.
 #[derive(Debug, Clone, PartialEq, Eq)]
 pub enum FalloHttp {
-    /// El envoltorio la rechazo antes de la red.
-    Rechazada(String),
+    /// El envoltorio la rechazo antes de la red, con el motivo.
+    Rechazada(&'static Texto),
     /// No se pudo conectar (DNS, conexion, tiempo agotado).
     Sinred(String),
     /// Fallo de TLS (certificado, almacen del sistema).
     Tls(String),
-    /// Respuesta mayor que [`LIMITE_CUERPO`] o ilegible.
+    /// Respuesta mayor que [`LIMITE_CUERPO`] al leerla, o ilegible.
     RespuestaInvalida(String),
+    /// La respuesta declara un tamano mayor que [`LIMITE_CUERPO`].
+    CuerpoGrande,
     /// La API respondio con una redireccion (no se siguen).
     Redireccion(u16),
     /// Otro fallo del cliente.
@@ -243,7 +244,7 @@ impl Cliente {
         url.strip_prefix(self.base.texto())
             .filter(|r| r.starts_with('/'))
             .map(str::to_owned)
-            .ok_or_else(|| FalloHttp::Rechazada("paginacion hacia otro destino".into()))
+            .ok_or(FalloHttp::Rechazada(&t::PAGINACION_OTRO_DESTINO))
     }
 
     /// Hace la llamada. `ruta` incluye la consulta (`?a=b`).
@@ -292,9 +293,7 @@ impl Cliente {
         // sistema puede notificar como desconexion (y se veria `sinred`).
         let declarado = cabecera("content-length").and_then(|v| v.trim().parse::<u64>().ok());
         if declarado.is_some_and(|n| n > LIMITE_CUERPO) {
-            return Err(FalloHttp::RespuestaInvalida(format!(
-                "la respuesta declara mas de {LIMITE_CUERPO} bytes"
-            )));
+            return Err(FalloHttp::CuerpoGrande);
         }
         // Sin tamano declarado (chunked), el limite corta al leer.
         let cuerpo = r
@@ -359,21 +358,21 @@ fn tipo_error(e: &ureq::Error) -> String {
 }
 
 /// Comprueba la ruta (con su consulta) antes de la red, sin admitir `%2F`.
-pub fn validar_ruta(ruta: &str) -> Result<(), String> {
+pub fn validar_ruta(ruta: &str) -> Result<(), &'static Texto> {
     validar_ruta_con(ruta, false)
 }
 
 /// Comprueba la ruta. Con `admite_barra_codificada` (GitLab) acepta la
 /// secuencia exacta `%2F` en el camino y comprueba los segmentos con ella ya
 /// decodificada: `a%2F..` es un `..` y `%2f` o `%252F` siguen rechazados.
-pub fn validar_ruta_con(ruta: &str, admite_barra_codificada: bool) -> Result<(), String> {
+pub fn validar_ruta_con(ruta: &str, admite_barra_codificada: bool) -> Result<(), &'static Texto> {
     let (camino, consulta) = ruta.split_once('?').unwrap_or((ruta, ""));
     if !camino.starts_with('/') || camino.starts_with("//") {
-        return Err("la ruta tiene que ser relativa a la API".into());
+        return Err(&t::RUTA_NO_RELATIVA);
     }
     let minus = ruta.to_ascii_lowercase();
     if ruta.contains('\\') || minus.contains("%2e") || minus.contains("%25") {
-        return Err("ruta con barra invertida o puntos codificados".into());
+        return Err(&t::RUTA_CODIFICADA);
     }
     let decodificado;
     let camino = if admite_barra_codificada {
@@ -386,17 +385,17 @@ pub fn validar_ruta_con(ruta: &str, admite_barra_codificada: bool) -> Result<(),
         .split('/')
         .any(|s| s.is_empty() || s == "." || s == "..")
     {
-        return Err("ruta con segmentos vacios, . o ..".into());
+        return Err(&t::RUTA_SEGMENTOS);
     }
     let no_reservado = |b: u8| b.is_ascii_alphanumeric() || b"-._~".contains(&b);
     if !camino.bytes().all(|b| no_reservado(b) || b == b'/') {
-        return Err("caracter no permitido en la ruta".into());
+        return Err(&t::CARACTER_EN_RUTA);
     }
     if !consulta
         .bytes()
         .all(|b| no_reservado(b) || b"=&:%,".contains(&b))
     {
-        return Err("caracter no permitido en la consulta".into());
+        return Err(&t::CARACTER_EN_CONSULTA);
     }
     Ok(())
 }
diff --git a/crates/tc2/src/plataforma/puertas.rs b/crates/tc2/src/plataforma/puertas.rs
index dc759c2..42d65bf 100644
--- a/crates/tc2/src/plataforma/puertas.rs
+++ b/crates/tc2/src/plataforma/puertas.rs
@@ -3,6 +3,8 @@
 //! GitLab, para que las dos den la misma linea byte a byte.
 
 use crate::contrato::{Informe, Resultado};
+use crate::i18n::catalogo::plataforma as t;
+use crate::i18n::{Mensaje, Texto};
 
 /// Nombre de la operacion.
 pub const ESTADO_PUERTAS: &str = "estado-puertas";
@@ -51,7 +53,7 @@ pub fn evaluar(l: &Lecturas) -> Informe {
             .con("pr", pr)
             .con("motivo", "check-rojo")
             .con_lista("checks", rojo)
-            .diagnostico("hay checks obligatorios en rojo: corrige y vuelve a empujar");
+            .diagnostico(&t::CHECKS_EN_ROJO);
     }
     let mut falta: Vec<String> = l
         .checks
@@ -81,10 +83,7 @@ pub fn evaluar(l: &Lecturas) -> Informe {
     if falta.is_empty() {
         i
     } else {
-        let diag = format!(
-            "faltan puertas: {}; espera o atiende lo que falta (al-dia o sin-conflictos: lanza actualizar-rama)",
-            falta.join(", ")
-        );
+        let diag = Mensaje::nuevo(&t::FALTAN_PUERTAS).con("falta", falta.join(", "));
         i.con_lista("falta", falta).diagnostico(diag)
     }
 }
@@ -101,7 +100,7 @@ pub fn pr_cerrado(pr: u64) -> Informe {
     Informe::nuevo(ESTADO_PUERTAS, Resultado::Rechazada)
         .con("pr", pr.to_string())
         .con("motivo", "pr-cerrado")
-        .diagnostico("el PR esta cerrado sin mergear: reabrelo o abre otro")
+        .diagnostico(&t::PR_CERRADO_REABRE)
 }
 
 /// La politica exige impuesta una garantia que el plan no da.
@@ -110,20 +109,22 @@ pub fn rechazada_por_plan(pr: u64, garantia: &str) -> Informe {
         .con("pr", pr.to_string())
         .con("motivo", "plan")
         .con("garantia", garantia)
-        .diagnostico(format!(
-            "la politica del proyecto exige {garantia} impuesta y el plan de la plataforma no la da: cambia de plan o quita {garantia} de la politica"
-        ))
+        .diagnostico(Mensaje::nuevo(&t::RECHAZADA_POR_PLAN).con("garantia", garantia))
 }
 
 /// La plataforma no tiene configurada una garantia que se lee de su
 /// configuracion (GitLab: G01, rama destino protegida; G03 y demas de
 /// Premium, una regla de aprobacion). `arreglo` dice que hacer.
-pub fn rechazada_por_config(pr: u64, garantia: &str, arreglo: &str) -> Informe {
+pub fn rechazada_por_config(pr: u64, garantia: &str, arreglo: &'static Texto) -> Informe {
     Informe::nuevo(ESTADO_PUERTAS, Resultado::Rechazada)
         .con("pr", pr.to_string())
         .con("motivo", "config")
         .con("garantia", garantia)
-        .diagnostico(format!("{garantia} no se cumple: {arreglo}"))
+        .diagnostico(
+            Mensaje::nuevo(&t::NO_SE_CUMPLE)
+                .con("garantia", garantia)
+                .con_mensaje("arreglo", arreglo),
+        )
 }
 
 #[cfg(test)]
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-024/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-024/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-024/tarea.md                                    | 7 ++++++-
 3 files changed, 6 insertions(+), 1 deletion(-)
````

## Tests del diff (3 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/cli.rs" ":(literal)crates/tc2/tests/comun/ops.rs" ":(literal)crates/tc2/tests/config.rs"
````

````
crates/tc2/tests/cli.rs       | 42 +++++++++++++++++++++++++++++++++++++++++-
 crates/tc2/tests/comun/ops.rs |  8 +++++---
 crates/tc2/tests/config.rs    | 16 +++++++++++++---
 3 files changed, 59 insertions(+), 7 deletions(-)
````
