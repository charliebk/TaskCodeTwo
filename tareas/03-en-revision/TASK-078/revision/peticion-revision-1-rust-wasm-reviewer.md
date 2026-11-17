# Peticion de revision — TASK-078 (ronda 1)

- Tarea: TASK-078 — F2-14 tc2 desinstalar: quitar TaskCodeTwo de forma limpia
- Rama revisada: feature/task-078-f2-14-taskctl-desinstalar-quitar-taskcod
- Rama base: develop
- Commit revisado (HEAD): b855c2057a8375e142ebdec0a665bce971e0af33
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-078 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-078/tarea.md
- tareas/03-en-revision/TASK-078/planificacion/
- crates/distribucion/src/lib.rs
- crates/tc2/src/actualizar_cli.rs
- crates/tc2/src/contrato.rs
- crates/tc2/src/desinstalar/maquina.rs
- crates/tc2/src/desinstalar/mod.rs
- crates/tc2/src/desinstalar/proyecto.rs
- crates/tc2/src/desinstalar/puertos.rs
- crates/tc2/src/desinstalar/texto.rs
- crates/tc2/src/git/puerto.rs
- crates/tc2/src/i18n/catalogo/cli.rs
- crates/tc2/src/i18n/catalogo/desinstalar.rs
- crates/tc2/src/i18n/catalogo/mod.rs
- crates/tc2/src/instalacion.rs
- crates/tc2/src/lib.rs
- crates/tc2/tests/desinstalar.rs
- xtask/src/instaladores.rs

## Commits a revisar (git log develop..HEAD)

````
b855c20 docs(TASK-078): autorrevision con las revisoras de la tarea
390580b refactor(TASK-078): leer y ruta como texto en un solo sitio de desinstalar
4c8c27e docs(TASK-078): como desinstalar tc2 de la maquina y de un proyecto
5e891ff test(TASK-078): paso de desinstalacion en probar-instalador.sh (sin verificar hasta el push)
6de40bd test(TASK-078): un test por criterio con HOME falso y maquina en memoria
71a2810 feat(TASK-078): tc2 desinstalar con plan como dato, --simular y los dos ambitos
91665e6 feat(TASK-078): puerto git con borrado acotado de la rama de estado y lecturas sin cerrojos opcionales
02f2bd4 feat(TASK-078): marcas del perfil y status line como constantes compartidas con install.sh
9a32c3b refactor(TASK-078): extraer instalacion con Modo, rutas y Entorno inyectado
b6fffce chore(TASK-078): tarea en curso
````

## Diff de tu dominio (16 fichero(s) de 18; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/distribucion/src/lib.rs b/crates/distribucion/src/lib.rs
index 7805a61..8f974b5 100644
--- a/crates/distribucion/src/lib.rs
+++ b/crates/distribucion/src/lib.rs
@@ -22,6 +22,24 @@ pub const REPO: &str = "charliebk/TaskCodeTwo";
 /// (`gh attestation verify --signer-workflow`), como en los instaladores.
 pub const WORKFLOW_RELEASE: &str = "charliebk/TaskCodeTwo/.github/workflows/release.yml";
 
+/// Marca de inicio del bloque que `install.sh` anade al perfil del shell
+/// (F2-11) y que `tc2 desinstalar` quita (F2-14). Un test de `xtask` exige
+/// que `install.sh` use exactamente esta.
+pub const MARCA_INICIO_PERFIL: &str = "# >>> tc2 (TaskCodeTwo) >>>";
+
+/// Marca de fin del bloque del perfil del shell.
+pub const MARCA_FIN_PERFIL: &str = "# <<< tc2 (TaskCodeTwo) <<<";
+
+/// Linea del bloque del perfil entre las marcas, sin el directorio: empieza
+/// asi y lo demas es el `export` con el directorio de la instalacion.
+pub const LINEA_PERFIL: &str = "case \":$PATH:\" in ";
+
+/// `command` de la status line de la 2.0 en `settings.json` (F1-12 D1a; la
+/// escribira F3-12). Es el reconocedor cerrado de `tc2 desinstalar`: otra
+/// status line, tambien la de la 0.16 (`taskctl estado --stdin`), no es
+/// suya.
+pub const STATUS_LINE: &str = "tc2 estado --stdin";
+
 /// Releases del repositorio: `cli.lock` apunta a `<RELEASES>/download/<tag>/`.
 pub const RELEASES: &str = "https://github.com/charliebk/TaskCodeTwo/releases";
 
diff --git a/crates/tc2/src/actualizar_cli.rs b/crates/tc2/src/actualizar_cli.rs
index abe8140..40a1368 100644
--- a/crates/tc2/src/actualizar_cli.rs
+++ b/crates/tc2/src/actualizar_cli.rs
@@ -36,68 +36,7 @@ use crate::i18n::catalogo::actualizar as txt;
 /// Nombre de la orden.
 pub const NOMBRE: &str = "actualizar";
 
-/// Quien gestiona el `tc2` que se esta ejecutando.
-#[derive(Debug, Clone, Copy, PartialEq, Eq)]
-pub enum Modo {
-    /// Instalado con la linea de instalacion (o copiado a mano): se actualiza.
-    Instalador,
-    /// La cache del plugin (`.../taskcodetwo/cli/<version>/tc2[.exe]`): lo
-    /// gestiona el plugin con `cli.lock`.
-    Plugin,
-    /// Un gestor de paquetes (npm, Homebrew, winget, Scoop, cargo): se
-    /// actualiza con el.
-    Gestor(&'static str),
-}
-
-/// Modo de un ejecutable por su ruta (sin tocar el disco). Reconoce las
-/// rutas de Windows, macOS y Linux sin depender del sistema en que corre.
-pub fn detectar_modo(ruta: &str) -> Modo {
-    let partes: Vec<String> = ruta
-        .split(['/', '\\'])
-        .filter(|p| !p.is_empty())
-        .map(str::to_ascii_lowercase)
-        .collect();
-    let n = partes.len();
-    let es_tc2 = partes.last().is_some_and(|f| f == "tc2" || f == "tc2.exe");
-    if es_tc2
-        && n >= 4
-        && partes.get(n - 4).is_some_and(|p| p == "taskcodetwo")
-        && partes.get(n - 3).is_some_and(|p| p == "cli")
-    {
-        return Modo::Plugin;
-    }
-    // Cada gestor por su propia estructura de carpetas, no por un nombre
-    // suelto: una carpeta `Scoop\bin` o `~/.cargo/otra` de la persona no es
-    // de un gestor. `dir` son las carpetas, sin el fichero.
-    let dir = partes.get(..n.saturating_sub(1)).unwrap_or_default();
-    let seguidas = |a: &str, b: &str| dir.windows(2).any(|w| w[0] == a && w[1] == b);
-    // `<raiz>/node_modules/<paquete>/...`: el binario dentro de un paquete.
-    let npm = dir
-        .iter()
-        .position(|p| p == "node_modules")
-        .is_some_and(|i| i + 1 < dir.len());
-    // `Cellar/<formula>/<version>/...`, o el prefijo de Homebrew.
-    let homebrew = dir
-        .iter()
-        .position(|p| p == "cellar")
-        .is_some_and(|i| i + 2 < dir.len())
-        || dir.starts_with(&["opt".to_owned(), "homebrew".to_owned()])
-        || seguidas("linuxbrew", ".linuxbrew");
-    let gestor = if npm {
-        Some("npm")
-    } else if homebrew {
-        Some("homebrew")
-    } else if seguidas("winget", "packages") || seguidas("winget", "links") {
-        Some("winget")
-    } else if seguidas("scoop", "apps") || seguidas("scoop", "shims") {
-        Some("scoop")
-    } else if dir.len() >= 2 && dir[dir.len() - 2] == ".cargo" && dir[dir.len() - 1] == "bin" {
-        Some("cargo")
-    } else {
-        None
-    };
-    gestor.map_or(Modo::Instalador, Modo::Gestor)
-}
+pub use crate::instalacion::{Modo, detectar_modo, limpiar_old, ruta_old};
 
 /// Compara dos versiones `X.Y.Z[-pre]` con las reglas de semver: el nucleo
 /// por numeros y, a igual nucleo, una prerelease es anterior a la estable;
@@ -809,79 +748,10 @@ pub fn sustituir_con(
     Ok(())
 }
 
-/// `<exe>.old`: donde queda el binario anterior en Windows hasta la
-/// siguiente ejecucion.
-pub fn ruta_old(exe: &Path) -> PathBuf {
-    let mut s = exe.as_os_str().to_owned();
-    s.push(".old");
-    PathBuf::from(s)
-}
-
-/// Borra `<exe>.old` si lo hay (lo deja una actualizacion en Windows, donde
-/// un ejecutable en marcha no se puede borrar). Si sigue en uso, se queda.
-pub fn limpiar_old(exe: &Path) {
-    let _ = std::fs::remove_file(ruta_old(exe));
-}
-
 #[cfg(test)]
 mod tests {
     use super::*;
 
-    /// TASK-027, criterio 6: la cache del plugin se reconoce en los tres
-    /// sistemas, y no se confunde con una instalacion normal.
-    #[test]
-    fn detectar_modo_con_rutas_de_los_tres_sistemas() {
-        for plugin in [
-            r"C:\Users\ana\AppData\Local\taskcodetwo\cli\2.0.0\tc2.exe",
-            r"c:\users\ana\appdata\local\TaskCodeTwo\CLI\2.0.0-rc.1\TC2.EXE",
-            "/Users/ana/.cache/taskcodetwo/cli/2.0.0/tc2",
-            "/Users/ana/Library/Caches/taskcodetwo/cli/2.0.0/tc2",
-            "/home/ana/.cache/taskcodetwo/cli/2.0.0/tc2",
-        ] {
-            assert_eq!(detectar_modo(plugin), Modo::Plugin, "{plugin}");
-        }
-        for instalador in [
-            r"C:\Users\ana\AppData\Local\taskcodetwo\bin\tc2.exe",
-            "/Users/ana/.local/bin/tc2",
-            "/home/ana/.local/bin/tc2",
-            "/home/ana/taskcodetwo/cli/tc2",
-            "/home/ana/taskcodetwo/cli/2.0.0/otro",
-            "/opt/herramientas/tc2",
-        ] {
-            assert_eq!(detectar_modo(instalador), Modo::Instalador, "{instalador}");
-        }
-        for (ruta, gestor) in [
-            (
-                r"C:\Users\ana\AppData\Roaming\npm\node_modules\taskcodetwo\bin\tc2.exe",
-                "npm",
-            ),
-            ("/opt/homebrew/Cellar/tc2/2.0.0/bin/tc2", "homebrew"),
-            (
-                r"C:\Users\ana\AppData\Local\Microsoft\WinGet\Packages\x\tc2.exe",
-                "winget",
-            ),
-            (r"C:\Users\ana\scoop\apps\tc2\current\tc2.exe", "scoop"),
-            ("/home/ana/.cargo/bin/tc2", "cargo"),
-            ("/home/linuxbrew/.linuxbrew/bin/tc2", "homebrew"),
-            (r"C:\Users\ana\scoop\shims\tc2.exe", "scoop"),
-        ] {
-            assert_eq!(detectar_modo(ruta), Modo::Gestor(gestor), "{ruta}");
-        }
-        // Revision 1, MEN-1: un nombre de gestor suelto en la ruta no basta.
-        for propia in [
-            r"C:\Users\ana\Scoop\bin\tc2.exe",
-            r"C:\herramientas\scoop\tc2.exe",
-            "/home/ana/.cargo/otra/tc2",
-            "/home/ana/.cargo/bin/sub/tc2",
-            "/home/ana/node_modules/tc2",
-            "/home/ana/cellar/tc2",
-            "/home/ana/homebrew/tc2",
-            r"C:\Users\ana\winget\tc2.exe",
-        ] {
-            assert_eq!(detectar_modo(propia), Modo::Instalador, "{propia}");
-        }
-    }
-
     /// Revision 2, MEN-6: el plazo se cumple aunque el hijo deje un nieto
     /// con las tuberias heredadas (aqui, un shell que lanza una espera larga
     /// y no la sustituye con `exec`). Antes, la llamada esperaba al nieto.
diff --git a/crates/tc2/src/contrato.rs b/crates/tc2/src/contrato.rs
index 523bb3f..6f71178 100644
--- a/crates/tc2/src/contrato.rs
+++ b/crates/tc2/src/contrato.rs
@@ -249,7 +249,7 @@ pub fn corto(hash: &str) -> String {
 
 /// Los valores no llevan espacios: espacio, `%`, controles (y `,` dentro de
 /// una lista) se escriben como `%XX`, de forma reversible.
-fn escapar_valor(v: &str, en_lista: bool) -> String {
+pub(crate) fn escapar_valor(v: &str, en_lista: bool) -> String {
     let mut s = String::with_capacity(v.len());
     for c in v.chars() {
         if c == '%' || c.is_whitespace() || c.is_control() || (en_lista && c == ',') {
diff --git a/crates/tc2/src/desinstalar/maquina.rs b/crates/tc2/src/desinstalar/maquina.rs
new file mode 100644
index 0000000..7b65f28
--- /dev/null
+++ b/crates/tc2/src/desinstalar/maquina.rs
@@ -0,0 +1,450 @@
+//! Inventario y plan de `desinstalar --maquina`: el binario y lo que dejan
+//! junto a el los instaladores y `actualizar --cli`, el bloque del perfil
+//! del shell, el PATH de usuario de Windows, la cache del lanzador, la
+//! status line y, solo como aviso, el plugin y los hooks.
+
+use std::collections::BTreeSet;
+use std::path::{Path, PathBuf};
+
+use distribucion::{MARCA_INICIO_PERFIL, STATUS_LINE};
+
+use super::texto::{hooks_con_tc2, quitar_bloque, quitar_del_path, quitar_miembro};
+use super::{Accion, Contexto, Decision, Efecto, leer, no_se_lee, orden_borrar, t};
+use crate::contrato::Informe;
+use crate::git::misma_ruta;
+use crate::instalacion::{Modo, Sistema, orden_del_gestor, ruta_old};
+
+/// Perfiles del shell donde `install.sh` puede dejar el bloque.
+const PERFILES: [&str; 4] = [".profile", ".bashrc", ".zshrc", ".bash_profile"];
+
+/// Hijos de una carpeta (vacio si no existe).
+fn hijos(dir: &Path) -> Vec<PathBuf> {
+    std::fs::read_dir(dir)
+        .map(|d| d.flatten().map(|e| e.path()).collect())
+        .unwrap_or_default()
+}
+
+/// Un temporal que deja un instalador o `actualizar --cli` junto al binario:
+/// `.tc2-<v>-<pid>.descarga` o `.tc2.nuevo.<pid>[.exe]`.
+fn es_temporal(nombre: &str) -> bool {
+    (nombre.starts_with(".tc2-") && nombre.ends_with(".descarga"))
+        || nombre.starts_with(".tc2.nuevo.")
+}
+
+/// Expande `%LOCALAPPDATA%` y `%USERPROFILE%` como lo haria Windows al
+/// comparar entradas del PATH (las unicas que puede usar un instalador).
+fn expandir(c: &Contexto<'_>, e: &str) -> String {
+    let mut s = e.to_owned();
+    for (var, valor) in [
+        ("%LOCALAPPDATA%", &c.entorno.local_app_data),
+        ("%USERPROFILE%", &c.entorno.home),
+    ] {
+        if let Some(v) = valor {
+            let mut out = String::new();
+            let mut resto = s.as_str();
+            while let Some(i) = resto.to_ascii_uppercase().find(var) {
+                out.push_str(&resto[..i]);
+                out.push_str(&t(v));
+                resto = &resto[i + var.len()..];
+            }
+            out.push_str(resto);
+            s = out;
+        }
+    }
+    s
+}
+
+pub(super) fn plan(c: &Contexto<'_>, v: &mut Vec<Accion>) -> Result<(), Informe> {
+    let e = c.entorno;
+    let windows = e.sistema == Sistema::Windows;
+    // Lo que se va a quitar entero: para saber si una carpeta queda vacia.
+    let mut quitados: BTreeSet<PathBuf> = BTreeSet::new();
+    let mut path = if windows {
+        c.maquina
+            .leer_path_usuario()
+            .map_err(|er| no_se_lee(Path::new("HKCU\\Environment\\Path"), &er))?
+    } else {
+        None
+    };
+    // Indice de la accion de autoborrado, si la hay.
+    let mut autoborrado: Option<usize> = None;
+    let mut exe_cubierto = false;
+
+    // Plugin y marketplace: de Claude Code, la persona (D3).
+    let plugins = plugins(c)?;
+
+    for dir in e.dirs_de_instalacion() {
+        let bin = dir.join(e.sistema.binario());
+        let hay_bin = bin.is_file();
+        let es_exe = hay_bin && misma_ruta(&bin, &e.exe);
+        exe_cubierto |= es_exe;
+        let en_uso = hay_bin
+            && windows
+            && c.maquina
+                .en_uso_por_otro(&bin, e.pid)
+                .map_err(|er| no_se_lee(&bin, &er))?;
+        let mut grupo: Vec<Accion> = Vec::new();
+        let path_antes = path.clone();
+        let mut temporales: Vec<PathBuf> = hijos(&dir)
+            .into_iter()
+            .filter(|p| {
+                p.file_name()
+                    .and_then(|n| n.to_str())
+                    .is_some_and(es_temporal)
+            })
+            .collect();
+        temporales.sort();
+        for p in temporales {
+            grupo.push(Accion::borra(t(&p), "temporal", Efecto::Fichero(p.clone())));
+            quitados.insert(p);
+        }
+        let old = ruta_old(&bin);
+        if old.is_file() {
+            grupo.push(Accion::borra(
+                t(&old),
+                "binario-old",
+                Efecto::Fichero(old.clone()),
+            ));
+            quitados.insert(old);
+        }
+        if hay_bin {
+            if es_exe && windows {
+                // El binario en marcha no se puede borrar en Windows: lo borra
+                // un proceso auxiliar al salir (D4).
+                autoborrado = Some(v.len() + grupo.len());
+                grupo.push(
+                    Accion::borra(
+                        t(&bin),
+                        "binario",
+                        Efecto::Autoborrado {
+                            ficheros: vec![bin.clone()],
+                            dirs: Vec::new(),
+                        },
+                    )
+                    .motivo("al-salir"),
+                );
+            } else {
+                grupo.push(Accion::borra(
+                    t(&bin),
+                    "binario",
+                    Efecto::Fichero(bin.clone()),
+                ));
+            }
+            quitados.insert(bin.clone());
+        }
+        if e.dir_propio(&dir) && dir.is_dir() && hijos(&dir).iter().all(|h| quitados.contains(h)) {
+            if let (true, Some(i)) = (es_exe && windows, autoborrado) {
+                if let Some(Accion {
+                    efecto: Efecto::Autoborrado { dirs, .. },
+                    ..
+                }) = grupo.get_mut(i - v.len())
+                {
+                    dirs.push(dir.clone());
+                }
+                grupo.push(
+                    Accion::nueva(t(&dir), "carpeta", "taskcodetwo", Decision::Borra)
+                        .motivo("al-salir"),
+                );
+            } else {
+                grupo.push(Accion::borra(
+                    t(&dir),
+                    "carpeta",
+                    Efecto::CarpetaSiVacia(dir.clone()),
+                ));
+            }
+            quitados.insert(dir.clone());
+        }
+        // La entrada del PATH de usuario (inversa de `Add-PathUsuario`).
+        if let Some(valor) = path.clone()
+            && let Some(nuevo) = quitar_del_path(&valor, &t(&dir), |x| expandir(c, x))
+        {
+            grupo.push(Accion::borra(
+                format!("HKCU\\Environment\\Path:{}", t(&dir)),
+                "path-usuario",
+                Efecto::PathUsuario {
+                    viejo: valor,
+                    nuevo: nuevo.clone(),
+                },
+            ));
+            path = Some(nuevo);
+        }
+        if en_uso {
+            // Otro proceso ejecuta este binario: nada de su grupo se toca
+            // (ni el PATH), para no dejarlo a medias (C8).
+            for a in &mut grupo {
+                a.decision = Decision::Pendiente;
+                a.motivo = Some("en-uso".to_owned());
+                a.efecto = Efecto::Ninguno;
+            }
+            if autoborrado.is_some_and(|i| i >= v.len()) {
+                autoborrado = None;
+            }
+            path = path_antes;
+        }
+        v.extend(grupo);
+    }
+
+    // El `tc2` que se ejecuta, si no es de un instalador.
+    if !exe_cubierto {
+        match e.modo() {
+            Modo::Plugin => {
+                let orden = plugins.first().map_or_else(
+                    || "claude plugin uninstall taskcodetwo".to_owned(),
+                    Clone::clone,
+                );
+                v.push(
+                    Accion::nueva(t(&e.exe), "binario", "plugin", Decision::Persona).orden(orden),
+                );
+            }
+            Modo::Gestor(g) => {
+                v.push(
+                    Accion::nueva(t(&e.exe), "binario", "gestor", Decision::Persona)
+                        .motivo(g)
+                        .orden(orden_del_gestor(g)),
+                );
+            }
+            Modo::Instalador => {
+                if e.exe.is_file() {
+                    v.push(
+                        Accion::nueva(t(&e.exe), "binario", "taskcodetwo", Decision::Persona)
+                            .motivo("fuera-de-los-instaladores")
+                            .orden(orden_borrar(e.sistema, &e.exe)),
+                    );
+                }
+            }
+        }
+    }
+
+    // Bloque de marcas de los perfiles del shell (Unix).
+    if !windows && let Some(home) = &e.home {
+        for nombre in PERFILES {
+            let p = home.join(nombre);
+            let Some(bytes) = leer(&p)? else { continue };
+            let Ok(texto) = String::from_utf8(bytes.clone()) else {
+                if bytes
+                    .windows(MARCA_INICIO_PERFIL.len())
+                    .any(|w| w == MARCA_INICIO_PERFIL.as_bytes())
+                {
+                    v.push(
+                        Accion::nueva(t(&p), "perfil", "taskcodetwo", Decision::Persona)
+                            .motivo("no-utf8"),
+                    );
+                }
+                continue;
+            };
+            match quitar_bloque(&texto) {
+                Ok(None) => {}
+                Ok(Some(nuevo)) => v.push(Accion::borra(
+                    t(&p),
+                    "perfil",
+                    Efecto::Reescribir {
+                        ruta: p.clone(),
+                        antes: bytes,
+                        despues: nuevo.into_bytes(),
+                    },
+                )),
+                Err(m) => v.push(
+                    Accion::nueva(t(&p), "perfil", "taskcodetwo", Decision::Persona)
+                        .motivo(m.motivo()),
+                ),
+            }
+        }
+    }
+
+    // Cache del lanzador (`taskcodetwo/cli/`, F1-14) y la carpeta raiz.
+    for raiz in e.raices_de_cache() {
+        let cli = raiz.join("cli");
+        if cli.is_dir() {
+            if e.modo() == Modo::Plugin && e.exe.starts_with(&cli) {
+                // La version que se ejecuta es del plugin: la quita la persona
+                // tras desinstalar el plugin; las demas, ya.
+                let mut versiones = hijos(&cli);
+                versiones.sort();
+                for ver in versiones {
+                    if e.exe.starts_with(&ver) {
+                        v.push(
+                            Accion::nueva(t(&ver), "cache-cli", "taskcodetwo", Decision::Persona)
+                                .motivo("en-uso-por-el-plugin")
+                                .orden(orden_borrar(e.sistema, &ver)),
+                        );
+                    } else {
+                        v.push(Accion::borra(
+                            t(&ver),
+                            "cache-cli",
+                            Efecto::Carpeta(ver.clone()),
+                        ));
+                    }
+                }
+            } else {
+                v.push(Accion::borra(
+                    t(&cli),
+                    "cache-cli",
+                    Efecto::Carpeta(cli.clone()),
+                ));
+                quitados.insert(cli.clone());
+            }
+        }
+        if raiz.is_dir() && hijos(&raiz).iter().all(|h| quitados.contains(h)) {
+            match autoborrado.and_then(|i| v.get_mut(i)) {
+                Some(Accion {
+                    efecto: Efecto::Autoborrado { dirs, .. },
+                    ..
+                }) if e.dir_por_defecto().is_some_and(|d| d.starts_with(&raiz)) => {
+                    dirs.push(raiz.clone());
+                    v.push(
+                        Accion::nueva(t(&raiz), "carpeta", "taskcodetwo", Decision::Borra)
+                            .motivo("al-salir"),
+                    );
+                }
+                _ => v.push(Accion::borra(
+                    t(&raiz),
+                    "carpeta",
+                    Efecto::CarpetaSiVacia(raiz.clone()),
+                )),
+            }
+        }
+    }
+
+    claude(c, v, plugins)
+}
+
+/// Ordenes `claude plugin uninstall` del plugin de la 2.0, una por scope en
+/// que este instalado, leidas de `plugins/installed_plugins.json` (solo
+/// lectura: el fichero es de Claude Code).
+fn plugins(c: &Contexto<'_>) -> Result<Vec<String>, Informe> {
+    let Some(dir) = c.entorno.dir_claude() else {
+        return Ok(Vec::new());
+    };
+    let p = dir.join("plugins").join("installed_plugins.json");
+    let Some(bytes) = leer(&p)? else {
+        return Ok(Vec::new());
+    };
+    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_default();
+    let mut ordenes = Vec::new();
+    if let Some(m) = v.get("plugins").and_then(serde_json::Value::as_object) {
+        for (clave, entradas) in m {
+            if clave.split('@').next() != Some(crate::huella::NOMBRE) {
+                continue;
+            }
+            let scopes: Vec<&str> = entradas
+                .as_array()
+                .map(|a| {
+                    a.iter()
+                        .filter_map(|x| x.get("scope").and_then(serde_json::Value::as_str))
+                        .collect()
+                })
+                .unwrap_or_default();
+            if scopes.is_empty() {
+                ordenes.push(format!("claude plugin uninstall {clave}"));
+            }
+            for s in scopes {
+                ordenes.push(format!("claude plugin uninstall {clave} --scope {s}"));
+            }
+        }
+    }
+    Ok(ordenes)
+}
+
+/// Lo de Claude Code: la status line (solo la de la 2.0, por su `command`
+/// exacto), los hooks que nombran `tc2` (aviso; nunca se editan) y el plugin
+/// (la persona) y el de la 0.16 (se conserva, F9-10).
+fn claude(c: &Contexto<'_>, v: &mut Vec<Accion>, plugins: Vec<String>) -> Result<(), Informe> {
+    let Some(dir) = c.entorno.dir_claude() else {
+        return Ok(());
+    };
+    let settings = dir.join("settings.json");
+    if let Some(bytes) = leer(&settings)? {
+        let texto = String::from_utf8(bytes.clone()).ok();
+        let valor = texto
+            .as_deref()
+            .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok());
+        match (texto, valor) {
+            (Some(texto), Some(valor)) => {
+                if let Some(sl) = valor.get("statusLine") {
+                    let cmd = sl
+                        .get("command")
+                        .and_then(serde_json::Value::as_str)
+                        .unwrap_or_default();
+                    let ruta = format!("{}#statusLine", t(&settings));
+                    if cmd.trim() == STATUS_LINE {
+                        match quitar_miembro(&texto, "statusLine") {
+                            Some(nuevo) => v.push(Accion::borra(
+                                ruta,
+                                "status-line",
+                                Efecto::Reescribir {
+                                    ruta: settings.clone(),
+                                    antes: bytes,
+                                    despues: nuevo.into_bytes(),
+                                },
+                            )),
+                            None => v.push(
+                                Accion::nueva(
+                                    ruta,
+                                    "status-line",
+                                    "taskcodetwo",
+                                    Decision::Persona,
+                                )
+                                .motivo("no-se-reescribe"),
+                            ),
+                        }
+                    } else {
+                        let dueno = if cmd.trim_start().starts_with("taskctl") {
+                            "taskcode-016"
+                        } else {
+                            "ajeno"
+                        };
+                        v.push(Accion::nueva(
+                            ruta,
+                            "status-line",
+                            dueno,
+                            Decision::Conserva,
+                        ));
+                    }
+                }
+                for _ in hooks_con_tc2(&valor) {
+                    v.push(
+                        Accion::nueva(
+                            format!("{}#hooks", t(&settings)),
+                            "hook",
+                            "ajeno",
+                            Decision::Conserva,
+                        )
+                        .motivo("nombra-tc2"),
+                    );
+                }
+            }
+            _ => v.push(
+                Accion::nueva(t(&settings), "settings", "claude-code", Decision::Conserva)
+                    .motivo("json-invalido"),
+            ),
+        }
+    }
+    for orden in plugins {
+        v.push(
+            Accion::nueva(
+                crate::huella::NOMBRE,
+                "plugin",
+                "claude-code",
+                Decision::Persona,
+            )
+            .orden(orden),
+        );
+    }
+    // El plugin de la 0.16 se nombra y no se toca (D6, F9-10).
+    let instalados = dir.join("plugins").join("installed_plugins.json");
+    if let Some(bytes) = leer(&instalados)? {
+        let val: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_default();
+        if let Some(m) = val.get("plugins").and_then(serde_json::Value::as_object) {
+            for clave in m
+                .keys()
+                .filter(|k| k.split('@').next() == Some("taskcode-plugin"))
+            {
+                v.push(
+                    Accion::nueva(clave.as_str(), "plugin", "taskcode-016", Decision::Conserva)
+                        .motivo("desinstalacion-f9-10"),
+                );
+            }
+        }
+    }
+    Ok(())
+}
diff --git a/crates/tc2/src/desinstalar/mod.rs b/crates/tc2/src/desinstalar/mod.rs
new file mode 100644
index 0000000..6587863
--- /dev/null
+++ b/crates/tc2/src/desinstalar/mod.rs
@@ -0,0 +1,641 @@
+//! `tc2 desinstalar` (F2-14, RR-16): quita TaskCodeTwo de forma limpia.
+//!
+//! Dos ambitos explicitos, `--maquina` y `--proyecto` (sin ninguno, `uso`).
+//! Cada uno hace un inventario de solo lectura con la huella encontrada y su
+//! dueno, calcula el plan (una [`Accion`] por ruta) y lo aplica tal cual.
+//! `--simular` imprime el plan y para (patron «plan como dato» de F1-12).
+//! Ningun paso usa la red.
+//!
+//! Solo se borra lo que un reconocedor cerrado da por propio de la 2.0
+//! (ruta exacta, marcas exactas o `command` exacto, con los nombres de
+//! [`crate::huella`] y de [`distribucion`]). Lo de Claude Code, un gestor de
+//! paquetes, otra herramienta o la 0.16 se lista, con la orden que lo quita
+//! si la hay, y no se toca (D3, D6). En un proyecto, la config solo con
+//! `--config` y la rama de estado local solo con `--borrar-tareas <rama>`;
+//! la remota nunca (D1, D2).
+//!
+//! Salida: una linea `accion ruta=... clase=... dueno=... decision=...` por
+//! accion y al final la linea de contrato `desinstalar <resultado> ...`.
+
+mod maquina;
+mod proyecto;
+pub mod puertos;
+pub mod texto;
+
+use std::path::{Path, PathBuf};
+
+use crate::Salida;
+use crate::contrato::{Informe, Resultado, cadena_json, escapar_valor};
+use crate::git::Git;
+use crate::i18n::catalogo::desinstalar as txt;
+use crate::i18n::{Idioma, Mensaje};
+use crate::instalacion::{Entorno, Sistema};
+pub use puertos::{Maquina, MaquinaEnMemoria, MaquinaReal};
+
+/// Nombre de la orden.
+pub const NOMBRE: &str = "desinstalar";
+
+/// Opciones de la orden.
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct Opciones {
+    /// Quitar lo de la maquina.
+    pub maquina: bool,
+    /// Quitar lo del proyecto de la carpeta actual.
+    pub proyecto: bool,
+    /// Solo imprimir el plan.
+    pub simular: bool,
+    /// Borrar tambien `.taskcode/config.yml` y `personas.yml`.
+    pub config: bool,
+    /// Borrar la rama local de estado (y las refs pendientes): el nombre
+    /// exacto de la rama, como confirmacion.
+    pub borrar_tareas: Option<String>,
+}
+
+/// Lee las opciones (sin `--json`).
+pub fn leer_opciones(args: &[String]) -> Result<Opciones, Informe> {
+    let uso = |motivo: &str| {
+        Informe::nuevo(NOMBRE, Resultado::Uso)
+            .con("motivo", motivo)
+            .diagnostico(&txt::USO_DESINSTALAR)
+    };
+    let mut op = Opciones::default();
+    let mut i = args.iter();
+    while let Some(a) = i.next() {
+        match a.as_str() {
+            "--maquina" if !op.maquina => op.maquina = true,
+            "--proyecto" if !op.proyecto => op.proyecto = true,
+            "--simular" if !op.simular => op.simular = true,
+            "--config" if !op.config => op.config = true,
+            "--borrar-tareas" if op.borrar_tareas.is_none() => {
+                let Some(r) = i.next() else {
+                    return Err(uso("sin-rama"));
+                };
+                op.borrar_tareas = Some(r.clone());
+            }
+            _ => return Err(uso("argumentos")),
+        }
+    }
+    if !op.maquina && !op.proyecto {
+        return Err(uso("sin-ambito"));
+    }
+    if (op.config || op.borrar_tareas.is_some()) && !op.proyecto {
+        return Err(uso("opcion-de-proyecto"));
+    }
+    if let Some(r) = &op.borrar_tareas
+        && r != crate::huella::RAMA_ESTADO
+    {
+        return Err(Informe::nuevo(NOMBRE, Resultado::Uso)
+            .con("motivo", "rama-no-coincide")
+            .diagnostico(
+                Mensaje::nuevo(&txt::NOMBRE_NO_COINCIDE)
+                    .con("rama", crate::huella::RAMA_ESTADO)
+                    .con("nombre", r.as_str()),
+            ));
+    }
+    Ok(op)
+}
+
+/// Que se decide hacer con una ruta.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Decision {
+    /// Es de la 2.0 y se quita.
+    Borra,
+    /// Se queda (de otro, o falta el permiso de la persona).
+    Conserva,
+    /// La quita la persona con la orden de la linea.
+    Persona,
+    /// Es de la 2.0 pero no se pudo quitar ahora (en uso, o un fallo).
+    Pendiente,
+}
+
+impl Decision {
+    /// Valor de `decision=`.
+    pub fn texto(self) -> &'static str {
+        match self {
+            Self::Borra => "borra",
+            Self::Conserva => "conserva",
+            Self::Persona => "persona",
+            Self::Pendiente => "pendiente",
+        }
+    }
+}
+
+/// Lo que hace una accion `borra` al aplicarse.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum Efecto {
+    /// Nada (lineas informativas).
+    Ninguno,
+    /// Borrar un fichero.
+    Fichero(PathBuf),
+    /// Borrar una carpeta con todo lo de dentro.
+    Carpeta(PathBuf),
+    /// Borrar una carpeta si ha quedado vacia.
+    CarpetaSiVacia(PathBuf),
+    /// Reescribir un fichero (si sigue siendo `antes`).
+    Reescribir {
+        /// El fichero.
+        ruta: PathBuf,
+        /// Lo que tenia al hacer el plan.
+        antes: Vec<u8>,
+        /// Lo que tendra.
+        despues: Vec<u8>,
+    },
+    /// Cambiar el PATH de usuario de Windows.
+    PathUsuario {
+        /// Valor al hacer el plan.
+        viejo: String,
+        /// Valor nuevo.
+        nuevo: String,
+    },
+    /// Borrar el binario en marcha tras salir (Windows).
+    Autoborrado {
+        /// Ficheros.
+        ficheros: Vec<PathBuf>,
+        /// Carpetas que se borran si quedan vacias.
+        dirs: Vec<PathBuf>,
+    },
+    /// `git worktree remove <ruta>` (sin forzar).
+    QuitarWorktree {
+        /// Worktree principal.
+        principal: PathBuf,
+        /// Ruta del worktree.
+        ruta: PathBuf,
+    },
+    /// `git worktree prune` (registros de worktrees ya borrados).
+    PodarWorktrees {
+        /// Worktree principal.
+        principal: PathBuf,
+    },
+    /// `git update-ref -d <ref> <sha>` de una ref propia.
+    BorrarRef {
+        /// Worktree principal.
+        principal: PathBuf,
+        /// La ref.
+        referencia: String,
+        /// Su valor al hacer el plan.
+        sha: String,
+    },
+    /// Borrar la rama local de estado (si sigue en `sha`).
+    BorrarRamaEstado {
+        /// Worktree principal.
+        principal: PathBuf,
+        /// Su valor al hacer el plan.
+        sha: String,
+    },
+}
+
+/// Una linea del plan.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Accion {
+    /// Ruta (o ref, o entrada del PATH).
+    pub ruta: String,
+    /// Que es: `binario`, `perfil`, `status-line`, `worktree-estado`...
+    pub clase: &'static str,
+    /// De quien es: `taskcodetwo`, `claude-code`, `taskcode-016`, `gestor`,
+    /// `ajeno`.
+    pub dueno: &'static str,
+    /// Que se hace.
+    pub decision: Decision,
+    /// Por que (si no es evidente).
+    pub motivo: Option<String>,
+    /// La orden que lo quita, cuando la decision es de la persona.
+    pub orden: Option<String>,
+    /// Si impide aplicar el plan (`rechazada` sin tocar nada).
+    pub bloquea: bool,
+    /// Lo que se hace al aplicar.
+    pub efecto: Efecto,
+}
+
+impl Accion {
+    /// Linea informativa sin efecto.
+    pub(crate) fn nueva(
+        ruta: impl Into<String>,
+        clase: &'static str,
+        dueno: &'static str,
+        decision: Decision,
+    ) -> Self {
+        Self {
+            ruta: ruta.into(),
+            clase,
+            dueno,
+            decision,
+            motivo: None,
+            orden: None,
+            bloquea: false,
+            efecto: Efecto::Ninguno,
+        }
+    }
+
+    /// Accion `borra` de la 2.0 con su efecto.
+    pub(crate) fn borra(ruta: impl Into<String>, clase: &'static str, efecto: Efecto) -> Self {
+        let mut a = Self::nueva(ruta, clase, "taskcodetwo", Decision::Borra);
+        a.efecto = efecto;
+        a
+    }
+
+    #[must_use]
+    pub(crate) fn motivo(mut self, m: impl Into<String>) -> Self {
+        self.motivo = Some(m.into());
+        self
+    }
+
+    #[must_use]
+    pub(crate) fn orden(mut self, o: impl Into<String>) -> Self {
+        self.orden = Some(o.into());
+        self
+    }
+
+    /// Linea `accion ruta=... clase=... dueno=... decision=...`.
+    pub fn linea(&self) -> String {
+        let mut s = format!(
+            "accion ruta={} clase={} dueno={} decision={}",
+            escapar_valor(&self.ruta, false),
+            self.clase,
+            self.dueno,
+            self.decision.texto()
+        );
+        for (k, v) in [("motivo", &self.motivo), ("orden", &self.orden)] {
+            if let Some(v) = v {
+                s.push_str(&format!(" {k}={}", escapar_valor(v, false)));
+            }
+        }
+        s
+    }
+
+    /// La misma linea en JSON.
+    pub fn json(&self) -> String {
+        let mut s = format!(
+            "{{\"accion\":{{\"ruta\":{},\"clase\":{},\"dueno\":{},\"decision\":{}",
+            cadena_json(&self.ruta),
+            cadena_json(self.clase),
+            cadena_json(self.dueno),
+            cadena_json(self.decision.texto())
+        );
+        for (k, v) in [("motivo", &self.motivo), ("orden", &self.orden)] {
+            if let Some(v) = v {
+                s.push_str(&format!(",\"{k}\":{}", cadena_json(v)));
+            }
+        }
+        s.push_str("}}");
+        s
+    }
+
+    /// Aviso para la persona por stderr (aunque la orden salga bien): la
+    /// orden que tiene que ejecutar, o el hook que nombra `tc2`.
+    fn aviso(&self) -> Option<Mensaje> {
+        if self.clase == "hook" {
+            return Some(Mensaje::nuevo(&txt::AVISO_HOOK).con("ruta", self.ruta.as_str()));
+        }
+        match (&self.decision, &self.orden) {
+            (Decision::Persona, Some(o)) => Some(
+                Mensaje::nuevo(&txt::HAZ_TU)
+                    .con("clase", self.clase)
+                    .con("ruta", self.ruta.as_str())
+                    .con("orden", o.as_str()),
+            ),
+            _ => None,
+        }
+    }
+}
+
+/// Lo que se necesita para desinstalar, inyectable en los tests.
+pub struct Contexto<'a> {
+    /// HOME, `LOCALAPPDATA`, el ejecutable...
+    pub entorno: &'a Entorno,
+    /// Carpeta del proyecto (para `--proyecto`).
+    pub dir: &'a Path,
+    /// Puerto Git.
+    pub git: &'a Git,
+    /// PATH de usuario, procesos y autoborrado.
+    pub maquina: &'a dyn Maquina,
+}
+
+/// Resultado: las acciones (ya aplicadas, salvo con `--simular`) y la linea
+/// final.
+#[derive(Debug, Clone)]
+pub struct Hecho {
+    /// Una por ruta.
+    pub acciones: Vec<Accion>,
+    /// La linea de contrato final.
+    pub informe: Informe,
+}
+
+impl Hecho {
+    fn solo(informe: Informe) -> Self {
+        Self {
+            acciones: Vec::new(),
+            informe,
+        }
+    }
+
+    /// La salida del proceso: las lineas de accion y la final; en stderr, el
+    /// diagnostico si fallo y los avisos para la persona.
+    pub fn salida_en(&self, json: bool, idioma: Idioma) -> Salida {
+        let fin = self.informe.salida_en(json, idioma);
+        let mut stdout = String::new();
+        let mut stderr = String::new();
+        for a in &self.acciones {
+            stdout.push_str(&if json { a.json() } else { a.linea() });
+            stdout.push('\n');
+            if let Some(m) = a.aviso() {
+                stderr.push_str(&m.texto(idioma));
+                stderr.push('\n');
+            }
+        }
+        stdout.push_str(&fin.stdout);
+        stderr.push_str(&fin.stderr);
+        Salida {
+            codigo: fin.codigo,
+            stdout,
+            stderr,
+        }
+    }
+}
+
+/// Orden `rm` para la persona, segun el sistema.
+pub(crate) fn orden_borrar(sistema: Sistema, ruta: &Path) -> String {
+    let r = ruta.display().to_string();
+    match sistema {
+        Sistema::Windows => format!(
+            "Remove-Item -LiteralPath '{}' -Recurse -Force",
+            r.replace('\'', "''")
+        ),
+        Sistema::Mac | Sistema::Linux => format!("rm -rf '{}'", r.replace('\'', "'\\''")),
+    }
+}
+
+/// Ruta como texto para la linea.
+pub(crate) fn t(p: &Path) -> String {
+    p.display().to_string()
+}
+
+/// Lee un fichero; `None` si no existe.
+pub(crate) fn leer(p: &Path) -> Result<Option<Vec<u8>>, Informe> {
+    match std::fs::read(p) {
+        Ok(b) => Ok(Some(b)),
+        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
+        Err(e) => Err(no_se_lee(p, &e)),
+    }
+}
+
+/// Informe de error al leer `ruta`.
+pub(crate) fn no_se_lee(ruta: &Path, e: &std::io::Error) -> Informe {
+    Informe::nuevo(NOMBRE, Resultado::Error)
+        .con("ruta", ruta.display().to_string())
+        .diagnostico(
+            Mensaje::nuevo(&txt::NO_SE_LEE)
+                .con("ruta", ruta.display().to_string())
+                .con("error", e.to_string()),
+        )
+}
+
+/// El plan completo: primero el proyecto y luego la maquina (el orden
+/// recomendado).
+pub fn plan(op: &Opciones, c: &Contexto<'_>) -> Result<Vec<Accion>, Informe> {
+    let mut v = Vec::new();
+    if op.proyecto {
+        proyecto::plan(op, c, &mut v)?;
+    }
+    if op.maquina {
+        maquina::plan(c, &mut v)?;
+    }
+    Ok(v)
+}
+
+/// `desinstalar` con las opciones ya leidas.
+pub fn desinstalar(op: &Opciones, c: &Contexto<'_>) -> Hecho {
+    let ambito = match (op.proyecto, op.maquina) {
+        (true, true) => "proyecto,maquina",
+        (true, false) => "proyecto",
+        _ => "maquina",
+    };
+    let mut acciones = match plan(op, c) {
+        Ok(v) => v,
+        Err(i) => return Hecho::solo(i),
+    };
+    let base = |r: Resultado, acciones: &[Accion]| {
+        let n = |d: Decision| acciones.iter().filter(|a| a.decision == d).count();
+        let mut i = Informe::nuevo(NOMBRE, r).con("ambito", ambito);
+        if op.simular {
+            i = i.con("simular", "si");
+        }
+        i.con("borra", n(Decision::Borra).to_string())
+            .con("conserva", n(Decision::Conserva).to_string())
+            .con("persona", n(Decision::Persona).to_string())
+            .con("pendiente", n(Decision::Pendiente).to_string())
+    };
+    if let Some(b) = acciones.iter().find(|a| a.bloquea) {
+        let motivo = b.motivo.clone().unwrap_or_default();
+        let i = base(Resultado::Rechazada, &acciones)
+            .con("motivo", motivo.clone())
+            .con("ruta", b.ruta.clone())
+            .diagnostico(
+                Mensaje::nuevo(&txt::BLOQUEADA)
+                    .con("ruta", b.ruta.clone())
+                    .con("motivo", motivo),
+            );
+        return Hecho {
+            acciones,
+            informe: i,
+        };
+    }
+    if !op.simular {
+        if op.proyecto
+            && let Err(i) = proyecto::cerrojo_libre(c)
+        {
+            return Hecho {
+                acciones,
+                informe: i,
+            };
+        }
+        let mut fallo: Option<Informe> = None;
+        for a in acciones.iter_mut() {
+            if a.decision != Decision::Borra {
+                continue;
+            }
+            if fallo.is_some() {
+                a.decision = Decision::Pendiente;
+                a.motivo = Some("no-aplicada".to_owned());
+                continue;
+            }
+            if let Err(e) = aplicar(&a.efecto, c) {
+                a.decision = Decision::Pendiente;
+                a.motivo = Some("error".to_owned());
+                fallo = Some(
+                    Informe::nuevo(NOMBRE, Resultado::Error)
+                        .con("ruta", a.ruta.clone())
+                        .diagnostico(
+                            Mensaje::nuevo(&txt::FALLO_AL_APLICAR)
+                                .con("ruta", a.ruta.clone())
+                                .con("error", e),
+                        ),
+                );
+            }
+        }
+        if let Some(i) = fallo {
+            let mut f = base(Resultado::Error, &acciones);
+            f.claves.extend(i.claves);
+            f.diagnostico = i.diagnostico;
+            return Hecho {
+                acciones,
+                informe: f,
+            };
+        }
+    }
+    let en_uso = acciones
+        .iter()
+        .find(|a| a.decision == Decision::Pendiente && a.motivo.as_deref() == Some("en-uso"));
+    let informe = if let Some(a) = en_uso {
+        let ruta = a.ruta.clone();
+        base(Resultado::Rechazada, &acciones)
+            .con("motivo", "en-uso")
+            .con("ruta", ruta.clone())
+            .diagnostico(Mensaje::nuevo(&txt::EN_USO).con("ruta", ruta))
+    } else if acciones.iter().any(|a| a.decision == Decision::Borra) {
+        base(Resultado::Ok, &acciones)
+    } else {
+        base(Resultado::SinCambios, &acciones)
+    };
+    Hecho { acciones, informe }
+}
+
+/// Aplica un efecto. `Err` con el texto del fallo.
+fn aplicar(e: &Efecto, c: &Contexto<'_>) -> Result<(), String> {
+    let no_existe = |r: std::io::Result<()>| match r {
+        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
+        otro => otro.map_err(|e| e.to_string()),
+    };
+    match e {
+        Efecto::Ninguno => Ok(()),
+        Efecto::Fichero(p) => no_existe(std::fs::remove_file(p)),
+        Efecto::Carpeta(p) => no_existe(std::fs::remove_dir_all(p)),
+        Efecto::CarpetaSiVacia(p) => match std::fs::read_dir(p).map(|mut d| d.next().is_none()) {
+            Ok(true) => no_existe(std::fs::remove_dir(p)),
+            // No ha quedado vacia (algo nuevo de la persona): se queda.
+            Ok(false) => Ok(()),
+            Err(e) => no_existe(Err(e)),
+        },
+        Efecto::Reescribir {
+            ruta,
+            antes,
+            despues,
+        } => {
+            // Si la persona lo cambio despues del plan, no se pisa.
+            let ahora = std::fs::read(ruta).map_err(|e| e.to_string())?;
+            if &ahora != antes {
+                return Err("cambio desde el plan".to_owned());
+            }
+            std::fs::write(ruta, despues).map_err(|e| e.to_string())
+        }
+        Efecto::PathUsuario { viejo, nuevo } => c
+            .maquina
+            .escribir_path_usuario(viejo, nuevo)
+            .map_err(|e| e.to_string()),
+        Efecto::Autoborrado { ficheros, dirs } => c
+            .maquina
+            .programar_borrado(c.entorno.pid, ficheros, dirs)
+            .map_err(|e| e.to_string()),
+        Efecto::QuitarWorktree { principal, ruta } => {
+            proyecto::quitar_worktree(c.git, principal, ruta)
+        }
+        Efecto::PodarWorktrees { principal } => proyecto::podar(c.git, principal),
+        Efecto::BorrarRef {
+            principal,
+            referencia,
+            sha,
+        } => proyecto::borrar_ref(c.git, principal, referencia, sha),
+        Efecto::BorrarRamaEstado { principal, sha } => proyecto::borrar_rama(c.git, principal, sha),
+    }
+}
+
+/// La orden completa desde la linea de ordenes, en la carpeta actual y con
+/// el entorno y la maquina de verdad.
+pub fn orden(args: &[String], json: bool, idioma: Idioma) -> Salida {
+    let op = match leer_opciones(args) {
+        Ok(o) => o,
+        Err(i) => return i.salida_en(json, idioma),
+    };
+    let entorno = match Entorno::del_proceso() {
+        Ok(e) => e,
+        Err(e) => {
+            return Informe::nuevo(NOMBRE, Resultado::Error)
+                .diagnostico(Mensaje::nuevo(&txt::SIN_ENTORNO).con("error", e.to_string()))
+                .salida_en(json, idioma);
+        }
+    };
+    let dir = match std::env::current_dir() {
+        Ok(d) => d,
+        Err(e) => {
+            return Informe::nuevo(NOMBRE, Resultado::Error)
+                .diagnostico(
+                    Mensaje::nuevo(&crate::i18n::catalogo::cli::CARPETA_ACTUAL)
+                        .con("error", e.to_string()),
+                )
+                .salida_en(json, idioma);
+        }
+    };
+    let git = Git::nuevo();
+    let maquina = MaquinaReal;
+    let c = Contexto {
+        entorno: &entorno,
+        dir: &dir,
+        git: &git,
+        maquina: &maquina,
+    };
+    desinstalar(&op, &c).salida_en(json, idioma)
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn opciones() {
+        let v = |a: &[&str]| a.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
+        assert_eq!(
+            leer_opciones(&v(&["--proyecto", "--borrar-tareas", "taskcodetwo/tareas"])),
+            Ok(Opciones {
+                proyecto: true,
+                borrar_tareas: Some("taskcodetwo/tareas".into()),
+                ..Opciones::default()
+            })
+        );
+        for (args, motivo) in [
+            (v(&[]), "sin-ambito"),
+            (v(&["--simular"]), "sin-ambito"),
+            (v(&["--maquina", "--maquina"]), "argumentos"),
+            (v(&["--maquina", "--config"]), "opcion-de-proyecto"),
+            (v(&["--proyecto", "--borrar-tareas"]), "sin-rama"),
+            (
+                v(&["--proyecto", "--borrar-tareas", "tareas"]),
+                "rama-no-coincide",
+            ),
+            (v(&["--proyecto", "x"]), "argumentos"),
+        ] {
+            let i = leer_opciones(&args).err();
+            assert_eq!(
+                i.as_ref().map(|i| (i.resultado, i.valor("motivo"))),
+                Some((Resultado::Uso, Some(motivo))),
+                "{args:?}"
+            );
+        }
+    }
+
+    #[test]
+    fn linea_y_json_de_una_accion() {
+        let a = Accion::nueva("C:/a b", "plugin", "claude-code", Decision::Persona)
+            .orden("claude plugin uninstall taskcodetwo@m");
+        assert_eq!(
+            a.linea(),
+            "accion ruta=C:/a%20b clase=plugin dueno=claude-code decision=persona orden=claude%20plugin%20uninstall%20taskcodetwo@m"
+        );
+        assert_eq!(
+            a.json(),
+            "{\"accion\":{\"ruta\":\"C:/a b\",\"clase\":\"plugin\",\"dueno\":\"claude-code\",\"decision\":\"persona\",\"orden\":\"claude plugin uninstall taskcodetwo@m\"}}"
+        );
+        assert_eq!(
+            orden_borrar(Sistema::Linux, Path::new("/a'b")),
+            "rm -rf '/a'\\''b'"
+        );
+    }
+}
diff --git a/crates/tc2/src/desinstalar/proyecto.rs b/crates/tc2/src/desinstalar/proyecto.rs
new file mode 100644
index 0000000..420a278
--- /dev/null
+++ b/crates/tc2/src/desinstalar/proyecto.rs
@@ -0,0 +1,340 @@
+//! Inventario y plan de `desinstalar --proyecto`: los worktrees de la 2.0
+//! (estado y tareas), su carpeta, la carpeta propia en la comun de git, las
+//! lineas de `info/exclude`, las refs propias, la rama local de estado y la
+//! configuracion. Los nombres salen de [`crate::huella`].
+
+use std::path::{Path, PathBuf};
+use std::time::Duration;
+
+use super::texto::hooks_con_tc2;
+use super::{Accion, Contexto, Decision, Efecto, NOMBRE, Opciones, leer, t};
+use crate::contrato::{Informe, Resultado};
+use crate::estado::{CARPETA, Cerrojo};
+use crate::git::estado::dir_comun;
+use crate::git::rama::renombrar;
+use crate::git::{
+    CARPETA_WORKTREES, Git, REMOTO, Subcomando, commit_de, es_ancestro, exigir,
+    localizar_principal, misma_ruta, worktrees,
+};
+use crate::huella::{self, RAMA_ESTADO};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::desinstalar as txt;
+
+/// Lineas que la 2.0 escribe en `info/exclude` (`git::rama::excluir`).
+fn lineas_exclude() -> [String; 2] {
+    [format!("/{CARPETA_WORKTREES}/"), format!("/{CARPETA}/")]
+}
+
+/// Por que un worktree no se puede quitar sin perder nada, o `None`.
+fn bloqueo(
+    git: &Git,
+    principal: &Path,
+    ruta: &Path,
+    rama: Option<&str>,
+    de_tarea: bool,
+) -> Result<Option<&'static str>, Informe> {
+    let status = exigir(
+        git,
+        ruta,
+        NOMBRE,
+        Subcomando::Status,
+        &["--porcelain", "-z"],
+    )?;
+    if !status.is_empty() {
+        return Ok(Some("cambios-sin-guardar"));
+    }
+    if !de_tarea {
+        // La rama de estado se conserva (salvo --borrar-tareas): sus commits
+        // no se pierden al quitar el worktree.
+        return Ok(None);
+    }
+    let Some(rama) = rama else {
+        return Ok(Some("sin-rama"));
+    };
+    let publicada = match commit_de(
+        git,
+        principal,
+        NOMBRE,
+        &format!("refs/remotes/{REMOTO}/{rama}"),
+    )? {
+        Some(remota) => es_ancestro(git, ruta, NOMBRE, "HEAD", &remota)?,
+        None => false,
+    };
+    Ok((!publicada).then_some("sin-publicar"))
+}
+
+pub(super) fn plan(op: &Opciones, c: &Contexto<'_>, v: &mut Vec<Accion>) -> Result<(), Informe> {
+    let g = c.git;
+    let principal = localizar_principal(g, c.dir).map_err(|i| renombrar(i, NOMBRE))?;
+    let comun = dir_comun(g, &principal, NOMBRE)?;
+    let carpeta_wt = principal.join(CARPETA_WORKTREES);
+    let mut podar = false;
+    let mut quitados: Vec<PathBuf> = Vec::new();
+    for w in worktrees(g, &principal, NOMBRE)?.iter().skip(1) {
+        let de_estado = w.rama.as_deref() == Some(RAMA_ESTADO);
+        let de_tarea = w.ruta.parent().is_some_and(|p| misma_ruta(p, &carpeta_wt));
+        if !de_estado && !de_tarea {
+            continue;
+        }
+        if !w.ruta.exists() {
+            podar = true;
+            continue;
+        }
+        let clase = if de_estado {
+            "worktree-estado"
+        } else {
+            "worktree-tarea"
+        };
+        match bloqueo(g, &principal, &w.ruta, w.rama.as_deref(), de_tarea)? {
+            Some(m) => {
+                let mut a =
+                    Accion::nueva(t(&w.ruta), clase, "taskcodetwo", Decision::Conserva).motivo(m);
+                a.bloquea = true;
+                v.push(a);
+            }
+            None => {
+                quitados.push(w.ruta.clone());
+                v.push(Accion::borra(
+                    t(&w.ruta),
+                    clase,
+                    Efecto::QuitarWorktree {
+                        principal: principal.clone(),
+                        ruta: w.ruta.clone(),
+                    },
+                ));
+            }
+        }
+    }
+    if podar {
+        v.push(Accion::borra(
+            t(&comun.join("worktrees")),
+            "registro-worktree",
+            Efecto::PodarWorktrees {
+                principal: principal.clone(),
+            },
+        ));
+    }
+    // `.taskcodetwo/worktrees/` y `.taskcodetwo/`, si se quedan vacias.
+    let raiz_wt = carpeta_wt.parent().map(Path::to_path_buf);
+    for d in [Some(carpeta_wt.clone()), raiz_wt].into_iter().flatten() {
+        if !d.is_dir() {
+            continue;
+        }
+        let todo_se_va = std::fs::read_dir(&d)
+            .map(|it| {
+                it.flatten()
+                    .all(|h| quitados.iter().any(|q| misma_ruta(q, &h.path())))
+            })
+            .unwrap_or(false);
+        if todo_se_va {
+            quitados.push(d.clone());
+            v.push(Accion::borra(
+                t(&d),
+                "carpeta",
+                Efecto::CarpetaSiVacia(d.clone()),
+            ));
+        } else {
+            v.push(
+                Accion::nueva(t(&d), "carpeta", "taskcodetwo", Decision::Conserva)
+                    .motivo("no-vacia"),
+            );
+        }
+    }
+    // Lineas de `info/exclude` (inversa de `git::rama::excluir`).
+    let exclude = comun.join("info").join("exclude");
+    if let Some(bytes) = leer(&exclude)?
+        && let Ok(texto) = String::from_utf8(bytes.clone())
+    {
+        let propias = lineas_exclude();
+        let nuevo: String = texto
+            .split_inclusive('\n')
+            .filter(|l| !propias.iter().any(|p| l.trim() == p))
+            .collect();
+        if nuevo != texto {
+            v.push(Accion::borra(
+                t(&exclude),
+                "exclude",
+                Efecto::Reescribir {
+                    ruta: exclude.clone(),
+                    antes: bytes,
+                    despues: nuevo.into_bytes(),
+                },
+            ));
+        }
+    }
+    // La carpeta propia en la comun: cerrojo, cache de identidad, indices.
+    let propia = huella::en_comun(&comun);
+    if propia.is_dir() {
+        v.push(Accion::borra(
+            t(&propia),
+            "carpeta-comun",
+            Efecto::Carpeta(propia.clone()),
+        ));
+    }
+    // Refs propias (operaciones pendientes y rechazadas): son tareas sin
+    // publicar, asi que van con la rama (C7).
+    let refs = exigir(
+        g,
+        &principal,
+        NOMBRE,
+        Subcomando::ForEachRef,
+        &["--format=%(refname) %(objectname)", huella::PREFIJO_REFS],
+    )?;
+    for l in refs.lines() {
+        let Some((r, sha)) = l.split_once(' ') else {
+            continue;
+        };
+        if op.borrar_tareas.is_some() {
+            v.push(Accion::borra(
+                r,
+                "ref-propia",
+                Efecto::BorrarRef {
+                    principal: principal.clone(),
+                    referencia: r.to_owned(),
+                    sha: sha.to_owned(),
+                },
+            ));
+        } else {
+            v.push(
+                Accion::nueva(r, "ref-propia", "taskcodetwo", Decision::Conserva)
+                    .motivo("sin-borrar-tareas"),
+            );
+        }
+    }
+    // La rama local de estado: solo con --borrar-tareas <nombre exacto>.
+    let local = format!("refs/heads/{RAMA_ESTADO}");
+    if let Some(sha) = commit_de(g, &principal, NOMBRE, &local)? {
+        if op.borrar_tareas.is_some() {
+            v.push(Accion::borra(
+                local,
+                "rama-estado",
+                Efecto::BorrarRamaEstado {
+                    principal: principal.clone(),
+                    sha,
+                },
+            ));
+        } else {
+            v.push(
+                Accion::nueva(local, "rama-estado", "taskcodetwo", Decision::Conserva)
+                    .motivo("sin-borrar-tareas"),
+            );
+        }
+    }
+    // La remota nunca (D2): con --borrar-tareas, la orden para la persona.
+    let remota = format!("refs/remotes/{REMOTO}/{RAMA_ESTADO}");
+    if commit_de(g, &principal, NOMBRE, &remota)?.is_some() {
+        let a = Accion::nueva(remota, "rama-remota", "taskcodetwo", Decision::Conserva);
+        v.push(if op.borrar_tareas.is_some() {
+            let mut a = a.orden(format!("git push {REMOTO} --delete {RAMA_ESTADO}"));
+            a.decision = Decision::Persona;
+            a
+        } else {
+            a.motivo("remota")
+        });
+    }
+    // Configuracion del proyecto: solo con --config (C6). La de la 0.16
+    // (sin `formato: 2`) no es de la 2.0: se queda siempre (D6).
+    let config = principal.join(crate::config::RUTA);
+    if let Some(bytes) = leer(&config)? {
+        let de_la_2 = String::from_utf8_lossy(&bytes)
+            .lines()
+            .any(|l| l.trim() == "formato: 2");
+        v.push(match (de_la_2, op.config) {
+            (false, _) => Accion::nueva(t(&config), "config", "taskcode-016", Decision::Conserva),
+            (true, true) => Accion::borra(t(&config), "config", Efecto::Fichero(config.clone())),
+            (true, false) => Accion::nueva(t(&config), "config", "taskcodetwo", Decision::Conserva)
+                .motivo("sin-config"),
+        });
+    }
+    let personas = principal.join(crate::plataforma::personas::RUTA);
+    if personas.is_file() {
+        v.push(if op.config {
+            Accion::borra(t(&personas), "config", Efecto::Fichero(personas.clone()))
+        } else {
+            Accion::nueva(t(&personas), "config", "taskcodetwo", Decision::Conserva)
+                .motivo("sin-config")
+        });
+    }
+    // Hooks del proyecto que nombran tc2: aviso, nunca se editan (C4).
+    for nombre in ["settings.json", "settings.local.json"] {
+        let p = principal.join(".claude").join(nombre);
+        if let Some(bytes) = leer(&p)?
+            && let Ok(valor) = serde_json::from_slice::<serde_json::Value>(&bytes)
+        {
+            for _ in hooks_con_tc2(&valor) {
+                v.push(
+                    Accion::nueva(
+                        format!("{}#hooks", t(&p)),
+                        "hook",
+                        "ajeno",
+                        Decision::Conserva,
+                    )
+                    .motivo("nombra-tc2"),
+                );
+            }
+        }
+    }
+    Ok(())
+}
+
+/// Antes de aplicar: que ningun otro `tc2` este escribiendo el estado (el
+/// mismo cerrojo que la transaccion). Solo si la carpeta propia existe: si
+/// no, nadie lo tiene y no se crea.
+pub(super) fn cerrojo_libre(c: &Contexto<'_>) -> Result<(), Informe> {
+    let principal = localizar_principal(c.git, c.dir).map_err(|i| renombrar(i, NOMBRE))?;
+    let comun = dir_comun(c.git, &principal, NOMBRE)?;
+    if !huella::en_comun(&comun)
+        .join(huella::FICHERO_CERROJO)
+        .is_file()
+    {
+        return Ok(());
+    }
+    match Cerrojo::tomar(&comun, Duration::ZERO) {
+        Ok(_) => Ok(()),
+        Err(_) => Err(Informe::nuevo(NOMBRE, Resultado::Rechazada)
+            .con("motivo", "cerrojo")
+            .diagnostico(Mensaje::nuevo(&txt::CERROJO_OCUPADO))),
+    }
+}
+
+/// Texto de un informe de error de git, para la linea de fallo.
+fn texto(i: &Informe) -> String {
+    i.diagnostico_en(crate::i18n::Idioma::Es)
+        .unwrap_or_else(|| i.linea())
+}
+
+pub(super) fn quitar_worktree(git: &Git, principal: &Path, ruta: &Path) -> Result<(), String> {
+    let r = ruta.to_str().ok_or_else(|| t(ruta))?;
+    exigir(git, principal, NOMBRE, Subcomando::Worktree, &["remove", r])
+        .map(|_| ())
+        .map_err(|i| texto(&i))
+}
+
+pub(super) fn podar(git: &Git, principal: &Path) -> Result<(), String> {
+    exigir(git, principal, NOMBRE, Subcomando::Worktree, &["prune"])
+        .map(|_| ())
+        .map_err(|i| texto(&i))
+}
+
+pub(super) fn borrar_ref(git: &Git, principal: &Path, r: &str, sha: &str) -> Result<(), String> {
+    exigir(
+        git,
+        principal,
+        NOMBRE,
+        Subcomando::UpdateRef,
+        &["-d", r, sha],
+    )
+    .map(|_| ())
+    .map_err(|i| texto(&i))
+}
+
+pub(super) fn borrar_rama(git: &Git, principal: &Path, sha: &str) -> Result<(), String> {
+    let s = git
+        .borrar_rama_de_estado(principal, sha)
+        .map_err(|e| e.to_string())?;
+    if s.exito() {
+        Ok(())
+    } else {
+        Err(s.primera_linea_error())
+    }
+}
diff --git a/crates/tc2/src/desinstalar/puertos.rs b/crates/tc2/src/desinstalar/puertos.rs
new file mode 100644
index 0000000..5af475e
--- /dev/null
+++ b/crates/tc2/src/desinstalar/puertos.rs
@@ -0,0 +1,227 @@
+//! Puerto de lo que `desinstalar --maquina` toca fuera del sistema de
+//! ficheros: el PATH de usuario del registro de Windows, si otro proceso usa
+//! un binario y el borrado del binario en marcha tras salir (F2-14, D4 y D5).
+//!
+//! [`MaquinaReal`] lo hace con un hijo `powershell.exe` (solo Windows; en
+//! Unix no hay registro y un binario en marcha se puede borrar). Los tests
+//! usan [`MaquinaEnMemoria`]: nunca tocan el registro ni lanzan nada.
+
+use std::cell::RefCell;
+use std::io;
+use std::path::{Path, PathBuf};
+
+/// Lo que `desinstalar` necesita de la maquina.
+pub trait Maquina {
+    /// Valor crudo (sin expandir `%VARIABLES%`) del PATH de usuario, o
+    /// `None` si no hay (o no es Windows).
+    fn leer_path_usuario(&self) -> io::Result<Option<String>>;
+    /// Escribe `nuevo` en el PATH de usuario si sigue valiendo `viejo` (si
+    /// no, error: otro lo cambio entre medias), con el mismo tipo de valor.
+    fn escribir_path_usuario(&self, viejo: &str, nuevo: &str) -> io::Result<()>;
+    /// Si otro proceso distinto de `pid` ejecuta `ruta`.
+    fn en_uso_por_otro(&self, ruta: &Path, pid: u32) -> io::Result<bool>;
+    /// Deja un proceso auxiliar que, cuando `pid` acabe (como mucho 10 s),
+    /// borra `ficheros` y despues las carpetas de `dirs` que queden vacias.
+    fn programar_borrado(&self, pid: u32, ficheros: &[PathBuf], dirs: &[PathBuf])
+    -> io::Result<()>;
+}
+
+/// Una peticion de autoborrado registrada por [`MaquinaEnMemoria`].
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Borrado {
+    /// Proceso al que se espera.
+    pub pid: u32,
+    /// Ficheros que se borran.
+    pub ficheros: Vec<PathBuf>,
+    /// Carpetas que se borran si quedan vacias.
+    pub dirs: Vec<PathBuf>,
+}
+
+/// Maquina falsa para los tests: un PATH de usuario en memoria, una lista
+/// de binarios «en uso por otro» y el registro de los autoborrados.
+#[derive(Debug, Default)]
+pub struct MaquinaEnMemoria {
+    /// PATH de usuario.
+    pub path: RefCell<Option<String>>,
+    /// Binarios que otro proceso esta ejecutando.
+    pub en_uso: Vec<PathBuf>,
+    /// Autoborrados pedidos.
+    pub borrados: RefCell<Vec<Borrado>>,
+}
+
+impl Maquina for MaquinaEnMemoria {
+    fn leer_path_usuario(&self) -> io::Result<Option<String>> {
+        Ok(self.path.borrow().clone())
+    }
+
+    fn escribir_path_usuario(&self, viejo: &str, nuevo: &str) -> io::Result<()> {
+        let mut p = self.path.borrow_mut();
+        if p.as_deref() != Some(viejo) {
+            return Err(io::Error::other("el PATH cambio entre medias"));
+        }
+        *p = Some(nuevo.to_owned());
+        Ok(())
+    }
+
+    fn en_uso_por_otro(&self, ruta: &Path, _pid: u32) -> io::Result<bool> {
+        Ok(self.en_uso.iter().any(|r| r == ruta))
+    }
+
+    fn programar_borrado(
+        &self,
+        pid: u32,
+        ficheros: &[PathBuf],
+        dirs: &[PathBuf],
+    ) -> io::Result<()> {
+        self.borrados.borrow_mut().push(Borrado {
+            pid,
+            ficheros: ficheros.to_vec(),
+            dirs: dirs.to_vec(),
+        });
+        Ok(())
+    }
+}
+
+/// La maquina de verdad.
+#[derive(Debug, Default)]
+pub struct MaquinaReal;
+
+/// Inversa de `Add-PathUsuario` de `install.ps1`: lee el valor crudo, sin
+/// expandir las `%VARIABLES%`.
+#[cfg(windows)]
+const PS_LEER_PATH: &str = "[Console]::OutputEncoding = [Text.Encoding]::UTF8; $k = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment'); if ($null -eq $k) { exit 3 }; $v = $k.GetValue('Path', $null, [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames); $k.Close(); if ($null -eq $v) { exit 3 }; [Console]::Out.Write([string]$v)";
+
+/// Escribe el PATH nuevo con el mismo tipo de valor si sigue siendo el
+/// viejo, y avisa a las ventanas abiertas como el instalador.
+#[cfg(windows)]
+const PS_ESCRIBIR_PATH: &str = "$ErrorActionPreference = 'Stop'; $k = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment', $true); try { $actual = [string]$k.GetValue('Path', '', [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames); if ($actual -cne $env:TC2_PATH_VIEJO) { exit 4 }; $tipo = [Microsoft.Win32.RegistryValueKind]::ExpandString; if ($k.GetValueNames() -contains 'Path') { $tipo = $k.GetValueKind('Path') }; $k.SetValue('Path', $env:TC2_PATH_NUEVO, $tipo) } finally { $k.Close() }; [Environment]::SetEnvironmentVariable('TC2_INSTALAR_AVISO', '1', 'User'); [Environment]::SetEnvironmentVariable('TC2_INSTALAR_AVISO', $null, 'User')";
+
+/// Cuenta los procesos distintos de `TC2_PID` que ejecutan `TC2_RUTA`.
+#[cfg(windows)]
+const PS_EN_USO: &str = "$n = @(Get-CimInstance Win32_Process | Where-Object { $_.ExecutablePath -and $_.ExecutablePath -ieq $env:TC2_RUTA -and $_.ProcessId -ne [int]$env:TC2_PID }).Count; [Console]::Out.Write($n)";
+
+/// Espera al proceso (como mucho 10 s) y borra; reintenta unos segundos si
+/// un antivirus tiene el fichero abierto un momento.
+#[cfg(windows)]
+const PS_AUTOBORRADO: &str = "try { Wait-Process -Id ([int]$env:TC2_PID) -Timeout 10 -ErrorAction Stop } catch {}; foreach ($f in ($env:TC2_FICHEROS -split \"`n\")) { if (-not $f) { continue }; for ($i = 0; $i -lt 20; $i++) { if (-not (Test-Path -LiteralPath $f)) { break }; try { Remove-Item -LiteralPath $f -Force -ErrorAction Stop; break } catch { Start-Sleep -Milliseconds 250 } } }; foreach ($d in ($env:TC2_DIRS -split \"`n\")) { if ($d -and (Test-Path -LiteralPath $d) -and -not (Get-ChildItem -LiteralPath $d -Force)) { Remove-Item -LiteralPath $d -Force -ErrorAction SilentlyContinue } }";
+
+#[cfg(windows)]
+fn powershell(script: &str, vars: &[(&str, &std::ffi::OsStr)]) -> io::Result<std::process::Output> {
+    let mut c = std::process::Command::new("powershell.exe");
+    c.args(["-NoProfile", "-NonInteractive", "-Command", script])
+        .stdin(std::process::Stdio::null());
+    for (k, v) in vars {
+        c.env(k, v);
+    }
+    c.output()
+}
+
+#[cfg(windows)]
+impl Maquina for MaquinaReal {
+    fn leer_path_usuario(&self) -> io::Result<Option<String>> {
+        let o = powershell(PS_LEER_PATH, &[])?;
+        match o.status.code() {
+            Some(0) => Ok(Some(String::from_utf8_lossy(&o.stdout).into_owned())),
+            Some(3) => Ok(None),
+            _ => Err(io::Error::other(
+                String::from_utf8_lossy(&o.stderr).trim().to_owned(),
+            )),
+        }
+    }
+
+    fn escribir_path_usuario(&self, viejo: &str, nuevo: &str) -> io::Result<()> {
+        let o = powershell(
+            PS_ESCRIBIR_PATH,
+            &[
+                ("TC2_PATH_VIEJO", viejo.as_ref()),
+                ("TC2_PATH_NUEVO", nuevo.as_ref()),
+            ],
+        )?;
+        match o.status.code() {
+            Some(0) => Ok(()),
+            Some(4) => Err(io::Error::other("el PATH de usuario cambio entre medias")),
+            _ => Err(io::Error::other(
+                String::from_utf8_lossy(&o.stderr).trim().to_owned(),
+            )),
+        }
+    }
+
+    fn en_uso_por_otro(&self, ruta: &Path, pid: u32) -> io::Result<bool> {
+        let pid = pid.to_string();
+        let o = powershell(
+            PS_EN_USO,
+            &[("TC2_RUTA", ruta.as_os_str()), ("TC2_PID", pid.as_ref())],
+        )?;
+        if !o.status.success() {
+            return Err(io::Error::other(
+                String::from_utf8_lossy(&o.stderr).trim().to_owned(),
+            ));
+        }
+        Ok(String::from_utf8_lossy(&o.stdout).trim() != "0")
+    }
+
+    fn programar_borrado(
+        &self,
+        pid: u32,
+        ficheros: &[PathBuf],
+        dirs: &[PathBuf],
+    ) -> io::Result<()> {
+        use std::os::windows::process::CommandExt;
+        // DETACHED_PROCESS | CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP: el
+        // auxiliar sobrevive a este proceso y a su consola.
+        const FLAGS: u32 = 0x0000_0008 | 0x0800_0000 | 0x0000_0200;
+        let unir = |v: &[PathBuf]| {
+            let mut s = std::ffi::OsString::new();
+            for (i, p) in v.iter().enumerate() {
+                if i > 0 {
+                    s.push("\n");
+                }
+                s.push(p.as_os_str());
+            }
+            s
+        };
+        std::process::Command::new("powershell.exe")
+            .args([
+                "-NoProfile",
+                "-NonInteractive",
+                "-WindowStyle",
+                "Hidden",
+                "-Command",
+                PS_AUTOBORRADO,
+            ])
+            .env("TC2_PID", pid.to_string())
+            .env("TC2_FICHEROS", unir(ficheros))
+            .env("TC2_DIRS", unir(dirs))
+            .stdin(std::process::Stdio::null())
+            .stdout(std::process::Stdio::null())
+            .stderr(std::process::Stdio::null())
+            .creation_flags(FLAGS)
+            .spawn()
+            .map(|_| ())
+    }
+}
+
+#[cfg(not(windows))]
+impl Maquina for MaquinaReal {
+    fn leer_path_usuario(&self) -> io::Result<Option<String>> {
+        Ok(None)
+    }
+
+    fn escribir_path_usuario(&self, _viejo: &str, _nuevo: &str) -> io::Result<()> {
+        Err(io::Error::from(io::ErrorKind::Unsupported))
+    }
+
+    fn en_uso_por_otro(&self, _ruta: &Path, _pid: u32) -> io::Result<bool> {
+        // En Unix un binario en marcha se puede borrar: nunca bloquea.
+        Ok(false)
+    }
+
+    fn programar_borrado(
+        &self,
+        _pid: u32,
+        _ficheros: &[PathBuf],
+        _dirs: &[PathBuf],
+    ) -> io::Result<()> {
+        Err(io::Error::from(io::ErrorKind::Unsupported))
+    }
+}
diff --git a/crates/tc2/src/desinstalar/texto.rs b/crates/tc2/src/desinstalar/texto.rs
new file mode 100644
index 0000000..8abad1b
--- /dev/null
+++ b/crates/tc2/src/desinstalar/texto.rs
@@ -0,0 +1,309 @@
+//! Funciones puras de `desinstalar`: quitar el bloque de marcas de un
+//! perfil, quitar un miembro de un JSON sin tocar el resto de bytes, buscar
+//! `tc2` en los hooks y quitar una entrada del PATH de usuario de Windows.
+
+use distribucion::{LINEA_PERFIL, MARCA_FIN_PERFIL, MARCA_INICIO_PERFIL};
+
+/// Linea sin su final (`\n` o `\r\n`).
+fn sin_final(l: &str) -> &str {
+    l.strip_suffix('\n')
+        .map_or(l, |x| x.strip_suffix('\r').unwrap_or(x))
+}
+
+/// Por que un perfil con marcas no se toca.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum MarcasRaras {
+    /// No hay exactamente un inicio y despues un fin.
+    Incompletas,
+    /// Entre las marcas no esta solo la linea que escribe `install.sh`.
+    Tocadas,
+}
+
+impl MarcasRaras {
+    /// Valor de `motivo=`.
+    pub fn motivo(self) -> &'static str {
+        match self {
+            Self::Incompletas => "marcas-incompletas",
+            Self::Tocadas => "bloque-modificado",
+        }
+    }
+}
+
+/// Inversa de `escribir_bloque` de `install.sh`: quita el bloque entre las
+/// marcas y la linea en blanco que el instalador escribio delante. El resto
+/// queda identico byte a byte. `Ok(None)` si no hay marcas; `Err` si las
+/// marcas no son un bloque entero de tres lineas (no se toca nada).
+pub fn quitar_bloque(texto: &str) -> Result<Option<String>, MarcasRaras> {
+    let lineas: Vec<&str> = texto.split_inclusive('\n').collect();
+    let inicios: Vec<usize> = (0..lineas.len())
+        .filter(|&i| sin_final(lineas[i]) == MARCA_INICIO_PERFIL)
+        .collect();
+    let fines: Vec<usize> = (0..lineas.len())
+        .filter(|&i| sin_final(lineas[i]) == MARCA_FIN_PERFIL)
+        .collect();
+    match (inicios.as_slice(), fines.as_slice()) {
+        ([], []) => Ok(None),
+        ([i], [f]) if i < f => {
+            let (i, f) = (*i, *f);
+            if f != i + 2 || !sin_final(lineas[i + 1]).starts_with(LINEA_PERFIL) {
+                return Err(MarcasRaras::Tocadas);
+            }
+            let desde = if i > 0 && sin_final(lineas[i - 1]).is_empty() {
+                i - 1
+            } else {
+                i
+            };
+            let mut s = String::with_capacity(texto.len());
+            for l in lineas.get(..desde).unwrap_or_default() {
+                s.push_str(l);
+            }
+            for l in lineas.get(f + 1..).unwrap_or_default() {
+                s.push_str(l);
+            }
+            Ok(Some(s))
+        }
+        _ => Err(MarcasRaras::Incompletas),
+    }
+}
+
+/// Un miembro del objeto de primer nivel: clave y donde empieza y acaba.
+#[derive(Debug)]
+struct Miembro {
+    clave: String,
+    inicio: usize,
+    fin: usize,
+}
+
+/// Recorre el objeto de primer nivel de `texto` (JSON ya validado) y da sus
+/// miembros, y el indice de la `{` de apertura.
+fn miembros(texto: &str) -> Option<(usize, Vec<Miembro>)> {
+    let b = texto.as_bytes();
+    let mut i = 0;
+    let saltar = |i: &mut usize| {
+        while *i < b.len() && b[*i].is_ascii_whitespace() {
+            *i += 1;
+        }
+    };
+    // Fin de una cadena que empieza en `i` (en la comilla).
+    let fin_cadena = |mut i: usize| -> Option<usize> {
+        i += 1;
+        while i < b.len() {
+            match b[i] {
+                b'\\' => i += 2,
+                b'"' => return Some(i + 1),
+                _ => i += 1,
+            }
+        }
+        None
+    };
+    saltar(&mut i);
+    if b.get(i) != Some(&b'{') {
+        return None;
+    }
+    let apertura = i;
+    i += 1;
+    let mut v = Vec::new();
+    loop {
+        saltar(&mut i);
+        match b.get(i)? {
+            b'}' => return Some((apertura, v)),
+            b',' => {
+                i += 1;
+                continue;
+            }
+            b'"' => {}
+            _ => return None,
+        }
+        let inicio = i;
+        let fc = fin_cadena(i)?;
+        let clave: String = serde_json::from_str(texto.get(i..fc)?).ok()?;
+        i = fc;
+        saltar(&mut i);
+        if b.get(i) != Some(&b':') {
+            return None;
+        }
+        i += 1;
+        saltar(&mut i);
+        // El valor: hasta la `,` o la `}` del primer nivel.
+        let mut prof = 0usize;
+        while i < b.len() {
+            match b[i] {
+                b'"' => {
+                    i = fin_cadena(i)?;
+                    continue;
+                }
+                b'{' | b'[' => prof += 1,
+                b'}' | b']' if prof > 0 => prof -= 1,
+                b',' | b'}' if prof == 0 => break,
+                _ => {}
+            }
+            i += 1;
+        }
+        let mut fin = i;
+        while fin > inicio && b[fin - 1].is_ascii_whitespace() {
+            fin -= 1;
+        }
+        v.push(Miembro { clave, inicio, fin });
+    }
+}
+
+/// Quita el miembro `clave` del objeto de primer nivel de `texto` sin tocar
+/// ningun otro byte: con la coma y el espacio que lo separan del siguiente
+/// (o del anterior, si es el ultimo). `None` si no esta, si el JSON no es un
+/// objeto o si el resultado no es el mismo JSON sin esa clave.
+pub fn quitar_miembro(texto: &str, clave: &str) -> Option<String> {
+    let antes: serde_json::Value = serde_json::from_str(texto).ok()?;
+    let (apertura, ms) = miembros(texto)?;
+    let pos = ms.iter().position(|m| m.clave == clave)?;
+    let m = &ms[pos];
+    let (desde, hasta) = match (pos.checked_sub(1).and_then(|p| ms.get(p)), ms.get(pos + 1)) {
+        // Hay uno detras: desde la clave hasta el inicio del siguiente.
+        (_, Some(sig)) => (m.inicio, sig.inicio),
+        // Es el ultimo con uno delante: desde el final del anterior.
+        (Some(ant), None) => (ant.fin, m.fin),
+        // Es el unico.
+        (None, None) => (apertura + 1, m.fin),
+    };
+    let mut s = String::with_capacity(texto.len());
+    s.push_str(texto.get(..desde)?);
+    s.push_str(texto.get(hasta..)?);
+    let despues: serde_json::Value = serde_json::from_str(&s).ok()?;
+    let mut esperado = antes;
+    esperado.as_object_mut()?.remove(clave);
+    (despues == esperado).then_some(s)
+}
+
+/// Si un `command` de un hook nombra `tc2` como palabra (`tc2`, `tc2.exe` o
+/// una ruta que acaba en ellos).
+pub fn nombra_tc2(command: &str) -> bool {
+    command
+        .split(|c: char| c.is_whitespace() || "\"'`;&|()/\\".contains(c))
+        .any(|p| p.eq_ignore_ascii_case("tc2") || p.eq_ignore_ascii_case("tc2.exe"))
+}
+
+/// Los `command` de los hooks de un `settings.json` que nombran `tc2`.
+pub fn hooks_con_tc2(v: &serde_json::Value) -> Vec<String> {
+    fn recorrer(v: &serde_json::Value, out: &mut Vec<String>) {
+        match v {
+            serde_json::Value::Object(m) => {
+                for (k, x) in m {
+                    match x {
+                        serde_json::Value::String(c) if k == "command" && nombra_tc2(c) => {
+                            out.push(c.clone());
+                        }
+                        _ => recorrer(x, out),
+                    }
+                }
+            }
+            serde_json::Value::Array(a) => a.iter().for_each(|x| recorrer(x, out)),
+            _ => {}
+        }
+    }
+    let mut out = Vec::new();
+    if let Some(h) = v.get("hooks") {
+        recorrer(h, &mut out);
+    }
+    out
+}
+
+/// Entrada del PATH normalizada para comparar como `Add-PathUsuario`: sin
+/// `\` al final, con `\` como separador y sin distinguir mayusculas.
+fn normal(e: &str) -> String {
+    e.replace('/', "\\").trim_end_matches('\\').to_lowercase()
+}
+
+/// Inversa exacta de `Add-PathUsuario` de `install.ps1`: quita del valor
+/// crudo del PATH de usuario las entradas que, expandidas con `expandir`,
+/// son `dir`. Las demas entradas (y su orden, y las vacias) quedan igual.
+/// `None` si no habia ninguna.
+pub fn quitar_del_path(
+    valor: &str,
+    dir: &str,
+    expandir: impl Fn(&str) -> String,
+) -> Option<String> {
+    let objetivo = normal(dir);
+    let partes: Vec<&str> = valor.split(';').collect();
+    let quedan: Vec<&str> = partes
+        .iter()
+        .copied()
+        .filter(|e| e.is_empty() || normal(&expandir(e)) != objetivo)
+        .collect();
+    (quedan.len() != partes.len()).then(|| quedan.join(";"))
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    const BLOQUE: &str = "\n# >>> tc2 (TaskCodeTwo) >>>\ncase \":$PATH:\" in *\":/h/.local/bin:\"*) ;; *) export PATH=\"/h/.local/bin:$PATH\" ;; esac\n# <<< tc2 (TaskCodeTwo) <<<\n";
+
+    #[test]
+    fn quitar_bloque_deja_el_resto_byte_a_byte() {
+        for original in ["", "alias ll='ls -l'\n", "# mio\r\n\r\nexport A=1\r\n"] {
+            let con = format!("{original}{BLOQUE}");
+            assert_eq!(quitar_bloque(&con), Ok(Some(original.to_owned())));
+        }
+        // Con algo detras del bloque, tambien.
+        let con = format!("a\n{BLOQUE}b\n");
+        assert_eq!(quitar_bloque(&con), Ok(Some("a\nb\n".to_owned())));
+        assert_eq!(quitar_bloque("nada\n"), Ok(None));
+        // Marcas tocadas a mano: no se toca.
+        let sin_fin = "# >>> tc2 (TaskCodeTwo) >>>\nexport PATH=\"/v:$PATH\"\n";
+        assert_eq!(quitar_bloque(sin_fin), Err(MarcasRaras::Incompletas));
+        let doble = format!("{BLOQUE}{BLOQUE}");
+        assert_eq!(quitar_bloque(&doble), Err(MarcasRaras::Incompletas));
+        let otro = "# >>> tc2 (TaskCodeTwo) >>>\nrm -rf ~\n# <<< tc2 (TaskCodeTwo) <<<\n";
+        assert_eq!(quitar_bloque(otro), Err(MarcasRaras::Tocadas));
+    }
+
+    #[test]
+    fn quitar_miembro_sin_tocar_el_resto() {
+        let t = "{\n  \"a\": 1,\n  \"statusLine\": {\"type\": \"command\", \"command\": \"x, }\"},\n  \"b\": [1, {\"c\": 2}]\n}\n";
+        assert_eq!(
+            quitar_miembro(t, "statusLine").as_deref(),
+            Some("{\n  \"a\": 1,\n  \"b\": [1, {\"c\": 2}]\n}\n")
+        );
+        let ultimo = "{\r\n  \"a\": 1,\r\n  \"statusLine\": {}\r\n}";
+        assert_eq!(
+            quitar_miembro(ultimo, "statusLine").as_deref(),
+            Some("{\r\n  \"a\": 1\r\n}")
+        );
+        assert_eq!(
+            quitar_miembro("{\"statusLine\": 1}", "statusLine").as_deref(),
+            Some("{}")
+        );
+        assert_eq!(quitar_miembro("{\"a\": 1}", "statusLine"), None);
+        assert_eq!(quitar_miembro("[1]", "statusLine"), None);
+        assert_eq!(quitar_miembro("{\"statusLine\": ", "statusLine"), None);
+        // Una clave con escape no se confunde.
+        let esc = "{\"status\\u004cine\": 1, \"x\": \"statusLine\"}";
+        assert_eq!(
+            quitar_miembro(esc, "statusLine").as_deref(),
+            Some("{\"x\": \"statusLine\"}")
+        );
+    }
+
+    #[test]
+    fn hooks_que_nombran_tc2() {
+        let v: serde_json::Value = serde_json::from_str(
+            r#"{"hooks":{"Stop":[{"hooks":[{"type":"command","command":"tc2 estado"},{"type":"command","command":"/x/tc2.exe guardar"},{"type":"command","command":"tc23 x"},{"type":"command","command":"taskctl tc2x"}]}]},"command":"tc2"}"#,
+        )
+        .unwrap_or_default();
+        assert_eq!(hooks_con_tc2(&v), vec!["tc2 estado", "/x/tc2.exe guardar"]);
+    }
+
+    #[test]
+    fn quitar_del_path_inversa_de_add_path_usuario() {
+        let exp = |e: &str| e.replace("%LOCALAPPDATA%", r"C:\U\L");
+        let dir = r"C:\U\L\taskcodetwo\bin";
+        assert_eq!(
+            quitar_del_path(r"C:\a;%LOCALAPPDATA%\taskcodetwo\bin\;;D:\b", dir, exp).as_deref(),
+            Some(r"C:\a;;D:\b")
+        );
+        assert_eq!(
+            quitar_del_path(r"c:\u\l\TASKCODETWO\bin;C:\a", dir, exp).as_deref(),
+            Some(r"C:\a")
+        );
+        assert_eq!(quitar_del_path(r"C:\a;C:\U\L\taskcodetwo", dir, exp), None);
+    }
+}
diff --git a/crates/tc2/src/git/puerto.rs b/crates/tc2/src/git/puerto.rs
index 7e71078..3cdf400 100644
--- a/crates/tc2/src/git/puerto.rs
+++ b/crates/tc2/src/git/puerto.rs
@@ -193,9 +193,12 @@ const ENTORNO_QUITADO: [&str; 6] = [
     "GIT_NAMESPACE",
 ];
 
-/// Entorno fijo de cada llamada (F1-4, decision 1).
-const ENTORNO_FIJO: [(&str, &str); 5] = [
+/// Entorno fijo de cada llamada (F1-4, decision 1). `GIT_OPTIONAL_LOCKS=0`
+/// (F2-14): una lectura como `status` no reescribe el indice de paso, asi
+/// que `tc2 desinstalar --simular` no cambia ni un byte del repo.
+const ENTORNO_FIJO: [(&str, &str); 6] = [
     ("LC_ALL", "C"),
+    ("GIT_OPTIONAL_LOCKS", "0"),
     ("LANG", "C"),
     ("GIT_TERMINAL_PROMPT", "0"),
     ("GIT_PAGER", "cat"),
@@ -320,6 +323,31 @@ impl Git {
         })
     }
 
+    /// Borra la rama local de estado (`refs/heads/taskcodetwo/tareas`) si
+    /// sigue en `viejo`: `git update-ref -d <ref> <viejo>`. Es la unica
+    /// excepcion a «nunca `refs/heads/*`» (F1-4 D3) y solo la usa
+    /// `tc2 desinstalar --borrar-tareas` con el nombre exacto (F2-14, C7): la
+    /// ref la fija esta funcion, no quien llama, y con el valor viejo no se
+    /// borra un commit que no se ha visto.
+    pub fn borrar_rama_de_estado(&self, dir: &Path, viejo: &str) -> Result<SalidaGit, ErrorGit> {
+        if viejo.len() < 40 || !viejo.bytes().all(|b| b.is_ascii_hexdigit()) {
+            return Err(ErrorGit::Prohibido(
+                Mensaje::nuevo(&txt::ARGUMENTO_PROHIBIDO)
+                    .con("argumento", format!("update-ref -d {viejo}")),
+            ));
+        }
+        let r = format!("refs/heads/{}", crate::huella::RAMA_ESTADO);
+        let salida = self
+            .comando(dir, Subcomando::UpdateRef, &["-d", &r, viejo], None)
+            .output()
+            .map_err(|e| ErrorGit::Lanzar(e.to_string()))?;
+        Ok(SalidaGit {
+            codigo: salida.status.code().unwrap_or(-1),
+            stdout: String::from_utf8_lossy(&salida.stdout).into_owned(),
+            stderr: String::from_utf8_lossy(&salida.stderr).into_owned(),
+        })
+    }
+
     /// `git -c ... <sub> <args>` en `dir` con el entorno fijo, y su linea en
     /// el registro.
     fn comando(
diff --git a/crates/tc2/src/i18n/catalogo/cli.rs b/crates/tc2/src/i18n/catalogo/cli.rs
index 2347eaf..dd8f6f6 100644
--- a/crates/tc2/src/i18n/catalogo/cli.rs
+++ b/crates/tc2/src/i18n/catalogo/cli.rs
@@ -10,8 +10,8 @@ crate::i18n::textos! {
         en: "tc2: could not write the output: {error}",
     }
     USO {
-        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json]",
-        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json]",
+        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
+        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
     }
     FALTA_LA_ORDEN {
         es: "tc2: falta la orden. {uso}",
diff --git a/crates/tc2/src/i18n/catalogo/desinstalar.rs b/crates/tc2/src/i18n/catalogo/desinstalar.rs
new file mode 100644
index 0000000..efeb039
--- /dev/null
+++ b/crates/tc2/src/i18n/catalogo/desinstalar.rs
@@ -0,0 +1,44 @@
+//! Mensajes de `tc2 desinstalar` (F2-14).
+
+crate::i18n::textos! {
+    USO_DESINSTALAR {
+        es: "uso: desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]; --config y --borrar-tareas solo con --proyecto",
+        en: "usage: desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]; --config and --borrar-tareas only with --proyecto",
+    }
+    NOMBRE_NO_COINCIDE {
+        es: "--borrar-tareas exige el nombre exacto de la rama de estado, {rama}; se recibio «{nombre}». No se ha tocado nada",
+        en: "--borrar-tareas needs the exact name of the state branch, {rama}; got «{nombre}». Nothing was touched",
+    }
+    HAZ_TU {
+        es: "{clase} {ruta}: no lo quita tc2; quitalo tu con: {orden}",
+        en: "{clase} {ruta}: tc2 does not remove it; remove it yourself with: {orden}",
+    }
+    AVISO_HOOK {
+        es: "aviso: {ruta} tiene un hook que nombra tc2; desinstalar nunca edita los hooks: quitalo tu si ya no lo quieres",
+        en: "warning: {ruta} has a hook that names tc2; desinstalar never edits hooks: remove it yourself if you no longer want it",
+    }
+    BLOQUEADA {
+        es: "no se ha tocado nada: {ruta} ({motivo}). Guarda y publica sus cambios (o quitalo tu) y repite",
+        en: "nothing was touched: {ruta} ({motivo}). Save and push its changes (or remove it yourself) and try again",
+    }
+    CERROJO_OCUPADO {
+        es: "otro tc2 esta escribiendo el estado de este repo; no se ha tocado nada: espera a que acabe y repite",
+        en: "another tc2 is writing this repo's state; nothing was touched: wait for it to finish and try again",
+    }
+    EN_USO {
+        es: "{ruta} esta en uso por otro proceso: no se ha tocado ni el ni su entrada del PATH; cierralo y repite tc2 desinstalar --maquina (lo demas ya se quito)",
+        en: "{ruta} is in use by another process: neither it nor its PATH entry was touched; close it and run tc2 desinstalar --maquina again (the rest is already removed)",
+    }
+    FALLO_AL_APLICAR {
+        es: "no se pudo quitar {ruta}: {error}. Lo de las lineas anteriores ya se quito: arregla la causa y repite tc2 desinstalar",
+        en: "could not remove {ruta}: {error}. What the previous lines list is already removed: fix the cause and run tc2 desinstalar again",
+    }
+    NO_SE_LEE {
+        es: "no se puede leer {ruta}: {error}",
+        en: "cannot read {ruta}: {error}",
+    }
+    SIN_ENTORNO {
+        es: "no se puede saber donde esta este tc2: {error}",
+        en: "cannot tell where this tc2 is: {error}",
+    }
+}
diff --git a/crates/tc2/src/i18n/catalogo/mod.rs b/crates/tc2/src/i18n/catalogo/mod.rs
index 9f91662..d4e49fa 100644
--- a/crates/tc2/src/i18n/catalogo/mod.rs
+++ b/crates/tc2/src/i18n/catalogo/mod.rs
@@ -5,6 +5,7 @@ pub mod actualizar;
 pub mod cli;
 pub mod comun;
 pub mod config;
+pub mod desinstalar;
 pub mod estado;
 pub mod git;
 pub mod identidad;
@@ -19,6 +20,7 @@ pub fn todos() -> impl Iterator<Item = &'static Texto> {
         cli::TODOS,
         comun::TODOS,
         config::TODOS,
+        desinstalar::TODOS,
         estado::TODOS,
         git::TODOS,
         identidad::TODOS,
diff --git a/crates/tc2/src/instalacion.rs b/crates/tc2/src/instalacion.rs
new file mode 100644
index 0000000..eddad3b
--- /dev/null
+++ b/crates/tc2/src/instalacion.rs
@@ -0,0 +1,375 @@
+//! Donde vive el `tc2` de esta maquina (F2-12 y F2-14): quien lo gestiona
+//! ([`Modo`]), las rutas que dejan los instaladores de F2-11 y el lanzador
+//! del plugin, y el `.old` de una actualizacion en Windows.
+//!
+//! Todo lo que depende del entorno del proceso (HOME, `LOCALAPPDATA`,
+//! `TC2_INSTALL_DIR`...) entra por un [`Entorno`] inyectado: los tests de
+//! `tc2 desinstalar` usan un HOME falso y nunca el real de la maquina.
+
+use std::path::{Path, PathBuf};
+
+/// Quien gestiona el `tc2` que se esta ejecutando.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Modo {
+    /// Instalado con la linea de instalacion (o copiado a mano): se actualiza.
+    Instalador,
+    /// La cache del plugin (`.../taskcodetwo/cli/<version>/tc2[.exe]`): lo
+    /// gestiona el plugin con `cli.lock`.
+    Plugin,
+    /// Un gestor de paquetes (npm, Homebrew, winget, Scoop, cargo): se
+    /// actualiza con el.
+    Gestor(&'static str),
+}
+
+/// Modo de un ejecutable por su ruta (sin tocar el disco). Reconoce las
+/// rutas de Windows, macOS y Linux sin depender del sistema en que corre.
+pub fn detectar_modo(ruta: &str) -> Modo {
+    let partes: Vec<String> = ruta
+        .split(['/', '\\'])
+        .filter(|p| !p.is_empty())
+        .map(str::to_ascii_lowercase)
+        .collect();
+    let n = partes.len();
+    let es_tc2 = partes.last().is_some_and(|f| f == "tc2" || f == "tc2.exe");
+    if es_tc2
+        && n >= 4
+        && partes.get(n - 4).is_some_and(|p| p == "taskcodetwo")
+        && partes.get(n - 3).is_some_and(|p| p == "cli")
+    {
+        return Modo::Plugin;
+    }
+    // Cada gestor por su propia estructura de carpetas, no por un nombre
+    // suelto: una carpeta `Scoop\bin` o `~/.cargo/otra` de la persona no es
+    // de un gestor. `dir` son las carpetas, sin el fichero.
+    let dir = partes.get(..n.saturating_sub(1)).unwrap_or_default();
+    let seguidas = |a: &str, b: &str| dir.windows(2).any(|w| w[0] == a && w[1] == b);
+    // `<raiz>/node_modules/<paquete>/...`: el binario dentro de un paquete.
+    let npm = dir
+        .iter()
+        .position(|p| p == "node_modules")
+        .is_some_and(|i| i + 1 < dir.len());
+    // `Cellar/<formula>/<version>/...`, o el prefijo de Homebrew.
+    let homebrew = dir
+        .iter()
+        .position(|p| p == "cellar")
+        .is_some_and(|i| i + 2 < dir.len())
+        || dir.starts_with(&["opt".to_owned(), "homebrew".to_owned()])
+        || seguidas("linuxbrew", ".linuxbrew");
+    let gestor = if npm {
+        Some("npm")
+    } else if homebrew {
+        Some("homebrew")
+    } else if seguidas("winget", "packages") || seguidas("winget", "links") {
+        Some("winget")
+    } else if seguidas("scoop", "apps") || seguidas("scoop", "shims") {
+        Some("scoop")
+    } else if dir.len() >= 2 && dir[dir.len() - 2] == ".cargo" && dir[dir.len() - 1] == "bin" {
+        Some("cargo")
+    } else {
+        None
+    };
+    gestor.map_or(Modo::Instalador, Modo::Gestor)
+}
+
+/// La orden exacta que quita un `tc2` instalado con un gestor de paquetes
+/// (nombres de F1-12 D1a: el paquete es `taskcodetwo`; en cargo, el crate
+/// `tc2`).
+pub fn orden_del_gestor(gestor: &str) -> String {
+    match gestor {
+        "npm" => "npm uninstall -g taskcodetwo",
+        "homebrew" => "brew uninstall taskcodetwo",
+        "winget" => "winget uninstall taskcodetwo",
+        "scoop" => "scoop uninstall taskcodetwo",
+        "cargo" => "cargo uninstall tc2",
+        _ => "",
+    }
+    .to_owned()
+}
+
+/// `<exe>.old`: donde queda el binario anterior en Windows hasta la
+/// siguiente ejecucion.
+pub fn ruta_old(exe: &Path) -> PathBuf {
+    let mut s = exe.as_os_str().to_owned();
+    s.push(".old");
+    PathBuf::from(s)
+}
+
+/// Borra `<exe>.old` si lo hay (lo deja una actualizacion en Windows, donde
+/// un ejecutable en marcha no se puede borrar). Si sigue en uso, se queda.
+pub fn limpiar_old(exe: &Path) {
+    let _ = std::fs::remove_file(ruta_old(exe));
+}
+
+/// Sistema para el que se calculan las rutas.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Sistema {
+    /// Windows.
+    Windows,
+    /// macOS.
+    Mac,
+    /// Linux y el resto de Unix.
+    Linux,
+}
+
+impl Sistema {
+    /// El sistema en que corre este proceso.
+    pub fn actual() -> Self {
+        if cfg!(windows) {
+            Self::Windows
+        } else if cfg!(target_os = "macos") {
+            Self::Mac
+        } else {
+            Self::Linux
+        }
+    }
+
+    /// Nombre del binario: `tc2.exe` o `tc2`.
+    pub fn binario(self) -> &'static str {
+        match self {
+            Self::Windows => "tc2.exe",
+            Self::Mac | Self::Linux => "tc2",
+        }
+    }
+}
+
+/// Lo que se lee del entorno del proceso, inyectable en los tests.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Entorno {
+    /// Sistema para el que se calculan las rutas.
+    pub sistema: Sistema,
+    /// HOME (en Windows, `USERPROFILE`).
+    pub home: Option<PathBuf>,
+    /// `LOCALAPPDATA` (Windows).
+    pub local_app_data: Option<PathBuf>,
+    /// `XDG_CACHE_HOME` (Unix).
+    pub xdg_cache_home: Option<PathBuf>,
+    /// `TC2_INSTALL_DIR`, si se instalo en otro sitio.
+    pub install_dir: Option<PathBuf>,
+    /// `CLAUDE_CONFIG_DIR` (por defecto `~/.claude`).
+    pub claude_config_dir: Option<PathBuf>,
+    /// El ejecutable de este `tc2`.
+    pub exe: PathBuf,
+    /// PID de este proceso.
+    pub pid: u32,
+}
+
+/// Una variable de entorno no vacia como ruta.
+fn ruta_de(valor: Option<String>) -> Option<PathBuf> {
+    valor.filter(|v| !v.is_empty()).map(PathBuf::from)
+}
+
+impl Entorno {
+    /// El entorno del proceso, leido con `leer` (para no tocar las variables
+    /// del proceso en los tests).
+    pub fn de_variables(
+        sistema: Sistema,
+        exe: PathBuf,
+        pid: u32,
+        leer: impl Fn(&str) -> Option<String>,
+    ) -> Self {
+        let home = match sistema {
+            Sistema::Windows => ruta_de(leer("USERPROFILE")).or_else(|| ruta_de(leer("HOME"))),
+            Sistema::Mac | Sistema::Linux => ruta_de(leer("HOME")),
+        };
+        Self {
+            sistema,
+            home,
+            local_app_data: ruta_de(leer("LOCALAPPDATA")),
+            xdg_cache_home: ruta_de(leer("XDG_CACHE_HOME")),
+            install_dir: ruta_de(leer("TC2_INSTALL_DIR")),
+            claude_config_dir: ruta_de(leer("CLAUDE_CONFIG_DIR")),
+            exe,
+            pid,
+        }
+    }
+
+    /// El entorno real de este proceso.
+    pub fn del_proceso() -> std::io::Result<Self> {
+        Ok(Self::de_variables(
+            Sistema::actual(),
+            std::env::current_exe()?,
+            std::process::id(),
+            |k| std::env::var(k).ok(),
+        ))
+    }
+
+    /// Modo del `tc2` que se esta ejecutando.
+    pub fn modo(&self) -> Modo {
+        detectar_modo(&self.exe.to_string_lossy())
+    }
+
+    /// Directorio por defecto de los instaladores de F2-11:
+    /// `%LOCALAPPDATA%\taskcodetwo\bin` o `~/.local/bin`.
+    pub fn dir_por_defecto(&self) -> Option<PathBuf> {
+        match self.sistema {
+            Sistema::Windows => self
+                .local_app_data
+                .as_ref()
+                .map(|l| l.join("taskcodetwo").join("bin")),
+            Sistema::Mac | Sistema::Linux => {
+                self.home.as_ref().map(|h| h.join(".local").join("bin"))
+            }
+        }
+    }
+
+    /// Si `dir` es una carpeta solo de `tc2` (la de Windows por defecto):
+    /// vacia, se puede borrar. `~/.local/bin` o un `TC2_INSTALL_DIR` no.
+    pub fn dir_propio(&self, dir: &Path) -> bool {
+        self.sistema == Sistema::Windows && self.dir_por_defecto().as_deref() == Some(dir)
+    }
+
+    /// Directorios donde pudo dejar `tc2` un instalador: `TC2_INSTALL_DIR` y
+    /// el de por defecto, sin repetir.
+    pub fn dirs_de_instalacion(&self) -> Vec<PathBuf> {
+        let mut v: Vec<PathBuf> = Vec::new();
+        for d in [self.install_dir.clone(), self.dir_por_defecto()]
+            .into_iter()
+            .flatten()
+        {
+            if !v.contains(&d) {
+                v.push(d);
+            }
+        }
+        v
+    }
+
+    /// Carpetas `taskcodetwo` de la cache del lanzador del plugin (ADR F1-14,
+    /// punto 4), sin repetir: `%LOCALAPPDATA%\taskcodetwo`,
+    /// `$XDG_CACHE_HOME/taskcodetwo` o `~/.cache/taskcodetwo` y, en macOS,
+    /// `~/Library/Caches/taskcodetwo`. La cache es su subcarpeta `cli`.
+    pub fn raices_de_cache(&self) -> Vec<PathBuf> {
+        let mut v: Vec<PathBuf> = Vec::new();
+        let mut poner = |p: Option<PathBuf>| {
+            if let Some(p) = p.map(|p| p.join("taskcodetwo"))
+                && !v.contains(&p)
+            {
+                v.push(p);
+            }
+        };
+        match self.sistema {
+            Sistema::Windows => poner(self.local_app_data.clone()),
+            Sistema::Mac | Sistema::Linux => {
+                if self.sistema == Sistema::Mac {
+                    poner(self.home.as_ref().map(|h| h.join("Library").join("Caches")));
+                }
+                poner(
+                    self.xdg_cache_home
+                        .clone()
+                        .or_else(|| self.home.as_ref().map(|h| h.join(".cache"))),
+                );
+            }
+        }
+        v
+    }
+
+    /// Carpeta de configuracion de Claude Code (`CLAUDE_CONFIG_DIR` o
+    /// `~/.claude`).
+    pub fn dir_claude(&self) -> Option<PathBuf> {
+        self.claude_config_dir
+            .clone()
+            .or_else(|| self.home.as_ref().map(|h| h.join(".claude")))
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    /// TASK-027, criterio 6: la cache del plugin se reconoce en los tres
+    /// sistemas, y no se confunde con una instalacion normal.
+    #[test]
+    fn detectar_modo_con_rutas_de_los_tres_sistemas() {
+        for plugin in [
+            r"C:\Users\ana\AppData\Local\taskcodetwo\cli\2.0.0\tc2.exe",
+            r"c:\users\ana\appdata\local\TaskCodeTwo\CLI\2.0.0-rc.1\TC2.EXE",
+            "/Users/ana/.cache/taskcodetwo/cli/2.0.0/tc2",
+            "/Users/ana/Library/Caches/taskcodetwo/cli/2.0.0/tc2",
+            "/home/ana/.cache/taskcodetwo/cli/2.0.0/tc2",
+        ] {
+            assert_eq!(detectar_modo(plugin), Modo::Plugin, "{plugin}");
+        }
+        for instalador in [
+            r"C:\Users\ana\AppData\Local\taskcodetwo\bin\tc2.exe",
+            "/Users/ana/.local/bin/tc2",
+            "/home/ana/.local/bin/tc2",
+            "/home/ana/taskcodetwo/cli/tc2",
+            "/home/ana/taskcodetwo/cli/2.0.0/otro",
+            "/opt/herramientas/tc2",
+        ] {
+            assert_eq!(detectar_modo(instalador), Modo::Instalador, "{instalador}");
+        }
+        for (ruta, gestor) in [
+            (
+                r"C:\Users\ana\AppData\Roaming\npm\node_modules\taskcodetwo\bin\tc2.exe",
+                "npm",
+            ),
+            ("/opt/homebrew/Cellar/tc2/2.0.0/bin/tc2", "homebrew"),
+            (
+                r"C:\Users\ana\AppData\Local\Microsoft\WinGet\Packages\x\tc2.exe",
+                "winget",
+            ),
+            (r"C:\Users\ana\scoop\apps\tc2\current\tc2.exe", "scoop"),
+            ("/home/ana/.cargo/bin/tc2", "cargo"),
+            ("/home/linuxbrew/.linuxbrew/bin/tc2", "homebrew"),
+            (r"C:\Users\ana\scoop\shims\tc2.exe", "scoop"),
+        ] {
+            assert_eq!(detectar_modo(ruta), Modo::Gestor(gestor), "{ruta}");
+            assert!(!orden_del_gestor(gestor).is_empty(), "{gestor}");
+        }
+        // Revision 1, MEN-1: un nombre de gestor suelto en la ruta no basta.
+        for propia in [
+            r"C:\Users\ana\Scoop\bin\tc2.exe",
+            r"C:\herramientas\scoop\tc2.exe",
+            "/home/ana/.cargo/otra/tc2",
+            "/home/ana/.cargo/bin/sub/tc2",
+            "/home/ana/node_modules/tc2",
+            "/home/ana/cellar/tc2",
+            "/home/ana/homebrew/tc2",
+            r"C:\Users\ana\winget\tc2.exe",
+        ] {
+            assert_eq!(detectar_modo(propia), Modo::Instalador, "{propia}");
+        }
+    }
+
+    #[test]
+    fn rutas_del_entorno_por_sistema() {
+        let vars = |k: &str| match k {
+            "HOME" => Some("/h".to_owned()),
+            "USERPROFILE" => Some("C:/u".to_owned()),
+            "LOCALAPPDATA" => Some("C:/u/L".to_owned()),
+            "TC2_INSTALL_DIR" => Some(String::new()),
+            _ => None,
+        };
+        let w = Entorno::de_variables(Sistema::Windows, "C:/x/tc2.exe".into(), 1, vars);
+        assert_eq!(w.home, Some(PathBuf::from("C:/u")));
+        assert_eq!(w.install_dir, None);
+        assert_eq!(
+            w.dirs_de_instalacion(),
+            vec![PathBuf::from("C:/u/L/taskcodetwo/bin")]
+        );
+        assert!(w.dir_propio(Path::new("C:/u/L/taskcodetwo/bin")));
+        assert_eq!(
+            w.raices_de_cache(),
+            vec![PathBuf::from("C:/u/L/taskcodetwo")]
+        );
+        assert_eq!(w.dir_claude(), Some(PathBuf::from("C:/u/.claude")));
+        let l = Entorno::de_variables(Sistema::Linux, "/h/.local/bin/tc2".into(), 1, vars);
+        assert_eq!(
+            l.dirs_de_instalacion(),
+            vec![PathBuf::from("/h/.local/bin")]
+        );
+        assert!(!l.dir_propio(Path::new("/h/.local/bin")));
+        assert_eq!(
+            l.raices_de_cache(),
+            vec![PathBuf::from("/h/.cache/taskcodetwo")]
+        );
+        let m = Entorno::de_variables(Sistema::Mac, "/h/.local/bin/tc2".into(), 1, vars);
+        assert_eq!(
+            m.raices_de_cache(),
+            vec![
+                PathBuf::from("/h/Library/Caches/taskcodetwo"),
+                PathBuf::from("/h/.cache/taskcodetwo")
+            ]
+        );
+    }
+}
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 485290e..0d9264c 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -10,7 +10,8 @@
 //! F2-8), [`version`] (`version --json` y `compatible`, F2-12) y
 //! [`actualizar_cli`] (`actualizar --cli`, F2-12) e [`identidad`] (quien
 //! ejecuta y `tc2 identidad`, F2-13) y [`huella`] (los nombres que la 2.0
-//! deja en un repo, F1-12 D1a). Las capas
+//! deja en un repo, F1-12 D1a) e [`instalacion`] (donde vive el `tc2` de la
+//! maquina, F2-12 y F2-14) y [`desinstalar`] (quitar TaskCodeTwo, F2-14). Las capas
 //! que vienen entran como modulos de esta biblioteca.
 
 pub mod actualizar_cli;
@@ -19,11 +20,13 @@ pub mod carril;
 pub mod config;
 pub mod contrato;
 pub mod descarga;
+pub mod desinstalar;
 pub mod estado;
 pub mod git;
 pub mod huella;
 pub mod i18n;
 pub mod identidad;
+pub mod instalacion;
 pub mod plataforma;
 pub mod tarea;
 pub mod version;
@@ -113,6 +116,10 @@ where
             let (json, resto) = con_json(resto);
             actualizar_cli::orden(&resto).salida_en(json, idioma)
         }
+        [orden, resto @ ..] if orden == desinstalar::NOMBRE => {
+            let (json, resto) = con_json(resto);
+            desinstalar::orden(&resto, json, idioma)
+        }
         [orden, resto @ ..] if orden == identidad::NOMBRE => {
             let (json, resto) = con_json(resto);
             orden_identidad(&resto).salida_en(json, idioma)
diff --git a/xtask/src/instaladores.rs b/xtask/src/instaladores.rs
index f7a445b..c3e0bdd 100644
--- a/xtask/src/instaladores.rs
+++ b/xtask/src/instaladores.rs
@@ -149,3 +149,30 @@ fn instaladores_ascii_y_con_la_llamada_en_la_ultima_linea() {
         .collect();
     assert!(con_exit.is_empty(), "install.ps1 usa exit: {con_exit:?}");
 }
+
+/// TASK-078 (F2-14): `install.sh` escribe el bloque del perfil con las
+/// marcas y la linea que `tc2 desinstalar` reconoce (constantes de
+/// `distribucion`), y `install.ps1` anade el directorio al PATH del usuario
+/// con `Add-PathUsuario`, cuya inversa hace `tc2 desinstalar`.
+#[test]
+fn install_sh_usa_las_marcas_que_quita_desinstalar() {
+    let sh = leer("install.sh");
+    assert!(
+        sh.contains(&format!(
+            "MARCA_INICIO=\"{}\"",
+            distribucion::MARCA_INICIO_PERFIL
+        )),
+        "install.sh no usa MARCA_INICIO_PERFIL"
+    );
+    assert!(
+        sh.contains(&format!("MARCA_FIN=\"{}\"", distribucion::MARCA_FIN_PERFIL)),
+        "install.sh no usa MARCA_FIN_PERFIL"
+    );
+    assert!(
+        sh.contains(&format!("printf '{}", distribucion::LINEA_PERFIL)),
+        "install.sh no escribe la linea del bloque"
+    );
+    let ps1 = leer("install.ps1");
+    assert!(ps1.contains("function Add-PathUsuario"));
+    assert!(ps1.contains(r"taskcodetwo\bin"));
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
tareas/{01-en-diseno => 02-en-curso}/TASK-078/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-078/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-078/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/desinstalar.rs"
````

````
crates/tc2/tests/desinstalar.rs | 785 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 785 insertions(+)
````
