# Peticion de revision — TASK-044 (ronda 1)

- Tarea: TASK-044 — F5-4 Informe de verificacion en JSON para el revisor IA
- Rama revisada: feature/task-044-f5-4-informe-de-verificacion-en-json-par
- Rama base: develop
- Commit revisado (HEAD): cf29ab03914858f9e85525bb157b39be5715d0ca
- Fecha: 2026-11-19
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-044 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-044/tarea.md
- tareas/03-en-revision/TASK-044/planificacion/
- crates/tc2/src/lib.rs
- crates/tc2/src/verificacion/informe.rs
- crates/tc2/src/verificacion/mod.rs
- xtask/src/main.rs
- xtask/src/workflows.rs

## Commits a revisar (git log develop..HEAD)

````
cf29ab0 fix(TASK-044): mutaciones del informe con el fixture en CRLF
c56abd8 feat(TASK-044): informe de verificacion en JSON para el revisor IA (F5-4)
a81a8fe chore(TASK-044): tarea en curso
````

## Diff de tu dominio (5 fichero(s) de 11; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 8ded35b..9a4ada2 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -55,6 +55,7 @@ pub mod solape;
 pub mod tablero;
 pub mod tarea;
 pub mod toma;
+pub mod verificacion;
 pub mod version;
 mod yaml;
 
diff --git a/crates/tc2/src/verificacion/informe.rs b/crates/tc2/src/verificacion/informe.rs
new file mode 100644
index 0000000..52b8c24
--- /dev/null
+++ b/crates/tc2/src/verificacion/informe.rs
@@ -0,0 +1,311 @@
+//! Informe de verificacion en JSON para el revisor IA (F5-4).
+//!
+//! El comando de verificacion del proyecto lo escribe en la ruta de
+//! `TC2_INFORME` (tambien en rojo) y la CI solo lo transporta: un artefacto
+//! `informe-verificacion-<perfil>` por perfil y la linea `informe=` en el
+//! resumen del check `verificacion` (F1-7, decision 1). El contrato es
+//! `crates/tc2/esquemas/informe-verificacion.schema.json`, version 1; aqui
+//! estan su lector ([`leer`]) y el texto con el que el revisor lo cita
+//! ([`seccion_revisor`]).
+//!
+//! El JSON lo escribe codigo del PR: es detalle para citar y nunca decide. El
+//! hecho autoritativo para no repetir la suite es el check `verificacion` de
+//! la App `github-actions` (`app_id` 15368) en el head del PR.
+
+use serde::Deserialize;
+
+/// Unica version del contrato que se lee.
+pub const VERSION: u32 = 1;
+
+/// Perfiles de F1-7 (decision 4).
+const PERFILES: [&str; 4] = ["rapida", "impacto", "completa", "pesada"];
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
+#[serde(rename_all = "lowercase")]
+pub enum Resultado {
+    Verde,
+    Rojo,
+}
+
+impl Resultado {
+    fn texto(self) -> &'static str {
+        match self {
+            Resultado::Verde => "verde",
+            Resultado::Rojo => "rojo",
+        }
+    }
+}
+
+#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct Paso {
+    pub nombre: String,
+    pub resultado: Resultado,
+    pub segundos: u64,
+}
+
+/// Recuento de tests, opcional (F1-11, practica 8).
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct Tests {
+    pub total: u64,
+    pub fallidos: u64,
+}
+
+#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct Informe {
+    pub version: u32,
+    pub perfil: String,
+    pub resultado: Resultado,
+    /// Obligatorio aunque sea `null`: con `deserialize_with` serde no lo da
+    /// por ausente.
+    #[serde(deserialize_with = "Option::deserialize")]
+    pub commit: Option<String>,
+    pub inicio: String,
+    pub fin: String,
+    pub pasos: Vec<Paso>,
+    #[serde(default)]
+    pub tests: Option<Tests>,
+}
+
+/// Lee y valida un informe v1. Lo que el esquema no admite es error: clave
+/// desconocida, otra version, perfil, commit, fecha o nombre de paso fuera de
+/// forma, `verde` con algun paso en rojo o mas fallidos que tests.
+pub fn leer(texto: &str) -> Result<Informe, String> {
+    let informe: Informe =
+        serde_json::from_str(texto).map_err(|e| format!("informe de verificacion: {e}"))?;
+    if informe.version != VERSION {
+        return Err(format!(
+            "informe de verificacion: version {} no admitida (se lee la {VERSION})",
+            informe.version
+        ));
+    }
+    if !PERFILES.contains(&informe.perfil.as_str()) {
+        return Err(format!(
+            "informe de verificacion: perfil desconocido «{}»",
+            informe.perfil
+        ));
+    }
+    if let Some(commit) = &informe.commit {
+        let hex = |c: char| c.is_ascii_digit() || ('a'..='f').contains(&c);
+        if commit.len() != 40 || !commit.chars().all(hex) {
+            return Err(format!(
+                "informe de verificacion: commit no valido «{commit}»"
+            ));
+        }
+    }
+    for (campo, fecha) in [("inicio", &informe.inicio), ("fin", &informe.fin)] {
+        if !es_fecha_utc(fecha) {
+            return Err(format!(
+                "informe de verificacion: {campo} no es AAAA-MM-DDTHH:MM:SSZ: «{fecha}»"
+            ));
+        }
+    }
+    for paso in &informe.pasos {
+        if !es_nombre_de_paso(&paso.nombre) {
+            return Err(format!(
+                "informe de verificacion: nombre de paso no valido «{}»",
+                paso.nombre
+            ));
+        }
+    }
+    if informe.resultado == Resultado::Verde
+        && informe.pasos.iter().any(|p| p.resultado == Resultado::Rojo)
+    {
+        return Err("informe de verificacion: verde con un paso en rojo".to_owned());
+    }
+    if let Some(t) = informe.tests
+        && t.fallidos > t.total
+    {
+        return Err(format!(
+            "informe de verificacion: {} fallidos de {} tests",
+            t.fallidos, t.total
+        ));
+    }
+    Ok(informe)
+}
+
+/// `AAAA-MM-DDTHH:MM:SSZ`, solo la forma.
+fn es_fecha_utc(texto: &str) -> bool {
+    let forma = b"0000-00-00T00:00:00Z";
+    texto.len() == forma.len()
+        && texto.bytes().zip(forma).all(|(c, f)| match f {
+            b'0' => c.is_ascii_digit(),
+            otro => c == *otro,
+        })
+}
+
+/// `^[a-z0-9][a-z0-9._-]{0,63}$`: el nombre llega al texto del revisor, asi
+/// que no lleva espacios, `|` ni saltos de linea.
+fn es_nombre_de_paso(nombre: &str) -> bool {
+    let valido = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit();
+    let mut chars = nombre.chars();
+    chars.next().is_some_and(valido)
+        && nombre.len() <= 64
+        && chars.all(|c| valido(c) || matches!(c, '.' | '_' | '-'))
+}
+
+/// Seccion en Markdown con la que el revisor IA (F6-3) cita el informe del
+/// artefacto `artefacto`: perfil, resultado, commit, pasos, recuento y la
+/// regla de no relanzar lo que ya paso en verde.
+pub fn seccion_revisor(informe: &Informe, artefacto: &str) -> String {
+    let mut s = String::from("## Informe de verificacion (F5-4)\n\n");
+    s.push_str(&format!(
+        "Artefacto `{artefacto}`: perfil `{}`, resultado `{}`, commit `{}`, de {} a {}.\n\n",
+        informe.perfil,
+        informe.resultado.texto(),
+        informe.commit.as_deref().unwrap_or("desconocido"),
+        informe.inicio,
+        informe.fin
+    ));
+    s.push_str("| paso | resultado | segundos |\n|---|---|---|\n");
+    for p in &informe.pasos {
+        s.push_str(&format!(
+            "| {} | {} | {} |\n",
+            p.nombre,
+            p.resultado.texto(),
+            p.segundos
+        ));
+    }
+    if let Some(t) = informe.tests {
+        s.push_str(&format!(
+            "\nTests: {} en total, {} fallidos.\n",
+            t.total, t.fallidos
+        ));
+    }
+    s.push_str(
+        "\nRegla: el hecho autoritativo es el check `verificacion` de la App \
+         `github-actions` (app_id 15368) en el head del PR. Si ese check esta en \
+         verde para este commit, no relances los pasos que figuran en verde: \
+         citalos desde este informe. El informe lo escribe codigo del PR: es \
+         detalle para citar y nunca decide; si contradice al check, manda el \
+         check.\n",
+    );
+    s
+}
+
+#[cfg(test)]
+#[allow(clippy::panic)]
+mod tests {
+    use super::*;
+
+    const VERDE: &str = include_str!("../../tests/fixtures/informe-verificacion/xtask.json");
+    const ROJO: &str = include_str!("../../tests/fixtures/informe-verificacion/xtask-rojo.json");
+    const ESQUEMA: &str = include_str!("../../esquemas/informe-verificacion.schema.json");
+
+    fn leido(texto: &str) -> Informe {
+        leer(texto).unwrap_or_else(|e| panic!("{e}"))
+    }
+
+    #[test]
+    fn el_fixture_se_lee_y_el_revisor_lo_cita() {
+        let verde = leido(VERDE);
+        assert_eq!(verde.perfil, "completa");
+        assert_eq!(verde.pasos.len(), 4);
+        assert_eq!(verde.tests, None);
+        let seccion = seccion_revisor(&verde, "informe-verificacion-completa");
+        for buscado in [
+            "Artefacto `informe-verificacion-completa`",
+            "perfil `completa`, resultado `verde`",
+            "commit `0123456789abcdef0123456789abcdef01234567`",
+            "| fmt | verde | 2 |",
+            "| reproducible | verde | 266 |",
+            "check `verificacion`",
+            "app_id 15368",
+            "no relances los pasos que figuran en verde",
+            "nunca decide",
+        ] {
+            assert!(
+                seccion.contains(buscado),
+                "falta «{buscado}» en:\n{seccion}"
+            );
+        }
+        let rojo = leido(ROJO);
+        assert_eq!(rojo.commit, None);
+        let seccion = seccion_revisor(&rojo, "informe-verificacion-rapida");
+        assert!(seccion.contains("commit `desconocido`"), "{seccion}");
+        assert!(seccion.contains("| clippy | rojo | 42 |"), "{seccion}");
+        let con_tests = VERDE.replacen(
+            "  \"pasos\"",
+            "  \"tests\": { \"total\": 10, \"fallidos\": 0 },\n  \"pasos\"",
+            1,
+        );
+        assert!(
+            seccion_revisor(&leido(&con_tests), "a").contains("Tests: 10 en total, 0 fallidos.")
+        );
+        // El esquema declara las mismas claves que el lector.
+        let esquema: serde_json::Value =
+            serde_json::from_str(ESQUEMA).unwrap_or_else(|e| panic!("{e}"));
+        let claves: Vec<&str> = esquema["properties"]
+            .as_object()
+            .map(|m| m.keys().map(String::as_str).collect())
+            .unwrap_or_default();
+        let mut claves = claves;
+        claves.sort_unstable();
+        assert_eq!(
+            claves,
+            [
+                "commit",
+                "fin",
+                "inicio",
+                "pasos",
+                "perfil",
+                "resultado",
+                "tests",
+                "version"
+            ]
+        );
+        assert_eq!(esquema["properties"]["version"]["const"], VERSION);
+    }
+
+    #[test]
+    fn lo_que_el_contrato_no_admite_es_error() {
+        let casos = [
+            ("\"version\": 1", "\"version\": 2", "version 2"),
+            (
+                "\"version\": 1,",
+                "\"version\": 1,\n  \"extra\": 0,",
+                "unknown field",
+            ),
+            (
+                "\"segundos\": 2 }",
+                "\"segundos\": 2, \"x\": 1 }",
+                "unknown field",
+            ),
+            (
+                "  \"commit\": \"0123456789abcdef0123456789abcdef01234567\",\n",
+                "",
+                "commit",
+            ),
+            ("\"completa\"", "\"otra\"", "perfil desconocido"),
+            (
+                "\"0123456789abcdef",
+                "\"0123456789ABCDEF",
+                "commit no valido",
+            ),
+            ("\"2026-11-19T17:30:57Z\"", "\"2026-11-19 17:30\"", "inicio"),
+            ("\"fmt\"", "\"fmt | x\"", "nombre de paso"),
+            (
+                "\"verde\", \"segundos\": 95",
+                "\"rojo\", \"segundos\": 95",
+                "verde con un paso en rojo",
+            ),
+            ("\"segundos\": 2 }", "\"segundos\": -2 }", "invalid value"),
+            (
+                "  \"pasos\"",
+                "  \"tests\": { \"total\": 1, \"fallidos\": 2 },\n  \"pasos\"",
+                "2 fallidos de 1",
+            ),
+        ];
+        // Un clon con core.autocrlf=true da el fixture con CRLF.
+        let verde = VERDE.replace("\r\n", "\n");
+        for (original, mutado, esperado) in casos {
+            assert!(verde.contains(original), "el caso no casa: {original}");
+            match leer(&verde.replacen(original, mutado, 1)) {
+                Ok(i) => panic!("«{mutado}» se acepto: {i:?}"),
+                Err(e) => assert!(e.contains(esperado), "«{esperado}» no esta en «{e}»"),
+            }
+        }
+    }
+}
diff --git a/crates/tc2/src/verificacion/mod.rs b/crates/tc2/src/verificacion/mod.rs
new file mode 100644
index 0000000..d588027
--- /dev/null
+++ b/crates/tc2/src/verificacion/mod.rs
@@ -0,0 +1,7 @@
+//! Verificacion del proyecto vista desde tc2 (F5).
+//!
+//! - [`informe`]: el informe JSON de verificacion (F5-4) que escribe el
+//!   comando del proyecto en `TC2_INFORME` y la CI sube como artefacto; su
+//!   lector y la seccion con la que el revisor IA lo cita.
+
+pub mod informe;
diff --git a/xtask/src/main.rs b/xtask/src/main.rs
index 30550c7..0f8e7fc 100644
--- a/xtask/src/main.rs
+++ b/xtask/src/main.rs
@@ -6,6 +6,9 @@
 //! - `completa`: lo anterior y la comprobacion de build reproducible: copia el
 //!   arbol a dos rutas distintas, compila `tc2` en release en limpio en cada
 //!   una y exige el mismo sha256 (ADR F1-14). La CI solo la corre en Windows.
+//! - Con `TC2_INFORME=<ruta>` escribe ahi el informe JSON de F5-4 (pasos,
+//!   resultado y segundos), tambien en rojo; contrato en
+//!   `crates/tc2/esquemas/informe-verificacion.schema.json`.
 //!
 //! Distribucion (F2-10), las mismas ordenes en local y en `release.yml`:
 //!
@@ -250,30 +253,200 @@ fn escribir_sumas(dir: &Path) -> Result<(), String> {
     Ok(())
 }
 
+/// Corre los pasos del perfil y, con `TC2_INFORME`, escribe ahi el informe
+/// JSON de F5-4 (tambien en rojo).
 fn verificar(perfil: Perfil) -> Result<(), String> {
+    let destino = std::env::var_os("TC2_INFORME")
+        .filter(|v| !v.is_empty())
+        .map(PathBuf::from);
+    let inicio = segundos_unix();
+    let mut pasos = Vec::new();
+    let resultado = verificar_pasos(perfil, &mut pasos);
+    let Some(ruta) = destino else {
+        return resultado;
+    };
+    let informe = InformeVerificacion {
+        perfil,
+        verde: resultado.is_ok(),
+        commit: commit_actual(),
+        inicio,
+        fin: segundos_unix(),
+        pasos,
+    };
+    match (escribir_informe(&ruta, &informe.json()), resultado) {
+        (Ok(()), r) => {
+            println!("xtask: informe de verificacion en {}", ruta.display());
+            r
+        }
+        (Err(e), Ok(())) => Err(e),
+        (Err(e), Err(r)) => {
+            eprintln!("xtask: {e}");
+            Err(r)
+        }
+    }
+}
+
+fn verificar_pasos(perfil: Perfil, pasos: &mut Vec<PasoVerificacion>) -> Result<(), String> {
     guarda_rustflags()?;
     let raiz = raiz_workspace()?;
-    cargo(&raiz, &["fmt", "--all", "--check"], None)?;
-    cargo(
-        &raiz,
-        &[
-            "clippy",
-            "--workspace",
-            "--all-targets",
-            "--locked",
-            "--",
-            "-D",
-            "warnings",
-        ],
-        None,
-    )?;
-    cargo(&raiz, &["test", "--workspace", "--locked"], None)?;
+    paso(pasos, "fmt", || {
+        cargo(&raiz, &["fmt", "--all", "--check"], None)
+    })?;
+    paso(pasos, "clippy", || {
+        cargo(
+            &raiz,
+            &[
+                "clippy",
+                "--workspace",
+                "--all-targets",
+                "--locked",
+                "--",
+                "-D",
+                "warnings",
+            ],
+            None,
+        )
+    })?;
+    paso(pasos, "test", || {
+        cargo(&raiz, &["test", "--workspace", "--locked"], None)
+    })?;
     if perfil == Perfil::Completa {
-        reproducible(&raiz)?;
+        paso(pasos, "reproducible", || reproducible(&raiz))?;
     }
     Ok(())
 }
 
+/// Corre un paso y anota su resultado y su duracion.
+fn paso(
+    pasos: &mut Vec<PasoVerificacion>,
+    nombre: &'static str,
+    f: impl FnOnce() -> Result<(), String>,
+) -> Result<(), String> {
+    let t = std::time::Instant::now();
+    let r = f();
+    pasos.push(PasoVerificacion {
+        nombre,
+        verde: r.is_ok(),
+        segundos: t.elapsed().as_secs(),
+    });
+    r
+}
+
+// --- Informe de verificacion (F5-4) ------------------------------------------
+//
+// Contrato: crates/tc2/esquemas/informe-verificacion.schema.json (v1), que lee
+// `tc2::verificacion::informe`. Se escribe a mano: xtask no depende de tc2 ni
+// de serde. El test compara con el fixture que lee tc2.
+
+#[derive(Debug, Clone, PartialEq, Eq)]
+struct PasoVerificacion {
+    nombre: &'static str,
+    verde: bool,
+    segundos: u64,
+}
+
+#[derive(Debug, Clone, PartialEq, Eq)]
+struct InformeVerificacion {
+    perfil: Perfil,
+    verde: bool,
+    commit: Option<String>,
+    /// Segundos desde la epoca Unix, en UTC.
+    inicio: u64,
+    fin: u64,
+    pasos: Vec<PasoVerificacion>,
+}
+
+impl Perfil {
+    fn nombre(self) -> &'static str {
+        match self {
+            Perfil::Rapida => "rapida",
+            Perfil::Completa => "completa",
+        }
+    }
+}
+
+fn resultado(verde: bool) -> &'static str {
+    if verde { "verde" } else { "rojo" }
+}
+
+impl InformeVerificacion {
+    fn json(&self) -> String {
+        let commit = self
+            .commit
+            .as_deref()
+            .map_or_else(|| "null".to_owned(), |c| format!("\"{c}\""));
+        let pasos: Vec<String> = self
+            .pasos
+            .iter()
+            .map(|p| {
+                format!(
+                    "    {{ \"nombre\": \"{}\", \"resultado\": \"{}\", \"segundos\": {} }}",
+                    p.nombre,
+                    resultado(p.verde),
+                    p.segundos
+                )
+            })
+            .collect();
+        format!(
+            "{{\n  \"version\": 1,\n  \"perfil\": \"{}\",\n  \"resultado\": \"{}\",\n  \"commit\": {commit},\n  \"inicio\": \"{}\",\n  \"fin\": \"{}\",\n  \"pasos\": [\n{}\n  ]\n}}\n",
+            self.perfil.nombre(),
+            resultado(self.verde),
+            fecha_utc(self.inicio),
+            fecha_utc(self.fin),
+            pasos.join(",\n")
+        )
+    }
+}
+
+fn escribir_informe(ruta: &Path, json: &str) -> Result<(), String> {
+    if let Some(padre) = ruta.parent().filter(|p| !p.as_os_str().is_empty()) {
+        std::fs::create_dir_all(padre).map_err(|e| format!("{}: {e}", padre.display()))?;
+    }
+    std::fs::write(ruta, json).map_err(|e| format!("informe {}: {e}", ruta.display()))
+}
+
+fn segundos_unix() -> u64 {
+    std::time::SystemTime::now()
+        .duration_since(std::time::UNIX_EPOCH)
+        .map_or(0, |d| d.as_secs())
+}
+
+/// `git rev-parse HEAD`, solo si da 40 hex en minusculas.
+fn commit_actual() -> Option<String> {
+    let raiz = raiz_workspace().ok()?;
+    let salida = Command::new("git")
+        .args(["rev-parse", "HEAD"])
+        .current_dir(raiz)
+        .output()
+        .ok()?;
+    let texto = String::from_utf8(salida.stdout).ok()?;
+    let sha = texto.trim();
+    let hex = |c: char| c.is_ascii_digit() || ('a'..='f').contains(&c);
+    (salida.status.success() && sha.len() == 40 && sha.chars().all(hex)).then(|| sha.to_owned())
+}
+
+/// `AAAA-MM-DDTHH:MM:SSZ` de unos segundos Unix (algoritmo de dias civiles de
+/// H. Hinnant).
+fn fecha_utc(segundos: u64) -> String {
+    let dias = segundos / 86_400;
+    let resto = segundos % 86_400;
+    let z = dias + 719_468;
+    let era = z / 146_097;
+    let doe = z - era * 146_097;
+    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
+    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
+    let mp = (5 * doy + 2) / 153;
+    let dia = doy - (153 * mp + 2) / 5 + 1;
+    let mes = if mp < 10 { mp + 3 } else { mp - 9 };
+    let anio = yoe + era * 400 + u64::from(mes <= 2);
+    format!(
+        "{anio:04}-{mes:02}-{dia:02}T{:02}:{:02}:{:02}Z",
+        resto / 3600,
+        resto % 3600 / 60,
+        resto % 60
+    )
+}
+
 fn raiz_workspace() -> Result<PathBuf, String> {
     Path::new(env!("CARGO_MANIFEST_DIR"))
         .parent()
@@ -488,6 +661,50 @@ mod tests {
         );
     }
 
+    /// El informe de una ejecucion con pasos fijos es byte a byte el fixture
+    /// que lee `tc2::verificacion::informe` (contrato entre las dos cajas).
+    #[test]
+    fn el_informe_es_el_fixture_de_tc2() {
+        let fixture = |nombre: &str| {
+            let ruta = Path::new(env!("CARGO_MANIFEST_DIR"))
+                .join("../crates/tc2/tests/fixtures/informe-verificacion")
+                .join(nombre);
+            std::fs::read_to_string(&ruta)
+                .map(|t| t.replace("\r\n", "\n"))
+                .map_err(|e| format!("{}: {e}", ruta.display()))
+        };
+        let p = |nombre, verde, segundos| PasoVerificacion {
+            nombre,
+            verde,
+            segundos,
+        };
+        let verde = InformeVerificacion {
+            perfil: Perfil::Completa,
+            verde: true,
+            commit: Some("0123456789abcdef0123456789abcdef01234567".to_owned()),
+            inicio: 1_795_109_457,
+            fin: 1_795_110_130,
+            pasos: vec![
+                p("fmt", true, 2),
+                p("clippy", true, 95),
+                p("test", true, 310),
+                p("reproducible", true, 266),
+            ],
+        };
+        assert_eq!(Ok(verde.json()), fixture("xtask.json"));
+        let rojo = InformeVerificacion {
+            perfil: Perfil::Rapida,
+            verde: false,
+            commit: None,
+            inicio: 1_795_111_200,
+            fin: 1_795_111_243,
+            pasos: vec![p("fmt", true, 1), p("clippy", false, 42)],
+        };
+        assert_eq!(Ok(rojo.json()), fixture("xtask-rojo.json"));
+        assert_eq!(fecha_utc(0), "1970-01-01T00:00:00Z");
+        assert_eq!(fecha_utc(951_868_799), "2000-02-29T23:59:59Z");
+    }
+
     #[test]
     fn comparar_hashes_exige_dos_iguales() {
         assert!(comparar_hashes(&v(&["aa", "aa"])).is_ok());
diff --git a/xtask/src/workflows.rs b/xtask/src/workflows.rs
index 879d5e5..b1373f6 100644
--- a/xtask/src/workflows.rs
+++ b/xtask/src/workflows.rs
@@ -9,9 +9,10 @@
 //!   `repository_dispatch`, `workflow_run`, comentarios...);
 //! - bloque `permissions` arriba y en cada job, sin `write-all` ni
 //!   `read-all`; un job con algun `write` es de clase privilegio;
-//! - acciones admitidas: solo `actions/checkout`, fijada por SHA y con
+//! - acciones admitidas: `actions/checkout`, fijada por SHA y con
 //!   `persist-credentials: false`; en clase privilegio, sin `ref:` ni
-//!   `repository:`;
+//!   `repository:`; y `actions/upload-artifact` (informe de F5-4), fijada
+//!   por SHA y solo en clase ejecuta;
 //! - ni `secrets: inherit`, ni secretos en clase ejecuta, ni
 //!   `github.event.*`, `github.head_ref` o `toJSON(github...)` en `run:`
 //!   (los datos del evento se pasan por `env:`).
@@ -323,7 +324,11 @@ const EVENTOS_ADMITIDOS: [&str; 5] = [
     "workflow_dispatch",
 ];
 
-const ACCIONES_ADMITIDAS: [&str; 1] = ["actions/checkout"];
+/// Subida del informe JSON (F5-4): solo en clase ejecuta (sin escritura ni
+/// secretos), fijada por SHA.
+const SUBIDA: &str = "actions/upload-artifact";
+
+const ACCIONES_ADMITIDAS: [&str; 2] = ["actions/checkout", SUBIDA];
 
 /// Datos del evento que el autor del PR controla: nunca dentro de `run:`.
 /// Se buscan sobre el valor normalizado (ver `normalizar`).
@@ -497,6 +502,12 @@ fn lint_github(texto: &str) -> Vec<String> {
             if !es_sha(version) {
                 errores.push(format!("job {nombre}: {accion} sin fijar por SHA"));
             }
+            if accion == SUBIDA {
+                if privilegio {
+                    errores.push(format!("job {nombre}: {SUBIDA} en clase privilegio"));
+                }
+                continue;
+            }
             let con = paso.get("with");
             let opcion = |k: &str| con.and_then(|w| w.get(k));
             if opcion("persist-credentials").and_then(Nodo::texto) != Some("false") {
@@ -992,6 +1003,89 @@ fn la_concurrencia_es_por_pr_o_ref() {
     );
 }
 
+/// F5-4: `ejecutar` sube el informe aunque el comando falle, el publicador
+/// nombra los artefactos en el resumen, la plantilla los guarda siempre y la
+/// subida no se admite en un job que escribe.
+#[test]
+fn el_informe_se_sube_siempre_y_solo_desde_ejecutar() {
+    let raiz = yaml(&leer(WORKFLOW));
+    let job = |n: &str| raiz.get("jobs").and_then(|j| j.get(n));
+    let pasos = |n: &str| match job(n).and_then(|j| j.get("steps")) {
+        Some(Nodo::Lista(p)) => p.clone(),
+        _ => Vec::new(),
+    };
+    let subida = pasos("ejecutar")
+        .into_iter()
+        .find(|p| {
+            p.get("uses")
+                .and_then(Nodo::texto)
+                .is_some_and(|u| u.starts_with("actions/upload-artifact@"))
+        })
+        .unwrap_or_else(|| panic!("ejecutar no sube el informe"));
+    assert_eq!(subida.get("if").and_then(Nodo::texto), Some("always()"));
+    let con = |k: &str| {
+        subida
+            .get("with")
+            .and_then(|w| w.get(k))
+            .and_then(Nodo::texto)
+    };
+    assert_eq!(
+        con("name"),
+        Some("informe-verificacion-${{ matrix.perfil }}")
+    );
+    assert_eq!(con("if-no-files-found"), Some("ignore"));
+    let publicar = pasos("verificacion")
+        .first()
+        .and_then(|p| p.get("run"))
+        .and_then(Nodo::texto)
+        .map(str::to_owned)
+        .unwrap_or_default();
+    assert!(publicar.contains("informe=$informes"), "{publicar}");
+
+    let texto = leer(PLANTILLA);
+    let (_, cuerpo) = plantilla(&texto);
+    let artefactos = cuerpo
+        .get(".tc2-verificacion")
+        .and_then(|b| b.get("artifacts"));
+    assert_eq!(
+        artefactos.and_then(|a| a.get("when")).and_then(Nodo::texto),
+        Some("always")
+    );
+    assert!(
+        cuerpo
+            .get(".tc2-verificacion")
+            .and_then(|b| b.get("variables"))
+            .and_then(|v| v.get("TC2_INFORME"))
+            .is_some()
+    );
+
+    let en_privilegio = BUENO.replacen(
+        "      - env:\n          GH_TOKEN",
+        "      - uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02\n      - env:\n          GH_TOKEN",
+        1,
+    );
+    assert_ne!(en_privilegio, BUENO);
+    let errores = lint_github(&en_privilegio);
+    assert!(
+        errores
+            .iter()
+            .any(|e| e.contains("job publicar: actions/upload-artifact en clase privilegio")),
+        "{errores:?}"
+    );
+    let sin_sha = BUENO.replacen(
+        "      - run: echo \"$TITULO\"",
+        "      - uses: actions/upload-artifact@v4",
+        1,
+    );
+    assert!(
+        lint_github(&sin_sha)
+            .iter()
+            .any(|e| e.contains("actions/upload-artifact sin fijar por SHA")),
+        "{:?}",
+        lint_github(&sin_sha)
+    );
+}
+
 /// `rules:if` no ve las `variables:` de los jobs: la plantilla no las usa en
 /// las reglas, y el lint lo detecta si alguien lo vuelve a hacer.
 #[test]
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-044/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-044/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-044/tarea.md                                    | 3 ++-
 3 files changed, 2 insertions(+), 1 deletion(-)
````
