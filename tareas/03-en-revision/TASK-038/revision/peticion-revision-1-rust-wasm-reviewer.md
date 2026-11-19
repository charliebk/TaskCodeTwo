# Peticion de revision — TASK-038 (ronda 1)

- Tarea: TASK-038 — F4-3 Aprobacion de un hito en lote
- Rama revisada: feature/task-038-f4-3-aprobacion-de-un-hito-en-lote
- Rama base: develop
- Commit revisado (HEAD): 5ae20d8b11347556bddbfe0fe074e9827f4825f5
- Fecha: 2026-11-19
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-038 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-038/tarea.md
- tareas/03-en-revision/TASK-038/planificacion/
- crates/tc2/src/estado/transaccion.rs
- crates/tc2/src/i18n/catalogo/cli.rs
- crates/tc2/src/i18n/catalogo/plan.rs
- crates/tc2/src/plan/aprobar.rs
- crates/tc2/src/plan/mod.rs
- crates/tc2/tests/aprobar.rs

## Commits a revisar (git log develop..HEAD)

````
5ae20d8 chore(TASK-038): autorrevision
5112652 feat(TASK-038): adendas F4-3 en F1-6 y F1-16 y test del tope
5a39354 feat(TASK-038): tc2 approve --hito <n> aprueba un hito en lote
1f2243d chore(TASK-038): tarea en curso
````

## Diff de tu dominio (6 fichero(s) de 8; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/estado/transaccion.rs b/crates/tc2/src/estado/transaccion.rs
index b1c30af..7ea6a95 100644
--- a/crates/tc2/src/estado/transaccion.rs
+++ b/crates/tc2/src/estado/transaccion.rs
@@ -70,6 +70,9 @@ pub trait Operacion {
     /// Como se sella su commit (F4-2): [`Sello::Normal`] salvo `approve`,
     /// que firma. Con [`Sello::Firmado`] las pendientes no entran en ese
     /// commit (D4a de TASK-037): se publican con la siguiente operacion.
+    /// En cada intento se pide antes de [`Operacion::aplicar`] (solo cuenta
+    /// si es [`Sello::Normal`]) y otra vez despues, para construir el
+    /// commit: los trailers pueden salir de lo que decidio `aplicar` (F4-3).
     fn sello(&self) -> Sello {
         Sello::Normal
     }
@@ -320,9 +323,9 @@ fn intento(
     let mut vista = Vista::nueva(git, principal, base.clone())?;
     let mut anotaciones = Vec::new();
     let mut destinos = Vec::new();
-    let sello = op.sello();
+    let firmado = op.sello() != Sello::Normal;
     // Un commit firmado lleva solo lo que firma la persona (D4a).
-    if remoto && sello == Sello::Normal {
+    if remoto && !firmado {
         for p in pendientes::listar(git, principal)? {
             let reconstruida = p.anotacion.as_deref().and_then(|a| (c.catalogo)(a));
             let Some(o) = reconstruida else {
@@ -377,6 +380,10 @@ fn intento(
     }
 
     let mensaje = mensaje(op.nombre(), &anotaciones);
+    // El sello otra vez, ya aplicada la operacion en este intento: sus
+    // trailers pueden depender de lo que decidio `aplicar` (el lote de
+    // `approve --hito`, F4-3). El de antes solo decide las pendientes.
+    let sello = op.sello();
     let Some(sha) = construir_commit_sellado(
         git,
         principal,
diff --git a/crates/tc2/src/i18n/catalogo/cli.rs b/crates/tc2/src/i18n/catalogo/cli.rs
index c78f094..fd27afc 100644
--- a/crates/tc2/src/i18n/catalogo/cli.rs
+++ b/crates/tc2/src/i18n/catalogo/cli.rs
@@ -10,8 +10,8 @@ crate::i18n::textos! {
         en: "tc2: could not write the output: {error}",
     }
     USO {
-        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<motivo>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <carril>] | tc2 plan guardar [--json] TASK-NNN <destino> <fichero> | tc2 approve [--json] TASK-NNN [--pedir-cambios <fichero>] | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
-        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<reason>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <lane>] | tc2 plan guardar [--json] TASK-NNN <destination> <file> | tc2 approve [--json] TASK-NNN [--pedir-cambios <file>] | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
+        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<motivo>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <carril>] | tc2 plan guardar [--json] TASK-NNN <destino> <fichero> | tc2 approve [--json] TASK-NNN [--pedir-cambios <fichero>] | tc2 approve [--json] --hito <n> | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
+        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<reason>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <lane>] | tc2 plan guardar [--json] TASK-NNN <destination> <file> | tc2 approve [--json] TASK-NNN [--pedir-cambios <file>] | tc2 approve [--json] --hito <n> | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
     }
     FALTA_LA_ORDEN {
         es: "tc2: falta la orden. {uso}",
diff --git a/crates/tc2/src/i18n/catalogo/plan.rs b/crates/tc2/src/i18n/catalogo/plan.rs
index 1ac01c7..a44acf3 100644
--- a/crates/tc2/src/i18n/catalogo/plan.rs
+++ b/crates/tc2/src/i18n/catalogo/plan.rs
@@ -63,8 +63,20 @@ crate::i18n::textos! {
         en: "{ruta} is larger than {max} bytes: it is not published",
     }
     USO_APPROVE {
-        es: "uso: approve [--json] TASK-NNN [--pedir-cambios <fichero>]",
-        en: "usage: approve [--json] TASK-NNN [--pedir-cambios <file>]",
+        es: "uso: approve [--json] TASK-NNN [--pedir-cambios <fichero>] | approve [--json] --hito <n>",
+        en: "usage: approve [--json] TASK-NNN [--pedir-cambios <file>] | approve [--json] --hito <n>",
+    }
+    HITO_VACIO {
+        es: "el hito {hito} no tiene ninguna tarea abierta: comprueba el numero con tc2 roadmap",
+        en: "milestone {hito} has no open task: check the number with tc2 roadmap",
+    }
+    HITO_DEMASIADAS {
+        es: "el hito {hito} tiene {n} planes por aprobar, mas que el tope de {max}: apruebalos uno a uno con tc2 approve TASK-NNN",
+        en: "milestone {hito} has {n} plans to approve, more than the limit of {max}: approve them one by one with tc2 approve TASK-NNN",
+    }
+    HITO_NO_LISTAS {
+        es: "{n} tareas del hito {hito} no se pueden aprobar (ver tareas=): no se ha aprobado ninguna; arreglalas y vuelve a lanzar tc2 approve --hito {hito}",
+        en: "{n} tasks of milestone {hito} cannot be approved (see tareas=): none was approved; fix them and run tc2 approve --hito {hito} again",
     }
     DECIDIDO_POR {
         es: "approve no admite --decidido-por: aprueba siempre una persona con su firma, en cualquier modo (ADR F1-10, decision 6)",
diff --git a/crates/tc2/src/plan/aprobar.rs b/crates/tc2/src/plan/aprobar.rs
index 6c9a7f4..86c51f5 100644
--- a/crates/tc2/src/plan/aprobar.rs
+++ b/crates/tc2/src/plan/aprobar.rs
@@ -13,7 +13,13 @@
 //!   (D1a). No concede nada: la puede lanzar tambien el agente. No toca
 //!   `tarea.md` (como `plan guardar`, no es una fase).
 //!
-//! Las dos exigen red y nunca quedan pendientes (no tienen catalogo).
+//! - [`AprobarHito`] (F4-3): `approve --hito <n>`, los planes de todas las
+//!   tareas abiertas del sprint `n` aun sin aprobar, con las mismas
+//!   comprobaciones que [`Aprobar`] ([`aprobable`]), en un unico commit
+//!   firmado con un `Plan-Blob` por tarea. Todo o nada y con tope
+//!   ([`MAX_HITO`]).
+//!
+//! Las tres exigen red y nunca quedan pendientes (no tienen catalogo).
 
 use std::cell::RefCell;
 
@@ -22,12 +28,13 @@ use super::plantillas as p;
 use super::roles::{self, Rol};
 use crate::config::Config;
 use crate::estado::{
-    Cambio, Estado, Fallo, Operacion, Sello, de_otra_persona, por_estado, tarea_de,
+    Cambio, Estado, Fallo, Operacion, Sello, de_otra_persona, por_estado, tarea_de, tareas,
 };
-use crate::i18n::Mensaje;
+use crate::grafo::{DESCARTADA, TERMINADA};
 use crate::i18n::catalogo::plan as txt;
+use crate::i18n::{Mensaje, Texto};
 use crate::identidad::Quien;
-use crate::tarea::{Asignado, Tarea, Transicion};
+use crate::tarea::{Asignado, Cuerpo, Documento, Tarea, Transicion};
 use crate::toma::instante;
 
 /// Nombre de la orden, de la operacion y de su fila.
@@ -86,10 +93,6 @@ impl Aprobar {
     pub fn dueno(&self) -> Option<String> {
         self.dueno.borrow().clone()
     }
-
-    fn rechazo(&self, motivo: &str, m: Mensaje) -> Fallo {
-        Fallo::rechazada(motivo, m.con("id", self.id.as_str()))
-    }
 }
 
 /// Si `plan` es el esqueleto de `plan-final.md` que dejo `plan`: se compara
@@ -164,52 +167,277 @@ impl Operacion for Aprobar {
 
     fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
         *self.dueno.borrow_mut() = None;
-        let (mut t, mut cuerpo) = tarea_de(estado, &self.id)?;
-        if let Some((fallo, dueno)) = de_otra_persona(
+        let (t, cuerpo) = tarea_de(estado, &self.id)?;
+        let plan = estado.leer(&p::ruta_plan_final(&self.id))?;
+        let instante = instante(self.ahora);
+        let quien = Aprobacion {
+            asignado: &self.asignado,
+            instante: &instante,
+            modo: self.modo,
+        };
+        match aprobable(
             &self.id,
-            t.asignado_a.as_ref(),
-            &self.asignado,
-            NOMBRE,
-            ("de-otra-persona", &txt::APROBAR_DE_OTRA_PERSONA),
+            t,
+            cuerpo,
+            plan.as_deref(),
+            &estado.rutas(),
+            &quien,
         ) {
-            *self.dueno.borrow_mut() = dueno;
-            return Err(fallo);
+            Ok(cambio) => Ok(cambio.into_iter().collect()),
+            Err((fallo, dueno)) => {
+                *self.dueno.borrow_mut() = dueno;
+                Err(fallo)
+            }
         }
-        // Ya aprobado: nada que escribir ni que firmar otra vez.
-        if t.plan_aprobado {
-            return Ok(Vec::new());
+    }
+}
+
+/// Quien aprueba, cuando y con que modo por defecto: lo comun a todas las
+/// tareas de una aprobacion (una o un hito).
+struct Aprobacion<'a> {
+    asignado: &'a Asignado,
+    instante: &'a str,
+    modo: &'a str,
+}
+
+/// La aprobacion de la tarea `id` ya leida (`t`, `cuerpo`) con su
+/// `plan-final.md` (`None` si no existe): `Ok(None)` si ya estaba aprobada
+/// (nada que escribir), `Ok(Some)` con el `tarea.md` aprobado, o el rechazo
+/// con el login de quien la lleva si es de otra persona. Las comprobaciones
+/// y su orden son las de F4-2: otra persona, ya aprobada, estado, pausa,
+/// sin plan y esqueleto.
+fn aprobable(
+    id: &str,
+    mut t: Tarea,
+    mut cuerpo: Cuerpo,
+    plan: Option<&str>,
+    rutas: &[String],
+    a: &Aprobacion<'_>,
+) -> Result<Option<Cambio>, (Fallo, Option<String>)> {
+    let rechazo = |motivo: &str, m: Mensaje| (Fallo::rechazada(motivo, m.con("id", id)), None);
+    if let Some(r) = de_otra_persona(
+        id,
+        t.asignado_a.as_ref(),
+        a.asignado,
+        NOMBRE,
+        ("de-otra-persona", &txt::APROBAR_DE_OTRA_PERSONA),
+    ) {
+        return Err(r);
+    }
+    // Ya aprobado: nada que escribir ni que firmar otra vez.
+    if t.plan_aprobado {
+        return Ok(None);
+    }
+    if t.estado != EN_DISENO {
+        return Err((por_estado(id, &t.estado, NOMBRE, &[EN_DISENO]), None));
+    }
+    if cuerpo.en_pausa() {
+        return Err(rechazo("pausada", Mensaje::nuevo(&txt::APROBAR_PAUSADA)));
+    }
+    let Some(plan) = plan else {
+        return Err(rechazo("sin-plan", Mensaje::nuevo(&txt::SIN_PLAN)));
+    };
+    if es_esqueleto(plan, &t, rutas) {
+        return Err(rechazo(
+            "plan-esqueleto",
+            Mensaje::nuevo(&txt::PLAN_ESQUELETO),
+        ));
+    }
+    let modo = modo_congelado(&cuerpo).unwrap_or_else(|| a.modo.to_owned());
+    t.plan_aprobado = true;
+    a.instante
+        .get(..10)
+        .unwrap_or_default()
+        .clone_into(&mut t.actualizado);
+    cuerpo.anadir_transicion(&Transicion {
+        fecha: a.instante,
+        fase: FASE,
+        modo: &modo,
+        decidido_por: PERSONA,
+        version: crate::VERSION,
+    });
+    Ok(Some(Cambio::nuevo(
+        format!("{id}/tarea.md"),
+        t.escribir(&cuerpo),
+    )))
+}
+
+/// Tope de planes que aprueba un `approve --hito` (D5a de TASK-038): solo
+/// cuentan los que cambiarian.
+pub const MAX_HITO: usize = 20;
+
+/// Nombre de la operacion de `--hito` (en el commit).
+pub const NOMBRE_HITO: &str = "approve-hito";
+
+/// `approve --hito <n>` (F4-3): aprobar de una vez los planes de las tareas
+/// abiertas (ni `terminada` ni `descartada`) del sprint `n` que aun no lo
+/// estan, en un unico commit firmado con un `Plan-Blob` por tarea. Todo o
+/// nada: si alguna no se puede aprobar no se escribe ni se firma nada y
+/// [`AprobarHito::fallos`] dice cuales y por que.
+#[derive(Debug)]
+pub struct AprobarHito {
+    hito: u32,
+    asignado: Asignado,
+    ahora: u64,
+    modo: &'static str,
+    lote: RefCell<Vec<String>>,
+    fallos: RefCell<Vec<(String, String)>>,
+}
+
+impl AprobarHito {
+    /// `approve --hito <hito>` por `quien` en `ahora`; `None` si no es una
+    /// persona (RR-04). El modo por defecto es el de [`Aprobar::nueva`].
+    pub fn nueva(hito: u32, quien: &Quien, ahora: u64) -> Option<Self> {
+        let Quien::Persona(_) = quien else {
+            return None;
+        };
+        Some(Self {
+            hito,
+            asignado: quien.asignado(),
+            ahora,
+            modo: Config::suelo().modo_flujo.nombre(),
+            lote: RefCell::new(Vec::new()),
+            fallos: RefCell::new(Vec::new()),
+        })
+    }
+
+    /// El `modo_flujo` de la configuracion de la base (como
+    /// [`Aprobar::con_config`]).
+    #[must_use]
+    pub fn con_config(mut self, config: &Config) -> Self {
+        self.modo = config.modo_flujo.nombre();
+        self
+    }
+
+    /// Los IDs que aprobo el ultimo intento, en orden.
+    pub fn aprobadas(&self) -> Vec<String> {
+        self.lote.borrow().clone()
+    }
+
+    /// `(ID, motivo)` de las tareas que impidieron el ultimo intento
+    /// (`no-listas`), en orden.
+    pub fn fallos(&self) -> Vec<(String, String)> {
+        self.fallos.borrow().clone()
+    }
+
+    /// Rechazo del hito entero con `n` tareas (las que fallan o las que se
+    /// pasan del tope).
+    fn rechazo(&self, motivo: &str, m: &'static Texto, n: usize) -> Fallo {
+        Fallo::rechazada(
+            motivo,
+            Mensaje::nuevo(m)
+                .con("hito", self.hito.to_string())
+                .con("n", n.to_string())
+                .con("max", MAX_HITO.to_string()),
+        )
+    }
+}
+
+/// Lo que importa de un `tarea.md` para el lote: su sprint, si esta abierta
+/// y si ya esta aprobada. `None` si no se entiende (no se sabe su sprint).
+fn del_hito(d: &Result<Documento, crate::tarea::ErrorTarea>) -> Option<(u32, bool, bool)> {
+    let abierta = |e: &str| e != TERMINADA && e != DESCARTADA;
+    match d.as_ref().ok()? {
+        Documento::Formato2(t, _) => Some((t.sprint, abierta(&t.estado), t.plan_aprobado)),
+        Documento::Formato1(t, _) => Some((t.sprint, abierta(&t.estado), t.plan_aprobado)),
+    }
+}
+
+impl Operacion for AprobarHito {
+    fn nombre(&self) -> &str {
+        NOMBRE_HITO
+    }
+
+    fn anotacion(&self) -> String {
+        format!(
+            "{NOMBRE_HITO}|{}|{}|{}",
+            self.hito,
+            self.asignado.id().unwrap_or_default(),
+            self.asignado.login().unwrap_or_default()
+        )
+    }
+
+    fn exige_red(&self) -> bool {
+        true
+    }
+
+    /// Un `Plan-Blob` por tarea del lote, en orden de ID. El lote lo fija
+    /// [`AprobarHito::aplicar`]: la transaccion vuelve a pedir el sello
+    /// despues de aplicar en cada intento.
+    fn sello(&self) -> Sello {
+        Sello::Firmado {
+            trailers: self
+                .lote
+                .borrow()
+                .iter()
+                .map(|id| (PLAN_BLOB, p::ruta_plan_final(id)))
+                .collect(),
         }
-        if t.estado != EN_DISENO {
-            return Err(por_estado(&self.id, &t.estado, NOMBRE, &[EN_DISENO]));
+    }
+
+    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
+        self.lote.borrow_mut().clear();
+        self.fallos.borrow_mut().clear();
+        let rutas = estado.rutas();
+        let mut abiertas = 0usize;
+        let mut candidatas = Vec::new();
+        for (ruta, doc) in tareas(estado, &rutas)? {
+            let Some((sprint, abierta, aprobada)) = del_hito(&doc) else {
+                continue;
+            };
+            if sprint != self.hito || !abierta {
+                continue;
+            }
+            abiertas += 1;
+            if !aprobada {
+                let id = ruta.split_once('/').map_or("", |(c, _)| c).to_owned();
+                candidatas.push((id, doc));
+            }
         }
-        if cuerpo.en_pausa() {
-            return Err(self.rechazo("pausada", Mensaje::nuevo(&txt::APROBAR_PAUSADA)));
+        if abiertas == 0 {
+            return Err(self.rechazo("hito-vacio", &txt::HITO_VACIO, 0));
         }
-        let ruta = p::ruta_plan_final(&self.id);
-        let Some(plan) = estado.leer(&ruta)? else {
-            return Err(self.rechazo("sin-plan", Mensaje::nuevo(&txt::SIN_PLAN)));
-        };
-        if es_esqueleto(&plan, &t, &estado.rutas()) {
-            return Err(self.rechazo("plan-esqueleto", Mensaje::nuevo(&txt::PLAN_ESQUELETO)));
+        if candidatas.len() > MAX_HITO {
+            return Err(self.rechazo("demasiadas", &txt::HITO_DEMASIADAS, candidatas.len()));
         }
-        let modo = modo_congelado(&cuerpo).unwrap_or_else(|| self.modo.to_owned());
+        let planes: Vec<String> = candidatas
+            .iter()
+            .map(|(id, _)| p::ruta_plan_final(id))
+            .collect();
+        let planes = estado.leer_varios(&planes)?;
         let instante = instante(self.ahora);
-        t.plan_aprobado = true;
-        instante
-            .get(..10)
-            .unwrap_or_default()
-            .clone_into(&mut t.actualizado);
-        cuerpo.anadir_transicion(&Transicion {
-            fecha: &instante,
-            fase: FASE,
-            modo: &modo,
-            decidido_por: PERSONA,
-            version: crate::VERSION,
-        });
-        Ok(vec![Cambio::nuevo(
-            format!("{}/tarea.md", self.id),
-            t.escribir(&cuerpo),
-        )])
+        let quien = Aprobacion {
+            asignado: &self.asignado,
+            instante: &instante,
+            modo: self.modo,
+        };
+        let mut cambios = Vec::new();
+        let mut lote = Vec::new();
+        let mut fallos = Vec::new();
+        for ((id, doc), plan) in candidatas.into_iter().zip(planes) {
+            // `del_hito` solo deja pasar las que se entienden: la otra es
+            // una de la 0.16, que se convierte con `tc2 actualizar`.
+            let Ok(Documento::Formato2(t, c)) = doc else {
+                fallos.push((id, "formato-1".to_owned()));
+                continue;
+            };
+            match aprobable(&id, t, c, plan.as_deref(), &rutas, &quien) {
+                Ok(Some(c)) => {
+                    cambios.push(c);
+                    lote.push(id);
+                }
+                Ok(None) => {}
+                Err((Fallo::Rechazada { motivo, .. }, _)) => fallos.push((id, motivo)),
+                Err((e @ Fallo::Error(_), _)) => return Err(e),
+            }
+        }
+        if !fallos.is_empty() {
+            let n = fallos.len();
+            *self.fallos.borrow_mut() = fallos;
+            return Err(self.rechazo("no-listas", &txt::HITO_NO_LISTAS, n));
+        }
+        *self.lote.borrow_mut() = lote;
+        Ok(cambios)
     }
 }
 
@@ -520,4 +748,225 @@ mod tests {
         let ya = PedirCambios::nueva("TASK-003", &agente, "x\n");
         assert_eq!(motivo(&ya.aplicar(&e)), Some("plan-aprobado"));
     }
+
+    /// `tarea.md` de `id` en `estado` y en el sprint `sprint`.
+    fn md_sprint(id: &str, estado: &str, sprint: u32) -> String {
+        let Ok(Documento::Formato2(mut t, c)) = crate::tarea::leer(&md(id, "en-diseno")) else {
+            unreachable!()
+        };
+        estado.clone_into(&mut t.estado);
+        if estado == DESCARTADA {
+            t.motivo = Some("ya no hace falta".to_owned());
+        }
+        t.sprint = sprint;
+        let texto = t.escribir(&c);
+        assert!(crate::tarea::leer(&texto).is_ok(), "{texto}");
+        texto
+    }
+
+    fn id(n: usize) -> String {
+        format!("TASK-{n:03}")
+    }
+
+    fn plan_de(n: usize) -> String {
+        p::ruta_plan_final(&id(n))
+    }
+
+    fn hito(n: u32) -> AprobarHito {
+        AprobarHito::nueva(n, &persona(), AHORA)
+            .unwrap_or_else(|| unreachable!())
+            .con_config(&Config::suelo())
+    }
+
+    /// C2 de TASK-038: el agente no puede construir un `AprobarHito`
+    /// (RR-04); el sello es firmado y la anotacion lleva el hito.
+    #[test]
+    fn solo_una_persona_aprueba_un_hito() {
+        assert!(AprobarHito::nueva(4, &Quien::Agente(cuenta()), AHORA).is_none());
+        let op = hito(4);
+        assert!(op.exige_red());
+        assert_eq!(op.anotacion(), "approve-hito|4|7|ana");
+        assert_eq!(op.sello(), Sello::Firmado { trailers: vec![] });
+    }
+
+    /// C1 de TASK-038 (unitario): diez planes del sprint 4 de una vez, con
+    /// la misma fila e instante; la ya aprobada, la terminada y la de otro
+    /// sprint no entran; el sello lleva diez `Plan-Blob` en orden de ID.
+    #[test]
+    fn aplicar_hito_aprueba_diez() {
+        let mut m = BTreeMap::new();
+        for n in 1..=10 {
+            m.insert(
+                format!("{}/tarea.md", id(n)),
+                md_sprint(&id(n), "en-diseno", 4),
+            );
+            m.insert(plan_de(n), format!("# Plan {n}\n"));
+        }
+        m.insert(
+            "TASK-011/tarea.md".to_owned(),
+            md_sprint("TASK-011", "en-diseno", 4)
+                .replace("plan_aprobado: false", "plan_aprobado: true"),
+        );
+        m.insert(plan_de(11), "# Plan\n".to_owned());
+        m.insert(
+            "TASK-012/tarea.md".to_owned(),
+            md_sprint("TASK-012", "terminada", 4),
+        );
+        m.insert(
+            "TASK-013/tarea.md".to_owned(),
+            md_sprint("TASK-013", "en-diseno", 3),
+        );
+        m.insert(plan_de(13), "# Plan\n".to_owned());
+        let e = EnMemoria(m);
+        let op = hito(4);
+        let cambios = op.aplicar(&e).unwrap_or_default();
+        let esperadas: Vec<String> = (1..=10).map(id).collect();
+        let rutas: Vec<String> = cambios.iter().map(|c| c.ruta.clone()).collect();
+        assert_eq!(
+            rutas,
+            esperadas
+                .iter()
+                .map(|i| format!("{i}/tarea.md"))
+                .collect::<Vec<_>>()
+        );
+        let fila = format!(
+            "| 2026-11-19T10:20:30Z | approve | semiautomatico | persona | {} |\n",
+            crate::VERSION
+        );
+        for c in &cambios {
+            assert!(c.contenido.contains("\nplan_aprobado: true\n"), "{c:?}");
+            assert!(c.contenido.contains("\nactualizado: \"2026-11-19\"\n"));
+            assert!(c.contenido.ends_with(&fila), "{}", c.contenido);
+        }
+        assert_eq!(op.aprobadas(), esperadas);
+        assert!(op.fallos().is_empty());
+        assert_eq!(
+            op.sello(),
+            Sello::Firmado {
+                trailers: (1..=10).map(|n| (PLAN_BLOB, plan_de(n))).collect()
+            }
+        );
+    }
+
+    /// C1 de TASK-038 (unitario): todo o nada. Una sin plan y otra en pausa
+    /// rechazan el hito entero (`no-listas`, con las dos y su motivo, sin
+    /// lote ni trailers); otra persona tambien; sin abiertas, `hito-vacio`;
+    /// 21 por aprobar, `demasiadas`; todas aprobadas, nada que hacer.
+    #[test]
+    fn hito_todo_o_nada() {
+        let mut m = BTreeMap::new();
+        // Sprint 4: una lista, una sin plan, una en pausa.
+        m.insert(
+            "TASK-001/tarea.md".to_owned(),
+            md_sprint("TASK-001", "en-diseno", 4),
+        );
+        m.insert(plan_de(1), "# Plan\n".to_owned());
+        m.insert(
+            "TASK-002/tarea.md".to_owned(),
+            md_sprint("TASK-002", "planificada", 4),
+        );
+        m.insert(
+            "TASK-003/tarea.md".to_owned(),
+            md_sprint("TASK-003", "en-diseno", 4)
+                + "| 2026-11-18T10:00:00Z | pausa | manual | agente | 0.1 |\n",
+        );
+        m.insert(plan_de(3), "# Plan\n".to_owned());
+        // Sprint 5: solo terminadas o descartadas.
+        m.insert(
+            "TASK-004/tarea.md".to_owned(),
+            md_sprint("TASK-004", "terminada", 5),
+        );
+        m.insert(
+            "TASK-005/tarea.md".to_owned(),
+            md_sprint("TASK-005", "descartada", 5),
+        );
+        // Sprint 7: todas aprobadas.
+        m.insert(
+            "TASK-006/tarea.md".to_owned(),
+            md_sprint("TASK-006", "en-diseno", 7)
+                .replace("plan_aprobado: false", "plan_aprobado: true"),
+        );
+        // Sprint 8: la de otra persona.
+        m.insert(
+            "TASK-007/tarea.md".to_owned(),
+            md_sprint("TASK-007", "en-diseno", 8)
+                .replace("asignado_a: null", "asignado_a: {id: 9, login: \"beto\"}"),
+        );
+        m.insert(plan_de(7), "# Plan\n".to_owned());
+        // Sprint 6: 21 por aprobar.
+        for n in 100..121 {
+            m.insert(
+                format!("{}/tarea.md", id(n)),
+                md_sprint(&id(n), "en-diseno", 6),
+            );
+            m.insert(plan_de(n), "# Plan\n".to_owned());
+        }
+        let e = EnMemoria(m);
+
+        let op = hito(4);
+        let r = op.aplicar(&e);
+        assert_eq!(motivo(&r), Some("no-listas"), "{r:?}");
+        assert_eq!(
+            op.fallos(),
+            vec![
+                ("TASK-002".to_owned(), "estado-no-valido".to_owned()),
+                ("TASK-003".to_owned(), "pausada".to_owned()),
+            ]
+        );
+        assert!(op.aprobadas().is_empty());
+        assert_eq!(op.sello(), Sello::Firmado { trailers: vec![] });
+
+        assert_eq!(motivo(&hito(5).aplicar(&e)), Some("hito-vacio"));
+        assert_eq!(motivo(&hito(9).aplicar(&e)), Some("hito-vacio"));
+        assert_eq!(motivo(&hito(6).aplicar(&e)), Some("demasiadas"));
+        assert_eq!(hito(7).aplicar(&e), Ok(Vec::new()));
+        let op = hito(8);
+        assert_eq!(motivo(&op.aplicar(&e)), Some("no-listas"));
+        assert_eq!(
+            op.fallos(),
+            vec![("TASK-007".to_owned(), "de-otra-persona".to_owned())]
+        );
+
+        // Una en `en-diseno` sin plan-final.md: `sin-plan`.
+        let mut m = e.0.clone();
+        m.insert(
+            "TASK-002/tarea.md".to_owned(),
+            md_sprint("TASK-002", "en-diseno", 4),
+        );
+        m.remove(&plan_de(3));
+        m.insert(
+            "TASK-003/tarea.md".to_owned(),
+            md_sprint("TASK-003", "en-diseno", 4),
+        );
+        let op = hito(4);
+        assert_eq!(motivo(&op.aplicar(&EnMemoria(m))), Some("no-listas"));
+        assert_eq!(
+            op.fallos(),
+            vec![
+                ("TASK-002".to_owned(), "sin-plan".to_owned()),
+                ("TASK-003".to_owned(), "sin-plan".to_owned()),
+            ]
+        );
+    }
+
+    /// Exactamente en el tope (20) si se aprueba.
+    #[test]
+    fn hito_justo_en_el_tope() {
+        let mut m = BTreeMap::new();
+        for n in 1..=MAX_HITO {
+            m.insert(
+                format!("{}/tarea.md", id(n)),
+                md_sprint(&id(n), "en-diseno", 4),
+            );
+            m.insert(plan_de(n), "# Plan\n".to_owned());
+        }
+        // D5a: una ya aprobada del mismo sprint no cuenta para el tope.
+        m.insert(
+            "TASK-099/tarea.md".to_owned(),
+            md_sprint("TASK-099", "en-diseno", 4)
+                .replace("plan_aprobado: false", "plan_aprobado: true"),
+        );
+        let op = hito(4);
+        assert_eq!(op.aplicar(&EnMemoria(m)).map(|c| c.len()), Ok(MAX_HITO));
+    }
 }
diff --git a/crates/tc2/src/plan/mod.rs b/crates/tc2/src/plan/mod.rs
index 47d4073..e2c7892 100644
--- a/crates/tc2/src/plan/mod.rs
+++ b/crates/tc2/src/plan/mod.rs
@@ -27,6 +27,13 @@
 //! - `approve [--json] TASK-NNN --pedir-cambios <fichero>`: publica
 //!   `brainstorm/cambios-pedidos-<n>.md` en la ronda mayor y la cierra
 //!   ([`PedirCambios`], D1a); la puede lanzar tambien el agente.
+//! - `approve [--json] --hito <n>` (F4-3): como `approve` (persona primero,
+//!   `fetch`, configuracion), con [`aprobar::AprobarHito`]: los planes de
+//!   las tareas abiertas del sprint `n` (`4` o `H4`) en un unico commit
+//!   firmado, todo o nada. Linea: `approve ok hito= commit= ...
+//!   aprobadas=`; si alguna no esta lista, `rechazada motivo=no-listas
+//!   tareas=ID:motivo,...`; tambien `hito-vacio` y `demasiadas max=20`. Con
+//!   un ID, con `--pedir-cambios` o con un hito que no es un numero, `uso`.
 //!
 //! Lanzadas desde un worktree de tarea escriben el estado en el principal
 //! (lo localiza la transaccion). El entorno es el de `start`
@@ -51,7 +58,7 @@ use crate::i18n::catalogo::plan as txt;
 use crate::plataforma::errores::sanear;
 use crate::toma::Entorno;
 
-pub use aprobar::{Aprobar, PedirCambios};
+pub use aprobar::{Aprobar, AprobarHito, PedirCambios};
 pub use guardar::{Guardar, destino_valido};
 pub use operacion::{Forma, Planificar, Resumen};
 
@@ -179,6 +186,9 @@ pub fn orden_approve(args: &[String], e: &Entorno<'_>) -> Informe {
     let Some((id, resto)) = args.split_first() else {
         return uso_approve("sin-id");
     };
+    if id == HITO {
+        return orden_approve_hito(resto, e);
+    }
     if !git::es_id(id) {
         return uso_approve("id-invalido");
     }
@@ -187,11 +197,9 @@ pub fn orden_approve(args: &[String], e: &Entorno<'_>) -> Informe {
     while let Some(a) = it.next() {
         match a.as_str() {
             // F1-10 decision 6: la parada `approve` firmado es fija.
-            x if x == "--decidido-por" || x.starts_with("--decidido-por=") => {
-                return Informe::nuevo(aprobar::NOMBRE, Resultado::Uso)
-                    .con("motivo", "decidido-por")
-                    .diagnostico(&txt::DECIDIDO_POR);
-            }
+            x if es_decidido_por(x) => return decidido_por(),
+            // Un hito o una tarea, no las dos cosas.
+            x if x == HITO || x.starts_with("--hito=") => return uso_approve("hito-con-id"),
             "--pedir-cambios" if fichero.is_none() => match it.next() {
                 Some(f) => fichero = Some(f),
                 None => return uso_approve("sin-fichero"),
@@ -211,6 +219,98 @@ pub fn orden_approve(args: &[String], e: &Entorno<'_>) -> Informe {
     }
 }
 
+/// Opcion de `approve` que aprueba un hito entero (F4-3).
+const HITO: &str = "--hito";
+
+fn es_decidido_por(a: &str) -> bool {
+    a == "--decidido-por" || a.starts_with("--decidido-por=")
+}
+
+fn decidido_por() -> Informe {
+    Informe::nuevo(aprobar::NOMBRE, Resultado::Uso)
+        .con("motivo", "decidido-por")
+        .diagnostico(&txt::DECIDIDO_POR)
+}
+
+/// El numero de hito de `--hito`: `4` o `H4` (como lo escribe el
+/// roadmap).
+fn numero_de_hito(a: &str) -> Option<u32> {
+    let cifras = a.strip_prefix(['H', 'h']).unwrap_or(a);
+    if cifras.is_empty() || !cifras.bytes().all(|b| b.is_ascii_digit()) {
+        return None;
+    }
+    cifras.parse().ok()
+}
+
+/// `approve --hito <n>`: sin ID ni `--pedir-cambios` (`uso`).
+fn orden_approve_hito(args: &[String], e: &Entorno<'_>) -> Informe {
+    let Some((n, resto)) = args.split_first() else {
+        return uso_approve("sin-hito");
+    };
+    let Some(hito) = numero_de_hito(n) else {
+        return uso_approve("hito-invalido");
+    };
+    match resto.first().map(String::as_str) {
+        None => aprobar_hito_en(e, hito),
+        Some(x) if es_decidido_por(x) => decidido_por(),
+        Some(x) if git::es_id(x) => uso_approve("hito-con-id"),
+        Some("--pedir-cambios") => uso_approve("hito-con-pedir-cambios"),
+        Some(_) => uso_approve("argumentos-de-mas"),
+    }
+}
+
+/// `approve --hito <hito>` en el repo de `e` (F4-3): la linea lleva
+/// `hito=` primero, `aprobadas=` con el lote y, si alguna tarea no esta
+/// lista, `tareas=ID:motivo,...`.
+pub fn aprobar_hito_en(e: &Entorno<'_>, hito: u32) -> Informe {
+    let mut i = match paso_aprobar_hito(e, hito) {
+        Ok(i) | Err(i) => i,
+    };
+    aprobar::NOMBRE.clone_into(&mut i.comando);
+    i.claves
+        .retain(|(k, _)| k != "operacion" && k != "id" && k != "hito");
+    i.claves
+        .insert(0, ("hito".to_owned(), Valor::Texto(hito.to_string())));
+    if i.resultado == Resultado::Sinred && i.valor("pendiente").is_none() {
+        i = i.con("pendiente", "no");
+    }
+    i
+}
+
+fn paso_aprobar_hito(e: &Entorno<'_>, hito: u32) -> Result<Informe, Informe> {
+    // Como `approve`: quien, lo primero.
+    let quien = match (e.quien)() {
+        Ok(q) => q,
+        Err(i) if i.resultado == Resultado::Rechazada => return Err(sin_persona(Some(i))),
+        Err(i) => return Err(i),
+    };
+    let Some(op) = aprobar::AprobarHito::nueva(hito, &quien, e.ahora) else {
+        return Err(sin_persona(None));
+    };
+    traer(e.git, e.dir)?;
+    let op = op.con_config(&config_de_base(e.git, e.dir)?);
+    let t = escribir(e, &op);
+    match t.resultado {
+        Resultado::Ok => Ok(t.con_lista("aprobadas", op.aprobadas())),
+        Resultado::SinCambios => Ok(t),
+        Resultado::Rechazada => {
+            let t = match t.valor("motivo") {
+                Some("no-listas") => t.con_lista(
+                    "tareas",
+                    op.fallos()
+                        .into_iter()
+                        .map(|(id, m)| format!("{id}:{m}"))
+                        .collect(),
+                ),
+                Some("demasiadas") => t.con("max", aprobar::MAX_HITO.to_string()),
+                _ => t,
+            };
+            Err(t)
+        }
+        _ => Err(t),
+    }
+}
+
 /// `approve` de la tarea `id` en el repo de `e`.
 pub fn aprobar_en(e: &Entorno<'_>, id: &str) -> Informe {
     let i = match paso_aprobar(e, id) {
@@ -462,6 +562,68 @@ mod tests {
         }
     }
 
+    /// C2 de TASK-038 (unitario): `approve --hito` con el agente es
+    /// `rechazada motivo=identidad rol=persona` sin una llamada a git; mal
+    /// llamado, `uso` sin preguntar siquiera quien es.
+    #[test]
+    fn approve_hito_sin_persona_o_mal_llamado_no_toca_nada() {
+        use crate::identidad::Cuenta;
+        use crate::plataforma::Proveedor;
+        let agente = || -> Result<Quien, Informe> {
+            Ok(Quien::Agente(Cuenta {
+                proveedor: Proveedor::GitHub,
+                id: 7,
+                login: "ana".to_owned(),
+            }))
+        };
+        type Quienes<'a> = &'a dyn Fn() -> Result<Quien, Informe>;
+        let nadie = || -> Result<Quien, Informe> { unreachable!() };
+        let casos: [(&[&str], Quienes<'_>, &str); 7] = [
+            (
+                &["--hito", "4"],
+                &agente,
+                "approve rechazada hito=4 motivo=identidad rol=persona",
+            ),
+            (
+                &["--hito", "H4"],
+                &agente,
+                "approve rechazada hito=4 motivo=identidad rol=persona",
+            ),
+            (&["--hito", "x"], &nadie, "approve uso motivo=hito-invalido"),
+            (
+                &["--hito", "-1"],
+                &nadie,
+                "approve uso motivo=hito-invalido",
+            ),
+            (&["--hito"], &nadie, "approve uso motivo=sin-hito"),
+            (
+                &["TASK-001", "--hito=4"],
+                &nadie,
+                "approve uso motivo=hito-con-id",
+            ),
+            (
+                &["--hito", "4", "--decidido-por", "automatico"],
+                &nadie,
+                "approve uso motivo=decidido-por",
+            ),
+        ];
+        for (args, quien, esperado) in casos {
+            let g = Git::nuevo();
+            let dir = std::env::temp_dir();
+            let e = Entorno {
+                dir: &dir,
+                git: &g,
+                quien,
+                catalogo: &sin_catalogo,
+                ahora: 0,
+            };
+            let args: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
+            let i = orden_approve(&args, &e);
+            assert_eq!(i.linea(), esperado, "{args:?}");
+            assert!(g.registro().is_empty(), "{:?}", g.registro());
+        }
+    }
+
     /// El uso no anuncia `--carril errata`: `plan` nunca la acepta (todo
     /// tipo declara fix o funcionalidad y el carril solo sube).
     #[test]
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-038/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-038/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-038/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/aprobar.rs"
````

````
crates/tc2/tests/aprobar.rs | 246 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 246 insertions(+)
````
