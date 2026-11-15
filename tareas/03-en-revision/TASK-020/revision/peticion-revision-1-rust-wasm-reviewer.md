# Peticion de revision — TASK-020 (ronda 1)

- Tarea: TASK-020 — F2-5 Modelo de tarea y parser/serializador de frontmatter YAML valido
- Rama revisada: feature/task-020-f2-5-modelo-de-tarea-y-parser-serializad
- Rama base: develop
- Commit revisado (HEAD): f2fdb154fbe7f9cef83906ff34b4c79fd56b290d
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-020 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-020/tarea.md
- tareas/03-en-revision/TASK-020/planificacion/
- crates/tc2/Cargo.toml
- crates/tc2/src/alta.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/tarea/cuerpo.rs
- crates/tc2/src/tarea/mod.rs
- crates/tc2/src/tarea/modelo.rs
- crates/tc2/src/tarea/yaml.rs
- crates/tc2/tests/tarea.rs

## Commits a revisar (git log develop..HEAD)

````
f2fdb15 chore(TASK-020): autorrevision ronda 1
edd0790 fix(TASK-020): delimitador con espacios como la 0.16 y doc de escribir
cbfcb20 feat(TASK-020): modelo de tarea y frontmatter YAML valido (F2-5)
06f5656 chore(TASK-020): tarea en curso
````

## Diff de tu dominio (8 fichero(s) de 8; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/Cargo.toml b/crates/tc2/Cargo.toml
index 000b656..5e341ec 100644
--- a/crates/tc2/Cargo.toml
+++ b/crates/tc2/Cargo.toml
@@ -14,3 +14,16 @@ path = "src/main.rs"
 
 [lints]
 workspace = true
+
+# F2-5: lectura del frontmatter YAML (F1-14, fila F1-9: serde-saphyr con
+# claves duplicadas como error). Solo se lee con el crate; la escritura es un
+# emisor propio (forma fija de F1-9). Todo detras de `tarea::yaml`.
+[dependencies]
+serde = { version = "1", features = ["derive"] }
+serde-saphyr = { version = "2.1", default-features = false, features = ["deserialize"] }
+
+# Parser YAML de otra base de codigo (port de libyaml, como el PyYAML del PoC
+# de F1-9) para la ida y vuelta de los tests: con el de produccion seria
+# circular.
+[dev-dependencies]
+serde_yaml = "0.9"
diff --git a/crates/tc2/src/alta.rs b/crates/tc2/src/alta.rs
index a20c9f2..a91c1b7 100644
--- a/crates/tc2/src/alta.rs
+++ b/crates/tc2/src/alta.rs
@@ -11,6 +11,7 @@ use std::cell::Cell;
 use std::time::{SystemTime, UNIX_EPOCH};
 
 use crate::estado::{Cambio, Estado, Fallo, Operacion};
+use crate::tarea::{Cuerpo, Tarea};
 
 /// Nombre de la operacion y de la orden del binario.
 pub const NOMBRE: &str = "new";
@@ -18,57 +19,6 @@ pub const NOMBRE: &str = "new";
 /// Prefijo de las carpetas de tarea en la rama de estado.
 const PREFIJO: &str = "TASK-";
 
-/// Plantilla de `tarea.md` del ADR F1-9, punto 6 (formato 2). La sustituye el
-/// modelo de tarea de F2-5. `{id}`, `{titulo}` y `{fecha}` se rellenan;
-/// `rama` queda en `null` hasta F2-4b, que la deriva del ID.
-const PLANTILLA: &str = "---
-formato: 2
-id: \"{id}\"
-titulo: \"{titulo}\"
-tipo: \"feature\"
-sprint: 0
-etiquetas: []
-complejidad: null
-carril: null
-estado: \"planificada\"
-motivo: null
-plan_aprobado: false
-rama: null
-asignado_a: null
-roles:
-  diseno: {modelo: null, skills: [], regla: null}
-  implementacion: {modelo: \"sonnet\", skills: [], regla: null}
-  revision: {agente: \"general-purpose\", modelo: null, skills: [], regla: null}
-estimacion: null
-tokens_diseno: null
-tokens_implementacion: null
-tokens_revision: null
-creado: \"{fecha}\"
-actualizado: \"{fecha}\"
-dependencias: []
----
-## Objetivo
-
-<que se quiere y por que>
-
-## Criterios de aceptacion
-
-- [ ] <criterio comprobable>
-
-## Escenarios
-
-<opcional: 1. **Dado** <estado>, **cuando** <accion>, **entonces** <resultado>>
-
-## Resultado
-
-<obligatoria al cerrar: que se hizo, divergencias con el plan, hallazgos no corregidos y por que>
-
-## Transiciones
-
-| fecha | fase | modo | decidido_por | version |
-|---|---|---|---|---|
-";
-
 /// Alta de una tarea: reserva el siguiente ID y escribe su `tarea.md`.
 #[derive(Debug)]
 pub struct Alta {
@@ -127,14 +77,7 @@ pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
     titulo_valido(titulo).then(|| Box::new(Alta::nueva(titulo)) as Box<dyn Operacion>)
 }
 
-/// Si `titulo` vale: no vacio y sin saltos de linea ni caracteres de control
-/// (tampoco los separadores U+2028 y U+2029, que YAML lee como salto).
-pub fn titulo_valido(titulo: &str) -> bool {
-    !titulo.trim().is_empty()
-        && !titulo
-            .chars()
-            .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
-}
+pub use crate::tarea::titulo_valido;
 
 /// `TASK-NNN` con al menos tres cifras (`TASK-1000` sin renumerar).
 pub fn formato_id(n: u32) -> String {
@@ -166,18 +109,10 @@ pub fn siguiente_id(rutas: &[String]) -> Result<u32, Fallo> {
         .ok_or_else(|| Fallo::Error("no quedan IDs de tarea".to_owned()))
 }
 
-/// Escapa un texto para una cadena YAML entre comillas dobles (F1-9 punto 5).
-fn escapar(texto: &str) -> String {
-    texto.replace('\\', "\\\\").replace('"', "\\\"")
-}
-
-/// `tarea.md` de una tarea recien creada.
+/// `tarea.md` de una tarea recien creada: el modelo de F2-5 con la
+/// plantilla de F1-9 (punto 6).
 pub fn tarea_md(id: &str, titulo: &str, fecha: &str) -> String {
-    PLANTILLA
-        .replace("{id}", id)
-        .replace("{fecha}", fecha)
-        // El titulo el ultimo: un `{fecha}` en el titulo no se sustituye.
-        .replace("{titulo}", &escapar(titulo))
+    Tarea::nueva(id, titulo, fecha).escribir(&Cuerpo::plantilla())
 }
 
 /// Fecha de hoy en UTC, `AAAA-MM-DD`, sin dependencias.
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 1a74577..0462abc 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -2,14 +2,15 @@
 //!
 //! Modulos: [`contrato`] (salida de toda operacion, ADR F1-4), [`git`]
 //! (capa de operaciones Git deterministas, F2-2) y [`estado`] (rama de
-//! estado `taskcode/tareas`, F2-3) y [`alta`] (alta de tareas con reserva de
-//! ID, F2-4). Las capas que vienen (modelo de tarea de F2-5...) entran como
-//! modulos de esta biblioteca.
+//! estado `taskcode/tareas`, F2-3), [`alta`] (alta de tareas con reserva de
+//! ID, F2-4) y [`tarea`] (modelo de tarea y frontmatter YAML, F2-5). Las capas
+//! que vienen entran como modulos de esta biblioteca.
 
 pub mod alta;
 pub mod contrato;
 pub mod estado;
 pub mod git;
+pub mod tarea;
 
 use contrato::{Informe, Resultado};
 
diff --git a/crates/tc2/src/tarea/cuerpo.rs b/crates/tc2/src/tarea/cuerpo.rs
new file mode 100644
index 0000000..6c827cf
--- /dev/null
+++ b/crates/tc2/src/tarea/cuerpo.rs
@@ -0,0 +1,122 @@
+//! Cuerpo Markdown de `tarea.md` por secciones (ADR F1-9, punto 4): se
+//! reconocen por su titulo, no por su posicion, y se conservan byte a byte.
+
+/// Cuerpo de la plantilla de una tarea recien creada (F1-9, punto 6).
+const PLANTILLA: &str = "## Objetivo
+
+<que se quiere y por que>
+
+## Criterios de aceptacion
+
+- [ ] <criterio comprobable>
+
+## Escenarios
+
+<opcional: 1. **Dado** <estado>, **cuando** <accion>, **entonces** <resultado>>
+
+## Resultado
+
+<obligatoria al cerrar: que se hizo, divergencias con el plan, hallazgos no corregidos y por que>
+
+## Transiciones
+
+| fecha | fase | modo | decidido_por | version |
+|---|---|---|---|---|
+";
+
+/// Una seccion `## <titulo>` con su texto, incluida la linea del titulo.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Seccion {
+    /// Titulo sin `## ` ni espacios al final.
+    pub titulo: String,
+    /// Texto tal cual, desde la linea `## ` hasta la siguiente seccion.
+    pub texto: String,
+}
+
+/// Cuerpo de una tarea: lo que va antes de la primera seccion y las
+/// secciones en su orden. Escribirlo es concatenar lo leido.
+#[derive(Debug, Clone, PartialEq, Eq, Default)]
+pub struct Cuerpo {
+    /// Texto antes de la primera seccion (normalmente vacio).
+    pub preambulo: String,
+    /// Secciones en el orden del fichero.
+    pub secciones: Vec<Seccion>,
+}
+
+/// Si `linea` abre o cierra un bloque de codigo cercado.
+fn es_cerca(linea: &str) -> bool {
+    let l = linea.trim_start();
+    l.starts_with("```") || l.starts_with("~~~")
+}
+
+/// Titulo de `linea` si es un encabezado de seccion `## ` (no `###`).
+fn titulo(linea: &str) -> Option<String> {
+    let resto = linea.strip_prefix("## ")?;
+    Some(resto.trim_end().to_owned())
+}
+
+impl Cuerpo {
+    /// Parte `texto` en secciones. Un `## ` dentro de un bloque de codigo
+    /// cercado no abre seccion.
+    pub fn leer(texto: &str) -> Self {
+        let mut cuerpo = Self::default();
+        let mut en_codigo = false;
+        for linea in texto.split_inclusive('\n') {
+            let encabezado = if en_codigo { None } else { titulo(linea) };
+            if es_cerca(linea) {
+                en_codigo = !en_codigo;
+            }
+            match (encabezado, cuerpo.secciones.last_mut()) {
+                (Some(t), _) => cuerpo.secciones.push(Seccion {
+                    titulo: t,
+                    texto: linea.to_owned(),
+                }),
+                (None, Some(s)) => s.texto.push_str(linea),
+                (None, None) => cuerpo.preambulo.push_str(linea),
+            }
+        }
+        cuerpo
+    }
+
+    /// El cuerpo de una tarea recien creada.
+    pub fn plantilla() -> Self {
+        Self::leer(PLANTILLA)
+    }
+
+    /// Texto del cuerpo: lo leido, byte a byte.
+    pub fn escribir(&self) -> String {
+        let mut s = self.preambulo.clone();
+        for seccion in &self.secciones {
+            s.push_str(&seccion.texto);
+        }
+        s
+    }
+
+    /// La primera seccion con ese titulo.
+    pub fn seccion(&self, titulo: &str) -> Option<&Seccion> {
+        self.secciones.iter().find(|s| s.titulo == titulo)
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn secciones_por_titulo_y_byte_a_byte() {
+        let t = "intro\n## Transiciones\r\n| a |\n## Resultado \nhecho\n```md\n## no es seccion\n```\n### sub\n## Objetivo\nx";
+        let c = Cuerpo::leer(t);
+        assert_eq!(c.preambulo, "intro\n");
+        let titulos: Vec<&str> = c.secciones.iter().map(|s| s.titulo.as_str()).collect();
+        assert_eq!(titulos, ["Transiciones", "Resultado", "Objetivo"]);
+        assert!(
+            c.seccion("Resultado")
+                .is_some_and(|s| s.texto.contains("## no es seccion"))
+        );
+        assert!(c.seccion("Escenarios").is_none());
+        assert_eq!(c.escribir(), t);
+        let p = Cuerpo::plantilla();
+        assert_eq!(p.escribir(), PLANTILLA);
+        assert_eq!(p.secciones.len(), 5);
+    }
+}
diff --git a/crates/tc2/src/tarea/mod.rs b/crates/tc2/src/tarea/mod.rs
new file mode 100644
index 0000000..4c32c41
--- /dev/null
+++ b/crates/tc2/src/tarea/mod.rs
@@ -0,0 +1,80 @@
+//! Modelo de tarea (F2-5; ADR F1-9): parser y serializador de `tarea.md`.
+//!
+//! - [`leer`] distingue el formato por la clave `formato`: con ella, formato 2
+//!   ([`Tarea`], el que escribe la 2.0); sin ella, formato 1 de la 0.16
+//!   ([`TareaV1`]), que se lee para migrarlo pero no se escribe (F1-9 punto 3).
+//! - [`Tarea::escribir`] emite el frontmatter en la forma fija de F1-9: YAML
+//!   valido para un parser estandar (H11), cadenas siempre entre comillas.
+//! - El cuerpo se lee por secciones reconocidas por titulo ([`Cuerpo`]) y se
+//!   conserva byte a byte.
+//!
+//! Hoja pura: no depende de git, de la red ni de la rama de estado.
+
+mod cuerpo;
+mod modelo;
+mod yaml;
+
+use std::fmt;
+
+pub use cuerpo::{Cuerpo, Seccion};
+pub use modelo::{
+    COMPLEJIDADES, ESTADOS, REGLAS, Rol, RolRevision, Roles, TIPOS, Tarea, TareaV1, id_valido,
+    titulo_valido,
+};
+
+/// Un `tarea.md` leido.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum Documento {
+    /// Formato 1 (0.16), sin `formato`.
+    Formato1(TareaV1, Cuerpo),
+    /// Formato 2 (`formato: 2`).
+    Formato2(Tarea, Cuerpo),
+}
+
+/// Por que no se pudo leer un `tarea.md`.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct ErrorTarea {
+    /// Clave del frontmatter a la que se refiere, si es de una.
+    pub clave: Option<String>,
+    /// Que falla.
+    pub mensaje: String,
+}
+
+impl ErrorTarea {
+    pub(crate) fn general(mensaje: impl Into<String>) -> Self {
+        Self {
+            clave: None,
+            mensaje: mensaje.into(),
+        }
+    }
+
+    pub(crate) fn en(clave: &str, mensaje: impl Into<String>) -> Self {
+        Self {
+            clave: Some(clave.to_owned()),
+            mensaje: mensaje.into(),
+        }
+    }
+}
+
+impl fmt::Display for ErrorTarea {
+    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
+        match &self.clave {
+            Some(c) => write!(f, "{c}: {}", self.mensaje),
+            None => f.write_str(&self.mensaje),
+        }
+    }
+}
+
+impl std::error::Error for ErrorTarea {}
+
+/// Lee un `tarea.md` (todo es dato no confiable, F1-8: esquema estricto,
+/// claves duplicadas rechazadas, desconocidas conservadas).
+pub fn leer(texto: &str) -> Result<Documento, ErrorTarea> {
+    let partes = yaml::partir(texto)?;
+    let cuerpo = Cuerpo::leer(partes.cuerpo);
+    if partes.tiene("formato") {
+        Ok(Documento::Formato2(Tarea::leer(&partes)?, cuerpo))
+    } else {
+        Ok(Documento::Formato1(TareaV1::leer(&partes)?, cuerpo))
+    }
+}
diff --git a/crates/tc2/src/tarea/modelo.rs b/crates/tc2/src/tarea/modelo.rs
new file mode 100644
index 0000000..18ad743
--- /dev/null
+++ b/crates/tc2/src/tarea/modelo.rs
@@ -0,0 +1,626 @@
+//! Campos del frontmatter (ADR F1-9, punto 3) con esquema estricto (punto 5):
+//! formato 2 ([`Tarea`], el que escribe la 2.0) y formato 1 ([`TareaV1`], el
+//! de la 0.16, que se lee para migrarlo pero no se escribe).
+
+use serde::Deserialize;
+use serde::de::IgnoredAny;
+
+use super::ErrorTarea;
+use super::cuerpo::Cuerpo;
+use super::yaml::{self, Partes};
+
+/// Valores de `tipo` (los de la 0.16, que mandan en el flujo de ramas, F1-3).
+pub const TIPOS: [&str; 4] = ["feature", "fix", "hotfix", "release"];
+/// Valores de `complejidad` (los de la 0.16).
+pub const COMPLEJIDADES: [&str; 5] = ["trivial", "simple", "media", "alta", "critica"];
+/// Valores de `estado` del formato 2 (F1-9: gana `descartada`).
+pub const ESTADOS: [&str; 6] = [
+    "planificada",
+    "en-diseno",
+    "en-curso",
+    "en-revision",
+    "terminada",
+    "descartada",
+];
+/// Valores de `estado` del formato 1 (0.16).
+const ESTADOS_V1: [&str; 5] = [
+    "planificada",
+    "en-diseno",
+    "en-curso",
+    "en-revision",
+    "terminada",
+];
+/// Valores de la regla de seleccion de skill (0.16; el contenido es de F1-15).
+pub const REGLAS: [&str; 3] = ["solape", "prioridad", "llm"];
+
+/// Claves del formato 2 en el orden de la plantilla de F1-9 (punto 6).
+pub(crate) const CLAVES: [&str; 21] = [
+    "formato",
+    "id",
+    "titulo",
+    "tipo",
+    "sprint",
+    "etiquetas",
+    "complejidad",
+    "carril",
+    "estado",
+    "motivo",
+    "plan_aprobado",
+    "rama",
+    "asignado_a",
+    "roles",
+    "estimacion",
+    "tokens_diseno",
+    "tokens_implementacion",
+    "tokens_revision",
+    "creado",
+    "actualizado",
+    "dependencias",
+];
+// Sin `zonas` (F1-17) ni `hito` (F1-18) ni el `roles` ampliado de F1-15:
+// hasta que los recoja la tarea que los usa, `zonas` y `hito` se conservan
+// como claves desconocidas.
+
+/// Claves del formato 1 (`TASK_FIELD_ORDER` del plugin 0.16.1).
+pub(crate) const CLAVES_V1: [&str; 22] = [
+    "id",
+    "titulo",
+    "tipo",
+    "sprint",
+    "etiquetas",
+    "complejidad",
+    "modelo_sugerido",
+    "estado",
+    "plan_aprobado",
+    "rama",
+    "asignado_a",
+    "agente_revisor",
+    "skills_recomendados",
+    "regla_seleccion_skill",
+    "ultimo_commit_revisado",
+    "revision_codex",
+    "tokens_diseno",
+    "tokens_implementacion",
+    "tokens_revision",
+    "creado",
+    "actualizado",
+    "dependencias",
+];
+
+/// Si `titulo` vale: no vacio y sin saltos de linea ni caracteres de control
+/// (tampoco los separadores U+2028 y U+2029, que YAML 1.1 lee como salto).
+pub fn titulo_valido(titulo: &str) -> bool {
+    !titulo.trim().is_empty() && texto_valido(titulo)
+}
+
+/// Sin caracteres de control ni separadores de linea.
+fn texto_valido(texto: &str) -> bool {
+    !texto
+        .chars()
+        .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
+}
+
+/// `TASK-NNN` (al menos tres cifras) o `TASK-NNNa` (parte de un troceo).
+pub fn id_valido(id: &str) -> bool {
+    let Some(resto) = id.strip_prefix("TASK-") else {
+        return false;
+    };
+    let resto = resto
+        .strip_suffix(|c: char| c.is_ascii_lowercase())
+        .unwrap_or(resto);
+    resto.len() >= 3 && resto.bytes().all(|b| b.is_ascii_digit())
+}
+
+/// `AAAA-MM-DD` con cifras.
+fn fecha_valida(f: &str) -> bool {
+    let b = f.as_bytes();
+    b.len() == 10
+        && b.iter().enumerate().all(|(i, c)| match i {
+            4 | 7 => *c == b'-',
+            _ => c.is_ascii_digit(),
+        })
+}
+
+/// Un rol de `roles` (diseno, implementacion).
+#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct Rol {
+    /// Modelo del rol (contenido de F1-15).
+    pub modelo: Option<String>,
+    /// Skills del rol.
+    pub skills: Vec<String>,
+    /// Regla que eligio la skill.
+    pub regla: Option<String>,
+}
+
+/// El rol de revision, que ademas nombra el agente.
+#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct RolRevision {
+    /// Agente que revisa.
+    pub agente: Option<String>,
+    /// Modelo del revisor.
+    pub modelo: Option<String>,
+    /// Skills del revisor.
+    pub skills: Vec<String>,
+    /// Regla que eligio la skill.
+    pub regla: Option<String>,
+}
+
+/// El mapa `roles` (RR-17). Un campo que no conoce el modelo se rechaza: al
+/// reescribirlo se perderia.
+#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct Roles {
+    /// Diseno.
+    pub diseno: Rol,
+    /// Implementacion.
+    pub implementacion: Rol,
+    /// Revision.
+    pub revision: RolRevision,
+}
+
+/// Tarea en formato 2 (F1-9, punto 3).
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Tarea {
+    pub id: String,
+    pub titulo: String,
+    pub tipo: String,
+    pub sprint: u32,
+    pub etiquetas: Vec<String>,
+    pub complejidad: Option<String>,
+    pub carril: Option<String>,
+    pub estado: String,
+    pub motivo: Option<String>,
+    pub plan_aprobado: bool,
+    pub rama: Option<String>,
+    pub asignado_a: Option<String>,
+    pub roles: Roles,
+    /// Entrada `estimacion:` cruda (su forma la fija F1-17); `None` es `null`.
+    pub estimacion: Option<String>,
+    pub tokens_diseno: Option<u64>,
+    pub tokens_implementacion: Option<u64>,
+    pub tokens_revision: Option<u64>,
+    pub creado: String,
+    pub actualizado: String,
+    pub dependencias: Vec<String>,
+    /// Claves desconocidas con su texto crudo, en su orden (F1-9 D4a: se
+    /// conservan en la ida y vuelta; `doctor` avisa).
+    pub desconocidas: Vec<(String, String)>,
+}
+
+#[derive(Deserialize)]
+struct CrudoV2 {
+    formato: Option<u32>,
+    id: Option<String>,
+    titulo: Option<String>,
+    tipo: Option<String>,
+    sprint: Option<u32>,
+    etiquetas: Option<Vec<String>>,
+    complejidad: Option<String>,
+    carril: Option<String>,
+    estado: Option<String>,
+    motivo: Option<String>,
+    plan_aprobado: Option<bool>,
+    rama: Option<String>,
+    asignado_a: Option<String>,
+    roles: Option<Roles>,
+    estimacion: Option<IgnoredAny>,
+    tokens_diseno: Option<u64>,
+    tokens_implementacion: Option<u64>,
+    tokens_revision: Option<u64>,
+    creado: Option<String>,
+    actualizado: Option<String>,
+    dependencias: Option<Vec<String>>,
+}
+
+/// Valor obligatorio (presente y no `null`).
+fn exigir<T>(clave: &str, v: Option<T>) -> Result<T, ErrorTarea> {
+    v.ok_or_else(|| ErrorTarea::en(clave, "falta o es null"))
+}
+
+/// Texto sin caracteres de control.
+fn texto(clave: &str, s: String) -> Result<String, ErrorTarea> {
+    if texto_valido(&s) {
+        Ok(s)
+    } else {
+        Err(ErrorTarea::en(clave, "lleva caracteres de control"))
+    }
+}
+
+fn texto_opcional(clave: &str, s: Option<String>) -> Result<Option<String>, ErrorTarea> {
+    s.map(|s| texto(clave, s)).transpose()
+}
+
+fn textos(clave: &str, v: Vec<String>) -> Result<Vec<String>, ErrorTarea> {
+    v.into_iter().map(|s| texto(clave, s)).collect()
+}
+
+/// Valor de una lista cerrada.
+fn uno_de(clave: &str, s: String, permitidos: &[&str]) -> Result<String, ErrorTarea> {
+    if permitidos.contains(&s.as_str()) {
+        Ok(s)
+    } else {
+        Err(ErrorTarea::en(
+            clave,
+            format!("«{s}» no es uno de: {}", permitidos.join(", ")),
+        ))
+    }
+}
+
+fn uno_de_opcional(
+    clave: &str,
+    s: Option<String>,
+    permitidos: &[&str],
+) -> Result<Option<String>, ErrorTarea> {
+    s.map(|s| uno_de(clave, s, permitidos)).transpose()
+}
+
+fn id(clave: &str, s: String) -> Result<String, ErrorTarea> {
+    if id_valido(&s) {
+        Ok(s)
+    } else {
+        Err(ErrorTarea::en(clave, format!("«{s}» no es un ID TASK-NNN")))
+    }
+}
+
+fn fecha(clave: &str, s: String) -> Result<String, ErrorTarea> {
+    if fecha_valida(&s) {
+        Ok(s)
+    } else {
+        Err(ErrorTarea::en(
+            clave,
+            format!("«{s}» no es una fecha AAAA-MM-DD"),
+        ))
+    }
+}
+
+fn rol(clave: &str, r: Rol) -> Result<Rol, ErrorTarea> {
+    Ok(Rol {
+        modelo: texto_opcional(clave, r.modelo)?,
+        skills: textos(clave, r.skills)?,
+        regla: uno_de_opcional(clave, r.regla, &REGLAS)?,
+    })
+}
+
+/// Claves que faltan de `claves` en `partes` (todas son obligatorias).
+fn faltan(partes: &Partes<'_>, claves: &[&str]) -> Result<(), ErrorTarea> {
+    match claves.iter().find(|c| !partes.tiene(c)) {
+        Some(c) => Err(ErrorTarea::en(c, "falta")),
+        None => Ok(()),
+    }
+}
+
+/// Entradas de `partes` cuya clave no esta en `claves`.
+fn desconocidas(partes: &Partes<'_>, claves: &[&str]) -> Vec<(String, String)> {
+    partes
+        .entradas
+        .iter()
+        .filter(|e| !claves.contains(&e.clave.as_str()))
+        .map(|e| (e.clave.clone(), e.crudo.clone()))
+        .collect()
+}
+
+impl Tarea {
+    /// Tarea recien creada con los valores de la plantilla de F1-9.
+    pub fn nueva(id: &str, titulo: &str, fecha: &str) -> Self {
+        Self {
+            id: id.to_owned(),
+            titulo: titulo.to_owned(),
+            tipo: "feature".to_owned(),
+            sprint: 0,
+            etiquetas: Vec::new(),
+            complejidad: None,
+            carril: None,
+            estado: "planificada".to_owned(),
+            motivo: None,
+            plan_aprobado: false,
+            // La deriva del ID F2-4b; hasta entonces, `null`.
+            rama: None,
+            asignado_a: None,
+            roles: Roles {
+                diseno: Rol::default(),
+                implementacion: Rol {
+                    modelo: Some("sonnet".to_owned()),
+                    ..Rol::default()
+                },
+                revision: RolRevision {
+                    agente: Some("general-purpose".to_owned()),
+                    ..RolRevision::default()
+                },
+            },
+            estimacion: None,
+            tokens_diseno: None,
+            tokens_implementacion: None,
+            tokens_revision: None,
+            creado: fecha.to_owned(),
+            actualizado: fecha.to_owned(),
+            dependencias: Vec::new(),
+            desconocidas: Vec::new(),
+        }
+    }
+
+    /// Claves desconocidas conservadas (para el aviso de `doctor`).
+    pub fn claves_desconocidas(&self) -> &[(String, String)] {
+        &self.desconocidas
+    }
+
+    /// Lee el frontmatter de formato 2 con el esquema estricto.
+    pub(crate) fn leer(partes: &Partes<'_>) -> Result<Self, ErrorTarea> {
+        faltan(partes, &CLAVES)?;
+        let c: CrudoV2 = yaml::decodificar(&partes.yaml())?;
+        let formato = exigir("formato", c.formato)?;
+        if formato != 2 {
+            return Err(ErrorTarea::en(
+                "formato",
+                format!(
+                    "formato {formato} no soportado (se lee el 2, o sin `formato` el 1 de la 0.16)"
+                ),
+            ));
+        }
+        let titulo = exigir("titulo", c.titulo)?;
+        if !titulo_valido(&titulo) {
+            return Err(ErrorTarea::en(
+                "titulo",
+                "vacio o con caracteres de control",
+            ));
+        }
+        let estado = uno_de("estado", exigir("estado", c.estado)?, &ESTADOS)?;
+        let motivo = texto_opcional("motivo", c.motivo)?;
+        if estado == "descartada" && motivo.as_deref().is_none_or(|m| m.trim().is_empty()) {
+            return Err(ErrorTarea::en(
+                "motivo",
+                "obligatorio con `estado: descartada`",
+            ));
+        }
+        let roles = exigir("roles", c.roles)?;
+        let revision = roles.revision;
+        let estimacion = c.estimacion.and_then(|_| {
+            partes
+                .entradas
+                .iter()
+                .find(|e| e.clave == "estimacion")
+                .map(|e| e.crudo.clone())
+        });
+        Ok(Self {
+            id: id("id", exigir("id", c.id)?)?,
+            titulo,
+            tipo: uno_de("tipo", exigir("tipo", c.tipo)?, &TIPOS)?,
+            sprint: exigir("sprint", c.sprint)?,
+            etiquetas: textos("etiquetas", exigir("etiquetas", c.etiquetas)?)?,
+            complejidad: uno_de_opcional("complejidad", c.complejidad, &COMPLEJIDADES)?,
+            carril: texto_opcional("carril", c.carril)?,
+            estado,
+            motivo,
+            plan_aprobado: exigir("plan_aprobado", c.plan_aprobado)?,
+            rama: texto_opcional("rama", c.rama)?,
+            asignado_a: texto_opcional("asignado_a", c.asignado_a)?,
+            roles: Roles {
+                diseno: rol("roles.diseno", roles.diseno)?,
+                implementacion: rol("roles.implementacion", roles.implementacion)?,
+                revision: RolRevision {
+                    agente: texto_opcional("roles.revision", revision.agente)?,
+                    modelo: texto_opcional("roles.revision", revision.modelo)?,
+                    skills: textos("roles.revision", revision.skills)?,
+                    regla: uno_de_opcional("roles.revision", revision.regla, &REGLAS)?,
+                },
+            },
+            estimacion,
+            tokens_diseno: c.tokens_diseno,
+            tokens_implementacion: c.tokens_implementacion,
+            tokens_revision: c.tokens_revision,
+            creado: fecha("creado", exigir("creado", c.creado)?)?,
+            actualizado: fecha("actualizado", exigir("actualizado", c.actualizado)?)?,
+            dependencias: exigir("dependencias", c.dependencias)?
+                .into_iter()
+                .map(|d| id("dependencias", d))
+                .collect::<Result<_, _>>()?,
+            desconocidas: desconocidas(partes, &CLAVES),
+        })
+    }
+
+    /// Frontmatter en la forma fija de F1-9 seguido de `cuerpo`. Las claves
+    /// desconocidas van al final, en su orden y con su texto original.
+    /// No valida: un texto con caracteres de control sale escapado (YAML
+    /// valido) pero [`super::leer`] lo rechaza, y `estimacion` y las
+    /// desconocidas se copian tal cual (tienen que venir de una lectura).
+    pub fn escribir(&self, cuerpo: &Cuerpo) -> String {
+        let r = &self.roles;
+        let rol = |x: &Rol| {
+            format!(
+                "{{modelo: {}, skills: {}, regla: {}}}",
+                yaml::opcional(x.modelo.as_deref()),
+                yaml::lista(&x.skills),
+                yaml::opcional(x.regla.as_deref())
+            )
+        };
+        let mut s = String::from("---\nformato: 2\n");
+        let mut linea = |clave: &str, valor: String| {
+            s.push_str(clave);
+            s.push_str(": ");
+            s.push_str(&valor);
+            s.push('\n');
+        };
+        linea("id", yaml::cadena(&self.id));
+        linea("titulo", yaml::cadena(&self.titulo));
+        linea("tipo", yaml::cadena(&self.tipo));
+        linea("sprint", self.sprint.to_string());
+        linea("etiquetas", yaml::lista(&self.etiquetas));
+        linea("complejidad", yaml::opcional(self.complejidad.as_deref()));
+        linea("carril", yaml::opcional(self.carril.as_deref()));
+        linea("estado", yaml::cadena(&self.estado));
+        linea("motivo", yaml::opcional(self.motivo.as_deref()));
+        linea("plan_aprobado", self.plan_aprobado.to_string());
+        linea("rama", yaml::opcional(self.rama.as_deref()));
+        linea("asignado_a", yaml::opcional(self.asignado_a.as_deref()));
+        let rv = &r.revision;
+        let revision = format!(
+            "{{agente: {}, modelo: {}, skills: {}, regla: {}}}",
+            yaml::opcional(rv.agente.as_deref()),
+            yaml::opcional(rv.modelo.as_deref()),
+            yaml::lista(&rv.skills),
+            yaml::opcional(rv.regla.as_deref())
+        );
+        let roles = format!(
+            "\n  diseno: {}\n  implementacion: {}\n  revision: {revision}",
+            rol(&r.diseno),
+            rol(&r.implementacion)
+        );
+        s.push_str("roles:");
+        s.push_str(&roles);
+        s.push('\n');
+        match &self.estimacion {
+            Some(crudo) => s.push_str(crudo),
+            None => s.push_str("estimacion: null\n"),
+        }
+        let mut linea = |clave: &str, valor: String| {
+            s.push_str(clave);
+            s.push_str(": ");
+            s.push_str(&valor);
+            s.push('\n');
+        };
+        linea("tokens_diseno", yaml::numero(self.tokens_diseno));
+        linea(
+            "tokens_implementacion",
+            yaml::numero(self.tokens_implementacion),
+        );
+        linea("tokens_revision", yaml::numero(self.tokens_revision));
+        linea("creado", yaml::cadena(&self.creado));
+        linea("actualizado", yaml::cadena(&self.actualizado));
+        linea("dependencias", yaml::lista(&self.dependencias));
+        for (_, crudo) in &self.desconocidas {
+            s.push_str(crudo);
+        }
+        s.push_str("---\n");
+        s.push_str(&cuerpo.escribir());
+        s
+    }
+}
+
+/// Tarea en formato 1, el de la 0.16 (los 22 campos de `TASK_FIELD_ORDER`).
+/// Se lee para migrarla (F1-12); no se escribe.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct TareaV1 {
+    pub id: String,
+    pub titulo: String,
+    pub tipo: String,
+    pub sprint: u32,
+    pub etiquetas: Vec<String>,
+    pub complejidad: Option<String>,
+    pub modelo_sugerido: String,
+    pub estado: String,
+    pub plan_aprobado: bool,
+    pub rama: String,
+    pub asignado_a: Option<String>,
+    pub agente_revisor: String,
+    pub skills_recomendados: Vec<String>,
+    pub regla_seleccion_skill: Option<String>,
+    pub ultimo_commit_revisado: Option<String>,
+    pub revision_codex: bool,
+    pub tokens_diseno: Option<u64>,
+    pub tokens_implementacion: Option<u64>,
+    pub tokens_revision: Option<u64>,
+    pub creado: String,
+    pub actualizado: String,
+    pub dependencias: Vec<String>,
+    /// Claves desconocidas con su texto crudo, en su orden.
+    pub desconocidas: Vec<(String, String)>,
+}
+
+#[derive(Deserialize)]
+struct CrudoV1 {
+    id: Option<String>,
+    titulo: Option<String>,
+    tipo: Option<String>,
+    sprint: Option<u32>,
+    etiquetas: Option<Vec<String>>,
+    complejidad: Option<String>,
+    modelo_sugerido: Option<String>,
+    estado: Option<String>,
+    plan_aprobado: Option<bool>,
+    rama: Option<String>,
+    asignado_a: Option<String>,
+    agente_revisor: Option<String>,
+    skills_recomendados: Option<Vec<String>>,
+    regla_seleccion_skill: Option<String>,
+    ultimo_commit_revisado: Option<String>,
+    revision_codex: Option<bool>,
+    tokens_diseno: Option<u64>,
+    tokens_implementacion: Option<u64>,
+    tokens_revision: Option<u64>,
+    creado: Option<String>,
+    actualizado: Option<String>,
+    dependencias: Option<Vec<String>>,
+}
+
+/// La 0.16 escapa `"` pero no `\` (H11): un `\` que no va seguido de `"`
+/// lo leeria mal un parser estandar (`C:\ruta` daria un retorno de carro).
+/// Mejor un error claro que un valor cambiado en silencio.
+fn escape_ambiguo(partes: &Partes<'_>) -> Result<(), ErrorTarea> {
+    for e in partes
+        .entradas
+        .iter()
+        .filter(|e| CLAVES_V1.contains(&e.clave.as_str()))
+    {
+        let mut cs = e.crudo.chars().peekable();
+        while let Some(c) = cs.next() {
+            if c == '\\' && cs.next_if_eq(&'"').is_none() {
+                return Err(ErrorTarea::en(
+                    &e.clave,
+                    "lleva `\\` con el escape de la 0.16, que un parser YAML estandar lee distinto (H11)",
+                ));
+            }
+        }
+    }
+    Ok(())
+}
+
+impl TareaV1 {
+    /// Lee el frontmatter de formato 1 con las reglas de la 0.16
+    /// (`validateTask`): obligatorios los que la 0.16 exige.
+    pub(crate) fn leer(partes: &Partes<'_>) -> Result<Self, ErrorTarea> {
+        escape_ambiguo(partes)?;
+        let c: CrudoV1 = yaml::decodificar(&partes.yaml())?;
+        Ok(Self {
+            id: id("id", exigir("id", c.id)?)?,
+            titulo: texto("titulo", exigir("titulo", c.titulo)?)?,
+            tipo: uno_de("tipo", exigir("tipo", c.tipo)?, &TIPOS)?,
+            sprint: exigir("sprint", c.sprint)?,
+            etiquetas: textos("etiquetas", exigir("etiquetas", c.etiquetas)?)?,
+            complejidad: uno_de_opcional("complejidad", c.complejidad, &COMPLEJIDADES)?,
+            modelo_sugerido: texto(
+                "modelo_sugerido",
+                exigir("modelo_sugerido", c.modelo_sugerido)?,
+            )?,
+            estado: uno_de("estado", exigir("estado", c.estado)?, &ESTADOS_V1)?,
+            plan_aprobado: exigir("plan_aprobado", c.plan_aprobado)?,
+            rama: texto("rama", exigir("rama", c.rama)?)?,
+            asignado_a: texto_opcional("asignado_a", c.asignado_a)?,
+            agente_revisor: texto(
+                "agente_revisor",
+                exigir("agente_revisor", c.agente_revisor)?,
+            )?,
+            skills_recomendados: textos(
+                "skills_recomendados",
+                exigir("skills_recomendados", c.skills_recomendados)?,
+            )?,
+            regla_seleccion_skill: uno_de_opcional(
+                "regla_seleccion_skill",
+                c.regla_seleccion_skill,
+                &REGLAS,
+            )?,
+            ultimo_commit_revisado: texto_opcional(
+                "ultimo_commit_revisado",
+                c.ultimo_commit_revisado,
+            )?,
+            revision_codex: exigir("revision_codex", c.revision_codex)?,
+            tokens_diseno: c.tokens_diseno,
+            tokens_implementacion: c.tokens_implementacion,
+            tokens_revision: c.tokens_revision,
+            creado: texto("creado", exigir("creado", c.creado)?)?,
+            actualizado: texto("actualizado", exigir("actualizado", c.actualizado)?)?,
+            dependencias: textos("dependencias", exigir("dependencias", c.dependencias)?)?,
+            desconocidas: desconocidas(partes, &CLAVES_V1),
+        })
+    }
+}
diff --git a/crates/tc2/src/tarea/yaml.rs b/crates/tc2/src/tarea/yaml.rs
new file mode 100644
index 0000000..bd12415
--- /dev/null
+++ b/crates/tc2/src/tarea/yaml.rs
@@ -0,0 +1,250 @@
+//! Costura YAML del modelo de tarea (ADR F1-14, punto 1: el YAML vive detras
+//! de un modulo propio para poder cambiar de crate sin tocar el resto).
+//!
+//! - Lectura: [`partir`] separa el frontmatter en entradas de primer nivel con
+//!   su texto crudo (para conservar las claves desconocidas byte a byte) y
+//!   [`decodificar`] valida el documento entero con `serde-saphyr` (claves
+//!   duplicadas, anclas, alias y claves de fusion como error) y decodifica
+//!   las claves conocidas.
+//! - Escritura: emisor propio ([`cadena`], [`lista`]...), porque F1-9 fija la
+//!   forma byte a byte (comillas dobles siempre, orden de claves, `roles` en
+//!   flujo) y un serializador generico no la da.
+
+use serde::de::DeserializeOwned;
+use serde_saphyr::{DuplicateKeyPolicy, MergeKeyPolicy, Options};
+
+use super::ErrorTarea;
+
+/// Delimitador del frontmatter.
+const DELIMITADOR: &str = "---";
+
+/// Una entrada de primer nivel del frontmatter: su clave y sus lineas tal
+/// cual (con `\n` al final de cada una).
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub(crate) struct Entrada {
+    pub clave: String,
+    pub crudo: String,
+}
+
+/// Un `tarea.md` partido: entradas del frontmatter y el cuerpo intacto.
+#[derive(Debug)]
+pub(crate) struct Partes<'a> {
+    pub entradas: Vec<Entrada>,
+    pub cuerpo: &'a str,
+}
+
+impl Partes<'_> {
+    /// Si el frontmatter tiene la clave `clave`.
+    pub fn tiene(&self, clave: &str) -> bool {
+        self.entradas.iter().any(|e| e.clave == clave)
+    }
+
+    /// El texto YAML del frontmatter (sin delimitadores), normalizado a `\n`.
+    pub fn yaml(&self) -> String {
+        self.entradas.iter().map(|e| e.crudo.as_str()).collect()
+    }
+}
+
+/// Si `clave` es una clave de primer nivel admitida: minusculas ASCII,
+/// cifras, `_` y `-`. Las claves entre comillas o complejas se rechazan para
+/// que la clave que ve el separador sea la misma que ve el parser.
+fn clave_admitida(clave: &str) -> bool {
+    !clave.is_empty()
+        && clave
+            .bytes()
+            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
+}
+
+/// Parte `texto` en frontmatter y cuerpo. Admite BOM UTF-8 y finales `\r\n`
+/// (el frontmatter se normaliza a `\n`; el cuerpo queda intacto).
+pub(crate) fn partir(texto: &str) -> Result<Partes<'_>, ErrorTarea> {
+    let texto = texto.strip_prefix('\u{feff}').unwrap_or(texto);
+    let mut lineas = texto.split_inclusive('\n');
+    let sin_fin = |l: &str| l.trim_end_matches('\n').trim_end_matches('\r').to_owned();
+    match lineas.next() {
+        Some(l) if sin_fin(l).trim_end() == DELIMITADOR => {}
+        _ => return Err(ErrorTarea::general("no empieza con un frontmatter `---`")),
+    }
+    let mut consumido = texto.split_inclusive('\n').next().map_or(0, str::len);
+    let mut entradas: Vec<Entrada> = Vec::new();
+    let mut cerrado = false;
+    for l in lineas {
+        consumido += l.len();
+        let linea = sin_fin(l);
+        // Como la 0.16: espacios al final del delimitador se admiten.
+        if linea.trim_end() == DELIMITADOR {
+            cerrado = true;
+            break;
+        }
+        let continua = linea.is_empty() || linea.starts_with([' ', '\t', '#']);
+        if continua {
+            match entradas.last_mut() {
+                Some(e) => {
+                    e.crudo.push_str(&linea);
+                    e.crudo.push('\n');
+                }
+                None if linea.trim().is_empty() => {}
+                None => {
+                    return Err(ErrorTarea::general(
+                        "texto antes de la primera clave del frontmatter",
+                    ));
+                }
+            }
+            continue;
+        }
+        let Some((clave, _)) = linea.split_once(':') else {
+            return Err(ErrorTarea::general(format!(
+                "linea del frontmatter sin clave: «{linea}»"
+            )));
+        };
+        if !clave_admitida(clave) {
+            return Err(ErrorTarea::general(format!(
+                "clave no admitida en el frontmatter: «{clave}»"
+            )));
+        }
+        if entradas.iter().any(|e| e.clave == clave) {
+            return Err(ErrorTarea::en(clave, "clave duplicada"));
+        }
+        entradas.push(Entrada {
+            clave: clave.to_owned(),
+            crudo: format!("{linea}\n"),
+        });
+    }
+    if !cerrado {
+        return Err(ErrorTarea::general("el frontmatter no se cierra con `---`"));
+    }
+    Ok(Partes {
+        entradas,
+        cuerpo: texto.get(consumido..).unwrap_or(""),
+    })
+}
+
+/// Opciones de lectura: duplicadas, claves de fusion, anclas y alias como
+/// error (ninguna tarea real los usa; un alias que crece es un vector de
+/// denegacion de servicio, F1-8).
+fn opciones() -> Options {
+    let mut o = Options::default();
+    o.duplicate_keys = DuplicateKeyPolicy::Error;
+    o.merge_keys = MergeKeyPolicy::Error;
+    let mut presupuesto = o.budget.unwrap_or_default();
+    presupuesto.max_aliases = 0;
+    presupuesto.max_anchors = 0;
+    o.budget = Some(presupuesto);
+    o
+}
+
+/// Valida el YAML entero y decodifica las claves conocidas en `T` (las
+/// desconocidas se validan como YAML y se ignoran: las conserva [`partir`]).
+pub(crate) fn decodificar<T: DeserializeOwned>(yaml: &str) -> Result<T, ErrorTarea> {
+    serde_saphyr::from_str_with_options::<T>(yaml, opciones())
+        .map_err(|e| ErrorTarea::general(format!("frontmatter YAML no valido: {e}")))
+}
+
+/// Cadena YAML entre comillas dobles (F1-9 punto 5): `\` y `"` escapados y
+/// los caracteres de control y separadores de linea como `\uXXXX`, para que
+/// cualquier texto salga como YAML valido de una sola linea.
+pub(crate) fn cadena(texto: &str) -> String {
+    let mut s = String::with_capacity(texto.len() + 2);
+    s.push('"');
+    for c in texto.chars() {
+        match c {
+            '\\' => s.push_str("\\\\"),
+            '"' => s.push_str("\\\""),
+            c if c.is_control() || matches!(c, '\u{2028}' | '\u{2029}' | '\u{feff}') => {
+                s.push_str(&format!("\\u{:04X}", u32::from(c)));
+            }
+            c => s.push(c),
+        }
+    }
+    s.push('"');
+    s
+}
+
+/// Cadena opcional: `null` o [`cadena`].
+pub(crate) fn opcional(texto: Option<&str>) -> String {
+    texto.map_or_else(|| "null".to_owned(), cadena)
+}
+
+/// Lista en flujo de cadenas: `[]` o `["a", "b"]`.
+pub(crate) fn lista(elementos: &[String]) -> String {
+    let dentro: Vec<String> = elementos.iter().map(|e| cadena(e)).collect();
+    format!("[{}]", dentro.join(", "))
+}
+
+/// Entero opcional: `null` o el numero.
+pub(crate) fn numero(n: Option<u64>) -> String {
+    n.map_or_else(|| "null".to_owned(), |n| n.to_string())
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn partir_separa_entradas_y_cuerpo_intacto() -> Result<(), ErrorTarea> {
+        let t = "\u{feff}---\r\nid: \"TASK-001\"\r\nroles:\r\n  a: {b: 1}\r\n# nota\r\nx: 1\r\n--- \r\n## Objetivo\r\n";
+        let p = partir(t)?;
+        let claves: Vec<&str> = p.entradas.iter().map(|e| e.clave.as_str()).collect();
+        assert_eq!(claves, ["id", "roles", "x"]);
+        assert_eq!(p.entradas[1].crudo, "roles:\n  a: {b: 1}\n# nota\n");
+        assert_eq!(p.cuerpo, "## Objetivo\r\n");
+        assert!(p.tiene("x") && !p.tiene("formato"));
+        Ok(())
+    }
+
+    #[test]
+    fn partir_rechaza_lo_que_el_separador_no_entiende() {
+        for (t, error) in [
+            ("sin frontmatter\n", "no empieza"),
+            ("---\nid: 1\n", "no se cierra"),
+            ("---\n  suelto\nid: 1\n---\n", "antes de la primera"),
+            ("---\n\"id\": 1\n---\n", "no admitida"),
+            ("---\nId: 1\n---\n", "no admitida"),
+            ("---\n? id\n: 1\n---\n", "sin clave"),
+            ("---\nsolo texto\n---\n", "sin clave"),
+            ("---\nid: 1\nid: 2\n---\n", "duplicada"),
+        ] {
+            let e = partir(t).err().map(|e| e.to_string());
+            assert!(
+                e.as_deref().is_some_and(|e| e.contains(error)),
+                "{t:?}: {e:?}"
+            );
+        }
+    }
+
+    #[derive(serde::Deserialize, Debug)]
+    struct Prueba {
+        a: Option<String>,
+    }
+
+    #[test]
+    fn decodificar_rechaza_duplicadas_anidadas_anclas_y_fusion() -> Result<(), ErrorTarea> {
+        for (yaml, motivo) in [
+            ("a: \"x\"\nb: {c: 1, c: 2}\n", "uplicate"),
+            ("a: &x \"v\"\nb: *x\n", "nchor"),
+            ("b: {c: 1}\nd:\n  <<: {c: 2}\n", "erge"),
+        ] {
+            // Control positivo: con las opciones por defecto, anclas y fusion
+            // se aceptarian; el rechazo es de las opciones de `opciones()`.
+            let e = decodificar::<Prueba>(yaml).err().map(|e| e.to_string());
+            assert!(
+                e.as_deref().is_some_and(|e| e.contains(motivo)),
+                "{yaml:?}: {e:?}"
+            );
+        }
+        assert!(serde_saphyr::from_str::<Prueba>("a: &x \"v\"\nb: *x\n").is_ok());
+        let p = decodificar::<Prueba>("a: \"C:\\\\ruta\"\nb: [1, 2]\n")?;
+        assert_eq!(p.a.as_deref(), Some("C:\\ruta"));
+        Ok(())
+    }
+
+    #[test]
+    fn cadena_escapa_lo_necesario() {
+        assert_eq!(cadena("C:\\r \"x\" ñ"), "\"C:\\\\r \\\"x\\\" ñ\"");
+        assert_eq!(cadena("a\nb\u{2028}"), "\"a\\u000Ab\\u2028\"");
+        assert_eq!(opcional(None), "null");
+        assert_eq!(lista(&[]), "[]");
+        assert_eq!(lista(&["a".into(), "b c".into()]), "[\"a\", \"b c\"]");
+        assert_eq!(numero(Some(7)), "7");
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
Cargo.lock                                                                                | 262 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{01-en-diseno => 02-en-curso}/TASK-020/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{01-en-diseno => 02-en-curso}/TASK-020/planificacion/plan-final.md                 |   0
 tareas/{01-en-diseno => 02-en-curso}/TASK-020/tarea.md                                    |   7 ++-
 4 files changed, 268 insertions(+), 1 deletion(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/tarea.rs"
````

````
crates/tc2/tests/tarea.rs | 441 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 441 insertions(+)
````
