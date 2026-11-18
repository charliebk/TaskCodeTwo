# Peticion de revision — TASK-127 (ronda 3)

- Tarea: TASK-127 — Escrituras de estado con muchos ficheros fallan en Windows (git hash-object)
- Rama revisada: fix/task-127-escrituras-de-estado-con-muchos-ficheros
- Rama base: develop
- Commit revisado (HEAD): fa39e430c433bf9411d29cb9b108ee2daf7b55df
- Fecha: 2026-11-18
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-127 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 0d318d505400cf903b38867e6846e8355967785c (el commit revisado en la ronda anterior)
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
informe de esta ronda (informe-revision-3-rust-wasm-reviewer.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-127/tarea.md
- tareas/03-en-revision/TASK-127/planificacion/
- crates/tc2/src/git/estado.rs

## Hallazgos de la ronda 2 pendientes de verificar

No tienes hallazgos pendientes propios: recibes esta ronda porque la correccion
toca ficheros de tu dominio. Revisa en el diff de esta ronda lo que la correccion
cambia en tu terreno y si abre un fallo.

## Commits a revisar (git log 0d318d505400cf903b38867e6846e8355967785c..HEAD)

````
fa39e43 chore(TASK-127): ronda extra y autorrevision
eb0eb3d fix(TASK-127): hashear protege el control de un sha por fichero
5ff21d3 chore(TASK-127): peticion de revision ronda 2
````

## Diff desde la ronda anterior (git diff 0d318d505400cf903b38867e6846e8355967785c..HEAD)

````diff
diff --git a/crates/tc2/src/git/estado.rs b/crates/tc2/src/git/estado.rs
index 6d069da..54dafe1 100644
--- a/crates/tc2/src/git/estado.rs
+++ b/crates/tc2/src/git/estado.rs
@@ -297,12 +297,21 @@ fn con_entrada(
     }
 }
 
-/// Un sha por linea de la salida de `hash-object --stdin-paths`, solo si hay
-/// exactamente uno por fichero: si faltara alguno, emparejar por orden daria
-/// a cada ruta el contenido de otra.
-fn shas_completos(salida: &str, n: usize) -> Option<Vec<&str>> {
-    let shas: Vec<&str> = salida.lines().map(str::trim).collect();
-    (shas.len() == n).then_some(shas)
+/// Escribe en el almacen de objetos los ficheros de `rutas` (una ruta por
+/// linea, `n` en total) con un solo `hash-object --stdin-paths` y devuelve
+/// su sha en el mismo orden. Error si no hay exactamente uno por fichero:
+/// emparejar por orden daria a cada ruta el contenido de otra.
+fn hashear(git: &Git, dir: &Path, op: &str, rutas: &str, n: usize) -> Paso<Vec<String>> {
+    let salida = con_entrada(git, dir, op, Subcomando::HashObject, None, rutas.as_bytes())?;
+    let shas: Vec<String> = String::from_utf8_lossy(&salida)
+        .lines()
+        .map(|l| l.trim().to_owned())
+        .collect();
+    if shas.len() == n {
+        Ok(shas)
+    } else {
+        Err(Informe::nuevo(op, Resultado::Error).diagnostico(&txt::HASH_OBJECT_INCOMPLETO))
+    }
 }
 
 /// Construye un commit con el arbol de `base` mas `cambios` y `base` como
@@ -343,11 +352,7 @@ pub(crate) fn construir_commit(
             rutas.push_str(&texto(op, &f)?);
             rutas.push('\n');
         }
-        let shas = con_entrada(git, dir, op, Subcomando::HashObject, None, rutas.as_bytes())?;
-        let shas = String::from_utf8_lossy(&shas);
-        let shas = shas_completos(&shas, cambios.len()).ok_or_else(|| {
-            Informe::nuevo(op, Resultado::Error).diagnostico(&txt::HASH_OBJECT_INCOMPLETO)
-        })?;
+        let shas = hashear(git, dir, op, &rutas, cambios.len())?;
         let mut entradas = String::new();
         for (c, sha) in cambios.iter().zip(&shas) {
             entradas.push_str(&format!("100644 {sha}\t{}\0", c.ruta));
@@ -486,6 +491,7 @@ mod tests {
 #[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
 mod tests_stdin {
     use super::*;
+    use crate::i18n::Idioma;
 
     /// Repo vacio de verdad en una carpeta temporal propia.
     fn repo(nombre: &str) -> PathBuf {
@@ -502,12 +508,33 @@ mod tests_stdin {
         d
     }
 
+    /// TASK-127: `hashear` da un sha por fichero y, si `hash-object` no
+    /// devuelve exactamente uno por fichero, `error` con
+    /// `HASH_OBJECT_INCOMPLETO` en vez de emparejar mal rutas y contenidos.
     #[test]
-    fn shas_completos_exige_uno_por_fichero() {
-        assert_eq!(shas_completos("a\nb\n", 2), Some(vec!["a", "b"]));
-        assert_eq!(shas_completos("a\n", 2), None);
-        assert_eq!(shas_completos("a\nb\nc\n", 2), None);
-        assert_eq!(shas_completos("", 0), Some(vec![]));
+    fn hashear_exige_un_sha_por_fichero() {
+        let d = repo("hashear");
+        let git = Git::nuevo();
+        let f = d.join("blob");
+        std::fs::write(&f, "hola\n").unwrap();
+        let rutas = format!("{}\n", f.display());
+
+        let shas = hashear(&git, &d, "prueba", &rutas, 1).expect("un fichero, un sha");
+        assert_eq!(shas.len(), 1);
+        assert_eq!(shas[0].len(), 40, "{shas:?}");
+
+        for n in [0, 2] {
+            let i = hashear(&git, &d, "prueba", &rutas, n).expect_err("cuenta distinta");
+            assert_eq!(i.resultado, Resultado::Error);
+            let incompleto = Informe::nuevo("prueba", Resultado::Error)
+                .diagnostico(&txt::HASH_OBJECT_INCOMPLETO);
+            assert_eq!(
+                i.diagnostico_en(Idioma::Es),
+                incompleto.diagnostico_en(Idioma::Es),
+                "n={n}"
+            );
+        }
+        let _ = std::fs::remove_dir_all(&d);
     }
 
     /// TASK-127: si git falla con la entrada por stdin, `con_entrada` da
@@ -553,7 +580,7 @@ mod tests_stdin {
             entrada.as_bytes(),
         );
         let shas = String::from_utf8(r.expect("hash-object de un fichero que existe")).unwrap();
-        assert_eq!(shas_completos(&shas, 1).map(|v| v[0].len()), Some(40));
+        assert_eq!(shas.trim().len(), 40, "{shas}");
         let _ = std::fs::remove_dir_all(&d);
     }
 }
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 0d318d505400cf903b38867e6846e8355967785c..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/03-en-revision/TASK-127/revision/informe-revision-1-code-quality-reviewer.md  |  51 +++++++++++++++++++---
 tareas/03-en-revision/TASK-127/revision/informe-revision-1-rust-wasm-reviewer.md     | Bin 866 -> 2889 bytes
 tareas/03-en-revision/TASK-127/revision/informe-revision-2-code-quality-reviewer.md  |  21 +++++++++
 tareas/03-en-revision/TASK-127/revision/informe-revision-2-rust-wasm-reviewer.md     |  21 +++++++++
 tareas/03-en-revision/TASK-127/revision/peticion-revision-2-code-quality-reviewer.md | 248 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-127/revision/peticion-revision-2-rust-wasm-reviewer.md    | 243 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-127/tarea.md                                              |  12 +++++-
 7 files changed, 590 insertions(+), 6 deletions(-)
````
