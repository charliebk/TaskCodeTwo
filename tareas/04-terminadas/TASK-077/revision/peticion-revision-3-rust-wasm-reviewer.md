# Peticion de revision — TASK-077 (ronda 3)

- Tarea: TASK-077 — F2-13 Identidad de la persona desde la plataforma
- Rama revisada: feature/task-077-f2-13-identidad-de-la-persona-desde-la-p
- Rama base: develop
- Commit revisado (HEAD): 91f308772a3e21675c1a1cb6ce87fb26e7cb1a29
- Fecha: 2026-11-16
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-077 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 2fae3e4b7d864f66f5668dac349b21c345ce3f9a (el commit revisado en la ronda anterior)
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
informe de esta ronda (informe-revision-3-rust-wasm-reviewer.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-077/tarea.md
- tareas/03-en-revision/TASK-077/planificacion/
- crates/tc2/src/git/remoto.rs
- crates/tc2/src/i18n/catalogo/git.rs
- crates/tc2/src/i18n/catalogo/identidad.rs
- crates/tc2/src/identidad.rs
- crates/tc2/tests/identidad.rs

## Hallazgos de la ronda 2 pendientes de verificar

Tus 4 hallazgo(s) MENOR de la ronda 2 no se reverifican (los comprueba
quien coordina): recibes esta ronda porque la correccion
toca ficheros de tu dominio. Revisa en el diff de esta ronda lo que la correccion
cambia en tu terreno y si abre un fallo.

## Commits a revisar (git log 2fae3e4b7d864f66f5668dac349b21c345ce3f9a..HEAD)

````
91f3087 chore(TASK-077): ronda extra 3
66fed63 fix(TASK-077): mensaje de fuera de lista con terminal sin la pista de tc2 identidad y origin sin rama base como sin-rama-base
3fed357 chore(TASK-077): veredictos y coste de la revision ronda 2
f077ed6 chore(TASK-077): informes de revision ronda 2
8ea5cb4 chore(TASK-077): peticion de revision ronda 2
````

## Diff desde la ronda anterior (git diff 2fae3e4b7d864f66f5668dac349b21c345ce3f9a..HEAD)

````diff
diff --git a/crates/tc2/src/git/remoto.rs b/crates/tc2/src/git/remoto.rs
index 268dd06..54b14a2 100644
--- a/crates/tc2/src/git/remoto.rs
+++ b/crates/tc2/src/git/remoto.rs
@@ -181,7 +181,9 @@ fn fallo_fetch(op: &str, s: &SalidaGit) -> Informe {
 }
 
 /// Trae solo `rama` de `origin` (actualiza `origin/<rama>`), con la misma
-/// clasificacion de fallos que [`fetch`].
+/// clasificacion de fallos que [`fetch`], salvo que `origin` no tenga la
+/// rama: eso no es falta de red sino `rechazada motivo=sin-rama-base
+/// rama=<rama>`.
 pub(crate) fn traer_rama(git: &Git, dir: &Path, op: &str, rama: &str) -> Paso<()> {
     let s = correr(
         git,
@@ -191,10 +193,19 @@ pub(crate) fn traer_rama(git: &Git, dir: &Path, op: &str, rama: &str) -> Paso<()
         &["-q", "--end-of-options", REMOTO, rama],
     )?;
     if s.exito() {
-        Ok(())
-    } else {
-        Err(fallo_fetch(op, &s))
+        return Ok(());
+    }
+    if clasificar_fallo_remoto(&s.stderr) == FalloRemoto::RamaInexistente {
+        return Err(Informe::nuevo(op, Resultado::Rechazada)
+            .con("motivo", "sin-rama-base")
+            .con("rama", rama)
+            .diagnostico(
+                Mensaje::nuevo(&txt::SIN_RAMA_BASE)
+                    .con("rama", rama)
+                    .con("remoto", REMOTO),
+            ));
     }
+    Err(fallo_fetch(op, &s))
 }
 
 fn traer(git: &Git, dir: &Path, op: &str) -> Paso<Informe> {
diff --git a/crates/tc2/src/i18n/catalogo/git.rs b/crates/tc2/src/i18n/catalogo/git.rs
index 21ab19f..107b1e4 100644
--- a/crates/tc2/src/i18n/catalogo/git.rs
+++ b/crates/tc2/src/i18n/catalogo/git.rs
@@ -70,6 +70,10 @@ crate::i18n::textos! {
         es: "{op}: no se pudo leer {remoto}; reintenta con red",
         en: "{op}: could not read {remoto}; retry with network access",
     }
+    SIN_RAMA_BASE {
+        es: "{remoto} no tiene la rama base {rama}: creala y publicala (o revisa que {remoto} es el repo del proyecto); reintentar no lo arregla",
+        en: "{remoto} has no base branch {rama}: create and publish it (or check that {remoto} is the project's repo); retrying will not fix it",
+    }
     DENEGADA_CREDENCIALES {
         es: "{op}: {detalle}; revisa tus credenciales o permisos en el remoto",
         en: "{op}: {detalle}; check your credentials or permissions on the remote",
diff --git a/crates/tc2/src/i18n/catalogo/identidad.rs b/crates/tc2/src/i18n/catalogo/identidad.rs
index 3c10023..cdb570e 100644
--- a/crates/tc2/src/i18n/catalogo/identidad.rs
+++ b/crates/tc2/src/i18n/catalogo/identidad.rs
@@ -55,8 +55,8 @@ crate::i18n::textos! {
         en: "the account of the person token is web-flow, GitHub's web account: TC2_TOKEN_PERSONA must be your user's token",
     }
     CUENTA_FUERA_DE_LISTA {
-        es: "la cuenta {login} (id {id}) no esta en {ruta}: pide que la anadan con un PR, o usa el token de una persona de la lista; si ya la anadieron, ejecuta `tc2 identidad` (actualiza la lista)",
-        en: "account {login} (id {id}) is not in {ruta}: ask for it to be added with a PR, or use the token of a person on the list; if it was already added, run `tc2 identidad` (it updates the list)",
+        es: "la cuenta {login} (id {id}) no esta en {ruta} de la rama base actualizada ({base}): pide que la anadan con un PR, o usa el token de una persona de la lista",
+        en: "account {login} (id {id}) is not in {ruta} of the updated base branch ({base}): ask for it to be added with a PR, or use the token of a person on the list",
     }
     LOGIN_DE_LA_API {
         es: "la plataforma devolvio un login que no vale para {login} (id {id}): revisa la cuenta del token",
diff --git a/crates/tc2/src/identidad.rs b/crates/tc2/src/identidad.rs
index 483396b..f2bafb7 100644
--- a/crates/tc2/src/identidad.rs
+++ b/crates/tc2/src/identidad.rs
@@ -137,7 +137,8 @@ impl NoAsignable {
             Self::FueraDeLista => Mensaje::nuevo(&t::CUENTA_FUERA_DE_LISTA)
                 .con("login", login)
                 .con("id", u.id.to_string())
-                .con("ruta", RUTA_PERSONAS),
+                .con("ruta", RUTA_PERSONAS)
+                .con("base", config::BASE),
             Self::LoginNoValido => Mensaje::nuevo(&t::LOGIN_DE_LA_API)
                 .con("login", login)
                 .con("id", u.id.to_string()),
````

## Excluido del diff (5 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 2fae3e4b7d864f66f5668dac349b21c345ce3f9a..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/03-en-revision/TASK-077/revision/informe-revision-2-code-quality-reviewer.md  |  63 ++++++++++++++
 tareas/03-en-revision/TASK-077/revision/informe-revision-2-rust-wasm-reviewer.md     | 116 +++++++++++++++++++++++++
 tareas/03-en-revision/TASK-077/revision/peticion-revision-2-code-quality-reviewer.md | 506 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-077/revision/peticion-revision-2-rust-wasm-reviewer.md    | 502 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-077/tarea.md                                              |   9 +-
 5 files changed, 1195 insertions(+), 1 deletion(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff 2fae3e4b7d864f66f5668dac349b21c345ce3f9a..HEAD -- ":(literal)crates/tc2/tests/identidad.rs"
````

````
crates/tc2/tests/identidad.rs | 45 +++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 45 insertions(+)
````
