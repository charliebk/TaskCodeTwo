# Peticion de revision — TASK-017 (ronda 1)

- Tarea: TASK-017 — F2-2 Capa de operaciones Git deterministas
- Rama revisada: feature/task-017-f2-2-capa-de-operaciones-git-determinist
- Rama base: develop
- Commit revisado (HEAD): 7d2d4c06fe9dd81556612abcb562219d65e6fa8a
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-017 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-017/tarea.md
- tareas/03-en-revision/TASK-017/planificacion/
- crates/tc2/src/contrato.rs
- crates/tc2/src/git/diff.rs
- crates/tc2/src/git/guardar.rs
- crates/tc2/src/git/mod.rs
- crates/tc2/src/git/puerto.rs
- crates/tc2/src/git/rama.rs
- crates/tc2/src/git/remoto.rs
- crates/tc2/src/git/tag.rs
- crates/tc2/src/git/worktree.rs
- crates/tc2/src/lib.rs
- crates/tc2/tests/comun/mod.rs
- crates/tc2/tests/git_binario.rs
- crates/tc2/tests/git_ciclo.rs
- crates/tc2/tests/git_conflicto.rs
- crates/tc2/tests/git_push.rs

## Commits a revisar (git log develop..HEAD)

````
7d2d4c0 chore(TASK-017): criterios y autorrevision
743a5d3 fix(TASK-017): no reescribir info/exclude si no se puede leer
740fd7a test(TASK-017): tests de la capa Git contra repos reales
a49bc89 feat(TASK-017): contrato de salida, puerto Git y operaciones locales
081b8ac chore(TASK-017): tarea en curso
````

## Diff de tu dominio (15 fichero(s) de 15; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/contrato.rs b/crates/tc2/src/contrato.rs
new file mode 100644
index 0000000..5ceb5c2
--- /dev/null
+++ b/crates/tc2/src/contrato.rs
@@ -0,0 +1,321 @@
+//! Contrato de salida de las operaciones (ADR F1-4, decisiones 4 y 5).
+//!
+//! Toda operacion devuelve un [`Informe`]. En el caso normal se ve como una
+//! sola linea `<comando> <resultado> clave=valor...` en stdout, sin ANSI; el
+//! diagnostico va a stderr y solo si la operacion no salio bien. Con `--json`
+//! la misma informacion sale como un objeto JSON en una linea.
+//!
+//! La tabla de codigos de salida vive aqui y solo aqui: la propone F1-4 y la
+//! fija F2-3, que solo tendra que tocar este fichero.
+
+use crate::Salida;
+
+/// Resultado de una operacion. Cada uno lleva su codigo de salida.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Resultado {
+    /// Hecho.
+    Ok,
+    /// Ya estaba hecho: no se ha cambiado nada.
+    SinCambios,
+    /// La operacion no procede; el motivo va en la linea. No se reintenta.
+    Rechazada,
+    /// No se pudo hablar con el remoto.
+    Sinred,
+    /// Se agotaron los reintentos sin escribir nada.
+    Agotada,
+    /// Hay un conflicto que resolver a mano.
+    Conflicto,
+    /// Puertas pendientes en la plataforma (lo emite el puerto Plataforma).
+    Esperando,
+    /// La llamada esta mal formada.
+    Uso,
+    /// Fallo interno: no se reintenta, se avisa a la persona.
+    Error,
+}
+
+impl Resultado {
+    /// Palabra del resultado en la linea de contrato.
+    pub fn texto(self) -> &'static str {
+        match self {
+            Self::Ok => "ok",
+            Self::SinCambios => "sin-cambios",
+            Self::Rechazada => "rechazada",
+            Self::Sinred => "sinred",
+            Self::Agotada => "agotada",
+            Self::Conflicto => "conflicto",
+            Self::Esperando => "esperando",
+            Self::Uso => "uso",
+            Self::Error => "error",
+        }
+    }
+
+    /// Codigo de salida del proceso (tabla de F1-4, decision 5).
+    pub fn codigo(self) -> u8 {
+        match self {
+            Self::Ok | Self::SinCambios => 0,
+            Self::Error => 1,
+            Self::Uso => 2,
+            Self::Rechazada => 3,
+            Self::Sinred => 4,
+            Self::Agotada => 5,
+            Self::Conflicto => 6,
+            Self::Esperando => 7,
+        }
+    }
+
+    /// Si quien lo recibe puede seguir sin hacer nada mas.
+    pub fn es_exito(self) -> bool {
+        self.codigo() == 0
+    }
+}
+
+/// Valor de una clave: texto suelto o lista (se escribe separada por comas).
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum Valor {
+    /// Un valor.
+    Texto(String),
+    /// Varios valores; en la linea, separados por comas.
+    Lista(Vec<String>),
+}
+
+/// Lo que devuelve una operacion.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Informe {
+    /// Nombre de la operacion o del comando (`guardar`, `push`...).
+    pub comando: String,
+    /// Como termino.
+    pub resultado: Resultado,
+    /// Claves en el orden en que se escriben.
+    pub claves: Vec<(String, Valor)>,
+    /// Una linea que dice que hacer; solo se muestra si no fue un exito.
+    pub diagnostico: Option<String>,
+}
+
+impl Informe {
+    /// Informe sin claves.
+    pub fn nuevo(comando: &str, resultado: Resultado) -> Self {
+        Self {
+            comando: comando.to_owned(),
+            resultado,
+            claves: Vec::new(),
+            diagnostico: None,
+        }
+    }
+
+    /// Anade una clave de texto.
+    #[must_use]
+    pub fn con(mut self, clave: &str, valor: impl Into<String>) -> Self {
+        self.claves
+            .push((clave.to_owned(), Valor::Texto(valor.into())));
+        self
+    }
+
+    /// Anade una clave de lista.
+    #[must_use]
+    pub fn con_lista(mut self, clave: &str, valores: Vec<String>) -> Self {
+        self.claves.push((clave.to_owned(), Valor::Lista(valores)));
+        self
+    }
+
+    /// Fija el diagnostico (una linea).
+    #[must_use]
+    pub fn diagnostico(mut self, texto: impl Into<String>) -> Self {
+        self.diagnostico = Some(texto.into());
+        self
+    }
+
+    /// Valor de texto de una clave, si existe.
+    pub fn valor(&self, clave: &str) -> Option<&str> {
+        self.claves.iter().find_map(|(k, v)| match v {
+            Valor::Texto(t) if k == clave => Some(t.as_str()),
+            _ => None,
+        })
+    }
+
+    /// Linea de contrato, sin salto final.
+    pub fn linea(&self) -> String {
+        let mut s = format!("{} {}", self.comando, self.resultado.texto());
+        for (k, v) in &self.claves {
+            s.push(' ');
+            s.push_str(k);
+            s.push('=');
+            match v {
+                Valor::Texto(t) => s.push_str(&escapar_valor(t, false)),
+                Valor::Lista(l) => {
+                    let partes: Vec<String> = l.iter().map(|e| escapar_valor(e, true)).collect();
+                    s.push_str(&partes.join(","));
+                }
+            }
+        }
+        s
+    }
+
+    /// La misma informacion en JSON, en una linea.
+    pub fn json(&self) -> String {
+        let mut s = format!(
+            "{{\"comando\":{},\"resultado\":{},\"codigo\":{},\"claves\":{{",
+            cadena_json(&self.comando),
+            cadena_json(self.resultado.texto()),
+            self.resultado.codigo()
+        );
+        for (i, (k, v)) in self.claves.iter().enumerate() {
+            if i > 0 {
+                s.push(',');
+            }
+            s.push_str(&cadena_json(k));
+            s.push(':');
+            match v {
+                Valor::Texto(t) => s.push_str(&cadena_json(t)),
+                Valor::Lista(l) => {
+                    let partes: Vec<String> = l.iter().map(|e| cadena_json(e)).collect();
+                    s.push('[');
+                    s.push_str(&partes.join(","));
+                    s.push(']');
+                }
+            }
+        }
+        s.push('}');
+        if let Some(d) = self.diagnostico_visible() {
+            s.push_str(",\"diagnostico\":");
+            s.push_str(&cadena_json(d));
+        }
+        s.push('}');
+        s
+    }
+
+    fn diagnostico_visible(&self) -> Option<&str> {
+        if self.resultado.es_exito() {
+            None
+        } else {
+            self.diagnostico.as_deref()
+        }
+    }
+
+    /// Lo convierte en la salida del proceso.
+    pub fn salida(&self, json: bool) -> Salida {
+        let stdout = if json { self.json() } else { self.linea() };
+        let stderr = match self.diagnostico_visible() {
+            Some(d) => format!("{}\n", una_linea(d)),
+            None => String::new(),
+        };
+        Salida {
+            codigo: self.resultado.codigo(),
+            stdout: format!("{stdout}\n"),
+            stderr,
+        }
+    }
+}
+
+impl From<Informe> for Salida {
+    fn from(i: Informe) -> Self {
+        i.salida(false)
+    }
+}
+
+/// Hash abreviado a 12 caracteres, como fija el contrato.
+pub fn corto(hash: &str) -> String {
+    hash.chars().take(12).collect()
+}
+
+/// Los valores no llevan espacios: espacio, `%`, controles (y `,` dentro de
+/// una lista) se escriben como `%XX`, de forma reversible.
+fn escapar_valor(v: &str, en_lista: bool) -> String {
+    let mut s = String::with_capacity(v.len());
+    for c in v.chars() {
+        if c == '%' || c.is_whitespace() || c.is_control() || (en_lista && c == ',') {
+            let mut buf = [0u8; 4];
+            for b in c.encode_utf8(&mut buf).bytes() {
+                s.push_str(&format!("%{b:02X}"));
+            }
+        } else {
+            s.push(c);
+        }
+    }
+    s
+}
+
+fn una_linea(t: &str) -> String {
+    t.lines().map(str::trim).collect::<Vec<_>>().join(" ")
+}
+
+fn cadena_json(t: &str) -> String {
+    let mut s = String::with_capacity(t.len() + 2);
+    s.push('"');
+    for c in t.chars() {
+        match c {
+            '"' => s.push_str("\\\""),
+            '\\' => s.push_str("\\\\"),
+            '\n' => s.push_str("\\n"),
+            '\r' => s.push_str("\\r"),
+            '\t' => s.push_str("\\t"),
+            c if (c as u32) < 0x20 => s.push_str(&format!("\\u{:04x}", c as u32)),
+            c => s.push(c),
+        }
+    }
+    s.push('"');
+    s
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn codigos_de_la_tabla_de_f1_4() {
+        let tabla = [
+            (Resultado::Ok, "ok", 0),
+            (Resultado::SinCambios, "sin-cambios", 0),
+            (Resultado::Error, "error", 1),
+            (Resultado::Uso, "uso", 2),
+            (Resultado::Rechazada, "rechazada", 3),
+            (Resultado::Sinred, "sinred", 4),
+            (Resultado::Agotada, "agotada", 5),
+            (Resultado::Conflicto, "conflicto", 6),
+            (Resultado::Esperando, "esperando", 7),
+        ];
+        for (r, t, c) in tabla {
+            assert_eq!((r.texto(), r.codigo()), (t, c));
+        }
+    }
+
+    #[test]
+    fn linea_de_contrato_y_json_con_las_mismas_claves() {
+        let i = Informe::nuevo("guardar", Resultado::Ok)
+            .con("commit", corto("48d72d1e513bffffffffffffffffffffffffffff"))
+            .con("nota", "con espacio y 100%")
+            .con_lista("ficheros", vec!["a,b.txt".into(), "c.txt".into()])
+            .diagnostico("no se ve en un exito");
+        let s = i.salida(false);
+        assert_eq!(s.codigo, 0);
+        assert_eq!(
+            s.stdout,
+            "guardar ok commit=48d72d1e513b nota=con%20espacio%20y%20100%25 ficheros=a%2Cb.txt,c.txt\n"
+        );
+        assert!(s.stderr.is_empty());
+        assert_eq!(s.stdout.lines().count(), 1);
+        assert!(!s.stdout.contains('\u{1b}'));
+        let j = i.salida(true);
+        assert_eq!(
+            j.stdout,
+            "{\"comando\":\"guardar\",\"resultado\":\"ok\",\"codigo\":0,\"claves\":{\"commit\":\"48d72d1e513b\",\"nota\":\"con espacio y 100%\",\"ficheros\":[\"a,b.txt\",\"c.txt\"]}}\n"
+        );
+    }
+
+    #[test]
+    fn el_diagnostico_va_a_stderr_solo_si_falla() {
+        let s = Informe::nuevo("push", Resultado::Rechazada)
+            .con("motivo", "no-fast-forward")
+            .diagnostico("lanza\nactualizar-rama")
+            .salida(false);
+        assert_eq!(s.codigo, 3);
+        assert_eq!(s.stdout, "push rechazada motivo=no-fast-forward\n");
+        assert_eq!(s.stderr, "lanza actualizar-rama\n");
+        let j = Informe::nuevo("x", Resultado::Uso)
+            .diagnostico("di \"algo\"")
+            .salida(true);
+        assert!(
+            j.stdout
+                .ends_with(",\"diagnostico\":\"di \\\"algo\\\"\"}\n")
+        );
+    }
+}
diff --git a/crates/tc2/src/git/diff.rs b/crates/tc2/src/git/diff.rs
new file mode 100644
index 0000000..635204f
--- /dev/null
+++ b/crates/tc2/src/git/diff.rs
@@ -0,0 +1,55 @@
+//! `diff-tarea`: resumen en una linea de lo que cambia la tarea frente a su
+//! base (ADR F1-4). El diff completo, a peticion, es de `review` (F6-2).
+
+use std::path::Path;
+
+use super::{Git, Paso, Subcomando, base_de_rama, commit_de, exigir, rama_actual};
+use crate::contrato::{Informe, Resultado, corto};
+
+/// Ficheros y lineas que cambian desde el `merge-base` con la base de la rama
+/// (incluye lo no guardado del worktree; no los ficheros sin seguimiento).
+pub fn diff_tarea(git: &Git, dir: &Path) -> Informe {
+    diff_paso(git, dir).unwrap_or_else(|i| i)
+}
+
+fn diff_paso(git: &Git, dir: &Path) -> Paso<Informe> {
+    const OP: &str = "diff-tarea";
+    let Some(rama) = rama_actual(git, dir, OP)? else {
+        return Ok(Informe::nuevo(OP, Resultado::Rechazada)
+            .con("motivo", "sin-rama")
+            .diagnostico("lanzalo en el worktree de la tarea"));
+    };
+    let base = base_de_rama(&rama);
+    if commit_de(git, dir, OP, base)?.is_none() {
+        return Ok(Informe::nuevo(OP, Resultado::Rechazada)
+            .con("base", base)
+            .con("motivo", "base-no-existe"));
+    }
+    let desde = exigir(git, dir, OP, Subcomando::MergeBase, &["HEAD", base])?;
+    let desde = desde.trim();
+    let numstat = exigir(git, dir, OP, Subcomando::Diff, &["--numstat", "-z", desde])?;
+    let (mut ficheros, mut mas, mut menos) = (0usize, 0u64, 0u64);
+    // Con -z: `<mas>\t<menos>\t<ruta>\0`; en binarios, `-\t-\t`; en un
+    // renombrado la ruta va vacia y siguen dos campos con las rutas.
+    let mut campos = numstat.split('\0');
+    while let Some(c) = campos.next() {
+        if c.is_empty() {
+            continue;
+        }
+        let mut partes = c.splitn(3, '\t');
+        let (a, b, ruta) = (partes.next(), partes.next(), partes.next());
+        if ruta == Some("") {
+            campos.next();
+            campos.next();
+        }
+        ficheros += 1;
+        mas += a.and_then(|x| x.parse().ok()).unwrap_or(0);
+        menos += b.and_then(|x| x.parse().ok()).unwrap_or(0);
+    }
+    Ok(Informe::nuevo(OP, Resultado::Ok)
+        .con("base", base)
+        .con("desde", corto(desde))
+        .con("ficheros", ficheros.to_string())
+        .con("mas", mas.to_string())
+        .con("menos", menos.to_string()))
+}
diff --git a/crates/tc2/src/git/guardar.rs b/crates/tc2/src/git/guardar.rs
new file mode 100644
index 0000000..881e33c
--- /dev/null
+++ b/crates/tc2/src/git/guardar.rs
@@ -0,0 +1,209 @@
+//! `guardar "<mensaje>" <rutas...>`: commit de las rutas dadas y solo de
+//! ellas, nunca `add -A` (RF-23, ADR F1-4 D2a). Con un merge abierto (lo deja
+//! `actualizar-rama` tras un conflicto) cierra el merge si las rutas resuelven
+//! todos los conflictos y ya no tienen marcas.
+
+use std::path::{Component, Path};
+
+use super::remoto::{merge_abierto, sin_resolver};
+use super::{Git, Paso, Subcomando, commit_de, entradas_status, exigir};
+use crate::contrato::{Informe, Resultado, corto};
+
+const OP: &str = "guardar";
+
+/// Guarda `rutas` (relativas a `dir`) con `mensaje`.
+pub fn guardar(git: &Git, dir: &Path, mensaje: &str, rutas: &[&str]) -> Informe {
+    guardar_paso(git, dir, mensaje, rutas).unwrap_or_else(|i| i)
+}
+
+fn uso(motivo: &str) -> Informe {
+    Informe::nuevo(OP, Resultado::Uso)
+        .con("motivo", motivo)
+        .diagnostico("uso: guardar \"<mensaje>\" <rutas...>")
+}
+
+fn guardar_paso(git: &Git, dir: &Path, mensaje: &str, rutas: &[&str]) -> Paso<Informe> {
+    if mensaje.trim().is_empty() {
+        return Err(uso("mensaje-vacio"));
+    }
+    if rutas.is_empty() {
+        return Err(uso("sin-rutas"));
+    }
+    let prefijo = exigir(git, dir, OP, Subcomando::RevParse, &["--show-prefix"])?;
+    let mut desde_raiz = Vec::with_capacity(rutas.len());
+    for r in rutas {
+        match relativa_a_la_raiz(prefijo.trim(), r) {
+            Some(p) => desde_raiz.push(p),
+            None => return Err(uso("ruta-fuera-del-repo")),
+        }
+    }
+    let mut args = vec!["--"];
+    args.extend_from_slice(rutas);
+
+    if merge_abierto(git, dir, OP)? {
+        let pendientes: Vec<String> = sin_resolver(git, dir, OP)?
+            .into_iter()
+            .filter(|f| !desde_raiz.contains(f))
+            .collect();
+        if !pendientes.is_empty() {
+            return Ok(Informe::nuevo(OP, Resultado::Rechazada)
+                .con("motivo", "conflicto-sin-resolver")
+                .con_lista("ficheros", pendientes.clone())
+                .diagnostico(format!(
+                    "resuelve {} y guardalo con el resto",
+                    pendientes.join(" ")
+                )));
+        }
+        let con_marcas: Vec<String> = rutas
+            .iter()
+            .filter(|r| tiene_marcas(&dir.join(r)))
+            .map(|r| (*r).to_owned())
+            .collect();
+        if !con_marcas.is_empty() {
+            return Ok(Informe::nuevo(OP, Resultado::Rechazada)
+                .con("motivo", "marcas-de-conflicto")
+                .con_lista("ficheros", con_marcas.clone())
+                .diagnostico(format!(
+                    "quita las marcas de conflicto de {}",
+                    con_marcas.join(" ")
+                )));
+        }
+        let estado = exigir(
+            git,
+            dir,
+            OP,
+            Subcomando::Status,
+            &["--porcelain", "-z", "--untracked-files=all"],
+        )?;
+        let utiles = conocidas(dir, rutas, &desde_raiz, &entradas_status(&estado));
+        if !utiles.is_empty() {
+            let mut add = vec!["--"];
+            add.extend_from_slice(&utiles);
+            exigir(git, dir, OP, Subcomando::Add, &add)?;
+        }
+        // Durante un merge el commit es del indice entero (el merge), no
+        // parcial: git no admite rutas aqui.
+        exigir(git, dir, OP, Subcomando::Commit, &["-q", "-m", mensaje])?;
+        let head = commit_de(git, dir, OP, "HEAD")?.unwrap_or_default();
+        return Ok(Informe::nuevo(OP, Resultado::Ok)
+            .con("commit", corto(&head))
+            .con("tipo", "merge")
+            .con("ficheros", rutas.len().to_string())
+            .con("sin-guardar", sin_guardar(git, dir)?.to_string()));
+    }
+
+    let mut status = vec!["--porcelain", "-z", "--untracked-files=all"];
+    status.extend_from_slice(&args);
+    let entradas = entradas_status(&exigir(git, dir, OP, Subcomando::Status, &status)?);
+    let cambios = entradas.len();
+    if cambios == 0 {
+        return Ok(Informe::nuevo(OP, Resultado::SinCambios)
+            .con("sin-guardar", sin_guardar(git, dir)?.to_string()));
+    }
+    // Solo las rutas que git conoce o que existen: una ruta sin cambios ni
+    // fichero haria fallar el `add` entero. `add -- <rutas>` recoge tambien
+    // los borrados (git >= 2.0) y nunca toca otras rutas.
+    let utiles = conocidas(dir, rutas, &desde_raiz, &entradas);
+    let mut add = vec!["--"];
+    add.extend_from_slice(&utiles);
+    exigir(git, dir, OP, Subcomando::Add, &add)?;
+    let mut commit = vec!["-q", "-m", mensaje, "--only", "--"];
+    commit.extend_from_slice(&utiles);
+    exigir(git, dir, OP, Subcomando::Commit, &commit)?;
+    let head = commit_de(git, dir, OP, "HEAD")?.unwrap_or_default();
+    Ok(Informe::nuevo(OP, Resultado::Ok)
+        .con("commit", corto(&head))
+        .con("ficheros", cambios.to_string())
+        .con("sin-guardar", sin_guardar(git, dir)?.to_string()))
+}
+
+/// Rutas que existen en disco o que aparecen en el status (borradas).
+fn conocidas<'a>(
+    dir: &Path,
+    rutas: &[&'a str],
+    desde_raiz: &[String],
+    status: &[(String, String)],
+) -> Vec<&'a str> {
+    rutas
+        .iter()
+        .zip(desde_raiz)
+        .filter(|(r, raiz)| {
+            dir.join(r).symlink_metadata().is_ok()
+                || status.iter().any(|(_, s)| {
+                    s == *raiz
+                        || s.strip_prefix(raiz.as_str())
+                            .is_some_and(|resto| resto.starts_with('/'))
+                })
+        })
+        .map(|(r, _)| *r)
+        .collect()
+}
+
+/// Cambios que quedan en el worktree sin guardar (se informan, no se tocan).
+fn sin_guardar(git: &Git, dir: &Path) -> Paso<usize> {
+    let s = exigir(
+        git,
+        dir,
+        OP,
+        Subcomando::Status,
+        &["--porcelain", "-z", "--untracked-files=all"],
+    )?;
+    Ok(entradas_status(&s).len())
+}
+
+/// Lineas de marca de conflicto que deja git (`<<<<<<< ` y `>>>>>>> `).
+fn tiene_marcas(fichero: &Path) -> bool {
+    std::fs::read(fichero).is_ok_and(|b| {
+        String::from_utf8_lossy(&b).lines().any(|l| {
+            l.starts_with("<<<<<<< ")
+                || l.starts_with(">>>>>>> ")
+                || l == "<<<<<<<"
+                || l == ">>>>>>>"
+        })
+    })
+}
+
+/// Ruta relativa a la raiz del worktree, con `/`, resolviendo `.` y `..`.
+/// `None` si sale del worktree o es absoluta.
+fn relativa_a_la_raiz(prefijo: &str, ruta: &str) -> Option<String> {
+    let mut partes: Vec<String> = prefijo
+        .split('/')
+        .filter(|p| !p.is_empty())
+        .map(str::to_owned)
+        .collect();
+    let ruta = ruta.replace('\\', "/");
+    for c in Path::new(&ruta).components() {
+        match c {
+            Component::Normal(p) => partes.push(p.to_string_lossy().into_owned()),
+            Component::CurDir => {}
+            Component::ParentDir => {
+                partes.pop()?;
+            }
+            Component::RootDir | Component::Prefix(_) => return None,
+        }
+    }
+    Some(partes.join("/"))
+}
+
+#[cfg(test)]
+mod tests {
+    use super::relativa_a_la_raiz;
+
+    #[test]
+    fn rutas_relativas_a_la_raiz() {
+        assert_eq!(
+            relativa_a_la_raiz("", "a/b.txt").as_deref(),
+            Some("a/b.txt")
+        );
+        assert_eq!(
+            relativa_a_la_raiz("sub/", "./x.txt").as_deref(),
+            Some("sub/x.txt")
+        );
+        assert_eq!(
+            relativa_a_la_raiz("sub/", "..\\y.txt").as_deref(),
+            Some("y.txt")
+        );
+        assert_eq!(relativa_a_la_raiz("", "../fuera.txt"), None);
+        assert_eq!(relativa_a_la_raiz("", "/abs.txt"), None);
+    }
+}
diff --git a/crates/tc2/src/git/mod.rs b/crates/tc2/src/git/mod.rs
new file mode 100644
index 0000000..8c0f2bd
--- /dev/null
+++ b/crates/tc2/src/git/mod.rs
@@ -0,0 +1,238 @@
+//! Capa de operaciones Git deterministas (F2-2, ADR F1-4).
+//!
+//! Cada operacion es una funcion que recibe el puerto [`Git`] y la carpeta
+//! desde la que se lanza, y devuelve un [`Informe`] con el contrato de
+//! salida. Son publicas en la biblioteca para que las compongan los comandos
+//! de fase (F2-3, F3-6, F7-3); en el binario solo se ven `guardar` y
+//! `actualizar-rama` (D1a).
+//!
+//! El remoto es siempre `origin` y las bases, `origin/develop` u
+//! `origin/main` (ADR F1-3).
+
+mod diff;
+mod guardar;
+mod puerto;
+mod rama;
+mod remoto;
+mod tag;
+mod worktree;
+
+pub use diff::diff_tarea;
+pub use guardar::guardar;
+pub use puerto::{ErrorGit, Git, SalidaGit, Subcomando};
+pub use rama::{comprobar_id, crear_rama_y_worktree, localizar_principal, reanudar, validar_base};
+pub use remoto::{ESPERAS_PUSH, actualizar_principal, actualizar_rama, fetch, push};
+pub use tag::tag_anotado;
+pub use worktree::quitar_worktree;
+
+use std::path::{Component, Path, PathBuf};
+
+use crate::contrato::{Informe, Resultado};
+
+/// Nombre del remoto (ADR F1-3).
+pub const REMOTO: &str = "origin";
+
+/// Carpeta de los worktrees de tarea, relativa al worktree principal.
+pub const CARPETA_WORKTREES: &str = ".taskcode/worktrees";
+
+/// Resultado interno: el `Err` ya es el informe que se devuelve.
+pub(crate) type Paso<T> = Result<T, Informe>;
+
+/// Lanza `git` y convierte el fallo al lanzar en un informe `error`.
+pub(crate) fn correr(
+    git: &Git,
+    dir: &Path,
+    op: &str,
+    sub: Subcomando,
+    args: &[&str],
+) -> Paso<SalidaGit> {
+    git.ejecutar(dir, sub, args).map_err(|e| {
+        Informe::nuevo(op, Resultado::Error)
+            .con("git", sub.nombre())
+            .diagnostico(e.to_string())
+    })
+}
+
+/// Como [`correr`], pero un codigo distinto de 0 tambien es `error`.
+pub(crate) fn exigir(
+    git: &Git,
+    dir: &Path,
+    op: &str,
+    sub: Subcomando,
+    args: &[&str],
+) -> Paso<String> {
+    let s = correr(git, dir, op, sub, args)?;
+    if s.exito() {
+        Ok(s.stdout)
+    } else {
+        Err(Informe::nuevo(op, Resultado::Error)
+            .con("git", sub.nombre())
+            .diagnostico(format!(
+                "git {} fallo: {}",
+                sub.nombre(),
+                s.primera_linea_error()
+            )))
+    }
+}
+
+/// Commit al que apunta `referencia`, o `None` si no existe.
+pub(crate) fn commit_de(git: &Git, dir: &Path, op: &str, referencia: &str) -> Paso<Option<String>> {
+    let r = format!("{referencia}^{{commit}}");
+    let s = correr(
+        git,
+        dir,
+        op,
+        Subcomando::RevParse,
+        &["-q", "--verify", "--end-of-options", &r],
+    )?;
+    Ok(s.exito().then(|| s.stdout.trim().to_owned()))
+}
+
+/// Si `a` esta contenido en `b`.
+pub(crate) fn es_ancestro(git: &Git, dir: &Path, op: &str, a: &str, b: &str) -> Paso<bool> {
+    let s = correr(
+        git,
+        dir,
+        op,
+        Subcomando::MergeBase,
+        &["--is-ancestor", a, b],
+    )?;
+    match s.codigo {
+        0 => Ok(true),
+        1 => Ok(false),
+        _ => Err(Informe::nuevo(op, Resultado::Error)
+            .diagnostico(format!("git merge-base fallo: {}", s.primera_linea_error()))),
+    }
+}
+
+/// Rama local en la que esta `dir`, o `None` si HEAD esta suelto.
+pub(crate) fn rama_actual(git: &Git, dir: &Path, op: &str) -> Paso<Option<String>> {
+    let s = exigir(
+        git,
+        dir,
+        op,
+        Subcomando::RevParse,
+        &["--symbolic-full-name", "HEAD"],
+    )?;
+    Ok(s.trim().strip_prefix("refs/heads/").map(str::to_owned))
+}
+
+/// `develop` y `main` no se tocan desde una operacion de tarea.
+pub(crate) fn es_protegida(rama: &str) -> bool {
+    rama == "develop" || rama == "main"
+}
+
+/// Base de una rama de tarea segun su prefijo (ADR F1-3): `hotfix/` sale de
+/// `main`; el resto, de `develop`.
+pub fn base_de_rama(rama: &str) -> &'static str {
+    if rama.starts_with("hotfix/") {
+        "origin/main"
+    } else {
+        "origin/develop"
+    }
+}
+
+/// Un worktree de `git worktree list --porcelain`.
+#[derive(Debug, Clone)]
+pub(crate) struct Worktree {
+    pub ruta: PathBuf,
+    pub rama: Option<String>,
+}
+
+pub(crate) fn worktrees(git: &Git, dir: &Path, op: &str) -> Paso<Vec<Worktree>> {
+    let s = exigir(git, dir, op, Subcomando::Worktree, &["list", "--porcelain"])?;
+    let mut v: Vec<Worktree> = Vec::new();
+    for l in s.lines() {
+        if let Some(r) = l.strip_prefix("worktree ") {
+            v.push(Worktree {
+                ruta: PathBuf::from(r),
+                rama: None,
+            });
+        } else if let (Some(r), Some(w)) = (l.strip_prefix("branch refs/heads/"), v.last_mut()) {
+            w.rama = Some(r.to_owned());
+        }
+    }
+    Ok(v)
+}
+
+/// Si dos rutas son la misma carpeta (en Windows `git` escribe `C:/...` y el
+/// sistema puede dar nombres cortos 8.3: se comparan canonicas si existen).
+pub(crate) fn misma_ruta(a: &Path, b: &Path) -> bool {
+    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
+        (Ok(x), Ok(y)) => x == y,
+        _ => normalizar(a) == normalizar(b),
+    }
+}
+
+fn normalizar(p: &Path) -> Vec<String> {
+    p.components()
+        .filter(|c| !matches!(c, Component::CurDir))
+        .map(|c| {
+            let t = c.as_os_str().to_string_lossy().replace('\\', "/");
+            if cfg!(windows) { t.to_lowercase() } else { t }
+        })
+        .collect()
+}
+
+/// Entradas de `git status --porcelain -z`: (estado XY, ruta). Las de
+/// renombrado traen un segundo campo (la ruta de origen) que se salta.
+pub(crate) fn entradas_status(salida: &str) -> Vec<(String, String)> {
+    let mut v = Vec::new();
+    let mut campos = salida.split('\0').filter(|c| !c.is_empty());
+    while let Some(c) = campos.next() {
+        let xy = c.get(..2).unwrap_or(c).to_owned();
+        let ruta = c.get(3..).unwrap_or("").to_owned();
+        if xy.starts_with('R') || xy.starts_with('C') {
+            campos.next();
+        }
+        v.push((xy, ruta));
+    }
+    v
+}
+
+/// `TASK-NNN` con al menos tres cifras.
+pub(crate) fn es_id(id: &str) -> bool {
+    id.strip_prefix("TASK-")
+        .is_some_and(|n| n.len() >= 3 && n.bytes().all(|b| b.is_ascii_digit()))
+}
+
+/// Nombre de la rama de una tarea (ADR F1-3): `<tipo>/<task-nnn>-<slug>`.
+pub fn nombre_de_rama(id: &str, tipo: &str, slug: &str) -> String {
+    format!("{tipo}/{}-{slug}", id.to_lowercase())
+}
+
+/// Ruta relativa al principal, con `/`, para la linea de contrato.
+pub fn worktree_relativo(id: &str) -> String {
+    format!("{CARPETA_WORKTREES}/{id}")
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn status_z_con_renombrado_y_espacios() {
+        let s = " M a b.txt\0R  nuevo.txt\0viejo.txt\0?? d/\0";
+        let e = entradas_status(s);
+        assert_eq!(
+            e,
+            vec![
+                (" M".to_owned(), "a b.txt".to_owned()),
+                ("R ".to_owned(), "nuevo.txt".to_owned()),
+                ("??".to_owned(), "d/".to_owned()),
+            ]
+        );
+    }
+
+    #[test]
+    fn ids_ramas_y_bases() {
+        assert!(es_id("TASK-007") && es_id("TASK-1234"));
+        assert!(!es_id("TASK-7") && !es_id("task-007") && !es_id("TASK-00a"));
+        assert_eq!(
+            nombre_de_rama("TASK-007", "feature", "login"),
+            "feature/task-007-login"
+        );
+        assert_eq!(base_de_rama("hotfix/task-009-x"), "origin/main");
+        assert_eq!(base_de_rama("feature/task-007-login"), "origin/develop");
+    }
+}
diff --git a/crates/tc2/src/git/puerto.rs b/crates/tc2/src/git/puerto.rs
new file mode 100644
index 0000000..21543cd
--- /dev/null
+++ b/crates/tc2/src/git/puerto.rs
@@ -0,0 +1,295 @@
+//! Puerto `Git` (ADR F1-4, decision 1): lanza el binario `git` con un entorno
+//! fijo en cada llamada y solo con los subcomandos de la lista permitida.
+//!
+//! La lista permitida es un `enum`: `reset`, `rebase`, `stash` o `clean` no se
+//! pueden escribir (RN-05 por construccion, H9). Dentro de los permitidos, los
+//! argumentos prohibidos (`--force`, refspec `+`, borrar ramas o tags...) se
+//! rechazan antes de lanzar nada.
+
+use std::cell::RefCell;
+use std::ffi::OsString;
+use std::path::Path;
+use std::process::Command;
+
+/// Subcomandos de `git` que la capa puede lanzar (lista permitida de F1-4 B2b,
+/// reducida a los que usan las operaciones de F2-2).
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Subcomando {
+    /// `git add`
+    Add,
+    /// `git branch`
+    Branch,
+    /// `git commit`
+    Commit,
+    /// `git diff`
+    Diff,
+    /// `git fetch`
+    Fetch,
+    /// `git for-each-ref`
+    ForEachRef,
+    /// `git ls-remote`
+    LsRemote,
+    /// `git merge`
+    Merge,
+    /// `git merge-base`
+    MergeBase,
+    /// `git pull`
+    Pull,
+    /// `git push`
+    Push,
+    /// `git rev-parse`
+    RevParse,
+    /// `git show-ref`
+    ShowRef,
+    /// `git status`
+    Status,
+    /// `git tag`
+    Tag,
+    /// `git worktree`
+    Worktree,
+}
+
+impl Subcomando {
+    /// Nombre del subcomando en la linea de `git`.
+    pub fn nombre(self) -> &'static str {
+        match self {
+            Self::Add => "add",
+            Self::Branch => "branch",
+            Self::Commit => "commit",
+            Self::Diff => "diff",
+            Self::Fetch => "fetch",
+            Self::ForEachRef => "for-each-ref",
+            Self::LsRemote => "ls-remote",
+            Self::Merge => "merge",
+            Self::MergeBase => "merge-base",
+            Self::Pull => "pull",
+            Self::Push => "push",
+            Self::RevParse => "rev-parse",
+            Self::ShowRef => "show-ref",
+            Self::Status => "status",
+            Self::Tag => "tag",
+            Self::Worktree => "worktree",
+        }
+    }
+}
+
+/// Lo que devolvio una llamada a `git` que llego a ejecutarse.
+#[derive(Debug, Clone)]
+pub struct SalidaGit {
+    /// Codigo de salida (`-1` si el proceso murio por una senal).
+    pub codigo: i32,
+    /// Salida estandar, tal cual.
+    pub stdout: String,
+    /// Salida de error, tal cual.
+    pub stderr: String,
+}
+
+impl SalidaGit {
+    /// Si `git` termino con 0.
+    pub fn exito(&self) -> bool {
+        self.codigo == 0
+    }
+
+    /// Primera linea no vacia de stderr, para un diagnostico.
+    pub fn primera_linea_error(&self) -> String {
+        self.stderr
+            .lines()
+            .map(str::trim)
+            .find(|l| !l.is_empty())
+            .unwrap_or("sin mensaje")
+            .to_owned()
+    }
+}
+
+/// La llamada no llego a ejecutarse.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum ErrorGit {
+    /// Argumento prohibido dentro de un subcomando permitido.
+    Prohibido(String),
+    /// No se pudo lanzar el binario.
+    Lanzar(String),
+}
+
+impl std::fmt::Display for ErrorGit {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        match self {
+            Self::Prohibido(a) => write!(f, "argumento prohibido para la capa: {a}"),
+            Self::Lanzar(e) => write!(f, "no se pudo lanzar git: {e}"),
+        }
+    }
+}
+
+/// Variables que, heredadas, harian que `git` mirase otro repo u otro indice.
+const ENTORNO_QUITADO: [&str; 6] = [
+    "GIT_DIR",
+    "GIT_WORK_TREE",
+    "GIT_INDEX_FILE",
+    "GIT_COMMON_DIR",
+    "GIT_OBJECT_DIRECTORY",
+    "GIT_NAMESPACE",
+];
+
+/// Entorno fijo de cada llamada (F1-4, decision 1).
+const ENTORNO_FIJO: [(&str, &str); 5] = [
+    ("LC_ALL", "C"),
+    ("LANG", "C"),
+    ("GIT_TERMINAL_PROMPT", "0"),
+    ("GIT_PAGER", "cat"),
+    ("GIT_EDITOR", "true"),
+];
+
+/// Puerto `Git`: un binario `git` y un registro de cada llamada.
+#[derive(Debug)]
+pub struct Git {
+    binario: OsString,
+    entorno_extra: Vec<(OsString, OsString)>,
+    registro: RefCell<Vec<String>>,
+}
+
+impl Default for Git {
+    fn default() -> Self {
+        Self::nuevo()
+    }
+}
+
+impl Git {
+    /// El `git` del `PATH`, con el entorno del proceso mas el entorno fijo.
+    pub fn nuevo() -> Self {
+        Self {
+            binario: OsString::from("git"),
+            entorno_extra: Vec::new(),
+            registro: RefCell::new(Vec::new()),
+        }
+    }
+
+    /// Anade variables de entorno a cada llamada (los tests aislan asi la
+    /// configuracion del usuario sin tocar el entorno del proceso).
+    #[must_use]
+    pub fn con_entorno(mut self, vars: Vec<(OsString, OsString)>) -> Self {
+        self.entorno_extra = vars;
+        self
+    }
+
+    /// Copia del registro: una linea `git <subcomando> <args>` por llamada
+    /// ejecutada.
+    pub fn registro(&self) -> Vec<String> {
+        self.registro.borrow().clone()
+    }
+
+    /// Lanza `git <sub> <args>` en `dir`. Devuelve `Ok` aunque `git` termine
+    /// con error (lo interpreta la operacion); `Err` si no llega a lanzarse.
+    pub fn ejecutar(
+        &self,
+        dir: &Path,
+        sub: Subcomando,
+        args: &[&str],
+    ) -> Result<SalidaGit, ErrorGit> {
+        comprobar_argumentos(sub, args)?;
+        let mut cmd = Command::new(&self.binario);
+        cmd.current_dir(dir)
+            .args(["-c", "color.ui=never", "-c", "core.quotepath=false"])
+            .arg(sub.nombre())
+            .args(args);
+        for v in ENTORNO_QUITADO {
+            cmd.env_remove(v);
+        }
+        for (k, v) in ENTORNO_FIJO {
+            cmd.env(k, v);
+        }
+        for (k, v) in &self.entorno_extra {
+            cmd.env(k, v);
+        }
+        self.registro
+            .borrow_mut()
+            .push(format!("git {} {}", sub.nombre(), args.join(" ")));
+        let salida = cmd.output().map_err(|e| ErrorGit::Lanzar(e.to_string()))?;
+        Ok(SalidaGit {
+            codigo: salida.status.code().unwrap_or(-1),
+            stdout: String::from_utf8_lossy(&salida.stdout).into_owned(),
+            stderr: String::from_utf8_lossy(&salida.stderr).into_owned(),
+        })
+    }
+}
+
+/// Argumentos prohibidos dentro de los subcomandos permitidos (F1-4,
+/// decision 3). Se mira cada argumento entero, no subcadenas (H9).
+fn comprobar_argumentos(sub: Subcomando, args: &[&str]) -> Result<(), ErrorGit> {
+    let prohibido = |a: &str| Err(ErrorGit::Prohibido(format!("{} {a}", sub.nombre())));
+    for &a in args {
+        if a.starts_with("--force") {
+            return prohibido(a);
+        }
+        let malo = match sub {
+            Subcomando::Push => {
+                a == "-f"
+                    || a == "-d"
+                    || a == "--delete"
+                    || a == "--mirror"
+                    || a == "--prune"
+                    || a == "--all"
+                    || a.starts_with('+')
+                    || a.starts_with(':')
+            }
+            Subcomando::Branch => matches!(
+                a,
+                "-f" | "-d" | "-D" | "-m" | "-M" | "-c" | "-C" | "--delete" | "--move" | "--copy"
+            ),
+            Subcomando::Worktree => a == "-f",
+            Subcomando::Tag => matches!(a, "-d" | "-f" | "--delete"),
+            Subcomando::Merge => a == "--abort",
+            Subcomando::Commit => a == "--amend",
+            _ => false,
+        };
+        if malo {
+            return prohibido(a);
+        }
+    }
+    if sub == Subcomando::Pull && !args.contains(&"--ff-only") {
+        return prohibido("sin --ff-only");
+    }
+    Ok(())
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn rechaza_argumentos_prohibidos_sin_lanzar_git() {
+        // Binario inexistente: si llegara a lanzarse, el error seria `Lanzar`.
+        let git = Git {
+            binario: OsString::from("no-existe-este-git"),
+            entorno_extra: Vec::new(),
+            registro: RefCell::new(Vec::new()),
+        };
+        let casos: [(Subcomando, &[&str]); 12] = [
+            (Subcomando::Push, &["origin", "+r:r"]),
+            (Subcomando::Push, &["--force-with-lease", "origin", "r"]),
+            (Subcomando::Push, &["origin", ":r"]),
+            (Subcomando::Push, &["--mirror", "origin"]),
+            (Subcomando::Push, &["-f", "origin", "r"]),
+            (Subcomando::Branch, &["-D", "r"]),
+            (Subcomando::Branch, &["-f", "r", "HEAD"]),
+            (Subcomando::Worktree, &["remove", "--force", "w"]),
+            (Subcomando::Tag, &["-d", "v1"]),
+            (Subcomando::Merge, &["--abort"]),
+            (Subcomando::Pull, &["origin", "develop"]),
+            (Subcomando::Commit, &["--amend", "-m", "x"]),
+        ];
+        for (sub, args) in casos {
+            let r = git.ejecutar(Path::new("."), sub, args);
+            assert!(
+                matches!(r, Err(ErrorGit::Prohibido(_))),
+                "{sub:?} {args:?}: {r:?}"
+            );
+        }
+        assert!(git.registro().is_empty());
+        // Un argumento permitido si intenta lanzar (y falla por el binario).
+        let r = git.ejecutar(
+            Path::new("."),
+            Subcomando::Push,
+            &["--porcelain", "origin", "a:b"],
+        );
+        assert!(matches!(r, Err(ErrorGit::Lanzar(_))));
+    }
+}
diff --git a/crates/tc2/src/git/rama.rs b/crates/tc2/src/git/rama.rs
new file mode 100644
index 0000000..c8f6d99
--- /dev/null
+++ b/crates/tc2/src/git/rama.rs
@@ -0,0 +1,329 @@
+//! Operaciones de rama y worktree de tarea: `localizar-principal`,
+//! `validar-base`, `comprobar-id`, `crear-rama-y-worktree` y `reanudar`.
+
+use std::path::{Path, PathBuf};
+
+use super::{
+    CARPETA_WORKTREES, Git, Paso, REMOTO, Subcomando, commit_de, correr, es_id, exigir, misma_ruta,
+    nombre_de_rama, worktree_relativo, worktrees,
+};
+use crate::contrato::{Informe, Resultado, corto};
+
+/// Worktree principal: la primera entrada de `git worktree list` (ADR F1-3).
+/// Funciona lanzado desde el principal o desde cualquier worktree de tarea.
+pub fn localizar_principal(git: &Git, dir: &Path) -> Result<PathBuf, Informe> {
+    const OP: &str = "localizar-principal";
+    let s = correr(git, dir, OP, Subcomando::Worktree, &["list", "--porcelain"])?;
+    let primera = s
+        .stdout
+        .lines()
+        .next()
+        .and_then(|l| l.strip_prefix("worktree "))
+        .filter(|_| s.exito());
+    match primera {
+        Some(r) => Ok(PathBuf::from(r)),
+        None => Err(Informe::nuevo(OP, Resultado::Rechazada)
+            .con("motivo", "no-es-repo")
+            .diagnostico("lanza el comando dentro de un repositorio git")),
+    }
+}
+
+/// Comprueba que `base` existe y apunta a un commit.
+pub fn validar_base(git: &Git, dir: &Path, base: &str) -> Informe {
+    const OP: &str = "validar-base";
+    if base.is_empty() || base.starts_with('-') {
+        return Informe::nuevo(OP, Resultado::Uso).con("motivo", "base-invalida");
+    }
+    match commit_de(git, dir, OP, base) {
+        Ok(Some(c)) => Informe::nuevo(OP, Resultado::Ok)
+            .con("base", base)
+            .con("commit", corto(&c)),
+        Ok(None) => Informe::nuevo(OP, Resultado::Rechazada)
+            .con("base", base)
+            .con("motivo", "no-existe")
+            .diagnostico(format!(
+                "la base {base} no existe: lanza fetch o revisa el nombre"
+            )),
+        Err(i) => i,
+    }
+}
+
+/// Mira si el ID ya tiene rama, en local o en el remoto (patron
+/// `*/task-nnn-*`). Ocupado → `rechazada motivo=ocupada`.
+pub fn comprobar_id(git: &Git, dir: &Path, id: &str) -> Informe {
+    const OP: &str = "comprobar-id";
+    if !es_id(id) {
+        return Informe::nuevo(OP, Resultado::Uso).con("motivo", "id-invalido");
+    }
+    match ramas_del_id(git, dir, OP, id) {
+        Ok(r) if r.is_empty() => Informe::nuevo(OP, Resultado::Ok).con("id", id),
+        Ok(r) => Informe::nuevo(OP, Resultado::Rechazada)
+            .con("id", id)
+            .con("motivo", "ocupada")
+            .con_lista("ramas", r)
+            .diagnostico(format!("{id} ya tiene rama: elige otro ID")),
+        Err(i) => i,
+    }
+}
+
+fn ramas_del_id(git: &Git, dir: &Path, op: &str, id: &str) -> Paso<Vec<String>> {
+    let patron = format!("*/{}-*", id.to_lowercase());
+    let remoto = correr(
+        git,
+        dir,
+        op,
+        Subcomando::LsRemote,
+        &["--heads", REMOTO, &patron],
+    )?;
+    if !remoto.exito() {
+        return Err(Informe::nuevo(op, Resultado::Sinred)
+            .con("remoto", REMOTO)
+            .diagnostico(format!("{op}: no se pudo leer {REMOTO}; reintenta con red")));
+    }
+    let local = exigir(
+        git,
+        dir,
+        op,
+        Subcomando::Branch,
+        &["--list", "--format=%(refname:short)", &patron],
+    )?;
+    let mut ramas: Vec<String> = remoto
+        .stdout
+        .lines()
+        .filter_map(|l| l.split('\t').nth(1))
+        .filter_map(|r| r.strip_prefix("refs/heads/"))
+        .chain(local.lines().map(str::trim).filter(|l| !l.is_empty()))
+        .map(str::to_owned)
+        .collect();
+    ramas.sort();
+    ramas.dedup();
+    Ok(ramas)
+}
+
+/// Valida `id`, `tipo` y `slug` y devuelve el nombre de rama.
+fn rama_valida(op: &str, id: &str, tipo: &str, slug: &str) -> Paso<String> {
+    let tipo_ok = !tipo.is_empty() && tipo.bytes().all(|b| b.is_ascii_lowercase());
+    let slug_ok = !slug.is_empty()
+        && !slug.starts_with('-')
+        && slug
+            .bytes()
+            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
+    if !es_id(id) {
+        Err(Informe::nuevo(op, Resultado::Uso).con("motivo", "id-invalido"))
+    } else if !tipo_ok {
+        Err(Informe::nuevo(op, Resultado::Uso).con("motivo", "tipo-invalido"))
+    } else if !slug_ok {
+        Err(Informe::nuevo(op, Resultado::Uso).con("motivo", "slug-invalido"))
+    } else {
+        Ok(nombre_de_rama(id, tipo, slug))
+    }
+}
+
+fn ruta_texto(op: &str, p: &Path) -> Paso<String> {
+    p.to_str().map(str::to_owned).ok_or_else(|| {
+        Informe::nuevo(op, Resultado::Error).diagnostico("la ruta del worktree no es UTF-8")
+    })
+}
+
+/// Que `git status` del principal no muestre los worktrees de tarea:
+/// `/.taskcode/worktrees/` en `info/exclude` del repo comun (no versionado).
+fn excluir_worktrees(git: &Git, principal: &Path, op: &str) -> Paso<()> {
+    let comun = exigir(
+        git,
+        principal,
+        op,
+        Subcomando::RevParse,
+        &["--git-common-dir"],
+    )?;
+    let comun = principal.join(comun.trim());
+    let info = comun.join("info");
+    let fichero = info.join("exclude");
+    let linea = format!("/{CARPETA_WORKTREES}/");
+    let error = |e: std::io::Error| {
+        Informe::nuevo(op, Resultado::Error).diagnostico(format!("{}: {e}", fichero.display()))
+    };
+    // Solo un fichero ausente cuenta como vacio: si no se puede leer, no se
+    // reescribe (se perderia lo que la persona tenga en el).
+    let actual = match std::fs::read_to_string(&fichero) {
+        Ok(t) => t,
+        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
+        Err(e) => return Err(error(e)),
+    };
+    if actual.lines().any(|l| l.trim() == linea) {
+        return Ok(());
+    }
+    std::fs::create_dir_all(&info).map_err(error)?;
+    let mut nuevo = actual;
+    if !nuevo.is_empty() && !nuevo.ends_with('\n') {
+        nuevo.push('\n');
+    }
+    nuevo.push_str(&linea);
+    nuevo.push('\n');
+    std::fs::write(&fichero, nuevo).map_err(error)
+}
+
+/// Rama y worktree nuevos para una tarea (caso b de F1-3). Repetirlo da
+/// `sin-cambios`; si la rama ya existe sin su worktree, la recupera
+/// [`reanudar`].
+pub fn crear_rama_y_worktree(
+    git: &Git,
+    dir: &Path,
+    id: &str,
+    tipo: &str,
+    slug: &str,
+    base: &str,
+) -> Informe {
+    crear(git, dir, id, tipo, slug, base).unwrap_or_else(|i| i)
+}
+
+fn crear(git: &Git, dir: &Path, id: &str, tipo: &str, slug: &str, base: &str) -> Paso<Informe> {
+    const OP: &str = "crear-rama-y-worktree";
+    let rama = rama_valida(OP, id, tipo, slug)?;
+    if base.is_empty() || base.starts_with('-') {
+        return Err(Informe::nuevo(OP, Resultado::Uso).con("motivo", "base-invalida"));
+    }
+    let principal = localizar_principal(git, dir).map_err(|i| renombrar(i, OP))?;
+    let wt = principal.join(CARPETA_WORKTREES).join(id);
+    let rel = worktree_relativo(id);
+    let rechazo = |motivo: &str| {
+        Informe::nuevo(OP, Resultado::Rechazada)
+            .con("rama", rama.as_str())
+            .con("motivo", motivo)
+    };
+    if let Some(w) = worktrees(git, &principal, OP)?
+        .into_iter()
+        .find(|w| w.rama.as_deref() == Some(rama.as_str()))
+    {
+        return Ok(if misma_ruta(&w.ruta, &wt) {
+            Informe::nuevo(OP, Resultado::SinCambios)
+                .con("rama", rama.as_str())
+                .con("worktree", rel)
+        } else {
+            rechazo("rama-en-otro-worktree").diagnostico("la rama ya esta abierta en otro worktree")
+        });
+    }
+    if commit_de(git, &principal, OP, &format!("refs/heads/{rama}"))?.is_some() {
+        return Ok(rechazo("rama-existe").diagnostico("la rama ya existe: la recupera reanudar"));
+    }
+    if commit_de(
+        git,
+        &principal,
+        OP,
+        &format!("refs/remotes/{REMOTO}/{rama}"),
+    )?
+    .is_some()
+    {
+        return Ok(rechazo("rama-en-remoto")
+            .diagnostico("la rama ya esta publicada: la recupera reanudar"));
+    }
+    if wt.exists() {
+        return Ok(rechazo("carpeta-ocupada")
+            .diagnostico(format!("{rel} ya existe y no es el worktree de la tarea")));
+    }
+    let Some(commit) = commit_de(git, &principal, OP, base)? else {
+        return Ok(Informe::nuevo(OP, Resultado::Rechazada)
+            .con("base", base)
+            .con("motivo", "base-no-existe")
+            .diagnostico(format!(
+                "la base {base} no existe: lanza fetch o revisa el nombre"
+            )));
+    };
+    excluir_worktrees(git, &principal, OP)?;
+    let wt_txt = ruta_texto(OP, &wt)?;
+    exigir(
+        git,
+        &principal,
+        OP,
+        Subcomando::Worktree,
+        &["add", "-q", "--no-track", "-b", &rama, &wt_txt, &commit],
+    )?;
+    Ok(Informe::nuevo(OP, Resultado::Ok)
+        .con("rama", rama.as_str())
+        .con("worktree", rel)
+        .con("base", corto(&commit)))
+}
+
+/// Recupera la tarea tras perder el worktree o la rama local (caso c de
+/// F1-3): crea solo lo que falte. Nada que crear → `sin-cambios`.
+pub fn reanudar(git: &Git, dir: &Path, id: &str, tipo: &str, slug: &str) -> Informe {
+    reanudar_paso(git, dir, id, tipo, slug).unwrap_or_else(|i| i)
+}
+
+fn reanudar_paso(git: &Git, dir: &Path, id: &str, tipo: &str, slug: &str) -> Paso<Informe> {
+    const OP: &str = "reanudar";
+    let rama = rama_valida(OP, id, tipo, slug)?;
+    let principal = localizar_principal(git, dir).map_err(|i| renombrar(i, OP))?;
+    let wt = principal.join(CARPETA_WORKTREES).join(id);
+    let rel = worktree_relativo(id);
+    let rechazo = |motivo: &str| {
+        Informe::nuevo(OP, Resultado::Rechazada)
+            .con("rama", rama.as_str())
+            .con("motivo", motivo)
+    };
+    // Un worktree borrado a mano sigue registrado hasta que se poda.
+    exigir(git, &principal, OP, Subcomando::Worktree, &["prune"])?;
+    let mut creado = Vec::new();
+    if commit_de(git, &principal, OP, &format!("refs/heads/{rama}"))?.is_none() {
+        let remota = format!("refs/remotes/{REMOTO}/{rama}");
+        if commit_de(git, &principal, OP, &remota)?.is_none() {
+            return Ok(rechazo("rama-no-existe")
+                .diagnostico("la tarea no tiene rama ni en local ni en el remoto"));
+        }
+        exigir(
+            git,
+            &principal,
+            OP,
+            Subcomando::Branch,
+            &["-q", "--no-track", &rama, &remota],
+        )?;
+        creado.push("rama".to_owned());
+    }
+    match worktrees(git, &principal, OP)?
+        .into_iter()
+        .find(|w| w.rama.as_deref() == Some(rama.as_str()))
+    {
+        Some(w) if misma_ruta(&w.ruta, &wt) => {}
+        Some(_) => {
+            return Ok(rechazo("rama-en-otro-worktree")
+                .diagnostico("la rama ya esta abierta en otro worktree"));
+        }
+        None => {
+            if wt.exists() {
+                return Ok(rechazo("carpeta-ocupada")
+                    .diagnostico(format!("{rel} ya existe y no es el worktree de la tarea")));
+            }
+            excluir_worktrees(git, &principal, OP)?;
+            let wt_txt = ruta_texto(OP, &wt)?;
+            exigir(
+                git,
+                &principal,
+                OP,
+                Subcomando::Worktree,
+                &["add", "-q", &wt_txt, &rama],
+            )?;
+            creado.push("worktree".to_owned());
+        }
+    }
+    let i = Informe::nuevo(
+        OP,
+        if creado.is_empty() {
+            Resultado::SinCambios
+        } else {
+            Resultado::Ok
+        },
+    )
+    .con("rama", rama.as_str())
+    .con("worktree", rel);
+    Ok(if creado.is_empty() {
+        i
+    } else {
+        i.con_lista("creado", creado)
+    })
+}
+
+/// Un fallo de `localizar-principal` se informa con el nombre de la operacion
+/// que lo pidio.
+pub(crate) fn renombrar(mut i: Informe, op: &str) -> Informe {
+    op.clone_into(&mut i.comando);
+    i
+}
diff --git a/crates/tc2/src/git/remoto.rs b/crates/tc2/src/git/remoto.rs
new file mode 100644
index 0000000..8987859
--- /dev/null
+++ b/crates/tc2/src/git/remoto.rs
@@ -0,0 +1,335 @@
+//! Operaciones con el remoto: `fetch`, `push` con reintento,
+//! `actualizar-rama` y `actualizar-principal`.
+
+use std::path::Path;
+use std::time::Duration;
+
+use super::rama::renombrar;
+use super::{
+    Git, Paso, REMOTO, Subcomando, base_de_rama, commit_de, correr, entradas_status, es_ancestro,
+    es_protegida, exigir, localizar_principal, rama_actual,
+};
+use crate::contrato::{Informe, Resultado, corto};
+
+/// Esperas entre intentos de `push`: tres intentos, a 1 s y 2 s.
+pub const ESPERAS_PUSH: [Duration; 2] = [Duration::from_secs(1), Duration::from_secs(2)];
+
+/// Huella de las ramas remotas, para saber si el `fetch` trajo algo.
+fn refs_remotas(git: &Git, dir: &Path, op: &str) -> Paso<String> {
+    exigir(
+        git,
+        dir,
+        op,
+        Subcomando::ForEachRef,
+        &[
+            "--format=%(refname) %(objectname)",
+            &format!("refs/remotes/{REMOTO}"),
+        ],
+    )
+}
+
+fn sinred(op: &str) -> Informe {
+    Informe::nuevo(op, Resultado::Sinred)
+        .con("remoto", REMOTO)
+        .diagnostico(format!("{op}: no se pudo leer {REMOTO}; reintenta con red"))
+}
+
+/// Trae el remoto. Nada nuevo → `sin-cambios`; sin remoto → `sinred`.
+pub fn fetch(git: &Git, dir: &Path) -> Informe {
+    traer(git, dir, "fetch").unwrap_or_else(|i| i)
+}
+
+fn traer(git: &Git, dir: &Path, op: &str) -> Paso<Informe> {
+    let antes = refs_remotas(git, dir, op)?;
+    let s = correr(git, dir, op, Subcomando::Fetch, &["-q", REMOTO])?;
+    if !s.exito() {
+        return Err(sinred(op));
+    }
+    let despues = refs_remotas(git, dir, op)?;
+    let r = if antes == despues {
+        Resultado::SinCambios
+    } else {
+        Resultado::Ok
+    };
+    Ok(Informe::nuevo(op, r).con("remoto", REMOTO))
+}
+
+/// Como termino un intento de push, leido de `git push --porcelain`.
+enum Intento {
+    Publicado,
+    AlDia,
+    NoFastForward,
+    Fallo(String),
+}
+
+fn intento_push(git: &Git, dir: &Path, op: &str, refspec: &str) -> Paso<Intento> {
+    let s = correr(
+        git,
+        dir,
+        op,
+        Subcomando::Push,
+        &["--porcelain", REMOTO, refspec],
+    )?;
+    // Linea de porcelana: `<flag>\t<origen>:<destino>\t<resumen>`.
+    let linea = s
+        .stdout
+        .lines()
+        .find(|l| l.contains('\t') && l.split('\t').nth(1).is_some_and(|r| r.contains(':')));
+    let flag = linea.and_then(|l| l.chars().next());
+    let resumen = linea.and_then(|l| l.split('\t').nth(2)).unwrap_or("");
+    Ok(match flag {
+        Some('=') if s.exito() => Intento::AlDia,
+        Some(' ' | '*') if s.exito() => Intento::Publicado,
+        Some('!') if resumen.starts_with("[rejected]") => Intento::NoFastForward,
+        _ => Intento::Fallo(if resumen.is_empty() {
+            s.primera_linea_error()
+        } else {
+            resumen.to_owned()
+        }),
+    })
+}
+
+/// Publica `refspec` con reintentos ante fallos transitorios. Un
+/// non-fast-forward no se reintenta. Devuelve el resultado y los intentos.
+pub(crate) fn publicar(
+    git: &Git,
+    dir: &Path,
+    op: &str,
+    refspec: &str,
+    esperas: &[Duration],
+) -> Paso<(Resultado, usize, String)> {
+    let mut ultimo = String::new();
+    for intento in 0..=esperas.len() {
+        match intento_push(git, dir, op, refspec)? {
+            Intento::AlDia => return Ok((Resultado::SinCambios, intento + 1, String::new())),
+            Intento::Publicado => return Ok((Resultado::Ok, intento + 1, String::new())),
+            Intento::NoFastForward => {
+                return Ok((Resultado::Rechazada, intento + 1, String::new()));
+            }
+            Intento::Fallo(m) => ultimo = m,
+        }
+        if let Some(e) = esperas.get(intento) {
+            std::thread::sleep(*e);
+        }
+    }
+    Ok((Resultado::Sinred, esperas.len() + 1, ultimo))
+}
+
+/// Publica la rama actual en `origin` con el mismo nombre, sin force.
+/// `esperas` fija los reintentos (en produccion, [`ESPERAS_PUSH`]).
+pub fn push(git: &Git, dir: &Path, esperas: &[Duration]) -> Informe {
+    push_paso(git, dir, "push", esperas).unwrap_or_else(|i| i)
+}
+
+fn push_paso(git: &Git, dir: &Path, op: &str, esperas: &[Duration]) -> Paso<Informe> {
+    let rama = rama_de_tarea(git, dir, op)?;
+    let head = commit_de(git, dir, op, "HEAD")?.unwrap_or_default();
+    let refspec = format!("refs/heads/{rama}:refs/heads/{rama}");
+    let (r, intentos, ultimo) = publicar(git, dir, op, &refspec, esperas)?;
+    let i = Informe::nuevo(op, r).con("rama", rama.as_str());
+    Ok(match r {
+        Resultado::Ok => i
+            .con("commit", corto(&head))
+            .con("intentos", intentos.to_string()),
+        Resultado::SinCambios => i.con("commit", corto(&head)),
+        Resultado::Rechazada => i
+            .con("motivo", "no-fast-forward")
+            .diagnostico("lanza actualizar-rama"),
+        _ => i
+            .con("intentos", intentos.to_string())
+            .diagnostico(format!("push: {ultimo}; reintenta con red")),
+    })
+}
+
+/// La rama actual, si es de tarea (ni HEAD suelto ni `develop`/`main`).
+fn rama_de_tarea(git: &Git, dir: &Path, op: &str) -> Paso<String> {
+    match rama_actual(git, dir, op)? {
+        None => Err(Informe::nuevo(op, Resultado::Rechazada)
+            .con("motivo", "sin-rama")
+            .diagnostico("lanzalo en el worktree de la tarea")),
+        Some(r) if es_protegida(&r) => Err(Informe::nuevo(op, Resultado::Rechazada)
+            .con("rama", r.as_str())
+            .con("motivo", "rama-protegida")
+            .diagnostico("lanzalo en el worktree de la tarea, no en develop ni en main")),
+        Some(r) => Ok(r),
+    }
+}
+
+/// Ficheros con conflicto sin resolver.
+pub(crate) fn sin_resolver(git: &Git, dir: &Path, op: &str) -> Paso<Vec<String>> {
+    let s = exigir(
+        git,
+        dir,
+        op,
+        Subcomando::Diff,
+        &["--name-only", "--diff-filter=U", "-z"],
+    )?;
+    Ok(s.split('\0')
+        .filter(|f| !f.is_empty())
+        .map(str::to_owned)
+        .collect())
+}
+
+/// Si hay un merge abierto (`MERGE_HEAD`).
+pub(crate) fn merge_abierto(git: &Git, dir: &Path, op: &str) -> Paso<bool> {
+    Ok(commit_de(git, dir, op, "MERGE_HEAD")?.is_some())
+}
+
+/// Lleva a la rama de la tarea lo nuevo de su base (y lo que otra persona
+/// haya publicado en la misma rama) con merge, **sin rebase**, y publica.
+/// Conflicto → `conflicto` (6) con los ficheros; repetirlo no vuelve a
+/// mergear. Se resuelve y se lanza `guardar`.
+pub fn actualizar_rama(git: &Git, dir: &Path, esperas: &[Duration]) -> Informe {
+    actualizar(git, dir, esperas).unwrap_or_else(|i| i)
+}
+
+fn actualizar(git: &Git, dir: &Path, esperas: &[Duration]) -> Paso<Informe> {
+    const OP: &str = "actualizar-rama";
+    let rama = rama_de_tarea(git, dir, OP)?;
+    let base = base_de_rama(&rama);
+    let conflicto = |ficheros: Vec<String>| {
+        let diag = if ficheros.is_empty() {
+            "lanza guardar con los ficheros resueltos".to_owned()
+        } else {
+            format!("resuelve {} y lanza guardar", ficheros.join(" "))
+        };
+        Informe::nuevo(OP, Resultado::Conflicto)
+            .con("base", base)
+            .con_lista("ficheros", ficheros)
+            .diagnostico(diag)
+    };
+    if merge_abierto(git, dir, OP)? {
+        return Ok(conflicto(sin_resolver(git, dir, OP)?));
+    }
+    let pendientes = exigir(
+        git,
+        dir,
+        OP,
+        Subcomando::Status,
+        &["--porcelain", "-z", "--untracked-files=no"],
+    )?;
+    if !entradas_status(&pendientes).is_empty() {
+        return Ok(Informe::nuevo(OP, Resultado::Rechazada)
+            .con("motivo", "cambios-sin-guardar")
+            .diagnostico("guarda los cambios con guardar antes de actualizar la rama"));
+    }
+    traer(git, dir, OP)?;
+    if commit_de(git, dir, OP, base)?.is_none() {
+        return Ok(Informe::nuevo(OP, Resultado::Rechazada)
+            .con("base", base)
+            .con("motivo", "base-no-existe")
+            .diagnostico(format!("{base} no existe en el remoto")));
+    }
+    let remota = format!("{REMOTO}/{rama}");
+    let hay_remota = commit_de(git, dir, OP, &remota)?.is_some();
+    let mut mergeado = false;
+    let fuentes: Vec<&str> = if hay_remota {
+        vec![remota.as_str(), base]
+    } else {
+        vec![base]
+    };
+    for fuente in fuentes {
+        if es_ancestro(git, dir, OP, fuente, "HEAD")? {
+            continue;
+        }
+        let s = correr(
+            git,
+            dir,
+            OP,
+            Subcomando::Merge,
+            &["-q", "--no-edit", "--no-ff", fuente],
+        )?;
+        if !s.exito() {
+            if merge_abierto(git, dir, OP)? {
+                return Ok(conflicto(sin_resolver(git, dir, OP)?));
+            }
+            return Ok(Informe::nuevo(OP, Resultado::Error)
+                .con("base", base)
+                .diagnostico(format!("git merge fallo: {}", s.primera_linea_error())));
+        }
+        mergeado = true;
+    }
+    // Publica si la rama ya estaba publicada y va por delante (merge recien
+    // hecho, o un conflicto resuelto con guardar).
+    let mut publicada = false;
+    if hay_remota && !es_ancestro(git, dir, OP, "HEAD", &remota)? {
+        let refspec = format!("refs/heads/{rama}:refs/heads/{rama}");
+        let (r, intentos, ultimo) = publicar(git, dir, OP, &refspec, esperas)?;
+        match r {
+            Resultado::Ok | Resultado::SinCambios => publicada = true,
+            Resultado::Rechazada => {
+                return Ok(Informe::nuevo(OP, Resultado::Rechazada)
+                    .con("rama", rama.as_str())
+                    .con("motivo", "no-fast-forward")
+                    .diagnostico(
+                        "alguien publico en la rama entre medias: vuelve a lanzar actualizar-rama",
+                    ));
+            }
+            _ => {
+                return Ok(Informe::nuevo(OP, Resultado::Sinred)
+                    .con("rama", rama.as_str())
+                    .con("intentos", intentos.to_string())
+                    .diagnostico(format!(
+                        "push: {ultimo}; la rama esta actualizada en local, reintenta con red"
+                    )));
+            }
+        }
+    }
+    let head = commit_de(git, dir, OP, "HEAD")?.unwrap_or_default();
+    if !mergeado && !publicada {
+        return Ok(Informe::nuevo(OP, Resultado::SinCambios).con("base", base));
+    }
+    Ok(Informe::nuevo(OP, Resultado::Ok)
+        .con("base", base)
+        .con("commit", corto(&head))
+        .con("publicada", if publicada { "si" } else { "no" }))
+}
+
+/// Adelanta la rama del worktree principal (`develop` o `main`) con
+/// `pull --ff-only`. Al dia → `sin-cambios`; divergente → `rechazada`.
+pub fn actualizar_principal(git: &Git, dir: &Path, rama: &str) -> Informe {
+    actualizar_principal_paso(git, dir, rama).unwrap_or_else(|i| i)
+}
+
+fn actualizar_principal_paso(git: &Git, dir: &Path, rama: &str) -> Paso<Informe> {
+    const OP: &str = "actualizar-principal";
+    if !es_protegida(rama) {
+        return Err(Informe::nuevo(OP, Resultado::Uso).con("motivo", "rama-invalida"));
+    }
+    let principal = localizar_principal(git, dir).map_err(|i| renombrar(i, OP))?;
+    let actual = rama_actual(git, &principal, OP)?;
+    if actual.as_deref() != Some(rama) {
+        return Ok(Informe::nuevo(OP, Resultado::Rechazada)
+            .con("rama", rama)
+            .con("motivo", "principal-en-otra-rama")
+            .diagnostico(format!("el worktree principal no esta en {rama}")));
+    }
+    traer(git, &principal, OP)?;
+    let remota = format!("{REMOTO}/{rama}");
+    let Some(r) = commit_de(git, &principal, OP, &remota)? else {
+        return Ok(Informe::nuevo(OP, Resultado::Rechazada)
+            .con("rama", rama)
+            .con("motivo", "sin-rama-remota"));
+    };
+    if es_ancestro(git, &principal, OP, &r, "HEAD")? {
+        return Ok(Informe::nuevo(OP, Resultado::SinCambios).con("rama", rama));
+    }
+    if !es_ancestro(git, &principal, OP, "HEAD", &r)? {
+        return Ok(Informe::nuevo(OP, Resultado::Rechazada)
+            .con("rama", rama)
+            .con("motivo", "divergente")
+            .diagnostico(format!(
+                "{rama} local tiene commits que no estan en {remota}"
+            )));
+    }
+    exigir(
+        git,
+        &principal,
+        OP,
+        Subcomando::Pull,
+        &["-q", "--ff-only", REMOTO, rama],
+    )?;
+    Ok(Informe::nuevo(OP, Resultado::Ok)
+        .con("rama", rama)
+        .con("commit", corto(&r)))
+}
diff --git a/crates/tc2/src/git/tag.rs b/crates/tc2/src/git/tag.rs
new file mode 100644
index 0000000..de5df46
--- /dev/null
+++ b/crates/tc2/src/git/tag.rs
@@ -0,0 +1,130 @@
+//! `tag-anotado`: tag anotado sobre un commit y publicado, idempotente.
+
+use std::path::Path;
+use std::time::Duration;
+
+use super::remoto::publicar;
+use super::{Git, Paso, REMOTO, Subcomando, commit_de, correr, exigir};
+use crate::contrato::{Informe, Resultado, corto};
+
+/// Pone `tag` sobre `referencia` y lo publica. Compara con el tag local
+/// (aunque no este publicado) y con el del remoto: en otro commit →
+/// `rechazada` sin publicar nada; en el mismo y publicado → `sin-cambios`.
+pub fn tag_anotado(
+    git: &Git,
+    dir: &Path,
+    tag: &str,
+    referencia: &str,
+    esperas: &[Duration],
+) -> Informe {
+    tag_paso(git, dir, tag, referencia, esperas).unwrap_or_else(|i| i)
+}
+
+fn tag_paso(
+    git: &Git,
+    dir: &Path,
+    tag: &str,
+    referencia: &str,
+    esperas: &[Duration],
+) -> Paso<Informe> {
+    const OP: &str = "tag-anotado";
+    let valido = |t: &str| {
+        !t.is_empty()
+            && !t.starts_with('-')
+            && !t.chars().any(|c| c.is_whitespace() || c.is_control())
+    };
+    if !valido(tag) || !valido(referencia) {
+        return Err(Informe::nuevo(OP, Resultado::Uso).con("motivo", "argumento-invalido"));
+    }
+    let Some(commit) = commit_de(git, dir, OP, referencia)? else {
+        return Ok(Informe::nuevo(OP, Resultado::Rechazada)
+            .con("tag", tag)
+            .con("motivo", "ref-no-existe")
+            .diagnostico(format!("{referencia} no existe")));
+    };
+    let otro_commit = |donde: &str| {
+        Informe::nuevo(OP, Resultado::Rechazada)
+            .con("tag", tag)
+            .con("motivo", "existe-en-otro-commit")
+            .con("donde", donde)
+            .diagnostico(format!(
+                "el tag {tag} ya existe sobre otro commit: elige otro nombre"
+            ))
+    };
+    let local = commit_de(git, dir, OP, &format!("refs/tags/{tag}"))?;
+    if local.as_ref().is_some_and(|l| *l != commit) {
+        return Ok(otro_commit("local"));
+    }
+    let remoto = commit_remoto(git, dir, OP, tag)?;
+    if remoto.as_ref().is_some_and(|r| *r != commit) {
+        return Ok(otro_commit("remoto"));
+    }
+    if local.is_some() && remoto.is_some() {
+        return Ok(Informe::nuevo(OP, Resultado::SinCambios)
+            .con("tag", tag)
+            .con("commit", corto(&commit)));
+    }
+    if local.is_none() {
+        exigir(
+            git,
+            dir,
+            OP,
+            Subcomando::Tag,
+            &["-a", "-m", tag, "--", tag, &commit],
+        )?;
+    }
+    if remoto.is_none() {
+        let refspec = format!("refs/tags/{tag}:refs/tags/{tag}");
+        let (r, intentos, ultimo) = publicar(git, dir, OP, &refspec, esperas)?;
+        if !r.es_exito() {
+            return Ok(Informe::nuevo(
+                OP,
+                if r == Resultado::Rechazada {
+                    Resultado::Rechazada
+                } else {
+                    Resultado::Sinred
+                },
+            )
+            .con("tag", tag)
+            .con("intentos", intentos.to_string())
+            .diagnostico(format!(
+                "no se pudo publicar el tag: {ultimo}; el tag queda en local, relanza con red"
+            )));
+        }
+    }
+    Ok(Informe::nuevo(OP, Resultado::Ok)
+        .con("tag", tag)
+        .con("commit", corto(&commit)))
+}
+
+/// Commit del tag en el remoto (anotado: su `^{}`; ligero: el propio).
+fn commit_remoto(git: &Git, dir: &Path, op: &str, tag: &str) -> Paso<Option<String>> {
+    let r = format!("refs/tags/{tag}");
+    let pelado = format!("{r}^{{}}");
+    let s = correr(
+        git,
+        dir,
+        op,
+        Subcomando::LsRemote,
+        &["--tags", REMOTO, &r, &pelado],
+    )?;
+    if !s.exito() {
+        return Err(Informe::nuevo(op, Resultado::Sinred)
+            .con("remoto", REMOTO)
+            .diagnostico(format!("{op}: no se pudo leer {REMOTO}; reintenta con red")));
+    }
+    let mut directo = None;
+    for l in s.stdout.lines() {
+        let mut c = l.split('\t');
+        let (Some(sha), Some(nombre)) = (c.next(), c.next()) else {
+            continue;
+        };
+        if nombre == pelado {
+            return Ok(Some(sha.to_owned()));
+        }
+        if nombre == r {
+            directo = Some(sha.to_owned());
+        }
+    }
+    Ok(directo)
+}
diff --git a/crates/tc2/src/git/worktree.rs b/crates/tc2/src/git/worktree.rs
new file mode 100644
index 0000000..4f7d8b8
--- /dev/null
+++ b/crates/tc2/src/git/worktree.rs
@@ -0,0 +1,69 @@
+//! `quitar-worktree`: quita el worktree de una tarea cerrada; la rama se
+//! conserva (ADR F1-3: las ramas no se borran).
+
+use std::path::Path;
+
+use super::rama::renombrar;
+use super::{
+    CARPETA_WORKTREES, Git, Paso, REMOTO, Subcomando, commit_de, es_ancestro, es_id, exigir,
+    localizar_principal, misma_ruta, worktree_relativo, worktrees,
+};
+use crate::contrato::{Informe, Resultado};
+
+/// Quita `.taskcode/worktrees/<id>`. No existe → `sin-cambios`. Con cambios
+/// sin guardar o commits sin publicar → `rechazada`, sin tocar nada.
+pub fn quitar_worktree(git: &Git, dir: &Path, id: &str) -> Informe {
+    quitar(git, dir, id).unwrap_or_else(|i| i)
+}
+
+fn quitar(git: &Git, dir: &Path, id: &str) -> Paso<Informe> {
+    const OP: &str = "quitar-worktree";
+    if !es_id(id) {
+        return Err(Informe::nuevo(OP, Resultado::Uso).con("motivo", "id-invalido"));
+    }
+    let principal = localizar_principal(git, dir).map_err(|i| renombrar(i, OP))?;
+    let wt = principal.join(CARPETA_WORKTREES).join(id);
+    let Some(w) = worktrees(git, &principal, OP)?
+        .into_iter()
+        .find(|w| misma_ruta(&w.ruta, &wt))
+    else {
+        return Ok(Informe::nuevo(OP, Resultado::SinCambios).con("id", id));
+    };
+    let rechazo = |motivo: &str, que_hacer: &str| {
+        Informe::nuevo(OP, Resultado::Rechazada)
+            .con("id", id)
+            .con("motivo", motivo)
+            .diagnostico(que_hacer.to_owned())
+    };
+    let status = exigir(git, &w.ruta, OP, Subcomando::Status, &["--porcelain", "-z"])?;
+    if !status.is_empty() {
+        return Ok(rechazo(
+            "cambios-sin-guardar",
+            "guarda o descarta los cambios del worktree antes de quitarlo",
+        ));
+    }
+    if let Some(rama) = &w.rama {
+        let publicada = match commit_de(
+            git,
+            &principal,
+            OP,
+            &format!("refs/remotes/{REMOTO}/{rama}"),
+        )? {
+            Some(remota) => es_ancestro(git, &w.ruta, OP, "HEAD", &remota)?,
+            None => false,
+        };
+        if !publicada {
+            return Ok(rechazo(
+                "sin-publicar",
+                "publica la rama antes de quitar el worktree",
+            ));
+        }
+    }
+    let rel = worktree_relativo(id);
+    exigir(git, &principal, OP, Subcomando::Worktree, &["remove", &rel])?;
+    let i = Informe::nuevo(OP, Resultado::Ok).con("id", id);
+    Ok(match w.rama {
+        Some(r) => i.con("rama", r),
+        None => i,
+    })
+}
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 69e8f3a..6fa118d 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -1,8 +1,13 @@
 //! Nucleo del CLI `tc2` de TaskCodeTwo.
 //!
-//! El esqueleto (F2-1) solo sabe decir su version. Las capas que vienen
-//! (operaciones Git de F2-2, modelo de tarea de F2-5...) entran como modulos
-//! de esta biblioteca.
+//! Modulos: [`contrato`] (salida de toda operacion, ADR F1-4) y [`git`]
+//! (capa de operaciones Git deterministas, F2-2). Las capas que vienen
+//! (modelo de tarea de F2-5...) entran como modulos de esta biblioteca.
+
+pub mod contrato;
+pub mod git;
+
+use contrato::{Informe, Resultado};
 
 /// Version del CLI, la de `Cargo.toml`.
 pub const VERSION: &str = env!("CARGO_PKG_VERSION");
@@ -19,7 +24,11 @@ pub struct Salida {
     pub stderr: String,
 }
 
-/// Interpreta los argumentos (sin el nombre del programa).
+/// Uso del binario, para los mensajes de error.
+const USO: &str = "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json]";
+
+/// Interpreta los argumentos (sin el nombre del programa) y ejecuta la orden
+/// en la carpeta actual.
 pub fn ejecutar<I, S>(args: I) -> Salida
 where
     I: IntoIterator<Item = S>,
@@ -29,22 +38,65 @@ where
     match args.as_slice() {
         [a] if a == "--version" || a == "-V" => Salida {
             codigo: 0,
-            stdout: format!("tc2 {VERSION}\n"),
+            stdout: format!(
+                "tc2 {VERSION}
+"
+            ),
             stderr: String::new(),
         },
+        [orden, resto @ ..] if orden == "guardar" || orden == "actualizar-rama" => {
+            let (json, resto) = match resto {
+                [j, r @ ..] if j == "--json" => (true, r),
+                r => (false, r),
+            };
+            orden_git(orden, resto).salida(json)
+        }
         [] => Salida {
             codigo: 2,
             stdout: String::new(),
-            stderr: "tc2: falta la orden. Uso: tc2 --version\n".to_owned(),
+            stderr: format!(
+                "tc2: falta la orden. {USO}
+"
+            ),
         },
         [otro, ..] => Salida {
             codigo: 2,
             stdout: String::new(),
-            stderr: format!("tc2: orden desconocida «{otro}». Uso: tc2 --version\n"),
+            stderr: format!(
+                "tc2: orden desconocida «{otro}». {USO}
+"
+            ),
         },
     }
 }
 
+/// Las dos operaciones Git publicas (ADR F1-4 D1a), en la carpeta actual.
+fn orden_git(orden: &str, args: &[String]) -> Informe {
+    let dir = match std::env::current_dir() {
+        Ok(d) => d,
+        Err(e) => {
+            return Informe::nuevo(orden, Resultado::Error)
+                .diagnostico(format!("no se puede leer la carpeta actual: {e}"));
+        }
+    };
+    let g = git::Git::nuevo();
+    if orden == "guardar" {
+        let Some((mensaje, rutas)) = args.split_first() else {
+            return Informe::nuevo(orden, Resultado::Uso)
+                .con("motivo", "sin-mensaje")
+                .diagnostico("uso: guardar \"<mensaje>\" <rutas...>");
+        };
+        let rutas: Vec<&str> = rutas.iter().map(String::as_str).collect();
+        git::guardar(&g, &dir, mensaje, &rutas)
+    } else if args.is_empty() {
+        git::actualizar_rama(&g, &dir, &git::ESPERAS_PUSH)
+    } else {
+        Informe::nuevo(orden, Resultado::Uso)
+            .con("motivo", "argumentos-de-mas")
+            .diagnostico("uso: actualizar-rama [--json]")
+    }
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
@@ -62,4 +114,29 @@ mod tests {
     fn sin_orden_falla() {
         assert_eq!(ejecutar(Vec::<String>::new()).codigo, 2);
     }
+
+    #[test]
+    fn guardar_sin_mensaje_o_sin_rutas_es_uso() {
+        let s = ejecutar(["guardar"]);
+        assert_eq!(
+            (s.codigo, s.stdout.as_str()),
+            (
+                2,
+                "guardar uso motivo=sin-mensaje
+"
+            )
+        );
+        let s = ejecutar(["guardar", "--json", "mensaje"]);
+        assert_eq!(s.codigo, 2);
+        assert!(
+            s.stdout
+                .starts_with("{\"comando\":\"guardar\",\"resultado\":\"uso\"")
+        );
+        let s = ejecutar(["actualizar-rama", "de-mas"]);
+        assert_eq!(
+            s.stdout,
+            "actualizar-rama uso motivo=argumentos-de-mas
+"
+        );
+    }
 }
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-017/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-017/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-017/tarea.md                                    | 13 +++++++++----
 3 files changed, 9 insertions(+), 4 deletions(-)
````

## Tests del diff (5 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/comun/mod.rs" ":(literal)crates/tc2/tests/git_binario.rs" ":(literal)crates/tc2/tests/git_ciclo.rs" ":(literal)crates/tc2/tests/git_conflicto.rs" ":(literal)crates/tc2/tests/git_push.rs"
````

````
crates/tc2/tests/comun/mod.rs     | 261 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/git_binario.rs   |  87 +++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/git_ciclo.rs     | 109 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/git_conflicto.rs |  93 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/git_push.rs      | 125 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 5 files changed, 675 insertions(+)
````
