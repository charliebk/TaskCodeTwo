# Peticion de revision — TASK-127 (ronda 2)

- Tarea: TASK-127 — Escrituras de estado con muchos ficheros fallan en Windows (git hash-object)
- Rama revisada: fix/task-127-escrituras-de-estado-con-muchos-ficheros
- Rama base: develop
- Commit revisado (HEAD): 0d318d505400cf903b38867e6846e8355967785c
- Fecha: 2026-11-18
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-127 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 7d1963a0d80f19af3edc996e533d258f37d41a69 (el commit revisado en la ronda anterior)
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

- tareas/03-en-revision/TASK-127/tarea.md
- tareas/03-en-revision/TASK-127/planificacion/
- crates/tc2/src/git/estado.rs
- docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md

## Hallazgos de la ronda 1 pendientes de verificar

No tienes hallazgos pendientes propios: recibes esta ronda porque la correccion
toca ficheros de tu dominio. Revisa en el diff de esta ronda lo que la correccion
cambia en tu terreno y si abre un fallo.

## Commits a revisar (git log 7d1963a0d80f19af3edc996e533d258f37d41a69..HEAD)

````
0d318d5 chore(TASK-127): autorrevision ronda 2
4589b99 fix(TASK-127): tests del fallo de git con stdin y del control de shas
fb22e0f chore(TASK-127): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 7d1963a0d80f19af3edc996e533d258f37d41a69..HEAD)

````diff
diff --git a/crates/tc2/src/git/estado.rs b/crates/tc2/src/git/estado.rs
index e3abb7b..6d069da 100644
--- a/crates/tc2/src/git/estado.rs
+++ b/crates/tc2/src/git/estado.rs
@@ -286,22 +286,25 @@ fn con_entrada(
     if s.codigo == 0 {
         Ok(s.stdout)
     } else {
+        let s = super::SalidaGit {
+            codigo: s.codigo,
+            stdout: String::new(),
+            stderr: s.stderr,
+        };
         Err(Informe::nuevo(op, Resultado::Error)
             .con("git", sub.nombre())
-            .diagnostico(
-                Mensaje::nuevo(&txt::GIT_FALLO)
-                    .con("subcomando", sub.nombre())
-                    .con(
-                        "error",
-                        s.stderr
-                            .lines()
-                            .find(|l| !l.trim().is_empty())
-                            .unwrap_or_default(),
-                    ),
-            ))
+            .diagnostico(git_fallo(sub, &s)))
     }
 }
 
+/// Un sha por linea de la salida de `hash-object --stdin-paths`, solo si hay
+/// exactamente uno por fichero: si faltara alguno, emparejar por orden daria
+/// a cada ruta el contenido de otra.
+fn shas_completos(salida: &str, n: usize) -> Option<Vec<&str>> {
+    let shas: Vec<&str> = salida.lines().map(str::trim).collect();
+    (shas.len() == n).then_some(shas)
+}
+
 /// Construye un commit con el arbol de `base` mas `cambios` y `base` como
 /// padre (sin padre si `base` es `None`: la raiz de la rama huerfana). No
 /// mueve ninguna rama. `None` si el arbol queda igual que el de `base`.
@@ -342,12 +345,9 @@ pub(crate) fn construir_commit(
         }
         let shas = con_entrada(git, dir, op, Subcomando::HashObject, None, rutas.as_bytes())?;
         let shas = String::from_utf8_lossy(&shas);
-        let shas: Vec<&str> = shas.lines().map(str::trim).collect();
-        if shas.len() != cambios.len() {
-            return Err(
-                Informe::nuevo(op, Resultado::Error).diagnostico(&txt::HASH_OBJECT_INCOMPLETO)
-            );
-        }
+        let shas = shas_completos(&shas, cambios.len()).ok_or_else(|| {
+            Informe::nuevo(op, Resultado::Error).diagnostico(&txt::HASH_OBJECT_INCOMPLETO)
+        })?;
         let mut entradas = String::new();
         for (c, sha) in cambios.iter().zip(&shas) {
             entradas.push_str(&format!("100644 {sha}\t{}\0", c.ruta));
@@ -481,3 +481,79 @@ mod tests {
         }
     }
 }
+
+#[cfg(test)]
+#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
+mod tests_stdin {
+    use super::*;
+
+    /// Repo vacio de verdad en una carpeta temporal propia.
+    fn repo(nombre: &str) -> PathBuf {
+        let d = std::env::temp_dir().join(format!("tc2-estado-{nombre}-{}", std::process::id()));
+        let _ = std::fs::remove_dir_all(&d);
+        std::fs::create_dir_all(&d).unwrap();
+        let ok = std::process::Command::new("git")
+            .current_dir(&d)
+            .args(["init", "-q"])
+            .status()
+            .unwrap()
+            .success();
+        assert!(ok);
+        d
+    }
+
+    #[test]
+    fn shas_completos_exige_uno_por_fichero() {
+        assert_eq!(shas_completos("a\nb\n", 2), Some(vec!["a", "b"]));
+        assert_eq!(shas_completos("a\n", 2), None);
+        assert_eq!(shas_completos("a\nb\nc\n", 2), None);
+        assert_eq!(shas_completos("", 0), Some(vec![]));
+    }
+
+    /// TASK-127: si git falla con la entrada por stdin, `con_entrada` da
+    /// `error` con el subcomando, no `Ok`; con datos buenos, un sha por ruta.
+    #[test]
+    fn con_entrada_da_error_si_git_falla() {
+        let d = repo("con-entrada");
+        let git = Git::nuevo();
+        let entrada = format!("{}\n", d.join("no-existe").display());
+        let r = con_entrada(
+            &git,
+            &d,
+            "prueba",
+            Subcomando::HashObject,
+            None,
+            entrada.as_bytes(),
+        );
+        let i = r.expect_err("hash-object de un fichero que no existe");
+        assert_eq!(i.resultado, Resultado::Error);
+        assert!(i.linea().contains("git=hash-object"), "{}", i.linea());
+
+        let indice = d.join("indice");
+        let r = con_entrada(
+            &git,
+            &d,
+            "prueba",
+            Subcomando::UpdateIndex,
+            Some(&indice),
+            b"basura sin forma\0",
+        );
+        let i = r.expect_err("update-index con una entrada mal formada");
+        assert!(i.linea().contains("git=update-index"), "{}", i.linea());
+
+        let f = d.join("blob");
+        std::fs::write(&f, "hola\n").unwrap();
+        let entrada = format!("{}\n", f.display());
+        let r = con_entrada(
+            &git,
+            &d,
+            "prueba",
+            Subcomando::HashObject,
+            None,
+            entrada.as_bytes(),
+        );
+        let shas = String::from_utf8(r.expect("hash-object de un fichero que existe")).unwrap();
+        assert_eq!(shas_completos(&shas, 1).map(|v| v[0].len()), Some(40));
+        let _ = std::fs::remove_dir_all(&d);
+    }
+}
diff --git a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
index 610ebbc..c522d20 100644
--- a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
+++ b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
@@ -575,4 +575,4 @@ con su evidencia en los tests de `crates/tc2/tests/estado_*.rs`:
    `sinred`. El test de F2-2 que simulaba lo transitorio con un hook que
    rechaza una vez pasa a simularlo con un `receive-pack` que muere la
    primera vez.
-- Adenda (2026-11-18, TASK-127): `commit-estado` pasa las rutas de los blobs y las entradas del índice por stdin, no por argumentos: con 1000 ficheros la línea de órdenes pasaba del tope de Windows y `hash-object` fallaba. El puerto `Git` admite entrada por stdin en tres formas exactas y en ninguna otra: `cat-file` (F2-4b), `hash-object -w --no-filters --stdin-paths` (`Git::ejecutar_con_entrada`) y `update-index --add -z --index-info` solo con índice temporal de ruta absoluta (`Git::ejecutar_con_indice_y_entrada`). `update-ref --stdin` sigue prohibido. Un commit de estado lanza así un número fijo de procesos sea cual sea el número de ficheros (test `mil_ficheros_en_una_transaccion_con_procesos_acotados` de `estado_casos.rs`); el registro de la capa deja de mostrar esas rutas.
+- Adenda (2026-11-18, TASK-127): `commit-estado` pasa las rutas de los blobs y las entradas del índice por stdin, no por argumentos: con 1000 ficheros la línea de órdenes pasaba del tope de Windows y `hash-object` fallaba. El puerto `Git` admite entrada por stdin solo en tres subcomandos: `cat-file` (F2-4b, solo lectura, con sus argumentos comprobados como los de cualquier llamada) y, en su forma exacta y ninguna otra, `hash-object -w --no-filters --stdin-paths` (`Git::ejecutar_con_entrada`) y `update-index --add -z --index-info` solo con índice temporal de ruta absoluta (`Git::ejecutar_con_indice_y_entrada`). `update-ref --stdin` sigue prohibido. Un commit de estado lanza así un número fijo de procesos sea cual sea el número de ficheros (test `mil_ficheros_en_una_transaccion_con_procesos_acotados` de `estado_casos.rs`); el registro de la capa deja de mostrar esas rutas.
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 7d1963a0d80f19af3edc996e533d258f37d41a69..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-127/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-127/planificacion/plan-final.md                 |   0
 tareas/03-en-revision/TASK-127/revision/informe-revision-1-code-quality-reviewer.md         |  21 ++++++
 tareas/03-en-revision/TASK-127/revision/informe-revision-1-rust-wasm-reviewer.md            |  21 ++++++
 tareas/03-en-revision/TASK-127/revision/peticion-revision-1-code-quality-reviewer.md        |  75 +++++++++++++++++++++
 tareas/03-en-revision/TASK-127/revision/peticion-revision-1-rust-wasm-reviewer.md           | 366 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-127/tarea.md                                    |  12 +++-
 7 files changed, 492 insertions(+), 3 deletions(-)
````
