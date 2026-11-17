# Peticion de revision — TASK-034 (ronda 2)

- Tarea: TASK-034 — F3-7 descartar y pausa
- Rama revisada: feature/task-034-f3-7-descartar-y-pausa
- Rama base: develop
- Commit revisado (HEAD): 7fffc653759ff7aef08b5137ea8837a61bf7350f
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-034 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 5dc73db88b26d5303bba1983e29f5d2de4803f22 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (5 hallazgo(s)))

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

- tareas/03-en-revision/TASK-034/tarea.md
- tareas/03-en-revision/TASK-034/planificacion/
- crates/tc2/src/ciclo/operacion.rs
- crates/tc2/src/config/mod.rs
- crates/tc2/src/estado/mod.rs
- crates/tc2/src/estado/tareas.rs
- crates/tc2/src/i18n/catalogo/ciclo.rs
- crates/tc2/src/i18n/catalogo/comun.rs
- crates/tc2/src/i18n/catalogo/toma.rs
- crates/tc2/src/identidad.rs
- crates/tc2/src/toma/operacion.rs
- crates/tc2/tests/ciclo.rs
- crates/tc2/tests/config.rs
- docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 3 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| 1 | IMPORTANTE | abierto | crates/tc2/src/ciclo/mod.rs (`hay_remoto` en `paso_descartar`) | informe-revision-1-code-quality-reviewer.md |

## Commits a revisar (git log 5dc73db88b26d5303bba1983e29f5d2de4803f22..HEAD)

````
7fffc65 refactor(TASK-034): rechazo por asignado y por estado en un solo sitio para start y ciclo
ff5b236 fix(TASK-034): personas y config de la base desde una subcarpeta del clon
a22dee0 docs(TASK-034): lineas de pausa y reanudar en la adenda de F1-4 con su test
c8acfda test(TASK-034): E2E de descartar con la rama publicada desde otro clon, sin publicar y con HEAD suelto
db0bc53 chore(TASK-034): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 5dc73db88b26d5303bba1983e29f5d2de4803f22..HEAD)

````diff
diff --git a/crates/tc2/src/ciclo/operacion.rs b/crates/tc2/src/ciclo/operacion.rs
index b00f0b0..6432f38 100644
--- a/crates/tc2/src/ciclo/operacion.rs
+++ b/crates/tc2/src/ciclo/operacion.rs
@@ -19,8 +19,8 @@
 
 use std::cell::RefCell;
 
-use crate::estado::{Cambio, Estado, Fallo, Operacion, tarea_de};
-use crate::i18n::Mensaje;
+use crate::estado::{Cambio, Estado, Fallo, Operacion, de_otra_persona, por_estado, tarea_de};
+use crate::grafo::DESCARTADA;
 use crate::i18n::catalogo::ciclo as txt;
 use crate::identidad::Quien;
 use crate::tarea::{Asignado, Cuerpo, PAUSA, REANUDAR, Tarea, Transicion};
@@ -30,11 +30,9 @@ use crate::toma::operacion::MODO;
 /// Nombre de `descartar`, de su operacion y de su fila de transicion.
 pub const DESCARTAR: &str = "descartar";
 
-/// Estado en el que queda una tarea descartada.
-pub const DESCARTADA: &str = "descartada";
-
-/// Estados desde los que se descarta, se pausa o se reanuda.
-pub const DE_ORIGEN: [&str; 4] = ["planificada", "en-diseno", "en-curso", "en-revision"];
+/// Estados de una tarea abierta (ni terminada ni descartada): desde ellos se
+/// descarta, se pausa o se reanuda.
+pub const ABIERTAS: [&str; 4] = ["planificada", "en-diseno", "en-curso", "en-revision"];
 
 /// Largo maximo del motivo de un descarte, en caracteres: cabe en una
 /// linea del ROADMAP y en la anotacion del commit.
@@ -68,43 +66,27 @@ impl Lanzada {
         }
     }
 
-    fn rechazo(&self, motivo: &str, m: Mensaje) -> Fallo {
-        Fallo::rechazada(motivo, m.con("id", self.id.as_str()))
-    }
-
-    /// La tarea de `estado`, si quien la lanza puede cambiarla (D2): la lleva
-    /// el o nadie.
-    fn tarea(&self, estado: &dyn Estado) -> Result<(Tarea, Cuerpo), Fallo> {
+    /// La tarea de `estado`, si quien lanza `orden` puede cambiarla (D2): la
+    /// lleva el o nadie.
+    fn tarea(&self, estado: &dyn Estado, orden: &str) -> Result<(Tarea, Cuerpo), Fallo> {
         *self.dueno.borrow_mut() = None;
         let (t, c) = tarea_de(estado, &self.id)?;
-        match &t.asignado_a {
-            Some(a) if !a.misma_persona(&self.asignado) => Err(match (a.login(), a.heredado()) {
-                (Some(login), _) => {
-                    *self.dueno.borrow_mut() = Some(login.to_owned());
-                    self.rechazo(
-                        "de-otra-persona",
-                        Mensaje::nuevo(&txt::DE_OTRA_PERSONA).con("login", login),
-                    )
-                }
-                (None, texto) => self.rechazo(
-                    "asignado-heredado",
-                    Mensaje::nuevo(&txt::ASIGNADO_HEREDADO).con("texto", texto.unwrap_or_default()),
-                ),
-            }),
-            _ => Ok((t, c)),
+        if let Some((fallo, dueno)) = de_otra_persona(
+            &self.id,
+            t.asignado_a.as_ref(),
+            &self.asignado,
+            orden,
+            ("de-otra-persona", &txt::DE_OTRA_PERSONA),
+        ) {
+            *self.dueno.borrow_mut() = dueno;
+            return Err(fallo);
         }
+        Ok((t, c))
     }
 
-    /// `terminada`, `descartada` u otro estado que no es de [`DE_ORIGEN`].
-    fn estado_no_valido(&self, estado: &str) -> Fallo {
-        match estado {
-            "terminada" => self.rechazo("terminada", Mensaje::nuevo(&txt::TERMINADA)),
-            DESCARTADA => self.rechazo("descartada", Mensaje::nuevo(&txt::DESCARTADA)),
-            otro => self.rechazo(
-                "estado-no-valido",
-                Mensaje::nuevo(&txt::ESTADO_NO_VALIDO).con("estado", otro),
-            ),
-        }
+    /// `terminada`, `descartada` u otro estado que no es de [`ABIERTAS`].
+    fn estado_no_valido(&self, estado: &str, orden: &str) -> Fallo {
+        por_estado(&self.id, estado, orden, &ABIERTAS)
     }
 
     /// El `tarea.md` con `actualizado` de hoy y la fila `fase`.
@@ -175,12 +157,12 @@ impl Operacion for Descartar {
     }
 
     fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
-        let (mut t, cuerpo) = self.base.tarea(estado)?;
+        let (mut t, cuerpo) = self.base.tarea(estado, DESCARTAR)?;
         match t.estado.as_str() {
             // Ya descartada: nada que escribir; el motivo es el de entonces.
             DESCARTADA => return Ok(Vec::new()),
-            e if DE_ORIGEN.contains(&e) => {}
-            otro => return Err(self.base.estado_no_valido(otro)),
+            e if ABIERTAS.contains(&e) => {}
+            otro => return Err(self.base.estado_no_valido(otro, DESCARTAR)),
         }
         DESCARTADA.clone_into(&mut t.estado);
         t.motivo = Some(self.motivo.clone());
@@ -233,9 +215,9 @@ impl Operacion for Pausar {
     }
 
     fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
-        let (t, cuerpo) = self.base.tarea(estado)?;
-        if !DE_ORIGEN.contains(&t.estado.as_str()) {
-            return Err(self.base.estado_no_valido(&t.estado));
+        let (t, cuerpo) = self.base.tarea(estado, self.fase())?;
+        if !ABIERTAS.contains(&t.estado.as_str()) {
+            return Err(self.base.estado_no_valido(&t.estado, self.fase()));
         }
         if cuerpo.en_pausa() != self.reanudar {
             // Ya en pausa (o ya sin ella): nada que escribir.
diff --git a/crates/tc2/src/config/mod.rs b/crates/tc2/src/config/mod.rs
index 2ef5a98..6e005db 100644
--- a/crates/tc2/src/config/mod.rs
+++ b/crates/tc2/src/config/mod.rs
@@ -508,12 +508,21 @@ fn texto_de_base(git: &Git, dir: &Path) -> Paso<String> {
             .con("motivo", "sin-base")
             .diagnostico(Mensaje::nuevo(&t::SIN_BASE).con("base", BASE)));
     }
+    // `--full-tree`: `RUTA` es desde la raiz del repo, no desde `dir` (que
+    // puede ser una subcarpeta del clon o de un worktree).
     let listado = git::exigir(
         git,
         dir,
         NOMBRE,
         Subcomando::LsTree,
-        &["--name-only", "--end-of-options", BASE, "--", RUTA],
+        &[
+            "--full-tree",
+            "--name-only",
+            "--end-of-options",
+            BASE,
+            "--",
+            RUTA,
+        ],
     )?;
     if listado.trim() != RUTA {
         return Err(Invalida::una(
diff --git a/crates/tc2/src/estado/mod.rs b/crates/tc2/src/estado/mod.rs
index b3d85f3..73b2ad6 100644
--- a/crates/tc2/src/estado/mod.rs
+++ b/crates/tc2/src/estado/mod.rs
@@ -31,7 +31,7 @@ pub use crate::git::Cambio;
 pub use cerrojo::Cerrojo;
 pub use lectura::{Estado, Vista, leer};
 pub use montaje::{Previsto, montar, montar_con, previsto};
-pub use tareas::{Leida, es_tarea_md, tarea_de, tareas};
+pub use tareas::{Leida, de_otra_persona, es_tarea_md, por_estado, tarea_de, tareas};
 pub use transaccion::{Catalogo, Contexto, Fallo, Operacion, Politica, Roadmap, transaccion};
 
 /// Rama huerfana del estado (ADR F1-1; nombre en [`crate::huella`]).
diff --git a/crates/tc2/src/estado/tareas.rs b/crates/tc2/src/estado/tareas.rs
index 8b9d1f1..942390e 100644
--- a/crates/tc2/src/estado/tareas.rs
+++ b/crates/tc2/src/estado/tareas.rs
@@ -4,9 +4,10 @@
 
 use super::lectura::Estado;
 use super::transaccion::Fallo;
-use crate::i18n::Mensaje;
+use crate::grafo::{DESCARTADA, TERMINADA};
 use crate::i18n::catalogo::comun as txt;
-use crate::tarea::{Cuerpo, Documento, ErrorTarea, Tarea};
+use crate::i18n::{Mensaje, Texto};
+use crate::tarea::{Asignado, Cuerpo, Documento, ErrorTarea, Tarea};
 
 /// Prefijo de las carpetas de tarea.
 const PREFIJO: &str = "TASK-";
@@ -66,9 +67,123 @@ pub fn tarea_de(estado: &dyn Estado, id: &str) -> Result<(Tarea, Cuerpo), Fallo>
     }
 }
 
+/// El rechazo de la tarea `id` si su `asignado_a` es otra persona que
+/// `quien`, comun a las operaciones de una tarea: `None` si no la lleva
+/// nadie o la lleva el. La de una persona de la lista se rechaza con `ajena`
+/// (motivo y texto de la orden: `tomada` en `start`, `de-otra-persona` en
+/// `descartar`, `pausa` y `reanudar`) y devuelve su login; un texto heredado
+/// de la 0.16, `asignado-heredado` en todas.
+pub fn de_otra_persona(
+    id: &str,
+    asignado_a: Option<&Asignado>,
+    quien: &Asignado,
+    orden: &str,
+    ajena: (&str, &'static Texto),
+) -> Option<(Fallo, Option<String>)> {
+    let a = asignado_a.filter(|a| !a.misma_persona(quien))?;
+    let rechazo = |motivo: &str, m: Mensaje| Fallo::rechazada(motivo, m.con("id", id));
+    Some(match (a.login(), a.heredado()) {
+        (Some(login), _) => (
+            rechazo(ajena.0, Mensaje::nuevo(ajena.1).con("login", login)),
+            Some(login.to_owned()),
+        ),
+        (None, texto) => (
+            rechazo(
+                "asignado-heredado",
+                Mensaje::nuevo(&txt::TAREA_ASIGNADO_HEREDADO)
+                    .con("texto", texto.unwrap_or_default())
+                    .con("orden", orden),
+            ),
+            None,
+        ),
+    })
+}
+
+/// El rechazo de la tarea `id` en un `estado` que no admite la `orden`
+/// (no es de `admitidos`): `terminada`, `descartada` o `estado-no-valido`.
+pub fn por_estado(id: &str, estado: &str, orden: &str, admitidos: &[&str]) -> Fallo {
+    let (motivo, m) = match estado {
+        TERMINADA => ("terminada", Mensaje::nuevo(&txt::TAREA_TERMINADA)),
+        DESCARTADA => ("descartada", Mensaje::nuevo(&txt::TAREA_DESCARTADA)),
+        otro => (
+            "estado-no-valido",
+            Mensaje::nuevo(&txt::TAREA_ESTADO_NO_VALIDO)
+                .con("estado", otro)
+                .con("estados", admitidos.join(", ")),
+        ),
+    };
+    Fallo::rechazada(motivo, m.con("id", id).con("orden", orden))
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
+    use crate::i18n::Idioma;
+
+    /// Motivo y diagnostico (en los dos idiomas) de un rechazo.
+    fn partes(f: &Fallo) -> (String, Vec<String>) {
+        match f {
+            Fallo::Rechazada {
+                motivo,
+                diagnostico,
+            } => (
+                motivo.clone(),
+                [Idioma::Es, Idioma::En]
+                    .into_iter()
+                    .map(|i| diagnostico.texto(i))
+                    .collect(),
+            ),
+            Fallo::Error(_) => (String::new(), Vec::new()),
+        }
+    }
+
+    /// Los rechazos comunes de `start` y de `descartar`/`pausa`/`reanudar`
+    /// llevan el ID y la orden; el de estado, los estados que la admiten.
+    #[test]
+    fn rechazos_comunes_por_estado_y_por_asignado() {
+        for (estado, motivo) in [
+            (TERMINADA, "terminada"),
+            (DESCARTADA, "descartada"),
+            ("en-curso", "estado-no-valido"),
+        ] {
+            let (m, textos) = partes(&por_estado(
+                "TASK-007",
+                estado,
+                "start",
+                &["planificada", "en-diseno"],
+            ));
+            assert_eq!(m, motivo);
+            for t in textos {
+                assert!(t.contains("TASK-007") && t.contains("tc2 start"), "{t}");
+                if motivo == "estado-no-valido" {
+                    assert!(
+                        t.contains("en-curso") && t.contains("planificada, en-diseno"),
+                        "{t}"
+                    );
+                }
+            }
+        }
+        let ana = Asignado::persona(7, "ana");
+        let beto = Asignado::persona(9, "beto");
+        let yo = ana.clone().unwrap_or_else(|| unreachable!());
+        let ajena = (
+            "de-otra-persona",
+            &crate::i18n::catalogo::ciclo::DE_OTRA_PERSONA,
+        );
+        assert!(de_otra_persona("TASK-007", None, &yo, "pausa", ajena).is_none());
+        assert!(de_otra_persona("TASK-007", ana.as_ref(), &yo, "pausa", ajena).is_none());
+        let (f, dueno) = de_otra_persona("TASK-007", beto.as_ref(), &yo, "pausa", ajena)
+            .unwrap_or_else(|| unreachable!());
+        assert_eq!(dueno.as_deref(), Some("beto"));
+        let (m, textos) = partes(&f);
+        assert_eq!(m, "de-otra-persona");
+        assert!(
+            textos
+                .iter()
+                .all(|t| t.contains("beto") && t.contains("TASK-007")),
+            "{textos:?}"
+        );
+    }
 
     #[test]
     fn rutas_de_tarea_md() {
diff --git a/crates/tc2/src/i18n/catalogo/ciclo.rs b/crates/tc2/src/i18n/catalogo/ciclo.rs
index ad70d3d..965a891 100644
--- a/crates/tc2/src/i18n/catalogo/ciclo.rs
+++ b/crates/tc2/src/i18n/catalogo/ciclo.rs
@@ -21,20 +21,4 @@ crate::i18n::textos! {
         es: "la tarea {id} la lleva {login}: solo quien la lleva (o cualquiera, si no la lleva nadie) puede descartarla, pausarla o reanudarla",
         en: "task {id} is taken by {login}: only whoever has it (or anyone, if nobody has it) can discard, pause or resume it",
     }
-    ASIGNADO_HEREDADO {
-        es: "la tarea {id} tiene un asignado_a heredado de la 0.16 ({texto}): hay que reasignarla a una persona de la lista antes de cambiarla",
-        en: "task {id} has an assignee inherited from 0.16 ({texto}): it has to be reassigned to a person on the list before changing it",
-    }
-    TERMINADA {
-        es: "la tarea {id} ya esta terminada: no se puede descartar, pausar ni reanudar",
-        en: "task {id} is already finished: it cannot be discarded, paused or resumed",
-    }
-    DESCARTADA {
-        es: "la tarea {id} esta descartada: no se puede pausar ni reanudar",
-        en: "task {id} is discarded: it cannot be paused or resumed",
-    }
-    ESTADO_NO_VALIDO {
-        es: "la tarea {id} esta {estado}: solo se descartan, pausan o reanudan tareas planificadas, en diseno, en curso o en revision",
-        en: "task {id} is {estado}: only planned, in-design, in-progress or in-review tasks can be discarded, paused or resumed",
-    }
 }
diff --git a/crates/tc2/src/i18n/catalogo/comun.rs b/crates/tc2/src/i18n/catalogo/comun.rs
index e6c9fc1..73f2f68 100644
--- a/crates/tc2/src/i18n/catalogo/comun.rs
+++ b/crates/tc2/src/i18n/catalogo/comun.rs
@@ -18,4 +18,22 @@ crate::i18n::textos! {
         es: "el tarea.md de {id} no se entiende ({error}): corrigelo antes de cambiarla",
         en: "the tarea.md of {id} cannot be read ({error}): fix it before changing it",
     }
+    /// Rechazos comunes de las operaciones de una tarea; `{orden}` es la
+    /// orden (`start`, `descartar`, `pausa`, `reanudar`).
+    TAREA_ASIGNADO_HEREDADO {
+        es: "la tarea {id} tiene un asignado_a heredado de la 0.16 ({texto}): hay que reasignarla a una persona de la lista antes de usar tc2 {orden} con ella",
+        en: "task {id} has an assignee inherited from 0.16 ({texto}): it has to be reassigned to a person on the list before using tc2 {orden} on it",
+    }
+    TAREA_TERMINADA {
+        es: "la tarea {id} ya esta terminada: tc2 {orden} no se aplica a una tarea terminada",
+        en: "task {id} is already finished: tc2 {orden} does not apply to a finished task",
+    }
+    TAREA_DESCARTADA {
+        es: "la tarea {id} esta descartada: tc2 {orden} no se aplica a una tarea descartada",
+        en: "task {id} is discarded: tc2 {orden} does not apply to a discarded task",
+    }
+    TAREA_ESTADO_NO_VALIDO {
+        es: "la tarea {id} esta {estado}: tc2 {orden} solo se aplica a tareas en {estados}",
+        en: "task {id} is {estado}: tc2 {orden} only applies to tasks in {estados}",
+    }
 }
diff --git a/crates/tc2/src/i18n/catalogo/toma.rs b/crates/tc2/src/i18n/catalogo/toma.rs
index ff7d278..e1374f7 100644
--- a/crates/tc2/src/i18n/catalogo/toma.rs
+++ b/crates/tc2/src/i18n/catalogo/toma.rs
@@ -17,22 +17,6 @@ crate::i18n::textos! {
         es: "la tarea {id} ya la lleva {login}: elige otra con tc2 next o pide que te la reasigne",
         en: "task {id} is already taken by {login}: pick another one with tc2 next or ask to have it reassigned",
     }
-    ASIGNADO_HEREDADO {
-        es: "la tarea {id} tiene un asignado_a heredado de la 0.16 ({texto}): hay que reasignarla a una persona de la lista antes de empezarla",
-        en: "task {id} has an assignee inherited from 0.16 ({texto}): it has to be reassigned to a person on the list before starting it",
-    }
-    TERMINADA {
-        es: "la tarea {id} ya esta terminada: no se puede empezar",
-        en: "task {id} is already finished: it cannot be started",
-    }
-    DESCARTADA {
-        es: "la tarea {id} esta descartada: no se puede empezar",
-        en: "task {id} is discarded: it cannot be started",
-    }
-    ESTADO_NO_VALIDO {
-        es: "la tarea {id} esta {estado}: start solo empieza tareas planificadas o en diseno",
-        en: "task {id} is {estado}: start only starts planned or in-design tasks",
-    }
     PAUSADA {
         es: "la tarea {id} esta en pausa: reanudala con tc2 reanudar {id} antes de empezarla",
         en: "task {id} is paused: resume it with tc2 reanudar {id} before starting it",
diff --git a/crates/tc2/src/identidad.rs b/crates/tc2/src/identidad.rs
index 3101c52..56c8c97 100644
--- a/crates/tc2/src/identidad.rs
+++ b/crates/tc2/src/identidad.rs
@@ -395,7 +395,11 @@ pub fn personas_de_base(g: &Git, dir: &Path) -> Paso<Personas> {
         dir,
         NOMBRE,
         Subcomando::LsTree,
+        // `--full-tree`: la ruta es desde la raiz del repo y no desde la
+        // carpeta en la que se lanza (una subcarpeta del clon o del
+        // worktree no la encontraria).
         &[
+            "--full-tree",
             "--name-only",
             "--end-of-options",
             config::BASE,
diff --git a/crates/tc2/src/toma/operacion.rs b/crates/tc2/src/toma/operacion.rs
index 9b60a2a..75d8e98 100644
--- a/crates/tc2/src/toma/operacion.rs
+++ b/crates/tc2/src/toma/operacion.rs
@@ -22,7 +22,7 @@
 
 use std::cell::RefCell;
 
-use crate::estado::{Cambio, Estado, Fallo, Operacion, tarea_de};
+use crate::estado::{Cambio, Estado, Fallo, Operacion, de_otra_persona, por_estado, tarea_de};
 use crate::grafo::{Nodo, dependencia_cumplida};
 use crate::i18n::Mensaje;
 use crate::i18n::catalogo::toma as txt;
@@ -35,8 +35,8 @@ use super::NOMBRE;
 /// la reanuda sin escribir estado (F1-3, decision 2, caso c).
 pub const EMPEZADA: [&str; 2] = ["en-curso", "en-revision"];
 
-/// Estados desde los que se puede empezar (D1 de TASK-033).
-pub const DE_ORIGEN: [&str; 2] = ["planificada", "en-diseno"];
+/// Estados desde los que `start` empieza una tarea (D1 de TASK-033).
+pub const EMPEZABLES: [&str; 2] = ["planificada", "en-diseno"];
 
 /// Estado en el que queda la tarea tomada.
 pub const EN_CURSO: &str = "en-curso";
@@ -135,36 +135,18 @@ impl Operacion for Tomar {
         if mia && EMPEZADA.contains(&t.estado.as_str()) {
             return Ok(Vec::new());
         }
-        match &t.asignado_a {
-            Some(a) if !mia => {
-                return Err(match (a.login(), a.heredado()) {
-                    (Some(login), _) => {
-                        *self.dueno.borrow_mut() = Some(login.to_owned());
-                        self.rechazo("tomada", Mensaje::nuevo(&txt::TOMADA).con("login", login))
-                    }
-                    (None, texto) => self.rechazo(
-                        "asignado-heredado",
-                        Mensaje::nuevo(&txt::ASIGNADO_HEREDADO)
-                            .con("texto", texto.unwrap_or_default()),
-                    ),
-                });
-            }
-            _ => {}
+        if let Some((fallo, dueno)) = de_otra_persona(
+            &self.id,
+            t.asignado_a.as_ref(),
+            &self.asignado,
+            NOMBRE,
+            ("tomada", &txt::TOMADA),
+        ) {
+            *self.dueno.borrow_mut() = dueno;
+            return Err(fallo);
         }
-        match t.estado.as_str() {
-            e if DE_ORIGEN.contains(&e) => {}
-            "terminada" => {
-                return Err(self.rechazo("terminada", Mensaje::nuevo(&txt::TERMINADA)));
-            }
-            "descartada" => {
-                return Err(self.rechazo("descartada", Mensaje::nuevo(&txt::DESCARTADA)));
-            }
-            otro => {
-                return Err(self.rechazo(
-                    "estado-no-valido",
-                    Mensaje::nuevo(&txt::ESTADO_NO_VALIDO).con("estado", otro),
-                ));
-            }
+        if !EMPEZABLES.contains(&t.estado.as_str()) {
+            return Err(por_estado(&self.id, &t.estado, NOMBRE, &EMPEZABLES));
         }
         // En pausa (TASK-034): no se empieza hasta reanudarla.
         if cuerpo.en_pausa() {
diff --git a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
index 1cfd75e..2faab15 100644
--- a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
+++ b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
@@ -7,7 +7,7 @@
 - Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a, D5a del plan de TASK-005)
 - Adenda (2026-11-17, TASK-032, F3-5): entra `prs-abiertos` en la lista cerrada (decisión 2) para `tc2 board` (D2 de TASK-032, aceptada por Carlos el 2026-11-16); detalle de llamadas en la [adenda de F1-13](F1-13-adaptador-de-plataforma.md). `board` y `status`, como `roadmap`, escriben avisos de una línea en stderr con código 0; el canal fiable son las claves de la línea y del JSON
 - Adenda (2026-11-17, TASK-033, F3-6): `start [--json] TASK-NNN` compone `fetch`, `validar-base`, `comprobar-id`, `commit-estado`, `crear-rama-y-worktree` y `reanudar` sin operaciones nuevas (la base de un `hotfix` es el último tag por versión de `refs/tags/` contenido en `origin/main`, leído con `for-each-ref --merged`, o su punta). Línea de éxito: `start ok id=… rama=… worktree=… base=… estado=<commit de la toma> asignado=<login>`; dueño con la tarea empezada: `sin-cambios` o `ok creado=rama,worktree` sin escribir estado. Motivos de `rechazada` (3): `no-existe`, `formato-1` (con `arreglo=tc2 actualizar`), `ilegible`, `tomada` (con `asignado=<login>`), `asignado-heredado`, `terminada`, `descartada`, `estado-no-valido`, `dependencias-pendientes` (con `dependencias=`), `rama-invalida`, `tipo-release`, `ocupada` (con `ramas=`), `carpeta-ocupada` (la carpeta del worktree existe y no es el worktree registrado de la rama; se comprueba antes de publicar la toma, sin `estado=`) e `identidad`; `uso` (2): `sin-id`, `id-invalido`, `argumentos-de-mas`; sin red, `sinred pendiente=no` (4)
-- Adenda (2026-11-17, TASK-034, F3-7): `descartar [--json] TASK-NNN "<motivo>"`, `pausa [--json] TASK-NNN` y `reanudar [--json] TASK-NNN` componen `fetch` (solo `descartar`), `commit-estado` y `quitar-worktree` (solo `descartar`, tras publicar) sin operaciones nuevas. Líneas: `descartar ok id=… commit=… intentos=… tareas=… worktree=quitado|ninguno|conservado [worktree-motivo=cambios-sin-guardar|sin-publicar|sin-rama|en-uso|error]` (el worktree conservado no hace fallar la orden: el descarte ya está publicado y la rama no se toca nunca); `pausa ok id=… commit=…` y `reanudar ok id=… commit=…`. `sin-cambios` (0): descartar una descartada (vale el motivo de entonces y se intenta quitar el worktree), pausar una en pausa o reanudar una que no lo está. Motivos de `rechazada` (3): `no-existe`, `formato-1` (con `arreglo=tc2 actualizar`), `ilegible`, `de-otra-persona` (con `asignado=<login>`; D2: puede quien la lleva o cualquiera si no la lleva nadie), `asignado-heredado`, `terminada`, `descartada` (pausa y reanudar), `estado-no-valido` e `identidad`; `start` gana `pausada` (con `arreglo=tc2 reanudar`). `uso` (2): `sin-id`, `id-invalido`, `argumentos-de-mas` y, en `descartar`, `sin-motivo` y `motivo-invalido` (vacío, solo espacios, saltos de línea o caracteres de control, o más de 500 caracteres). Sin red: `descartar sinred pendiente=no` (4); `pausa`/`reanudar sinred pendiente=si` (4), que la siguiente transacción con red publica (adenda de TASK-034 en F1-2)
+- Adenda (2026-11-17, TASK-034, F3-7): `descartar [--json] TASK-NNN "<motivo>"`, `pausa [--json] TASK-NNN` y `reanudar [--json] TASK-NNN` componen `fetch` (solo `descartar`), `commit-estado` y `quitar-worktree` (solo `descartar`, tras publicar) sin operaciones nuevas. Líneas: `descartar ok id=… commit=… intentos=… tareas=… worktree=quitado|ninguno|conservado [worktree-motivo=cambios-sin-guardar|sin-publicar|sin-rama|en-uso|error]` (el worktree conservado no hace fallar la orden: el descarte ya está publicado y la rama no se toca nunca); `pausa ok id=… commit=… intentos=… tareas=…` y `reanudar ok id=… commit=… intentos=… tareas=…` (las claves de la transacción, como `descartar`, sin `worktree`). `sin-cambios` (0): descartar una descartada (vale el motivo de entonces y se intenta quitar el worktree), pausar una en pausa o reanudar una que no lo está. Motivos de `rechazada` (3): `no-existe`, `formato-1` (con `arreglo=tc2 actualizar`), `ilegible`, `de-otra-persona` (con `asignado=<login>`; D2: puede quien la lleva o cualquiera si no la lleva nadie), `asignado-heredado`, `terminada`, `descartada` (pausa y reanudar), `estado-no-valido` e `identidad`; `start` gana `pausada` (con `arreglo=tc2 reanudar`). `uso` (2): `sin-id`, `id-invalido`, `argumentos-de-mas` y, en `descartar`, `sin-motivo` y `motivo-invalido` (vacío, solo espacios, saltos de línea o caracteres de control, o más de 500 caracteres). Sin red: `descartar sinred pendiente=no` (4); `pausa`/`reanudar sinred pendiente=si` (4), que la siguiente transacción con red publica (adenda de TASK-034 en F1-2)
 - Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
 
 ## Contexto
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 5dc73db88b26d5303bba1983e29f5d2de4803f22..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-034/planificacion/brainstorm/peticion-plan-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-034/planificacion/plan-final.md                 |    0
 tareas/03-en-revision/TASK-034/revision/informe-revision-1-code-quality-reviewer.md         |   21 ++
 tareas/03-en-revision/TASK-034/revision/informe-revision-1-rust-wasm-reviewer.md            |   21 ++
 tareas/03-en-revision/TASK-034/revision/peticion-revision-1-code-quality-reviewer.md        |  133 ++++++++
 tareas/03-en-revision/TASK-034/revision/peticion-revision-1-rust-wasm-reviewer.md           | 1642 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-034/tarea.md                                    |    6 +-
 7 files changed, 1821 insertions(+), 2 deletions(-)
````

## Tests del diff (2 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff 5dc73db88b26d5303bba1983e29f5d2de4803f22..HEAD -- ":(literal)crates/tc2/tests/ciclo.rs" ":(literal)crates/tc2/tests/config.rs"
````

````
crates/tc2/tests/ciclo.rs  | 112 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/config.rs |   6 ++++++
 2 files changed, 118 insertions(+)
````
