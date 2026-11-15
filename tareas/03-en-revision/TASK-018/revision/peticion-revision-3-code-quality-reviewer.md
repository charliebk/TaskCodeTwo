# Peticion de revision — TASK-018 (ronda 3)

- Tarea: TASK-018 — F2-3 Rama de estado
- Rama revisada: feature/task-018-f2-3-rama-de-estado
- Rama base: develop
- Commit revisado (HEAD): 36d96aab6cbf310f59acaea255107de76aa9218e
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-018 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 09e487ed1a5f3f43221c6cb4a968ea0549237c73 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (2 hallazgo(s)))

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
informe de esta ronda (informe-revision-3-code-quality-reviewer.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-018/tarea.md
- tareas/03-en-revision/TASK-018/planificacion/
- crates/tc2/src/estado/montaje.rs
- crates/tc2/src/estado/transaccion.rs
- crates/tc2/src/git/remoto.rs
- crates/tc2/src/git/tag.rs
- crates/tc2/tests/estado_bordes.rs
- docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md

## Hallazgos de la ronda 2 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMPORTANTE-2 | IMPORTANTE | corregido | crates/tc2/src/git/remoto.rs (`clasificar_fallo_remoto`) | informe-revision-2-code-quality-reviewer.md |

## Commits a revisar (git log 09e487ed1a5f3f43221c6cb4a968ea0549237c73..HEAD)

````
36d96aa chore(TASK-018): autorrevision y ronda extra 3
9e99510 fix(TASK-018): correcciones de la revision ronda 2
b35ce92 chore(TASK-018): peticion de revision ronda 2
````

## Diff desde la ronda anterior (git diff 09e487ed1a5f3f43221c6cb4a968ea0549237c73..HEAD)

````diff
diff --git a/crates/tc2/src/estado/montaje.rs b/crates/tc2/src/estado/montaje.rs
index 0bbb944..03c77c9 100644
--- a/crates/tc2/src/estado/montaje.rs
+++ b/crates/tc2/src/estado/montaje.rs
@@ -157,6 +157,12 @@ fn paso(git: &Git, dir: &Path, politica: Politica, puede_crear: bool) -> Result<
                                 s.primera_linea_error()
                             )));
                     }
+                    FalloRemoto::Local => {
+                        return Err(crate::git::remoto::error_local(
+                            OP,
+                            &s.primera_linea_error(),
+                        ));
+                    }
                     otro => {
                         return Ok(Informe::nuevo(OP, Resultado::Denegada)
                             .con("motivo", otro.motivo())
diff --git a/crates/tc2/src/estado/transaccion.rs b/crates/tc2/src/estado/transaccion.rs
index c6004c6..31f7672 100644
--- a/crates/tc2/src/estado/transaccion.rs
+++ b/crates/tc2/src/estado/transaccion.rs
@@ -278,6 +278,12 @@ fn intento(
                 FalloRemoto::SinRed => {
                     return sin_red(c, op, anotacion, principal, &s.primera_linea_error());
                 }
+                FalloRemoto::Local => {
+                    return Err(crate::git::remoto::error_local(
+                        OP,
+                        &s.primera_linea_error(),
+                    ));
+                }
                 otro => return Ok(Intento::Hecho(denegada(op, otro.motivo(), &s.stderr))),
             }
         }
@@ -296,6 +302,11 @@ fn intento(
                 continue;
             };
             match o.aplicar(&vista) {
+                // Una pendiente que choca por mayusculas con lo publicado ya
+                // no procede: si se quedara, bloquearia cada operacion.
+                Ok(cambios) if choque(&vista.rutas(), &cambios).is_some() => {
+                    destinos.push((p, Destino::Rechazada));
+                }
                 Ok(cambios) => {
                     vista.superponer(cambios);
                     anotaciones.push(o.anotacion());
@@ -309,7 +320,12 @@ fn intento(
         }
     }
     match op.aplicar(&vista) {
-        Ok(cambios) => vista.superponer(cambios),
+        Ok(cambios) => {
+            if let Some(i) = duplicada(&vista.rutas(), &cambios, op) {
+                return Ok(Intento::Hecho(i));
+            }
+            vista.superponer(cambios);
+        }
         Err(Fallo::Rechazada {
             motivo,
             diagnostico,
@@ -328,9 +344,6 @@ fn intento(
         vista.superponer(vec![Cambio::nuevo("ROADMAP.md", texto)]);
     }
 
-    if let Some(i) = choque_de_mayusculas(&vista, op) {
-        return Ok(Intento::Hecho(i));
-    }
     let mensaje = mensaje(op.nombre(), &anotaciones);
     let Some(sha) = construir_commit(
         git,
@@ -374,27 +387,33 @@ fn intento(
         }
         Resultado::Rechazada => Ok(Intento::Reintentar),
         Resultado::Denegada => Ok(Intento::Hecho(denegada(op, p.motivo, &p.ultimo))),
+        // Fallo del propio clon al publicar: ni sin red ni pendiente.
+        Resultado::Error => Err(crate::git::remoto::error_local(OP, &p.ultimo)),
         _ => sin_red(c, op, anotacion, principal, &p.ultimo),
     }
 }
 
-/// Una ruta escrita que solo difiere en mayusculas de otra de la rama: en
-/// Windows o macOS las dos chocarian al sacarlas al worktree.
-fn choque_de_mayusculas(vista: &Vista<'_>, op: &dyn Operacion) -> Option<Informe> {
-    let rutas = vista.rutas();
-    vista.cambios().into_iter().find_map(|c| {
+/// Primera ruta de `cambios` que solo difiere en mayusculas de otra ya
+/// presente en `rutas` (en Windows o macOS las dos chocarian al sacarlas al
+/// worktree): (la nueva, la que estaba).
+fn choque(rutas: &[String], cambios: &[Cambio]) -> Option<(String, String)> {
+    cambios.iter().find_map(|c| {
         let minus = c.ruta.to_lowercase();
         rutas
             .iter()
             .find(|r| **r != c.ruta && r.to_lowercase() == minus)
-            .map(|otra| {
-                informe(Resultado::Rechazada, op)
-                    .con("motivo", "ruta-duplicada")
-                    .diagnostico(format!(
-                        "{} solo difiere en mayusculas de {otra}: chocarian en Windows y macOS",
-                        c.ruta
-                    ))
-            })
+            .map(|otra| (c.ruta.clone(), otra.clone()))
+    })
+}
+
+/// `rechazada motivo=ruta-duplicada` si los cambios de la operacion chocan.
+fn duplicada(rutas: &[String], cambios: &[Cambio], op: &dyn Operacion) -> Option<Informe> {
+    choque(rutas, cambios).map(|(nueva, otra)| {
+        informe(Resultado::Rechazada, op)
+            .con("motivo", "ruta-duplicada")
+            .diagnostico(format!(
+                "{nueva} solo difiere en mayusculas de {otra}: chocarian en Windows y macOS"
+            ))
     })
 }
 
@@ -466,7 +485,12 @@ fn sin_red(
     };
     let mut vista = Vista::nueva(git, principal, conocida.clone())?;
     match op.aplicar(&vista) {
-        Ok(cambios) => vista.superponer(cambios),
+        Ok(cambios) => {
+            if let Some(i) = duplicada(&vista.rutas(), &cambios, op) {
+                return Ok(Intento::Hecho(i));
+            }
+            vista.superponer(cambios);
+        }
         Err(Fallo::Rechazada {
             motivo,
             diagnostico,
@@ -479,9 +503,6 @@ fn sin_red(
         }
         Err(Fallo::Error(d)) => return Err(error(d)),
     }
-    if let Some(i) = choque_de_mayusculas(&vista, op) {
-        return Ok(Intento::Hecho(i));
-    }
     let mensaje = mensaje(op.nombre(), &[anotacion.to_owned()]);
     let Some(sha) = construir_commit(
         git,
diff --git a/crates/tc2/src/git/remoto.rs b/crates/tc2/src/git/remoto.rs
index 0698127..3d98c09 100644
--- a/crates/tc2/src/git/remoto.rs
+++ b/crates/tc2/src/git/remoto.rs
@@ -46,6 +46,9 @@ pub enum FalloRemoto {
     RemotoNoExiste,
     /// El remoto existe pero no tiene la rama pedida.
     RamaInexistente,
+    /// Fallo del propio clon, no del remoto (p. ej. sin permiso para
+    /// escribir en `.git`): `error`, ni red ni credenciales.
+    Local,
 }
 
 impl FalloRemoto {
@@ -56,6 +59,7 @@ impl FalloRemoto {
             Self::Credenciales => "credenciales",
             Self::RemotoNoExiste => "remoto-no-existe",
             Self::RamaInexistente => "rama-no-existe",
+            Self::Local => "error-local",
         }
     }
 }
@@ -77,6 +81,7 @@ pub fn clasificar_fallo_remoto(stderr: &str) -> FalloRemoto {
         "returned error: 401",
         "returned error: 403",
         "remote: permission to ",
+        "access denied",
     ]) || (t.contains("permission denied")
         && t.contains("could not read from remote repository"))
     {
@@ -91,6 +96,10 @@ pub fn clasificar_fallo_remoto(stderr: &str) -> FalloRemoto {
         "returned error: 404",
     ]) {
         FalloRemoto::RemotoNoExiste
+    } else if t.contains("permission denied") {
+        // Sin las marcas del remoto de arriba, es el sistema de ficheros del
+        // clon (`cannot open .git/FETCH_HEAD: Permission denied`).
+        FalloRemoto::Local
     } else {
         FalloRemoto::SinRed
     }
@@ -109,6 +118,26 @@ pub(crate) fn denegada(op: &str, fallo: FalloRemoto, detalle: &str) -> Informe {
         .diagnostico(format!("{op}: {detalle}; {que_hacer}"))
 }
 
+/// Fallo del propio clon al hablar con el remoto: `error` (1).
+pub(crate) fn error_local(op: &str, detalle: &str) -> Informe {
+    Informe::nuevo(op, Resultado::Error)
+        .con("motivo", FalloRemoto::Local.motivo())
+        .diagnostico(format!(
+            "{op}: {detalle}; fallo local de git: revisa los permisos de la carpeta del repo"
+        ))
+}
+
+/// Que hacer tras un push que no salio, segun el resultado.
+pub(crate) fn que_hacer_al_publicar(r: Resultado) -> &'static str {
+    match r {
+        Resultado::Denegada => {
+            "el remoto deniega el push: revisa sus reglas, tus permisos o avisa a la persona"
+        }
+        Resultado::Error => "fallo local de git: revisa los permisos de la carpeta del repo",
+        _ => "reintenta con red",
+    }
+}
+
 /// Trae el remoto. Nada nuevo → `sin-cambios`; sin red → `sinred`; remoto
 /// inexistente o sin permiso → `denegada`.
 pub fn fetch(git: &Git, dir: &Path) -> Informe {
@@ -121,6 +150,7 @@ fn traer(git: &Git, dir: &Path, op: &str) -> Paso<Informe> {
     if !s.exito() {
         return Err(match clasificar_fallo_remoto(&s.stderr) {
             FalloRemoto::SinRed | FalloRemoto::RamaInexistente => sinred(op),
+            FalloRemoto::Local => error_local(op, &s.primera_linea_error()),
             otro => denegada(op, otro, &s.primera_linea_error()),
         });
     }
@@ -142,6 +172,8 @@ enum Intento {
     /// (`[remote rejected]` por un hook o una regla, credenciales, remoto
     /// inexistente), el motivo: eso no se arregla reintentando.
     Fallo(String, Option<&'static str>),
+    /// Fallo del propio clon (permisos en `.git`): no es del remoto.
+    Local(String),
 }
 
 fn intento_push(git: &Git, dir: &Path, op: &str, refspec: &str) -> Paso<Intento> {
@@ -177,6 +209,7 @@ fn intento_push(git: &Git, dir: &Path, op: &str, refspec: &str) -> Paso<Intento>
             };
             match clasificar_fallo_remoto(&s.stderr) {
                 FalloRemoto::SinRed | FalloRemoto::RamaInexistente => Intento::Fallo(detalle, None),
+                FalloRemoto::Local => Intento::Local(detalle),
                 otro => Intento::Fallo(detalle, Some(otro.motivo())),
             }
         }
@@ -203,14 +236,14 @@ fn es_contencion(resumen: &str) -> bool {
 
 /// Como termino [`publicar`].
 pub(crate) struct Publicacion {
-    /// `Ok`, `SinCambios`, `Rechazada` (non-fast-forward), `Denegada` o
-    /// `Sinred`.
+    /// `Ok`, `SinCambios`, `Rechazada` (non-fast-forward), `Denegada`,
+    /// `Error` (fallo local) o `Sinred`.
     pub resultado: Resultado,
     /// Intentos hechos.
     pub intentos: usize,
     /// Ultimo fallo, para el diagnostico.
     pub ultimo: String,
-    /// Motivo de una `Denegada`.
+    /// Motivo de una `Denegada` o un `Error` local.
     pub motivo: &'static str,
 }
 
@@ -242,6 +275,14 @@ pub(crate) fn publicar(
             Intento::Fallo(m, Some(motivo)) => {
                 return Ok(fin(Resultado::Denegada, intento + 1, m, motivo));
             }
+            Intento::Local(m) => {
+                return Ok(fin(
+                    Resultado::Error,
+                    intento + 1,
+                    m,
+                    FalloRemoto::Local.motivo(),
+                ));
+            }
             Intento::Fallo(m, None) => ultimo = m,
         }
         if let Some(e) = esperas.get(intento) {
@@ -272,15 +313,13 @@ fn push_paso(git: &Git, dir: &Path, op: &str, esperas: &[Duration]) -> Paso<Info
         Resultado::Rechazada => i
             .con("motivo", "no-fast-forward")
             .diagnostico("lanza actualizar-rama"),
-        Resultado::Denegada => i
+        Resultado::Denegada | Resultado::Error => i
             .con("motivo", p.motivo)
             .con("intentos", intentos.to_string())
-            .diagnostico(format!(
-                "push: {ultimo}; el remoto deniega el push: revisa sus reglas, tus permisos o avisa a la persona"
-            )),
+            .diagnostico(format!("push: {ultimo}; {}", que_hacer_al_publicar(r))),
         _ => i
             .con("intentos", intentos.to_string())
-            .diagnostico(format!("push: {ultimo}; reintenta con red")),
+            .diagnostico(format!("push: {ultimo}; {}", que_hacer_al_publicar(r))),
     })
 }
 
@@ -414,13 +453,9 @@ fn actualizar(git: &Git, dir: &Path, esperas: &[Duration]) -> Paso<Informe> {
                     ));
             }
             otro => {
-                let que_hacer = if otro == Resultado::Denegada {
-                    "el remoto deniega el push: revisa sus reglas, tus permisos o avisa a la persona"
-                } else {
-                    "reintenta con red"
-                };
+                let que_hacer = que_hacer_al_publicar(otro);
                 let mut i = Informe::nuevo(OP, otro).con("rama", rama.as_str());
-                if otro == Resultado::Denegada {
+                if matches!(otro, Resultado::Denegada | Resultado::Error) {
                     i = i.con("motivo", p.motivo);
                 }
                 return Ok(i.con("intentos", intentos.to_string()).diagnostico(format!(
@@ -541,7 +576,19 @@ mod tests {
     #[test]
     fn permiso_local_no_es_de_credenciales() {
         let local = "error: cannot open .git/FETCH_HEAD: Permission denied";
-        assert_eq!(clasificar_fallo_remoto(local), FalloRemoto::SinRed);
+        assert_eq!(clasificar_fallo_remoto(local), FalloRemoto::Local);
+        // Una denegacion del remoto sigue siendo de credenciales.
+        for remoto in [
+            "remote: access denied",
+            "fatal: unable to access 'https://host/a.git/': access denied",
+            "remote: Access denied.\nfatal: Could not read from remote repository.",
+        ] {
+            assert_eq!(
+                clasificar_fallo_remoto(remoto),
+                FalloRemoto::Credenciales,
+                "{remoto}"
+            );
+        }
     }
 
     #[test]
diff --git a/crates/tc2/src/git/tag.rs b/crates/tc2/src/git/tag.rs
index f5d03bd..12f852d 100644
--- a/crates/tc2/src/git/tag.rs
+++ b/crates/tc2/src/git/tag.rs
@@ -77,13 +77,9 @@ fn tag_paso(
         let refspec = format!("refs/tags/{tag}:refs/tags/{tag}");
         let p = publicar(git, dir, OP, &refspec, esperas)?;
         if !p.resultado.es_exito() {
-            let que_hacer = if p.resultado == Resultado::Denegada {
-                "el remoto lo deniega: revisa sus reglas o tus permisos"
-            } else {
-                "relanza con red"
-            };
+            let que_hacer = super::remoto::que_hacer_al_publicar(p.resultado);
             let mut i = Informe::nuevo(OP, p.resultado).con("tag", tag);
-            if p.resultado == Resultado::Denegada {
+            if matches!(p.resultado, Resultado::Denegada | Resultado::Error) {
                 i = i.con("motivo", p.motivo);
             }
             return Ok(i
diff --git a/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md b/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
index 2b2887e..f9e20b7 100644
--- a/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
+++ b/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
@@ -445,3 +445,15 @@ non-fast-forward; se trata como contención (ver la adenda de F1-4).
   lock ref» o «failed to update ref» se trata siempre como contención; si un
   servidor lo diera de forma permanente, acabaría en `agotada` tras los
   intentos (acotado), no en `denegada`. No visto en ningún servidor real.
+
+### Tras la revisión de la ronda 2 de TASK-018
+
+- **Fallo local frente a remoto**: un `permission denied` sin las marcas del
+  remoto (`publickey`, «could not read from remote repository») es del
+  sistema de ficheros del clon y da `error` (1) con `motivo=error-local`; no
+  `sinred` (que mandaría reintentar) ni `credenciales`. `access denied`
+  sigue siendo de credenciales.
+- **Choque por mayúsculas por etapas**: una pendiente que al re-ejecutarse
+  chocaría con lo ya publicado va a `rechazadas`; solo la operación actual
+  puede devolver `rechazada motivo=ruta-duplicada`. Antes, la pendiente
+  bloqueaba todas las escrituras del clon.
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 09e487ed1a5f3f43221c6cb4a968ea0549237c73..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/03-en-revision/TASK-018/revision/informe-revision-1-code-quality-reviewer.md  |  76 +++++++++++++++--
 tareas/03-en-revision/TASK-018/revision/informe-revision-1-rust-wasm-reviewer.md     |  45 ++++++++--
 tareas/03-en-revision/TASK-018/revision/informe-revision-2-code-quality-reviewer.md  |  21 +++++
 tareas/03-en-revision/TASK-018/revision/informe-revision-2-rust-wasm-reviewer.md     |  21 +++++
 tareas/03-en-revision/TASK-018/revision/peticion-revision-2-code-quality-reviewer.md | 491 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-018/revision/peticion-revision-2-rust-wasm-reviewer.md    | 487 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-018/tarea.md                                              |  11 ++-
 7 files changed, 1135 insertions(+), 17 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff 09e487ed1a5f3f43221c6cb4a968ea0549237c73..HEAD -- ":(literal)crates/tc2/tests/estado_bordes.rs"
````

````
crates/tc2/tests/estado_bordes.rs | 76 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 76 insertions(+)
````
