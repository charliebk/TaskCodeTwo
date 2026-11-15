# Peticion de revision — TASK-021 (ronda 1)

- Tarea: TASK-021 — F2-6 Adaptador GitHub
- Rama revisada: feature/task-021-f2-6-adaptador-github
- Rama base: develop
- Commit revisado (HEAD): aa704e87c860ed49942176ea8c711ba40a83e567
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-021 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-021/tarea.md
- tareas/03-en-revision/TASK-021/planificacion/
- crates/tc2/Cargo.toml
- crates/tc2/src/lib.rs
- crates/tc2/src/plataforma/credenciales.rs
- crates/tc2/src/plataforma/deteccion.rs
- crates/tc2/src/plataforma/errores.rs
- crates/tc2/src/plataforma/github.rs
- crates/tc2/src/plataforma/http.rs
- crates/tc2/src/plataforma/mod.rs
- crates/tc2/src/plataforma/puertas.rs
- crates/tc2/tests/comun/http_mock.rs
- crates/tc2/tests/comun/mod.rs
- crates/tc2/tests/plataforma.rs
- crates/tc2/tests/plataforma_e2e.rs

## Commits a revisar (git log develop..HEAD)

````
aa704e8 chore(TASK-021): constancia de autorrevision
cd3479a test(TASK-021): paginacion de reviews y ultima review por persona
35ba1d7 test(TASK-021): E2E contra el repo de pruebas de GitHub y formas observadas
7638221 test(TASK-021): bateria de la PoC M, envoltorio y errores contra un servidor simulado
167ca23 feat(TASK-021): puerto Plataforma, envoltorio HTTP y adaptador GitHub
daa1160 chore(TASK-021): tarea en curso
````

## Diff de tu dominio (13 fichero(s) de 20; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/Cargo.toml b/crates/tc2/Cargo.toml
index 5e341ec..3b70158 100644
--- a/crates/tc2/Cargo.toml
+++ b/crates/tc2/Cargo.toml
@@ -21,6 +21,12 @@ workspace = true
 [dependencies]
 serde = { version = "1", features = ["derive"] }
 serde-saphyr = { version = "2.1", default-features = false, features = ["deserialize"] }
+# F2-6: adaptador GitHub por la API REST (F1-13 decision 1, F1-14): cliente
+# HTTP bloqueante con TLS del almacen del sistema (rustls-platform-verifier
+# por la feature `platform-verifier`), sin `gh`. Todo detras de
+# `plataforma::http`.
+serde_json = "1"
+ureq = { version = "3.5", default-features = false, features = ["rustls", "platform-verifier"] }
 
 # Parser YAML de otra base de codigo (port de libyaml, como el PyYAML del PoC
 # de F1-9) para la ida y vuelta de los tests: con el de produccion seria
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 0462abc..8a0cf4f 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -3,13 +3,15 @@
 //! Modulos: [`contrato`] (salida de toda operacion, ADR F1-4), [`git`]
 //! (capa de operaciones Git deterministas, F2-2) y [`estado`] (rama de
 //! estado `taskcode/tareas`, F2-3), [`alta`] (alta de tareas con reserva de
-//! ID, F2-4) y [`tarea`] (modelo de tarea y frontmatter YAML, F2-5). Las capas
+//! ID, F2-4) [`tarea`] (modelo de tarea y frontmatter YAML, F2-5) y
+//! [`plataforma`] (puerto Plataforma y adaptador GitHub, F2-6). Las capas
 //! que vienen entran como modulos de esta biblioteca.
 
 pub mod alta;
 pub mod contrato;
 pub mod estado;
 pub mod git;
+pub mod plataforma;
 pub mod tarea;
 
 use contrato::{Informe, Resultado};
diff --git a/crates/tc2/src/plataforma/credenciales.rs b/crates/tc2/src/plataforma/credenciales.rs
new file mode 100644
index 0000000..c18cd99
--- /dev/null
+++ b/crates/tc2/src/plataforma/credenciales.rs
@@ -0,0 +1,127 @@
+//! Credenciales por rol (ADR F1-6, decisiones 1 y 5).
+//!
+//! Cada operacion del puerto declara con que [`Rol`] corre y pide su token a
+//! un proveedor de [`Credenciales`]. Si falta, la operacion sale `rechazada
+//! motivo=identidad` y nunca prueba con otro rol.
+//!
+//! Hasta que F6-1 cree las Apps, el proveedor de produccion lee variables de
+//! entorno; el token de persona solo se usa con un terminal interactivo.
+
+use std::collections::BTreeMap;
+use std::io::IsTerminal;
+
+/// Identidad con la que corre una operacion (F1-6).
+#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub enum Rol {
+    /// El agente implementador.
+    Agente,
+    /// El revisor IA.
+    Revisor,
+    /// La persona.
+    Persona,
+}
+
+impl Rol {
+    /// Nombre en la linea de contrato.
+    pub fn nombre(self) -> &'static str {
+        match self {
+            Self::Agente => "agente",
+            Self::Revisor => "revisor",
+            Self::Persona => "persona",
+        }
+    }
+
+    /// Variable de entorno de su token.
+    pub fn variable(self) -> &'static str {
+        match self {
+            Self::Agente => "TC2_TOKEN_AGENTE",
+            Self::Revisor => "TC2_TOKEN_REVISOR",
+            Self::Persona => "TC2_TOKEN_PERSONA",
+        }
+    }
+}
+
+/// Un token. No tiene `Display` y su `Debug` no lo ensena.
+#[derive(Clone, PartialEq, Eq)]
+pub struct Secreto(String);
+
+impl Secreto {
+    /// Envuelve un token.
+    pub fn nuevo(token: impl Into<String>) -> Self {
+        Self(token.into())
+    }
+
+    /// El valor, solo para la cabecera `Authorization`.
+    pub(crate) fn exponer(&self) -> &str {
+        &self.0
+    }
+}
+
+impl std::fmt::Debug for Secreto {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        f.write_str("Secreto(***)")
+    }
+}
+
+/// Proveedor de tokens por rol.
+pub trait Credenciales {
+    /// Token de ese rol, si lo hay.
+    fn token(&self, rol: Rol) -> Option<Secreto>;
+}
+
+/// Tokens de las variables `TC2_TOKEN_<ROL>`. El de persona solo si la
+/// entrada estandar es un terminal (F1-6 decision 5: token de persona solo
+/// con TTY).
+#[derive(Debug, Clone, Copy, Default)]
+pub struct CredencialesEntorno;
+
+impl Credenciales for CredencialesEntorno {
+    fn token(&self, rol: Rol) -> Option<Secreto> {
+        if rol == Rol::Persona && !std::io::stdin().is_terminal() {
+            return None;
+        }
+        std::env::var(rol.variable())
+            .ok()
+            .filter(|t| !t.trim().is_empty())
+            .map(Secreto::nuevo)
+    }
+}
+
+/// Tokens fijos (tests y arneses E2E).
+#[derive(Debug, Clone, Default)]
+pub struct CredencialesFijas(BTreeMap<Rol, Secreto>);
+
+impl CredencialesFijas {
+    /// Sin ningun token.
+    pub fn nuevas() -> Self {
+        Self::default()
+    }
+
+    /// Anade el token de un rol.
+    #[must_use]
+    pub fn con(mut self, rol: Rol, token: Secreto) -> Self {
+        self.0.insert(rol, token);
+        self
+    }
+}
+
+impl Credenciales for CredencialesFijas {
+    fn token(&self, rol: Rol) -> Option<Secreto> {
+        self.0.get(&rol).cloned()
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn el_debug_no_ensena_el_token() {
+        let s = Secreto::nuevo("ghp_supersecreto");
+        assert_eq!(format!("{s:?}"), "Secreto(***)");
+        let c = CredencialesFijas::nuevas().con(Rol::Agente, s);
+        assert!(!format!("{c:?}").contains("supersecreto"));
+        assert!(c.token(Rol::Persona).is_none());
+        assert!(c.token(Rol::Agente).is_some());
+    }
+}
diff --git a/crates/tc2/src/plataforma/deteccion.rs b/crates/tc2/src/plataforma/deteccion.rs
new file mode 100644
index 0000000..ea4bf8c
--- /dev/null
+++ b/crates/tc2/src/plataforma/deteccion.rs
@@ -0,0 +1,209 @@
+//! Deteccion de la plataforma por el host de `origin` (F1-13, decision 7,
+//! D8a): `github.com` es GitHub y `gitlab.com` es GitLab; otro host exige la
+//! clave `plataforma` con la URL de la API, y si falta o contradice el host
+//! se falla cerrado.
+//!
+//! Quien llama obtiene la URL con `git ls-remote --get-url origin` y la clave
+//! de `.taskcode/config.yml` (F3-1); aqui llegan como parametros.
+
+use super::http::Base;
+use crate::contrato::{Informe, Resultado};
+
+/// Nombre de la operacion en los errores.
+const OP: &str = "detectar-plataforma";
+
+/// Plataforma soportada.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Proveedor {
+    /// GitHub.
+    GitHub,
+    /// GitLab.
+    GitLab,
+}
+
+impl Proveedor {
+    /// Nombre en la configuracion y en la linea.
+    pub fn nombre(self) -> &'static str {
+        match self {
+            Self::GitHub => "github",
+            Self::GitLab => "gitlab",
+        }
+    }
+}
+
+/// A donde habla el puerto.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Destino {
+    /// Plataforma.
+    pub proveedor: Proveedor,
+    /// `dueno/repo` (GitHub) o `grupo/.../proyecto` (GitLab).
+    pub proyecto: String,
+    /// Base de la API.
+    pub api: Base,
+}
+
+/// Detecta la plataforma. `clave` es `(plataforma, url_api)` de la
+/// configuracion, si esta.
+pub fn detectar(url_origin: &str, clave: Option<(&str, &str)>) -> Result<Destino, Informe> {
+    let (host, proyecto) = partir_url(url_origin).ok_or_else(|| {
+        Informe::nuevo(OP, Resultado::Rechazada)
+            .con("motivo", "origin")
+            .diagnostico(
+                "no se entiende la URL de origin: tiene que ser https, ssh o scp con dueno y repo",
+            )
+    })?;
+    let por_host = match host.as_str() {
+        "github.com" => Some(Proveedor::GitHub),
+        "gitlab.com" => Some(Proveedor::GitLab),
+        _ => None,
+    };
+    let declarado = match clave {
+        None => None,
+        Some((p, url)) => {
+            let prov = match p {
+                "github" => Proveedor::GitHub,
+                "gitlab" => Proveedor::GitLab,
+                _ => {
+                    return Err(rechazo("la clave plataforma vale github o gitlab"));
+                }
+            };
+            let base = Base::nueva(url).map_err(|m| rechazo(&format!("URL de la API: {m}")))?;
+            Some((prov, base))
+        }
+    };
+    let (proveedor, api) = match (por_host, declarado) {
+        (Some(h), Some((d, _))) if h != d => {
+            return Err(rechazo(&format!(
+                "la clave plataforma dice {} y origin es de {host}",
+                d.nombre()
+            )));
+        }
+        (Some(h), Some((_, base))) => (h, base),
+        (Some(Proveedor::GitHub), None) => (Proveedor::GitHub, Base::github()),
+        (Some(Proveedor::GitLab), None) => (
+            Proveedor::GitLab,
+            Base::nueva("https://gitlab.com/api/v4").map_err(|m| rechazo(&m))?,
+        ),
+        (None, Some((d, base))) => (d, base),
+        (None, None) => {
+            return Err(rechazo(&format!(
+                "origin es de {host}: declara plataforma (github o gitlab) y la URL de su API en .taskcode/config.yml"
+            )));
+        }
+    };
+    let segmentos = proyecto.split('/').count();
+    if proveedor == Proveedor::GitHub && segmentos != 2 {
+        return Err(rechazo("en GitHub origin tiene que ser dueno/repo"));
+    }
+    Ok(Destino {
+        proveedor,
+        proyecto,
+        api,
+    })
+}
+
+fn rechazo(m: &str) -> Informe {
+    Informe::nuevo(OP, Resultado::Rechazada)
+        .con("motivo", "plataforma")
+        .diagnostico(m.to_owned())
+}
+
+/// `(host, proyecto)` de una URL https, ssh o scp, sin `.git` ni barras.
+fn partir_url(url: &str) -> Option<(String, String)> {
+    let url = url.trim();
+    let (host, ruta) = if let Some(r) = url
+        .strip_prefix("https://")
+        .or_else(|| url.strip_prefix("ssh://"))
+    {
+        let (autoridad, ruta) = r.split_once('/')?;
+        let host = autoridad.rsplit('@').next()?;
+        let host = host.split(':').next()?;
+        (host, ruta)
+    } else if !url.contains("://") {
+        let (izq, ruta) = url.split_once(':')?;
+        let host = izq.rsplit('@').next()?;
+        (host, ruta)
+    } else {
+        return None;
+    };
+    let ruta = ruta.trim_matches('/');
+    let ruta = ruta.strip_suffix(".git").unwrap_or(ruta);
+    let valido = |s: &str| {
+        !s.is_empty()
+            && s != "."
+            && s != ".."
+            && s.bytes()
+                .all(|b| b.is_ascii_alphanumeric() || b"-._".contains(&b))
+    };
+    let host_ok = !host.is_empty()
+        && host
+            .bytes()
+            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-');
+    if !host_ok || !ruta.split('/').all(valido) || !ruta.contains('/') {
+        return None;
+    }
+    Some((host.to_ascii_lowercase(), ruta.to_owned()))
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn por_host_y_por_clave() {
+        for url in [
+            "https://github.com/charliebk/TaskCodeTwo.git",
+            "git@github.com:charliebk/TaskCodeTwo.git",
+            "ssh://git@github.com/charliebk/TaskCodeTwo",
+        ] {
+            let d = detectar(url, None).ok();
+            assert_eq!(
+                d.map(|d| (d.proveedor, d.proyecto, d.api.texto().to_owned())),
+                Some((
+                    Proveedor::GitHub,
+                    "charliebk/TaskCodeTwo".to_owned(),
+                    "https://api.github.com".to_owned()
+                )),
+                "{url}"
+            );
+        }
+        let d = detectar("git@gitlab.com:g/sub/p.git", None).ok();
+        assert_eq!(d.map(|d| d.proveedor), Some(Proveedor::GitLab));
+        let d = detectar(
+            "https://git.empresa.com/o/r.git",
+            Some(("github", "https://git.empresa.com/api/v3")),
+        )
+        .ok();
+        assert_eq!(
+            d.map(|d| d.api.texto().to_owned()),
+            Some("https://git.empresa.com/api/v3".to_owned())
+        );
+    }
+
+    #[test]
+    fn falla_cerrado() {
+        for (url, clave) in [
+            ("https://git.empresa.com/o/r.git", None),
+            (
+                "https://github.com/o/r",
+                Some(("gitlab", "https://gitlab.com/api/v4")),
+            ),
+            (
+                "https://git.empresa.com/o/r",
+                Some(("bitbucket", "https://x")),
+            ),
+            ("https://git.empresa.com/o/r", Some(("github", "http://x"))),
+            ("https://github.com/o/r/s", None),
+            ("https://github.com/o", None),
+            ("file:///c/repo", None),
+            ("https://github.com/o/..", None),
+        ] {
+            let r = detectar(url, clave);
+            assert!(
+                r.as_ref()
+                    .is_err_and(|i| i.resultado == Resultado::Rechazada),
+                "{url}: {r:?}"
+            );
+        }
+    }
+}
diff --git a/crates/tc2/src/plataforma/errores.rs b/crates/tc2/src/plataforma/errores.rs
new file mode 100644
index 0000000..d4af11f
--- /dev/null
+++ b/crates/tc2/src/plataforma/errores.rs
@@ -0,0 +1,183 @@
+//! De HTTP a la linea de contrato, con el arreglo para la persona (F2-6:
+//! «errores dirigidos a la persona»). Es el unico sitio que conoce los
+//! codigos HTTP; el adaptador de GitLab (F2-9) lo reutiliza.
+//!
+//! El diagnostico dice que hacer. El mensaje de la API es dato no confiable
+//! (F1-8, Z2): se recorta y se quitan los caracteres de control antes de
+//! citarlo. Nunca lleva el token ni cabeceras.
+
+use super::credenciales::Rol;
+use super::http::{FalloHttp, Respuesta};
+use crate::contrato::{Informe, Resultado};
+
+/// Aviso de GitHub cuando el plan no da rulesets ni proteccion de rama.
+pub const AVISO_PLAN: &str = "Upgrade to GitHub Pro";
+
+/// Mensaje de la API (`message`), saneado: sin controles y como mucho 200
+/// caracteres.
+pub fn mensaje_api(r: &Respuesta) -> String {
+    let m = serde_json::from_str::<serde_json::Value>(&r.cuerpo)
+        .ok()
+        .and_then(|v| v.get("message").and_then(|m| m.as_str()).map(str::to_owned))
+        .unwrap_or_default();
+    sanear(&m)
+}
+
+/// Texto externo listo para un diagnostico.
+pub fn sanear(t: &str) -> String {
+    t.chars()
+        .map(|c| if c.is_control() { ' ' } else { c })
+        .take(200)
+        .collect::<String>()
+        .trim()
+        .to_owned()
+}
+
+/// Si la respuesta es el 403 de plan de GitHub.
+pub fn es_aviso_de_plan(r: &Respuesta) -> bool {
+    r.estado == 403 && mensaje_api(r).contains(AVISO_PLAN)
+}
+
+/// Si la respuesta es un limite de peticiones agotado.
+fn es_limite(r: &Respuesta) -> bool {
+    (r.estado == 403 || r.estado == 429)
+        && (r.restantes.as_deref() == Some("0") || r.reintentar.is_some())
+}
+
+/// Segundos de espera que da la API, si los da.
+fn espera(r: &Respuesta) -> String {
+    if let Some(s) = r.reintentar.as_deref().filter(|s| es_numero(s)) {
+        return s.to_owned();
+    }
+    let ahora = std::time::SystemTime::now()
+        .duration_since(std::time::UNIX_EPOCH)
+        .map(|d| d.as_secs())
+        .unwrap_or(0);
+    r.reinicio
+        .as_deref()
+        .and_then(|s| s.parse::<u64>().ok())
+        .map_or_else(|| "60".to_owned(), |t| t.saturating_sub(ahora).to_string())
+}
+
+fn es_numero(s: &str) -> bool {
+    !s.is_empty() && s.len() <= 10 && s.bytes().all(|b| b.is_ascii_digit())
+}
+
+/// Traduce una respuesta que no es 2xx. `op` es la operacion, `rol` con que
+/// identidad corria y `permiso` el que necesita (para el 403 sin plan).
+pub fn traducir(op: &str, rol: Rol, permiso: &str, r: &Respuesta) -> Informe {
+    let api = mensaje_api(r);
+    let cita = if api.is_empty() {
+        String::new()
+    } else {
+        format!(" (GitHub: «{api}»)")
+    };
+    match r.estado {
+        401 => Informe::nuevo(op, Resultado::Denegada)
+            .con("motivo", "credenciales")
+            .con("rol", rol.nombre())
+            .diagnostico(format!(
+                "el token de {} no vale o ha caducado: define o renueva {}{cita}",
+                rol.nombre(),
+                rol.variable()
+            )),
+        _ if es_limite(r) => {
+            let s = espera(r);
+            Informe::nuevo(op, Resultado::Agotada)
+                .con("motivo", "limite-peticiones")
+                .con("reintenta-en", s.clone())
+                .diagnostico(format!(
+                    "limite de peticiones de la API agotado: espera {s} s y relanza{cita}"
+                ))
+        }
+        403 if api.contains(AVISO_PLAN) => Informe::nuevo(op, Resultado::Rechazada)
+            .con("motivo", "plan")
+            .diagnostico(format!(
+                "el plan de GitHub no da esta funcion en un repo privado: haz el repo publico o usa GitHub Pro, Team o Enterprise{cita}"
+            )),
+        403 => Informe::nuevo(op, Resultado::Denegada)
+            .con("motivo", "permisos")
+            .con("rol", rol.nombre())
+            .diagnostico(format!(
+                "el token de {} no tiene permiso: necesita {permiso}{cita}",
+                rol.nombre()
+            )),
+        404 => Informe::nuevo(op, Resultado::Denegada)
+            .con("motivo", "no-existe-o-sin-acceso")
+            .con("rol", rol.nombre())
+            .diagnostico(format!(
+                "no existe o el token de {} no lo ve: revisa el repo, el PR o el commit y el alcance del token (GitHub da 404 en un repo privado sin acceso){cita}",
+                rol.nombre()
+            )),
+        405 => Informe::nuevo(op, Resultado::Rechazada)
+            .con("motivo", "no-mergeable")
+            .diagnostico(format!(
+                "GitHub no deja mergear: faltan puertas o hay conflictos; lanza estado-puertas{cita}"
+            )),
+        409 => Informe::nuevo(op, Resultado::Rechazada)
+            .con("motivo", "head-cambiado")
+            .diagnostico(format!(
+                "la rama ha cambiado desde que se evaluaron las puertas: vuelve a lanzar estado-puertas{cita}"
+            )),
+        // El detalle del 422 viene en `errors[].message`, no en `message`.
+        422 if r.cuerpo.contains("No commits between") => Informe::nuevo(op, Resultado::Rechazada)
+            .con("motivo", "sin-commits")
+            .diagnostico(format!(
+                "la rama no tiene commits nuevos respecto a la base: no hay nada que revisar{cita}"
+            )),
+        422 => Informe::nuevo(op, Resultado::Error)
+            .con("motivo", "validacion")
+            .diagnostico(format!("GitHub rechazo los datos de la peticion{cita}")),
+        e => Informe::nuevo(op, Resultado::Error)
+            .con("motivo", "http")
+            .con("estado", e.to_string())
+            .diagnostico(format!(
+                "respuesta inesperada de GitHub ({e}): reintenta mas tarde{cita}"
+            )),
+    }
+}
+
+/// Traduce un fallo sin respuesta.
+pub fn fallo(op: &str, f: &FalloHttp) -> Informe {
+    match f {
+        FalloHttp::Rechazada(m) => Informe::nuevo(op, Resultado::Rechazada)
+            .con("motivo", "destino")
+            .diagnostico(format!(
+                "llamada rechazada antes de salir a la red: {}",
+                sanear(m)
+            )),
+        FalloHttp::Sinred(m) => Informe::nuevo(op, Resultado::Sinred).diagnostico(format!(
+            "no se pudo hablar con la API: comprueba la red y reintenta ({})",
+            sanear(m)
+        )),
+        FalloHttp::Tls(m) => Informe::nuevo(op, Resultado::Error)
+            .con("motivo", "tls")
+            .diagnostico(format!(
+                "fallo de TLS: si la red usa un proxy con su propia CA, instalala en el almacen de certificados del sistema ({})",
+                sanear(m)
+            )),
+        FalloHttp::RespuestaInvalida(m) => Informe::nuevo(op, Resultado::Error)
+            .con("motivo", "respuesta-invalida")
+            .diagnostico(format!(
+                "respuesta de la API ilegible o demasiado grande ({})",
+                sanear(m)
+            )),
+        FalloHttp::Redireccion(e) => Informe::nuevo(op, Resultado::Error)
+            .con("motivo", "redireccion")
+            .diagnostico(format!(
+                "la API respondio con una redireccion ({e}) y no se siguen: revisa la URL de la API o el nombre del repo (renombrado o movido)"
+            )),
+        FalloHttp::Otro(m) => Informe::nuevo(op, Resultado::Error)
+            .con("motivo", "cliente")
+            .diagnostico(format!("fallo del cliente HTTP ({})", sanear(m))),
+    }
+}
+
+/// Respuesta 2xx con un cuerpo que no tiene la forma esperada.
+pub fn respuesta_invalida(op: &str, que: &str) -> Informe {
+    Informe::nuevo(op, Resultado::Error)
+        .con("motivo", "respuesta-invalida")
+        .diagnostico(format!(
+            "la respuesta de GitHub no tiene la forma esperada ({que})"
+        ))
+}
diff --git a/crates/tc2/src/plataforma/github.rs b/crates/tc2/src/plataforma/github.rs
new file mode 100644
index 0000000..b74f845
--- /dev/null
+++ b/crates/tc2/src/plataforma/github.rs
@@ -0,0 +1,735 @@
+//! Adaptador de GitHub del puerto `Plataforma` (F2-6, ADR F1-13 decisiones
+//! 1 y 4; F1-6 decision 1 para la identidad de cada operacion).
+//!
+//! Cada endpoint es una variante de [`Endpoint`], que construye el metodo y
+//! la ruta: no hay llamadas con una URL libre. Las respuestas son dato no
+//! confiable (F1-8, Z2): se leen con structs que solo tienen los campos que
+//! se usan y un campo ausente da `error motivo=respuesta-invalida`.
+
+use std::cell::RefCell;
+use std::collections::BTreeMap;
+
+use serde::Deserialize;
+use serde::de::DeserializeOwned;
+
+use super::credenciales::{Credenciales, Rol, Secreto};
+use super::errores::{self, es_aviso_de_plan};
+use super::http::{Base, Cliente, Metodo, Respuesta, codificar};
+use super::puertas::{self, EstadoCheck, Lecturas};
+use super::{Firma, ModeloPuertas, Paso, Personas, Plataforma, ReglasLeidas, Usuario};
+use crate::contrato::{Informe, Resultado, Valor, corto};
+
+const ABRIR_PR: &str = "abrir-pr";
+const MERGE_PR: &str = "merge-pr";
+const LEER_PUERTAS: &str = "leer-puertas";
+const LEER_PERMISOS_CI: &str = "leer-permisos-ci";
+const FIRMA_COMMIT: &str = "firma-commit";
+const IDENTIDAD: &str = "identidad";
+
+/// Paginas como mucho al seguir `Link rel=next` (100 elementos cada una).
+const MAX_PAGINAS: usize = 10;
+
+/// Garantias que en GitHub dan los rulesets (tier «Free publico;
+/// Pro/Team/Enterprise» de la tabla de F1-13): sin ellos, el 403 de plan.
+const GARANTIAS_DE_RULESETS: [&str; 10] = [
+    "G01", "G02", "G03", "G04", "G05", "G06", "G07", "G08", "G09", "G11",
+];
+
+/// Endpoints de la API que usa el adaptador (lista cerrada).
+enum Endpoint<'a> {
+    Usuario,
+    Pr(u64),
+    PrsAbiertos { cabeza: &'a str, base: &'a str },
+    CrearPr,
+    Merge(u64),
+    ReglasRama(&'a str),
+    Rulesets,
+    CheckRuns(&'a str),
+    Reviews(u64),
+    PermisosWorkflow,
+    Commit(&'a str),
+}
+
+impl Endpoint<'_> {
+    fn metodo_y_ruta(&self, repo: &str) -> (Metodo, String) {
+        let r = format!("/repos/{repo}");
+        match self {
+            Self::Usuario => (Metodo::Get, "/user".into()),
+            Self::Pr(n) => (Metodo::Get, format!("{r}/pulls/{n}")),
+            Self::PrsAbiertos { cabeza, base } => (
+                Metodo::Get,
+                format!(
+                    "{r}/pulls?head={}&base={}&state=open&per_page=100",
+                    codificar(cabeza),
+                    codificar(base)
+                ),
+            ),
+            Self::CrearPr => (Metodo::Post, format!("{r}/pulls")),
+            Self::Merge(n) => (Metodo::Put, format!("{r}/pulls/{n}/merge")),
+            Self::ReglasRama(rama) => (Metodo::Get, format!("{r}/rules/branches/{rama}")),
+            Self::Rulesets => (Metodo::Get, format!("{r}/rulesets?per_page=100")),
+            Self::CheckRuns(sha) => (
+                Metodo::Get,
+                format!("{r}/commits/{sha}/check-runs?per_page=100"),
+            ),
+            Self::Reviews(n) => (Metodo::Get, format!("{r}/pulls/{n}/reviews?per_page=100")),
+            Self::PermisosWorkflow => (Metodo::Get, format!("{r}/actions/permissions/workflow")),
+            Self::Commit(sha) => (Metodo::Get, format!("{r}/commits/{sha}")),
+        }
+    }
+}
+
+#[derive(Deserialize)]
+struct UsuarioApi {
+    #[serde(default)]
+    login: String,
+    id: u64,
+    #[serde(rename = "type")]
+    tipo: String,
+}
+
+impl From<UsuarioApi> for Usuario {
+    fn from(u: UsuarioApi) -> Self {
+        Self {
+            login: u.login,
+            id: u.id,
+            tipo: u.tipo,
+        }
+    }
+}
+
+#[derive(Deserialize)]
+struct RefApi {
+    #[serde(rename = "ref")]
+    nombre: String,
+    #[serde(default)]
+    sha: String,
+}
+
+#[derive(Deserialize)]
+struct PrApi {
+    number: u64,
+    state: String,
+    #[serde(default)]
+    merged: bool,
+    #[serde(default)]
+    mergeable_state: Option<String>,
+    head: RefApi,
+    base: RefApi,
+}
+
+#[derive(Deserialize)]
+struct CheckRunsApi {
+    check_runs: Vec<CheckRunApi>,
+}
+
+#[derive(Deserialize)]
+struct CheckRunApi {
+    name: String,
+    status: String,
+    #[serde(default)]
+    conclusion: Option<String>,
+    #[serde(default)]
+    app: Option<AppApi>,
+}
+
+#[derive(Deserialize)]
+struct AppApi {
+    id: u64,
+}
+
+#[derive(Deserialize)]
+struct ReviewApi {
+    state: String,
+    #[serde(default)]
+    user: Option<UsuarioApi>,
+}
+
+#[derive(Deserialize)]
+struct MergeApi {
+    merged: bool,
+    #[serde(default)]
+    sha: String,
+}
+
+#[derive(Deserialize)]
+struct PermisosApi {
+    can_approve_pull_request_reviews: bool,
+    default_workflow_permissions: String,
+}
+
+#[derive(Deserialize)]
+struct CommitApi {
+    commit: CommitInternoApi,
+    #[serde(default)]
+    committer: Option<UsuarioApi>,
+}
+
+#[derive(Deserialize)]
+struct CommitInternoApi {
+    verification: VerificacionApi,
+}
+
+#[derive(Deserialize)]
+struct VerificacionApi {
+    verified: bool,
+    #[serde(default)]
+    reason: String,
+}
+
+#[derive(Deserialize)]
+struct RulesetApi {
+    name: String,
+}
+
+#[derive(Deserialize)]
+struct ReglaApi {
+    #[serde(rename = "type")]
+    tipo: String,
+}
+
+/// Adaptador de GitHub para un repo.
+pub struct GitHub {
+    cliente: Cliente,
+    repo: String,
+    credenciales: Box<dyn Credenciales>,
+    personas: Personas,
+    identidades: RefCell<BTreeMap<Rol, Usuario>>,
+}
+
+impl std::fmt::Debug for GitHub {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        f.debug_struct("GitHub")
+            .field("cliente", &self.cliente)
+            .field("repo", &self.repo)
+            .finish_non_exhaustive()
+    }
+}
+
+impl GitHub {
+    /// Adaptador para `dueno/repo` en la API `base`.
+    pub fn nuevo(
+        base: Base,
+        repo: &str,
+        credenciales: Box<dyn Credenciales>,
+        personas: Personas,
+    ) -> Result<Self, Informe> {
+        let partes: Vec<&str> = repo.split('/').collect();
+        let valido = partes.len() == 2
+            && partes.iter().all(|p| {
+                !p.is_empty()
+                    && *p != "."
+                    && *p != ".."
+                    && p.bytes()
+                        .all(|b| b.is_ascii_alphanumeric() || b"-._".contains(&b))
+            });
+        if !valido {
+            return Err(Informe::nuevo("plataforma", Resultado::Uso)
+                .con("motivo", "repo-invalido")
+                .diagnostico("el repo de GitHub tiene que ser dueno/repo"));
+        }
+        Ok(Self {
+            cliente: Cliente::nuevo(base),
+            repo: repo.to_owned(),
+            credenciales,
+            personas,
+            identidades: RefCell::new(BTreeMap::new()),
+        })
+    }
+
+    /// Llamadas hechas, `METODO ruta` (para medir el coste por operacion).
+    pub fn registro(&self) -> Vec<String> {
+        self.cliente.registro()
+    }
+
+    fn token(&self, op: &str, rol: Rol) -> Paso<Secreto> {
+        self.credenciales.token(rol).ok_or_else(|| {
+            let extra = if rol == Rol::Persona {
+                " (el de persona solo se usa desde un terminal interactivo)"
+            } else {
+                ""
+            };
+            Informe::nuevo(op, Resultado::Rechazada)
+                .con("motivo", "identidad")
+                .con("falta", rol.nombre())
+                .diagnostico(format!(
+                    "esta operacion corre con la identidad de {}: define {}{extra}",
+                    rol.nombre(),
+                    rol.variable()
+                ))
+        })
+    }
+
+    /// Llamada cruda: cualquier estado HTTP llega como respuesta.
+    fn pedir(
+        &self,
+        op: &str,
+        rol: Rol,
+        ep: &Endpoint<'_>,
+        cuerpo: Option<&str>,
+    ) -> Paso<Respuesta> {
+        let token = self.token(op, rol)?;
+        let (metodo, ruta) = ep.metodo_y_ruta(&self.repo);
+        self.cliente
+            .llamar(metodo, &ruta, Some(&token), cuerpo)
+            .map_err(|f| errores::fallo(op, &f))
+    }
+
+    /// Lectura que exige 2xx y la forma `T`.
+    fn leer<T: DeserializeOwned>(
+        &self,
+        op: &str,
+        rol: Rol,
+        permiso: &str,
+        ep: &Endpoint<'_>,
+    ) -> Paso<T> {
+        let r = self.pedir(op, rol, ep, None)?;
+        if !r.exito() {
+            return Err(errores::traducir(op, rol, permiso, &r));
+        }
+        parsear(op, &r.cuerpo)
+    }
+
+    /// Lista paginada: sigue `Link rel=next` en la misma base, hasta
+    /// [`MAX_PAGINAS`].
+    fn leer_paginas<T, F>(
+        &self,
+        op: &str,
+        rol: Rol,
+        permiso: &str,
+        ep: &Endpoint<'_>,
+        mut extraer: F,
+    ) -> Paso<Vec<T>>
+    where
+        F: FnMut(&str) -> Paso<Vec<T>>,
+    {
+        let token = self.token(op, rol)?;
+        let (_, mut ruta) = ep.metodo_y_ruta(&self.repo);
+        let mut todo = Vec::new();
+        for _ in 0..MAX_PAGINAS {
+            let r = self
+                .cliente
+                .llamar(Metodo::Get, &ruta, Some(&token), None)
+                .map_err(|f| errores::fallo(op, &f))?;
+            if !r.exito() {
+                return Err(errores::traducir(op, rol, permiso, &r));
+            }
+            todo.extend(extraer(&r.cuerpo)?);
+            match r.siguiente() {
+                Some(url) => {
+                    ruta = self
+                        .cliente
+                        .ruta_de(url)
+                        .map_err(|f| errores::fallo(op, &f))?;
+                }
+                None => return Ok(todo),
+            }
+        }
+        Err(Informe::nuevo(op, Resultado::Error)
+            .con("motivo", "demasiadas-paginas")
+            .diagnostico(format!(
+                "la lista tiene mas de {} elementos: no se evalua a medias",
+                MAX_PAGINAS * 100
+            )))
+    }
+
+    /// La identidad de persona tiene que estar en la lista de personas.
+    fn exigir_persona(&self, op: &str) -> Paso<()> {
+        let u = self.identidad(Rol::Persona).map_err(|mut i| {
+            op.clone_into(&mut i.comando);
+            i
+        })?;
+        if u.es_persona(&self.personas) {
+            Ok(())
+        } else {
+            Err(Informe::nuevo(op, Resultado::Rechazada)
+                .con("motivo", "identidad")
+                .con("rol", "persona")
+                .diagnostico(format!(
+                    "el token de persona es de «{}», que no esta en la lista de personas del proyecto",
+                    errores::sanear(&u.login)
+                )))
+        }
+    }
+
+    fn pr_abierto(&self, rama: &str, base: &str) -> Paso<Option<u64>> {
+        let cabeza = format!("{}:{rama}", self.dueno());
+        let prs: Vec<PrApi> = self.leer_paginas(
+            ABRIR_PR,
+            Rol::Agente,
+            "pull_requests:read",
+            &Endpoint::PrsAbiertos {
+                cabeza: &cabeza,
+                base,
+            },
+            |c| parsear(ABRIR_PR, c),
+        )?;
+        Ok(prs
+            .into_iter()
+            .find(|p| p.head.nombre == rama && p.base.nombre == base)
+            .map(|p| p.number))
+    }
+
+    fn dueno(&self) -> &str {
+        self.repo.split('/').next().unwrap_or_default()
+    }
+}
+
+fn parsear<T: DeserializeOwned>(op: &str, cuerpo: &str) -> Paso<T> {
+    serde_json::from_str(cuerpo)
+        .map_err(|e| errores::respuesta_invalida(op, &errores::sanear(&e.to_string())))
+}
+
+/// Rama aceptable en una ruta de la API (la misma forma que exige Git, mas
+/// estricta).
+fn rama_valida(r: &str) -> bool {
+    !r.is_empty()
+        && r.len() <= 200
+        && !r.starts_with(['-', '/', '.'])
+        && !r.ends_with(['/', '.'])
+        && !r.contains("..")
+        && !r.contains("//")
+        && !r.ends_with(".lock")
+        && r.bytes()
+            .all(|b| b.is_ascii_alphanumeric() || b"-._/".contains(&b))
+}
+
+fn sha_valido(s: &str) -> bool {
+    s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit())
+}
+
+fn uso(op: &str, motivo: &str, diag: &str) -> Informe {
+    Informe::nuevo(op, Resultado::Uso)
+        .con("motivo", motivo)
+        .diagnostico(diag.to_owned())
+}
+
+impl Plataforma for GitHub {
+    fn abrir_pr(&self, rama: &str, base: &str, titulo: &str, cuerpo: &str) -> Informe {
+        if !rama_valida(rama) || !rama_valida(base) {
+            return uso(ABRIR_PR, "rama-invalida", "nombre de rama no valido");
+        }
+        if titulo.trim().is_empty() || titulo.chars().any(char::is_control) {
+            return uso(
+                ABRIR_PR,
+                "titulo-invalido",
+                "el titulo no puede estar vacio ni llevar controles",
+            );
+        }
+        match self.pr_abierto(rama, base) {
+            Ok(Some(n)) => {
+                return Informe::nuevo(ABRIR_PR, Resultado::SinCambios).con("pr", n.to_string());
+            }
+            Ok(None) => {}
+            Err(i) => return i,
+        }
+        let datos = serde_json::json!({
+            "title": titulo, "head": rama, "base": base, "body": cuerpo,
+        })
+        .to_string();
+        let r = match self.pedir(ABRIR_PR, Rol::Agente, &Endpoint::CrearPr, Some(&datos)) {
+            Ok(r) => r,
+            Err(i) => return i,
+        };
+        if r.estado == 422 && r.cuerpo.contains("already exists") {
+            // Otro proceso lo abrio entre la lectura y la escritura.
+            return match self.pr_abierto(rama, base) {
+                Ok(Some(n)) => {
+                    Informe::nuevo(ABRIR_PR, Resultado::SinCambios).con("pr", n.to_string())
+                }
+                Ok(None) => errores::traducir(ABRIR_PR, Rol::Agente, "pull_requests:write", &r),
+                Err(i) => i,
+            };
+        }
+        if !r.exito() {
+            return errores::traducir(ABRIR_PR, Rol::Agente, "pull_requests:write", &r);
+        }
+        match parsear::<PrApi>(ABRIR_PR, &r.cuerpo) {
+            Ok(p) => Informe::nuevo(ABRIR_PR, Resultado::Ok).con("pr", p.number.to_string()),
+            Err(i) => i,
+        }
+    }
+
+    fn estado_puertas(&self, rol: Rol, pr: u64, modelo: &ModeloPuertas) -> Informe {
+        let op = puertas::ESTADO_PUERTAS;
+        if rol == Rol::Revisor {
+            return Informe::nuevo(op, Resultado::Rechazada)
+                .con("motivo", "identidad")
+                .con("rol", rol.nombre())
+                .diagnostico("estado-puertas corre con la identidad de agente o de persona");
+        }
+        if rol == Rol::Persona
+            && let Err(i) = self.exigir_persona(op)
+        {
+            return i;
+        }
+        let permiso = "pull_requests:read y checks:read";
+        let p: PrApi = match self.leer(op, rol, permiso, &Endpoint::Pr(pr)) {
+            Ok(p) => p,
+            Err(i) => return i,
+        };
+        if p.merged {
+            return puertas::mergeado(pr);
+        }
+        if p.state != "open" {
+            return Informe::nuevo(op, Resultado::Rechazada)
+                .con("pr", pr.to_string())
+                .con("motivo", "pr-cerrado")
+                .diagnostico("el PR esta cerrado sin mergear: reabrelo o abre otro");
+        }
+        if !rama_valida(&p.base.nombre) || !sha_valido(&p.head.sha) {
+            return errores::respuesta_invalida(op, "rama base o sha del head");
+        }
+        let r = match self.pedir(op, rol, &Endpoint::ReglasRama(&p.base.nombre), None) {
+            Ok(r) => r,
+            Err(i) => return i,
+        };
+        let plan_da_reglas = !es_aviso_de_plan(&r);
+        if plan_da_reglas && !r.exito() {
+            return errores::traducir(op, rol, permiso, &r);
+        }
+        if !plan_da_reglas
+            && let Some(g) = modelo
+                .exigir_impuestas
+                .iter()
+                .find(|g| GARANTIAS_DE_RULESETS.contains(&g.as_str()))
+        {
+            return puertas::rechazada_por_plan(pr, g);
+        }
+        let runs: Vec<CheckRunApi> =
+            match self.leer_paginas(op, rol, permiso, &Endpoint::CheckRuns(&p.head.sha), |c| {
+                parsear::<CheckRunsApi>(op, c).map(|x| x.check_runs)
+            }) {
+                Ok(v) => v,
+                Err(i) => return i,
+            };
+        // La API da los check-runs del mas nuevo al mas viejo: cuenta el
+        // primero de cada nombre que publico su origen (integration_id).
+        let checks = modelo
+            .checks
+            .iter()
+            .map(|(nombre, origen)| {
+                let estado = runs
+                    .iter()
+                    .find(|r| &r.name == nombre && r.app.as_ref().is_some_and(|a| a.id == *origen))
+                    .map_or(EstadoCheck::Pendiente, |r| {
+                        if r.status != "completed" {
+                            EstadoCheck::Pendiente
+                        } else if r.conclusion.as_deref() == Some("success") {
+                            EstadoCheck::Ok
+                        } else {
+                            EstadoCheck::Fallo
+                        }
+                    });
+                (nombre.clone(), estado)
+            })
+            .collect();
+        let reviews: Vec<ReviewApi> =
+            match self.leer_paginas(op, rol, permiso, &Endpoint::Reviews(pr), |c| parsear(op, c)) {
+                Ok(v) => v,
+                Err(i) => return i,
+            };
+        // Las reviews vienen de la mas vieja a la mas nueva: cuenta la ultima
+        // de cada cuenta, por id (F1-6, regla 6).
+        let mut ultima: BTreeMap<u64, (String, Usuario)> = BTreeMap::new();
+        for rv in reviews {
+            if let Some(u) = rv.user {
+                ultima.insert(u.id, (rv.state, u.into()));
+            }
+        }
+        let aprobadas: Vec<&Usuario> = ultima
+            .values()
+            .filter(|(s, _)| s == "APPROVED")
+            .map(|(_, u)| u)
+            .collect();
+        let aprobaciones = aprobadas
+            .iter()
+            .filter(|u| u.es_persona(&self.personas))
+            .count();
+        puertas::evaluar(&Lecturas {
+            pr,
+            checks,
+            aprobaciones,
+            ignoradas: aprobadas.len() - aprobaciones,
+            al_dia: p.mergeable_state.as_deref() != Some("behind"),
+        })
+    }
+
+    fn merge_pr(&self, pr: u64, sha: &str) -> Informe {
+        if !sha_valido(sha) {
+            return uso(
+                MERGE_PR,
+                "sha-invalido",
+                "el sha esperado tiene que ser completo (40 hex)",
+            );
+        }
+        if let Err(i) = self.exigir_persona(MERGE_PR) {
+            return i;
+        }
+        let permiso = "contents:write";
+        let p: PrApi = match self.leer(MERGE_PR, Rol::Persona, permiso, &Endpoint::Pr(pr)) {
+            Ok(p) => p,
+            Err(i) => return i,
+        };
+        if p.merged {
+            return Informe::nuevo(MERGE_PR, Resultado::SinCambios)
+                .con("pr", pr.to_string())
+                .con("estado", "mergeado");
+        }
+        if p.state != "open" {
+            return Informe::nuevo(MERGE_PR, Resultado::Rechazada)
+                .con("pr", pr.to_string())
+                .con("motivo", "pr-cerrado")
+                .diagnostico("el PR esta cerrado sin mergear");
+        }
+        if !p.head.sha.eq_ignore_ascii_case(sha) {
+            return Informe::nuevo(MERGE_PR, Resultado::Rechazada)
+                .con("pr", pr.to_string())
+                .con("motivo", "head-cambiado")
+                .diagnostico("la rama ha cambiado desde que se evaluaron las puertas: vuelve a lanzar estado-puertas");
+        }
+        let datos = serde_json::json!({ "merge_method": "merge", "sha": sha }).to_string();
+        let r = match self.pedir(MERGE_PR, Rol::Persona, &Endpoint::Merge(pr), Some(&datos)) {
+            Ok(r) => r,
+            Err(i) => return i,
+        };
+        if !r.exito() {
+            let mut i = errores::traducir(MERGE_PR, Rol::Persona, permiso, &r);
+            i.claves
+                .insert(0, ("pr".to_owned(), Valor::Texto(pr.to_string())));
+            return i;
+        }
+        match parsear::<MergeApi>(MERGE_PR, &r.cuerpo) {
+            Ok(m) if m.merged && sha_valido(&m.sha) => Informe::nuevo(MERGE_PR, Resultado::Ok)
+                .con("pr", pr.to_string())
+                .con("commit", corto(&m.sha)),
+            Ok(_) => errores::respuesta_invalida(MERGE_PR, "merged o sha"),
+            Err(i) => i,
+        }
+    }
+
+    fn leer_puertas(&self, rol: Rol, rama: &str) -> Paso<ReglasLeidas> {
+        if !rama_valida(rama) {
+            return Err(uso(
+                LEER_PUERTAS,
+                "rama-invalida",
+                "nombre de rama no valido",
+            ));
+        }
+        if rol == Rol::Revisor {
+            return Err(Informe::nuevo(LEER_PUERTAS, Resultado::Rechazada)
+                .con("motivo", "identidad")
+                .con("rol", rol.nombre())
+                .diagnostico("leer-puertas corre con la identidad de agente o de persona"));
+        }
+        let permiso = "metadata:read";
+        let r = self.pedir(LEER_PUERTAS, rol, &Endpoint::Rulesets, None)?;
+        if es_aviso_de_plan(&r) {
+            return Ok(ReglasLeidas {
+                plan_da_reglas: false,
+                ..ReglasLeidas::default()
+            });
+        }
+        if !r.exito() {
+            return Err(errores::traducir(LEER_PUERTAS, rol, permiso, &r));
+        }
+        let rulesets: Vec<RulesetApi> = parsear(LEER_PUERTAS, &r.cuerpo)?;
+        let reglas: Vec<ReglaApi> =
+            self.leer(LEER_PUERTAS, rol, permiso, &Endpoint::ReglasRama(rama))?;
+        Ok(ReglasLeidas {
+            plan_da_reglas: true,
+            rulesets: rulesets.into_iter().map(|r| r.name).collect(),
+            reglas_rama: reglas.into_iter().map(|r| r.tipo).collect(),
+        })
+    }
+
+    fn leer_permisos_ci(&self) -> Informe {
+        if let Err(i) = self.exigir_persona(LEER_PERMISOS_CI) {
+            return i;
+        }
+        let p: PermisosApi = match self.leer(
+            LEER_PERMISOS_CI,
+            Rol::Persona,
+            "administration:read",
+            &Endpoint::PermisosWorkflow,
+        ) {
+            Ok(p) => p,
+            Err(i) => return i,
+        };
+        let permisos = errores::sanear(&p.default_workflow_permissions);
+        if p.can_approve_pull_request_reviews {
+            Informe::nuevo(LEER_PERMISOS_CI, Resultado::Rechazada)
+                .con("motivo", "ci-puede-aprobar")
+                .con("permisos", permisos)
+                .diagnostico("el token de Actions puede aprobar PR: desactiva «Allow GitHub Actions to create and approve pull requests» en Settings > Actions > General (G16)")
+        } else {
+            Informe::nuevo(LEER_PERMISOS_CI, Resultado::Ok)
+                .con("ci-aprueba", "false")
+                .con("permisos", permisos)
+        }
+    }
+
+    fn firma_commit(&self, sha: &str) -> Paso<Firma> {
+        if !sha_valido(sha) {
+            return Err(uso(
+                FIRMA_COMMIT,
+                "sha-invalido",
+                "el sha tiene que ser completo (40 hex)",
+            ));
+        }
+        let c: CommitApi = self.leer(
+            FIRMA_COMMIT,
+            Rol::Agente,
+            "contents:read",
+            &Endpoint::Commit(sha),
+        )?;
+        Ok(Firma {
+            verificada: c.commit.verification.verified,
+            motivo: errores::sanear(&c.commit.verification.reason),
+            committer: c.committer.map(Usuario::from),
+        })
+    }
+
+    fn identidad(&self, rol: Rol) -> Paso<Usuario> {
+        if let Some(u) = self.identidades.borrow().get(&rol) {
+            return Ok(u.clone());
+        }
+        let u: UsuarioApi = self.leer(IDENTIDAD, rol, "(ninguno)", &Endpoint::Usuario)?;
+        let u = Usuario::from(u);
+        self.identidades.borrow_mut().insert(rol, u.clone());
+        Ok(u)
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn ramas_y_shas() {
+        for r in ["develop", "feature/task-021-f2-6-adaptador-github", "a.b_c"] {
+            assert!(rama_valida(r), "{r}");
+        }
+        for r in [
+            "", "-x", "/x", "x/", "a..b", "a//b", "a b", "a%2e", "x.lock", ".x",
+        ] {
+            assert!(!rama_valida(r), "{r}");
+        }
+        assert!(sha_valido(&"a".repeat(40)));
+        assert!(!sha_valido(&"g".repeat(40)));
+        assert!(!sha_valido("abc"));
+    }
+
+    #[test]
+    fn repo_invalido_es_uso() {
+        for repo in ["o", "o/r/s", "o/..", "o/r?x", "/r"] {
+            let r = GitHub::nuevo(
+                Base::github(),
+                repo,
+                Box::new(super::super::CredencialesFijas::nuevas()),
+                Personas::default(),
+            );
+            assert!(r.is_err_and(|i| i.resultado == Resultado::Uso), "{repo}");
+        }
+    }
+}
diff --git a/crates/tc2/src/plataforma/http.rs b/crates/tc2/src/plataforma/http.rs
new file mode 100644
index 0000000..510ee31
--- /dev/null
+++ b/crates/tc2/src/plataforma/http.rs
@@ -0,0 +1,424 @@
+//! Envoltorio unico de llamadas HTTP del puerto Plataforma (F1-13, D6a y
+//! decision 1; F1-14 punto 2).
+//!
+//! Toda llamada a la API sale por [`Cliente::llamar`], que comprueba antes de
+//! abrir la red:
+//! - el metodo: solo los de [`Metodo`] (lista cerrada por tipo);
+//! - el destino: la ruta es relativa a una [`Base`] fija (la API de GitHub,
+//!   la que diga la configuracion o, solo para tests, `http://127.0.0.1`);
+//! - la ruta: empieza por `/`, sin segmentos `.` ni `..`, sin `\`, sin
+//!   `%2e` ni `%25` (tampoco codificados dos veces) y sin caracteres fuera
+//!   de los no reservados.
+//!
+//! No sigue redirecciones, limita el tamano de la respuesta y los tiempos de
+//! espera, y lleva un registro `METODO ruta` sin cabeceras. El token nunca
+//! sale en un error ni en el registro.
+
+use std::cell::RefCell;
+use std::time::Duration;
+
+use super::credenciales::Secreto;
+
+/// Tamano maximo del cuerpo de una respuesta.
+pub const LIMITE_CUERPO: u64 = 1024 * 1024;
+
+/// Tiempo maximo de una llamada completa.
+const ESPERA_TOTAL: Duration = Duration::from_secs(30);
+
+/// Tiempo maximo para conectar.
+const ESPERA_CONEXION: Duration = Duration::from_secs(10);
+
+/// La API publica de GitHub.
+pub const API_GITHUB: &str = "https://api.github.com";
+
+/// Metodos permitidos. No hay `DELETE` ni `PATCH`: ninguna operacion de la
+/// lista cerrada los usa.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Metodo {
+    /// Lectura.
+    Get,
+    /// Crear (abrir un PR).
+    Post,
+    /// Mergear un PR.
+    Put,
+}
+
+impl Metodo {
+    /// Nombre HTTP.
+    pub fn nombre(self) -> &'static str {
+        match self {
+            Self::Get => "GET",
+            Self::Post => "POST",
+            Self::Put => "PUT",
+        }
+    }
+}
+
+/// Origen de la API: esquema, host y, si acaso, puerto y prefijo, sin barra
+/// final. Solo `https`, salvo `http://127.0.0.1:<puerto>` para los tests.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Base(String);
+
+impl Base {
+    /// La API publica de GitHub.
+    pub fn github() -> Self {
+        Self(API_GITHUB.to_owned())
+    }
+
+    /// Valida una base. `Err` con el motivo si no vale.
+    pub fn nueva(url: &str) -> Result<Self, String> {
+        let url = url.trim_end_matches('/');
+        let resto = if let Some(r) = url.strip_prefix("https://") {
+            r
+        } else if let Some(r) = url.strip_prefix("http://127.0.0.1:") {
+            if r.is_empty() || !r.bytes().all(|b| b.is_ascii_digit()) {
+                return Err("http solo vale para 127.0.0.1 con un puerto y sin ruta".into());
+            }
+            return Ok(Self(url.to_owned()));
+        } else {
+            return Err("la URL de la API tiene que ser https".into());
+        };
+        let (host, ruta) = resto.split_once('/').unwrap_or((resto, ""));
+        let host_valido = !host.is_empty()
+            && host
+                .bytes()
+                .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-' || b == b':')
+            && !host.starts_with(['.', '-', ':']);
+        if !host_valido {
+            return Err("host de la API no valido (sin usuario ni contrasena)".into());
+        }
+        if !ruta.is_empty() && validar_ruta(&format!("/{ruta}")).is_err() {
+            return Err("prefijo de la API no valido".into());
+        }
+        Ok(Self(url.to_owned()))
+    }
+
+    /// Texto de la base.
+    pub fn texto(&self) -> &str {
+        &self.0
+    }
+}
+
+/// Respuesta ya leida: estado, cuerpo y las cabeceras que se usan.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Respuesta {
+    /// Codigo HTTP.
+    pub estado: u16,
+    /// Cuerpo como texto (hasta [`LIMITE_CUERPO`]).
+    pub cuerpo: String,
+    /// `Link` (paginacion).
+    pub link: Option<String>,
+    /// `x-ratelimit-remaining`.
+    pub restantes: Option<String>,
+    /// `retry-after`, en segundos.
+    pub reintentar: Option<String>,
+    /// `x-ratelimit-reset`, en segundos desde 1970.
+    pub reinicio: Option<String>,
+}
+
+impl Respuesta {
+    /// Si el estado es 2xx.
+    pub fn exito(&self) -> bool {
+        (200..300).contains(&self.estado)
+    }
+
+    /// Siguiente pagina de `Link` (`rel="next"`), si la hay.
+    pub fn siguiente(&self) -> Option<&str> {
+        let link = self.link.as_deref()?;
+        link.split(',').find_map(|parte| {
+            let (url, params) = parte.split_once(';')?;
+            let es_next = params.split(';').any(|p| p.trim() == "rel=\"next\"");
+            let url = url.trim().strip_prefix('<')?.strip_suffix('>')?;
+            es_next.then_some(url)
+        })
+    }
+}
+
+/// Por que no hubo respuesta HTTP.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum FalloHttp {
+    /// El envoltorio la rechazo antes de la red.
+    Rechazada(String),
+    /// No se pudo conectar (DNS, conexion, tiempo agotado).
+    Sinred(String),
+    /// Fallo de TLS (certificado, almacen del sistema).
+    Tls(String),
+    /// Respuesta mayor que [`LIMITE_CUERPO`] o ilegible.
+    RespuestaInvalida(String),
+    /// La API respondio con una redireccion (no se siguen).
+    Redireccion(u16),
+    /// Otro fallo del cliente.
+    Otro(String),
+}
+
+/// Cliente HTTP con su base fija.
+pub struct Cliente {
+    base: Base,
+    agente: ureq::Agent,
+    registro: RefCell<Vec<String>>,
+}
+
+impl std::fmt::Debug for Cliente {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        f.debug_struct("Cliente").field("base", &self.base).finish()
+    }
+}
+
+impl Cliente {
+    /// Cliente para una base ya validada.
+    pub fn nuevo(base: Base) -> Self {
+        let tls = ureq::tls::TlsConfig::builder()
+            .root_certs(ureq::tls::RootCerts::PlatformVerifier)
+            .build();
+        let agente: ureq::Agent = ureq::Agent::config_builder()
+            .tls_config(tls)
+            .http_status_as_error(false)
+            .max_redirects(0)
+            .max_redirects_will_error(false)
+            .save_redirect_history(false)
+            .timeout_global(Some(ESPERA_TOTAL))
+            .timeout_connect(Some(ESPERA_CONEXION))
+            .user_agent(format!("taskcodetwo-tc2/{}", crate::VERSION))
+            .build()
+            .into();
+        Self {
+            base,
+            agente,
+            registro: RefCell::new(Vec::new()),
+        }
+    }
+
+    /// La base del cliente.
+    pub fn base(&self) -> &Base {
+        &self.base
+    }
+
+    /// Llamadas hechas, `METODO ruta`, en orden.
+    pub fn registro(&self) -> Vec<String> {
+        self.registro.borrow().clone()
+    }
+
+    /// Convierte una URL absoluta de paginacion en ruta, solo si es de la
+    /// misma base.
+    pub fn ruta_de(&self, url: &str) -> Result<String, FalloHttp> {
+        url.strip_prefix(self.base.texto())
+            .filter(|r| r.starts_with('/'))
+            .map(str::to_owned)
+            .ok_or_else(|| FalloHttp::Rechazada("paginacion hacia otro destino".into()))
+    }
+
+    /// Hace la llamada. `ruta` incluye la consulta (`?a=b`).
+    pub fn llamar(
+        &self,
+        metodo: Metodo,
+        ruta: &str,
+        token: Option<&Secreto>,
+        cuerpo: Option<&str>,
+    ) -> Result<Respuesta, FalloHttp> {
+        validar_ruta(ruta).map_err(FalloHttp::Rechazada)?;
+        self.registro
+            .borrow_mut()
+            .push(format!("{} {}", metodo.nombre(), ruta));
+        let url = format!("{}{}", self.base.texto(), ruta);
+        let resultado = match metodo {
+            Metodo::Get => self.cabeceras(self.agente.get(&url), token).call(),
+            Metodo::Post => self
+                .cabeceras(self.agente.post(&url), token)
+                .header("Content-Type", "application/json")
+                .send(cuerpo.unwrap_or("{}")),
+            Metodo::Put => self
+                .cabeceras(self.agente.put(&url), token)
+                .header("Content-Type", "application/json")
+                .send(cuerpo.unwrap_or("{}")),
+        };
+        let mut r = resultado.map_err(clasificar)?;
+        let estado = r.status().as_u16();
+        if (300..400).contains(&estado) {
+            return Err(FalloHttp::Redireccion(estado));
+        }
+        let cabecera = |n: &str| {
+            r.headers()
+                .get(n)
+                .and_then(|v| v.to_str().ok())
+                .map(str::to_owned)
+        };
+        let link = cabecera("link");
+        let restantes = cabecera("x-ratelimit-remaining");
+        let reintentar = cabecera("retry-after");
+        let reinicio = cabecera("x-ratelimit-reset");
+        let cuerpo = r
+            .body_mut()
+            .with_config()
+            .limit(LIMITE_CUERPO)
+            .read_to_string()
+            .map_err(clasificar)?;
+        Ok(Respuesta {
+            estado,
+            cuerpo,
+            link,
+            restantes,
+            reintentar,
+            reinicio,
+        })
+    }
+
+    fn cabeceras<B>(
+        &self,
+        p: ureq::RequestBuilder<B>,
+        token: Option<&Secreto>,
+    ) -> ureq::RequestBuilder<B> {
+        let p = p
+            .header("Accept", "application/vnd.github+json")
+            .header("X-GitHub-Api-Version", "2022-11-28");
+        match token {
+            Some(t) => p.header("Authorization", format!("Bearer {}", t.exponer())),
+            None => p,
+        }
+    }
+}
+
+/// Clasifica un error de ureq sin copiar nada que pueda llevar el token
+/// (ureq no pone cabeceras en sus errores; aun asi solo se usa el tipo).
+fn clasificar(e: ureq::Error) -> FalloHttp {
+    match e {
+        ureq::Error::HostNotFound
+        | ureq::Error::ConnectionFailed
+        | ureq::Error::Timeout(_)
+        | ureq::Error::Io(_)
+        | ureq::Error::BodyStalled => FalloHttp::Sinred(tipo_error(&e)),
+        ureq::Error::Tls(_)
+        | ureq::Error::Rustls(_)
+        | ureq::Error::Pem(_)
+        | ureq::Error::TlsRequired => FalloHttp::Tls(tipo_error(&e)),
+        ureq::Error::BodyExceedsLimit(_) | ureq::Error::LargeResponseHeader(..) => {
+            FalloHttp::RespuestaInvalida(tipo_error(&e))
+        }
+        ureq::Error::TooManyRedirects | ureq::Error::RedirectFailed => FalloHttp::Redireccion(0),
+        _ => FalloHttp::Otro(tipo_error(&e)),
+    }
+}
+
+/// Texto corto del error de ureq (su `Display` no lleva cabeceras).
+fn tipo_error(e: &ureq::Error) -> String {
+    let t = e.to_string();
+    t.chars().filter(|c| !c.is_control()).take(200).collect()
+}
+
+/// Comprueba la ruta (con su consulta) antes de la red.
+pub fn validar_ruta(ruta: &str) -> Result<(), String> {
+    let (camino, consulta) = ruta.split_once('?').unwrap_or((ruta, ""));
+    if !camino.starts_with('/') || camino.starts_with("//") {
+        return Err("la ruta tiene que ser relativa a la API".into());
+    }
+    let minus = ruta.to_ascii_lowercase();
+    if ruta.contains('\\') || minus.contains("%2e") || minus.contains("%25") {
+        return Err("ruta con barra invertida o puntos codificados".into());
+    }
+    if camino[1..]
+        .split('/')
+        .any(|s| s.is_empty() || s == "." || s == "..")
+    {
+        return Err("ruta con segmentos vacios, . o ..".into());
+    }
+    let no_reservado = |b: u8| b.is_ascii_alphanumeric() || b"-._~".contains(&b);
+    if !camino.bytes().all(|b| no_reservado(b) || b == b'/') {
+        return Err("caracter no permitido en la ruta".into());
+    }
+    if !consulta
+        .bytes()
+        .all(|b| no_reservado(b) || b"=&:%,".contains(&b))
+    {
+        return Err("caracter no permitido en la consulta".into());
+    }
+    Ok(())
+}
+
+/// Codifica un valor de consulta (`%XX` todo lo que no sea no reservado).
+pub fn codificar(v: &str) -> String {
+    let mut s = String::with_capacity(v.len());
+    for b in v.bytes() {
+        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
+            s.push(char::from(b));
+        } else {
+            s.push_str(&format!("%{b:02X}"));
+        }
+    }
+    s
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn rutas_rechazadas_antes_de_la_red() {
+        for r in [
+            "repos/o/r",
+            "//evil.example/x",
+            "/repos/o/../r",
+            "/repos/o/./r",
+            "/repos/o/%2e%2e/r",
+            "/repos/o/%252e%252e/r",
+            "/repos/o\\r",
+            "/repos//r",
+            "/repos/o/r?x=<",
+            "/repos/o r",
+            "/repos/o/r%2Fx",
+        ] {
+            assert!(validar_ruta(r).is_err(), "{r}");
+        }
+        for r in [
+            "/user",
+            "/repos/o/r/pulls?head=o%3Afeature%2Fx&state=open",
+            "/repos/o/r/rules/branches/feature/task-1",
+        ] {
+            assert!(validar_ruta(r).is_ok(), "{r}");
+        }
+    }
+
+    #[test]
+    fn bases_validas_e_invalidas() {
+        assert!(Base::nueva("https://api.github.com").is_ok());
+        assert!(Base::nueva("https://git.empresa.com/api/v3/").is_ok());
+        assert!(Base::nueva("http://127.0.0.1:8080").is_ok());
+        for b in [
+            "http://api.github.com",
+            "http://127.0.0.1:80/x",
+            "http://localhost:80",
+            "https://user:pw@host",
+            "https://",
+            "ftp://x",
+            "https://host/../x",
+        ] {
+            assert!(Base::nueva(b).is_err(), "{b}");
+        }
+    }
+
+    #[test]
+    fn link_next() {
+        let r = Respuesta {
+            estado: 200,
+            cuerpo: String::new(),
+            link: Some(
+                "<https://api.github.com/x?page=1>; rel=\"prev\", <https://api.github.com/x?page=3>; rel=\"next\""
+                    .into(),
+            ),
+            restantes: None,
+            reintentar: None,
+            reinicio: None,
+        };
+        assert_eq!(r.siguiente(), Some("https://api.github.com/x?page=3"));
+        let c = Cliente::nuevo(Base::github());
+        assert_eq!(
+            c.ruta_de("https://api.github.com/x?page=3").ok().as_deref(),
+            Some("/x?page=3")
+        );
+        assert!(c.ruta_de("https://api.github.com.evil/x").is_err());
+        assert!(c.ruta_de("https://evil.example/x").is_err());
+    }
+
+    #[test]
+    fn codificar_valores() {
+        assert_eq!(codificar("o:feature/x y"), "o%3Afeature%2Fx%20y");
+    }
+}
diff --git a/crates/tc2/src/plataforma/mod.rs b/crates/tc2/src/plataforma/mod.rs
new file mode 100644
index 0000000..ea8cbf0
--- /dev/null
+++ b/crates/tc2/src/plataforma/mod.rs
@@ -0,0 +1,142 @@
+//! Puerto `Plataforma` (ADR F1-4 decision 1, F1-13 D1a) y su adaptador de
+//! GitHub (F2-6).
+//!
+//! El puerto es la unica parte del CLI que habla con la plataforma, por su
+//! API HTTP y sin `gh` (F1-14). Sus operaciones forman una lista cerrada y
+//! devuelven el contrato de salida de [`crate::contrato`]; cada una corre con
+//! la identidad que le da F1-6 y falla cerrada sin ella. GitLab (F2-9)
+//! implementa el mismo trait y pasa la misma bateria.
+//!
+//! Se usa desde la biblioteca: el binario no gana subcomandos hasta que los
+//! comandos de fase (F6-2 `review`, F7 `finish`, F3-8 `doctor`) lo llamen.
+
+pub mod credenciales;
+pub mod deteccion;
+pub mod errores;
+pub mod github;
+pub mod http;
+pub mod puertas;
+
+use std::collections::BTreeSet;
+
+pub use credenciales::{Credenciales, CredencialesEntorno, CredencialesFijas, Rol, Secreto};
+pub use deteccion::{Destino, Proveedor, detectar};
+pub use github::GitHub;
+pub use puertas::{EstadoCheck, Lecturas, evaluar};
+
+use crate::contrato::Informe;
+
+/// Resultado de una lectura tipada: el `Err` ya es el informe que se
+/// devuelve.
+pub type Paso<T> = Result<T, Informe>;
+
+/// Lista de personas por id numerico (F1-6, decision 3). La lee F2-13 del
+/// fichero de `.taskcode/`; aqui llega ya leida.
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct Personas(BTreeSet<u64>);
+
+impl Personas {
+    /// Lista con estos ids.
+    pub fn nuevas(ids: impl IntoIterator<Item = u64>) -> Self {
+        Self(ids.into_iter().collect())
+    }
+
+    /// Si el id es de una persona de la lista.
+    pub fn contiene(&self, id: u64) -> bool {
+        self.0.contains(&id)
+    }
+}
+
+/// Cuenta de la plataforma.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Usuario {
+    /// Nombre de usuario.
+    pub login: String,
+    /// Id numerico (el que se compara, no el login).
+    pub id: u64,
+    /// `User` o `Bot` (GitHub).
+    pub tipo: String,
+}
+
+impl Usuario {
+    /// Persona que cuenta: `User`, en la lista y no `web-flow` (F1-6,
+    /// decision 3, reglas 2, 3, 4 y 6).
+    pub fn es_persona(&self, personas: &Personas) -> bool {
+        self.tipo == "User" && self.login != "web-flow" && personas.contiene(self.id)
+    }
+}
+
+/// Checks obligatorios con su origen y garantias que la politica exige
+/// impuestas por la plataforma (F1-13, decision 5).
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct ModeloPuertas {
+    /// `(nombre, integration_id)`: un check solo cuenta si lo publico esa
+    /// App (G09).
+    pub checks: Vec<(String, u64)>,
+    /// Ids de garantia (`G03`...) que tienen que estar impuestas.
+    pub exigir_impuestas: Vec<String>,
+}
+
+/// Firma de un commit (contrato del firmante, F1-6 decision 3).
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Firma {
+    /// `commit.verification.verified`.
+    pub verificada: bool,
+    /// `commit.verification.reason`.
+    pub motivo: String,
+    /// La cuenta `committer`, si GitHub la asocia.
+    pub committer: Option<Usuario>,
+}
+
+impl Firma {
+    /// Si el commit cuenta como aprobacion de una persona: firma verificada
+    /// y committer persona de la lista (reglas 1 a 4; la 5, `Plan-Blob`, es
+    /// de quien llama).
+    pub fn cuenta_como_aprobacion(&self, personas: &Personas) -> bool {
+        self.verificada
+            && self
+                .committer
+                .as_ref()
+                .is_some_and(|c| c.es_persona(personas))
+    }
+}
+
+/// Reglas leidas de la plataforma (F1-5, `leer-puertas`). La comparacion con
+/// el modelo es de F3-8.
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct ReglasLeidas {
+    /// `false` si el plan no da rulesets (403 con aviso de plan).
+    pub plan_da_reglas: bool,
+    /// Nombres de los rulesets del repo.
+    pub rulesets: Vec<String>,
+    /// Tipos de regla que aplican a la rama.
+    pub reglas_rama: Vec<String>,
+}
+
+/// Puerto `Plataforma`: lista cerrada de operaciones (F1-4, F1-5, F1-6,
+/// F1-13). Las escrituras de puertas (`aplicar-puertas`) son de F3-1.
+pub trait Plataforma {
+    /// `abrir-pr` (agente): abre el PR de `rama` contra `base`, o devuelve
+    /// el que ya haya abierto (`sin-cambios`).
+    fn abrir_pr(&self, rama: &str, base: &str, titulo: &str, cuerpo: &str) -> Informe;
+
+    /// `estado-puertas` (agente o persona): checks con su origen,
+    /// aprobaciones de personas y rama al dia.
+    fn estado_puertas(&self, rol: Rol, pr: u64, modelo: &ModeloPuertas) -> Informe;
+
+    /// `merge-pr` (persona): merge commit del PR si su head sigue siendo
+    /// `sha` (el evaluado).
+    fn merge_pr(&self, pr: u64, sha: &str) -> Informe;
+
+    /// `leer-puertas` (agente o persona).
+    fn leer_puertas(&self, rol: Rol, rama: &str) -> Paso<ReglasLeidas>;
+
+    /// `leer-permisos-ci` (persona): que el token de la CI no pueda aprobar.
+    fn leer_permisos_ci(&self) -> Informe;
+
+    /// Firma del commit de `approve` (agente).
+    fn firma_commit(&self, sha: &str) -> Paso<Firma>;
+
+    /// Usuario autenticado con el token de ese rol.
+    fn identidad(&self, rol: Rol) -> Paso<Usuario>;
+}
diff --git a/crates/tc2/src/plataforma/puertas.rs b/crates/tc2/src/plataforma/puertas.rs
new file mode 100644
index 0000000..8eba041
--- /dev/null
+++ b/crates/tc2/src/plataforma/puertas.rs
@@ -0,0 +1,142 @@
+//! Evaluacion neutral de las puertas (F1-5 y F1-13, PoC M): de lo leido en
+//! la plataforma a la linea `estado-puertas ...`. Es pura y comun a GitHub y
+//! GitLab, para que las dos den la misma linea byte a byte.
+
+use crate::contrato::{Informe, Resultado};
+
+/// Nombre de la operacion.
+pub const ESTADO_PUERTAS: &str = "estado-puertas";
+
+/// Estado de un check obligatorio, contado solo si lo publico su origen.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum EstadoCheck {
+    /// Terminado en verde.
+    Ok,
+    /// Sin terminar o sin publicar por su origen.
+    Pendiente,
+    /// Terminado en rojo.
+    Fallo,
+}
+
+/// Lo leido de un PR abierto, ya traducido al modelo neutral.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Lecturas {
+    /// Numero del PR (o iid del MR).
+    pub pr: u64,
+    /// Checks obligatorios con su estado, en el orden del modelo.
+    pub checks: Vec<(String, EstadoCheck)>,
+    /// Aprobaciones de personas de la lista.
+    pub aprobaciones: usize,
+    /// Aprobaciones que no cuentan (bots, cuentas fuera de la lista).
+    pub ignoradas: usize,
+    /// Si la rama esta al dia con la base.
+    pub al_dia: bool,
+}
+
+/// `EstadoPuertas` neutral: rojo si un check fallo; esperando si falta algo;
+/// ok si no falta nada.
+pub fn evaluar(l: &Lecturas) -> Informe {
+    let pr = l.pr.to_string();
+    let mut rojo: Vec<String> = l
+        .checks
+        .iter()
+        .filter(|(_, e)| *e == EstadoCheck::Fallo)
+        .map(|(n, _)| n.clone())
+        .collect();
+    if !rojo.is_empty() {
+        rojo.sort();
+        return Informe::nuevo(ESTADO_PUERTAS, Resultado::Rechazada)
+            .con("pr", pr)
+            .con("motivo", "check-rojo")
+            .con_lista("checks", rojo)
+            .diagnostico("hay checks obligatorios en rojo: corrige y vuelve a empujar");
+    }
+    let mut falta: Vec<String> = l
+        .checks
+        .iter()
+        .filter(|(_, e)| *e != EstadoCheck::Ok)
+        .map(|(n, _)| n.clone())
+        .collect();
+    falta.sort();
+    if l.aprobaciones < 1 {
+        falta.push("aprobacion".into());
+    }
+    if !l.al_dia {
+        falta.push("al-dia".into());
+    }
+    let resultado = if falta.is_empty() {
+        Resultado::Ok
+    } else {
+        Resultado::Esperando
+    };
+    let i = Informe::nuevo(ESTADO_PUERTAS, resultado)
+        .con("pr", pr)
+        .con("aprobaciones", l.aprobaciones.to_string())
+        .con("ignoradas", l.ignoradas.to_string());
+    if falta.is_empty() {
+        i
+    } else {
+        let diag = format!(
+            "faltan puertas: {}; espera o atiende lo que falta (al-dia: lanza actualizar-rama)",
+            falta.join(", ")
+        );
+        i.con_lista("falta", falta).diagnostico(diag)
+    }
+}
+
+/// PR ya mergeado: nada que evaluar.
+pub fn mergeado(pr: u64) -> Informe {
+    Informe::nuevo(ESTADO_PUERTAS, Resultado::SinCambios)
+        .con("pr", pr.to_string())
+        .con("estado", "mergeado")
+}
+
+/// La politica exige impuesta una garantia que el plan no da.
+pub fn rechazada_por_plan(pr: u64, garantia: &str) -> Informe {
+    Informe::nuevo(ESTADO_PUERTAS, Resultado::Rechazada)
+        .con("pr", pr.to_string())
+        .con("motivo", "plan")
+        .con("garantia", garantia)
+        .diagnostico(format!(
+            "la politica del proyecto exige {garantia} impuesta y el plan de la plataforma no la da: cambia de plan o quita {garantia} de la politica"
+        ))
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn l(checks: &[(&str, EstadoCheck)], aprob: usize, al_dia: bool) -> Lecturas {
+        Lecturas {
+            pr: 7,
+            checks: checks.iter().map(|(n, e)| ((*n).to_owned(), *e)).collect(),
+            aprobaciones: aprob,
+            ignoradas: 0,
+            al_dia,
+        }
+    }
+
+    #[test]
+    fn rojo_gana_a_lo_que_falta() {
+        let i = evaluar(&l(
+            &[("b", EstadoCheck::Fallo), ("a", EstadoCheck::Fallo)],
+            0,
+            false,
+        ));
+        assert_eq!(
+            i.linea(),
+            "estado-puertas rechazada pr=7 motivo=check-rojo checks=a,b"
+        );
+        let i = evaluar(&l(&[("c", EstadoCheck::Pendiente)], 0, false));
+        assert_eq!(
+            i.linea(),
+            "estado-puertas esperando pr=7 aprobaciones=0 ignoradas=0 falta=c,aprobacion,al-dia"
+        );
+        assert_eq!(i.resultado.codigo(), 7);
+        let i = evaluar(&l(&[("c", EstadoCheck::Ok)], 1, true));
+        assert_eq!(
+            i.linea(),
+            "estado-puertas ok pr=7 aprobaciones=1 ignoradas=0"
+        );
+    }
+}
````

## Excluido del diff (5 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
Cargo.lock                                                                                | 564 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-
 tareas/02-en-curso/TASK-021/evidencia/e2e.log                                             |  24 +++++
 tareas/{01-en-diseno => 02-en-curso}/TASK-021/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{01-en-diseno => 02-en-curso}/TASK-021/planificacion/plan-final.md                 |   0
 tareas/{01-en-diseno => 02-en-curso}/TASK-021/tarea.md                                    |   8 +-
 5 files changed, 594 insertions(+), 2 deletions(-)
````

## Tests del diff (4 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/comun/http_mock.rs" ":(literal)crates/tc2/tests/comun/mod.rs" ":(literal)crates/tc2/tests/plataforma.rs" ":(literal)crates/tc2/tests/plataforma_e2e.rs"
````

````
crates/tc2/tests/comun/http_mock.rs | 155 +++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/comun/mod.rs       |   1 +
 crates/tc2/tests/plataforma.rs      | 563 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/plataforma_e2e.rs  | 384 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 4 files changed, 1103 insertions(+)
````
