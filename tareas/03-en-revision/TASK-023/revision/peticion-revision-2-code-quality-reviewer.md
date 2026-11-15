# Peticion de revision — TASK-023 (ronda 2)

- Tarea: TASK-023 — F2-7 Configuracion minima con esquema validado y valores por defecto
- Rama revisada: feature/task-023-f2-7-configuracion-minima-con-esquema-va
- Rama base: develop
- Commit revisado (HEAD): 964898f7d7e9298ce912ae2a54c2b8b39e88f998
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-023 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 25e7d2ce796319aac3b8ab5b4d26518c6c197f78 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (12 hallazgo(s)))

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
informe de esta ronda (informe-revision-2-code-quality-reviewer.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-023/tarea.md
- tareas/03-en-revision/TASK-023/planificacion/
- crates/tc2/esquemas/config.schema.json
- crates/tc2/src/config/claves.rs
- crates/tc2/src/config/mod.rs
- crates/tc2/src/config/validar.rs
- crates/tc2/tests/config.rs

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 4 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/config/validar.rs:276 | informe-revision-1-code-quality-reviewer.md |

## Commits a revisar (git log 25e7d2ce796319aac3b8ab5b4d26518c6c197f78..HEAD)

````
964898f chore(TASK-023): autorrevision ronda 2
8671970 fix(TASK-023): hallazgos de la revision ronda 1
05eb4d3 chore(TASK-023): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 25e7d2ce796319aac3b8ab5b4d26518c6c197f78..HEAD)

````diff
diff --git a/crates/tc2/esquemas/config.schema.json b/crates/tc2/esquemas/config.schema.json
index b35ff50..4ec41f7 100644
--- a/crates/tc2/esquemas/config.schema.json
+++ b/crates/tc2/esquemas/config.schema.json
@@ -7,7 +7,7 @@
       "default": [],
       "description": "ficheros que suma la huella de confianza [F1-11 decision 1; F1-8 decision 5]",
       "items": {
-        "description": "glob relativo: sin negacion (!), rutas absolutas ni .., y nunca todo el arbol (**, *)",
+        "description": "glob relativo: sin negacion (!), rutas absolutas ni .., y nunca todo el arbol (ningun segmento solo de * y .)",
         "minLength": 1,
         "type": "string"
       },
@@ -27,7 +27,7 @@
           "escribe": {
             "description": "lo unico que un genera puede escribir; obligatoria en genera, prohibida en comprueba",
             "items": {
-              "description": "ruta normalizada bajo tareas/, sin \\, :, comodines, caracteres de control, nombres reservados de Windows ni .git; nunca tareas/ entera, tareas/ROADMAP.md ni tareas/TASK-NNN/ (sin distinguir mayusculas)",
+              "description": "ruta normalizada bajo tareas/, sin \\, :, comodines, caracteres de control, nombres reservados de Windows, .git, ~ ni punto o espacio al final de un segmento; nunca tareas/ entera, tareas/ROADMAP.md ni tareas/TASK-NNN/ (sin distinguir mayusculas)",
               "minLength": 1,
               "type": "string"
             },
@@ -741,7 +741,7 @@
               "type": "string"
             },
             "cron": {
-              "description": "cuando corre. expresion cron de 5 campos",
+              "description": "cuando corre. expresion cron de 5 campos con sus rangos (minuto, hora, dia, mes, dia de la semana)",
               "minLength": 1,
               "type": "string"
             },
diff --git a/crates/tc2/src/config/claves.rs b/crates/tc2/src/config/claves.rs
index 1cd2236..039751f 100644
--- a/crates/tc2/src/config/claves.rs
+++ b/crates/tc2/src/config/claves.rs
@@ -100,12 +100,14 @@ impl Regla {
             ),
             Self::Glob => Some("glob relativo: sin negacion (!), rutas absolutas ni .."),
             Self::GlobAcotado => Some(
-                "glob relativo: sin negacion (!), rutas absolutas ni .., y nunca todo el arbol (**, *)",
+                "glob relativo: sin negacion (!), rutas absolutas ni .., y nunca todo el arbol (ningun segmento solo de * y .)",
             ),
             Self::RutaTareas => Some(
-                "ruta normalizada bajo tareas/, sin \\, :, comodines, caracteres de control, nombres reservados de Windows ni .git; nunca tareas/ entera, tareas/ROADMAP.md ni tareas/TASK-NNN/ (sin distinguir mayusculas)",
+                "ruta normalizada bajo tareas/, sin \\, :, comodines, caracteres de control, nombres reservados de Windows, .git, ~ ni punto o espacio al final de un segmento; nunca tareas/ entera, tareas/ROADMAP.md ni tareas/TASK-NNN/ (sin distinguir mayusculas)",
+            ),
+            Self::Cron => Some(
+                "expresion cron de 5 campos con sus rangos (minuto, hora, dia, mes, dia de la semana)",
             ),
-            Self::Cron => Some("expresion cron de 5 campos"),
             Self::Url => Some("URL https de la API (o http://127.0.0.1:<puerto> en pruebas)"),
         }
     }
@@ -633,6 +635,7 @@ mod tests {
     #[test]
     fn claves_de_primer_nivel_no_pasan_de_12_y_cada_una_cita_su_adr() {
         let t = tabla();
+        assert_eq!(TOPE_CLAVES, 12, "el tope lo fija el plan (F2-7)");
         assert!(t.len() <= TOPE_CLAVES, "{} claves", t.len());
         for c in &t {
             assert!(!c.origen.is_empty(), "{} sin origen", c.nombre);
diff --git a/crates/tc2/src/config/mod.rs b/crates/tc2/src/config/mod.rs
index e2a78ab..86189a6 100644
--- a/crates/tc2/src/config/mod.rs
+++ b/crates/tc2/src/config/mod.rs
@@ -295,6 +295,8 @@ pub enum Motivo {
     SinConfig,
     /// Falta `---` o `formato: 2` (un proyecto de la 0.16).
     Formato,
+    /// `formato` mayor que 2: el proyecto es de un `tc2` mas nuevo.
+    FormatoNuevo,
     /// YAML mal formado, clave duplicada, ancla o alias.
     Yaml,
     /// No cumple el esquema.
@@ -307,6 +309,7 @@ impl Motivo {
         match self {
             Self::SinConfig => "sin-config",
             Self::Formato => "formato",
+            Self::FormatoNuevo => "formato-nuevo",
             Self::Yaml => "yaml",
             Self::ConfigInvalida => "config-invalida",
         }
@@ -377,12 +380,13 @@ impl Invalida {
         let arreglo = match self.motivo {
             Motivo::SinConfig => "crea la configuracion con tc2 init".to_owned(),
             Motivo::Formato => "es un proyecto de la 0.16: migralo con tc2 actualizar".to_owned(),
+            Motivo::FormatoNuevo => "el proyecto es de un tc2 mas nuevo: actualiza tc2".to_owned(),
             Motivo::Yaml => format!("corrige el YAML de {RUTA}"),
             Motivo::ConfigInvalida
                 if self
                     .errores
                     .iter()
-                    .any(|e| e.mensaje.contains("tc2 actualizar")) =>
+                    .any(|e| CLAVES_016.contains(&e.ruta.as_str())) =>
             {
                 "quedan claves de la 0.16: tc2 actualizar las traduce".to_owned()
             }
@@ -424,6 +428,13 @@ pub fn leer(texto: &str) -> Result<Config, Invalida> {
     };
     match formato {
         Some(f) if f.as_u64() == Some(2) => {}
+        Some(f) if f.as_u64().is_some_and(|n| n > 2) => {
+            return Err(Invalida::una(
+                Motivo::FormatoNuevo,
+                "formato",
+                format!("formato {f}: este tc2 solo lee el formato 2"),
+            ));
+        }
         Some(f) => {
             return Err(Invalida::una(
                 Motivo::Formato,
diff --git a/crates/tc2/src/config/validar.rs b/crates/tc2/src/config/validar.rs
index 5045663..1c0f261 100644
--- a/crates/tc2/src/config/validar.rs
+++ b/crates/tc2/src/config/validar.rs
@@ -59,6 +59,14 @@ pub(crate) fn validar(nodo: &Nodo, valor: &Value, ruta: &str, errores: &mut Vec<
             {
                 err(format!("el primer elemento tiene que ser «{p}»"));
             }
+            let textos: Vec<&str> = l.iter().filter_map(Value::as_str).collect();
+            if let Some(d) = textos
+                .iter()
+                .enumerate()
+                .find_map(|(i, t)| textos.get(..i).is_some_and(|a| a.contains(t)).then_some(t))
+            {
+                err(format!("«{d}» repetido"));
+            }
             for (i, x) in l.iter().enumerate() {
                 validar(de, x, &format!("{ruta}[{i}]"), errores);
             }
@@ -218,6 +226,8 @@ pub(crate) fn comprobar(regla: Regla, t: &str) -> Option<String> {
         Regla::Orden => {
             if t.trim().is_empty() {
                 Some("orden vacia".to_owned())
+            } else if t.contains(['\n', '\r']) {
+                Some("la orden tiene saltos de linea: escribela en una sola linea".to_owned())
             } else if control {
                 Some("la orden tiene un caracter de control (¿un \\v de una ruta de Windows entre comillas dobles? escribela entre comillas simples o con /)".to_owned())
             } else {
@@ -227,9 +237,7 @@ pub(crate) fn comprobar(regla: Regla, t: &str) -> Option<String> {
         Regla::Glob | Regla::GlobAcotado => {
             if let Some(m) = glob_malo(t) {
                 Some(format!("glob no admitido «{t}»: {m}"))
-            } else if regla == Regla::GlobAcotado
-                && matches!(t, "*" | "**" | "**/*" | "**/**" | "./**" | "./*")
-            {
+            } else if regla == Regla::GlobAcotado && todo_el_arbol(t) {
                 Some(format!("glob no admitido «{t}»: nunca todo el arbol"))
             } else {
                 None
@@ -238,23 +246,58 @@ pub(crate) fn comprobar(regla: Regla, t: &str) -> Option<String> {
         Regla::RutaTareas => ruta_tareas(t)
             .err()
             .map(|m| format!("ruta no admitida «{t}»: {m}")),
-        Regla::Cron => {
-            let campos: Vec<&str> = t.split_whitespace().collect();
-            let ok = campos.len() == 5
-                && campos.iter().all(|c| {
-                    c.bytes()
-                        .all(|b| b.is_ascii_digit() || b"*,/-".contains(&b))
-                });
-            (!ok).then(|| {
-                format!("cron no admitido «{t}»: se esperan 5 campos con cifras, *, ',', / o -")
-            })
-        }
+        Regla::Cron => (!cron_valido(t)).then(|| {
+            format!(
+                "cron no admitido «{t}»: se esperan 5 campos (minuto 0-59, hora 0-23, dia 1-31, mes 1-12, dia de la semana 0-7) con *, a, a-b, ',' y /n"
+            )
+        }),
         Regla::Url => Base::nueva(t)
             .err()
             .map(|m| format!("URL no admitida: {m}")),
     }
 }
 
+/// Si un glob de `confianza_incluye` abarca todo el arbol: quitado `./`,
+/// ningun segmento tiene mas que `*` y `.` (`**`, `*.*`, `**/`, `.`...).
+fn todo_el_arbol(g: &str) -> bool {
+    let g = g.trim_start_matches("./");
+    g.split('/')
+        .filter(|p| !p.is_empty())
+        .all(|p| p.chars().all(|c| c == '*' || c == '.'))
+}
+
+/// Expresion cron de 5 campos con sus rangos (`*`, `a`, `a-b`, listas con
+/// `,` y paso `/n` con n >= 1).
+fn cron_valido(t: &str) -> bool {
+    const RANGOS: [(u32, u32); 5] = [(0, 59), (0, 23), (1, 31), (1, 12), (0, 7)];
+    let campos: Vec<&str> = t.split_whitespace().collect();
+    campos.len() == RANGOS.len()
+        && campos.iter().zip(RANGOS).all(|(c, (min, max))| {
+            c.split(',').all(|item| {
+                let (base, paso) = match item.split_once('/') {
+                    Some((b, p)) => (b, Some(p)),
+                    None => (item, None),
+                };
+                let en_rango = |x: &str| {
+                    !x.is_empty()
+                        && x.bytes().all(|b| b.is_ascii_digit())
+                        && x.parse::<u32>().is_ok_and(|n| n >= min && n <= max)
+                };
+                let base_ok = base == "*"
+                    || en_rango(base)
+                    || base.split_once('-').is_some_and(|(a, b)| {
+                        en_rango(a) && en_rango(b) && a.parse::<u32>().ok() <= b.parse::<u32>().ok()
+                    });
+                let paso_ok = paso.is_none_or(|p| {
+                    !p.is_empty()
+                        && p.bytes().all(|b| b.is_ascii_digit())
+                        && p.parse::<u32>().is_ok_and(|n| n >= 1)
+                });
+                base_ok && paso_ok
+            })
+        })
+}
+
 fn glob_malo(g: &str) -> Option<&'static str> {
     if g.is_empty() {
         Some("vacio")
@@ -295,6 +338,11 @@ pub(crate) fn ruta_tareas(t: &str) -> Result<String, &'static str> {
                     return Err("sale de la raiz");
                 }
             }
+            p if p.ends_with(['.', ' ']) || p.contains('~') => {
+                return Err(
+                    "sin punto ni espacio al final de un segmento ni ~ (Windows los ignora o los acorta)",
+                );
+            }
             p => partes.push(p),
         }
     }
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 25e7d2ce796319aac3b8ab5b4d26518c6c197f78..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-023/planificacion/brainstorm/peticion-plan-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-023/planificacion/plan-final.md                 |    0
 tareas/03-en-revision/TASK-023/revision/informe-revision-1-code-quality-reviewer.md         |   87 +++++
 tareas/03-en-revision/TASK-023/revision/informe-revision-1-rust-wasm-reviewer.md            |  124 +++++++
 tareas/03-en-revision/TASK-023/revision/peticion-revision-1-code-quality-reviewer.md        |  900 ++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-023/revision/peticion-revision-1-rust-wasm-reviewer.md           | 1920 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-023/tarea.md                                    |   12 +-
 7 files changed, 3040 insertions(+), 3 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff 25e7d2ce796319aac3b8ab5b4d26518c6c197f78..HEAD -- ":(literal)crates/tc2/tests/config.rs"
````

````
crates/tc2/tests/config.rs | 32 ++++++++++++++++++++++++++++++++
 1 file changed, 32 insertions(+)
````
