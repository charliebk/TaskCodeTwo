# Peticion de revision — TASK-078 (ronda 2)

- Tarea: TASK-078 — F2-14 tc2 desinstalar: quitar TaskCodeTwo de forma limpia
- Rama revisada: feature/task-078-f2-14-taskctl-desinstalar-quitar-taskcod
- Rama base: develop
- Commit revisado (HEAD): b0f38bd9378c4b8d05c63255a0ec7c3e89f834ab
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-078 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde b855c2057a8375e142ebdec0a665bce971e0af33 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (18 hallazgo(s)))

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

- tareas/03-en-revision/TASK-078/tarea.md
- tareas/03-en-revision/TASK-078/planificacion/
- crates/tc2/src/actualizar_cli.rs
- crates/tc2/src/desinstalar/maquina.rs
- crates/tc2/src/desinstalar/mod.rs
- crates/tc2/src/desinstalar/proyecto.rs
- crates/tc2/src/desinstalar/puertos.rs
- crates/tc2/src/desinstalar/texto.rs
- crates/tc2/src/git/mod.rs
- crates/tc2/src/i18n/catalogo/desinstalar.rs
- crates/tc2/src/main.rs
- crates/tc2/tests/desinstalar.rs
- docs/distribucion/instalar.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 9 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | abierto | crates/tc2/src/desinstalar/mod.rs:485 | informe-revision-1-code-quality-reviewer.md |
| IMP-2 | IMPORTANTE | abierto | crates/tc2/src/desinstalar/maquina.rs:182 | informe-revision-1-code-quality-reviewer.md |

## Commits a revisar (git log b855c2057a8375e142ebdec0a665bce971e0af33..HEAD)

````
b0f38bd fix(TASK-078): instalar.md sin TAB y con los avisos de uso
c378e94 fix(TASK-078): maquina segun la revision 1, con sus tests
ca7a71a fix(TASK-078): proyecto y resumen segun la revision 1
d7ed058 fix(TASK-078): autoborrado real sin DETACHED_PROCESS y PATH de usuario vacio sin null
f618989 chore(TASK-078): veredictos y coste de la revision ronda 1
be5786f chore(TASK-078): informes de revision ronda 1
2fa75ca chore(TASK-078): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff b855c2057a8375e142ebdec0a665bce971e0af33..HEAD)

````diff
diff --git a/crates/tc2/src/actualizar_cli.rs b/crates/tc2/src/actualizar_cli.rs
index 40a1368..fb05d3a 100644
--- a/crates/tc2/src/actualizar_cli.rs
+++ b/crates/tc2/src/actualizar_cli.rs
@@ -514,7 +514,10 @@ fn ultima_linea(bytes: &[u8]) -> Option<String> {
 /// entonces y los hilos acaban solos cuando el nieto cierre las tuberias (o
 /// con el proceso). Al nieto no se le mata: no hay una forma portable de
 /// matar el arbol sin dependencias.
-fn ejecutar_con_limite(mut cmd: Command, limite: Duration) -> std::io::Result<Option<Output>> {
+pub(crate) fn ejecutar_con_limite(
+    mut cmd: Command,
+    limite: Duration,
+) -> std::io::Result<Option<Output>> {
     use std::io::Read;
     use std::sync::mpsc;
     let mut hijo = cmd
diff --git a/crates/tc2/src/desinstalar/maquina.rs b/crates/tc2/src/desinstalar/maquina.rs
index 7b65f28..da3dac5 100644
--- a/crates/tc2/src/desinstalar/maquina.rs
+++ b/crates/tc2/src/desinstalar/maquina.rs
@@ -70,14 +70,23 @@ pub(super) fn plan(c: &Contexto<'_>, v: &mut Vec<Accion>) -> Result<(), Informe>
     let mut autoborrado: Option<usize> = None;
     let mut exe_cubierto = false;
 
-    // Plugin y marketplace: de Claude Code, la persona (D3).
-    let plugins = plugins(c)?;
+    // Plugin y marketplace: de Claude Code, la persona (D3). Se lee una vez.
+    let instalados = instalados(c)?;
+    let plugins = plugins(instalados.as_ref());
 
     for dir in e.dirs_de_instalacion() {
         let bin = dir.join(e.sistema.binario());
-        let hay_bin = bin.is_file();
-        let es_exe = hay_bin && misma_ruta(&bin, &e.exe);
+        let es_exe = bin.is_file() && misma_ruta(&bin, &e.exe);
         exe_cubierto |= es_exe;
+        // Solo es nuestro si es este `tc2`, si esta en la carpeta propia de
+        // Windows o si dice ser `tc2` (`version --json`). Un enlace a otra
+        // cosa o una herramienta ajena llamada `tc2` se quedan.
+        let hay_bin = bin.is_file() && (es_exe || e.dir_propio(&dir) || es_tc2(&bin));
+        if bin.is_file() && !hay_bin {
+            v.push(
+                Accion::nueva(t(&bin), "binario", "ajeno", Decision::Conserva).motivo("no-es-tc2"),
+            );
+        }
         let en_uso = hay_bin
             && windows
             && c.maquina
@@ -85,6 +94,8 @@ pub(super) fn plan(c: &Contexto<'_>, v: &mut Vec<Accion>) -> Result<(), Informe>
                 .map_err(|er| no_se_lee(&bin, &er))?;
         let mut grupo: Vec<Accion> = Vec::new();
         let path_antes = path.clone();
+        // Lo que este grupo quita: solo cuenta para las carpetas si se aplica.
+        let mut del_grupo: BTreeSet<PathBuf> = BTreeSet::new();
         let mut temporales: Vec<PathBuf> = hijos(&dir)
             .into_iter()
             .filter(|p| {
@@ -96,7 +107,7 @@ pub(super) fn plan(c: &Contexto<'_>, v: &mut Vec<Accion>) -> Result<(), Informe>
         temporales.sort();
         for p in temporales {
             grupo.push(Accion::borra(t(&p), "temporal", Efecto::Fichero(p.clone())));
-            quitados.insert(p);
+            del_grupo.insert(p);
         }
         let old = ruta_old(&bin);
         if old.is_file() {
@@ -105,7 +116,7 @@ pub(super) fn plan(c: &Contexto<'_>, v: &mut Vec<Accion>) -> Result<(), Informe>
                 "binario-old",
                 Efecto::Fichero(old.clone()),
             ));
-            quitados.insert(old);
+            del_grupo.insert(old);
         }
         if hay_bin {
             if es_exe && windows {
@@ -130,9 +141,14 @@ pub(super) fn plan(c: &Contexto<'_>, v: &mut Vec<Accion>) -> Result<(), Informe>
                     Efecto::Fichero(bin.clone()),
                 ));
             }
-            quitados.insert(bin.clone());
+            del_grupo.insert(bin.clone());
         }
-        if e.dir_propio(&dir) && dir.is_dir() && hijos(&dir).iter().all(|h| quitados.contains(h)) {
+        if e.dir_propio(&dir)
+            && dir.is_dir()
+            && hijos(&dir)
+                .iter()
+                .all(|h| quitados.contains(h) || del_grupo.contains(h))
+        {
             if let (true, Some(i)) = (es_exe && windows, autoborrado) {
                 if let Some(Accion {
                     efecto: Efecto::Autoborrado { dirs, .. },
@@ -152,7 +168,7 @@ pub(super) fn plan(c: &Contexto<'_>, v: &mut Vec<Accion>) -> Result<(), Informe>
                     Efecto::CarpetaSiVacia(dir.clone()),
                 ));
             }
-            quitados.insert(dir.clone());
+            del_grupo.insert(dir.clone());
         }
         // La entrada del PATH de usuario (inversa de `Add-PathUsuario`).
         if let Some(valor) = path.clone()
@@ -180,6 +196,8 @@ pub(super) fn plan(c: &Contexto<'_>, v: &mut Vec<Accion>) -> Result<(), Informe>
                 autoborrado = None;
             }
             path = path_antes;
+        } else {
+            quitados.extend(del_grupo);
         }
         v.extend(grupo);
     }
@@ -189,7 +207,7 @@ pub(super) fn plan(c: &Contexto<'_>, v: &mut Vec<Accion>) -> Result<(), Informe>
         match e.modo() {
             Modo::Plugin => {
                 let orden = plugins.first().map_or_else(
-                    || "claude plugin uninstall taskcodetwo".to_owned(),
+                    || "claude plugin uninstall 'taskcodetwo'".to_owned(),
                     Clone::clone,
                 );
                 v.push(
@@ -305,50 +323,90 @@ pub(super) fn plan(c: &Contexto<'_>, v: &mut Vec<Accion>) -> Result<(), Informe>
         }
     }
 
-    claude(c, v, plugins)
+    claude(c, v, plugins, instalados.as_ref())
 }
 
-/// Ordenes `claude plugin uninstall` del plugin de la 2.0, una por scope en
-/// que este instalado, leidas de `plugins/installed_plugins.json` (solo
-/// lectura: el fichero es de Claude Code).
-fn plugins(c: &Contexto<'_>) -> Result<Vec<String>, Informe> {
+/// Tiempo maximo de `<binario> version --json` al comprobar si es `tc2`.
+const ESPERA_VERSION: std::time::Duration = std::time::Duration::from_secs(10);
+
+/// Si `bin` es un `tc2`: no es un enlace simbolico y responde a
+/// `version --json` con su version y su protocolo (ADR F1-14, punto 5).
+fn es_tc2(bin: &Path) -> bool {
+    if std::fs::symlink_metadata(bin).is_ok_and(|m| m.file_type().is_symlink()) {
+        return false;
+    }
+    let mut cmd = std::process::Command::new(bin);
+    cmd.args(["version", "--json"]);
+    let Ok(Some(o)) = crate::actualizar_cli::ejecutar_con_limite(cmd, ESPERA_VERSION) else {
+        return false;
+    };
+    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap_or_default();
+    o.status.success()
+        && v.get("version").is_some_and(serde_json::Value::is_string)
+        && v.get("protocolo").is_some_and(serde_json::Value::is_u64)
+}
+
+/// `plugins/installed_plugins.json` de Claude Code (solo lectura), si esta
+/// y es JSON.
+fn instalados(c: &Contexto<'_>) -> Result<Option<serde_json::Value>, Informe> {
     let Some(dir) = c.entorno.dir_claude() else {
-        return Ok(Vec::new());
+        return Ok(None);
     };
     let p = dir.join("plugins").join("installed_plugins.json");
-    let Some(bytes) = leer(&p)? else {
-        return Ok(Vec::new());
-    };
-    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_default();
+    Ok(leer(&p)?.and_then(|b| serde_json::from_slice(&b).ok()))
+}
+
+/// Un nombre o scope que se puede poner en una orden: letras, digitos, `.`,
+/// `_`, `-` y `@`.
+fn seguro(t: &str) -> bool {
+    !t.is_empty()
+        && t.chars()
+            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '@'))
+}
+
+/// Ordenes `claude plugin uninstall` del plugin de la 2.0, una por scope en
+/// que este instalado. Una clave o un scope con otros caracteres no se pone
+/// en una orden; la clave va entre comillas simples.
+fn plugins(instalados: Option<&serde_json::Value>) -> Vec<String> {
     let mut ordenes = Vec::new();
-    if let Some(m) = v.get("plugins").and_then(serde_json::Value::as_object) {
-        for (clave, entradas) in m {
-            if clave.split('@').next() != Some(crate::huella::NOMBRE) {
-                continue;
-            }
-            let scopes: Vec<&str> = entradas
-                .as_array()
-                .map(|a| {
-                    a.iter()
-                        .filter_map(|x| x.get("scope").and_then(serde_json::Value::as_str))
-                        .collect()
-                })
-                .unwrap_or_default();
-            if scopes.is_empty() {
-                ordenes.push(format!("claude plugin uninstall {clave}"));
-            }
-            for s in scopes {
-                ordenes.push(format!("claude plugin uninstall {clave} --scope {s}"));
-            }
+    let Some(m) = instalados
+        .and_then(|v| v.get("plugins"))
+        .and_then(serde_json::Value::as_object)
+    else {
+        return ordenes;
+    };
+    for (clave, entradas) in m {
+        if clave.split('@').next() != Some(crate::huella::NOMBRE) || !seguro(clave) {
+            continue;
+        }
+        let scopes: Vec<&str> = entradas
+            .as_array()
+            .map(|a| {
+                a.iter()
+                    .filter_map(|x| x.get("scope").and_then(serde_json::Value::as_str))
+                    .filter(|s| seguro(s))
+                    .collect()
+            })
+            .unwrap_or_default();
+        if scopes.is_empty() {
+            ordenes.push(format!("claude plugin uninstall '{clave}'"));
+        }
+        for s in scopes {
+            ordenes.push(format!("claude plugin uninstall '{clave}' --scope {s}"));
         }
     }
-    Ok(ordenes)
+    ordenes
 }
 
 /// Lo de Claude Code: la status line (solo la de la 2.0, por su `command`
 /// exacto), los hooks que nombran `tc2` (aviso; nunca se editan) y el plugin
 /// (la persona) y el de la 0.16 (se conserva, F9-10).
-fn claude(c: &Contexto<'_>, v: &mut Vec<Accion>, plugins: Vec<String>) -> Result<(), Informe> {
+fn claude(
+    c: &Contexto<'_>,
+    v: &mut Vec<Accion>,
+    plugins: Vec<String>,
+    instalados: Option<&serde_json::Value>,
+) -> Result<(), Informe> {
     let Some(dir) = c.entorno.dir_claude() else {
         return Ok(());
     };
@@ -431,19 +489,18 @@ fn claude(c: &Contexto<'_>, v: &mut Vec<Accion>, plugins: Vec<String>) -> Result
         );
     }
     // El plugin de la 0.16 se nombra y no se toca (D6, F9-10).
-    let instalados = dir.join("plugins").join("installed_plugins.json");
-    if let Some(bytes) = leer(&instalados)? {
-        let val: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_default();
-        if let Some(m) = val.get("plugins").and_then(serde_json::Value::as_object) {
-            for clave in m
-                .keys()
-                .filter(|k| k.split('@').next() == Some("taskcode-plugin"))
-            {
-                v.push(
-                    Accion::nueva(clave.as_str(), "plugin", "taskcode-016", Decision::Conserva)
-                        .motivo("desinstalacion-f9-10"),
-                );
-            }
+    if let Some(m) = instalados
+        .and_then(|v| v.get("plugins"))
+        .and_then(serde_json::Value::as_object)
+    {
+        for clave in m
+            .keys()
+            .filter(|k| k.split('@').next() == Some("taskcode-plugin"))
+        {
+            v.push(
+                Accion::nueva(clave.as_str(), "plugin", "taskcode-016", Decision::Conserva)
+                    .motivo("desinstalacion-f9-10"),
+            );
         }
     }
     Ok(())
diff --git a/crates/tc2/src/desinstalar/mod.rs b/crates/tc2/src/desinstalar/mod.rs
index 6587863..f453cc9 100644
--- a/crates/tc2/src/desinstalar/mod.rs
+++ b/crates/tc2/src/desinstalar/mod.rs
@@ -200,6 +200,8 @@ pub struct Accion {
     pub motivo: Option<String>,
     /// La orden que lo quita, cuando la decision es de la persona.
     pub orden: Option<String>,
+    /// Commit que se pierde si se aplica (la rama de estado adelantada).
+    pub sha: Option<String>,
     /// Si impide aplicar el plan (`rechazada` sin tocar nada).
     pub bloquea: bool,
     /// Lo que se hace al aplicar.
@@ -221,6 +223,7 @@ impl Accion {
             decision,
             motivo: None,
             orden: None,
+            sha: None,
             bloquea: false,
             efecto: Efecto::Ninguno,
         }
@@ -254,7 +257,11 @@ impl Accion {
             self.dueno,
             self.decision.texto()
         );
-        for (k, v) in [("motivo", &self.motivo), ("orden", &self.orden)] {
+        for (k, v) in [
+            ("motivo", &self.motivo),
+            ("orden", &self.orden),
+            ("sha", &self.sha),
+        ] {
             if let Some(v) = v {
                 s.push_str(&format!(" {k}={}", escapar_valor(v, false)));
             }
@@ -271,7 +278,11 @@ impl Accion {
             cadena_json(self.dueno),
             cadena_json(self.decision.texto())
         );
-        for (k, v) in [("motivo", &self.motivo), ("orden", &self.orden)] {
+        for (k, v) in [
+            ("motivo", &self.motivo),
+            ("orden", &self.orden),
+            ("sha", &self.sha),
+        ] {
             if let Some(v) = v {
                 s.push_str(&format!(",\"{k}\":{}", cadena_json(v)));
             }
@@ -283,6 +294,13 @@ impl Accion {
     /// Aviso para la persona por stderr (aunque la orden salga bien): la
     /// orden que tiene que ejecutar, o el hook que nombra `tc2`.
     fn aviso(&self) -> Option<Mensaje> {
+        if let (Some("adelantada-del-remoto"), Some(sha)) = (self.motivo.as_deref(), &self.sha) {
+            return Some(
+                Mensaje::nuevo(&txt::AVISO_ADELANTADA)
+                    .con("rama", crate::huella::RAMA_ESTADO)
+                    .con("sha", sha.as_str()),
+            );
+        }
         if self.clase == "hook" {
             return Some(Mensaje::nuevo(&txt::AVISO_HOOK).con("ruta", self.ruta.as_str()));
         }
@@ -424,7 +442,16 @@ pub fn desinstalar(op: &Opciones, c: &Contexto<'_>) -> Hecho {
             .con("persona", n(Decision::Persona).to_string())
             .con("pendiente", n(Decision::Pendiente).to_string())
     };
-    if let Some(b) = acciones.iter().find(|a| a.bloquea) {
+    if let Some(b) = acciones.iter().find(|a| a.bloquea).cloned() {
+        // No se toca nada: lo que se habria borrado queda pendiente, y el
+        // resumen no cuenta borrados que no hay.
+        for a in acciones
+            .iter_mut()
+            .filter(|a| a.decision == Decision::Borra)
+        {
+            a.decision = Decision::Pendiente;
+            a.motivo = Some("no-aplicada".to_owned());
+        }
         let motivo = b.motivo.clone().unwrap_or_default();
         let i = base(Resultado::Rechazada, &acciones)
             .con("motivo", motivo.clone())
@@ -482,9 +509,16 @@ pub fn desinstalar(op: &Opciones, c: &Contexto<'_>) -> Hecho {
             };
         }
     }
-    let en_uso = acciones
-        .iter()
-        .find(|a| a.decision == Decision::Pendiente && a.motivo.as_deref() == Some("en-uso"));
+    // El que se nombra es el binario del grupo en uso, no su primera linea
+    // (un temporal o el `.old`).
+    let en_uso_de = |clase: Option<&str>| {
+        acciones.iter().find(|a| {
+            a.decision == Decision::Pendiente
+                && a.motivo.as_deref() == Some("en-uso")
+                && clase.is_none_or(|c| a.clase == c)
+        })
+    };
+    let en_uso = en_uso_de(Some("binario")).or_else(|| en_uso_de(None));
     let informe = if let Some(a) = en_uso {
         let ruta = a.ruta.clone();
         base(Resultado::Rechazada, &acciones)
@@ -525,7 +559,7 @@ fn aplicar(e: &Efecto, c: &Contexto<'_>) -> Result<(), String> {
             if &ahora != antes {
                 return Err("cambio desde el plan".to_owned());
             }
-            std::fs::write(ruta, despues).map_err(|e| e.to_string())
+            escribir_atomico(ruta, despues).map_err(|e| e.to_string())
         }
         Efecto::PathUsuario { viejo, nuevo } => c
             .maquina
@@ -548,6 +582,29 @@ fn aplicar(e: &Efecto, c: &Contexto<'_>) -> Result<(), String> {
     }
 }
 
+/// Escribe `ruta` de una vez: un temporal en la misma carpeta y un
+/// renombrado, para que un corte no deje el perfil o el `settings.json` de
+/// la persona vacio. Si `ruta` es un enlace simbolico se escribe su destino
+/// (como `cat >` de `install.sh`), y se conservan sus permisos.
+fn escribir_atomico(ruta: &Path, contenido: &[u8]) -> std::io::Result<()> {
+    let destino = if std::fs::symlink_metadata(ruta)?.file_type().is_symlink() {
+        std::fs::canonicalize(ruta)?
+    } else {
+        ruta.to_path_buf()
+    };
+    let permisos = std::fs::metadata(&destino)?.permissions();
+    let mut nombre = destino.file_name().unwrap_or_default().to_os_string();
+    nombre.push(format!(".tc2-tmp-{}", std::process::id()));
+    let tmp = destino.with_file_name(nombre);
+    let r = std::fs::write(&tmp, contenido)
+        .and_then(|()| std::fs::set_permissions(&tmp, permisos))
+        .and_then(|()| std::fs::rename(&tmp, &destino));
+    if r.is_err() {
+        let _ = std::fs::remove_file(&tmp);
+    }
+    r
+}
+
 /// La orden completa desde la linea de ordenes, en la carpeta actual y con
 /// el entorno y la maquina de verdad.
 pub fn orden(args: &[String], json: bool, idioma: Idioma) -> Salida {
diff --git a/crates/tc2/src/desinstalar/proyecto.rs b/crates/tc2/src/desinstalar/proyecto.rs
index 420a278..83d7432 100644
--- a/crates/tc2/src/desinstalar/proyecto.rs
+++ b/crates/tc2/src/desinstalar/proyecto.rs
@@ -13,7 +13,7 @@ use crate::estado::{CARPETA, Cerrojo};
 use crate::git::estado::dir_comun;
 use crate::git::rama::renombrar;
 use crate::git::{
-    CARPETA_WORKTREES, Git, REMOTO, Subcomando, commit_de, es_ancestro, exigir,
+    CARPETA_WORKTREES, Git, REMOTO, Subcomando, Worktree, commit_de, es_ancestro, exigir,
     localizar_principal, misma_ruta, worktrees,
 };
 use crate::huella::{self, RAMA_ESTADO};
@@ -25,14 +25,30 @@ fn lineas_exclude() -> [String; 2] {
     [format!("/{CARPETA_WORKTREES}/"), format!("/{CARPETA}/")]
 }
 
-/// Por que un worktree no se puede quitar sin perder nada, o `None`.
+/// Si la carpeta `dir` esta dentro de `ruta` (o es ella).
+fn dentro(dir: &Path, ruta: &Path) -> bool {
+    match (std::fs::canonicalize(dir), std::fs::canonicalize(ruta)) {
+        (Ok(d), Ok(r)) => d.starts_with(r),
+        _ => dir.starts_with(ruta),
+    }
+}
+
+/// Por que un worktree no se puede quitar sin perder nada (o sin que falle
+/// `git worktree remove` a mitad del plan), o `None`.
 fn bloqueo(
-    git: &Git,
+    c: &Contexto<'_>,
     principal: &Path,
-    ruta: &Path,
-    rama: Option<&str>,
+    w: &Worktree,
     de_tarea: bool,
 ) -> Result<Option<&'static str>, Informe> {
+    let (git, ruta, rama) = (c.git, w.ruta.as_path(), w.rama.as_deref());
+    if w.bloqueado {
+        return Ok(Some("worktree-bloqueado"));
+    }
+    // Desde dentro, `git worktree remove` no puede borrar la carpeta actual.
+    if dentro(c.dir, ruta) {
+        return Ok(Some("carpeta-actual-dentro"));
+    }
     let status = exigir(
         git,
         ruta,
@@ -85,7 +101,7 @@ pub(super) fn plan(op: &Opciones, c: &Contexto<'_>, v: &mut Vec<Accion>) -> Resu
         } else {
             "worktree-tarea"
         };
-        match bloqueo(g, &principal, &w.ruta, w.rama.as_deref(), de_tarea)? {
+        match bloqueo(c, &principal, w, de_tarea)? {
             Some(m) => {
                 let mut a =
                     Accion::nueva(t(&w.ruta), clase, "taskcodetwo", Decision::Conserva).motivo(m);
@@ -203,16 +219,29 @@ pub(super) fn plan(op: &Opciones, c: &Contexto<'_>, v: &mut Vec<Accion>) -> Resu
     }
     // La rama local de estado: solo con --borrar-tareas <nombre exacto>.
     let local = format!("refs/heads/{RAMA_ESTADO}");
+    let remota = format!("refs/remotes/{REMOTO}/{RAMA_ESTADO}");
+    let sha_remota = commit_de(g, &principal, NOMBRE, &remota)?;
     if let Some(sha) = commit_de(g, &principal, NOMBRE, &local)? {
         if op.borrar_tareas.is_some() {
-            v.push(Accion::borra(
+            // Con commits que el remoto no tiene, se avisa y la linea lleva
+            // el sha para recuperarlos (`git branch <rama> <sha>`).
+            let publicada = match &sha_remota {
+                Some(r) => es_ancestro(g, &principal, NOMBRE, &sha, r)?,
+                None => false,
+            };
+            let mut a = Accion::borra(
                 local,
                 "rama-estado",
                 Efecto::BorrarRamaEstado {
                     principal: principal.clone(),
-                    sha,
+                    sha: sha.clone(),
                 },
-            ));
+            );
+            if !publicada {
+                a = a.motivo("adelantada-del-remoto");
+                a.sha = Some(sha);
+            }
+            v.push(a);
         } else {
             v.push(
                 Accion::nueva(local, "rama-estado", "taskcodetwo", Decision::Conserva)
@@ -221,8 +250,7 @@ pub(super) fn plan(op: &Opciones, c: &Contexto<'_>, v: &mut Vec<Accion>) -> Resu
         }
     }
     // La remota nunca (D2): con --borrar-tareas, la orden para la persona.
-    let remota = format!("refs/remotes/{REMOTO}/{RAMA_ESTADO}");
-    if commit_de(g, &principal, NOMBRE, &remota)?.is_some() {
+    if sha_remota.is_some() {
         let a = Accion::nueva(remota, "rama-remota", "taskcodetwo", Decision::Conserva);
         v.push(if op.borrar_tareas.is_some() {
             let mut a = a.orden(format!("git push {REMOTO} --delete {RAMA_ESTADO}"));
@@ -232,13 +260,20 @@ pub(super) fn plan(op: &Opciones, c: &Contexto<'_>, v: &mut Vec<Accion>) -> Resu
             a.motivo("remota")
         });
     }
-    // Configuracion del proyecto: solo con --config (C6). La de la 0.16
-    // (sin `formato: 2`) no es de la 2.0: se queda siempre (D6).
+    // Configuracion del proyecto: solo con --config (C6). Es de la 2.0 si
+    // `config::leer` la reconoce (formato 2); la de la 0.16 no es suya y se
+    // queda siempre, y con ella `personas.yml`, compartido (F1-12 D4a, D6).
     let config = principal.join(crate::config::RUTA);
+    let mut de_la_016 = false;
     if let Some(bytes) = leer(&config)? {
-        let de_la_2 = String::from_utf8_lossy(&bytes)
-            .lines()
-            .any(|l| l.trim() == "formato: 2");
+        let de_la_2 = match crate::config::leer(&String::from_utf8_lossy(&bytes)) {
+            Ok(_) => true,
+            Err(e) => matches!(
+                e.motivo,
+                crate::config::Motivo::ConfigInvalida | crate::config::Motivo::FormatoNuevo
+            ),
+        };
+        de_la_016 = !de_la_2;
         v.push(match (de_la_2, op.config) {
             (false, _) => Accion::nueva(t(&config), "config", "taskcode-016", Decision::Conserva),
             (true, true) => Accion::borra(t(&config), "config", Efecto::Fichero(config.clone())),
@@ -248,7 +283,9 @@ pub(super) fn plan(op: &Opciones, c: &Contexto<'_>, v: &mut Vec<Accion>) -> Resu
     }
     let personas = principal.join(crate::plataforma::personas::RUTA);
     if personas.is_file() {
-        v.push(if op.config {
+        v.push(if de_la_016 {
+            Accion::nueva(t(&personas), "config", "taskcode-016", Decision::Conserva)
+        } else if op.config {
             Accion::borra(t(&personas), "config", Efecto::Fichero(personas.clone()))
         } else {
             Accion::nueva(t(&personas), "config", "taskcodetwo", Decision::Conserva)
diff --git a/crates/tc2/src/desinstalar/puertos.rs b/crates/tc2/src/desinstalar/puertos.rs
index 5af475e..9be575a 100644
--- a/crates/tc2/src/desinstalar/puertos.rs
+++ b/crates/tc2/src/desinstalar/puertos.rs
@@ -92,9 +92,11 @@ pub struct MaquinaReal;
 const PS_LEER_PATH: &str = "[Console]::OutputEncoding = [Text.Encoding]::UTF8; $k = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment'); if ($null -eq $k) { exit 3 }; $v = $k.GetValue('Path', $null, [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames); $k.Close(); if ($null -eq $v) { exit 3 }; [Console]::Out.Write([string]$v)";
 
 /// Escribe el PATH nuevo con el mismo tipo de valor si sigue siendo el
-/// viejo, y avisa a las ventanas abiertas como el instalador.
+/// viejo, y avisa a las ventanas abiertas como el instalador. Una variable
+/// de entorno vacia llega como `$null`: `[string]` la vuelve `''` (un PATH
+/// que se queda vacio se escribe vacio, sin pasar `$null` a `SetValue`).
 #[cfg(windows)]
-const PS_ESCRIBIR_PATH: &str = "$ErrorActionPreference = 'Stop'; $k = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment', $true); try { $actual = [string]$k.GetValue('Path', '', [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames); if ($actual -cne $env:TC2_PATH_VIEJO) { exit 4 }; $tipo = [Microsoft.Win32.RegistryValueKind]::ExpandString; if ($k.GetValueNames() -contains 'Path') { $tipo = $k.GetValueKind('Path') }; $k.SetValue('Path', $env:TC2_PATH_NUEVO, $tipo) } finally { $k.Close() }; [Environment]::SetEnvironmentVariable('TC2_INSTALAR_AVISO', '1', 'User'); [Environment]::SetEnvironmentVariable('TC2_INSTALAR_AVISO', $null, 'User')";
+const PS_ESCRIBIR_PATH: &str = "$ErrorActionPreference = 'Stop'; $k = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment', $true); try { $actual = [string]$k.GetValue('Path', '', [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames); if ($actual -cne [string]$env:TC2_PATH_VIEJO) { exit 4 }; $tipo = [Microsoft.Win32.RegistryValueKind]::ExpandString; if ($k.GetValueNames() -contains 'Path') { $tipo = $k.GetValueKind('Path') }; $k.SetValue('Path', [string]$env:TC2_PATH_NUEVO, $tipo) } finally { $k.Close() }; [Environment]::SetEnvironmentVariable('TC2_INSTALAR_AVISO', '1', 'User'); [Environment]::SetEnvironmentVariable('TC2_INSTALAR_AVISO', $null, 'User')";
 
 /// Cuenta los procesos distintos de `TC2_PID` que ejecutan `TC2_RUTA`.
 #[cfg(windows)]
@@ -167,9 +169,10 @@ impl Maquina for MaquinaReal {
         dirs: &[PathBuf],
     ) -> io::Result<()> {
         use std::os::windows::process::CommandExt;
-        // DETACHED_PROCESS | CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP: el
-        // auxiliar sobrevive a este proceso y a su consola.
-        const FLAGS: u32 = 0x0000_0008 | 0x0800_0000 | 0x0000_0200;
+        // CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP: el auxiliar sobrevive a
+        // este proceso. Sin DETACHED_PROCESS: con el, `powershell.exe` sale
+        // con 0 sin ejecutar el script (revision 1, IMP-1).
+        const FLAGS: u32 = 0x0800_0000 | 0x0000_0200;
         let unir = |v: &[PathBuf]| {
             let mut s = std::ffi::OsString::new();
             for (i, p) in v.iter().enumerate() {
@@ -225,3 +228,44 @@ impl Maquina for MaquinaReal {
         Err(io::Error::from(io::ErrorKind::Unsupported))
     }
 }
+
+#[cfg(all(test, windows))]
+mod tests {
+    use super::*;
+    use std::time::{Duration, Instant};
+
+    /// Revision 1, IMP-1 (C8): el auxiliar REAL borra un fichero y una
+    /// carpeta temporales propios cuando el PID ya ha acabado, en <= 10 s.
+    /// No toca el binario, el PATH ni el registro.
+    #[test]
+    fn el_auxiliar_real_borra_tras_salir_el_proceso() -> io::Result<()> {
+        let dir = std::env::temp_dir()
+            .join("tc2-tests")
+            .join(format!("autoborrado-{}", std::process::id()));
+        let _ = std::fs::remove_dir_all(&dir);
+        let sub = dir.join("carpeta con 'comilla' y $");
+        std::fs::create_dir_all(&sub)?;
+        let fichero = sub.join("tc2.exe");
+        std::fs::write(&fichero, b"x")?;
+        // Un PID que ya ha acabado.
+        let mut hijo = std::process::Command::new("cmd")
+            .args(["/c", "exit 0"])
+            .spawn()?;
+        let pid = hijo.id();
+        hijo.wait()?;
+        MaquinaReal.programar_borrado(
+            pid,
+            std::slice::from_ref(&fichero),
+            std::slice::from_ref(&sub),
+        )?;
+        let inicio = Instant::now();
+        while (fichero.exists() || sub.exists()) && inicio.elapsed() < Duration::from_secs(10) {
+            std::thread::sleep(Duration::from_millis(100));
+        }
+        let borrado = !fichero.exists() && !sub.exists();
+        let tardo = inicio.elapsed();
+        let _ = std::fs::remove_dir_all(&dir);
+        assert!(borrado, "tras {tardo:?} siguen el fichero o la carpeta");
+        Ok(())
+    }
+}
diff --git a/crates/tc2/src/desinstalar/texto.rs b/crates/tc2/src/desinstalar/texto.rs
index 8abad1b..addd36b 100644
--- a/crates/tc2/src/desinstalar/texto.rs
+++ b/crates/tc2/src/desinstalar/texto.rs
@@ -10,6 +10,22 @@ fn sin_final(l: &str) -> &str {
         .map_or(l, |x| x.strip_suffix('\r').unwrap_or(x))
 }
 
+/// Si `l` es exactamente la linea que escribe `install.sh` entre las
+/// marcas: `case ":$PATH:" in *":DIR:"*) ;; *) export PATH="DIR:$PATH" ;;
+/// esac`, con el mismo `DIR` en los dos sitios.
+fn linea_del_instalador(l: &str) -> bool {
+    let Some(r) = l
+        .strip_prefix(LINEA_PERFIL)
+        .and_then(|r| r.strip_prefix("*\":"))
+    else {
+        return false;
+    };
+    let Some((dir, resto)) = r.split_once(":\"*) ;; *) export PATH=\"") else {
+        return false;
+    };
+    !dir.is_empty() && resto == format!("{dir}:$PATH\" ;; esac")
+}
+
 /// Por que un perfil con marcas no se toca.
 #[derive(Debug, Clone, Copy, PartialEq, Eq)]
 pub enum MarcasRaras {
@@ -32,7 +48,9 @@ impl MarcasRaras {
 /// Inversa de `escribir_bloque` de `install.sh`: quita el bloque entre las
 /// marcas y la linea en blanco que el instalador escribio delante. El resto
 /// queda identico byte a byte. `Ok(None)` si no hay marcas; `Err` si las
-/// marcas no son un bloque entero de tres lineas (no se toca nada).
+/// marcas no son un bloque entero de tres lineas con la linea exacta del
+/// instalador (no se toca nada). Si el perfil no acababa en salto de linea
+/// antes de instalar, `install.sh` le anadio uno y ese no se quita.
 pub fn quitar_bloque(texto: &str) -> Result<Option<String>, MarcasRaras> {
     let lineas: Vec<&str> = texto.split_inclusive('\n').collect();
     let inicios: Vec<usize> = (0..lineas.len())
@@ -45,7 +63,7 @@ pub fn quitar_bloque(texto: &str) -> Result<Option<String>, MarcasRaras> {
         ([], []) => Ok(None),
         ([i], [f]) if i < f => {
             let (i, f) = (*i, *f);
-            if f != i + 2 || !sin_final(lineas[i + 1]).starts_with(LINEA_PERFIL) {
+            if f != i + 2 || !linea_del_instalador(sin_final(lineas[i + 1])) {
                 return Err(MarcasRaras::Tocadas);
             }
             let desde = if i > 0 && sin_final(lineas[i - 1]).is_empty() {
@@ -254,6 +272,16 @@ mod tests {
         assert_eq!(quitar_bloque(&doble), Err(MarcasRaras::Incompletas));
         let otro = "# >>> tc2 (TaskCodeTwo) >>>\nrm -rf ~\n# <<< tc2 (TaskCodeTwo) <<<\n";
         assert_eq!(quitar_bloque(otro), Err(MarcasRaras::Tocadas));
+        // Revision 1, MEN-1: la linea del instalador con algo anadido.
+        let anadido = BLOQUE.replace(
+            " esac
+",
+            " esac; rm -rf ~/x
+",
+        );
+        assert_eq!(quitar_bloque(&anadido), Err(MarcasRaras::Tocadas));
+        let otro_dir = BLOQUE.replacen("/h/.local/bin:$PATH", "/otro:$PATH", 1);
+        assert_eq!(quitar_bloque(&otro_dir), Err(MarcasRaras::Tocadas));
     }
 
     #[test]
diff --git a/crates/tc2/src/git/mod.rs b/crates/tc2/src/git/mod.rs
index a7684b7..9a3d4de 100644
--- a/crates/tc2/src/git/mod.rs
+++ b/crates/tc2/src/git/mod.rs
@@ -151,6 +151,8 @@ pub fn base_de_rama(rama: &str) -> &'static str {
 pub(crate) struct Worktree {
     pub ruta: PathBuf,
     pub rama: Option<String>,
+    /// Con `git worktree lock`: `git worktree remove` lo rechaza.
+    pub bloqueado: bool,
 }
 
 pub(crate) fn worktrees(git: &Git, dir: &Path, op: &str) -> Paso<Vec<Worktree>> {
@@ -161,9 +163,12 @@ pub(crate) fn worktrees(git: &Git, dir: &Path, op: &str) -> Paso<Vec<Worktree>>
             v.push(Worktree {
                 ruta: PathBuf::from(r),
                 rama: None,
+                bloqueado: false,
             });
         } else if let (Some(r), Some(w)) = (l.strip_prefix("branch refs/heads/"), v.last_mut()) {
             w.rama = Some(r.to_owned());
+        } else if let (true, Some(w)) = (l == "locked" || l.starts_with("locked "), v.last_mut()) {
+            w.bloqueado = true;
         }
     }
     Ok(v)
diff --git a/crates/tc2/src/i18n/catalogo/desinstalar.rs b/crates/tc2/src/i18n/catalogo/desinstalar.rs
index efeb039..9d3ee56 100644
--- a/crates/tc2/src/i18n/catalogo/desinstalar.rs
+++ b/crates/tc2/src/i18n/catalogo/desinstalar.rs
@@ -17,6 +17,10 @@ crate::i18n::textos! {
         es: "aviso: {ruta} tiene un hook que nombra tc2; desinstalar nunca edita los hooks: quitalo tu si ya no lo quieres",
         en: "warning: {ruta} has a hook that names tc2; desinstalar never edits hooks: remove it yourself if you no longer want it",
     }
+    AVISO_ADELANTADA {
+        es: "aviso: la rama local {rama} tiene commits que el remoto no tiene; se borra porque lo pediste con su nombre exacto. Para recuperarlos: git branch {rama} {sha}",
+        en: "warning: the local branch {rama} has commits the remote does not have; it is deleted because you asked for it by its exact name. To recover them: git branch {rama} {sha}",
+    }
     BLOQUEADA {
         es: "no se ha tocado nada: {ruta} ({motivo}). Guarda y publica sus cambios (o quitalo tu) y repite",
         en: "nothing was touched: {ruta} ({motivo}). Save and push its changes (or remove it yourself) and try again",
diff --git a/crates/tc2/src/main.rs b/crates/tc2/src/main.rs
index f925ace..7e2c429 100644
--- a/crates/tc2/src/main.rs
+++ b/crates/tc2/src/main.rs
@@ -13,8 +13,14 @@ fn main() -> ExitCode {
     // F2-12: en Windows, `actualizar --cli` deja `tc2.exe.old` (un ejecutable
     // en marcha no se puede borrar); lo borra la siguiente ejecucion. Una
     // llamada al sistema si no existe.
+    // `desinstalar` no: su `--simular` no toca nada (revision 1, MEN-6 de
+    // TASK-078), y el `.old` lo quita su propio plan.
     #[cfg(windows)]
-    if let Ok(exe) = std::env::current_exe() {
+    if std::env::args_os()
+        .nth(1)
+        .is_none_or(|a| a != "desinstalar")
+        && let Ok(exe) = std::env::current_exe()
+    {
         tc2::actualizar_cli::limpiar_old(&exe);
     }
     let idioma = Idioma::de_entorno(|k| std::env::var(k).ok());
diff --git a/docs/distribucion/instalar.md b/docs/distribucion/instalar.md
index 8d31cce..b0726b7 100644
--- a/docs/distribucion/instalar.md
+++ b/docs/distribucion/instalar.md
@@ -159,15 +159,19 @@ tc2 desinstalar --maquina
   ejecutar la persona salen también, legibles, por stderr.
 - **Qué quita `--maquina`** (solo lo que reconoce por su ruta, sus marcas o
   su `command` exactos): el binario de `TC2_INSTALL_DIR` y del directorio por
-  defecto, su `tc2.exe.old` y los temporales `.tc2-*.descarga` y
+  defecto (si es este `tc2`, está en la carpeta propia de Windows o responde
+  a `tc2 version --json`; otro `tc2` queda `dueno=ajeno`), su `tc2.exe.old` y los temporales `.tc2-*.descarga` y
   `.tc2.nuevo.*`; el bloque entre marcas de `~/.profile`, `~/.bashrc`,
   `~/.zshrc` y `~/.bash_profile` (el resto del perfil queda igual byte a
-  byte); en Windows, la entrada del PATH de usuario (la inversa de lo que
+  byte, salvo el salto de línea final que `install.sh` añadió a un perfil que
+  no lo tenía; si la línea entre las marcas no es exactamente la del
+  instalador, no se toca y queda para la persona); en Windows, la entrada del PATH de usuario (la inversa de lo que
   añadió `install.ps1`; las demás entradas, intactas) y la carpeta
-  `%LOCALAPPDATA%	askcodetwo` si queda vacía; la caché del lanzador
+  `%LOCALAPPDATA%\taskcodetwo` si queda vacía; la caché del lanzador
   (`taskcodetwo/cli/`), y la `statusLine` de `~/.claude/settings.json` solo si
-  su `command` es exactamente `tc2 estado --stdin` (sin tocar ningún otro
-  byte del fichero).
+  su `command` es exactamente `tc2 estado --stdin`, sin contar los espacios
+  de los extremos (sin tocar ningún otro byte del fichero). Los ficheros se
+  reescriben de una vez (temporal y renombrado).
 - **En Windows, el `tc2.exe` en marcha** no se puede borrar: lo borra un
   proceso auxiliar en cuanto `tc2` sale (como mucho 10 s; `motivo=al-salir`).
   Si otro proceso está ejecutando ese binario, su grupo (binario, `.old`,
@@ -175,7 +179,7 @@ tc2 desinstalar --maquina
   resto se quita y la orden sale `rechazada` (3).
 - **Qué no toca**: los hooks de un `settings.json` (si uno nombra `tc2`, lo
   avisa); el plugin y el marketplace, que son de Claude Code (imprime
-  `claude plugin uninstall taskcodetwo@<marketplace> --scope <scope>`); un
+  `claude plugin uninstall 'taskcodetwo@<marketplace>' --scope <scope>`); un
   `tc2` de la caché del plugin o de un gestor de paquetes (imprime la orden
   del gestor: `npm uninstall -g taskcodetwo`, `brew uninstall taskcodetwo`,
   `winget uninstall taskcodetwo`, `scoop uninstall taskcodetwo` o `cargo
@@ -185,14 +189,22 @@ tc2 desinstalar --maquina
   los de tarea (`.taskcodetwo/worktrees/<id>`), con sus registros;
   `.git/taskcodetwo/` (cerrojo, caché de identidad, índices temporales) y las
   líneas `/.taskcodetwo/worktrees/` y `/tareas/` de `.git/info/exclude`. Si un
-  worktree tiene cambios sin guardar o commits sin publicar, sale `rechazada`
-  sin tocar nada. Nunca usa la red ni toca el remoto.
-- **Con confirmación**: `.taskcode/config.yml` (solo la de formato 2) y
-  `.taskcode/personas.yml` solo con `--config`; la rama local
+  worktree tiene cambios sin guardar o commits sin publicar, está bloqueado
+  (`git worktree lock`) o contiene la carpeta desde la que se lanza, sale
+  `rechazada` sin tocar nada. Nunca usa la red ni toca el remoto.
+- **Con confirmación**: `.taskcode/config.yml` (solo la de formato 2, la que
+  lee `tc2`) y `.taskcode/personas.yml` solo con `--config`; con la config de
+  la 0.16, las dos se quedan, porque son compartidas; la rama local
   `taskcodetwo/tareas` y las refs `refs/taskcodetwo/*` solo con
   `--borrar-tareas taskcodetwo/tareas` (el nombre exacto). La rama remota
   nunca: imprime `git push origin --delete taskcodetwo/tareas` para que lo
-  decida la persona.
+  decida la persona. Si la rama local tiene commits que el remoto no tiene,
+  la línea lleva `motivo=adelantada-del-remoto sha=<sha>` y stderr dice cómo
+  recuperarlos (`git branch taskcodetwo/tareas <sha>`).
+- **Uso**: `--config` y `--borrar-tareas` sin `--proyecto` dan `uso` (2), y
+  otro nombre de rama en `--borrar-tareas`, también. Con `--maquina
+  --proyecto` fuera de un repo sale `rechazada motivo=no-es-repo` sin hacer
+  nada, tampoco en la máquina.
 
 ## Compatibilidad con el plugin
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff b855c2057a8375e142ebdec0a665bce971e0af33..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-078/planificacion/brainstorm/peticion-plan-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-078/planificacion/plan-final.md                 |    0
 tareas/03-en-revision/TASK-078/revision/informe-revision-1-code-quality-reviewer.md         |   98 ++++
 tareas/03-en-revision/TASK-078/revision/informe-revision-1-rust-wasm-reviewer.md            |   92 ++++
 tareas/03-en-revision/TASK-078/revision/peticion-revision-1-code-quality-reviewer.md        |  237 +++++++++
 tareas/03-en-revision/TASK-078/revision/peticion-revision-1-rust-wasm-reviewer.md           | 2886 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-078/tarea.md                                    |   18 +-
 7 files changed, 3327 insertions(+), 4 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff b855c2057a8375e142ebdec0a665bce971e0af33..HEAD -- ":(literal)crates/tc2/tests/desinstalar.rs"
````

````
crates/tc2/tests/desinstalar.rs | 159 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++----
 1 file changed, 155 insertions(+), 4 deletions(-)
````
