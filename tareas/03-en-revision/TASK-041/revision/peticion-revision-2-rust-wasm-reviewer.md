# Peticion de revision — TASK-041 (ronda 2)

- Tarea: TASK-041 — F5-1 Workflow reutilizable de GitHub Actions y plantilla de GitLab
- Rama revisada: feature/task-041-f5-1-workflow-reutilizable-de-github-act
- Rama base: develop
- Commit revisado (HEAD): 5fe9c4fca6475b66f6bbe86d8025263d7efd485e
- Fecha: 2026-11-19
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-041 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 648720a2f6f21211bc3b31ff5e612126c6c7ce61 (el commit revisado en la ronda anterior)
- Alcance: completa (motivo: un informe de la ronda anterior no tiene tabla de hallazgos legible)

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
informe de esta ronda (informe-revision-2-rust-wasm-reviewer.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-041/tarea.md
- tareas/03-en-revision/TASK-041/planificacion/
- xtask/src/workflows.rs

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido (944a974) | xtask/src/workflows.rs:145-164 (`leer_yaml`), 184-219 (`mapa`), 221-255 (`lista`) | informe-revision-1-rust-wasm-reviewer.md |
| IMP-2 | IMPORTANTE | corregido (944a974) | xtask/src/workflows.rs:441-447 (raiz sin inspeccionar), 377-383 y 465-474 (patrones) | informe-revision-1-rust-wasm-reviewer.md |
| IMP-3 | IMPORTANTE | corregido (944a974) | xtask/src/workflows.rs:705-711 (`la_plantilla_de_gitlab_rechaza_datos_del_autor_y_secretos`), `.gitattributes | informe-revision-1-rust-wasm-reviewer.md |
| MEN-1 | MENOR | corregido (944a974) | xtask/src/workflows.rs:477-481 y 448-506 (jobs con `uses:` a nivel de job) | informe-revision-1-rust-wasm-reviewer.md |

Tabla de hallazgos ausente o ilegible en: informe-revision-1-code-quality-reviewer.md. Lee esos informes enteros.

## Commits a revisar (git log 648720a2f6f21211bc3b31ff5e612126c6c7ce61..HEAD)

````
5fe9c4f docs(TASK-041): pipeline real de GitLab en verde y en rojo
944a974 fix(TASK-041): hallazgos de la ronda 1 de revision
171ec22 chore(TASK-041): peticion de revision ronda 1
````

## Diff de tu dominio (1 fichero(s) de 5; git diff 648720a2f6f21211bc3b31ff5e612126c6c7ce61..HEAD -- <tus ficheros>)

````diff
diff --git a/xtask/src/workflows.rs b/xtask/src/workflows.rs
index fed108a..879d5e5 100644
--- a/xtask/src/workflows.rs
+++ b/xtask/src/workflows.rs
@@ -73,8 +73,11 @@ struct Linea {
     vacia: bool,
 }
 
-/// Lee un documento YAML del subconjunto descrito arriba.
-fn leer_yaml(texto: &str) -> Nodo {
+/// Lee un documento YAML del subconjunto descrito arriba. Falla en cerrado:
+/// si queda alguna linea sin consumir (sangria inesperada, continuacion de un
+/// escalar plano...) o una linea de mapa no es `clave:`/`clave: valor`, el
+/// documento no se entiende y el lint no puede darlo por bueno.
+fn leer_yaml(texto: &str) -> Result<Nodo, String> {
     let mut lineas: Vec<Linea> = texto
         .lines()
         .map(|l| {
@@ -89,12 +92,26 @@ fn leer_yaml(texto: &str) -> Nodo {
         .collect();
     let mut i = 0;
     saltar_vacias(&lineas, &mut i);
-    match lineas.get(i).map(|l| l.sangria) {
-        Some(sangria) => bloque(&mut lineas, &mut i, sangria),
+    let raiz = match lineas.get(i).map(|l| l.sangria) {
+        Some(sangria) => bloque(&mut lineas, &mut i, sangria)?,
         None => Nodo::Mapa(Vec::new()),
+    };
+    saltar_vacias(&lineas, &mut i);
+    match lineas.get(i) {
+        Some(l) => Err(format!(
+            "documento no entendido: linea {}: {}",
+            i + 1,
+            l.texto
+        )),
+        None => Ok(raiz),
     }
 }
 
+/// `leer_yaml` para las pruebas que leen un documento que tiene que entenderse.
+fn yaml(texto: &str) -> Nodo {
+    leer_yaml(texto).unwrap_or_else(|e| panic!("{e}"))
+}
+
 fn saltar_vacias(lineas: &[Linea], i: &mut usize) {
     while lineas.get(*i).is_some_and(|l| l.vacia) {
         *i += 1;
@@ -105,7 +122,7 @@ fn es_item(texto: &str) -> bool {
     texto == "-" || texto.starts_with("- ")
 }
 
-fn bloque(lineas: &mut [Linea], i: &mut usize, sangria: usize) -> Nodo {
+fn bloque(lineas: &mut [Linea], i: &mut usize, sangria: usize) -> Result<Nodo, String> {
     if lineas.get(*i).is_some_and(|l| es_item(&l.texto)) {
         lista(lineas, i, sangria)
     } else {
@@ -113,7 +130,7 @@ fn bloque(lineas: &mut [Linea], i: &mut usize, sangria: usize) -> Nodo {
     }
 }
 
-fn mapa(lineas: &mut [Linea], i: &mut usize, sangria: usize) -> Nodo {
+fn mapa(lineas: &mut [Linea], i: &mut usize, sangria: usize) -> Result<Nodo, String> {
     let mut pares = Vec::new();
     loop {
         saltar_vacias(lineas, i);
@@ -124,8 +141,11 @@ fn mapa(lineas: &mut [Linea], i: &mut usize, sangria: usize) -> Nodo {
         let texto = linea.texto.clone();
         *i += 1;
         let (clave, valor) = match texto.find(": ") {
-            Some(p) => (&texto[..p], quitar_comentario(&texto[p + 2..])),
-            None => (texto.trim_end_matches(':'), String::new()),
+            Some(p) => (texto[..p].to_owned(), quitar_comentario(&texto[p + 2..])),
+            None => match quitar_comentario(&texto).strip_suffix(':') {
+                Some(c) => (c.to_owned(), String::new()),
+                None => return Err(format!("documento no entendido: linea {}: {texto}", *i)),
+            },
         };
         let clave = sin_comillas(clave.trim()).to_owned();
         let nodo = if valor.starts_with('|') || valor.starts_with('>') {
@@ -137,20 +157,20 @@ fn mapa(lineas: &mut [Linea], i: &mut usize, sangria: usize) -> Nodo {
             match lineas.get(*i) {
                 Some(sig) if sig.sangria > sangria => {
                     let s = sig.sangria;
-                    bloque(lineas, i, s)
+                    bloque(lineas, i, s)?
                 }
                 Some(sig) if sig.sangria == sangria && es_item(&sig.texto) => {
-                    lista(lineas, i, sangria)
+                    lista(lineas, i, sangria)?
                 }
                 _ => Nodo::Escalar(String::new()),
             }
         };
         pares.push((clave, nodo));
     }
-    Nodo::Mapa(pares)
+    Ok(Nodo::Mapa(pares))
 }
 
-fn lista(lineas: &mut [Linea], i: &mut usize, sangria: usize) -> Nodo {
+fn lista(lineas: &mut [Linea], i: &mut usize, sangria: usize) -> Result<Nodo, String> {
     let mut items = Vec::new();
     loop {
         saltar_vacias(lineas, i);
@@ -165,7 +185,7 @@ fn lista(lineas: &mut [Linea], i: &mut usize, sangria: usize) -> Nodo {
             match lineas.get(*i) {
                 Some(sig) if sig.sangria > sangria => {
                     let s = sig.sangria;
-                    items.push(bloque(lineas, i, s));
+                    items.push(bloque(lineas, i, s)?);
                 }
                 _ => items.push(Nodo::Escalar(String::new())),
             }
@@ -177,13 +197,13 @@ fn lista(lineas: &mut [Linea], i: &mut usize, sangria: usize) -> Nodo {
                 l.sangria = s;
                 l.texto = resto;
             }
-            items.push(mapa(lineas, i, s));
+            items.push(mapa(lineas, i, s)?);
         } else {
             *i += 1;
             items.push(escalar_o_flujo(&quitar_comentario(&resto)));
         }
     }
-    Nodo::Lista(items)
+    Ok(Nodo::Lista(items))
 }
 
 fn parece_clave(texto: &str) -> bool {
@@ -306,6 +326,7 @@ const EVENTOS_ADMITIDOS: [&str; 5] = [
 const ACCIONES_ADMITIDAS: [&str; 1] = ["actions/checkout"];
 
 /// Datos del evento que el autor del PR controla: nunca dentro de `run:`.
+/// Se buscan sobre el valor normalizado (ver `normalizar`).
 const DATOS_DEL_EVENTO: [&str; 5] = [
     "github.event.",
     "github.event[",
@@ -314,6 +335,26 @@ const DATOS_DEL_EVENTO: [&str; 5] = [
     "secrets.",
 ];
 
+/// El valor en minusculas, sin espacios y con los corchetes de cadena como
+/// puntos: `toJSON( github)` pasa a `tojson(github` y `github['event']` o
+/// `secrets["X"]` a `github.event` y `secrets.x`, para que los patrones no
+/// dependan de como se escriba la expresion.
+fn normalizar(valor: &str) -> String {
+    valor
+        .to_ascii_lowercase()
+        .chars()
+        .filter(|c| !c.is_whitespace())
+        .collect::<String>()
+        .replace("['", ".")
+        .replace("[\"", ".")
+        .replace("']", "")
+        .replace("\"]", "")
+}
+
+fn usa_secretos(normalizado: &str) -> bool {
+    normalizado.contains("secrets.") || normalizado.contains("secrets[")
+}
+
 fn eventos(on: &Nodo) -> Vec<String> {
     match on {
         Nodo::Escalar(s) => vec![s.clone()],
@@ -351,7 +392,10 @@ fn es_sha(texto: &str) -> bool {
 }
 
 fn lint_github(texto: &str) -> Vec<String> {
-    let raiz = leer_yaml(texto);
+    let raiz = match leer_yaml(texto) {
+        Ok(r) => r,
+        Err(e) => return vec![e],
+    };
     let mut errores = Vec::new();
     match raiz.get("on") {
         None => errores.push("sin bloque on".to_owned()),
@@ -373,6 +417,28 @@ fn lint_github(texto: &str) -> Vec<String> {
     if let Err(e) = escribe(raiz.get("permissions")) {
         errores.push(format!("raiz: {e}"));
     }
+    // `env` y `defaults` de la raiz llegan a todos los jobs, tambien a los de
+    // clase ejecuta: ningun secreto en ellos, y en `defaults` (que acaba en
+    // la linea de comandos: shell, working-directory) tampoco datos del evento.
+    for bloque_raiz in ["env", "defaults"] {
+        let mut valores = Vec::new();
+        if let Some(n) = raiz.get(bloque_raiz) {
+            n.escalares(bloque_raiz, &mut valores);
+        }
+        for (_, valor) in valores {
+            let normalizado = normalizar(valor);
+            if usa_secretos(&normalizado) {
+                errores.push(format!("raiz: secreto en {bloque_raiz}"));
+            }
+            if bloque_raiz == "defaults" {
+                for dato in DATOS_DEL_EVENTO {
+                    if normalizado.contains(dato) {
+                        errores.push(format!("raiz: {dato} en defaults"));
+                    }
+                }
+            }
+        }
+    }
     let Some(jobs @ Nodo::Mapa(_)) = raiz.get("jobs") else {
         errores.push("sin jobs".to_owned());
         return errores;
@@ -388,19 +454,28 @@ fn lint_github(texto: &str) -> Vec<String> {
                 false
             }
         };
+        // Un job que llama a otro workflow reutilizable: fijado por SHA.
+        if let Some(uses) = job.get("uses").and_then(Nodo::texto) {
+            let version = uses.split_once('@').map_or("", |(_, v)| v);
+            if !es_sha(version) {
+                errores.push(format!(
+                    "job {nombre}: workflow reutilizable sin fijar por SHA: {uses}"
+                ));
+            }
+        }
         let mut valores = Vec::new();
         job.escalares("", &mut valores);
         for (clave, valor) in &valores {
             if *clave == "secrets" && *valor == "inherit" {
                 errores.push(format!("job {nombre}: secrets: inherit"));
             }
-            if !privilegio && valor.contains("secrets.") {
+            let normalizado = normalizar(valor);
+            if !privilegio && usa_secretos(&normalizado) {
                 errores.push(format!("job {nombre}: secreto en clase ejecuta"));
             }
             if *clave == "run" {
-                let minusculas = valor.to_ascii_lowercase();
                 for dato in DATOS_DEL_EVENTO {
-                    if minusculas.contains(dato) {
+                    if normalizado.contains(dato) {
                         errores.push(format!("job {nombre}: {dato} en run"));
                     }
                 }
@@ -454,20 +529,76 @@ const VARIABLES_DEL_AUTOR: [&str; 7] = [
 ];
 
 /// La cabecera `spec:` y el cuerpo de una plantilla con `spec:inputs`.
-fn partir_plantilla(texto: &str) -> (Nodo, Nodo) {
+fn partir_plantilla(texto: &str) -> Result<(Nodo, Nodo), String> {
     match texto.split_once("\n---") {
-        Some((cabecera, cuerpo)) => (leer_yaml(cabecera), leer_yaml(cuerpo)),
-        None => (Nodo::Mapa(Vec::new()), leer_yaml(texto)),
+        Some((cabecera, cuerpo)) => Ok((leer_yaml(cabecera)?, leer_yaml(cuerpo)?)),
+        None => Ok((Nodo::Mapa(Vec::new()), leer_yaml(texto)?)),
     }
 }
 
+/// Claves de primer nivel que no son jobs.
+const CLAVES_GLOBALES: [&str; 6] = [
+    "variables",
+    "default",
+    "include",
+    "stages",
+    "workflow",
+    "spec",
+];
+
+/// `$VAR` o `${VAR}` en la expresion, con el nombre completo (`$VAR_X` no es
+/// `$VAR`).
+fn lee_variable(expresion: &str, variable: &str) -> bool {
+    if expresion.contains(&format!("${{{variable}}}")) {
+        return true;
+    }
+    let buscado = format!("${variable}");
+    expresion.match_indices(&buscado).any(|(p, _)| {
+        !expresion[p + buscado.len()..]
+            .chars()
+            .next()
+            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
+    })
+}
+
 fn lint_gitlab(texto: &str) -> Vec<String> {
-    let (cabecera, cuerpo) = partir_plantilla(texto);
+    let (cabecera, cuerpo) = match partir_plantilla(texto) {
+        Ok(p) => p,
+        Err(e) => return vec![e],
+    };
     let mut errores = Vec::new();
     if cabecera.get("spec").and_then(|s| s.get("inputs")).is_none() {
         errores.push("sin spec:inputs".to_owned());
     }
-    for nombre in cuerpo.claves() {
+    let jobs: Vec<&str> = cuerpo
+        .claves()
+        .into_iter()
+        .filter(|k| !CLAVES_GLOBALES.contains(k))
+        .collect();
+    // `rules:if` se evalua al crear el pipeline, sin las `variables:` de los
+    // jobs (GitLab no las ve ahi): una regla que lea una variable de job no
+    // casa nunca. Las reglas usan variables globales o predefinidas.
+    let de_job: BTreeSet<&str> = jobs
+        .iter()
+        .filter_map(|j| cuerpo.get(j).and_then(|n| n.get("variables")))
+        .flat_map(Nodo::claves)
+        .collect();
+    for nombre in &jobs {
+        let mut condiciones = Vec::new();
+        if let Some(reglas) = cuerpo.get(nombre).and_then(|n| n.get("rules")) {
+            reglas.escalares("rules", &mut condiciones);
+        }
+        for (clave, condicion) in condiciones {
+            for variable in &de_job {
+                if clave == "if" && lee_variable(condicion, variable) {
+                    errores.push(format!(
+                        "job {nombre}: rules:if lee la variable de job {variable}"
+                    ));
+                }
+            }
+        }
+    }
+    for nombre in jobs {
         let Some(job) = cuerpo.get(nombre) else {
             continue;
         };
@@ -497,7 +628,18 @@ fn leer(relativa: &str) -> String {
     let ruta = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
         .join("..")
         .join(relativa);
-    std::fs::read_to_string(&ruta).unwrap_or_else(|e| panic!("{}: {e}", ruta.display()))
+    let texto =
+        std::fs::read_to_string(&ruta).unwrap_or_else(|e| panic!("{}: {e}", ruta.display()));
+    sin_crlf(&texto)
+}
+
+/// Fines de linea LF, con independencia de `core.autocrlf` en el clon.
+fn sin_crlf(texto: &str) -> String {
+    texto.replace("\r\n", "\n")
+}
+
+fn plantilla(texto: &str) -> (Nodo, Nodo) {
+    partir_plantilla(texto).unwrap_or_else(|e| panic!("{e}"))
 }
 
 const WORKFLOW: &str = ".github/workflows/verificacion.yml";
@@ -505,7 +647,7 @@ const PLANTILLA: &str = "plantillas/gitlab/verificacion.gitlab-ci.yml";
 
 #[test]
 fn el_lector_entiende_el_subconjunto() {
-    let yaml = "\
+    let doc = "\
 a: 1 # comentario
 b:
   - c: x
@@ -521,7 +663,7 @@ g: |
 h:
 - i
 ";
-    let raiz = leer_yaml(yaml);
+    let raiz = yaml(doc);
     assert_eq!(raiz.get("a"), Some(&Nodo::Escalar("1".into())));
     let Some(Nodo::Lista(b)) = raiz.get("b") else {
         panic!("b no es lista: {raiz:?}");
@@ -552,7 +694,7 @@ h:
 #[test]
 fn el_workflow_reutilizable_pasa_el_lint() {
     let texto = leer(WORKFLOW);
-    let raiz = leer_yaml(&texto);
+    let raiz = yaml(&texto);
     // El lector ve lo que hay que ver: tres jobs con sus pasos.
     let jobs = raiz.get("jobs").map(Nodo::claves).unwrap_or_default();
     assert_eq!(jobs, ["buscar", "ejecutar", "verificacion"]);
@@ -562,7 +704,7 @@ fn el_workflow_reutilizable_pasa_el_lint() {
 #[test]
 fn la_plantilla_de_gitlab_pasa_el_lint() {
     let texto = leer(PLANTILLA);
-    let (_, cuerpo) = partir_plantilla(&texto);
+    let (_, cuerpo) = plantilla(&texto);
     assert!(cuerpo.claves().contains(&"verificacion-completa"));
     assert_eq!(lint_gitlab(&texto), Vec::<String>::new());
 }
@@ -572,7 +714,7 @@ fn la_plantilla_de_gitlab_pasa_el_lint() {
 /// `stage` en GitLab).
 #[test]
 fn workflow_y_plantilla_tienen_las_mismas_entradas() {
-    let raiz = leer_yaml(&leer(WORKFLOW));
+    let raiz = yaml(&leer(WORKFLOW));
     let github: BTreeSet<&str> = raiz
         .get("on")
         .and_then(|o| o.get("workflow_call"))
@@ -583,7 +725,7 @@ fn workflow_y_plantilla_tienen_las_mismas_entradas() {
         .filter(|k| !k.starts_with("runner-") && *k != "shell")
         .collect();
     let texto = leer(PLANTILLA);
-    let (cabecera, _) = partir_plantilla(&texto);
+    let (cabecera, _) = plantilla(&texto);
     let gitlab: BTreeSet<&str> = cabecera
         .get("spec")
         .and_then(|s| s.get("inputs"))
@@ -734,3 +876,145 @@ fn la_plantilla_de_gitlab_rechaza_datos_del_autor_y_secretos() {
     );
     assert!(lint_gitlab("a:\n  script: [x]\n").contains(&"sin spec:inputs".to_owned()));
 }
+
+/// Un clon de Windows con `core.autocrlf=true` (si `.gitattributes` no
+/// aplica) da los `.yml` con CRLF: `leer` los deja en LF y las pruebas que
+/// buscan lineas literales siguen valiendo.
+#[test]
+fn leer_normaliza_crlf() {
+    let con_crlf = leer(PLANTILLA).replace('\n', "\r\n");
+    let texto = sin_crlf(&con_crlf);
+    assert!(!texto.contains('\r'));
+    assert!(texto.contains("    - $[[ inputs.comando ]] $[[ inputs.argumentos-completa ]]\n"));
+    assert_eq!(lint_gitlab(&texto), Vec::<String>::new());
+}
+
+/// Lo que el lector no entiende no se descarta en silencio: el lint falla.
+#[test]
+fn el_lector_falla_en_cerrado() {
+    let continuacion = BUENO.replacen(
+        "      - run: echo \"$TITULO\"\n",
+        "      - run: echo uno\n          dos\n  malo:\n    permissions: write-all\n    steps:\n      - run: echo ${{ github.event.issue.title }}\n",
+        1,
+    );
+    let sangria_impar = BUENO.replacen(
+        "      - run: echo \"$TITULO\"\n",
+        "      - run: echo a\n         b\n  otro:\n    permissions: write-all\n",
+        1,
+    );
+    let sin_dos_puntos = BUENO.replacen("permissions: {}\n", "permissions: {}\nsuelto\n", 1);
+    for texto in [continuacion, sangria_impar, sin_dos_puntos] {
+        assert_ne!(texto, BUENO);
+        let errores = lint_github(&texto);
+        assert!(
+            errores.iter().any(|e| e.contains("documento no entendido")),
+            "{errores:?}"
+        );
+    }
+}
+
+/// `env` y `defaults` de la raiz, corchetes y espacios en las expresiones.
+#[test]
+fn el_lint_ve_la_raiz_y_las_formas_alternativas() {
+    let casos: [(&str, &str, &str); 5] = [
+        (
+            "permissions: {}\n",
+            "permissions: {}\nenv:\n  CLAVE: ${{ secrets.TOKEN }}\n",
+            "raiz: secreto en env",
+        ),
+        (
+            "permissions: {}\n",
+            "permissions: {}\ndefaults:\n  run:\n    working-directory: ${{ github.head_ref }}\n",
+            "raiz: github.head_ref en defaults",
+        ),
+        (
+            "      - run: echo \"$TITULO\"",
+            "      - run: echo ${{ github['event']['pull_request']['title'] }}",
+            "github.event. en run",
+        ),
+        (
+            "      BASE: ${{ github.base_ref }}",
+            "      BASE: ${{ secrets['TOKEN'] }}",
+            "secreto en clase ejecuta",
+        ),
+        (
+            "      - run: echo \"$TITULO\"",
+            "      - run: echo '${{ toJSON( github) }}'",
+            "tojson(github en run",
+        ),
+    ];
+    for (original, mutado, esperado) in casos {
+        assert!(BUENO.contains(original), "el caso no casa: {original}");
+        let errores = lint_github(&BUENO.replacen(original, mutado, 1));
+        assert!(
+            errores.iter().any(|e| e.contains(esperado)),
+            "«{esperado}» no salio; errores: {errores:?}"
+        );
+    }
+}
+
+/// Un job que llama a otro workflow reutilizable lo fija por SHA.
+#[test]
+fn uses_a_nivel_de_job_exige_sha() {
+    let llamador = |version: &str| {
+        format!(
+            "on:\n  push:\npermissions: {{}}\njobs:\n  v:\n    permissions:\n      contents: read\n    uses: o/r/.github/workflows/x.yml@{version}\n"
+        )
+    };
+    assert_eq!(
+        lint_github(&llamador("0123456789abcdef0123456789abcdef01234567")),
+        Vec::<String>::new()
+    );
+    let errores = lint_github(&llamador("v1"));
+    assert!(
+        errores
+            .iter()
+            .any(|e| e.contains("job v: workflow reutilizable sin fijar por SHA")),
+        "{errores:?}"
+    );
+}
+
+/// El grupo de concurrencia incluye la PR o la ref: solo se encolan las
+/// ejecuciones del mismo PR, no las de otros.
+#[test]
+fn la_concurrencia_es_por_pr_o_ref() {
+    let raiz = yaml(&leer(WORKFLOW));
+    let grupo = raiz
+        .get("jobs")
+        .and_then(|j| j.get("ejecutar"))
+        .and_then(|e| e.get("concurrency"))
+        .and_then(|c| c.get("group"))
+        .and_then(Nodo::texto)
+        .unwrap_or_default();
+    assert!(
+        grupo.contains("github.event.pull_request.number || github.ref"),
+        "{grupo}"
+    );
+}
+
+/// `rules:if` no ve las `variables:` de los jobs: la plantilla no las usa en
+/// las reglas, y el lint lo detecta si alguien lo vuelve a hacer.
+#[test]
+fn las_reglas_de_gitlab_no_leen_variables_de_job() {
+    let texto = leer(PLANTILLA);
+    let (_, cuerpo) = plantilla(&texto);
+    assert!(cuerpo.claves().contains(&"variables"));
+    let mutado = texto.replacen(
+        "    TC2_PERFIL: rapida\n",
+        "    TC2_PERFIL: rapida\n    TC2_SOLO: rapida\n",
+        1,
+    );
+    let mutado = mutado.replacen(
+        "    - if: $TC2_PERFILES =~ / rapida /\n",
+        "    - if: $TC2_SOLO == \"rapida\"\n",
+        1,
+    );
+    assert_ne!(mutado, texto);
+    let errores = lint_gitlab(&mutado);
+    assert!(
+        errores
+            .iter()
+            .any(|e| e.contains("verificacion-rapida: rules:if lee la variable de job TC2_SOLO")),
+        "{errores:?}"
+    );
+}
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 648720a2f6f21211bc3b31ff5e612126c6c7ce61..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-041/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-041/planificacion/plan-final.md                 |   0
 tareas/03-en-revision/TASK-041/revision/informe-revision-1-code-quality-reviewer.md         |  21 +++
 tareas/03-en-revision/TASK-041/revision/informe-revision-1-rust-wasm-reviewer.md            |  21 +++
 tareas/03-en-revision/TASK-041/revision/peticion-revision-1-code-quality-reviewer.md        | 580 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-041/revision/peticion-revision-1-rust-wasm-reviewer.md           | 822 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-041/tarea.md                                    |   7 +-
 7 files changed, 1449 insertions(+), 2 deletions(-)
````
