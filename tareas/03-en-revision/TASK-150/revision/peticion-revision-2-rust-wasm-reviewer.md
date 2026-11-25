# Peticion de revision — TASK-150 (ronda 2)

- Tarea: TASK-150 — Auditoria 4D: utilidades comunes de E-S y tiempo (4 IMPORTANTE)
- Rama revisada: fix/task-150-auditoria-4d-utilidades-comunes-de-e-s-y
- Rama base: develop
- Commit revisado (HEAD): 81155ba1361f4d574148d86fbdebc38567c8d55d
- Fecha: 2026-11-25
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-150 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 8347d882515db613e63d96e66e2e2f147fad004f (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (3 hallazgo(s)))

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

- tareas/03-en-revision/TASK-150/tarea.md
- tareas/03-en-revision/TASK-150/planificacion/
- crates/tc2/src/escritura.rs
- crates/tc2/src/hallazgos/orden.rs
- crates/tc2/src/lectura.rs
- crates/tc2/src/seleccion/orden.rs
- crates/tc2/tests/hallazgos.rs

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 2 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/hallazgos/orden.rs:223 (`leer_entrada`) y crates/tc2/src/seleccion/orden.rs (`--rutas`, `--tarea`, ficheros de `paso_seleccion`) | informe-revision-1-rust-wasm-reviewer.md |

## Commits a revisar (git log 8347d882515db613e63d96e66e2e2f147fad004f..HEAD)

````
81155ba chore(TASK-150): correcciones de la revision 1 anotadas
9ce49a2 fix(TASK-150): MEN-2 doc de escritura::atomica completa
3c1d524 fix(TASK-150): IMP-1 y MEN-1 lectura lossy para entradas que no son informes
52c5097 chore(TASK-150): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 8347d882515db613e63d96e66e2e2f147fad004f..HEAD)

````diff
diff --git a/crates/tc2/src/escritura.rs b/crates/tc2/src/escritura.rs
index 0301aa5..51f067b 100644
--- a/crates/tc2/src/escritura.rs
+++ b/crates/tc2/src/escritura.rs
@@ -23,12 +23,14 @@ const ESPERAS_RENAME: [Duration; 5] = [
 ];
 
 /// Escribe `ruta` de una vez. Si `ruta` es un enlace simbolico se escribe su
-/// destino (como `cat >` de `install.sh`); si el destino ya existe, el nuevo
+/// destino (como `cat >` de `install.sh`); un enlace roto da el error de
+/// `canonicalize` y no crea nada. Si el destino ya existe, el nuevo
 /// conserva sus permisos. Crea la carpeta si falta, escribe el temporal
 /// `<nombre>.tc2-tmp-<pid>` al lado y lo renombra encima. Si el `rename`
-/// falla, se repite tras cada espera de [`ESPERAS_RENAME`]. Si falla la
-/// escritura, los permisos o el ultimo `rename`, se borra el temporal y se
-/// devuelve el error.
+/// falla, por el motivo que sea (no solo un lector abierto en Windows:
+/// tambien un destino que es una carpeta), se repite tras cada espera de
+/// [`ESPERAS_RENAME`]. Si falla la escritura, los permisos o el ultimo
+/// `rename`, se borra el temporal y se devuelve el error.
 pub(crate) fn atomica(ruta: &Path, contenido: &[u8]) -> io::Result<()> {
     atomica_con(ruta, contenido, &ESPERAS_RENAME)
 }
diff --git a/crates/tc2/src/hallazgos/orden.rs b/crates/tc2/src/hallazgos/orden.rs
index d1d28ca..5a22b3f 100644
--- a/crates/tc2/src/hallazgos/orden.rs
+++ b/crates/tc2/src/hallazgos/orden.rs
@@ -220,8 +220,10 @@ fn fallo_io(p: &Path, error: String) -> Informe {
     )
 }
 
+/// Una entrada que no es el informe (respuesta, diff, contexto, JSON,
+/// rutas): con tope y `U+FFFD`, como antes de TASK-150 (IMP-1).
 fn leer_entrada(p: &Path, tope: u64) -> Result<String, Informe> {
-    crate::lectura::texto_con_tope(p, tope)
+    crate::lectura::texto_con_tope_lossy(p, tope)
         .map_err(|e| fallo_io(p, e.to_string()))?
         .ok_or_else(|| {
             Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(
@@ -327,7 +329,7 @@ fn validar(a: &[String]) -> Result<Informe, Informe> {
 
 /// Una salida de `validar` del contrato vigente, o `None`.
 fn leer_validados(p: &Path) -> Option<super::Validados> {
-    let t = crate::lectura::texto_con_tope(p, TOPE_VALIDADOS).ok()??;
+    let t = crate::lectura::texto_con_tope_lossy(p, TOPE_VALIDADOS).ok()??;
     serde_json::from_str::<super::Validados>(&t)
         .ok()
         .filter(|v| v.version == super::VERSION)
diff --git a/crates/tc2/src/lectura.rs b/crates/tc2/src/lectura.rs
index 5d73569..20079cd 100644
--- a/crates/tc2/src/lectura.rs
+++ b/crates/tc2/src/lectura.rs
@@ -15,7 +15,10 @@
 //! Y el «leer con tope» de todos (A8-17): [`acotado`] lee como mucho un
 //! byte mas del tope y rechaza si llega a leerlo; [`con_tope`] y
 //! [`texto_con_tope`] lo aplican a un fichero. El texto es UTF-8 estricto:
-//! unos bytes que no lo son dan `InvalidData`, nunca `U+FFFD`.
+//! unos bytes que no lo son dan `InvalidData`, nunca `U+FFFD` (decision 3
+//! de TASK-150, para los informes del revisor). Las entradas que no son
+//! informes (un diff, la respuesta de la revisora, una lista de rutas) usan
+//! [`texto_con_tope_lossy`]: el mismo tope y `U+FFFD`, como antes.
 
 use std::io::{self, Read as _};
 use std::path::Path;
@@ -67,6 +70,12 @@ pub(crate) fn texto_con_tope(ruta: &Path, tope: u64) -> io::Result<Option<String
         .transpose()
 }
 
+/// El texto de `ruta` con [`con_tope`]: `None` si pasa de `tope`, y los
+/// bytes que no son UTF-8 como `U+FFFD` (IMP-1 de TASK-150).
+pub(crate) fn texto_con_tope_lossy(ruta: &Path, tope: u64) -> io::Result<Option<String>> {
+    Ok(con_tope(ruta, tope)?.map(|b| String::from_utf8_lossy(&b).into_owned()))
+}
+
 /// Tope de un fichero externo (D4 de TASK-029).
 pub const MAX_BYTES: usize = 1 << 20;
 
@@ -151,7 +160,8 @@ mod tests {
     }
 
     /// Decision 3 de TASK-150: `texto_con_tope` rechaza lo que no es UTF-8
-    /// con `InvalidData` (sin `U+FFFD`); `con_tope` da los bytes tal cual.
+    /// con `InvalidData` (sin `U+FFFD`); `con_tope` da los bytes tal cual, y
+    /// `texto_con_tope_lossy` (IMP-1), el texto con `U+FFFD` y el mismo tope.
     #[test]
     fn texto_con_tope_es_utf8_estricto() -> io::Result<()> {
         let dir = std::env::temp_dir()
@@ -162,9 +172,13 @@ mod tests {
         std::fs::write(&malo, [b'a', 0xff])?;
         let texto = texto_con_tope(&malo, 16).map_err(|e| e.kind());
         let bytes = con_tope(&malo, 16)?;
+        let lossy = texto_con_tope_lossy(&malo, 16)?;
+        let grande = texto_con_tope_lossy(&malo, 1)?;
         let _ = std::fs::remove_dir_all(&dir);
         assert_eq!(texto, Err(io::ErrorKind::InvalidData));
         assert_eq!(bytes, Some(vec![b'a', 0xff]));
+        assert_eq!(lossy, Some("a\u{fffd}".to_owned()));
+        assert_eq!(grande, None);
         Ok(())
     }
 }
diff --git a/crates/tc2/src/seleccion/orden.rs b/crates/tc2/src/seleccion/orden.rs
index 617800b..56841be 100644
--- a/crates/tc2/src/seleccion/orden.rs
+++ b/crates/tc2/src/seleccion/orden.rs
@@ -422,7 +422,7 @@ fn paso(a: &[String], e: &Entorno<'_>) -> Result<Informe, Informe> {
                 .ok_or_else(|| fallo_lectura("-", format!("mas de {TOPE_RUTAS} bytes")))?;
             String::from_utf8(b).map_err(|x| fallo_lectura("-", x.to_string()))?
         }
-        Some(f) => crate::lectura::texto_con_tope(Path::new(f), TOPE_RUTAS)
+        Some(f) => crate::lectura::texto_con_tope_lossy(Path::new(f), TOPE_RUTAS)
             .map_err(|x| fallo_lectura(f, x.to_string()))?
             .ok_or_else(|| fallo_lectura(f, format!("mas de {TOPE_RUTAS} bytes")))?,
         None => return Err(uso("sin-rutas")),
@@ -569,7 +569,7 @@ fn paso_seleccion(a: &[String], e: &Entorno<'_>) -> Result<Informe, Informe> {
     };
     let (t, c) = match (&fichero, &id) {
         (Some(f), _) => {
-            let texto = crate::lectura::texto_con_tope(Path::new(f), TOPE_RUTAS)
+            let texto = crate::lectura::texto_con_tope_lossy(Path::new(f), TOPE_RUTAS)
                 .map_err(|x| error(f, x.to_string()))?
                 .ok_or_else(|| error(f, format!("mas de {TOPE_RUTAS} bytes")))?;
             tarea_de_fichero(&texto).map_err(|x| error(f, x))?
@@ -585,7 +585,7 @@ fn paso_seleccion(a: &[String], e: &Entorno<'_>) -> Result<Informe, Informe> {
     };
     let rutas = match &rutas_f {
         Some(f) => separar_rutas(
-            &crate::lectura::texto_con_tope(Path::new(f), TOPE_RUTAS)
+            &crate::lectura::texto_con_tope_lossy(Path::new(f), TOPE_RUTAS)
                 .map_err(|x| error(f, x.to_string()))?
                 .ok_or_else(|| error(f, format!("mas de {TOPE_RUTAS} bytes")))?,
             false,
````

## Excluido del diff (5 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 8347d882515db613e63d96e66e2e2f147fad004f..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-150/planificacion/brainstorm/peticion-plan-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-150/planificacion/plan-final.md                 |    0
 tareas/03-en-revision/TASK-150/revision/informe-revision-1-rust-wasm-reviewer.md            |   75 +++++
 tareas/03-en-revision/TASK-150/revision/peticion-revision-1-rust-wasm-reviewer.md           | 1602 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-150/tarea.md                                    |   12 +-
 5 files changed, 1686 insertions(+), 3 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff 8347d882515db613e63d96e66e2e2f147fad004f..HEAD -- ":(literal)crates/tc2/tests/hallazgos.rs"
````

````
crates/tc2/tests/hallazgos.rs | 36 ++++++++++++++++++++++++++++++++++++
 1 file changed, 36 insertions(+)
````
## Candidatos 4D

Pistas generadas desde el diff, no hallazgos: comprueba cada una (Regla 4D de `revision.md`).

Sin candidatos: ninguna definicion nueva se parece a otra y ningun test nuevo promete algo que haya que mutar.

