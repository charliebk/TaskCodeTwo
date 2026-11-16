# Peticion de revision — TASK-077 (ronda 1)

- Tarea: TASK-077 — F2-13 Identidad de la persona desde la plataforma
- Rama revisada: feature/task-077-f2-13-identidad-de-la-persona-desde-la-p
- Rama base: develop
- Commit revisado (HEAD): eebf5f753da191740858ecfa9eea2af9c918b956
- Fecha: 2026-11-16
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-077 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-077/tarea.md
- tareas/03-en-revision/TASK-077/planificacion/
- crates/tc2/src/git/estado.rs
- crates/tc2/src/i18n/catalogo/cli.rs
- crates/tc2/src/i18n/catalogo/identidad.rs
- crates/tc2/src/i18n/catalogo/mod.rs
- crates/tc2/src/identidad.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/plataforma/mod.rs
- crates/tc2/src/plataforma/personas.rs
- crates/tc2/src/tarea/mod.rs
- crates/tc2/src/tarea/modelo.rs
- crates/tc2/tests/identidad.rs
- crates/tc2/tests/tarea.rs

## Commits a revisar (git log develop..HEAD)

````
eebf5f7 chore(TASK-077): criterios C1 y C7 con el formato del contrato F1-4
3b4e626 docs(TASK-077): autorrevision con las dos revisoras
ce6cc61 fix(TASK-077): autorrevision, login de la cuenta sin rodeos y doc del modulo identidad
b934342 feat(TASK-077): conversion de asignado_a heredado por correos y nota en el ADR F1-9
2474c3a feat(TASK-077): orden tc2 identidad con su catalogo y tests contra GitHub y GitLab simulados
15ee5b7 feat(TASK-077): quien ejecuta, cache local de identidad con TTL y rechazo de bots
6bdcf37 feat(TASK-077): asignado_a como persona {id, login} o texto heredado de la 0.16
b3ed37d feat(TASK-077): lista de personas con login, nombre y correos y unicidad
f175cc6 chore(TASK-077): tarea en curso
````

## Diff de tu dominio (12 fichero(s) de 13; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/git/estado.rs b/crates/tc2/src/git/estado.rs
index 5b8ce3e..be0dee3 100644
--- a/crates/tc2/src/git/estado.rs
+++ b/crates/tc2/src/git/estado.rs
@@ -353,6 +353,19 @@ pub(crate) fn hay_remoto(git: &Git, dir: &Path, op: &str) -> Paso<bool> {
     Ok(s.exito() && s.stdout.trim() != super::REMOTO && !s.stdout.trim().is_empty())
 }
 
+/// URL de `origin`, si esta configurado (sin hablar con el remoto).
+pub(crate) fn url_origin(git: &Git, dir: &Path, op: &str) -> Paso<Option<String>> {
+    let s = correr(
+        git,
+        dir,
+        op,
+        Subcomando::LsRemote,
+        &["--get-url", super::REMOTO],
+    )?;
+    let url = s.stdout.trim();
+    Ok((s.exito() && url != super::REMOTO && !url.is_empty()).then(|| url.to_owned()))
+}
+
 #[cfg(test)]
 mod tests {
     use super::{partir_batch, ruta_valida};
diff --git a/crates/tc2/src/i18n/catalogo/cli.rs b/crates/tc2/src/i18n/catalogo/cli.rs
index 3cadfae..2347eaf 100644
--- a/crates/tc2/src/i18n/catalogo/cli.rs
+++ b/crates/tc2/src/i18n/catalogo/cli.rs
@@ -10,8 +10,8 @@ crate::i18n::textos! {
         en: "tc2: could not write the output: {error}",
     }
     USO {
-        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json]",
-        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json]",
+        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json]",
+        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json]",
     }
     FALTA_LA_ORDEN {
         es: "tc2: falta la orden. {uso}",
diff --git a/crates/tc2/src/i18n/catalogo/identidad.rs b/crates/tc2/src/i18n/catalogo/identidad.rs
new file mode 100644
index 0000000..eafd668
--- /dev/null
+++ b/crates/tc2/src/i18n/catalogo/identidad.rs
@@ -0,0 +1,103 @@
+//! Mensajes de la identidad de la persona (F2-13): la lista
+//! `.taskcode/personas.yml` (`plataforma/personas.rs`), quien ejecuta
+//! (`identidad.rs`) y la orden `tc2 identidad`.
+
+crate::i18n::textos! {
+    // --- Lista de personas (plataforma/personas.rs) ---
+    PERSONAS_YAML {
+        es: "{ruta} no es YAML valido o no tiene la forma esperada (`personas:` con `id` y `login` por entrada): {error}",
+        en: "{ruta} is not valid YAML or does not have the expected shape (`personas:` with `id` and `login` per entry): {error}",
+    }
+    PERSONAS_ENTRADA {
+        es: "{ruta}: la entrada {n} no vale: {que}",
+        en: "{ruta}: entry {n} is not valid: {que}",
+    }
+    ID_CERO {
+        es: "el `id` tiene que ser el numero de la cuenta (mayor que 0)",
+        en: "`id` must be the account number (greater than 0)",
+    }
+    LOGIN_NO_VALIDO {
+        es: "el `login` no puede estar vacio ni llevar espacios ni caracteres de control",
+        en: "`login` cannot be empty or contain spaces or control characters",
+    }
+    NOMBRE_NO_VALIDO {
+        es: "el `nombre` no puede estar vacio ni llevar caracteres de control",
+        en: "`nombre` cannot be empty or contain control characters",
+    }
+    CORREO_NO_VALIDO {
+        es: "cada `correos` tiene que ser una direccion sin espacios ni caracteres de control",
+        en: "each `correos` entry must be an address without spaces or control characters",
+    }
+    ID_REPETIDO {
+        es: "{ruta}: las entradas {a} y {b} repiten el id {id}; cada cuenta va una sola vez",
+        en: "{ruta}: entries {a} and {b} repeat id {id}; each account goes only once",
+    }
+    LOGIN_REPETIDO {
+        es: "{ruta}: las entradas {a} y {b} repiten el login (sin distinguir mayusculas); cada cuenta va una sola vez",
+        en: "{ruta}: entries {a} and {b} repeat the login (case-insensitive); each account goes only once",
+    }
+    CORREO_REPETIDO {
+        es: "{ruta}: las entradas {a} y {b} repiten el correo {correo}; un correo de la 0.16 solo puede ser de una persona",
+        en: "{ruta}: entries {a} and {b} repeat the address {correo}; a 0.16 address can belong to only one person",
+    }
+
+    // --- Quien ejecuta (identidad.rs) ---
+    CUENTA_BOT {
+        es: "la cuenta del token de persona ({login}) es un bot: TC2_TOKEN_PERSONA tiene que ser el token de tu usuario",
+        en: "the account of the person token ({login}) is a bot: TC2_TOKEN_PERSONA must be your user's token",
+    }
+    CUENTA_WEB_FLOW {
+        es: "la cuenta del token de persona es web-flow, la de la web de GitHub: TC2_TOKEN_PERSONA tiene que ser el token de tu usuario",
+        en: "the account of the person token is web-flow, GitHub's web account: TC2_TOKEN_PERSONA must be your user's token",
+    }
+    CUENTA_FUERA_DE_LISTA {
+        es: "la cuenta {login} (id {id}) no esta en {ruta}: pide que la anadan con un PR, o usa el token de una persona de la lista",
+        en: "account {login} (id {id}) is not in {ruta}: ask for it to be added with a PR, or use the token of a person on the list",
+    }
+    LOGIN_DE_LA_API {
+        es: "la plataforma devolvio un login que no vale para {login} (id {id}): revisa la cuenta del token",
+        en: "the platform returned an invalid login for {login} (id {id}): check the token's account",
+    }
+    SIN_IDENTIDAD_GUARDADA {
+        es: "sin terminal, el agente actua en nombre de la persona guardada y {estado}: lanza `tc2 identidad` en un terminal con TC2_TOKEN_PERSONA",
+        en: "without a terminal, the agent acts on behalf of the stored person and {estado}: run `tc2 identidad` in a terminal with TC2_TOKEN_PERSONA",
+    }
+    CACHE_FALTA {
+        es: "no hay ninguna guardada",
+        en: "there is none stored",
+    }
+    CACHE_CADUCADA {
+        es: "la guardada tiene mas de 7 dias",
+        en: "the stored one is more than 7 days old",
+    }
+    CACHE_OTRA_PLATAFORMA {
+        es: "la guardada es de otra plataforma",
+        en: "the stored one is from another platform",
+    }
+    CACHE_FUERA_DE_LISTA {
+        es: "la guardada ({login}, id {id}) ya no esta en la lista de personas",
+        en: "the stored one ({login}, id {id}) is no longer on the list of people",
+    }
+    NO_SE_ESCRIBE_CACHE {
+        es: "no se pudo guardar la identidad en {ruta}: {error}",
+        en: "could not store the identity in {ruta}: {error}",
+    }
+
+    // --- Orden `tc2 identidad` ---
+    USO_IDENTIDAD {
+        es: "uso: identidad [--json]",
+        en: "usage: identidad [--json]",
+    }
+    NECESITA_TERMINAL {
+        es: "`tc2 identidad` la lanza la persona en un terminal con TC2_TOKEN_PERSONA: sin terminal no se verifica ni se guarda nada",
+        en: "`tc2 identidad` is run by the person in a terminal with TC2_TOKEN_PERSONA: without a terminal nothing is verified or stored",
+    }
+    SIN_ORIGIN {
+        es: "el repo no tiene `origin`: la identidad sale de la plataforma de `origin` (GitHub o GitLab) y no hay modo sin plataforma",
+        en: "the repo has no `origin`: the identity comes from the `origin` platform (GitHub or GitLab) and there is no mode without a platform",
+    }
+    SIN_PERSONAS {
+        es: "no hay {ruta} en {base}: anade la lista de personas (id y login de cada cuenta) con un PR",
+        en: "there is no {ruta} in {base}: add the list of people (id and login of each account) with a PR",
+    }
+}
diff --git a/crates/tc2/src/i18n/catalogo/mod.rs b/crates/tc2/src/i18n/catalogo/mod.rs
index b2ff678..9f91662 100644
--- a/crates/tc2/src/i18n/catalogo/mod.rs
+++ b/crates/tc2/src/i18n/catalogo/mod.rs
@@ -7,6 +7,7 @@ pub mod comun;
 pub mod config;
 pub mod estado;
 pub mod git;
+pub mod identidad;
 pub mod plataforma;
 
 use super::Texto;
@@ -20,6 +21,7 @@ pub fn todos() -> impl Iterator<Item = &'static Texto> {
         config::TODOS,
         estado::TODOS,
         git::TODOS,
+        identidad::TODOS,
         plataforma::TODOS,
     ]
     .into_iter()
diff --git a/crates/tc2/src/identidad.rs b/crates/tc2/src/identidad.rs
new file mode 100644
index 0000000..b9e101c
--- /dev/null
+++ b/crates/tc2/src/identidad.rs
@@ -0,0 +1,666 @@
+//! Identidad de la persona (F2-13, RR-12): el «quien» de una tarea es la
+//! cuenta de GitHub o GitLab autenticada, no un texto libre.
+//!
+//! - Con terminal, la persona se identifica por la API con su token
+//!   (`GET /user` por el puerto [`Plataforma`], sin `gh` ni `glab`, F1-14):
+//!   tiene que ser `User`, no `web-flow` y estar en `.taskcode/personas.yml`
+//!   por su `id` (F1-6, decision 3). `tc2 identidad` la guarda en la cache
+//!   local.
+//! - Sin terminal, quien ejecuta es el agente, en nombre de la persona de la
+//!   cache (D3, RN-06): no pide nada a la red y su transicion lleva
+//!   `decidido_por: agente`. Sin cache, caducada (7 dias) o de otra
+//!   plataforma, se rechaza pidiendo `tc2 identidad`.
+//! - Sin plataforma reconocida no hay identidad (D4): no hay vuelta a texto
+//!   libre.
+//!
+//! La cache vive en `<git-common-dir>/tc2/identidad.yml`: no se commitea y la
+//! comparten los worktrees del clon. Editarla no aprueba nada: la aprobacion
+//! sigue siendo la firma de F1-6/F4-2 (riesgo aceptado en el plan).
+
+use std::path::{Path, PathBuf};
+use std::time::{SystemTime, UNIX_EPOCH};
+
+use serde::Deserialize;
+
+use crate::config;
+use crate::contrato::{Informe, Resultado};
+use crate::git::{self, Git, Paso, Subcomando};
+use crate::i18n::catalogo::identidad as t;
+use crate::i18n::{Mensaje, Texto};
+use crate::plataforma::personas::RUTA as RUTA_PERSONAS;
+use crate::plataforma::{
+    Credenciales, GitHub, GitLab, Personas, Plataforma, Proveedor, Rol, Usuario, detectar,
+};
+use crate::tarea::Asignado;
+
+/// Nombre de la orden y de la operacion en la linea.
+pub const NOMBRE: &str = "identidad";
+
+/// Vigencia de la identidad guardada: 7 dias desde que se verifico (D2).
+pub const TTL_SEGUNDOS: u64 = 7 * 24 * 60 * 60;
+
+/// Margen para un reloj adelantado: una cache «del futuro» mas alla de esto
+/// no vale.
+const MARGEN_RELOJ: u64 = 5 * 60;
+
+/// Carpeta y fichero de la cache bajo la carpeta comun del repo.
+const CARPETA_CACHE: &str = "tc2";
+const FICHERO_CACHE: &str = "identidad.yml";
+
+/// Cuenta de la plataforma que puede llevar tareas: persona de la lista.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Cuenta {
+    /// Plataforma de la cuenta (los `id` de GitHub y GitLab no se comparan).
+    pub proveedor: Proveedor,
+    /// Id numerico.
+    pub id: u64,
+    /// Login (para ensenar).
+    pub login: String,
+}
+
+/// Quien ejecuta una operacion.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum Quien {
+    /// La persona, verificada ahora por la API con su token (terminal).
+    Persona(Cuenta),
+    /// El agente, sin terminal, en nombre de la persona de la cache.
+    Agente(Cuenta),
+}
+
+impl Quien {
+    /// La cuenta de la persona (la propia o en cuyo nombre actua el agente).
+    pub fn cuenta(&self) -> &Cuenta {
+        match self {
+            Self::Persona(c) | Self::Agente(c) => c,
+        }
+    }
+
+    /// Valor de `decidido_por` de la transicion.
+    pub fn decidido_por(&self) -> &'static str {
+        match self {
+            Self::Persona(_) => "persona",
+            Self::Agente(_) => "agente",
+        }
+    }
+
+    /// El `asignado_a` que escribe una toma: siempre la persona, nunca un
+    /// texto.
+    pub fn asignado(&self) -> Asignado {
+        let c = self.cuenta();
+        Asignado::Persona {
+            id: c.id,
+            login: c.login.clone(),
+        }
+    }
+}
+
+/// Por que una cuenta no puede llevar una tarea.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum NoAsignable {
+    /// Cuenta `Bot` (o cualquier tipo que no sea `User`).
+    Bot,
+    /// `web-flow`, la cuenta de la web de GitHub.
+    WebFlow,
+    /// No esta en `.taskcode/personas.yml`.
+    FueraDeLista,
+    /// La API dio un login vacio o con espacios o controles.
+    LoginNoValido,
+}
+
+impl NoAsignable {
+    /// Valor de `cuenta=` en la linea.
+    pub fn nombre(self) -> &'static str {
+        match self {
+            Self::Bot => "bot",
+            Self::WebFlow => "web-flow",
+            Self::FueraDeLista => "fuera-de-lista",
+            Self::LoginNoValido => "login-no-valido",
+        }
+    }
+
+    /// El rechazo, con el arreglo para la persona.
+    pub fn informe(self, u: &Usuario) -> Informe {
+        let login = crate::plataforma::errores::sanear(&u.login);
+        let m = match self {
+            Self::Bot => Mensaje::nuevo(&t::CUENTA_BOT).con("login", login),
+            Self::WebFlow => Mensaje::nuevo(&t::CUENTA_WEB_FLOW),
+            Self::FueraDeLista => Mensaje::nuevo(&t::CUENTA_FUERA_DE_LISTA)
+                .con("login", login)
+                .con("id", u.id.to_string())
+                .con("ruta", RUTA_PERSONAS),
+            Self::LoginNoValido => Mensaje::nuevo(&t::LOGIN_DE_LA_API)
+                .con("login", login)
+                .con("id", u.id.to_string()),
+        };
+        Informe::nuevo(NOMBRE, Resultado::Rechazada)
+            .con("motivo", "identidad")
+            .con("rol", "persona")
+            .con("cuenta", self.nombre())
+            .diagnostico(m)
+    }
+}
+
+/// La cuenta `u` de `proveedor`, si puede llevar tareas: `User`, no
+/// `web-flow` y en la lista por su `id`. Una cuenta `Bot` nunca llega a
+/// `asignado_a`.
+pub fn asignable(
+    u: &Usuario,
+    personas: &Personas,
+    proveedor: Proveedor,
+) -> Result<Cuenta, NoAsignable> {
+    if u.tipo != "User" {
+        return Err(NoAsignable::Bot);
+    }
+    if u.login == "web-flow" {
+        return Err(NoAsignable::WebFlow);
+    }
+    if !personas.contiene(u.id) {
+        return Err(NoAsignable::FueraDeLista);
+    }
+    // El mismo criterio de login que `asignado_a` (un login que la tarea no
+    // podria guardar no se acepta aqui).
+    Asignado::persona(u.id, &u.login).ok_or(NoAsignable::LoginNoValido)?;
+    Ok(Cuenta {
+        proveedor,
+        id: u.id,
+        login: u.login.clone(),
+    })
+}
+
+/// Identidad guardada en la cache.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Guardada {
+    /// La cuenta.
+    pub cuenta: Cuenta,
+    /// Cuando se verifico por la API, en segundos desde 1970.
+    pub verificado: u64,
+}
+
+impl Guardada {
+    /// Si sigue valiendo en `ahora`: menos de 7 dias y no del futuro.
+    pub fn vigente(&self, ahora: u64) -> bool {
+        self.verificado <= ahora.saturating_add(MARGEN_RELOJ)
+            && ahora.saturating_sub(self.verificado) < TTL_SEGUNDOS
+    }
+}
+
+#[derive(Deserialize)]
+#[serde(deny_unknown_fields)]
+struct GuardadaCruda {
+    proveedor: String,
+    id: u64,
+    login: String,
+    verificado: u64,
+}
+
+/// Cache local de la identidad (`<git-common-dir>/tc2/identidad.yml`).
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Cache {
+    ruta: PathBuf,
+}
+
+impl Cache {
+    /// La cache del repo cuya carpeta comun es `comun`.
+    pub fn en(comun: &Path) -> Self {
+        Self {
+            ruta: comun.join(CARPETA_CACHE).join(FICHERO_CACHE),
+        }
+    }
+
+    /// Ruta del fichero.
+    pub fn ruta(&self) -> &Path {
+        &self.ruta
+    }
+
+    /// Lo guardado; `None` si no hay o no se entiende (se trata como si no
+    /// hubiera: hay que volver a lanzar `tc2 identidad`).
+    pub fn leer(&self) -> Option<Guardada> {
+        let texto = std::fs::read_to_string(&self.ruta).ok()?;
+        let c: GuardadaCruda =
+            serde_saphyr::from_str_with_options(&texto, crate::yaml::opciones()).ok()?;
+        let proveedor = match c.proveedor.as_str() {
+            "github" => Proveedor::GitHub,
+            "gitlab" => Proveedor::GitLab,
+            _ => return None,
+        };
+        Asignado::persona(c.id, &c.login)?;
+        Some(Guardada {
+            cuenta: Cuenta {
+                proveedor,
+                id: c.id,
+                login: c.login,
+            },
+            verificado: c.verificado,
+        })
+    }
+
+    /// Guarda `cuenta` verificada en `ahora`. Escribe un temporal y lo
+    /// renombra: quien lee no ve nunca un fichero a medias.
+    pub fn guardar(&self, cuenta: &Cuenta, ahora: u64) -> Paso<()> {
+        let error = |e: std::io::Error| {
+            Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(
+                Mensaje::nuevo(&t::NO_SE_ESCRIBE_CACHE)
+                    .con("ruta", self.ruta.display().to_string())
+                    .con("error", e.to_string()),
+            )
+        };
+        if let Some(padre) = self.ruta.parent() {
+            std::fs::create_dir_all(padre).map_err(error)?;
+        }
+        let texto = format!(
+            "---\nproveedor: \"{}\"\nid: {}\nlogin: {}\nverificado: {ahora}\n",
+            cuenta.proveedor.nombre(),
+            cuenta.id,
+            crate::tarea::yaml::cadena(&cuenta.login),
+        );
+        let tmp = self
+            .ruta
+            .with_extension(format!("yml.tmp-{}", std::process::id()));
+        std::fs::write(&tmp, texto).map_err(error)?;
+        std::fs::rename(&tmp, &self.ruta).map_err(|e| {
+            let _ = std::fs::remove_file(&tmp);
+            error(e)
+        })
+    }
+}
+
+/// Segundos desde 1970 (0 si el reloj esta antes, que deja cualquier cache
+/// por caducada o del futuro).
+pub fn ahora() -> u64 {
+    SystemTime::now()
+        .duration_since(UNIX_EPOCH)
+        .map_or(0, |d| d.as_secs())
+}
+
+/// Rechazo por falta de una identidad guardada que valga.
+fn sin_guardada(estado: &'static str, que: Mensaje) -> Informe {
+    Informe::nuevo(NOMBRE, Resultado::Rechazada)
+        .con("motivo", "identidad")
+        .con("cache", estado)
+        .diagnostico(Mensaje::nuevo(&t::SIN_IDENTIDAD_GUARDADA).con_mensaje("estado", que))
+}
+
+/// Quien ejecuta: con terminal, la persona del token verificada ahora por la
+/// API; sin terminal, el agente en nombre de la persona de la cache vigente,
+/// sin pedir nada a la red. La persona tiene que estar en `personas` (por
+/// `id`; su login sale de la lista si esta, que refresca un renombrado).
+pub fn quien_ejecuta(
+    plataforma: &dyn Plataforma,
+    proveedor: Proveedor,
+    personas: &Personas,
+    cache: &Cache,
+    tty: bool,
+    ahora: u64,
+) -> Paso<Quien> {
+    if tty {
+        let u = plataforma.identidad(Rol::Persona).map_err(|mut i| {
+            NOMBRE.clone_into(&mut i.comando);
+            i
+        })?;
+        return asignable(&u, personas, proveedor)
+            .map(Quien::Persona)
+            .map_err(|n| n.informe(&u));
+    }
+    let Some(g) = cache.leer() else {
+        return Err(sin_guardada("falta", Mensaje::nuevo(&t::CACHE_FALTA)));
+    };
+    if g.cuenta.proveedor != proveedor {
+        return Err(sin_guardada(
+            "otra-plataforma",
+            Mensaje::nuevo(&t::CACHE_OTRA_PLATAFORMA),
+        ));
+    }
+    if !g.vigente(ahora) {
+        return Err(sin_guardada("caducada", Mensaje::nuevo(&t::CACHE_CADUCADA)));
+    }
+    if !personas.contiene(g.cuenta.id) {
+        return Err(sin_guardada(
+            "fuera-de-lista",
+            Mensaje::nuevo(&t::CACHE_FUERA_DE_LISTA)
+                .con("login", g.cuenta.login.clone())
+                .con("id", g.cuenta.id.to_string()),
+        ));
+    }
+    let mut cuenta = g.cuenta;
+    if let Some(p) = personas.por_id(cuenta.id) {
+        cuenta.login.clone_from(&p.login);
+    }
+    Ok(Quien::Agente(cuenta))
+}
+
+/// Convierte un `asignado_a` heredado de la 0.16 si su texto es un
+/// `correos:` de una persona de la lista (D5); si no casa, queda como esta
+/// (`doctor` avisa y `reasignar`, F3-10, lo resuelve).
+pub fn convertir_heredado(a: Asignado, personas: &Personas) -> Asignado {
+    let persona = a
+        .heredado()
+        .and_then(|texto| personas.por_correo(texto))
+        .and_then(|p| Asignado::persona(p.id, &p.login));
+    persona.unwrap_or(a)
+}
+
+/// Lo que necesita la orden `tc2 identidad` del entorno, para poder
+/// probarla sin terminal real ni tokens del proceso.
+pub struct Entorno<'a> {
+    /// Carpeta desde la que se lanza.
+    pub dir: &'a Path,
+    /// Puerto Git.
+    pub git: &'a Git,
+    /// Si la entrada estandar es un terminal.
+    pub tty: bool,
+    /// Tokens por rol.
+    pub credenciales: Box<dyn Credenciales>,
+    /// Hora, en segundos desde 1970.
+    pub ahora: u64,
+}
+
+/// Lista de personas de la base (`origin/develop`), nunca del worktree:
+/// cambiarla exige un PR aprobado (F1-6, decision 3).
+pub fn personas_de_base(g: &Git, dir: &Path) -> Paso<Personas> {
+    let falta = || {
+        Informe::nuevo(NOMBRE, Resultado::Rechazada)
+            .con("motivo", "sin-personas")
+            .diagnostico(
+                Mensaje::nuevo(&t::SIN_PERSONAS)
+                    .con("ruta", RUTA_PERSONAS)
+                    .con("base", config::BASE),
+            )
+    };
+    if git::commit_de(g, dir, NOMBRE, config::BASE)?.is_none() {
+        return Err(falta());
+    }
+    let listado = git::exigir(
+        g,
+        dir,
+        NOMBRE,
+        Subcomando::LsTree,
+        &[
+            "--name-only",
+            "--end-of-options",
+            config::BASE,
+            "--",
+            RUTA_PERSONAS,
+        ],
+    )?;
+    if listado.trim() != RUTA_PERSONAS {
+        return Err(falta());
+    }
+    let texto = git::estado::leer_blob(g, dir, NOMBRE, config::BASE, RUTA_PERSONAS)?;
+    Personas::leer(&texto).map_err(|mut i| {
+        NOMBRE.clone_into(&mut i.comando);
+        i
+    })
+}
+
+/// La clave `plataforma` de la configuracion de la base, si la hay. Sin
+/// base o sin configuracion se detecta solo por el host; una configuracion
+/// que no vale falla.
+fn clave_plataforma(g: &Git, dir: &Path) -> Paso<Option<(&'static str, String)>> {
+    match config::de_base(g, dir) {
+        Ok(c) => Ok(c.plataforma.map(|p| (p.proveedor.nombre(), p.api))),
+        Err(i) if matches!(i.valor("motivo"), Some("sin-base" | "sin-config")) => Ok(None),
+        Err(mut i) => {
+            NOMBRE.clone_into(&mut i.comando);
+            Err(i)
+        }
+    }
+}
+
+/// Uso de la orden.
+fn uso(diag: &'static Texto) -> Informe {
+    Informe::nuevo(NOMBRE, Resultado::Uso)
+        .con("motivo", "argumentos-de-mas")
+        .diagnostico(diag)
+}
+
+/// `tc2 identidad`: con terminal y `TC2_TOKEN_PERSONA`, verifica la persona
+/// por la API y la guarda en la cache. Sale `identidad ok rol=persona id=N
+/// login=L`. Si no es una persona de la lista, sin red o sin plataforma
+/// reconocida, falla sin tocar la cache.
+pub fn orden(args: &[String], e: Entorno<'_>) -> Informe {
+    if !args.is_empty() {
+        return uso(&t::USO_IDENTIDAD);
+    }
+    match orden_paso(e) {
+        Ok(i) | Err(i) => i,
+    }
+}
+
+fn orden_paso(e: Entorno<'_>) -> Paso<Informe> {
+    if !e.tty {
+        return Err(Informe::nuevo(NOMBRE, Resultado::Rechazada)
+            .con("motivo", "identidad")
+            .con("rol", "persona")
+            .diagnostico(&t::NECESITA_TERMINAL));
+    }
+    let comun = git::estado::dir_comun(e.git, e.dir, NOMBRE)?;
+    let Some(url) = git::estado::url_origin(e.git, e.dir, NOMBRE)? else {
+        return Err(Informe::nuevo(NOMBRE, Resultado::Rechazada)
+            .con("motivo", "plataforma")
+            .diagnostico(&t::SIN_ORIGIN));
+    };
+    let clave = clave_plataforma(e.git, e.dir)?;
+    let destino =
+        detectar(&url, clave.as_ref().map(|(p, a)| (*p, a.as_str()))).map_err(|mut i| {
+            NOMBRE.clone_into(&mut i.comando);
+            i
+        })?;
+    let personas = personas_de_base(e.git, e.dir)?;
+    let plataforma: Box<dyn Plataforma> = match destino.proveedor {
+        Proveedor::GitHub => Box::new(GitHub::nuevo(
+            destino.api,
+            &destino.proyecto,
+            e.credenciales,
+            personas.clone(),
+        )?),
+        Proveedor::GitLab => Box::new(GitLab::nuevo(
+            destino.api,
+            &destino.proyecto,
+            e.credenciales,
+            personas.clone(),
+        )?),
+    };
+    let cache = Cache::en(&comun);
+    let quien = quien_ejecuta(
+        plataforma.as_ref(),
+        destino.proveedor,
+        &personas,
+        &cache,
+        true,
+        e.ahora,
+    )?;
+    let c = quien.cuenta();
+    cache.guardar(c, e.ahora)?;
+    Ok(Informe::nuevo(NOMBRE, Resultado::Ok)
+        .con("rol", "persona")
+        .con("id", c.id.to_string())
+        .con("login", c.login.clone()))
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::plataforma::{Firma, ModeloPuertas, ReglasLeidas};
+    use std::cell::Cell;
+
+    /// Plataforma de prueba: solo `identidad`, contando las llamadas.
+    struct Falsa {
+        usuario: Usuario,
+        llamadas: Cell<usize>,
+    }
+
+    impl Falsa {
+        fn de(login: &str, id: u64, tipo: &str) -> Self {
+            Self {
+                usuario: Usuario {
+                    login: login.to_owned(),
+                    id,
+                    tipo: tipo.to_owned(),
+                },
+                llamadas: Cell::new(0),
+            }
+        }
+    }
+
+    fn sin_uso(op: &str) -> Informe {
+        Informe::nuevo(op, Resultado::Error)
+    }
+
+    impl Plataforma for Falsa {
+        fn abrir_pr(&self, _: &str, _: &str, _: &str, _: &str) -> Informe {
+            sin_uso("abrir-pr")
+        }
+        fn estado_puertas(&self, _: Rol, _: u64, _: &ModeloPuertas) -> Informe {
+            sin_uso("estado-puertas")
+        }
+        fn merge_pr(&self, _: u64, _: &str) -> Informe {
+            sin_uso("merge-pr")
+        }
+        fn leer_puertas(&self, _: Rol, _: &str) -> Paso<ReglasLeidas> {
+            Err(sin_uso("leer-puertas"))
+        }
+        fn leer_permisos_ci(&self) -> Informe {
+            sin_uso("leer-permisos-ci")
+        }
+        fn firma_commit(&self, _: &str) -> Paso<Firma> {
+            Err(sin_uso("firma-commit"))
+        }
+        fn identidad(&self, _: Rol) -> Paso<Usuario> {
+            self.llamadas.set(self.llamadas.get() + 1);
+            Ok(self.usuario.clone())
+        }
+    }
+
+    fn carpeta(nombre: &str) -> PathBuf {
+        let d = std::env::temp_dir()
+            .join("tc2-tests")
+            .join(format!("identidad-{nombre}-{}", std::process::id()));
+        let _ = std::fs::remove_dir_all(&d);
+        d
+    }
+
+    const AHORA: u64 = 1_800_000_000;
+
+    /// C6: una cuenta `Bot` (o `web-flow`, o fuera de la lista) nunca acaba
+    /// en `asignado_a`: error tipado. Sin terminal, `Quien` es `Agente` y la
+    /// transicion lleva `decidido_por: agente`; con terminal, `persona`.
+    #[test]
+    fn bot_nunca_asignado_y_sin_tty_es_agente() {
+        let personas = Personas::nuevas([7, 8]);
+        let u = |login: &str, id, tipo: &str| Usuario {
+            login: login.to_owned(),
+            id,
+            tipo: tipo.to_owned(),
+        };
+        let gh = Proveedor::GitHub;
+        assert_eq!(
+            asignable(&u("ci[bot]", 7, "Bot"), &personas, gh),
+            Err(NoAsignable::Bot)
+        );
+        assert_eq!(
+            asignable(&u("web-flow", 7, "User"), &personas, gh),
+            Err(NoAsignable::WebFlow)
+        );
+        assert_eq!(
+            asignable(&u("otra", 9, "User"), &personas, gh),
+            Err(NoAsignable::FueraDeLista)
+        );
+        assert_eq!(
+            asignable(&u("a b", 7, "User"), &personas, gh),
+            Err(NoAsignable::LoginNoValido)
+        );
+        let bot = Falsa::de("ci[bot]", 7, "Bot");
+        let cache = Cache::en(&carpeta("bot"));
+        let r = quien_ejecuta(&bot, gh, &personas, &cache, true, AHORA);
+        assert_eq!(
+            r.err().map(|i| i.linea()).as_deref(),
+            Some("identidad rechazada motivo=identidad rol=persona cuenta=bot")
+        );
+        // Con terminal: persona.
+        let ana = Falsa::de("ana", 7, "User");
+        let q = quien_ejecuta(&ana, gh, &personas, &cache, true, AHORA);
+        assert_eq!(q.as_ref().map(Quien::decidido_por), Ok("persona"));
+        assert_eq!(
+            q.map(|q| q.asignado()),
+            Ok(Asignado::Persona {
+                id: 7,
+                login: "ana".to_owned()
+            })
+        );
+        // Sin terminal, con la cache de ana: agente en su nombre, sin
+        // llamar a la plataforma.
+        let cuenta = Cuenta {
+            proveedor: gh,
+            id: 7,
+            login: "ana".to_owned(),
+        };
+        assert_eq!(cache.guardar(&cuenta, AHORA), Ok(()));
+        let antes = ana.llamadas.get();
+        let q = quien_ejecuta(&ana, gh, &personas, &cache, false, AHORA + 60);
+        assert_eq!(q.as_ref().map(Quien::decidido_por), Ok("agente"));
+        assert_eq!(q.map(|q| q.cuenta().clone()), Ok(cuenta));
+        assert_eq!(ana.llamadas.get(), antes, "sin terminal no hay llamadas");
+        let _ = std::fs::remove_dir_all(carpeta("bot"));
+    }
+
+    /// C7 (biblioteca): cache que falta, caducada, del futuro, de otra
+    /// plataforma o de alguien que ya no esta en la lista: rechazada pidiendo
+    /// `tc2 identidad`. El login sale de la lista si esta (renombrado).
+    #[test]
+    fn cache_vigencia_y_rechazos() -> Result<(), Informe> {
+        let d = carpeta("cache");
+        let cache = Cache::en(&d);
+        let gh = Proveedor::GitHub;
+        let personas = Personas::leer("personas:\n  - {id: 7, login: \"ana-nueva\"}\n")?;
+        let p = Falsa::de("x", 1, "User");
+        let linea = |ahora| {
+            quien_ejecuta(&p, gh, &personas, &cache, false, ahora)
+                .err()
+                .map(|i| i.linea())
+                .unwrap_or_default()
+        };
+        assert_eq!(
+            linea(AHORA),
+            "identidad rechazada motivo=identidad cache=falta"
+        );
+        let ana = Cuenta {
+            proveedor: gh,
+            id: 7,
+            login: "ana".to_owned(),
+        };
+        assert_eq!(cache.guardar(&ana, AHORA), Ok(()));
+        assert!(linea(AHORA + TTL_SEGUNDOS).ends_with("cache=caducada"));
+        assert!(linea(AHORA - MARGEN_RELOJ - 1).ends_with("cache=caducada"));
+        let q = quien_ejecuta(&p, gh, &personas, &cache, false, AHORA + TTL_SEGUNDOS - 1);
+        assert_eq!(
+            q.map(|q| q.cuenta().login.clone()).as_deref(),
+            Ok("ana-nueva")
+        );
+        let otra = quien_ejecuta(&p, Proveedor::GitLab, &personas, &cache, false, AHORA);
+        assert!(
+            otra.err()
+                .is_some_and(|i| i.linea().ends_with("cache=otra-plataforma"))
+        );
+        let fuera = quien_ejecuta(&p, gh, &Personas::nuevas([8]), &cache, false, AHORA);
+        assert!(
+            fuera
+                .err()
+                .is_some_and(|i| i.linea().ends_with("cache=fuera-de-lista"))
+        );
+        // Un fichero que no se entiende cuenta como si no hubiera.
+        assert!(std::fs::write(cache.ruta(), "---\nid: x\n").is_ok());
+        assert!(cache.leer().is_none());
+        assert!(linea(AHORA).ends_with("cache=falta"));
+        let _ = std::fs::remove_dir_all(&d);
+        Ok(())
+    }
+
+    #[test]
+    fn convertir_no_toca_una_persona() -> Result<(), Informe> {
+        let personas =
+            Personas::leer("personas:\n  - {id: 7, login: \"ana\", correos: [\"ana@x.com\"]}\n")?;
+        let p = Asignado::persona(9, "luis");
+        assert_eq!(p.clone().map(|a| convertir_heredado(a, &personas)), p);
+        Ok(())
+    }
+}
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 14b8c7e..a44624b 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -8,7 +8,8 @@
 //! F2-9), [`carril`] (tabla de suelo de F1-10) y [`config`] (configuracion
 //! con esquema validado, F2-7) e [`i18n`] (mensajes en espanol e ingles,
 //! F2-8), [`version`] (`version --json` y `compatible`, F2-12) y
-//! [`actualizar_cli`] (`actualizar --cli`, F2-12). Las capas
+//! [`actualizar_cli`] (`actualizar --cli`, F2-12) e [`identidad`] (quien
+//! ejecuta y `tc2 identidad`, F2-13). Las capas
 //! que vienen entran como modulos de esta biblioteca.
 
 pub mod actualizar_cli;
@@ -20,6 +21,7 @@ pub mod descarga;
 pub mod estado;
 pub mod git;
 pub mod i18n;
+pub mod identidad;
 pub mod plataforma;
 pub mod tarea;
 pub mod version;
@@ -109,6 +111,10 @@ where
             let (json, resto) = con_json(resto);
             actualizar_cli::orden(&resto).salida_en(json, idioma)
         }
+        [orden, resto @ ..] if orden == identidad::NOMBRE => {
+            let (json, resto) = con_json(resto);
+            orden_identidad(&resto).salida_en(json, idioma)
+        }
         [] => Salida {
             codigo: 2,
             stdout: String::new(),
@@ -200,6 +206,30 @@ fn orden_git(orden: &str, args: &[String]) -> Informe {
     }
 }
 
+/// `identidad [--json]` (F2-13) en la carpeta actual, con el terminal y los
+/// tokens del proceso.
+fn orden_identidad(args: &[String]) -> Informe {
+    use std::io::IsTerminal;
+    let dir = match std::env::current_dir() {
+        Ok(d) => d,
+        Err(e) => {
+            return Informe::nuevo(identidad::NOMBRE, Resultado::Error)
+                .diagnostico(Mensaje::nuevo(&txt::CARPETA_ACTUAL).con("error", e.to_string()));
+        }
+    };
+    let g = git::Git::nuevo();
+    identidad::orden(
+        args,
+        identidad::Entorno {
+            dir: &dir,
+            git: &g,
+            tty: std::io::stdin().is_terminal(),
+            credenciales: Box::new(plataforma::CredencialesEntorno),
+            ahora: identidad::ahora(),
+        },
+    )
+}
+
 /// Catalogo de produccion: reconstruye las operaciones de la rama de estado
 /// desde su anotacion. Las operaciones nuevas se anaden aqui.
 fn catalogo(anotacion: &str) -> Option<Box<dyn estado::Operacion>> {
diff --git a/crates/tc2/src/plataforma/mod.rs b/crates/tc2/src/plataforma/mod.rs
index 5e02e82..a6a8fbb 100644
--- a/crates/tc2/src/plataforma/mod.rs
+++ b/crates/tc2/src/plataforma/mod.rs
@@ -18,14 +18,14 @@ pub mod errores;
 pub mod github;
 pub mod gitlab;
 pub mod http;
+pub mod personas;
 pub mod puertas;
 
-use std::collections::BTreeSet;
-
 pub use credenciales::{Credenciales, CredencialesEntorno, CredencialesFijas, Rol, Secreto};
 pub use deteccion::{Destino, detectar};
 pub use github::GitHub;
 pub use gitlab::GitLab;
+pub use personas::{Clave, Persona, Personas};
 pub use puertas::{EstadoCheck, Lecturas, evaluar};
 
 use crate::contrato::Informe;
@@ -61,54 +61,6 @@ impl Proveedor {
     }
 }
 
-/// Lista de personas por id numerico (F1-6, decision 3) y, para GitLab, los
-/// ids de sus claves de firma (F1-13, decision 6: la firma de GitLab no trae
-/// el usuario). La lee F2-13 del fichero de `.taskcode/`; aqui llega ya
-/// leida.
-#[derive(Debug, Clone, Default, PartialEq, Eq)]
-pub struct Personas {
-    ids: BTreeSet<u64>,
-    claves: BTreeSet<Clave>,
-}
-
-/// Clave de firma registrada en la plataforma, con su tipo: los ids de las
-/// claves SSH y GPG de GitLab son espacios distintos y no se comparan entre
-/// si.
-#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
-pub enum Clave {
-    /// Clave SSH (`key.id`).
-    Ssh(u64),
-    /// Clave GPG (`gpg_key_id`).
-    Gpg(u64),
-}
-
-impl Personas {
-    /// Lista con estos ids.
-    pub fn nuevas(ids: impl IntoIterator<Item = u64>) -> Self {
-        Self {
-            ids: ids.into_iter().collect(),
-            claves: BTreeSet::new(),
-        }
-    }
-
-    /// La misma lista con estas claves de firma.
-    #[must_use]
-    pub fn con_claves(mut self, claves: impl IntoIterator<Item = Clave>) -> Self {
-        self.claves = claves.into_iter().collect();
-        self
-    }
-
-    /// Si el id es de una persona de la lista.
-    pub fn contiene(&self, id: u64) -> bool {
-        self.ids.contains(&id)
-    }
-
-    /// Si la clave de firma es de una persona de la lista.
-    pub fn contiene_clave(&self, clave: Clave) -> bool {
-        self.claves.contains(&clave)
-    }
-}
-
 /// Cuenta de la plataforma.
 #[derive(Debug, Clone, PartialEq, Eq)]
 pub struct Usuario {
diff --git a/crates/tc2/src/plataforma/personas.rs b/crates/tc2/src/plataforma/personas.rs
new file mode 100644
index 0000000..2aff828
--- /dev/null
+++ b/crates/tc2/src/plataforma/personas.rs
@@ -0,0 +1,343 @@
+//! Lista de personas (F1-6, decision 3; F2-13): el fichero
+//! `.taskcode/personas.yml` de la rama de codigo, bajo CODEOWNERS.
+//!
+//! Cada entrada guarda el `id` numerico de la cuenta (lo que se compara) y su
+//! `login` (lo que se ensena). `nombre` es opcional y solo acompana al login;
+//! `correos` son los textos libres de `asignado_a` de la 0.16 que se
+//! convierten a esta persona (migracion, F2-13 D5); `claves_ssh` y
+//! `claves_gpg`, los ids de sus claves de firma en GitLab (F1-13, decision
+//! 6). Aqui solo se lee y se valida el texto: sin E/S.
+
+use std::collections::BTreeSet;
+
+use serde::Deserialize;
+
+use crate::contrato::{Informe, Resultado};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::identidad as t;
+
+/// Ruta de la lista en la rama de codigo.
+pub const RUTA: &str = ".taskcode/personas.yml";
+
+/// Operacion en los errores de lectura.
+const OP: &str = "personas";
+
+/// Clave de firma registrada en la plataforma, con su tipo: los ids de las
+/// claves SSH y GPG de GitLab son espacios distintos y no se comparan entre
+/// si.
+#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
+pub enum Clave {
+    /// Clave SSH (`key.id`).
+    Ssh(u64),
+    /// Clave GPG (`gpg_key_id`).
+    Gpg(u64),
+}
+
+/// Una persona de la lista.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Persona {
+    /// Id numerico de la cuenta: el que se compara.
+    pub id: u64,
+    /// Login de la cuenta: el que se ensena.
+    pub login: String,
+    /// Nombre para acompanar al login; nunca lo sustituye.
+    pub nombre: Option<String>,
+    /// Textos de `asignado_a` de la 0.16 que son esta persona.
+    pub correos: Vec<String>,
+}
+
+impl Persona {
+    /// Como se ensena: siempre el login, y el nombre al lado si lo hay.
+    pub fn mostrar(&self) -> String {
+        match &self.nombre {
+            Some(n) => format!("{} ({n})", self.login),
+            None => self.login.clone(),
+        }
+    }
+}
+
+/// Lista de personas por id numerico (F1-6, decision 3) y, para GitLab, los
+/// ids de sus claves de firma (F1-13, decision 6: la firma de GitLab no trae
+/// el usuario).
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct Personas {
+    ids: BTreeSet<u64>,
+    claves: BTreeSet<Clave>,
+    entradas: Vec<Persona>,
+}
+
+#[derive(Deserialize)]
+#[serde(deny_unknown_fields)]
+struct ListaCruda {
+    personas: Vec<EntradaCruda>,
+}
+
+#[derive(Deserialize)]
+#[serde(deny_unknown_fields)]
+struct EntradaCruda {
+    id: u64,
+    login: String,
+    #[serde(default)]
+    nombre: Option<String>,
+    #[serde(default)]
+    correos: Vec<String>,
+    #[serde(default)]
+    claves_ssh: Vec<u64>,
+    #[serde(default)]
+    claves_gpg: Vec<u64>,
+}
+
+/// Sin espacios ni caracteres de control, y no vacio.
+fn palabra_valida(s: &str) -> bool {
+    !s.is_empty()
+        && !s
+            .chars()
+            .any(|c| c.is_control() || c.is_whitespace() || matches!(c, '\u{2028}' | '\u{2029}'))
+}
+
+/// Rechazo de la lista con su diagnostico.
+fn rechazo(m: Mensaje) -> Informe {
+    Informe::nuevo(OP, Resultado::Rechazada)
+        .con("motivo", "personas")
+        .diagnostico(m)
+}
+
+/// Una entrada para los diagnosticos: su numero (desde 1), login e id.
+fn cita(n: usize, p: &Persona) -> String {
+    format!("{n} ({}, id {})", p.login, p.id)
+}
+
+impl Personas {
+    /// Lista con estos ids (sin login: para tests y arneses que solo
+    /// comparan ids).
+    pub fn nuevas(ids: impl IntoIterator<Item = u64>) -> Self {
+        Self {
+            ids: ids.into_iter().collect(),
+            claves: BTreeSet::new(),
+            entradas: Vec::new(),
+        }
+    }
+
+    /// La misma lista con estas claves de firma.
+    #[must_use]
+    pub fn con_claves(mut self, claves: impl IntoIterator<Item = Clave>) -> Self {
+        self.claves = claves.into_iter().collect();
+        self
+    }
+
+    /// Lee y valida el texto de `.taskcode/personas.yml`: `id` mayor que 0 y
+    /// unico, `login` unico sin distinguir mayusculas, `correos` unicos. Un
+    /// repetido se rechaza nombrando las dos entradas.
+    pub fn leer(texto: &str) -> Result<Self, Informe> {
+        let cruda: ListaCruda = serde_saphyr::from_str_with_options(texto, crate::yaml::opciones())
+            .map_err(|e| {
+                rechazo(
+                    Mensaje::nuevo(&t::PERSONAS_YAML)
+                        .con("ruta", RUTA)
+                        .con("error", e.to_string()),
+                )
+            })?;
+        let mut lista = Self::default();
+        for (i, e) in cruda.personas.into_iter().enumerate() {
+            let n = i + 1;
+            let entrada = |que: &'static crate::i18n::Texto| {
+                rechazo(
+                    Mensaje::nuevo(&t::PERSONAS_ENTRADA)
+                        .con("ruta", RUTA)
+                        .con("n", n.to_string())
+                        .con_mensaje("que", que),
+                )
+            };
+            if e.id == 0 {
+                return Err(entrada(&t::ID_CERO));
+            }
+            if !palabra_valida(&e.login) {
+                return Err(entrada(&t::LOGIN_NO_VALIDO));
+            }
+            if e.nombre
+                .as_deref()
+                .is_some_and(|s| s.trim().is_empty() || s.chars().any(char::is_control))
+            {
+                return Err(entrada(&t::NOMBRE_NO_VALIDO));
+            }
+            if !e.correos.iter().all(|c| palabra_valida(c)) {
+                return Err(entrada(&t::CORREO_NO_VALIDO));
+            }
+            let nueva = Persona {
+                id: e.id,
+                login: e.login,
+                nombre: e.nombre,
+                correos: e.correos,
+            };
+            for (j, otra) in lista.entradas.iter().enumerate() {
+                let (a, b) = (cita(j + 1, otra), cita(n, &nueva));
+                if otra.id == nueva.id {
+                    return Err(rechazo(
+                        Mensaje::nuevo(&t::ID_REPETIDO)
+                            .con("ruta", RUTA)
+                            .con("a", a)
+                            .con("b", b)
+                            .con("id", nueva.id.to_string()),
+                    ));
+                }
+                if otra.login.to_lowercase() == nueva.login.to_lowercase() {
+                    return Err(rechazo(
+                        Mensaje::nuevo(&t::LOGIN_REPETIDO)
+                            .con("ruta", RUTA)
+                            .con("a", a)
+                            .con("b", b),
+                    ));
+                }
+                if let Some(c) = nueva
+                    .correos
+                    .iter()
+                    .find(|c| otra.correos.iter().any(|o| o.eq_ignore_ascii_case(c)))
+                {
+                    return Err(rechazo(
+                        Mensaje::nuevo(&t::CORREO_REPETIDO)
+                            .con("ruta", RUTA)
+                            .con("a", a)
+                            .con("b", b)
+                            .con("correo", c.clone()),
+                    ));
+                }
+            }
+            lista.ids.insert(nueva.id);
+            lista
+                .claves
+                .extend(e.claves_ssh.into_iter().map(Clave::Ssh));
+            lista
+                .claves
+                .extend(e.claves_gpg.into_iter().map(Clave::Gpg));
+            lista.entradas.push(nueva);
+        }
+        Ok(lista)
+    }
+
+    /// Si el id es de una persona de la lista.
+    pub fn contiene(&self, id: u64) -> bool {
+        self.ids.contains(&id)
+    }
+
+    /// Si la clave de firma es de una persona de la lista.
+    pub fn contiene_clave(&self, clave: Clave) -> bool {
+        self.claves.contains(&clave)
+    }
+
+    /// La persona de ese id, con su login de la lista (la lista es la
+    /// autoridad que refresca un login renombrado).
+    pub fn por_id(&self, id: u64) -> Option<&Persona> {
+        self.entradas.iter().find(|p| p.id == id)
+    }
+
+    /// La persona que declara ese correo de la 0.16 (sin distinguir
+    /// mayusculas).
+    pub fn por_correo(&self, correo: &str) -> Option<&Persona> {
+        let correo = correo.trim();
+        self.entradas
+            .iter()
+            .find(|p| p.correos.iter().any(|c| c.eq_ignore_ascii_case(correo)))
+    }
+
+    /// Las entradas, en el orden del fichero.
+    pub fn entradas(&self) -> &[Persona] {
+        &self.entradas
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::i18n::Idioma;
+
+    fn error(texto: &str) -> String {
+        Personas::leer(texto)
+            .err()
+            .and_then(|i| i.diagnostico_en(Idioma::Es))
+            .unwrap_or_default()
+    }
+
+    /// C3: un `id` repetido, o un `login` repetido sin distinguir
+    /// mayusculas, se rechaza nombrando las dos entradas.
+    #[test]
+    fn id_o_login_repetido_se_rechaza_nombrando_las_dos_entradas() {
+        let id = "personas:\n  - {id: 7, login: \"ana\"}\n  - {id: 8, login: \"luis\"}\n  - {id: 7, login: \"otra\"}\n";
+        let e = error(id);
+        assert!(
+            e.contains("1 (ana, id 7)") && e.contains("3 (otra, id 7)") && e.contains("id 7"),
+            "{e}"
+        );
+        let login = "personas:\n  - {id: 7, login: \"Ana\"}\n  - {id: 9, login: \"aNA\"}\n";
+        let e = error(login);
+        assert!(
+            e.contains("1 (Ana, id 7)") && e.contains("2 (aNA, id 9)"),
+            "{e}"
+        );
+        let i = Personas::leer(login).err();
+        assert_eq!(
+            i.map(|i| i.linea()).as_deref(),
+            Some("personas rechazada motivo=personas")
+        );
+        let correo = "personas:\n  - {id: 7, login: \"a\", correos: [\"x@y\"]}\n  - {id: 9, login: \"b\", correos: [\"X@Y\"]}\n";
+        assert!(error(correo).contains("x@y") || error(correo).contains("X@Y"));
+    }
+
+    #[test]
+    fn entradas_mal_formadas_se_rechazan() {
+        for (texto, pista) in [
+            ("personas:\n  - {id: 0, login: \"a\"}\n", "mayor que 0"),
+            ("personas:\n  - {id: 1, login: \"\"}\n", "login"),
+            ("personas:\n  - {id: 1, login: \"a b\"}\n", "login"),
+            (
+                "personas:\n  - {id: 1, login: \"a\", nombre: \"\"}\n",
+                "nombre",
+            ),
+            (
+                "personas:\n  - {id: 1, login: \"a\", correos: [\"\"]}\n",
+                "correos",
+            ),
+            ("personas:\n  - {id: 1, login: \"a\", rol: \"x\"}\n", "YAML"),
+            ("personas:\n  - {id: \"uno\", login: \"a\"}\n", "YAML"),
+            ("otra: 1\n", "YAML"),
+            (
+                "personas:\n  - {id: 1, login: \"a\"}\n  - {id: 1, login: \"a\"}\n",
+                "id 1",
+            ),
+        ] {
+            let e = error(texto);
+            assert!(e.contains(pista), "{texto:?}: {e}");
+        }
+    }
+
+    /// C4: en pantalla sale siempre el login; el nombre lo acompana y nunca
+    /// lo sustituye. Dos personas con el mismo nombre se distinguen por login
+    /// e id.
+    #[test]
+    fn se_ensena_el_login_y_el_nombre_no_lo_sustituye() -> Result<(), Informe> {
+        let l = Personas::leer(
+            "personas:\n  - {id: 7, login: \"ana-g\", nombre: \"Ana\"}\n  - {id: 8, login: \"ana-p\", nombre: \"Ana\"}\n  - {id: 9, login: \"luis\"}\n",
+        )?;
+        let (a, b) = (l.por_id(7), l.por_id(8));
+        assert_eq!(a.map(Persona::mostrar).as_deref(), Some("ana-g (Ana)"));
+        assert_eq!(b.map(Persona::mostrar).as_deref(), Some("ana-p (Ana)"));
+        assert_ne!(a.map(|p| p.id), b.map(|p| p.id));
+        assert_eq!(l.por_id(9).map(Persona::mostrar).as_deref(), Some("luis"));
+        for p in l.entradas() {
+            assert!(p.mostrar().starts_with(&p.login));
+        }
+        assert!(l.contiene(8) && !l.contiene(10));
+        Ok(())
+    }
+
+    #[test]
+    fn correos_y_claves() -> Result<(), Informe> {
+        let l = Personas::leer(
+            "personas:\n  - id: 7\n    login: \"ana\"\n    correos: [\"Ana@X.com\"]\n    claves_ssh: [3]\n    claves_gpg: [4]\n",
+        )?;
+        assert_eq!(l.por_correo(" ana@x.com").map(|p| p.id), Some(7));
+        assert!(l.por_correo("otra@x.com").is_none());
+        assert!(l.contiene_clave(Clave::Ssh(3)) && l.contiene_clave(Clave::Gpg(4)));
+        assert!(!l.contiene_clave(Clave::Ssh(4)));
+        Ok(())
+    }
+}
diff --git a/crates/tc2/src/tarea/mod.rs b/crates/tc2/src/tarea/mod.rs
index 39a1402..96ae2b7 100644
--- a/crates/tc2/src/tarea/mod.rs
+++ b/crates/tc2/src/tarea/mod.rs
@@ -13,14 +13,14 @@
 mod cuerpo;
 mod modelo;
 pub mod texto;
-mod yaml;
+pub(crate) mod yaml;
 
 use std::fmt;
 
 pub use cuerpo::{Cuerpo, Seccion};
 pub use modelo::{
-    COMPLEJIDADES, ESTADOS, REGLAS, Rol, RolRevision, Roles, TIPOS, Tarea, TareaV1, id_valido,
-    titulo_valido,
+    Asignado, COMPLEJIDADES, ESTADOS, Heredado, REGLAS, Rol, RolRevision, Roles, TIPOS, Tarea,
+    TareaV1, id_valido, titulo_valido,
 };
 
 /// Un `tarea.md` leido.
diff --git a/crates/tc2/src/tarea/modelo.rs b/crates/tc2/src/tarea/modelo.rs
index db129a0..10a3c37 100644
--- a/crates/tc2/src/tarea/modelo.rs
+++ b/crates/tc2/src/tarea/modelo.rs
@@ -121,6 +121,138 @@ fn fecha_valida(f: &str) -> bool {
         })
 }
 
+/// Texto libre de un `asignado_a` de la 0.16 que no se ha convertido a una
+/// persona de la plataforma. Solo lo construye el lector: la 2.0 nunca
+/// asigna un texto, y al reescribir una tarea lo conserva tal cual hasta que
+/// `reasignar` (F3-10) lo resuelva.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Heredado(String);
+
+impl Heredado {
+    /// El texto tal como se leyo.
+    pub fn texto(&self) -> &str {
+        &self.0
+    }
+}
+
+/// Quien lleva la tarea (F2-13): la cuenta de la plataforma por su `id`
+/// numerico, con el `login` al lado para ensenarlo, o el texto heredado de la
+/// 0.16.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum Asignado {
+    /// Cuenta de la plataforma. Se compara por `id`; `login` es una copia
+    /// para mostrar que refresca la lista de personas.
+    Persona {
+        /// Id numerico de la cuenta.
+        id: u64,
+        /// Login de la cuenta.
+        login: String,
+    },
+    /// Texto libre de la 0.16 (solo lo crea el lector).
+    Heredado(Heredado),
+}
+
+impl Asignado {
+    /// Cuenta `id` con `login`; `None` si el id es 0 o el login esta vacio o
+    /// lleva espacios o caracteres de control.
+    pub fn persona(id: u64, login: &str) -> Option<Self> {
+        (id > 0 && login_valido(login)).then(|| Self::Persona {
+            id,
+            login: login.to_owned(),
+        })
+    }
+
+    /// Id de la cuenta, si es una persona.
+    pub fn id(&self) -> Option<u64> {
+        match self {
+            Self::Persona { id, .. } => Some(*id),
+            Self::Heredado(_) => None,
+        }
+    }
+
+    /// Login de la cuenta, si es una persona.
+    pub fn login(&self) -> Option<&str> {
+        match self {
+            Self::Persona { login, .. } => Some(login),
+            Self::Heredado(_) => None,
+        }
+    }
+
+    /// Texto heredado de la 0.16, si lo es.
+    pub fn heredado(&self) -> Option<&str> {
+        match self {
+            Self::Persona { .. } => None,
+            Self::Heredado(h) => Some(h.texto()),
+        }
+    }
+
+    /// «Ya la lleva»: la cuenta `id` es la asignada. Compara solo el `id`: el
+    /// mismo login con otro id es otra persona y un login renombrado con el
+    /// mismo id es la misma. Un texto heredado no lo lleva nadie.
+    pub fn la_lleva(&self, id: u64) -> bool {
+        self.id() == Some(id)
+    }
+
+    /// Si `otro` es la misma persona (mismo `id`).
+    pub fn misma_persona(&self, otro: &Self) -> bool {
+        otro.id().is_some_and(|id| self.la_lleva(id))
+    }
+
+    /// Lo que se escribe en el frontmatter.
+    fn yaml(&self) -> String {
+        match self {
+            Self::Persona { id, login } => {
+                format!("{{id: {id}, login: {}}}", yaml::cadena(login))
+            }
+            Self::Heredado(h) => yaml::cadena(h.texto()),
+        }
+    }
+}
+
+/// Login de una cuenta: no vacio, sin espacios ni caracteres de control.
+fn login_valido(login: &str) -> bool {
+    !login.is_empty() && texto_valido(login) && !login.chars().any(char::is_whitespace)
+}
+
+/// `asignado_a` del formato 2: `null`, `{id, login}` o un texto heredado.
+fn asignado(v: Option<serde_json::Value>) -> Result<Option<Asignado>, ErrorTarea> {
+    use serde_json::Value;
+    const CLAVE: &str = "asignado_a";
+    match v {
+        None | Some(Value::Null) => Ok(None),
+        Some(Value::String(s)) if !s.trim().is_empty() => {
+            Ok(Some(Asignado::Heredado(Heredado(texto(CLAVE, s)?))))
+        }
+        Some(Value::Object(m)) => {
+            if let Some(k) = m.keys().find(|k| *k != "id" && *k != "login") {
+                return Err(ErrorTarea::en(
+                    CLAVE,
+                    format!("clave «{k}» desconocida (se espera {{id, login}})"),
+                ));
+            }
+            let id = m.get("id").and_then(Value::as_u64);
+            let login = m.get("login").and_then(Value::as_str);
+            match (id, login) {
+                (Some(id), Some(login)) => Asignado::persona(id, login).ok_or_else(|| {
+                    ErrorTarea::en(
+                        CLAVE,
+                        "`id` tiene que ser mayor que 0 y `login` sin espacios ni caracteres de control",
+                    )
+                }),
+                _ => Err(ErrorTarea::en(
+                    CLAVE,
+                    "se espera {id: <entero sin comillas>, login: \"<login>\"}",
+                )),
+            }
+            .map(Some)
+        }
+        Some(_) => Err(ErrorTarea::en(
+            CLAVE,
+            "se espera null, {id, login} o el texto de la 0.16",
+        )),
+    }
+}
+
 /// Un rol de `roles` (diseno, implementacion).
 #[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
 #[serde(deny_unknown_fields)]
@@ -174,7 +306,7 @@ pub struct Tarea {
     pub motivo: Option<String>,
     pub plan_aprobado: bool,
     pub rama: Option<String>,
-    pub asignado_a: Option<String>,
+    pub asignado_a: Option<Asignado>,
     pub roles: Roles,
     /// Entrada `estimacion:` cruda (su forma la fija F1-17); `None` es `null`.
     /// Privada: solo llega de una lectura, que la ha validado como YAML.
@@ -205,7 +337,7 @@ struct CrudoV2 {
     motivo: Option<String>,
     plan_aprobado: Option<bool>,
     rama: Option<String>,
-    asignado_a: Option<String>,
+    asignado_a: Option<serde_json::Value>,
     roles: Option<Roles>,
     estimacion: Option<IgnoredAny>,
     tokens_diseno: Option<u64>,
@@ -412,7 +544,7 @@ impl Tarea {
             motivo,
             plan_aprobado: exigir("plan_aprobado", c.plan_aprobado)?,
             rama: texto_opcional("rama", c.rama)?,
-            asignado_a: texto_opcional("asignado_a", c.asignado_a)?,
+            asignado_a: asignado(c.asignado_a)?,
             roles: Roles {
                 diseno: rol("roles.diseno", roles.diseno)?,
                 implementacion: rol("roles.implementacion", roles.implementacion)?,
@@ -471,7 +603,12 @@ impl Tarea {
         linea("motivo", yaml::opcional(self.motivo.as_deref()));
         linea("plan_aprobado", self.plan_aprobado.to_string());
         linea("rama", yaml::opcional(self.rama.as_deref()));
-        linea("asignado_a", yaml::opcional(self.asignado_a.as_deref()));
+        linea(
+            "asignado_a",
+            self.asignado_a
+                .as_ref()
+                .map_or_else(|| "null".to_owned(), Asignado::yaml),
+        );
         let rv = &r.revision;
         let revision = format!(
             "{{agente: {}, modelo: {}, skills: {}, regla: {}}}",
@@ -530,7 +667,8 @@ pub struct TareaV1 {
     pub estado: String,
     pub plan_aprobado: bool,
     pub rama: String,
-    pub asignado_a: Option<String>,
+    /// El texto libre de la 0.16, siempre [`Asignado::Heredado`].
+    pub asignado_a: Option<Asignado>,
     pub agente_revisor: String,
     pub skills_recomendados: Vec<String>,
     pub regla_seleccion_skill: Option<String>,
@@ -684,7 +822,9 @@ impl TareaV1 {
             estado: uno_de("estado", exigir("estado", c.estado)?, &ESTADOS_V1)?,
             plan_aprobado: exigir("plan_aprobado", c.plan_aprobado)?,
             rama: texto("rama", exigir("rama", c.rama)?)?,
-            asignado_a: texto_opcional("asignado_a", c.asignado_a)?,
+            asignado_a: texto_opcional("asignado_a", c.asignado_a)?
+                .filter(|s| !s.trim().is_empty())
+                .map(|s| Asignado::Heredado(Heredado(s))),
             agente_revisor: texto(
                 "agente_revisor",
                 exigir("agente_revisor", c.agente_revisor)?,
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-077/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-077/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-077/tarea.md                                    | 12 +++++++++---
 3 files changed, 9 insertions(+), 3 deletions(-)
````

## Tests del diff (2 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/identidad.rs" ":(literal)crates/tc2/tests/tarea.rs"
````

````
crates/tc2/tests/identidad.rs | 376 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/tarea.rs     | 105 ++++++++++++++++++++++++++++++++++++++++++++-
 2 files changed, 479 insertions(+), 2 deletions(-)
````
