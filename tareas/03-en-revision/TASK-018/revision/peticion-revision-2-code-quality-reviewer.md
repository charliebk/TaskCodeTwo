# Peticion de revision — TASK-018 (ronda 2)

- Tarea: TASK-018 — F2-3 Rama de estado
- Rama revisada: feature/task-018-f2-3-rama-de-estado
- Rama base: develop
- Commit revisado (HEAD): 09e487ed1a5f3f43221c6cb4a968ea0549237c73
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-018 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 4f80989fc93337d12966d31c9be87260371a8ddd (el commit revisado en la ronda anterior)
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
informe de esta ronda (informe-revision-2-code-quality-reviewer.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-018/tarea.md
- tareas/03-en-revision/TASK-018/planificacion/
- crates/tc2/src/estado/mod.rs
- crates/tc2/src/estado/montaje.rs
- crates/tc2/src/estado/transaccion.rs
- crates/tc2/src/git/estado.rs
- crates/tc2/src/git/remoto.rs
- crates/tc2/tests/comun/ops.rs
- crates/tc2/tests/estado_bordes.rs
- crates/tc2/tests/git_casos_limite.rs
- docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 2 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMPORTANTE-1 | IMPORTANTE | corregido | crates/tc2/src/estado/transaccion.rs (`intento`, `adelantar`) | informe-revision-1-code-quality-reviewer.md |

## Commits a revisar (git log 4f80989fc93337d12966d31c9be87260371a8ddd..HEAD)

````
09e487e chore(TASK-018): autorrevision ronda 2
f96b878 fix(TASK-018): correcciones de la revision ronda 1
97fbcfa chore(TASK-018): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 4f80989fc93337d12966d31c9be87260371a8ddd..HEAD)

````diff
diff --git a/crates/tc2/src/estado/mod.rs b/crates/tc2/src/estado/mod.rs
index df882a2..5e47ad5 100644
--- a/crates/tc2/src/estado/mod.rs
+++ b/crates/tc2/src/estado/mod.rs
@@ -27,7 +27,7 @@ mod transaccion;
 pub use crate::git::Cambio;
 pub use cerrojo::Cerrojo;
 pub use lectura::{Estado, Vista, leer};
-pub use montaje::montar;
+pub use montaje::{montar, montar_con};
 pub use transaccion::{Catalogo, Contexto, Fallo, Operacion, Politica, Roadmap, transaccion};
 
 /// Rama huerfana del estado (ADR F1-1).
diff --git a/crates/tc2/src/estado/montaje.rs b/crates/tc2/src/estado/montaje.rs
index 40b696f..0bbb944 100644
--- a/crates/tc2/src/estado/montaje.rs
+++ b/crates/tc2/src/estado/montaje.rs
@@ -7,11 +7,12 @@
 
 use std::path::Path;
 
+use super::cerrojo::{Cerrojo, ErrorCerrojo};
 use super::lectura::Estado;
 use super::transaccion::{Contexto, Fallo, Operacion, Politica, transaccion};
 use super::{CARPETA, RAMA};
 use crate::contrato::{Informe, Resultado, corto};
-use crate::git::estado::hay_remoto;
+use crate::git::estado::{dir_comun, hay_remoto};
 use crate::git::rama::excluir;
 use crate::git::{
     Cambio, FalloRemoto, Git, REMOTO, Subcomando, clasificar_fallo_remoto, commit_de, correr,
@@ -67,11 +68,27 @@ pub fn montar(git: &Git, dir: &Path) -> Informe {
 
 /// Como [`montar`], con otra politica de reintentos para crear la rama.
 pub fn montar_con(git: &Git, dir: &Path, politica: Politica) -> Informe {
-    paso(git, dir, politica).unwrap_or_else(renombrado)
+    paso(git, dir, politica, true).unwrap_or_else(renombrado)
 }
 
-fn paso(git: &Git, dir: &Path, politica: Politica) -> Result<Informe, Informe> {
+/// `puede_crear`: si puede crear la rama con una transaccion; tras crearla
+/// se vuelve a entrar una vez, ya sin poder crear, para montarla.
+fn paso(git: &Git, dir: &Path, politica: Politica, puede_crear: bool) -> Result<Informe, Informe> {
     let principal = localizar_principal(git, dir)?;
+    // El mismo cerrojo que la transaccion: la persona y sus agentes del mismo
+    // clon no montan a la vez (dos `worktree add` en la misma carpeta).
+    let comun = dir_comun(git, &principal, OP)?;
+    let cerrojo = match Cerrojo::tomar(&comun, politica.espera_cerrojo) {
+        Ok(c) => c,
+        Err(ErrorCerrojo::Ocupado) => {
+            return Ok(Informe::nuevo(OP, Resultado::Agotada)
+                .con("motivo", "cerrojo")
+                .diagnostico("otro proceso de este clon esta montando o escribiendo la rama de estado: reintenta"));
+        }
+        Err(ErrorCerrojo::Io(e)) => {
+            return Err(Informe::nuevo(OP, Resultado::Error).diagnostico(format!("cerrojo: {e}")));
+        }
+    };
     let carpeta = principal.join(CARPETA);
     let ok = |r: Resultado, commit: Option<String>| {
         let i = Informe::nuevo(OP, r).con("rama", RAMA).con("ruta", CARPETA);
@@ -154,6 +171,13 @@ fn paso(git: &Git, dir: &Path, politica: Politica) -> Result<Informe, Informe> {
         if en_remoto {
             exigir(git, &principal, OP, Subcomando::Branch, &[RAMA, &remota])?;
         } else {
+            if !puede_crear {
+                return Err(Informe::nuevo(OP, Resultado::Error)
+                    .diagnostico(format!("{RAMA} sigue sin existir tras crearla")));
+            }
+            // La transaccion toma el cerrojo ella misma: se suelta antes y,
+            // creada la rama, se vuelve a empezar para montarla.
+            drop(cerrojo);
             let mut c = Contexto::nuevo(git, &principal, &ninguna);
             c.politica = politica;
             let i = transaccion(&c, &Inicializar);
@@ -161,11 +185,7 @@ fn paso(git: &Git, dir: &Path, politica: Politica) -> Result<Informe, Informe> {
                 return Ok(renombrado(i));
             }
 
-            if commit_de(git, &principal, OP, &local)?.is_none() {
-                // Otro clon la creo entre medias: la transaccion la vio en
-                // el remoto y no escribio nada.
-                exigir(git, &principal, OP, Subcomando::Branch, &[RAMA, &remota])?;
-            }
+            return paso(git, dir, politica, false);
         }
     }
 
diff --git a/crates/tc2/src/estado/transaccion.rs b/crates/tc2/src/estado/transaccion.rs
index b78c067..c6004c6 100644
--- a/crates/tc2/src/estado/transaccion.rs
+++ b/crates/tc2/src/estado/transaccion.rs
@@ -271,7 +271,10 @@ fn intento(
             commit_de(git, principal, OP, &rama_remota())?
         } else {
             match clasificar_fallo_remoto(&s.stderr) {
-                FalloRemoto::RamaInexistente => None,
+                // El remoto aun no tiene la rama. Si existe la local (un repo
+                // que se monto sin remoto y al que luego se anadio `origin`),
+                // se parte de ella para publicar su historia, no de cero.
+                FalloRemoto::RamaInexistente => commit_de(git, principal, OP, &rama_local())?,
                 FalloRemoto::SinRed => {
                     return sin_red(c, op, anotacion, principal, &s.primera_linea_error());
                 }
@@ -298,8 +301,10 @@ fn intento(
                     anotaciones.push(o.anotacion());
                     destinos.push((p, Destino::Publicada));
                 }
-                Err(Fallo::Rechazada { .. }) => destinos.push((p, Destino::Rechazada)),
-                Err(Fallo::Error(d)) => return Err(error(d)),
+                // Tambien un error al re-ejecutarla: si abortara, bloquearia
+                // todas las escrituras siguientes mientras la ref siga ahi.
+                // Rechazada, su contenido queda en git y la linea lo dice.
+                Err(_) => destinos.push((p, Destino::Rechazada)),
             }
         }
     }
@@ -323,6 +328,9 @@ fn intento(
         vista.superponer(vec![Cambio::nuevo("ROADMAP.md", texto)]);
     }
 
+    if let Some(i) = choque_de_mayusculas(&vista, op) {
+        return Ok(Intento::Hecho(i));
+    }
     let mensaje = mensaje(op.nombre(), &anotaciones);
     let Some(sha) = construir_commit(
         git,
@@ -370,6 +378,26 @@ fn intento(
     }
 }
 
+/// Una ruta escrita que solo difiere en mayusculas de otra de la rama: en
+/// Windows o macOS las dos chocarian al sacarlas al worktree.
+fn choque_de_mayusculas(vista: &Vista<'_>, op: &dyn Operacion) -> Option<Informe> {
+    let rutas = vista.rutas();
+    vista.cambios().into_iter().find_map(|c| {
+        let minus = c.ruta.to_lowercase();
+        rutas
+            .iter()
+            .find(|r| **r != c.ruta && r.to_lowercase() == minus)
+            .map(|otra| {
+                informe(Resultado::Rechazada, op)
+                    .con("motivo", "ruta-duplicada")
+                    .diagnostico(format!(
+                        "{} solo difiere en mayusculas de {otra}: chocarian en Windows y macOS",
+                        c.ruta
+                    ))
+            })
+    })
+}
+
 fn denegada(op: &dyn Operacion, motivo: &str, detalle: &str) -> Informe {
     let primera = detalle
         .lines()
@@ -451,6 +479,9 @@ fn sin_red(
         }
         Err(Fallo::Error(d)) => return Err(error(d)),
     }
+    if let Some(i) = choque_de_mayusculas(&vista, op) {
+        return Ok(Intento::Hecho(i));
+    }
     let mensaje = mensaje(op.nombre(), &[anotacion.to_owned()]);
     let Some(sha) = construir_commit(
         git,
@@ -548,3 +579,28 @@ fn adelantar(git: &Git, principal: &Path, sha: &str) -> Result<Tareas, Informe>
     }
     Ok(Tareas::SinMontar)
 }
+
+#[cfg(test)]
+mod tests {
+    use super::espera;
+    use std::time::Duration;
+
+    #[test]
+    fn espera_exponencial_con_jitter_entre_la_mitad_y_el_total() {
+        let base = Duration::from_millis(100);
+        for n in 1..=4u32 {
+            let tope = base * 2u32.pow(n - 1);
+            for _ in 0..50 {
+                let e = espera(base, n);
+                assert!(
+                    e >= tope / 2 && e <= tope,
+                    "n={n}: {e:?} fuera de [{:?}, {tope:?}]",
+                    tope / 2
+                );
+            }
+        }
+        let distintas: std::collections::BTreeSet<Duration> =
+            (0..20).map(|_| espera(base, 1)).collect();
+        assert!(distintas.len() > 1, "sin jitter");
+    }
+}
diff --git a/crates/tc2/src/git/estado.rs b/crates/tc2/src/git/estado.rs
index 87cb44f..f6f9123 100644
--- a/crates/tc2/src/git/estado.rs
+++ b/crates/tc2/src/git/estado.rs
@@ -39,14 +39,37 @@ pub(crate) fn ruta_valida(ruta: &str) -> bool {
         && !ruta.contains(',')
         && !ruta.contains(':')
         && !ruta.chars().any(char::is_control)
-        && ruta
-            .split('/')
-            .all(|p| !p.is_empty() && p != "." && p != ".." && !p.eq_ignore_ascii_case(".git"))
+        && !ruta.contains(['<', '>', '"', '|', '?', '*'])
+        && ruta.split('/').all(|p| {
+            !p.is_empty()
+                && p != "."
+                && p != ".."
+                && !p.eq_ignore_ascii_case(".git")
+                && !p.ends_with(['.', ' '])
+                && !reservado_en_windows(p)
+        })
         && Path::new(ruta)
             .components()
             .all(|c| matches!(c, Component::Normal(_)))
 }
 
+/// Nombres que Windows reserva aunque lleven extension (`aux.md`): un clon en
+/// Windows no podria sacarlos al worktree.
+fn reservado_en_windows(parte: &str) -> bool {
+    let base = parte
+        .split('.')
+        .next()
+        .unwrap_or(parte)
+        .to_ascii_uppercase();
+    matches!(base.as_str(), "CON" | "PRN" | "AUX" | "NUL")
+        || ((base.starts_with("COM") || base.starts_with("LPT"))
+            && base.len() == 4
+            && base
+                .as_bytes()
+                .get(3)
+                .is_some_and(|b| (b'1'..=b'9').contains(b)))
+}
+
 /// Carpeta comun del repo (`.git` del principal), absoluta.
 pub(crate) fn dir_comun(git: &Git, dir: &Path, op: &str) -> Paso<PathBuf> {
     let s = exigir(
@@ -121,7 +144,14 @@ impl Temporal {
 
 impl Drop for Temporal {
     fn drop(&mut self) {
+        // Borrar es limpieza: si falla, no hay a quien avisar desde `Drop` y
+        // la carpeta no afecta a la correccion (la vacia el siguiente que
+        // use el mismo nombre). La carpeta padre se quita si queda vacia
+        // (`remove_dir` no borra una con contenido de otro proceso).
         let _ = std::fs::remove_dir_all(&self.0);
+        if let Some(padre) = self.0.parent() {
+            let _ = std::fs::remove_dir(padre);
+        }
     }
 }
 
@@ -252,7 +282,14 @@ mod tests {
 
     #[test]
     fn rutas_de_la_rama_de_estado() {
-        for ok in ["ROADMAP.md", "TASK-001/tarea.md", "a b/c.md"] {
+        for ok in [
+            "ROADMAP.md",
+            "TASK-001/tarea.md",
+            "a b/c.md",
+            "auxiliar.md",
+            "com10.md",
+            "console.md",
+        ] {
             assert!(ruta_valida(ok), "{ok}");
         }
         for malo in [
@@ -267,6 +304,18 @@ mod tests {
             "C:/x",
             "a,b",
             "a\nb",
+            "notas/aux.md",
+            "CON",
+            "x/com1.txt",
+            "LPT9",
+            "a?b.md",
+            "a*b",
+            "a<b",
+            "a|b",
+            "a\"b",
+            "fin.",
+            "fin ",
+            "carpeta./x",
         ] {
             assert!(!ruta_valida(malo), "{malo}");
         }
diff --git a/crates/tc2/src/git/remoto.rs b/crates/tc2/src/git/remoto.rs
index 54fcf2f..0698127 100644
--- a/crates/tc2/src/git/remoto.rs
+++ b/crates/tc2/src/git/remoto.rs
@@ -70,15 +70,19 @@ pub fn clasificar_fallo_remoto(stderr: &str) -> FalloRemoto {
         FalloRemoto::RamaInexistente
     } else if alguno(&[
         "authentication failed",
-        "permission denied",
+        "permission denied (publickey",
         "could not read username",
         "could not read password",
         "terminal prompts disabled",
         "returned error: 401",
         "returned error: 403",
-        "access denied",
-        "permission to ",
-    ]) {
+        "remote: permission to ",
+    ]) || (t.contains("permission denied")
+        && t.contains("could not read from remote repository"))
+    {
+        // `permission denied` suelto tambien lo da el sistema de ficheros
+        // local (no poder escribir en `.git`): solo cuenta como credenciales
+        // si git dice ademas que no pudo leer del remoto.
         FalloRemoto::Credenciales
     } else if alguno(&[
         "does not appear to be a git repository",
@@ -534,6 +538,12 @@ mod tests {
         }
     }
 
+    #[test]
+    fn permiso_local_no_es_de_credenciales() {
+        let local = "error: cannot open .git/FETCH_HEAD: Permission denied";
+        assert_eq!(clasificar_fallo_remoto(local), FalloRemoto::SinRed);
+    }
+
     #[test]
     fn contencion_frente_a_denegacion() {
         assert!(es_contencion(
diff --git a/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md b/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
index 09576f2..2b2887e 100644
--- a/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
+++ b/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
@@ -415,3 +415,33 @@ contra la plataforma: sigue en F9-4.
 **Contención en el remoto** (nuevo): dos pushes simultáneos al mismo remoto
 local pueden dar `[remote rejected] (cannot lock ref …)` en vez de
 non-fast-forward; se trata como contención (ver la adenda de F1-4).
+
+### Tras la revisión de la ronda 1 de TASK-018
+
+- **Remoto añadido después** (RN-06 → equipo): si el remoto aún no tiene la
+  rama de estado y existe la local, la transacción parte de la local y
+  publica su historia; antes empezaba de cero y la local divergía para
+  siempre. Test `remoto_anadido_despues_publica_la_historia_local`.
+- **`montar` toma el mismo cerrojo** que la transacción: cuatro `montar` a
+  la vez en el mismo clon dan un `ok` y tres `sin-cambios`, no `error`.
+- **Una pendiente que falla al re-ejecutarse** (error interno, no solo
+  rechazo) pasa a `rechazadas`: no bloquea las escrituras siguientes.
+- **Rutas**: además de relativas y sin `..`, portables entre sistemas (sin
+  `<>:"|?*`, sin nombres reservados de Windows, sin final `.` o espacio); una
+  ruta que solo difiere en mayúsculas de otra de la rama da `rechazada
+  motivo=ruta-duplicada`.
+- **`tareas=con-cambios` y `tareas=diverge` salen con `ok`** (aceptado): lo
+  escrito está publicado y correcto; lo que no se pudo es adelantar la
+  carpeta de la persona, y la línea lo dice. El contrato de F1-4 no muestra
+  diagnóstico en un éxito; avisar de forma visible es de `status` (F3-5b).
+  Sin remoto el mismo choque es `rechazada motivo=tareas-con-cambios`,
+  porque no hay otro sitio donde quede escrito.
+- **Límite aceptado: contenido UTF-8.** La vista lee los ficheros como texto
+  (el puerto convierte la salida de `git` con sustitución); un fichero que no
+  sea UTF-8 se corrompería al leerlo y reescribirlo. El formato de tarea
+  (F1-9) es UTF-8; lo que escriba la persona a mano en otra codificación no
+  está cubierto.
+- **Límite aceptado: contención por texto.** `[remote rejected]` con «cannot
+  lock ref» o «failed to update ref» se trata siempre como contención; si un
+  servidor lo diera de forma permanente, acabaría en `agotada` tras los
+  intentos (acotado), no en `denegada`. No visto en ningún servidor real.
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 4f80989fc93337d12966d31c9be87260371a8ddd..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-018/planificacion/brainstorm/peticion-plan-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-018/planificacion/plan-final.md                 |    0
 tareas/03-en-revision/TASK-018/revision/informe-revision-1-code-quality-reviewer.md         |   21 +
 tareas/03-en-revision/TASK-018/revision/informe-revision-1-rust-wasm-reviewer.md            |   21 +
 tareas/03-en-revision/TASK-018/revision/peticion-revision-1-code-quality-reviewer.md        |  184 ++++++++
 tareas/03-en-revision/TASK-018/revision/peticion-revision-1-rust-wasm-reviewer.md           | 2347 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-018/tarea.md                                    |   12 +-
 7 files changed, 2582 insertions(+), 3 deletions(-)
````

## Tests del diff (3 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff 4f80989fc93337d12966d31c9be87260371a8ddd..HEAD -- ":(literal)crates/tc2/tests/comun/ops.rs" ":(literal)crates/tc2/tests/estado_bordes.rs" ":(literal)crates/tc2/tests/git_casos_limite.rs"
````

````
crates/tc2/tests/comun/ops.rs        |  36 ++++++++++++++++++
 crates/tc2/tests/estado_bordes.rs    | 319 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/git_casos_limite.rs |  42 +++++++++++++++++++++
 3 files changed, 397 insertions(+)
````
