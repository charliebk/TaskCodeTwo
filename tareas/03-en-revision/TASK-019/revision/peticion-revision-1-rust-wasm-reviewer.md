# Peticion de revision — TASK-019 (ronda 1)

- Tarea: TASK-019 — F2-4 Reserva de IDs por push atomico
- Rama revisada: feature/task-019-f2-4-reserva-de-ids-por-push-atomico
- Rama base: develop
- Commit revisado (HEAD): f7f9446522807d5cea787a20dc1286abdf525a6d
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-019 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-019/tarea.md
- tareas/03-en-revision/TASK-019/planificacion/
- crates/tc2/src/alta.rs
- crates/tc2/src/git/remoto.rs
- crates/tc2/src/lib.rs
- crates/tc2/tests/alta.rs

## Commits a revisar (git log develop..HEAD)

````
f7f9446 chore(TASK-019): autorrevision
c121de2 fix(TASK-019): rechazar U+2028 y U+2029 en el titulo
a1723eb feat(TASK-019): alta de tareas con reserva de ID por push atomico (tc2 new)
0ddcab1 chore(TASK-019): tarea en curso
````

## Diff de tu dominio (4 fichero(s) de 5; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/alta.rs b/crates/tc2/src/alta.rs
new file mode 100644
index 0000000..82c6c46
--- /dev/null
+++ b/crates/tc2/src/alta.rs
@@ -0,0 +1,300 @@
+//! Alta de tareas con reserva de ID (F2-4; ADR F1-2, punto 3, y F1-1).
+//!
+//! El ID se reserva con el push de la rama de estado: [`Alta`] calcula el
+//! siguiente ID sobre el estado fresco de cada intento (maximo + 1, nunca el
+//! recuento) y crea la carpeta fija `TASK-NNN/` con su `tarea.md`. Si otro
+//! clon publico antes, el push se rechaza, la transaccion re-ejecuta la
+//! operacion sobre lo nuevo y sale el numero siguiente. La carpeta publicada
+//! es el compare-and-swap: reservar un numero sin crearla no reservaria nada.
+
+use std::cell::Cell;
+use std::time::{SystemTime, UNIX_EPOCH};
+
+use crate::estado::{Cambio, Estado, Fallo, Operacion};
+
+/// Nombre de la operacion y de la orden del binario.
+pub const NOMBRE: &str = "new";
+
+/// Prefijo de las carpetas de tarea en la rama de estado.
+const PREFIJO: &str = "TASK-";
+
+/// Plantilla de `tarea.md` del ADR F1-9, punto 6 (formato 2). La sustituye el
+/// modelo de tarea de F2-5. `{id}`, `{titulo}` y `{fecha}` se rellenan;
+/// `rama` queda en `null` hasta F2-4b, que la deriva del ID.
+const PLANTILLA: &str = "---
+formato: 2
+id: \"{id}\"
+titulo: \"{titulo}\"
+tipo: \"feature\"
+sprint: 0
+etiquetas: []
+complejidad: null
+carril: null
+estado: \"planificada\"
+motivo: null
+plan_aprobado: false
+rama: null
+asignado_a: null
+roles:
+  diseno: {modelo: null, skills: [], regla: null}
+  implementacion: {modelo: \"sonnet\", skills: [], regla: null}
+  revision: {agente: \"general-purpose\", modelo: null, skills: [], regla: null}
+estimacion: null
+tokens_diseno: null
+tokens_implementacion: null
+tokens_revision: null
+creado: \"{fecha}\"
+actualizado: \"{fecha}\"
+dependencias: []
+---
+## Objetivo
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
+/// Alta de una tarea: reserva el siguiente ID y escribe su `tarea.md`.
+#[derive(Debug)]
+pub struct Alta {
+    titulo: String,
+    fecha: String,
+    id: Cell<Option<u32>>,
+}
+
+impl Alta {
+    /// Alta con `titulo` y la fecha de hoy (UTC).
+    pub fn nueva(titulo: &str) -> Self {
+        Self::con_fecha(titulo, &hoy())
+    }
+
+    /// Alta con una fecha `AAAA-MM-DD` dada.
+    pub fn con_fecha(titulo: &str, fecha: &str) -> Self {
+        Self {
+            titulo: titulo.to_owned(),
+            fecha: fecha.to_owned(),
+            id: Cell::new(None),
+        }
+    }
+
+    /// ID calculado en el ultimo intento (el que publico, si la transaccion
+    /// salio bien).
+    pub fn id(&self) -> Option<String> {
+        self.id.get().map(formato_id)
+    }
+}
+
+impl Operacion for Alta {
+    fn nombre(&self) -> &str {
+        NOMBRE
+    }
+    fn anotacion(&self) -> String {
+        format!("{NOMBRE}|{}", self.titulo)
+    }
+    fn exige_red(&self) -> bool {
+        // Reserva algo compartido: sin red falla, nunca queda pendiente (D2b).
+        true
+    }
+    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
+        let n = siguiente_id(&estado.rutas())?;
+        self.id.set(Some(n));
+        let id = formato_id(n);
+        Ok(vec![Cambio::nuevo(
+            format!("{id}/tarea.md"),
+            tarea_md(&id, &self.titulo, &self.fecha),
+        )])
+    }
+}
+
+/// Reconstruye un [`Alta`] desde su anotacion (para el catalogo).
+pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
+    let titulo = a.strip_prefix(NOMBRE)?.strip_prefix('|')?;
+    titulo_valido(titulo).then(|| Box::new(Alta::nueva(titulo)) as Box<dyn Operacion>)
+}
+
+/// Si `titulo` vale: no vacio y sin saltos de linea ni caracteres de control
+/// (tampoco los separadores U+2028 y U+2029, que YAML lee como salto).
+pub fn titulo_valido(titulo: &str) -> bool {
+    !titulo.trim().is_empty()
+        && !titulo
+            .chars()
+            .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
+}
+
+/// `TASK-NNN` con al menos tres cifras (`TASK-1000` sin renumerar).
+pub fn formato_id(n: u32) -> String {
+    format!("{PREFIJO}{n:03}")
+}
+
+/// El numero de la carpeta de tarea del primer componente de `ruta`.
+fn numero(ruta: &str) -> Option<u32> {
+    let primero = ruta.split('/').next()?;
+    let cifras = primero.strip_prefix(PREFIJO)?;
+    if cifras.is_empty() || !cifras.bytes().all(|b| b.is_ascii_digit()) {
+        return None;
+    }
+    cifras.parse().ok()
+}
+
+/// Siguiente ID: el maximo de las carpetas `TASK-<cifras>` mas uno (cuentan
+/// terminadas y descartadas, que no se borran; los huecos no se rellenan).
+pub fn siguiente_id(rutas: &[String]) -> Result<u32, Fallo> {
+    let maximo = rutas.iter().filter_map(|r| numero(r)).max().unwrap_or(0);
+    maximo
+        .checked_add(1)
+        .ok_or_else(|| Fallo::Error("no quedan IDs de tarea".to_owned()))
+}
+
+/// Escapa un texto para una cadena YAML entre comillas dobles (F1-9 punto 5).
+fn escapar(texto: &str) -> String {
+    texto.replace('\\', "\\\\").replace('"', "\\\"")
+}
+
+/// `tarea.md` de una tarea recien creada.
+pub fn tarea_md(id: &str, titulo: &str, fecha: &str) -> String {
+    PLANTILLA
+        .replace("{id}", id)
+        .replace("{fecha}", fecha)
+        // El titulo el ultimo: un `{fecha}` en el titulo no se sustituye.
+        .replace("{titulo}", &escapar(titulo))
+}
+
+/// Fecha de hoy en UTC, `AAAA-MM-DD`, sin dependencias.
+fn hoy() -> String {
+    let segundos = SystemTime::now()
+        .duration_since(UNIX_EPOCH)
+        .map(|d| d.as_secs())
+        .unwrap_or(0);
+    fecha_de_dias(i64::try_from(segundos / 86_400).unwrap_or(0))
+}
+
+/// Fecha civil de un numero de dias desde 1970-01-01 (algoritmo de Howard
+/// Hinnant, `civil_from_days`).
+fn fecha_de_dias(dias: i64) -> String {
+    let z = dias + 719_468;
+    let era = z.div_euclid(146_097);
+    let doe = z.rem_euclid(146_097);
+    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
+    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
+    let mp = (5 * doy + 2) / 153;
+    let d = doy - (153 * mp + 2) / 5 + 1;
+    let m = if mp < 10 { mp + 3 } else { mp - 9 };
+    let a = yoe + era * 400 + i64::from(m <= 2);
+    format!("{a:04}-{m:02}-{d:02}")
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn rutas(v: &[&str]) -> Vec<String> {
+        v.iter().map(|s| (*s).to_owned()).collect()
+    }
+
+    #[test]
+    fn siguiente_id_es_el_maximo_mas_uno() {
+        // TASK-007 es una descartada: cuenta igual; el hueco no se rellena.
+        let r = rutas(&[
+            "LEEME.md",
+            "ROADMAP.md",
+            "TASK-001/tarea.md",
+            "TASK-007/tarea.md",
+            "TASK-007/planificacion/plan-final.md",
+            "notas/TASK-050.md",
+            "TASK-1a/tarea.md",
+            "TASK-/tarea.md",
+            "TASK-+9/tarea.md",
+        ]);
+        assert_eq!(siguiente_id(&r), Ok(8));
+        assert_eq!(siguiente_id(&[]), Ok(1));
+        assert_eq!(siguiente_id(&rutas(&["TASK-999/tarea.md"])), Ok(1000));
+        assert_eq!(formato_id(1000), "TASK-1000");
+        assert_eq!(formato_id(8), "TASK-008");
+        let tope = rutas(&[&format!("TASK-{}/tarea.md", u32::MAX)]);
+        assert!(matches!(siguiente_id(&tope), Err(Fallo::Error(_))));
+    }
+
+    #[test]
+    fn plantilla_escapa_el_titulo_y_sigue_f1_9() {
+        let t = tarea_md("TASK-008", "C:\\ruta\\ \"x\" {fecha}", "2026-11-15");
+        assert!(
+            t.contains("titulo: \"C:\\\\ruta\\\\ \\\"x\\\" {fecha}\"\n"),
+            "{t}"
+        );
+        let claves: Vec<&str> = t
+            .lines()
+            .skip(1)
+            .take_while(|l| *l != "---")
+            .filter(|l| !l.starts_with(' '))
+            .filter_map(|l| l.split(':').next())
+            .collect();
+        assert_eq!(
+            claves,
+            [
+                "formato",
+                "id",
+                "titulo",
+                "tipo",
+                "sprint",
+                "etiquetas",
+                "complejidad",
+                "carril",
+                "estado",
+                "motivo",
+                "plan_aprobado",
+                "rama",
+                "asignado_a",
+                "roles",
+                "estimacion",
+                "tokens_diseno",
+                "tokens_implementacion",
+                "tokens_revision",
+                "creado",
+                "actualizado",
+                "dependencias",
+            ]
+        );
+        assert!(t.contains("id: \"TASK-008\"\n"));
+        assert!(t.contains("creado: \"2026-11-15\"\nactualizado: \"2026-11-15\"\n"));
+        assert!(t.contains("rama: null\n"));
+    }
+
+    #[test]
+    fn fechas_civiles() {
+        assert_eq!(fecha_de_dias(0), "1970-01-01");
+        assert_eq!(fecha_de_dias(11_016), "2000-02-29");
+        assert_eq!(fecha_de_dias(20_772), "2026-11-15");
+        assert_eq!(fecha_de_dias(-1), "1969-12-31");
+    }
+
+    #[test]
+    fn anotacion_ida_y_vuelta() {
+        let a = Alta::con_fecha("con | barra", "2026-11-15");
+        assert_eq!(a.anotacion(), "new|con | barra");
+        assert!(desde_anotacion(&a.anotacion()).is_some());
+        assert!(desde_anotacion("new|").is_none());
+        assert!(desde_anotacion("otra|x").is_none());
+        assert!(desde_anotacion("newx|t").is_none());
+        assert!(!titulo_valido("a\nb"));
+        assert!(!titulo_valido("a\tb"));
+        assert!(!titulo_valido("a\u{2028}b"));
+        assert!(!titulo_valido("a\u{85}b"));
+        assert!(!titulo_valido("   "));
+        assert!(titulo_valido("Título con ñ"));
+    }
+}
diff --git a/crates/tc2/src/git/remoto.rs b/crates/tc2/src/git/remoto.rs
index e08330a..557e7a6 100644
--- a/crates/tc2/src/git/remoto.rs
+++ b/crates/tc2/src/git/remoto.rs
@@ -230,10 +230,13 @@ fn intento_push(git: &Git, dir: &Path, op: &str, refspec: &str) -> Paso<Intento>
 /// `[remote rejected]` por contencion: otro push actualizo la misma ref a la
 /// vez y el remoto no pudo tomar su cerrojo (visto en F2-3 con dos procesos
 /// contra el mismo remoto). Es lo mismo que un non-fast-forward: la ref se
-/// movio; no es una denegacion.
+/// movio; no es una denegacion. `reference already exists`: dos clones
+/// crean a la vez una ref que no existia (la rama de estado en el primer
+/// `new`, visto en F2-4) y el remoto rechaza al segundo.
 fn es_contencion(resumen: &str) -> bool {
     let t = resumen.to_ascii_lowercase();
     [
+        "reference already exists",
         "cannot lock ref",
         "failed to lock",
         "unable to lock",
@@ -624,6 +627,9 @@ mod tests {
             "[remote rejected] (cannot lock ref 'refs/heads/taskcode/tareas': is at 1a but expected 2b)"
         ));
         assert!(es_contencion("[remote rejected] (failed to update ref)"));
+        assert!(es_contencion(
+            "[remote rejected] (reference already exists)"
+        ));
         assert!(!es_contencion(
             "[remote rejected] (pre-receive hook declined)"
         ));
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index e0b2f50..01f04c9 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -2,9 +2,11 @@
 //!
 //! Modulos: [`contrato`] (salida de toda operacion, ADR F1-4), [`git`]
 //! (capa de operaciones Git deterministas, F2-2) y [`estado`] (rama de
-//! estado `taskcode/tareas`, F2-3). Las capas que vienen (modelo de tarea de
-//! F2-5...) entran como modulos de esta biblioteca.
+//! estado `taskcode/tareas`, F2-3) y [`alta`] (alta de tareas con reserva de
+//! ID, F2-4). Las capas que vienen (modelo de tarea de F2-5...) entran como
+//! modulos de esta biblioteca.
 
+pub mod alta;
 pub mod contrato;
 pub mod estado;
 pub mod git;
@@ -27,7 +29,7 @@ pub struct Salida {
 }
 
 /// Uso del binario, para los mensajes de error.
-const USO: &str = "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json]";
+const USO: &str = "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\"";
 
 /// Interpreta los argumentos (sin el nombre del programa) y ejecuta la orden
 /// en la carpeta actual.
@@ -53,6 +55,13 @@ where
             };
             orden_git(orden, resto).salida(json)
         }
+        [orden, resto @ ..] if orden == alta::NOMBRE => {
+            let (json, resto) = match resto {
+                [j, r @ ..] if j == "--json" => (true, r),
+                r => (false, r),
+            };
+            orden_new(resto).salida(json)
+        }
         [] => Salida {
             codigo: 2,
             stdout: String::new(),
@@ -99,10 +108,68 @@ fn orden_git(orden: &str, args: &[String]) -> Informe {
     }
 }
 
+/// Catalogo de produccion: reconstruye las operaciones de la rama de estado
+/// desde su anotacion. Las operaciones nuevas se anaden aqui.
+fn catalogo(anotacion: &str) -> Option<Box<dyn estado::Operacion>> {
+    alta::desde_anotacion(anotacion)
+}
+
+/// `new "<titulo>"`: alta de una tarea con su ID reservado por el push de la
+/// rama de estado (F2-4). Sale `new ok id=TASK-NNN ...`.
+fn orden_new(args: &[String]) -> Informe {
+    let titulo = match args {
+        [t] if alta::titulo_valido(t) => t,
+        [_] => {
+            return Informe::nuevo(alta::NOMBRE, Resultado::Uso)
+                .con("motivo", "titulo-invalido")
+                .diagnostico("el titulo no puede estar vacio ni llevar saltos de linea");
+        }
+        _ => {
+            return Informe::nuevo(alta::NOMBRE, Resultado::Uso)
+                .con("motivo", "sin-titulo")
+                .diagnostico("uso: new [--json] \"<titulo>\"");
+        }
+    };
+    let dir = match std::env::current_dir() {
+        Ok(d) => d,
+        Err(e) => {
+            return Informe::nuevo(alta::NOMBRE, Resultado::Error)
+                .diagnostico(format!("no se puede leer la carpeta actual: {e}"));
+        }
+    };
+    let g = git::Git::nuevo();
+    let op = alta::Alta::nueva(titulo);
+    let c = estado::Contexto::nuevo(&g, &dir, &catalogo);
+    let mut i = estado::transaccion(&c, &op);
+    alta::NOMBRE.clone_into(&mut i.comando);
+    if i.resultado == Resultado::Ok
+        && let Some(id) = op.id()
+    {
+        i.claves
+            .insert(0, ("id".to_owned(), contrato::Valor::Texto(id)));
+    }
+    i
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
 
+    #[test]
+    fn new_sin_titulo_o_con_salto_de_linea_es_uso() {
+        for (args, motivo) in [
+            (vec!["new"], "sin-titulo"),
+            (vec!["new", "--json"], "sin-titulo"),
+            (vec!["new", "a", "b"], "sin-titulo"),
+            (vec!["new", "linea\notra"], "titulo-invalido"),
+            (vec!["new", "  "], "titulo-invalido"),
+        ] {
+            let s = ejecutar(&args);
+            assert_eq!(s.codigo, 2, "{args:?}");
+            assert!(s.stdout.contains(motivo), "{args:?}: {}", s.stdout);
+        }
+    }
+
     #[test]
     fn version_corta_y_larga() {
         for a in ["--version", "-V"] {
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-019/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-019/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-019/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/alta.rs"
````

````
crates/tc2/tests/alta.rs | 176 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 176 insertions(+)
````
