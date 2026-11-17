# Peticion de revision — TASK-080 (ronda 1)

- Tarea: TASK-080 — F3-9 Inventario de skills y agentes instalados
- Rama revisada: feature/task-080-f3-9-inventario-de-skills-y-agentes-inst
- Rama base: develop
- Commit revisado (HEAD): 79c0ce94aa17c4a16e87c010e64ab5ef88e25586
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-080 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-080/tarea.md
- tareas/03-en-revision/TASK-080/planificacion/
- crates/tc2/src/desinstalar/maquina.rs
- crates/tc2/src/doctor/mod.rs
- crates/tc2/src/doctor/recursos.rs
- crates/tc2/src/i18n/catalogo/doctor.rs
- crates/tc2/src/inventario.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/recursos.rs
- crates/tc2/tests/doctor.rs
- crates/tc2/tests/frescura.rs
- crates/tc2/tests/recursos.rs

## Commits a revisar (git log develop..HEAD)

````
79c0ce9 docs(TASK-080): autorrevision con rust-wasm-reviewer y code-quality-reviewer
b1c1c92 feat(TASK-080): inventario con los plugins sincronizados desde claude.ai
185b6ff docs(TASK-080): doctor --recursos en instalar.md y adenda de F1-14
4c609ec feat(TASK-080): tc2 doctor --recursos
d3b0f4e feat(TASK-080): evaluar_recursos del manifiesto de F1-14
74711b3 refactor(TASK-080): desinstalar lee installed_plugins.json con el inventario
60998eb feat(TASK-080): inventario de skills y agentes instalados
7214ed5 chore(TASK-080): tarea en curso
````

## Diff de tu dominio (10 fichero(s) de 12; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/desinstalar/maquina.rs b/crates/tc2/src/desinstalar/maquina.rs
index da3dac5..357f51a 100644
--- a/crates/tc2/src/desinstalar/maquina.rs
+++ b/crates/tc2/src/desinstalar/maquina.rs
@@ -347,13 +347,15 @@ fn es_tc2(bin: &Path) -> bool {
 }
 
 /// `plugins/installed_plugins.json` de Claude Code (solo lectura), si esta
-/// y es JSON.
+/// y es JSON. La lectura es la del inventario (F3-9).
 fn instalados(c: &Contexto<'_>) -> Result<Option<serde_json::Value>, Informe> {
     let Some(dir) = c.entorno.dir_claude() else {
         return Ok(None);
     };
-    let p = dir.join("plugins").join("installed_plugins.json");
-    Ok(leer(&p)?.and_then(|b| serde_json::from_slice(&b).ok()))
+    match crate::inventario::leer_instalados(&dir) {
+        Ok(v) => Ok(v.and_then(Result::ok)),
+        Err(e) => Err(no_se_lee(&crate::inventario::ruta_instalados(&dir), &e)),
+    }
 }
 
 /// Un nombre o scope que se puede poner en una orden: letras, digitos, `.`,
diff --git a/crates/tc2/src/doctor/mod.rs b/crates/tc2/src/doctor/mod.rs
index bf59fec..77e3686 100644
--- a/crates/tc2/src/doctor/mod.rs
+++ b/crates/tc2/src/doctor/mod.rs
@@ -16,11 +16,19 @@
 //! los peores primero, y si no caben, `y N mas: tc2 doctor --json`. Con
 //! `--json`, una linea con todos los hallazgos (tambien los `ok`). Codigo 3
 //! con alguna `falla` (D5a); 0 con avisos o no-comprobados.
+//!
+//! `doctor --recursos` (F3-9, [`recursos`]): el inventario de skills y
+//! agentes y el manifiesto de recursos, con la primera linea `doctor
+//! <ok|rechazada> fallos=N avisos=N no-comprobado=N skills=N agentes=N
+//! inventario=<12hex>` y `y N mas: tc2 doctor --recursos --json`; con
+//! `--json`, ademas de `hallazgos`, la lista `inventario`. El `doctor`
+//! normal lleva el hallazgo agregado `recursos no-instalados`.
 
 pub mod capacidades;
 mod entorno;
 mod estado;
 pub mod plataforma;
+pub mod recursos;
 
 use std::path::Path;
 
@@ -191,23 +199,61 @@ fn cuantos(hs: &[Hallazgo], e: Estado) -> usize {
     hs.iter().filter(|h| h.estado == e).count()
 }
 
-/// La linea de contrato de `doctor` (funcion pura): `rechazada` con alguna
-/// `falla` (D5a), `ok` si no. El diagnostico nombra el primer fallo.
-pub fn informe(hs: &[Hallazgo], r: &Resumen) -> Informe {
+/// Donde se ve el detalle de `doctor`.
+const DETALLE: &str = "tc2 doctor --json";
+
+/// Donde se ve el detalle de `doctor --recursos`.
+const DETALLE_RECURSOS: &str = "tc2 doctor --recursos --json";
+
+/// El principio de la linea de contrato: `rechazada` con alguna `falla`
+/// (D5a), `ok` si no, y las tres cuentas.
+fn cuentas(hs: &[Hallazgo]) -> Informe {
     let fallos = cuantos(hs, Estado::Falla);
     let resultado = if fallos > 0 {
         Resultado::Rechazada
     } else {
         Resultado::Ok
     };
-    let mut i = Informe::nuevo(NOMBRE, resultado)
+    Informe::nuevo(NOMBRE, resultado)
         .con("fallos", fallos.to_string())
         .con("avisos", cuantos(hs, Estado::Aviso).to_string())
         .con(
             "no-comprobado",
             cuantos(hs, Estado::NoComprobado).to_string(),
         )
-        .con("plataforma", r.plataforma.clone());
+}
+
+/// El diagnostico que nombra el primer fallo, si lo hay.
+fn con_diagnostico(i: Informe, hs: &[Hallazgo], detalle: &str) -> Informe {
+    match ordenados(hs)
+        .into_iter()
+        .find(|h| h.estado == Estado::Falla)
+    {
+        Some(primero) => i.diagnostico(
+            Mensaje::nuevo(&txt::CON_FALLOS)
+                .con("n", cuantos(hs, Estado::Falla).to_string())
+                .con("primero", primero.linea())
+                .con("detalle", detalle),
+        ),
+        None => i,
+    }
+}
+
+/// La linea de contrato de `doctor --recursos` (funcion pura): las cuentas,
+/// `skills=`, `agentes=` e `inventario=` (la huella), sin `plataforma=`.
+pub fn informe_recursos(hs: &[Hallazgo], inv: &crate::inventario::Inventario) -> Informe {
+    use crate::inventario::Tipo;
+    let i = cuentas(hs)
+        .con("skills", inv.cuantas(Tipo::Skill).to_string())
+        .con("agentes", inv.cuantas(Tipo::Agente).to_string())
+        .con("inventario", inv.huella());
+    con_diagnostico(i, hs, DETALLE_RECURSOS)
+}
+
+/// La linea de contrato de `doctor` (funcion pura): `rechazada` con alguna
+/// `falla` (D5a), `ok` si no. El diagnostico nombra el primer fallo.
+pub fn informe(hs: &[Hallazgo], r: &Resumen) -> Informe {
+    let mut i = cuentas(hs).con("plataforma", r.plataforma.clone());
     if let Some(c) = &r.capacidades {
         i = i.con("plan", c.plan.clone());
         if let Some((imp, sus, nod)) = c.cuentas {
@@ -226,17 +272,7 @@ pub fn informe(hs: &[Hallazgo], r: &Resumen) -> Informe {
     if let Some(s) = &r.sincronizado {
         i = i.con("sincronizado", s.clone());
     }
-    if let Some(primero) = ordenados(hs)
-        .into_iter()
-        .find(|h| h.estado == Estado::Falla)
-    {
-        i = i.diagnostico(
-            Mensaje::nuevo(&txt::CON_FALLOS)
-                .con("n", fallos.to_string())
-                .con("primero", primero.linea()),
-        );
-    }
-    i
+    con_diagnostico(i, hs, DETALLE)
 }
 
 /// Los hallazgos que no son `ok`, los peores primero (estable: dentro de
@@ -251,38 +287,79 @@ pub fn ordenados(hs: &[Hallazgo]) -> Vec<&Hallazgo> {
 /// los hallazgos que caben en [`MAX_LINEAS`]; si no caben todos, los
 /// primeros y `y N mas: tc2 doctor --json`.
 pub fn lineas(hs: &[Hallazgo], r: &Resumen) -> Vec<String> {
-    let mut v = vec![informe(hs, r).linea()];
+    lineas_de(informe(hs, r).linea(), hs, DETALLE)
+}
+
+/// Las lineas de stdout de `doctor --recursos` sin `--json` (funcion pura),
+/// con `y N mas: tc2 doctor --recursos --json` si no caben.
+pub fn lineas_recursos(hs: &[Hallazgo], inv: &crate::inventario::Inventario) -> Vec<String> {
+    lineas_de(informe_recursos(hs, inv).linea(), hs, DETALLE_RECURSOS)
+}
+
+fn lineas_de(primera: String, hs: &[Hallazgo], detalle: &str) -> Vec<String> {
+    let mut v = vec![primera];
     let malos = ordenados(hs);
     let hueco = MAX_LINEAS - 1;
     if malos.len() <= hueco {
         v.extend(malos.iter().map(|h| h.linea()));
     } else {
         v.extend(malos.iter().take(hueco - 1).map(|h| h.linea()));
-        v.push(format!(
-            "y {} mas: tc2 doctor --json",
-            malos.len() - (hueco - 1)
-        ));
+        v.push(format!("y {} mas: {detalle}", malos.len() - (hueco - 1)));
     }
     v
 }
 
-/// La salida del proceso: texto o JSON (una linea con todos los
-/// hallazgos), diagnostico en stderr solo con alguna `falla`.
-pub fn salida(hs: &[Hallazgo], r: &Resumen, json: bool, idioma: Idioma) -> Salida {
-    let i = informe(hs, r);
+/// La salida de un informe: texto (`lineas`) o JSON (una linea con todos
+/// los hallazgos y `extra`, ya con su clave), diagnostico en stderr solo con
+/// alguna `falla`.
+fn salida_de(
+    i: &Informe,
+    hs: &[Hallazgo],
+    json: bool,
+    idioma: Idioma,
+    lineas: Vec<String>,
+    extra: &str,
+) -> Salida {
     let mut s = i.salida_en(json, idioma);
     if json {
         let todos: Vec<String> = hs.iter().map(Hallazgo::json).collect();
         let linea = s.stdout.trim_end();
         if let Some(sin_cierre) = linea.strip_suffix('}') {
-            s.stdout = format!("{sin_cierre},\"hallazgos\":[{}]}}\n", todos.join(","));
+            s.stdout = format!(
+                "{sin_cierre},\"hallazgos\":[{}]{extra}}}\n",
+                todos.join(",")
+            );
         }
     } else {
-        s.stdout = lineas(hs, r).join("\n") + "\n";
+        s.stdout = lineas.join("\n") + "\n";
     }
     s
 }
 
+/// La salida del proceso: texto o JSON (una linea con todos los
+/// hallazgos), diagnostico en stderr solo con alguna `falla`.
+pub fn salida(hs: &[Hallazgo], r: &Resumen, json: bool, idioma: Idioma) -> Salida {
+    salida_de(&informe(hs, r), hs, json, idioma, lineas(hs, r), "")
+}
+
+/// La salida de `doctor --recursos`: como [`salida`], y en JSON ademas la
+/// lista `inventario` (D4a: la lista completa solo en JSON).
+pub fn salida_recursos(
+    hs: &[Hallazgo],
+    inv: &crate::inventario::Inventario,
+    json: bool,
+    idioma: Idioma,
+) -> Salida {
+    salida_de(
+        &informe_recursos(hs, inv),
+        hs,
+        json,
+        idioma,
+        lineas_recursos(hs, inv),
+        &format!(",\"inventario\":{}", inv.json()),
+    )
+}
+
 /// Lo que necesita `doctor` del entorno.
 pub struct Entorno<'a> {
     /// Carpeta desde la que se lanza.
@@ -291,6 +368,10 @@ pub struct Entorno<'a> {
     pub git: &'a Git,
     /// Construye el adaptador de la plataforma.
     pub conectar: &'a Conectar<'a>,
+    /// Carpeta de Claude Code (`CLAUDE_CONFIG_DIR` o `~/.claude`) para el
+    /// inventario. `None`: no se sabe; el `doctor` normal no agrega los
+    /// recursos y `--recursos` lo dice como `no-comprobado`.
+    pub claude: Option<&'a Path>,
 }
 
 /// Todas las comprobaciones. `Err` solo si no se puede ni empezar (fuera de
@@ -325,6 +406,9 @@ pub fn comprobar(e: &Entorno<'_>) -> Result<(Vec<Hallazgo>, Resumen), Informe> {
     hs.extend(estado::instalacion(e.git, e.dir));
     let (h, plataforma, capacidades) = plataforma::comprobar(e);
     hs.extend(h);
+    if e.claude.is_some() {
+        hs.push(recursos::agregado(e));
+    }
     // La hora de la marca, sin `fetch` (D4a de F3-5b: doctor solo lee).
     let sincronizado =
         crate::estado::sincronizado_texto(crate::estado::sincronizado_en(e.git, e.dir));
@@ -339,21 +423,28 @@ pub fn comprobar(e: &Entorno<'_>) -> Result<(Vec<Hallazgo>, Resumen), Informe> {
     ))
 }
 
-/// Lee las opciones (sin `--json`): ninguna.
-fn leer_opciones(args: &[String]) -> Result<(), Informe> {
-    if args.is_empty() {
-        Ok(())
-    } else {
-        Err(Informe::nuevo(NOMBRE, Resultado::Uso)
+/// Lee las opciones (sin `--json`): nada o `--recursos` (devuelve si esta).
+fn leer_opciones(args: &[String]) -> Result<bool, Informe> {
+    match args {
+        [] => Ok(false),
+        [a] if a == "--recursos" => Ok(true),
+        _ => Err(Informe::nuevo(NOMBRE, Resultado::Uso)
             .con("motivo", "argumentos")
-            .diagnostico(&txt::USO_DOCTOR))
+            .diagnostico(&txt::USO_DOCTOR)),
     }
 }
 
-/// `doctor [--json]` (sin `--json`, que quita `lib.rs`).
+/// `doctor [--recursos] [--json]` (sin `--json`, que quita `lib.rs`).
 pub fn orden(args: &[String], json: bool, idioma: Idioma, e: &Entorno<'_>) -> Salida {
-    if let Err(i) = leer_opciones(args) {
-        return i.salida_en(json, idioma);
+    let con_recursos = match leer_opciones(args) {
+        Ok(r) => r,
+        Err(i) => return i.salida_en(json, idioma),
+    };
+    if con_recursos {
+        return match recursos::comprobar(e) {
+            Ok((hs, inv)) => salida_recursos(&hs, &inv, json, idioma),
+            Err(i) => i.salida_en(json, idioma),
+        };
     }
     match comprobar(e) {
         Ok((hs, r)) => salida(&hs, &r, json, idioma),
@@ -469,15 +560,86 @@ mod tests {
         );
     }
 
+    /// C11: `--recursos` solo; cualquier otra cosa, `doctor uso
+    /// motivo=argumentos`.
     #[test]
     fn argumentos_de_mas_son_uso() {
-        assert!(leer_opciones(&[]).is_ok());
-        for a in ["--recursos", "x"] {
-            let i = leer_opciones(&[a.to_owned()]).err();
+        assert_eq!(leer_opciones(&[]).ok(), Some(false));
+        assert_eq!(leer_opciones(&["--recursos".to_owned()]).ok(), Some(true));
+        for a in [
+            vec!["x"],
+            vec!["--recursos", "x"],
+            vec!["--recursos", "--recursos"],
+            vec!["x", "--recursos"],
+        ] {
+            let args: Vec<String> = a.iter().map(|s| (*s).to_owned()).collect();
+            let i = leer_opciones(&args).err();
             assert_eq!(
                 i.map(|i| i.linea()).as_deref(),
-                Some("doctor uso motivo=argumentos")
+                Some("doctor uso motivo=argumentos"),
+                "{a:?}"
             );
         }
     }
+
+    /// C4: la primera linea de `--recursos` lleva `skills=`, `agentes=` e
+    /// `inventario=` (12 hex) sin `plataforma=`; como mucho 5 lineas, con `y
+    /// N mas: tc2 doctor --recursos --json`. C5: el JSON en una linea con
+    /// `hallazgos` (tambien los ok) e `inventario`, sin ANSI.
+    #[test]
+    fn salida_de_recursos() {
+        use crate::inventario::{Entrada, Inventario, Origen, Tipo};
+        let inv = Inventario::nuevo(
+            vec![
+                Entrada {
+                    tipo: Tipo::Agente,
+                    id: "general-purpose".to_owned(),
+                    origen: Origen::Integrado,
+                },
+                Entrada {
+                    tipo: Tipo::Skill,
+                    id: "alfa".to_owned(),
+                    origen: Origen::Usuario,
+                },
+            ],
+            Vec::new(),
+        );
+        let hs = vec![
+            h("recursos", "taskcodetwo", Estado::Falla),
+            h("recursos", "a", Estado::Aviso),
+            h("recursos", "b", Estado::Aviso),
+            h("recursos", "c", Estado::NoComprobado),
+            h("recursos", "d", Estado::Aviso),
+            h("recursos", "alfa", Estado::Ok),
+        ];
+        let l = lineas_recursos(&hs, &inv);
+        assert_eq!(l.len(), MAX_LINEAS);
+        assert_eq!(
+            l[0],
+            format!(
+                "doctor rechazada fallos=1 avisos=3 no-comprobado=1 skills=1 agentes=1 inventario={}",
+                inv.huella()
+            )
+        );
+        assert!(!l[0].contains("plataforma="));
+        assert_eq!(l[1], "recursos taskcodetwo falla motivo=m");
+        assert_eq!(l[4], "y 2 mas: tc2 doctor --recursos --json");
+        let s = salida_recursos(&hs, &inv, false, Idioma::Es);
+        assert_eq!(s.codigo, 3);
+        assert_eq!(s.stderr.lines().count(), 1);
+        assert!(
+            s.stderr.contains("tc2 doctor --recursos --json"),
+            "{}",
+            s.stderr
+        );
+        let j = salida_recursos(&hs, &inv, true, Idioma::Es);
+        assert_eq!(j.stdout.lines().count(), 1);
+        assert!(!j.stdout.contains('\u{1b}'));
+        let v: serde_json::Value = serde_json::from_str(&j.stdout).unwrap_or_default();
+        assert_eq!(v["hallazgos"].as_array().map(Vec::len), Some(6));
+        assert_eq!(v["hallazgos"][5]["estado"], "ok");
+        assert_eq!(v["claves"]["inventario"], inv.huella());
+        assert_eq!(v["inventario"][0]["id"], "alfa");
+        assert_eq!(v["inventario"][1]["origen"], "integrado");
+    }
 }
diff --git a/crates/tc2/src/doctor/recursos.rs b/crates/tc2/src/doctor/recursos.rs
new file mode 100644
index 0000000..c26942e
--- /dev/null
+++ b/crates/tc2/src/doctor/recursos.rs
@@ -0,0 +1,294 @@
+//! `tc2 doctor --recursos` (F3-9; ADR F1-14 punto 8): el inventario de
+//! skills y agentes de la maquina y del proyecto y el manifiesto de recursos,
+//! como hallazgos del area `recursos`.
+//!
+//! Lee, sin escribir nada: la carpeta de Claude Code (la que da
+//! `CLAUDE_CONFIG_DIR` o `~/.claude`), `.claude/` del proyecto, `recursos:`
+//! de la configuracion de la base (`origin/develop`) y `roles.*` de las
+//! tareas no terminadas de la rama de estado (lo ultimo traido, sin
+//! `fetch`). Los ejecutables se buscan en el PATH.
+//!
+//! [`hallazgo`] adapta un [`Recurso`] de [`crate::recursos`] a un
+//! [`Hallazgo`]: `recursos <id> <estado> [motivo=] tipo= [tarea=] [nivel=]
+//! [version=] [arreglo=]`. `nivel=` solo se escribe si no es `necesario`
+//! (el de lo que declara el proyecto o la tarea).
+
+use std::collections::BTreeSet;
+use std::path::{Path, PathBuf};
+
+use super::{Entorno, Estado, Hallazgo, NOMBRE, entorno};
+use crate::contrato::Informe;
+use crate::estado::Estado as _;
+use crate::git::{Subcomando, correr};
+use crate::grafo::{DESCARTADA, TERMINADA};
+use crate::inventario::{self, Inventario};
+use crate::recursos::{
+    self, Fuente, Manifiesto, Momento, NO_INSTALADO, Nivel, Recomendado, Recurso, TipoRecurso,
+};
+use crate::tarea::Documento;
+
+/// Area de los hallazgos.
+pub const AREA: &str = "recursos";
+
+/// Arreglo del hallazgo agregado del `doctor` normal.
+pub const ARREGLO: &str = "tc2 doctor --recursos";
+
+/// Ejecutables convenientes que se buscan siempre (F1-14, punto 8).
+const CONVENIENTES: [&str; 1] = ["rtk"];
+
+/// El hallazgo de un recurso evaluado (funcion pura).
+pub fn hallazgo(x: &Recurso) -> Hallazgo {
+    let estado = match x.estado {
+        recursos::Estado::Falla => Estado::Falla,
+        recursos::Estado::Aviso => Estado::Aviso,
+        recursos::Estado::NoComprobado => Estado::NoComprobado,
+        recursos::Estado::Ok => Estado::Ok,
+    };
+    let mut h = Hallazgo::nuevo(AREA, x.id.clone(), estado);
+    if let Some(c) = x.comprobacion {
+        h = h.motivo(c);
+    }
+    h = h.con("tipo", x.tipo);
+    if let Some(t) = &x.tarea {
+        h = h.con("tarea", t.clone());
+    }
+    if x.nivel != Nivel::Necesario {
+        h = h.con("nivel", x.nivel.texto());
+    }
+    if let Some(v) = &x.version {
+        h = h.con("version", v.clone());
+    }
+    if let Some(a) = &x.arreglo {
+        h = h.arreglo(a.clone());
+    }
+    h
+}
+
+/// La raiz del worktree desde el que se lanza (donde esta `.claude/` del
+/// proyecto). Fuera de un repo, el rechazo de `doctor`.
+fn raiz(e: &Entorno<'_>) -> Result<PathBuf, Informe> {
+    entorno::repo(e.git, e.dir)?;
+    let s = correr(
+        e.git,
+        e.dir,
+        NOMBRE,
+        Subcomando::RevParse,
+        &["--show-toplevel"],
+    )?;
+    let t = s.stdout.trim();
+    Ok(if s.exito() && !t.is_empty() {
+        PathBuf::from(t)
+    } else {
+        e.dir.to_path_buf()
+    })
+}
+
+/// Un hallazgo `no-comprobado` de una fuente de recomendaciones.
+fn sin_leer(id: &str, i: &Informe) -> Hallazgo {
+    Hallazgo::nuevo(AREA, id, Estado::NoComprobado).motivo(
+        i.valor("motivo")
+            .map_or_else(|| i.resultado.texto().to_owned(), str::to_owned),
+    )
+}
+
+/// Lo que recomiendan `recursos:` de la configuracion y `roles.*` de las
+/// tareas no terminadas (D1a), con lo que no se pudo leer.
+fn recomendados(e: &Entorno<'_>) -> (Vec<Recomendado>, Vec<Hallazgo>) {
+    let mut v = Vec::new();
+    let mut hs = Vec::new();
+    match crate::config::de_base(e.git, e.dir) {
+        Ok(c) => {
+            for r in c.recursos {
+                if let Some(tipo) = TipoRecurso::de_texto(&r.tipo) {
+                    v.push(Recomendado {
+                        id: r.id,
+                        tipo,
+                        fuente: Fuente::Config,
+                    });
+                }
+            }
+        }
+        // Sin configuracion no hay `recursos:` que comprobar.
+        Err(i) if i.valor("motivo") == Some("sin-config") => {}
+        Err(i) => hs.push(sin_leer("config", &i)),
+    }
+    let vista = match crate::estado::leer(e.git, e.dir) {
+        Ok(v) => v,
+        Err(i) => {
+            hs.push(sin_leer("tareas", &i));
+            return (v, hs);
+        }
+    };
+    let leidas = match crate::estado::tareas(&vista, &vista.rutas()) {
+        Ok(l) => l,
+        Err(_) => {
+            hs.push(
+                Hallazgo::nuevo(AREA, "tareas", Estado::NoComprobado).motivo("estado-ilegible"),
+            );
+            return (v, hs);
+        }
+    };
+    for (_, l) in leidas {
+        // Las ilegibles ya las cuenta `doctor` en `estado tareas`; las de la
+        // 0.16 no tienen `roles`.
+        let Ok(Documento::Formato2(t, _)) = l else {
+            continue;
+        };
+        if t.estado == TERMINADA || t.estado == DESCARTADA {
+            continue;
+        }
+        let fuente = Fuente::Tarea(t.id.clone());
+        let r = &t.roles;
+        for s in r
+            .diseno
+            .skills
+            .iter()
+            .chain(&r.implementacion.skills)
+            .chain(&r.revision.skills)
+        {
+            v.push(Recomendado {
+                id: s.clone(),
+                tipo: TipoRecurso::Skill,
+                fuente: fuente.clone(),
+            });
+        }
+        if let Some(a) = &r.revision.agente {
+            v.push(Recomendado {
+                id: a.clone(),
+                tipo: TipoRecurso::Agente,
+                fuente,
+            });
+        }
+    }
+    (v, hs)
+}
+
+/// Si hay un ejecutable `nombre` en alguna carpeta del PATH (en Windows,
+/// con las extensiones de `PATHEXT`).
+fn en_path(nombre: &str) -> bool {
+    let Some(path) = std::env::var_os("PATH") else {
+        return false;
+    };
+    let exts: Vec<String> = if cfg!(windows) {
+        std::env::var("PATHEXT")
+            .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_owned())
+            .split(';')
+            .filter(|x| !x.is_empty())
+            .map(str::to_owned)
+            .collect()
+    } else {
+        vec![String::new()]
+    };
+    std::env::split_paths(&path).any(|d| {
+        exts.iter()
+            .any(|x| d.join(format!("{nombre}{x}")).is_file())
+    })
+}
+
+/// Todos los hallazgos de recursos y el inventario. `Err` fuera de un repo.
+pub fn comprobar(e: &Entorno<'_>) -> Result<(Vec<Hallazgo>, Inventario), Informe> {
+    let raiz = raiz(e)?;
+    comprobar_en(e, &raiz)
+}
+
+fn comprobar_en(e: &Entorno<'_>, raiz: &Path) -> Result<(Vec<Hallazgo>, Inventario), Informe> {
+    let mut hs = Vec::new();
+    let lectura = inventario::leer(e.claude, raiz);
+    if e.claude.is_none() {
+        hs.push(
+            Hallazgo::nuevo(AREA, "maquina", Estado::NoComprobado)
+                .motivo("sin-carpeta-de-claude")
+                .arreglo("definir-CLAUDE_CONFIG_DIR-o-HOME"),
+        );
+    }
+    for i in &lectura.ilegibles {
+        hs.push(
+            Hallazgo::nuevo(AREA, i.id.clone(), Estado::NoComprobado)
+                .motivo(i.motivo)
+                .con("ruta", i.ruta.clone()),
+        );
+    }
+    for i in &lectura.avisos {
+        hs.push(
+            Hallazgo::nuevo(AREA, i.id.clone(), Estado::Aviso)
+                .motivo(i.motivo)
+                .con("ruta", i.ruta.clone()),
+        );
+    }
+    let (rec, h) = recomendados(e);
+    hs.extend(h);
+    let buscar: BTreeSet<String> = rec
+        .iter()
+        .filter(|r| r.tipo == TipoRecurso::Ejecutable)
+        .map(|r| r.id.clone())
+        .chain(CONVENIENTES.iter().map(|x| (*x).to_owned()))
+        .collect();
+    let m = Manifiesto {
+        recomendados: rec,
+        ejecutables: buscar.into_iter().filter(|n| en_path(n)).collect(),
+        mcp: lectura.mcp.clone(),
+        completo: e.claude.is_some() && lectura.ilegibles.is_empty(),
+    };
+    let informe = recursos::evaluar_recursos(&lectura.inventario, &m, &Momento::Doctor);
+    hs.extend(informe.recursos.iter().map(hallazgo));
+    Ok((hs, lectura.inventario))
+}
+
+/// El hallazgo agregado del `doctor` normal: `falla ids=N` con lo que no
+/// esta instalado, `no-comprobado` si algo no se pudo leer, `ok` si no.
+pub fn agregado(e: &Entorno<'_>) -> Hallazgo {
+    let h = |x: Estado| Hallazgo::nuevo(AREA, "no-instalados", x);
+    match comprobar(e) {
+        Ok((hs, _)) => {
+            let n = hs
+                .iter()
+                .filter(|h| h.estado == Estado::Falla && h.motivo.as_deref() == Some(NO_INSTALADO))
+                .count();
+            if n > 0 {
+                h(Estado::Falla).con("ids", n.to_string()).arreglo(ARREGLO)
+            } else if hs.iter().any(|h| h.estado == Estado::NoComprobado) {
+                h(Estado::NoComprobado)
+                    .motivo("recursos-sin-comprobar")
+                    .arreglo(ARREGLO)
+            } else {
+                h(Estado::Ok)
+            }
+        }
+        Err(i) => sin_leer("no-instalados", &i),
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    /// C2 (parte pura): la linea de un recomendado que falta.
+    #[test]
+    fn la_linea_de_un_recurso() {
+        let r = Recurso {
+            id: "fantasma".to_owned(),
+            tipo: "skill",
+            nivel: Nivel::Necesario,
+            estado: recursos::Estado::Falla,
+            comprobacion: Some(NO_INSTALADO),
+            tarea: None,
+            version: None,
+            arreglo: Some("instalar-skill-o-quitarla-de-recursos".to_owned()),
+        };
+        assert_eq!(
+            hallazgo(&r).linea(),
+            "recursos fantasma falla motivo=no-instalado tipo=skill arreglo=instalar-skill-o-quitarla-de-recursos"
+        );
+        let r = Recurso {
+            nivel: Nivel::Estorba,
+            estado: recursos::Estado::Aviso,
+            tarea: Some("TASK-001".to_owned()),
+            version: Some("0.16.1".to_owned()),
+            ..r
+        };
+        assert_eq!(
+            hallazgo(&r).linea(),
+            "recursos fantasma aviso motivo=no-instalado tipo=skill tarea=TASK-001 nivel=estorba version=0.16.1 arreglo=instalar-skill-o-quitarla-de-recursos"
+        );
+    }
+}
diff --git a/crates/tc2/src/i18n/catalogo/doctor.rs b/crates/tc2/src/i18n/catalogo/doctor.rs
index ea75626..c424be0 100644
--- a/crates/tc2/src/i18n/catalogo/doctor.rs
+++ b/crates/tc2/src/i18n/catalogo/doctor.rs
@@ -2,11 +2,11 @@
 
 crate::i18n::textos! {
     USO_DOCTOR {
-        es: "uso: doctor [--json] (doctor --recursos llega con F3-9)",
-        en: "usage: doctor [--json] (doctor --recursos comes with F3-9)",
+        es: "uso: doctor [--recursos] [--json]",
+        en: "usage: doctor [--recursos] [--json]",
     }
     CON_FALLOS {
-        es: "doctor encontro {n} fallo(s); el primero: {primero}. Detalle: tc2 doctor --json",
-        en: "doctor found {n} failure(s); the first one: {primero}. Details: tc2 doctor --json",
+        es: "doctor encontro {n} fallo(s); el primero: {primero}. Detalle: {detalle}",
+        en: "doctor found {n} failure(s); the first one: {primero}. Details: {detalle}",
     }
 }
diff --git a/crates/tc2/src/inventario.rs b/crates/tc2/src/inventario.rs
new file mode 100644
index 0000000..6835d69
--- /dev/null
+++ b/crates/tc2/src/inventario.rs
@@ -0,0 +1,1121 @@
+//! Inventario de skills y agentes instalados (F3-9, RF-14; ADR F1-15
+//! decision 2): la unica fuente de lo que existe en esta maquina y en este
+//! proyecto. Lo usan `tc2 doctor --recursos` y, con el manifiesto de F1-14
+//! (punto 8), `init`, `start` y `seleccionar`.
+//!
+//! Fuentes, todas de solo lectura:
+//!
+//! - los plugins de `<claude>/plugins/installed_plugins.json` (version 2:
+//!   `plugins.<nombre@marketplace>` con `installPath`, `version` y `scope`)
+//!   que esten habilitados: `enabledPlugins` de `settings.json` del usuario,
+//!   `.claude/settings.json` y `.claude/settings.local.json` del proyecto, el
+//!   ultimo que lo dice gana (D2a: `false` no cuenta como instalado); sin
+//!   entrada, `defaultEnabled` de su `.claude-plugin/plugin.json` (por defecto
+//!   `true`). Un `scope` de proyecto o local solo cuenta si su `projectPath`
+//!   es este proyecto. Sus skills (`skills/<n>/SKILL.md`) y agentes
+//!   (`agents/<n>.md`) llevan el prefijo del plugin (`plugin:id`);
+//! - los plugins sincronizados desde claude.ai (`<claude>/plugins/synced/`,
+//!   id `<name>@synced`): los del `manifest.json` de cuenta mas reciente,
+//!   salvo `"<name>@synced": false`, `defaultEnabled: false` o
+//!   `syncClaudeAiPlugins: false`; ceden ante un plugin de otro origen con el
+//!   mismo nombre y `plugins/.trash/` no cuenta;
+//! - `<claude>/skills` y `<claude>/agents` (usuario);
+//! - `.claude/skills` y `.claude/agents` del proyecto;
+//! - los agentes integrados de Claude Code ([`INTEGRADOS`]).
+//!
+//! El id es el `name:` del frontmatter y, si falta, el nombre de la carpeta o
+//! del fichero, con aviso si no coinciden (D5a). Solo se lee la cabecera de
+//! cada fichero ([`MAX_CABECERA`]). Lo que no se puede leer no se da por
+//! instalado ni por ausente: queda en [`Lectura::ilegibles`] y quien decide
+//! lo dice como `no-comprobado` (C8).
+//!
+//! El resultado no depende del orden del sistema de ficheros: se ordena por
+//! (tipo, id) y no se repite (C10).
+
+use std::collections::{BTreeMap, BTreeSet};
+use std::io::Read;
+use std::path::{Path, PathBuf};
+
+use serde_json::Value;
+
+use crate::contrato::cadena_json;
+
+/// Agentes integrados de Claude Code que siempre estan.
+pub const INTEGRADOS: [&str; 1] = ["general-purpose"];
+
+/// Bytes que se leen como mucho de cada `SKILL.md` o agente: el frontmatter
+/// va al principio y un fichero enorme no se lee entero.
+pub const MAX_CABECERA: u64 = 16 * 1024;
+
+/// Fichero de plugins instalados de Claude Code, relativo a su carpeta.
+const INSTALADOS: [&str; 2] = ["plugins", "installed_plugins.json"];
+
+/// Que es una entrada.
+#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub enum Tipo {
+    /// Una skill (`SKILL.md`).
+    Skill,
+    /// Un agente (`<n>.md` en `agents/`).
+    Agente,
+}
+
+impl Tipo {
+    /// Valor en la salida: `skill` o `agente`.
+    pub fn texto(self) -> &'static str {
+        match self {
+            Self::Skill => "skill",
+            Self::Agente => "agente",
+        }
+    }
+}
+
+/// De donde viene una entrada.
+#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub enum Origen {
+    /// `<claude>/skills` o `<claude>/agents`.
+    Usuario,
+    /// `.claude/skills` o `.claude/agents` del proyecto.
+    Proyecto,
+    /// Un plugin habilitado.
+    Plugin {
+        /// Nombre del plugin (el prefijo de sus ids).
+        nombre: String,
+        /// Version instalada, si se sabe.
+        version: Option<String>,
+    },
+    /// Integrado en Claude Code.
+    Integrado,
+}
+
+impl Origen {
+    /// Valor en la salida: `usuario`, `proyecto`, `plugin` o `integrado`.
+    pub fn texto(&self) -> &'static str {
+        match self {
+            Self::Usuario => "usuario",
+            Self::Proyecto => "proyecto",
+            Self::Plugin { .. } => "plugin",
+            Self::Integrado => "integrado",
+        }
+    }
+}
+
+/// Una skill o un agente instalado.
+#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub struct Entrada {
+    /// Tipo (primero, para ordenar por tipo y luego por id).
+    pub tipo: Tipo,
+    /// Id con el que se nombra (`plugin:n` en un plugin).
+    pub id: String,
+    /// De donde viene.
+    pub origen: Origen,
+}
+
+impl Entrada {
+    /// La entrada como objeto JSON.
+    pub fn json(&self) -> String {
+        let mut s = format!(
+            "{{\"id\":{},\"tipo\":{},\"origen\":{}",
+            cadena_json(&self.id),
+            cadena_json(self.tipo.texto()),
+            cadena_json(self.origen.texto())
+        );
+        if let Origen::Plugin { nombre, version } = &self.origen {
+            s.push_str(&format!(",\"plugin\":{}", cadena_json(nombre)));
+            if let Some(v) = version {
+                s.push_str(&format!(",\"version\":{}", cadena_json(v)));
+            }
+        }
+        s.push('}');
+        s
+    }
+}
+
+/// Un plugin habilitado.
+#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub struct Plugin {
+    /// Nombre (el prefijo de sus ids).
+    pub nombre: String,
+    /// Clave en `installed_plugins.json` (`nombre@marketplace`).
+    pub clave: String,
+    /// Version instalada, si se sabe.
+    pub version: Option<String>,
+}
+
+/// Lo instalado y habilitado, ordenado y sin repetir.
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct Inventario {
+    entradas: Vec<Entrada>,
+    plugins: Vec<Plugin>,
+}
+
+impl Inventario {
+    /// Inventario de `entradas` y `plugins` en cualquier orden: se ordenan
+    /// por (tipo, id) y una entrada repetida (mismo tipo e id) se queda con
+    /// el primer origen en el orden de [`Origen`].
+    pub fn nuevo(mut entradas: Vec<Entrada>, mut plugins: Vec<Plugin>) -> Self {
+        entradas.sort();
+        entradas.dedup_by(|b, a| a.tipo == b.tipo && a.id == b.id);
+        plugins.sort();
+        plugins.dedup_by(|b, a| a.nombre == b.nombre);
+        Self { entradas, plugins }
+    }
+
+    /// Las entradas, ordenadas por (tipo, id).
+    pub fn entradas(&self) -> &[Entrada] {
+        &self.entradas
+    }
+
+    /// Los plugins habilitados, por nombre.
+    pub fn plugins(&self) -> &[Plugin] {
+        &self.plugins
+    }
+
+    /// Si hay una entrada de `tipo` con `id`.
+    pub fn contiene(&self, tipo: Tipo, id: &str) -> bool {
+        self.entradas.iter().any(|e| e.tipo == tipo && e.id == id)
+    }
+
+    /// El plugin habilitado de nombre `nombre`.
+    pub fn plugin(&self, nombre: &str) -> Option<&Plugin> {
+        self.plugins.iter().find(|p| p.nombre == nombre)
+    }
+
+    /// Cuantas entradas hay de `tipo`.
+    pub fn cuantas(&self, tipo: Tipo) -> usize {
+        self.entradas.iter().filter(|e| e.tipo == tipo).count()
+    }
+
+    /// Huella del inventario (12 hex del sha256 de su forma canonica): la
+    /// que se anota en `roles.<rol>.inventario` (F1-15 decision 10).
+    pub fn huella(&self) -> String {
+        let mut s = String::new();
+        for e in &self.entradas {
+            s.push_str(&format!(
+                "{}\t{}\t{}",
+                e.tipo.texto(),
+                e.id,
+                e.origen.texto()
+            ));
+            if let Origen::Plugin { nombre, version } = &e.origen {
+                s.push_str(&format!("\t{nombre}\t{}", version.as_deref().unwrap_or("")));
+            }
+            s.push('\n');
+        }
+        for p in &self.plugins {
+            s.push_str(&format!(
+                "plugin\t{}\t{}\t{}\n",
+                p.nombre,
+                p.clave,
+                p.version.as_deref().unwrap_or("")
+            ));
+        }
+        crate::contrato::corto(&distribucion::sha256::hex(s.as_bytes()))
+    }
+
+    /// Las entradas como lista JSON.
+    pub fn json(&self) -> String {
+        let v: Vec<String> = self.entradas.iter().map(Entrada::json).collect();
+        format!("[{}]", v.join(","))
+    }
+}
+
+/// Algo que se encontro al leer: un fichero que no se pudo leer o un
+/// nombre que no coincide con su carpeta.
+#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
+pub struct Incidencia {
+    /// A que se refiere: el id de la entrada o la clave del plugin.
+    pub id: String,
+    /// Por que (sin espacios): `no-se-lee`, `json-invalido`,
+    /// `formato-desconocido`, `sin-install-path`, `install-path-inexistente`
+    /// o `nombre-distinto`.
+    pub motivo: &'static str,
+    /// Fichero o carpeta.
+    pub ruta: String,
+}
+
+/// Lo que da [`leer`].
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct Lectura {
+    /// Lo instalado y habilitado.
+    pub inventario: Inventario,
+    /// Fuentes que no se pudieron leer: el inventario puede estar
+    /// incompleto.
+    pub ilegibles: Vec<Incidencia>,
+    /// Avisos que no impiden leer (`nombre-distinto`).
+    pub avisos: Vec<Incidencia>,
+    /// Nombres de servidores MCP configurados (deteccion aproximada para el
+    /// nivel conveniente de F1-14): `.claude.json`, `.mcp.json` del proyecto y
+    /// de cada plugin habilitado.
+    pub mcp: BTreeSet<String>,
+}
+
+/// Lo que se va acumulando al leer.
+#[derive(Default)]
+struct Acumulado {
+    entradas: Vec<Entrada>,
+    plugins: Vec<Plugin>,
+    ilegibles: Vec<Incidencia>,
+    avisos: Vec<Incidencia>,
+    mcp: BTreeSet<String>,
+}
+
+fn t(p: &Path) -> String {
+    p.display().to_string()
+}
+
+/// Lee un fichero; `Ok(None)` si no existe.
+fn leer_fichero(p: &Path) -> std::io::Result<Option<Vec<u8>>> {
+    match std::fs::read(p) {
+        Ok(b) => Ok(Some(b)),
+        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
+        Err(e) => Err(e),
+    }
+}
+
+/// Ruta de `installed_plugins.json` en la carpeta de Claude Code.
+pub fn ruta_instalados(dir_claude: &Path) -> PathBuf {
+    INSTALADOS
+        .iter()
+        .fold(dir_claude.to_path_buf(), |p, s| p.join(s))
+}
+
+/// `installed_plugins.json` de Claude Code (solo lectura): `Ok(None)` si no
+/// existe; `Ok(Some(Err))` si no es JSON. Lo usan el inventario y
+/// `desinstalar --maquina`.
+pub fn leer_instalados(
+    dir_claude: &Path,
+) -> std::io::Result<Option<Result<Value, serde_json::Error>>> {
+    Ok(leer_fichero(&ruta_instalados(dir_claude))?.map(|b| serde_json::from_slice(&b)))
+}
+
+/// Un JSON de configuracion: `None` si no existe; incidencia si no se lee o
+/// no es JSON.
+fn json_de(p: &Path, a: &mut Acumulado, id: &str) -> Option<Value> {
+    match leer_fichero(p) {
+        Ok(None) => None,
+        Ok(Some(b)) => match serde_json::from_slice(&b) {
+            Ok(v) => Some(v),
+            Err(_) => {
+                a.ilegibles.push(Incidencia {
+                    id: id.to_owned(),
+                    motivo: "json-invalido",
+                    ruta: t(p),
+                });
+                None
+            }
+        },
+        Err(_) => {
+            a.ilegibles.push(Incidencia {
+                id: id.to_owned(),
+                motivo: "no-se-lee",
+                ruta: t(p),
+            });
+            None
+        }
+    }
+}
+
+/// El `name:` del frontmatter (`---` ... `---`) de los primeros
+/// [`MAX_CABECERA`] bytes; `None` si no hay. Las comillas se quitan.
+pub fn nombre_del_frontmatter(cabecera: &str) -> Option<String> {
+    let texto = cabecera.strip_prefix('\u{feff}').unwrap_or(cabecera);
+    let mut lineas = texto.lines();
+    if lineas.next().map(str::trim_end) != Some("---") {
+        return None;
+    }
+    for l in lineas {
+        let l = l.trim_end();
+        if l == "---" {
+            break;
+        }
+        if let Some(v) = l.strip_prefix("name:") {
+            let v = v.trim();
+            let v = v
+                .strip_prefix('"')
+                .and_then(|x| x.strip_suffix('"'))
+                .or_else(|| v.strip_prefix('\'').and_then(|x| x.strip_suffix('\'')))
+                .unwrap_or(v)
+                .trim();
+            return (!v.is_empty()).then(|| v.to_owned());
+        }
+    }
+    None
+}
+
+/// La cabecera de `p` (como mucho [`MAX_CABECERA`] bytes).
+fn cabecera(p: &Path) -> std::io::Result<String> {
+    let mut b = Vec::new();
+    std::fs::File::open(p)?
+        .take(MAX_CABECERA)
+        .read_to_end(&mut b)?;
+    Ok(String::from_utf8_lossy(&b).into_owned())
+}
+
+/// Hijos de `dir` ordenados (vacio si no existe).
+fn hijos(dir: &Path) -> Vec<PathBuf> {
+    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
+        .map(|d| d.flatten().map(|e| e.path()).collect())
+        .unwrap_or_default();
+    v.sort();
+    v
+}
+
+/// Lee las skills de `dir` (`<n>/SKILL.md`) y los agentes de `agentes`
+/// (`<n>.md`) con el `prefijo` y el `origen` dados.
+fn leer_carpetas(skills: &Path, agentes: &Path, prefijo: &str, origen: &Origen, a: &mut Acumulado) {
+    let mut ficheros: Vec<(Tipo, PathBuf, String)> = Vec::new();
+    for d in hijos(skills) {
+        let f = d.join("SKILL.md");
+        if d.is_dir()
+            && f.is_file()
+            && let Some(n) = d.file_name().and_then(|n| n.to_str())
+        {
+            ficheros.push((Tipo::Skill, f, n.to_owned()));
+        }
+    }
+    for f in hijos(agentes) {
+        if f.is_file()
+            && f.extension().is_some_and(|e| e.eq_ignore_ascii_case("md"))
+            && let Some(n) = f.file_stem().and_then(|n| n.to_str())
+        {
+            let n = n.to_owned();
+            ficheros.push((Tipo::Agente, f, n));
+        }
+    }
+    for (tipo, f, carpeta) in ficheros {
+        let nombre = match cabecera(&f) {
+            Ok(c) => nombre_del_frontmatter(&c),
+            Err(_) => {
+                a.ilegibles.push(Incidencia {
+                    id: format!("{prefijo}{carpeta}"),
+                    motivo: "no-se-lee",
+                    ruta: t(&f),
+                });
+                continue;
+            }
+        };
+        let id = match nombre {
+            Some(n) => {
+                if n != carpeta {
+                    a.avisos.push(Incidencia {
+                        id: format!("{prefijo}{n}"),
+                        motivo: "nombre-distinto",
+                        ruta: t(&f),
+                    });
+                }
+                n
+            }
+            None => carpeta,
+        };
+        a.entradas.push(Entrada {
+            tipo,
+            id: format!("{prefijo}{id}"),
+            origen: origen.clone(),
+        });
+    }
+}
+
+/// Las claves de `mcpServers` de un JSON.
+fn servidores_mcp(v: &Value, a: &mut Acumulado) {
+    if let Some(m) = v.get("mcpServers").and_then(Value::as_object) {
+        a.mcp.extend(m.keys().cloned());
+    }
+}
+
+/// Lo que dicen los settings sobre los plugins.
+struct Ajustes {
+    /// `enabledPlugins` de los tres ficheros, el ultimo gana.
+    habilitados: BTreeMap<String, bool>,
+    /// `syncClaudeAiPlugins` del usuario (por defecto `true`).
+    sincronizar: bool,
+}
+
+/// `enabledPlugins` de los tres ficheros de settings (el ultimo gana) y
+/// `syncClaudeAiPlugins` de los del usuario.
+fn ajustes(dir_claude: &Path, raiz: &Path, a: &mut Acumulado) -> Ajustes {
+    let mut m = BTreeMap::new();
+    let mut sincronizar = true;
+    let claude = raiz.join(".claude");
+    for (i, p) in [
+        dir_claude.join("settings.json"),
+        claude.join("settings.json"),
+        claude.join("settings.local.json"),
+    ]
+    .iter()
+    .enumerate()
+    {
+        let Some(v) = json_de(p, a, "settings") else {
+            continue;
+        };
+        if let Some(e) = v.get("enabledPlugins").and_then(Value::as_object) {
+            for (k, x) in e {
+                if let Some(b) = x.as_bool() {
+                    m.insert(k.clone(), b);
+                }
+            }
+        }
+        if i == 0 && v.get("syncClaudeAiPlugins").and_then(Value::as_bool) == Some(false) {
+            sincronizar = false;
+        }
+    }
+    Ajustes {
+        habilitados: m,
+        sincronizar,
+    }
+}
+
+/// Carga el plugin de `dir` con clave `clave` (`nombre@origen`) si esta
+/// habilitado: `enabledPlugins` o, sin entrada, `defaultEnabled` de su
+/// manifiesto. Su nombre (prefijo de los ids) es el `name` del manifiesto o
+/// `por_defecto`. `omitir` descarta un nombre ya cargado (los sincronizados
+/// ceden ante cualquier otro origen). Devuelve si lo cargo.
+fn cargar_plugin(
+    dir: &Path,
+    clave: &str,
+    por_defecto: &str,
+    version: Option<&str>,
+    ajustes: &Ajustes,
+    omitir: bool,
+    a: &mut Acumulado,
+) -> bool {
+    let manifiesto = json_de(&dir.join(".claude-plugin").join("plugin.json"), a, clave);
+    let por_defecto_on = manifiesto
+        .as_ref()
+        .and_then(|m| m.get("defaultEnabled"))
+        .and_then(Value::as_bool)
+        .unwrap_or(true);
+    if !ajustes
+        .habilitados
+        .get(clave)
+        .copied()
+        .unwrap_or(por_defecto_on)
+    {
+        return false;
+    }
+    let nombre = manifiesto
+        .as_ref()
+        .and_then(|m| m.get("name"))
+        .and_then(Value::as_str)
+        .filter(|n| !n.is_empty())
+        .unwrap_or(por_defecto)
+        .to_owned();
+    if omitir && a.plugins.iter().any(|p| p.nombre == nombre) {
+        return false;
+    }
+    let version = version
+        .or_else(|| {
+            manifiesto
+                .as_ref()
+                .and_then(|m| m.get("version"))
+                .and_then(Value::as_str)
+        })
+        .map(str::to_owned);
+    if let Some(m) = json_de(&dir.join(".mcp.json"), a, clave) {
+        servidores_mcp(&m, a);
+    }
+    let origen = Origen::Plugin {
+        nombre: nombre.clone(),
+        version: version.clone(),
+    };
+    leer_carpetas(
+        &dir.join("skills"),
+        &dir.join("agents"),
+        &format!("{nombre}:"),
+        &origen,
+        a,
+    );
+    a.plugins.push(Plugin {
+        nombre,
+        clave: clave.to_owned(),
+        version,
+    });
+    true
+}
+
+/// Clave de orden de `lastUpdated` (numero o texto ISO).
+fn orden_de_fecha(v: Option<&Value>) -> String {
+    match v {
+        Some(Value::Number(n)) => format!("{:030.3}", n.as_f64().unwrap_or(0.0)),
+        Some(Value::String(s)) => s.clone(),
+        _ => String::new(),
+    }
+}
+
+/// Los plugins sincronizados desde claude.ai (`plugins/synced/<cuenta>/`,
+/// id `<name>@synced`; doc de Claude Code «Plugin loading reference»). Cuenta
+/// solo la carpeta de cuenta cuyo `manifest.json` tiene el `lastUpdated` mas
+/// reciente (la de la sesion que sincroniza en cada arranque); de ella, cada
+/// plugin del manifiesto con su carpeta `<name>~g<generation>` o `<name>`.
+/// `plugins/.trash/` no cuenta. Con `syncClaudeAiPlugins: false` no se leen.
+fn leer_sincronizados(dir_claude: &Path, ajustes: &Ajustes, a: &mut Acumulado) {
+    if !ajustes.sincronizar {
+        return;
+    }
+    let raiz = dir_claude.join("plugins").join("synced");
+    let mut cuentas: Vec<(String, PathBuf, Value)> = Vec::new();
+    for c in hijos(&raiz).into_iter().filter(|c| c.is_dir()) {
+        let id = format!(
+            "synced/{}",
+            c.file_name().and_then(|n| n.to_str()).unwrap_or_default()
+        );
+        let ruta = c.join("manifest.json");
+        if !ruta.is_file() {
+            continue;
+        }
+        let Some(m) = json_de(&ruta, a, &id) else {
+            continue;
+        };
+        if !m.get("plugins").is_some_and(Value::is_array) {
+            a.ilegibles.push(Incidencia {
+                id,
+                motivo: "formato-desconocido",
+                ruta: t(&ruta),
+            });
+            continue;
+        }
+        cuentas.push((orden_de_fecha(m.get("lastUpdated")), c, m));
+    }
+    // La mas reciente; a igualdad, la ultima por nombre (determinista).
+    let Some((_, cuenta, m)) = cuentas
+        .into_iter()
+        .max_by(|x, y| (&x.0, &x.1).cmp(&(&y.0, &y.1)))
+    else {
+        return;
+    };
+    let lista = m
+        .get("plugins")
+        .and_then(Value::as_array)
+        .cloned()
+        .unwrap_or_default();
+    for p in lista {
+        let Some(nombre) = p
+            .get("name")
+            .and_then(Value::as_str)
+            .filter(|n| !n.is_empty())
+        else {
+            continue;
+        };
+        let generacion = p.get("generation").and_then(Value::as_u64).unwrap_or(1);
+        let candidatas = [
+            (generacion > 1).then(|| cuenta.join(format!("{nombre}~g{generacion}"))),
+            Some(cuenta.join(nombre)),
+        ];
+        let Some(dir) = candidatas.into_iter().flatten().find(|d| d.is_dir()) else {
+            a.ilegibles.push(Incidencia {
+                id: format!("{nombre}@synced"),
+                motivo: "sin-carpeta",
+                ruta: t(&cuenta),
+            });
+            continue;
+        };
+        let version = p.get("version").and_then(Value::as_str);
+        cargar_plugin(
+            &dir,
+            &format!("{nombre}@synced"),
+            nombre,
+            version,
+            ajustes,
+            true,
+            a,
+        );
+    }
+}
+
+/// Si una entrada de `installed_plugins.json` vale para este proyecto: de
+/// usuario (o sin `scope`) siempre; de proyecto o local, si su
+/// `projectPath` es `raiz`.
+fn aplica(x: &Value, raiz: &Path) -> bool {
+    match x.get("scope").and_then(Value::as_str) {
+        None | Some("user") => true,
+        Some(_) => x
+            .get("projectPath")
+            .and_then(Value::as_str)
+            .is_some_and(|p| crate::git::misma_ruta(Path::new(p), raiz)),
+    }
+}
+
+/// Los plugins habilitados con sus skills y agentes.
+fn leer_plugins(dir_claude: &Path, raiz: &Path, ajustes: &Ajustes, a: &mut Acumulado) {
+    let ruta = ruta_instalados(dir_claude);
+    let v = match leer_instalados(dir_claude) {
+        Ok(None) => return,
+        Ok(Some(Ok(v))) => v,
+        Ok(Some(Err(_))) => {
+            a.ilegibles.push(Incidencia {
+                id: "plugins".to_owned(),
+                motivo: "json-invalido",
+                ruta: t(&ruta),
+            });
+            return;
+        }
+        Err(_) => {
+            a.ilegibles.push(Incidencia {
+                id: "plugins".to_owned(),
+                motivo: "no-se-lee",
+                ruta: t(&ruta),
+            });
+            return;
+        }
+    };
+    let Some(mapa) = v.get("plugins").and_then(Value::as_object) else {
+        a.ilegibles.push(Incidencia {
+            id: "plugins".to_owned(),
+            motivo: "formato-desconocido",
+            ruta: t(&ruta),
+        });
+        return;
+    };
+    for (clave, entradas) in mapa {
+        let Some(x) = entradas
+            .as_array()
+            .and_then(|l| l.iter().find(|x| aplica(x, raiz)))
+        else {
+            continue;
+        };
+        if ajustes.habilitados.get(clave) == Some(&false) {
+            continue;
+        }
+        let Some(install) = x.get("installPath").and_then(Value::as_str) else {
+            a.ilegibles.push(Incidencia {
+                id: clave.clone(),
+                motivo: "sin-install-path",
+                ruta: t(&ruta),
+            });
+            continue;
+        };
+        let dir = PathBuf::from(install);
+        if !dir.is_dir() {
+            a.ilegibles.push(Incidencia {
+                id: clave.clone(),
+                motivo: "install-path-inexistente",
+                ruta: install.to_owned(),
+            });
+            continue;
+        }
+        cargar_plugin(
+            &dir,
+            clave,
+            clave.split('@').next().unwrap_or(clave),
+            x.get("version").and_then(Value::as_str),
+            ajustes,
+            false,
+            a,
+        );
+    }
+}
+
+/// `.claude.json` de Claude Code: dentro de `CLAUDE_CONFIG_DIR` si se dio
+/// una carpeta propia, o junto a `~/.claude` (`~/.claude.json`).
+fn ruta_claude_json(dir_claude: &Path) -> PathBuf {
+    match (dir_claude.file_name(), dir_claude.parent()) {
+        (Some(n), Some(padre)) if n == ".claude" => padre.join(".claude.json"),
+        _ => dir_claude.join(".claude.json"),
+    }
+}
+
+/// El inventario de la maquina (`dir_claude`, la carpeta de Claude Code; sin
+/// ella, solo el proyecto y lo integrado) y del proyecto en `raiz`. Solo
+/// lee.
+pub fn leer(dir_claude: Option<&Path>, raiz: &Path) -> Lectura {
+    let mut a = Acumulado::default();
+    if let Some(d) = dir_claude {
+        let ajustes = ajustes(d, raiz, &mut a);
+        leer_plugins(d, raiz, &ajustes, &mut a);
+        // Despues de los instalados: un sincronizado cede ante otro origen
+        // con el mismo nombre (doc «Plugin loading reference», Name
+        // conflicts, punto 5).
+        leer_sincronizados(d, &ajustes, &mut a);
+        leer_carpetas(
+            &d.join("skills"),
+            &d.join("agents"),
+            "",
+            &Origen::Usuario,
+            &mut a,
+        );
+        if let Some(v) = json_de(&ruta_claude_json(d), &mut a, "mcp") {
+            servidores_mcp(&v, &mut a);
+        }
+    }
+    let proyecto = raiz.join(".claude");
+    leer_carpetas(
+        &proyecto.join("skills"),
+        &proyecto.join("agents"),
+        "",
+        &Origen::Proyecto,
+        &mut a,
+    );
+    if let Some(v) = json_de(&raiz.join(".mcp.json"), &mut a, "mcp") {
+        servidores_mcp(&v, &mut a);
+    }
+    for id in INTEGRADOS {
+        a.entradas.push(Entrada {
+            tipo: Tipo::Agente,
+            id: id.to_owned(),
+            origen: Origen::Integrado,
+        });
+    }
+    a.ilegibles.sort();
+    a.ilegibles.dedup();
+    a.avisos.sort();
+    a.avisos.dedup();
+    Lectura {
+        inventario: Inventario::nuevo(a.entradas, a.plugins),
+        ilegibles: a.ilegibles,
+        avisos: a.avisos,
+        mcp: a.mcp,
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    /// Carpeta temporal propia que se borra al terminar (nunca el `.claude`
+    /// real).
+    struct Tmp(PathBuf);
+
+    impl Tmp {
+        fn nueva(nombre: &str) -> Self {
+            let p = std::env::temp_dir()
+                .join("tc2-tests")
+                .join(format!("inventario-{nombre}-{}", std::process::id()));
+            let _ = std::fs::remove_dir_all(&p);
+            std::fs::create_dir_all(&p).unwrap_or_default();
+            Self(p)
+        }
+
+        fn escribir(&self, rel: &str, texto: &str) {
+            let p = self.0.join(rel);
+            if let Some(d) = p.parent() {
+                std::fs::create_dir_all(d).unwrap_or_default();
+            }
+            std::fs::write(p, texto).unwrap_or_default();
+        }
+    }
+
+    impl Drop for Tmp {
+        fn drop(&mut self) {
+            let _ = std::fs::remove_dir_all(&self.0);
+        }
+    }
+
+    fn skill(nombre: &str) -> String {
+        format!("---\nname: {nombre}\ndescription: x\n---\n# cuerpo\n")
+    }
+
+    fn lineas(l: &Lectura) -> Vec<String> {
+        l.inventario
+            .entradas()
+            .iter()
+            .map(|e| format!("{} {} {}", e.tipo.texto(), e.id, e.origen.texto()))
+            .collect()
+    }
+
+    #[test]
+    fn el_frontmatter_da_el_nombre() {
+        assert_eq!(
+            nombre_del_frontmatter("---\nname: alfa\n---\n").as_deref(),
+            Some("alfa")
+        );
+        assert_eq!(
+            nombre_del_frontmatter("\u{feff}---\r\ndescription: x\r\nname: \"beta\"\r\n---\r\n")
+                .as_deref(),
+            Some("beta")
+        );
+        assert_eq!(
+            nombre_del_frontmatter("---\nname: 'g'\n---").as_deref(),
+            Some("g")
+        );
+        for sin in [
+            "",
+            "# titulo\nname: x\n",
+            "---\ndescription: x\n---\nname: x\n",
+            "---\nname:\n---\n",
+            "---\n  name: sangrado\n---\n",
+        ] {
+            assert_eq!(nombre_del_frontmatter(sin), None, "{sin:?}");
+        }
+    }
+
+    /// C1 (parte de biblioteca) y D5a: lo de usuario, proyecto, el plugin
+    /// habilitado con su prefijo y lo integrado; sin el plugin apagado ni el
+    /// de otro proyecto; ordenado por (tipo, id); un `name:` distinto del de
+    /// su carpeta da el id del frontmatter con aviso.
+    #[test]
+    fn lee_lo_instalado_y_habilitado() {
+        let m = Tmp::nueva("c1");
+        let claude = m.0.join("claude");
+        let raiz = m.0.join("proyecto");
+        let bueno = m.0.join("cache").join("bueno");
+        let apagado = m.0.join("cache").join("apagado");
+        let ajeno = m.0.join("cache").join("ajeno");
+        m.escribir("claude/skills/zeta/SKILL.md", &skill("zeta"));
+        m.escribir("claude/skills/carpeta/SKILL.md", &skill("otro-nombre"));
+        m.escribir("claude/skills/sin-skill/LEEME.md", "x");
+        m.escribir("claude/agents/revisor.md", "---\nname: revisor\n---\n");
+        m.escribir("claude/agents/notas.txt", "x");
+        m.escribir(
+            "proyecto/.claude/skills/local/SKILL.md",
+            "# sin frontmatter\n",
+        );
+        m.escribir("cache/bueno/skills/rev/SKILL.md", &skill("rev"));
+        m.escribir("cache/bueno/agents/ag.md", "---\nname: ag\n---\n");
+        m.escribir(
+            "cache/bueno/.claude-plugin/plugin.json",
+            r#"{"name": "bueno", "version": "9.9.9"}"#,
+        );
+        m.escribir("cache/bueno/.mcp.json", r#"{"mcpServers": {"engram": {}}}"#);
+        m.escribir("cache/apagado/skills/x/SKILL.md", &skill("x"));
+        m.escribir("cache/ajeno/skills/y/SKILL.md", &skill("y"));
+        let instalados = serde_json::json!({
+            "version": 2,
+            "plugins": {
+                "bueno@mk": [{"scope": "user", "installPath": bueno, "version": "1.0.0"}],
+                "apagado@mk": [{"scope": "user", "installPath": apagado}],
+                "ajeno@mk": [{"scope": "project", "projectPath": m.0.join("otro"), "installPath": ajeno}]
+            }
+        });
+        m.escribir(
+            "claude/plugins/installed_plugins.json",
+            &instalados.to_string(),
+        );
+        m.escribir(
+            "claude/settings.json",
+            r#"{"enabledPlugins": {"bueno@mk": false, "apagado@mk": false}}"#,
+        );
+        // El proyecto vuelve a habilitar `bueno` (gana el mas cercano).
+        m.escribir(
+            "proyecto/.claude/settings.local.json",
+            r#"{"enabledPlugins": {"bueno@mk": true}}"#,
+        );
+        let l = leer(Some(&claude), &raiz);
+        assert_eq!(
+            lineas(&l),
+            [
+                "skill bueno:rev plugin",
+                "skill local proyecto",
+                "skill otro-nombre usuario",
+                "skill zeta usuario",
+                "agente bueno:ag plugin",
+                "agente general-purpose integrado",
+                "agente revisor usuario",
+            ]
+        );
+        assert_eq!(l.ilegibles, []);
+        assert_eq!(
+            l.avisos
+                .iter()
+                .map(|i| format!("{} {}", i.id, i.motivo))
+                .collect::<Vec<_>>(),
+            ["otro-nombre nombre-distinto"]
+        );
+        assert_eq!(
+            l.inventario.plugins(),
+            [Plugin {
+                nombre: "bueno".to_owned(),
+                clave: "bueno@mk".to_owned(),
+                version: Some("1.0.0".to_owned()),
+            }]
+        );
+        assert!(l.mcp.contains("engram"));
+        assert!(l.inventario.contiene(Tipo::Skill, "bueno:rev"));
+        assert!(!l.inventario.contiene(Tipo::Agente, "bueno:rev"));
+        assert_eq!(l.inventario.huella().len(), 12);
+        // Sin carpeta de Claude Code: solo el proyecto y lo integrado.
+        let solo = leer(None, &raiz);
+        assert_eq!(
+            lineas(&solo),
+            ["skill local proyecto", "agente general-purpose integrado"]
+        );
+    }
+
+    /// C8: `installed_plugins.json` corrupto o un `installPath` que no existe
+    /// quedan como ilegibles con su motivo; el plugin no cuenta.
+    #[test]
+    fn plugins_ilegibles_no_cuentan() {
+        let m = Tmp::nueva("c8");
+        let claude = m.0.join("claude");
+        m.escribir("claude/plugins/installed_plugins.json", "{ no es json");
+        let l = leer(Some(&claude), &m.0);
+        assert_eq!(
+            l.ilegibles
+                .iter()
+                .map(|i| format!("{} {}", i.id, i.motivo))
+                .collect::<Vec<_>>(),
+            ["plugins json-invalido"]
+        );
+        let instalados = serde_json::json!({"version": 2, "plugins": {
+            "taskcodetwo@mk": [{"scope": "user", "installPath": m.0.join("no-existe")}],
+            "sin@mk": [{"scope": "user"}]
+        }});
+        m.escribir(
+            "claude/plugins/installed_plugins.json",
+            &instalados.to_string(),
+        );
+        let l = leer(Some(&claude), &m.0);
+        assert_eq!(
+            l.ilegibles
+                .iter()
+                .map(|i| format!("{} {}", i.id, i.motivo))
+                .collect::<Vec<_>>(),
+            [
+                "sin@mk sin-install-path",
+                "taskcodetwo@mk install-path-inexistente"
+            ]
+        );
+        assert!(l.inventario.plugin("taskcodetwo").is_none());
+        m.escribir("claude/plugins/installed_plugins.json", "[1, 2]");
+        let l = leer(Some(&claude), &m.0);
+        assert_eq!(l.ilegibles[0].motivo, "formato-desconocido");
+    }
+
+    /// Plugins sincronizados desde claude.ai (`plugins/synced/<cuenta>/`):
+    /// cuenta la cuenta del `manifest.json` mas reciente; la carpeta `~g2` se
+    /// nombra por el `name` de su `plugin.json`; `<name>@synced: false` lo
+    /// apaga; cede ante un instalado del mismo nombre; `.trash` no cuenta; un
+    /// manifiesto corrupto o un plugin sin carpeta quedan como ilegibles; con
+    /// `syncClaudeAiPlugins: false` no se lee nada.
+    #[test]
+    fn sincronizados_desde_claude_ai() {
+        let m = Tmp::nueva("sincronizados");
+        let claude = m.0.join("claude");
+        let bueno = m.0.join("cache").join("bueno");
+        m.escribir("cache/bueno/skills/rev/SKILL.md", &skill("rev"));
+        m.escribir(
+            "claude/plugins/installed_plugins.json",
+            &serde_json::json!({"version": 2, "plugins": {
+                "bueno@mk": [{"scope": "user", "installPath": bueno}]
+            }})
+            .to_string(),
+        );
+        let s = "claude/plugins/synced";
+        let entrada = |n: &str| {
+            serde_json::json!({"pluginId": format!("id-{n}"), "name": n, "version": "1.0.0",
+                "marketplaceName": "mk-claude", "installationPreference": "available",
+                "marketplaceKind": "anthropic_catalog", "generation": 2})
+        };
+        m.escribir(
+            &format!("{s}/cuenta-vieja/manifest.json"),
+            &serde_json::json!({"lastUpdated": "2026-01-01T00:00:00Z", "plugins": [entrada("viejo")]})
+                .to_string(),
+        );
+        m.escribir(
+            &format!("{s}/cuenta-vieja/viejo~g2/skills/v/SKILL.md"),
+            &skill("v"),
+        );
+        m.escribir(
+            &format!("{s}/cuenta-nueva/manifest.json"),
+            &serde_json::json!({"lastUpdated": "2026-11-01T00:00:00Z", "plugins": [
+                entrada("datos"), entrada("apagado-s"), entrada("dup"), entrada("falta")
+            ]})
+            .to_string(),
+        );
+        m.escribir(
+            &format!("{s}/cuenta-nueva/datos~g2/.claude-plugin/plugin.json"),
+            r#"{"name": "data"}"#,
+        );
+        m.escribir(&format!("{s}/cuenta-nueva/datos~g2.meta.json"), "{}");
+        m.escribir(
+            &format!("{s}/cuenta-nueva/datos~g2/skills/analizar/SKILL.md"),
+            &skill("analizar"),
+        );
+        m.escribir(
+            &format!("{s}/cuenta-nueva/datos~g2/agents/ag-datos.md"),
+            "x",
+        );
+        m.escribir(
+            &format!("{s}/cuenta-nueva/apagado-s/skills/x/SKILL.md"),
+            &skill("x"),
+        );
+        m.escribir(
+            &format!("{s}/cuenta-nueva/dup~g2/.claude-plugin/plugin.json"),
+            r#"{"name": "bueno"}"#,
+        );
+        m.escribir(
+            &format!("{s}/cuenta-nueva/dup~g2/skills/otra/SKILL.md"),
+            &skill("otra"),
+        );
+        m.escribir(&format!("{s}/cuenta-rota/manifest.json"), "{ roto");
+        m.escribir(
+            "claude/plugins/.trash/basura/skills/z/SKILL.md",
+            &skill("z"),
+        );
+        m.escribir(
+            "claude/settings.json",
+            r#"{"enabledPlugins": {"apagado-s@synced": false}}"#,
+        );
+        let l = leer(Some(&claude), &m.0.join("p"));
+        assert_eq!(
+            lineas(&l),
+            [
+                "skill bueno:rev plugin",
+                "skill data:analizar plugin",
+                "agente data:ag-datos plugin",
+                "agente general-purpose integrado",
+            ]
+        );
+        assert_eq!(
+            l.inventario
+                .plugins()
+                .iter()
+                .map(|p| format!("{} {} {:?}", p.nombre, p.clave, p.version))
+                .collect::<Vec<_>>(),
+            ["bueno bueno@mk None", "data datos@synced Some(\"1.0.0\")"]
+        );
+        assert_eq!(
+            l.ilegibles
+                .iter()
+                .map(|i| format!("{} {}", i.id, i.motivo))
+                .collect::<Vec<_>>(),
+            [
+                "falta@synced sin-carpeta",
+                "synced/cuenta-rota json-invalido"
+            ]
+        );
+        m.escribir("claude/settings.json", r#"{"syncClaudeAiPlugins": false}"#);
+        let l = leer(Some(&claude), &m.0.join("p"));
+        assert!(l.inventario.plugin("data").is_none());
+        assert!(l.ilegibles.is_empty(), "{:?}", l.ilegibles);
+    }
+
+    /// C10 (parte de biblioteca): el mismo contenido creado en otro orden da
+    /// el mismo inventario y la misma huella; otro contenido, otra huella.
+    #[test]
+    fn determinista_con_otro_orden_de_creacion() {
+        let nombres = ["c", "a", "b"];
+        let mut huellas = Vec::new();
+        for (i, orden) in [nombres, [nombres[2], nombres[1], nombres[0]]]
+            .iter()
+            .enumerate()
+        {
+            let m = Tmp::nueva(&format!("c10-{i}"));
+            for n in orden {
+                m.escribir(&format!("claude/skills/{n}/SKILL.md"), &skill(n));
+                m.escribir(&format!("claude/agents/ag-{n}.md"), "x");
+            }
+            let l = leer(Some(&m.0.join("claude")), &m.0.join("p"));
+            huellas.push((l.inventario.huella(), l.inventario.json()));
+        }
+        assert_eq!(huellas[0], huellas[1]);
+        let otro = Inventario::nuevo(
+            vec![Entrada {
+                tipo: Tipo::Skill,
+                id: "a".to_owned(),
+                origen: Origen::Proyecto,
+            }],
+            Vec::new(),
+        );
+        assert_ne!(otro.huella(), huellas[0].0);
+    }
+
+    #[test]
+    fn la_cabecera_de_un_fichero_enorme_se_corta() {
+        let m = Tmp::nueva("enorme");
+        let mut texto = skill("grande");
+        texto.push_str(&"x".repeat(usize::try_from(MAX_CABECERA).unwrap_or(0) * 4));
+        m.escribir("s/grande/SKILL.md", &texto);
+        let c = cabecera(&m.0.join("s/grande/SKILL.md")).unwrap_or_default();
+        assert_eq!(c.len(), usize::try_from(MAX_CABECERA).unwrap_or(0));
+        assert_eq!(nombre_del_frontmatter(&c).as_deref(), Some("grande"));
+    }
+}
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 130a393..e856f6c 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -38,7 +38,9 @@ pub mod identidad;
 pub mod importar;
 pub mod init;
 pub mod instalacion;
+pub mod inventario;
 pub mod plataforma;
+pub mod recursos;
 pub mod roadmap;
 pub mod siguiente;
 pub mod situacion;
@@ -361,10 +363,20 @@ fn orden_doctor(args: &[String], json: bool, idioma: Idioma) -> Salida {
     };
     let g = git::Git::nuevo();
     let conectar = tablero::conectar_produccion(&g, &dir);
+    // La carpeta de Claude Code (`CLAUDE_CONFIG_DIR` o `~/.claude`), solo para
+    // leer el inventario (F3-9).
+    let claude = instalacion::Entorno::de_variables(
+        instalacion::Sistema::actual(),
+        std::path::PathBuf::new(),
+        std::process::id(),
+        |k| std::env::var(k).ok(),
+    )
+    .dir_claude();
     let e = doctor::Entorno {
         dir: &dir,
         git: &g,
         conectar: &conectar,
+        claude: claude.as_deref(),
     };
     doctor::orden(args, json, idioma, &e)
 }
diff --git a/crates/tc2/src/recursos.rs b/crates/tc2/src/recursos.rs
new file mode 100644
index 0000000..cce0103
--- /dev/null
+++ b/crates/tc2/src/recursos.rs
@@ -0,0 +1,490 @@
+//! Manifiesto de recursos (ADR F1-14, punto 8; F1-15 decision 11): una
+//! funcion pura, [`evaluar_recursos`], que cruza el inventario de F3-9 con lo
+//! que se recomienda y con el suelo del binario, y dice de cada recurso su
+//! nivel, como salio y que hacer.
+//!
+//! | Nivel | Que entra aqui |
+//! |---|---|
+//! | obligatorio | el plugin `taskcodetwo` (sus skills de fase llegan con el plugin; mientras no este, una sola `falla`, D3a) |
+//! | necesario | lo que nombran `recursos:` de la configuracion y `roles.*` de las tareas no terminadas (D1a) |
+//! | conveniente | engram, context7 y rtk: avisa y sigue |
+//! | estorba | la 0.16 (`taskcode-plugin`) habilitada a la vez: avisa |
+//!
+//! No depende de `doctor`: `init`, `start` y `seleccionar` lo usan con su
+//! propio adaptador. Sin E/S: lo que hay en la maquina entra ya leido
+//! (inventario, servidores MCP y ejecutables encontrados en el PATH).
+
+use std::collections::BTreeSet;
+
+use crate::inventario::{Inventario, Tipo};
+
+/// Nombre del plugin de la 2.0 (F1-12).
+pub const PLUGIN: &str = crate::huella::NOMBRE;
+
+/// Nombre del plugin de la 0.16.
+pub const PLUGIN_016: &str = "taskcode-plugin";
+
+/// Orden que instala el plugin de la 2.0.
+pub const ARREGLO_PLUGIN: &str = "claude plugin install taskcodetwo";
+
+/// Motivo de lo que no esta instalado.
+pub const NO_INSTALADO: &str = "no-instalado";
+
+/// Motivo de lo que no se puede decidir porque el inventario no se leyo
+/// entero (C8: nunca `ok`).
+pub const INCOMPLETO: &str = "inventario-incompleto";
+
+/// Nivel de un recurso (F1-14, punto 8).
+#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
+pub enum Nivel {
+    /// Para con el arreglo en `init`, `doctor` y `start`.
+    Obligatorio,
+    /// Lo declara el proyecto o la tarea.
+    Necesario,
+    /// Avisa y sigue.
+    Conveniente,
+    /// Interfiere: avisa.
+    Estorba,
+}
+
+impl Nivel {
+    /// Valor en la salida.
+    pub fn texto(self) -> &'static str {
+        match self {
+            Self::Obligatorio => "obligatorio",
+            Self::Necesario => "necesario",
+            Self::Conveniente => "conveniente",
+            Self::Estorba => "estorba",
+        }
+    }
+}
+
+/// Como salio la comprobacion de un recurso.
+#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
+pub enum Estado {
+    /// Falta algo obligatorio o necesario.
+    Falla,
+    /// Falta algo conveniente o hay algo que estorba.
+    Aviso,
+    /// No se pudo decidir.
+    NoComprobado,
+    /// Esta.
+    Ok,
+}
+
+/// Que es un recurso recomendado.
+#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
+pub enum TipoRecurso {
+    /// Skill del inventario.
+    Skill,
+    /// Agente del inventario.
+    Agente,
+    /// Ejecutable en el PATH.
+    Ejecutable,
+}
+
+impl TipoRecurso {
+    /// Valor en la salida.
+    pub fn texto(self) -> &'static str {
+        match self {
+            Self::Skill => "skill",
+            Self::Agente => "agente",
+            Self::Ejecutable => "ejecutable",
+        }
+    }
+
+    /// El tipo de `recursos[].tipo` de la configuracion.
+    pub fn de_texto(t: &str) -> Option<Self> {
+        match t {
+            "skill" => Some(Self::Skill),
+            "agente" => Some(Self::Agente),
+            "ejecutable" => Some(Self::Ejecutable),
+            _ => None,
+        }
+    }
+}
+
+/// Quien recomienda un recurso.
+#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
+pub enum Fuente {
+    /// `recursos:` de la configuracion del proyecto.
+    Config,
+    /// `roles.*` de una tarea no terminada (su ID).
+    Tarea(String),
+}
+
+/// Un recurso que alguien recomienda.
+#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
+pub struct Recomendado {
+    /// Id (el del inventario o el nombre del ejecutable).
+    pub id: String,
+    /// Que es.
+    pub tipo: TipoRecurso,
+    /// Quien lo recomienda.
+    pub fuente: Fuente,
+}
+
+/// Lo que se recomienda y lo que hay en la maquina fuera del inventario.
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct Manifiesto {
+    /// Lo recomendado, en cualquier orden (se ordena y no se repite).
+    pub recomendados: Vec<Recomendado>,
+    /// Ejecutables encontrados en el PATH (de los que se buscaron).
+    pub ejecutables: BTreeSet<String>,
+    /// Servidores MCP configurados.
+    pub mcp: BTreeSet<String>,
+    /// Si el inventario se leyo entero. Si no, lo que no aparece queda
+    /// `no-comprobado` (C8).
+    pub completo: bool,
+}
+
+/// Para que se evalua.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum Momento {
+    /// `tc2 init`: el suelo y lo de la configuracion.
+    Init,
+    /// `tc2 doctor --recursos`: todo.
+    Doctor,
+    /// La entrada de `start` de una tarea: lo obligatorio y lo de esa tarea.
+    Start(String),
+}
+
+/// Un recurso evaluado.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Recurso {
+    /// Id.
+    pub id: String,
+    /// `skill`, `agente`, `ejecutable`, `plugin` o `mcp`.
+    pub tipo: &'static str,
+    /// Nivel.
+    pub nivel: Nivel,
+    /// Como salio.
+    pub estado: Estado,
+    /// Por que (sin espacios): `no-instalado`, `inventario-incompleto`,
+    /// `taskcode-016-habilitado`...; `None` si esta.
+    pub comprobacion: Option<&'static str>,
+    /// Tarea que lo recomienda, si es de una tarea.
+    pub tarea: Option<String>,
+    /// Version, si se sabe (plugins).
+    pub version: Option<String>,
+    /// Que hacer (sin espacios), si algo no esta bien.
+    pub arreglo: Option<String>,
+}
+
+impl Recurso {
+    fn nuevo(id: impl Into<String>, tipo: &'static str, nivel: Nivel, estado: Estado) -> Self {
+        Self {
+            id: id.into(),
+            tipo,
+            nivel,
+            estado,
+            comprobacion: None,
+            tarea: None,
+            version: None,
+            arreglo: None,
+        }
+    }
+
+    fn por(mut self, c: &'static str) -> Self {
+        self.comprobacion = Some(c);
+        self
+    }
+
+    fn arreglo(mut self, a: impl Into<String>) -> Self {
+        self.arreglo = Some(a.into());
+        self
+    }
+}
+
+/// El resultado de [`evaluar_recursos`], en orden fijo: obligatorio,
+/// necesario (por id y fuente), conveniente y estorba.
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct InformeRecursos {
+    /// Los recursos evaluados.
+    pub recursos: Vec<Recurso>,
+}
+
+impl InformeRecursos {
+    /// Cuantos `no-instalado` en `falla` hay.
+    pub fn no_instalados(&self) -> usize {
+        self.recursos
+            .iter()
+            .filter(|r| r.estado == Estado::Falla && r.comprobacion == Some(NO_INSTALADO))
+            .count()
+    }
+}
+
+/// El arreglo de un recomendado que falta: el plugin si el id lleva su
+/// prefijo; si no, instalarlo o quitarlo de quien lo nombra.
+fn arreglo_de(r: &Recomendado) -> String {
+    if r.tipo != TipoRecurso::Ejecutable
+        && let Some((plugin, _)) = r.id.split_once(':')
+        && !plugin.is_empty()
+    {
+        return format!("claude plugin install {plugin}");
+    }
+    match &r.fuente {
+        Fuente::Config => format!("instalar-{}-o-quitarla-de-recursos", r.tipo.texto()),
+        Fuente::Tarea(t) => format!("instalar-{}-o-cambiar-roles-de-{t}", r.tipo.texto()),
+    }
+}
+
+/// Si un servidor o plugin con `nombre` esta configurado (sin mayusculas;
+/// deteccion aproximada).
+fn hay_mcp(inv: &Inventario, m: &Manifiesto, nombre: &str) -> bool {
+    m.mcp
+        .iter()
+        .chain(inv.plugins().iter().map(|p| &p.nombre))
+        .any(|x| x.to_ascii_lowercase().contains(nombre))
+}
+
+/// Evalua el manifiesto contra el inventario (funcion pura, determinista).
+pub fn evaluar_recursos(inv: &Inventario, m: &Manifiesto, momento: &Momento) -> InformeRecursos {
+    let mut v = Vec::new();
+
+    // Obligatorio: el plugin de la 2.0 (D3a: una sola falla mientras no
+    // este; sus skills de fase llegan con el).
+    match inv.plugin(PLUGIN) {
+        Some(p) => {
+            let mut r = Recurso::nuevo(PLUGIN, "plugin", Nivel::Obligatorio, Estado::Ok);
+            r.version.clone_from(&p.version);
+            v.push(r);
+        }
+        None if !m.completo => v.push(
+            Recurso::nuevo(PLUGIN, "plugin", Nivel::Obligatorio, Estado::NoComprobado)
+                .por(INCOMPLETO),
+        ),
+        None => v.push(
+            Recurso::nuevo(PLUGIN, "plugin", Nivel::Obligatorio, Estado::Falla)
+                .por(NO_INSTALADO)
+                .arreglo(ARREGLO_PLUGIN),
+        ),
+    }
+
+    // Necesario: lo recomendado, ordenado y sin repetir.
+    let mut rec = m.recomendados.clone();
+    rec.sort();
+    rec.dedup();
+    for r in rec.iter().filter(|r| match (momento, &r.fuente) {
+        (Momento::Doctor, _) | (Momento::Init, Fuente::Config) => true,
+        (Momento::Start(t), Fuente::Tarea(x)) => t == x,
+        _ => false,
+    }) {
+        let esta = match r.tipo {
+            TipoRecurso::Skill => inv.contiene(Tipo::Skill, &r.id),
+            TipoRecurso::Agente => inv.contiene(Tipo::Agente, &r.id),
+            TipoRecurso::Ejecutable => m.ejecutables.contains(&r.id),
+        };
+        let mut x = if esta {
+            Recurso::nuevo(r.id.clone(), r.tipo.texto(), Nivel::Necesario, Estado::Ok)
+        } else if r.tipo != TipoRecurso::Ejecutable && !m.completo {
+            Recurso::nuevo(
+                r.id.clone(),
+                r.tipo.texto(),
+                Nivel::Necesario,
+                Estado::NoComprobado,
+            )
+            .por(INCOMPLETO)
+        } else {
+            Recurso::nuevo(
+                r.id.clone(),
+                r.tipo.texto(),
+                Nivel::Necesario,
+                Estado::Falla,
+            )
+            .por(NO_INSTALADO)
+            .arreglo(arreglo_de(r))
+        };
+        if let Fuente::Tarea(t) = &r.fuente {
+            x.tarea = Some(t.clone());
+        }
+        v.push(x);
+    }
+    if matches!(momento, Momento::Start(_)) {
+        return InformeRecursos { recursos: v };
+    }
+
+    // Conveniente: avisa y sigue (engram no es obligatorio).
+    for (id, tipo, esta) in [
+        ("engram", "mcp", hay_mcp(inv, m, "engram")),
+        ("context7", "mcp", hay_mcp(inv, m, "context7")),
+        ("rtk", "ejecutable", m.ejecutables.contains("rtk")),
+    ] {
+        v.push(if esta {
+            Recurso::nuevo(id, tipo, Nivel::Conveniente, Estado::Ok)
+        } else {
+            Recurso::nuevo(id, tipo, Nivel::Conveniente, Estado::Aviso)
+                .por(NO_INSTALADO)
+                .arreglo(format!("instalar-{id}"))
+        });
+    }
+
+    // Estorba: la 0.16 habilitada a la vez.
+    if let Some(p) = inv.plugin(PLUGIN_016) {
+        let mut r = Recurso::nuevo(PLUGIN_016, "plugin", Nivel::Estorba, Estado::Aviso)
+            .por("taskcode-016-habilitado")
+            .arreglo(format!("claude plugin disable {}", p.clave));
+        r.version.clone_from(&p.version);
+        v.push(r);
+    }
+    InformeRecursos { recursos: v }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::inventario::{Entrada, Origen, Plugin};
+
+    fn plugin(nombre: &str) -> Plugin {
+        Plugin {
+            nombre: nombre.to_owned(),
+            clave: format!("{nombre}@mk"),
+            version: Some("1.0.0".to_owned()),
+        }
+    }
+
+    fn inventario(plugins: &[&str], skills: &[&str]) -> Inventario {
+        Inventario::nuevo(
+            skills
+                .iter()
+                .map(|s| Entrada {
+                    tipo: Tipo::Skill,
+                    id: (*s).to_owned(),
+                    origen: Origen::Usuario,
+                })
+                .collect(),
+            plugins.iter().map(|p| plugin(p)).collect(),
+        )
+    }
+
+    fn manifiesto(recomendados: Vec<Recomendado>) -> Manifiesto {
+        Manifiesto {
+            recomendados,
+            completo: true,
+            ..Manifiesto::default()
+        }
+    }
+
+    fn lineas(i: &InformeRecursos) -> Vec<String> {
+        i.recursos
+            .iter()
+            .map(|r| {
+                format!(
+                    "{} {} {} {:?} {} {}",
+                    r.id,
+                    r.tipo,
+                    r.nivel.texto(),
+                    r.estado,
+                    r.comprobacion.unwrap_or("-"),
+                    r.arreglo.as_deref().unwrap_or("-")
+                )
+            })
+            .collect()
+    }
+
+    /// C9: sin el plugin `taskcodetwo`, una sola `falla` con el arreglo de
+    /// instalarlo; con la 0.16, `aviso` de nivel estorba; sin engram,
+    /// context7 ni rtk, `aviso` de nivel conveniente (no son fallos).
+    #[test]
+    fn suelo_del_manifiesto() {
+        let i = evaluar_recursos(
+            &inventario(&[PLUGIN_016], &[]),
+            &manifiesto(Vec::new()),
+            &Momento::Doctor,
+        );
+        assert_eq!(
+            lineas(&i),
+            [
+                "taskcodetwo plugin obligatorio Falla no-instalado claude plugin install taskcodetwo",
+                "engram mcp conveniente Aviso no-instalado instalar-engram",
+                "context7 mcp conveniente Aviso no-instalado instalar-context7",
+                "rtk ejecutable conveniente Aviso no-instalado instalar-rtk",
+                "taskcode-plugin plugin estorba Aviso taskcode-016-habilitado claude plugin disable taskcode-plugin@mk",
+            ]
+        );
+        assert_eq!(
+            i.recursos
+                .iter()
+                .filter(|r| r.estado == Estado::Falla)
+                .count(),
+            1
+        );
+        assert_eq!(i.no_instalados(), 1);
+        // Con todo: el plugin, un MCP por nombre, otro por plugin y rtk.
+        let m = Manifiesto {
+            ejecutables: BTreeSet::from(["rtk".to_owned()]),
+            mcp: BTreeSet::from(["Engram-Memoria".to_owned()]),
+            ..manifiesto(Vec::new())
+        };
+        let i = evaluar_recursos(
+            &inventario(&[PLUGIN, "context7"], &[]),
+            &m,
+            &Momento::Doctor,
+        );
+        assert!(i.recursos.iter().all(|r| r.estado == Estado::Ok), "{i:?}");
+        assert_eq!(i.recursos[0].version.as_deref(), Some("1.0.0"));
+    }
+
+    /// C2 y C3 (parte pura): lo recomendado que no esta es `falla
+    /// no-instalado` con arreglo; si esta, `ok`; con el inventario
+    /// incompleto, `no-comprobado` (C8); `start` solo mira su tarea.
+    #[test]
+    fn lo_recomendado_se_cruza_con_el_inventario() {
+        let rec = vec![
+            Recomendado {
+                id: "fantasma".to_owned(),
+                tipo: TipoRecurso::Skill,
+                fuente: Fuente::Config,
+            },
+            Recomendado {
+                id: "esta".to_owned(),
+                tipo: TipoRecurso::Skill,
+                fuente: Fuente::Tarea("TASK-002".to_owned()),
+            },
+            Recomendado {
+                id: "otro:x".to_owned(),
+                tipo: TipoRecurso::Agente,
+                fuente: Fuente::Tarea("TASK-001".to_owned()),
+            },
+            Recomendado {
+                id: "fantasma".to_owned(),
+                tipo: TipoRecurso::Skill,
+                fuente: Fuente::Config,
+            },
+        ];
+        let inv = inventario(&[PLUGIN], &["esta"]);
+        let i = evaluar_recursos(&inv, &manifiesto(rec.clone()), &Momento::Doctor);
+        assert_eq!(
+            lineas(&i)[1..4],
+            [
+                "esta skill necesario Ok - -",
+                "fantasma skill necesario Falla no-instalado instalar-skill-o-quitarla-de-recursos",
+                "otro:x agente necesario Falla no-instalado claude plugin install otro",
+            ]
+        );
+        assert_eq!(i.recursos[3].tarea.as_deref(), Some("TASK-001"));
+        assert_eq!(i.no_instalados(), 2);
+        let m = Manifiesto {
+            completo: false,
+            ..manifiesto(rec.clone())
+        };
+        let i = evaluar_recursos(&inv, &m, &Momento::Doctor);
+        assert_eq!(
+            i.recursos
+                .iter()
+                .filter(|r| r.comprobacion == Some(INCOMPLETO))
+                .count(),
+            2
+        );
+        assert!(i.recursos.iter().all(|r| r.estado != Estado::Falla));
+        let i = evaluar_recursos(
+            &inv,
+            &manifiesto(rec),
+            &Momento::Start("TASK-001".to_owned()),
+        );
+        assert_eq!(
+            i.recursos.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(),
+            ["taskcodetwo", "otro:x"]
+        );
+    }
+}
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-080/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-080/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-080/tarea.md                                    | 36 ++++++++++++++++++++++++++++++++++--
 3 files changed, 34 insertions(+), 2 deletions(-)
````

## Tests del diff (3 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/doctor.rs" ":(literal)crates/tc2/tests/frescura.rs" ":(literal)crates/tc2/tests/recursos.rs"
````

````
crates/tc2/tests/doctor.rs   |  13 +++-
 crates/tc2/tests/frescura.rs |   1 +
 crates/tc2/tests/recursos.rs | 666 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 3 files changed, 679 insertions(+), 1 deletion(-)
````
