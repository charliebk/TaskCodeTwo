# Peticion de revision — TASK-029 (ronda 1)

- Tarea: TASK-029 — F3-2 tc2 new e import en bloque con dependencias e hito
- Rama revisada: feature/task-029-f3-2-new-e-import-en-bloque-con-dependen
- Rama base: develop
- Commit revisado (HEAD): 80f72ac09fe9e660606b55f88dc82c61307d5b56
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-029 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-029/tarea.md
- tareas/03-en-revision/TASK-029/planificacion/
- crates/tc2/src/alta.rs
- crates/tc2/src/estado/lectura.rs
- crates/tc2/src/i18n/catalogo/cli.rs
- crates/tc2/src/i18n/catalogo/importar.rs
- crates/tc2/src/i18n/catalogo/mod.rs
- crates/tc2/src/importar/fichero.rs
- crates/tc2/src/importar/mod.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/tarea/cuerpo.rs
- crates/tc2/src/tarea/mod.rs
- crates/tc2/src/tarea/modelo.rs
- crates/tc2/tests/alta.rs
- crates/tc2/tests/importar.rs

## Commits a revisar (git log develop..HEAD)

````
80f72ac docs(TASK-029): autorrevision con las revisoras de rust y calidad
a2232d9 refactor(TASK-029): correcciones de la autorrevision
895658a feat(TASK-029): new acepta los campos de RF-01 y rechaza un enunciado no listo
a5d2fb1 feat(TASK-029): import --simular como plan sobre el estado conocido
17dde3e feat(TASK-029): tc2 import en una sola transaccion
049e8c1 feat(TASK-029): parser puro del fichero de import
76ac492 refactor(TASK-029): nucleo altas comun a new e import
a0f9452 chore(TASK-029): tarea en curso
````

## Diff de tu dominio (13 fichero(s) de 14; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/alta.rs b/crates/tc2/src/alta.rs
index 83cdb3f..0197bb5 100644
--- a/crates/tc2/src/alta.rs
+++ b/crates/tc2/src/alta.rs
@@ -23,10 +23,252 @@ pub const NOMBRE: &str = "new";
 /// Prefijo de las carpetas de tarea en la rama de estado.
 const PREFIJO: &str = "TASK-";
 
+/// Una tarea por crear (RF-01): lo que pone quien la da de alta, con las
+/// dependencias ya resueltas a `TASK-NNN`. La comparten `new` (un lote de
+/// una) e `import` (F3-2): un solo camino para reservar el ID, derivar la
+/// rama y avisar de los titulos parecidos.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Entrada {
+    /// Titulo ([`titulo_valido`]).
+    pub titulo: String,
+    /// Tipo (`tarea::TIPOS`); decide el prefijo de la rama.
+    pub tipo: String,
+    /// Sprint (el hito, D2 de TASK-029).
+    pub sprint: u32,
+    /// Objetivo; vacio deja el de la plantilla.
+    pub objetivo: String,
+    /// Criterios de aceptacion; sin ninguno queda el de la plantilla.
+    pub criterios: Vec<String>,
+    /// Etiquetas.
+    pub etiquetas: Vec<String>,
+    /// Complejidad (`tarea::COMPLEJIDADES`).
+    pub complejidad: Option<String>,
+    /// Dependencias, `TASK-NNN`.
+    pub dependencias: Vec<String>,
+    /// `lote/clave` si llega de un import (D1 de TASK-029).
+    pub lote: Option<String>,
+}
+
+impl Entrada {
+    /// Entrada con solo el titulo y los valores de la plantilla de F1-9.
+    pub fn nueva(titulo: &str) -> Self {
+        Self {
+            titulo: titulo.to_owned(),
+            tipo: TIPO.to_owned(),
+            sprint: 0,
+            objetivo: String::new(),
+            criterios: Vec::new(),
+            etiquetas: Vec::new(),
+            complejidad: None,
+            dependencias: Vec::new(),
+            lote: None,
+        }
+    }
+
+    /// Si el enunciado esta listo (RF-01). Sin objetivo ni criterios es el
+    /// borrador de la plantilla, que `new` admite como hasta F2-4 salvo que
+    /// `exige_criterios` (el import, como la 0.16, pide al menos uno). Con
+    /// algo escrito, no esta listo si no hay criterios, si uno esta vacio o
+    /// repetido, si queda un hueco de la plantilla (`<...>`) o si hay
+    /// caracteres de control (el objetivo admite saltos de linea).
+    pub fn listo(&self, exige_criterios: bool) -> Result<(), NoListo> {
+        let objetivo = self.objetivo.trim();
+        if objetivo.is_empty() && self.criterios.is_empty() && !exige_criterios {
+            return Ok(());
+        }
+        if self.criterios.is_empty() {
+            return Err(NoListo::SinCriterios);
+        }
+        let control = |s: &str, salto: bool| {
+            s.chars().any(|c| {
+                (c.is_control() && !(salto && c == '\n')) || matches!(c, '\u{2028}' | '\u{2029}')
+            })
+        };
+        if control(objetivo, true) || self.criterios.iter().any(|c| control(c, false)) {
+            return Err(NoListo::Control);
+        }
+        let hueco = |s: &str| s.starts_with('<') && s.ends_with('>');
+        if hueco(objetivo) {
+            return Err(NoListo::Plantilla);
+        }
+        let mut vistos = BTreeSet::new();
+        for c in self.criterios.iter().map(|c| c.trim()) {
+            if c.is_empty() {
+                return Err(NoListo::CriterioVacio);
+            }
+            if hueco(c) {
+                return Err(NoListo::Plantilla);
+            }
+            if !vistos.insert(crate::tarea::texto::plegar(c)) {
+                return Err(NoListo::Repetido);
+            }
+        }
+        Ok(())
+    }
+}
+
+/// `motivo=` de una dependencia que no existe en el estado.
+pub(crate) const REFERENCIA_INEXISTENTE: &str = "referencia-inexistente";
+
+/// Una dependencia que no existe es un fallo de la llamada (el fichero o
+/// los argumentos), no del estado: `uso` (2) en vez de `rechazada` (C2).
+pub(crate) fn referencia_a_uso(i: &mut crate::contrato::Informe) {
+    if i.resultado == crate::contrato::Resultado::Rechazada
+        && i.valor("motivo") == Some(REFERENCIA_INEXISTENTE)
+    {
+        i.resultado = crate::contrato::Resultado::Uso;
+    }
+}
+
+/// Un sprint: entero sin signo, sin espacios ni `+` (`--sprint`, `sprint:`).
+pub(crate) fn sprint_de(v: &str) -> Option<u32> {
+    v.bytes()
+        .all(|b| b.is_ascii_digit())
+        .then(|| v.parse().ok())
+        .flatten()
+}
+
+/// `uso` de `new` con `motivo` y `diagnostico`.
+fn uso(motivo: &str, diagnostico: impl Into<Mensaje>) -> crate::contrato::Informe {
+    crate::contrato::Informe::nuevo(NOMBRE, crate::contrato::Resultado::Uso)
+        .con("motivo", motivo)
+        .diagnostico(diagnostico)
+}
+
+/// Lee los argumentos de `new` (sin `--json`): el titulo y los campos de
+/// RF-01 (`--tipo`, `--objetivo`, `--criterio`..., `--etiqueta`...,
+/// `--complejidad`, `--sprint`, `--dependencia`...). Rechaza con `uso` un
+/// valor fuera de su lista y un enunciado que no esta listo
+/// ([`Entrada::listo`]).
+pub fn entrada_de_args(args: &[String]) -> Result<Entrada, crate::contrato::Informe> {
+    use crate::i18n::catalogo::cli as tc;
+    use crate::i18n::catalogo::importar as ti;
+    let invalido = |opcion: &str, valor: &str, permitidos: &str| {
+        uso(
+            "valor-invalido",
+            Mensaje::nuevo(&ti::VALOR_INVALIDO)
+                .con("opcion", opcion)
+                .con("valor", valor)
+                .con("permitidos", permitidos),
+        )
+        .con("opcion", opcion.trim_start_matches('-'))
+    };
+    let mut titulos: Vec<&String> = Vec::new();
+    let mut e = Entrada::nueva("");
+    let mut vistas: BTreeSet<&str> = BTreeSet::new();
+    let mut it = args.iter();
+    while let Some(a) = it.next() {
+        let opcion = a.as_str();
+        if !opcion.starts_with("--") {
+            titulos.push(a);
+            continue;
+        }
+        let repetible = matches!(opcion, "--criterio" | "--etiqueta" | "--dependencia");
+        let conocida = repetible
+            || matches!(
+                opcion,
+                "--tipo" | "--objetivo" | "--complejidad" | "--sprint"
+            );
+        if !conocida || (!repetible && !vistas.insert(opcion)) {
+            return Err(uso("argumentos-de-mas", &ti::USO_NEW_CAMPOS));
+        }
+        let Some(v) = it.next() else {
+            return Err(uso("falta-valor", &ti::USO_NEW_CAMPOS).con("opcion", &opcion[2..]));
+        };
+        match opcion {
+            "--tipo" if crate::tarea::TIPOS.contains(&v.as_str()) => v.clone_into(&mut e.tipo),
+            "--tipo" => return Err(invalido(opcion, v, &crate::tarea::TIPOS.join(", "))),
+            "--complejidad" if crate::tarea::COMPLEJIDADES.contains(&v.as_str()) => {
+                e.complejidad = Some(v.clone());
+            }
+            "--complejidad" => {
+                return Err(invalido(opcion, v, &crate::tarea::COMPLEJIDADES.join(", ")));
+            }
+            "--sprint" => {
+                e.sprint = sprint_de(v).ok_or_else(|| invalido(opcion, v, "0, 1, 2..."))?;
+            }
+            // Un argumento con CRLF (Windows) cuenta como saltos de linea.
+            "--objetivo" => e.objetivo = v.replace("\r\n", "\n"),
+            "--criterio" => e.criterios.push(v.clone()),
+            "--etiqueta" if crate::tarea::login_valido(v) => e.etiquetas.push(v.clone()),
+            "--etiqueta" => {
+                return Err(uso(
+                    "valor-invalido",
+                    Mensaje::nuevo(&ti::ETIQUETA_INVALIDA).con("valor", v.as_str()),
+                )
+                .con("opcion", "etiqueta"));
+            }
+            _ if crate::tarea::id_valido(v) => {
+                if !e.dependencias.contains(v) {
+                    e.dependencias.push(v.clone());
+                }
+            }
+            _ => return Err(invalido(opcion, v, "TASK-NNN")),
+        }
+    }
+    let titulo = match titulos.as_slice() {
+        [t] if titulo_valido(t) => (*t).clone(),
+        [_] => return Err(uso("titulo-invalido", &tc::TITULO_INVALIDO)),
+        [] => return Err(uso("sin-titulo", &tc::USO_NEW)),
+        _ => return Err(uso("argumentos-de-mas", &tc::TITULO_ENTRE_COMILLAS)),
+    };
+    e.titulo = titulo;
+    if let Err(c) = e.listo(false) {
+        return Err(uso(
+            "enunciado-no-listo",
+            Mensaje::nuevo(&ti::NO_LISTO)
+                .con("titulo", e.titulo.as_str())
+                .con_mensaje("causa", c.mensaje()),
+        )
+        .con("causa", c.causa()));
+    }
+    Ok(e)
+}
+
+/// Por que un enunciado no esta listo (RF-01).
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum NoListo {
+    /// Hay enunciado pero ningun criterio.
+    SinCriterios,
+    /// Un criterio vacio.
+    CriterioVacio,
+    /// Queda un hueco de la plantilla.
+    Plantilla,
+    /// Un criterio repetido.
+    Repetido,
+    /// Caracteres de control.
+    Control,
+}
+
+impl NoListo {
+    /// Valor de `causa=` en la linea de contrato.
+    pub fn causa(self) -> &'static str {
+        match self {
+            Self::SinCriterios => "sin-criterios",
+            Self::CriterioVacio => "criterio-vacio",
+            Self::Plantilla => "plantilla",
+            Self::Repetido => "criterio-repetido",
+            Self::Control => "caracter-de-control",
+        }
+    }
+
+    /// La causa explicada, para el diagnostico.
+    pub fn mensaje(self) -> Mensaje {
+        use crate::i18n::catalogo::importar as t;
+        Mensaje::nuevo(match self {
+            Self::SinCriterios => &t::CAUSA_SIN_CRITERIOS,
+            Self::CriterioVacio => &t::CAUSA_CRITERIO_VACIO,
+            Self::Plantilla => &t::CAUSA_PLANTILLA,
+            Self::Repetido => &t::CAUSA_REPETIDO,
+            Self::Control => &t::CAUSA_CONTROL,
+        })
+    }
+}
+
 /// Alta de una tarea: reserva el siguiente ID y escribe su `tarea.md`.
 #[derive(Debug)]
 pub struct Alta {
-    titulo: String,
+    entrada: Entrada,
     fecha: String,
     ocupados: BTreeSet<u32>,
     id: Cell<Option<u32>>,
@@ -41,8 +283,17 @@ impl Alta {
 
     /// Alta con una fecha `AAAA-MM-DD` dada.
     pub fn con_fecha(titulo: &str, fecha: &str) -> Self {
+        Self::de_entrada(Entrada::nueva(titulo), fecha)
+    }
+
+    /// Alta de `entrada` con la fecha de hoy (UTC).
+    pub fn con_entrada(entrada: Entrada) -> Self {
+        Self::de_entrada(entrada, &hoy())
+    }
+
+    fn de_entrada(entrada: Entrada, fecha: &str) -> Self {
         Self {
-            titulo: titulo.to_owned(),
+            entrada,
             fecha: fecha.to_owned(),
             ocupados: BTreeSet::new(),
             id: Cell::new(None),
@@ -60,7 +311,8 @@ impl Alta {
 
     /// Rama derivada del ID del ultimo intento (F2-4b).
     pub fn rama(&self) -> Option<String> {
-        self.id().map(|id| rama_de(&id, &self.titulo))
+        self.id()
+            .map(|id| rama_de(&id, &self.entrada.tipo, &self.entrada.titulo))
     }
 
     /// ID calculado en el ultimo intento (el que publico, si la transaccion
@@ -80,7 +332,7 @@ impl Operacion for Alta {
         NOMBRE
     }
     fn anotacion(&self) -> String {
-        format!("{NOMBRE}|{}", self.titulo)
+        format!("{NOMBRE}|{}", self.entrada.titulo)
     }
     fn exige_red(&self) -> bool {
         // Reserva algo compartido: sin red falla, nunca queda pendiente (D2b).
@@ -88,18 +340,94 @@ impl Operacion for Alta {
     }
     fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
         let rutas = estado.rutas();
-        let n = siguiente_id(&rutas, &self.ocupados)?;
-        self.id.set(Some(n));
-        let id = formato_id(n);
-        comprobar_rama(&id, &self.titulo)?;
+        // Las dependencias tienen que existir en el estado fresco (RF-03).
+        if let Some(d) = self
+            .entrada
+            .dependencias
+            .iter()
+            .find(|d| !rutas.iter().any(|r| *r == format!("{d}/tarea.md")))
+        {
+            return Err(Fallo::rechazada(
+                REFERENCIA_INEXISTENTE,
+                Mensaje::nuevo(&crate::i18n::catalogo::importar::DEPENDENCIA_INEXISTENTE)
+                    .con("referencia", d.as_str()),
+            ));
+        }
+        let ids = reservar(&rutas, &self.ocupados, 1)?;
+        self.id.set(ids.first().copied());
         // Sobre el mismo estado fresco del intento: dos altas a la vez se ven
         // (la segunda en publicar avisa de la primera, TASK-076 C1).
-        *self.parecidas.borrow_mut() = parecidas(estado, &rutas, &self.titulo)?;
-        Ok(vec![Cambio::nuevo(
+        let tareas = tareas(estado, &rutas)?;
+        let a = altas(
+            &tareas,
+            &ids,
+            std::slice::from_ref(&self.entrada),
+            &self.fecha,
+        )?;
+        *self.parecidas.borrow_mut() = a.parecidas;
+        Ok(a.cambios)
+    }
+}
+
+/// Lo que sale de [`altas`]: un `tarea.md` por entrada y las tareas abiertas
+/// que se le parecen.
+#[derive(Debug)]
+pub(crate) struct Altas {
+    /// Un cambio por entrada, en su orden.
+    pub cambios: Vec<Cambio>,
+    /// Tareas abiertas parecidas a alguna entrada, sin repetir, por ID.
+    pub parecidas: Vec<Parecida>,
+}
+
+/// Nucleo puro del alta (F2-4 y F3-2): `entradas[i]` nace con el ID
+/// `ids[i]`. Comprueba la rama derivada de cada una y busca las tareas
+/// abiertas de `tareas` con un titulo parecido.
+pub(crate) fn altas(
+    tareas: &[Tarea],
+    ids: &[u32],
+    entradas: &[Entrada],
+    fecha: &str,
+) -> Result<Altas, Fallo> {
+    let mut cambios = Vec::with_capacity(entradas.len());
+    let mut vistas = BTreeSet::new();
+    let mut parecidas = Vec::new();
+    for (i, e) in entradas.iter().enumerate() {
+        let id = formato_id(
+            ids.get(i)
+                .copied()
+                .ok_or_else(|| Fallo::Error((&crate::i18n::catalogo::estado::SIN_IDS).into()))?,
+        );
+        comprobar_rama(&id, &e.tipo, &e.titulo)?;
+        cambios.push(Cambio::nuevo(
             format!("{id}/tarea.md"),
-            tarea_md(&id, &self.titulo, &self.fecha),
-        )])
+            tarea_de(&id, e, fecha),
+        ));
+        for p in parecidas_a(tareas, &e.titulo) {
+            if vistas.insert(p.id.clone()) {
+                parecidas.push(p);
+            }
+        }
     }
+    parecidas.sort_by(|a: &Parecida, b| a.id.cmp(&b.id));
+    Ok(Altas { cambios, parecidas })
+}
+
+/// `n` IDs consecutivos desde [`siguiente_id`]: los de un lote no dejan
+/// huecos entre si (ADR F1-2, punto 3).
+pub(crate) fn reservar(
+    rutas: &[String],
+    ocupados: &BTreeSet<u32>,
+    n: usize,
+) -> Result<Vec<u32>, Fallo> {
+    let primero = siguiente_id(rutas, ocupados)?;
+    (0..n)
+        .map(|i| {
+            u32::try_from(i)
+                .ok()
+                .and_then(|i| primero.checked_add(i))
+                .ok_or_else(|| Fallo::Error((&crate::i18n::catalogo::estado::SIN_IDS).into()))
+        })
+        .collect()
 }
 
 /// Estados en los que una tarea esta abierta: contra ellas se avisa.
@@ -121,27 +449,35 @@ fn es_tarea_md(ruta: &str) -> bool {
     numero(ruta).is_some() && ruta.split_once('/').is_some_and(|(_, r)| r == "tarea.md")
 }
 
-/// Tareas abiertas cuyo titulo se parece a `titulo`, en orden de ID. Los
-/// `tarea.md` que no se leen o son de formato 1 se saltan sin fallar.
-fn parecidas(estado: &dyn Estado, rutas: &[String], titulo: &str) -> Result<Vec<Parecida>, Fallo> {
+/// Las tareas de formato 2 del estado, en orden de ruta, leidas en un solo
+/// proceso (F2-4b). Los `tarea.md` que no se leen o son de formato 1 se
+/// saltan sin fallar.
+pub(crate) fn tareas(estado: &dyn Estado, rutas: &[String]) -> Result<Vec<Tarea>, Fallo> {
     let mds: Vec<String> = rutas.iter().filter(|r| es_tarea_md(r)).cloned().collect();
-    let mut v = Vec::new();
-    for md in estado.leer_varios(&mds)?.into_iter().flatten() {
-        let Ok(Documento::Formato2(t, _)) = crate::tarea::leer(&md) else {
-            continue;
-        };
-        if !ABIERTOS.contains(&t.estado.as_str()) {
-            continue;
-        }
-        if let Some(p) = parecido(titulo, &t.titulo) {
-            v.push(Parecida {
-                id: t.id,
-                titulo: t.titulo,
+    Ok(estado
+        .leer_varios(&mds)?
+        .into_iter()
+        .flatten()
+        .filter_map(|md| match crate::tarea::leer(&md) {
+            Ok(Documento::Formato2(t, _)) => Some(t),
+            _ => None,
+        })
+        .collect())
+}
+
+/// Tareas abiertas de `tareas` cuyo titulo se parece a `titulo`.
+fn parecidas_a(tareas: &[Tarea], titulo: &str) -> Vec<Parecida> {
+    tareas
+        .iter()
+        .filter(|t| ABIERTOS.contains(&t.estado.as_str()))
+        .filter_map(|t| {
+            parecido(titulo, &t.titulo).map(|p| Parecida {
+                id: t.id.clone(),
+                titulo: t.titulo.clone(),
                 parecido: p,
-            });
-        }
-    }
-    Ok(v)
+            })
+        })
+        .collect()
 }
 
 /// Una linea de aviso con las tareas parecidas, en `idioma`; `None` si no
@@ -218,16 +554,16 @@ pub fn siguiente_id(rutas: &[String], ocupados: &BTreeSet<u32>) -> Result<u32, F
 /// Tipo con el que nace una tarea (plantilla de F1-9, punto 6).
 const TIPO: &str = "feature";
 
-/// Rama de la tarea `id` (ADR F1-3 y F1-9 punto 6): `feature/task-nnn-<slug>`
+/// Rama de la tarea `id` (ADR F1-3 y F1-9 punto 6): `<tipo>/task-nnn-<slug>`
 /// con el slug del titulo (F2-4b).
-pub fn rama_de(id: &str, titulo: &str) -> String {
-    crate::git::nombre_de_rama(id, TIPO, &slug(titulo))
+pub fn rama_de(id: &str, tipo: &str, titulo: &str) -> String {
+    crate::git::nombre_de_rama(id, tipo, &slug(titulo))
 }
 
 /// La rama derivada pasa la validacion de las operaciones de rama: si no,
 /// `start` no podria crearla.
-fn comprobar_rama(id: &str, titulo: &str) -> Result<(), Fallo> {
-    crate::git::rama::rama_valida(NOMBRE, id, TIPO, &slug(titulo))
+fn comprobar_rama(id: &str, tipo: &str, titulo: &str) -> Result<(), Fallo> {
+    crate::git::rama::rama_valida(NOMBRE, id, tipo, &slug(titulo))
         .map(drop)
         .map_err(|i| {
             Fallo::Error(
@@ -238,18 +574,29 @@ fn comprobar_rama(id: &str, titulo: &str) -> Result<(), Fallo> {
         })
 }
 
-/// `tarea.md` de una tarea recien creada: el modelo de F2-5 con la
-/// plantilla de F1-9 (punto 6) y la rama derivada del ID (F2-4b).
+/// `tarea.md` de una tarea recien creada con solo el titulo: el modelo de
+/// F2-5 con la plantilla de F1-9 (punto 6) y la rama derivada del ID (F2-4b).
 pub fn tarea_md(id: &str, titulo: &str, fecha: &str) -> String {
-    let mut t = Tarea::nueva(id, titulo, fecha);
-    // El tipo y la rama salen de la misma constante: no pueden divergir.
-    TIPO.clone_into(&mut t.tipo);
-    t.rama = Some(rama_de(id, titulo));
-    t.escribir(&Cuerpo::plantilla())
+    tarea_de(id, &Entrada::nueva(titulo), fecha)
+}
+
+/// `tarea.md` de `entrada` con el ID `id`: el enunciado en el cuerpo y el
+/// resto en el frontmatter. El tipo y la rama salen del mismo campo: no
+/// pueden divergir.
+fn tarea_de(id: &str, e: &Entrada, fecha: &str) -> String {
+    let mut t = Tarea::nueva(id, &e.titulo, fecha);
+    e.tipo.clone_into(&mut t.tipo);
+    t.sprint = e.sprint;
+    t.etiquetas.clone_from(&e.etiquetas);
+    t.complejidad.clone_from(&e.complejidad);
+    t.dependencias.clone_from(&e.dependencias);
+    t.lote.clone_from(&e.lote);
+    t.rama = Some(rama_de(id, &e.tipo, &e.titulo));
+    t.escribir(&Cuerpo::con_enunciado(&e.objetivo, &e.criterios))
 }
 
 /// Fecha de hoy en UTC, `AAAA-MM-DD`, sin dependencias.
-fn hoy() -> String {
+pub(crate) fn hoy() -> String {
     let segundos = SystemTime::now()
         .duration_since(UNIX_EPOCH)
         .map(|d| d.as_secs())
@@ -357,6 +704,30 @@ mod tests {
                 "dependencias",
             ]
         );
+        // Adenda de TASK-029 a F1-9: una tarea importada lleva ademas `lote`,
+        // la ultima clave conocida; sin lote no se escribe.
+        let mut e = Entrada::nueva("x");
+        e.lote = Some("hito37/F3-2".to_owned());
+        let importada = tarea_de("TASK-009", &e, "2026-11-17");
+        let ultimas: Vec<&str> = importada
+            .lines()
+            .skip(1)
+            .take_while(|l| *l != "---")
+            .filter_map(|l| l.split(':').next())
+            .collect();
+        assert_eq!(ultimas.len(), claves.len() + 4); // + las 3 de `roles` y `lote`
+        assert_eq!(ultimas.last(), Some(&"lote"));
+        assert!(importada.contains("\ndependencias: []\nlote: \"hito37/F3-2\"\n---\n"));
+        let leida = match crate::tarea::leer(&importada) {
+            Ok(Documento::Formato2(t, _)) => Some((t.claves_desconocidas().len(), t.lote)),
+            _ => None,
+        };
+        assert_eq!(leida, Some((0, Some("hito37/F3-2".to_owned()))));
+        for malo in ["sin-barra", "/clave", "lote/"] {
+            let t = importada.replace("hito37/F3-2", malo);
+            assert!(crate::tarea::leer(&t).is_err(), "{malo}");
+        }
+        assert!(!t.contains("\nlote:"));
         assert!(t.contains("id: \"TASK-008\"\n"));
         assert!(t.contains("creado: \"2026-11-15\"\nactualizado: \"2026-11-15\"\n"));
         assert!(
@@ -377,7 +748,7 @@ mod tests {
             ("Задача без латиницы", "feature/task-076-tarea"),
             ("漢字", "feature/task-076-tarea"),
         ] {
-            assert_eq!(rama_de("TASK-076", titulo), rama);
+            assert_eq!(rama_de("TASK-076", TIPO, titulo), rama);
             let md = tarea_md("TASK-076", titulo, "2026-11-16");
             let leida = match crate::tarea::leer(&md) {
                 Ok(crate::tarea::Documento::Formato2(t, _)) => t.rama,
@@ -389,9 +760,145 @@ mod tests {
                 crate::git::rama::rama_valida("t", "TASK-076", TIPO, slug).ok(),
                 Some(rama.to_owned())
             );
-            assert_eq!(comprobar_rama("TASK-076", titulo), Ok(()));
+            assert_eq!(comprobar_rama("TASK-076", TIPO, titulo), Ok(()));
+        }
+        assert_eq!(rama_de("TASK-1000", TIPO, "x"), "feature/task-1000-x");
+    }
+
+    /// TASK-029: con `tipo` como parametro, cada tipo del modelo da una rama
+    /// que pasa `rama_valida`, y el `tarea.md` lleva ese tipo y esa rama.
+    #[test]
+    fn rama_derivada_por_tipo_pasa_rama_valida() {
+        for tipo in crate::tarea::TIPOS {
+            let mut e = Entrada::nueva("Arreglar el login");
+            tipo.clone_into(&mut e.tipo);
+            let rama = rama_de("TASK-029", tipo, &e.titulo);
+            assert_eq!(rama, format!("{tipo}/task-029-arreglar-el-login"));
+            assert_eq!(comprobar_rama("TASK-029", tipo, &e.titulo), Ok(()));
+            let leida = match crate::tarea::leer(&tarea_de("TASK-029", &e, "2026-11-17")) {
+                Ok(Documento::Formato2(t, _)) => Some((t.tipo, t.rama)),
+                _ => None,
+            };
+            assert_eq!(leida, Some((tipo.to_owned(), Some(rama))));
+        }
+    }
+
+    fn args(v: &[&str]) -> Vec<String> {
+        v.iter().map(|s| (*s).to_owned()).collect()
+    }
+
+    /// C9 (unitario): `new` acepta los campos de RF-01 y rechaza con `uso`
+    /// un valor fuera de su lista o un enunciado que no esta listo.
+    #[test]
+    fn new_acepta_los_campos_de_rf_01() {
+        let e = entrada_de_args(&args(&[
+            "--tipo",
+            "fix",
+            "Arreglar el login",
+            "--objetivo",
+            "Que entre",
+            "--criterio",
+            "a",
+            "--criterio",
+            "b",
+            "--etiqueta",
+            "rust",
+            "--complejidad",
+            "media",
+            "--sprint",
+            "3",
+            "--dependencia",
+            "TASK-001",
+            "--dependencia",
+            "TASK-001",
+        ]));
+        let mut esperada = Entrada::nueva("Arreglar el login");
+        "fix".clone_into(&mut esperada.tipo);
+        "Que entre".clone_into(&mut esperada.objetivo);
+        esperada.criterios = vec!["a".into(), "b".into()];
+        esperada.etiquetas = vec!["rust".into()];
+        esperada.complejidad = Some("media".into());
+        esperada.sprint = 3;
+        esperada.dependencias = vec!["TASK-001".into()];
+        assert_eq!(e, Ok(esperada));
+        // Solo el titulo sigue valiendo (borrador de la plantilla, F2-4).
+        assert_eq!(entrada_de_args(&args(&["x"])), Ok(Entrada::nueva("x")));
+        let motivo = |a: &[&str]| {
+            entrada_de_args(&args(a)).err().map(|i| {
+                (
+                    i.valor("motivo").unwrap_or_default().to_owned(),
+                    i.valor("causa").unwrap_or_default().to_owned(),
+                )
+            })
+        };
+        let m = |a: &str, b: &str| Some((a.to_owned(), b.to_owned()));
+        for (a, esperado) in [
+            (
+                vec!["x", "--objetivo", "solo"],
+                m("enunciado-no-listo", "sin-criterios"),
+            ),
+            (
+                vec!["x", "--criterio", "<criterio comprobable>"],
+                m("enunciado-no-listo", "plantilla"),
+            ),
+            (
+                vec![
+                    "x",
+                    "--objetivo",
+                    "<que se quiere y por que>",
+                    "--criterio",
+                    "c",
+                ],
+                m("enunciado-no-listo", "plantilla"),
+            ),
+            (
+                vec!["x", "--criterio", "  "],
+                m("enunciado-no-listo", "criterio-vacio"),
+            ),
+            (
+                vec!["x", "--criterio", "Uno", "--criterio", "uno"],
+                m("enunciado-no-listo", "criterio-repetido"),
+            ),
+            (
+                vec!["x", "--criterio", "a\nb"],
+                m("enunciado-no-listo", "caracter-de-control"),
+            ),
+            (vec!["x", "--tipo", "epica"], m("valor-invalido", "")),
+            (
+                vec!["x", "--complejidad", "enorme"],
+                m("valor-invalido", ""),
+            ),
+            (vec!["x", "--sprint", "-2"], m("valor-invalido", "")),
+            (
+                vec!["x", "--etiqueta", "con espacio"],
+                m("valor-invalido", ""),
+            ),
+            (vec!["x", "--dependencia", "F2-4"], m("valor-invalido", "")),
+            (vec!["x", "--tipo"], m("falta-valor", "")),
+            (
+                vec!["x", "--tipo", "fix", "--tipo", "fix"],
+                m("argumentos-de-mas", ""),
+            ),
+            (vec!["x", "--otra", "v"], m("argumentos-de-mas", "")),
+            (vec!["--criterio", "c"], m("sin-titulo", "")),
+        ] {
+            assert_eq!(motivo(&a), esperado, "{a:?}");
         }
-        assert_eq!(rama_de("TASK-1000", "x"), "feature/task-1000-x");
+        // El objetivo admite varias lineas.
+        assert!(entrada_de_args(&args(&["x", "--objetivo", "a\nb", "--criterio", "c"])).is_ok());
+    }
+
+    #[test]
+    fn reservar_da_ids_consecutivos_sin_huecos() {
+        let r = rutas(&["TASK-004/tarea.md"]);
+        assert_eq!(reservar(&r, &BTreeSet::from([6]), 3), Ok(vec![7, 8, 9]));
+        assert_eq!(reservar(&r, &BTreeSet::new(), 0), Ok(vec![]));
+        let tope = rutas(&[&format!("TASK-{}/tarea.md", u32::MAX - 2)]);
+        assert_eq!(reservar(&tope, &BTreeSet::new(), 2).map(|v| v.len()), Ok(2));
+        assert!(matches!(
+            reservar(&tope, &BTreeSet::new(), 3),
+            Err(Fallo::Error(_))
+        ));
     }
 
     #[test]
diff --git a/crates/tc2/src/estado/lectura.rs b/crates/tc2/src/estado/lectura.rs
index bc02b6b..6ec3d1e 100644
--- a/crates/tc2/src/estado/lectura.rs
+++ b/crates/tc2/src/estado/lectura.rs
@@ -62,6 +62,18 @@ impl<'a> Vista<'a> {
         })
     }
 
+    /// Vista del estado vacio (la rama de estado aun no existe), para
+    /// calcular un plan sin escribir (`import --simular`).
+    pub fn vacia(git: &'a Git, dir: &Path) -> Self {
+        Self {
+            git,
+            dir: dir.to_path_buf(),
+            commit: None,
+            rutas: BTreeSet::new(),
+            encima: BTreeMap::new(),
+        }
+    }
+
     /// Commit sobre el que se lee (`None`: la rama no existe).
     pub fn commit(&self) -> Option<&str> {
         self.commit.as_deref()
diff --git a/crates/tc2/src/i18n/catalogo/cli.rs b/crates/tc2/src/i18n/catalogo/cli.rs
index 6016b01..86f8a72 100644
--- a/crates/tc2/src/i18n/catalogo/cli.rs
+++ b/crates/tc2/src/i18n/catalogo/cli.rs
@@ -10,8 +10,8 @@ crate::i18n::textos! {
         en: "tc2: could not write the output: {error}",
     }
     USO {
-        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
-        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
+        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
+        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
     }
     FALTA_LA_ORDEN {
         es: "tc2: falta la orden. {uso}",
diff --git a/crates/tc2/src/i18n/catalogo/importar.rs b/crates/tc2/src/i18n/catalogo/importar.rs
new file mode 100644
index 0000000..6b7e755
--- /dev/null
+++ b/crates/tc2/src/i18n/catalogo/importar.rs
@@ -0,0 +1,112 @@
+//! Mensajes de `tc2 import` y del enunciado de `tc2 new` (F3-2).
+
+crate::i18n::textos! {
+    USO_IMPORT {
+        es: "uso: import [--json] [--simular] [--sprint <N>] <fichero>",
+        en: "usage: import [--json] [--simular] [--sprint <N>] <file>",
+    }
+    NO_SE_LEE {
+        es: "no se puede leer {ruta}: {error}. No se ha importado nada",
+        en: "cannot read {ruta}: {error}. Nothing was imported",
+    }
+    NO_UTF8 {
+        es: "{ruta} no es texto UTF-8. No se ha importado nada",
+        en: "{ruta} is not UTF-8 text. Nothing was imported",
+    }
+    FICHERO_GRANDE {
+        es: "el fichero pasa de {limite} bytes: partelo en varios lotes. No se ha importado nada",
+        en: "the file is larger than {limite} bytes: split it into several batches. Nothing was imported",
+    }
+    DEMASIADAS_ENTRADAS {
+        es: "el fichero tiene mas de {limite} entradas: partelo en varios lotes. No se ha importado nada",
+        en: "the file has more than {limite} entries: split it into several batches. Nothing was imported",
+    }
+    SIN_ENTRADAS {
+        es: "el fichero no tiene ninguna entrada «### <titulo>». No se ha importado nada",
+        en: "the file has no «### <title>» entry. Nothing was imported",
+    }
+    CABECERA_SIN_CERRAR {
+        es: "la cabecera que abre «---» en la linea 1 no se cierra con otra linea «---». No se ha importado nada",
+        en: "the header opened by «---» on line 1 is not closed by another «---» line. Nothing was imported",
+    }
+    CABECERA_INVALIDA {
+        es: "linea {linea} de la cabecera: solo valen «lote: <nombre>» y «sprint: <numero>», una vez cada una. No se ha importado nada",
+        en: "header line {linea}: only «lote: <name>» and «sprint: <number>» are allowed, once each. Nothing was imported",
+    }
+    LOTE_INVALIDO {
+        es: "el lote «{lote}» no vale: letras y cifras ASCII, «.», «_» y «-», hasta 64. No se ha importado nada",
+        en: "the batch name «{lote}» is not valid: ASCII letters and digits, «.», «_» and «-», up to 64. Nothing was imported",
+    }
+    SPRINT_EN_CONFLICTO {
+        es: "la cabecera dice sprint {cabecera} y --sprint dice {opcion}: deja solo uno. No se ha importado nada",
+        en: "the header says sprint {cabecera} and --sprint says {opcion}: keep only one. Nothing was imported",
+    }
+    LINEA_SUELTA {
+        es: "linea {linea}: no es un criterio («- ...»), el objetivo («> ...»), «Dependencias: ...» ni una linea en blanco. No se ha importado nada",
+        en: "line {linea}: it is not a criterion («- ...»), the goal («> ...»), «Dependencias: ...» or a blank line. Nothing was imported",
+    }
+    TITULO_INVALIDO {
+        es: "linea {linea}: el «###» no tiene titulo o lleva caracteres de control. No se ha importado nada",
+        en: "line {linea}: the «###» has no title or contains control characters. Nothing was imported",
+    }
+    NO_LISTO_EN_LINEA {
+        es: "linea {linea}: «{titulo}» no esta listo: {causa}. No se ha importado nada",
+        en: "line {linea}: «{titulo}» is not ready: {causa}. Nothing was imported",
+    }
+    NO_LISTO {
+        es: "«{titulo}» no esta listo: {causa}. No se ha creado nada",
+        en: "«{titulo}» is not ready: {causa}. Nothing was created",
+    }
+    CAUSA_SIN_CRITERIOS {
+        es: "no tiene ningun criterio de aceptacion",
+        en: "it has no acceptance criteria",
+    }
+    CAUSA_CRITERIO_VACIO {
+        es: "tiene un criterio vacio",
+        en: "it has an empty criterion",
+    }
+    CAUSA_PLANTILLA {
+        es: "conserva un texto de la plantilla («<...>»)",
+        en: "it keeps a template placeholder («<...>»)",
+    }
+    CAUSA_REPETIDO {
+        es: "repite un criterio",
+        en: "it repeats a criterion",
+    }
+    CAUSA_CONTROL {
+        es: "lleva caracteres de control",
+        en: "it contains control characters",
+    }
+    CLAVE_DUPLICADA {
+        es: "linea {linea}: la clave «{clave}» ya la usa otra entrada; empieza el «###» por una clave distinta (p. ej. «### F3-2 ...»). No se ha importado nada",
+        en: "line {linea}: the key «{clave}» is already used by another entry; start the «###» with a different key (e.g. «### F3-2 ...»). Nothing was imported",
+    }
+    REFERENCIA_INEXISTENTE {
+        es: "linea {linea}: la dependencia «{referencia}» no es una clave de este fichero ni un TASK-NNN. No se ha importado nada",
+        en: "line {linea}: the dependency «{referencia}» is neither a key of this file nor a TASK-NNN. Nothing was imported",
+    }
+    EXTERNA_INEXISTENTE {
+        es: "la dependencia {referencia} de «{clave}» no existe en la rama de estado. No se ha importado nada",
+        en: "the dependency {referencia} of «{clave}» does not exist in the state branch. Nothing was imported",
+    }
+    DEPENDENCIA_INEXISTENTE {
+        es: "la dependencia {referencia} no existe en la rama de estado. No se ha creado nada",
+        en: "the dependency {referencia} does not exist in the state branch. Nothing was created",
+    }
+    CICLO {
+        es: "las dependencias forman un ciclo: {ciclo}. No se ha importado nada",
+        en: "the dependencies form a cycle: {ciclo}. Nothing was imported",
+    }
+    VALOR_INVALIDO {
+        es: "{opcion} no vale «{valor}»: {permitidos}",
+        en: "{opcion} does not accept «{valor}»: {permitidos}",
+    }
+    ETIQUETA_INVALIDA {
+        es: "--etiqueta no vale «{valor}»: una palabra sin espacios ni caracteres de control",
+        en: "--etiqueta does not accept «{valor}»: one word without spaces or control characters",
+    }
+    USO_NEW_CAMPOS {
+        es: "uso: new [--json] \"<titulo>\" [--tipo <tipo>] [--objetivo \"<texto>\"] [--criterio \"<texto>\"]... [--etiqueta <e>]... [--complejidad <c>] [--sprint <N>] [--dependencia TASK-NNN]...",
+        en: "usage: new [--json] \"<title>\" [--tipo <type>] [--objetivo \"<text>\"] [--criterio \"<text>\"]... [--etiqueta <tag>]... [--complejidad <c>] [--sprint <N>] [--dependencia TASK-NNN]...",
+    }
+}
diff --git a/crates/tc2/src/i18n/catalogo/mod.rs b/crates/tc2/src/i18n/catalogo/mod.rs
index f8f8c74..cdacf1a 100644
--- a/crates/tc2/src/i18n/catalogo/mod.rs
+++ b/crates/tc2/src/i18n/catalogo/mod.rs
@@ -9,6 +9,7 @@ pub mod desinstalar;
 pub mod estado;
 pub mod git;
 pub mod identidad;
+pub mod importar;
 pub mod init;
 pub mod plataforma;
 
@@ -25,6 +26,7 @@ pub fn todos() -> impl Iterator<Item = &'static Texto> {
         estado::TODOS,
         git::TODOS,
         identidad::TODOS,
+        importar::TODOS,
         init::TODOS,
         plataforma::TODOS,
     ]
diff --git a/crates/tc2/src/importar/fichero.rs b/crates/tc2/src/importar/fichero.rs
new file mode 100644
index 0000000..15eb179
--- /dev/null
+++ b/crates/tc2/src/importar/fichero.rs
@@ -0,0 +1,642 @@
+//! Fichero de import (F3-2, RF-02): texto → [`Lote`]. Puro: no toca disco,
+//! git ni la red; todo lo que se puede rechazar sin el estado (limites,
+//! formato, claves, referencias locales y ciclos) se rechaza aqui, antes de
+//! hablar con el remoto.
+//!
+//! Formato, el del import de la 0.16 (`src/core/import-parser.ts`) con dos
+//! anadidos opcionales:
+//!
+//! ```text
+//! ---                      <- cabecera opcional: lote y sprint
+//! lote: fase-3
+//! sprint: 3
+//! ---
+//! ### F3-2 Titulo          <- una entrada; la clave es el primer token
+//! > objetivo               <- lineas `>` antes del primer criterio
+//! - criterio               <- uno o mas; una linea sangrada lo continua
+//! Dependencias: F2-4, TASK-019
+//! ```
+//!
+//! Como en la 0.16: lo anterior al primer `###` se ignora, un encabezado de
+//! otro nivel cierra la entrada y sin cabecera el sprint llega por
+//! `--sprint`. A diferencia de la 0.16, una entrada mala no se salta: el
+//! lote entero se rechaza (todo o nada, F1-2).
+
+use std::collections::{BTreeMap, BTreeSet};
+
+use crate::alta::{Entrada, NoListo};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::importar as txt;
+use crate::tarea::{id_valido, titulo_valido};
+
+/// Tamano maximo del fichero (D4 de TASK-029).
+pub const MAX_BYTES: usize = 1 << 20;
+/// Entradas maximas por lote (D4 de TASK-029).
+pub const MAX_ENTRADAS: usize = 200;
+/// Longitud maxima de una clave de entrada o de un nombre de lote.
+const MAX_NOMBRE: usize = 64;
+
+/// Topes del fichero (parametro, D4).
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub struct Limites {
+    /// Bytes.
+    pub bytes: usize,
+    /// Entradas.
+    pub entradas: usize,
+}
+
+impl Default for Limites {
+    fn default() -> Self {
+        Self {
+            bytes: MAX_BYTES,
+            entradas: MAX_ENTRADAS,
+        }
+    }
+}
+
+/// Un fichero de import leido.
+#[derive(Debug, Clone, PartialEq, Eq, Default)]
+pub struct Lote {
+    /// `lote:` de la cabecera, si la hay.
+    pub nombre: Option<String>,
+    /// `sprint:` de la cabecera, si lo hay.
+    pub sprint: Option<u32>,
+    /// Entradas en el orden del fichero.
+    pub entradas: Vec<EntradaLote>,
+}
+
+/// Una entrada del fichero.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct EntradaLote {
+    /// Linea (desde 1) de su `###`.
+    pub linea: usize,
+    /// Clave dentro del lote: el primer token del titulo si parece una
+    /// (`F3-2`, `T1`, `A.4`: letras, cifras, `.`, `_`, `-` y al menos una
+    /// cifra) o, si no, el slug del titulo.
+    pub clave: String,
+    /// Titulo completo, como en la 0.16 (la clave incluida).
+    pub titulo: String,
+    /// Objetivo (lineas `>`).
+    pub objetivo: String,
+    /// Criterios.
+    pub criterios: Vec<String>,
+    /// Dependencias tal como se escribieron: claves de este lote o
+    /// `TASK-NNN`.
+    pub dependencias: Vec<String>,
+}
+
+impl EntradaLote {
+    /// La [`Entrada`] del alta, sin dependencias (se resuelven contra el
+    /// estado).
+    pub fn entrada(&self, sprint: u32) -> Entrada {
+        let mut e = Entrada::nueva(&self.titulo);
+        e.sprint = sprint;
+        e.objetivo.clone_from(&self.objetivo);
+        e.criterios.clone_from(&self.criterios);
+        e
+    }
+}
+
+/// Por que no se acepta un fichero: `uso` con este `motivo=`.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct ErrorLote {
+    /// Valor de `motivo=`.
+    pub motivo: &'static str,
+    /// Claves de mas para la linea (`linea=`, `causa=`...).
+    pub claves: Vec<(&'static str, String)>,
+    /// Diagnostico.
+    pub mensaje: Mensaje,
+}
+
+impl ErrorLote {
+    fn nuevo(motivo: &'static str, mensaje: Mensaje) -> Self {
+        Self {
+            motivo,
+            claves: Vec::new(),
+            mensaje,
+        }
+    }
+
+    fn en(motivo: &'static str, linea: usize, mensaje: Mensaje) -> Self {
+        Self::nuevo(motivo, mensaje.con("linea", linea.to_string())).con("linea", linea.to_string())
+    }
+
+    fn con(mut self, clave: &'static str, valor: String) -> Self {
+        self.claves.push((clave, valor));
+        self
+    }
+}
+
+/// Nombre de lote valido: ASCII alfanumerico, `.`, `_` y `-`, hasta 64.
+pub fn nombre_valido(n: &str) -> bool {
+    !n.is_empty()
+        && n.len() <= MAX_NOMBRE
+        && n.bytes()
+            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
+}
+
+/// Si `token` sirve de clave explicita: [`nombre_valido`] con alguna cifra
+/// y sin ser un `TASK-NNN` (que en `Dependencias:` es una tarea existente).
+fn parece_clave(token: &str) -> bool {
+    nombre_valido(token) && token.bytes().any(|b| b.is_ascii_digit()) && !id_valido(token)
+}
+
+/// Lee un fichero de import.
+pub fn leer(texto: &str, limites: Limites) -> Result<Lote, ErrorLote> {
+    if texto.len() > limites.bytes {
+        return Err(grande(limites));
+    }
+    let texto = texto.strip_prefix('\u{feff}').unwrap_or(texto);
+    let lineas: Vec<&str> = texto
+        .split('\n')
+        .map(|l| l.strip_suffix('\r').unwrap_or(l))
+        .collect();
+    let (nombre, sprint, desde) = cabecera(&lineas)?;
+    let entradas = entradas(&lineas, desde, limites)?;
+    if entradas.is_empty() {
+        return Err(ErrorLote::nuevo(
+            "sin-entradas",
+            Mensaje::nuevo(&txt::SIN_ENTRADAS),
+        ));
+    }
+    referencias(&entradas)?;
+    Ok(Lote {
+        nombre,
+        sprint,
+        entradas,
+    })
+}
+
+/// El error de fichero demasiado grande (tambien lo da el lector de disco
+/// antes de leerlo).
+pub fn grande(limites: Limites) -> ErrorLote {
+    ErrorLote::nuevo(
+        "fichero-grande",
+        Mensaje::nuevo(&txt::FICHERO_GRANDE).con("limite", limites.bytes.to_string()),
+    )
+    .con("limite", limites.bytes.to_string())
+}
+
+/// Cabecera `---` opcional: (lote, sprint, primera linea tras ella).
+fn cabecera(lineas: &[&str]) -> Result<(Option<String>, Option<u32>, usize), ErrorLote> {
+    if lineas.first() != Some(&"---") {
+        return Ok((None, None, 0));
+    }
+    let Some(fin) = lineas
+        .iter()
+        .skip(1)
+        .position(|l| *l == "---")
+        .map(|p| p + 1)
+    else {
+        return Err(ErrorLote::en(
+            "cabecera-sin-cerrar",
+            1,
+            Mensaje::nuevo(&txt::CABECERA_SIN_CERRAR),
+        ));
+    };
+    let mut nombre = None;
+    let mut sprint = None;
+    for (i, l) in lineas.iter().enumerate().take(fin).skip(1) {
+        let l = l.trim();
+        if l.is_empty() || l.starts_with('#') {
+            continue;
+        }
+        let malo = || {
+            ErrorLote::en(
+                "cabecera-invalida",
+                i + 1,
+                Mensaje::nuevo(&txt::CABECERA_INVALIDA),
+            )
+        };
+        let (k, v) = l.split_once(':').ok_or_else(malo)?;
+        let v = sin_comillas(v.trim());
+        match k.trim() {
+            "lote" if nombre.is_none() => {
+                if !nombre_valido(v) {
+                    return Err(ErrorLote::nuevo(
+                        "lote-invalido",
+                        Mensaje::nuevo(&txt::LOTE_INVALIDO).con("lote", v),
+                    ));
+                }
+                nombre = Some(v.to_owned());
+            }
+            "sprint" if sprint.is_none() => {
+                sprint = Some(crate::alta::sprint_de(v).ok_or_else(malo)?);
+            }
+            _ => return Err(malo()),
+        }
+    }
+    Ok((nombre, sprint, fin + 1))
+}
+
+/// `"x"` o `'x'` → `x`.
+fn sin_comillas(v: &str) -> &str {
+    for q in ['"', '\''] {
+        if let Some(s) = v.strip_prefix(q).and_then(|s| s.strip_suffix(q)) {
+            return s;
+        }
+    }
+    v
+}
+
+/// Entrada a medio leer.
+struct EnCurso {
+    linea: usize,
+    titulo: String,
+    objetivo: Vec<String>,
+    criterios: Vec<String>,
+    dependencias: Vec<String>,
+}
+
+/// Las entradas desde la linea `desde` (indice), con las reglas de la 0.16.
+fn entradas(
+    lineas: &[&str],
+    desde: usize,
+    limites: Limites,
+) -> Result<Vec<EntradaLote>, ErrorLote> {
+    let mut v: Vec<EntradaLote> = Vec::new();
+    let mut claves: BTreeSet<String> = BTreeSet::new();
+    let mut actual: Option<EnCurso> = None;
+    let mut pegada = false;
+    let mut cerrar = |e: Option<EnCurso>, v: &mut Vec<EntradaLote>| -> Result<(), ErrorLote> {
+        let Some(e) = e else { return Ok(()) };
+        let lote = cerrada(e)?;
+        if !claves.insert(lote.clave.clone()) {
+            return Err(ErrorLote::en(
+                "clave-duplicada",
+                lote.linea,
+                Mensaje::nuevo(&txt::CLAVE_DUPLICADA).con("clave", lote.clave.as_str()),
+            )
+            .con("clave", lote.clave.clone()));
+        }
+        if v.len() >= limites.entradas {
+            return Err(ErrorLote::nuevo(
+                "demasiadas-entradas",
+                Mensaje::nuevo(&txt::DEMASIADAS_ENTRADAS)
+                    .con("limite", limites.entradas.to_string()),
+            )
+            .con("limite", limites.entradas.to_string()));
+        }
+        v.push(lote);
+        Ok(())
+    };
+    for (i, linea) in lineas.iter().enumerate().skip(desde) {
+        let n = i + 1;
+        if let Some(t) = linea
+            .strip_prefix("###")
+            .filter(|t| t.is_empty() || t.starts_with([' ', '\t']))
+        {
+            cerrar(actual.take(), &mut v)?;
+            actual = Some(EnCurso {
+                linea: n,
+                titulo: t.trim().to_owned(),
+                objetivo: Vec::new(),
+                criterios: Vec::new(),
+                dependencias: Vec::new(),
+            });
+            pegada = false;
+            continue;
+        }
+        let Some(e) = actual.as_mut() else {
+            continue; // preambulo: se ignora, como en la 0.16
+        };
+        if es_encabezado(linea) {
+            cerrar(actual.take(), &mut v)?;
+            continue;
+        }
+        if linea.trim().is_empty() {
+            pegada = false;
+            continue;
+        }
+        if e.criterios.is_empty()
+            && let Some(o) = linea.strip_prefix('>')
+        {
+            e.objetivo.push(o.strip_prefix(' ').unwrap_or(o).to_owned());
+            continue;
+        }
+        if let Some(d) = dependencias(linea) {
+            e.dependencias.extend(d);
+            pegada = false;
+            continue;
+        }
+        if let Some(c) = criterio(linea) {
+            e.criterios.push(c.trim().to_owned());
+            pegada = true;
+            continue;
+        }
+        if pegada
+            && linea.starts_with([' ', '\t'])
+            && let Some(ultimo) = e.criterios.last_mut()
+        {
+            ultimo.push(' ');
+            ultimo.push_str(linea.trim());
+            continue;
+        }
+        return Err(ErrorLote::en(
+            "linea-suelta",
+            n,
+            Mensaje::nuevo(&txt::LINEA_SUELTA),
+        ));
+    }
+    cerrar(actual.take(), &mut v)?;
+    Ok(v)
+}
+
+/// `#`..`######` seguido de espacio (otro nivel que `###` cierra la
+/// entrada).
+fn es_encabezado(l: &str) -> bool {
+    let n = l.bytes().take_while(|b| *b == b'#').count();
+    (1..=6).contains(&n) && l[n..].starts_with([' ', '\t'])
+}
+
+/// `- x` o `* x`, con sangria.
+fn criterio(l: &str) -> Option<&str> {
+    let t = l.trim_start();
+    let resto = t.strip_prefix(['-', '*'])?;
+    resto
+        .starts_with([' ', '\t'])
+        .then(|| resto.trim())
+        .filter(|r| !r.is_empty())
+}
+
+/// `Dependencias: a, b` (sin distinguir mayusculas en la palabra).
+fn dependencias(l: &str) -> Option<Vec<String>> {
+    const PALABRA: &str = "dependencias:";
+    let cabeza = l.get(..PALABRA.len())?;
+    if !cabeza.eq_ignore_ascii_case(PALABRA) {
+        return None;
+    }
+    Some(
+        l[PALABRA.len()..]
+            .split(',')
+            .map(str::trim)
+            .filter(|d| !d.is_empty())
+            .map(str::to_owned)
+            .collect(),
+    )
+}
+
+/// Comprueba una entrada terminada y le pone la clave.
+fn cerrada(e: EnCurso) -> Result<EntradaLote, ErrorLote> {
+    if !titulo_valido(&e.titulo) {
+        return Err(ErrorLote::en(
+            "titulo-invalido",
+            e.linea,
+            Mensaje::nuevo(&txt::TITULO_INVALIDO),
+        ));
+    }
+    let primero = e.titulo.split_whitespace().next().unwrap_or_default();
+    let clave = if parece_clave(primero) {
+        primero.to_owned()
+    } else {
+        crate::tarea::texto::slug(&e.titulo)
+    };
+    let mut dependencias = Vec::new();
+    for d in e.dependencias {
+        if !dependencias.contains(&d) {
+            dependencias.push(d);
+        }
+    }
+    let lote = EntradaLote {
+        linea: e.linea,
+        clave,
+        titulo: e.titulo,
+        objetivo: e.objetivo.join("\n").trim().to_owned(),
+        criterios: e.criterios,
+        dependencias,
+    };
+    lote.entrada(0).listo(true).map_err(|c: NoListo| {
+        ErrorLote::en(
+            "enunciado-no-listo",
+            lote.linea,
+            Mensaje::nuevo(&txt::NO_LISTO_EN_LINEA)
+                .con("titulo", lote.titulo.as_str())
+                .con_mensaje("causa", c.mensaje()),
+        )
+        .con("causa", c.causa().to_owned())
+    })?;
+    Ok(lote)
+}
+
+/// Cada dependencia es una clave del lote o un `TASK-NNN`, y las locales
+/// no forman ciclos.
+fn referencias(entradas: &[EntradaLote]) -> Result<(), ErrorLote> {
+    let indice: BTreeMap<&str, usize> = entradas
+        .iter()
+        .enumerate()
+        .map(|(i, e)| (e.clave.as_str(), i))
+        .collect();
+    let mut aristas: Vec<Vec<usize>> = Vec::with_capacity(entradas.len());
+    for e in entradas {
+        let mut a = Vec::new();
+        for d in &e.dependencias {
+            match indice.get(d.as_str()) {
+                Some(j) => a.push(*j),
+                None if id_valido(d) => {}
+                None => {
+                    return Err(ErrorLote::en(
+                        crate::alta::REFERENCIA_INEXISTENTE,
+                        e.linea,
+                        Mensaje::nuevo(&txt::REFERENCIA_INEXISTENTE).con("referencia", d.as_str()),
+                    )
+                    .con("referencia", d.clone()));
+                }
+            }
+        }
+        aristas.push(a);
+    }
+    if let Some(ciclo) = ciclo(&aristas) {
+        let nombres: Vec<&str> = ciclo
+            .iter()
+            .filter_map(|i| entradas.get(*i).map(|e| e.clave.as_str()))
+            .collect();
+        return Err(ErrorLote::nuevo(
+            "ciclo",
+            Mensaje::nuevo(&txt::CICLO).con("ciclo", nombres.join(" -> ")),
+        )
+        .con("ciclo", nombres.join(",")));
+    }
+    Ok(())
+}
+
+/// Un ciclo del grafo (`aristas[i]`: de quien depende `i`), cerrado (el
+/// primero se repite al final), o `None`. Iterativo: 200 entradas en
+/// cadena no gastan pila.
+fn ciclo(aristas: &[Vec<usize>]) -> Option<Vec<usize>> {
+    #[derive(Clone, Copy, PartialEq, Eq)]
+    enum Color {
+        Blanco,
+        Gris,
+        Negro,
+    }
+    let mut color = vec![Color::Blanco; aristas.len()];
+    for raiz in 0..aristas.len() {
+        if color[raiz] != Color::Blanco {
+            continue;
+        }
+        // Pila de (nodo, siguiente arista por mirar).
+        let mut pila: Vec<(usize, usize)> = vec![(raiz, 0)];
+        color[raiz] = Color::Gris;
+        while let Some((nodo, k)) = pila.last_mut() {
+            let nodo = *nodo;
+            let Some(&sig) = aristas.get(nodo).and_then(|a| a.get(*k)) else {
+                color[nodo] = Color::Negro;
+                pila.pop();
+                continue;
+            };
+            *k += 1;
+            match color.get(sig) {
+                Some(Color::Blanco) => {
+                    color[sig] = Color::Gris;
+                    pila.push((sig, 0));
+                }
+                Some(Color::Gris) => {
+                    let desde = pila.iter().position(|(n, _)| *n == sig).unwrap_or(0);
+                    let mut c: Vec<usize> = pila[desde..].iter().map(|(n, _)| *n).collect();
+                    c.push(sig);
+                    return Some(c);
+                }
+                _ => {}
+            }
+        }
+    }
+    None
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn ok(t: &str) -> Lote {
+        let r = leer(t, Limites::default());
+        assert!(r.is_ok(), "{r:?}");
+        r.unwrap_or_default()
+    }
+
+    fn motivo(t: &str) -> &'static str {
+        leer(t, Limites::default()).map_or_else(|e| e.motivo, |_| "ok")
+    }
+
+    #[test]
+    fn formato_016_sin_cabecera() {
+        let l = ok(
+            "# Plan\n\nprosa ignorada\n\n### F1-19 ADR memoria\n> Objetivo uno\n>\n> dos\n- Criterio a\n  sigue a\n* Criterio b\n\n### Sin clave aqui\n- c\n## Otra cosa\ntexto suelto tras cerrar\n",
+        );
+        assert_eq!((l.nombre, l.sprint), (None, None));
+        let e = &l.entradas;
+        assert_eq!(e.len(), 2);
+        assert_eq!((e[0].linea, e[0].clave.as_str()), (5, "F1-19"));
+        assert_eq!(e[0].titulo, "F1-19 ADR memoria");
+        assert_eq!(e[0].objetivo, "Objetivo uno\n\ndos");
+        assert_eq!(e[0].criterios, ["Criterio a sigue a", "Criterio b"]);
+        assert_eq!(e[1].clave, "sin-clave-aqui");
+        assert_eq!(e[1].objetivo, "");
+    }
+
+    #[test]
+    fn cabecera_y_dependencias() {
+        let l = ok(
+            "\u{feff}---\r\nlote: fase-3\r\nsprint: \"3\"\r\n---\r\n### A1 Uno\r\n- c\r\n### A2 Dos\r\nDependencias: A1, TASK-019, A1\r\n- c\r\n",
+        );
+        assert_eq!((l.nombre.as_deref(), l.sprint), (Some("fase-3"), Some(3)));
+        assert_eq!(l.entradas[1].dependencias, ["A1", "TASK-019"]);
+        assert_eq!(l.entradas[1].linea, 7);
+        for (t, m) in [
+            ("---\nlote: x\n", "cabecera-sin-cerrar"),
+            ("---\nhito: 3\n---\n### A1 x\n- c\n", "cabecera-invalida"),
+            ("---\nsprint: -1\n---\n### A1 x\n- c\n", "cabecera-invalida"),
+            (
+                "---\nsprint: 1\nsprint: 1\n---\n### A1 x\n- c\n",
+                "cabecera-invalida",
+            ),
+            (
+                "---\nlote: con espacio\n---\n### A1 x\n- c\n",
+                "lote-invalido",
+            ),
+            ("---\nlote: a/b\n---\n### A1 x\n- c\n", "lote-invalido"),
+        ] {
+            assert_eq!(motivo(t), m, "{t}");
+        }
+    }
+
+    #[test]
+    fn entradas_malas_rechazan_el_lote_entero() {
+        for (t, m) in [
+            ("", "sin-entradas"),
+            ("solo prosa\n", "sin-entradas"),
+            ("### A1 x\n", "enunciado-no-listo"),
+            ("### A1 x\n> solo objetivo\n", "enunciado-no-listo"),
+            ("### A1 x\n- <criterio comprobable>\n", "enunciado-no-listo"),
+            ("### A1 x\n- a\n- A\n", "enunciado-no-listo"),
+            ("### \n- a\n", "titulo-invalido"),
+            ("###\n- a\n", "titulo-invalido"),
+            ("### A1 x\nprosa\n- a\n", "linea-suelta"),
+            ("### A1 x\n- a\n> objetivo tarde\n", "linea-suelta"),
+            ("### A1 x\n- a\n\n  sangrada tras blanco\n", "linea-suelta"),
+            ("### A1 x\n- a\n### A1 y\n- b\n", "clave-duplicada"),
+            (
+                "### A1 x\n- a\nDependencias: B9\n",
+                "referencia-inexistente",
+            ),
+            ("### A1 x\n- a\nDependencias: A1\n", "ciclo"),
+            (
+                "### A1 x\n- a\nDependencias: A3\n### A2 y\n- b\nDependencias: A1\n### A3 z\n- c\nDependencias: A2\n",
+                "ciclo",
+            ),
+        ] {
+            assert_eq!(motivo(t), m, "{t:?}");
+        }
+        // La linea del error es la del `###` de la entrada mala.
+        let mut t = String::new();
+        for i in 1..=40 {
+            t.push_str(&format!("### T{i} tarea {i}\n- criterio {i}\n"));
+        }
+        let malo = t.replacen("### T30 tarea 30\n- criterio 30\n", "### T30 tarea 30\n", 1);
+        let e = leer(&malo, Limites::default()).err();
+        assert_eq!(
+            e.map(|e| (e.motivo, e.claves)),
+            Some((
+                "enunciado-no-listo",
+                vec![
+                    ("linea", "59".to_owned()),
+                    ("causa", "sin-criterios".to_owned())
+                ]
+            ))
+        );
+        let e = leer(&t.replace("criterio 3\n", "x\nprosa\n"), Limites::default()).err();
+        assert_eq!(e.map(|e| e.motivo), Some("linea-suelta"));
+    }
+
+    #[test]
+    fn limites_de_bytes_y_de_entradas() {
+        let l = Limites {
+            bytes: 64,
+            entradas: 2,
+        };
+        assert_eq!(
+            leer(&"x".repeat(65), l).err().map(|e| e.motivo),
+            Some("fichero-grande")
+        );
+        let tres = "### A1 a\n- c\n### A2 b\n- c\n### A3 c\n- c\n";
+        assert_eq!(tres.len(), 39);
+        assert_eq!(
+            leer(tres, l).err().map(|e| e.motivo),
+            Some("demasiadas-entradas")
+        );
+        assert_eq!(leer(&tres[..26], l).map(|x| x.entradas.len()), Ok(2));
+    }
+
+    #[test]
+    fn claves_y_ciclos() {
+        assert!(parece_clave("F3-2") && parece_clave("T1") && parece_clave("A.4"));
+        assert!(!parece_clave("Arreglar") && !parece_clave("TASK-019") && !parece_clave("F3/2"));
+        assert_eq!(ciclo(&[vec![1], vec![2], vec![]]), None);
+        assert_eq!(ciclo(&[vec![1], vec![0]]), Some(vec![0, 1, 0]));
+        assert_eq!(ciclo(&[vec![], vec![2], vec![1]]), Some(vec![1, 2, 1]));
+        // Una cadena larga no desborda la pila.
+        let cadena: Vec<Vec<usize>> = (0..10_000)
+            .map(|i| if i == 0 { vec![] } else { vec![i - 1] })
+            .collect();
+        assert_eq!(ciclo(&cadena), None);
+    }
+}
diff --git a/crates/tc2/src/importar/mod.rs b/crates/tc2/src/importar/mod.rs
new file mode 100644
index 0000000..d961e7c
--- /dev/null
+++ b/crates/tc2/src/importar/mod.rs
@@ -0,0 +1,447 @@
+//! `tc2 import` (F3-2, RF-02): alta en bloque desde un fichero de
+//! planificacion, con dependencias e hito (el sprint, D2 de TASK-029).
+//!
+//! - [`fichero`] lee el fichero (puro: limites, formato, claves y ciclos).
+//! - [`Importar`] es **una sola** [`Operacion`] sobre la transaccion de F2-3:
+//!   un commit, un push y N IDs consecutivos (ADR F1-2, punto 3). Comparte
+//!   con `new` el nucleo [`crate::alta`] (reserva, rama y parecidas).
+//!
+//! Idempotencia (D1): cada tarea importada guarda `lote: "<lote>/<clave>"`.
+//! Al repetir el import se crean solo las entradas cuya clave no esta; las
+//! que ya estan no se actualizan (un import no sincroniza) y salen en
+//! `existentes=`. Si no falta ninguna, `sin-cambios` sin commit.
+//!
+//! Dependencias: una clave del fichero se resuelve al ID de su entrada (la
+//! nueva o la ya importada); un `TASK-NNN` tiene que existir en el estado
+//! fresco del intento. Una referencia que no existe da `uso` (2): el fallo
+//! es del fichero, y no se escribe nada.
+//!
+//! `--simular` («plan como dato», como `init` y `desinstalar`) calcula el
+//! mismo plan con la misma [`Operacion`] sobre el estado que el clon ya
+//! conoce: sin `fetch` ni transaccion, asi que no cambia ni un byte en local
+//! ni en el remoto. `base=` dice sobre que commit se calculo; si otro clon
+//! publico despues, los IDs reales pueden ser otros.
+
+pub mod fichero;
+
+use std::cell::RefCell;
+use std::collections::{BTreeMap, BTreeSet};
+use std::path::Path;
+
+use crate::alta::{self, Parecida};
+use crate::contrato::{Informe, Resultado, Valor, corto};
+use crate::estado::{self, Cambio, Catalogo, Estado, Fallo, Operacion};
+use crate::git;
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::cli as txt_cli;
+use crate::i18n::catalogo::importar as txt;
+use fichero::{EntradaLote, ErrorLote, Limites};
+
+/// Nombre de la operacion y de la orden.
+pub const NOMBRE: &str = "import";
+
+/// Lo que hace (o haria) un import sobre un estado.
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct Plan {
+    /// IDs que se crean, en el orden del fichero.
+    pub creadas: Vec<String>,
+    /// IDs de las entradas que ya estaban importadas, en el orden del
+    /// fichero.
+    pub existentes: Vec<String>,
+    /// Tareas abiertas (de fuera del lote) con un titulo parecido.
+    pub parecidas: Vec<Parecida>,
+}
+
+/// Alta en bloque de las entradas de un lote.
+#[derive(Debug)]
+pub struct Importar {
+    lote: String,
+    sprint: u32,
+    entradas: Vec<EntradaLote>,
+    fecha: String,
+    ocupados: BTreeSet<u32>,
+    plan: RefCell<Option<Plan>>,
+}
+
+impl Importar {
+    /// Import de `entradas` con el nombre de lote `lote`, en `sprint`.
+    pub fn nuevo(lote: &str, sprint: u32, entradas: Vec<EntradaLote>) -> Self {
+        Self {
+            lote: lote.to_owned(),
+            sprint,
+            entradas,
+            fecha: alta::hoy(),
+            ocupados: BTreeSet::new(),
+            plan: RefCell::new(None),
+        }
+    }
+
+    /// IDs con rama (`git::ids_con_rama`), como en `new` (F2-4b).
+    #[must_use]
+    pub fn con_ocupados(mut self, ocupados: BTreeSet<u32>) -> Self {
+        self.ocupados = ocupados;
+        self
+    }
+
+    /// El plan del ultimo intento.
+    pub fn plan(&self) -> Option<Plan> {
+        self.plan.borrow().clone()
+    }
+
+    fn lote_de(&self, e: &EntradaLote) -> String {
+        format!("{}/{}", self.lote, e.clave)
+    }
+}
+
+impl Operacion for Importar {
+    fn nombre(&self) -> &str {
+        NOMBRE
+    }
+    fn anotacion(&self) -> String {
+        format!("{NOMBRE}|{}|{}", self.lote, self.entradas.len())
+    }
+    fn exige_red(&self) -> bool {
+        // Reserva IDs: sin red falla, nunca queda pendiente (F1-2 D2b).
+        true
+    }
+    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
+        let rutas = estado.rutas();
+        let tareas = alta::tareas(estado, &rutas)?;
+        let prefijo = format!("{}/", self.lote);
+        let del_lote = |t: &crate::tarea::Tarea| {
+            t.lote
+                .as_deref()
+                .and_then(|l| l.strip_prefix(&prefijo))
+                .map(str::to_owned)
+        };
+        // Clave → ID de lo ya importado de este lote (la primera si hubiera
+        // dos con la misma clave, puestas a mano).
+        let mut mapa: BTreeMap<String, String> = BTreeMap::new();
+        for t in &tareas {
+            if let Some(c) = del_lote(t) {
+                mapa.entry(c).or_insert_with(|| t.id.clone());
+            }
+        }
+        let (existentes, faltan): (Vec<&EntradaLote>, Vec<&EntradaLote>) = self
+            .entradas
+            .iter()
+            .partition(|e| mapa.contains_key(&e.clave));
+        let existentes: Vec<String> = existentes
+            .iter()
+            .filter_map(|e| mapa.get(&e.clave).cloned())
+            .collect();
+        let ids = alta::reservar(&rutas, &self.ocupados, faltan.len())?;
+        for (e, id) in faltan.iter().zip(&ids) {
+            mapa.insert(e.clave.clone(), alta::formato_id(*id));
+        }
+        let mut entradas = Vec::with_capacity(faltan.len());
+        for e in &faltan {
+            let mut en = e.entrada(self.sprint);
+            en.lote = Some(self.lote_de(e));
+            for d in &e.dependencias {
+                let id = match mapa.get(d) {
+                    Some(id) => id.clone(),
+                    None if rutas.iter().any(|r| *r == format!("{d}/tarea.md")) => d.clone(),
+                    None => {
+                        return Err(Fallo::rechazada(
+                            alta::REFERENCIA_INEXISTENTE,
+                            Mensaje::nuevo(&txt::EXTERNA_INEXISTENTE)
+                                .con("referencia", d.as_str())
+                                .con("clave", e.clave.as_str()),
+                        ));
+                    }
+                };
+                en.dependencias.push(id);
+            }
+            entradas.push(en);
+        }
+        // Las del propio lote no cuentan como parecidas: son el mismo plan.
+        let fuera: Vec<crate::tarea::Tarea> = tareas
+            .into_iter()
+            .filter(|t| del_lote(t).is_none())
+            .collect();
+        let a = alta::altas(&fuera, &ids, &entradas, &self.fecha)?;
+        *self.plan.borrow_mut() = Some(Plan {
+            creadas: ids.iter().map(|n| alta::formato_id(*n)).collect(),
+            existentes,
+            parecidas: a.parecidas,
+        });
+        Ok(a.cambios)
+    }
+}
+
+/// Opciones de la orden (sin `--json`).
+#[derive(Debug, Clone, PartialEq, Eq)]
+struct Opciones {
+    simular: bool,
+    sprint: Option<u32>,
+    fichero: String,
+}
+
+fn uso(motivo: &str) -> Informe {
+    Informe::nuevo(NOMBRE, Resultado::Uso)
+        .con("motivo", motivo)
+        .diagnostico(&txt::USO_IMPORT)
+}
+
+fn opciones(args: &[String]) -> Result<Opciones, Informe> {
+    let mut simular = false;
+    let mut sprint = None;
+    let mut fichero = None;
+    let mut it = args.iter();
+    while let Some(a) = it.next() {
+        match a.as_str() {
+            "--simular" if !simular => simular = true,
+            "--sprint" if sprint.is_none() => {
+                let v = it.next().ok_or_else(|| uso("sprint-invalido"))?;
+                sprint = Some(alta::sprint_de(v).ok_or_else(|| uso("sprint-invalido"))?);
+            }
+            f if !f.starts_with("--") && fichero.is_none() => fichero = Some(f.to_owned()),
+            _ => return Err(uso("argumentos")),
+        }
+    }
+    let fichero = fichero.ok_or_else(|| uso("sin-fichero"))?;
+    Ok(Opciones {
+        simular,
+        sprint,
+        fichero,
+    })
+}
+
+fn de_error(e: ErrorLote) -> Informe {
+    let mut i = Informe::nuevo(NOMBRE, Resultado::Uso).con("motivo", e.motivo);
+    for (k, v) in e.claves {
+        i = i.con(k, v);
+    }
+    i.diagnostico(e.mensaje)
+}
+
+/// Lee el fichero sin cargar mas del tope de bytes: se lee como mucho un
+/// byte de mas, y si llega a leerse el fichero es demasiado grande (sin
+/// carrera entre mirar el tamano y leer).
+fn leer_fichero(ruta: &Path, limites: Limites) -> Result<String, Informe> {
+    use std::io::Read as _;
+    let no_se_lee = |e: std::io::Error| {
+        Informe::nuevo(NOMBRE, Resultado::Uso)
+            .con("motivo", "no-se-lee")
+            .diagnostico(
+                Mensaje::nuevo(&txt::NO_SE_LEE)
+                    .con("ruta", ruta.display().to_string())
+                    .con("error", e.to_string()),
+            )
+    };
+    let f = std::fs::File::open(ruta).map_err(no_se_lee)?;
+    let tope = u64::try_from(limites.bytes).unwrap_or(u64::MAX);
+    let mut bytes = Vec::new();
+    f.take(tope.saturating_add(1))
+        .read_to_end(&mut bytes)
+        .map_err(no_se_lee)?;
+    if bytes.len() > limites.bytes {
+        return Err(de_error(fichero::grande(limites)));
+    }
+    String::from_utf8(bytes).map_err(|_| {
+        Informe::nuevo(NOMBRE, Resultado::Uso)
+            .con("motivo", "no-utf8")
+            .diagnostico(Mensaje::nuevo(&txt::NO_UTF8).con("ruta", ruta.display().to_string()))
+    })
+}
+
+/// `import [--simular] [--sprint N] <fichero>` en la carpeta actual. Las
+/// tareas abiertas parecidas quedan en `parecidas` para el aviso.
+pub fn orden(args: &[String], catalogo: Catalogo<'_>, parecidas: &mut Vec<Parecida>) -> Informe {
+    let o = match opciones(args) {
+        Ok(o) => o,
+        Err(i) => return i,
+    };
+    let dir = match std::env::current_dir() {
+        Ok(d) => d,
+        Err(e) => {
+            return Informe::nuevo(NOMBRE, Resultado::Error)
+                .diagnostico(Mensaje::nuevo(&txt_cli::CARPETA_ACTUAL).con("error", e.to_string()));
+        }
+    };
+    let g = git::Git::nuevo();
+    importar_en(&g, &dir, &o, catalogo, parecidas)
+}
+
+fn importar_en(
+    g: &git::Git,
+    dir: &Path,
+    o: &Opciones,
+    catalogo: Catalogo<'_>,
+    parecidas: &mut Vec<Parecida>,
+) -> Informe {
+    let limites = Limites::default();
+    let ruta = dir.join(&o.fichero);
+    let texto = match leer_fichero(&ruta, limites) {
+        Ok(t) => t,
+        Err(i) => return i,
+    };
+    let lote = match fichero::leer(&texto, limites) {
+        Ok(l) => l,
+        Err(e) => return de_error(e),
+    };
+    let sprint = match (lote.sprint, o.sprint) {
+        (Some(c), Some(f)) if c != f => {
+            return Informe::nuevo(NOMBRE, Resultado::Uso)
+                .con("motivo", "sprint-en-conflicto")
+                .diagnostico(
+                    Mensaje::nuevo(&txt::SPRINT_EN_CONFLICTO)
+                        .con("cabecera", c.to_string())
+                        .con("opcion", f.to_string()),
+                );
+        }
+        (c, f) => c.or(f).unwrap_or(0),
+    };
+    // Sin `lote:` en la cabecera, el nombre del fichero (en slug).
+    let nombre = lote.nombre.clone().unwrap_or_else(|| {
+        let tallo = ruta
+            .file_stem()
+            .map(|s| s.to_string_lossy().into_owned())
+            .unwrap_or_default();
+        crate::tarea::texto::slug(&tallo)
+    });
+    // Las ramas `*/task-*` una vez, antes de la transaccion (F2-4b).
+    let ocupados = match git::ids_con_rama(g, dir, NOMBRE) {
+        Ok(x) => x,
+        Err(mut i) => {
+            NOMBRE.clone_into(&mut i.comando);
+            if i.resultado == Resultado::Sinred {
+                i = i.con("pendiente", "no");
+            }
+            return i;
+        }
+    };
+    let op = Importar::nuevo(&nombre, sprint, lote.entradas).con_ocupados(ocupados);
+    if o.simular {
+        return simular(g, dir, &op, parecidas);
+    }
+    let c = estado::Contexto::nuevo(g, dir, catalogo);
+    let mut i = estado::transaccion(&c, &op);
+    NOMBRE.clone_into(&mut i.comando);
+    alta::referencia_a_uso(&mut i);
+    if i.resultado.es_exito()
+        && let Some(p) = op.plan()
+    {
+        i = con_plan(i, &p, parecidas);
+    }
+    i
+}
+
+/// Pone `creadas=`, `existentes=`, `ids=` (y `parecidas=`) al principio.
+fn con_plan(mut i: Informe, p: &Plan, parecidas: &mut Vec<Parecida>) -> Informe {
+    let mut delante = vec![
+        (
+            "creadas".to_owned(),
+            Valor::Texto(p.creadas.len().to_string()),
+        ),
+        (
+            "existentes".to_owned(),
+            Valor::Texto(p.existentes.len().to_string()),
+        ),
+    ];
+    if !p.creadas.is_empty() {
+        delante.push(("ids".to_owned(), Valor::Lista(p.creadas.clone())));
+    }
+    if !p.parecidas.is_empty() {
+        let ids = p.parecidas.iter().map(|x| x.id.clone()).collect();
+        delante.push(("parecidas".to_owned(), Valor::Lista(ids)));
+    }
+    parecidas.clone_from(&p.parecidas);
+    delante.append(&mut i.claves);
+    i.claves = delante;
+    i
+}
+
+/// `--simular`: el plan sobre el estado conocido, sin `fetch` ni escribir.
+fn simular(g: &git::Git, dir: &Path, op: &Importar, parecidas: &mut Vec<Parecida>) -> Informe {
+    let vista = match estado::leer(g, dir) {
+        Ok(v) => v,
+        // Sin rama de estado todavia: el estado vacio (el primer ID es el
+        // siguiente a las ramas de tarea).
+        Err(i) if i.valor("motivo") == Some("sin-rama-de-estado") => estado::Vista::vacia(g, dir),
+        Err(mut i) => {
+            NOMBRE.clone_into(&mut i.comando);
+            return i;
+        }
+    };
+    match op.aplicar(&vista) {
+        Ok(cambios) => {
+            let r = if cambios.is_empty() {
+                Resultado::SinCambios
+            } else {
+                Resultado::Ok
+            };
+            let mut i = Informe::nuevo(NOMBRE, r);
+            if let Some(b) = vista.commit() {
+                i = i.con("base", corto(b));
+            }
+            let p = op.plan().unwrap_or_default();
+            let mut i = con_plan(i, &p, parecidas);
+            // `simular=si` primero: se ve de un vistazo que no se escribio.
+            i.claves
+                .insert(0, ("simular".to_owned(), Valor::Texto("si".to_owned())));
+            i
+        }
+        Err(Fallo::Rechazada {
+            motivo,
+            diagnostico,
+        }) => {
+            let mut i = Informe::nuevo(NOMBRE, Resultado::Rechazada)
+                .con("simular", "si")
+                .con("motivo", motivo)
+                .diagnostico(diagnostico);
+            alta::referencia_a_uso(&mut i);
+            i
+        }
+        Err(Fallo::Error(d)) => Informe::nuevo(NOMBRE, Resultado::Error)
+            .con("simular", "si")
+            .diagnostico(d),
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn args(v: &[&str]) -> Vec<String> {
+        v.iter().map(|s| (*s).to_owned()).collect()
+    }
+
+    #[test]
+    fn opciones_de_la_orden() {
+        assert_eq!(
+            opciones(&args(&["--simular", "f.md", "--sprint", "3"])),
+            Ok(Opciones {
+                simular: true,
+                sprint: Some(3),
+                fichero: "f.md".to_owned()
+            })
+        );
+        for (a, m) in [
+            (vec![], "sin-fichero"),
+            (vec!["--simular"], "sin-fichero"),
+            (vec!["a.md", "--simular", "--simular"], "argumentos"),
+            (vec!["a.md", "b.md"], "argumentos"),
+            (vec!["a.md", "--sprint", "1", "--sprint", "1"], "argumentos"),
+            (vec!["a.md", "--otra"], "argumentos"),
+            (vec!["a.md", "--sprint"], "sprint-invalido"),
+            (vec!["a.md", "--sprint", "-1"], "sprint-invalido"),
+            (vec!["a.md", "--sprint", "x"], "sprint-invalido"),
+        ] {
+            let r = opciones(&args(&a));
+            assert_eq!(
+                r.err().and_then(|i| i.valor("motivo").map(str::to_owned)),
+                Some(m.to_owned()),
+                "{a:?}"
+            );
+        }
+    }
+
+    #[test]
+    fn anotacion_en_una_linea() {
+        let op = Importar::nuevo("fase-3", 3, Vec::new());
+        assert_eq!(op.anotacion(), "import|fase-3|0");
+        assert!(op.exige_red());
+    }
+}
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 724d883..156a393 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -12,7 +12,8 @@
 //! ejecuta y `tc2 identidad`, F2-13) y [`huella`] (los nombres que la 2.0
 //! deja en un repo, F1-12 D1a) e [`init`] (deja un repo listo, F3-1) e
 //! [`instalacion`] (donde vive el `tc2` de la
-//! maquina, F2-12 y F2-14) y [`desinstalar`] (quitar TaskCodeTwo, F2-14). Las capas
+//! maquina, F2-12 y F2-14) y [`desinstalar`] (quitar TaskCodeTwo, F2-14) e
+//! [`importar`] (alta en bloque con dependencias e hito, F3-2). Las capas
 //! que vienen entran como modulos de esta biblioteca.
 
 pub mod actualizar_cli;
@@ -27,6 +28,7 @@ pub mod git;
 pub mod huella;
 pub mod i18n;
 pub mod identidad;
+pub mod importar;
 pub mod init;
 pub mod instalacion;
 pub mod plataforma;
@@ -95,16 +97,14 @@ where
                 r => (false, r),
             };
             let mut parecidas = Vec::new();
-            let mut s = orden_new(resto, &mut parecidas).salida_en(json, idioma);
-            // El aviso de titulos parecidos va a stderr con codigo 0: es la
-            // unica excepcion a «diagnostico solo si falla» (F2-4b, D1).
-            if s.codigo == 0
-                && let Some(a) = alta::aviso_parecidas(&parecidas, idioma)
-            {
-                s.stderr.push_str(&a);
-                s.stderr.push('\n');
-            }
-            s
+            let s = orden_new(resto, &mut parecidas).salida_en(json, idioma);
+            con_aviso(s, &parecidas, idioma)
+        }
+        [orden, resto @ ..] if orden == importar::NOMBRE => {
+            let (json, resto) = con_json(resto);
+            let mut parecidas = Vec::new();
+            let s = importar::orden(&resto, &catalogo, &mut parecidas).salida_en(json, idioma);
+            con_aviso(s, &parecidas, idioma)
         }
         [orden, resto @ ..] if orden == version::VERSION => {
             let (json, resto) = con_json(resto);
@@ -156,6 +156,18 @@ where
     }
 }
 
+/// El aviso de titulos parecidos va a stderr con codigo 0: es la unica
+/// excepcion a «diagnostico solo si falla» (F2-4b, D1).
+fn con_aviso(mut s: Salida, parecidas: &[alta::Parecida], idioma: Idioma) -> Salida {
+    if s.codigo == 0
+        && let Some(a) = alta::aviso_parecidas(parecidas, idioma)
+    {
+        s.stderr.push_str(&a);
+        s.stderr.push('\n');
+    }
+    s
+}
+
 /// Quita `--json` de donde este y dice si estaba.
 fn con_json(args: &[String]) -> (bool, Vec<String>) {
     let json = args.iter().any(|a| a == "--json");
@@ -251,27 +263,15 @@ fn catalogo(anotacion: &str) -> Option<Box<dyn estado::Operacion>> {
     alta::desde_anotacion(anotacion)
 }
 
-/// `new "<titulo>"`: alta de una tarea con su ID reservado por el push de la
-/// rama de estado (F2-4). Sale `new ok id=TASK-NNN rama=... [parecidas=...]`;
-/// las tareas abiertas parecidas quedan ademas en `parecidas` para el aviso.
+/// `new "<titulo>" [campos de RF-01]`: alta de una tarea con su ID reservado
+/// por el push de la rama de estado (F2-4). Sale `new ok id=TASK-NNN
+/// rama=... [parecidas=...]`; las tareas abiertas parecidas quedan ademas en
+/// `parecidas` para el aviso. Un enunciado que no esta listo o una
+/// dependencia que no existe es `uso` sin escribir nada (TASK-029).
 fn orden_new(args: &[String], parecidas: &mut Vec<alta::Parecida>) -> Informe {
-    let titulo = match args {
-        [t] if alta::titulo_valido(t) => t,
-        [_] => {
-            return Informe::nuevo(alta::NOMBRE, Resultado::Uso)
-                .con("motivo", "titulo-invalido")
-                .diagnostico(&txt::TITULO_INVALIDO);
-        }
-        [] => {
-            return Informe::nuevo(alta::NOMBRE, Resultado::Uso)
-                .con("motivo", "sin-titulo")
-                .diagnostico(&txt::USO_NEW);
-        }
-        _ => {
-            return Informe::nuevo(alta::NOMBRE, Resultado::Uso)
-                .con("motivo", "argumentos-de-mas")
-                .diagnostico(&txt::TITULO_ENTRE_COMILLAS);
-        }
+    let entrada = match alta::entrada_de_args(args) {
+        Ok(e) => e,
+        Err(i) => return i,
     };
     let dir = match std::env::current_dir() {
         Ok(d) => d,
@@ -293,10 +293,11 @@ fn orden_new(args: &[String], parecidas: &mut Vec<alta::Parecida>) -> Informe {
             return i;
         }
     };
-    let op = alta::Alta::nueva(titulo).con_ocupados(ocupados);
+    let op = alta::Alta::con_entrada(entrada).con_ocupados(ocupados);
     let c = estado::Contexto::nuevo(&g, &dir, &catalogo);
     let mut i = estado::transaccion(&c, &op);
     alta::NOMBRE.clone_into(&mut i.comando);
+    alta::referencia_a_uso(&mut i);
     if i.resultado == Resultado::Ok
         && let (Some(id), Some(rama)) = (op.id(), op.rama())
     {
diff --git a/crates/tc2/src/tarea/cuerpo.rs b/crates/tc2/src/tarea/cuerpo.rs
index dc5f814..1aba934 100644
--- a/crates/tc2/src/tarea/cuerpo.rs
+++ b/crates/tc2/src/tarea/cuerpo.rs
@@ -24,6 +24,23 @@ const PLANTILLA: &str = "## Objetivo
 |---|---|---|---|---|
 ";
 
+/// Hueco del objetivo en la plantilla.
+const OBJETIVO_PLANTILLA: &str = "<que se quiere y por que>";
+/// Hueco del criterio en la plantilla.
+const CRITERIO_PLANTILLA: &str = "- [ ] <criterio comprobable>";
+
+/// Una linea de texto libre que no abre seccion ni cerca: si empieza (con
+/// hasta 3 espacios) por `#`, `` ` `` o `~`, se le antepone `\`.
+fn escapar_linea(linea: &str) -> String {
+    let sin = linea.trim_start_matches(' ');
+    let sangria = linea.len() - sin.len();
+    if sangria <= 3 && sin.starts_with(['#', '`', '~']) {
+        format!("{}\\{sin}", &linea[..sangria])
+    } else {
+        linea.to_owned()
+    }
+}
+
 /// Una seccion `## <titulo>` con su texto, incluida la linea del titulo.
 #[derive(Debug, Clone, PartialEq, Eq)]
 pub struct Seccion {
@@ -115,6 +132,38 @@ impl Cuerpo {
         Self::leer(PLANTILLA)
     }
 
+    /// La plantilla con el enunciado puesto (RF-01): `objetivo` en
+    /// `## Objetivo` y un `- [ ] <criterio>` por criterio. Lo que venga vacio
+    /// se queda con el texto de la plantilla. Una linea del objetivo que
+    /// abriria una seccion o una cerca de codigo se escapa con `\`: el texto
+    /// es dato no confiable y no puede cambiar la forma del cuerpo (F1-8).
+    pub fn con_enunciado(objetivo: &str, criterios: &[String]) -> Self {
+        // Se parte la plantilla por sus huecos antes de meter nada: un texto
+        // que contenga el hueco del criterio no desvia la sustitucion.
+        let partes = PLANTILLA
+            .split_once(OBJETIVO_PLANTILLA)
+            .and_then(|(a, resto)| resto.split_once(CRITERIO_PLANTILLA).map(|(b, c)| (a, b, c)));
+        let Some((antes, entre, despues)) = partes else {
+            return Self::plantilla();
+        };
+        let objetivo = if objetivo.trim().is_empty() {
+            OBJETIVO_PLANTILLA.to_owned()
+        } else {
+            let lineas: Vec<String> = objetivo.trim().lines().map(escapar_linea).collect();
+            lineas.join("\n")
+        };
+        let lista = if criterios.is_empty() {
+            CRITERIO_PLANTILLA.to_owned()
+        } else {
+            let v: Vec<String> = criterios
+                .iter()
+                .map(|c| format!("- [ ] {}", c.trim()))
+                .collect();
+            v.join("\n")
+        };
+        Self::leer(&format!("{antes}{objetivo}{entre}{lista}{despues}"))
+    }
+
     /// Texto del cuerpo: lo leido, byte a byte.
     pub fn escribir(&self) -> String {
         let mut s = self.preambulo.clone();
@@ -152,6 +201,50 @@ mod tests {
         assert_eq!(p.secciones.len(), 5);
     }
 
+    #[test]
+    fn enunciado_en_la_plantilla_sin_romper_las_secciones() {
+        assert_eq!(Cuerpo::con_enunciado("", &[]), Cuerpo::plantilla());
+        let c = Cuerpo::con_enunciado(
+            "Primera linea\n## no es seccion\n```\n  # tampoco",
+            &["uno".to_owned(), " dos ".to_owned()],
+        );
+        let titulos: Vec<&str> = c.secciones.iter().map(|s| s.titulo.as_str()).collect();
+        assert_eq!(
+            titulos,
+            [
+                "Objetivo",
+                "Criterios de aceptacion",
+                "Escenarios",
+                "Resultado",
+                "Transiciones"
+            ]
+        );
+        let objetivo = c.seccion("Objetivo").map(|s| s.texto.as_str());
+        assert_eq!(
+            objetivo,
+            Some("## Objetivo\n\nPrimera linea\n\\## no es seccion\n\\```\n  \\# tampoco\n\n")
+        );
+        let criterios = c
+            .seccion("Criterios de aceptacion")
+            .map(|s| s.texto.as_str());
+        assert_eq!(
+            criterios,
+            Some("## Criterios de aceptacion\n\n- [ ] uno\n- [ ] dos\n\n")
+        );
+        assert_eq!(escapar_linea("    # codigo"), "    # codigo");
+        // Un objetivo con el texto del hueco del criterio no lo desvia.
+        let c = Cuerpo::con_enunciado(
+            "ver - [ ] <criterio comprobable> aqui",
+            &["real".to_owned()],
+        );
+        let t = c.escribir();
+        assert!(
+            t.contains("\n\nver - [ ] <criterio comprobable> aqui\n\n"),
+            "{t}"
+        );
+        assert!(t.contains("\n\n- [ ] real\n\n"), "{t}");
+    }
+
     #[test]
     fn cercas_como_commonmark() {
         let titulos = |t: &str| -> Vec<String> {
diff --git a/crates/tc2/src/tarea/mod.rs b/crates/tc2/src/tarea/mod.rs
index 198b2ae..bd99610 100644
--- a/crates/tc2/src/tarea/mod.rs
+++ b/crates/tc2/src/tarea/mod.rs
@@ -24,6 +24,10 @@ pub use modelo::{
 };
 
 /// Un `tarea.md` leido.
+// Se lee uno a uno y se destructura enseguida: la diferencia de tamano entre
+// variantes (la clave `lote` de TASK-029 la paso de 200 bytes) no se nota, y
+// un `Box` cambiaria el patron de todos los que lo leen.
+#[allow(clippy::large_enum_variant)]
 #[derive(Debug, Clone, PartialEq, Eq)]
 pub enum Documento {
     /// Formato 1 (0.16), sin `formato`.
diff --git a/crates/tc2/src/tarea/modelo.rs b/crates/tc2/src/tarea/modelo.rs
index d0f8d7b..38b32b6 100644
--- a/crates/tc2/src/tarea/modelo.rs
+++ b/crates/tc2/src/tarea/modelo.rs
@@ -57,6 +57,11 @@ pub(crate) const CLAVES: [&str; 21] = [
     "actualizado",
     "dependencias",
 ];
+/// Claves opcionales del formato 2: si faltan valen `null` y solo se
+/// escriben con valor, despues de `dependencias` (adenda de TASK-029 a F1-9:
+/// una tarea de antes se lee y se reescribe byte a byte igual).
+pub(crate) const OPCIONALES: [&str; 1] = ["lote"];
+
 // Sin `zonas` (F1-17) ni `hito` (F1-18) ni el `roles` ampliado de F1-15:
 // hasta que los recoja la tarea que los usa, `zonas` y `hito` se conservan
 // como claves desconocidas.
@@ -325,6 +330,9 @@ pub struct Tarea {
     pub creado: String,
     pub actualizado: String,
     pub dependencias: Vec<String>,
+    /// `lote/clave` de la tarea importada que la creo (F3-2, D1 de TASK-029):
+    /// por ella un import repetido la reconoce. Opcional.
+    pub lote: Option<String>,
     /// Claves desconocidas con su texto crudo, en su orden (F1-9 D4a: se
     /// conservan en la ida y vuelta; `doctor` avisa). Privadas: solo llegan
     /// de una lectura, que las ha validado y comprobado sin duplicadas.
@@ -354,6 +362,7 @@ struct CrudoV2 {
     creado: Option<String>,
     actualizado: Option<String>,
     dependencias: Option<Vec<String>>,
+    lote: Option<String>,
 }
 
 /// Valor obligatorio (presente y no `null`).
@@ -406,6 +415,24 @@ fn id(clave: &str, s: String) -> Result<String, ErrorTarea> {
     }
 }
 
+/// `lote`: `<lote>/<clave>`, sin caracteres de control, o ausente/`null`.
+fn lote(s: Option<String>) -> Result<Option<String>, ErrorTarea> {
+    match s {
+        None => Ok(None),
+        Some(s)
+            if texto_valido(&s)
+                && s.split_once('/')
+                    .is_some_and(|(a, b)| !a.is_empty() && !b.is_empty()) =>
+        {
+            Ok(Some(s))
+        }
+        Some(s) => Err(ErrorTarea::en(
+            "lote",
+            format!("«{s}» no es <lote>/<clave>"),
+        )),
+    }
+}
+
 fn fecha(clave: &str, s: String) -> Result<String, ErrorTarea> {
     if fecha_valida(&s) {
         Ok(s)
@@ -478,6 +505,7 @@ impl Tarea {
             creado: fecha.to_owned(),
             actualizado: fecha.to_owned(),
             dependencias: Vec::new(),
+            lote: None,
             desconocidas: Vec::new(),
         }
     }
@@ -573,7 +601,8 @@ impl Tarea {
                 .into_iter()
                 .map(|d| id("dependencias", d))
                 .collect::<Result<_, _>>()?,
-            desconocidas: desconocidas(partes, &CLAVES),
+            lote: lote(c.lote)?,
+            desconocidas: desconocidas(partes, &[&CLAVES[..], &OPCIONALES[..]].concat()),
         })
     }
 
@@ -652,6 +681,9 @@ impl Tarea {
         linea("creado", yaml::cadena(&self.creado));
         linea("actualizado", yaml::cadena(&self.actualizado));
         linea("dependencias", yaml::lista(&self.dependencias));
+        if let Some(l) = &self.lote {
+            linea("lote", yaml::cadena(l));
+        }
         for (_, crudo) in &self.desconocidas {
             s.push_str(crudo);
         }
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-029/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-029/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-029/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````

## Tests del diff (2 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/alta.rs" ":(literal)crates/tc2/tests/importar.rs"
````

````
crates/tc2/tests/alta.rs     |  98 ++++++++++++++++++++++++++++++++
 crates/tc2/tests/importar.rs | 499 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 2 files changed, 597 insertions(+)
````
