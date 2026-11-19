# Peticion de revision — TASK-047 (ronda 2)

- Tarea: TASK-047 — F6-1 Identidades: agente implementador
- Rama revisada: feature/task-047-f6-1-identidades-agente-implementador
- Rama base: develop
- Commit revisado (HEAD): 5f3521117b2005d83286d3bdf9cbd6d0c21ec125
- Fecha: 2026-11-19
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-047 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 8123fa68e74571def0591f22da57cfa2536e687c (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (11 hallazgo(s)))

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

- tareas/03-en-revision/TASK-047/tarea.md
- tareas/03-en-revision/TASK-047/planificacion/
- crates/tc2/src/git/puerto.rs
- crates/tc2/src/i18n/catalogo/plataforma.rs
- crates/tc2/src/plataforma/app_github.rs
- crates/tc2/src/plataforma/conexion.rs
- crates/tc2/src/plataforma/credenciales.rs
- crates/tc2/src/plataforma/entorno_git.rs
- crates/tc2/src/plataforma/github.rs
- crates/tc2/tests/agente_git/mod.rs
- crates/tc2/tests/identidades.rs
- crates/tc2/tests/plataforma_e2e.rs
- docs/adr/F1-6-identidades-agente-revisor-persona.md

## Hallazgos de la ronda 1 pendientes de verificar

Tus 5 hallazgo(s) MENOR de la ronda 1 no se reverifican (los comprueba
quien coordina): recibes esta ronda porque la correccion
toca ficheros de tu dominio. Revisa en el diff de esta ronda lo que la correccion
cambia en tu terreno y si abre un fallo.

## Commits a revisar (git log 8123fa68e74571def0591f22da57cfa2536e687c..HEAD)

````
5f35211 fix(TASK-047): hallazgos de la ronda 1 de revision
ebb6ad8 chore(TASK-047): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 8123fa68e74571def0591f22da57cfa2536e687c..HEAD)

````diff
diff --git a/crates/tc2/src/git/puerto.rs b/crates/tc2/src/git/puerto.rs
index 3972606..424211a 100644
--- a/crates/tc2/src/git/puerto.rs
+++ b/crates/tc2/src/git/puerto.rs
@@ -216,6 +216,10 @@ const ENTORNO_FIJO: [(&str, &str); 6] = [
 pub struct Git {
     binario: OsString,
     entorno_extra: Vec<(OsString, OsString)>,
+    /// Entradas de config de ambito «command» (`GIT_CONFIG_KEY_<n>` y
+    /// `GIT_CONFIG_VALUE_<n>`) como pares `(clave, valor)`: se numeran al
+    /// lanzar, a continuacion de las que ya traiga el proceso.
+    config: Vec<(OsString, OsString)>,
     registro: RefCell<Vec<String>>,
 }
 
@@ -231,6 +235,7 @@ impl Git {
         Self {
             binario: OsString::from("git"),
             entorno_extra: Vec::new(),
+            config: Vec::new(),
             registro: RefCell::new(Vec::new()),
         }
     }
@@ -246,20 +251,26 @@ impl Git {
     }
 
     /// Anade variables de entorno a cada llamada (los tests aislan asi la
-    /// configuracion del usuario sin tocar el entorno del proceso).
+    /// configuracion del usuario sin tocar el entorno del proceso). Igual
+    /// que [`Git::con_entorno_extra`]: acumula, no sustituye, asi que el
+    /// orden de las dos no importa.
     #[must_use]
-    pub fn con_entorno(mut self, vars: Vec<(OsString, OsString)>) -> Self {
-        self.entorno_extra = vars;
-        self
+    pub fn con_entorno(self, vars: Vec<(OsString, OsString)>) -> Self {
+        self.con_entorno_extra(vars)
     }
 
     /// Anade variables a las que ya tenga el puerto, sin quitarlas: el
     /// entorno del agente de F6-1 (`plataforma::entorno_git`: cabecera de
     /// autenticacion por `GIT_CONFIG_*` y autor), que asi nunca pasa por la
-    /// linea de ordenes ni por el registro.
+    /// linea de ordenes ni por el registro. Las entradas `GIT_CONFIG_COUNT`,
+    /// `GIT_CONFIG_KEY_<n>` y `GIT_CONFIG_VALUE_<n>` se acumulan: cada lote
+    /// se numera a continuacion de los anteriores y de las que ya traiga el
+    /// proceso, sin pisar ninguna.
     #[must_use]
     pub fn con_entorno_extra(mut self, vars: Vec<(OsString, OsString)>) -> Self {
-        self.entorno_extra.extend(vars);
+        let (resto, config) = separar_config(vars);
+        self.entorno_extra.extend(resto);
+        self.config.extend(config);
         self
     }
 
@@ -468,6 +479,15 @@ impl Git {
         for (k, v) in &self.entorno_extra {
             cmd.env(k, v);
         }
+        if !self.config.is_empty() {
+            let previas = std::env::var("GIT_CONFIG_COUNT")
+                .ok()
+                .and_then(|n| n.trim().parse::<usize>().ok())
+                .unwrap_or(0);
+            for (k, v) in numerar_config(previas, &self.config) {
+                cmd.env(k, v);
+            }
+        }
         if let Some(i) = indice {
             cmd.env("GIT_INDEX_FILE", i);
         }
@@ -601,6 +621,58 @@ fn es_prohibido(sub: Subcomando, a: &str) -> bool {
     }
 }
 
+/// Variable de entorno (o entrada de config) `(nombre, valor)`.
+type Par = (OsString, OsString);
+
+/// Separa de `vars` las entradas de config de ambito «command»: devuelve las
+/// demas variables y los pares `(clave, valor)` de `GIT_CONFIG_KEY_<n>` y
+/// `GIT_CONFIG_VALUE_<n>` para `n` menor que `GIT_CONFIG_COUNT`, en orden
+/// (las que git no leeria, por encima de la cuenta o sin pareja, se
+/// descartan).
+fn separar_config(vars: Vec<Par>) -> (Vec<Par>, Vec<Par>) {
+    let mut resto = Vec::new();
+    let mut cuenta = 0usize;
+    let mut claves = std::collections::BTreeMap::new();
+    let mut valores = std::collections::BTreeMap::new();
+    for (k, v) in vars {
+        let nombre = k.to_str().unwrap_or_default();
+        if nombre == "GIT_CONFIG_COUNT" {
+            cuenta = v.to_str().and_then(|n| n.trim().parse().ok()).unwrap_or(0);
+        } else if let Some(n) = nombre
+            .strip_prefix("GIT_CONFIG_KEY_")
+            .and_then(|n| n.parse::<usize>().ok())
+        {
+            claves.insert(n, v);
+        } else if let Some(n) = nombre
+            .strip_prefix("GIT_CONFIG_VALUE_")
+            .and_then(|n| n.parse::<usize>().ok())
+        {
+            valores.insert(n, v);
+        } else {
+            resto.push((k, v));
+        }
+    }
+    let config = (0..cuenta)
+        .filter_map(|n| Some((claves.remove(&n)?, valores.remove(&n)?)))
+        .collect();
+    (resto, config)
+}
+
+/// Variables `GIT_CONFIG_*` de `config` numeradas tras las `previas` que ya
+/// trae el proceso, con la cuenta total.
+fn numerar_config(previas: usize, config: &[(OsString, OsString)]) -> Vec<(OsString, OsString)> {
+    let mut vars = vec![(
+        OsString::from("GIT_CONFIG_COUNT"),
+        OsString::from((previas + config.len()).to_string()),
+    )];
+    for (i, (k, v)) in config.iter().enumerate() {
+        let n = previas + i;
+        vars.push((OsString::from(format!("GIT_CONFIG_KEY_{n}")), k.clone()));
+        vars.push((OsString::from(format!("GIT_CONFIG_VALUE_{n}")), v.clone()));
+    }
+    vars
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
@@ -611,10 +683,55 @@ mod tests {
         Git {
             binario: OsString::from("no-existe-este-git"),
             entorno_extra: Vec::new(),
+            config: Vec::new(),
             registro: RefCell::new(Vec::new()),
         }
     }
 
+    fn par(k: &str, v: &str) -> (OsString, OsString) {
+        (OsString::from(k), OsString::from(v))
+    }
+
+    /// MEN-4 / MENOR-5 de la ronda 1 de TASK-047: los lotes de
+    /// `GIT_CONFIG_*` se acumulan en cualquier orden y se numeran tras los
+    /// que ya trae el proceso, sin pisar ninguno.
+    #[test]
+    fn las_entradas_de_config_se_acumulan_y_se_numeran_tras_las_previas() {
+        let g = Git::nuevo()
+            .con_entorno_extra(vec![
+                par("GIT_CONFIG_COUNT", "1"),
+                par("GIT_CONFIG_KEY_0", "a.b"),
+                par("GIT_CONFIG_VALUE_0", "1"),
+                par("OTRA", "x"),
+            ])
+            .con_entorno(vec![
+                par("GIT_CONFIG_COUNT", "2"),
+                par("GIT_CONFIG_KEY_0", "c.d"),
+                par("GIT_CONFIG_VALUE_0", "2"),
+                par("GIT_CONFIG_KEY_1", "e.f"),
+                par("GIT_CONFIG_VALUE_1", "3"),
+                // Fuera de la cuenta: git no la leeria.
+                par("GIT_CONFIG_KEY_5", "no.vale"),
+            ]);
+        assert_eq!(g.entorno_extra, vec![par("OTRA", "x")]);
+        assert_eq!(
+            g.config,
+            vec![par("a.b", "1"), par("c.d", "2"), par("e.f", "3")]
+        );
+        assert_eq!(
+            numerar_config(2, &g.config),
+            vec![
+                par("GIT_CONFIG_COUNT", "5"),
+                par("GIT_CONFIG_KEY_2", "a.b"),
+                par("GIT_CONFIG_VALUE_2", "1"),
+                par("GIT_CONFIG_KEY_3", "c.d"),
+                par("GIT_CONFIG_VALUE_3", "2"),
+                par("GIT_CONFIG_KEY_4", "e.f"),
+                par("GIT_CONFIG_VALUE_4", "3"),
+            ]
+        );
+    }
+
     #[test]
     fn rechaza_cada_argumento_prohibido_sin_lanzar_git() {
         let git = git_sin_binario();
diff --git a/crates/tc2/src/i18n/catalogo/plataforma.rs b/crates/tc2/src/i18n/catalogo/plataforma.rs
index 6c32039..b839002 100644
--- a/crates/tc2/src/i18n/catalogo/plataforma.rs
+++ b/crates/tc2/src/i18n/catalogo/plataforma.rs
@@ -106,6 +106,14 @@ crate::i18n::textos! {
         es: "la GitHub App de {rol} no consiguio su token de instalacion ({causa}: {detalle}): revisa {variables}, que la App este instalada en el repo y que tenga los permisos del rol",
         en: "the {rol} GitHub App did not get its installation token ({causa}: {detalle}): check {variables}, that the App is installed on the repo and that it has the role permissions",
     }
+    APP_SOLO_GITHUB {
+        es: "en GitLab la GitHub App de {rol} no sirve ({variables} solo valen en GitHub): define {variable} con un token de proyecto o de cuenta de servicio de {rol}",
+        en: "on GitLab the {rol} GitHub App does not apply ({variables} only work on GitHub): set {variable} to a project or service account token for {rol}",
+    }
+    APP_TLS {
+        es: "la GitHub App de {rol} no consiguio su token por un fallo de TLS ({detalle}): si la red usa un proxy con su propia CA, instalala en el almacen de certificados del sistema; revisa tambien la hora del equipo",
+        en: "the {rol} GitHub App did not get its token because of a TLS failure ({detalle}): if the network uses a proxy with its own CA, install it in the system certificate store; also check the machine clock",
+    }
     SOLO_TERMINAL {
         es: " (el de persona solo se usa desde un terminal interactivo)",
         en: " (the persona one is only used from an interactive terminal)",
diff --git a/crates/tc2/src/plataforma/app_github.rs b/crates/tc2/src/plataforma/app_github.rs
index 7c803e7..537255f 100644
--- a/crates/tc2/src/plataforma/app_github.rs
+++ b/crates/tc2/src/plataforma/app_github.rs
@@ -26,6 +26,7 @@ use serde::Deserialize;
 
 use super::credenciales::{Credenciales, FalloCredencial, Rol, Secreto, causas, id_valido};
 use super::errores::sanear;
+use super::github::repo_valido;
 use super::http::{Base, Cliente, FalloHttp, Metodo, Respuesta};
 
 /// Antes de `expires_at` se deja de usar un token y se pide otro (los
@@ -69,6 +70,10 @@ struct InstalacionApi {
 struct TokenApi {
     token: String,
     expires_at: String,
+    /// Permisos que concede de verdad (GitHub los devuelve; si faltan en la
+    /// respuesta no se puede comprobar y se acepta).
+    #[serde(default)]
+    permissions: Option<std::collections::BTreeMap<String, String>>,
 }
 
 #[derive(Deserialize)]
@@ -206,16 +211,7 @@ impl CredencialesApp {
     ) -> Result<Respuesta, FalloCredencial> {
         self.cliente
             .llamar(metodo, ruta, Some(jwt), cuerpo)
-            .map_err(|f| match f {
-                FalloHttp::Sinred(m) => FalloCredencial::Acunar {
-                    causa: causas::SINRED,
-                    detalle: sanear(&m),
-                },
-                otro => FalloCredencial::Acunar {
-                    causa: causas::RESPUESTA_INESPERADA,
-                    detalle: sanear(&format!("{otro:?}")),
-                },
-            })
+            .map_err(|f| fallo_transporte(&f))
     }
 
     /// Id de la instalacion de la App en el repo (una vez por proceso).
@@ -270,6 +266,14 @@ impl CredencialesApp {
                 detalle: "token".to_owned(),
             });
         }
+        if let Some(concedidos) = &t.permissions
+            && let Some(falta) = permiso_que_falta(self.rol, concedidos)
+        {
+            return Err(FalloCredencial::Acunar {
+                causa: causas::PERMISOS_NO_CONCEDIDOS,
+                detalle: falta,
+            });
+        }
         Ok((Secreto::nuevo(t.token), expira))
     }
 }
@@ -306,17 +310,51 @@ fn ahora() -> u64 {
         .map_or(0, |d| d.as_secs())
 }
 
-/// `dueno/repo` con la misma forma que exige el adaptador.
-fn repo_valido(repo: &str) -> bool {
-    let partes: Vec<&str> = repo.split('/').collect();
-    partes.len() == 2
-        && partes.iter().all(|p| {
-            !p.is_empty()
-                && *p != "."
-                && *p != ".."
-                && p.bytes()
-                    .all(|b| b.is_ascii_alphanumeric() || b"-._".contains(&b))
+/// Primer permiso del rol que el token no concede con el nivel pedido
+/// (`write` cubre `read`), como `permiso:nivel`.
+fn permiso_que_falta(
+    rol: Rol,
+    concedidos: &std::collections::BTreeMap<String, String>,
+) -> Option<String> {
+    permisos(rol)
+        .unwrap_or_default()
+        .iter()
+        .find(|(k, v)| {
+            let dado = concedidos.get(*k).map(String::as_str);
+            !(dado == Some(*v) || (*v == "read" && dado == Some("write")))
         })
+        .map(|(k, v)| format!("{k}:{v}"))
+}
+
+/// Fallo del transporte al pedir el token: sin red, `sinred`; TLS, `tls`
+/// (el diagnostico dice que revisar); el resto, `respuesta-inesperada` con
+/// un detalle legible (nunca el `Debug` de la variante).
+fn fallo_transporte(f: &FalloHttp) -> FalloCredencial {
+    let (causa, detalle) = match f {
+        FalloHttp::Sinred(m) => (causas::SINRED, sanear(m)),
+        FalloHttp::Tls(m) => (causas::TLS, sanear(m)),
+        FalloHttp::RespuestaInvalida(m) => (
+            causas::RESPUESTA_INESPERADA,
+            sanear(&format!("respuesta ilegible: {m}")),
+        ),
+        FalloHttp::CuerpoGrande => (
+            causas::RESPUESTA_INESPERADA,
+            "respuesta demasiado grande".to_owned(),
+        ),
+        FalloHttp::Redireccion(e) => (
+            causas::RESPUESTA_INESPERADA,
+            format!("redireccion {e}: revisa la URL de la API"),
+        ),
+        FalloHttp::Rechazada(_) => (
+            causas::RESPUESTA_INESPERADA,
+            "ruta rechazada antes de la red".to_owned(),
+        ),
+        FalloHttp::Otro(m) => (
+            causas::RESPUESTA_INESPERADA,
+            sanear(&format!("cliente HTTP: {m}")),
+        ),
+    };
+    FalloCredencial::Acunar { causa, detalle }
 }
 
 /// Fallo por el estado HTTP: 401 es el JWT, 404 la instalacion, 422 los
@@ -351,10 +389,18 @@ fn leer_pem(pem: &str) -> Result<RsaKeyPair, FalloCredencial> {
         causa: causas::CLAVE_ILEGIBLE,
         detalle: detalle.to_owned(),
     };
-    let mut lineas = pem.lines().map(str::trim).skip_while(|l| l.is_empty());
+    // Un BOM UTF-8 (`Set-Content -Encoding utf8` de Windows PowerShell 5.1)
+    // y el texto antes de `-----BEGIN` (comentarios, como admite RFC 7468)
+    // se ignoran.
+    let mut lineas = pem
+        .trim_start_matches('\u{feff}')
+        .lines()
+        .map(str::trim)
+        .skip_while(|l| !l.starts_with("-----BEGIN "));
     let etiqueta = lineas
         .next()
-        .and_then(|l| l.strip_prefix("-----BEGIN "))
+        .ok_or_else(|| ilegible("pem sin -----BEGIN"))?
+        .strip_prefix("-----BEGIN ")
         .and_then(|l| l.strip_suffix("-----"))
         .ok_or_else(|| ilegible("pem"))?
         .to_owned();
diff --git a/crates/tc2/src/plataforma/conexion.rs b/crates/tc2/src/plataforma/conexion.rs
index 20cbc17..de00753 100644
--- a/crates/tc2/src/plataforma/conexion.rs
+++ b/crates/tc2/src/plataforma/conexion.rs
@@ -95,8 +95,17 @@ impl Conexion {
                     .variables_app()
                     .map(|(a, b)| format!("{a}, {b}"))
                     .unwrap_or_default();
-                let m = Mensaje::nuevo(&t::APP_SIN_TOKEN)
+                // Cada causa con su accion: fuera de GitHub hace falta el
+                // token del rol; TLS es del almacen de certificados; el
+                // resto, de la App.
+                let texto = match causa {
+                    causas::APP_SOLO_GITHUB => &t::APP_SOLO_GITHUB,
+                    causas::TLS => &t::APP_TLS,
+                    _ => &t::APP_SIN_TOKEN,
+                };
+                let m = Mensaje::nuevo(texto)
                     .con("rol", rol.nombre())
+                    .con("variable", rol.variable())
                     .con("causa", causa)
                     .con("detalle", errores::sanear(&detalle))
                     .con("variables", variables);
diff --git a/crates/tc2/src/plataforma/credenciales.rs b/crates/tc2/src/plataforma/credenciales.rs
index 3c27dc0..3c439fa 100644
--- a/crates/tc2/src/plataforma/credenciales.rs
+++ b/crates/tc2/src/plataforma/credenciales.rs
@@ -99,8 +99,8 @@ pub enum FalloCredencial {
 pub mod causas {
     /// La App esta a medias: id sin clave o al reves, o id no valido.
     pub const APP_INCOMPLETA: &str = "app-incompleta";
-    /// La App solo existe en GitHub (en GitLab va un token de proyecto o de
-    /// cuenta de servicio en `TC2_TOKEN_<ROL>`).
+    /// La App solo existe en GitHub: fuera de GitHub se usa el token de
+    /// `TC2_TOKEN_<ROL>` del mismo rol y esta causa sale si no lo hay.
     pub const APP_SOLO_GITHUB: &str = "app-solo-github";
     /// No se pudo leer el PEM o no es una clave RSA que se entienda.
     pub const CLAVE_ILEGIBLE: &str = "clave-ilegible";
@@ -112,6 +112,9 @@ pub mod causas {
     pub const PERMISOS_NO_CONCEDIDOS: &str = "permisos-no-concedidos";
     /// Sin red al pedir el token.
     pub const SINRED: &str = "sinred";
+    /// Fallo de TLS al pedir el token (certificado, almacen del sistema,
+    /// proxy con su propia CA).
+    pub const TLS: &str = "tls";
     /// Respuesta con otro estado o con una forma inesperada.
     pub const RESPUESTA_INESPERADA: &str = "respuesta-inesperada";
 }
@@ -147,7 +150,8 @@ struct ConfigApp {
 }
 
 /// Lo que da el entorno para un rol con App: la App ya atada a un repo, la
-/// configuracion aun sin atar, o la App a medias.
+/// configuracion aun sin atar (destino que no es GitHub: se usa el token del
+/// mismo rol si lo hay), o la App a medias.
 enum EstadoApp {
     SinAtar(ConfigApp),
     Atada(Box<CredencialesApp>),
@@ -193,6 +197,8 @@ pub fn de_entorno_con(leer: impl Fn(&str) -> Option<String>, tty: bool) -> Crede
             match (valor(vid), valor(vclave)) {
                 (None, None) => {}
                 (Some(id), Some(clave)) if id_valido(id.trim()) => {
+                    // Sin `continue`: el token del mismo rol se guarda para
+                    // un destino que no es GitHub (la App solo vale ahi).
                     apps.insert(
                         rol,
                         EstadoApp::SinAtar(ConfigApp {
@@ -200,7 +206,6 @@ pub fn de_entorno_con(leer: impl Fn(&str) -> Option<String>, tty: bool) -> Crede
                             clave: PathBuf::from(clave.trim()),
                         }),
                     );
-                    continue;
                 }
                 _ => {
                     apps.insert(
@@ -221,13 +226,21 @@ pub fn de_entorno_con(leer: impl Fn(&str) -> Option<String>, tty: bool) -> Crede
     CredencialesDeEntorno { tokens, apps }
 }
 
-/// Id de App (numerico) o client id (`Iv1.` y alfanumericos).
+/// Id de App (numerico, un entero sin ceros a la izquierda: va sin comillas
+/// en el `iss` del JWT) o client id (`Iv1.` y alfanumericos).
 pub(crate) fn id_valido(id: &str) -> bool {
-    !id.is_empty()
+    let forma = !id.is_empty()
         && id.len() <= 64
         && id
             .bytes()
-            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_')
+            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_');
+    if !forma {
+        return false;
+    }
+    if id.bytes().all(|b| b.is_ascii_digit()) {
+        return id.parse::<u64>().is_ok_and(|n| n.to_string() == id);
+    }
+    true
 }
 
 impl Credenciales for CredencialesDeEntorno {
@@ -239,10 +252,17 @@ impl Credenciales for CredencialesDeEntorno {
         match self.apps.get(&rol) {
             Some(EstadoApp::Atada(app)) => app.obtener(rol),
             Some(EstadoApp::Fallo(f)) => Err(f.clone()),
-            Some(EstadoApp::SinAtar(c)) => Err(FalloCredencial::Acunar {
-                causa: causas::APP_SOLO_GITHUB,
-                detalle: format!("app {}", c.id),
-            }),
+            // Destino que no es GitHub (GitLab): la App no sirve y se usa el
+            // token del mismo rol; nunca el de otro.
+            Some(EstadoApp::SinAtar(_)) => {
+                self.tokens
+                    .get(&rol)
+                    .cloned()
+                    .ok_or_else(|| FalloCredencial::Acunar {
+                        causa: causas::APP_SOLO_GITHUB,
+                        detalle: rol.variable().to_owned(),
+                    })
+            }
             None => self.tokens.get(&rol).cloned().ok_or(FalloCredencial::Falta),
         }
     }
@@ -307,6 +327,26 @@ mod tests {
         assert!(!format!("{e:?}").contains("supersecreto"));
     }
 
+    /// MENOR-2 de la ronda 1 de TASK-047: un id numerico es un entero sin
+    /// ceros a la izquierda (va sin comillas en el `iss`); un client id, texto.
+    #[test]
+    fn el_id_de_app_numerico_no_admite_ceros_a_la_izquierda() {
+        for id in ["1", "123456", "Iv1.abc", "Iv23li_x"] {
+            assert!(id_valido(id), "{id}");
+        }
+        for id in [
+            "",
+            "0123",
+            "00",
+            "007",
+            "99999999999999999999999",
+            "1-2",
+            "a b",
+        ] {
+            assert!(!id_valido(id), "{id}");
+        }
+    }
+
     fn lector(vars: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
         let m: BTreeMap<String, String> = vars
             .iter()
@@ -333,16 +373,29 @@ mod tests {
         // Con terminal, si.
         let c = de_entorno_con(lector(&todo), true);
         assert_eq!(c.token(Rol::Persona), Some(Secreto::nuevo("tok-persona")));
-        // El agente tiene App: su token fijo no se usa nunca, y sin atar a
-        // un repo de GitHub la App no da token (ni cae al de la variable).
+        // El agente tiene App. Sin atar a un repo de GitHub (destino GitLab)
+        // la App no sirve: se usa el token del mismo rol (MEN-1 de la ronda
+        // 1 de TASK-047)...
         assert!(c.tiene(Rol::Agente));
-        match c.obtener(Rol::Agente) {
-            Err(FalloCredencial::Acunar { causa, .. }) => {
-                assert_eq!(causa, causas::APP_SOLO_GITHUB);
-            }
-            otro => panic!("{otro:?}"),
-        }
-        // Atada, con una clave que no existe: fallo al acunar, no «falta».
+        assert_eq!(c.obtener(Rol::Agente), Ok(Secreto::nuevo("tok-agente")));
+        // ...y si no lo hay, `app-solo-github` nombrando la variable del
+        // token de ese rol; nunca el token de otro rol (aqui, persona).
+        let solo_app = [
+            ("TC2_TOKEN_PERSONA", "tok-persona"),
+            ("TC2_TOKEN_REVISOR", "tok-rev"),
+            ("TC2_APP_AGENTE_ID", "123"),
+            ("TC2_APP_AGENTE_CLAVE", "/no/existe/clave.pem"),
+        ];
+        let c = de_entorno_con(lector(&solo_app), true);
+        assert_eq!(
+            c.obtener(Rol::Agente),
+            Err(FalloCredencial::Acunar {
+                causa: causas::APP_SOLO_GITHUB,
+                detalle: "TC2_TOKEN_AGENTE".to_owned(),
+            })
+        );
+        // Atada, con una clave que no existe: fallo al acunar, no «falta»,
+        // y el token de la variable no se usa en GitHub.
         let mut c = de_entorno_con(lector(&todo), true);
         c.atar_github(&Base::github(), "o/r");
         match c.obtener(Rol::Agente) {
@@ -370,6 +423,13 @@ mod tests {
                 ("TC2_APP_AGENTE_ID", "12 3"),
                 ("TC2_APP_AGENTE_CLAVE", "/x.pem"),
             ],
+            // Id numerico con ceros a la izquierda: `iss` seria JSON
+            // invalido (MENOR-2 de la ronda 1 de TASK-047).
+            vec![
+                ("TC2_APP_AGENTE_ID", "0123"),
+                ("TC2_APP_AGENTE_CLAVE", "/x.pem"),
+                ("TC2_TOKEN_AGENTE", "t"),
+            ],
         ] {
             let c = de_entorno_con(lector(&vars), true);
             match c.obtener(Rol::Agente) {
diff --git a/crates/tc2/src/plataforma/entorno_git.rs b/crates/tc2/src/plataforma/entorno_git.rs
index 1b7253e..f9974de 100644
--- a/crates/tc2/src/plataforma/entorno_git.rs
+++ b/crates/tc2/src/plataforma/entorno_git.rs
@@ -60,8 +60,10 @@ fn usuario(p: Proveedor) -> &'static str {
 
 /// Variables para que `git` hable con `url_remoto` (p. ej.
 /// `https://github.com/`; la cabecera solo va a las URL que empiezan asi)
-/// con `token` y firme como `autor`. Ocupan las entradas 0 de
-/// `GIT_CONFIG_*`: no se combinan con otras definidas igual.
+/// con `token` y firme como `autor`. La cabecera es un lote de una entrada
+/// de `GIT_CONFIG_*` numerado desde 0: [`crate::git::Git::con_entorno_extra`]
+/// lo renumera tras las entradas que ya tengan el puerto y el proceso, sin
+/// pisarlas. Pasadas a otro `Command` a mano, ocupan la entrada 0.
 pub fn entorno_git(
     proveedor: Proveedor,
     url_remoto: &str,
diff --git a/crates/tc2/src/plataforma/github.rs b/crates/tc2/src/plataforma/github.rs
index 38b574b..a79e158 100644
--- a/crates/tc2/src/plataforma/github.rs
+++ b/crates/tc2/src/plataforma/github.rs
@@ -303,6 +303,21 @@ impl std::fmt::Debug for GitHub {
     }
 }
 
+/// `dueno/repo` de GitHub: dos partes no vacias, ni `.` ni `..`, con
+/// alfanumericos, `-`, `.` y `_`. Una sola regla para el adaptador y para la
+/// GitHub App ([`super::app_github`]), que asi no divergen.
+pub(crate) fn repo_valido(repo: &str) -> bool {
+    let partes: Vec<&str> = repo.split('/').collect();
+    partes.len() == 2
+        && partes.iter().all(|p| {
+            !p.is_empty()
+                && *p != "."
+                && *p != ".."
+                && p.bytes()
+                    .all(|b| b.is_ascii_alphanumeric() || b"-._".contains(&b))
+        })
+}
+
 impl GitHub {
     /// Adaptador para `dueno/repo` en la API `base`.
     pub fn nuevo(
@@ -311,16 +326,7 @@ impl GitHub {
         credenciales: Box<dyn Credenciales>,
         personas: Personas,
     ) -> Result<Self, Informe> {
-        let partes: Vec<&str> = repo.split('/').collect();
-        let valido = partes.len() == 2
-            && partes.iter().all(|p| {
-                !p.is_empty()
-                    && *p != "."
-                    && *p != ".."
-                    && p.bytes()
-                        .all(|b| b.is_ascii_alphanumeric() || b"-._".contains(&b))
-            });
-        if !valido {
+        if !repo_valido(repo) {
             return Err(Informe::nuevo("plataforma", Resultado::Uso)
                 .con("motivo", "repo-invalido")
                 .diagnostico(&t::REPO_GITHUB));
diff --git a/docs/adr/F1-6-identidades-agente-revisor-persona.md b/docs/adr/F1-6-identidades-agente-revisor-persona.md
index 1099a7d..57f8210 100644
--- a/docs/adr/F1-6-identidades-agente-revisor-persona.md
+++ b/docs/adr/F1-6-identidades-agente-revisor-persona.md
@@ -761,12 +761,19 @@ Implementa la decisión 1 (proveedor de credenciales por rol), la tabla 2
   consigue su token → `denegada motivo=credenciales rol=<rol> causa=<causa>`
   (código 8), con `causa` entre `app-incompleta`, `app-solo-github`,
   `clave-ilegible`, `jwt-rechazado` (401), `sin-instalacion` (404),
-  `permisos-no-concedidos` (422) y `respuesta-inesperada`; sin red,
-  `sinred`. El diagnóstico nombra las variables de la App, nunca el token.
+  `permisos-no-concedidos` (422, o un 201 cuyos `permissions` no cubren los
+  pedidos), `tls` (el diagnóstico manda al almacén de certificados) y
+  `respuesta-inesperada` (con un detalle legible, nunca el `Debug` del
+  fallo); sin red, `sinred`. El diagnóstico nombra las variables de la App,
+  nunca el token. El id numérico de la App no admite ceros a la izquierda
+  (`app-incompleta`) y el PEM admite BOM UTF-8 y texto antes de
+  `-----BEGIN` (ronda 1 de revisión).
 - **GitLab sin código nuevo**: el agente es una cuenta de servicio (o un
   token de proyecto) con rol Developer y su token va en `TC2_TOKEN_AGENTE`;
-  el adaptador ya trata `bot: true` como `Bot`. Una App configurada con
-  GitLab da `causa=app-solo-github`.
+  el adaptador ya trata `bot: true` como `Bot`. Con una App configurada y un
+  destino GitLab se usa `TC2_TOKEN_<ROL>` del mismo rol (nunca de otro); si
+  no lo hay, `causa=app-solo-github` y el diagnóstico dice que en GitLab la
+  App no sirve y que hay que definir `TC2_TOKEN_<ROL>`.
 - **Git del agente** (D3 del plan: solo el punto de extensión; conectarlo a
   `review` es de F6-2): `plataforma::entorno_git(proveedor, url, token,
   autor)` devuelve `GIT_CONFIG_COUNT/KEY_0/VALUE_0` con
@@ -774,7 +781,9 @@ Implementa la decisión 1 (proveedor de credenciales por rol), la tabla 2
   (`x-access-token` en GitHub, `oauth2` en GitLab), limitado a la URL del
   remoto, y `GIT_AUTHOR_*`/`GIT_COMMITTER_*` con `<slug>[bot]` y
   `<id>+<slug>[bot]@users.noreply.github.com`. Se compone con
-  `Git::con_entorno_extra`: el token no pasa por la línea de órdenes, ni por
+  `Git::con_entorno_extra` (que, como `con_entorno`, acumula: las entradas
+  `GIT_CONFIG_*` se renumeran tras las del puerto y las del proceso, sin
+  pisarlas): el token no pasa por la línea de órdenes, ni por
   el registro del puerto, ni por los argumentos de `git-remote-http`
   (comprobado con `GIT_TRACE`). `git` no depende de `plataforma`.
 - **`identidad(Rol::Agente)` en GitHub** hace `GET /user`, que no funciona
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 8123fa68e74571def0591f22da57cfa2536e687c..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-047/planificacion/brainstorm/peticion-plan-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-047/planificacion/plan-final.md                 |    0
 tareas/03-en-revision/TASK-047/revision/informe-revision-1-code-quality-reviewer.md         |   21 ++
 tareas/03-en-revision/TASK-047/revision/informe-revision-1-rust-wasm-reviewer.md            |   21 ++
 tareas/03-en-revision/TASK-047/revision/peticion-revision-1-code-quality-reviewer.md        |  220 ++++++++++++++++++
 tareas/03-en-revision/TASK-047/revision/peticion-revision-1-rust-wasm-reviewer.md           | 1259 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-047/tarea.md                                    |    7 +-
 7 files changed, 1526 insertions(+), 2 deletions(-)
````

## Tests del diff (3 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff 8123fa68e74571def0591f22da57cfa2536e687c..HEAD -- ":(literal)crates/tc2/tests/agente_git/mod.rs" ":(literal)crates/tc2/tests/identidades.rs" ":(literal)crates/tc2/tests/plataforma_e2e.rs"
````

````
crates/tc2/tests/agente_git/mod.rs |  22 ---------
 crates/tc2/tests/identidades.rs    | 410 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-
 crates/tc2/tests/plataforma_e2e.rs |  22 +++++----
 3 files changed, 422 insertions(+), 32 deletions(-)
````
