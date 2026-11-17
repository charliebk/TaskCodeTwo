# Peticion de revision — TASK-034 (ronda 1)

- Tarea: TASK-034 — F3-7 descartar y pausa
- Rama revisada: feature/task-034-f3-7-descartar-y-pausa
- Rama base: develop
- Commit revisado (HEAD): 5dc73db88b26d5303bba1983e29f5d2de4803f22
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-034 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-034/tarea.md
- tareas/03-en-revision/TASK-034/planificacion/
- crates/tc2/src/ciclo/mod.rs
- crates/tc2/src/ciclo/operacion.rs
- crates/tc2/src/estado/mod.rs
- crates/tc2/src/estado/tareas.rs
- crates/tc2/src/i18n/catalogo/ciclo.rs
- crates/tc2/src/i18n/catalogo/cli.rs
- crates/tc2/src/i18n/catalogo/comun.rs
- crates/tc2/src/i18n/catalogo/mod.rs
- crates/tc2/src/i18n/catalogo/toma.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/roadmap/generar.rs
- crates/tc2/src/roadmap/mod.rs
- crates/tc2/src/situacion.rs
- crates/tc2/src/tarea/cuerpo.rs
- crates/tc2/src/tarea/mod.rs
- crates/tc2/src/toma/mod.rs
- crates/tc2/src/toma/operacion.rs
- crates/tc2/tests/ciclo.rs

## Commits a revisar (git log develop..HEAD)

````
5dc73db docs(TASK-034): autorrevision con rust-wasm-reviewer y code-quality-reviewer
e4c2c67 test(TASK-034): E2E de descartar, pausa y reanudar y adendas de F1-2 y F1-4
90dc2e1 feat(TASK-034): ordenes descartar, pausa y reanudar en el CLI
5ebae52 feat(TASK-034): operaciones Descartar y Pausar sobre la rama de estado
aee7594 feat(TASK-034): pausa calculada de las transiciones y motivo de las descartadas
a630e8e refactor(TASK-034): lectura comun tarea_de para las operaciones de una tarea
c451c7a chore(TASK-034): tarea en curso
````

## Diff de tu dominio (18 fichero(s) de 21; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/ciclo/mod.rs b/crates/tc2/src/ciclo/mod.rs
new file mode 100644
index 0000000..094fcca
--- /dev/null
+++ b/crates/tc2/src/ciclo/mod.rs
@@ -0,0 +1,192 @@
+//! `tc2 descartar`, `tc2 pausa` y `tc2 reanudar` (F3-7, RF-07 y RF-08).
+//!
+//! Las operaciones de la rama de estado estan en [`operacion`]; aqui, las
+//! ordenes del CLI:
+//!
+//! - `descartar [--json] TASK-NNN "<motivo>"`: `fetch` (exige red: sin ella,
+//!   `sinred pendiente=no` sin escribir nada), quien la lanza (identidad), el
+//!   descarte por el punto unico de escritura ([`estado::transaccion`], con
+//!   el `ROADMAP.md` regenerado en el mismo commit) y, publicado (o ya
+//!   descartada), el worktree de la tarea con [`git::quitar_worktree`] (D3):
+//!   se quita si esta limpio y publicado; si no, se conserva y la linea lo
+//!   dice (`worktree=conservado worktree-motivo=...`). La rama no se toca
+//!   nunca. Lo unico posterior a publicar es quitar el worktree, que nunca
+//!   hace fallar la orden: no hay nada que comprobar antes que pueda dejar
+//!   el descarte publicado y la orden rechazada.
+//! - `pausa [--json] TASK-NNN` y `reanudar [--json] TASK-NNN`: quien la
+//!   lanza y la fila por la transaccion; sin red quedan pendientes y la
+//!   siguiente transaccion con red las publica (D4, ADR F1-2 punto 5).
+//!
+//! Lanzadas desde un worktree de tarea escriben el estado en el principal
+//! (lo localiza la transaccion). El entorno es el de `start`
+//! ([`crate::toma::Entorno`]).
+
+pub mod operacion;
+
+use std::path::Path;
+
+use crate::contrato::{Informe, Resultado, Valor};
+use crate::estado::{self, Operacion};
+use crate::git::{self, Git};
+use crate::i18n::catalogo::ciclo as txt;
+use crate::i18n::{Mensaje, Texto};
+use crate::toma::Entorno;
+
+pub use operacion::{DESCARTAR, Descartar, MOTIVO_MAX, Pausar, desde_anotacion, motivo_valido};
+
+/// Nombre de la orden `pausa` (y de su fila de transicion).
+pub const PAUSA: &str = crate::tarea::PAUSA;
+
+/// Nombre de la orden `reanudar` (y de su fila de transicion).
+pub const REANUDAR: &str = crate::tarea::REANUDAR;
+
+fn uso(nombre: &str, motivo: &str, m: impl Into<Mensaje>) -> Informe {
+    Informe::nuevo(nombre, Resultado::Uso)
+        .con("motivo", motivo)
+        .diagnostico(m)
+}
+
+/// `descartar TASK-NNN "<motivo>"` (sin `--json`, que quita `lib.rs`). Sin
+/// motivo, `uso motivo=sin-motivo`; vacio, solo espacios, con saltos de
+/// linea o caracteres de control o de mas de [`MOTIVO_MAX`] caracteres,
+/// `uso motivo=motivo-invalido`. Nada se lee ni se escribe antes.
+pub fn orden_descartar(args: &[String], e: &Entorno<'_>) -> Informe {
+    let u = |motivo: &str| uso(DESCARTAR, motivo, &txt::USO_DESCARTAR);
+    match args {
+        [] => u("sin-id"),
+        [id, ..] if !git::es_id(id) => u("id-invalido"),
+        [_] => u("sin-motivo"),
+        [id, m] if motivo_valido(m) => descartar_en(e, id, m),
+        [_, _] => uso(
+            DESCARTAR,
+            "motivo-invalido",
+            Mensaje::nuevo(&txt::MOTIVO_INVALIDO).con("max", MOTIVO_MAX.to_string()),
+        ),
+        _ => u("argumentos-de-mas"),
+    }
+}
+
+/// `pausa TASK-NNN` o, con `reanudar`, `reanudar TASK-NNN`.
+pub fn orden_pausa(args: &[String], reanudar: bool, e: &Entorno<'_>) -> Informe {
+    let (nombre, m): (&str, &'static Texto) = if reanudar {
+        (REANUDAR, &txt::USO_REANUDAR)
+    } else {
+        (PAUSA, &txt::USO_PAUSA)
+    };
+    match args {
+        [id] if git::es_id(id) => pausa_en(e, id, reanudar),
+        [_] => uso(nombre, "id-invalido", m),
+        [] => uso(nombre, "sin-id", m),
+        _ => uso(nombre, "argumentos-de-mas", m),
+    }
+}
+
+/// `descartar` de la tarea `id` con `motivo` (ya validado) en el repo de
+/// `e`.
+pub fn descartar_en(e: &Entorno<'_>, id: &str, motivo: &str) -> Informe {
+    let i = match paso_descartar(e, id, motivo) {
+        Ok(i) | Err(i) => i,
+    };
+    de_la_orden(i, DESCARTAR, id)
+}
+
+/// `pausa` (o `reanudar`) de la tarea `id` en el repo de `e`.
+pub fn pausa_en(e: &Entorno<'_>, id: &str, reanudar: bool) -> Informe {
+    let op = match (e.quien)() {
+        Ok(q) => Pausar::nueva(id, &q, reanudar, e.ahora),
+        Err(i) => return de_la_orden(i, if reanudar { REANUDAR } else { PAUSA }, id),
+    };
+    let t = escribir(e, &op);
+    let i = con_dueno(t, op.dueno());
+    de_la_orden(i, op.nombre(), id)
+}
+
+fn paso_descartar(e: &Entorno<'_>, id: &str, motivo: &str) -> Result<Informe, Informe> {
+    let g = e.git;
+    let principal = git::localizar_principal(g, e.dir)?;
+    // Exige red (D4): lo de ahora del remoto, tambien las ramas, para saber
+    // si la de la tarea esta publicada antes de quitar su worktree.
+    if git::estado::hay_remoto(g, &principal, DESCARTAR)? {
+        let f = git::fetch(g, &principal);
+        if !f.resultado.es_exito() {
+            return Err(f);
+        }
+    }
+    let quien = (e.quien)()?;
+    let op = Descartar::nueva(id, &quien, motivo, e.ahora);
+    let t = escribir(e, &op);
+    match t.resultado {
+        // Publicado, o ya descartada (repetir termina de quitar el worktree).
+        Resultado::Ok | Resultado::SinCambios => Ok(con_worktree(t, g, &principal, e.dir, id)),
+        _ => Err(con_dueno(t, op.dueno())),
+    }
+}
+
+/// La operacion por el punto unico de escritura, con el `ROADMAP.md`
+/// regenerado en el mismo commit.
+fn escribir(e: &Entorno<'_>, op: &dyn Operacion) -> Informe {
+    let personas = crate::roadmap::personas(e.git, e.dir);
+    let generar = crate::roadmap::generador(&personas);
+    let ctx = estado::Contexto::nuevo(e.git, e.dir, e.catalogo, Some(&generar));
+    estado::transaccion(&ctx, op)
+}
+
+/// Un rechazo con lo que sabe la operacion: el login de quien la lleva o
+/// el arreglo de una tarea de la 0.16.
+fn con_dueno(mut i: Informe, dueno: Option<String>) -> Informe {
+    if i.resultado != Resultado::Rechazada {
+        return i;
+    }
+    match (i.valor("motivo"), dueno) {
+        (Some("de-otra-persona"), Some(l)) => i = i.con("asignado", l),
+        (Some("formato-1"), _) => i = i.con("arreglo", "tc2 actualizar"),
+        _ => {}
+    }
+    i
+}
+
+/// Tras el descarte, el worktree de la tarea (D3): `worktree=quitado`,
+/// `ninguno` (no habia) o `conservado` con `worktree-motivo=` (cambios sin
+/// guardar, commits sin publicar, HEAD suelto, lanzado desde dentro o un
+/// fallo de git). La rama, local y remota, sigue.
+fn con_worktree(i: Informe, g: &Git, principal: &Path, dir: &Path, id: &str) -> Informe {
+    let wt = principal.join(git::CARPETA_WORKTREES).join(id);
+    if dentro_de(dir, &wt) {
+        return i
+            .con("worktree", "conservado")
+            .con("worktree-motivo", "en-uso");
+    }
+    let q = git::quitar_worktree(g, principal, id);
+    match q.resultado {
+        Resultado::Ok => i.con("worktree", "quitado"),
+        Resultado::SinCambios => i.con("worktree", "ninguno"),
+        _ => {
+            let motivo = q.valor("motivo").unwrap_or("error").to_owned();
+            i.con("worktree", "conservado")
+                .con("worktree-motivo", motivo)
+        }
+    }
+}
+
+/// Si `dir` esta dentro de `wt` (o es el): quitarlo dejaria la orden sin
+/// carpeta.
+fn dentro_de(dir: &Path, wt: &Path) -> bool {
+    match (std::fs::canonicalize(dir), std::fs::canonicalize(wt)) {
+        (Ok(d), Ok(w)) => d.starts_with(w),
+        _ => false,
+    }
+}
+
+/// El informe como de la orden: el comando, `id=` el primero y sin la clave
+/// `operacion` de la transaccion. Un `sinred` sin `pendiente` (el `fetch`
+/// previo de `descartar`) dice `pendiente=no`.
+fn de_la_orden(mut i: Informe, nombre: &str, id: &str) -> Informe {
+    nombre.clone_into(&mut i.comando);
+    i.claves.retain(|(k, _)| k != "operacion" && k != "id");
+    i.claves
+        .insert(0, ("id".to_owned(), Valor::Texto(id.to_owned())));
+    if i.resultado == Resultado::Sinred && i.valor("pendiente").is_none() {
+        i = i.con("pendiente", "no");
+    }
+    i
+}
diff --git a/crates/tc2/src/ciclo/operacion.rs b/crates/tc2/src/ciclo/operacion.rs
new file mode 100644
index 0000000..b00f0b0
--- /dev/null
+++ b/crates/tc2/src/ciclo/operacion.rs
@@ -0,0 +1,508 @@
+//! Descartar, pausar y reanudar una tarea como operaciones de la rama de
+//! estado (F3-7; ADR F1-2 y F1-9). Como [`crate::toma::Tomar`], pasan por el
+//! punto unico de escritura y deciden solo con el estado que reciben; leen
+//! unicamente el `tarea.md` de su tarea y lo reescriben: frontmatter, una
+//! fila nueva en `## Transiciones` y el resto byte a byte.
+//!
+//! - Quien (D2 de TASK-034): quien lleva la tarea o cualquiera si no la lleva
+//!   nadie; la de otra persona, `rechazada motivo=de-otra-persona` (o
+//!   `asignado-heredado` si es un texto de la 0.16).
+//! - [`Descartar`]: de `planificada`, `en-diseno`, `en-curso` o
+//!   `en-revision` a `descartada` con su `motivo`; conserva `asignado_a` y
+//!   `rama` (D7). Ya descartada, `sin-cambios` con el motivo de entonces;
+//!   terminada, `rechazada motivo=terminada`. Exige red: cambia `next` y las
+//!   dependencias de todos (D4).
+//! - [`Pausar`]: la pausa no es un estado (D1): una fila `pausa` o
+//!   `reanudar` sin tocar estado, asignado ni rama
+//!   ([`crate::tarea::Cuerpo::en_pausa`]). Repetirla, o reanudar una que no
+//!   esta en pausa, `sin-cambios`. Sin red queda pendiente (D4).
+
+use std::cell::RefCell;
+
+use crate::estado::{Cambio, Estado, Fallo, Operacion, tarea_de};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::ciclo as txt;
+use crate::identidad::Quien;
+use crate::tarea::{Asignado, Cuerpo, PAUSA, REANUDAR, Tarea, Transicion};
+use crate::toma::instante;
+use crate::toma::operacion::MODO;
+
+/// Nombre de `descartar`, de su operacion y de su fila de transicion.
+pub const DESCARTAR: &str = "descartar";
+
+/// Estado en el que queda una tarea descartada.
+pub const DESCARTADA: &str = "descartada";
+
+/// Estados desde los que se descarta, se pausa o se reanuda.
+pub const DE_ORIGEN: [&str; 4] = ["planificada", "en-diseno", "en-curso", "en-revision"];
+
+/// Largo maximo del motivo de un descarte, en caracteres: cabe en una
+/// linea del ROADMAP y en la anotacion del commit.
+pub const MOTIVO_MAX: usize = 500;
+
+/// Si `motivo` vale para un descarte: no vacio, sin saltos de linea ni
+/// caracteres de control (la regla de los titulos) y de [`MOTIVO_MAX`]
+/// caracteres como mucho.
+pub fn motivo_valido(motivo: &str) -> bool {
+    crate::tarea::titulo_valido(motivo) && motivo.chars().count() <= MOTIVO_MAX
+}
+
+/// Lo comun de las tres: la tarea, quien la lanza y cuando.
+#[derive(Debug)]
+struct Lanzada {
+    id: String,
+    asignado: Asignado,
+    decidido_por: &'static str,
+    ahora: u64,
+    dueno: RefCell<Option<String>>,
+}
+
+impl Lanzada {
+    fn nueva(id: &str, asignado: Asignado, decidido_por: &'static str, ahora: u64) -> Self {
+        Self {
+            id: id.to_owned(),
+            asignado,
+            decidido_por,
+            ahora,
+            dueno: RefCell::new(None),
+        }
+    }
+
+    fn rechazo(&self, motivo: &str, m: Mensaje) -> Fallo {
+        Fallo::rechazada(motivo, m.con("id", self.id.as_str()))
+    }
+
+    /// La tarea de `estado`, si quien la lanza puede cambiarla (D2): la lleva
+    /// el o nadie.
+    fn tarea(&self, estado: &dyn Estado) -> Result<(Tarea, Cuerpo), Fallo> {
+        *self.dueno.borrow_mut() = None;
+        let (t, c) = tarea_de(estado, &self.id)?;
+        match &t.asignado_a {
+            Some(a) if !a.misma_persona(&self.asignado) => Err(match (a.login(), a.heredado()) {
+                (Some(login), _) => {
+                    *self.dueno.borrow_mut() = Some(login.to_owned());
+                    self.rechazo(
+                        "de-otra-persona",
+                        Mensaje::nuevo(&txt::DE_OTRA_PERSONA).con("login", login),
+                    )
+                }
+                (None, texto) => self.rechazo(
+                    "asignado-heredado",
+                    Mensaje::nuevo(&txt::ASIGNADO_HEREDADO).con("texto", texto.unwrap_or_default()),
+                ),
+            }),
+            _ => Ok((t, c)),
+        }
+    }
+
+    /// `terminada`, `descartada` u otro estado que no es de [`DE_ORIGEN`].
+    fn estado_no_valido(&self, estado: &str) -> Fallo {
+        match estado {
+            "terminada" => self.rechazo("terminada", Mensaje::nuevo(&txt::TERMINADA)),
+            DESCARTADA => self.rechazo("descartada", Mensaje::nuevo(&txt::DESCARTADA)),
+            otro => self.rechazo(
+                "estado-no-valido",
+                Mensaje::nuevo(&txt::ESTADO_NO_VALIDO).con("estado", otro),
+            ),
+        }
+    }
+
+    /// El `tarea.md` con `actualizado` de hoy y la fila `fase`.
+    fn escribir(&self, mut t: Tarea, mut cuerpo: Cuerpo, fase: &str) -> Vec<Cambio> {
+        let instante = instante(self.ahora);
+        t.actualizado = instante.get(..10).unwrap_or_default().to_owned();
+        cuerpo.anadir_transicion(&Transicion {
+            fecha: &instante,
+            fase,
+            modo: MODO,
+            decidido_por: self.decidido_por,
+            version: crate::VERSION,
+        });
+        vec![Cambio::nuevo(
+            format!("{}/tarea.md", self.id),
+            t.escribir(&cuerpo),
+        )]
+    }
+
+    /// `<nombre>|TASK-NNN|<id>|<login>`.
+    fn anotacion(&self, nombre: &str) -> String {
+        format!(
+            "{nombre}|{}|{}|{}",
+            self.id,
+            self.asignado.id().unwrap_or_default(),
+            self.asignado.login().unwrap_or_default()
+        )
+    }
+}
+
+/// Descartar la tarea `id` con un motivo (RF-07).
+#[derive(Debug)]
+pub struct Descartar {
+    base: Lanzada,
+    motivo: String,
+}
+
+impl Descartar {
+    /// El descarte de `id` por `quien` con `motivo` en el instante `ahora`
+    /// (segundos desde 1970, UTC). El motivo se valida antes, en la orden
+    /// ([`motivo_valido`]).
+    pub fn nueva(id: &str, quien: &Quien, motivo: &str, ahora: u64) -> Self {
+        Self {
+            base: Lanzada::nueva(id, quien.asignado(), quien.decidido_por(), ahora),
+            motivo: motivo.to_owned(),
+        }
+    }
+
+    /// Login de quien lleva la tarea si el ultimo intento la rechazo por
+    /// `de-otra-persona`.
+    pub fn dueno(&self) -> Option<String> {
+        self.base.dueno.borrow().clone()
+    }
+}
+
+impl Operacion for Descartar {
+    fn nombre(&self) -> &str {
+        DESCARTAR
+    }
+
+    fn anotacion(&self) -> String {
+        format!("{}|{}", self.base.anotacion(DESCARTAR), self.motivo)
+    }
+
+    fn exige_red(&self) -> bool {
+        // Cambia `next` y las dependencias de todos: sin red falla (D4).
+        true
+    }
+
+    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
+        let (mut t, cuerpo) = self.base.tarea(estado)?;
+        match t.estado.as_str() {
+            // Ya descartada: nada que escribir; el motivo es el de entonces.
+            DESCARTADA => return Ok(Vec::new()),
+            e if DE_ORIGEN.contains(&e) => {}
+            otro => return Err(self.base.estado_no_valido(otro)),
+        }
+        DESCARTADA.clone_into(&mut t.estado);
+        t.motivo = Some(self.motivo.clone());
+        Ok(self.base.escribir(t, cuerpo, DESCARTAR))
+    }
+}
+
+/// Pausar (o reanudar) la tarea `id` (RF-08).
+#[derive(Debug)]
+pub struct Pausar {
+    base: Lanzada,
+    reanudar: bool,
+}
+
+impl Pausar {
+    /// La pausa de `id` por `quien` en el instante `ahora`; con `reanudar`,
+    /// la vuelta.
+    pub fn nueva(id: &str, quien: &Quien, reanudar: bool, ahora: u64) -> Self {
+        Self {
+            base: Lanzada::nueva(id, quien.asignado(), quien.decidido_por(), ahora),
+            reanudar,
+        }
+    }
+
+    /// Login de quien lleva la tarea si el ultimo intento la rechazo por
+    /// `de-otra-persona`.
+    pub fn dueno(&self) -> Option<String> {
+        self.base.dueno.borrow().clone()
+    }
+
+    fn fase(&self) -> &'static str {
+        if self.reanudar { REANUDAR } else { PAUSA }
+    }
+}
+
+impl Operacion for Pausar {
+    fn nombre(&self) -> &str {
+        self.fase()
+    }
+
+    fn anotacion(&self) -> String {
+        // Con el instante: si queda pendiente sin red, la fila lleva la hora
+        // de la pausa y no la de su publicacion (F1-17 mide las pausas).
+        format!("{}|{}", self.base.anotacion(self.fase()), self.base.ahora)
+    }
+
+    fn exige_red(&self) -> bool {
+        // Transicion de una tarea propia: sin red queda pendiente (D4).
+        false
+    }
+
+    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
+        let (t, cuerpo) = self.base.tarea(estado)?;
+        if !DE_ORIGEN.contains(&t.estado.as_str()) {
+            return Err(self.base.estado_no_valido(&t.estado));
+        }
+        if cuerpo.en_pausa() != self.reanudar {
+            // Ya en pausa (o ya sin ella): nada que escribir.
+            return Ok(Vec::new());
+        }
+        Ok(self.base.escribir(t, cuerpo, self.fase()))
+    }
+}
+
+/// Reconstruye una operacion de este modulo desde su anotacion (para el
+/// catalogo): `descartar|TASK-NNN|<id>|<login>|<motivo>`,
+/// `pausa|TASK-NNN|<id>|<login>|<instante>` o `reanudar|...`. Se re-ejecuta
+/// sin persona delante: `decidido_por: agente`, como en
+/// [`crate::toma::desde_anotacion`].
+pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
+    let mut partes = a.splitn(5, '|');
+    let nombre = partes.next()?;
+    let id = partes.next()?;
+    let cuenta: u64 = partes.next()?.parse().ok()?;
+    let asignado = Asignado::persona(cuenta, partes.next()?)?;
+    let resto = partes.next()?;
+    if !crate::git::es_id(id) {
+        return None;
+    }
+    match nombre {
+        DESCARTAR if motivo_valido(resto) => Some(Box::new(Descartar {
+            base: Lanzada::nueva(id, asignado, "agente", crate::identidad::ahora()),
+            motivo: resto.to_owned(),
+        })),
+        PAUSA | REANUDAR => {
+            let ahora: u64 = resto.parse().ok()?;
+            Some(Box::new(Pausar {
+                base: Lanzada::nueva(id, asignado, "agente", ahora),
+                reanudar: nombre == REANUDAR,
+            }))
+        }
+        _ => None,
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::identidad::Cuenta;
+    use crate::plataforma::Proveedor;
+    use crate::roadmap::tests::EnMemoria;
+    use std::collections::BTreeMap;
+
+    /// 2026-11-17T10:20:30Z.
+    const AHORA: u64 = 1_794_910_830;
+
+    fn ana() -> Quien {
+        Quien::Agente(Cuenta {
+            proveedor: Proveedor::GitHub,
+            id: 7,
+            login: "ana".to_owned(),
+        })
+    }
+
+    fn md(id: &str, estado: &str, asignado: Option<Asignado>) -> String {
+        let mut t = Tarea::nueva(id, &format!("Tarea {id}"), "2026-11-01");
+        t.estado = estado.to_owned();
+        if estado == DESCARTADA {
+            t.motivo = Some("el de entonces".to_owned());
+        }
+        t.asignado_a = asignado;
+        t.rama = Some(crate::alta::rama_de(id, &t.tipo, &t.titulo));
+        t.escribir(&Cuerpo::plantilla())
+    }
+
+    fn estado(tareas: &[(&str, String)]) -> EnMemoria {
+        EnMemoria(
+            tareas
+                .iter()
+                .map(|(id, t)| (format!("{id}/tarea.md"), t.clone()))
+                .collect::<BTreeMap<_, _>>(),
+        )
+    }
+
+    fn motivo(r: &Result<Vec<Cambio>, Fallo>) -> Option<&str> {
+        match r {
+            Err(Fallo::Rechazada { motivo, .. }) => Some(motivo),
+            _ => None,
+        }
+    }
+
+    fn texto(r: &Result<Vec<Cambio>, Fallo>) -> String {
+        match r.as_deref() {
+            Ok([c]) => c.contenido.clone(),
+            otro => format!("{otro:?}"),
+        }
+    }
+
+    fn fila(fase: &str) -> String {
+        format!(
+            "| 2026-11-17T10:20:30Z | {fase} | manual | agente | {} |\n",
+            crate::VERSION
+        )
+    }
+
+    /// C1 (unitario): desde cada estado de origen, `estado: descartada`, el
+    /// motivo, `actualizado` y la fila; asignado, rama y el resto, igual.
+    #[test]
+    fn descartar_escribe_solo_lo_suyo() {
+        let yo = ana().asignado();
+        for (e, asignado) in [
+            ("planificada", None),
+            ("en-diseno", None),
+            ("en-curso", Some(yo.clone())),
+            ("en-revision", Some(yo.clone())),
+        ] {
+            let antes = md("TASK-001", e, asignado);
+            let s = estado(&[("TASK-001", antes.clone())]);
+            let op = Descartar::nueva("TASK-001", &ana(), "ya no hace falta | x", AHORA);
+            let r = op.aplicar(&s);
+            let esperado = antes
+                .replace(
+                    &format!("estado: \"{e}\"\nmotivo: null"),
+                    "estado: \"descartada\"\nmotivo: \"ya no hace falta | x\"",
+                )
+                .replace("actualizado: \"2026-11-01\"", "actualizado: \"2026-11-17\"")
+                + &fila("descartar");
+            assert_eq!(texto(&r), esperado, "{e}");
+            assert!(op.exige_red());
+            assert_eq!(op.dueno(), None);
+        }
+    }
+
+    /// C6 y C7 (unitarios): de otra persona o heredada se rechaza; ya
+    /// descartada, nada que escribir; terminada, rechazada; ilegible o
+    /// inexistente, como `start` (la lectura comun, `tarea_de`).
+    #[test]
+    fn descartar_rechazos_y_sin_cambios() {
+        let beto = Asignado::persona(9, "beto");
+        let heredado = md("TASK-003", "planificada", None)
+            .replace("asignado_a: null", "asignado_a: \"ana@x.com\"");
+        let s = estado(&[
+            ("TASK-001", md("TASK-001", "en-curso", beto.clone())),
+            ("TASK-002", md("TASK-002", DESCARTADA, None)),
+            ("TASK-003", heredado),
+            ("TASK-004", md("TASK-004", "terminada", None)),
+            ("TASK-005", md("TASK-005", DESCARTADA, beto)),
+            ("TASK-006", "---\nid: TASK-006\n---\n".to_owned()),
+        ]);
+        let op = Descartar::nueva("TASK-001", &ana(), "m", AHORA);
+        assert_eq!(motivo(&op.aplicar(&s)), Some("de-otra-persona"));
+        assert_eq!(op.dueno().as_deref(), Some("beto"));
+        let op = Descartar::nueva("TASK-002", &ana(), "otro motivo", AHORA);
+        assert_eq!(op.aplicar(&s), Ok(Vec::new()));
+        for (id, esperado) in [
+            ("TASK-003", "asignado-heredado"),
+            ("TASK-004", "terminada"),
+            ("TASK-005", "de-otra-persona"),
+            ("TASK-006", "ilegible"),
+            ("TASK-050", "no-existe"),
+        ] {
+            let r = Descartar::nueva(id, &ana(), "m", AHORA).aplicar(&s);
+            assert_eq!(motivo(&r), Some(esperado), "{id}");
+        }
+    }
+
+    /// C9 y C10 (unitarios): la pausa anade su fila sin tocar estado,
+    /// asignado ni rama; repetirla no escribe; reanudar anade la suya y
+    /// reanudar sin pausa no escribe.
+    #[test]
+    fn pausa_y_reanudar_por_filas() {
+        let yo = ana().asignado();
+        let antes = md("TASK-001", "en-curso", Some(yo));
+        let s = estado(&[("TASK-001", antes.clone())]);
+        let reanudar = Pausar::nueva("TASK-001", &ana(), true, AHORA);
+        assert_eq!(reanudar.aplicar(&s), Ok(Vec::new()));
+        let pausa = Pausar::nueva("TASK-001", &ana(), false, AHORA);
+        let pausada = texto(&pausa.aplicar(&s));
+        let esperado = antes.replace("actualizado: \"2026-11-01\"", "actualizado: \"2026-11-17\"")
+            + &fila("pausa");
+        assert_eq!(pausada, esperado);
+        assert!(!pausa.exige_red());
+        assert_eq!(pausa.nombre(), "pausa");
+        let s = estado(&[("TASK-001", pausada.clone())]);
+        assert_eq!(pausa.aplicar(&s), Ok(Vec::new()));
+        assert_eq!(texto(&reanudar.aplicar(&s)), pausada + &fila("reanudar"));
+        assert_eq!(reanudar.nombre(), "reanudar");
+    }
+
+    /// C6 y C7 (unitarios): pausar o reanudar la de otra persona, una
+    /// terminada o una descartada se rechaza.
+    #[test]
+    fn pausa_rechazos() {
+        let s = estado(&[
+            (
+                "TASK-001",
+                md("TASK-001", "planificada", Asignado::persona(9, "beto")),
+            ),
+            ("TASK-002", md("TASK-002", "terminada", None)),
+            ("TASK-003", md("TASK-003", DESCARTADA, None)),
+        ]);
+        for reanudar in [false, true] {
+            let op = Pausar::nueva("TASK-001", &ana(), reanudar, AHORA);
+            assert_eq!(motivo(&op.aplicar(&s)), Some("de-otra-persona"));
+            assert_eq!(op.dueno().as_deref(), Some("beto"));
+            for (id, esperado) in [("TASK-002", "terminada"), ("TASK-003", "descartada")] {
+                let r = Pausar::nueva(id, &ana(), reanudar, AHORA).aplicar(&s);
+                assert_eq!(motivo(&r), Some(esperado), "{id} {reanudar}");
+            }
+        }
+    }
+
+    #[test]
+    fn motivo_del_descarte() {
+        for bueno in [
+            "x",
+            "ya no hace falta | por TASK-002",
+            &"a".repeat(MOTIVO_MAX),
+        ] {
+            assert!(motivo_valido(bueno), "{bueno}");
+        }
+        for malo in [
+            "",
+            "   ",
+            "a\nb",
+            "a\tb",
+            "a\u{2028}b",
+            &"a".repeat(MOTIVO_MAX + 1),
+        ] {
+            assert!(!motivo_valido(malo), "{malo:?}");
+        }
+    }
+
+    /// Las anotaciones se reconstruyen igual (la pausa, con su instante) y
+    /// las malas no.
+    #[test]
+    fn anotacion_ida_y_vuelta() {
+        let ops: [Box<dyn Operacion>; 3] = [
+            Box::new(Descartar::nueva("TASK-001", &ana(), "por | esto", AHORA)),
+            Box::new(Pausar::nueva("TASK-001", &ana(), false, AHORA)),
+            Box::new(Pausar::nueva("TASK-001", &ana(), true, AHORA)),
+        ];
+        let anotaciones: Vec<String> = ops.iter().map(|o| o.anotacion()).collect();
+        assert_eq!(
+            anotaciones,
+            [
+                "descartar|TASK-001|7|ana|por | esto",
+                "pausa|TASK-001|7|ana|1794910830",
+                "reanudar|TASK-001|7|ana|1794910830",
+            ]
+        );
+        for (o, a) in ops.iter().zip(&anotaciones) {
+            let r = desde_anotacion(a);
+            assert_eq!(
+                r.as_ref()
+                    .map(|x| (x.nombre().to_owned(), x.anotacion(), x.exige_red())),
+                Some((o.nombre().to_owned(), a.clone(), o.exige_red()))
+            );
+        }
+        for mala in [
+            "descartar|TASK-001|7|ana",
+            "descartar|TASK-001|7|ana|",
+            "descartar|TASK-001|7|ana| ",
+            "descartar|TASK-1|7|ana|m",
+            "descartar|TASK-001|0|ana|m",
+            "pausa|TASK-001|7|ana",
+            "pausa|TASK-001|7|ana|x",
+            "pausa|TASK-001|x|ana|1",
+            "reanudar|TASK-01|7|ana|1",
+            "start|TASK-001|7|ana|1",
+        ] {
+            assert!(desde_anotacion(mala).is_none(), "{mala}");
+        }
+    }
+}
diff --git a/crates/tc2/src/estado/mod.rs b/crates/tc2/src/estado/mod.rs
index 0008894..b3d85f3 100644
--- a/crates/tc2/src/estado/mod.rs
+++ b/crates/tc2/src/estado/mod.rs
@@ -31,7 +31,7 @@ pub use crate::git::Cambio;
 pub use cerrojo::Cerrojo;
 pub use lectura::{Estado, Vista, leer};
 pub use montaje::{Previsto, montar, montar_con, previsto};
-pub use tareas::{Leida, es_tarea_md, tareas};
+pub use tareas::{Leida, es_tarea_md, tarea_de, tareas};
 pub use transaccion::{Catalogo, Contexto, Fallo, Operacion, Politica, Roadmap, transaccion};
 
 /// Rama huerfana del estado (ADR F1-1; nombre en [`crate::huella`]).
diff --git a/crates/tc2/src/estado/tareas.rs b/crates/tc2/src/estado/tareas.rs
index 293fcfb..8b9d1f1 100644
--- a/crates/tc2/src/estado/tareas.rs
+++ b/crates/tc2/src/estado/tareas.rs
@@ -4,7 +4,9 @@
 
 use super::lectura::Estado;
 use super::transaccion::Fallo;
-use crate::tarea::{Documento, ErrorTarea};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::comun as txt;
+use crate::tarea::{Cuerpo, Documento, ErrorTarea, Tarea};
 
 /// Prefijo de las carpetas de tarea.
 const PREFIJO: &str = "TASK-";
@@ -42,6 +44,28 @@ pub fn tareas(estado: &dyn Estado, rutas: &[String]) -> Result<Vec<Leida>, Fallo
         .collect())
 }
 
+/// El `tarea.md` de `id` en formato 2, para cambiarlo: la lectura comun de
+/// las operaciones de una tarea (`start`, `descartar`, `pausa`, `reanudar`).
+/// Sin el fichero, `rechazada motivo=no-existe`; en formato 1 de la 0.16,
+/// `formato-1` (se convierte con `tc2 actualizar`); si no se entiende,
+/// `ilegible`. Los diagnosticos llevan el ID.
+pub fn tarea_de(estado: &dyn Estado, id: &str) -> Result<(Tarea, Cuerpo), Fallo> {
+    let rechazo = |motivo: &str, m: Mensaje| Fallo::rechazada(motivo, m.con("id", id));
+    let Some(texto) = estado.leer(&format!("{id}/tarea.md"))? else {
+        return Err(rechazo("no-existe", Mensaje::nuevo(&txt::TAREA_NO_EXISTE)));
+    };
+    match crate::tarea::leer(&texto) {
+        Ok(Documento::Formato2(t, c)) => Ok((t, c)),
+        Ok(Documento::Formato1(..)) => {
+            Err(rechazo("formato-1", Mensaje::nuevo(&txt::TAREA_FORMATO_1)))
+        }
+        Err(e) => Err(rechazo(
+            "ilegible",
+            Mensaje::nuevo(&txt::TAREA_ILEGIBLE).con("error", e.to_string()),
+        )),
+    }
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
diff --git a/crates/tc2/src/i18n/catalogo/ciclo.rs b/crates/tc2/src/i18n/catalogo/ciclo.rs
new file mode 100644
index 0000000..ad70d3d
--- /dev/null
+++ b/crates/tc2/src/i18n/catalogo/ciclo.rs
@@ -0,0 +1,40 @@
+//! Mensajes de `tc2 descartar`, `tc2 pausa` y `tc2 reanudar` (F3-7).
+
+crate::i18n::textos! {
+    USO_DESCARTAR {
+        es: "uso: descartar [--json] TASK-NNN \"<motivo>\"",
+        en: "usage: descartar [--json] TASK-NNN \"<reason>\"",
+    }
+    USO_PAUSA {
+        es: "uso: pausa [--json] TASK-NNN",
+        en: "usage: pausa [--json] TASK-NNN",
+    }
+    USO_REANUDAR {
+        es: "uso: reanudar [--json] TASK-NNN",
+        en: "usage: reanudar [--json] TASK-NNN",
+    }
+    MOTIVO_INVALIDO {
+        es: "el motivo del descarte no puede estar vacio ni llevar saltos de linea o caracteres de control, y tiene como mucho {max} caracteres",
+        en: "the reason for discarding cannot be empty or contain line breaks or control characters, and has at most {max} characters",
+    }
+    DE_OTRA_PERSONA {
+        es: "la tarea {id} la lleva {login}: solo quien la lleva (o cualquiera, si no la lleva nadie) puede descartarla, pausarla o reanudarla",
+        en: "task {id} is taken by {login}: only whoever has it (or anyone, if nobody has it) can discard, pause or resume it",
+    }
+    ASIGNADO_HEREDADO {
+        es: "la tarea {id} tiene un asignado_a heredado de la 0.16 ({texto}): hay que reasignarla a una persona de la lista antes de cambiarla",
+        en: "task {id} has an assignee inherited from 0.16 ({texto}): it has to be reassigned to a person on the list before changing it",
+    }
+    TERMINADA {
+        es: "la tarea {id} ya esta terminada: no se puede descartar, pausar ni reanudar",
+        en: "task {id} is already finished: it cannot be discarded, paused or resumed",
+    }
+    DESCARTADA {
+        es: "la tarea {id} esta descartada: no se puede pausar ni reanudar",
+        en: "task {id} is discarded: it cannot be paused or resumed",
+    }
+    ESTADO_NO_VALIDO {
+        es: "la tarea {id} esta {estado}: solo se descartan, pausan o reanudan tareas planificadas, en diseno, en curso o en revision",
+        en: "task {id} is {estado}: only planned, in-design, in-progress or in-review tasks can be discarded, paused or resumed",
+    }
+}
diff --git a/crates/tc2/src/i18n/catalogo/cli.rs b/crates/tc2/src/i18n/catalogo/cli.rs
index 212ad1f..7e83773 100644
--- a/crates/tc2/src/i18n/catalogo/cli.rs
+++ b/crates/tc2/src/i18n/catalogo/cli.rs
@@ -10,8 +10,8 @@ crate::i18n::textos! {
         en: "tc2: could not write the output: {error}",
     }
     USO {
-        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
-        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
+        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<motivo>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
+        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<reason>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
     }
     FALTA_LA_ORDEN {
         es: "tc2: falta la orden. {uso}",
diff --git a/crates/tc2/src/i18n/catalogo/comun.rs b/crates/tc2/src/i18n/catalogo/comun.rs
index fe75232..e6c9fc1 100644
--- a/crates/tc2/src/i18n/catalogo/comun.rs
+++ b/crates/tc2/src/i18n/catalogo/comun.rs
@@ -4,4 +4,18 @@ crate::i18n::textos! {
     /// Texto que viene de fuera de `tc2` (git, el sistema, HTTP) y se
     /// muestra tal cual.
     EXTERNO { es: "{texto}", en: "{texto}" }
+    /// La tarea que se quiere cambiar no esta en la rama de estado
+    /// (`start`, `descartar`, `pausa`, `reanudar`).
+    TAREA_NO_EXISTE {
+        es: "la tarea {id} no existe en la rama de estado: comprueba el ID con tc2 status",
+        en: "task {id} does not exist in the state branch: check the ID with tc2 status",
+    }
+    TAREA_FORMATO_1 {
+        es: "la tarea {id} esta en el formato de la 0.16: conviertela con tc2 actualizar antes de cambiarla",
+        en: "task {id} is in the 0.16 format: convert it with tc2 actualizar before changing it",
+    }
+    TAREA_ILEGIBLE {
+        es: "el tarea.md de {id} no se entiende ({error}): corrigelo antes de cambiarla",
+        en: "the tarea.md of {id} cannot be read ({error}): fix it before changing it",
+    }
 }
diff --git a/crates/tc2/src/i18n/catalogo/mod.rs b/crates/tc2/src/i18n/catalogo/mod.rs
index 8c8c27b..a98a1bd 100644
--- a/crates/tc2/src/i18n/catalogo/mod.rs
+++ b/crates/tc2/src/i18n/catalogo/mod.rs
@@ -2,6 +2,7 @@
 //! `src/` escribe sus entradas en el suyo y no se pisan.
 
 pub mod actualizar;
+pub mod ciclo;
 pub mod cli;
 pub mod comun;
 pub mod config;
@@ -23,6 +24,7 @@ use super::Texto;
 pub fn todos() -> impl Iterator<Item = &'static Texto> {
     [
         actualizar::TODOS,
+        ciclo::TODOS,
         cli::TODOS,
         comun::TODOS,
         config::TODOS,
diff --git a/crates/tc2/src/i18n/catalogo/toma.rs b/crates/tc2/src/i18n/catalogo/toma.rs
index ae64cfe..ff7d278 100644
--- a/crates/tc2/src/i18n/catalogo/toma.rs
+++ b/crates/tc2/src/i18n/catalogo/toma.rs
@@ -13,18 +13,6 @@ crate::i18n::textos! {
         es: "la tarea {id} es de tipo release: una release no se empieza con start, sale de develop con tc2 release (ADR F1-3)",
         en: "task {id} is of type release: a release is not started with start, it comes from develop with tc2 release (ADR F1-3)",
     }
-    NO_EXISTE {
-        es: "la tarea {id} no existe en la rama de estado: comprueba el ID con tc2 status",
-        en: "task {id} does not exist in the state branch: check the ID with tc2 status",
-    }
-    FORMATO_1 {
-        es: "la tarea {id} esta en el formato de la 0.16: conviertela con tc2 actualizar antes de empezarla",
-        en: "task {id} is in the 0.16 format: convert it with tc2 actualizar before starting it",
-    }
-    ILEGIBLE {
-        es: "el tarea.md de {id} no se entiende ({error}): corrigelo antes de empezarla",
-        en: "the tarea.md of {id} cannot be read ({error}): fix it before starting it",
-    }
     TOMADA {
         es: "la tarea {id} ya la lleva {login}: elige otra con tc2 next o pide que te la reasigne",
         en: "task {id} is already taken by {login}: pick another one with tc2 next or ask to have it reassigned",
@@ -45,6 +33,10 @@ crate::i18n::textos! {
         es: "la tarea {id} esta {estado}: start solo empieza tareas planificadas o en diseno",
         en: "task {id} is {estado}: start only starts planned or in-design tasks",
     }
+    PAUSADA {
+        es: "la tarea {id} esta en pausa: reanudala con tc2 reanudar {id} antes de empezarla",
+        en: "task {id} is paused: resume it with tc2 reanudar {id} before starting it",
+    }
     DEPENDENCIAS_PENDIENTES {
         es: "la tarea {id} depende de tareas sin terminar ({ids}): terminalas antes o elige otra con tc2 next",
         en: "task {id} depends on unfinished tasks ({ids}): finish them first or pick another one with tc2 next",
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 9853288..766cf08 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -16,12 +16,14 @@
 //! [`importar`] (alta en bloque con dependencias e hito, F3-2) y [`grafo`] y
 //! [`siguiente`] (orden de trabajo y `next`, F3-3) y [`situacion`] y
 //! [`roadmap`] (el calculo comun y `roadmap`, F3-4) y [`tablero`]
-//! (`board` y `status`, F3-5) y [`toma`] (`start`, F3-6). Las capas
+//! (`board` y `status`, F3-5) y [`toma`] (`start`, F3-6) y [`ciclo`]
+//! (`descartar`, `pausa` y `reanudar`, F3-7). Las capas
 //! que vienen entran como modulos de esta biblioteca.
 
 pub mod actualizar_cli;
 pub mod alta;
 pub mod carril;
+pub mod ciclo;
 pub mod config;
 pub mod contrato;
 pub mod descarga;
@@ -149,7 +151,17 @@ where
         }
         [orden, resto @ ..] if orden == toma::NOMBRE => {
             let (json, resto) = con_json(resto);
-            orden_start(&resto).salida_en(json, idioma)
+            de_una_tarea(toma::NOMBRE, |e| toma::orden(&resto, e)).salida_en(json, idioma)
+        }
+        [orden, resto @ ..] if orden == ciclo::DESCARTAR => {
+            let (json, resto) = con_json(resto);
+            de_una_tarea(ciclo::DESCARTAR, |e| ciclo::orden_descartar(&resto, e))
+                .salida_en(json, idioma)
+        }
+        [orden, resto @ ..] if orden == ciclo::PAUSA || orden == ciclo::REANUDAR => {
+            let (json, resto) = con_json(resto);
+            let reanudar = orden == ciclo::REANUDAR;
+            de_una_tarea(orden, |e| ciclo::orden_pausa(&resto, reanudar, e)).salida_en(json, idioma)
         }
         [orden, resto @ ..] if orden == identidad::NOMBRE => {
             let (json, resto) = con_json(resto);
@@ -282,14 +294,15 @@ fn orden_identidad(args: &[String]) -> Informe {
     )
 }
 
-/// `start [--json] TASK-NNN` (F3-6) en la carpeta actual: quien la toma sale
-/// de [`identidad::resolver`] con el terminal y los tokens del proceso.
-fn orden_start(args: &[String]) -> Informe {
+/// Una orden sobre una tarea (`start`, F3-6; `descartar`, `pausa` y
+/// `reanudar`, F3-7) en la carpeta actual: quien la lanza sale de
+/// [`identidad::resolver`] con el terminal y los tokens del proceso.
+fn de_una_tarea(nombre: &str, orden: impl FnOnce(&toma::Entorno<'_>) -> Informe) -> Informe {
     use std::io::IsTerminal;
     let dir = match std::env::current_dir() {
         Ok(d) => d,
         Err(e) => {
-            return Informe::nuevo(toma::NOMBRE, Resultado::Error)
+            return Informe::nuevo(nombre, Resultado::Error)
                 .diagnostico(Mensaje::nuevo(&txt::CARPETA_ACTUAL).con("error", e.to_string()));
         }
     };
@@ -311,7 +324,7 @@ fn orden_start(args: &[String]) -> Informe {
         catalogo: &catalogo,
         ahora,
     };
-    toma::orden(args, &e)
+    orden(&e)
 }
 
 /// `status` o `board` (F3-5) en la carpeta actual.
@@ -340,6 +353,7 @@ fn catalogo(anotacion: &str) -> Option<Box<dyn estado::Operacion>> {
     alta::desde_anotacion(anotacion)
         .or_else(|| roadmap::desde_anotacion(anotacion))
         .or_else(|| toma::desde_anotacion(anotacion))
+        .or_else(|| ciclo::desde_anotacion(anotacion))
 }
 
 /// `new "<titulo>" [campos de RF-01]`: alta de una tarea con su ID reservado
@@ -517,6 +531,21 @@ mod tests {
         assert!(catalogo("start|TASK-001|7").is_none());
     }
 
+    /// F3-7: el catalogo reconstruye descartar, pausa y reanudar (una pausa
+    /// pendiente sin red se re-ejecuta desde aqui, TASK-034 C8).
+    #[test]
+    fn el_catalogo_reconstruye_descartar_pausa_y_reanudar() {
+        for a in [
+            "descartar|TASK-001|7|ana|ya no | hace falta",
+            "pausa|TASK-001|7|ana|1794910830",
+            "reanudar|TASK-001|7|ana|1794910830",
+        ] {
+            let op = catalogo(a);
+            assert_eq!(op.map(|o| o.anotacion()), Some(a.to_owned()), "{a}");
+        }
+        assert!(catalogo("pausa|TASK-001|7|ana").is_none());
+    }
+
     /// TASK-033 C5: un ID mal formado (o sin ID, o de mas) es `uso` (2) sin
     /// tocar nada, con el diagnostico en los dos idiomas.
     #[test]
@@ -544,6 +573,124 @@ mod tests {
         );
     }
 
+    /// TASK-034 C11 y C2: `descartar`, `pausa` y `reanudar` con un ID mal
+    /// formado, sin ID o con argumentos de mas son `uso` (2) como `start`;
+    /// `descartar` sin motivo o con uno invalido, tambien. Nada se lee: no
+    /// hay repo en la carpeta de los tests de la biblioteca que importe.
+    #[test]
+    fn descartar_pausa_y_reanudar_mal_llamados_son_uso() {
+        let largo = "m".repeat(ciclo::MOTIVO_MAX + 1);
+        for (args, motivo, es, en) in [
+            (
+                vec!["descartar"],
+                "sin-id",
+                "uso: descartar",
+                "usage: descartar",
+            ),
+            (
+                vec!["descartar", "TASK-1", "m"],
+                "id-invalido",
+                "uso: descartar",
+                "usage: descartar",
+            ),
+            (
+                vec!["descartar", "TASK-001"],
+                "sin-motivo",
+                "uso: descartar",
+                "usage: descartar",
+            ),
+            (
+                vec!["descartar", "TASK-001", ""],
+                "motivo-invalido",
+                "el motivo",
+                "the reason",
+            ),
+            (
+                vec!["descartar", "TASK-001", "  "],
+                "motivo-invalido",
+                "el motivo",
+                "the reason",
+            ),
+            (
+                vec!["descartar", "TASK-001", "a\nb"],
+                "motivo-invalido",
+                "el motivo",
+                "the reason",
+            ),
+            (
+                vec!["descartar", "TASK-001", "a\tb"],
+                "motivo-invalido",
+                "el motivo",
+                "the reason",
+            ),
+            (
+                vec!["descartar", "TASK-001", &largo],
+                "motivo-invalido",
+                "el motivo",
+                "the reason",
+            ),
+            (
+                vec!["descartar", "--json", "TASK-001", "m", "x"],
+                "argumentos-de-mas",
+                "uso: descartar",
+                "usage: descartar",
+            ),
+            (vec!["pausa"], "sin-id", "uso: pausa", "usage: pausa"),
+            (
+                vec!["pausa", "task-001"],
+                "id-invalido",
+                "uso: pausa",
+                "usage: pausa",
+            ),
+            (
+                vec!["pausa", "TASK-001", "x"],
+                "argumentos-de-mas",
+                "uso: pausa",
+                "usage: pausa",
+            ),
+            (
+                vec!["reanudar"],
+                "sin-id",
+                "uso: reanudar",
+                "usage: reanudar",
+            ),
+            (
+                vec!["reanudar", "TASK-001a1"],
+                "id-invalido",
+                "uso: reanudar",
+                "usage: reanudar",
+            ),
+            (
+                vec!["reanudar", "TASK-001", "x"],
+                "argumentos-de-mas",
+                "uso: reanudar",
+                "usage: reanudar",
+            ),
+        ] {
+            let s_es = ejecutar_en(Idioma::Es, &args);
+            let s_en = ejecutar_en(Idioma::En, &args);
+            assert_eq!((s_es.codigo, s_en.codigo), (2, 2), "{args:?}");
+            let json = args.contains(&"--json");
+            if !json {
+                assert_eq!(
+                    s_es.stdout,
+                    format!("{} uso motivo={motivo}\n", args[0]),
+                    "{args:?}"
+                );
+                assert_eq!(s_es.stdout, s_en.stdout);
+                assert!(s_es.stderr.starts_with(es), "{args:?}: {}", s_es.stderr);
+                assert!(s_en.stderr.starts_with(en), "{args:?}: {}", s_en.stderr);
+            } else {
+                assert_eq!(s_es.stdout.lines().count(), 1);
+                assert!(
+                    s_es.stdout.contains(&format!("\"motivo\":\"{motivo}\"")),
+                    "{}",
+                    s_es.stdout
+                );
+            }
+        }
+    }
+
     #[test]
     fn sin_orden_falla() {
         assert_eq!(ejecutar(Vec::<String>::new()).codigo, 2);
diff --git a/crates/tc2/src/roadmap/generar.rs b/crates/tc2/src/roadmap/generar.rs
index 3af7722..7c0dd24 100644
--- a/crates/tc2/src/roadmap/generar.rs
+++ b/crates/tc2/src/roadmap/generar.rs
@@ -19,6 +19,9 @@ use crate::tarea::Asignado;
 /// Tercera linea del fichero: lo generado y su formato.
 pub const MARCA: &str = "<!-- generado por tc2; formato 2; no editar fuera de la zona manual -->";
 
+/// Como se marca una tarea en pausa (TASK-034).
+const EN_PAUSA: &str = "en pausa";
+
 /// Lo que se consulta al escribir.
 struct Datos<'a> {
     s: &'a Situacion,
@@ -75,13 +78,25 @@ impl<'a> Datos<'a> {
         v
     }
 
-    /// Texto del estado y clase Mermaid de una tarea viva.
+    /// Texto del estado y clase Mermaid de una tarea viva. Una en pausa
+    /// (TASK-034) lo dice: planificada, `en pausa`; empezada, su estado con
+    /// `· en pausa` (sin coma: la linea de «En marcha» se parte por `, `).
     fn estado(&self, f: &Ficha) -> (String, &'static str) {
         let id = f.nodo.id.as_str();
         let e = f.nodo.estado().unwrap_or_default();
         if self.atascadas.contains(id) {
             return ("atascada (ciclo)".to_owned(), "atascada");
         }
+        let (texto, clase) = self.estado_sin_pausa(f, e);
+        match e {
+            _ if !f.en_pausa || e == TERMINADA => (texto, clase),
+            PLANIFICADA => (EN_PAUSA.to_owned(), "espera"),
+            _ => (format!("{texto} · {EN_PAUSA}"), clase),
+        }
+    }
+
+    fn estado_sin_pausa(&self, f: &Ficha, e: &str) -> (String, &'static str) {
+        let id = f.nodo.id.as_str();
         match e {
             TERMINADA => ("terminada".to_owned(), "terminada"),
             "en-diseno" => ("en diseño".to_owned(), "marcha"),
@@ -175,10 +190,10 @@ pub fn generar(s: &Situacion, zona: &str, personas: &Personas) -> String {
     en_marcha(&d, &mut o, "En curso", &["en-diseno", "en-curso"]);
     en_marcha(&d, &mut o, "En revisión", &["en-revision"]);
     hitos(&d, &mut o);
-    plegadas(&d, &mut o, "Terminadas", |f| {
+    plegadas(&d, &mut o, "Terminadas", false, |f| {
         f.nodo.estado() == Some(TERMINADA)
     });
-    plegadas(&d, &mut o, "Descartadas", |f| {
+    plegadas(&d, &mut o, "Descartadas", true, |f| {
         f.nodo.fase == Fase::Descartada
     });
     o
@@ -339,7 +354,16 @@ fn grafo_del_hito(d: &Datos<'_>, abiertas: &[&Ficha]) -> String {
     mermaid::bloque(&nodos, &aristas)
 }
 
-fn plegadas(d: &Datos<'_>, o: &mut String, titulo: &str, filtro: impl Fn(&Ficha) -> bool) {
+/// Las terminadas o las descartadas, plegadas; con `motivo`, cada linea
+/// lleva el de su descarte (`sin motivo` en una de formato 1, que no lo
+/// guarda: lo trae la migracion de F1-12).
+fn plegadas(
+    d: &Datos<'_>,
+    o: &mut String,
+    titulo: &str,
+    motivo: bool,
+    filtro: impl Fn(&Ficha) -> bool,
+) {
     let mut v: Vec<&Ficha> = d.fichas.values().copied().filter(|f| filtro(f)).collect();
     v.sort_by_key(|f| grafo::numero_y_sufijo(&f.nodo.id));
     let _ = writeln!(o, "\n## {titulo}\n");
@@ -353,13 +377,21 @@ fn plegadas(d: &Datos<'_>, o: &mut String, titulo: &str, filtro: impl Fn(&Ficha)
         v.len()
     );
     for f in v {
-        let _ = writeln!(
+        let _ = write!(
             o,
             "- {} — {} (H{})",
             f.nodo.id,
             celda(&f.titulo),
             f.nodo.sprint
         );
+        if motivo {
+            let m = f
+                .motivo
+                .as_deref()
+                .map_or_else(|| "sin motivo".to_owned(), celda);
+            let _ = write!(o, " — motivo: {m}");
+        }
+        o.push('\n');
     }
     o.push_str("\n</details>\n");
 }
diff --git a/crates/tc2/src/roadmap/mod.rs b/crates/tc2/src/roadmap/mod.rs
index 489789d..ce33413 100644
--- a/crates/tc2/src/roadmap/mod.rs
+++ b/crates/tc2/src/roadmap/mod.rs
@@ -727,6 +727,56 @@ pub(crate) mod tests {
         assert_eq!(de_estado(&otra, &personas()).ok(), Some(r));
     }
 
+    /// TASK-034 (C3 y C9): la descartada sale con su motivo y fuera de los
+    /// hitos; una planificada en pausa sale de las listas y el ROADMAP la
+    /// marca, y una en curso en pausa sigue en marcha marcada. Reanudada,
+    /// los mismos bytes que sin pausa.
+    #[test]
+    fn descartadas_con_motivo_y_pausas_marcadas() {
+        let referencia = de_estado(&estado_de_referencia(), &personas()).unwrap_or_default();
+        assert!(referencia.contains("\n- TASK-006 — Ya no (H2) — motivo: ya no hace falta\n"));
+        assert!(!referencia.contains("| TASK-006 |"), "{referencia}");
+        let fila =
+            |fase: &str| format!("| 2026-11-17T10:00:00Z | {fase} | manual | agente | 0.1 |\n");
+        let con_filas = |fases: &[&str]| {
+            let mut e = estado_de_referencia();
+            for id in ["TASK-005", "TASK-002"] {
+                let ruta = format!("{id}/tarea.md");
+                let mut t = e.0.get(&ruta).cloned().unwrap_or_default();
+                t.push_str("## Transiciones\n\n| fecha | fase | modo | decidido_por | version |\n|---|---|---|---|---|\n");
+                for f in fases {
+                    t.push_str(&fila(f));
+                }
+                e.0.insert(ruta, t);
+            }
+            e
+        };
+        let e = con_filas(&["start", "pausa"]);
+        let s = situacion::calcular(&e).unwrap_or_default();
+        assert!(s.listas.is_empty(), "{:?}", s.listas);
+        assert_eq!(s.en_marcha.first().map(String::as_str), Some("TASK-008"));
+        let r = de_estado(&e, &personas()).unwrap_or_default();
+        assert!(r.contains("- **Listas para empezar:** ninguna\n"), "{r}");
+        assert!(r.contains("TASK-002 (en curso · en pausa)"), "{r}");
+        assert!(
+            r.contains("| 4 | TASK-005 | Lista | TASK-001 | en pausa | — |\n"),
+            "{r}"
+        );
+        assert!(
+            r.contains("| 5 | TASK-002 | Motor | TASK-001 | en curso · en pausa | @ana |\n"),
+            "{r}"
+        );
+        assert!(r.contains("T005[\"TASK-005 · Lista\"]:::espera"), "{r}");
+        // La linea de «En marcha» se sigue leyendo como la de `next`.
+        assert_eq!(
+            generar::ids_de_la_linea(&r, "En marcha"),
+            Some(s.en_marcha.clone())
+        );
+        // Reanudada: como sin pausa.
+        let e = con_filas(&["pausa", "reanudar"]);
+        assert_eq!(de_estado(&e, &personas()).ok(), Some(referencia));
+    }
+
     /// D3: con la zona rota se devuelve el anterior tal cual.
     #[test]
     fn zona_rota_deja_el_anterior() {
diff --git a/crates/tc2/src/situacion.rs b/crates/tc2/src/situacion.rs
index 2ec13bf..a742997 100644
--- a/crates/tc2/src/situacion.rs
+++ b/crates/tc2/src/situacion.rs
@@ -26,22 +26,39 @@ pub struct Ficha {
     pub asignado: Option<Asignado>,
     /// Rama de la tarea, si la tiene (la que casa con su PR, F3-5).
     pub rama: Option<String>,
+    /// Por que se descarto (formato 2; el formato 1 no lo lleva).
+    pub motivo: Option<String>,
+    /// En pausa: la ultima fila `pausa`/`reanudar` de sus transiciones es
+    /// `pausa` ([`crate::tarea::Cuerpo::en_pausa`], TASK-034).
+    pub en_pausa: bool,
 }
 
 impl Ficha {
     /// Ficha de un `tarea.md` leido.
     pub fn de(doc: &Documento) -> Self {
-        let (titulo, asignado, rama) = match doc {
-            Documento::Formato1(t, _) => {
-                (t.titulo.clone(), t.asignado_a.clone(), Some(t.rama.clone()))
-            }
-            Documento::Formato2(t, _) => (t.titulo.clone(), t.asignado_a.clone(), t.rama.clone()),
+        let (titulo, asignado, rama, motivo, cuerpo) = match doc {
+            Documento::Formato1(t, c) => (
+                t.titulo.clone(),
+                t.asignado_a.clone(),
+                Some(t.rama.clone()),
+                None,
+                c,
+            ),
+            Documento::Formato2(t, c) => (
+                t.titulo.clone(),
+                t.asignado_a.clone(),
+                t.rama.clone(),
+                t.motivo.clone(),
+                c,
+            ),
         };
         Self {
             nodo: Nodo::de(doc),
             titulo,
             asignado,
             rama: rama.filter(|r| !r.trim().is_empty()),
+            motivo,
+            en_pausa: cuerpo.en_pausa(),
         }
     }
 }
@@ -96,7 +113,7 @@ pub struct Situacion {
     pub hitos: Hitos,
     /// Orden de trabajo, atascadas y avisos.
     pub orden: grafo::Orden,
-    /// Listas para empezar, en el orden de trabajo.
+    /// Listas para empezar, en el orden de trabajo (sin las en pausa).
     pub listas: Vec<String>,
     /// En diseno, en curso o en revision, en el orden de trabajo.
     pub en_marcha: Vec<String>,
@@ -142,8 +159,18 @@ pub fn calcular(estado: &dyn Estado) -> Result<Situacion, Fallo> {
         _ => Hitos::del_roadmap(zona::POR_DEFECTO),
     };
     let orden = grafo::orden_de_trabajo(&nodos, &hitos);
+    // Una en pausa no se ofrece para empezar (TASK-034 C9); sigue en el
+    // orden de trabajo. Con un ID repetido vale su primera ficha, como en el
+    // ROADMAP.
+    let mut listas = grafo::listas(&orden, &nodos);
+    listas.retain(|id| {
+        !fichas
+            .iter()
+            .find(|f| f.nodo.id == *id)
+            .is_some_and(|f| f.en_pausa)
+    });
     Ok(Situacion {
-        listas: grafo::listas(&orden, &nodos),
+        listas,
         en_marcha: grafo::en_marcha(&orden, &nodos),
         fichas,
         nodos,
diff --git a/crates/tc2/src/tarea/cuerpo.rs b/crates/tc2/src/tarea/cuerpo.rs
index c6dd0c2..9a16481 100644
--- a/crates/tc2/src/tarea/cuerpo.rs
+++ b/crates/tc2/src/tarea/cuerpo.rs
@@ -217,6 +217,31 @@ impl Cuerpo {
     }
 }
 
+impl Cuerpo {
+    /// Si la tarea esta en pausa (D1 de TASK-034): de las filas de
+    /// `## Transiciones` con fase [`PAUSA`] o [`REANUDAR`], la ultima es
+    /// `pausa`. Sin ninguna de las dos, no. La pausa no es un estado (F1-9
+    /// cierra la lista): son estas filas, las que F1-17 descuenta.
+    pub fn en_pausa(&self) -> bool {
+        let Some(s) = self.seccion(TRANSICIONES) else {
+            return false;
+        };
+        s.texto
+            .lines()
+            .rev()
+            .filter_map(|l| l.trim().strip_prefix('|')?.split('|').nth(1))
+            .map(str::trim)
+            .find(|fase| *fase == PAUSA || *fase == REANUDAR)
+            == Some(PAUSA)
+    }
+}
+
+/// Fase de la fila que pone una tarea en pausa (`tc2 pausa`).
+pub const PAUSA: &str = "pausa";
+
+/// Fase de la fila que la saca de la pausa (`tc2 reanudar`).
+pub const REANUDAR: &str = "reanudar";
+
 /// Titulo de la seccion de transiciones.
 const TRANSICIONES: &str = "Transiciones";
 
@@ -379,6 +404,31 @@ mod tests {
         assert_eq!(Cuerpo::leer(&c.escribir()), c);
     }
 
+    /// TASK-034 (C9 y C10): en pausa si la ultima fila `pausa`/`reanudar`
+    /// es `pausa`; las demas filas no cuentan, ni una `pausa` fuera de la
+    /// seccion.
+    #[test]
+    fn en_pausa_por_la_ultima_fila_de_pausa_o_reanudar() {
+        let fila = |fase: &'static str| Transicion { fase, ..T };
+        let mut c = Cuerpo::plantilla();
+        assert!(!c.en_pausa());
+        c.anadir_transicion(&fila(PAUSA));
+        assert!(c.en_pausa());
+        // Otra transicion detras no la saca de la pausa.
+        c.anadir_transicion(&fila("start"));
+        assert!(c.en_pausa());
+        c.anadir_transicion(&fila(REANUDAR));
+        assert!(!c.en_pausa());
+        c.anadir_transicion(&fila(PAUSA));
+        assert!(c.en_pausa());
+        // Una «pausa» en otra seccion o fuera de la columna de la fase, no.
+        let c = Cuerpo::leer(
+            "## Objetivo\n| x | pausa | y |\n## Transiciones\n| fecha | fase |\n|---|---|\n| a | reanudar | pausa |\n",
+        );
+        assert!(!c.en_pausa());
+        assert!(!Cuerpo::leer("").en_pausa());
+    }
+
     #[test]
     fn cercas_como_commonmark() {
         let titulos = |t: &str| -> Vec<String> {
diff --git a/crates/tc2/src/tarea/mod.rs b/crates/tc2/src/tarea/mod.rs
index de9ebde..fb22564 100644
--- a/crates/tc2/src/tarea/mod.rs
+++ b/crates/tc2/src/tarea/mod.rs
@@ -17,7 +17,7 @@ pub(crate) mod yaml;
 
 use std::fmt;
 
-pub use cuerpo::{Cuerpo, Seccion, Transicion};
+pub use cuerpo::{Cuerpo, PAUSA, REANUDAR, Seccion, Transicion};
 pub use modelo::{
     Asignado, COMPLEJIDADES, ESTADOS, Heredado, LOGIN_MAX, REGLAS, Rol, RolRevision, Roles, TIPOS,
     Tarea, TareaV1, id_valido, login_valido, titulo_valido,
diff --git a/crates/tc2/src/toma/mod.rs b/crates/tc2/src/toma/mod.rs
index 415937e..4c2a575 100644
--- a/crates/tc2/src/toma/mod.rs
+++ b/crates/tc2/src/toma/mod.rs
@@ -112,6 +112,7 @@ fn rechazo_de_toma(mut i: Informe, op: &Tomar) -> Informe {
         }
         Some("dependencias-pendientes") => i = i.con_lista("dependencias", op.pendientes()),
         Some("formato-1") => i = i.con("arreglo", "tc2 actualizar"),
+        Some("pausada") => i = i.con("arreglo", "tc2 reanudar"),
         _ => {}
     }
     i
diff --git a/crates/tc2/src/toma/operacion.rs b/crates/tc2/src/toma/operacion.rs
index 5109489..9b60a2a 100644
--- a/crates/tc2/src/toma/operacion.rs
+++ b/crates/tc2/src/toma/operacion.rs
@@ -22,12 +22,12 @@
 
 use std::cell::RefCell;
 
-use crate::estado::{Cambio, Estado, Fallo, Operacion};
+use crate::estado::{Cambio, Estado, Fallo, Operacion, tarea_de};
 use crate::grafo::{Nodo, dependencia_cumplida};
 use crate::i18n::Mensaje;
 use crate::i18n::catalogo::toma as txt;
 use crate::identidad::Quien;
-use crate::tarea::{Asignado, Documento, Transicion};
+use crate::tarea::{Asignado, Transicion};
 
 use super::NOMBRE;
 
@@ -122,21 +122,7 @@ impl Operacion for Tomar {
         self.pendientes.borrow_mut().clear();
         *self.rama.borrow_mut() = None;
         let ruta = format!("{}/tarea.md", self.id);
-        let Some(texto) = estado.leer(&ruta)? else {
-            return Err(self.rechazo("no-existe", Mensaje::nuevo(&txt::NO_EXISTE)));
-        };
-        let (mut t, mut cuerpo) = match crate::tarea::leer(&texto) {
-            Ok(Documento::Formato2(t, c)) => (t, c),
-            Ok(Documento::Formato1(..)) => {
-                return Err(self.rechazo("formato-1", Mensaje::nuevo(&txt::FORMATO_1)));
-            }
-            Err(e) => {
-                return Err(self.rechazo(
-                    "ilegible",
-                    Mensaje::nuevo(&txt::ILEGIBLE).con("error", e.to_string()),
-                ));
-            }
-        };
+        let (mut t, mut cuerpo) = tarea_de(estado, &self.id)?;
         let rama = t
             .rama
             .clone()
@@ -180,6 +166,10 @@ impl Operacion for Tomar {
                 ));
             }
         }
+        // En pausa (TASK-034): no se empieza hasta reanudarla.
+        if cuerpo.en_pausa() {
+            return Err(self.rechazo("pausada", Mensaje::nuevo(&txt::PAUSADA)));
+        }
         let rutas: Vec<String> = t
             .dependencias
             .iter()
@@ -262,7 +252,7 @@ mod tests {
     use crate::identidad::Cuenta;
     use crate::plataforma::Proveedor;
     use crate::roadmap::tests::EnMemoria;
-    use crate::tarea::{Cuerpo, Tarea};
+    use crate::tarea::{Cuerpo, Documento, Tarea};
     use std::collections::BTreeMap;
 
     /// 2026-11-17T10:20:30Z.
@@ -480,6 +470,21 @@ mod tests {
         }
     }
 
+    /// TASK-034 C9 (unitario): una planificada en pausa no se empieza
+    /// (`pausada`); reanudada, si.
+    #[test]
+    fn en_pausa_no_se_empieza() {
+        let fila =
+            |fase: &str| format!("| 2026-11-17T09:00:00Z | {fase} | manual | agente | 0.1 |\n");
+        let pausada = md("TASK-001", "planificada", &[], None) + &fila("pausa");
+        let s = estado(&[("TASK-001", pausada.clone())]);
+        let r = Tomar::nueva("TASK-001", &ana(), AHORA).aplicar(&s);
+        assert_eq!(motivo(&r), Some("pausada"));
+        let s = estado(&[("TASK-001", pausada + &fila("reanudar"))]);
+        let r = Tomar::nueva("TASK-001", &ana(), AHORA).aplicar(&s);
+        assert_eq!(r.map(|c| c.len()), Ok(1));
+    }
+
     #[test]
     fn anotacion_ida_y_vuelta() {
         let op = Tomar::nueva("TASK-001", &ana(), AHORA);
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-034/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-034/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-034/tarea.md                                    | 24 ++++++++++++++++++++++--
 3 files changed, 22 insertions(+), 2 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/ciclo.rs"
````

````
crates/tc2/tests/ciclo.rs | 692 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 692 insertions(+)
````
