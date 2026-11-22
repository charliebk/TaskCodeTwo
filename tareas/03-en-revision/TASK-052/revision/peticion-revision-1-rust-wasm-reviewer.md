# Peticion de revision — TASK-052 (ronda 1)

- Tarea: TASK-052 — F6-6 Rondas incrementales
- Rama revisada: feature/task-052-f6-6-rondas-incrementales
- Rama base: develop
- Commit revisado (HEAD): 8822e8ee1e21657de2fd037cc0d334b27fe58006
- Fecha: 2026-11-22
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-052 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-052/tarea.md
- tareas/03-en-revision/TASK-052/planificacion/
- crates/tc2/src/hallazgos/mod.rs
- crates/tc2/src/hallazgos/orden.rs
- crates/tc2/src/hallazgos/ronda.rs
- crates/tc2/src/i18n/catalogo/hallazgos.rs
- crates/tc2/src/i18n/catalogo/seleccion.rs
- crates/tc2/src/plataforma/github.rs
- crates/tc2/src/revision/mod.rs
- crates/tc2/src/seleccion/mod.rs
- crates/tc2/src/seleccion/orden.rs
- crates/tc2/tests/hallazgos.rs
- crates/tc2/tests/plataforma_e2e.rs
- crates/tc2/tests/revision.rs
- crates/tc2/tests/revisoras.rs
- xtask/src/workflows.rs

## Commits a revisar (git log develop..HEAD)

````
8822e8e feat(TASK-052): review guarda el informe del head anterior y adendas F6-6
2015305 feat(TASK-052): workflow revision-ia con ronda anterior e incremental
21709fa feat(TASK-052): tc2 revisoras --ronda para aplicar S13
5d8ad46 feat(TASK-052): ronda siguiente desde el check anterior y ordenes de rondas
b70d3b3 feat(TASK-052): contrato v3 de hallazgos con rondas y cierres
8084d7c chore(TASK-052): tarea en curso
````

## Diff de tu dominio (14 fichero(s) de 23; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/hallazgos/mod.rs b/crates/tc2/src/hallazgos/mod.rs
index d1ef6e9..fa929bb 100644
--- a/crates/tc2/src/hallazgos/mod.rs
+++ b/crates/tc2/src/hallazgos/mod.rs
@@ -23,8 +23,18 @@
 //! CRITICO o IMPORTANTE aceptado o si alguna revisora no dejo respuesta
 //! valida; `success` si no), su resumen ([`resumen_veredicto`]) y las
 //! anotaciones por linea ([`anotaciones`]). Tambien es una funcion pura.
+//!
+//! F6-6 (contrato v3): rondas incrementales. Cada check describe su ronda
+//! (titulo con `ronda=`) y lleva en sus anotaciones todos los hallazgos que
+//! siguen abiertos ([`AceptadoDe`] con `ronda` y `estado`). [`ronda`] decide
+//! la ronda siguiente a partir del check anterior; en modo incremental cada
+//! revisora recibe sus abiertos y contesta con un objeto `{hallazgos,
+//! cierres}`: un abierto CRITICO o IMPORTANTE solo se cierra con un
+//! [`Cierre`] valido ([`validar_con`]); si no, sigue abierto (falla en
+//! cerrado). [`veredicto_ronda`] cuenta nuevos, persisten y cerrados.
 
 pub mod orden;
+pub mod ronda;
 
 use std::collections::{BTreeMap, BTreeSet};
 
@@ -34,7 +44,7 @@ use serde_json::Value;
 use crate::verificacion::informe::{Informe as InformeVerificacion, Resultado as ResultadoPaso};
 
 /// Version del contrato de hallazgos.
-pub const VERSION: u32 = 2;
+pub const VERSION: u32 = 3;
 
 /// Hallazgos que se validan como mucho por respuesta; los que sobran se
 /// rechazan juntos con [`motivo::FUERA_DE_TOPE`].
@@ -68,11 +78,18 @@ pub mod motivo {
     pub const BORDE_SIN_DIFERENCIA: &str = "borde-sin-diferencia";
     /// `paso_rojo` no es un paso en rojo del informe de verificacion.
     pub const PASO_ROJO_NO_ESTA: &str = "paso-rojo-no-esta";
+    /// El cierre nombra un id que no es de un abierto de la revisora (F6-6).
+    pub const CIERRE_DESCONOCIDO: &str = "cierre-desconocido";
+    /// El fichero o la linea del cierre no estan en el diff incremental.
+    pub const CIERRE_FUERA_DEL_DIFF: &str = "cierre-fuera-del-diff";
+    /// El abierto traia `paso_rojo` y ese paso sigue en rojo (o no hay
+    /// informe de verificacion).
+    pub const PASO_ROJO_SIGUE_ROJO: &str = "paso-rojo-sigue-rojo";
 }
 
 /// Los motivos de rechazo con su explicacion, en el orden en que se
 /// comprueban. Van tal cual al prompt ([`instrucciones`]) y al esquema.
-pub const MOTIVOS: [(&str, &str); 13] = [
+pub const MOTIVOS: [(&str, &str); 16] = [
     (
         motivo::SIN_BLOQUE,
         "la respuesta no trae ningun bloque json",
@@ -83,7 +100,7 @@ pub const MOTIVOS: [(&str, &str); 13] = [
     ),
     (
         motivo::JSON_INVALIDO,
-        "el bloque no se cierra, no es JSON o no es una lista",
+        "el bloque no se cierra, no es JSON, o no es una lista ni un objeto {hallazgos, cierres}",
     ),
     (
         motivo::FUERA_DE_TOPE,
@@ -119,6 +136,18 @@ pub const MOTIVOS: [(&str, &str); 13] = [
         motivo::PASO_ROJO_NO_ESTA,
         "paso_rojo no es un paso en rojo del informe de verificacion, o no hay informe",
     ),
+    (
+        motivo::CIERRE_DESCONOCIDO,
+        "el cierre nombra un id que no es de un hallazgo CRITICO o IMPORTANTE abierto de tu lista",
+    ),
+    (
+        motivo::CIERRE_FUERA_DEL_DIFF,
+        "el fichero o la linea del cierre no estan en el lado nuevo del diff incremental que ves",
+    ),
+    (
+        motivo::PASO_ROJO_SIGUE_ROJO,
+        "el abierto traia paso_rojo y ese paso sigue en rojo en el informe de verificacion, o no hay informe",
+    ),
 ];
 
 /// Severidad de un hallazgo (las de la revision por pares de TaskCode).
@@ -280,8 +309,32 @@ pub struct Aceptado {
     pub evidencia: Evidencia,
 }
 
+/// Un cierre de un abierto (F6-6), tal como lo da la revisora: el id del
+/// abierto y la linea del lado nuevo del diff incremental que lo arregla.
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct Cierre {
+    pub id: String,
+    pub fichero: String,
+    pub linea: u32,
+    pub motivo: String,
+}
+
+/// Un cierre aceptado: el [`Cierre`] con la revisora y la ronda del abierto
+/// que cierra (un cierre cierra todos los abiertos de la lista con su id).
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct CierreAceptado {
+    pub revisora: String,
+    pub ronda: u32,
+    pub id: String,
+    pub fichero: String,
+    pub linea: u32,
+    pub motivo: String,
+}
+
 /// Un rechazo: de un hallazgo (`indice` en la lista) o de toda la respuesta
-/// (`indice` nulo).
+/// (`indice` nulo). Los cierres se numeran detras de los hallazgos.
 #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
 pub struct Rechazo {
     pub indice: Option<usize>,
@@ -301,6 +354,9 @@ pub struct Validados {
     pub revisora: String,
     pub aceptados: Vec<Aceptado>,
     pub rechazados: Vec<Rechazo>,
+    /// Los cierres aceptados (F6-6; vacio en la ronda 1).
+    #[serde(default)]
+    pub cierres: Vec<CierreAceptado>,
 }
 
 /// Un [`Rechazo`] leido de vuelta, con el motivo aun sin contrastar.
@@ -672,7 +728,10 @@ fn validar_uno(
     if !vistos.insert(c.id.clone()) {
         return Err(fallo(
             motivo::ID_REPETIDO,
-            format!("el id {} ya lo usa un hallazgo anterior", c.id),
+            format!(
+                "el id {} ya lo usa un hallazgo anterior o un abierto de la ronda anterior",
+                c.id
+            ),
         ));
     }
     let e: Evidencia = serde_json::from_value(evidencia.clone())
@@ -712,21 +771,16 @@ fn validar_uno(
             "esperado y obtenido son iguales: no hay borde".to_owned(),
         ));
     }
-    if let Some(paso) = &e.paso_rojo {
-        let en_rojo = informe.is_some_and(|i| {
-            i.pasos
-                .iter()
-                .any(|p| p.nombre == paso.trim() && p.resultado == ResultadoPaso::Rojo)
-        });
-        if !en_rojo {
-            return Err(fallo(
-                motivo::PASO_ROJO_NO_ESTA,
-                format!(
-                    "el paso {} no figura en rojo en el informe de verificacion",
-                    paso.trim()
-                ),
-            ));
-        }
+    if let Some(paso) = &e.paso_rojo
+        && !en_rojo(informe, paso)
+    {
+        return Err(fallo(
+            motivo::PASO_ROJO_NO_ESTA,
+            format!(
+                "el paso {} no figura en rojo en el informe de verificacion",
+                paso.trim()
+            ),
+        ));
     }
     Ok(Aceptado {
         id: c.id,
@@ -740,18 +794,115 @@ fn validar_uno(
 }
 
 /// Valida la respuesta de la revisora `revisora` contra el diff que vio y,
-/// si lo hay, el informe de verificacion (para `paso_rojo`).
+/// si lo hay, el informe de verificacion (para `paso_rojo`). Ronda 1: sin
+/// abiertos ([`validar_con`]).
 pub fn validar(
     revisora: &str,
     respuesta: &str,
     diff: &Diff,
     informe: Option<&InformeVerificacion>,
+) -> Validados {
+    validar_con(revisora, respuesta, diff, informe, &[])
+}
+
+/// Si `paso` figura en rojo en el informe.
+fn en_rojo(informe: Option<&InformeVerificacion>, paso: &str) -> bool {
+    informe.is_some_and(|i| {
+        i.pasos
+            .iter()
+            .any(|p| p.nombre == paso.trim() && p.resultado == ResultadoPaso::Rojo)
+    })
+}
+
+/// Valida un cierre contra los abiertos CRITICO o IMPORTANTE de la revisora,
+/// el diff incremental y el informe; da los abiertos que cierra.
+fn validar_cierre<'a>(
+    c: &Value,
+    diff: &Diff,
+    informe: Option<&InformeVerificacion>,
+    abiertos: &'a [AceptadoDe],
+    vistos: &mut BTreeSet<String>,
+) -> Result<(Cierre, Vec<&'a AceptadoDe>), Fallo> {
+    let id = c
+        .get("id")
+        .and_then(Value::as_str)
+        .filter(|s| es_id(s))
+        .map(str::to_owned);
+    let fallo = |motivo: &'static str, detalle: String| Fallo {
+        id: id.clone(),
+        motivo,
+        detalle,
+    };
+    let c: Cierre = serde_json::from_value(c.clone())
+        .map_err(|e| fallo(motivo::HALLAZGO_INVALIDO, format!("cierre: {e}")))?;
+    if !es_id(&c.id) || c.linea == 0 || !no_vacio(&c.fichero) || !no_vacio(&c.motivo) {
+        return Err(fallo(
+            motivo::HALLAZGO_INVALIDO,
+            "cierre: id valido, linea desde 1, y fichero y motivo no vacios".to_owned(),
+        ));
+    }
+    if !vistos.insert(c.id.clone()) {
+        return Err(fallo(
+            motivo::ID_REPETIDO,
+            format!("otro cierre anterior ya cierra {}", c.id),
+        ));
+    }
+    let cerrados: Vec<&AceptadoDe> = abiertos
+        .iter()
+        .filter(|a| a.hallazgo.id == c.id && a.hallazgo.severidad != Severidad::Menor)
+        .collect();
+    if cerrados.is_empty() {
+        return Err(fallo(
+            motivo::CIERRE_DESCONOCIDO,
+            format!(
+                "{} no es un abierto CRITICO o IMPORTANTE de esta revisora",
+                c.id
+            ),
+        ));
+    }
+    if diff.linea(&c.fichero, c.linea).is_none() {
+        return Err(fallo(
+            motivo::CIERRE_FUERA_DEL_DIFF,
+            format!(
+                "{}:{} no esta en el lado nuevo del diff incremental",
+                c.fichero, c.linea
+            ),
+        ));
+    }
+    for a in &cerrados {
+        if let Some(paso) = &a.hallazgo.evidencia.paso_rojo
+            && (informe.is_none() || en_rojo(informe, paso))
+        {
+            return Err(fallo(
+                motivo::PASO_ROJO_SIGUE_ROJO,
+                format!(
+                    "el paso {} sigue en rojo en el informe de verificacion, o no hay informe",
+                    paso.trim()
+                ),
+            ));
+        }
+    }
+    Ok((c, cerrados))
+}
+
+/// Como [`validar`], con los abiertos de la ronda anterior que se le dieron a
+/// esta revisora (F6-6). La respuesta puede ser la lista de hallazgos (v2) o
+/// un objeto `{"hallazgos": [...], "cierres": [...]}`; un hallazgo nuevo no
+/// puede reutilizar el id de un abierto, y cada cierre se valida contra los
+/// abiertos CRITICO o IMPORTANTE, el diff (el incremental) y el informe.
+pub fn validar_con(
+    revisora: &str,
+    respuesta: &str,
+    diff: &Diff,
+    informe: Option<&InformeVerificacion>,
+    abiertos: &[AceptadoDe],
 ) -> Validados {
     let mut v = Validados {
         version: VERSION,
         revisora: revisora.to_owned(),
         aceptados: Vec::new(),
         rechazados: Vec::new(),
+        cierres: Vec::new(),
     };
     let de_toda = |motivo: &'static str, detalle: String| Rechazo {
         indice: None,
@@ -759,17 +910,33 @@ pub fn validar(
         motivo,
         detalle,
     };
-    let lista = match bloque_json(respuesta).map(|b| serde_json::from_str::<Value>(&b)) {
+    let forma = "el bloque json no es una lista ni un objeto {hallazgos, cierres}";
+    let bloque = bloque_json(respuesta).map(|b| serde_json::from_str::<Value>(&b));
+    let (lista, cierres) = match bloque {
         Err((m, d)) => {
             v.rechazados.push(de_toda(m, d));
             return v;
         }
-        Ok(Ok(Value::Array(l))) => l,
+        Ok(Ok(Value::Array(l))) => (l, Vec::new()),
+        Ok(Ok(Value::Object(o))) => {
+            let lista = |k: &str| match o.get(k) {
+                Some(Value::Array(l)) => Some(l.clone()),
+                None if k == "cierres" => Some(Vec::new()),
+                _ => None,
+            };
+            let solo_esas = o.keys().all(|k| k == "hallazgos" || k == "cierres");
+            match (lista("hallazgos"), lista("cierres")) {
+                (Some(h), Some(c)) if solo_esas => (h, c),
+                _ => {
+                    v.rechazados
+                        .push(de_toda(motivo::JSON_INVALIDO, forma.to_owned()));
+                    return v;
+                }
+            }
+        }
         Ok(Ok(_)) => {
-            v.rechazados.push(de_toda(
-                motivo::JSON_INVALIDO,
-                "el bloque json no es una lista".to_owned(),
-            ));
+            v.rechazados
+                .push(de_toda(motivo::JSON_INVALIDO, forma.to_owned()));
             return v;
         }
         Ok(Err(e)) => {
@@ -780,7 +947,8 @@ pub fn validar(
             return v;
         }
     };
-    let mut vistos = BTreeSet::new();
+    // Los ids de los abiertos no se reutilizan: un cierre los nombra.
+    let mut vistos: BTreeSet<String> = abiertos.iter().map(|a| a.hallazgo.id.clone()).collect();
     for (i, h) in lista.iter().enumerate() {
         if i >= TOPE_HALLAZGOS {
             v.rechazados.push(Rechazo {
@@ -804,6 +972,42 @@ pub fn validar(
             }),
         }
     }
+    let mut cerrados_vistos = BTreeSet::new();
+    for (j, c) in cierres.iter().enumerate() {
+        let indice = lista.len() + j;
+        if j >= TOPE_HALLAZGOS {
+            v.rechazados.push(Rechazo {
+                indice: Some(indice),
+                id: None,
+                motivo: motivo::FUERA_DE_TOPE,
+                detalle: format!(
+                    "{} cierres pasan del tope de {TOPE_HALLAZGOS}",
+                    cierres.len() - TOPE_HALLAZGOS
+                ),
+            });
+            break;
+        }
+        match validar_cierre(c, diff, informe, abiertos, &mut cerrados_vistos) {
+            Ok((c, cerrados)) => {
+                for a in cerrados {
+                    v.cierres.push(CierreAceptado {
+                        revisora: a.revisora.clone(),
+                        ronda: a.ronda,
+                        id: c.id.clone(),
+                        fichero: c.fichero.clone(),
+                        linea: c.linea,
+                        motivo: c.motivo.clone(),
+                    });
+                }
+            }
+            Err(f) => v.rechazados.push(Rechazo {
+                indice: Some(indice),
+                id: f.id,
+                motivo: f.motivo,
+                detalle: f.detalle,
+            }),
+        }
+    }
     v
 }
 
@@ -880,11 +1084,24 @@ El check `revision-ia` lista los aceptados y los rechazados con su motivo.\n\n",
         "  - `borde`: `entrada`, `esperado` (lo que deberia salir) y `obtenido` (lo que sale segun el codigo), distintos.\n\n",
     );
     s.push_str(
-        "En esta ronda nadie ejecuta la evidencia: se comprueba que este completa y anclada al diff (la ejecucion es de F6-6). \
+        "Nadie ejecuta la evidencia: se comprueba que este completa y anclada al diff. \
 No inventes lineas que no ves: si el diff esta cortado, el hallazgo de lo que no llego se rechaza.\n\n",
+    );
+    s.push_str(
+        "Ronda incremental (F6-6): si el prompt trae la seccion «Hallazgos abiertos de la ronda anterior», el diff es el de lo cambiado desde esa ronda y el bloque es un objeto `{\"hallazgos\": [...], \"cierres\": [...]}`. \
+En `hallazgos` van solo los nuevos, con un id que no use ningun abierto. En `cierres`, uno por cada abierto CRITICO o IMPORTANTE de tu lista que el diff arregla, con estas claves y ninguna mas:\n\n",
+    );
+    s.push_str("- `id`: el del abierto.\n");
+    s.push_str(
+        "- `fichero` y `linea`: una linea del lado nuevo del diff incremental en la que queda arreglado (se cuenta como en los hallazgos).\n",
+    );
+    s.push_str("- `motivo`: por que queda arreglado, en una frase.\n\n");
+    s.push_str(
+        "Un abierto sin cierre valido sigue abierto. Si traia `paso_rojo`, ese paso tiene que estar ya en verde en el informe de verificacion. \
+Los MENOR no se cierran asi: se dan por cerrados cuando cambia su fichero.\n\n",
     );
     s.push_str(&format!(
-        "Como mucho {TOPE_HALLAZGOS} hallazgos. Motivos de rechazo:\n\n"
+        "Como mucho {TOPE_HALLAZGOS} hallazgos y {TOPE_HALLAZGOS} cierres. Motivos de rechazo:\n\n"
     ));
     for (id, explicacion) in MOTIVOS {
         s.push_str(&format!("- `{id}`: {explicacion}.\n"));
@@ -892,9 +1109,28 @@ No inventes lineas que no ves: si el diff esta cortado, el hallazgo de lo que no
     s.push_str("\nEjemplo del bloque:\n\n```json\n");
     s.push_str(EJEMPLO);
     s.push_str("\n```\n");
+    s.push_str(
+        "\nEjemplo del bloque en una ronda incremental (va tambien en un bloque ```json):\n\n```\n",
+    );
+    s.push_str(EJEMPLO_INCREMENTAL);
+    s.push_str("\n```\n");
     s
 }
 
+/// Ejemplo de respuesta de una ronda incremental (lo validan los tests
+/// contra `tests/fixtures/hallazgos/diff.txt` con un abierto `I1`).
+const EJEMPLO_INCREMENTAL: &str = r#"{
+  "hallazgos": [],
+  "cierres": [
+    {
+      "id": "I1",
+      "fichero": "src/calculo.rs",
+      "linea": 12,
+      "motivo": "El tope ya tiene su test."
+    }
+  ]
+}"#;
+
 /// Texto de la respuesta del modelo como texto plano de Markdown en una
 /// linea: sin saltos ni controles, con la puntuacion que da formato escapada
 /// y cortado a `tope` caracteres. Es dato del modelo (F6-7): no puede abrir
@@ -1056,6 +1292,9 @@ pub enum Dictamen {
     /// Alguna revisora no dejo respuesta valida, o no hubo ninguna: la
     /// revision no ocurrio (falla en cerrado, decision 1 de TASK-051).
     Incompleta,
+    /// Se paso el tope de rondas con un CRITICO o IMPORTANTE abierto: no se
+    /// consulta al modelo y los abiertos se arrastran (F6-6).
+    Tope,
 }
 
 impl Dictamen {
@@ -1065,6 +1304,7 @@ impl Dictamen {
             Dictamen::Rojo => "rojo",
             Dictamen::Verde => "verde",
             Dictamen::Incompleta => "incompleta",
+            Dictamen::Tope => "tope",
         }
     }
 
@@ -1073,7 +1313,7 @@ impl Dictamen {
     pub fn conclusion(self) -> &'static str {
         match self {
             Dictamen::Verde => "success",
-            Dictamen::Rojo | Dictamen::Incompleta => "failure",
+            Dictamen::Rojo | Dictamen::Incompleta | Dictamen::Tope => "failure",
         }
     }
 
@@ -1084,18 +1324,89 @@ impl Dictamen {
             "rojo" => Some(Dictamen::Rojo),
             "verde" => Some(Dictamen::Verde),
             "incompleta" => Some(Dictamen::Incompleta),
+            "tope" => Some(Dictamen::Tope),
             _ => None,
         }
     }
 }
 
+/// La ronda de un titulo del check: la de `(ronda=R ...)` (v3) o 1 si es un
+/// titulo v2, sin `ronda=` (F6-5).
+pub fn ronda_de_titulo(titulo: &str) -> Option<u32> {
+    let resto = titulo.strip_prefix(PREFIJO_TITULO)?;
+    match resto.split_once("(ronda=") {
+        Some((_, r)) => r
+            .split(' ')
+            .next()
+            .and_then(|n| n.parse().ok())
+            .filter(|n| *n >= 1),
+        None => Some(1),
+    }
+}
+
+/// Si un hallazgo sigue abierto o se cerro (F6-6).
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
+#[serde(rename_all = "lowercase")]
+pub enum Estado {
+    #[default]
+    Abierto,
+    Cerrado,
+}
+
+impl Estado {
+    /// Como se escribe en el contrato.
+    pub fn texto(self) -> &'static str {
+        match self {
+            Estado::Abierto => "abierto",
+            Estado::Cerrado => "cerrado",
+        }
+    }
+}
+
+fn ronda_uno() -> u32 {
+    1
+}
+
 /// Un aceptado con la revisora que lo encontro: el `raw_details` de su
-/// anotacion, que `tc2 review` lee de vuelta.
+/// anotacion, que `tc2 review` y la ronda siguiente leen de vuelta. F6-6:
+/// la ronda en la que se encontro y si sigue abierto; un `raw_details` v2,
+/// sin esas claves, se lee como ronda 1 y abierto.
 #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
 #[serde(deny_unknown_fields)]
 pub struct AceptadoDe {
     pub revisora: String,
     pub hallazgo: Aceptado,
+    #[serde(default = "ronda_uno")]
+    pub ronda: u32,
+    #[serde(default)]
+    pub estado: Estado,
+}
+
+impl AceptadoDe {
+    /// Si sigue abierto.
+    pub fn abierto(&self) -> bool {
+        self.estado == Estado::Abierto
+    }
+
+    /// Si es CRITICO o IMPORTANTE.
+    pub fn grave(&self) -> bool {
+        self.hallazgo.severidad != Severidad::Menor
+    }
+}
+
+/// Cuenta CRITICO, IMPORTANTE y MENOR de los abiertos de `aceptados`.
+fn recuento(aceptados: &[AceptadoDe]) -> (usize, usize, usize) {
+    let n = |s: Severidad| {
+        aceptados
+            .iter()
+            .filter(|a| a.abierto() && a.hallazgo.severidad == s)
+            .count()
+    };
+    (
+        n(Severidad::Critico),
+        n(Severidad::Importante),
+        n(Severidad::Menor),
+    )
 }
 
 /// Una revisora en el resumen del veredicto.
@@ -1113,11 +1424,24 @@ pub struct Fila {
 #[derive(Debug, Clone, PartialEq, Eq)]
 pub struct Veredicto {
     pub dictamen: Dictamen,
+    /// Ronda de este check (F6-6; 1 sin ronda anterior).
+    pub ronda: u32,
+    /// Modo de la ronda (F6-6).
+    pub modo: ronda::Modo,
+    /// Abiertos por severidad (nuevos y los que persisten).
     pub criticos: usize,
     pub importantes: usize,
     pub menores: usize,
+    /// Aceptados nuevos de esta ronda.
+    pub nuevos: usize,
+    /// Abiertos de rondas anteriores que siguen abiertos.
+    pub persisten: usize,
+    /// Abiertos de rondas anteriores cerrados en esta.
+    pub cerrados: usize,
     pub rechazados: usize,
     pub filas: Vec<Fila>,
+    /// Todos: los abiertos (nuevos y persistentes) y los cerrados en esta
+    /// ronda, con su `estado`.
     pub aceptados: Vec<AceptadoDe>,
 }
 
@@ -1130,13 +1454,34 @@ impl Veredicto {
             .count()
     }
 
+    /// Si fue una verificacion ligera, sin modelo (F6-6).
+    pub fn ligera(&self) -> bool {
+        self.modo == ronda::Modo::Ligera
+    }
+
     /// Titulo del check.
     pub fn titulo(&self) -> String {
-        titulo_de(self.dictamen, self.criticos, self.importantes, self.menores)
+        titulo_de(
+            self.dictamen,
+            self.ronda,
+            (self.criticos, self.importantes, self.menores),
+        )
     }
 }
 
-fn titulo_de(d: Dictamen, criticos: usize, importantes: usize, menores: usize) -> String {
+fn titulo_de(
+    d: Dictamen,
+    ronda: u32,
+    (criticos, importantes, menores): (usize, usize, usize),
+) -> String {
+    format!(
+        "{PREFIJO_TITULO}{} (ronda={ronda} criticos={criticos} importantes={importantes} menores={menores})",
+        d.texto()
+    )
+}
+
+/// El titulo de un check v2 (F6-5, sin ronda): se sigue leyendo como ronda 1.
+fn titulo_v2(d: Dictamen, (criticos, importantes, menores): (usize, usize, usize)) -> String {
     format!(
         "{PREFIJO_TITULO}{} (criticos={criticos} importantes={importantes} menores={menores})",
         d.texto()
@@ -1151,6 +1496,10 @@ fn titulo_de(d: Dictamen, criticos: usize, importantes: usize, menores: usize) -
 /// con cualquier titulo (el de `publicar` cuando no hay veredicto de
 /// `validar` o la API lo rechaza). `None` si no casan: un check con menos
 /// anotaciones de las que dice su titulo (un lote que no llego) o alterado.
+///
+/// F6-6: los recuentos son de los abiertos (los cerrados en esa ronda van
+/// como anotacion `notice`); vale el titulo v3, con `ronda=`, y el v2 (ronda
+/// 1); un `tope` tiene algun CRITICO o IMPORTANTE abierto, como un `rojo`.
 pub fn dictamen_publicado(
     titulo: &str,
     conclusion: &str,
@@ -1160,35 +1509,46 @@ pub fn dictamen_publicado(
     if conclusion != d.conclusion() {
         return None;
     }
-    let n = |s: Severidad| {
-        aceptados
-            .iter()
-            .filter(|a| a.hallazgo.severidad == s)
-            .count()
-    };
-    let (c, i, m) = (
-        n(Severidad::Critico),
-        n(Severidad::Importante),
-        n(Severidad::Menor),
-    );
+    let (c, i, m) = recuento(aceptados);
     let coherente = match d {
-        Dictamen::Rojo => c + i > 0,
+        Dictamen::Rojo | Dictamen::Tope => c + i > 0,
         Dictamen::Verde => c + i == 0,
         Dictamen::Incompleta => true,
     };
-    let casa =
-        titulo == titulo_de(d, c, i, m) || (d == Dictamen::Incompleta && aceptados.is_empty());
+    let casa = ronda_de_titulo(titulo).is_some_and(|r| titulo == titulo_de(d, r, (c, i, m)))
+        || titulo == titulo_v2(d, (c, i, m))
+        || (d == Dictamen::Incompleta && aceptados.is_empty());
     (coherente && casa).then_some(d)
 }
 
 /// Junta los resultados de `validar` de cada revisora (`None` si no dejo
-/// uno legible). En F6-5 todo aceptado esta abierto (el cierre es de F6-6).
+/// uno legible), sin ronda anterior: ronda 1 ([`veredicto_ronda`]).
 pub fn veredicto(revisoras: &[Option<Validados>]) -> Veredicto {
+    veredicto_ronda(revisoras, None)
+}
+
+/// Junta los resultados de `validar` de cada revisora con la decision de la
+/// ronda (F6-6). Los aceptados de las revisoras son nuevos de esta ronda; de
+/// los abiertos heredados, un CRITICO o IMPORTANTE se cierra solo si alguna
+/// revisora dejo un cierre valido de el, y los MENOR cerrados por la
+/// decision (su fichero cambio) salen cerrados. `tope` y `ligera` no
+/// consultan al modelo: sin revisoras, dan `tope` (con los abiertos
+/// arrastrados) y `verde`.
+pub fn veredicto_ronda(
+    revisoras: &[Option<Validados>],
+    decision: Option<&ronda::Decision>,
+) -> Veredicto {
+    let numero = decision.map_or(1, |d| d.ronda);
     let mut v = Veredicto {
         dictamen: Dictamen::Verde,
+        ronda: numero,
+        modo: decision.map_or(ronda::Modo::Completa, |d| d.modo),
         criticos: 0,
         importantes: 0,
         menores: 0,
+        nuevos: 0,
+        persisten: 0,
+        cerrados: 0,
         rechazados: 0,
         filas: Vec::new(),
         aceptados: Vec::new(),
@@ -1204,14 +1564,12 @@ pub fn veredicto(revisoras: &[Option<Validados>]) -> Veredicto {
             continue;
         };
         for a in &r.aceptados {
-            match a.severidad {
-                Severidad::Critico => v.criticos += 1,
-                Severidad::Importante => v.importantes += 1,
-                Severidad::Menor => v.menores += 1,
-            }
+            v.nuevos += 1;
             v.aceptados.push(AceptadoDe {
                 revisora: r.revisora.clone(),
                 hallazgo: a.clone(),
+                ronda: numero,
+                estado: Estado::Abierto,
             });
         }
         v.rechazados += r.rechazados.len();
@@ -1226,7 +1584,38 @@ pub fn veredicto(revisoras: &[Option<Validados>]) -> Veredicto {
                 .map(|x| x.motivo),
         });
     }
-    v.dictamen = if v.filas.is_empty() || v.incompletas() > 0 {
+    if let Some(d) = decision {
+        let cerrado_por = |a: &AceptadoDe| {
+            revisoras.iter().flatten().any(|r| {
+                r.cierres.iter().any(|c| {
+                    c.revisora == a.revisora && c.ronda == a.ronda && c.id == a.hallazgo.id
+                })
+            })
+        };
+        for a in &d.abiertos {
+            let mut a = a.clone();
+            if a.grave() && cerrado_por(&a) {
+                a.estado = Estado::Cerrado;
+                v.cerrados += 1;
+            } else {
+                a.estado = Estado::Abierto;
+                v.persisten += 1;
+            }
+            v.aceptados.push(a);
+        }
+        for a in &d.cerrados {
+            let mut a = a.clone();
+            a.estado = Estado::Cerrado;
+            v.cerrados += 1;
+            v.aceptados.push(a);
+        }
+    }
+    (v.criticos, v.importantes, v.menores) = recuento(&v.aceptados);
+    v.dictamen = if v.modo == ronda::Modo::Tope {
+        Dictamen::Tope
+    } else if v.modo == ronda::Modo::Ligera && v.filas.is_empty() {
+        Dictamen::Verde
+    } else if v.filas.is_empty() || v.incompletas() > 0 {
         Dictamen::Incompleta
     } else if v.criticos + v.importantes > 0 {
         Dictamen::Rojo
@@ -1325,7 +1714,8 @@ fn raw_details(a: &AceptadoDe) -> String {
 }
 
 /// Una anotacion por aceptado, en lotes de [`ANOTACIONES_POR_LOTE`]:
-/// `failure` para CRITICO e IMPORTANTE, `warning` para MENOR; titulo y
+/// `failure` para CRITICO e IMPORTANTE, `warning` para MENOR y `notice` para
+/// los cerrados en esta ronda (F6-6); titulo y
 /// mensaje pasados por [`plano`] (un mensaje vacio pasa a
 /// [`MENSAJE_SIN_TEXTO`]); `raw_details`, el [`AceptadoDe`] en JSON acotado a
 /// [`TOPE_RAW_DETAILS`].
@@ -1335,20 +1725,24 @@ pub fn anotaciones(v: &Veredicto) -> Vec<Vec<Anotacion>> {
         .iter()
         .map(|a| {
             let h = &a.hallazgo;
-            let titulo = format!(
+            let mut titulo = format!(
                 "{} {} {} ({})",
                 plano(&h.id, 64),
                 h.severidad.texto(),
                 h.clase.texto(),
                 plano(&a.revisora, 120)
             );
+            if !a.abierto() {
+                titulo.push_str(&format!(" cerrado en la ronda {}", v.ronda));
+            }
             Anotacion {
                 path: h.fichero.clone(),
                 start_line: h.linea,
                 end_line: h.linea,
-                annotation_level: match h.severidad {
-                    Severidad::Critico | Severidad::Importante => "failure",
-                    Severidad::Menor => "warning",
+                annotation_level: match (a.estado, h.severidad) {
+                    (Estado::Cerrado, _) => "notice",
+                    (_, Severidad::Critico | Severidad::Importante) => "failure",
+                    (_, Severidad::Menor) => "warning",
                 }
                 .to_owned(),
                 // GitHub corta el titulo en 255 caracteres.
@@ -1378,12 +1772,21 @@ pub fn resumen_veredicto(v: &Veredicto, extra: &str) -> String {
         Dictamen::Incompleta => {
             "Alguna revisora no dejo una respuesta valida (o no hubo ninguna): la revision no se completo y el check sale en rojo (F6-5)."
         }
+        Dictamen::Tope => {
+            "Se paso el tope de rondas con hallazgos CRITICOS o IMPORTANTES abiertos: no se consulta al modelo, los abiertos siguen y el check sale en rojo. Una ronda mas la decide la persona en `## Rondas extra` de la tarea (F6-6)."
+        }
     };
     let mut s = format!(
         "**Veredicto: {}** (conclusion {}). {explicacion}\n\n\
-         Aceptados: {} CRITICO, {} IMPORTANTE, {} MENOR; rechazados: {}; revisoras: {}, incompletas: {}.\n\n",
+         Ronda {} ({}): {} nuevos, {} persisten, {} cerrados.\n\n\
+         Abiertos: {} CRITICO, {} IMPORTANTE, {} MENOR; rechazados: {}; revisoras: {}, incompletas: {}.\n\n",
         v.dictamen.texto(),
         v.dictamen.conclusion(),
+        v.ronda,
+        v.modo.texto(),
+        v.nuevos,
+        v.persisten,
+        v.cerrados,
         v.criticos,
         v.importantes,
         v.menores,
@@ -1432,28 +1835,37 @@ pub struct InformeTarea {
     pub sha: String,
     pub conclusion: String,
     pub veredicto: Dictamen,
+    /// Ronda del check (F6-6).
+    pub ronda: u32,
+    /// Abiertos por severidad.
     pub criticos: usize,
     pub importantes: usize,
     pub menores: usize,
+    /// Abiertos encontrados en esta ronda, abiertos de rondas anteriores y
+    /// cerrados en esta (F6-6).
+    pub nuevos: usize,
+    pub persisten: usize,
+    pub cerrados: usize,
     /// El resumen del check (ya pasado por [`plano`]).
     pub resumen: String,
     pub aceptados: Vec<AceptadoDe>,
 }
 
 impl InformeTarea {
-    /// El informe de la tarea `tarea` con lo leido del check.
+    /// El informe de la tarea `tarea` con lo leido del check de la ronda
+    /// `ronda`.
     pub fn nuevo(
         tarea: &str,
-        pr: u64,
-        sha: &str,
+        (pr, sha, ronda): (u64, &str, u32),
         (conclusion, veredicto): (&str, Dictamen),
         resumen: &str,
         aceptados: Vec<AceptadoDe>,
     ) -> Self {
-        let n = |s: Severidad| {
+        let (criticos, importantes, menores) = recuento(&aceptados);
+        let abiertos = |nueva: bool| {
             aceptados
                 .iter()
-                .filter(|a| a.hallazgo.severidad == s)
+                .filter(|a| a.abierto() && (a.ronda >= ronda) == nueva)
                 .count()
         };
         Self {
@@ -1463,9 +1875,13 @@ impl InformeTarea {
             sha: sha.to_owned(),
             conclusion: conclusion.to_owned(),
             veredicto,
-            criticos: n(Severidad::Critico),
-            importantes: n(Severidad::Importante),
-            menores: n(Severidad::Menor),
+            ronda,
+            criticos,
+            importantes,
+            menores,
+            nuevos: abiertos(true),
+            persisten: abiertos(false),
+            cerrados: aceptados.iter().filter(|a| !a.abierto()).count(),
             resumen: resumen.to_owned(),
             aceptados,
         }
@@ -1476,12 +1892,17 @@ impl InformeTarea {
     pub fn markdown(&self) -> String {
         let mut s = format!(
             "# Revision IA de {} (PR {}, commit {})\n\n\
-             Veredicto: **{}** (conclusion {}). Aceptados: {} CRITICO, {} IMPORTANTE, {} MENOR.\n\n",
+             Veredicto: **{}** (conclusion {}). Ronda {}: {} nuevos, {} persisten, {} cerrados. \
+             Abiertos: {} CRITICO, {} IMPORTANTE, {} MENOR.\n\n",
             plano(&self.tarea, 64),
             self.pr,
             plano(&self.sha, 64),
             self.veredicto.texto(),
             plano(&self.conclusion, 32),
+            self.ronda,
+            self.nuevos,
+            self.persisten,
+            self.cerrados,
             self.criticos,
             self.importantes,
             self.menores
@@ -1490,13 +1911,15 @@ impl InformeTarea {
             s.push_str("Sin hallazgos aceptados.\n");
         } else {
             s.push_str(
-                "| revisora | id | severidad | clase | fichero | linea | evidencia | descripcion |\n|---|---|---|---|---|---|---|---|\n",
+                "| revisora | ronda | estado | id | severidad | clase | fichero | linea | evidencia | descripcion |\n|---|---|---|---|---|---|---|---|---|---|\n",
             );
             for a in &self.aceptados {
                 let h = &a.hallazgo;
                 s.push_str(&format!(
-                    "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
+                    "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
                     plano(&a.revisora, 120),
+                    a.ronda,
+                    a.estado.texto(),
                     plano(&h.id, 64),
                     h.severidad.texto(),
                     h.clase.texto(),
@@ -1710,6 +2133,20 @@ mod tests {
         }
         let v = validar("r", &t, &diff(), None);
         assert_eq!((v.aceptados.len(), v.rechazados.len()), (2, 0), "{v:?}");
+        // F6-6: el ejemplo incremental cierra el abierto I1.
+        for x in ["«Hallazgos abiertos de la ronda anterior»", "`cierres`"] {
+            assert!(t.contains(x), "falta {x}");
+        }
+        assert!(t.contains(EJEMPLO_INCREMENTAL));
+        let i1 = abierto("I1", "IMPORTANTE", 12, EV_BORDE);
+        let v = validar_con(
+            "r",
+            &format!("```json\n{EJEMPLO_INCREMENTAL}\n```"),
+            &diff(),
+            None,
+            &[i1],
+        );
+        assert_eq!((v.cierres.len(), v.rechazados.len()), (1, 0), "{v:?}");
     }
 
     /// El esquema declara los mismos motivos, severidades y tipos que el
@@ -1853,8 +2290,7 @@ mod tests {
         );
         let informe = InformeTarea::nuevo(
             "TASK-051",
-            12,
-            &"a".repeat(40),
+            (12, &"a".repeat(40), 1),
             ("failure", Dictamen::Rojo),
             "r",
             Vec::new(),
@@ -1864,6 +2300,30 @@ mod tests {
         assert_eq!(requeridas("informe-tarea"), claves(&informe));
         let de = serde_json::to_value(&v.aceptados).unwrap_or_else(|x| panic!("{x}"));
         assert_eq!(requeridas("aceptado-de"), claves(&de[0]));
+        // F6-6: el cierre y el cierre aceptado, con las claves del codigo.
+        let c = Cierre {
+            id: "I1".to_owned(),
+            fichero: "f".to_owned(),
+            linea: 1,
+            motivo: "m".to_owned(),
+        };
+        let cv = serde_json::to_value(&c).unwrap_or_else(|x| panic!("{x}"));
+        assert_eq!(requeridas("cierre"), claves(&cv));
+        assert_eq!(claves(&e["$defs"]["cierre"]["properties"]), claves(&cv));
+        let ca = CierreAceptado {
+            revisora: "r".to_owned(),
+            ronda: 1,
+            id: c.id,
+            fichero: c.fichero,
+            linea: 1,
+            motivo: c.motivo,
+        };
+        let ca = serde_json::to_value(&ca).unwrap_or_else(|x| panic!("{x}"));
+        assert_eq!(requeridas("cierre-aceptado"), claves(&ca));
+        assert_eq!(
+            e["$defs"]["informe-tarea"]["properties"]["veredicto"]["enum"],
+            serde_json::json!(["rojo", "verde", "incompleta", "tope"])
+        );
     }
 
     /// Un mensaje que `plano` deja vacio no sale vacio, y `raw_details` cabe
@@ -1943,6 +2403,169 @@ mod tests {
         assert!(!pasos.is_empty() && pasos.len() < 30_000, "{}", pasos.len());
     }
 
+    /// Un abierto de la revisora `r` en la ronda 1, con la evidencia dada.
+    fn abierto(id: &str, severidad: &str, linea: u32, evidencia: &str) -> AceptadoDe {
+        serde_json::from_str(&format!(
+            r#"{{"revisora":"r","hallazgo":{{"id":"{id}","severidad":"{severidad}","clase":"correccion","fichero":"src/calculo.rs","linea":{linea},"descripcion":"d","evidencia":{evidencia}}}}}"#
+        ))
+        .unwrap_or_else(|e| panic!("{e}"))
+    }
+
+    const EV_BORDE: &str = r#"{"tipo":"borde","entrada":"x","esperado":"1","obtenido":"2"}"#;
+
+    /// F6-6, criterio 1 (unitario): cada regla del cierre rechaza con su
+    /// motivo y un abierto sin cierre valido no se cierra.
+    #[test]
+    fn cada_regla_del_cierre_rechaza_con_su_motivo() {
+        let d = diff();
+        let informe = crate::verificacion::informe::leer(ROJO).unwrap_or_else(|e| panic!("{e}"));
+        let abiertos = vec![
+            abierto("I1", "IMPORTANTE", 12, EV_BORDE),
+            abierto("M1", "MENOR", 8, EV_BORDE),
+            abierto(
+                "R1",
+                "CRITICO",
+                5,
+                r#"{"tipo":"reproduccion","paso_rojo":"clippy"}"#,
+            ),
+            abierto(
+                "F1",
+                "IMPORTANTE",
+                5,
+                r#"{"tipo":"reproduccion","paso_rojo":"fmt"}"#,
+            ),
+        ];
+        let cierre = |id: &str, linea: u32| {
+            format!(
+                r#"{{"id":"{id}","fichero":"src/calculo.rs","linea":{linea},"motivo":"arreglado"}}"#
+            )
+        };
+        let con = |cierres: &[String]| {
+            format!(
+                "Informe.\n\n```json\n{{\"hallazgos\": [], \"cierres\": [{}]}}\n```\n",
+                cierres.join(",")
+            )
+        };
+        let casos: Vec<(String, &str)> = vec![
+            (con(&[cierre("X9", 12)]), "X9:cierre-desconocido"),
+            // Los MENOR no pasan por el cierre del modelo.
+            (con(&[cierre("M1", 8)]), "M1:cierre-desconocido"),
+            (con(&[cierre("I1", 99)]), "I1:cierre-fuera-del-diff"),
+            (con(&[cierre("R1", 5)]), "R1:paso-rojo-sigue-rojo"),
+            (
+                con(&[r#"{"id":"I1","fichero":"src/calculo.rs","linea":12}"#.to_owned()]),
+                "I1:hallazgo-invalido",
+            ),
+            (
+                con(&[cierre("I1", 12).replace("arreglado", " ")]),
+                "I1:hallazgo-invalido",
+            ),
+            // Un hallazgo nuevo no reutiliza el id de un abierto.
+            (
+                una(BORDE.replace("\"id\":\"B\"", "\"id\":\"I1\"").as_str()),
+                "I1:id-repetido",
+            ),
+            (
+                "```json\n{\"hallazgos\": [], \"otra\": []}\n```".to_owned(),
+                "respuesta:json-invalido",
+            ),
+            (
+                "```json\n{\"cierres\": []}\n```".to_owned(),
+                "respuesta:json-invalido",
+            ),
+        ];
+        for (respuesta, esperado) in casos {
+            let v = validar_con("r", &respuesta, &d, Some(&informe), &abiertos);
+            assert_eq!(v.motivos(), [esperado], "{respuesta}");
+            assert!(v.cierres.is_empty(), "{respuesta}: {:?}", v.cierres);
+        }
+        // Un segundo cierre del mismo abierto se rechaza; el primero vale.
+        let v = validar_con(
+            "r",
+            &con(&[cierre("I1", 12), cierre("I1", 12)]),
+            &d,
+            Some(&informe),
+            &abiertos,
+        );
+        assert_eq!(v.motivos(), ["I1:id-repetido"]);
+        assert_eq!(v.cierres.len(), 1);
+        // Sin informe, un abierto con paso_rojo no se cierra (falla en
+        // cerrado); con el paso ya en verde (fmt), si.
+        let v = validar_con("r", &con(&[cierre("F1", 5)]), &d, None, &abiertos);
+        assert_eq!(v.motivos(), ["F1:paso-rojo-sigue-rojo"]);
+        let v = validar_con(
+            "r",
+            &con(&[cierre("F1", 5), cierre("I1", 12)]),
+            &d,
+            Some(&informe),
+            &abiertos,
+        );
+        assert_eq!(v.motivos(), Vec::<String>::new());
+        let cerrados: Vec<(&str, u32)> =
+            v.cierres.iter().map(|c| (c.id.as_str(), c.ronda)).collect();
+        assert_eq!(cerrados, [("F1", 1), ("I1", 1)]);
+        // Los cierres se numeran detras de los hallazgos.
+        let r = format!(
+            "```json\n{{\"hallazgos\": [{BORDE}], \"cierres\": [{}]}}\n```",
+            cierre("X9", 12)
+        );
+        let v = validar_con("r", &r, &d, Some(&informe), &abiertos);
+        assert_eq!(v.rechazados[0].indice, Some(1));
+        assert_eq!(v.aceptados.len(), 1);
+    }
+
+    /// F6-6, criterio 2: lo publicado por F6-5 (v2) se sigue leyendo: un
+    /// `raw_details` sin ronda ni estado es ronda 1 y abierto, y el titulo
+    /// sin `ronda=` es ronda 1. Un `tope` sin abiertos graves no casa.
+    #[test]
+    fn lo_publicado_en_v2_se_sigue_leyendo() {
+        let v2: AceptadoDe = serde_json::from_str(
+            r#"{"revisora":"r","hallazgo":{"id":"I1","severidad":"IMPORTANTE","clase":"contrato","fichero":"a","linea":1,"descripcion":"d","evidencia":{"tipo":"borde","entrada":"x","esperado":"1","obtenido":"2"}}}"#,
+        )
+        .unwrap_or_else(|e| panic!("{e}"));
+        assert_eq!((v2.ronda, v2.estado), (1, Estado::Abierto));
+        let t2 = "revision-ia: rojo (criticos=0 importantes=1 menores=0)";
+        assert_eq!(ronda_de_titulo(t2), Some(1));
+        assert_eq!(
+            dictamen_publicado(t2, "failure", std::slice::from_ref(&v2)),
+            Some(Dictamen::Rojo)
+        );
+        let t3 = "revision-ia: rojo (ronda=3 criticos=0 importantes=1 menores=0)";
+        assert_eq!(ronda_de_titulo(t3), Some(3));
+        assert_eq!(
+            dictamen_publicado(t3, "failure", std::slice::from_ref(&v2)),
+            Some(Dictamen::Rojo)
+        );
+        assert_eq!(
+            ronda_de_titulo("revision-ia: rojo (ronda=0 criticos=0)"),
+            None
+        );
+        assert_eq!(ronda_de_titulo("otro"), None);
+        // Los cerrados no cuentan: un rojo con su unico grave cerrado no casa.
+        let mut cerrado = v2.clone();
+        cerrado.estado = Estado::Cerrado;
+        assert_eq!(dictamen_publicado(t3, "failure", &[cerrado.clone()]), None);
+        let verde = "revision-ia: verde (ronda=2 criticos=0 importantes=0 menores=0)";
+        assert_eq!(
+            dictamen_publicado(verde, "success", &[cerrado]),
+            Some(Dictamen::Verde)
+        );
+        let tope = "revision-ia: tope (ronda=3 criticos=0 importantes=1 menores=0)";
+        assert_eq!(
+            dictamen_publicado(tope, "failure", std::slice::from_ref(&v2)),
+            Some(Dictamen::Tope)
+        );
+        assert_eq!(dictamen_publicado(tope, "success", &[v2]), None);
+        assert_eq!(
+            dictamen_publicado(
+                "revision-ia: tope (ronda=3 criticos=0 importantes=0 menores=0)",
+                "failure",
+                &[]
+            ),
+            None
+        );
+    }
+
     /// El dictamen publicado solo se da si el titulo casa con la conclusion
     /// y los aceptados (MEN-1 de la ronda 1 de TASK-051).
     #[test]
diff --git a/crates/tc2/src/hallazgos/orden.rs b/crates/tc2/src/hallazgos/orden.rs
index ed07257..9757142 100644
--- a/crates/tc2/src/hallazgos/orden.rs
+++ b/crates/tc2/src/hallazgos/orden.rs
@@ -27,6 +27,23 @@
 //! veredicto=<rojo|verde|incompleta> conclusion=<success|failure>
 //! criticos=N importantes=M menores=K revisoras=R incompletas=I lotes=L`.
 //! Un veredicto rojo no es un fallo de la orden: sale `ok`.
+//!
+//! F6-6 (rondas incrementales):
+//! - `anterior --check-runs <json>... [--origen <id>]`: de los volcados de
+//!   check-runs de los commits del PR (del mas nuevo al mas viejo, sin el
+//!   head), el check que sirve de ronda anterior; la App del revisor sale de
+//!   `--origen` o de `plataforma.revisor_id` de la configuracion de la base.
+//!   Linea: `hallazgos ok anterior=<check-id>|ninguno [sha=<sha>]
+//!   avisos=<...>`.
+//! - `ronda ... --salida <dir>`: con las anotaciones de ese check y los
+//!   ficheros cambiados desde su commit, escribe `ronda.json`
+//!   (`$defs/ronda`). Linea: `hallazgos ok ronda=R modo=<m>
+//!   anterior=<sha12|ninguno> abiertos=N tope=T avisos=<...>`.
+//! - `abiertos --ronda <json> --detalle <json> --salida <dir>`: los abiertos
+//!   CRITICO e IMPORTANTE de cada revisora (`abiertos-<i>.json` y `.md`).
+//! - `validar --abiertos <json>` valida tambien los cierres; `veredicto
+//!   --ronda <json>` cuenta nuevos, persisten y cerrados (la linea gana
+//!   `ronda=R modo=<m> nuevos=N persisten=P cerrados=C`).
 
 use std::io::Read;
 use std::path::{Path, PathBuf};
@@ -65,6 +82,7 @@ struct Args {
     informe: Option<PathBuf>,
     salida: Option<PathBuf>,
     resumen: Option<PathBuf>,
+    abiertos: Option<PathBuf>,
 }
 
 fn uso(motivo: &str) -> Informe {
@@ -100,6 +118,7 @@ fn args(a: &[String]) -> Result<Args, Informe> {
             "--informe" if r.informe.is_none() => r.informe = Some(valor()?.into()),
             "--salida" if r.salida.is_none() => r.salida = Some(valor()?.into()),
             "--resumen" if r.resumen.is_none() => r.resumen = Some(valor()?.into()),
+            "--abiertos" if r.abiertos.is_none() => r.abiertos = Some(valor()?.into()),
             _ => return Err(uso("argumento-desconocido")),
         }
     }
@@ -169,6 +188,15 @@ pub fn orden(a: &[String], json: bool, idioma: Idioma) -> Salida {
         [s, resto @ ..] if s == "veredicto" => match veredicto(resto) {
             Ok(i) | Err(i) => i.salida_en(json, idioma),
         },
+        [s, resto @ ..] if s == "anterior" => match anterior(resto) {
+            Ok(i) | Err(i) => i.salida_en(json, idioma),
+        },
+        [s, resto @ ..] if s == "ronda" => match ronda(resto) {
+            Ok(i) | Err(i) => i.salida_en(json, idioma),
+        },
+        [s, resto @ ..] if s == "abiertos" => match abiertos(resto) {
+            Ok(i) | Err(i) => i.salida_en(json, idioma),
+        },
         [] => uso("sin-suborden").salida_en(json, idioma),
         _ => uso("suborden-desconocida").salida_en(json, idioma),
     }
@@ -192,7 +220,11 @@ fn validar(a: &[String]) -> Result<Informe, Informe> {
             },
         },
     };
-    let v = super::validar(revisora, &respuesta, &diff, inf.as_ref());
+    let abiertos: Vec<super::AceptadoDe> = match &a.abiertos {
+        Some(p) => leer_json(p, "una lista de abiertos de tc2 hallazgos abiertos")?,
+        None => Vec::new(),
+    };
+    let v = super::validar_con(revisora, &respuesta, &diff, inf.as_ref(), &abiertos);
     if let Some(p) = &a.salida {
         let texto = serde_json::to_string_pretty(&v).map_err(|e| fallo_io(p, e.to_string()))?;
         std::fs::write(p, texto + "\n").map_err(|e| fallo_io(p, e.to_string()))?;
@@ -205,6 +237,9 @@ fn validar(a: &[String]) -> Result<Informe, Informe> {
         .con("aceptados", v.aceptados.len().to_string())
         .con("rechazados", v.rechazados.len().to_string())
         .con_lista("motivos", v.motivos());
+    if a.abiertos.is_some() {
+        i = i.con("cierres", v.cierres.len().to_string());
+    }
     if let Some(e) = estado {
         i = i.con("informe", e);
     }
@@ -227,6 +262,7 @@ fn escribir_json(p: &Path, v: &serde_json::Value) -> Result<(), Informe> {
 fn veredicto(a: &[String]) -> Result<Informe, Informe> {
     let mut salida: Option<PathBuf> = None;
     let mut extra: Option<PathBuf> = None;
+    let mut decision: Option<PathBuf> = None;
     let mut entradas: Vec<PathBuf> = Vec::new();
     let mut it = a.iter();
     while let Some(x) = it.next() {
@@ -235,6 +271,7 @@ fn veredicto(a: &[String]) -> Result<Informe, Informe> {
             "--hallazgos" => entradas.push(valor()?.into()),
             "--salida" if salida.is_none() => salida = Some(valor()?.into()),
             "--extra" if extra.is_none() => extra = Some(valor()?.into()),
+            "--ronda" if decision.is_none() => decision = Some(valor()?.into()),
             _ => return Err(uso("argumento-desconocido")),
         }
     }
@@ -247,7 +284,8 @@ fn veredicto(a: &[String]) -> Result<Informe, Informe> {
     };
     let revisoras: Vec<Option<super::Validados>> =
         entradas.iter().map(|p| leer_validados(p)).collect();
-    let v = super::veredicto(&revisoras);
+    let decision = decision.as_deref().map(leer_decision).transpose()?;
+    let v = super::veredicto_ronda(&revisoras, decision.as_ref());
     let lotes = super::anotaciones(&v);
     let (titulo, resumen) = (v.titulo(), super::resumen_veredicto(&v, &extra));
     std::fs::create_dir_all(&dir).map_err(|e| fallo_io(&dir, e.to_string()))?;
@@ -280,5 +318,296 @@ fn veredicto(a: &[String]) -> Result<Informe, Informe> {
         .con("menores", v.menores.to_string())
         .con("revisoras", v.filas.len().to_string())
         .con("incompletas", v.incompletas().to_string())
-        .con("lotes", lotes.len().max(1).to_string()))
+        .con("lotes", lotes.len().max(1).to_string())
+        .con("ronda", v.ronda.to_string())
+        .con("modo", v.modo.texto())
+        .con("nuevos", v.nuevos.to_string())
+        .con("persisten", v.persisten.to_string())
+        .con("cerrados", v.cerrados.to_string()))
+}
+
+// --- Rondas (F6-6) ------------------------------------------------------------
+
+/// Bytes que se leen como mucho de cada volcado de check-runs, pagina de
+/// anotaciones, `ronda.json`, abiertos o detalle (F6-6).
+pub const TOPE_JSON: u64 = 4 * 1024 * 1024;
+
+fn no_es(p: &Path, que: &str, error: String) -> Informe {
+    Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(
+        Mensaje::nuevo(&txt::NO_ES_DEL_CONTRATO)
+            .con("ruta", p.display().to_string())
+            .con("que", que)
+            .con("error", error),
+    )
+}
+
+/// Lee `p` (con tope) como `T`; si no, el error dice que se esperaba `que`.
+fn leer_json<T: serde::de::DeserializeOwned>(p: &Path, que: &str) -> Result<T, Informe> {
+    let texto = leer_entrada(p, TOPE_JSON)?;
+    serde_json::from_str(&texto).map_err(|e| no_es(p, que, e.to_string()))
+}
+
+/// Un volcado de `GET .../commits/<sha>/check-runs`.
+#[derive(serde::Deserialize)]
+struct PaginaChecks {
+    check_runs: Vec<super::ronda::CheckRun>,
+}
+
+/// Una anotacion de `GET .../check-runs/<id>/annotations`: su `raw_details`.
+#[derive(serde::Deserialize)]
+struct AnotacionLeida {
+    #[serde(default)]
+    raw_details: Option<String>,
+}
+
+/// La App del revisor: `--origen` o `plataforma.revisor_id` de la
+/// configuracion de la base; con la configuracion (para el tope) y los
+/// avisos.
+fn origen_y_config(
+    origen: Option<u64>,
+    g: &crate::git::Git,
+    dir: &Path,
+) -> (Option<u64>, crate::config::Config, Vec<String>) {
+    let mut avisos = Vec::new();
+    let config = crate::config::de_base(g, dir).unwrap_or_else(|_| {
+        avisos.push("config-de-base-no-leida".to_owned());
+        crate::config::Config::suelo()
+    });
+    let origen = origen
+        .or_else(|| config.plataforma.as_ref().and_then(|p| p.revisor_id))
+        .filter(|id| *id != 0);
+    if origen.is_none() {
+        avisos.push("sin-origen".to_owned());
+    }
+    (origen, config, avisos)
+}
+
+fn carpeta_actual() -> Result<PathBuf, Informe> {
+    std::env::current_dir().map_err(|e| fallo_io(Path::new("."), e.to_string()))
+}
+
+fn id_de_app(v: &str) -> Result<u64, Informe> {
+    v.parse::<u64>()
+        .ok()
+        .filter(|n| *n != 0)
+        .ok_or_else(|| uso("origen-invalido"))
+}
+
+/// Los check-runs de cada `--check-runs`, en el orden dado (del commit mas
+/// nuevo al mas viejo).
+fn leer_checks(ficheros: &[PathBuf]) -> Result<Vec<Vec<super::ronda::CheckRun>>, Informe> {
+    ficheros
+        .iter()
+        .map(|p| {
+            leer_json::<PaginaChecks>(p, "un volcado de check-runs de la API").map(|x| x.check_runs)
+        })
+        .collect()
+}
+
+/// `anterior --check-runs <json>... [--origen <id>]`: el check que sirve de
+/// ronda anterior ([`super::ronda::elegir`]). Linea: `hallazgos ok
+/// anterior=<id>|ninguno [sha=<sha>] [avisos=...]`.
+fn anterior(a: &[String]) -> Result<Informe, Informe> {
+    let mut checks: Vec<PathBuf> = Vec::new();
+    let mut origen: Option<u64> = None;
+    let mut it = a.iter();
+    while let Some(x) = it.next() {
+        let mut valor = || it.next().cloned().ok_or_else(|| uso("falta-valor"));
+        match x.as_str() {
+            "--check-runs" => checks.push(valor()?.into()),
+            "--origen" if origen.is_none() => origen = Some(id_de_app(&valor()?)?),
+            _ => return Err(uso("argumento-desconocido")),
+        }
+    }
+    let dir = carpeta_actual()?;
+    let (origen, _, avisos) = origen_y_config(origen, &crate::git::Git::nuevo(), &dir);
+    let commits = leer_checks(&checks)?;
+    let elegido = origen.and_then(|o| super::ronda::elegir(commits, o));
+    let mut i = Informe::nuevo(NOMBRE, Resultado::Ok);
+    i = match &elegido {
+        Some(c) => i
+            .con("anterior", c.id.to_string())
+            .con("sha", c.head_sha.as_str()),
+        None => i.con("anterior", "ninguno"),
+    };
+    Ok(i.con_lista("avisos", avisos))
+}
+
+/// Rutas separadas por NUL (la salida de `git diff --name-only -z`).
+fn rutas_nul(p: &Path) -> Result<Vec<String>, Informe> {
+    let texto = leer_entrada(p, crate::seleccion::orden::TOPE_RUTAS)?;
+    Ok(texto
+        .split('\0')
+        .filter(|r| !r.trim().is_empty())
+        .map(str::to_owned)
+        .collect())
+}
+
+/// Las rondas extra de la tarea `id` en la rama de estado, o el aviso.
+fn extra_de_tarea(g: &crate::git::Git, dir: &Path, id: &str) -> Result<u32, String> {
+    let v = crate::estado::leer(g, dir)
+        .map_err(|i| format!("tarea-no-leida:{}", i.valor("motivo").unwrap_or("estado")))?;
+    let (_, cuerpo) = crate::estado::tarea_de(&v, id).map_err(|f| match f {
+        crate::estado::Fallo::Rechazada { motivo, .. } => format!("tarea-no-leida:{motivo}"),
+        crate::estado::Fallo::Error(_) => "tarea-no-leida:error".to_owned(),
+    })?;
+    Ok(super::ronda::rondas_extra(&cuerpo))
+}
+
+/// `ronda --check-runs <json>... [--anotaciones <json>]... --rutas <nul>
+/// [--tarea TASK-NNN] [--origen <id>] --salida <dir>`: escribe
+/// `<dir>/ronda.json` ([`super::ronda::decidir`]). El tope efectivo es
+/// `tope_rondas` de la base mas los elementos de `## Rondas extra` de la
+/// tarea. Linea: `hallazgos ok ronda=R modo=<m> anterior=<sha12|ninguno>
+/// abiertos=N tope=T avisos=<...>`.
+fn ronda(a: &[String]) -> Result<Informe, Informe> {
+    let (mut checks, mut anotaciones): (Vec<PathBuf>, Vec<PathBuf>) = (Vec::new(), Vec::new());
+    let (mut rutas, mut tarea, mut salida): (Option<PathBuf>, Option<String>, Option<PathBuf>) =
+        (None, None, None);
+    let mut origen: Option<u64> = None;
+    let mut it = a.iter();
+    while let Some(x) = it.next() {
+        let mut valor = || it.next().cloned().ok_or_else(|| uso("falta-valor"));
+        match x.as_str() {
+            "--check-runs" => checks.push(valor()?.into()),
+            "--anotaciones" => anotaciones.push(valor()?.into()),
+            "--rutas" if rutas.is_none() => rutas = Some(valor()?.into()),
+            "--tarea" if tarea.is_none() => {
+                let v = valor()?;
+                if !crate::git::es_id(&v) {
+                    return Err(uso("id-invalido"));
+                }
+                tarea = Some(v);
+            }
+            "--origen" if origen.is_none() => origen = Some(id_de_app(&valor()?)?),
+            "--salida" if salida.is_none() => salida = Some(valor()?.into()),
+            _ => return Err(uso("argumento-desconocido")),
+        }
+    }
+    let Some(rutas) = rutas else {
+        return Err(uso("sin-rutas"));
+    };
+    let Some(dir_salida) = salida else {
+        return Err(uso("sin-salida"));
+    };
+    let cambiados = rutas_nul(&rutas)?;
+    let dir = carpeta_actual()?;
+    let g = crate::git::Git::nuevo();
+    let (origen, config, mut avisos) = origen_y_config(origen, &g, &dir);
+    let mut tope = u32::from(config.tope_rondas);
+    if let Some(id) = &tarea {
+        match extra_de_tarea(&g, &dir, id) {
+            Ok(n) => tope = tope.saturating_add(n),
+            Err(aviso) => avisos.push(aviso),
+        }
+    }
+    let commits = leer_checks(&checks)?;
+    let elegido = origen.and_then(|o| super::ronda::elegir(commits, o));
+    let previa = match &elegido {
+        None => None,
+        Some(c) => {
+            let mut aceptados = Vec::new();
+            for p in &anotaciones {
+                let pagina: Vec<AnotacionLeida> =
+                    leer_json(p, "una pagina de anotaciones de la API")?;
+                for n in pagina {
+                    let a = n
+                        .raw_details
+                        .as_deref()
+                        .and_then(|r| serde_json::from_str::<super::AceptadoDe>(r).ok())
+                        .ok_or_else(|| {
+                            no_es(
+                                p,
+                                "una pagina de anotaciones de tc2 hallazgos veredicto",
+                                "una anotacion sin raw_details de tc2".to_owned(),
+                            )
+                        })?;
+                    aceptados.push(a);
+                }
+            }
+            let previa = super::ronda::anterior(c, aceptados).ok_or_else(|| {
+                Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(
+                    Mensaje::nuevo(&txt::ANTERIOR_INCOHERENTE)
+                        .con("check", c.id.to_string())
+                        .con("sha", crate::contrato::corto(&c.head_sha)),
+                )
+            })?;
+            Some(previa)
+        }
+    };
+    let d = super::ronda::decidir(previa.as_ref(), &cambiados, tope);
+    std::fs::create_dir_all(&dir_salida).map_err(|e| fallo_io(&dir_salida, e.to_string()))?;
+    let v = serde_json::to_value(&d).map_err(|e| fallo_io(&dir_salida, e.to_string()))?;
+    escribir_json(&dir_salida.join("ronda.json"), &v)?;
+    Ok(Informe::nuevo(NOMBRE, Resultado::Ok)
+        .con("ronda", d.ronda.to_string())
+        .con("modo", d.modo.texto())
+        .con(
+            "anterior",
+            d.sha_anterior
+                .as_deref()
+                .map_or("ninguno".to_owned(), |s| s.chars().take(12).collect()),
+        )
+        .con("abiertos", d.abiertos.len().to_string())
+        .con("tope", d.tope.to_string())
+        .con_lista("avisos", avisos))
+}
+
+/// Una `ronda.json` del contrato vigente.
+fn leer_decision(p: &Path) -> Result<super::ronda::Decision, Informe> {
+    let d: super::ronda::Decision = leer_json(p, "una ronda.json de tc2 hallazgos ronda")?;
+    if d.version != super::VERSION {
+        return Err(no_es(
+            p,
+            "una ronda.json de tc2 hallazgos ronda",
+            format!("version {} y no {}", d.version, super::VERSION),
+        ));
+    }
+    Ok(d)
+}
+
+/// El detalle de `tc2 revisoras --detalle`.
+#[derive(serde::Deserialize)]
+struct Detalle {
+    revisoras: Vec<super::ronda::RevisoraDetalle>,
+}
+
+/// `abiertos --ronda <json> --detalle <json> --salida <dir>`: por revisora
+/// del detalle (en su orden), `abiertos-<i>.json` y `abiertos-<i>.md`
+/// ([`super::ronda::repartir`]). Linea: `hallazgos ok revisoras=N
+/// abiertos=M sin-revisora=K`.
+fn abiertos(a: &[String]) -> Result<Informe, Informe> {
+    let (mut r, mut detalle, mut salida): (Option<PathBuf>, Option<PathBuf>, Option<PathBuf>) =
+        (None, None, None);
+    let mut it = a.iter();
+    while let Some(x) = it.next() {
+        let mut valor = || it.next().cloned().ok_or_else(|| uso("falta-valor"));
+        match x.as_str() {
+            "--ronda" if r.is_none() => r = Some(valor()?.into()),
+            "--detalle" if detalle.is_none() => detalle = Some(valor()?.into()),
+            "--salida" if salida.is_none() => salida = Some(valor()?.into()),
+            _ => return Err(uso("argumento-desconocido")),
+        }
+    }
+    let (Some(r), Some(detalle), Some(dir)) = (r, detalle, salida) else {
+        return Err(uso("faltan-argumentos"));
+    };
+    let d = leer_decision(&r)?;
+    let det: Detalle = leer_json(&detalle, "un detalle de tc2 revisoras")?;
+    let reparto = super::ronda::repartir(&d.abiertos, &det.revisoras);
+    std::fs::create_dir_all(&dir).map_err(|e| fallo_io(&dir, e.to_string()))?;
+    let mut dados = 0;
+    for (i, lista) in reparto.iter().enumerate() {
+        dados += lista.len();
+        let v = serde_json::to_value(lista).map_err(|e| fallo_io(&dir, e.to_string()))?;
+        escribir_json(&dir.join(format!("abiertos-{i}.json")), &v)?;
+        let md = dir.join(format!("abiertos-{i}.md"));
+        std::fs::write(&md, super::ronda::abiertos_markdown(lista))
+            .map_err(|e| fallo_io(&md, e.to_string()))?;
+    }
+    let graves = d.abiertos.iter().filter(|a| a.grave()).count();
+    Ok(Informe::nuevo(NOMBRE, Resultado::Ok)
+        .con("revisoras", reparto.len().to_string())
+        .con("abiertos", dados.to_string())
+        .con("sin-revisora", (graves - dados.min(graves)).to_string()))
 }
diff --git a/crates/tc2/src/hallazgos/ronda.rs b/crates/tc2/src/hallazgos/ronda.rs
new file mode 100644
index 0000000..29ccf27
--- /dev/null
+++ b/crates/tc2/src/hallazgos/ronda.rs
@@ -0,0 +1,442 @@
+//! Rondas incrementales del revisor IA (F6-6; RF-33).
+//!
+//! Funciones puras: la ronda siguiente se decide con el check `revision-ia`
+//! anterior, el que publico la App del revisor (no la rama de estado, que
+//! escribe el agente: F1-8). Cada check lleva su ronda en el titulo y todos
+//! los hallazgos que siguen abiertos en sus anotaciones.
+
+use std::collections::BTreeSet;
+
+use serde::{Deserialize, Serialize};
+
+use super::{AceptadoDe, Dictamen, VERSION, dictamen_publicado, plano, ronda_de_titulo};
+
+/// Como se revisa esta ronda.
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(rename_all = "lowercase")]
+pub enum Modo {
+    /// Ronda 1 (o sin ronda anterior valida): todo el diff contra la base.
+    Completa,
+    /// El diff desde el commit de la ronda anterior y sus abiertos.
+    Incremental,
+    /// Anterior en verde y los cambios solo tocan ficheros con MENOR
+    /// abiertos: verificacion determinista, sin modelo; la ronda no sube.
+    Ligera,
+    /// Pasado el tope de rondas con un CRITICO o IMPORTANTE abierto: sin
+    /// modelo, los abiertos se arrastran y el check sale `tope`.
+    Tope,
+}
+
+impl Modo {
+    /// Como se escribe en la linea y en `ronda.json`.
+    pub fn texto(self) -> &'static str {
+        match self {
+            Modo::Completa => "completa",
+            Modo::Incremental => "incremental",
+            Modo::Ligera => "ligera",
+            Modo::Tope => "tope",
+        }
+    }
+}
+
+/// La decision de la ronda: `ronda.json` (`$defs/ronda` de
+/// `hallazgos.schema.json`).
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct Decision {
+    pub version: u32,
+    pub ronda: u32,
+    pub modo: Modo,
+    /// Commit del check anterior (base del diff incremental).
+    pub sha_anterior: Option<String>,
+    /// Tope efectivo de rondas.
+    pub tope: u32,
+    /// Abiertos heredados que siguen abiertos.
+    pub abiertos: Vec<AceptadoDe>,
+    /// MENOR heredados que se cierran porque su fichero cambio.
+    pub cerrados: Vec<AceptadoDe>,
+}
+
+/// Un check-run tal como lo da la API (`GET .../commits/<sha>/check-runs`):
+/// solo lo que se mira; lo demas se ignora.
+#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
+pub struct CheckRun {
+    #[serde(default)]
+    pub id: u64,
+    #[serde(default)]
+    pub name: String,
+    #[serde(default)]
+    pub status: String,
+    #[serde(default)]
+    pub conclusion: Option<String>,
+    #[serde(default)]
+    pub head_sha: String,
+    #[serde(default)]
+    pub app: Option<AppCheck>,
+    #[serde(default)]
+    pub output: Option<SalidaCheck>,
+}
+
+/// La App que publico el check.
+#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
+pub struct AppCheck {
+    #[serde(default)]
+    pub id: u64,
+}
+
+/// `output` de un check-run.
+#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
+pub struct SalidaCheck {
+    #[serde(default)]
+    pub title: Option<String>,
+    #[serde(default)]
+    pub summary: Option<String>,
+}
+
+impl CheckRun {
+    /// `output.title`, o vacio.
+    pub fn titulo(&self) -> &str {
+        self.output
+            .as_ref()
+            .and_then(|o| o.title.as_deref())
+            .unwrap_or_default()
+    }
+}
+
+/// El check `revision-ia` de un commit que cuenta: el mas nuevo de ese
+/// nombre que publico la App `origen` (la API los da del mas nuevo al mas
+/// viejo), y solo si ya termino. Lo comparten `leer-revision` (F6-5) y la
+/// ronda siguiente (F6-6).
+pub fn revision_de_origen(runs: Vec<CheckRun>, origen: u64) -> Option<CheckRun> {
+    runs.into_iter()
+        .find(|r| {
+            r.name == crate::revision::REVISION_IA && r.app.as_ref().is_some_and(|a| a.id == origen)
+        })
+        .filter(|r| r.status == "completed" && r.id != 0)
+}
+
+/// El check que sirve de ronda anterior: recorriendo los commits del PR del
+/// mas nuevo al mas viejo (sin el head), el primero cuyo `revision-ia` de la
+/// App `origen` dice `rojo` o `verde` en su titulo. Una ronda `incompleta`
+/// no cuenta ni sirve de base, y una `tope` tampoco (no reviso nada; lleva
+/// los mismos abiertos que la ronda que la origino).
+pub fn elegir(commits: Vec<Vec<CheckRun>>, origen: u64) -> Option<CheckRun> {
+    commits.into_iter().find_map(|runs| {
+        revision_de_origen(runs, origen).filter(|r| {
+            matches!(
+                Dictamen::de_titulo(r.titulo()),
+                Some(Dictamen::Rojo | Dictamen::Verde)
+            )
+        })
+    })
+}
+
+/// La ronda anterior, leida de su check y de sus anotaciones.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Anterior {
+    pub ronda: u32,
+    pub sha: String,
+    pub dictamen: Dictamen,
+    /// Solo los que siguen abiertos.
+    pub abiertos: Vec<AceptadoDe>,
+}
+
+/// La ronda anterior de `check` con los aceptados leidos de sus anotaciones,
+/// solo si el check casa ([`dictamen_publicado`], la regla de
+/// `leer-revision`), dice `rojo` o `verde` y su commit es un SHA. `None` si
+/// no: el workflow lo trata como un check alterado (falla en cerrado).
+pub fn anterior(check: &CheckRun, aceptados: Vec<AceptadoDe>) -> Option<Anterior> {
+    let titulo = check.titulo();
+    let conclusion = check.conclusion.as_deref().unwrap_or_default();
+    let dictamen = dictamen_publicado(titulo, conclusion, &aceptados)
+        .filter(|d| matches!(d, Dictamen::Rojo | Dictamen::Verde))?;
+    let sha_valido =
+        check.head_sha.len() == 40 && check.head_sha.chars().all(|c| c.is_ascii_hexdigit());
+    if !sha_valido {
+        return None;
+    }
+    Some(Anterior {
+        ronda: ronda_de_titulo(titulo)?,
+        sha: check.head_sha.to_ascii_lowercase(),
+        dictamen,
+        abiertos: aceptados.into_iter().filter(AceptadoDe::abierto).collect(),
+    })
+}
+
+/// La ronda siguiente a `anterior` (F6-6, decisiones 4 a 6 del plan de
+/// TASK-052), con `cambiados` los ficheros del diff desde el commit
+/// anterior y `tope` el tope efectivo de rondas:
+///
+/// - sin anterior: ronda 1, `completa`;
+/// - anterior `verde` y los cambios solo tocan ficheros con MENOR abiertos:
+///   `ligera` (sin modelo; la ronda no sube; los MENOR de ficheros
+///   cambiados se cierran);
+/// - anterior `rojo` y la ronda siguiente pasa de `tope`: `tope` (sin
+///   modelo; los abiertos se arrastran todos);
+/// - si no, `incremental`: la ronda sube, los MENOR de ficheros cambiados
+///   se cierran (no pasan por el cierre del modelo) y los CRITICO e
+///   IMPORTANTE siguen abiertos hasta un cierre valido.
+pub fn decidir(anterior: Option<&Anterior>, cambiados: &[String], tope: u32) -> Decision {
+    let tope = tope.max(1);
+    let Some(a) = anterior else {
+        return Decision {
+            version: VERSION,
+            ronda: 1,
+            modo: Modo::Completa,
+            sha_anterior: None,
+            tope,
+            abiertos: Vec::new(),
+            cerrados: Vec::new(),
+        };
+    };
+    let cambiado = |f: &str| cambiados.iter().any(|c| c == f);
+    let menores: BTreeSet<&str> = a
+        .abiertos
+        .iter()
+        .filter(|x| !x.grave())
+        .map(|x| x.hallazgo.fichero.as_str())
+        .collect();
+    let (modo, ronda) = if a.dictamen == Dictamen::Verde
+        && cambiados.iter().all(|c| menores.contains(c.as_str()))
+    {
+        (Modo::Ligera, a.ronda)
+    } else if a.dictamen == Dictamen::Rojo && a.ronda.saturating_add(1) > tope {
+        (Modo::Tope, a.ronda.saturating_add(1))
+    } else {
+        (Modo::Incremental, a.ronda.saturating_add(1))
+    };
+    let (cerrados, abiertos): (Vec<AceptadoDe>, Vec<AceptadoDe>) = a
+        .abiertos
+        .iter()
+        .cloned()
+        .partition(|x| modo != Modo::Tope && !x.grave() && cambiado(&x.hallazgo.fichero));
+    Decision {
+        version: VERSION,
+        ronda,
+        modo,
+        sha_anterior: Some(a.sha.clone()),
+        tope,
+        abiertos,
+        cerrados,
+    }
+}
+
+/// Una revisora del detalle de `tc2 revisoras` (`--detalle`).
+#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
+pub struct RevisoraDetalle {
+    pub id: String,
+    #[serde(default)]
+    pub rutas: Vec<String>,
+}
+
+/// Reparte los abiertos CRITICO e IMPORTANTE entre las revisoras elegidas:
+/// cada uno a su `revisora` si esta elegida; si no, a la primera que tiene
+/// su fichero entre sus rutas. Uno que no casa con ninguna no se da a nadie
+/// (nadie puede cerrarlo: sigue abierto). Los MENOR no pasan por el cierre
+/// del modelo.
+pub fn repartir(abiertos: &[AceptadoDe], detalle: &[RevisoraDetalle]) -> Vec<Vec<AceptadoDe>> {
+    let mut r: Vec<Vec<AceptadoDe>> = vec![Vec::new(); detalle.len()];
+    for a in abiertos.iter().filter(|a| a.abierto() && a.grave()) {
+        let k = detalle.iter().position(|d| d.id == a.revisora).or_else(|| {
+            detalle
+                .iter()
+                .position(|d| d.rutas.contains(&a.hallazgo.fichero))
+        });
+        if let Some(lista) = k.and_then(|k| r.get_mut(k)) {
+            lista.push(a.clone());
+        }
+    }
+    r
+}
+
+/// Los abiertos de una revisora para su prompt, en Markdown, con todo el
+/// texto del modelo por [`plano`] (dato, no instrucciones).
+pub fn abiertos_markdown(abiertos: &[AceptadoDe]) -> String {
+    if abiertos.is_empty() {
+        return "No tienes hallazgos abiertos que cerrar.\n".to_owned();
+    }
+    let mut s = String::from(
+        "| id | ronda | severidad | clase | fichero | linea | descripcion |\n|---|---|---|---|---|---|---|\n",
+    );
+    for a in abiertos {
+        let h = &a.hallazgo;
+        s.push_str(&format!(
+            "| {} | {} | {} | {} | {} | {} | {} |\n",
+            plano(&h.id, 64),
+            a.ronda,
+            h.severidad.texto(),
+            h.clase.texto(),
+            plano(&h.fichero, 200),
+            h.linea,
+            plano(&h.descripcion, 400)
+        ));
+    }
+    s
+}
+
+/// Cuantas rondas extra pide la tarea: los elementos de lista de su seccion
+/// `## Rondas extra` (F1-16; uno por ronda).
+pub fn rondas_extra(cuerpo: &crate::tarea::Cuerpo) -> u32 {
+    cuerpo.seccion("Rondas extra").map_or(0, |s| {
+        let n = s
+            .texto
+            .lines()
+            .filter(|l| l.starts_with("- ") || l.starts_with("* "))
+            .count();
+        u32::try_from(n).unwrap_or(u32::MAX)
+    })
+}
+
+#[cfg(test)]
+#[allow(clippy::panic)]
+mod tests {
+    use super::*;
+    use crate::hallazgos::Estado;
+
+    fn a(id: &str, severidad: &str, fichero: &str) -> AceptadoDe {
+        serde_json::from_str(&format!(
+            r#"{{"revisora":"rev:a","hallazgo":{{"id":"{id}","severidad":"{severidad}","clase":"correccion","fichero":"{fichero}","linea":1,"descripcion":"d","evidencia":{{"tipo":"borde","entrada":"x","esperado":"1","obtenido":"2"}}}}}}"#
+        ))
+        .unwrap_or_else(|e| panic!("{e}"))
+    }
+
+    fn previa(dictamen: Dictamen, ronda: u32, abiertos: Vec<AceptadoDe>) -> Anterior {
+        Anterior {
+            ronda,
+            sha: "a".repeat(40),
+            dictamen,
+            abiertos,
+        }
+    }
+
+    fn rutas(r: &[&str]) -> Vec<String> {
+        r.iter().map(|x| (*x).to_owned()).collect()
+    }
+
+    fn ids(v: &[AceptadoDe]) -> Vec<&str> {
+        v.iter().map(|x| x.hallazgo.id.as_str()).collect()
+    }
+
+    /// F6-6, criterio 2 (unitario): las reglas de la decision de la ronda
+    /// (RF-33; decisiones 4 a 6 del plan).
+    #[test]
+    fn decidir_aplica_tope_ligera_e_incremental() {
+        // Sin anterior (o con una incompleta, que `elegir` salta): ronda 1.
+        let d = decidir(None, &rutas(&["x"]), 2);
+        assert_eq!((d.ronda, d.modo, d.sha_anterior), (1, Modo::Completa, None));
+        // Rojo en la ronda 2 con tope 2: tope, sin modelo, con todo arrastrado
+        // (tambien el MENOR de un fichero cambiado).
+        let rojo = previa(
+            Dictamen::Rojo,
+            2,
+            vec![a("I1", "IMPORTANTE", "a.rs"), a("M1", "MENOR", "b.rs")],
+        );
+        let d = decidir(Some(&rojo), &rutas(&["a.rs", "b.rs"]), 2);
+        assert_eq!((d.ronda, d.modo, d.tope), (3, Modo::Tope, 2));
+        assert_eq!(ids(&d.abiertos), ["I1", "M1"]);
+        assert!(d.cerrados.is_empty());
+        // Una ronda extra en la tarea sube el tope: incremental.
+        let d = decidir(Some(&rojo), &rutas(&["a.rs", "b.rs"]), 3);
+        assert_eq!((d.ronda, d.modo), (3, Modo::Incremental));
+        assert_eq!(d.sha_anterior.as_deref(), Some("a".repeat(40).as_str()));
+        // El MENOR de un fichero cambiado se cierra sin modelo; el grave sigue.
+        assert_eq!(ids(&d.abiertos), ["I1"]);
+        assert_eq!(ids(&d.cerrados), ["M1"]);
+        // Verde con cambios solo en ficheros con MENOR abiertos: ligera, la
+        // ronda no sube.
+        let verde = previa(
+            Dictamen::Verde,
+            2,
+            vec![a("M1", "MENOR", "b.rs"), a("M2", "MENOR", "c.rs")],
+        );
+        let d = decidir(Some(&verde), &rutas(&["b.rs"]), 2);
+        assert_eq!((d.ronda, d.modo), (2, Modo::Ligera));
+        assert_eq!(ids(&d.cerrados), ["M1"]);
+        assert_eq!(ids(&d.abiertos), ["M2"]);
+        // Un cambio fuera de ellos: incremental (y en verde no hay tope).
+        let d = decidir(Some(&verde), &rutas(&["b.rs", "otro.rs"]), 2);
+        assert_eq!((d.ronda, d.modo), (3, Modo::Incremental));
+    }
+
+    fn check(id: u64, app: u64, titulo: &str, estado: &str) -> CheckRun {
+        CheckRun {
+            id,
+            name: "revision-ia".to_owned(),
+            status: estado.to_owned(),
+            conclusion: Some("failure".to_owned()),
+            head_sha: "b".repeat(40),
+            app: Some(AppCheck { id: app }),
+            output: Some(SalidaCheck {
+                title: Some(titulo.to_owned()),
+                summary: None,
+            }),
+        }
+    }
+
+    /// Solo cuenta el check de la App del revisor, terminado; una ronda
+    /// `incompleta` o `tope` no sirve de base y se sigue al commit anterior.
+    #[test]
+    fn elegir_salta_incompletas_y_otras_apps() {
+        let rojo = "revision-ia: rojo (ronda=1 criticos=0 importantes=1 menores=0)";
+        let commits = vec![
+            vec![check(5, 7, "revision-ia: incompleta (x)", "completed")],
+            vec![check(4, 99, rojo, "completed")],
+            vec![check(3, 7, rojo, "in_progress")],
+            vec![check(
+                2,
+                7,
+                "revision-ia: tope (ronda=3 criticos=0 importantes=1 menores=0)",
+                "completed",
+            )],
+            vec![check(1, 7, rojo, "completed")],
+        ];
+        assert_eq!(elegir(commits.clone(), 7).map(|c| c.id), Some(1));
+        assert_eq!(elegir(commits, 8), None);
+        // `anterior` exige que el check case con sus anotaciones.
+        let c = check(1, 7, rojo, "completed");
+        let i1 = a("I1", "IMPORTANTE", "a.rs");
+        let p = anterior(&c, vec![i1.clone()]).unwrap_or_else(|| panic!("no casa"));
+        assert_eq!((p.ronda, p.dictamen), (1, Dictamen::Rojo));
+        assert_eq!(anterior(&c, Vec::new()), None);
+        let mut cerrado = i1;
+        cerrado.estado = Estado::Cerrado;
+        assert_eq!(anterior(&c, vec![cerrado]), None);
+    }
+
+    /// Cada abierto grave va a su revisora si esta elegida, o a la que tiene
+    /// su fichero; los MENOR no se reparten. `## Rondas extra` cuenta los
+    /// elementos de primer nivel.
+    #[test]
+    fn repartir_y_rondas_extra() {
+        let detalle = vec![
+            RevisoraDetalle {
+                id: "rev:b".to_owned(),
+                rutas: rutas(&["a.rs"]),
+            },
+            RevisoraDetalle {
+                id: "rev:a".to_owned(),
+                rutas: rutas(&["z.rs"]),
+            },
+        ];
+        let mut otra = a("I2", "CRITICO", "a.rs");
+        otra.revisora = "rev:c".to_owned();
+        let mut suelta = a("I3", "IMPORTANTE", "nada.rs");
+        suelta.revisora = "rev:d".to_owned();
+        let r = repartir(
+            &[
+                a("I1", "IMPORTANTE", "a.rs"),
+                otra,
+                a("M1", "MENOR", "a.rs"),
+                suelta,
+            ],
+            &detalle,
+        );
+        assert_eq!(ids(&r[0]), ["I2"]);
+        assert_eq!(ids(&r[1]), ["I1"]);
+        let cuerpo = crate::tarea::Cuerpo::leer(
+            "## Rondas extra\n\n- Ronda 3: la decide Carlos.\n  - detalle\n- Ronda 4\n\n## Otra\n\n- no cuenta\n",
+        );
+        assert_eq!(rondas_extra(&cuerpo), 2);
+        assert_eq!(rondas_extra(&crate::tarea::Cuerpo::leer("## Otra\n")), 0);
+    }
+}
diff --git a/crates/tc2/src/i18n/catalogo/hallazgos.rs b/crates/tc2/src/i18n/catalogo/hallazgos.rs
index 6033788..7e24f3f 100644
--- a/crates/tc2/src/i18n/catalogo/hallazgos.rs
+++ b/crates/tc2/src/i18n/catalogo/hallazgos.rs
@@ -1,9 +1,9 @@
-//! Mensajes de `tc2 hallazgos` (F6-4).
+//! Mensajes de `tc2 hallazgos` (F6-4, F6-5 y F6-6).
 
 crate::i18n::textos! {
     USO_HALLAZGOS {
-        es: "uso: hallazgos [--json] validar --respuesta <fichero> --diff <fichero> --revisora <id> [--informe <informe.json>] [--salida <salida.json>] [--resumen <salida.md>] | hallazgos [--json] veredicto --salida <carpeta> [--hallazgos <salida-de-validar.json>]... [--extra <lineas.txt>] | hallazgos [--json] instrucciones",
-        en: "usage: hallazgos [--json] validar --respuesta <file> --diff <file> --revisora <id> [--informe <report.json>] [--salida <output.json>] [--resumen <output.md>] | hallazgos [--json] veredicto --salida <folder> [--hallazgos <validar-output.json>]... [--extra <lines.txt>] | hallazgos [--json] instrucciones",
+        es: "uso: hallazgos [--json] validar --respuesta <fichero> --diff <fichero> --revisora <id> [--informe <informe.json>] [--abiertos <abiertos.json>] [--salida <salida.json>] [--resumen <salida.md>] | hallazgos [--json] veredicto --salida <carpeta> [--hallazgos <salida-de-validar.json>]... [--extra <lineas.txt>] [--ronda <ronda.json>] | hallazgos [--json] anterior --check-runs <json>... [--origen <app-id>] | hallazgos [--json] ronda --check-runs <json>... [--anotaciones <json>]... --rutas <fichero-nul> [--tarea TASK-NNN] [--origen <app-id>] --salida <carpeta> | hallazgos [--json] abiertos --ronda <ronda.json> --detalle <detalle.json> --salida <carpeta> | hallazgos [--json] instrucciones",
+        en: "usage: hallazgos [--json] validar --respuesta <file> --diff <file> --revisora <id> [--informe <report.json>] [--abiertos <open.json>] [--salida <output.json>] [--resumen <output.md>] | hallazgos [--json] veredicto --salida <folder> [--hallazgos <validar-output.json>]... [--extra <lines.txt>] [--ronda <ronda.json>] | hallazgos [--json] anterior --check-runs <json>... [--origen <app-id>] | hallazgos [--json] ronda --check-runs <json>... [--anotaciones <json>]... --rutas <nul-file> [--tarea TASK-NNN] [--origen <app-id>] --salida <folder> | hallazgos [--json] abiertos --ronda <ronda.json> --detalle <detail.json> --salida <folder> | hallazgos [--json] instrucciones",
     }
     NO_SE_LEE {
         es: "no se puede leer ni escribir {ruta}: {error}; comprueba la ruta y los permisos y vuelve a lanzar la orden",
@@ -13,4 +13,12 @@ crate::i18n::textos! {
         es: "{ruta} pasa de {tope} bytes; recorta la entrada (el workflow corta la respuesta a 64 KiB y el diff a tope-diff)",
         en: "{ruta} is larger than {tope} bytes; trim the input (the workflow cuts the response to 64 KiB and the diff to tope-diff)",
     }
+    NO_ES_DEL_CONTRATO {
+        es: "{ruta} no es {que}: {error}; pasale el fichero tal como lo dejo el paso anterior del workflow (o la API de GitHub), sin editarlo, y vuelve a lanzar la orden",
+        en: "{ruta} is not {que}: {error}; pass the file exactly as the previous workflow step (or the GitHub API) left it, unedited, and run the command again",
+    }
+    ANTERIOR_INCOHERENTE {
+        es: "el check revision-ia {check} del commit {sha} no casa con sus anotaciones (titulo, conclusion y hallazgos abiertos): falta una pagina de anotaciones o el check esta alterado; vuelve a lanzar el workflow y, si sigue, revisa ese check en GitHub antes de seguir (la ronda no se calcula sin el: falla en cerrado)",
+        en: "the revision-ia check {check} on commit {sha} does not match its annotations (title, conclusion and open findings): an annotations page is missing or the check was altered; re-run the workflow and, if it persists, inspect that check on GitHub before going on (the round is not computed without it: fail closed)",
+    }
 }
diff --git a/crates/tc2/src/i18n/catalogo/seleccion.rs b/crates/tc2/src/i18n/catalogo/seleccion.rs
index de8a751..b22314c 100644
--- a/crates/tc2/src/i18n/catalogo/seleccion.rs
+++ b/crates/tc2/src/i18n/catalogo/seleccion.rs
@@ -2,8 +2,8 @@
 
 crate::i18n::textos! {
     USO_REVISORAS {
-        es: "uso: revisoras [--json] --rutas <fichero|-> [--nul] [--tarea TASK-NNN] [--carril <carril>] [--complejidad <nivel>] [--plugin <nombre>=<carpeta>]... [--informe <fichero> --seccion <salida> [--artefacto <nombre>]] [--detalle <salida.json>]",
-        en: "usage: revisoras [--json] --rutas <file|-> [--nul] [--tarea TASK-NNN] [--carril <lane>] [--complejidad <level>] [--plugin <name>=<folder>]... [--informe <file> --seccion <output> [--artefacto <name>]] [--detalle <output.json>]",
+        es: "uso: revisoras [--json] --rutas <fichero|-> [--nul] [--tarea TASK-NNN] [--carril <carril>] [--complejidad <nivel>] [--plugin <nombre>=<carpeta>]... [--informe <fichero> --seccion <salida> [--artefacto <nombre>]] [--ronda <n>] [--detalle <salida.json>]",
+        en: "usage: revisoras [--json] --rutas <file|-> [--nul] [--tarea TASK-NNN] [--carril <lane>] [--complejidad <level>] [--plugin <name>=<folder>]... [--informe <file> --seccion <output> [--artefacto <name>]] [--ronda <n>] [--detalle <output.json>]",
     }
     NO_SE_LEE {
         es: "no se puede leer ni escribir {ruta}: {error}",
diff --git a/crates/tc2/src/plataforma/github.rs b/crates/tc2/src/plataforma/github.rs
index 492dcf9..a5ac3c7 100644
--- a/crates/tc2/src/plataforma/github.rs
+++ b/crates/tc2/src/plataforma/github.rs
@@ -22,6 +22,7 @@ use super::{
     RevisionPublicada, Usuario, aplicar_en_lote,
 };
 use crate::contrato::{Informe, Resultado, Valor, corto};
+use crate::hallazgos::ronda;
 use crate::i18n::Mensaje;
 use crate::i18n::catalogo::plataforma as t;
 
@@ -189,28 +190,21 @@ struct CheckRunsApi {
     check_runs: Vec<CheckRunApi>,
 }
 
+/// Los check-runs de un commit para `leer-revision`, con la forma que
+/// comparte con la ronda siguiente (F6-6).
+#[derive(Deserialize)]
+struct CheckRunsRevision {
+    check_runs: Vec<ronda::CheckRun>,
+}
+
 #[derive(Deserialize)]
 struct CheckRunApi {
-    /// Solo lo usa `leer-revision` (F6-5).
-    #[serde(default)]
-    id: u64,
     name: String,
     status: String,
     #[serde(default)]
     conclusion: Option<String>,
     #[serde(default)]
     app: Option<AppApi>,
-    /// Solo lo usa `leer-revision` (F6-5).
-    #[serde(default)]
-    output: Option<SalidaCheckApi>,
-}
-
-#[derive(Deserialize, Default)]
-struct SalidaCheckApi {
-    #[serde(default)]
-    title: Option<String>,
-    #[serde(default)]
-    summary: Option<String>,
 }
 
 /// Una anotacion de un check-run: solo su `raw_details` (F6-5).
@@ -623,21 +617,15 @@ impl Plataforma for GitHub {
             return Err(self.con.invalida_por(op, &t::RAMA_BASE_O_SHA));
         }
         let (_, ruta) = self.ep(&Endpoint::CheckRuns(&p.head.sha));
-        let runs: Vec<CheckRunApi> = self.con.leer_paginas(op, rol, permiso, ruta, |c| {
+        let runs: Vec<ronda::CheckRun> = self.con.leer_paginas(op, rol, permiso, ruta, |c| {
             self.con
-                .parsear::<CheckRunsApi>(op, c)
+                .parsear::<CheckRunsRevision>(op, c)
                 .map(|x| x.check_runs)
         })?;
         // Como `estado-puertas`: el mas nuevo de ese nombre que publico su
-        // origen, y solo si ya termino.
-        let check = runs
-            .into_iter()
-            .find(|r| {
-                r.name == crate::revision::REVISION_IA
-                    && r.app.as_ref().is_some_and(|a| a.id == origen)
-            })
-            .filter(|r| r.status == "completed" && r.id != 0);
-        let Some(check) = check else {
+        // origen, y solo si ya termino (la misma regla que la ronda
+        // siguiente, F6-6).
+        let Some(check) = ronda::revision_de_origen(runs, origen) else {
             return Err(Informe::nuevo(op, Resultado::Esperando)
                 .con("pr", pr.to_string())
                 .con("motivo", "sin-check")
diff --git a/crates/tc2/src/revision/mod.rs b/crates/tc2/src/revision/mod.rs
index a00d228..a5dd22b 100644
--- a/crates/tc2/src/revision/mod.rs
+++ b/crates/tc2/src/revision/mod.rs
@@ -20,7 +20,10 @@
 //! 5. F6-5: si `revision-ia` ya termino (`ok` o `fallo`),
 //!    [`Plataforma::leer_revision`] con el agente y el informe en la carpeta
 //!    de la tarea por el punto unico de escritura ([`GuardarRevision`]:
-//!    `TASK-NNN/revision/revision-ia-<sha12>.json` y `.md`).
+//!    `TASK-NNN/revision/revision-ia-<sha12>.json` y `.md`). F6-6: tambien
+//!    antes del push, con el PR de la rama ya abierto y la tarea en
+//!    revision, el del head anterior (el de la ronda anterior), que despues
+//!    del push ya no se lee por el PR.
 //!
 //! «PR enlazado a la tarea» sin tocar el esquema (F1-9): titulo
 //! `TASK-NNN: <titulo>` y cuerpo de [`cuerpo_pr`] con las rutas de la tarea
@@ -29,7 +32,8 @@
 //!
 //! Linea: `review ok id=TASK-NNN rama=<r> pr=<n> estado=en-revision
 //! revision-ia=<ok|pendiente|fallo|sin-revisor|sin-leer>
-//! [hallazgos=<rojo|verde|incompleta|sin-leer> [informe=<ruta>]] revisoras=<ids>
+//! [hallazgos=<rojo|verde|incompleta|tope|sin-leer> [informe=<ruta> ronda=R
+//! nuevos=N cerrados=C]] revisoras=<ids>
 //! modelo=<m> reglas=<ids> avisos=<...>` (la seleccion de F6-3, anotada en
 //! `roles.revision`). Repetirla sin
 //! nada nuevo sale `sin-cambios`.
@@ -286,8 +290,34 @@ fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
     // 4. La seleccion de revision (F6-3), con el diff de la rama y lo de la
     // base; no cambia nada si falla: lo dice en `avisos`.
     let sel = seleccion_de(g, &principal, &rama, (&t, &cuerpo), e.claude);
-    // 5. Push y PR con el agente.
+    // 5. Push y PR con el agente. F6-6: antes de empujar, si el PR de la rama
+    // ya existe y su check `revision-ia` termino, el informe de ese head (la
+    // ronda anterior): despues del push el PR apunta al commit nuevo y ese
+    // check ya no se lee por el PR.
     let p = (e.conectar)()?;
+    let personas = crate::roadmap::personas(g, dir);
+    let generar = crate::roadmap::generador(&personas);
+    let ctx = estado::Contexto::nuevo(g, dir, e.base.catalogo, Some(&generar));
+    let revisor = config::de_base(g, &principal)
+        .ok()
+        .and_then(|c| c.plataforma)
+        .and_then(|p| p.revisor_id);
+    let origen = revisor.filter(|id| *id != 0);
+    let con_pr = |i: Informe, pr: u64| i.con("rama", rama.as_str()).con("pr", pr.to_string());
+    let mut previo: Option<Guardado> = None;
+    if let (Some(origen), true) = (origen, t.estado == EN_REVISION)
+        && let Some(abierto) = p
+            .prs_abiertos(Rol::Agente)
+            .ok()
+            .and_then(|l| l.into_iter().find(|x| x.rama == rama))
+        && matches!(
+            revision_ia(p.as_ref(), abierto.numero, Some(origen)),
+            "ok" | "fallo"
+        )
+    {
+        previo = guardar_informe(p.as_ref(), &ctx, (id, &quien), abierto.numero, origen)
+            .map_err(|i| con_pr(i, abierto.numero))?;
+    }
     let publicado = publicar(
         p.as_ref(),
         g,
@@ -298,9 +328,6 @@ fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
     )?;
     let pr = publicado.pr.to_string();
     // 6. El estado, con el PR ya abierto (y la seleccion en `roles.revision`).
-    let personas = crate::roadmap::personas(g, dir);
-    let generar = crate::roadmap::generador(&personas);
-    let ctx = estado::Contexto::nuevo(g, dir, e.base.catalogo, Some(&generar));
     let op = Revisar::nueva(id, &quien, ahora).con_seleccion(AnotacionRevision {
         agente: sel.agentes.first().cloned(),
         modelo: sel.modelo.clone(),
@@ -318,47 +345,28 @@ fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
         }
     }
     // 7. `revision-ia`, leido con el agente.
-    let revisor = config::de_base(g, &principal)
-        .ok()
-        .and_then(|c| c.plataforma)
-        .and_then(|p| p.revisor_id);
     let ria = revision_ia(p.as_ref(), publicado.pr, revisor);
     // 8. F6-5: con el check terminado, su informe en la carpeta de la tarea.
-    let mut hallazgos: Option<(&'static str, Option<String>)> = None;
-    let mut guardado = false;
-    if let (Some(origen), "ok" | "fallo") = (revisor.filter(|id| *id != 0), ria) {
-        let leido = p
-            .leer_revision(Rol::Agente, publicado.pr, origen)
-            .ok()
-            .and_then(|r| {
-                let d = crate::hallazgos::Dictamen::de_titulo(&r.titulo)?;
-                Some(crate::hallazgos::InformeTarea::nuevo(
-                    id,
-                    publicado.pr,
-                    &r.sha,
-                    (&r.conclusion, d),
-                    &r.resumen,
-                    r.aceptados,
-                ))
-            });
-        hallazgos = Some(("sin-leer", None));
-        if let Some(informe) = leido {
-            let op = GuardarRevision::nueva(id, &quien, &informe);
-            let gr = estado::transaccion(&ctx, &op);
-            match gr.resultado {
-                Resultado::Ok => guardado = true,
-                Resultado::SinCambios => {}
-                _ => {
-                    let mut gr = gr.con("rama", rama.as_str()).con("pr", pr);
-                    if let Some(l) = op.dueno() {
-                        gr = gr.con("asignado", l);
-                    }
-                    return Err(gr);
-                }
-            }
-            hallazgos = Some((informe.veredicto.texto(), Some(op.ruta_json())));
-        }
+    let mut leido_tras_push: Option<Option<Guardado>> = None;
+    if let (Some(origen), "ok" | "fallo") = (origen, ria) {
+        leido_tras_push = Some(
+            guardar_informe(p.as_ref(), &ctx, (id, &quien), publicado.pr, origen)
+                .map_err(|i| con_pr(i, publicado.pr))?,
+        );
     }
+    let guardado = previo.as_ref().is_some_and(|g| g.escrito)
+        || leido_tras_push
+            .as_ref()
+            .and_then(Option::as_ref)
+            .is_some_and(|g| g.escrito);
+    // La linea dice el informe del head actual si se leyo; si no, el del
+    // head anterior guardado antes del push; y `sin-leer` si el check
+    // termino y no se pudo leer.
+    let hallazgos: Option<Guardado> = match (leido_tras_push, previo) {
+        (Some(Some(g)), _) | (_, Some(g)) => Some(g),
+        (Some(None), None) => Some(Guardado::sin_leer()),
+        (None, None) => None,
+    };
     let nada_nuevo = !publicado.empujado
         && !publicado.abierto
         && !guardado
@@ -373,10 +381,14 @@ fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
         .con("pr", pr)
         .con("estado", EN_REVISION)
         .con(REVISION_IA, ria);
-    if let Some((veredicto, ruta)) = hallazgos {
-        i = i.con("hallazgos", veredicto);
-        if let Some(r) = ruta {
-            i = i.con("informe", r);
+    if let Some(g) = hallazgos {
+        i = i.con("hallazgos", g.veredicto);
+        if let Some((ruta, ronda, nuevos, cerrados)) = g.informe {
+            i = i
+                .con("informe", ruta)
+                .con("ronda", ronda.to_string())
+                .con("nuevos", nuevos.to_string())
+                .con("cerrados", cerrados.to_string());
         }
     }
     Ok(i.con_lista("revisoras", sel.revisoras)
@@ -385,6 +397,74 @@ fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
         .con_lista("avisos", sel.avisos))
 }
 
+/// Lo que dejo [`guardar_informe`] para la linea de `review`.
+struct Guardado {
+    /// El veredicto del informe, o `sin-leer`.
+    veredicto: &'static str,
+    /// Ruta del JSON en la rama de estado, ronda, nuevos y cerrados.
+    informe: Option<(String, u32, usize, usize)>,
+    /// Si se escribio algo en la rama de estado.
+    escrito: bool,
+}
+
+impl Guardado {
+    fn sin_leer() -> Self {
+        Self {
+            veredicto: "sin-leer",
+            informe: None,
+            escrito: false,
+        }
+    }
+}
+
+/// Lee con el agente el check `revision-ia` del PR `pr` (ya terminado) y
+/// guarda su informe en la carpeta de la tarea (F6-5). `Ok(None)` si no se
+/// pudo leer (o su titulo no es de tc2); `Err` si la escritura fallo.
+fn guardar_informe(
+    p: &dyn Plataforma,
+    ctx: &estado::Contexto<'_>,
+    (id, quien): (&str, &crate::identidad::Quien),
+    pr: u64,
+    origen: u64,
+) -> Result<Option<Guardado>, Informe> {
+    let leido = p.leer_revision(Rol::Agente, pr, origen).ok().and_then(|r| {
+        let d = crate::hallazgos::Dictamen::de_titulo(&r.titulo)?;
+        let ronda = crate::hallazgos::ronda_de_titulo(&r.titulo)?;
+        Some(crate::hallazgos::InformeTarea::nuevo(
+            id,
+            (pr, &r.sha, ronda),
+            (&r.conclusion, d),
+            &r.resumen,
+            r.aceptados,
+        ))
+    });
+    let Some(informe) = leido else {
+        return Ok(None);
+    };
+    let op = GuardarRevision::nueva(id, quien, &informe);
+    let gr = estado::transaccion(ctx, &op);
+    let escrito = match gr.resultado {
+        Resultado::Ok => true,
+        Resultado::SinCambios => false,
+        _ => {
+            return Err(match op.dueno() {
+                Some(l) => gr.con("asignado", l),
+                None => gr,
+            });
+        }
+    };
+    Ok(Some(Guardado {
+        veredicto: informe.veredicto.texto(),
+        informe: Some((
+            op.ruta_json(),
+            informe.ronda,
+            informe.nuevos,
+            informe.cerrados,
+        )),
+        escrito,
+    }))
+}
+
 /// La seleccion de revision de la tarea `t` en la rama `rama` (F6-3): rutas
 /// de `git diff --no-renames origin/develop...<rama>`, inventario y catalogo
 /// de [`seleccion::orden::fuentes`], carril y complejidad de
diff --git a/crates/tc2/src/seleccion/mod.rs b/crates/tc2/src/seleccion/mod.rs
index 0b63154..c939493 100644
--- a/crates/tc2/src/seleccion/mod.rs
+++ b/crates/tc2/src/seleccion/mod.rs
@@ -11,7 +11,8 @@
 //!
 //! Lo que F6-3 no evalua todavia (adenda F6-3 de F1-15): sin el detector
 //! `dominios()` de F4-6 la lista de dominios es vacia, asi que S10 y S11 no
-//! se cumplen nunca; sin rondas de F6-6 la ronda es 1 y S13 no se cumple.
+//! se cumplen nunca. La ronda (F6-6) la da `--ronda` de `tc2 revisoras`
+//! (en CI, la de `tc2 hallazgos ronda`); sin ella es 1 y S13 no se cumple.
 //! Las filas se leen igual y su vocabulario se valida.
 
 pub mod catalogo;
diff --git a/crates/tc2/src/seleccion/orden.rs b/crates/tc2/src/seleccion/orden.rs
index a07c582..a91fa78 100644
--- a/crates/tc2/src/seleccion/orden.rs
+++ b/crates/tc2/src/seleccion/orden.rs
@@ -1,7 +1,7 @@
 //! `tc2 revisoras [--json] --rutas <fichero|-> [--nul] [--tarea TASK-NNN]
 //! [--carril <c>] [--complejidad <x>] [--plugin <nombre>=<dir>]...
 //! [--informe <fichero> --seccion <salida> [--artefacto <n>]]
-//! [--detalle <salida.json>]` (F6-3).
+//! [--ronda <n>] [--detalle <salida.json>]` (F6-3).
 //!
 //! Las rutas son dato (una por linea, o separadas por NUL con `--nul`; los
 //! dos nombres de cada renombrado), con un tope de [`TOPE_RUTAS`] bytes
@@ -32,6 +32,10 @@
 //! [`seccion_revisor`] y la linea dice `informe=verde|rojo`; si no,
 //! `informe=grande|invalido|sin-informe` y no se escribe seccion.
 //!
+//! `--ronda` (F6-6): la ronda de la revision (por defecto 1), para las
+//! reglas con `ronda>=N` (S13). En CI sale de `ronda.json` de `tc2 hallazgos
+//! ronda`.
+//!
 //! Linea: `revisoras ok revisoras=<ids> agentes=<ids> modelo=<m>
 //! modelo-api=<id> reglas=<ids> avisos=<...> carril=<c> complejidad=<x>
 //! [informe=<...>]`. Lo de cada revisora (su `SKILL.md` y las rutas que
@@ -248,6 +252,7 @@ struct Args {
     artefacto: Option<String>,
     seccion: Option<PathBuf>,
     detalle: Option<PathBuf>,
+    ronda: Option<u32>,
 }
 
 /// Las rutas del texto: con `nul`, separadas por NUL y literales (la salida
@@ -313,6 +318,14 @@ fn args(a: &[String]) -> Result<Args, Informe> {
             "--artefacto" if r.artefacto.is_none() => r.artefacto = Some(valor()?),
             "--seccion" if r.seccion.is_none() => r.seccion = Some(PathBuf::from(valor()?)),
             "--detalle" if r.detalle.is_none() => r.detalle = Some(PathBuf::from(valor()?)),
+            "--ronda" if r.ronda.is_none() => {
+                let n = valor()?
+                    .parse::<u32>()
+                    .ok()
+                    .filter(|n| *n >= 1)
+                    .ok_or_else(|| uso("ronda-invalida"))?;
+                r.ronda = Some(n);
+            }
             _ => return Err(uso("argumento-desconocido")),
         }
     }
@@ -391,7 +404,7 @@ fn paso(a: &[String], e: &Entorno<'_>) -> Result<Informe, Informe> {
         carril,
         complejidad: cx.as_deref(),
         modelos_permitidos: &config.seleccion.modelos_permitidos,
-        ronda: 1,
+        ronda: a.ronda.unwrap_or(1),
     };
     let mut s = super::revision(&rutas, &f.catalogo, &f.inventario, &ctx).map_err(|x| {
         Informe::nuevo(NOMBRE, Resultado::Error)
diff --git a/xtask/src/workflows.rs b/xtask/src/workflows.rs
index 64a5292..86ee117 100644
--- a/xtask/src/workflows.rs
+++ b/xtask/src/workflows.rs
@@ -2209,6 +2209,64 @@ fn publicar_transporta_el_veredicto(texto: &str) -> Result<(), String> {
     Ok(())
 }
 
+/// F6-6: `preparar` lee el check de la ronda anterior (`checks: read`), corre
+/// `tc2 hallazgos anterior` y `tc2 hallazgos ronda` y pasa la ronda a `tc2
+/// revisoras`; `validar` pasa los abiertos a `validar` y la ronda a
+/// `veredicto`. Sin una de esas piezas el workflow seguiria dando siempre la
+/// ronda 1 (o un `incompleta`) y solo el E2E lo veria.
+fn rondas_en_el_workflow(texto: &str) -> Result<(), String> {
+    let raiz = leer_yaml(texto)?;
+    let checks = raiz
+        .get("jobs")
+        .and_then(|j| j.get("preparar"))
+        .and_then(|j| j.get("permissions"))
+        .and_then(|p| p.get("checks"))
+        .and_then(Nodo::texto);
+    if checks != Some("read") {
+        return Err("preparar sin checks: read".to_owned());
+    }
+    let preparar = runs_del_job(&raiz, "preparar");
+    for (pieza, error) in [
+        (
+            "tc2 hallazgos anterior ",
+            "preparar no corre tc2 hallazgos anterior",
+        ),
+        (
+            "tc2 hallazgos ronda ",
+            "preparar no corre tc2 hallazgos ronda",
+        ),
+        (
+            "tc2 hallazgos abiertos ",
+            "preparar no corre tc2 hallazgos abiertos",
+        ),
+    ] {
+        if !preparar.contains(pieza) {
+            return Err(error.to_owned());
+        }
+    }
+    let revisoras = preparar
+        .split("tc2 revisoras ")
+        .nth(1)
+        .and_then(|r| r.split("> \"$salida/seleccion.json\"").next())
+        .unwrap_or_default();
+    if !revisoras.contains("--ronda \"$numero\"") {
+        return Err("tc2 revisoras sin --ronda".to_owned());
+    }
+    let validar = runs_del_job(&raiz, "validar");
+    let veredicto = validar
+        .split("tc2 hallazgos veredicto ")
+        .nth(1)
+        .and_then(|r| r.lines().next())
+        .unwrap_or_default();
+    if !veredicto.contains("\"${ronda[@]}\"") || !validar.contains("ronda=(--ronda ") {
+        return Err("veredicto sin --ronda".to_owned());
+    }
+    if !validar.contains("abiertos=(--abiertos ") || !validar.contains("\"${abiertos[@]}\"") {
+        return Err("validar sin --abiertos".to_owned());
+    }
+    Ok(())
+}
+
 /// El workflow del revisor pasa el lint con el perfil revision-ia: cuatro
 /// jobs (F6-4); `consultar` y `publicar` en el entorno `revisor`, sin
 /// checkout, cada uno solo con su secreto; `preparar` y `validar` sin
@@ -2275,6 +2333,7 @@ fn el_workflow_del_revisor_pasa_el_lint() {
     );
     assert_eq!(validar_sube_lo_que_lee_publicar(&texto), Ok(()));
     assert_eq!(publicar_transporta_el_veredicto(&texto), Ok(()));
+    assert_eq!(rondas_en_el_workflow(&texto), Ok(()));
     assert!(
         runs_de("preparar")
             .iter()
@@ -2469,6 +2528,52 @@ fn cada_mutacion_del_revisor_da_su_error() {
             "«{esperado}» no salio: {r:?}"
         );
     }
+    // F6-6: una mutacion por regla de las rondas da rojo.
+    let rondas: [(&str, &str, &str); 7] = [
+        (
+            "      actions: read\n      checks: read\n    outputs:",
+            "      actions: read\n    outputs:",
+            "preparar sin checks: read",
+        ),
+        (
+            "linea=$(cd base && tc2 hallazgos anterior ",
+            "linea=$(cd base && tc2 hallazgos previo ",
+            "preparar no corre tc2 hallazgos anterior",
+        ),
+        (
+            "(cd base && tc2 hallazgos ronda ",
+            "(cd base && tc2 hallazgos rondas ",
+            "preparar no corre tc2 hallazgos ronda",
+        ),
+        (
+            "              tc2 hallazgos abiertos --ronda",
+            "              tc2 hallazgos repartir --ronda",
+            "preparar no corre tc2 hallazgos abiertos",
+        ),
+        (
+            " --ronda \"$numero\" --detalle",
+            " --detalle",
+            "tc2 revisoras sin --ronda",
+        ),
+        (
+            "\"$out/resumen.txt\" \"${ronda[@]}\" \"${veredicto[@]}\"",
+            "\"$out/resumen.txt\" \"${veredicto[@]}\"",
+            "veredicto sin --ronda",
+        ),
+        (
+            "\"${informe[@]}\" \"${abiertos[@]}\" \\",
+            "\"${informe[@]}\" \\",
+            "validar sin --abiertos",
+        ),
+    ];
+    for (original, mutado, esperado) in rondas {
+        assert!(texto.contains(original), "el caso no casa: {original}");
+        let r = rondas_en_el_workflow(&texto.replacen(original, mutado, 1));
+        assert!(
+            r.as_ref().is_err_and(|e| e.contains(esperado)),
+            "«{esperado}» no salio: {r:?}"
+        );
+    }
 }
 
 /// El llamador de la guia de F6-3 pasa el lint de su perfil; `secrets:
@@ -2516,6 +2621,17 @@ fn llamador_de_revision_ia_hereda_solo_hacia_revision_ia() {
             .iter()
             .any(|e| e.contains("secrets: inherit"))
     );
+    // F6-6: el llamador da checks: read (preparar lee la ronda anterior) y
+    // nunca checks: write.
+    assert!(llamador.contains("      checks: read\n"), "{llamador}");
+    let checks_escritura = llamador.replace("      checks: read\n", "      checks: write\n");
+    assert!(
+        lint_llamador_revision(&checks_escritura)
+            .iter()
+            .any(|e| e.contains("escritura")),
+        "{:?}",
+        lint_llamador_revision(&checks_escritura)
+    );
     // El llamador no escribe con el GITHUB_TOKEN.
     let con_escritura = llamador.replace("      contents: read\n", "      contents: write\n");
     assert_ne!(con_escritura, llamador);
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-052/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-052/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-052/tarea.md                                    | 3 ++-
 3 files changed, 2 insertions(+), 1 deletion(-)
````

## Tests del diff (4 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/hallazgos.rs" ":(literal)crates/tc2/tests/plataforma_e2e.rs" ":(literal)crates/tc2/tests/revision.rs" ":(literal)crates/tc2/tests/revisoras.rs"
````

````
crates/tc2/tests/hallazgos.rs      | 251 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-----
 crates/tc2/tests/plataforma_e2e.rs |   3 +-
 crates/tc2/tests/revision.rs       | 171 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++----
 crates/tc2/tests/revisoras.rs      |  16 ++++++++++
 4 files changed, 426 insertions(+), 15 deletions(-)
````
