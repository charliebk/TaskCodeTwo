# Peticion de revision — TASK-143 (ronda 1)

- Tarea: TASK-143 — Auditoria 4D: A5 seleccion, inventario, recursos y doctor (6 IMPORTANTE)
- Rama revisada: fix/task-143-auditoria-4d-a5-seleccion-inventario-rec
- Rama base: develop
- Commit revisado (HEAD): 38bd5b5a2a0260a0046096e4c1764798472d8dea
- Fecha: 2026-11-25
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-143 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-143/tarea.md
- tareas/03-en-revision/TASK-143/planificacion/
- crates/tc2/src/inventario.rs
- crates/tc2/src/recursos.rs
- crates/tc2/src/seleccion/catalogo.rs
- crates/tc2/src/seleccion/mod.rs
- crates/tc2/src/seleccion/orden.rs
- crates/tc2/tests/comun/lanzador.rs
- crates/tc2/tests/comun/mod.rs
- crates/tc2/tests/doctor.rs
- crates/tc2/tests/recursos.rs
- crates/tc2/tests/revisoras.rs

## Commits a revisar (git log develop..HEAD)

````
38bd5b5 docs(TASK-143): resultado, criterios y autorrevision
26dbb31 fix(TASK-143): A5-09 doctor_no_escribe_nada usa doctor_contra_en
7862b68 fix(TASK-143): A5-08 salida acotada de doctor en un ayudante
495aee4 fix(TASK-143): A5-06 las_reglas_se_leen_de_la_tabla exige Ok
4b93705 fix(TASK-143): A5-03 revisora de dominio no instalada da S11
91a6e16 fix(TASK-143): A5-02 carpeta del plugin con la regla del scope
9388109 fix(TASK-143): A5-01 una sola lectura de skills de una carpeta
7b534bb chore(TASK-143): tarea en curso
````

## Diff de tu dominio (10 fichero(s) de 10; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/inventario.rs b/crates/tc2/src/inventario.rs
index 4f4fa40..0d3e659 100644
--- a/crates/tc2/src/inventario.rs
+++ b/crates/tc2/src/inventario.rs
@@ -139,6 +139,20 @@ pub struct Plugin {
     pub clave: String,
     /// Version instalada, si se sabe.
     pub version: Option<String>,
+    /// Carpeta de la instalacion que se eligio (la de `installPath` que
+    /// vale para el `scope` de este proyecto); `None` en los sincronizados
+    /// de claude.ai. No entra en [`Inventario::huella`]: depende de la
+    /// maquina.
+    pub carpeta: Option<PathBuf>,
+}
+
+/// De donde se carga un plugin.
+#[derive(Clone, Copy, PartialEq, Eq)]
+enum Fuente {
+    /// De `installed_plugins.json`.
+    Instalado,
+    /// Sincronizado desde claude.ai: cede ante un nombre ya cargado.
+    Sincronizado,
 }
 
 /// Lo instalado y habilitado, ordenado y sin repetir.
@@ -373,9 +387,9 @@ fn cabecera(p: &Path) -> std::io::Result<String> {
 /// Hijos de `dir` ordenados (vacio si no existe). Si la carpeta o una de
 /// sus entradas no se puede leer (p. ej. sin permiso), una incidencia
 /// `no-se-lee` de `id` en `ilegibles`: nunca cuenta como vacia sin decirlo.
-fn hijos(dir: &Path, id: &str, a: &mut Acumulado) -> Vec<PathBuf> {
+fn hijos(dir: &Path, id: &str, ilegibles: &mut Vec<Incidencia>) -> Vec<PathBuf> {
     let mut no_se_lee = || {
-        a.ilegibles.push(Incidencia {
+        ilegibles.push(Incidencia {
             id: id.to_owned(),
             motivo: "no-se-lee",
             ruta: t(dir),
@@ -404,13 +418,13 @@ fn nombre_utf8(
     n: Option<&std::ffi::OsStr>,
     prefijo: &str,
     ruta: &Path,
-    a: &mut Acumulado,
+    ilegibles: &mut Vec<Incidencia>,
 ) -> Option<String> {
     let n = n?;
     if let Some(s) = n.to_str() {
         return Some(s.to_owned());
     }
-    a.ilegibles.push(Incidencia {
+    ilegibles.push(Incidencia {
         id: format!("{prefijo}{}", n.to_string_lossy()),
         motivo: "nombre-no-utf8",
         ruta: t(ruta),
@@ -418,58 +432,109 @@ fn nombre_utf8(
     None
 }
 
-/// Lee las skills de `dir` (`<n>/SKILL.md`) y los agentes de `agentes`
-/// (`<n>.md`) con el `prefijo` y el `origen` dados.
-fn leer_carpetas(skills: &Path, agentes: &Path, prefijo: &str, origen: &Origen, a: &mut Acumulado) {
-    let mut ficheros: Vec<(Tipo, PathBuf, String)> = Vec::new();
-    for d in hijos(skills, &format!("{prefijo}skills"), a) {
+/// El id (sin prefijo) de una skill o un agente: el `name:` del frontmatter
+/// de su `cabecera` o, si no lo dice, el nombre de su `carpeta` (o fichero).
+pub(crate) fn nombre_de(cabecera: &str, carpeta: &str) -> String {
+    nombre_del_frontmatter(cabecera).unwrap_or_else(|| carpeta.to_owned())
+}
+
+/// Una skill o un agente leido de una carpeta: su entrada, el fichero y la
+/// cabecera que se leyo (como mucho [`MAX_CABECERA`] bytes).
+pub(crate) struct Leida {
+    /// La entrada del inventario.
+    pub(crate) entrada: Entrada,
+    /// `SKILL.md` o `<n>.md`.
+    pub(crate) ruta: PathBuf,
+    /// Su cabecera.
+    pub(crate) cabecera: String,
+}
+
+/// Lee la cabecera de `f` (de `tipo`, con nombre de carpeta o fichero
+/// `carpeta`) y calcula su id: `no-se-lee` en `ilegibles` si no se lee y
+/// `nombre-distinto` en `avisos` si el `name:` no coincide con `carpeta`.
+fn leer_una(
+    tipo: Tipo,
+    f: PathBuf,
+    carpeta: &str,
+    prefijo: &str,
+    origen: &Origen,
+    ilegibles: &mut Vec<Incidencia>,
+    avisos: &mut Vec<Incidencia>,
+) -> Option<Leida> {
+    let Ok(cabecera) = cabecera(&f) else {
+        ilegibles.push(Incidencia {
+            id: format!("{prefijo}{carpeta}"),
+            motivo: "no-se-lee",
+            ruta: t(&f),
+        });
+        return None;
+    };
+    let id = nombre_de(&cabecera, carpeta);
+    if id != carpeta {
+        avisos.push(Incidencia {
+            id: format!("{prefijo}{id}"),
+            motivo: "nombre-distinto",
+            ruta: t(&f),
+        });
+    }
+    Some(Leida {
+        entrada: Entrada {
+            tipo,
+            id: format!("{prefijo}{id}"),
+            origen: origen.clone(),
+        },
+        ruta: f,
+        cabecera,
+    })
+}
+
+/// Las skills de `dir` (`<n>/SKILL.md`, ordenadas) con el `prefijo` de ids
+/// (`plugin:` o vacio) y su `origen`. Lo que no se puede leer va a
+/// `ilegibles` y un `name:` distinto de su carpeta a `avisos` (C8): es la
+/// unica lectura de skills de una carpeta (la usa tambien el catalogo).
+pub(crate) fn skills_de(
+    dir: &Path,
+    prefijo: &str,
+    origen: &Origen,
+    ilegibles: &mut Vec<Incidencia>,
+    avisos: &mut Vec<Incidencia>,
+) -> Vec<Leida> {
+    let mut v = Vec::new();
+    for d in hijos(dir, &format!("{prefijo}skills"), ilegibles) {
         let f = d.join("SKILL.md");
         if d.is_dir()
             && f.is_file()
-            && let Some(n) = nombre_utf8(d.file_name(), prefijo, &d, a)
+            && let Some(n) = nombre_utf8(d.file_name(), prefijo, &d, ilegibles)
+            && let Some(l) = leer_una(Tipo::Skill, f, &n, prefijo, origen, ilegibles, avisos)
         {
-            ficheros.push((Tipo::Skill, f, n));
+            v.push(l);
         }
     }
-    for f in hijos(agentes, &format!("{prefijo}agents"), a) {
+    v
+}
+
+/// Lee las skills de `dir` (`<n>/SKILL.md`) y los agentes de `agentes`
+/// (`<n>.md`) con el `prefijo` y el `origen` dados.
+fn leer_carpetas(skills: &Path, agentes: &Path, prefijo: &str, origen: &Origen, a: &mut Acumulado) {
+    let leidas = skills_de(skills, prefijo, origen, &mut a.ilegibles, &mut a.avisos);
+    a.entradas.extend(leidas.into_iter().map(|l| l.entrada));
+    for f in hijos(agentes, &format!("{prefijo}agents"), &mut a.ilegibles) {
         if f.is_file()
             && f.extension().is_some_and(|e| e.eq_ignore_ascii_case("md"))
-            && let Some(n) = nombre_utf8(f.file_stem(), prefijo, &f, a)
+            && let Some(n) = nombre_utf8(f.file_stem(), prefijo, &f, &mut a.ilegibles)
+            && let Some(l) = leer_una(
+                Tipo::Agente,
+                f,
+                &n,
+                prefijo,
+                origen,
+                &mut a.ilegibles,
+                &mut a.avisos,
+            )
         {
-            ficheros.push((Tipo::Agente, f, n));
+            a.entradas.push(l.entrada);
         }
     }
-    for (tipo, f, carpeta) in ficheros {
-        let nombre = match cabecera(&f) {
-            Ok(c) => nombre_del_frontmatter(&c),
-            Err(_) => {
-                a.ilegibles.push(Incidencia {
-                    id: format!("{prefijo}{carpeta}"),
-                    motivo: "no-se-lee",
-                    ruta: t(&f),
-                });
-                continue;
-            }
-        };
-        let id = match nombre {
-            Some(n) => {
-                if n != carpeta {
-                    a.avisos.push(Incidencia {
-                        id: format!("{prefijo}{n}"),
-                        motivo: "nombre-distinto",
-                        ruta: t(&f),
-                    });
-                }
-                n
-            }
-            None => carpeta,
-        };
-        a.entradas.push(Entrada {
-            tipo,
-            id: format!("{prefijo}{id}"),
-            origen: origen.clone(),
-        });
-    }
 }
 
 /// Las claves de `mcpServers` de un JSON.
@@ -524,15 +589,17 @@ fn ajustes(dir_claude: &Path, raiz: &Path, a: &mut Acumulado) -> Ajustes {
 /// Carga el plugin de `dir` con clave `clave` (`nombre@origen`) si esta
 /// habilitado: `enabledPlugins` o, sin entrada, `defaultEnabled` de su
 /// manifiesto. Su nombre (prefijo de los ids) es el `name` del manifiesto o
-/// `por_defecto`. `omitir` descarta un nombre ya cargado (los sincronizados
-/// ceden ante cualquier otro origen). Devuelve si lo cargo.
+/// `por_defecto`. Un [`Fuente::Sincronizado`] cede ante un nombre ya
+/// cargado (de cualquier otro origen) y no guarda carpeta; uno
+/// [`Fuente::Instalado`] guarda `dir` en [`Plugin::carpeta`]. Devuelve si lo
+/// cargo.
 fn cargar_plugin(
     dir: &Path,
     clave: &str,
     por_defecto: &str,
     version: Option<&str>,
     ajustes: &Ajustes,
-    omitir: bool,
+    fuente: Fuente,
     a: &mut Acumulado,
 ) -> bool {
     let manifiesto = json_de(&dir.join(".claude-plugin").join("plugin.json"), a, clave);
@@ -556,7 +623,7 @@ fn cargar_plugin(
         .filter(|n| !n.is_empty())
         .unwrap_or(por_defecto)
         .to_owned();
-    if omitir && a.plugins.iter().any(|p| p.nombre == nombre) {
+    if fuente == Fuente::Sincronizado && a.plugins.iter().any(|p| p.nombre == nombre) {
         return false;
     }
     let version = version
@@ -585,6 +652,7 @@ fn cargar_plugin(
         nombre,
         clave: clave.to_owned(),
         version,
+        carpeta: (fuente == Fuente::Instalado).then(|| dir.to_path_buf()),
     });
     true
 }
@@ -712,7 +780,10 @@ fn leer_sincronizados(dir_claude: &Path, ajustes: &Ajustes, a: &mut Acumulado) {
     }
     let raiz = dir_claude.join("plugins").join("synced");
     let mut cuentas: Vec<(Option<i64>, PathBuf, Value)> = Vec::new();
-    for c in hijos(&raiz, "synced", a).into_iter().filter(|c| c.is_dir()) {
+    for c in hijos(&raiz, "synced", &mut a.ilegibles)
+        .into_iter()
+        .filter(|c| c.is_dir())
+    {
         let id = format!(
             "synced/{}",
             c.file_name()
@@ -792,7 +863,7 @@ fn leer_sincronizados(dir_claude: &Path, ajustes: &Ajustes, a: &mut Acumulado) {
             nombre,
             version,
             ajustes,
-            true,
+            Fuente::Sincronizado,
             a,
         );
     }
@@ -902,7 +973,7 @@ fn leer_plugins(dir_claude: &Path, raiz: &Path, ajustes: &Ajustes, a: &mut Acumu
             clave.split('@').next().unwrap_or(clave),
             x.get("version").and_then(Value::as_str),
             ajustes,
-            false,
+            Fuente::Instalado,
             a,
         );
     }
@@ -1135,6 +1206,7 @@ mod tests {
                 nombre: "bueno".to_owned(),
                 clave: "bueno@mk".to_owned(),
                 version: Some("1.0.0".to_owned()),
+                carpeta: Some(bueno.clone()),
             }]
         );
         assert!(l.mcp.contains("engram"));
@@ -1383,6 +1455,87 @@ mod tests {
         }
     }
 
+    /// A5-02 (TASK-143): la carpeta del plugin habilitado sale de una sola
+    /// regla, la del `scope` del proyecto. Con la instalacion de otro
+    /// proyecto delante de la de usuario, el inventario guarda la de usuario
+    /// y el catalogo lee las revisoras de esa, sin `version` y con la misma.
+    #[test]
+    fn la_carpeta_del_plugin_es_la_del_scope_del_proyecto() {
+        let m = Tmp::nueva("a5-02");
+        let claude = m.0.join("claude");
+        let raiz = m.0.join("proyecto");
+        let a = m.0.join("cache").join("a");
+        let b = m.0.join("cache").join("b");
+        let rev = |n: &str| format!("---\nname: {n}\n---\n```yaml\nrol: revisor\n```\n");
+        m.escribir("cache/a/skills/de-a/SKILL.md", &rev("de-a"));
+        m.escribir("cache/b/skills/de-b/SKILL.md", &rev("de-b"));
+        for version in [None, Some("1.0.0")] {
+            let mut otro = serde_json::json!({
+                "scope": "project", "projectPath": m.0.join("otro"), "installPath": a
+            });
+            let mut usuario = serde_json::json!({"scope": "user", "installPath": b});
+            if let Some(v) = version {
+                otro["version"] = v.into();
+                usuario["version"] = v.into();
+            }
+            let instalados =
+                serde_json::json!({"version": 2, "plugins": {"p@mk": [otro, usuario]}});
+            m.escribir(
+                "claude/plugins/installed_plugins.json",
+                &instalados.to_string(),
+            );
+            let l = leer(Some(&claude), &raiz);
+            let carpetas: Vec<_> = l.inventario.plugins().iter().map(|p| &p.carpeta).collect();
+            assert_eq!(carpetas, [&Some(b.clone())], "{version:?}");
+            let ids: Vec<String> = crate::seleccion::catalogo::de_instalados(&l.inventario)
+                .into_iter()
+                .map(|r| r.id)
+                .collect();
+            assert_eq!(ids, ["p:de-b"], "{version:?}");
+        }
+    }
+
+    /// A5-01 (TASK-143): `skills_de` es la unica lectura de skills de una
+    /// carpeta. Una carpeta `skills` que no se lista deja `no-se-lee` con el
+    /// prefijo del plugin, tambien por el catalogo (C8); una legible da las
+    /// mismas entradas y avisos que [`leer`].
+    #[test]
+    fn skills_de_una_carpeta_deja_incidencias() {
+        let m = Tmp::nueva("a5-01");
+        m.escribir("p/skills", "no soy una carpeta");
+        let roto = m.0.join("p").join("skills");
+        let o = Origen::Plugin {
+            nombre: "p".to_owned(),
+            version: None,
+        };
+        let esperada = vec![Incidencia {
+            id: "p:skills".to_owned(),
+            motivo: "no-se-lee",
+            ruta: t(&roto),
+        }];
+        let (mut il, mut av) = (Vec::new(), Vec::new());
+        assert!(skills_de(&roto, "p:", &o, &mut il, &mut av).is_empty());
+        assert_eq!((&il, &av), (&esperada, &Vec::new()));
+        let c = crate::seleccion::catalogo::de_carpeta(&roto, "p:", &o);
+        assert_eq!((c.ilegibles, c.entradas), (esperada, Vec::new()));
+        // Legible: lo mismo que el inventario del proyecto.
+        m.escribir("q/.claude/skills/a/SKILL.md", &skill("a"));
+        m.escribir("q/.claude/skills/b/SKILL.md", &skill("otro"));
+        let (mut il, mut av) = (Vec::new(), Vec::new());
+        let dir = m.0.join("q").join(".claude").join("skills");
+        let v = skills_de(&dir, "", &Origen::Proyecto, &mut il, &mut av);
+        let l = leer(None, &m.0.join("q"));
+        let skills: Vec<&Entrada> = l
+            .inventario
+            .entradas()
+            .iter()
+            .filter(|e| e.tipo == Tipo::Skill)
+            .collect();
+        assert_eq!(v.iter().map(|x| &x.entrada).collect::<Vec<_>>(), skills);
+        assert_eq!((il, av), (l.ilegibles, l.avisos));
+        assert!(v.iter().all(|x| x.cabecera.starts_with("---\nname: ")));
+    }
+
     /// H2 (ronda 1): una carpeta `skills` o `agents` que no se puede listar
     /// (aqui, un fichero en su lugar: `read_dir` falla sin ser `NotFound`,
     /// como sin permiso) es una incidencia `no-se-lee`, no una carpeta vacia.
diff --git a/crates/tc2/src/recursos.rs b/crates/tc2/src/recursos.rs
index 99f976a..d538931 100644
--- a/crates/tc2/src/recursos.rs
+++ b/crates/tc2/src/recursos.rs
@@ -345,6 +345,7 @@ mod tests {
             nombre: nombre.to_owned(),
             clave: format!("{nombre}@mk"),
             version: Some("1.0.0".to_owned()),
+            carpeta: None,
         }
     }
 
diff --git a/crates/tc2/src/seleccion/catalogo.rs b/crates/tc2/src/seleccion/catalogo.rs
index c67fb59..d341b80 100644
--- a/crates/tc2/src/seleccion/catalogo.rs
+++ b/crates/tc2/src/seleccion/catalogo.rs
@@ -5,8 +5,9 @@
 //! `umbral_dominios`) y, para las ajenas, los que declara `recursos:` de la
 //! configuracion ([`de_config`], F4-6).
 //!
-//! Se apoya en [`crate::inventario`] sin cambiarlo: el inventario dice que
-//! existe; el catalogo lee, de las mismas carpetas, que declara cada skill.
+//! Se apoya en [`crate::inventario`]: el inventario dice que existe (y lee
+//! las skills de una carpeta, `inventario::skills_de`); el catalogo saca,
+//! de esas mismas cabeceras, que declara cada skill.
 //! Solo se lee la cabecera de cada `SKILL.md` ([`MAX_CABECERA`]): un bloque
 //! que cae mas alla no cuenta (la skill queda sin metadatos y no se elige).
 //!
@@ -14,20 +15,18 @@
 //! - [`de_carpeta`]: `<dir>/<n>/SKILL.md` de un plugin dado por ruta (el
 //!   revisor en CI, con el plugin a una version fijada) o de una carpeta de
 //!   skills;
-//! - [`de_instalados`]: los plugins habilitados del inventario, por su
-//!   `installPath` de `installed_plugins.json`;
+//! - [`de_instalados`]: los plugins habilitados del inventario, por la
+//!   carpeta que el inventario eligio para cada uno;
 //! - [`de_base`]: `.claude/skills` del proyecto **en la base**
 //!   (`origin/develop`), nunca del worktree del PR (decision 2).
 
-use std::io::Read;
-use std::path::{Path, PathBuf};
+use std::path::Path;
 
 use serde::Deserialize;
-use serde_json::Value;
 
 use crate::contrato::Informe;
 use crate::git::{Git, Subcomando};
-use crate::inventario::{self, Entrada, Inventario, MAX_CABECERA, Origen, Tipo};
+use crate::inventario::{self, Entrada, Incidencia, Inventario, MAX_CABECERA, Origen, Tipo};
 
 /// Roles que puede declarar una entrada del catalogo.
 pub const ROLES: [&str; 3] = ["revisor", "diseno", "ejecucion"];
@@ -192,6 +191,11 @@ pub struct Lectura {
     pub entradas: Vec<Entrada>,
     /// Entradas del catalogo (revisoras y skills de diseno o ejecucion).
     pub revisoras: Vec<Revisora>,
+    /// Lo que no se pudo leer (C8): no cuenta como instalado ni como
+    /// ausente.
+    pub ilegibles: Vec<Incidencia>,
+    /// Avisos que no impiden leer (`nombre-distinto`).
+    pub avisos: Vec<Incidencia>,
 }
 
 /// Las entradas que declara `recursos:` de la configuracion (F1-14, F4-6):
@@ -216,75 +220,30 @@ pub fn de_config(recursos: &[crate::config::Recurso]) -> Vec<Revisora> {
         .collect()
 }
 
-/// Lee como mucho [`MAX_CABECERA`] bytes de `p`.
-fn cabecera(p: &Path) -> Option<String> {
-    let mut b = Vec::new();
-    std::fs::File::open(p)
-        .ok()?
-        .take(MAX_CABECERA)
-        .read_to_end(&mut b)
-        .ok()?;
-    Some(String::from_utf8_lossy(&b).into_owned())
-}
-
 /// Las skills de `skills` (`<n>/SKILL.md`) con el `prefijo` de ids
-/// (`plugin:` o vacio) y su `origen`.
+/// (`plugin:` o vacio) y su `origen`, leidas con
+/// `inventario::skills_de`: lo que no se puede leer queda en
+/// [`Lectura::ilegibles`] (C8), y cada cabecera se lee una sola vez.
 pub fn de_carpeta(skills: &Path, prefijo: &str, origen: &Origen) -> Lectura {
     let mut l = Lectura::default();
-    let mut hijos: Vec<PathBuf> = std::fs::read_dir(skills)
-        .map(|d| d.filter_map(Result::ok).map(|e| e.path()).collect())
-        .unwrap_or_default();
-    hijos.sort();
-    for d in hijos {
-        let f = d.join("SKILL.md");
-        let (Some(carpeta), Some(texto)) = (
-            d.file_name().and_then(|n| n.to_str()).map(str::to_owned),
-            f.is_file().then(|| cabecera(&f)).flatten(),
-        ) else {
-            continue;
-        };
-        let nombre = inventario::nombre_del_frontmatter(&texto).unwrap_or(carpeta);
-        let id = format!("{prefijo}{nombre}");
-        l.entradas.push(Entrada {
-            tipo: Tipo::Skill,
-            id: id.clone(),
-            origen: origen.clone(),
-        });
-        if let Some(r) = recurso_de(&id, &texto, &f.display().to_string()) {
+    for s in inventario::skills_de(skills, prefijo, origen, &mut l.ilegibles, &mut l.avisos) {
+        if let Some(r) = recurso_de(&s.entrada.id, &s.cabecera, &s.ruta.display().to_string()) {
             l.revisoras.push(r);
         }
+        l.entradas.push(s.entrada);
     }
     l
 }
 
-/// Las revisoras de los plugins habilitados de `inv`, leidas de su
-/// `installPath` en `installed_plugins.json` de `dir_claude` (la instalacion
-/// con la version del inventario o, si no la dice, la primera con ruta
-/// absoluta que existe). Los sincronizados de claude.ai no se leen (no traen
-/// revisoras con metadatos).
-pub fn de_instalados(dir_claude: &Path, inv: &Inventario) -> Vec<Revisora> {
-    let Ok(Some(Ok(v))) = inventario::leer_instalados(dir_claude) else {
-        return Vec::new();
-    };
+/// Las revisoras de los plugins habilitados de `inv`, leidas de la carpeta
+/// que el inventario eligio para cada uno ([`inventario::Plugin::carpeta`]:
+/// la instalacion que vale para el `scope` de este proyecto), sin volver a
+/// leer `installed_plugins.json`. Los sincronizados de claude.ai no se leen
+/// (no traen revisoras con metadatos).
+pub fn de_instalados(inv: &Inventario) -> Vec<Revisora> {
     let mut salida = Vec::new();
     for p in inv.plugins() {
-        let Some(lista) = v
-            .get("plugins")
-            .and_then(|m| m.get(&p.clave))
-            .and_then(Value::as_array)
-        else {
-            continue;
-        };
-        let ruta = |x: &Value| {
-            x.get("installPath")
-                .and_then(Value::as_str)
-                .map(PathBuf::from)
-                .filter(|d| d.is_absolute() && d.is_dir())
-        };
-        let misma = lista.iter().find(|x| {
-            p.version.is_some() && x.get("version").and_then(Value::as_str) == p.version.as_deref()
-        });
-        let Some(dir) = misma.and_then(ruta).or_else(|| lista.iter().find_map(ruta)) else {
+        let Some(dir) = &p.carpeta else {
             continue;
         };
         let origen = Origen::Plugin {
@@ -332,8 +291,7 @@ pub fn de_base(git: &Git, dir: &Path, base: &str) -> Result<Lectura, Informe> {
         let carpeta = ruta
             .trim_start_matches(".claude/skills/")
             .trim_end_matches("/SKILL.md");
-        let nombre =
-            inventario::nombre_del_frontmatter(&texto).unwrap_or_else(|| carpeta.to_owned());
+        let nombre = inventario::nombre_de(&texto, carpeta);
         l.entradas.push(Entrada {
             tipo: Tipo::Skill,
             id: nombre.clone(),
diff --git a/crates/tc2/src/seleccion/mod.rs b/crates/tc2/src/seleccion/mod.rs
index bc0958a..12916e5 100644
--- a/crates/tc2/src/seleccion/mod.rs
+++ b/crates/tc2/src/seleccion/mod.rs
@@ -256,12 +256,13 @@ impl Estado<'_> {
         self.entrada.dominios.iter().any(|(x, _)| x == d)
     }
 
-    /// Revisoras declaradas para el dominio `d`, por id.
+    /// Revisoras declaradas e instaladas para el dominio `d`, por id (S10 y
+    /// S11: una declarada y no instalada cuenta como «sin revisora»).
     fn revisoras_de(&self, d: &str) -> Vec<String> {
         self.cat
             .revisoras()
             .iter()
-            .filter(|r| r.dominios.iter().any(|x| x == d))
+            .filter(|r| r.dominios.iter().any(|x| x == d) && self.instalado(&r.id))
             .map(|r| r.id.clone())
             .collect()
     }
@@ -846,7 +847,12 @@ mod tests {
             "| S12 | `revision` | `carril=fix` | `lanzador` | `general-purpose` |",
         );
         assert_ne!(adr, tabla::ADR);
-        let s = revision_con(&adr, &rutas, &c, &i, &ctx).unwrap_or_default();
+        // La copia se tiene que leer (un `Err` no vale como seleccion vacia)
+        // y dar una seleccion de verdad (M04), sin el lanzador de S12.
+        let r = revision_con(&adr, &rutas, &c, &i, &ctx);
+        assert!(r.is_ok(), "{r:?}");
+        let s = r.unwrap_or_default();
+        assert!(s.reglas.contains(&"M04".to_owned()), "{s:?}");
         assert!(s.agentes.is_empty(), "{s:?}");
         let roto = tabla::ADR.replace("`lanzador`", "`a-ojo`");
         assert!(revision_con(&roto, &rutas, &c, &i, &ctx).is_err());
@@ -1327,6 +1333,68 @@ mod tests {
         assert_eq!(i.avisos, ["empate-por-id:e-m,e-n"]);
     }
 
+    /// A5-03 (TASK-143, F1-15 S10 y S11): una revisora de dominio declarada
+    /// en `recursos:` y no instalada cuenta como «sin revisora»: S11 con el
+    /// aviso `dominio-sin-revisora`, sin S10, S14 ni `no-instalado`.
+    /// Instalada, S10 la anade sin aviso.
+    #[test]
+    fn revisora_de_dominio_no_instalada_es_dominio_sin_revisora() {
+        let (cat, inv) = catalogo_muestra(&[]);
+        let mut rs: Vec<Revisora> = cat.revisoras().iter().chain(cat.otros()).cloned().collect();
+        rs.extend(catalogo::de_config(&[crate::config::Recurso {
+            id: "security-review".to_owned(),
+            tipo: "skill".to_owned(),
+            rol: Some("revisor".to_owned()),
+            dominios: vec!["puertas-seguridad".to_owned()],
+            etiquetas: Vec::new(),
+            prioridad: None,
+        }]));
+        let cat = Catalogo::nuevo(rs);
+        let criterios = vec!["criterio".to_owned()];
+        let puertas = dominios::Texto {
+            objetivo: "El check obligatorio",
+            criterios: &criterios,
+            etiquetas: &[],
+        };
+        let rutas = vec!["src/a.rs".to_owned()];
+        let r = |inv: &Inventario| {
+            tres(
+                "feature",
+                Carril::Funcionalidad,
+                "media",
+                &puertas,
+                &rutas,
+                &cat,
+                inv,
+                &todos(),
+                1,
+            )
+            .r
+        };
+        let s = r(&inv);
+        assert_eq!(s.avisos, ["dominio-sin-revisora:puertas-seguridad"]);
+        assert!(s.reglas.contains(&"S11".to_owned()), "{s:?}");
+        assert!(!s.reglas.iter().any(|x| x == "S10" || x == "S14"), "{s:?}");
+        assert!(
+            !s.revisoras.contains(&"security-review".to_owned()),
+            "{s:?}"
+        );
+        // Instalada: S10 sin aviso.
+        let mut es = inv.entradas().to_vec();
+        es.push(Entrada {
+            tipo: Tipo::Skill,
+            id: "security-review".to_owned(),
+            origen: Origen::Proyecto,
+        });
+        let s = r(&Inventario::nuevo(es, Vec::new()));
+        assert!(s.revisoras.contains(&"security-review".to_owned()), "{s:?}");
+        assert!(
+            s.reglas.contains(&"S10".to_owned()) && !s.reglas.contains(&"S11".to_owned()),
+            "{s:?}"
+        );
+        assert!(s.avisos.is_empty(), "{s:?}");
+    }
+
     /// Criterio 3 (RR-17): el nucleo de `seleccion` no nombra procesos, red,
     /// ficheros ni clientes de modelos fuera de los tests; y un agente de
     /// diseno quitado del inventario sale con `no-instalado` (S14), sin
diff --git a/crates/tc2/src/seleccion/orden.rs b/crates/tc2/src/seleccion/orden.rs
index 521045f..a1288f7 100644
--- a/crates/tc2/src/seleccion/orden.rs
+++ b/crates/tc2/src/seleccion/orden.rs
@@ -93,7 +93,8 @@ pub struct Fuentes {
     /// Las entradas con metadatos (revisoras y skills de diseno o
     /// ejecucion).
     pub catalogo: Catalogo,
-    /// Avisos (`sin-base`).
+    /// Avisos (`sin-base`, `sin-skills-de-base` y, con `--plugin`,
+    /// `ilegible:<motivo>:<id>` de lo que no se pudo leer).
     pub avisos: Vec<String>,
 }
 
@@ -120,9 +121,7 @@ pub fn fuentes(
     let mut lista_plugins = Vec::new();
     if plugins.is_empty() {
         let l = inventario::leer(claude, dir);
-        if let Some(c) = claude {
-            revisoras.extend(catalogo::de_instalados(c, &l.inventario));
-        }
+        revisoras.extend(catalogo::de_instalados(&l.inventario));
         entradas.extend(
             l.inventario
                 .entradas()
@@ -138,6 +137,12 @@ pub fn fuentes(
                 version: None,
             };
             let l = catalogo::de_carpeta(&d.join("skills"), &format!("{nombre}:"), &origen);
+            // Lo que no se lee no desaparece sin rastro (C8).
+            avisos.extend(
+                l.ilegibles
+                    .iter()
+                    .map(|i| format!("ilegible:{}:{}", i.motivo, i.id)),
+            );
             entradas.extend(l.entradas);
             revisoras.extend(l.revisoras);
         }
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/01-en-diseno/TASK-143/tarea.md                                                     | 48 ------------------------------------------------
 tareas/{01-en-diseno => 02-en-curso}/TASK-143/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-143/planificacion/plan-final.md                 |  0
 tareas/02-en-curso/TASK-143/tarea.md                                                      | 79 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 4 files changed, 79 insertions(+), 48 deletions(-)
````

## Tests del diff (5 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/comun/lanzador.rs" ":(literal)crates/tc2/tests/comun/mod.rs" ":(literal)crates/tc2/tests/doctor.rs" ":(literal)crates/tc2/tests/recursos.rs" ":(literal)crates/tc2/tests/revisoras.rs"
````

````
crates/tc2/tests/comun/lanzador.rs | 24 ++++++++++++++++++++++++
 crates/tc2/tests/comun/mod.rs      |  2 +-
 crates/tc2/tests/doctor.rs         | 71 +++++++++++++++++++++++++++++++----------------------------------------
 crates/tc2/tests/recursos.rs       | 16 +---------------
 crates/tc2/tests/revisoras.rs      | 36 ++++++++++++++++++++++++++++++++++++
 5 files changed, 93 insertions(+), 56 deletions(-)
````
## Candidatos 4D

Pistas generadas desde el diff, no hallazgos: comprueba cada una (Regla 4D de `revision.md`).

### Posible duplicacion con lo que ya existia

Definiciones nuevas con nombre igual o parecido a otra de la rama base. Mira si hacen lo mismo con pequenas diferencias (duplicacion real = IMPORTANTE).

- `Fuente` (crates/tc2/src/inventario.rs:151) ~ `Fuente` (crates/tc2/src/recursos.rs:109), `fuentes` (crates/tc2/src/i18n/guarda.rs:25), `Fuentes` (crates/tc2/src/seleccion/orden.rs:90) y 1 mas
- `hijos` (crates/tc2/src/inventario.rs:390) ~ `hijos` (crates/tc2/src/desinstalar/maquina.rs:21), `Hitos` (crates/tc2/src/grafo.rs:111), `hitos` (crates/tc2/src/roadmap/generar.rs:259)
- `Leida` (crates/tc2/src/inventario.rs:443) ~ `leido` (crates/tc2/src/estimacion/historico.rs:150), `Leido` (crates/tc2/src/tablero.rs:410), `leido` (crates/tc2/src/verificacion/informe.rs:197) y 3 mas
- `de_instalados` (crates/tc2/src/seleccion/catalogo.rs:243) ~ `instalados` (crates/tc2/src/desinstalar/maquina.rs:363), `no_instalados` (crates/tc2/src/recursos.rs:209)

