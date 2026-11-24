# Peticion de revision — TASK-140 (ronda 1)

- Tarea: TASK-140 — Auditoria 4D: A2 hallazgos y revision (7 IMPORTANTE)
- Rama revisada: fix/task-140-auditoria-4d-a2-hallazgos-y-revision-7-i
- Rama base: develop
- Commit revisado (HEAD): 468ef2f20633db8d12b6b4b92a44665c0eefd3f7
- Fecha: 2026-11-24
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-140 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-140/tarea.md
- tareas/03-en-revision/TASK-140/planificacion/
- crates/tc2/src/hallazgos/mod.rs
- crates/tc2/src/hallazgos/orden.rs
- crates/tc2/src/hallazgos/ronda.rs
- crates/tc2/tests/hallazgos.rs

## Commits a revisar (git log develop..HEAD)

````
468ef2f docs(TASK-140): resultado, criterios y autorrevision
ef2855a fix(TASK-140): A2-05 analizador de opciones comun para las subordenes de hallazgos
8cdddda fix(TASK-140): A2-04 celdas y fila unicas para las tres tablas de hallazgos
117aa2f fix(TASK-140): A2-03 cortar y Fallo::rechazo unicos para hallazgos y cierres
b0bedef fix(TASK-140): A2-02 fallo_de unico para validar_uno y validar_cierre
e485bd3 fix(TASK-140): A2-12 test de la mezcla con conflictos en desde_incremental
17a2aa4 fix(TASK-140): A2-11 test del dictamen de veredicto_ronda en tope y ligera
df6c0dc fix(TASK-140): A2-13 test del tope de cierres fuera-de-tope
4667b9f chore(TASK-140): tarea en curso
````

## Diff de tu dominio (4 fichero(s) de 4; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/hallazgos/mod.rs b/crates/tc2/src/hallazgos/mod.rs
index 8755dad..4c6f3a0 100644
--- a/crates/tc2/src/hallazgos/mod.rs
+++ b/crates/tc2/src/hallazgos/mod.rs
@@ -629,6 +629,47 @@ struct Fallo {
     detalle: String,
 }
 
+impl Fallo {
+    /// El rechazo de la entrada `indice` de la respuesta.
+    fn rechazo(self, indice: usize) -> Rechazo {
+        Rechazo {
+            indice: Some(indice),
+            id: self.id,
+            motivo: self.motivo,
+            detalle: self.detalle,
+        }
+    }
+}
+
+/// Las `TOPE_HALLAZGOS` primeras entradas de `lista` y, si sobran, un unico
+/// rechazo `fuera-de-tope` con el indice de la primera que sobra (las de
+/// `lista` se numeran desde `primero`); `que` las nombra en el detalle.
+fn cortar<'a>(lista: &'a [Value], primero: usize, que: &str) -> (&'a [Value], Option<Rechazo>) {
+    let (dentro, fuera) = lista.split_at(lista.len().min(TOPE_HALLAZGOS));
+    let rechazo = (!fuera.is_empty()).then(|| Rechazo {
+        indice: Some(primero + TOPE_HALLAZGOS),
+        id: None,
+        motivo: motivo::FUERA_DE_TOPE,
+        detalle: format!("{} {que} pasan del tope de {TOPE_HALLAZGOS}", fuera.len()),
+    });
+    (dentro, rechazo)
+}
+
+/// El constructor de los `Fallo` de un hallazgo o un cierre: con su `id` si
+/// lo trae y es valido, para que el rechazo lo nombre.
+fn fallo_de(v: &Value) -> impl Fn(&'static str, String) -> Fallo + use<> {
+    let id = v
+        .get("id")
+        .and_then(Value::as_str)
+        .filter(|s| es_id(s))
+        .map(str::to_owned);
+    move |motivo, detalle| Fallo {
+        id: id.clone(),
+        motivo,
+        detalle,
+    }
+}
+
 /// Que la evidencia traiga los campos de su tipo, llenos, y ninguno ajeno.
 fn evidencia_completa(e: &Evidencia) -> Result<(), String> {
     let texto = |o: &Option<String>| o.as_deref().map(no_vacio);
@@ -692,16 +733,7 @@ fn validar_uno(
             detalle: "el hallazgo no es un objeto".to_owned(),
         });
     };
-    let id = obj
-        .get("id")
-        .and_then(Value::as_str)
-        .filter(|s| es_id(s))
-        .map(str::to_owned);
-    let fallo = |motivo: &'static str, detalle: String| Fallo {
-        id: id.clone(),
-        motivo,
-        detalle,
-    };
+    let fallo = fallo_de(h);
     let evidencia = match obj.get("evidencia") {
         None | Some(Value::Null) => {
             return Err(fallo(
@@ -825,16 +857,7 @@ fn validar_cierre<'a>(
     abiertos: &'a [AceptadoDe],
     vistos: &mut BTreeSet<String>,
 ) -> Result<(Cierre, Vec<&'a AceptadoDe>), Fallo> {
-    let id = c
-        .get("id")
-        .and_then(Value::as_str)
-        .filter(|s| es_id(s))
-        .map(str::to_owned);
-    let fallo = |motivo: &'static str, detalle: String| Fallo {
-        id: id.clone(),
-        motivo,
-        detalle,
-    };
+    let fallo = fallo_de(c);
     let c: Cierre = serde_json::from_value(c.clone())
         .map_err(|e| fallo(motivo::HALLAZGO_INVALIDO, format!("cierre: {e}")))?;
     if !es_id(&c.id) || c.linea == 0 || !no_vacio(&c.fichero) || !no_vacio(&c.motivo) {
@@ -953,44 +976,17 @@ pub fn validar_con(
     };
     // Los ids de los abiertos no se reutilizan: un cierre los nombra.
     let mut vistos: BTreeSet<String> = abiertos.iter().map(|a| a.hallazgo.id.clone()).collect();
-    for (i, h) in lista.iter().enumerate() {
-        if i >= TOPE_HALLAZGOS {
-            v.rechazados.push(Rechazo {
-                indice: Some(i),
-                id: None,
-                motivo: motivo::FUERA_DE_TOPE,
-                detalle: format!(
-                    "{} hallazgos pasan del tope de {TOPE_HALLAZGOS}",
-                    lista.len() - TOPE_HALLAZGOS
-                ),
-            });
-            break;
-        }
+    let (dentro, sobran) = cortar(&lista, 0, "hallazgos");
+    for (i, h) in dentro.iter().enumerate() {
         match validar_uno(h, diff, informe, &mut vistos) {
             Ok(a) => v.aceptados.push(a),
-            Err(f) => v.rechazados.push(Rechazo {
-                indice: Some(i),
-                id: f.id,
-                motivo: f.motivo,
-                detalle: f.detalle,
-            }),
+            Err(f) => v.rechazados.push(f.rechazo(i)),
         }
     }
+    v.rechazados.extend(sobran);
     let mut cerrados_vistos = BTreeSet::new();
-    for (j, c) in cierres.iter().enumerate() {
-        let indice = lista.len() + j;
-        if j >= TOPE_HALLAZGOS {
-            v.rechazados.push(Rechazo {
-                indice: Some(indice),
-                id: None,
-                motivo: motivo::FUERA_DE_TOPE,
-                detalle: format!(
-                    "{} cierres pasan del tope de {TOPE_HALLAZGOS}",
-                    cierres.len() - TOPE_HALLAZGOS
-                ),
-            });
-            break;
-        }
+    let (dentro, sobran) = cortar(&cierres, lista.len(), "cierres");
+    for (j, c) in dentro.iter().enumerate() {
         match validar_cierre(c, diff, informe, abiertos, &mut cerrados_vistos) {
             Ok((c, cerrados)) => {
                 for a in cerrados {
@@ -1004,14 +1000,10 @@ pub fn validar_con(
                     });
                 }
             }
-            Err(f) => v.rechazados.push(Rechazo {
-                indice: Some(indice),
-                id: f.id,
-                motivo: f.motivo,
-                detalle: f.detalle,
-            }),
+            Err(f) => v.rechazados.push(f.rechazo(lista.len() + j)),
         }
     }
+    v.rechazados.extend(sobran);
     v
 }
 
@@ -1178,6 +1170,25 @@ pub fn prosa(revisora: &str, respuesta: &str) -> String {
     }
 }
 
+/// Las seis celdas de un hallazgo en una tabla Markdown (id, severidad,
+/// clase, fichero, linea y descripcion), con el texto del modelo por
+/// [`plano`] y sus topes; cada tabla pone sus columnas propias entre ellas.
+fn celdas(h: &Aceptado) -> [String; 6] {
+    [
+        plano(&h.id, 64),
+        h.severidad.texto().to_owned(),
+        h.clase.texto().to_owned(),
+        plano(&h.fichero, 200),
+        h.linea.to_string(),
+        plano(&h.descripcion, 400),
+    ]
+}
+
+/// Una fila de tabla Markdown con esas celdas, ya pasadas por [`plano`].
+fn fila(celdas: &[String]) -> String {
+    format!("| {} |\n", celdas.join(" | "))
+}
+
 /// La seccion del check `revision-ia` para una revisora: las tablas de
 /// aceptados y rechazados, con todo el texto del modelo pasado por
 /// [`plano`].
@@ -1191,16 +1202,17 @@ pub fn resumen_markdown(v: &Validados) -> String {
     if !v.aceptados.is_empty() {
         s.push_str("| id | severidad | clase | fichero | linea | evidencia | descripcion |\n|---|---|---|---|---|---|---|\n");
         for a in &v.aceptados {
-            s.push_str(&format!(
-                "| {} | {} | {} | {} | {} | {} | {} |\n",
-                plano(&a.id, 64),
-                a.severidad.texto(),
-                a.clase.texto(),
-                plano(&a.fichero, 200),
-                a.linea,
-                a.evidencia.tipo.texto(),
-                plano(&a.descripcion, 400)
-            ));
+            let [id, severidad, clase, fichero, linea, descripcion] = celdas(a);
+            let evidencia = a.evidencia.tipo.texto().to_owned();
+            s.push_str(&fila(&[
+                id,
+                severidad,
+                clase,
+                fichero,
+                linea,
+                evidencia,
+                descripcion,
+            ]));
         }
         s.push('\n');
     }
@@ -1865,19 +1877,19 @@ impl InformeTarea {
             );
             for a in &self.aceptados {
                 let h = &a.hallazgo;
-                s.push_str(&format!(
-                    "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
+                let [id, severidad, clase, fichero, linea, descripcion] = celdas(h);
+                s.push_str(&fila(&[
                     plano(&a.revisora, 120),
-                    a.ronda,
-                    a.estado.texto(),
-                    plano(&h.id, 64),
-                    h.severidad.texto(),
-                    h.clase.texto(),
-                    plano(&h.fichero, 200),
-                    h.linea,
-                    h.evidencia.tipo.texto(),
-                    plano(&h.descripcion, 400)
-                ));
+                    a.ronda.to_string(),
+                    a.estado.texto().to_owned(),
+                    id,
+                    severidad,
+                    clase,
+                    fichero,
+                    linea,
+                    h.evidencia.tipo.texto().to_owned(),
+                    descripcion,
+                ]));
             }
         }
         s
@@ -2424,6 +2436,21 @@ mod tests {
         );
         assert_eq!(v.motivos(), ["I1:id-repetido"]);
         assert_eq!(v.cierres.len(), 1);
+        // Mas de TOPE_HALLAZGOS cierres: los que sobran, un rechazo con el
+        // indice del primero que pasa (sin hallazgos, el del cierre 100).
+        let muchos = vec![cierre("I1", 12); TOPE_HALLAZGOS + 1];
+        let v = validar_con("r", &con(&muchos), &d, Some(&informe), &abiertos);
+        let motivos = v.motivos();
+        assert_eq!(
+            motivos.last(),
+            Some(&format!("#{TOPE_HALLAZGOS}:fuera-de-tope"))
+        );
+        assert_eq!(motivos.len(), TOPE_HALLAZGOS, "{motivos:?}");
+        assert_eq!(
+            v.rechazados.last().map(|r| r.detalle.as_str()),
+            Some(format!("1 cierres pasan del tope de {TOPE_HALLAZGOS}").as_str())
+        );
+        assert_eq!(v.cierres.len(), 1);
         // MEN-4 (rust) de la ronda 1: sin `hallazgos` (como sin `cierres`),
         // es una lista vacia y el cierre valido no se pierde.
         let solo_cierres = format!("```json\n{{\"cierres\": [{}]}}\n```", cierre("I1", 12));
@@ -2454,6 +2481,35 @@ mod tests {
         assert_eq!(v.aceptados.len(), 1);
     }
 
+    /// A2-04: la fila de un hallazgo sale entera, con el texto por `plano`,
+    /// en la tabla del check y en la de los abiertos del prompt (la del
+    /// informe de la tarea la fija `tests/revision.rs`).
+    #[test]
+    fn la_fila_de_un_hallazgo_sale_entera_en_cada_tabla() {
+        let mut a = abierto("I1", "IMPORTANTE", 12, EV_BORDE);
+        a.ronda = 2;
+        a.hallazgo.descripcion = "falta | algo".to_owned();
+        let v = Validados {
+            version: VERSION,
+            revisora: "r".to_owned(),
+            aceptados: vec![a.hallazgo.clone()],
+            rechazados: Vec::new(),
+            cierres: Vec::new(),
+        };
+        let md = resumen_markdown(&v);
+        assert!(
+            md.contains("\n| I1 | IMPORTANTE | correccion | src/calculo.rs | 12 | borde | falta \\| algo |\n"),
+            "{md}"
+        );
+        let md = ronda::abiertos_markdown(&[a]);
+        assert!(
+            md.ends_with(
+                "\n| I1 | 2 | IMPORTANTE | correccion | src/calculo.rs | 12 | falta \\| algo |\n"
+            ),
+            "{md}"
+        );
+    }
+
     /// MEN-1 (rust) de la ronda 1: el informe de la tarea cuenta nuevos,
     /// persisten y cerrados igual que el check, tambien en una ronda `ligera`
     /// (la ronda no sube y no hay nada nuevo). El modo sale de la linea
@@ -2494,6 +2550,8 @@ mod tests {
         assert_eq!(d.modo, ronda::Modo::Ligera);
         let v = veredicto_ronda(&[], Some(&d));
         assert_eq!((v.nuevos, v.persisten, v.cerrados), (0, 1, 1));
+        // Ligera sin filas: la ronda no sube y sale verde.
+        assert_eq!(v.dictamen, Dictamen::Verde);
         cuenta(&v);
         // Incremental con un nuevo: ese cuenta como nuevo y M2 persiste.
         let d = ronda::decidir(
@@ -2516,7 +2574,10 @@ mod tests {
         rojo.abiertos
             .push(abierto("I1", "IMPORTANTE", 12, EV_BORDE));
         let d = ronda::decidir(Some(&rojo), &[], 2, false);
-        cuenta(&veredicto_ronda(&[], Some(&d)));
+        let v = veredicto_ronda(&[], Some(&d));
+        // Tope con un IMPORTANTE abierto: nunca verde (la puerta no pasa).
+        assert_eq!((v.modo, v.dictamen), (ronda::Modo::Tope, Dictamen::Tope));
+        cuenta(&v);
     }
 
     /// F6-6, criterio 2: lo publicado por F6-5 (v2) se sigue leyendo: un
diff --git a/crates/tc2/src/hallazgos/orden.rs b/crates/tc2/src/hallazgos/orden.rs
index e919fa8..f2d256d 100644
--- a/crates/tc2/src/hallazgos/orden.rs
+++ b/crates/tc2/src/hallazgos/orden.rs
@@ -58,6 +58,7 @@
 //!   workflow ya no lo arma. Linea: `hallazgos ok revisora=<id> bytes=<n>
 //!   cortado=<si|no> abiertos=<si|no>`.
 
+use std::collections::BTreeMap;
 use std::io::Read;
 use std::path::{Path, PathBuf};
 
@@ -87,7 +88,6 @@ pub const TOPE_VALIDADOS: u64 = 4 * 1024 * 1024;
 /// Bytes que se leen como mucho de `--extra`.
 pub const TOPE_EXTRA: u64 = 64 * 1024;
 
-#[derive(Default)]
 struct Args {
     respuesta: Option<PathBuf>,
     diff: Option<PathBuf>,
@@ -114,29 +114,93 @@ fn es_revisora(t: &str) -> bool {
             .all(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '.' | '_' | '-' | '/'))
 }
 
-fn args(a: &[String]) -> Result<Args, Informe> {
-    let mut r = Args::default();
+/// Las opciones de una suborden, por nombre, con sus valores en orden.
+struct Opciones(BTreeMap<&'static str, Vec<String>>);
+
+impl Opciones {
+    /// El valor de una opcion que va como mucho una vez.
+    fn una(&self, nombre: &str) -> Option<String> {
+        self.0.get(nombre).and_then(|v| v.first()).cloned()
+    }
+
+    /// El valor de una opcion unica, como ruta.
+    fn ruta(&self, nombre: &str) -> Option<PathBuf> {
+        self.una(nombre).map(PathBuf::from)
+    }
+
+    /// Los valores de una opcion repetible, como rutas, en el orden dado.
+    fn rutas(&self, nombre: &str) -> Vec<PathBuf> {
+        self.0
+            .get(nombre)
+            .map(|v| v.iter().map(PathBuf::from).collect())
+            .unwrap_or_default()
+    }
+}
+
+/// Analiza `--opcion valor` de una suborden: cada opcion lleva un valor; las
+/// de `unicas` van como mucho una vez y las de `repetibles`, las que se
+/// quiera. Una opcion que no es de ninguna, o una unica repetida (antes de
+/// leer su valor), es `argumento-desconocido`; una sin valor al final,
+/// `falta-valor`. Que el valor valga lo mira cada suborden despues.
+fn opciones(
+    a: &[String],
+    unicas: &[&'static str],
+    repetibles: &[&'static str],
+) -> Result<Opciones, Informe> {
+    let mut o: BTreeMap<&'static str, Vec<String>> = BTreeMap::new();
     let mut it = a.iter();
     while let Some(x) = it.next() {
-        let mut valor = || it.next().cloned().ok_or_else(|| uso("falta-valor"));
-        match x.as_str() {
-            "--respuesta" if r.respuesta.is_none() => r.respuesta = Some(valor()?.into()),
-            "--diff" if r.diff.is_none() => r.diff = Some(valor()?.into()),
-            "--revisora" if r.revisora.is_none() => {
-                let v = valor()?;
-                if !es_revisora(&v) {
-                    return Err(uso("revisora-invalida"));
-                }
-                r.revisora = Some(v);
-            }
-            "--informe" if r.informe.is_none() => r.informe = Some(valor()?.into()),
-            "--salida" if r.salida.is_none() => r.salida = Some(valor()?.into()),
-            "--resumen" if r.resumen.is_none() => r.resumen = Some(valor()?.into()),
-            "--abiertos" if r.abiertos.is_none() => r.abiertos = Some(valor()?.into()),
-            "--prosa" if r.prosa.is_none() => r.prosa = Some(valor()?.into()),
-            _ => return Err(uso("argumento-desconocido")),
+        let Some(nombre) = unicas.iter().chain(repetibles).find(|n| **n == x.as_str()) else {
+            return Err(uso("argumento-desconocido"));
+        };
+        let valores = o.entry(nombre).or_default();
+        if unicas.contains(nombre) && !valores.is_empty() {
+            return Err(uso("argumento-desconocido"));
         }
+        valores.push(it.next().cloned().ok_or_else(|| uso("falta-valor"))?);
     }
+    Ok(Opciones(o))
+}
+
+/// `Err(motivo)` de uso si el valor de `nombre` esta y no cumple `vale`.
+fn comprobar(
+    o: &Opciones,
+    nombre: &str,
+    vale: fn(&str) -> bool,
+    motivo: &str,
+) -> Result<(), Informe> {
+    match o.una(nombre) {
+        Some(v) if !vale(&v) => Err(uso(motivo)),
+        _ => Ok(()),
+    }
+}
+
+fn args(a: &[String]) -> Result<Args, Informe> {
+    let o = opciones(
+        a,
+        &[
+            "--respuesta",
+            "--diff",
+            "--revisora",
+            "--informe",
+            "--salida",
+            "--resumen",
+            "--abiertos",
+            "--prosa",
+        ],
+        &[],
+    )?;
+    comprobar(&o, "--revisora", es_revisora, "revisora-invalida")?;
+    let r = Args {
+        respuesta: o.ruta("--respuesta"),
+        diff: o.ruta("--diff"),
+        revisora: o.una("--revisora"),
+        informe: o.ruta("--informe"),
+        salida: o.ruta("--salida"),
+        resumen: o.ruta("--resumen"),
+        abiertos: o.ruta("--abiertos"),
+        prosa: o.ruta("--prosa"),
+    };
     for (falta, motivo) in [
         (r.respuesta.is_none(), "sin-respuesta"),
         (r.diff.is_none(), "sin-diff"),
@@ -285,22 +349,10 @@ fn escribir_json(p: &Path, v: &serde_json::Value) -> Result<(), Informe> {
 }
 
 fn veredicto(a: &[String]) -> Result<Informe, Informe> {
-    let mut salida: Option<PathBuf> = None;
-    let mut extra: Option<PathBuf> = None;
-    let mut decision: Option<PathBuf> = None;
-    let mut entradas: Vec<PathBuf> = Vec::new();
-    let mut it = a.iter();
-    while let Some(x) = it.next() {
-        let mut valor = || it.next().cloned().ok_or_else(|| uso("falta-valor"));
-        match x.as_str() {
-            "--hallazgos" => entradas.push(valor()?.into()),
-            "--salida" if salida.is_none() => salida = Some(valor()?.into()),
-            "--extra" if extra.is_none() => extra = Some(valor()?.into()),
-            "--ronda" if decision.is_none() => decision = Some(valor()?.into()),
-            _ => return Err(uso("argumento-desconocido")),
-        }
-    }
-    let Some(dir) = salida else {
+    let o = opciones(a, &["--salida", "--extra", "--ronda"], &["--hallazgos"])?;
+    let (extra, decision, entradas) =
+        (o.ruta("--extra"), o.ruta("--ronda"), o.rutas("--hallazgos"));
+    let Some(dir) = o.ruta("--salida") else {
         return Err(uso("sin-salida"));
     };
     let extra = match &extra {
@@ -433,17 +485,9 @@ fn leer_checks(ficheros: &[PathBuf]) -> Result<Vec<Vec<super::ronda::CheckRun>>,
 /// ronda anterior ([`super::ronda::elegir`]). Linea: `hallazgos ok
 /// anterior=<id>|ninguno [sha=<sha>] [avisos=...]`.
 fn anterior(a: &[String]) -> Result<Informe, Informe> {
-    let mut checks: Vec<PathBuf> = Vec::new();
-    let mut origen: Option<u64> = None;
-    let mut it = a.iter();
-    while let Some(x) = it.next() {
-        let mut valor = || it.next().cloned().ok_or_else(|| uso("falta-valor"));
-        match x.as_str() {
-            "--check-runs" => checks.push(valor()?.into()),
-            "--origen" if origen.is_none() => origen = Some(id_de_app(&valor()?)?),
-            _ => return Err(uso("argumento-desconocido")),
-        }
-    }
+    let o = opciones(a, &["--origen"], &["--check-runs"])?;
+    let origen = o.una("--origen").as_deref().map(id_de_app).transpose()?;
+    let checks = o.rutas("--check-runs");
     let dir = carpeta_actual()?;
     let (origen, _, avisos) = origen_y_config(origen, &crate::git::Git::nuevo(), &dir);
     let commits = leer_checks(&checks)?;
@@ -486,33 +530,22 @@ fn extra_de_tarea(g: &crate::git::Git, dir: &Path, id: &str) -> Result<u32, Stri
 /// tarea. Linea: `hallazgos ok ronda=R modo=<m> anterior=<sha12|ninguno>
 /// abiertos=N tope=T avisos=<...>`.
 fn ronda(a: &[String]) -> Result<Informe, Informe> {
-    let (mut checks, mut anotaciones): (Vec<PathBuf>, Vec<PathBuf>) = (Vec::new(), Vec::new());
-    let (mut rutas, mut tarea, mut salida): (Option<PathBuf>, Option<String>, Option<PathBuf>) =
-        (None, None, None);
-    let mut origen: Option<u64> = None;
-    let mut it = a.iter();
-    while let Some(x) = it.next() {
-        let mut valor = || it.next().cloned().ok_or_else(|| uso("falta-valor"));
-        match x.as_str() {
-            "--check-runs" => checks.push(valor()?.into()),
-            "--anotaciones" => anotaciones.push(valor()?.into()),
-            "--rutas" if rutas.is_none() => rutas = Some(valor()?.into()),
-            "--tarea" if tarea.is_none() => {
-                let v = valor()?;
-                if !crate::git::es_id(&v) {
-                    return Err(uso("id-invalido"));
-                }
-                tarea = Some(v);
-            }
-            "--origen" if origen.is_none() => origen = Some(id_de_app(&valor()?)?),
-            "--salida" if salida.is_none() => salida = Some(valor()?.into()),
-            _ => return Err(uso("argumento-desconocido")),
-        }
-    }
-    let Some(rutas) = rutas else {
+    let o = opciones(
+        a,
+        &["--rutas", "--tarea", "--origen", "--salida"],
+        &["--check-runs", "--anotaciones"],
+    )?;
+    comprobar(&o, "--tarea", crate::git::es_id, "id-invalido")?;
+    let origen = o.una("--origen").as_deref().map(id_de_app).transpose()?;
+    let (checks, anotaciones, tarea) = (
+        o.rutas("--check-runs"),
+        o.rutas("--anotaciones"),
+        o.una("--tarea"),
+    );
+    let Some(rutas) = o.ruta("--rutas") else {
         return Err(uso("sin-rutas"));
     };
-    let Some(dir_salida) = salida else {
+    let Some(dir_salida) = o.ruta("--salida") else {
         return Err(uso("sin-salida"));
     };
     let cambiados = rutas_nul(&rutas)?;
@@ -661,19 +694,10 @@ pub fn desde_incremental(
 /// ([`desde_incremental`]). Linea: `hallazgos ok desde=<oid>|base
 /// motivo=<directo|merge|no-ancestro|sin-merge-base|conflicto|git>`.
 fn desde(a: &[String]) -> Result<Informe, Informe> {
-    let (mut anterior, mut head, mut base): (Option<String>, Option<String>, Option<String>) =
-        (None, None, None);
-    let mut it = a.iter();
-    while let Some(x) = it.next() {
-        let mut valor = || it.next().cloned().ok_or_else(|| uso("falta-valor"));
-        match x.as_str() {
-            "--anterior" if anterior.is_none() => anterior = Some(valor()?),
-            "--head" if head.is_none() => head = Some(valor()?),
-            "--base" if base.is_none() => base = Some(valor()?),
-            _ => return Err(uso("argumento-desconocido")),
-        }
-    }
-    let (Some(anterior), Some(head), Some(base)) = (anterior, head, base) else {
+    let o = opciones(a, &["--anterior", "--head", "--base"], &[])?;
+    let (Some(anterior), Some(head), Some(base)) =
+        (o.una("--anterior"), o.una("--head"), o.una("--base"))
+    else {
         return Err(uso("faltan-argumentos"));
     };
     if !es_sha(&anterior) || !es_sha(&head) {
@@ -714,19 +738,10 @@ struct Detalle {
 /// ([`super::ronda::repartir`]). Linea: `hallazgos ok revisoras=N
 /// abiertos=M sin-revisora=K`.
 fn abiertos(a: &[String]) -> Result<Informe, Informe> {
-    let (mut r, mut detalle, mut salida): (Option<PathBuf>, Option<PathBuf>, Option<PathBuf>) =
-        (None, None, None);
-    let mut it = a.iter();
-    while let Some(x) = it.next() {
-        let mut valor = || it.next().cloned().ok_or_else(|| uso("falta-valor"));
-        match x.as_str() {
-            "--ronda" if r.is_none() => r = Some(valor()?.into()),
-            "--detalle" if detalle.is_none() => detalle = Some(valor()?.into()),
-            "--salida" if salida.is_none() => salida = Some(valor()?.into()),
-            _ => return Err(uso("argumento-desconocido")),
-        }
-    }
-    let (Some(r), Some(detalle), Some(dir)) = (r, detalle, salida) else {
+    let o = opciones(a, &["--ronda", "--detalle", "--salida"], &[])?;
+    let (Some(r), Some(detalle), Some(dir)) =
+        (o.ruta("--ronda"), o.ruta("--detalle"), o.ruta("--salida"))
+    else {
         return Err(uso("faltan-argumentos"));
     };
     let d = leer_decision(&r)?;
@@ -757,49 +772,37 @@ pub const TOPE_SKILL: u64 = 4 * 1024 * 1024;
 
 /// `prompt ...`: el prompt de una revisora ([`super::prompt::componer`]).
 fn prompt(a: &[String]) -> Result<Informe, Informe> {
-    let mut rutas: [Option<PathBuf>; 6] = Default::default();
-    let (mut revisora, mut bytes, mut tope): (Option<String>, Option<u64>, Option<u64>) =
-        (None, None, None);
-    let numero = |v: String| v.parse::<u64>().map_err(|_| uso("numero-invalido"));
-    let mut it = a.iter();
-    while let Some(x) = it.next() {
-        let mut valor = || it.next().cloned().ok_or_else(|| uso("falta-valor"));
-        let k = match x.as_str() {
-            "--contexto" => 0,
-            "--skill" => 1,
-            "--seccion" => 2,
-            "--abiertos" => 3,
-            "--diff" => 4,
-            "--salida" => 5,
-            "--revisora" if revisora.is_none() => {
-                let v = valor()?;
-                if !es_revisora(&v) {
-                    return Err(uso("revisora-invalida"));
-                }
-                revisora = Some(v);
-                continue;
-            }
-            "--bytes" if bytes.is_none() => {
-                bytes = Some(numero(valor()?)?);
-                continue;
-            }
-            "--tope" if tope.is_none() => {
-                tope = Some(numero(valor()?)?);
-                continue;
-            }
-            _ => return Err(uso("argumento-desconocido")),
-        };
-        if rutas.get(k).is_some_and(Option::is_some) {
-            return Err(uso("argumento-desconocido"));
-        }
-        if let Some(r) = rutas.get_mut(k) {
-            *r = Some(valor()?.into());
-        }
-    }
-    let [contexto, skill, seccion, abiertos, diff, salida] = rutas;
-    let (Some(revisora), Some(contexto), Some(diff), Some(bytes), Some(tope), Some(salida)) =
-        (revisora, contexto, diff, bytes, tope, salida)
-    else {
+    let o = opciones(
+        a,
+        &[
+            "--contexto",
+            "--skill",
+            "--seccion",
+            "--abiertos",
+            "--diff",
+            "--salida",
+            "--revisora",
+            "--bytes",
+            "--tope",
+        ],
+        &[],
+    )?;
+    comprobar(&o, "--revisora", es_revisora, "revisora-invalida")?;
+    let numero = |n: &str| {
+        o.una(n)
+            .map(|v| v.parse::<u64>().map_err(|_| uso("numero-invalido")))
+            .transpose()
+    };
+    let (bytes, tope) = (numero("--bytes")?, numero("--tope")?);
+    let (skill, seccion, abiertos) = (o.ruta("--skill"), o.ruta("--seccion"), o.ruta("--abiertos"));
+    let (Some(revisora), Some(contexto), Some(diff), Some(bytes), Some(tope), Some(salida)) = (
+        o.una("--revisora"),
+        o.ruta("--contexto"),
+        o.ruta("--diff"),
+        bytes,
+        tope,
+        o.ruta("--salida"),
+    ) else {
         return Err(uso("faltan-argumentos"));
     };
     let opcional =
diff --git a/crates/tc2/src/hallazgos/ronda.rs b/crates/tc2/src/hallazgos/ronda.rs
index fadf48c..98c2f85 100644
--- a/crates/tc2/src/hallazgos/ronda.rs
+++ b/crates/tc2/src/hallazgos/ronda.rs
@@ -9,7 +9,7 @@ use std::collections::BTreeSet;
 
 use serde::{Deserialize, Serialize};
 
-use super::{AceptadoDe, Dictamen, VERSION, dictamen_publicado, plano, ronda_de_titulo};
+use super::{AceptadoDe, Dictamen, VERSION, celdas, dictamen_publicado, fila, ronda_de_titulo};
 
 /// Como se revisa esta ronda.
 #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
@@ -314,7 +314,7 @@ pub fn repartir(abiertos: &[AceptadoDe], detalle: &[RevisoraDetalle]) -> Vec<Vec
 }
 
 /// Los abiertos de una revisora para su prompt, en Markdown, con todo el
-/// texto del modelo por [`plano`] (dato, no instrucciones).
+/// texto del modelo por [`super::plano`] (dato, no instrucciones).
 pub fn abiertos_markdown(abiertos: &[AceptadoDe]) -> String {
     if abiertos.is_empty() {
         return "No tienes hallazgos abiertos que cerrar.\n".to_owned();
@@ -323,17 +323,16 @@ pub fn abiertos_markdown(abiertos: &[AceptadoDe]) -> String {
         "| id | ronda | severidad | clase | fichero | linea | descripcion |\n|---|---|---|---|---|---|---|\n",
     );
     for a in abiertos {
-        let h = &a.hallazgo;
-        s.push_str(&format!(
-            "| {} | {} | {} | {} | {} | {} | {} |\n",
-            plano(&h.id, 64),
-            a.ronda,
-            h.severidad.texto(),
-            h.clase.texto(),
-            plano(&h.fichero, 200),
-            h.linea,
-            plano(&h.descripcion, 400)
-        ));
+        let [id, severidad, clase, fichero, linea, descripcion] = celdas(&a.hallazgo);
+        s.push_str(&fila(&[
+            id,
+            a.ronda.to_string(),
+            severidad,
+            clase,
+            fichero,
+            linea,
+            descripcion,
+        ]));
     }
     s
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
tareas/{01-en-diseno => 02-en-curso}/TASK-140/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-140/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-140/tarea.md                                    | 48 ++++++++++++++++++++++++++++++++++++++++--------
 3 files changed, 40 insertions(+), 8 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/hallazgos.rs"
````

````
crates/tc2/tests/hallazgos.rs | 110 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 110 insertions(+)
````
## Candidatos 4D

Pistas generadas desde el diff, no hallazgos: comprueba cada una (Regla 4D de `revision.md`).

### Posible duplicacion con lo que ya existia

Definiciones nuevas con nombre igual o parecido a otra de la rama base. Mira si hacen lo mismo con pequenas diferencias (duplicacion real = IMPORTANTE).

- `rechazo` (crates/tc2/src/hallazgos/mod.rs:634) ~ `Rechazo` (crates/tc2/src/hallazgos/mod.rs:341), `rechazo` (crates/tc2/src/plan/aprobar.rs:334), `rechazo` (crates/tc2/src/plan/aprobar.rs:496) y 6 mas
- `cortar` (crates/tc2/src/hallazgos/mod.rs:647) ~ `contar` (crates/tc2/src/doctor/capacidades.rs:388), `contar` (crates/tc2/src/estado/pendientes.rs:71), `cortar` (crates/tc2/src/hallazgos/prompt.rs:51)
- `celdas` (crates/tc2/src/hallazgos/mod.rs:1176) ~ `celda` (crates/tc2/src/estimacion/real.rs:158), `celda` (crates/tc2/src/roadmap/generar.rs:158), `celdas` (crates/tc2/src/seleccion/mod.rs:964)
- `fila` (crates/tc2/src/hallazgos/mod.rs:1188) ~ `fila` (crates/tc2/src/ciclo/liberar.rs:438), `fila` (crates/tc2/src/ciclo/operacion.rs:296), `Fila` (crates/tc2/src/doctor/capacidades.rs:49) y 6 mas
- `Opciones` (crates/tc2/src/hallazgos/orden.rs:118) ~ `Opciones` (crates/tc2/src/actualizar_cli.rs:96), `Opciones` (crates/tc2/src/desinstalar/mod.rs:40), `opciones` (crates/tc2/src/desinstalar/mod.rs:632) y 5 mas
- `una` (crates/tc2/src/hallazgos/orden.rs:122) ~ `una` (crates/tc2/src/config/mod.rs:375), `una` (crates/tc2/src/hallazgos/mod.rs:1901)
- `ruta` (crates/tc2/src/hallazgos/orden.rs:127) ~ `ruta` (crates/tc2/src/estado/frescura.rs:154), `ruta` (crates/tc2/src/identidad.rs:222), `ruta` (crates/tc2/src/progreso/fichero.rs:115) y 2 mas
- `rutas` (crates/tc2/src/hallazgos/orden.rs:132) ~ `rutas` (crates/tc2/src/alta.rs:704), `rutas` (crates/tc2/src/estado/lectura.rs:19), `rutas` (crates/tc2/src/estado/lectura.rs:105) y 5 mas
- `opciones` (crates/tc2/src/hallazgos/orden.rs:145) ~ `Opciones` (crates/tc2/src/actualizar_cli.rs:96), `Opciones` (crates/tc2/src/desinstalar/mod.rs:40), `opciones` (crates/tc2/src/desinstalar/mod.rs:632) y 5 mas
- `comprobar` (crates/tc2/src/hallazgos/orden.rs:166) ~ `comprobar` (crates/tc2/src/config/validar.rs:227), `comprobar` (crates/tc2/src/doctor/mod.rs:379), `comprobar` (crates/tc2/src/doctor/plataforma.rs:243) y 1 mas
- `args` (crates/tc2/src/hallazgos/orden.rs:178) ~ `args` (crates/tc2/src/alta.rs:864), `Args` (crates/tc2/src/hallazgos/orden.rs:91), `args` (crates/tc2/src/importar/mod.rs:436) y 2 mas

### Posible duplicacion dentro de la tarea

Definiciones nuevas parecidas entre si: la misma tarea puede estar escribiendo dos veces lo mismo.

- `Opciones` (crates/tc2/src/hallazgos/orden.rs:118) ~ `opciones` (crates/tc2/src/hallazgos/orden.rs:145)

