# Peticion de revision — TASK-085 (ronda 1)

- Tarea: TASK-085 — F4-6 plan propone skills, agentes y modelo por rol
- Rama revisada: feature/task-085-f4-6-plan-propone-skills-agentes-y-model
- Rama base: develop
- Commit revisado (HEAD): 0689c2cec9429cd0695cd68458dcdad880695989
- Fecha: 2026-11-22
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-085 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-085/tarea.md
- tareas/03-en-revision/TASK-085/planificacion/
- crates/tc2/src/config/claves.rs
- crates/tc2/src/config/mod.rs
- crates/tc2/src/doctor/recursos.rs
- crates/tc2/src/i18n/catalogo/seleccion.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/plan/aprobar.rs
- crates/tc2/src/plan/heuristica.rs
- crates/tc2/src/plan/mod.rs
- crates/tc2/src/plan/operacion.rs
- crates/tc2/src/plan/plantillas.rs
- crates/tc2/src/revision/mod.rs
- crates/tc2/src/revision/operacion.rs
- crates/tc2/src/seleccion/catalogo.rs
- crates/tc2/src/seleccion/dominios.rs
- crates/tc2/src/seleccion/mod.rs
- crates/tc2/src/seleccion/orden.rs
- crates/tc2/src/seleccion/tabla.rs
- crates/tc2/src/tarea/modelo.rs
- crates/tc2/tests/aprobar.rs
- crates/tc2/tests/comun/mod.rs
- crates/tc2/tests/plan.rs
- crates/tc2/tests/seleccion.rs
- crates/tc2/tests/tarea.rs

## Commits a revisar (git log develop..HEAD)

````
0689c2c docs(TASK-085): tabla de 10 tareas de TaskCodeTwo para confirmar por Carlos
7345f6a feat(TASK-085): orden tc2 seleccion, adenda F4-6 en F1-15 y 10 tareas de TaskCodeTwo
4dde725 feat(TASK-085): plan anota la seleccion, approve la muestra y la revision usa dominios
8244644 feat(TASK-085): mapa roles ampliado en tarea.md
a45118d feat(TASK-085): seleccionar por rol (diseno, implementacion, revision) y catalogo ampliado
7b4efa6 feat(TASK-085): tabla-dominios, tres columnas de modelo y detector dominios()
4287a9b chore(TASK-085): tarea en curso
````

## Diff de tu dominio (23 fichero(s) de 48; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/config/claves.rs b/crates/tc2/src/config/claves.rs
index 927f582..ec09a1a 100644
--- a/crates/tc2/src/config/claves.rs
+++ b/crates/tc2/src/config/claves.rs
@@ -464,7 +464,22 @@ pub(crate) fn tabla() -> Vec<Campo> {
                         uno_de(&["skill", "agente", "ejecutable"]),
                         "que es",
                     ),
-                    Campo::opcional("rol", uno_de(&["revisor"]), "rol que cubre"),
+                    Campo::opcional(
+                        "rol",
+                        uno_de(&["revisor", "diseno", "ejecucion"]),
+                        "rol que cubre; sin rol no se elige (F1-15 decision 8)",
+                    ),
+                    Campo::opcional(
+                        "etiquetas",
+                        lista(texto(Regla::Nombre), 1),
+                        "etiquetas con las que compite una skill de ejecucion (S07)",
+                    )
+                    .con_defecto(json!([])),
+                    Campo::opcional(
+                        "prioridad",
+                        entero(0, 1000),
+                        "desempate de S07: mayor primero",
+                    ),
                     Campo::opcional(
                         "dominios",
                         lista(uno_de(&DOMINIOS), 1),
diff --git a/crates/tc2/src/config/mod.rs b/crates/tc2/src/config/mod.rs
index 49c8006..7076ede 100644
--- a/crates/tc2/src/config/mod.rs
+++ b/crates/tc2/src/config/mod.rs
@@ -220,10 +220,17 @@ pub struct Recurso {
     pub id: String,
     /// `skill`, `agente` o `ejecutable`.
     pub tipo: String,
-    /// `revisor`, si lo es.
+    /// `revisor`, `diseno` o `ejecucion` (F1-15 decision 8): sin rol no se
+    /// elige.
     pub rol: Option<String>,
     /// Dominios de F1-15 que cubre.
     pub dominios: Vec<String>,
+    /// Etiquetas (solape de S07; F4-6).
+    #[serde(default)]
+    pub etiquetas: Vec<String>,
+    /// Prioridad (desempate de S07; F4-6).
+    #[serde(default)]
+    pub prioridad: Option<u32>,
 }
 
 /// `seleccion` (F1-15).
diff --git a/crates/tc2/src/doctor/recursos.rs b/crates/tc2/src/doctor/recursos.rs
index bdc5429..fef78ed 100644
--- a/crates/tc2/src/doctor/recursos.rs
+++ b/crates/tc2/src/doctor/recursos.rs
@@ -172,11 +172,20 @@ fn recomendados(e: &Entorno<'_>) -> (Vec<Recomendado>, Vec<Hallazgo>) {
                 fuente: fuente.clone(),
             });
         }
-        if let Some(a) = &r.revision.agente {
+        // Los agentes que anoto la seleccion (F4-6) y el lanzador de la
+        // revision.
+        for a in r
+            .diseno
+            .agentes
+            .iter()
+            .chain(&r.implementacion.agentes)
+            .chain(&r.revision.agentes)
+            .chain(&r.revision.agente)
+        {
             v.push(Recomendado {
                 id: a.clone(),
                 tipo: TipoRecurso::Agente,
-                fuente,
+                fuente: fuente.clone(),
             });
         }
     }
diff --git a/crates/tc2/src/i18n/catalogo/seleccion.rs b/crates/tc2/src/i18n/catalogo/seleccion.rs
index b22314c..f2981fc 100644
--- a/crates/tc2/src/i18n/catalogo/seleccion.rs
+++ b/crates/tc2/src/i18n/catalogo/seleccion.rs
@@ -1,10 +1,14 @@
-//! Mensajes de `tc2 revisoras` (F6-3).
+//! Mensajes de `tc2 revisoras` (F6-3) y `tc2 seleccion` (F4-6).
 
 crate::i18n::textos! {
     USO_REVISORAS {
         es: "uso: revisoras [--json] --rutas <fichero|-> [--nul] [--tarea TASK-NNN] [--carril <carril>] [--complejidad <nivel>] [--plugin <nombre>=<carpeta>]... [--informe <fichero> --seccion <salida> [--artefacto <nombre>]] [--ronda <n>] [--detalle <salida.json>]",
         en: "usage: revisoras [--json] --rutas <file|-> [--nul] [--tarea TASK-NNN] [--carril <lane>] [--complejidad <level>] [--plugin <name>=<folder>]... [--informe <file> --seccion <output> [--artefacto <name>]] [--ronda <n>] [--detalle <output.json>]",
     }
+    USO_SELECCION {
+        es: "uso: seleccion [--json] TASK-NNN | --tarea-fichero <tarea.md> [--rutas <fichero>]; da la tarea por su ID (rama de estado) o por fichero",
+        en: "usage: seleccion [--json] TASK-NNN | --tarea-fichero <tarea.md> [--rutas <file>]; give the task by its ID (state branch) or by file",
+    }
     NO_SE_LEE {
         es: "no se puede leer ni escribir {ruta}: {error}",
         en: "cannot read or write {ruta}: {error}",
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 56374d5..68abc7e 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -228,6 +228,10 @@ where
             let (json, resto) = con_json(resto);
             orden_revisoras(&resto).salida_en(json, idioma)
         }
+        [orden, resto @ ..] if orden == seleccion::orden::SELECCION => {
+            let (json, resto) = con_json(resto);
+            orden_seleccion(&resto).salida_en(json, idioma)
+        }
         [orden, resto @ ..] if orden == hallazgos::orden::NOMBRE => {
             let (json, resto) = con_json(resto);
             hallazgos::orden::orden(&resto, json, idioma)
@@ -555,7 +559,7 @@ fn orden_review(args: &[String], e: &toma::Entorno<'_>) -> Informe {
 
 /// La carpeta de Claude Code (`CLAUDE_CONFIG_DIR` o `~/.claude`), solo para
 /// leer el inventario (F3-9, F6-3).
-fn dir_claude() -> Option<std::path::PathBuf> {
+pub(crate) fn dir_claude() -> Option<std::path::PathBuf> {
     instalacion::Entorno::de_variables(
         instalacion::Sistema::actual(),
         std::path::PathBuf::new(),
@@ -597,6 +601,29 @@ fn orden_revisoras(args: &[String]) -> Informe {
     )
 }
 
+/// `seleccion [--json] ...` (F4-6) en la carpeta actual.
+fn orden_seleccion(args: &[String]) -> Informe {
+    let dir = match std::env::current_dir() {
+        Ok(d) => d,
+        Err(e) => {
+            return Informe::nuevo(seleccion::orden::SELECCION, Resultado::Error)
+                .diagnostico(Mensaje::nuevo(&txt::CARPETA_ACTUAL).con("error", e.to_string()));
+        }
+    };
+    let g = git::Git::nuevo();
+    let claude = dir_claude();
+    let stdin = || Ok(Vec::new());
+    seleccion::orden::orden_seleccion(
+        args,
+        &seleccion::orden::Entorno {
+            dir: &dir,
+            git: &g,
+            claude: claude.as_deref(),
+            stdin: &stdin,
+        },
+    )
+}
+
 /// `estimar [--json] [TASK-NNN | --error TASK-NNN]` (F3-13) en la carpeta
 /// actual. Sin `--json` y con exito, la linea para la persona va detras de
 /// la de contrato (como mucho 5 lineas, F1-4 decision 4).
diff --git a/crates/tc2/src/plan/aprobar.rs b/crates/tc2/src/plan/aprobar.rs
index 0d2514c..df123d4 100644
--- a/crates/tc2/src/plan/aprobar.rs
+++ b/crates/tc2/src/plan/aprobar.rs
@@ -60,6 +60,7 @@ pub struct Aprobar {
     ahora: u64,
     modo: &'static str,
     dueno: RefCell<Option<String>>,
+    roles: RefCell<Option<crate::tarea::Roles>>,
 }
 
 impl Aprobar {
@@ -77,9 +78,16 @@ impl Aprobar {
             ahora,
             modo: Config::suelo().modo_flujo.nombre(),
             dueno: RefCell::new(None),
+            roles: RefCell::new(None),
         })
     }
 
+    /// `roles` de la tarea leida en el ultimo intento: la seleccion que
+    /// firma la persona (F4-6, F1-15 decision 3).
+    pub fn roles(&self) -> Option<crate::tarea::Roles> {
+        self.roles.borrow().clone()
+    }
+
     /// El `modo_flujo` de la configuracion de la base, para la fila si la
     /// tarea no tiene ninguna `plan`.
     #[must_use]
@@ -168,6 +176,7 @@ impl Operacion for Aprobar {
     fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
         *self.dueno.borrow_mut() = None;
         let (t, cuerpo) = tarea_de(estado, &self.id)?;
+        *self.roles.borrow_mut() = Some(t.roles.clone());
         let plan = estado.leer(&p::ruta_plan_final(&self.id))?;
         let instante = instante(self.ahora);
         let quien = Aprobacion {
diff --git a/crates/tc2/src/plan/heuristica.rs b/crates/tc2/src/plan/heuristica.rs
index ab589e2..ec9c8b8 100644
--- a/crates/tc2/src/plan/heuristica.rs
+++ b/crates/tc2/src/plan/heuristica.rs
@@ -275,7 +275,7 @@ pub fn nivel(puntos: u32) -> &'static str {
 }
 
 /// Roles que pide un nivel sin el tope de `hotfix`.
-fn por_nivel(nivel: &str) -> u8 {
+pub fn por_nivel(nivel: &str) -> u8 {
     NIVELES
         .iter()
         .position(|n| *n == nivel)
diff --git a/crates/tc2/src/plan/mod.rs b/crates/tc2/src/plan/mod.rs
index e2c7892..446ebc3 100644
--- a/crates/tc2/src/plan/mod.rs
+++ b/crates/tc2/src/plan/mod.rs
@@ -10,7 +10,10 @@
 //!   peticiones de la ronda en `brainstorm/` y el esqueleto de
 //!   `plan-final.md` si no existe; la tarea queda `en-diseno` con el carril
 //!   y el modo congelados. Linea: `plan ok id= commit= ... carril= modo=
-//!   ronda= roles= forma=redaccion|unificador peticiones=`. El CLI no llama
+//!   ronda= roles= forma=redaccion|unificador peticiones= dominios=
+//!   modelo-diseno= modelo-implementacion= avisos=` (F4-6: la seleccion de
+//!   diseno e implementacion, anotada en `roles.diseno` y
+//!   `roles.implementacion`). El CLI no llama
 //!   a ningun modelo (como la 0.16): quien orquesta lanza los agentes.
 //! - `plan guardar [--json] TASK-NNN <destino> <fichero>`: publica un
 //!   fichero (de fuera del repo) como `plan-final.md` o como la salida de un
@@ -358,11 +361,35 @@ fn paso_aprobar(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
     let op = op.con_config(&config_de_base(e.git, e.dir)?);
     let t = escribir(e, &op);
     match t.resultado {
-        Resultado::Ok | Resultado::SinCambios => Ok(t),
+        Resultado::Ok | Resultado::SinCambios => Ok(con_seleccion(t, op.roles())),
         _ => Err(con_dueno(t, op.dueno())),
     }
 }
 
+/// La seleccion que firma la persona (F4-6; F1-15 decision 3): dominios
+/// con su senal (`ninguno` tambien), roles de diseno y modelos, leidos de
+/// `roles.diseno` y `roles.implementacion` sin recalcular. Una tarea sin la
+/// seleccion de F4-6 (planificada antes) no la muestra.
+fn con_seleccion(i: Informe, roles: Option<crate::tarea::Roles>) -> Informe {
+    let Some(r) = roles.filter(|r| !r.diseno.reglas.is_empty()) else {
+        return i;
+    };
+    let n = r
+        .diseno
+        .agentes
+        .iter()
+        .filter(|a| !a.ends_with(roles::UNIFICADOR))
+        .count();
+    let senales: Vec<String> = r.diseno.senales.clone();
+    i.con("dominios", lista_o_ninguno(&senales))
+        .con("roles", n.to_string())
+        .con("modelo-diseno", r.diseno.modelo.as_deref().unwrap_or("-"))
+        .con(
+            "modelo-implementacion",
+            r.implementacion.modelo.as_deref().unwrap_or("-"),
+        )
+}
+
 fn paso_pedir_cambios(e: &Entorno<'_>, id: &str, contenido: &str) -> Result<Informe, Informe> {
     let quien = (e.quien)()?;
     traer(e.git, e.dir)?;
@@ -424,7 +451,12 @@ fn paso_plan(e: &Entorno<'_>, id: &str, carril: Option<Carril>) -> Result<Inform
     traer(e.git, e.dir)?;
     let quien = (e.quien)()?;
     let config = config_de_base(e.git, e.dir)?;
-    let op = Planificar::nueva(id, &quien, carril, &config, e.ahora);
+    // Inventario y catalogo para la seleccion (F4-6): se leen antes de la
+    // operacion, que no lee el disco.
+    let claude = crate::dir_claude();
+    let f = crate::seleccion::orden::fuentes(e.git, e.dir, claude.as_deref(), &[]);
+    let op = Planificar::nueva(id, &quien, carril, &config, e.ahora)
+        .con_fuentes(f.inventario, f.catalogo);
     let t = escribir(e, &op);
     match t.resultado {
         Resultado::Ok | Resultado::SinCambios => Ok(con_resumen(t, op.resumen())),
@@ -470,6 +502,24 @@ fn con_resumen(i: Informe, r: Option<Resumen>) -> Informe {
         .con_lista("roles", r.roles.iter().map(|x| (*x).to_owned()).collect())
         .con("forma", r.forma.nombre())
         .con_lista("peticiones", r.peticiones)
+        .con("dominios", lista_o_ninguno(&r.dominios))
+        .con("modelo-diseno", o_guion(&r.modelos.0))
+        .con("modelo-implementacion", o_guion(&r.modelos.1))
+        .con_lista("avisos", r.avisos)
+}
+
+/// `a,b` o `ninguno` (F1-15: «dominios: ninguno» tambien se muestra).
+pub(crate) fn lista_o_ninguno(v: &[String]) -> String {
+    if v.is_empty() {
+        "ninguno".to_owned()
+    } else {
+        v.join(",")
+    }
+}
+
+/// El valor o `-` si esta vacio.
+pub(crate) fn o_guion(s: &str) -> &str {
+    if s.is_empty() { "-" } else { s }
 }
 
 /// Un rechazo con lo que sabe la operacion: quien la lleva o el arreglo de
diff --git a/crates/tc2/src/plan/operacion.rs b/crates/tc2/src/plan/operacion.rs
index cc7aba7..471b3c5 100644
--- a/crates/tc2/src/plan/operacion.rs
+++ b/crates/tc2/src/plan/operacion.rs
@@ -14,10 +14,18 @@
 //! - Modo: el de la ultima fila `plan` de `## Transiciones` si la hay (ya
 //!   congelado); si no, `modo_flujo` de la configuracion de la base. Va en
 //!   la columna `modo` de la fila `plan` (F1-10, decision 5).
-//! - Roles: [`Carril::roles_diseno`] con la complejidad
-//!   ([`heuristica::resolver`]) y [`roles::seleccionar`]; un rol es la forma
-//!   `redaccion` (`peticion-plan-<n>.md`), el resto `unificador` (una
-//!   peticion por rol y `peticion-unificador-<n>.md`).
+//! - Roles (F4-6): [`seleccion::seleccionar`] para diseno e implementacion
+//!   con [`Carril::roles_diseno`] por la complejidad
+//!   ([`heuristica::resolver`]), los dominios del texto y el inventario y
+//!   el catalogo que recibe la operacion como dato: suelo 2 con
+//!   `puertas-seguridad`, rol de rendimiento con su dominio, unificador con
+//!   dos o mas, y el modelo de cada rol. Los ids elegidos se mapean a
+//!   [`roles`] por su sufijo. Un rol es la forma `redaccion`
+//!   (`peticion-plan-<n>.md`), el resto `unificador` (una peticion por rol y
+//!   `peticion-unificador-<n>.md`); cada peticion dice el agente y el
+//!   modelo. La seleccion se anota en `roles.diseno` y
+//!   `roles.implementacion`; un agente no instalado no rompe `plan`: va con
+//!   su aviso `no-instalado` (S14).
 //! - Ronda: la mayor con testigo (`peticion-unificador-<n>.md` o
 //!   `peticion-plan-<n>.md`) en `TASK-NNN/planificacion/brainstorm/`. Si sus
 //!   peticiones son exactamente las que tocan ahora y no tiene cambios
@@ -43,6 +51,8 @@ use crate::estado::{Cambio, Estado, Fallo, Operacion, de_otra_persona, por_estad
 use crate::i18n::catalogo::plan as txt;
 use crate::i18n::{Mensaje, Texto};
 use crate::identidad::Quien;
+use crate::inventario::Inventario;
+use crate::seleccion::{self, Catalogo, dominios};
 use crate::tarea::{Asignado, Cuerpo, Transicion};
 use crate::toma::instante;
 
@@ -89,6 +99,12 @@ pub struct Resumen {
     pub forma: Forma,
     /// Peticiones de la ronda (nombres de fichero de `brainstorm/`).
     pub peticiones: Vec<String>,
+    /// Dominios detectados (F4-6).
+    pub dominios: Vec<String>,
+    /// Modelo de diseno y de implementacion.
+    pub modelos: (String, String),
+    /// Avisos de la seleccion (diseno e implementacion).
+    pub avisos: Vec<String>,
 }
 
 /// Planificar la tarea `id`.
@@ -101,6 +117,8 @@ pub struct Planificar {
     carril: Option<Carril>,
     modo: &'static str,
     config: Config,
+    inventario: Inventario,
+    catalogo: Catalogo,
     resumen: RefCell<Option<Resumen>>,
     dueno: RefCell<Option<String>>,
 }
@@ -123,11 +141,70 @@ impl Planificar {
             carril,
             modo: config.modo_flujo.nombre(),
             config: config.clone(),
+            inventario: Inventario::default(),
+            catalogo: Catalogo::default(),
             resumen: RefCell::new(None),
             dueno: RefCell::new(None),
         }
     }
 
+    /// El inventario y el catalogo con los que se selecciona (leidos antes
+    /// de la operacion, que no lee el disco).
+    #[must_use]
+    pub fn con_fuentes(mut self, inventario: Inventario, catalogo: Catalogo) -> Self {
+        self.inventario = inventario;
+        self.catalogo = catalogo;
+        self
+    }
+
+    /// La seleccion de diseno e implementacion de la tarea (F4-6).
+    fn seleccion(
+        &self,
+        t: &crate::tarea::Tarea,
+        enunciado: &Enunciado,
+        carril: Carril,
+        resolucion: &heuristica::Resolucion,
+        n: u8,
+    ) -> (seleccion::Seleccion, seleccion::Seleccion) {
+        let tabla = seleccion::tabla::dominios(seleccion::tabla::ADR).unwrap_or_default();
+        let texto = dominios::Texto {
+            objetivo: &enunciado.objetivo,
+            criterios: &enunciado.criterios,
+            etiquetas: &t.etiquetas,
+        };
+        let doms = dominios::dominios(&texto, &[], &tabla, &self.config.seleccion.dominios_extra);
+        let entrada = seleccion::Entrada {
+            rutas: &[],
+            dominios: &doms,
+            etiquetas: &t.etiquetas,
+            roles_base: n,
+        };
+        let ctx = seleccion::Contexto {
+            carril,
+            complejidad: Some(seleccion::orden::complejidad(resolucion)),
+            modelos_permitidos: &self.config.seleccion.modelos_permitidos,
+            ronda: 1,
+        };
+        let s = |rol| {
+            seleccion::seleccionar(rol, &entrada, &self.catalogo, &self.inventario, &ctx)
+                .unwrap_or_default()
+        };
+        (s(seleccion::Rol::Diseno), s(seleccion::Rol::Implementacion))
+    }
+
+    /// `roles.<rol>` con la seleccion `s` (la `regla` de la 0.16 se
+    /// conserva).
+    fn anotar(&self, r: &mut crate::tarea::Rol, s: &seleccion::Seleccion, agentes: &[String]) {
+        r.modelo = (!s.modelo.is_empty()).then(|| s.modelo.clone());
+        r.skills.clone_from(&s.skills);
+        r.agentes = agentes.to_vec();
+        r.reglas.clone_from(&s.reglas);
+        r.senales.clone_from(&s.senales);
+        r.avisos.clone_from(&s.avisos);
+        r.inventario = Some(self.inventario.huella());
+        r.catalogo = Some(self.catalogo.huella());
+    }
+
     /// Lo que decidio el ultimo intento que llego a decidir.
     pub fn resumen(&self) -> Option<Resumen> {
         self.resumen.borrow().clone()
@@ -234,6 +311,46 @@ fn peticiones(d: &Datos<'_>, roles: &[Rol], forma: Forma) -> Vec<(String, String
     v
 }
 
+/// Los dominios de unas senales `<dominio>:<senal>:<valor>`, sin repetir.
+pub fn dominios_de(senales: &[String]) -> Vec<String> {
+    let mut v: Vec<String> = Vec::new();
+    for s in senales {
+        let d = s.split(':').next().unwrap_or_default().to_owned();
+        if !v.contains(&d) {
+            v.push(d);
+        }
+    }
+    v
+}
+
+/// Los roles de [`roles`] que corresponden a los agentes de diseno de la
+/// seleccion (por el sufijo del id: `taskcode-plugin:brainstorm-riesgos`
+/// es `brainstorm-riesgos`), sin el unificador, que decide la forma. Con
+/// `n` por encima de la escalera (`politica_carriles`, hasta 5) se suman los
+/// condicionales que falten, como [`roles::seleccionar`]. Si la tabla no se
+/// pudo leer (sin filas aplicadas), los de antes de F4-6.
+fn roles_de(s: &seleccion::Seleccion, n: u8, etiquetas: &[String]) -> Vec<Rol> {
+    if s.reglas.is_empty() {
+        return roles::seleccionar(n, etiquetas);
+    }
+    let mut v: Vec<Rol> = s
+        .roles
+        .iter()
+        .filter_map(|id| {
+            let sufijo = id.rsplit(':').next().unwrap_or(id);
+            roles::todos().find(|r| r.id == sufijo).copied()
+        })
+        .collect();
+    let mut extra = usize::from(n).saturating_sub(roles::ESCALERA.len());
+    for c in &roles::CONDICIONALES {
+        if extra > 0 && !v.contains(&c.rol) {
+            extra -= 1;
+            v.push(c.rol);
+        }
+    }
+    v
+}
+
 impl Operacion for Planificar {
     fn nombre(&self) -> &str {
         NOMBRE
@@ -310,7 +427,8 @@ impl Operacion for Planificar {
             &enunciado,
         );
         let n = carril.roles_diseno(resolucion.roles, &t.tipo, &self.config);
-        let roles = roles::seleccionar(n, &t.etiquetas);
+        let (sd, si) = self.seleccion(&t, &enunciado, carril, &resolucion, n);
+        let roles = roles_de(&sd, n, &t.etiquetas);
         let forma = if roles.len() == 1 {
             Forma::Redaccion
         } else {
@@ -331,6 +449,8 @@ impl Operacion for Planificar {
             fecha: &fecha,
             enunciado: &enunciado,
             resolucion: &resolucion,
+            agentes: &sd.roles,
+            modelo: (!sd.modelo.is_empty()).then_some(sd.modelo.as_str()),
         };
         let nombres = |ronda: u32| -> Vec<String> {
             peticiones(&datos(ronda), &roles, forma)
@@ -362,13 +482,21 @@ impl Operacion for Planificar {
             roles: roles.iter().map(|r| r.titulo).collect(),
             forma,
             peticiones: nombres(ronda),
+            dominios: dominios_de(&sd.senales),
+            modelos: (sd.modelo.clone(), si.modelo.clone()),
+            avisos: sd.avisos.iter().chain(&si.avisos).cloned().collect(),
         });
+        let mut nuevos = t.roles.clone();
+        self.anotar(&mut nuevos.diseno, &sd, &sd.roles);
+        self.anotar(&mut nuevos.implementacion, &si, &[]);
         let cambia_tarea = t.estado != EN_DISENO
             || t.carril.as_deref() != Some(carril.nombre())
+            || t.roles != nuevos
             || !cambios.is_empty();
         if cambia_tarea {
             EN_DISENO.clone_into(&mut t.estado);
             t.carril = Some(carril.nombre().to_owned());
+            t.roles = nuevos;
             fecha.clone_into(&mut t.actualizado);
             cuerpo.anadir_transicion(&Transicion {
                 fecha: &instante,
diff --git a/crates/tc2/src/plan/plantillas.rs b/crates/tc2/src/plan/plantillas.rs
index e8044dd..36cf667 100644
--- a/crates/tc2/src/plan/plantillas.rs
+++ b/crates/tc2/src/plan/plantillas.rs
@@ -50,6 +50,25 @@ pub struct Datos<'a> {
     pub enunciado: &'a Enunciado,
     /// Complejidad declarada frente a heuristica.
     pub resolucion: &'a Resolucion,
+    /// Agentes de diseno de la seleccion (ids del inventario, F4-6).
+    pub agentes: &'a [String],
+    /// Modelo de diseno de la seleccion (F4-6); `None` si no lo hay.
+    pub modelo: Option<&'a str>,
+}
+
+/// Con que agente y modelo se lanza el rol `rid` (F4-6): el id del
+/// inventario que la seleccion eligio para ese rol (`<plugin>:<rid>`) o, si
+/// no eligio ninguno, el propio `rid`.
+fn lanzar(d: &Datos<'_>, rid: &str) -> String {
+    let agente = d
+        .agentes
+        .iter()
+        .find(|a| a.rsplit(':').next() == Some(rid))
+        .map_or(rid, String::as_str);
+    match d.modelo {
+        Some(m) => format!("lanzalo con el agente `{agente}` y el modelo `{m}`"),
+        None => format!("lanzalo con el agente `{agente}`"),
+    }
 }
 
 /// `peticion-<rol>-<ronda>.md` (el id ya lleva `brainstorm-`).
@@ -230,8 +249,9 @@ pub fn peticion_rol(d: &Datos<'_>, rol: &Rol, otros: &[Rol]) -> String {
         )
     };
     format!(
-        "# Peticion de brainstorm — {id}, rol {titulo} (ronda {ronda})\n\n{cab}- Rol: `{rid}` — lanzalo con el agente de ese mismo nombre\n- Vuelca tu respuesta en: `{salida}` (se publica con `{orden}`)\n\n## Tu pregunta\n\n> {pregunta}\n\n## Que miras\n\n{mira}\n## Que NO miras\n\nNo es una lista de cortesia. Si la ignoras y lo cubres todo \"por si acaso\", el acotado de contexto se deshace sin que se note y esta tarea deja de tener varios puntos de vista:\n\n{no_mira}\n{enun}\n## Como entregas\n\nEscribe en `{salida}`, con estas secciones y en este orden:\n\n{entregables}\n## Reglas\n\n- **Propones enfoque; no implementas.** No escribas codigo de produccion ni modifiques ficheros del repo: tu salida es un documento.\n- Evidencia, no suposicion: si afirmas que algo se comporta de cierta manera, es porque lo has mirado. Di de donde lo sacas.\n- Prefiere lo concreto: rutas, nombres y casos reales por encima de recomendaciones genericas.\n- {companeros}",
+        "# Peticion de brainstorm — {id}, rol {titulo} (ronda {ronda})\n\n{cab}- Rol: `{rid}` — {lanza}\n- Vuelca tu respuesta en: `{salida}` (se publica con `{orden}`)\n\n## Tu pregunta\n\n> {pregunta}\n\n## Que miras\n\n{mira}\n## Que NO miras\n\nNo es una lista de cortesia. Si la ignoras y lo cubres todo \"por si acaso\", el acotado de contexto se deshace sin que se note y esta tarea deja de tener varios puntos de vista:\n\n{no_mira}\n{enun}\n## Como entregas\n\nEscribe en `{salida}`, con estas secciones y en este orden:\n\n{entregables}\n## Reglas\n\n- **Propones enfoque; no implementas.** No escribas codigo de produccion ni modifiques ficheros del repo: tu salida es un documento.\n- Evidencia, no suposicion: si afirmas que algo se comporta de cierta manera, es porque lo has mirado. Di de donde lo sacas.\n- Prefiere lo concreto: rutas, nombres y casos reales por encima de recomendaciones genericas.\n- {companeros}",
         id = d.id,
+        lanza = lanzar(d, rol.id),
         titulo = rol.titulo,
         ronda = d.ronda,
         cab = cabecera(d),
@@ -256,8 +276,9 @@ fn que_trae(seccion_roles: &str) -> String {
 /// el plan.
 pub fn peticion_redaccion(d: &Datos<'_>, rol: &Rol) -> String {
     format!(
-        "# Peticion de redaccion del plan — {id} (ronda {ronda})\n\n{cab}- Rol: `{rid}` — lanzalo con el agente de ese mismo nombre\n- Quien orquesta publica tu respuesta con: `{orden}`\n\n**Esta tarea se planifica con un solo rol y no hay unificador**: tu respuesta ES el plan final. Redactala con las secciones de abajo; quien orquesta la publica tal cual en `plan-final.md` (tu no escribes ficheros), sin salida intermedia que consolidar.\n\n{replan}## Tu pregunta\n\n> {pregunta}\n\n## Que miras\n\n{mira}\n## Que NO miras\n\nQue lo mires solo tu no te autoriza a cubrirlo todo: lo que queda fuera de tu pregunta no se mira, pero se DICE. Anota en el plan lo que no cubres, para que quien lo apruebe sepa que ese lado no se ha revisado:\n\n{no_mira}\n{complejidad}\n{trae}## Reglas\n\n- **Propones enfoque; no implementas.** No escribas codigo de produccion ni modifiques ficheros del repo: tu salida es el plan.\n- Evidencia, no suposicion: si afirmas que algo se comporta de cierta manera, es porque lo has mirado. Di de donde lo sacas.\n\n{enun}",
+        "# Peticion de redaccion del plan — {id} (ronda {ronda})\n\n{cab}- Rol: `{rid}` — {lanza}\n- Quien orquesta publica tu respuesta con: `{orden}`\n\n**Esta tarea se planifica con un solo rol y no hay unificador**: tu respuesta ES el plan final. Redactala con las secciones de abajo; quien orquesta la publica tal cual en `plan-final.md` (tu no escribes ficheros), sin salida intermedia que consolidar.\n\n{replan}## Tu pregunta\n\n> {pregunta}\n\n## Que miras\n\n{mira}\n## Que NO miras\n\nQue lo mires solo tu no te autoriza a cubrirlo todo: lo que queda fuera de tu pregunta no se mira, pero se DICE. Anota en el plan lo que no cubres, para que quien lo apruebe sepa que ese lado no se ha revisado:\n\n{no_mira}\n{complejidad}\n{trae}## Reglas\n\n- **Propones enfoque; no implementas.** No escribas codigo de produccion ni modifiques ficheros del repo: tu salida es el plan.\n- Evidencia, no suposicion: si afirmas que algo se comporta de cierta manera, es porque lo has mirado. Di de donde lo sacas.\n\n{enun}",
         id = d.id,
+        lanza = lanzar(d, rol.id),
         ronda = d.ronda,
         cab = cabecera(d),
         rid = rol.id,
@@ -302,11 +323,12 @@ pub fn peticion_unificador(d: &Datos<'_>, roles: &[Rol]) -> String {
         _ => "- Los desacuerdos entre roles y como se resuelve cada uno.\n",
     };
     format!(
-        "# Peticion al unificador — {id} (ronda {ronda})\n\n{cab}- Rol: `{UNIFICADOR}` — lanzalo con el agente de ese mismo nombre\n- Vuelca el resultado con: `{orden}`\n\n{replan}## Salidas que tienes que consolidar\n\n{salidas}\n{complejidad}\n## Como consolidas\n\n{como}\n{trae}{enun}",
+        "# Peticion al unificador — {id} (ronda {ronda})\n\n{cab}- Rol: `{UNIFICADOR}` — {lanza}\n- Vuelca el resultado con: `{orden}`\n\n{replan}## Salidas que tienes que consolidar\n\n{salidas}\n{complejidad}\n## Como consolidas\n\n{como}\n{trae}{enun}",
         id = d.id,
         ronda = d.ronda,
         cab = cabecera(d),
         orden = guardar(d.id, PLAN_FINAL),
+        lanza = lanzar(d, UNIFICADOR),
         replan = replanificacion(d),
         complejidad = bloque_complejidad(d, roles),
         trae = que_trae(seccion),
@@ -362,6 +384,8 @@ mod tests {
             fecha: "2026-11-18",
             enunciado: e,
             resolucion: r,
+            agentes: &[],
+            modelo: None,
         }
     }
 
diff --git a/crates/tc2/src/revision/mod.rs b/crates/tc2/src/revision/mod.rs
index a5dd22b..8b2a486 100644
--- a/crates/tc2/src/revision/mod.rs
+++ b/crates/tc2/src/revision/mod.rs
@@ -332,6 +332,9 @@ fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
         agente: sel.agentes.first().cloned(),
         modelo: sel.modelo.clone(),
         skills: sel.revisoras.clone(),
+        reglas: sel.reglas.clone(),
+        senales: sel.senales.clone(),
+        avisos: sel.avisos.clone(),
     });
     let tr = estado::transaccion(&ctx, &op);
     match tr.resultado {
@@ -497,7 +500,22 @@ pub fn seleccion_de(
         modelos_permitidos: &cfg.seleccion.modelos_permitidos,
         ronda: 1,
     };
-    let mut s = seleccion::revision(&rutas, &f.catalogo, &f.inventario, &ctx).unwrap_or_default();
+    // F4-6: con los dominios (texto vivo, rutas del diff y lo congelado).
+    let doms = seleccion::orden::dominios_de(Some((t, cuerpo)), &rutas, &cfg);
+    let entrada = seleccion::Entrada {
+        rutas: &rutas,
+        dominios: &doms,
+        etiquetas: &t.etiquetas,
+        roles_base: 0,
+    };
+    let mut s = seleccion::seleccionar(
+        seleccion::Rol::Revision,
+        &entrada,
+        &f.catalogo,
+        &f.inventario,
+        &ctx,
+    )
+    .unwrap_or_default();
     avisos.append(&mut s.avisos);
     s.avisos = avisos;
     s
diff --git a/crates/tc2/src/revision/operacion.rs b/crates/tc2/src/revision/operacion.rs
index 978c8ad..45f52e3 100644
--- a/crates/tc2/src/revision/operacion.rs
+++ b/crates/tc2/src/revision/operacion.rs
@@ -44,6 +44,12 @@ pub struct AnotacionRevision {
     pub modelo: String,
     /// Revisoras.
     pub skills: Vec<String>,
+    /// Filas aplicadas, senales de dominio y avisos (F4-6).
+    pub reglas: Vec<String>,
+    /// Senales de dominio.
+    pub senales: Vec<String>,
+    /// Avisos.
+    pub avisos: Vec<String>,
 }
 
 /// Pasar la tarea `id` a revision.
@@ -89,6 +95,9 @@ impl Revisar {
         rv.agente.clone_from(&s.agente);
         rv.modelo = Some(s.modelo.clone());
         rv.skills.clone_from(&s.skills);
+        rv.reglas.clone_from(&s.reglas);
+        rv.senales.clone_from(&s.senales);
+        rv.avisos.clone_from(&s.avisos);
         Some(rv)
     }
 
@@ -264,6 +273,9 @@ mod tests {
             agente: Some("general-purpose".to_owned()),
             modelo: "opus".to_owned(),
             skills: skills.iter().map(|s| (*s).to_owned()).collect(),
+            reglas: Vec::new(),
+            senales: Vec::new(),
+            avisos: Vec::new(),
         };
         let antes = md("en-curso", ana().asignado().into());
         let op = Revisar::nueva("TASK-001", &ana(), AHORA).con_seleccion(sel(&["p:rust"]));
diff --git a/crates/tc2/src/seleccion/catalogo.rs b/crates/tc2/src/seleccion/catalogo.rs
index 827facd..53608c5 100644
--- a/crates/tc2/src/seleccion/catalogo.rs
+++ b/crates/tc2/src/seleccion/catalogo.rs
@@ -1,6 +1,9 @@
-//! Catalogo de revisoras (F1-15 decisiones 2 y 8): los metadatos de seleccion
-//! que cada skill declara en su bloque ```` ```yaml ```` (`rol: revisor`,
-//! `patrones_archivo`, y en la generica `fallback` y `umbral_dominios`).
+//! Catalogo de seleccion (F1-15 decisiones 2 y 8): los metadatos que cada
+//! skill declara en su bloque ```` ```yaml ```` (`rol` ∈ {`revisor`,
+//! `diseno`, `ejecucion`}, `etiquetas`, `dominios`, `prioridad`; en las
+//! revisoras `patrones_archivo`, y en la generica `fallback` y
+//! `umbral_dominios`) y, para las ajenas, los que declara `recursos:` de la
+//! configuracion ([`de_config`], F4-6).
 //!
 //! Se apoya en [`crate::inventario`] sin cambiarlo: el inventario dice que
 //! existe; el catalogo lee, de las mismas carpetas, que declara cada skill.
@@ -26,11 +29,23 @@ use crate::contrato::Informe;
 use crate::git::{Git, Subcomando};
 use crate::inventario::{self, Entrada, Inventario, MAX_CABECERA, Origen, Tipo};
 
-/// Una revisora del catalogo.
+/// Roles que puede declarar una entrada del catalogo.
+pub const ROLES: [&str; 3] = ["revisor", "diseno", "ejecucion"];
+
+/// Una entrada del catalogo (una revisora o una skill de diseno o de
+/// ejecucion).
 #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
 pub struct Revisora {
     /// Id del inventario (`plugin:n` en un plugin).
     pub id: String,
+    /// `revisor`, `diseno` o `ejecucion`.
+    pub rol: String,
+    /// Etiquetas (solape de S07).
+    pub etiquetas: Vec<String>,
+    /// Dominios de F1-15 que cubre (S06, S07, S10).
+    pub dominios: Vec<String>,
+    /// Prioridad (desempate de S07; mayor primero).
+    pub prioridad: u32,
     /// Globs de `patrones_archivo`.
     pub patrones: Vec<String>,
     /// `fallback: true` (la generica): no compite por patrones.
@@ -41,19 +56,22 @@ pub struct Revisora {
     pub ruta: String,
 }
 
-/// Las revisoras del catalogo, ordenadas por id y sin repetir.
+/// Las entradas del catalogo, ordenadas por id y sin repetir: las
+/// revisoras y las demas.
 #[derive(Debug, Clone, Default, PartialEq, Eq)]
 pub struct Catalogo {
     revisoras: Vec<Revisora>,
+    otros: Vec<Revisora>,
 }
 
 impl Catalogo {
-    /// Catalogo de `revisoras` en cualquier orden: el resultado no depende
+    /// Catalogo de `entradas` en cualquier orden: el resultado no depende
     /// del orden (se queda la primera de cada id tras ordenar).
-    pub fn nuevo(mut revisoras: Vec<Revisora>) -> Self {
-        revisoras.sort();
-        revisoras.dedup_by(|b, a| a.id == b.id);
-        Self { revisoras }
+    pub fn nuevo(mut entradas: Vec<Revisora>) -> Self {
+        entradas.sort();
+        entradas.dedup_by(|b, a| a.id == b.id);
+        let (revisoras, otros) = entradas.into_iter().partition(|r| r.rol == "revisor");
+        Self { revisoras, otros }
     }
 
     /// Las revisoras, por id.
@@ -61,6 +79,18 @@ impl Catalogo {
         &self.revisoras
     }
 
+    /// Las skills de diseno y de ejecucion, por id.
+    pub fn otros(&self) -> &[Revisora] {
+        &self.otros
+    }
+
+    /// Huella del catalogo (12 hex del sha256 de su forma canonica, como
+    /// [`Inventario::huella`]).
+    pub fn huella(&self) -> String {
+        let s = format!("{:?}{:?}", self.revisoras, self.otros);
+        crate::contrato::corto(&distribucion::sha256::hex(s.as_bytes()))
+    }
+
     /// La revisora de id `id`.
     pub fn revisora(&self, id: &str) -> Option<&Revisora> {
         self.revisoras.iter().find(|r| r.id == id)
@@ -76,6 +106,12 @@ struct Bloque {
     #[serde(default)]
     fallback: bool,
     umbral_dominios: Option<u32>,
+    #[serde(default)]
+    etiquetas: Vec<String>,
+    #[serde(default)]
+    dominios: Vec<String>,
+    #[serde(default)]
+    prioridad: u32,
 }
 
 /// El primer bloque ```` ```yaml ```` de `cabecera` (el texto entre la
@@ -99,6 +135,13 @@ pub fn bloque_yaml(cabecera: &str) -> Option<&str> {
 /// yaml, no es `rol: revisor` o el bloque no se entiende (lo instalado sin
 /// metadatos no se elige, decision 2).
 pub fn revisora_de(id: &str, texto: &str, ruta: &str) -> Option<Revisora> {
+    recurso_de(id, texto, ruta).filter(|r| r.rol == "revisor")
+}
+
+/// La entrada que declara el texto de una skill, con cualquiera de los
+/// [`ROLES`]: `None` si no tiene bloque yaml, su `rol` no es de la lista o
+/// el bloque no se entiende.
+pub fn recurso_de(id: &str, texto: &str, ruta: &str) -> Option<Revisora> {
     let mut corte = texto
         .len()
         .min(usize::try_from(MAX_CABECERA).unwrap_or(usize::MAX));
@@ -109,8 +152,13 @@ pub fn revisora_de(id: &str, texto: &str, ruta: &str) -> Option<Revisora> {
     let b: Bloque =
         serde_saphyr::from_str_with_options(bloque_yaml(cabecera)?, crate::yaml::opciones())
             .ok()?;
-    (b.rol.as_deref() == Some("revisor")).then(|| Revisora {
+    let rol = b.rol.filter(|r| ROLES.contains(&r.as_str()))?;
+    Some(Revisora {
         id: id.to_owned(),
+        rol,
+        etiquetas: b.etiquetas,
+        dominios: b.dominios,
+        prioridad: b.prioridad,
         patrones: b.patrones_archivo,
         fallback: b.fallback,
         umbral_dominios: b.umbral_dominios,
@@ -118,15 +166,38 @@ pub fn revisora_de(id: &str, texto: &str, ruta: &str) -> Option<Revisora> {
     })
 }
 
-/// Lo leido de una fuente: las entradas que existen y las revisoras.
+/// Lo leido de una fuente: las entradas que existen y las que declaran
+/// metadatos de seleccion.
 #[derive(Debug, Clone, Default)]
 pub struct Lectura {
     /// Skills encontradas (para el inventario).
     pub entradas: Vec<Entrada>,
-    /// Revisoras con metadatos.
+    /// Entradas del catalogo (revisoras y skills de diseno o ejecucion).
     pub revisoras: Vec<Revisora>,
 }
 
+/// Las entradas que declara `recursos:` de la configuracion (F1-14, F4-6):
+/// solo las que dicen un `rol` de [`ROLES`]; las demas no se eligen.
+pub fn de_config(recursos: &[crate::config::Recurso]) -> Vec<Revisora> {
+    recursos
+        .iter()
+        .filter_map(|r| {
+            let rol = r.rol.clone().filter(|x| ROLES.contains(&x.as_str()))?;
+            Some(Revisora {
+                id: r.id.clone(),
+                rol,
+                etiquetas: r.etiquetas.clone(),
+                dominios: r.dominios.clone(),
+                prioridad: r.prioridad.unwrap_or(0),
+                patrones: Vec::new(),
+                fallback: false,
+                umbral_dominios: None,
+                ruta: ".taskcode/config.yml".to_owned(),
+            })
+        })
+        .collect()
+}
+
 /// Lee como mucho [`MAX_CABECERA`] bytes de `p`.
 fn cabecera(p: &Path) -> Option<String> {
     let mut b = Vec::new();
@@ -161,7 +232,7 @@ pub fn de_carpeta(skills: &Path, prefijo: &str, origen: &Origen) -> Lectura {
             id: id.clone(),
             origen: origen.clone(),
         });
-        if let Some(r) = revisora_de(&id, &texto, &f.display().to_string()) {
+        if let Some(r) = recurso_de(&id, &texto, &f.display().to_string()) {
             l.revisoras.push(r);
         }
     }
@@ -250,7 +321,7 @@ pub fn de_base(git: &Git, dir: &Path, base: &str) -> Result<Lectura, Informe> {
             id: nombre.clone(),
             origen: Origen::Proyecto,
         });
-        if let Some(r) = revisora_de(&nombre, &texto, &format!("{base}:{ruta}")) {
+        if let Some(r) = recurso_de(&nombre, &texto, &format!("{base}:{ruta}")) {
             l.revisoras.push(r);
         }
     }
@@ -270,6 +341,22 @@ mod tests {
         assert!(revisora_de("p:x", "---\nname: x\n---\n", "x").is_none());
         assert!(revisora_de("p:x", "```yaml\nrol: revisor\n", "x").is_none());
         assert!(revisora_de("p:x", "```yaml\nrol: ejecucion\n```\n", "x").is_none());
+        // F4-6: una skill de ejecucion entra en el catalogo con sus metadatos.
+        let e = recurso_de(
+            "p:x",
+            "```yaml\nrol: ejecucion\netiquetas: [a, b]\ndominios: [rendimiento]\nprioridad: 10\n```\n",
+            "x",
+        );
+        assert_eq!(
+            e.map(|e| (e.rol, e.etiquetas, e.dominios, e.prioridad)),
+            Some((
+                "ejecucion".to_owned(),
+                vec!["a".to_owned(), "b".to_owned()],
+                vec!["rendimiento".to_owned()],
+                10
+            ))
+        );
+        assert!(recurso_de("p:x", "```yaml\nrol: otro\n```\n", "x").is_none());
         assert!(revisora_de("p:x", "```yaml\nrol: [\n```\n", "x").is_none());
         // Un bloque mas alla de la cabecera no cuenta.
         let lejos = format!(
@@ -283,6 +370,10 @@ mod tests {
     fn el_catalogo_no_depende_del_orden() {
         let r = |id: &str| Revisora {
             id: id.to_owned(),
+            rol: "revisor".to_owned(),
+            etiquetas: Vec::new(),
+            dominios: Vec::new(),
+            prioridad: 0,
             patrones: Vec::new(),
             fallback: false,
             umbral_dominios: None,
diff --git a/crates/tc2/src/seleccion/dominios.rs b/crates/tc2/src/seleccion/dominios.rs
new file mode 100644
index 0000000..9afd736
--- /dev/null
+++ b/crates/tc2/src/seleccion/dominios.rs
@@ -0,0 +1,229 @@
+//! Detector de dominio de F1-15 (decision 3; F4-6): funcion pura sobre el
+//! texto de la tarea, sus etiquetas y las rutas del cambio, con la
+//! `tabla-dominios` del ADR ([`super::tabla::dominios`]) y lo que la
+//! configuracion anade (`seleccion.dominios_extra`, que solo anade senales a
+//! un dominio de la lista cerrada).
+//!
+//! - Etiqueta: igualdad, plegada (sin tildes ni mayusculas, como la PoC T).
+//! - Palabra: al principio de una palabra del objetivo o de los criterios,
+//!   plegados con [`crate::tarea::texto::plegar`].
+//! - Ruta: glob sin distinguir mayusculas sobre cada ruta (los dos nombres
+//!   de cada renombrado llegan como dos rutas).
+//!
+//! El texto de la tarea no es confiable y solo sube: [`union`] suma las
+//! senales congeladas con el plan aprobado (`roles.diseno.senales`) a las
+//! del texto vivo, de modo que reescribir la tarea no quita un dominio.
+
+use std::collections::{BTreeMap, BTreeSet};
+
+use super::tabla::{DOMINIOS, FilaDominio, Senal};
+use crate::config::Senales;
+use crate::tarea::texto::plegar;
+
+/// Lo que se mira de una tarea.
+#[derive(Debug, Clone, Copy)]
+pub struct Texto<'a> {
+    /// Objetivo.
+    pub objetivo: &'a str,
+    /// Criterios de aceptacion.
+    pub criterios: &'a [String],
+    /// Etiquetas.
+    pub etiquetas: &'a [String],
+}
+
+/// Dominios detectados, en el orden de la lista cerrada, cada uno con sus
+/// senales `<senal>:<valor>` ordenadas y sin repetir.
+pub type Dominios = Vec<(String, Vec<String>)>;
+
+fn casa_palabra(texto: &str, valor: &str) -> bool {
+    let v = plegar(valor);
+    !v.is_empty() && format!(" {texto}").contains(&format!(" {v}"))
+}
+
+/// Los dominios de `t` y `rutas` con `tabla` y `extra`.
+pub fn dominios(
+    t: &Texto<'_>,
+    rutas: &[String],
+    tabla: &[FilaDominio],
+    extra: &BTreeMap<String, Senales>,
+) -> Dominios {
+    let texto = plegar(&format!("{}\n{}", t.objetivo, t.criterios.join("\n")));
+    let etiquetas: BTreeSet<String> = t.etiquetas.iter().map(|e| plegar(e)).collect();
+    let mut filas: Vec<(String, Senal, String)> = tabla
+        .iter()
+        .map(|f| (f.dominio.clone(), f.senal, f.valor.clone()))
+        .collect();
+    for (d, s) in extra {
+        for (lista, senal) in [
+            (&s.etiquetas, Senal::Etiqueta),
+            (&s.palabras, Senal::Palabra),
+            (&s.rutas, Senal::Ruta),
+        ] {
+            filas.extend(lista.iter().map(|v| (d.clone(), senal, v.clone())));
+        }
+    }
+    let mut vistos: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
+    // Un dominio fuera de la lista (no lo admite la configuracion) se cae
+    // en [`ordenar`].
+    for (d, senal, valor) in filas {
+        let ok = match senal {
+            Senal::Etiqueta => etiquetas.contains(&plegar(&valor)),
+            Senal::Palabra => casa_palabra(&texto, &valor),
+            Senal::Ruta => {
+                let g = super::globs(std::slice::from_ref(&valor));
+                rutas.iter().any(|r| g.is_match(super::normalizar(r)))
+            }
+        };
+        if ok {
+            vistos
+                .entry(d)
+                .or_default()
+                .insert(format!("{}:{valor}", senal.nombre()));
+        }
+    }
+    ordenar(vistos)
+}
+
+fn ordenar(vistos: BTreeMap<String, BTreeSet<String>>) -> Dominios {
+    DOMINIOS
+        .iter()
+        .filter_map(|d| {
+            vistos
+                .get(*d)
+                .map(|s| ((*d).to_owned(), s.iter().cloned().collect()))
+        })
+        .collect()
+}
+
+/// Las senales como se anotan en `roles.<rol>.senales`:
+/// `<dominio>:<senal>:<valor>`.
+pub fn anotar(d: &Dominios) -> Vec<String> {
+    d.iter()
+        .flat_map(|(dom, ss)| ss.iter().map(move |s| format!("{dom}:{s}")))
+        .collect()
+}
+
+/// `d` mas las senales `congeladas` (`<dominio>:<senal>:<valor>`): solo
+/// anade. Una senal congelada de un dominio fuera de la lista no cuenta.
+pub fn union(d: &Dominios, congeladas: &[String]) -> Dominios {
+    let mut vistos: BTreeMap<String, BTreeSet<String>> = d
+        .iter()
+        .map(|(k, v)| (k.clone(), v.iter().cloned().collect()))
+        .collect();
+    for c in congeladas {
+        if let Some((dom, s)) = c.split_once(':')
+            && DOMINIOS.contains(&dom)
+            && !s.is_empty()
+        {
+            vistos
+                .entry(dom.to_owned())
+                .or_default()
+                .insert(s.to_owned());
+        }
+    }
+    ordenar(vistos)
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::seleccion::tabla;
+
+    fn tabla_adr() -> Vec<FilaDominio> {
+        tabla::dominios(tabla::ADR).unwrap_or_default()
+    }
+
+    fn v(xs: &[&str]) -> Vec<String> {
+        xs.iter().map(|x| (*x).to_owned()).collect()
+    }
+
+    fn det(obj: &str, etq: &[&str], rutas: &[&str]) -> Dominios {
+        let criterios = v(&["criterio"]);
+        let etiquetas = v(etq);
+        let t = Texto {
+            objetivo: obj,
+            criterios: &criterios,
+            etiquetas: &etiquetas,
+        };
+        dominios(&t, &v(rutas), &tabla_adr(), &BTreeMap::new())
+    }
+
+    /// Criterio 3 (tabla adversaria): tildes y mayusculas, palabra a mitad
+    /// de otra, etiqueta, los dos nombres de un renombrado,
+    /// `.Claude/settings.json`, `dominios_extra` que solo anade y la union
+    /// con las senales congeladas que no deja quitar un dominio.
+    #[test]
+    fn tabla_adversaria() {
+        let p = |s: &[&str]| vec![("puertas-seguridad".to_owned(), v(s))];
+        // Tildes y mayusculas: «APROBACIÓN» casa con `aprobacion`.
+        assert_eq!(
+            det("Pide la APROBACIÓN", &[], &[]),
+            p(&["palabra:aprobacion"])
+        );
+        // Al principio de palabra: «checks» si, «recheck» y «matoken» no.
+        assert_eq!(det("los checks", &[], &[]), p(&["palabra:check"]));
+        assert!(det("un recheck y un matoken", &[], &[]).is_empty());
+        // Etiqueta exacta (plegada); una parecida no.
+        assert_eq!(det("nada", &["Seguridad"], &[]), p(&["etiqueta:seguridad"]));
+        assert!(det("nada", &["seguridades"], &[]).is_empty());
+        // Rendimiento por etiqueta.
+        assert_eq!(
+            det("nada", &["wasm"], &[]),
+            vec![("rendimiento".to_owned(), v(&["etiqueta:wasm"]))]
+        );
+        // Renombrado: basta con uno de los dos nombres.
+        assert_eq!(
+            det("nada", &[], &["src/a.rs", ".github/workflows/x.yml"]),
+            p(&["ruta:.github/**"])
+        );
+        // Sin distinguir mayusculas ni barras de Windows.
+        assert_eq!(
+            det("nada", &[], &[".Claude\\settings.json"]),
+            p(&["ruta:.claude/**"])
+        );
+        assert_eq!(
+            det("nada", &[], &["docs/CODEOWNERS"]),
+            p(&["ruta:**/CODEOWNERS"])
+        );
+        assert!(det("nada", &[], &["src/claude/x.rs"]).is_empty());
+        // `dominios_extra` solo anade (un dominio nuevo se ignora).
+        let extra = BTreeMap::from([
+            (
+                "puertas-seguridad".to_owned(),
+                Senales {
+                    etiquetas: Vec::new(),
+                    palabras: v(&["credencial"]),
+                    rutas: v(&["src/auth/**"]),
+                },
+            ),
+            (
+                "datos".to_owned(),
+                Senales {
+                    etiquetas: Vec::new(),
+                    palabras: v(&["tabla"]),
+                    rutas: Vec::new(),
+                },
+            ),
+        ]);
+        let criterios = v(&["la tabla"]);
+        let t = Texto {
+            objetivo: "Una credencial nueva",
+            criterios: &criterios,
+            etiquetas: &[],
+        };
+        let d = dominios(&t, &v(&["src/auth/x.rs"]), &tabla_adr(), &extra);
+        assert_eq!(d, p(&["palabra:credencial", "ruta:src/auth/**"]));
+        // Union: el texto vivo ya no dice nada y lo congelado sigue.
+        let vivo = det("otro vocabulario", &[], &[]);
+        assert!(vivo.is_empty());
+        let congeladas = v(&["puertas-seguridad:palabra:check", "datos:palabra:tabla"]);
+        assert_eq!(union(&vivo, &congeladas), p(&["palabra:check"]));
+        assert_eq!(
+            anotar(&union(&det("token", &[], &[]), &congeladas)),
+            v(&[
+                "puertas-seguridad:palabra:check",
+                "puertas-seguridad:palabra:token"
+            ])
+        );
+    }
+}
diff --git a/crates/tc2/src/seleccion/mod.rs b/crates/tc2/src/seleccion/mod.rs
index c939493..f17618f 100644
--- a/crates/tc2/src/seleccion/mod.rs
+++ b/crates/tc2/src/seleccion/mod.rs
@@ -1,21 +1,20 @@
-//! Seleccion determinista de revisoras y modelo (F6-3; ADR F1-15 decision 1,
-//! adenda F6-3).
+//! Seleccion determinista de agentes, skills y modelo por rol (F6-3 y F4-6;
+//! ADR F1-15 decision 1).
 //!
-//! [`revision`] es una funcion pura: recibe las rutas del diff (los dos
-//! nombres de cada renombrado), el catalogo y el inventario de la base, el
-//! carril efectivo, la complejidad y los modelos permitidos, y devuelve las
-//! revisoras, el lanzador, el modelo, las filas que se aplicaron y los
-//! avisos. No lee ficheros, no llama a la red ni a un modelo: recorre las
-//! filas de `revision` y `todos` de `tabla-seleccion` en su orden ([`tabla`])
-//! y la columna «Revisión» de `tabla-modelo`.
-//!
-//! Lo que F6-3 no evalua todavia (adenda F6-3 de F1-15): sin el detector
-//! `dominios()` de F4-6 la lista de dominios es vacia, asi que S10 y S11 no
-//! se cumplen nunca. La ronda (F6-6) la da `--ronda` de `tc2 revisoras`
-//! (en CI, la de `tc2 hallazgos ronda`); sin ella es 1 y S13 no se cumple.
-//! Las filas se leen igual y su vocabulario se valida.
+//! [`seleccionar`] es una funcion pura: recibe el rol (diseno,
+//! implementacion o revision), las rutas del cambio (los dos nombres de cada
+//! renombrado), los dominios detectados ([`dominios`]), las etiquetas, el
+//! numero de roles de diseno por carril y complejidad, el catalogo y el
+//! inventario, el carril efectivo, la complejidad y los modelos permitidos,
+//! y devuelve agentes, skills (y revisoras), modelo, filas aplicadas,
+//! senales y avisos. No lee ficheros, no llama a la red ni a un modelo:
+//! recorre las filas de su rol y de `todos` de `tabla-seleccion` en su orden
+//! ([`tabla`]) y la columna de su rol de `tabla-modelo`. [`revision`] es la
+//! envoltura de F6-3 (sin dominios). La ronda (F6-6) la da `--ronda` de
+//! `tc2 revisoras` (en CI, la de `tc2 hallazgos ronda`); sin ella es 1.
 
 pub mod catalogo;
+pub mod dominios;
 pub mod orden;
 pub mod tabla;
 
@@ -47,19 +46,72 @@ pub struct Contexto<'a> {
     pub ronda: u32,
 }
 
-/// La seleccion de revision.
+/// Rol para el que se selecciona.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Rol {
+    /// Diseno (`plan`).
+    Diseno,
+    /// Implementacion (`plan`).
+    Implementacion,
+    /// Revision (`review` y el revisor en CI).
+    Revision,
+}
+
+impl Rol {
+    /// Nombre en la tabla y en `roles.<rol>`.
+    pub fn nombre(self) -> &'static str {
+        match self {
+            Self::Diseno => "diseno",
+            Self::Implementacion => "implementacion",
+            Self::Revision => "revision",
+        }
+    }
+
+    fn fila(self) -> RolFila {
+        match self {
+            Self::Diseno => RolFila::Diseno,
+            Self::Implementacion => RolFila::Implementacion,
+            Self::Revision => RolFila::Revision,
+        }
+    }
+}
+
+/// Lo que se sabe de la tarea ademas del carril y la complejidad.
+#[derive(Debug, Clone, Copy)]
+pub struct Entrada<'a> {
+    /// Rutas del cambio (en `plan`, vacias).
+    pub rutas: &'a [String],
+    /// Dominios detectados ([`dominios::dominios`], con la union de lo
+    /// congelado si lo hay).
+    pub dominios: &'a dominios::Dominios,
+    /// Etiquetas de la tarea (solape de S07).
+    pub etiquetas: &'a [String],
+    /// Roles de diseno por carril y complejidad
+    /// ([`Carril::roles_diseno`]): el numero de S02.
+    pub roles_base: u8,
+}
+
+/// La seleccion de un rol.
 #[derive(Debug, Clone, Default, PartialEq, Eq)]
 pub struct Seleccion {
-    /// Revisoras (skills), por id.
+    /// Revisoras (skills), por id (revision).
     pub revisoras: Vec<String>,
-    /// Agentes que lanzan cada revisora (S12).
+    /// Agentes: los que lanzan cada revisora (S12) o los de diseno (S02 a
+    /// S05), sin los no instalados (S14).
     pub agentes: Vec<String>,
-    /// Modelo de la escala (`haiku`, `sonnet` u `opus`).
+    /// Agentes de diseno elegidos por las filas, en su orden y aunque no
+    /// esten instalados (la peticion se escribe igual, con el aviso).
+    pub roles: Vec<String>,
+    /// Skills de diseno (S06) o de ejecucion (S07).
+    pub skills: Vec<String>,
+    /// Modelo de la escala (`haiku`, `sonnet` u `opus`); vacio sin plan.
     pub modelo: String,
     /// Ids de las filas que se aplicaron (la de `tabla-modelo` primero).
     pub reglas: Vec<String>,
     /// Avisos (`no-instalado:<id>`, `complejidad-desconocida`...).
     pub avisos: Vec<String>,
+    /// Senales de dominio (`<dominio>:<senal>:<valor>`).
+    pub senales: Vec<String>,
     /// Rutas que recibe cada revisora (la de dominio, las que casan con sus
     /// patrones; la generica, las sueltas o todas).
     pub rutas: BTreeMap<String, Vec<String>>,
@@ -135,15 +187,28 @@ fn casar(rutas: &[String], cat: &Catalogo) -> Casan {
     }
 }
 
-/// Modelo de revision de `tabla-modelo` para el carril y la complejidad.
-fn fila_modelo(filas: &[tabla::FilaModelo], carril: Carril, cx: &str) -> Option<(String, String)> {
+/// Modelo de `rol` en `tabla-modelo` para el carril y la complejidad
+/// (vacio en diseno de `errata`, «—»).
+fn fila_modelo(
+    filas: &[tabla::FilaModelo],
+    rol: Rol,
+    carril: Carril,
+    cx: &str,
+) -> Option<(String, String)> {
     filas
         .iter()
         .find(|f| {
             f.carril == carril.nombre()
                 && (f.complejidades.is_empty() || f.complejidades.iter().any(|x| x == cx))
         })
-        .map(|f| (f.id.clone(), f.revision.clone()))
+        .map(|f| {
+            let m = match rol {
+                Rol::Diseno => f.diseno.clone().unwrap_or_default(),
+                Rol::Implementacion => f.implementacion.clone(),
+                Rol::Revision => f.revision.clone(),
+            };
+            (f.id.clone(), m)
+        })
 }
 
 /// El permitido mas cercano por debajo de `modelo`; si no hay, por encima.
@@ -169,7 +234,17 @@ struct Estado<'a> {
     umbral: u32,
     generica: Option<String>,
     inv: &'a Inventario,
+    cat: &'a Catalogo,
     ctx: &'a Contexto<'a>,
+    entrada: &'a Entrada<'a>,
+    /// Dominio que se evalua en una fila por dominio (S10, S11).
+    dom: Option<String>,
+    /// Diseno: roles de la escalera (S02, S03), ids de la escalera (los de
+    /// S02), roles anadidos (S04, S05) y si hay plan (S01).
+    n: usize,
+    escalera: Vec<String>,
+    extra: Vec<String>,
+    plan: bool,
 }
 
 impl Estado<'_> {
@@ -177,21 +252,105 @@ impl Estado<'_> {
         self.inv.contiene(Tipo::Skill, id) || self.inv.contiene(Tipo::Agente, id)
     }
 
+    fn hay_dominio(&self, d: &str) -> bool {
+        self.entrada.dominios.iter().any(|(x, _)| x == d)
+    }
+
+    /// Revisoras declaradas para el dominio `d`, por id.
+    fn revisoras_de(&self, d: &str) -> Vec<String> {
+        self.cat
+            .revisoras()
+            .iter()
+            .filter(|r| r.dominios.iter().any(|x| x == d))
+            .map(|r| r.id.clone())
+            .collect()
+    }
+
+    /// Skills `rol: diseno` de algun dominio detectado, por id.
+    fn skills_diseno(&self) -> Vec<String> {
+        self.cat
+            .otros()
+            .iter()
+            .filter(|r| r.rol == "diseno" && r.dominios.iter().any(|d| self.hay_dominio(d)))
+            .map(|r| r.id.clone())
+            .collect()
+    }
+
+    /// Candidatas de ejecucion con solape de etiquetas o dominios, en orden:
+    /// mayor solape, mayor prioridad, menor id.
+    fn ejecucion(&self) -> Vec<(usize, u32, String)> {
+        let etq: Vec<String> = self
+            .entrada
+            .etiquetas
+            .iter()
+            .map(|e| crate::tarea::texto::plegar(e))
+            .collect();
+        let mut v: Vec<(usize, u32, String)> = self
+            .cat
+            .otros()
+            .iter()
+            .filter(|r| r.rol == "ejecucion")
+            .filter_map(|r| {
+                let s = r
+                    .etiquetas
+                    .iter()
+                    .filter(|e| etq.contains(&crate::tarea::texto::plegar(e)))
+                    .count()
+                    + r.dominios.iter().filter(|d| self.hay_dominio(d)).count();
+                (s > 0).then(|| (s, r.prioridad, r.id.clone()))
+            })
+            .collect();
+        v.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)).then(a.2.cmp(&b.2)));
+        v
+    }
+
+    /// Roles de diseno elegidos hasta ahora (escalera y anadidos).
+    fn roles_diseno(&self) -> Vec<String> {
+        if !self.plan {
+            return Vec::new();
+        }
+        self.escalera
+            .iter()
+            .take(self.n)
+            .chain(&self.extra)
+            .cloned()
+            .collect()
+    }
+
+    /// Diseno: los agentes de los roles elegidos pasan a la seleccion (antes
+    /// de las filas de `todos`, que miran lo instalado).
+    fn materializar(&mut self) {
+        if self.sel.roles.is_empty() {
+            self.sel.roles = self.roles_diseno();
+            self.sel.agentes.clone_from(&self.sel.roles);
+        }
+    }
+
     fn atomo(&self, texto: &str) -> bool {
         let n = u32::try_from(self.casan.por_revisora.len()).unwrap_or(u32::MAX);
-        let elegidos = || self.sel.revisoras.iter().chain(&self.sel.agentes);
+        let elegidos = || {
+            self.sel
+                .revisoras
+                .iter()
+                .chain(&self.sel.agentes)
+                .chain(&self.sel.skills)
+        };
         match texto {
             "siempre" => true,
             "rutas-casan-entre-1-y-umbral" => (1..=self.umbral).contains(&n),
             "rutas-no-casan-o-pasan-umbral" => n == 0 || n > self.umbral,
             "hay-no-instalado" => elegidos().any(|id| !self.instalado(id)),
             "modelo-no-permitido" => !self.ctx.modelos_permitidos.contains(&self.sel.modelo),
-            // Sin detector de dominios (F4-6) ni skills de diseno o ejecucion
-            // en la revision: no se cumplen.
-            "dominio-con-revisora"
-            | "dominio-sin-revisora"
-            | "hay-skill-diseno-del-dominio"
-            | "hay-skill-ejecucion-con-solape" => false,
+            "dominio-con-revisora" => self
+                .dom
+                .as_deref()
+                .is_some_and(|d| !self.revisoras_de(d).is_empty()),
+            "dominio-sin-revisora" => self
+                .dom
+                .as_deref()
+                .is_some_and(|d| self.revisoras_de(d).is_empty()),
+            "hay-skill-diseno-del-dominio" => !self.skills_diseno().is_empty(),
+            "hay-skill-ejecucion-con-solape" => !self.ejecucion().is_empty(),
             t => {
                 if let Some(c) = t.strip_prefix("carril!=") {
                     c != self.ctx.carril.nombre()
@@ -199,8 +358,12 @@ impl Estado<'_> {
                     c == self.ctx.carril.nombre()
                 } else if let Some(r) = t.strip_prefix("ronda>=") {
                     r.parse().is_ok_and(|r: u32| self.ctx.ronda >= r)
+                } else if let Some(d) = t.strip_prefix("dominio=") {
+                    self.hay_dominio(d)
+                } else if let Some(r) = t.strip_prefix("roles>=") {
+                    r.parse()
+                        .is_ok_and(|r: usize| self.roles_diseno().len() >= r)
                 } else {
-                    // `dominio=` (lista vacia) y `roles>=` (solo diseno).
                     false
                 }
             }
@@ -209,8 +372,53 @@ impl Estado<'_> {
 
     /// Aplica la accion de `f`; `true` si cambio algo.
     fn aplicar(&mut self, f: &FilaSeleccion) -> bool {
-        let antes = self.sel.clone();
+        let antes = (self.sel.clone(), self.n, self.extra.clone(), self.plan);
         match f.accion.as_str() {
+            "sin-plan" => self.plan = false,
+            "roles=max(roles_min,complejidad)" => {
+                f.ids.clone_into(&mut self.escalera);
+                self.n = usize::from(self.entrada.roles_base);
+                // La fila fija el numero aunque coincida con el de partida.
+                return true;
+            }
+            "skills-diseno-del-dominio" => {
+                let v = self.skills_diseno();
+                self.sel.skills.extend(v);
+            }
+            "skill-ejecucion=solape,prioridad,id" => {
+                let c = self.ejecucion();
+                if let Some((s, p, id)) = c.first() {
+                    let empate: Vec<&str> = c
+                        .iter()
+                        .filter(|x| x.0 == *s && x.1 == *p)
+                        .map(|x| x.2.as_str())
+                        .collect();
+                    if empate.len() > 1 {
+                        self.sel
+                            .avisos
+                            .push(format!("empate-por-id:{}", empate.join(",")));
+                    }
+                    self.sel.skills.push(id.clone());
+                }
+            }
+            "revisora-del-dominio" => {
+                if let Some(id) = self
+                    .dom
+                    .as_deref()
+                    .and_then(|d| self.revisoras_de(d).into_iter().next())
+                {
+                    self.sel.revisoras.push(id.clone());
+                    self.sel
+                        .rutas
+                        .entry(id)
+                        .or_insert_with(|| self.todas.clone());
+                }
+            }
+            "aviso=dominio-sin-revisora" => {
+                if let Some(d) = &self.dom {
+                    self.sel.avisos.push(format!("dominio-sin-revisora:{d}"));
+                }
+            }
             "revisoras-de-rutas+generica-si-sueltas" => {
                 for (id, rutas) in &self.casan.por_revisora {
                     self.sel.revisoras.push(id.clone());
@@ -236,6 +444,7 @@ impl Estado<'_> {
                     .revisoras
                     .iter()
                     .chain(&self.sel.agentes)
+                    .chain(&self.sel.skills)
                     .filter(|id| !self.instalado(id))
                     .cloned()
                     .collect();
@@ -245,6 +454,7 @@ impl Estado<'_> {
                 }
                 self.sel.revisoras.retain(|id| !fuera.contains(id));
                 self.sel.agentes.retain(|id| !fuera.contains(id));
+                self.sel.skills.retain(|id| !fuera.contains(id));
             }
             "restringir-modelo" => {
                 if let Some(m) = restringir(&self.sel.modelo, self.ctx.modelos_permitidos) {
@@ -256,6 +466,17 @@ impl Estado<'_> {
             }
             a => {
                 if let Some(n) = a
+                    .strip_prefix("suelo=")
+                    .and_then(|n| n.parse::<usize>().ok())
+                {
+                    self.n = self.n.max(n);
+                } else if let Some(r) = a.strip_prefix("mas-rol=") {
+                    // El agente es el de la fila (columna «Ids del inventario»).
+                    let id = f.ids.first().cloned().unwrap_or_else(|| r.to_owned());
+                    if !self.extra.contains(&id) {
+                        self.extra.push(id);
+                    }
+                } else if let Some(n) = a
                     .strip_prefix("modelo-baja=")
                     .and_then(|n| n.parse::<usize>().ok())
                 {
@@ -267,21 +488,24 @@ impl Estado<'_> {
                         (*m).clone_into(&mut self.sel.modelo);
                     }
                 }
-                // `revisora-del-dominio` y `aviso=dominio-sin-revisora`: sin
-                // dominios no hay nada que anadir.
             }
         }
         self.sel.revisoras.sort();
         self.sel.revisoras.dedup();
-        self.sel.agentes.sort();
-        self.sel.agentes.dedup();
-        self.sel != antes
+        // Los agentes de diseno van en el orden de sus roles.
+        if f.rol == RolFila::Revision {
+            self.sel.agentes.sort();
+            self.sel.agentes.dedup();
+        }
+        self.sel.skills.dedup();
+        self.sel.avisos.dedup();
+        (self.sel.clone(), self.n, self.extra.clone(), self.plan) != antes
     }
 }
 
-/// La seleccion de revision para las `rutas` del diff. `Err` si las tablas
-/// del ADR no se leen (no pasa con el ADR compilado: lo prueba
-/// `las_tablas_del_adr_se_leen`).
+/// La seleccion de revision para las `rutas` del diff, sin dominios (la de
+/// F6-3). `Err` si las tablas del ADR no se leen (no pasa con el ADR
+/// compilado: lo prueba `las_tablas_del_adr_se_leen`).
 pub fn revision(
     rutas: &[String],
     cat: &Catalogo,
@@ -298,9 +522,40 @@ pub fn revision_con(
     cat: &Catalogo,
     inv: &Inventario,
     ctx: &Contexto<'_>,
+) -> Result<Seleccion, String> {
+    let sin = dominios::Dominios::new();
+    let e = Entrada {
+        rutas,
+        dominios: &sin,
+        etiquetas: &[],
+        roles_base: 0,
+    };
+    seleccionar_con(adr, Rol::Revision, &e, cat, inv, ctx)
+}
+
+/// La seleccion de `rol` con las tablas del ADR compilado.
+pub fn seleccionar(
+    rol: Rol,
+    e: &Entrada<'_>,
+    cat: &Catalogo,
+    inv: &Inventario,
+    ctx: &Contexto<'_>,
+) -> Result<Seleccion, String> {
+    seleccionar_con(tabla::ADR, rol, e, cat, inv, ctx)
+}
+
+/// Como [`seleccionar`], con las tablas de `adr`.
+pub fn seleccionar_con(
+    adr: &str,
+    rol: Rol,
+    entrada: &Entrada<'_>,
+    cat: &Catalogo,
+    inv: &Inventario,
+    ctx: &Contexto<'_>,
 ) -> Result<Seleccion, String> {
     let filas = tabla::seleccion(adr)?;
     let modelos = tabla::modelo(adr)?;
+    let rutas = entrada.rutas;
     let mut todas: Vec<String> = rutas
         .iter()
         .map(|r| normalizar(r))
@@ -328,10 +583,11 @@ pub fn revision_con(
             "media"
         }
     };
-    let (id_modelo, modelo) = fila_modelo(&modelos, ctx.carril, cx)
+    let (id_modelo, modelo) = fila_modelo(&modelos, rol, ctx.carril, cx)
         .ok_or_else(|| format!("tabla-modelo sin fila para {}/{cx}", ctx.carril.nombre()))?;
     sel.modelo = modelo;
     sel.reglas.push(id_modelo);
+    sel.senales = dominios::anotar(entrada.dominios);
     let mut e = Estado {
         sel,
         casan: casar(&todas, cat),
@@ -339,17 +595,59 @@ pub fn revision_con(
         umbral,
         generica,
         inv,
+        cat,
         ctx,
+        entrada,
+        dom: None,
+        n: 0,
+        escalera: Vec::new(),
+        extra: Vec::new(),
+        plan: true,
     };
     for f in filas
         .iter()
-        .filter(|f| matches!(f.rol, RolFila::Revision | RolFila::Todos))
+        .filter(|f| f.rol == rol.fila() || f.rol == RolFila::Todos)
     {
-        let cumple = f.condicion.iter().all(|a| e.atomo(&a.texto) != a.negado);
-        if cumple && e.aplicar(f) {
+        if rol == Rol::Diseno {
+            if f.rol == RolFila::Todos {
+                e.materializar();
+            } else if !e.plan {
+                // Tras `sin-plan` no se evalua ninguna fila mas de diseno.
+                continue;
+            }
+        }
+        // Las filas que miran «el dominio» se evaluan una vez por dominio.
+        let por_dominio = f.condicion.iter().any(|a| {
+            matches!(
+                a.texto.as_str(),
+                "dominio-con-revisora" | "dominio-sin-revisora"
+            )
+        });
+        let doms: Vec<Option<String>> = if por_dominio {
+            entrada
+                .dominios
+                .iter()
+                .map(|(d, _)| Some(d.clone()))
+                .collect()
+        } else {
+            vec![None]
+        };
+        let mut anotada = false;
+        for d in doms {
+            e.dom = d;
+            let cumple = f.condicion.iter().all(|a| e.atomo(&a.texto) != a.negado);
+            if cumple && e.aplicar(f) {
+                anotada = true;
+            }
+        }
+        e.dom = None;
+        if anotada {
             e.sel.reglas.push(f.id.clone());
         }
     }
+    if rol == Rol::Diseno {
+        e.materializar();
+    }
     Ok(e.sel)
 }
 
@@ -553,4 +851,476 @@ mod tests {
         let roto = tabla::ADR.replace("`lanzador`", "`a-ojo`");
         assert!(revision_con(&roto, &rutas, &c, &i, &ctx).is_err());
     }
+
+    // --- F4-6: diseno e implementacion ---
+
+    /// Agentes de diseno del plugin 0.16 (ids de la `tabla-seleccion`).
+    const DISENO: [&str; 6] = [
+        "arquitectura",
+        "riesgos",
+        "testing",
+        "dominio",
+        "rendimiento",
+        "unificador",
+    ];
+
+    /// Catalogo de la seccion I del ADR: las cinco revisoras, `task-workflow`
+    /// declarada en `recursos:` (como la tomo la PoC de `catalogo-skills.yml`)
+    /// y lo instalado: lo anterior, los seis agentes de brainstorm y el
+    /// integrado; `sin` quita ids del inventario.
+    fn catalogo_muestra(sin: &[&str]) -> (Catalogo, Inventario) {
+        let (c, i) = catalogo(&[]);
+        let mut rs: Vec<Revisora> = c.revisoras().to_vec();
+        rs.extend(catalogo::de_config(&[crate::config::Recurso {
+            id: "taskcode-plugin:task-workflow".to_owned(),
+            tipo: "skill".to_owned(),
+            rol: Some("ejecucion".to_owned()),
+            dominios: Vec::new(),
+            etiquetas: [
+                "taskctl",
+                "sprints",
+                "gitflow",
+                "revision-por-pares",
+                "planificacion",
+            ]
+            .iter()
+            .map(|x| (*x).to_owned())
+            .collect(),
+            prioridad: Some(10),
+        }]));
+        let mut es: Vec<Entrada> = i.entradas().to_vec();
+        es.push(Entrada {
+            tipo: Tipo::Skill,
+            id: "taskcode-plugin:task-workflow".to_owned(),
+            origen: Origen::Integrado,
+        });
+        for n in DISENO {
+            es.push(Entrada {
+                tipo: Tipo::Agente,
+                id: format!("taskcode-plugin:brainstorm-{n}"),
+                origen: Origen::Integrado,
+            });
+        }
+        es.retain(|e| !sin.contains(&e.id.as_str()));
+        (Catalogo::nuevo(rs), Inventario::nuevo(es, Vec::new()))
+    }
+
+    /// Las tres selecciones de una tarea.
+    struct Tres {
+        d: Seleccion,
+        i: Seleccion,
+        r: Seleccion,
+    }
+
+    #[allow(clippy::too_many_arguments)]
+    fn tres(
+        tipo: &str,
+        carril: Carril,
+        cx: &str,
+        texto: &dominios::Texto<'_>,
+        rutas: &[String],
+        cat: &Catalogo,
+        inv: &Inventario,
+        permitidos: &[String],
+        ronda: u32,
+    ) -> Tres {
+        let tabla = tabla::dominios(tabla::ADR).unwrap_or_default();
+        let doms = dominios::dominios(texto, rutas, &tabla, &BTreeMap::new());
+        let base = carril.roles_diseno(
+            crate::plan::heuristica::por_nivel(cx),
+            tipo,
+            &crate::config::Config::suelo(),
+        );
+        let e = super::Entrada {
+            rutas,
+            dominios: &doms,
+            etiquetas: texto.etiquetas,
+            roles_base: base,
+        };
+        let ctx = Contexto {
+            carril,
+            complejidad: Some(cx),
+            modelos_permitidos: permitidos,
+            ronda,
+        };
+        let s = |rol| seleccionar(rol, &e, cat, inv, &ctx).unwrap_or_default();
+        Tres {
+            d: s(Rol::Diseno),
+            i: s(Rol::Implementacion),
+            r: s(Rol::Revision),
+        }
+    }
+
+    fn corto(id: &str) -> String {
+        id.rsplit(':')
+            .next()
+            .unwrap_or(id)
+            .trim_start_matches("brainstorm-")
+            .to_owned()
+    }
+
+    /// Las celdas de la `tabla-muestra` del ADR para una tarea (Dominios,
+    /// Diseno, Implementacion, Revision, Modelo D / I / R y Reglas).
+    fn celdas(t: &Tres) -> Vec<String> {
+        // La PoC muestra la primera senal de cada dominio (ordenadas).
+        let mut doms: Vec<String> = Vec::new();
+        for x in &t.d.senales {
+            let dom = x.split(':').next().unwrap_or_default();
+            if !doms.iter().any(|d| d.starts_with(&format!("{dom} ("))) {
+                doms.push(format!("{})", x.replacen(':', " (", 1)));
+            }
+        }
+        let roles: Vec<String> =
+            t.d.roles
+                .iter()
+                .map(|a| corto(a))
+                .filter(|a| a != "unificador")
+                .collect();
+        let mut diseno = format!("{}: {}", roles.len(), roles.join(", "));
+        if t.d.roles.iter().any(|a| corto(a) == "unificador") {
+            diseno.push_str(" + unificador");
+        }
+        let lista = |v: &[String]| {
+            if v.is_empty() {
+                "—".to_owned()
+            } else {
+                v.iter().map(|x| corto(x)).collect::<Vec<_>>().join(", ")
+            }
+        };
+        // Las de dominio por id y la generica al final (como la PoC).
+        let mut revs: Vec<String> = t.r.revisoras.iter().map(|x| corto(x)).collect();
+        revs.sort_by_key(|x| (x == "code-quality-reviewer", x.clone()));
+        let mut revision = if revs.is_empty() {
+            "—".to_owned()
+        } else {
+            revs.join(", ")
+        };
+        let avisos: Vec<&str> =
+            t.r.avisos
+                .iter()
+                .filter_map(|a| a.strip_prefix("dominio-sin-revisora:"))
+                .collect();
+        if !avisos.is_empty() {
+            revision.push_str(&format!("; aviso: {} sin revisora", avisos.join(", ")));
+        }
+        let m = |s: &Seleccion| {
+            if s.modelo.is_empty() {
+                "—".to_owned()
+            } else {
+                s.modelo.clone()
+            }
+        };
+        vec![
+            if doms.is_empty() {
+                "—".to_owned()
+            } else {
+                doms.join(", ")
+            },
+            diseno,
+            lista(&t.i.skills),
+            revision,
+            format!("{} / {} / {}", m(&t.d), m(&t.i), m(&t.r)),
+            format!(
+                "{} · {} · {}",
+                t.d.reglas.join(" "),
+                t.i.reglas.join(" "),
+                t.r.reglas.join(" ")
+            ),
+        ]
+    }
+
+    #[derive(serde::Deserialize)]
+    struct TareaMuestra {
+        tarea: String,
+        tipo: String,
+        etiquetas: Vec<String>,
+        objetivo: String,
+        criterios: Vec<String>,
+        carril: String,
+        rutas: Vec<String>,
+    }
+
+    /// Criterio 1 (dentro de muestra): con lo que la PoC T uso de las 10
+    /// tareas de ToolsaWeb y OpenDockTrading (fixture
+    /// `tests/fixtures/muestra-f1-15/tareas.json`, leida en solo lectura de
+    /// sus commits fijos) y el catalogo de la seccion I, [`seleccionar`] da
+    /// exactamente las columnas de la `tabla-muestra` del ADR, la que Carlos
+    /// marco «Coincide las 10». El carril efectivo es el de la fixture y la
+    /// complejidad la de la tabla (entradas de la PoC).
+    #[test]
+    fn la_muestra_del_adr_se_reproduce() {
+        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
+            .join("tests/fixtures/muestra-f1-15/tareas.json");
+        let tareas: Vec<TareaMuestra> =
+            serde_json::from_str(&std::fs::read_to_string(p).unwrap_or_default())
+                .unwrap_or_default();
+        assert_eq!(tareas.len(), 10);
+        let filas: Vec<Vec<String>> = tabla::filas(tabla::ADR, "tabla-muestra")
+            .unwrap_or_default()
+            .into_iter()
+            .filter(|f| f.first().is_some_and(|c| c.contains('/')))
+            .collect();
+        assert_eq!(filas.len(), 10);
+        let (cat, inv) = catalogo_muestra(&[]);
+        let permitidos = todos();
+        let mut hash = Vec::new();
+        for t in &tareas {
+            let fila = filas
+                .iter()
+                .find(|f| f.first() == Some(&t.tarea))
+                .cloned()
+                .unwrap_or_default();
+            let efectivo = fila
+                .get(2)
+                .and_then(|c| c.split('→').nth(1))
+                .map(str::trim)
+                .unwrap_or_default();
+            assert_eq!(efectivo, t.carril, "{}", t.tarea);
+            let cx = fila
+                .get(3)
+                .and_then(|c| c.split(' ').next())
+                .unwrap_or_default();
+            let texto = dominios::Texto {
+                objetivo: &t.objetivo,
+                criterios: &t.criterios,
+                etiquetas: &t.etiquetas,
+            };
+            let carril = Carril::desde(&t.carril).unwrap_or(Carril::Funcionalidad);
+            let s = tres(
+                &t.tipo,
+                carril,
+                cx,
+                &texto,
+                &t.rutas,
+                &cat,
+                &inv,
+                &permitidos,
+                1,
+            );
+            let esperado: Vec<String> = fila
+                .get(4..10)
+                .map(|x| x.iter().map(|c| c.replace('`', "")).collect())
+                .unwrap_or_default();
+            assert_eq!(celdas(&s), esperado, "{}", t.tarea);
+            hash.push(celdas(&s));
+        }
+        // Determinista: otra vez, con el catalogo en otro orden, lo mismo.
+        let mut rs: Vec<Revisora> = cat.revisoras().to_vec();
+        rs.extend(cat.otros().iter().cloned());
+        rs.reverse();
+        let cat2 = Catalogo::nuevo(rs);
+        assert_eq!(cat2, cat);
+    }
+
+    /// Criterio 2: el modelo cambia con el carril y la complejidad en los
+    /// tres roles (M01 a M07, errata sin diseno); S13 baja en la ronda 2 sin
+    /// dominio y no con `puertas-seguridad`; S15 restringe tambien diseno e
+    /// implementacion.
+    #[test]
+    fn modelo_por_rol_carril_y_complejidad() {
+        let (cat, inv) = catalogo_muestra(&[]);
+        let permitidos = todos();
+        let criterios = vec!["criterio".to_owned()];
+        let neutro = dominios::Texto {
+            objetivo: "Cambio",
+            criterios: &criterios,
+            etiquetas: &[],
+        };
+        let rutas = vec!["src/a.rs".to_owned()];
+        let modelos = |carril, cx: &str| {
+            let t = tres(
+                "feature",
+                carril,
+                cx,
+                &neutro,
+                &rutas,
+                &cat,
+                &inv,
+                &permitidos,
+                1,
+            );
+            (
+                t.d.modelo,
+                t.i.modelo,
+                t.r.modelo,
+                t.d.reglas.first().cloned().unwrap_or_default(),
+            )
+        };
+        let m = |d: &str, i: &str, r: &str, id: &str| {
+            (d.to_owned(), i.to_owned(), r.to_owned(), id.to_owned())
+        };
+        assert_eq!(
+            modelos(Carril::Errata, "media"),
+            m("", "haiku", "sonnet", "M01")
+        );
+        assert_eq!(
+            modelos(Carril::Fix, "simple"),
+            m("sonnet", "sonnet", "sonnet", "M02")
+        );
+        assert_eq!(
+            modelos(Carril::Fix, "alta"),
+            m("opus", "sonnet", "opus", "M03")
+        );
+        assert_eq!(
+            modelos(Carril::Funcionalidad, "trivial"),
+            m("sonnet", "sonnet", "sonnet", "M04")
+        );
+        assert_eq!(
+            modelos(Carril::Funcionalidad, "media"),
+            m("opus", "sonnet", "opus", "M05")
+        );
+        assert_eq!(
+            modelos(Carril::Funcionalidad, "critica"),
+            m("opus", "opus", "opus", "M06")
+        );
+        assert_eq!(
+            modelos(Carril::RiesgoAlto, "simple"),
+            m("opus", "opus", "opus", "M07")
+        );
+        // Errata: sin plan (S01), ningun agente de diseno.
+        let t = tres(
+            "feature",
+            Carril::Errata,
+            "media",
+            &neutro,
+            &rutas,
+            &cat,
+            &inv,
+            &permitidos,
+            1,
+        );
+        assert_eq!(t.d.reglas, ["M01", "S01"]);
+        assert!(t.d.roles.is_empty() && t.d.agentes.is_empty());
+        // S13: ronda 2 baja sin dominio; con puertas-seguridad no.
+        let r2 = |texto: &dominios::Texto<'_>| {
+            tres(
+                "feature",
+                Carril::Funcionalidad,
+                "media",
+                texto,
+                &rutas,
+                &cat,
+                &inv,
+                &permitidos,
+                2,
+            )
+            .r
+        };
+        let s = r2(&neutro);
+        assert_eq!(s.modelo, "sonnet");
+        assert!(s.reglas.contains(&"S13".to_owned()), "{s:?}");
+        let puertas = dominios::Texto {
+            objetivo: "El check obligatorio",
+            criterios: &criterios,
+            etiquetas: &[],
+        };
+        let s = r2(&puertas);
+        assert_eq!(s.modelo, "opus");
+        assert!(!s.reglas.contains(&"S13".to_owned()), "{s:?}");
+        assert_eq!(s.avisos, ["dominio-sin-revisora:puertas-seguridad"]);
+        assert!(s.reglas.contains(&"S11".to_owned()), "{s:?}");
+        // S15 en diseno e implementacion.
+        let solo = vec!["haiku".to_owned(), "sonnet".to_owned()];
+        let t = tres(
+            "feature",
+            Carril::RiesgoAlto,
+            "media",
+            &neutro,
+            &rutas,
+            &cat,
+            &inv,
+            &solo,
+            1,
+        );
+        for s in [&t.d, &t.i] {
+            assert_eq!(s.modelo, "sonnet", "{s:?}");
+            assert!(s.reglas.contains(&"S15".to_owned()), "{s:?}");
+            assert_eq!(s.avisos, ["modelo-no-permitido:opus->sonnet"]);
+        }
+    }
+
+    /// Criterio 3 (RR-17): el nucleo de `seleccion` no nombra procesos, red,
+    /// ficheros ni clientes de modelos fuera de los tests; y un agente de
+    /// diseno quitado del inventario sale con `no-instalado` (S14), sin
+    /// dejar de estar en los roles (la peticion se escribe igual).
+    #[test]
+    fn sin_red_ni_modelo() {
+        for (n, fuente) in [
+            ("mod.rs", include_str!("mod.rs")),
+            ("dominios.rs", include_str!("dominios.rs")),
+            ("tabla.rs", include_str!("tabla.rs")),
+        ] {
+            let nucleo = fuente.split("#[cfg(test)]").next().unwrap_or_default();
+            for prohibido in [
+                "std::process",
+                "std::net",
+                "std::fs",
+                "ureq",
+                "reqwest",
+                "Command::new",
+                "api.anthropic",
+            ] {
+                assert!(!nucleo.contains(prohibido), "{n} nombra {prohibido}");
+            }
+        }
+        let (cat, inv) = catalogo_muestra(&["taskcode-plugin:brainstorm-riesgos"]);
+        let criterios = vec!["criterio".to_owned()];
+        let puertas = dominios::Texto {
+            objetivo: "Un token nuevo",
+            criterios: &criterios,
+            etiquetas: &[],
+        };
+        let t = tres(
+            "feature",
+            Carril::Funcionalidad,
+            "simple",
+            &puertas,
+            &[],
+            &cat,
+            &inv,
+            &todos(),
+            1,
+        );
+        assert_eq!(
+            t.d.roles,
+            [
+                "taskcode-plugin:brainstorm-arquitectura",
+                "taskcode-plugin:brainstorm-riesgos",
+                "taskcode-plugin:brainstorm-unificador"
+            ]
+        );
+        assert_eq!(
+            t.d.agentes,
+            [
+                "taskcode-plugin:brainstorm-arquitectura",
+                "taskcode-plugin:brainstorm-unificador"
+            ]
+        );
+        assert_eq!(
+            t.d.avisos,
+            ["no-instalado:taskcode-plugin:brainstorm-riesgos"]
+        );
+        assert_eq!(t.d.reglas, ["M04", "S02", "S03", "S05", "S14"]);
+        // Rendimiento por etiqueta: su rol y, con dos, el unificador.
+        let etq = vec!["wasm".to_owned()];
+        let rend = dominios::Texto {
+            objetivo: "Cambio",
+            criterios: &criterios,
+            etiquetas: &etq,
+        };
+        let (cat, inv) = catalogo_muestra(&[]);
+        let t = tres(
+            "feature",
+            Carril::Funcionalidad,
+            "simple",
+            &rend,
+            &[],
+            &cat,
+            &inv,
+            &todos(),
+            1,
+        );
+        assert_eq!(t.d.reglas, ["M04", "S02", "S04", "S05"]);
+        assert_eq!(t.d.roles.len(), 3, "{:?}", t.d.roles);
+    }
 }
diff --git a/crates/tc2/src/seleccion/orden.rs b/crates/tc2/src/seleccion/orden.rs
index a91fa78..a1d01d9 100644
--- a/crates/tc2/src/seleccion/orden.rs
+++ b/crates/tc2/src/seleccion/orden.rs
@@ -1,3 +1,6 @@
+//! `tc2 seleccion` (F4-6, [`orden_seleccion`]): los tres roles de una tarea
+//! sin escribir nada.
+//!
 //! `tc2 revisoras [--json] --rutas <fichero|-> [--nul] [--tarea TASK-NNN]
 //! [--carril <c>] [--complejidad <x>] [--plugin <nombre>=<dir>]...
 //! [--informe <fichero> --seccion <salida> [--artefacto <n>]]
@@ -87,7 +90,8 @@ pub struct Entorno<'a> {
 pub struct Fuentes {
     /// Lo instalado.
     pub inventario: Inventario,
-    /// Las revisoras con metadatos.
+    /// Las entradas con metadatos (revisoras y skills de diseno o
+    /// ejecucion).
     pub catalogo: Catalogo,
     /// Avisos (`sin-base`).
     pub avisos: Vec<String>,
@@ -145,6 +149,10 @@ pub fn fuentes(
             });
         }
     }
+    // Lo que declara `recursos:` de la configuracion de la base (F4-6).
+    if let Ok(cfg) = crate::config::de_base(g, dir) {
+        revisoras.extend(catalogo::de_config(&cfg.recursos));
+    }
     Fuentes {
         inventario: Inventario::nuevo(entradas, lista_plugins),
         catalogo: Catalogo::nuevo(revisoras),
@@ -170,17 +178,55 @@ pub fn de_tarea(t: &Tarea, c: &Cuerpo) -> (Carril, &'static str) {
         &t.dependencias,
         &e,
     );
+    (carril, complejidad(&r))
+}
+
+/// Los dominios de una tarea para la revision (F4-6; F1-15 decision 3):
+/// los del texto vivo y las `rutas` del diff con `dominios_extra` de la
+/// configuracion de la base, mas las senales congeladas con el plan
+/// aprobado (`roles.diseno.senales`): reescribir la tarea no quita un
+/// dominio. Sin tarea, solo las rutas.
+pub fn dominios_de(
+    tarea: Option<(&Tarea, &Cuerpo)>,
+    rutas: &[String],
+    cfg: &crate::config::Config,
+) -> super::dominios::Dominios {
+    let tabla = super::tabla::dominios(super::tabla::ADR).unwrap_or_default();
+    let extra = &cfg.seleccion.dominios_extra;
+    match tarea {
+        Some((t, c)) => {
+            let e = heuristica::Enunciado::de(c);
+            let texto = super::dominios::Texto {
+                objetivo: &e.objetivo,
+                criterios: &e.criterios,
+                etiquetas: &t.etiquetas,
+            };
+            let d = super::dominios::dominios(&texto, rutas, &tabla, extra);
+            super::dominios::union(&d, &t.roles.diseno.senales)
+        }
+        None => {
+            let texto = super::dominios::Texto {
+                objetivo: "",
+                criterios: &[],
+                etiquetas: &[],
+            };
+            super::dominios::dominios(&texto, rutas, &tabla, extra)
+        }
+    }
+}
+
+/// La complejidad de la seleccion: la mayor entre la declarada y la
+/// heuristica (F1-15 decision 1).
+pub fn complejidad(r: &heuristica::Resolucion) -> &'static str {
     let indice = |n: &str| heuristica::NIVELES.iter().position(|x| *x == n);
-    let cx = r
-        .declarado
+    r.declarado
         .as_deref()
         .and_then(indice)
         .into_iter()
         .chain(indice(r.heuristico))
         .max()
         .and_then(|i| heuristica::NIVELES.get(i).copied())
-        .unwrap_or(r.heuristico);
-    (carril, cx)
+        .unwrap_or(r.heuristico)
 }
 
 /// Las claves de la seleccion en un informe.
@@ -372,6 +418,11 @@ fn paso(a: &[String], e: &Entorno<'_>) -> Result<Informe, Informe> {
     let mut avisos = f.avisos.clone();
     let mut carril = Carril::Funcionalidad;
     let mut cx: Option<String> = None;
+    let config = crate::config::de_base(e.git, e.dir).unwrap_or_else(|_| {
+        avisos.push("config-de-base-no-leida".to_owned());
+        crate::config::Config::suelo()
+    });
+    let mut doms = dominios_de(None, &rutas, &config);
     if let Some(id) = &a.tarea {
         let leida = crate::estado::leer(e.git, e.dir)
             .map_err(|i| i.valor("motivo").unwrap_or("estado").to_owned())
@@ -386,6 +437,7 @@ fn paso(a: &[String], e: &Entorno<'_>) -> Result<Informe, Informe> {
                 let (k, x) = de_tarea(&t, &c);
                 carril = k;
                 cx = Some(x.to_owned());
+                doms = dominios_de(Some((&t, &c)), &rutas, &config);
             }
             Err(m) => avisos.push(format!("tarea-no-leida:{m}")),
         }
@@ -396,17 +448,26 @@ fn paso(a: &[String], e: &Entorno<'_>) -> Result<Informe, Informe> {
     if a.complejidad.is_some() {
         cx = a.complejidad.clone();
     }
-    let config = crate::config::de_base(e.git, e.dir).unwrap_or_else(|_| {
-        avisos.push("config-de-base-no-leida".to_owned());
-        crate::config::Config::suelo()
-    });
     let ctx = Contexto {
         carril,
         complejidad: cx.as_deref(),
         modelos_permitidos: &config.seleccion.modelos_permitidos,
         ronda: a.ronda.unwrap_or(1),
     };
-    let mut s = super::revision(&rutas, &f.catalogo, &f.inventario, &ctx).map_err(|x| {
+    let entrada = super::Entrada {
+        rutas: &rutas,
+        dominios: &doms,
+        etiquetas: &[],
+        roles_base: 0,
+    };
+    let mut s = super::seleccionar(
+        super::Rol::Revision,
+        &entrada,
+        &f.catalogo,
+        &f.inventario,
+        &ctx,
+    )
+    .map_err(|x| {
         Informe::nuevo(NOMBRE, Resultado::Error)
             .diagnostico(Mensaje::nuevo(&txt::TABLA_ILEGIBLE).con("error", x))
     })?;
@@ -430,3 +491,180 @@ fn paso(a: &[String], e: &Entorno<'_>) -> Result<Informe, Informe> {
     }
     Ok(i)
 }
+
+/// Nombre de la orden que da la seleccion de los tres roles (F4-6).
+pub const SELECCION: &str = "seleccion";
+
+fn uso_seleccion(motivo: &str) -> Informe {
+    Informe::nuevo(SELECCION, Resultado::Uso)
+        .con("motivo", motivo)
+        .diagnostico(&txt::USO_SELECCION)
+}
+
+/// Una tarea leida de un fichero en formato 1 (0.16) o 2: la de la 0.16 se
+/// pasa al modelo de formato 2 con lo que la seleccion mira (tipo,
+/// etiquetas, complejidad y dependencias).
+fn tarea_de_fichero(texto: &str) -> Result<(Tarea, Cuerpo), String> {
+    match crate::tarea::leer(texto).map_err(|e| e.to_string())? {
+        crate::tarea::Documento::Formato2(t, c) => Ok((t, c)),
+        crate::tarea::Documento::Formato1(v, c) => {
+            let mut t = Tarea::nueva(&v.id, &v.titulo, &v.creado);
+            t.tipo = v.tipo;
+            t.etiquetas = v.etiquetas;
+            t.complejidad = v.complejidad;
+            t.dependencias = v.dependencias;
+            Ok((t, c))
+        }
+    }
+}
+
+/// `seleccion [--json] [TASK-NNN] [--tarea-fichero <tarea.md>] [--rutas
+/// <fichero>]` (F4-6): la seleccion de diseno, implementacion y revision de
+/// una tarea sin escribir nada (contrato para el orquestador, F4-4, y
+/// `lote`, F1-16). La tarea sale de la rama de estado o, con
+/// `--tarea-fichero`, de un `tarea.md` en formato 1 o 2 (para medir sobre
+/// tareas reales de la 0.16). Diseno e implementacion se calculan como en
+/// `plan` (el texto, sin rutas, y las senales congeladas); revision, con
+/// las `--rutas` del diff. Inventario, catalogo y configuracion, los de
+/// [`fuentes`] y la base.
+pub fn orden_seleccion(a: &[String], e: &Entorno<'_>) -> Informe {
+    match paso_seleccion(a, e) {
+        Ok(i) | Err(i) => i,
+    }
+}
+
+fn paso_seleccion(a: &[String], e: &Entorno<'_>) -> Result<Informe, Informe> {
+    let (mut id, mut fichero, mut rutas_f) = (None, None, None);
+    let mut it = a.iter();
+    while let Some(x) = it.next() {
+        let mut valor = || {
+            it.next()
+                .cloned()
+                .ok_or_else(|| uso_seleccion("falta-valor"))
+        };
+        match x.as_str() {
+            "--tarea-fichero" if fichero.is_none() => fichero = Some(valor()?),
+            "--rutas" if rutas_f.is_none() => rutas_f = Some(valor()?),
+            v if id.is_none() && crate::git::es_id(v) => id = Some(v.to_owned()),
+            _ => return Err(uso_seleccion("argumento-desconocido")),
+        }
+    }
+    let error = |ruta: &str, err: String| {
+        Informe::nuevo(SELECCION, Resultado::Error).diagnostico(
+            Mensaje::nuevo(&txt::NO_SE_LEE)
+                .con("ruta", ruta)
+                .con("error", err),
+        )
+    };
+    let (t, c) = match (&fichero, &id) {
+        (Some(f), _) => {
+            let texto = leer_con_tope(Path::new(f), TOPE_RUTAS)
+                .map_err(|x| error(f, x.to_string()))?
+                .ok_or_else(|| error(f, format!("mas de {TOPE_RUTAS} bytes")))?;
+            tarea_de_fichero(&texto).map_err(|x| error(f, x))?
+        }
+        (None, Some(id)) => {
+            let v = crate::estado::leer(e.git, e.dir).map_err(|mut i| {
+                SELECCION.clone_into(&mut i.comando);
+                i
+            })?;
+            crate::estado::tarea_de(&v, id).map_err(|f| match f {
+                crate::estado::Fallo::Rechazada {
+                    motivo,
+                    diagnostico,
+                } => Informe::nuevo(SELECCION, Resultado::Rechazada)
+                    .con("motivo", motivo)
+                    .diagnostico(diagnostico),
+                crate::estado::Fallo::Error(m) => {
+                    Informe::nuevo(SELECCION, Resultado::Error).diagnostico(m)
+                }
+            })?
+        }
+        (None, None) => return Err(uso_seleccion("sin-tarea")),
+    };
+    let rutas = match &rutas_f {
+        Some(f) => separar_rutas(
+            &leer_con_tope(Path::new(f), TOPE_RUTAS)
+                .map_err(|x| error(f, x.to_string()))?
+                .ok_or_else(|| error(f, format!("mas de {TOPE_RUTAS} bytes")))?,
+            false,
+        ),
+        None => Vec::new(),
+    };
+    let f = fuentes(e.git, e.dir, e.claude, &[]);
+    let mut avisos = f.avisos.clone();
+    let config = crate::config::de_base(e.git, e.dir).unwrap_or_else(|_| {
+        avisos.push("config-de-base-no-leida".to_owned());
+        crate::config::Config::suelo()
+    });
+    let (carril, _) = de_tarea(&t, &c);
+    let enunciado = heuristica::Enunciado::de(&c);
+    let r = heuristica::resolver(
+        t.complejidad.as_deref(),
+        &t.etiquetas,
+        &t.tipo,
+        &t.dependencias,
+        &enunciado,
+    );
+    let cx = complejidad(&r);
+    let n = carril.roles_diseno(r.roles, &t.tipo, &config);
+    let ctx = Contexto {
+        carril,
+        complejidad: Some(cx),
+        modelos_permitidos: &config.seleccion.modelos_permitidos,
+        ronda: 1,
+    };
+    let doms_plan = dominios_de(Some((&t, &c)), &[], &config);
+    let doms_rev = dominios_de(Some((&t, &c)), &rutas, &config);
+    let sel = |rol, doms: &super::dominios::Dominios| {
+        let entrada = super::Entrada {
+            rutas: &rutas,
+            dominios: doms,
+            etiquetas: &t.etiquetas,
+            roles_base: n,
+        };
+        super::seleccionar(rol, &entrada, &f.catalogo, &f.inventario, &ctx).map_err(|x| {
+            Informe::nuevo(SELECCION, Resultado::Error)
+                .diagnostico(Mensaje::nuevo(&txt::TABLA_ILEGIBLE).con("error", x))
+        })
+    };
+    let d = sel(super::Rol::Diseno, &doms_plan)?;
+    let im = sel(super::Rol::Implementacion, &doms_plan)?;
+    let rv = sel(super::Rol::Revision, &doms_rev)?;
+    for (rol, s) in [("diseno", &d), ("implementacion", &im), ("revision", &rv)] {
+        avisos.extend(s.avisos.iter().map(|a| format!("{rol}:{a}")));
+    }
+    let nombres = |d: &super::dominios::Dominios| {
+        let v: Vec<String> = d.iter().map(|(x, _)| x.clone()).collect();
+        if v.is_empty() {
+            "ninguno".to_owned()
+        } else {
+            v.join(",")
+        }
+    };
+    let modelo = |s: &Seleccion| {
+        if s.modelo.is_empty() {
+            "-".to_owned()
+        } else {
+            s.modelo.clone()
+        }
+    };
+    Ok(Informe::nuevo(SELECCION, Resultado::Ok)
+        .con("id", t.id.as_str())
+        .con("carril", carril.nombre())
+        .con("complejidad", cx)
+        .con("dominios", nombres(&doms_rev))
+        .con("dominios-diseno", nombres(&doms_plan))
+        .con_lista("senales", rv.senales.clone())
+        .con_lista("diseno-agentes", d.roles.clone())
+        .con("diseno-modelo", modelo(&d))
+        .con_lista("diseno-reglas", d.reglas.clone())
+        .con_lista("implementacion-skills", im.skills.clone())
+        .con("implementacion-modelo", modelo(&im))
+        .con_lista("implementacion-reglas", im.reglas.clone())
+        .con_lista("revision-revisoras", rv.revisoras.clone())
+        .con_lista("revision-agentes", rv.agentes.clone())
+        .con("revision-modelo", modelo(&rv))
+        .con_lista("revision-reglas", rv.reglas.clone())
+        .con_lista("avisos", avisos))
+}
diff --git a/crates/tc2/src/seleccion/tabla.rs b/crates/tc2/src/seleccion/tabla.rs
index fb76737..20fc51e 100644
--- a/crates/tc2/src/seleccion/tabla.rs
+++ b/crates/tc2/src/seleccion/tabla.rs
@@ -1,5 +1,5 @@
-//! Las tablas `tabla-seleccion` y `tabla-modelo` del ADR F1-15, leidas del
-//! propio ADR (decision 1: las reglas no se escriben aparte en el codigo).
+//! Las tablas `tabla-seleccion`, `tabla-modelo` y `tabla-dominios` del ADR
+//! F1-15, leidas del propio ADR (decision 1: las reglas no se escriben aparte en el codigo).
 //!
 //! El ADR entra en el binario con `include_str!`: cambiar una fila del ADR
 //! cambia la seleccion al recompilar, y una condicion o una accion fuera del
@@ -17,7 +17,7 @@ pub const ESCALA: [&str; 3] = ["haiku", "sonnet", "opus"];
 const CARRILES: [&str; 4] = ["errata", "fix", "funcionalidad", "riesgo-alto"];
 
 /// Dominios cerrados (F1-15 decision 3).
-const DOMINIOS: [&str; 2] = ["puertas-seguridad", "rendimiento"];
+pub const DOMINIOS: [&str; 2] = ["puertas-seguridad", "rendimiento"];
 
 /// Atomos sin argumento del vocabulario cerrado (decision 1).
 const ATOMOS_FIJOS: [&str; 9] = [
@@ -78,14 +78,53 @@ pub struct FilaModelo {
     pub carril: String,
     /// Complejidades a las que se aplica; vacia es «cualquiera».
     pub complejidades: Vec<String>,
+    /// Modelo de diseno (columna «Diseño»); `None` con «—» (sin plan).
+    pub diseno: Option<String>,
+    /// Modelo de implementacion (columna «Implementación»).
+    pub implementacion: String,
     /// Modelo de revision (columna «Revisión»).
     pub revision: String,
 }
 
+/// Tipo de senal de una fila de `tabla-dominios`.
+#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
+pub enum Senal {
+    /// Igualdad exacta con una etiqueta de la tarea.
+    Etiqueta,
+    /// Principio de una palabra del objetivo o de los criterios.
+    Palabra,
+    /// Glob sobre las rutas del cambio, sin distinguir mayusculas.
+    Ruta,
+}
+
+impl Senal {
+    /// Nombre en la tabla y en las senales anotadas.
+    pub fn nombre(self) -> &'static str {
+        match self {
+            Self::Etiqueta => "etiqueta",
+            Self::Palabra => "palabra",
+            Self::Ruta => "ruta",
+        }
+    }
+}
+
+/// Una fila de `tabla-dominios`.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct FilaDominio {
+    /// `D01`...
+    pub id: String,
+    /// Dominio de la lista cerrada.
+    pub dominio: String,
+    /// Tipo de senal.
+    pub senal: Senal,
+    /// Valor (etiqueta, palabra o glob).
+    pub valor: String,
+}
+
 /// Las filas entre `<!-- <marca>:inicio -->` y `<!-- <marca>:fin -->`, sin
 /// la cabecera ni el separador, como listas de celdas sin espacios ni
 /// comillas invertidas.
-fn filas(texto: &str, marca: &str) -> Result<Vec<Vec<String>>, String> {
+pub(crate) fn filas(texto: &str, marca: &str) -> Result<Vec<Vec<String>>, String> {
     let inicio = format!("<!-- {marca}:inicio -->");
     let fin = format!("<!-- {marca}:fin -->");
     let desde = texto
@@ -225,7 +264,7 @@ pub fn seleccion(texto: &str) -> Result<Vec<FilaSeleccion>, String> {
 pub fn modelo(texto: &str) -> Result<Vec<FilaModelo>, String> {
     let mut v = Vec::new();
     for c in filas(texto, "tabla-modelo")? {
-        let [id, carril, cx, _diseno, _impl, revision, ..] = c.as_slice() else {
+        let [id, carril, cx, diseno, implementacion, revision, ..] = c.as_slice() else {
             return Err(format!("fila de tabla-modelo incompleta: {c:?}"));
         };
         let carril = sin_comillas(carril);
@@ -243,15 +282,27 @@ pub fn modelo(texto: &str) -> Result<Vec<FilaModelo>, String> {
                 return Err(format!("{id}: complejidad desconocida {x}"));
             }
         }
-        let revision = sin_comillas(revision);
-        if !ESCALA.contains(&revision.as_str()) {
-            return Err(format!("{id}: modelo fuera de la escala {revision}"));
-        }
+        let en_escala = |m: &str| -> Result<String, String> {
+            let m = sin_comillas(m);
+            if ESCALA.contains(&m.as_str()) {
+                Ok(m)
+            } else {
+                Err(format!("{id}: modelo fuera de la escala {m}"))
+            }
+        };
+        // «—» en diseno: sin plan (errata, S01).
+        let diseno = if sin_comillas(diseno) == "—" {
+            None
+        } else {
+            Some(en_escala(diseno)?)
+        };
         v.push(FilaModelo {
             id: id.clone(),
             carril,
             complejidades,
-            revision,
+            diseno,
+            implementacion: en_escala(implementacion)?,
+            revision: en_escala(revision)?,
         });
     }
     // Cada combinacion de carril y complejidad, exactamente una fila.
@@ -272,6 +323,41 @@ pub fn modelo(texto: &str) -> Result<Vec<FilaModelo>, String> {
     Ok(v)
 }
 
+/// Lee `tabla-dominios` de `texto`: dominio de la lista cerrada, senal
+/// `etiqueta|palabra|ruta` y valor no vacio.
+pub fn dominios(texto: &str) -> Result<Vec<FilaDominio>, String> {
+    let mut v: Vec<FilaDominio> = Vec::new();
+    for c in filas(texto, "tabla-dominios")? {
+        let [id, dominio, senal, valor, ..] = c.as_slice() else {
+            return Err(format!("fila de tabla-dominios incompleta: {c:?}"));
+        };
+        let dominio = sin_comillas(dominio);
+        if !DOMINIOS.contains(&dominio.as_str()) {
+            return Err(format!("{id}: dominio fuera de la lista cerrada {dominio}"));
+        }
+        let senal = match sin_comillas(senal).as_str() {
+            "etiqueta" => Senal::Etiqueta,
+            "palabra" => Senal::Palabra,
+            "ruta" => Senal::Ruta,
+            otra => return Err(format!("{id}: senal desconocida {otra}")),
+        };
+        let valor = sin_comillas(valor);
+        if valor.is_empty() {
+            return Err(format!("{id}: valor vacio"));
+        }
+        if v.iter().any(|f| f.id == *id) {
+            return Err(format!("{id} repetida"));
+        }
+        v.push(FilaDominio {
+            id: id.clone(),
+            dominio,
+            senal,
+            valor,
+        });
+    }
+    Ok(v)
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
@@ -293,6 +379,43 @@ mod tests {
         assert_eq!(s13.map(|c| c.len()), Some(3));
         let m = modelo(ADR).unwrap_or_default();
         assert_eq!(m.len(), 7, "{:?}", modelo(ADR).err());
+        // Las tres columnas: M01 (errata) sin diseno; M05 opus/sonnet/opus.
+        let col = |id: &str| {
+            m.iter().find(|f| f.id == id).map(|f| {
+                (
+                    f.diseno.clone(),
+                    f.implementacion.clone(),
+                    f.revision.clone(),
+                )
+            })
+        };
+        assert_eq!(
+            col("M01"),
+            Some((None, "haiku".to_owned(), "sonnet".to_owned()))
+        );
+        assert_eq!(
+            col("M05"),
+            Some((
+                Some("opus".to_owned()),
+                "sonnet".to_owned(),
+                "opus".to_owned()
+            ))
+        );
+        let d = dominios(ADR).unwrap_or_default();
+        assert_eq!(d.len(), 26, "{:?}", dominios(ADR).err());
+        assert_eq!(
+            d.iter().find(|f| f.id == "D22").map(|f| (
+                f.dominio.as_str(),
+                f.senal,
+                f.valor.as_str()
+            )),
+            Some(("puertas-seguridad", Senal::Ruta, ".claude/**"))
+        );
+        // Fuera de la lista cerrada o de las senales, no se lee.
+        assert!(dominios(&ADR.replace("| D23 | `rendimiento`", "| D23 | `datos`")).is_err());
+        assert!(
+            dominios(&ADR.replace("| ruta | `.claude/**`", "| fichero | `.claude/**`")).is_err()
+        );
     }
 
     /// Lo que no es del vocabulario no se lee.
diff --git a/crates/tc2/src/tarea/modelo.rs b/crates/tc2/src/tarea/modelo.rs
index 25c28bf..785479d 100644
--- a/crates/tc2/src/tarea/modelo.rs
+++ b/crates/tc2/src/tarea/modelo.rs
@@ -274,8 +274,25 @@ pub struct Rol {
     pub modelo: Option<String>,
     /// Skills del rol.
     pub skills: Vec<String>,
-    /// Regla que eligio la skill.
+    /// Regla que eligio la skill (vocabulario de la 0.16).
     pub regla: Option<String>,
+    /// Agentes elegidos (F1-15 decision 10; F4-6).
+    #[serde(default)]
+    pub agentes: Vec<String>,
+    /// Ids de las filas de las tablas de F1-15 que se aplicaron.
+    #[serde(default)]
+    pub reglas: Vec<String>,
+    /// Senales de dominio (`<dominio>:<senal>:<valor>`); en diseno son las
+    /// congeladas con el plan aprobado.
+    #[serde(default)]
+    pub senales: Vec<String>,
+    /// Avisos de la seleccion.
+    #[serde(default)]
+    pub avisos: Vec<String>,
+    /// Huella del inventario con el que se calculo.
+    pub inventario: Option<String>,
+    /// Huella del catalogo con el que se calculo.
+    pub catalogo: Option<String>,
 }
 
 /// El rol de revision, que ademas nombra el agente.
@@ -288,8 +305,27 @@ pub struct RolRevision {
     pub modelo: Option<String>,
     /// Skills del revisor.
     pub skills: Vec<String>,
-    /// Regla que eligio la skill.
+    /// Regla que eligio la skill (vocabulario de la 0.16).
     pub regla: Option<String>,
+    /// Agentes elegidos (F1-15 decision 10; F4-6).
+    #[serde(default)]
+    pub agentes: Vec<String>,
+    /// Ids de las filas de las tablas de F1-15 que se aplicaron.
+    #[serde(default)]
+    pub reglas: Vec<String>,
+    /// Senales de dominio (`<dominio>:<senal>:<valor>`); en diseno son las
+    /// congeladas con el plan aprobado.
+    #[serde(default)]
+    pub senales: Vec<String>,
+    /// Avisos de la seleccion.
+    #[serde(default)]
+    pub avisos: Vec<String>,
+    /// Huella del inventario con el que se calculo.
+    pub inventario: Option<String>,
+    /// Huella del catalogo con el que se calculo.
+    pub catalogo: Option<String>,
+    /// Ronda de revision en la que se calculo.
+    pub ronda: Option<u32>,
 }
 
 /// El mapa `roles` (RR-17). Un campo que no conoce el modelo se rechaza: al
@@ -450,9 +486,43 @@ fn rol(clave: &str, r: Rol) -> Result<Rol, ErrorTarea> {
         modelo: texto_opcional(clave, r.modelo)?,
         skills: textos(clave, r.skills)?,
         regla: uno_de_opcional(clave, r.regla, &REGLAS)?,
+        agentes: textos(clave, r.agentes)?,
+        reglas: textos(clave, r.reglas)?,
+        senales: textos(clave, r.senales)?,
+        avisos: textos(clave, r.avisos)?,
+        inventario: texto_opcional(clave, r.inventario)?,
+        catalogo: texto_opcional(clave, r.catalogo)?,
     })
 }
 
+/// Las claves ampliadas de F4-6, solo las que tienen valor (una tarea de
+/// antes se reescribe byte a byte igual).
+fn ampliado(
+    agentes: &[String],
+    reglas: &[String],
+    senales: &[String],
+    avisos: &[String],
+    huellas: (Option<&str>, Option<&str>),
+) -> String {
+    let mut s = String::new();
+    for (k, v) in [
+        ("agentes", agentes),
+        ("reglas", reglas),
+        ("senales", senales),
+        ("avisos", avisos),
+    ] {
+        if !v.is_empty() {
+            s.push_str(&format!(", {k}: {}", yaml::lista(v)));
+        }
+    }
+    for (k, v) in [("inventario", huellas.0), ("catalogo", huellas.1)] {
+        if let Some(v) = v {
+            s.push_str(&format!(", {k}: {}", yaml::cadena(v)));
+        }
+    }
+    s
+}
+
 /// Claves que faltan de `claves` en `partes` (todas son obligatorias).
 fn faltan(partes: &Partes<'_>, claves: &[&str]) -> Result<(), ErrorTarea> {
     match claves.iter().find(|c| !partes.tiene(c)) {
@@ -597,6 +667,13 @@ impl Tarea {
                     modelo: texto_opcional("roles.revision", revision.modelo)?,
                     skills: textos("roles.revision", revision.skills)?,
                     regla: uno_de_opcional("roles.revision", revision.regla, &REGLAS)?,
+                    agentes: textos("roles.revision", revision.agentes)?,
+                    reglas: textos("roles.revision", revision.reglas)?,
+                    senales: textos("roles.revision", revision.senales)?,
+                    avisos: textos("roles.revision", revision.avisos)?,
+                    inventario: texto_opcional("roles.revision", revision.inventario)?,
+                    catalogo: texto_opcional("roles.revision", revision.catalogo)?,
+                    ronda: revision.ronda,
                 },
             },
             estimacion,
@@ -624,10 +701,17 @@ impl Tarea {
         let r = &self.roles;
         let rol = |x: &Rol| {
             format!(
-                "{{modelo: {}, skills: {}, regla: {}}}",
+                "{{modelo: {}, skills: {}, regla: {}{}}}",
                 yaml::opcional(x.modelo.as_deref()),
                 yaml::lista(&x.skills),
-                yaml::opcional(x.regla.as_deref())
+                yaml::opcional(x.regla.as_deref()),
+                ampliado(
+                    &x.agentes,
+                    &x.reglas,
+                    &x.senales,
+                    &x.avisos,
+                    (x.inventario.as_deref(), x.catalogo.as_deref())
+                )
             )
         };
         let mut s = String::from("---\nformato: 2\n");
@@ -655,12 +739,23 @@ impl Tarea {
                 .map_or_else(|| "null".to_owned(), Asignado::yaml),
         );
         let rv = &r.revision;
+        let ronda = rv
+            .ronda
+            .map(|n| format!(", ronda: {n}"))
+            .unwrap_or_default();
         let revision = format!(
-            "{{agente: {}, modelo: {}, skills: {}, regla: {}}}",
+            "{{agente: {}, modelo: {}, skills: {}, regla: {}{}{ronda}}}",
             yaml::opcional(rv.agente.as_deref()),
             yaml::opcional(rv.modelo.as_deref()),
             yaml::lista(&rv.skills),
-            yaml::opcional(rv.regla.as_deref())
+            yaml::opcional(rv.regla.as_deref()),
+            ampliado(
+                &rv.agentes,
+                &rv.reglas,
+                &rv.senales,
+                &rv.avisos,
+                (rv.inventario.as_deref(), rv.catalogo.as_deref())
+            )
         );
         let roles = format!(
             "\n  diseno: {}\n  implementacion: {}\n  revision: {revision}",
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/02-en-curso/TASK-085/evidencias/tabla-10-tareas-taskcodetwo.md                     | 16 ++++++++++++++++
 tareas/{01-en-diseno => 02-en-curso}/TASK-085/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-085/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-085/tarea.md                                    |  3 ++-
 4 files changed, 18 insertions(+), 1 deletion(-)
````

## Tests del diff (5 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/aprobar.rs" ":(literal)crates/tc2/tests/comun/mod.rs" ":(literal)crates/tc2/tests/plan.rs" ":(literal)crates/tc2/tests/seleccion.rs" ":(literal)crates/tc2/tests/tarea.rs"
````

````
crates/tc2/tests/aprobar.rs   | 163 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/comun/mod.rs |  53 +++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/plan.rs      |  86 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/seleccion.rs | 163 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/tarea.rs     |  35 +++++++++++++++++++++++++++++++++++
 5 files changed, 500 insertions(+)
````
