# Peticion de revision — TASK-145 (ronda 1)

- Tarea: TASK-145 — Auditoria 4D: A7 config, init, desinstalar, i18n, confianza y ciclo (8 IMPORTANTE)
- Rama revisada: fix/task-145-auditoria-4d-a7-config-init-desinstalar
- Rama base: develop
- Commit revisado (HEAD): fce8bd62afb7e421e03fef98b93e6c8ff949cadf
- Fecha: 2026-11-25
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-145 (criterios de aceptacion y plan)
- Alcance: completa (motivo: primera ronda de revision)

## Instrucciones para el agente revisor

Eres un revisor INDEPENDIENTE del agente que implemento. Tu trabajo es
reproducir empiricamente, no leer el diff y opinar: clona el repo a un
directorio temporal, corre la suite tu mismo (salvo que esta
peticion ya traiga la verificacion de la ronda en verde para el commit revisado;
entonces vale ese resultado) y construye el caso que
rompe el codigo antes de reportarlo. Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. Vuelca tu salida en el
informe de esta ronda (informe-revision-1-rust-wasm-reviewer.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-145/tarea.md
- tareas/03-en-revision/TASK-145/planificacion/
- crates/tc2/src/ciclo/liberar.rs
- crates/tc2/src/ciclo/mod.rs
- crates/tc2/src/ciclo/operacion.rs
- crates/tc2/src/ciclo/prueba.rs
- crates/tc2/src/confianza/mod.rs
- crates/tc2/src/config/mod.rs
- crates/tc2/src/contrato.rs
- crates/tc2/src/desinstalar/maquina.rs
- crates/tc2/src/desinstalar/mod.rs
- crates/tc2/src/desinstalar/proyecto.rs
- crates/tc2/src/estado/lectura.rs
- crates/tc2/src/i18n/catalogo/comun.rs
- crates/tc2/src/i18n/catalogo/desinstalar.rs
- crates/tc2/src/i18n/catalogo/init.rs
- crates/tc2/src/i18n/guarda.rs
- crates/tc2/src/init/aplicar.rs
- crates/tc2/src/init/inventario.rs
- crates/tc2/src/init/mod.rs
- crates/tc2/src/lectura.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/plan_dato.rs
- crates/tc2/tests/init.rs
- crates/tc2/tests/verificar.rs

## Commits a revisar (git log develop..HEAD)

````
fce8bd6 docs(TASK-145): resultado, criterios y autorrevision
f2b311e fix(TASK-145): A7-11 test de ramas-sin-remoto con develop sin publicar
ff94d16 fix(TASK-145): A7-09 del_arbol da sin-config solo si no existe el fichero
be71fe0 fix(TASK-145): A7-10 la guarda de i18n caza Err con prosa y EXTERNO
d85a18a fix(TASK-145): A7-07 el fallo al aplicar va como Mensaje y se traduce
df58237 fix(TASK-145): A7-01 esqueleto plan como dato comun en plan_dato
6a155a9 fix(TASK-145): A7-03 estado y confiar comparten Calculo en confianza
09671c6 fix(TASK-145): A7-05 un solo ayudante repo_con_remoto en los tests de ciclo
d04891a fix(TASK-145): A7-04 soporte de test comun en ciclo::prueba
9703b07 chore(TASK-145): tarea en curso
````

## Diff de tu dominio (23 fichero(s) de 23; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/ciclo/liberar.rs b/crates/tc2/src/ciclo/liberar.rs
index 77d1c75..f64293a 100644
--- a/crates/tc2/src/ciclo/liberar.rs
+++ b/crates/tc2/src/ciclo/liberar.rs
@@ -387,28 +387,12 @@ pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
 #[cfg(test)]
 mod tests {
     use super::*;
+    use crate::ciclo::prueba::{AHORA, Extra, ana, cuenta, fila};
     use crate::estado::prueba::{EnMemoria, estado, motivo, texto};
     use crate::grafo::DESCARTADA;
-    use crate::identidad::Cuenta;
-    use crate::plataforma::Proveedor;
 
-    /// 2026-11-17T10:20:30Z.
-    const AHORA: u64 = 1_794_910_830;
     const DIA: u64 = 86_400;
 
-    fn cuenta(id: u64, login: &str) -> Cuenta {
-        Cuenta {
-            proveedor: Proveedor::GitHub,
-            id,
-            login: login.to_owned(),
-        }
-    }
-
-    /// Ana desde su agente (sin terminal).
-    fn ana() -> Quien {
-        Quien::Agente(cuenta(7, "ana"))
-    }
-
     /// Beto en un terminal.
     fn beto() -> Quien {
         Quien::Persona(cuenta(9, "beto"))
@@ -419,27 +403,13 @@ mod tests {
     }
 
     /// Un `tarea.md` con una fila de transicion de hace `hace` segundos.
+    /// Con el plan aprobado (estas operaciones no lo tocan).
     fn md(id: &str, estado: &str, asignado: Option<Asignado>, hace: u64) -> String {
-        let mut t = Tarea::nueva(id, &format!("Tarea {id}"), "2026-11-01");
-        t.estado = estado.to_owned();
-        if estado == DESCARTADA {
-            t.motivo = Some("no".to_owned());
-        }
-        t.plan_aprobado = true;
-        t.asignado_a = asignado;
-        t.rama = Some(crate::alta::rama_de(id, &t.tipo, &t.titulo));
-        let fila = format!(
-            "| {} | start | manual | persona | 0.1 |\n",
-            crate::toma::instante(AHORA - hace)
-        );
-        t.escribir(&Cuerpo::plantilla()) + &fila
-    }
-
-    fn fila(fase: &str, por: &str) -> String {
-        format!(
-            "| 2026-11-17T10:20:30Z | {fase} | manual | {por} | {} |\n",
-            crate::VERSION
-        )
+        let extra = Extra {
+            plan_aprobado: true,
+            hace: Some(hace),
+        };
+        crate::ciclo::prueba::md(id, estado, asignado, extra)
     }
 
     /// C1 (unitario): el dueno (su agente) libera una en curso: `planificada`,
diff --git a/crates/tc2/src/ciclo/mod.rs b/crates/tc2/src/ciclo/mod.rs
index ef0ff96..d619a15 100644
--- a/crates/tc2/src/ciclo/mod.rs
+++ b/crates/tc2/src/ciclo/mod.rs
@@ -34,6 +34,8 @@
 
 pub mod liberar;
 pub mod operacion;
+#[cfg(test)]
+pub(crate) mod prueba;
 
 use std::path::Path;
 
@@ -465,18 +467,24 @@ mod tests {
         None
     }
 
-    /// M-2 de la ronda 1: quien se reasigna a si misma la tarea de otra
-    /// persona (inactiva, en un terminal) se la queda con su worktree de
-    /// este clon: no se rechaza `sin-publicar` aunque tenga trabajo sin
-    /// publicar (`&& !para_mi`) ni se intenta quitar (`if para_mi`).
-    #[test]
-    fn reasignarse_a_si_misma_no_toca_su_worktree() -> R<()> {
+    /// Escenario git de los dos tests: un bare `remoto.git` y su clon.
+    struct Repo {
+        _borrar: Carpeta,
+        raiz: PathBuf,
+        clon: PathBuf,
+        git: Git,
+    }
+
+    /// Monta en `tc2-tests/{nombre}-{pid}` un clon de `develop` con un
+    /// primer commit de `ficheros` (ruta relativa, contenido), crea `ramas`
+    /// en ese commit y publica `develop` y `ramas` en `origin` (el bare).
+    fn repo_con_remoto(nombre: &str, ficheros: &[(&str, &str)], ramas: &[&str]) -> R<Repo> {
         let raiz = std::env::temp_dir()
             .join("tc2-tests")
-            .join(format!("ciclo-para-mi-{}", std::process::id()));
+            .join(format!("{nombre}-{}", std::process::id()));
         let _ = std::fs::remove_dir_all(&raiz);
         std::fs::create_dir_all(&raiz)?;
-        let _borrar = Carpeta(raiz.clone());
+        let borrar = Carpeta(raiz.clone());
         aislar(&raiz)?;
         sh(
             &raiz,
@@ -485,16 +493,49 @@ mod tests {
         )?;
         sh(&raiz, &raiz, &["init", "-q", "-b", "develop", "clon"])?;
         let (bare, clon) = (raiz.join("remoto.git"), raiz.join("clon"));
-        std::fs::create_dir_all(clon.join(".taskcode"))?;
-        std::fs::write(clon.join(crate::plataforma::personas::RUTA), PERSONAS)?;
+        for (ruta, contenido) in ficheros {
+            let ruta = clon.join(ruta);
+            if let Some(padre) = ruta.parent() {
+                std::fs::create_dir_all(padre)?;
+            }
+            std::fs::write(ruta, contenido)?;
+        }
         sh(&raiz, &clon, &["add", "."])?;
-        sh(&raiz, &clon, &["commit", "-q", "-m", "personas"])?;
-        sh(&raiz, &clon, &["branch", "main"])?;
+        sh(&raiz, &clon, &["commit", "-q", "-m", "inicial"])?;
+        for rama in ramas {
+            sh(&raiz, &clon, &["branch", rama])?;
+        }
         let url = bare.to_str().ok_or("ruta no UTF-8")?;
         sh(&raiz, &clon, &["remote", "add", "origin", url])?;
-        sh(&raiz, &clon, &["push", "-q", "origin", "develop", "main"])?;
+        let mut push = vec!["push", "-q", "origin", "develop"];
+        push.extend_from_slice(ramas);
+        sh(&raiz, &clon, &push)?;
         sh(&raiz, &clon, &["fetch", "-q", "origin"])?;
-        let g = Git::nuevo().con_entorno(entorno(&raiz));
+        let git = Git::nuevo().con_entorno(entorno(&raiz));
+        Ok(Repo {
+            _borrar: borrar,
+            raiz,
+            clon,
+            git,
+        })
+    }
+
+    /// M-2 de la ronda 1: quien se reasigna a si misma la tarea de otra
+    /// persona (inactiva, en un terminal) se la queda con su worktree de
+    /// este clon: no se rechaza `sin-publicar` aunque tenga trabajo sin
+    /// publicar (`&& !para_mi`) ni se intenta quitar (`if para_mi`).
+    #[test]
+    fn reasignarse_a_si_misma_no_toca_su_worktree() -> R<()> {
+        let Repo {
+            _borrar,
+            clon,
+            git: g,
+            ..
+        } = repo_con_remoto(
+            "ciclo-para-mi",
+            &[(crate::plataforma::personas::RUTA, PERSONAS)],
+            &["main"],
+        )?;
         // TASK-001, de beto y empezada en este clon.
         let entrada = crate::alta::entrada_de_args(&["Uno".to_owned()]).map_err(|i| i.linea())?;
         let alta = crate::alta::Alta::con_entrada(entrada);
@@ -536,28 +577,12 @@ mod tests {
     /// (no el de por defecto en silencio); con una valida, su valor.
     #[test]
     fn umbral_sin_config_invalida_o_valida() -> R<()> {
-        let raiz = std::env::temp_dir()
-            .join("tc2-tests")
-            .join(format!("ciclo-umbral-{}", std::process::id()));
-        let _ = std::fs::remove_dir_all(&raiz);
-        std::fs::create_dir_all(&raiz)?;
-        let _borrar = Carpeta(raiz.clone());
-        aislar(&raiz)?;
-        sh(
-            &raiz,
-            &raiz,
-            &["init", "-q", "--bare", "-b", "develop", "remoto.git"],
-        )?;
-        sh(&raiz, &raiz, &["init", "-q", "-b", "develop", "clon"])?;
-        let (bare, clon) = (raiz.join("remoto.git"), raiz.join("clon"));
-        std::fs::write(clon.join("README.md"), "linea 1\n")?;
-        sh(&raiz, &clon, &["add", "."])?;
-        sh(&raiz, &clon, &["commit", "-q", "-m", "inicial"])?;
-        let url = bare.to_str().ok_or("ruta no UTF-8")?;
-        sh(&raiz, &clon, &["remote", "add", "origin", url])?;
-        sh(&raiz, &clon, &["push", "-q", "origin", "develop"])?;
-        sh(&raiz, &clon, &["fetch", "-q", "origin"])?;
-        let g = Git::nuevo().con_entorno(entorno(&raiz));
+        let Repo {
+            _borrar,
+            raiz,
+            clon,
+            git: g,
+        } = repo_con_remoto("ciclo-umbral", &[("README.md", "linea 1\n")], &[])?;
         assert_eq!(
             umbral_de_inactividad(&g, &clon, LIBERAR).map_err(|i| i.linea()),
             Ok(DIAS_POR_DEFECTO)
diff --git a/crates/tc2/src/ciclo/operacion.rs b/crates/tc2/src/ciclo/operacion.rs
index cede29d..c52c8d2 100644
--- a/crates/tc2/src/ciclo/operacion.rs
+++ b/crates/tc2/src/ciclo/operacion.rs
@@ -267,37 +267,17 @@ pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
 #[cfg(test)]
 mod tests {
     use super::*;
+    use crate::ciclo::prueba::{AHORA, Extra, ana};
     use crate::estado::prueba::{estado, motivo, texto};
-    use crate::identidad::Cuenta;
-    use crate::plataforma::Proveedor;
-
-    /// 2026-11-17T10:20:30Z.
-    const AHORA: u64 = 1_794_910_830;
-
-    fn ana() -> Quien {
-        Quien::Agente(Cuenta {
-            proveedor: Proveedor::GitHub,
-            id: 7,
-            login: "ana".to_owned(),
-        })
-    }
 
+    /// Sin plan aprobado ni filas de transicion.
     fn md(id: &str, estado: &str, asignado: Option<Asignado>) -> String {
-        let mut t = Tarea::nueva(id, &format!("Tarea {id}"), "2026-11-01");
-        t.estado = estado.to_owned();
-        if estado == DESCARTADA {
-            t.motivo = Some("el de entonces".to_owned());
-        }
-        t.asignado_a = asignado;
-        t.rama = Some(crate::alta::rama_de(id, &t.tipo, &t.titulo));
-        t.escribir(&Cuerpo::plantilla())
+        crate::ciclo::prueba::md(id, estado, asignado, Extra::default())
     }
 
+    /// La fila que anade el agente de Ana.
     fn fila(fase: &str) -> String {
-        format!(
-            "| 2026-11-17T10:20:30Z | {fase} | manual | agente | {} |\n",
-            crate::VERSION
-        )
+        crate::ciclo::prueba::fila(fase, "agente")
     }
 
     /// C1 (unitario): desde cada estado de origen, `estado: descartada`, el
diff --git a/crates/tc2/src/ciclo/prueba.rs b/crates/tc2/src/ciclo/prueba.rs
new file mode 100644
index 0000000..23a3bbc
--- /dev/null
+++ b/crates/tc2/src/ciclo/prueba.rs
@@ -0,0 +1,92 @@
+//! Soporte comun de los tests unitarios de las operaciones de `ciclo`
+//! (A7-04): el instante fijo, la persona de los escenarios, el `tarea.md` de
+//! partida y la fila de transicion que anade una operacion. Lo comun a toda
+//! operacion sobre la rama de estado esta en `estado::prueba`.
+
+use crate::grafo::DESCARTADA;
+use crate::identidad::{Cuenta, Quien};
+use crate::plataforma::Proveedor;
+use crate::tarea::{Asignado, Cuerpo, Tarea};
+
+/// 2026-11-17T10:20:30Z.
+pub(crate) const AHORA: u64 = 1_794_910_830;
+
+/// Cuenta de GitHub con ese id y login.
+pub(crate) fn cuenta(id: u64, login: &str) -> Cuenta {
+    Cuenta {
+        proveedor: Proveedor::GitHub,
+        id,
+        login: login.to_owned(),
+    }
+}
+
+/// Ana desde su agente (sin terminal).
+pub(crate) fn ana() -> Quien {
+    Quien::Agente(cuenta(7, "ana"))
+}
+
+/// Lo que cambia entre los `tarea.md` de partida de cada modulo.
+#[derive(Default)]
+pub(crate) struct Extra {
+    /// `plan_aprobado: true`.
+    pub(crate) plan_aprobado: bool,
+    /// Una fila de transicion `start` de hace tantos segundos (respecto a
+    /// [`AHORA`]); sin ella, la tabla de transiciones queda vacia.
+    pub(crate) hace: Option<u64>,
+}
+
+/// Un `tarea.md` con ese estado y asignado, su rama y lo que diga `extra`.
+/// Una descartada lleva el motivo `el de entonces`.
+pub(crate) fn md(id: &str, estado: &str, asignado: Option<Asignado>, extra: Extra) -> String {
+    let mut t = Tarea::nueva(id, &format!("Tarea {id}"), "2026-11-01");
+    t.estado = estado.to_owned();
+    if estado == DESCARTADA {
+        t.motivo = Some("el de entonces".to_owned());
+    }
+    t.plan_aprobado = extra.plan_aprobado;
+    t.asignado_a = asignado;
+    t.rama = Some(crate::alta::rama_de(id, &t.tipo, &t.titulo));
+    let fila = extra.hace.map_or_else(String::new, |hace| {
+        format!(
+            "| {} | start | manual | persona | 0.1 |\n",
+            crate::toma::instante(AHORA - hace)
+        )
+    });
+    t.escribir(&Cuerpo::plantilla()) + &fila
+}
+
+/// La fila de transicion de `fase` que anade `por` en [`AHORA`].
+pub(crate) fn fila(fase: &str, por: &str) -> String {
+    format!(
+        "| 2026-11-17T10:20:30Z | {fase} | manual | {por} | {} |\n",
+        crate::VERSION
+    )
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    /// El `tarea.md` de partida: una descartada lleva siempre el motivo
+    /// `el de entonces` (un solo literal en los dos modulos, A7-04); `extra`
+    /// solo cambia `plan_aprobado` y la fila de transicion.
+    #[test]
+    fn md_de_partida() {
+        let liberar = Extra {
+            plan_aprobado: true,
+            hace: Some(0),
+        };
+        for extra in [Extra::default(), liberar] {
+            let (aprobado, con_fila) = (extra.plan_aprobado, extra.hace.is_some());
+            let t = md("TASK-001", DESCARTADA, None, extra);
+            assert!(t.contains("\nmotivo: \"el de entonces\"\n"), "{t}");
+            assert!(t.contains(&format!("\nplan_aprobado: {aprobado}\n")), "{t}");
+            assert_eq!(
+                t.ends_with("| start | manual | persona | 0.1 |\n"),
+                con_fila
+            );
+        }
+        let t = md("TASK-001", "planificada", None, Extra::default());
+        assert!(t.contains("\nmotivo: null\n"), "{t}");
+    }
+}
diff --git a/crates/tc2/src/confianza/mod.rs b/crates/tc2/src/confianza/mod.rs
index bbf9046..6dfe2f0 100644
--- a/crates/tc2/src/confianza/mod.rs
+++ b/crates/tc2/src/confianza/mod.rs
@@ -203,42 +203,18 @@ pub fn estado(
     incluye: &[String],
     op: &str,
 ) -> Result<Evaluacion, Informe> {
-    let ruta = ruta_almacen(almacen, repo);
-    let mut a = leer_almacen(&ruta);
-    let entrada = a.claves.get(clave).cloned();
-    let c = cierre::calcular(
-        &repo.raiz,
-        &repo.indice,
-        orden,
-        incluye,
-        entrada.as_ref().map(|e| e.ficheros.as_slice()),
-    )
-    .map_err(|e| error_de_cierre(op, &e))?;
-    let huella = cierre::huella(orden, incluye, &c.ficheros);
-    let estado = match &entrada {
+    let mut c = Calculo::nuevo(repo, almacen, clave, orden, incluye, op, true)?;
+    let estado = match &c.anterior {
         None => Estado::Nunca,
-        Some(e) if e.huella == huella => Estado::Confiado,
+        Some(e) if e.huella == c.huella => Estado::Confiado,
         Some(_) => Estado::Cambiado,
     };
-    if estado == Estado::Confiado && c.hasheados > 0 {
+    if estado == Estado::Confiado && c.cierre.hasheados > 0 {
         // Solo la cache: misma huella. Si no se puede escribir, la proxima
-        // vez se vuelve a leer, nada mas.
-        a.claves.insert(
-            clave.to_owned(),
-            Entrada {
-                huella: huella.clone(),
-                ficheros: c.ficheros.clone(),
-            },
-        );
-        let _ = escribir_almacen(&ruta, &a, op);
+        // vez se vuelve a leer, nada mas: el fallo no se propaga.
+        let _ = c.guardar(clave, op);
     }
-    Ok(Evaluacion {
-        estado,
-        huella,
-        ficheros: c.ficheros.len(),
-        hasheados: c.hasheados,
-        globs_vacios: c.globs_vacios.clone(),
-    })
+    Ok(c.evaluacion(estado))
 }
 
 /// Confia en `orden` tal como esta ahora en el arbol: rehashea todo el
@@ -251,27 +227,74 @@ pub fn confiar(
     incluye: &[String],
     op: &str,
 ) -> Result<Evaluacion, Informe> {
-    let ruta = ruta_almacen(almacen, repo);
-    let mut a = leer_almacen(&ruta);
-    a.version = VERSION_ALMACEN;
-    let c = cierre::calcular(&repo.raiz, &repo.indice, orden, incluye, None)
-        .map_err(|e| error_de_cierre(op, &e))?;
-    let huella = cierre::huella(orden, incluye, &c.ficheros);
-    a.claves.insert(
-        clave.to_owned(),
-        Entrada {
-            huella: huella.clone(),
-            ficheros: c.ficheros.clone(),
-        },
-    );
-    escribir_almacen(&ruta, &a, op)?;
-    Ok(Evaluacion {
-        estado: Estado::Confiado,
-        huella,
-        ficheros: c.ficheros.len(),
-        hasheados: c.hasheados,
-        globs_vacios: c.globs_vacios.clone(),
-    })
+    let mut c = Calculo::nuevo(repo, almacen, clave, orden, incluye, op, false)?;
+    c.guardar(clave, op)?;
+    Ok(c.evaluacion(Estado::Confiado))
+}
+
+/// Lo que comparten [`estado`] y [`confiar`]: el almacen leido, la entrada
+/// anterior de la clave, el cierre calculado y su huella.
+struct Calculo {
+    ruta: PathBuf,
+    almacen: Almacen,
+    anterior: Option<Entrada>,
+    cierre: cierre::Cierre,
+    huella: String,
+}
+
+impl Calculo {
+    /// Lee el almacen y calcula el cierre de `orden`; con `cache`, reutiliza
+    /// el sha de los ficheros de la entrada anterior que no cambiaron.
+    fn nuevo(
+        repo: &Repo,
+        almacen: &Path,
+        clave: &str,
+        orden: &str,
+        incluye: &[String],
+        op: &str,
+        cache: bool,
+    ) -> Result<Self, Informe> {
+        let ruta = ruta_almacen(almacen, repo);
+        let almacen = leer_almacen(&ruta);
+        let anterior = almacen.claves.get(clave).cloned();
+        let previos = anterior
+            .as_ref()
+            .filter(|_| cache)
+            .map(|e| e.ficheros.as_slice());
+        let cierre = cierre::calcular(&repo.raiz, &repo.indice, orden, incluye, previos)
+            .map_err(|e| error_de_cierre(op, &e))?;
+        let huella = cierre::huella(orden, incluye, &cierre.ficheros);
+        Ok(Self {
+            ruta,
+            almacen,
+            anterior,
+            cierre,
+            huella,
+        })
+    }
+
+    /// Guarda en el almacen la huella y los ficheros de `clave`.
+    fn guardar(&mut self, clave: &str, op: &str) -> Result<(), Informe> {
+        self.almacen.version = VERSION_ALMACEN;
+        self.almacen.claves.insert(
+            clave.to_owned(),
+            Entrada {
+                huella: self.huella.clone(),
+                ficheros: self.cierre.ficheros.clone(),
+            },
+        );
+        escribir_almacen(&self.ruta, &self.almacen, op)
+    }
+
+    fn evaluacion(self, estado: Estado) -> Evaluacion {
+        Evaluacion {
+            estado,
+            ficheros: self.cierre.ficheros.len(),
+            hasheados: self.cierre.hasheados,
+            globs_vacios: self.cierre.globs_vacios,
+            huella: self.huella,
+        }
+    }
 }
 
 /// `tc2 confiar <clave>` en la carpeta actual: confia en la orden de `clave`
diff --git a/crates/tc2/src/config/mod.rs b/crates/tc2/src/config/mod.rs
index 7076ede..f612467 100644
--- a/crates/tc2/src/config/mod.rs
+++ b/crates/tc2/src/config/mod.rs
@@ -530,16 +530,19 @@ pub fn de_base(git: &Git, dir: &Path) -> Result<Config, Informe> {
 
 /// Lee la configuracion del arbol de trabajo de `raiz` (F1-8 decision 5:
 /// la orden que se confia y se ejecuta es la del arbol, no la de la base).
-/// `Err` lleva el informe `config rechazada`.
+/// `Err` lleva el informe `config rechazada` (sin fichero, `sin-config`), o
+/// `config error ruta=...` si el fichero existe y no se lee (sin permiso,
+/// no UTF-8: A7-09).
 pub fn del_arbol(raiz: &Path) -> Result<Config, Informe> {
     match std::fs::read_to_string(raiz.join(RUTA)) {
         Ok(texto) => leer(&texto).map_err(|e| e.informe()),
-        Err(_) => Err(Invalida::una(
+        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(Invalida::una(
             Motivo::SinConfig,
             "",
             Mensaje::nuevo(&t::NO_HAY_CONFIG_EN_ARBOL).con("ruta", RUTA),
         )
         .informe()),
+        Err(e) => Err(crate::lectura::no_se_lee(NOMBRE, Path::new(RUTA), &e)),
     }
 }
 
@@ -588,4 +591,42 @@ mod tests {
         assert_eq!(leer("---\r\nformato: 2\r\n"), Ok(Config::suelo()));
         assert_eq!(leer(texto_minimo()), Ok(Config::suelo()));
     }
+
+    /// A7-09: un `config.yml` que existe y no se lee (UTF-16LE con BOM, lo
+    /// que deja el Bloc de notas con «Unicode») da `config error` con la
+    /// ruta y la causa, no `sin-config` con el arreglo de `tc2 init`; sin
+    /// fichero, `sin-config`.
+    #[test]
+    fn del_arbol_distingue_no_existe_de_no_se_lee() -> std::io::Result<()> {
+        let raiz = std::env::temp_dir()
+            .join("tc2-tests")
+            .join(format!("config-del-arbol-{}", std::process::id()));
+        let _ = std::fs::remove_dir_all(&raiz);
+        std::fs::create_dir_all(raiz.join(".taskcode"))?;
+        let sin = del_arbol(&raiz).err();
+        let mut utf16 = vec![0xFF, 0xFE];
+        utf16.extend(
+            "---\nformato: 2\n"
+                .encode_utf16()
+                .flat_map(u16::to_le_bytes),
+        );
+        std::fs::write(raiz.join(RUTA), utf16)?;
+        let ilegible = del_arbol(&raiz).err();
+        let _ = std::fs::remove_dir_all(&raiz);
+        assert_eq!(
+            sin.as_ref().map(|i| (i.resultado, i.valor("motivo"))),
+            Some((Resultado::Rechazada, Some("sin-config")))
+        );
+        let i = ilegible.ok_or(std::io::ErrorKind::InvalidData)?;
+        assert_eq!(i.linea(), format!("config error ruta={RUTA}"));
+        let diag = i
+            .diagnostico_en(crate::i18n::Idioma::Es)
+            .unwrap_or_default();
+        assert!(
+            diag.starts_with(&format!("no se puede leer {RUTA}: ")),
+            "{diag}"
+        );
+        assert!(!diag.contains("tc2 init"), "{diag}");
+        Ok(())
+    }
 }
diff --git a/crates/tc2/src/contrato.rs b/crates/tc2/src/contrato.rs
index a92ddaf..846c319 100644
--- a/crates/tc2/src/contrato.rs
+++ b/crates/tc2/src/contrato.rs
@@ -211,6 +211,15 @@ impl Informe {
         self.diagnostico.as_ref().map(|d| d.texto(idioma))
     }
 
+    /// El diagnostico como mensaje neutro, para meterlo en el hueco de otro
+    /// (F2-8: se traduce al escribir la salida, A7-07). Sin diagnostico
+    /// queda la linea de contrato, que no se traduce (ADR F1-4).
+    pub(crate) fn mensaje(&self) -> Mensaje {
+        self.diagnostico
+            .clone()
+            .unwrap_or_else(|| Mensaje::externo(self.linea()))
+    }
+
     /// Lo convierte en la salida del proceso, con el diagnostico en espanol.
     pub fn salida(&self, json: bool) -> Salida {
         self.salida_en(json, Idioma::Es)
diff --git a/crates/tc2/src/desinstalar/maquina.rs b/crates/tc2/src/desinstalar/maquina.rs
index 84e7c2a..4d07e35 100644
--- a/crates/tc2/src/desinstalar/maquina.rs
+++ b/crates/tc2/src/desinstalar/maquina.rs
@@ -9,10 +9,11 @@ use std::path::{Path, PathBuf};
 use distribucion::{MARCA_INICIO_PERFIL, STATUS_LINE};
 
 use super::texto::{hooks_con_tc2, quitar_bloque, quitar_del_path, quitar_miembro};
-use super::{Accion, Contexto, Decision, Efecto, leer, no_se_lee, orden_borrar, t};
+use super::{Accion, Contexto, Decision, Efecto, NOMBRE, orden_borrar, t};
 use crate::contrato::Informe;
 use crate::git::misma_ruta;
 use crate::instalacion::{Modo, Sistema, orden_del_gestor, ruta_old};
+use crate::lectura::{no_se_lee, opcional};
 
 /// Perfiles del shell donde `install.sh` puede dejar el bloque.
 const PERFILES: [&str; 4] = [".profile", ".bashrc", ".zshrc", ".bash_profile"];
@@ -62,7 +63,7 @@ pub(super) fn plan(c: &Contexto<'_>, v: &mut Vec<Accion>) -> Result<(), Informe>
     let mut path = if windows {
         c.maquina
             .leer_path_usuario()
-            .map_err(|er| no_se_lee(Path::new("HKCU\\Environment\\Path"), &er))?
+            .map_err(|er| no_se_lee(NOMBRE, Path::new("HKCU\\Environment\\Path"), &er))?
     } else {
         None
     };
@@ -91,7 +92,7 @@ pub(super) fn plan(c: &Contexto<'_>, v: &mut Vec<Accion>) -> Result<(), Informe>
             && windows
             && c.maquina
                 .en_uso_por_otro(&bin, e.pid)
-                .map_err(|er| no_se_lee(&bin, &er))?;
+                .map_err(|er| no_se_lee(NOMBRE, &bin, &er))?;
         let mut grupo: Vec<Accion> = Vec::new();
         let path_antes = path.clone();
         // Lo que este grupo quita: solo cuenta para las carpetas si se aplica.
@@ -237,7 +238,9 @@ pub(super) fn plan(c: &Contexto<'_>, v: &mut Vec<Accion>) -> Result<(), Informe>
     if !windows && let Some(home) = &e.home {
         for nombre in PERFILES {
             let p = home.join(nombre);
-            let Some(bytes) = leer(&p)? else { continue };
+            let Some(bytes) = opcional(NOMBRE, &p)? else {
+                continue;
+            };
             let Ok(texto) = String::from_utf8(bytes.clone()) else {
                 if bytes
                     .windows(MARCA_INICIO_PERFIL.len())
@@ -366,7 +369,11 @@ fn instalados(c: &Contexto<'_>) -> Result<Option<serde_json::Value>, Informe> {
     };
     match crate::inventario::leer_instalados(&dir) {
         Ok(v) => Ok(v.and_then(Result::ok)),
-        Err(e) => Err(no_se_lee(&crate::inventario::ruta_instalados(&dir), &e)),
+        Err(e) => Err(no_se_lee(
+            NOMBRE,
+            &crate::inventario::ruta_instalados(&dir),
+            &e,
+        )),
     }
 }
 
@@ -425,7 +432,7 @@ fn claude(
         return Ok(());
     };
     let settings = dir.join("settings.json");
-    if let Some(bytes) = leer(&settings)? {
+    if let Some(bytes) = opcional(NOMBRE, &settings)? {
         let texto = String::from_utf8(bytes.clone()).ok();
         let valor = texto
             .as_deref()
diff --git a/crates/tc2/src/desinstalar/mod.rs b/crates/tc2/src/desinstalar/mod.rs
index 861f638..8a97ef5 100644
--- a/crates/tc2/src/desinstalar/mod.rs
+++ b/crates/tc2/src/desinstalar/mod.rs
@@ -27,9 +27,11 @@ use std::path::{Path, PathBuf};
 use crate::Salida;
 use crate::contrato::{Informe, LineaAccion, Resultado};
 use crate::git::Git;
+use crate::i18n::catalogo::comun::CAMBIO_DESDE_EL_PLAN;
 use crate::i18n::catalogo::desinstalar as txt;
 use crate::i18n::{Idioma, Mensaje};
 use crate::instalacion::{Entorno, Sistema};
+use crate::plan_dato::{self, Paso, externo, t};
 pub use puertos::{Maquina, MaquinaEnMemoria, MaquinaReal};
 
 /// Nombre de la orden.
@@ -298,6 +300,19 @@ impl Accion {
     }
 }
 
+impl Paso for Accion {
+    fn aplicable(&self) -> bool {
+        self.decision == Decision::Borra
+    }
+    fn aplazar(&mut self, motivo: &str) {
+        self.decision = Decision::Pendiente;
+        self.motivo = Some(motivo.to_owned());
+    }
+    fn ruta(&self) -> &str {
+        &self.ruta
+    }
+}
+
 /// Lo que se necesita para desinstalar, inyectable en los tests.
 pub struct Contexto<'a> {
     /// HOME, `LOCALAPPDATA`, el ejecutable...
@@ -364,31 +379,6 @@ pub(crate) fn orden_borrar(sistema: Sistema, ruta: &Path) -> String {
     }
 }
 
-/// Ruta como texto para la linea.
-pub(crate) fn t(p: &Path) -> String {
-    p.display().to_string()
-}
-
-/// Lee un fichero; `None` si no existe.
-pub(crate) fn leer(p: &Path) -> Result<Option<Vec<u8>>, Informe> {
-    match std::fs::read(p) {
-        Ok(b) => Ok(Some(b)),
-        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
-        Err(e) => Err(no_se_lee(p, &e)),
-    }
-}
-
-/// Informe de error al leer `ruta`.
-pub(crate) fn no_se_lee(ruta: &Path, e: &std::io::Error) -> Informe {
-    Informe::nuevo(NOMBRE, Resultado::Error)
-        .con("ruta", ruta.display().to_string())
-        .diagnostico(
-            Mensaje::nuevo(&txt::NO_SE_LEE)
-                .con("ruta", ruta.display().to_string())
-                .con("error", e.to_string()),
-        )
-}
-
 /// El plan completo: primero el proyecto y luego la maquina (el orden
 /// recomendado).
 pub fn plan(op: &Opciones, c: &Contexto<'_>) -> Result<Vec<Accion>, Informe> {
@@ -427,13 +417,7 @@ pub fn desinstalar(op: &Opciones, c: &Contexto<'_>) -> Hecho {
     if let Some(b) = acciones.iter().find(|a| a.bloquea).cloned() {
         // No se toca nada: lo que se habria borrado queda pendiente, y el
         // resumen no cuenta borrados que no hay.
-        for a in acciones
-            .iter_mut()
-            .filter(|a| a.decision == Decision::Borra)
-        {
-            a.decision = Decision::Pendiente;
-            a.motivo = Some("no-aplicada".to_owned());
-        }
+        plan_dato::rechazar(&mut acciones);
         let motivo = b.motivo.clone().unwrap_or_default();
         let i = base(Resultado::Rechazada, &acciones)
             .con("motivo", motivo.clone())
@@ -457,38 +441,17 @@ pub fn desinstalar(op: &Opciones, c: &Contexto<'_>) -> Hecho {
                 informe: i,
             };
         }
-        let mut fallo: Option<Informe> = None;
-        for a in acciones.iter_mut() {
-            if a.decision != Decision::Borra {
-                continue;
-            }
-            if fallo.is_some() {
-                a.decision = Decision::Pendiente;
-                a.motivo = Some("no-aplicada".to_owned());
-                continue;
-            }
-            if let Err(e) = aplicar(&a.efecto, c) {
-                a.decision = Decision::Pendiente;
-                a.motivo = Some("error".to_owned());
-                fallo = Some(
-                    Informe::nuevo(NOMBRE, Resultado::Error)
-                        .con("ruta", a.ruta.clone())
-                        .diagnostico(
-                            Mensaje::nuevo(&txt::FALLO_AL_APLICAR)
-                                .con("ruta", a.ruta.clone())
-                                .con("error", e),
-                        ),
+        if let Some((ruta, e)) =
+            plan_dato::aplicar_en_orden(&mut acciones, |a| aplicar(&a.efecto, c))
+        {
+            let informe = base(Resultado::Error, &acciones)
+                .con("ruta", ruta.clone())
+                .diagnostico(
+                    Mensaje::nuevo(&txt::FALLO_AL_APLICAR)
+                        .con("ruta", ruta)
+                        .con_mensaje("error", e),
                 );
-            }
-        }
-        if let Some(i) = fallo {
-            let mut f = base(Resultado::Error, &acciones);
-            f.claves.extend(i.claves);
-            f.diagnostico = i.diagnostico;
-            return Hecho {
-                acciones,
-                informe: f,
-            };
+            return Hecho { acciones, informe };
         }
     }
     // El que se nombra es el binario del grupo en uso, no su primera linea
@@ -516,10 +479,10 @@ pub fn desinstalar(op: &Opciones, c: &Contexto<'_>) -> Hecho {
 }
 
 /// Aplica un efecto. `Err` con el texto del fallo.
-fn aplicar(e: &Efecto, c: &Contexto<'_>) -> Result<(), String> {
+fn aplicar(e: &Efecto, c: &Contexto<'_>) -> Result<(), Mensaje> {
     let no_existe = |r: std::io::Result<()>| match r {
         Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
-        otro => otro.map_err(|e| e.to_string()),
+        otro => otro.map_err(externo),
     };
     match e {
         Efecto::Ninguno => Ok(()),
@@ -535,22 +498,15 @@ fn aplicar(e: &Efecto, c: &Contexto<'_>) -> Result<(), String> {
             ruta,
             antes,
             despues,
-        } => {
-            // Si la persona lo cambio despues del plan, no se pisa.
-            let ahora = std::fs::read(ruta).map_err(|e| e.to_string())?;
-            if &ahora != antes {
-                return Err("cambio desde el plan".to_owned());
-            }
-            escribir_atomico(ruta, despues).map_err(|e| e.to_string())
-        }
+        } => reescribir(ruta, antes, despues),
         Efecto::PathUsuario { viejo, nuevo } => c
             .maquina
             .escribir_path_usuario(viejo, nuevo)
-            .map_err(|e| e.to_string()),
+            .map_err(externo),
         Efecto::Autoborrado { ficheros, dirs } => c
             .maquina
             .programar_borrado(c.entorno.pid, ficheros, dirs)
-            .map_err(|e| e.to_string()),
+            .map_err(externo),
         Efecto::QuitarWorktree { principal, ruta } => {
             proyecto::quitar_worktree(c.git, principal, ruta)
         }
@@ -564,6 +520,16 @@ fn aplicar(e: &Efecto, c: &Contexto<'_>) -> Result<(), String> {
     }
 }
 
+/// Cambia `ruta` de `antes` a `despues`; si la persona lo cambio despues
+/// del plan, no se pisa.
+fn reescribir(ruta: &Path, antes: &[u8], despues: &[u8]) -> Result<(), Mensaje> {
+    let ahora = std::fs::read(ruta).map_err(externo)?;
+    if ahora != antes {
+        return Err(Mensaje::nuevo(&CAMBIO_DESDE_EL_PLAN).con("ruta", t(ruta)));
+    }
+    escribir_atomico(ruta, despues).map_err(externo)
+}
+
 /// Escribe `ruta` de una vez: un temporal en la misma carpeta y un
 /// renombrado, para que un corte no deje el perfil o el `settings.json` de
 /// la persona vacio. Si `ruta` es un enlace simbolico se escribe su destino
@@ -602,16 +568,9 @@ pub fn orden(args: &[String], json: bool, idioma: Idioma) -> Salida {
                 .salida_en(json, idioma);
         }
     };
-    let dir = match std::env::current_dir() {
+    let dir = match crate::tablero::carpeta_actual(NOMBRE) {
         Ok(d) => d,
-        Err(e) => {
-            return Informe::nuevo(NOMBRE, Resultado::Error)
-                .diagnostico(
-                    Mensaje::nuevo(&crate::i18n::catalogo::cli::CARPETA_ACTUAL)
-                        .con("error", e.to_string()),
-                )
-                .salida_en(json, idioma);
-        }
+        Err(i) => return i.salida_en(json, idioma),
     };
     let git = Git::nuevo();
     let maquina = MaquinaReal;
@@ -660,6 +619,37 @@ mod tests {
         }
     }
 
+    /// A7-07: un fichero que cambio desde el plan no se pisa, y el
+    /// diagnostico sale entero en el idioma pedido (con `en`, sin un trozo
+    /// en espanol en `{error}`).
+    #[test]
+    fn cambio_desde_el_plan_en_ingles() -> std::io::Result<()> {
+        let dir = std::env::temp_dir()
+            .join("tc2-tests")
+            .join(format!("desinstalar-reescribir-{}", std::process::id()));
+        std::fs::create_dir_all(&dir)?;
+        let ruta = dir.join(".profile");
+        std::fs::write(&ruta, "cambiado por la persona\n")?;
+        let r = reescribir(&ruta, b"original\n", b"");
+        let ahora = std::fs::read_to_string(&ruta);
+        let _ = std::fs::remove_dir_all(&dir);
+        assert_eq!(ahora?, "cambiado por la persona\n");
+        let e = r.err().map(|m| {
+            Mensaje::nuevo(&txt::FALLO_AL_APLICAR)
+                .con("ruta", t(&ruta))
+                .con_mensaje("error", m)
+                .texto(Idioma::En)
+        });
+        let r = t(&ruta);
+        assert_eq!(
+            e,
+            Some(format!(
+                "could not remove {r}: {r} changed after the plan was made. What the previous lines list is already removed: fix the cause and run tc2 desinstalar again"
+            ))
+        );
+        Ok(())
+    }
+
     #[test]
     fn linea_y_json_de_una_accion() {
         let a = Accion::nueva("C:/a b", "plugin", "claude-code", Decision::Persona)
diff --git a/crates/tc2/src/desinstalar/proyecto.rs b/crates/tc2/src/desinstalar/proyecto.rs
index 30dcf44..a68537b 100644
--- a/crates/tc2/src/desinstalar/proyecto.rs
+++ b/crates/tc2/src/desinstalar/proyecto.rs
@@ -7,7 +7,7 @@ use std::path::{Path, PathBuf};
 use std::time::Duration;
 
 use super::texto::hooks_con_tc2;
-use super::{Accion, Contexto, Decision, Efecto, NOMBRE, Opciones, leer, t};
+use super::{Accion, Contexto, Decision, Efecto, NOMBRE, Opciones, t};
 use crate::contrato::{Informe, Resultado};
 use crate::estado::Cerrojo;
 use crate::git::estado::dir_comun;
@@ -19,6 +19,8 @@ use crate::git::{
 use crate::huella::{self, RAMA_ESTADO};
 use crate::i18n::Mensaje;
 use crate::i18n::catalogo::desinstalar as txt;
+use crate::lectura::opcional;
+use crate::plan_dato::externo;
 
 /// Si la carpeta `dir` esta dentro de `ruta` (o es ella).
 fn dentro(dir: &Path, ruta: &Path) -> bool {
@@ -153,7 +155,7 @@ pub(super) fn plan(op: &Opciones, c: &Contexto<'_>, v: &mut Vec<Accion>) -> Resu
     }
     // Lineas de `info/exclude` (inversa de `git::rama::excluir`).
     let exclude = comun.join("info").join("exclude");
-    if let Some(bytes) = leer(&exclude)?
+    if let Some(bytes) = opcional(NOMBRE, &exclude)?
         && let Ok(texto) = String::from_utf8(bytes.clone())
     {
         let propias = huella::LINEAS_EXCLUDE;
@@ -261,7 +263,7 @@ pub(super) fn plan(op: &Opciones, c: &Contexto<'_>, v: &mut Vec<Accion>) -> Resu
     // queda siempre, y con ella `personas.yml`, compartido (F1-12 D4a, D6).
     let config = principal.join(crate::config::RUTA);
     let mut de_la_016 = false;
-    if let Some(bytes) = leer(&config)? {
+    if let Some(bytes) = opcional(NOMBRE, &config)? {
         let de_la_2 = match crate::config::leer(&String::from_utf8_lossy(&bytes)) {
             Ok(_) => true,
             Err(e) => matches!(
@@ -291,7 +293,7 @@ pub(super) fn plan(op: &Opciones, c: &Contexto<'_>, v: &mut Vec<Accion>) -> Resu
     // Lo versionado que deja `tc2 init` (F3-1): las lineas de `.gitignore` y
     // el workflow estan en la historia; los quita la persona con un PR.
     let gitignore = principal.join(".gitignore");
-    if let Some(bytes) = leer(&gitignore)? {
+    if let Some(bytes) = opcional(NOMBRE, &gitignore)? {
         let texto = String::from_utf8_lossy(&bytes);
         if huella::LINEAS_GITIGNORE
             .iter()
@@ -330,7 +332,7 @@ pub(super) fn plan(op: &Opciones, c: &Contexto<'_>, v: &mut Vec<Accion>) -> Resu
     // Hooks del proyecto que nombran tc2: aviso, nunca se editan (C4).
     for nombre in ["settings.json", "settings.local.json"] {
         let p = principal.join(".claude").join(nombre);
-        if let Some(bytes) = leer(&p)?
+        if let Some(bytes) = opcional(NOMBRE, &p)?
             && let Ok(valor) = serde_json::from_slice::<serde_json::Value>(&bytes)
         {
             for _ in hooks_con_tc2(&valor) {
@@ -369,26 +371,23 @@ pub(super) fn cerrojo_libre(c: &Contexto<'_>) -> Result<(), Informe> {
     }
 }
 
-/// Texto de un informe de error de git, para la linea de fallo.
-fn texto(i: &Informe) -> String {
-    i.diagnostico_en(crate::i18n::Idioma::Es)
-        .unwrap_or_else(|| i.linea())
-}
+// Los ayudantes de la aplicacion devuelven el mensaje del fallo para el
+// hueco `{error}` de `FALLO_AL_APLICAR` (A7-07): se traduce al escribir.
 
-pub(super) fn quitar_worktree(git: &Git, principal: &Path, ruta: &Path) -> Result<(), String> {
-    let r = ruta.to_str().ok_or_else(|| t(ruta))?;
+pub(super) fn quitar_worktree(git: &Git, principal: &Path, ruta: &Path) -> Result<(), Mensaje> {
+    let r = ruta.to_str().ok_or_else(|| externo(t(ruta)))?;
     exigir(git, principal, NOMBRE, Subcomando::Worktree, &["remove", r])
         .map(|_| ())
-        .map_err(|i| texto(&i))
+        .map_err(|i| i.mensaje())
 }
 
-pub(super) fn podar(git: &Git, principal: &Path) -> Result<(), String> {
+pub(super) fn podar(git: &Git, principal: &Path) -> Result<(), Mensaje> {
     exigir(git, principal, NOMBRE, Subcomando::Worktree, &["prune"])
         .map(|_| ())
-        .map_err(|i| texto(&i))
+        .map_err(|i| i.mensaje())
 }
 
-pub(super) fn borrar_ref(git: &Git, principal: &Path, r: &str, sha: &str) -> Result<(), String> {
+pub(super) fn borrar_ref(git: &Git, principal: &Path, r: &str, sha: &str) -> Result<(), Mensaje> {
     exigir(
         git,
         principal,
@@ -397,16 +396,16 @@ pub(super) fn borrar_ref(git: &Git, principal: &Path, r: &str, sha: &str) -> Res
         &["-d", r, sha],
     )
     .map(|_| ())
-    .map_err(|i| texto(&i))
+    .map_err(|i| i.mensaje())
 }
 
-pub(super) fn borrar_rama(git: &Git, principal: &Path, sha: &str) -> Result<(), String> {
+pub(super) fn borrar_rama(git: &Git, principal: &Path, sha: &str) -> Result<(), Mensaje> {
     let s = git
         .borrar_rama_de_estado(principal, sha)
-        .map_err(|e| e.to_string())?;
+        .map_err(|e| e.mensaje())?;
     if s.exito() {
         Ok(())
     } else {
-        Err(s.primera_linea_error())
+        Err(externo(s.primera_linea_error()))
     }
 }
diff --git a/crates/tc2/src/estado/lectura.rs b/crates/tc2/src/estado/lectura.rs
index 6d1094a..552b69f 100644
--- a/crates/tc2/src/estado/lectura.rs
+++ b/crates/tc2/src/estado/lectura.rs
@@ -29,11 +29,7 @@ pub trait Estado {
 /// Un fallo de lectura de git como [`Fallo::Error`]. Sin diagnostico queda
 /// la linea de contrato, que no se traduce (ADR F1-4).
 fn fallo_de_lectura(i: &Informe) -> Fallo {
-    Fallo::Error(
-        i.diagnostico
-            .clone()
-            .unwrap_or_else(|| crate::i18n::Mensaje::externo(i.linea())),
-    )
+    Fallo::Error(i.mensaje())
 }
 
 /// Vista de un commit de la rama de estado (o del estado vacio si la rama no
diff --git a/crates/tc2/src/i18n/catalogo/comun.rs b/crates/tc2/src/i18n/catalogo/comun.rs
index 73f2f68..043cd03 100644
--- a/crates/tc2/src/i18n/catalogo/comun.rs
+++ b/crates/tc2/src/i18n/catalogo/comun.rs
@@ -4,6 +4,18 @@ crate::i18n::textos! {
     /// Texto que viene de fuera de `tc2` (git, el sistema, HTTP) y se
     /// muestra tal cual.
     EXTERNO { es: "{texto}", en: "{texto}" }
+    /// Un fichero propio que existe y no se lee (`init`, `desinstalar`,
+    /// la configuracion: `crate::lectura::no_se_lee`).
+    NO_SE_LEE {
+        es: "no se puede leer {ruta}: {error}",
+        en: "cannot read {ruta}: {error}",
+    }
+    /// Plan como dato (`init`, `desinstalar`): el fichero cambio entre el
+    /// plan y su aplicacion, y no se pisa.
+    CAMBIO_DESDE_EL_PLAN {
+        es: "{ruta} cambio despues de hacer el plan",
+        en: "{ruta} changed after the plan was made",
+    }
     /// La tarea que se quiere cambiar no esta en la rama de estado
     /// (`start`, `descartar`, `pausa`, `reanudar`).
     TAREA_NO_EXISTE {
diff --git a/crates/tc2/src/i18n/catalogo/desinstalar.rs b/crates/tc2/src/i18n/catalogo/desinstalar.rs
index 9d3ee56..75084cc 100644
--- a/crates/tc2/src/i18n/catalogo/desinstalar.rs
+++ b/crates/tc2/src/i18n/catalogo/desinstalar.rs
@@ -37,10 +37,6 @@ crate::i18n::textos! {
         es: "no se pudo quitar {ruta}: {error}. Lo de las lineas anteriores ya se quito: arregla la causa y repite tc2 desinstalar",
         en: "could not remove {ruta}: {error}. What the previous lines list is already removed: fix the cause and run tc2 desinstalar again",
     }
-    NO_SE_LEE {
-        es: "no se puede leer {ruta}: {error}",
-        en: "cannot read {ruta}: {error}",
-    }
     SIN_ENTORNO {
         es: "no se puede saber donde esta este tc2: {error}",
         en: "cannot tell where this tc2 is: {error}",
diff --git a/crates/tc2/src/i18n/catalogo/init.rs b/crates/tc2/src/i18n/catalogo/init.rs
index 64f9f92..401a5bd 100644
--- a/crates/tc2/src/i18n/catalogo/init.rs
+++ b/crates/tc2/src/i18n/catalogo/init.rs
@@ -53,13 +53,11 @@ crate::i18n::textos! {
         es: "no se pudo crear {ruta}: {error}. Lo de las lineas anteriores ya esta hecho: arregla la causa y repite tc2 init",
         en: "could not create {ruta}: {error}. What the previous lines list is already done: fix the cause and run tc2 init again",
     }
-    NO_SE_LEE {
-        es: "no se puede leer {ruta}: {error}",
-        en: "cannot read {ruta}: {error}",
-    }
-    CAMBIO_DESDE_EL_PLAN {
-        es: "{ruta} cambio despues de hacer el plan",
-        en: "{ruta} changed after the plan was made",
+    /// Error interno: el efecto de las puertas se aplica con la plataforma,
+    /// no por el camino de los ficheros.
+    EFECTO_PUERTAS_SIN_PLATAFORMA {
+        es: "aplicar-puertas sin plataforma",
+        en: "aplicar-puertas without a platform",
     }
     AVISO_SIN_RELACION {
         es: "aviso: la historia de este clon no tiene nada en comun con la de origin; init no propone publicar ninguna rama de codigo (la rama de estado {rama}, huerfana, si se publica): decide tu si este clon es de ese remoto",
diff --git a/crates/tc2/src/i18n/guarda.rs b/crates/tc2/src/i18n/guarda.rs
index 7fb0cb5..bb97766 100644
--- a/crates/tc2/src/i18n/guarda.rs
+++ b/crates/tc2/src/i18n/guarda.rs
@@ -5,13 +5,17 @@
 use std::collections::BTreeMap;
 use std::path::{Path, PathBuf};
 
-/// Usos permitidos de `Mensaje::externo(` por fichero: texto que viene de
+/// Usos permitidos por fichero de `Mensaje::externo(` y de la entrada
+/// `EXTERNO` del catalogo (la que hay debajo, A7-10): texto que viene de
 /// fuera (git, el sistema, HTTP) sin nada que traducir alrededor. Lista
 /// cerrada: un uso nuevo tiene que anadirse aqui a proposito.
 const EXTERNOS: &[(&str, usize)] = &[
-    // Fallo al leer un blob sin diagnostico: queda la linea de contrato, que
-    // no se traduce (ADR F1-4).
-    ("estado/lectura.rs", 1),
+    // `Informe::mensaje`: un informe sin diagnostico deja la linea de
+    // contrato, que no se traduce (ADR F1-4).
+    ("contrato.rs", 1),
+    // `plan_dato::externo`: el error del sistema o de git en el hueco
+    // `{error}` de `FALLO_AL_APLICAR` (A7-07).
+    ("plan_dato.rs", 1),
 ];
 
 /// Formas de meter un literal en un diagnostico sin pasar por el catalogo.
@@ -22,6 +26,30 @@ const PROHIBIDOS: &[&str] = &[
     "Mensaje::externo(format!",
 ];
 
+/// `Err("...")` con prosa (un literal con espacios) permitidos por fichero:
+/// errores internos que no llegan a la persona como diagnostico propio.
+/// Un `Err` con prosa acaba en el hueco `{error}` de otro mensaje y sale en
+/// espanol con cualquier idioma (A7-07). Lista cerrada.
+const LITERALES_ERR: &[(&str, usize)] = &[
+    // Comprobacion del binario descargado: el texto va a la linea de fallo
+    // de `actualizar --cli` (fuera de A7, decision 2 de TASK-145).
+    ("actualizar_cli.rs", 1),
+    // Validacion del informe de verificacion (fuera de A7, decision 2 de
+    // TASK-145).
+    ("verificacion/informe.rs", 1),
+];
+
+/// Cuantos `Err("...")` de `codigo` llevan un literal con espacios.
+fn literales_err(codigo: &str) -> usize {
+    codigo
+        .match_indices("Err(\"")
+        .filter(|(i, m)| {
+            let resto = &codigo[i + m.len()..];
+            resto.split('"').next().is_some_and(|l| l.contains(' '))
+        })
+        .count()
+}
+
 fn fuentes(dir: &Path, base: &Path, v: &mut Vec<(String, String)>) {
     let Ok(entradas) = std::fs::read_dir(dir) else {
         return;
@@ -62,21 +90,32 @@ fn ningun_literal_fuera_del_catalogo() {
     );
     let mut fallos = Vec::new();
     let mut externos: BTreeMap<&str, usize> = BTreeMap::new();
+    let mut literales: BTreeMap<&str, usize> = BTreeMap::new();
     for (rel, codigo) in &v {
         for p in PROHIBIDOS {
             if codigo.contains(p) {
                 fallos.push(format!("{rel}: usa `{p}`: pasa el texto al catalogo"));
             }
         }
-        let n = codigo.matches("Mensaje::externo(").count();
+        let n = codigo.matches("Mensaje::externo(").count() + codigo.matches("EXTERNO").count();
         if n > 0 {
             externos.insert(rel.as_str(), n);
         }
+        let n = literales_err(codigo);
+        if n > 0 {
+            literales.insert(rel.as_str(), n);
+        }
     }
     let permitidos: BTreeMap<&str, usize> = EXTERNOS.iter().copied().collect();
     if externos != permitidos {
         fallos.push(format!(
-            "usos de Mensaje::externo distintos de EXTERNOS: hay {externos:?}, se permiten {permitidos:?}"
+            "usos de Mensaje::externo o EXTERNO distintos de EXTERNOS: hay {externos:?}, se permiten {permitidos:?}"
+        ));
+    }
+    let permitidos: BTreeMap<&str, usize> = LITERALES_ERR.iter().copied().collect();
+    if literales != permitidos {
+        fallos.push(format!(
+            "Err(\"...\") con prosa distintos de LITERALES_ERR: hay {literales:?}, se permiten {permitidos:?}: pasa el texto al catalogo"
         ));
     }
     assert!(fallos.is_empty(), "{}", fallos.join("\n"));
diff --git a/crates/tc2/src/init/aplicar.rs b/crates/tc2/src/init/aplicar.rs
index c0ef56e..a427375 100644
--- a/crates/tc2/src/init/aplicar.rs
+++ b/crates/tc2/src/init/aplicar.rs
@@ -13,28 +13,25 @@ use crate::git::estado::construir_commit;
 use crate::git::remoto::publicar;
 use crate::git::{Cambio, Git, Subcomando, exigir};
 use crate::huella::LINEAS_EXCLUDE;
-use crate::i18n::Idioma;
 use crate::i18n::Mensaje;
+use crate::i18n::catalogo::comun::CAMBIO_DESDE_EL_PLAN;
 use crate::i18n::catalogo::init as txt;
+use crate::plan_dato::externo;
 use crate::plataforma::puertas::{APLICAR_PUERTAS, Origenes, modelo};
 
 /// Mensaje del commit raiz de `main` y `develop`.
 const MENSAJE_RAIZ: &str = "chore: inicia TaskCodeTwo (tc2 init)";
 
-/// Texto de un informe para la linea de fallo.
-fn texto(i: &Informe) -> String {
-    i.diagnostico_en(Idioma::Es).unwrap_or_else(|| i.linea())
-}
-
-/// Aplica un efecto. `Err` con el texto del fallo.
-pub(super) fn aplicar(e: &Efecto, c: &Contexto<'_>) -> Result<(), String> {
+/// Aplica un efecto. `Err` con el mensaje del fallo, para el hueco
+/// `{error}` de `FALLO_AL_APLICAR` (se traduce al escribir la salida).
+pub(super) fn aplicar(e: &Efecto, c: &Contexto<'_>) -> Result<(), Mensaje> {
     match e {
         Efecto::Ninguno => Ok(()),
         Efecto::Escribir { ruta, contenido } => escribir_nuevo(ruta, contenido),
         Efecto::Anadir { ruta, lineas } => anadir(ruta, lineas),
         Efecto::Excluir { principal } => {
             for l in LINEAS_EXCLUDE {
-                crate::git::rama::excluir(c.git, principal, NOMBRE, l).map_err(|i| texto(&i))?;
+                crate::git::rama::excluir(c.git, principal, NOMBRE, l).map_err(|i| i.mensaje())?;
             }
             Ok(())
         }
@@ -44,17 +41,19 @@ pub(super) fn aplicar(e: &Efecto, c: &Contexto<'_>) -> Result<(), String> {
             ramas,
             head,
             publicar,
-        } => ramas_raiz(c.git, principal, ficheros, ramas, *head, *publicar).map_err(|i| texto(&i)),
+        } => {
+            ramas_raiz(c.git, principal, ficheros, ramas, *head, *publicar).map_err(|i| i.mensaje())
+        }
         Efecto::MontarEstado { principal } => {
             let i = crate::estado::montar(c.git, principal);
             if i.resultado.es_exito() {
                 Ok(())
             } else {
-                Err(texto(&i))
+                Err(i.mensaje())
             }
         }
         // Necesita la plataforma: lo aplica [`puertas`] desde `init`.
-        Efecto::AplicarPuertas { .. } => Err("aplicar-puertas sin plataforma".to_owned()),
+        Efecto::AplicarPuertas { .. } => Err(Mensaje::nuevo(&txt::EFECTO_PUERTAS_SIN_PLATAFORMA)),
     }
 }
 
@@ -69,9 +68,9 @@ pub(super) fn puertas(conexion: &Conexion, origenes: &Origenes) -> Informe {
 
 /// Crea `ruta` con `contenido`; si ya existe (la creo alguien despues del
 /// plan), falla sin tocarla.
-fn escribir_nuevo(ruta: &Path, contenido: &str) -> Result<(), String> {
+fn escribir_nuevo(ruta: &Path, contenido: &str) -> Result<(), Mensaje> {
     if let Some(padre) = ruta.parent() {
-        std::fs::create_dir_all(padre).map_err(|e| e.to_string())?;
+        std::fs::create_dir_all(padre).map_err(externo)?;
     }
     let mut f = std::fs::OpenOptions::new()
         .write(true)
@@ -79,23 +78,21 @@ fn escribir_nuevo(ruta: &Path, contenido: &str) -> Result<(), String> {
         .open(ruta)
         .map_err(|e| {
             if e.kind() == std::io::ErrorKind::AlreadyExists {
-                Mensaje::nuevo(&txt::CAMBIO_DESDE_EL_PLAN)
-                    .con("ruta", ruta.display().to_string())
-                    .texto(Idioma::Es)
+                Mensaje::nuevo(&CAMBIO_DESDE_EL_PLAN).con("ruta", ruta.display().to_string())
             } else {
-                e.to_string()
+                externo(e)
             }
         })?;
-    f.write_all(contenido.as_bytes()).map_err(|e| e.to_string())
+    f.write_all(contenido.as_bytes()).map_err(externo)
 }
 
 /// Anade a `ruta` las `lineas` que le falten, sin tocar las demas (y con
 /// el fin de linea que ya use el fichero).
-fn anadir(ruta: &Path, lineas: &[String]) -> Result<(), String> {
+fn anadir(ruta: &Path, lineas: &[String]) -> Result<(), Mensaje> {
     let actual = match std::fs::read_to_string(ruta) {
         Ok(t) => t,
         Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
-        Err(e) => return Err(e.to_string()),
+        Err(e) => return Err(externo(e)),
     };
     let fin = if actual.contains("\r\n") {
         "\r\n"
@@ -116,7 +113,7 @@ fn anadir(ruta: &Path, lineas: &[String]) -> Result<(), String> {
     if nuevo == actual {
         return Ok(());
     }
-    std::fs::write(ruta, nuevo).map_err(|e| e.to_string())
+    std::fs::write(ruta, nuevo).map_err(externo)
 }
 
 /// Commit raiz con `ficheros` (tal como estan en el disco) y `ramas` en el.
@@ -134,7 +131,8 @@ fn ramas_raiz(
     let mut cambios = Vec::with_capacity(ficheros.len());
     for f in ficheros {
         let ruta = principal.join(f);
-        let bytes = std::fs::read(&ruta).map_err(|e| super::no_se_lee(&ruta, &e))?;
+        let bytes =
+            std::fs::read(&ruta).map_err(|e| crate::lectura::no_se_lee(NOMBRE, &ruta, &e))?;
         cambios.push(Cambio::nuevo(
             f.as_str(),
             String::from_utf8_lossy(&bytes).into_owned(),
@@ -170,3 +168,63 @@ fn ramas_raiz(
     }
     Ok(())
 }
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::i18n::Idioma;
+
+    /// A7-07: un fichero que alguien creo despues del plan no se pisa, y el
+    /// diagnostico sale entero en ingles con `en`.
+    #[test]
+    fn escribir_nuevo_sobre_uno_existente_en_ingles() -> std::io::Result<()> {
+        let dir = std::env::temp_dir()
+            .join("tc2-tests")
+            .join(format!("init-escribir-{}", std::process::id()));
+        std::fs::create_dir_all(&dir)?;
+        let ruta = dir.join("config.yml");
+        std::fs::write(&ruta, "de otra persona\n")?;
+        let r = escribir_nuevo(&ruta, "de tc2\n");
+        let ahora = std::fs::read_to_string(&ruta);
+        let _ = std::fs::remove_dir_all(&dir);
+        assert_eq!(ahora?, "de otra persona\n");
+        let r_txt = ruta.display().to_string();
+        let e = r.err().map(|m| {
+            Mensaje::nuevo(&txt::FALLO_AL_APLICAR)
+                .con("ruta", r_txt.clone())
+                .con_mensaje("error", m)
+                .texto(Idioma::En)
+        });
+        assert_eq!(
+            e,
+            Some(format!(
+                "could not create {r_txt}: {r_txt} changed after the plan was made. What the previous lines list is already done: fix the cause and run tc2 init again"
+            ))
+        );
+        Ok(())
+    }
+
+    /// A7-07: el informe de un paso que falla entra en `{error}` como
+    /// mensaje (se traduce); sin diagnostico, la linea de contrato tal cual.
+    #[test]
+    fn informe_como_mensaje() {
+        let con = Informe::nuevo("estado", Resultado::Error)
+            .diagnostico(Mensaje::nuevo(&txt::EFECTO_PUERTAS_SIN_PLATAFORMA));
+        let sin = Informe::nuevo("estado", Resultado::Error).con("motivo", "x");
+        assert_eq!(
+            con.mensaje().texto(Idioma::En),
+            "aplicar-puertas without a platform"
+        );
+        assert_eq!(sin.mensaje().texto(Idioma::En), sin.linea());
+        let fallo = Mensaje::nuevo(&txt::FALLO_AL_APLICAR)
+            .con("ruta", "taskcodetwo-main")
+            .con_mensaje("error", con.mensaje())
+            .texto(Idioma::En);
+        assert!(
+            fallo.starts_with(
+                "could not create taskcodetwo-main: aplicar-puertas without a platform. "
+            ),
+            "{fallo}"
+        );
+    }
+}
diff --git a/crates/tc2/src/init/inventario.rs b/crates/tc2/src/init/inventario.rs
index 00b04be..26b5064 100644
--- a/crates/tc2/src/init/inventario.rs
+++ b/crates/tc2/src/init/inventario.rs
@@ -3,7 +3,7 @@
 
 use std::path::Path;
 
-use super::{ARREGLO_016, Accion, Contexto, Decision, Efecto, NOMBRE, ORDEN_PUERTAS, leer, t};
+use super::{ARREGLO_016, Accion, Contexto, Decision, Efecto, NOMBRE, ORDEN_PUERTAS, t};
 use crate::config::{self, Motivo};
 use crate::contrato::{Informe, Resultado};
 use crate::estado::{self, CARPETA, Previsto};
@@ -16,6 +16,7 @@ use crate::git::{
 use crate::huella::{LINEAS_EXCLUDE, LINEAS_GITIGNORE, RUTA_WORKFLOW};
 use crate::i18n::Mensaje;
 use crate::i18n::catalogo::init as txt;
+use crate::lectura::opcional;
 use crate::plataforma::personas::{self, Personas};
 use crate::plataforma::puertas::{modelo, ruleset_github};
 use crate::plataforma::{EstadoPuerta, GitHub, GitLab, Plataforma, Proveedor, Rol, detectar};
@@ -82,7 +83,7 @@ pub(super) fn plan(c: &Contexto<'_>, aplicar: bool) -> Result<(Vec<Accion>, Cone
     let mut clave: Option<(&'static str, String)> = None;
     // `plataforma.revisor_id` (TASK-118): el origen de `revision-ia`.
     let mut revisor: Option<u64> = None;
-    match leer(&ruta_config)? {
+    match opcional(NOMBRE, &ruta_config)? {
         None => v.push(Accion::crea(
             t(&ruta_config),
             "config",
@@ -120,7 +121,7 @@ pub(super) fn plan(c: &Contexto<'_>, aplicar: bool) -> Result<(Vec<Accion>, Cone
 
     // `.gitignore`: solo se anaden lineas.
     let gitignore = principal.join(".gitignore");
-    let texto = leer(&gitignore)?
+    let texto = opcional(NOMBRE, &gitignore)?
         .map(|b| String::from_utf8_lossy(&b).into_owned())
         .unwrap_or_default();
     let lineas = faltan(&texto, &LINEAS_GITIGNORE);
@@ -161,7 +162,7 @@ pub(super) fn plan(c: &Contexto<'_>, aplicar: bool) -> Result<(Vec<Accion>, Cone
     // `info/exclude`.
     let comun = dir_comun(g, &principal, NOMBRE)?;
     let exclude = comun.join("info").join("exclude");
-    let texto = leer(&exclude)?
+    let texto = opcional(NOMBRE, &exclude)?
         .map(|b| String::from_utf8_lossy(&b).into_owned())
         .unwrap_or_default();
     v.push(if faltan(&texto, &LINEAS_EXCLUDE).is_empty() {
@@ -767,7 +768,7 @@ fn plataforma(
     // La lista de personas, si ya existe y vale: `aplicar-puertas` exige
     // que la identidad de persona este en ella (TASK-118). Sin ella, vacia
     // (las lecturas de `init` no la miran).
-    let lista = leer(&principal.join(personas::RUTA))?
+    let lista = opcional(NOMBRE, &principal.join(personas::RUTA))?
         .and_then(|b| Personas::leer(&String::from_utf8_lossy(&b)).ok())
         .unwrap_or_default();
     let p: Result<Box<dyn Plataforma>, Informe> = match destino.proveedor {
@@ -803,7 +804,7 @@ fn motivo_de(i: &Informe) -> String {
 fn personas_yml(principal: &Path, plataforma: &Conexion) -> Result<Accion, Informe> {
     let ruta = principal.join(personas::RUTA);
     let nueva = |d: Decision| Accion::nueva(t(&ruta), "personas", "taskcodetwo", d);
-    if let Some(bytes) = leer(&ruta)? {
+    if let Some(bytes) = opcional(NOMBRE, &ruta)? {
         return Ok(match Personas::leer(&String::from_utf8_lossy(&bytes)) {
             Ok(_) => nueva(Decision::Existe),
             Err(_) => {
diff --git a/crates/tc2/src/init/mod.rs b/crates/tc2/src/init/mod.rs
index f97faa8..25499ae 100644
--- a/crates/tc2/src/init/mod.rs
+++ b/crates/tc2/src/init/mod.rs
@@ -40,6 +40,7 @@ use crate::contrato::{Informe, LineaAccion, Resultado, Valor};
 use crate::git::Git;
 use crate::i18n::catalogo::init as txt;
 use crate::i18n::{Idioma, Mensaje};
+use crate::plan_dato::{self, Paso, t};
 use crate::plataforma::Credenciales;
 use crate::plataforma::puertas::Origenes;
 
@@ -265,6 +266,19 @@ impl Accion {
     }
 }
 
+impl Paso for Accion {
+    fn aplicable(&self) -> bool {
+        self.decision == Decision::Crea
+    }
+    fn aplazar(&mut self, motivo: &str) {
+        self.decision = Decision::Pendiente;
+        self.motivo = Some(motivo.to_owned());
+    }
+    fn ruta(&self) -> &str {
+        &self.ruta
+    }
+}
+
 /// Lo que necesita `init`, inyectable en los tests.
 pub struct Contexto<'a> {
     /// Carpeta del proyecto.
@@ -343,31 +357,6 @@ impl Hecho {
     }
 }
 
-/// Ruta como texto para la linea.
-pub(crate) fn t(p: &Path) -> String {
-    p.display().to_string()
-}
-
-/// Informe de error al leer `ruta`.
-pub(crate) fn no_se_lee(ruta: &Path, e: &std::io::Error) -> Informe {
-    Informe::nuevo(NOMBRE, Resultado::Error)
-        .con("ruta", t(ruta))
-        .diagnostico(
-            Mensaje::nuevo(&txt::NO_SE_LEE)
-                .con("ruta", t(ruta))
-                .con("error", e.to_string()),
-        )
-}
-
-/// Lee un fichero; `None` si no existe.
-pub(crate) fn leer(p: &Path) -> Result<Option<Vec<u8>>, Informe> {
-    match std::fs::read(p) {
-        Ok(b) => Ok(Some(b)),
-        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
-        Err(e) => Err(no_se_lee(p, &e)),
-    }
-}
-
 /// El plan, sin aplicar nada (sin `--aplicar-puertas`).
 pub fn plan(c: &Contexto<'_>) -> Result<Vec<Accion>, Informe> {
     inventario::plan(c, false).map(|(v, _)| v)
@@ -426,10 +415,7 @@ pub fn init(op: Opciones, c: &Contexto<'_>) -> Hecho {
         .cloned()
     {
         // No se toca nada: lo que se habria creado queda pendiente.
-        for a in acciones.iter_mut().filter(|a| a.decision == Decision::Crea) {
-            a.decision = Decision::Pendiente;
-            a.motivo = Some("no-aplicada".to_owned());
-        }
+        plan_dato::rechazar(&mut acciones);
         let motivo = b.motivo.clone().unwrap_or_default();
         let mut cabeza = vec![("motivo", texto(&motivo))];
         if let Some(o) = &b.orden {
@@ -448,38 +434,22 @@ pub fn init(op: Opciones, c: &Contexto<'_>) -> Hecho {
         return Hecho { acciones, informe };
     }
     if !op.simular {
-        // (ruta, error) del primer fallo; lo que venia detras no se aplica.
-        let mut fallo: Option<(String, String)> = None;
         // El informe de `aplicar-puertas`, si se lanzo.
         let mut puertas: Option<Informe> = None;
-        for a in acciones.iter_mut() {
-            if a.decision != Decision::Crea {
-                continue;
-            }
-            if fallo.is_some() {
-                a.decision = Decision::Pendiente;
-                a.motivo = Some("no-aplicada".to_owned());
-                continue;
-            }
-            let r = match &a.efecto {
-                Efecto::AplicarPuertas { origenes, .. } => {
-                    let i = aplicar::puertas(&conexion, origenes);
-                    let r = if i.resultado.es_exito() {
-                        Ok(())
-                    } else {
-                        Err(i.diagnostico_en(Idioma::Es).unwrap_or_else(|| i.linea()))
-                    };
-                    puertas = Some(i);
-                    r
-                }
-                e => aplicar::aplicar(e, c),
-            };
-            if let Err(e) = r {
-                a.decision = Decision::Pendiente;
-                a.motivo = Some("error".to_owned());
-                fallo = Some((a.ruta.clone(), e));
+        // (ruta, error) del primer fallo; lo que venia detras no se aplica.
+        let mut fallo = plan_dato::aplicar_en_orden(&mut acciones, |a| match &a.efecto {
+            Efecto::AplicarPuertas { origenes, .. } => {
+                let i = aplicar::puertas(&conexion, origenes);
+                let r = if i.resultado.es_exito() {
+                    Ok(())
+                } else {
+                    Err(i.mensaje())
+                };
+                puertas = Some(i);
+                r
             }
-        }
+            e => aplicar::aplicar(e, c),
+        });
         let mut cabeza = Vec::new();
         if let Some(i) = &puertas {
             // Cada linea `ruleset` dice lo que de verdad paso: creada,
@@ -529,7 +499,7 @@ pub fn init(op: Opciones, c: &Contexto<'_>) -> Hecho {
             let informe = cuenta(Resultado::Error, &acciones, &cabeza).diagnostico(
                 Mensaje::nuevo(&txt::FALLO_AL_APLICAR)
                     .con("ruta", ruta)
-                    .con("error", e),
+                    .con_mensaje("error", e),
             );
             return Hecho { acciones, informe };
         }
@@ -550,16 +520,9 @@ pub fn orden(args: &[String], json: bool, idioma: Idioma) -> Salida {
         Ok(o) => o,
         Err(i) => return i.salida_en(json, idioma),
     };
-    let dir = match std::env::current_dir() {
+    let dir = match crate::tablero::carpeta_actual(NOMBRE) {
         Ok(d) => d,
-        Err(e) => {
-            return Informe::nuevo(NOMBRE, Resultado::Error)
-                .diagnostico(
-                    Mensaje::nuevo(&crate::i18n::catalogo::cli::CARPETA_ACTUAL)
-                        .con("error", e.to_string()),
-                )
-                .salida_en(json, idioma);
-        }
+        Err(i) => return i.salida_en(json, idioma),
     };
     let git = Git::nuevo();
     let credenciales = || -> Box<dyn Credenciales> { crate::plataforma::de_entorno() };
diff --git a/crates/tc2/src/lectura.rs b/crates/tc2/src/lectura.rs
index ef2085e..eb5f5bc 100644
--- a/crates/tc2/src/lectura.rs
+++ b/crates/tc2/src/lectura.rs
@@ -5,12 +5,42 @@
 //! El fichero lo pasa quien lanza la orden: es un dato no confiable (F1-8),
 //! asi que la ruta y el error del sistema salen ya saneados
 //! ([`sanear`]: controles a espacio, 200 caracteres como mucho).
+//!
+//! Tambien la lectura de un fichero propio del proyecto o de la maquina
+//! (la de `init`, `desinstalar` y la configuracion, A7-01 y A7-09):
+//! [`opcional`] distingue «no existe» de «no se lee», y [`no_se_lee`] da el
+//! informe `error` de lo segundo a nombre de la orden. Esas rutas las
+//! calcula `tc2`, no las pasa la persona: no se sanean.
 
 use std::io::Read as _;
 use std::path::Path;
 
+use crate::contrato::{Informe, Resultado};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::comun::NO_SE_LEE;
 use crate::plataforma::errores::sanear;
 
+/// Informe `{op} error ruta=...` de un fichero que existe y no se lee.
+pub(crate) fn no_se_lee(op: &str, ruta: &Path, e: &std::io::Error) -> Informe {
+    let r = ruta.display().to_string();
+    Informe::nuevo(op, Resultado::Error)
+        .con("ruta", r.clone())
+        .diagnostico(
+            Mensaje::nuevo(&NO_SE_LEE)
+                .con("ruta", r)
+                .con("error", e.to_string()),
+        )
+}
+
+/// Lee un fichero; `None` si no existe y [`no_se_lee`] si no se lee.
+pub(crate) fn opcional(op: &str, ruta: &Path) -> Result<Option<Vec<u8>>, Informe> {
+    match std::fs::read(ruta) {
+        Ok(b) => Ok(Some(b)),
+        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
+        Err(e) => Err(no_se_lee(op, ruta, &e)),
+    }
+}
+
 /// Tope de un fichero externo (D4 de TASK-029).
 pub const MAX_BYTES: usize = 1 << 20;
 
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index ce02215..05150ab 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -50,6 +50,7 @@ pub mod instalacion;
 pub mod inventario;
 mod lectura;
 pub mod plan;
+mod plan_dato;
 pub mod plataforma;
 pub mod progreso;
 pub mod recursos;
diff --git a/crates/tc2/src/plan_dato.rs b/crates/tc2/src/plan_dato.rs
new file mode 100644
index 0000000..f2da919
--- /dev/null
+++ b/crates/tc2/src/plan_dato.rs
@@ -0,0 +1,137 @@
+//! Esqueleto «plan como dato» (F1-12 PL1) comun a `tc2 init` y `tc2
+//! desinstalar` (A7-01): un inventario de solo lectura calcula el plan y la
+//! ejecucion aplica ese mismo plan. Aqui va lo que no depende de la orden:
+//! el rechazo previo, que no aplica nada, y el bucle que aplica en orden y
+//! corta en el primer fallo. Cada orden pone su [`Paso`] (que se aplica y
+//! como se aplaza) y su efecto.
+
+use std::path::Path;
+
+use crate::i18n::Mensaje;
+
+/// Una linea del plan, vista desde el bucle de aplicacion.
+pub(crate) trait Paso {
+    /// Se aplica al ejecutar el plan (`crea` en init, `borra` en
+    /// desinstalar).
+    fn aplicable(&self) -> bool;
+    /// Queda `pendiente` con ese motivo.
+    fn aplazar(&mut self, motivo: &str);
+    /// Ruta (o ref, o nombre) de la linea.
+    fn ruta(&self) -> &str;
+}
+
+/// El plan no se aplica: lo que se habria hecho queda `pendiente
+/// motivo=no-aplicada`, y el resumen no cuenta lo que no se hizo.
+pub(crate) fn rechazar<P: Paso>(acciones: &mut [P]) {
+    for a in acciones.iter_mut().filter(|a| a.aplicable()) {
+        a.aplazar("no-aplicada");
+    }
+}
+
+/// Aplica en orden las lineas aplicables con `aplicar`. En el primer fallo
+/// esa linea queda `pendiente motivo=error` y las de detras `pendiente
+/// motivo=no-aplicada`, sin intentarlas. Devuelve la ruta y el error del
+/// fallo, si lo hubo.
+pub(crate) fn aplicar_en_orden<P: Paso, E>(
+    acciones: &mut [P],
+    mut aplicar: impl FnMut(&P) -> Result<(), E>,
+) -> Option<(String, E)> {
+    let mut fallo = None;
+    for a in acciones.iter_mut().filter(|a| a.aplicable()) {
+        if fallo.is_some() {
+            a.aplazar("no-aplicada");
+            continue;
+        }
+        if let Err(e) = aplicar(a) {
+            a.aplazar("error");
+            fallo = Some((a.ruta().to_owned(), e));
+        }
+    }
+    fallo
+}
+
+/// Error de fuera de `tc2` (el sistema de ficheros, git) para el hueco
+/// `{error}` de `FALLO_AL_APLICAR`: se muestra tal cual, sin nada que
+/// traducir alrededor (A7-07).
+pub(crate) fn externo(e: impl std::fmt::Display) -> Mensaje {
+    Mensaje::externo(e.to_string())
+}
+
+/// Ruta como texto para la linea.
+pub(crate) fn t(p: &Path) -> String {
+    p.display().to_string()
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    /// Linea de prueba: (ruta, aplicable, motivo).
+    #[derive(Debug, PartialEq)]
+    struct L(&'static str, bool, Option<String>);
+
+    impl Paso for L {
+        fn aplicable(&self) -> bool {
+            self.1
+        }
+        fn aplazar(&mut self, motivo: &str) {
+            self.1 = false;
+            self.2 = Some(motivo.to_owned());
+        }
+        fn ruta(&self) -> &str {
+            self.0
+        }
+    }
+
+    fn plan() -> Vec<L> {
+        vec![
+            L("a", true, None),
+            L("b", false, None),
+            L("c", true, None),
+            L("d", true, None),
+        ]
+    }
+
+    /// Tras el primer fallo, la que falla queda `error` y lo de detras
+    /// `no-aplicada` sin intentarse; lo no aplicable no se toca.
+    #[test]
+    fn corta_en_el_primer_fallo() {
+        let mut v = plan();
+        let mut intentadas = Vec::new();
+        let fallo = aplicar_en_orden(&mut v, |l| {
+            intentadas.push(l.0);
+            if l.0 == "c" { Err("roto") } else { Ok(()) }
+        });
+        assert_eq!(fallo, Some(("c".to_owned(), "roto")));
+        assert_eq!(intentadas, ["a", "c"]);
+        assert_eq!(
+            v,
+            [
+                L("a", true, None),
+                L("b", false, None),
+                L("c", false, Some("error".to_owned())),
+                L("d", false, Some("no-aplicada".to_owned())),
+            ]
+        );
+        let mut v = plan();
+        assert_eq!(aplicar_en_orden(&mut v, |_| Ok::<(), ()>(())), None);
+        assert_eq!(v, plan());
+    }
+
+    /// `rechazar` aplaza solo las aplicables, sin aplicar nada.
+    #[test]
+    fn rechazar_aplaza_las_aplicables() {
+        let mut v = plan();
+        rechazar(&mut v);
+        let no = || Some("no-aplicada".to_owned());
+        assert_eq!(
+            v,
+            [
+                L("a", false, no()),
+                L("b", false, None),
+                L("c", false, no()),
+                L("d", false, no()),
+            ]
+        );
+    }
+}
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/01-en-diseno/TASK-145/tarea.md                                                     | 49 -------------------------------------------------
 tareas/{01-en-diseno => 02-en-curso}/TASK-145/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-145/planificacion/plan-final.md                 |  0
 tareas/02-en-curso/TASK-145/tarea.md                                                      | 83 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 4 files changed, 83 insertions(+), 49 deletions(-)
````

## Tests del diff (2 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/init.rs" ":(literal)crates/tc2/tests/verificar.rs"
````

````
crates/tc2/tests/init.rs      | 23 ++++++++++++++++++++---
 crates/tc2/tests/verificar.rs | 49 +++++++++++++++++++++++++++++++++++++++++++++++++
 2 files changed, 69 insertions(+), 3 deletions(-)
````
## Candidatos 4D

Pistas generadas desde el diff, no hallazgos: comprueba cada una (Regla 4D de `revision.md`).

### Posible duplicacion con lo que ya existia

Definiciones nuevas con nombre igual o parecido a otra de la rama base. Mira si hacen lo mismo con pequenas diferencias (duplicacion real = IMPORTANTE).

- `Repo` (crates/tc2/src/ciclo/mod.rs:471) ~ `Repo` (crates/tc2/src/confianza/mod.rs:87), `repo` (crates/tc2/src/doctor/entorno.rs:83), `repo` (crates/tc2/src/git/estado.rs:505) y 3 mas
- `repo_con_remoto` (crates/tc2/src/ciclo/mod.rs:481) ~ `repo_sin_remoto` (crates/tc2/tests/comun/mod.rs:97)
- `cuenta` (crates/tc2/src/ciclo/prueba.rs:15) ~ `cuenta` (crates/tc2/src/ciclo/liberar.rs:399), `Cuenta` (crates/tc2/src/identidad.rs:54), `cuenta` (crates/tc2/src/identidad.rs:74) y 5 mas
- `ana` (crates/tc2/src/ciclo/prueba.rs:24) ~ `ana` (crates/tc2/src/ciclo/liberar.rs:408), `ana` (crates/tc2/src/ciclo/operacion.rs:277), `ana` (crates/tc2/src/plan/guardar.rs:157) y 4 mas
- `fila` (crates/tc2/src/ciclo/prueba.rs:59) ~ `fila` (crates/tc2/src/ciclo/liberar.rs:438), `fila` (crates/tc2/src/ciclo/operacion.rs:296), `Fila` (crates/tc2/src/doctor/capacidades.rs:49) y 6 mas
- `Calculo` (crates/tc2/src/confianza/mod.rs:237) ~ `Calculo` (crates/tc2/src/alta.rs:453), `Calculo` (crates/tc2/src/siguiente.rs:91)
- `nuevo` (crates/tc2/src/confianza/mod.rs:248) ~ `nueva` (crates/tc2/src/alta.rs:55), `nueva` (crates/tc2/src/alta.rs:323), `nueva` (crates/tc2/src/ciclo/liberar.rs:86) y 56 mas
- `guardar` (crates/tc2/src/confianza/mod.rs:277) ~ `guardar` (crates/tc2/src/estado/pendientes.rs:99), `guardar` (crates/tc2/src/git/guardar.rs:17), `guardar` (crates/tc2/src/identidad.rs:261) y 2 mas
- `evaluacion` (crates/tc2/src/confianza/mod.rs:289) ~ `Evaluacion` (crates/tc2/src/confianza/mod.rs:72)
- `mensaje` (crates/tc2/src/contrato.rs:217) ~ `mensaje` (crates/tc2/src/alta.rs:287), `mensaje` (crates/tc2/src/estado/transaccion.rs:246), `mensaje` (crates/tc2/src/git/puerto.rs:224) y 1 mas
- `ruta` (crates/tc2/src/desinstalar/mod.rs:311) ~ `ruta` (crates/tc2/src/estado/frescura.rs:154), `ruta` (crates/tc2/src/hallazgos/orden.rs:127), `ruta` (crates/tc2/src/identidad.rs:222) y 3 mas
- `aplicar` (crates/tc2/src/desinstalar/mod.rs:482) ~ `aplica` (crates/tc2/src/inventario.rs:875), `aplicar` (crates/tc2/src/alta.rs:400), `aplicar` (crates/tc2/src/ciclo/liberar.rs:243) y 29 mas
- `quitar_worktree` (crates/tc2/src/desinstalar/proyecto.rs:377) ~ `quitar_worktree` (crates/tc2/src/git/worktree.rs:79)
- `borrar_rama` (crates/tc2/src/desinstalar/proyecto.rs:402) ~ `borrar_rama` (crates/tc2/tests/plataforma_e2e.rs:130), `borrar_rama` (crates/tc2/tests/plataforma_gitlab_e2e.rs:171)
- `aplicar` (crates/tc2/src/init/aplicar.rs:27) ~ `aplica` (crates/tc2/src/inventario.rs:875), `aplicar` (crates/tc2/src/alta.rs:400), `aplicar` (crates/tc2/src/ciclo/liberar.rs:243) y 29 mas
- ... y 7 mas (tope 15)

### Posible duplicacion dentro de la tarea

Definiciones nuevas parecidas entre si: la misma tarea puede estar escribiendo dos veces lo mismo.

- `aplicable` (crates/tc2/src/desinstalar/mod.rs:304) ~ `aplicable` (crates/tc2/src/init/mod.rs:270)
- `aplicable` (crates/tc2/src/desinstalar/mod.rs:304) ~ `aplicable` (crates/tc2/src/plan_dato.rs:16)
- `aplazar` (crates/tc2/src/desinstalar/mod.rs:307) ~ `aplazar` (crates/tc2/src/init/mod.rs:273)
- `aplazar` (crates/tc2/src/desinstalar/mod.rs:307) ~ `aplazar` (crates/tc2/src/plan_dato.rs:18)
- `ruta` (crates/tc2/src/desinstalar/mod.rs:311) ~ `ruta` (crates/tc2/src/init/mod.rs:277)
- `ruta` (crates/tc2/src/desinstalar/mod.rs:311) ~ `ruta` (crates/tc2/src/plan_dato.rs:20)
- `aplicar` (crates/tc2/src/desinstalar/mod.rs:482) ~ `aplicar` (crates/tc2/src/init/aplicar.rs:27)
- `aplicable` (crates/tc2/src/init/mod.rs:270) ~ `aplicable` (crates/tc2/src/plan_dato.rs:16)
- `aplazar` (crates/tc2/src/init/mod.rs:273) ~ `aplazar` (crates/tc2/src/plan_dato.rs:18)
- `ruta` (crates/tc2/src/init/mod.rs:277) ~ `ruta` (crates/tc2/src/plan_dato.rs:20)

