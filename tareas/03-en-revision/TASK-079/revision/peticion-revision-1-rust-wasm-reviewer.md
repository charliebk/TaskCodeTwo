# Peticion de revision — TASK-079 (ronda 1)

- Tarea: TASK-079 — F3-5b status y roadmap siempre frescos
- Rama revisada: feature/task-079-f3-5b-status-y-roadmap-siempre-frescos
- Rama base: develop
- Commit revisado (HEAD): 67b8a3455d064a006a5464c96db0e3877b4a4834
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-079 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-079/tarea.md
- tareas/03-en-revision/TASK-079/planificacion/
- crates/tc2/src/alta.rs
- crates/tc2/src/desinstalar/proyecto.rs
- crates/tc2/src/doctor/estado.rs
- crates/tc2/src/doctor/mod.rs
- crates/tc2/src/estado/frescura.rs
- crates/tc2/src/estado/lectura.rs
- crates/tc2/src/estado/mod.rs
- crates/tc2/src/estado/pendientes.rs
- crates/tc2/src/estado/transaccion.rs
- crates/tc2/src/huella.rs
- crates/tc2/src/i18n/catalogo/estado.rs
- crates/tc2/src/i18n/catalogo/tablero.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/roadmap/mod.rs
- crates/tc2/src/siguiente.rs
- crates/tc2/src/tablero.rs
- crates/tc2/tests/desinstalar.rs
- crates/tc2/tests/frescura.rs
- crates/tc2/tests/siguiente.rs
- crates/tc2/tests/tablero.rs

## Commits a revisar (git log develop..HEAD)

````
67b8a34 docs(TASK-079): autorrevision con rust-wasm-reviewer y code-quality-reviewer
69d39bd refactor(TASK-079): formato de frescura.rs con cargo fmt
dfd9270 docs(TASK-079): adendas de F1-4 y F1-2 por la lectura fresca
26ceae8 test(TASK-079): criterios C1 a C9 de la lectura fresca
aedaf7a feat(TASK-079): la transaccion renueva la marca tras publicar
c76837f feat(TASK-079): status, board, next y roadmap traen antes de mostrar
07396f2 feat(TASK-079): lectura fresca de la rama de estado
716af9d refactor(TASK-079): traer_estado extraido de la transaccion
0a01917 feat(TASK-079): marca de sincronizacion y formato de la frescura
d2ac114 feat(TASK-079): fichero de la marca de sincronizacion en la huella
26330f7 chore(TASK-079): tarea en curso
````

## Diff de tu dominio (20 fichero(s) de 22; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/alta.rs b/crates/tc2/src/alta.rs
index 0f8afc7..088e8ca 100644
--- a/crates/tc2/src/alta.rs
+++ b/crates/tc2/src/alta.rs
@@ -603,6 +603,20 @@ pub(crate) fn hoy() -> String {
     fecha_de_dias(i64::try_from(segundos / 86_400).unwrap_or(0))
 }
 
+/// Instante UTC `AAAA-MM-DDTHH:MM:SSZ` de unos segundos desde 1970 (F3-5b,
+/// D6a), con la fecha de [`fecha_de_dias`].
+pub(crate) fn instante_utc(segundos: u64) -> String {
+    let dias = i64::try_from(segundos / 86_400).unwrap_or(0);
+    let s = segundos % 86_400;
+    format!(
+        "{}T{:02}:{:02}:{:02}Z",
+        fecha_de_dias(dias),
+        s / 3600,
+        s % 3600 / 60,
+        s % 60
+    )
+}
+
 /// Fecha civil de un numero de dias desde 1970-01-01 (algoritmo de Howard
 /// Hinnant, `civil_from_days`).
 pub(crate) fn fecha_de_dias(dias: i64) -> String {
@@ -964,6 +978,9 @@ mod tests {
         assert_eq!(fecha_de_dias(11_016), "2000-02-29");
         assert_eq!(fecha_de_dias(20_772), "2026-11-15");
         assert_eq!(fecha_de_dias(-1), "1969-12-31");
+        assert_eq!(instante_utc(0), "1970-01-01T00:00:00Z");
+        assert_eq!(instante_utc(951_868_799), "2000-02-29T23:59:59Z");
+        assert_eq!(instante_utc(1_794_931_452), "2026-11-17T16:04:12Z");
     }
 
     #[test]
diff --git a/crates/tc2/src/desinstalar/proyecto.rs b/crates/tc2/src/desinstalar/proyecto.rs
index 236e257..30dcf44 100644
--- a/crates/tc2/src/desinstalar/proyecto.rs
+++ b/crates/tc2/src/desinstalar/proyecto.rs
@@ -173,7 +173,8 @@ pub(super) fn plan(op: &Opciones, c: &Contexto<'_>, v: &mut Vec<Accion>) -> Resu
             ));
         }
     }
-    // La carpeta propia en la comun: cerrojo, cache de identidad, indices.
+    // La carpeta propia en la comun: cerrojo, cache de identidad, marca de
+    // sincronizacion (F3-5b), indices. Se borra entera.
     let propia = huella::en_comun(&comun);
     if propia.is_dir() {
         v.push(Accion::borra(
diff --git a/crates/tc2/src/doctor/estado.rs b/crates/tc2/src/doctor/estado.rs
index fd3f0f4..a3f2418 100644
--- a/crates/tc2/src/doctor/estado.rs
+++ b/crates/tc2/src/doctor/estado.rs
@@ -1,6 +1,7 @@
 //! Rama de estado e instalacion.
 //!
-//! - La rama de estado se lee como `board` (lo ultimo traido, sin `fetch`):
+//! - La rama de estado se lee sobre la misma ref que `board`, pero sin
+//!   `fetch` (lo ultimo traido; D4a de F3-5b: `doctor` solo lee):
 //!   ausente, `falla motivo=sin-rama-de-estado`; una ref que existe y no es
 //!   un commit, `falla motivo=ref-ilegible ref=...`; un `tarea.md` que no se
 //!   entiende, `falla motivo=tarea-ilegible ids=...`.
diff --git a/crates/tc2/src/doctor/mod.rs b/crates/tc2/src/doctor/mod.rs
index fe0f6a7..bf59fec 100644
--- a/crates/tc2/src/doctor/mod.rs
+++ b/crates/tc2/src/doctor/mod.rs
@@ -2,7 +2,7 @@
 //! instalacion y a la plataforma para que TaskCodeTwo funcione, con el
 //! arreglo de cada fallo (RF-64).
 //!
-//! Solo lee, como `board`: ni `fetch` ni escrituras en el repo, en
+//! Solo lee: ni `fetch` (`board`, desde F3-5b, si trae) ni escrituras en el repo, en
 //! `<git-common-dir>/taskcodetwo/` ni en la plataforma (solo `GET`). Cada
 //! comprobacion deja un [`Hallazgo`] (`ok`, `aviso`, `falla` o
 //! `no-comprobado`); el agregador ([`informe`]) es puro.
@@ -10,7 +10,8 @@
 //! Salida (F1-4 decision 4, F1-13 seccion C): como mucho 5 lineas en
 //! stdout. La primera, `doctor <ok|rechazada> fallos=N avisos=N
 //! no-comprobado=N plataforma=... [plan=... impuestas=N sustituidas=N
-//! no-disponibles=N [no-comprobadas=N]] [commit=...]`; detras, los
+//! no-disponibles=N [no-comprobadas=N]] [commit=...] [sincronizado=...]`
+//! (la hora de la marca de F3-5b, sin `fetch`); detras, los
 //! hallazgos que no son `ok`,
 //! los peores primero, y si no caben, `y N mas: tc2 doctor --json`. Con
 //! `--json`, una linea con todos los hallazgos (tambien los `ok`). Codigo 3
@@ -180,6 +181,9 @@ pub struct Resumen {
     pub capacidades: Option<capacidades::Cuentas>,
     /// Commit de la rama de estado (12), si se leyo.
     pub commit: Option<String>,
+    /// Hora del ultimo `fetch` con exito de la rama de estado o `nunca`
+    /// (F3-5b, D4a): de la marca, sin `fetch`.
+    pub sincronizado: Option<String>,
 }
 
 /// Cuantos hallazgos hay en `e`.
@@ -219,6 +223,9 @@ pub fn informe(hs: &[Hallazgo], r: &Resumen) -> Informe {
     if let Some(c) = &r.commit {
         i = i.con("commit", c.clone());
     }
+    if let Some(s) = &r.sincronizado {
+        i = i.con("sincronizado", s.clone());
+    }
     if let Some(primero) = ordenados(hs)
         .into_iter()
         .find(|h| h.estado == Estado::Falla)
@@ -318,12 +325,16 @@ pub fn comprobar(e: &Entorno<'_>) -> Result<(Vec<Hallazgo>, Resumen), Informe> {
     hs.extend(estado::instalacion(e.git, e.dir));
     let (h, plataforma, capacidades) = plataforma::comprobar(e);
     hs.extend(h);
+    // La hora de la marca, sin `fetch` (D4a de F3-5b: doctor solo lee).
+    let sincronizado =
+        crate::estado::sincronizado_texto(crate::estado::sincronizado_en(e.git, e.dir));
     Ok((
         hs,
         Resumen {
             plataforma,
             capacidades,
             commit,
+            sincronizado: Some(sincronizado),
         },
     ))
 }
@@ -363,9 +374,24 @@ mod tests {
             plataforma: "ok".to_owned(),
             capacidades: None,
             commit: Some("0123456789ab".to_owned()),
+            sincronizado: None,
         }
     }
 
+    /// C8 (parte pura): `sincronizado=` va detras de `commit=` cuando se
+    /// sabe.
+    #[test]
+    fn sincronizado_tras_el_commit() {
+        let r = Resumen {
+            sincronizado: Some("2026-11-17T16:04:12Z".to_owned()),
+            ..resumen()
+        };
+        assert_eq!(
+            informe(&[], &r).linea(),
+            "doctor ok fallos=0 avisos=0 no-comprobado=0 plataforma=ok commit=0123456789ab sincronizado=2026-11-17T16:04:12Z"
+        );
+    }
+
     /// C7: la primera linea fija; detras, los peores primero; con mas de 4,
     /// 3 y `y N mas`; nunca mas de 5 lineas; JSON con todos y sin ANSI.
     #[test]
diff --git a/crates/tc2/src/estado/frescura.rs b/crates/tc2/src/estado/frescura.rs
new file mode 100644
index 0000000..c88f3b3
--- /dev/null
+++ b/crates/tc2/src/estado/frescura.rs
@@ -0,0 +1,451 @@
+//! Frescura de la lectura (F3-5b, RR-12): la hora del ultimo `fetch` con
+//! exito de la rama de estado y por que la ultima lectura no pudo traerla.
+//!
+//! - [`Marca`] guarda la hora en `<git-common-dir>/taskcodetwo/sincronizacion`
+//!   (D3a), como la cache de identidad: temporal y `rename`, de modo que quien
+//!   lee no ve nunca un fichero a medias. Un fichero que no se entiende, o
+//!   demasiado grande, es «nunca»: la hora nunca sale mas fresca de lo real.
+//! - [`Frescura`] es la unica fuente del formato: las claves `remoto=` y
+//!   `sincronizado=` ([`Frescura::anotar`]) y el aviso de stderr
+//!   ([`Frescura::aviso`]). La hora va en UTC, `AAAA-MM-DDTHH:MM:SSZ`, mas
+//!   «hace N» en los textos para la persona (D6a).
+
+use std::path::{Path, PathBuf};
+
+use super::{OP, RAMA};
+use crate::contrato::Informe;
+use crate::git::{FalloRemoto, Git, REMOTO, Subcomando, clasificar_fallo_remoto, correr};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::estado as txt;
+
+/// Como fue el `fetch` de la rama de estado.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub(crate) enum Traida {
+    /// Salio bien: `origin/taskcodetwo/tareas` es lo ultimo publicado.
+    Hecha,
+    /// `origin` no tiene la rama de estado.
+    SinRama,
+    /// No salio: la clasificacion y el stderr de git.
+    Fallo {
+        /// Por que.
+        fallo: FalloRemoto,
+        /// stderr entero.
+        stderr: String,
+        /// Primera linea de error.
+        primera: String,
+    },
+}
+
+impl Traida {
+    /// Lo que dice la lectura de este intento.
+    pub(crate) fn remoto(&self) -> Remoto {
+        match self {
+            Self::Hecha => Remoto::Consultado,
+            Self::SinRama => Remoto::SinRama,
+            Self::Fallo { fallo, .. } => match fallo {
+                FalloRemoto::SinRed => Remoto::SinRed,
+                FalloRemoto::Local => Remoto::ErrorLocal,
+                FalloRemoto::RamaInexistente => Remoto::SinRama,
+                otro => Remoto::Denegado(otro.motivo().to_owned()),
+            },
+        }
+    }
+}
+
+/// Trae solo la rama de estado (`fetch` de `refs/heads/taskcodetwo/tareas` a
+/// `refs/remotes/origin/taskcodetwo/tareas`, sin `+`) y, si sale bien,
+/// registra `ahora` en la marca. Es el `fetch` de cada intento de la
+/// transaccion y el de la lectura fresca: uno solo, sin copias. Un fallo al
+/// escribir la marca no se propaga: la hora se queda la anterior, nunca mas
+/// fresca de lo real. `Err` solo si no se pudo lanzar git.
+pub(crate) fn traer_estado(
+    git: &Git,
+    principal: &Path,
+    marca: &Marca,
+    ahora: u64,
+) -> Result<Traida, Informe> {
+    let refspec = format!("refs/heads/{RAMA}:refs/remotes/{REMOTO}/{RAMA}");
+    let s = correr(
+        git,
+        principal,
+        OP,
+        Subcomando::Fetch,
+        &["-q", REMOTO, &refspec],
+    )?;
+    if s.exito() {
+        let _ = marca.registrar(ahora);
+        return Ok(Traida::Hecha);
+    }
+    Ok(match clasificar_fallo_remoto(&s.stderr) {
+        FalloRemoto::RamaInexistente => Traida::SinRama,
+        fallo => Traida::Fallo {
+            fallo,
+            primera: s.primera_linea_error(),
+            stderr: s.stderr,
+        },
+    })
+}
+
+/// Bytes como mucho del fichero de la marca (un numero y un salto de linea).
+const LIMITE_MARCA: u64 = 64;
+
+/// Marca de la ultima sincronizacion de la rama de estado.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Marca {
+    ruta: PathBuf,
+}
+
+impl Marca {
+    /// La marca del repo cuya carpeta comun es `comun`.
+    pub fn en(comun: &Path) -> Self {
+        Self {
+            ruta: crate::huella::en_comun(comun).join(crate::huella::FICHERO_SINCRONIZACION),
+        }
+    }
+
+    /// Ruta del fichero.
+    pub fn ruta(&self) -> &Path {
+        &self.ruta
+    }
+
+    /// Segundos desde 1970 del ultimo `fetch` con exito; `None` si no hay
+    /// marca o no se entiende (se trata como «nunca»).
+    pub fn leer(&self) -> Option<u64> {
+        use std::io::Read;
+        let mut texto = String::new();
+        std::fs::File::open(&self.ruta)
+            .ok()?
+            .take(LIMITE_MARCA + 1)
+            .read_to_string(&mut texto)
+            .ok()?;
+        if texto.len() as u64 > LIMITE_MARCA {
+            return None;
+        }
+        let t = texto.strip_suffix('\n').unwrap_or(&texto);
+        if t.is_empty() || !t.bytes().all(|b| b.is_ascii_digit()) {
+            return None;
+        }
+        t.parse().ok()
+    }
+
+    /// Guarda `ahora`. Escribe un temporal y lo renombra.
+    pub fn registrar(&self, ahora: u64) -> std::io::Result<()> {
+        if let Some(padre) = self.ruta.parent() {
+            std::fs::create_dir_all(padre)?;
+        }
+        let tmp = self
+            .ruta
+            .with_extension(format!("tmp-{}", std::process::id()));
+        std::fs::write(&tmp, format!("{ahora}\n"))?;
+        std::fs::rename(&tmp, &self.ruta).inspect_err(|_| {
+            let _ = std::fs::remove_file(&tmp);
+        })
+    }
+}
+
+/// Que paso al intentar traer la rama de estado antes de mostrar.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum Remoto {
+    /// El `fetch` salio bien: se muestra lo ultimo publicado.
+    Consultado,
+    /// Sin red (o un fallo que no se reconoce): se muestra lo ultimo traido.
+    SinRed,
+    /// `origin` responde pero no deja leer; el motivo es el de
+    /// [`crate::git::FalloRemoto::motivo`] (`credenciales`,
+    /// `remoto-no-existe`...).
+    Denegado(String),
+    /// Fallo del propio clon al traer (p. ej. sin permiso en `.git`).
+    ErrorLocal,
+    /// `origin` responde pero no tiene la rama de estado.
+    SinRama,
+    /// Sin `origin`: la rama local es el estado y no se trae nada.
+    SinRemoto,
+}
+
+impl Remoto {
+    /// Valor de `remoto=`.
+    pub fn texto(&self) -> &'static str {
+        match self {
+            Self::Consultado => "consultado",
+            Self::SinRed => "sin-red",
+            Self::Denegado(_) => "denegado",
+            Self::ErrorLocal => "error-local",
+            Self::SinRama => "sin-rama",
+            Self::SinRemoto => "sin-remoto",
+        }
+    }
+
+    /// Por que no es fresca, para el aviso; `None` si no hay nada que avisar.
+    fn porque(&self) -> Option<Mensaje> {
+        Some(match self {
+            Self::Consultado | Self::SinRemoto => return None,
+            Self::SinRed => Mensaje::nuevo(&txt::PORQUE_SIN_RED),
+            Self::Denegado(m) => Mensaje::nuevo(&txt::PORQUE_DENEGADO).con("motivo", m.clone()),
+            Self::ErrorLocal => Mensaje::nuevo(&txt::PORQUE_ERROR_LOCAL),
+            Self::SinRama => Mensaje::nuevo(&txt::PORQUE_SIN_RAMA),
+        })
+    }
+}
+
+/// Lo que se sabe de la frescura de una lectura.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Frescura {
+    /// Como fue el intento de traer la rama de estado.
+    pub remoto: Remoto,
+    /// Ultimo `fetch` con exito (segundos desde 1970), si lo hubo.
+    pub sincronizado: Option<u64>,
+}
+
+/// Valor de `sincronizado=`: la hora UTC o `nunca`.
+pub fn sincronizado_texto(sincronizado: Option<u64>) -> String {
+    sincronizado.map_or_else(|| "nunca".to_owned(), crate::alta::instante_utc)
+}
+
+/// «hace N» entre `entonces` y `ahora` (un `entonces` del futuro, por un
+/// reloj desajustado, cuenta como «hace menos de 1 min»).
+pub fn hace(entonces: u64, ahora: u64) -> Mensaje {
+    let s = ahora.saturating_sub(entonces);
+    match s {
+        0..60 => Mensaje::nuevo(&txt::HACE_MENOS_DE_UN_MINUTO),
+        60..3600 => Mensaje::nuevo(&txt::HACE_MINUTOS).con("n", (s / 60).to_string()),
+        3600..172_800 => Mensaje::nuevo(&txt::HACE_HORAS).con("n", (s / 3600).to_string()),
+        _ => Mensaje::nuevo(&txt::HACE_DIAS).con("n", (s / 86_400).to_string()),
+    }
+}
+
+impl Frescura {
+    /// Anade `remoto=` y `sincronizado=` al informe.
+    #[must_use]
+    pub fn anotar(&self, i: Informe) -> Informe {
+        i.con("remoto", self.remoto.texto())
+            .con("sincronizado", self.texto())
+    }
+
+    /// Valor de `sincronizado=`.
+    pub fn texto(&self) -> String {
+        sincronizado_texto(self.sincronizado)
+    }
+
+    /// Lo que se ensena en el texto para la persona (linea de `status` y
+    /// cabecera de `board`): la hora, o `None` sin `origin` (no hay nada que
+    /// sincronizar).
+    pub fn en_texto(&self) -> Option<String> {
+        (self.remoto != Remoto::SinRemoto).then(|| self.texto())
+    }
+
+    /// Aviso para stderr si la lectura no trajo lo ultimo publicado: por
+    /// que, y desde cuando es lo que se muestra (o que no se sincronizo
+    /// nunca). `None` si se consulto o no hay `origin`.
+    pub fn aviso(&self, ahora: u64) -> Option<Mensaje> {
+        let porque = self.remoto.porque()?;
+        Some(match self.sincronizado {
+            Some(s) => Mensaje::nuevo(&txt::AVISO_NO_FRESCO)
+                .con_mensaje("porque", porque)
+                .con("hora", crate::alta::instante_utc(s))
+                .con_mensaje("hace", hace(s, ahora)),
+            None => Mensaje::nuevo(&txt::AVISO_NUNCA_SINCRONIZADO).con_mensaje("porque", porque),
+        })
+    }
+
+    /// Una linea para stderr con la hora de la ultima sincronizacion (la de
+    /// `roadmap`, cuyo stdout es el ROADMAP byte a byte): el aviso si lo
+    /// hay; si no, la hora; nada sin `origin`.
+    pub fn linea(&self, ahora: u64) -> Option<Mensaje> {
+        if let Some(a) = self.aviso(ahora) {
+            return Some(a);
+        }
+        match (&self.remoto, self.sincronizado) {
+            (Remoto::Consultado, Some(s)) => Some(
+                Mensaje::nuevo(&txt::SINCRONIZADO)
+                    .con("hora", crate::alta::instante_utc(s))
+                    .con_mensaje("hace", hace(s, ahora)),
+            ),
+            _ => None,
+        }
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::contrato::Resultado;
+    use crate::i18n::Idioma;
+
+    /// 2026-11-17T16:04:12Z.
+    const T: u64 = 1_794_931_452;
+
+    fn carpeta(nombre: &str) -> PathBuf {
+        let d = std::env::temp_dir()
+            .join("tc2-tests")
+            .join(format!("frescura-{nombre}-{}", std::process::id()));
+        let _ = std::fs::remove_dir_all(&d);
+        d
+    }
+
+    #[test]
+    fn marca_ida_y_vuelta() -> std::io::Result<()> {
+        let comun = carpeta("ida");
+        let m = Marca::en(&comun);
+        assert_eq!(m.ruta(), comun.join("taskcodetwo").join("sincronizacion"));
+        assert_eq!(m.leer(), None, "sin fichero es nunca");
+        m.registrar(T)?;
+        assert_eq!(m.leer(), Some(T));
+        m.registrar(T + 5)?;
+        assert_eq!(m.leer(), Some(T + 5));
+        // Sin temporales sueltos.
+        let sueltos: Vec<_> = std::fs::read_dir(comun.join("taskcodetwo"))?
+            .flatten()
+            .map(|e| e.file_name())
+            .collect();
+        assert_eq!(sueltos.len(), 1, "{sueltos:?}");
+        let _ = std::fs::remove_dir_all(&comun);
+        Ok(())
+    }
+
+    #[test]
+    fn marca_corrupta_o_enorme_es_nunca() -> std::io::Result<()> {
+        let comun = carpeta("corrupta");
+        let m = Marca::en(&comun);
+        std::fs::create_dir_all(comun.join("taskcodetwo"))?;
+        for malo in [
+            "".to_owned(),
+            "\n".to_owned(),
+            "abc\n".to_owned(),
+            "-5\n".to_owned(),
+            "12 34\n".to_owned(),
+            "99999999999999999999999\n".to_owned(),
+            format!("{}\n", "1".repeat(100)),
+        ] {
+            std::fs::write(m.ruta(), &malo)?;
+            assert_eq!(m.leer(), None, "{malo:?}");
+        }
+        std::fs::write(m.ruta(), "1794931452")?;
+        assert_eq!(m.leer(), Some(T), "sin salto final tambien vale");
+        let _ = std::fs::remove_dir_all(&comun);
+        Ok(())
+    }
+
+    #[test]
+    fn hace_en_minutos_horas_y_dias() {
+        let es = |a: u64, b: u64| hace(a, b).texto(Idioma::Es);
+        assert_eq!(es(T, T), "hace menos de 1 min");
+        assert_eq!(es(T + 10, T), "hace menos de 1 min", "reloj desajustado");
+        assert_eq!(es(T, T + 59), "hace menos de 1 min");
+        assert_eq!(es(T, T + 60), "hace 1 min");
+        assert_eq!(es(T, T + 3599), "hace 59 min");
+        assert_eq!(es(T, T + 3600), "hace 1 h");
+        assert_eq!(es(T, T + 172_799), "hace 47 h");
+        assert_eq!(es(T, T + 3 * 86_400), "hace 3 dias");
+        assert_eq!(hace(T, T + 120).texto(Idioma::En), "2 min ago");
+    }
+
+    /// `anotar` y `aviso` por cada `Remoto`, en los dos idiomas.
+    #[test]
+    fn anotar_y_aviso_por_cada_remoto() {
+        let base = Informe::nuevo("status", Resultado::Ok);
+        let casos = [
+            (Remoto::Consultado, "consultado", None, None),
+            (
+                Remoto::SinRed,
+                "sin-red",
+                Some(
+                    "aviso: sin red, no se pudo traer la rama de estado de origin; se muestra lo que este clon trajo el 2026-11-17T16:04:12Z (hace 3 min)",
+                ),
+                Some(
+                    "warning: no network, the state branch could not be fetched from origin; showing what this clone fetched at 2026-11-17T16:04:12Z (3 min ago)",
+                ),
+            ),
+            (
+                Remoto::Denegado("credenciales".to_owned()),
+                "denegado",
+                Some(
+                    "aviso: origin no deja leer la rama de estado (credenciales); se muestra lo que este clon trajo el 2026-11-17T16:04:12Z (hace 3 min)",
+                ),
+                Some(
+                    "warning: origin does not allow reading the state branch (credenciales); showing what this clone fetched at 2026-11-17T16:04:12Z (3 min ago)",
+                ),
+            ),
+            (
+                Remoto::ErrorLocal,
+                "error-local",
+                Some(
+                    "aviso: no se pudo traer la rama de estado por un fallo de este clon; se muestra lo que este clon trajo el 2026-11-17T16:04:12Z (hace 3 min)",
+                ),
+                Some(
+                    "warning: the state branch could not be fetched because of a failure in this clone; showing what this clone fetched at 2026-11-17T16:04:12Z (3 min ago)",
+                ),
+            ),
+            (
+                Remoto::SinRama,
+                "sin-rama",
+                Some(
+                    "aviso: origin no tiene la rama de estado; se muestra lo que este clon trajo el 2026-11-17T16:04:12Z (hace 3 min)",
+                ),
+                Some(
+                    "warning: origin does not have the state branch; showing what this clone fetched at 2026-11-17T16:04:12Z (3 min ago)",
+                ),
+            ),
+            (Remoto::SinRemoto, "sin-remoto", None, None),
+        ];
+        for (remoto, valor, es, en) in casos {
+            let f = Frescura {
+                remoto: remoto.clone(),
+                sincronizado: Some(T),
+            };
+            assert_eq!(
+                f.anotar(base.clone()).linea(),
+                format!("status ok remoto={valor} sincronizado=2026-11-17T16:04:12Z")
+            );
+            let a = f.aviso(T + 180);
+            assert_eq!(a.as_ref().map(|m| m.texto(Idioma::Es)).as_deref(), es);
+            assert_eq!(a.as_ref().map(|m| m.texto(Idioma::En)).as_deref(), en);
+            // Nunca sincronizado: `nunca` y, si habia que avisar, el aviso
+            // de nunca.
+            let nunca = Frescura {
+                remoto,
+                sincronizado: None,
+            };
+            assert_eq!(
+                nunca.anotar(base.clone()).linea(),
+                format!("status ok remoto={valor} sincronizado=nunca")
+            );
+            let a = nunca.aviso(T).map(|m| m.texto(Idioma::Es));
+            assert_eq!(a.is_some(), es.is_some());
+            if let Some(a) = a {
+                assert!(
+                    a.ends_with(
+                        " y este clon no la ha sincronizado nunca; lo que se muestra puede no estar al dia"
+                    ),
+                    "{a}"
+                );
+            }
+        }
+    }
+
+    #[test]
+    fn en_texto_y_linea() {
+        let f = |remoto, sincronizado| Frescura {
+            remoto,
+            sincronizado,
+        };
+        assert_eq!(
+            f(Remoto::Consultado, Some(T)).en_texto().as_deref(),
+            Some("2026-11-17T16:04:12Z")
+        );
+        assert_eq!(f(Remoto::SinRed, None).en_texto().as_deref(), Some("nunca"));
+        assert_eq!(f(Remoto::SinRemoto, Some(T)).en_texto(), None);
+        assert_eq!(
+            f(Remoto::Consultado, Some(T))
+                .linea(T + 30)
+                .map(|m| m.texto(Idioma::Es))
+                .as_deref(),
+            Some("estado sincronizado con origin el 2026-11-17T16:04:12Z (hace menos de 1 min)")
+        );
+        assert!(
+            f(Remoto::SinRed, Some(T))
+                .linea(T)
+                .is_some_and(|m| m.texto(Idioma::Es).starts_with("aviso: sin red"))
+        );
+        assert_eq!(f(Remoto::SinRemoto, None).linea(T), None);
+    }
+}
diff --git a/crates/tc2/src/estado/lectura.rs b/crates/tc2/src/estado/lectura.rs
index b366395..de2b626 100644
--- a/crates/tc2/src/estado/lectura.rs
+++ b/crates/tc2/src/estado/lectura.rs
@@ -6,10 +6,11 @@
 use std::collections::{BTreeMap, BTreeSet};
 use std::path::{Path, PathBuf};
 
+use super::frescura::{Frescura, Marca, Remoto, traer_estado};
 use super::transaccion::Fallo;
 use super::{CARPETA, OP, RAMA};
 use crate::contrato::{Informe, Resultado};
-use crate::git::estado::{hay_remoto, leer_blob, leer_blobs, listar_arbol};
+use crate::git::estado::{dir_comun, hay_remoto, leer_blob, leer_blobs, listar_arbol};
 use crate::git::{Cambio, Git, REMOTO, localizar_principal};
 
 /// Lo que una operacion puede leer del estado.
@@ -79,6 +80,11 @@ impl<'a> Vista<'a> {
         self.commit.as_deref()
     }
 
+    /// Carpeta desde la que se lee (la principal del repo).
+    pub(crate) fn dir(&self) -> &Path {
+        &self.dir
+    }
+
     /// Superpone cambios (los posteriores ganan).
     pub(crate) fn superponer(&mut self, cambios: Vec<Cambio>) {
         for c in cambios {
@@ -139,26 +145,117 @@ impl Estado for Vista<'_> {
     }
 }
 
-/// La carpeta principal y la ref de la rama de estado que lee [`leer`]: la
-/// remota si hay `origin`, la local si no. `doctor` (F3-8) decide sobre la
-/// misma ref.
-pub(crate) fn referencia_de_lectura(git: &Git, dir: &Path) -> Result<(PathBuf, String), Informe> {
+/// La carpeta principal, la ref de la rama de estado que se lee y si hay
+/// `origin`.
+fn lectura_de(git: &Git, dir: &Path) -> Result<(PathBuf, String, bool), Informe> {
     let principal = localizar_principal(git, dir)?;
-    let referencia = if hay_remoto(git, &principal, OP)? {
+    let remoto = hay_remoto(git, &principal, OP)?;
+    let referencia = if remoto {
         format!("refs/remotes/{REMOTO}/{RAMA}")
     } else {
         format!("refs/heads/{RAMA}")
     };
-    Ok((principal, referencia))
+    Ok((principal, referencia, remoto))
+}
+
+/// La carpeta principal y la ref de la rama de estado que lee [`leer`]: la
+/// remota si hay `origin`, la local si no. `doctor` (F3-8) decide sobre la
+/// misma ref.
+pub(crate) fn referencia_de_lectura(git: &Git, dir: &Path) -> Result<(PathBuf, String), Informe> {
+    lectura_de(git, dir).map(|(p, r, _)| (p, r))
 }
 
 /// Vista del estado tal como lo conoce este clon: lo ultimo traido de
-/// `origin/taskcodetwo/tareas` (sin `fetch`: la lectura fresca es de F3-5b) o,
-/// en un repo sin remoto, la rama local. Sin rama de estado → `rechazada`.
+/// `origin/taskcodetwo/tareas` (sin `fetch`; la lectura fresca es
+/// [`leer_fresco`]) o, en un repo sin remoto, la rama local. Sin rama de
+/// estado → `rechazada`. La usan `doctor`, `import` y `start`.
 pub fn leer<'a>(git: &'a Git, dir: &Path) -> Result<Vista<'a>, Informe> {
     let (principal, referencia) = referencia_de_lectura(git, dir)?;
-    match crate::git::commit_de(git, &principal, OP, &referencia)? {
-        Some(c) => Vista::nueva(git, &principal, Some(c)),
+    vista_de(git, &principal, &referencia)
+}
+
+/// Lectura fresca (F3-5b): trae la rama de estado de `origin` (el mismo
+/// `fetch` que la transaccion), renueva la marca si sale bien y da la vista
+/// de la misma ref que [`leer`] con su [`Frescura`]. Un fallo del remoto no
+/// es error: se lee lo ultimo traido y la frescura dice por que. Sin
+/// `origin` no hay `fetch`. No toma el cerrojo de estado ni escribe nada mas
+/// que `refs/remotes/origin/taskcodetwo/tareas` y la marca.
+pub fn leer_fresco<'a>(
+    git: &'a Git,
+    dir: &Path,
+    ahora: u64,
+) -> Result<(Vista<'a>, Frescura), Informe> {
+    let (principal, referencia, remoto) = lectura_de(git, dir)?;
+    let marca = Marca::en(&dir_comun(git, &principal, OP)?);
+    let remoto = if remoto {
+        traer_estado(git, &principal, &marca, ahora)?.remoto()
+    } else {
+        Remoto::SinRemoto
+    };
+    let frescura = Frescura {
+        remoto,
+        sincronizado: marca.leer(),
+    };
+    Ok((vista_de(git, &principal, &referencia)?, frescura))
+}
+
+/// Hora de la ultima sincronizacion del clon de `dir`, sin `fetch` (la de
+/// `doctor`, D4a). `None` si no se sincronizo nunca o no se puede saber.
+pub fn sincronizado_en(git: &Git, dir: &Path) -> Option<u64> {
+    let principal = localizar_principal(git, dir).ok()?;
+    Marca::en(&dir_comun(git, &principal, OP).ok()?).leer()
+}
+
+/// Lo que `status` dice del propio clon (F3-5b; F1-2 D5 lo encarga).
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub struct Local {
+    /// Operaciones pendientes de publicar.
+    pub pendientes: usize,
+    /// `con-cambios` (la carpeta montada no esta en lo leido y tiene cambios
+    /// sin publicar, que impiden adelantarla) o `diverge` (la rama local tiene
+    /// commits que no estan en lo leido); `None` si no hay nada que avisar.
+    pub tareas: Option<&'static str>,
+}
+
+/// El [`Local`] del clon de `vista`, comparando la rama local con el commit
+/// que se leyo. Solo lee: `status --porcelain` de `tareas/` solo se lanza si
+/// la rama local no esta en lo leido.
+pub fn local_de(git: &Git, vista: &Vista<'_>) -> Result<Local, Informe> {
+    let dir = vista.dir();
+    let pendientes = super::pendientes::contar(git, dir)?;
+    let rama = format!("refs/heads/{RAMA}");
+    let tareas = match (crate::git::commit_de(git, dir, OP, &rama)?, vista.commit()) {
+        (Some(local), Some(leido)) if local != leido => {
+            if !crate::git::es_ancestro(git, dir, OP, &local, leido)? {
+                Some("diverge")
+            } else {
+                let montada = crate::git::worktrees(git, dir, OP)?
+                    .into_iter()
+                    .find(|w| w.rama.as_deref() == Some(RAMA) && w.ruta.exists());
+                match montada {
+                    Some(w) => {
+                        let s = crate::git::exigir(
+                            git,
+                            &w.ruta,
+                            OP,
+                            crate::git::Subcomando::Status,
+                            &["--porcelain"],
+                        )?;
+                        (!s.trim().is_empty()).then_some("con-cambios")
+                    }
+                    None => None,
+                }
+            }
+        }
+        _ => None,
+    };
+    Ok(Local { pendientes, tareas })
+}
+
+/// Vista de `referencia`; sin rama de estado → `rechazada`.
+fn vista_de<'a>(git: &'a Git, principal: &Path, referencia: &str) -> Result<Vista<'a>, Informe> {
+    match crate::git::commit_de(git, principal, OP, referencia)? {
+        Some(c) => Vista::nueva(git, principal, Some(c)),
         None => Err(Informe::nuevo(OP, Resultado::Rechazada)
             .con("motivo", "sin-rama-de-estado")
             .diagnostico(
diff --git a/crates/tc2/src/estado/mod.rs b/crates/tc2/src/estado/mod.rs
index 0ac0470..bb2aac7 100644
--- a/crates/tc2/src/estado/mod.rs
+++ b/crates/tc2/src/estado/mod.rs
@@ -13,6 +13,10 @@
 //! - [`leer`] da una vista del estado (lo ultimo traido del remoto, o la
 //!   rama local en un repo sin remoto) y [`tareas`], sus `tarea.md` de los
 //!   dos formatos con su lectura.
+//! - [`leer_fresco`] (F3-5b) trae antes la rama de estado con el mismo
+//!   `fetch` que la transaccion y dice su [`Frescura`]: la hora del ultimo
+//!   `fetch` con exito ([`Marca`]) y por que no se pudo traer, si no se pudo.
+//!   La usan `status`, `board`, `next` y `roadmap`.
 //!
 //! La correccion no depende de `tareas/`: se lee y se escribe sobre commits;
 //! la carpeta montada solo se adelanta (fast-forward) tras publicar.
@@ -21,6 +25,7 @@
 //! codigos de la tabla de `contrato.rs`.
 
 mod cerrojo;
+mod frescura;
 mod lectura;
 mod montaje;
 mod pendientes;
@@ -29,8 +34,9 @@ mod transaccion;
 
 pub use crate::git::Cambio;
 pub use cerrojo::Cerrojo;
+pub use frescura::{Frescura, Marca, Remoto, hace, sincronizado_texto};
 pub(crate) use lectura::referencia_de_lectura;
-pub use lectura::{Estado, Vista, leer};
+pub use lectura::{Estado, Local, Vista, leer, leer_fresco, local_de, sincronizado_en};
 pub use montaje::{Previsto, montar, montar_con, previsto};
 pub use tareas::{Leida, de_otra_persona, es_tarea_md, por_estado, tarea_de, tareas};
 pub use transaccion::{Catalogo, Contexto, Fallo, Operacion, Politica, Roadmap, transaccion};
diff --git a/crates/tc2/src/estado/pendientes.rs b/crates/tc2/src/estado/pendientes.rs
index f35973c..e605702 100644
--- a/crates/tc2/src/estado/pendientes.rs
+++ b/crates/tc2/src/estado/pendientes.rs
@@ -66,6 +66,12 @@ pub(crate) fn anotacion_de(mensaje: &str) -> Option<String> {
         .filter(|a| !a.is_empty())
 }
 
+/// Cuantas pendientes hay (las refs de [`REFS_PENDIENTES`]), sin leer sus
+/// mensajes: lo que dice `status` (F3-5b, F1-2 D5).
+pub(crate) fn contar(git: &Git, dir: &Path) -> Paso<usize> {
+    refs(git, dir, REFS_PENDIENTES).map(|v| v.len())
+}
+
 /// Pendientes en orden de llegada.
 pub(crate) fn listar(git: &Git, dir: &Path) -> Paso<Vec<Pendiente>> {
     let mut v = Vec::new();
diff --git a/crates/tc2/src/estado/transaccion.rs b/crates/tc2/src/estado/transaccion.rs
index dd6eeca..2cd1f44 100644
--- a/crates/tc2/src/estado/transaccion.rs
+++ b/crates/tc2/src/estado/transaccion.rs
@@ -13,6 +13,7 @@ use std::path::Path;
 use std::time::Duration;
 
 use super::cerrojo::{Cerrojo, ErrorCerrojo};
+use super::frescura::{Marca, Traida, traer_estado};
 use super::lectura::{Estado, Vista};
 use super::pendientes::{self, ANOTACION, Destino};
 use super::{CARPETA, OP, RAMA};
@@ -20,8 +21,8 @@ use crate::contrato::{Informe, Resultado, corto};
 use crate::git::estado::{construir_commit, dir_comun, hay_remoto};
 use crate::git::remoto::publicar;
 use crate::git::{
-    Cambio, FalloRemoto, Git, REMOTO, Subcomando, clasificar_fallo_remoto, commit_de, correr,
-    es_ancestro, exigir, localizar_principal, worktrees,
+    Cambio, FalloRemoto, Git, REMOTO, Subcomando, commit_de, correr, es_ancestro, exigir,
+    localizar_principal, worktrees,
 };
 use crate::i18n::Mensaje;
 use crate::i18n::catalogo::estado as txt;
@@ -191,6 +192,7 @@ fn ejecutar(c: &Contexto<'_>, op: &dyn Operacion) -> Result<Informe, Informe> {
     let principal = localizar_principal(c.git, c.dir)?;
     let comun = dir_comun(c.git, &principal, OP)?;
     let remoto = hay_remoto(c.git, &principal, OP)?;
+    let marca = Marca::en(&comun);
     let mut n = 0;
     loop {
         n += 1;
@@ -207,7 +209,7 @@ fn ejecutar(c: &Contexto<'_>, op: &dyn Operacion) -> Result<Informe, Informe> {
                 ));
             }
         };
-        let r = intento(c, op, &anotacion, &principal, remoto, n);
+        let r = intento(c, op, &anotacion, &principal, remoto, &marca, n);
         drop(cerrojo);
         match r? {
             Intento::Hecho(i) => return Ok(i),
@@ -268,37 +270,32 @@ fn intento(
     anotacion: &str,
     principal: &Path,
     remoto: bool,
+    marca: &Marca,
     n: u32,
 ) -> Result<Intento, Informe> {
     let git = c.git;
     let base = if remoto {
-        let refspec = format!("refs/heads/{RAMA}:{}", rama_remota());
-        let s = correr(
-            git,
-            principal,
-            OP,
-            Subcomando::Fetch,
-            &["-q", REMOTO, &refspec],
-        )?;
-        if s.exito() {
-            commit_de(git, principal, OP, &rama_remota())?
-        } else {
-            match clasificar_fallo_remoto(&s.stderr) {
-                // El remoto aun no tiene la rama. Si existe la local (un repo
-                // que se monto sin remoto y al que luego se anadio `origin`),
-                // se parte de ella para publicar su historia, no de cero.
-                FalloRemoto::RamaInexistente => commit_de(git, principal, OP, &rama_local())?,
+        // El mismo `fetch` que la lectura fresca (F3-5b): con exito, renueva
+        // la marca de la ultima sincronizacion.
+        match traer_estado(git, principal, marca, crate::identidad::ahora())? {
+            Traida::Hecha => commit_de(git, principal, OP, &rama_remota())?,
+            // El remoto aun no tiene la rama. Si existe la local (un repo
+            // que se monto sin remoto y al que luego se anadio `origin`),
+            // se parte de ella para publicar su historia, no de cero.
+            Traida::SinRama => commit_de(git, principal, OP, &rama_local())?,
+            Traida::Fallo {
+                fallo,
+                stderr,
+                primera,
+            } => match fallo {
                 FalloRemoto::SinRed => {
-                    return sin_red(c, op, anotacion, principal, &s.primera_linea_error());
+                    return sin_red(c, op, anotacion, principal, &primera);
                 }
                 FalloRemoto::Local => {
-                    return Err(crate::git::remoto::error_local(
-                        OP,
-                        &s.primera_linea_error(),
-                    ));
+                    return Err(crate::git::remoto::error_local(OP, &primera));
                 }
-                otro => return Ok(Intento::Hecho(denegada(op, otro.motivo(), &s.stderr))),
-            }
+                otro => return Ok(Intento::Hecho(denegada(op, otro.motivo(), &stderr))),
+            },
         }
     } else {
         commit_de(git, principal, OP, &rama_local())?
@@ -396,6 +393,11 @@ fn intento(
     let p = publicar(git, principal, OP, &format!("{sha}:refs/heads/{RAMA}"), &[])?;
     match p.resultado {
         Resultado::Ok | Resultado::SinCambios => {
+            // Publicado: lo de `origin` es lo que se acaba de escribir, asi que
+            // el clon esta sincronizado ahora (C5 de TASK-079: tambien el
+            // primer `new` de un remoto sin rama de estado, cuyo `fetch` no
+            // salio). Un fallo al escribir la marca no cambia el resultado.
+            let _ = marca.registrar(crate::identidad::ahora());
             let (publicadas, rechazadas) = pendientes::resolver(git, principal, &destinos)?;
             let t = adelantar(git, principal, &sha)?;
             Ok(Intento::Hecho(ok(op, &sha, n, t, publicadas, rechazadas)))
diff --git a/crates/tc2/src/huella.rs b/crates/tc2/src/huella.rs
index e510fa7..ef98994 100644
--- a/crates/tc2/src/huella.rs
+++ b/crates/tc2/src/huella.rs
@@ -8,8 +8,10 @@
 //! - las refs propias, bajo [`PREFIJO_REFS`];
 //! - la carpeta de los worktrees de tarea, [`CARPETA_WORKTREES`];
 //! - lo que se guarda en la carpeta comun de git (`.git` del principal),
-//!   bajo [`CARPETA_COMUN`]: el cerrojo, la cache de identidad y los indices
-//!   temporales de la transaccion.
+//!   bajo [`CARPETA_COMUN`]: el cerrojo, la cache de identidad, la marca de
+//!   la ultima sincronizacion y los indices temporales de la transaccion.
+//!   `tc2 desinstalar` borra la carpeta entera, asi que un fichero nuevo
+//!   aqui queda reconocido sin tocarlo.
 //!
 //! La configuracion del proyecto (`.taskcode/config.yml`, `personas.yml` y
 //! el vocabulario) no esta aqui: F1-12 D4a la deja compartida con la 0.16.
@@ -53,6 +55,10 @@ pub const FICHERO_CERROJO: &str = "estado.lock";
 /// Cache de la identidad, en [`CARPETA_COMUN`] (F2-13).
 pub const FICHERO_IDENTIDAD: &str = "identidad.yml";
 
+/// Hora del ultimo `fetch` con exito de la rama de estado, en
+/// [`CARPETA_COMUN`] (F3-5b, D3a).
+pub const FICHERO_SINCRONIZACION: &str = "sincronizacion";
+
 /// Indices temporales de la transaccion, en [`CARPETA_COMUN`].
 pub const CARPETA_TEMPORALES: &str = "estado-tmp";
 
@@ -106,6 +112,16 @@ mod tests {
         assert!(TEXTO_LEEME.contains(&format!("(`{RAMA_ESTADO}`)")));
         assert_eq!(RUTA_WORKFLOW, format!(".github/workflows/{NOMBRE}.yml"));
         assert_eq!(PREFIJO_RULESETS, format!("{NOMBRE}-"));
+        // Los ficheros de la carpeta comun no se pisan entre si.
+        let ficheros = [
+            FICHERO_CERROJO,
+            FICHERO_IDENTIDAD,
+            FICHERO_SINCRONIZACION,
+            CARPETA_TEMPORALES,
+        ];
+        for (i, a) in ficheros.iter().enumerate() {
+            assert!(ficheros[i + 1..].iter().all(|b| a != b), "{a} repetido");
+        }
     }
 
     /// Las lineas de `exclude` y `.gitignore` nombran las carpetas reales.
diff --git a/crates/tc2/src/i18n/catalogo/estado.rs b/crates/tc2/src/i18n/catalogo/estado.rs
index 3a010ec..d6b9b8d 100644
--- a/crates/tc2/src/i18n/catalogo/estado.rs
+++ b/crates/tc2/src/i18n/catalogo/estado.rs
@@ -84,6 +84,52 @@ crate::i18n::textos! {
         en: "could not fast-forward {rama}: {error}",
     }
 
+    // Frescura de la lectura (`frescura.rs`, F3-5b).
+    AVISO_NO_FRESCO {
+        es: "aviso: {porque}; se muestra lo que este clon trajo el {hora} ({hace})",
+        en: "warning: {porque}; showing what this clone fetched at {hora} ({hace})",
+    }
+    AVISO_NUNCA_SINCRONIZADO {
+        es: "aviso: {porque} y este clon no la ha sincronizado nunca; lo que se muestra puede no estar al dia",
+        en: "warning: {porque} and this clone has never synchronised it; what is shown may be out of date",
+    }
+    PORQUE_SIN_RED {
+        es: "sin red, no se pudo traer la rama de estado de origin",
+        en: "no network, the state branch could not be fetched from origin",
+    }
+    PORQUE_DENEGADO {
+        es: "origin no deja leer la rama de estado ({motivo})",
+        en: "origin does not allow reading the state branch ({motivo})",
+    }
+    PORQUE_ERROR_LOCAL {
+        es: "no se pudo traer la rama de estado por un fallo de este clon",
+        en: "the state branch could not be fetched because of a failure in this clone",
+    }
+    PORQUE_SIN_RAMA {
+        es: "origin no tiene la rama de estado",
+        en: "origin does not have the state branch",
+    }
+    SINCRONIZADO {
+        es: "estado sincronizado con origin el {hora} ({hace})",
+        en: "state synchronised with origin at {hora} ({hace})",
+    }
+    HACE_MENOS_DE_UN_MINUTO {
+        es: "hace menos de 1 min",
+        en: "less than 1 min ago",
+    }
+    HACE_MINUTOS {
+        es: "hace {n} min",
+        en: "{n} min ago",
+    }
+    HACE_HORAS {
+        es: "hace {n} h",
+        en: "{n} h ago",
+    }
+    HACE_DIAS {
+        es: "hace {n} dias",
+        en: "{n} days ago",
+    }
+
     // Alta (`alta.rs`).
     SIN_IDS {
         es: "no quedan IDs de tarea",
diff --git a/crates/tc2/src/i18n/catalogo/tablero.rs b/crates/tc2/src/i18n/catalogo/tablero.rs
index dc8ab96..330aaf7 100644
--- a/crates/tc2/src/i18n/catalogo/tablero.rs
+++ b/crates/tc2/src/i18n/catalogo/tablero.rs
@@ -29,6 +29,14 @@ crate::i18n::textos! {
         es: "aviso: en revision sin PR abierto listo: {ids}",
         en: "warning: in review without an open ready pull request: {ids}",
     }
+    AVISO_TAREAS_CON_CAMBIOS {
+        es: "aviso: {carpeta}/ tiene cambios sin publicar y no esta en lo publicado; la siguiente escritura no podra adelantarla: publicalos o deshazlos",
+        en: "warning: {carpeta}/ has unpublished changes and is behind what was published; the next write will not be able to fast-forward it: publish or undo them",
+    }
+    AVISO_TAREAS_DIVERGE {
+        es: "aviso: la rama local {rama} tiene commits que no estan en lo publicado; {carpeta}/ no se adelanta: revisalos",
+        en: "warning: the local branch {rama} has commits that are not in what was published; {carpeta}/ is not fast-forwarded: review them",
+    }
     AVISO_PR_SIN_REVISION {
         es: "aviso: con PR abierto listo y sin pasar a revision: {ids}",
         en: "warning: with an open ready pull request but not in review: {ids}",
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 5d9a746..130a393 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -120,7 +120,7 @@ where
         }
         [orden, resto @ ..] if orden == siguiente::NOMBRE => {
             let (json, resto) = con_json(resto);
-            siguiente::orden(&resto).salida_en(json, idioma)
+            siguiente::orden(&resto, json, idioma)
         }
         [orden, resto @ ..] if orden == roadmap::NOMBRE => {
             let (json, resto) = con_json(resto);
diff --git a/crates/tc2/src/roadmap/mod.rs b/crates/tc2/src/roadmap/mod.rs
index ce33413..aa6cb31 100644
--- a/crates/tc2/src/roadmap/mod.rs
+++ b/crates/tc2/src/roadmap/mod.rs
@@ -8,10 +8,14 @@
 //!   cada transaccion de estado ([`crate::estado::Contexto::nuevo`]): se
 //!   regenera, nunca se mergea (ADR F1-2, D2.4).
 //! - La orden `roadmap [--comparar | --escribir] [--json]` (D2): sin
-//!   opciones imprime el ROADMAP que toca sobre el estado que conoce el clon
-//!   (solo lee); `--comparar` dice si lo publicado esta al dia (`ok`) o no
-//!   (`rechazada motivo=desfasado|roadmap-roto`), sin escribir; `--escribir`
-//!   lo publica como una transaccion mas ([`Regenerar`]).
+//!   opciones imprime el ROADMAP que toca sobre el estado recien traido
+//!   (F3-5b: trae la rama de estado y solo escribe
+//!   `origin/taskcodetwo/tareas` y la marca de sincronizacion); `--comparar`
+//!   dice si lo publicado esta al dia (`ok`) o no (`rechazada
+//!   motivo=desfasado|roadmap-roto`), sin escribir; `--escribir` lo publica
+//!   como una transaccion mas ([`Regenerar`]). La hora de la ultima
+//!   sincronizacion va en `remoto=` y `sincronizado=` y, en el texto, a
+//!   stderr: el stdout sigue siendo el ROADMAP byte a byte.
 //! - La zona manual se edita en `tareas/ROADMAP.md` y se publica con
 //!   `--escribir` (D4): si la zona de la carpeta montada cambio, se valida, se
 //!   prepara (`git add`) y es la que se publica, byte a byte, salvo que la
@@ -28,7 +32,7 @@ use std::path::{Path, PathBuf};
 
 use crate::Salida;
 use crate::contrato::{Informe, Resultado, Valor, corto};
-use crate::estado::{self, CARPETA, Cambio, Catalogo, Estado, Fallo, Operacion, RAMA};
+use crate::estado::{self, CARPETA, Cambio, Catalogo, Estado, Fallo, Frescura, Operacion, RAMA};
 use crate::git::{Git, Subcomando, correr, exigir, localizar_principal, worktrees};
 use crate::i18n::catalogo::cli as txt_cli;
 use crate::i18n::catalogo::roadmap as txt;
@@ -247,19 +251,35 @@ pub fn orden(args: &[String], json: bool, idioma: Idioma, catalogo: Catalogo<'_>
         }
     };
     let g = Git::nuevo();
+    let ahora = crate::identidad::ahora();
+    // La frescura de las dos que leen (F3-5b): en el texto de `roadmap`, la
+    // hora va a stderr (el stdout es el ROADMAP byte a byte, C9).
+    let mut frescura = None;
     let i = match modo {
-        Modo::Imprimir => match imprimir_en(&g, &dir) {
-            Ok((texto, i)) if !json => {
+        Modo::Imprimir => match imprimir(&g, &dir, ahora) {
+            Ok((texto, i, f)) if !json => {
+                let mut stderr = texto_de_avisos(&i, idioma);
+                if let Some(m) = f.linea(ahora) {
+                    stderr.push_str(&m.texto(idioma));
+                    stderr.push('\n');
+                }
                 return Salida {
                     codigo: 0,
                     stdout: texto,
-                    stderr: texto_de_avisos(&i, idioma),
+                    stderr,
                 };
             }
-            Ok((texto, i)) => i.con("texto", texto),
+            Ok((texto, i, f)) => {
+                frescura = Some(f);
+                i.con("texto", texto)
+            }
             Err(i) => i,
         },
-        Modo::Comparar => comparar_en(&g, &dir),
+        Modo::Comparar => {
+            let (i, f) = comparar(&g, &dir, ahora);
+            frescura = f;
+            i
+        }
         Modo::Escribir => escribir_en(&g, &dir, catalogo),
     };
     let mut s = i.salida_en(json, idioma);
@@ -267,6 +287,10 @@ pub fn orden(args: &[String], json: bool, idioma: Idioma, catalogo: Catalogo<'_>
     // si falla, stderr es el diagnostico.
     if s.codigo == 0 {
         s.stderr.push_str(&texto_de_avisos(&i, idioma));
+        if let Some(m) = frescura.and_then(|f| f.aviso(ahora)) {
+            s.stderr.push_str(&m.texto(idioma));
+            s.stderr.push('\n');
+        }
     }
     s
 }
@@ -288,12 +312,12 @@ fn de_fallo(f: Fallo) -> Informe {
     }
 }
 
-/// La situacion del estado que conoce el clon (sin `fetch`) y su commit.
-fn situacion_en(g: &Git, dir: &Path) -> Result<(Situacion, String), Informe> {
-    let vista = estado::leer(g, dir).map_err(renombrado)?;
+/// La situacion del estado recien traido (F3-5b), su commit y su frescura.
+fn situacion_en(g: &Git, dir: &Path, ahora: u64) -> Result<(Situacion, String, Frescura), Informe> {
+    let (vista, frescura) = estado::leer_fresco(g, dir, ahora).map_err(renombrado)?;
     let commit = vista.commit().map(corto).unwrap_or_default();
     let s = situacion::calcular(&vista).map_err(de_fallo)?;
-    Ok((s, commit))
+    Ok((s, commit, frescura))
 }
 
 /// Personas de la base y los avisos que tocan.
@@ -309,20 +333,26 @@ fn personas_con_avisos(g: &Git, dir: &Path) -> (Personas, Vec<&'static str>) {
     )
 }
 
-/// El ROADMAP que toca sobre el estado que conoce el clon, con el informe
-/// `ok` (su `commit=` y sus `avisos=`). Con la zona rota, `rechazada
-/// motivo=roadmap-roto`.
+/// El ROADMAP que toca sobre el estado recien traido, con el informe `ok`
+/// (su `commit=`, sus `avisos=`, `remoto=` y `sincronizado=`). Con la zona
+/// rota, `rechazada motivo=roadmap-roto`.
 pub fn imprimir_en(g: &Git, dir: &Path) -> Result<(String, Informe), Informe> {
-    let (s, commit) = situacion_en(g, dir)?;
+    imprimir(g, dir, crate::identidad::ahora()).map(|(t, i, _)| (t, i))
+}
+
+/// [`imprimir_en`] con su frescura. El texto es solo el ROADMAP: la hora
+/// nunca entra en el (si no, `--comparar` saldria siempre desfasado).
+fn imprimir(g: &Git, dir: &Path, ahora: u64) -> Result<(String, Informe, Frescura), Informe> {
+    let (s, commit, frescura) = situacion_en(g, dir, ahora)?;
     let (personas, avisos) = personas_con_avisos(g, dir);
     match de_situacion(&s, &personas) {
-        Some(t) => Ok((
-            t,
-            con_avisos(
+        Some(t) => {
+            let i = con_avisos(
                 Informe::nuevo(NOMBRE, Resultado::Ok).con("commit", commit),
                 &avisos,
-            ),
-        )),
+            );
+            Ok((t, frescura.anotar(i), frescura))
+        }
         None => Err(zona_rota(&s, &commit)),
     }
 }
@@ -336,14 +366,20 @@ fn zona_rota(s: &Situacion, commit: &str) -> Informe {
 
 /// `--comparar`: `ok` si el `ROADMAP.md` publicado es el que toca;
 /// `rechazada motivo=desfasado` si no; `rechazada motivo=roadmap-roto` si
-/// su zona esta rota (D3). No escribe nada.
+/// su zona esta rota (D3). Trae antes la rama de estado (F3-5b) y no escribe
+/// nada mas.
 pub fn comparar_en(g: &Git, dir: &Path) -> Informe {
-    let (s, commit) = match situacion_en(g, dir) {
+    comparar(g, dir, crate::identidad::ahora()).0
+}
+
+/// [`comparar_en`] con su frescura (`None` si no se llego a leer).
+fn comparar(g: &Git, dir: &Path, ahora: u64) -> (Informe, Option<Frescura>) {
+    let (s, commit, frescura) = match situacion_en(g, dir, ahora) {
         Ok(x) => x,
-        Err(i) => return i,
+        Err(i) => return (i, None),
     };
     let (personas, avisos) = personas_con_avisos(g, dir);
-    match de_situacion(&s, &personas) {
+    let i = match de_situacion(&s, &personas) {
         None => zona_rota(&s, &commit),
         Some(t) if s.roadmap.as_deref() == Some(t.as_str()) => con_avisos(
             Informe::nuevo(NOMBRE, Resultado::Ok).con("commit", commit),
@@ -353,7 +389,8 @@ pub fn comparar_en(g: &Git, dir: &Path) -> Informe {
             .con("motivo", "desfasado")
             .con("commit", commit)
             .diagnostico(&txt::DESFASADO),
-    }
+    };
+    (frescura.anotar(i), Some(frescura))
 }
 
 /// `tareas/` montada (la carpeta del worktree de la rama de estado).
diff --git a/crates/tc2/src/siguiente.rs b/crates/tc2/src/siguiente.rs
index f37e3c8..7922c14 100644
--- a/crates/tc2/src/siguiente.rs
+++ b/crates/tc2/src/siguiente.rs
@@ -1,14 +1,16 @@
 //! `tc2 next [--json]` (F3-3): lo listo para empezar por dependencias y
 //! prioridad.
 //!
-//! Lee el estado tal como lo conoce este clon ([`estado::leer`]: sin
-//! `fetch` y sin escribir; la lectura fresca es de F3-5b) y calcula con
-//! [`crate::situacion`], lo mismo que `roadmap` (TASK-031 C4). El orden de
-//! los hitos sale de los titulos `### Hn — titulo` de la zona manual del
-//! `ROADMAP.md` de la rama de estado (D2 de TASK-030).
+//! Trae antes la rama de estado ([`estado::leer_fresco`], F3-5b: solo
+//! escribe `origin/taskcodetwo/tareas` y la marca de sincronizacion) y
+//! calcula con [`crate::situacion`], lo mismo que `roadmap` (TASK-031 C4).
+//! El orden de los hitos sale de los titulos `### Hn — titulo` de la zona
+//! manual del `ROADMAP.md` de la rama de estado (D2 de TASK-030).
 //!
-//! Salida: `next ok listas=... en-marcha=... commit=<sha> [avisos]`, con
-//! `commit=` la foto del estado usada. Avisos, solo si los hay:
+//! Salida: `next ok listas=... en-marcha=... commit=<sha> [avisos]
+//! remoto=... sincronizado=...`, con `commit=` la foto del estado usada; sin
+//! red, lo ultimo traido con un aviso en stderr y codigo 0. Avisos, solo si
+//! los hay:
 //! `ilegibles=` (`tarea.md` que no se entienden: no se saltan en silencio),
 //! `ciclos=` (las atascadas), `dependencias-inexistentes=`,
 //! `depende-de-descartada=` (`A>B`), `hitos-sin-titulo=`, `duplicadas=`
@@ -17,43 +19,62 @@
 
 use std::path::Path;
 
+use crate::Salida;
 use crate::contrato::{Informe, Resultado, corto};
 use crate::estado::{self, Estado, Fallo};
 use crate::git::Git;
-use crate::i18n::Mensaje;
 use crate::i18n::catalogo::cli as txt_cli;
 use crate::i18n::catalogo::siguiente as txt;
+use crate::i18n::{Idioma, Mensaje};
 use crate::situacion::{self, Situacion};
 
 /// Nombre de la orden.
 pub const NOMBRE: &str = "next";
 
 /// `next [--json]` en la carpeta actual (sin `--json`, que quita `lib.rs`).
-pub fn orden(args: &[String]) -> Informe {
+/// Si no se pudo traer la rama de estado, el aviso va a stderr con codigo 0.
+pub fn orden(args: &[String], json: bool, idioma: Idioma) -> Salida {
     if !args.is_empty() {
         return Informe::nuevo(NOMBRE, Resultado::Uso)
             .con("motivo", "argumentos-de-mas")
-            .diagnostico(&txt::USO_NEXT);
+            .diagnostico(&txt::USO_NEXT)
+            .salida_en(json, idioma);
     }
     match std::env::current_dir() {
-        Ok(dir) => siguiente_en(&Git::nuevo(), &dir),
+        Ok(dir) => {
+            let (i, aviso) = siguiente(&Git::nuevo(), &dir, crate::identidad::ahora());
+            let mut s = i.salida_en(json, idioma);
+            if let (0, Some(a)) = (s.codigo, aviso) {
+                s.stderr.push_str(&a.texto(idioma));
+                s.stderr.push('\n');
+            }
+            s
+        }
         Err(e) => Informe::nuevo(NOMBRE, Resultado::Error)
-            .diagnostico(Mensaje::nuevo(&txt_cli::CARPETA_ACTUAL).con("error", e.to_string())),
+            .diagnostico(Mensaje::nuevo(&txt_cli::CARPETA_ACTUAL).con("error", e.to_string()))
+            .salida_en(json, idioma),
     }
 }
 
-/// `next` sobre el estado que conoce el clon de `dir`.
+/// `next` sobre el estado recien traido al clon de `dir` (F3-5b).
 pub fn siguiente_en(g: &Git, dir: &Path) -> Informe {
-    let vista = match estado::leer(g, dir) {
+    siguiente(g, dir, crate::identidad::ahora()).0
+}
+
+/// `next` tras traer la rama de estado: el informe (con `remoto=` y
+/// `sincronizado=`) y el aviso de frescura, si no se pudo traer.
+fn siguiente(g: &Git, dir: &Path, ahora: u64) -> (Informe, Option<Mensaje>) {
+    let (vista, frescura) = match estado::leer_fresco(g, dir, ahora) {
         Ok(v) => v,
         Err(mut i) => {
             NOMBRE.clone_into(&mut i.comando);
-            return i;
+            return (i, None);
         }
     };
+    let aviso = frescura.aviso(ahora);
     let commit = vista.commit().map(corto).unwrap_or_default();
-    match calcular(&vista) {
-        Ok(c) => c.informe(&commit),
+    let i = match calcular(&vista) {
+        Ok(c) => frescura.anotar(c.informe(&commit)),
         Err(Fallo::Rechazada {
             motivo,
             diagnostico,
@@ -61,7 +82,8 @@ pub fn siguiente_en(g: &Git, dir: &Path) -> Informe {
             .con("motivo", motivo)
             .diagnostico(diagnostico),
         Err(Fallo::Error(d)) => Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d),
-    }
+    };
+    (i, aviso)
 }
 
 /// Lo que se calcula sobre el estado: la [`Situacion`] comun con `roadmap`.
@@ -197,7 +219,10 @@ mod tests {
 
     #[test]
     fn argumentos_de_mas_es_uso() {
-        let i = orden(&["x".to_owned()]);
-        assert_eq!(i.linea(), "next uso motivo=argumentos-de-mas");
+        let s = orden(&["x".to_owned()], false, Idioma::Es);
+        assert_eq!(
+            (s.codigo, s.stdout.as_str()),
+            (2, "next uso motivo=argumentos-de-mas\n")
+        );
     }
 }
diff --git a/crates/tc2/src/tablero.rs b/crates/tc2/src/tablero.rs
index 1692e22..de9baba 100644
--- a/crates/tc2/src/tablero.rs
+++ b/crates/tc2/src/tablero.rs
@@ -2,11 +2,15 @@
 //! y que espera revision.
 //!
 //! Las dos ordenes solo leen. Salen de la misma [`situacion::calcular`] sobre
-//! [`estado::leer`] que `next` y `roadmap` (misma foto y mismo `commit=`), sin
-//! `fetch` (la frescura es de F3-5b) y sin escribir nada.
+//! [`estado::leer_fresco`] que `next` y `roadmap` (misma foto y mismo
+//! `commit=`): traen antes la rama de estado (F3-5b) y dicen la hora de la
+//! ultima sincronizacion (`remoto=` y `sincronizado=`; sin red, lo ultimo
+//! traido con un aviso en stderr y codigo 0). No escriben nada mas que
+//! `origin/taskcodetwo/tareas` y la marca de sincronizacion.
 //!
-//! - `status [--json]`: una linea de equipo, sin red. Con `--json`, `status
-//!   ok en-diseno=... en-curso=... en-revision=... listas=... commit=...`.
+//! - `status [--json]`: una linea de equipo; no consulta la plataforma. Con
+//!   `--json`, `status ok en-diseno=... en-curso=... en-revision=... listas=...
+//!   commit=... remoto=... sincronizado=... pendientes=N [tareas=...]`.
 //! - `board [--sin-plataforma] [--json]`: la vista completa por grupos (en
 //!   diseno, en curso, en revision y listas para empezar), con ID, titulo,
 //!   quien y PR; terminadas y descartadas solo como recuento.
@@ -21,7 +25,7 @@ use std::path::Path;
 
 use crate::Salida;
 use crate::contrato::{Informe, Resultado, corto};
-use crate::estado::{self, Fallo};
+use crate::estado::{self, Fallo, Frescura, Local};
 use crate::git::Git;
 use crate::grafo::{Fase, TERMINADA};
 use crate::i18n::catalogo::cli as txt_cli;
@@ -75,6 +79,9 @@ pub struct Tablero {
     pub descartadas: usize,
     /// Carpetas con un `tarea.md` que no se entiende.
     pub ilegibles: Vec<String>,
+    /// Hora de la ultima sincronizacion para el texto (`None`: no se
+    /// ensena, sin `origin`).
+    pub sincronizado: Option<String>,
 }
 
 impl Tablero {
@@ -195,6 +202,9 @@ pub fn linea_status(t: &Tablero) -> String {
         partes.push("nada en marcha ni listo para empezar".to_owned());
     }
     partes.push(format!("commit {}", t.commit));
+    if let Some(s) = &t.sincronizado {
+        partes.push(format!("sincronizado {s}"));
+    }
     partes.join(" · ")
 }
 
@@ -202,7 +212,11 @@ pub fn linea_status(t: &Tablero) -> String {
 /// el recuento de terminadas y descartadas. `pr` da la celda del PR de cada
 /// fila (`None`: sin columna).
 pub fn texto_board(t: &Tablero, plataforma: &str, pr: Option<&dyn Fn(&Fila) -> String>) -> String {
-    let mut o = format!("Tablero · commit {} · plataforma {plataforma}\n", t.commit);
+    let mut o = format!("Tablero · commit {} · plataforma {plataforma}", t.commit);
+    if let Some(s) = &t.sincronizado {
+        let _ = write!(o, " · sincronizado {s}");
+    }
+    o.push('\n');
     for (_, titulo, filas) in t.grupos() {
         let _ = writeln!(o, "\n{titulo} ({})", filas.len());
         if filas.is_empty() {
@@ -262,17 +276,37 @@ fn rellenar(t: &str, ancho: usize) -> String {
 /// Aviso: la lista de personas existe y no se pudo leer.
 const PERSONAS_ILEGIBLES: &str = "personas-ilegibles";
 
-/// El tablero del estado que conoce el clon de `dir`, con los avisos de
-/// personas. Sin rama de estado, `rechazada motivo=sin-rama-de-estado`.
-fn tablero_en(
-    g: &Git,
-    dir: &Path,
-    op: &str,
-) -> Result<(Tablero, Vec<&'static str>, Personas), Informe> {
-    let vista = estado::leer(g, dir).map_err(|mut i| {
+/// Lo leido para `status` y `board`.
+struct Leido {
+    /// El tablero, con la hora para el texto.
+    tablero: Tablero,
+    /// Avisos de personas.
+    avisos: Vec<&'static str>,
+    /// La lista de personas de la base.
+    personas: Personas,
+    /// La frescura de la lectura.
+    frescura: Frescura,
+    /// Lo del propio clon, si se pidio (solo `status`).
+    local: Option<Local>,
+    /// Cuando se leyo, para el «hace N».
+    ahora: u64,
+}
+
+/// El tablero del estado que conoce el clon de `dir` tras traer la rama de
+/// estado (F3-5b), con los avisos de personas. Sin rama de estado,
+/// `rechazada motivo=sin-rama-de-estado`.
+fn tablero_en(g: &Git, dir: &Path, op: &str, con_local: bool) -> Result<Leido, Informe> {
+    let renombrar = |mut i: Informe| {
         op.clone_into(&mut i.comando);
         i
-    })?;
+    };
+    let ahora = crate::identidad::ahora();
+    let (vista, frescura) = estado::leer_fresco(g, dir, ahora).map_err(renombrar)?;
+    let local = if con_local {
+        Some(estado::local_de(g, &vista).map_err(renombrar)?)
+    } else {
+        None
+    };
     let commit = vista.commit().map(corto).unwrap_or_default();
     let s = situacion::calcular(&vista).map_err(|f| match f {
         Fallo::Rechazada {
@@ -289,7 +323,16 @@ fn tablero_en(
     } else {
         Vec::new()
     };
-    Ok((tablero(&s, &personas, &commit), avisos, personas))
+    let mut t = tablero(&s, &personas, &commit);
+    t.sincronizado = frescura.en_texto();
+    Ok(Leido {
+        tablero: t,
+        avisos,
+        personas,
+        frescura,
+        local,
+        ahora,
+    })
 }
 
 // La conexion con la plataforma vive en `identidad` desde F3-8 (la usan
@@ -358,19 +401,36 @@ pub fn orden_status(args: &[String], json: bool, idioma: Idioma, e: &Entorno<'_>
             .salida_en(json, idioma);
     }
     match status_en(e) {
-        Ok((linea, _)) if !json => Salida {
-            codigo: 0,
-            stdout: format!("{linea}\n"),
-            stderr: String::new(),
-        },
-        Ok((_, i)) | Err(i) => i.salida_en(json, idioma),
+        Ok((linea, i, avisos)) => {
+            let mut s = if json {
+                i.salida_en(true, idioma)
+            } else {
+                Salida {
+                    codigo: 0,
+                    stdout: format!("{linea}\n"),
+                    stderr: String::new(),
+                }
+            };
+            s.stderr.push_str(&lineas(&avisos, idioma));
+            s
+        }
+        Err(i) => i.salida_en(json, idioma),
     }
 }
 
-/// `status` sobre el estado que conoce el clon: la linea de texto y el
-/// informe.
-pub fn status_en(e: &Entorno<'_>) -> Result<(String, Informe), Informe> {
-    let (t, _, _) = tablero_en(e.git, e.dir, STATUS)?;
+/// Mensajes en lineas para stderr.
+pub fn lineas(avisos: &[Mensaje], idioma: Idioma) -> String {
+    avisos
+        .iter()
+        .map(|m| format!("{}\n", m.texto(idioma)))
+        .collect()
+}
+
+/// `status` sobre el estado recien traido: la linea de texto, el informe y
+/// los avisos para stderr (la frescura si no se pudo traer y `tareas/`).
+pub fn status_en(e: &Entorno<'_>) -> Result<(String, Informe, Vec<Mensaje>), Informe> {
+    let l = tablero_en(e.git, e.dir, STATUS, true)?;
+    let t = &l.tablero;
     let mut i = Informe::nuevo(STATUS, Resultado::Ok);
     for (clave, _, filas) in t.grupos() {
         i = i.con_lista(clave, ids(filas));
@@ -379,7 +439,21 @@ pub fn status_en(e: &Entorno<'_>) -> Result<(String, Informe), Informe> {
     if !t.ilegibles.is_empty() {
         i = i.con_lista("ilegibles", t.ilegibles.clone());
     }
-    Ok((linea_status(&t), i))
+    i = l.frescura.anotar(i);
+    let mut avisos: Vec<Mensaje> = l.frescura.aviso(l.ahora).into_iter().collect();
+    if let Some(local) = l.local {
+        i = i.con("pendientes", local.pendientes.to_string());
+        if let Some(tareas) = local.tareas {
+            i = i.con("tareas", tareas);
+            let m = if tareas == "diverge" {
+                Mensaje::nuevo(&txt::AVISO_TAREAS_DIVERGE).con("rama", estado::RAMA)
+            } else {
+                Mensaje::nuevo(&txt::AVISO_TAREAS_CON_CAMBIOS)
+            };
+            avisos.push(m.con("carpeta", estado::CARPETA));
+        }
+    }
+    Ok((linea_status(t), i, avisos))
 }
 
 /// Que hace `board` con la plataforma.
@@ -405,8 +479,9 @@ pub fn orden_board(args: &[String], json: bool, idioma: Idioma, e: &Entorno<'_>)
         }
     };
     match board_en(e, modo) {
-        Ok((texto, i)) => {
-            let avisos = texto_de_avisos(&i, idioma);
+        Ok((texto, i, de_frescura)) => {
+            let mut avisos = texto_de_avisos(&i, idioma);
+            avisos.push_str(&lineas(&de_frescura, idioma));
             let mut s = if json {
                 i.con("texto", texto).salida_en(true, idioma)
             } else {
@@ -423,14 +498,15 @@ pub fn orden_board(args: &[String], json: bool, idioma: Idioma, e: &Entorno<'_>)
     }
 }
 
-/// `board` sobre el estado que conoce el clon: el texto y el informe (sin
-/// `texto=`).
-pub fn board_en(e: &Entorno<'_>, modo: Modo) -> Result<(String, Informe), Informe> {
+/// `board` sobre el estado recien traido: el texto, el informe (sin
+/// `texto=`, con `remoto=` y `sincronizado=`) y el aviso de frescura para
+/// stderr, si lo hay.
+pub fn board_en(e: &Entorno<'_>, modo: Modo) -> Result<(String, Informe, Vec<Mensaje>), Informe> {
     // El estado primero: si no hay rama de estado, no se habla con nadie.
-    let (t, avisos, personas) = tablero_en(e.git, e.dir, BOARD)?;
+    let l = tablero_en(e.git, e.dir, BOARD, false)?;
     let consulta = match modo {
         Modo::SinPlataforma => Consulta::NoConsultada,
-        Modo::ConPlataforma => match (e.conectar)(&personas) {
+        Modo::ConPlataforma => match (e.conectar)(&l.personas) {
             Ok(c) => match c.plataforma.prs_abiertos(c.rol) {
                 Ok(prs) => Consulta::Hecha(prs),
                 Err(i) => Consulta::de_fallo(&i),
@@ -438,7 +514,12 @@ pub fn board_en(e: &Entorno<'_>, modo: Modo) -> Result<(String, Informe), Inform
             Err(c) => c,
         },
     };
-    Ok(board_de(&t, &consulta, &avisos))
+    let (texto, i) = board_de(&l.tablero, &consulta, &l.avisos);
+    Ok((
+        texto,
+        l.frescura.anotar(i),
+        l.frescura.aviso(l.ahora).into_iter().collect(),
+    ))
 }
 
 /// Texto e informe de `board` para un tablero y lo que se supo de la
@@ -794,9 +875,32 @@ mod tests {
             terminadas: 4,
             descartadas: 1,
             ilegibles: Vec::new(),
+            sincronizado: None,
         }
     }
 
+    /// C3 (parte pura): la hora va al final de la linea de `status` y de la
+    /// cabecera de `board`; sin hora (sin `origin`), no sale.
+    #[test]
+    fn la_hora_en_la_linea_y_la_cabecera() {
+        let t = Tablero {
+            sincronizado: Some("2026-11-17T16:04:12Z".to_owned()),
+            ..de_prueba()
+        };
+        assert!(
+            linea_status(&t).ends_with(&format!(
+                "commit {COMMIT} · sincronizado 2026-11-17T16:04:12Z"
+            )),
+            "{}",
+            linea_status(&t)
+        );
+        assert!(texto_board(&t, "ok", None).starts_with(
+            "Tablero · commit 0123456789ab · plataforma ok · sincronizado 2026-11-17T16:04:12Z\n"
+        ));
+        assert!(!linea_status(&de_prueba()).contains("sincronizado"));
+        assert!(!texto_board(&de_prueba(), "ok", None).contains("sincronizado"));
+    }
+
     /// C3b: `board` lista los cuatro grupos con ID, titulo, quien y PR, y
     /// avisa `revision-sin-pr=` (sin PR o solo en borrador) y
     /// `pr-sin-revision=` (PR listo de una tarea que no esta en revision).
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-079/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-079/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-079/tarea.md                                    | 29 +++++++++++++++++++++++++++--
 3 files changed, 27 insertions(+), 2 deletions(-)
````

## Tests del diff (4 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/desinstalar.rs" ":(literal)crates/tc2/tests/frescura.rs" ":(literal)crates/tc2/tests/siguiente.rs" ":(literal)crates/tc2/tests/tablero.rs"
````

````
crates/tc2/tests/desinstalar.rs |   7 ++
 crates/tc2/tests/frescura.rs    | 625 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/siguiente.rs   |  73 ++++++++++++++-----
 crates/tc2/tests/tablero.rs     |  73 +++++++++++++++----
 4 files changed, 749 insertions(+), 29 deletions(-)
````
