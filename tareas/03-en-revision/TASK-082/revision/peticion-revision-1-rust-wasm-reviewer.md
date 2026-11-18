# Peticion de revision — TASK-082 (ronda 1)

- Tarea: TASK-082 — F3-11 Aviso de solape entre tareas en curso
- Rama revisada: feature/task-082-f3-11-aviso-de-solape-entre-tareas-en-cu
- Rama base: develop
- Commit revisado (HEAD): 1866457c7b052bf73ffa8205973f0d364b28b111
- Fecha: 2026-11-18
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-082 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-082/tarea.md
- tareas/03-en-revision/TASK-082/planificacion/
- crates/tc2/src/git/diff.rs
- crates/tc2/src/git/mod.rs
- crates/tc2/src/i18n/catalogo/toma.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/solape.rs
- crates/tc2/src/toma/mod.rs
- crates/tc2/src/toma/operacion.rs
- crates/tc2/tests/liberar.rs
- crates/tc2/tests/solape.rs

## Commits a revisar (git log develop..HEAD)

````
1866457 docs(TASK-082): autorrevision con rust-wasm-reviewer y code-quality-reviewer
277a5b9 docs(TASK-082): adenda de F1-17 con las divergencias del solape sin zonas
57bd1fb test(TASK-082): solape en start con dos clones y remoto bare (C1 a C10)
7cece03 feat(TASK-082): aviso y nota de solape en stderr, saneados y con catalogo
14bf30a feat(TASK-082): start calcula el solape y anade las claves solape a la linea
6516ff0 feat(TASK-082): ficheros_de_rama con un solo git diff --name-only sin red
d4f04e5 feat(TASK-082): nucleo puro del solape (modulo, cruzar y tareas en curso)
25f3d0a chore(TASK-082): tarea en curso
````

## Diff de tu dominio (9 fichero(s) de 10; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/git/diff.rs b/crates/tc2/src/git/diff.rs
index 7ef0dc4..ff05e5f 100644
--- a/crates/tc2/src/git/diff.rs
+++ b/crates/tc2/src/git/diff.rs
@@ -1,6 +1,9 @@
 //! `diff-tarea`: resumen en una linea de lo que cambia la tarea frente a su
 //! base (ADR F1-4). El diff completo, a peticion, es de `review` (F6-2).
+//! [`ficheros_de_rama`]: los ficheros que toca una rama, para el aviso de
+//! solape de `start` (F3-11).
 
+use std::collections::BTreeSet;
 use std::path::Path;
 
 use super::{Git, Paso, Subcomando, base_de_rama, commit_de, exigir, rama_actual};
@@ -13,6 +16,33 @@ pub fn diff_tarea(git: &Git, dir: &Path) -> Informe {
     diff_paso(git, dir).unwrap_or_else(|i| i)
 }
 
+/// Los ficheros que cambia `rama` desde su `merge-base` con `base` (F3-11):
+/// un solo `git diff --name-only -z --no-renames <base>...<rama> --`, sin
+/// red. Con `--no-renames` un renombrado da los dos nombres (el viejo y el
+/// nuevo), como S08 de F1-15; con `-z` y `core.quotepath=false`, las rutas
+/// tal cual, sin comillas. Un `diff` que falla (sin `merge-base`, la base o
+/// la rama no existen) es el `Err`.
+pub fn ficheros_de_rama(
+    git: &Git,
+    dir: &Path,
+    op: &str,
+    base: &str,
+    rama: &str,
+) -> Paso<BTreeSet<String>> {
+    let rango = format!("{base}...{rama}");
+    let s = exigir(
+        git,
+        dir,
+        op,
+        Subcomando::Diff,
+        &["--name-only", "-z", "--no-renames", &rango, "--"],
+    )?;
+    Ok(s.split('\0')
+        .filter(|r| !r.is_empty())
+        .map(str::to_owned)
+        .collect())
+}
+
 fn diff_paso(git: &Git, dir: &Path) -> Paso<Informe> {
     const OP: &str = "diff-tarea";
     let Some(rama) = rama_actual(git, dir, OP)? else {
diff --git a/crates/tc2/src/git/mod.rs b/crates/tc2/src/git/mod.rs
index 58bffca..5eb37a6 100644
--- a/crates/tc2/src/git/mod.rs
+++ b/crates/tc2/src/git/mod.rs
@@ -18,7 +18,7 @@ pub(crate) mod remoto;
 mod tag;
 mod worktree;
 
-pub use diff::diff_tarea;
+pub use diff::{diff_tarea, ficheros_de_rama};
 pub use estado::Cambio;
 pub use guardar::guardar;
 pub use puerto::{ErrorGit, Git, PREFIJO_REFS_PROPIAS, SalidaCruda, SalidaGit, Subcomando};
diff --git a/crates/tc2/src/i18n/catalogo/toma.rs b/crates/tc2/src/i18n/catalogo/toma.rs
index e1374f7..1a50719 100644
--- a/crates/tc2/src/i18n/catalogo/toma.rs
+++ b/crates/tc2/src/i18n/catalogo/toma.rs
@@ -25,4 +25,24 @@ crate::i18n::textos! {
         es: "la tarea {id} depende de tareas sin terminar ({ids}): terminalas antes o elige otra con tc2 next",
         en: "task {id} depends on unfinished tasks ({ids}): finish them first or pick another one with tc2 next",
     }
+    AVISO_SOLAPE {
+        es: "aviso: {id} ({quien}) toca {que}: hablad antes de seguir (solo es un aviso; start ya se hizo)",
+        en: "warning: {id} ({quien}) touches {que}: talk before going on (only a warning; start is done)",
+    }
+    SOLAPE_FICHEROS {
+        es: "los mismos ficheros ({ficheros})",
+        en: "the same files ({ficheros})",
+    }
+    SOLAPE_MODULOS {
+        es: "los mismos modulos ({modulos})",
+        en: "the same modules ({modulos})",
+    }
+    SOLAPE_FICHEROS_Y_MODULOS {
+        es: "los mismos ficheros ({ficheros}) y modulos ({modulos})",
+        en: "the same files ({ficheros}) and modules ({modulos})",
+    }
+    NOTA_EN_CURSO {
+        es: "nota: tu rama aun no cambia nada; en curso con rama publicada: {tareas}",
+        en: "note: your branch changes nothing yet; in progress with a published branch: {tareas}",
+    }
 }
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index de3f5d0..914275b 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -16,7 +16,8 @@
 //! [`importar`] (alta en bloque con dependencias e hito, F3-2) y [`grafo`] y
 //! [`siguiente`] (orden de trabajo y `next`, F3-3) y [`situacion`] y
 //! [`roadmap`] (el calculo comun y `roadmap`, F3-4) y [`tablero`]
-//! (`board` y `status`, F3-5) y [`toma`] (`start`, F3-6) y [`ciclo`]
+//! (`board` y `status`, F3-5) y [`toma`] (`start`, F3-6) y [`solape`]
+//! (aviso de solape de `start`, F3-11) y [`ciclo`]
 //! (`descartar`, `pausa` y `reanudar`, F3-7; `liberar` y `reasignar`,
 //! F3-10) y [`doctor`] (F3-8). Las capas
 //! que vienen entran como modulos de esta biblioteca.
@@ -47,6 +48,7 @@ pub mod recursos;
 pub mod roadmap;
 pub mod siguiente;
 pub mod situacion;
+pub mod solape;
 pub mod tablero;
 pub mod tarea;
 pub mod toma;
@@ -163,7 +165,13 @@ where
         }
         [orden, resto @ ..] if orden == toma::NOMBRE => {
             let (json, resto) = con_json(resto);
-            de_una_tarea(toma::NOMBRE, (json, idioma), |e| toma::orden(&resto, e))
+            let mut solape = None;
+            let s = de_una_tarea(toma::NOMBRE, (json, idioma), |e| {
+                let (i, s) = toma::orden_con_solape(&resto, e);
+                solape = s;
+                i
+            });
+            con_aviso_de_solape(s, solape.as_ref(), idioma)
         }
         [orden, resto @ ..] if orden == ciclo::DESCARTAR => {
             let (json, resto) = con_json(resto);
@@ -236,6 +244,20 @@ fn con_aviso(mut s: Salida, parecidas: &[alta::Parecida], idioma: Idioma) -> Sal
     s
 }
 
+/// El aviso de solape de `start` (F3-11) va a stderr con codigo 0, como el
+/// de titulos parecidos: solo para la persona; el canal fiable son las
+/// claves `solape*` de la linea y del JSON. Nunca cambia codigo ni stdout.
+fn con_aviso_de_solape(mut s: Salida, solape: Option<&solape::Solape>, idioma: Idioma) -> Salida {
+    if s.codigo != 0 {
+        return s;
+    }
+    for a in solape.map(solape::Solape::avisos).unwrap_or_default() {
+        s.stderr.push_str(&a.texto(idioma));
+        s.stderr.push('\n');
+    }
+    s
+}
+
 /// Quita `--json` de donde este y dice si estaba.
 fn con_json(args: &[String]) -> (bool, Vec<String>) {
     let json = args.iter().any(|a| a == "--json");
diff --git a/crates/tc2/src/solape.rs b/crates/tc2/src/solape.rs
new file mode 100644
index 0000000..831d00c
--- /dev/null
+++ b/crates/tc2/src/solape.rs
@@ -0,0 +1,613 @@
+//! Aviso de solape entre tareas en curso (F3-11; ADR F1-17, punto 11 y su
+//! adenda de TASK-082).
+//!
+//! Sin zonas declaradas (llegan con el plan de F4-1), el solape sale de lo
+//! que ya tocan las ramas: el diff real de la rama propia frente al de cada
+//! otra tarea en curso o en revision con la rama publicada en `origin`.
+//! Dos tareas solapan si tocan el mismo fichero o el mismo **modulo**, que
+//! es la carpeta padre del fichero (los de la raiz no tienen modulo).
+//!
+//! Es siempre un aviso: nunca cambia el resultado de `start` (F1-17, D4a).
+//!
+//! Punto de extension de F4-1: [`Toca`] ganara las `zonas` (globs) de la
+//! tarea y [`cruzar`] la regla de prefijo de F1-17 (punto 11).
+//!
+//! [`calcular`] corre tras el `fetch` de `start` y sin red: un
+//! `for-each-ref` de `refs/remotes/origin/`, el `diff` de la rama propia y
+//! uno por cada otra tarea con rama publicada (1 + k + 1 procesos de git).
+
+use std::collections::BTreeSet;
+use std::path::Path;
+
+use crate::contrato::Informe;
+use crate::git::{self, Git};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::toma as txt;
+use crate::plataforma::Personas;
+use crate::situacion::{self, Ficha};
+use crate::tarea::Documento;
+
+/// Estados que cuentan para el solape (D2a de TASK-082): las tareas que
+/// alguien esta cambiando o que esperan revision con sus cambios en la rama.
+pub const ESTADOS: [&str; 2] = ["en-curso", "en-revision"];
+
+/// Una tarea en curso o en revision, como la proyecta [`Ficha::de`] (la
+/// misma lectura de `board`, `status` y `roadmap`).
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct EnCurso {
+    /// `TASK-NNN`.
+    pub id: String,
+    /// Quien la lleva, como lo ensenan las vistas (`@login`, el texto
+    /// heredado o `—`).
+    pub quien: String,
+    /// Su rama, si la tiene.
+    pub rama: Option<String>,
+}
+
+/// Las tareas en curso o en revision de unos `tarea.md` leidos, por ID y
+/// sin repetir (con un ID repetido vale la primera, como en el ROADMAP).
+pub fn en_curso<'a>(docs: impl IntoIterator<Item = &'a Documento>) -> Vec<EnCurso> {
+    let sin_lista = Personas::default();
+    let mut v: Vec<EnCurso> = docs
+        .into_iter()
+        .map(Ficha::de)
+        .filter(|f| f.nodo.estado().is_some_and(|e| ESTADOS.contains(&e)))
+        .map(|f| EnCurso {
+            quien: situacion::quien(f.asignado.as_ref(), &sin_lista).texto(),
+            id: f.nodo.id,
+            rama: f.rama,
+        })
+        .collect();
+    v.sort_by(|a, b| a.id.cmp(&b.id));
+    v.dedup_by(|b, a| a.id == b.id);
+    v
+}
+
+/// Lo que toca otra tarea: su ID, quien la lleva y los ficheros de su rama.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Toca {
+    /// `TASK-NNN`.
+    pub id: String,
+    /// Quien la lleva (texto de las vistas).
+    pub quien: String,
+    /// Ficheros que cambia su rama desde su base.
+    pub ficheros: BTreeSet<String>,
+}
+
+/// Modulo de un fichero (D1a de TASK-082): su carpeta padre. Un fichero de
+/// la raiz no tiene modulo. Las rutas son las de `git diff`, siempre con
+/// `/`: una `\` es parte del nombre, no un separador.
+pub fn modulo(ruta: &str) -> Option<&str> {
+    let (padre, _) = ruta.rsplit_once('/')?;
+    (!padre.is_empty()).then_some(padre)
+}
+
+fn modulos(ficheros: &BTreeSet<String>) -> BTreeSet<&str> {
+    ficheros.iter().filter_map(|f| modulo(f)).collect()
+}
+
+/// El solape de la tarea propia con otra.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Cruce {
+    /// `TASK-NNN` de la otra.
+    pub id: String,
+    /// Quien la lleva (texto de las vistas).
+    pub quien: String,
+    /// Ficheros que tocan las dos, en orden.
+    pub ficheros: Vec<String>,
+    /// Modulos que tocan las dos sin compartir fichero en ellos, en orden.
+    pub modulos: Vec<String>,
+}
+
+/// Los cruces de los ficheros `propia` con cada una de `otras`, por ID. Un
+/// fichero comun es un cruce de fichero; un modulo comun en el que no hay
+/// fichero comun, un cruce de modulo (asi se distingue «tocamos lo mismo» de
+/// «tocamos la misma carpeta»). Sin nada comun, la otra no sale.
+pub fn cruzar(propia: &BTreeSet<String>, otras: &[Toca]) -> Vec<Cruce> {
+    let mios = modulos(propia);
+    let mut v: Vec<Cruce> = otras
+        .iter()
+        .filter_map(|o| {
+            let ficheros: Vec<String> = propia.intersection(&o.ficheros).cloned().collect();
+            let ya: BTreeSet<&str> = ficheros.iter().filter_map(|f| modulo(f)).collect();
+            let modulos: Vec<String> = mios
+                .intersection(&modulos(&o.ficheros))
+                .filter(|m| !ya.contains(*m))
+                .map(|m| (*m).to_owned())
+                .collect();
+            (!ficheros.is_empty() || !modulos.is_empty()).then(|| Cruce {
+                id: o.id.clone(),
+                quien: o.quien.clone(),
+                ficheros,
+                modulos,
+            })
+        })
+        .collect();
+    v.sort_by(|a, b| a.id.cmp(&b.id));
+    v
+}
+
+/// Nombre de la operacion en los informes de error de git (no se ven: un
+/// fallo del solape solo marca `solape-incompleto`).
+const OP: &str = "solape";
+
+/// Elementos como mucho de cada lista de la linea (D5a de TASK-082); si se
+/// corta alguna, `solape-truncado=si`.
+pub const MAX_LISTA: usize = 20;
+
+/// Modulos como mucho por tarea en `en-curso-modulos` y en la nota (D3b).
+pub const MAX_MODULOS_NOTA: usize = 5;
+
+/// Lo que sale del solape de una tarea.
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct Solape {
+    /// Las otras tareas que tocan lo mismo, por ID.
+    pub cruces: Vec<Cruce>,
+    /// Las otras tareas con la rama publicada y su diff leido, por ID.
+    pub publicadas: Vec<Toca>,
+    /// Las otras tareas sin `origin/<rama>` (o sin `rama:`): no se sabe que
+    /// tocan.
+    pub sin_rama: Vec<String>,
+    /// Las tareas cuyo `diff` fallo (la propia incluida): su solape no se
+    /// sabe.
+    pub incompletas: Vec<String>,
+    /// La rama propia aun no cambia nada (primer `start`, D3b): en vez de
+    /// cruces, la nota con lo que tocan las demas.
+    pub propia_vacia: bool,
+}
+
+/// El solape de la tarea `id` (rama `rama`, ya creada en local) con las
+/// tareas `en_curso`, desde la carpeta `dir` del repo. Sin red: lo que
+/// sabe el clon tras el `fetch` de `start`.
+///
+/// `None` (ninguna clave) si no hay otra tarea en curso o el clon no conoce
+/// ninguna rama de `origin` (sin remoto o sin haberlo traido nunca). Un
+/// `diff` que falla no rompe nada: la tarea queda en `incompletas`.
+pub fn calcular(g: &Git, dir: &Path, id: &str, rama: &str, en_curso: &[EnCurso]) -> Option<Solape> {
+    let otras: Vec<&EnCurso> = en_curso.iter().filter(|t| t.id != id).collect();
+    if otras.is_empty() {
+        return None;
+    }
+    let mut s = Solape::default();
+    let remotas = match git::fechas_de_ramas(g, dir, OP) {
+        Ok(r) if r.is_empty() => return None,
+        Ok(r) => r,
+        Err(_) => {
+            s.incompletas = otras.iter().map(|t| t.id.clone()).collect();
+            return Some(s);
+        }
+    };
+    let propia = git::ficheros_de_rama(
+        g,
+        dir,
+        OP,
+        git::base_de_rama(rama),
+        &format!("refs/heads/{rama}"),
+    )
+    .ok();
+    if propia.is_none() {
+        s.incompletas.push(id.to_owned());
+    }
+    for t in otras {
+        let Some(r) = t.rama.as_deref().filter(|r| remotas.contains_key(*r)) else {
+            s.sin_rama.push(t.id.clone());
+            continue;
+        };
+        let remota = format!("refs/remotes/{}/{r}", git::REMOTO);
+        match git::ficheros_de_rama(g, dir, OP, git::base_de_rama(r), &remota) {
+            Ok(ficheros) => s.publicadas.push(Toca {
+                id: t.id.clone(),
+                quien: t.quien.clone(),
+                ficheros,
+            }),
+            Err(_) => s.incompletas.push(t.id.clone()),
+        }
+    }
+    s.incompletas.sort();
+    match propia {
+        Some(p) if p.is_empty() => s.propia_vacia = true,
+        Some(p) => s.cruces = cruzar(&p, &s.publicadas),
+        None => {}
+    }
+    Some(s)
+}
+
+/// `TASK-NNN:<valor>`: un elemento de una lista de la linea. `:` no se
+/// escapa en un valor de lista (F1-4, decision 4: solo espacio, `%`,
+/// controles y `,`) y el ID no lo lleva, asi que el primer `:` separa.
+fn de(id: &str, valor: &str) -> String {
+    format!("{id}:{valor}")
+}
+
+/// Anade la lista `clave` si no esta vacia, cortada a [`MAX_LISTA`].
+fn lista(i: Informe, clave: &str, mut v: Vec<String>, truncado: &mut bool) -> Informe {
+    if v.is_empty() {
+        return i;
+    }
+    if v.len() > MAX_LISTA {
+        v.truncate(MAX_LISTA);
+        *truncado = true;
+    }
+    i.con_lista(clave, v)
+}
+
+/// Los modulos de una tarea para la nota, como mucho [`MAX_MODULOS_NOTA`].
+fn modulos_nota(t: &Toca) -> Vec<&str> {
+    modulos(&t.ficheros)
+        .into_iter()
+        .take(MAX_MODULOS_NOTA)
+        .collect()
+}
+
+impl Solape {
+    /// Las claves del solape en la linea de `start` (D5a de TASK-082):
+    /// `solape`, `solape-ficheros`, `solape-modulos` y `solape-asignados`
+    /// si cruza con alguna; `en-curso-modulos` en el primer `start` (D3b);
+    /// `solape-sin-rama` y `solape-incompleto` si las hay; y
+    /// `solape-truncado=si` si se corto alguna lista. Una lista vacia no
+    /// sale.
+    pub fn con_claves(&self, mut i: Informe) -> Informe {
+        let mut t = false;
+        let c = &self.cruces;
+        i = lista(
+            i,
+            "solape",
+            c.iter().map(|c| c.id.clone()).collect(),
+            &mut t,
+        );
+        let ficheros = c
+            .iter()
+            .flat_map(|c| c.ficheros.iter().map(|f| de(&c.id, f)))
+            .collect();
+        i = lista(i, "solape-ficheros", ficheros, &mut t);
+        let modulos = c
+            .iter()
+            .flat_map(|c| c.modulos.iter().map(|m| de(&c.id, m)))
+            .collect();
+        i = lista(i, "solape-modulos", modulos, &mut t);
+        let asignados = c.iter().map(|c| de(&c.id, &c.quien)).collect();
+        i = lista(i, "solape-asignados", asignados, &mut t);
+        if self.propia_vacia {
+            let en_curso = self
+                .publicadas
+                .iter()
+                .flat_map(|p| modulos_nota(p).into_iter().map(|m| de(&p.id, m)))
+                .collect();
+            i = lista(i, "en-curso-modulos", en_curso, &mut t);
+        }
+        i = lista(i, "solape-sin-rama", self.sin_rama.clone(), &mut t);
+        i = lista(i, "solape-incompleto", self.incompletas.clone(), &mut t);
+        if t { i.con("solape-truncado", "si") } else { i }
+    }
+
+    /// Lo que se dice a la persona por stderr (D3b y D4a de TASK-082): una
+    /// linea `aviso:` por tarea que cruza y, en el primer `start`, una
+    /// `nota:` con las tareas en curso con rama publicada, su @ y sus
+    /// modulos. Login, rutas e IDs son texto ajeno: van saneados (F1-17,
+    /// punto 8). Sin nada que decir, vacio.
+    pub fn avisos(&self) -> Vec<Mensaje> {
+        let mut v: Vec<Mensaje> = self
+            .cruces
+            .iter()
+            .map(|c| {
+                let ficheros = resumen(c.ficheros.iter().map(String::as_str));
+                let modulos = resumen(c.modulos.iter().map(String::as_str));
+                let que = match (c.ficheros.is_empty(), c.modulos.is_empty()) {
+                    (false, true) => {
+                        Mensaje::nuevo(&txt::SOLAPE_FICHEROS).con("ficheros", ficheros)
+                    }
+                    (true, _) => Mensaje::nuevo(&txt::SOLAPE_MODULOS).con("modulos", modulos),
+                    (false, false) => Mensaje::nuevo(&txt::SOLAPE_FICHEROS_Y_MODULOS)
+                        .con("ficheros", ficheros)
+                        .con("modulos", modulos),
+                };
+                Mensaje::nuevo(&txt::AVISO_SOLAPE)
+                    .con("id", sanear(&c.id, MAX_ID))
+                    .con("quien", sanear(&c.quien, MAX_QUIEN))
+                    .con_mensaje("que", que)
+            })
+            .collect();
+        if self.propia_vacia && !self.publicadas.is_empty() {
+            let tareas: Vec<String> = self
+                .publicadas
+                .iter()
+                .map(|p| {
+                    let id = sanear(&p.id, MAX_ID);
+                    let quien = sanear(&p.quien, MAX_QUIEN);
+                    match modulos_nota(p) {
+                        m if m.is_empty() => format!("{id} ({quien})"),
+                        m => format!("{id} ({quien}: {})", resumen(m.into_iter())),
+                    }
+                })
+                .collect();
+            v.push(Mensaje::nuevo(&txt::NOTA_EN_CURSO).con("tareas", tareas.join("; ")));
+        }
+        v
+    }
+}
+
+/// Caracteres como mucho de un ID, un quien y una ruta en el aviso.
+const MAX_ID: usize = 20;
+const MAX_QUIEN: usize = 40;
+const MAX_RUTA: usize = 120;
+
+/// Elementos como mucho de una lista en el aviso; el resto, `+N`.
+const MAX_EN_AVISO: usize = 5;
+
+fn sanear(t: &str, max: usize) -> String {
+    crate::tablero::sanear(t, max)
+}
+
+/// Las rutas de un aviso: saneadas, separadas por comas y, pasadas
+/// [`MAX_EN_AVISO`], `+N`.
+fn resumen<'a>(v: impl Iterator<Item = &'a str>) -> String {
+    let todas: Vec<&str> = v.collect();
+    let mut s: Vec<String> = todas
+        .iter()
+        .take(MAX_EN_AVISO)
+        .map(|r| sanear(r, MAX_RUTA))
+        .collect();
+    if todas.len() > MAX_EN_AVISO {
+        s.push(format!("+{}", todas.len() - MAX_EN_AVISO));
+    }
+    s.join(", ")
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::tarea::{Asignado, Cuerpo, Tarea};
+
+    fn conjunto(v: &[&str]) -> BTreeSet<String> {
+        v.iter().map(|s| (*s).to_owned()).collect()
+    }
+
+    fn toca(id: &str, ficheros: &[&str]) -> Toca {
+        Toca {
+            id: id.to_owned(),
+            quien: format!("@{}", id.to_lowercase()),
+            ficheros: conjunto(ficheros),
+        }
+    }
+
+    #[test]
+    fn modulo_es_la_carpeta_padre() {
+        assert_eq!(modulo("src/x/a.rs"), Some("src/x"));
+        assert_eq!(modulo("a/b/c/d.txt"), Some("a/b/c"));
+        assert_eq!(modulo("crates/tc2/src/lib.rs"), Some("crates/tc2/src"));
+        // La raiz no tiene modulo.
+        assert_eq!(modulo("Cargo.toml"), None);
+        assert_eq!(modulo("README.md"), None);
+        assert_eq!(modulo("/raro"), None);
+        // `\` no separa: git escribe siempre `/`.
+        assert_eq!(modulo("src\\x\\a.rs"), None);
+        assert_eq!(modulo("src/x\\a.rs"), Some("src"));
+    }
+
+    /// C1, C2, C3 y C8 (unitarios).
+    #[test]
+    fn cruzar_por_fichero_por_modulo_o_nada() {
+        let propia = conjunto(&["src/x/a.rs"]);
+        // C1: el mismo fichero; su modulo no se repite como cruce de modulo.
+        let c = cruzar(&propia, &[toca("TASK-001", &["src/x/a.rs", "src/x/c.rs"])]);
+        assert_eq!(c.len(), 1);
+        assert_eq!(c[0].ficheros, ["src/x/a.rs"]);
+        assert!(c[0].modulos.is_empty(), "{c:?}");
+        // C2: otro fichero en la misma carpeta.
+        let c = cruzar(&propia, &[toca("TASK-001", &["src/x/b.rs"])]);
+        assert_eq!(c[0].ficheros, Vec::<String>::new());
+        assert_eq!(c[0].modulos, ["src/x"]);
+        // C3: nada en comun.
+        assert!(cruzar(&propia, &[toca("TASK-001", &["web/y/c.ts"])]).is_empty());
+        // Una carpeta hija no es el mismo modulo.
+        assert!(cruzar(&propia, &[toca("TASK-001", &["src/x/z/b.rs"])]).is_empty());
+        // C8: en la raiz no hay modulo; el mismo fichero si cruza.
+        let raiz = conjunto(&["Cargo.toml"]);
+        assert!(cruzar(&raiz, &[toca("TASK-001", &["README.md"])]).is_empty());
+        let c = cruzar(&raiz, &[toca("TASK-001", &["Cargo.toml"])]);
+        assert_eq!(c[0].ficheros, ["Cargo.toml"]);
+        assert!(c[0].modulos.is_empty());
+        // Sin ficheros propios, nada.
+        assert!(cruzar(&BTreeSet::new(), &[toca("TASK-001", &["src/x/a.rs"])]).is_empty());
+    }
+
+    /// Varias tareas: salen por ID, solo las que cruzan, cada una con lo
+    /// suyo; un modulo con fichero comun no sale como modulo, otro si.
+    #[test]
+    fn cruzar_varias_en_orden_estable() {
+        let propia = conjunto(&["src/x/a.rs", "src/y/b.rs", "docs/c.md"]);
+        let otras = [
+            toca("TASK-009", &["src/y/otro.rs"]),
+            toca("TASK-002", &["web/z.ts"]),
+            toca("TASK-004", &["src/x/a.rs", "src/y/b.rs", "docs/d.md"]),
+        ];
+        let c = cruzar(&propia, &otras);
+        let ids: Vec<&str> = c.iter().map(|c| c.id.as_str()).collect();
+        assert_eq!(ids, ["TASK-004", "TASK-009"]);
+        assert_eq!(c[0].ficheros, ["src/x/a.rs", "src/y/b.rs"]);
+        assert_eq!(c[0].modulos, ["docs"]);
+        assert_eq!(c[0].quien, "@task-004");
+        assert_eq!(c[1].modulos, ["src/y"]);
+        // Mismo resultado con las otras en otro orden.
+        let mut al_reves = otras.to_vec();
+        al_reves.reverse();
+        assert_eq!(cruzar(&propia, &al_reves), c);
+    }
+
+    fn doc(id: &str, estado: &str, asignado: Option<Asignado>, rama: Option<&str>) -> Documento {
+        let mut t = Tarea::nueva(id, "Una", "2026-11-01");
+        t.estado = estado.to_owned();
+        t.asignado_a = asignado;
+        t.rama = rama.map(str::to_owned);
+        Documento::Formato2(t, Cuerpo::plantilla())
+    }
+
+    fn cruce(id: &str, quien: &str, ficheros: &[&str], modulos: &[&str]) -> Cruce {
+        Cruce {
+            id: id.to_owned(),
+            quien: quien.to_owned(),
+            ficheros: ficheros.iter().map(|s| (*s).to_owned()).collect(),
+            modulos: modulos.iter().map(|s| (*s).to_owned()).collect(),
+        }
+    }
+
+    fn linea(s: &Solape) -> String {
+        s.con_claves(Informe::nuevo("start", crate::contrato::Resultado::Ok))
+            .linea()
+    }
+
+    /// C1 y C2 (unitarios): las claves planas `TASK-NNN:valor`, solo las
+    /// listas con algo; sin cruces ni nada mas, ninguna clave (C3).
+    #[test]
+    fn claves_del_solape() {
+        let s = Solape {
+            cruces: vec![
+                cruce("TASK-001", "@ana", &["src/x/a.rs"], &[]),
+                cruce("TASK-003", "@beto", &[], &["web/y"]),
+            ],
+            sin_rama: vec!["TASK-004".to_owned()],
+            ..Solape::default()
+        };
+        assert_eq!(
+            linea(&s),
+            "start ok solape=TASK-001,TASK-003 solape-ficheros=TASK-001:src/x/a.rs \
+             solape-modulos=TASK-003:web/y solape-asignados=TASK-001:@ana,TASK-003:@beto \
+             solape-sin-rama=TASK-004"
+        );
+        assert_eq!(linea(&Solape::default()), "start ok");
+        // `en-curso-modulos` solo con la rama propia vacia (D3b), como
+        // mucho 5 por tarea.
+        let muchos: Vec<String> = (0..7).map(|n| format!("m{n}/f.rs")).collect();
+        let p = Toca {
+            id: "TASK-002".to_owned(),
+            quien: "@luis".to_owned(),
+            ficheros: muchos.into_iter().chain(["raiz.md".to_owned()]).collect(),
+        };
+        let mut s = Solape {
+            publicadas: vec![p],
+            ..Solape::default()
+        };
+        assert_eq!(linea(&s), "start ok");
+        s.propia_vacia = true;
+        assert_eq!(
+            linea(&s),
+            "start ok en-curso-modulos=TASK-002:m0,TASK-002:m1,TASK-002:m2,TASK-002:m3,TASK-002:m4"
+        );
+        let n = s.avisos();
+        assert_eq!(n.len(), 1);
+        assert_eq!(
+            n[0].texto(crate::i18n::Idioma::Es),
+            "nota: tu rama aun no cambia nada; en curso con rama publicada: \
+             TASK-002 (@luis: m0, m1, m2, m3, m4)"
+        );
+    }
+
+    /// D5a: cada lista, como mucho 20, y `solape-truncado=si` si se corta.
+    #[test]
+    fn listas_cortadas_a_veinte() {
+        let ficheros: Vec<String> = (0..25).map(|n| format!("f{n:02}.rs")).collect();
+        let refs: Vec<&str> = ficheros.iter().map(String::as_str).collect();
+        let s = Solape {
+            cruces: vec![cruce("TASK-001", "@ana", &refs, &[])],
+            ..Solape::default()
+        };
+        let i = s.con_claves(Informe::nuevo("start", crate::contrato::Resultado::Ok));
+        let l = i.linea();
+        let lista = l
+            .split(' ')
+            .find_map(|kv| kv.strip_prefix("solape-ficheros="))
+            .unwrap_or_default();
+        assert_eq!(lista.split(',').count(), MAX_LISTA, "{l}");
+        assert_eq!(i.valor("solape-truncado"), Some("si"), "{l}");
+        let corta = Solape {
+            cruces: vec![cruce("TASK-001", "@ana", &refs[..MAX_LISTA], &[])],
+            ..Solape::default()
+        };
+        assert_eq!(linea(&corta).find("solape-truncado"), None);
+        // En el aviso, 5 y `+N`.
+        let a = s.avisos()[0].texto(crate::i18n::Idioma::Es);
+        assert!(
+            a.contains("(f00.rs, f01.rs, f02.rs, f03.rs, f04.rs, +20)"),
+            "{a}"
+        );
+    }
+
+    /// C1 (unitario) y C11: el aviso nombra la tarea, quien la lleva y lo
+    /// que toca; un login, una ruta o un ID con caracteres de control (o de
+    /// formato) salen saneados, en los dos idiomas. En la linea, el contrato
+    /// los escapa `%XX`.
+    #[test]
+    fn avisos_saneados() {
+        let s = Solape {
+            cruces: vec![
+                cruce("TASK-001", "@ana", &["src/x/a.rs"], &[]),
+                cruce(
+                    "TASK-00\u{1b}[31m2",
+                    "@lu\u{7}is\u{202e}",
+                    &["src/\u{1b}]8;;x\u{7}b.rs"],
+                    &["web/\r\ny"],
+                ),
+            ],
+            ..Solape::default()
+        };
+        let a = s.avisos();
+        assert_eq!(a.len(), 2);
+        let es = a[0].texto(crate::i18n::Idioma::Es);
+        assert_eq!(
+            es,
+            "aviso: TASK-001 (@ana) toca los mismos ficheros (src/x/a.rs): hablad antes de \
+             seguir (solo es un aviso; start ya se hizo)"
+        );
+        for idioma in [crate::i18n::Idioma::Es, crate::i18n::Idioma::En] {
+            let t = a[1].texto(idioma);
+            assert!(
+                !t.chars().any(|c| c.is_control() || c == '\u{202e}'),
+                "{t:?}"
+            );
+            assert!(t.contains("@lu is") && t.contains("web/ y"), "{t:?}");
+        }
+        let l = linea(&s);
+        assert!(!l.chars().any(char::is_control), "{l:?}");
+        assert!(l.contains("TASK-00%1B[31m2:@lu%07is"), "{l}");
+    }
+
+    /// D2a: cuentan en curso y en revision (C7: una terminada no), con quien
+    /// las lleva como en las vistas y por ID.
+    #[test]
+    fn en_curso_filtra_por_estado() {
+        let docs = [
+            doc("TASK-003", "en-revision", None, Some("feature/task-003-x")),
+            doc(
+                "TASK-001",
+                "en-curso",
+                Asignado::persona(7, "ana"),
+                Some("feature/task-001-x"),
+            ),
+            doc(
+                "TASK-002",
+                "terminada",
+                Asignado::persona(7, "ana"),
+                Some("feature/task-002-x"),
+            ),
+            doc("TASK-004", "planificada", None, None),
+            doc("TASK-005", "en-diseno", None, None),
+            doc("TASK-001", "en-curso", Asignado::persona(8, "beto"), None),
+        ];
+        let v = en_curso(docs.iter());
+        assert_eq!(
+            v,
+            [
+                EnCurso {
+                    id: "TASK-001".to_owned(),
+                    quien: "@ana".to_owned(),
+                    rama: Some("feature/task-001-x".to_owned()),
+                },
+                EnCurso {
+                    id: "TASK-003".to_owned(),
+                    quien: "—".to_owned(),
+                    rama: Some("feature/task-003-x".to_owned()),
+                },
+            ]
+        );
+    }
+}
diff --git a/crates/tc2/src/toma/mod.rs b/crates/tc2/src/toma/mod.rs
index dcaeaeb..3b7367f 100644
--- a/crates/tc2/src/toma/mod.rs
+++ b/crates/tc2/src/toma/mod.rs
@@ -23,6 +23,11 @@
 //!
 //! Lanzada desde un worktree de tarea escribe el estado en el principal (lo
 //! localiza la transaccion) y crea el worktree junto al principal.
+//!
+//! Con la rama y el worktree listos, el aviso de solape (F3-11,
+//! [`crate::solape`]): que otras tareas en curso tocan lo mismo. Solo anade
+//! claves a la linea (y el aviso de stderr en `lib.rs`); nunca cambia el
+//! resultado ni el codigo.
 
 pub mod operacion;
 
@@ -34,6 +39,7 @@ use crate::git::{self, Git, Subcomando};
 use crate::i18n::Mensaje;
 use crate::i18n::catalogo::toma as txt;
 use crate::identidad::Quien;
+use crate::solape::{self, EnCurso, Solape};
 
 pub use operacion::{Tomar, desde_anotacion, instante};
 
@@ -61,25 +67,37 @@ pub struct Entorno<'a> {
 
 /// `start TASK-NNN` (sin `--json`, que quita `lib.rs`).
 pub fn orden(args: &[String], e: &Entorno<'_>) -> Informe {
+    orden_con_solape(args, e).0
+}
+
+/// Como [`orden`], y el solape calculado si la toma salio bien, para el
+/// aviso de stderr (F3-11).
+pub fn orden_con_solape(args: &[String], e: &Entorno<'_>) -> (Informe, Option<Solape>) {
     let uso = |motivo: &str| {
         Informe::nuevo(NOMBRE, Resultado::Uso)
             .con("motivo", motivo)
             .diagnostico(&txt::USO_START)
     };
     match args {
-        [id] if git::es_id(id) => start_en(e, id),
-        [_] => uso("id-invalido"),
-        [] => uso("sin-id"),
-        _ => uso("argumentos-de-mas"),
+        [id] if git::es_id(id) => start_con_solape(e, id),
+        [_] => (uso("id-invalido"), None),
+        [] => (uso("sin-id"), None),
+        _ => (uso("argumentos-de-mas"), None),
     }
 }
 
 /// `start` de la tarea `id` en el repo de `e`.
 pub fn start_en(e: &Entorno<'_>, id: &str) -> Informe {
-    let i = match paso(e, id) {
+    start_con_solape(e, id).0
+}
+
+/// Como [`start_en`], y el solape calculado si la toma salio bien.
+pub fn start_con_solape(e: &Entorno<'_>, id: &str) -> (Informe, Option<Solape>) {
+    let mut s = None;
+    let i = match paso(e, id, &mut s) {
         Ok(i) | Err(i) => i,
     };
-    de_start(i, id)
+    (de_start(i, id), s)
 }
 
 /// El informe como de `start`: el comando, `id=` el primero y sin la clave
@@ -119,7 +137,7 @@ fn rechazo_de_toma(mut i: Informe, op: &Tomar) -> Informe {
     i
 }
 
-fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
+fn paso(e: &Entorno<'_>, id: &str, s: &mut Option<Solape>) -> Result<Informe, Informe> {
     let g = e.git;
     let principal = git::localizar_principal(g, e.dir)?;
     // 1. Lo de ahora del remoto: el estado, la base y las ramas.
@@ -171,7 +189,8 @@ fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
     };
     if empezada {
         // 5c. Ya es suya y esta empezada: crear solo lo que falte.
-        return Ok(destino.crear(None, &quien));
+        let i = destino.crear(None, &quien);
+        return Ok(destino.con_solape(i, || en_curso_de(&vista), s));
     }
     // 5. El ID no tiene rama de nadie mas. La suya propia (la de `rama:`)
     // no cuenta: tras `liberar` o `reasignar` (F3-10) la siguiente persona
@@ -212,10 +231,14 @@ fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
             {
                 i = i.con(crate::progreso::CLAVE, crate::progreso::NO_ESCRITO);
             }
-            Ok(i)
+            // El solape (F3-11) con las tareas en curso que leyo la toma.
+            Ok(destino.con_solape(i, || op.en_curso(), s))
         }
         // Entre la lectura y el push paso a ser suya y empezada.
-        Resultado::SinCambios => Ok(destino.crear(None, &quien)),
+        Resultado::SinCambios => {
+            let i = destino.crear(None, &quien);
+            Ok(destino.con_solape(i, || en_curso_de(&vista), s))
+        }
         Resultado::Rechazada => Err(rechazo_de_toma(t, &op)),
         _ => Err(t),
     }
@@ -239,6 +262,16 @@ impl Estado for SinHistorico<'_> {
     }
 }
 
+/// Las tareas en curso o en revision de un estado leido (un solo `git
+/// cat-file --batch` con todos los `tarea.md`). Si la lectura falla, `None`:
+/// sin solape, nunca un fallo de `start`.
+fn en_curso_de(e: &dyn Estado) -> Option<Vec<EnCurso>> {
+    let leidas = estado::tareas(e, &e.rutas()).ok()?;
+    Some(solape::en_curso(
+        leidas.iter().filter_map(|(_, d)| d.as_ref().ok()),
+    ))
+}
+
 /// Base de la rama (ADR F1-3, decision 1; D4 de TASK-033): `origin/develop`;
 /// para `hotfix/`, el ultimo tag (por version) contenido en `origin/main` o,
 /// sin tags, su punta; `release/` no se empieza con `start`.
@@ -335,6 +368,29 @@ impl Destino<'_> {
         s
     }
 
+    /// Anade a `i` las claves del solape (F3-11) y lo deja en `salida`, solo
+    /// si `i` es un exito: la rama propia ya existe en local. `en_curso` se
+    /// evalua solo entonces. Nunca cambia el resultado de `i`.
+    fn con_solape(
+        &self,
+        i: Informe,
+        en_curso: impl FnOnce() -> Option<Vec<EnCurso>>,
+        salida: &mut Option<Solape>,
+    ) -> Informe {
+        if !i.resultado.es_exito() {
+            return i;
+        }
+        let Some(lista) = en_curso() else {
+            return i;
+        };
+        let Some(s) = solape::calcular(self.g, self.principal, self.id, self.rama, &lista) else {
+            return i;
+        };
+        let i = s.con_claves(i);
+        *salida = Some(s);
+        i
+    }
+
     fn crear_paso(&self, nueva: bool) -> Result<Informe, Informe> {
         let (g, p) = (self.g, self.principal);
         let local = git::commit_de(g, p, NOMBRE, &format!("refs/heads/{}", self.rama))?;
diff --git a/crates/tc2/src/toma/operacion.rs b/crates/tc2/src/toma/operacion.rs
index b4ff72b..23205f1 100644
--- a/crates/tc2/src/toma/operacion.rs
+++ b/crates/tc2/src/toma/operacion.rs
@@ -61,6 +61,7 @@ pub struct Tomar {
     rama: RefCell<Option<String>>,
     estimacion: RefCell<Option<Estimacion>>,
     progreso: RefCell<Option<crate::progreso::Progreso>>,
+    en_curso: RefCell<Option<Vec<crate::solape::EnCurso>>>,
 }
 
 impl Tomar {
@@ -82,9 +83,17 @@ impl Tomar {
             rama: RefCell::new(None),
             estimacion: RefCell::new(None),
             progreso: RefCell::new(None),
+            en_curso: RefCell::new(None),
         }
     }
 
+    /// Las tareas en curso o en revision del estado del ultimo intento que
+    /// escribio la toma, de la misma lectura de todos los `tarea.md` que el
+    /// historico (F3-11): el solape de `start` no vuelve a leer el estado.
+    pub fn en_curso(&self) -> Option<Vec<crate::solape::EnCurso>> {
+        self.en_curso.borrow().clone()
+    }
+
     /// El progreso en vivo de la tarea tomada en el ultimo intento (F3-12),
     /// calculado con el mismo historico y los mismos commits que la
     /// estimacion: `start` lo escribe sin volver a leer el estado ni la
@@ -153,6 +162,7 @@ impl Operacion for Tomar {
         *self.rama.borrow_mut() = None;
         *self.estimacion.borrow_mut() = None;
         *self.progreso.borrow_mut() = None;
+        *self.en_curso.borrow_mut() = None;
         let ruta = format!("{}/tarea.md", self.id);
         let (mut t, mut cuerpo) = tarea_de(estado, &self.id)?;
         let rama = t
@@ -211,6 +221,9 @@ impl Operacion for Tomar {
         // La estimacion, recalculada con las terminadas del estado fresco
         // (F3-13, D2a): `calculada` es la hora del `start`.
         let leidas = crate::estado::tareas(estado, &estado.rutas())?;
+        *self.en_curso.borrow_mut() = Some(crate::solape::en_curso(
+            leidas.iter().filter_map(|(_, d)| d.as_ref().ok()),
+        ));
         let h = crate::estimacion::historico(&leidas, &self.commits);
         let e = crate::estimacion::recalcular(&mut t, &cuerpo, &h, self.ahora);
         *self.estimacion.borrow_mut() = Some(e);
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-082/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-082/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-082/tarea.md                                    | 32 ++++++++++++++++++++++++++++++--
 3 files changed, 30 insertions(+), 2 deletions(-)
````

## Tests del diff (2 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/liberar.rs" ":(literal)crates/tc2/tests/solape.rs"
````

````
crates/tc2/tests/liberar.rs |   9 ++-
 crates/tc2/tests/solape.rs  | 581 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 2 files changed, 588 insertions(+), 2 deletions(-)
````
