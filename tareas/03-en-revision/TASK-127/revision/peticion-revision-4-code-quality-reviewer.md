# Peticion de revision — TASK-127 (ronda 4)

- Tarea: TASK-127 — Escrituras de estado con muchos ficheros fallan en Windows (git hash-object)
- Rama revisada: fix/task-127-escrituras-de-estado-con-muchos-ficheros
- Rama base: develop
- Commit revisado (HEAD): e7646219e90763898c5c0fb75a8298e8391f4486
- Fecha: 2026-11-18
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-127 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde fa39e430c433bf9411d29cb9b108ee2daf7b55df (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (1 hallazgo(s)))

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

- tareas/03-en-revision/TASK-127/tarea.md
- tareas/03-en-revision/TASK-127/planificacion/
- crates/tc2/src/git/estado.rs
- crates/tc2/src/git/puerto.rs

## Hallazgos de la ronda 3 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/git/estado.rs:355 | informe-revision-3-code-quality-reviewer.md |

## Commits a revisar (git log fa39e430c433bf9411d29cb9b108ee2daf7b55df..HEAD)

````
e764621 chore(TASK-127): ronda extra 4 y autorrevision
cdbd5ba test(TASK-127): construir_commit rechaza shas incompletos con un git falso
834affd chore(TASK-127): peticion de revision ronda 3
````

## Diff desde la ronda anterior (git diff fa39e430c433bf9411d29cb9b108ee2daf7b55df..HEAD)

````diff
diff --git a/crates/tc2/src/git/estado.rs b/crates/tc2/src/git/estado.rs
index 54dafe1..a69f629 100644
--- a/crates/tc2/src/git/estado.rs
+++ b/crates/tc2/src/git/estado.rs
@@ -537,6 +537,62 @@ mod tests_stdin {
         let _ = std::fs::remove_dir_all(&d);
     }
 
+    /// Git falso que delega en el real salvo `hash-object`, al que responde
+    /// con un unico sha sea cual sea el numero de rutas. El subcomando es el
+    /// quinto argumento (`-c color.ui=never -c core.quotepath=false <sub>`).
+    fn git_que_pierde_shas(d: &Path) -> Git {
+        let sha = "0123456789012345678901234567890123456789";
+        if cfg!(windows) {
+            let f = d.join("git-falso.cmd");
+            let guion = format!(
+                "@echo off\r\nif \"%5\"==\"hash-object\" (\r\n  echo {sha}\r\n  exit /b 0\r\n)\r\ngit %*\r\n"
+            );
+            std::fs::write(&f, guion).unwrap();
+            Git::con_binario(f)
+        } else {
+            let f = d.join("git-falso.sh");
+            let guion = format!(
+                "#!/bin/sh\nif [ \"$5\" = hash-object ]; then cat >/dev/null; echo {sha}; exit 0; fi\nexec git \"$@\"\n"
+            );
+            std::fs::write(&f, guion).unwrap();
+            #[cfg(unix)]
+            {
+                use std::os::unix::fs::PermissionsExt;
+                std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o755)).unwrap();
+            }
+            Git::con_binario(f)
+        }
+    }
+
+    /// TASK-127: si `hash-object` devuelve menos shas que ficheros,
+    /// `construir_commit` falla con `HASH_OBJECT_INCOMPLETO` y no construye
+    /// ningun commit (emparejar por orden daria a cada ruta otro contenido).
+    #[test]
+    fn construir_commit_rechaza_shas_incompletos() {
+        let d = repo("shas-incompletos");
+        let git = git_que_pierde_shas(&d);
+        let cambios = [Cambio::nuevo("a.md", "a\n"), Cambio::nuevo("b.md", "b\n")];
+        let i = construir_commit(&git, &d, "prueba", None, &cambios, "m")
+            .expect_err("dos ficheros y un solo sha");
+        assert_eq!(i.resultado, Resultado::Error);
+        let incompleto =
+            Informe::nuevo("prueba", Resultado::Error).diagnostico(&txt::HASH_OBJECT_INCOMPLETO);
+        assert_eq!(
+            i.diagnostico_en(Idioma::Es),
+            incompleto.diagnostico_en(Idioma::Es),
+            "{}",
+            i.linea()
+        );
+        assert!(
+            !git.registro()
+                .iter()
+                .any(|l| l.starts_with("git commit-tree")),
+            "{:?}",
+            git.registro()
+        );
+        let _ = std::fs::remove_dir_all(&d);
+    }
+
     /// TASK-127: si git falla con la entrada por stdin, `con_entrada` da
     /// `error` con el subcomando, no `Ok`; con datos buenos, un sha por ruta.
     #[test]
diff --git a/crates/tc2/src/git/puerto.rs b/crates/tc2/src/git/puerto.rs
index 81364cb..7b401a1 100644
--- a/crates/tc2/src/git/puerto.rs
+++ b/crates/tc2/src/git/puerto.rs
@@ -235,6 +235,16 @@ impl Git {
         }
     }
 
+    /// Puerto con otro binario en lugar de `git` (solo tests: un git falso
+    /// que provoca salidas que el real no da).
+    #[cfg(test)]
+    pub(crate) fn con_binario(binario: impl Into<OsString>) -> Self {
+        Self {
+            binario: binario.into(),
+            ..Self::nuevo()
+        }
+    }
+
     /// Anade variables de entorno a cada llamada (los tests aislan asi la
     /// configuracion del usuario sin tocar el entorno del proceso).
     #[must_use]
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff fa39e430c433bf9411d29cb9b108ee2daf7b55df..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/03-en-revision/TASK-127/revision/informe-revision-2-code-quality-reviewer.md  |  50 ++++++++++++++++++++++++---
 tareas/03-en-revision/TASK-127/revision/informe-revision-2-rust-wasm-reviewer.md     |  64 +++++++++++++++++++++++++++++-----
 tareas/03-en-revision/TASK-127/revision/informe-revision-3-code-quality-reviewer.md  |  21 ++++++++++++
 tareas/03-en-revision/TASK-127/revision/informe-revision-3-rust-wasm-reviewer.md     |  21 ++++++++++++
 tareas/03-en-revision/TASK-127/revision/peticion-revision-3-code-quality-reviewer.md | 201 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-127/revision/peticion-revision-3-rust-wasm-reviewer.md    | 196 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-127/tarea.md                                              |   8 ++++-
 7 files changed, 547 insertions(+), 14 deletions(-)
````
