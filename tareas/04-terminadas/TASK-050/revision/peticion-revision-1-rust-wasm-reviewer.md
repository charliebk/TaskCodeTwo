# Peticion de revision — TASK-050 (ronda 1)

- Tarea: TASK-050 — F6-4 Busqueda de errores
- Rama revisada: feature/task-050-f6-4-busqueda-de-errores
- Rama base: develop
- Commit revisado (HEAD): 90640c05dc759042baa565655060c3224b753c6f
- Fecha: 2026-11-20
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-050 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-050/tarea.md
- tareas/03-en-revision/TASK-050/planificacion/
- crates/tc2/src/hallazgos/mod.rs
- crates/tc2/src/hallazgos/orden.rs
- crates/tc2/src/i18n/catalogo/cli.rs
- crates/tc2/src/i18n/catalogo/hallazgos.rs
- crates/tc2/src/i18n/catalogo/mod.rs
- crates/tc2/src/lib.rs
- crates/tc2/tests/hallazgos.rs
- xtask/src/workflows.rs

## Commits a revisar (git log develop..HEAD)

````
90640c0 feat(TASK-050): revision-ia valida los hallazgos antes de publicar
96d665e feat(TASK-050): tc2 hallazgos valida la evidencia de cada hallazgo
936ffd3 chore(TASK-050): tarea en curso
````

## Diff de tu dominio (8 fichero(s) de 15; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/hallazgos/mod.rs b/crates/tc2/src/hallazgos/mod.rs
new file mode 100644
index 0000000..a4d31e2
--- /dev/null
+++ b/crates/tc2/src/hallazgos/mod.rs
@@ -0,0 +1,1075 @@
+//! Busqueda de errores con evidencia (F6-4; RR-03; ADR F1-8 decision 3 y
+//! adenda F6-4).
+//!
+//! [`validar`] es una funcion pura: recibe la respuesta de una revisora (el
+//! texto que devolvio el modelo), el diff que vio esa revisora ([`Diff`]) y,
+//! si lo hay, el informe de verificacion de F5-4, y devuelve los hallazgos
+//! aceptados y los rechazados con su motivo ([`Validados`]). No lee ficheros,
+//! no llama a la red ni a un modelo: el modelo no decide, lo que devuelve es
+//! un dato que se valida.
+//!
+//! Contrato (`crates/tc2/esquemas/hallazgos.schema.json`, version 1): la
+//! respuesta trae exactamente un bloque `json` cercado con acentos graves con
+//! una lista de hallazgos (`$defs/respuesta` del esquema); cada hallazgo
+//! apunta a una linea del lado nuevo del diff y trae evidencia de su tipo
+//! (`reproduccion`, `mutante` o `borde`), tambien los MENOR. La evidencia se
+//! declara y se ancla al diff, pero no se ejecuta: eso es de F6-6 (adenda
+//! F6-4 de F1-8). [`instrucciones`] es el texto que va al prompt, hecho con
+//! las mismas constantes que el validador para que no se separen.
+
+pub mod orden;
+
+use std::collections::{BTreeMap, BTreeSet};
+
+use serde::{Deserialize, Serialize};
+use serde_json::Value;
+
+use crate::verificacion::informe::{Informe as InformeVerificacion, Resultado as ResultadoPaso};
+
+/// Version del contrato de hallazgos.
+pub const VERSION: u32 = 1;
+
+/// Hallazgos que se validan como mucho por respuesta; los que sobran se
+/// rechazan juntos con [`motivo::FUERA_DE_TOPE`].
+pub const TOPE_HALLAZGOS: usize = 100;
+
+/// Ids estables de los motivos de rechazo.
+pub mod motivo {
+    /// La respuesta no trae ningun bloque `json`.
+    pub const SIN_BLOQUE: &str = "sin-bloque";
+    /// Trae mas de un bloque `json`.
+    pub const BLOQUE_REPETIDO: &str = "bloque-repetido";
+    /// El bloque no se cierra, no es JSON o no es una lista.
+    pub const JSON_INVALIDO: &str = "json-invalido";
+    /// La lista pasa de [`super::TOPE_HALLAZGOS`].
+    pub const FUERA_DE_TOPE: &str = "fuera-de-tope";
+    /// El hallazgo no tiene la forma del contrato.
+    pub const HALLAZGO_INVALIDO: &str = "hallazgo-invalido";
+    /// Otro hallazgo anterior usa el mismo id.
+    pub const ID_REPETIDO: &str = "id-repetido";
+    /// Sin evidencia.
+    pub const SIN_EVIDENCIA: &str = "sin-evidencia";
+    /// La evidencia no tiene los campos de su tipo.
+    pub const EVIDENCIA_INCOMPLETA: &str = "evidencia-incompleta";
+    /// El fichero o la linea no estan en el diff.
+    pub const LINEA_FUERA_DEL_DIFF: &str = "linea-fuera-del-diff";
+    /// `linea_original` no es el texto de la linea citada.
+    pub const MUTANTE_NO_COINCIDE: &str = "mutante-no-coincide";
+    /// La mutacion no cambia la linea.
+    pub const MUTANTE_SIN_CAMBIO: &str = "mutante-sin-cambio";
+    /// `esperado` y `obtenido` son iguales.
+    pub const BORDE_SIN_DIFERENCIA: &str = "borde-sin-diferencia";
+    /// `paso_rojo` no es un paso en rojo del informe de verificacion.
+    pub const PASO_ROJO_NO_ESTA: &str = "paso-rojo-no-esta";
+}
+
+/// Los motivos de rechazo con su explicacion, en el orden en que se
+/// comprueban. Van tal cual al prompt ([`instrucciones`]) y al esquema.
+pub const MOTIVOS: [(&str, &str); 13] = [
+    (
+        motivo::SIN_BLOQUE,
+        "la respuesta no trae ningun bloque json",
+    ),
+    (
+        motivo::BLOQUE_REPETIDO,
+        "la respuesta trae mas de un bloque json",
+    ),
+    (
+        motivo::JSON_INVALIDO,
+        "el bloque no se cierra, no es JSON o no es una lista",
+    ),
+    (
+        motivo::FUERA_DE_TOPE,
+        "la lista pasa del tope de hallazgos: se rechazan los que sobran",
+    ),
+    (
+        motivo::HALLAZGO_INVALIDO,
+        "no es un objeto, le falta una clave, trae una desconocida o un valor fuera de forma (id, severidad, linea, texto vacio)",
+    ),
+    (motivo::ID_REPETIDO, "un hallazgo anterior ya usa ese id"),
+    (motivo::SIN_EVIDENCIA, "no trae evidencia"),
+    (
+        motivo::EVIDENCIA_INCOMPLETA,
+        "tipo desconocido, faltan campos de su tipo, alguno esta vacio o trae campos de otro tipo",
+    ),
+    (
+        motivo::LINEA_FUERA_DEL_DIFF,
+        "el fichero o la linea no estan en el lado nuevo del diff que vio la revisora",
+    ),
+    (
+        motivo::MUTANTE_NO_COINCIDE,
+        "linea_original no es el texto de esa linea en el diff",
+    ),
+    (
+        motivo::MUTANTE_SIN_CAMBIO,
+        "la mutacion es igual a la linea original",
+    ),
+    (
+        motivo::BORDE_SIN_DIFERENCIA,
+        "esperado y obtenido son iguales",
+    ),
+    (
+        motivo::PASO_ROJO_NO_ESTA,
+        "paso_rojo no es un paso en rojo del informe de verificacion, o no hay informe",
+    ),
+];
+
+/// Severidad de un hallazgo (las de la revision por pares de TaskCode).
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(rename_all = "UPPERCASE")]
+pub enum Severidad {
+    Critico,
+    Importante,
+    Menor,
+}
+
+impl Severidad {
+    /// Como se escribe en el contrato.
+    pub fn texto(self) -> &'static str {
+        match self {
+            Severidad::Critico => "CRITICO",
+            Severidad::Importante => "IMPORTANTE",
+            Severidad::Menor => "MENOR",
+        }
+    }
+}
+
+/// Tipo de evidencia (RR-03: reproduccion, mutantes y bordes).
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(rename_all = "lowercase")]
+pub enum TipoEvidencia {
+    Reproduccion,
+    Mutante,
+    Borde,
+}
+
+impl TipoEvidencia {
+    /// Como se escribe en el contrato.
+    pub fn texto(self) -> &'static str {
+        match self {
+            TipoEvidencia::Reproduccion => "reproduccion",
+            TipoEvidencia::Mutante => "mutante",
+            TipoEvidencia::Borde => "borde",
+        }
+    }
+
+    /// Los campos que admite este tipo (ademas de `tipo`).
+    fn campos(self) -> &'static [&'static str] {
+        match self {
+            TipoEvidencia::Reproduccion => &["pasos", "fallo_esperado", "paso_rojo"],
+            TipoEvidencia::Mutante => &["linea_original", "mutacion", "test"],
+            TipoEvidencia::Borde => &["entrada", "esperado", "obtenido"],
+        }
+    }
+
+    /// Lo que pide, para el detalle del rechazo.
+    fn pide(self) -> &'static str {
+        match self {
+            TipoEvidencia::Reproduccion => "pasos y fallo_esperado, o paso_rojo",
+            TipoEvidencia::Mutante => "linea_original, mutacion y test",
+            TipoEvidencia::Borde => "entrada, esperado y obtenido",
+        }
+    }
+}
+
+/// Evidencia de un hallazgo: `tipo` y los campos de ese tipo, ninguno de
+/// otro (lo comprueba [`validar`]; aqui solo se rechaza lo desconocido).
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct Evidencia {
+    pub tipo: TipoEvidencia,
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub pasos: Option<Vec<String>>,
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub fallo_esperado: Option<String>,
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub paso_rojo: Option<String>,
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub linea_original: Option<String>,
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub mutacion: Option<String>,
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub test: Option<String>,
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub entrada: Option<String>,
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub esperado: Option<String>,
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub obtenido: Option<String>,
+}
+
+/// Las claves de un hallazgo sin la evidencia.
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct Cabecera {
+    id: String,
+    severidad: Severidad,
+    fichero: String,
+    linea: u32,
+    descripcion: String,
+}
+
+/// Un hallazgo que cumple el contrato.
+#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
+pub struct Aceptado {
+    pub id: String,
+    pub severidad: Severidad,
+    pub fichero: String,
+    pub linea: u32,
+    pub descripcion: String,
+    pub evidencia: Evidencia,
+}
+
+/// Un rechazo: de un hallazgo (`indice` en la lista) o de toda la respuesta
+/// (`indice` nulo).
+#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
+pub struct Rechazo {
+    pub indice: Option<usize>,
+    /// El id del hallazgo, si tiene uno con forma valida.
+    pub id: Option<String>,
+    /// Uno de [`MOTIVOS`].
+    pub motivo: &'static str,
+    /// Que fallo, para la persona.
+    pub detalle: String,
+}
+
+/// Resultado de validar la respuesta de una revisora.
+#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
+pub struct Validados {
+    pub version: u32,
+    pub revisora: String,
+    pub aceptados: Vec<Aceptado>,
+    pub rechazados: Vec<Rechazo>,
+}
+
+impl Rechazo {
+    /// A quien se refiere: el id del hallazgo, `#<indice>` si no tiene uno
+    /// valido, o `respuesta`.
+    pub fn quien(&self) -> String {
+        match (&self.id, self.indice) {
+            (Some(id), _) => id.clone(),
+            (None, Some(i)) => format!("#{i}"),
+            (None, None) => "respuesta".to_owned(),
+        }
+    }
+}
+
+impl Validados {
+    /// `quien:motivo` por rechazo, para la linea de contrato.
+    pub fn motivos(&self) -> Vec<String> {
+        self.rechazados
+            .iter()
+            .map(|r| format!("{}:{}", r.quien(), r.motivo))
+            .collect()
+    }
+}
+
+/// Las lineas del lado nuevo de un diff unificado de git: por fichero, el
+/// numero de linea y su texto (las de contexto y las anadidas).
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct Diff {
+    lineas: BTreeMap<String, BTreeMap<u32, String>>,
+}
+
+impl Diff {
+    /// Lee un diff de `git diff` (con o sin prefijos `a/` y `b/`). Lo que no
+    /// entiende lo salta; un diff cortado deja fuera lo que no llego.
+    pub fn leer(texto: &str) -> Self {
+        let mut d = Diff::default();
+        let mut fichero: Option<String> = None;
+        let mut nuevo: u32 = 0;
+        let mut quedan_nuevo: u32 = 0;
+        let mut quedan_viejo: u32 = 0;
+        for l in texto.split('\n') {
+            let l = l.strip_suffix('\r').unwrap_or(l);
+            if quedan_nuevo > 0 || quedan_viejo > 0 {
+                let mut registrar = |resto: &str| {
+                    if let Some(f) = &fichero {
+                        d.lineas
+                            .entry(f.clone())
+                            .or_default()
+                            .insert(nuevo, resto.to_owned());
+                    }
+                };
+                match l.as_bytes().first() {
+                    Some(b'+') if quedan_nuevo > 0 => {
+                        registrar(&l[1..]);
+                        nuevo = nuevo.saturating_add(1);
+                        quedan_nuevo -= 1;
+                        continue;
+                    }
+                    Some(b'-') if quedan_viejo > 0 => {
+                        quedan_viejo -= 1;
+                        continue;
+                    }
+                    Some(b' ') if quedan_nuevo > 0 => {
+                        registrar(&l[1..]);
+                        nuevo = nuevo.saturating_add(1);
+                        quedan_nuevo -= 1;
+                        quedan_viejo = quedan_viejo.saturating_sub(1);
+                        continue;
+                    }
+                    Some(b'\\') => continue,
+                    // Fin inesperado del trozo (diff mal formado): se sigue
+                    // buscando cabeceras.
+                    _ => {
+                        quedan_nuevo = 0;
+                        quedan_viejo = 0;
+                    }
+                }
+            }
+            if l.starts_with("diff --git ") {
+                fichero = None;
+            } else if let Some(r) = l.strip_prefix("+++ ") {
+                fichero = ruta_nueva(r);
+            } else if let Some(r) = l.strip_prefix("@@ ")
+                && let Some((viejo, (inicio, largo))) = cabecera_de_trozo(r)
+            {
+                nuevo = inicio;
+                quedan_nuevo = largo;
+                quedan_viejo = viejo;
+            }
+        }
+        d
+    }
+
+    /// El texto de la linea `n` del lado nuevo de `fichero`, sin el `+` ni
+    /// el espacio inicial.
+    pub fn linea(&self, fichero: &str, n: u32) -> Option<&str> {
+        self.lineas
+            .get(fichero)
+            .and_then(|m| m.get(&n))
+            .map(String::as_str)
+    }
+}
+
+/// La ruta de una linea `+++ `: sin el tabulador final que git pone a las
+/// rutas con espacios, sin comillas y sin `b/`; `None` si es `/dev/null`.
+fn ruta_nueva(r: &str) -> Option<String> {
+    let r = r.trim_end_matches('\t');
+    if r == "/dev/null" {
+        return None;
+    }
+    let r = r
+        .strip_prefix('"')
+        .and_then(|x| x.strip_suffix('"'))
+        .unwrap_or(r);
+    Some(r.strip_prefix("b/").unwrap_or(r).to_owned())
+}
+
+/// `-a,b +c,d @@ ...` da `(b, (c, d))`; sin coma, el largo es 1.
+fn cabecera_de_trozo(r: &str) -> Option<(u32, (u32, u32))> {
+    let mut partes = r.split_whitespace();
+    let viejo = rango(partes.next()?.strip_prefix('-')?)?;
+    let nuevo = rango(partes.next()?.strip_prefix('+')?)?;
+    Some((viejo.1, nuevo))
+}
+
+fn rango(t: &str) -> Option<(u32, u32)> {
+    match t.split_once(',') {
+        Some((a, b)) => Some((a.parse().ok()?, b.parse().ok()?)),
+        None => Some((t.parse().ok()?, 1)),
+    }
+}
+
+/// El contenido del unico bloque `json` de la respuesta, o el motivo y el
+/// detalle del rechazo de toda la respuesta.
+fn bloque_json(respuesta: &str) -> Result<String, (&'static str, String)> {
+    let mut bloques: Vec<String> = Vec::new();
+    let mut abierto: Option<(usize, String)> = None;
+    for l in respuesta.split('\n') {
+        let l = l.strip_suffix('\r').unwrap_or(l);
+        let t = l.trim();
+        let graves = t.chars().take_while(|c| *c == '`').count();
+        match &mut abierto {
+            Some((n, contenido)) => {
+                if graves >= *n && graves == t.len() {
+                    bloques.push(std::mem::take(contenido));
+                    abierto = None;
+                } else {
+                    contenido.push_str(l);
+                    contenido.push('\n');
+                }
+            }
+            None => {
+                if graves >= 3 && t[graves..].trim().eq_ignore_ascii_case("json") {
+                    abierto = Some((graves, String::new()));
+                }
+            }
+        }
+    }
+    if abierto.is_some() {
+        return Err((
+            motivo::JSON_INVALIDO,
+            "el bloque json no se cierra".to_owned(),
+        ));
+    }
+    match bloques.len() {
+        0 => Err((
+            motivo::SIN_BLOQUE,
+            "la respuesta no trae un bloque json con la lista de hallazgos".to_owned(),
+        )),
+        1 => Ok(bloques.remove(0)),
+        n => Err((
+            motivo::BLOQUE_REPETIDO,
+            format!("la respuesta trae {n} bloques json; tiene que traer uno"),
+        )),
+    }
+}
+
+/// Un id: letra o digito y luego letras, digitos, `.`, `_` o `-`; hasta 64.
+fn es_id(t: &str) -> bool {
+    let mut c = t.chars();
+    t.len() <= 64
+        && c.next().is_some_and(|x| x.is_ascii_alphanumeric())
+        && c.all(|x| x.is_ascii_alphanumeric() || matches!(x, '.' | '_' | '-'))
+}
+
+fn no_vacio(t: &str) -> bool {
+    !t.trim().is_empty()
+}
+
+struct Fallo {
+    id: Option<String>,
+    motivo: &'static str,
+    detalle: String,
+}
+
+/// Que la evidencia traiga los campos de su tipo, llenos, y ninguno ajeno.
+fn evidencia_completa(e: &Evidencia) -> Result<(), String> {
+    let texto = |o: &Option<String>| o.as_deref().map(no_vacio);
+    let campos: [(&str, Option<bool>); 9] = [
+        (
+            "pasos",
+            e.pasos
+                .as_ref()
+                .map(|p| !p.is_empty() && p.iter().all(|s| no_vacio(s))),
+        ),
+        ("fallo_esperado", texto(&e.fallo_esperado)),
+        ("paso_rojo", texto(&e.paso_rojo)),
+        ("linea_original", texto(&e.linea_original)),
+        ("mutacion", texto(&e.mutacion)),
+        ("test", texto(&e.test)),
+        ("entrada", texto(&e.entrada)),
+        ("esperado", texto(&e.esperado)),
+        ("obtenido", texto(&e.obtenido)),
+    ];
+    let propios = e.tipo.campos();
+    for (nombre, estado) in campos {
+        match estado {
+            Some(_) if !propios.contains(&nombre) => {
+                return Err(format!(
+                    "{nombre} no es de una evidencia {}",
+                    e.tipo.texto()
+                ));
+            }
+            Some(false) => return Err(format!("{nombre} esta vacio")),
+            _ => {}
+        }
+    }
+    let tiene = |n: &str| campos.iter().any(|(m, x)| *m == n && *x == Some(true));
+    let completa = match e.tipo {
+        TipoEvidencia::Reproduccion => {
+            (tiene("pasos") && tiene("fallo_esperado")) || tiene("paso_rojo")
+        }
+        _ => propios.iter().all(|n| tiene(n)),
+    };
+    if completa {
+        Ok(())
+    } else {
+        Err(format!(
+            "una evidencia {} pide {}",
+            e.tipo.texto(),
+            e.tipo.pide()
+        ))
+    }
+}
+
+fn validar_uno(
+    h: &Value,
+    diff: &Diff,
+    informe: Option<&InformeVerificacion>,
+    vistos: &mut BTreeSet<String>,
+) -> Result<Aceptado, Fallo> {
+    let Some(obj) = h.as_object() else {
+        return Err(Fallo {
+            id: None,
+            motivo: motivo::HALLAZGO_INVALIDO,
+            detalle: "el hallazgo no es un objeto".to_owned(),
+        });
+    };
+    let id = obj
+        .get("id")
+        .and_then(Value::as_str)
+        .filter(|s| es_id(s))
+        .map(str::to_owned);
+    let fallo = |motivo: &'static str, detalle: String| Fallo {
+        id: id.clone(),
+        motivo,
+        detalle,
+    };
+    let evidencia = match obj.get("evidencia") {
+        None | Some(Value::Null) => {
+            return Err(fallo(
+                motivo::SIN_EVIDENCIA,
+                "el hallazgo no trae evidencia".to_owned(),
+            ));
+        }
+        Some(e) => e,
+    };
+    let mut resto = obj.clone();
+    resto.remove("evidencia");
+    let c: Cabecera = serde_json::from_value(Value::Object(resto))
+        .map_err(|e| fallo(motivo::HALLAZGO_INVALIDO, e.to_string()))?;
+    if !es_id(&c.id) {
+        return Err(fallo(
+            motivo::HALLAZGO_INVALIDO,
+            "id no valido: letras, digitos, punto, guion y guion bajo, hasta 64".to_owned(),
+        ));
+    }
+    if c.linea == 0 || !no_vacio(&c.fichero) || !no_vacio(&c.descripcion) {
+        return Err(fallo(
+            motivo::HALLAZGO_INVALIDO,
+            "linea desde 1, y fichero y descripcion no vacios".to_owned(),
+        ));
+    }
+    if !vistos.insert(c.id.clone()) {
+        return Err(fallo(
+            motivo::ID_REPETIDO,
+            format!("el id {} ya lo usa un hallazgo anterior", c.id),
+        ));
+    }
+    let e: Evidencia = serde_json::from_value(evidencia.clone())
+        .map_err(|x| fallo(motivo::EVIDENCIA_INCOMPLETA, x.to_string()))?;
+    evidencia_completa(&e).map_err(|x| fallo(motivo::EVIDENCIA_INCOMPLETA, x))?;
+    let Some(texto) = diff.linea(&c.fichero, c.linea) else {
+        return Err(fallo(
+            motivo::LINEA_FUERA_DEL_DIFF,
+            format!(
+                "{}:{} no esta en el lado nuevo del diff que vio la revisora",
+                c.fichero, c.linea
+            ),
+        ));
+    };
+    if let Some(original) = &e.linea_original {
+        if original.trim() != texto.trim() {
+            return Err(fallo(
+                motivo::MUTANTE_NO_COINCIDE,
+                format!(
+                    "linea_original no es el texto de {}:{} en el diff",
+                    c.fichero, c.linea
+                ),
+            ));
+        }
+        if e.mutacion.as_deref().map(str::trim) == Some(original.trim()) {
+            return Err(fallo(
+                motivo::MUTANTE_SIN_CAMBIO,
+                "la mutacion es igual a la linea original".to_owned(),
+            ));
+        }
+    }
+    if let (Some(esperado), Some(obtenido)) = (&e.esperado, &e.obtenido)
+        && esperado.trim() == obtenido.trim()
+    {
+        return Err(fallo(
+            motivo::BORDE_SIN_DIFERENCIA,
+            "esperado y obtenido son iguales: no hay borde".to_owned(),
+        ));
+    }
+    if let Some(paso) = &e.paso_rojo {
+        let en_rojo = informe.is_some_and(|i| {
+            i.pasos
+                .iter()
+                .any(|p| p.nombre == paso.trim() && p.resultado == ResultadoPaso::Rojo)
+        });
+        if !en_rojo {
+            return Err(fallo(
+                motivo::PASO_ROJO_NO_ESTA,
+                format!(
+                    "el paso {} no figura en rojo en el informe de verificacion",
+                    paso.trim()
+                ),
+            ));
+        }
+    }
+    Ok(Aceptado {
+        id: c.id,
+        severidad: c.severidad,
+        fichero: c.fichero,
+        linea: c.linea,
+        descripcion: c.descripcion,
+        evidencia: e,
+    })
+}
+
+/// Valida la respuesta de la revisora `revisora` contra el diff que vio y,
+/// si lo hay, el informe de verificacion (para `paso_rojo`).
+pub fn validar(
+    revisora: &str,
+    respuesta: &str,
+    diff: &Diff,
+    informe: Option<&InformeVerificacion>,
+) -> Validados {
+    let mut v = Validados {
+        version: VERSION,
+        revisora: revisora.to_owned(),
+        aceptados: Vec::new(),
+        rechazados: Vec::new(),
+    };
+    let de_toda = |motivo: &'static str, detalle: String| Rechazo {
+        indice: None,
+        id: None,
+        motivo,
+        detalle,
+    };
+    let lista = match bloque_json(respuesta).map(|b| serde_json::from_str::<Value>(&b)) {
+        Err((m, d)) => {
+            v.rechazados.push(de_toda(m, d));
+            return v;
+        }
+        Ok(Ok(Value::Array(l))) => l,
+        Ok(Ok(_)) => {
+            v.rechazados.push(de_toda(
+                motivo::JSON_INVALIDO,
+                "el bloque json no es una lista".to_owned(),
+            ));
+            return v;
+        }
+        Ok(Err(e)) => {
+            v.rechazados.push(de_toda(
+                motivo::JSON_INVALIDO,
+                format!("el bloque no es JSON: {e}"),
+            ));
+            return v;
+        }
+    };
+    let mut vistos = BTreeSet::new();
+    for (i, h) in lista.iter().enumerate() {
+        if i >= TOPE_HALLAZGOS {
+            v.rechazados.push(Rechazo {
+                indice: Some(i),
+                id: None,
+                motivo: motivo::FUERA_DE_TOPE,
+                detalle: format!(
+                    "{} hallazgos pasan del tope de {TOPE_HALLAZGOS}",
+                    lista.len() - TOPE_HALLAZGOS
+                ),
+            });
+            break;
+        }
+        match validar_uno(h, diff, informe, &mut vistos) {
+            Ok(a) => v.aceptados.push(a),
+            Err(f) => v.rechazados.push(Rechazo {
+                indice: Some(i),
+                id: f.id,
+                motivo: f.motivo,
+                detalle: f.detalle,
+            }),
+        }
+    }
+    v
+}
+
+/// Ejemplo de respuesta valida de las instrucciones (lo validan los tests
+/// contra `tests/fixtures/hallazgos/diff.txt`).
+const EJEMPLO: &str = r#"[
+  {
+    "id": "H1",
+    "severidad": "CRITICO",
+    "fichero": "src/calculo.rs",
+    "linea": 5,
+    "descripcion": "La condicion esta al reves: con dos o mas valores devuelve None.",
+    "evidencia": {
+      "tipo": "borde",
+      "entrada": "media(&[2.0, 4.0])",
+      "esperado": "Some(3.0)",
+      "obtenido": "None"
+    }
+  },
+  {
+    "id": "H2",
+    "severidad": "MENOR",
+    "fichero": "src/calculo.rs",
+    "linea": 12,
+    "descripcion": "Ningun test fija el tope de 100.",
+    "evidencia": {
+      "tipo": "mutante",
+      "linea_original": "x.min(100)",
+      "mutacion": "x.min(101)",
+      "test": "tope_corta_en_100"
+    }
+  }
+]"#;
+
+/// El texto de «Lo que devuelves» del prompt de cada revisora, hecho con las
+/// mismas constantes que [`validar`].
+pub fn instrucciones() -> String {
+    let mut s = String::new();
+    s.push_str(
+        "Devuelve un informe en Markdown en espanol y, al final, exactamente un bloque ```json con la lista de tus hallazgos (`[]` si no hay ninguno). \
+Solo cuenta lo que hay en ese bloque: `tc2 hallazgos` lo valida con reglas fijas y rechaza todo hallazgo que no las cumpla, tambien los MENOR. \
+El check `revision-ia` lista los aceptados y los rechazados con su motivo.\n\n",
+    );
+    s.push_str("Cada hallazgo es un objeto con estas claves y ninguna mas:\n\n");
+    s.push_str("- `id`: corto y unico en la lista (letras, digitos, `.`, `_` o `-`; hasta 64).\n");
+    let severidades: Vec<String> = [Severidad::Critico, Severidad::Importante, Severidad::Menor]
+        .iter()
+        .map(|x| format!("`{}`", x.texto()))
+        .collect();
+    s.push_str(&format!("- `severidad`: {}.\n", severidades.join(", ")));
+    s.push_str(
+        "- `fichero`: la ruta tal como sale en la cabecera `+++ b/<ruta>` del diff, sin `b/`.\n",
+    );
+    s.push_str(
+        "- `linea`: numero de linea del lado nuevo del diff. Se cuenta desde el `+c` de la cabecera `@@ -a,b +c,d @@`, sumando las lineas que empiezan por espacio o por `+` y saltando las que empiezan por `-`. Tiene que ser una linea del diff que ves.\n",
+    );
+    s.push_str("- `descripcion`: que falla y por que, en una o dos frases.\n");
+    s.push_str(
+        "- `evidencia`: obligatoria en todos. Un objeto con `tipo` y los campos de ese tipo, llenos, y ninguno de otro tipo:\n",
+    );
+    s.push_str(
+        "  - `reproduccion`: `pasos` (lista no vacia) y `fallo_esperado`; o `paso_rojo`, el nombre de un paso en rojo del informe de verificacion (F5-4).\n",
+    );
+    s.push_str(
+        "  - `mutante`: `linea_original` (el texto de la linea `linea` copiado tal cual del diff, sin el `+` ni el espacio inicial), `mutacion` (esa linea cambiada, distinta) y `test` (el test que deberia detectar la mutacion y hoy no lo hace).\n",
+    );
+    s.push_str(
+        "  - `borde`: `entrada`, `esperado` (lo que deberia salir) y `obtenido` (lo que sale segun el codigo), distintos.\n\n",
+    );
+    s.push_str(
+        "En esta ronda nadie ejecuta la evidencia: se comprueba que este completa y anclada al diff (la ejecucion es de F6-6). \
+No inventes lineas que no ves: si el diff esta cortado, el hallazgo de lo que no llego se rechaza.\n\n",
+    );
+    s.push_str(&format!(
+        "Como mucho {TOPE_HALLAZGOS} hallazgos. Motivos de rechazo:\n\n"
+    ));
+    for (id, explicacion) in MOTIVOS {
+        s.push_str(&format!("- `{id}`: {explicacion}.\n"));
+    }
+    s.push_str("\nEjemplo del bloque:\n\n```json\n");
+    s.push_str(EJEMPLO);
+    s.push_str("\n```\n");
+    s
+}
+
+/// Texto de la respuesta del modelo como texto plano de Markdown en una
+/// linea: sin saltos ni controles, con la puntuacion que da formato escapada
+/// y cortado a `tope` caracteres. Es dato del modelo (F6-7): no puede abrir
+/// tablas, enlaces, imagenes ni HTML en el check.
+pub fn plano(t: &str, tope: usize) -> String {
+    let mut s = String::new();
+    let mut n = 0;
+    let mut espacio = false;
+    for c in t.chars() {
+        if n >= tope {
+            s.push('…');
+            break;
+        }
+        if c.is_whitespace() || c.is_control() {
+            if !espacio && !s.is_empty() {
+                s.push(' ');
+                n += 1;
+            }
+            espacio = true;
+            continue;
+        }
+        espacio = false;
+        if matches!(
+            c,
+            '\\' | '`'
+                | '*'
+                | '_'
+                | '['
+                | ']'
+                | '('
+                | ')'
+                | '<'
+                | '>'
+                | '|'
+                | '!'
+                | '~'
+                | '#'
+                | '&'
+        ) {
+            s.push('\\');
+        }
+        s.push(c);
+        n += 1;
+    }
+    s.trim_end().to_owned()
+}
+
+/// La seccion del check `revision-ia` para una revisora: las tablas de
+/// aceptados y rechazados, con todo el texto del modelo pasado por
+/// [`plano`].
+pub fn resumen_markdown(v: &Validados) -> String {
+    let mut s = format!(
+        "### Hallazgos de {}: {} aceptados, {} rechazados\n\n",
+        plano(&v.revisora, 120),
+        v.aceptados.len(),
+        v.rechazados.len()
+    );
+    if !v.aceptados.is_empty() {
+        s.push_str("| id | severidad | fichero | linea | evidencia | descripcion |\n|---|---|---|---|---|---|\n");
+        for a in &v.aceptados {
+            s.push_str(&format!(
+                "| {} | {} | {} | {} | {} | {} |\n",
+                plano(&a.id, 64),
+                a.severidad.texto(),
+                plano(&a.fichero, 200),
+                a.linea,
+                a.evidencia.tipo.texto(),
+                plano(&a.descripcion, 400)
+            ));
+        }
+        s.push('\n');
+    }
+    if !v.rechazados.is_empty() {
+        s.push_str("| rechazado | motivo | detalle |\n|---|---|---|\n");
+        for r in &v.rechazados {
+            s.push_str(&format!(
+                "| {} | {} | {} |\n",
+                plano(&r.quien(), 64),
+                r.motivo,
+                plano(&r.detalle, 300)
+            ));
+        }
+        s.push('\n');
+    }
+    s
+}
+
+#[cfg(test)]
+#[allow(clippy::panic)]
+mod tests {
+    use super::*;
+
+    const DIFF: &str = include_str!("../../tests/fixtures/hallazgos/diff.txt");
+    const MIXTA: &str = include_str!("../../tests/fixtures/hallazgos/respuesta-mixta.md");
+    const ROJO: &str = include_str!("../../tests/fixtures/informe-verificacion/xtask-rojo.json");
+    const ESQUEMA: &str = include_str!("../../esquemas/hallazgos.schema.json");
+
+    fn diff() -> Diff {
+        Diff::leer(DIFF)
+    }
+
+    fn una(json: &str) -> String {
+        format!("Informe.\n\n```json\n[{json}]\n```\n")
+    }
+
+    const BORDE: &str = r#"{"id":"B","severidad":"MENOR","fichero":"src/calculo.rs","linea":5,"descripcion":"d","evidencia":{"tipo":"borde","entrada":"x","esperado":"1","obtenido":"2"}}"#;
+
+    #[test]
+    fn el_diff_da_las_lineas_del_lado_nuevo() {
+        let d = diff();
+        assert_eq!(
+            d.linea("src/calculo.rs", 1),
+            Some("//! Calculos de prueba.")
+        );
+        assert_eq!(d.linea("src/calculo.rs", 5), Some("    if v.len() > 1 {"));
+        assert_eq!(d.linea("src/calculo.rs", 12), Some("    x.min(100)"));
+        assert_eq!(d.linea("src/calculo.rs", 13), Some("}"));
+        assert_eq!(d.linea("src/calculo.rs", 14), None);
+        assert_eq!(d.linea("src/calculo.rs", 0), None);
+        // Las lineas borradas no estan en el lado nuevo: tras la 2 viene la 3
+        // anadida, no la vieja.
+        assert_eq!(
+            d.linea("src/calculo.rs", 3),
+            Some("/// Media de los valores; `None` si no hay ninguno.")
+        );
+        // Segundo fichero, un segundo trozo y una linea anadida que empieza
+        // por `++`.
+        assert_eq!(d.linea("docs/nota.md", 10), Some("++ no es una cabecera"));
+        assert_eq!(d.linea("docs/nota.md", 11), Some("fin"));
+        assert_eq!(d.linea("docs/nota.md", 1), Some("uno bis"));
+        assert_eq!(d.linea("docs/nota.md", 5), None);
+        // Un fichero borrado no deja lineas.
+        assert_eq!(d.linea("viejo.rs", 1), None);
+        // Cortado a mitad, lo que no llego no esta.
+        let corte = DIFF.find("pub fn tope").unwrap_or(DIFF.len());
+        let cortado = Diff::leer(&DIFF[..corte]);
+        assert_eq!(cortado.linea("src/calculo.rs", 10), Some(""));
+        assert_eq!(cortado.linea("src/calculo.rs", 12), None);
+    }
+
+    #[test]
+    fn la_respuesta_mixta_da_tres_aceptados_y_dos_rechazados() {
+        let v = validar("r", MIXTA, &diff(), None);
+        let ids: Vec<&str> = v.aceptados.iter().map(|a| a.id.as_str()).collect();
+        assert_eq!(ids, ["H1", "H2", "H3"]);
+        assert_eq!(v.motivos(), ["H4:sin-evidencia", "H5:linea-fuera-del-diff"]);
+    }
+
+    /// Cada regla, con un caso que la rompe.
+    #[test]
+    fn cada_regla_rechaza_con_su_motivo() {
+        let d = diff();
+        let casos: Vec<(String, &str)> = vec![
+            ("Sin bloque, solo prosa.".to_owned(), "respuesta:sin-bloque"),
+            (format!("{}\n{}", una(BORDE), una(BORDE)), "respuesta:bloque-repetido"),
+            ("```json\n[\n".to_owned(), "respuesta:json-invalido"),
+            ("```json\n[{]\n```".to_owned(), "respuesta:json-invalido"),
+            ("```json\n{\"id\":\"B\"}\n```".to_owned(), "respuesta:json-invalido"),
+            (una("3"), "#0:hallazgo-invalido"),
+            (una(&BORDE.replace("\"MENOR\"", "\"GRAVE\"")), "B:hallazgo-invalido"),
+            (una(&BORDE.replace("\"linea\":5", "\"linea\":0")), "B:hallazgo-invalido"),
+            (una(&BORDE.replace("\"linea\":5", "\"linea\":5,\"x\":1")), "B:hallazgo-invalido"),
+            (una(&BORDE.replace("\"descripcion\":\"d\"", "\"descripcion\":\" \"")), "B:hallazgo-invalido"),
+            (una(&BORDE.replace("\"id\":\"B\"", "\"id\":\"no vale\"")), "#0:hallazgo-invalido"),
+            (una(&format!("{BORDE},{BORDE}")), "B:id-repetido"),
+            (una(&BORDE.replace(",\"evidencia\":{\"tipo\":\"borde\",\"entrada\":\"x\",\"esperado\":\"1\",\"obtenido\":\"2\"}", "")), "B:sin-evidencia"),
+            (una(&BORDE.replace("\"tipo\":\"borde\"", "\"tipo\":\"opinion\"")), "B:evidencia-incompleta"),
+            (una(&BORDE.replace(",\"obtenido\":\"2\"", "")), "B:evidencia-incompleta"),
+            (una(&BORDE.replace("\"obtenido\":\"2\"", "\"obtenido\":\"\"")), "B:evidencia-incompleta"),
+            (una(&BORDE.replace("\"obtenido\":\"2\"", "\"obtenido\":\"2\",\"test\":\"t\"")), "B:evidencia-incompleta"),
+            (una(&BORDE.replace("\"obtenido\":\"2\"", "\"obtenido\":\"2\",\"x\":1")), "B:evidencia-incompleta"),
+            (una(&BORDE.replace("\"linea\":5", "\"linea\":99")), "B:linea-fuera-del-diff"),
+            (una(&BORDE.replace("src/calculo.rs", "src/otro.rs")), "B:linea-fuera-del-diff"),
+            (una(&BORDE.replace("\"obtenido\":\"2\"", "\"obtenido\":\" 1 \"")), "B:borde-sin-diferencia"),
+            (
+                una(r#"{"id":"M","severidad":"IMPORTANTE","fichero":"src/calculo.rs","linea":12,"descripcion":"d","evidencia":{"tipo":"mutante","linea_original":"x.min(10)","mutacion":"x.min(11)","test":"t"}}"#),
+                "M:mutante-no-coincide",
+            ),
+            (
+                una(r#"{"id":"M","severidad":"IMPORTANTE","fichero":"src/calculo.rs","linea":12,"descripcion":"d","evidencia":{"tipo":"mutante","linea_original":"x.min(100)","mutacion":" x.min(100)","test":"t"}}"#),
+                "M:mutante-sin-cambio",
+            ),
+            (
+                una(r#"{"id":"R","severidad":"MENOR","fichero":"src/calculo.rs","linea":8,"descripcion":"d","evidencia":{"tipo":"reproduccion","pasos":[]}}"#),
+                "R:evidencia-incompleta",
+            ),
+            (
+                una(r#"{"id":"R","severidad":"MENOR","fichero":"src/calculo.rs","linea":8,"descripcion":"d","evidencia":{"tipo":"reproduccion","pasos":["a"]}}"#),
+                "R:evidencia-incompleta",
+            ),
+            (
+                una(r#"{"id":"R","severidad":"MENOR","fichero":"src/calculo.rs","linea":8,"descripcion":"d","evidencia":{"tipo":"reproduccion","paso_rojo":"clippy"}}"#),
+                "R:paso-rojo-no-esta",
+            ),
+        ];
+        for (respuesta, esperado) in casos {
+            let v = validar("r", &respuesta, &d, None);
+            assert_eq!(v.motivos(), [esperado], "{respuesta}");
+        }
+        // El caso base se acepta.
+        let v = validar("r", &una(BORDE), &d, None);
+        assert_eq!((v.aceptados.len(), v.rechazados.len()), (1, 0));
+        // Mas de TOPE_HALLAZGOS: los que sobran, un rechazo.
+        let muchos: Vec<String> = (0..=TOPE_HALLAZGOS)
+            .map(|i| BORDE.replace("\"id\":\"B\"", &format!("\"id\":\"B{i}\"")))
+            .collect();
+        let v = validar("r", &una(&muchos.join(",")), &d, None);
+        assert_eq!(v.aceptados.len(), TOPE_HALLAZGOS);
+        assert_eq!(v.motivos(), [format!("#{TOPE_HALLAZGOS}:fuera-de-tope")]);
+    }
+
+    #[test]
+    fn paso_rojo_se_contrasta_con_el_informe() {
+        let informe = crate::verificacion::informe::leer(ROJO).unwrap_or_else(|e| panic!("{e}"));
+        let r = |paso: &str| {
+            una(&format!(
+                r#"{{"id":"R","severidad":"MENOR","fichero":"src/calculo.rs","linea":8,"descripcion":"d","evidencia":{{"tipo":"reproduccion","paso_rojo":"{paso}"}}}}"#
+            ))
+        };
+        let v = validar("r", &r("clippy"), &diff(), Some(&informe));
+        assert_eq!((v.aceptados.len(), v.rechazados.len()), (1, 0), "{v:?}");
+        // `fmt` esta en verde en ese informe.
+        let v = validar("r", &r("fmt"), &diff(), Some(&informe));
+        assert_eq!(v.motivos(), ["R:paso-rojo-no-esta"]);
+    }
+
+    /// Las instrucciones nombran cada motivo y su ejemplo pasa el validador.
+    #[test]
+    fn las_instrucciones_salen_del_validador() {
+        let t = instrucciones();
+        for (id, _) in MOTIVOS {
+            assert!(t.contains(&format!("`{id}`")), "falta {id}");
+        }
+        for x in [
+            "`CRITICO`",
+            "`IMPORTANTE`",
+            "`MENOR`",
+            "`reproduccion`",
+            "`mutante`",
+            "`borde`",
+        ] {
+            assert!(t.contains(x), "falta {x}");
+        }
+        let v = validar("r", &t, &diff(), None);
+        assert_eq!((v.aceptados.len(), v.rechazados.len()), (2, 0), "{v:?}");
+    }
+
+    /// El esquema declara los mismos motivos, severidades y tipos que el
+    /// codigo, y las mismas claves que la salida.
+    #[test]
+    fn el_esquema_y_el_codigo_dicen_lo_mismo() {
+        let e: Value = serde_json::from_str(ESQUEMA).unwrap_or_else(|x| panic!("{x}"));
+        let textos = |v: &Value| -> Vec<String> {
+            v.as_array()
+                .map(|a| {
+                    a.iter()
+                        .filter_map(Value::as_str)
+                        .map(str::to_owned)
+                        .collect()
+                })
+                .unwrap_or_default()
+        };
+        let motivos: Vec<String> = MOTIVOS.iter().map(|(m, _)| (*m).to_owned()).collect();
+        assert_eq!(
+            textos(&e["$defs"]["rechazo"]["properties"]["motivo"]["enum"]),
+            motivos
+        );
+        assert_eq!(
+            textos(&e["$defs"]["hallazgo"]["properties"]["severidad"]["enum"]),
+            ["CRITICO", "IMPORTANTE", "MENOR"]
+        );
+        assert_eq!(e["properties"]["version"]["const"], VERSION);
+        let v = validar("r", MIXTA, &diff(), None);
+        let salida = serde_json::to_value(&v).unwrap_or_else(|x| panic!("{x}"));
+        let claves = |v: &Value| -> Vec<String> {
+            let mut c: Vec<String> = v
+                .as_object()
+                .map(|m| m.keys().cloned().collect())
+                .unwrap_or_default();
+            c.sort();
+            c
+        };
+        assert_eq!(claves(&salida), claves(&e["properties"]));
+        assert_eq!(
+            claves(&salida["rechazados"][0]),
+            claves(&e["$defs"]["rechazo"]["properties"])
+        );
+        assert_eq!(
+            claves(&salida["aceptados"][0]),
+            claves(&e["$defs"]["hallazgo"]["properties"])
+        );
+        for (tipo, campos) in [
+            (TipoEvidencia::Reproduccion, "reproduccion"),
+            (TipoEvidencia::Mutante, "mutante"),
+            (TipoEvidencia::Borde, "borde"),
+        ] {
+            let mut esperadas: Vec<String> =
+                tipo.campos().iter().map(|c| (*c).to_owned()).collect();
+            esperadas.push("tipo".to_owned());
+            esperadas.sort();
+            assert_eq!(claves(&e["$defs"][campos]["properties"]), esperadas);
+        }
+    }
+
+    #[test]
+    fn el_resumen_no_deja_pasar_markdown_del_modelo() {
+        let malo = BORDE.replace(
+            "\"descripcion\":\"d\"",
+            "\"descripcion\":\"| x | ![i](http://e/p.png) <b>\\n# t `c`\"",
+        );
+        let v = validar("r", &una(&malo), &diff(), None);
+        let md = resumen_markdown(&v);
+        assert!(
+            md.contains(r"\| x \| \!\[i\]\(http://e/p.png\) \<b\> \# t \`c\`"),
+            "{md}"
+        );
+        assert_eq!(plano(&"a".repeat(10), 4), "aaaa…");
+        let v = validar("r", "nada", &diff(), None);
+        assert!(resumen_markdown(&v).contains("| respuesta | sin-bloque |"));
+    }
+}
diff --git a/crates/tc2/src/hallazgos/orden.rs b/crates/tc2/src/hallazgos/orden.rs
new file mode 100644
index 0000000..abb4c12
--- /dev/null
+++ b/crates/tc2/src/hallazgos/orden.rs
@@ -0,0 +1,191 @@
+//! `tc2 hallazgos [--json] validar --respuesta <md> --diff <txt>
+//! --revisora <id> [--informe <json>] [--salida <json>] [--resumen <md>]` y
+//! `tc2 hallazgos [--json] instrucciones` (F6-4).
+//!
+//! `validar` lee la respuesta de una revisora (con un tope de
+//! [`TOPE_RESPUESTA`] bytes), el diff que vio (tope [`TOPE_DIFF`]) y, si se
+//! da, el informe de verificacion de F5-4 (tope [`TOPE_INFORME`]; si no se
+//! lee, `paso_rojo` no se acepta), y escribe la linea `hallazgos ok
+//! revisora=<id> aceptados=N rechazados=M motivos=<quien:motivo,...>
+//! [informe=leido|sin-informe|grande|invalido]`. Con `--salida`, el JSON del
+//! esquema `hallazgos.schema.json`; con `--resumen`, la seccion en Markdown
+//! para el check `revision-ia`. Que haya rechazos no es un fallo de la
+//! orden: sale `ok` y el rojo lo decide F6-5.
+//!
+//! `instrucciones` escribe en stdout el texto que va al prompt
+//! ([`super::instrucciones`]); con `--json`, en la clave `instrucciones` del
+//! contrato.
+
+use std::io::Read;
+use std::path::{Path, PathBuf};
+
+use crate::Salida;
+use crate::contrato::{Informe, Resultado};
+use crate::i18n::catalogo::hallazgos as txt;
+use crate::i18n::{Idioma, Mensaje};
+use crate::verificacion::informe;
+
+/// Nombre de la orden.
+pub const NOMBRE: &str = "hallazgos";
+
+/// Bytes que se leen como mucho de la respuesta del modelo (el workflow la
+/// corta antes a 64 KiB).
+pub const TOPE_RESPUESTA: u64 = 256 * 1024;
+
+/// Bytes que se leen como mucho del diff.
+pub const TOPE_DIFF: u64 = 4 * 1024 * 1024;
+
+/// Bytes que se leen como mucho del informe de verificacion (el mismo tope
+/// que `tc2 revisoras`).
+pub const TOPE_INFORME: u64 = crate::seleccion::orden::TOPE_INFORME;
+
+#[derive(Default)]
+struct Args {
+    respuesta: Option<PathBuf>,
+    diff: Option<PathBuf>,
+    revisora: Option<String>,
+    informe: Option<PathBuf>,
+    salida: Option<PathBuf>,
+    resumen: Option<PathBuf>,
+}
+
+fn uso(motivo: &str) -> Informe {
+    Informe::nuevo(NOMBRE, Resultado::Uso)
+        .con("motivo", motivo)
+        .diagnostico(&txt::USO_HALLAZGOS)
+}
+
+/// Un id de revisora: el de `tc2 revisoras` (`plugin:skill`), sin espacios
+/// ni comas, para que la linea no cambie de forma.
+fn es_revisora(t: &str) -> bool {
+    !t.is_empty()
+        && t.len() <= 200
+        && t.chars()
+            .all(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '.' | '_' | '-' | '/'))
+}
+
+fn args(a: &[String]) -> Result<Args, Informe> {
+    let mut r = Args::default();
+    let mut it = a.iter();
+    while let Some(x) = it.next() {
+        let mut valor = || it.next().cloned().ok_or_else(|| uso("falta-valor"));
+        match x.as_str() {
+            "--respuesta" if r.respuesta.is_none() => r.respuesta = Some(valor()?.into()),
+            "--diff" if r.diff.is_none() => r.diff = Some(valor()?.into()),
+            "--revisora" if r.revisora.is_none() => {
+                let v = valor()?;
+                if !es_revisora(&v) {
+                    return Err(uso("revisora-invalida"));
+                }
+                r.revisora = Some(v);
+            }
+            "--informe" if r.informe.is_none() => r.informe = Some(valor()?.into()),
+            "--salida" if r.salida.is_none() => r.salida = Some(valor()?.into()),
+            "--resumen" if r.resumen.is_none() => r.resumen = Some(valor()?.into()),
+            _ => return Err(uso("argumento-desconocido")),
+        }
+    }
+    for (falta, motivo) in [
+        (r.respuesta.is_none(), "sin-respuesta"),
+        (r.diff.is_none(), "sin-diff"),
+        (r.revisora.is_none(), "sin-revisora"),
+    ] {
+        if falta {
+            return Err(uso(motivo));
+        }
+    }
+    Ok(r)
+}
+
+/// Lee como mucho `tope` bytes de `p`: `Ok(None)` si pasa del tope.
+fn leer_con_tope(p: &Path, tope: u64) -> std::io::Result<Option<String>> {
+    let mut b = Vec::new();
+    std::fs::File::open(p)?.take(tope + 1).read_to_end(&mut b)?;
+    if b.len() as u64 > tope {
+        return Ok(None);
+    }
+    Ok(Some(String::from_utf8_lossy(&b).into_owned()))
+}
+
+fn fallo_io(p: &Path, error: String) -> Informe {
+    Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(
+        Mensaje::nuevo(&txt::NO_SE_LEE)
+            .con("ruta", p.display().to_string())
+            .con("error", error),
+    )
+}
+
+fn leer_entrada(p: &Path, tope: u64) -> Result<String, Informe> {
+    leer_con_tope(p, tope)
+        .map_err(|e| fallo_io(p, e.to_string()))?
+        .ok_or_else(|| {
+            Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(
+                Mensaje::nuevo(&txt::DEMASIADO_GRANDE)
+                    .con("ruta", p.display().to_string())
+                    .con("tope", tope.to_string()),
+            )
+        })
+}
+
+/// `hallazgos ...` (sin `--json`, que quita `lib.rs`).
+pub fn orden(a: &[String], json: bool, idioma: Idioma) -> Salida {
+    match a {
+        [s] if s == "instrucciones" => {
+            let texto = super::instrucciones();
+            if json {
+                Informe::nuevo(NOMBRE, Resultado::Ok)
+                    .con("instrucciones", texto)
+                    .salida_en(true, idioma)
+            } else {
+                Salida {
+                    codigo: 0,
+                    stdout: texto,
+                    stderr: String::new(),
+                }
+            }
+        }
+        [s, ..] if s == "instrucciones" => uso("argumentos-de-mas").salida_en(json, idioma),
+        [s, resto @ ..] if s == "validar" => match validar(resto) {
+            Ok(i) | Err(i) => i.salida_en(json, idioma),
+        },
+        [] => uso("sin-suborden").salida_en(json, idioma),
+        _ => uso("suborden-desconocida").salida_en(json, idioma),
+    }
+}
+
+fn validar(a: &[String]) -> Result<Informe, Informe> {
+    let a = args(a)?;
+    let (Some(pr), Some(pd), Some(revisora)) = (&a.respuesta, &a.diff, &a.revisora) else {
+        return Err(uso("sin-respuesta"));
+    };
+    let respuesta = leer_entrada(pr, TOPE_RESPUESTA)?;
+    let diff = super::Diff::leer(&leer_entrada(pd, TOPE_DIFF)?);
+    let (estado, inf) = match &a.informe {
+        None => (None, None),
+        Some(p) => match leer_con_tope(p, TOPE_INFORME) {
+            Err(_) => (Some("sin-informe"), None),
+            Ok(None) => (Some("grande"), None),
+            Ok(Some(t)) => match informe::leer(&t) {
+                Ok(i) => (Some("leido"), Some(i)),
+                Err(_) => (Some("invalido"), None),
+            },
+        },
+    };
+    let v = super::validar(revisora, &respuesta, &diff, inf.as_ref());
+    if let Some(p) = &a.salida {
+        let texto = serde_json::to_string_pretty(&v).map_err(|e| fallo_io(p, e.to_string()))?;
+        std::fs::write(p, texto + "\n").map_err(|e| fallo_io(p, e.to_string()))?;
+    }
+    if let Some(p) = &a.resumen {
+        std::fs::write(p, super::resumen_markdown(&v)).map_err(|e| fallo_io(p, e.to_string()))?;
+    }
+    let mut i = Informe::nuevo(NOMBRE, Resultado::Ok)
+        .con("revisora", revisora.as_str())
+        .con("aceptados", v.aceptados.len().to_string())
+        .con("rechazados", v.rechazados.len().to_string())
+        .con_lista("motivos", v.motivos());
+    if let Some(e) = estado {
+        i = i.con("informe", e);
+    }
+    Ok(i)
+}
diff --git a/crates/tc2/src/i18n/catalogo/cli.rs b/crates/tc2/src/i18n/catalogo/cli.rs
index c6beca2..53ee3d1 100644
--- a/crates/tc2/src/i18n/catalogo/cli.rs
+++ b/crates/tc2/src/i18n/catalogo/cli.rs
@@ -10,8 +10,8 @@ crate::i18n::textos! {
         en: "tc2: could not write the output: {error}",
     }
     USO {
-        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<motivo>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <carril>] | tc2 plan guardar [--json] TASK-NNN <destino> <fichero> | tc2 approve [--json] TASK-NNN [--pedir-cambios <fichero>] | tc2 approve [--json] --hito <n> | tc2 review [--json] TASK-NNN | tc2 revisoras [--json] --rutas <fichero|-> [opciones] | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
-        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<reason>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <lane>] | tc2 plan guardar [--json] TASK-NNN <destination> <file> | tc2 approve [--json] TASK-NNN [--pedir-cambios <file>] | tc2 approve [--json] --hito <n> | tc2 review [--json] TASK-NNN | tc2 revisoras [--json] --rutas <file|-> [options] | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
+        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<motivo>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <carril>] | tc2 plan guardar [--json] TASK-NNN <destino> <fichero> | tc2 approve [--json] TASK-NNN [--pedir-cambios <fichero>] | tc2 approve [--json] --hito <n> | tc2 review [--json] TASK-NNN | tc2 revisoras [--json] --rutas <fichero|-> [opciones] | tc2 hallazgos [--json] (validar --respuesta <fichero> --diff <fichero> --revisora <id> [opciones] | instrucciones) | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
+        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<reason>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <lane>] | tc2 plan guardar [--json] TASK-NNN <destination> <file> | tc2 approve [--json] TASK-NNN [--pedir-cambios <file>] | tc2 approve [--json] --hito <n> | tc2 review [--json] TASK-NNN | tc2 revisoras [--json] --rutas <file|-> [options] | tc2 hallazgos [--json] (validar --respuesta <file> --diff <file> --revisora <id> [options] | instrucciones) | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
     }
     FALTA_LA_ORDEN {
         es: "tc2: falta la orden. {uso}",
diff --git a/crates/tc2/src/i18n/catalogo/hallazgos.rs b/crates/tc2/src/i18n/catalogo/hallazgos.rs
new file mode 100644
index 0000000..bba2c01
--- /dev/null
+++ b/crates/tc2/src/i18n/catalogo/hallazgos.rs
@@ -0,0 +1,16 @@
+//! Mensajes de `tc2 hallazgos` (F6-4).
+
+crate::i18n::textos! {
+    USO_HALLAZGOS {
+        es: "uso: hallazgos [--json] validar --respuesta <fichero> --diff <fichero> --revisora <id> [--informe <informe.json>] [--salida <salida.json>] [--resumen <salida.md>] | hallazgos [--json] instrucciones",
+        en: "usage: hallazgos [--json] validar --respuesta <file> --diff <file> --revisora <id> [--informe <report.json>] [--salida <output.json>] [--resumen <output.md>] | hallazgos [--json] instrucciones",
+    }
+    NO_SE_LEE {
+        es: "no se puede leer ni escribir {ruta}: {error}; comprueba la ruta y los permisos y vuelve a lanzar la orden",
+        en: "cannot read or write {ruta}: {error}; check the path and permissions and run the command again",
+    }
+    DEMASIADO_GRANDE {
+        es: "{ruta} pasa de {tope} bytes; recorta la entrada (el workflow corta la respuesta a 64 KiB y el diff a tope-diff)",
+        en: "{ruta} is larger than {tope} bytes; trim the input (the workflow cuts the response to 64 KiB and the diff to tope-diff)",
+    }
+}
diff --git a/crates/tc2/src/i18n/catalogo/mod.rs b/crates/tc2/src/i18n/catalogo/mod.rs
index a9238af..5661fb2 100644
--- a/crates/tc2/src/i18n/catalogo/mod.rs
+++ b/crates/tc2/src/i18n/catalogo/mod.rs
@@ -11,6 +11,7 @@ pub mod doctor;
 pub mod estado;
 pub mod estimacion;
 pub mod git;
+pub mod hallazgos;
 pub mod identidad;
 pub mod importar;
 pub mod init;
@@ -39,6 +40,7 @@ pub fn todos() -> impl Iterator<Item = &'static Texto> {
         estado::TODOS,
         estimacion::TODOS,
         git::TODOS,
+        hallazgos::TODOS,
         identidad::TODOS,
         importar::TODOS,
         init::TODOS,
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 49da780..e1a09ec 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -21,7 +21,8 @@
 //! (`descartar`, `pausa` y `reanudar`, F3-7; `liberar` y `reasignar`,
 //! F3-10) y [`doctor`] (F3-8) y [`plan`] (`plan` y `plan guardar`, F4-1;
 //! `approve`, F4-2) y [`revision`] (`review`, F6-2) y [`seleccion`]
-//! (`revisoras`, revisoras y modelo de F1-15, F6-3). Las capas
+//! (`revisoras`, revisoras y modelo de F1-15, F6-3) y [`hallazgos`]
+//! (`hallazgos`, hallazgos con evidencia, F6-4). Las capas
 //! que vienen entran como modulos de esta biblioteca.
 
 pub mod actualizar_cli;
@@ -37,6 +38,7 @@ pub mod estado;
 pub mod estimacion;
 pub mod git;
 pub mod grafo;
+pub mod hallazgos;
 pub mod huella;
 pub mod i18n;
 pub mod identidad;
@@ -224,6 +226,10 @@ where
             let (json, resto) = con_json(resto);
             orden_revisoras(&resto).salida_en(json, idioma)
         }
+        [orden, resto @ ..] if orden == hallazgos::orden::NOMBRE => {
+            let (json, resto) = con_json(resto);
+            hallazgos::orden::orden(&resto, json, idioma)
+        }
         [orden, resto @ ..] if orden == estimacion::NOMBRE => {
             let (json, resto) = con_json(resto);
             orden_estimar(&resto, json, idioma)
diff --git a/xtask/src/workflows.rs b/xtask/src/workflows.rs
index 886c7b2..22b15f9 100644
--- a/xtask/src/workflows.rs
+++ b/xtask/src/workflows.rs
@@ -1383,37 +1383,83 @@ fn las_reglas_de_gitlab_no_leen_variables_de_job() {
 
 const REVISION_IA: &str = ".github/workflows/revision-ia.yml";
 
-/// El workflow del revisor pasa el lint con el perfil revision-ia: dos
-/// jobs, el publicador en el entorno `revisor`, sin checkout, con los tres
-/// secretos y sin escritura del GITHUB_TOKEN.
+/// El workflow del revisor pasa el lint con el perfil revision-ia: cuatro
+/// jobs (F6-4); `consultar` y `publicar` en el entorno `revisor`, sin
+/// checkout, cada uno solo con su secreto; `preparar` y `validar` sin
+/// secretos; `validar` corre `tc2 hallazgos validar` y `publicar` lee
+/// `revision-ia-hallazgos`, no las respuestas del modelo en bruto.
 #[test]
 fn el_workflow_del_revisor_pasa_el_lint() {
     let texto = leer(REVISION_IA);
     let raiz = yaml(&texto);
     let jobs = raiz.get("jobs").map(Nodo::claves).unwrap_or_default();
-    assert_eq!(jobs, ["preparar", "publicar"]);
+    assert_eq!(jobs, ["preparar", "consultar", "validar", "publicar"]);
     assert_eq!(lint_revision(&texto), Vec::<String>::new());
-    let publicar = raiz.get("jobs").and_then(|j| j.get("publicar"));
-    assert_eq!(
-        publicar
-            .and_then(|p| p.get("environment"))
-            .and_then(Nodo::texto),
-        Some("revisor")
-    );
-    let mut valores = Vec::new();
-    if let Some(p) = publicar {
-        p.escalares("", &mut valores);
-    }
-    for s in [
-        "TC2_APP_REVISOR_ID",
-        "TC2_APP_REVISOR_CLAVE",
-        "ANTHROPIC_API_KEY",
-    ] {
+    let job = |n: &str| raiz.get("jobs").and_then(|j| j.get(n));
+    let valores = |n: &str| -> Vec<String> {
+        let mut v = Vec::new();
+        if let Some(j) = job(n) {
+            j.escalares("", &mut v);
+        }
+        v.iter().map(|(_, x)| normalizar(x)).collect()
+    };
+    let usa = |n: &str, buscado: &str| valores(n).iter().any(|v| v.contains(buscado));
+    let entorno = |n: &str| {
+        job(n)
+            .and_then(|j| j.get("environment"))
+            .and_then(Nodo::texto)
+    };
+    assert_eq!(entorno("consultar"), Some("revisor"));
+    assert_eq!(entorno("publicar"), Some("revisor"));
+    assert_eq!(entorno("preparar"), None);
+    assert_eq!(entorno("validar"), None);
+    let secretos = [
+        ("ANTHROPIC_API_KEY", "consultar"),
+        ("TC2_APP_REVISOR_ID", "publicar"),
+        ("TC2_APP_REVISOR_CLAVE", "publicar"),
+    ];
+    for (s, dueno) in secretos {
         let buscado = format!("secrets.{}", s.to_ascii_lowercase());
-        let en_publicar = valores
+        for n in jobs.iter().copied() {
+            assert_eq!(usa(n, &buscado), n == dueno, "{s} en {n}");
+        }
+    }
+    for n in ["preparar", "validar"] {
+        assert!(!usa(n, "secrets."), "secreto en {n}");
+    }
+    // validar valida con tc2 y sube el artefacto que lee publicar.
+    let runs_de = |n: &str| -> Vec<String> {
+        let mut v = Vec::new();
+        if let Some(j) = job(n) {
+            j.escalares("", &mut v);
+        }
+        v.iter()
+            .filter(|(k, _)| *k == "run")
+            .map(|(_, x)| (*x).to_owned())
+            .collect()
+    };
+    assert!(
+        runs_de("validar")
             .iter()
-            .any(|(_, v)| normalizar(v).contains(&buscado));
-        assert!(en_publicar, "falta el secreto {s} en publicar");
+            .any(|r| r.contains("tc2 hallazgos validar"))
+    );
+    assert!(usa("validar", "revision-ia-hallazgos"));
+    assert!(
+        runs_de("preparar")
+            .iter()
+            .any(|r| r.contains("tc2 hallazgos instrucciones"))
+    );
+    // publicar: solo el artefacto validado, nunca las respuestas en bruto.
+    assert!(usa("publicar", "revision-ia-hallazgos"));
+    assert!(!usa("publicar", "revision-ia-prompts"));
+    assert!(!usa("publicar", "needs.consultar.outputs"));
+    assert!(!usa("publicar", "api.anthropic.com"));
+    // Lo que viene de otro job (las respuestas del modelo) entra por env,
+    // nunca interpolado en run.
+    for n in jobs.iter().copied() {
+        for r in runs_de(n) {
+            assert!(!normalizar(&r).contains("needs."), "needs. en run de {n}");
+        }
     }
     // Con el perfil de verificacion, el entorno y el evento no valen.
     assert!(!lint_github(&texto).is_empty());
@@ -1422,7 +1468,7 @@ fn el_workflow_del_revisor_pasa_el_lint() {
 #[test]
 fn cada_mutacion_del_revisor_da_su_error() {
     let texto = leer(REVISION_IA);
-    let casos: [(&str, &str, &str); 9] = [
+    let casos: [(&str, &str, &str); 11] = [
         (
             "  workflow_call:\n",
             "  pull_request:\n  workflow_call:\n",
@@ -1446,6 +1492,11 @@ fn cada_mutacion_del_revisor_da_su_error() {
         (
             "    environment: revisor\n",
             "",
+            "job consultar: secreto en clase ejecuta",
+        ),
+        (
+            "    timeout-minutes: 15\n    environment: revisor\n",
+            "    timeout-minutes: 15\n",
             "job publicar: secreto en clase ejecuta",
         ),
         (
@@ -1454,10 +1505,15 @@ fn cada_mutacion_del_revisor_da_su_error() {
             "entorno no admitido",
         ),
         (
-            "    permissions:\n      actions: read\n    steps:\n      - name: Revisar",
-            "    permissions:\n      actions: read\n      checks: write\n    steps:\n      - name: Revisar",
+            "    permissions:\n      actions: read\n    steps:\n      - name: Publicar",
+            "    permissions:\n      actions: read\n      checks: write\n    steps:\n      - name: Publicar",
             "job publicar: escritura con GITHUB_TOKEN",
         ),
+        (
+            "          CONSULTAR: ${{ needs.consultar.result }}\n          R0:",
+            "          CONSULTAR: ${{ needs.consultar.result }}\n          CLAVE: ${{ secrets.ANTHROPIC_API_KEY }}\n          R0:",
+            "job validar: secreto en clase ejecuta",
+        ),
         (
             "          FORZADA: ${{ inputs.tc2-sha }}\n",
             "          FORZADA: ${{ secrets.ANTHROPIC_API_KEY }}\n",
@@ -1479,8 +1535,8 @@ fn cada_mutacion_del_revisor_da_su_error() {
     }
     // Un checkout en el publicador: privilegio por el entorno.
     let con_checkout = texto.replacen(
-        "    steps:\n      - name: Revisar",
-        "    steps:\n      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1\n        with:\n          persist-credentials: false\n          ref: develop\n      - name: Revisar",
+        "    steps:\n      - name: Publicar",
+        "    steps:\n      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1\n        with:\n          persist-credentials: false\n          ref: develop\n      - name: Publicar",
         1,
     );
     assert_ne!(con_checkout, texto);
@@ -1491,6 +1547,20 @@ fn cada_mutacion_del_revisor_da_su_error() {
             .any(|e| e.contains("job publicar: checkout en clase privilegio")),
         "{errores:?}"
     );
+    // consultar no sube artefactos: es de clase privilegio.
+    let con_subida = texto.replacen(
+        "          done < \"$dir/prompts/lista.tsv\"\n\n  validar:\n",
+        "          done < \"$dir/prompts/lista.tsv\"\n      - uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02\n        with:\n          name: x\n          path: y\n\n  validar:\n",
+        1,
+    );
+    assert_ne!(con_subida, texto);
+    let errores = lint_revision(&con_subida);
+    assert!(
+        errores
+            .iter()
+            .any(|e| e.contains("job consultar: actions/upload-artifact en clase privilegio")),
+        "{errores:?}"
+    );
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
tareas/{01-en-diseno => 02-en-curso}/TASK-050/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-050/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-050/tarea.md                                    | 3 ++-
 3 files changed, 2 insertions(+), 1 deletion(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/hallazgos.rs"
````

````
crates/tc2/tests/hallazgos.rs | 363 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 363 insertions(+)
````
