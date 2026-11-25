# Peticion de revision — TASK-149 (ronda 1)

- Tarea: TASK-149 — Auditoria 4D: capa comun de ordenes y operaciones con dueno (6 IMPORTANTE)
- Rama revisada: fix/task-149-auditoria-4d-capa-comun-de-ordenes-y-ope
- Rama base: develop
- Commit revisado (HEAD): 02d8a4d06b04453d55e45765ce24186700d13434
- Fecha: 2026-11-25
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-149 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-149/tarea.md
- tareas/03-en-revision/TASK-149/planificacion/
- crates/tc2/src/alta.rs
- crates/tc2/src/ciclo/liberar.rs
- crates/tc2/src/ciclo/mod.rs
- crates/tc2/src/ciclo/operacion.rs
- crates/tc2/src/config/mod.rs
- crates/tc2/src/contrato.rs
- crates/tc2/src/estado/dueno.rs
- crates/tc2/src/estado/mod.rs
- crates/tc2/src/estado/transaccion.rs
- crates/tc2/src/estimacion/mod.rs
- crates/tc2/src/hallazgos/orden.rs
- crates/tc2/src/importar/mod.rs
- crates/tc2/src/plan/aprobar.rs
- crates/tc2/src/plan/guardar.rs
- crates/tc2/src/plan/mod.rs
- crates/tc2/src/plan/operacion.rs
- crates/tc2/src/revision/guardar.rs
- crates/tc2/src/revision/mod.rs
- crates/tc2/src/revision/operacion.rs
- crates/tc2/src/roadmap/mod.rs
- crates/tc2/src/seleccion/orden.rs
- crates/tc2/src/siguiente.rs
- crates/tc2/src/tablero.rs
- crates/tc2/src/toma/mod.rs
- crates/tc2/src/toma/operacion.rs

## Commits a revisar (git log develop..HEAD)

````
02d8a4d chore(TASK-149): resultado y autorrevision
959fdad fix(TASK-149): copias de valor en contrato::valor_de
932567c fix(TASK-149): A8-19 anotacion con dueno en estado::AnotacionConDueno
8f11a44 fix(TASK-149): A8-07 preambulo con dueno en estado::tarea_propia
b917683 fix(TASK-149): A8-10 politica de lectura de la base en de_base_o_suelo
b67b539 fix(TASK-149): A8-09 con_dueno y escribir escritos una vez
c360bba fix(TASK-149): A8-08 adaptador de la transaccion a la orden en como_orden
bb0a417 fix(TASK-149): A8-06 conversion Fallo a Informe en Fallo::informe
eaa77f0 chore(TASK-149): tarea en curso
````

## Diff de tu dominio (25 fichero(s) de 25; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/alta.rs b/crates/tc2/src/alta.rs
index 7368de8..7e24646 100644
--- a/crates/tc2/src/alta.rs
+++ b/crates/tc2/src/alta.rs
@@ -181,9 +181,9 @@ pub fn entrada_de_args(args: &[String]) -> Result<Entrada, crate::contrato::Info
         if !conocida || (!repetible && !vistas.insert(opcion)) {
             return Err(uso("argumentos-de-mas", &ti::USO_NEW_CAMPOS));
         }
-        let Some(v) = it.next() else {
-            return Err(uso("falta-valor", &ti::USO_NEW_CAMPOS).con("opcion", &opcion[2..]));
-        };
+        let v = &crate::contrato::valor_de(&mut it, |m| {
+            uso(m, &ti::USO_NEW_CAMPOS).con("opcion", &opcion[2..])
+        })?;
         match opcion {
             "--tipo" if crate::tarea::TIPOS.contains(&v.as_str()) => v.clone_into(&mut e.tipo),
             "--tipo" => return Err(invalido(opcion, v, &crate::tarea::TIPOS.join(", "))),
diff --git a/crates/tc2/src/ciclo/liberar.rs b/crates/tc2/src/ciclo/liberar.rs
index f64293a..45e6352 100644
--- a/crates/tc2/src/ciclo/liberar.rs
+++ b/crates/tc2/src/ciclo/liberar.rs
@@ -29,7 +29,7 @@
 use std::cell::RefCell;
 
 use super::operacion::{ABIERTAS, Lanzada};
-use crate::estado::{Cambio, Estado, Fallo, Operacion, por_estado, tarea_de};
+use crate::estado::{AnotacionConDueno, Cambio, Estado, Fallo, Operacion, por_estado, tarea_de};
 use crate::i18n::Mensaje;
 use crate::i18n::catalogo::ciclo as txt;
 use crate::identidad::Quien;
@@ -232,7 +232,7 @@ impl Operacion for Liberar {
     }
 
     fn anotacion(&self) -> String {
-        format!("{}|{}", self.d.base.anotacion(LIBERAR), self.d.cola())
+        self.d.base.anotacion(LIBERAR, &self.d.cola())
     }
 
     fn exige_red(&self) -> bool {
@@ -309,13 +309,13 @@ impl Operacion for Reasignar {
     }
 
     fn anotacion(&self) -> String {
-        format!(
-            "{}|{}|{}|{}",
-            self.d.base.anotacion(REASIGNAR),
+        let resto = format!(
+            "{}|{}|{}",
             self.a.id().unwrap_or_default(),
             self.a.login().unwrap_or_default(),
             self.d.cola()
-        )
+        );
+        self.d.base.anotacion(REASIGNAR, &resto)
     }
 
     fn exige_red(&self) -> bool {
@@ -349,12 +349,13 @@ impl Operacion for Reasignar {
 /// o ->|<dias>|<ahora>`. Exigen red y no quedan pendientes; si se
 /// re-ejecutaran, sin persona delante: `decidido_por: agente`.
 pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
-    let partes: Vec<&str> = a.split('|').collect();
-    let (nombre, id) = (*partes.first()?, *partes.get(1)?);
-    if !crate::git::es_id(id) {
-        return None;
-    }
-    let asignado = Asignado::persona(partes.get(2)?.parse().ok()?, partes.get(3)?)?;
+    let AnotacionConDueno {
+        orden: nombre,
+        id,
+        asignado,
+        resto,
+    } = AnotacionConDueno::leer(a)?;
+    let partes: Vec<&str> = resto.map(|r| r.split('|').collect()).unwrap_or_default();
     let cola = |p: &[&str]| -> Option<(Option<u64>, u64, u64)> {
         let [actividad, dias, ahora] = p else {
             return None;
@@ -367,14 +368,14 @@ pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
     };
     match nombre {
         LIBERAR => {
-            let (actividad, dias, ahora) = cola(partes.get(4..)?)?;
+            let (actividad, dias, ahora) = cola(&partes)?;
             Some(Box::new(Liberar {
                 d: Dejar::nueva(id, asignado, "agente", ahora, actividad, dias),
             }))
         }
         REASIGNAR => {
-            let destino = Asignado::persona(partes.get(4)?.parse().ok()?, partes.get(5)?)?;
-            let (actividad, dias, ahora) = cola(partes.get(6..)?)?;
+            let destino = Asignado::persona(partes.first()?.parse().ok()?, partes.get(1)?)?;
+            let (actividad, dias, ahora) = cola(partes.get(2..)?)?;
             Some(Box::new(Reasignar {
                 d: Dejar::nueva(id, asignado, "agente", ahora, actividad, dias),
                 a: destino,
diff --git a/crates/tc2/src/ciclo/mod.rs b/crates/tc2/src/ciclo/mod.rs
index d619a15..1559b38 100644
--- a/crates/tc2/src/ciclo/mod.rs
+++ b/crates/tc2/src/ciclo/mod.rs
@@ -39,8 +39,8 @@ pub(crate) mod prueba;
 
 use std::path::Path;
 
-use crate::contrato::{Informe, Resultado, Valor};
-use crate::estado::{self, Fallo, Operacion};
+use crate::contrato::{Informe, Resultado};
+use crate::estado::{self, Operacion, como_orden, con_dueno};
 use crate::git::{self, Git};
 use crate::i18n::catalogo::ciclo as txt;
 use crate::i18n::{Mensaje, Texto};
@@ -138,7 +138,7 @@ pub fn dejar_en(e: &Entorno<'_>, id: &str, destino: Option<&str>) -> Informe {
     let i = match paso_dejar(e, id, destino) {
         Ok(i) | Err(i) => i,
     };
-    de_la_orden(i, nombre, id)
+    como_orden(i, nombre, "id", id)
 }
 
 /// La persona de la lista de la base con ese login (sin distinguir
@@ -161,21 +161,12 @@ fn destino_de(e: &Entorno<'_>, login: &str) -> Result<Asignado, Informe> {
 }
 
 /// Umbral de dias sin actividad (clave `dias_sin_actividad` de la
-/// configuracion de la base, D1a). Sin base o sin configuracion, el de por
-/// defecto; una configuracion que no vale (o que no se pudo leer) falla con
-/// su informe a nombre de `op`, como `identidad::clave_plataforma`:
-/// no se cae en silencio al de por defecto (M-4 de la ronda 1).
+/// configuracion de la base, D1a), con la politica de
+/// [`crate::config::de_base_o_suelo`]: sin base o sin configuracion, el del
+/// suelo (el de por defecto, [`DIAS_POR_DEFECTO`]); una configuracion que no
+/// vale falla con su informe a nombre de `op` (M-4 de la ronda 1).
 pub fn umbral_de_inactividad(g: &Git, dir: &Path, op: &str) -> Result<u64, Informe> {
-    match crate::config::de_base(g, dir) {
-        Ok(c) => Ok(u64::from(c.dias_sin_actividad)),
-        Err(i) if matches!(i.valor("motivo"), Some("sin-base" | "sin-config")) => {
-            Ok(DIAS_POR_DEFECTO)
-        }
-        Err(mut i) => {
-            op.clone_into(&mut i.comando);
-            Err(i)
-        }
-    }
+    crate::config::de_base_o_suelo(g, dir, op).map(|c| u64::from(c.dias_sin_actividad))
 }
 
 /// Las dos operaciones, para tratarlas igual.
@@ -244,16 +235,7 @@ fn paso_dejar(e: &Entorno<'_>, id: &str, destino: Option<&str>) -> Result<Inform
     // En seco sobre lo leido: los rechazos, sin esperar al push.
     let cambia = match op.op().aplicar(&vista) {
         Ok(c) => !c.is_empty(),
-        Err(Fallo::Rechazada {
-            motivo,
-            diagnostico,
-        }) => {
-            let i = Informe::nuevo(nombre, Resultado::Rechazada)
-                .con("motivo", motivo)
-                .diagnostico(diagnostico);
-            return Err(con_rechazo(i, op.dejar()));
-        }
-        Err(Fallo::Error(d)) => return Err(Informe::nuevo(nombre, Resultado::Error).diagnostico(d)),
+        Err(f) => return Err(con_rechazo(f.informe(nombre), op.dejar())),
     };
     // El trabajo sin publicar del worktree de la tarea en este clon (D4):
     // antes de escribir nada.
@@ -270,7 +252,7 @@ fn paso_dejar(e: &Entorno<'_>, id: &str, destino: Option<&str>) -> Result<Inform
                     .con("que", m),
             ));
     }
-    let t = escribir(e, op.op());
+    let t = e.escribir(op.op());
     match t.resultado {
         Resultado::Ok | Resultado::SinCambios => {
             let mut i = t;
@@ -329,18 +311,18 @@ pub fn descartar_en(e: &Entorno<'_>, id: &str, motivo: &str) -> Informe {
     let i = match paso_descartar(e, id, motivo) {
         Ok(i) | Err(i) => i,
     };
-    de_la_orden(i, DESCARTAR, id)
+    como_orden(i, DESCARTAR, "id", id)
 }
 
 /// `pausa` (o `reanudar`) de la tarea `id` en el repo de `e`.
 pub fn pausa_en(e: &Entorno<'_>, id: &str, reanudar: bool) -> Informe {
     let op = match (e.quien)() {
         Ok(q) => Pausar::nueva(id, &q, reanudar, e.ahora),
-        Err(i) => return de_la_orden(i, if reanudar { REANUDAR } else { PAUSA }, id),
+        Err(i) => return como_orden(i, if reanudar { REANUDAR } else { PAUSA }, "id", id),
     };
-    let t = escribir(e, &op);
+    let t = e.escribir(&op);
     let i = con_dueno(t, op.dueno());
-    de_la_orden(i, op.nombre(), id)
+    como_orden(i, op.nombre(), "id", id)
 }
 
 fn paso_descartar(e: &Entorno<'_>, id: &str, motivo: &str) -> Result<Informe, Informe> {
@@ -356,7 +338,7 @@ fn paso_descartar(e: &Entorno<'_>, id: &str, motivo: &str) -> Result<Informe, In
     }
     let quien = (e.quien)()?;
     let op = Descartar::nueva(id, &quien, motivo, e.ahora);
-    let t = escribir(e, &op);
+    let t = e.escribir(&op);
     match t.resultado {
         // Publicado, o ya descartada (repetir termina de quitar el worktree).
         Resultado::Ok | Resultado::SinCambios => Ok(con_worktree(t, g, &principal, e.dir, id)),
@@ -364,29 +346,6 @@ fn paso_descartar(e: &Entorno<'_>, id: &str, motivo: &str) -> Result<Informe, In
     }
 }
 
-/// La operacion por el punto unico de escritura, con el `ROADMAP.md`
-/// regenerado en el mismo commit.
-fn escribir(e: &Entorno<'_>, op: &dyn Operacion) -> Informe {
-    let personas = crate::roadmap::personas(e.git, e.dir);
-    let generar = crate::roadmap::generador(&personas);
-    let ctx = estado::Contexto::nuevo(e.git, e.dir, e.catalogo, Some(&generar));
-    estado::transaccion(&ctx, op)
-}
-
-/// Un rechazo con lo que sabe la operacion: el login de quien la lleva o
-/// el arreglo de una tarea de la 0.16.
-fn con_dueno(mut i: Informe, dueno: Option<String>) -> Informe {
-    if i.resultado != Resultado::Rechazada {
-        return i;
-    }
-    match (i.valor("motivo"), dueno) {
-        (Some("de-otra-persona"), Some(l)) => i = i.con("asignado", l),
-        (Some("formato-1"), _) => i = i.con("arreglo", "tc2 actualizar"),
-        _ => {}
-    }
-    i
-}
-
 /// Tras el descarte, el worktree de la tarea (D3): `worktree=quitado`,
 /// `ninguno` (no habia) o `conservado` con `worktree-motivo=` (cambios sin
 /// guardar, commits sin publicar, HEAD suelto, lanzado desde dentro o un
@@ -419,20 +378,6 @@ fn dentro_de(dir: &Path, wt: &Path) -> bool {
     }
 }
 
-/// El informe como de la orden: el comando, `id=` el primero y sin la clave
-/// `operacion` de la transaccion. Un `sinred` sin `pendiente` (el `fetch`
-/// previo de `descartar`) dice `pendiente=no`.
-fn de_la_orden(mut i: Informe, nombre: &str, id: &str) -> Informe {
-    nombre.clone_into(&mut i.comando);
-    i.claves.retain(|(k, _)| k != "operacion" && k != "id");
-    i.claves
-        .insert(0, ("id".to_owned(), Valor::Texto(id.to_owned())));
-    if i.resultado == Resultado::Sinred && i.valor("pendiente").is_none() {
-        i = i.con("pendiente", "no");
-    }
-    i
-}
-
 #[cfg(test)]
 mod tests {
     use super::*;
@@ -572,6 +517,48 @@ mod tests {
         Ok(())
     }
 
+    /// A8-10: la politica comun de `plan` y de `liberar`: sin configuracion
+    /// en la base, el suelo; con una que no vale, su rechazo a nombre de la
+    /// orden que la pidio.
+    #[test]
+    fn config_de_la_base_o_suelo_a_nombre_de_la_orden() -> R<()> {
+        let Repo {
+            _borrar,
+            raiz,
+            clon,
+            git: g,
+        } = repo_con_remoto(
+            "ciclo-suelo",
+            &[(
+                "README.md",
+                "linea 1
+",
+            )],
+            &[],
+        )?;
+        assert_eq!(
+            crate::config::de_base_o_suelo(&g, &clon, "plan").map_err(|i| i.linea()),
+            Ok(crate::config::Config::suelo())
+        );
+        std::fs::create_dir_all(clon.join(".taskcode"))?;
+        std::fs::write(
+            clon.join(crate::config::RUTA),
+            "---
+formato: 9
+",
+        )?;
+        sh(&raiz, &clon, &["add", "."])?;
+        sh(&raiz, &clon, &["commit", "-q", "-m", "config"])?;
+        sh(&raiz, &clon, &["push", "-q", "origin", "develop"])?;
+        sh(&raiz, &clon, &["fetch", "-q", "origin"])?;
+        let i = crate::config::de_base_o_suelo(&g, &clon, "plan")
+            .err()
+            .ok_or("acepto formato 9")?;
+        assert_eq!(i.comando, "plan", "{}", i.linea());
+        assert_eq!(i.resultado, Resultado::Rechazada, "{}", i.linea());
+        Ok(())
+    }
+
     /// M-4 de la ronda 1: sin configuracion, el umbral de por defecto; con
     /// una que no vale, el rechazo de la configuracion a nombre de la orden
     /// (no el de por defecto en silencio); con una valida, su valor.
diff --git a/crates/tc2/src/ciclo/operacion.rs b/crates/tc2/src/ciclo/operacion.rs
index c52c8d2..3206115 100644
--- a/crates/tc2/src/ciclo/operacion.rs
+++ b/crates/tc2/src/ciclo/operacion.rs
@@ -19,7 +19,9 @@
 
 use std::cell::RefCell;
 
-use crate::estado::{Cambio, Estado, Fallo, Operacion, de_otra_persona, por_estado, tarea_de};
+use crate::estado::{
+    AnotacionConDueno, Cambio, Estado, Fallo, Operacion, por_estado, tarea_propia,
+};
 use crate::grafo::DESCARTADA;
 use crate::i18n::catalogo::ciclo as txt;
 use crate::identidad::Quien;
@@ -75,19 +77,14 @@ impl Lanzada {
     /// La tarea de `estado`, si quien lanza `orden` puede cambiarla (D2): la
     /// lleva el o nadie.
     fn tarea(&self, estado: &dyn Estado, orden: &str) -> Result<(Tarea, Cuerpo), Fallo> {
-        *self.dueno.borrow_mut() = None;
-        let (t, c) = tarea_de(estado, &self.id)?;
-        if let Some((fallo, dueno)) = de_otra_persona(
+        tarea_propia(
+            estado,
             &self.id,
-            t.asignado_a.as_ref(),
             &self.asignado,
             orden,
             ("de-otra-persona", &txt::DE_OTRA_PERSONA),
-        ) {
-            *self.dueno.borrow_mut() = dueno;
-            return Err(fallo);
-        }
-        Ok((t, c))
+            &self.dueno,
+        )
     }
 
     /// `terminada`, `descartada` u otro estado que no es de [`ABIERTAS`].
@@ -112,14 +109,15 @@ impl Lanzada {
         )]
     }
 
-    /// `<nombre>|TASK-NNN|<id>|<login>`.
-    pub(super) fn anotacion(&self, nombre: &str) -> String {
-        format!(
-            "{nombre}|{}|{}|{}",
-            self.id,
-            self.asignado.id().unwrap_or_default(),
-            self.asignado.login().unwrap_or_default()
-        )
+    /// `<nombre>|TASK-NNN|<id>|<login>|<resto>` ([`AnotacionConDueno`]).
+    pub(super) fn anotacion(&self, nombre: &str, resto: &str) -> String {
+        AnotacionConDueno {
+            orden: nombre,
+            id: &self.id,
+            asignado: self.asignado.clone(),
+            resto: Some(resto),
+        }
+        .escribir()
     }
 }
 
@@ -154,7 +152,7 @@ impl Operacion for Descartar {
     }
 
     fn anotacion(&self) -> String {
-        format!("{}|{}", self.base.anotacion(DESCARTAR), self.motivo)
+        self.base.anotacion(DESCARTAR, &self.motivo)
     }
 
     fn exige_red(&self) -> bool {
@@ -212,7 +210,8 @@ impl Operacion for Pausar {
     fn anotacion(&self) -> String {
         // Con el instante: si queda pendiente sin red, la fila lleva la hora
         // de la pausa y no la de su publicacion (F1-17 mide las pausas).
-        format!("{}|{}", self.base.anotacion(self.fase()), self.base.ahora)
+        self.base
+            .anotacion(self.fase(), &self.base.ahora.to_string())
     }
 
     fn exige_red(&self) -> bool {
@@ -239,15 +238,13 @@ impl Operacion for Pausar {
 /// sin persona delante: `decidido_por: agente`, como en
 /// [`crate::toma::desde_anotacion`].
 pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
-    let mut partes = a.splitn(5, '|');
-    let nombre = partes.next()?;
-    let id = partes.next()?;
-    let cuenta: u64 = partes.next()?.parse().ok()?;
-    let asignado = Asignado::persona(cuenta, partes.next()?)?;
-    let resto = partes.next()?;
-    if !crate::git::es_id(id) {
-        return None;
-    }
+    let AnotacionConDueno {
+        orden: nombre,
+        id,
+        asignado,
+        resto,
+    } = AnotacionConDueno::leer(a)?;
+    let resto = resto?;
     match nombre {
         DESCARTAR if motivo_valido(resto) => Some(Box::new(Descartar {
             base: Lanzada::nueva(id, asignado, "agente", crate::identidad::ahora()),
diff --git a/crates/tc2/src/config/mod.rs b/crates/tc2/src/config/mod.rs
index f612467..d6e1c18 100644
--- a/crates/tc2/src/config/mod.rs
+++ b/crates/tc2/src/config/mod.rs
@@ -528,6 +528,23 @@ pub fn de_base(git: &Git, dir: &Path) -> Result<Config, Informe> {
     leer(&texto).map_err(|e| e.informe())
 }
 
+/// La politica de lectura de la base para una orden (A8-10): sin base o sin
+/// configuracion, el [`Config::suelo`]; una que no vale (o que no se pudo
+/// leer) falla con su informe a nombre de `orden`, sin caer en silencio al
+/// suelo (M-4 de la ronda 1 de TASK-081).
+pub fn de_base_o_suelo(git: &Git, dir: &Path, orden: &str) -> Result<Config, Informe> {
+    match de_base(git, dir) {
+        Ok(c) => Ok(c),
+        Err(i) if matches!(i.valor("motivo"), Some("sin-base" | "sin-config")) => {
+            Ok(Config::suelo())
+        }
+        Err(mut i) => {
+            orden.clone_into(&mut i.comando);
+            Err(i)
+        }
+    }
+}
+
 /// Lee la configuracion del arbol de trabajo de `raiz` (F1-8 decision 5:
 /// la orden que se confia y se ejecuta es la del arbol, no la de la base).
 /// `Err` lleva el informe `config rechazada` (sin fichero, `sin-config`), o
diff --git a/crates/tc2/src/contrato.rs b/crates/tc2/src/contrato.rs
index de49cbb..259ebc7 100644
--- a/crates/tc2/src/contrato.rs
+++ b/crates/tc2/src/contrato.rs
@@ -306,6 +306,18 @@ impl LineaAccion<'_> {
     }
 }
 
+/// Motivo de una opcion sin valor al final de los argumentos.
+pub(crate) const FALTA_VALOR: &str = "falta-valor";
+
+/// El valor de la opcion que se acaba de leer: el siguiente argumento, o el
+/// `uso` de la orden con [`FALTA_VALOR`] si no hay (copias de TASK-140).
+pub(crate) fn valor_de<'a>(
+    it: &mut impl Iterator<Item = &'a String>,
+    uso: impl FnOnce(&str) -> Informe,
+) -> Result<String, Informe> {
+    it.next().cloned().ok_or_else(|| uso(FALTA_VALOR))
+}
+
 /// Hash abreviado a 12 caracteres, como fija el contrato.
 pub fn corto(hash: &str) -> String {
     hash.chars().take(12).collect()
@@ -354,6 +366,21 @@ pub(crate) fn cadena_json(t: &str) -> String {
 mod tests {
     use super::*;
 
+    /// Copias de `valor` (TASK-140): el siguiente argumento, o el `uso` de la
+    /// orden con `falta-valor` si no queda ninguno.
+    #[test]
+    fn valor_de_la_opcion_o_falta_valor() {
+        let args = ["--rutas".to_owned(), "a.txt".to_owned()];
+        let uso = |m: &str| Informe::nuevo("seleccion", Resultado::Uso).con("motivo", m);
+        let mut it = args.iter();
+        assert_eq!(it.next().map(String::as_str), Some("--rutas"));
+        assert_eq!(valor_de(&mut it, uso), Ok("a.txt".to_owned()));
+        assert_eq!(
+            valor_de(&mut it, uso).map_err(|i| i.linea()),
+            Err("seleccion uso motivo=falta-valor".to_owned())
+        );
+    }
+
     #[test]
     fn codigos_de_la_tabla_fijada_en_f2_3() {
         let tabla = [
diff --git a/crates/tc2/src/estado/dueno.rs b/crates/tc2/src/estado/dueno.rs
new file mode 100644
index 0000000..d2d0320
--- /dev/null
+++ b/crates/tc2/src/estado/dueno.rs
@@ -0,0 +1,220 @@
+//! Las operaciones con dueno de la rama de estado (grupo «operacion con
+//! dueno» de la auditoria 4D): lo que comparten `review`, `plan`, `approve`,
+//! `start`, `descartar`, `pausa`, `reanudar`, `liberar` y `reasignar` sobre
+//! una tarea que lleva una persona.
+
+use std::cell::RefCell;
+
+use super::lectura::Estado;
+use super::tareas::{de_otra_persona, tarea_de};
+use super::transaccion::Fallo;
+use crate::contrato::{Informe, Resultado};
+use crate::i18n::Texto;
+use crate::tarea::{Asignado, Cuerpo, Tarea};
+
+/// La tarea `id` de `estado`, si `quien` puede cambiarla con `orden` (A8-07,
+/// D2 de TASK-034): la lleva el o nadie. El preambulo comun de
+/// [`super::Operacion::aplicar`] en las operaciones con dueno: limpia
+/// `dueno` (la operacion se re-ejecuta en cada intento), lee la tarea y, si
+/// la lleva otra persona, guarda su login en `dueno` y la rechaza con `ajena`
+/// (motivo y texto de la orden), o con `asignado-heredado` si es un texto de
+/// la 0.16.
+pub fn tarea_propia(
+    estado: &dyn Estado,
+    id: &str,
+    quien: &Asignado,
+    orden: &str,
+    ajena: (&str, &'static Texto),
+    dueno: &RefCell<Option<String>>,
+) -> Result<(Tarea, Cuerpo), Fallo> {
+    *dueno.borrow_mut() = None;
+    let (t, c) = tarea_de(estado, id)?;
+    if let Some((fallo, login)) = de_otra_persona(id, t.asignado_a.as_ref(), quien, orden, ajena) {
+        *dueno.borrow_mut() = login;
+        return Err(fallo);
+    }
+    Ok((t, c))
+}
+
+/// La anotacion de una operacion con dueno (A8-19):
+/// `<orden>|<id>|<cuenta>|<login>[|<resto>]`. Se escribe y se lee solo
+/// aqui; cada operacion interpreta su `resto` (el instante, el motivo, el
+/// carril...), que puede llevar `|`.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct AnotacionConDueno<'a> {
+    /// Nombre de la operacion (`start`, `review`, `descartar`...).
+    pub orden: &'a str,
+    /// La tarea (en `approve-hito`, el numero del hito).
+    pub id: &'a str,
+    /// Quien la lanza.
+    pub asignado: Asignado,
+    /// Lo que anade la operacion, si anade algo.
+    pub resto: Option<&'a str>,
+}
+
+impl<'a> AnotacionConDueno<'a> {
+    /// La linea de la anotacion.
+    pub fn escribir(&self) -> String {
+        let mut s = format!(
+            "{}|{}|{}|{}",
+            self.orden,
+            self.id,
+            self.asignado.id().unwrap_or_default(),
+            self.asignado.login().unwrap_or_default()
+        );
+        if let Some(r) = self.resto {
+            s.push('|');
+            s.push_str(r);
+        }
+        s
+    }
+
+    /// La anotacion de `a`, si tiene la forma: un ID de tarea, una cuenta
+    /// numerica y un login que valen ([`Asignado::persona`]). `None` si no.
+    pub fn leer(a: &'a str) -> Option<Self> {
+        let mut partes = a.splitn(5, '|');
+        let orden = partes.next()?;
+        let id = partes.next()?;
+        let cuenta: u64 = partes.next()?.parse().ok()?;
+        let asignado = Asignado::persona(cuenta, partes.next()?)?;
+        crate::git::es_id(id).then_some(Self {
+            orden,
+            id,
+            asignado,
+            resto: partes.next(),
+        })
+    }
+}
+
+/// Un rechazo con lo que sabe la operacion (A8-09): el login de quien lleva
+/// la tarea (`asignado=`) o el arreglo de una tarea de la 0.16. Cualquier
+/// otro resultado queda como esta.
+pub fn con_dueno(mut i: Informe, dueno: Option<String>) -> Informe {
+    if i.resultado != Resultado::Rechazada {
+        return i;
+    }
+    match (i.valor("motivo"), dueno) {
+        (Some("de-otra-persona"), Some(l)) => i = i.con("asignado", l),
+        (Some("formato-1"), _) => i = i.con("arreglo", "tc2 actualizar"),
+        _ => {}
+    }
+    i
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::ciclo::prueba::{Extra, md};
+    use crate::estado::prueba::estado;
+
+    /// A8-19: ida y vuelta de la operacion entera, sin `resto`, con uno y con
+    /// uno que lleva `|`; las anotaciones malas no se leen.
+    #[test]
+    fn anotacion_con_dueno_ida_y_vuelta() {
+        let ana = Asignado::persona(7, "ana").unwrap_or_else(|| unreachable!());
+        for resto in [None, Some("1794910830"), Some(""), Some("por | esto")] {
+            let a = AnotacionConDueno {
+                orden: "descartar",
+                id: "TASK-001",
+                asignado: ana.clone(),
+                resto,
+            };
+            let texto = a.escribir();
+            assert_eq!(AnotacionConDueno::leer(&texto), Some(a), "{texto}");
+        }
+        let a = AnotacionConDueno {
+            orden: "start",
+            id: "TASK-012",
+            asignado: ana,
+            resto: Some("x|y"),
+        };
+        assert_eq!(a.escribir(), "start|TASK-012|7|ana|x|y");
+        for mala in [
+            "start|TASK-1|7|ana",
+            "start|TASK-001|x|ana",
+            "start|TASK-001|0|ana",
+            "start|TASK-001|7|",
+            "start|TASK-001|7",
+            "start",
+        ] {
+            assert_eq!(AnotacionConDueno::leer(mala), None, "{mala}");
+        }
+    }
+
+    /// A8-07: la propia o sin dueno se lee; la ajena guarda el login y se
+    /// rechaza con el motivo de la orden; la heredada y la inexistente no
+    /// dejan dueno, tampoco el de un intento anterior.
+    #[test]
+    fn tarea_propia_ajena_heredada_e_inexistente() {
+        let ana = Asignado::persona(7, "ana").unwrap_or_else(|| unreachable!());
+        let beto = Asignado::persona(9, "beto");
+        let heredado = md("TASK-003", "en-curso", None, Extra::default())
+            .replace("asignado_a: null", "asignado_a: \"ana@x.com\"");
+        let s = estado(&[
+            (
+                "TASK-001",
+                md("TASK-001", "en-curso", Some(ana.clone()), Extra::default()),
+            ),
+            (
+                "TASK-002",
+                md("TASK-002", "en-curso", beto, Extra::default()),
+            ),
+            ("TASK-003", heredado),
+            (
+                "TASK-004",
+                md("TASK-004", "en-curso", None, Extra::default()),
+            ),
+        ]);
+        let ajena = ("de-otra", &crate::i18n::catalogo::plan::DE_OTRA_PERSONA);
+        let dueno = RefCell::new(None);
+        let leer = |id: &str| tarea_propia(&s, id, &ana, "plan", ajena, &dueno);
+        let motivo = |r: Result<(Tarea, Cuerpo), Fallo>| match r {
+            Err(Fallo::Rechazada { motivo, .. }) => motivo,
+            otro => format!("{otro:?}"),
+        };
+        assert_eq!(
+            leer("TASK-001").map(|(t, _)| t.id).ok().as_deref(),
+            Some("TASK-001")
+        );
+        assert_eq!(*dueno.borrow(), None);
+        assert_eq!(motivo(leer("TASK-002")), "de-otra");
+        assert_eq!(dueno.borrow().as_deref(), Some("beto"));
+        assert_eq!(motivo(leer("TASK-003")), "asignado-heredado");
+        assert_eq!(*dueno.borrow(), None);
+        *dueno.borrow_mut() = Some("beto".to_owned());
+        assert_eq!(motivo(leer("TASK-009")), "no-existe");
+        assert_eq!(*dueno.borrow(), None);
+        *dueno.borrow_mut() = Some("beto".to_owned());
+        assert!(leer("TASK-004").is_ok());
+        assert_eq!(*dueno.borrow(), None);
+    }
+
+    /// A8-09: `asignado=` solo en un rechazo `de-otra-persona` con dueno; el
+    /// arreglo de `formato-1`; lo demas, sin tocar.
+    #[test]
+    fn rechazo_con_lo_que_sabe_la_operacion() {
+        let rechazo = |m: &str| Informe::nuevo("plan", Resultado::Rechazada).con("motivo", m);
+        let ana = || Some("ana".to_owned());
+        assert_eq!(
+            con_dueno(rechazo("de-otra-persona"), ana()).linea(),
+            "plan rechazada motivo=de-otra-persona asignado=ana"
+        );
+        assert_eq!(
+            con_dueno(rechazo("de-otra-persona"), None).linea(),
+            "plan rechazada motivo=de-otra-persona"
+        );
+        assert_eq!(
+            con_dueno(rechazo("formato-1"), ana()).linea(),
+            "plan rechazada motivo=formato-1 arreglo=tc2%20actualizar"
+        );
+        assert_eq!(
+            con_dueno(rechazo("no-existe"), ana()).linea(),
+            "plan rechazada motivo=no-existe"
+        );
+        let error = Informe::nuevo("plan", Resultado::Error).con("motivo", "de-otra-persona");
+        assert_eq!(
+            con_dueno(error, ana()).linea(),
+            "plan error motivo=de-otra-persona"
+        );
+    }
+}
diff --git a/crates/tc2/src/estado/mod.rs b/crates/tc2/src/estado/mod.rs
index d11752c..31f7255 100644
--- a/crates/tc2/src/estado/mod.rs
+++ b/crates/tc2/src/estado/mod.rs
@@ -25,6 +25,7 @@
 //! codigos de la tabla de `contrato.rs`.
 
 mod cerrojo;
+mod dueno;
 mod frescura;
 mod lectura;
 mod montaje;
@@ -36,12 +37,15 @@ mod transaccion;
 
 pub use crate::git::{Cambio, Sello};
 pub use cerrojo::Cerrojo;
+pub use dueno::{AnotacionConDueno, con_dueno, tarea_propia};
 pub use frescura::{Frescura, Marca, Remoto, hace, sincronizado_texto};
 pub(crate) use lectura::referencia_de_lectura;
 pub use lectura::{Estado, Local, Vista, leer, leer_fresco, local_de, sincronizado_en};
 pub use montaje::{Previsto, montar, montar_con, previsto};
 pub use tareas::{Leida, de_otra_persona, es_tarea_md, por_estado, tarea_de, tareas};
-pub use transaccion::{Catalogo, Contexto, Fallo, Operacion, Politica, Roadmap, transaccion};
+pub use transaccion::{
+    Catalogo, Contexto, Fallo, Operacion, Politica, Roadmap, como_orden, transaccion,
+};
 
 /// Rama huerfana del estado (ADR F1-1; nombre en [`crate::huella`]).
 pub const RAMA: &str = crate::huella::RAMA_ESTADO;
diff --git a/crates/tc2/src/estado/transaccion.rs b/crates/tc2/src/estado/transaccion.rs
index e43d469..7074cda 100644
--- a/crates/tc2/src/estado/transaccion.rs
+++ b/crates/tc2/src/estado/transaccion.rs
@@ -17,7 +17,7 @@ use super::frescura::{Marca, Traida, rama_reescrita, traer_estado};
 use super::lectura::{Estado, Vista};
 use super::pendientes::{self, ANOTACION, Destino};
 use super::{CARPETA, OP, RAMA};
-use crate::contrato::{Informe, Resultado, corto};
+use crate::contrato::{Informe, Resultado, Valor, corto};
 use crate::git::estado::{
     Sello, construir_commit, construir_commit_sellado, dir_comun, hay_remoto,
 };
@@ -52,6 +52,21 @@ impl Fallo {
             diagnostico: diagnostico.into(),
         }
     }
+
+    /// El informe de la orden `comando` (A8-06): `rechazada motivo=` con el
+    /// diagnostico, o `error` con el diagnostico. Cada orden anade sus claves
+    /// encima.
+    pub fn informe(self, comando: &str) -> Informe {
+        match self {
+            Self::Rechazada {
+                motivo,
+                diagnostico,
+            } => Informe::nuevo(comando, Resultado::Rechazada)
+                .con("motivo", motivo)
+                .diagnostico(diagnostico),
+            Self::Error(d) => Informe::nuevo(comando, Resultado::Error).diagnostico(d),
+        }
+    }
 }
 
 /// Una operacion sobre el estado. Se re-ejecuta en cada intento: tiene que
@@ -173,6 +188,23 @@ fn informe(r: Resultado, op: &dyn Operacion) -> Informe {
     Informe::nuevo(OP, r).con("operacion", op.nombre())
 }
 
+/// El informe de la transaccion como de la orden `nombre` (A8-08): el
+/// comando, `clave=valor` el primero (`id=` de una tarea o `hito=` de
+/// `approve --hito`) y sin la clave `operacion` ni el `id=` de la
+/// transaccion. Un `sinred` sin `pendiente` dice `pendiente=no`: las ordenes
+/// que lo usan exigen red, o el `sinred` viene de su `fetch` previo.
+pub fn como_orden(mut i: Informe, nombre: &str, clave: &str, valor: &str) -> Informe {
+    nombre.clone_into(&mut i.comando);
+    i.claves
+        .retain(|(k, _)| k != "operacion" && k != "id" && k != clave);
+    i.claves
+        .insert(0, (clave.to_owned(), Valor::Texto(valor.to_owned())));
+    if i.resultado == Resultado::Sinred && i.valor("pendiente").is_none() {
+        i = i.con("pendiente", "no");
+    }
+    i
+}
+
 fn error(diagnostico: impl Into<Mensaje>) -> Informe {
     Informe::nuevo(OP, Resultado::Error).diagnostico(diagnostico)
 }
@@ -639,9 +671,53 @@ fn adelantar(git: &Git, principal: &Path, sha: &str) -> Result<Tareas, Informe>
 
 #[cfg(test)]
 mod tests {
-    use super::espera;
+    use super::{Fallo, como_orden, espera};
+    use crate::contrato::{Informe, Resultado};
+    use crate::i18n::Mensaje;
     use std::time::Duration;
 
+    /// A8-08: la linea de la transaccion como de la orden, con `clave=` el
+    /// primero, sin `operacion` ni el `id` de la transaccion, y
+    /// `pendiente=no` solo en un `sinred` que no lo dice.
+    #[test]
+    fn informe_de_la_transaccion_como_de_la_orden() {
+        let ok = Informe::nuevo("estado", Resultado::Ok)
+            .con("operacion", "start")
+            .con("id", "TASK-001")
+            .con("commit", "abc1234");
+        assert_eq!(
+            como_orden(ok, "start", "id", "TASK-002").linea(),
+            "start ok id=TASK-002 commit=abc1234"
+        );
+        let hito = Informe::nuevo("estado", Resultado::Sinred)
+            .con("operacion", "approve")
+            .con("id", "TASK-009")
+            .con("hito", "9");
+        assert_eq!(
+            como_orden(hito, "approve", "hito", "3").linea(),
+            "approve sinred hito=3 pendiente=no"
+        );
+        let dicho = Informe::nuevo("estado", Resultado::Sinred).con("pendiente", "si");
+        assert_eq!(
+            como_orden(dicho, "plan", "id", "TASK-001").linea(),
+            "plan sinred id=TASK-001 pendiente=si"
+        );
+    }
+
+    /// A8-06: el rechazo y el error de la rama de estado como informe de la
+    /// orden, con el diagnostico en los dos.
+    #[test]
+    fn fallo_como_informe_de_la_orden() {
+        let d = Mensaje::externo("haz esto");
+        let r = Fallo::rechazada("tomada", d.clone()).informe("start");
+        assert_eq!(r.linea(), "start rechazada motivo=tomada");
+        assert_eq!(r.resultado, Resultado::Rechazada);
+        assert_eq!(r.diagnostico, Some(d.clone()));
+        let e = Fallo::Error(d.clone()).informe("roadmap");
+        assert_eq!(e.linea(), "roadmap error");
+        assert_eq!(e.diagnostico, Some(d));
+    }
+
     #[test]
     fn espera_exponencial_con_jitter_entre_la_mitad_y_el_total() {
         let base = Duration::from_millis(100);
diff --git a/crates/tc2/src/estimacion/mod.rs b/crates/tc2/src/estimacion/mod.rs
index 797f208..1da471d 100644
--- a/crates/tc2/src/estimacion/mod.rs
+++ b/crates/tc2/src/estimacion/mod.rs
@@ -238,20 +238,21 @@ fn rellenar(e: &Entorno<'_>) -> Informe {
     i
 }
 
+/// Un rechazo de la lectura dice `id=` el primero, antes de `motivo=`; un
+/// `error` queda como esta.
+fn con_id_si_rechazada(mut i: Informe, id: &str) -> Informe {
+    if i.resultado == Resultado::Rechazada {
+        i.claves
+            .insert(0, ("id".to_owned(), Valor::Texto(id.to_owned())));
+    }
+    i
+}
+
 /// La tarea `id` leida del estado conocido (sin `fetch`: solo lectura), con
 /// todos los `tarea.md` y los commits.
 fn leer(e: &Entorno<'_>, id: &str) -> Result<(Tarea, Cuerpo, Historico, Commits), Informe> {
     let vista = estado::leer(e.git, e.dir).map_err(de_estimar)?;
-    let rechazo = |f: Fallo| match f {
-        Fallo::Rechazada {
-            motivo,
-            diagnostico,
-        } => Informe::nuevo(NOMBRE, Resultado::Rechazada)
-            .con("id", id)
-            .con("motivo", motivo)
-            .diagnostico(diagnostico),
-        Fallo::Error(d) => Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d),
-    };
+    let rechazo = |f: Fallo| con_id_si_rechazada(f.informe(NOMBRE), id);
     let (t, c) = estado::tarea_de(&vista, id).map_err(rechazo)?;
     let leidas = estado::tareas(&vista, &vista.rutas()).map_err(rechazo)?;
     let commits = commits_de(e.git, e.dir);
@@ -356,6 +357,23 @@ fn medir(e: &Entorno<'_>, id: &str) -> (Informe, Option<String>) {
 mod tests {
     use super::*;
 
+    /// A8-06: el rechazo de la lectura conserva `id=` antes de `motivo=`; el
+    /// error no lleva `id`.
+    #[test]
+    fn rechazo_de_lectura_con_id_el_primero() {
+        let m = Mensaje::externo("x");
+        let r = con_id_si_rechazada(
+            Fallo::rechazada("no-existe", m.clone()).informe(NOMBRE),
+            "TASK-007",
+        );
+        assert_eq!(
+            r.linea(),
+            format!("{NOMBRE} rechazada id=TASK-007 motivo=no-existe")
+        );
+        let e = con_id_si_rechazada(Fallo::Error(m).informe(NOMBRE), "TASK-007");
+        assert_eq!(e.linea(), format!("{NOMBRE} error"));
+    }
+
     #[test]
     fn decimales_con_punto_y_con_coma() {
         assert_eq!(decimales(0.6), ("0.60".to_owned(), "0,60".to_owned()));
diff --git a/crates/tc2/src/hallazgos/orden.rs b/crates/tc2/src/hallazgos/orden.rs
index f2d256d..f188bf7 100644
--- a/crates/tc2/src/hallazgos/orden.rs
+++ b/crates/tc2/src/hallazgos/orden.rs
@@ -63,7 +63,7 @@ use std::io::Read;
 use std::path::{Path, PathBuf};
 
 use crate::Salida;
-use crate::contrato::{Informe, Resultado};
+use crate::contrato::{Informe, Resultado, valor_de};
 use crate::i18n::catalogo::hallazgos as txt;
 use crate::i18n::{Idioma, Mensaje};
 use crate::verificacion::informe;
@@ -157,7 +157,7 @@ fn opciones(
         if unicas.contains(nombre) && !valores.is_empty() {
             return Err(uso("argumento-desconocido"));
         }
-        valores.push(it.next().cloned().ok_or_else(|| uso("falta-valor"))?);
+        valores.push(valor_de(&mut it, uso)?);
     }
     Ok(Opciones(o))
 }
diff --git a/crates/tc2/src/importar/mod.rs b/crates/tc2/src/importar/mod.rs
index 975b69f..908ffa1 100644
--- a/crates/tc2/src/importar/mod.rs
+++ b/crates/tc2/src/importar/mod.rs
@@ -402,20 +402,13 @@ fn simular(g: &git::Git, dir: &Path, op: &Importar, parecidas: &mut Vec<Parecida
                 .insert(0, ("simular".to_owned(), Valor::Texto("si".to_owned())));
             i
         }
-        Err(Fallo::Rechazada {
-            motivo,
-            diagnostico,
-        }) => {
-            let mut i = Informe::nuevo(NOMBRE, Resultado::Rechazada)
-                .con("simular", "si")
-                .con("motivo", motivo)
-                .diagnostico(diagnostico);
+        Err(f) => {
+            let mut i = f.informe(NOMBRE);
+            i.claves
+                .insert(0, ("simular".to_owned(), Valor::Texto("si".to_owned())));
             alta::referencia_a_uso(&mut i);
             i
         }
-        Err(Fallo::Error(d)) => Informe::nuevo(NOMBRE, Resultado::Error)
-            .con("simular", "si")
-            .diagnostico(d),
     }
 }
 
diff --git a/crates/tc2/src/plan/aprobar.rs b/crates/tc2/src/plan/aprobar.rs
index f95b16e..c1243ef 100644
--- a/crates/tc2/src/plan/aprobar.rs
+++ b/crates/tc2/src/plan/aprobar.rs
@@ -28,7 +28,8 @@ use super::plantillas as p;
 use super::roles::{self, Rol};
 use crate::config::Config;
 use crate::estado::{
-    Cambio, Estado, Fallo, Operacion, Sello, de_otra_persona, por_estado, tarea_de, tareas,
+    AnotacionConDueno, Cambio, Estado, Fallo, Operacion, Sello, de_otra_persona, por_estado,
+    tarea_de, tarea_propia, tareas,
 };
 use crate::grafo::{DESCARTADA, TERMINADA};
 use crate::i18n::catalogo::plan as txt;
@@ -154,12 +155,13 @@ impl Operacion for Aprobar {
     }
 
     fn anotacion(&self) -> String {
-        format!(
-            "{NOMBRE}|{}|{}|{}",
-            self.id,
-            self.asignado.id().unwrap_or_default(),
-            self.asignado.login().unwrap_or_default()
-        )
+        AnotacionConDueno {
+            orden: NOMBRE,
+            id: &self.id,
+            asignado: self.asignado.clone(),
+            resto: None,
+        }
+        .escribir()
     }
 
     fn exige_red(&self) -> bool {
@@ -359,12 +361,13 @@ impl Operacion for AprobarHito {
     }
 
     fn anotacion(&self) -> String {
-        format!(
-            "{NOMBRE_HITO}|{}|{}|{}",
-            self.hito,
-            self.asignado.id().unwrap_or_default(),
-            self.asignado.login().unwrap_or_default()
-        )
+        AnotacionConDueno {
+            orden: NOMBRE_HITO,
+            id: &self.hito.to_string(),
+            asignado: self.asignado.clone(),
+            resto: None,
+        }
+        .escribir()
     }
 
     fn exige_red(&self) -> bool {
@@ -504,12 +507,13 @@ impl Operacion for PedirCambios {
     }
 
     fn anotacion(&self) -> String {
-        format!(
-            "{NOMBRE_CAMBIOS}|{}|{}|{}",
-            self.id,
-            self.asignado.id().unwrap_or_default(),
-            self.asignado.login().unwrap_or_default()
-        )
+        AnotacionConDueno {
+            orden: NOMBRE_CAMBIOS,
+            id: &self.id,
+            asignado: self.asignado.clone(),
+            resto: None,
+        }
+        .escribir()
     }
 
     fn exige_red(&self) -> bool {
@@ -519,18 +523,14 @@ impl Operacion for PedirCambios {
 
     fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
         *self.ronda.borrow_mut() = None;
-        *self.dueno.borrow_mut() = None;
-        let (t, _) = tarea_de(estado, &self.id)?;
-        if let Some((fallo, dueno)) = de_otra_persona(
+        let (t, _) = tarea_propia(
+            estado,
             &self.id,
-            t.asignado_a.as_ref(),
             &self.asignado,
             NOMBRE,
             ("de-otra-persona", &txt::APROBAR_DE_OTRA_PERSONA),
-        ) {
-            *self.dueno.borrow_mut() = dueno;
-            return Err(fallo);
-        }
+            &self.dueno,
+        )?;
         if t.estado != EN_DISENO {
             return Err(por_estado(&self.id, &t.estado, NOMBRE, &[EN_DISENO]));
         }
diff --git a/crates/tc2/src/plan/guardar.rs b/crates/tc2/src/plan/guardar.rs
index d6c7f13..844b07c 100644
--- a/crates/tc2/src/plan/guardar.rs
+++ b/crates/tc2/src/plan/guardar.rs
@@ -19,7 +19,9 @@ use std::cell::RefCell;
 
 use super::plantillas::{self as p, BRAINSTORM, PLAN_FINAL};
 use super::roles;
-use crate::estado::{Cambio, Estado, Fallo, Operacion, de_otra_persona, por_estado, tarea_de};
+use crate::estado::{
+    AnotacionConDueno, Cambio, Estado, Fallo, Operacion, por_estado, tarea_propia,
+};
 use crate::i18n::Mensaje;
 use crate::i18n::catalogo::plan as txt;
 use crate::identidad::Quien;
@@ -92,13 +94,13 @@ impl Operacion for Guardar {
     }
 
     fn anotacion(&self) -> String {
-        format!(
-            "{NOMBRE_OP}|{}|{}|{}|{}",
-            self.id,
-            self.asignado.id().unwrap_or_default(),
-            self.asignado.login().unwrap_or_default(),
-            self.destino
-        )
+        AnotacionConDueno {
+            orden: NOMBRE_OP,
+            id: &self.id,
+            asignado: self.asignado.clone(),
+            resto: Some(&self.destino),
+        }
+        .escribir()
     }
 
     fn exige_red(&self) -> bool {
@@ -108,18 +110,14 @@ impl Operacion for Guardar {
     }
 
     fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
-        *self.dueno.borrow_mut() = None;
-        let (t, _) = tarea_de(estado, &self.id)?;
-        if let Some((fallo, dueno)) = de_otra_persona(
+        let (t, _) = tarea_propia(
+            estado,
             &self.id,
-            t.asignado_a.as_ref(),
             &self.asignado,
             super::NOMBRE,
             ("de-otra-persona", &txt::DE_OTRA_PERSONA),
-        ) {
-            *self.dueno.borrow_mut() = dueno;
-            return Err(fallo);
-        }
+            &self.dueno,
+        )?;
         if t.estado != EN_DISENO {
             return Err(por_estado(&self.id, &t.estado, super::NOMBRE, &[EN_DISENO]));
         }
diff --git a/crates/tc2/src/plan/mod.rs b/crates/tc2/src/plan/mod.rs
index ef0da3f..113ef7c 100644
--- a/crates/tc2/src/plan/mod.rs
+++ b/crates/tc2/src/plan/mod.rs
@@ -5,7 +5,7 @@
 //!   lanza (identidad), la configuracion de la base (`modo_flujo` y
 //!   `politica_carriles`; sin base o sin configuracion, el suelo) y la
 //!   operacion [`Planificar`] por el punto unico de escritura
-//!   ([`estado::transaccion`], con el `ROADMAP.md` del mismo commit).
+//!   ([`crate::estado::transaccion`], con el `ROADMAP.md` del mismo commit).
 //!   Publica en la carpeta fija de la tarea, `TASK-NNN/planificacion/`: las
 //!   peticiones de la ronda en `brainstorm/` y el esqueleto de
 //!   `plan-final.md` si no existe; la tarea queda `en-diseno` con el carril
@@ -54,7 +54,7 @@ use std::path::Path;
 use crate::carril::Carril;
 use crate::config::Config;
 use crate::contrato::{Informe, Resultado, Valor};
-use crate::estado::{self, Operacion};
+use crate::estado::{como_orden, con_dueno};
 use crate::git::{self, Git};
 use crate::i18n::Mensaje;
 use crate::i18n::catalogo::plan as txt;
@@ -163,7 +163,7 @@ pub fn plan_en(e: &Entorno<'_>, id: &str, carril: Option<Carril>) -> Informe {
     let i = match paso_plan(e, id, carril) {
         Ok(i) | Err(i) => i,
     };
-    de_la_orden(NOMBRE, i, "id", id)
+    como_orden(i, NOMBRE, "id", id)
 }
 
 fn uso_approve(motivo: &str) -> Informe {
@@ -258,13 +258,13 @@ pub fn aprobar_hito_en(e: &Entorno<'_>, hito: u32) -> Informe {
     let i = match paso_aprobar_hito(e, hito) {
         Ok(i) | Err(i) => i,
     };
-    de_la_orden(aprobar::NOMBRE, i, "hito", &hito.to_string())
+    como_orden(i, aprobar::NOMBRE, "hito", &hito.to_string())
 }
 
 fn paso_aprobar_hito(e: &Entorno<'_>, hito: u32) -> Result<Informe, Informe> {
     let (op, config) = persona_y_config(e, |q| aprobar::AprobarHito::nueva(hito, q, e.ahora))?;
     let op = op.con_config(&config);
-    let t = escribir(e, &op);
+    let t = e.escribir(&op);
     match t.resultado {
         Resultado::Ok => Ok(t.con_lista("aprobadas", op.aprobadas())),
         Resultado::SinCambios => Ok(t),
@@ -291,7 +291,7 @@ pub fn aprobar_en(e: &Entorno<'_>, id: &str) -> Informe {
     let i = match paso_aprobar(e, id) {
         Ok(i) | Err(i) => i,
     };
-    de_la_orden(aprobar::NOMBRE, i, "id", id)
+    como_orden(i, aprobar::NOMBRE, "id", id)
 }
 
 /// `approve --pedir-cambios` de `contenido` en la tarea `id` del repo de
@@ -300,7 +300,7 @@ pub fn pedir_cambios_en(e: &Entorno<'_>, id: &str, contenido: &str) -> Informe {
     let i = match paso_pedir_cambios(e, id, contenido) {
         Ok(i) | Err(i) => i,
     };
-    de_la_orden(aprobar::NOMBRE, i, "id", id)
+    como_orden(i, aprobar::NOMBRE, "id", id)
 }
 
 /// `rechazada motivo=identidad rol=persona` (RR-04), con lo que diga el
@@ -341,7 +341,7 @@ fn persona_y_config<O>(
 fn paso_aprobar(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
     let (op, config) = persona_y_config(e, |q| Aprobar::nueva(id, q, e.ahora))?;
     let op = op.con_config(&config);
-    let t = escribir(e, &op);
+    let t = e.escribir(&op);
     match t.resultado {
         Resultado::Ok | Resultado::SinCambios => Ok(con_seleccion(t, op.roles())),
         _ => Err(con_dueno(t, op.dueno())),
@@ -376,7 +376,7 @@ fn paso_pedir_cambios(e: &Entorno<'_>, id: &str, contenido: &str) -> Result<Info
     let quien = (e.quien)()?;
     traer(e.git, e.dir)?;
     let op = PedirCambios::nueva(id, &quien, contenido);
-    let t = escribir(e, &op);
+    let t = e.escribir(&op);
     match t.resultado {
         Resultado::Ok | Resultado::SinCambios => {
             let n = op.ronda().unwrap_or_default();
@@ -395,7 +395,7 @@ pub fn guardar_en(e: &Entorno<'_>, id: &str, destino: &str, contenido: &str) ->
     let i = match paso_guardar(e, id, destino, contenido) {
         Ok(i) | Err(i) => i,
     };
-    let mut i = de_la_orden(NOMBRE, i, "id", id);
+    let mut i = como_orden(i, NOMBRE, "id", id);
     i.claves
         .insert(1, ("destino".to_owned(), Valor::Texto(destino.to_owned())));
     i
@@ -413,20 +413,10 @@ fn traer(g: &Git, dir: &Path) -> Result<(), Informe> {
     Ok(())
 }
 
-/// La configuracion de la base (F1-10 decision 3). Sin base o sin
-/// configuracion, el suelo; una que no vale falla con su informe a nombre
-/// de `plan` (como `ciclo::umbral_de_inactividad`).
+/// La configuracion de la base (F1-10 decision 3), con la politica de
+/// [`crate::config::de_base_o_suelo`] a nombre de `plan`.
 pub fn config_de_base(g: &Git, dir: &Path) -> Result<Config, Informe> {
-    match crate::config::de_base(g, dir) {
-        Ok(c) => Ok(c),
-        Err(i) if matches!(i.valor("motivo"), Some("sin-base" | "sin-config")) => {
-            Ok(Config::suelo())
-        }
-        Err(mut i) => {
-            NOMBRE.clone_into(&mut i.comando);
-            Err(i)
-        }
-    }
+    crate::config::de_base_o_suelo(g, dir, NOMBRE)
 }
 
 fn paso_plan(e: &Entorno<'_>, id: &str, carril: Option<Carril>) -> Result<Informe, Informe> {
@@ -439,7 +429,7 @@ fn paso_plan(e: &Entorno<'_>, id: &str, carril: Option<Carril>) -> Result<Inform
     let f = crate::seleccion::orden::fuentes(e.git, e.dir, claude.as_deref(), &[]);
     let op = Planificar::nueva(id, &quien, carril, &config, e.ahora)
         .con_fuentes(f.inventario, f.catalogo);
-    let t = escribir(e, &op);
+    let t = e.escribir(&op);
     match t.resultado {
         Resultado::Ok | Resultado::SinCambios => Ok(con_resumen(t, op.resumen())),
         _ => Err(con_dueno(t, op.dueno())),
@@ -457,22 +447,13 @@ fn paso_guardar(
     let Some(op) = Guardar::nueva(id, &quien, destino, contenido) else {
         return Err(uso("destino-invalido"));
     };
-    let t = escribir(e, &op);
+    let t = e.escribir(&op);
     match t.resultado {
         Resultado::Ok | Resultado::SinCambios => Ok(t),
         _ => Err(con_dueno(t, op.dueno())),
     }
 }
 
-/// La operacion por el punto unico de escritura, con el `ROADMAP.md`
-/// regenerado en el mismo commit.
-fn escribir(e: &Entorno<'_>, op: &dyn Operacion) -> Informe {
-    let personas = crate::roadmap::personas(e.git, e.dir);
-    let generar = crate::roadmap::generador(&personas);
-    let ctx = estado::Contexto::nuevo(e.git, e.dir, e.catalogo, Some(&generar));
-    estado::transaccion(&ctx, op)
-}
-
 /// Las claves de lo que decidio `plan`.
 fn con_resumen(i: Informe, r: Option<Resumen>) -> Informe {
     let Some(r) = r else {
@@ -504,39 +485,10 @@ pub(crate) fn o_guion(s: &str) -> &str {
     if s.is_empty() { "-" } else { s }
 }
 
-/// Un rechazo con lo que sabe la operacion: quien la lleva o el arreglo de
-/// una tarea de la 0.16.
-fn con_dueno(mut i: Informe, dueno: Option<String>) -> Informe {
-    if i.resultado != Resultado::Rechazada {
-        return i;
-    }
-    match (i.valor("motivo"), dueno) {
-        (Some("de-otra-persona"), Some(l)) => i = i.con("asignado", l),
-        (Some("formato-1"), _) => i = i.con("arreglo", "tc2 actualizar"),
-        _ => {}
-    }
-    i
-}
-
-/// El informe como de la orden `nombre`: el comando, `clave=valor` el
-/// primero (`id=` de una tarea o `hito=` de `approve --hito`) y sin la
-/// clave `operacion` ni el `id=` de la transaccion. Un `sinred` sin
-/// `pendiente` dice `pendiente=no` (exige red).
-fn de_la_orden(nombre: &str, mut i: Informe, clave: &str, valor: &str) -> Informe {
-    nombre.clone_into(&mut i.comando);
-    i.claves
-        .retain(|(k, _)| k != "operacion" && k != "id" && k != clave);
-    i.claves
-        .insert(0, (clave.to_owned(), Valor::Texto(valor.to_owned())));
-    if i.resultado == Resultado::Sinred && i.valor("pendiente").is_none() {
-        i = i.con("pendiente", "no");
-    }
-    i
-}
-
 #[cfg(test)]
 mod tests {
     use super::*;
+    use crate::estado::Operacion;
     use crate::i18n::Idioma;
 
     /// Carpeta temporal propia del test: se borra al acabar.
diff --git a/crates/tc2/src/plan/operacion.rs b/crates/tc2/src/plan/operacion.rs
index e6b3adc..bd84d21 100644
--- a/crates/tc2/src/plan/operacion.rs
+++ b/crates/tc2/src/plan/operacion.rs
@@ -47,7 +47,9 @@ use super::plantillas::{self as p, Datos};
 use super::roles::{self, Rol};
 use crate::carril::Carril;
 use crate::config::Config;
-use crate::estado::{Cambio, Estado, Fallo, Operacion, de_otra_persona, por_estado, tarea_de};
+use crate::estado::{
+    AnotacionConDueno, Cambio, Estado, Fallo, Operacion, por_estado, tarea_propia,
+};
 use crate::i18n::catalogo::plan as txt;
 use crate::i18n::{Mensaje, Texto};
 use crate::identidad::Quien;
@@ -348,13 +350,13 @@ impl Operacion for Planificar {
     }
 
     fn anotacion(&self) -> String {
-        format!(
-            "{NOMBRE}|{}|{}|{}|{}",
-            self.id,
-            self.asignado.id().unwrap_or_default(),
-            self.asignado.login().unwrap_or_default(),
-            self.carril.map_or("-", Carril::nombre)
-        )
+        AnotacionConDueno {
+            orden: NOMBRE,
+            id: &self.id,
+            asignado: self.asignado.clone(),
+            resto: Some(self.carril.map_or("-", Carril::nombre)),
+        }
+        .escribir()
     }
 
     fn exige_red(&self) -> bool {
@@ -365,18 +367,14 @@ impl Operacion for Planificar {
 
     fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
         *self.resumen.borrow_mut() = None;
-        *self.dueno.borrow_mut() = None;
-        let (mut t, mut cuerpo) = tarea_de(estado, &self.id)?;
-        if let Some((fallo, dueno)) = de_otra_persona(
+        let (mut t, mut cuerpo) = tarea_propia(
+            estado,
             &self.id,
-            t.asignado_a.as_ref(),
             &self.asignado,
             NOMBRE,
             ("de-otra-persona", &txt::DE_OTRA_PERSONA),
-        ) {
-            *self.dueno.borrow_mut() = dueno;
-            return Err(fallo);
-        }
+            &self.dueno,
+        )?;
         if !PLANIFICABLES.contains(&t.estado.as_str()) {
             return Err(por_estado(&self.id, &t.estado, NOMBRE, &PLANIFICABLES));
         }
diff --git a/crates/tc2/src/revision/guardar.rs b/crates/tc2/src/revision/guardar.rs
index 4279bcd..c7bde4c 100644
--- a/crates/tc2/src/revision/guardar.rs
+++ b/crates/tc2/src/revision/guardar.rs
@@ -17,7 +17,9 @@
 
 use std::cell::RefCell;
 
-use crate::estado::{Cambio, Estado, Fallo, Operacion, de_otra_persona, por_estado, tarea_de};
+use crate::estado::{
+    AnotacionConDueno, Cambio, Estado, Fallo, Operacion, por_estado, tarea_propia,
+};
 use crate::hallazgos::InformeTarea;
 use crate::i18n::catalogo::revision as txt;
 use crate::identidad::Quien;
@@ -78,13 +80,13 @@ impl Operacion for GuardarRevision {
     }
 
     fn anotacion(&self) -> String {
-        format!(
-            "{NOMBRE_OP}|{}|{}|{}|{}",
-            self.id,
-            self.asignado.id().unwrap_or_default(),
-            self.asignado.login().unwrap_or_default(),
-            self.sha12
-        )
+        AnotacionConDueno {
+            orden: NOMBRE_OP,
+            id: &self.id,
+            asignado: self.asignado.clone(),
+            resto: Some(&self.sha12),
+        }
+        .escribir()
     }
 
     fn exige_red(&self) -> bool {
@@ -94,18 +96,14 @@ impl Operacion for GuardarRevision {
     }
 
     fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
-        *self.dueno.borrow_mut() = None;
-        let (t, _) = tarea_de(estado, &self.id)?;
-        if let Some((fallo, dueno)) = de_otra_persona(
+        let (t, _) = tarea_propia(
+            estado,
             &self.id,
-            t.asignado_a.as_ref(),
             &self.asignado,
             NOMBRE,
             ("de-otra-persona", &txt::DE_OTRA_PERSONA),
-        ) {
-            *self.dueno.borrow_mut() = dueno;
-            return Err(fallo);
-        }
+            &self.dueno,
+        )?;
         if t.estado != EN_REVISION {
             return Err(por_estado(&self.id, &t.estado, NOMBRE, &[EN_REVISION]));
         }
diff --git a/crates/tc2/src/revision/mod.rs b/crates/tc2/src/revision/mod.rs
index 5bfd383..d6650f5 100644
--- a/crates/tc2/src/revision/mod.rs
+++ b/crates/tc2/src/revision/mod.rs
@@ -46,7 +46,7 @@ use std::time::Duration;
 
 use crate::config;
 use crate::contrato::{Informe, Resultado, Valor};
-use crate::estado::{self, Fallo, Operacion};
+use crate::estado::{self, Fallo, Operacion, como_orden};
 use crate::git::{self, Git};
 use crate::i18n::Mensaje;
 use crate::i18n::catalogo::revision as txt;
@@ -106,7 +106,7 @@ pub fn review_en(e: &Entorno<'_>, id: &str) -> Informe {
     let i = match paso(e, id) {
         Ok(i) | Err(i) => i,
     };
-    de_review(i, id)
+    como_orden(i, NOMBRE, "id", id)
 }
 
 /// Titulo del PR: `TASK-NNN: <titulo>`.
@@ -195,26 +195,6 @@ pub fn revision_ia(p: &dyn Plataforma, pr: u64, revisor_id: Option<u64>) -> &'st
     }
 }
 
-/// El informe como de `review`: el comando, `id=` el primero y sin la clave
-/// `operacion` de la transaccion. Un `sinred` dice `pendiente=no` (`review`
-/// exige red).
-fn de_review(mut i: Informe, id: &str) -> Informe {
-    NOMBRE.clone_into(&mut i.comando);
-    i.claves.retain(|(k, _)| k != "operacion" && k != "id");
-    i.claves
-        .insert(0, ("id".to_owned(), Valor::Texto(id.to_owned())));
-    if i.resultado == Resultado::Sinred && i.valor("pendiente").is_none() {
-        i = i.con("pendiente", "no");
-    }
-    i
-}
-
-fn rechazada(motivo: &str, m: Mensaje) -> Informe {
-    Informe::nuevo(NOMBRE, Resultado::Rechazada)
-        .con("motivo", motivo)
-        .diagnostico(m)
-}
-
 fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
     let (g, dir, ahora) = (e.base.git, e.base.dir, e.base.ahora);
     let principal = git::localizar_principal(g, dir)?;
@@ -230,42 +210,28 @@ fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
     // 2. El paso a revision en seco: los rechazos, antes de empujar nada.
     let sondeo = Revisar::nueva(id, &quien, ahora);
     if let Err(f) = sondeo.aplicar(&vista) {
-        return Err(match f {
-            Fallo::Rechazada {
-                motivo,
-                diagnostico,
-            } => {
-                let i = rechazada(&motivo, diagnostico);
-                match sondeo.dueno() {
-                    Some(l) => i.con("asignado", l),
-                    None => i,
-                }
-            }
-            Fallo::Error(d) => Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d),
+        let i = f.informe(NOMBRE);
+        return Err(match sondeo.dueno() {
+            Some(l) if i.resultado == Resultado::Rechazada => i.con("asignado", l),
+            _ => i,
         });
     }
-    let (t, cuerpo) = match estado::tarea_de(&vista, id) {
-        Ok(tc) => tc,
-        Err(Fallo::Rechazada {
-            motivo,
-            diagnostico,
-        }) => return Err(rechazada(&motivo, diagnostico)),
-        Err(Fallo::Error(d)) => return Err(Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d)),
-    };
+    let (t, cuerpo) = estado::tarea_de(&vista, id).map_err(|f| f.informe(NOMBRE))?;
     // 3. La rama, contra `develop` (D3) y con su worktree.
     let Some(rama) = t.rama.clone() else {
-        return Err(rechazada(
-            "sin-rama",
-            Mensaje::nuevo(&txt::SIN_RAMA).con("id", id),
-        ));
+        return Err(
+            Fallo::rechazada("sin-rama", Mensaje::nuevo(&txt::SIN_RAMA).con("id", id))
+                .informe(NOMBRE),
+        );
     };
     if git::base_de_rama(&rama) != config::BASE {
-        return Err(rechazada(
+        return Err(Fallo::rechazada(
             "carril-no-soportado",
             Mensaje::nuevo(&txt::CARRIL_NO_SOPORTADO)
                 .con("id", id)
                 .con("rama", rama.as_str()),
         )
+        .informe(NOMBRE)
         .con("rama", rama.as_str()));
     }
     let wt = principal.join(git::CARPETA_WORKTREES).join(id);
@@ -274,18 +240,21 @@ fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
             .iter()
             .any(|w| w.rama.as_deref() == Some(rama.as_str()) && git::misma_ruta(&w.ruta, &wt));
     if !registrado {
-        return Err(rechazada(
+        return Err(Fallo::rechazada(
             "sin-worktree",
             Mensaje::nuevo(&txt::SIN_WORKTREE)
                 .con("id", id)
                 .con("ruta", git::worktree_relativo(id))
                 .con("rama", rama.as_str()),
         )
+        .informe(NOMBRE)
         .con("rama", rama.as_str())
         .con("arreglo", format!("tc2 start {id}")));
     }
     let Some(url) = git::estado::url_origin(g, &wt, NOMBRE)? else {
-        return Err(rechazada("plataforma", Mensaje::nuevo(&txt::SIN_ORIGIN)));
+        return Err(
+            Fallo::rechazada("plataforma", Mensaje::nuevo(&txt::SIN_ORIGIN)).informe(NOMBRE),
+        );
     };
     // 4. La seleccion de revision (F6-3), con el diff de la rama y lo de la
     // base; no cambia nada si falla: lo dice en `avisos`.
diff --git a/crates/tc2/src/revision/operacion.rs b/crates/tc2/src/revision/operacion.rs
index a664d9d..022eca3 100644
--- a/crates/tc2/src/revision/operacion.rs
+++ b/crates/tc2/src/revision/operacion.rs
@@ -23,7 +23,9 @@
 
 use std::cell::RefCell;
 
-use crate::estado::{Cambio, Estado, Fallo, Operacion, de_otra_persona, por_estado, tarea_de};
+use crate::estado::{
+    AnotacionConDueno, Cambio, Estado, Fallo, Operacion, por_estado, tarea_propia,
+};
 use crate::i18n::catalogo::revision as txt;
 use crate::identidad::Quien;
 use crate::tarea::{Asignado, Transicion};
@@ -114,13 +116,13 @@ impl Operacion for Revisar {
     }
 
     fn anotacion(&self) -> String {
-        format!(
-            "{NOMBRE}|{}|{}|{}|{}",
-            self.id,
-            self.asignado.id().unwrap_or_default(),
-            self.asignado.login().unwrap_or_default(),
-            self.ahora
-        )
+        AnotacionConDueno {
+            orden: NOMBRE,
+            id: &self.id,
+            asignado: self.asignado.clone(),
+            resto: Some(&self.ahora.to_string()),
+        }
+        .escribir()
     }
 
     fn exige_red(&self) -> bool {
@@ -129,18 +131,14 @@ impl Operacion for Revisar {
     }
 
     fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
-        *self.dueno.borrow_mut() = None;
-        let (mut t, mut cuerpo) = tarea_de(estado, &self.id)?;
-        if let Some((fallo, dueno)) = de_otra_persona(
+        let (mut t, mut cuerpo) = tarea_propia(
+            estado,
             &self.id,
-            t.asignado_a.as_ref(),
             &self.asignado,
             NOMBRE,
             ("de-otra-persona", &txt::DE_OTRA_PERSONA),
-        ) {
-            *self.dueno.borrow_mut() = dueno;
-            return Err(fallo);
-        }
+            &self.dueno,
+        )?;
         let anotada = self.anotar(&t);
         match t.estado.as_str() {
             EN_REVISION => {
@@ -183,16 +181,9 @@ impl Operacion for Revisar {
 /// nunca queda pendiente; si se re-ejecutara, sin persona delante:
 /// `decidido_por: agente`.
 pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
-    let mut partes = a.splitn(5, '|');
-    if partes.next()? != NOMBRE {
-        return None;
-    }
-    let id = partes.next()?;
-    let cuenta: u64 = partes.next()?.parse().ok()?;
-    let asignado = Asignado::persona(cuenta, partes.next()?)?;
-    let ahora: u64 = partes.next()?.parse().ok()?;
-    crate::git::es_id(id)
-        .then(|| Box::new(Revisar::con(id, asignado, "agente", ahora)) as Box<dyn Operacion>)
+    let a = AnotacionConDueno::leer(a).filter(|a| a.orden == NOMBRE)?;
+    let ahora: u64 = a.resto?.parse().ok()?;
+    Some(Box::new(Revisar::con(a.id, a.asignado, "agente", ahora)))
 }
 
 #[cfg(test)]
diff --git a/crates/tc2/src/roadmap/mod.rs b/crates/tc2/src/roadmap/mod.rs
index cd09e72..adad6e8 100644
--- a/crates/tc2/src/roadmap/mod.rs
+++ b/crates/tc2/src/roadmap/mod.rs
@@ -300,23 +300,11 @@ fn renombrado(mut i: Informe) -> Informe {
     i
 }
 
-fn de_fallo(f: Fallo) -> Informe {
-    match f {
-        Fallo::Rechazada {
-            motivo,
-            diagnostico,
-        } => Informe::nuevo(NOMBRE, Resultado::Rechazada)
-            .con("motivo", motivo)
-            .diagnostico(diagnostico),
-        Fallo::Error(d) => Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d),
-    }
-}
-
 /// La situacion del estado recien traido (F3-5b), su commit y su frescura.
 fn situacion_en(g: &Git, dir: &Path, ahora: u64) -> Result<(Situacion, String, Frescura), Informe> {
     let (vista, frescura) = estado::leer_fresco(g, dir, ahora).map_err(renombrado)?;
     let commit = vista.commit().map(corto).unwrap_or_default();
-    let s = situacion::calcular(&vista).map_err(de_fallo)?;
+    let s = situacion::calcular(&vista).map_err(|f| f.informe(NOMBRE))?;
     Ok((s, commit, frescura))
 }
 
@@ -359,7 +347,8 @@ fn imprimir(g: &Git, dir: &Path, ahora: u64) -> Result<(String, Informe, Frescur
 
 fn zona_rota(s: &Situacion, commit: &str) -> Informe {
     let motivo = s.roadmap_roto().map_or("", zona::Rota::motivo);
-    de_fallo(roto(ROADMAP, motivo))
+    roto(ROADMAP, motivo)
+        .informe(NOMBRE)
         .con("roto", motivo)
         .con("commit", commit)
 }
@@ -444,8 +433,16 @@ fn editado_en(g: &Git, carpeta: &Path) -> Result<Lectura, Informe> {
     )?;
     let mia = match zona::extraer(&texto) {
         Ok(Some(z)) => z.to_owned(),
-        Ok(None) => return Err(de_fallo(roto(&donde(), "sin-zona")).con("roto", "sin-zona")),
-        Err(r) => return Err(de_fallo(roto(&donde(), r.motivo())).con("roto", r.motivo())),
+        Ok(None) => {
+            return Err(roto(&donde(), "sin-zona")
+                .informe(NOMBRE)
+                .con("roto", "sin-zona"));
+        }
+        Err(r) => {
+            return Err(roto(&donde(), r.motivo())
+                .informe(NOMBRE)
+                .con("roto", r.motivo()));
+        }
     };
     let base = crate::git::estado::leer_blob(g, carpeta, NOMBRE, "HEAD", ROADMAP)
         .ok()
diff --git a/crates/tc2/src/seleccion/orden.rs b/crates/tc2/src/seleccion/orden.rs
index a1288f7..f20d576 100644
--- a/crates/tc2/src/seleccion/orden.rs
+++ b/crates/tc2/src/seleccion/orden.rs
@@ -51,7 +51,7 @@ use std::path::{Path, PathBuf};
 use super::catalogo::{self, Catalogo};
 use super::{Contexto, Seleccion};
 use crate::carril::Carril;
-use crate::contrato::{Informe, Resultado};
+use crate::contrato::{Informe, Resultado, valor_de};
 use crate::git::Git;
 use crate::i18n::Mensaje;
 use crate::i18n::catalogo::seleccion as txt;
@@ -352,7 +352,7 @@ fn args(a: &[String]) -> Result<Args, Informe> {
     let mut r = Args::default();
     let mut it = a.iter();
     while let Some(x) = it.next() {
-        let mut valor = || it.next().cloned().ok_or_else(|| uso("falta-valor"));
+        let mut valor = || valor_de(&mut it, uso);
         match x.as_str() {
             "--rutas" if r.rutas.is_none() => r.rutas = Some(valor()?),
             "--nul" if !r.nul => r.nul = true,
@@ -563,11 +563,7 @@ fn paso_seleccion(a: &[String], e: &Entorno<'_>) -> Result<Informe, Informe> {
     let (mut id, mut fichero, mut rutas_f) = (None, None, None);
     let mut it = a.iter();
     while let Some(x) = it.next() {
-        let mut valor = || {
-            it.next()
-                .cloned()
-                .ok_or_else(|| uso_seleccion("falta-valor"))
-        };
+        let mut valor = || valor_de(&mut it, uso_seleccion);
         match x.as_str() {
             "--tarea-fichero" if fichero.is_none() => fichero = Some(valor()?),
             "--rutas" if rutas_f.is_none() => rutas_f = Some(valor()?),
@@ -594,17 +590,7 @@ fn paso_seleccion(a: &[String], e: &Entorno<'_>) -> Result<Informe, Informe> {
                 SELECCION.clone_into(&mut i.comando);
                 i
             })?;
-            crate::estado::tarea_de(&v, id).map_err(|f| match f {
-                crate::estado::Fallo::Rechazada {
-                    motivo,
-                    diagnostico,
-                } => Informe::nuevo(SELECCION, Resultado::Rechazada)
-                    .con("motivo", motivo)
-                    .diagnostico(diagnostico),
-                crate::estado::Fallo::Error(m) => {
-                    Informe::nuevo(SELECCION, Resultado::Error).diagnostico(m)
-                }
-            })?
+            crate::estado::tarea_de(&v, id).map_err(|f| f.informe(SELECCION))?
         }
         (None, None) => return Err(uso_seleccion("sin-tarea")),
     };
diff --git a/crates/tc2/src/siguiente.rs b/crates/tc2/src/siguiente.rs
index 99934e4..eeb0068 100644
--- a/crates/tc2/src/siguiente.rs
+++ b/crates/tc2/src/siguiente.rs
@@ -75,13 +75,7 @@ fn siguiente(g: &Git, dir: &Path, ahora: u64) -> (Informe, Option<Mensaje>) {
     let commit = vista.commit().map(corto).unwrap_or_default();
     let i = match calcular(&vista) {
         Ok(c) => frescura.anotar(c.informe(&commit)),
-        Err(Fallo::Rechazada {
-            motivo,
-            diagnostico,
-        }) => Informe::nuevo(NOMBRE, Resultado::Rechazada)
-            .con("motivo", motivo)
-            .diagnostico(diagnostico),
-        Err(Fallo::Error(d)) => Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d),
+        Err(f) => f.informe(NOMBRE),
     };
     (i, aviso)
 }
diff --git a/crates/tc2/src/tablero.rs b/crates/tc2/src/tablero.rs
index f8ac354..1de681c 100644
--- a/crates/tc2/src/tablero.rs
+++ b/crates/tc2/src/tablero.rs
@@ -440,15 +440,7 @@ fn tablero_en(g: &Git, dir: &Path, op: &str, con_local: bool) -> Result<Leido, I
         None
     };
     let commit = vista.commit().map(corto).unwrap_or_default();
-    let s = situacion::calcular(&vista).map_err(|f| match f {
-        Fallo::Rechazada {
-            motivo,
-            diagnostico,
-        } => Informe::nuevo(op, Resultado::Rechazada)
-            .con("motivo", motivo)
-            .diagnostico(diagnostico),
-        Fallo::Error(d) => Informe::nuevo(op, Resultado::Error).diagnostico(d),
-    })?;
+    let s = situacion::calcular(&vista).map_err(|f: Fallo| f.informe(op))?;
     let (personas, ilegibles) = crate::roadmap::personas_y_aviso(g, dir);
     let avisos = if ilegibles {
         vec![PERSONAS_ILEGIBLES]
diff --git a/crates/tc2/src/toma/mod.rs b/crates/tc2/src/toma/mod.rs
index a6ef595..28d2c10 100644
--- a/crates/tc2/src/toma/mod.rs
+++ b/crates/tc2/src/toma/mod.rs
@@ -33,8 +33,8 @@ pub mod operacion;
 
 use std::path::Path;
 
-use crate::contrato::{Informe, Resultado, Valor};
-use crate::estado::{self, Catalogo, Estado, Fallo, Operacion};
+use crate::contrato::{Informe, Resultado};
+use crate::estado::{self, Catalogo, Estado, Fallo, Operacion, como_orden};
 use crate::git::{self, Git, Subcomando};
 use crate::i18n::Mensaje;
 use crate::i18n::catalogo::toma as txt;
@@ -65,6 +65,17 @@ pub struct Entorno<'a> {
     pub ahora: u64,
 }
 
+impl Entorno<'_> {
+    /// La operacion por el punto unico de escritura, con el `ROADMAP.md`
+    /// regenerado en el mismo commit (A8-09).
+    pub(crate) fn escribir(&self, op: &dyn Operacion) -> Informe {
+        let personas = crate::roadmap::personas(self.git, self.dir);
+        let generar = crate::roadmap::generador(&personas);
+        let ctx = estado::Contexto::nuevo(self.git, self.dir, self.catalogo, Some(&generar));
+        estado::transaccion(&ctx, op)
+    }
+}
+
 /// `start TASK-NNN` (sin `--json`, que quita `lib.rs`).
 pub fn orden(args: &[String], e: &Entorno<'_>) -> Informe {
     orden_con_solape(args, e).0
@@ -97,27 +108,7 @@ pub fn start_con_solape(e: &Entorno<'_>, id: &str) -> (Informe, Option<Solape>)
     let i = match paso(e, id, &mut s) {
         Ok(i) | Err(i) => i,
     };
-    (de_start(i, id), s)
-}
-
-/// El informe como de `start`: el comando, `id=` el primero y sin la clave
-/// `operacion` de la transaccion. Un `sinred` dice siempre `pendiente=no`
-/// (`start` exige red, F1-2 D2b).
-fn de_start(mut i: Informe, id: &str) -> Informe {
-    NOMBRE.clone_into(&mut i.comando);
-    i.claves.retain(|(k, _)| k != "operacion" && k != "id");
-    i.claves
-        .insert(0, ("id".to_owned(), Valor::Texto(id.to_owned())));
-    if i.resultado == Resultado::Sinred && i.valor("pendiente").is_none() {
-        i = i.con("pendiente", "no");
-    }
-    i
-}
-
-fn rechazada(motivo: &str, m: Mensaje) -> Informe {
-    Informe::nuevo(NOMBRE, Resultado::Rechazada)
-        .con("motivo", motivo)
-        .diagnostico(m)
+    (como_orden(i, NOMBRE, "id", id), s)
 }
 
 /// Un rechazo de la toma con lo que sabe la operacion: el login de quien la
@@ -156,20 +147,17 @@ fn paso(e: &Entorno<'_>, id: &str, s: &mut Option<Solape>) -> Result<Informe, In
     let sondeo = Tomar::nueva(id, &quien, e.ahora);
     let empezada = match sondeo.aplicar(&SinHistorico(&vista)) {
         Ok(c) => c.is_empty(),
-        Err(Fallo::Rechazada {
-            motivo,
-            diagnostico,
-        }) => return Err(rechazo_de_toma(rechazada(&motivo, diagnostico), &sondeo)),
-        Err(Fallo::Error(d)) => return Err(Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d)),
+        Err(f) => return Err(rechazo_de_toma(f.informe(NOMBRE), &sondeo)),
     };
     let rama = sondeo.rama().unwrap_or_default();
     let Some((tipo, slug)) = git::partes_de_rama(id, &rama) else {
-        return Err(rechazada(
+        return Err(Fallo::rechazada(
             "rama-invalida",
             Mensaje::nuevo(&txt::RAMA_INVALIDA)
                 .con("id", id)
                 .con("rama", rama.as_str()),
         )
+        .informe(NOMBRE)
         .con("rama", rama.as_str()));
     };
     // 4. La base, valida antes de escribir nada (F1-3).
@@ -208,10 +196,7 @@ fn paso(e: &Entorno<'_>, id: &str, s: &mut Option<Solape>) -> Result<Informe, In
     let commits = crate::estimacion::commits_de(g, &principal);
     let leido = commits.leido();
     let op = Tomar::nueva(id, &quien, e.ahora).con_commits(commits);
-    let personas = crate::roadmap::personas(g, e.dir);
-    let generar = crate::roadmap::generador(&personas);
-    let ctx = estado::Contexto::nuevo(g, e.dir, e.catalogo, Some(&generar));
-    let t = estado::transaccion(&ctx, &op);
+    let t = e.escribir(&op);
     match t.resultado {
         Resultado::Ok => {
             let commit = t.valor("commit").map(str::to_owned);
@@ -283,10 +268,11 @@ fn en_curso_de(e: &dyn Estado) -> Option<Vec<EnCurso>> {
 /// sin tags, su punta; `release/` no se empieza con `start`.
 fn base_de(g: &Git, principal: &Path, id: &str, rama: &str) -> Result<String, Informe> {
     if rama.starts_with("release/") {
-        return Err(rechazada(
+        return Err(Fallo::rechazada(
             "tipo-release",
             Mensaje::nuevo(&txt::TIPO_RELEASE).con("id", id),
-        ));
+        )
+        .informe(NOMBRE));
     }
     if !rama.starts_with("hotfix/") {
         return Ok(git::base_de_rama(rama).to_owned());
@@ -330,11 +316,12 @@ fn carpeta_libre(g: &Git, principal: &Path, id: &str, rama: &str) -> Result<(),
     if registrado {
         return Ok(());
     }
-    Err(rechazada(
+    Err(Fallo::rechazada(
         "carpeta-ocupada",
         Mensaje::nuevo(&crate::i18n::catalogo::git::CARPETA_OCUPADA)
             .con("ruta", git::worktree_relativo(id)),
     )
+    .informe(NOMBRE)
     .con("rama", rama))
 }
 
diff --git a/crates/tc2/src/toma/operacion.rs b/crates/tc2/src/toma/operacion.rs
index f241d61..d89aa9d 100644
--- a/crates/tc2/src/toma/operacion.rs
+++ b/crates/tc2/src/toma/operacion.rs
@@ -24,7 +24,9 @@
 
 use std::cell::RefCell;
 
-use crate::estado::{Cambio, Estado, Fallo, Operacion, de_otra_persona, por_estado, tarea_de};
+use crate::estado::{
+    AnotacionConDueno, Cambio, Estado, Fallo, Operacion, de_otra_persona, por_estado, tarea_de,
+};
 use crate::estimacion::{Commits, Estimacion};
 use crate::grafo::{Nodo, dependencia_cumplida};
 use crate::i18n::Mensaje;
@@ -143,12 +145,13 @@ impl Operacion for Tomar {
     }
 
     fn anotacion(&self) -> String {
-        format!(
-            "{NOMBRE}|{}|{}|{}",
-            self.id,
-            self.asignado.id().unwrap_or_default(),
-            self.asignado.login().unwrap_or_default()
-        )
+        AnotacionConDueno {
+            orden: NOMBRE,
+            id: &self.id,
+            asignado: self.asignado.clone(),
+            resto: None,
+        }
+        .escribir()
     }
 
     fn exige_red(&self) -> bool {
@@ -264,21 +267,13 @@ pub fn instante(segundos: u64) -> String {
 /// (para el catalogo). `start` exige red y nunca queda pendiente; si se
 /// re-ejecutara, lo haria el CLI sin persona delante: `decidido_por: agente`.
 pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
-    let mut partes = a.splitn(4, '|');
-    if partes.next()? != NOMBRE {
-        return None;
-    }
-    let id = partes.next()?;
-    let cuenta: u64 = partes.next()?.parse().ok()?;
-    let asignado = Asignado::persona(cuenta, partes.next()?)?;
-    crate::git::es_id(id).then(|| {
-        Box::new(Tomar::con(
-            id,
-            asignado,
-            "agente",
-            crate::identidad::ahora(),
-        )) as Box<dyn Operacion>
-    })
+    let a = AnotacionConDueno::leer(a).filter(|a| a.orden == NOMBRE && a.resto.is_none())?;
+    Some(Box::new(Tomar::con(
+        a.id,
+        a.asignado,
+        "agente",
+        crate::identidad::ahora(),
+    )))
 }
 
 #[cfg(test)]
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/01-en-diseno/TASK-149/tarea.md                                                     | 48 ------------------------------------------------
 tareas/{01-en-diseno => 02-en-curso}/TASK-149/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-149/planificacion/plan-final.md                 |  0
 tareas/02-en-curso/TASK-149/tarea.md                                                      | 81 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 4 files changed, 81 insertions(+), 48 deletions(-)
````
## Candidatos 4D

Pistas generadas desde el diff, no hallazgos: comprueba cada una (Regla 4D de `revision.md`).

### Posible duplicacion con lo que ya existia

Definiciones nuevas con nombre igual o parecido a otra de la rama base. Mira si hacen lo mismo con pequenas diferencias (duplicacion real = IMPORTANTE).

- `anotacion` (crates/tc2/src/ciclo/operacion.rs:113) ~ `anotacion` (crates/tc2/src/alta.rs:393), `anotacion` (crates/tc2/src/ciclo/liberar.rs:234), `anotacion` (crates/tc2/src/ciclo/liberar.rs:311) y 25 mas
- `escribir` (crates/tc2/src/estado/dueno.rs:57) ~ `escribir` (crates/tc2/src/ciclo/mod.rs:369), `escribir` (crates/tc2/src/ciclo/operacion.rs:99), `escribir` (crates/tc2/src/inventario.rs:1062) y 6 mas
- `leer` (crates/tc2/src/estado/dueno.rs:74) ~ `leer` (crates/tc2/src/config/mod.rs:437), `leer` (crates/tc2/src/estado/frescura.rs:160), `leer` (crates/tc2/src/estado/lectura.rs:21) y 23 mas
- `con_dueno` (crates/tc2/src/estado/dueno.rs:92) ~ `con_dueno` (crates/tc2/src/ciclo/mod.rs:378), `con_dueno` (crates/tc2/src/plan/mod.rs:509)
- `informe` (crates/tc2/src/estado/transaccion.rs:59) ~ `informe` (crates/tc2/src/config/mod.rs:384), `Informe` (crates/tc2/src/contrato.rs:90), `informe` (crates/tc2/src/descarga.rs:157) y 5 mas
- `escribir` (crates/tc2/src/toma/mod.rs:71) ~ `escribir` (crates/tc2/src/ciclo/mod.rs:369), `escribir` (crates/tc2/src/ciclo/operacion.rs:99), `escribir` (crates/tc2/src/inventario.rs:1062) y 6 mas

### Posible duplicacion dentro de la tarea

Definiciones nuevas parecidas entre si: la misma tarea puede estar escribiendo dos veces lo mismo.

- `escribir` (crates/tc2/src/estado/dueno.rs:57) ~ `escribir` (crates/tc2/src/toma/mod.rs:71)

