# Peticion de revision — TASK-028 (ronda 4)

- Tarea: TASK-028 — F3-1 tc2 init
- Rama revisada: feature/task-028-f3-1-taskctl-init-setup
- Rama base: develop
- Commit revisado (HEAD): c9b549c33536f3809efaa1417675c5750db90d33
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-028 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde baef020e65d468e9fae8fcb9d6b62530091616dd (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (8 hallazgo(s)))

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
informe de esta ronda (informe-revision-4-code-quality-reviewer.md), sin borrar la peticion.

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

## Hallazgos de la ronda 3 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 1 hallazgo(s) MENOR de la ronda 3 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/init/inventario.rs:369 | informe-revision-3-code-quality-reviewer.md |
| IMP-2 | IMPORTANTE | abierto | crates/tc2/src/i18n/catalogo/init.rs:44 | informe-revision-3-code-quality-reviewer.md |
| IMP-3 | IMPORTANTE | abierto | crates/tc2/src/init/inventario.rs:303 | informe-revision-3-code-quality-reviewer.md |

## Commits a revisar (git log baef020e65d468e9fae8fcb9d6b62530091616dd..HEAD)

````
c9b549c fix(TASK-028): relacion de historias en tres estados, avisos que dicen lo cierto sobre la rama de estado y orden en toda linea persona
6d7767a chore(TASK-028): veredictos de la ronda 3 y ronda extra 4
2309e4e chore(TASK-028): informes de revision ronda 3
5c1d4a5 chore(TASK-028): peticion de revision ronda 3
````

## Diff desde la ronda anterior (git diff baef020e65d468e9fae8fcb9d6b62530091616dd..HEAD)

````diff
diff --git a/crates/tc2/src/i18n/catalogo/init.rs b/crates/tc2/src/i18n/catalogo/init.rs
index 973c81d..dcddac2 100644
--- a/crates/tc2/src/i18n/catalogo/init.rs
+++ b/crates/tc2/src/i18n/catalogo/init.rs
@@ -42,8 +42,12 @@ crate::i18n::textos! {
         en: "{ruta} changed after the plan was made",
     }
     AVISO_SIN_RELACION {
-        es: "aviso: la historia de este clon no tiene nada en comun con la de origin; init no propone publicar ninguna rama: decide tu si este clon es de ese remoto",
-        en: "warning: this clone's history has nothing in common with origin's; init does not suggest pushing any branch: decide yourself whether this clone belongs to that remote",
+        es: "aviso: la historia de este clon no tiene nada en comun con la de origin; init no propone publicar ninguna rama de codigo (la rama de estado {rama}, huerfana, si se publica): decide tu si este clon es de ese remoto",
+        en: "warning: this clone's history has nothing in common with origin's; init does not suggest pushing any code branch (the state branch {rama}, an orphan, is pushed): decide yourself whether this clone belongs to that remote",
+    }
+    AVISO_NO_COMPROBADO {
+        es: "aviso: no se pudo comprobar si la historia de este clon tiene algo en comun con la de origin (faltan sus commits en el clon); init no propone publicar ninguna rama de codigo (la rama de estado {rama}, huerfana, si se publica): ejecuta git fetch origin y repite tc2 init",
+        en: "warning: could not check whether this clone's history has anything in common with origin's (its commits are missing from the clone); init does not suggest pushing any code branch (the state branch {rama}, an orphan, is pushed): run git fetch origin and run tc2 init again",
     }
     AVISO_PERSONA {
         es: "{n} acciones las haces tu (decision=persona): cada linea trae la orden",
diff --git a/crates/tc2/src/init/inventario.rs b/crates/tc2/src/init/inventario.rs
index c649242..dbe37fb 100644
--- a/crates/tc2/src/init/inventario.rs
+++ b/crates/tc2/src/init/inventario.rs
@@ -276,16 +276,46 @@ fn ramas_remotas(c: &Contexto<'_>, principal: &Path) -> Result<Vec<(String, Stri
         .collect())
 }
 
+/// Como esta la historia local frente a la de `origin`.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+enum Historia {
+    /// Tienen algun ancestro comun.
+    Relacionada,
+    /// Se comprobo cada rama de `origin` y ninguna tiene nada en comun.
+    SinRelacion,
+    /// Falta en el clon algun commit de `origin` con el que comparar (y con
+    /// los que hay, ninguno en comun): hace falta `git fetch origin`.
+    NoComprobable,
+}
+
+impl Historia {
+    /// Motivo de las lineas cuando no se propone publicar.
+    fn motivo(self) -> Option<&'static str> {
+        match self {
+            Self::Relacionada => None,
+            Self::SinRelacion => Some(MOTIVO_SIN_RELACION),
+            Self::NoComprobable => Some(MOTIVO_NO_COMPROBADO),
+        }
+    }
+}
+
+/// Motivo de una rama cuya historia no tiene nada en comun con `origin`.
+pub(super) const MOTIVO_SIN_RELACION: &str = "historia-sin-relacion";
+
+/// Motivo de una rama cuya relacion con `origin` no se pudo comprobar.
+pub(super) const MOTIVO_NO_COMPROBADO: &str = "no-comprobado";
+
 /// Si la historia local (HEAD, o la primera rama local) tiene algun ancestro
 /// comun con alguna rama de `origin`. Solo con lo que ya hay en el clon (sin
-/// `fetch`: `--simular` no escribe): la rama remota que se trajo o el sha de
-/// `ls-remote` si ese commit ya esta. Si no se puede saber, `false`: es el
-/// lado prudente (no se propone publicar).
+/// `fetch`: `--simular` no escribe): por cada rama de `origin`, la rama
+/// remota que se trajo y el sha de `ls-remote` si ese commit ya esta (aunque
+/// falte la ref remota, p. ej. traido por sha). Si alguna rama no tiene
+/// commit con el que comparar, no se afirma que no haya relacion.
 fn historia_relacionada(
     c: &Contexto<'_>,
     principal: &Path,
     remotas: &[(&str, &str)],
-) -> Result<bool, Informe> {
+) -> Result<Historia, Informe> {
     let g = c.git;
     let primera_local = exigir(
         g,
@@ -297,8 +327,9 @@ fn historia_relacionada(
     let Some(local) = commit_de(g, principal, NOMBRE, "HEAD")?
         .or_else(|| Some(primera_local.trim().to_owned()).filter(|s| !s.is_empty()))
     else {
-        return Ok(false);
+        return Ok(Historia::NoComprobable);
     };
+    let mut falta_alguna = false;
     for (nombre, sha) in remotas {
         let traida = commit_de(
             g,
@@ -307,6 +338,9 @@ fn historia_relacionada(
             &format!("refs/remotes/{REMOTO}/{nombre}"),
         )?;
         let conocida = commit_de(g, principal, NOMBRE, sha)?;
+        if traida.is_none() && conocida.is_none() {
+            falta_alguna = true;
+        }
         for candidato in [traida, conocida].into_iter().flatten() {
             let s = correr(
                 g,
@@ -316,11 +350,15 @@ fn historia_relacionada(
                 &[&local, &candidato],
             )?;
             if s.exito() && !s.stdout.trim().is_empty() {
-                return Ok(true);
+                return Ok(Historia::Relacionada);
             }
         }
     }
-    Ok(false)
+    Ok(if falta_alguna {
+        Historia::NoComprobable
+    } else {
+        Historia::SinRelacion
+    })
 }
 
 /// Lineas de `main` y `develop` (D3). El repo cuenta como vacio solo si no
@@ -367,10 +405,21 @@ fn ramas(
         .collect();
     let historia_remota: Vec<&str> = con_sha.iter().map(|(n, _)| *n).collect();
     let vacio = !hay_commits && historia_remota.is_empty();
-    // Con historia en los dos lados, que tengan algo en comun.
-    let sin_relacion = hay_commits
-        && !historia_remota.is_empty()
-        && !historia_relacionada(c, principal, &con_sha)?;
+    // Con historia en los dos lados, que tengan algo en comun. Si no la
+    // tienen, o no se pudo comprobar, no se propone publicar nada.
+    let relacion = if hay_commits && !historia_remota.is_empty() {
+        historia_relacionada(c, principal, &con_sha)?
+    } else {
+        Historia::Relacionada
+    };
+    let sin_publicar = relacion.motivo();
+    // Rama de `origin` desde la que partir (la otra rama de codigo, o la
+    // primera que haya).
+    let otra_remota = RAMAS
+        .iter()
+        .copied()
+        .find(|x| historia_remota.contains(x))
+        .or_else(|| historia_remota.first().copied());
     let mut nuevas: Vec<String> = Vec::new();
     let mut lineas = Vec::new();
     for r in RAMAS {
@@ -379,11 +428,15 @@ fn ramas(
         let en_remoto = historia_remota.contains(&r);
         let a = Accion::nueva(ruta, "rama", "proyecto", Decision::Existe);
         lineas.push(if local {
-            if remoto && !en_remoto && sin_relacion {
-                // Publicarla subiria a `origin` una historia ajena a la suya.
-                let mut a = a.motivo("historia-sin-relacion");
-                a.decision = Decision::Persona;
-                a
+            if let (true, false, Some(m)) = (remoto, en_remoto, sin_publicar) {
+                // Publicarla podria subir a `origin` una historia ajena a la
+                // suya: la orden trae el remoto y comprueba si tienen algo en
+                // comun (`merge-base` vacio: nada).
+                let comprobar = match otra_remota {
+                    Some(b) => format!("git fetch {REMOTO} && git merge-base {r} {REMOTO}/{b}"),
+                    None => format!("git fetch {REMOTO}"),
+                };
+                persona(a, m, comprobar)
             } else if remoto && !en_remoto {
                 persona(a, "sin-publicar", format!("git push {REMOTO} {r}"))
             } else {
@@ -396,24 +449,19 @@ fn ramas(
                 format!("git fetch {REMOTO} {r} && git branch {r} {REMOTO}/{r}"),
             )
         } else if !vacio {
-            // Si `origin` tiene historia, desde ella (la otra rama de codigo,
-            // o la primera que haya); si no, desde la otra rama local (HEAD
-            // puede estar sin nacer en esta, tras un `init` cortado a medias)
-            // o desde HEAD.
-            let otra_remota = RAMAS
-                .iter()
-                .find(|x| historia_remota.contains(x))
-                .or_else(|| historia_remota.first());
+            // Si `origin` tiene historia, desde ella; si no, desde la otra
+            // rama local (HEAD puede estar sin nacer en esta, tras un `init`
+            // cortado a medias) o desde HEAD.
             let mut o = match (otra_remota, locales.first()) {
                 (Some(b), _) => format!("git fetch {REMOTO} {b} && git branch {r} {REMOTO}/{b}"),
                 (None, Some(l)) => format!("git branch {r} {l}"),
                 (None, None) => format!("git branch {r} HEAD"),
             };
-            if remoto && !sin_relacion {
+            if remoto && sin_publicar.is_none() {
                 o.push_str(&format!(" && git push {REMOTO} {r}"));
             }
-            let motivo = if sin_relacion {
-                "historia-sin-relacion"
+            let motivo = if let Some(m) = sin_publicar {
+                m
             } else if hay_commits {
                 "con-historia"
             } else {
diff --git a/crates/tc2/src/init/mod.rs b/crates/tc2/src/init/mod.rs
index dbc7b0d..52a399f 100644
--- a/crates/tc2/src/init/mod.rs
+++ b/crates/tc2/src/init/mod.rs
@@ -291,13 +291,22 @@ impl Hecho {
             );
             stderr.push('\n');
         }
-        if self
-            .acciones
-            .iter()
-            .any(|a| a.motivo.as_deref() == Some("historia-sin-relacion"))
-        {
-            stderr.push_str(&Mensaje::nuevo(&txt::AVISO_SIN_RELACION).texto(idioma));
-            stderr.push('\n');
+        for (motivo, aviso) in [
+            (inventario::MOTIVO_SIN_RELACION, &txt::AVISO_SIN_RELACION),
+            (inventario::MOTIVO_NO_COMPROBADO, &txt::AVISO_NO_COMPROBADO),
+        ] {
+            if self
+                .acciones
+                .iter()
+                .any(|a| a.motivo.as_deref() == Some(motivo))
+            {
+                stderr.push_str(
+                    &Mensaje::nuevo(aviso)
+                        .con("rama", crate::huella::RAMA_ESTADO)
+                        .texto(idioma),
+                );
+                stderr.push('\n');
+            }
         }
         stderr.push_str(&fin.stderr);
         Salida {
````

## Excluido del diff (5 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff baef020e65d468e9fae8fcb9d6b62530091616dd..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/03-en-revision/TASK-028/revision/informe-revision-3-code-quality-reviewer.md  |  98 +++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-028/revision/informe-revision-3-rust-wasm-reviewer.md     | 117 ++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-028/revision/peticion-revision-3-code-quality-reviewer.md | 391 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-028/revision/peticion-revision-3-rust-wasm-reviewer.md    | 391 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-028/tarea.md                                              |   6 +-
 5 files changed, 1002 insertions(+), 1 deletion(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff baef020e65d468e9fae8fcb9d6b62530091616dd..HEAD -- ":(literal)crates/tc2/tests/init.rs"
````

````
crates/tc2/tests/init.rs | 188 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-----------------------
 1 file changed, 162 insertions(+), 26 deletions(-)
````
