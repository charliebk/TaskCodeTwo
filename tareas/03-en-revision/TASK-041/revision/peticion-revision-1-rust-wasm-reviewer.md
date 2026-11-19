# Peticion de revision — TASK-041 (ronda 1)

- Tarea: TASK-041 — F5-1 Workflow reutilizable de GitHub Actions y plantilla de GitLab
- Rama revisada: feature/task-041-f5-1-workflow-reutilizable-de-github-act
- Rama base: develop
- Commit revisado (HEAD): 648720a2f6f21211bc3b31ff5e612126c6c7ce61
- Fecha: 2026-11-19
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-041 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-041/tarea.md
- tareas/03-en-revision/TASK-041/planificacion/
- xtask/src/main.rs
- xtask/src/workflows.rs

## Commits a revisar (git log develop..HEAD)

````
648720a docs(TASK-041): resultado del E2E, suposiciones verificadas y divergencias
790b72d feat(TASK-041): workflow reutilizable de verificacion, plantilla de GitLab y lint de lista blanca
e69d89a chore(TASK-041): tarea en curso
````

## Diff de tu dominio (2 fichero(s) de 5; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/xtask/src/main.rs b/xtask/src/main.rs
index f8f51e7..30550c7 100644
--- a/xtask/src/main.rs
+++ b/xtask/src/main.rs
@@ -25,6 +25,8 @@
 
 #[cfg(test)]
 mod instaladores;
+#[cfg(test)]
+mod workflows;
 
 use std::path::{Path, PathBuf};
 use std::process::{Command, ExitCode};
diff --git a/xtask/src/workflows.rs b/xtask/src/workflows.rs
new file mode 100644
index 0000000..fed108a
--- /dev/null
+++ b/xtask/src/workflows.rs
@@ -0,0 +1,736 @@
+//! Lint de lista blanca de los workflows de verificacion (TASK-041, F5-1).
+//!
+//! ADR F1-8, decision 1: los workflows de TaskCode separan dos clases de jobs
+//! (ejecuta: solo `contents: read` y ningun secreto; privilegio: escribe, y
+//! lee el PR solo como datos) y el lint se formula como **lista blanca**:
+//!
+//! - eventos admitidos: `workflow_call`, `pull_request`, `push`,
+//!   `merge_group` y `workflow_dispatch` (fuera `pull_request_target`,
+//!   `repository_dispatch`, `workflow_run`, comentarios...);
+//! - bloque `permissions` arriba y en cada job, sin `write-all` ni
+//!   `read-all`; un job con algun `write` es de clase privilegio;
+//! - acciones admitidas: solo `actions/checkout`, fijada por SHA y con
+//!   `persist-credentials: false`; en clase privilegio, sin `ref:` ni
+//!   `repository:`;
+//! - ni `secrets: inherit`, ni secretos en clase ejecuta, ni
+//!   `github.event.*`, `github.head_ref` o `toJSON(github...)` en `run:`
+//!   (los datos del evento se pasan por `env:`).
+//!
+//! En la plantilla de GitLab se aplica lo que le corresponde: ningun
+//! `secrets:` y ninguna variable que escribe el autor del MR o del commit en
+//! `script:`. Ademas, las entradas de la plantilla son las del workflow.
+//!
+//! Los ficheros son YAML de un subconjunto pequeno (mapas y listas por
+//! sangria, escalares, bloques `|` y mapas o listas de flujo sin anidar): el
+//! lector de abajo basta para ellos sin anadir dependencias al xtask.
+#![allow(clippy::panic)] // modulo solo de test: panic! es la forma de fallar
+
+use std::collections::BTreeSet;
+use std::path::PathBuf;
+
+#[derive(Debug, Clone, PartialEq, Eq)]
+enum Nodo {
+    Escalar(String),
+    Mapa(Vec<(String, Nodo)>),
+    Lista(Vec<Nodo>),
+}
+
+impl Nodo {
+    fn get(&self, clave: &str) -> Option<&Nodo> {
+        match self {
+            Nodo::Mapa(pares) => pares.iter().find(|(k, _)| k == clave).map(|(_, v)| v),
+            _ => None,
+        }
+    }
+
+    fn texto(&self) -> Option<&str> {
+        match self {
+            Nodo::Escalar(s) => Some(s),
+            _ => None,
+        }
+    }
+
+    fn claves(&self) -> Vec<&str> {
+        match self {
+            Nodo::Mapa(pares) => pares.iter().map(|(k, _)| k.as_str()).collect(),
+            _ => Vec::new(),
+        }
+    }
+
+    /// Todos los escalares del arbol, con la clave que los contiene.
+    fn escalares<'a>(&'a self, clave: &'a str, salida: &mut Vec<(&'a str, &'a str)>) {
+        match self {
+            Nodo::Escalar(s) => salida.push((clave, s)),
+            Nodo::Mapa(pares) => pares.iter().for_each(|(k, v)| v.escalares(k, salida)),
+            Nodo::Lista(items) => items.iter().for_each(|v| v.escalares(clave, salida)),
+        }
+    }
+}
+
+struct Linea {
+    sangria: usize,
+    texto: String,
+    vacia: bool,
+}
+
+/// Lee un documento YAML del subconjunto descrito arriba.
+fn leer_yaml(texto: &str) -> Nodo {
+    let mut lineas: Vec<Linea> = texto
+        .lines()
+        .map(|l| {
+            let sin_fin = l.trim_end();
+            let contenido = sin_fin.trim_start();
+            Linea {
+                sangria: sin_fin.len() - contenido.len(),
+                texto: contenido.to_owned(),
+                vacia: contenido.is_empty() || contenido.starts_with('#'),
+            }
+        })
+        .collect();
+    let mut i = 0;
+    saltar_vacias(&lineas, &mut i);
+    match lineas.get(i).map(|l| l.sangria) {
+        Some(sangria) => bloque(&mut lineas, &mut i, sangria),
+        None => Nodo::Mapa(Vec::new()),
+    }
+}
+
+fn saltar_vacias(lineas: &[Linea], i: &mut usize) {
+    while lineas.get(*i).is_some_and(|l| l.vacia) {
+        *i += 1;
+    }
+}
+
+fn es_item(texto: &str) -> bool {
+    texto == "-" || texto.starts_with("- ")
+}
+
+fn bloque(lineas: &mut [Linea], i: &mut usize, sangria: usize) -> Nodo {
+    if lineas.get(*i).is_some_and(|l| es_item(&l.texto)) {
+        lista(lineas, i, sangria)
+    } else {
+        mapa(lineas, i, sangria)
+    }
+}
+
+fn mapa(lineas: &mut [Linea], i: &mut usize, sangria: usize) -> Nodo {
+    let mut pares = Vec::new();
+    loop {
+        saltar_vacias(lineas, i);
+        let Some(linea) = lineas.get(*i) else { break };
+        if linea.sangria != sangria || es_item(&linea.texto) {
+            break;
+        }
+        let texto = linea.texto.clone();
+        *i += 1;
+        let (clave, valor) = match texto.find(": ") {
+            Some(p) => (&texto[..p], quitar_comentario(&texto[p + 2..])),
+            None => (texto.trim_end_matches(':'), String::new()),
+        };
+        let clave = sin_comillas(clave.trim()).to_owned();
+        let nodo = if valor.starts_with('|') || valor.starts_with('>') {
+            Nodo::Escalar(bloque_literal(lineas, i, sangria))
+        } else if !valor.is_empty() {
+            escalar_o_flujo(&valor)
+        } else {
+            saltar_vacias(lineas, i);
+            match lineas.get(*i) {
+                Some(sig) if sig.sangria > sangria => {
+                    let s = sig.sangria;
+                    bloque(lineas, i, s)
+                }
+                Some(sig) if sig.sangria == sangria && es_item(&sig.texto) => {
+                    lista(lineas, i, sangria)
+                }
+                _ => Nodo::Escalar(String::new()),
+            }
+        };
+        pares.push((clave, nodo));
+    }
+    Nodo::Mapa(pares)
+}
+
+fn lista(lineas: &mut [Linea], i: &mut usize, sangria: usize) -> Nodo {
+    let mut items = Vec::new();
+    loop {
+        saltar_vacias(lineas, i);
+        let Some(linea) = lineas.get(*i) else { break };
+        if linea.sangria != sangria || !es_item(&linea.texto) {
+            break;
+        }
+        let resto = linea.texto[1..].trim_start().to_owned();
+        if resto.is_empty() {
+            *i += 1;
+            saltar_vacias(lineas, i);
+            match lineas.get(*i) {
+                Some(sig) if sig.sangria > sangria => {
+                    let s = sig.sangria;
+                    items.push(bloque(lineas, i, s));
+                }
+                _ => items.push(Nodo::Escalar(String::new())),
+            }
+        } else if parece_clave(&resto) {
+            // `- clave: valor`: el item es un mapa cuya primera clave empieza
+            // donde acaba el guion; el resto de sus claves van a esa sangria.
+            let s = linea.sangria + (linea.texto.len() - resto.len());
+            if let Some(l) = lineas.get_mut(*i) {
+                l.sangria = s;
+                l.texto = resto;
+            }
+            items.push(mapa(lineas, i, s));
+        } else {
+            *i += 1;
+            items.push(escalar_o_flujo(&quitar_comentario(&resto)));
+        }
+    }
+    Nodo::Lista(items)
+}
+
+fn parece_clave(texto: &str) -> bool {
+    if texto.starts_with(['"', '\'', '$', '{', '[']) {
+        return false;
+    }
+    let clave = match texto.find(": ") {
+        Some(p) => &texto[..p],
+        None if texto.ends_with(':') => &texto[..texto.len() - 1],
+        None => return false,
+    };
+    !clave.is_empty()
+        && clave
+            .chars()
+            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
+}
+
+/// Las lineas de un bloque `|` o `>`: todas las que van mas sangradas que la
+/// clave, y las vacias entre ellas. Los `#` de dentro no son comentarios.
+fn bloque_literal(lineas: &[Linea], i: &mut usize, sangria: usize) -> String {
+    let mut texto = Vec::new();
+    let base = lineas[*i..]
+        .iter()
+        .find(|x| !x.texto.is_empty())
+        .map_or(0, |x| x.sangria);
+    while let Some(l) = lineas.get(*i) {
+        let en_blanco = l.texto.is_empty();
+        if !en_blanco && l.sangria <= sangria {
+            break;
+        }
+        if en_blanco {
+            let sigue = lineas[*i..]
+                .iter()
+                .find(|x| !x.texto.is_empty())
+                .is_some_and(|x| x.sangria > sangria);
+            if !sigue {
+                break;
+            }
+        }
+        let relativa = if en_blanco {
+            0
+        } else {
+            l.sangria.saturating_sub(base)
+        };
+        texto.push(format!("{}{}", " ".repeat(relativa), l.texto));
+        *i += 1;
+    }
+    texto.join("\n")
+}
+
+fn quitar_comentario(valor: &str) -> String {
+    let mut comillas: Option<char> = None;
+    let mut anterior = ' ';
+    for (p, c) in valor.char_indices() {
+        match comillas {
+            Some(q) if c == q => comillas = None,
+            Some(_) => {}
+            None if (c == '"' || c == '\'') && anterior == ' ' => comillas = Some(c),
+            None if c == '#' && anterior == ' ' => return valor[..p].trim_end().to_owned(),
+            None => {}
+        }
+        anterior = c;
+    }
+    valor.trim_end().to_owned()
+}
+
+fn sin_comillas(valor: &str) -> &str {
+    let v = valor.trim();
+    for q in ['"', '\''] {
+        if v.len() >= 2 && v.starts_with(q) && v.ends_with(q) {
+            return &v[1..v.len() - 1];
+        }
+    }
+    v
+}
+
+fn escalar_o_flujo(valor: &str) -> Nodo {
+    let v = valor.trim();
+    let partes = |interior: &str| -> Vec<String> {
+        interior
+            .split(',')
+            .map(str::trim)
+            .filter(|s| !s.is_empty())
+            .map(str::to_owned)
+            .collect()
+    };
+    if v.starts_with('{') && v.ends_with('}') && !v.starts_with("{{") {
+        let pares = partes(&v[1..v.len() - 1])
+            .into_iter()
+            .map(|p| match p.split_once(':') {
+                Some((k, x)) => (
+                    sin_comillas(k).to_owned(),
+                    Nodo::Escalar(sin_comillas(x).to_owned()),
+                ),
+                None => (sin_comillas(&p).to_owned(), Nodo::Escalar(String::new())),
+            })
+            .collect();
+        return Nodo::Mapa(pares);
+    }
+    if v.starts_with('[') && v.ends_with(']') {
+        let items = partes(&v[1..v.len() - 1])
+            .into_iter()
+            .map(|p| Nodo::Escalar(sin_comillas(&p).to_owned()))
+            .collect();
+        return Nodo::Lista(items);
+    }
+    Nodo::Escalar(sin_comillas(v).to_owned())
+}
+
+// --- Lint del workflow de GitHub ---------------------------------------------
+
+const EVENTOS_ADMITIDOS: [&str; 5] = [
+    "workflow_call",
+    "pull_request",
+    "push",
+    "merge_group",
+    "workflow_dispatch",
+];
+
+const ACCIONES_ADMITIDAS: [&str; 1] = ["actions/checkout"];
+
+/// Datos del evento que el autor del PR controla: nunca dentro de `run:`.
+const DATOS_DEL_EVENTO: [&str; 5] = [
+    "github.event.",
+    "github.event[",
+    "github.head_ref",
+    "tojson(github",
+    "secrets.",
+];
+
+fn eventos(on: &Nodo) -> Vec<String> {
+    match on {
+        Nodo::Escalar(s) => vec![s.clone()],
+        Nodo::Lista(items) => items
+            .iter()
+            .filter_map(|n| n.texto().map(str::to_owned))
+            .collect(),
+        Nodo::Mapa(_) => on.claves().into_iter().map(str::to_owned).collect(),
+    }
+}
+
+/// `Ok(true)` si los permisos tienen algun `write`; `Err` si no son un bloque
+/// explicito valido.
+fn escribe(permisos: Option<&Nodo>) -> Result<bool, String> {
+    match permisos {
+        None => Err("sin bloque permissions".to_owned()),
+        Some(Nodo::Mapa(pares)) => {
+            let mut escribe = false;
+            for (alcance, valor) in pares {
+                match valor.texto() {
+                    Some("read" | "none") => {}
+                    Some("write") => escribe = true,
+                    otro => return Err(format!("permiso {alcance}: {otro:?}")),
+                }
+            }
+            Ok(escribe)
+        }
+        Some(Nodo::Escalar(s)) if s == "{}" => Ok(false),
+        Some(otro) => Err(format!("permissions no admitido: {otro:?}")),
+    }
+}
+
+fn es_sha(texto: &str) -> bool {
+    texto.len() == 40 && texto.chars().all(|c| c.is_ascii_hexdigit())
+}
+
+fn lint_github(texto: &str) -> Vec<String> {
+    let raiz = leer_yaml(texto);
+    let mut errores = Vec::new();
+    match raiz.get("on") {
+        None => errores.push("sin bloque on".to_owned()),
+        Some(on) => {
+            for e in eventos(on) {
+                if !EVENTOS_ADMITIDOS.contains(&e.as_str()) {
+                    errores.push(format!("evento no admitido: {e}"));
+                }
+            }
+            if on
+                .get("workflow_call")
+                .and_then(|w| w.get("secrets"))
+                .is_some()
+            {
+                errores.push("workflow_call declara secretos".to_owned());
+            }
+        }
+    }
+    if let Err(e) = escribe(raiz.get("permissions")) {
+        errores.push(format!("raiz: {e}"));
+    }
+    let Some(jobs @ Nodo::Mapa(_)) = raiz.get("jobs") else {
+        errores.push("sin jobs".to_owned());
+        return errores;
+    };
+    for nombre in jobs.claves() {
+        let Some(job) = jobs.get(nombre) else {
+            continue;
+        };
+        let privilegio = match escribe(job.get("permissions")) {
+            Ok(p) => p,
+            Err(e) => {
+                errores.push(format!("job {nombre}: {e}"));
+                false
+            }
+        };
+        let mut valores = Vec::new();
+        job.escalares("", &mut valores);
+        for (clave, valor) in &valores {
+            if *clave == "secrets" && *valor == "inherit" {
+                errores.push(format!("job {nombre}: secrets: inherit"));
+            }
+            if !privilegio && valor.contains("secrets.") {
+                errores.push(format!("job {nombre}: secreto en clase ejecuta"));
+            }
+            if *clave == "run" {
+                let minusculas = valor.to_ascii_lowercase();
+                for dato in DATOS_DEL_EVENTO {
+                    if minusculas.contains(dato) {
+                        errores.push(format!("job {nombre}: {dato} en run"));
+                    }
+                }
+            }
+        }
+        let pasos = match job.get("steps") {
+            Some(Nodo::Lista(p)) => p.as_slice(),
+            _ => &[],
+        };
+        for paso in pasos {
+            let Some(uses) = paso.get("uses").and_then(Nodo::texto) else {
+                continue;
+            };
+            let (accion, version) = uses.split_once('@').unwrap_or((uses, ""));
+            if !ACCIONES_ADMITIDAS.contains(&accion) {
+                errores.push(format!("job {nombre}: accion no admitida: {accion}"));
+                continue;
+            }
+            if !es_sha(version) {
+                errores.push(format!("job {nombre}: {accion} sin fijar por SHA"));
+            }
+            let con = paso.get("with");
+            let opcion = |k: &str| con.and_then(|w| w.get(k));
+            if opcion("persist-credentials").and_then(Nodo::texto) != Some("false") {
+                errores.push(format!(
+                    "job {nombre}: checkout sin persist-credentials: false"
+                ));
+            }
+            if privilegio && (opcion("ref").is_some() || opcion("repository").is_some()) {
+                errores.push(format!(
+                    "job {nombre}: checkout con ref o repository en clase privilegio"
+                ));
+            }
+        }
+    }
+    errores
+}
+
+// --- Lint de la plantilla de GitLab ------------------------------------------
+
+/// Variables predefinidas que escribe el autor del MR o del commit: nunca
+/// dentro de `script:` (se pasan, si hace falta, por `variables:`).
+const VARIABLES_DEL_AUTOR: [&str; 7] = [
+    "CI_MERGE_REQUEST_TITLE",
+    "CI_MERGE_REQUEST_DESCRIPTION",
+    "CI_MERGE_REQUEST_SOURCE_BRANCH_NAME",
+    "CI_COMMIT_MESSAGE",
+    "CI_COMMIT_TITLE",
+    "CI_COMMIT_DESCRIPTION",
+    "CI_COMMIT_REF_NAME",
+];
+
+/// La cabecera `spec:` y el cuerpo de una plantilla con `spec:inputs`.
+fn partir_plantilla(texto: &str) -> (Nodo, Nodo) {
+    match texto.split_once("\n---") {
+        Some((cabecera, cuerpo)) => (leer_yaml(cabecera), leer_yaml(cuerpo)),
+        None => (Nodo::Mapa(Vec::new()), leer_yaml(texto)),
+    }
+}
+
+fn lint_gitlab(texto: &str) -> Vec<String> {
+    let (cabecera, cuerpo) = partir_plantilla(texto);
+    let mut errores = Vec::new();
+    if cabecera.get("spec").and_then(|s| s.get("inputs")).is_none() {
+        errores.push("sin spec:inputs".to_owned());
+    }
+    for nombre in cuerpo.claves() {
+        let Some(job) = cuerpo.get(nombre) else {
+            continue;
+        };
+        if job.get("secrets").is_some() {
+            errores.push(format!("job {nombre}: secrets"));
+        }
+        let mut lineas = Vec::new();
+        for clave in ["script", "before_script", "after_script"] {
+            if let Some(s) = job.get(clave) {
+                s.escalares(clave, &mut lineas);
+            }
+        }
+        for (clave, linea) in lineas {
+            for variable in VARIABLES_DEL_AUTOR {
+                if linea.contains(variable) {
+                    errores.push(format!("job {nombre}: {variable} en {clave}"));
+                }
+            }
+        }
+    }
+    errores
+}
+
+// --- Pruebas -------------------------------------------------------------------
+
+fn leer(relativa: &str) -> String {
+    let ruta = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
+        .join("..")
+        .join(relativa);
+    std::fs::read_to_string(&ruta).unwrap_or_else(|e| panic!("{}: {e}", ruta.display()))
+}
+
+const WORKFLOW: &str = ".github/workflows/verificacion.yml";
+const PLANTILLA: &str = "plantillas/gitlab/verificacion.gitlab-ci.yml";
+
+#[test]
+fn el_lector_entiende_el_subconjunto() {
+    let yaml = "\
+a: 1 # comentario
+b:
+  - c: x
+    d: 'y # no'
+  - z
+e: { k: v, j: w }
+f: [p, q]
+g: |
+  linea # dentro
+    sangrada
+
+  tras vacia
+h:
+- i
+";
+    let raiz = leer_yaml(yaml);
+    assert_eq!(raiz.get("a"), Some(&Nodo::Escalar("1".into())));
+    let Some(Nodo::Lista(b)) = raiz.get("b") else {
+        panic!("b no es lista: {raiz:?}");
+    };
+    assert_eq!(b[0].get("d"), Some(&Nodo::Escalar("y # no".into())));
+    assert_eq!(b[1], Nodo::Escalar("z".into()));
+    assert_eq!(
+        raiz.get("e").and_then(|e| e.get("j")),
+        Some(&Nodo::Escalar("w".into()))
+    );
+    assert_eq!(
+        raiz.get("f"),
+        Some(&Nodo::Lista(vec![
+            Nodo::Escalar("p".into()),
+            Nodo::Escalar("q".into())
+        ]))
+    );
+    assert_eq!(
+        raiz.get("g").and_then(Nodo::texto),
+        Some("linea # dentro\n  sangrada\n\ntras vacia")
+    );
+    assert_eq!(
+        raiz.get("h"),
+        Some(&Nodo::Lista(vec![Nodo::Escalar("i".into())]))
+    );
+}
+
+#[test]
+fn el_workflow_reutilizable_pasa_el_lint() {
+    let texto = leer(WORKFLOW);
+    let raiz = leer_yaml(&texto);
+    // El lector ve lo que hay que ver: tres jobs con sus pasos.
+    let jobs = raiz.get("jobs").map(Nodo::claves).unwrap_or_default();
+    assert_eq!(jobs, ["buscar", "ejecutar", "verificacion"]);
+    assert_eq!(lint_github(&texto), Vec::<String>::new());
+}
+
+#[test]
+fn la_plantilla_de_gitlab_pasa_el_lint() {
+    let texto = leer(PLANTILLA);
+    let (_, cuerpo) = partir_plantilla(&texto);
+    assert!(cuerpo.claves().contains(&"verificacion-completa"));
+    assert_eq!(lint_gitlab(&texto), Vec::<String>::new());
+}
+
+/// Las entradas comunes son las mismas en los dos: lo propio de cada
+/// plataforma es el runner (`runner-*` y `shell` en GitHub, `tags-*` y
+/// `stage` en GitLab).
+#[test]
+fn workflow_y_plantilla_tienen_las_mismas_entradas() {
+    let raiz = leer_yaml(&leer(WORKFLOW));
+    let github: BTreeSet<&str> = raiz
+        .get("on")
+        .and_then(|o| o.get("workflow_call"))
+        .and_then(|w| w.get("inputs"))
+        .map(Nodo::claves)
+        .unwrap_or_default()
+        .into_iter()
+        .filter(|k| !k.starts_with("runner-") && *k != "shell")
+        .collect();
+    let texto = leer(PLANTILLA);
+    let (cabecera, _) = partir_plantilla(&texto);
+    let gitlab: BTreeSet<&str> = cabecera
+        .get("spec")
+        .and_then(|s| s.get("inputs"))
+        .map(Nodo::claves)
+        .unwrap_or_default()
+        .into_iter()
+        .filter(|k| !k.starts_with("tags-") && *k != "stage")
+        .collect();
+    assert_eq!(github.len(), 6, "{github:?}");
+    assert_eq!(github, gitlab);
+}
+
+/// Un workflow minimo que cumple todo: el caso positivo de las mutaciones.
+const BUENO: &str = "\
+on:
+  pull_request:
+permissions: {}
+jobs:
+  ejecutar:
+    permissions:
+      contents: read
+    env:
+      BASE: ${{ github.base_ref }}
+      TITULO: ${{ github.event.pull_request.title }}
+    steps:
+      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
+        with:
+          persist-credentials: false
+      - run: echo \"$TITULO\"
+  publicar:
+    permissions:
+      checks: write
+    steps:
+      - env:
+          GH_TOKEN: ${{ github.token }}
+        run: |
+          # comentario con secrets en prosa no cuenta: es un bloque
+          gh api repos/x/check-runs
+";
+
+#[test]
+fn el_caso_positivo_pasa() {
+    assert_eq!(lint_github(BUENO), Vec::<String>::new());
+}
+
+#[test]
+fn cada_mutacion_prohibida_da_su_error() {
+    let casos: [(&str, &str, &str); 11] = [
+        (
+            "  pull_request:\n",
+            "  pull_request_target:\n",
+            "evento no admitido: pull_request_target",
+        ),
+        (
+            "  pull_request:\n",
+            "  pull_request:\n  workflow_run:\n",
+            "evento no admitido: workflow_run",
+        ),
+        ("permissions: {}\n", "permissions: write-all\n", "raiz"),
+        (
+            "    permissions:\n      contents: read\n",
+            "    runs-on: x\n",
+            "job ejecutar: sin bloque permissions",
+        ),
+        (
+            "      - run: echo \"$TITULO\"",
+            "      - run: echo \"${{ github.event.pull_request.title }}\"",
+            "github.event. en run",
+        ),
+        (
+            "      - run: echo \"$TITULO\"",
+            "      - run: git push origin ${{ github.head_ref }}",
+            "github.head_ref en run",
+        ),
+        (
+            "      BASE: ${{ github.base_ref }}",
+            "      BASE: ${{ secrets.TOKEN }}",
+            "secreto en clase ejecuta",
+        ),
+        (
+            "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1",
+            "actions/checkout@v4",
+            "sin fijar por SHA",
+        ),
+        (
+            "          persist-credentials: false\n",
+            "          fetch-depth: 0\n",
+            "persist-credentials",
+        ),
+        (
+            "      - env:\n          GH_TOKEN",
+            "      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1\n        with:\n          persist-credentials: false\n          ref: ${{ github.event.pull_request.head.sha }}\n      - env:\n          GH_TOKEN",
+            "ref o repository en clase privilegio",
+        ),
+        (
+            "      - run: echo \"$TITULO\"",
+            "      - uses: actions/github-script@60a0d83039c74a4aee543508d2ffcb1c3799cdea",
+            "accion no admitida: actions/github-script",
+        ),
+    ];
+    for (original, mutado, esperado) in casos {
+        assert!(BUENO.contains(original), "el caso no casa: {original}");
+        let texto = BUENO.replacen(original, mutado, 1);
+        let errores = lint_github(&texto);
+        assert!(
+            errores.iter().any(|e| e.contains(esperado)),
+            "«{esperado}» no salio; errores: {errores:?}"
+        );
+    }
+    let llamador = "on:\n  push:\npermissions: {}\njobs:\n  v:\n    permissions:\n      contents: read\n    uses: o/r/.github/workflows/x.yml@0000000000000000000000000000000000000000\n    secrets: inherit\n";
+    assert!(
+        lint_github(llamador)
+            .iter()
+            .any(|e| e.contains("secrets: inherit")),
+        "{:?}",
+        lint_github(llamador)
+    );
+}
+
+#[test]
+fn la_plantilla_de_gitlab_rechaza_datos_del_autor_y_secretos() {
+    let texto = leer(PLANTILLA);
+    let original = "    - $[[ inputs.comando ]] $[[ inputs.argumentos-completa ]]\n";
+    assert!(texto.contains(original));
+    let mutado = texto.replacen(
+        original,
+        "    - echo \"$CI_COMMIT_MESSAGE\"\n    - $[[ inputs.comando ]]\n",
+        1,
+    );
+    let errores = lint_gitlab(&mutado);
+    assert!(
+        errores
+            .iter()
+            .any(|e| e.contains("verificacion-completa: CI_COMMIT_MESSAGE en script")),
+        "{errores:?}"
+    );
+    let con_secreto = texto.replacen(
+        "verificacion-pesada:\n",
+        "verificacion-pesada:\n  secrets:\n    CLAVE:\n      vault: x/y@z\n",
+        1,
+    );
+    assert!(
+        lint_gitlab(&con_secreto)
+            .iter()
+            .any(|e| e.contains("verificacion-pesada: secrets")),
+        "{:?}",
+        lint_gitlab(&con_secreto)
+    );
+    assert!(lint_gitlab("a:\n  script: [x]\n").contains(&"sin spec:inputs".to_owned()));
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
tareas/{01-en-diseno => 02-en-curso}/TASK-041/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-041/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-041/tarea.md                                    | 3 ++-
 3 files changed, 2 insertions(+), 1 deletion(-)
````
