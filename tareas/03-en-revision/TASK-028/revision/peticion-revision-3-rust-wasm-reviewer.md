# Peticion de revision — TASK-028 (ronda 3)

- Tarea: TASK-028 — F3-1 tc2 init
- Rama revisada: feature/task-028-f3-1-taskctl-init-setup
- Rama base: develop
- Commit revisado (HEAD): baef020e65d468e9fae8fcb9d6b62530091616dd
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-028 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 2da874858cbdd8149b00e8c3fa4fe864bd3a560a (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (9 hallazgo(s)))

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
informe de esta ronda (informe-revision-3-rust-wasm-reviewer.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-028/tarea.md
- tareas/03-en-revision/TASK-028/planificacion/
- crates/tc2/src/i18n/catalogo/init.rs
- crates/tc2/src/init/inventario.rs
- crates/tc2/src/init/mod.rs
- crates/tc2/tests/init.rs

## Hallazgos de la ronda 2 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 1 hallazgo(s) MENOR de la ronda 2 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | abierto | crates/tc2/src/init/inventario.rs:342 | informe-revision-2-rust-wasm-reviewer.md |

## Commits a revisar (git log 2da874858cbdd8149b00e8c3fa4fe864bd3a560a..HEAD)

````
baef020 fix(TASK-028): rama que falta desde origin, sin push si la historia local no tiene relacion, y comillas segun el shell
e1fcf95 chore(TASK-028): veredictos de la ronda 2 y ronda extra 3
078436c chore(TASK-028): informes de revision ronda 2
ed7869f chore(TASK-028): peticion de revision ronda 2
````

## Diff desde la ronda anterior (git diff 2da874858cbdd8149b00e8c3fa4fe864bd3a560a..HEAD)

````diff
diff --git a/crates/tc2/src/i18n/catalogo/init.rs b/crates/tc2/src/i18n/catalogo/init.rs
index a9ce500..973c81d 100644
--- a/crates/tc2/src/i18n/catalogo/init.rs
+++ b/crates/tc2/src/i18n/catalogo/init.rs
@@ -41,6 +41,10 @@ crate::i18n::textos! {
         es: "{ruta} cambio despues de hacer el plan",
         en: "{ruta} changed after the plan was made",
     }
+    AVISO_SIN_RELACION {
+        es: "aviso: la historia de este clon no tiene nada en comun con la de origin; init no propone publicar ninguna rama: decide tu si este clon es de ese remoto",
+        en: "warning: this clone's history has nothing in common with origin's; init does not suggest pushing any branch: decide yourself whether this clone belongs to that remote",
+    }
     AVISO_PERSONA {
         es: "{n} acciones las haces tu (decision=persona): cada linea trae la orden",
         en: "{n} actions are yours to do (decision=persona): each line has the command",
diff --git a/crates/tc2/src/init/inventario.rs b/crates/tc2/src/init/inventario.rs
index 9bafe8c..c649242 100644
--- a/crates/tc2/src/init/inventario.rs
+++ b/crates/tc2/src/init/inventario.rs
@@ -44,10 +44,16 @@ pub(super) fn misma_linea(a: &str, b: &str) -> bool {
     norm(a) == norm(b)
 }
 
-/// Orden entrecomillada para un shell POSIX (y PowerShell, salvo `'`):
-/// una ruta con espacios o `;` no parte la orden ni ejecuta nada.
-pub(super) fn comillas(s: &str) -> String {
-    format!("'{}'", s.replace('\'', "'\\''"))
+/// Argumento entrecomillado para el shell de la persona: una ruta con
+/// espacios, `;` o `'` no parte la orden ni ejecuta nada. Comillas simples
+/// en los dos, que no expanden nada; dentro, la `'` se escribe `''` en
+/// PowerShell (`windows`) y `'\''` en `sh`, cuya forma PowerShell trocearia.
+pub(super) fn comillas(s: &str, windows: bool) -> String {
+    if windows {
+        format!("'{}'", s.replace('\'', "''"))
+    } else {
+        format!("'{}'", s.replace('\'', "'\\''"))
+    }
 }
 
 fn sinred(detalle: &str) -> Informe {
@@ -193,7 +199,7 @@ pub(super) fn plan(c: &Contexto<'_>) -> Result<Vec<Accion>, Informe> {
         .motivo("montada-en-otra-ruta")
         .orden(format!(
             "git worktree move {} {}",
-            comillas(&t(&r)),
+            comillas(&t(&r), cfg!(windows)),
             CARPETA
         )),
         Previsto::SinRed(_) | Previsto::Denegada(..) => {
@@ -242,9 +248,9 @@ fn rechazo_016(ruta: String, clase: &'static str) -> Accion {
     a
 }
 
-/// Todas las ramas que tiene `origin` (para saber si tiene historia, no solo
-/// `main` y `develop`). Sin red, `sinred`.
-fn ramas_remotas(c: &Contexto<'_>, principal: &Path) -> Result<Vec<String>, Informe> {
+/// Todas las ramas que tiene `origin`, `(nombre, sha)` (para saber si tiene
+/// historia, no solo `main` y `develop`). Sin red, `sinred`.
+fn ramas_remotas(c: &Contexto<'_>, principal: &Path) -> Result<Vec<(String, String)>, Informe> {
     let s = correr(
         c.git,
         principal,
@@ -261,24 +267,77 @@ fn ramas_remotas(c: &Contexto<'_>, principal: &Path) -> Result<Vec<String>, Info
     }
     Ok(s.stdout
         .lines()
-        .filter_map(|l| l.split_whitespace().nth(1))
-        .filter_map(|r| r.strip_prefix("refs/heads/"))
-        .map(str::to_owned)
+        .filter_map(|l| l.split_once(char::is_whitespace))
+        .filter_map(|(sha, r)| {
+            r.trim()
+                .strip_prefix("refs/heads/")
+                .map(|n| (n.to_owned(), sha.to_owned()))
+        })
         .collect())
 }
 
+/// Si la historia local (HEAD, o la primera rama local) tiene algun ancestro
+/// comun con alguna rama de `origin`. Solo con lo que ya hay en el clon (sin
+/// `fetch`: `--simular` no escribe): la rama remota que se trajo o el sha de
+/// `ls-remote` si ese commit ya esta. Si no se puede saber, `false`: es el
+/// lado prudente (no se propone publicar).
+fn historia_relacionada(
+    c: &Contexto<'_>,
+    principal: &Path,
+    remotas: &[(&str, &str)],
+) -> Result<bool, Informe> {
+    let g = c.git;
+    let primera_local = exigir(
+        g,
+        principal,
+        NOMBRE,
+        Subcomando::ForEachRef,
+        &["--count=1", "--format=%(objectname)", "refs/heads/"],
+    )?;
+    let Some(local) = commit_de(g, principal, NOMBRE, "HEAD")?
+        .or_else(|| Some(primera_local.trim().to_owned()).filter(|s| !s.is_empty()))
+    else {
+        return Ok(false);
+    };
+    for (nombre, sha) in remotas {
+        let traida = commit_de(
+            g,
+            principal,
+            NOMBRE,
+            &format!("refs/remotes/{REMOTO}/{nombre}"),
+        )?;
+        let conocida = commit_de(g, principal, NOMBRE, sha)?;
+        for candidato in [traida, conocida].into_iter().flatten() {
+            let s = correr(
+                g,
+                principal,
+                NOMBRE,
+                Subcomando::MergeBase,
+                &[&local, &candidato],
+            )?;
+            if s.exito() && !s.stdout.trim().is_empty() {
+                return Ok(true);
+            }
+        }
+    }
+    Ok(false)
+}
+
 /// Lineas de `main` y `develop` (D3). El repo cuenta como vacio solo si no
 /// tiene commits y `origin` no tiene ninguna rama (la de estado no cuenta:
 /// es huerfana). Vacio: un commit raiz con la configuracion y `.gitignore` y
 /// las dos ramas en el (publicadas si hay remoto). Con historia, local o en
 /// `origin`, la rama que falta la crea la persona desde la que ya hay: un
-/// commit raiz propio dejaria en el remoto dos historias sin relacion.
+/// commit raiz propio dejaria en el remoto dos historias sin relacion. Si
+/// `origin` tiene historia, se parte siempre de ella (`origin/<rama>`), y
+/// si la historia local no tiene relacion con la remota no se propone
+/// publicar nada (`motivo=historia-sin-relacion`, con aviso).
 /// Devuelve si el repo estaba vacio.
 fn ramas(
     c: &Contexto<'_>,
     principal: &Path,
     remoto: bool,
-    remotas: &[String],
+    remotas: &[(String, String)],
     v: &mut Vec<Accion>,
 ) -> Result<bool, Informe> {
     let g = c.git;
@@ -301,12 +360,17 @@ fn ramas(
         }
     }
     // Ramas de `origin` con historia del proyecto (sin la de estado).
-    let historia_remota: Vec<&str> = remotas
+    let con_sha: Vec<(&str, &str)> = remotas
         .iter()
-        .map(String::as_str)
-        .filter(|r| *r != crate::huella::RAMA_ESTADO)
+        .map(|(n, s)| (n.as_str(), s.as_str()))
+        .filter(|(n, _)| *n != crate::huella::RAMA_ESTADO)
         .collect();
+    let historia_remota: Vec<&str> = con_sha.iter().map(|(n, _)| *n).collect();
     let vacio = !hay_commits && historia_remota.is_empty();
+    // Con historia en los dos lados, que tengan algo en comun.
+    let sin_relacion = hay_commits
+        && !historia_remota.is_empty()
+        && !historia_relacionada(c, principal, &con_sha)?;
     let mut nuevas: Vec<String> = Vec::new();
     let mut lineas = Vec::new();
     for r in RAMAS {
@@ -315,7 +379,12 @@ fn ramas(
         let en_remoto = historia_remota.contains(&r);
         let a = Accion::nueva(ruta, "rama", "proyecto", Decision::Existe);
         lineas.push(if local {
-            if remoto && !en_remoto {
+            if remoto && !en_remoto && sin_relacion {
+                // Publicarla subiria a `origin` una historia ajena a la suya.
+                let mut a = a.motivo("historia-sin-relacion");
+                a.decision = Decision::Persona;
+                a
+            } else if remoto && !en_remoto {
                 persona(a, "sin-publicar", format!("git push {REMOTO} {r}"))
             } else {
                 a
@@ -327,24 +396,25 @@ fn ramas(
                 format!("git fetch {REMOTO} {r} && git branch {r} {REMOTO}/{r}"),
             )
         } else if !vacio {
-            // Desde la otra rama local si existe (HEAD puede estar sin nacer
-            // en esta, tras un `init` cortado a medias); si no, desde la otra
-            // de `origin`, desde HEAD o desde la primera rama de `origin`.
+            // Si `origin` tiene historia, desde ella (la otra rama de codigo,
+            // o la primera que haya); si no, desde la otra rama local (HEAD
+            // puede estar sin nacer en esta, tras un `init` cortado a medias)
+            // o desde HEAD.
             let otra_remota = RAMAS
                 .iter()
                 .find(|x| historia_remota.contains(x))
                 .or_else(|| historia_remota.first());
-            let mut o = match (locales.first(), otra_remota) {
-                (Some(l), _) => format!("git branch {r} {l}"),
-                (None, Some(b)) if !hay_commits => {
-                    format!("git fetch {REMOTO} {b} && git branch {r} {REMOTO}/{b}")
-                }
-                _ => format!("git branch {r} HEAD"),
+            let mut o = match (otra_remota, locales.first()) {
+                (Some(b), _) => format!("git fetch {REMOTO} {b} && git branch {r} {REMOTO}/{b}"),
+                (None, Some(l)) => format!("git branch {r} {l}"),
+                (None, None) => format!("git branch {r} HEAD"),
             };
-            if remoto {
+            if remoto && !sin_relacion {
                 o.push_str(&format!(" && git push {REMOTO} {r}"));
             }
-            let motivo = if hay_commits {
+            let motivo = if sin_relacion {
+                "historia-sin-relacion"
+            } else if hay_commits {
                 "con-historia"
             } else {
                 "remoto-con-historia"
@@ -544,3 +614,38 @@ fn puertas(plataforma: &Conexion, v: &mut Vec<Accion>) {
         });
     }
 }
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    /// Revision 2, MEN-5: en `sh`, la `'` se cierra, se escapa y se reabre;
+    /// el `;` queda dentro de las comillas.
+    #[test]
+    fn comillas_para_sh() {
+        assert_eq!(comillas("/r/a b", false), "'/r/a b'");
+        assert_eq!(
+            comillas("/r/a'; echo X; 'b", false),
+            "'/r/a'\\''; echo X; '\\''b'"
+        );
+    }
+
+    /// Revision 2, MEN-5: en PowerShell la `'` se dobla (`''`) dentro de
+    /// comillas simples, que no expanden `$` ni `` ` ``; el `;` queda
+    /// dentro.
+    #[test]
+    fn comillas_para_powershell() {
+        assert_eq!(comillas("C:/r/a b", true), "'C:/r/a b'");
+        assert_eq!(
+            comillas("C:/r/a'; calc; 'b $x `y", true),
+            "'C:/r/a''; calc; ''b $x `y'"
+        );
+    }
+
+    #[test]
+    fn misma_regla_con_o_sin_barra_final() {
+        assert!(misma_linea("/tareas", "/tareas/"));
+        assert!(misma_linea("  /tareas/ ", "/tareas/"));
+        assert!(!misma_linea("/tareas-viejas/", "/tareas/"));
+    }
+}
diff --git a/crates/tc2/src/init/mod.rs b/crates/tc2/src/init/mod.rs
index 9ccf451..dbc7b0d 100644
--- a/crates/tc2/src/init/mod.rs
+++ b/crates/tc2/src/init/mod.rs
@@ -291,6 +291,14 @@ impl Hecho {
             );
             stderr.push('\n');
         }
+        if self
+            .acciones
+            .iter()
+            .any(|a| a.motivo.as_deref() == Some("historia-sin-relacion"))
+        {
+            stderr.push_str(&Mensaje::nuevo(&txt::AVISO_SIN_RELACION).texto(idioma));
+            stderr.push('\n');
+        }
         stderr.push_str(&fin.stderr);
         Salida {
             codigo: fin.codigo,
````

## Excluido del diff (5 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 2da874858cbdd8149b00e8c3fa4fe864bd3a560a..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/03-en-revision/TASK-028/revision/informe-revision-2-code-quality-reviewer.md  |  89 ++++++++++++++++++
 tareas/03-en-revision/TASK-028/revision/informe-revision-2-rust-wasm-reviewer.md     | 126 +++++++++++++++++++++++++
 tareas/03-en-revision/TASK-028/revision/peticion-revision-2-code-quality-reviewer.md | 545 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-028/revision/peticion-revision-2-rust-wasm-reviewer.md    | 545 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-028/tarea.md                                              |  12 ++-
 5 files changed, 1315 insertions(+), 2 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff 2da874858cbdd8149b00e8c3fa4fe864bd3a560a..HEAD -- ":(literal)crates/tc2/tests/init.rs"
````

````
crates/tc2/tests/init.rs | 73 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 73 insertions(+)
````
