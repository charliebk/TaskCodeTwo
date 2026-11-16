# Peticion de revision — TASK-077 (ronda 2)

- Tarea: TASK-077 — F2-13 Identidad de la persona desde la plataforma
- Rama revisada: feature/task-077-f2-13-identidad-de-la-persona-desde-la-p
- Rama base: develop
- Commit revisado (HEAD): 2fae3e4b7d864f66f5668dac349b21c345ce3f9a
- Fecha: 2026-11-16
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-077 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde eebf5f753da191740858ecfa9eea2af9c918b956 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (5 hallazgo(s)))

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

- tareas/03-en-revision/TASK-077/tarea.md
- tareas/03-en-revision/TASK-077/planificacion/
- crates/tc2/src/git/remoto.rs
- crates/tc2/src/i18n/catalogo/identidad.rs
- crates/tc2/src/identidad.rs
- crates/tc2/src/plataforma/personas.rs
- crates/tc2/src/tarea/mod.rs
- crates/tc2/src/tarea/modelo.rs
- crates/tc2/tests/identidad.rs

## Hallazgos de la ronda 1 pendientes de verificar

Tus 3 hallazgo(s) MENOR de la ronda 1 no se reverifican (los comprueba
quien coordina): recibes esta ronda porque la correccion
toca ficheros de tu dominio. Revisa en el diff de esta ronda lo que la correccion
cambia en tu terreno y si abre un fallo.

## Commits a revisar (git log eebf5f753da191740858ecfa9eea2af9c918b956..HEAD)

````
2fae3e4 fix(TASK-077): tc2 identidad trae la rama base antes de leer la lista, cache con tope y cuenta no-persona
8e7a307 fix(TASK-077): correos sin distinguir mayusculas fuera de ASCII, test que nombra las dos entradas y longitud maxima del login
475074e chore(TASK-077): veredictos y coste de la revision ronda 1
70e08c6 chore(TASK-077): informes de revision ronda 1
7a8d6e5 chore(TASK-077): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff eebf5f753da191740858ecfa9eea2af9c918b956..HEAD)

````diff
diff --git a/crates/tc2/src/git/remoto.rs b/crates/tc2/src/git/remoto.rs
index 0ac781a..268dd06 100644
--- a/crates/tc2/src/git/remoto.rs
+++ b/crates/tc2/src/git/remoto.rs
@@ -6,8 +6,8 @@ use std::time::Duration;
 
 use super::rama::renombrar;
 use super::{
-    Git, Paso, REMOTO, Subcomando, base_de_rama, commit_de, correr, entradas_status, es_ancestro,
-    es_protegida, exigir, git_fallo, localizar_principal, rama_actual,
+    Git, Paso, REMOTO, SalidaGit, Subcomando, base_de_rama, commit_de, correr, entradas_status,
+    es_ancestro, es_protegida, exigir, git_fallo, localizar_principal, rama_actual,
 };
 use crate::contrato::{Informe, Resultado, corto};
 use crate::i18n::catalogo::git as txt;
@@ -171,15 +171,37 @@ pub fn fetch(git: &Git, dir: &Path) -> Informe {
     traer(git, dir, "fetch").unwrap_or_else(|i| i)
 }
 
+/// Informe de un `fetch` que fallo: sin red, fallo local o denegado.
+fn fallo_fetch(op: &str, s: &SalidaGit) -> Informe {
+    match clasificar_fallo_remoto(&s.stderr) {
+        FalloRemoto::SinRed | FalloRemoto::RamaInexistente => sinred(op),
+        FalloRemoto::Local => error_local(op, &s.primera_linea_error()),
+        otro => denegada(op, otro, &s.primera_linea_error()),
+    }
+}
+
+/// Trae solo `rama` de `origin` (actualiza `origin/<rama>`), con la misma
+/// clasificacion de fallos que [`fetch`].
+pub(crate) fn traer_rama(git: &Git, dir: &Path, op: &str, rama: &str) -> Paso<()> {
+    let s = correr(
+        git,
+        dir,
+        op,
+        Subcomando::Fetch,
+        &["-q", "--end-of-options", REMOTO, rama],
+    )?;
+    if s.exito() {
+        Ok(())
+    } else {
+        Err(fallo_fetch(op, &s))
+    }
+}
+
 fn traer(git: &Git, dir: &Path, op: &str) -> Paso<Informe> {
     let antes = refs_remotas(git, dir, op)?;
     let s = correr(git, dir, op, Subcomando::Fetch, &["-q", REMOTO])?;
     if !s.exito() {
-        return Err(match clasificar_fallo_remoto(&s.stderr) {
-            FalloRemoto::SinRed | FalloRemoto::RamaInexistente => sinred(op),
-            FalloRemoto::Local => error_local(op, &s.primera_linea_error()),
-            otro => denegada(op, otro, &s.primera_linea_error()),
-        });
+        return Err(fallo_fetch(op, &s));
     }
     let despues = refs_remotas(git, dir, op)?;
     let r = if antes == despues {
diff --git a/crates/tc2/src/i18n/catalogo/identidad.rs b/crates/tc2/src/i18n/catalogo/identidad.rs
index eafd668..3c10023 100644
--- a/crates/tc2/src/i18n/catalogo/identidad.rs
+++ b/crates/tc2/src/i18n/catalogo/identidad.rs
@@ -17,8 +17,8 @@ crate::i18n::textos! {
         en: "`id` must be the account number (greater than 0)",
     }
     LOGIN_NO_VALIDO {
-        es: "el `login` no puede estar vacio ni llevar espacios ni caracteres de control",
-        en: "`login` cannot be empty or contain spaces or control characters",
+        es: "el `login` no puede estar vacio, pasar de 255 caracteres ni llevar espacios ni caracteres de control",
+        en: "`login` cannot be empty, be longer than 255 characters or contain spaces or control characters",
     }
     NOMBRE_NO_VALIDO {
         es: "el `nombre` no puede estar vacio ni llevar caracteres de control",
@@ -46,13 +46,17 @@ crate::i18n::textos! {
         es: "la cuenta del token de persona ({login}) es un bot: TC2_TOKEN_PERSONA tiene que ser el token de tu usuario",
         en: "the account of the person token ({login}) is a bot: TC2_TOKEN_PERSONA must be your user's token",
     }
+    CUENTA_NO_PERSONA {
+        es: "la cuenta del token de persona ({login}) es de tipo {tipo}, no de una persona: TC2_TOKEN_PERSONA tiene que ser el token de tu usuario",
+        en: "the account of the person token ({login}) is of type {tipo}, not a person: TC2_TOKEN_PERSONA must be your user's token",
+    }
     CUENTA_WEB_FLOW {
         es: "la cuenta del token de persona es web-flow, la de la web de GitHub: TC2_TOKEN_PERSONA tiene que ser el token de tu usuario",
         en: "the account of the person token is web-flow, GitHub's web account: TC2_TOKEN_PERSONA must be your user's token",
     }
     CUENTA_FUERA_DE_LISTA {
-        es: "la cuenta {login} (id {id}) no esta en {ruta}: pide que la anadan con un PR, o usa el token de una persona de la lista",
-        en: "account {login} (id {id}) is not in {ruta}: ask for it to be added with a PR, or use the token of a person on the list",
+        es: "la cuenta {login} (id {id}) no esta en {ruta}: pide que la anadan con un PR, o usa el token de una persona de la lista; si ya la anadieron, ejecuta `tc2 identidad` (actualiza la lista)",
+        en: "account {login} (id {id}) is not in {ruta}: ask for it to be added with a PR, or use the token of a person on the list; if it was already added, run `tc2 identidad` (it updates the list)",
     }
     LOGIN_DE_LA_API {
         es: "la plataforma devolvio un login que no vale para {login} (id {id}): revisa la cuenta del token",
@@ -75,8 +79,8 @@ crate::i18n::textos! {
         en: "the stored one is from another platform",
     }
     CACHE_FUERA_DE_LISTA {
-        es: "la guardada ({login}, id {id}) ya no esta en la lista de personas",
-        en: "the stored one ({login}, id {id}) is no longer on the list of people",
+        es: "la guardada ({login}, id {id}) no esta en la lista de personas de esta copia; si ya la anadieron, ejecuta `tc2 identidad` (actualiza la lista)",
+        en: "the stored one ({login}, id {id}) is not on this copy's list of people; if it was already added, run `tc2 identidad` (it updates the list)",
     }
     NO_SE_ESCRIBE_CACHE {
         es: "no se pudo guardar la identidad en {ruta}: {error}",
diff --git a/crates/tc2/src/identidad.rs b/crates/tc2/src/identidad.rs
index b9e101c..483396b 100644
--- a/crates/tc2/src/identidad.rs
+++ b/crates/tc2/src/identidad.rs
@@ -47,6 +47,10 @@ const MARGEN_RELOJ: u64 = 5 * 60;
 const CARPETA_CACHE: &str = "tc2";
 const FICHERO_CACHE: &str = "identidad.yml";
 
+/// Tamano maximo de la cache que se lee (la que escribe `tc2` ocupa menos
+/// de 400 bytes).
+pub const LIMITE_CACHE: u64 = 64 * 1024;
+
 /// Cuenta de la plataforma que puede llevar tareas: persona de la lista.
 #[derive(Debug, Clone, PartialEq, Eq)]
 pub struct Cuenta {
@@ -97,8 +101,10 @@ impl Quien {
 /// Por que una cuenta no puede llevar una tarea.
 #[derive(Debug, Clone, Copy, PartialEq, Eq)]
 pub enum NoAsignable {
-    /// Cuenta `Bot` (o cualquier tipo que no sea `User`).
+    /// Cuenta `Bot`.
     Bot,
+    /// Otro tipo que no es `User` ni `Bot` (`Organization`, `Mannequin`).
+    NoPersona,
     /// `web-flow`, la cuenta de la web de GitHub.
     WebFlow,
     /// No esta en `.taskcode/personas.yml`.
@@ -112,6 +118,7 @@ impl NoAsignable {
     pub fn nombre(self) -> &'static str {
         match self {
             Self::Bot => "bot",
+            Self::NoPersona => "no-persona",
             Self::WebFlow => "web-flow",
             Self::FueraDeLista => "fuera-de-lista",
             Self::LoginNoValido => "login-no-valido",
@@ -123,6 +130,9 @@ impl NoAsignable {
         let login = crate::plataforma::errores::sanear(&u.login);
         let m = match self {
             Self::Bot => Mensaje::nuevo(&t::CUENTA_BOT).con("login", login),
+            Self::NoPersona => Mensaje::nuevo(&t::CUENTA_NO_PERSONA)
+                .con("login", login)
+                .con("tipo", crate::plataforma::errores::sanear(&u.tipo)),
             Self::WebFlow => Mensaje::nuevo(&t::CUENTA_WEB_FLOW),
             Self::FueraDeLista => Mensaje::nuevo(&t::CUENTA_FUERA_DE_LISTA)
                 .con("login", login)
@@ -148,8 +158,10 @@ pub fn asignable(
     personas: &Personas,
     proveedor: Proveedor,
 ) -> Result<Cuenta, NoAsignable> {
-    if u.tipo != "User" {
-        return Err(NoAsignable::Bot);
+    match u.tipo.as_str() {
+        "User" => {}
+        "Bot" => return Err(NoAsignable::Bot),
+        _ => return Err(NoAsignable::NoPersona),
     }
     if u.login == "web-flow" {
         return Err(NoAsignable::WebFlow);
@@ -215,7 +227,18 @@ impl Cache {
     /// Lo guardado; `None` si no hay o no se entiende (se trata como si no
     /// hubiera: hay que volver a lanzar `tc2 identidad`).
     pub fn leer(&self) -> Option<Guardada> {
-        let texto = std::fs::read_to_string(&self.ruta).ok()?;
+        use std::io::Read;
+        // Con tope: un fichero mayor no es una cache de `tc2` (se trata como
+        // corrupta, es decir, como si no hubiera).
+        let mut texto = String::new();
+        std::fs::File::open(&self.ruta)
+            .ok()?
+            .take(LIMITE_CACHE + 1)
+            .read_to_string(&mut texto)
+            .ok()?;
+        if texto.len() as u64 > LIMITE_CACHE {
+            return None;
+        }
         let c: GuardadaCruda =
             serde_saphyr::from_str_with_options(&texto, crate::yaml::opciones()).ok()?;
         let proveedor = match c.proveedor.as_str() {
@@ -439,6 +462,12 @@ fn orden_paso(e: Entorno<'_>) -> Paso<Informe> {
             .con("motivo", "plataforma")
             .diagnostico(&t::SIN_ORIGIN));
     };
+    // Con terminal ya hace falta red (la API): se trae antes la rama base,
+    // para que la lista de personas y la configuracion sean las de ahora
+    // (quien acaba de entrar en la lista por un PR fusionado). Sin red,
+    // `sinred` y la cache sin tocar.
+    let rama_base = config::BASE.strip_prefix("origin/").unwrap_or(config::BASE);
+    git::remoto::traer_rama(e.git, e.dir, NOMBRE, rama_base)?;
     let clave = clave_plataforma(e.git, e.dir)?;
     let destino =
         detectar(&url, clave.as_ref().map(|(p, a)| (*p, a.as_str()))).map_err(|mut i| {
@@ -655,6 +684,76 @@ mod tests {
         Ok(())
     }
 
+    /// Un tipo que no es `User` ni `Bot` (`Organization`, `Mannequin`) no se
+    /// presenta como bot: `cuenta=no-persona`.
+    #[test]
+    fn otro_tipo_de_cuenta_es_no_persona_y_no_bot() {
+        let personas = Personas::nuevas([7]);
+        for tipo in ["Organization", "Mannequin"] {
+            let u = Usuario {
+                login: "org".to_owned(),
+                id: 7,
+                tipo: tipo.to_owned(),
+            };
+            let r = asignable(&u, &personas, Proveedor::GitHub);
+            assert_eq!(r, Err(NoAsignable::NoPersona), "{tipo}");
+            let i = NoAsignable::NoPersona.informe(&u);
+            assert_eq!(
+                i.linea(),
+                "identidad rechazada motivo=identidad rol=persona cuenta=no-persona"
+            );
+            for idioma in [crate::i18n::Idioma::Es, crate::i18n::Idioma::En] {
+                let d = i.diagnostico_en(idioma).unwrap_or_default();
+                assert!(d.contains(tipo) && !d.contains("bot"), "{d}");
+            }
+        }
+    }
+
+    /// La cache se lee con tope (64 KiB): mayor, cuenta como ausente. Un
+    /// login de mas de 255 caracteres no vale ni en la API ni en la cache.
+    #[test]
+    fn cache_con_tope_y_login_con_longitud_maxima() {
+        let d = carpeta("tope");
+        let cache = Cache::en(&d);
+        let cuenta = |login: &str| Cuenta {
+            proveedor: Proveedor::GitHub,
+            id: 7,
+            login: login.to_owned(),
+        };
+        assert_eq!(cache.guardar(&cuenta("ana"), AHORA), Ok(()));
+        assert!(cache.leer().is_some());
+        // La misma cache con relleno de comentario hasta pasar el tope.
+        let texto = std::fs::read_to_string(cache.ruta()).unwrap_or_default();
+        let relleno = format!("{texto}#{}\n", "x".repeat(70 * 1024));
+        assert!(std::fs::write(cache.ruta(), &relleno).is_ok());
+        assert!(cache.leer().is_none(), "mayor que el tope: ausente");
+        let justo = format!(
+            "{texto}#{}\n",
+            "x".repeat(usize::try_from(LIMITE_CACHE).unwrap_or(0) - texto.len() - 2)
+        );
+        assert_eq!(justo.len() as u64, LIMITE_CACHE);
+        assert!(std::fs::write(cache.ruta(), &justo).is_ok());
+        assert!(cache.leer().is_some(), "en el tope exacto: vale");
+        // Login: 255 vale, 256 no.
+        let (max, largo) = ("a".repeat(255), "a".repeat(256));
+        assert!(Asignado::persona(7, &max).is_some());
+        assert!(Asignado::persona(7, &largo).is_none());
+        let u = Usuario {
+            login: largo.clone(),
+            id: 7,
+            tipo: "User".to_owned(),
+        };
+        assert_eq!(
+            asignable(&u, &Personas::nuevas([7]), Proveedor::GitHub),
+            Err(NoAsignable::LoginNoValido)
+        );
+        assert_eq!(cache.guardar(&cuenta(&max), AHORA), Ok(()));
+        assert!(cache.leer().is_some());
+        assert_eq!(cache.guardar(&cuenta(&largo), AHORA), Ok(()));
+        assert!(cache.leer().is_none(), "login de mas en la cache: ausente");
+        let _ = std::fs::remove_dir_all(&d);
+    }
+
     #[test]
     fn convertir_no_toca_una_persona() -> Result<(), Informe> {
         let personas =
diff --git a/crates/tc2/src/plataforma/personas.rs b/crates/tc2/src/plataforma/personas.rs
index 2aff828..b559768 100644
--- a/crates/tc2/src/plataforma/personas.rs
+++ b/crates/tc2/src/plataforma/personas.rs
@@ -95,6 +95,12 @@ fn palabra_valida(s: &str) -> bool {
             .any(|c| c.is_control() || c.is_whitespace() || matches!(c, '\u{2028}' | '\u{2029}'))
 }
 
+/// Dos correos son el mismo sin distinguir mayusculas, tambien fuera de
+/// ASCII (`É` y `é`), igual que el login.
+fn mismo_correo(a: &str, b: &str) -> bool {
+    a.to_lowercase() == b.to_lowercase()
+}
+
 /// Rechazo de la lista con su diagnostico.
 fn rechazo(m: Mensaje) -> Informe {
     Informe::nuevo(OP, Resultado::Rechazada)
@@ -151,7 +157,7 @@ impl Personas {
             if e.id == 0 {
                 return Err(entrada(&t::ID_CERO));
             }
-            if !palabra_valida(&e.login) {
+            if !crate::tarea::login_valido(&e.login) {
                 return Err(entrada(&t::LOGIN_NO_VALIDO));
             }
             if e.nombre
@@ -191,7 +197,7 @@ impl Personas {
                 if let Some(c) = nueva
                     .correos
                     .iter()
-                    .find(|c| otra.correos.iter().any(|o| o.eq_ignore_ascii_case(c)))
+                    .find(|c| otra.correos.iter().any(|o| mismo_correo(o, c)))
                 {
                     return Err(rechazo(
                         Mensaje::nuevo(&t::CORREO_REPETIDO)
@@ -236,7 +242,7 @@ impl Personas {
         let correo = correo.trim();
         self.entradas
             .iter()
-            .find(|p| p.correos.iter().any(|c| c.eq_ignore_ascii_case(correo)))
+            .find(|p| p.correos.iter().any(|c| mismo_correo(c, correo)))
     }
 
     /// Las entradas, en el orden del fichero.
@@ -279,7 +285,26 @@ mod tests {
             Some("personas rechazada motivo=personas")
         );
         let correo = "personas:\n  - {id: 7, login: \"a\", correos: [\"x@y\"]}\n  - {id: 9, login: \"b\", correos: [\"X@Y\"]}\n";
-        assert!(error(correo).contains("x@y") || error(correo).contains("X@Y"));
+        let e = error(correo);
+        assert!(
+            e.contains("1 (a, id 7)") && e.contains("2 (b, id 9)") && e.contains("X@Y"),
+            "{e}"
+        );
+    }
+
+    /// Los correos se comparan sin distinguir mayusculas tambien fuera de
+    /// ASCII, como el login: `É@x` y `é@x` son el mismo.
+    #[test]
+    fn correos_sin_distinguir_mayusculas_fuera_de_ascii() -> Result<(), Informe> {
+        let repetido = "personas:\n  - {id: 7, login: \"a\", correos: [\"É@x\"]}\n  - {id: 9, login: \"b\", correos: [\"é@x\"]}\n";
+        let e = error(repetido);
+        assert!(
+            e.contains("1 (a, id 7)") && e.contains("2 (b, id 9)") && e.contains("é@x"),
+            "{e}"
+        );
+        let l = Personas::leer("personas:\n  - {id: 7, login: \"a\", correos: [\"É@x\"]}\n")?;
+        assert_eq!(l.por_correo("é@X").map(|p| p.id), Some(7));
+        Ok(())
     }
 
     #[test]
diff --git a/crates/tc2/src/tarea/mod.rs b/crates/tc2/src/tarea/mod.rs
index 96ae2b7..198b2ae 100644
--- a/crates/tc2/src/tarea/mod.rs
+++ b/crates/tc2/src/tarea/mod.rs
@@ -19,8 +19,8 @@ use std::fmt;
 
 pub use cuerpo::{Cuerpo, Seccion};
 pub use modelo::{
-    Asignado, COMPLEJIDADES, ESTADOS, Heredado, REGLAS, Rol, RolRevision, Roles, TIPOS, Tarea,
-    TareaV1, id_valido, titulo_valido,
+    Asignado, COMPLEJIDADES, ESTADOS, Heredado, LOGIN_MAX, REGLAS, Rol, RolRevision, Roles, TIPOS,
+    Tarea, TareaV1, id_valido, login_valido, titulo_valido,
 };
 
 /// Un `tarea.md` leido.
diff --git a/crates/tc2/src/tarea/modelo.rs b/crates/tc2/src/tarea/modelo.rs
index 10a3c37..d0f8d7b 100644
--- a/crates/tc2/src/tarea/modelo.rs
+++ b/crates/tc2/src/tarea/modelo.rs
@@ -209,9 +209,17 @@ impl Asignado {
     }
 }
 
-/// Login de una cuenta: no vacio, sin espacios ni caracteres de control.
-fn login_valido(login: &str) -> bool {
-    !login.is_empty() && texto_valido(login) && !login.chars().any(char::is_whitespace)
+/// Longitud maxima de un login, en caracteres (GitLab admite 255; GitHub,
+/// 39).
+pub const LOGIN_MAX: usize = 255;
+
+/// Login de una cuenta: no vacio, de [`LOGIN_MAX`] caracteres como mucho, sin
+/// espacios ni caracteres de control.
+pub fn login_valido(login: &str) -> bool {
+    !login.is_empty()
+        && login.chars().count() <= LOGIN_MAX
+        && texto_valido(login)
+        && !login.chars().any(char::is_whitespace)
 }
 
 /// `asignado_a` del formato 2: `null`, `{id, login}` o un texto heredado.
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff eebf5f753da191740858ecfa9eea2af9c918b956..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-077/planificacion/brainstorm/peticion-plan-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-077/planificacion/plan-final.md                 |    0
 tareas/03-en-revision/TASK-077/revision/informe-revision-1-code-quality-reviewer.md         |   80 +++++
 tareas/03-en-revision/TASK-077/revision/informe-revision-1-rust-wasm-reviewer.md            |   81 +++++
 tareas/03-en-revision/TASK-077/revision/peticion-revision-1-code-quality-reviewer.md        |   92 ++++++
 tareas/03-en-revision/TASK-077/revision/peticion-revision-1-rust-wasm-reviewer.md           | 1654 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-077/tarea.md                                    |   16 +-
 7 files changed, 1920 insertions(+), 3 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff eebf5f753da191740858ecfa9eea2af9c918b956..HEAD -- ":(literal)crates/tc2/tests/identidad.rs"
````

````
crates/tc2/tests/identidad.rs | 136 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-------
 1 file changed, 129 insertions(+), 7 deletions(-)
````
