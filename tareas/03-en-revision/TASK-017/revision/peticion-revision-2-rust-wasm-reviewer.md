# Peticion de revision — TASK-017 (ronda 2)

- Tarea: TASK-017 — F2-2 Capa de operaciones Git deterministas
- Rama revisada: feature/task-017-f2-2-capa-de-operaciones-git-determinist
- Rama base: develop
- Commit revisado (HEAD): c943b1c5a8f3575a0fb320fbed783ac55f3ebb1a
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-017 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 7d2d4c06fe9dd81556612abcb562219d65e6fa8a (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (7 hallazgo(s)))

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

- tareas/03-en-revision/TASK-017/tarea.md
- tareas/03-en-revision/TASK-017/planificacion/
- crates/tc2/src/git/guardar.rs
- crates/tc2/src/git/puerto.rs
- crates/tc2/src/git/remoto.rs
- crates/tc2/src/git/tag.rs
- crates/tc2/src/git/worktree.rs
- crates/tc2/tests/git_casos_limite.rs

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 4 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMPORTANTE-1 | IMPORTANTE | corregido | crates/tc2/src/git/worktree.rs:45 | informe-revision-1-rust-wasm-reviewer.md |
| IMPORTANTE-2 | IMPORTANTE | corregido | crates/tc2/src/git/guardar.rs:121 | informe-revision-1-rust-wasm-reviewer.md |
| IMPORTANTE-3 | IMPORTANTE | corregido | crates/tc2/src/git/puerto.rs:219 | informe-revision-1-rust-wasm-reviewer.md |

## Commits a revisar (git log 7d2d4c06fe9dd81556612abcb562219d65e6fa8a..HEAD)

````
c943b1c chore(TASK-017): autorrevision ronda 2
ca600a5 fix(TASK-017): correcciones de la revision ronda 1
bcc43ba chore(TASK-017): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 7d2d4c06fe9dd81556612abcb562219d65e6fa8a..HEAD)

````diff
diff --git a/crates/tc2/src/git/guardar.rs b/crates/tc2/src/git/guardar.rs
index 881e33c..de4a337 100644
--- a/crates/tc2/src/git/guardar.rs
+++ b/crates/tc2/src/git/guardar.rs
@@ -75,11 +75,11 @@ fn guardar_paso(git: &Git, dir: &Path, mensaje: &str, rutas: &[&str]) -> Paso<In
             Subcomando::Status,
             &["--porcelain", "-z", "--untracked-files=all"],
         )?;
-        let utiles = conocidas(dir, rutas, &desde_raiz, &entradas_status(&estado));
-        if !utiles.is_empty() {
-            let mut add = vec!["--"];
-            add.extend_from_slice(&utiles);
-            exigir(git, dir, OP, Subcomando::Add, &add)?;
+        let (add, _) = repartir(dir, rutas, &desde_raiz, &entradas_con_origen(&estado));
+        if !add.is_empty() {
+            let mut a = vec!["--"];
+            a.extend_from_slice(&add);
+            exigir(git, dir, OP, Subcomando::Add, &a)?;
         }
         // Durante un merge el commit es del indice entero (el merge), no
         // parcial: git no admite rutas aqui.
@@ -94,22 +94,23 @@ fn guardar_paso(git: &Git, dir: &Path, mensaje: &str, rutas: &[&str]) -> Paso<In
 
     let mut status = vec!["--porcelain", "-z", "--untracked-files=all"];
     status.extend_from_slice(&args);
-    let entradas = entradas_status(&exigir(git, dir, OP, Subcomando::Status, &status)?);
-    let cambios = entradas.len();
+    let salida = exigir(git, dir, OP, Subcomando::Status, &status)?;
+    let cambios = entradas_status(&salida).len();
     if cambios == 0 {
         return Ok(Informe::nuevo(OP, Resultado::SinCambios)
             .con("sin-guardar", sin_guardar(git, dir)?.to_string()));
     }
-    // Solo las rutas que git conoce o que existen: una ruta sin cambios ni
-    // fichero haria fallar el `add` entero. `add -- <rutas>` recoge tambien
-    // los borrados (git >= 2.0) y nunca toca otras rutas.
-    let utiles = conocidas(dir, rutas, &desde_raiz, &entradas);
-    let mut add = vec!["--"];
-    add.extend_from_slice(&utiles);
-    exigir(git, dir, OP, Subcomando::Add, &add)?;
-    let mut commit = vec!["-q", "-m", mensaje, "--only", "--"];
-    commit.extend_from_slice(&utiles);
-    exigir(git, dir, OP, Subcomando::Commit, &commit)?;
+    let (add, commit) = repartir(dir, rutas, &desde_raiz, &entradas_con_origen(&salida));
+    if !add.is_empty() {
+        let mut a = vec!["--"];
+        a.extend_from_slice(&add);
+        exigir(git, dir, OP, Subcomando::Add, &a)?;
+    }
+    // `--only`: el commit lleva solo estas rutas aunque haya otras en el
+    // indice (RF-23).
+    let mut c = vec!["-q", "-m", mensaje, "--only", "--"];
+    c.extend_from_slice(&commit);
+    exigir(git, dir, OP, Subcomando::Commit, &c)?;
     let head = commit_de(git, dir, OP, "HEAD")?.unwrap_or_default();
     Ok(Informe::nuevo(OP, Resultado::Ok)
         .con("commit", corto(&head))
@@ -117,26 +118,60 @@ fn guardar_paso(git: &Git, dir: &Path, mensaje: &str, rutas: &[&str]) -> Paso<In
         .con("sin-guardar", sin_guardar(git, dir)?.to_string()))
 }
 
-/// Rutas que existen en disco o que aparecen en el status (borradas).
-fn conocidas<'a>(
+/// Reparte las rutas pedidas en las que hay que pasar al indice (`add`) y
+/// las que entran en el commit:
+/// - en disco: `add` y commit;
+/// - borrada en el worktree sin pasar al indice (` D`): `add` y commit;
+/// - borrada ya en el indice (`git rm`, origen de un `git mv`): solo commit,
+///   porque `add` de una ruta que no esta ni en disco ni en el indice falla;
+/// - desconocida para git y ausente del disco: se ignora (nada que guardar).
+fn repartir<'a>(
     dir: &Path,
     rutas: &[&'a str],
     desde_raiz: &[String],
     status: &[(String, String)],
-) -> Vec<&'a str> {
-    rutas
-        .iter()
-        .zip(desde_raiz)
-        .filter(|(r, raiz)| {
-            dir.join(r).symlink_metadata().is_ok()
-                || status.iter().any(|(_, s)| {
-                    s == *raiz
-                        || s.strip_prefix(raiz.as_str())
-                            .is_some_and(|resto| resto.starts_with('/'))
-                })
-        })
-        .map(|(r, _)| *r)
-        .collect()
+) -> (Vec<&'a str>, Vec<&'a str>) {
+    let (mut add, mut commit) = (Vec::new(), Vec::new());
+    for (r, raiz) in rutas.iter().zip(desde_raiz) {
+        let estados: Vec<&str> = status
+            .iter()
+            .filter(|(_, s)| {
+                s == raiz
+                    || s.strip_prefix(raiz.as_str())
+                        .is_some_and(|resto| resto.starts_with('/'))
+            })
+            .map(|(xy, _)| xy.as_str())
+            .collect();
+        if dir.join(r).symlink_metadata().is_ok() || estados.iter().any(|xy| xy.ends_with('D')) {
+            add.push(*r);
+            commit.push(*r);
+        } else if !estados.is_empty() {
+            commit.push(*r);
+        }
+    }
+    (add, commit)
+}
+
+/// Como [`entradas_status`], pero un renombrado tambien aporta su ruta de
+/// origen, como borrada en el indice (`D `).
+fn entradas_con_origen(salida: &str) -> Vec<(String, String)> {
+    let mut v = Vec::new();
+    let mut campos = salida.split('\0').filter(|c| !c.is_empty());
+    while let Some(c) = campos.next() {
+        let xy = c.get(..2).unwrap_or(c).to_owned();
+        let ruta = c.get(3..).unwrap_or("").to_owned();
+        // La copia (`C`) tambien trae el origen, pero el origen sigue ahi.
+        let origen = if xy.starts_with('R') || xy.starts_with('C') {
+            campos.next()
+        } else {
+            None
+        };
+        if let (Some(o), true) = (origen, xy.starts_with('R')) {
+            v.push(("D ".to_owned(), o.to_owned()));
+        }
+        v.push((xy, ruta));
+    }
+    v
 }
 
 /// Cambios que quedan en el worktree sin guardar (se informan, no se tocan).
diff --git a/crates/tc2/src/git/puerto.rs b/crates/tc2/src/git/puerto.rs
index 21543cd..220669e 100644
--- a/crates/tc2/src/git/puerto.rs
+++ b/crates/tc2/src/git/puerto.rs
@@ -212,35 +212,27 @@ impl Git {
 }
 
 /// Argumentos prohibidos dentro de los subcomandos permitidos (F1-4,
-/// decision 3). Se mira cada argumento entero, no subcadenas (H9).
+/// decision 3). Se mira cada argumento entero, no subcadenas (H9). No se
+/// miran el texto de un mensaje (`-m <texto>` en `commit` y `tag`) ni las
+/// rutas tras `--` en los subcomandos que las aceptan (`add`, `commit`,
+/// `status`, `diff`): ahi una palabra como `--force` es dato, no opcion.
 fn comprobar_argumentos(sub: Subcomando, args: &[&str]) -> Result<(), ErrorGit> {
     let prohibido = |a: &str| Err(ErrorGit::Prohibido(format!("{} {a}", sub.nombre())));
-    for &a in args {
-        if a.starts_with("--force") {
-            return prohibido(a);
+    let con_rutas = matches!(
+        sub,
+        Subcomando::Add | Subcomando::Commit | Subcomando::Status | Subcomando::Diff
+    );
+    let con_mensaje = matches!(sub, Subcomando::Commit | Subcomando::Tag);
+    let mut iter = args.iter().copied();
+    while let Some(a) = iter.next() {
+        if a == "--" && con_rutas {
+            break;
         }
-        let malo = match sub {
-            Subcomando::Push => {
-                a == "-f"
-                    || a == "-d"
-                    || a == "--delete"
-                    || a == "--mirror"
-                    || a == "--prune"
-                    || a == "--all"
-                    || a.starts_with('+')
-                    || a.starts_with(':')
-            }
-            Subcomando::Branch => matches!(
-                a,
-                "-f" | "-d" | "-D" | "-m" | "-M" | "-c" | "-C" | "--delete" | "--move" | "--copy"
-            ),
-            Subcomando::Worktree => a == "-f",
-            Subcomando::Tag => matches!(a, "-d" | "-f" | "--delete"),
-            Subcomando::Merge => a == "--abort",
-            Subcomando::Commit => a == "--amend",
-            _ => false,
-        };
-        if malo {
+        if a == "-m" && con_mensaje {
+            iter.next();
+            continue;
+        }
+        if es_prohibido(sub, a) {
             return prohibido(a);
         }
     }
@@ -250,46 +242,114 @@ fn comprobar_argumentos(sub: Subcomando, args: &[&str]) -> Result<(), ErrorGit>
     Ok(())
 }
 
+fn es_prohibido(sub: Subcomando, a: &str) -> bool {
+    if a.starts_with("--force") {
+        return true;
+    }
+    match sub {
+        Subcomando::Push => {
+            matches!(
+                a,
+                "-f" | "-d" | "--delete" | "--mirror" | "--prune" | "--all"
+            ) || a.starts_with('+')
+                || a.starts_with(':')
+        }
+        Subcomando::Branch => matches!(
+            a,
+            "-f" | "-d" | "-D" | "-m" | "-M" | "-c" | "-C" | "--delete" | "--move" | "--copy"
+        ),
+        Subcomando::Worktree => a == "-f",
+        Subcomando::Tag => matches!(a, "-d" | "-f" | "--delete"),
+        Subcomando::Merge => a == "--abort",
+        Subcomando::Commit => a == "--amend",
+        _ => false,
+    }
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
 
-    #[test]
-    fn rechaza_argumentos_prohibidos_sin_lanzar_git() {
-        // Binario inexistente: si llegara a lanzarse, el error seria `Lanzar`.
-        let git = Git {
+    /// Puerto con un binario inexistente: si una llamada llegara a lanzarse,
+    /// el error seria `Lanzar`, no `Prohibido`.
+    fn git_sin_binario() -> Git {
+        Git {
             binario: OsString::from("no-existe-este-git"),
             entorno_extra: Vec::new(),
             registro: RefCell::new(Vec::new()),
-        };
-        let casos: [(Subcomando, &[&str]); 12] = [
-            (Subcomando::Push, &["origin", "+r:r"]),
+        }
+    }
+
+    #[test]
+    fn rechaza_cada_argumento_prohibido_sin_lanzar_git() {
+        let git = git_sin_binario();
+        // Una fila por regla de `es_prohibido` y de la decision 3 de F1-4,
+        // formas cortas y largas.
+        let casos: &[(Subcomando, &[&str])] = &[
+            (Subcomando::Push, &["--force", "origin", "r"]),
             (Subcomando::Push, &["--force-with-lease", "origin", "r"]),
-            (Subcomando::Push, &["origin", ":r"]),
-            (Subcomando::Push, &["--mirror", "origin"]),
             (Subcomando::Push, &["-f", "origin", "r"]),
-            (Subcomando::Branch, &["-D", "r"]),
+            (Subcomando::Push, &["-d", "origin", "r"]),
+            (Subcomando::Push, &["--delete", "origin", "r"]),
+            (Subcomando::Push, &["--mirror", "origin"]),
+            (Subcomando::Push, &["--prune", "origin"]),
+            (Subcomando::Push, &["--all", "origin"]),
+            (Subcomando::Push, &["origin", "+r:r"]),
+            (Subcomando::Push, &["origin", ":r"]),
             (Subcomando::Branch, &["-f", "r", "HEAD"]),
+            (Subcomando::Branch, &["--force", "r", "HEAD"]),
+            (Subcomando::Branch, &["-d", "r"]),
+            (Subcomando::Branch, &["-D", "r"]),
+            (Subcomando::Branch, &["--delete", "r"]),
+            (Subcomando::Branch, &["-m", "r", "s"]),
+            (Subcomando::Branch, &["-M", "r", "s"]),
+            (Subcomando::Branch, &["--move", "r", "s"]),
+            (Subcomando::Branch, &["-c", "r", "s"]),
+            (Subcomando::Branch, &["-C", "r", "s"]),
+            (Subcomando::Branch, &["--copy", "r", "s"]),
+            (Subcomando::Worktree, &["remove", "-f", "w"]),
             (Subcomando::Worktree, &["remove", "--force", "w"]),
             (Subcomando::Tag, &["-d", "v1"]),
+            (Subcomando::Tag, &["--delete", "v1"]),
+            (Subcomando::Tag, &["-f", "v1"]),
+            (Subcomando::Tag, &["--force", "v1"]),
             (Subcomando::Merge, &["--abort"]),
-            (Subcomando::Pull, &["origin", "develop"]),
             (Subcomando::Commit, &["--amend", "-m", "x"]),
+            (Subcomando::Pull, &["origin", "develop"]),
         ];
         for (sub, args) in casos {
-            let r = git.ejecutar(Path::new("."), sub, args);
+            let r = git.ejecutar(Path::new("."), *sub, args);
             assert!(
                 matches!(r, Err(ErrorGit::Prohibido(_))),
                 "{sub:?} {args:?}: {r:?}"
             );
         }
         assert!(git.registro().is_empty());
-        // Un argumento permitido si intenta lanzar (y falla por el binario).
-        let r = git.ejecutar(
-            Path::new("."),
-            Subcomando::Push,
-            &["--porcelain", "origin", "a:b"],
-        );
-        assert!(matches!(r, Err(ErrorGit::Lanzar(_))));
+    }
+
+    #[test]
+    fn mensajes_y_rutas_no_son_opciones() {
+        let git = git_sin_binario();
+        let permitidos: &[(Subcomando, &[&str])] = &[
+            (Subcomando::Push, &["--porcelain", "origin", "a:b"]),
+            (
+                Subcomando::Commit,
+                &["-q", "-m", "--force no se usa", "--only", "--", "a.txt"],
+            ),
+            (Subcomando::Add, &["--", "--force-x.txt"]),
+            (Subcomando::Status, &["--porcelain", "--", "--amend"]),
+            (Subcomando::Tag, &["-a", "-m", "-d", "--", "v1", "abc"]),
+            (Subcomando::Pull, &["-q", "--ff-only", "origin", "develop"]),
+        ];
+        for (sub, args) in permitidos {
+            let r = git.ejecutar(Path::new("."), *sub, args);
+            assert!(
+                matches!(r, Err(ErrorGit::Lanzar(_))),
+                "{sub:?} {args:?}: {r:?}"
+            );
+        }
+        // Tras `--` en push no hay rutas: se sigue mirando.
+        let r = git.ejecutar(Path::new("."), Subcomando::Push, &["origin", "--", "+r:r"]);
+        assert!(matches!(r, Err(ErrorGit::Prohibido(_))));
     }
 }
diff --git a/crates/tc2/src/git/remoto.rs b/crates/tc2/src/git/remoto.rs
index 8987859..96a07b1 100644
--- a/crates/tc2/src/git/remoto.rs
+++ b/crates/tc2/src/git/remoto.rs
@@ -59,7 +59,9 @@ enum Intento {
     Publicado,
     AlDia,
     NoFastForward,
-    Fallo(String),
+    /// Fallo con su resumen; `true` si el remoto respondio y lo rechazo
+    /// (`[remote rejected]`: un hook o una regla), no un fallo de red.
+    Fallo(String, bool),
 }
 
 fn intento_push(git: &Git, dir: &Path, op: &str, refspec: &str) -> Paso<Intento> {
@@ -81,16 +83,15 @@ fn intento_push(git: &Git, dir: &Path, op: &str, refspec: &str) -> Paso<Intento>
         Some('=') if s.exito() => Intento::AlDia,
         Some(' ' | '*') if s.exito() => Intento::Publicado,
         Some('!') if resumen.starts_with("[rejected]") => Intento::NoFastForward,
-        _ => Intento::Fallo(if resumen.is_empty() {
-            s.primera_linea_error()
-        } else {
-            resumen.to_owned()
-        }),
+        _ if resumen.is_empty() => Intento::Fallo(s.primera_linea_error(), false),
+        _ => Intento::Fallo(resumen.to_owned(), resumen.starts_with("[remote rejected]")),
     })
 }
 
 /// Publica `refspec` con reintentos ante fallos transitorios. Un
-/// non-fast-forward no se reintenta. Devuelve el resultado y los intentos.
+/// non-fast-forward no se reintenta. Agotados los intentos: `agotada` si el
+/// remoto respondio rechazando (hook o regla: con red no se arregla) y
+/// `sinred` si no. Devuelve el resultado, los intentos y el ultimo fallo.
 pub(crate) fn publicar(
     git: &Git,
     dir: &Path,
@@ -99,6 +100,7 @@ pub(crate) fn publicar(
     esperas: &[Duration],
 ) -> Paso<(Resultado, usize, String)> {
     let mut ultimo = String::new();
+    let mut rechazo_remoto = false;
     for intento in 0..=esperas.len() {
         match intento_push(git, dir, op, refspec)? {
             Intento::AlDia => return Ok((Resultado::SinCambios, intento + 1, String::new())),
@@ -106,13 +108,21 @@ pub(crate) fn publicar(
             Intento::NoFastForward => {
                 return Ok((Resultado::Rechazada, intento + 1, String::new()));
             }
-            Intento::Fallo(m) => ultimo = m,
+            Intento::Fallo(m, remoto) => {
+                ultimo = m;
+                rechazo_remoto = remoto;
+            }
         }
         if let Some(e) = esperas.get(intento) {
             std::thread::sleep(*e);
         }
     }
-    Ok((Resultado::Sinred, esperas.len() + 1, ultimo))
+    let r = if rechazo_remoto {
+        Resultado::Agotada
+    } else {
+        Resultado::Sinred
+    };
+    Ok((r, esperas.len() + 1, ultimo))
 }
 
 /// Publica la rama actual en `origin` con el mismo nombre, sin force.
@@ -135,6 +145,12 @@ fn push_paso(git: &Git, dir: &Path, op: &str, esperas: &[Duration]) -> Paso<Info
         Resultado::Rechazada => i
             .con("motivo", "no-fast-forward")
             .diagnostico("lanza actualizar-rama"),
+        Resultado::Agotada => i
+            .con("motivo", "remoto-rechaza")
+            .con("intentos", intentos.to_string())
+            .diagnostico(format!(
+                "push: {ultimo}; el remoto rechaza el push: revisa sus reglas o avisa a la persona"
+            )),
         _ => i
             .con("intentos", intentos.to_string())
             .diagnostico(format!("push: {ultimo}; reintenta con red")),
@@ -232,12 +248,16 @@ fn actualizar(git: &Git, dir: &Path, esperas: &[Duration]) -> Paso<Informe> {
         if es_ancestro(git, dir, OP, fuente, "HEAD")? {
             continue;
         }
+        // La base entra siempre con merge commit (queda constancia de la
+        // actualizacion); lo publicado en la propia rama, por fast-forward si
+        // se puede, para no ensuciar el historial.
+        let modo = if fuente == base { "--no-ff" } else { "--ff" };
         let s = correr(
             git,
             dir,
             OP,
             Subcomando::Merge,
-            &["-q", "--no-edit", "--no-ff", fuente],
+            &["-q", "--no-edit", modo, fuente],
         )?;
         if !s.exito() {
             if merge_abierto(git, dir, OP)? {
@@ -265,12 +285,17 @@ fn actualizar(git: &Git, dir: &Path, esperas: &[Duration]) -> Paso<Informe> {
                         "alguien publico en la rama entre medias: vuelve a lanzar actualizar-rama",
                     ));
             }
-            _ => {
-                return Ok(Informe::nuevo(OP, Resultado::Sinred)
+            otro => {
+                let que_hacer = if otro == Resultado::Agotada {
+                    "el remoto rechaza el push: revisa sus reglas o avisa a la persona"
+                } else {
+                    "reintenta con red"
+                };
+                return Ok(Informe::nuevo(OP, otro)
                     .con("rama", rama.as_str())
                     .con("intentos", intentos.to_string())
                     .diagnostico(format!(
-                        "push: {ultimo}; la rama esta actualizada en local, reintenta con red"
+                        "push: {ultimo}; la rama esta actualizada en local; {que_hacer}"
                     )));
             }
         }
diff --git a/crates/tc2/src/git/tag.rs b/crates/tc2/src/git/tag.rs
index de5df46..784cc84 100644
--- a/crates/tc2/src/git/tag.rs
+++ b/crates/tc2/src/git/tag.rs
@@ -77,19 +77,12 @@ fn tag_paso(
         let refspec = format!("refs/tags/{tag}:refs/tags/{tag}");
         let (r, intentos, ultimo) = publicar(git, dir, OP, &refspec, esperas)?;
         if !r.es_exito() {
-            return Ok(Informe::nuevo(
-                OP,
-                if r == Resultado::Rechazada {
-                    Resultado::Rechazada
-                } else {
-                    Resultado::Sinred
-                },
-            )
-            .con("tag", tag)
-            .con("intentos", intentos.to_string())
-            .diagnostico(format!(
-                "no se pudo publicar el tag: {ultimo}; el tag queda en local, relanza con red"
-            )));
+            return Ok(Informe::nuevo(OP, r)
+                .con("tag", tag)
+                .con("intentos", intentos.to_string())
+                .diagnostico(format!(
+                    "no se pudo publicar el tag: {ultimo}; el tag queda en local, relanza con red"
+                )));
         }
     }
     Ok(Informe::nuevo(OP, Resultado::Ok)
diff --git a/crates/tc2/src/git/worktree.rs b/crates/tc2/src/git/worktree.rs
index 4f7d8b8..2af2175 100644
--- a/crates/tc2/src/git/worktree.rs
+++ b/crates/tc2/src/git/worktree.rs
@@ -11,7 +11,7 @@ use super::{
 use crate::contrato::{Informe, Resultado};
 
 /// Quita `.taskcode/worktrees/<id>`. No existe → `sin-cambios`. Con cambios
-/// sin guardar o commits sin publicar → `rechazada`, sin tocar nada.
+/// sin guardar o commits sin publicar o HEAD suelto → `rechazada`, sin tocar nada.
 pub fn quitar_worktree(git: &Git, dir: &Path, id: &str) -> Informe {
     quitar(git, dir, id).unwrap_or_else(|i| i)
 }
@@ -42,28 +42,32 @@ fn quitar(git: &Git, dir: &Path, id: &str) -> Paso<Informe> {
             "guarda o descarta los cambios del worktree antes de quitarlo",
         ));
     }
-    if let Some(rama) = &w.rama {
-        let publicada = match commit_de(
-            git,
-            &principal,
-            OP,
-            &format!("refs/remotes/{REMOTO}/{rama}"),
-        )? {
-            Some(remota) => es_ancestro(git, &w.ruta, OP, "HEAD", &remota)?,
-            None => false,
-        };
-        if !publicada {
-            return Ok(rechazo(
-                "sin-publicar",
-                "publica la rama antes de quitar el worktree",
-            ));
-        }
+    // Con HEAD suelto no hay rama remota con la que comparar: sus commits se
+    // perderian al quitar el worktree.
+    let Some(rama) = &w.rama else {
+        return Ok(rechazo(
+            "sin-rama",
+            "el worktree tiene HEAD suelto: vuelve a la rama de la tarea",
+        ));
+    };
+    let publicada = match commit_de(
+        git,
+        &principal,
+        OP,
+        &format!("refs/remotes/{REMOTO}/{rama}"),
+    )? {
+        Some(remota) => es_ancestro(git, &w.ruta, OP, "HEAD", &remota)?,
+        None => false,
+    };
+    if !publicada {
+        return Ok(rechazo(
+            "sin-publicar",
+            "publica la rama antes de quitar el worktree",
+        ));
     }
     let rel = worktree_relativo(id);
     exigir(git, &principal, OP, Subcomando::Worktree, &["remove", &rel])?;
-    let i = Informe::nuevo(OP, Resultado::Ok).con("id", id);
-    Ok(match w.rama {
-        Some(r) => i.con("rama", r),
-        None => i,
-    })
+    Ok(Informe::nuevo(OP, Resultado::Ok)
+        .con("id", id)
+        .con("rama", rama.as_str()))
 }
````

## Excluido del diff (5 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 7d2d4c06fe9dd81556612abcb562219d65e6fa8a..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-017/planificacion/brainstorm/peticion-plan-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-017/planificacion/plan-final.md                 |    0
 tareas/03-en-revision/TASK-017/revision/informe-revision-1-rust-wasm-reviewer.md            |   21 +
 tareas/03-en-revision/TASK-017/revision/peticion-revision-1-rust-wasm-reviewer.md           | 2268 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-017/tarea.md                                    |   10 +-
 5 files changed, 2296 insertions(+), 3 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff 7d2d4c06fe9dd81556612abcb562219d65e6fa8a..HEAD -- ":(literal)crates/tc2/tests/git_casos_limite.rs"
````

````
crates/tc2/tests/git_casos_limite.rs | 154 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 154 insertions(+)
````
