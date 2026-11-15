# Peticion de revision — TASK-021 (ronda 2)

- Tarea: TASK-021 — F2-6 Adaptador GitHub
- Rama revisada: feature/task-021-f2-6-adaptador-github
- Rama base: develop
- Commit revisado (HEAD): 80b51da9ce8382316c6717f8bee44e45c9b1feae
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-021 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde aa704e87c860ed49942176ea8c711ba40a83e567 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (17 hallazgo(s)))

## Instrucciones para el agente revisor (verificacion acotada)

Eres un revisor INDEPENDIENTE del agente que implemento. La rama ya paso una
revision completa: esta ronda verifica las correcciones de la ronda anterior,
no vuelve a revisar la rama entera ni los MENORES de la ronda anterior (tu
tabla solo trae CRITICOS e IMPORTANTES). Haz esto, y nada mas:

1. Comprueba, uno por uno, que cada hallazgo pendiente de tu tabla de abajo
   queda resuelto por el diff de esta ronda (los marcados `corregido` tambien:
   verificalo, no lo des por bueno). Para un CRITICO o IMPORTANTE, reproduce el
   fallo original y comprueba que ya no ocurre; puedes leer el repo entero si la
   causa no esta en el diff.
2. Ejecuta la suite (o el comando de verificacion del proyecto) sobre el commit
   revisado y di el resultado: verificar sin ejecutar no vale. Si esta peticion
   ya trae la verificacion de la ronda en verde para ese mismo commit, no la
   repitas: cita ese resultado y corre solo los tests de lo que verificas.
3. Busca en el diff de esta ronda el fallo que la correccion abre: es justo
   donde se cuelan los fallos nuevos. Si encuentras uno, construye el caso que
   lo rompe antes de reportarlo.

Solo abre un hallazgo NUEVO si es CRITICO o si lo introduce la propia correccion
(algo que el diff de esta ronda rompe). Lo demas que veas fuera de eso va como
nota al final del informe, no en la tabla: no abre otra ronda.

Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. En la tabla, cada hallazgo pendiente que ya este
resuelto va como `corregido`; el que no, `abierto`. Vuelca tu salida en el
informe de esta ronda (informe-revision-2-rust-wasm-reviewer.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-021/tarea.md
- tareas/03-en-revision/TASK-021/planificacion/
- crates/tc2/src/plataforma/deteccion.rs
- crates/tc2/src/plataforma/errores.rs
- crates/tc2/src/plataforma/github.rs
- crates/tc2/src/plataforma/http.rs
- crates/tc2/src/plataforma/puertas.rs
- crates/tc2/tests/plataforma.rs
- crates/tc2/tests/plataforma_e2e.rs

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 5 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/tests/plataforma.rs:326 | informe-revision-1-rust-wasm-reviewer.md |
| IMP-2 | IMPORTANTE | corregido | crates/tc2/src/plataforma/deteccion.rs:81 | informe-revision-1-rust-wasm-reviewer.md |
| IMP-3 | IMPORTANTE | corregido | crates/tc2/src/plataforma/github.rs:514 | informe-revision-1-rust-wasm-reviewer.md |
| IMP-4 | IMPORTANTE | corregido | crates/tc2/src/plataforma/mod.rs:64 | informe-revision-1-rust-wasm-reviewer.md |

## Commits a revisar (git log aa704e87c860ed49942176ea8c711ba40a83e567..HEAD)

````
80b51da chore(TASK-021): constancia de autorrevision ronda 2
229caed fix(TASK-021): ronda 1, limpieza del E2E aunque falle a medias y evidencia repetida
ce6a6d1 fix(TASK-021): ronda 1 (limite de cuerpo, API acotada al host, reviews y checks como GitHub, tests de protecciones)
14aae54 chore(TASK-021): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff aa704e87c860ed49942176ea8c711ba40a83e567..HEAD)

````diff
diff --git a/crates/tc2/src/plataforma/deteccion.rs b/crates/tc2/src/plataforma/deteccion.rs
index ea4bf8c..000097e 100644
--- a/crates/tc2/src/plataforma/deteccion.rs
+++ b/crates/tc2/src/plataforma/deteccion.rs
@@ -78,12 +78,19 @@ pub fn detectar(url_origin: &str, clave: Option<(&str, &str)>) -> Result<Destino
                 d.nombre()
             )));
         }
-        (Some(h), Some((_, base))) => (h, base),
-        (Some(Proveedor::GitHub), None) => (Proveedor::GitHub, Base::github()),
-        (Some(Proveedor::GitLab), None) => (
-            Proveedor::GitLab,
-            Base::nueva("https://gitlab.com/api/v4").map_err(|m| rechazo(&m))?,
-        ),
+        // Con un host conocido, la API es la oficial: una clave que apunte a
+        // otra mandaria los tokens fuera (falla cerrado, decision 7).
+        (Some(h), Some((_, base))) => {
+            let oficial = api_oficial(h).map_err(|m| rechazo(&m))?;
+            if base != oficial {
+                return Err(rechazo(&format!(
+                    "origin es de {host} y su API es {}: la clave de la configuracion no puede cambiarla",
+                    oficial.texto()
+                )));
+            }
+            (h, base)
+        }
+        (Some(h), None) => (h, api_oficial(h).map_err(|m| rechazo(&m))?),
         (None, Some((d, base))) => (d, base),
         (None, None) => {
             return Err(rechazo(&format!(
@@ -102,6 +109,14 @@ pub fn detectar(url_origin: &str, clave: Option<(&str, &str)>) -> Result<Destino
     })
 }
 
+/// API publica de un host conocido.
+fn api_oficial(p: Proveedor) -> Result<Base, String> {
+    match p {
+        Proveedor::GitHub => Ok(Base::github()),
+        Proveedor::GitLab => Base::nueva("https://gitlab.com/api/v4"),
+    }
+}
+
 fn rechazo(m: &str) -> Informe {
     Informe::nuevo(OP, Resultado::Rechazada)
         .con("motivo", "plataforma")
@@ -193,6 +208,14 @@ mod tests {
                 Some(("bitbucket", "https://x")),
             ),
             ("https://git.empresa.com/o/r", Some(("github", "http://x"))),
+            (
+                "https://github.com/o/r",
+                Some(("github", "https://api.evil.example")),
+            ),
+            (
+                "git@gitlab.com:g/p.git",
+                Some(("gitlab", "https://evil.example/api/v4")),
+            ),
             ("https://github.com/o/r/s", None),
             ("https://github.com/o", None),
             ("file:///c/repo", None),
diff --git a/crates/tc2/src/plataforma/errores.rs b/crates/tc2/src/plataforma/errores.rs
index d4af11f..d4f21dd 100644
--- a/crates/tc2/src/plataforma/errores.rs
+++ b/crates/tc2/src/plataforma/errores.rs
@@ -40,8 +40,13 @@ pub fn es_aviso_de_plan(r: &Respuesta) -> bool {
 
 /// Si la respuesta es un limite de peticiones agotado.
 fn es_limite(r: &Respuesta) -> bool {
+    // El limite secundario de GitHub puede llegar sin cabeceras: entonces lo
+    // dice el mensaje.
     (r.estado == 403 || r.estado == 429)
-        && (r.restantes.as_deref() == Some("0") || r.reintentar.is_some())
+        && (r.restantes.as_deref() == Some("0")
+            || r.reintentar.is_some()
+            || r.estado == 429
+            || mensaje_api(r).to_ascii_lowercase().contains("rate limit"))
 }
 
 /// Segundos de espera que da la API, si los da.
diff --git a/crates/tc2/src/plataforma/github.rs b/crates/tc2/src/plataforma/github.rs
index b74f845..9569247 100644
--- a/crates/tc2/src/plataforma/github.rs
+++ b/crates/tc2/src/plataforma/github.rs
@@ -149,7 +149,7 @@ struct ReviewApi {
 struct MergeApi {
     merged: bool,
     #[serde(default)]
-    sha: String,
+    sha: Option<String>,
 }
 
 #[derive(Deserialize)]
@@ -515,7 +515,12 @@ impl Plataforma for GitHub {
                     .map_or(EstadoCheck::Pendiente, |r| {
                         if r.status != "completed" {
                             EstadoCheck::Pendiente
-                        } else if r.conclusion.as_deref() == Some("success") {
+                        } else if matches!(
+                            r.conclusion.as_deref(),
+                            Some("success" | "neutral" | "skipped")
+                        ) {
+                            // GitHub da por cumplido un check obligatorio
+                            // que acaba neutral o saltado.
                             EstadoCheck::Ok
                         } else {
                             EstadoCheck::Fallo
@@ -530,10 +535,16 @@ impl Plataforma for GitHub {
                 Err(i) => return i,
             };
         // Las reviews vienen de la mas vieja a la mas nueva: cuenta la ultima
-        // de cada cuenta, por id (F1-6, regla 6).
+        // de cada cuenta, por id (F1-6, regla 6). Solo cambian el estado las
+        // que deciden: un comentario o una review pendiente no retiran una
+        // aprobacion anterior (como en GitHub).
         let mut ultima: BTreeMap<u64, (String, Usuario)> = BTreeMap::new();
         for rv in reviews {
-            if let Some(u) = rv.user {
+            let decide = matches!(
+                rv.state.as_str(),
+                "APPROVED" | "CHANGES_REQUESTED" | "DISMISSED"
+            );
+            if let Some(u) = rv.user.filter(|_| decide) {
                 ultima.insert(u.id, (rv.state, u.into()));
             }
         }
@@ -552,6 +563,9 @@ impl Plataforma for GitHub {
             aprobaciones,
             ignoradas: aprobadas.len() - aprobaciones,
             al_dia: p.mergeable_state.as_deref() != Some("behind"),
+            // `unknown` (GitHub aun calculando) no se da por conflicto:
+            // `merge-pr` lo vuelve a comprobar en la plataforma (405).
+            sin_conflictos: p.mergeable_state.as_deref() != Some("dirty"),
         })
     }
 
@@ -600,10 +614,16 @@ impl Plataforma for GitHub {
             return i;
         }
         match parsear::<MergeApi>(MERGE_PR, &r.cuerpo) {
-            Ok(m) if m.merged && sha_valido(&m.sha) => Informe::nuevo(MERGE_PR, Resultado::Ok)
+            Ok(MergeApi {
+                merged: true,
+                sha: Some(s),
+            }) if sha_valido(&s) => Informe::nuevo(MERGE_PR, Resultado::Ok)
                 .con("pr", pr.to_string())
-                .con("commit", corto(&m.sha)),
-            Ok(_) => errores::respuesta_invalida(MERGE_PR, "merged o sha"),
+                .con("commit", corto(&s)),
+            // Mergeado aunque la respuesta no traiga un sha legible: se dice
+            // que esta hecho, sin inventar el commit.
+            Ok(m) if m.merged => Informe::nuevo(MERGE_PR, Resultado::Ok).con("pr", pr.to_string()),
+            Ok(_) => errores::respuesta_invalida(MERGE_PR, "merged"),
             Err(i) => i,
         }
     }
diff --git a/crates/tc2/src/plataforma/http.rs b/crates/tc2/src/plataforma/http.rs
index 510ee31..db63d5c 100644
--- a/crates/tc2/src/plataforma/http.rs
+++ b/crates/tc2/src/plataforma/http.rs
@@ -71,22 +71,30 @@ impl Base {
         let resto = if let Some(r) = url.strip_prefix("https://") {
             r
         } else if let Some(r) = url.strip_prefix("http://127.0.0.1:") {
-            if r.is_empty() || !r.bytes().all(|b| b.is_ascii_digit()) {
+            if !puerto_valido(r) {
                 return Err("http solo vale para 127.0.0.1 con un puerto y sin ruta".into());
             }
             return Ok(Self(url.to_owned()));
         } else {
             return Err("la URL de la API tiene que ser https".into());
         };
-        let (host, ruta) = resto.split_once('/').unwrap_or((resto, ""));
+        let (autoridad, ruta) = resto.split_once('/').unwrap_or((resto, ""));
+        let (host, puerto) = match autoridad.split_once(':') {
+            Some((h, p)) => (h, Some(p)),
+            None => (autoridad, None),
+        };
         let host_valido = !host.is_empty()
             && host
                 .bytes()
-                .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-' || b == b':')
-            && !host.starts_with(['.', '-', ':']);
+                .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
+            && !host.starts_with(['.', '-'])
+            && puerto.is_none_or(puerto_valido);
         if !host_valido {
             return Err("host de la API no valido (sin usuario ni contrasena)".into());
         }
+        if ruta.contains('?') || ruta.contains('#') {
+            return Err("el prefijo de la API no lleva consulta ni fragmento".into());
+        }
         if !ruta.is_empty() && validar_ruta(&format!("/{ruta}")).is_err() {
             return Err("prefijo de la API no valido".into());
         }
@@ -99,6 +107,14 @@ impl Base {
     }
 }
 
+/// Puerto decimal entre 1 y 65535.
+fn puerto_valido(p: &str) -> bool {
+    !p.is_empty()
+        && p.len() <= 5
+        && p.bytes().all(|b| b.is_ascii_digit())
+        && p.parse::<u32>().is_ok_and(|n| (1..=65535).contains(&n))
+}
+
 /// Respuesta ya leida: estado, cuerpo y las cabeceras que se usan.
 #[derive(Debug, Clone, PartialEq, Eq)]
 pub struct Respuesta {
@@ -246,6 +262,16 @@ impl Cliente {
         let restantes = cabecera("x-ratelimit-remaining");
         let reintentar = cabecera("retry-after");
         let reinicio = cabecera("x-ratelimit-reset");
+        // Si la respuesta declara su tamano y pasa del limite, no se lee:
+        // cortar la lectura a medias deja la conexion en un estado que el
+        // sistema puede notificar como desconexion (y se veria `sinred`).
+        let declarado = cabecera("content-length").and_then(|v| v.trim().parse::<u64>().ok());
+        if declarado.is_some_and(|n| n > LIMITE_CUERPO) {
+            return Err(FalloHttp::RespuestaInvalida(format!(
+                "la respuesta declara mas de {LIMITE_CUERPO} bytes"
+            )));
+        }
+        // Sin tamano declarado (chunked), el limite corta al leer.
         let cuerpo = r
             .body_mut()
             .with_config()
@@ -389,6 +415,11 @@ mod tests {
             "https://",
             "ftp://x",
             "https://host/../x",
+            "https://host/api?x=1",
+            "https://host:0",
+            "https://host:99999",
+            "https://host:8a",
+            "http://127.0.0.1:70000",
         ] {
             assert!(Base::nueva(b).is_err(), "{b}");
         }
diff --git a/crates/tc2/src/plataforma/puertas.rs b/crates/tc2/src/plataforma/puertas.rs
index 8eba041..2ac31f3 100644
--- a/crates/tc2/src/plataforma/puertas.rs
+++ b/crates/tc2/src/plataforma/puertas.rs
@@ -31,6 +31,8 @@ pub struct Lecturas {
     pub ignoradas: usize,
     /// Si la rama esta al dia con la base.
     pub al_dia: bool,
+    /// Si la rama se puede mergear sin conflictos.
+    pub sin_conflictos: bool,
 }
 
 /// `EstadoPuertas` neutral: rojo si un check fallo; esperando si falta algo;
@@ -64,6 +66,9 @@ pub fn evaluar(l: &Lecturas) -> Informe {
     if !l.al_dia {
         falta.push("al-dia".into());
     }
+    if !l.sin_conflictos {
+        falta.push("sin-conflictos".into());
+    }
     let resultado = if falta.is_empty() {
         Resultado::Ok
     } else {
@@ -77,7 +82,7 @@ pub fn evaluar(l: &Lecturas) -> Informe {
         i
     } else {
         let diag = format!(
-            "faltan puertas: {}; espera o atiende lo que falta (al-dia: lanza actualizar-rama)",
+            "faltan puertas: {}; espera o atiende lo que falta (al-dia o sin-conflictos: lanza actualizar-rama)",
             falta.join(", ")
         );
         i.con_lista("falta", falta).diagnostico(diag)
@@ -113,6 +118,7 @@ mod tests {
             aprobaciones: aprob,
             ignoradas: 0,
             al_dia,
+            sin_conflictos: al_dia,
         }
     }
 
@@ -130,7 +136,7 @@ mod tests {
         let i = evaluar(&l(&[("c", EstadoCheck::Pendiente)], 0, false));
         assert_eq!(
             i.linea(),
-            "estado-puertas esperando pr=7 aprobaciones=0 ignoradas=0 falta=c,aprobacion,al-dia"
+            "estado-puertas esperando pr=7 aprobaciones=0 ignoradas=0 falta=c,aprobacion,al-dia,sin-conflictos"
         );
         assert_eq!(i.resultado.codigo(), 7);
         let i = evaluar(&l(&[("c", EstadoCheck::Ok)], 1, true));
````

## Excluido del diff (8 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff aa704e87c860ed49942176ea8c711ba40a83e567..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-021/evidencia/e2e.log                           |   29 +-
 tareas/{02-en-curso => 03-en-revision}/TASK-021/planificacion/brainstorm/peticion-plan-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-021/planificacion/plan-final.md                 |    0
 tareas/03-en-revision/TASK-021/revision/informe-revision-1-code-quality-reviewer.md         |   21 +
 tareas/03-en-revision/TASK-021/revision/informe-revision-1-rust-wasm-reviewer.md            |   21 +
 tareas/03-en-revision/TASK-021/revision/peticion-revision-1-code-quality-reviewer.md        |   98 +++++
 tareas/03-en-revision/TASK-021/revision/peticion-revision-1-rust-wasm-reviewer.md           | 2142 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-021/tarea.md                                    |   12 +-
 8 files changed, 2307 insertions(+), 16 deletions(-)
````

## Tests del diff (2 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff aa704e87c860ed49942176ea8c711ba40a83e567..HEAD -- ":(literal)crates/tc2/tests/plataforma.rs" ":(literal)crates/tc2/tests/plataforma_e2e.rs"
````

````
crates/tc2/tests/plataforma.rs     | 333 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/plataforma_e2e.rs |  81 ++++++++++++++++++++++++++++++++------
 2 files changed, 402 insertions(+), 12 deletions(-)
````
