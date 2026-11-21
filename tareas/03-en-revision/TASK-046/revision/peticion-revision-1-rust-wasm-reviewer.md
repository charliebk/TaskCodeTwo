# Peticion de revision — TASK-046 (ronda 1)

- Tarea: TASK-046 — F5-6 Hook local rapido, opcional y con confianza
- Rama revisada: feature/task-046-f5-6-hook-local-rapido-opcional-y-con-co
- Rama base: develop
- Commit revisado (HEAD): e33879a25a4399d5755559aa147ff96497ca231a
- Fecha: 2026-11-21
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-046 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-046/tarea.md
- tareas/03-en-revision/TASK-046/planificacion/
- crates/tc2/src/confianza/cierre.rs
- crates/tc2/src/confianza/mod.rs
- crates/tc2/src/config/mod.rs
- crates/tc2/src/desinstalar/maquina.rs
- crates/tc2/src/git/puerto.rs
- crates/tc2/src/i18n/catalogo/cli.rs
- crates/tc2/src/i18n/catalogo/config.rs
- crates/tc2/src/i18n/catalogo/mod.rs
- crates/tc2/src/i18n/catalogo/verificacion.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/verificacion/ejecutar.rs
- crates/tc2/src/verificacion/mod.rs
- crates/tc2/tests/desinstalar.rs
- crates/tc2/tests/verificar.rs

## Commits a revisar (git log develop..HEAD)

````
e33879a docs(TASK-046): hook local de F5-6, entrada de settings.json y cifra del presupuesto de 1 s
b2f573b feat(TASK-046): desinstalar --maquina borra el almacen de confianza
74017d5 feat(TASK-046): tc2 verificar con la puerta de confianza y adaptador --hook
1245aa5 feat(TASK-046): confianza local por clave y tc2 confiar
6e88918 chore(TASK-046): tarea en curso
````

## Diff de tu dominio (14 fichero(s) de 15; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/confianza/cierre.rs b/crates/tc2/src/confianza/cierre.rs
new file mode 100644
index 0000000..6e72e26
--- /dev/null
+++ b/crates/tc2/src/confianza/cierre.rs
@@ -0,0 +1,289 @@
+//! Cierre de una orden (F1-8 decision 5): los ficheros **versionados** que
+//! se ejecutan como script y que la orden nombra y, a su vez, los que nombran
+//! estos (`%~dp0` resuelto), mas los de los globs de `confianza_incluye`.
+//!
+//! Solo se siguen tokens con forma de ruta (llevan `.` o `/`), como el
+//! prototipo de la PoC A1 de F1-8 (`tareas/04-terminadas/TASK-009/poc/
+//! poc-f1-8.sh`): lo que se carga por nombre de modulo no entra, y se declara
+//! en `confianza_incluye`.
+//!
+//! La cache de D2a: un fichero cuyo `(tamano, mtime)` coincide con el de la
+//! entrada confiada no se vuelve a leer; se reutilizan su `sha` y sus
+//! candidatos. Los candidatos son rutas normalizadas, no resueltas: un
+//! fichero versionado despues de confiar entra en el cierre aunque el script
+//! que lo nombra no haya cambiado.
+
+use std::collections::{BTreeMap, BTreeSet, HashMap};
+use std::path::Path;
+use std::time::UNIX_EPOCH;
+
+use globset::{GlobBuilder, GlobSetBuilder};
+use serde::{Deserialize, Serialize};
+
+/// Extensiones de lo que se ejecuta como script (`""`: sin extension).
+const SCRIPT: [&str; 11] = [
+    "bat", "cmd", "sh", "ps1", "psm1", "py", "js", "mjs", "cjs", "ts", "",
+];
+
+/// Por encima de este tamano un fichero se hashea pero no se buscan rutas en
+/// el (como la PoC).
+const MAX_ESCANEO: u64 = 2_000_000;
+
+/// Un fichero del cierre, como se guarda en el almacen.
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+pub struct Fichero {
+    /// Ruta relativa a la raiz del arbol, con `/`, como la da `git ls-files`.
+    pub ruta: String,
+    /// Bytes.
+    pub tamano: u64,
+    /// `mtime` en nanosegundos desde la epoca Unix.
+    pub mtime: u64,
+    /// sha256 del contenido, en hexadecimal.
+    pub sha: String,
+    /// Rutas normalizadas que nombra (sin resolver).
+    pub candidatos: Vec<String>,
+}
+
+/// Ficheros versionados del arbol, con la clave de busqueda del sistema: sin
+/// distinguir mayusculas en Windows y macOS, donde `Scripts\V.bat` ejecuta
+/// `scripts/v.bat`.
+pub struct Indice {
+    rutas: HashMap<String, String>,
+    todas: Vec<String>,
+}
+
+fn clave_busqueda(ruta: &str) -> String {
+    if cfg!(any(windows, target_os = "macos")) {
+        ruta.to_lowercase()
+    } else {
+        ruta.to_owned()
+    }
+}
+
+impl Indice {
+    /// Indice de la salida de `git ls-files -z`.
+    pub fn de_ls_files(salida: &str) -> Self {
+        let todas: Vec<String> = salida
+            .split('\0')
+            .filter(|r| !r.is_empty())
+            .map(str::to_owned)
+            .collect();
+        let rutas = todas
+            .iter()
+            .map(|r| (clave_busqueda(r), r.clone()))
+            .collect();
+        Self { rutas, todas }
+    }
+
+    /// La ruta versionada que es `candidato`, si se ejecuta como script.
+    fn script(&self, candidato: &str) -> Option<&str> {
+        let real = self.rutas.get(&clave_busqueda(candidato))?;
+        es_script(real).then_some(real.as_str())
+    }
+}
+
+fn es_script(ruta: &str) -> bool {
+    let nombre = ruta.rsplit('/').next().unwrap_or(ruta);
+    let ext = match nombre.rfind('.') {
+        Some(i) if i > 0 => nombre[i + 1..].to_ascii_lowercase(),
+        _ => String::new(),
+    };
+    SCRIPT.contains(&ext.as_str())
+}
+
+/// Une `base` y `rel` y quita `.` y `..`. `None` si sale del arbol o es
+/// absoluta.
+fn normalizar(base: &str, rel: &str) -> Option<String> {
+    if rel.starts_with('/') {
+        return None;
+    }
+    let mut partes: Vec<&str> = Vec::new();
+    for p in base.split('/').chain(rel.split('/')) {
+        match p {
+            "" | "." => {}
+            ".." => {
+                partes.pop()?;
+            }
+            otro => partes.push(otro),
+        }
+    }
+    (!partes.is_empty()).then(|| partes.join("/"))
+}
+
+fn es_de_token(c: char) -> bool {
+    c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '%' | '~' | '-' | '\\' | '/')
+}
+
+/// Rutas normalizadas que nombra `texto`, escrito en la carpeta `dir` (vacia
+/// en la raiz). Cada token con forma de ruta da hasta dos: relativo a `dir`
+/// y relativo a la raiz.
+pub fn candidatos(texto: &str, dir: &str) -> Vec<String> {
+    let dp0 = if dir.is_empty() {
+        String::new()
+    } else {
+        format!("{dir}/")
+    };
+    let mut v: BTreeSet<String> = BTreeSet::new();
+    for tok in texto.split(|c: char| !es_de_token(c)) {
+        if tok.is_empty() {
+            continue;
+        }
+        let t = tok.replace("%~dp0", &dp0).replace('\\', "/");
+        if !t.contains(['.', '/']) || t == "." || t == ".." {
+            continue;
+        }
+        let bases: &[&str] = if dir.is_empty() { &[""] } else { &[dir, ""] };
+        for b in bases {
+            if let Some(n) = normalizar(b, &t) {
+                v.insert(n);
+            }
+        }
+    }
+    v.into_iter().collect()
+}
+
+fn mtime_ns(m: &std::fs::Metadata) -> u64 {
+    m.modified()
+        .ok()
+        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
+        .map_or(0, |d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX))
+}
+
+/// sha256 en hexadecimal.
+pub fn sha_hex(datos: &[u8]) -> String {
+    use std::fmt::Write;
+    let d = ring::digest::digest(&ring::digest::SHA256, datos);
+    let mut s = String::with_capacity(64);
+    for b in d.as_ref() {
+        let _ = write!(s, "{b:02x}");
+    }
+    s
+}
+
+/// Resultado de calcular un cierre.
+pub struct Cierre {
+    /// Ficheros, por ruta.
+    pub ficheros: Vec<Fichero>,
+    /// Bytes leidos para hashear (0 si todo salio de la cache).
+    pub hasheados: u64,
+}
+
+/// Calcula el cierre de `orden` en `raiz`. Con `cache`, un fichero con el
+/// mismo `(tamano, mtime)` que su entrada no se vuelve a leer.
+pub fn calcular(
+    raiz: &Path,
+    indice: &Indice,
+    orden: &str,
+    incluye: &[String],
+    cache: Option<&[Fichero]>,
+) -> std::io::Result<Cierre> {
+    let cache: HashMap<&str, &Fichero> = cache
+        .unwrap_or_default()
+        .iter()
+        .map(|f| (f.ruta.as_str(), f))
+        .collect();
+    let mut pendientes: Vec<String> = candidatos(orden, "")
+        .iter()
+        .filter_map(|c| indice.script(c).map(str::to_owned))
+        .collect();
+    let mut globs = GlobSetBuilder::new();
+    for g in incluye {
+        if let Ok(g) = GlobBuilder::new(g).literal_separator(true).build() {
+            globs.add(g);
+        }
+    }
+    if let Ok(set) = globs.build()
+        && !incluye.is_empty()
+    {
+        pendientes.extend(indice.todas.iter().filter(|r| set.is_match(r)).cloned());
+    }
+    let mut vistos: BTreeMap<String, Fichero> = BTreeMap::new();
+    let mut hasheados = 0u64;
+    while let Some(rel) = pendientes.pop() {
+        if vistos.contains_key(&rel) {
+            continue;
+        }
+        let ruta = raiz.join(&rel);
+        // Versionado pero borrado del arbol: no se ejecuta; que falte cambia
+        // la huella.
+        let Ok(meta) = std::fs::metadata(&ruta) else {
+            continue;
+        };
+        if !meta.is_file() {
+            continue;
+        }
+        let (tamano, mtime) = (meta.len(), mtime_ns(&meta));
+        let f = match cache.get(rel.as_str()) {
+            Some(c) if c.tamano == tamano && c.mtime == mtime => (*c).clone(),
+            _ => {
+                let datos = std::fs::read(&ruta)?;
+                hasheados += datos.len() as u64;
+                let dir = rel.rsplit_once('/').map_or("", |(d, _)| d);
+                let candidatos = if es_script(&rel) && tamano <= MAX_ESCANEO {
+                    candidatos(&String::from_utf8_lossy(&datos), dir)
+                } else {
+                    Vec::new()
+                };
+                Fichero {
+                    ruta: rel.clone(),
+                    tamano,
+                    mtime,
+                    sha: sha_hex(&datos),
+                    candidatos,
+                }
+            }
+        };
+        for c in &f.candidatos {
+            if let Some(r) = indice.script(c)
+                && !vistos.contains_key(r)
+            {
+                pendientes.push(r.to_owned());
+            }
+        }
+        vistos.insert(rel, f);
+    }
+    Ok(Cierre {
+        ficheros: vistos.into_values().collect(),
+        hasheados,
+    })
+}
+
+/// Huella de la orden, de los globs y del contenido del cierre.
+pub fn huella(orden: &str, incluye: &[String], ficheros: &[Fichero]) -> String {
+    let mut globs: Vec<&str> = incluye.iter().map(String::as_str).collect();
+    globs.sort_unstable();
+    let mut s = format!("orden\0{orden}\0incluye\0{}\0", globs.join("\0"));
+    for f in ficheros {
+        s.push_str(&f.ruta);
+        s.push('\0');
+        s.push_str(&f.sha);
+        s.push('\n');
+    }
+    sha_hex(s.as_bytes())
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn candidatos_resuelven_dp0_barras_y_puntos() {
+        let c = candidatos(
+            r#"call "%~dp0lib\paso.bat" && python -I ../x.py"#,
+            "scripts",
+        );
+        assert!(c.contains(&"scripts/lib/paso.bat".to_owned()), "{c:?}");
+        assert!(c.contains(&"x.py".to_owned()), "{c:?}");
+        // Tokens sin forma de ruta no cuentan.
+        assert!(!c.iter().any(|x| x == "call" || x == "python"), "{c:?}");
+        assert_eq!(normalizar("", "../fuera.sh"), None);
+        assert_eq!(normalizar("", "/abs.sh"), None);
+    }
+
+    #[test]
+    fn script_por_extension() {
+        assert!(es_script("scripts/v.BAT"));
+        assert!(es_script("scripts/verificar"));
+        assert!(!es_script("README.md"));
+    }
+}
diff --git a/crates/tc2/src/confianza/mod.rs b/crates/tc2/src/confianza/mod.rs
new file mode 100644
index 0000000..5482ec1
--- /dev/null
+++ b/crates/tc2/src/confianza/mod.rs
@@ -0,0 +1,336 @@
+//! Confianza local (F1-8 decision 5): la unica interfaz que decide si `tc2`
+//! puede ejecutar una orden de la configuracion del repo.
+//!
+//! `estado(repo, clave, orden, incluye) -> confiado | cambiado | nunca`, con
+//! `clave` la ruta de la clave en la configuracion (`verificacion.comando`) y
+//! la orden leida del arbol de trabajo. Generica por clave: no sabe de hooks
+//! ni de perfiles.
+//!
+//! - **Huella**: sha256 de la orden, de los globs de `confianza_incluye` y
+//!   del contenido de su cierre ([`cierre`]).
+//! - **Almacen fuera del arbol**: `<cache>/taskcodetwo/confianza/<sha>.json`
+//!   (`%LOCALAPPDATA%` en Windows, `$XDG_CACHE_HOME` o `~/.cache` en el
+//!   resto), un fichero por repo con nombre el sha256 de la ruta canonica del
+//!   directorio comun de Git: los worktrees comparten confianza y otro clon
+//!   no. Nada del repo cuenta. Si el almacen se borra o no se lee, todo
+//!   vuelve a `nunca` (falla cerrado).
+//! - **Estados**: `nunca` (sin entrada), `confiado` (misma huella),
+//!   `cambiado` (otra huella). Por contenido: volver al contenido confiado
+//!   vuelve a dar `confiado`.
+
+pub mod cierre;
+
+use std::collections::BTreeMap;
+use std::path::{Path, PathBuf};
+
+use serde::{Deserialize, Serialize};
+
+use crate::config::Config;
+use crate::contrato::{Informe, Resultado};
+use crate::git::{self, Git, Subcomando};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::verificacion as t;
+use crate::instalacion;
+
+pub use cierre::Fichero;
+
+/// Nombre de la orden `tc2 confiar`.
+pub const NOMBRE: &str = "confiar";
+
+/// Claves de la configuracion cuya orden se puede confiar hoy.
+pub const CLAVES: [&str; 1] = ["verificacion.comando"];
+
+/// Version del fichero del almacen.
+const VERSION_ALMACEN: u32 = 1;
+
+/// Estado de confianza de una orden.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Estado {
+    /// Misma huella que la confiada.
+    Confiado,
+    /// Hay confianza para la clave, pero con otra huella.
+    Cambiado,
+    /// Sin confianza para la clave en este repo.
+    Nunca,
+}
+
+impl Estado {
+    /// Nombre en la linea de contrato.
+    pub fn nombre(self) -> &'static str {
+        match self {
+            Self::Confiado => "confiado",
+            Self::Cambiado => "cambiado",
+            Self::Nunca => "nunca",
+        }
+    }
+}
+
+/// Lo que se sabe de una orden tras calcular su huella.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Evaluacion {
+    /// Estado.
+    pub estado: Estado,
+    /// Huella actual.
+    pub huella: String,
+    /// Ficheros del cierre.
+    pub ficheros: usize,
+    /// Bytes leidos para hashear (0 en caliente y sin cambios).
+    pub hasheados: u64,
+}
+
+/// Un repo abierto para la confianza: su arbol de trabajo, el nombre de su
+/// fichero en el almacen y sus ficheros versionados.
+pub struct Repo {
+    /// Raiz del arbol de trabajo (`--show-toplevel`).
+    pub raiz: PathBuf,
+    /// sha256 de la ruta canonica del directorio comun de Git.
+    pub id: String,
+    indice: cierre::Indice,
+}
+
+/// Abre el repo de `dir`.
+pub fn abrir(git: &Git, dir: &Path, op: &str) -> Result<Repo, Informe> {
+    let raiz = git::exigir(
+        git,
+        dir,
+        op,
+        Subcomando::RevParse,
+        &["--path-format=absolute", "--show-toplevel"],
+    )?;
+    let raiz = PathBuf::from(raiz.trim());
+    let comun = git::estado::dir_comun(git, dir, op)?;
+    let comun = std::fs::canonicalize(&comun).unwrap_or(comun);
+    let ls = git::exigir(git, &raiz, op, Subcomando::LsFiles, &["-z"])?;
+    Ok(Repo {
+        raiz,
+        id: cierre::sha_hex(comun.to_string_lossy().as_bytes()),
+        indice: cierre::Indice::de_ls_files(&ls),
+    })
+}
+
+/// Carpeta del almacen de confianza de esta maquina: la subcarpeta
+/// `confianza` de la carpeta `taskcodetwo` de datos locales
+/// (`%LOCALAPPDATA%\taskcodetwo` o `$XDG_CACHE_HOME/taskcodetwo`, la misma
+/// que la cache del lanzador, F1-14).
+pub fn almacen(e: &instalacion::Entorno) -> Option<PathBuf> {
+    e.raices_de_cache().pop().map(|r| r.join("confianza"))
+}
+
+/// La orden de `clave` en `config`, tal como se hashea. En
+/// `verificacion.comando` entra la shell: cambiarla cambia lo que se ejecuta.
+pub fn orden_configurada(config: &Config, clave: &str) -> Option<String> {
+    match clave {
+        "verificacion.comando" => config
+            .verificacion
+            .as_ref()
+            .map(|v| format!("{}\n{}", v.shell, v.comando)),
+        _ => None,
+    }
+}
+
+#[derive(Debug, Default, Serialize, Deserialize)]
+struct Almacen {
+    version: u32,
+    claves: BTreeMap<String, Entrada>,
+}
+
+#[derive(Debug, Clone, Serialize, Deserialize)]
+struct Entrada {
+    huella: String,
+    ficheros: Vec<Fichero>,
+}
+
+fn ruta_almacen(almacen: &Path, repo: &Repo) -> PathBuf {
+    almacen.join(format!("{}.json", repo.id))
+}
+
+/// El almacen del repo; vacio si no existe o no se lee (falla cerrado).
+fn leer_almacen(ruta: &Path) -> Almacen {
+    std::fs::read_to_string(ruta)
+        .ok()
+        .and_then(|s| serde_json::from_str::<Almacen>(&s).ok())
+        .filter(|a| a.version == VERSION_ALMACEN)
+        .unwrap_or_default()
+}
+
+fn escribir_almacen(ruta: &Path, a: &Almacen, op: &str) -> Result<(), Informe> {
+    let error = |e: &dyn std::fmt::Display| {
+        Informe::nuevo(op, Resultado::Error).diagnostico(
+            Mensaje::nuevo(&t::ALMACEN_NO_ESCRITO)
+                .con("ruta", ruta.display().to_string())
+                .con("error", e.to_string()),
+        )
+    };
+    if let Some(padre) = ruta.parent() {
+        std::fs::create_dir_all(padre).map_err(|e| error(&e))?;
+    }
+    let texto = serde_json::to_string_pretty(a).map_err(|e| error(&e))?;
+    let tmp = ruta.with_extension(format!("json.{}.tmp", std::process::id()));
+    std::fs::write(&tmp, texto).map_err(|e| error(&e))?;
+    std::fs::rename(&tmp, ruta).map_err(|e| {
+        let _ = std::fs::remove_file(&tmp);
+        error(&e)
+    })
+}
+
+fn error_de_lectura(op: &str, e: &std::io::Error) -> Informe {
+    Informe::nuevo(op, Resultado::Error)
+        .diagnostico(Mensaje::nuevo(&t::CIERRE_NO_LEIDO).con("error", e.to_string()))
+}
+
+/// Estado de confianza de `orden` (la de `clave`) en `repo`. En caliente
+/// reutiliza el sha de los ficheros con el mismo `(tamano, mtime)`; si sigue
+/// confiada y algun fichero se volvio a leer, refresca la cache.
+pub fn estado(
+    repo: &Repo,
+    almacen: &Path,
+    clave: &str,
+    orden: &str,
+    incluye: &[String],
+    op: &str,
+) -> Result<Evaluacion, Informe> {
+    let ruta = ruta_almacen(almacen, repo);
+    let mut a = leer_almacen(&ruta);
+    let entrada = a.claves.get(clave).cloned();
+    let c = cierre::calcular(
+        &repo.raiz,
+        &repo.indice,
+        orden,
+        incluye,
+        entrada.as_ref().map(|e| e.ficheros.as_slice()),
+    )
+    .map_err(|e| error_de_lectura(op, &e))?;
+    let huella = cierre::huella(orden, incluye, &c.ficheros);
+    let estado = match &entrada {
+        None => Estado::Nunca,
+        Some(e) if e.huella == huella => Estado::Confiado,
+        Some(_) => Estado::Cambiado,
+    };
+    if estado == Estado::Confiado && c.hasheados > 0 {
+        // Solo la cache: misma huella. Si no se puede escribir, la proxima
+        // vez se vuelve a leer, nada mas.
+        a.claves.insert(
+            clave.to_owned(),
+            Entrada {
+                huella: huella.clone(),
+                ficheros: c.ficheros.clone(),
+            },
+        );
+        let _ = escribir_almacen(&ruta, &a, op);
+    }
+    Ok(Evaluacion {
+        estado,
+        huella,
+        ficheros: c.ficheros.len(),
+        hasheados: c.hasheados,
+    })
+}
+
+/// Confia en `orden` tal como esta ahora en el arbol: rehashea todo el
+/// cierre, sin cache, y lo guarda en el almacen.
+pub fn confiar(
+    repo: &Repo,
+    almacen: &Path,
+    clave: &str,
+    orden: &str,
+    incluye: &[String],
+    op: &str,
+) -> Result<Evaluacion, Informe> {
+    let ruta = ruta_almacen(almacen, repo);
+    let mut a = leer_almacen(&ruta);
+    a.version = VERSION_ALMACEN;
+    let c = cierre::calcular(&repo.raiz, &repo.indice, orden, incluye, None)
+        .map_err(|e| error_de_lectura(op, &e))?;
+    let huella = cierre::huella(orden, incluye, &c.ficheros);
+    a.claves.insert(
+        clave.to_owned(),
+        Entrada {
+            huella: huella.clone(),
+            ficheros: c.ficheros.clone(),
+        },
+    );
+    escribir_almacen(&ruta, &a, op)?;
+    Ok(Evaluacion {
+        estado: Estado::Confiado,
+        huella,
+        ficheros: c.ficheros.len(),
+        hasheados: c.hasheados,
+    })
+}
+
+/// `tc2 confiar <clave>` en la carpeta actual: confia en la orden de `clave`
+/// tal como esta en el arbol de trabajo y en lo que ejecuta.
+pub fn orden(args: &[String]) -> Informe {
+    let uso = || {
+        Informe::nuevo(NOMBRE, Resultado::Uso)
+            .diagnostico(Mensaje::nuevo(&t::USO_CONFIAR).con("claves", CLAVES.join(", ")))
+    };
+    let [clave] = args else {
+        return uso().con("motivo", "argumentos");
+    };
+    if clave.starts_with('-') {
+        return uso().con("motivo", "argumentos");
+    }
+    if !CLAVES.contains(&clave.as_str()) {
+        return Informe::nuevo(NOMBRE, Resultado::Uso)
+            .con("motivo", "clave-desconocida")
+            .diagnostico(
+                Mensaje::nuevo(&t::CLAVE_DESCONOCIDA)
+                    .con("clave", clave.as_str())
+                    .con("claves", CLAVES.join(", ")),
+            );
+    }
+    match confiar_en(clave) {
+        Ok(i) | Err(i) => i,
+    }
+}
+
+fn confiar_en(clave: &str) -> Result<Informe, Informe> {
+    let dir = std::env::current_dir().map_err(|e| {
+        Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(
+            Mensaje::nuevo(&crate::i18n::catalogo::cli::CARPETA_ACTUAL).con("error", e.to_string()),
+        )
+    })?;
+    let almacen = almacen_del_proceso(NOMBRE)?;
+    let repo = abrir(&Git::nuevo(), &dir, NOMBRE)?;
+    let config = crate::config::del_arbol(&repo.raiz).map_err(|mut i| {
+        i.comando = NOMBRE.to_owned();
+        i
+    })?;
+    let Some(orden) = orden_configurada(&config, clave) else {
+        return Err(Informe::nuevo(NOMBRE, Resultado::Rechazada)
+            .con("motivo", "sin-orden")
+            .con("clave", clave)
+            .diagnostico(
+                Mensaje::nuevo(&t::SIN_ORDEN)
+                    .con("ruta", crate::config::RUTA)
+                    .con("clave", clave),
+            ));
+    };
+    let ev = confiar(
+        &repo,
+        &almacen,
+        clave,
+        &orden,
+        &config.confianza_incluye,
+        NOMBRE,
+    )?;
+    Ok(Informe::nuevo(NOMBRE, Resultado::Ok)
+        .con("clave", clave)
+        .con("huella", crate::contrato::corto(&ev.huella))
+        .con("ficheros", ev.ficheros.to_string())
+        .con("hasheados", ev.hasheados.to_string()))
+}
+
+/// El almacen del entorno de este proceso, o el informe que dice que hacer.
+pub fn almacen_del_proceso(op: &str) -> Result<PathBuf, Informe> {
+    instalacion::Entorno::del_proceso()
+        .ok()
+        .as_ref()
+        .and_then(almacen)
+        .ok_or_else(|| {
+            Informe::nuevo(op, Resultado::Error)
+                .con("motivo", "sin-almacen")
+                .diagnostico(&t::SIN_ALMACEN)
+        })
+}
diff --git a/crates/tc2/src/config/mod.rs b/crates/tc2/src/config/mod.rs
index 14b0db0..49c8006 100644
--- a/crates/tc2/src/config/mod.rs
+++ b/crates/tc2/src/config/mod.rs
@@ -521,6 +521,21 @@ pub fn de_base(git: &Git, dir: &Path) -> Result<Config, Informe> {
     leer(&texto).map_err(|e| e.informe())
 }
 
+/// Lee la configuracion del arbol de trabajo de `raiz` (F1-8 decision 5:
+/// la orden que se confia y se ejecuta es la del arbol, no la de la base).
+/// `Err` lleva el informe `config rechazada`.
+pub fn del_arbol(raiz: &Path) -> Result<Config, Informe> {
+    match std::fs::read_to_string(raiz.join(RUTA)) {
+        Ok(texto) => leer(&texto).map_err(|e| e.informe()),
+        Err(_) => Err(Invalida::una(
+            Motivo::SinConfig,
+            "",
+            Mensaje::nuevo(&t::NO_HAY_CONFIG_EN_ARBOL).con("ruta", RUTA),
+        )
+        .informe()),
+    }
+}
+
 fn texto_de_base(git: &Git, dir: &Path) -> Paso<String> {
     if git::commit_de(git, dir, NOMBRE, BASE)?.is_none() {
         return Err(Informe::nuevo(NOMBRE, Resultado::Rechazada)
diff --git a/crates/tc2/src/desinstalar/maquina.rs b/crates/tc2/src/desinstalar/maquina.rs
index 357f51a..84e7c2a 100644
--- a/crates/tc2/src/desinstalar/maquina.rs
+++ b/crates/tc2/src/desinstalar/maquina.rs
@@ -269,7 +269,8 @@ pub(super) fn plan(c: &Contexto<'_>, v: &mut Vec<Accion>) -> Result<(), Informe>
         }
     }
 
-    // Cache del lanzador (`taskcodetwo/cli/`, F1-14) y la carpeta raiz.
+    // Cache del lanzador (`taskcodetwo/cli/`, F1-14), el almacen de
+    // confianza y la carpeta raiz.
     for raiz in e.raices_de_cache() {
         let cli = raiz.join("cli");
         if cli.is_dir() {
@@ -302,6 +303,17 @@ pub(super) fn plan(c: &Contexto<'_>, v: &mut Vec<Accion>) -> Result<(), Informe>
                 quitados.insert(cli.clone());
             }
         }
+        // Almacen de confianza local (F5-6, F1-8 decision 5): sin el, todo
+        // vuelve a `nunca` (falla cerrado).
+        let confianza = raiz.join("confianza");
+        if confianza.is_dir() {
+            v.push(Accion::borra(
+                t(&confianza),
+                "confianza",
+                Efecto::Carpeta(confianza.clone()),
+            ));
+            quitados.insert(confianza);
+        }
         if raiz.is_dir() && hijos(&raiz).iter().all(|h| quitados.contains(h)) {
             match autoborrado.and_then(|i| v.get_mut(i)) {
                 Some(Accion {
diff --git a/crates/tc2/src/git/puerto.rs b/crates/tc2/src/git/puerto.rs
index 28067aa..622e481 100644
--- a/crates/tc2/src/git/puerto.rs
+++ b/crates/tc2/src/git/puerto.rs
@@ -38,6 +38,9 @@ pub enum Subcomando {
     ForEachRef,
     /// `git hash-object`
     HashObject,
+    /// `git ls-files` (lectura; los ficheros versionados del cierre de
+    /// confianza, F5-6)
+    LsFiles,
     /// `git ls-remote`
     LsRemote,
     /// `git log` (lectura; el historico de la estimacion, F3-13)
@@ -89,6 +92,7 @@ impl Subcomando {
             Self::Fetch => "fetch",
             Self::ForEachRef => "for-each-ref",
             Self::HashObject => "hash-object",
+            Self::LsFiles => "ls-files",
             Self::LsRemote => "ls-remote",
             Self::Log => "log",
             Self::LsTree => "ls-tree",
diff --git a/crates/tc2/src/i18n/catalogo/cli.rs b/crates/tc2/src/i18n/catalogo/cli.rs
index 53ee3d1..c941eb5 100644
--- a/crates/tc2/src/i18n/catalogo/cli.rs
+++ b/crates/tc2/src/i18n/catalogo/cli.rs
@@ -10,8 +10,8 @@ crate::i18n::textos! {
         en: "tc2: could not write the output: {error}",
     }
     USO {
-        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<motivo>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <carril>] | tc2 plan guardar [--json] TASK-NNN <destino> <fichero> | tc2 approve [--json] TASK-NNN [--pedir-cambios <fichero>] | tc2 approve [--json] --hito <n> | tc2 review [--json] TASK-NNN | tc2 revisoras [--json] --rutas <fichero|-> [opciones] | tc2 hallazgos [--json] (validar --respuesta <fichero> --diff <fichero> --revisora <id> [opciones] | instrucciones) | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
-        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<reason>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <lane>] | tc2 plan guardar [--json] TASK-NNN <destination> <file> | tc2 approve [--json] TASK-NNN [--pedir-cambios <file>] | tc2 approve [--json] --hito <n> | tc2 review [--json] TASK-NNN | tc2 revisoras [--json] --rutas <file|-> [options] | tc2 hallazgos [--json] (validar --respuesta <file> --diff <file> --revisora <id> [options] | instrucciones) | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
+        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<motivo>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <carril>] | tc2 plan guardar [--json] TASK-NNN <destino> <fichero> | tc2 approve [--json] TASK-NNN [--pedir-cambios <fichero>] | tc2 approve [--json] --hito <n> | tc2 review [--json] TASK-NNN | tc2 revisoras [--json] --rutas <fichero|-> [opciones] | tc2 hallazgos [--json] (validar --respuesta <fichero> --diff <fichero> --revisora <id> [opciones] | instrucciones) | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 confiar [--json] <clave> | tc2 verificar [--json] --perfil <perfil> [--hook] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
+        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<reason>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <lane>] | tc2 plan guardar [--json] TASK-NNN <destination> <file> | tc2 approve [--json] TASK-NNN [--pedir-cambios <file>] | tc2 approve [--json] --hito <n> | tc2 review [--json] TASK-NNN | tc2 revisoras [--json] --rutas <file|-> [options] | tc2 hallazgos [--json] (validar --respuesta <file> --diff <file> --revisora <id> [options] | instrucciones) | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 confiar [--json] <key> | tc2 verificar [--json] --perfil <profile> [--hook] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
     }
     FALTA_LA_ORDEN {
         es: "tc2: falta la orden. {uso}",
diff --git a/crates/tc2/src/i18n/catalogo/config.rs b/crates/tc2/src/i18n/catalogo/config.rs
index 996e090..92ea2cb 100644
--- a/crates/tc2/src/i18n/catalogo/config.rs
+++ b/crates/tc2/src/i18n/catalogo/config.rs
@@ -80,6 +80,10 @@ crate::i18n::textos! {
         es: "no hay {ruta} en {base}",
         en: "there is no {ruta} in {base}",
     }
+    NO_HAY_CONFIG_EN_ARBOL {
+        es: "no hay {ruta} en el arbol de trabajo",
+        en: "there is no {ruta} in the working tree",
+    }
 
     // --- Validacion contra la tabla (validar.rs) ---
     ENTERO_EN_RANGO {
diff --git a/crates/tc2/src/i18n/catalogo/mod.rs b/crates/tc2/src/i18n/catalogo/mod.rs
index 5661fb2..17e6252 100644
--- a/crates/tc2/src/i18n/catalogo/mod.rs
+++ b/crates/tc2/src/i18n/catalogo/mod.rs
@@ -24,6 +24,7 @@ pub mod seleccion;
 pub mod siguiente;
 pub mod tablero;
 pub mod toma;
+pub mod verificacion;
 
 use super::Texto;
 
@@ -53,6 +54,7 @@ pub fn todos() -> impl Iterator<Item = &'static Texto> {
         siguiente::TODOS,
         tablero::TODOS,
         toma::TODOS,
+        verificacion::TODOS,
     ]
     .into_iter()
     .flatten()
diff --git a/crates/tc2/src/i18n/catalogo/verificacion.rs b/crates/tc2/src/i18n/catalogo/verificacion.rs
new file mode 100644
index 0000000..fa94c89
--- /dev/null
+++ b/crates/tc2/src/i18n/catalogo/verificacion.rs
@@ -0,0 +1,69 @@
+//! Mensajes de `tc2 confiar` y `tc2 verificar` (F5-6) y de la confianza
+//! local (F1-8 decision 5).
+
+crate::i18n::textos! {
+    USO_CONFIAR {
+        es: "uso: confiar [--json] <clave> (se pueden confiar: {claves})",
+        en: "usage: confiar [--json] <key> (keys that can be trusted: {claves})",
+    }
+    CLAVE_DESCONOCIDA {
+        es: "tc2 no sabe confiar la clave «{clave}»: las que se pueden confiar son {claves}",
+        en: "tc2 cannot trust the key «{clave}»: the keys that can be trusted are {claves}",
+    }
+    SIN_ORDEN {
+        es: "{ruta} no declara {clave}: declarala antes de confiar en ella",
+        en: "{ruta} does not declare {clave}: declare it before trusting it",
+    }
+    SIN_ALMACEN {
+        es: "no se sabe donde guardar la confianza: define LOCALAPPDATA (Windows) o XDG_CACHE_HOME o HOME (resto)",
+        en: "no place to store the trust: set LOCALAPPDATA (Windows) or XDG_CACHE_HOME or HOME (others)",
+    }
+    ALMACEN_NO_ESCRITO {
+        es: "no se pudo escribir la confianza en {ruta}: {error}",
+        en: "could not write the trust to {ruta}: {error}",
+    }
+    CIERRE_NO_LEIDO {
+        es: "no se pudo leer un fichero de lo que ejecuta la orden: {error}",
+        en: "could not read a file of what the command runs: {error}",
+    }
+    USO_VERIFICAR {
+        es: "uso: verificar [--json] --perfil <rapida|impacto|completa|pesada> [--hook]",
+        en: "usage: verificar [--json] --perfil <rapida|impacto|completa|pesada> [--hook]",
+    }
+    SIN_VERIFICACION {
+        es: "{ruta} no declara verificacion: declara verificacion.comando, verificacion.shell y sus perfiles",
+        en: "{ruta} does not declare verificacion: declare verificacion.comando, verificacion.shell and its profiles",
+    }
+    SIN_PERFIL {
+        es: "{ruta} no declara el perfil {perfil}: anadelo en verificacion.perfiles con su timeout",
+        en: "{ruta} does not declare the profile {perfil}: add it under verificacion.perfiles with its timeout",
+    }
+    NUNCA_CONFIADA {
+        es: "la orden de {clave} no esta confiada en este repo y no se ejecuta: revisa la orden y los scripts que lanza y, si te fias, ejecuta `tc2 confiar {clave}`",
+        en: "the command of {clave} is not trusted in this repo and is not run: review the command and the scripts it launches and, if you trust them, run `tc2 confiar {clave}`",
+    }
+    CAMBIADA {
+        es: "la orden de {clave} o un script que ejecuta cambio desde que confiaste en ella y no se ejecuta: revisa el cambio y, si te fias, ejecuta `tc2 confiar {clave}`",
+        en: "the command of {clave} or a script it runs changed since you trusted it and is not run: review the change and, if you trust it, run `tc2 confiar {clave}`",
+    }
+    NO_SE_LANZA {
+        es: "no se pudo lanzar la shell {shell}: {error}; comprueba que esta en el PATH o cambia verificacion.shell",
+        en: "could not launch the shell {shell}: {error}; check that it is on the PATH or change verificacion.shell",
+    }
+    ROJO {
+        es: "la verificacion {perfil} salio en rojo (codigo {codigo}): mira su salida y corrige lo que falle",
+        en: "the {perfil} verification is red (exit code {codigo}): read its output and fix what fails",
+    }
+    TIMEOUT {
+        es: "la verificacion {perfil} agoto su timeout de {timeout} s y cuenta como rojo: acelera la suite o sube verificacion.perfiles.{perfil}.timeout",
+        en: "the {perfil} verification ran out of its {timeout} s timeout and counts as red: speed up the suite or raise verificacion.perfiles.{perfil}.timeout",
+    }
+    HOOK_NO_SE_EJECUTA {
+        es: "tc2 verificar --hook: no se ejecuta la verificacion {perfil}: {motivo}",
+        en: "tc2 verificar --hook: the {perfil} verification is not run: {motivo}",
+    }
+    HOOK_ROJO {
+        es: "tc2 verificar --hook: {motivo}. Ultimas lineas de su salida:",
+        en: "tc2 verificar --hook: {motivo}. Last lines of its output:",
+    }
+}
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index e1a09ec..8b472e1 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -29,6 +29,7 @@ pub mod actualizar_cli;
 pub mod alta;
 pub mod carril;
 pub mod ciclo;
+pub mod confianza;
 pub mod config;
 pub mod contrato;
 pub mod descarga;
@@ -234,6 +235,14 @@ where
             let (json, resto) = con_json(resto);
             orden_estimar(&resto, json, idioma)
         }
+        [orden, resto @ ..] if orden == verificacion::ejecutar::NOMBRE => {
+            let (json, resto) = con_json(resto);
+            verificacion::ejecutar::orden(&resto, json, idioma)
+        }
+        [orden, resto @ ..] if orden == confianza::NOMBRE => {
+            let (json, resto) = con_json(resto);
+            confianza::orden(&resto).salida_en(json, idioma)
+        }
         [orden, resto @ ..] if orden == identidad::NOMBRE => {
             let (json, resto) = con_json(resto);
             orden_identidad(&resto).salida_en(json, idioma)
diff --git a/crates/tc2/src/verificacion/ejecutar.rs b/crates/tc2/src/verificacion/ejecutar.rs
new file mode 100644
index 0000000..81b11ff
--- /dev/null
+++ b/crates/tc2/src/verificacion/ejecutar.rs
@@ -0,0 +1,400 @@
+//! `tc2 verificar --perfil <p> [--hook]` (F5-6): lanza el comando de
+//! verificacion del proyecto, solo si su orden esta confiada.
+//!
+//! El nucleo es un comando determinista (F1-11 decision 10): lee
+//! `verificacion` de `.taskcode/config.yml` **del arbol de trabajo**
+//! (F1-8 decision 5), pregunta a [`crate::confianza::estado`] por la clave
+//! `verificacion.comando` y, si esta confiada, lanza `verificacion.shell` con
+//! `<comando> --perfil <p>`, `TC2_PERFIL=<p>` y el `timeout` del perfil.
+//! Codigos (F1-11 decision 8): cualquier codigo distinto de 0 es rojo, y un
+//! `timeout` agotado tambien, salvo en `rapida`, donde es «excedido».
+//!
+//! `--hook` es la misma orden colgada del evento `Stop` de Claude Code: la
+//! decision es la misma y solo cambia como se devuelve. Con
+//! `stop_hook_active: true` en el JSON de stdin sale con 0 sin ejecutar
+//! nada; si no se ejecuta (sin confianza, sin el perfil, configuracion
+//! invalida) sale con 0 y una linea en stderr que dice que hacer, nunca
+//! bloquea por eso; en rojo sale con 2 y un resumen corto en stderr, que
+//! Claude Code le pasa al agente.
+
+use std::io::{IsTerminal, Read};
+use std::path::{Path, PathBuf};
+use std::process::{Command, Stdio};
+use std::time::{Duration, Instant};
+
+use crate::Salida;
+use crate::confianza::{self, Estado};
+use crate::config::{PerfilVerificacion, Verificacion};
+use crate::contrato::{Informe, Resultado};
+use crate::git::Git;
+use crate::i18n::catalogo::verificacion as t;
+use crate::i18n::{Idioma, Mensaje};
+
+/// Nombre de la orden.
+pub const NOMBRE: &str = "verificar";
+
+/// Clave de la configuracion que se confia y se ejecuta.
+pub const CLAVE: &str = "verificacion.comando";
+
+/// Perfiles de F1-7 (decision 4).
+pub const PERFILES: [&str; 4] = ["rapida", "impacto", "completa", "pesada"];
+
+/// Lineas de la salida del comando que `--hook` pasa al agente en rojo.
+const LINEAS_RESUMEN: usize = 15;
+
+/// Tope de lo que se lee de stdin con `--hook`.
+const LIMITE_STDIN: u64 = 1 << 20;
+
+/// Cada cuanto se mira si el comando termino.
+const SONDEO: Duration = Duration::from_millis(10);
+
+/// `tc2 verificar` con los argumentos ya sin `--json`.
+pub fn orden(args: &[String], json: bool, idioma: Idioma) -> Salida {
+    let mut perfil = None;
+    let mut hook = false;
+    let mut it = args.iter();
+    while let Some(a) = it.next() {
+        match a.as_str() {
+            "--perfil" => match it.next() {
+                Some(p) if perfil.is_none() => perfil = Some(p.clone()),
+                _ => return uso().salida_en(json, idioma),
+            },
+            "--hook" if !hook => hook = true,
+            _ => return uso().salida_en(json, idioma),
+        }
+    }
+    let Some(perfil) = perfil.filter(|p| PERFILES.contains(&p.as_str())) else {
+        return uso().salida_en(json, idioma);
+    };
+    if hook && stop_hook_activo() {
+        return Informe::nuevo(NOMBRE, Resultado::SinCambios)
+            .con("motivo", "stop-hook-activo")
+            .con("perfil", perfil.as_str())
+            .salida_en(json, idioma);
+    }
+    let captura = hook.then(fichero_de_captura);
+    let informe = match std::env::current_dir() {
+        Ok(dir) => {
+            let almacen = confianza::almacen_del_proceso(NOMBRE);
+            match almacen {
+                Ok(a) => verificar(&Git::nuevo(), &dir, &a, &perfil, captura.as_deref()),
+                Err(i) => i,
+            }
+        }
+        Err(e) => Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(
+            Mensaje::nuevo(&crate::i18n::catalogo::cli::CARPETA_ACTUAL).con("error", e.to_string()),
+        ),
+    };
+    let salida = if hook {
+        como_hook(&informe, &perfil, captura.as_deref(), json, idioma)
+    } else {
+        informe.salida_en(json, idioma)
+    };
+    if let Some(c) = captura {
+        let _ = std::fs::remove_file(c);
+    }
+    salida
+}
+
+fn uso() -> Informe {
+    Informe::nuevo(NOMBRE, Resultado::Uso)
+        .con("motivo", "argumentos")
+        .diagnostico(&t::USO_VERIFICAR)
+}
+
+/// `stop_hook_active` del JSON de stdin del evento `Stop`. Sin JSON (stdin
+/// en una terminal, vacio o mal formado) no se sabe y se sigue: el tope de
+/// continuaciones de Claude Code corta un bucle.
+fn stop_hook_activo() -> bool {
+    let stdin = std::io::stdin();
+    if stdin.is_terminal() {
+        return false;
+    }
+    let mut entrada = String::new();
+    let _ = stdin.lock().take(LIMITE_STDIN).read_to_string(&mut entrada);
+    serde_json::from_str::<serde_json::Value>(&entrada)
+        .ok()
+        .and_then(|v| {
+            v.get("stop_hook_active")
+                .and_then(serde_json::Value::as_bool)
+        })
+        .unwrap_or(false)
+}
+
+fn fichero_de_captura() -> PathBuf {
+    std::env::temp_dir().join(format!("tc2-verificar-{}.log", std::process::id()))
+}
+
+/// La decision y la ejecucion, comunes a la orden y al hook. Con `captura`,
+/// la salida del comando va a ese fichero; sin ella, a stderr.
+pub fn verificar(
+    git: &Git,
+    dir: &Path,
+    almacen: &Path,
+    perfil: &str,
+    captura: Option<&Path>,
+) -> Informe {
+    match decidir_y_lanzar(git, dir, almacen, perfil, captura) {
+        Ok(i) | Err(i) => i,
+    }
+}
+
+fn decidir_y_lanzar(
+    git: &Git,
+    dir: &Path,
+    almacen: &Path,
+    perfil: &str,
+    captura: Option<&Path>,
+) -> Result<Informe, Informe> {
+    let repo = confianza::abrir(git, dir, NOMBRE)?;
+    let config = crate::config::del_arbol(&repo.raiz).map_err(|mut i| {
+        i.comando = NOMBRE.to_owned();
+        i
+    })?;
+    let rechazo = |motivo: &str, m: Mensaje| {
+        Informe::nuevo(NOMBRE, Resultado::Rechazada)
+            .con("motivo", motivo)
+            .con("perfil", perfil)
+            .diagnostico(m)
+    };
+    let Some(v) = config.verificacion.as_ref() else {
+        return Err(rechazo(
+            "sin-verificacion",
+            Mensaje::nuevo(&t::SIN_VERIFICACION).con("ruta", crate::config::RUTA),
+        ));
+    };
+    let Some(p) = perfil_de(v, perfil) else {
+        return Err(rechazo(
+            "sin-perfil",
+            Mensaje::nuevo(&t::SIN_PERFIL)
+                .con("ruta", crate::config::RUTA)
+                .con("perfil", perfil),
+        ));
+    };
+    let orden = confianza::orden_configurada(&config, CLAVE).unwrap_or_default();
+    let ev = confianza::estado(
+        &repo,
+        almacen,
+        CLAVE,
+        &orden,
+        &config.confianza_incluye,
+        NOMBRE,
+    )?;
+    let base = |r: Resultado| {
+        Informe::nuevo(NOMBRE, r)
+            .con("perfil", perfil)
+            .con("clave", CLAVE)
+            .con("confianza", ev.estado.nombre())
+            .con("hasheados", ev.hasheados.to_string())
+    };
+    match ev.estado {
+        Estado::Confiado => {}
+        Estado::Nunca | Estado::Cambiado => {
+            let que = if ev.estado == Estado::Nunca {
+                &t::NUNCA_CONFIADA
+            } else {
+                &t::CAMBIADA
+            };
+            return Err(base(Resultado::Rechazada)
+                .con("motivo", "sin-confianza")
+                .diagnostico(Mensaje::nuevo(que).con("clave", CLAVE)));
+        }
+    }
+    let inicio = Instant::now();
+    let fin = lanzar(&repo.raiz, v, perfil, p.timeout, captura).map_err(|e| {
+        base(Resultado::Error)
+            .con("motivo", "no-se-lanza")
+            .diagnostico(
+                Mensaje::nuevo(&t::NO_SE_LANZA)
+                    .con("shell", v.shell.as_str())
+                    .con("error", e.to_string()),
+            )
+    })?;
+    let segundos = inicio.elapsed().as_secs().to_string();
+    Ok(match fin {
+        Fin::Codigo(0) => base(Resultado::Ok)
+            .con("resultado", "verde")
+            .con("segundos", segundos),
+        // F1-11 decision 8: en la rapida, agotar el presupuesto no es rojo.
+        Fin::Timeout if perfil == "rapida" => base(Resultado::Ok)
+            .con("resultado", "excedido")
+            .con("segundos", segundos),
+        Fin::Timeout => base(Resultado::Rechazada)
+            .con("motivo", "timeout")
+            .con("resultado", "rojo")
+            .con("segundos", segundos)
+            .diagnostico(
+                Mensaje::nuevo(&t::TIMEOUT)
+                    .con("perfil", perfil)
+                    .con("timeout", p.timeout.to_string()),
+            ),
+        Fin::Codigo(c) => base(Resultado::Rechazada)
+            .con("motivo", "rojo")
+            .con("resultado", "rojo")
+            .con("codigo", c.to_string())
+            .con("segundos", segundos)
+            .diagnostico(
+                Mensaje::nuevo(&t::ROJO)
+                    .con("perfil", perfil)
+                    .con("codigo", c.to_string()),
+            ),
+    })
+}
+
+fn perfil_de<'a>(v: &'a Verificacion, perfil: &str) -> Option<&'a PerfilVerificacion> {
+    match perfil {
+        "rapida" => v.perfiles.rapida.as_ref(),
+        "impacto" => v.perfiles.impacto.as_ref(),
+        "completa" => Some(&v.perfiles.completa),
+        "pesada" => v.perfiles.pesada.as_ref(),
+        _ => None,
+    }
+}
+
+/// Como termino el comando.
+enum Fin {
+    /// Con este codigo (`-1` si murio por una senal).
+    Codigo(i32),
+    /// Se agoto el `timeout` y se mato.
+    Timeout,
+}
+
+/// La linea de la shell: `<shell> <opciones> "<comando> --perfil <p>"`.
+fn orden_de_shell(shell: &str, linea: &str) -> Command {
+    match shell {
+        "cmd" => {
+            let mut c = Command::new("cmd");
+            #[cfg(windows)]
+            {
+                use std::os::windows::process::CommandExt;
+                c.raw_arg(format!("/d /s /c \"{linea}\""));
+            }
+            #[cfg(not(windows))]
+            c.args(["/d", "/s", "/c", linea]);
+            c
+        }
+        "pwsh" => {
+            let mut c = Command::new("pwsh");
+            c.args(["-NoProfile", "-NonInteractive", "-Command", linea]);
+            c
+        }
+        otra => {
+            let mut c = Command::new(otra);
+            c.args(["-c", linea]);
+            c
+        }
+    }
+}
+
+fn lanzar(
+    raiz: &Path,
+    v: &Verificacion,
+    perfil: &str,
+    timeout: u32,
+    captura: Option<&Path>,
+) -> std::io::Result<Fin> {
+    let linea = format!("{} --perfil {perfil}", v.comando);
+    let mut c = orden_de_shell(&v.shell, &linea);
+    c.current_dir(raiz)
+        .env("TC2_PERFIL", perfil)
+        .stdin(Stdio::null());
+    match captura {
+        Some(ruta) => {
+            let f = std::fs::File::create(ruta)?;
+            c.stdout(f.try_clone()?).stderr(f);
+        }
+        // La salida del comando no se mezcla con la linea de contrato.
+        None => {
+            c.stdout(std::io::stderr()).stderr(Stdio::inherit());
+        }
+    }
+    #[cfg(unix)]
+    {
+        use std::os::unix::process::CommandExt;
+        c.process_group(0);
+    }
+    let mut hijo = c.spawn()?;
+    let limite = Instant::now() + Duration::from_secs(u64::from(timeout));
+    loop {
+        if let Some(s) = hijo.try_wait()? {
+            return Ok(Fin::Codigo(s.code().unwrap_or(-1)));
+        }
+        if Instant::now() >= limite {
+            matar_arbol(hijo.id());
+            let _ = hijo.kill();
+            let _ = hijo.wait();
+            return Ok(Fin::Timeout);
+        }
+        std::thread::sleep(SONDEO);
+    }
+}
+
+/// Mata el comando y lo que haya lanzado (la shell sola dejaria vivos a sus
+/// hijos).
+fn matar_arbol(pid: u32) {
+    let pid = pid.to_string();
+    #[cfg(windows)]
+    let mut c = {
+        let mut c = Command::new("taskkill");
+        c.args(["/T", "/F", "/PID", &pid]);
+        c
+    };
+    #[cfg(not(windows))]
+    let mut c = {
+        let mut c = Command::new("kill");
+        c.args(["-KILL", "--", &format!("-{pid}")]);
+        c
+    };
+    let _ = c
+        .stdin(Stdio::null())
+        .stdout(Stdio::null())
+        .stderr(Stdio::null())
+        .status();
+}
+
+/// Si el informe es un rojo de la verificacion (y no un «no se ejecuta»).
+fn es_rojo(i: &Informe) -> bool {
+    matches!(i.valor("motivo"), Some("rojo" | "timeout"))
+}
+
+/// El mismo informe, devuelto como lo espera el evento `Stop`: 2 en rojo
+/// (bloquea y el resumen va al agente), 0 en todo lo demas.
+fn como_hook(
+    i: &Informe,
+    perfil: &str,
+    captura: Option<&Path>,
+    json: bool,
+    idioma: Idioma,
+) -> Salida {
+    let mut s = i.salida_en(json, idioma);
+    if i.resultado.es_exito() {
+        return s;
+    }
+    let motivo = i.diagnostico_en(idioma).unwrap_or_default();
+    if es_rojo(i) {
+        let mut resumen = Mensaje::nuevo(&t::HOOK_ROJO)
+            .con("motivo", motivo)
+            .texto(idioma);
+        resumen.push('\n');
+        if let Some(texto) = captura.and_then(|c| std::fs::read(c).ok()) {
+            let texto = String::from_utf8_lossy(&texto);
+            let lineas: Vec<&str> = texto.lines().collect();
+            for l in &lineas[lineas.len().saturating_sub(LINEAS_RESUMEN)..] {
+                resumen.push_str(l);
+                resumen.push('\n');
+            }
+        }
+        s.codigo = 2;
+        s.stderr = resumen;
+    } else {
+        s.codigo = 0;
+        s.stderr = format!(
+            "{}\n",
+            Mensaje::nuevo(&t::HOOK_NO_SE_EJECUTA)
+                .con("perfil", perfil)
+                .con("motivo", motivo)
+                .texto(idioma)
+        );
+    }
+    s
+}
diff --git a/crates/tc2/src/verificacion/mod.rs b/crates/tc2/src/verificacion/mod.rs
index d588027..52ff5bb 100644
--- a/crates/tc2/src/verificacion/mod.rs
+++ b/crates/tc2/src/verificacion/mod.rs
@@ -3,5 +3,8 @@
 //! - [`informe`]: el informe JSON de verificacion (F5-4) que escribe el
 //!   comando del proyecto en `TC2_INFORME` y la CI sube como artefacto; su
 //!   lector y la seccion con la que el revisor IA lo cita.
+//! - [`ejecutar`]: `tc2 verificar --perfil <p> [--hook]` (F5-6), que lanza
+//!   el comando del proyecto solo con su orden confiada (F1-8 decision 5).
 
+pub mod ejecutar;
 pub mod informe;
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/01-en-diseno/TASK-046/tarea.md                                                     | 47 -----------------------------------------------
 tareas/{01-en-diseno => 02-en-curso}/TASK-046/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-046/planificacion/plan-final.md                 |  0
 tareas/02-en-curso/TASK-046/tarea.md                                                      | 88 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 4 files changed, 88 insertions(+), 47 deletions(-)
````

## Tests del diff (2 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/desinstalar.rs" ":(literal)crates/tc2/tests/verificar.rs"
````

````
crates/tc2/tests/desinstalar.rs |   9 ++-
 crates/tc2/tests/verificar.rs   | 554 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 2 files changed, 562 insertions(+), 1 deletion(-)
````
