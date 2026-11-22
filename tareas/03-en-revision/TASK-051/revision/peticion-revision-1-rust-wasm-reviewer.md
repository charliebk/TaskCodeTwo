# Peticion de revision — TASK-051 (ronda 1)

- Tarea: TASK-051 — F6-5 Hallazgos estructurados
- Rama revisada: feature/task-051-f6-5-hallazgos-estructurados
- Rama base: develop
- Commit revisado (HEAD): 02000f5a7c78e8404c5a4257bdb7eb7ecc6bfa1b
- Fecha: 2026-11-22
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-051 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-051/tarea.md
- tareas/03-en-revision/TASK-051/planificacion/
- crates/tc2/src/hallazgos/mod.rs
- crates/tc2/src/hallazgos/orden.rs
- crates/tc2/src/i18n/catalogo/hallazgos.rs
- crates/tc2/src/i18n/catalogo/plataforma.rs
- crates/tc2/src/plataforma/github.rs
- crates/tc2/src/plataforma/mod.rs
- crates/tc2/src/revision/guardar.rs
- crates/tc2/src/revision/mod.rs
- crates/tc2/tests/hallazgos.rs
- crates/tc2/tests/plataforma.rs
- crates/tc2/tests/plataforma_e2e.rs
- crates/tc2/tests/revision.rs
- xtask/src/workflows.rs

## Commits a revisar (git log develop..HEAD)

````
02000f5 test(TASK-051): E2E de revision-ia con hallazgos en el repo de pruebas
051c157 feat(TASK-051): review guarda el informe de revision-ia en la tarea
9d30c10 feat(TASK-051): leer-revision en el puerto y en GitHub
62fdc8c feat(TASK-051): revision-ia publica el veredicto de tc2 y el lint lo exige
53456c0 feat(TASK-051): tc2 hallazgos veredicto con check.json y lotes
6f24681 feat(TASK-051): clase, esquema v2, veredicto y anotaciones puros
57e103f chore(TASK-051): tarea en curso
````

## Diff de tu dominio (13 fichero(s) de 22; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/hallazgos/mod.rs b/crates/tc2/src/hallazgos/mod.rs
index 39f583b..b0290f3 100644
--- a/crates/tc2/src/hallazgos/mod.rs
+++ b/crates/tc2/src/hallazgos/mod.rs
@@ -8,14 +8,21 @@
 //! no llama a la red ni a un modelo: el modelo no decide, lo que devuelve es
 //! un dato que se valida.
 //!
-//! Contrato (`crates/tc2/esquemas/hallazgos.schema.json`, version 1): la
+//! Contrato (`crates/tc2/esquemas/hallazgos.schema.json`, version 2): la
 //! respuesta trae exactamente un bloque `json` cercado con acentos graves con
 //! una lista de hallazgos (`$defs/respuesta` del esquema); cada hallazgo
-//! apunta a una linea del lado nuevo del diff y trae evidencia de su tipo
+//! apunta a una linea del lado nuevo del diff, lleva su clase de la lista
+//! cerrada de F1-15 ([`Clase`], F6-5) y trae evidencia de su tipo
 //! (`reproduccion`, `mutante` o `borde`), tambien los MENOR. La evidencia se
 //! declara y se ancla al diff, pero no se ejecuta: eso es de F6-6 (adenda
 //! F6-4 de F1-8). [`instrucciones`] es el texto que va al prompt, hecho con
 //! las mismas constantes que el validador para que no se separen.
+//!
+//! F6-5: [`veredicto`] junta los [`Validados`] de todas las revisoras y saca
+//! la conclusion del check `revision-ia` (`failure` si queda abierto un
+//! CRITICO o IMPORTANTE aceptado o si alguna revisora no dejo respuesta
+//! valida; `success` si no), su resumen ([`resumen_veredicto`]) y las
+//! anotaciones por linea ([`anotaciones`]). Tambien es una funcion pura.
 
 pub mod orden;
 
@@ -27,7 +34,7 @@ use serde_json::Value;
 use crate::verificacion::informe::{Informe as InformeVerificacion, Resultado as ResultadoPaso};
 
 /// Version del contrato de hallazgos.
-pub const VERSION: u32 = 1;
+pub const VERSION: u32 = 2;
 
 /// Hallazgos que se validan como mucho por respuesta; los que sobran se
 /// rechazan juntos con [`motivo::FUERA_DE_TOPE`].
@@ -134,6 +141,55 @@ impl Severidad {
     }
 }
 
+/// Clase de un hallazgo: la lista cerrada de F1-15 (decision 12 y D7a), con
+/// precedencia en este orden si encaja en varias (F6-5).
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(rename_all = "kebab-case")]
+pub enum Clase {
+    PuertaSeguridad,
+    Contrato,
+    Correccion,
+    FidelidadEvidencia,
+    Reproducibilidad,
+}
+
+impl Clase {
+    /// Todas, en el orden de precedencia.
+    pub const TODAS: [Clase; 5] = [
+        Clase::PuertaSeguridad,
+        Clase::Contrato,
+        Clase::Correccion,
+        Clase::FidelidadEvidencia,
+        Clase::Reproducibilidad,
+    ];
+
+    /// Como se escribe en el contrato.
+    pub fn texto(self) -> &'static str {
+        match self {
+            Clase::PuertaSeguridad => "puerta-seguridad",
+            Clase::Contrato => "contrato",
+            Clase::Correccion => "correccion",
+            Clase::FidelidadEvidencia => "fidelidad-evidencia",
+            Clase::Reproducibilidad => "reproducibilidad",
+        }
+    }
+
+    /// Cuando se aplica, para el prompt.
+    fn cuando(self) -> &'static str {
+        match self {
+            Clase::PuertaSeguridad => {
+                "la seguridad, o una puerta que deja pasar lo que deberia parar"
+            }
+            Clase::Contrato => "una interfaz, formato o linea de contrato no cumple lo acordado",
+            Clase::Correccion => "el codigo hace otra cosa de la que deberia",
+            Clase::FidelidadEvidencia => {
+                "lo que se afirma (docs, cifras, tests) no se sostiene con la evidencia"
+            }
+            Clase::Reproducibilidad => "el resultado depende del entorno, del orden o del azar",
+        }
+    }
+}
+
 /// Tipo de evidencia (RR-03: reproduccion, mutantes y bordes).
 #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
 #[serde(rename_all = "lowercase")]
@@ -204,16 +260,20 @@ pub struct Evidencia {
 struct Cabecera {
     id: String,
     severidad: Severidad,
+    clase: Clase,
     fichero: String,
     linea: u32,
     descripcion: String,
 }
 
-/// Un hallazgo que cumple el contrato.
-#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
+/// Un hallazgo que cumple el contrato. Se lee de vuelta (F6-5) del
+/// `raw_details` de su anotacion, sin claves de mas.
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(deny_unknown_fields)]
 pub struct Aceptado {
     pub id: String,
     pub severidad: Severidad,
+    pub clase: Clase,
     pub fichero: String,
     pub linea: u32,
     pub descripcion: String,
@@ -227,14 +287,15 @@ pub struct Rechazo {
     pub indice: Option<usize>,
     /// El id del hallazgo, si tiene uno con forma valida.
     pub id: Option<String>,
-    /// Uno de [`MOTIVOS`].
+    /// Uno de [`MOTIVOS`]; al leerlo de vuelta, uno desconocido es error.
     pub motivo: &'static str,
     /// Que fallo, para la persona.
     pub detalle: String,
 }
 
 /// Resultado de validar la respuesta de una revisora.
-#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(deny_unknown_fields)]
 pub struct Validados {
     pub version: u32,
     pub revisora: String,
@@ -242,6 +303,34 @@ pub struct Validados {
     pub rechazados: Vec<Rechazo>,
 }
 
+/// Un [`Rechazo`] leido de vuelta, con el motivo aun sin contrastar.
+#[derive(Deserialize)]
+#[serde(deny_unknown_fields)]
+struct RechazoLeido {
+    indice: Option<usize>,
+    id: Option<String>,
+    motivo: String,
+    detalle: String,
+}
+
+impl<'de> Deserialize<'de> for Rechazo {
+    /// El motivo tiene que ser uno de [`MOTIVOS`].
+    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
+        let r = RechazoLeido::deserialize(d)?;
+        let motivo = MOTIVOS
+            .iter()
+            .map(|(id, _)| *id)
+            .find(|id| *id == r.motivo)
+            .ok_or_else(|| serde::de::Error::custom(format!("motivo desconocido: {}", r.motivo)))?;
+        Ok(Self {
+            indice: r.indice,
+            id: r.id,
+            motivo,
+            detalle: r.detalle,
+        })
+    }
+}
+
 impl Rechazo {
     /// A quien se refiere: el id del hallazgo, `#<indice>` si no tiene uno
     /// valido, o `respuesta`.
@@ -642,6 +731,7 @@ fn validar_uno(
     Ok(Aceptado {
         id: c.id,
         severidad: c.severidad,
+        clase: c.clase,
         fichero: c.fichero,
         linea: c.linea,
         descripcion: c.descripcion,
@@ -723,6 +813,7 @@ const EJEMPLO: &str = r#"[
   {
     "id": "H1",
     "severidad": "CRITICO",
+    "clase": "correccion",
     "fichero": "src/calculo.rs",
     "linea": 5,
     "descripcion": "La condicion esta al reves: con dos o mas valores devuelve None.",
@@ -736,6 +827,7 @@ const EJEMPLO: &str = r#"[
   {
     "id": "H2",
     "severidad": "MENOR",
+    "clase": "fidelidad-evidencia",
     "fichero": "src/calculo.rs",
     "linea": 12,
     "descripcion": "Ningun test fija el tope de 100.",
@@ -764,6 +856,10 @@ El check `revision-ia` lista los aceptados y los rechazados con su motivo.\n\n",
         .map(|x| format!("`{}`", x.texto()))
         .collect();
     s.push_str(&format!("- `severidad`: {}.\n", severidades.join(", ")));
+    s.push_str("- `clase`: una de esta lista; si encaja en varias, la primera (F1-15):\n");
+    for c in Clase::TODAS {
+        s.push_str(&format!("  - `{}`: {}.\n", c.texto(), c.cuando()));
+    }
     s.push_str(
         "- `fichero`: la ruta tal como sale en la cabecera `+++ b/<ruta>` del diff, sin `b/`.\n",
     );
@@ -910,12 +1006,13 @@ pub fn resumen_markdown(v: &Validados) -> String {
         v.rechazados.len()
     );
     if !v.aceptados.is_empty() {
-        s.push_str("| id | severidad | fichero | linea | evidencia | descripcion |\n|---|---|---|---|---|---|\n");
+        s.push_str("| id | severidad | clase | fichero | linea | evidencia | descripcion |\n|---|---|---|---|---|---|---|\n");
         for a in &v.aceptados {
             s.push_str(&format!(
-                "| {} | {} | {} | {} | {} | {} |\n",
+                "| {} | {} | {} | {} | {} | {} | {} |\n",
                 plano(&a.id, 64),
                 a.severidad.texto(),
+                a.clase.texto(),
                 plano(&a.fichero, 200),
                 a.linea,
                 a.evidencia.tipo.texto(),
@@ -939,6 +1036,363 @@ pub fn resumen_markdown(v: &Validados) -> String {
     s
 }
 
+// --- Veredicto del check (F6-5) ---------------------------------------------
+
+/// Anotaciones como mucho por peticion a la API de check-runs (tope de
+/// GitHub).
+pub const ANOTACIONES_POR_LOTE: usize = 50;
+
+/// Prefijo del titulo del check; tras el viene el veredicto.
+pub const PREFIJO_TITULO: &str = "revision-ia: ";
+
+/// Lo que dice el check.
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(rename_all = "lowercase")]
+pub enum Dictamen {
+    /// Queda abierto un CRITICO o IMPORTANTE aceptado.
+    Rojo,
+    /// Sin CRITICOS ni IMPORTANTES abiertos.
+    Verde,
+    /// Alguna revisora no dejo respuesta valida, o no hubo ninguna: la
+    /// revision no ocurrio (falla en cerrado, decision 1 de TASK-051).
+    Incompleta,
+}
+
+impl Dictamen {
+    /// Como se escribe en la linea y en el titulo.
+    pub fn texto(self) -> &'static str {
+        match self {
+            Dictamen::Rojo => "rojo",
+            Dictamen::Verde => "verde",
+            Dictamen::Incompleta => "incompleta",
+        }
+    }
+
+    /// Conclusion del check-run: solo `success` o `failure` (nunca
+    /// `neutral`, que GitHub da por cumplido).
+    pub fn conclusion(self) -> &'static str {
+        match self {
+            Dictamen::Verde => "success",
+            Dictamen::Rojo | Dictamen::Incompleta => "failure",
+        }
+    }
+
+    /// El dictamen de un titulo del check (`revision-ia: <dictamen> ...`).
+    pub fn de_titulo(titulo: &str) -> Option<Self> {
+        let resto = titulo.strip_prefix(PREFIJO_TITULO)?;
+        match resto.split(' ').next()? {
+            "rojo" => Some(Dictamen::Rojo),
+            "verde" => Some(Dictamen::Verde),
+            "incompleta" => Some(Dictamen::Incompleta),
+            _ => None,
+        }
+    }
+}
+
+/// Un aceptado con la revisora que lo encontro: el `raw_details` de su
+/// anotacion, que `tc2 review` lee de vuelta.
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct AceptadoDe {
+    pub revisora: String,
+    pub hallazgo: Aceptado,
+}
+
+/// Una revisora en el resumen del veredicto.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Fila {
+    /// `None` si no dejo un JSON de `validar` legible.
+    pub revisora: Option<String>,
+    pub aceptados: usize,
+    pub rechazados: usize,
+    /// El motivo del rechazo de toda la respuesta, si lo hubo.
+    pub incompleta: Option<&'static str>,
+}
+
+/// El veredicto de todas las revisoras.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Veredicto {
+    pub dictamen: Dictamen,
+    pub criticos: usize,
+    pub importantes: usize,
+    pub menores: usize,
+    pub rechazados: usize,
+    pub filas: Vec<Fila>,
+    pub aceptados: Vec<AceptadoDe>,
+}
+
+impl Veredicto {
+    /// Revisoras sin respuesta valida.
+    pub fn incompletas(&self) -> usize {
+        self.filas
+            .iter()
+            .filter(|f| f.revisora.is_none() || f.incompleta.is_some())
+            .count()
+    }
+
+    /// Titulo del check.
+    pub fn titulo(&self) -> String {
+        format!(
+            "{PREFIJO_TITULO}{} (criticos={} importantes={} menores={})",
+            self.dictamen.texto(),
+            self.criticos,
+            self.importantes,
+            self.menores
+        )
+    }
+}
+
+/// Junta los resultados de `validar` de cada revisora (`None` si no dejo
+/// uno legible). En F6-5 todo aceptado esta abierto (el cierre es de F6-6).
+pub fn veredicto(revisoras: &[Option<Validados>]) -> Veredicto {
+    let mut v = Veredicto {
+        dictamen: Dictamen::Verde,
+        criticos: 0,
+        importantes: 0,
+        menores: 0,
+        rechazados: 0,
+        filas: Vec::new(),
+        aceptados: Vec::new(),
+    };
+    for r in revisoras {
+        let Some(r) = r else {
+            v.filas.push(Fila {
+                revisora: None,
+                aceptados: 0,
+                rechazados: 0,
+                incompleta: None,
+            });
+            continue;
+        };
+        for a in &r.aceptados {
+            match a.severidad {
+                Severidad::Critico => v.criticos += 1,
+                Severidad::Importante => v.importantes += 1,
+                Severidad::Menor => v.menores += 1,
+            }
+            v.aceptados.push(AceptadoDe {
+                revisora: r.revisora.clone(),
+                hallazgo: a.clone(),
+            });
+        }
+        v.rechazados += r.rechazados.len();
+        v.filas.push(Fila {
+            revisora: Some(r.revisora.clone()),
+            aceptados: r.aceptados.len(),
+            rechazados: r.rechazados.len(),
+            incompleta: r
+                .rechazados
+                .iter()
+                .find(|x| x.indice.is_none())
+                .map(|x| x.motivo),
+        });
+    }
+    v.dictamen = if v.filas.is_empty() || v.incompletas() > 0 {
+        Dictamen::Incompleta
+    } else if v.criticos + v.importantes > 0 {
+        Dictamen::Rojo
+    } else {
+        Dictamen::Verde
+    };
+    v
+}
+
+/// Una anotacion del check-run (API de check-runs de GitHub).
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct Anotacion {
+    pub path: String,
+    pub start_line: u32,
+    pub end_line: u32,
+    pub annotation_level: String,
+    pub title: String,
+    pub message: String,
+    pub raw_details: String,
+}
+
+/// Una anotacion por aceptado, en lotes de [`ANOTACIONES_POR_LOTE`]:
+/// `failure` para CRITICO e IMPORTANTE, `warning` para MENOR; titulo y
+/// mensaje pasados por [`plano`]; `raw_details`, el [`AceptadoDe`] en JSON.
+pub fn anotaciones(v: &Veredicto) -> Vec<Vec<Anotacion>> {
+    let todas: Vec<Anotacion> = v
+        .aceptados
+        .iter()
+        .map(|a| {
+            let h = &a.hallazgo;
+            let titulo = format!(
+                "{} {} {} ({})",
+                plano(&h.id, 64),
+                h.severidad.texto(),
+                h.clase.texto(),
+                plano(&a.revisora, 120)
+            );
+            Anotacion {
+                path: h.fichero.clone(),
+                start_line: h.linea,
+                end_line: h.linea,
+                annotation_level: match h.severidad {
+                    Severidad::Critico | Severidad::Importante => "failure",
+                    Severidad::Menor => "warning",
+                }
+                .to_owned(),
+                // GitHub corta el titulo en 255 caracteres.
+                title: titulo.chars().take(255).collect(),
+                message: plano(&h.descripcion, 4000),
+                raw_details: serde_json::to_string(a).unwrap_or_default(),
+            }
+        })
+        .collect();
+    todas
+        .chunks(ANOTACIONES_POR_LOTE)
+        .map(<[Anotacion]>::to_vec)
+        .collect()
+}
+
+/// El resumen del check: el veredicto, una fila por revisora (todo lo del
+/// modelo por [`plano`]) y, si se dan, las lineas de `extra` (la seleccion y
+/// el estado de cada revisora que deja el workflow), tambien por [`plano`].
+pub fn resumen_veredicto(v: &Veredicto, extra: &str) -> String {
+    let explicacion = match v.dictamen {
+        Dictamen::Rojo => {
+            "Queda abierto al menos un hallazgo CRITICO o IMPORTANTE aceptado: el check sale en rojo (F6-5)."
+        }
+        Dictamen::Verde => "Sin hallazgos CRITICOS ni IMPORTANTES abiertos.",
+        Dictamen::Incompleta => {
+            "Alguna revisora no dejo una respuesta valida (o no hubo ninguna): la revision no se completo y el check sale en rojo (F6-5)."
+        }
+    };
+    let mut s = format!(
+        "**Veredicto: {}** (conclusion {}). {explicacion}\n\n\
+         Aceptados: {} CRITICO, {} IMPORTANTE, {} MENOR; rechazados: {}; revisoras: {}, incompletas: {}.\n\n",
+        v.dictamen.texto(),
+        v.dictamen.conclusion(),
+        v.criticos,
+        v.importantes,
+        v.menores,
+        v.rechazados,
+        v.filas.len(),
+        v.incompletas()
+    );
+    if !v.filas.is_empty() {
+        s.push_str("| revisora | aceptados | rechazados | respuesta |\n|---|---|---|---|\n");
+        for (k, f) in v.filas.iter().enumerate() {
+            let (quien, respuesta) = match (&f.revisora, f.incompleta) {
+                (None, _) => (format!("#{k}"), "sin-resultado".to_owned()),
+                (Some(r), None) => (plano(r, 120), "valida".to_owned()),
+                (Some(r), Some(m)) => (plano(r, 120), format!("incompleta ({m})")),
+            };
+            s.push_str(&format!(
+                "| {quien} | {} | {} | {respuesta} |\n",
+                f.aceptados, f.rechazados
+            ));
+        }
+        s.push('\n');
+    }
+    let lineas: Vec<String> = extra
+        .lines()
+        .filter(|l| no_vacio(l))
+        .map(|l| plano(l, 2000))
+        .collect();
+    if !lineas.is_empty() {
+        s.push_str("Seleccion y estado de cada revisora:\n\n");
+        for l in lineas {
+            s.push_str(&format!("- {l}\n"));
+        }
+    }
+    s
+}
+
+/// El informe de la revision IA en la carpeta de la tarea (F6-5,
+/// `$defs/informe-tarea`): leido del check publicado, sin la prosa del
+/// modelo.
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct InformeTarea {
+    pub version: u32,
+    pub tarea: String,
+    pub pr: u64,
+    pub sha: String,
+    pub conclusion: String,
+    pub veredicto: Dictamen,
+    pub criticos: usize,
+    pub importantes: usize,
+    pub menores: usize,
+    /// El resumen del check (ya pasado por [`plano`]).
+    pub resumen: String,
+    pub aceptados: Vec<AceptadoDe>,
+}
+
+impl InformeTarea {
+    /// El informe de la tarea `tarea` con lo leido del check.
+    pub fn nuevo(
+        tarea: &str,
+        pr: u64,
+        sha: &str,
+        (conclusion, veredicto): (&str, Dictamen),
+        resumen: &str,
+        aceptados: Vec<AceptadoDe>,
+    ) -> Self {
+        let n = |s: Severidad| {
+            aceptados
+                .iter()
+                .filter(|a| a.hallazgo.severidad == s)
+                .count()
+        };
+        Self {
+            version: VERSION,
+            tarea: tarea.to_owned(),
+            pr,
+            sha: sha.to_owned(),
+            conclusion: conclusion.to_owned(),
+            veredicto,
+            criticos: n(Severidad::Critico),
+            importantes: n(Severidad::Importante),
+            menores: n(Severidad::Menor),
+            resumen: resumen.to_owned(),
+            aceptados,
+        }
+    }
+
+    /// El mismo informe en Markdown: veredicto y tabla de aceptados por
+    /// [`plano`].
+    pub fn markdown(&self) -> String {
+        let mut s = format!(
+            "# Revision IA de {} (PR {}, commit {})\n\n\
+             Veredicto: **{}** (conclusion {}). Aceptados: {} CRITICO, {} IMPORTANTE, {} MENOR.\n\n",
+            plano(&self.tarea, 64),
+            self.pr,
+            plano(&self.sha, 64),
+            self.veredicto.texto(),
+            plano(&self.conclusion, 32),
+            self.criticos,
+            self.importantes,
+            self.menores
+        );
+        if self.aceptados.is_empty() {
+            s.push_str("Sin hallazgos aceptados.\n");
+        } else {
+            s.push_str(
+                "| revisora | id | severidad | clase | fichero | linea | evidencia | descripcion |\n|---|---|---|---|---|---|---|---|\n",
+            );
+            for a in &self.aceptados {
+                let h = &a.hallazgo;
+                s.push_str(&format!(
+                    "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
+                    plano(&a.revisora, 120),
+                    plano(&h.id, 64),
+                    h.severidad.texto(),
+                    h.clase.texto(),
+                    plano(&h.fichero, 200),
+                    h.linea,
+                    h.evidencia.tipo.texto(),
+                    plano(&h.descripcion, 400)
+                ));
+            }
+        }
+        s
+    }
+}
+
 #[cfg(test)]
 #[allow(clippy::panic)]
 mod tests {
@@ -957,7 +1411,7 @@ mod tests {
         format!("Informe.\n\n```json\n[{json}]\n```\n")
     }
 
-    const BORDE: &str = r#"{"id":"B","severidad":"MENOR","fichero":"src/calculo.rs","linea":5,"descripcion":"d","evidencia":{"tipo":"borde","entrada":"x","esperado":"1","obtenido":"2"}}"#;
+    const BORDE: &str = r#"{"id":"B","severidad":"MENOR","clase":"correccion","fichero":"src/calculo.rs","linea":5,"descripcion":"d","evidencia":{"tipo":"borde","entrada":"x","esperado":"1","obtenido":"2"}}"#;
 
     #[test]
     fn el_diff_da_las_lineas_del_lado_nuevo() {
@@ -1015,7 +1469,7 @@ mod tests {
         let v = validar(
             "r",
             &una(
-                r#"{"id":"Q","severidad":"MENOR","fichero":"src/ñ.rs","linea":2,"descripcion":"d","evidencia":{"tipo":"borde","entrada":"x","esperado":"1","obtenido":"2"}}"#,
+                r#"{"id":"Q","severidad":"MENOR","clase":"correccion","fichero":"src/ñ.rs","linea":2,"descripcion":"d","evidencia":{"tipo":"borde","entrada":"x","esperado":"1","obtenido":"2"}}"#,
             ),
             &d,
             None,
@@ -1045,6 +1499,8 @@ mod tests {
             ("```json\n{\"id\":\"B\"}\n```".to_owned(), "respuesta:json-invalido"),
             (una("3"), "#0:hallazgo-invalido"),
             (una(&BORDE.replace("\"MENOR\"", "\"GRAVE\"")), "B:hallazgo-invalido"),
+            (una(&BORDE.replace("\"clase\":\"correccion\",", "")), "B:hallazgo-invalido"),
+            (una(&BORDE.replace("\"correccion\"", "\"estilo\"")), "B:hallazgo-invalido"),
             (una(&BORDE.replace("\"linea\":5", "\"linea\":0")), "B:hallazgo-invalido"),
             (una(&BORDE.replace("\"linea\":5", "\"linea\":5,\"x\":1")), "B:hallazgo-invalido"),
             (una(&BORDE.replace("\"descripcion\":\"d\"", "\"descripcion\":\" \"")), "B:hallazgo-invalido"),
@@ -1060,23 +1516,23 @@ mod tests {
             (una(&BORDE.replace("src/calculo.rs", "src/otro.rs")), "B:linea-fuera-del-diff"),
             (una(&BORDE.replace("\"obtenido\":\"2\"", "\"obtenido\":\" 1 \"")), "B:borde-sin-diferencia"),
             (
-                una(r#"{"id":"M","severidad":"IMPORTANTE","fichero":"src/calculo.rs","linea":12,"descripcion":"d","evidencia":{"tipo":"mutante","linea_original":"x.min(10)","mutacion":"x.min(11)","test":"t"}}"#),
+                una(r#"{"id":"M","severidad":"IMPORTANTE","clase":"contrato","fichero":"src/calculo.rs","linea":12,"descripcion":"d","evidencia":{"tipo":"mutante","linea_original":"x.min(10)","mutacion":"x.min(11)","test":"t"}}"#),
                 "M:mutante-no-coincide",
             ),
             (
-                una(r#"{"id":"M","severidad":"IMPORTANTE","fichero":"src/calculo.rs","linea":12,"descripcion":"d","evidencia":{"tipo":"mutante","linea_original":"x.min(100)","mutacion":" x.min(100)","test":"t"}}"#),
+                una(r#"{"id":"M","severidad":"IMPORTANTE","clase":"contrato","fichero":"src/calculo.rs","linea":12,"descripcion":"d","evidencia":{"tipo":"mutante","linea_original":"x.min(100)","mutacion":" x.min(100)","test":"t"}}"#),
                 "M:mutante-sin-cambio",
             ),
             (
-                una(r#"{"id":"R","severidad":"MENOR","fichero":"src/calculo.rs","linea":8,"descripcion":"d","evidencia":{"tipo":"reproduccion","pasos":[]}}"#),
+                una(r#"{"id":"R","severidad":"MENOR","clase":"correccion","fichero":"src/calculo.rs","linea":8,"descripcion":"d","evidencia":{"tipo":"reproduccion","pasos":[]}}"#),
                 "R:evidencia-incompleta",
             ),
             (
-                una(r#"{"id":"R","severidad":"MENOR","fichero":"src/calculo.rs","linea":8,"descripcion":"d","evidencia":{"tipo":"reproduccion","pasos":["a"]}}"#),
+                una(r#"{"id":"R","severidad":"MENOR","clase":"correccion","fichero":"src/calculo.rs","linea":8,"descripcion":"d","evidencia":{"tipo":"reproduccion","pasos":["a"]}}"#),
                 "R:evidencia-incompleta",
             ),
             (
-                una(r#"{"id":"R","severidad":"MENOR","fichero":"src/calculo.rs","linea":8,"descripcion":"d","evidencia":{"tipo":"reproduccion","paso_rojo":"clippy"}}"#),
+                una(r#"{"id":"R","severidad":"MENOR","clase":"correccion","fichero":"src/calculo.rs","linea":8,"descripcion":"d","evidencia":{"tipo":"reproduccion","paso_rojo":"clippy"}}"#),
                 "R:paso-rojo-no-esta",
             ),
         ];
@@ -1101,7 +1557,7 @@ mod tests {
         let informe = crate::verificacion::informe::leer(ROJO).unwrap_or_else(|e| panic!("{e}"));
         let r = |paso: &str| {
             una(&format!(
-                r#"{{"id":"R","severidad":"MENOR","fichero":"src/calculo.rs","linea":8,"descripcion":"d","evidencia":{{"tipo":"reproduccion","paso_rojo":"{paso}"}}}}"#
+                r#"{{"id":"R","severidad":"MENOR","clase":"correccion","fichero":"src/calculo.rs","linea":8,"descripcion":"d","evidencia":{{"tipo":"reproduccion","paso_rojo":"{paso}"}}}}"#
             ))
         };
         let v = validar("r", &r("clippy"), &diff(), Some(&informe));
@@ -1131,6 +1587,9 @@ mod tests {
         ] {
             assert!(t.contains(x), "falta {x}");
         }
+        for c in Clase::TODAS {
+            assert!(t.contains(&format!("`{}`", c.texto())), "falta {c:?}");
+        }
         let v = validar("r", &t, &diff(), None);
         assert_eq!((v.aceptados.len(), v.rechazados.len()), (2, 0), "{v:?}");
     }
@@ -1159,6 +1618,11 @@ mod tests {
             textos(&e["$defs"]["hallazgo"]["properties"]["severidad"]["enum"]),
             ["CRITICO", "IMPORTANTE", "MENOR"]
         );
+        let clases: Vec<&str> = Clase::TODAS.iter().map(|c| c.texto()).collect();
+        assert_eq!(
+            textos(&e["$defs"]["hallazgo"]["properties"]["clase"]["enum"]),
+            clases
+        );
         assert_eq!(e["properties"]["version"]["const"], VERSION);
         let v = validar("r", MIXTA, &diff(), None);
         let salida = serde_json::to_value(&v).unwrap_or_else(|x| panic!("{x}"));
diff --git a/crates/tc2/src/hallazgos/orden.rs b/crates/tc2/src/hallazgos/orden.rs
index abb4c12..ed07257 100644
--- a/crates/tc2/src/hallazgos/orden.rs
+++ b/crates/tc2/src/hallazgos/orden.rs
@@ -15,6 +15,18 @@
 //! `instrucciones` escribe en stdout el texto que va al prompt
 //! ([`super::instrucciones`]); con `--json`, en la clave `instrucciones` del
 //! contrato.
+//!
+//! `veredicto --salida <dir> [--hallazgos <json>]... [--extra <txt>]` (F6-5)
+//! junta las salidas de `validar` de todas las revisoras ([`super::veredicto`];
+//! una que no se lee, pasa de [`TOPE_VALIDADOS`] o no es del contrato cuenta
+//! como revisora sin respuesta valida) y escribe en `<dir>` el `check.json`
+//! que publica el workflow (`conclusion`, `output.title`, `output.summary` y
+//! las primeras 50 anotaciones) y un `anotaciones-<k>.json` por cada lote
+//! siguiente. `--extra` son lineas que van al final del resumen (la
+//! seleccion y el estado de cada revisora). Linea: `hallazgos ok
+//! veredicto=<rojo|verde|incompleta> conclusion=<success|failure>
+//! criticos=N importantes=M menores=K revisoras=R incompletas=I lotes=L`.
+//! Un veredicto rojo no es un fallo de la orden: sale `ok`.
 
 use std::io::Read;
 use std::path::{Path, PathBuf};
@@ -39,6 +51,12 @@ pub const TOPE_DIFF: u64 = 4 * 1024 * 1024;
 /// que `tc2 revisoras`).
 pub const TOPE_INFORME: u64 = crate::seleccion::orden::TOPE_INFORME;
 
+/// Bytes que se leen como mucho de cada salida de `validar` (F6-5).
+pub const TOPE_VALIDADOS: u64 = 4 * 1024 * 1024;
+
+/// Bytes que se leen como mucho de `--extra`.
+pub const TOPE_EXTRA: u64 = 64 * 1024;
+
 #[derive(Default)]
 struct Args {
     respuesta: Option<PathBuf>,
@@ -148,6 +166,9 @@ pub fn orden(a: &[String], json: bool, idioma: Idioma) -> Salida {
         [s, resto @ ..] if s == "validar" => match validar(resto) {
             Ok(i) | Err(i) => i.salida_en(json, idioma),
         },
+        [s, resto @ ..] if s == "veredicto" => match veredicto(resto) {
+            Ok(i) | Err(i) => i.salida_en(json, idioma),
+        },
         [] => uso("sin-suborden").salida_en(json, idioma),
         _ => uso("suborden-desconocida").salida_en(json, idioma),
     }
@@ -189,3 +210,75 @@ fn validar(a: &[String]) -> Result<Informe, Informe> {
     }
     Ok(i)
 }
+
+/// Una salida de `validar` del contrato vigente, o `None`.
+fn leer_validados(p: &Path) -> Option<super::Validados> {
+    let t = leer_con_tope(p, TOPE_VALIDADOS).ok()??;
+    serde_json::from_str::<super::Validados>(&t)
+        .ok()
+        .filter(|v| v.version == super::VERSION)
+}
+
+fn escribir_json(p: &Path, v: &serde_json::Value) -> Result<(), Informe> {
+    let texto = serde_json::to_string_pretty(v).map_err(|e| fallo_io(p, e.to_string()))?;
+    std::fs::write(p, texto + "\n").map_err(|e| fallo_io(p, e.to_string()))
+}
+
+fn veredicto(a: &[String]) -> Result<Informe, Informe> {
+    let mut salida: Option<PathBuf> = None;
+    let mut extra: Option<PathBuf> = None;
+    let mut entradas: Vec<PathBuf> = Vec::new();
+    let mut it = a.iter();
+    while let Some(x) = it.next() {
+        let mut valor = || it.next().cloned().ok_or_else(|| uso("falta-valor"));
+        match x.as_str() {
+            "--hallazgos" => entradas.push(valor()?.into()),
+            "--salida" if salida.is_none() => salida = Some(valor()?.into()),
+            "--extra" if extra.is_none() => extra = Some(valor()?.into()),
+            _ => return Err(uso("argumento-desconocido")),
+        }
+    }
+    let Some(dir) = salida else {
+        return Err(uso("sin-salida"));
+    };
+    let extra = match &extra {
+        Some(p) => leer_entrada(p, TOPE_EXTRA)?,
+        None => String::new(),
+    };
+    let revisoras: Vec<Option<super::Validados>> =
+        entradas.iter().map(|p| leer_validados(p)).collect();
+    let v = super::veredicto(&revisoras);
+    let lotes = super::anotaciones(&v);
+    let (titulo, resumen) = (v.titulo(), super::resumen_veredicto(&v, &extra));
+    std::fs::create_dir_all(&dir).map_err(|e| fallo_io(&dir, e.to_string()))?;
+    let salida_de = |anotaciones: &[super::Anotacion]| {
+        serde_json::json!({
+            "title": titulo,
+            "summary": resumen,
+            "annotations": anotaciones,
+        })
+    };
+    let primero = lotes.first().map(Vec::as_slice).unwrap_or_default();
+    escribir_json(
+        &dir.join("check.json"),
+        &serde_json::json!({
+            "conclusion": v.dictamen.conclusion(),
+            "output": salida_de(primero),
+        }),
+    )?;
+    for (k, lote) in lotes.iter().enumerate().skip(1) {
+        escribir_json(
+            &dir.join(format!("anotaciones-{k}.json")),
+            &serde_json::json!({ "output": salida_de(lote) }),
+        )?;
+    }
+    Ok(Informe::nuevo(NOMBRE, Resultado::Ok)
+        .con("veredicto", v.dictamen.texto())
+        .con("conclusion", v.dictamen.conclusion())
+        .con("criticos", v.criticos.to_string())
+        .con("importantes", v.importantes.to_string())
+        .con("menores", v.menores.to_string())
+        .con("revisoras", v.filas.len().to_string())
+        .con("incompletas", v.incompletas().to_string())
+        .con("lotes", lotes.len().max(1).to_string()))
+}
diff --git a/crates/tc2/src/i18n/catalogo/hallazgos.rs b/crates/tc2/src/i18n/catalogo/hallazgos.rs
index bba2c01..6033788 100644
--- a/crates/tc2/src/i18n/catalogo/hallazgos.rs
+++ b/crates/tc2/src/i18n/catalogo/hallazgos.rs
@@ -2,8 +2,8 @@
 
 crate::i18n::textos! {
     USO_HALLAZGOS {
-        es: "uso: hallazgos [--json] validar --respuesta <fichero> --diff <fichero> --revisora <id> [--informe <informe.json>] [--salida <salida.json>] [--resumen <salida.md>] | hallazgos [--json] instrucciones",
-        en: "usage: hallazgos [--json] validar --respuesta <file> --diff <file> --revisora <id> [--informe <report.json>] [--salida <output.json>] [--resumen <output.md>] | hallazgos [--json] instrucciones",
+        es: "uso: hallazgos [--json] validar --respuesta <fichero> --diff <fichero> --revisora <id> [--informe <informe.json>] [--salida <salida.json>] [--resumen <salida.md>] | hallazgos [--json] veredicto --salida <carpeta> [--hallazgos <salida-de-validar.json>]... [--extra <lineas.txt>] | hallazgos [--json] instrucciones",
+        en: "usage: hallazgos [--json] validar --respuesta <file> --diff <file> --revisora <id> [--informe <report.json>] [--salida <output.json>] [--resumen <output.md>] | hallazgos [--json] veredicto --salida <folder> [--hallazgos <validar-output.json>]... [--extra <lines.txt>] | hallazgos [--json] instrucciones",
     }
     NO_SE_LEE {
         es: "no se puede leer ni escribir {ruta}: {error}; comprueba la ruta y los permisos y vuelve a lanzar la orden",
diff --git a/crates/tc2/src/i18n/catalogo/plataforma.rs b/crates/tc2/src/i18n/catalogo/plataforma.rs
index c55c71d..a4dc504 100644
--- a/crates/tc2/src/i18n/catalogo/plataforma.rs
+++ b/crates/tc2/src/i18n/catalogo/plataforma.rs
@@ -314,6 +314,18 @@ crate::i18n::textos! {
     }
 
     // --- aplicar-puertas (TASK-118) ---
+    LEER_REVISION_NO_SOPORTADA {
+        es: "esta plataforma no publica el check revision-ia (la revision IA en CI solo existe en GitHub): no hay informe que leer",
+        en: "this platform does not publish the revision-ia check (AI review in CI only exists on GitHub): there is no report to read",
+    }
+    REVISION_SIN_CHECK {
+        es: "el PR {pr} aun no tiene terminado el check revision-ia de la App {origen} en {sha}: espera a que acabe el workflow revision-ia y vuelve a lanzar tc2 review",
+        en: "PR {pr} does not yet have a finished revision-ia check from App {origen} on {sha}: wait for the revision-ia workflow to finish and run tc2 review again",
+    }
+    RAW_DETAILS_REVISION {
+        es: "el raw_details de una anotacion del check revision-ia, que no es un hallazgo de tc2 hallazgos veredicto",
+        en: "the raw_details of a revision-ia check annotation, which is not a finding from tc2 hallazgos veredicto",
+    }
     APLICAR_NO_SOPORTADA {
         es: "esta plataforma no admite aplicar las puertas desde tc2: aplicalas a mano con la propuesta de tc2 init --json",
         en: "this platform does not support applying the gates from tc2: apply them by hand with the proposal from tc2 init --json",
diff --git a/crates/tc2/src/plataforma/github.rs b/crates/tc2/src/plataforma/github.rs
index bedc67a..b795667 100644
--- a/crates/tc2/src/plataforma/github.rs
+++ b/crates/tc2/src/plataforma/github.rs
@@ -17,10 +17,12 @@ use super::errores::{self, es_aviso_de_plan};
 use super::http::{Base, Metodo, codificar};
 use super::puertas::{self, EstadoCheck, Lecturas, Origenes, Puerta};
 use super::{
-    Creada, EstadoPuerta, FalloLectura, Firma, ModeloPuertas, Paso, Personas, PlanObservado,
-    Plataforma, PrAbierto, Proveedor, PuertasLeidas, ReglasLeidas, Usuario, aplicar_en_lote,
+    Creada, EstadoPuerta, FalloLectura, Firma, LEER_REVISION, ModeloPuertas, Paso, Personas,
+    PlanObservado, Plataforma, PrAbierto, Proveedor, PuertasLeidas, ReglasLeidas,
+    RevisionPublicada, Usuario, aplicar_en_lote,
 };
 use crate::contrato::{Informe, Resultado, Valor, corto};
+use crate::i18n::Mensaje;
 use crate::i18n::catalogo::plataforma as t;
 
 const ABRIR_PR: &str = "abrir-pr";
@@ -68,6 +70,7 @@ enum Endpoint<'a> {
     CrearRuleset,
     CheckRuns(&'a str),
     CheckRunsDeRama(&'a str),
+    Anotaciones(u64),
     Reviews(u64),
     PermisosWorkflow,
     Commit(&'a str),
@@ -104,6 +107,10 @@ impl Endpoint<'_> {
                 Metodo::Get,
                 format!("{r}/commits/heads/{rama}/check-runs?per_page=100"),
             ),
+            Self::Anotaciones(id) => (
+                Metodo::Get,
+                format!("{r}/check-runs/{id}/annotations?per_page=100"),
+            ),
             Self::Reviews(n) => (Metodo::Get, format!("{r}/pulls/{n}/reviews?per_page=100")),
             Self::PermisosWorkflow => (Metodo::Get, format!("{r}/actions/permissions/workflow")),
             Self::Commit(sha) => (Metodo::Get, format!("{r}/commits/{sha}")),
@@ -184,12 +191,33 @@ struct CheckRunsApi {
 
 #[derive(Deserialize)]
 struct CheckRunApi {
+    /// Solo lo usa `leer-revision` (F6-5).
+    #[serde(default)]
+    id: u64,
     name: String,
     status: String,
     #[serde(default)]
     conclusion: Option<String>,
     #[serde(default)]
     app: Option<AppApi>,
+    /// Solo lo usa `leer-revision` (F6-5).
+    #[serde(default)]
+    output: Option<SalidaCheckApi>,
+}
+
+#[derive(Deserialize, Default)]
+struct SalidaCheckApi {
+    #[serde(default)]
+    title: Option<String>,
+    #[serde(default)]
+    summary: Option<String>,
+}
+
+/// Una anotacion de un check-run: solo su `raw_details` (F6-5).
+#[derive(Deserialize)]
+struct AnotacionApi {
+    #[serde(default)]
+    raw_details: Option<String>,
 }
 
 #[derive(Deserialize)]
@@ -582,6 +610,70 @@ impl Plataforma for GitHub {
         })
     }
 
+    fn leer_revision(&self, rol: Rol, pr: u64, origen: u64) -> Paso<RevisionPublicada> {
+        let op = LEER_REVISION;
+        if rol == Rol::Revisor {
+            return Err(rol_no_valido(op, rol));
+        }
+        let permiso = &t::PERMISO_PR_Y_CHECKS;
+        let p: PrApi = self
+            .con
+            .leer(op, rol, permiso, self.ep(&Endpoint::Pr(pr)))?;
+        if !sha_valido(&p.head.sha) {
+            return Err(self.con.invalida_por(op, &t::RAMA_BASE_O_SHA));
+        }
+        let (_, ruta) = self.ep(&Endpoint::CheckRuns(&p.head.sha));
+        let runs: Vec<CheckRunApi> = self.con.leer_paginas(op, rol, permiso, ruta, |c| {
+            self.con
+                .parsear::<CheckRunsApi>(op, c)
+                .map(|x| x.check_runs)
+        })?;
+        // Como `estado-puertas`: el mas nuevo de ese nombre que publico su
+        // origen, y solo si ya termino.
+        let check = runs
+            .into_iter()
+            .find(|r| {
+                r.name == crate::revision::REVISION_IA
+                    && r.app.as_ref().is_some_and(|a| a.id == origen)
+            })
+            .filter(|r| r.status == "completed" && r.id != 0);
+        let Some(check) = check else {
+            return Err(Informe::nuevo(op, Resultado::Esperando)
+                .con("pr", pr.to_string())
+                .con("motivo", "sin-check")
+                .diagnostico(
+                    Mensaje::nuevo(&t::REVISION_SIN_CHECK)
+                        .con("pr", pr.to_string())
+                        .con("origen", origen.to_string())
+                        .con("sha", corto(&p.head.sha)),
+                ));
+        };
+        let (_, ruta) = self.ep(&Endpoint::Anotaciones(check.id));
+        let crudos: Vec<Option<String>> = self.con.leer_paginas(op, rol, permiso, ruta, |c| {
+            self.con
+                .parsear::<Vec<AnotacionApi>>(op, c)
+                .map(|v| v.into_iter().map(|a| a.raw_details).collect())
+        })?;
+        // Cada `raw_details` es un aceptado que escribio `tc2 hallazgos
+        // veredicto`; uno que falta o no tiene esa forma falla cerrado.
+        let aceptados = crudos
+            .iter()
+            .map(|c| {
+                c.as_deref()
+                    .and_then(|c| serde_json::from_str::<crate::hallazgos::AceptadoDe>(c).ok())
+                    .ok_or_else(|| self.con.invalida_por(op, &t::RAW_DETAILS_REVISION))
+            })
+            .collect::<Paso<Vec<_>>>()?;
+        let salida = check.output.unwrap_or_default();
+        Ok(RevisionPublicada {
+            sha: p.head.sha,
+            conclusion: check.conclusion.unwrap_or_default(),
+            titulo: salida.title.unwrap_or_default(),
+            resumen: salida.summary.unwrap_or_default(),
+            aceptados,
+        })
+    }
+
     fn merge_pr(&self, pr: u64, sha: &str) -> Informe {
         if !sha_valido(sha) {
             return uso(MERGE_PR, "sha-invalido", &t::SHA_ESPERADO_COMPLETO);
diff --git a/crates/tc2/src/plataforma/mod.rs b/crates/tc2/src/plataforma/mod.rs
index 5d7e53a..af0f062 100644
--- a/crates/tc2/src/plataforma/mod.rs
+++ b/crates/tc2/src/plataforma/mod.rs
@@ -274,6 +274,26 @@ pub enum PlanObservado {
     GitLab(Option<TierGitLab>),
 }
 
+/// Nombre de la operacion `leer-revision` (adenda F6-5 a F1-4 y F1-13).
+pub const LEER_REVISION: &str = "leer-revision";
+
+/// Lo que `leer-revision` lee del check `revision-ia` terminado (F6-5): la
+/// conclusion, el titulo y el resumen del check, y los aceptados de sus
+/// anotaciones (`raw_details`).
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct RevisionPublicada {
+    /// Sha de la cabeza del PR en la que esta el check (40 hex).
+    pub sha: String,
+    /// `success` o `failure` (o lo que diga la plataforma).
+    pub conclusion: String,
+    /// `output.title`.
+    pub titulo: String,
+    /// `output.summary`.
+    pub resumen: String,
+    /// Uno por anotacion, en el orden de la plataforma.
+    pub aceptados: Vec<crate::hallazgos::AceptadoDe>,
+}
+
 /// Un PR (o MR) abierto del propio repo, con lo minimo que necesita `board`
 /// (F3-5, D3 de TASK-032: sin checks ni aprobaciones).
 #[derive(Debug, Clone, PartialEq, Eq)]
@@ -419,6 +439,21 @@ pub trait Plataforma {
         .diagnostico(&crate::i18n::catalogo::plataforma::ENTORNO_PUSH_NO_SOPORTADO))
     }
 
+    /// `leer-revision` (agente o persona; adenda F6-5 a F1-4 y F1-13): el
+    /// check `revision-ia` que publico `origen` en la cabeza del PR `pr`, ya
+    /// terminado, con los aceptados de sus anotaciones. Si aun no esta,
+    /// `esperando motivo=sin-check`. Por defecto (GitLab, donde la revision
+    /// en CI no existe, y los adaptadores de prueba), `rechazada
+    /// motivo=no-soportada`.
+    fn leer_revision(&self, rol: Rol, pr: u64, origen: u64) -> Paso<RevisionPublicada> {
+        let _ = (rol, pr, origen);
+        Err(
+            Informe::nuevo(LEER_REVISION, crate::contrato::Resultado::Rechazada)
+                .con("motivo", "no-soportada")
+                .diagnostico(&crate::i18n::catalogo::plataforma::LEER_REVISION_NO_SOPORTADA),
+        )
+    }
+
     /// `estado-puertas` (agente o persona): checks con su origen,
     /// aprobaciones de personas y rama al dia.
     fn estado_puertas(&self, rol: Rol, pr: u64, modelo: &ModeloPuertas) -> Informe;
diff --git a/crates/tc2/src/revision/guardar.rs b/crates/tc2/src/revision/guardar.rs
new file mode 100644
index 0000000..4279bcd
--- /dev/null
+++ b/crates/tc2/src/revision/guardar.rs
@@ -0,0 +1,120 @@
+//! El informe de la revision IA en la carpeta de la tarea (F6-5), como
+//! operacion de la rama de estado (precedente: `plan guardar`, F4-1).
+//!
+//! `tc2 review` lee el check `revision-ia` ya terminado
+//! ([`crate::plataforma::Plataforma::leer_revision`], con el agente) y lo
+//! publica en `TASK-NNN/revision/revision-ia-<sha12>.json`
+//! (`$defs/informe-tarea` de `hallazgos.schema.json`) y `.md` (generado por
+//! tc2 con el veredicto y la tabla de aceptados por `plano`, sin la prosa del
+//! modelo). La CI no escribe en la rama de estado: no tiene la identidad del
+//! agente (F1-6, F1-8).
+//!
+//! - Solo con la tarea `en-revision` y de quien la lleva (la regla de
+//!   `review`).
+//! - El mismo contenido que ya esta publicado: `sin-cambios`. No toca
+//!   `tarea.md`.
+//! - Exige red: el contenido no cabe en la anotacion.
+
+use std::cell::RefCell;
+
+use crate::estado::{Cambio, Estado, Fallo, Operacion, de_otra_persona, por_estado, tarea_de};
+use crate::hallazgos::InformeTarea;
+use crate::i18n::catalogo::revision as txt;
+use crate::identidad::Quien;
+use crate::tarea::Asignado;
+
+use super::{EN_REVISION, NOMBRE};
+
+/// Nombre de la operacion (en el commit y en la anotacion).
+pub const NOMBRE_OP: &str = "revision-guardar";
+
+/// Carpeta de la tarea en la que se guarda.
+pub const CARPETA: &str = "revision";
+
+/// Publicar el informe de la revision IA de una tarea.
+#[derive(Debug)]
+pub struct GuardarRevision {
+    id: String,
+    asignado: Asignado,
+    sha12: String,
+    json: String,
+    md: String,
+    dueno: RefCell<Option<String>>,
+}
+
+impl GuardarRevision {
+    /// Guardar `informe` en la carpeta de la tarea `id`, por `quien`.
+    pub fn nueva(id: &str, quien: &Quien, informe: &InformeTarea) -> Self {
+        let json = serde_json::to_string_pretty(informe).unwrap_or_default() + "\n";
+        Self {
+            id: id.to_owned(),
+            asignado: quien.asignado(),
+            sha12: informe.sha.chars().take(12).collect(),
+            json,
+            md: informe.markdown(),
+            dueno: RefCell::new(None),
+        }
+    }
+
+    /// Ruta del JSON en la rama de estado.
+    pub fn ruta_json(&self) -> String {
+        format!("{}/{CARPETA}/revision-ia-{}.json", self.id, self.sha12)
+    }
+
+    fn ruta_md(&self) -> String {
+        format!("{}/{CARPETA}/revision-ia-{}.md", self.id, self.sha12)
+    }
+
+    /// Login de quien lleva la tarea si el ultimo intento la rechazo por
+    /// `de-otra-persona`.
+    pub fn dueno(&self) -> Option<String> {
+        self.dueno.borrow().clone()
+    }
+}
+
+impl Operacion for GuardarRevision {
+    fn nombre(&self) -> &str {
+        NOMBRE_OP
+    }
+
+    fn anotacion(&self) -> String {
+        format!(
+            "{NOMBRE_OP}|{}|{}|{}|{}",
+            self.id,
+            self.asignado.id().unwrap_or_default(),
+            self.asignado.login().unwrap_or_default(),
+            self.sha12
+        )
+    }
+
+    fn exige_red(&self) -> bool {
+        // El informe no cabe en la anotacion: sin red falla, nunca queda
+        // pendiente.
+        true
+    }
+
+    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
+        *self.dueno.borrow_mut() = None;
+        let (t, _) = tarea_de(estado, &self.id)?;
+        if let Some((fallo, dueno)) = de_otra_persona(
+            &self.id,
+            t.asignado_a.as_ref(),
+            &self.asignado,
+            NOMBRE,
+            ("de-otra-persona", &txt::DE_OTRA_PERSONA),
+        ) {
+            *self.dueno.borrow_mut() = dueno;
+            return Err(fallo);
+        }
+        if t.estado != EN_REVISION {
+            return Err(por_estado(&self.id, &t.estado, NOMBRE, &[EN_REVISION]));
+        }
+        let mut cambios = Vec::new();
+        for (ruta, contenido) in [(self.ruta_json(), &self.json), (self.ruta_md(), &self.md)] {
+            if estado.leer(&ruta)?.as_deref() != Some(contenido.as_str()) {
+                cambios.push(Cambio::nuevo(ruta, contenido.clone()));
+            }
+        }
+        Ok(cambios)
+    }
+}
diff --git a/crates/tc2/src/revision/mod.rs b/crates/tc2/src/revision/mod.rs
index aeb9b9d..a00d228 100644
--- a/crates/tc2/src/revision/mod.rs
+++ b/crates/tc2/src/revision/mod.rs
@@ -17,6 +17,10 @@
 //!    revision» sin PR. Si falla, la linea lleva el `pr=` y repetir converge.
 //! 4. [`Plataforma::estado_puertas`] con el agente para leer `revision-ia`
 //!    (D1: se lee y se informa; lo publica el revisor en F6-3).
+//! 5. F6-5: si `revision-ia` ya termino (`ok` o `fallo`),
+//!    [`Plataforma::leer_revision`] con el agente y el informe en la carpeta
+//!    de la tarea por el punto unico de escritura ([`GuardarRevision`]:
+//!    `TASK-NNN/revision/revision-ia-<sha12>.json` y `.md`).
 //!
 //! «PR enlazado a la tarea» sin tocar el esquema (F1-9): titulo
 //! `TASK-NNN: <titulo>` y cuerpo de [`cuerpo_pr`] con las rutas de la tarea
@@ -24,11 +28,13 @@
 //! rama.
 //!
 //! Linea: `review ok id=TASK-NNN rama=<r> pr=<n> estado=en-revision
-//! revision-ia=<ok|pendiente|fallo|sin-revisor|sin-leer> revisoras=<ids>
+//! revision-ia=<ok|pendiente|fallo|sin-revisor|sin-leer>
+//! [hallazgos=<rojo|verde|incompleta|sin-leer> [informe=<ruta>]] revisoras=<ids>
 //! modelo=<m> reglas=<ids> avisos=<...>` (la seleccion de F6-3, anotada en
 //! `roles.revision`). Repetirla sin
 //! nada nuevo sale `sin-cambios`.
 
+pub mod guardar;
 pub mod operacion;
 
 use std::path::Path;
@@ -45,6 +51,7 @@ use crate::seleccion;
 use crate::tarea::Tarea;
 use crate::toma;
 
+pub use guardar::GuardarRevision;
 pub use operacion::{AnotacionRevision, EN_REVISION, Revisar, desde_anotacion};
 
 /// Nombre de la orden y de la operacion de la rama de estado.
@@ -316,19 +323,63 @@ fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
         .and_then(|c| c.plataforma)
         .and_then(|p| p.revisor_id);
     let ria = revision_ia(p.as_ref(), publicado.pr, revisor);
-    let nada_nuevo =
-        !publicado.empujado && !publicado.abierto && tr.resultado == Resultado::SinCambios;
+    // 8. F6-5: con el check terminado, su informe en la carpeta de la tarea.
+    let mut hallazgos: Option<(&'static str, Option<String>)> = None;
+    let mut guardado = false;
+    if let (Some(origen), "ok" | "fallo") = (revisor.filter(|id| *id != 0), ria) {
+        let leido = p
+            .leer_revision(Rol::Agente, publicado.pr, origen)
+            .ok()
+            .and_then(|r| {
+                let d = crate::hallazgos::Dictamen::de_titulo(&r.titulo)?;
+                Some(crate::hallazgos::InformeTarea::nuevo(
+                    id,
+                    publicado.pr,
+                    &r.sha,
+                    (&r.conclusion, d),
+                    &r.resumen,
+                    r.aceptados,
+                ))
+            });
+        hallazgos = Some(("sin-leer", None));
+        if let Some(informe) = leido {
+            let op = GuardarRevision::nueva(id, &quien, &informe);
+            let gr = estado::transaccion(&ctx, &op);
+            match gr.resultado {
+                Resultado::Ok => guardado = true,
+                Resultado::SinCambios => {}
+                _ => {
+                    let mut gr = gr.con("rama", rama.as_str()).con("pr", pr);
+                    if let Some(l) = op.dueno() {
+                        gr = gr.con("asignado", l);
+                    }
+                    return Err(gr);
+                }
+            }
+            hallazgos = Some((informe.veredicto.texto(), Some(op.ruta_json())));
+        }
+    }
+    let nada_nuevo = !publicado.empujado
+        && !publicado.abierto
+        && !guardado
+        && tr.resultado == Resultado::SinCambios;
     let resultado = if nada_nuevo {
         Resultado::SinCambios
     } else {
         Resultado::Ok
     };
-    Ok(Informe::nuevo(NOMBRE, resultado)
+    let mut i = Informe::nuevo(NOMBRE, resultado)
         .con("rama", rama.as_str())
         .con("pr", pr)
         .con("estado", EN_REVISION)
-        .con(REVISION_IA, ria)
-        .con_lista("revisoras", sel.revisoras)
+        .con(REVISION_IA, ria);
+    if let Some((veredicto, ruta)) = hallazgos {
+        i = i.con("hallazgos", veredicto);
+        if let Some(r) = ruta {
+            i = i.con("informe", r);
+        }
+    }
+    Ok(i.con_lista("revisoras", sel.revisoras)
         .con("modelo", sel.modelo)
         .con_lista("reglas", sel.reglas)
         .con_lista("avisos", sel.avisos))
diff --git a/xtask/src/workflows.rs b/xtask/src/workflows.rs
index f657909..c432c30 100644
--- a/xtask/src/workflows.rs
+++ b/xtask/src/workflows.rs
@@ -2125,7 +2125,7 @@ fn artefactos_subidos(raiz: &Nodo, job: &str) -> Vec<String> {
 
 /// `validar` sube exactamente el artefacto `revision-ia-hallazgos` y
 /// `publicar` lee ese mismo nombre; si no casan, el check sale siempre
-/// `neutral` con `sin-hallazgos` y solo el E2E lo veria.
+/// `failure` con veredicto `incompleta` y solo el E2E lo veria.
 fn validar_sube_lo_que_lee_publicar(texto: &str) -> Result<(), String> {
     const NOMBRE: &str = "revision-ia-hallazgos";
     let raiz = leer_yaml(texto)?;
@@ -2146,6 +2146,63 @@ fn validar_sube_lo_que_lee_publicar(texto: &str) -> Result<(), String> {
     Ok(())
 }
 
+/// Conclusiones que admite la API de check-runs.
+const CONCLUSIONES: [&str; 8] = [
+    "action_required",
+    "cancelled",
+    "failure",
+    "neutral",
+    "success",
+    "skipped",
+    "stale",
+    "timed_out",
+];
+
+/// Los `run` de un job, juntos.
+fn runs_del_job(raiz: &Nodo, job: &str) -> String {
+    let mut valores = Vec::new();
+    if let Some(j) = raiz.get("jobs").and_then(|j| j.get(job)) {
+        j.escalares("", &mut valores);
+    }
+    valores
+        .iter()
+        .filter(|(k, _)| *k == "run")
+        .map(|(_, v)| *v)
+        .collect::<Vec<_>>()
+        .join("\n")
+}
+
+/// F6-5: la conclusion la calcula `tc2 hallazgos veredicto` en `validar` y
+/// `publicar` solo la transporta: lee `check.json` del artefacto, las
+/// unicas conclusiones que nombra son `success` y `failure` (nunca
+/// `neutral`, que GitHub da por cumplido) y `success` no aparece como
+/// literal (solo puede venir de `check.json`; el literal `failure` es el de
+/// la revision incompleta sin artefacto).
+fn publicar_transporta_el_veredicto(texto: &str) -> Result<(), String> {
+    let raiz = leer_yaml(texto)?;
+    if !runs_del_job(&raiz, "validar").contains("tc2 hallazgos veredicto ") {
+        return Err("validar no corre tc2 hallazgos veredicto".to_owned());
+    }
+    let run = runs_del_job(&raiz, "publicar");
+    if !run.contains("jq -r '.conclusion // \"\"' \"$dir/check.json\"") {
+        return Err("publicar no lee check.json del artefacto".to_owned());
+    }
+    let palabras: BTreeSet<&str> = run
+        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
+        .filter(|p| CONCLUSIONES.contains(p))
+        .collect();
+    if palabras != BTreeSet::from(["failure", "success"]) {
+        return Err(format!(
+            "publicar nombra las conclusiones {palabras:?}; solo se admiten success y failure"
+        ));
+    }
+    let literal = run.replace(' ', "");
+    if literal.contains("conclusion:\"success\"") {
+        return Err("publicar pone success como literal: tiene que venir de check.json".to_owned());
+    }
+    Ok(())
+}
+
 /// El workflow del revisor pasa el lint con el perfil revision-ia: cuatro
 /// jobs (F6-4); `consultar` y `publicar` en el entorno `revisor`, sin
 /// checkout, cada uno solo con su secreto; `preparar` y `validar` sin
@@ -2211,6 +2268,7 @@ fn el_workflow_del_revisor_pasa_el_lint() {
         ["revision-ia-hallazgos"]
     );
     assert_eq!(validar_sube_lo_que_lee_publicar(&texto), Ok(()));
+    assert_eq!(publicar_transporta_el_veredicto(&texto), Ok(()));
     assert!(
         runs_de("preparar")
             .iter()
@@ -2350,6 +2408,42 @@ fn cada_mutacion_del_revisor_da_su_error() {
             "«{esperado}» no salio: {r:?}"
         );
     }
+    // F6-5: una mutacion por regla del veredicto da rojo.
+    let veredicto: [(&str, &str, &str); 5] = [
+        (
+            "          tc2 hallazgos veredicto --salida",
+            "          tc2 hallazgos resumen --salida",
+            "validar no corre tc2 hallazgos veredicto",
+        ),
+        (
+            "[ -s \"$dir/check.json\" ]; then\n            conclusion=$(jq -r '.conclusion // \"\"' \"$dir/check.json\")",
+            "[ -s \"$dir/resumen.txt\" ]; then\n            conclusion=$(jq -r '.conclusion // \"\"' \"$dir/resumen.json\")",
+            "publicar no lee check.json del artefacto",
+        ),
+        (
+            "            success|failure) cp",
+            "            success|failure|neutral) cp",
+            "solo se admiten success y failure",
+        ),
+        (
+            "{conclusion: \"failure\", output: {title: \"revision-ia: incompleta",
+            "{conclusion: \"neutral\", output: {title: \"revision-ia: incompleta",
+            "solo se admiten success y failure",
+        ),
+        (
+            "{conclusion: \"failure\", output: {title: \"revision-ia: incompleta",
+            "{conclusion: \"success\", output: {title: \"revision-ia: incompleta",
+            "success como literal",
+        ),
+    ];
+    for (original, mutado, esperado) in veredicto {
+        assert!(texto.contains(original), "el caso no casa: {original}");
+        let r = publicar_transporta_el_veredicto(&texto.replacen(original, mutado, 1));
+        assert!(
+            r.as_ref().is_err_and(|e| e.contains(esperado)),
+            "«{esperado}» no salio: {r:?}"
+        );
+    }
 }
 
 /// El llamador de la guia de F6-3 pasa el lint de su perfil; `secrets:
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-051/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-051/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-051/tarea.md                                    | 3 ++-
 3 files changed, 2 insertions(+), 1 deletion(-)
````

## Tests del diff (4 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/hallazgos.rs" ":(literal)crates/tc2/tests/plataforma.rs" ":(literal)crates/tc2/tests/plataforma_e2e.rs" ":(literal)crates/tc2/tests/revision.rs"
````

````
crates/tc2/tests/hallazgos.rs      | 259 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++--
 crates/tc2/tests/plataforma.rs     | 132 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-
 crates/tc2/tests/plataforma_e2e.rs | 201 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/revision.rs       | 159 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++--
 4 files changed, 745 insertions(+), 6 deletions(-)
````
