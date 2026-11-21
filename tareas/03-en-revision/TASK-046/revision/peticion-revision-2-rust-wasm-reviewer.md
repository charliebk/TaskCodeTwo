# Peticion de revision — TASK-046 (ronda 2)

- Tarea: TASK-046 — F5-6 Hook local rapido, opcional y con confianza
- Rama revisada: feature/task-046-f5-6-hook-local-rapido-opcional-y-con-co
- Rama base: develop
- Commit revisado (HEAD): 83f83d690e1069f9fa2c28c04630593d1c1fd353
- Fecha: 2026-11-21
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-046 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde e33879a25a4399d5755559aa147ff96497ca231a (el commit revisado en la ronda anterior)
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
informe de esta ronda (informe-revision-2-rust-wasm-reviewer.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-046/tarea.md
- tareas/03-en-revision/TASK-046/planificacion/
- crates/tc2/src/confianza/cierre.rs
- crates/tc2/src/confianza/mod.rs
- crates/tc2/src/config/validar.rs
- crates/tc2/src/contrato.rs
- crates/tc2/src/i18n/catalogo/config.rs
- crates/tc2/src/i18n/catalogo/verificacion.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/verificacion/ejecutar.rs
- crates/tc2/tests/desinstalar.rs
- crates/tc2/tests/verificar.rs
- docs/ci/verificacion-reutilizable.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 4 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMPORTANTE-1 | IMPORTANTE | corregido (c6ffbc1) | crates/tc2/src/confianza/cierre.rs | informe-revision-1-rust-wasm-reviewer.md |
| IMPORTANTE-2 | IMPORTANTE | corregido (c6ffbc1, f7a9be6) | crates/tc2/src/confianza/cierre.rs, crates/tc2/src/config/validar.rs | informe-revision-1-rust-wasm-reviewer.md |

## Commits a revisar (git log e33879a25a4399d5755559aa147ff96497ca231a..HEAD)

````
83f83d6 chore(TASK-046): informes y estado de los hallazgos de la ronda 1
f7a9be6 test(TASK-046): un glob invalido es error del cierre
8fd2d67 docs(TASK-046): limites del cierre y comportamiento de --hook tras la ronda 1
c6ffbc1 fix(TASK-046): correcciones de la ronda 1 de revision
81a171f chore(TASK-046): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff e33879a25a4399d5755559aa147ff96497ca231a..HEAD)

````diff
diff --git a/crates/tc2/src/confianza/cierre.rs b/crates/tc2/src/confianza/cierre.rs
index 6e72e26..e774048 100644
--- a/crates/tc2/src/confianza/cierre.rs
+++ b/crates/tc2/src/confianza/cierre.rs
@@ -17,7 +17,7 @@ use std::collections::{BTreeMap, BTreeSet, HashMap};
 use std::path::Path;
 use std::time::UNIX_EPOCH;
 
-use globset::{GlobBuilder, GlobSetBuilder};
+use globset::{GlobBuilder, GlobMatcher};
 use serde::{Deserialize, Serialize};
 
 /// Extensiones de lo que se ejecuta como script (`""`: sin extension).
@@ -110,13 +110,35 @@ fn normalizar(base: &str, rel: &str) -> Option<String> {
     (!partes.is_empty()).then(|| partes.join("/"))
 }
 
+/// Caracter de un token sin comillas: alfanumerico (tambien no ASCII, como
+/// `verificación.bat`) o `_ . % ~ - \ /`.
 fn es_de_token(c: char) -> bool {
-    c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '%' | '~' | '-' | '\\' | '/')
+    c.is_alphanumeric() || matches!(c, '_' | '.' | '%' | '~' | '-' | '\\' | '/')
+}
+
+/// Largo maximo de un texto entre comillas que se toma como ruta.
+const MAX_ENTRECOMILLADO: usize = 260;
+
+/// Textos entre comillas (`"..."` o `'...'`) de una linea: cada uno es una
+/// ruta candidata entera, con espacios (`call "%~dp0lib\mi paso.bat"`).
+fn entrecomillados(linea: &str) -> Vec<&str> {
+    let mut v = Vec::new();
+    let mut resto = linea;
+    while let Some(i) = resto.find(['"', '\'']) {
+        let comilla = &resto[i..=i];
+        let tras = &resto[i + 1..];
+        let Some(j) = tras.find(comilla) else {
+            break;
+        };
+        v.push(&tras[..j]);
+        resto = &tras[j + 1..];
+    }
+    v
 }
 
 /// Rutas normalizadas que nombra `texto`, escrito en la carpeta `dir` (vacia
-/// en la raiz). Cada token con forma de ruta da hasta dos: relativo a `dir`
-/// y relativo a la raiz.
+/// en la raiz). Cada token con forma de ruta, y cada texto entre comillas,
+/// da hasta dos: relativo a `dir` y relativo a la raiz.
 pub fn candidatos(texto: &str, dir: &str) -> Vec<String> {
     let dp0 = if dir.is_empty() {
         String::new()
@@ -124,13 +146,14 @@ pub fn candidatos(texto: &str, dir: &str) -> Vec<String> {
         format!("{dir}/")
     };
     let mut v: BTreeSet<String> = BTreeSet::new();
-    for tok in texto.split(|c: char| !es_de_token(c)) {
+    let mut anadir = |tok: &str| {
+        let tok = tok.trim();
         if tok.is_empty() {
-            continue;
+            return;
         }
         let t = tok.replace("%~dp0", &dp0).replace('\\', "/");
         if !t.contains(['.', '/']) || t == "." || t == ".." {
-            continue;
+            return;
         }
         let bases: &[&str] = if dir.is_empty() { &[""] } else { &[dir, ""] };
         for b in bases {
@@ -138,6 +161,16 @@ pub fn candidatos(texto: &str, dir: &str) -> Vec<String> {
                 v.insert(n);
             }
         }
+    };
+    for linea in texto.lines() {
+        for e in entrecomillados(linea) {
+            if e.len() <= MAX_ENTRECOMILLADO {
+                anadir(e);
+            }
+        }
+        for tok in linea.split(|c: char| !es_de_token(c)) {
+            anadir(tok);
+        }
     }
     v.into_iter().collect()
 }
@@ -166,6 +199,51 @@ pub struct Cierre {
     pub ficheros: Vec<Fichero>,
     /// Bytes leidos para hashear (0 si todo salio de la cache).
     pub hasheados: u64,
+    /// Globs de `confianza_incluye` que no casan con ningun fichero
+    /// versionado.
+    pub globs_vacios: Vec<String>,
+}
+
+/// Por que no se pudo calcular el cierre.
+#[derive(Debug)]
+pub enum ErrorCierre {
+    /// Un fichero del cierre no se pudo leer.
+    Lectura(std::io::Error),
+    /// Un glob de `confianza_incluye` no compila (falla cerrado).
+    Glob {
+        /// El glob tal como esta en la configuracion.
+        glob: String,
+        /// Lo que dice el compilador de globs.
+        error: String,
+    },
+}
+
+/// Compila un glob de `confianza_incluye` como se comparan las rutas del
+/// cierre: con `./` inicial quitado (como lo admite el validador), `*` sin
+/// cruzar `/` y sin distinguir mayusculas en Windows y macOS.
+pub fn compilar_glob(glob: &str) -> Result<GlobMatcher, String> {
+    let mut g = glob;
+    while let Some(r) = g.strip_prefix("./") {
+        g = r;
+    }
+    GlobBuilder::new(g)
+        .literal_separator(true)
+        .case_insensitive(cfg!(any(windows, target_os = "macos")))
+        .build()
+        .map(|g| g.compile_matcher())
+        .map_err(|e| e.kind().to_string())
+}
+
+fn compilar_globs(incluye: &[String]) -> Result<Vec<GlobMatcher>, ErrorCierre> {
+    incluye
+        .iter()
+        .map(|g| {
+            compilar_glob(g).map_err(|error| ErrorCierre::Glob {
+                glob: g.clone(),
+                error,
+            })
+        })
+        .collect()
 }
 
 /// Calcula el cierre de `orden` en `raiz`. Con `cache`, un fichero con el
@@ -176,7 +254,7 @@ pub fn calcular(
     orden: &str,
     incluye: &[String],
     cache: Option<&[Fichero]>,
-) -> std::io::Result<Cierre> {
+) -> Result<Cierre, ErrorCierre> {
     let cache: HashMap<&str, &Fichero> = cache
         .unwrap_or_default()
         .iter()
@@ -186,17 +264,14 @@ pub fn calcular(
         .iter()
         .filter_map(|c| indice.script(c).map(str::to_owned))
         .collect();
-    let mut globs = GlobSetBuilder::new();
-    for g in incluye {
-        if let Ok(g) = GlobBuilder::new(g).literal_separator(true).build() {
-            globs.add(g);
+    let mut globs_vacios = Vec::new();
+    for (texto, g) in incluye.iter().zip(compilar_globs(incluye)?) {
+        let antes = pendientes.len();
+        pendientes.extend(indice.todas.iter().filter(|r| g.is_match(r)).cloned());
+        if pendientes.len() == antes {
+            globs_vacios.push(texto.clone());
         }
     }
-    if let Ok(set) = globs.build()
-        && !incluye.is_empty()
-    {
-        pendientes.extend(indice.todas.iter().filter(|r| set.is_match(r)).cloned());
-    }
     let mut vistos: BTreeMap<String, Fichero> = BTreeMap::new();
     let mut hasheados = 0u64;
     while let Some(rel) = pendientes.pop() {
@@ -214,9 +289,10 @@ pub fn calcular(
         }
         let (tamano, mtime) = (meta.len(), mtime_ns(&meta));
         let f = match cache.get(rel.as_str()) {
-            Some(c) if c.tamano == tamano && c.mtime == mtime => (*c).clone(),
+            // Sin `mtime` del sistema (0) no hay cache: se lee siempre.
+            Some(c) if c.tamano == tamano && c.mtime == mtime && mtime != 0 => (*c).clone(),
             _ => {
-                let datos = std::fs::read(&ruta)?;
+                let datos = std::fs::read(&ruta).map_err(ErrorCierre::Lectura)?;
                 hasheados += datos.len() as u64;
                 let dir = rel.rsplit_once('/').map_or("", |(d, _)| d);
                 let candidatos = if es_script(&rel) && tamano <= MAX_ESCANEO {
@@ -245,6 +321,7 @@ pub fn calcular(
     Ok(Cierre {
         ficheros: vistos.into_values().collect(),
         hasheados,
+        globs_vacios,
     })
 }
 
@@ -286,4 +363,23 @@ mod tests {
         assert!(es_script("scripts/verificar"));
         assert!(!es_script("README.md"));
     }
+
+    /// Ronda 1, IMP-2: un glob de `confianza_incluye` que no compila no se
+    /// ignora (aunque la config ya lo rechace, el cierre falla cerrado solo).
+    #[test]
+    fn un_glob_invalido_es_error_del_cierre() {
+        let indice = Indice::de_ls_files("a.sh\0");
+        let r = calcular(
+            Path::new("."),
+            &indice,
+            "sh a.sh",
+            &["x/[".to_owned()],
+            None,
+        );
+        assert!(
+            matches!(r, Err(ErrorCierre::Glob { ref glob, .. }) if glob == "x/["),
+            "{:?}",
+            r.err()
+        );
+    }
 }
diff --git a/crates/tc2/src/confianza/mod.rs b/crates/tc2/src/confianza/mod.rs
index 5482ec1..8323853 100644
--- a/crates/tc2/src/confianza/mod.rs
+++ b/crates/tc2/src/confianza/mod.rs
@@ -41,7 +41,10 @@ pub const NOMBRE: &str = "confiar";
 pub const CLAVES: [&str; 1] = ["verificacion.comando"];
 
 /// Version del fichero del almacen.
-const VERSION_ALMACEN: u32 = 1;
+/// Version del fichero del almacen. La 2 (ronda 1 de TASK-046) cambia los
+/// `candidatos` (comillas y no ASCII): un almacen de la 1 se lee vacio y todo
+/// vuelve a `nunca` (falla cerrado).
+const VERSION_ALMACEN: u32 = 2;
 
 /// Estado de confianza de una orden.
 #[derive(Debug, Clone, Copy, PartialEq, Eq)]
@@ -76,6 +79,8 @@ pub struct Evaluacion {
     pub ficheros: usize,
     /// Bytes leidos para hashear (0 en caliente y sin cambios).
     pub hasheados: u64,
+    /// Globs de `confianza_incluye` que no casan con ningun fichero.
+    pub globs_vacios: Vec<String>,
 }
 
 /// Un repo abierto para la confianza: su arbol de trabajo, el nombre de su
@@ -173,9 +178,19 @@ fn escribir_almacen(ruta: &Path, a: &Almacen, op: &str) -> Result<(), Informe> {
     })
 }
 
-fn error_de_lectura(op: &str, e: &std::io::Error) -> Informe {
-    Informe::nuevo(op, Resultado::Error)
-        .diagnostico(Mensaje::nuevo(&t::CIERRE_NO_LEIDO).con("error", e.to_string()))
+fn error_de_cierre(op: &str, e: &cierre::ErrorCierre) -> Informe {
+    match e {
+        cierre::ErrorCierre::Lectura(e) => Informe::nuevo(op, Resultado::Error)
+            .diagnostico(Mensaje::nuevo(&t::CIERRE_NO_LEIDO).con("error", e.to_string())),
+        cierre::ErrorCierre::Glob { glob, error } => Informe::nuevo(op, Resultado::Rechazada)
+            .con("motivo", "glob-invalido")
+            .diagnostico(
+                Mensaje::nuevo(&t::GLOB_INVALIDO)
+                    .con("glob", glob.as_str())
+                    .con("error", error.as_str())
+                    .con("ruta", crate::config::RUTA),
+            ),
+    }
 }
 
 /// Estado de confianza de `orden` (la de `clave`) en `repo`. En caliente
@@ -199,7 +214,7 @@ pub fn estado(
         incluye,
         entrada.as_ref().map(|e| e.ficheros.as_slice()),
     )
-    .map_err(|e| error_de_lectura(op, &e))?;
+    .map_err(|e| error_de_cierre(op, &e))?;
     let huella = cierre::huella(orden, incluye, &c.ficheros);
     let estado = match &entrada {
         None => Estado::Nunca,
@@ -223,6 +238,7 @@ pub fn estado(
         huella,
         ficheros: c.ficheros.len(),
         hasheados: c.hasheados,
+        globs_vacios: c.globs_vacios.clone(),
     })
 }
 
@@ -240,7 +256,7 @@ pub fn confiar(
     let mut a = leer_almacen(&ruta);
     a.version = VERSION_ALMACEN;
     let c = cierre::calcular(&repo.raiz, &repo.indice, orden, incluye, None)
-        .map_err(|e| error_de_lectura(op, &e))?;
+        .map_err(|e| error_de_cierre(op, &e))?;
     let huella = cierre::huella(orden, incluye, &c.ficheros);
     a.claves.insert(
         clave.to_owned(),
@@ -255,6 +271,7 @@ pub fn confiar(
         huella,
         ficheros: c.ficheros.len(),
         hasheados: c.hasheados,
+        globs_vacios: c.globs_vacios.clone(),
     })
 }
 
@@ -315,11 +332,32 @@ fn confiar_en(clave: &str) -> Result<Informe, Informe> {
         &config.confianza_incluye,
         NOMBRE,
     )?;
-    Ok(Informe::nuevo(NOMBRE, Resultado::Ok)
+    let i = Informe::nuevo(NOMBRE, Resultado::Ok)
         .con("clave", clave)
         .con("huella", crate::contrato::corto(&ev.huella))
         .con("ficheros", ev.ficheros.to_string())
-        .con("hasheados", ev.hasheados.to_string()))
+        .con("hasheados", ev.hasheados.to_string());
+    Ok(if ev.globs_vacios.is_empty() {
+        i
+    } else {
+        i.con_lista("globs-sin-ficheros", ev.globs_vacios)
+    })
+}
+
+/// Aviso para la persona (stderr, codigo 0) si `tc2 confiar` salio bien con
+/// globs de `confianza_incluye` que no casan con nada: lo que declaro no
+/// entra en la confianza.
+pub fn aviso_globs(i: &Informe, idioma: crate::i18n::Idioma) -> Option<String> {
+    let globs = i.claves.iter().find_map(|(k, v)| match v {
+        crate::contrato::Valor::Lista(l) if k == "globs-sin-ficheros" => Some(l.join(", ")),
+        _ => None,
+    })?;
+    Some(
+        Mensaje::nuevo(&t::GLOBS_SIN_FICHEROS)
+            .con("globs", globs)
+            .con("ruta", crate::config::RUTA)
+            .texto(idioma),
+    )
 }
 
 /// El almacen del entorno de este proceso, o el informe que dice que hacer.
diff --git a/crates/tc2/src/config/validar.rs b/crates/tc2/src/config/validar.rs
index 61faf1d..9c89a01 100644
--- a/crates/tc2/src/config/validar.rs
+++ b/crates/tc2/src/config/validar.rs
@@ -244,9 +244,15 @@ pub(crate) fn comprobar(regla: Regla, s: &str) -> Option<Mensaje> {
         }
         Regla::Glob | Regla::GlobAcotado => {
             let motivo = if let Some(m) = glob_malo(s) {
-                Some(m)
+                Some(Mensaje::nuevo(m))
             } else if regla == Regla::GlobAcotado && todo_el_arbol(s) {
-                Some(&t::NUNCA_TODO_EL_ARBOL)
+                Some(Mensaje::nuevo(&t::NUNCA_TODO_EL_ARBOL))
+            } else if regla == Regla::GlobAcotado {
+                // `confianza_incluye`: un glob que no compila no se ignora
+                // (F1-8 decision 5, falla cerrado).
+                crate::confianza::cierre::compilar_glob(s)
+                    .err()
+                    .map(|e| Mensaje::nuevo(&t::GLOB_NO_COMPILA).con("error", e))
             } else {
                 None
             };
@@ -420,6 +426,8 @@ mod tests {
         assert!(comprobar(Regla::GlobAcotado, "**").is_some());
         assert!(comprobar(Regla::GlobAcotado, "*").is_some());
         assert!(comprobar(Regla::GlobAcotado, "scripts/*.py").is_none());
+        assert!(comprobar(Regla::GlobAcotado, "./cfg/*.cfg").is_none());
+        assert!(comprobar(Regla::GlobAcotado, "cfg/[").is_some());
         assert!(comprobar(Regla::Cron, "0 3 * * 1-5").is_none());
         assert!(comprobar(Regla::Cron, "0 3 * *").is_some());
         assert!(comprobar(Regla::Cron, "@daily x y z w").is_some());
diff --git a/crates/tc2/src/contrato.rs b/crates/tc2/src/contrato.rs
index 4864435..a92ddaf 100644
--- a/crates/tc2/src/contrato.rs
+++ b/crates/tc2/src/contrato.rs
@@ -319,7 +319,7 @@ pub(crate) fn escapar_valor(v: &str, en_lista: bool) -> String {
     s
 }
 
-fn una_linea(t: &str) -> String {
+pub(crate) fn una_linea(t: &str) -> String {
     t.lines().map(str::trim).collect::<Vec<_>>().join(" ")
 }
 
diff --git a/crates/tc2/src/i18n/catalogo/config.rs b/crates/tc2/src/i18n/catalogo/config.rs
index 92ea2cb..1d9c401 100644
--- a/crates/tc2/src/i18n/catalogo/config.rs
+++ b/crates/tc2/src/i18n/catalogo/config.rs
@@ -200,6 +200,10 @@ crate::i18n::textos! {
         es: "glob no admitido «{glob}»: {motivo}",
         en: "glob not accepted «{glob}»: {motivo}",
     }
+    GLOB_NO_COMPILA {
+        es: "no es un glob valido ({error}); revisa corchetes y llaves",
+        en: "it is not a valid glob ({error}); check brackets and braces",
+    }
     NUNCA_TODO_EL_ARBOL {
         es: "nunca todo el arbol",
         en: "never the whole tree",
diff --git a/crates/tc2/src/i18n/catalogo/verificacion.rs b/crates/tc2/src/i18n/catalogo/verificacion.rs
index fa94c89..f497314 100644
--- a/crates/tc2/src/i18n/catalogo/verificacion.rs
+++ b/crates/tc2/src/i18n/catalogo/verificacion.rs
@@ -26,6 +26,14 @@ crate::i18n::textos! {
         es: "no se pudo leer un fichero de lo que ejecuta la orden: {error}",
         en: "could not read a file of what the command runs: {error}",
     }
+    GLOB_INVALIDO {
+        es: "el glob «{glob}» de confianza_incluye no es valido ({error}) y no se confia ni se ejecuta nada: corrigelo en {ruta}",
+        en: "the glob «{glob}» in confianza_incluye is not valid ({error}) and nothing is trusted or run: fix it in {ruta}",
+    }
+    GLOBS_SIN_FICHEROS {
+        es: "aviso: estos globs de confianza_incluye no casan con ningun fichero versionado y no suman nada a la confianza: {globs}; revisalos en {ruta}",
+        en: "warning: these confianza_incluye globs match no tracked file and add nothing to the trust: {globs}; check them in {ruta}",
+    }
     USO_VERIFICAR {
         es: "uso: verificar [--json] --perfil <rapida|impacto|completa|pesada> [--hook]",
         en: "usage: verificar [--json] --perfil <rapida|impacto|completa|pesada> [--hook]",
@@ -62,6 +70,10 @@ crate::i18n::textos! {
         es: "tc2 verificar --hook: no se ejecuta la verificacion {perfil}: {motivo}",
         en: "tc2 verificar --hook: the {perfil} verification is not run: {motivo}",
     }
+    HOOK_MAL_ESCRITO {
+        es: "tc2 verificar --hook: la entrada del hook esta mal escrita y no se ejecuta nada: {motivo}; corrigela en settings.json (`tc2 verificar --perfil rapida --hook`)",
+        en: "tc2 verificar --hook: the hook entry is malformed and nothing is run: {motivo}; fix it in settings.json (`tc2 verificar --perfil rapida --hook`)",
+    }
     HOOK_ROJO {
         es: "tc2 verificar --hook: {motivo}. Ultimas lineas de su salida:",
         en: "tc2 verificar --hook: {motivo}. Last lines of its output:",
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 8b472e1..2f84ca6 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -241,7 +241,15 @@ where
         }
         [orden, resto @ ..] if orden == confianza::NOMBRE => {
             let (json, resto) = con_json(resto);
-            confianza::orden(&resto).salida_en(json, idioma)
+            let i = confianza::orden(&resto);
+            let mut s = i.salida_en(json, idioma);
+            if s.codigo == 0
+                && let Some(a) = confianza::aviso_globs(&i, idioma)
+            {
+                s.stderr.push_str(&a);
+                s.stderr.push('\n');
+            }
+            s
         }
         [orden, resto @ ..] if orden == identidad::NOMBRE => {
             let (json, resto) = con_json(resto);
diff --git a/crates/tc2/src/verificacion/ejecutar.rs b/crates/tc2/src/verificacion/ejecutar.rs
index 81b11ff..939de95 100644
--- a/crates/tc2/src/verificacion/ejecutar.rs
+++ b/crates/tc2/src/verificacion/ejecutar.rs
@@ -50,29 +50,30 @@ const SONDEO: Duration = Duration::from_millis(10);
 
 /// `tc2 verificar` con los argumentos ya sin `--json`.
 pub fn orden(args: &[String], json: bool, idioma: Idioma) -> Salida {
-    let mut perfil = None;
-    let mut hook = false;
-    let mut it = args.iter();
-    while let Some(a) = it.next() {
-        match a.as_str() {
-            "--perfil" => match it.next() {
-                Some(p) if perfil.is_none() => perfil = Some(p.clone()),
-                _ => return uso().salida_en(json, idioma),
-            },
-            "--hook" if !hook => hook = true,
-            _ => return uso().salida_en(json, idioma),
-        }
-    }
-    let Some(perfil) = perfil.filter(|p| PERFILES.contains(&p.as_str())) else {
-        return uso().salida_en(json, idioma);
-    };
+    // Con `--hook`, nada que no sea el rojo de la rapida bloquea la parada:
+    // `stop_hook_active` corta antes de validar nada mas, y un error de uso
+    // (la entrada de settings.json mal escrita) sale con 0 y un aviso.
+    let hook = args.iter().any(|a| a == "--hook");
     if hook && stop_hook_activo() {
         return Informe::nuevo(NOMBRE, Resultado::SinCambios)
             .con("motivo", "stop-hook-activo")
-            .con("perfil", perfil.as_str())
             .salida_en(json, idioma);
     }
-    let captura = hook.then(fichero_de_captura);
+    let perfil = match leer_argumentos(args) {
+        Ok(p) => p,
+        Err(i) if hook => {
+            let mut s = i.salida_en(json, idioma);
+            s.codigo = 0;
+            s.stderr = aviso_de_hook(
+                Mensaje::nuevo(&t::HOOK_MAL_ESCRITO)
+                    .con("motivo", i.diagnostico_en(idioma).unwrap_or_default()),
+                idioma,
+            );
+            return s;
+        }
+        Err(i) => return i.salida_en(json, idioma),
+    };
+    let captura = if hook { crear_captura() } else { None };
     let informe = match std::env::current_dir() {
         Ok(dir) => {
             let almacen = confianza::almacen_del_proceso(NOMBRE);
@@ -102,6 +103,32 @@ fn uso() -> Informe {
         .diagnostico(&t::USO_VERIFICAR)
 }
 
+/// El perfil de `--perfil <p> [--hook]`, o el informe de uso.
+fn leer_argumentos(args: &[String]) -> Result<String, Informe> {
+    let mut perfil = None;
+    let mut hook = false;
+    let mut it = args.iter();
+    while let Some(a) = it.next() {
+        match a.as_str() {
+            "--perfil" => match it.next() {
+                Some(p) if perfil.is_none() => perfil = Some(p.clone()),
+                _ => return Err(uso()),
+            },
+            "--hook" if !hook => hook = true,
+            _ => return Err(uso()),
+        }
+    }
+    perfil
+        .filter(|p| PERFILES.contains(&p.as_str()))
+        .ok_or_else(uso)
+}
+
+/// Una linea de stderr para la persona con `--hook` (el diagnostico de una
+/// config con YAML invalido trae varias).
+fn aviso_de_hook(m: Mensaje, idioma: Idioma) -> String {
+    format!("{}\n", crate::contrato::una_linea(&m.texto(idioma)))
+}
+
 /// `stop_hook_active` del JSON de stdin del evento `Stop`. Sin JSON (stdin
 /// en una terminal, vacio o mal formado) no se sabe y se sigue: el tope de
 /// continuaciones de Claude Code corta un bucle.
@@ -121,8 +148,44 @@ fn stop_hook_activo() -> bool {
         .unwrap_or(false)
 }
 
-fn fichero_de_captura() -> PathBuf {
-    std::env::temp_dir().join(format!("tc2-verificar-{}.log", std::process::id()))
+/// Fichero donde `--hook` recoge la salida del comando: nombre aleatorio y
+/// creado en exclusiva (`create_new`, no sigue un enlace ni pisa un fichero
+/// que ya exista); se borra al terminar. Si no se puede crear, la salida va a
+/// stderr como sin `--hook`.
+fn crear_captura() -> Option<PathBuf> {
+    use ring::rand::SecureRandom;
+    let mut azar = [0u8; 16];
+    ring::rand::SystemRandom::new().fill(&mut azar).ok()?;
+    let nombre = crate::confianza::cierre::sha_hex(&azar);
+    let ruta = std::env::temp_dir().join(format!("tc2-verificar-{}.log", &nombre[..32]));
+    std::fs::File::options()
+        .write(true)
+        .create_new(true)
+        .open(&ruta)
+        .ok()
+        .map(|_| ruta)
+}
+
+/// Tope de lo que se lee del final de la captura para el resumen.
+const COLA_BYTES: u64 = 64 * 1024;
+
+/// Las ultimas [`LINEAS_RESUMEN`] lineas de la captura, leyendo solo su
+/// final.
+fn cola(ruta: &Path) -> Vec<String> {
+    use std::io::{Seek, SeekFrom};
+    let Ok(mut f) = std::fs::File::open(ruta) else {
+        return Vec::new();
+    };
+    let largo = f.metadata().map(|m| m.len()).unwrap_or(0);
+    let _ = f.seek(SeekFrom::Start(largo.saturating_sub(COLA_BYTES)));
+    let mut datos = Vec::new();
+    let _ = f.take(COLA_BYTES).read_to_end(&mut datos);
+    let texto = String::from_utf8_lossy(&datos);
+    let lineas: Vec<&str> = texto.lines().collect();
+    lineas[lineas.len().saturating_sub(LINEAS_RESUMEN)..]
+        .iter()
+        .map(|l| (*l).to_owned())
+        .collect()
 }
 
 /// La decision y la ejecucion, comunes a la orden y al hook. Con `captura`,
@@ -300,7 +363,8 @@ fn lanzar(
         .stdin(Stdio::null());
     match captura {
         Some(ruta) => {
-            let f = std::fs::File::create(ruta)?;
+            // Ya creado en exclusiva por `crear_captura`: no se crea de nuevo.
+            let f = std::fs::File::options().write(true).open(ruta)?;
             c.stdout(f.try_clone()?).stderr(f);
         }
         // La salida del comando no se mezcla con la linea de contrato.
@@ -376,24 +440,19 @@ fn como_hook(
             .con("motivo", motivo)
             .texto(idioma);
         resumen.push('\n');
-        if let Some(texto) = captura.and_then(|c| std::fs::read(c).ok()) {
-            let texto = String::from_utf8_lossy(&texto);
-            let lineas: Vec<&str> = texto.lines().collect();
-            for l in &lineas[lineas.len().saturating_sub(LINEAS_RESUMEN)..] {
-                resumen.push_str(l);
-                resumen.push('\n');
-            }
+        for l in captura.map(cola).unwrap_or_default() {
+            resumen.push_str(&l);
+            resumen.push('\n');
         }
         s.codigo = 2;
         s.stderr = resumen;
     } else {
         s.codigo = 0;
-        s.stderr = format!(
-            "{}\n",
+        s.stderr = aviso_de_hook(
             Mensaje::nuevo(&t::HOOK_NO_SE_EJECUTA)
                 .con("perfil", perfil)
-                .con("motivo", motivo)
-                .texto(idioma)
+                .con("motivo", motivo),
+            idioma,
         );
     }
     s
diff --git a/docs/ci/verificacion-reutilizable.md b/docs/ci/verificacion-reutilizable.md
index 1eff885..e28b44a 100644
--- a/docs/ci/verificacion-reutilizable.md
+++ b/docs/ci/verificacion-reutilizable.md
@@ -507,11 +507,21 @@ antes que `tc2`. El evento `Stop` no admite `matcher`. Comportamiento con
 
 | Caso | Código | Qué ve el agente |
 |---|---|---|
-| `stop_hook_active: true` en el JSON de stdin (Claude Code ya continúa por un hook) | 0 | nada; no se ejecuta |
+| `stop_hook_active: true` en el JSON de stdin (Claude Code ya continúa por un hook) | 0 | nada; no se ejecuta. Se mira antes que los argumentos |
+| entrada mal escrita (perfil inexistente, opción desconocida) | 0 | no bloquea; una línea en stderr dice que se corrija en `settings.json` |
 | sin confianza, sin perfil `rapida` o con la configuración inválida | 0 | no bloquea; una línea en stderr dice qué hacer (`tc2 confiar verificacion.comando`, o el arreglo de la configuración); no se ejecuta |
 | rápida en verde o `excedido` | 0 | nada |
 | rápida en rojo | 2 | bloquea la parada; en stderr el motivo y las últimas 15 líneas de la salida del comando |
 
+Con otro perfil (`--perfil completa --hook`, fuera de lo documentado), su
+rojo y su `timeout` agotado también salen con 2. La salida del comando se
+recoge en un fichero temporal de nombre aleatorio, creado en exclusiva, del
+que solo se lee el final y que se borra al terminar (si Claude Code mata el
+hook por su `timeout`, puede quedar en la carpeta temporal del usuario).
+`--hook` lee el JSON de stdin hasta que se cierra: Claude Code lo cierra tras
+escribirlo; un lanzador que lo deje abierto hace esperar a `tc2` (aceptado,
+MENOR-3 de la ronda 1).
+
 `desinstalar` no edita `settings.json`: avisa de los hooks que nombran `tc2`
 y los deja (F2-14).
 
@@ -540,12 +550,32 @@ Con el binario de depuración, el peor caso fue 530 ms (en frío). Se lanza a
 mano: `cargo test -p tc2 --release --test verificar -- --ignored --nocapture`
 (la verificación del repo no corre las pruebas ignoradas).
 
-**Límites** (F1-8, decisión 5): no entra en el cierre el código al que se
-llega por herramientas (`build.rs`, tests, dependencias) ni lo que se carga
-por nombre de módulo y no por ruta; eso se declara en `confianza_incluye`.
-Un cambio que conserve tamaño y `mtime` no se detecta en caliente. No se han
-revisado enlaces simbólicos ni la carrera entre hashear y ejecutar. En
-Windows y macOS las rutas del cierre se comparan sin distinguir mayúsculas.
+**Cómo se forma el cierre.** Se siguen los tokens con forma de ruta (llevan
+`.` o `/`, con letras también no ASCII, como `verificación.bat`) y, enteros,
+los textos entre comillas de una línea (`call "%~dp0lib\mi paso.bat"`), con
+`%~dp0` resuelto a la carpeta del script. En Windows y macOS las rutas se
+comparan sin distinguir mayúsculas. Los globs de `confianza_incluye` admiten
+`./` al principio y siguen la misma regla de mayúsculas; un glob que no
+compila es error de la configuración (no se confía ni se ejecuta nada) y uno
+que no casa con ningún fichero versionado se avisa en `tc2 confiar`
+(`globs-sin-ficheros=` y una línea en stderr).
+
+**Límites** (F1-8, decisión 5), riesgos residuales aceptados:
+
+- No entra en el cierre el código al que se llega por herramientas
+  (`build.rs`, tests, dependencias) ni lo que se carga por nombre de módulo y
+  no por ruta; eso se declara en `confianza_incluye`.
+- Solo cuentan los ficheros **versionados**: un script sin versionar o
+  ignorado que la orden llame se ejecuta sin entrar en la huella.
+- Un enlace simbólico a fichero se hashea por su destino; un enlace a
+  **carpeta** o un **submódulo** dejan sus scripts fuera (`git ls-files` no
+  los lista dentro). No probado en esta máquina (sin permiso para `mklink`).
+- Un cambio que conserve tamaño y `mtime` no se detecta en caliente (D2a);
+  si el sistema no da `mtime`, no hay caché y se rehashea siempre.
+- Entre hashear y ejecutar hay una ventana (TOCTOU): la shell relee los
+  scripts al ejecutarlos y quien pueda escribir en el árbol en ese instante
+  ejecuta lo suyo.
+
 Este repo aún no puede estrenar el hook: su `.taskcode/config.yml` es de la
 0.16 (sin `formato: 2`) y `tc2 verificar` la rechaza; con `--hook` sale con 0
 y avisa.
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff e33879a25a4399d5755559aa147ff96497ca231a..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-046/planificacion/brainstorm/peticion-plan-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-046/planificacion/plan-final.md                 |    0
 tareas/03-en-revision/TASK-046/revision/informe-revision-1-code-quality-reviewer.md         |   95 +++++++
 tareas/03-en-revision/TASK-046/revision/informe-revision-1-rust-wasm-reviewer.md            |   81 ++++++
 tareas/03-en-revision/TASK-046/revision/peticion-revision-1-code-quality-reviewer.md        |  206 +++++++++++++++
 tareas/03-en-revision/TASK-046/revision/peticion-revision-1-rust-wasm-reviewer.md           | 1385 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-046/tarea.md                                    |   14 +-
 7 files changed, 1779 insertions(+), 2 deletions(-)
````

## Tests del diff (2 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff e33879a25a4399d5755559aa147ff96497ca231a..HEAD -- ":(literal)crates/tc2/tests/desinstalar.rs" ":(literal)crates/tc2/tests/verificar.rs"
````

````
crates/tc2/tests/desinstalar.rs |   4 +++
 crates/tc2/tests/verificar.rs   | 251 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++--
 2 files changed, 252 insertions(+), 3 deletions(-)
````
